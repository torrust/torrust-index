// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`feed_forward_delta_one_normal`] | invariant | Each observation contributes exactly one unit to the structure's running total, checked after every batch of a long run: the total is always the number of values fed so far and never drifts from it. Importance is therefore a count of traffic rather than a derived score, which is what lets the weight a range carries be compared against another range's honestly. |
//! | [`feed_forward_delta_one_anomalous`] | invariant | cites (´claim:invariant:the-running-total-counts-exactly-one-unit-per-observation´) |
//! | [`feed_forward_delta_one_after_decay`] | invariant | cites (´claim:invariant:the-running-total-counts-exactly-one-unit-per-observation´) |
//! | [`analysis_width_equals_128_minus_depth`] | invariant | Every cell and every ancestor in a report analyses exactly the domain less the levels its position has already fixed. The relation is arithmetic rather than incidental: the bits routing resolved are constant for everything arriving in that cell and carry no information, so what remains is precisely what a tracker there can learn from, and its declared analysis is that and nothing else. |
//! | [`energy_ratio_bounded_zero_to_one`] | invariant | The fraction of structure a tracker has managed to capture is reported as a genuine fraction: it never falls below nothing and never exceeds everything, in any cell or ancestor of a report. Because it is bounded on both sides, a caller can read it directly as how much of what arrives the model explains, and can compare one cell's figure against another's. |
//! | [`rank_bounded_by_max_rank`] | invariant | No tracker in a report has grown past the ceiling its configuration set, at any level of the tree. The ceiling is what makes a tracker's cost knowable in advance, so it has to be a hard limit on what the adaptation may reach for rather than a target it aims at — traffic complicated enough to justify more structure still does not get more. |
//! | [`no_nan_scores_after_warmup`] | invariant | After seeding and a run of warming batches, no axis of any reported cell hands back a number that is not a number. This matters more than tidiness: such a value compares false against every threshold, so a single one would silently disarm the alerting that reads it, and the arithmetic guards its absence rather than callers being expected to check. |
//! | [`noise_injected_before_real_observations`] | invariant | After a run that creates cells below the root, every cell being tracked carries synthetic observations — including those that came into being while traffic was already flowing. The ordering within the step is fixed rather than raced: preparation happens before a tracker is shown real data, so no cell is ever in the position of judging its first batch against nothing. |
//! | [`graph_updated_before_analysis_set`] | invariant | The summary and the detail of a batch report describe the same moment: when the summary says cells are competing, the report already carries their entries. The structure is brought up to date before attention is reapportioned within the same batch, so a split does not leave a batch whose summary counts a cell that the report cannot show. |
//! | [`root_always_in_analysis_set`] | invariant | Whatever range a batch is aimed at, the reported set of analysed cells still reaches back to the root — its shallowest member is the root in every batch of a run that cycles through all the leading ranges. The root's place is unconditional, so every chain of ancestors terminates and there is always a model covering traffic that belongs to no more specific cell. |
//! | [`root_survives_extreme_decay`] | invariant | Ageing severe enough to annihilate the accumulated standing of everything else still leaves the root tracker in place, and traffic arriving afterwards is reported against it again. The root is permanent by construction rather than by having earned its standing, because a sentinel that could decay away its last model would have nothing to route to and no way to begin again. |
//! | [`score_polarity_higher_is_more_anomalous`] | invariant | Two sentinels given identical settling traffic diverge in the expected direction once one of them is fed structurally novel batches: its peak drift reading ends up above the other's. Scores point one way — larger means more anomalous — so a caller may threshold and compare them without having to know which axis produced the number or which direction it runs in. |
//! | [`cusum_accumulators_non_negative`] | invariant | Through a run of ordinary traffic, no drift accumulator on any axis of any reported cell ever goes below nothing. The accumulator is clamped at rest deliberately: a stretch of quieter-than-usual traffic must not bank negative standing that a later attack could spend, so a rise always starts from zero and means what it says. |
//! | [`lifetime_observations_monotonically_increases`] | invariant | The lifetime observation count rises by exactly the size of each batch, for batches ranging from a single value to many. It is a plain census of what was handed in — never sampled, never rounded and never adjusted for what the values looked like — which is what makes it usable as the denominator when judging any rate the sentinel reports. |
//! | [`cells_tracked_always_at_least_one`] | invariant | At no point in a sentinel's life is it tracking nothing: not at birth before any traffic, not through a run of ingestion, and not after ageing severe enough to strip away everything that had accumulated. There is always at least the root, so the question "what does the sentinel make of this value" always has an answer. |
//! | [`assert_invariants_under_random_traffic`] | invariant | Traffic with no structure at all — values scattered across the whole domain, in batches whose size changes from one to the next — leaves every structural property standing, checked after each batch. The guarantees are not conditioned on the traffic being well behaved or on batches being uniform, which is the whole point of calling them guarantees. |
//! | [`assert_invariants_after_decay_regrowth`] | invariant | A sentinel whose tree has been collapsed by severe ageing and then made to regrow under traffic spread across the ranges satisfies every structural property throughout the regrowth, batch by batch. The transient state of a system rebuilding itself is exactly where a bound is likeliest to slip, so the guarantees are asserted while it is in motion rather than once it has settled. |
//! | [`a_single_arrival_outlives_the_projection_only_in_the_accumulator`] | invariant | The feed-forward count is checked in the accumulator's own domain rather than through a floating-point projection, because past a certain magnitude the projection cannot express a single arrival. The projection is lossy by its own documentation, and at the first magnitude where consecutive integers stop being separately representable, a total and that same total plus one arrival land on the same number while a total plus two lands two away. A check that projects both sides and allows them to differ by less than one arrival therefore rejects arithmetic that is exactly right. The accumulator keeps the distinction the projection loses, so the comparison belongs there; this test pins the property the choice rests on rather than the failure itself, which is some nine quadrillion observations away and not reachable by a test. |

