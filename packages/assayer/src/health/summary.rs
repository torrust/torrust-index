// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Health summary and query types: the two tiers the queries are priced in
//! (´dec:health:tiered-queries´).
//!
//! Provides the `HealthSummary` (fast, ~100 ns) and `SystemHealthReport`
//! (comprehensive, ~20 ms) types returned by the three health query methods.
//!
//! # Cross-References
//!
//! - (´dec:health:non-exhaustive-report´) — why the report type stays open to
//!   fields it does not yet carry
//! - (´dec:health:on-demand-composite´) — the composite stage the
//!   comprehensive tier computes when it is asked, and may regress

use std::collections::{BTreeMap, HashMap};

use indexmap::IndexMap;

use super::blend_stats::BlendStatistics;
use super::composite::CompositeConvergenceStage;
use super::concordance::ConcordanceHealth;
use super::counters::AssessmentDegradationSnapshot;
use super::identity_tracker::{IdentityConvergenceHealth, IdentityConvergenceStage};
use super::latency::FeedbackLatencyStages;
use super::platt_tracker::PlattConvergenceState;
use super::published::DiscriminationMetrics;
use crate::error::LabelPathStop;
use crate::signal::SignalCacheHealth;
use crate::types::{DimensionId, ModelId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Health Summary (fast path, ~100 ns)
// ═══════════════════════════════════════════════════════════════════════════════

/// Lightweight health summary from two `ArcSwap`s + atomics.
///
/// Returned by `Assayer::health_summary()`. No locks acquired.
/// ~30 flat scalar fields.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct HealthSummary {
    // ─── Convergence ─────────────────────────────────────────────────
    /// Composite convergence stage.
    pub convergence_stage: CompositeConvergenceStage,

    /// Platt calibration convergence state.
    pub platt_state: PlattConvergenceState,

    /// Most recent calibration delta.
    pub last_delta_cal: f64,

    /// Total Platt refits completed.
    pub platt_refits_completed: u32,

    // ─── Label counters ──────────────────────────────────────────────
    /// Total labels processed.
    pub total_labels: u64,

    /// Labels eligible for model updates.
    pub eligible_labels: u64,

    // ─── Assessment counters ─────────────────────────────────────────
    /// Total assessments processed.
    pub total_assessments: u64,

    /// Degradation counters snapshot.
    pub degradation: AssessmentDegradationSnapshot,

    // ─── Blend statistics ────────────────────────────────────────────
    /// Blend weight statistics (percentiles, EWMA).
    pub blend_statistics: BlendStatistics,

    // ─── Promoted metrics ────────────────────────────────────────────
    /// Feature-stable outcome drift flag.
    pub feature_stable_outcome_drift: bool,

    /// Quantile error concentration (if computed).
    pub quantile_error_concentration: Option<f64>,

    // ─── CP5/CP6 ─────────────────────────────────────────────────────
    /// Total CP5 reverts.
    pub cp5_reverts: u64,

    /// Total CP6 reverts.
    pub cp6_reverts: u64,

    // ─── Discrimination ──────────────────────────────────────────────
    /// Aggregate AUC (if computed).
    pub auc_aggregate: Option<f64>,

    /// Recent AUC (if computed).
    pub auc_recent: Option<f64>,

    // ─── Precision (from health ArcSwap) ─────────────────────────────
    /// Maximum cheap diagonal ratio across all models.
    ///
    /// The ratio bounds the true condition number from below, so the maximum
    /// of it bounds the worst true condition number from below too
    /// (´def:monitoring:condition-number´).
    pub max_diagonal_ratio: f64,

    /// Maximum share of a least-confident direction held up by the spectral
    /// floor rather than by evidence, across all models
    /// (´dec:posterior:spectral-floor´).
    ///
    /// Zero where no model has rebuilt yet, which the compact tier does not
    /// distinguish from zero degradation: an aggregate has one number and the
    /// detailed tier carries the absence (´dec:health:tiered-queries´).
    pub max_floor_share: f64,

    /// Maximum synchronisation error (´def:monitoring:synchronisation-error´) across all models.
    pub max_sync_error: f64,

    /// Maximum, across all models, of the part of that reading the
    /// replenishment clamp put there (´req:gaussian:prior-replenishment-floor´).
    ///
    /// A maximum over a conservative quantity: it names the model leaning
    /// hardest on its prior, which is a convergence-state signal rather than
    /// a fault.
    pub max_prior_induced_sync_error: f64,

    /// Maximum, across all models, of what is left once that part is taken
    /// out — the drift the cadence is actually reacting to
    /// (´dec:posterior:adaptive-cadence´).
    ///
    /// This is the aggregate to watch. The reading above can sit high for
    /// ever on a model with exhausted coordinates without anything being
    /// wrong; this one rising is rounding the incremental maintenance has
    /// accumulated and a recomputation can remove.
    pub max_sync_error_residual: f64,

    /// `true` if any model has experienced a cascade terminus event.
    pub any_cascade_terminus: bool,

    /// `true` once the label path has stopped and no further label will be
    /// applied (´dec:surface:async-label´).
    ///
    /// The severe reading of this tier. Every other flag here says that some
    /// quantity has moved; this one says the engine has stopped learning,
    /// which is the one condition a poller of the cheap tier must not have to
    /// read the expensive one to discover. Why it stopped is in the detailed
    /// tier, because a cause is not a scalar (´dec:health:tiered-queries´).
    pub label_path_stopped: bool,

    // ─── Platt κ (from model snapshot ArcSwap) ───────────────────────
    /// Sister-model Platt calibration κ.
    pub platt_kappa_sister: f64,

    /// Anchor-model Platt calibration κ.
    pub platt_kappa_anchor: f64,

    /// Global positive-class prior from the published model snapshot.
    pub p_positive_global: f64,

    /// Eligible positive-class prior from the published model snapshot.
    pub p_positive_eligible: f64,

    // ─── Events ──────────────────────────────────────────────────────
    /// Health events dropped due to channel overflow.
    pub health_events_dropped: u64,

    /// Drift-reset events emitted.
    pub drift_resets: u64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// System Health Report (full, ~20 ms)
// ═══════════════════════════════════════════════════════════════════════════════

/// Comprehensive system health report.
///
/// Returned by `Assayer::full_health_report()`. Reads two `ArcSwap`s,
/// iterates `DashMap`, acquires `RwLock`s.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SystemHealthReport {
    /// Convergence health across all processes.
    pub convergence: ConvergenceHealth,

    /// Drift state per model.
    pub drift: HashMap<ModelId, DriftHealth>,

    /// Precision health per model.
    pub precision: HashMap<ModelId, PrecisionHealthDetail>,

    /// Calibration health.
    pub calibration: CalibrationHealth,

    /// Per-Sentinel health (insertion-order for deterministic export).
    pub sentinels: IndexMap<SentinelId, SentinelHealth>,

    /// Fraction of registered Sentinels with a cached report.
    pub sentinel_coverage: f64,

    /// Per-outcome-axis health (insertion-order for deterministic export).
    pub axes: IndexMap<OutcomeAxisId, AxisHealth>,

    /// Per-identity-dimension health (insertion-order for deterministic export).
    pub identity_dimensions: IndexMap<DimensionId, IdentityDimensionHealth>,

    /// Buffer health (pending assessment buffer).
    pub buffer: BufferHealth,

    /// Label submission queue health.
    pub label_queue: LabelQueueHealth,

    /// Standardisation health.
    pub standardisation: StandardisationHealth,

    /// Label integrity health.
    pub label_integrity: LabelIntegrityHealth,

    /// Assessment degradation summary.
    pub assessment_degradation: AssessmentDegradationSummary,

    /// Discrimination metrics (if computed).
    pub discrimination: Option<DiscriminationMetrics>,

    /// Blend statistics.
    pub blend_statistics: BlendStatistics,

    /// Concordance tracker health.
    pub concordance: ConcordanceHealth,

    /// Ledger health: entry counts, root EWMAs, and the traffic-weighted value the Ledger realises.
    pub ledger: LedgerHealth,

    /// Cross-layer observability readings whose inputs are available.
    pub observability: ObservabilityHealth,

    /// End-to-end feedback latency and its four-stage decomposition
    /// (´def:monitoring:feedback-latency´). The report stage, and so the
    /// total, is a lower bound.
    pub cross_layer: CrossLayerHealth,

    /// Importance ceiling health accumulated at label time.
    pub importance_ceiling: ImportanceCeilingHealth,

    /// What repeated marginalisation has cost the models.
    pub marginalisation: MarginalisationHealth,

    // ─── Global counters, emitted as metrics ─────────────────────────
    // (´dec:metrics:catalogue-membership´)
    /// Whether any model has experienced a Cholesky cascade terminus.
    pub cascade_terminus_active: bool,

    /// Why the label path stopped, or `None` while it is running: a numeric
    /// checkpoint judged the working copy corrupt and the rebuild from the last
    /// published snapshot was refused, so no further label is applied while
    /// assessments keep answering from that snapshot (´dec:surface:async-label´).
    /// The cause is carried whole here and reduced to a flag on the compact tier,
    /// because what a host does about it depends on which model was refused and
    /// how far into the label stream the engine was (´dec:health:tiered-queries´).
    pub label_path_stop: Option<LabelPathStop>,

    /// Health events lost due to channel capacity overflow.
    pub health_events_dropped: u64,

    /// Schur complement corrections skipped: marginalisations that adopted the
    /// kept block instead of the correction, whether by either guard, by the
    /// factorisation, or by the verification
    /// (´alg:gaussian:regularised-schur´).
    pub schur_corrections_skipped: u64,

    /// Signal-cache counters and occupancy.
    pub signal_cache: SignalCacheHealth,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sub-Types
// ═══════════════════════════════════════════════════════════════════════════════

/// What repeated marginalisation has cost the models
/// (´alg:gaussian:regularised-schur´).
///
/// Every lifecycle removal that folds a dimension out computes what its
/// approximation was worth, and these figures are those results folded across
/// the events this process has served. They are a monitor and not a bound: the
/// corpus composes no law over repeated marginalisations
/// (´inv:guarantee:structural-exactness´), so what the host reads here is a
/// trend that argues for a recompute, not a certificate about the posterior.
///
/// The counters are monotone across a deployment: the whole-state checkpoint
/// carries the marginalisation history through a process restart
/// (´dec:durability:checkpoint-journal´).
#[derive(Clone, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct MarginalisationHealth {
    /// Total marginalisations folded in, across every model and event.
    pub events: u64,

    /// Of which reported a measured error, the exact term having been
    /// numerically reachable.
    pub measured_events: u64,

    /// Of which reported an upper bound instead, the exact term having been
    /// out of reach — which is the regime a raised posture ceiling admits.
    pub bounded_events: u64,

    /// Of which could report no finite figure at all, the removed block
    /// having no usable least eigenvalue.
    pub unbounded_events: u64,

    /// Of which discarded the correction and adopted the kept block.
    pub corrections_skipped: u64,

    /// The sum of the per-event discarded precision mass, over the events that
    /// carried a figure. Measurements and bounds are summed alike, so it is an
    /// upper estimate; it is in the precision matrices' own units, which
    /// forgetting moves, so it is read as a trend rather than as a level.
    pub cumulative_discarded_precision_mass: f64,

    /// The largest share of a correction any single event lost, in `[0, 1]`.
    /// Dimensionless, and so — unlike the sum above — comparable across the
    /// whole life of a deployment. One means an event discarded a correction
    /// whole.
    pub worst_correction_loss_fraction: f64,
}

