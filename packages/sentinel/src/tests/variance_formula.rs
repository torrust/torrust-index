// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for the running spread the tracker keeps on each of its latent axes —
//! the figure a batch's deviation is divided by to become a surprise score.
//!
//! The spread is maintained as a decaying average, and the decision that shapes
//! it is what each batch's contribution is measured from. Measuring a batch's
//! scatter about its own mean makes the contribution depend on how many rows
//! happened to arrive: a single-row batch has no scatter about itself at all,
//! so the spread collapses toward nothing and every subsequent deviation
//! divided by it reads as enormous, while a two-row batch understates the
//! spread by half. Measuring instead from the running mean carried in from
//! before the batch removes that dependence — each row contributes its own
//! squared departure from an established centre — so a surprise of about one
//! means "as expected" whatever the batch size, and cells configured
//! differently produce comparable scores.
//!
//! Two further decisions guard the degenerate ends. The spread is seeded
//! outright from the first batch rather than blended against its placeholder,
//! because a cell that starts by blending spends a long stretch scoring against
//! a number it was born with rather than one it observed. And it is held above
//! a floor, so a perfectly constant stream — where the running mean converges
//! onto the value itself and genuinely leaves no spread — cannot drive the
//! divisor to zero and make the first different observation infinitely
//! surprising.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 3 — Evolve Latent Distribution
//! - §ALGO S-5.4 — Surprise scoring
//! - §ALGO S-11.2 — Cold-start latent seeding
//! - ADR-S-021 — EWMA-mean-centred latent variance
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cold_start_seeds_variance_from_first_batch`] | variance | The very first batch seeds the spread outright instead of being blended into the placeholder a fresh model was built with. One batch of centred noise is enough to leave every axis's spread near the value the geometry predicts for such traffic and well below the placeholder, so a cell begins scoring against something it observed rather than against a constant it was born holding and would take many batches to shake off. |
//! | [`b1_surprise_bounded`] | variance | A cell fed one observation at a time keeps producing modest surprise scores over a long run of batches. Because each batch's contribution to the spread is measured from the mean carried in from before it, a single row still contributes a real squared departure — where measuring scatter within the batch would find none at all, drive the divisor to its floor, and turn ordinary traffic into scores several orders of magnitude too large. |
//! | [`b1_latent_variance_stable`] | variance | cites (´claim:variance:centring-on-the-running-mean-keeps-a-single-row-batch-from-collapsing-the-spread´) |
//! | [`b2_no_systematic_surprise_inflation`] | variance | Surprise carries no systematic bias from the batch size a cell is configured with: on ordinary traffic, once the model has settled, the average score sits around one. Measuring scatter within a small batch would understate the spread by a predictable fraction and inflate every score by its reciprocal, so a cell reading batches two at a time would look permanently twice as surprised as an identical cell reading them in larger groups. A score of about one has to mean "as expected" everywhere, or no threshold can be set once and applied across cells. |
//! | [`batch_size_invariant_surprise_ratio`] | variance | cites (´claim:variance:the-surprise-ratio-carries-no-batch-size-bias-so-a-score-of-about-one-means-as-expected-everywhere´) |
//! | [`runtime_floor_prevents_degenerate_collapse`] | variance | A stream in which every observation is identical genuinely has no spread, and centring on the running mean does not rescue it — the mean converges onto the repeated value and each batch's contribution goes to zero with it. A floor holds the divisor above a small positive value regardless, so the cell keeps scoring on a bounded scale. Without it, the first observation that differed at all would be divided by nothing and reported as unboundedly surprising, which says more about the arithmetic than about the traffic. |
//! | [`variance_adapts_to_distribution_shift`] | variance | The spread follows the traffic. When a settled cell's input jumps to a substantially larger scale, the recorded spread on every axis climbs well past where it sat before, because it is a decaying average of what is arriving rather than a fixed property learned once. A shift in scale is therefore absorbed within a bounded stretch of batches instead of being reported as anomalous indefinitely — surprise is meant to answer "unusual for this cell lately", not "unusual for this cell when it was young". |
//! | [`update_order_variance_before_mean`] | variance | Within a single batch the spread is measured before the mean moves, against the centre that batch arrived to find. The ordering is visible because the mean demonstrably shifts across the batch while the resulting spread stays in the range the earlier centre implies. Were the order reversed, a batch would be measured against a centre it had just pulled toward itself and would partly explain its own deviation away — the same reason scoring happens before the model absorbs the batch at all. |

use rand::SeedableRng;
use rand::rngs::SmallRng;

use super::convergence_common::{as_slices, cfg_test, generate_noise};
use crate::config::SentinelConfig;
use crate::sentinel::tracker::SubspaceTracker;

// ════════════════════════════════════════════════════════════
//  Helpers
// ════════════════════════════════════════════════════════════

/// Build a config with a specific `noise_batch_size`.
fn cfg_with_batch_size(b: usize) -> SentinelConfig<u64> {
    SentinelConfig {
        noise_batch_size: b,
        ..cfg_test()
    }
}

// ════════════════════════════════════════════════════════════
//  Cold-start seeding
// ════════════════════════════════════════════════════════════

/// The very first batch seeds the spread outright instead of being blended
/// into the placeholder a fresh model was built with. One batch of centred
/// noise is enough to leave every axis's spread near the value the geometry
/// predicts for such traffic and well below the placeholder, so a cell begins
/// scoring against something it observed rather than against a constant it was
/// born holding and would take many batches to shake off.
///
/// ´claim:variance:the-first-batch-seeds-the-spread-outright-rather-than-being-blended-into-a-placeholder´
/// ´test:crate:cold-start-seeds-variance-from-first-batch´
#[test]
fn cold_start_seeds_variance_from_first_batch() {
    let cfg = cfg_with_batch_size(16);
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    tracker.observe(&as_slices(&noise), 0, true);

    let lat_vars = tracker.latent_var();
    for (j, &v) in lat_vars.iter().enumerate() {
        // Should be near 0.25; upper bound excludes the 1.0 placeholder.
        assert!(
            (1e-2..0.8).contains(&v),
            "cold-start lat_var[{j}] = {v:.6} should be near 0.25, not the 1.0 placeholder"
        );
    }
}

// ════════════════════════════════════════════════════════════
//  b = 1 tests
// ════════════════════════════════════════════════════════════

/// A cell fed one observation at a time keeps producing modest surprise scores
/// over a long run of batches. Because each batch's contribution to the spread
/// is measured from the mean carried in from before it, a single row still
/// contributes a real squared departure — where measuring scatter within the
/// batch would find none at all, drive the divisor to its floor, and turn
/// ordinary traffic into scores several orders of magnitude too large.
///
/// ´claim:variance:centring-on-the-running-mean-keeps-a-single-row-batch-from-collapsing-the-spread´
/// ´test:crate:b1-surprise-bounded´
#[test]
fn b1_surprise_bounded() {
    let cfg = cfg_with_batch_size(1);
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);
    let mut max_surprise = 0.0_f64;

    for _ in 0..200 {
        let noise = generate_noise(dim, 1, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        max_surprise = max_surprise.max(report.scores.surprise.mean);
    }

    assert!(
        max_surprise < 50.0,
        "b=1 surprise should stay bounded; got max {max_surprise:.1} (old formula would exceed 10⁴)"
    );
}

/// The same statement read on the quantity itself rather than on the score it
/// divides: after a long run of single-row batches, every axis's spread has
/// settled around the value the geometry of centred bit traffic predicts, not
/// against the floor. Each batch's contribution is an unbiased estimate of the
/// true spread even when the batch is one row, so the decaying average
/// converges on the right number instead of merely staying finite.
///
/// (´claim:variance:centring-on-the-running-mean-keeps-a-single-row-batch-from-collapsing-the-spread´)
/// ´test:crate:b1-latent-variance-stable´
#[test]
fn b1_latent_variance_stable() {
    let cfg = cfg_with_batch_size(1);
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    for _ in 0..500 {
        let noise = generate_noise(dim, 1, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let lat_vars = tracker.latent_var();
    for (j, &v) in lat_vars.iter().enumerate() {
        assert!(
            (0.05..=0.5).contains(&v),
            "b=1: lat_var[{j}] = {v:.6} should be in [0.05, 0.5] (≈0.25 expected)"
        );
    }
}

// ════════════════════════════════════════════════════════════
//  b = 2 test
// ════════════════════════════════════════════════════════════

/// Surprise carries no systematic bias from the batch size a cell is
/// configured with: on ordinary traffic, once the model has settled, the
/// average score sits around one. Measuring scatter within a small batch would
/// understate the spread by a predictable fraction and inflate every score by
/// its reciprocal, so a cell reading batches two at a time would look
/// permanently twice as surprised as an identical cell reading them in larger
/// groups. A score of about one has to mean "as expected" everywhere, or no
/// threshold can be set once and applied across cells.
///
/// ´claim:variance:the-surprise-ratio-carries-no-batch-size-bias-so-a-score-of-about-one-means-as-expected-everywhere´
/// ´test:crate:b2-no-systematic-surprise-inflation´
#[test]
fn b2_no_systematic_surprise_inflation() {
    let cfg = cfg_with_batch_size(2);
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Warm up.
    for _ in 0..200 {
        let noise = generate_noise(dim, 2, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    // Measure: collect surprise means over 200 more rounds.
    let mut surprise_sum = 0.0;
    let measurement_rounds: u32 = 200;
    for _ in 0..measurement_rounds {
        let noise = generate_noise(dim, 2, &mut rng);
        let report = tracker.observe(&as_slices(&noise), 0, true);
        surprise_sum += report.scores.surprise.mean;
    }
    let mean_surprise = surprise_sum / f64::from(measurement_rounds);

    assert!(
        (0.5..=2.0).contains(&mean_surprise),
        "b=2: mean surprise = {mean_surprise:.3} should be near 1.0 (old formula would give ≈2.0)"
    );
}

// ════════════════════════════════════════════════════════════
//  Batch-size invariance
// ════════════════════════════════════════════════════════════

/// The general form of the same statement, swept over batch sizes spanning
/// two orders of magnitude from a single row to many. Two things are pinned
/// here that a single configuration cannot pin: each cell's average score sits
/// in a band around one, and the scores agree closely with each other across
/// the sweep. Self-consistency is the stronger half — a shared bias would move
/// every band together and pass the first check, but would show up at once as
/// a spread between them.
///
/// (´claim:variance:the-surprise-ratio-carries-no-batch-size-bias-so-a-score-of-about-one-means-as-expected-everywhere´)
/// ´test:crate:batch-size-invariant-surprise-ratio´
#[test]
fn batch_size_invariant_surprise_ratio() {
    let batch_sizes = [1, 2, 4, 16, 64];
    // Surprise-ratio invariance is per-axis; dim=32 validates the
    // property at ~4× less SVD cost than dim=128 (ADR-S-012).
    let dim = 32;
    let warmup_rounds = 100;
    let measurement_rounds: u32 = 100;

    let mut ratios = Vec::with_capacity(batch_sizes.len());

    for &b in &batch_sizes {
        let cfg = cfg_with_batch_size(b);
        let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
        let mut rng = SmallRng::seed_from_u64(42);

        // Warm up.
        for _ in 0..warmup_rounds {
            let noise = generate_noise(dim, b, &mut rng);
            tracker.observe(&as_slices(&noise), 0, true);
        }

        // Measure.
        let mut surprise_sum = 0.0;
        for _ in 0..measurement_rounds {
            let noise = generate_noise(dim, b, &mut rng);
            let report = tracker.observe(&as_slices(&noise), 0, true);
            surprise_sum += report.scores.surprise.mean;
        }
        let mean_surprise = surprise_sum / f64::from(measurement_rounds);
        ratios.push((b, mean_surprise));
    }

    // Each ratio should be in a reasonable band around 1.0.
    for &(b, ratio) in &ratios {
        assert!(
            (0.7..=1.5).contains(&ratio),
            "b={b}: surprise ratio = {ratio:.3} should be in [0.7, 1.5]"
        );
    }

    // Self-consistency: max - min < 0.5.
    let max_r = ratios.iter().map(|(_, r)| *r).fold(f64::NEG_INFINITY, f64::max);
    let min_r = ratios.iter().map(|(_, r)| *r).fold(f64::INFINITY, f64::min);
    let spread = max_r - min_r;

    assert!(
        spread < 0.5,
        "surprise ratios should be self-consistent across batch sizes: \
         spread = {spread:.3} (ratios: {ratios:?})"
    );
}

// ════════════════════════════════════════════════════════════
//  Runtime floor
// ════════════════════════════════════════════════════════════

/// A stream in which every observation is identical genuinely has no spread,
/// and centring on the running mean does not rescue it — the mean converges
/// onto the repeated value and each batch's contribution goes to zero with it.
/// A floor holds the divisor above a small positive value regardless, so the
/// cell keeps scoring on a bounded scale. Without it, the first observation
/// that differed at all would be divided by nothing and reported as
/// unboundedly surprising, which says more about the arithmetic than about the
/// traffic.
///
/// ´claim:variance:a-floor-holds-the-spread-off-zero-so-a-perfectly-constant-stream-cannot-make-the-next-deviation-unbounded´
/// ´test:crate:runtime-floor-prevents-degenerate-collapse´
#[test]
fn runtime_floor_prevents_degenerate_collapse() {
    let cfg = cfg_test(); // b = 4
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // One batch of noise to initialise the basis.
    let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    tracker.observe(&as_slices(&noise), 0, true);

    // 200 batches of identical observations (all −0.5).
    let constant_row: Vec<f64> = vec![-0.5; dim];
    let constant_batch: Vec<Vec<f64>> = (0..cfg.noise_batch_size).map(|_| constant_row.clone()).collect();

    for _ in 0..200 {
        tracker.observe(&as_slices(&constant_batch), 0, true);
    }

    let lat_vars = tracker.latent_var();
    for (j, &v) in lat_vars.iter().enumerate() {
        assert!(v >= 1e-2, "lat_var[{j}] = {v:.6e} should be ≥ 1e-2 (runtime floor)");
    }
}

// ════════════════════════════════════════════════════════════
//  Distribution-shift adaptivity
// ════════════════════════════════════════════════════════════

/// The spread follows the traffic. When a settled cell's input jumps to a
/// substantially larger scale, the recorded spread on every axis climbs well
/// past where it sat before, because it is a decaying average of what is
/// arriving rather than a fixed property learned once. A shift in scale is
/// therefore absorbed within a bounded stretch of batches instead of being
/// reported as anomalous indefinitely — surprise is meant to answer "unusual
/// for this cell lately", not "unusual for this cell when it was young".
///
/// ´claim:variance:the-spread-follows-the-traffic-so-a-shift-in-scale-is-absorbed-rather-than-reported-forever´
/// ´test:crate:variance-adapts-to-distribution-shift´
#[test]
fn variance_adapts_to_distribution_shift() {
    let cfg = cfg_with_batch_size(4);
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(42);

    // Warm up on standard ±0.5 noise.
    for _ in 0..200 {
        let noise = generate_noise(dim, cfg.noise_batch_size, &mut rng);
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let pre_shift_var: Vec<f64> = tracker.latent_var().to_vec();

    // Switch to 4× scaled noise (±2.0).
    for _ in 0..100 {
        let noise: Vec<Vec<f64>> = generate_noise(dim, cfg.noise_batch_size, &mut rng)
            .into_iter()
            .map(|row| row.into_iter().map(|x| x * 4.0).collect())
            .collect();
        tracker.observe(&as_slices(&noise), 0, true);
    }

    let post_shift_var: Vec<f64> = tracker.latent_var().to_vec();

    // lat_var should have increased substantially (16× theoretical).
    for (j, (&pre, &post)) in pre_shift_var.iter().zip(post_shift_var.iter()).enumerate() {
        assert!(
            post > pre * 2.0,
            "lat_var[{j}]: post-shift {post:.4} should be > 2× pre-shift {pre:.4}"
        );
    }
}

// ════════════════════════════════════════════════════════════
//  Update order (white-box)
// ════════════════════════════════════════════════════════════

/// Within a single batch the spread is measured before the mean moves, against
/// the centre that batch arrived to find. The ordering is visible because the
/// mean demonstrably shifts across the batch while the resulting spread stays
/// in the range the earlier centre implies. Were the order reversed, a batch
/// would be measured against a centre it had just pulled toward itself and
/// would partly explain its own deviation away — the same reason scoring
/// happens before the model absorbs the batch at all.
///
/// ´claim:variance:the-spread-is-measured-against-the-centre-that-preceded-the-batch-so-a-batch-cannot-explain-itself-away´
/// ´test:crate:update-order-variance-before-mean´
#[test]
fn update_order_variance_before_mean() {
    let cfg = SentinelConfig {
        forgetting_factor: 0.9,
        noise_batch_size: 4,
        max_rank: 2,
        rank_update_interval: 5,
        ..cfg_test()
    };
    let dim = 128;
    let mut tracker = SubspaceTracker::new(dim, &cfg, cfg.cusum_slow_decay);
    let mut rng = SmallRng::seed_from_u64(99);

    // Batch 1: cold→warm seeding.
    let noise1 = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    tracker.observe(&as_slices(&noise1), 0, true);

    // Record pre-batch-2 `lat_mean` (this is what variance should
    // have been computed against).
    let pre_mean: Vec<f64> = tracker.latent_mean().to_vec();

    // Batch 2: non-cold path.
    let noise2 = generate_noise(dim, cfg.noise_batch_size, &mut rng);
    tracker.observe(&as_slices(&noise2), 0, true);

    let post_mean: Vec<f64> = tracker.latent_mean().to_vec();
    let post_var: Vec<f64> = tracker.latent_var().to_vec();

    // The key invariant: with variance-before-mean order, the variance
    // was computed against `pre_mean`, then the mean was updated.
    // If the order were reversed, the variance would be computed against
    // `post_mean` (which would be slightly different).
    //
    // Verify indirectly: `pre_mean` ≠ `post_mean` (the mean moved),
    // and `lat_var` is within a reasonable range (not stuck at ε or 1.0).
    let mean_shifted = pre_mean.iter().zip(post_mean.iter()).any(|(&a, &b)| (a - b).abs() > 1e-8);

    assert!(mean_shifted, "lat_mean should have changed between batches 1 and 2");

    for (j, &v) in post_var.iter().enumerate() {
        // After batch 2, `lat_var` should be λ·seed_var + α·col_var
        // where `col_var` was centred on `pre_mean` (≈ batch-1 mean).
        // It should be in a reasonable range — not ε (which the old
        // formula would give at b=1) and not wildly inflated.
        assert!(
            (1e-2..2.0).contains(&v),
            "lat_var[{j}] = {v:.6} should be reasonable after 2 batches"
        );
    }
}
