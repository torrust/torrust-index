// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Published health summary: the second `ArcSwap`, swapped independently of
//! the model snapshot (´dec:health:independent-publication´).
//!
//! This is the health counterpart to `ModelSnapshot`. The model owner
//! publishes it alongside the model snapshot at step 17 (snapshot first,
//! health second — intermediate state is benign).
//!
//! # Cross-References
//!
//! - (´dec:retention:health-independent´) — what the second swap costs and
//!   buys against publishing health with the snapshot
//! - (´dec:health:tiered-queries´) — the cheap tier this swap is what a
//!   frequent poller reads

#![allow(dead_code)]

use std::collections::HashMap;

use super::composite::CompositeConvergenceStage;
use super::concordance::PerEntityConcordanceTracker;
use super::drift::DriftState;
use super::identity_tracker::IdentityConvergenceTracker;
use super::latency::FeedbackLatencyStages;
use super::legibility::LegibilityReadings;
use super::platt_tracker::{PlattConvergenceState, PlattConvergenceTracker};
use super::summary::{AlarmOutcomeHealth, ImportanceCeilingHealth, MarginalisationHealth, PublishedRebuildVerdict};
use crate::feature::standardisation::StandardisationPhase;
use crate::types::{Action, DimensionId, ModelId, OutcomeAxisId, SentinelId};

/// A single calibration entry unpacked for discrimination computation.
///
/// `(rho_eff, positive, anchor_weight, uncertainty, axis_data)` — the
/// uncertainty is the assessment-time probability-space `σ_p̂`, the
/// denominator of the standardised residual
/// (´alg:monitoring:empirical-coverage´).
type CalibrationRow = (f64, bool, f64, f64, Vec<(OutcomeAxisId, f64, f64)>);

// ═══════════════════════════════════════════════════════════════════════════════
// Published Health Summary
// ═══════════════════════════════════════════════════════════════════════════════

/// Published health summary, shared via a second `ArcSwap`.
///
/// Created by `WorkingCopy::to_health_summary()` at label step 17.
/// Readers access it through `Assayer::health_summary()` and
/// `Assayer::full_health_report()`.
#[derive(Clone, Debug)]
pub struct PublishedHealthSummary {
    // ─── Convergence trackers (cloned from working copy) ─────────────
    /// Platt calibration tracker state.
    pub platt_tracker: PlattConvergenceTracker,

    /// Per-dimension identity convergence trackers.
    pub identity_trackers: HashMap<DimensionId, IdentityConvergenceTracker>,

    // ─── Precision health ────────────────────────────────────────────
    /// Per-model precision health (κ, sync error, regularisation count).
    pub precision_health: HashMap<ModelId, ModelPrecisionHealth>,

    // ─── Label counters ──────────────────────────────────────────────
    /// Total labels processed.
    pub total_labels: u64,

    /// Labels that were eligible for model updates.
    pub eligible_labels: u64,

    /// Per-action label counts.
    pub labels_by_action: HashMap<Action, u64>,

    /// Bounded per-entity concordance state.
    pub per_entity_concordance: PerEntityConcordanceTracker,

    /// Operational and sister importance-ceiling readings.
    pub importance_ceiling: ImportanceCeilingHealth,

    /// Per-Sentinel alarm/outcome disagreement accumulators.
    pub alarm_outcome: HashMap<SentinelId, AlarmOutcomeHealth>,

    /// Per-Sentinel legibility readings, reduced from the evidence the model
    /// owner accumulates (´def:monitoring:slot-association´),
    /// (´def:monitoring:slot-contribution´).
    ///
    /// The readings ride the summary and the evidence does not. Both readings
    /// are ratios of accumulated quantities and the ratio is what a reader
    /// wants; carrying the accumulators here would put a block's whole moment
    /// arithmetic on a surface read on every poll.
    pub sentinel_legibility: HashMap<SentinelId, LegibilityReadings>,

    /// Per-dimension legibility readings, the same two readings taken over
    /// each identity dimension's own block.
    pub dimension_legibility: HashMap<DimensionId, LegibilityReadings>,

