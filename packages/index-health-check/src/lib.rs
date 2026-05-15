//! Minimal health-check library for Torrust Index containers.
//!
//! Uses only `std::net::TcpStream` — no async runtime, no TLS crate.

use std::io::{BufRead, BufReader, Write};
use std::net::{SocketAddr, TcpStream, ToSocketAddrs};
use std::sync::mpsc;
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;

#[cfg(test)]
mod tests;

/// Schema version of the health-check JSON output.
pub const SCHEMA: u32 = 1;

#[derive(Serialize)]
pub struct HealthCheckOutput {
    pub schema: u32,
    pub target: String,
    pub status: u16,
    pub elapsed_ms: u64,
}

#[derive(Debug)]
pub struct HealthCheckError(pub String);

impl std::fmt::Display for HealthCheckError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&self.0)
    }
}

/// # Errors
///
/// Returns an error if the URL is malformed, the connection fails,
/// or the server responds with a non-2xx status.
pub fn do_health_check(url: &str) -> Result<HealthCheckOutput, HealthCheckError> {
    let start = Instant::now();

    // Parse URL: expect http://host:port/path
    let stripped = url
        .strip_prefix("http://")
        .ok_or_else(|| HealthCheckError(format!("unsupported URL scheme: {url}")))?;

    let (host_port, path) = stripped
        .find('/')
        .map_or((stripped, "/"), |i| (&stripped[..i], &stripped[i..]));

    let timeout = Duration::from_secs(5);

    // Resolve `host:port` ourselves so we can apply an explicit
    // connect-timeout: `TcpStream::connect` falls back to the OS's
    // SYN-retransmit schedule (often tens of seconds), which is far
    // too long for a container orchestrator's health probe.
    //
    // `localhost` typically resolves to both `::1` and `127.0.0.1`.
    // Glibc returns IPv6 first, but the index API binds to
    // `0.0.0.0` (IPv4 only) and the tracker statistics importer to
    // `127.0.0.1`, so a probe that only tries the first address
    // gets `Connection refused` on `::1` and never reaches the
    // listening IPv4 socket. We follow Happy Eyeballs (RFC 8305):
    // prefer IPv6 first, but launch the IPv4 attempt after a
    // small delay so a black-holed (no-RST) IPv6 address cannot
    // burn the entire HEALTHCHECK budget. First success wins.
    let addrs: Vec<SocketAddr> = host_port
        .to_socket_addrs()
        .map_err(|e| HealthCheckError(format!("resolve {host_port}: {e}")))?
        .collect();
    if addrs.is_empty() {
        return Err(HealthCheckError(format!("resolve {host_port}: no addresses returned")));
    }

    let mut stream = happy_eyeballs_connect(host_port, &addrs, timeout)?;
    stream
        .set_read_timeout(Some(timeout))
        .map_err(|e| HealthCheckError(format!("set read timeout: {e}")))?;
    stream
        .set_write_timeout(Some(timeout))
        .map_err(|e| HealthCheckError(format!("set write timeout: {e}")))?;

    let request = format!("GET {path} HTTP/1.1\r\nHost: {host_port}\r\nConnection: close\r\n\r\n");
    stream
        .write_all(request.as_bytes())
        .map_err(|e| HealthCheckError(format!("write request: {e}")))?;
    stream.flush().map_err(|e| HealthCheckError(format!("flush request: {e}")))?;

    let mut reader = BufReader::new(&stream);
    let mut status_line = String::new();
    reader
        .read_line(&mut status_line)
        .map_err(|e| HealthCheckError(format!("read status line: {e}")))?;

    // Parse "HTTP/1.x NNN ..."
    let parts: Vec<&str> = status_line.splitn(3, ' ').collect();
    if parts.len() < 2 {
        return Err(HealthCheckError(format!("malformed status line: {status_line:?}")));
    }
    let status: u16 = parts[1]
        .parse()
        .map_err(|_| HealthCheckError(format!("invalid status code: {:?}", parts[1])))?;

    let elapsed_ms = u64::try_from(start.elapsed().as_millis()).unwrap_or(u64::MAX);

    if !(200..300).contains(&status) {
        return Err(HealthCheckError(format!("non-success status {status} from {url}")));
    }

    Ok(HealthCheckOutput {
        schema: SCHEMA,
        target: url.to_string(),
        status,
        elapsed_ms,
    })
}

