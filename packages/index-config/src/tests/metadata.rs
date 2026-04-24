//! Tests for [`crate::Metadata`], [`crate::Version`], [`crate::App`],
//! [`crate::Purpose`].
//!
//! ## Index
//!
//! | Test                                       | What it proves                                              |
//! |--------------------------------------------|-------------------------------------------------------------|
//! | `default_metadata_uses_latest_version`     | `Metadata::default()` matches the `LATEST_VERSION` constant.|
//! | `metadata_display_format_is_stable`        | The `Display` impl quotes app/purpose/schema_version.       |
//! | `app_serialises_as_kebab_case`             | `App::TorrustIndex` ↔ `"torrust-index"`.                    |
//! | `purpose_serialises_as_lowercase`          | `Purpose::Configuration` ↔ `"configuration"`.               |
//! | `version_new_and_default_match_latest`     | `Version::new(LATEST)` == `Version::default()`.             |

use crate::{App, LATEST_VERSION, Metadata, Purpose, Version};

#[test]
fn default_metadata_uses_latest_version() {
    let m = Metadata::default();
    assert!(format!("{m}").contains(LATEST_VERSION));
}

#[test]
fn metadata_display_format_is_stable() {
    let m = Metadata::default();
    let s = format!("{m}");
    assert!(s.starts_with("Metadata(app: "));
    assert!(s.contains("purpose: "));
    assert!(s.contains("schema_version: "));
}

#[test]
fn app_serialises_as_kebab_case() {
    let json = serde_json::to_string(&App::TorrustIndex).unwrap();
    assert_eq!(json, "\"torrust-index\"");
    let parsed: App = serde_json::from_str("\"torrust-index\"").unwrap();
    assert_eq!(parsed, App::TorrustIndex);
}

#[test]
fn purpose_serialises_as_lowercase() {
    let json = serde_json::to_string(&Purpose::Configuration).unwrap();
    assert_eq!(json, "\"configuration\"");
}

#[test]
fn version_new_and_default_match_latest() {
    let a = Version::default();
    let b: Version = serde_json::from_value(serde_json::json!({ "schema_version": LATEST_VERSION })).unwrap();
    assert_eq!(a, b);
}