//! Integration tests for the properties the sentinel is required to hold at
//! **all times, whatever the traffic** — the statements a reader of any report
//! is entitled to assume without checking.
//!
//! They fall into a few families. Accounting: each observation adds exactly one
//! to the structure's running total, whatever the batch contained, so importance
//! measures weight of traffic and can never be inflated by the content of a
//! request. Geometry: a cell's analysis is exactly as wide as the domain less
//! the levels routing has already resolved, its captured-energy fraction lies in
//! the unit interval, its rank never exceeds the configured ceiling, and no
//! score is ever reported as not-a-number. Ordering within a batch: a tracker is
//! warmed before it is asked about real traffic, and a cell the summary counts
//! as competing is already reported in the same batch that created it.
//!
//! Two of the families exist because of what a caller does with a report. The
//! root is present in every analysis set, so every chain of ancestors
//! terminates and there is always somewhere to attribute traffic that belongs
//! nowhere else; and it survives decay severe enough to collapse everything
//! else, because a sentinel with no root would have nowhere to begin again.
//! Score polarity runs one way — higher means more anomalous, on every axis —
//! so a caller may compare and threshold without asking which direction this
//! particular number points in, and the drift accumulators never go negative,
//! so a quiet period cannot bank credit against a future attack.
//!
//! The last tests are cross-cutting: they assert the whole suite batch by batch
//! under unstructured traffic of varying size, and across a collapse and
//! regrowth of the tree, on the principle that an invariant is only worth the
//! name if it holds while the system is in motion.

mod common;

use common::{ScenarioBuilder, anomalous_values, assert_invariants, cell_values, integration_config, max_cusum, test_config};
use torrust_mudlark::Inspectable;
use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};

// ── G1: Feed-forward invariant ──────────────────────────────

/// Each observation contributes exactly one unit to the structure's running
/// total, checked after every batch of a long run: the total is always the
/// number of values fed so far and never drifts from it. Importance is
/// therefore a count of traffic rather than a derived score, which is what lets
/// the weight a range carries be compared against another range's honestly.
///
/// ´claim:invariant:the-running-total-counts-exactly-one-unit-per-observation´
/// ´test:integration:feed-forward-delta-one-normal´
#[test]
fn feed_forward_delta_one_normal() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    for batch_num in 1..=50u64 {
        let values = cell_values(0xA, 8);
        s.ingest(&values);
        assert_eq!(
            s.graph().total_sum(),
            batch_num * 8,
            "total_sum must equal cumulative observation count"
        );
    }
}

