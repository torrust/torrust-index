// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Collected unit tests for the sentinel crate.
//!
//! Tests that formerly lived as inline `#[cfg(test)] mod tests { … }`
//! blocks inside their parent modules are refactored here so the
//! production source files stay focused on production code.
//!
//! The convergence characterisation suite (ADR-S-013) also lives
//! under this tree.

mod analysis_set;
mod config;
mod convergence_clipping;
mod convergence_common;
mod convergence_diagnostics;
mod convergence_eta;
mod convergence_ewma;
mod convergence_fixes;
mod convergence_noise;
mod cusum;
mod ewma;
mod observation;
mod report;
mod tracker;
mod variance_formula;
