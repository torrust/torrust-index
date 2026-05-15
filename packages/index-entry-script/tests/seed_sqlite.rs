//! # `seed_sqlite` — host-side integration tests
//!
//! Drives the `seed_sqlite` shell helper in
//! `share/container/entry_script_lib_sh` via `sh` subprocess
//! and asserts every ADR-T-009 §7.2 outcome that does not
//! require root or writing to a managed container volume:
//!
//! | Test                           | Outcome                          |
//! |--------------------------------|----------------------------------|
//! | `empty_path_errors`            | exit 1 with JSON diagnostic      |
//! | `memory_path_skips`            | exit 0 with JSON info status     |
//! | `relative_path_skips`          | exit 0 with JSON warning status  |
//! | `nonempty_absolute_untouched`  | exit 0, file bytes unchanged     |
//! | `outside_volumes_errors`       | exit 1 with JSON diagnostic      |
//!
//! The "missing-under-volume seeded" outcome (mkdir + `inst()`
//! into `/var/lib/torrust/index/`) requires root and the
//! container's `torrust` user; it is exercised by the
//! container e2e suite.

use std::fs;

use tempfile::TempDir;
use torrust_index_entry_script::{assert_entry_script_record, run_sh_with_args, single_stderr_json_record};

const SNIPPET: &str = "seed_sqlite \"$1\"";

#[test]
fn empty_path_errors() {
    let out = run_sh_with_args(SNIPPET, &[""]);
    assert_eq!(out.status.code(), Some(1), "expected exit 1");
    let record = single_stderr_json_record(&out);
    assert_entry_script_record(&record, "diagnostic", "error", "database.path is empty");
    assert_eq!(
        record.pointer("/fields/exit_code").and_then(serde_json::Value::as_u64),
        Some(1)
    );
}

#[test]
fn memory_path_skips() {
    let out = run_sh_with_args(SNIPPET, &[":memory:"]);
    assert!(
        out.status.success(),
        "expected success; status={:?}; stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );
    let record = single_stderr_json_record(&out);
    assert_entry_script_record(&record, "status", "info", ":memory:");
}

#[test]
fn relative_path_skips() {
    let out = run_sh_with_args(SNIPPET, &["data/index.db"]);
    assert!(
        out.status.success(),
        "expected success; status={:?}; stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );
    let record = single_stderr_json_record(&out);
    assert_entry_script_record(&record, "status", "warn", "relative SQLite path");
}

#[test]
fn nonempty_absolute_untouched() {
    let dir = TempDir::new().expect("tempdir");
    let path = dir.path().join("existing.db");
    let payload = b"preexisting-content";
    fs::write(&path, payload).expect("write existing");

    let path_str = path.to_str().expect("tempdir path is utf-8");
    let out = run_sh_with_args(SNIPPET, &[path_str]);

    assert!(
        out.status.success(),
        "expected success; status={:?}; stderr={}",
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );

    let after = fs::read(&path).expect("read after");
    assert_eq!(after, payload, "non-empty existing file must not be touched");
}

#[test]
fn outside_volumes_errors() {
    let dir = TempDir::new().expect("tempdir");
    // Force the parent-doesn't-exist branch so seed_sqlite
    // hits the volumes-only guard rather than trying to
    // install into an existing tempdir.
    let path = dir.path().join("sub").join("missing.db");
    let path_str = path.to_str().expect("tempdir path is utf-8");

    let out = run_sh_with_args(SNIPPET, &[path_str]);

    assert_eq!(out.status.code(), Some(1), "expected exit 1");
    let record = single_stderr_json_record(&out);
    assert_entry_script_record(&record, "diagnostic", "error", "outside the volumes");
    assert_eq!(
        record.pointer("/fields/exit_code").and_then(serde_json::Value::as_u64),
        Some(1)
    );
}
