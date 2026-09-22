// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Linear algebra foundation for the Assayer crate.
//!
//! This module encapsulates all interaction with the `faer` linear algebra
//! backend behind an isolation layer. No direct `faer` imports are permitted
//! outside this module (enforced by convention; CI lint planned).
//!
//! # Module Structure
//!
//! - [`symmetric`] — `SymmetricMatrix` newtype with symmetry enforcement
//! - [`bridge`] — Cholesky factorisation, inverse, solve, and vector extraction
//! - [`convert`] — `faer` ↔ `Vec<f64>` conversion helpers (always available)
//! - [`serde_support`] — `Serialize`/`Deserialize` impls for `faer` types
//!   (requires the `serde` feature)
//!
//! # Cross-References
//!
//! - (´dec:substrate:symmetry-invariant´) — symmetric matrix storage and the
//!   invariant its layout maintains
//! - (´dec:substrate:single-backend´) — the one linear algebra backend, `faer`

// These modules are `pub(crate)` and consumed only by other internal
// modules and tests.  `dead_code` is suppressed until public API
// consumers exist.
#[allow(dead_code)]
pub mod bridge;
pub mod convert;
#[allow(dead_code)]
pub mod symmetric;

#[cfg(feature = "serde")]
#[allow(dead_code)]
pub mod serde_support;
