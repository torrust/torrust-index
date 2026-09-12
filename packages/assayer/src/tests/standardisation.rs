// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`standardise_zero_mean_unit_variance`] | feature | A feature whose running statistics already say zero mean and unit variance comes through standardisation essentially unchanged, the only movement being the stability epsilon added to the divisor. The transform is therefore idempotent on already-standard input rather than shrinking it a little on every pass, which matters because assembled vectors are standardised once per assessment against statistics that hover near that state. |
//! | [`standardise_shift_and_scale`] | feature | Standardisation subtracts the running mean and divides by the square root of the running variance, so a value one deviation above the mean becomes one, one below becomes minus one, and two above becomes two. Every feature is thereby expressed in deviations of its own history, which is what lets a single set of model weights read slots whose raw units have nothing in common. |
//! | [`standardise_bias_excluded`] | feature | The leading bias slot passes through untouched even when its statistics claim a mean of ten and a large variance — figures that would badly distort it were they applied. The bias is a constant one by construction, and standardising a constant would destroy the intercept the model relies on, so the slot is excluded by position rather than trusted to have sensible statistics. |
//! | [`standardise_negative_variance_guard`] | feature | A variance estimate that has drifted below zero is floored at zero before the square root is taken, so the standardised vector stays finite instead of filling with NaN. Exponential smoothing can in principle carry a variance negative, and a single NaN would not stay local: it propagates through the dot product and corrupts the whole prediction, so the guard sits at the one place the square root is taken. |
//! | [`update_running_stats`] | feature | The running mean converges on the stream it is fed: a hundred observations clustered around three leave the estimate near three, having started at zero. The statistics are learned from live traffic rather than configured, so a feature whose typical magnitude shifts as the deployment changes is re-centred without anyone retuning it. |
//! | [`clip_extreme_values`] | feature | An observation far outside the current spread is clamped to the clip boundary before it touches the running statistics: a value of a hundred against a two-deviation clip moves the mean as if it had been two. Without the clamp a single freak observation would drag the reference so far that genuinely ordinary traffic afterwards would read as anomalous — the outlier would define the norm it was supposed to stand out from. |
//! | [`variance_floor_applied`] | feature | However little a feature actually varies, its recorded variance is held at or above the configured floor — here rising to meet the floor from beneath it under perfectly constant input. Standardisation divides by that figure, so a feature that has been flat for a long stretch would otherwise turn its first small deviation into an enormous standardised value. |
//! | [`extend_adds_priors`] | feature | Positions appended to a growing vector arrive with the mean and variance their declared feature class prescribes — a rate starting near a third with small spread, a binary indicator at a half with the variance of a fair coin — and the three parallel records stay the same length. A new feature must be standardised sensibly from its very first observation, and its class already says roughly where it will sit. |
//! | [`extend_refuses_desynchronising_extension`] | feature | An extension that would leave the standardisation vectors a different length than the model dimension they extend alongside is refused loudly rather than accepted in silence. The class-assignment requirement (´req:standardisation:class-assignment´) names no enforcer, and this assertion is it: every lifecycle path that extends the models routes its new classes through here, so a caller that counts positions one way for the models and another way for the statistics becomes a failing test instead of a quiet corruption discovered by whichever assessment standardises against the truncated vectors first. |
//! | [`restandardise_mismatch_bounded_at_fifty_labels`] | feature | The "Re-Standardisation Mismatch Bound" (´bound:standardisation:restandardisation´). Store an assessment's assembled φ under standardisation statistics `(μ̄, v̄) = (2.0, 4.0)` on one ZScore feature (σ = 2). Then process 50 further observations under the default `gamma_std = 0.9998` (´alg:standardisation:label-time-procedure´) — the same smoothing rate the engine ships — with a bounded-in-support jittered stream. Re-standardise the same raw value against the updated `(μ̄, v̄)` at label time. For a typical 2σ feature (raw = 6.0, giving φ_rec = 1.0 on the fresh stats), the README scenario bounds the mismatch at 0.14. This reproduces that ceiling directly on the primitives the label-time pipeline calls into, confirming the short-window drift per label stays within the documented envelope. Mechanic: under `gamma_std = 0.9998`, each label contributes ≈ 2e-4 to the mean. Across 50 labels on a stream jittered around raw = 10, the mean shifts by ≲ 0.08 and the variance by a similar small fraction — far less than the 0.14 budget the spec allots for a 2σ feature. A stored vector and its outcome are separated by up to fifty further observations, and the shipped smoothing rate is slow enough that re-standardising the same raw value against the drifted statistics moves it by well under the documented ceiling. Learning can therefore work from the raw value at label time rather than having to preserve the exact statistics each assessment was standardised under. |