impl From<&crate::model::marginalise::MarginalisationErrorLedger> for MarginalisationHealth {
    fn from(ledger: &crate::model::marginalise::MarginalisationErrorLedger) -> Self {
        Self {
            events: ledger.events,
            measured_events: ledger.measured_events,
            bounded_events: ledger.bounded_events,
            unbounded_events: ledger.unbounded_events,
            corrections_skipped: ledger.corrections_skipped(),
            cumulative_discarded_precision_mass: ledger.cumulative_discarded_precision_mass,
            worst_correction_loss_fraction: ledger.worst_correction_loss_fraction,
        }
    }
}

/// Convergence health across all processes.
///
/// TODO ´todo:code:expand-this-to-the-full-convergence´: expand this to the full convergence health report shape:
/// eligible/assessment counters, batch init, Sentinel bootstraps,
/// concordance, Platt health, and per-identity convergence health — the
/// report type is open to them (´dec:health:non-exhaustive-report´).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ConvergenceHealth {
    /// Composite convergence stage.
    pub composite_stage: CompositeConvergenceStage,

    /// Platt calibration state.
    pub platt_state: PlattConvergenceState,

    /// Per-dimension identity convergence.
    pub identity_stages: HashMap<DimensionId, IdentityConvergenceStage>,
}

