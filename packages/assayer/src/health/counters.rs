// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Assessment degradation counters: the aggregate face of degradation that
//! travels with the result rather than through an error channel
//! (´cor:degradation:in-band-travel´).
//!
//! Atomic assessment counters incremented at assessment return when their
//! corresponding conditions occur.
//!
//! # Cross-References
//!
//! - (´dec:health:tiered-queries´) — the cheap tier that reads these atomics
//! - (´dec:degradation:infallible-core´) — the absent error channel these
//!   counters are the aggregate face of

use std::sync::atomic::{AtomicU64, Ordering};

/// Atomic count of emitted drift-reset events.
pub struct DriftResetCounter {
    /// Monotone event total.
    value: AtomicU64,
}

impl DriftResetCounter {
    /// Creates a zeroed drift-reset counter.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    /// Records one emitted drift-reset event.
    pub fn increment(&self) {
        self.value.fetch_add(1, Ordering::Relaxed);
    }

    /// Reads the current drift-reset count.
    #[must_use]
    pub fn value(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for DriftResetCounter {
    fn default() -> Self {
        Self::new()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assessment Degradation Counters
// ═══════════════════════════════════════════════════════════════════════════════

/// Atomic counters for assessment-time degradation events.
///
/// Incremented on the assessment path (no locks). Read by `health_summary()`.
pub struct AssessmentDegradationCounters {
    /// Total assessments that had any degradation.
    pub total_degraded: AtomicU64,
    /// Total signals sanitised across all assessments.
    pub signals_sanitised: AtomicU64,
    /// Total signal shape mismatches across all assessments.
    pub signals_shape_mismatched: AtomicU64,
    /// Total unknown signal names across all assessments.
    pub signals_unknown: AtomicU64,
    /// Total Sentinel slots zeroed due to NaN.
    pub sentinel_slots_zeroed: AtomicU64,
    /// Total features sanitised post-standardisation.
    pub features_sanitised: AtomicU64,
    /// Total model fallback-to-prior events.
    pub model_fallbacks: AtomicU64,
    /// Total cold-ramp observations skipped on a full command channel.
    pub batch_init_observations_skipped: AtomicU64,
    /// Total assessments completed without a reporting Sentinel.
    pub zero_sentinel_assessments: AtomicU64,
}

impl AssessmentDegradationCounters {
    /// Creates zeroed counters.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            total_degraded: AtomicU64::new(0),
            signals_sanitised: AtomicU64::new(0),
            signals_shape_mismatched: AtomicU64::new(0),
            signals_unknown: AtomicU64::new(0),
            sentinel_slots_zeroed: AtomicU64::new(0),
            features_sanitised: AtomicU64::new(0),
            model_fallbacks: AtomicU64::new(0),
            batch_init_observations_skipped: AtomicU64::new(0),
            zero_sentinel_assessments: AtomicU64::new(0),
        }
    }

    /// Records aggregate conditions from an assessment result.
    pub fn record(&self, degradation: &crate::health::DegradationContext, zero_sentinels: bool) {
        if degradation.is_degraded() {
            self.total_degraded.fetch_add(1, Ordering::Relaxed);
        }
        if degradation.signals_sanitised > 0 {
            self.signals_sanitised
                .fetch_add(u64::from(degradation.signals_sanitised), Ordering::Relaxed);
        }
        if degradation.signals_shape_mismatched > 0 {
            self.signals_shape_mismatched
                .fetch_add(u64::from(degradation.signals_shape_mismatched), Ordering::Relaxed);
        }
        if degradation.signals_unknown > 0 {
            self.signals_unknown
                .fetch_add(u64::from(degradation.signals_unknown), Ordering::Relaxed);
        }
        let nan_sentinels = degradation.nan_sentinels.len() as u64;
        if nan_sentinels > 0 {
            self.sentinel_slots_zeroed.fetch_add(nan_sentinels, Ordering::Relaxed);
        }
        if degradation.features_sanitised > 0 {
            self.features_sanitised
                .fetch_add(u64::from(degradation.features_sanitised), Ordering::Relaxed);
        }
        let model_fallbacks = degradation.degraded_models.len() as u64;
        if model_fallbacks > 0 {
            self.model_fallbacks.fetch_add(model_fallbacks, Ordering::Relaxed);
        }
        if degradation.batch_init_observations_skipped > 0 {
            self.batch_init_observations_skipped
                .fetch_add(u64::from(degradation.batch_init_observations_skipped), Ordering::Relaxed);
        }
        if zero_sentinels {
            self.zero_sentinel_assessments.fetch_add(1, Ordering::Relaxed);
        }
    }

    /// Snapshots the current counter values.
    #[must_use]
    pub fn snapshot(&self) -> AssessmentDegradationSnapshot {
        AssessmentDegradationSnapshot {
            total_degraded: self.total_degraded.load(Ordering::Relaxed),
            signals_sanitised: self.signals_sanitised.load(Ordering::Relaxed),
            signals_shape_mismatched: self.signals_shape_mismatched.load(Ordering::Relaxed),
            signals_unknown: self.signals_unknown.load(Ordering::Relaxed),
            sentinel_slots_zeroed: self.sentinel_slots_zeroed.load(Ordering::Relaxed),
            features_sanitised: self.features_sanitised.load(Ordering::Relaxed),
            model_fallbacks: self.model_fallbacks.load(Ordering::Relaxed),
            batch_init_observations_skipped: self.batch_init_observations_skipped.load(Ordering::Relaxed),
            zero_sentinel_assessments: self.zero_sentinel_assessments.load(Ordering::Relaxed),
        }
    }
}

impl Default for AssessmentDegradationCounters {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for AssessmentDegradationCounters {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AssessmentDegradationCounters")
            .field("total_degraded", &self.total_degraded.load(Ordering::Relaxed))
            .finish_non_exhaustive()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Snapshot
// ═══════════════════════════════════════════════════════════════════════════════

/// Point-in-time snapshot of degradation counters.
#[derive(Clone, Debug, Default)]
pub struct AssessmentDegradationSnapshot {
    /// Total assessments that had any degradation.
    pub total_degraded: u64,
    /// Total signals sanitised across all assessments.
    pub signals_sanitised: u64,
    /// Total signal shape mismatches across all assessments.
    pub signals_shape_mismatched: u64,
    /// Total unknown signal names across all assessments.
    pub signals_unknown: u64,
    /// Total Sentinel slots zeroed due to NaN.
    pub sentinel_slots_zeroed: u64,
    /// Total features sanitised post-standardisation.
    pub features_sanitised: u64,
    /// Total model fallback-to-prior events.
    pub model_fallbacks: u64,
    /// Total cold-ramp observations skipped on a full command channel.
    pub batch_init_observations_skipped: u64,
    /// Total assessments completed without a reporting Sentinel.
    pub zero_sentinel_assessments: u64,
}
