// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The convergence guarantees everything above the baselines rests on,
//! established on the estimator alone — no subspace, no scoring pipeline,
//! no noise.
//!
//! A baseline is a running mean and spread that forget the past
//! geometrically, and almost everything the sentinel promises about
//! warm-up follows from what that single decay factor implies. Fed a
//! constant level the estimator closes on it at a rate the factor
//! dictates, so a warm-up budget can be computed from configuration
//! instead of discovered by running. The approach never rebounds and the
//! arrival is permanent while the input holds, so "converged" is a
//! well-defined moment rather than a lull. A longer memory buys steadiness
//! by taking longer to arrive, which is the trade the factor exists to
//! express, and a step change is adopted rather than resisted, because a
//! baseline models what is normal now.
//!
//! Two decisions are visible here rather than derived. The first batch
//! seeds the mean and the spread outright instead of blending against the
//! values a fresh estimator is constructed with: those are placeholders
//! chosen to keep early z-scores finite, and blending against them would
//! plant a bias that then has to decay away. And a batch is one step of
//! learning however many samples it carries, because the update consumes
//! the batch mean — which is why convergence is counted in rounds
//! throughout, and never in samples.
//!
//! Clipping appears once, at the end, for the property that motivates the
//! warm-up exemption: a ceiling can only slow the approach to a distant
//! level, never hasten it.
//!
//! # §-references
//!
//! - §ALGO S-7.1.1 — EWMA outlier filter
//! - ADR-S-013 — Warm-up convergence benchmark
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`ewma_cold_start_sets_mean_directly`] | convergence | The first batch seeds the baseline outright instead of being blended with the placeholder the baseline was constructed with. That placeholder is a stand-in chosen to keep early z-scores finite, not an observation, so blending against it would plant a bias that then has to decay away. |
//! | [`ewma_cold_start_sets_variance_from_batch`] | convergence | cites (´claim:convergence:the-first-batch-seeds-the-baseline-outright-instead-of-blending-with-the-placeholder´) |
//! | [`ewma_pure_convergence_rate`] | convergence | Fed a constant level, the baseline closes on it at the rate the forgetting factor dictates — the steps needed to reach a given relative error are what geometric decay predicts. Convergence time is therefore something that can be computed from configuration rather than discovered by running. |
//! | [`ewma_higher_lambda_converges_slower`] | convergence | A longer memory buys steadiness by taking longer to arrive: the slower-decaying baseline needs strictly more steps to reach the same relative error. This is the trade the forgetting factor exists to express, and it is why a warm-up budget is sized against the configured factor rather than fixed at some number of rounds. |
//! | [`ewma_convergence_independent_of_batch_size`] | convergence | A batch is one step of learning however many samples it carries: batches spanning a wide range of sizes all reach the same relative error in the same number of steps, because the update consumes the batch mean. Convergence is counted in rounds, not in samples. |
//! | [`ewma_error_decreases_monotonically`] | convergence | Approaching a constant level the error never rebounds: each step is a convex move toward the target and cannot carry the mean past it. A baseline that oscillated on the way in would leave every convergence test ambiguous about when it had arrived. |
//! | [`ewma_steady_state_is_stable`] | convergence | Once arrived, the baseline neither drifts nor oscillates — a long further run against the same level leaves it exactly there. Arrival is permanent while the input holds, so a later departure can be attributed to the input rather than to the estimator. |
//! | [`ewma_tracks_step_change`] | convergence | After settling on one level the baseline re-converges when the input steps to another, within a time the forgetting factor bounds. A baseline models what is normal now, so a genuine change of regime has to be adopted rather than resisted indefinitely. |
//! | [`ewma_variance_convergence`] | convergence | The spread converges on much the same schedule as the mean, since both are carried by the same decay factor. That matters because a z-score divides one by the other: a spread lagging far behind its mean would leave scores miscalibrated even after the level itself looked settled. |
//! | [`ewma_clipping_slows_convergence`] | clipping | A ceiling can only slow the approach to a distant level, never hasten it: when the target sits far above the current mean, the clip rejects the very batches that would move it. This is the cost the warm-up exemption exists to avoid paying while a baseline has not yet found its level. |

use crate::ewma::EwmaStats;

// ════════════════════════════════════════════════════════════
//  1. Cold start
// ════════════════════════════════════════════════════════════

