// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Health monitoring types.
//!
//! This module provides:
//!
//! - [`DegradationContext`](degradation::DegradationContext) — per-assessment
//!   degradation events from NaN checkpoints CP1–CP4
//! - [`ConcordanceTracker`](concordance::ConcordanceTracker) — rolling window
//!   of per-axis z-scores with threshold recalibration
//! - [`PlattConvergenceTracker`](platt_tracker::PlattConvergenceTracker) —
//!   tracks Platt calibration convergence
//! - [`IdentityConvergenceTracker`](identity_tracker::IdentityConvergenceTracker) —
//!   per-dimension competitive set stability tracking
//! - [`CompositeConvergenceStage`](composite::CompositeConvergenceStage) —
//!   overall system convergence stage
//! - [`HealthEvent`](events::HealthEvent) — health and convergence events
//! - [`DriftState`](drift::DriftState) — per-model CUSUM drift accumulators
//! - [`BlendStatisticsTracker`](blend_stats::BlendStatisticsTracker) — blend
//!   weight distribution tracking, one concrete tracker per process
//!   (´dec:health:concrete-trackers´)
//! - [`FeedbackLatencyTracker`](latency::FeedbackLatencyTracker) — the four
//!   stages of end-to-end feedback latency, smoothed as labels publish
//!   (´def:monitoring:feedback-latency´)
//! - [`BlockLegibilityEvidence`](legibility::BlockLegibilityEvidence) — the
//!   stored evidence behind one entity block's two legibility readings,
//!   folded prequentially at label time
//!   (´def:monitoring:slot-association´), (´def:monitoring:slot-contribution´)
//! - [`AssessmentDegradationCounters`](counters::AssessmentDegradationCounters) — atomic
//!   degradation event counters, the aggregate face of degradation that
//!   travels in band (´cor:degradation:in-band-travel´)
//! - [`PublishedHealthSummary`](published::PublishedHealthSummary) — second
//!   `ArcSwap`, published independently of the model snapshot
//!   (´dec:health:independent-publication´)
//!
//! # Cross-References
//!
//! - (´dec:degradation:retain-and-flag´) — why a degraded value is flagged
//!   rather than failed
//! - (´dec:retention:pending-map´) — the concurrent map that carries a
//!   `DegradationContext` with its assessment
//! - (´tab:warmup:stages´) — the warm-up stages the convergence trackers report
//!   against
//! - (´dec:health:tiered-queries´) — the cost tiers these types are read through
//! - (´alg:monitoring:drift-cusums´) — the prediction-drift detector behind
//!   `DriftState`

mod blend_stats;
mod composite;
mod concordance;
mod counters;
mod degradation;
mod drift;
mod events;
mod identity_tracker;
mod latency;
mod legibility;
mod platt_tracker;
mod published;
mod summary;

// Re-exports for crate-internal use and integration tests.
// Items are `pub` (not `pub(crate)`) to allow re-export from the crate root,
// but remain hidden from the public API because this module is private.
pub use blend_stats::{BlendStatistics, BlendStatisticsConfig, BlendStatisticsTracker};
pub use composite::{CompositeConvergenceStage, compute_composite_stage};
#[allow(unused_imports)]
pub use concordance::{
    ConcordanceCheckpointState, ConcordanceConfig, ConcordanceHealth, ConcordanceTracker, EntityConcordanceHealth,
    PerEntityConcordanceTracker,
};
#[allow(unused_imports)]
pub use counters::{AssessmentDegradationCounters, AssessmentDegradationSnapshot, DriftResetCounter};
pub use degradation::DegradationContext;
pub use drift::{DriftConfig, DriftState};
#[allow(unused_imports)]
pub use events::{
    ConvergenceEvent, DEFAULT_EVENT_CHANNEL_CAPACITY, HealthEvent, LifecycleHealthEvent, PrecisionHealthEvent,
    check_convergence_event, create_event_channel, emit_drift_reset_event, emit_health_event,
};
#[allow(unused_imports)]
pub use identity_tracker::{IdentityConvergenceHealth, IdentityConvergenceStage, IdentityConvergenceTracker};
pub use latency::{FeedbackLatencyStages, FeedbackLatencyTracker};
#[allow(unused_imports)]
pub use legibility::{BlockLegibilityEvidence, LegibilityReadings, null_association_floor};
#[allow(unused_imports)]
pub use platt_tracker::{DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT};
pub use platt_tracker::{PlattConvergenceState, PlattConvergenceTracker, PlattRefitResult};
pub use published::compute_discrimination_metrics_with_monitoring;
#[allow(unused_imports)]
pub use published::{DiscriminationMetrics, ModelPrecisionHealth, PublishedHealthSummary, compute_discrimination_metrics};
pub use summary::{
    AlarmOutcomeHealth, AssessmentDegradationSummary, AxisHealth, BufferHealth, CalibrationHealth, ConvergenceHealth,
    CrossLayerHealth, DriftHealth, HealthSummary, IdentityDimensionHealth, ImportanceCeilingHealth, LabelIntegrityHealth,
    LabelQueueHealth, LedgerHealth, MarginalisationHealth, ObservabilityHealth, PrecisionHealthDetail, PublishedRebuildVerdict,
    SentinelHealth, SentinelLedgerHealth, SentinelStructuralHealth, StandardisationHealth, SystemHealthReport,
};