//! Crate-level tests for feature standardisation (´sec:standardisation:online´).
//!
//! Features arrive on wildly different scales — z-scores near unit variance,
//! log-scaled counts in the single digits, fractions in the unit interval — and
//! the model needs them comparable. Standardisation subtracts a running mean and
//! divides by a running deviation that are themselves maintained online, so
//! these tests fix both halves: the transform applied to a vector, and the EWMA
//! that keeps the statistics it is applied against.
//!
//! Two defences run through the update path and are pinned here. Extreme
//! observations are clipped before they move the statistics, so one outlier
//! cannot drag the reference it will later be measured against; and the
//! variance carries a floor and a guard against having drifted below zero, so
//! the division can never produce a non-finite feature.

use crate::feature::standardisation::{
    FeatureClass, StandardisationConfig, extend_standardisation, standardise_in_place, update_standardisation,
};
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_finite, assert_near};

// ─────────────────────────────────────────────────────────────────────────────
// standardise_in_place tests
// ─────────────────────────────────────────────────────────────────────────────

/// A feature whose running statistics already say zero mean and unit variance
/// comes through standardisation essentially unchanged, the only movement being
/// the stability epsilon added to the divisor. The transform is therefore
/// idempotent on already-standard input rather than shrinking it a little on
/// every pass, which matters because assembled vectors are standardised once
/// per assessment against statistics that hover near that state.
///
/// ´claim:feature:standardising-an-already-standard-feature-leaves-it-where-it-is´
/// ´test:crate:standardise-zero-mean-unit-variance´
#[test]
fn standardise_zero_mean_unit_variance() {
    // Features already standardised should be unchanged (except for epsilon effect)
    let mut phi = vec![1.0, 0.5, -0.5, 1.5, -1.5];
    let means = vec![0.0; 5];
    let variances = vec![1.0; 5];

    standardise_in_place(&mut phi, &means, &variances, 1e-8);

    // Index 0 (bias) unchanged
    assert_near(phi[0], 1.0, DEFAULT_TOLERANCES.bit_identical, "bias preserved");
    // Other indices: (x - 0) / (1 + ε) ≈ x
    let tol = DEFAULT_TOLERANCES.ewma;
    assert_near(phi[1], 0.5, tol, "phi[1]");
    assert_near(phi[2], -0.5, tol, "phi[2]");
    assert_near(phi[3], 1.5, tol, "phi[3]");
    assert_near(phi[4], -1.5, tol, "phi[4]");
}

/// Standardisation subtracts the running mean and divides by the square root of
/// the running variance, so a value one deviation above the mean becomes one,
/// one below becomes minus one, and two above becomes two. Every feature is
/// thereby expressed in deviations of its own history, which is what lets a
/// single set of model weights read slots whose raw units have nothing in
/// common.
///
/// ´claim:feature:standardisation-subtracts-the-running-mean-and-divides-by-the-running-deviation´
/// ´test:crate:standardise-shift-and-scale´
#[test]
fn standardise_shift_and_scale() {
    // Mean = 5, Variance = 4 (σ = 2)
    // Input: 7 → (7 - 5) / 2 = 1
    let mut phi = vec![1.0, 7.0, 3.0, 9.0];
    let means = vec![0.0, 5.0, 5.0, 5.0];
    let variances = vec![0.0, 4.0, 4.0, 4.0];

    standardise_in_place(&mut phi, &means, &variances, 0.0);

    let tol = DEFAULT_TOLERANCES.default;
    assert_near(phi[0], 1.0, DEFAULT_TOLERANCES.bit_identical, "bias unchanged");
    assert_near(phi[1], 1.0, tol, "(7-5)/2");
    assert_near(phi[2], -1.0, tol, "(3-5)/2");
    assert_near(phi[3], 2.0, tol, "(9-5)/2");
}

/// The leading bias slot passes through untouched even when its statistics
/// claim a mean of ten and a large variance — figures that would badly distort
/// it were they applied. The bias is a constant one by construction, and
/// standardising a constant would destroy the intercept the model relies on,
/// so the slot is excluded by position rather than trusted to have sensible
/// statistics.
///
/// ´claim:feature:the-bias-slot-is-excluded-from-standardisation-whatever-its-statistics-say´
/// ´test:crate:standardise-bias-excluded´
#[test]
fn standardise_bias_excluded() {
    let mut phi = vec![1.0, 2.0, 3.0];
    let means = vec![10.0, 1.0, 2.0]; // Bias mean = 10 (would affect if not skipped)
    let variances = vec![100.0, 1.0, 1.0];

    standardise_in_place(&mut phi, &means, &variances, 0.0);

    // Bias (index 0) should remain 1.0
    assert_near(phi[0], 1.0, DEFAULT_TOLERANCES.bit_identical, "bias excluded");
}

