// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Snapshot and working-copy infrastructure.
//!
//! This module provides the concurrency bridge between the single-writer
//! model-owner thread and multiple lock-free readers. The design follows the
//! swap discipline a reader observes (´dec:concurrency:snapshot-swap´) and the
//! single published snapshot it reads whole (´dec:retention:monolithic-snapshot´).
//!
//! # Module Structure
//!
//! - [`published`] — `ModelSnapshot`, `AxisModelState`, `DimensionMapSnapshot`,
//!   `DriftState`, `CalibrationBufferSnapshot`:
//!   the read-only snapshot published via `ArcSwap`.
//! - [`working`] — `WorkingCopy`, `WorkingAxisModel`, `ModelConfig`:
//!   the mutable counterpart maintained by the model-owner thread.
//! - [`shared`] — `SharedState`: the `ArcSwap<ModelSnapshot>` holder
//!   that readers load and the owner stores.
//!
//! # Cross-References
//!
//! - (´dec:concurrency:snapshot-swap´) — lock-free publication by one swap
//! - (´dec:substrate:dense-dynamic´) — the dynamic-dimension model type published
//! - (´dec:retention:precision-excluded´) — snapshot composition and the B-exclusion
//! - (´dec:posterior:recomputation-trigger´) — what the Cholesky tracking fields count towards

// These modules are `pub(crate)` and consumed only by tests.
// `dead_code` is suppressed until public API consumers exist.
#[allow(dead_code)]
pub mod published;
#[allow(dead_code)]
pub mod shared;
#[allow(dead_code)]
pub mod working;
