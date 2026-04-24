//! Shared test fixtures for `Settings`.
//!
//! After ADR-T-009 §D2, `Settings` carries no `impl Default`:
//! `tracker.token` and `database.connect_url` are mandatory at the
//! schema level. This module owns the single canonical placeholder
//! TOML used wherever a "minimal but legal" baseline is needed —
//! by this crate's own tests, by the root crate's `#[cfg(test)]`
//! constructor, and by integration tests in either crate.
//!
//! The module is `#[doc(hidden)] pub` so it is reachable from
//! integration test binaries without appearing in the public docs.

use crate::{Info, Settings, load_settings};

/// Minimum legal TOML — every mandatory option present, nothing
/// more. Mirrors the operator-supplied surface after ADR-T-009 §D2.
pub const PLACEHOLDER_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"

[database]
connect_url = "sqlite://data.db?mode=rwc"
"#;

/// A fully-populated `Settings` loaded from [`PLACEHOLDER_TOML`].
///
/// Replaces the previous `Settings::default()` test fixture.
///
/// # Panics
///
/// Panics if [`PLACEHOLDER_TOML`] ever stops loading — that would
/// be a bug in the loader and must surface loudly.
#[must_use]
pub fn placeholder_settings() -> Settings {
    load_settings(&Info::from_toml(PLACEHOLDER_TOML)).expect("PLACEHOLDER_TOML must load")
}
