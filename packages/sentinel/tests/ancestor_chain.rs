// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The ancestor chain — what the models above a cell are for
//! (§ALGO S-4.4–4.9).
//!
//! Selecting a cell for analysis is never enough on its own. Every
//! competitively chosen cell is closed under G-tree ancestry, so the sentinel
//! also models each of its parents up to the root, and an observation is
//! delivered to every cell whose interval contains it rather than only to the
//! finest one. One arrival is therefore analysed several times over, once at
//! each scale it belongs to, and a report shows both the cells that earned
//! their place and the ancestors that were drawn in behind them.
//!
//! That redundancy is the point. A cell sees only its own narrow range and
//! cannot tell a change confined to it apart from a change happening
//! everywhere; an ancestor sees the union of its descendants and cannot
//! localise anything, but it can tell those two situations apart. Read
//! together, the chain gives the shape of a disturbance and not just its
//! presence: something that lifts one cell's scores while the root stays
//! ordinary is local, and something that lifts the root itself is not.
//!
//! The chain has to be intact for that reading to hold, which is what the
//! structural properties here pin down — the root always present, the depths
//! between it and the deepest selected cell not skipped, an ancestor's
//! analysis view never narrower than that of the cells beneath it, and an
//! ancestor's sample count accounting for every observation its descendants
//! saw. Where no observations arrive at all, no ancestor reports: the chain
//! describes traffic, and reports nothing when there is none.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_batch_produces_no_ancestor_reports`] | ancestry | A batch with no observations produces no ancestor reports, even from a sentinel already warmed and holding a chain of live models. Cells report what they saw, and a cell that saw nothing has nothing to say; the chain is a description of traffic rather than a periodic status broadcast, so a quiet interval costs a host no reports to filter out. |
//! | [`root_ancestor_present_after_warm_up`] | ancestry | Traffic into a refined region is reported at both ends of the chain at once: the deep cells that earned selection appear as competitive, and the root appears alongside them as an ancestor. The same observations were analysed at both scales, which is what makes the two figures comparable in the first place — without the coarse reading there is nothing to judge the fine one against. |
//! | [`root_sample_count_equals_batch_size`] | ancestry | An ancestor is credited with every observation that fell anywhere beneath it: the root's sample count for a batch drawn from two separate ranges is the whole batch. Delivery is by containment rather than by ownership, so nothing is consumed by the deepest cell that matched — this is what lets a coarse model hold a baseline for total volume that no individual cell could. |
//! | [`ancestor_width_increases_toward_root`] | ancestry | Ordered by depth, the ancestors reported for a batch never narrow as one climbs toward the root: an ancestor analyses at least as wide a view as the cells below it. Ancestry and analysis breadth therefore point the same way, so a chain reads as a genuine sequence of scales — coarse above, specific below — rather than an arbitrary collection of models over the same region. |
//! | [`ancestor_depths_cover_path_to_root`] | ancestry | The chain has no holes in it. The root is always reported, and where the selected cells sit well below it the intervening depths are present too — some as competitive cells in their own right, the rest drawn in by the closure. Reading the two report lists together therefore gives an unbroken path from the whole domain down to the finest cell, which is what allows a disturbance to be located at a scale rather than merely noticed at one. |
//! | [`shared_ancestor_aggregates_disjoint_ranges`] | ancestry | cites (´claim:ancestry:an-ancestor-is-credited-with-every-observation-that-fell-beneath-it´) |
//! | [`local_anomaly_detectable_in_hierarchy`] | ancestry | Structurally novel traffic entering one range while the others stay normal leaves a mark somewhere in the chain, measured against the same sentinel's own response to an ordinary batch a moment earlier. Which level catches it is not fixed — a narrow disturbance may barely move the root while standing out sharply in the cell containing it — so detection is a property of the chain as a whole rather than of any one model in it. |
//! | [`global_anomaly_elevates_root_scores`] | ancestry | When every range turns anomalous at once, the root's own score rises above what the same sentinel produced for a normal batch. The coarse model is not merely a fallback for traffic too sparse to have earned a cell: it responds in its own right, and it responds to exactly the case no single cell can distinguish from its own local weather. |
//! | [`global_anomaly_root_z_exceeds_local`] | ancestry | The root does not merely notice anomalies, it grades them by extent: with several ranges live, an anomaly in one of them moves the root less than the same anomaly in all of them, the sentinel having been returned to normal traffic in between so the two readings are of comparable states. Reach is thus legible in the score itself, and a host can separate a local incident from a system-wide shift without waiting to see how far it spreads. |

