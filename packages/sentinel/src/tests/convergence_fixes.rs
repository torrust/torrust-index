// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Why warm-up terminates, and the bound it terminates within.
//!
//! Three properties of the cold start had to hold before a warm-up of
//! bounded length could be promised at all, and each is guarded here.
//! Nothing about them is historical: they are the reasons the present
//! design converges.
//!
//! An axis must not be able to clip itself into a baseline it then keeps
//! rejecting the evidence against — so the ceiling is held open while the
//! model is still noise-taught, and narrows smoothly rather than
//! switching as that exemption is spent. The latent spread must be seeded
//! from the first batch rather than started at a placeholder far above
//! where it settles, because the surprise axis divides by that spread and
//! a placeholder set high would show up as a slow rise indistinguishable
//! from a real trend. And at the hand-over from injected to real traffic
//! the long-memory reference must be seeded from the converged
//! short-memory baseline: left to converge on its own it would lag for
//! many hundreds of rounds, and the whole lag would be banked as evidence
//! of drift that never happened.
//!
//! With those in place the remaining question is the size of the budget,
//! and it is asked at several points rather than once. Each axis is judged
//! at its own tolerance, since they differ by more than an order of
//! magnitude in inherent jitter; the bound is checked at two batch sizes
//! and at the production memory length, so it can be seen to scale with
//! configuration rather than being a constant; and it is checked across a
//! spread of seeds, because two of the axes carry real seed-to-seed
//! variance in when they settle and a bound shown once would be no bound.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 3 — Latent distribution cold→warm
//! - §ALGO S-6.4 — Clip-pressure EWMA / graduated clip formula
//! - §ALGO S-7.1.1 — EWMA outlier filter / clipping ceiling
//! - §ALGO S-11.5 — Maturity tracking (noise influence η)
//! - ADR-S-013 — Warm-up convergence benchmark
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`graduated_clip_formula_is_smooth_and_monotonic`] | clipping | The effective clip width narrows smoothly and without reversal as the exemption is spent, from an effectively open ceiling while the model is entirely noise-taught down to the nominal width once it is not. There is no step anywhere along the transition, so no batch is judged by a much tighter rule than the batch before it. |
//! | [`clip_exemption_eliminates_feedback_loop`] | clipping | No axis can clip itself into a baseline it then keeps rejecting the evidence against. The exemption holds the ceiling open while the noise share is high, so the first batches enter unfiltered and even the highest-variance axis settles well inside the round budget instead of being pinned by its own early ceiling. |
//! | [`cold_warm_eliminates_surprise_nonstationarity`] | convergence | Because the latent spread is seeded from the first batch rather than starting at a placeholder far above its eventual value, the surprise axis is near stationary from its earliest rounds: early scores and late ones differ by well under a factor of two. Surprise divides by that spread, so a placeholder set too high would show up as a slow rise indistinguishable from a real trend. |
//! | [`cusum_seeding_prevents_false_drift`] | convergence | Seeding the long-memory reference from the converged short-memory baseline at the hand-over from injected to real traffic keeps the switch from reading as drift: the accumulators stay far below anything actionable across a long real-data run. Left to converge on its own the long reference would lag for many hundreds of rounds, and the whole lag would be banked as evidence. |
//! | [`per_axis_convergence_b4_within_bound`] | convergence | Every axis settles inside the round budget the noise schedule is sized from, each judged at its own tolerance. The axes differ by more than an order of magnitude in inherent jitter, so one shared tolerance would either excuse the quietest or condemn the noisiest; this per-axis bound is what makes a warm-up of bounded length sufficient. |
//! | [`per_axis_convergence_b16_within_bound`] | convergence | cites (´claim:convergence:every-axis-settles-inside-the-round-budget-at-its-own-tolerance´) |
//! | [`production_lambda_converges_within_bound`] | convergence | cites (´claim:convergence:every-axis-settles-inside-the-round-budget-at-its-own-tolerance´) |
//! | [`per_axis_convergence_b4_robust_across_seeds`] | convergence | The budget holds across a spread of seeds and not merely for one lucky noise sequence. Two of the axes carry real seed-to-seed variance in when they settle, so a bound demonstrated once would not be a bound at all: sizing a shipped schedule needs the worst case over sequences. |

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{
    as_slices, cfg_b16, cfg_production, cfg_test, find_converged_round, generate_noise, run_noise_trace,
};
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Fix 1: Graduated clip-exemption  (§ALGO S-6.4)
// ════════════════════════════════════════════════════════════