/// A variance estimate that has drifted below zero is floored at zero before
/// the square root is taken, so the standardised vector stays finite instead of
/// filling with NaN. Exponential smoothing can in principle carry a variance
/// negative, and a single NaN would not stay local: it propagates through the
/// dot product and corrupts the whole prediction, so the guard sits at the one
/// place the square root is taken.
///
/// ´claim:feature:a-variance-that-has-drifted-below-zero-is-floored-before-the-square-root´
/// ´test:crate:standardise-negative-variance-guard´
#[test]
fn standardise_negative_variance_guard() {
    let mut phi = vec![1.0, 5.0, 3.0];
    let means = vec![0.0, 2.0, 1.0];
    let variances = vec![1.0, -0.5, -1e-10]; // Negative variances from EWMA drift

    standardise_in_place(&mut phi, &means, &variances, 1e-8);

    // Bias unchanged.
    assert_near(phi[0], 1.0, DEFAULT_TOLERANCES.bit_identical, "bias unchanged");
    // With .max(0.0): sqrt(max(-0.5, 0)) + ε = ε, so (5 - 2) / ε.
    // The key invariant: no NaN.
    assert_finite(&phi, "standardised phi under negative variance");
}

// ─────────────────────────────────────────────────────────────────────────────
// update_standardisation tests
// ─────────────────────────────────────────────────────────────────────────────

/// The running mean converges on the stream it is fed: a hundred observations
/// clustered around three leave the estimate near three, having started at
/// zero. The statistics are learned from live traffic rather than configured,
/// so a feature whose typical magnitude shifts as the deployment changes is
/// re-centred without anyone retuning it.
///
/// ´claim:feature:the-running-mean-converges-on-the-stream-it-is-fed´
/// ´test:crate:update-running-stats´
#[test]
fn update_running_stats() {
    let mut means = vec![0.0; 5];
    let mut variances = vec![1.0; 5];
    let config = StandardisationConfig {
        gamma_std: 0.9,
        epsilon: 1e-8,
        n_clip: 10.0,
        v_floor: 0.01,
        ..StandardisationConfig::default()
    };

    // Send 100 observations with mean 3.0 and variance ~1.0
    for _ in 0..100 {
        let phi_raw = vec![1.0, 3.0, 3.2, 2.8, 3.1];
        update_standardisation(&phi_raw, &mut means, &mut variances, &config);
    }

    // After 100 updates with γ=0.9, mean should be close to 3.0
    // (1 - 0.9^100) ≈ 1.0, so converged to input mean
    assert_finite(&means, "EMA means");
    assert_finite(&variances, "EMA variances");
    assert_near(means[1], 3.0, 0.5, "mean[1] converged");
    assert_near(means[2], 3.0, 0.5, "mean[2] converged");
}

/// An observation far outside the current spread is clamped to the clip
/// boundary before it touches the running statistics: a value of a hundred
/// against a two-deviation clip moves the mean as if it had been two. Without
/// the clamp a single freak observation would drag the reference so far that
/// genuinely ordinary traffic afterwards would read as anomalous — the outlier
/// would define the norm it was supposed to stand out from.
///
/// ´claim:feature:an-extreme-observation-is-clipped-before-it-moves-the-running-statistics´
/// ´test:crate:clip-extreme-values´
#[test]
fn clip_extreme_values() {
    let mut means = vec![0.0; 3];
    let mut variances = vec![1.0; 3];
    let config = StandardisationConfig {
        gamma_std: 0.5,
        epsilon: 1e-8,
        n_clip: 2.0, // Clip at ±2σ
        v_floor: 0.01,
        ..StandardisationConfig::default()
    };

    // Extreme value at index 1: 100.0 (way above clip threshold)
    let phi_raw = vec![1.0, 100.0, 0.5];
    update_standardisation(&phi_raw, &mut means, &mut variances, &config);

    // Mean should not jump to 50 (would happen without clipping)
    // With clip at 2σ=2, extreme value is clipped to 2.0
    // Update: 0.5 * 0 + 0.5 * 2 = 1.0
    assert_near(means[1], 1.0, 0.5, "mean[1] clipped");
}

