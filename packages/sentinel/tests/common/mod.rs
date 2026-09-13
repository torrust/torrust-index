// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Shared helpers for Sentinel integration tests.
//!
//! # Submodules
//!
//! | Module          | Contents                                                    |
//! |-----------------|-------------------------------------------------------------|
//! | [`assertions`]  | [`assert_invariants()`], [`assert_reports_identical()`], [`max_cusum()`], [`max_novelty_z()`], [`root_novelty_z()`] |
//! | [`builders`]    | [`seeded_sentinel()`], [`ScenarioBuilder`]                  |
//! | [`config`]      | [`test_config()`], [`cold_config()`], [`integration_config()`] |
//! | [`generators`]  | [`cell_values()`], [`cell_values_prefix()`], [`anomalous_values()`] |

// Each integration test file includes this module independently, so
// not every test file uses every helper.
#![allow(dead_code, unused_imports)]

mod assertions;
mod builders;
mod config;
mod generators;

pub use assertions::{assert_invariants, assert_reports_identical, max_cusum, max_novelty_z, root_novelty_z};
pub use builders::{ScenarioBuilder, seeded_sentinel};
pub use config::{cold_config, integration_config, test_config};
pub use generators::{anomalous_values, cell_values, cell_values_prefix};