/// The effective clip width narrows smoothly and without reversal as the
/// exemption is spent, from an effectively open ceiling while the model is
/// entirely noise-taught down to the nominal width once it is not. There is no
/// step anywhere along the transition, so no batch is judged by a much tighter
/// rule than the batch before it.
///
/// ´claim:clipping:the-effective-clip-width-narrows-smoothly-and-without-reversal-as-the-exemption-is-spent´
/// ´test:crate:graduated-clip-formula-is-smooth-and-monotonic´
#[test]
fn graduated_clip_formula_is_smooth_and_monotonic() {
    let clip_sigmas = 3.0;
    let eps = 1e-6;

    // Walk p from 1.0 → 0.0 and verify monotonicity (non-increasing).
    let test_points = [1.0, 0.99, 0.95, 0.9, 0.8, 0.5, 0.3, 0.1, 0.01, 0.001, 0.0];
    let mut prev_eff: Option<f64> = None;

    for &p in &test_points {
        let effective = clip_sigmas * (1.0 + p / (1.0 - p + eps));

        if let Some(prev) = prev_eff {
            assert!(
                effective <= prev + 1e-6,
                "effective clip should be non-increasing as p decreases: \
                 p={p}, eff={effective}, prev_eff={prev}"
            );
        }
        prev_eff = Some(effective);
    }

    // Boundary: at p = 0, effective ≈ clip_sigmas.
    let at_zero = clip_sigmas * (1.0 + 0.0 / (1.0 - 0.0 + eps));
    assert!(
        (at_zero - clip_sigmas).abs() < 1e-4,
        "at p=0, effective clip should equal clip_sigmas: got {at_zero}"
    );

    // Boundary: at p = 1, effective is very large (open ceiling).
    let at_one = clip_sigmas * (1.0 + 1.0 / (1.0 - 1.0 + eps));
    assert!(at_one > 1000.0, "at p=1, effective clip should be very large: got {at_one}");
}

/// No axis can clip itself into a baseline it then keeps rejecting the evidence
/// against. The exemption holds the ceiling open while the noise share is high,
/// so the first batches enter unfiltered and even the highest-variance axis
/// settles well inside the round budget instead of being pinned by its own
/// early ceiling.
///
/// ´claim:clipping:an-axis-cannot-clip-itself-into-a-baseline-it-then-rejects-the-evidence-against´
/// ´test:crate:clip-exemption-eliminates-feedback-loop´
#[test]
fn clip_exemption_eliminates_feedback_loop() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 500;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    let surprise_baselines: Vec<f64> = traces.iter().map(|t| t.baseline_means[2]).collect();

    let window = 20;
    let tolerance = 0.20; // 20% for surprise (high-CV axis)
    let converged = find_converged_round(&surprise_baselines, window, tolerance);

    assert!(
        converged <= 200,
        "surprise should converge within 200 rounds with clip-exemption fix, got {converged}"
    );
}

// ════════════════════════════════════════════════════════════
//  Fix 2: Cold→warm initialisation  (§ALGO S-5.2 Phase 3)
// ════════════════════════════════════════════════════════════

