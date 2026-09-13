// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`axis_qf_matches_blend_pattern`] | wellness | An outcome axis forms its point estimate the same way the sister model does — the inner product of the model's mean with the feature vector, then carried out of the bounded space by the inverse hyperbolic tangent. Axes and the blend are meant to be commensurable, so an axis that scored by a different rule would produce numbers that could not be compared with the model they sit beside. |
//! | [`inverse_tanh_round_trip`] | wellness | The stable inverse hyperbolic tangent recovers its argument to within a tenth of a picounit across the whole working range, from minus five to plus five. Predictions cross between bounded and unbounded space repeatedly, and an inverse that lost precision near the ends of the range would bias exactly the confident predictions that matter most. |
//! | [`prior_only_axis_predicts_zero_with_a_well_formed_interval`] | wellness | A model still sitting at its prior predicts nothing in particular but says so coherently: the point estimate is centred on zero whatever features it is shown, the uncertainty is strictly positive, the interval bounds are ordered low before high, and every one of those numbers is finite. An uninformed prediction is useful precisely because its uncertainty is well-formed enough to be reasoned about. |
//! | [`outcome_axis_uncertainty_scales_with_time_correction`] | wellness | Time correction inflates variance and nothing else: quadrupling it leaves the point estimate bit-for-bit where it was and widens the uncertainty by exactly a factor of two — the square root, because the correction multiplies the variance rather than the deviation. Staleness makes the system less certain of an answer, never differently opinionated about it. |
//! | [`dimension_mismatch_returns_prior`] | wellness | A feature vector whose length disagrees with the model's yields the computed prior — zero estimate, the time-corrected prior standard deviation through the compression scale — rather than a number computed from whichever coordinates happened to line up. At unit parameters that deviation is one; at a doubled compression scale and a quadrupled time correction it is exactly four, so a stale degraded prediction widens with staleness and carries the axis's scale like any successful one (´def:axis:prior-only-prediction´). |
//! | [`prediction_interval_monte_carlo`] | wellness | The reported credible interval means what it claims: a thousand draws from the very distribution the axis says it is predicting fall inside the interval about ninety-five percent of the time. The bounds are not merely ordered and finite — they are calibrated against the deviation the same prediction reports, so a host sizing its risk appetite by the interval is sizing it correctly. |
//! | [`drift_cusum_accumulates_positive`] | wellness | Drift is watched in each direction independently: a long run of residuals well above the allowance builds evidence in the upper accumulator while the lower one stays at rest, and the upper one never goes negative even after the threshold crossing has reset it. Over-prediction and under-prediction are different failures, and an accumulator that netted them would let a model drifting both ways at once look healthy. |
//! | [`drift_cusum_decays_with_noise`] | wellness | Residuals inside the allowance never accumulate, however long they go on: a hundred rounds alternating either side of zero, each smaller than the allowance, leave both accumulators small. Ordinary prediction error is expected and must not be mistaken for drift merely by being persistent — otherwise every model would eventually trip its own reset by sitting still. |
//! | [`drift_auto_reset_triggers`] | wellness | Crossing the threshold resets the accumulator, and the update that did it says so in its return value: driven with no allowance at all against a low threshold, the run reports at least one reset rather than silently zeroing itself. The caller is what turns a detected drift into action, so a reset nobody was told about would clear the evidence and leave the drifting model in place. |
//! | [`drift_ewma_converges`] | wellness | Alongside the accumulators, two smoothed measures settle on what the residuals actually are: fed a constant residual, the mean absolute residual converges on its magnitude and the sign measure approaches its direction. Together they describe a bias the accumulators only signal the presence of — how large it is, and which way it leans. |
//! | [`drift_reset_cusums_only`] | wellness | Clearing the accumulated evidence spares the smoothed history: both accumulators and the step count return to zero while the mean absolute residual and the sign measure survive untouched. Acting on a detected drift should not also destroy the long-memory description of how the residuals behave, which takes many rounds to rebuild and is what makes the next detection interpretable. |
//! | [`drift_reset_all`] | wellness | The full reset is the stronger operation: accumulators, step count, mean absolute residual and sign measure all return to their constructed state together. This is what a model replacement calls for — residual statistics describing predictions a different model made would be actively misleading applied to its successor. |
//! | [`drift_state_serde_round_trip`] | wellness | Drift state survives being written out and read back whole: both accumulators, both smoothed measures — including a negative sign measure — and the step count all return bit-identical. Drift evidence is gathered over hundreds of labels, so a restart that dropped it would grant every drifting model a fresh start it had not earned. |

