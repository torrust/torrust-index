// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Public `request_labels()` method on `Assayer`.
//!
//! Delegates to the Core label-guidance implementation without sending work
//! through the model-owner thread.
//!
//! # Cross-References
//!
//! - Core label guidance (´sec:guidance:core´)
//! - Guidance is a read-only, infallible, synchronous query (´dec:surface:read-only-guidance´)

use crate::Assayer;
use crate::guidance::{self, LabelBudget, LabelGuidanceParams, LabelRequests};

impl Assayer {
    /// Returns label guidance for live pending assessments.
    ///
    /// This is a read-only, synchronous, infallible Core query. Each category
    /// is scored and truncated independently; candidates can appear in more
    /// than one category, with cross-category membership reported through
    /// [`crate::LabelCandidate::also_in`].
    #[must_use]
    pub fn request_labels(&self, budget: LabelBudget, params: LabelGuidanceParams) -> LabelRequests {
        guidance::request_labels(self, budget, params)
    }
}