/// The first batch seeds the baseline outright instead of being blended with
/// the placeholder the baseline was constructed with. That placeholder is a
/// stand-in chosen to keep early z-scores finite, not an observation, so
/// blending against it would plant a bias that then has to decay away.
///
/// ´claim:convergence:the-first-batch-seeds-the-baseline-outright-instead-of-blending-with-the-placeholder´
/// ´test:crate:ewma-cold-start-sets-mean-directly´
#[test]
fn ewma_cold_start_sets_mean_directly() {
    for lambda in [0.90, 0.95, 0.99] {
        let mut ewma = EwmaStats::new(lambda);
        assert!(!ewma.is_warm());

        ewma.update(&[42.0, 42.0], f64::INFINITY);
        assert!(ewma.is_warm());
        assert!(
            (ewma.mean() - 42.0).abs() < 1e-10,
            "λ={lambda}: cold start should set mean=42.0, got {}",
            ewma.mean()
        );
    }
}

/// The same seeding governs the spread: a first batch with structure in it sets
/// the variance from that batch rather than leaving the constructed placeholder
/// standing. This is the half clipping depends on, since the ceiling is defined
/// from the spread.
///
/// (´claim:convergence:the-first-batch-seeds-the-baseline-outright-instead-of-blending-with-the-placeholder´)
/// ´test:crate:ewma-cold-start-sets-variance-from-batch´
#[test]
fn ewma_cold_start_sets_variance_from_batch() {
    let mut ewma = EwmaStats::new(0.95);
    assert!((ewma.variance() - 1.0).abs() < f64::EPSILON, "placeholder should be 1.0");

    // Batch mean = 10.0, MSD = ((−1)² + 0² + 1²) / 3 = 2/3
    ewma.update(&[9.0, 10.0, 11.0], f64::INFINITY);
    let expected_var = 2.0 / 3.0;
    assert!(
        (ewma.variance() - expected_var).abs() < 1e-10,
        "cold start should set variance from batch: expected {expected_var}, got {}",
        ewma.variance()
    );
}

// ════════════════════════════════════════════════════════════
//  2. Mean convergence rate
// ════════════════════════════════════════════════════════════

/// Fed a constant level, the baseline closes on it at the rate the forgetting
/// factor dictates — the steps needed to reach a given relative error are what
/// geometric decay predicts. Convergence time is therefore something that can
/// be computed from configuration rather than discovered by running.
///
/// ´claim:convergence:the-baseline-closes-on-a-constant-level-at-the-rate-the-forgetting-factor-dictates´
/// ´test:crate:ewma-pure-convergence-rate´
#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ewma_pure_convergence_rate() {
    let lambda = 0.95_f64;
    let mut ewma = EwmaStats::new(lambda);

    let target = 5.0;
    ewma.update(&[target; 4], f64::INFINITY);
    assert!(
        (ewma.mean() - target).abs() < 1e-10,
        "cold→warm should set mean exactly, got {}",
        ewma.mean()
    );

    let new_target = 10.0;
    let new_values = [new_target; 4];

    let mut steps_to_converge = None;
    for step in 1..=200 {
        ewma.update(&new_values, f64::INFINITY);
        let error = (ewma.mean() - new_target).abs() / new_target;
        if error < 0.05 && steps_to_converge.is_none() {
            steps_to_converge = Some(step);
        }
    }

    let n = steps_to_converge.expect("should converge within 200 steps");
    let theory = 0.05_f64.log(lambda).ceil() as usize;

    assert!(
        n <= theory + 2,
        "pure EWMA should converge close to theory: got {n}, expected ~{theory}"
    );
}

/// A longer memory buys steadiness by taking longer to arrive: the
/// slower-decaying baseline needs strictly more steps to reach the same
/// relative error. This is the trade the forgetting factor exists to express,
/// and it is why a warm-up budget is sized against the configured factor rather
/// than fixed at some number of rounds.
///
/// ´claim:convergence:a-longer-memory-buys-steadiness-by-taking-longer-to-arrive´
/// ´test:crate:ewma-higher-lambda-converges-slower´
#[test]
fn ewma_higher_lambda_converges_slower() {
    fn steps_to_converge(lambda: f64) -> usize {
        let mut ewma = EwmaStats::new(lambda);
        ewma.update(&[1.0; 4], f64::INFINITY); // cold start

        let target = 10.0;
        let values = [target; 4];
        for step in 1..=1000 {
            ewma.update(&values, f64::INFINITY);
            if (ewma.mean() - target).abs() / target < 0.05 {
                return step;
            }
        }
        1001
    }

    let fast = steps_to_converge(0.90);
    let slow = steps_to_converge(0.99);
    assert!(slow > fast, "λ=0.99 should be slower than λ=0.90: fast={fast}, slow={slow}");
}

