// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`starts_cold`] | ewma | A newly constructed baseline is cold, and the mean and spread it reports are placeholders of one rather than anything measured. They are deliberately wide so that a value scored before any real data has arrived cannot come back with an extreme departure. |
//! | [`first_update_warms`] | ewma | One update is the whole of warming: a single batch takes the baseline out of its cold state for good. There is no minimum sample count to reach and no separate warming phase to wait out at this level. |
//! | [`first_update_sets_mean_to_batch_mean`] | ewma | The first batch is adopted outright: the mean becomes the batch's own mean exactly, with no trace of the placeholder blended in. Decaying towards the first real data instead would leave the baseline anchored for many batches to a value that was never an observation. |
//! | [`first_update_single_value_keeps_unit_variance`] | ewma | A batch of one carries a mean but no spread, so the mean is taken and the variance is left exactly as it stood. Computing a deviation from a single sample would give zero — a spread the data does not support, and one that would make every subsequent value look like an outlier. |
//! | [`first_update_multi_value_sets_sample_variance`] | ewma | A first batch of more than one sets the spread from its own mean squared deviation, taken about the batch mean and divided by the count with no correction applied. Both the centre and the spread the baseline starts from are therefore measurements rather than defaults. |
//! | [`reset_cold_restores_placeholder_state`] | ewma | Resetting a warm baseline returns it to exactly the state construction left it in — placeholders restored and warmth withdrawn — rather than merely clearing a flag over learned numbers. What it learned is discarded, not hidden. |
//! | [`reset_cold_allows_re_warming`] | ewma | After a reset the next batch is adopted outright, exactly as the very first one was: the mean lands on the new batch's value with nothing of the discarded baseline pulling it back. Resetting therefore genuinely re-starts the baseline rather than leaving it to decay out of its old position. |
//! | [`seed_from_copies_warm_state`] | ewma | Seeding transfers both what a baseline learned and the fact that it learned it: the receiver takes the source's mean and spread and becomes warm. This is how a slow baseline is started from a fast one that has already converged, so the pair begin in agreement instead of the slow one spending its warm-up disagreeing with a baseline that is already right. |
//! | [`seed_from_cold_source_does_not_warm_target`] | ewma | Warmth is never manufactured by seeding. A cold source hands over its placeholder numbers but leaves the receiver cold, so a baseline seeded before anything was learned still takes the cold path on its own first batch rather than blending against values nothing measured. |
//! | [`update_empty_is_noop`] | ewma | A batch with nothing in it leaves both the mean and the spread exactly where they were. An idle interval is therefore not a data point: the baseline does not drift simply because time passed without observations. |
//! | [`outliers_are_rejected`] | ewma | A warm baseline refuses values above its own ceiling before it learns anything, and a batch consisting entirely of such values moves it not at all. This is the poisoning defence: an attacker cannot walk the notion of normal upward by feeding extremes, because the extremes are precisely what never reaches the baseline. |
//! | [`update_single_value_does_not_update_variance`] | ewma | cites (´claim:ewma:a-batch-of-one-carries-no-spread-so-the-variance-is-left-untouched´) |
//! | [`update_skips_clipping_when_cold`] | ewma | While cold there is no ceiling to clip against, so nothing is rejected: a first batch far above the placeholder mean is accepted whole and becomes the baseline. The placeholders are not a real notion of normal, and filtering against them would let an arbitrary constant decide which of the first real observations the sentinel was allowed to see. |
//! | [`update_raw_empty_is_noop`] | ewma | cites (´claim:ewma:an-empty-batch-teaches-the-baseline-nothing´) |
//! | [`update_raw_matches_update_when_no_clipping`] | ewma | The unclipped path is the clipped one with only the filter removed: given a clip so wide that nothing could be rejected, the two produce the same mean and the same spread. Callers that do their own outlier rejection get identical arithmetic, so the two entry points cannot drift into disagreeing about what a batch means. |
//! | [`decays_toward_new_data`] | ewma | Sustained new data pulls the baseline away from what it learned before: a mean established at one level and then fed a different one repeatedly ends up near the new level. The baseline tracks the present rather than averaging over all history, which is what makes it a moving notion of normal rather than a permanent one. |
//! | [`higher_decay_forgets_slower`] | ewma | The decay factor is the length of the baseline's memory. Two baselines started at the same level and fed the same contradicting data diverge in the expected direction: the one with the higher factor still holds more of the original level. This is what lets a fast and a slow baseline over the same stream disagree usefully, which is the whole basis of drift detection. |
//! | [`z_score_is_zero_at_mean`] | ewma | A value sitting at the baseline scores essentially nothing. The z-score measures signed departure from what the baseline expects, so agreement with it is the origin of the scale rather than a point somewhere along it. |
//! | [`z_score_positive_above_mean`] | ewma | cites (´claim:ewma:the-z-score-is-signed-departure-from-the-baseline-and-zero-at-it´) |
//! | [`z_score_negative_below_mean`] | ewma | cites (´claim:ewma:the-z-score-is-signed-departure-from-the-baseline-and-zero-at-it´) |
//! | [`ceiling_returns_infinity_when_cold`] | ewma | cites (´claim:ewma:while-cold-there-is-no-ceiling-to-clip-against-so-nothing-is-rejected´) |
//! | [`ceiling_returns_mean_plus_sigmas_when_warm`] | ewma | Once warm, the ceiling stands a fixed number of standard deviations above the mean — the requested multiple of the baseline's own square-rooted spread, added to its own centre. The threshold is therefore relative to what this cell has learned, not an absolute score chosen in advance for every cell alike. |
//! | [`clip_sigmas_affects_ceiling`] | ewma | The clip setting is a monotone dial on how much the baseline is willing to learn from: given the same elevated value, a baseline clipping at a wide multiple moves at least as far as one clipping tightly. Tightening the setting can only ever admit less, so an operator turning it down is trading responsiveness for poisoning resistance and never the reverse. |
//! | [`snapshot_matches_state`] | ewma | The snapshot a report carries holds the same mean and spread the baseline's own accessors report. What a caller reads out of a report is the state the engine is scoring against, not a rounded or separately derived summary of it. |
//! | [`variance_floor_is_respected`] | ewma | A batch of identical values has no deviation at all, yet the spread does not reach zero: a floor holds it above. Without it a perfectly quiet period would collapse the spread, the ceiling would close onto the mean, and every subsequent value — however ordinary — would be rejected as an outlier, leaving the baseline permanently frozen at the quiet level. |

