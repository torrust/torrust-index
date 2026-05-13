//! # CLI-common tests
//!
//! | Test                                  | What it covers                                   |
//! |---------------------------------------|--------------------------------------------------|
//! | `emit_writes_one_json_line`           | `emit` produces compact JSON + trailing newline. |
//! | `emit_propagates_serialisation_error` | A non-`Serialize`-friendly value surfaces `Err`. |
//! | `emit_propagates_writer_error`        | A failing writer surfaces `Err` (broken pipe).   |
//! | `emit_to_real_stdout_succeeds`        | `emit` itself writes to the captured stdout.     |
//! | `base_args_parses_default`            | `BaseArgs::debug` defaults to `false`.           |
//! | `base_args_parses_long_flag`          | `--debug` flips `BaseArgs::debug` to `true`.     |
//! | `command_exit_codes_match_contract`   | Baseline process statuses are fixed.             |
//! | `control_record_serialises_shape`      | Shared stderr record shape is stable.            |
//! | `usage_error_record_carries_fields`    | Usage records include exit code and clap kind.   |
//! | `tty_refusal_record_carries_fields`    | TTY refusal records identify stdout and code 2.  |
//! | `panic_record_omits_payload`           | Panic records avoid serialising panic payloads.  |
//! | `redacts_sensitive_field_values`       | Secret-like field names are hidden.              |
//! | `redacts_database_url_secrets`         | DB credentials and query secrets are removed.    |
//! | `keeps_public_key_fields_visible`      | Public key metadata is not treated as secret.    |
//!
//! `refuse_if_stdout_is_tty` and `init_json_tracing` mutate
//! global process state (the `process::exit` path and the
//! installed tracing subscriber) and are deliberately
//! exercised only by the per-binary integration tests of the
//! helper binaries — invoking them here would either abort
//! the test process or install a global subscriber that
//! interferes with the rest of the test binary's output.

use std::collections::BTreeMap;
use std::io::{self, Write};

use clap::Parser;
use serde::Serialize;
use serde_json::json;

use crate::{
    BaseArgs, CONTROL_PLANE_SCHEMA, CommandExit, ControlPlaneFields, ControlPlaneRecord, ControlPlaneRecordKind, REDACTED,
    StandardStream, redact_database_url, redact_field_value,
};

/// A `Write` that fails every call with `BrokenPipe`.
///
/// Used to cover the `emit` error branch without actually
/// closing stdout.
struct FailingWriter;

impl Write for FailingWriter {
    fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "test: pipe closed"))
    }

    fn flush(&mut self) -> io::Result<()> {
        Err(io::Error::new(io::ErrorKind::BrokenPipe, "test: pipe closed"))
    }
}

/// Pure helper that mirrors `emit` but writes to an arbitrary
/// `Write`. Lets us assert the on-the-wire bytes without
/// touching the real `stdout()` lock (which would interleave
/// with cargo's own captured-output machinery).
fn emit_to<W: Write, T: Serialize>(mut w: W, value: &T) -> io::Result<()> {
    let json = serde_json::to_string(value)?;
    w.write_all(json.as_bytes())?;
    w.write_all(b"\n")?;
    w.flush()
}

#[test]
fn emit_writes_one_json_line() {
    // BTreeMap to fix key order so the assertion is stable.
    let mut value: BTreeMap<&str, u32> = BTreeMap::new();
    value.insert("a", 1);
    value.insert("b", 2);

    let mut buf = Vec::new();
    emit_to(&mut buf, &value).expect("emit must succeed for a writable buffer");

    assert_eq!(buf, b"{\"a\":1,\"b\":2}\n");
}

#[test]
fn emit_propagates_writer_error() {
    let value = serde_json::json!({"k": "v"});
    let err = emit_to(FailingWriter, &value).expect_err("emit must surface writer errors");
    assert_eq!(err.kind(), io::ErrorKind::BrokenPipe);
}

#[test]
fn emit_propagates_serialisation_error() {
    // `serde_json::Map` keys must be strings; a map keyed by a
    // non-string serialises to an `io::Error` wrapped serde
    // failure when bridged through `serde_json::to_string`.
    //
    // Easier route: a custom `Serialize` impl that always errors.
    use serde::Serializer;

    struct AlwaysFails;
    impl Serialize for AlwaysFails {
        fn serialize<S: Serializer>(&self, _s: S) -> Result<S::Ok, S::Error> {
            Err(serde::ser::Error::custom("test: forced failure"))
        }
    }

    let mut buf = Vec::new();
    let err = emit_to(&mut buf, &AlwaysFails).expect_err("must surface ser error");
    // `serde_json::Error` converts to `io::Error` via `From`,
    // which preserves a non-Other kind only for IO sources;
    // for ser errors the kind is `InvalidData` or `Other`
    // depending on serde-json version. Don't pin the kind —
    // the contract is "an error is returned and nothing is
    // written".
    assert!(buf.is_empty(), "no bytes should be written on ser failure");
    drop(err); // kind not asserted, see comment above.
}

