//! # Config-probe unit tests
//!
//! | Test                                              | What it covers                                   |
//! |---------------------------------------------------|--------------------------------------------------|
//! | `sqlite_relative_url_yields_relative_path`        | `sqlite://data.db?mode=rwc` → `data.db`          |
//! | `sqlite_absolute_url_yields_absolute_path`        | `sqlite:///var/...` → `/var/...`                 |
//! | `sqlite_memory_url_yields_memory_marker`          | `sqlite::memory:` → `:memory:`                   |
//! | `sqlite_percent_encoded_path_is_decoded`          | `%20` → space                                    |
//! | `mysql_url_yields_null_path`                      | mysql → `null` path                              |
//! | `mariadb_url_is_unsupported`                      | mariadb is not a supported backend (exit 5)      |
//! | `postgres_url_is_unsupported`                     | exit-5 condition                                 |
//! | `auth_pem_only_resolves_to_pem`                   | PEM beats absent path                            |
//! | `auth_path_only_resolves_to_path`                 | path-only branch                                 |
//! | `auth_neither_resolves_to_none`                   | nothing configured                               |
//! | `auth_both_pem_and_path_pem_wins`                 | precedence per D3 raw-presence reporting         |
//! | `empty_tracker_token_is_rejected`                 | exit-4 condition                                 |
//! | `probe_round_trips_as_json`                       | JSON output deserialises back                    |
//! | `auth_key_source_serialises_lowercase`            | enum wire format is stable                       |
//! | `driver_serialises_lowercase`                     | `Driver` enum wire format is stable              |
//! | `placeholder_settings_yield_known_shape`          | end-to-end shape via `settings_with_database`    |

use serde_json::json;
use torrust_index_config::test_helpers::placeholder_settings;
use torrust_index_config::{Info, load_settings};
use url::Url;

use crate::{AuthKeySource, AuthProbe, DatabaseProbe, Driver, Probe, ProbeError, SCHEMA, probe, probe_database};

fn settings_with_database(connect_url: &str) -> torrust_index_config::Settings {
    let toml = format!(
        r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "{connect_url}"
"#
    );
    load_settings(&Info::from_toml(&toml)).expect("test TOML must load")
}

#[test]
fn sqlite_relative_url_yields_relative_path() {
    let url = Url::parse("sqlite://data.db?mode=rwc").unwrap();
    let out = probe_database(&url).unwrap();
    assert_eq!(out.driver, Driver::Sqlite);
    assert_eq!(out.path.as_deref(), Some("data.db"));
}

#[test]
fn sqlite_absolute_url_yields_absolute_path() {
    let url = Url::parse("sqlite:///var/lib/torrust/index.db").unwrap();
    let out = probe_database(&url).unwrap();
    assert_eq!(out.driver, Driver::Sqlite);
    assert_eq!(out.path.as_deref(), Some("/var/lib/torrust/index.db"));
}

#[test]
fn sqlite_memory_url_yields_memory_marker() {
    let url = Url::parse("sqlite::memory:").unwrap();
    let out = probe_database(&url).unwrap();
    assert_eq!(out.driver, Driver::Sqlite);
    assert_eq!(out.path.as_deref(), Some(":memory:"));
}

#[test]
fn sqlite_percent_encoded_path_is_decoded() {
    let url = Url::parse("sqlite:///srv/My%20Data/x.db").unwrap();
    let out = probe_database(&url).unwrap();
    assert_eq!(out.path.as_deref(), Some("/srv/My Data/x.db"));
}

#[test]
fn mysql_url_yields_null_path() {
    let url = Url::parse("mysql://user:pass@host:3306/db").unwrap();
    let out = probe_database(&url).unwrap();
    assert_eq!(out.driver, Driver::Mysql);
    assert_eq!(out.path, None);
}