//! Unit tests for [`EwmaStats`](crate::ewma::EwmaStats) — the running mean
//! and spread every anomaly axis is scored against.
//!
//! A baseline has two lives. Cold, it holds placeholders rather than
//! measurements, and the first batch it sees is adopted outright: there is
//! nothing yet to blend with, and nothing to call an outlier against, so the
//! clip filter does not run. Warm, it blends each batch in by the decay
//! factor and refuses anything above its own ceiling first.
//!
//! That refusal is what makes the baseline hard to poison. Anomaly scores are
//! non-negative and right-skewed — an attacker inflates them and never
//! deflates them — so only the upper tail is clipped, and a batch consisting
//! entirely of outliers teaches the baseline nothing at all rather than
//! dragging it upward. The variance floor guards the same property from the
//! other side: a perfectly quiet period cannot collapse the spread to zero
//! and thereby turn every later value into an outlier.

use crate::ewma::*;

// ── Construction & initial state ────────────────────────────

/// A newly constructed baseline is cold, and the mean and spread it reports
/// are placeholders of one rather than anything measured. They are
/// deliberately wide so that a value scored before any real data has arrived
/// cannot come back with an extreme departure.
///
/// ´claim:ewma:a-fresh-baseline-is-cold-and-holds-placeholders-not-measurements´
/// ´test:crate:starts-cold´
#[test]
fn starts_cold() {
    let stats = EwmaStats::new(0.99);
    assert!(!stats.is_warm());
    assert!((stats.mean() - 1.0).abs() < f64::EPSILON);
    assert!((stats.variance() - 1.0).abs() < f64::EPSILON);
}