/// The accounting is blind to content: a batch of structurally novel values
/// arriving after a settled run raises the total by its own size and no more.
/// This is the security-relevant half of the rule — how anomalous a request
/// looks buys it no extra standing in the structure, and no attacker can win
/// attention for a region by making its traffic look strange.
///
/// (´claim:invariant:the-running-total-counts-exactly-one-unit-per-observation´)
/// ´test:integration:feed-forward-delta-one-anomalous´
#[test]
fn feed_forward_delta_one_anomalous() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // Normal traffic.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 8));
    }
    let before = s.graph().total_sum();

    // Anomalous traffic — graph total_sum should still increase by exactly n.
    let anomalous = anomalous_values(0xA, 8);
    s.ingest(&anomalous);
    assert_eq!(
        s.graph().total_sum(),
        before + 8,
        "anomalous traffic should not change delta-one accounting"
    );
}

/// Ageing the structure down rescales what has accumulated, but it does not
/// disturb the increment: a batch arriving afterwards adds exactly its own size
/// on top of the reduced total. The two mechanisms compose cleanly, so a
/// long-lived sentinel can forget old traffic without its counting of new
/// traffic going wrong.
///
/// (´claim:invariant:the-running-total-counts-exactly-one-unit-per-observation´)
/// ´test:integration:feed-forward-delta-one-after-decay´
#[test]
fn feed_forward_delta_one_after_decay() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    s.ingest(&cell_values(0xA, 100));
    s.decay(0.5, 0.0);
    let after_decay = s.graph().total_sum();

    s.ingest(&cell_values(0xA, 10));
    assert_eq!(
        s.graph().total_sum(),
        after_decay + 10,
        "after decay + ingest, total_sum reflects only the new ingest count"
    );
}

// ── G2: Constant-norm geometry ──────────────────────────────

/// Every cell and every ancestor in a report analyses exactly the domain less
/// the levels its position has already fixed. The relation is arithmetic rather
/// than incidental: the bits routing resolved are constant for everything
/// arriving in that cell and carry no information, so what remains is precisely
/// what a tracker there can learn from, and its declared analysis is that and
/// nothing else.
///
/// ´claim:invariant:a-cells-analysis-covers-the-domain-less-the-levels-routing-already-fixed´
/// ´test:integration:analysis-width-equals-128-minus-depth´
#[test]
fn analysis_width_equals_128_minus_depth() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            max_rank: 2,
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    let report = s.ingest(&cell_values(0xA, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.analysis_width,
            128 - cr.depth as usize,
            "analysis_width mismatch at depth {}",
            cr.depth
        );
    }
}

/// The fraction of structure a tracker has managed to capture is reported as a
/// genuine fraction: it never falls below nothing and never exceeds everything,
/// in any cell or ancestor of a report. Because it is bounded on both sides, a
/// caller can read it directly as how much of what arrives the model explains,
/// and can compare one cell's figure against another's.
///
/// ´claim:invariant:the-captured-structure-fraction-stays-inside-the-unit-interval´
/// ´test:integration:energy-ratio-bounded-zero-to-one´
#[test]
fn energy_ratio_bounded_zero_to_one() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            max_rank: 2,
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    let report = s.ingest(&cell_values(0xA, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(
            (0.0..=1.0).contains(&cr.energy_ratio),
            "energy_ratio {} out of [0.0, 1.0] at depth {}",
            cr.energy_ratio,
            cr.depth
        );
    }
}

/// No tracker in a report has grown past the ceiling its configuration set, at
/// any level of the tree. The ceiling is what makes a tracker's cost knowable in
/// advance, so it has to be a hard limit on what the adaptation may reach for
/// rather than a target it aims at — traffic complicated enough to justify more
/// structure still does not get more.
///
/// ´claim:invariant:no-tracker-grows-past-the-ceiling-its-configuration-set´
/// ´test:integration:rank-bounded-by-max-rank´
#[test]
fn rank_bounded_by_max_rank() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 2,
        split_threshold: 10,
        ..integration_config()
    };
    let max_rank = cfg.max_rank;

    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    let report = s.ingest(&cell_values(0xA, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(
            cr.rank <= max_rank,
            "rank {} exceeds max_rank {} at depth {}",
            cr.rank,
            max_rank,
            cr.depth
        );
    }
}

