// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Feature extraction and assembly.
//!
//! This module provides components for extracting features from Sentinel
//! reports and assembling them into feature vectors for the Bayesian model.
//!
//! # Module Structure
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`aggregate`] | Cross-Sentinel aggregate features (15 features) |
//! | [`assembly`] | Feature vector assembly from components |
//! | [`bootstrap`] | Bootstrap and batch init accumulators |
//! | [`dimension_map`] | Feature vector index tracking |
//! | [`interaction`] | Interaction templates and computation |
//! | [`standardisation`] | Feature standardisation and running statistics |
//!
//! # Cross-References
//!
//! - (´dec:ordering:synchronous-reception´) — the reception that delivers the
//!   Sentinel reports these components extract from
//! - (´dec:vector:block-order´) — the one canonical block order of the feature
//!   vector these components assemble
//! - (´dec:vector:statistic-timing´) — when the running statistics observe a
//!   vector and when their continuing update is applied
//! - (´def:extraction:slot´) — the per-Sentinel slot's structure: an
//!   occupancy indicator, then the extraction behind it

pub mod aggregate;
pub mod assembly;
pub mod bootstrap;
pub mod dimension_map;
pub mod interaction;
pub mod permutation;
pub mod standardisation;
