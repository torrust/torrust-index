// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Bayesian linear model and parameter types.
//!
//! This module provides the core `BayesianLinearModel` type, dense and
//! dynamically dimensioned (´dec:substrate:dense-dynamic´), and the
//! serialisable `ModelParameters` snapshot used for lock-free publication
//! and persistence.
//!
//! # Module Structure
//!
//! - [`bayesian`] — `BayesianLinearModel`: dynamic-dimension conjugate
//!   normal model with precision/covariance dual tracking.
//! - [`parameters`] — `ModelParameters`: lightweight snapshot of μ and Σ
//!   for `ArcSwap` publication; B is excluded and reconstructed on demand
//!   (´dec:retention:precision-excluded´).
//! - [`hibernation`] — the archive a hibernating deregistration writes and a
//!   later registration of the same identifier reads
//!   (´alg:registry:hibernation´).
//!
//! # Cross-References
//!
//! - (´dec:substrate:dense-dynamic´) — the dynamic-dimension model type
//! - (´dec:retention:precision-excluded´) — snapshot composition and the
//!   B-exclusion
//! - (´dec:posterior:recomputation-trigger´) — what the Cholesky recomputation
//!   tracking fields are counting towards
//! - (´dec:posterior:half-solve´) — the Schur complement marginalisation

// These modules are `pub(crate)` and consumed only by snapshot and
// tests.  `dead_code` is suppressed until public API consumers
// exist.
#[allow(dead_code)]
pub mod bayesian;
#[allow(dead_code)]
pub mod hibernation;
pub mod marginalise;
#[allow(dead_code)]
pub mod parameters;
#[allow(dead_code)]
pub mod recompute;
#[allow(dead_code)]
pub mod update;