#[test]
fn mariadb_url_is_unsupported() {
    // The application's own `databases::database::get_driver`
    // does not recognise `mariadb` (only `sqlite` / `mysql`),
    // so the probe must reject it at the container boundary
    // rather than emit a driver value the entry script could
    // not dispatch on.
    let url = Url::parse("mariadb://user:pass@host:3306/db").unwrap();
    match probe_database(&url) {
        Err(ProbeError::UnsupportedScheme(s)) => assert_eq!(s, "mariadb"),
        other => panic!("expected UnsupportedScheme, got {other:?}"),
    }
}

#[test]
fn postgres_url_is_unsupported() {
    let url = Url::parse("postgres://user@host/db").unwrap();
    match probe_database(&url) {
        Err(ProbeError::UnsupportedScheme(s)) => assert_eq!(s, "postgres"),
        other => panic!("expected UnsupportedScheme, got {other:?}"),
    }
}

#[test]
fn auth_pem_only_resolves_to_pem() {
    let key = super::probe_auth_key(Some("-----BEGIN PRIVATE KEY-----\n..."), None);
    assert!(key.pem_set);
    assert!(!key.path_set);
    assert_eq!(key.source, AuthKeySource::Pem);
    assert_eq!(key.path, None);
}

#[test]
fn auth_path_only_resolves_to_path() {
    let key = super::probe_auth_key(None, Some("/etc/torrust/index/auth/private.pem"));
    assert!(!key.pem_set);
    assert!(key.path_set);
    assert_eq!(key.source, AuthKeySource::Path);
    assert_eq!(key.path.as_deref(), Some("/etc/torrust/index/auth/private.pem"));
}

#[test]
fn auth_neither_resolves_to_none() {
    let key = super::probe_auth_key(None, None);
    assert!(!key.pem_set);
    assert!(!key.path_set);
    assert_eq!(key.source, AuthKeySource::None);
    assert_eq!(key.path, None);
}

#[test]
fn auth_both_pem_and_path_pem_wins() {
    let key = super::probe_auth_key(Some("PEM"), Some("/some/path"));
    assert!(key.pem_set);
    assert!(key.path_set);
    assert_eq!(key.source, AuthKeySource::Pem);
    assert_eq!(key.path, None, "PEM-wins precedence yields null path");
}

#[test]
fn empty_tracker_token_is_rejected() {
    // Use a TOML that bypasses ApiToken::new (which would assert).
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
    let settings = load_settings(&Info::from_toml(toml)).expect("empty token must still parse");
    match probe(&settings) {
        Err(ProbeError::EmptyTrackerToken) => {}
        other => panic!("expected EmptyTrackerToken, got {other:?}"),
    }
}

#[test]
fn probe_round_trips_as_json() {
    let settings = placeholder_settings();
    let out = probe(&settings).unwrap();
    let json = serde_json::to_string(&out).unwrap();
    let parsed: Probe = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.schema, SCHEMA);
    assert_eq!(parsed, out);
}

#[test]
fn auth_key_source_serialises_lowercase() {
    let auth = AuthProbe {
        private_key: super::probe_auth_key(Some("PEM"), None),
        public_key: super::probe_auth_key(None, Some("/p")),
    };
    let value = serde_json::to_value(&auth).unwrap();
    assert_eq!(value["private_key"]["source"], json!("pem"));
    assert_eq!(value["public_key"]["source"], json!("path"));
}

#[test]
fn driver_serialises_lowercase() {
    assert_eq!(serde_json::to_value(Driver::Sqlite).unwrap(), json!("sqlite"));
    assert_eq!(serde_json::to_value(Driver::Mysql).unwrap(), json!("mysql"));
}

#[test]
fn placeholder_settings_yield_known_shape() {
    let settings = settings_with_database("sqlite://data.db?mode=rwc");
    let out = probe(&settings).unwrap();
    assert_eq!(
        out.database,
        DatabaseProbe {
            driver: Driver::Sqlite,
            path: Some("data.db".into()),
        }
    );
    assert_eq!(out.auth.private_key.source, AuthKeySource::None);
    assert_eq!(out.auth.public_key.source, AuthKeySource::None);
}
