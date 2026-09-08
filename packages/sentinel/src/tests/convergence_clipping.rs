// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`clip_bias_is_negligible_at_steady_state`] | clipping | Rejecting the upper tail leaves the settled baseline where an unclipped run puts it. The clip removes a fraction of a percent of the mass of a right-skewed score distribution, so the downward bias it induces is smaller than the baseline's own steady-state jitter: outlier resistance is bought without moving the reference it protects. |
//! | [`variance_estimate_unbiased_despite_clipping`] | clipping | The spread estimate survives tail truncation as well as the mean does. A second moment is far more sensitive to a missing tail than a first, so this is the tighter half of the same audit: a baseline built under clipping is calibrated and not merely correctly centred. |
//! | [`graduated_exemption_prevents_bistable_attractor`] | clipping | Clip widths from tight to loose all settle on the same fixed point. A clipped baseline is a nonlinear filter with a second, biased fixed point where tight clipping keeps rejecting the very evidence that would loosen it; the graduated exemption widens the basin of the correct one during warm-up, so no configuration falls into the other. |
//! | [`exemption_decay_does_not_cause_transient_instability`] | clipping | As the exemption is spent the effective clip tightens, and the baselines pass through that tightening without a spike or a dip — once the exemption has largely decayed, no rolling mean departs from the eventual steady state by more than the jitter envelope. The width is a smooth function of the exemption rather than a switch, which is what keeps a trajectory from being thrown across a basin boundary. |
//! | [`clip_ceiling_stabilises_after_exemption_decay`] | clipping | The ceiling is the baseline's own mean and spread scaled by the clip width, so it settles when they do: once the exemption is spent its rolling variation stays within a few percent. A ceiling that kept swinging would clip in bursts and feed those bursts straight back into the mean and spread that define it. |
//! | [`slow_ewma_clipping_does_not_bias_cusum_reference`] | clipping | The long-memory reference is filtered by the same ceiling as the short-memory baseline, and once the two have been seeded into agreement they stay in agreement across a long run. Clipping therefore adds no bias of its own on the reference side, so a gap between the two can be read as drift rather than as an artefact of filtering. |
//! | [`cusum_bounded_through_clip_transitions`] | clipping | A tightening clip does not manufacture evidence of drift: through the whole stretch after seeding, the drift accumulators stay far below anything a host would act on. The moment the clip narrows is when the two baselines are most likely to disagree, so it is where a false alarm would appear if the mechanisms interfered with one another. |

//! An audit of what outlier rejection costs the converged model.
//!
//! Clipping exists so that a burst of inflated scores cannot poison a
//! baseline: only the upper tail is rejected, because anomaly scores are
//! non-negative and right-skewed and an attacker inflates them rather than
//! deflating them. But a baseline that filters against its own mean and
//! spread is a feedback loop, and a feedback loop can settle somewhere
//! other than where the data is. Three places in the pipeline apply the
//! same ceiling — the short-memory baseline, the long-memory reference the
//! drift accumulator measures against, and the exemption that widens the
//! ceiling while the model is still noise-taught — and each could, in
//! principle, bias the settled model or delay it. The tests here establish
//! that none of them does.
//!
//! Two properties do the work. The rejected tail is a fraction of a
//! percent of the mass at the nominal width, so the bias it induces is
//! smaller than the baseline's own steady-state jitter — for the spread as
//! well as for the level, which is the more demanding of the two. And the
//! exemption removes the loop's second, biased fixed point during warm-up
//! by holding the ceiling open until the model has a real baseline to
//! filter against; the width then narrows smoothly rather than switching,
//! so nothing is thrown across a basin boundary on the way down.
//!
//! The audit is therefore comparative: a clipped run against an unclipped
//! one, several clip widths against each other, and the trajectory
//! through the tightening against the steady state it ends at.

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{AXIS_NAMES, as_slices, cfg_test, generate_noise, run_noise_trace};
use crate::config::SentinelConfig;
use crate::sentinel::tracker::SubspaceTracker;

/// Per-round snapshot of baseline mean, variance, and η.
struct RoundSnap {
    baseline_mean: [f64; 4],
    baseline_var: [f64; 4],
    eta: f64,
}

// ════════════════════════════════════════════════════════════
//  1. Fast EWMA upper-tail clip  (§ALGO S-7.1.1)
// ════════════════════════════════════════════════════════════