/// A batch is one step of learning however many samples it carries: batches
/// spanning a wide range of sizes all reach the same relative error in the same
/// number of steps, because the update consumes the batch mean. Convergence is
/// counted in rounds, not in samples.
///
/// ´claim:convergence:a-batch-is-one-step-of-learning-however-many-samples-it-carries´
/// ´test:crate:ewma-convergence-independent-of-batch-size´
#[test]
fn ewma_convergence_independent_of_batch_size() {
    let lambda = 0.95_f64;
    let target = 7.0;

    let mut results = Vec::new();
    for batch_size in [1, 4, 16, 64] {
        let mut ewma = EwmaStats::new(lambda);
        let values: Vec<f64> = vec![target; batch_size];

        ewma.update(&[0.1], f64::INFINITY);

        let mut steps = None;
        for step in 1..=200 {
            ewma.update(&values, f64::INFINITY);
            let error = (ewma.mean() - target).abs() / target;
            if error < 0.05 && steps.is_none() {
                steps = Some(step);
            }
        }
        results.push(steps.expect("should converge"));
    }

    let min = *results.iter().min().unwrap();
    let max = *results.iter().max().unwrap();
    assert!(
        max - min <= 1,
        "convergence should be independent of batch size: min={min}, max={max}, results={results:?}"
    );
}

// ════════════════════════════════════════════════════════════
//  3. Convergence quality
// ════════════════════════════════════════════════════════════

/// Approaching a constant level the error never rebounds: each step is a convex
/// move toward the target and cannot carry the mean past it. A baseline that
/// oscillated on the way in would leave every convergence test ambiguous about
/// when it had arrived.
///
/// ´claim:convergence:the-approach-to-a-constant-level-never-rebounds´
/// ´test:crate:ewma-error-decreases-monotonically´
#[test]
fn ewma_error_decreases_monotonically() {
    let lambda = 0.95_f64;
    let mut ewma = EwmaStats::new(lambda);
    ewma.update(&[1.0; 4], f64::INFINITY); // cold start at 1.0

    let target = 10.0;
    let values = [target; 4];
    let mut prev_error = f64::INFINITY;

    for step in 1..=100 {
        ewma.update(&values, f64::INFINITY);
        let error = (ewma.mean() - target).abs();
        assert!(
            error <= prev_error + 1e-12,
            "error increased at step {step}: {prev_error} → {error}"
        );
        prev_error = error;
    }
}

/// Once arrived, the baseline neither drifts nor oscillates — a long further
/// run against the same level leaves it exactly there. Arrival is permanent
/// while the input holds, so a later departure can be attributed to the input
/// rather than to the estimator.
///
/// ´claim:convergence:arrival-is-permanent-while-the-input-holds´
/// ´test:crate:ewma-steady-state-is-stable´
#[test]
fn ewma_steady_state_is_stable() {
    let lambda = 0.95_f64;
    let target = 7.5;
    let mut ewma = EwmaStats::new(lambda);

    // Converge fully.
    ewma.update(&[target; 4], f64::INFINITY);
    for _ in 0..200 {
        ewma.update(&[target; 4], f64::INFINITY);
    }

    // Run 100 more steps and verify no drift.
    for step in 1..=100 {
        ewma.update(&[target; 4], f64::INFINITY);
        let error = (ewma.mean() - target).abs();
        assert!(
            error < 1e-10,
            "steady state drifted at step {step}: mean={}, target={target}",
            ewma.mean()
        );
    }
}

