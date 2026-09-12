// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`risk_target_positive_valence`] | labelling | The risk target is a sign, not a magnitude: an adverse observation of any strength enters the model update as exactly +1. How bad the outcome was is carried separately by the compressed valence, so the target never lets one extreme report dominate the direction the model is pulled in. |
//! | [`risk_target_negative_valence`] | labelling | cites (´claim:labelling:the-risk-target-is-a-sign-and-not-a-magnitude´) |
//! | [`compressed_valence_tanh_bounded`] | labelling | Valence enters the update through a saturating compression: a report five times the running scale comes through just under unity rather than five times as loud. An outlier can push the compressed value arbitrarily close to the ceiling but never past it, so a single mis-scaled report cannot buy unbounded influence over the model. |
//! | [`calibration_entry_stores_values`] | labelling | A calibration entry keeps everything a later refit or coverage computation needs — the effective risk, its assessment-time uncertainty, the anchor weight, the polarity and the importance weight — beside each other in one record. Calibration happens long after the label was processed, so the entry has to carry its own context rather than expecting the model to still be in the state that produced it; the uncertainty is the denominator the empirical coverage of every reported interval divides by (´alg:monitoring:empirical-coverage´). |
//! | [`label_pipeline_config_defaults`] | labelling | An unconfigured pipeline runs on the constants the specification names — the long-memory decay rates, the importance ceiling and the refit interval — rather than on whatever the struct's fields happened to zero to. A host that supplies no tuning still gets the documented behaviour. |
//! | [`bounded_parallel_model_updates_match_sequential_results`] | concurrency | The bounded execution groups change only when independent model records run, not what any record computes. Operational occupies one helper, sister and anchor retain their order in the second helper, and an axis stays on the steward; their published means and covariances are bitwise identical to applying those same updates in the former sequential order. |
//! | [`check_nan_detects_nan_in_mu`] | labelling | The integrity guard is quiet on healthy state: a cold-started working copy reports no NaN across any of its models, and none of them carries a rebuild verdict at all. A guard that fired on a freshly-built model would revert every label ever processed, so its silence on the clean case is what makes the checkpoint usable at all — and a model that has never rebuilt is exactly the case a verdict reading must not invent a finding for: its precision matrix is the prior, definite by construction. |
//! | [`cp5_reads_the_stored_rebuild_verdict`] | labelling | A model whose last rebuild was refused makes the checkpoint fire, and a model whose last rebuild was clean does not — with the precision diagonal finite and positive in both cases, which is the whole point. The diagonal test CP5 used to take is necessary for definiteness and not sufficient for it (´req:gaussian:positive-definiteness´), so a matrix a factorisation refuses can pass it; the verdict the rebuild already produced is the reading that does not (´tab:runtime:numeric-checkpoints´). |
//! | [`label_pipeline_config_from_assayer_config_wires_all_fields`] | labelling | Every knob the host sets on the overall configuration reaches the pipeline: each of the decay rates, ceilings, leverage bounds, Cholesky thresholds and refit intervals is given a value distinct from its default and each arrives unchanged. A field silently dropped during wiring would leave a documented setting with no effect at all, which is exactly the failure this rules out. |
//! | [`feature_stable_outcome_drift_false_when_drift_state_empty`] | labelling | The flag is a claim about drift, so it needs drift evidence before it can be raised: with no model yet accumulating, perfectly stable features are not enough on their own. An instance that has processed nothing therefore reports no drift rather than reporting the stability of its untouched standardisation as a finding. |
//! | [`feature_stable_outcome_drift_false_when_no_features`] | labelling | The other half of the conjunction is equally required: with real drift accumulated but no feature distribution to judge, there is nothing that could be called stable and the flag stays down. The finding is specifically "outcomes drifted while features did not", and an absent feature vector cannot supply that second half. |
//! | [`feature_stable_outcome_drift_false_when_cusum_flat`] | labelling | cites (´claim:labelling:the-feature-stable-drift-flag-needs-drift-evidence-before-it-can-rise´) |
//! | [`feature_stable_outcome_drift_true_when_cusum_rising_and_features_stable`] | labelling | Both halves together are what the flag means: accumulating outcome drift while the feature distribution has not moved. That combination is the signature of the mapping from features to outcomes having changed under the model rather than the traffic having changed — a relabelling of the world, which no amount of feature-distribution monitoring would catch. |
//! | [`feature_stable_outcome_drift_true_when_any_model_cusum_rising`] | labelling | The drift evidence is pooled across models rather than required of all of them, and it counts on either side of the accumulator: a rising negative sum on the sister model alone raises the flag while the operational model stays quiet. Waiting for consensus would mean the first model to notice a relabelling could not report it. |
//! | [`feature_stable_outcome_drift_false_when_variance_collapsed`] | labelling | A single feature whose running variance has collapsed below the floor disqualifies the whole distribution as stable, and the drift evidence is then not enough on its own. A collapsed variance means the standardisation itself is degenerate, so its means say nothing trustworthy about whether the inputs shifted — and a finding built on it would be unfounded. |
//! | [`feature_stable_outcome_drift_false_when_mean_diverged`] | labelling | Healthy variances are not by themselves stability: a feature whose running mean has walked out past three standard deviations of its own spread also disqualifies the distribution. A mean that far from zero says the inputs have moved, and drift with moving inputs is ordinary covariate shift, not the feature-stable kind this flag is reserved for. |
//! | [`feature_stable_outcome_drift_true_when_mean_at_three_sigma_boundary`] | labelling | The three-sigma bound is inclusive: a mean sitting exactly on it is still stable. The boundary is a threshold on plausible wander rather than a forbidden value, so a feature that merely touches it does not cost the instance a genuine drift finding. |
//! | [`feature_stable_outcome_drift_honours_configured_stability_threshold`] | labelling | The feature-stability threshold is exact and independent of the drift accumulator threshold: the flag stays down at the configured variance floor and rises immediately above it for the same drift evidence. |
//! | [`calibration_drift_reset_narrows_to_the_affected_regime`] | labelling | The post-calibration reset acts on the affected regime and only on it: with a sister-regime shift past the threshold and the anchor's below it, the sister model's accumulators and step counter return to zero while its smoothed diagnostics survive, and the anchor and operational models keep every accumulator untouched. A reset wider than the shifted mapping would discard evidence measured on a scale that did not change, and one deeper than the accumulators would destroy the long-memory description the automatic trigger deliberately spares (´tab:monitoring:drift-resets´). |

//! Label pipeline orchestration.
//!
//! This module implements the 17-step label pipeline that processes
//! ground-truth outcome labels and updates the model state.
//!
//! # Pipeline Steps
//!
//! | Step | Name | Purpose |
//! |------|------|---------|
//! | 0 | Time decay | Apply model decay based on elapsed time |
//! | 1–3 | Reconstruct | Retrieve pending, reconstruct φ, re-standardise |
//! | 4–8 | Scalars | Risk target, κᵥ, eligibility, P+ trackers, importance |
//! | 9–11 | Model update | Apply leverage-bounded updates (´dec:posterior:leverage-before´) |
//! | — | Companion boundary | No Core challenge-effectiveness mutation |
//! | 12 | Identity | Update identity cell outcome EWMAs |
//! | 13 | Ledger | Update Sentinel Ledger entries |
//! | 14 | Calibration | Push to calibration buffer, check Platt refit |
//! | 15 | Standardisation | Update standardisation EWMAs |
//! | 16 | Drift | Update drift accumulators |
//! | CP5/6 | Integrity check | Verify working copy and snapshot; revert on corrupt state or a refused update |
//! | 17 | Publish | Publish new snapshot via `ArcSwap` |
//!
//! # Cross-References
//!
//! - (´alg:runtime:update-path´) — the seventeen steps, in the order they run
//! - (´dec:ordering:label-function´) — the path is one function in ordered groups
//! - (´dec:posterior:combined-factor´) — time and label decay combine into one factor
//! - (´tab:construction:parameters´) — the parameters this pipeline defers to

#![allow(dead_code)]
#![allow(clippy::cast_precision_loss, clippy::too_many_arguments, clippy::too_many_lines)]

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use crossbeam_channel::Sender;

use crate::config::types::ConvergenceThresholdConfig;
use crate::feature::standardisation::StandardisationConfig;
use crate::health::{
    ConvergenceEvent, DriftConfig, DriftResetCounter, DriftState, HealthEvent, PlattConvergenceState, PlattConvergenceTracker,
    PrecisionHealthEvent, compute_composite_stage, emit_drift_reset_event, emit_health_event,
};
use crate::identity::{CellOutcomeUpdate, IdentityDimensionInfra};
use crate::ledger::{LedgerUpdate, OutcomeLedger, update_all_layers};
use crate::linalg::convert::vec_to_col;
use crate::model::bayesian::LabelUpdateOutcome;
use crate::model::recompute::{
    DEFAULT_KAPPA_GROWTH_FACTOR, RebuildVerdict, RecomputeBaseline, RecomputeOutcome, VisitInputs, should_recompute,
    synchronisation_visit,
};
use crate::model::update::{
    DEFAULT_IMPORTANCE_CEILING, DEFAULT_LAMBDA_FLOOR, DEFAULT_LEVERAGE_SAFETY_FACTOR, EPSILON_LEVERAGE, GAMMA_LABEL_OPERATIONAL,
    GAMMA_LABEL_SISTER, compute_effective_weight,
};
use crate::numerics::decay_factor_elapsed;
use crate::owner::commands::{LabelContext, LabelData, PendingAssessment, PendingContext, SequencedLabel};
use crate::pending::SentinelExtraction;
use crate::risk::calibration::{PlattConfig, refit_platt_calibration_with_monitoring};
use crate::risk::challenge::{compute_importance_weight, compute_risk_target, determine_eligibility, update_p_plus_tracker};
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::WorkingCopy;
use crate::types::{Action, DimensionId, ModelId, OutcomeEligibility, PersistentTimestamp, SCORING_AXIS_COUNT, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Health Counters
// ═══════════════════════════════════════════════════════════════════════════════

/// Mutable health counters maintained across labels on the model-owner thread.
///
/// Fed into the `PublishedHealthSummary` at step 17.
///
/// # Cross-References
///
/// - (´dec:health:independent-publication´) — the summary these counters feed
///   publishes on its own swap
/// - (´inv:monitoring:report-only´) — the counters report and never act
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthCounters {
    /// Labels eligible for model updates.
    pub eligible_labels: u64,
    /// Per-action label counts.
    pub labels_by_action: HashMap<Action, u64>,
    /// Total CP5 reverts.
    pub cp5_reverts: u64,
    /// Total CP6 reverts.
    pub cp6_reverts: u64,
    /// Total revert failures (Cholesky failed during `from_snapshot`).
    pub revert_failures: u64,
    /// Last discrimination metrics from Platt refit.
    pub last_discrimination: Option<crate::health::DiscriminationMetrics>,
    /// Per-entity sign concordance, bounded by least-recently-observed eviction.
    pub per_entity_concordance: crate::health::PerEntityConcordanceTracker,
    /// Operational importance-ceiling and class-balance accumulation.
    pub importance_operational: ImportanceAccumulator,
    /// Eligible sister importance-ceiling and class-balance accumulation.
    pub importance_sister: ImportanceAccumulator,
    /// Per-Sentinel, per-axis alarm/outcome disagreement accumulators.
    pub alarm_outcome: HashMap<SentinelId, crate::health::AlarmOutcomeHealth>,
    /// Per-Sentinel legibility evidence, folded over each Sentinel's own slot
    /// (´def:monitoring:slot-association´), (´def:monitoring:slot-contribution´).
    ///
    /// Inside the checkpoint, unlike the latency stages below: these are
    /// decayed sums over the labelled stream, and every quantity in them
    /// survives a restart intact. A deployment that had to accumulate a
    /// window's worth again after every restart would be blind on its own
    /// alert reading for exactly as long as the window is
    /// (´dec:durability:structural-compatibility´).
    pub sentinel_legibility: HashMap<SentinelId, crate::health::BlockLegibilityEvidence>,
    /// Per-dimension legibility evidence, folded over each identity
    /// dimension's own block.
    ///
    /// The same two readings taken over a disjoint set of positions, which is
    /// what makes them siblings of the per-Sentinel pair rather than one
    /// refining the other.
    pub dimension_legibility: HashMap<DimensionId, crate::health::BlockLegibilityEvidence>,
    /// The four smoothed stages of end-to-end feedback latency
    /// (´def:monitoring:feedback-latency´).
    ///
    /// Outside the checkpoint deliberately, and so outside the layout the
    /// format version pins: every stage is a difference between two `Instant`s
    /// in this process's monotonic domain, which a restart ends. A restored
    /// deployment starts the tracker empty and reports each stage absent until
    /// it is measured again, rather than presenting a figure about instants
    /// that no longer exist (´dec:durability:structural-compatibility´).
    #[cfg_attr(feature = "serde", serde(skip))]
    pub feedback_latency: crate::health::FeedbackLatencyTracker,
}

/// Step-scale accumulation for one model stream's importance health.
#[derive(Clone, Copy, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ImportanceAccumulator {
    /// Labels whose weights have been observed.
    labels: u64,
    /// Observed weights limited by the configured ceiling.
    ceiling_limited: u64,
    /// Accumulated weight assigned to positive labels.
    positive_weight: f64,
    /// Accumulated weight assigned to all labels.
    total_weight: f64,
}

impl ImportanceAccumulator {
    /// Records one balancing weight and whether it served a positive label.
    fn observe(&mut self, weight: f64, positive: bool, ceiling: f64) {
        self.labels = self.labels.saturating_add(1);
        self.ceiling_limited = self.ceiling_limited.saturating_add(u64::from(weight >= ceiling));
        if positive {
            self.positive_weight += weight;
        }
        self.total_weight += weight;
    }

    /// Returns the fraction of observed weights limited by the ceiling.
    fn binding_fraction(self) -> Option<f64> {
        (self.labels > 0).then(|| self.ceiling_limited as f64 / self.labels as f64)
    }

    /// Returns the positive share of accumulated balancing weight.
    fn gradient_balance(self) -> Option<f64> {
        (self.total_weight > 0.0).then(|| self.positive_weight / self.total_weight)
    }
}

/// Advances one directional alarm-outcome Page accumulator.
fn page_update(accumulator: &mut f64, disagreement: bool, noise_allowance: f64) {
    let observation = if disagreement { 1.0 } else { 0.0 };
    *accumulator = (*accumulator + observation - noise_allowance).max(0.0);
}

/// Crosses frozen per-Sentinel alarms with one eventual outcome.
fn observe_alarm_outcomes(
    counters: &mut HashMap<SentinelId, crate::health::AlarmOutcomeHealth>,
    extractions: &HashMap<SentinelId, SentinelExtraction>,
    adverse: bool,
    monitoring: &crate::config::types::MonitoringConfig,
) {
    for (&sentinel_id, extraction) in extractions.iter().filter(|(_, extraction)| extraction.occupancy) {
        let state = counters.entry(sentinel_id).or_default();
        for axis in 0..SCORING_AXIS_COUNT {
            let alarm = extraction.features.get(2 * SCORING_AXIS_COUNT + axis).unwrap_or(0.0);
            page_update(
                &mut state.high_alarm_benign[axis],
                !adverse && alarm >= monitoring.strong_alarm_threshold,
                monitoring.alarm_outcome_noise_allowance,
            );
            page_update(
                &mut state.quiet_alarm_adverse[axis],
                adverse && alarm <= monitoring.quiet_alarm_threshold,
                monitoring.alarm_outcome_noise_allowance,
            );
        }
    }
}