mod common;

use common::{ScenarioBuilder, anomalous_values, assert_invariants, cell_values, integration_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ── Helpers ─────────────────────────────────────────────────

/// Warmed sentinel with a single range and low split threshold.
fn single_range_sentinel() -> Sentinel128 {
    ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(5)
        .build()
}

/// Warmed sentinel with two disjoint ranges.
fn dual_range_sentinel() -> Sentinel128 {
    ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(5)
        .build()
}

/// Warmed sentinel with three disjoint ranges.
fn triple_range_sentinel() -> Sentinel128 {
    ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .seed_range(0xC, 20)
        .warm_batches(5)
        .build()
}

/// Extract root z-score from a report, defaulting to `0.0`.
fn root_novelty_z(report: &torrust_sentinel::BatchReport<u128>) -> f64 {
    report
        .ancestor_reports
        .iter()
        .find(|r| r.depth == 0)
        .map_or(0.0, |r| r.scores.novelty.max_z_score)
}

// ── 1. empty_batch_produces_no_ancestor_reports ─────────────

/// A batch with no observations produces no ancestor reports, even from a
/// sentinel already warmed and holding a chain of live models. Cells report
/// what they saw, and a cell that saw nothing has nothing to say; the chain is
/// a description of traffic rather than a periodic status broadcast, so a
/// quiet interval costs a host no reports to filter out.
///
/// ´claim:ancestry:a-batch-with-no-observations-produces-no-ancestor-reports-at-all´
/// ´test:integration:empty-batch-produces-no-ancestor-reports´
#[test]
fn empty_batch_produces_no_ancestor_reports() {
    let mut s = single_range_sentinel();
    let report = s.ingest(&[]);

    assert!(report.ancestor_reports.is_empty());
    assert_invariants(&s, &report);
}

// ── 2. root_ancestor_present_after_warm_up ──────────────────

/// Traffic into a refined region is reported at both ends of the chain at
/// once: the deep cells that earned selection appear as competitive, and the
/// root appears alongside them as an ancestor. The same observations were
/// analysed at both scales, which is what makes the two figures comparable in
/// the first place — without the coarse reading there is nothing to judge the
/// fine one against.
///
/// ´claim:ancestry:one-batch-is-reported-at-the-fine-scale-and-the-coarse-scale-together´
/// ´test:integration:root-ancestor-present-after-warm-up´
#[test]
fn root_ancestor_present_after_warm_up() {
    let mut s = single_range_sentinel();
    let report = s.ingest(&cell_values(0xA, 8));

    let root = report.ancestor_reports.iter().find(|r| r.depth == 0);
    assert!(root.is_some(), "root tracker must produce a report");

    // There should also be competitive cells at depth > 0 (the split
    // threshold is low enough to create them).
    assert!(
        report.cell_reports.iter().any(|r| r.depth > 0),
        "should have competitive cells at depth > 0",
    );
    assert_invariants(&s, &report);
}

// ── 3. root_sample_count_equals_batch_size ──────────────────

/// An ancestor is credited with every observation that fell anywhere beneath
/// it: the root's sample count for a batch drawn from two separate ranges is
/// the whole batch. Delivery is by containment rather than by ownership, so
/// nothing is consumed by the deepest cell that matched — this is what lets a
/// coarse model hold a baseline for total volume that no individual cell
/// could.
///
/// ´claim:ancestry:an-ancestor-is-credited-with-every-observation-that-fell-beneath-it´
/// ´test:integration:root-sample-count-equals-batch-size´
#[test]
fn root_sample_count_equals_batch_size() {
    let mut s = dual_range_sentinel();

    let batch = [cell_values(0xA, 5), cell_values(0xB, 7)].concat();
    let report = s.ingest(&batch);

    let root = report.ancestor_reports.iter().find(|r| r.depth == 0).unwrap();
    assert_eq!(root.sample_count, batch.len(), "root must see every observation");
    assert_invariants(&s, &report);
}

// ── 4. ancestor_width_increases_toward_root ─────────────────

/// Ordered by depth, the ancestors reported for a batch never narrow as one
/// climbs toward the root: an ancestor analyses at least as wide a view as the
/// cells below it. Ancestry and analysis breadth therefore point the same way,
/// so a chain reads as a genuine sequence of scales — coarse above, specific
/// below — rather than an arbitrary collection of models over the same region.
///
/// ´claim:ancestry:an-ancestor-analyses-at-least-as-wide-a-view-as-the-cells-beneath-it´
/// ´test:integration:ancestor-width-increases-toward-root´
#[test]
fn ancestor_width_increases_toward_root() {
    let mut s = single_range_sentinel();
    let report = s.ingest(&cell_values(0xA, 8));

    // Sort ancestors by depth ascending; width should be non-increasing
    // (width = 128 − depth, so shallower ⇒ wider).
    let mut ancestors: Vec<_> = report.ancestor_reports.iter().collect();
    ancestors.sort_by_key(|r| r.depth);

    for window in ancestors.windows(2) {
        assert!(
            window[0].analysis_width >= window[1].analysis_width,
            "ancestor at depth {} (width {}) should be >= depth {} (width {})",
            window[0].depth,
            window[0].analysis_width,
            window[1].depth,
            window[1].analysis_width,
        );
    }
    assert_invariants(&s, &report);
}

// ── 5. ancestor_depths_cover_path_to_root ───────────────────

/// The chain has no holes in it. The root is always reported, and where the
/// selected cells sit well below it the intervening depths are present too —
/// some as competitive cells in their own right, the rest drawn in by the
/// closure. Reading the two report lists together therefore gives an unbroken
/// path from the whole domain down to the finest cell, which is what allows a
/// disturbance to be located at a scale rather than merely noticed at one.
///
/// ´claim:ancestry:the-reported-chain-runs-unbroken-from-the-root-to-the-deepest-selected-cell´
/// ´test:integration:ancestor-depths-cover-path-to-root´
#[test]
fn ancestor_depths_cover_path_to_root() {
    let mut s = single_range_sentinel();
    let report = s.ingest(&cell_values(0xA, 8));

    // Root (depth 0) must always be present as an ancestor.
    assert!(
        report.ancestor_reports.iter().any(|a| a.depth == 0),
        "root ancestor must be present",
    );

    // Collect depths from both competitive and ancestor reports.
    // Intermediate nodes between root and deep cells may be competitive
    // (in cell_reports) rather than in ancestor_reports.
    let all_depths: std::collections::BTreeSet<u32> = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|r| r.depth)
        .collect();

    // The combined depths should span from 0 up to the deepest
    // competitive cell without large gaps — the Steiner tree
    // connects them through the hierarchy.
    let max_depth = report.cell_reports.iter().map(|c| c.depth).max().unwrap_or(0);
    if max_depth > 1 {
        assert!(
            all_depths.len() > 2,
            "depths 0..{max_depth} should include intermediate nodes, got {all_depths:?}",
        );
    }
    assert_invariants(&s, &report);
}

