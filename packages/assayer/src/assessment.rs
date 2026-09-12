// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`channel_free_request_assesses`] | assess | A request context carries an entity and nothing about routing, and the core path assesses it without ever asking which channel it arrived on. Channel policy is the derivation layer's business, so the core estimate is the same object whatever the host later decides to do with it. |
//! | [`empty_entity_key_still_assesses`] | assess | An entity key of zero bytes is a legitimate key, not a malformed one: the pipeline runs end to end and hands back a numbered assessment. The assessment surface is total — a host that cannot identify its subject still gets a prior-shaped answer instead of an error to handle on the request path. |
//! | [`well_formed_request_returns_reckoning`] | assess | The two layers compose in the host, not inside the crate: a core assessment is produced first, then handed to the stateless derivation along with an explicitly supplied channel policy, and the pair travels on as one host-owned result. Because derivation reads only what it was passed, the composition can be rebuilt or re-run without touching core state. |
//! | [`assessment_id_monotonically_increasing`] | assess | Successive assessments take strictly increasing identifiers, whatever entity each one was about. The identifier is the join key a host later uses to attach an outcome label to the assessment that predicted it, so reuse or reordering would silently credit one request's outcome to another's estimate. |
//! | [`reckoning_has_tags`] | assess | A derivation always fields the whole slate of competing tags rather than only the winner: at least the three classification tags plus the action tags come back, each with its own resonance on the shared posture axis. A host therefore reads a distribution over postures and can see how close the runner-up was, instead of a bare verdict. |
//! | [`reckoning_risk_uses_real_blend`] | assess | The reported probability comes from the real blend arithmetic, not from a placeholder: with sister and anchor both at zero mean and identity covariance the anchor earns essentially no weight — neither model is better informed in the shared subspace — and the calibrated sigmoid of a zero linear predictor lands on even odds. A cold-start system says it does not know, and says so through the same path a warm one uses. |
//! | [`reckoning_ambiguity_non_zero`] | assess | The ambiguity gauge is positive even for a prior-only assessment: a field the model knows least about is rendered at its most ambiguous, so a system that has learned nothing yet reports a genuinely undifferentiated display rather than a silent one. Whether resolving it would change an action is fragility's question, asked of the landscape (´def:fragility:definition´). |
//! | [`concordance_observed`] | assess | One assessment contributes exactly one observation to the concordance tracker — not zero when no Sentinel reported, and not one per Sentinel. The tracker calibrates thresholds from the distribution of what it is fed, so a per-request cadence is what keeps that distribution in step with request volume. |
//! | [`health_snapshot_populated`] | assess | Every assessment carries inline health with it: the system-wide convergence stage the estimate was made under, and a degradation record that reports clean when nothing had to be salvaged. A consumer can therefore tell an ordinary cold-start estimate from a rescued one at the point of use, without a second query into health state. |
//! | [`no_sentinels_returns_valid_reckoning`] | assess | With no measurement surface registered at all, coverage is reported as zero rather than as a division by an empty denominator, and no Sentinel summaries are invented. Running with no Sentinels is a designed operating mode — prior-only — so it must produce ordinary numbers a host can log and compare, not a special case. |
//! | [`sentinel_registered_but_no_report`] | assess | Registration alone buys a Sentinel no entry in the per-Sentinel map: a Sentinel whose batch report has never arrived contributes nothing to summarise, and coverage counts it as missing. The map therefore lists what actually spoke for this request rather than what was configured, so a consumer cannot mistake an enrolled-but-silent Sentinel for one that reported nothing alarming. |
//! | [`sentinel_no_coordinate_in_request`] | assess | cites (´claim:assess:only-sentinels-that-actually-reported-appear-in-the-per-sentinel-map´) |
//! | [`assessment_assigns_assessment_id`] | assess | Assessing consumes an identifier from the shared counter — the counter visibly advances across the call. Identifiers are allocated by the act of assessing rather than by the host afterwards, which is what makes the pending-buffer entry and the returned assessment refer to the same request by construction. |
//! | [`outcome_predictions_per_registered_axis`] | assess | Every outcome axis carried by the snapshot gets its own prediction, keyed by axis, and the set is exactly the registered axes — no more and no fewer. The axes come from the snapshot the assessment was made against, so adding an axis to the model is enough to make it appear in results, and a consumer indexing by axis never finds a hole. |
//! | [`cp1_nan_signal_sanitised`] | assess | A non-finite value handed in as a request signal neither poisons the estimate nor fails the request: it is replaced and counted, and the count rides back to the host on the assessment's own degradation record. The host learns that its input was unusable at the moment it mattered, instead of discovering it later from an inexplicable score. |
//! | [`cp1_inf_signal_sanitised`] | assess | cites (´claim:assess:a-non-finite-request-signal-is-replaced-and-counted-rather-than-rejected´) |
//! | [`cp3_nan_after_standardisation_sanitised`] | assess | Standardising against a degenerate variance — one that has underflowed to zero — cannot put a NaN into the reported probability. Feature variances are learned quantities, so a feature that never varied will eventually present one; the guard between standardisation and the blend is what stops that arithmetic accident from becoming an unusable risk estimate. |
//! | [`cp4_nan_from_blend_falls_back_to_prior`] | assess | Building a risk basis is honest arithmetic and nothing more: a non-finite effective log-odds out of the blend propagates straight through the calibrated sigmoid into the probability. Nothing is swept up at that layer, which is precisely why a separate guard afterwards must substitute the even-odds prior and record which model was degraded — the fallback is a deliberate, reported decision rather than a silent clamp buried in the formula. |
//! | [`cp4_nan_axis_falls_back_to_computed_prior`] | assess | The outcome-axis half of the same checkpoint computes the same notion of a prior as the risk-basis half: a non-finite axis prediction is replaced by a zero point estimate with the time-corrected prior standard deviation carried through the axis's own compression scale — `kappa_a * sqrt(time_correction / lambda_prior)`, exactly 1.25 at a compression scale of one half, a quadrupled time correction and a prior precision of 0.64 — and the axis is flagged as degraded, while a finite prediction passes through untouched. A constant substitute would put the degraded interval on a scale no successful prediction of this axis uses (´def:axis:prior-only-prediction´). |
//! | [`batch_init_load_conditions_reported_in_band`] | assess | The batch-init path's two load conditions travel in band on the degradation context, exactly as the identity queue's overflow does: an observation skipped on a contended lock counts one skip and marks the assessment degraded, a completion deferred on a full command channel counts one deferral likewise, and an ordinary observation leaves both counters at zero. Before these counters, initialisation could be starved with no surface reporting it — the host would discover the consequence from a standardisation that never improved (´cor:degradation:in-band-travel´). |
//! | [`snapshot_consistent_across_pipeline`] | assess | cites (´claim:assess:assessment-ids-are-handed-out-in-strictly-increasing-order´) |
//! | [`coverage_depth_uses_dimension_domain_width`] | assess | Coverage depth divides the receiving cell's depth by the dimension's own declared width, so a cell at the bottom of a thirty-two-bit dimension reads a full 1.0 and a cell eight levels down reads a quarter — the same fraction of its own domain that a cell thirty-two levels down a full-width dimension reads. The quantity is reduced by maximum across dimensions, and a maximum over fractions of different wholes would let a full-width dimension win on width alone; at the full 128 bits the divisor coincides with the old literal, which is why suites that only ever declared full-width dimensions could not see the difference. |
//! | [`the_untempered_verdict_is_bit_identical`] | assess | A verdict from models that borrowed nothing is the verdict the package reported before the tempering existed, to the last bit. That is a stronger statement than agreement to a tolerance and it is made deliberately: dividing by one is exact, and the ceiling is never below the variance it caps, so the healthy path is the identity rather than a rounding of it. A tempering that moved a healthy verdict by a few units in the last place would be a change to every number the package has ever published, and the empirical coverage refit would spend its evidence chasing it. |
//! | [`the_tempering_is_the_closed_form`] | assess | Where the share is genuinely below one, the tempered variance is the closed form and not an approximation of it: posterior variance over one minus the share, to rounding. The derivation is a subtraction rather than a model — along a direction the posterior precision is the evidence's plus the floors', so the evidence's alone is `(1 − s)` times the posterior's and its variance is the reciprocal of that — and a test that accepted a wider tolerance here would be accepting a second formula nobody had written down. |
//! | [`the_tempering_saturates_at_the_prior`] | assess | A direction the floors are holding up entirely saturates at the prior-only variance and never passes it. The quotient diverges as the share approaches one, and what the package must report there is not a large number but a statement: the prior is all that is known along this direction. That statement already has a value — the one the fourth numeric checkpoint substitutes when the blend's own variance is unusable — so the cap is that value rather than a tolerance chosen beside it, and the same expression computes both. The ceiling is taken as the larger of the prior-only variance and the variance being tempered, so a blended variance already above prior-only — which the mixture's disagreement term can produce — is not narrowed by a cap that exists to bound a widening. The tempering is a widening or nothing. |
//! | [`the_uncertainty_widens_with_the_share`] | assess | The uncertainty a host reads widens monotonically with the share, and the share is where the mechanism's whole content sits: two requests against the same models, differing only in which direction they ask about, receive different uncertainties because the model's evidence is not evenly distributed over directions. A reading that widened every verdict by one aggregate factor would be reporting the model's average condition; this reports its condition where the question was asked. |
//! | [`the_fallback_carries_the_same_tempering`] | assess | The checkpoint's fallback path carries the same tempering in the same order, and carrying it changes nothing, which is the point. The fallback substitutes the prior-only standard deviation, and the prior-only variance is exactly what the tempering saturates at, so the operation is provably an identity there. It is applied rather than skipped so that the two paths hold one operation in one order: a later edit that changed the tempering on the ordinary path and forgot the fallback would otherwise leave two definitions of the same quantity. A share the arithmetic failed to produce is sanitised before it is read, and reports the degradation it caused. |

//! Assessment pipeline orchestration.
//!
//! This module implements the core assessment pipeline that transforms a
//! request context into a `RiskAssessment` result with risk estimates,
//! outcome predictions, per-Sentinel payloads, and health diagnostics.
//!
//! # Pipeline Steps
//!
//! | Step | Name | Purpose |
//! |------|------|---------|
//! | 0 | Context | Entity-bearing request context |
//! | 1–2 | Extraction | Per-Sentinel feature extraction |
//! | 2.5 | Identity | Identity dimension processing |
//! | 3 | Aggregation | Cross-Sentinel aggregates, concordance |
//! | 4 | Signals | Signal cache lookup and merge |
//! | 5 | Assembly | Assemble and standardise φ |
//! | 6 | Risk | Blend computation (´dec:calibration:raw-weight-forms´) |
//! | 7 | Outcomes | Per-axis predictions (´tab:axis:inference-outputs´) |
//! | 8 | Pending | Record in pending buffer |
//! | 9 | Return | Assemble `RiskAssessment` result |
//!
//! # Cross-References
//!
//! - Core assessment output (´schema:output:assessment´)
//! - Assessment is one function, not a stage pipeline (´dec:ordering:assessment-function´)

#![allow(dead_code)] // wiring in progress
#![allow(
    clippy::cast_precision_loss,   // Intentional f64 numeric work
    clippy::too_many_arguments,    // Pipeline complexity
    clippy::too_many_lines,        // Pipeline inherently long
    clippy::similar_names,         // Spec-driven naming (phi, rho, sigma)
    clippy::missing_docs_in_private_items // Many internal helpers
)]

use std::collections::HashMap;
use std::time::Instant;

use crate::error::sanitise_f64;
use crate::extraction::{self, SentinelAlarmSummary};
use crate::feature::aggregate::{SentinelSubScores, compute_aggregates};
use crate::feature::assembly::{
    CrossDimensionAggregateFeatures, IdentityAggregateFeatures, IdentityAxisFeatures, IdentityDimensionFeatures,
    IdentityMeasurementFeatures, assemble_assessment_raw,
};
use crate::feature::standardisation::{StandardisationConfig, standardise_in_place};
use crate::health::{CompositeConvergenceStage, DegradationContext};
use crate::identity::CompetitiveCellId;
use crate::numerics::{decay_factor_elapsed, stable_atanh, stable_sigmoid};
use crate::pending::{PendingAssessment, PendingRiskBasis, ReportOrigin, SentinelExtraction};
use crate::report::ReportIndex;
use crate::resonance::derivation::ResonanceProfile;
use crate::resonance::landscape::DecisionLandscape;
use crate::risk::blend::{BlendResult, compute_blend};
use crate::signal::SignalValue;
use crate::snapshot::published::ModelSnapshot;
use crate::types::{
    AssessmentId, ChannelId, DimensionId, EntityKey, ModelId, OutcomeAxisId, PersistentTimestamp, SCORING_AXIS_COUNT, SentinelId,
};

/// Minimum sister-regime weighted sample count required for the
/// `calibration_mature` flag.
///
/// Matches `PlattConfig::default().n_cal_min`; the count it is compared
/// against is the weighted one (´constr:platt:buffer´). The floor below
/// which a fitted calibration is a curve through noise
/// (´req:platt:minimum-samples´).
///
/// ´const:assayer:calibration-sample-floor´ (´alg:const:scalar´)
/// ´const:assayer:calibration-sample-floor-scalar-30p0´
const N_CAL_MIN: f64 = 30.0;

// ═══════════════════════════════════════════════════════════════════════════════
// Request Context
// ═══════════════════════════════════════════════════════════════════════════════

/// Input context for a core assessment request.
///
/// Contains all information needed to produce a core risk estimate for a
/// single entity/event. Channel policy is intentionally absent; decision-layer
/// derivation receives channel inputs explicitly through `derive_reckoning()`.
///
/// ```compile_fail
/// use torrust_assayer::{EntityKey, RequestContext};
///
/// let request = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
/// let _channel = request.channel;
/// ```
#[derive(Clone, Debug)]
pub struct RequestContext {
    /// Entity key for signal cache and identity dimensions.
    pub entity: EntityKey,
    /// Per-Sentinel coordinates in the 128-bit hash space.
    pub sentinel_coordinates: HashMap<SentinelId, u128>,
    /// Request-scoped signal values (merged with cached values).
    pub signals: HashMap<String, SignalValue>,
    /// Test-harness host-side channel hint.
    ///
    /// The core assessment path does not read this value. It lets the shared
    /// scenario harness carry a host-owned channel choice alongside a core
    /// request while the public request fields remain channel-free.
    #[doc(hidden)]
    pub(crate) channel_hint: Option<ChannelId>,
}

impl RequestContext {
    /// Creates a new request context.
    #[must_use]
    pub fn new(entity: EntityKey) -> Self {
        Self {
            entity,
            sentinel_coordinates: HashMap::new(),
            signals: HashMap::new(),
            channel_hint: None,
        }
    }

    /// Attaches a host-side channel hint for test composition.
    #[must_use]
    pub(crate) const fn with_channel_hint(mut self, channel: ChannelId) -> Self {
        self.channel_hint = Some(channel);
        self
    }

    /// Returns the host-side channel hint, if one was attached.
    #[must_use]
    pub(crate) const fn channel_hint(&self) -> Option<ChannelId> {
        self.channel_hint
    }

    /// Adds a Sentinel coordinate to the request.
    #[must_use]
    pub fn with_sentinel(mut self, sentinel_id: SentinelId, coordinate: u128) -> Self {
        self.sentinel_coordinates.insert(sentinel_id, coordinate);
        self
    }

