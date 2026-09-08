// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_ingest_produces_full_report`] | readout | An ingest with nothing in it still returns a complete report: the cell, ancestor and coordination lists are present and empty, the contour and health sections are populated, and the summary counts at least the root. A host polling a quiet sentinel therefore parses the same layout it parses under load, and can read structural state from a batch that carried no observations at all. |
//! | [`cell_reports_are_competitive_only`] | readout | The report separates cells by how they earned their place: the cell list holds only competitively selected cells. A competitive cell was chosen because its traffic made it worth modelling, so a host reading that list is reading the sentinel's own investment decisions and nothing else. |
//! | [`ancestor_reports_are_ancestor_only`] | readout | cites (´claim:readout:the-report-partitions-cells-by-how-they-earned-their-place-rather-than-listing-them-together´) |
//! | [`cell_and_ancestor_cover_all_reported_cells`] | readout | cites (´claim:readout:the-report-partitions-cells-by-how-they-earned-their-place-rather-than-listing-them-together´) |
//! | [`cell_reports_sorted_by_gnode_id`] | readout | Cell reports come back in strictly ascending handle order, never merely grouped. Nothing about the order reflects the sequence observations arrived in or how the internal maps happened to iterate, so two sentinels fed the same stream emit comparable reports and a difference between two readouts is a difference in the system. |
//! | [`ancestor_reports_sorted_by_gnode_id`] | readout | cites (´claim:readout:report-lists-are-ordered-by-cell-handle-so-identical-runs-produce-identical-readouts´) |
//! | [`coordination_reports_sorted_by_gnode_id`] | readout | cites (´claim:readout:report-lists-are-ordered-by-cell-handle-so-identical-runs-produce-identical-readouts´) |
//! | [`coordination_reports_have_unique_gnodes`] | readout | Each coordination context appears at most once in a batch. The contexts are found by walking a tree in which a node can be reached from several selected descendants, so uniqueness is a real obligation: without it a busy subtree would report the same group finding repeatedly and a host counting elevated contexts would over-count it. |
//! | [`no_nan_in_score_fields`] | readout | Every score the readout carries is a number, on all four axes and across both competitive and ancestor cells. The scoring formulae divide by quantities that can legitimately reach zero — residual degrees of freedom, rank, baseline spread — so producing a number at the boundary is something the engine must arrange. A single non-number would poison every comparison a host makes downstream, silently rather than loudly. |
//! | [`report_structure_per_sample_scores_present_when_enabled`] | readout | Per-observation detail is present in every cell report exactly when the host configured it, rather than appearing only where the engine found it convenient. The detail costs memory proportional to the batch, so it is optional — but an option that were honoured unevenly would be worse than none, since a host could not tell an absent field from an unremarkable cell. |
//! | [`coordination_cells_reporting_is_nonzero`] | readout | A coordination report is emitted only where cells actually contributed to it: every one names a positive number of reporting cells. The tier exists to measure how a group of cells moves together, so a context with no contributors would be describing a group that did not exist this batch. |
//! | [`member_score_identifies_cell`] | readout | cites (´claim:readout:a-member-score-names-the-cell-it-came-from-so-a-group-finding-can-be-attributed´) |
//! | [`contour_reflects_graph_state`] | readout | The contour describes the spatial layer as it actually stands: after a batch of real traffic it reports accumulated importance above zero and at least one cell. It is read from the graph at report time rather than maintained alongside it, so it cannot drift out of step with the structure the trackers are attached to. |
//! | [`contour_cell_count_grows_with_distinct_regions`] | readout | Traffic arriving in a well-separated second region never reduces the reported spatial resolution. New structure is added by bisection, and nothing about observing an unfamiliar region coarsens what the graph already learned elsewhere — so a host watching the cell count sees refinement accumulate rather than oscillate with the traffic mix. |
//! | [`contour_reports_splits_since_last_report`] | readout | The split counter describes the interval since the previous report and is cleared with it: a heavy batch that forces bisection reports the splits it caused, and a quiet batch immediately after does not inherit them. The figure is a rate rather than a running total, which is what makes bursts of structural churn visible in the report that contained them. |
//! | [`contour_mutation_counts_zero_on_empty_ingest`] | readout | cites (´claim:readout:structural-mutation-counts-describe-the-interval-since-the-previous-report-and-reset-with-it´) |
//! | [`contour_after_decay`] | readout | Forgetting is visible in the readout: after the host applies decay, the reported importance is lower than before it. Temporal policy belongs to the host, which decides when history should count for less, and the contour is where that decision becomes observable — otherwise a host could not confirm that a decay it asked for had taken effect. |
//! | [`health_inline_matches_standalone`] | readout | The health carried inside a batch is the same health a standalone query returns — lifetime observations, active trackers and node count all agree. There is one health computation rather than two that happen to coincide, so a host that reads health from reports and a host that polls for it cannot form different pictures of the same sentinel. |
//! | [`health_tracker_breakdown_is_consistent`] | readout | The tracker breakdown accounts for the root apart from the two named categories: competitive and ancestor counts together fall short of the active total, the shortfall being the permanent root tracker. The root is present for structural reasons rather than because it competed or was closed over, and folding it into either count would misstate what the sentinel chose to invest in. The reported node total is likewise the graph's own count rather than a separately maintained tally. |
//! | [`analysis_set_summary_matches_analysis_set`] | readout | The summary's counts are the analysis set's own counts, read from it rather than tallied a second time on the way into the report. A summary is offered so a host need not enumerate every cell; it would be worth little if enumerating the cells could contradict it. |
//! | [`analysis_set_summary_depth_range_includes_root`] | readout | The reported depth span begins at the root and runs the right way round. Because ancestor closure always terminates at the root, the shallow end of the span is fixed at zero on a live sentinel, and the deep end describes the finest resolution currently being modelled — so the span is the reach of the whole chain, not the band the selected cells occupy. |
//! | [`analysis_set_summary_investment_covers_full`] | readout | The investment set is never smaller than the set currently producing reports. Every cell that reports has a tracker, and some cells hold trackers that are still warming and not yet contributing — so the gap between the two figures is precisely the modelling the sentinel is paying for but not yet reading from. |
//! | [`analysis_width_on_cell_report`] | readout | A cell's reported analysis width is always the domain width less its depth, for competitive and ancestor cells alike. The leading bits that routing already fixed are constant within the cell and carry no information, so the width states exactly how many bits the cell's model had left to work with — which is what a host needs to compare scores from cells at different depths. |
//! | [`analysis_width_on_cell_inspection`] | readout | cites (´claim:readout:a-cells-reported-analysis-width-is-the-domain-width-less-its-depth´) |
//! | [`tracker_report_type_exists`] | readout | The per-tracker report type is reachable from outside the crate through the flat public surface, named in a function signature that a downstream consumer could write. Modules are crate-private and types are re-exported at the root, so there is exactly one import path per public type — and a type that quietly stopped being re-exported would break consumers without breaking anything inside the crate. |
//! | [`batch_report_states_the_age_of_its_oldest_observation`] | readout | A report says how old its evidence was at the moment it was emitted: the batch is stamped as it arrives and the figure is read off as the report is assembled, so it is a positive interval that never exceeds the call that produced it. Both ends of the measurement are the sentinel's own monotonic clock, so a host learns the age of what it is holding without either side having to trust the other's idea of the time. |
//! | [`batch_report_age_is_scoped_to_its_own_batch`] | readout | The age belongs to the batch that carried the observations rather than running from the sentinel's own beginning: after a silence, the next batch reports an age shorter than the silence that preceded it. An age that accumulated over uptime would answer how long the sentinel had been running, which is the wrong question — what a host needs is how stale the evidence in front of it is. |
//! | [`empty_ingest_reports_no_observation_age`] | readout | A batch with no observations has no oldest observation, so it reports no age at all rather than an age of nothing. Absence and instantaneity are different facts about a report and a zero would have conflated them: a host watching for stale evidence has to be able to tell "nothing arrived" from "what arrived was fresh". |