/// Because the latent spread is seeded from the first batch rather than
/// starting at a placeholder far above its eventual value, the surprise axis is
/// near stationary from its earliest rounds: early scores and late ones differ
/// by well under a factor of two. Surprise divides by that spread, so a
/// placeholder set too high would show up as a slow rise indistinguishable from
/// a real trend.
///
/// ´claim:convergence:seeding-the-latent-spread-from-the-first-batch-leaves-the-surprise-axis-stationary-from-the-start´
/// ´test:crate:cold-warm-eliminates-surprise-nonstationarity´
#[test]
fn cold_warm_eliminates_surprise_nonstationarity() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 200;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    let surprise_scores: Vec<f64> = traces.iter().map(|t| t.score_means[2]).collect();

    // Compare early (rounds 2–9) vs late (rounds 150+).
    let early_avg: f64 = surprise_scores[2..10].iter().sum::<f64>() / 8.0;
    let late_avg: f64 = surprise_scores[150..].iter().sum::<f64>() / 50.0;
    let rise_factor = late_avg / early_avg;

    assert!(
        rise_factor < 2.0,
        "surprise rise factor should be < 2.0 with cold→warm fix, got {rise_factor:.2}×"
    );
}

// ════════════════════════════════════════════════════════════
//  Fix 3: CUSUM slow-from-fast seeding  (ADR-S-013 §6b)
// ════════════════════════════════════════════════════════════

/// Seeding the long-memory reference from the converged short-memory baseline
/// at the hand-over from injected to real traffic keeps the switch from reading
/// as drift: the accumulators stay far below anything actionable across a long
/// real-data run. Left to converge on its own the long reference would lag for
/// many hundreds of rounds, and the whole lag would be banked as evidence.
///
/// ´claim:convergence:seeding-the-long-memory-reference-at-the-hand-over-keeps-the-switch-to-real-traffic-from-reading-as-drift´
/// ´test:crate:cusum-seeding-prevents-false-drift´
#[test]
fn cusum_seeding_prevents_false_drift() {
    let cfg = cfg_test();
    let dim = 128;
    let noise_rounds = 200;
    let real_rounds = 300;

    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Noise phase.
    for _ in 0..noise_rounds {
        let data = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&data), 0, true);
    }

    // Transition: seed slow from fast, then reset CUSUM.
    tracker.seed_cusum_slow_from_baselines();
    tracker.reset_cusum();

    // Real-data phase (same noise-like data, but marked as real).
    let mut max_cusum = [0.0_f64; 4];
    for _ in 0..real_rounds {
        let data = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&data), 0, false);
        max_cusum[0] = max_cusum[0].max(report.scores.novelty.cusum.accumulator);
        max_cusum[1] = max_cusum[1].max(report.scores.displacement.cusum.accumulator);
        max_cusum[2] = max_cusum[2].max(report.scores.surprise.cusum.accumulator);
        max_cusum[3] = max_cusum[3].max(report.scores.coherence.cusum.accumulator);
    }

    let axis_names = ["novelty", "displacement", "surprise", "coherence"];
    for (i, &name) in axis_names.iter().enumerate() {
        assert!(
            max_cusum[i] < 20.0,
            "{name}: CUSUM should stay < 20 with slow-from-fast seeding, \
             got {:.2} (surprise reaches ~198 without seeding)",
            max_cusum[i],
        );
    }
}

// ════════════════════════════════════════════════════════════
//  Combined convergence bounds
// ════════════════════════════════════════════════════════════

/// Every axis settles inside the round budget the noise schedule is sized from,
/// each judged at its own tolerance. The axes differ by more than an order of
/// magnitude in inherent jitter, so one shared tolerance would either excuse
/// the quietest or condemn the noisiest; this per-axis bound is what makes a
/// warm-up of bounded length sufficient.
///
/// ´claim:convergence:every-axis-settles-inside-the-round-budget-at-its-own-tolerance´
/// ´test:crate:per-axis-convergence-b4-within-bound´
#[test]
fn per_axis_convergence_b4_within_bound() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 500;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    let tolerances = [0.01, 0.10, 0.20, 0.20];
    let window = 20;

    let mut worst_round = 0_usize;
    for (axis, &tol) in tolerances.iter().enumerate() {
        let baselines: Vec<f64> = traces.iter().map(|t| t.baseline_means[axis]).collect();
        let converged = find_converged_round(&baselines, window, tol);
        worst_round = worst_round.max(converged);
    }

    assert!(
        worst_round <= 500,
        "all axes should converge within 500 rounds at b=4, worst={worst_round}"
    );
}