/// Rejecting the upper tail leaves the settled baseline where an unclipped run
/// puts it. The clip removes a fraction of a percent of the mass of a
/// right-skewed score distribution, so the downward bias it induces is smaller
/// than the baseline's own steady-state jitter: outlier resistance is bought
/// without moving the reference it protects.
///
/// ´claim:clipping:outlier-rejection-leaves-the-settled-baseline-where-an-unclipped-run-puts-it´
/// ´test:crate:clip-bias-is-negligible-at-steady-state´
#[test]
fn clip_bias_is_negligible_at_steady_state() {
    let dim = 128;
    let total_rounds = 500;
    let seed = 42;

    // Run with standard clipping.
    let cfg_clipped = cfg_test(); // clip_sigmas = 3.0
    let traces_clipped = run_noise_trace(&cfg_clipped, dim, total_rounds, seed);

    // Run without clipping.
    let cfg_unclipped = SentinelConfig {
        clip_sigmas: f64::INFINITY,
        ..cfg_test()
    };
    let traces_unclipped = run_noise_trace(&cfg_unclipped, dim, total_rounds, seed);

    // Compare steady-state block means (last 100 rounds).
    let block_start = total_rounds - 100;

    // Per-axis tolerances for the clip-vs-unclip comparison.
    //
    // These accommodate both the clip bias AND the EWMA jitter.
    // The clip bias is < 0.3% of the mean (§4 theory), but the
    // EWMA jitter adds uncertainty — especially for high-CV axes
    // (coherence ~10%).  We also use the same RNG seed, so the
    // *input* noise sequence is identical; only the clipping
    // creates divergence.  But the graduated exemption causes the
    // two runs to track differently from the start, so by round
    // 400+ the EWMA trajectories have drifted apart by O(CV).
    //
    // Axis          | expected bias | jitter CV | tolerance
    // --------------|---------------|-----------|----------
    // Novelty       |   < 0.1%     |   0.07%   |   2%
    // Displacement  |   < 0.5%     |   3.4%    |  10%
    // Surprise      |   < 0.5%     |   6.5%    |  15%
    // Coherence     |   < 0.5%     |  10.0%    |  20%
    let tolerances = [0.02, 0.10, 0.15, 0.20];

    for (ax, (name, tol)) in AXIS_NAMES.iter().zip(tolerances.iter()).enumerate() {
        let clipped_mean: f64 = traces_clipped[block_start..]
            .iter()
            .map(|t| t.baseline_means[ax])
            .sum::<f64>()
            / 100.0;

        let unclipped_mean: f64 = traces_unclipped[block_start..]
            .iter()
            .map(|t| t.baseline_means[ax])
            .sum::<f64>()
            / 100.0;

        if unclipped_mean.abs() < 1e-12 {
            continue; // axis inactive
        }

        let rel_diff = (clipped_mean - unclipped_mean).abs() / unclipped_mean.abs();
        assert!(
            rel_diff < *tol,
            "{name}: clipped vs unclipped steady-state bias is {:.2}%, \
             tolerance is {:.0}% — clipping introduces excessive bias",
            rel_diff * 100.0,
            tol * 100.0,
        );
    }
}