/// After settling on one level the baseline re-converges when the input steps
/// to another, within a time the forgetting factor bounds. A baseline models
/// what is normal now, so a genuine change of regime has to be adopted rather
/// than resisted indefinitely.
///
/// ´claim:convergence:a-settled-baseline-re-converges-after-an-abrupt-level-shift´
/// ´test:crate:ewma-tracks-step-change´
#[test]
fn ewma_tracks_step_change() {
    let lambda = 0.95_f64;
    let mut ewma = EwmaStats::new(lambda);

    // Converge to 5.0.
    ewma.update(&[5.0; 4], f64::INFINITY);
    for _ in 0..200 {
        ewma.update(&[5.0; 4], f64::INFINITY);
    }
    assert!((ewma.mean() - 5.0).abs() < 1e-10, "should have converged to 5.0");

    // Step change to 15.0 — verify re-convergence.
    let new_target = 15.0;
    let mut reconverged = false;
    for step in 1..=200 {
        ewma.update(&[new_target; 4], f64::INFINITY);
        if (ewma.mean() - new_target).abs() / new_target < 0.05 {
            reconverged = true;
            // Verify it happened in a reasonable number of steps.
            assert!(step <= 80, "re-convergence took too long after step change: {step} steps");
            break;
        }
    }
    assert!(reconverged, "EWMA did not re-converge to {new_target}");
}

// ════════════════════════════════════════════════════════════
//  4. Variance convergence
// ════════════════════════════════════════════════════════════

/// The spread converges on much the same schedule as the mean, since both are
/// carried by the same decay factor. That matters because a z-score divides one
/// by the other: a spread lagging far behind its mean would leave scores
/// miscalibrated even after the level itself looked settled.
///
/// ´claim:convergence:the-spread-converges-on-the-same-schedule-as-the-mean´
/// ´test:crate:ewma-variance-convergence´
#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
fn ewma_variance_convergence() {
    let lambda = 0.95_f64;
    let mut ewma = EwmaStats::new(lambda);

    ewma.update(&[5.0, 5.0, 5.0, 5.0], f64::INFINITY);

    // Feed values with variance = 2/3 (values 9, 10, 11).
    let target_var = 2.0 / 3.0;
    let values = [9.0, 10.0, 11.0];

    let mut steps_to_converge = None;
    for step in 1..=200 {
        ewma.update(&values, f64::INFINITY);
        let error = (ewma.variance() - target_var).abs() / target_var;
        if error < 0.10 && steps_to_converge.is_none() {
            steps_to_converge = Some(step);
        }
    }

    let n = steps_to_converge.expect("variance should converge within 200 steps");
    let theory = 0.10_f64.log(lambda).ceil() as usize;
    assert!(n <= theory + 5, "variance convergence too slow: got {n}, expected ~{theory}");
}

// ════════════════════════════════════════════════════════════
//  5. Clipping interaction
// ════════════════════════════════════════════════════════════

/// A ceiling can only slow the approach to a distant level, never hasten it:
/// when the target sits far above the current mean, the clip rejects the very
/// batches that would move it. This is the cost the warm-up exemption exists to
/// avoid paying while a baseline has not yet found its level.
///
/// ´claim:clipping:a-ceiling-can-only-slow-the-approach-to-a-distant-level-never-hasten-it´
/// ´test:crate:ewma-clipping-slows-convergence´
#[test]
fn ewma_clipping_slows_convergence() {
    let lambda = 0.95_f64;

    let mut no_clip = EwmaStats::new(lambda);
    let mut clipped = EwmaStats::new(lambda);

    no_clip.update(&[1.0, 1.0, 1.0], f64::INFINITY);
    clipped.update(&[1.0, 1.0, 1.0], 3.0);

    let target = 10.0;
    let values = [target; 4];

    let mut no_clip_steps = None;
    let mut clipped_steps = None;

    for step in 1..=500 {
        no_clip.update(&values, f64::INFINITY);
        clipped.update(&values, 3.0);

        if (no_clip.mean() - target).abs() / target < 0.05 && no_clip_steps.is_none() {
            no_clip_steps = Some(step);
        }
        if (clipped.mean() - target).abs() / target < 0.05 && clipped_steps.is_none() {
            clipped_steps = Some(step);
        }
    }

    let n_no_clip = no_clip_steps.expect("no_clip should converge");
    let n_clipped = clipped_steps.unwrap_or(500);
    assert!(n_clipped >= n_no_clip, "clipping should not speed up convergence");
}