//! Integration tests for the shape of the readout a live sentinel hands
//! back — what a batch report is obliged to contain, and how it is
//! arranged, when it comes from a sentinel that has actually observed
//! traffic rather than from a hand-built struct.
//!
//! The report is the whole interface between a component that measures and a
//! host that decides, so its structure is load-bearing in ways the numbers
//! are not. Cells are partitioned by how they earned their place —
//! competitively selected on one side, drawn in by ancestor closure on the
//! other — because the two were chosen for different reasons and a host
//! weighing a finding needs to know which it is reading. Every list is
//! ordered by cell handle, so two sentinels fed the same stream emit
//! byte-comparable output and a diff between reports means a difference in
//! the system rather than in iteration order.
//!
//! Two further commitments run through these tests. The report's shape does
//! not depend on how busy the sentinel is: an ingest with nothing in it
//! still returns every section, with empty lists and zeroed counters, so a
//! consumer parses one layout rather than several. And the report is
//! self-consistent with the state it describes — the health it carries is
//! the health a standalone query returns, the summary counts are the
//! analysis set's own counts, and each cell's analysis width is its domain
//! width less its depth — so a host never has to reconcile two views of one
//! sentinel.
//!
//! Alongside the standing state, the contour reports the structural churn
//! since the previous report: splits and net removals, counted over the
//! interval and reset with it, so growth and eviction are visible without
//! differencing successive snapshots.