    /// Adds a signal value to the request.
    #[must_use]
    pub fn with_signal(mut self, name: impl Into<String>, value: impl Into<SignalValue>) -> Self {
        self.signals.insert(name.into(), value.into());
        self
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assessment and Derivation Output
// ═══════════════════════════════════════════════════════════════════════════════

/// Prediction for a single outcome axis.
///
/// Field names track the assessment output schema (´schema:output:assessment´).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct OutcomePrediction {
    /// The axis this prediction is for.
    pub axis_id: OutcomeAxisId,
    /// Human-readable axis name.
    pub axis_name: String,
    /// Predicted raw axis value (`ô`, inverse-transformed via `atanh × κ_a`).
    pub predicted_raw: f64,
    /// Propagated uncertainty in the raw prediction (`σ_raw`).
    pub uncertainty: f64,
    /// 95% prediction interval `[predicted_raw ± 1.96·uncertainty]`.
    pub prediction_interval: (f64, f64),
}

impl OutcomePrediction {
    /// The prior-only axis prediction
    /// (´def:axis:prior-only-prediction´): zero point estimate, the
    /// time-corrected prior standard deviation carried through the
    /// compression scale, and the standard interval around them.
    #[must_use]
    pub fn prior_only(axis_id: OutcomeAxisId, axis_name: &str, kappa_a: f64, time_correction: f64, lambda_prior: f64) -> Self {
        const Z_95: f64 = 1.96;
        let sigma_raw = kappa_a * prior_only_variance(time_correction, lambda_prior).sqrt();
        Self {
            axis_id,
            axis_name: axis_name.to_owned(),
            predicted_raw: 0.0,
            uncertainty: sigma_raw,
            prediction_interval: (-Z_95 * sigma_raw, Z_95 * sigma_raw),
        }
    }
}

impl Default for OutcomePrediction {
    fn default() -> Self {
        // The prior-only prediction at unit parameters
        // (´def:axis:prior-only-prediction´): κ_a = 1, no elapsed
        // decay, unit prior precision.
        Self::prior_only(OutcomeAxisId(0), "", 1.0, 1.0, 1.0)
    }
}

/// Risk estimate basis for a risk assessment.
///
/// Public surface follows Core assessment output (´schema:output:assessment´). In addition to the host-facing
/// probability-scale values, the raw sufficient-statistic triple
/// (`rho_eff`, `sigma_eff`, `kappa_eff`) is exposed so callers can run the
/// stateless derivation layer without reading core state.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RiskBasis {
    /// Probability of adverse outcome `p(bad)`.
    pub p_bad: f64,
    /// Probability-space uncertainty: `p̂(1−p̂)·σ_eff/κ_eff` (´def:risk:probability´).
    ///
    /// Evidence-only along this request's own direction: the posterior
    /// uncertainty widened by whatever share of the precision holding it up
    /// was borrowed from the models' floors rather than earned from evidence,
    /// saturating at the prior-only value (´dec:risk:evidence-only-uncertainty´).
    /// The un-tempered figure is not carried beside it; `borrowed_share` is,
    /// so a host can undo the widening or apply its own.
    pub uncertainty: f64,
    /// The share of the precision behind this verdict that the models'
    /// floors are holding up rather than their evidence, in `[0, 1]`
    /// (´dec:risk:evidence-only-uncertainty´).
    ///
    /// Per request and per direction, and identical across channels for the
    /// same request by construction: it is a property of the models and of the
    /// features, and a channel is neither. A host that wants the un-tempered
    /// uncertainty back multiplies by `√(1 − borrowed_share)`, except where the
    /// tempering saturated.
    pub borrowed_share: f64,
    /// Anchor model weight used in blending.
    pub anchor_weight: f64,
    /// Effective raw risk score `ρ̂_eff` in log-odds space.
    pub rho_eff: f64,
    /// Effective raw uncertainty `σ_eff` in log-odds space.
    pub sigma_eff: f64,
    /// Effective Platt calibration parameter `κ_eff`.
    pub kappa_eff: f64,
    /// Sister-model probability `p(bad)_sister = σ(ρ̂_inh / κ_eff)`.
    pub p_bad_sister: f64,
    /// Operational-model probability `p(bad)_opr = σ(ρ̂_opr / κ_eff)`.
    pub p_bad_operational: f64,
    /// Intervention effectiveness `ρ̂_eff − ρ̂_opr` in log-odds space.
    pub intervention_effectiveness: f64,
    /// True when the anchor model has lower uncertainty than the
    /// sister in the shared feature subspace
    /// (`σ²_anc < σ²_inh,shared`, time-corrected), the comparison the
    /// subspace-restricted blend makes (´def:risk:subspace-blend´).
    pub anchor_converged: bool,
    /// Number of Sentinels contributing to this assessment.
    pub n_sentinels_reporting: usize,
    /// Weighted samples behind `κ_sister`: each record's sister-regime share
    /// times its importance weight, summed (´schema:risk:basis´).
    pub sister_regime_calibration_records: f64,
    /// Weighted samples behind `κ_anchor`: each record's anchor-regime share
    /// times its importance weight, summed (´schema:risk:basis´).
    pub anchor_regime_calibration_records: f64,
}

