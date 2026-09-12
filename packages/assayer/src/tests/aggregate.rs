// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_sentinels_returns_zeros`] | feature | A round in which no Sentinel reported aggregates to fifteen finite zeros — coverage included, even though Sentinels were registered and could have reported. Nothing was heard, so nothing is asserted; a fold over an empty population must not leak the sentinel value it started from into a block the model reads as fleet-wide calm. |
//! | [`single_sentinel_stats`] | feature | With one Sentinel reporting, the distribution slots collapse onto it: the maximum and the mean both equal its own loudest axis and the spread is exactly zero, while the per-axis maxima are simply that Sentinel's own scores. A population of one has no disagreement to describe, so the spread reports none rather than an artefact of dividing by a degenerate count. |
//! | [`three_sentinels_golden`] | feature | The block reduces across Sentinels in two different orders, and the difference matters. The leading slots take each Sentinel's loudest axis first and then summarise those figures across the fleet; the per-axis slots hold each axis fixed and maximise over Sentinels, so a Sentinel that is loudest overall need not own every per-axis maximum. Checked against a hand-computed set of three, both reductions land where they should. |
//! | [`aggregate_concordance_threshold`] | feature | Concordance is computed one axis at a time, each axis counting the Sentinels whose score on that axis clears that axis's own threshold, so the four fractions differ from one another over the same population. Agreement on novelty is a different fact from agreement on coherence, and collapsing them would hide whether a fleet-wide disturbance is broad or confined to a single kind of anomaly. |
//! | [`coverage_fraction`] | feature | Coverage is the share of registered Sentinels that reported, counted from the population sizes alone — three of five reporting gives the same figure whatever those three had to say, here nothing at all. Every other slot in the block is conditioned on who was heard from, so the model needs to know how much of the fleet that was before it can weigh them. |
//! | [`axis_breadth_counts`] | feature | Breadth counts axes, not Sentinels: it is the number of axes whose fleet-wide maximum clears that axis's threshold, four when every axis is elevated and two when only two are. An anomaly showing on every axis at once is a different kind of event from one confined to a single axis, and breadth is the slot that tells them apart at a glance. |
//! | [`cross_axis_product`] | feature | One slot multiplies the fleet-wide novelty maximum by the displacement maximum rather than adding them. A product stays small unless both are elevated, so the slot fires only on the conjunction — novelty and displacement together — which a linear model reading the two maxima separately could not express. |
//! | [`zero_registered_coverage`] | feature | cites (´claim:feature:coverage-is-the-share-of-registered-sentinels-that-reported-regardless-of-their-scores´) |

//! Crate-level tests for aggregate feature computation (´sec:feature:aggregate´).
//!
//! The aggregate block is the part of the feature vector that describes the
//! fleet rather than any one Sentinel: how loud the loudest is, how the loudness
//! is distributed, how many Sentinels and how many axes agree, and how much of
//! the registered population was heard from at all. These tests fix what each
//! of the fifteen slots means and how the block behaves when the population is
//! degenerate — nobody reporting, one Sentinel reporting, nobody registered.

use crate::feature::aggregate::{SentinelSubScores, compute_aggregates};
use crate::testing::{DEFAULT_TOLERANCES, assert_finite, assert_near};
use crate::types::{SCORING_AXIS_COUNT, SentinelId};

fn default_thresholds() -> [f64; SCORING_AXIS_COUNT] {
    [0.5, 0.5, 0.5, 0.5]
}

/// A round in which no Sentinel reported aggregates to fifteen finite zeros —
/// coverage included, even though Sentinels were registered and could have
/// reported. Nothing was heard, so nothing is asserted; a fold over an empty
/// population must not leak the sentinel value it started from into a block the
/// model reads as fleet-wide calm.
///
/// ´claim:feature:a-round-with-no-reporting-sentinels-aggregates-to-fifteen-finite-zeros´
/// ´test:crate:empty-sentinels-returns-zeros´
#[test]
fn empty_sentinels_returns_zeros() {
    let tol = DEFAULT_TOLERANCES;
    let features = compute_aggregates(&[], &default_thresholds(), 5);
    assert_finite(&features, "empty_sentinels features");
    for (i, &v) in features.iter().enumerate() {
        assert_near(v, 0.0, tol.bit_identical, &format!("features[{i}]"));
    }
}

