//! # `seed_sqlite` — host-side integration tests
//!
//! Drives the `seed_sqlite` shell helper in
//! `share/container/entry_script_lib_sh` via `sh` subprocess
//! and asserts every ADR-T-009 §7.2 outcome that does not
//! require root or writing to a managed container volume:
//!
//! | Test                           | Outcome                          |
//! |--------------------------------|----------------------------------|
//! | `empty_path_errors`            | exit 1 with "database.path is empty" |
//! | `memory_path_skips`            | exit 0 with INFO line            |
//! | `relative_path_skips`          | exit 0 with WARN line            |
//! | `nonempty_absolute_untouched`  | exit 0, file bytes unchanged     |
//! | `outside_volumes_errors`       | exit 1 with "outside the … volumes" |
//!
//! The "missing-under-volume seeded" outcome (mkdir + `inst()`
//! into `/var/lib/torrust/index/`) requires root and the
//! container's `torrust` user; it is exercised by the
//! container e2e suite (Phase 8/9).

use std::fs;

use tempfile::TempDir;
use torrust_index_entry_script::run_sh_with_args;

const SNIPPET: &str = "seed_sqlite \"$1\"";

#[test]
fn empty_path_errors() {
    let out = run_sh_with_args(SNIPPET, &[""]);
    assert_eq!(out.status.code(), Some(1), "expected exit 1");
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("database.path is empty"),
        "missing diagnostic; stderr={stderr}",
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("INFO") && stderr.contains(":memory:"),
        "missing INFO/:memory: line; stderr={stderr}",
    );
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("WARN") && stderr.contains("relative SQLite path"),
        "missing WARN/relative line; stderr={stderr}",
    );
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
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        stderr.contains("outside the") && stderr.contains("volumes"),
        "missing volumes-guard diagnostic; stderr={stderr}",
    );
}