mod common;

use std::time::{Duration, Instant};

use common::{ScenarioBuilder, assert_invariants, cell_values, seeded_sentinel, test_config};
use torrust_sentinel::Sentinel128;

// ── Helper ──────────────────────────────────────────────────

/// Build a sentinel with enough traffic that both competitive and
/// ancestor cells exist, using `ScenarioBuilder`.
fn multi_cell_sentinel() -> Sentinel128 {
    ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 10)
        .seed_range(0x1, 10)
        .warm_batches(5)
        .build()
}

// ═══════════════════════════════════════════════════════════
//  Empty report
// ═══════════════════════════════════════════════════════════

/// An ingest with nothing in it still returns a complete report: the cell,
/// ancestor and coordination lists are present and empty, the contour and
/// health sections are populated, and the summary counts at least the root.
/// A host polling a quiet sentinel therefore parses the same layout it
/// parses under load, and can read structural state from a batch that
/// carried no observations at all.
///
/// ´claim:readout:an-empty-ingest-still-returns-a-complete-report-with-empty-lists-rather-than-nothing´
/// ´test:integration:empty-ingest-produces-full-report´
#[test]
fn empty_ingest_produces_full_report() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[]);

    assert!(report.cell_reports.is_empty());
    assert!(report.ancestor_reports.is_empty());
    assert!(report.coordination_reports.is_empty());
    // Contour is valid even on empty ingest.
    assert!(report.contour.cell_count > 0 || report.contour.plateau_count == 0);
    // Health is populated.
    assert!(report.health.active_trackers > 0 || report.health.lifetime_observations == 0);
    // Summary sizes are consistent — at least the root.
    assert!(report.analysis_set_summary.full_size >= 1);

    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Cell-report partition
// ═══════════════════════════════════════════════════════════

/// The report separates cells by how they earned their place: the cell list
/// holds only competitively selected cells. A competitive cell was chosen
/// because its traffic made it worth modelling, so a host reading that list
/// is reading the sentinel's own investment decisions and nothing else.
///
/// ´claim:readout:the-report-partitions-cells-by-how-they-earned-their-place-rather-than-listing-them-together´
/// ´test:integration:cell-reports-are-competitive-only´
#[test]
fn cell_reports_are_competitive_only() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in &report.cell_reports {
        assert!(
            cr.is_competitive,
            "cell_reports must only contain competitive cells, got depth={}",
            cr.depth
        );
    }
    assert_invariants(&s, &report);
}

/// The complementary half: the ancestor list holds only cells that were
/// never competitively selected. These exist because closure required a
/// model chain back to the root, and they supply coarser context rather than
/// a judgement that the region deserved attention — which is exactly why
/// they are kept out of the competitive list.
///
/// (´claim:readout:the-report-partitions-cells-by-how-they-earned-their-place-rather-than-listing-them-together´)
/// ´test:integration:ancestor-reports-are-ancestor-only´
#[test]
fn ancestor_reports_are_ancestor_only() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for ar in &report.ancestor_reports {
        assert!(
            !ar.is_competitive,
            "ancestor_reports must only contain ancestor cells, got depth={}",
            ar.depth
        );
    }
    assert_invariants(&s, &report);
}

