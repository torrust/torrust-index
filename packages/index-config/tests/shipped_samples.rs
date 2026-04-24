//! Integration test: every checked-in `index.*.toml` from the
//! repository's `share/default/config/` directory is *structurally
//! valid* but intentionally incomplete.
//!
//! Per ADR-T-009 §D2, shipped defaults carry no credentials and no
//! environment-coupled values: `tracker.token`, `database.connect_url`,
//! `[mail.smtp]`, and `[auth]` paths must come from the operator at
//! runtime (env vars or a side-loaded TOML). A shipped sample
//! therefore parses as TOML but fails the schema's mandatory-field
//! check until the operator supplies those values.
//!
//! The samples are embedded at compile time with `include_str!` so
//! the test binary is self-contained and does not depend on the
//! repository layout at runtime.
//!
//! ## Index
//!
//! | Test                                              | What it proves                                                       |
//! |---------------------------------------------------|----------------------------------------------------------------------|
//! | `every_shipped_index_toml_is_valid_toml`          | Every embedded `index.*.toml` is syntactically valid TOML.           |
//! | `every_shipped_index_toml_omits_credentials`      | No shipped sample carries `connect_url` / `token` / `[mail.smtp]`.   |
//! | `every_shipped_index_toml_demands_runtime_secrets`| The schema rejects each sample for a missing mandatory secret field. |
//! | `development_sqlite3_uses_info_threshold`         | The dev sample still sets `logging.threshold = "info"`.              |

use torrust_index_config::{Error, Info, Threshold, load_settings};

/// Embedded `index.*.toml` samples shipped in `share/default/config/`.
///
/// Keep in sync with the directory contents. Non-`index.*` files
/// (e.g. tracker samples) are intentionally excluded.
const SHIPPED_INDEX_SAMPLES: &[(&str, &str)] = &[
    (
        "index.container.toml",
        include_str!("../../../share/default/config/index.container.toml"),
    ),
    (
        "index.development.sqlite3.toml",
        include_str!("../../../share/default/config/index.development.sqlite3.toml"),
    ),
    (
        "index.private.e2e.container.sqlite3.toml",
        include_str!("../../../share/default/config/index.private.e2e.container.sqlite3.toml"),
    ),
    (
        "index.public.e2e.container.toml",
        include_str!("../../../share/default/config/index.public.e2e.container.toml"),
    ),
];

const DEVELOPMENT_SQLITE3_TOML: &str = include_str!("../../../share/default/config/index.development.sqlite3.toml");

#[test]
fn every_shipped_index_toml_is_valid_toml() {
    assert!(
        !SHIPPED_INDEX_SAMPLES.is_empty(),
        "expected to find shipped index.*.toml samples"
    );

    for (name, toml_str) in SHIPPED_INDEX_SAMPLES {
        let parsed: Result<toml::Table, _> = toml::from_str(toml_str);
        assert!(parsed.is_ok(), "shipped sample {name} is not valid TOML: {:?}", parsed.err());
    }
}

#[test]
fn every_shipped_index_toml_omits_credentials() {
    // ADR-T-009 §D2: no credentials, no `connect_url`, no SMTP
    // hostnames, no auth-key paths in shipped defaults.
    for (name, toml) in SHIPPED_INDEX_SAMPLES {
        for forbidden in ["connect_url", "token =", "[mail.smtp]", "private_key_path", "public_key_path"] {
            assert!(
                !toml.contains(forbidden),
                "shipped sample {name} must not contain `{forbidden}` (ADR-T-009 §D2)"
            );
        }
    }
}

#[test]
fn every_shipped_index_toml_demands_runtime_secrets() {
    // The sample on its own must fail the loader: the operator is
    // required to supply `tracker.token` and `database.connect_url`
    // (via env var or a side-loaded TOML) at runtime. The exact
    // missing-field name varies per sample (whichever is checked
    // first wins), so the test only asserts that the load *fails*
    // with an error mentioning `token` or `connect_url`.
    for (name, toml) in SHIPPED_INDEX_SAMPLES {
        match load_settings(&Info::from_toml(toml)) {
            Err(Error::ConfigError { source }) => {
                let msg = source.to_string();
                assert!(
                    msg.contains("token") || msg.contains("connect_url"),
                    "shipped sample {name} failed for unexpected reason: {msg}"
                );
            }
            Ok(_) => panic!("shipped sample {name} loaded without operator-supplied secrets"),
            Err(other) => panic!("shipped sample {name} failed with unexpected error variant: {other:?}"),
        }
    }
}

#[test]
fn development_sqlite3_uses_info_threshold() {
    // The dev sample is operator-incomplete too, but its `logging`
    // section is still inspectable via raw TOML parsing.
    let table: toml::Table = toml::from_str(DEVELOPMENT_SQLITE3_TOML).expect("dev sample must be valid TOML");
    let threshold = table
        .get("logging")
        .and_then(|l| l.get("threshold"))
        .and_then(toml::Value::as_str)
        .expect("dev sample must declare logging.threshold");
    assert_eq!(threshold, Threshold::Info.to_string());
}