/// Per-model drift health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct DriftHealth {
    /// CUSUM positive accumulator.
    pub s_plus: f64,
    /// CUSUM negative accumulator.
    pub s_minus: f64,
    /// Mean absolute residual EWMA.
    pub mean_abs_residual: f64,
    /// Residual sign EWMA.
    pub sign_ewma: f64,
    /// Steps since last reset.
    pub steps_since_reset: u64,
}

/// What a rebuild's factorisation answered, as the health surface publishes it.
///
/// A mirror of the recomputation's own verdict rather than that type itself.
/// This surface carries readings and not model internals — the spectrum a
/// rebuild takes arrives here as the three scalars derived from it and never
/// as the reading the model holds — so a host matching on this matches the
/// published vocabulary and is not coupled to the recomputation's
/// (´dec:posterior:measured-adoption´).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[non_exhaustive]
pub enum PublishedRebuildVerdict {
    /// The factorisation succeeded.
    Clean,
    /// The factorisation was refused and the rebuild offered nothing.
    Refused,
}

/// Per-model precision health detail.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct PrecisionHealthDetail {
    /// The true condition number measured at the last rebuild, absent until
    /// one has run (´def:monitoring:condition-number´).
    pub kappa: Option<f64>,
    /// The cheap diagonal ratio, a lower bound on the true condition number.
    pub diagonal_ratio: f64,
    /// The least eigenvalue measured at the last rebuild.
    pub lambda_min: Option<f64>,
    /// The spectral floor in force (´dec:posterior:spectral-floor´).
    pub spectral_floor: Option<f64>,
    /// The share of the least-confident direction the floor holds up.
    pub floor_share: Option<f64>,
    /// Rebuilds that were needed and left the model no better, or that the
    /// factorisation refused — the alarm
    /// (´dec:posterior:measured-adoption´).
    pub alarms: u64,
    /// Visits that measured the model and rebuilt nothing
    /// (´dec:posterior:recomputation-trigger´).
    pub measurements: u64,
    /// The labels the model had absorbed when the last visit measured it.
    pub last_measurement_labels: u32,
    /// Rebuilds at which the spectral floor repaired the spectrum
    /// (´dec:posterior:spectral-floor´).
    pub floored_rebuilds: u64,
    /// Total Cholesky recomputes for this model.
    pub cholesky_recomputes: u64,
    /// Total cascade terminus events for this model.
    pub cascade_terminus_count: u64,
    /// Last synchronisation error (´def:monitoring:synchronisation-error´).
    pub last_sync_error: f64,
    /// The part of that reading the replenishment clamp put there
    /// (´req:gaussian:prior-replenishment-floor´).
    pub last_prior_induced_sync_error: f64,
    /// What is left of it once that part is taken out, and what the cadence
    /// acted on (´dec:posterior:adaptive-cadence´).
    pub last_sync_error_residual: f64,
    /// The rounding level of the reading that residual was taken out of
    /// (´def:monitoring:synchronisation-error´).
    pub last_sync_error_resolution: f64,
    /// Whether that residual was at or below that level, and therefore not
    /// evidence of drift however far above the threshold it stood.
    pub last_measurement_at_resolution: bool,
    /// The drift the last rebuild measured after itself, absent until one has
    /// run (´dec:posterior:measured-adoption´).
    ///
    /// The readings above are retaken at every visit; this one and the two
    /// below stand for the last time a covariance was recomputed, which on a
    /// healthy model is a long way back.
    pub last_sync_error_after: Option<f64>,
    /// What the last rebuild's factorisation answered, absent until one has
    /// run (´dec:posterior:measured-adoption´).
    pub last_rebuild_verdict: Option<PublishedRebuildVerdict>,
    /// Whether the last rebuild's offered covariance was adopted, absent until
    /// one has run (´dec:posterior:measured-adoption´).
    ///
    /// A rebuild is declined on either of two conditions — a drift after it
    /// that does not improve on the drift before it, or one at or over the
    /// model's width-scaled threshold — and the two readings beside this flag
    /// are what separate them.
    pub last_rebuild_adopted: Option<bool>,
    /// Effective recomputation interval after adaptive shortening or recovery.
    pub n_recompute_effective: u32,
    /// Clean high-error recomputations that shortened the effective interval.
    pub sync_error_shortenings: u64,
    /// Dimensions sitting at the prior floor, counted and reported
    /// (´req:monitoring:replenishment-floor´).
    pub dimensions_at_floor: usize,
}