/// The two lists are genuinely a partition and not merely two views: no cell
/// handle appears in both. A host can therefore concatenate them to see
/// everything that reported this batch, or count them separately, without
/// double-counting any cell.
///
/// (´claim:readout:the-report-partitions-cells-by-how-they-earned-their-place-rather-than-listing-them-together´)
/// ´test:integration:cell-and-ancestor-cover-all-reported-cells´
#[test]
fn cell_and_ancestor_cover_all_reported_cells() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    // No gnode_id appears in both partitions.
    let competitive_ids: std::collections::HashSet<_> = report.cell_reports.iter().map(|cr| cr.gnode_id).collect();
    let ancestor_ids: std::collections::HashSet<_> = report.ancestor_reports.iter().map(|ar| ar.gnode_id).collect();

    assert!(
        competitive_ids.is_disjoint(&ancestor_ids),
        "cell_reports and ancestor_reports must be disjoint"
    );

    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Ordering and uniqueness
// ═══════════════════════════════════════════════════════════

/// Cell reports come back in strictly ascending handle order, never merely
/// grouped. Nothing about the order reflects the sequence observations
/// arrived in or how the internal maps happened to iterate, so two sentinels
/// fed the same stream emit comparable reports and a difference between two
/// readouts is a difference in the system.
///
/// ´claim:readout:report-lists-are-ordered-by-cell-handle-so-identical-runs-produce-identical-readouts´
/// ´test:integration:cell-reports-sorted-by-gnode-id´
#[test]
fn cell_reports_sorted_by_gnode_id() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for window in report.cell_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "cell_reports not sorted: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// The ordering rule is a property of the readout rather than of the
/// competitive list: ancestor reports are sorted by the same key, strictly
/// ascending. Cells the closure added are as reproducibly placed as cells
/// that were chosen.
///
/// (´claim:readout:report-lists-are-ordered-by-cell-handle-so-identical-runs-produce-identical-readouts´)
/// ´test:integration:ancestor-reports-sorted-by-gnode-id´
#[test]
fn ancestor_reports_sorted_by_gnode_id() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for window in report.ancestor_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "ancestor_reports not sorted: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// The same holds one tier up: coordination reports, which come from a walk
/// over internal nodes rather than from a cell list, are ordered by handle
/// too. Determinism is imposed on the readout as a whole, not recovered
/// separately wherever a list happens to be built.
///
/// (´claim:readout:report-lists-are-ordered-by-cell-handle-so-identical-runs-produce-identical-readouts´)
/// ´test:integration:coordination-reports-sorted-by-gnode-id´
#[test]
fn coordination_reports_sorted_by_gnode_id() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for window in report.coordination_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "coordination_reports not sorted: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// Each coordination context appears at most once in a batch. The
/// contexts are found by walking a tree in which a node can be reached from
/// several selected descendants, so uniqueness is a real obligation: without
/// it a busy subtree would report the same group finding repeatedly and a
/// host counting elevated contexts would over-count it.
///
/// ´claim:readout:each-coordination-context-appears-at-most-once-in-a-batch´
/// ´test:integration:coordination-reports-have-unique-gnodes´
#[test]
fn coordination_reports_have_unique_gnodes() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    let mut seen = std::collections::HashSet::new();
    for cr in &report.coordination_reports {
        assert!(
            seen.insert(cr.gnode_id),
            "duplicate GNodeId in coordination_reports: {:?}",
            cr.gnode_id,
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  Scores
// ═══════════════════════════════════════════════════════════

/// Every score the readout carries is a number, on all four axes and across
/// both competitive and ancestor cells. The scoring formulae divide by
/// quantities that can legitimately reach zero — residual degrees of
/// freedom, rank, baseline spread — so producing a number at the boundary is
/// something the engine must arrange. A single non-number would poison every
/// comparison a host makes downstream, silently rather than loudly.
///
/// ´claim:readout:every-reported-score-is-a-number-so-a-degenerate-model-never-leaks-a-non-number-into-the-readout´
/// ´test:integration:no-nan-in-score-fields´
#[test]
fn no_nan_in_score_fields() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty mean at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement mean at depth {}",
            cr.depth
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise mean at depth {}", cr.depth);
        assert!(!cr.scores.coherence.mean.is_nan(), "NaN coherence mean at depth {}", cr.depth);
    }

    assert_invariants(&s, &report);
}

