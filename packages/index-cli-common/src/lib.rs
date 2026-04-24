//! Shared CLI scaffolding for Torrust Index helper binaries (P9).
//!
//! Every helper binary uses this crate for:
//! - TTY refusal (P8)
//! - JSON tracing initialisation on stderr
//! - JSON output on stdout

use std::io::{self, IsTerminal, Write};

/// Refuse to run if stdout is a terminal (P8).
///
/// Emits a `tracing::error!` event (NDJSON on stderr, per P9)
/// and exits with code 2. Call this **after** [`init_json_tracing`]
/// so the diagnostic is structured rather than a bare `eprintln!`.
pub fn refuse_if_stdout_is_tty(binary_name: &str) {
    if io::stdout().is_terminal() {
        tracing::error!(
            binary = binary_name,
            "stdout is a terminal \u{2014} pipe to a file or another process"
        );
        std::process::exit(2);
    }
}

/// Initialise `tracing-subscriber` with JSON output on stderr.
pub fn init_json_tracing(level: tracing::Level) {
    tracing_subscriber::fmt()
        .json()
        .with_max_level(level)
        .with_writer(io::stderr)
        .init();
}

/// Serialise `value` as one JSON object + trailing newline to stdout.
///
/// # Errors
///
/// Returns an error if serialisation or writing to stdout fails.
pub fn emit<T: serde::Serialize>(value: &T) -> io::Result<()> {
    let json = serde_json::to_string(value)?;
    let stdout = io::stdout();
    let mut out = stdout.lock();
    out.write_all(json.as_bytes())?;
    out.write_all(b"\n")?;
    out.flush()
}

/// Common `--debug` flag for all helpers. Flatten into each
/// binary's `Args` struct via `#[command(flatten)]`.
#[derive(clap::Args)]
pub struct BaseArgs {
    /// Enable debug-level logging on stderr.
    #[arg(long)]
    pub debug: bool,
}