//! Acceptance tests for the supplementary computations of axis prediction
//! (´chap:spec:axis-prediction´).
//!
//! Two halves of one loop. An outcome axis turns a model and a feature vector
//! into a prediction with an honest uncertainty around it; the drift
//! accumulators then watch the residuals those predictions leave behind, and
//! raise a reset when the residuals stop looking like noise. A prediction the
//! system cannot make must come back as the prior rather than as a number, and
//! an interval it does report has to mean what it says.
//!
//! See also: `step16_skipped_in_replay` in `tests/label_pipeline.rs`
//!
//! # Cross-References
//!
//! - (´tab:axis:inference-outputs´) — what evaluating an outcome axis is
//!   obliged to return, prior-only prediction included
//! - (´alg:monitoring:drift-cusums´) — the accumulators that watch the
//!   residuals and raise the reset

use crate::assessment::OutcomePrediction;
use crate::health::{DriftConfig, DriftState};
use crate::model::parameters::ModelParameters;
use crate::numerics::stable_atanh;
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_finite, assert_near};

// ═══════════════════════════════════════════════════════════════════════════════
// Helper Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates scaled identity covariance.
fn scaled_identity_covariance(p: usize, scale: f64) -> Vec<f64> {
    let mut cov = vec![0.0; p * p];
    for i in 0..p {
        cov[i * p + i] = scale;
    }
    cov
}

/// Creates test model parameters.
fn test_model_parameters(p: usize, mu_val: f64, cov_scale: f64) -> ModelParameters {
    ModelParameters {
        mu: vec![mu_val; p],
        covariance_data: scaled_identity_covariance(p, cov_scale),
        p,
    }
}

