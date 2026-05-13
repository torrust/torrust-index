//! Shared CLI scaffolding for Torrust Index command-line tools (ADR-T-010).
//!
//! Every helper binary uses this crate for:
//! - JSON wrapping for `clap` help, version, and argv errors
//! - JSON control-plane records on stderr before tracing is installed
//! - TTY refusal for stdout result data
//! - JSON panic diagnostics without Rust's default text panic hook
//! - JSON tracing initialisation on stderr with `RUST_LOG` / `--debug` precedence
//! - JSON output on stdout
//! - small runners for stdout-producing and no-stdout commands

use std::borrow::Cow;
use std::ffi::{OsStr, OsString};
use std::io::{self, IsTerminal, Write};
use std::process::ExitCode;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Mutex, MutexGuard, TryLockError};

use clap::{CommandFactory, Parser};
use serde::{Deserialize, Serialize};
use tracing_subscriber::fmt::MakeWriter;

#[cfg(test)]
mod tests;

/// Schema version for shared ADR-T-010 control-plane records.
pub const CONTROL_PLANE_SCHEMA: u32 = 1;

/// Placeholder used when a diagnostic value is intentionally hidden.
pub const REDACTED: &str = "[redacted]";

static PANIC_REPORTED: AtomicBool = AtomicBool::new(false);
static STDERR_WRITE_LOCK: Mutex<()> = Mutex::new(());

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

    /// Build a non-error status record.
    #[must_use]
    pub fn status(command: &str, message: &str) -> Self {
        Self::new(command, ControlPlaneRecordKind::Status, message, None)
    }

    /// Build a runtime diagnostic record.
    #[must_use]
    pub fn diagnostic(command: &str, message: &str) -> Self {
        Self::new(command, ControlPlaneRecordKind::Diagnostic, message, None)
    }
}

/// Control-plane record and exit class produced while parsing argv.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CliExit {
    /// Record to write to stderr.
    pub record: ControlPlaneRecord,
    /// Process exit class to use after writing the record.
    pub exit: CommandExit,
}

impl CliExit {
    /// Build a new parse-control exit value.
    #[must_use]
    pub const fn new(record: ControlPlaneRecord, exit: CommandExit) -> Self {
        Self { record, exit }
    }

    /// Return this exit class as a standard library [`ExitCode`].
    #[must_use]
    pub fn exit_code(&self) -> ExitCode {
        self.exit.exit_code()
    }
}

/// Source used to choose the JSON tracing filter directive.
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TracingFilterSource {
    /// The `RUST_LOG` environment variable supplied the directive.
    RustLog,
    /// The shared `--debug` flag selected debug logging.
    DebugFlag,
    /// The caller-supplied default level was used.
    DefaultLevel,
}

/// Resolved JSON tracing filter directive.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TracingFilter {
    /// Directive passed to `tracing-subscriber`'s environment filter.
    pub directive: String,
    /// Input source that selected the directive.
    pub source: TracingFilterSource,
}