/// After seeding and a run of warming batches, no axis of any reported cell
/// hands back a number that is not a number. This matters more than tidiness:
/// such a value compares false against every threshold, so a single one would
/// silently disarm the alerting that reads it, and the arithmetic guards its
/// absence rather than callers being expected to check.
///
/// ´claim:invariant:no-scoring-axis-ever-hands-back-a-value-that-is-not-a-number´
/// ´test:integration:no-nan-scores-after-warmup´
#[test]
fn no_nan_scores_after_warmup() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            max_rank: 2,
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    let report = s.ingest(&cell_values(0xA, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement at depth {}",
            cr.depth
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise at depth {}", cr.depth);
    }
}

// ── G3: Step ordering ──────────────────────────────────────

/// After a run that creates cells below the root, every cell being tracked
/// carries synthetic observations — including those that came into being while
/// traffic was already flowing. The ordering within the step is fixed rather
/// than raced: preparation happens before a tracker is shown real data, so no
/// cell is ever in the position of judging its first batch against nothing.
///
/// ´claim:invariant:a-tracker-is-prepared-before-it-is-ever-shown-real-traffic´
/// ´test:integration:noise-injected-before-real-observations´
#[test]
fn noise_injected_before_real_observations() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed enough traffic to create at least one competitive cell.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 20));
    }

    // Every tracker should have noise_observations > 0 (noise ran first).
    for &gnode in &s.cell_gnodes() {
        let insp = s.inspect_cell(gnode).unwrap();
        assert!(
            insp.maturity.noise_observations > 0,
            "cell at depth {} should have noise observations",
            insp.depth
        );
    }
}

/// The summary and the detail of a batch report describe the same moment: when
/// the summary says cells are competing, the report already carries their
/// entries. The structure is brought up to date before attention is
/// reapportioned within the same batch, so a split does not leave a batch whose
/// summary counts a cell that the report cannot show.
///
/// ´claim:invariant:a-cell-the-summary-counts-as-competing-is-already-detailed-in-the-same-report´
/// ´test:integration:graph-updated-before-analysis-set´
#[test]
fn graph_updated_before_analysis_set() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed concentrated traffic to force a split.
    for _ in 0..5 {
        s.ingest(&cell_values(0xA, 20));
    }

    // After enough traffic to split, the new cell should appear in
    // the analysis set in the same batch's report.
    let report = s.ingest(&cell_values(0xA, 20));
    let summary = &report.analysis_set_summary;

    // If competitive cells exist, they must already be represented.
    if summary.competitive_size > 0 {
        assert!(
            !report.cell_reports.is_empty(),
            "competitive cells in summary but no cell_reports"
        );
    }
}

// ── Root invariants ─────────────────────────────────────────

/// Whatever range a batch is aimed at, the reported set of analysed cells still
/// reaches back to the root — its shallowest member is the root in every batch
/// of a run that cycles through all the leading ranges. The root's place is
/// unconditional, so every chain of ancestors terminates and there is always a
/// model covering traffic that belongs to no more specific cell.
///
/// ´claim:invariant:the-root-is-in-every-analysed-set-so-every-ancestor-chain-terminates´
/// ´test:integration:root-always-in-analysis-set´
#[test]
fn root_always_in_analysis_set() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    for i in 0..30u128 {
        let report = s.ingest(&cell_values(i % 16, 8));
        assert_eq!(
            report.analysis_set_summary.depth_range.0, 0,
            "root (depth 0) must always be in the analysis set"
        );
    }
}

/// Ageing severe enough to annihilate the accumulated standing of everything
/// else still leaves the root tracker in place, and traffic arriving afterwards
/// is reported against it again. The root is permanent by construction rather
/// than by having earned its standing, because a sentinel that could decay away
/// its last model would have nothing to route to and no way to begin again.
///
/// ´claim:invariant:the-root-tracker-outlives-any-amount-of-ageing´
/// ´test:integration:root-survives-extreme-decay´
#[test]
fn root_survives_extreme_decay() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(20).build();

    assert!(s.cells_tracked() > 1);

    // Annihilate everything.
    s.decay(0.0001, 0.0);

    // Root must still be present.
    let root = s.graph().g_root();
    assert!(s.inspect_cell(root).is_some(), "root tracker must survive decay");

    // Re-ingest — root should still produce reports.
    let report = s.ingest(&cell_values(0xA, 8));
    assert!(
        report.ancestor_reports.iter().any(|r| r.depth == 0),
        "root must appear in ancestor_reports after decay + re-ingest"
    );
}

// ── Score invariants ────────────────────────────────────────