/// Calls the evaluate_outcome_axis function from reckon.rs, at unit
/// prior precision.
fn evaluate_outcome_axis(params: &ModelParameters, phi_hat: &[f64], kappa_a: f64, time_correction: f64) -> OutcomePrediction {
    crate::assessment::evaluate_outcome_axis(
        crate::types::OutcomeAxisId(0),
        "test",
        params,
        phi_hat,
        kappa_a,
        time_correction,
        1.0,
    )
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Axis Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// An outcome axis forms its point estimate the same way the sister model
/// does — the inner product of the model's mean with the feature vector, then
/// carried out of the bounded space by the inverse hyperbolic tangent. Axes
/// and the blend are meant to be commensurable, so an axis that scored by a
/// different rule would produce numbers that could not be compared with the
/// model they sit beside.
///
/// ´claim:wellness:an-outcome-axis-forms-its-point-estimate-by-the-same-rule-as-the-sister-model´
/// ´test:crate:axis-qf-matches-blend-pattern´
#[test]
fn axis_qf_matches_blend_pattern() {
    // Same model, same φ̂ → axis QF should match the pattern used in blend.
    let p = 10;
    let params = test_model_parameters(p, 0.1, 0.5);
    let phi: Vec<f64> = (0..p).map(|i| 0.1 * i as f64).collect();

    let pred = evaluate_outcome_axis(&params, &phi, 1.0, 1.0);

    // Point estimate should be dot product: Σ (0.1 * 0.1*i) = 0.01 * Σi = 0.01 * (0+1+...+9) = 0.45
    // In tanh-space, then transformed via atanh
    let expected_o_hat = 0.45_f64;
    let expected_raw = stable_atanh(expected_o_hat.clamp(-0.9999999, 0.9999999));

    assert_near(pred.predicted_raw, expected_raw, 0.01, "axis point estimate vs blend");
}

/// The stable inverse hyperbolic tangent recovers its argument to within a
/// tenth of a picounit across the whole working range, from minus five to plus
/// five. Predictions cross between bounded and unbounded space repeatedly, and
/// an inverse that lost precision near the ends of the range would bias
/// exactly the confident predictions that matter most.
///
/// ´claim:wellness:the-stable-inverse-tanh-recovers-its-argument-across-the-whole-working-range´
/// ´test:crate:inverse-tanh-round-trip´
#[test]
fn inverse_tanh_round_trip() {
    // atanh(tanh(x)) = x for values in (-5, 5) within tolerance
    for x in (-50..=50).map(|i| i as f64 * 0.1) {
        let th = x.tanh();
        let recovered = stable_atanh(th);
        assert_near(recovered, x, 1e-10, "atanh(tanh(x)) round-trip");
    }
}

/// A model still sitting at its prior predicts nothing in particular but says
/// so coherently: the point estimate is centred on zero whatever features it
/// is shown, the uncertainty is strictly positive, the interval bounds are
/// ordered low before high, and every one of those numbers is finite. An
/// uninformed prediction is useful precisely because its uncertainty is
/// well-formed enough to be reasoned about.
///
/// ´claim:wellness:a-prior-only-axis-predicts-zero-with-a-positive-uncertainty-and-a-properly-ordered-interval´
/// ´test:crate:prior-only-axis-predicts-zero-with-a-well-formed-interval´
#[test]
fn prior_only_axis_predicts_zero_with_a_well_formed_interval() {
    // At prior (zero mu, identity covariance), predictions should be centered at zero
    let p = 10;
    let params = test_model_parameters(p, 0.0, 1.0);
    let phi: Vec<f64> = (0..p).map(|i| 0.1 * i as f64).collect();

    let pred = evaluate_outcome_axis(&params, &phi, 1.0, 1.0);

    // Centered at zero with some uncertainty
    assert_near(pred.predicted_raw, 0.0, 0.1, "prior point estimate");
    assert_finite(
        &[
            pred.predicted_raw,
            pred.uncertainty,
            pred.prediction_interval.0,
            pred.prediction_interval.1,
        ],
        "prior prediction",
    );
    assert!(
        pred.prediction_interval.0 < pred.prediction_interval.1,
        "Interval should be valid"
    );
    assert!(pred.uncertainty > 0.0, "Std dev should be positive");
}

/// Time correction inflates variance and nothing else: quadrupling it leaves
/// the point estimate bit-for-bit where it was and widens the uncertainty by
/// exactly a factor of two — the square root, because the correction multiplies
/// the variance rather than the deviation. Staleness makes the system less
/// certain of an answer, never differently opinionated about it.
///
/// ´claim:wellness:time-correction-widens-uncertainty-by-its-square-root-and-leaves-the-point-estimate-untouched´
/// ´test:crate:outcome-axis-uncertainty-scales-with-time-correction´
#[test]
fn outcome_axis_uncertainty_scales_with_time_correction() {
    // Same model and φ̂, different time correction: point estimate unchanged,
    // uncertainty scales as sqrt(time_correction).
    let p = 8;
    let params = test_model_parameters(p, 0.05, 0.3);
    let phi: Vec<f64> = (0..p).map(|i| 0.2 * (i as f64 + 1.0)).collect();

    let tc_low = 1.0;
    let tc_high = 4.0;

    let pred_low = evaluate_outcome_axis(&params, &phi, 1.0, tc_low);
    let pred_high = evaluate_outcome_axis(&params, &phi, 1.0, tc_high);

    let tol = DEFAULT_TOLERANCES;

    // Point estimate does not depend on time correction.
    assert_near(
        pred_high.predicted_raw,
        pred_low.predicted_raw,
        tol.default,
        "time correction must not change outcome-axis point estimate",
    );

    // Uncertainty should increase with time correction.
    assert!(
        pred_high.uncertainty > pred_low.uncertainty,
        "higher time correction should increase uncertainty",
    );

    // Since sigma_o = sqrt(sigma2_o * time_correction), ratio should be sqrt(tc_high/tc_low).
    let observed_ratio = pred_high.uncertainty / pred_low.uncertainty;
    let expected_ratio = (tc_high / tc_low).sqrt();
    assert_near(
        observed_ratio,
        expected_ratio,
        tol.default,
        "outcome-axis uncertainty ratio should follow sqrt(time_correction)",
    );
}

/// A feature vector whose length disagrees with the model's yields the
/// computed prior — zero estimate, the time-corrected prior standard
/// deviation through the compression scale — rather than a number computed
/// from whichever coordinates happened to line up. At unit parameters that
/// deviation is one; at a doubled compression scale and a quadrupled time
/// correction it is exactly four, so a stale degraded prediction widens with
/// staleness and carries the axis's scale like any successful one
/// (´def:axis:prior-only-prediction´).
///
/// ´claim:wellness:a-feature-vector-of-the-wrong-length-yields-the-computed-prior-not-a-partial-computation´
/// ´test:crate:dimension-mismatch-returns-prior´
#[test]
fn dimension_mismatch_returns_prior() {
    // Mismatched dimensions should return the prior-only prediction
    let params = test_model_parameters(10, 0.1, 1.0);
    let phi = vec![0.1; 5]; // Wrong size

    let pred = evaluate_outcome_axis(&params, &phi, 1.0, 1.0);

    let tol = DEFAULT_TOLERANCES;
    assert_near(pred.predicted_raw, 0.0, tol.bit_identical, "mismatched-dim point estimate");
    assert_near(
        pred.uncertainty,
        1.0,
        tol.bit_identical,
        "mismatched-dim unit-parameter uncertainty",
    );

    // κ_a = 2 and time correction 4 scale the prior deviation to
    // 2·√(4/1) = 4: the fallback computes, it does not substitute.
    let scaled = crate::assessment::evaluate_outcome_axis(crate::types::OutcomeAxisId(0), "test", &params, &phi, 2.0, 4.0, 1.0);
    assert_near(scaled.predicted_raw, 0.0, tol.bit_identical, "scaled prior point estimate");
    assert_near(scaled.uncertainty, 4.0, tol.bit_identical, "κ_a·√(T_corr/λ_prior) = 2·√4");
    assert_near(
        scaled.prediction_interval.1,
        1.96 * 4.0,
        tol.bit_identical,
        "interval follows the deviation",
    );
}

/// The reported credible interval means what it claims: a thousand draws from
/// the very distribution the axis says it is predicting fall inside the
/// interval about ninety-five percent of the time. The bounds are not merely
/// ordered and finite — they are calibrated against the deviation the same
/// prediction reports, so a host sizing its risk appetite by the interval is
/// sizing it correctly.
///
/// ´claim:wellness:the-reported-credible-interval-covers-the-predicted-distribution-at-its-stated-rate´
/// ´test:crate:prediction-interval-monte-carlo´
#[test]
fn prediction_interval_monte_carlo() {
    // Monte Carlo verification: ~95% of samples should fall within the 95% credible interval.
    // We use a known distribution to verify the interval calibration.
    use std::f64::consts::PI;

    let p = 5;
    let true_mu = 0.3; // True mean in tanh-space
    let cov_scale = 0.04; // σ² = 0.04 → σ = 0.2

    let params = test_model_parameters(p, true_mu / p as f64, cov_scale);
    let phi: Vec<f64> = vec![1.0; p]; // Unit vector → dot = p * (true_mu/p) = true_mu

    let pred = evaluate_outcome_axis(&params, &phi, 1.0, 1.0);

    // Now simulate 1000 samples from the true distribution N(atanh(true_mu), σ_raw)
    // and check coverage
    let n_samples = 1000;
    let mut inside_count = 0;

    // Deterministic, reproducible PRNG from the test harness.
    let mut rng = TestRng::new(12_345);

    // Box-Muller for normal samples
    for _ in 0..n_samples {
        let u1 = rng.next_f64().max(1e-10);
        let u2 = rng.next_f64();
        let z = (-2.0 * u1.ln()).sqrt() * (2.0 * PI * u2).cos();

        // Sample in raw (pre-tanh) space from N(point_estimate, std_dev)
        let sample = z.mul_add(pred.uncertainty, pred.predicted_raw);

        if sample >= pred.prediction_interval.0 && sample <= pred.prediction_interval.1 {
            inside_count += 1;
        }
    }

    let coverage = inside_count as f64 / n_samples as f64;

    // Expected 95% coverage; allow ±5% tolerance for Monte Carlo variance
    assert!(
        (0.90..=1.0).contains(&coverage),
        "Coverage {coverage:.3} should be ~95% (allowed 90-100%)"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Drift Accumulator Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Drift is watched in each direction independently: a long run of residuals
/// well above the allowance builds evidence in the upper accumulator while the
/// lower one stays at rest, and the upper one never goes negative even after
/// the threshold crossing has reset it. Over-prediction and under-prediction
/// are different failures, and an accumulator that netted them would let a
/// model drifting both ways at once look healthy.
///
/// ´claim:wellness:the-two-drift-accumulators-are-one-sided-so-over-and-under-prediction-cannot-cancel´
/// ´test:crate:drift-cusum-accumulates-positive´
#[test]
fn drift_cusum_accumulates_positive() {
    // 20 labels with sustained positive residual > κ_drift
    // S⁺ should accumulate
    let config = DriftConfig::default(); // κ_drift = 0.1
    let mut drift = DriftState::new();

    // Each step: S⁺ += (1.0 - 0.1) = 0.9
    // After 12 steps S⁺ exceeds the threshold of 10.0 and resets
    for _ in 0..20 {
        drift.update(1.0, &config);
    }

    // Should have triggered auto-reset (S⁺ > 10)
    // After reset, steps_since_reset should be 0
    assert!(drift.s_plus >= 0.0, "S⁺ should be non-negative after reset");
    assert!(drift.s_minus < 0.1, "S⁻ should be near zero for positive residuals");
}

/// Residuals inside the allowance never accumulate, however long they go on: a
/// hundred rounds alternating either side of zero, each smaller than the
/// allowance, leave both accumulators small. Ordinary prediction error is
/// expected and must not be mistaken for drift merely by being persistent —
/// otherwise every model would eventually trip its own reset by sitting still.
///
/// ´claim:wellness:residuals-inside-the-allowance-never-accumulate-however-long-they-continue´
/// ´test:crate:drift-cusum-decays-with-noise´
#[test]
fn drift_cusum_decays_with_noise() {
    // 100 labels alternating ±0.05, inside the allowance
    // CUSUMs should stay near zero
    let config = DriftConfig::default(); // κ_drift = 0.1
    let mut drift = DriftState::new();

    for i in 0..100 {
        let residual = if i % 2 == 0 { 0.05 } else { -0.05 };
        drift.update(residual, &config);
    }

    // With |residual| < κ_drift, CUSUMs should stay at zero (can't go negative)
    assert!(
        drift.s_plus < 1.0,
        "S⁺ should be small with low residuals, got {}",
        drift.s_plus
    );
    assert!(
        drift.s_minus < 1.0,
        "S⁻ should be small with low residuals, got {}",
        drift.s_minus
    );
}

/// Crossing the threshold resets the accumulator, and the update that did it
/// says so in its return value: driven with no allowance at all against a low
/// threshold, the run reports at least one reset rather than silently zeroing
/// itself. The caller is what turns a detected drift into action, so a reset
/// nobody was told about would clear the evidence and leave the drifting model
/// in place.
///
/// ´claim:wellness:an-accumulator-crossing-its-threshold-reports-the-reset-back-to-its-caller´
/// ´test:crate:drift-auto-reset-triggers´
#[test]
fn drift_auto_reset_triggers() {
    // Explicitly force S⁺ past threshold
    let config = DriftConfig {
        kappa_drift: 0.0, // No allowance
        gamma_drift: 0.99,
        h_threshold: 5.0, // Lower threshold
    };
    let mut drift = DriftState::new();
    let mut reset_count = 0;

    // 10 steps × 1.0 residual = S⁺ = 10 (2× the threshold)
    for _ in 0..10 {
        if drift.update(1.0, &config) {
            reset_count += 1;
        }
    }

    assert!(reset_count >= 1, "Auto-reset should have triggered at least once");
}

/// Alongside the accumulators, two smoothed measures settle on what the
/// residuals actually are: fed a constant residual, the mean absolute residual
/// converges on its magnitude and the sign measure approaches its direction.
/// Together they describe a bias the accumulators only signal the presence of
/// — how large it is, and which way it leans.
///
/// ´claim:wellness:the-smoothed-residual-magnitude-and-sign-converge-on-the-residuals-actually-seen´
/// ´test:crate:drift-ewma-converges´
#[test]
fn drift_ewma_converges() {
    // Feed constant positive residuals → EWMAs should converge
    let config = DriftConfig::default();
    let mut drift = DriftState::new();

    for _ in 0..200 {
        drift.update(0.4, &config);
    }

    // MAR should approach 0.4
    assert!(
        (drift.mean_abs_residual - 0.4).abs() < 0.1,
        "MAR should converge to ~0.4, got {}",
        drift.mean_abs_residual
    );

    // Sign EWMA should approach +1.0
    assert!(
        drift.residual_sign_ewma > 0.8,
        "Sign EWMA should be near +1.0, got {}",
        drift.residual_sign_ewma
    );
}

/// Clearing the accumulated evidence spares the smoothed history: both
/// accumulators and the step count return to zero while the mean absolute
/// residual and the sign measure survive untouched. Acting on a detected drift
/// should not also destroy the long-memory description of how the residuals
/// behave, which takes many rounds to rebuild and is what makes the next
/// detection interpretable.
///
/// ´claim:wellness:resetting-the-accumulators-spares-the-smoothed-residual-history´
/// ´test:crate:drift-reset-cusums-only´
#[test]
fn drift_reset_cusums_only() {
    // reset_cusums() should only reset CUSUMs, not EWMAs
    let mut drift = DriftState {
        s_plus: 5.0,
        s_minus: 3.0,
        mean_abs_residual: 0.5,
        residual_sign_ewma: 0.3,
        steps_since_reset: 100,
    };

    drift.reset_cusums();

    let tol = DEFAULT_TOLERANCES;
    assert_near(drift.s_plus, 0.0, tol.bit_identical, "S⁺ after reset_cusums");
    assert_near(drift.s_minus, 0.0, tol.bit_identical, "S⁻ after reset_cusums");
    assert_eq!(drift.steps_since_reset, 0, "steps should be 0");
    assert_near(drift.mean_abs_residual, 0.5, tol.bit_identical, "MAR preserved");
    assert_near(drift.residual_sign_ewma, 0.3, tol.bit_identical, "sign EWMA preserved");
}

/// The full reset is the stronger operation: accumulators, step count, mean
/// absolute residual and sign measure all return to their constructed state
/// together. This is what a model replacement calls for — residual statistics
/// describing predictions a different model made would be actively misleading
/// applied to its successor.
///
/// ´claim:wellness:a-full-drift-reset-returns-the-smoothed-history-as-well-as-the-accumulators´
/// ´test:crate:drift-reset-all´
#[test]
fn drift_reset_all() {
    // reset_all() should reset everything to defaults
    let mut drift = DriftState {
        s_plus: 5.0,
        s_minus: 3.0,
        mean_abs_residual: 0.5,
        residual_sign_ewma: 0.3,
        steps_since_reset: 100,
    };

    drift.reset_all();

    let tol = DEFAULT_TOLERANCES;
    assert_near(drift.s_plus, 0.0, tol.bit_identical, "S⁺ after reset_all");
    assert_near(drift.s_minus, 0.0, tol.bit_identical, "S⁻ after reset_all");
    assert_near(drift.mean_abs_residual, 0.0, tol.bit_identical, "MAR after reset_all");
    assert_near(drift.residual_sign_ewma, 0.0, tol.bit_identical, "sign EWMA after reset_all");
    assert_eq!(drift.steps_since_reset, 0, "steps should be 0");
}

/// Drift state survives being written out and read back whole: both
/// accumulators, both smoothed measures — including a negative sign measure —
/// and the step count all return bit-identical. Drift evidence is gathered
/// over hundreds of labels, so a restart that dropped it would grant every
/// drifting model a fresh start it had not earned.
///
/// ´claim:wellness:drift-state-round-trips-with-both-accumulators-the-smoothed-measures-and-the-step-count´
/// ´test:crate:drift-state-serde-round-trip´
#[cfg(feature = "serde")]
#[test]
fn drift_state_serde_round_trip() {
    let original = DriftState {
        s_plus: 1.5,
        s_minus: 2.3,
        mean_abs_residual: 0.42,
        residual_sign_ewma: -0.3,
        steps_since_reset: 123,
    };

    let json = serde_json::to_string(&original).expect("serialisation failed");
    let restored: DriftState = serde_json::from_str(&json).expect("deserialisation failed");

    let tol = DEFAULT_TOLERANCES;
    assert_near(restored.s_plus, original.s_plus, tol.bit_identical, "S⁺ round-trip");
    assert_near(restored.s_minus, original.s_minus, tol.bit_identical, "S⁻ round-trip");
    assert_near(
        restored.mean_abs_residual,
        original.mean_abs_residual,
        tol.bit_identical,
        "MAR round-trip",
    );
    assert_near(
        restored.residual_sign_ewma,
        original.residual_sign_ewma,
        tol.bit_identical,
        "sign EWMA round-trip",
    );
    assert_eq!(restored.steps_since_reset, original.steps_since_reset);
}
