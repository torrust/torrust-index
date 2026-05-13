//! Shared CLI scaffolding for Torrust Index command-line tools (ADR-T-010).
//!
//! Every helper binary uses this crate for:
//! - TTY refusal for stdout result data
//! - JSON tracing initialisation on stderr
//! - JSON output on stdout

use std::borrow::Cow;
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;

use serde::{Deserialize, Serialize};

#[cfg(test)]
mod tests;

/// Schema version for shared ADR-T-010 control-plane records.
pub const CONTROL_PLANE_SCHEMA: u32 = 1;

/// Placeholder used when a diagnostic value is intentionally hidden.
pub const REDACTED: &str = "[redacted]";

/// Baseline exit-code classes shared by ADR-T-010 command-line tools.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CommandExit {
    /// Successful completion.
    Success,
    /// Runtime, startup, internal, or command execution failure.
    Failure,
    /// Command-line usage failure, including clap errors and TTY refusal.
    Usage,
}

impl CommandExit {
    /// Return the numeric process status for this exit class.
    #[must_use]
    pub const fn code(self) -> u8 {
        match self {
            Self::Success => 0,
            Self::Failure => 1,
            Self::Usage => 2,
        }
    }

    /// Return this class as a standard library [`ExitCode`].
    #[must_use]
    pub fn exit_code(self) -> ExitCode {
        ExitCode::from(self.code())
    }

    /// Map a numeric status back to a shared exit class.
    #[must_use]
    pub const fn from_code(code: u8) -> Option<Self> {
        match code {
            0 => Some(Self::Success),
            1 => Some(Self::Failure),
            2 => Some(Self::Usage),
            _ => None,
        }
    }
}

/// JSON record kinds emitted on stderr as ADR-T-010 control-plane data.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ControlPlaneRecordKind {
    /// Help text requested by the caller.
    Help,
    /// Version information requested by the caller.
    Version,
    /// Command-line usage or argv parsing failure.
    UsageError,
    /// Refusal to write stdout result data directly to a terminal.
    TtyRefusal,
    /// Panic diagnostic emitted by the shared panic hook.
    Panic,
    /// Non-error status update.
    Status,
    /// Runtime diagnostic or error.
    Diagnostic,
}

/// Standard streams referenced by control-plane records.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StandardStream {
    /// Standard output.
    Stdout,
    /// Standard error.
    Stderr,
}

/// Structured fields attached to a control-plane record.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ControlPlaneFields {
    /// Help text returned by clap or a command-specific renderer.
    Help { text: String },
    /// Version string returned by clap or the command.
    Version { version: String },
    /// Details for an argv parsing or usage error.
    UsageError { exit_code: u8, clap_error_kind: String },
    /// Details for stdout TTY refusal.
    TtyRefusal { exit_code: u8, stream: StandardStream },
    /// Details for a panic diagnostic. The panic payload is deliberately omitted.
    Panic {
        exit_code: u8,
        thread: Option<String>,
        location: Option<String>,
    },
}

/// Shared JSON control-plane record written to stderr.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ControlPlaneRecord {
    /// Record schema version.
    pub schema: u32,
    /// Binary or entrypoint name.
    pub command: String,
    /// Machine-readable record kind.
    pub kind: ControlPlaneRecordKind,
    /// Short human-readable message, still carried inside JSON.
    pub message: String,
    /// Kind-specific structured fields.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fields: Option<ControlPlaneFields>,
}

impl ControlPlaneRecord {
    /// Build a generic control-plane record.
    #[must_use]
    pub fn new(command: &str, kind: ControlPlaneRecordKind, message: &str, fields: Option<ControlPlaneFields>) -> Self {
        Self {
            schema: CONTROL_PLANE_SCHEMA,
            command: command.to_string(),
            kind,
            message: message.to_string(),
            fields,
        }
    }