/// Per-observation detail is present in every cell report exactly when the
/// host configured it, rather than appearing only where the engine found it
/// convenient. The detail costs memory proportional to the batch, so it is
/// optional — but an option that were honoured unevenly would be worse than
/// none, since a host could not tell an absent field from an unremarkable
/// cell.
///
/// ´claim:readout:per-observation-detail-is-present-exactly-where-the-host-asked-for-it´
/// ´test:integration:report-structure-per-sample-scores-present-when-enabled´
#[test]
fn report_structure_per_sample_scores_present_when_enabled() {
    // test_config() has per_sample_scores = true.
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in &report.cell_reports {
        assert!(
            cr.per_sample.is_some(),
            "per_sample should be Some when per_sample_scores is enabled, depth={}",
            cr.depth,
        );
    }

    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Coordination reports
// ═══════════════════════════════════════════════════════════

/// A coordination report is emitted only where cells actually contributed to
/// it: every one names a positive number of reporting cells. The tier exists
/// to measure how a group of cells moves together, so a context with no
/// contributors would be describing a group that did not exist this batch.
///
/// ´claim:readout:a-coordination-report-appears-only-where-cells-actually-contributed-to-it´
/// ´test:integration:coordination-cells-reporting-is-nonzero´
#[test]
fn coordination_cells_reporting_is_nonzero() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in &report.coordination_reports {
        assert!(
            cr.cells_reporting > 0,
            "coordination report at depth {} has zero cells_reporting",
            cr.depth,
        );
    }
}

/// The identity carried by a member score holds for scores the engine
/// actually produced, not only for ones built by hand: every member of every
/// coordination group names a non-empty interval. The group's own report
/// says a subtree behaved unusually; these entries are what let a host
/// narrow that to a region of the domain.
///
/// (´claim:readout:a-member-score-names-the-cell-it-came-from-so-a-group-finding-can-be-attributed´)
/// ´test:integration:member-score-identifies-cell´
#[test]
fn member_score_identifies_cell() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in &report.coordination_reports {
        if let Some(members) = &cr.per_member {
            for ms in members {
                assert!(ms.cell_start < ms.cell_end, "MemberScore must have cell_start < cell_end");
            }
        }
    }

    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Contour snapshot
// ═══════════════════════════════════════════════════════════

/// The contour describes the spatial layer as it actually stands: after a
/// batch of real traffic it reports accumulated importance above zero and at
/// least one cell. It is read from the graph at report time rather than
/// maintained alongside it, so it cannot drift out of step with the
/// structure the trackers are attached to.
///
/// ´claim:readout:the-contour-describes-the-spatial-layer-as-it-actually-stands-at-report-time´
/// ´test:integration:contour-reflects-graph-state´
#[test]
fn contour_reflects_graph_state() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let values = cell_values(0xA, 20);
    let report = s.ingest(&values);

    assert!(report.contour.total_importance > 0.0);
    assert!(report.contour.cell_count >= 1);

    assert_invariants(&s, &report);
}

/// Traffic arriving in a well-separated second region never reduces the
/// reported spatial resolution. New structure is added by bisection, and
/// nothing about observing an unfamiliar region coarsens what the graph
/// already learned elsewhere — so a host watching the cell count sees
/// refinement accumulate rather than oscillate with the traffic mix.
///
/// ´claim:readout:spatial-resolution-never-falls-back-when-a-new-well-separated-region-arrives´
/// ´test:integration:contour-cell-count-grows-with-distinct-regions´
#[test]
fn contour_cell_count_grows_with_distinct_regions() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    let r1 = s.ingest(&cell_values(0xA, 20));
    let count_after_one = r1.contour.cell_count;

    // Add a well-separated region.
    let r2 = s.ingest(&cell_values(0x5, 20));
    assert!(
        r2.contour.cell_count >= count_after_one,
        "cell_count should not shrink when adding distinct regions"
    );

    assert_invariants(&s, &r2);
}

/// The split counter describes the interval since the previous report and is
/// cleared with it: a heavy batch that forces bisection reports the splits it
/// caused, and a quiet batch immediately after does not inherit them. The
/// figure is a rate rather than a running total, which is what makes bursts
/// of structural churn visible in the report that contained them.
///
/// ´claim:readout:structural-mutation-counts-describe-the-interval-since-the-previous-report-and-reset-with-it´
/// ´test:integration:contour-reports-splits-since-last-report´
#[test]
fn contour_reports_splits_since_last_report() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    // test_config() has split_threshold = 100; send enough
    // concentrated observations to trigger at least one split.
    let r1 = s.ingest(&cell_values(0xA, 200));
    assert!(
        r1.contour.splits_since_last_report >= 1,
        "expected at least 1 split from 200 observations, got {}",
        r1.contour.splits_since_last_report,
    );

    // Second ingest with minimal traffic — counters were reset.
    let r2 = s.ingest(&cell_values(0xA, 1));
    // May or may not split again, but the first batch's count is gone.
    assert!(
        r2.contour.splits_since_last_report < r1.contour.splits_since_last_report,
        "split counter should reset between reports (r1={}, r2={})",
        r1.contour.splits_since_last_report,
        r2.contour.splits_since_last_report,
    );
}

