//! # Health-check tests
//!
//! | Test                               | What it covers                        |
//! |------------------------------------|---------------------------------------|
//! | `success_on_200`                   | Happy path: 200 OK response           |
//! | `success_output_carries_schema`     | Output schema field is stable         |
//! | `failure_on_non_2xx`               | Non-success HTTP status code          |
//! | `failure_on_connection_refused`     | Target not listening                  |
//! | `failure_on_read_timeout`            | Server accepts but never responds     |
//! | `failure_on_malformed_status_line`  | Server sends garbage                  |
//! | `localhost_falls_back_to_ipv4`      | `::1` refuses, `127.0.0.1` listens    |
//! | `ipv6_literal_url_is_supported`     | `http://[::1]:port/` parses + connects |
//! | `prefers_ipv6_when_both_listen`     | Dual-stack: IPv6 attempted first      |
//! | `error_display_carries_message`     | `HealthCheckError: Display`           |
//! | `unsupported_scheme_url_is_rejected` | `https://...` URL fails fast         |
//! | `malformed_url_no_host_fails`       | Resolver-failure branch               |

use std::io::{Read, Write};
use std::net::{IpAddr, TcpListener, ToSocketAddrs};

/// Bind an ephemeral-port listener and return (listener, url).
fn ephemeral_server() -> (TcpListener, String) {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let addr = listener.local_addr().unwrap();
    let url = format!("http://127.0.0.1:{}/health_check", addr.port());
    (listener, url)
}

/// Accept one connection, read the request, respond with `response`.
fn serve_once(listener: &TcpListener, response: &[u8]) {
    let (mut stream, _) = listener.accept().unwrap();
    let mut buf = [0u8; 1024];
    let _n = stream.read(&mut buf);
    stream.write_all(response).unwrap();
    stream.flush().unwrap();
}

#[test]
fn success_on_200() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_ok());
    let output = result.unwrap();
    assert_eq!(output.schema, super::SCHEMA);
    assert_eq!(output.status, 200);
}

#[test]
fn success_output_carries_schema() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let output = handle.join().unwrap().unwrap();
    assert_eq!(output.schema, super::SCHEMA);
}

#[test]
fn failure_on_non_2xx() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_err());
}

#[test]
fn failure_on_connection_refused() {
    // Use an ephemeral port that nothing is listening on.
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener); // Close it so connection is refused.
    let url = format!("http://127.0.0.1:{port}/health_check");
    let result = super::do_health_check(&url);
    assert!(result.is_err());
}

#[test]
fn failure_on_read_timeout() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    // Accept the connection but never send a response.
    let (_stream, _) = listener.accept().unwrap();
    let result = handle.join().unwrap();
    assert!(result.is_err());
}

#[test]
fn failure_on_malformed_status_line() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"GARBAGE\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_err());
}

/// Regression: the in-container probe targets `localhost`, which
/// glibc resolves to `::1` first and `127.0.0.1` second. The index
/// API binds to `0.0.0.0` (IPv4) and the importer to `127.0.0.1`,
/// so a probe that only tried the first resolved address would
/// fail with `Connection refused` on `::1` and never reach the
/// listening IPv4 socket. Bind an IPv4-only listener on the same
/// port `::1` rejects, then probe via a name that resolves to
/// both — the call must succeed by falling back to IPv4.
#[test]
fn localhost_falls_back_to_ipv4() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();

    // Sanity-check that `localhost` actually resolves to both
    // families on this host; if it doesn't (e.g. IPv6-disabled
    // CI runner), the test isn't exercising the regression path
    // and we skip rather than give a false pass.
    let mut saw_v6 = false;
    let mut saw_v4 = false;
    for a in ("localhost", port).to_socket_addrs().unwrap() {
        match a.ip() {
            IpAddr::V6(_) => saw_v6 = true,
            IpAddr::V4(_) => saw_v4 = true,
        }
    }
    if !(saw_v6 && saw_v4) {
        eprintln!("skipping: `localhost` does not resolve to both v6 and v4 here");
        return;
    }

    let url = format!("http://localhost:{port}/health_check");
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_ok(), "expected v4 fallback to succeed");
}

