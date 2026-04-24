//! # Config-probe binary integration tests
//!
//! | Test                                       | What it covers                                |
//! |--------------------------------------------|-----------------------------------------------|
//! | `binary_emits_json_for_inline_toml`        | Happy-path: stdout is one JSON object         |
//! | `binary_exits_3_on_missing_connect_url`    | Loader-failure path                           |
//! | `binary_exits_4_on_empty_tracker_token`    | Empty-token guard                             |
//! | `binary_exits_5_on_postgres_scheme`        | Unsupported scheme                            |
//! | `binary_exits_2_on_unknown_flag`           | Pins clap's argv-parse exit code (P8 §6.1)    |
//!
//! These tests invoke the compiled probe binary via Cargo so the
//! TTY refusal, exit codes, and stdout shape are all exercised
//! end-to-end.

use std::process::{Command, Stdio};

use torrust_index_config_probe::{Driver, Probe};

fn probe_command(toml: &str) -> Command {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_torrust-index-config-probe"));
    cmd.env("TORRUST_INDEX_CONFIG_TOML", toml);
    cmd.stdin(Stdio::null());
    cmd.stdout(Stdio::piped());
    cmd.stderr(Stdio::piped());
    cmd
}

const VALID_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;

#[test]
fn binary_emits_json_for_inline_toml() {
    let out = probe_command(VALID_TOML).output().expect("probe binary must run");
    assert!(
        out.status.success(),
        "expected success; stderr={}",
        String::from_utf8_lossy(&out.stderr)
    );

    let stdout = String::from_utf8(out.stdout).expect("stdout must be utf-8");
    assert!(stdout.ends_with('\n'), "stdout must end with a newline");

    let parsed: Probe = serde_json::from_str(stdout.trim_end()).expect("stdout must be one JSON object");
    assert_eq!(parsed.schema, torrust_index_config_probe::SCHEMA);
    assert_eq!(parsed.database.driver, Driver::Sqlite);
}

#[test]
fn binary_exits_3_on_missing_connect_url() {
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"
"#;
    let out = probe_command(toml).output().expect("probe binary must run");
    assert_eq!(out.status.code(), Some(3));
    assert!(out.stdout.is_empty(), "stdout must be empty on failure");
}

#[test]
fn binary_exits_4_on_empty_tracker_token() {
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = ""

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;
    let out = probe_command(toml).output().expect("probe binary must run");
    assert_eq!(out.status.code(), Some(4));
    assert!(out.stdout.is_empty(), "stdout must be empty on failure");
}

#[test]
fn binary_exits_5_on_postgres_scheme() {
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "postgres://user@host/db"
"#;
    let out = probe_command(toml).output().expect("probe binary must run");
    assert_eq!(out.status.code(), Some(5));
    assert!(out.stdout.is_empty(), "stdout must be empty on failure");
}

#[test]
fn binary_exits_2_on_unknown_flag() {
    // §6.1 lists exit code 2 for "clap argv-parse failure"
    // alongside the TTY refusal. Pin clap's behaviour so a
    // future upstream change can't silently violate the
    // contract. We don't pre-set TORRUST_INDEX_CONFIG_TOML
    // because clap rejects argv before the loader runs.
    let out = Command::new(env!("CARGO_BIN_EXE_torrust-index-config-probe"))
        .arg("--this-flag-does-not-exist")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output()
        .expect("probe binary must run");
    assert_eq!(out.status.code(), Some(2));
    assert!(out.stdout.is_empty(), "stdout must be empty on failure");
}