/// However little a feature actually varies, its recorded variance is held at
/// or above the configured floor — here rising to meet the floor from beneath
/// it under perfectly constant input. Standardisation divides by that figure,
/// so a feature that has been flat for a long stretch would otherwise turn its
/// first small deviation into an enormous standardised value.
///
/// ´claim:feature:the-running-variance-is-held-at-or-above-the-configured-floor´
/// ´test:crate:variance-floor-applied´
#[test]
fn variance_floor_applied() {
    let mut means = vec![0.0; 3];
    let mut variances = vec![0.0001; 3]; // Very low variance
    let config = StandardisationConfig {
        gamma_std: 0.9,
        epsilon: 1e-8,
        n_clip: 10.0,
        v_floor: 0.01,
        ..StandardisationConfig::default()
    };

    // Constant observations
    let phi_raw = vec![1.0, 5.0, 5.0];
    update_standardisation(&phi_raw, &mut means, &mut variances, &config);

    // Variance should be at least v_floor
    assert!(variances[1] >= config.v_floor);
    assert!(variances[2] >= config.v_floor);
}

// ─────────────────────────────────────────────────────────────────────────────
// extend_standardisation tests
// ─────────────────────────────────────────────────────────────────────────────

/// Positions appended to a growing vector arrive with the mean and variance
/// their declared feature class prescribes — a rate starting near a third with
/// small spread, a binary indicator at a half with the variance of a fair coin
/// — and the three parallel records stay the same length. A new feature must be
/// standardised sensibly from its very first observation, and its class already
/// says roughly where it will sit.
///
/// ´claim:feature:new-positions-start-from-the-prior-of-their-declared-feature-class´
/// ´test:crate:extend-adds-priors´
#[test]
fn extend_adds_priors() {
    let mut means = vec![1.0, 0.0];
    let mut variances = vec![0.0, 1.0];
    let mut classes = vec![FeatureClass::Bias, FeatureClass::ZScore];

    let new_classes = vec![FeatureClass::RateOrFraction, FeatureClass::Binary];
    extend_standardisation(&mut means, &mut variances, &mut classes, &new_classes, 4);

    assert_eq!(means.len(), 4);
    assert_eq!(variances.len(), 4);
    assert_eq!(classes.len(), 4);

    // RateOrFraction: (0.3, 0.05) (´tab:standardisation:class-priors´)
    let tol = DEFAULT_TOLERANCES.bit_identical;
    assert_near(means[2], 0.3, tol, "RateOrFraction prior mean");
    assert_near(variances[2], 0.05, tol, "RateOrFraction prior variance");

    // Binary: (0.5, 0.25)
    assert_near(means[3], 0.5, tol, "Binary prior mean");
    assert_near(variances[3], 0.25, tol, "Binary prior variance");
}

/// An extension that would leave the standardisation vectors a different
/// length than the model dimension they extend alongside is refused loudly
/// rather than accepted in silence. The class-assignment requirement
/// (´req:standardisation:class-assignment´) names no enforcer, and this
/// assertion is it: every lifecycle path that extends the models routes
/// its new classes through here, so a caller that counts positions one way
/// for the models and another way for the statistics becomes a failing
/// test instead of a quiet corruption discovered by whichever assessment
/// standardises against the truncated vectors first.
///
/// ´claim:feature:a-desynchronising-standardisation-extension-is-refused-loudly´
/// ´test:crate:extend-refuses-desynchronising-extension´
#[test]
#[should_panic(expected = "standardisation extension desynchronised from the model dimension")]
fn extend_refuses_desynchronising_extension() {
    let mut means = vec![1.0, 0.0];
    let mut variances = vec![0.0, 1.0];
    let mut classes = vec![FeatureClass::Bias, FeatureClass::ZScore];

    // The models were extended by two positions (dimension 4); only one
    // class is handed over.
    let new_classes = vec![FeatureClass::Binary];
    extend_standardisation(&mut means, &mut variances, &mut classes, &new_classes, 4);
}

// ─────────────────────────────────────────────────────────────────────────────
// Re-standardisation mismatch bound (´bound:standardisation:restandardisation´)
// ─────────────────────────────────────────────────────────────────────────────