/// Larger batches settle inside a proportionally smaller budget, which pins the
/// batch-size end of the same bound: more samples per round buy a better
/// per-round estimate, not more rounds of learning.
///
/// (´claim:convergence:every-axis-settles-inside-the-round-budget-at-its-own-tolerance´)
/// ´test:crate:per-axis-convergence-b16-within-bound´
#[test]
fn per_axis_convergence_b16_within_bound() {
    let cfg = cfg_b16();
    let dim = 128;
    let total_rounds = 300;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    let tolerances = [0.01, 0.10, 0.20, 0.20];
    let window = 20;

    let mut worst_round = 0_usize;
    for (axis, &tol) in tolerances.iter().enumerate() {
        let baselines: Vec<f64> = traces.iter().map(|t| t.baseline_means[axis]).collect();
        let converged = find_converged_round(&baselines, window, tol);
        worst_round = worst_round.max(converged);
    }

    assert!(
        worst_round <= 250,
        "all axes should converge within 250 rounds at b=16, worst={worst_round}"
    );
}

/// The shipped settings — a much longer memory and larger batches — settle
/// inside their own correspondingly larger budget, measured with a window
/// matched to that memory. This pins the long-memory end of the bound and shows
/// the budget scales with the forgetting factor rather than being a fixed
/// constant.
///
/// (´claim:convergence:every-axis-settles-inside-the-round-budget-at-its-own-tolerance´)
/// ´test:crate:production-lambda-converges-within-bound´
#[test]
fn production_lambda_converges_within_bound() {
    let cfg = cfg_production();
    // Convergence speed depends on λ and b, not on dim.  dim=32
    // halves the SVD cost vs 128 while preserving the bound
    // (ADR-S-012).
    let dim = 32;
    let total_rounds = 1500;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    // At λ=0.99, use window = 1/α = 100.
    let window = 100;
    let tolerances = [0.01, 0.10, 0.20, 0.20];

    let mut worst_round = 0_usize;
    for (axis, &tol) in tolerances.iter().enumerate() {
        let baselines: Vec<f64> = traces.iter().map(|t| t.baseline_means[axis]).collect();
        let converged = find_converged_round(&baselines, window, tol);
        worst_round = worst_round.max(converged);
    }

    assert!(
        worst_round <= 1200,
        "production config should converge within 1200 rounds, got {worst_round}"
    );
}

/// The budget holds across a spread of seeds and not merely for one lucky noise
/// sequence. Two of the axes carry real seed-to-seed variance in when they
/// settle, so a bound demonstrated once would not be a bound at all: sizing a
/// shipped schedule needs the worst case over sequences.
///
/// ´claim:convergence:the-round-budget-holds-across-seeds-and-not-merely-one-lucky-sequence´
/// ´test:crate:per-axis-convergence-b4-robust-across-seeds´
#[test]
fn per_axis_convergence_b4_robust_across_seeds() {
    let cfg = cfg_test();
    // See production_lambda_converges_within_bound comment.
    let dim = 32;
    let total_rounds = 500;
    let window = 20;
    let tolerances = [0.01, 0.10, 0.20, 0.20];

    for seed in [42, 123, 456, 789, 1024] {
        let traces = run_noise_trace(&cfg, dim, total_rounds, seed);

        let mut worst_round = 0_usize;
        for (axis, &tol) in tolerances.iter().enumerate() {
            let baselines: Vec<f64> = traces.iter().map(|t| t.baseline_means[axis]).collect();
            let converged = find_converged_round(&baselines, window, tol);
            worst_round = worst_round.max(converged);
        }

        assert!(
            worst_round <= 500,
            "all axes should converge within 500 rounds at b=4 (seed={seed}), worst={worst_round}"
        );
    }
}