    /// Build a help record.
    #[must_use]
    pub fn help(command: &str, text: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Help,
            "help requested",
            Some(ControlPlaneFields::Help { text: text.to_string() }),
        )
    }

    /// Build a version record.
    #[must_use]
    pub fn version(command: &str, version: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Version,
            "version requested",
            Some(ControlPlaneFields::Version {
                version: version.to_string(),
            }),
        )
    }

    /// Build an argv usage-error record.
    #[must_use]
    pub fn usage_error(command: &str, message: &str, clap_error_kind: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::UsageError,
            message,
            Some(ControlPlaneFields::UsageError {
                exit_code: CommandExit::Usage.code(),
                clap_error_kind: clap_error_kind.to_string(),
            }),
        )
    }

    /// Build a stdout TTY-refusal record.
    #[must_use]
    pub fn tty_refusal(command: &str) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::TtyRefusal,
            "stdout is a terminal; pipe to a file or another process",
            Some(ControlPlaneFields::TtyRefusal {
                exit_code: CommandExit::Usage.code(),
                stream: StandardStream::Stdout,
            }),
        )
    }

    /// Build a panic diagnostic record.
    #[must_use]
    pub fn panic(command: &str, thread: Option<&str>, location: Option<&str>) -> Self {
        Self::new(
            command,
            ControlPlaneRecordKind::Panic,
            "unexpected panic",
            Some(ControlPlaneFields::Panic {
                exit_code: CommandExit::Failure.code(),
                thread: thread.map(str::to_string),
                location: location.map(str::to_string),
            }),
        )
    }
}

/// Return true when a diagnostic field name is likely to contain a secret.
#[must_use]
pub fn is_sensitive_field_name(field_name: &str) -> bool {
    const SENSITIVE_MARKERS: &[&str] = &[
        "admin_token",
        "api_key",
        "apikey",
        "credential",
        "credentials",
        "jwt_secret",
        "mailer_password",
        "passwd",
        "password",
        "private_key",
        "pwd",
        "secret",
        "session_secret",
        "smtp_password",
        "token",
    ];

    let normalized = normalize_name(field_name);
    SENSITIVE_MARKERS.iter().any(|marker| normalized.contains(marker))
}

/// Redact a diagnostic field value according to the ADR-T-010 policy.
#[must_use]
pub fn redact_field_value<'a>(field_name: &str, value: &'a str) -> Cow<'a, str> {
    if is_sensitive_field_name(field_name) {
        Cow::Borrowed(REDACTED)
    } else {
        redact_database_url(value)
    }
}

/// Redact credentials and secret query parameters from a database URL.
#[must_use]
pub fn redact_database_url(value: &str) -> Cow<'_, str> {
    const DATABASE_SCHEMES: &[&str] = &["mariadb", "mysql", "postgres", "postgresql", "sqlite"];

    let Ok(mut url) = url::Url::parse(value) else {
        return Cow::Borrowed(value);
    };

    if !DATABASE_SCHEMES.contains(&url.scheme()) {
        return Cow::Borrowed(value);
    }

    let username_changed = !url.username().is_empty() && url.set_username("").is_ok();
    let password_changed = url.password().is_some() && url.set_password(None).is_ok();

    let mut safe_query_pairs = Vec::new();
    let mut removed_query_pair = false;
    for (key, query_value) in url.query_pairs() {
        if is_sensitive_query_key(&key) {
            removed_query_pair = true;
        } else {
            safe_query_pairs.push((key.into_owned(), query_value.into_owned()));
        }
    }

    if removed_query_pair {
        url.set_query(None);
        {
            let mut query_pairs = url.query_pairs_mut();
            for (key, query_value) in safe_query_pairs {
                query_pairs.append_pair(&key, &query_value);
            }
        }
    }

    let changed = username_changed || password_changed || removed_query_pair;

    if changed {
        Cow::Owned(url.to_string())
    } else {
        Cow::Borrowed(value)
    }
}

fn normalize_name(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            normalized.push(ch.to_ascii_lowercase());
        } else {
            normalized.push('_');
        }
    }
    normalized
}

fn is_sensitive_query_key(key: &str) -> bool {
    let normalized = normalize_name(key);
    normalized == "key" || normalized.ends_with("_key") || is_sensitive_field_name(key)
}

/// Refuse to run if stdout is a terminal (ADR-T-010).
///
/// Emits a `tracing::error!` event (NDJSON on stderr, per ADR-T-010)
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