    // ─── Standardisation state ───────────────────────────────────────
    /// Where the cold prior-mass ramp stands at the moment this report was
    /// taken (´tab:monitoring:standardisation-transition´).
    ///
    /// Current, not retrospective: the compact pair riding an assessment
    /// describes the coordinate system that one result was scored in, and
    /// during a transition the two legitimately disagree by however many
    /// advances fell between them.
    pub standardisation_phase: StandardisationPhase,

    /// Accepted cold-ramp observations at the moment this report was taken.
    ///
    /// Reported beside the phase because the phase alone says which of three
    /// regimes the ramp is in and nothing about how far along it has come
    /// (´dec:health:standardisation-phase-reported´).
    pub standardisation_observations: usize,

    /// Number of features at the variance floor. The bias position is not
    /// counted: nothing standardises it.
    pub features_at_floor: usize,

    // ─── Platt calibration state ─────────────────────────────────────
    /// Most recent calibration delta.
    pub last_delta_cal: f64,

    /// Current Platt convergence state.
    pub platt_state: PlattConvergenceState,

    // ─── CP5/CP6 counters ────────────────────────────────────────────
    /// Total CP5 reverts (working copy NaN detected).
    pub cp5_reverts: u64,

    /// Total CP6 reverts (snapshot NaN detected).
    pub cp6_reverts: u64,

    // ─── Cached discrimination (set at Platt refit) ──────────────────
    /// Discrimination metrics from last Platt refit.
    pub discrimination: Option<DiscriminationMetrics>,

    // ─── Promoted metrics ────────────────────────────────────────────
    /// Feature-stable outcome drift: `true` when drift CUSUM rising
    /// and standardisation EWMAs stable.
    pub feature_stable_outcome_drift: bool,

    /// Quantile error concentration from last Platt refit.
    pub quantile_error_concentration: Option<f64>,

    // ─── Drift state ─────────────────────────────────────────────────
    /// Per-model drift accumulators.
    pub drift_state: HashMap<ModelId, DriftState>,

    /// Composite convergence stage at publish time.
    pub convergence_stage: CompositeConvergenceStage,

    // ─── Marginalisation approximation ───────────────────────────────
    /// What repeated marginalisation has cost the models
    /// (´alg:gaussian:regularised-schur´).
    pub marginalisation: MarginalisationHealth,

    // ─── Feedback latency ────────────────────────────────────────────
    /// The four smoothed stages of end-to-end feedback latency, as they stood
    /// when this summary was published (´def:monitoring:feedback-latency´).
    ///
    /// Measured on the model owner's thread, one observation per published
    /// label, and carried here because that is the only thread that sees all
    /// four boundaries. The first stage is a lower bound.
    pub feedback_latency: FeedbackLatencyStages,
}

