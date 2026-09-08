// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! How much of what a tracker knows came out of a bottle.
//!
//! A cell is warmed on synthetic noise before it ever judges real traffic,
//! and the noise share η is the tracker's own estimate of how much of its
//! state that warming still accounts for. It is the same exponential
//! forgetting the baselines use, applied to a single indicator: an
//! injected sample pulls the share toward one, a real sample pulls it
//! toward zero, and each is weighted exactly as that sample's contribution
//! to the model will be. The share is therefore not a count of rounds but
//! a statement about how much of the present model is still synthetic.
//!
//! Three properties make it usable rather than merely descriptive. It is
//! a proportion under every workload, so a consumer never has to guard
//! against a value that is not a fraction. It moves in one direction per
//! kind of input, so a threshold crossing means the same thing whenever it
//! happens — which is what lets the clip exemption and the end of warm-up
//! be keyed to it. And it matches its closed form to the last digit,
//! because a batch's decay is computed as one power rather than
//! accumulated sample by sample.
//!
//! The observation counters run alongside and answer a different question.
//! They tally samples of each kind and never decay, so a host can still
//! see how much of a model's experience was synthetic long after the share
//! itself has been forgotten.
//!
//! # §-references
//!
//! - §ALGO S-11.5 — Maturity tracking (noise influence η)
//! - ADR-S-013 — Warm-up convergence benchmark
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`eta_starts_at_one_for_cold_tracker`] | noise | A tracker that has seen nothing counts as entirely noise-taught. Everything it will learn first comes from injected traffic, so the honest starting position is that none of its state yet reflects real observations. |
//! | [`counters_start_at_zero_for_cold_tracker`] | noise | A fresh tracker claims no observations of either kind, and the total is exactly the two counts together. The counters are a record of what was fed in rather than an estimate, so nothing may be presumed before anything arrives. |
//! | [`eta_tracks_theory_exactly`] | noise | The noise share follows its closed form to the last digit through an injection phase and then a long run of real batches, because a batch's worth of decay is computed as a single power rather than accumulated a sample at a time. A quantity that gates clip width and decides when warm-up is over has to be reproducible, not merely approximately right. |
//! | [`eta_decreases_monotonically_under_real_data`] | noise | Every real batch lowers the noise share and none raises it, so warm-up influence is spent and never regained by ordinary operation. Monotonicity is what makes the share usable as a maturity signal: a threshold crossing means the same thing whenever it happens. |
//! | [`eta_increases_monotonically_under_noise`] | noise | Injection pushes the share back up from wherever real data drove it, batch by batch and without reversal. A cell whose model is re-warmed is therefore re-declared immature rather than left claiming a maturity its state no longer has. |
//! | [`eta_stays_in_unit_interval`] | noise | The share is a proportion and stays one under any interleaving of injected and real batches. Both updates are convex steps toward an endpoint inside the interval, so no mixture of workloads can carry it out of range and no consumer has to guard against a value that is not a fraction. |
//! | [`eta_converges_to_one_under_noise`] | noise | Indefinite injection holds the share at exactly one, its fixed point: noise cannot make a model more than entirely noise-taught. A long warm-up therefore has a stable end state rather than an accumulating one. |
//! | [`eta_decays_toward_zero_under_real_only`] | noise | A long enough run of real data drives the share to effectively nothing, so a warm-up is eventually forgotten completely. The decay is geometric in the number of samples seen, which is why the threshold that ends warm-up is reached within a bounded number of batches rather than merely approached. |
//! | [`observation_counters_mixed_sequence`] | noise | The counters tally samples rather than batches and keep the two kinds apart: a stretch of injection moves only the noise count, a stretch of real traffic only the other, and the total is their sum. A host can therefore still tell how much of a model's experience was synthetic long after the noise share itself has decayed away. |
//! | [`counters_track_real_only_sequence`] | noise | cites (´claim:noise:the-counters-tally-samples-not-batches-and-keep-the-two-kinds-apart´) |

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{as_slices, cfg_test, generate_noise};
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Initial conditions
// ════════════════════════════════════════════════════════════

/// A tracker that has seen nothing counts as entirely noise-taught. Everything
/// it will learn first comes from injected traffic, so the honest starting
/// position is that none of its state yet reflects real observations.
///
/// ´claim:noise:a-tracker-that-has-seen-nothing-counts-as-entirely-noise-taught´
/// ´test:crate:eta-starts-at-one-for-cold-tracker´
#[test]
fn eta_starts_at_one_for_cold_tracker() {
    let cfg = cfg_test();
    let tracker = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    assert!((tracker.maturity().noise_influence - 1.0).abs() < f64::EPSILON);
}

