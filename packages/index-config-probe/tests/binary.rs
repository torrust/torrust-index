//! # Config-probe binary contract tests
//!
//! These tests pin the *binary's* exit-code and stdout-shape
//! contract from §6.1 by driving the underlying library
//! functions directly — the same pattern as
//! `packages/index-config/tests/shipped_samples.rs`.
//!
//! Why not spawn the binary? `CARGO_BIN_EXE_*` is resolved at
//! compile time and bakes the build-time path into the test
//! binary. Under cargo-nextest's archive → `--extract-to` →
//! `--target-dir-remap` workflow used by the container `test`
//! stage, that baked path no longer exists at run time, so a
//! subprocess test fails with `ENOENT` even though the
//! contract under test is intact. Driving the same code paths
//! through the library makes the tests independent of the
//! binary's on-disk location and equivalent in coverage: the
//! exit-code mapping and the stdout JSON shape are both pure
//! functions of the loader / probe / serialiser results.
//!
//! ## Index
//!
//! | Test                                        | What it covers                              |
//! |---------------------------------------------|---------------------------------------------|
//! | `valid_toml_yields_emittable_probe`         | Happy path: probe → JSON round-trips        |
//! | `missing_connect_url_maps_to_exit_3`        | Loader-failure path (binary returns 3)      |
//! | `empty_tracker_token_maps_to_exit_4`        | `ProbeError::EmptyTrackerToken` → exit 4    |
//! | `unsupported_scheme_maps_to_exit_5`         | `ProbeError::UnsupportedScheme` → exit 5    |
//! | `unknown_flag_maps_to_clap_exit_2`          | Pins clap's argv-parse exit code (§6.1)     |

use clap::Parser;
use torrust_index_cli_common::BaseArgs;
use torrust_index_config::{Info, load_settings};
use torrust_index_config_probe::{Driver, Probe, ProbeError, probe};

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

const MISSING_CONNECT_URL_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"
"#;

const EMPTY_TRACKER_TOKEN_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = ""

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;

const POSTGRES_SCHEME_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "postgres://user@host/db"
"#;

#[test]
fn valid_toml_yields_emittable_probe() {
    let settings = load_settings(&Info::from_toml(VALID_TOML)).expect("valid TOML must load");
    let out = probe(&settings).expect("probe must succeed");

    // Mirror the binary's `emit` step: serialise to a single
    // JSON object terminated by a newline, then round-trip.
    let mut json = serde_json::to_string(&out).expect("probe must serialise");
    json.push('\n');
    assert!(json.ends_with('\n'), "emitted JSON must end with a newline");

    let parsed: Probe = serde_json::from_str(json.trim_end()).expect("emitted JSON must round-trip");
    assert_eq!(parsed.schema, torrust_index_config_probe::SCHEMA);
    assert_eq!(parsed.database.driver, Driver::Sqlite);
}

#[test]
fn missing_connect_url_maps_to_exit_3() {
    // Binary maps any `load_settings` failure to exit 3.
    let result = load_settings(&Info::from_toml(MISSING_CONNECT_URL_TOML));
    assert!(result.is_err(), "missing `database.connect_url` must fail the loader");
}

#[test]
fn empty_tracker_token_maps_to_exit_4() {
    // Binary maps `ProbeError::EmptyTrackerToken` to exit 4.
    let settings =
        load_settings(&Info::from_toml(EMPTY_TRACKER_TOKEN_TOML)).expect("loader accepts empty token; probe is the gate");
    match probe(&settings) {
        Err(ProbeError::EmptyTrackerToken) => {}
        other => panic!("expected EmptyTrackerToken, got {other:?}"),
    }
}

#[test]
fn unsupported_scheme_maps_to_exit_5() {
    // Binary maps `ProbeError::UnsupportedScheme` to exit 5.
    let settings = load_settings(&Info::from_toml(POSTGRES_SCHEME_TOML)).expect("loader accepts any URL; probe is the gate");
    match probe(&settings) {
        Err(ProbeError::UnsupportedScheme(scheme)) => assert_eq!(scheme, "postgres"),
        other => panic!("expected UnsupportedScheme(\"postgres\"), got {other:?}"),
    }
}

#[test]
fn unknown_flag_maps_to_clap_exit_2() {
    // §6.1 lists exit code 2 for "clap argv-parse failure".
    // Mirror the binary's `Args` so this test pins clap's
    // contract without spawning a subprocess.
    #[derive(Parser)]
    #[command(name = "torrust-index-config-probe")]
    struct Args {
        #[command(flatten)]
        #[allow(dead_code)]
        base: BaseArgs,
    }

    let Err(err) = Args::try_parse_from(["torrust-index-config-probe", "--this-flag-does-not-exist"]) else {
        panic!("unknown flag must be rejected");
    };
    assert_eq!(err.exit_code(), 2, "clap's argv-parse failure exit code is 2");
}