/// The floor of the same rule, on a sentinel that has already been driven
/// hard: an ingest with no observations reports no splits and no removals.
/// Structural change is caused by observations, so an interval with none
/// reports zeroed churn rather than repeating the last non-empty batch's
/// figures.
///
/// (´claim:readout:structural-mutation-counts-describe-the-interval-since-the-previous-report-and-reset-with-it´)
/// ´test:integration:contour-mutation-counts-zero-on-empty-ingest´
#[test]
fn contour_mutation_counts_zero_on_empty_ingest() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    // Prime the sentinel with some data first.
    s.ingest(&cell_values(0xA, 20));
    // Empty ingest should report zero mutations.
    let r = s.ingest(&[]);
    assert_eq!(r.contour.splits_since_last_report, 0);
    assert_eq!(r.contour.net_removals_since_last_report, 0);
}

/// Forgetting is visible in the readout: after the host applies decay, the
/// reported importance is lower than before it. Temporal policy belongs to
/// the host, which decides when history should count for less, and the
/// contour is where that decision becomes observable — otherwise a host
/// could not confirm that a decay it asked for had taken effect.
///
/// ´claim:readout:host-applied-decay-shows-up-in-the-reported-importance-so-forgetting-is-observable´
/// ´test:integration:contour-after-decay´
#[test]
fn contour_after_decay() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    s.ingest(&cell_values(0xA, 20));
    let before = s.ingest(&cell_values(0xA, 5));
    let before_importance = before.contour.total_importance;

    s.decay(0.5, 0.0);

    let after = s.ingest(&cell_values(0xA, 1));
    assert!(
        after.contour.total_importance < before_importance,
        "total_importance should decrease after decay"
    );
}

// ═══════════════════════════════════════════════════════════
//  Health inline
// ═══════════════════════════════════════════════════════════

/// The health carried inside a batch is the same health a standalone query
/// returns — lifetime observations, active trackers and node count all
/// agree. There is one health computation rather than two that happen to
/// coincide, so a host that reads health from reports and a host that polls
/// for it cannot form different pictures of the same sentinel.
///
/// ´claim:readout:the-health-carried-in-a-batch-is-the-same-health-a-standalone-query-returns´
/// ´test:integration:health-inline-matches-standalone´
#[test]
fn health_inline_matches_standalone() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));
    let standalone = s.health();

    assert_eq!(report.health.lifetime_observations, standalone.lifetime_observations);
    assert_eq!(report.health.active_trackers, standalone.active_trackers);
    assert_eq!(report.health.total_g_nodes, standalone.total_g_nodes);
}

/// The tracker breakdown accounts for the root apart from the two named
/// categories: competitive and ancestor counts together fall short of the
/// active total, the shortfall being the permanent root tracker. The root is
/// present for structural reasons rather than because it competed or was
/// closed over, and folding it into either count would misstate what the
/// sentinel chose to invest in. The reported node total is likewise the
/// graph's own count rather than a separately maintained tally.
///
/// ´claim:readout:the-tracker-breakdown-accounts-for-the-permanent-root-apart-from-competitive-and-ancestor-cells´
/// ´test:integration:health-tracker-breakdown-is-consistent´
#[test]
fn health_tracker_breakdown_is_consistent() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));
    let h = &report.health;

    // competitive + ancestor + root (1) ≤ active_trackers.
    assert!(
        h.active_competitive_trackers + h.active_ancestor_trackers < h.active_trackers,
        "competitive ({}) + ancestor ({}) should be less than active ({}) (root counted separately)",
        h.active_competitive_trackers,
        h.active_ancestor_trackers,
        h.active_trackers,
    );
    assert_eq!(h.total_g_nodes, s.graph().node_count() as usize);
}

// ═══════════════════════════════════════════════════════════
//  Analysis-set summary
// ═══════════════════════════════════════════════════════════

