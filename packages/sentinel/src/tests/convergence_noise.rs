// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`noise_baselines_converge_within_bound`] | convergence | A baseline counts as converged when an early block of rounds and a late one agree, not when its round-to-round jitter has stopped. Steady-state jitter is irreducible and differs by more than a hundredfold across the axes, so a single-round criterion tight enough for the quietest axis would report the noisiest as permanently unconverged long after it had reached its correct level. |
//! | [`baseline_variance_converges_within_bound`] | convergence | cites (´claim:convergence:convergence-is-agreement-between-an-early-and-a-late-block-not-the-absence-of-jitter´) |
//! | [`frozen_subspace_converges_at_least_as_fast`] | convergence | Holding the subspace still never makes the baselines settle later than letting it evolve. Scores are measured against the model, so a model that is itself still moving is a second source of variation on top of the input; removing it can only help. |
//! | [`rank_reaches_max_within_expected_rounds`] | convergence | Rank climbs to its ceiling within a few adaptation intervals, one step at each. The model is therefore at full width long before the baselines settle, so the bulk of a warm-up is spent learning score distributions rather than discovering how many directions to keep. |
//! | [`coherence_activates_at_rank_two`] | convergence | Coherence is a statement about pairs of latent directions, so below two directions there are no pairs and the axis does not exist. Its baseline is held at the cold placeholder rather than being fed the identically zero scores, and it enters through the cold-start path the moment a second direction appears — otherwise it would converge onto zero and then have to unlearn it. |
//! | [`energy_ratio_stabilises_under_noise`] | convergence | The share of energy the retained directions capture settles onto a narrow plateau near the threshold that chose the rank. That ratio is the quantity rank adaptation reads, so its settling is what stops the rank from oscillating. |
//! | [`report_baseline_is_pre_update_snapshot`] | convergence | A report carries the baseline the batch was scored against, never the one the batch produced: the first report shows the cold placeholder even though the internal state has already moved, and the second shows exactly what the first batch left behind. A batch can therefore never partly explain itself away, and a reader can reconstruct the comparison that produced the scores. |
//! | [`z_scores_stay_bounded_at_steady_state`] | convergence | Measured against a settled baseline, ordinary traffic scores near zero on average and stays unremarkable on every single round. The baselines are calibrated and not merely stable: a systematic offset would mean every batch looked mildly anomalous, leaving no headroom to signal one that genuinely was. |
//! | [`latent_variance_reaches_steady_state`] | convergence | The latent spread settles far below the value a freshly constructed tracker holds. That gap is why the first batch seeds the spread instead of blending toward it: starting an order of magnitude high would suppress the surprise axis for as long as the gap took to decay. |
//! | [`latent_mean_stays_near_zero`] | convergence | Under centred input the latent mean stays at zero, so the value a fresh tracker starts from was already the right one. The encoding centres every bit for exactly this reason, and it is what lets the spread be measured about a fixed origin rather than a moving one. |
//! | [`subspace_evolution_does_not_dominate_latvar_transient`] | convergence | The latent spread's approach to its steady state is governed by the forgetting factor, not by the subspace still moving underneath it: an evolving basis and an effectively frozen one land in the same neighbourhood. Warm-up length can therefore be reasoned about from the decay alone. |
//! | [`production_noise_provides_reasonable_novelty_baseline`] | convergence | Novelty settles quickest of the four axes: a short schedule already places it within a few percent of where a far longer run leaves it. It is built from reconstruction error rather than from latent statistics, so it does not have to wait for the latent distribution to settle first — which is what makes a short warm-up useful before the other axes are ready. |
//! | [`deterministic_under_same_seed`] | convergence | The same seed replays the same run bit for bit, baseline for baseline. Nothing in the pipeline depends on iteration order over an unordered structure or on timing, so a convergence figure is a property of the configuration and the seed rather than of the machine that measured it. |