/// A fresh tracker claims no observations of either kind, and the total is
/// exactly the two counts together. The counters are a record of what was fed
/// in rather than an estimate, so nothing may be presumed before anything
/// arrives.
///
/// ´claim:noise:a-fresh-tracker-claims-no-observations-of-either-kind´
/// ´test:crate:counters-start-at-zero-for-cold-tracker´
#[test]
fn counters_start_at_zero_for_cold_tracker() {
    let cfg = cfg_test();
    let tracker = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    let m = tracker.maturity();
    assert_eq!(m.real_observations, 0);
    assert_eq!(m.noise_observations, 0);
    assert_eq!(m.total_observations(), 0);
}

// ════════════════════════════════════════════════════════════
//  η recurrence
// ════════════════════════════════════════════════════════════

/// The noise share follows its closed form to the last digit through an
/// injection phase and then a long run of real batches, because a batch's worth
/// of decay is computed as a single power rather than accumulated a sample at a
/// time. A quantity that gates clip width and decides when warm-up is over has
/// to be reproducible, not merely approximately right.
///
/// ´claim:noise:the-noise-share-matches-its-closed-form-because-a-batch-of-decay-is-one-power-not-a-loop´
/// ´test:crate:eta-tracks-theory-exactly´
#[test]
#[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap, clippy::suboptimal_flops)]
fn eta_tracks_theory_exactly() {
    let cfg = cfg_test();
    let dim = 128;
    let lambda = cfg.forgetting_factor;
    let noise_rounds = 10_usize;
    let noise_batch_size = cfg.noise_batch_size;
    let real_batch_size = 20_usize;
    let total_real_batches = 50;

    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Phase 1: noise injection.
    for _ in 0..noise_rounds {
        let noise = generate_noise(dim, noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let eta_after_noise = tracker.maturity().noise_influence;

    // Theoretical η after noise.
    let mut eta_theory = 1.0_f64;
    let lam_noise = lambda.powi(noise_batch_size as i32);
    for _ in 0..noise_rounds {
        eta_theory = lam_noise * eta_theory + (1.0 - lam_noise);
    }
    assert!(
        (eta_after_noise - eta_theory).abs() < 1e-10,
        "η after noise: got {eta_after_noise:.12}, theory {eta_theory:.12}"
    );

    // Phase 2: real batches — η decays as λ^(bs·n).
    let real_noise = generate_noise(dim, real_batch_size, &mut rng);
    let real_slices = as_slices(&real_noise);

    let mut max_error = 0.0_f64;
    for n in 1..=total_real_batches {
        tracker.observe(&real_slices, 0, false);
        let eta = tracker.maturity().noise_influence;
        let eta_real_theory = lambda.powi((real_batch_size * n) as i32) * eta_theory;
        max_error = max_error.max((eta - eta_real_theory).abs());
    }

    assert!(
        max_error < 1e-10,
        "η should match theory exactly (via powi), max error = {max_error:.2e}"
    );
}

// ════════════════════════════════════════════════════════════
//  Monotonicity & bounds
// ════════════════════════════════════════════════════════════

/// Every real batch lowers the noise share and none raises it, so warm-up
/// influence is spent and never regained by ordinary operation. Monotonicity is
/// what makes the share usable as a maturity signal: a threshold crossing means
/// the same thing whenever it happens.
///
/// ´claim:noise:every-real-batch-lowers-the-noise-share-and-none-raises-it´
/// ´test:crate:eta-decreases-monotonically-under-real-data´
#[test]
fn eta_decreases_monotonically_under_real_data() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Seed with noise.
    for _ in 0..5 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let mut prev_eta = tracker.maturity().noise_influence;

    // Real data — η must strictly decrease.
    for batch in 0..50 {
        let data = generate_noise(dim, 8, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
        let eta = tracker.maturity().noise_influence;
        assert!(
            eta < prev_eta + f64::EPSILON,
            "batch {batch}: η increased: {prev_eta:.10} → {eta:.10}"
        );
        prev_eta = eta;
    }
}

/// Injection pushes the share back up from wherever real data drove it, batch
/// by batch and without reversal. A cell whose model is re-warmed is therefore
/// re-declared immature rather than left claiming a maturity its state no
/// longer has.
///
/// ´claim:noise:injection-pushes-the-noise-share-back-up-without-reversal´
/// ´test:crate:eta-increases-monotonically-under-noise´
#[test]
fn eta_increases_monotonically_under_noise() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Drive η below 1.0 with real data.
    for _ in 0..30 {
        let data = generate_noise(dim, 8, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
    }

    let eta_before_noise = tracker.maturity().noise_influence;
    assert!(
        eta_before_noise < 0.5,
        "precondition: η should be well below 1.0, got {eta_before_noise}"
    );

    let mut prev_eta = eta_before_noise;

    // Noise — η must strictly increase toward 1.0.
    for batch in 0..30 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
        let eta = tracker.maturity().noise_influence;
        assert!(
            eta > prev_eta - f64::EPSILON,
            "batch {batch}: η decreased under noise: {prev_eta:.10} → {eta:.10}"
        );
        prev_eta = eta;
    }
}

/// The share is a proportion and stays one under any interleaving of injected
/// and real batches. Both updates are convex steps toward an endpoint inside
/// the interval, so no mixture of workloads can carry it out of range and no
/// consumer has to guard against a value that is not a fraction.
///
/// ´claim:noise:the-noise-share-is-a-proportion-under-any-interleaving-of-workloads´
/// ´test:crate:eta-stays-in-unit-interval´
#[test]
fn eta_stays_in_unit_interval() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(123);

    for round in 0..100 {
        let is_noise = round % 3 == 0; // ~1/3 noise, ~2/3 real
        let batch_size = if is_noise { cfg.noise_batch_size } else { 8 };
        let data = generate_noise(dim, batch_size, &mut rng);
        tracker.observe(&as_slices(&data), 0, is_noise);

        let eta = tracker.maturity().noise_influence;
        assert!((0.0..=1.0).contains(&eta), "round {round}: η out of [0, 1]: {eta}");
    }
}