/// The "Re-Standardisation Mismatch Bound" (´bound:standardisation:restandardisation´).
///
/// Store an assessment's assembled φ under standardisation statistics
/// `(μ̄, v̄) = (2.0, 4.0)` on one ZScore feature (σ = 2). Then process 50
/// further observations under the default `gamma_std = 0.9998` (´alg:standardisation:label-time-procedure´)
/// — the same smoothing rate the engine ships — with a bounded-in-support
/// jittered stream. Re-standardise the same raw value against the updated
/// `(μ̄, v̄)` at label time. For a typical 2σ feature (raw = 6.0, giving
/// φ_rec = 1.0 on the fresh stats), the README scenario bounds the mismatch
/// at 0.14. This reproduces that ceiling directly on the primitives the
/// label-time pipeline calls into, confirming the short-window drift per
/// label stays within the documented envelope.
///
/// Mechanic: under `gamma_std = 0.9998`, each label contributes ≈ 2e-4 to
/// the mean. Across 50 labels on a stream jittered around raw = 10, the mean
/// shifts by ≲ 0.08 and the variance by a similar small fraction — far less
/// than the 0.14 budget the spec allots for a 2σ feature.
///
/// A stored vector and its outcome are separated by up to fifty further
/// observations, and the shipped smoothing rate is slow enough that
/// re-standardising the same raw value against the drifted statistics moves it
/// by well under the documented ceiling. Learning can therefore work from the
/// raw value at label time rather than having to preserve the exact statistics
/// each assessment was standardised under.
///
/// ´claim:feature:re-standardising-a-stored-value-fifty-labels-later-stays-within-the-documented-ceiling´
/// ´test:crate:restandardise-mismatch-bounded-at-fifty-labels´
#[test]
fn restandardise_mismatch_bounded_at_fifty_labels() {
    // Indices: 0 = bias, 1 = the ZScore feature under test.
    let mut means = vec![0.0, 2.0];
    let mut variances = vec![0.0, 4.0];
    let config = StandardisationConfig::default(); // gamma_std = 0.9998 (´alg:standardisation:label-time-procedure´)

    // φ_rec at assessment time: raw value = 6.0 (2σ above μ̄ = 2.0 with σ = 2).
    let raw_value = 6.0_f64;
    let mut phi_rec = vec![1.0, raw_value];
    standardise_in_place(&mut phi_rec, &means, &variances, config.epsilon);

    let tol_bit = DEFAULT_TOLERANCES.standardisation;
    assert_near(phi_rec[1], 2.0, tol_bit, "φ_rec = (6 − 2) / sqrt(4) = 2.0");

    // 50 labels of streaming updates against a near-stationary workload
    // (mean ≈ 2, σ ≈ 2 — the same regime the stored φ was standardised
    // under). This is the adjacent-distribution case the 0.14 bound is
    // calibrated for; under large distribution shifts the bound would
    // require proportionally more labels to absorb.
    let mut rng = TestRng::new(0x5E57_AB1E_4242_u64);
    for _ in 0..50 {
        // Uniform in [−2, 6] — mean 2, range ±4 around μ̄.
        let sample = rng.next_f64().mul_add(8.0, -2.0);
        let phi_raw = vec![1.0, sample];
        update_standardisation(&phi_raw, &mut means, &mut variances, &config);
    }

    assert_finite(&means, "means after 50 labels");
    assert_finite(&variances, "variances after 50 labels");

    // Evolved stats should still be near the initial (μ̄, v̄) — per-label
    // shift under gamma_std = 0.9998 is ≈ 2e-4.
    let delta_mu = (means[1] - 2.0).abs();
    let delta_v = (variances[1] - 4.0).abs();
    assert!(
        delta_mu < 0.10,
        "μ̄ shift over 50 labels should stay small (got {delta_mu:.4}); the scenario presumes gamma_std ≈ 0.9998"
    );
    assert!(delta_v < 1.0, "v̄ shift over 50 labels should stay bounded (got {delta_v:.4})");

    // φ_lab at label time: re-standardise the same raw value against
    // the evolved (μ̄, v̄).
    let mut phi_lab = vec![1.0, raw_value];
    standardise_in_place(&mut phi_lab, &means, &variances, config.epsilon);

    let mismatch = (phi_lab[1] - phi_rec[1]).abs();
    assert_finite(&phi_lab, "φ_lab after re-standardisation");
    assert!(
        mismatch < 0.14,
        "|φ_lab − φ_rec| = {mismatch:.6} must stay below the 0.14 ceiling \
         for a typical 2σ feature across 50 labels (bound:standardisation:restandardisation)"
    );

    // Bias index is never re-standardised; it round-trips verbatim.
    assert_near(
        phi_lab[0],
        1.0,
        DEFAULT_TOLERANCES.bit_identical,
        "bias unchanged under re-standardisation",
    );
}