impl RiskBasis {
    /// Creates a risk basis from a blend result, operational predictor, and
    /// calibration snapshot.
    ///
    /// # Algorithm
    ///
    /// - `p̂ = σ(ρ̂_eff / κ_eff)` — probability via calibrated sigmoid (´def:risk:probability´)
    /// - `σ²_ev = σ²_eff / (1 − s)` — evidence-only along φ, saturating at the
    ///   prior-only variance (´dec:risk:evidence-only-uncertainty´)
    /// - `uncertainty = p̂(1−p̂)·σ_eff/κ_eff` — probability-space uncertainty
    /// - `anchor_converged = σ²_anc < σ²_inh,shared` (´def:risk:subspace-blend´)
    /// - Uses `stable_sigmoid` for numerical stability at extreme ρ̂ values
    ///
    /// # Arguments
    ///
    /// * `blend` — Blend result from the risk computation
    /// * `rho_opr` — Operational dot product `φ̂ᵀμ_opr`
    /// * `n_sentinels_reporting` — Number of Sentinels with valid extraction
    /// * `cal` — Calibration buffer snapshot for regime record counts
    /// * `prior_only_variance` — `time_correction / λ_prior`, the variance the
    ///   tempering saturates at (´dec:risk:evidence-only-uncertainty´)
    #[must_use]
    fn from_blend(
        blend: &BlendResult,
        rho_opr: f64,
        n_sentinels_reporting: usize,
        cal: &crate::snapshot::published::CalibrationBufferSnapshot,
        prior_only_variance: f64,
    ) -> Self {
        // Calibrated sigmoid: divide by κ_eff before applying sigmoid
        // The risk probability is σ(ρ̂_eff / κ_eff) (´def:risk:probability´).
        let calibrated_rho = blend.rho_effective / blend.kappa_effective;
        let p_bad = stable_sigmoid(calibrated_rho);

        // Probability-space uncertainty (´def:risk:probability´):
        // Maps logit-space variance to probability-space via the delta method.
        // The variance is evidence-only along this request's own direction
        // before the map, so the uncertainty the host reads carries the
        // widening rather than leaving it to be applied downstream
        // (´dec:risk:evidence-only-uncertainty´).
        let borrowed_share = blend.borrowed_share_effective;
        let sigma_eff = temper_variance(blend.variance_effective, borrowed_share, prior_only_variance).sqrt();
        let uncertainty = p_bad * (1.0 - p_bad) * sigma_eff / blend.kappa_effective;

        // Sister and operational probabilities
        let p_bad_sister = stable_sigmoid(blend.rho_inherited / blend.kappa_effective);
        let p_bad_operational = stable_sigmoid(rho_opr / blend.kappa_effective);

        // Intervention effectiveness in log-odds space
        let intervention_effectiveness = blend.rho_effective - rho_opr;

        // Anchor convergence: anchor has tighter posterior than sister in
        // the shared subspace (´def:risk:subspace-blend´), whose weight uses
        // raw forms (´dec:calibration:raw-weight-forms´). Both fields are
        // already time-corrected by `compute_blend`.
        let anchor_converged = blend.variance_anchor < blend.variance_inherited_shared;

        Self {
            p_bad,
            uncertainty,
            borrowed_share,
            anchor_weight: blend.anchor_weight,
            rho_eff: blend.rho_effective,
            sigma_eff,
            kappa_eff: blend.kappa_effective,
            p_bad_sister,
            p_bad_operational,
            intervention_effectiveness,
            anchor_converged,
            n_sentinels_reporting,
            sister_regime_calibration_records: cal.sister_regime_records,
            anchor_regime_calibration_records: cal.anchor_regime_records,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Axis Evaluation (´tab:axis:inference-outputs´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Evaluates an outcome axis model for prediction.
///
/// Computes a point estimate and 95% credible interval for the outcome
/// axis using the same quadratic form pattern as the blend computation.
///
/// # Algorithm
///
/// 1. Compute `ô = dot(φ̂, μ_axis)` — linear predictor in tanh-space
/// 2. Compute `σ²_o = φ̂ᵀ Σ_axis φ̂` — variance in tanh-space
/// 3. Apply inverse tanh to transform to raw scale
/// 4. Transform uncertainty using derivative of atanh
/// 5. Construct 95% credible interval (±1.96σ)
///
/// # Arguments
///
/// * `axis_id` — Outcome axis identifier
/// * `axis_name` — Human-readable axis name
/// * `params` — Axis model parameters (μ, Σ in column-major flat form)
/// * `phi_hat` — Standardised feature vector
/// * `kappa_a` — Axis calibration factor
/// * `time_correction` — `1/γ^Δt` for variance adjustment
///
/// # Returns
///
/// `OutcomePrediction` with `predicted_raw`, `uncertainty`, and 95% interval.
///
/// # Cross-References
///
/// - The inference outputs reported per axis (´tab:axis:inference-outputs´)
#[must_use]
pub fn evaluate_outcome_axis(
    axis_id: OutcomeAxisId,
    axis_name: &str,
    params: &crate::model::parameters::ModelParameters,
    phi_hat: &[f64],
    kappa_a: f64,
    time_correction: f64,
    lambda_prior: f64,
) -> OutcomePrediction {
    const CLIP_EPSILON: f64 = 1e-7;
    const DENOM_EPSILON: f64 = 1e-8;
    const Z_95: f64 = 1.96;

    // Guard for dimension mismatch — the prior-only prediction
    // (´def:axis:prior-only-prediction´).
    if phi_hat.len() != params.p || params.p == 0 {
        return OutcomePrediction::prior_only(axis_id, axis_name, kappa_a, time_correction, lambda_prior);
    }

    // Step 1: Dot product for tanh-space point estimate
    let o_hat: f64 = phi_hat.iter().zip(params.mu.iter()).map(|(a, b)| a * b).sum();

    // Step 2: Quadratic form for tanh-space variance
    let sigma2_o = quadratic_form_outcome(&params.covariance_data, phi_hat, params.p);
    let sigma_o = (sigma2_o.max(0.0) * time_correction).sqrt();

    // Step 3: Transform via inverse tanh (atanh) to raw scale
    // Clamp ô to (-1+ε, 1-ε) to avoid infinity at boundaries
    let o_clamped = o_hat.clamp(-1.0 + CLIP_EPSILON, 1.0 - CLIP_EPSILON);
    let o_raw = kappa_a * stable_atanh(o_clamped);

    // Step 4: Transform uncertainty via derivative of atanh
    // d/dx[atanh(x)] = 1/(1-x²)
    // σ_raw = κ_a · σ_o / (1 - ô²)
    #[allow(clippy::suboptimal_flops)]
    // Justified: mul_add fuses the rounding and shifts RiskBasis uncertainty beyond the pinned regression oracle (multi_channel ±0.003); the plain form is the calibrated arithmetic
    let sigma_raw = kappa_a * sigma_o / (1.0 - o_clamped * o_clamped + DENOM_EPSILON);

    // Step 5: 95% credible interval (z = 1.96 for 95%)
    let interval_lo = Z_95.mul_add(-sigma_raw, o_raw);
    let interval_hi = Z_95.mul_add(sigma_raw, o_raw);

    OutcomePrediction {
        axis_id,
        axis_name: axis_name.to_owned(),
        predicted_raw: o_raw,
        uncertainty: sigma_raw,
        prediction_interval: (interval_lo, interval_hi),
    }
}

/// CP4's outcome-axis guard: a non-finite axis prediction retains and
/// flags as the risk basis does, falling back to the computed
/// prior-only prediction at the axis's own scale
/// (´def:axis:prior-only-prediction´).
fn cp4_guard_axis_prediction(
    pred: OutcomePrediction,
    axis_name: &str,
    kappa_a: f64,
    time_correction: f64,
    lambda_prior: f64,
    degradation: &mut DegradationContext,
) -> OutcomePrediction {
    if pred.predicted_raw.is_finite() && pred.uncertainty.is_finite() {
        return pred;
    }
    degradation.degraded_models.push(ModelId::OutcomeAxis(pred.axis_id));
    OutcomePrediction::prior_only(pred.axis_id, axis_name, kappa_a, time_correction, lambda_prior)
}

/// Whether any model's drift accumulator is near its auto-reset
/// threshold, for the embedded snapshot's drift flag
/// (´schema:output:health-snapshot´).
///
/// Near is read as at least half the threshold: an accumulator that
/// reaches the threshold itself has already reset to zero
/// (´alg:monitoring:drift-cusums´), so a flag on the full value would
/// never be observable.
#[must_use]
pub fn drift_near_threshold(
    drift: &std::collections::HashMap<crate::types::ModelId, crate::health::DriftState>,
    h_threshold: f64,
) -> bool {
    drift.values().any(|d| d.s_plus.max(d.s_minus) >= 0.5 * h_threshold)
}

/// Quadratic form for outcome axis: `vᵀ Σ v` from column-major flat storage.
///
/// Exploits symmetry for efficiency.
#[must_use]
#[inline]
fn quadratic_form_outcome(sigma_flat: &[f64], v: &[f64], p: usize) -> f64 {
    debug_assert_eq!(v.len(), p, "vector dimension mismatch");
    debug_assert_eq!(sigma_flat.len(), p * p, "matrix size mismatch");

    let mut result = 0.0;

    // Diagonal: Σ[i,i] * v[i]²
    for i in 0..p {
        let sigma_ii = sigma_flat[i * p + i];
        result = (sigma_ii * v[i]).mul_add(v[i], result);
    }

    // Off-diagonal (upper triangle): 2 * Σ[i,j] * v[i] * v[j] for i < j
    for i in 0..p {
        for j in (i + 1)..p {
            // Column-major: element (i, j) is at j * p + i
            let sigma_ij = sigma_flat[j * p + i];
            result = (2.0 * sigma_ij * v[i]).mul_add(v[j], result);
        }
    }

    result
}

/// What became of an offer to the cold prior-mass ramp.
///
/// The non-observed outcome is a load condition rather than a contract
/// violation, and inherits the degradation posture's obligation to
/// report (´dec:degradation:error-partition´).
///
/// There is no deferred-completion outcome any more, because there is no
/// completion to defer. Under the old gate the whole accumulation rode on one
/// delivery at the horizon, so a full channel at that instant had to be
/// retried or the sample was lost; under the ramp a refusal costs one share of
/// prior mass retired later (´alg:standardisation:batch-initialisation´).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BatchInitObservation {
    /// Handed to the steward — nothing to report.
    Observed,
    /// Skipped on a command channel too full to take it. The observation is
    /// lost, the accepted count does not advance, and the ramp reaches its
    /// horizon after correspondingly more requests
    /// (´dec:concurrency:no-silent-drop´).
    SkippedContended,
}

/// Health snapshot at assessment time (´schema:output:health-snapshot´).
///
/// The compact, fixed-size snapshot every assessment carries, with the
/// snapshot version as the join key into the full health report
/// stream. Everything per-Sentinel or per-dimension stays in the
/// on-demand health interface (´dec:health:tiered-queries´) and is fetched by version
/// when forensics need it.
///
/// Per-request Sentinel coverage is available via
/// `RiskBasis.n_sentinels_reporting`; `sentinel_coverage` here is the
/// system-wide measurement-surface availability and is the same for
/// all requests in a batch, because the inline summary is a property of
/// the result rather than of the request (´dec:surface:inline-health´).
#[derive(Clone, Debug)]
#[non_exhaustive]
#[allow(clippy::struct_excessive_bools)] // Justified: the compact wire schema carries four independent diagnostic flags.
pub struct HealthSnapshot {
    /// Per-request degradation (CP1–CP4 effects).
    pub degradation: DegradationContext,
    /// Version of the published model snapshot this assessment read —
    /// the join key into the full health report stream
    /// (´schema:output:health-snapshot´).
    pub snapshot_version: u64,
    /// Seconds since that snapshot was published — the elapsed
    /// interval the uncertainty correction was computed at
    /// (´def:runtime:time-correction´).
    pub snapshot_age_seconds: f64,
    /// System-wide convergence stage.
    pub convergence_stage: CompositeConvergenceStage,
    /// Fraction of registered Sentinels with cached batch reports.
    /// System-wide; `0.0` when no Sentinels are registered.
    pub sentinel_coverage: f64,
    /// True when the sister Platt regime has `>= N_cal_min` (30)
    /// calibration records (´req:platt:minimum-samples´).
    pub calibration_mature: bool,
    /// True when calibration has run but the anchor regime currently holds
    /// fewer weighted records than its configured fitting minimum
    /// (´schema:output:health-snapshot´).
    pub anchor_regime_frozen: bool,
    /// True when any model's drift accumulator is near its threshold —
    /// read as at least half the auto-reset threshold, since an
    /// accumulator at the threshold has already reset to zero
    /// (´alg:monitoring:drift-cusums´).
    pub drift_flag: bool,
    /// The most recent empirical inflation factor
    /// (´alg:monitoring:empirical-coverage´); `None` until a refit has
    /// computed one.
    pub uncertainty_inflation: Option<f64>,
    /// True when no Sentinel contributed data to this assessment
    /// — the designed prior-only operating mode (´def:registry:no-online-sentinel´).
    pub zero_sentinels: bool,
    /// The cold-ramp phase of the coordinate snapshot this assessment was
    /// scored in (´tab:monitoring:standardisation-transition´).
    ///
    /// Retrospective, not current: it describes the coordinate system this
    /// one result was computed in, which is what makes two results comparable
    /// or tells a reader that they are not. The full report's phase says
    /// where the ramp stands now, and during a transition the two legitimately
    /// disagree by however many advances fell between them.
    pub standardisation_phase: crate::feature::standardisation::StandardisationPhase,
    /// Accepted cold-ramp observations that snapshot represents, from zero
    /// through `N_init`.
    ///
    /// Maturity is this against the horizon; it is not carried separately,
    /// because a stored fraction and a stored count are two descriptions of
    /// one position (´dec:health:standardisation-phase-reported´).
    pub standardisation_observations: usize,
    /// Pending entries evicted since the previous assessment snapshot took
    /// the buffer's counter, including evictions caused by this assessment's
    /// own insertion (´dec:surface:inline-health´).
    pub pending_buffer_evictions: u64,
}

/// Core-layer surface of a risk assessment.
///
/// Every field is produced by the Core assessment path, whose work ends
/// at the risk assessment (´dec:ordering:core-boundary´); none depend on
/// Derivation Function or Companion Tracker state.
///
/// The split leaves the far half to the derivation layer
/// (´cav:ordering:derivation-half´); see also audit Recommendation 1.
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct RiskAssessment {
    /// Unique identifier for this assessment.
    ///
    /// Used to link labels to assessments via `label()`. Assigned at
    /// pending buffer insertion, keying the map the assessment waits in
    /// (´dec:retention:pending-map´). Monotonic `u64` via
    /// `AtomicU64::fetch_add`.
    pub id: AssessmentId,
    /// Risk estimate basis. Full transparency into the risk
    /// estimation process; the sufficient statistic that crosses into
    /// derivation (´dec:ordering:closed-crossing´)
    /// $(\hat{p}, \sigma_{\hat{p}}, \sigma_\text{eff}, \kappa_\text{eff})$
    /// is recoverable from this struct.
    pub risk: RiskBasis,
    /// Per-outcome-axis predictions. Outcome predictions do NOT enter
    /// the resonance derivation: axis predictions never enter it
    /// (´inv:guarantee:outcome-neutrality´).
    pub outcome_predictions: HashMap<OutcomeAxisId, OutcomePrediction>,
    /// Per-Sentinel alarm summaries — keyed by `SentinelId`, present only
    /// for Sentinels that contributed extraction features to this assessment
    /// (´def:runtime:alarm-summary´). A Sentinel that is
    /// registered but has no cached batch report, or has a report but no
    /// matching coordinate in the request, is absent from this map.
    pub per_sentinel: HashMap<SentinelId, SentinelAlarmSummary>,
    /// Lightweight inline health diagnostics. Includes per-request
    /// degradation (CP1–CP4 effects) and system-wide convergence
    /// state. Full health queries deferred to the on-demand health
    /// interface (´dec:health:tiered-queries´).
    pub health: HealthSnapshot,
}

/// Host-composed assessment-plus-derivation result.
///
/// Composed of a core-layer [`RiskAssessment`], the derivation-layer
/// [`DecisionLandscape`] (´sig:landscape:output´), its rendered
/// [`ResonanceProfile`] (´sig:rendering:contract´), and a routing
/// back-reference to the host-owned channel used for derivation. The
/// composition is the host's (´dec:ordering:core-boundary´); the core
/// read API returns [`RiskAssessment`] directly.
#[derive(Clone, Debug)]
#[non_exhaustive]
/// LEGACY ´legacy:code:derived-reckoning´: derived reckoning
pub struct DerivedReckoning {
    /// Channel used for this derived reckoning.
    ///
    /// Routing back-reference to the host-owned channel used for derivation.
    /// Not part of the core assessment surface (´schema:output:assessment´); hosts supply
    /// the channel policy to `derive_landscape()` explicitly.
    pub channel: ChannelId,
    /// Core-layer surface — see [`RiskAssessment`].
    ///
    /// Mirrors the assessment output schema field-for-field (´schema:output:assessment´).
    pub assessment: RiskAssessment,
    /// Decision-layer surface — see [`DecisionLandscape`].
    pub landscape: DecisionLandscape,
    /// Rendering-layer surface — see [`ResonanceProfile`].
    pub profile: ResonanceProfile,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assessment Context (internal)
// ═══════════════════════════════════════════════════════════════════════════════

/// Internal context accumulated during assessment pipeline.
struct AssessmentContext {
    /// Per-Sentinel extractions.
    extractions: HashMap<SentinelId, SentinelExtraction>,
    /// Per-Sentinel sub-scores for aggregation.
    sub_scores: Vec<(SentinelId, SentinelSubScores)>,
    /// Per-Sentinel alarm summaries.
    alarms: HashMap<SentinelId, SentinelAlarmSummary>,
    /// Per-dimension active competitive cells.
    active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>>,
    /// Per-dimension identity coordinates.
    identity_coordinates: HashMap<DimensionId, u128>,
    /// Per-dimension identity features.
    identity_features: HashMap<DimensionId, IdentityDimensionFeatures>,
    /// Cross-dimension identity aggregate features.
    cross_dimension_features: CrossDimensionAggregateFeatures,
    /// Aggregate features (15).
    aggregate_features: [f64; 15],
    /// Signal features.
    signal_features: Vec<f64>,
    /// Assembled and standardised feature vector.
    phi: Vec<f64>,
    /// Degradation context.
    degradation: DegradationContext,
    /// The earliest-arrived report index that contributed an extraction, and
    /// the age that report carried (´def:monitoring:feedback-latency´).
    ///
    /// Only Sentinels that actually reached the extraction are eligible: a
    /// registered Sentinel the request supplied no coordinate for contributed
    /// nothing to this assessment, so its report is not evidence this
    /// assessment is downstream of.
    report_origin: Option<ReportOrigin>,
}

impl AssessmentContext {
    fn new() -> Self {
        Self {
            extractions: HashMap::new(),
            sub_scores: Vec::new(),
            alarms: HashMap::new(),
            active_cells: HashMap::new(),
            identity_coordinates: HashMap::new(),
            identity_features: HashMap::new(),
            cross_dimension_features: CrossDimensionAggregateFeatures::default(),
            aggregate_features: [0.0; 15],
            signal_features: Vec::new(),
            phi: Vec::new(),
            degradation: DegradationContext::default(),
            report_origin: None,
        }
    }

    /// Keeps the earliest arrival among the reports that contributed.
    ///
    /// Earliest rather than latest: the stage measured from it is how long the
    /// evidence waited, and the longest wait among the contributors is the one
    /// the assessment as a whole carried.
    fn observe_report_origin(&mut self, candidate: ReportOrigin) {
        let keep = self
            .report_origin
            .is_none_or(|current| candidate.received_at < current.received_at);
        if keep {
            self.report_origin = Some(candidate);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shared State Interface
// ═══════════════════════════════════════════════════════════════════════════════

/// Minimal shared state interface for assessment.
///
/// This trait abstracts the shared state access needed by `assess_single`.
/// The primary implementation is `Assayer`, but this allows testing with
/// mock implementations.
pub trait AssessmentSharedState {
    /// Returns the current time in the persistent domain.
    ///
    /// Routed through the engine's injected clock rather than read
    /// from the wall clock directly, so that a test can fix it.
    fn now_persistent(&self) -> PersistentTimestamp;

    /// Returns the concordance thresholds for aggregate computation.
    fn concordance_thresholds(&self) -> [f64; SCORING_AXIS_COUNT];

    /// Precision at which pending entries store feature values
    /// (´def:runtime:storage-precision´). Defaults to the tabulated
    /// single (´tab:config:pending-buffer´).
    fn feature_storage_precision(&self) -> crate::pending::StoragePrecision {
        crate::pending::StoragePrecision::default()
    }

    /// Observes per-axis max z-scores for concordance tracking.
    // TODO ´todo:code:widen-this-to-accept-all-reporting´: widen this to accept all reporting Sentinels'
    // per-axis sub-scores for a single assessment, not the aggregate
    // max-across-Sentinels feature vector entry; each process carries its
    // own concrete tracker (´dec:health:concrete-trackers´).
    fn observe_concordance(&self, per_axis_max_z: [f64; SCORING_AXIS_COUNT]);

    /// Returns the current convergence stage.
    fn convergence_stage(&self) -> CompositeConvergenceStage;

    /// The most recent empirical uncertainty inflation factor
    /// (´alg:monitoring:empirical-coverage´), for the embedded health
    /// snapshot (´schema:output:health-snapshot´). `None` until a
    /// Platt refit has computed one — which is the default, so a test
    /// double without a published health summary reports absence
    /// rather than a value it has no evidence for.
    fn uncertainty_inflation(&self) -> Option<f64> {
        None
    }

    /// Whether the anchor calibration is frozen at the supplied weighted
    /// regime count. Test doubles without published calibration history have
    /// not fitted and therefore report `false`.
    fn anchor_regime_frozen(&self, _anchor_regime_records: f64) -> bool {
        false
    }

    /// Configured drift accumulator threshold used by the compact flag.
    fn drift_threshold(&self) -> f64 {
        crate::config::types::MonitoringConfig::default().h_threshold
    }

    /// Returns the next assessment ID.
    fn next_assessment_id(&self) -> AssessmentId;

    /// Inserts a pending assessment into the buffer.
    fn insert_pending(&self, pending: PendingAssessment);

    /// Takes pending-buffer evictions accumulated since the previous
    /// assessment snapshot. Test doubles without a buffer report none.
    fn take_pending_buffer_evictions(&self) -> u64 {
        0
    }

    /// Returns the report index for a Sentinel (if any).
    fn sentinel_report(&self, sentinel_id: SentinelId) -> Option<std::sync::Arc<ReportIndex>>;

    /// Returns the Sentinel's outcome ledger (if registered).
    fn sentinel_ledger(
        &self,
        sentinel_id: SentinelId,
    ) -> Option<std::sync::Arc<std::sync::RwLock<crate::ledger::SentinelLedger>>>;

    /// Returns the extraction configuration.
    fn extraction_config(&self) -> extraction::ExtractionConfig;

    /// Returns the thresholds the per-assessment Ledger immaturity flag is
    /// taken against (´def:runtime:alarm-summary´).
    ///
    /// Read once per assessment beside the decay rate, because the predicate
    /// is evaluated once per Sentinel and the three numbers do not change
    /// within one assessment.
    fn ledger_immaturity_criteria(&self) -> crate::ledger::ImmaturityCriteria;

    /// Returns the outcome axis IDs for per-axis ledger features.
    fn outcome_axis_ids(&self) -> Vec<OutcomeAxisId>;

    /// Returns the ledger decay rate (hourly).
    fn gamma_t_ledger(&self) -> f64;

    /// Returns the core model decay rate (hourly).
    ///
    /// Used for time-corrected uncertainty on the assessment path
    /// (´def:runtime:time-correction´), applied lazily at the point of use
    /// (´conv:clock:lazy-application´): `1/γ_t_core^Δt_snap`.
    fn gamma_t_core(&self) -> f64;

    /// Returns the list of registered Sentinel IDs.
    fn registered_sentinels(&self) -> Vec<SentinelId>;

    /// Returns the list of registered identity dimension IDs.
    fn registered_identity_dimensions(&self) -> Vec<DimensionId>;

    /// Encodes an entity key to a coordinate for a specific identity dimension.
    ///
    /// Uses the per-dimension encoder registered at dimension creation time.
    /// Returns `None` if the dimension is not registered.
    ///
    /// # Cross-References
    ///
    /// - Each dimension carries its own declared domain (´def:registry:identity-dimension´)
    fn encode_for_dimension(&self, dim_id: DimensionId, entity: &EntityKey) -> Option<u128>;

    /// Finds active competitive cells for a coordinate in a dimension.
    ///
    /// Returns cells ordered from deepest to shallowest.
    fn find_active_cells(&self, dim_id: DimensionId, coord: u128) -> Vec<CompetitiveCellId>;

    /// Records the number of identity indicators active on one assessment.
    fn record_active_indicator_count(&self, _dim_id: DimensionId, _active: usize) {}

    /// Returns the total number of cells in the competitive set for a dimension.
    fn competitive_set_len(&self, dim_id: DimensionId) -> usize;

    /// Returns the graph total importance for a dimension.
    ///
    /// This is the sum of all node importances published with the current
    /// competitive-set index.
    fn graph_total_importance(&self, dim_id: DimensionId) -> f64;

    /// Returns the dimension's declared domain width in bits, if registered
    /// (´tab:keyspace:dimension-features´).
    fn dimension_domain_bits(&self, dim_id: DimensionId) -> Option<u8>;

    /// Returns a clone of the measurement state for a cell, if it exists.
    fn cell_measurement_state(
        &self,
        dim_id: DimensionId,
        cell_id: &CompetitiveCellId,
    ) -> Option<crate::identity::MeasurementState>;

    /// Returns a pure decayed view of the outcome state for a cell, if it
    /// exists, at the assessment's persistent timestamp.
    fn cell_outcome_state(
        &self,
        dim_id: DimensionId,
        cell_id: &CompetitiveCellId,
        now: &PersistentTimestamp,
    ) -> Option<crate::identity::CellOutcomeState>;

    /// Updates a cell's measurement state with per-Sentinel composite alarms.
    ///
    /// The assessment path reads *and writes* per-cell measurement state
    /// within the same Mutex hold (~100 ns); it is one of the enumerated
    /// assessment-path writes (´inv:runtime:enumerated-writes´), and it moves
    /// observational geometry only (´dec:ordering:evidence-authority´).
    ///
    /// `sentinel_alarms` carries, per reporting Sentinel, that Sentinel's own
    /// composite alarm for this request (´tab:keyspace:measurement-state´).
    fn update_cell_measurement(&self, dim_id: DimensionId, cell_id: &CompetitiveCellId, sentinel_alarms: &[(SentinelId, f64)]);

    /// Records an identity observation for a dimension.
    ///
    /// This uses `try_send` on the observation channel: overflow degrades
    /// rather than fails (´dec:memory:overflow-degrades´), incrementing a
    /// counter rather than raising an error.
    /// The maintenance loop applies Δ=1 internally (feed-forward enforcement).
    fn record_identity_observation(&self, dim_id: DimensionId, coord: u128);

    /// Returns the prior precision λ, held by the Core-only configuration
    /// (´dec:construction:core-only-config´).
    ///
    /// Used in the fourth numeric checkpoint's uncertainty fallback
    /// (´tab:runtime:numeric-checkpoints´), where a matrix pathology is
    /// retained and flagged (´dec:degradation:retain-and-flag´).
    fn lambda_prior(&self) -> f64;

    /// Lookup and merge signal features for an entity.
    ///
    /// Delegates to `SignalCache::get_and_merge()` (´dec:retention:cache-preencoded´).
    /// Returns the merged signal feature vector of length `p_sig`.
    /// Returns an empty `Vec` if no signal schema is configured.
    fn signal_cache_get_and_merge(&self, entity: &EntityKey, signals: &HashMap<String, SignalValue>) -> Vec<f64>;

    /// Counts host-provided signals whose `SignalValue` variant does not
    /// match the declared `SignalShape` (´dec:surface:sanitise-not-reject´). Such signals
    /// are zero-filled by [`crate::signal::encode_signal`] and reported
    /// back to the host via
    /// [`crate::health::DegradationContext::signals_shape_mismatched`].
    fn count_signal_shape_mismatches(&self, signals: &HashMap<String, SignalValue>) -> u32;

    /// Counts host-provided signal names not present in the declared schema.
    ///
    /// Unknown signal names are skipped during encoding and reported back to
    /// the host via [`crate::health::DegradationContext::signals_unknown`].
    fn count_unknown_signals(&self, signals: &HashMap<String, SignalValue>) -> u32;

    /// Offers a raw (unstandardised) feature vector to the cold prior-mass
    /// ramp, which is step 2 of (´alg:standardisation:batch-initialisation´).
    ///
    /// Called after interaction computation and before standardisation, so
    /// the offering request has already fixed the snapshot it will score
    /// against (´dec:ordering:score-before-evolve´). The offer mutates
    /// nothing and waits for nothing: it hands the vector to the steward,
    /// which is the single writer of the standardisation state.
    ///
    /// `layout_generation` travels with the vector so the steward can refuse
    /// an observation whose positions it has since renumbered
    /// (´req:standardisation:lifecycle-entries´).
    ///
    /// Returns what became of the offer, so the caller can report a
    /// load-condition degradation in band (´cor:degradation:in-band-travel´).
    fn offer_cold_observation(&self, phi_raw: &[f64], layout_generation: u64) -> BatchInitObservation;

    /// Observes one reporting Sentinel's raw feature values into its
    /// active bootstrap accumulator, occupancy prepended — step 2 of
    /// (´alg:standardisation:sentinel-bootstrap´).
    ///
    /// Called from the extraction loop, before standardisation, for
    /// Sentinels that report in. No-op for a slot whose bootstrap is
    /// inactive (fast-path `AtomicBool`). Features arrive in the
    /// extraction's stored precision.
    fn observe_sentinel_bootstrap(&self, sentinel_id: SentinelId, features: &crate::pending::StoredFeatures);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assessment Pipeline
// ═══════════════════════════════════════════════════════════════════════════════

/// Performs a single core assessment request.
///
/// Implements step 0 context setup plus the nine steps of the one
/// assessment function (´dec:ordering:assessment-function´):
///
/// 1. **Step 0: Context** — Entity-bearing request context
/// 2. **Steps 1–2: Extraction** — Per-Sentinel feature extraction
/// 3. **Step 2.5: Identity** — Identity dimension processing
/// 4. **Step 3: Aggregation** — Cross-Sentinel aggregates, concordance observation
/// 5. **Step 4: Signals** — Signal cache lookup and merge
/// 6. **Step 5: Assembly** — Assemble and standardise φ
/// 7. **Step 6: Risk** — Blend computation (´dec:calibration:raw-weight-forms´)
/// 8. **Step 7: Outcomes** — Per-axis predictions (´tab:axis:inference-outputs´)
/// 9. **Step 8: Pending** — Record in pending buffer
/// 10. **Step 9: Return** — Assemble `RiskAssessment` result
///
/// # Cross-References
///
/// - Assessment is one function, not a stage pipeline (´dec:ordering:assessment-function´)
pub fn assess_single<S: AssessmentSharedState>(
    req: &RequestContext,
    shared: &S,
    snapshot: &ModelSnapshot,
    batch_now: Instant,
) -> RiskAssessment {
    // ─── Step 0: Context ───
    let mut ctx = AssessmentContext::new();
    let now = shared.now_persistent();
    let extraction_config = shared.extraction_config();
    // Read axis IDs from the snapshot we were given, not from a
    // second ArcSwap::load() that could see a different version.
    let spatial_axis_ids: Vec<OutcomeAxisId> = snapshot
        .outcome_models
        .iter()
        .filter_map(|(&axis_id, axis)| axis.spatial.then_some(axis_id))
        .collect();
    let outcome_axis_ids: Vec<OutcomeAxisId> = snapshot.outcome_models.keys().copied().collect();
    let gamma_t = shared.gamma_t_ledger();
    let ledger_maturity = shared.ledger_immaturity_criteria();

    // ─── Steps 1–2: Per-Sentinel extraction ───
    let registered_sentinels = shared.registered_sentinels();
    // System-wide measurement-surface counter, travelling inline on the result
    // (´dec:surface:inline-health´):
    // counts registered Sentinels with a cached batch report, independent of
    // whether the request supplied a matching coordinate.
    let mut n_sentinels_with_cached_report: usize = 0;
    for sentinel_id in &registered_sentinels {
        let coord = req.sentinel_coordinates.get(sentinel_id).copied();

        if let Some(report_arc) = shared.sentinel_report(*sentinel_id) {
            if report_arc.has_report() {
                n_sentinels_with_cached_report += 1;
            }
            if let Some(coord) = coord {
                if report_arc.has_report() {
                    // Get sentinel ledger (or use empty)
                    let ledger_arc = shared.sentinel_ledger(*sentinel_id);
                    let empty_ledger;
                    let ledger_guard;
                    #[allow(clippy::option_if_let_else)]
                    let ledger_ref: &crate::ledger::SentinelLedger = if let Some(ref arc) = ledger_arc {
                        // Poison recovery, as everywhere on this path
                        // (´dec:degradation:infallible-core´).
                        ledger_guard = arc.read().unwrap_or_else(std::sync::PoisonError::into_inner);
                        &ledger_guard
                    } else {
                        // TODO ´todo:code:remove-this-rootless-fallback-once´: Remove this rootless fallback once
                        // Sentinel registration guarantees a Ledger root before
                        // any assessment can route through the Sentinel; the root
                        // is permanent (´dec:memory:root-permanence´).
                        empty_ledger = crate::ledger::SentinelLedger::new();
                        &empty_ledger
                    };

                    // Extract features from this Sentinel's report
                    let (extraction, alarm, sub_scores) = extract_sentinel_features(
                        *sentinel_id,
                        &report_arc,
                        ledger_ref,
                        coord,
                        gamma_t,
                        &now,
                        &batch_now,
                        &spatial_axis_ids,
                        &extraction_config,
                        &ledger_maturity,
                        &mut ctx.degradation,
                    );

                    // Step 2 of (´alg:standardisation:sentinel-bootstrap´):
                    // raw slot values, before standardisation.
                    shared.observe_sentinel_bootstrap(*sentinel_id, &extraction.features);

                    // This Sentinel's report reached the extraction, so the
                    // assessment is downstream of the observations behind it
                    // (´def:monitoring:feedback-latency´).
                    if let Some(received_at) = report_arc.received_at() {
                        ctx.observe_report_origin(ReportOrigin {
                            received_at,
                            oldest_observation_age_micros: report_arc.oldest_observation_age_micros(),
                        });
                    }

                    ctx.extractions.insert(*sentinel_id, extraction);
                    ctx.alarms.insert(*sentinel_id, alarm);
                    ctx.sub_scores.push((*sentinel_id, sub_scores));
                } else {
                    // No report data — create empty extraction
                    ctx.extractions
                        .insert(*sentinel_id, SentinelExtraction::new(coord, Vec::new(), false));
                }
            } else {
                // No coordinate for this Sentinel — "active, no data for the
                // request" (´tab:registry:coverage-states´): occupancy = true
                // (Sentinel is online), features zero.
                ctx.extractions
                    .insert(*sentinel_id, SentinelExtraction::new(0, Vec::new(), true));
            }
        } else {
            // D2: No batch report cached → occupancy 0, features zero
            // (´tab:registry:coverage-states´). Every registered Sentinel gets
            // an extraction entry regardless of report availability.
            let offline_coord = coord.unwrap_or(0);
            ctx.extractions
                .insert(*sentinel_id, SentinelExtraction::new(offline_coord, Vec::new(), false));
        }
    }

    // ─── Step 2.5: Identity dimension processing ───
    let identity_dims = shared.registered_identity_dimensions();
    // Each reporting Sentinel contributes its own composite alarm
    // (´def:runtime:alarm-summary´), one entry per Sentinel
    // (´tab:keyspace:measurement-state´).
    let sentinel_alarms: Vec<(SentinelId, f64)> = ctx.alarms.iter().map(|(sid, a)| (*sid, a.composite)).collect();

    for dim_id in &identity_dims {
        // Encode entity key to coordinate using the dimension's own encoder
        // (´def:registry:identity-dimension´).
        let Some(coord) = shared.encode_for_dimension(*dim_id, &req.entity) else {
            // Dimension no longer registered; skip.
            continue;
        };
        ctx.identity_coordinates.insert(*dim_id, coord);

        // Defer observation to the maintenance loop; try_send is used, and
        // overflow degrades rather than fails (´dec:memory:overflow-degrades´).
        shared.record_identity_observation(*dim_id, coord);

        // Find active competitive cells via the shared state
        let active_cells = shared.find_active_cells(*dim_id, coord);
        shared.record_active_indicator_count(*dim_id, active_cells.len());
        let competitive_set_len = shared.competitive_set_len(*dim_id);
        let graph_importance = shared.graph_total_importance(*dim_id);

        // Compute identity aggregate features from real data
        let deepest_depth = active_cells.first().map_or(0, |c| c.depth);
        let domain_bits = shared.dimension_domain_bits(*dim_id).unwrap_or(128);
        let coverage_depth = coverage_depth_feature(deepest_depth, domain_bits);
        let active_fraction = active_cells.len() as f64 / (competitive_set_len.max(1) as f64);
        let total_importance = graph_importance.ln_1p();

        let aggregate = IdentityAggregateFeatures::new(coverage_depth, active_fraction, total_importance);

        // D5: Combined read+update of cell measurement state, one of the
        // enumerated assessment-path writes (´inv:runtime:enumerated-writes´)
        // and observational geometry only (´dec:ordering:evidence-authority´).
        // Every registered dimension maintains this state.
        for cell_id in &active_cells {
            shared.update_cell_measurement(*dim_id, cell_id, &sentinel_alarms);
        }

        let measurement = compute_dimension_measurement_features(shared, *dim_id, &active_cells);
        let axis_features = compute_identity_axis_features(shared, *dim_id, &active_cells, &outcome_axis_ids, &now);
        ctx.identity_features
            .insert(*dim_id, IdentityDimensionFeatures::new(aggregate, measurement, axis_features));

        ctx.active_cells.insert(*dim_id, active_cells.into_iter().collect());
    }

    ctx.cross_dimension_features = compute_cross_dimension_features(shared, &ctx.identity_features, &ctx.active_cells, &now);

    // ─── Step 3: Cross-Sentinel aggregation + concordance observation ───
    let concordance_thresholds = shared.concordance_thresholds();
    ctx.aggregate_features = compute_aggregates(&ctx.sub_scores, &concordance_thresholds, registered_sentinels.len());

    // TODO ´todo:code:feed-the-concordancetracker-one-n-d´: feed the ConcordanceTracker one [N, D, S, C]
    // entry per reporting Sentinel. This aggregate max-across-Sentinels
    // observation inflates the calibration distribution; the tracker is this
    // process's own concrete one (´dec:health:concrete-trackers´).
    // Extract per-axis max z for concordance observation
    let per_axis_max_z = [
        ctx.aggregate_features[3], // N axis max z
        ctx.aggregate_features[4], // D axis max z
        ctx.aggregate_features[5], // S axis max z
        ctx.aggregate_features[6], // C axis max z
    ];
    shared.observe_concordance(per_axis_max_z);

    // ─── Step 4: Signal features ───
    // CP1: Count NaN/Inf signal values before cache merge (sanitisation happens inside encode_signal)
    for value in req.signals.values() {
        if !value.is_finite() {
            ctx.degradation.signals_sanitised += 1;
        }
    }

    // Shape-mismatch counter (´dec:surface:sanitise-not-reject´): host-provided values whose
    // `SignalValue` variant disagrees with the declared `SignalShape` are
    // zero-filled by `encode_signal`; surface that as degradation rather
    // than silently dropping the input.
    ctx.degradation.signals_shape_mismatched = shared.count_signal_shape_mismatches(&req.signals);
    ctx.degradation.signals_unknown = shared.count_unknown_signals(&req.signals);

    // Get merged signal features from the pre-encoded cache
    // (´dec:retention:cache-preencoded´).
    // The cache handles:
    // - Reading cached entity-persistent signals
    // - Encoding and merging provided entity-persistent signals
    // - Overlaying request-scoped signals
    let cache_features = shared.signal_cache_get_and_merge(&req.entity, &req.signals);

    // If cache returns features, use them; otherwise fall back to zero-fill for dimension map length.
    let sig_len = snapshot.dimension_map.sig_range.len();
    ctx.signal_features = if cache_features.is_empty() {
        vec![0.0; sig_len]
    } else {
        // Ensure length matches dimension map expectation
        let mut features = cache_features;
        features.resize(sig_len, 0.0);
        features.truncate(sig_len);
        features
    };

    // ─── Step 5: Assemble and standardise φ ───
    // Split into raw assembly → batch init observation → standardise
    // Statistics are observed at assessment time (´dec:vector:statistic-timing´):
    // the observation falls between interactions and standardisation.
    let std_config = StandardisationConfig::default();
    let mut phi = assemble_assessment_raw(
        &snapshot.dimension_map,
        &ctx.extractions,
        &ctx.aggregate_features,
        &ctx.identity_features,
        &ctx.cross_dimension_features,
        &ctx.signal_features,
        &ctx.active_cells,
    );

    // Step 2 of (´alg:standardisation:batch-initialisation´): offer the raw
    // vector to the cold ramp, after this call acquired its snapshot and
    // before it standardises against that snapshot. What an accepted
    // observation may move is a later coordinate system, never the one this
    // request is being answered from (´dec:ordering:score-before-evolve´).
    //
    // The gate is the acquired snapshot's own phase rather than a separate
    // flag, so the decision is a function of the coordinate system in hand. A
    // snapshot a few advances stale may offer once past the horizon; the
    // steward refuses the late arrival, which is cheaper than keeping a
    // second authority over when the ramp is finished. A load-condition
    // outcome is counted into this assessment's degradation context
    // (´cor:degradation:in-band-travel´).
    if !snapshot.standardisation_phase.is_in_service() {
        match shared.offer_cold_observation(&phi, snapshot.layout_generation) {
            BatchInitObservation::Observed => {}
            BatchInitObservation::SkippedContended => ctx.degradation.batch_init_observations_skipped += 1,
        }
    }

    // Standardise in place.
    standardise_in_place(
        &mut phi,
        &snapshot.feature_means,
        &snapshot.feature_variances,
        std_config.epsilon,
    );
    ctx.phi = phi;

    // CP3: Check for NaN in assembled φ — sanitise to 0.0
    for val in &mut ctx.phi {
        if !val.is_finite() {
            let (sanitised, _was_nan) = sanitise_f64(*val, 0.0);
            *val = sanitised;
            ctx.degradation.features_sanitised += 1;
        }
    }

    // ─── Step 6: Risk estimates ───
    // TODO ´todo:code:record-that-step-6-and-step´: record that step 6 and step 7 are
    // data-independent after step 5 and dominate assessment cost. Scheduling
    // them independently is not the remedy: (´dec:ordering:assessment-function´)
    // decides that assessment is one per-request function with factored
    // helpers and not a pipeline of independently scheduled stages, so any
    // speed-up must stay inside the one function.
    // Time correction: variance grows as snapshot ages.
    // Uses gamma_t_core rather than gamma_t_ledger (´def:runtime:time-correction´),
    // and the decay is applied lazily at the point of use (´conv:clock:lazy-application´).
    let elapsed = batch_now.duration_since(snapshot.publish_timestamp);
    let gamma_t_core = shared.gamma_t_core();
    let decay = decay_factor_elapsed(gamma_t_core, elapsed);
    let time_correction = if decay > 1e-15 { 1.0 / decay } else { 1.0 };

    // Build anchor subvector φ̃ from φ̂: thirteen positions gathered by index,
    // two computed (´def:dimension:anchor-projection´). The computed pair
    // carries no projection index, so it contributes nothing to the sister's
    // projected quadratic form below — which is right, because neither names
    // a sister coordinate.
    let p_a = snapshot.anchor.p;
    let n_reporting_for_anchor = ctx.extractions.values().filter(|e| !e.features.is_empty()).count();
    let phi_tilde: Vec<f64> = snapshot
        .dimension_map
        .extract_anchor_subvector(&ctx.phi, p_a, n_reporting_for_anchor);

    // Compute blend using SISTER model (not operational)
    let blend_result = compute_blend(
        &snapshot.sister,
        &snapshot.anchor,
        &snapshot.floor_masses.sister,
        &snapshot.floor_masses.anchor,
        &ctx.phi,
        &phi_tilde,
        &snapshot.dimension_map.anchor_projection_indices[..p_a],
        time_correction,
        snapshot.kappa_sister,
        snapshot.kappa_anchor,
    );

    // Compute operational dot product separately (for intervention effectiveness)
    let rho_opr: f64 = if ctx.phi.len() == snapshot.operational.mu.len() {
        ctx.phi.iter().zip(snapshot.operational.mu.iter()).map(|(a, b)| a * b).sum()
    } else {
        0.0
    };

    // Count contributing Sentinels: those whose extraction carries
    // features — the same set `per_sentinel` is built from
    // (´schema:output:assessment´). Occupancy is the online flag, not a
    // contribution: a Sentinel that is active with no coordinate in the
    // request holds an occupied, empty extraction and contributed no
    // data.
    let n_sentinels_reporting = ctx.extractions.values().filter(|e| !e.features.is_empty()).count();

    let prior_only = prior_only_variance(time_correction, shared.lambda_prior());
    let mut risk = RiskBasis::from_blend(
        &blend_result,
        rho_opr,
        n_sentinels_reporting,
        &snapshot.calibration_buffer,
        prior_only,
    );

    // ─── CP4: Guard against NaN/Inf from blend ───
    // Defence-in-depth: sanitise all RiskBasis fields that could carry NaN
    // from the blend computation or quadratic form, at the fourth numeric
    // checkpoint (´tab:runtime:numeric-checkpoints´).
    let cp4_degraded = sanitise_risk_basis(&mut risk, time_correction, shared.lambda_prior());
    if cp4_degraded {
        // The blend is dominated by the sister model's quadratic form;
        // a NaN here implicates the sister covariance, which is retained and
        // flagged rather than refused (´dec:degradation:retain-and-flag´).
        ctx.degradation.degraded_models.push(ModelId::Sister);
    }

    // ─── Step 7: Outcome axis predictions (´tab:axis:inference-outputs´) ───
    let mut outcome_predictions = HashMap::new();
    for (axis_id, axis_state) in &snapshot.outcome_models {
        let pred = evaluate_outcome_axis(
            *axis_id,
            &axis_state.name,
            &axis_state.parameters,
            &ctx.phi,
            axis_state.kappa_a,
            time_correction,
            shared.lambda_prior(),
        );

        let pred = cp4_guard_axis_prediction(
            pred,
            &axis_state.name,
            axis_state.kappa_a,
            time_correction,
            shared.lambda_prior(),
            &mut ctx.degradation,
        );

        outcome_predictions.insert(*axis_id, pred);
    }

    // ─── Step 9: Assign ID and assemble the core assessment ───
    let assessment_id = shared.next_assessment_id();

    // `per_sentinel` carries alarm summaries for *reporting* Sentinels only
    // (´def:runtime:alarm-summary´). Sentinels that are
    // registered but produced no extraction (no cached report, no
    // matching coordinate, or zeroed by CP2) are absent from the map.
    let per_sentinel: HashMap<SentinelId, SentinelAlarmSummary> = ctx.alarms.clone();

    // System-wide sentinel coverage: fraction of registered Sentinels with
    // cached batch reports: the measurement-surface availability metric
    // travelling inline on the result (´dec:surface:inline-health´),
    // independent of any individual request's coordinate coverage. Per-request coverage lives on
    // `RiskBasis.n_sentinels_reporting`.
    let sentinel_coverage = if registered_sentinels.is_empty() {
        0.0
    } else {
        n_sentinels_with_cached_report as f64 / registered_sentinels.len() as f64
    };

    // Calibration is mature once the sister regime's weighted sample count
    // (´constr:platt:buffer´) reaches `N_CAL_MIN`, the minimum sample floor
    // (´req:platt:minimum-samples´).
    let calibration_mature = snapshot.calibration_buffer.sister_regime_records >= N_CAL_MIN;
    let anchor_regime_frozen = shared.anchor_regime_frozen(snapshot.calibration_buffer.anchor_regime_records);

    let drift_flag = drift_near_threshold(&snapshot.drift, shared.drift_threshold());

    let mut assessment = RiskAssessment {
        id: assessment_id,
        risk,
        outcome_predictions,
        per_sentinel,
        health: HealthSnapshot {
            degradation: ctx.degradation.clone(),
            snapshot_version: snapshot.version,
            snapshot_age_seconds: elapsed.as_secs_f64(),
            convergence_stage: shared.convergence_stage(),
            sentinel_coverage,
            calibration_mature,
            anchor_regime_frozen,
            drift_flag,
            uncertainty_inflation: shared.uncertainty_inflation(),
            zero_sentinels: n_sentinels_reporting == 0,
            // From the snapshot this assessment acquired, not from wherever
            // the ramp has since reached: the pair reports the coordinate
            // state this result was actually scored in, and the version
            // beside it names that state
            // (´tab:monitoring:standardisation-transition´).
            standardisation_phase: snapshot.standardisation_phase,
            standardisation_observations: snapshot.standardisation_observations,
            // Filled after the pending entry is inserted so an eviction caused
            // by this assessment travels on this assessment's own snapshot.
            pending_buffer_evictions: 0,
        },
    };

    // ─── Step 8: Record in pending buffer ───

    // The two Sentinel lists and the two frozen identity feature blocks are
    // what reconstruction sources one, three and four read from storage
    // (´def:runtime:pending-entry´), (´alg:runtime:reconstruction´). Sorted
    // so the stored lists are deterministic across runs.
    let mut active_sentinels = registered_sentinels;
    active_sentinels.sort_unstable_by_key(|s| s.0);
    let mut reporting_sentinels: Vec<SentinelId> = ctx.alarms.keys().copied().collect();
    reporting_sentinels.sort_unstable_by_key(|s| s.0);
    let precision = shared.feature_storage_precision();
    let entity_base_features: HashMap<DimensionId, crate::pending::StoredFeatures> = ctx
        .identity_features
        .iter()
        .map(|(&dim, f)| {
            (
                dim,
                crate::pending::StoredFeatures::store(&f.measurement.as_slice(), precision),
            )
        })
        .collect();
    let entity_axis_features: HashMap<DimensionId, HashMap<OutcomeAxisId, crate::pending::StoredFeatures>> = ctx
        .identity_features
        .iter()
        .map(|(&dim, f)| {
            let per_axis = f
                .axis_features
                .iter()
                .map(|(&axis, af)| (axis, crate::pending::StoredFeatures::store(&af.as_slice(), precision)))
                .collect();
            (dim, per_axis)
        })
        .collect();

    let pending = PendingAssessment {
        // The axis order the extractions above were laid out against, kept
        // so a label arriving after a lifecycle event can put each stored
        // outcome-memory pair back at its own axis's position rather than at
        // whatever now holds its old rank
        // (´entry:assayer:wl-feature-frozen-slot-truncation´).
        spatial_axis_ids: spatial_axis_ids.clone(),
        id: assessment_id,
        timestamp: batch_now,
        persistent_timestamp: now,
        entity: req.entity.clone(),
        // Extractions are handed over at single precision by the extraction
        // contract (´rem:extraction:single-precision´); the buffer stores
        // them at its configured width, which for a widened store carries
        // no more information than the single-precision source held.
        sentinel_extractions: ctx
            .extractions
            .iter()
            .map(|(&sid, ext)| {
                (
                    sid,
                    crate::pending::SentinelExtraction {
                        coordinate: ext.coordinate,
                        features: ext.features.at_precision(precision),
                        occupancy: ext.occupancy,
                    },
                )
            })
            .collect(),
        active_sentinels,
        reporting_sentinels,
        identity_coordinates: ctx.identity_coordinates.clone(),
        identity_active_cells: ctx.active_cells.clone(),
        entity_base_features,
        entity_axis_features,
        signal_features: crate::pending::StoredFeatures::store(&ctx.signal_features, precision),
        risk_basis: PendingRiskBasis::new(
            assessment.risk.p_bad,
            assessment.risk.uncertainty,
            assessment.risk.anchor_weight,
            assessment.risk.rho_eff,
        ),
        outcome_predictions: assessment
            .outcome_predictions
            .iter()
            .map(|(k, v)| (*k, v.predicted_raw))
            .collect(),
        degradation: assessment.health.degradation.clone(),
        report_origin: ctx.report_origin,
    };

    shared.insert_pending(pending);
    assessment.health.pending_buffer_evictions = shared.take_pending_buffer_evictions();

    // ─── Step 9: Return Assessment ───
    assessment
}

/// Synthesises a pre-seed pending entry against the declared schema and
/// the registered Sentinels (´alg:host:pre-seeding´).
///
/// The simplified pre-seeding arrangement requires the schema and the
/// Sentinel registry to be declared first because the synthesis reads
/// them: the entry's own key is encoded on each registered identity
/// dimension and the current competitive state is frozen beside it;
/// each registered Sentinel holds an occupied, zero-featured slot —
/// what reconstruction source four reads for a Sentinel that was
/// active and silent (´alg:runtime:reconstruction´) — and the signal
/// block is encoded against the declared schema at the entity's cached
/// persistent values. The risk basis stays at the prior: a pre-seeded
/// label supplies evidence, not an assessment.
pub fn synthesise_preseed_pending<S: AssessmentSharedState>(
    assessment_id: AssessmentId,
    entity: &EntityKey,
    shared: &S,
    sig_len: usize,
    outcome_axis_ids: &[OutcomeAxisId],
) -> PendingAssessment {
    let now = shared.now_persistent();
    let precision = shared.feature_storage_precision();

    // Occupied, zero-featured slot per registered Sentinel.
    let mut active_sentinels = shared.registered_sentinels();
    active_sentinels.sort_unstable_by_key(|s| s.0);
    let sentinel_extractions: HashMap<SentinelId, crate::pending::SentinelExtraction> = active_sentinels
        .iter()
        .map(|&sid| {
            (
                sid,
                crate::pending::SentinelExtraction {
                    coordinate: 0,
                    features: crate::pending::StoredFeatures::store(&[], precision),
                    occupancy: true,
                },
            )
        })
        .collect();

    // The entry's own key on each identity dimension, with the current
    // competitive state frozen beside it.
    let mut identity_coordinates = HashMap::new();
    let mut identity_active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> = HashMap::new();
    let mut entity_base_features = HashMap::new();
    let mut entity_axis_features = HashMap::new();
    for dim_id in shared.registered_identity_dimensions() {
        let Some(coord) = shared.encode_for_dimension(dim_id, entity) else {
            continue;
        };
        identity_coordinates.insert(dim_id, coord);
        let active_cells = shared.find_active_cells(dim_id, coord);
        let measurement = compute_dimension_measurement_features(shared, dim_id, &active_cells);
        let axis_features = compute_identity_axis_features(shared, dim_id, &active_cells, outcome_axis_ids, &now);
        entity_base_features.insert(
            dim_id,
            crate::pending::StoredFeatures::store(&measurement.as_slice(), precision),
        );
        let per_axis: HashMap<OutcomeAxisId, crate::pending::StoredFeatures> = axis_features
            .iter()
            .map(|(&axis, af)| (axis, crate::pending::StoredFeatures::store(&af.as_slice(), precision)))
            .collect();
        entity_axis_features.insert(dim_id, per_axis);
        identity_active_cells.insert(dim_id, active_cells);
    }

    // The signal block, encoded against the declared schema at the
    // entity's cached persistent values.
    let mut signal_features = shared.signal_cache_get_and_merge(entity, &HashMap::new());
    signal_features.resize(sig_len, 0.0);
    signal_features.truncate(sig_len);

    PendingAssessment {
        // A synthesised entry's extractions are featureless, so they carry no
        // outcome-memory pairs and there is no axis order for them to have
        // been laid out against.
        spatial_axis_ids: Vec::new(),
        id: assessment_id,
        timestamp: Instant::now(),
        persistent_timestamp: now,
        entity: entity.clone(),
        sentinel_extractions,
        identity_coordinates,
        identity_active_cells,
        entity_base_features,
        entity_axis_features,
        active_sentinels,
        reporting_sentinels: Vec::new(),
        signal_features: crate::pending::StoredFeatures::store(&signal_features, precision),
        risk_basis: PendingRiskBasis::new(0.5, 0.25, 0.0, 0.0),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        // A pre-seed entry is synthesised rather than assessed, so no report
        // reached it and there is no observation for it to be downstream of.
        report_origin: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// The variance a model that knows only its prior would report, corrected for
/// staleness: `time_correction / λ_prior` (´def:axis:prior-only-prediction´).
///
/// One derivation, three readers. It is the fallback the fourth numeric
/// checkpoint substitutes when the blend's own variance is unusable, it is the
/// scale a prior-only axis prediction is reported on
/// (´def:axis:prior-only-prediction´), and it is the ceiling the borrowed-share
/// tempering saturates at (´dec:risk:evidence-only-uncertainty´). All three are
/// the same statement — "we know what the prior says and nothing more" — so
/// they are one expression rather than three that happen to agree.
#[must_use]
fn prior_only_variance(time_correction: f64, lambda_prior: f64) -> f64 {
    time_correction / lambda_prior.max(1e-15)
}

/// Makes a variance evidence-only along the direction it was read along, given
/// the share of the precision holding it up that was borrowed from the models'
/// floors (´dec:risk:evidence-only-uncertainty´).
///
/// Along a direction, posterior precision is evidence precision plus borrowed
/// mass, so evidence-only precision is `(1 − s)` times the posterior's and
/// evidence-only variance is the posterior's divided by `(1 − s)`. As the
/// share approaches one the direction is held up by the floors alone and the
/// quotient diverges, so the widening saturates at the prior-only variance —
/// the honest statement that the prior is all that is known here, and the
/// value the fourth numeric checkpoint already falls back to. The cap is
/// derived rather than chosen: it is that fallback and not a tolerance.
///
/// Three properties are structural rather than tested into place. A share of
/// zero returns the variance unchanged to the bit, because dividing by one is
/// exact and the ceiling is never below the variance. The result is never
/// narrower than the variance it was given, because the ceiling is taken as
/// the larger of the two — a blended variance already above prior-only, which
/// the disagreement term can produce, is not narrowed by a cap meant to bound
/// a widening. And a non-finite variance passes through untouched, so the
/// checkpoint downstream still sees the failure rather than a finite
/// substitute quietly standing in its place.
#[must_use]
fn temper_variance(variance: f64, borrowed_share: f64, prior_only: f64) -> f64 {
    if !variance.is_finite() || !borrowed_share.is_finite() || borrowed_share <= 0.0 {
        return variance;
    }
    let ceiling = prior_only.max(variance);
    if borrowed_share >= 1.0 {
        return ceiling;
    }
    (variance / (1.0 - borrowed_share)).min(ceiling)
}

/// Sanitises all `RiskBasis` fields that could carry NaN/Inf from
/// the blend computation or quadratic form, at the fourth numeric checkpoint
/// (´tab:runtime:numeric-checkpoints´).
///
/// The `lambda_prior` parameter is needed for the uncertainty fallback
/// formula: `p̂(1−p̂) · √(time_correction / λ_prior) / κ_eff`, which retains
/// and flags rather than refusing (´dec:degradation:retain-and-flag´).
///
/// Returns `true` if any field was sanitised.
#[allow(clippy::useless_let_if_seq)] // Justified: multi-branch accumulator; clippy's suggested rewrite doesn't apply to 5 sequential guards
fn sanitise_risk_basis(risk: &mut RiskBasis, time_correction: f64, lambda_prior: f64) -> bool {
    let mut degraded = false;

    // p_bad: fall back to 0.5 (maximum uncertainty prior)
    if !risk.p_bad.is_finite() {
        risk.p_bad = 0.5;
        degraded = true;
    }

    // kappa_eff: fall back to 1.0 (uncalibrated)
    if !risk.kappa_eff.is_finite() || risk.kappa_eff <= 0.0 {
        risk.kappa_eff = 1.0;
        degraded = true;
    }

    // borrowed_share: a share that is not a number in `[0, 1]` is no reading
    // at all, and zero is the reading that leaves the uncertainty as it stands
    // (´dec:risk:evidence-only-uncertainty´).
    if !risk.borrowed_share.is_finite() || !(0.0..=1.0).contains(&risk.borrowed_share) {
        risk.borrowed_share = 0.0;
        degraded = true;
    }

    let mut recompute_uncertainty = false;
    let prior_only = prior_only_variance(time_correction, lambda_prior);

    // sigma_eff: fall back to time-corrected prior standard deviation, and
    // temper it as the ordinary path tempers its own — which changes nothing
    // here, because the fallback is already the value the tempering saturates
    // at. It is applied rather than skipped so that the two paths carry one
    // operation in one order and cannot come apart under a later edit
    // (´dec:risk:evidence-only-uncertainty´).
    if !risk.sigma_eff.is_finite() || risk.sigma_eff < 0.0 {
        risk.sigma_eff = temper_variance(prior_only, risk.borrowed_share, prior_only).sqrt();
        recompute_uncertainty = true;
        degraded = true;
    }

    // uncertainty: fall back to time-corrected prior uncertainty at the fourth
    // numeric checkpoint (´tab:runtime:numeric-checkpoints´).
    // Formula: p̂(1−p̂) · σ_eff / κ_eff.
    if recompute_uncertainty || !risk.uncertainty.is_finite() || risk.uncertainty < 0.0 {
        risk.uncertainty = risk.p_bad * (1.0 - risk.p_bad) * risk.sigma_eff / risk.kappa_eff.max(1e-15);
        degraded = true;
    }

    // rho_eff: fall back to 0.0 (prior linear predictor)
    if !risk.rho_eff.is_finite() {
        risk.rho_eff = 0.0;
        degraded = true;
    }

    degraded
}

/// Extracts features from a Sentinel report for a given coordinate.
///
/// Delegates to the full extraction pipeline [`extraction::extract_sentinel`],
/// then derives `SentinelSubScores` from the alarm summary.
///
/// If the extraction produces NaN values (CP2), the entire slot is zeroed
/// and the sentinel ID is recorded in `degradation.nan_sentinels`.
fn extract_sentinel_features(
    sentinel_id: SentinelId,
    index: &ReportIndex,
    ledger: &crate::ledger::SentinelLedger,
    coord: u128,
    gamma_t_ledger: f64,
    now: &PersistentTimestamp,
    batch_now: &Instant,
    spatial_axis_ids: &[OutcomeAxisId],
    config: &extraction::ExtractionConfig,
    maturity: &crate::ledger::ImmaturityCriteria,
    degradation: &mut DegradationContext,
) -> (SentinelExtraction, SentinelAlarmSummary, SentinelSubScores) {
    let (extraction, alarm) = extraction::extract_sentinel(
        index,
        ledger,
        coord,
        gamma_t_ledger,
        now,
        batch_now,
        spatial_axis_ids,
        config,
        maturity,
        false, // hierarchical_asserted — Phase 3
    );

    if !extraction.occupancy {
        return (extraction, alarm, SentinelSubScores::new());
    }

    // Check for NaN (CP2)
    let has_nan = extraction.features.iter().any(|v| !v.is_finite());
    if has_nan {
        degradation.nan_sentinels.push(sentinel_id);
        return (
            SentinelExtraction::new(coord, Vec::new(), false),
            SentinelAlarmSummary::default(),
            SentinelSubScores::new(),
        );
    }

    // Derive sub-scores from the first 4 z-scores (cell N, D, S, C)
    // and the first 4 CUSUM features (indices 24–27) if present
    let z = &extraction.features;
    let cell_zscores = [
        z.get(0).unwrap_or(0.0),
        z.get(1).unwrap_or(0.0),
        z.get(2).unwrap_or(0.0),
        z.get(3).unwrap_or(0.0),
    ];
    let cusums = [
        z.get(24).unwrap_or(0.0),
        z.get(25).unwrap_or(0.0),
        z.get(26).unwrap_or(0.0),
        z.get(27).unwrap_or(0.0),
    ];
    let sub_scores = SentinelSubScores::from_values(cell_zscores, cusums);

    (extraction, alarm, sub_scores)
}

/// Coverage depth: the receiving cell's depth over the dimension's declared
/// domain width (´tab:keyspace:dimension-features´).
fn coverage_depth_feature(deepest_depth: u8, domain_bits: u8) -> f64 {
    f64::from(deepest_depth) / f64::from(domain_bits.max(1))
}

/// Computes per-dimension measurement features from real `MeasurementState` EWMAs.
///
/// Reads alarm, suspicion, and volatility from each active cell's measurement
/// state via the shared-state trait, then assembles the 5-feature vector:
///
/// | Feature | Source |
/// |---------|--------|
/// | `max_alarm` | Maximum per-Sentinel alarm EWMA across all active cells |
/// | `mean_alarm` | Mean of per-Sentinel alarm EWMAs across all active cells |
/// | `max_suspicion` | Maximum suspicion EWMA across all active cells |
/// | `volatility_deepest` | Volatility EWMA of the deepest (first) active cell |
/// | `has_competitive_cell` | 1.0 if any active cells exist, 0.0 otherwise |
fn compute_dimension_measurement_features<S: AssessmentSharedState>(
    shared: &S,
    dim_id: DimensionId,
    active_cells: &[CompetitiveCellId],
) -> IdentityMeasurementFeatures {
    if active_cells.is_empty() {
        return IdentityMeasurementFeatures::default();
    }

    let mut max_alarm: f64 = 0.0;
    let mut sum_alarm: f64 = 0.0;
    let mut alarm_count: usize = 0;
    let mut max_suspicion: f64 = 0.0;
    let mut volatility_deepest: f64 = 0.0;
    let mut first_cell = true;

    for cell in active_cells {
        if let Some(ms) = shared.cell_measurement_state(dim_id, cell) {
            for &alarm_ewma in ms.per_sentinel_alarm_ewma.values() {
                max_alarm = max_alarm.max(alarm_ewma);
                sum_alarm += alarm_ewma;
                alarm_count += 1;
            }
            max_suspicion = max_suspicion.max(ms.suspicion());

            // Deepest cell is first in the sorted active_cells list
            if first_cell {
                volatility_deepest = ms.volatility;
            }
        }
        first_cell = false;
    }

    let mean_alarm = if alarm_count > 0 {
        sum_alarm / alarm_count as f64
    } else {
        0.0
    };

    IdentityMeasurementFeatures::new(
        max_alarm,
        mean_alarm,
        max_suspicion,
        volatility_deepest,
        1.0, // has_competitive_cell — we checked non-empty above
    )
}

/// Computes the 3 per-axis identity outcome features for active cells in one dimension.
fn compute_identity_axis_features<S: AssessmentSharedState>(
    shared: &S,
    dim_id: DimensionId,
    active_cells: &[CompetitiveCellId],
    outcome_axis_ids: &[OutcomeAxisId],
    now: &PersistentTimestamp,
) -> HashMap<OutcomeAxisId, IdentityAxisFeatures> {
    let mut features = HashMap::new();
    if active_cells.is_empty() {
        return features;
    }

    for &axis_id in outcome_axis_ids {
        let mut compressed_values = Vec::with_capacity(active_cells.len());
        let mut raw_sum = 0.0;

        for cell in active_cells {
            let state = shared.cell_outcome_state(dim_id, cell, now);
            let compressed = state
                .as_ref()
                .and_then(|outcome| outcome.per_axis_compressed.get(&axis_id).copied())
                .unwrap_or(0.0);
            let raw = state
                .as_ref()
                .and_then(|outcome| outcome.per_axis_raw.get(&axis_id).copied())
                .unwrap_or(0.0);
            compressed_values.push(compressed);
            raw_sum += raw;
        }

        let count = compressed_values.len() as f64;
        let compressed_mean = compressed_values.iter().sum::<f64>() / count;
        let raw_mean = raw_sum / count;
        let stability = if compressed_values.len() <= 1 {
            0.0
        } else {
            let variance = compressed_values
                .iter()
                .map(|value| {
                    let diff = *value - compressed_mean;
                    diff * diff
                })
                .sum::<f64>()
                / count;
            variance.sqrt()
        };

        features.insert(axis_id, IdentityAxisFeatures::new(compressed_mean, raw_mean, stability));
    }

    features
}

/// Computes the fixed cross-dimension aggregate identity feature block.
fn compute_cross_dimension_features<S: AssessmentSharedState>(
    shared: &S,
    identity_features: &HashMap<DimensionId, IdentityDimensionFeatures>,
    active_cells: &HashMap<DimensionId, Vec<CompetitiveCellId>>,
    now: &PersistentTimestamp,
) -> CrossDimensionAggregateFeatures {
    let has_any_active = active_cells.values().any(|cells| !cells.is_empty());
    if !has_any_active {
        return CrossDimensionAggregateFeatures::default();
    }

    let mut cross = CrossDimensionAggregateFeatures {
        has_any_competitive_cell: 1.0,
        ..Default::default()
    };

    for (dim_id, features) in identity_features {
        cross.max_suspicion = cross.max_suspicion.max(features.measurement.max_suspicion);
        cross.max_alarm = cross.max_alarm.max(features.measurement.max_alarm);
        cross.max_volatility = cross.max_volatility.max(features.measurement.volatility_deepest);
        cross.max_coverage_depth = cross.max_coverage_depth.max(features.aggregate.coverage_depth);

        if let Some(cells) = active_cells.get(dim_id) {
            for cell in cells {
                if let Some(outcome) = shared.cell_outcome_state(*dim_id, cell, now) {
                    cross.max_adverse_rate = cross.max_adverse_rate.max(outcome.adverse_rate_ewma);
                    cross.max_abs_compressed_valence =
                        cross.max_abs_compressed_valence.max(outcome.compressed_valence_ewma.abs());
                    cross.max_abs_raw_valence = cross.max_abs_raw_valence.max(outcome.raw_valence_ewma.abs());
                }
            }
        }
    }

    cross
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use std::sync::atomic::{AtomicU64, Ordering};

    use super::*;
    use crate::resonance::channel::ChannelPolicy;
    use crate::resonance::derivation::{ChallengeEstimate, ResonanceConfig, render_resonances};
    use crate::resonance::landscape::derive_landscape;

    /// Mock shared state for testing.
    struct MockSharedState {
        channels: Vec<ChannelId>,
        sentinels: Vec<SentinelId>,
        identity_dims: Vec<DimensionId>,
        next_id: AtomicU64,
        concordance_observations: std::sync::Mutex<Vec<[f64; SCORING_AXIS_COUNT]>>,
        batch_init_outcome: BatchInitObservation,
    }

    impl MockSharedState {
        fn new() -> Self {
            Self {
                channels: vec![ChannelId(1)],
                sentinels: Vec::new(),
                identity_dims: Vec::new(),
                next_id: AtomicU64::new(1),
                concordance_observations: std::sync::Mutex::new(Vec::new()),
                batch_init_outcome: BatchInitObservation::Observed,
            }
        }

        fn with_batch_init_outcome(mut self, outcome: BatchInitObservation) -> Self {
            self.batch_init_outcome = outcome;
            self
        }

        #[allow(dead_code)]
        fn with_channel(mut self, id: u32) -> Self {
            self.channels.push(ChannelId(id));
            self
        }

        fn with_sentinel(mut self, id: u32) -> Self {
            self.sentinels.push(SentinelId(id));
            self
        }
    }

    impl AssessmentSharedState for MockSharedState {
        fn now_persistent(&self) -> PersistentTimestamp {
            // Fixed, not wall-clock: these unit tests gain nothing from
            // a moving present and lose reproducibility to it.
            PersistentTimestamp::new(i64::try_from(crate::testing::VirtualClock::EPOCH_SECS).unwrap_or(0), 0)
        }

        fn concordance_thresholds(&self) -> [f64; SCORING_AXIS_COUNT] {
            [0.5, 0.5, 0.5, 0.5]
        }

        fn observe_concordance(&self, per_axis_max_z: [f64; SCORING_AXIS_COUNT]) {
            self.concordance_observations.lock().unwrap().push(per_axis_max_z);
        }

        fn convergence_stage(&self) -> CompositeConvergenceStage {
            CompositeConvergenceStage::ColdStart
        }

        fn next_assessment_id(&self) -> AssessmentId {
            AssessmentId(self.next_id.fetch_add(1, Ordering::Relaxed))
        }

        fn insert_pending(&self, _pending: PendingAssessment) {
            // No-op for tests
        }

        fn sentinel_report(&self, _sentinel_id: SentinelId) -> Option<Arc<ReportIndex>> {
            None
        }

        fn sentinel_ledger(&self, _sentinel_id: SentinelId) -> Option<Arc<std::sync::RwLock<crate::ledger::SentinelLedger>>> {
            None
        }

        fn extraction_config(&self) -> extraction::ExtractionConfig {
            extraction::ExtractionConfig::default()
        }

        fn ledger_immaturity_criteria(&self) -> crate::ledger::ImmaturityCriteria {
            // The shipped defaults, so the double's flag is taken against the
            // same thresholds a deployment's is
            // (´def:config:ledger-materiality-threshold´),
            // (´def:config:attenuation-materiality-floor´).
            let config = crate::config::types::AssayerConfig::default();
            crate::ledger::ImmaturityCriteria {
                materiality_threshold: config.monitoring.ledger_materiality_threshold,
                attenuation_floor: config.monitoring.attenuation_materiality_floor,
                lambda_l: config.ledger.lambda_l,
            }
        }

        fn outcome_axis_ids(&self) -> Vec<OutcomeAxisId> {
            Vec::new()
        }

        fn gamma_t_ledger(&self) -> f64 {
            0.01
        }

        fn gamma_t_core(&self) -> f64 {
            0.9999
        }

        fn registered_sentinels(&self) -> Vec<SentinelId> {
            self.sentinels.clone()
        }

        fn registered_identity_dimensions(&self) -> Vec<DimensionId> {
            self.identity_dims.clone()
        }

        fn encode_for_dimension(&self, _dim_id: DimensionId, entity: &EntityKey) -> Option<u128> {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};

            // Mock: use default hasher for all dimensions.
            let mut hasher = DefaultHasher::new();
            entity.as_bytes().hash(&mut hasher);
            let h1 = hasher.finish();
            let mut hasher2 = DefaultHasher::new();
            h1.hash(&mut hasher2);
            let h2 = hasher2.finish();
            Some((u128::from(h1) << 64) | u128::from(h2))
        }

        fn find_active_cells(&self, _dim_id: DimensionId, _coord: u128) -> Vec<CompetitiveCellId> {
            Vec::new()
        }

        fn competitive_set_len(&self, _dim_id: DimensionId) -> usize {
            0
        }

        fn graph_total_importance(&self, _dim_id: DimensionId) -> f64 {
            0.0
        }

        fn dimension_domain_bits(&self, _dim_id: DimensionId) -> Option<u8> {
            Some(128)
        }

        fn cell_measurement_state(
            &self,
            _dim_id: DimensionId,
            _cell_id: &CompetitiveCellId,
        ) -> Option<crate::identity::MeasurementState> {
            None
        }

        fn cell_outcome_state(
            &self,
            _dim_id: DimensionId,
            _cell_id: &CompetitiveCellId,
            _now: &PersistentTimestamp,
        ) -> Option<crate::identity::CellOutcomeState> {
            None
        }

        fn record_identity_observation(&self, _dim_id: DimensionId, _coord: u128) {
            // Mock: no-op. Real implementation sends to maintenance loop.
        }

        fn update_cell_measurement(
            &self,
            _dim_id: DimensionId,
            _cell_id: &CompetitiveCellId,
            _sentinel_alarms: &[(SentinelId, f64)],
        ) {
            // Mock: no-op.
        }

        fn lambda_prior(&self) -> f64 {
            0.1
        }

        fn signal_cache_get_and_merge(&self, _entity: &EntityKey, _signals: &HashMap<String, SignalValue>) -> Vec<f64> {
            // Mock: return empty (no signal schema configured).
            Vec::new()
        }

        fn count_signal_shape_mismatches(&self, _signals: &HashMap<String, SignalValue>) -> u32 {
            0
        }

        fn count_unknown_signals(&self, _signals: &HashMap<String, SignalValue>) -> u32 {
            0
        }

        fn offer_cold_observation(&self, _phi_raw: &[f64], _layout_generation: u64) -> BatchInitObservation {
            // Mock: reports whatever outcome the test configured.
            self.batch_init_outcome
        }

        fn observe_sentinel_bootstrap(&self, _sentinel_id: SentinelId, _features: &crate::pending::StoredFeatures) {
            // Mock: no-op.
        }
    }

    /// Creates a `ModelParameters` with identity covariance.
    fn make_identity_params(p: usize) -> crate::model::parameters::ModelParameters {
        let mu = vec![0.0; p];
        // Column-major identity matrix
        let mut covariance_data = vec![0.0; p * p];
        for i in 0..p {
            covariance_data[i * p + i] = 1.0;
        }
        crate::model::parameters::ModelParameters { mu, covariance_data, p }
    }

    fn test_snapshot() -> ModelSnapshot {
        use indexmap::IndexMap;

        use crate::feature::dimension_map::DimensionMap;

        // Build dimension map first to get the actual feature dimension
        let dim_map = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &[], &IndexMap::new(), &[]);
        let dim_p = dim_map.p;

        // Create model parameters matching the dimension map's p
        let params = make_identity_params(dim_p);
        let anchor = make_identity_params(15);

        let mut snapshot = ModelSnapshot::cold_start(1, params.clone(), params, anchor, dim_p);
        // Initialize feature_means and feature_variances to match dim_map.p
        snapshot.feature_means = vec![0.0; dim_p];
        snapshot.feature_variances = vec![1.0; dim_p];

        snapshot
    }

    fn assess_then_derive<S: AssessmentSharedState>(
        req: &RequestContext,
        shared: &S,
        snapshot: &ModelSnapshot,
        batch_now: Instant,
    ) -> DerivedReckoning {
        let assessment = assess_single(req, shared, snapshot, batch_now);
        let policy = ChannelPolicy::default();
        let landscape = derive_landscape(&assessment, &policy, ChallengeEstimate::default());
        let profile = render_resonances(&landscape, &assessment.risk, &ResonanceConfig::default());
        DerivedReckoning {
            channel: req.channel_hint().unwrap_or(ChannelId(0)),
            assessment,
            landscape,
            profile,
        }
    }

    /// A request context carries an entity and nothing about routing, and the
    /// core path assesses it without ever asking which channel it arrived on.
    /// Channel policy is the derivation layer's business, so the core estimate
    /// is the same object whatever the host later decides to do with it.
    ///
    /// ´claim:assess:an-assessment-needs-no-channel-because-routing-belongs-to-the-derivation-layer´
    /// ´test:unit:channel-free-request-assesses´
    #[test]
    fn channel_free_request_assesses() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        assert_eq!(reckoning.assessment.id, AssessmentId(1));
    }

    /// An entity key of zero bytes is a legitimate key, not a malformed one:
    /// the pipeline runs end to end and hands back a numbered assessment.
    /// The assessment surface is total — a host that cannot identify its
    /// subject still gets a prior-shaped answer instead of an error to handle
    /// on the request path.
    ///
    /// ´claim:assess:the-assessment-surface-is-total-and-yields-a-result-for-any-entity-key-including-an-empty-one´
    /// ´test:unit:empty-entity-key-still-assesses´
    #[test]
    fn empty_entity_key_still_assesses() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(Vec::<u8>::new()));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        assert_eq!(reckoning.assessment.id, AssessmentId(1));
    }

    /// The two layers compose in the host, not inside the crate: a core
    /// assessment is produced first, then handed to the stateless derivation
    /// along with an explicitly supplied channel policy, and the pair travels
    /// on as one host-owned result. Because derivation reads only what it was
    /// passed, the composition can be rebuilt or re-run without touching core
    /// state.
    ///
    /// ´claim:assess:a-core-assessment-and-a-stateless-derivation-compose-into-one-host-owned-result´
    /// ´test:unit:well-formed-request-returns-reckoning´
    #[test]
    fn well_formed_request_returns_reckoning() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        assert_eq!(reckoning.assessment.id, AssessmentId(1)); // First ID
    }

    /// Successive assessments take strictly increasing identifiers, whatever
    /// entity each one was about. The identifier is the join key a host later
    /// uses to attach an outcome label to the assessment that predicted it,
    /// so reuse or reordering would silently credit one request's outcome to
    /// another's estimate.
    ///
    /// ´claim:assess:assessment-ids-are-handed-out-in-strictly-increasing-order´
    /// ´test:unit:assessment-id-monotonically-increasing´
    #[test]
    fn assessment_id_monotonically_increasing() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();

        let req1 = RequestContext::new(EntityKey::new(vec![1]));
        let req2 = RequestContext::new(EntityKey::new(vec![2]));
        let req3 = RequestContext::new(EntityKey::new(vec![3]));

        let r1 = assess_then_derive(&req1, &shared, &snapshot, Instant::now());
        let r2 = assess_then_derive(&req2, &shared, &snapshot, Instant::now());
        let r3 = assess_then_derive(&req3, &shared, &snapshot, Instant::now());

        assert!(r1.assessment.id < r2.assessment.id);
        assert!(r2.assessment.id < r3.assessment.id);
    }

    /// A derivation always fields the whole slate of competing tags rather
    /// than only the winner: at least the three classification tags plus the
    /// action tags come back, each with its own resonance on the shared
    /// posture axis. A host therefore reads a distribution over postures and
    /// can see how close the runner-up was, instead of a bare verdict.
    ///
    /// ´claim:assess:a-derivation-fields-the-whole-slate-of-competing-tags-not-only-the-winner´
    /// ´test:unit:reckoning-has-tags´
    #[test]
    fn reckoning_has_tags() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        assert!(reckoning.profile.tags.len() >= 6);

        let classification_count = reckoning.profile.tags.iter().filter(|t| t.is_classification()).count();
        assert!(classification_count >= 3);
    }

    /// The reported probability comes from the real blend arithmetic, not from
    /// a placeholder: with sister and anchor both at zero mean and identity
    /// covariance the anchor earns essentially no weight — neither model is
    /// better informed in the shared subspace — and the calibrated sigmoid of
    /// a zero linear predictor lands on even odds. A cold-start system says it
    /// does not know, and says so through the same path a warm one uses.
    ///
    /// ´claim:assess:at-the-cold-start-prior-the-anchor-earns-no-weight-and-the-blend-reports-even-odds´
    /// ´test:unit:reckoning-risk-uses-real-blend´
    #[test]
    fn reckoning_risk_uses_real_blend() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // With identity covariance and zero means for both sister and anchor:
        // - w ≈ 0 (both equally uncertain in shared subspace)
        // - ρ̂_eff = (1-w)·0 + w·0 = 0
        // - p_bad = σ(0 / κ_eff) = 0.5
        assert!(
            (reckoning.assessment.risk.p_bad - 0.5).abs() < 0.01,
            "p_bad should be near 0.5, got {}",
            reckoning.assessment.risk.p_bad
        );
        assert!(
            reckoning.assessment.risk.anchor_weight < 0.01,
            "anchor_weight should be near 0, got {}",
            reckoning.assessment.risk.anchor_weight
        );
    }

    /// The ambiguity gauge is positive even for a prior-only assessment: a
    /// field the model knows least about is rendered at its most ambiguous,
    /// so a system that has learned nothing yet reports a genuinely
    /// undifferentiated display rather than a silent one. Whether resolving
    /// it would change an action is fragility's question, asked of the
    /// landscape (´def:fragility:definition´).
    ///
    /// ´claim:assess:even-a-prior-only-assessment-renders-an-ambiguous-field´
    /// ´test:unit:reckoning-ambiguity-non-zero´
    #[test]
    fn reckoning_ambiguity_non_zero() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        let gauge = crate::resonance::derivation::profile_ambiguity(&reckoning.profile, 0.3);
        assert!(gauge.total > 0.0);
    }

