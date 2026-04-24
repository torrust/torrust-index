//! Tests for [`crate::load_settings`].
//!
//! ## Index
//!
//! | Test                                              | What it proves                                                       |
//! |---------------------------------------------------|----------------------------------------------------------------------|
//! | `minimum_valid_toml_loads_with_defaults`          | The shortest legal config produces a fully-populated `Settings`.     |
//! | `missing_logging_threshold_is_rejected`           | `MissingMandatoryOption("logging.threshold")` fires before defaults. |
//! | `missing_metadata_schema_version_is_rejected`     | `MissingMandatoryOption("metadata.schema_version")` fires.           |
//! | `missing_tracker_token_is_rejected`               | Absent `tracker.token` surfaces as a serde missing-field error.      |
//! | `missing_database_connect_url_is_rejected`        | Absent `database.connect_url` surfaces as a serde missing-field err. |
//! | `missing_database_section_is_rejected`            | Absent `[database]` section surfaces as a serde missing-field err.   |
//! | `unsupported_schema_version_is_rejected`          | A `1.0.0` schema is refused via `UnsupportedVersion`.                |
//! | `wrong_field_type_surfaces_as_config_error`       | A type mismatch in an optional field surfaces as `ConfigError`.      |
//! | `malformed_toml_collapses_to_missing_mandatory`   | Documents the quirk: unparseable TOML reads as "all keys missing".   |
//! | `info_from_toml_round_trips_through_load`         | `Info::from_toml` is the canonical hermetic entry point.             |
//! | `from_figment_error_into_config_error_preserves`  | The `From<figment::Error>` impl wires the source through.            |

use crate::tests::{MINIMUM_VALID_TOML, info_from, placeholder_settings};
use crate::{Error, Version, load_settings};

#[test]
fn minimum_valid_toml_loads_with_defaults() {
    let settings = load_settings(&info_from(MINIMUM_VALID_TOML)).expect("minimum TOML must load");

    // Defaults filled in:
    assert_eq!(settings.api.default_torrent_page_size, 10);
    assert_eq!(settings.image_cache.capacity, 128_000_000);
    assert_eq!(settings.tracker_statistics_importer.port, 3002);
    // Mandatory options preserved:
    assert_eq!(settings.tracker.token.to_string(), "MyAccessToken");
    assert_eq!(settings.database.connect_url.as_str(), "sqlite://data.db?mode=rwc");
    assert_eq!(settings.metadata.schema_version, Version::default());
}

#[test]
fn missing_logging_threshold_is_rejected() {
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::MissingMandatoryOption { path }) => assert_eq!(path, "logging.threshold"),
        other => panic!("expected MissingMandatoryOption(logging.threshold), got {other:?}"),
    }
}

#[test]
fn missing_metadata_schema_version_is_rejected() {
    let toml = r#"
[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::MissingMandatoryOption { path }) => assert_eq!(path, "metadata.schema_version"),
        other => panic!("expected MissingMandatoryOption(metadata.schema_version), got {other:?}"),
    }
}

#[test]
fn missing_tracker_token_is_rejected() {
    // Per ADR-T-009 §D2, `tracker.token` carries no schema-level
    // default — the absence surfaces as a serde missing-field error
    // (wrapped as `ConfigError`), not via `check_mandatory_options`.
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::ConfigError { source }) => {
            let msg = source.to_string();
            assert!(msg.contains("token"), "expected 'token' in error, got: {msg}");
        }
        other => panic!("expected ConfigError(missing field 'token'), got {other:?}"),
    }
}

#[test]
fn missing_database_connect_url_is_rejected() {
    // Per ADR-T-009 §D2, `database.connect_url` carries no
    // schema-level default — the absence surfaces as a serde
    // missing-field error (wrapped as `ConfigError`).
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::ConfigError { source }) => {
            let msg = source.to_string();
            assert!(msg.contains("connect_url"), "expected 'connect_url' in error, got: {msg}");
        }
        other => panic!("expected ConfigError(missing field 'connect_url'), got {other:?}"),
    }
}

#[test]
fn missing_database_section_is_rejected() {
    // The `[database]` section itself carries no `#[serde(default)]`,
    // so omitting it entirely fails the same way as omitting an
    // inner mandatory key (ADR-T-009 §D2, "explicit failure forces
    // an explicit choice").
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::ConfigError { source }) => {
            let msg = source.to_string();
            assert!(msg.contains("database"), "expected 'database' in error, got: {msg}");
        }
        other => panic!("expected ConfigError(missing field 'database'), got {other:?}"),
    }
}

#[test]
fn unsupported_schema_version_is_rejected() {
    let toml = r#"
[metadata]
schema_version = "1.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::UnsupportedVersion { version }) => {
            // `Version` round-trips through Display.
            let rendered = format!("{version:?}");
            assert!(rendered.contains("1.0.0"), "unexpected: {rendered}");
        }
        other => panic!("expected UnsupportedVersion, got {other:?}"),
    }
}

#[test]
fn wrong_field_type_surfaces_as_config_error() {
    // All mandatory options are present, so check_mandatory_options
    // succeeds — but `api.default_torrent_page_size` is declared as `u8`,
    // and a string is not assignable. Figment's `extract::<Settings>` then
    // fails with the wrapped error.
    let toml = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"

[api]
default_torrent_page_size = "not-a-number"
"#;
    match load_settings(&info_from(toml)) {
        Err(Error::ConfigError { source }) => {
            assert!(!source.to_string().is_empty(), "ConfigError must carry a source");
        }
        other => panic!("expected ConfigError, got {other:?}"),
    }
}

#[test]
fn malformed_toml_collapses_to_missing_mandatory() {
    // Quirk worth documenting: when the TOML source itself fails to
    // parse, Figment returns the parse error from `find_value`, which
    // our loader treats indistinguishably from "key absent". The first
    // mandatory key checked (`logging.threshold`) wins.
    let toml = "this is = \"not closed";
    match load_settings(&info_from(toml)) {
        Err(Error::MissingMandatoryOption { path }) => assert_eq!(path, "logging.threshold"),
        other => panic!("expected MissingMandatoryOption, got {other:?}"),
    }
}

#[test]
fn info_from_toml_round_trips_through_load() {
    // `placeholder_settings()` -> TOML -> back to Settings.
    let original = placeholder_settings();
    let serialised = original.to_toml();
    let reloaded = load_settings(&info_from(&serialised)).expect("placeholder Settings must round-trip via TOML");
    assert_eq!(reloaded, original);
}

#[test]
fn from_figment_error_into_config_error_preserves_source() {
    let figment_err: figment::Error = figment::Error::from("boom");
    let err: Error = figment_err.into();
    match err {
        Error::ConfigError { source } => assert!(source.to_string().contains("boom")),
        other => panic!("expected ConfigError, got {other:?}"),
    }
}