/// With one Sentinel reporting, the distribution slots collapse onto it: the
/// maximum and the mean both equal its own loudest axis and the spread is
/// exactly zero, while the per-axis maxima are simply that Sentinel's own
/// scores. A population of one has no disagreement to describe, so the spread
/// reports none rather than an artefact of dividing by a degenerate count.
///
/// ´claim:feature:a-single-reporting-sentinel-makes-the-maximum-and-mean-coincide-and-the-spread-vanish´
/// ´test:crate:single-sentinel-stats´
#[test]
fn single_sentinel_stats() {
    let tol = DEFAULT_TOLERANCES;
    let scores = SentinelSubScores::from_values([1.0, 2.0, 0.5, 0.3], [0.1, 0.2, 0.3, 0.4]);
    let input = vec![(SentinelId(1), scores)];
    let features = compute_aggregates(&input, &default_thresholds(), 1);
    assert_finite(&features, "single_sentinel features");

    // Max cell-level max z = 2.0 (displacement)
    assert_near(features[0], 2.0, tol.bit_identical, "max max_z");
    // Mean = 2.0 (single value)
    assert_near(features[1], 2.0, tol.bit_identical, "mean max_z");
    // Std = 0.0 (single value)
    assert_near(features[2], 0.0, tol.bit_identical, "std max_z");
    // Per-axis max z
    assert_near(features[3], 1.0, tol.bit_identical, "per-axis max N");
    assert_near(features[4], 2.0, tol.bit_identical, "per-axis max D");
    assert_near(features[5], 0.5, tol.bit_identical, "per-axis max S");
    assert_near(features[6], 0.3, tol.bit_identical, "per-axis max C");
    // Cross-axis product
    assert_near(features[11], 2.0, tol.bit_identical, "cross-axis product"); // 1.0 × 2.0
    // Coverage
    assert_near(features[13], 1.0, tol.bit_identical, "coverage"); // 1/1
    // Max CUSUM = 0.4
    assert_near(features[14], 0.4, tol.bit_identical, "max CUSUM");
}

/// The block reduces across Sentinels in two different orders, and the
/// difference matters. The leading slots take each Sentinel's loudest axis
/// first and then summarise those figures across the fleet; the per-axis slots
/// hold each axis fixed and maximise over Sentinels, so a Sentinel that is
/// loudest overall need not own every per-axis maximum. Checked against a
/// hand-computed set of three, both reductions land where they should.
///
/// ´claim:feature:the-block-reduces-both-loudest-axis-first-and-per-axis-across-sentinels´
/// ´test:crate:three-sentinels-golden´
#[test]
fn three_sentinels_golden() {
    // Hand-computed golden values
    let s1 = SentinelSubScores::from_values([0.3, 0.4, 0.2, 0.1], [0.1, 0.1, 0.1, 0.1]);
    let s2 = SentinelSubScores::from_values([0.8, 0.6, 0.7, 0.3], [0.2, 0.3, 0.2, 0.1]);
    let s3 = SentinelSubScores::from_values([0.5, 1.2, 0.4, 0.6], [0.1, 0.4, 0.1, 0.2]);

    let input = vec![(SentinelId(1), s1), (SentinelId(2), s2), (SentinelId(3), s3)];
    let features = compute_aggregates(&input, &default_thresholds(), 5);
    let tol = DEFAULT_TOLERANCES;
    assert_finite(&features, "three_sentinels features");

    // Max of max_z per Sentinel: max(0.4, 0.8, 1.2) = 1.2
    assert_near(features[0], 1.2, tol.bit_identical, "max max_z");

    // Mean of max_z per Sentinel: (0.4 + 0.8 + 1.2) / 3 = 0.8
    assert_near(features[1], 0.8, tol.default, "mean max_z");

    // Per-axis max z
    assert_near(features[3], 0.8, tol.bit_identical, "per-axis max N"); // max(0.3, 0.8, 0.5)
    assert_near(features[4], 1.2, tol.bit_identical, "per-axis max D"); // max(0.4, 0.6, 1.2)
    assert_near(features[5], 0.7, tol.bit_identical, "per-axis max S"); // max(0.2, 0.7, 0.4)
    assert_near(features[6], 0.6, tol.bit_identical, "per-axis max C"); // max(0.1, 0.3, 0.6)

    // Coverage: 3/5 = 0.6
    assert_near(features[13], 0.6, tol.bit_identical, "coverage");

    // Max CUSUM: max(0.1, 0.3, 0.4) = 0.4
    assert_near(features[14], 0.4, tol.bit_identical, "max CUSUM");
}

/// Concordance is computed one axis at a time, each axis counting the Sentinels
/// whose score on that axis clears that axis's own threshold, so the four
/// fractions differ from one another over the same population. Agreement on
/// novelty is a different fact from agreement on coherence, and collapsing them
/// would hide whether a fleet-wide disturbance is broad or confined to a single
/// kind of anomaly.
///
/// ´claim:feature:each-axis-carries-its-own-concordance-fraction-counted-against-its-own-threshold´
/// ´test:crate:aggregate-concordance-threshold´
#[test]
fn aggregate_concordance_threshold() {
    let s1 = SentinelSubScores::from_values([0.3, 0.6, 0.9, 0.2], [0.0; 4]);
    let s2 = SentinelSubScores::from_values([0.7, 0.4, 0.8, 0.1], [0.0; 4]);
    let s3 = SentinelSubScores::from_values([0.6, 0.3, 0.4, 0.9], [0.0; 4]);

    let input = vec![(SentinelId(1), s1), (SentinelId(2), s2), (SentinelId(3), s3)];
    let features = compute_aggregates(&input, &default_thresholds(), 3);
    let tol = DEFAULT_TOLERANCES;
    assert_finite(&features, "concordance features");

    // Concordance for N (threshold 0.5): 2/3 above (0.7, 0.6)
    assert_near(features[7], 2.0 / 3.0, tol.default, "concordance N");
    // Concordance for D (threshold 0.5): 1/3 above (0.6)
    assert_near(features[8], 1.0 / 3.0, tol.default, "concordance D");
    // Concordance for S (threshold 0.5): 2/3 above (0.9, 0.8)
    assert_near(features[9], 2.0 / 3.0, tol.default, "concordance S");
    // Concordance for C (threshold 0.5): 1/3 above (0.9)
    assert_near(features[10], 1.0 / 3.0, tol.default, "concordance C");
}