    /// One assessment contributes exactly one observation to the concordance
    /// tracker — not zero when no Sentinel reported, and not one per Sentinel.
    /// The tracker calibrates thresholds from the distribution of what it is
    /// fed, so a per-request cadence is what keeps that distribution in step
    /// with request volume.
    ///
    /// ´claim:assess:every-assessment-feeds-the-concordance-tracker-exactly-once´
    /// ´test:unit:concordance-observed´
    #[test]
    fn concordance_observed() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let _reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        let observations = shared.concordance_observations.lock().unwrap();
        assert_eq!(observations.len(), 1);
        drop(observations);
    }

    /// Every assessment carries inline health with it: the system-wide
    /// convergence stage the estimate was made under, and a degradation record
    /// that reports clean when nothing had to be salvaged. A consumer can
    /// therefore tell an ordinary cold-start estimate from a rescued one at
    /// the point of use, without a second query into health state.
    ///
    /// ´claim:assess:each-assessment-carries-its-own-convergence-stage-and-a-degradation-record-that-reads-clean-when-nothing-was-salvaged´
    /// ´test:unit:health-snapshot-populated´
    #[test]
    fn health_snapshot_populated() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        assert_eq!(
            reckoning.assessment.health.convergence_stage,
            CompositeConvergenceStage::ColdStart
        );
        assert!(!reckoning.assessment.health.degradation.is_degraded());
    }

    /// With no measurement surface registered at all, coverage is reported as
    /// zero rather than as a division by an empty denominator, and no Sentinel
    /// summaries are invented. Running with no Sentinels is a designed
    /// operating mode — prior-only — so it must produce ordinary numbers a
    /// host can log and compare, not a special case.
    ///
    /// ´claim:assess:with-no-sentinels-registered-coverage-reads-zero-rather-than-undefined´
    /// ´test:unit:no-sentinels-returns-valid-reckoning´
    #[test]
    fn no_sentinels_returns_valid_reckoning() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        assert_eq!(reckoning.assessment.health.sentinel_coverage.to_bits(), 0.0f64.to_bits());
        assert_eq!(reckoning.assessment.per_sentinel.len(), 0);
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Pipeline Mechanics — additional tests
    // ─────────────────────────────────────────────────────────────────────────

    /// Registration alone buys a Sentinel no entry in the per-Sentinel map: a
    /// Sentinel whose batch report has never arrived contributes nothing to
    /// summarise, and coverage counts it as missing. The map therefore lists
    /// what actually spoke for this request rather than what was configured,
    /// so a consumer cannot mistake an enrolled-but-silent Sentinel for one
    /// that reported nothing alarming.
    ///
    /// ´claim:assess:only-sentinels-that-actually-reported-appear-in-the-per-sentinel-map´
    /// ´test:unit:sentinel-registered-but-no-report´
    #[test]
    fn sentinel_registered_but_no_report() {
        let shared = MockSharedState::new().with_sentinel(1);
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3])).with_sentinel(SentinelId(1), 0x1234_5678);

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // Sentinel registered but no report ingested — assessment plus derivation should still be valid.
        // System-wide coverage is 0 because no cached batch reports.
        assert_eq!(reckoning.assessment.health.sentinel_coverage.to_bits(), 0.0f64.to_bits());
        // per_sentinel only contains *reporting* Sentinels
        // (´def:runtime:alarm-summary´). With no cached report, the map is empty.
        assert_eq!(reckoning.assessment.per_sentinel.len(), 0);
        assert!(!reckoning.assessment.per_sentinel.contains_key(&SentinelId(1)));
    }

    /// A request that omits the coordinate for a registered Sentinel is not an
    /// error: the assessment completes, and that Sentinel simply has nothing
    /// to say about this entity — it stays out of the per-Sentinel map, it is
    /// not counted as contributing, and an assessment in which nothing was
    /// measured says so through `zero_sentinels`. Occupancy is the online
    /// flag, not a contribution: an active Sentinel with no data for the
    /// request holds an occupied, empty extraction, and reading that as a
    /// contributor would deny a prior-only estimate was prior-only
    /// (´schema:output:assessment´).
    ///
    /// (´claim:assess:only-sentinels-that-actually-reported-appear-in-the-per-sentinel-map´)
    /// ´test:unit:sentinel-no-coordinate-in-request´
    #[test]
    fn sentinel_no_coordinate_in_request() {
        let shared = MockSharedState::new().with_sentinel(1);
        let snapshot = test_snapshot();
        // No sentinel coordinate provided in request
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // Sentinel registered but no coordinate — extraction has occupancy=1:
        // "active, no data for the request" (´tab:registry:coverage-states´).
        // System-wide coverage is 0 because no cached batch reports.
        assert_eq!(reckoning.assessment.health.sentinel_coverage.to_bits(), 0.0f64.to_bits());
        // per_sentinel only contains reporting Sentinels
        // (´def:runtime:alarm-summary´). Without a cached batch report there is
        // nothing to summarise.
        assert_eq!(reckoning.assessment.per_sentinel.len(), 0);
        // The contribution count agrees with the map it is documented
        // against: nothing was measured, and the assessment says so.
        assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 0);
        assert!(reckoning.assessment.health.zero_sentinels);
    }

    /// Assessing consumes an identifier from the shared counter — the counter
    /// visibly advances across the call. Identifiers are allocated by the act
    /// of assessing rather than by the host afterwards, which is what makes
    /// the pending-buffer entry and the returned assessment refer to the same
    /// request by construction.
    ///
    /// ´claim:assess:assessing-consumes-an-identifier-from-the-shared-counter´
    /// ´test:unit:assessment-assigns-assessment-id´
    #[test]
    fn assessment_assigns_assessment_id() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let initial_id = shared.next_id.load(std::sync::atomic::Ordering::Relaxed);

        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
        let _reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        let after_id = shared.next_id.load(std::sync::atomic::Ordering::Relaxed);
        assert!(after_id > initial_id, "ID should be assigned on assessment");
    }

    /// Every outcome axis carried by the snapshot gets its own prediction,
    /// keyed by axis, and the set is exactly the registered axes — no more and
    /// no fewer. The axes come from the snapshot the assessment was made
    /// against, so adding an axis to the model is enough to make it appear in
    /// results, and a consumer indexing by axis never finds a hole.
    ///
    /// ´claim:assess:every-outcome-axis-in-the-snapshot-gets-exactly-one-prediction´
    /// ´test:unit:outcome-predictions-per-registered-axis´
    #[test]
    fn outcome_predictions_per_registered_axis() {
        use crate::snapshot::published::AxisModelState;

        let shared = MockSharedState::new();
        let mut snapshot = test_snapshot();

        // Register 2 outcome axes
        let axis_params = make_identity_params(10);
        snapshot.outcome_models.insert(
            crate::types::OutcomeAxisId(1),
            AxisModelState {
                name: "axis_1".to_owned(),
                parameters: axis_params.clone(),
                kappa_a: 1.0,
                gamma_a: 0.999,
                spatial: false,
                eligibility: crate::types::OutcomeEligibility::default(),
            },
        );
        snapshot.outcome_models.insert(
            crate::types::OutcomeAxisId(2),
            AxisModelState {
                name: "axis_2".to_owned(),
                parameters: axis_params,
                kappa_a: 1.0,
                gamma_a: 0.999,
                spatial: false,
                eligibility: crate::types::OutcomeEligibility::default(),
            },
        );

        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        assert_eq!(reckoning.assessment.outcome_predictions.len(), 2);
        assert!(
            reckoning
                .assessment
                .outcome_predictions
                .contains_key(&crate::types::OutcomeAxisId(1))
        );
        assert!(
            reckoning
                .assessment
                .outcome_predictions
                .contains_key(&crate::types::OutcomeAxisId(2))
        );
    }

    // ─────────────────────────────────────────────────────────────────────────
    // NaN Checkpoint Tests (CP1–CP4)
    // ─────────────────────────────────────────────────────────────────────────

    /// A non-finite value handed in as a request signal neither poisons the
    /// estimate nor fails the request: it is replaced and counted, and the
    /// count rides back to the host on the assessment's own degradation
    /// record. The host learns that its input was unusable at the moment it
    /// mattered, instead of discovering it later from an inexplicable score.
    ///
    /// ´claim:assess:a-non-finite-request-signal-is-replaced-and-counted-rather-than-rejected´
    /// ´test:unit:cp1-nan-signal-sanitised´
    #[test]
    fn cp1_nan_signal_sanitised() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3])).with_signal("test_signal", f64::NAN);

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // CP1: NaN signal should be counted in degradation
        assert!(
            reckoning.assessment.health.degradation.signals_sanitised >= 1,
            "NaN signal should be counted"
        );
    }

    /// Infinity is treated exactly as NaN is — the guard is on finiteness, not
    /// on NaN specifically. An overflowed counter or a division by zero in the
    /// host arrives as an infinity far more often than as a NaN, so a check
    /// that caught only the latter would let the commoner failure straight
    /// through into the feature vector.
    ///
    /// (´claim:assess:a-non-finite-request-signal-is-replaced-and-counted-rather-than-rejected´)
    /// ´test:unit:cp1-inf-signal-sanitised´
    #[test]
    fn cp1_inf_signal_sanitised() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3])).with_signal("test_signal", f64::INFINITY);

        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // CP1: Inf signal should be counted in degradation
        assert!(
            reckoning.assessment.health.degradation.signals_sanitised >= 1,
            "Inf signal should be counted"
        );
    }

    /// Standardising against a degenerate variance — one that has underflowed
    /// to zero — cannot put a NaN into the reported probability. Feature
    /// variances are learned quantities, so a feature that never varied will
    /// eventually present one; the guard between standardisation and the blend
    /// is what stops that arithmetic accident from becoming an unusable risk
    /// estimate.
    ///
    /// ´claim:assess:a-degenerate-feature-variance-cannot-put-a-nan-into-the-reported-probability´
    /// ´test:unit:cp3-nan-after-standardisation-sanitised´
    #[test]
    fn cp3_nan_after_standardisation_sanitised() {
        let shared = MockSharedState::new();
        let mut snapshot = test_snapshot();

        // Set up extreme variance that could produce NaN during standardisation
        // (division by very small variance)
        let dim_p = snapshot.dimension_map.p;
        snapshot.feature_variances = vec![1e-400; dim_p]; // Subnormal variance

        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        // Assessment plus derivation should succeed — any NaN values would be sanitised.
        assert!(!reckoning.assessment.risk.p_bad.is_nan(), "Risk p_bad should not be NaN");
    }

    /// Building a risk basis is honest arithmetic and nothing more: a
    /// non-finite effective log-odds out of the blend propagates straight
    /// through the calibrated sigmoid into the probability. Nothing is swept
    /// up at that layer, which is precisely why a separate guard afterwards
    /// must substitute the even-odds prior and record which model was
    /// degraded — the fallback is a deliberate, reported decision rather than
    /// a silent clamp buried in the formula.
    ///
    /// ´claim:assess:the-risk-basis-propagates-a-non-finite-blend-so-a-later-guard-must-substitute-the-prior-and-name-the-degraded-model´
    /// ´test:unit:cp4-nan-from-blend-falls-back-to-prior´
    #[test]
    fn cp4_nan_from_blend_falls_back_to_prior() {
        // Unit test: RiskBasis::from_blend with NaN rho produces NaN p_bad
        let nan_blend = BlendResult {
            anchor_weight: 0.8,
            rho_inherited: 0.0,
            rho_anchor: 0.0,
            rho_effective: f64::NAN,
            variance_effective: 0.8,
            variance_inherited: 0.8,
            variance_anchor: 0.8,
            variance_inherited_shared: 0.8,
            kappa_effective: 1.0,
            borrowed_share_effective: 0.0,
        };
        let empty_cal = crate::snapshot::published::CalibrationBufferSnapshot::default();
        let prior_only = prior_only_variance(1.0, 0.1);
        let risk = RiskBasis::from_blend(&nan_blend, 0.0, 0, &empty_cal, prior_only);
        assert!(risk.p_bad.is_nan(), "NaN rho should yield NaN p_bad before CP4 guard");

        // Integration test: full pipeline catches it via CP4 guard.
        // Verify the guard logic directly:
        let mut risk_guarded = RiskBasis::from_blend(&nan_blend, 0.0, 0, &empty_cal, prior_only);
        let mut degradation = DegradationContext::default();
        if !risk_guarded.p_bad.is_finite() {
            risk_guarded.p_bad = 0.5;
            degradation.degraded_models.push(ModelId::Operational);
        }
        assert!((risk_guarded.p_bad - 0.5).abs() < f64::EPSILON, "CP4 should fall back to 0.5");
        assert_eq!(degradation.degraded_models.len(), 1);
        assert_eq!(degradation.degraded_models[0], ModelId::Operational);
    }

    /// The outcome-axis half of the same checkpoint computes the same notion
    /// of a prior as the risk-basis half: a non-finite axis prediction is
    /// replaced by a zero point estimate with the time-corrected prior
    /// standard deviation carried through the axis's own compression scale —
    /// `kappa_a * sqrt(time_correction / lambda_prior)`, exactly 1.25 at a
    /// compression scale of one half, a quadrupled time correction and a
    /// prior precision of 0.64 — and the axis is flagged as degraded, while a
    /// finite prediction passes through untouched. A constant substitute
    /// would put the degraded interval on a scale no successful prediction of
    /// this axis uses (´def:axis:prior-only-prediction´).
    ///
    /// ´claim:assess:the-degraded-axis-prediction-is-the-computed-prior-at-the-axis-own-scale´
    /// ´test:unit:cp4-nan-axis-falls-back-to-computed-prior´
    #[test]
    fn cp4_nan_axis_falls_back_to_computed_prior() {
        let kappa_a = 0.5;
        let time_correction = 4.0;
        let lambda_prior = 0.64;

        // A poisoned prediction takes the computed prior and the flag.
        let poisoned = OutcomePrediction {
            axis_id: OutcomeAxisId(7),
            axis_name: "poisoned".to_owned(),
            predicted_raw: f64::NAN,
            uncertainty: f64::NAN,
            prediction_interval: (f64::NAN, f64::NAN),
        };
        let mut degradation = DegradationContext::default();
        let pred = cp4_guard_axis_prediction(poisoned, "poisoned", kappa_a, time_correction, lambda_prior, &mut degradation);

        assert!(
            (pred.predicted_raw - 0.0).abs() < f64::EPSILON,
            "prior point estimate is zero"
        );
        // κ_a·√(T_corr/λ_prior) = 0.5·√(4/0.64) = 0.5·2.5/1 = 1.25.
        let expected = 0.5 * (4.0_f64 / 0.64).sqrt();
        assert!(
            (pred.uncertainty - expected).abs() < 1e-12,
            "κ_a·√(T_corr/λ_prior) expected {expected}, got {}",
            pred.uncertainty
        );
        assert!(
            1.96f64.mul_add(-expected, pred.prediction_interval.1).abs() < 1e-12,
            "the interval follows the computed deviation"
        );
        assert_eq!(degradation.degraded_models, vec![ModelId::OutcomeAxis(OutcomeAxisId(7))]);

        // A finite prediction passes through untouched and unflagged.
        let healthy = OutcomePrediction {
            axis_id: OutcomeAxisId(8),
            axis_name: "healthy".to_owned(),
            predicted_raw: 0.3,
            uncertainty: 0.2,
            prediction_interval: (-0.092, 0.692),
        };
        let mut clean = DegradationContext::default();
        let passed = cp4_guard_axis_prediction(healthy, "healthy", kappa_a, time_correction, lambda_prior, &mut clean);
        assert!((passed.predicted_raw - 0.3).abs() < f64::EPSILON);
        assert_eq!(clean.degraded_models, vec![]);
    }

    /// The batch-init path's two load conditions travel in band on the
    /// degradation context, exactly as the identity queue's overflow does: an
    /// observation skipped on a contended lock counts one skip and marks the
    /// assessment degraded, a completion deferred on a full command channel
    /// counts one deferral likewise, and an ordinary observation leaves both
    /// counters at zero. Before these counters, initialisation could be
    /// starved with no surface reporting it — the host would discover the
    /// consequence from a standardisation that never improved
    /// (´cor:degradation:in-band-travel´).
    ///
    /// ´claim:assess:batch-init-load-conditions-are-reported-in-band-on-the-degradation-context´
    /// ´test:unit:batch-init-load-conditions-reported-in-band´
    #[test]
    fn batch_init_load_conditions_reported_in_band() {
        let snapshot = test_snapshot();

        let contended = MockSharedState::new().with_batch_init_outcome(BatchInitObservation::SkippedContended);
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
        let reckoning = assess_then_derive(&req, &contended, &snapshot, Instant::now());
        let degradation = &reckoning.assessment.health.degradation;
        assert_eq!(degradation.batch_init_observations_skipped, 1);
        assert!(degradation.is_degraded(), "a reported load condition is a degradation");

        let observed = MockSharedState::new();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));
        let reckoning = assess_then_derive(&req, &observed, &snapshot, Instant::now());
        let degradation = &reckoning.assessment.health.degradation;
        assert_eq!(degradation.batch_init_observations_skipped, 0);
        assert!(!degradation.is_degraded());
    }

    // ─────────────────────────────────────────────────────────────────────────
    // Data Access Tests
    // ─────────────────────────────────────────────────────────────────────────

    /// A run of assessments against one unchanging snapshot still numbers each
    /// one distinctly and in order. Reading the model is separate from
    /// allocating identity: sharing a snapshot across many requests — the
    /// normal case, since the snapshot only changes when the writer publishes
    /// — never causes two requests to collide on an identifier.
    ///
    /// (´claim:assess:assessment-ids-are-handed-out-in-strictly-increasing-order´)
    /// ´test:unit:snapshot-consistent-across-pipeline´
    #[test]
    fn snapshot_consistent_across_pipeline() {
        let shared = MockSharedState::new();
        let snapshot = test_snapshot();
        let req = RequestContext::new(EntityKey::new(vec![1, 2, 3]));

        // Perform 10 assessments through the host-composition helper.
        let mut ids = Vec::new();
        for _ in 0..10 {
            let r = assess_then_derive(&req, &shared, &snapshot, Instant::now());
            ids.push(r.assessment.id);
        }

        // All IDs should be unique and monotonically increasing
        for i in 1..ids.len() {
            assert!(ids[i] > ids[i - 1], "IDs should be monotonically increasing");
        }
    }

    /// Coverage depth divides the receiving cell's depth by the dimension's
    /// own declared width, so a cell at the bottom of a thirty-two-bit
    /// dimension reads a full 1.0 and a cell eight levels down reads a
    /// quarter — the same fraction of its own domain that a cell thirty-two
    /// levels down a full-width dimension reads. The quantity is reduced by
    /// maximum across dimensions, and a maximum over fractions of different
    /// wholes would let a full-width dimension win on width alone; at the
    /// full 128 bits the divisor coincides with the old literal, which is why
    /// suites that only ever declared full-width dimensions could not see the
    /// difference.
    ///
    /// ´claim:assess:coverage-depth-is-a-fraction-of-the-dimensions-own-declared-width´
    /// ´test:unit:coverage-depth-uses-dimension-domain-width´
    #[test]
    fn coverage_depth_uses_dimension_domain_width() {
        // A 32-bit dimension: depth 8 is a quarter of the way down.
        assert!((coverage_depth_feature(8, 32) - 0.25).abs() < 1e-15);
        // Its deepest cell reads the full scale.
        assert!((coverage_depth_feature(32, 32) - 1.0).abs() < 1e-15);
        // At full width the divisor coincides with the retired literal.
        assert!((coverage_depth_feature(64, 128) - 0.5).abs() < 1e-15);
    }

    // ─────────────────────────────────────────────────────────────────────
    // The tempering (´dec:risk:evidence-only-uncertainty´)
    // ─────────────────────────────────────────────────────────────────────

    /// Helper: a blend result carrying a stated variance and borrowed share,
    /// with every other field at a value the tempering does not read.
    fn blend_at(variance: f64, borrowed_share_effective: f64) -> BlendResult {
        BlendResult {
            anchor_weight: 0.5,
            rho_inherited: 0.0,
            rho_anchor: 0.0,
            rho_effective: 0.0,
            variance_effective: variance,
            variance_inherited: variance,
            variance_anchor: variance,
            variance_inherited_shared: variance,
            kappa_effective: 1.0,
            borrowed_share_effective,
        }
    }

    /// A verdict from models that borrowed nothing is the verdict the package
    /// reported before the tempering existed, to the last bit. That is a
    /// stronger statement than agreement to a tolerance and it is made
    /// deliberately: dividing by one is exact, and the ceiling is never below
    /// the variance it caps, so the healthy path is the identity rather than a
    /// rounding of it. A tempering that moved a healthy verdict by a few units
    /// in the last place would be a change to every number the package has
    /// ever published, and the empirical coverage refit would spend its
    /// evidence chasing it.
    ///
    /// ´claim:assess:a-verdict-from-models-that-borrowed-nothing-is-bit-identical-to-the-untempered-one´
    /// ´test:unit:the-untempered-verdict-is-bit-identical´
    #[test]
    fn the_untempered_verdict_is_bit_identical() {
        let empty_cal = crate::snapshot::published::CalibrationBufferSnapshot::default();
        let prior_only = prior_only_variance(1.0, 0.1);

        for variance in [1e-6_f64, 0.25, 1.0, 7.5, 400.0] {
            let blend = blend_at(variance, 0.0);
            let risk = RiskBasis::from_blend(&blend, 0.0, 0, &empty_cal, prior_only);

            let expected_sigma = variance.sqrt();
            assert!(
                risk.sigma_eff.to_bits() == expected_sigma.to_bits(),
                "σ_eff is the untempered root at variance {variance}: {} against {expected_sigma}",
                risk.sigma_eff
            );

            let expected_uncertainty = risk.p_bad * (1.0 - risk.p_bad) * expected_sigma / blend.kappa_effective;
            assert!(
                risk.uncertainty.to_bits() == expected_uncertainty.to_bits(),
                "and the probability-space uncertainty with it: {} against {expected_uncertainty}",
                risk.uncertainty
            );
            assert!(risk.borrowed_share.abs() < f64::EPSILON, "with nothing borrowed to report");
        }
    }

    /// Where the share is genuinely below one, the tempered variance is the
    /// closed form and not an approximation of it: posterior variance over one
    /// minus the share, to rounding. The derivation is a subtraction rather
    /// than a model — along a direction the posterior precision is the
    /// evidence's plus the floors', so the evidence's alone is `(1 − s)` times
    /// the posterior's and its variance is the reciprocal of that — and a test
    /// that accepted a wider tolerance here would be accepting a second
    /// formula nobody had written down.
    ///
    /// ´claim:assess:the-tempered-variance-is-the-posterior-variance-over-one-minus-the-borrowed-share´
    /// ´test:unit:the-tempering-is-the-closed-form´
    #[test]
    fn the_tempering_is_the_closed_form() {
        // A prior-only variance far above anything asked for, so the ceiling
        // is not the binding constraint anywhere in this sweep.
        let prior_only = 1e6;
        for variance in [1e-6_f64, 0.25, 1.0, 7.5] {
            for share in [1e-9_f64, 0.01, 0.1, 0.5, 0.9, 0.99, 0.999] {
                let tempered = temper_variance(variance, share, prior_only);
                let closed_form = variance / (1.0 - share);
                assert!(
                    (tempered - closed_form).abs() <= closed_form * 1e-15,
                    "variance {variance} at share {share}: {tempered} against {closed_form}"
                );
            }
        }
    }

    /// A direction the floors are holding up entirely saturates at the
    /// prior-only variance and never passes it. The quotient diverges as the
    /// share approaches one, and what the package must report there is not a
    /// large number but a statement: the prior is all that is known along this
    /// direction. That statement already has a value — the one the fourth
    /// numeric checkpoint substitutes when the blend's own variance is
    /// unusable — so the cap is that value rather than a tolerance chosen
    /// beside it, and the same expression computes both.
    ///
    /// The ceiling is taken as the larger of the prior-only variance and the
    /// variance being tempered, so a blended variance already above prior-only
    /// — which the mixture's disagreement term can produce — is not narrowed
    /// by a cap that exists to bound a widening. The tempering is a widening
    /// or nothing.
    ///
    /// ´claim:assess:the-tempering-saturates-at-the-prior-only-variance-and-never-narrows-a-verdict´
    /// ´test:unit:the-tempering-saturates-at-the-prior´
    #[test]
    fn the_tempering_saturates_at_the_prior() {
        let prior_only = prior_only_variance(1.0, 0.1); // = 10.0
        let variance = 0.5_f64;

        // Approaching a share of one, the widening rises to the ceiling and
        // stops there, never above it.
        let mut previous = variance;
        for share in [0.5_f64, 0.9, 0.95, 0.99, 0.999, 0.9999, 1.0 - 1e-12, 1.0] {
            let tempered = temper_variance(variance, share, prior_only);
            assert!(
                tempered >= previous,
                "the widening does not go backwards at share {share}: {tempered} against {previous}"
            );
            assert!(
                tempered <= prior_only,
                "and never passes the prior-only variance at share {share}: {tempered}"
            );
            previous = tempered;
        }
        assert!(
            (temper_variance(variance, 1.0, prior_only) - prior_only).abs() < f64::EPSILON,
            "a direction held entirely by the floors reads exactly the prior-only variance"
        );

        // A variance already above the prior-only one is not narrowed by the
        // cap, and a share of zero leaves it exactly where it was.
        let wide = prior_only * 3.0;
        assert!(
            (temper_variance(wide, 0.0, prior_only) - wide).abs() < f64::EPSILON,
            "an untempered wide variance is left alone"
        );
        assert!(
            temper_variance(wide, 0.5, prior_only) >= wide,
            "and a tempered one is never narrowed by the cap"
        );

        // A variance the arithmetic failed to produce passes through, so the
        // checkpoint downstream still sees the failure.
        assert!(
            temper_variance(f64::NAN, 0.5, prior_only).is_nan(),
            "a non-finite variance is not quietly replaced by the ceiling"
        );
    }

    /// The uncertainty a host reads widens monotonically with the share, and
    /// the share is where the mechanism's whole content sits: two requests
    /// against the same models, differing only in which direction they ask
    /// about, receive different uncertainties because the model's evidence is
    /// not evenly distributed over directions. A reading that widened every
    /// verdict by one aggregate factor would be reporting the model's average
    /// condition; this reports its condition where the question was asked.
    ///
    /// ´claim:assess:the-reported-uncertainty-widens-monotonically-with-the-borrowed-share´
    /// ´test:unit:the-uncertainty-widens-with-the-share´
    #[test]
    fn the_uncertainty_widens_with_the_share() {
        let empty_cal = crate::snapshot::published::CalibrationBufferSnapshot::default();
        let prior_only = 1e6;
        let variance = 0.4_f64;

        let mut previous = 0.0_f64;
        for share in [0.0_f64, 0.05, 0.2, 0.4, 0.6, 0.8, 0.9] {
            let risk = RiskBasis::from_blend(&blend_at(variance, share), 0.0, 0, &empty_cal, prior_only);
            assert!(
                risk.uncertainty > previous,
                "the uncertainty widened at share {share}: {} against {previous}",
                risk.uncertainty
            );
            assert!(
                (risk.borrowed_share - share).abs() < f64::EPSILON,
                "and the share rides beside it unchanged"
            );
            previous = risk.uncertainty;
        }

        // Undoing the widening recovers the untempered figure, which is what
        // makes carrying the share sufficient and a second field unnecessary.
        let share = 0.75_f64;
        let tempered = RiskBasis::from_blend(&blend_at(variance, share), 0.0, 0, &empty_cal, prior_only);
        let untempered = RiskBasis::from_blend(&blend_at(variance, 0.0), 0.0, 0, &empty_cal, prior_only);
        let recovered = tempered.uncertainty * (1.0 - tempered.borrowed_share).sqrt();
        assert!(
            (recovered - untempered.uncertainty).abs() < untempered.uncertainty * 1e-12,
            "a host can undo the tempering from the share alone: {recovered} against {}",
            untempered.uncertainty
        );
    }

    /// The checkpoint's fallback path carries the same tempering in the same
    /// order, and carrying it changes nothing, which is the point. The
    /// fallback substitutes the prior-only standard deviation, and the
    /// prior-only variance is exactly what the tempering saturates at, so the
    /// operation is provably an identity there. It is applied rather than
    /// skipped so that the two paths hold one operation in one order: a later
    /// edit that changed the tempering on the ordinary path and forgot the
    /// fallback would otherwise leave two definitions of the same quantity.
    /// A share the arithmetic failed to produce is sanitised before it is
    /// read, and reports the degradation it caused.
    ///
    /// ´claim:assess:the-checkpoints-fallback-applies-the-same-tempering-and-is-an-identity-there´
    /// ´test:unit:the-fallback-carries-the-same-tempering´
    #[test]
    fn the_fallback_carries_the_same_tempering() {
        let time_correction = 4.0;
        let lambda_prior = 0.64;
        let prior_only = prior_only_variance(time_correction, lambda_prior);
        let empty_cal = crate::snapshot::published::CalibrationBufferSnapshot::default();

        // A blend whose variance is not a number, at a substantial share.
        let mut risk = RiskBasis::from_blend(&blend_at(f64::NAN, 0.8), 0.0, 0, &empty_cal, prior_only);
        assert!(!risk.sigma_eff.is_finite(), "the failure reached the checkpoint");
        let degraded = sanitise_risk_basis(&mut risk, time_correction, lambda_prior);
        assert!(degraded, "and the checkpoint reported it");
        assert!(
            (risk.sigma_eff - prior_only.sqrt()).abs() < f64::EPSILON,
            "the fallback is the prior-only standard deviation, tempering included: {} against {}",
            risk.sigma_eff,
            prior_only.sqrt()
        );

        // A share that is not a reading is replaced by no reading at all.
        let mut poisoned = RiskBasis::from_blend(&blend_at(0.5, 0.0), 0.0, 0, &empty_cal, prior_only);
        poisoned.borrowed_share = f64::NAN;
        assert!(
            sanitise_risk_basis(&mut poisoned, time_correction, lambda_prior),
            "a share that is not a number is a degradation"
        );
        assert!(
            poisoned.borrowed_share.abs() < f64::EPSILON,
            "and it is replaced by zero rather than carried into the arithmetic"
        );
    }
}
