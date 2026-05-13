//! # Health-check integration tests
//!
//! | Test                                    | What it covers                          |
//! |-----------------------------------------|-----------------------------------------|
//! | `success_on_healthy_server`             | 200 OK → Ok with status 200            |
//! | `failure_on_sick_server`                | 503 → Err                              |
//! | `failure_on_unreachable`                | Connection refused → Err               |
//! | `failure_on_unsupported_scheme`         | https:// → Err                         |
//! | `output_contains_expected_fields`       | Output has target, status, elapsed_ms   |

use std::io::{Read, Write};
use std::net::TcpListener;

use torrust_index_health_check::{SCHEMA, do_health_check};

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
    // Drain a single read of the request line/headers. We don't
    // care about the exact byte count — the test only needs to
    // unblock the client so it sees our canned response — but we
    // do want to surface unexpected IO errors loudly.
    let mut buf = [0u8; 1024];
    let _bytes_read = stream.read(&mut buf).expect("failed to read request from test client");
    stream.write_all(response).unwrap();
    stream.flush().unwrap();
}

#[test]
fn success_on_healthy_server() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_ok());
    assert_eq!(result.unwrap().status, 200);
}

#[test]
fn failure_on_sick_server() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\n\r\n");
    let result = handle.join().unwrap();
    assert!(result.is_err());
}

#[test]
fn failure_on_unreachable() {
    let listener = TcpListener::bind("127.0.0.1:0").unwrap();
    let port = listener.local_addr().unwrap().port();
    drop(listener);
    let url = format!("http://127.0.0.1:{port}/health_check");
    let result = do_health_check(&url);
    assert!(result.is_err());
}

#[test]
fn failure_on_unsupported_scheme() {
    let result = do_health_check("https://localhost:3001/health_check");
    assert!(result.is_err());
}

#[test]
fn output_contains_expected_fields() {
    let (listener, url) = ephemeral_server();
    let handle = std::thread::spawn(move || do_health_check(&url));
    serve_once(&listener, b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\n\r\n");
    let output = handle.join().unwrap().unwrap();

    let json = serde_json::to_value(&output).unwrap();
    assert_eq!(json["schema"], SCHEMA);
    assert!(json.get("target").is_some(), "missing 'target' field");
    assert!(json.get("status").is_some(), "missing 'status' field");
    assert!(json.get("elapsed_ms").is_some(), "missing 'elapsed_ms' field");
    assert_eq!(json["status"], 200);
}
