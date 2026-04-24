//! Integration test: load every checked-in `index.*.toml` from the
//! repository's `share/default/config/` directory.
//!
//! This is the "real-world" smoke test — if an operator-facing
//! sample TOML stops parsing, we want to know about it before they
//! do.
//!
//! The samples are embedded at compile time with `include_str!` so
//! the test binary is self-contained and does not depend on the
//! repository layout at runtime (e.g. when executed from a nextest
//! archive in a container where `share/` is absent).
//!
//! ## Index
//!
//! | Test                                       | What it proves                                         |
//! |--------------------------------------------|--------------------------------------------------------|
//! | `every_shipped_index_toml_loads`           | Every embedded `index.*.toml` parses.                  |
//! | `development_sqlite3_uses_info_threshold`  | The dev sample sets `logging.threshold = "info"`.      |

use torrust_index_config::{Info, Settings, Threshold, load_settings};

/// Embedded `index.*.toml` samples shipped in `share/default/config/`.
///
/// Keep in sync with the directory contents. Non-`index.*` files
/// (e.g. tracker samples) are intentionally excluded.
const SHIPPED_INDEX_SAMPLES: &[(&str, &str)] = &[
    (
        "index.container.mysql.toml",
        include_str!("../../../share/default/config/index.container.mysql.toml"),
    ),
    (
        "index.container.sqlite3.toml",
        include_str!("../../../share/default/config/index.container.sqlite3.toml"),
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
        "index.public.e2e.container.mysql.toml",
        include_str!("../../../share/default/config/index.public.e2e.container.mysql.toml"),
    ),
    (
        "index.public.e2e.container.sqlite3.toml",
        include_str!("../../../share/default/config/index.public.e2e.container.sqlite3.toml"),
    ),
];

const DEVELOPMENT_SQLITE3_TOML: &str = include_str!("../../../share/default/config/index.development.sqlite3.toml");

#[test]
fn every_shipped_index_toml_loads() {
    assert!(
        !SHIPPED_INDEX_SAMPLES.is_empty(),
        "expected to find shipped index.*.toml samples"
    );

    for (name, toml) in SHIPPED_INDEX_SAMPLES {
        let result: Result<Settings, _> = load_settings(&Info::from_toml(toml));
        assert!(result.is_ok(), "shipped sample {name} failed to load: {:?}", result.err());
    }
}

#[test]
fn development_sqlite3_uses_info_threshold() {
    let settings = load_settings(&Info::from_toml(DEVELOPMENT_SQLITE3_TOML)).expect("dev sample must load");
    assert_eq!(settings.logging.threshold, Threshold::Info);
}