// ── Cold → warm transition ──────────────────────────────────

/// One update is the whole of warming: a single batch takes the baseline out
/// of its cold state for good. There is no minimum sample count to reach and
/// no separate warming phase to wait out at this level.
///
/// ´claim:ewma:one-update-is-enough-to-turn-a-cold-baseline-warm´
/// ´test:crate:first-update-warms´
#[test]
fn first_update_warms() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[2.0, 3.0, 4.0], 3.0);
    assert!(stats.is_warm());
}

/// The first batch is adopted outright: the mean becomes the batch's own
/// mean exactly, with no trace of the placeholder blended in. Decaying
/// towards the first real data instead would leave the baseline anchored for
/// many batches to a value that was never an observation.
///
/// ´claim:ewma:the-first-batch-is-adopted-outright-rather-than-blended-with-the-placeholder´
/// ´test:crate:first-update-sets-mean-to-batch-mean´
#[test]
fn first_update_sets_mean_to_batch_mean() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[2.0, 4.0, 6.0], 3.0);
    // Cold-path sets mean = batch mean = (2+4+6)/3 = 4.0
    assert!((stats.mean() - 4.0).abs() < f64::EPSILON);
}

/// A batch of one carries a mean but no spread, so the mean is taken and the
/// variance is left exactly as it stood. Computing a deviation from a single
/// sample would give zero — a spread the data does not support, and one that
/// would make every subsequent value look like an outlier.
///
/// ´claim:ewma:a-batch-of-one-carries-no-spread-so-the-variance-is-left-untouched´
/// ´test:crate:first-update-single-value-keeps-unit-variance´
#[test]
fn first_update_single_value_keeps_unit_variance() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[5.0], 3.0);
    assert!(stats.is_warm());
    assert!((stats.mean() - 5.0).abs() < f64::EPSILON);
    // Single-element cold-path: variance stays at the 1.0 initial
    // (the code only computes variance when normals.len() > 1).
    assert!((stats.variance() - 1.0).abs() < f64::EPSILON);
}

/// A first batch of more than one sets the spread from its own mean squared
/// deviation, taken about the batch mean and divided by the count with no
/// correction applied. Both the centre and the spread the baseline starts
/// from are therefore measurements rather than defaults.
///
/// ´claim:ewma:a-first-batch-of-more-than-one-sets-the-spread-from-its-own-deviation´
/// ´test:crate:first-update-multi-value-sets-sample-variance´
#[test]
fn first_update_multi_value_sets_sample_variance() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[0.0, 10.0], 3.0);
    // mean = 5.0, variance = ((0-5)² + (10-5)²) / 2 = 25.0
    assert!((stats.mean() - 5.0).abs() < f64::EPSILON);
    assert!((stats.variance() - 25.0).abs() < 1e-12);
}

// ── reset_cold() ────────────────────────────────────────────

/// Resetting a warm baseline returns it to exactly the state construction
/// left it in — placeholders restored and warmth withdrawn — rather than
/// merely clearing a flag over learned numbers. What it learned is discarded,
/// not hidden.
///
/// ´claim:ewma:resetting-cold-returns-a-baseline-to-the-state-construction-left-it-in´
/// ´test:crate:reset-cold-restores-placeholder-state´
#[test]
fn reset_cold_restores_placeholder_state() {
    let mut stats = EwmaStats::new(0.95);
    stats.update(&[10.0, 20.0, 30.0], 3.0);
    assert!(stats.is_warm());

    stats.reset_cold();
    assert!(!stats.is_warm());
    assert!((stats.mean() - 1.0).abs() < f64::EPSILON);
    assert!((stats.variance() - 1.0).abs() < f64::EPSILON);
}