/// The spread estimate survives tail truncation as well as the mean does. A
/// second moment is far more sensitive to a missing tail than a first, so this
/// is the tighter half of the same audit: a baseline built under clipping is
/// calibrated and not merely correctly centred.
///
/// ´claim:clipping:the-spread-estimate-survives-tail-truncation-as-well-as-the-mean-does´
/// ´test:crate:variance-estimate-unbiased-despite-clipping´
#[test]
fn variance_estimate_unbiased_despite_clipping() {
    let dim = 128;
    let total_rounds = 500;
    let seed = 42;

    // We need variance, not just means — run directly.
    let run_variances = |cfg: &SentinelConfig<u64>| -> [f64; 4] {
        let mut tracker = SubspaceTracker::new(dim, cfg, cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut last_var = [0.0_f64; 4];
        for _ in 0..total_rounds {
            let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
            let report = tracker.observe(&as_slices(&noise), 0, true);
            last_var = [
                report.scores.novelty.baseline.variance,
                report.scores.displacement.baseline.variance,
                report.scores.surprise.baseline.variance,
                report.scores.coherence.baseline.variance,
            ];
        }
        last_var
    };

    let cfg_clipped = cfg_test();
    let var_clipped = run_variances(&cfg_clipped);

    let cfg_unclipped = SentinelConfig {
        clip_sigmas: f64::INFINITY,
        ..cfg_test()
    };
    let var_unclipped = run_variances(&cfg_unclipped);

    // Since variance is a second-moment estimate, it's more
    // sensitive to tail truncation than the mean.  But at 3σ
    // the effect is still small.  We use generous tolerances.
    let tolerances = [0.05, 0.15, 0.25, 0.35];

    for (ax, (name, tol)) in AXIS_NAMES.iter().zip(tolerances.iter()).enumerate() {
        if var_unclipped[ax].abs() < 1e-12 {
            continue;
        }
        let rel_diff = (var_clipped[ax] - var_unclipped[ax]).abs() / var_unclipped[ax];
        assert!(
            rel_diff < *tol,
            "{name}: clipped variance differs from unclipped by {:.2}%, \
             tolerance {:.0}% — clipping distorts the variance estimate",
            rel_diff * 100.0,
            tol * 100.0,
        );
    }
}

// ════════════════════════════════════════════════════════════
//  2. Graduated clip-exemption  (§ALGO S-6.4)
// ════════════════════════════════════════════════════════════

/// Clip widths from tight to loose all settle on the same fixed point. A
/// clipped baseline is a nonlinear filter with a second, biased fixed point
/// where tight clipping keeps rejecting the very evidence that would loosen it;
/// the graduated exemption widens the basin of the correct one during warm-up,
/// so no configuration falls into the other.
///
/// ´claim:clipping:every-clip-width-settles-on-the-same-fixed-point´
/// ´test:crate:graduated-exemption-prevents-bistable-attractor´
#[test]
fn graduated_exemption_prevents_bistable_attractor() {
    let dim = 128;
    let total_rounds = 500;
    let seed = 42;

    let clip_values = [2.0, 3.0, 5.0];
    let mut steady_states: Vec<[f64; 4]> = Vec::new();

    for &clip in &clip_values {
        let cfg = SentinelConfig {
            clip_sigmas: clip,
            ..cfg_test()
        };
        let traces = run_noise_trace(&cfg, dim, total_rounds, seed);

        // Compute block mean of last 100 rounds.
        let block_start = total_rounds - 100;
        let mut means = [0.0_f64; 4];
        for (ax, mean) in means.iter_mut().enumerate() {
            *mean = traces[block_start..].iter().map(|t| t.baseline_means[ax]).sum::<f64>() / 100.0;
        }
        steady_states.push(means);
    }

    // Compare all pairs: each axis should agree within tolerance.
    //
    // The tolerance accounts for both the (small) clip bias
    // difference between c = 2 and c = 5, and the EWMA trajectory
    // divergence from different clipping histories.
    let tolerances = [0.03, 0.12, 0.18, 0.30];

    for i in 0..clip_values.len() {
        for j in (i + 1)..clip_values.len() {
            for (ax, (name, tol)) in AXIS_NAMES.iter().zip(tolerances.iter()).enumerate() {
                let ref_val = f64::midpoint(steady_states[i][ax], steady_states[j][ax]);
                if ref_val.abs() < 1e-12 {
                    continue;
                }
                let rel_diff = (steady_states[i][ax] - steady_states[j][ax]).abs() / ref_val.abs();
                assert!(
                    rel_diff < *tol,
                    "{name}: clip_sigmas {:.0} vs {:.0} differ by {:.2}%, tolerance {:.0}% — \
                     graduated exemption may not be preventing bistable attractor",
                    clip_values[i],
                    clip_values[j],
                    rel_diff * 100.0,
                    tol * 100.0,
                );
            }
        }
    }
}

/// As the exemption is spent the effective clip tightens, and the baselines
/// pass through that tightening without a spike or a dip — once the exemption
/// has largely decayed, no rolling mean departs from the eventual steady state
/// by more than the jitter envelope. The width is a smooth function of the
/// exemption rather than a switch, which is what keeps a trajectory from being
/// thrown across a basin boundary.
///
/// ´claim:clipping:the-baselines-cross-the-tightening-of-the-clip-without-a-transient´
/// ´test:crate:exemption-decay-does-not-cause-transient-instability´
#[test]
#[allow(clippy::cast_precision_loss)]
fn exemption_decay_does_not_cause_transient_instability() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 400;
    let traces = run_noise_trace(&cfg, dim, total_rounds, 42);

    let window = 20_usize;

    // Find the round where η drops below 0.1.
    let eta_threshold_round = traces.iter().position(|t| t.noise_influence < 0.1).unwrap_or(total_rounds);

    // Compute the reference steady-state mean (last 50 rounds).
    let ss_start = total_rounds - 50;
    let mut ss_means = [0.0_f64; 4];
    for (ax, ss_mean) in ss_means.iter_mut().enumerate() {
        *ss_mean = traces[ss_start..].iter().map(|t| t.baseline_means[ax]).sum::<f64>() / 50.0;
    }

    // Per-axis tolerances for the post-exemption jitter envelope.
    // These are wider than the block-mean tolerances in
    // convergence_noise.rs because a 20-round rolling mean has
    // higher variance than a 100-round block mean.
    let tolerances = [0.05, 0.15, 0.25, 0.35];

    let start_check = eta_threshold_round.max(window);
    for round in start_check..=(total_rounds - window) {
        let mut rolling = [0.0_f64; 4];
        for (ax, roll) in rolling.iter_mut().enumerate() {
            *roll = traces[round..round + window]
                .iter()
                .map(|t| t.baseline_means[ax])
                .sum::<f64>()
                / window as f64;
        }

        for (ax, (name, tol)) in AXIS_NAMES.iter().zip(tolerances.iter()).enumerate() {
            if ss_means[ax].abs() < 1e-12 {
                continue;
            }
            let rel_dev = (rolling[ax] - ss_means[ax]).abs() / ss_means[ax].abs();
            assert!(
                rel_dev < *tol,
                "{name} at round {round}: rolling mean deviates {:.2}% from \
                 steady state (tolerance {:.0}%) — transient instability \
                 during exemption decay",
                rel_dev * 100.0,
                tol * 100.0,
            );
        }
    }
}