/// Two sentinels given identical settling traffic diverge in the expected
/// direction once one of them is fed structurally novel batches: its peak drift
/// reading ends up above the other's. Scores point one way — larger means more
/// anomalous — so a caller may threshold and compare them without having to
/// know which axis produced the number or which direction it runs in.
///
/// ´claim:invariant:scores-run-one-way-a-larger-reading-always-means-more-anomalous´
/// ´test:integration:score-polarity-higher-is-more-anomalous´
#[test]
fn score_polarity_higher_is_more_anomalous() {
    // Use a simple setup: repeated cell_values warm-up, then compare
    // max_cusum() under continued normal vs. anomalous traffic.
    let batch = cell_values(0xA, 8);

    let cfg = SentinelConfig::<u64> {
        max_rank: 2,
        split_threshold: 10,
        ..integration_config()
    };

    // ── Run A: continued normal traffic ──
    let mut sa = Sentinel128::new(cfg.clone()).unwrap();
    for _ in 0..20 {
        sa.ingest(&batch);
    }
    let normal_report = sa.ingest(&batch);
    let normal_cusum = max_cusum(&normal_report);

    // ── Run B: anomalous traffic after identical warm-up ──
    let mut sb = Sentinel128::new(cfg).unwrap();
    for _ in 0..20 {
        sb.ingest(&batch);
    }
    let anom_batch = anomalous_values(0xA, 8);
    for _ in 0..5 {
        sb.ingest(&anom_batch);
    }
    let anomaly_report = sb.ingest(&anom_batch);
    let anomaly_cusum = max_cusum(&anomaly_report);

    assert!(
        anomaly_cusum > normal_cusum,
        "anomalous data should produce higher CUSUM: anomaly={anomaly_cusum:.6}, normal={normal_cusum:.6}"
    );
}

/// Through a run of ordinary traffic, no drift accumulator on any axis of any
/// reported cell ever goes below nothing. The accumulator is clamped at rest
/// deliberately: a stretch of quieter-than-usual traffic must not bank negative
/// standing that a later attack could spend, so a rise always starts from zero
/// and means what it says.
///
/// ´claim:invariant:a-drift-accumulator-never-falls-below-zero-so-quiet-traffic-banks-no-credit´
/// ´test:integration:cusum-accumulators-non-negative´
#[test]
fn cusum_accumulators_non_negative() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(20).build();

    for _ in 0..10 {
        let report = s.ingest(&cell_values(0xA, 8));

        for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
            for (name, acc) in [
                ("novelty", cr.scores.novelty.cusum.accumulator),
                ("displacement", cr.scores.displacement.cusum.accumulator),
                ("surprise", cr.scores.surprise.cusum.accumulator),
                ("coherence", cr.scores.coherence.cusum.accumulator),
            ] {
                assert!(
                    acc >= 0.0,
                    "CUSUM {name} accumulator is negative ({acc}) at depth {}",
                    cr.depth
                );
            }
        }
    }
}

// ── Counters ────────────────────────────────────────────────

/// The lifetime observation count rises by exactly the size of each batch, for
/// batches ranging from a single value to many. It is a plain census of what
/// was handed in — never sampled, never rounded and never adjusted for what the
/// values looked like — which is what makes it usable as the denominator when
/// judging any rate the sentinel reports.
///
/// ´claim:invariant:the-lifetime-count-rises-by-exactly-the-size-of-each-batch´
/// ´test:integration:lifetime-observations-monotonically-increases´
#[test]
fn lifetime_observations_monotonically_increases() {
    let mut s = Sentinel128::new(integration_config()).unwrap();
    let mut prev = 0u64;

    for batch_size in [1, 5, 20, 3, 50] {
        s.ingest(&cell_values(0xA, batch_size));
        let current = s.lifetime_observations();
        assert_eq!(
            current,
            prev + batch_size as u64,
            "lifetime_observations should grow by batch size"
        );
        prev = current;
    }
}

/// At no point in a sentinel's life is it tracking nothing: not at birth before
/// any traffic, not through a run of ingestion, and not after ageing severe
/// enough to strip away everything that had accumulated. There is always at
/// least the root, so the question "what does the sentinel make of this value"
/// always has an answer.
///
/// ´claim:invariant:at-no-point-in-its-life-is-the-sentinel-tracking-nothing´
/// ´test:integration:cells-tracked-always-at-least-one´
#[test]
fn cells_tracked_always_at_least_one() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    assert!(s.cells_tracked() >= 1, "fresh sentinel must track at least the root");

    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 8));
        assert!(s.cells_tracked() >= 1, "cells_tracked must never drop below 1");
    }

    s.decay(0.0001, 0.0);
    assert!(s.cells_tracked() >= 1, "cells_tracked must stay ≥ 1 after extreme decay");
}