/// After a reset the next batch is adopted outright, exactly as the very
/// first one was: the mean lands on the new batch's value with nothing of the
/// discarded baseline pulling it back. Resetting therefore genuinely
/// re-starts the baseline rather than leaving it to decay out of its old
/// position.
///
/// ´claim:ewma:a-reset-baseline-re-warms-by-the-cold-path-not-by-decay´
/// ´test:crate:reset-cold-allows-re-warming´
#[test]
fn reset_cold_allows_re_warming() {
    let mut stats = EwmaStats::new(0.95);
    stats.update(&[10.0, 10.0, 10.0], 3.0);
    stats.reset_cold();
    stats.update(&[42.0, 42.0, 42.0], 3.0);
    assert!(stats.is_warm());
    assert!((stats.mean() - 42.0).abs() < f64::EPSILON);
}

// ── seed_from() ─────────────────────────────────────────────

/// Seeding transfers both what a baseline learned and the fact that it
/// learned it: the receiver takes the source's mean and spread and becomes
/// warm. This is how a slow baseline is started from a fast one that has
/// already converged, so the pair begin in agreement instead of the slow one
/// spending its warm-up disagreeing with a baseline that is already right.
///
/// ´claim:ewma:seeding-copies-the-sources-baseline-and-its-warmth-together´
/// ´test:crate:seed-from-copies-warm-state´
#[test]
fn seed_from_copies_warm_state() {
    let mut source = EwmaStats::new(0.99);
    source.update(&[5.0, 10.0, 15.0], 3.0);

    let mut target = EwmaStats::new(0.99);
    assert!(!target.is_warm());
    target.seed_from(&source);

    assert!(target.is_warm());
    assert!((target.mean() - source.mean()).abs() < f64::EPSILON);
    assert!((target.variance() - source.variance()).abs() < f64::EPSILON);
}

/// Warmth is never manufactured by seeding. A cold source hands over its
/// placeholder numbers but leaves the receiver cold, so a baseline seeded
/// before anything was learned still takes the cold path on its own first
/// batch rather than blending against values nothing measured.
///
/// ´claim:ewma:seeding-from-a-cold-source-copies-placeholders-without-conferring-warmth´
/// ´test:crate:seed-from-cold-source-does-not-warm-target´
#[test]
fn seed_from_cold_source_does_not_warm_target() {
    let source = EwmaStats::new(0.99); // never updated — cold
    let mut target = EwmaStats::new(0.99);
    target.seed_from(&source);

    assert!(!target.is_warm());
    // Still copies the placeholder values
    assert!((target.mean() - 1.0).abs() < f64::EPSILON);
    assert!((target.variance() - 1.0).abs() < f64::EPSILON);
}

// ── update() — clipped updates ──────────────────────────────

/// A batch with nothing in it leaves both the mean and the spread exactly
/// where they were. An idle interval is therefore not a data point: the
/// baseline does not drift simply because time passed without observations.
///
/// ´claim:ewma:an-empty-batch-teaches-the-baseline-nothing´
/// ´test:crate:update-empty-is-noop´
#[test]
fn update_empty_is_noop() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[5.0, 5.0, 5.0], 3.0);
    let mean_before = stats.mean();
    let var_before = stats.variance();

    stats.update(&[], 3.0);
    assert!((stats.mean() - mean_before).abs() < f64::EPSILON);
    assert!((stats.variance() - var_before).abs() < f64::EPSILON);
}

/// A warm baseline refuses values above its own ceiling before it learns
/// anything, and a batch consisting entirely of such values moves it not at
/// all. This is the poisoning defence: an attacker cannot walk the notion of
/// normal upward by feeding extremes, because the extremes are precisely what
/// never reaches the baseline.
///
/// ´claim:ewma:a-batch-entirely-above-the-ceiling-cannot-move-the-baseline-it-would-poison´
/// ´test:crate:outliers-are-rejected´
#[test]
fn outliers_are_rejected() {
    let mut stats = EwmaStats::new(0.99);
    // Warm up with small values
    stats.update(&[1.0, 1.0, 1.0], 3.0);
    let mean_before = stats.mean();

    // Feed extreme outlier — should be rejected
    stats.update(&[1000.0], 3.0);
    assert!(
        (stats.mean() - mean_before).abs() < f64::EPSILON,
        "mean should not change when all values are outliers"
    );
}