/// Per-attempt launch stagger (RFC 8305 "Connection Attempt Delay").
///
/// 250 ms is the spec's default. Long enough that a successful
/// connect on the preferred family almost always finishes first;
/// short enough that a black-holed address can't burn the entire
/// HEALTHCHECK budget.
const CONNECT_ATTEMPT_DELAY: Duration = Duration::from_millis(250);

/// Connect to the first address that responds, preferring IPv6
/// (RFC 6724 / 8305 default).
///
/// Algorithm (Happy Eyeballs v2, lite):
///
/// 1. Split resolved addresses by family, preserving the
///    resolver's relative order within each family.
/// 2. Interleave them — `v6, v4, v6, v4, …` — so an IPv6
///    address is always attempted first.
/// 3. Spawn one connect thread per address, staggered by
///    `CONNECT_ATTEMPT_DELAY`. Each attempt is bounded by the
///    remaining time of the overall `total_timeout` budget.
/// 4. The first successful `TcpStream` wins; remaining
///    in-flight attempts are abandoned (the channel send on
///    a closed receiver is silently dropped).
///
/// Threads outlive this call only briefly — at worst until
/// their connect attempt completes against the bounded
/// per-attempt timeout. This is a CLI binary that exits
/// immediately after `do_health_check` returns, so any
/// background I/O is reaped by the OS.
fn happy_eyeballs_connect(host_port: &str, addrs: &[SocketAddr], total_timeout: Duration) -> Result<TcpStream, HealthCheckError> {
    // Single-address fast path: no scheduling overhead, no thread.
    if let [only] = addrs {
        return TcpStream::connect_timeout(only, total_timeout)
            .map_err(|e| HealthCheckError(format!("connect to {host_port} ({only}): {e}")));
    }

    // Split by family, preserving resolver order within each.
    let (v6, v4): (Vec<SocketAddr>, Vec<SocketAddr>) = addrs.iter().copied().partition(SocketAddr::is_ipv6);

    // Interleave: IPv6 first, then matching IPv4, etc.
    let mut ordered = Vec::with_capacity(addrs.len());
    let mut v6 = v6.into_iter();
    let mut v4 = v4.into_iter();
    loop {
        let a = v6.next();
        let b = v4.next();
        if a.is_none() && b.is_none() {
            break;
        }
        if let Some(x) = a {
            ordered.push(x);
        }
        if let Some(x) = b {
            ordered.push(x);
        }
    }

    let deadline = Instant::now() + total_timeout;
    let (tx, rx) = mpsc::channel::<Result<TcpStream, HealthCheckError>>();

    for (i, addr) in ordered.iter().copied().enumerate() {
        let tx = tx.clone();
        let host_port = host_port.to_string();
        // `i as u32` would clip silently for absurd address counts;
        // saturate explicitly so the stagger stays well-defined.
        let stagger = CONNECT_ATTEMPT_DELAY * u32::try_from(i).unwrap_or(u32::MAX);
        thread::spawn(move || {
            if !stagger.is_zero() {
                thread::sleep(stagger);
            }
            let remaining = deadline.saturating_duration_since(Instant::now());
            if remaining.is_zero() {
                // Past the overall budget; don't even try.
                drop(tx.send(Err(HealthCheckError(format!(
                    "connect to {host_port} ({addr}): deadline elapsed before attempt"
                )))));
                return;
            }
            let result = TcpStream::connect_timeout(&addr, remaining)
                .map_err(|e| HealthCheckError(format!("connect to {host_port} ({addr}): {e}")));
            // Receiver may already be gone (another address won).
            // The send error is informational only.
            drop(tx.send(result));
        });
    }
    drop(tx); // close the original sender so `recv_timeout` exits
    // cleanly once every spawned thread has finished.

    let mut last_err: Option<HealthCheckError> = None;
    loop {
        let remaining = deadline.saturating_duration_since(Instant::now());
        if remaining.is_zero() {
            break;
        }
        match rx.recv_timeout(remaining) {
            Ok(Ok(stream)) => return Ok(stream),
            Ok(Err(e)) => last_err = Some(e),
            Err(_) => break, // overall deadline elapsed or all senders gone
        }
    }
    Err(last_err.unwrap_or_else(|| HealthCheckError(format!("connect to {host_port}: no address could be reached"))))
}