/// Per-Sentinel health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct SentinelHealth {
    /// Whether the Sentinel has a report.
    pub has_report: bool,
    /// Structural diagnostics from the latest received report.
    ///
    /// This value is read from the same immutable report index as
    /// [`Self::has_report`], so its fields are mutually consistent. It is not
    /// jointly atomic with the separately maintained Ledger reading below.
    pub structural: Option<SentinelStructuralHealth>,
    /// Directional disagreement accumulators for each anomaly axis.
    pub alarm_outcome: AlarmOutcomeHealth,
    /// Ledger entry count.
    pub ledger_entry_count: usize,
    /// Seconds since last report received (`None` if never reported).
    pub report_age_seconds: Option<f64>,
    /// Mean absolute operational weight over the Sentinel's slot
    /// (´def:monitoring:encoding-effectiveness´): how much weight mass the
    /// model carries on the Sentinel's features, and not a reading of how much
    /// the outcome depends on them. `None` when the published dimension map
    /// does not yet carry the Sentinel's slot, so an unmeasured Sentinel is
    /// distinguishable from one carrying no weight.
    pub informativeness: Option<f64>,
    /// The slot's association with the outcome, as a multiple of the null
    /// floor (´def:monitoring:slot-association´): the screening reading, and
    /// the one an alert hangs on. `None` until the block has accumulated a
    /// window's worth of labelled evidence.
    pub association: Option<f64>,
    /// What removing the slot from the score would cost, as a fraction of the
    /// score's own loss (´def:monitoring:slot-contribution´): the confirming
    /// reading, and the literal cost of retiring the Sentinel. `None` on the
    /// same evidence floor.
    pub contribution: Option<f64>,
    /// How old this Sentinel's held report says its oldest observation already
    /// was when the Sentinel emitted it, in seconds
    /// (´def:monitoring:feedback-latency´).
    ///
    /// A **lower bound** on the report stage and never the stage itself: it
    /// stops at the producer's emission, and the forwarding delay after that
    /// is an explicitly unmeasured residual. `None` when the held report
    /// carried no age, or when there is no held report.
    ///
    /// Distinct from [`report_age_seconds`](Self::report_age_seconds), which
    /// is measured on this process's clock from the report's arrival. The two
    /// are on either side of that arrival and are not added here: what
    /// separates them is the unmeasured residual.
    pub report_stage_lower_bound_seconds: Option<f64>,
}