/// The same restraint holds on the warm path as on the cold one: when only a
/// single value survives clipping, the mean moves and the spread does not.
/// The rule is about how much a batch can say about spread, not about which
/// stage of its life the baseline is in.
///
/// (´claim:ewma:a-batch-of-one-carries-no-spread-so-the-variance-is-left-untouched´)
/// ´test:crate:update-single-value-does-not-update-variance´
#[test]
fn update_single_value_does_not_update_variance() {
    let mut stats = EwmaStats::new(0.95);
    stats.update(&[5.0, 5.0, 5.0], 3.0);
    let var_before = stats.variance();

    // Single accepted value — variance path is skipped (normals.len() == 1).
    stats.update(&[5.0], 3.0);
    assert!(
        (stats.variance() - var_before).abs() < f64::EPSILON,
        "variance should not change from a single-element update"
    );
}

/// While cold there is no ceiling to clip against, so nothing is rejected: a
/// first batch far above the placeholder mean is accepted whole and becomes
/// the baseline. The placeholders are not a real notion of normal, and
/// filtering against them would let an arbitrary constant decide which of the
/// first real observations the sentinel was allowed to see.
///
/// ´claim:ewma:while-cold-there-is-no-ceiling-to-clip-against-so-nothing-is-rejected´
/// ´test:crate:update-skips-clipping-when-cold´
#[test]
fn update_skips_clipping_when_cold() {
    let mut stats = EwmaStats::new(0.99);
    // Even though 1000.0 is far from initial mean=1.0, cold-path accepts it.
    stats.update(&[1000.0, 1000.0], 3.0);
    assert!(stats.is_warm());
    assert!((stats.mean() - 1000.0).abs() < f64::EPSILON);
}

// ── update_raw() — unclipped updates ────────────────────────

/// The unclipped path treats an empty batch the same way: nothing in, nothing
/// changed. Externalising the outlier filter does not turn absence of data
/// into evidence.
///
/// (´claim:ewma:an-empty-batch-teaches-the-baseline-nothing´)
/// ´test:crate:update-raw-empty-is-noop´
#[test]
fn update_raw_empty_is_noop() {
    let mut stats = EwmaStats::new(0.99);
    stats.update_raw(&[5.0, 5.0, 5.0]);
    let mean_before = stats.mean();
    let var_before = stats.variance();

    stats.update_raw(&[]);
    assert!((stats.mean() - mean_before).abs() < f64::EPSILON);
    assert!((stats.variance() - var_before).abs() < f64::EPSILON);
}

/// The unclipped path is the clipped one with only the filter removed: given
/// a clip so wide that nothing could be rejected, the two produce the same
/// mean and the same spread. Callers that do their own outlier rejection get
/// identical arithmetic, so the two entry points cannot drift into
/// disagreeing about what a batch means.
///
/// ´claim:ewma:the-unclipped-path-is-the-clipped-one-with-only-the-filter-removed´
/// ´test:crate:update-raw-matches-update-when-no-clipping´
#[test]
fn update_raw_matches_update_when_no_clipping() {
    let mut a = EwmaStats::new(0.95);
    let mut b = EwmaStats::new(0.95);
    let values = &[1.0, 1.2, 0.8, 1.1, 0.9];

    a.update(values, 100.0); // clip_sigmas so high nothing is clipped
    b.update_raw(values);

    assert!((a.mean() - b.mean()).abs() < 1e-12);
    assert!((a.variance() - b.variance()).abs() < 1e-12);
}

// ── Decay behaviour ─────────────────────────────────────────

/// Sustained new data pulls the baseline away from what it learned before: a
/// mean established at one level and then fed a different one repeatedly ends
/// up near the new level. The baseline tracks the present rather than
/// averaging over all history, which is what makes it a moving notion of
/// normal rather than a permanent one.
///
/// ´claim:ewma:sustained-new-data-pulls-the-baseline-away-from-what-it-learned-before´
/// ´test:crate:decays-toward-new-data´
#[test]
fn decays_toward_new_data() {
    let mut stats = EwmaStats::new(0.90); // fast decay
    stats.update(&[10.0, 10.0, 10.0], 3.0);
    assert!((stats.mean() - 10.0).abs() < f64::EPSILON);

    // Push toward 0.0
    for _ in 0..50 {
        stats.update(&[0.0, 0.0, 0.0], 3.0);
    }
    assert!(stats.mean() < 1.0, "mean should have decayed toward 0.0");
}

