// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Test infrastructure: fluent builder, config presets, plan runner.
//!
//! This module is **public** so that both crate tests (`src/tests/`)
//! and integration tests (`tests/`) can share the same helpers.
//! It uses only Surface 1 / Surface 2 API — no `pub(crate)` internals.

mod builder;
mod config;
mod plan;
mod presets;
#[cfg(any(debug_assertions, test))]
mod rng;
mod runner;

// ── Re-exports ──────────────────────────────────────────────────

pub use self::builder::GraphCreator;
pub use self::config::*;
pub use self::plan::Plan;
pub use self::presets::*;
#[cfg(any(debug_assertions, test))]
pub use self::rng::*;
pub use self::runner::*;
