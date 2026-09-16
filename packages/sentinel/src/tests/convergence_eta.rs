// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! How much of what a tracker knows came out of a bottle.
//!
//! A cell is warmed on synthetic noise before it ever judges real traffic,
//! and the noise share η is the tracker's own estimate of how much of its
//! state that warming still accounts for. It uses the same exponential
//! forgetting cadence as the model: an injected batch pulls the share toward
//! one and a real batch pulls it toward zero. Observation counters remain
//! sample counts, while the influence states how much of the batch-updated
//! model is still synthetic.
//!
//! Three properties make it usable rather than merely descriptive. It is
//! a proportion under every workload, so a consumer never has to guard
//! against a value that is not a fraction. It moves in one direction per
//! kind of input, so a threshold crossing means the same thing whenever it
//! happens — which is what lets the clip exemption and the end of warm-up
//! be keyed to it. It also matches its batch-indexed closed form exactly and
//! is independent of the number of samples within each batch.
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
//! | [`eta_tracks_theory_exactly`] | noise | After each real batch the noise share equals its initial value times the model's forgetting factor raised to the number of batches, exactly matching the state it measures. |
//! | [`eta_decay_is_independent_of_batch_size`] | noise | One-sample and sixteen-sample batches apply the same decay to noise influence because each causes one model update. The observation counters still record their different sample counts. |
//! | [`eta_maturity_threshold_uses_model_batch_count`] | noise | The maturity threshold is crossed on the first batch for which the model's repeated forgetting factor takes influence below the threshold, with the count derived from that recurrence. |
//! | [`eta_decreases_monotonically_under_real_data`] | noise | Every real batch lowers the noise share and none raises it, so warm-up influence is spent and never regained by ordinary operation. Monotonicity is what makes the share usable as a maturity signal: a threshold crossing means the same thing whenever it happens. |
//! | [`eta_increases_monotonically_under_noise`] | noise | Injection pushes the share back up from wherever real data drove it, batch by batch and without reversal. A cell whose model is re-warmed is therefore re-declared immature rather than left claiming a maturity its state no longer has. |
//! | [`eta_stays_in_unit_interval`] | noise | The share is a proportion and stays one under any interleaving of injected and real batches. Both updates are convex steps toward an endpoint inside the interval, so no mixture of workloads can carry it out of range and no consumer has to guard against a value that is not a fraction. |
//! | [`eta_converges_to_one_under_noise`] | noise | Indefinite injection holds the share at exactly one, its fixed point: noise cannot make a model more than entirely noise-taught. A long warm-up therefore has a stable end state rather than an accumulating one. |
//! | [`eta_decays_toward_zero_under_real_only`] | noise | A long enough run of real batches drives the share below the maturity threshold, so warm-up is eventually forgotten on the same schedule as the model. |
//! | [`observation_counters_mixed_sequence`] | noise | The counters tally samples rather than batches and keep the two kinds apart: a stretch of injection moves only the noise count, a stretch of real traffic only the other, and the total is their sum. A host can therefore still tell how much of a model's experience was synthetic long after the noise share itself has decayed away. |
//! | [`counters_track_real_only_sequence`] | noise | cites (´claim:noise:the-counters-tally-samples-not-batches-and-keep-the-two-kinds-apart´) |

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{as_slices, cfg_test, generate_noise};
use crate::sentinel::tracker::SubspaceTracker;

const MATURITY_THRESHOLD: f64 = 0.01;

fn decay_crossing(initial: f64, lambda: f64) -> (usize, f64) {
    (1_usize..=usize::MAX)
        .scan(initial, |influence, batch| {
            *influence *= lambda;
            Some((batch, *influence))
        })
        .find(|(_, influence)| *influence < MATURITY_THRESHOLD)
        .expect("a validated forgetting factor must cross the maturity threshold")
}

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