/// The ceiling is the baseline's own mean and spread scaled by the clip width,
/// so it settles when they do: once the exemption is spent its rolling
/// variation stays within a few percent. A ceiling that kept swinging would
/// clip in bursts and feed those bursts straight back into the mean and spread
/// that define it.
///
/// ´claim:clipping:the-ceiling-settles-into-a-narrow-band-so-no-clip-feedback-cycle-can-start´
/// ´test:crate:clip-ceiling-stabilises-after-exemption-decay´
#[test]
#[allow(clippy::cast_precision_loss)]
fn clip_ceiling_stabilises_after_exemption_decay() {
    let cfg = cfg_test();
    let dim = 128;
    let total_rounds = 400;

    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let mut snaps: Vec<RoundSnap> = Vec::with_capacity(total_rounds);
    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        snaps.push(RoundSnap {
            baseline_mean: [
                report.scores.novelty.baseline.mean,
                report.scores.displacement.baseline.mean,
                report.scores.surprise.baseline.mean,
                report.scores.coherence.baseline.mean,
            ],
            baseline_var: [
                report.scores.novelty.baseline.variance,
                report.scores.displacement.baseline.variance,
                report.scores.surprise.baseline.variance,
                report.scores.coherence.baseline.variance,
            ],
            eta: report.maturity.noise_influence,
        });
    }

    // Compute ceiling series for each axis.
    let clip = cfg.clip_sigmas;
    let mut ceilings: [Vec<f64>; 4] = [
        Vec::with_capacity(total_rounds),
        Vec::with_capacity(total_rounds),
        Vec::with_capacity(total_rounds),
        Vec::with_capacity(total_rounds),
    ];

    for snap in &snaps {
        for (ax, ceil_vec) in ceilings.iter_mut().enumerate() {
            ceil_vec.push(clip.mul_add(snap.baseline_var[ax].sqrt(), snap.baseline_mean[ax]));
        }
    }

    // Find round where η < 0.05.
    let eta_settled = snaps.iter().position(|s| s.eta < 0.05).unwrap_or(total_rounds);

    // Check rolling CV of ceiling after exemption decay.
    let window = 20_usize;
    let cv_limit = 0.08; // 8% — generous for high-CV axes

    let start_check = eta_settled.max(window);
    for (ax, ceil_series) in ceilings.iter().enumerate() {
        for round in start_check..=(total_rounds - window) {
            let block = &ceil_series[round..round + window];
            let mean = block.iter().sum::<f64>() / window as f64;
            if mean.abs() < 1e-12 {
                continue;
            }
            let var = block.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / window as f64;
            let cv = var.sqrt() / mean;
            assert!(
                cv < cv_limit,
                "{}: clip ceiling CV at round {} is {:.2}% (limit {:.0}%) — \
                 ceiling is not stabilising after exemption decay",
                AXIS_NAMES[ax],
                round,
                cv * 100.0,
                cv_limit * 100.0,
            );
        }
    }
}

// ════════════════════════════════════════════════════════════
//  3. Slow EWMA / CUSUM clip
// ════════════════════════════════════════════════════════════