/// Label-time disagreement between one Sentinel's alarms and host outcomes.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct AlarmOutcomeHealth {
    /// High-alarm observations followed by benign outcomes, by anomaly axis.
    pub high_alarm_benign: [f64; crate::types::SCORING_AXIS_COUNT],
    /// Quiet observations followed by adverse outcomes, by anomaly axis.
    pub quiet_alarm_adverse: [f64; crate::types::SCORING_AXIS_COUNT],
}

/// Structural diagnostics relayed from a Sentinel's latest report.
#[derive(Clone, Debug, PartialEq)]
#[non_exhaustive]
pub struct SentinelStructuralHealth {
    /// Number of online competitive cells.
    pub competitive_cell_count: usize,
    /// Number of online cells in the full producing set.
    pub full_set_size: usize,
    /// G-tree depth range of the full producing set.
    pub depth_range: (u8, u8),
    /// Number of plateaus in the spatial contour.
    pub plateau_count: usize,
    /// Total contour importance.
    pub total_importance: f64,
    /// Number of terminal cells in the contour.
    pub contour_cell_count: usize,
    /// Structural splits since the previous report.
    pub splits_since_last_report: u32,
    /// Net structural removals since the previous report.
    pub net_removals_since_last_report: u32,
    /// Competitive-set importance range.
    pub importance_range: (f64, f64),
    /// Competitive-set V-tree depth range.
    pub v_depth_range: (usize, usize),
    /// Cells skipped because their suffix geometry was degenerate.
    pub degenerate_cells_skipped: usize,
}

/// Per-outcome-axis health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AxisHealth {
    /// Axis name.
    pub name: String,
    /// Condition number estimate.
    pub kappa_a: f64,
    /// Decay rate.
    pub gamma_a: f64,
}