impl Default for PublishedHealthSummary {
    fn default() -> Self {
        Self {
            platt_tracker: PlattConvergenceTracker::new(),
            identity_trackers: HashMap::new(),
            precision_health: HashMap::new(),
            total_labels: 0,
            eligible_labels: 0,
            labels_by_action: HashMap::new(),
            per_entity_concordance: PerEntityConcordanceTracker::default(),
            importance_ceiling: ImportanceCeilingHealth::default(),
            alarm_outcome: HashMap::new(),
            sentinel_legibility: HashMap::new(),
            dimension_legibility: HashMap::new(),
            standardisation_phase: StandardisationPhase::default(),
            standardisation_observations: 0,
            features_at_floor: 0,
            last_delta_cal: 0.0,
            platt_state: PlattConvergenceState::Initial,
            cp5_reverts: 0,
            cp6_reverts: 0,
            discrimination: None,
            feature_stable_outcome_drift: false,
            quantile_error_concentration: None,
            drift_state: HashMap::new(),
            convergence_stage: CompositeConvergenceStage::default(),
            marginalisation: MarginalisationHealth::default(),
            feedback_latency: FeedbackLatencyStages::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Model Precision Health
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-model precision health state.
///
/// Carries the aggregate counters the health queries read
/// (´dec:health:tiered-queries´) and the catalogue's entries are emitted from
/// (´dec:metrics:catalogue-membership´).
#[derive(Clone, Debug, Default)]
pub struct ModelPrecisionHealth {
    /// The model's true condition number, measured at the last rebuild, and
    /// absent until one has run (´def:monitoring:condition-number´).
    ///
    /// It is `None` rather than a sentinel before the first rebuild, because a
    /// condition number nobody has measured is not a condition number of one;
    /// filling it from the cheap diagonal ratio below would publish a lower
    /// bound where the field promises the quantity itself.
    pub kappa: Option<f64>,

    /// The cheap diagonal ratio, read on every label.
    ///
    /// The ratio of the largest precision diagonal entry to the smallest. It
    /// bounds the true condition number from below and does not estimate it,
    /// and the name says so (´def:monitoring:condition-number´).
    pub diagonal_ratio: f64,

    /// The least eigenvalue measured at the last rebuild, absent until one has
    /// run. Published beside the floor and the degradation reading so that a
    /// host can compute the reading itself.
    pub lambda_min: Option<f64>,

    /// The spectral floor in force since the last rebuild
    /// (´dec:posterior:spectral-floor´), absent until one has run.
    pub spectral_floor: Option<f64>,

    /// How much of the least-confident direction the floor is holding up
    /// rather than evidence, in `[0, 1]` (´dec:posterior:spectral-floor´).
    ///
    /// Zero-tending for a healthy model and one where nothing but the floor
    /// holds the weakest direction up. It is derived from the two readings
    /// above and published beside them rather than instead of them.
    pub floor_share: Option<f64>,

    /// Rebuilds that were needed and left this model no better, or that the
    /// factorisation refused — the alarm
    /// (´dec:posterior:measured-adoption´).
    ///
    /// A non-zero reading is a condition to look at, not a rate to expect. A
    /// rebuild runs only where a measurement found drift over the threshold
    /// and above the resolution of its own reading, so there is no longer any
    /// decline that is not this (´rep:assayer:declined-rebuild-cadence´).
    pub alarms: u64,

    /// Visits that measured this model and rebuilt nothing
    /// (´dec:posterior:recomputation-trigger´).
    ///
    /// The count a healthy model accumulates. It is published beside the
    /// recompute count rather than folded into it, because the two answer
    /// different questions: how often the cadence looked, and how often
    /// looking cost a factorisation.
    pub measurements: u64,

    /// The labels this model had absorbed when the last visit measured it.
    ///
    /// The three readings below were taken at that point, and the count is
    /// what places them: a reading at four labels and the same reading at a
    /// thousand say different things about the same model.
    pub last_measurement_labels: u32,

    /// Last synchronisation error (´def:monitoring:synchronisation-error´),
    /// the whole reading as the specification defines the quantity.
    pub last_sync_error: f64,

    /// The part of that reading the replenishment clamp put there rather than
    /// the arithmetic (´req:gaussian:prior-replenishment-floor´).
    ///
    /// Published beside the reading rather than instead of it, because the
    /// two answer different questions: the reading says how far the tracked
    /// pair has drifted apart, and this says how much of that distance a
    /// recomputation cannot close. It is conservative in direction — the
    /// covariance is wider than the inverse exactly along the directions the
    /// model has no evidence for — so a large figure here is a model leaning
    /// on its prior, not a model in error. Zero until the model's first
    /// rebuild has written a record to read it from.
    pub last_prior_induced_sync_error: f64,

    /// What is left of the reading once that part is taken out — the drift
    /// the arithmetic accumulated, and the quantity the cadence acts on
    /// (´dec:posterior:adaptive-cadence´).
    pub last_sync_error_residual: f64,

    /// The rounding level of the reading the residual was taken out of
    /// (´def:monitoring:synchronisation-error´).
    ///
    /// Published beside the residual so that a reader can see how far above
    /// its own resolution a residual stands rather than taking the verdict on
    /// trust. It is derived from the magnitudes of the two matrices the
    /// product was formed from and carries no configured constant.
    pub last_sync_error_resolution: f64,

    /// Whether that residual was at or below that level — the difference of
    /// two figures agreeing to the accuracy the arithmetic could deliver, and
    /// therefore not evidence of drift.
    ///
    /// A model whose prior-induced component is a modest share of its reading
    /// never reads true here. A model whose reading is almost entirely what
    /// the replenishment clamp put there does, and the flag is what stops its
    /// cadence shortening against the rounding of a subtraction.
    pub last_measurement_at_resolution: bool,

    /// The drift the last rebuild measured after itself, against the same
    /// precision matrix, and absent until a rebuild has run
    /// (´dec:posterior:measured-adoption´).
    ///
    /// The readings above stand for the last *visit*, which on a healthy
    /// model is a measurement that rebuilt nothing; this one and the two
    /// below stand for the last time a covariance was actually recomputed.
    /// The separation is what the two groups are for: a model can go a long
    /// way past its last rebuild without the readings beside it going stale,
    /// because they are retaken at every visit.
    pub last_sync_error_after: Option<f64>,

    /// What the last rebuild's factorisation answered, absent until one has
    /// run (´dec:posterior:measured-adoption´).
    ///
    /// A refused factorisation offered nothing, so its reading above is not a
    /// measurement of a covariance the model could have taken; the verdict is
    /// what separates that case from a rebuild that produced an answer the
    /// measurement then declined.
    pub last_rebuild_verdict: Option<PublishedRebuildVerdict>,

    /// Whether the last rebuild's offered covariance was adopted, absent until
    /// one has run (´dec:posterior:measured-adoption´).
    ///
    /// The counter above says how many rebuilds a model has declined over its
    /// life; this says what happened to the most recent one, which is the
    /// rebuild the two readings beside it were taken at.
    pub last_rebuild_adopted: Option<bool>,

    /// Effective recomputation interval after adaptive shortening or recovery.
    pub n_recompute_effective: u32,

    /// Clean high-error recomputations that shortened the effective interval.
    pub sync_error_shortenings: u64,

    /// Total Cholesky recomputes (clean + terminus).
    pub total_recomputes: u64,

    /// Rebuilds at which the spectral floor repaired the spectrum — the
    /// deficit was positive and the floored matrix was installed
    /// (´dec:posterior:spectral-floor´).
    pub floored_rebuilds: u64,

    /// Number of cascade terminus events (severe).
    pub cascade_terminus_events: u64,

    /// Dimensions sitting at the prior floor, counted and reported
    /// (´req:monitoring:replenishment-floor´).
    pub dimensions_at_floor: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Discrimination Metrics
// ═══════════════════════════════════════════════════════════════════════════════

/// Discrimination metrics computed at Platt refit time.
///
/// All `Option<f64>` fields are `None` when insufficient samples.
///
/// # Cross-References
///
/// - (´dec:health:cached-discrimination´) — computed once at refit and cached
/// - (´dec:health:non-exhaustive-report´) — why this type stays open to
///   fields it does not yet carry
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct DiscriminationMetrics {
    /// AUC over all calibration entries.
    pub auc_aggregate: Option<f64>,

    /// AUC over sister-regime entries (`anchor_weight` < 0.3).
    pub auc_sister_regime: Option<f64>,

    /// AUC over anchor-regime entries (`anchor_weight` >= 0.3).
    pub auc_anchor_regime: Option<f64>,

    /// AUC over the most recent `N_recent` = 500 entries
    /// (´tab:config:monitoring´).
    pub auc_recent: Option<f64>,

    /// Top decile lift: precision at top 10% / base rate.
    pub top_decile_lift: Option<f64>,

    /// Per-axis point-biserial correlation (predicted score vs binary outcome).
    ///
    /// Algebraically equivalent to Pearson's r on a continuous × binary
    /// pair (Tate 1954).  Significance testing differs from standard
    /// Pearson r — use the t-distribution with n − 2 df.
    pub per_axis_correlation: HashMap<OutcomeAxisId, f64>,

    /// `true` when the recent figure departs from the aggregate figure
    /// by more than the tolerance (´alg:monitoring:auc´).
    pub discrimination_instability: bool,

    /// Quantile error concentration (max/mean MAE ratio across quantiles).
    pub quantile_error_concentration: Option<f64>,

    /// Empirical fraction of standardised residuals with `|z| < 1`
    /// (´alg:monitoring:empirical-coverage´). Near 0.68 under calibrated
    /// Gaussian uncertainty; `None` below the per-regime sample gate.
    pub coverage_68: Option<f64>,

    /// Empirical fraction of standardised residuals with `|z| < 1.96`
    /// (´alg:monitoring:empirical-coverage´). Near 0.95 under calibrated
    /// Gaussian uncertainty; `None` below the per-regime sample gate.
    pub coverage_95: Option<f64>,

    /// Uncertainty inflation factor: the 0.975-quantile of `|z|` divided
    /// by 1.96 (´alg:monitoring:empirical-coverage´). `None` below the
    /// per-regime sample gate.
    pub uncertainty_inflation: Option<f64>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Discrimination Computation
// ═══════════════════════════════════════════════════════════════════════════════

/// Minimum samples per axis for correlation.
///
/// The diagnostic this gates is the axis analogue of rank discrimination
/// (´def:monitoring:axis-correlation´), which fixes that a minimum stands
/// without fixing what it is. It is fixed at
/// (´rem:health:axis-correlation-evidence´): a shipped operating point set
/// above the per-class minimum the discrimination metrics take, and one
/// that resolves the sign and rough size of an association rather than
/// its magnitude.
///
/// ´const:assayer:axis-correlation-gate´ (´alg:const:count´)
/// ´const:assayer:axis-correlation-gate-count-50´
const MIN_AXIS_CORRELATION_SAMPLES: usize = 50;

/// Tie tolerance for AUC rank assignment.
///
/// Calibrated `ρ_eff` values in [−3, 3] have ULP ≈ 4.4e-16.  Using bare
/// `f64::EPSILON` misses legitimate ties that differ by a few ULPs
/// after intermediate computation.  1e-12 stands about three and a
/// third decimal orders above that ULP and far below any score
/// difference the ranking is meant to resolve
/// (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:rank-tie-tolerance´ (´alg:const:scalar´)
/// ´const:assayer:rank-tie-tolerance-scalar-1en12´
const AUC_TIE_TOLERANCE: f64 = 1e-12;

/// Computes discrimination metrics from calibration data.
///
/// Pure function: buffer entries → metrics. Called inside `refit_platt_calibration()`.
///
/// # Single-sort optimisation
///
/// Sorts the full entry set once by `rho_eff` and assigns ranks with
/// tie correction in a single pass.  Per-partition AUCs (sister, anchor,
/// recent) are computed by summing the pre-assigned ranks for the
/// matching positive entries — no re-sort needed.
///
/// # Arguments
///
/// * `entries` — Slice of `(rho_eff, positive, anchor_weight, uncertainty, axis_data)` tuples
/// * `kappa_sister` / `kappa_anchor` — the current regime calibration
///   parameters, interpolated per entry for the coverage residuals
///   (´def:platt:regimes´)
/// * `n_cal_min` — the calibration minimum applied per regime as the
///   coverage sample gate (´req:platt:minimum-samples´)
#[must_use]
pub fn compute_discrimination_metrics(
    entries: &[CalibrationRow],
    kappa_sister: f64,
    kappa_anchor: f64,
    n_cal_min: usize,
) -> DiscriminationMetrics {
    let monitoring = crate::config::types::MonitoringConfig::default();
    compute_discrimination_metrics_with_monitoring(entries, kappa_sister, kappa_anchor, n_cal_min, &monitoring)
}

/// Computes discrimination with the host's monitoring thresholds.
#[must_use]
pub fn compute_discrimination_metrics_with_monitoring(
    entries: &[CalibrationRow],
    kappa_sister: f64,
    kappa_anchor: f64,
    n_cal_min: usize,
    monitoring: &crate::config::types::MonitoringConfig,
) -> DiscriminationMetrics {
    let min_class_count = monitoring.min_auc_class_count;
    if entries.len() < 2 * min_class_count {
        return DiscriminationMetrics::default();
    }

    // ── Single sort ──────────────────────────────────────────────────────
    //
    // Build (rho, positive, anchor_weight, original_index, rank) tuples
    // sorted by rho ascending.  Ranks assigned once with tie correction.
    let mut items: Vec<RankedEntry> = entries
        .iter()
        .enumerate()
        .map(|(idx, (rho, pos, w, _, _))| RankedEntry {
            rho: *rho,
            positive: *pos,
            anchor_weight: *w,
            original_index: idx,
            rank: 0.0,
        })
        .collect();

    items.sort_by(|a, b| a.rho.partial_cmp(&b.rho).unwrap_or(std::cmp::Ordering::Equal));
    assign_ranks(&mut items);

    // ── Partition AUCs via rank sums ─────────────────────────────────────
    let auc_aggregate = auc_from_ranks(items.iter().map(|e| (e.rank, e.positive)), min_class_count);

    let auc_sister_regime = auc_from_ranks(
        items.iter().filter(|e| e.anchor_weight < 0.3).map(|e| (e.rank, e.positive)),
        min_class_count,
    );

    let auc_anchor_regime = auc_from_ranks(
        items.iter().filter(|e| e.anchor_weight >= 0.3).map(|e| (e.rank, e.positive)),
        min_class_count,
    );

    let recent_start = entries.len().saturating_sub(monitoring.recent_discrimination_window);
    let auc_recent = auc_from_ranks(
        items
            .iter()
            .filter(|e| e.original_index >= recent_start)
            .map(|e| (e.rank, e.positive)),
        min_class_count,
    );

    // ── Non-AUC metrics ─────────────────────────────────────────────────
    let top_decile_lift = compute_top_decile_lift(entries, min_class_count);
    let per_axis_correlation = compute_per_axis_correlation(entries);

    // Temporal comparison — recent against aggregate (´alg:monitoring:auc´).
    let discrimination_instability = match (auc_recent, auc_aggregate) {
        (Some(r), Some(a)) => (r - a).abs() > 0.1,
        _ => false,
    };

    let quantile_error_concentration = compute_quantile_error_concentration(entries);

    let (coverage_68, coverage_95, uncertainty_inflation) =
        compute_empirical_coverage(entries, kappa_sister, kappa_anchor, n_cal_min);

    DiscriminationMetrics {
        auc_aggregate,
        auc_sister_regime,
        auc_anchor_regime,
        auc_recent,
        top_decile_lift,
        per_axis_correlation,
        discrimination_instability,
        quantile_error_concentration,
        coverage_68,
        coverage_95,
        uncertainty_inflation,
    }
}

/// Computes empirical coverage of the reported uncertainty
/// (´alg:monitoring:empirical-coverage´).
///
/// Per entry, the probability-scale prediction is formed at the current
/// calibration parameter — `p̂ = σ(ρ_eff / κ_eff(w))` with the entry's
/// stored blend weight through the shared regime transition — and the
/// standardised residual divides by the entry's stored assessment-time
/// `σ_p̂`. The sample gate is the calibration minimum applied per
/// regime partition at the 0.3 boundary (´req:platt:minimum-samples´):
/// a partition below the gate contributes no residuals, and with both
/// below it all three outputs are `None`. Rows whose stored uncertainty
/// is non-finite or non-positive cannot be standardised and count
/// toward neither the gate nor the fractions.
///
/// The inflation factor's quantile is the empirical inverse CDF: the
/// `ceil(0.975·n)`-th order statistic of `|z|`, one-based.
fn compute_empirical_coverage(
    entries: &[CalibrationRow],
    kappa_sister: f64,
    kappa_anchor: f64,
    n_cal_min: usize,
) -> (Option<f64>, Option<f64>, Option<f64>) {
    let usable = |sigma: &f64| sigma.is_finite() && *sigma > 0.0;

    let sister_usable = entries.iter().filter(|(_, _, w, s, _)| *w < 0.3 && usable(s)).count();
    let anchor_usable = entries.iter().filter(|(_, _, w, s, _)| *w >= 0.3 && usable(s)).count();
    let sister_open = sister_usable >= n_cal_min;
    let anchor_open = anchor_usable >= n_cal_min;
    if !sister_open && !anchor_open {
        return (None, None, None);
    }

    let mut abs_z: Vec<f64> = entries
        .iter()
        .filter(|(_, _, w, s, _)| usable(s) && if *w < 0.3 { sister_open } else { anchor_open })
        .map(|(rho, positive, w, sigma, _)| {
            let g = crate::numerics::regime_transition(*w);
            let kappa_eff = (1.0 - g).mul_add(kappa_sister, g * kappa_anchor);
            let p_hat = crate::numerics::stable_sigmoid(rho / kappa_eff.max(1e-15));
            let y = if *positive { 1.0 } else { 0.0 };
            ((y - p_hat) / sigma).abs()
        })
        .collect();

    if abs_z.is_empty() {
        return (None, None, None);
    }

    #[allow(clippy::cast_precision_loss)]
    let n = abs_z.len() as f64;
    let within_68 = abs_z.iter().filter(|z| **z < 1.0).count();
    let within_95 = abs_z.iter().filter(|z| **z < 1.96).count();

    abs_z.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    #[allow(clippy::cast_possible_truncation)]
    #[allow(clippy::cast_sign_loss)]
    let q_index = ((0.975 * n).ceil() as usize).clamp(1, abs_z.len()) - 1;
    let inflation = abs_z[q_index] / 1.96;

    #[allow(clippy::cast_precision_loss)]
    (Some(within_68 as f64 / n), Some(within_95 as f64 / n), Some(inflation))
}

/// One entry with its pre-assigned rank (from the single global sort).
struct RankedEntry {
    /// Calibrated risk score `ρ_eff`.
    rho: f64,
    /// Whether the outcome was positive.
    positive: bool,
    /// Anchor weight at assessment time.
    anchor_weight: f64,
    /// Position in the original `entries` slice (for the recent-window filter).
    original_index: usize,
    /// Average rank assigned after tie correction.
    rank: f64,
}

/// Assigns average ranks to a sorted `items` slice, handling ties with
/// the `AUC_TIE_TOLERANCE` threshold.
fn assign_ranks(items: &mut [RankedEntry]) {
    let n = items.len();
    let mut i = 0;
    while i < n {
        let mut j = i + 1;
        while j < n && (items[j].rho - items[i].rho).abs() <= AUC_TIE_TOLERANCE {
            j += 1;
        }
        // Average rank for positions i..j (1-based).
        #[allow(clippy::cast_precision_loss)]
        let avg_rank = (i + 1 + j) as f64 / 2.0;
        for item in &mut items[i..j] {
            item.rank = avg_rank;
        }
        i = j;
    }
}

/// Computes AUC from pre-assigned ranks via the Mann-Whitney U statistic.
///
/// Accepts an iterator of `(rank, positive)` pairs from any partition.
/// Returns `None` if the partition has fewer than the configured minimum
/// samples in either class.
fn auc_from_ranks(pairs: impl Iterator<Item = (f64, bool)>, min_class_count: usize) -> Option<f64> {
    let mut rank_sum_pos: f64 = 0.0;
    let mut n_pos: usize = 0;
    let mut n_total: usize = 0;

    for (rank, positive) in pairs {
        n_total += 1;
        if positive {
            n_pos += 1;
            rank_sum_pos += rank;
        }
    }

    let n_neg = n_total - n_pos;
    if n_pos < min_class_count || n_neg < min_class_count {
        return None;
    }

    // U = R_pos - n_pos*(n_pos+1)/2;  AUC = U / (n_pos * n_neg)
    #[allow(clippy::cast_precision_loss)]
    let u = rank_sum_pos - (n_pos as f64 * (n_pos as f64 + 1.0)) / 2.0;
    #[allow(clippy::cast_precision_loss)]
    let auc = u / (n_pos as f64 * n_neg as f64);
    Some(auc)
}

/// Computes top-decile lift: precision at top 10% / base rate.
fn compute_top_decile_lift(entries: &[CalibrationRow], min_class_count: usize) -> Option<f64> {
    if entries.len() < 2 * min_class_count {
        return None;
    }

    let total_positive = entries.iter().filter(|(_, pos, _, _, _)| *pos).count();
    if total_positive == 0 {
        return None;
    }

    #[allow(clippy::cast_precision_loss)]
    let base_rate = total_positive as f64 / entries.len() as f64;
    if base_rate < f64::EPSILON {
        return None;
    }

    // Sort by rho descending, take top 10%
    let mut sorted: Vec<_> = entries.iter().map(|(rho, pos, _, _, _)| (*rho, *pos)).collect();
    sorted.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let top_n = (entries.len() / 10).max(1);
    let top_positive = sorted[..top_n].iter().filter(|(_, pos)| *pos).count();

    #[allow(clippy::cast_precision_loss)]
    let top_precision = top_positive as f64 / top_n as f64;
    Some(top_precision / base_rate)
}

/// Computes Pearson correlation per outcome axis.
fn compute_per_axis_correlation(entries: &[CalibrationRow]) -> HashMap<OutcomeAxisId, f64> {
    // Collect per-axis (predicted, actual) pairs
    let mut axis_data: HashMap<OutcomeAxisId, Vec<(f64, f64)>> = HashMap::new();

    for (_, _, _, _, axis_pairs) in entries {
        for &(axis_id, predicted, actual) in axis_pairs {
            axis_data.entry(axis_id).or_default().push((predicted, actual));
        }
    }

    let mut result = HashMap::new();
    for (axis_id, pairs) in &axis_data {
        if pairs.len() >= MIN_AXIS_CORRELATION_SAMPLES
            && let Some(r) = pearson_correlation(pairs)
        {
            result.insert(*axis_id, r);
        }
    }
    result
}

/// Computes the point-biserial correlation coefficient.
///
/// Algebraically equivalent to Pearson's r when one variable is
/// continuous and the other is binary (Tate, 1954).  Note that
/// the sampling distribution under the null differs from the
/// standard Pearson r distribution.
fn pearson_correlation(pairs: &[(f64, f64)]) -> Option<f64> {
    let n = pairs.len();
    if n < 2 {
        return None;
    }

    #[allow(clippy::cast_precision_loss)]
    let n_f = n as f64;
    let sx: f64 = pairs.iter().map(|(x, _)| x).sum();
    let sy: f64 = pairs.iter().map(|(_, y)| y).sum();
    let sxx: f64 = pairs.iter().map(|(x, _)| x * x).sum();
    let syy: f64 = pairs.iter().map(|(_, y)| y * y).sum();
    let sxy: f64 = pairs.iter().map(|(x, y)| x * y).sum();

    let numerator = n_f.mul_add(sxy, -(sx * sy));
    let denom_x = n_f.mul_add(sxx, -(sx * sx));
    let denom_y = n_f.mul_add(syy, -(sy * sy));

    let denominator = (denom_x * denom_y).sqrt();
    if denominator < f64::EPSILON {
        return None;
    }
    Some(numerator / denominator)
}

/// Computes quantile error concentration.
///
/// Partition entries into 10 quantiles of `rho_eff`, compute MAE per
/// quantile, report max/mean ratio.
fn compute_quantile_error_concentration(entries: &[CalibrationRow]) -> Option<f64> {
    if entries.len() < 100 {
        return None;
    }

    // Sort by rho_eff
    let mut sorted: Vec<_> = entries.iter().map(|(rho, pos, _, _, _)| (*rho, *pos)).collect();
    sorted.sort_by(|a, b| a.0.partial_cmp(&b.0).unwrap_or(std::cmp::Ordering::Equal));

    let n_quantiles = 10;
    let quantile_size = sorted.len() / n_quantiles;
    if quantile_size == 0 {
        return None;
    }

    let mut quantile_maes = Vec::with_capacity(n_quantiles);

    for q in 0..n_quantiles {
        let start = q * quantile_size;
        let end = if q == n_quantiles - 1 {
            sorted.len()
        } else {
            (q + 1) * quantile_size
        };
        let slice = &sorted[start..end];

        if slice.is_empty() {
            continue;
        }

        // MAE: mean absolute error between sigmoid(rho) and binary outcome
        #[allow(clippy::cast_precision_loss)]
        let mae: f64 = slice
            .iter()
            .map(|(rho, pos)| {
                let predicted = crate::numerics::stable_sigmoid(*rho);
                let actual = if *pos { 1.0 } else { 0.0 };
                (predicted - actual).abs()
            })
            .sum::<f64>()
            / slice.len() as f64;

        quantile_maes.push(mae);
    }

    if quantile_maes.is_empty() {
        return None;
    }

    #[allow(clippy::cast_precision_loss)]
    let mean_mae: f64 = quantile_maes.iter().sum::<f64>() / quantile_maes.len() as f64;
    if mean_mae < f64::EPSILON {
        return None;
    }

    let max_mae = quantile_maes.iter().copied().fold(0.0_f64, f64::max);
    Some(max_mae / mean_mae)
}