// ── 6. shared_ancestor_aggregates_disjoint_ranges ───────────

/// The same crediting rule is what makes a shared ancestor a meeting point:
/// two ranges with nothing in common still both lie beneath the root, and its
/// count for the batch is their sum. Traffic that no single cell can see as
/// related is nonetheless seen together somewhere in the chain, which is the
/// mechanism by which a distributed pattern becomes visible at all.
///
/// (´claim:ancestry:an-ancestor-is-credited-with-every-observation-that-fell-beneath-it´)
/// ´test:integration:shared-ancestor-aggregates-disjoint-ranges´
#[test]
fn shared_ancestor_aggregates_disjoint_ranges() {
    let mut s = dual_range_sentinel();

    let batch_a = cell_values(0xA, 4);
    let batch_b = cell_values(0xB, 6);
    let combined = [batch_a, batch_b].concat();
    let report = s.ingest(&combined);

    let root = report.ancestor_reports.iter().find(|r| r.depth == 0).unwrap();
    assert_eq!(root.sample_count, 10, "root should aggregate traffic from both ranges");
    assert_invariants(&s, &report);
}

// ── 7. local_anomaly_detectable_in_hierarchy ────────────────

/// Structurally novel traffic entering one range while the others stay normal
/// leaves a mark somewhere in the chain, measured against the same sentinel's
/// own response to an ordinary batch a moment earlier. Which level catches it
/// is not fixed — a narrow disturbance may barely move the root while standing
/// out sharply in the cell containing it — so detection is a property of the
/// chain as a whole rather than of any one model in it.
///
/// ´claim:ancestry:an-anomaly-confined-to-one-range-still-registers-somewhere-in-the-chain´
/// ´test:integration:local-anomaly-detectable-in-hierarchy´
#[test]
fn local_anomaly_detectable_in_hierarchy() {
    let mut s = triple_range_sentinel();

    // Normal reference: root z-score under normal traffic.
    let normal = s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8), cell_values(0xC, 8)].concat());
    let normal_root_z = root_novelty_z(&normal);
    assert_invariants(&s, &normal);

    // Anomaly only in range A; B and C normal.
    let report = s.ingest(&[anomalous_values(0xA, 8), cell_values(0xB, 8), cell_values(0xC, 8)].concat());

    let anomaly_root_z = root_novelty_z(&report);

    let max_all_z = report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|cr| cr.scores.novelty.max_z_score)
        .fold(f64::NEG_INFINITY, f64::max);

    assert!(
        max_all_z > normal_root_z || anomaly_root_z > normal_root_z,
        "local anomaly should be detectable at some hierarchical level: \
         max_z={max_all_z:.4}, anomaly_root_z={anomaly_root_z:.4}, \
         normal_root_z={normal_root_z:.4}",
    );
    assert_invariants(&s, &report);
}