/// Per-identity-dimension health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct IdentityDimensionHealth {
    /// Human-readable dimension name.
    pub name: String,
    /// What this key space is, in the host's own words.
    pub description: String,
    /// What coordinates mean and how shared prefixes should be read.
    pub coordinate_semantics: String,
    /// Convergence health snapshot.
    pub convergence: IdentityConvergenceHealth,
    /// Current competitive cells grouped by dyadic depth.
    pub competitive_cell_depth_distribution: std::collections::BTreeMap<u8, usize>,
    /// Lifetime assessments grouped by active competitive-indicator count.
    pub active_indicator_count_distribution: std::collections::BTreeMap<usize, u64>,
    /// The share of assessed traffic whose identity coordinate fell in one of
    /// this dimension's competitive cells (´tab:monitoring:dimension-health´).
    ///
    /// Folded from the distribution above rather than counted separately, so
    /// the two rows describe the same traffic by construction. `None` before
    /// any traffic has been assessed against the dimension, which is a
    /// different state from traffic that arrived and matched nothing: that
    /// reads zero, and zero is the reading the host duty acts on
    /// (´req:keyspace:host-duties´).
    pub competitive_coverage_fraction: Option<f64>,
    /// Identity observations dropped because the maintenance queue could not
    /// accept them.
    pub observations_dropped: u64,
    /// Mean absolute operational weight over the dimension's own block
    /// (´def:monitoring:dimension-informativeness´): how much weight mass the
    /// model carries on the dimension's features, and not a reading of how
    /// much the outcome depends on them. The per-Sentinel sibling of this
    /// reading is [`SentinelHealth::informativeness`], taken over a disjoint
    /// set of positions, so the two are comparable readings and neither
    /// aggregates the other. `None` when the published dimension map does not
    /// yet carry the dimension's block, so an unmeasured dimension is
    /// distinguishable from one carrying no weight.
    pub informativeness: Option<f64>,
    /// Mean absolute operational weight over the dimension's competitive
    /// indicators — one position per competitive cell
    /// (´def:dimension:competitive-range´) — read as the weight mass the model
    /// carries on the dimension's cells, and not as a reading of whether the
    /// outcome depends on them.
    ///
    /// The same functional as [`informativeness`](Self::informativeness) over
    /// a different population of positions, and the populations are disjoint:
    /// that one is over the dimension's fixed-width block, which keeps its
    /// extent for the life of the dimension, and this one is over the
    /// indicators, whose count turns over with every restructuring. A
    /// dimension can carry weight in one and none in the other, which is why
    /// both are published and neither substitutes for the other
    /// (´def:monitoring:dimension-informativeness´).
    ///
    /// `None` when the published dimension map names no competitive-indicator
    /// position for the dimension — a dimension whose set has not formed yet
    /// has no cells to read weight over — and zero when the positions it names
    /// carry no weight.
    pub cell_weight_mass: Option<f64>,
    /// The block's association with the outcome, as a multiple of the null
    /// floor (´def:monitoring:slot-association´). The per-Sentinel sibling is
    /// [`SentinelHealth::association`]; both are the same functional over
    /// disjoint blocks.
    pub association: Option<f64>,
    /// What removing the block from the score would cost, as a fraction of
    /// the score's own loss (´def:monitoring:slot-contribution´). The
    /// per-Sentinel sibling is [`SentinelHealth::contribution`].
    pub contribution: Option<f64>,
}

/// Calibration health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct CalibrationHealth {
    /// Condition number for the sister model.
    pub kappa_sister: f64,
    /// Condition number for the anchor model.
    pub kappa_anchor: f64,
    /// Most recent calibration delta.
    pub delta_cal: f64,
    /// Total Platt refits completed.
    pub refits_completed: u32,
    /// Labels processed since the last Platt refit.
    pub labels_since_refit: u32,
    /// Total calibration buffer entries.
    pub buffer_total: usize,
    /// Weighted sister-regime sample count in the calibration buffer
    /// (´constr:platt:buffer´).
    pub sister_regime_records: f64,
    /// Weighted anchor-regime sample count in the calibration buffer
    /// (´constr:platt:buffer´).
    pub anchor_regime_records: f64,
    /// `true` when the anchor regime has too few records for refitting and its
    /// parameter has frozen (´disc:platt:regime-transition´). Reported because
    /// a frozen parameter is stale the moment the anchor reactivates.
    pub anchor_regime_frozen: bool,
    /// Labels since the anchor regime was last fitted, the other half of the
    /// starved-regime reading (´disc:platt:regime-transition´).
    pub labels_since_last_anchor_fit: u64,
    /// Current Platt convergence state.
    pub convergence_state: PlattConvergenceState,
}

