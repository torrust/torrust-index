// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Public `assess()` method on `Assayer`.
//!
//! Delegates to `assess_single()` for each request in the batch,
//! sharing a single monotonic clock reading across all requests.
//!
//! # Cross-References
//!
//! - Core assessment output (´schema:output:assessment´)
//! - The assessment call takes a batch and cannot fail (´dec:surface:batch-infallible´)

use std::sync::atomic::Ordering;

use crate::Assayer;
use crate::assessment::{self, RequestContext, RiskAssessment};

impl Assayer {
    /// Performs core risk assessment for a batch of request contexts.
    ///
    /// Reads the monotonic clock once and shares it across the batch.
    /// Each request is processed independently through the core assessment
    /// pipeline. Assessments are returned in the same order as the input
    /// requests.
    ///
    /// # Cross-References
    ///
    /// - Per-request snapshot acquisition (´dec:concurrency:per-request-load´)
    /// - Core assessment output (´schema:output:assessment´)
    /// - The assessment call takes a batch and cannot fail (´dec:surface:batch-infallible´)
    #[must_use]
    pub fn assess(&self, requests: &[RequestContext]) -> Vec<RiskAssessment> {
        // An empty request slice returns an empty result vector with no
        // snapshot load and no side effects (´dec:surface:batch-infallible´).
        if requests.is_empty() {
            return Vec::new();
        }

        // One monotonic reading for the whole batch
        // (´dec:ordering:batch-timestamp´): two requests in one batch
        // cannot disagree about the present, and with an injected clock
        // they cannot disagree across runs either.
        let batch_now = self.clock.now_monotonic();

        // TODO ´todo:code:consider-rayon-par-iter-once-end´: Consider `rayon::par_iter()` once end-to-end
        // profiling shows batch-level parallelism improves throughput; assessment is one function
        // rather than a stage pipeline (´dec:ordering:assessment-function´).
        requests
            .iter()
            .map(|req| {
                // Per-request snapshot load (´dec:concurrency:per-request-load´):
                // a label published mid-batch is visible to later requests.
                let snapshot = self.shared.published.load();
                let assessment = assessment::assess_single(req, self, &snapshot, batch_now);

                self.total_assessments.fetch_add(1, Ordering::Relaxed);

                // Track assessment blend weight and degradation counters.
                self.blend_stats.observe(assessment.risk.anchor_weight);
                self.degradation_counters
                    .record(&assessment.health.degradation, assessment.health.zero_sentinels);

                assessment
            })
            .collect()
    }
}
