//! # `validate_auth_keys` — host-side integration tests
//!
//! Drives the `validate_auth_keys` shell helper in
//! `share/container/entry_script_lib_sh` via `sh` subprocess
//! and asserts every branch of ADR-T-009 §7.1's invariants:
//!
//! | Test                              | Invariant exercised        |
//! |-----------------------------------|----------------------------|
//! | `both_none_passes`                | happy path — no config     |
//! | `both_path_passes`                | happy path — both PATH     |
//! | `both_pem_passes`                 | happy path — both PEM      |
//! | `private_pem_and_path_errors`     | JSON diagnostic for per-key mutual exclusion |
//! | `public_pem_and_path_errors`      | JSON diagnostic for per-key mutual exclusion |
//! | `private_only_errors`             | JSON diagnostic for pair completeness |
//! | `public_only_errors`              | JSON diagnostic for pair completeness |
//! | `mixed_pem_path_errors`           | JSON diagnostic for cross-pair source consistency |
//!
//! Argument order to `validate_auth_keys`:
//!
//! ```text
//! priv_pem_set priv_path_set priv_source
//! pub_pem_set  pub_path_set  pub_source
//! ```

use torrust_index_entry_script::{assert_entry_script_record, run_sh_with_args, single_stderr_json_record};

const SNIPPET: &str = "validate_auth_keys \"$@\"";

fn assert_pass(args: &[&str]) {
    let out = run_sh_with_args(SNIPPET, args);
    assert!(
        out.status.success(),
        "expected success for args {:?}; status={:?}; stderr={}",
        args,
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );
    assert!(
        out.stderr.is_empty(),
        "expected silent success for args {:?}; stderr={}",
        args,
        String::from_utf8_lossy(&out.stderr),
    );
}

fn assert_fail(args: &[&str], needle: &str) {
    let out = run_sh_with_args(SNIPPET, args);
    assert_eq!(
        out.status.code(),
        Some(1),
        "expected exit 1 for args {:?}; got {:?}; stderr={}",
        args,
        out.status.code(),
        String::from_utf8_lossy(&out.stderr),
    );
    let record = single_stderr_json_record(&out);
    assert_entry_script_record(&record, "diagnostic", "error", needle);
    assert_eq!(
        record.pointer("/fields/exit_code").and_then(serde_json::Value::as_u64),
        Some(1)
    );
}

#[test]
fn both_none_passes() {
    assert_pass(&["false", "false", "none", "false", "false", "none"]);
}

#[test]
fn both_path_passes() {
    assert_pass(&["false", "true", "path", "false", "true", "path"]);
}

#[test]
fn both_pem_passes() {
    assert_pass(&["true", "false", "pem", "true", "false", "pem"]);
}

#[test]
fn private_pem_and_path_errors() {
    assert_fail(&["true", "true", "pem", "false", "false", "none"], "mutually exclusive");
}

#[test]
fn public_pem_and_path_errors() {
    assert_fail(&["false", "false", "none", "true", "true", "pem"], "mutually exclusive");
}

#[test]
fn private_only_errors() {
    assert_fail(&["false", "true", "path", "false", "false", "none"], "complete pair");
}

#[test]
fn public_only_errors() {
    assert_fail(&["false", "false", "none", "false", "true", "path"], "complete pair");
}

#[test]
fn mixed_pem_path_errors() {
    assert_fail(&["true", "false", "pem", "false", "true", "path"], "mixed PEM/PATH");
}