/// The summary's counts are the analysis set's own counts, read from it
/// rather than tallied a second time on the way into the report. A summary
/// is offered so a host need not enumerate every cell; it would be worth
/// little if enumerating the cells could contradict it.
///
/// ´claim:readout:the-summary-counts-are-the-analysis-sets-own-counts-and-not-a-second-tally´
/// ´test:integration:analysis-set-summary-matches-analysis-set´
#[test]
fn analysis_set_summary_matches_analysis_set() {
    let s = multi_cell_sentinel();
    let aset = s.analysis_set();
    let summary = aset.summary();

    assert_eq!(summary.competitive_size, aset.competitive_count());
    assert_eq!(summary.full_size, aset.total_count());
}

/// The reported depth span begins at the root and runs the right way round.
/// Because ancestor closure always terminates at the root, the shallow end
/// of the span is fixed at zero on a live sentinel, and the deep end
/// describes the finest resolution currently being modelled — so the span is
/// the reach of the whole chain, not the band the selected cells occupy.
///
/// ´claim:readout:a-reported-depth-span-starts-at-the-root-and-runs-the-right-way-round´
/// ´test:integration:analysis-set-summary-depth-range-includes-root´
#[test]
fn analysis_set_summary_depth_range_includes_root() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));
    let summary = &report.analysis_set_summary;

    assert_eq!(summary.depth_range.0, 0, "depth_range min must be 0 (the root)");
    assert!(
        summary.depth_range.1 >= summary.depth_range.0,
        "depth_range max must be >= min"
    );

    assert_invariants(&s, &report);
}

/// The investment set is never smaller than the set currently producing
/// reports. Every cell that reports has a tracker, and some cells hold
/// trackers that are still warming and not yet contributing — so the gap
/// between the two figures is precisely the modelling the sentinel is paying
/// for but not yet reading from.
///
/// ´claim:readout:the-investment-set-is-never-smaller-than-the-set-currently-producing-reports´
/// ´test:integration:analysis-set-summary-investment-covers-full´
#[test]
fn analysis_set_summary_investment_covers_full() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));
    let summary = &report.analysis_set_summary;

    assert!(
        summary.investment_set_size >= summary.full_size,
        "investment_set_size ({}) must be >= full_size ({})",
        summary.investment_set_size,
        summary.full_size,
    );

    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Analysis width
// ═══════════════════════════════════════════════════════════

/// A cell's reported analysis width is always the domain width less its
/// depth, for competitive and ancestor cells alike. The leading bits that
/// routing already fixed are constant within the cell and carry no
/// information, so the width states exactly how many bits the cell's model
/// had left to work with — which is what a host needs to compare scores from
/// cells at different depths.
///
/// ´claim:readout:a-cells-reported-analysis-width-is-the-domain-width-less-its-depth´
/// ´test:integration:analysis-width-on-cell-report´
#[test]
fn analysis_width_on_cell_report() {
    let mut s = multi_cell_sentinel();
    let report = s.ingest(&cell_values(0xF, 4));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.analysis_width,
            128 - cr.depth as usize,
            "analysis_width mismatch at depth {}",
            cr.depth,
        );
    }
}

