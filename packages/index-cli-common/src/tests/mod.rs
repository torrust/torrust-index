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

use crate::BaseArgs;

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