/// Pending buffer health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct BufferHealth {
    /// Configured capacity.
    pub capacity: usize,
    /// Current utilisation.
    pub utilisation: usize,
    /// Lifetime pending entries inserted.
    pub insertions: u64,
    /// Lifetime live pending entries evicted at capacity.
    pub evictions: u64,
    /// Lifetime share of inserted pending entries that were evicted while
    /// still live. This is successful live evictions divided by insertions.
    pub eviction_rate: f64,
    /// Age in seconds of the oldest live pending entry, or `None` when the
    /// buffer is empty.
    pub oldest_pending_age_seconds: Option<f64>,
}

/// Bounded label submission queue health (´req:publication:label-queue´).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LabelQueueHealth {
    /// Configured queue capacity.
    pub capacity: usize,
    /// Submissions currently waiting for the model owner.
    pub depth: usize,
    /// Lifetime submissions refused by a full queue.
    pub drops: u64,
}

/// Fleet-wide observability derived from current producer state.
#[derive(Clone, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct ObservabilityHealth {
    /// Traffic-weighted share of measurement cells below the configured noise
    /// influence maturity threshold. Absent until a report carries traffic.
    pub maturity_coverage: Option<f64>,
    /// Share of active Ledger cells at or above the configured eligible-label
    /// adequacy threshold. Absent until a Ledger contains a cell.
    pub resolution_utilisation: Option<f64>,
}

/// Standardisation health (´tab:monitoring:standardisation-transition´).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct StandardisationHealth {
    /// Version of the published model snapshot that supplies this reading.
    pub snapshot_version: u64,
    /// Where the cold prior-mass ramp stands now.
    pub phase: crate::feature::standardisation::StandardisationPhase,
    /// Accepted cold-ramp observations now, from zero through `N_init`.
    /// Maturity is this against the horizon and is not stored beside it.
    pub accepted_observations: usize,
    /// Features at variance floor, excluding the unstandardised bias.
    pub features_at_floor: usize,
}

/// Label integrity health.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct LabelIntegrityHealth {
    /// Total labels processed.
    pub total_labels: u64,
    /// Labels eligible for updates.
    pub eligible_labels: u64,
    /// Eligibility rate (eligible / total).
    pub eligibility_rate: f64,
    /// Per-action label counts.
    pub per_action: HashMap<crate::types::Action, u64>,
    /// Non-finite valences repaired to zero at the label boundary
    /// (´dec:surface:sanitise-not-reject´).
    pub valences_sanitised: u64,
    /// Non-finite per-axis outcomes dropped at the label boundary
    /// (´dec:surface:sanitise-not-reject´).
    pub outcomes_dropped: u64,
}

/// Assessment degradation summary.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct AssessmentDegradationSummary {
    /// Total assessments.
    pub total_assessments: u64,
    /// Fraction of assessments with degradation.
    pub degraded_fraction: f64,
    /// Per-category counts.
    pub counters: AssessmentDegradationSnapshot,
}

/// End-to-end feedback latency, decomposed into the four stages that make it
/// up (´def:monitoring:feedback-latency´).
///
/// The point of the decomposition is that the stages are not comparable in
/// size: the human in the loop normally dominates, and a deployment that
/// answers slow feedback by tuning its queue or its publication cadence is
/// optimising the two smallest terms. Reporting the total alone would hide
/// exactly the fact an operator needs.
///
/// **The first stage is a lower bound.** It is built from the age each
/// producing Sentinel measured on its own clock, which stops at the moment
/// that Sentinel emitted its report; how long the host then held the report
/// before handing it over is measured by nothing and is left as an explicitly
/// unmeasured residual. The stage is therefore *at least* what is reported
/// here, and the total inherits the same qualification. Measuring the residual
/// would mean comparing two machines' clocks, and the figure's whole value is
/// that it compares none.
///
/// The three local stages are read off one injected monotonic clock, so their
/// differences are properties of the scenario rather than of how busy the
/// machine was. All four are smoothed over the same stream — one observation
/// per published label — so the total is a total of one journey rather than a
/// sum of averages taken over four different populations.
#[derive(Clone, Copy, Debug, Default, PartialEq)]
#[non_exhaustive]
pub struct CrossLayerHealth {
    /// The four stages as they stood at the last publication.
    pub stages: FeedbackLatencyStages,
    /// The sum of the stages that have readings, absent when none has.
    ///
    /// A lower bound whenever the report stage contributes.
    pub total_seconds_lower_bound: Option<f64>,
    /// The largest report-stage lower bound across the reports the fleet
    /// currently holds, in seconds; `None` when no held report carries an age.
    ///
    /// Scanned at query time from the current report indexes rather than
    /// smoothed over labels, so it answers a different question from
    /// [`stages`](Self::stages): not what the labels being published have been
    /// waiting, but how stale the worst evidence in hand is right now.
    pub max_report_stage_lower_bound_seconds: Option<f64>,
}