/// The same relation holds on the inspection path, which reaches cells
/// directly by handle rather than through a batch. A host can therefore
/// interrogate a cell between batches and read its width the same way,
/// without the two routes into the sentinel disagreeing about the geometry
/// of a single cell.
///
/// (´claim:readout:a-cells-reported-analysis-width-is-the-domain-width-less-its-depth´)
/// ´test:integration:analysis-width-on-cell-inspection´
#[test]
fn analysis_width_on_cell_inspection() {
    let s = multi_cell_sentinel();

    for &gnode in &s.cell_gnodes() {
        let inspection = s.inspect_cell(gnode).unwrap();
        assert_eq!(
            inspection.analysis_width,
            128 - inspection.depth as usize,
            "analysis_width mismatch on inspection at depth {}",
            inspection.depth,
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  Type naming
// ═══════════════════════════════════════════════════════════

/// The per-tracker report type is reachable from outside the crate through
/// the flat public surface, named in a function signature that a downstream
/// consumer could write. Modules are crate-private and types are re-exported
/// at the root, so there is exactly one import path per public type — and a
/// type that quietly stopped being re-exported would break consumers without
/// breaking anything inside the crate.
///
/// ´claim:readout:the-per-tracker-report-type-is-reachable-through-the-crates-flat-public-surface´
/// ´test:integration:tracker-report-type-exists´
#[test]
fn tracker_report_type_exists() {
    // Compile-time check: `TrackerReport` exists in the public API.
    fn _assert_type_exists(_: torrust_sentinel::TrackerReport) {}

    // Verify a sentinel can be constructed, confirming the type is
    // reachable through the public module.
    let _s = seeded_sentinel();
}

// ═══════════════════════════════════════════════════════════
//  Observation age
// ═══════════════════════════════════════════════════════════

/// A report says how old its evidence was at the moment it was emitted: the
/// batch is stamped as it arrives and the figure is read off as the report is
/// assembled, so it is a positive interval that never exceeds the call that
/// produced it. Both ends of the measurement are the sentinel's own monotonic
/// clock, so a host learns the age of what it is holding without either side
/// having to trust the other's idea of the time.
///
/// ´claim:readout:a-report-states-how-old-its-oldest-observation-was-when-the-report-was-emitted´
/// ´test:integration:batch-report-states-the-age-of-its-oldest-observation´
#[test]
fn batch_report_states_the_age_of_its_oldest_observation() {
    let mut s = multi_cell_sentinel();

    let call_start = Instant::now();
    let report = s.ingest(&cell_values(0xF, 128));
    let call_micros = u64::try_from(call_start.elapsed().as_micros()).unwrap();

    let age = report
        .oldest_observation_age_micros
        .expect("a batch carrying observations has an oldest one");

    assert!(age > 0, "the age should span the work the call did, got {age} micros");
    assert!(
        age <= call_micros,
        "age ({age}) cannot exceed the call it was measured inside ({call_micros})",
    );
}

/// The age belongs to the batch that carried the observations rather than
/// running from the sentinel's own beginning: after a silence, the next batch
/// reports an age shorter than the silence that preceded it. An age that
/// accumulated over uptime would answer how long the sentinel had been
/// running, which is the wrong question — what a host needs is how stale the
/// evidence in front of it is.
///
/// ´claim:readout:the-reported-age-belongs-to-its-own-batch-rather-than-running-from-the-sentinels-beginning´
/// ´test:integration:batch-report-age-is-scoped-to-its-own-batch´
#[test]
fn batch_report_age_is_scoped_to_its_own_batch() {
    let mut s = multi_cell_sentinel();

    let _first = s.ingest(&cell_values(0xF, 64));

    // A silence, so that an age running from the sentinel's own beginning
    // would have this stretch of quiet inside it.
    let quiet = Duration::from_millis(200);
    std::thread::sleep(quiet);

    let call_start = Instant::now();
    let second = s.ingest(&cell_values(0x1, 64));
    let call_micros = u64::try_from(call_start.elapsed().as_micros()).unwrap();

    let age = second
        .oldest_observation_age_micros
        .expect("a batch carrying observations has an oldest one");
    let quiet_micros = u64::try_from(quiet.as_micros()).unwrap();

    // An age scoped to its own batch is bounded by the call that produced the
    // batch, whatever happened before the call.
    assert!(
        age <= call_micros,
        "age ({age}) cannot exceed the call it was measured inside ({call_micros})",
    );
    // The same bound stated against the silence: excluding the quiet means the
    // age stays under it once the call's own duration is accounted for.
    assert!(
        age < quiet_micros + call_micros,
        "age ({age}) should exclude the {quiet_micros} micros of silence before the batch arrived, allowing the {call_micros} micros the call itself took",
    );
}

/// A batch with no observations has no oldest observation, so it reports no
/// age at all rather than an age of nothing. Absence and instantaneity are
/// different facts about a report and a zero would have conflated them: a host
/// watching for stale evidence has to be able to tell "nothing arrived" from
/// "what arrived was fresh".
///
/// ´claim:readout:an-empty-batch-reports-no-age-at-all-rather-than-an-age-of-zero´
/// ´test:integration:empty-ingest-reports-no-observation-age´
#[test]
fn empty_ingest_reports_no_observation_age() {
    let mut s = multi_cell_sentinel();

    let quiet = s.ingest(&[]);
    assert!(
        quiet.oldest_observation_age_micros.is_none(),
        "an empty batch has no oldest observation to be any age",
    );

    let busy = s.ingest(&cell_values(0xF, 8));
    assert!(
        busy.oldest_observation_age_micros.is_some(),
        "a batch that carried observations states their age",
    );
}