impl TracingFilter {
    /// Build a resolved tracing filter directive.
    #[must_use]
    pub fn new(directive: impl Into<String>, source: TracingFilterSource) -> Self {
        Self {
            directive: directive.into(),
            source,
        }
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

/// Resolve the JSON tracing filter with ADR-T-010 precedence.
///
/// `RUST_LOG` wins when set and non-empty. Otherwise `--debug` selects `debug`,
/// and the caller-supplied default level is used last.
#[must_use]
pub fn tracing_filter(debug: bool, default_level: tracing::Level) -> TracingFilter {
    tracing_filter_from_rust_log(std::env::var_os("RUST_LOG").as_deref(), debug, default_level)
}

fn tracing_filter_from_rust_log(rust_log: Option<&OsStr>, debug: bool, default_level: tracing::Level) -> TracingFilter {
    if let Some(value) = rust_log {
        let directive = value.to_string_lossy();
        let trimmed = directive.trim();
        if !trimmed.is_empty() {
            return TracingFilter::new(trimmed, TracingFilterSource::RustLog);
        }
    }

    if debug {
        TracingFilter::new("debug", TracingFilterSource::DebugFlag)
    } else {
        TracingFilter::new(level_directive(default_level), TracingFilterSource::DefaultLevel)
    }
}

fn level_directive(level: tracing::Level) -> String {
    level.as_str().to_ascii_lowercase()
}

/// Parse argv with `clap`, converting help, version, and usage errors into
/// ADR-T-010 JSON control-plane records.
///
/// # Errors
///
/// Returns [`CliExit`] when parsing should stop and the caller should write the
/// enclosed record to stderr before exiting with the enclosed exit class.
pub fn parse_args_from<T, I, A>(args: I) -> Result<T, CliExit>
where
    T: Parser,
    I: IntoIterator<Item = A>,
    A: Into<OsString> + Clone,
{
    T::try_parse_from(args).map_err(|clap_error| cli_exit_from_clap_error::<T>(&clap_error))
}

/// Parse process argv with `clap`, writing ADR-T-010 control-plane records and
/// exiting when parsing is a help, version, or usage-control path.
#[must_use]
pub fn parse_args_or_exit<T>() -> T
where
    T: Parser,
{
    match parse_args_from::<T, _, _>(std::env::args_os()) {
        Ok(args) => args,
        Err(exit) => {
            let _ignored = emit_control_plane_record(&exit.record);
            exit_with(exit.exit);
        }
    }
}

fn cli_exit_from_clap_error<T>(clap_error: &clap::Error) -> CliExit
where
    T: CommandFactory,
{
    let command_name = T::command().get_name().to_string();
    let text = clap_error.to_string().trim_end().to_string();

    match clap_error.kind() {
        clap::error::ErrorKind::DisplayHelp => CliExit::new(ControlPlaneRecord::help(&command_name, &text), CommandExit::Success),
        clap::error::ErrorKind::DisplayVersion => {
            CliExit::new(ControlPlaneRecord::version(&command_name, &text), CommandExit::Success)
        }
        _ => CliExit::new(
            ControlPlaneRecord::usage_error(&command_name, &text, &clap_error_kind_name(clap_error.kind())),
            CommandExit::Usage,
        ),
    }
}

fn clap_error_kind_name(kind: clap::error::ErrorKind) -> String {
    debug_name_to_snake_case(&format!("{kind:?}"))
}

fn debug_name_to_snake_case(name: &str) -> String {
    let mut normalized = String::with_capacity(name.len());
    let mut previous_was_lower_or_digit = false;

    for character in name.chars() {
        if character.is_ascii_uppercase() {
            if !normalized.is_empty() && previous_was_lower_or_digit {
                normalized.push('_');
            }
            normalized.push(character.to_ascii_lowercase());
            previous_was_lower_or_digit = false;
        } else {
            normalized.push(character);
            previous_was_lower_or_digit = character.is_ascii_lowercase() || character.is_ascii_digit();
        }
    }

    normalized
}

/// Write one ADR-T-010 control-plane JSON record to stderr.
///
/// This helper does not depend on `tracing`, so it is safe for clap help,
/// clap parse errors, early startup failures, TTY refusal, and panic hooks.
///
/// # Errors
///
/// Returns an error if serialisation or writing to stderr fails.
pub fn emit_control_plane_record(record: &ControlPlaneRecord) -> io::Result<()> {
    let mut writer = LockedStderrWriter::new();
    write_json_line(&mut writer, record)
}

fn try_emit_control_plane_record(record: &ControlPlaneRecord) -> io::Result<bool> {
    let Some(mut writer) = LockedStderrWriter::try_new() else {
        return Ok(false);
    };

    write_json_line(&mut writer, record)?;
    Ok(true)
}

fn write_json_line<W: Write, T: serde::Serialize>(writer: &mut W, value: &T) -> io::Result<()> {
    let json = serde_json::to_string(value)?;
    writer.write_all(json.as_bytes())?;
    writer.write_all(b"\n")?;
    writer.flush()
}

/// Install a JSON-only panic hook for ADR-T-010 command-line entrypoints.
///
/// The hook emits one best-effort control-plane record on stderr and terminates
/// the process with exit code 1 without invoking Rust's default text panic hook.
pub fn install_json_panic_hook(command_name: &str) {
    let command_name = command_name.to_string();
    std::panic::set_hook(Box::new(move |panic_info| {
        if !PANIC_REPORTED.swap(true, Ordering::SeqCst) {
            let current_thread = std::thread::current();
            let thread_name = current_thread.name();
            let location = panic_info
                .location()
                .map(|location| format!("{}:{}:{}", location.file(), location.line(), location.column()));
            let record = ControlPlaneRecord::panic(&command_name, thread_name, location.as_deref());
            let _ignored = try_emit_control_plane_record(&record);
        }

        exit_with(CommandExit::Failure);
    }));
}

/// Exit the current process with an ADR-T-010 exit class.
pub fn exit_with(exit: CommandExit) -> ! {
    std::process::exit(i32::from(exit.code()));
}

/// Return a TTY-refusal record when stdout is attached to a terminal.
#[must_use]
pub fn stdout_tty_refusal_record(command_name: &str) -> Option<ControlPlaneRecord> {
    if io::stdout().is_terminal() {
        Some(ControlPlaneRecord::tty_refusal(command_name))
    } else {
        None
    }
}

/// Refuse to run if stdout is a terminal (ADR-T-010).
///
/// Emits a JSON control-plane record on stderr and exits with code 2.
pub fn refuse_if_stdout_is_tty(binary_name: &str) {
    if let Some(record) = stdout_tty_refusal_record(binary_name) {
        let _ignored = emit_control_plane_record(&record);
        exit_with(CommandExit::Usage);
    }
}

/// Initialise `tracing-subscriber` with JSON output on stderr.
pub fn init_json_tracing(level: tracing::Level) {
    let _installed = try_init_json_tracing(level);
}

/// Try to initialise `tracing-subscriber` with JSON output on stderr.
///
/// Returns `false` if another subscriber is already installed.
#[must_use]
pub fn try_init_json_tracing(level: tracing::Level) -> bool {
    let filter = tracing_filter(false, level);
    try_init_json_tracing_with_directive(&filter.directive)
}

/// Initialise JSON stderr tracing with `RUST_LOG` / `--debug` precedence.
pub fn init_json_tracing_with_debug(debug: bool, default_level: tracing::Level) {
    let _installed = try_init_json_tracing_with_debug(debug, default_level);
}

/// Try to initialise JSON stderr tracing with `RUST_LOG` / `--debug` precedence.
///
/// Returns `false` if another subscriber is already installed.
#[must_use]
pub fn try_init_json_tracing_with_debug(debug: bool, default_level: tracing::Level) -> bool {
    let filter = tracing_filter(debug, default_level);
    try_init_json_tracing_with_directive(&filter.directive)
}

fn try_init_json_tracing_with_directive(filter_directive: &str) -> bool {
    let fallback_directive = level_directive(tracing::Level::INFO);
    let filter = tracing_subscriber::EnvFilter::try_new(filter_directive)
        .unwrap_or_else(|_error| tracing_subscriber::EnvFilter::new(fallback_directive));

    tracing_subscriber::fmt()
        .json()
        .with_env_filter(filter)
        .with_writer(LockedStderr)
        .try_init()
        .is_ok()
}

/// Serialise `value` as one JSON object + trailing newline to stdout.
///
/// # Errors
///
/// Returns an error if serialisation or writing to stdout fails.
pub fn emit<T: serde::Serialize>(value: &T) -> io::Result<()> {
    let stdout = io::stdout();
    let mut out = stdout.lock();
    write_json_line(&mut out, value)
}

/// Run a stdout-producing single-JSON-object command.
///
/// The runner installs the JSON panic hook, initialises JSON stderr tracing,
/// refuses terminal stdout, writes successful result data to stdout, and maps
/// failures to ADR-T-010 baseline exit classes.
pub fn run_stdout_json_command<Output, CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> ExitCode
where
    Output: serde::Serialize,
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<Output, CommandError>,
{
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    if let Some(record) = stdout_tty_refusal_record(command_name) {
        let _ignored = emit_control_plane_record(&record);
        return CommandExit::Usage.exit_code();
    }

    match run() {
        Ok(output) => match emit(&output) {
            Ok(()) => CommandExit::Success.exit_code(),
            Err(error) => {
                tracing::error!(error = %error, "failed to write JSON to stdout");
                CommandExit::Failure.exit_code()
            }
        },
        Err(error) => {
            tracing::error!(error = %error, "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}

/// Run a side-effect command that does not emit stdout result data.
///
/// The runner installs the JSON panic hook, initialises JSON stderr tracing, and
/// maps failures to ADR-T-010 baseline exit classes. It deliberately does not
/// perform stdout TTY refusal because the command has no stdout result data.
pub fn run_no_stdout_command<CommandError, Run>(
    command_name: &str,
    debug: bool,
    default_level: tracing::Level,
    run: Run,
) -> ExitCode
where
    CommandError: std::fmt::Display,
    Run: FnOnce() -> Result<(), CommandError>,
{
    install_json_panic_hook(command_name);
    init_json_tracing_with_debug(debug, default_level);

    match run() {
        Ok(()) => CommandExit::Success.exit_code(),
        Err(error) => {
            tracing::error!(error = %error, "command failed");
            CommandExit::Failure.exit_code()
        }
    }
}

struct LockedStderr;

impl<'writer> MakeWriter<'writer> for LockedStderr {
    type Writer = LockedStderrWriter;

    fn make_writer(&'writer self) -> Self::Writer {
        LockedStderrWriter::new()
    }
}

struct LockedStderrWriter {
    _guard: MutexGuard<'static, ()>,
    stderr: io::Stderr,
}

impl LockedStderrWriter {
    fn new() -> Self {
        Self {
            _guard: STDERR_WRITE_LOCK.lock().unwrap_or_else(std::sync::PoisonError::into_inner),
            stderr: io::stderr(),
        }
    }

    fn try_new() -> Option<Self> {
        match STDERR_WRITE_LOCK.try_lock() {
            Ok(guard) => Some(Self {
                _guard: guard,
                stderr: io::stderr(),
            }),
            Err(TryLockError::Poisoned(poisoned)) => Some(Self {
                _guard: poisoned.into_inner(),
                stderr: io::stderr(),
            }),
            Err(TryLockError::WouldBlock) => None,
        }
    }
}

impl Write for LockedStderrWriter {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        self.stderr.write(buf)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.stderr.flush()
    }
}

/// Common `--debug` flag for all helpers. Flatten into each
/// binary's `Args` struct via `#[command(flatten)]`.
#[derive(clap::Args)]
pub struct BaseArgs {
    /// Enable debug-level logging on stderr.
    #[arg(long)]
    pub debug: bool,
}