/// Per-Sentinel Ledger health: what the Ledger holds and what it realises
/// (´def:monitoring:ledger-value´).
///
/// The structural, maturity and traffic-weighted readings are scanned under the
/// existing `RwLock` read guard. Cell-set deltas are lifetime counters
/// accumulated from successful report ingestions.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct SentinelLedgerHealth {
    /// Total entries in this Sentinel's Ledger.
    pub entry_count: usize,
    /// Deepest active depth tier.
    pub max_depth: u8,
    /// Entry count at each active depth tier, ordered by depth.
    pub depth_histogram: BTreeMap<u8, usize>,
    /// Cells created by successful report ingestions over this process lifetime.
    pub cells_created: usize,
    /// Cells deleted by successful report ingestions over this process lifetime.
    pub cells_deleted: usize,
    /// Entries holding fewer eligible labels than the materiality threshold
    /// (´def:monitoring:immature-cells´).
    pub immature_cell_count: usize,
    /// Share of this Sentinel's assessed traffic in cells whose remembered
    /// adverse rate stands above the half-deviation materiality boundary — the
    /// traffic the Ledger is actually discriminating on
    /// (´def:monitoring:ledger-value´).
    pub value_realised: f64,
    /// Share of this Sentinel's assessed traffic in cells the Ledger would
    /// discriminate on but for insufficient evidence: cells whose true adverse
    /// rate clears the same boundary while their eligible arrival rate leaves
    /// the attenuated rate short of it (´thm:ledger:materiality´).
    pub attenuation_limited_fraction: f64,
    /// Root entry adverse outcome rate EWMA.
    pub root_adverse_rate: f64,
    /// Root entry compressed valence EWMA.
    pub root_compressed_valence: f64,
    /// Root entry raw valence EWMA.
    pub root_raw_valence: f64,
}

/// Aggregate Ledger health across all Sentinels: the fleet-wide reading of what
/// the Ledger is realising (´def:monitoring:ledger-value´).
///
/// The two traffic shares are reported together because neither is diagnostic
/// alone. A low realised value with a low attenuation-limited share says the
/// Ledger has nothing to say; the same low value with a high attenuation-limited
/// share says it would have something to say given evidence, and the two call
/// for opposite responses.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct LedgerHealth {
    /// Per-Sentinel Ledger breakdown.
    pub per_sentinel: IndexMap<SentinelId, SentinelLedgerHealth>,
    /// Total cells tracked across all Sentinels.
    pub total_cells_tracked: usize,
    /// Total immature cells across all Sentinels
    /// (´def:monitoring:immature-cells´).
    pub total_immature_cells: usize,
    /// Fleet-wide traffic-weighted value realised
    /// (´def:monitoring:ledger-value´).
    pub value_realised: f64,
    /// Fleet-wide traffic share the Ledger would discriminate on but for
    /// insufficient evidence (´thm:ledger:materiality´).
    pub attenuation_limited_fraction: f64,
}

/// Importance ceiling health accumulated at label time.
#[derive(Clone, Debug, Default)]
#[non_exhaustive]
pub struct ImportanceCeilingHealth {
    /// Fraction of operational labels whose balancing weight hit the ceiling.
    pub ceiling_binding_fraction: Option<f64>,
    /// Operational positive-class weight divided by total weight.
    pub gradient_balance: Option<f64>,
    /// Fraction of eligible sister labels whose balancing weight hit the ceiling.
    pub sister_ceiling_binding_fraction: Option<f64>,
    /// Sister positive-class weight divided by total eligible weight.
    pub sister_gradient_balance: Option<f64>,
}