//! What a whole tracker looks like once it has settled on noise.
//!
//! Warm-up feeds a fresh tracker synthetic traffic so that it arrives at
//! real work already knowing what ordinary looks like. That means more
//! than a mean per axis: the model has to have found how many directions
//! to keep, the latent distribution those directions are read against has
//! to have settled, and the four score baselines have to be calibrated
//! enough that ordinary traffic scores near zero and leaves headroom for
//! traffic that is not.
//!
//! The pieces settle in a definite order, and that order is why a bounded
//! warm-up works. Rank reaches its ceiling within a few adaptation
//! intervals and the captured energy plateaus near the threshold that
//! chose it, so the model is at full width long before the baselines are
//! anywhere near done — the bulk of a warm-up is spent learning score
//! distributions, not deciding on a geometry. The latent transient is
//! governed by the forgetting factor rather than by the subspace still
//! moving underneath it, so warm-up length can be reasoned about from the
//! decay alone. Coherence is the exception that proves the ordering: it
//! is a statement about pairs of directions, so it stays cold until there
//! are two to relate rather than converging onto the zeroes it would
//! otherwise be fed.
//!
//! Judging all this needs care about what convergence means. An
//! exponentially-weighted baseline never stops jittering, so settling is
//! established by agreement between an early block of rounds and a late
//! one, per axis and at that axis's own tolerance — for the spread as well
//! as the level, since a z-score divides by one and centres on the other.
//! And every measurement here is repeatable: the same seed replays the
//! same run bit for bit, so a convergence figure is a property of the
//! configuration rather than of the machine that took it.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 3 — Latent distribution cold→warm
//! - §ALGO S-7.1.1 — EWMA outlier filter
//! - §ALGO S-11.5 — Maturity tracking
//! - ADR-S-013 — Warm-up convergence benchmark
//! - ADR-S-014 — Subspace tracker visibility

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{as_slices, block_mean_relative_error, cfg_test, find_settled_round, generate_noise};
use crate::config::SentinelConfig;
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Baseline convergence
// ════════════════════════════════════════════════════════════

/// A baseline counts as converged when an early block of rounds and a late one
/// agree, not when its round-to-round jitter has stopped. Steady-state jitter
/// is irreducible and differs by more than a hundredfold across the axes, so a
/// single-round criterion tight enough for the quietest axis would report the
/// noisiest as permanently unconverged long after it had reached its correct
/// level.
///
/// ´claim:convergence:convergence-is-agreement-between-an-early-and-a-late-block-not-the-absence-of-jitter´
/// ´test:crate:noise-baselines-converge-within-bound´
#[test]
fn noise_baselines_converge_within_bound() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let total_rounds = 300;
    let mut rng = SmallRng::seed_from_u64(42);

    let mut traces: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        traces.push([
            report.scores.novelty.baseline.mean,
            report.scores.displacement.baseline.mean,
            report.scores.surprise.baseline.mean,
            report.scores.coherence.baseline.mean,
        ]);
    }

    // Per-axis tolerances for the block-mean comparison.
    //
    // These are set to ~3× the expected block-mean fluctuation,
    // which is CV_EWMA × √((1+λ)/(1−λ) / block_len).  With
    // block_len = 100 and λ = 0.95, the inflation factor is
    // √(31.4/100) ≈ 0.56, so block_CV ≈ 0.56 × EWMA_CV.
    //
    // Axis          | EWMA CV | block CV | 3σ bound | tolerance
    // --------------|---------|---------|----------|----------
    // Novelty       |  0.07%  |  0.04%  |   0.12%  |   2%
    // Displacement  |  3.4%   |  1.9%   |   5.7%   |  10%
    // Surprise      |  6.5%   |  3.6%   |  10.8%   |  15%
    // Coherence     | 10.0%   |  5.6%   |  16.8%   |  25%
    let axis_names = ["novelty", "displacement", "surprise", "coherence"];
    let tolerances = [0.02, 0.10, 0.15, 0.25];

    // Early block: rounds [50, 150) — well past the EWMA transient
    // (half-life ≈ 14 rounds at λ = 0.95, so by round 50 the bias
    // is λ⁵⁰ ≈ 0.077 of its initial value).
    //
    // Late block: rounds [200, 300) — the reference steady state.
    let early_start = 50;
    let early_end = 150;
    let late_start = 200;
    let late_end = total_rounds;

    for (ax, (name, tol)) in axis_names.iter().zip(tolerances.iter()).enumerate() {
        let err = block_mean_relative_error(&traces, ax, early_start, early_end, late_start, late_end);
        if let Some(rel_err) = err {
            assert!(
                rel_err < *tol,
                "{name} baseline not stationary: early vs late block \
                 differ by {:.2}%, tolerance is {:.0}%",
                rel_err * 100.0,
                tol * 100.0,
            );
        }
    }
}