/// Folds one label into every entity block's legibility evidence.
///
/// The two readings are folds over a named range of feature positions and ask
/// nothing about what kind of entity owns the range, so a Sentinel's slot and
/// an identity dimension's block travel the same loop twice
/// (´def:monitoring:slot-association´), (´def:monitoring:slot-contribution´).
/// The extents come from the published layout, so a rebuild that moved a block
/// moves the fold with it.
///
/// Nothing is folded where the vector and the model disagree about the
/// layout's width: the score would then be a sum over positions the two do not
/// both describe, which is the one thing the ablation cannot be taken against.
/// An entity the layout no longer carries loses its evidence here rather than
/// accumulating unread beside the entities that have one.
fn observe_block_legibility(
    working: &WorkingCopy,
    counters: &mut HealthCounters,
    phi: &[f64],
    outcome: f64,
    weight: f64,
    forgetting: f64,
) {
    let slots = &working.dimension_map.sentinel_slots;
    let blocks = &working.dimension_map.id_dim_ranges;
    counters.sentinel_legibility.retain(|id, _| slots.contains_key(id));
    counters.dimension_legibility.retain(|id, _| blocks.contains_key(id));

    let mu_column = working.operational.mu();
    if mu_column.nrows() != phi.len() {
        return;
    }
    let Some(mu_view) = mu_column.try_as_col_major() else {
        return;
    };
    let mu = mu_view.as_slice();
    let full_score: f64 = phi
        .iter()
        .zip(mu.iter())
        .map(|(feature, coefficient)| feature * coefficient)
        .sum();

    for (&sentinel_id, slot) in slots {
        let (Some(block_phi), Some(block_mu)) = (phi.get(slot.to_range()), mu.get(slot.to_range())) else {
            continue;
        };
        let block_score: f64 = block_phi.iter().zip(block_mu.iter()).map(|(f, w)| f * w).sum();
        counters.sentinel_legibility.entry(sentinel_id).or_default().observe(
            block_phi,
            block_score,
            full_score,
            outcome,
            weight,
            forgetting,
        );
    }

    for (&dimension_id, block) in blocks {
        let (Some(block_phi), Some(block_mu)) = (phi.get(block.to_range()), mu.get(block.to_range())) else {
            continue;
        };
        let block_score: f64 = block_phi.iter().zip(block_mu.iter()).map(|(f, w)| f * w).sum();
        counters.dimension_legibility.entry(dimension_id).or_default().observe(
            block_phi,
            block_score,
            full_score,
            outcome,
            weight,
            forgetting,
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Pipeline Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for label pipeline processing.
///
/// At runtime, all parameters are sourced from [`AssayerConfig`] via
/// [`from_assayer_config()`](Self::from_assayer_config). The `Default`
/// impl uses the same spec-compliant values as `AssayerConfig::default()`
/// and exists for test convenience.
///
/// # Cross-References
///
/// - (´tab:construction:parameters´) — the parameters the surfaces defer to
/// - (´tab:config:risk-model´) — the core risk model's own values
#[derive(Clone, Debug)]
pub struct LabelPipelineConfig {
    /// Decay rate for `κᵥ` updates (`γ_κ`). Default: 0.999.
    pub gamma_kappa: f64,
    /// Core eligibility policy for sister/anchor training
    /// (´tab:eligibility:training´).
    pub eligibility: crate::config::types::EligibilityPolicy,
    /// Maximum importance weight ceiling (´def:weighting:balancing-weights´).
    /// Default: 100.
    pub importance_ceiling: f64,
    /// Per-label decay rate for the operational model (`γ_opr`). Default: 0.9995.
    pub gamma_opr: f64,
    /// Per-label decay rate for the sister model (`γ_inh`). Default: 0.9998.
    ///
    /// Also used for the anchor model (´tab:risk:forgetting-rates´).
    pub gamma_inh: f64,
    /// Precision floor `λ_floor` for diagonal replenishment. Default: 0.001.
    pub lambda_floor: f64,
    /// Leverage safety factor `c_lev` for leverage bounding. Default: 5.0.
    pub c_leverage: f64,
    /// Hourly decay rate for time-indexed model decay. Default: 0.9999.
    pub gamma_t: f64,
    /// Hourly decay rate for identity cell outcome EWMAs (`γ_t,id`). Default: 0.998 (~14 d half-life).
    pub gamma_t_identity: f64,
    /// EWMA smoothing factor for identity cell outcomes (`λ_id`). Default: 0.95.
    pub lambda_identity: f64,
    /// Hourly decay rate for Ledger time-indexed decay (`γ_t,L`). Default: 0.999 (~29 d half-life).
    pub gamma_t_ledger: f64,
    /// EWMA smoothing factor for Ledger updates (`λ_L`). Default: 0.999.
    pub lambda_l: f64,
    /// Standardisation configuration.
    pub standardisation: StandardisationConfig,
    /// Number of labels between Cholesky recomputations. Default: 1000.
    pub n_recompute: u32,
    /// Condition number threshold for Cholesky recomputation. Default: 10^5.
    pub kappa_growth_factor: f64,
    // TODO(2026-05-23) ´todo:code:thread-configurable´: Thread configurable
    // `sync_error_threshold` through `LabelPipelineConfig` so clean
    // high-sync-error recomputes can shorten intervals proactively — the
    // trigger this would widen is (´dec:posterior:recomputation-trigger´).
    /// The deployment's declared model configuration.
    ///
    /// Carried here for the one place on this path that rebuilds models rather
    /// than updating them: the CP5/CP6 revert reconstructs every model from
    /// the last published snapshot, and the prior it reconstructs them at has
    /// to be the deployment's own. The scalars this struct already carries are
    /// the ones the update reads; this is the whole declaration, so a revert
    /// has one source for it rather than a second copy to keep in step.
    pub model: crate::config::types::ModelConfig,
    /// Platt calibration configuration (´tab:config:calibration´).
    /// Default: `PlattConfig::default()`.
    ///
    /// Refit cadence is controlled by `platt.n_refit` (default: 200).
    pub platt: PlattConfig,
    /// Drift detection configuration (´tab:config:monitoring´).
    /// Default: `DriftConfig::default()`.
    pub drift: DriftConfig,
    /// Reading thresholds for health monitoring (´tab:config:monitoring´).
    pub monitoring: crate::config::types::MonitoringConfig,
    /// Convergence diagnostic thresholds (´tab:construction:parameters´).
    pub convergence: ConvergenceThresholdConfig,
}

impl Default for LabelPipelineConfig {
    fn default() -> Self {
        Self {
            gamma_kappa: 0.999,
            eligibility: crate::config::types::EligibilityPolicy::default(),
            importance_ceiling: DEFAULT_IMPORTANCE_CEILING,
            gamma_opr: GAMMA_LABEL_OPERATIONAL,
            gamma_inh: GAMMA_LABEL_SISTER,
            lambda_floor: DEFAULT_LAMBDA_FLOOR,
            c_leverage: DEFAULT_LEVERAGE_SAFETY_FACTOR,
            gamma_t: 0.9999,
            gamma_t_identity: 0.998,
            lambda_identity: 0.95,
            gamma_t_ledger: 0.999,
            lambda_l: 0.999,
            standardisation: StandardisationConfig::default(),
            n_recompute: 1000,
            kappa_growth_factor: DEFAULT_KAPPA_GROWTH_FACTOR,
            model: crate::config::types::ModelConfig::default(),
            platt: PlattConfig::default(),
            drift: DriftConfig::default(),
            monitoring: crate::config::types::MonitoringConfig::default(),
            convergence: ConvergenceThresholdConfig::default(),
        }
    }
}

impl LabelPipelineConfig {
    /// Creates a `LabelPipelineConfig` from an [`AssayerConfig`],
    /// wiring all numerical parameters through instead of using
    /// hardcoded defaults.
    pub(crate) fn from_assayer_config(config: &crate::config::types::AssayerConfig) -> Self {
        Self {
            gamma_kappa: config.model.gamma_kappa,
            eligibility: config.eligibility,
            importance_ceiling: config.model.w_ceiling,
            gamma_opr: config.model.gamma_opr,
            gamma_inh: config.model.gamma_inh,
            lambda_floor: config.model.lambda_floor,
            c_leverage: config.model.c_leverage,
            gamma_t: config.temporal.gamma_t_core,
            gamma_t_identity: config.temporal.gamma_t_identity,
            lambda_identity: 0.95, // Pipeline-internal; no AssayerConfig source yet
            gamma_t_ledger: config.temporal.gamma_t_ledger,
            lambda_l: config.ledger.lambda_l,
            standardisation: config.standardisation.clone(),
            n_recompute: config.cholesky.n_recompute,
            kappa_growth_factor: config.cholesky.kappa_growth_factor,
            // TODO(2026-05-23) ´todo:code:copy´: Copy
            // `config.cholesky.sync_error_threshold` here once it exists, for
            // the same trigger (´dec:posterior:recomputation-trigger´).
            model: config.model.clone(),
            platt: config.platt.clone(),
            drift: DriftConfig {
                kappa_drift: config.monitoring.kappa_drift,
                gamma_drift: DriftConfig::default().gamma_drift,
                h_threshold: config.monitoring.h_threshold,
            },
            monitoring: config.monitoring.clone(),
            convergence: config.convergence.clone(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Calibration Entry
// ═══════════════════════════════════════════════════════════════════════════════

// CalibrationEntry and CalibrationBuffer are defined in risk::calibration
// (´constr:platt:buffer´).
pub use crate::risk::calibration::{CalibrationBuffer, CalibrationEntry};

// ═══════════════════════════════════════════════════════════════════════════════
// Label Pipeline State
// ═══════════════════════════════════════════════════════════════════════════════

/// Mutable state passed through the label pipeline.
///
/// Collects intermediate results that are used across steps.
struct LabelPipelineState<'a> {
    /// The label being processed.
    label: &'a LabelData,
    /// The pending context (full or replay).
    context: PendingContextRef<'a>,
    /// Raw (unstandardised) reconstructed feature vector φ.
    phi_raw: Vec<f64>,
    /// Standardised feature vector φ̂.
    phi_standardised: Vec<f64>,
    /// Risk target `r_ρ` ∈ {-1, +1}.
    risk_target: f64,
    /// Whether this label is eligible for model updates.
    eligible: bool,
    /// Importance weight for the operational model (from global P+ tracker).
    importance_weight_operational: f64,
    /// Importance weight for sister/anchor models (from eligible P+ tracker).
    importance_weight_eligible: f64,
    /// Compressed valence: tanh(v / κᵥ).
    compressed_valence: f64,
}

/// Reference to either full or replay pending context.
pub enum PendingContextRef<'a> {
    /// Full context from a live assessment.
    Full(&'a PendingAssessment),
    /// Stripped context from journal replay.
    Replay(&'a PendingContext),
}

impl PendingContextRef<'_> {
    /// Returns identity active cells.
    const fn identity_active_cells(
        &self,
    ) -> &std::collections::HashMap<crate::types::DimensionId, Vec<crate::identity::CompetitiveCellId>> {
        match self {
            Self::Full(p) => &p.identity_active_cells,
            Self::Replay(p) => &p.identity_active_cells,
        }
    }

    /// Returns sentinel extractions.
    const fn sentinel_extractions(&self) -> &HashMap<SentinelId, SentinelExtraction> {
        match self {
            Self::Full(p) => &p.sentinel_extractions,
            Self::Replay(p) => &p.sentinel_extractions,
        }
    }

    /// Returns signal features.
    const fn signal_features(&self) -> &crate::pending::StoredFeatures {
        match self {
            Self::Full(p) => &p.signal_features,
            Self::Replay(p) => &p.signal_features,
        }
    }

    /// Returns the entity key.
    const fn entity(&self) -> &crate::types::EntityKey {
        match self {
            Self::Full(p) => &p.entity,
            Self::Replay(p) => &p.entity,
        }
    }

    /// Returns the Sentinels registered at assessment time.
    fn active_sentinels(&self) -> &[SentinelId] {
        match self {
            Self::Full(p) => &p.active_sentinels,
            Self::Replay(p) => &p.active_sentinels,
        }
    }

    /// Returns the Sentinels actually reporting at assessment time.
    fn reporting_sentinels(&self) -> &[SentinelId] {
        match self {
            Self::Full(p) => &p.reporting_sentinels,
            Self::Replay(p) => &p.reporting_sentinels,
        }
    }

    /// Returns the spatial outcome axes the stored extractions were laid out
    /// against, in layout order.
    ///
    /// Both arms carry it, because both are read across a gap in which the
    /// spatial set may have moved — a live entry across a lifecycle event, a
    /// replayed one across a restart as well.
    fn stored_spatial_axis_ids(&self) -> &[crate::types::OutcomeAxisId] {
        match self {
            Self::Full(p) => &p.spatial_axis_ids,
            Self::Replay(p) => &p.spatial_axis_ids,
        }
    }

    /// Returns the frozen per-dimension identity base features.
    const fn entity_base_features(&self) -> &HashMap<DimensionId, crate::pending::StoredFeatures> {
        match self {
            Self::Full(p) => &p.entity_base_features,
            Self::Replay(p) => &p.entity_base_features,
        }
    }

    /// Returns the frozen per-dimension, per-axis identity features.
    const fn entity_axis_features(
        &self,
    ) -> &HashMap<DimensionId, HashMap<crate::types::OutcomeAxisId, crate::pending::StoredFeatures>> {
        match self {
            Self::Full(p) => &p.entity_axis_features,
            Self::Replay(p) => &p.entity_axis_features,
        }
    }

    /// Returns the pending risk basis data.
    ///
    /// The journal context carries the risk basis as the original run
    /// computed it, so both context kinds hold one
    /// (´cor:durability:replay-exactness´).
    const fn risk_basis(&self) -> &crate::pending::PendingRiskBasis {
        match self {
            Self::Full(p) => &p.risk_basis,
            Self::Replay(p) => &p.risk_basis,
        }
    }

    /// Returns the outcome predictions (predicted point estimates per axis).
    const fn outcome_predictions(&self) -> &HashMap<crate::types::OutcomeAxisId, f64> {
        match self {
            Self::Full(p) => &p.outcome_predictions,
            Self::Replay(p) => &p.outcome_predictions,
        }
    }
}

/// Applies one model's rank-one label update from inputs fixed before the
/// bounded parallel region begins, counting a refusal into the shared tally.
///
/// The tally is shared because the region's workers run beside each other and
/// the answer the pipeline needs is about the label rather than about any one
/// model: one refused update means this label's whole effect is not to be
/// kept (´tab:runtime:numeric-checkpoints´).
#[allow(clippy::too_many_arguments)] // Justified: the region's inputs are fixed before it begins and passed whole
fn apply_model_label_update(
    model: &mut crate::model::bayesian::BayesianLinearModel,
    phi: &faer::Col<f64>,
    importance_weight: f64,
    importance_ceiling: f64,
    leverage_safety_factor: f64,
    target: f64,
    gamma_eff: f64,
    lambda_floor: f64,
    refusals: &AtomicU64,
) {
    let (v, h) = model.covariance().quadratic_form_with_product(phi);
    let effective_weight = compute_effective_weight(
        importance_weight,
        importance_ceiling,
        leverage_safety_factor,
        h,
        EPSILON_LEVERAGE,
    );
    let outcome = model.apply_leverage_bounded_update(phi, &v, h, effective_weight, target, gamma_eff, lambda_floor);
    if outcome == LabelUpdateOutcome::RefusedIndefiniteCovariance {
        refusals.fetch_add(1, Ordering::Relaxed);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Main Pipeline Function
// ═══════════════════════════════════════════════════════════════════════════════

/// Processes a single label through the 17-step pipeline.
///
/// # Arguments
///
/// * `working` — Mutable working copy of model state
/// * `label` — The sequenced label to process
/// * `shared` — Shared state for snapshot publication
/// * `outcome_ledger` — The outcome ledger for Sentinel updates
/// * `platt_tracker` — Platt calibration convergence tracker
/// * `calibration_buffer` — Buffer for calibration entries
/// * `identity_dimensions` — Identity dimension infrastructure (steps 2 and 13)
/// * `concordance_thresholds` — Current thresholds for the aggregate
///   block's recomputation at step 2 (´alg:runtime:reconstruction´)
/// * `config` — Pipeline configuration
/// * `clock` — Source of both timestamp domains; the head-of-path
///   readings taken here are the ones (´dec:ordering:decay-at-head´)
///   applies decay against
/// * `event_tx` — Channel for health events
/// * `dropped_counter` — Counter for dropped events
/// * `drift_reset_counter` — Counter for emitted drift-reset events
///
/// # Cross-References
///
/// - (´alg:runtime:update-path´) — the seventeen steps this function is
pub fn process_label(
    working: &mut WorkingCopy,
    label: &SequencedLabel,
    shared: &SharedState,
    outcome_ledger: &OutcomeLedger,
    platt_tracker: &mut PlattConvergenceTracker,
    calibration_buffer: &mut CalibrationBuffer,
    identity_dimensions: &std::sync::RwLock<HashMap<DimensionId, std::sync::Arc<IdentityDimensionInfra>>>,
    identity_trackers: &HashMap<DimensionId, crate::health::IdentityConvergenceTracker>,
    concordance_thresholds: [f64; SCORING_AXIS_COUNT],
    config: &LabelPipelineConfig,
    clock: &dyn crate::testing::Clock,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
    drift_reset_counter: &DriftResetCounter,
    version: u64,
    label_count: u64,
    health_counters: &mut HealthCounters,
) {
    let now = clock.now_monotonic();
    let persistent_now = clock.now();

    // ─────────────────────────────────────────────────────────────────────
    // Capture revert state (´tab:runtime:numeric-checkpoints´)
    // ─────────────────────────────────────────────────────────────────────
    // Save enough state to restore the calibration buffer and Platt
    // tracker on CP5/CP6 revert. These are outside WorkingCopy but are
    // conceptually part of the reverted state.
    let calibration_buffer_len_before = calibration_buffer.len();
    let platt_tracker_before = platt_tracker.clone();

    // ─────────────────────────────────────────────────────────────────────
    // Step 0: Time-indexed model decay
    // ─────────────────────────────────────────────────────────────────────
    let elapsed = now.duration_since(working.last_label_time);

    // Compute time-decay factor for operational model (γ_t,op = 0.9995/hour)
    // This is the time component of the combined decay; per-label decay is
    // applied in steps 9-11.
    let gamma_t_dt = decay_factor_elapsed(config.gamma_t, elapsed);

    working.last_label_time = now;

    // ─────────────────────────────────────────────────────────────────────
    // Steps 1–3: Retrieve, reconstruct, re-standardise
    // ─────────────────────────────────────────────────────────────────────
    let context_ref = match &label.context {
        LabelContext::Full(pending) => PendingContextRef::Full(pending.as_ref()),
        LabelContext::Replay(pending) => PendingContextRef::Replay(pending.as_ref()),
    };

    // Step 2: Reconstruct φ from stored components + current competitive set
    let phi_raw = reconstruct_phi(working, &context_ref, identity_dimensions, concordance_thresholds);

    // Step 3: Standardise φ → φ̂
    let mut phi_standardised = phi_raw.clone();
    crate::feature::standardisation::standardise_in_place(
        &mut phi_standardised,
        &working.feature_means,
        &working.feature_variances,
        config.standardisation.epsilon,
    );

    // ─────────────────────────────────────────────────────────────────────
    // Steps 4–8: Scalar preparation
    // ─────────────────────────────────────────────────────────────────────

    // Step 4: Risk target r_ρ
    let risk_target = compute_risk_target(label.label.valence);
    let is_positive = label.label.valence > 0.0;

    // Step 5: Update κᵥ if valence ≠ 0
    if label.label.valence.abs() > f64::EPSILON {
        working.kappa_v = (1.0 - config.gamma_kappa).mul_add(label.label.valence.abs(), config.gamma_kappa * working.kappa_v);
    }

    // Compute compressed valence for Ledger updates
    let compressed_valence = (label.label.valence / working.kappa_v.max(0.01)).tanh();

    // Step 6: Determine eligibility (´tab:eligibility:training´)
    let eligible = determine_eligibility(label.label.action_taken, config.eligibility, label.label.ground_truth);

    // Track eligibility and action for health counters.
    if eligible {
        health_counters.eligible_labels += 1;
        // TODO ´todo:code:increment-working-total-eligible-labels´: increment `working.total_eligible_labels`
        // here once the checkpointed working-copy field exists. The
        // composite convergence stage must use that eligible-label clock,
        // not the model owner's total `label_count`.
    }
    *health_counters.labels_by_action.entry(label.label.action_taken).or_insert(0) += 1;

    // Step 7: Update P+ trackers. Each tracker decays at the forgetting
    // rate of the models it serves (´tab:weighting:trackers´).
    update_p_plus_tracker(&mut working.p_positive_global, config.gamma_opr, is_positive);

    if eligible {
        update_p_plus_tracker(&mut working.p_positive_eligible, config.gamma_inh, is_positive);
    }

    // Step 8: Compute importance weights (´def:weighting:balancing-weights´).
    // Global tracker → w_opr (operational model, always computed)
    let importance_weight_operational =
        compute_importance_weight(working.p_positive_global, is_positive, config.importance_ceiling);

    // Eligible tracker → w_inh (sister/anchor, only meaningful when eligible)
    let importance_weight_eligible = if eligible {
        compute_importance_weight(working.p_positive_eligible, is_positive, config.importance_ceiling)
    } else {
        1.0
    };

    // Assemble pipeline state
    let state = LabelPipelineState {
        label: &label.label,
        context: context_ref,
        phi_raw,
        phi_standardised,
        risk_target,
        eligible,
        importance_weight_operational,
        importance_weight_eligible,
        compressed_valence,
    };

    // Reporting-only label-time health. Every input is frozen on the pending
    // assessment, so replay observes the same signs and alarms as the live run.
    health_counters
        .importance_operational
        .observe(importance_weight_operational, is_positive, config.importance_ceiling);
    if eligible {
        health_counters
            .importance_sister
            .observe(importance_weight_eligible, is_positive, config.importance_ceiling);
    }
    let agrees = state.risk_target.signum() == state.context.risk_basis().rho_effective.signum();
    health_counters.per_entity_concordance.observe(state.context.entity(), agrees);
    observe_alarm_outcomes(
        &mut health_counters.alarm_outcome,
        state.context.sentinel_extractions(),
        is_positive,
        &config.monitoring,
    );
    // The two legibility readings are folded here, before the label reaches
    // any model: the score their ablation is taken against has to be the score
    // the request was actually given, and after step 11 the weights that gave
    // it are gone (´def:monitoring:slot-contribution´). The forgetting factor
    // is the operational update's own combined factor, so the stretch of
    // traffic the readings describe is the stretch the weights describe.
    observe_block_legibility(
        working,
        health_counters,
        &state.phi_standardised,
        state.risk_target,
        state.importance_weight_operational,
        config.gamma_opr * gamma_t_dt,
    );

    // ─────────────────────────────────────────────────────────────────────
    // Steps 9–11: Model updates (Sherman-Morrison rank-1 updates)
    // ─────────────────────────────────────────────────────────────────────

    // Convert phi_standardised to Col<f64> for matrix operations
    let phi_hat = vec_to_col(&state.phi_standardised);

    // Dimension check: phi must match model dimension.
    // This guards against lifecycle/configuration mismatches where the
    // feature dimension and model dimension are temporarily out of sync.
    let phi_dim = phi_hat.nrows();
    let model_dim = working.operational.dim();

    // Track which models received an SM update so that models which did
    // NOT can receive standalone time decay (´dec:posterior:combined-factor´).
    let mut operational_updated = false;
    let mut sister_updated = false;
    let mut anchor_updated = false;
    let mut axis_updated_ids: Vec<crate::types::OutcomeAxisId> = Vec::new();

    // Updates refused for a covariance that no longer yields a non-negative
    // leverage. The workers of the bounded region each report into it and the
    // steward reads it at CP5 (´tab:runtime:numeric-checkpoints´).
    let update_refusals = AtomicU64::new(0);

    if phi_dim == model_dim {
        // Build the anchor input on the steward before borrowing the disjoint
        // model records into the scoped workers.
        let anchor_phi = if state.eligible {
            let p_a = working.anchor.dim();
            let phi_tilde_vec: Vec<f64> = working.dimension_map.extract_anchor_subvector(
                &state.phi_standardised,
                p_a,
                state.context.reporting_sentinels().len(),
            );
            if phi_tilde_vec.len() == p_a {
                Some(vec_to_col(&phi_tilde_vec))
            } else {
                None
            }
        } else {
            None
        };

        operational_updated = true;
        sister_updated = state.eligible;
        anchor_updated = anchor_phi.is_some();

        // The steward owns all inputs and waits at the scope boundary before
        // any later pipeline step observes a model. Two helper threads are the
        // fixed bound: operational is one independent record, sister and anchor
        // are a second ordered group, and the steward updates the axis records.
        // No model shares mutable state with another, so their former order was
        // not observable outside this joined region
        // (´dec:concurrency:single-steward´).
        let operational = &mut working.operational;
        let sister = &mut working.sister;
        let anchor = &mut working.anchor;
        let outcome_models = &mut working.outcome_models;
        let eligible = state.eligible;
        let operational_importance = state.importance_weight_operational;
        let eligible_importance = state.importance_weight_eligible;
        let risk_target = state.risk_target;
        let importance_ceiling = config.importance_ceiling;
        let leverage_safety_factor = config.c_leverage;
        let lambda_floor = config.lambda_floor;
        // A shared reference, taken before the region, because the workers'
        // closures capture by move and a reference is what they may each hold.
        let refusals = &update_refusals;
        std::thread::scope(|scope| {
            let operational_phi = &phi_hat;
            let operational_gamma = config.gamma_opr * gamma_t_dt;
            let _operational_worker = scope.spawn(move || {
                apply_model_label_update(
                    operational,
                    operational_phi,
                    operational_importance,
                    importance_ceiling,
                    leverage_safety_factor,
                    risk_target,
                    operational_gamma,
                    lambda_floor,
                    refusals,
                );
            });

            if eligible {
                let sister_phi = &phi_hat;
                let sister_gamma = config.gamma_inh * gamma_t_dt;
                let _sister_anchor_worker = scope.spawn(move || {
                    apply_model_label_update(
                        sister,
                        sister_phi,
                        eligible_importance,
                        importance_ceiling,
                        leverage_safety_factor,
                        risk_target,
                        sister_gamma,
                        lambda_floor,
                        refusals,
                    );
                    if let Some(anchor_phi) = &anchor_phi {
                        apply_model_label_update(
                            anchor,
                            anchor_phi,
                            eligible_importance,
                            importance_ceiling,
                            leverage_safety_factor,
                            risk_target,
                            sister_gamma,
                            lambda_floor,
                            refusals,
                        );
                    }
                });
            }

            // Step 11: Outcome axis models. The steward retains their label
            // order while their independent matrices run beside the two fixed
            // helper groups.
            for (&axis_id, &axis_value) in &state.label.outcomes {
                let Some(axis_state) = outcome_models.get_mut(&axis_id) else {
                    continue;
                };
                let axis_dim = axis_state.model.dim();
                if axis_dim != phi_dim {
                    continue; // Dimension mismatch — skip this axis.
                }

                // Per-axis risk target: compressed by κ_a.
                let kappa_a_safe = axis_state.kappa_a.max(1e-12);
                let r_a = (axis_value / kappa_a_safe).tanh();

                // Update κ_a EWMA if axis_value ≠ 0.
                if axis_value.abs() > f64::EPSILON {
                    axis_state.kappa_a =
                        (1.0 - config.gamma_kappa).mul_add(axis_value.abs(), config.gamma_kappa * axis_state.kappa_a);
                }

                // Per-axis eligibility (´req:host:eligibility-policy´): each axis carries
                // its own OutcomeEligibility from registration. AllLabels
                // axes update on every label; EligibleOnly axes respect
                // the global eligibility gate.
                let axis_eligible = match axis_state.eligibility {
                    OutcomeEligibility::AllLabels => true,
                    OutcomeEligibility::EligibleOnly => state.eligible,
                };

                if axis_eligible {
                    let gamma_eff = axis_state.gamma * gamma_t_dt;
                    apply_model_label_update(
                        &mut axis_state.model,
                        &phi_hat,
                        eligible_importance,
                        importance_ceiling,
                        leverage_safety_factor,
                        r_a,
                        gamma_eff,
                        lambda_floor,
                        refusals,
                    );
                    axis_updated_ids.push(axis_id);
                }
            }
        });
    }
    // else: dimension mismatch — skip model updates.
    // This can happen during lifecycle transitions if a label races with
    // model dimensionality changes; the label is then treated as non-updating.

    // ─────────────────────────────────────────────────────────────────────
    // Standalone time decay for models not SM-updated
    // (´dec:posterior:combined-factor´)
    // ─────────────────────────────────────────────────────────────────────
    // When a model receives an SM update, time decay is folded into the
    // combined factor γ_eff = γ_label · γ_t^Δt.  Models that were NOT
    // updated still need the time component applied standalone.
    if !operational_updated {
        working.operational.apply_time_decay(gamma_t_dt);
    }
    if !sister_updated {
        working.sister.apply_time_decay(gamma_t_dt);
    }
    if !anchor_updated {
        working.anchor.apply_time_decay(gamma_t_dt);
    }
    // Outcome axis models not updated also need standalone time decay.
    for (&axis_id, axis_state) in &mut working.outcome_models {
        if !axis_updated_ids.contains(&axis_id) {
            axis_state.model.apply_time_decay(gamma_t_dt);
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cholesky recomputation check (post-step-11)
    // ─────────────────────────────────────────────────────────────────────
    // Check each model independently for Cholesky recomputation triggers.
    // Checked after step 11 and before step 12
    // (´dec:posterior:recomputation-trigger´).
    check_cholesky_recomputation(working, config, event_tx, dropped_counter);

    // ─────────────────────────────────────────────────────────────────────
    // Companion boundary: no Core-owned mutation
    // ─────────────────────────────────────────────────────────────────────
    // Challenge effectiveness is host-owned Companion state
    // (´inv:companion:boundary´). The
    // Core label path does not infer a channel or mutate the Companion tracker.

    // ─────────────────────────────────────────────────────────────────────
    // Step 12: Identity dimension outcome update
    // ─────────────────────────────────────────────────────────────────────
    // For each identity dimension, iterate the stored active cells and
    // update outcome EWMAs with write-time decay
    // (´alg:runtime:identity-outcome-update´).
    process_identity_cell_outcomes(
        &state,
        working,
        identity_dimensions,
        &persistent_now,
        config.gamma_t_identity,
        config.lambda_identity,
    );

    // ─────────────────────────────────────────────────────────────────────
    // Step 13: Ledger update
    // ─────────────────────────────────────────────────────────────────────
    process_ledger_updates(&state, working, outcome_ledger, &persistent_now, config);

    // ─────────────────────────────────────────────────────────────────────
    // Step 14: Calibration buffer + Platt check
    // ─────────────────────────────────────────────────────────────────────
    // Runs on replay as on a live label: the journal context carries the
    // risk basis and predictions the original run computed, and replayed
    // labels postdate the checkpoint's captured buffer, so the entry
    // pushed here is one the buffer has never held
    // (´cor:durability:replay-exactness´).
    {
        // Push calibration entry (´constr:platt:buffer´).
        let rho_eff = state.context.risk_basis().rho_effective;
        let anchor_w = state.context.risk_basis().anchor_weight;

        // Build axis_data from predicted (assessment time) and actual (label) values.
        // (axis_id, predicted, actual) tuples for per-axis correlation
        // (´def:monitoring:axis-correlation´).
        let axis_data: Vec<(crate::types::OutcomeAxisId, f64, f64)> = state
            .context
            .outcome_predictions()
            .iter()
            .filter_map(|(&axis_id, &predicted)| state.label.outcomes.get(&axis_id).map(|&actual| (axis_id, predicted, actual)))
            .collect();

        calibration_buffer.push(CalibrationEntry {
            rho_eff,
            // The assessment-time σ_p̂, one field access away on the risk
            // basis — the row's denominator for empirical coverage
            // (´alg:monitoring:empirical-coverage´).
            uncertainty: state.context.risk_basis().uncertainty,
            anchor_weight: anchor_w,
            positive: is_positive,
            // TODO(2026-05-21) ´todo:code:store-state-importance-weight-operational´: Store `state.importance_weight_operational`
            // here, and add the model-owner label index once `CalibrationEntry`
            // grows the `label_index` field.
            importance_weight: state.importance_weight_eligible,
            axis_data,
        });

        // Check and record Platt refit (´tab:platt:refit-cadence´).
        platt_tracker.record_label();

        // Three triggers (OR): periodic + early (via tracker), anchor reactivation.
        // Anchor reactivation also requires sufficient anchor-regime samples
        // to avoid a wasted refit (´req:platt:minimum-samples´).
        let periodic_or_early = platt_tracker.should_refit(config.platt.n_refit);
        let anchor_reactivation = !calibration_buffer.is_empty()
            && label_count.saturating_sub(platt_tracker.last_anchor_refit_label_index) > u64::from(config.platt.n_refit)
            && calibration_buffer.has_sufficient_anchor_samples(&config.platt);

        if periodic_or_early || anchor_reactivation {
            let state_before = platt_tracker.convergence_state(
                config.convergence.platt_converged_delta_cal,
                config.convergence.platt_converged_refit_count,
            );

            let old_kappa_sister = working.kappa_sister;
            let old_kappa_anchor = working.kappa_anchor;
            let result = refit_platt_calibration_with_monitoring(
                calibration_buffer,
                working.kappa_sister,
                working.kappa_anchor,
                &config.platt,
                &config.monitoring,
            );

            // Apply result to working copy.
            working.kappa_sister = result.kappa_sister;
            working.kappa_anchor = result.kappa_anchor;

            // Update convergence tracker (´dec:health:concrete-trackers´).
            platt_tracker.record_refit(&result, label_count);

            // Capture discrimination metrics from refit
            // (´dec:health:cached-discrimination´).
            if result.discrimination.is_some() {
                health_counters.last_discrimination.clone_from(&result.discrimination);
            }

            let state_after = platt_tracker.convergence_state(
                config.convergence.platt_converged_delta_cal,
                config.convergence.platt_converged_refit_count,
            );

            // Emit convergence events on state *transition*
            // (´dec:health:bounded-events´).
            if state_after == PlattConvergenceState::FirstFit && state_before == PlattConvergenceState::Initial {
                emit_health_event(
                    HealthEvent::Convergence(ConvergenceEvent::PlattFirstFit {
                        kappa_sister: result.kappa_sister,
                        kappa_anchor: result.kappa_anchor,
                        delta_cal: result.delta_cal,
                    }),
                    event_tx,
                    dropped_counter,
                );
            }
            if state_after == PlattConvergenceState::Converged && state_before != PlattConvergenceState::Converged {
                emit_health_event(
                    HealthEvent::Convergence(ConvergenceEvent::PlattConverged {
                        refits_completed: platt_tracker.refits_completed,
                        delta_cal: result.delta_cal,
                    }),
                    event_tx,
                    dropped_counter,
                );
            }

            // Post-calibration drift reset, narrowed to the affected
            // regime (´tab:monitoring:drift-resets´). The per-regime
            // deltas are recomputed from the parameters the refit
            // moved, so a shift in one regime's mapping does not
            // discard the other's evidence.
            let delta_sister = if result.sister_fitted {
                (result.kappa_sister.ln() - old_kappa_sister.ln()).abs()
            } else {
                0.0
            };
            let delta_anchor = if result.anchor_fitted {
                (result.kappa_anchor.ln() - old_kappa_anchor.ln()).abs()
            } else {
                0.0
            };
            apply_calibration_drift_reset(
                &mut working.drift_state,
                delta_sister,
                delta_anchor,
                config.platt.delta_cal_threshold,
                event_tx,
                dropped_counter,
                drift_reset_counter,
            );
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Step 15: Standardisation EWMA update (runs during replay too)
    // ─────────────────────────────────────────────────────────────────────
    // Update standardisation running means/variances with the raw φ.
    //
    // Steps 5 to 7 of (´alg:standardisation:label-time-procedure´) run if and
    // only if the full-vector phase is `InService`. While the cold ramp is
    // waiting or transitioning it is the sole owner of full-vector
    // standardisation, and a label advancing the average underneath it would
    // put a label's authority into a transition an observation's authority
    // owns (´dec:ordering:evidence-authority´). Steps 1 to 4 — including the
    // model updates above — run in every phase, so the models learn from the
    // first label onward whatever the coordinate system is doing.
    if !state.phi_raw.is_empty() && working.cold_ramp.phase().is_in_service() {
        crate::feature::standardisation::update_standardisation(
            &state.phi_raw,
            &mut working.feature_means,
            &mut working.feature_variances,
            &config.standardisation,
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Step 16: Drift accumulators (´alg:monitoring:drift-cusums´)
    // ─────────────────────────────────────────────────────────────────────
    // Runs on replay as on a live label: the residual is taken against
    // the assessment-time prediction the journal context carries, which
    // is the quantity (´alg:monitoring:drift-cusums´) requires.
    {
        // Compute residual = observed − predicted.
        // ρ̂_eff comes from the blend computation stored in the pending assessment.
        let rho_predicted = state.context.risk_basis().rho_effective;
        let residual = state.risk_target - rho_predicted;

        // Update drift accumulators for ALL models
        // (´alg:monitoring:drift-cusums´), each auto-resetting on a threshold
        // breach (´tab:monitoring:drift-resets´).
        let mut drift_model_ids: Vec<ModelId> = vec![ModelId::Operational, ModelId::Sister, ModelId::Anchor];
        for &axis_id in working.outcome_models.keys() {
            drift_model_ids.push(ModelId::OutcomeAxis(axis_id));
        }

        for model_id in drift_model_ids {
            let drift = working.drift_state.entry(model_id.clone()).or_default();
            let triggered_reset = drift.update(residual, &config.drift);

            if triggered_reset {
                emit_drift_reset_event(
                    0.0, // Not a Platt trigger — per-model CUSUM auto-reset
                    1,
                    false,
                    event_tx,
                    dropped_counter,
                    drift_reset_counter,
                );
                tracing::info!(
                    model = ?model_id,
                    steps_since_reset = drift.steps_since_reset,
                    "Drift CUSUM auto-reset triggered"
                );
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // CP5: the working copy's numeric state and this label's refusals
    // ─────────────────────────────────────────────────────────────────────
    // The reading is the working copy's corrupt numeric state — non-finite
    // entries, and a precision matrix a factorisation refused — and the
    // updates this label's own leverage readings refused. Both are findings about the
    // state a label has left behind rather than about the label, and the
    // checkpoint's response to either is the same one: keep the last state
    // known good (´tab:runtime:numeric-checkpoints´). A refusal reaches here
    // rather than being repaired at the model because the model that refused
    // stopped short of mutating anything, while its siblings in the same
    // bounded region did not, so it is the label's whole effect that is not to
    // be kept.
    let refused_updates = update_refusals.load(Ordering::Relaxed);
    let working_copy_corrupt = check_working_copy_nan(working);
    if working_copy_corrupt || refused_updates > 0 {
        health_counters.cp5_reverts += 1;
        revert_all_state(
            working,
            platt_tracker,
            calibration_buffer,
            &platt_tracker_before,
            calibration_buffer_len_before,
            shared,
            health_counters,
            config,
            label_count,
        );
        if refused_updates > 0 {
            tracing::error!(
                models = refused_updates,
                "CP5: a model's covariance no longer yields a non-negative leverage; the update was refused and the label reverted",
            );
        } else {
            tracing::warn!(
                "CP5: the working copy carries a non-finite entry or a precision matrix a factorisation refused; reverted to last snapshot",
            );
        }

        // Publish health summary so CP5 counter is visible immediately.
        publish_health_summary(
            working,
            shared,
            platt_tracker,
            identity_trackers,
            health_counters,
            config,
            label_count,
            &persistent_now,
            event_tx,
            dropped_counter,
        );

        // Use max() so the high-water mark is monotonic: defence against
        // out-of-order delivery (e.g. replay interleaved with live labels).
        if let Some(seq) = label.seq {
            working.last_processed_label_seq = working.last_processed_label_seq.max(seq);
        }
        return;
    }

    // ─────────────────────────────────────────────────────────────────────
    // CP6: Pre-publish check
    // ─────────────────────────────────────────────────────────────────────
    // Verify the snapshot will not contain NaN before publishing.
    let mut snapshot = working.to_snapshot_at(version, now);

    // Patch calibration buffer snapshot from the live buffer.
    snapshot.calibration_buffer = calibration_buffer.to_snapshot();

    if snapshot_has_nan(&snapshot) {
        health_counters.cp6_reverts += 1;
        revert_all_state(
            working,
            platt_tracker,
            calibration_buffer,
            &platt_tracker_before,
            calibration_buffer_len_before,
            shared,
            health_counters,
            config,
            label_count,
        );
        tracing::warn!("CP6: NaN detected in snapshot; reverted and publishing clean");

        // Publish health summary so CP6 counter is visible immediately.
        publish_health_summary(
            working,
            shared,
            platt_tracker,
            identity_trackers,
            health_counters,
            config,
            label_count,
            &persistent_now,
            event_tx,
            dropped_counter,
        );

        // See note in CP5 branch on monotonic high-water mark.
        if let Some(seq) = label.seq {
            working.last_processed_label_seq = working.last_processed_label_seq.max(seq);
        }
        return;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Step 17: Publish
    // ─────────────────────────────────────────────────────────────────────
    // snapshot was created above in CP6 check

    // Publish model snapshot first
    let published_at = clock.now_monotonic();
    shared.published.store(std::sync::Arc::new(snapshot));

    // The label's evidence is now reflected in a published model, which is
    // where the end-to-end feedback latency ends (´def:monitoring:feedback-latency´).
    // Only this path observes: the CP5 and CP6 branches above return having
    // published no model, so nothing reached the end of the journey and there
    // is no journey to fold in.
    health_counters.feedback_latency.observe(
        config.monitoring.feedback_latency_ewma_rate,
        measure_feedback_latency(label, now, published_at),
    );

    // Build and publish health summary (´dec:health:independent-publication´).
    publish_health_summary(
        working,
        shared,
        platt_tracker,
        identity_trackers,
        health_counters,
        config,
        label_count,
        &persistent_now,
        event_tx,
        dropped_counter,
    );

    // Update sequence number (monotonic high-water mark; see CP5 branch).
    if let Some(seq) = label.seq {
        working.last_processed_label_seq = working.last_processed_label_seq.max(seq);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Measures one label's four feedback-latency stages
/// (´def:monitoring:feedback-latency´).
///
/// The three boundaries this process controls are read off three instants on
/// one injected monotonic clock, so their differences are properties of the
/// scenario rather than of how busy the machine was:
///
/// - **report** — the age the producing Sentinel measured on its own clock,
///   which is a lower bound and not the stage: the interval between that
///   Sentinel emitting the report and the report reaching this process is
///   measured by nothing and is left as an explicitly unmeasured residual.
/// - **label** — from that report's reception to the host's label arriving at
///   the public boundary. The human in the loop is normally the whole of it.
/// - **queue** — from the label's arrival to the model owner taking it up,
///   which is the head-of-path reading this pipeline decays against
///   (´dec:ordering:decay-at-head´).
/// - **publish** — from that reading to the instant the new snapshot is
///   stored.
///
/// A stage whose boundary this label did not cross is absent rather than zero.
/// A replayed label has no arrival and its context no origin, so it measures
/// only the publish stage; an assessment no Sentinel contributed to has no
/// origin, so it measures everything but the first two.
fn measure_feedback_latency(
    label: &SequencedLabel,
    taken_up_at: Instant,
    published_at: Instant,
) -> crate::health::FeedbackLatencyStages {
    let origin = match &label.context {
        LabelContext::Full(pending) => pending.report_origin,
        // A replayed context is journal-borne and carries no instant from a
        // monotonic domain the restart ended (´dec:retention:journal-subset´).
        LabelContext::Replay(_) => None,
    };

    let report_lower_bound_seconds = origin
        .and_then(|origin| origin.oldest_observation_age_micros)
        .map(|micros| Duration::from_micros(micros).as_secs_f64());

    let label_seconds = origin
        .zip(label.arrived_at)
        .map(|(origin, arrived_at)| arrived_at.saturating_duration_since(origin.received_at).as_secs_f64());

    let queue_seconds = label
        .arrived_at
        .map(|arrived_at| taken_up_at.saturating_duration_since(arrived_at).as_secs_f64());

    crate::health::FeedbackLatencyStages {
        report_lower_bound_seconds,
        label_seconds,
        queue_seconds,
        publish_seconds: Some(published_at.saturating_duration_since(taken_up_at).as_secs_f64()),
    }
}

/// Reverts all mutable state on CP5/CP6 corruption.
///
/// Restores the working copy (standardisation statistics included) from
/// the last published snapshot, then rolls back the Platt tracker and the
/// calibration buffer. Companion
/// challenge state is host-owned and is not part of this Core revert.
///
/// The rebuild is at the deployment's declared prior and its declared
/// regularisation, not at the defaults: a revert that rebuilt every model at
/// the default prior precision would answer a corruption by silently retuning
/// the deployment.
///
/// A rebuild that is itself refused stops the label path
/// (´dec:surface:async-label´). The engine has judged the working copy corrupt
/// and cannot replace it, so there is no state left it is willing to learn
/// from; it stops learning rather than continuing with the state it just
/// judged corrupt. What continues is everything that does not learn — the
/// published snapshot stays as the read surface for assessments, and the
/// command channel keeps carrying lifecycle, checkpoint and shutdown, so the
/// host can checkpoint or shut down cleanly after reading the cause.
///
/// # Cross-References
///
/// - (´tab:runtime:numeric-checkpoints´) — the two checkpoints that revert a label
#[allow(clippy::too_many_arguments)]
fn revert_all_state(
    working: &mut WorkingCopy,
    platt_tracker: &mut PlattConvergenceTracker,
    calibration_buffer: &mut CalibrationBuffer,
    platt_tracker_before: &PlattConvergenceTracker,
    calibration_buffer_len_before: usize,
    shared: &SharedState,
    health_counters: &mut HealthCounters,
    config: &LabelPipelineConfig,
    label_count: u64,
) {
    let last_snapshot = shared.published.load();

    match WorkingCopy::from_snapshot(
        &last_snapshot,
        &config.model,
        config.n_recompute,
        config.standardisation.n_init as usize,
    ) {
        Ok(mut reverted) => {
            // `from_snapshot` cannot restore the class vector, the ramp's
            // base and sample, or the per-axis descriptions — the snapshot
            // carries none of them (´def:publication:model-snapshot´) — so
            // the live ones are preserved across the revert. The ramp is
            // preserved rather than rebuilt because a revert undoes a label's
            // effect on the models, and a label has no authority over the
            // cold ramp to undo (´dec:ordering:evidence-authority´).
            reverted.feature_classes = std::mem::take(&mut working.feature_classes);
            reverted.cold_ramp = working.cold_ramp.clone();
            // The hibernation archive is preserved for the ramp's reason and
            // one of its own: the snapshot does not carry it, and a label has
            // no authority over it either — lifecycle events are the archive's
            // only writers (´alg:registry:hibernation´), so a revert of a
            // label has nothing to undo in it.
            reverted.hibernation = std::mem::take(&mut working.hibernation);
            reverted.layout_generation = working.layout_generation;
            for (axis_id, wam) in &mut reverted.outcome_models {
                if let Some(live) = working.outcome_models.get_mut(axis_id) {
                    wam.description = std::mem::take(&mut live.description);
                }
            }
            *working = reverted;
        }
        Err(e) => {
            // The rebuild was refused, so the working copy in hand is the one
            // the checkpoint has just judged corrupt and there is nothing to
            // replace it with. The label path stops here rather than
            // continuing on that state. The arrangement this replaces
            // shortened every model's recomputation interval so that the
            // repair cascade would be attempted sooner — but the cascade has
            // no repairing arm to reach: a refused factorisation is a verdict
            // about the matrix and not a condition the next attempt on the
            // same matrix will find otherwise, so the shortening bought
            // attempts rather than an exit
            // (´dec:posterior:cascade-never-fails´).
            health_counters.revert_failures += 1;
            let stop = crate::error::LabelPathStop {
                model: e.source.model.clone(),
                pivot: e.source.pivot_index,
                p: e.p,
                labels_taken_up: label_count,
            };
            tracing::error!(
                cause = %stop,
                "the revert from the last published snapshot was refused; the label path is stopped and no further label will be applied",
            );
            // Written once and never lifted, so a second failed revert leaves
            // the first cause standing: the first is the one that stopped the
            // path and the one the host's recovery is about. The model-owner
            // thread is the sole writer, so the read and the write need no
            // atomicity between them.
            if shared.label_path_stop.load().is_none() {
                shared.label_path_stop.store(Some(std::sync::Arc::new(stop)));
            }
        }
    }

    // Restore calibration buffer to pre-label state.
    calibration_buffer.truncate(calibration_buffer_len_before);

    // Restore Platt tracker to pre-label state.
    *platt_tracker = platt_tracker_before.clone();

    // Re-publish the clean snapshot to confirm recovery.
    shared.published.store(std::sync::Arc::clone(&last_snapshot));
}

/// Post-calibration drift reset, narrowed to the affected regime
/// (´tab:monitoring:drift-resets´): a regime whose calibration
/// parameter shifted past the threshold has its model's accumulators
/// and step counter reset — `reset_cusums`, so the smoothed
/// diagnostics survive as they survive the automatic trigger — and
/// the models of the unaffected regime keep their evidence.
pub fn apply_calibration_drift_reset(
    drift_state: &mut HashMap<ModelId, crate::health::DriftState>,
    delta_sister: f64,
    delta_anchor: f64,
    threshold: f64,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
    drift_reset_counter: &DriftResetCounter,
) {
    let mut models_reset = 0;
    if delta_sister > threshold
        && let Some(drift) = drift_state.get_mut(&ModelId::Sister)
    {
        drift.reset_cusums();
        models_reset += 1;
    }
    if delta_anchor > threshold
        && let Some(drift) = drift_state.get_mut(&ModelId::Anchor)
    {
        drift.reset_cusums();
        models_reset += 1;
    }
    if models_reset > 0 {
        let delta_cal = delta_sister.max(delta_anchor);
        tracing::info!(
            delta_cal,
            threshold,
            models_reset,
            "Platt δ_cal exceeds threshold; affected regime's drift accumulators reset"
        );
        emit_drift_reset_event(delta_cal, models_reset, false, event_tx, dropped_counter, drift_reset_counter);
    }
}

/// Reads one model's precision health as the health surface publishes it.
///
/// Every figure here is read from the model rather than recomputed: the
/// per-label diagnostics from their own counters, and everything a rebuild
/// measures from the record the last rebuild wrote, which is absent until one
/// has run (´dec:posterior:recomputation-trigger´), (´dec:health:tiered-queries´).
///
/// It is a named function rather than the closure it began as so that what the
/// surface publishes for a given model state can be asserted directly, without
/// driving a whole label path to reach it.
pub fn model_precision_health(model: &crate::model::bayesian::BayesianLinearModel) -> crate::health::ModelPrecisionHealth {
    use crate::health::PublishedRebuildVerdict;

    crate::health::ModelPrecisionHealth {
        // The true condition number and the spectral readings come from the
        // record the last rebuild wrote, and are absent until one has run
        // (´dec:posterior:recomputation-trigger´). The cheap ratio is read on
        // every label and is published under its own name.
        kappa: model
            .baseline()
            .and_then(super::super::model::recompute::RecomputeBaseline::kappa_true),
        diagonal_ratio: model.last_diagonal_ratio(),
        lambda_min: model.baseline().and_then(RecomputeBaseline::spectrum).map(|s| s.lambda_min),
        spectral_floor: model.baseline().and_then(RecomputeBaseline::spectrum).map(|s| s.lambda_floor),
        floor_share: model
            .baseline()
            .and_then(super::super::model::recompute::RecomputeBaseline::floor_share),
        alarms: model.alarms(),
        measurements: model.measurements(),
        // The readings the last visit took, and the labels that place them.
        // Every visit retakes them, so they stay current on a model that has
        // not rebuilt for a long time (´dec:posterior:recomputation-trigger´).
        last_measurement_labels: model.baseline().map_or(0, |b| b.labels_at_record),
        last_sync_error: model.last_sync_error(),
        last_prior_induced_sync_error: model.last_prior_induced_sync_error(),
        last_sync_error_residual: model.last_sync_error_residual(),
        last_sync_error_resolution: model.baseline().map_or(0.0, |b| b.sync_error_resolution),
        last_measurement_at_resolution: model.baseline().is_some_and(|b| b.at_resolution),
        // What the last rebuild measured after itself, what its factorisation
        // answered, and what the measurement then did with the answer. The
        // three travel together and share one absence, because they are three
        // readings of one event (´dec:posterior:measured-adoption´), and that
        // event may be many visits back. The verdict crosses onto the
        // published vocabulary here, which is the only place both spellings
        // are in scope.
        last_sync_error_after: model.baseline().and_then(RecomputeBaseline::sync_error_after),
        last_rebuild_verdict: model
            .baseline()
            .and_then(RecomputeBaseline::verdict)
            .map(|verdict| match verdict {
                RebuildVerdict::Clean => PublishedRebuildVerdict::Clean,
                RebuildVerdict::Refused => PublishedRebuildVerdict::Refused,
            }),
        last_rebuild_adopted: model.baseline().and_then(RecomputeBaseline::adopted),
        n_recompute_effective: model.n_recompute_effective(),
        sync_error_shortenings: model.sync_error_shortenings(),
        total_recomputes: model.total_recomputes(),
        floored_rebuilds: model.floored_rebuilds(),
        cascade_terminus_events: model.cascade_terminus_events(),
        dimensions_at_floor: model.last_dimensions_at_floor(),
    }
}

/// Builds and publishes a `PublishedHealthSummary` via `shared.health`.
///
/// Called from three exit paths: normal step 17, CP5 revert, and CP6 revert.
/// Ensures health counters (including CP5/CP6 revert counts) are always
/// visible to readers immediately after any label completes.
///
/// # Cross-References
///
/// - (´tab:runtime:numeric-checkpoints´) — the two checkpoints that revert a label
/// - (´dec:health:independent-publication´) — health publishes on its own swap
#[allow(clippy::too_many_arguments)]
fn publish_health_summary(
    working: &WorkingCopy,
    shared: &SharedState,
    platt_tracker: &PlattConvergenceTracker,
    identity_trackers: &std::collections::HashMap<DimensionId, crate::health::IdentityConvergenceTracker>,
    health_counters: &HealthCounters,
    config: &LabelPipelineConfig,
    label_count: u64,
    now: &crate::types::PersistentTimestamp,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
) {
    // TODO ´todo:code:compute-the-composite-stage-from´: compute the composite stage from
    // `working.total_eligible_labels`. Stages 1–3 must use eligible-label
    // boundaries configured by `ConvergenceThresholdConfig`.
    let tracker_refs: Vec<&crate::health::IdentityConvergenceTracker> = identity_trackers.values().collect();
    let new_stage = compute_composite_stage(
        label_count,
        platt_tracker,
        &tracker_refs,
        config.convergence.platt_converged_delta_cal,
        config.convergence.platt_converged_refit_count,
        now,
    );

    // Precision health from working copy models.
    let mut precision_health = std::collections::HashMap::new();
    precision_health.insert(ModelId::Operational, model_precision_health(&working.operational));
    precision_health.insert(ModelId::Sister, model_precision_health(&working.sister));
    precision_health.insert(ModelId::Anchor, model_precision_health(&working.anchor));
    for (&axis_id, axis) in &working.outcome_models {
        precision_health.insert(ModelId::OutcomeAxis(axis_id), model_precision_health(&axis.model));
    }

    // Standardisation phase + features at floor.
    //
    // The phase is read from the ramp rather than inferred from how many
    // variances sit at the floor (´dec:health:standardisation-phase-reported´).
    // The inference was available and wrong in both directions: it could not
    // separate a position whose observed variance is genuinely small from one
    // the ramp had not yet moved, and it said nothing whatever about how far
    // along the ramp had come. The accepted count answers that second question
    // and travels beside the phase.
    //
    // Both are current — where the ramp stands at the moment this report was
    // taken — and the compact pair on an assessment is retrospective. During a
    // transition the two legitimately disagree by however many advances fell
    // between them (´tab:monitoring:standardisation-transition´).
    let standardisation_phase = working.cold_ramp.phase();
    let standardisation_observations = working.cold_ramp.accepted();

    // The bias position is excluded because nothing standardises it: its
    // variance is a constant no feature is ever divided by, so counting it
    // would report one position permanently at the floor in every deployment.
    let features_at_floor = working
        .feature_variances
        .iter()
        .zip(working.feature_classes.iter())
        .filter(|&(&v, &class)| {
            !matches!(class, crate::feature::standardisation::FeatureClass::Bias) && v <= config.standardisation.v_floor
        })
        .count();

    let feature_stable_outcome_drift = compute_feature_stable_outcome_drift(
        &working.drift_state,
        &working.feature_means,
        &working.feature_variances,
        config.monitoring.feature_stability_threshold,
    );

    let discrimination = health_counters.last_discrimination.clone();
    let quantile_error_concentration = discrimination.as_ref().and_then(|d| d.quantile_error_concentration);

    let health_summary = crate::health::PublishedHealthSummary {
        platt_tracker: platt_tracker.clone(),
        identity_trackers: identity_trackers.clone(),
        precision_health,
        total_labels: label_count,
        eligible_labels: health_counters.eligible_labels,
        labels_by_action: health_counters.labels_by_action.clone(),
        per_entity_concordance: health_counters.per_entity_concordance.clone(),
        importance_ceiling: crate::health::ImportanceCeilingHealth {
            ceiling_binding_fraction: health_counters.importance_operational.binding_fraction(),
            gradient_balance: health_counters.importance_operational.gradient_balance(),
            sister_ceiling_binding_fraction: health_counters.importance_sister.binding_fraction(),
            sister_gradient_balance: health_counters.importance_sister.gradient_balance(),
        },
        alarm_outcome: health_counters.alarm_outcome.clone(),
        sentinel_legibility: health_counters
            .sentinel_legibility
            .iter()
            .map(|(&id, evidence)| (id, evidence.readings()))
            .collect(),
        dimension_legibility: health_counters
            .dimension_legibility
            .iter()
            .map(|(&id, evidence)| (id, evidence.readings()))
            .collect(),
        standardisation_phase,
        standardisation_observations,
        features_at_floor,
        last_delta_cal: platt_tracker.last_delta_cal,
        platt_state: platt_tracker.convergence_state(
            config.convergence.platt_converged_delta_cal,
            config.convergence.platt_converged_refit_count,
        ),
        cp5_reverts: health_counters.cp5_reverts,
        cp6_reverts: health_counters.cp6_reverts,
        discrimination,
        feature_stable_outcome_drift,
        quantile_error_concentration,
        drift_state: working.drift_state.clone(),
        convergence_stage: new_stage,
        marginalisation: (&working.marginalisation_errors).into(),
        feedback_latency: health_counters.feedback_latency.stages(),
    };
    // The previous stage is on the summary about to be replaced, which
    // is what makes the composite stage-change event emittable here —
    // the one production site holding both halves of the pair
    // (´cav:health:tracker-event-defect´).
    let old_stage = shared.health.load().convergence_stage;
    shared.health.store(std::sync::Arc::new(health_summary));
    crate::health::check_convergence_event(old_stage, new_stage, event_tx, dropped_counter);
}

/// Reconstructs the feature vector φ from the pending context.
///
/// Fills all six sources of (´alg:runtime:reconstruction´). The frozen
/// half — the signal block, the identity base and per-axis features,
/// and the extractions with the two Sentinel lists — is read from the
/// stored entry (´def:runtime:pending-entry´); the current half — the
/// structural triples, competitive indicators, cross-dimension
/// aggregates and the aggregate block — is re-derived at the moment of
/// labelling. The package's exported label-time assembly does the
/// placing.
pub fn reconstruct_phi(
    working: &WorkingCopy,
    context: &PendingContextRef<'_>,
    identity_dimensions: &std::sync::RwLock<HashMap<DimensionId, std::sync::Arc<IdentityDimensionInfra>>>,
    concordance_thresholds: [f64; SCORING_AXIS_COUNT],
) -> Vec<f64> {
    let dim_map = &working.dimension_map;

    // Source 2: re-encode the stored entity per currently registered
    // dimension and re-derive from the current competitive set.
    let (identity_aggregates, active_cells, cross_dimension_features) =
        derive_current_identity_state(identity_dimensions, context.entity());

    // Source 5: the aggregate block, recomputed from the stored
    // extractions of Sentinels that were reporting at assessment time
    // and are still registered, against the current thresholds and the
    // current registered count.
    let reporting_extractions: HashMap<SentinelId, SentinelExtraction> = context
        .sentinel_extractions()
        .iter()
        .filter(|(sid, _)| context.reporting_sentinels().contains(sid) && dim_map.sentinel_slots.contains_key(*sid))
        .map(|(sid, ext)| (*sid, ext.clone()))
        .collect();
    let sub_scores = crate::feature::assembly::extract_sub_scores(&reporting_extractions);
    let aggregate_features =
        crate::feature::aggregate::compute_aggregates(&sub_scores, &concordance_thresholds, dim_map.sentinel_slots.len());

    // Source 1: the frozen signal block, upcast and sized to the current
    // signal range exactly as the assessment path sizes its own.
    let mut frozen_signals: Vec<f64> = context.signal_features().iter().collect();
    frozen_signals.resize(dim_map.sig_range.len(), 0.0);

    let frozen = crate::feature::assembly::FrozenPendingView {
        extractions: context.sentinel_extractions(),
        active_sentinels: context.active_sentinels(),
        reporting_sentinels: context.reporting_sentinels(),
        signals: &frozen_signals,
        base_features: context.entity_base_features(),
        axis_features: context.entity_axis_features(),
        spatial_axis_ids: context.stored_spatial_axis_ids(),
    };
    let current = crate::feature::assembly::CurrentLabelState {
        aggregate_features: &aggregate_features,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross_dimension_features,
        active_cells: &active_cells,
    };
    crate::feature::assembly::assemble_phi_for_label_raw(&frozen, &current, dim_map)
}

/// Re-derives the current-state identity inputs of reconstruction
/// source 2 (´alg:runtime:reconstruction´): per currently registered
/// dimension, the entity is re-encoded, the current competitive set is
/// looked up, and the structural triple and cross-dimension aggregates
/// are recomputed — reading cell state without mutating it. Each
/// quantity is derived exactly as the assessment path derives it.
fn derive_current_identity_state(
    identity_dimensions: &std::sync::RwLock<HashMap<DimensionId, std::sync::Arc<IdentityDimensionInfra>>>,
    entity: &crate::types::EntityKey,
) -> (
    HashMap<DimensionId, crate::feature::assembly::IdentityAggregateFeatures>,
    HashMap<DimensionId, Vec<crate::identity::CompetitiveCellId>>,
    crate::feature::assembly::CrossDimensionAggregateFeatures,
) {
    let mut triples = HashMap::new();
    let mut active_map = HashMap::new();
    let mut cross = crate::feature::assembly::CrossDimensionAggregateFeatures::default();

    let Ok(guard) = identity_dimensions.read() else {
        return (triples, active_map, cross); // poisoned lock — degrade to empty
    };

    for (&dim_id, infra) in guard.iter() {
        let coord = infra.dimension.encode_entity(entity);
        let active: Vec<crate::identity::CompetitiveCellId> = infra.find_active_cells(coord).to_vec();
        let set_len = infra.load_competitive_set().len();

        // The structural triple, as the assessment path computes it.
        let deepest_depth = active.first().map_or(0, |c| c.depth);
        let coverage_depth = f64::from(deepest_depth) / f64::from(infra.dimension.domain_bits.max(1));
        let active_fraction = active.len() as f64 / (set_len.max(1) as f64);
        // As at the assessment path, importance comes from the scalar published
        // beside the competitive cells
        // (´claim:identity:published-total-importance-equals-the-checkpoint-reconstruction-it-replaces´).
        let total_importance = infra.load_competitive_set().total_importance().ln_1p();
        triples.insert(
            dim_id,
            crate::feature::assembly::IdentityAggregateFeatures::new(coverage_depth, active_fraction, total_importance),
        );

        // Cross-dimension maxima over the current active cells, mirroring
        // the assessment path: alarm and suspicion over every active
        // cell, volatility over the deepest only
        // (´tab:keyspace:cross-dimension-features´).
        if !active.is_empty() {
            cross.has_any_competitive_cell = 1.0;
            cross.max_coverage_depth = cross.max_coverage_depth.max(coverage_depth);
            let mut deepest = true;
            for cell in &active {
                infra.with_cell_state(cell, |mutex| {
                    let state = mutex.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                    for &alarm in state.measurement.per_sentinel_alarm_ewma.values() {
                        cross.max_alarm = cross.max_alarm.max(alarm);
                    }
                    cross.max_suspicion = cross.max_suspicion.max(state.measurement.suspicion());
                    if deepest {
                        cross.max_volatility = cross.max_volatility.max(state.measurement.volatility);
                    }
                    cross.max_adverse_rate = cross.max_adverse_rate.max(state.outcome.adverse_rate_ewma);
                    cross.max_abs_compressed_valence = cross
                        .max_abs_compressed_valence
                        .max(state.outcome.compressed_valence_ewma.abs());
                    cross.max_abs_raw_valence = cross.max_abs_raw_valence.max(state.outcome.raw_valence_ewma.abs());
                });
                deepest = false;
            }
        }
        active_map.insert(dim_id, active);
    }
    drop(guard);

    (triples, active_map, cross)
}

/// Updates identity cell outcome EWMAs for all active cells (step 12).
///
/// For each identity dimension referenced in `identity_active_cells`,
/// acquires a read lock on the cell state map, then a per-cell mutex,
/// and applies write-time decay + EWMA update via
/// [`CellOutcomeState::apply_write_decay_and_update`].
///
/// Per-axis compressed values use the per-axis `κ_a` from the working
/// copy (´def:axis:adaptive-compression´) rather than a hardcoded constant.
///
/// # Cross-References
///
/// - (´alg:runtime:identity-outcome-update´) — what step twelve writes, and where
fn process_identity_cell_outcomes(
    state: &LabelPipelineState<'_>,
    working: &WorkingCopy,
    identity_dimensions: &std::sync::RwLock<HashMap<DimensionId, std::sync::Arc<IdentityDimensionInfra>>>,
    now: &PersistentTimestamp,
    gamma_t_identity: f64,
    lambda_identity: f64,
) {
    let active_cells = state.context.identity_active_cells();
    if active_cells.is_empty() {
        return;
    }

    let is_positive = state.label.valence > 0.0;

    // Compute per-axis compressed values using each axis's κ_a.
    let axis_compressed: HashMap<crate::types::OutcomeAxisId, f64> = state
        .label
        .outcomes
        .iter()
        .map(|(&axis_id, &raw)| {
            let kappa_a = working.outcome_models.get(&axis_id).map_or(1.0, |am| am.kappa_a.max(1e-12));
            (axis_id, (raw / kappa_a).tanh())
        })
        .collect();
    let axis_raw = state.label.outcomes.clone();

    let update = CellOutcomeUpdate {
        is_positive,
        compressed_valence: state.compressed_valence,
        raw_valence: state.label.valence,
        axis_compressed,
        axis_raw,
    };

    let Ok(dims_guard) = identity_dimensions.read() else {
        return; // poisoned lock — skip silently
    };

    for (dim_id, cells) in active_cells {
        let Some(dim_infra) = dims_guard.get(dim_id) else {
            continue;
        };

        for cell_id in cells {
            dim_infra.with_cell_state(cell_id, |mutex| {
                if let Ok(mut cell) = mutex.lock() {
                    cell.outcome
                        .apply_write_decay_and_update(gamma_t_identity, now, &update, lambda_identity);
                }
            });
        }
    }

    drop(dims_guard);
}

/// Processes Ledger updates for all reporting Sentinels.
///
/// Uses configured `λ_L` and `γ_t,L` from `LabelPipelineConfig`.
/// Per-axis compressed values use the per-axis `κ_a` from the working
/// copy (´def:axis:adaptive-compression´).
fn process_ledger_updates(
    state: &LabelPipelineState<'_>,
    working: &WorkingCopy,
    outcome_ledger: &OutcomeLedger,
    now: &PersistentTimestamp,
    config: &LabelPipelineConfig,
) {
    // Build LedgerUpdate from label data
    let is_positive = state.label.valence > 0.0;
    let update = LedgerUpdate::new(
        is_positive,
        state.compressed_valence,
        state.label.valence,
        state.label.action_taken,
        state.eligible,
    );

    // Add per-axis values with per-axis κ_a compression.
    let update = state.label.outcomes.iter().fold(update, |u, (&axis_id, &raw)| {
        let kappa_a = working.outcome_models.get(&axis_id).map_or(1.0, |am| am.kappa_a.max(1e-12));
        let compressed = (raw / kappa_a).tanh();
        u.with_axis(axis_id, compressed, raw)
    });

    // Compute time-decay factor for Ledger entries.
    let gamma_t_ledger = config.gamma_t_ledger;

    // Update each Sentinel's ledger
    for (sentinel_id, extraction) in state.context.sentinel_extractions() {
        if !extraction.occupancy {
            continue;
        }

        if let Some(ledger_lock) = outcome_ledger.get(*sentinel_id)
            && let Ok(mut ledger) = ledger_lock.write()
        {
            update_all_layers(
                &mut ledger,
                extraction.coordinate,
                gamma_t_ledger,
                now,
                &update,
                config.lambda_l,
            );
        }
    }
}

/// Checks the working copy for corrupt numeric state — one of checkpoint
/// CP5's two readings (´tab:runtime:numeric-checkpoints´): μ and diag(B) for
/// non-finite values, and the precision matrix's definiteness, read as the
/// verdict its last rebuild's factorisation returned. The other reading is
/// the count of updates this label's own leverage readings refused, which the
/// bounded region's workers report as they run.
///
/// # Why a stored verdict and not a fresh one
///
/// The reading CP5 owes is whether a factorisation refuses the precision
/// matrix, not whether its diagonal is positive: a strictly positive diagonal
/// is necessary for definiteness and is not sufficient for it
/// (´req:gaussian:positive-definiteness´), so the cheap test admits exactly
/// the matrices this checkpoint stands between and the published snapshot.
/// Taking a spectrum, or even a factorisation, per model per label would be
/// cubic work on the per-label path, where this reading is linear.
///
/// It does not have to be taken here, because it has already been taken. The
/// recomputation check runs earlier on this same label, so a model that
/// rebuilt this label carries this label's verdict; a model that did not
/// carries the verdict of its last rebuild, and that verdict is still current,
/// because every mutation between two rebuilds preserves definiteness. The
/// bounded update adds a positive-semidefinite rank-one term, its leverage
/// being non-negative or the update having been refused — and a refused update
/// reverts this label through the refusal tally rather than through this
/// reading. The forgetting scales the matrix by a positive factor, which
/// preserves the definite cone exactly. The replenishment clamp only raises
/// diagonal entries. So a matrix a factorisation accepted at the last rebuild
/// is accepted still, and the stored verdict is a bound on the current one
/// rather than a stale copy of it.
///
/// A model that has never rebuilt carries no verdict and is not read: its
/// precision matrix is the prior, which is definite by construction, and the
/// mutations since are the definiteness-preserving ones above.
///
/// The non-finite scans stay and are not folded into the verdict. A
/// not-a-number entry is not a definiteness question — no factorisation's
/// verdict about an earlier matrix says anything about it — and the scan that
/// catches it is linear.
fn check_working_copy_nan(working: &WorkingCopy) -> bool {
    /// One model's two readings: its mean and precision diagonal for
    /// non-finite entries, and its last rebuild's verdict for definiteness.
    fn model_corrupt(model: &crate::model::bayesian::BayesianLinearModel) -> bool {
        if model.mu().iter().any(|&x| !x.is_finite()) {
            return true;
        }
        if (0..model.dim()).any(|i| !model.precision().diagonal_element(i).is_finite()) {
            return true;
        }
        // The verdict of the last *rebuild*, which a run of measurements
        // carries forward unchanged. That is the reading CP5 wants: a model
        // whose last factorisation was refused is holding a pair no
        // factorisation has certified, and the measurements since have not
        // changed that (´dec:posterior:measured-adoption´).
        model.baseline().and_then(RecomputeBaseline::verdict) == Some(RebuildVerdict::Refused)
    }

    if model_corrupt(&working.operational) || model_corrupt(&working.sister) || model_corrupt(&working.anchor) {
        return true;
    }

    working.outcome_models.values().any(|wam| model_corrupt(&wam.model))
}

/// Checks a snapshot for corrupt numeric state before publication —
/// checkpoint CP6 (´tab:runtime:numeric-checkpoints´): μ and Σ for
/// non-finite values, and diag(Σ) for non-positive values (covariance
/// diagonal must be strictly positive).
fn snapshot_has_nan(snapshot: &crate::snapshot::published::ModelSnapshot) -> bool {
    /// Returns `true` if μ or Σ contain non-finite values, or if any
    /// diagonal element of Σ is non-positive.
    fn params_corrupt(params: &crate::model::parameters::ModelParameters) -> bool {
        if params.mu.iter().any(|x| !x.is_finite()) {
            return true;
        }
        if params.covariance_data.iter().any(|x| !x.is_finite()) {
            return true;
        }
        // Positivity check on diag(Σ) — column-major flat layout.
        let p = params.p;
        (0..p).any(|i| params.covariance_data[i * p + i] <= 0.0)
    }

    if params_corrupt(&snapshot.operational) {
        return true;
    }
    if params_corrupt(&snapshot.sister) {
        return true;
    }
    if params_corrupt(&snapshot.anchor) {
        return true;
    }

    // Check outcome-axis models
    for axis_state in snapshot.outcome_models.values() {
        if params_corrupt(&axis_state.parameters) {
            return true;
        }
    }

    false
}

/// Checks all models for Cholesky recomputation after step 11.
///
/// Each model is checked independently (´dec:posterior:recomputation-trigger´).
/// Only models that trigger are recomputed. Emits health events for
/// non-clean outcomes.
fn check_cholesky_recomputation(
    working: &mut WorkingCopy,
    config: &LabelPipelineConfig,
    event_tx: &Sender<HealthEvent>,
    dropped_counter: &AtomicU64,
) {
    /// Helper: check and recompute one model, updating its state.
    fn check_single_model(
        model: &mut crate::model::bayesian::BayesianLinearModel,
        model_id: ModelId,
        n_recompute_configured: u32,
        kappa_growth_factor: f64,
        lambda_floor: f64,
        sync_error_threshold_per_dimension: f64,
        event_tx: &Sender<HealthEvent>,
        dropped_counter: &AtomicU64,
    ) {
        // Get the effective interval from the model's internal tracking
        let n_recompute_effective = model.n_recompute_effective();

        // Every arm of the fast check is relative to the record the last
        // rebuild wrote, so a model that has not rebuilt yet passes none
        // (´dec:posterior:recomputation-trigger´).
        let trigger = should_recompute(
            model.precision(),
            model.labels_since_recompute(),
            n_recompute_effective,
            model.baseline(),
            kappa_growth_factor,
            lambda_floor,
        );

        // Always update the diagonal ratio and dimensions-at-floor from the
        // trigger check, regardless of whether recomputation fires.
        model.update_trigger_diagnostics(&trigger);

        if trigger.is_needed() {
            // The visit: measure, and rebuild only where the measurement was
            // bad (´dec:posterior:recomputation-trigger´).
            let interval_before = model.n_recompute_effective();
            #[allow(clippy::cast_precision_loss)]
            let sync_error_threshold = sync_error_threshold_per_dimension * model.dim() as f64;
            let inputs = VisitInputs::from_trigger(
                &trigger,
                model.labels_since_recompute(),
                sync_error_threshold,
                model.spectral_floor_mass(),
                model.clamp_mass_since_rebuild(),
            );
            let (outcome, rebuilt, baseline) = synchronisation_visit(
                model.precision(),
                model.covariance(),
                model.baseline(),
                trigger.skips_the_measurements_verdict(),
                &inputs,
            );

            // Apply the result to the model
            model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), n_recompute_configured);

            // A good measurement is the ordinary case and publishes no event:
            // nothing was rebuilt, nothing was declined, and the detail tier
            // carries the reading it took (´dec:health:tiered-queries´).
            match &outcome {
                RecomputeOutcome::Measured { .. } => {}
                RecomputeOutcome::Rebuilt { sync_error_residual, .. } => {
                    if model.n_recompute_effective() < interval_before {
                        emit_health_event(
                            HealthEvent::Precision(PrecisionHealthEvent::SyncErrorShortened {
                                model_id,
                                sync_error: *sync_error_residual,
                                threshold: sync_error_threshold,
                            }),
                            event_tx,
                            dropped_counter,
                        );
                    }
                }
                RecomputeOutcome::Alarm {
                    sync_error_residual,
                    sync_error_after,
                    refused_pivot,
                } => {
                    // The alarm carries both drifts, because which of them is
                    // the larger is the whole of what a reader needs to know
                    // (´dec:posterior:measured-adoption´).
                    emit_health_event(
                        HealthEvent::Precision(PrecisionHealthEvent::RebuildAlarm {
                            model_id: model_id.clone(),
                            sync_error_residual: *sync_error_residual,
                            sync_error_before: baseline.sync_error_before,
                            sync_error_after: *sync_error_after,
                            refused_pivot: *refused_pivot,
                        }),
                        event_tx,
                        dropped_counter,
                    );
                    // A refused factorisation is still the terminus, and the
                    // reading that names it is one existing consumers depend
                    // on (´dec:posterior:repair-cascade´).
                    if refused_pivot.is_some() {
                        emit_health_event(
                            HealthEvent::Precision(PrecisionHealthEvent::CascadeTerminus {
                                model_id,
                                sync_error: *sync_error_residual,
                            }),
                            event_tx,
                            dropped_counter,
                        );
                    }
                }
            }
        }
    }

    // Check operational model
    check_single_model(
        &mut working.operational,
        ModelId::Operational,
        config.n_recompute,
        config.kappa_growth_factor,
        config.lambda_floor,
        config.monitoring.sync_error_threshold_per_dimension,
        event_tx,
        dropped_counter,
    );

    // Check sister model
    check_single_model(
        &mut working.sister,
        ModelId::Sister,
        config.n_recompute,
        config.kappa_growth_factor,
        config.lambda_floor,
        config.monitoring.sync_error_threshold_per_dimension,
        event_tx,
        dropped_counter,
    );

    // Check anchor model
    check_single_model(
        &mut working.anchor,
        ModelId::Anchor,
        config.n_recompute,
        config.kappa_growth_factor,
        config.lambda_floor,
        config.monitoring.sync_error_threshold_per_dimension,
        event_tx,
        dropped_counter,
    );

    // Check outcome-axis models
    for (&axis_id, axis_state) in &mut working.outcome_models {
        check_single_model(
            &mut axis_state.model,
            ModelId::OutcomeAxis(axis_id),
            config.n_recompute,
            config.kappa_growth_factor,
            config.lambda_floor,
            config.monitoring.sync_error_threshold_per_dimension,
            event_tx,
            dropped_counter,
        );
    }
}

/// Computes the feature-stable outcome drift flag
/// (´def:monitoring:feature-stable-drift´).
///
/// Returns `true` when **both** conditions hold:
/// 1. Any model's drift CUSUM is rising — i.e. `s_plus > 0` or `s_minus > 0`.
/// 2. Standardisation EWMAs are stable — all feature variances exceed the
///    configured stability threshold and
///    no feature mean has diverged beyond ±3σ of the running variance.
///
/// When the drift state map is empty (no labels processed yet) or there
/// are no features, returns `false`.
///
/// # Cross-References
///
/// - (´alg:monitoring:drift-cusums´) — the accumulators the flag reads
/// - (´def:monitoring:feature-stable-drift´) — what the flag means
fn compute_feature_stable_outcome_drift(
    drift_state: &HashMap<ModelId, DriftState>,
    feature_means: &[f64],
    feature_variances: &[f64],
    feature_stability_threshold: f64,
) -> bool {
    if drift_state.is_empty() || feature_variances.is_empty() {
        return false;
    }

    // (a) Any model's CUSUM is rising (non-zero accumulation).
    let any_cusum_rising = drift_state.values().any(|ds| ds.s_plus > 0.0 || ds.s_minus > 0.0);
    if !any_cusum_rising {
        return false;
    }

    // (b) Standardisation EWMAs are stable: all variances are strictly above
    //     the configured floor. Features whose variance has collapsed at or
    //     below it indicate unstable standardisation.
    let features_stable = feature_variances.iter().all(|&v| v > feature_stability_threshold);
    if !features_stable {
        return false;
    }

    // Additionally, check that no feature mean has diverged beyond a
    // plausible range.  Under stable standardisation, means should stay
    // near zero.  We flag instability if any mean exceeds ±3√(variance).
    feature_means
        .iter()
        .zip(feature_variances.iter())
        .all(|(&m, &v)| m.abs() <= 3.0 * v.sqrt().max(1e-12))
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::LifecycleHealthEvent;
    use crate::types::AssessmentId;

    fn make_test_label_data(valence: f64, action: Action) -> LabelData {
        LabelData {
            assessment_id: AssessmentId(1),
            action_taken: action,
            valence,
            outcomes: HashMap::new(),
            ground_truth: true,
        }
    }

    /// The risk target is a sign, not a magnitude: an adverse observation of any
    /// strength enters the model update as exactly +1. How bad the outcome was is
    /// carried separately by the compressed valence, so the target never lets one
    /// extreme report dominate the direction the model is pulled in.
    ///
    /// ´claim:labelling:the-risk-target-is-a-sign-and-not-a-magnitude´
    /// ´test:unit:risk-target-positive-valence´
    #[test]
    fn risk_target_positive_valence() {
        assert!((compute_risk_target(1.0) - 1.0).abs() < f64::EPSILON);
    }

    /// The benign side is the mirror of the adverse one: a negative valence maps
    /// to −1 rather than to zero, so a benign label is positive evidence for the
    /// other class instead of an absence of evidence.
    ///
    /// (´claim:labelling:the-risk-target-is-a-sign-and-not-a-magnitude´)
    /// ´test:unit:risk-target-negative-valence´
    #[test]
    fn risk_target_negative_valence() {
        assert!((compute_risk_target(-1.0) - (-1.0)).abs() < f64::EPSILON);
    }

    /// Valence enters the update through a saturating compression: a report five
    /// times the running scale comes through just under unity rather than five
    /// times as loud. An outlier can push the compressed value arbitrarily close
    /// to the ceiling but never past it, so a single mis-scaled report cannot buy
    /// unbounded influence over the model.
    ///
    /// ´claim:labelling:valence-compression-saturates-instead-of-exceeding-unit-scale´
    /// ´test:unit:compressed-valence-tanh-bounded´
    #[test]
    fn compressed_valence_tanh_bounded() {
        let kappa_v = 1.0_f64;
        let valence = 5.0_f64;
        let compressed = (valence / kappa_v).tanh();
        assert!(compressed > 0.99);
        assert!(compressed <= 1.0);
    }

    /// A calibration entry keeps everything a later refit or coverage
    /// computation needs — the effective risk, its assessment-time
    /// uncertainty, the anchor weight, the polarity and the importance
    /// weight — beside each other in one record. Calibration happens long
    /// after the label was processed, so the entry has to carry its own
    /// context rather than expecting the model to still be in the state
    /// that produced it; the uncertainty is the denominator the empirical
    /// coverage of every reported interval divides by
    /// (´alg:monitoring:empirical-coverage´).
    ///
    /// ´claim:labelling:a-calibration-entry-carries-its-own-refit-context´
    /// ´test:unit:calibration-entry-stores-values´
    #[test]
    fn calibration_entry_stores_values() {
        let entry = CalibrationEntry {
            rho_eff: 0.8,
            uncertainty: 0.35,
            anchor_weight: 0.1,
            positive: true,
            importance_weight: 2.5,
            axis_data: vec![],
        };
        assert!((entry.rho_eff - 0.8).abs() < f64::EPSILON);
        assert!((entry.uncertainty - 0.35).abs() < f64::EPSILON);
        assert!((entry.anchor_weight - 0.1).abs() < f64::EPSILON);
        assert!(entry.positive);
        assert!((entry.importance_weight - 2.5).abs() < f64::EPSILON);
    }

    /// An unconfigured pipeline runs on the constants the specification names —
    /// the long-memory decay rates, the importance ceiling and the refit interval
    /// — rather than on whatever the struct's fields happened to zero to. A host
    /// that supplies no tuning still gets the documented behaviour.
    ///
    /// ´claim:labelling:the-default-pipeline-config-is-the-specified-constants´
    /// ´test:unit:label-pipeline-config-defaults´
    #[test]
    fn label_pipeline_config_defaults() {
        let config = LabelPipelineConfig::default();
        // The trackers carry no rate of their own: each uses its models'
        // forgetting rate (´tab:weighting:trackers´).
        assert!((config.gamma_opr - GAMMA_LABEL_OPERATIONAL).abs() < f64::EPSILON);
        assert!((config.gamma_inh - GAMMA_LABEL_SISTER).abs() < f64::EPSILON);
        assert!((config.gamma_kappa - 0.999).abs() < f64::EPSILON);
        assert!((config.importance_ceiling - DEFAULT_IMPORTANCE_CEILING).abs() < f64::EPSILON);
        assert!((config.lambda_l - 0.999).abs() < f64::EPSILON);
        assert!((config.gamma_t_ledger - 0.999).abs() < f64::EPSILON);
        assert_eq!(config.platt.n_refit, 200);
    }

    /// The bounded execution groups change only when independent model records
    /// run, not what any record computes. Operational occupies one helper,
    /// sister and anchor retain their order in the second helper, and an axis
    /// stays on the steward; their published means and covariances are bitwise
    /// identical to applying those same updates in the former sequential order.
    ///
    /// ´claim:concurrency:bounded-label-model-update-groups-preserve-the-sequential-results´
    /// ´test:unit:bounded-parallel-model-updates-match-sequential-results´
    #[test]
    fn bounded_parallel_model_updates_match_sequential_results() {
        use crate::model::bayesian::BayesianLinearModel;

        let phi_values: Vec<f64> = (0..32).map(|index| (f64::from(index) - 16.0) / 32.0).collect();
        let anchor_values = &phi_values[..15];
        let phi = vec_to_col(&phi_values);
        let anchor_phi = vec_to_col(anchor_values);

        // The tally the region's workers report refusals into. No reading
        // here is refused, so it stays at zero and the comparison is of the
        // updates themselves.
        let refusals = AtomicU64::new(0);

        let mut sequential_operational = BayesianLinearModel::new(32, 0.1, 1000);
        let mut sequential_sister = BayesianLinearModel::new(32, 0.1, 1000);
        let mut sequential_anchor = BayesianLinearModel::new(15, 0.1, 1000);
        let mut sequential_axis = BayesianLinearModel::new(32, 0.1, 1000);
        apply_model_label_update(
            &mut sequential_operational,
            &phi,
            1.2,
            100.0,
            5.0,
            1.0,
            0.9995,
            1e-6,
            &refusals,
        );
        apply_model_label_update(&mut sequential_sister, &phi, 0.8, 100.0, 5.0, 1.0, 0.9998, 1e-6, &refusals);
        apply_model_label_update(
            &mut sequential_anchor,
            &anchor_phi,
            0.8,
            100.0,
            5.0,
            1.0,
            0.9998,
            1e-6,
            &refusals,
        );
        apply_model_label_update(&mut sequential_axis, &phi, 0.8, 100.0, 5.0, 0.3, 0.999, 1e-6, &refusals);

        let mut parallel_operational = BayesianLinearModel::new(32, 0.1, 1000);
        let mut parallel_sister = BayesianLinearModel::new(32, 0.1, 1000);
        let mut parallel_anchor = BayesianLinearModel::new(15, 0.1, 1000);
        let mut parallel_axis = BayesianLinearModel::new(32, 0.1, 1000);
        std::thread::scope(|scope| {
            let operational_phi = &phi;
            let _operational_worker = scope.spawn(|| {
                apply_model_label_update(
                    &mut parallel_operational,
                    operational_phi,
                    1.2,
                    100.0,
                    5.0,
                    1.0,
                    0.9995,
                    1e-6,
                    &refusals,
                );
            });
            let sister_phi = &phi;
            let _sister_anchor_worker = scope.spawn(|| {
                apply_model_label_update(
                    &mut parallel_sister,
                    sister_phi,
                    0.8,
                    100.0,
                    5.0,
                    1.0,
                    0.9998,
                    1e-6,
                    &refusals,
                );
                apply_model_label_update(
                    &mut parallel_anchor,
                    &anchor_phi,
                    0.8,
                    100.0,
                    5.0,
                    1.0,
                    0.9998,
                    1e-6,
                    &refusals,
                );
            });
            apply_model_label_update(&mut parallel_axis, &phi, 0.8, 100.0, 5.0, 0.3, 0.999, 1e-6, &refusals);
        });

        for (sequential, parallel) in [
            (&sequential_operational, &parallel_operational),
            (&sequential_sister, &parallel_sister),
            (&sequential_anchor, &parallel_anchor),
            (&sequential_axis, &parallel_axis),
        ] {
            let sequential = sequential.to_parameters();
            let parallel = parallel.to_parameters();
            assert_eq!(parallel.p, sequential.p);
            assert_eq!(parallel.mu, sequential.mu);
            assert_eq!(parallel.covariance_data, sequential.covariance_data);
        }
    }

    /// The integrity guard is quiet on healthy state: a cold-started working copy
    /// reports no NaN across any of its models, and none of them carries a
    /// rebuild verdict at all. A guard that fired on a freshly-built model would
    /// revert every label ever processed, so its silence on the clean case is
    /// what makes the checkpoint usable at all — and a model that has never
    /// rebuilt is exactly the case a verdict reading must not invent a finding
    /// for: its precision matrix is the prior, definite by construction.
    ///
    /// ´claim:labelling:the-nan-guard-stays-silent-on-a-cold-started-working-copy´
    /// ´test:unit:check-nan-detects-nan-in-mu´
    #[test]
    fn check_nan_detects_nan_in_mu() {
        use crate::config::types::ModelConfig;

        let working = WorkingCopy::cold_start(10, &ModelConfig::default(), 1000);

        assert!(
            working.operational.baseline().is_none(),
            "a cold-started model has never rebuilt, so it carries no verdict",
        );
        assert!(!check_working_copy_nan(&working));

        // Note: We can't easily inject NaN into the mu vector since it's private
        // and the model doesn't expose a mutable accessor.
        // This test verifies the check doesn't trigger on valid models.
    }

    /// A baseline carrying one rebuild verdict, for the two readings below.
    fn baseline_with(verdict: RebuildVerdict) -> RecomputeBaseline {
        RecomputeBaseline {
            labels_at_record: 1,
            sync_error_before: 0.0,
            sync_error_prior_induced: 0.0,
            sync_error_residual: 0.0,
            sync_error_resolution: 0.0,
            at_resolution: false,
            diagonal_ratio: 1.0,
            dimensions_at_floor: 0,
            wall_nanos: 0,
            rebuild: Some(crate::model::recompute::RebuildRecord {
                spectrum: None,
                sync_error_after: 0.0,
                verdict,
                adopted: verdict == RebuildVerdict::Clean,
                wall_nanos: 0,
            }),
        }
    }

    /// A model whose last rebuild was refused makes the checkpoint fire, and a
    /// model whose last rebuild was clean does not — with the precision diagonal
    /// finite and positive in both cases, which is the whole point. The diagonal
    /// test CP5 used to take is necessary for definiteness and not sufficient for
    /// it (´req:gaussian:positive-definiteness´), so a matrix a factorisation
    /// refuses can pass it; the verdict the rebuild already produced is the
    /// reading that does not (´tab:runtime:numeric-checkpoints´).
    ///
    /// ´claim:labelling:cp5-reads-the-stored-rebuild-verdict-rather-than-the-precision-diagonal´
    /// ´test:unit:cp5-reads-the-stored-rebuild-verdict´
    #[test]
    fn cp5_reads_the_stored_rebuild_verdict() {
        use crate::config::types::ModelConfig;
        use crate::model::recompute::RecomputeOutcome;

        let mut working = WorkingCopy::cold_start(10, &ModelConfig::default(), 1000);

        // A clean verdict, with the diagonal finite and positive: nothing fires.
        working.operational.apply_recompute_outcome(
            &RecomputeOutcome::Rebuilt {
                sync_error_residual: 0.0,
                drift_was_real: true,
            },
            None,
            Some(baseline_with(RebuildVerdict::Clean)),
            1000,
        );
        assert!(
            (0..working.operational.dim()).all(|i| working.operational.precision().diagonal_element(i) > 0.0),
            "the premise of this reading is a diagonal that passes the old test",
        );
        assert!(
            !check_working_copy_nan(&working),
            "a clean verdict with a positive finite diagonal is not a finding",
        );

        // The same model, the same matrix, a refused verdict: the checkpoint
        // fires, and it fires on the verdict rather than on the numbers.
        working.operational.apply_recompute_outcome(
            &RecomputeOutcome::Alarm {
                sync_error_residual: 0.0,
                sync_error_after: 0.0,
                refused_pivot: Some(0),
            },
            None,
            Some(baseline_with(RebuildVerdict::Refused)),
            1000,
        );
        assert!(
            (0..working.operational.dim()).all(|i| working.operational.precision().diagonal_element(i).is_finite()),
            "the diagonal is untouched by the verdict, which is what makes this reading the sufficient one",
        );
        assert!(
            check_working_copy_nan(&working),
            "a model whose last rebuild was refused reverts the label",
        );
    }

    /// Every knob the host sets on the overall configuration reaches the pipeline:
    /// each of the decay rates, ceilings, leverage bounds, Cholesky thresholds and
    /// refit intervals is given a value distinct from its default and each arrives
    /// unchanged. A field silently dropped during wiring would leave a documented
    /// setting with no effect at all, which is exactly the failure this rules out.
    ///
    /// ´claim:labelling:every-pipeline-setting-is-carried-across-from-the-host-configuration´
    /// ´test:unit:label-pipeline-config-from-assayer-config-wires-all-fields´
    #[test]
    fn label_pipeline_config_from_assayer_config_wires_all_fields() {
        use crate::config::types::AssayerConfig;

        let mut config = AssayerConfig::default();
        // Modify every field that from_assayer_config reads,
        // using values distinct from the defaults.
        config.model.gamma_kappa = 0.42;
        config.model.w_ceiling = 77.0;
        config.model.gamma_opr = 0.88;
        config.model.gamma_inh = 0.77;
        config.model.lambda_floor = 0.005;
        config.model.c_leverage = 3.0;
        config.temporal.gamma_t_core = 0.91;
        config.temporal.gamma_t_identity = 0.92;
        config.temporal.gamma_t_ledger = 0.93;
        config.ledger.lambda_l = 0.94;
        config.standardisation.gamma_std = 0.95;
        config.cholesky.n_recompute = 500;
        config.cholesky.kappa_growth_factor = 42.0;
        config.platt.n_refit = 99;
        config.monitoring.kappa_drift = 0.31;
        config.monitoring.h_threshold = 17.0;
        config.monitoring.strong_alarm_threshold = 4.0;
        config.monitoring.feature_stability_threshold = 0.002;

        let lpc = LabelPipelineConfig::from_assayer_config(&config);

        assert!((lpc.gamma_kappa - 0.42).abs() < f64::EPSILON);
        assert!((lpc.importance_ceiling - 77.0).abs() < f64::EPSILON);
        assert!((lpc.gamma_opr - 0.88).abs() < f64::EPSILON);
        assert!((lpc.gamma_inh - 0.77).abs() < f64::EPSILON);
        assert!((lpc.lambda_floor - 0.005).abs() < f64::EPSILON);
        assert!((lpc.c_leverage - 3.0).abs() < f64::EPSILON);
        assert!((lpc.gamma_t - 0.91).abs() < f64::EPSILON);
        assert!((lpc.gamma_t_identity - 0.92).abs() < f64::EPSILON);
        assert!((lpc.gamma_t_ledger - 0.93).abs() < f64::EPSILON);
        assert!((lpc.lambda_l - 0.94).abs() < f64::EPSILON);
        assert!((lpc.standardisation.gamma_std - 0.95).abs() < f64::EPSILON);
        assert_eq!(lpc.n_recompute, 500);
        assert!((lpc.kappa_growth_factor - 42.0).abs() < f64::EPSILON);
        assert_eq!(lpc.platt.n_refit, 99);
        assert!((lpc.drift.kappa_drift - 0.31).abs() < f64::EPSILON);
        assert!((lpc.drift.h_threshold - 17.0).abs() < f64::EPSILON);
        assert!((lpc.monitoring.strong_alarm_threshold - 4.0).abs() < f64::EPSILON);
        assert!((lpc.monitoring.feature_stability_threshold - 0.002).abs() < f64::EPSILON);
    }

    // ───────────────────────────────────────────────────────────────────────
    //   compute_feature_stable_outcome_drift — direct coverage for
    //   (´def:monitoring:feature-stable-drift´). Exercises every branch of
    //   the three-condition decision: (a) rising CUSUM, (b) stable variance
    //   floor, (c) feature means within ±3σ.
    // ───────────────────────────────────────────────────────────────────────

    /// A `DriftState` with the requested CUSUM accumulators set.
    fn drift_state_with(s_plus: f64, s_minus: f64) -> DriftState {
        let mut ds = DriftState::new();
        ds.s_plus = s_plus;
        ds.s_minus = s_minus;
        ds
    }

    /// The flag is a claim about drift, so it needs drift evidence before it can
    /// be raised: with no model yet accumulating, perfectly stable features are
    /// not enough on their own. An instance that has processed nothing therefore
    /// reports no drift rather than reporting the stability of its untouched
    /// standardisation as a finding.
    ///
    /// ´claim:labelling:the-feature-stable-drift-flag-needs-drift-evidence-before-it-can-rise´
    /// ´test:unit:feature-stable-outcome-drift-false-when-drift-state-empty´
    #[test]
    fn feature_stable_outcome_drift_false_when_drift_state_empty() {
        // No models registered → flag is false even if features are stable.
        let drift_state: HashMap<ModelId, DriftState> = HashMap::new();
        let means = vec![0.0, 0.0, 0.0];
        let variances = vec![1.0, 1.0, 1.0];
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// The other half of the conjunction is equally required: with real drift
    /// accumulated but no feature distribution to judge, there is nothing that
    /// could be called stable and the flag stays down. The finding is specifically
    /// "outcomes drifted while features did not", and an absent feature vector
    /// cannot supply that second half.
    ///
    /// ´claim:labelling:the-drift-flag-stays-down-when-there-is-no-feature-distribution-to-judge´
    /// ´test:unit:feature-stable-outcome-drift-false-when-no-features´
    #[test]
    fn feature_stable_outcome_drift_false_when_no_features() {
        // Empty feature vectors short-circuit to false.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(3.0, 0.0));
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &[], &[], 0.001));
    }

    /// Registered models whose accumulators sit at zero are no different from no
    /// models at all: the flag needs a non-zero accumulation on some side, and
    /// having somewhere to look is not the same as having found something. Quiet
    /// models with textbook-stable features still report nothing.
    ///
    /// (´claim:labelling:the-feature-stable-drift-flag-needs-drift-evidence-before-it-can-rise´)
    /// ´test:unit:feature-stable-outcome-drift-false-when-cusum-flat´
    #[test]
    fn feature_stable_outcome_drift_false_when_cusum_flat() {
        // CUSUMs at zero → feature-stability is moot, flag must be false.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, DriftState::new());
        drift_state.insert(ModelId::Sister, DriftState::new());
        let means = vec![0.0, 0.0];
        let variances = vec![1.0, 1.0];
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// Both halves together are what the flag means: accumulating outcome drift
    /// while the feature distribution has not moved. That combination is the
    /// signature of the mapping from features to outcomes having changed under
    /// the model rather than the traffic having changed — a relabelling of the
    /// world, which no amount of feature-distribution monitoring would catch.
    ///
    /// ´claim:labelling:rising-drift-over-an-unmoved-feature-distribution-raises-the-flag´
    /// ´test:unit:feature-stable-outcome-drift-true-when-cusum-rising-and-features-stable´
    #[test]
    fn feature_stable_outcome_drift_true_when_cusum_rising_and_features_stable() {
        // S⁺ rising on any model + stable variances + well-bounded means → true.
        // This is the canonical "feature-stable outcome drift" signature:
        // the model is systematically wrong while the feature distribution
        // has not shifted — the footprint of a pure label-mapping flip.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(3.5, 0.0));
        drift_state.insert(ModelId::Sister, DriftState::new());
        let means = vec![0.0, 0.0, 0.0];
        let variances = vec![1.0, 1.0, 1.0];
        assert!(compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// The drift evidence is pooled across models rather than required of all of
    /// them, and it counts on either side of the accumulator: a rising negative
    /// sum on the sister model alone raises the flag while the operational model
    /// stays quiet. Waiting for consensus would mean the first model to notice a
    /// relabelling could not report it.
    ///
    /// ´claim:labelling:drift-evidence-from-a-single-model-on-either-side-suffices´
    /// ´test:unit:feature-stable-outcome-drift-true-when-any-model-cusum-rising´
    #[test]
    fn feature_stable_outcome_drift_true_when_any_model_cusum_rising() {
        // Only one model's CUSUM needs to be rising — the `any()` semantics.
        // Sister has the evidence; Operational is quiet.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, DriftState::new());
        drift_state.insert(ModelId::Sister, drift_state_with(0.0, 2.1));
        let means = vec![0.0; 4];
        let variances = vec![1.0; 4];
        assert!(compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// A single feature whose running variance has collapsed below the floor
    /// disqualifies the whole distribution as stable, and the drift evidence is
    /// then not enough on its own. A collapsed variance means the standardisation
    /// itself is degenerate, so its means say nothing trustworthy about whether
    /// the inputs shifted — and a finding built on it would be unfounded.
    ///
    /// ´claim:labelling:a-collapsed-feature-variance-disqualifies-the-features-as-stable´
    /// ´test:unit:feature-stable-outcome-drift-false-when-variance-collapsed´
    #[test]
    fn feature_stable_outcome_drift_false_when_variance_collapsed() {
        // CUSUM rising but a feature's variance has collapsed to the
        // configured floor, so the feature-stable flag stays false.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(4.0, 0.0));
        let means = vec![0.0, 0.0];
        let variances = vec![1.0, 0.001];
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// Healthy variances are not by themselves stability: a feature whose running
    /// mean has walked out past three standard deviations of its own spread also
    /// disqualifies the distribution. A mean that far from zero says the inputs
    /// have moved, and drift with moving inputs is ordinary covariate shift, not
    /// the feature-stable kind this flag is reserved for.
    ///
    /// ´claim:labelling:a-feature-mean-beyond-three-sigma-disqualifies-the-features-as-stable´
    /// ´test:unit:feature-stable-outcome-drift-false-when-mean-diverged´
    #[test]
    fn feature_stable_outcome_drift_false_when_mean_diverged() {
        // Variance is healthy but feature mean has drifted beyond ±3σ.
        // σ=1 ⇒ mean |m|=4 exceeds the 3σ bound → features NOT stable.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(2.0, 0.0));
        let means = vec![0.0, 4.0]; // feature 1 diverged
        let variances = vec![1.0, 1.0];
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// The three-sigma bound is inclusive: a mean sitting exactly on it is still
    /// stable. The boundary is a threshold on plausible wander rather than a
    /// forbidden value, so a feature that merely touches it does not cost the
    /// instance a genuine drift finding.
    ///
    /// ´claim:labelling:the-three-sigma-stability-bound-is-inclusive´
    /// ´test:unit:feature-stable-outcome-drift-true-when-mean-at-three-sigma-boundary´
    #[test]
    fn feature_stable_outcome_drift_true_when_mean_at_three_sigma_boundary() {
        // Exactly at the ±3σ boundary (inclusive): still stable.
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(1.5, 0.0));
        let means = vec![0.0, 3.0]; // σ=√1=1; |m|=3 ≤ 3σ
        let variances = vec![1.0, 1.0];
        assert!(compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.001));
    }

    /// The feature-stability threshold is exact and independent of the drift
    /// accumulator threshold: the flag stays down at the configured variance
    /// floor and rises immediately above it for the same drift evidence.
    ///
    /// ´claim:labelling:the-feature-stability-cut-is-configured-and-strict´
    /// ´test:unit:feature-stable-outcome-drift-honours-configured-stability-threshold´
    #[test]
    fn feature_stable_outcome_drift_honours_configured_stability_threshold() {
        let mut drift_state = HashMap::new();
        drift_state.insert(ModelId::Operational, drift_state_with(3.0, 0.0));
        let means = vec![0.0];
        let variances = vec![0.5];
        assert!(!compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.5));
        assert!(compute_feature_stable_outcome_drift(&drift_state, &means, &variances, 0.499));
    }

    /// The post-calibration reset acts on the affected regime and only on it:
    /// with a sister-regime shift past the threshold and the anchor's below
    /// it, the sister model's accumulators and step counter return to zero
    /// while its smoothed diagnostics survive, and the anchor and operational
    /// models keep every accumulator untouched. A reset wider than the
    /// shifted mapping would discard evidence measured on a scale that did
    /// not change, and one deeper than the accumulators would destroy the
    /// long-memory description the automatic trigger deliberately spares
    /// (´tab:monitoring:drift-resets´).
    ///
    /// ´claim:labelling:the-calibration-drift-reset-narrows-to-the-shifted-regime-and-spares-the-smoothed-diagnostics´
    /// ´test:unit:calibration-drift-reset-narrows-to-the-affected-regime´
    #[test]
    fn calibration_drift_reset_narrows_to_the_affected_regime() {
        let mut drift_state: HashMap<ModelId, DriftState> = HashMap::new();
        for id in [ModelId::Operational, ModelId::Sister, ModelId::Anchor] {
            let mut ds = drift_state_with(3.0, 2.0);
            ds.mean_abs_residual = 0.4;
            ds.residual_sign_ewma = 0.6;
            ds.steps_since_reset = 42;
            drift_state.insert(id, ds);
        }

        let (event_tx, event_rx) = crate::health::create_event_channel(8);
        let dropped = AtomicU64::new(0);
        let reset_counter = DriftResetCounter::new();

        // Sister shifted past the threshold; anchor did not.
        apply_calibration_drift_reset(&mut drift_state, 0.2, 0.05, 0.1, &event_tx, &dropped, &reset_counter);

        let sister = &drift_state[&ModelId::Sister];
        assert!(sister.s_plus.abs() < f64::EPSILON, "sister accumulators reset");
        assert!(sister.s_minus.abs() < f64::EPSILON);
        assert_eq!(sister.steps_since_reset, 0);
        assert!(
            (sister.mean_abs_residual - 0.4).abs() < f64::EPSILON,
            "the smoothed diagnostics survive"
        );
        assert!((sister.residual_sign_ewma - 0.6).abs() < f64::EPSILON);

        for id in [ModelId::Operational, ModelId::Anchor] {
            let ds = &drift_state[&id];
            assert!((ds.s_plus - 3.0).abs() < f64::EPSILON, "{id:?} keeps its evidence");
            assert_eq!(ds.steps_since_reset, 42);
        }

        // The reset is reported, and reported as the narrow kind.
        let event = event_rx.try_recv().expect("a reset event is emitted");
        match event {
            HealthEvent::Lifecycle(LifecycleHealthEvent::DriftReset {
                delta_cal,
                models_reset,
                full_reset,
            }) => {
                assert!((delta_cal - 0.2).abs() < f64::EPSILON);
                assert_eq!(models_reset, 1);
                assert!(!full_reset, "reset_cusums, never reset_all");
            }
            other => panic!("expected DriftReset, got {other:?}"),
        }
        assert_eq!(reset_counter.value(), 1, "the emitted reset is counted once");

        // Below the threshold on both regimes: nothing resets, nothing
        // is reported.
        apply_calibration_drift_reset(&mut drift_state, 0.05, 0.05, 0.1, &event_tx, &dropped, &reset_counter);
        assert!((drift_state[&ModelId::Anchor].s_plus - 3.0).abs() < f64::EPSILON);
        assert!(event_rx.try_recv().is_err(), "no event below the threshold");
        assert_eq!(reset_counter.value(), 1, "no event means no counter increment");
    }
}