// ════════════════════════════════════════════════════════════
//  Fixed points & limits
// ════════════════════════════════════════════════════════════

/// Indefinite injection holds the share at exactly one, its fixed point: noise
/// cannot make a model more than entirely noise-taught. A long warm-up
/// therefore has a stable end state rather than an accumulating one.
///
/// ´claim:noise:pure-injection-holds-the-noise-share-at-its-fixed-point-of-one´
/// ´test:crate:eta-converges-to-one-under-noise´
#[test]
fn eta_converges_to_one_under_noise() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(99);

    for _ in 1..=200 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let final_eta = tracker.maturity().noise_influence;
    assert!(
        (final_eta - 1.0).abs() < 1e-10,
        "η should stay at 1.0 under pure noise, got {final_eta}"
    );
}

/// A long enough run of real data drives the share to effectively nothing, so a
/// warm-up is eventually forgotten completely. The decay is geometric in the
/// number of samples seen, which is why the threshold that ends warm-up is
/// reached within a bounded number of batches rather than merely approached.
///
/// ´claim:noise:a-long-run-of-real-data-drives-the-noise-share-to-nothing´
/// ´test:crate:eta-decays-toward-zero-under-real-only´
#[test]
fn eta_decays_toward_zero_under_real_only() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(77);

    for _ in 0..200 {
        let data = generate_noise(dim, 8, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
    }

    let final_eta = tracker.maturity().noise_influence;
    assert!(
        final_eta < 1e-10,
        "η should decay toward 0 under real-only data, got {final_eta:.2e}"
    );
}

// ════════════════════════════════════════════════════════════
//  Observation counters
// ════════════════════════════════════════════════════════════

/// The counters tally samples rather than batches and keep the two kinds apart:
/// a stretch of injection moves only the noise count, a stretch of real traffic
/// only the other, and the total is their sum. A host can therefore still tell
/// how much of a model's experience was synthetic long after the noise share
/// itself has decayed away.
///
/// ´claim:noise:the-counters-tally-samples-not-batches-and-keep-the-two-kinds-apart´
/// ´test:crate:observation-counters-mixed-sequence´
#[test]
fn observation_counters_mixed_sequence() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // 10 noise rounds of batch_size=4.
    for _ in 0..10 {
        let noise = generate_noise(dim, 4, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }
    assert_eq!(tracker.maturity().noise_observations, 40);
    assert_eq!(tracker.maturity().real_observations, 0);

    // 5 real batches of batch_size=8.
    for _ in 0..5 {
        let data = generate_noise(dim, 8, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
    }
    assert_eq!(tracker.maturity().noise_observations, 40);
    assert_eq!(tracker.maturity().real_observations, 40);
    assert_eq!(tracker.maturity().total_observations(), 80);
}

/// With no injection at all the noise count stays at nothing while the real
/// count tracks every sample fed in, which pins the other end of the same
/// bookkeeping.
///
/// (´claim:noise:the-counters-tally-samples-not-batches-and-keep-the-two-kinds-apart´)
/// ´test:crate:counters-track-real-only-sequence´
#[test]
fn counters_track_real_only_sequence() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(55);

    for _ in 0..20 {
        let data = generate_noise(dim, 10, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
    }

    let m = tracker.maturity();
    assert_eq!(m.real_observations, 200);
    assert_eq!(m.noise_observations, 0);
    assert_eq!(m.total_observations(), 200);
}