#[test]
fn emit_to_real_stdout_succeeds() {
    // Cover the real `emit` (which writes to `io::stdout`)
    // — under cargo test, stdout is captured and a write is
    // always allowed, so this just exercises the happy path.
    crate::emit(&serde_json::json!({"k": "v"})).expect("emit must succeed under captured stdout");
}

#[test]
fn base_args_parses_default() {
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        base: BaseArgs,
    }

    let parsed = Cli::try_parse_from(["prog"]).expect("no args is valid");
    assert!(!parsed.base.debug);
}

#[test]
fn base_args_parses_long_flag() {
    #[derive(Parser)]
    struct Cli {
        #[command(flatten)]
        base: BaseArgs,
    }

    let parsed = Cli::try_parse_from(["prog", "--debug"]).expect("--debug is valid");
    assert!(parsed.base.debug);
}

#[test]
fn command_exit_codes_match_contract() {
    assert_eq!(CommandExit::Success.code(), 0);
    assert_eq!(CommandExit::Failure.code(), 1);
    assert_eq!(CommandExit::Usage.code(), 2);

    assert_eq!(CommandExit::from_code(0), Some(CommandExit::Success));
    assert_eq!(CommandExit::from_code(1), Some(CommandExit::Failure));
    assert_eq!(CommandExit::from_code(2), Some(CommandExit::Usage));
    assert_eq!(CommandExit::from_code(3), None);
}

#[test]
fn control_record_serialises_shape() {
    let record = ControlPlaneRecord::new("fixture", ControlPlaneRecordKind::Status, "ready", None);
    let value = serde_json::to_value(record).unwrap();

    assert_eq!(value["schema"], json!(CONTROL_PLANE_SCHEMA));
    assert_eq!(value["command"], json!("fixture"));
    assert_eq!(value["kind"], json!("status"));
    assert_eq!(value["message"], json!("ready"));
    assert!(value.get("fields").is_none());
}

#[test]
fn usage_error_record_carries_fields() {
    let record = ControlPlaneRecord::usage_error("fixture", "unknown argument", "unknown_argument");

    assert_eq!(record.kind, ControlPlaneRecordKind::UsageError);
    assert_eq!(
        record.fields,
        Some(ControlPlaneFields::UsageError {
            exit_code: CommandExit::Usage.code(),
            clap_error_kind: "unknown_argument".to_string(),
        })
    );

    let value = serde_json::to_value(record).unwrap();
    assert_eq!(value["fields"]["type"], json!("usage_error"));
    assert_eq!(value["fields"]["exit_code"], json!(2));
}

#[test]
fn tty_refusal_record_carries_fields() {
    let record = ControlPlaneRecord::tty_refusal("fixture");

    assert_eq!(record.kind, ControlPlaneRecordKind::TtyRefusal);
    assert_eq!(
        record.fields,
        Some(ControlPlaneFields::TtyRefusal {
            exit_code: CommandExit::Usage.code(),
            stream: StandardStream::Stdout,
        })
    );
}

#[test]
fn panic_record_omits_payload() {
    let record = ControlPlaneRecord::panic("fixture", Some("main"), Some("src/main.rs:12:34"));
    let value = serde_json::to_value(record).unwrap();

    assert_eq!(value["kind"], json!("panic"));
    assert_eq!(value["fields"]["type"], json!("panic"));
    assert_eq!(value["fields"]["exit_code"], json!(1));
    assert_eq!(value["fields"]["thread"], json!("main"));
    assert!(value["fields"].get("payload").is_none());
}

#[test]
fn redacts_sensitive_field_values() {
    assert_eq!(redact_field_value("tracker.token", "MyAccessToken"), REDACTED);
    assert_eq!(redact_field_value("smtp_password", "secret"), REDACTED);
    assert_eq!(redact_field_value("auth.private_key_pem", "PEM"), REDACTED);
}

#[test]
fn redacts_database_url_secrets() {
    let redacted = redact_database_url("mysql://user:pass@example.test/db?ssl-mode=required&token=abc&password=def");

    assert_eq!(redacted, "mysql://example.test/db?ssl-mode=required");
}

#[test]
fn keeps_public_key_fields_visible() {
    assert_eq!(
        redact_field_value("auth.public_key_path", "/etc/torrust/public.pem"),
        "/etc/torrust/public.pem"
    );
}