// ── Cross-cutting ───────────────────────────────────────────

/// Traffic with no structure at all — values scattered across the whole domain,
/// in batches whose size changes from one to the next — leaves every structural
/// property standing, checked after each batch. The guarantees are not
/// conditioned on the traffic being well behaved or on batches being uniform,
/// which is the whole point of calling them guarantees.
///
/// ´claim:invariant:the-whole-suite-holds-under-unstructured-traffic-in-batches-of-varying-size´
/// ´test:integration:assert-invariants-under-random-traffic´
#[test]
fn assert_invariants_under_random_traffic() {
    use std::hash::{DefaultHasher, Hash, Hasher};

    let mut s = Sentinel128::new(SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    })
    .unwrap();

    // Use a simple deterministic hash-based "random" generator.
    for batch_idx in 0..60u64 {
        let batch_size = (batch_idx % 63) as usize + 1; // 1–63
        let values: Vec<u128> = (0..batch_size)
            .map(|i| {
                let mut h = DefaultHasher::new();
                (batch_idx, i).hash(&mut h);
                u128::from(h.finish()) | (u128::from(h.finish()) << 64)
            })
            .collect();

        let report = s.ingest(&values);
        assert_invariants(&s, &report);
    }
}

/// A sentinel whose tree has been collapsed by severe ageing and then made to
/// regrow under traffic spread across the ranges satisfies every structural
/// property throughout the regrowth, batch by batch. The transient state of a
/// system rebuilding itself is exactly where a bound is likeliest to slip, so
/// the guarantees are asserted while it is in motion rather than once it has
/// settled.
///
/// ´claim:invariant:the-guarantees-hold-throughout-a-collapse-and-the-regrowth-that-follows´
/// ´test:integration:assert-invariants-after-decay-regrowth´
#[test]
fn assert_invariants_after_decay_regrowth() {
    let mut s = ScenarioBuilder::new()
        .seed_range(0xA, 20)
        .seed_range(0x5, 20)
        .warm_batches(20)
        .build();

    // Severe decay to collapse most cells.
    s.decay(0.001, 0.0);

    // Regrow with spread traffic so the tree re-balances.
    for i in 0..20u128 {
        let report = s.ingest(&cell_values(i % 16, 20));
        assert_invariants(&s, &report);
    }
}

/// The feed-forward count is checked in the accumulator's own domain rather
/// than through a floating-point projection, because past a certain magnitude
/// the projection cannot express a single arrival. The projection is lossy by
/// its own documentation, and at the first magnitude where consecutive
/// integers stop being separately representable, a total and that same total
/// plus one arrival land on the same number while a total plus two lands two
/// away. A check that projects both sides and allows them to differ by less
/// than one arrival therefore rejects arithmetic that is exactly right. The
/// accumulator keeps the distinction the projection loses, so the comparison
/// belongs there; this test pins the property the choice rests on rather than
/// the failure itself, which is some nine quadrillion observations away and
/// not reachable by a test.
///
/// ´claim:invariant:the-feed-forward-count-is-compared-in-the-accumulator-domain-because-the-projection-loses-a-single-arrival´
/// ´test:integration:a-single-arrival-outlives-the-projection-only-in-the-accumulator´
#[test]
fn a_single_arrival_outlives_the_projection_only_in_the_accumulator() {
    // The first magnitude at which consecutive integers stop being separately
    // representable in the projection's format.
    let total: u64 = 1u64 << 53;
    let one_more: u64 = total + 1;
    let two_more: u64 = total + 2;

    // The projection cannot tell one arrival from none at this magnitude.
    assert_eq!(
        total.to_f64_approx().to_bits(),
        one_more.to_f64_approx().to_bits(),
        "the projection separated one arrival"
    );
    // The accumulator's own comparison can.
    assert_ne!(total, one_more);

    // And the gap the projection does report can exceed a single arrival, so
    // an absolute tolerance of one arrival is not a safe reading of it.
    assert!((two_more.to_f64_approx() - one_more.to_f64_approx()).abs() > 1.0);
}