/// The same block comparison holds for the spread and not only for the level. A
/// z-score divides by the spread, so a settled mean over an unsettled variance
/// would still be miscalibrated — this pins the half that calibration depends
/// on.
///
/// (´claim:convergence:convergence-is-agreement-between-an-early-and-a-late-block-not-the-absence-of-jitter´)
/// ´test:crate:baseline-variance-converges-within-bound´
#[test]
fn baseline_variance_converges_within_bound() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let total_rounds = 300;
    let mut rng = SmallRng::seed_from_u64(42);

    let mut traces: Vec<[f64; 4]> = Vec::with_capacity(total_rounds);
    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        traces.push([
            report.scores.novelty.baseline.variance,
            report.scores.displacement.baseline.variance,
            report.scores.surprise.baseline.variance,
            report.scores.coherence.baseline.variance,
        ]);
    }

    // Variance has higher CV than mean, so use more generous tolerances.
    // Coherence variance is a 4th-order statistic (products of normals)
    // with very high inherent variability.
    let axis_names = ["novelty", "displacement", "surprise", "coherence"];
    let tolerances = [0.10, 0.20, 0.30, 0.50];

    let early_start = 50;
    let early_end = 150;
    let late_start = 200;
    let late_end = total_rounds;

    for (ax, (name, tol)) in axis_names.iter().zip(tolerances.iter()).enumerate() {
        let err = block_mean_relative_error(&traces, ax, early_start, early_end, late_start, late_end);
        if let Some(rel_err) = err {
            assert!(
                rel_err < *tol,
                "{name} baseline variance not stationary: early vs late block \
                 differ by {:.2}%, tolerance is {:.0}%",
                rel_err * 100.0,
                tol * 100.0,
            );
        }
    }
}

/// Holding the subspace still never makes the baselines settle later than
/// letting it evolve. Scores are measured against the model, so a model that is
/// itself still moving is a second source of variation on top of the input;
/// removing it can only help.
///
/// ´claim:convergence:a-moving-subspace-can-only-delay-the-baselines-never-hasten-them´
/// ´test:crate:frozen-subspace-converges-at-least-as-fast´
#[test]
fn frozen_subspace_converges_at_least_as_fast() {
    let dim = 128;
    let total_rounds = 300;
    let seed = 42;

    let run = |cfg: &SentinelConfig<u64>| -> usize {
        let mut tracker = SubspaceTracker::new(dim, cfg, cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(seed);
        let mut traces = Vec::with_capacity(total_rounds);
        for _ in 0..total_rounds {
            let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
            let report = tracker.observe(&as_slices(&noise), 0, true);
            traces.push([
                report.scores.novelty.baseline.mean,
                report.scores.displacement.baseline.mean,
                report.scores.surprise.baseline.mean,
                report.scores.coherence.baseline.mean,
            ]);
        }
        let reference = *traces.last().unwrap();
        find_settled_round(&traces, &reference, 0.05, 3).unwrap_or(total_rounds)
    };

    let cfg_frozen = SentinelConfig {
        rank_update_interval: 10_000,
        ..cfg_test()
    };

    let r_frozen = run(&cfg_frozen);
    let r_normal = run(&cfg_test());

    assert!(
        r_frozen <= r_normal,
        "frozen subspace should converge at least as fast: frozen={r_frozen}, normal={r_normal}"
    );
}

// ════════════════════════════════════════════════════════════
//  Rank adaptation
// ════════════════════════════════════════════════════════════

/// Rank climbs to its ceiling within a few adaptation intervals, one step at
/// each. The model is therefore at full width long before the baselines settle,
/// so the bulk of a warm-up is spent learning score distributions rather than
/// discovering how many directions to keep.
///
/// ´claim:convergence:rank-reaches-its-ceiling-long-before-the-baselines-settle´
/// ´test:crate:rank-reaches-max-within-expected-rounds´
#[test]
fn rank_reaches_max_within_expected_rounds() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let mut ranks: Vec<usize> = Vec::with_capacity(100);
    for _ in 0..100 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        ranks.push(report.rank);
    }

    let first_rank2 = ranks.iter().position(|&r| r >= 2);
    let r = first_rank2.expect("rank should reach 2 within 100 noise rounds");
    assert!(r <= 20, "rank should reach 2 within 20 noise rounds, got {r}");
}