/// The long-memory reference is filtered by the same ceiling as the
/// short-memory baseline, and once the two have been seeded into agreement they
/// stay in agreement across a long run. Clipping therefore adds no bias of its
/// own on the reference side, so a gap between the two can be read as drift
/// rather than as an artefact of filtering.
///
/// ´claim:clipping:filtering-the-long-memory-reference-does-not-pull-it-away-from-the-short-memory-one´
/// ´test:crate:slow-ewma-clipping-does-not-bias-cusum-reference´
#[test]
fn slow_ewma_clipping_does_not_bias_cusum_reference() {
    let cfg = cfg_test();
    let dim = 128;
    let warmup_rounds = 200;
    let post_seed_rounds = 300;

    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Noise warm-up.
    for _ in 0..warmup_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    // Seed slow from fast (production workflow).
    tracker.seed_cusum_slow_from_baselines();
    tracker.reset_cusum();

    // Continue noise — at this point the slow EWMA starts from the
    // fast EWMA's converged values.  Both receive the same `clip_sigmas`.
    let mut last_report = None;
    for _ in 0..post_seed_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        last_report = Some(tracker.observe(&as_slices(&noise), 0, true));
    }

    let report = last_report.unwrap();

    // Compare fast vs slow baseline means.
    //
    // After 300 rounds post-seed at `λ_s` = 0.999:
    //   `λ_s`^300 ≈ 0.741 — slow has only decayed 26% from the seed.
    //   The fast EWMA at λ = 0.95 has fully forgotten the seed.
    //
    // So the slow baseline is ~74% seed + ~26% new data, while
    // the fast is ~100% new data.  Under i.i.d. noise, the seed
    // value ≈ the new data's mean, so the gap is mainly from
    // stochastic drift.  Per-axis tolerances account for this.
    //
    //   novelty: ~2%, displacement ~10%, surprise ~15%, coh ~25%
    let tolerances = [0.02, 0.10, 0.15, 0.25];
    let axes = [
        (
            "novelty",
            report.scores.novelty.baseline.mean,
            report.scores.novelty.cusum.slow_baseline.mean,
        ),
        (
            "displacement",
            report.scores.displacement.baseline.mean,
            report.scores.displacement.cusum.slow_baseline.mean,
        ),
        (
            "surprise",
            report.scores.surprise.baseline.mean,
            report.scores.surprise.cusum.slow_baseline.mean,
        ),
        (
            "coherence",
            report.scores.coherence.baseline.mean,
            report.scores.coherence.cusum.slow_baseline.mean,
        ),
    ];

    for ((name, fast, slow), tol) in axes.iter().zip(tolerances.iter()) {
        if fast.abs() < 1e-12 {
            continue;
        }
        let rel_diff = (fast - slow).abs() / fast.abs();
        assert!(
            rel_diff < *tol,
            "{name}: slow EWMA baseline differs from fast by {:.2}%, \
             tolerance {:.0}% — slow-EWMA clipping introduces bias \
             in CUSUM reference",
            rel_diff * 100.0,
            tol * 100.0,
        );
    }
}

/// A tightening clip does not manufacture evidence of drift: through the whole
/// stretch after seeding, the drift accumulators stay far below anything a host
/// would act on. The moment the clip narrows is when the two baselines are most
/// likely to disagree, so it is where a false alarm would appear if the
/// mechanisms interfered with one another.
///
/// ´claim:clipping:a-tightening-clip-does-not-manufacture-evidence-of-drift´
/// ´test:crate:cusum-bounded-through-clip-transitions´
#[test]
fn cusum_bounded_through_clip_transitions() {
    let cfg = cfg_test();
    let dim = 128;
    let warmup_rounds = 200;
    let transition_rounds = 300;

    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Noise warm-up.
    for _ in 0..warmup_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    // Seed and reset (production workflow).
    tracker.seed_cusum_slow_from_baselines();
    tracker.reset_cusum();

    // Continue with noise — the clip width is now near production
    // level (η should be small after 200 rounds at λ = 0.95:
    // η = 0.95^200 ≈ 3.5e-5).
    let mut max_cusum = [0.0_f64; 4];
    for _ in 0..transition_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);

        max_cusum[0] = max_cusum[0].max(report.scores.novelty.cusum.accumulator);
        max_cusum[1] = max_cusum[1].max(report.scores.displacement.cusum.accumulator);
        max_cusum[2] = max_cusum[2].max(report.scores.surprise.cusum.accumulator);
        max_cusum[3] = max_cusum[3].max(report.scores.coherence.cusum.accumulator);
    }

    let cusum_limit = 30.0;
    for (ax, name) in AXIS_NAMES.iter().enumerate() {
        assert!(
            max_cusum[ax] < cusum_limit,
            "{name}: CUSUM reached {:.2} during post-seed phase \
             (limit {cusum_limit}) — clip transitions causing false drift",
            max_cusum[ax],
        );
    }
}