/// IPv6 literal in the URL must round-trip through both the
/// URL parser (which keeps the `[::1]:port` host token intact)
/// and `to_socket_addrs` (which understands bracketed literals).
#[test]
fn ipv6_literal_url_is_supported() {
    let listener = match TcpListener::bind("[::1]:0") {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping: cannot bind [::1]: {e}");
            return;
        }
    };
    let port = listener.local_addr().unwrap().port();
    let url = format!("http://[::1]:{port}/health_check");
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_ok(), "expected IPv6 literal probe to succeed");
}

/// Happy Eyeballs preference: when both families have a listener,
/// the IPv6 attempt fires first (no stagger) and should win the
/// race. Verified by binding only on `::1` and `127.0.0.1` to the
/// same ephemeral port and checking which one accepts. The IPv6
/// listener accepts; the IPv4 one stays idle.
#[test]
fn prefers_ipv6_when_both_listen() {
    use std::net::{Ipv4Addr, Ipv6Addr, SocketAddr};

    // Bind v6 first to discover an unused port; then bind the
    // matching v4 on the same port. If the kernel happens to be
    // configured with `bindv6only=0` and grabbed the v4 wildcard
    // already, the second bind will fail — skip in that case.
    let v6 = match TcpListener::bind(SocketAddr::from((Ipv6Addr::LOCALHOST, 0))) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping: cannot bind ::1: {e}");
            return;
        }
    };
    let port = v6.local_addr().unwrap().port();
    let v4 = match TcpListener::bind(SocketAddr::from((Ipv4Addr::LOCALHOST, port))) {
        Ok(l) => l,
        Err(e) => {
            eprintln!("skipping: cannot bind 127.0.0.1:{port}: {e}");
            return;
        }
    };

    // Sanity-check resolver order to make sure this host has both
    // families available for `localhost`.
    let mut saw_v6 = false;
    let mut saw_v4 = false;
    for a in ("localhost", port).to_socket_addrs().unwrap() {
        match a.ip() {
            IpAddr::V6(_) => saw_v6 = true,
            IpAddr::V4(_) => saw_v4 = true,
        }
    }
    if !(saw_v6 && saw_v4) {
        eprintln!("skipping: `localhost` does not resolve to both v6 and v4 here");
        return;
    }

    let url = format!("http://localhost:{port}/health_check");
    let handle = std::thread::spawn(move || super::do_health_check(&url));
    // The IPv6 listener fires first (no stagger). Accept on it;
    // the IPv4 listener intentionally never accepts.
    serve_once(&v6, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    drop(v4);
    assert!(result.is_ok(), "IPv6 should have won the connect race");
}

#[test]
fn error_display_carries_message() {
    let err = super::HealthCheckError("boom".to_string());
    assert_eq!(err.to_string(), "boom");
    // Smoke-check the `Debug` derive too \u2014 it surfaces the
    // wrapped string in error reporters.
    assert!(format!("{err:?}").contains("boom"));
}

#[test]
fn unsupported_scheme_url_is_rejected() {
    // The probe is HTTP-only on purpose (no TLS dependency).
    // Anything else \u2014 `https`, `unix`, `\xe2\x80\xa6` \u2014 fails fast at the
    // URL-parse stage.
    let result = super::do_health_check("https://example.com/health_check");
    let Err(err) = result else {
        panic!("https must be rejected");
    };
    assert!(err.to_string().contains("unsupported URL scheme"));
}

#[test]
fn unresolvable_host_surfaces_resolver_error() {
    // `.invalid` is reserved by RFC 6761 to never resolve, so
    // this exercises the `to_socket_addrs` error branch.
    // Use a `localhost`-ish form that includes a port so we
    // don't trip the URL parser before we reach the resolver.
    let result = super::do_health_check("http://nonexistent.invalid:65535/x");
    let Err(err) = result else {
        panic!("unresolvable host must error");
    };
    let msg = err.to_string();
    assert!(
        msg.contains("resolve") || msg.contains("connect"),
        "expected resolver/connect error, got: {msg}"
    );
}
