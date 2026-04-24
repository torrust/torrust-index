//! Crate-level test suite for `torrust-index-config`.
//!
//! These tests have `pub(crate)` visibility into the configuration
//! crate (per AGENTS.md "Test Locations"), and they exercise the
//! parsing surface — `load_settings`, defaults, mandatory-option
//! enforcement, schema-version handshake, secret redaction — without
//! relying on global process state (env vars).
//!
//! ## Index of submodules
//!
//! | Module        | Focus                                                |
//! |---------------|------------------------------------------------------|
//! | `loader`      | `load_settings` happy path + every `Error` variant.  |
//! | `metadata`    | `Metadata` / `Version` / `App` / `Purpose` shape.    |
//! | `redaction`   | `Settings::remove_secrets` covers every secret slot. |
//! | `quirks`      | Unicode, IPv6, NoneAsEmptyString, validator quirks.  |
//! | `permissions` | `Role` / `Action` / `Effect` round-trip & overrides. |

mod loader;
mod metadata;
mod permissions;
mod quirks;
mod redaction;

use crate::Info;

/// The minimum legal TOML — every mandatory option present, nothing
/// more. Used as a known-good baseline by many tests.
pub const MINIMUM_VALID_TOML: &str = r#"
[metadata]
schema_version = "2.0.0"

[logging]
threshold = "info"

[tracker]
token = "MyAccessToken"
"#;

/// Convenience: build an [`Info`] that bypasses the filesystem and
/// the environment so tests stay hermetic and parallel-safe.
pub fn info_from(toml: &str) -> Info {
    Info::from_toml(toml)
}