/// The decay factor is the length of the baseline's memory. Two baselines
/// started at the same level and fed the same contradicting data diverge in
/// the expected direction: the one with the higher factor still holds more of
/// the original level. This is what lets a fast and a slow baseline over the
/// same stream disagree usefully, which is the whole basis of drift
/// detection.
///
/// ´claim:ewma:a-higher-decay-factor-holds-the-past-longer´
/// ´test:crate:higher-decay-forgets-slower´
#[test]
fn higher_decay_forgets_slower() {
    let mut slow = EwmaStats::new(0.99); // slow decay (long memory)
    let mut fast = EwmaStats::new(0.90); // fast decay (short memory)

    // Both start at 10.0
    slow.update(&[10.0, 10.0, 10.0], 5.0);
    fast.update(&[10.0, 10.0, 10.0], 5.0);

    // Push both toward 0.0
    for _ in 0..20 {
        slow.update(&[0.0, 0.0, 0.0], 5.0);
        fast.update(&[0.0, 0.0, 0.0], 5.0);
    }

    // Slow retains more of the original 10.0
    assert!(
        slow.mean() > fast.mean(),
        "slow (λ=0.99) should retain more: slow={}, fast={}",
        slow.mean(),
        fast.mean()
    );
}

// ── z_score() ───────────────────────────────────────────────

/// A value sitting at the baseline scores essentially nothing. The z-score
/// measures signed departure from what the baseline expects, so agreement
/// with it is the origin of the scale rather than a point somewhere along it.
///
/// ´claim:ewma:the-z-score-is-signed-departure-from-the-baseline-and-zero-at-it´
/// ´test:crate:z-score-is-zero-at-mean´
#[test]
fn z_score_is_zero_at_mean() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[5.0, 5.0, 5.0, 5.0], 3.0);
    let z = stats.z_score(5.0, 1e-6);
    assert!(z.abs() < 0.01);
}

/// A value above the baseline scores positive, which is the direction that
/// matters: elevated anomaly scores are the ones the sentinel is watching for
/// and the ones an attacker would produce.
///
/// (´claim:ewma:the-z-score-is-signed-departure-from-the-baseline-and-zero-at-it´)
/// ´test:crate:z-score-positive-above-mean´
#[test]
fn z_score_positive_above_mean() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[5.0, 5.0, 5.0], 3.0);
    let z = stats.z_score(10.0, 1e-6);
    assert!(z > 0.0, "z-score should be positive above mean, got {z}");
}

/// A value below the baseline scores negative rather than being folded to a
/// magnitude. The sign survives, so a caller can tell a quiet departure from
/// an elevated one instead of seeing both as equally unusual.
///
/// (´claim:ewma:the-z-score-is-signed-departure-from-the-baseline-and-zero-at-it´)
/// ´test:crate:z-score-negative-below-mean´
#[test]
fn z_score_negative_below_mean() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[5.0, 5.0, 5.0], 3.0);
    let z = stats.z_score(1.0, 1e-6);
    assert!(z < 0.0, "z-score should be negative below mean, got {z}");
}

// ── ceiling() ───────────────────────────────────────────────

/// Asked for its ceiling while cold, a baseline answers with infinity — the
/// value that admits everything. The bypass on the cold update path is not a
/// special case hidden inside the update; it is visible in the ceiling
/// itself, so the two cannot disagree about whether clipping applies.
///
/// (´claim:ewma:while-cold-there-is-no-ceiling-to-clip-against-so-nothing-is-rejected´)
/// ´test:crate:ceiling-returns-infinity-when-cold´
#[test]
fn ceiling_returns_infinity_when_cold() {
    let stats = EwmaStats::new(0.95);
    assert!(stats.ceiling(3.0).is_infinite());
}