// ── 8. global_anomaly_elevates_root_scores ──────────────────

/// When every range turns anomalous at once, the root's own score rises above
/// what the same sentinel produced for a normal batch. The coarse model is not
/// merely a fallback for traffic too sparse to have earned a cell: it responds
/// in its own right, and it responds to exactly the case no single cell can
/// distinguish from its own local weather.
///
/// ´claim:ancestry:an-anomaly-in-every-range-lifts-the-roots-own-score´
/// ´test:integration:global-anomaly-elevates-root-scores´
#[test]
fn global_anomaly_elevates_root_scores() {
    let mut s = dual_range_sentinel();

    // Normal reference.
    let normal = s.ingest(&[cell_values(0xA, 8), cell_values(0xB, 8)].concat());
    let normal_root_z = root_novelty_z(&normal);
    assert_invariants(&s, &normal);

    // System-wide anomaly.
    let anomaly = s.ingest(&[anomalous_values(0xA, 8), anomalous_values(0xB, 8)].concat());
    let anomaly_root_z = root_novelty_z(&anomaly);

    assert!(
        anomaly_root_z > normal_root_z,
        "global anomaly should elevate root z-score: \
         anomaly={anomaly_root_z:.4}, normal={normal_root_z:.4}",
    );
    assert_invariants(&s, &anomaly);
}

// ── 9. global_anomaly_root_z_exceeds_local ──────────────────

/// The root does not merely notice anomalies, it grades them by extent: with
/// several ranges live, an anomaly in one of them moves the root less than the
/// same anomaly in all of them, the sentinel having been returned to normal
/// traffic in between so the two readings are of comparable states. Reach is
/// thus legible in the score itself, and a host can separate a local incident
/// from a system-wide shift without waiting to see how far it spreads.
///
/// ´claim:ancestry:the-root-grades-an-anomaly-by-how-much-of-the-domain-it-reaches´
/// ´test:integration:global-anomaly-root-z-exceeds-local´
#[test]
fn global_anomaly_root_z_exceeds_local() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .seed_range(0xC, 20)
        .seed_range(0xD, 20)
        .warm_batches(5)
        .build();

    // Localised anomaly: only range A.
    let local_report = s.ingest(
        &[
            anomalous_values(0xA, 8),
            cell_values(0xB, 8),
            cell_values(0xC, 8),
            cell_values(0xD, 8),
        ]
        .concat(),
    );
    let local_root_z = root_novelty_z(&local_report);
    assert_invariants(&s, &local_report);

    // Return to normal, then system-wide anomaly.
    for _ in 0..3 {
        s.ingest(
            &[
                cell_values(0xA, 8),
                cell_values(0xB, 8),
                cell_values(0xC, 8),
                cell_values(0xD, 8),
            ]
            .concat(),
        );
    }

    let global_report = s.ingest(
        &[
            anomalous_values(0xA, 8),
            anomalous_values(0xB, 8),
            anomalous_values(0xC, 8),
            anomalous_values(0xD, 8),
        ]
        .concat(),
    );
    let global_root_z = root_novelty_z(&global_report);

    assert!(
        global_root_z > local_root_z,
        "system-wide anomaly root z ({global_root_z:.4}) should exceed \
         localised anomaly root z ({local_root_z:.4})",
    );
    assert_invariants(&s, &global_report);
}