/// Coverage is the share of registered Sentinels that reported, counted from
/// the population sizes alone — three of five reporting gives the same figure
/// whatever those three had to say, here nothing at all. Every other slot in
/// the block is conditioned on who was heard from, so the model needs to know
/// how much of the fleet that was before it can weigh them.
///
/// ´claim:feature:coverage-is-the-share-of-registered-sentinels-that-reported-regardless-of-their-scores´
/// ´test:crate:coverage-fraction´
#[test]
fn coverage_fraction() {
    let tol = DEFAULT_TOLERANCES;
    let s1 = SentinelSubScores::new();
    let s2 = SentinelSubScores::new();
    let s3 = SentinelSubScores::new();

    let input = vec![(SentinelId(1), s1), (SentinelId(2), s2), (SentinelId(3), s3)];
    let features = compute_aggregates(&input, &default_thresholds(), 5);
    assert_finite(&features, "coverage features");

    // Coverage: 3/5 = 0.6
    assert_near(features[13], 0.6, tol.bit_identical, "coverage");
}

/// Breadth counts axes, not Sentinels: it is the number of axes whose
/// fleet-wide maximum clears that axis's threshold, four when every axis is
/// elevated and two when only two are. An anomaly showing on every axis at
/// once is a different kind of event from one confined to a single axis, and
/// breadth is the slot that tells them apart at a glance.
///
/// ´claim:feature:axis-breadth-counts-how-many-axes-have-a-fleet-maximum-above-threshold´
/// ´test:crate:axis-breadth-counts´
#[test]
fn axis_breadth_counts() {
    let tol = DEFAULT_TOLERANCES;

    // All axes at 0.6, threshold 0.5 → breadth = 4
    let s1 = SentinelSubScores::from_values([0.6, 0.6, 0.6, 0.6], [0.0; 4]);
    let input = vec![(SentinelId(1), s1)];
    let features = compute_aggregates(&input, &default_thresholds(), 1);
    assert_finite(&features, "breadth all-above features");
    assert_near(features[12], 4.0, tol.bit_identical, "breadth all-above");

    // Only N and S above threshold → breadth = 2
    let s2 = SentinelSubScores::from_values([0.6, 0.3, 0.7, 0.2], [0.0; 4]);
    let input2 = vec![(SentinelId(1), s2)];
    let features2 = compute_aggregates(&input2, &default_thresholds(), 1);
    assert_finite(&features2, "breadth partial features");
    assert_near(features2[12], 2.0, tol.bit_identical, "breadth partial");
}

/// One slot multiplies the fleet-wide novelty maximum by the displacement
/// maximum rather than adding them. A product stays small unless both are
/// elevated, so the slot fires only on the conjunction — novelty and
/// displacement together — which a linear model reading the two maxima
/// separately could not express.
///
/// ´claim:feature:the-cross-axis-slot-multiplies-the-novelty-and-displacement-maxima-so-it-fires-only-on-both´
/// ´test:crate:cross-axis-product´
#[test]
fn cross_axis_product() {
    let tol = DEFAULT_TOLERANCES;
    let s1 = SentinelSubScores::from_values([2.0, 3.0, 1.0, 0.5], [0.0; 4]);
    let input = vec![(SentinelId(1), s1)];
    let features = compute_aggregates(&input, &default_thresholds(), 1);
    assert_finite(&features, "cross_axis features");

    // Cross-axis product: N × D = 2.0 × 3.0 = 6.0
    assert_near(features[11], 6.0, tol.bit_identical, "cross-axis product");
}

/// When no Sentinels are registered at all, coverage reads zero rather than
/// dividing by an empty population — even though a Sentinel did report, which
/// is itself a contradictory state. The undefined ratio is answered with a
/// finite value, so an inconsistency in the registry cannot propagate a
/// non-finite number into the fleet-level block.
///
/// (´claim:feature:coverage-is-the-share-of-registered-sentinels-that-reported-regardless-of-their-scores´)
/// ´test:crate:zero-registered-coverage´
#[test]
fn zero_registered_coverage() {
    let tol = DEFAULT_TOLERANCES;
    let s1 = SentinelSubScores::new();
    let input = vec![(SentinelId(1), s1)];
    let features = compute_aggregates(&input, &default_thresholds(), 0);
    assert_finite(&features, "zero_registered features");

    // Coverage undefined (0/0), set to 0.0
    assert_near(features[13], 0.0, tol.bit_identical, "coverage (0 registered)");
}