/// Once warm, the ceiling stands a fixed number of standard deviations above
/// the mean — the requested multiple of the baseline's own square-rooted
/// spread, added to its own centre. The threshold is therefore relative to
/// what this cell has learned, not an absolute score chosen in advance for
/// every cell alike.
///
/// ´claim:ewma:a-warm-ceiling-stands-a-fixed-number-of-deviations-above-the-mean´
/// ´test:crate:ceiling-returns-mean-plus-sigmas-when-warm´
#[test]
fn ceiling_returns_mean_plus_sigmas_when_warm() {
    let mut stats = EwmaStats::new(0.95);
    stats.update(&[2.0, 2.0, 2.0], 3.0);
    let ceil = stats.ceiling(3.0);
    let expected = 3.0_f64.mul_add(stats.variance().sqrt(), stats.mean());
    assert!((ceil - expected).abs() < 1e-12);
}

/// The clip setting is a monotone dial on how much the baseline is willing to
/// learn from: given the same elevated value, a baseline clipping at a wide
/// multiple moves at least as far as one clipping tightly. Tightening the
/// setting can only ever admit less, so an operator turning it down is
/// trading responsiveness for poisoning resistance and never the reverse.
///
/// ´claim:ewma:widening-the-clip-admits-at-least-as-much-as-tightening-it´
/// ´test:crate:clip-sigmas-affects-ceiling´
#[test]
fn clip_sigmas_affects_ceiling() {
    // With tight clip (1σ), more values are rejected.
    let mut tight = EwmaStats::new(0.99);
    tight.update(&[1.0, 1.0, 1.0], 1.0);

    // With wide clip (5σ), fewer values are rejected.
    let mut wide = EwmaStats::new(0.99);
    wide.update(&[1.0, 1.0, 1.0], 5.0);

    // Feed a moderately elevated value.
    let elevated = &[3.0];
    let mean_before_tight = tight.mean();
    let mean_before_wide = wide.mean();
    tight.update(elevated, 1.0);
    wide.update(elevated, 5.0);

    // Wide should have moved toward 3.0 more than tight
    // (tight may reject 3.0 if it's beyond 1σ from mean ~1.0).
    let tight_delta = (tight.mean() - mean_before_tight).abs();
    let wide_delta = (wide.mean() - mean_before_wide).abs();
    assert!(
        wide_delta >= tight_delta,
        "wide clip should accept more: wide_delta={wide_delta}, tight_delta={tight_delta}"
    );
}

// ── snapshot() ──────────────────────────────────────────────

/// The snapshot a report carries holds the same mean and spread the
/// baseline's own accessors report. What a caller reads out of a report is
/// the state the engine is scoring against, not a rounded or separately
/// derived summary of it.
///
/// ´claim:ewma:a-snapshot-reports-the-baseline-the-accessors-report´
/// ´test:crate:snapshot-matches-state´
#[test]
fn snapshot_matches_state() {
    let mut stats = EwmaStats::new(0.99);
    stats.update(&[2.0, 4.0, 6.0], 3.0);
    let snap = stats.snapshot();
    assert!((snap.mean - stats.mean()).abs() < f64::EPSILON);
    assert!((snap.variance - stats.variance()).abs() < f64::EPSILON);
}

// ── Variance floor ──────────────────────────────────────────

/// A batch of identical values has no deviation at all, yet the spread does
/// not reach zero: a floor holds it above. Without it a perfectly quiet
/// period would collapse the spread, the ceiling would close onto the mean,
/// and every subsequent value — however ordinary — would be rejected as an
/// outlier, leaving the baseline permanently frozen at the quiet level.
///
/// ´claim:ewma:the-spread-is-floored-so-a-quiet-period-cannot-freeze-the-baseline´
/// ´test:crate:variance-floor-is-respected´
#[test]
fn variance_floor_is_respected() {
    let mut stats = EwmaStats::new(0.95);
    // All identical values → zero deviation, but floor should apply.
    stats.update(&[7.0, 7.0, 7.0, 7.0], 3.0);
    assert!(
        stats.variance() >= 1e-4,
        "variance should be clamped to floor 1e-4, got {}",
        stats.variance()
    );
}