/// Coherence is a statement about pairs of latent directions, so below two
/// directions there are no pairs and the axis does not exist. Its baseline is
/// held at the cold placeholder rather than being fed the identically zero
/// scores, and it enters through the cold-start path the moment a second
/// direction appears — otherwise it would converge onto zero and then have to
/// unlearn it.
///
/// ´claim:convergence:the-coherence-baseline-stays-cold-until-there-are-two-directions-to-relate´
/// ´test:crate:coherence-activates-at-rank-two´
#[test]
fn coherence_activates_at_rank_two() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let mut coherence_means: Vec<f64> = Vec::new();
    let mut ranks: Vec<usize> = Vec::new();

    for _ in 0..100 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        coherence_means.push(report.scores.coherence.baseline.mean);
        ranks.push(report.rank);
    }

    // Before rank 2: coherence baseline stays at cold default.
    let first_rank2 = ranks.iter().position(|&r| r >= 2).unwrap_or(100);
    for r in 0..first_rank2.min(100) {
        if ranks[r] < 2 {
            assert!(
                (coherence_means[r] - 1.0).abs() < 1e-10,
                "round {r}: coherence mean should be cold (1.0) at rank {}, got {:.6}",
                ranks[r],
                coherence_means[r]
            );
        }
    }

    // After rank 2: coherence should evolve away from 1.0.
    if first_rank2 + 5 < 100 {
        let late_mean = coherence_means[first_rank2 + 5];
        assert!(
            (late_mean - 1.0).abs() > 1e-6,
            "coherence should evolve after reaching rank 2, still at {late_mean:.6}"
        );
    }
}

/// The share of energy the retained directions capture settles onto a narrow
/// plateau near the threshold that chose the rank. That ratio is the quantity
/// rank adaptation reads, so its settling is what stops the rank from
/// oscillating.
///
/// ´claim:convergence:the-captured-energy-settles-onto-a-plateau-near-the-threshold-that-chose-the-rank´
/// ´test:crate:energy-ratio-stabilises-under-noise´
#[test]
fn energy_ratio_stabilises_under_noise() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let total_rounds = 200;
    let mut energies: Vec<f64> = Vec::with_capacity(total_rounds);
    for _ in 0..total_rounds {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        energies.push(report.energy_ratio);
    }

    // After rank reaches max, energy ratio should be stable.
    // Compare the last 50 rounds: max-min spread should be small.
    let tail = &energies[total_rounds - 50..];
    let min_e = tail.iter().copied().fold(f64::INFINITY, f64::min);
    let max_e = tail.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    let spread = max_e - min_e;

    assert!(
        spread < 0.05,
        "energy ratio not stable in last 50 rounds: spread = {spread:.4} (min={min_e:.4}, max={max_e:.4})"
    );
    assert!(
        min_e >= cfg.energy_threshold * 0.9,
        "energy ratio ({min_e:.4}) should be near the threshold ({:.2})",
        cfg.energy_threshold
    );
}

// ════════════════════════════════════════════════════════════
//  Report snapshot semantics
// ════════════════════════════════════════════════════════════

/// A report carries the baseline the batch was scored against, never the one
/// the batch produced: the first report shows the cold placeholder even though
/// the internal state has already moved, and the second shows exactly what the
/// first batch left behind. A batch can therefore never partly explain itself
/// away, and a reader can reconstruct the comparison that produced the scores.
///
/// ´claim:convergence:a-report-carries-the-baseline-the-batch-was-scored-against-not-the-one-it-produced´
/// ´test:crate:report-baseline-is-pre-update-snapshot´
#[test]
fn report_baseline_is_pre_update_snapshot() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // First observe: EWMA is cold → report shows cold default (1.0).
    let noise1 = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    let report1 = tracker.observe(&as_slices(&noise1), 0, true);

    assert!(
        (report1.scores.novelty.baseline.mean - 1.0).abs() < 1e-10,
        "first report should have cold baseline mean=1.0, got {}",
        report1.scores.novelty.baseline.mean
    );

    // Internal EWMA is now warm.
    let bl = tracker.axis_baselines();
    assert!(
        (bl.novelty_mean - 1.0).abs() > 1e-6,
        "after first observe, internal EWMA should have moved from 1.0, got {}",
        bl.novelty_mean
    );

    // Second observe: report baseline matches post-first-update state.
    let noise2 = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    let report2 = tracker.observe(&as_slices(&noise2), 0, true);

    assert!(
        (report2.scores.novelty.baseline.mean - bl.novelty_mean).abs() < 1e-10,
        "second report baseline ({:.6}) should match post-first-update state ({:.6})",
        report2.scores.novelty.baseline.mean,
        bl.novelty_mean
    );
}

