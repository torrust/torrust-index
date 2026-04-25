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
//! | `info`        | `Info::new` / `Info::from_env` / `Tls::default`.     |
//! | `loader`      | `load_settings` happy path + every `Error` variant.  |
//! | `metadata`    | `Metadata` / `Version` / `App` / `Purpose` shape.    |
//! | `redaction`   | `Settings::remove_secrets` covers every secret slot. |
//! | `quirks`      | Unicode, IPv6, NoneAsEmptyString, validator quirks.  |
//! | `permissions` | `Role` / `Action` / `Effect` round-trip & overrides. |
//! | `tracker`     | `Tracker::override_*` + `ApiToken` accessors.        |

mod info;
mod loader;
mod metadata;
mod permissions;
mod quirks;
mod redaction;
mod tracker;

use crate::Info;
pub use crate::test_helpers::{PLACEHOLDER_TOML as MINIMUM_VALID_TOML, placeholder_settings};

/// Convenience: build an [`Info`] that bypasses the filesystem and
/// the environment so tests stay hermetic and parallel-safe.
pub fn info_from(toml: &str) -> Info {
    Info::from_toml(toml)
}
