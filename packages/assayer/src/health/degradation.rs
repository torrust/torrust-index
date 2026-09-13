// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Per-assessment degradation context.
//!
//! Tracks sanitisation and degradation events from checkpoints CP1–CP4
//! (´tab:runtime:numeric-checkpoints´) during a single
//! assessment. Carried by `PendingAssessment` in the concurrent map that
//! holds an assessment awaiting its label (´dec:retention:pending-map´), and
//! by the `RiskAssessment` result struct.
//!
//! # Cross-References
//!
//! - (´cor:degradation:in-band-travel´) — the context travels with the result
//!   rather than through an error channel
//! - (´rem:degradation:queue-face´) — why a full queue degrades rather than
//!   failing on the observing path

use crate::types::{ModelId, SentinelId};

/// Degradation context for a single assessment.
///
/// Carried inside the `RiskAssessment` struct. All fields default to
/// zero / empty, representing no degradation.
///
/// The flag a degraded value is retained under (´dec:degradation:retain-and-flag´).
#[derive(Debug, Default, Clone)]
#[non_exhaustive]
pub struct DegradationContext {
    /// Signals that were NaN/Inf and sanitised to 0 (CP1).
    pub signals_sanitised: u32,

    /// Signals whose `SignalValue` variant did not match the declared
    /// `SignalShape` (e.g. `Categorical` for a `Scalar` signal). These
    /// inputs are zero-filled by `encode_signal` — sanitised
    /// at the boundary rather than rejected (´dec:surface:sanitise-not-reject´).
    pub signals_shape_mismatched: u32,

    /// Signal names not found in the declared schema, which is the authority
    /// for what has a position at all (´req:signal:schema-fixed´). Skipped
    /// during encoding.
    pub signals_unknown: u32,

    /// Sentinels whose extraction produced NaN and were zeroed
    /// entirely (CP2). Occupancy set to 0 for these Sentinels.
    pub nan_sentinels: Vec<SentinelId>,

    /// Sentinels whose cached batch report had per-cell data
    /// quality issues (NaN in scores, negative variance).
    /// Cell data zeroed; report otherwise intact.
    pub degraded_report_sentinels: Vec<SentinelId>,

    /// Features in the assembled $\hat\phi$ that were NaN/Inf and
    /// sanitised to 0 after standardisation (CP3).
    pub features_sanitised: u32,

    /// Models whose point estimate or uncertainty was NaN and
    /// fell back to the prior (CP4).
    pub degraded_models: Vec<ModelId>,

    /// Batch-init observations this assessment skipped on a contended
    /// lock. A load condition, reported in band like the identity
    /// queue's overflow (´cor:degradation:in-band-travel´).
    pub batch_init_observations_skipped: u32,
}

impl DegradationContext {
    /// Returns `true` if any degradation occurred during this assessment.
    #[must_use]
    pub const fn is_degraded(&self) -> bool {
        self.signals_sanitised > 0
            || self.signals_shape_mismatched > 0
            || self.signals_unknown > 0
            || !self.nan_sentinels.is_empty()
            || !self.degraded_report_sentinels.is_empty()
            || self.features_sanitised > 0
            || !self.degraded_models.is_empty()
            || self.batch_init_observations_skipped > 0
    }
}