/// Measured against a settled baseline, ordinary traffic scores near zero on
/// average and stays unremarkable on every single round. The baselines are
/// calibrated and not merely stable: a systematic offset would mean every batch
/// looked mildly anomalous, leaving no headroom to signal one that genuinely
/// was.
///
/// ´claim:convergence:against-a-settled-baseline-ordinary-traffic-scores-near-zero-and-never-extreme´
/// ´test:crate:z-scores-stay-bounded-at-steady-state´
#[test]
#[allow(clippy::cast_precision_loss)]
fn z_scores_stay_bounded_at_steady_state() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Warm up.
    for _ in 0..200 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    // Collect z-scores over the next 100 rounds.
    let mut z_traces: Vec<[f64; 4]> = Vec::with_capacity(100);
    for _ in 0..100 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        z_traces.push([
            report.scores.novelty.mean_z_score,
            report.scores.displacement.mean_z_score,
            report.scores.surprise.mean_z_score,
            report.scores.coherence.mean_z_score,
        ]);
    }

    let axis_names = ["novelty", "displacement", "surprise", "coherence"];
    for (ax, name) in axis_names.iter().enumerate() {
        let mean_z: f64 = z_traces.iter().map(|t| t[ax]).sum::<f64>() / z_traces.len() as f64;
        let max_abs_z: f64 = z_traces.iter().map(|t| t[ax].abs()).fold(0.0_f64, f64::max);
        // Average z-score should be near zero over many rounds.
        assert!(mean_z.abs() < 1.5, "{name} mean z-score = {mean_z:.2} — expected near zero");
        // Individual z-scores should not be extreme under noise.
        assert!(
            max_abs_z < 5.0,
            "{name} max |z| = {max_abs_z:.2} — expected < 5.0 under noise"
        );
    }
}

// ════════════════════════════════════════════════════════════
//  Latent statistics steady state
// ════════════════════════════════════════════════════════════

/// The latent spread settles far below the value a freshly constructed tracker
/// holds. That gap is why the first batch seeds the spread instead of blending
/// toward it: starting an order of magnitude high would suppress the surprise
/// axis for as long as the gap took to decay.
///
/// ´claim:convergence:the-latent-spread-settles-far-below-its-constructed-placeholder´
/// ´test:crate:latent-variance-reaches-steady-state´
#[test]
fn latent_variance_reaches_steady_state() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..500 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let lat_vars = tracker.latent_var();
    for (j, &v) in lat_vars.iter().enumerate() {
        assert!(v < 0.5, "lat_var[{j}] = {v:.6} should be ≪ 1.0 at steady state");
    }
}

/// Under centred input the latent mean stays at zero, so the value a fresh
/// tracker starts from was already the right one. The encoding centres every
/// bit for exactly this reason, and it is what lets the spread be measured
/// about a fixed origin rather than a moving one.
///
/// ´claim:convergence:centred-input-leaves-the-latent-mean-where-a-fresh-tracker-starts-it´
/// ´test:crate:latent-mean-stays-near-zero´
#[test]
fn latent_mean_stays_near_zero() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..500 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let lat_means = tracker.latent_mean();
    for (j, &m) in lat_means.iter().enumerate() {
        assert!(m.abs() < 0.1, "lat_mean[{j}] = {m:.6} should be near zero for centred noise");
    }
}