/// The noise share follows the same recurrence as the model it describes. A
/// real batch applies λ once, so after `k` batches the initial influence has
/// been multiplied by λ exactly `k` times, independent of the rows in them.
///
/// ´claim:noise:the-noise-share-follows-the-models-batch-indexed-recurrence´
/// ´test:crate:eta-tracks-theory-exactly´
#[test]
fn eta_tracks_theory_exactly() {
    let cfg = cfg_test();
    let lambda = cfg.forgetting_factor;
    let mut tracker = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);
    let rows = generate_noise(128, 20, &mut rng);
    let slices = as_slices(&rows);
    let mut expected_eta = tracker.maturity().noise_influence;

    for batch in 1..=7 {
        tracker.observe(&slices, 0, false);
        expected_eta *= lambda;
        assert_eq!(
            tracker.maturity().noise_influence.to_bits(),
            expected_eta.to_bits(),
            "batch {batch} must apply one model-decay step"
        );
    }
}

/// One-sample and sixteen-sample batches each evolve the learned model once,
/// so they must also apply the same single decay to its warm-up influence. The
/// separate observation counters continue to record how many rows arrived.
///
/// ´claim:noise:one-model-update-applies-one-noise-influence-decay-regardless-of-batch-size´
/// ´test:crate:eta-decay-is-independent-of-batch-size´
#[test]
fn eta_decay_is_independent_of_batch_size() {
    let cfg = cfg_test();
    let mut one = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    let mut sixteen = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(43);
    let one_row = generate_noise(128, 1, &mut rng);
    let sixteen_rows = generate_noise(128, 16, &mut rng);

    one.observe(&as_slices(&one_row), 0, false);
    sixteen.observe(&as_slices(&sixteen_rows), 0, false);

    assert_eq!(one.maturity().noise_influence.to_bits(), cfg.forgetting_factor.to_bits());
    assert_eq!(sixteen.maturity().noise_influence.to_bits(), cfg.forgetting_factor.to_bits());
    assert_eq!(
        one.maturity().noise_influence.to_bits(),
        sixteen.maturity().noise_influence.to_bits()
    );
    assert_eq!(one.maturity().real_observations, 1);
    assert_eq!(sixteen.maturity().real_observations, 16);
}

/// The maturity crossing count comes directly from repeatedly applying the
/// configured forgetting factor until influence is strictly below the same
/// threshold the tracker uses. No observed run supplies the expected count.
///
/// ´claim:noise:maturity-crosses-when-the-models-batch-decay-crosses-the-threshold´
/// ´test:crate:eta-maturity-threshold-uses-model-batch-count´
#[test]
fn eta_maturity_threshold_uses_model_batch_count() {
    let cfg = cfg_test();
    let (crossing_batch, expected_eta) = decay_crossing(1.0, cfg.forgetting_factor);

    let mut tracker = SubspaceTracker::new(128, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(44);
    let rows = generate_noise(128, 16, &mut rng);
    let slices = as_slices(&rows);

    for batch in 1..=crossing_batch {
        tracker.observe(&slices, 0, false);
        if batch < crossing_batch {
            assert!(tracker.maturity().noise_influence >= MATURITY_THRESHOLD);
        }
    }

    assert!(tracker.maturity().noise_influence < MATURITY_THRESHOLD);
    assert_eq!(tracker.maturity().noise_influence.to_bits(), expected_eta.to_bits());
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

/// A long enough run of real batches drives the share below the maturity
/// threshold, so warm-up is forgotten on the same geometric cadence as the
/// model rather than on a schedule determined by batch size.
///
/// ´claim:noise:a-long-run-of-real-data-drives-the-noise-share-to-nothing´
/// ´test:crate:eta-decays-toward-zero-under-real-only´
#[test]
fn eta_decays_toward_zero_under_real_only() {
    let cfg = cfg_test();
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(77);

    let (crossing_batch, _) = decay_crossing(tracker.maturity().noise_influence, cfg.forgetting_factor);
    for _ in 0..crossing_batch {
        let data = generate_noise(dim, 8, &mut rng);
        tracker.observe(&as_slices(&data), 0, false);
    }

    let final_eta = tracker.maturity().noise_influence;
    assert!(
        final_eta < MATURITY_THRESHOLD,
        "η should cross the model's maturity threshold, got {final_eta:.2e}"
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
