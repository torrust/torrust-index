// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Comprehensive comparison tests: Brand's incremental SVD vs naïve dense SVD.
//!
//! Every test runs both algorithms on the same inputs and asserts that
//! they produce equivalent results (up to SVD sign ambiguity and FP
//! tolerance).  This is the expanded, persistent version of the
//! `debug_assert` oracle in `maths::evolve()`.
//!
//! # §-references
//!
//! - ADR-S-016 — Brand's incremental SVD
//! - ADR-S-015 — Cell creation performance

mod brand_vs_naive;