/// The latent spread's approach to its steady state is governed by the
/// forgetting factor, not by the subspace still moving underneath it: an
/// evolving basis and an effectively frozen one land in the same neighbourhood.
/// Warm-up length can therefore be reasoned about from the decay alone.
///
/// ´claim:convergence:the-latent-transient-is-governed-by-the-forgetting-factor-not-by-the-subspace-still-moving´
/// ´test:crate:subspace-evolution-does-not-dominate-latvar-transient´
#[test]
fn subspace_evolution_does_not_dominate_latvar_transient() {
    let dim = 128;
    let total_rounds = 200;

    let run = |rank_update_interval: u64| -> Vec<f64> {
        let cfg = SentinelConfig {
            rank_update_interval,
            ..cfg_test()
        };
        let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(42);
        let mut var_trace = Vec::with_capacity(total_rounds);
        for _ in 0..total_rounds {
            let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
            tracker.observe(&as_slices(&noise), 0, true);
            var_trace.push(tracker.latent_var().first().copied().unwrap_or(0.0));
        }
        var_trace
    };

    let normal_trace = run(5); // normal rank adaptation
    let frozen_trace = run(10_000); // effectively frozen

    let normal_ref = *normal_trace.last().unwrap();
    let frozen_ref = *frozen_trace.last().unwrap();
    let diff_pct = (normal_ref - frozen_ref).abs() / normal_ref.abs().max(1e-12) * 100.0;

    assert!(
        diff_pct < 50.0,
        "lat_var steady states should be similar: normal={normal_ref:.6}, frozen={frozen_ref:.6} ({diff_pct:.1}% diff)"
    );
}

// ════════════════════════════════════════════════════════════
//  Production noise-rounds quality
// ════════════════════════════════════════════════════════════

/// Novelty settles quickest of the four axes: a short schedule already places
/// it within a few percent of where a far longer run leaves it. It is built
/// from reconstruction error rather than from latent statistics, so it does not
/// have to wait for the latent distribution to settle first — which is what
/// makes a short warm-up useful before the other axes are ready.
///
/// ´claim:convergence:novelty-settles-quickest-because-it-does-not-wait-on-the-latent-distribution´
/// ´test:crate:production-noise-provides-reasonable-novelty-baseline´
#[test]
fn production_noise_provides_reasonable_novelty_baseline() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let mut baselines_at_50 = [0.0_f64; 4];
    for round in 0..50 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        if round == 49 {
            baselines_at_50 = [
                report.scores.novelty.baseline.mean,
                report.scores.displacement.baseline.mean,
                report.scores.surprise.baseline.mean,
                report.scores.coherence.baseline.mean,
            ];
        }
    }

    // Run 250 more to get the reference.
    for _ in 50..300 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let bl = tracker.axis_baselines();
    let reference_novelty = bl.novelty_mean;

    if reference_novelty.abs() > 1e-12 {
        let novelty_error = (baselines_at_50[0] - reference_novelty).abs() / reference_novelty.abs();
        assert!(
            novelty_error < 0.05,
            "novelty at round 50 is {:.1}% off — should be <5%",
            novelty_error * 100.0
        );
    }
}

// ════════════════════════════════════════════════════════════
//  Determinism
// ════════════════════════════════════════════════════════════

/// The same seed replays the same run bit for bit, baseline for baseline.
/// Nothing in the pipeline depends on iteration order over an unordered
/// structure or on timing, so a convergence figure is a property of the
/// configuration and the seed rather than of the machine that measured it.
///
/// ´claim:convergence:the-same-seed-replays-the-same-run-bit-for-bit´
/// ´test:crate:deterministic-under-same-seed´
#[test]
fn deterministic_under_same_seed() {
    let run = || {
        let cfg = cfg_test();
        let dim = 128;
        let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(42);

        let mut baselines = Vec::with_capacity(100);
        for _ in 0..100 {
            let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
            let report = tracker.observe(&as_slices(&noise), 0, true);
            baselines.push([
                report.scores.novelty.baseline.mean,
                report.scores.displacement.baseline.mean,
                report.scores.surprise.baseline.mean,
                report.scores.coherence.baseline.mean,
            ]);
        }
        baselines
    };

    let a = run();
    let b = run();

    assert_eq!(a.len(), b.len());
    for (i, (ra, rb)) in a.iter().zip(b.iter()).enumerate() {
        for ax in 0..4 {
            assert!(
                (ra[ax] - rb[ax]).abs() == 0.0,
                "round {i} axis {ax}: run A = {}, run B = {} — not bit-identical",
                ra[ax],
                rb[ax]
            );
        }
    }
}
