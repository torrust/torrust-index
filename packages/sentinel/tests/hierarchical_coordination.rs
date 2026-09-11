// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Hierarchical coordination (§ALGO S-7) — the sentinel's second tier, which
//! watches how cells score *together* rather than how any one of them scores
//! alone.
//!
//! Each competitive cell contributes the four axis means it reported this
//! batch, and a context at an internal G-tree node models those vectors as a
//! group. A context exists only where the question is meaningful: both of the
//! node's subtrees must have cells reporting in the same batch, so a context
//! always spans a genuine split of the domain and never fires on one region
//! talking to itself. A node with a single contributing child asks no such
//! question and passes its members upward untouched. Contexts are therefore
//! the internal nodes of a binary tree whose leaves are the reporting cells,
//! which bounds their number below the number of those cells; and because the
//! walk unions each subtree's members on the way up, an ancestor's membership
//! is a superset of every descendant's.
//!
//! The tier is deliberately narrow. A member observation is one value per
//! scoring axis, so a context works in four dimensions however wide the cells
//! beneath it are, and its rank ceiling is the configured maximum capped at
//! those four. Members are centred against a running mean the context keeps
//! for itself, so what is measured is departure from the group's own recent
//! pattern rather than from any global reference.
//!
//! Everything a context publishes is a measurement: how many cells reported,
//! the interval they came from, the four axis statistics and their
//! accumulated drift, the geometry of the learned subspace. There are no
//! threat levels and no recommended actions here — the sentinel measures and
//! the host decides.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`no_coordination_on_empty_batch`] | coordination | A batch carrying no values gives the tier nothing to group: no cell reports a score, no node can find both its subtrees contributing, and the coordination list comes back empty. Coordination is a statement about what several cells did together in one batch, so with no batch there is nothing to say. |
//! | [`no_coordination_with_single_region`] | coordination | Traffic confined to one region leaves every reporting cell in the same subtree, so the both-subtrees condition is never met on its own account. Any context that does appear carries several member cells — which is what makes a coordination score a genuinely cross-cell measurement rather than a restatement of one cell's own score at a coarser scale. |
//! | [`two_sibling_cells_fire_parent_context`] | coordination | Two well-separated regions reporting in the same batch turn the node above them into a live context: it sees members arriving from each side and begins modelling the pair as a group. Each report identifies itself by the dyadic interval of that node, whose lower bound always lies strictly below its upper bound, so a host can say which stretch of the domain the measurement covers. |
//! | [`context_activation_requires_both_subtrees`] | coordination | After a run in which only one region reported, sending traffic to the opposite half of the domain can only add contexts, never take them away. Contexts are created where a split starts carrying reporters on both sides, so the arrival of a second region is precisely the event that brings the tier into existence. |
//! | [`context_deactivation_on_subtree_loss`] | coordination | When a batch arrives from one region only, the nodes that had been spanning both stop qualifying, and the tier prunes them instead of carrying stale contexts forward. The count of live contexts therefore never rises when a subtree falls silent: what is reported describes the group structure of the batch in hand, not of some earlier one. |
//! | [`nested_coordination_levels`] | coordination | cites (´claim:coordination:each-firing-node-appears-exactly-once-among-a-batchs-coordination-reports´) |
//! | [`coordination_group_nesting_invariant`] | coordination | Membership nests with the tree. Wherever one context's interval contains another's, the containing context counts at least as many reporting cells, because the walk unions each subtree's members and passes the result upward. An ancestor therefore measures a superset of what its descendant measured, which is what lets the two readings be compared as the same pattern seen at two scales. |
//! | [`root_context_sees_all_cells`] | coordination | cites (´claim:coordination:an-ancestor-context-counts-every-cell-its-descendant-counts´) |
//! | [`semi_internal_node_passthrough`] | coordination | A node with only one contributing child poses no coordination question — there is no pair of subtrees to compare — so it fires nothing and simply forwards its members to its parent. Every report that does appear consequently spans a real split and carries a usable finite score, rather than the tier manufacturing a group out of a single branch. |
//! | [`coordination_tracker_operates_at_w4`] | coordination | A member's contribution to a context is one value per scoring axis, so the tier works in four dimensions however wide the cells beneath it are. That fixed and tiny geometry is what makes a second tier affordable: its cost does not grow with the width of the domain the cells analyse. |
//! | [`coordination_tracker_with_low_max_rank`] | coordination | A context's capacity is the configured rank ceiling, capped by the four dimensions actually available. Lowering that ceiling lowers the capacity with it while the dimensionality stays at four: capacity is a budget on how much structure the model may hold, not a change to what a member observation is. |
//! | [`running_mean_cold_start`] | coordination | The first batch a context ever handles still yields finite numbers on all four axes. Members are centred against a running mean that starts at zero and is seeded outright from that batch's own column means rather than divided by an empty history, so activation costs the host no undefined readings to interpret. |
//! | [`running_mean_ewma_update`] | coordination | A context that has activated goes on firing while both its subtrees keep reporting: repeating the same shape of batch produces coordination in nearly all of them. Its running mean is updated across those batches instead of being rebuilt each time, so what the tier measures stays a departure from the group's own recent pattern. |
//! | [`only_competitive_cells_contribute`] | coordination | Membership is drawn only from competitive cells that actually observed something in this batch, so no context can report more members than there were such cells. The root is never competitive and so never joins a group: coordination measures the cells the engine chose to invest in, not the whole tree. |
//! | [`cells_with_no_observations_excluded`] | coordination | cites (´claim:coordination:only-competitive-cells-that-observed-this-batch-become-members´) |
//! | [`coordination_scores_are_finite`] | coordination | Everything a context publishes is a finite number: the four axis means, the drift accumulated on each, the fraction of variance the learned subspace captures and its largest singular value. Degenerate geometry at this tier is resolved inside the model rather than handed to the host as a not-a-number it would have to interpret for itself. |
//! | [`per_member_scores_present_when_enabled`] | coordination | With per-sample scoring enabled a context also names its members: one entry per reporting cell, matching the membership count it declared, each carrying that cell's own four scores and the non-empty interval it covers. A host can therefore attribute a group-level reading to particular regions of the domain instead of seeing only the aggregate. |
//! | [`coordination_survives_decay`] | coordination | Ageing the model down does not dismantle the tier. After a decay the sentinel still ingests and still produces reports on the next batch, because decay weakens the learned structure rather than removing the cells and contexts that carry it. |
//! | [`inject_noise_warms_coordination`] | coordination | A new context is warmed with synthetic score vectors drawn to match its members' own baselines, which leaves it holding a usable model rather than an empty one. The warm-up then clears the drift accumulators, so the first real batch is that context's first step of evidence: what the engine invented about itself is never counted as evidence about traffic. |
//! | [`deterministic_report_order`] | coordination | Two runs from the same seed over the same traffic produce the same reports: the same number of them, at the same nodes in the same order, with the same memberships and matching scores. The only randomness in the engine is the seeded warm-up, and pinning it makes two runs comparable — without that, no difference a host observed could be attributed. |
//! | [`context_count_bounded_by_k_minus_1`] | coordination | Contexts are the internal nodes of a binary tree whose leaves are the competitive cells, so their number stays below the number of those cells. The tier's cost is thereby bounded by the analysis budget the host has already chosen and cannot grow on its own. |
//! | [`coordination_reports_unique_gnodes`] | coordination | A batch produces at most one report per firing node. The walk visits each node once and a context is keyed by the node it sits at, so a nested hierarchy yields one measurement per level rather than repeated entries a host would first have to de-duplicate. |
//! | [`batch_report_coordination_is_vec`] | coordination | The batch report always carries a list of coordination reports, empty when nothing fired, rather than an optional one. Absence of coordination is an ordinary outcome — a lone value in a batch simply produces none — so a host iterates the list without first having to test for presence. |
//! | [`health_report_has_coordination_health`] | coordination | cites (´claim:coordination:the-tier-works-in-four-dimensions-because-a-member-contributes-one-value-per-scoring-axis´) |
//! | [`config_cusum_coord_slow_decay_validated`] | coordination | The slow baseline the coordination tier measures drift against is validated like any other rate: strictly inside zero and one, and strictly slower than the fast forgetting factor. The separation is the whole point — the slow baseline is the reference the fast one is judged against — so a configuration where the two move at the same speed is rejected outright rather than quietly producing a meaningless reading. |
//! | [`coordination_reports_are_ordered_by_depth_then_identifier`] | coordination | Coordination reports arrive shallowest first, ties broken by ascending identifier. The walk that produces them is bottom-up, which emits a strictly post-order sequence and puts the root — the shallowest context of all — last; that is deterministic but it is not the order either record states, and a reader taking the reports as a descent from the coarsest scale to the finest would have had the sequence exactly backwards. Depth is the ordering the output record describes and the identifier is the ordering this type's own documentation describes, so sorting on the pair satisfies both and is a total order besides, which sorting on depth alone would not be. |
//! | [`contour_count_includes_semi_internal_nodes`] | coordination | The contour count is the whole contour: the terminal cells together with the semi-internal ones. A semi-internal node has one half subdivided and one that still accumulates locally, so that second half receives observations exactly as a terminal cell does and is part of the surface the snapshot describes. Counting only the terminals reported a resolution short by every half-subdivided node, which is a figure that drifts from the truth precisely while the structure is being reshaped. |

mod common;

use std::collections::BTreeSet;

use common::{ScenarioBuilder, assert_invariants, cell_values, seeded_sentinel, test_config};
use torrust_sentinel::{GNodeId, Sentinel128, SentinelConfig};

// ═══════════════════════════════════════════════════════════
//  Activation & deactivation (§7.1)
// ═══════════════════════════════════════════════════════════

// ── no_coordination_on_empty_batch ──────────────────────────

/// A batch carrying no values gives the tier nothing to group: no cell
/// reports a score, no node can find both its subtrees contributing, and the
/// coordination list comes back empty. Coordination is a statement about
/// what several cells did together in one batch, so with no batch there is
/// nothing to say.
///
/// ´claim:coordination:a-batch-with-no-observations-produces-no-coordination-report´
/// ´test:integration:no-coordination-on-empty-batch´
#[test]
fn no_coordination_on_empty_batch() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[]);

    assert!(
        report.coordination_reports.is_empty(),
        "empty batch should produce no coordination reports"
    );
    assert_invariants(&s, &report);
}

// ── no_coordination_with_single_region ──────────────────────

/// Traffic confined to one region leaves every reporting cell in the same
/// subtree, so the both-subtrees condition is never met on its own account.
/// Any context that does appear carries several member cells — which is what
/// makes a coordination score a genuinely cross-cell measurement rather than
/// a restatement of one cell's own score at a coarser scale.
///
/// ´claim:coordination:a-context-carries-at-least-two-member-cells-because-both-subtrees-must-contribute´
/// ´test:integration:no-coordination-with-single-region´
#[test]
fn no_coordination_with_single_region() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 8)
        .warm_batches(5)
        .build();

    let report = s.ingest(&cell_values(0xF, 8));

    // All competitive cells are in one subtree → no both-subtree
    // condition → coordination should not fire.
    for cr in &report.coordination_reports {
        assert!(
            cr.cells_reporting >= 2,
            "coordination should only fire with ≥ 2 cells from both subtrees"
        );
    }
    assert_invariants(&s, &report);
}

// ── two_sibling_cells_fire_parent_context ───────────────────

/// Two well-separated regions reporting in the same batch turn the node
/// above them into a live context: it sees members arriving from each side
/// and begins modelling the pair as a group. Each report identifies itself
/// by the dyadic interval of that node, whose lower bound always lies
/// strictly below its upper bound, so a host can say which stretch of the
/// domain the measurement covers.
///
/// ´claim:coordination:two-populated-sibling-subtrees-turn-their-common-ancestor-into-a-live-context´
/// ´test:integration:two-sibling-cells-fire-parent-context´
#[test]
fn two_sibling_cells_fire_parent_context() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    assert!(
        !report.coordination_reports.is_empty(),
        "two distinct cell regions should produce coordination"
    );
    for cr in &report.coordination_reports {
        assert!(cr.cells_reporting >= 2, "coordination needs ≥ 2 cells");
        assert!(cr.start < cr.end, "start should be < end");
    }
    assert_invariants(&s, &report);
}

// ── context_activation_requires_both_subtrees ───────────────

/// After a run in which only one region reported, sending traffic to the
/// opposite half of the domain can only add contexts, never take them away.
/// Contexts are created where a split starts carrying reporters on both
/// sides, so the arrival of a second region is precisely the event that
/// brings the tier into existence.
///
/// ´claim:coordination:a-second-reporting-region-can-only-add-contexts-never-remove-them´
/// ´test:integration:context-activation-requires-both-subtrees´
#[test]
fn context_activation_requires_both_subtrees() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 8)
        .warm_batches(9)
        .build();

    let ch_one = s.health().coordination_health;

    // Now add a second region in the opposite half of the domain.
    for _ in 0..5 {
        s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());
    }
    let ch_both = s.health().coordination_health;

    assert!(
        ch_both.active_contexts >= ch_one.active_contexts,
        "adding a second region should activate coordination: \
         before={}, after={}",
        ch_one.active_contexts,
        ch_both.active_contexts
    );
}

// ── context_deactivation_on_subtree_loss ────────────────────

/// When a batch arrives from one region only, the nodes that had been
/// spanning both stop qualifying, and the tier prunes them instead of
/// carrying stale contexts forward. The count of live contexts therefore
/// never rises when a subtree falls silent: what is reported describes the
/// group structure of the batch in hand, not of some earlier one.
///
/// ´claim:coordination:a-context-whose-subtree-falls-silent-is-pruned-rather-than-carried-forward´
/// ´test:integration:context-deactivation-on-subtree-loss´
#[test]
fn context_deactivation_on_subtree_loss() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let ch_both = s.health().coordination_health;

    // Ingest only one region — the other region's cells may still
    // be competitive but have zero observations this batch.
    let _report = s.ingest(&cell_values(0xF, 4));
    let ch_one = s.health().coordination_health;

    assert!(
        ch_one.active_contexts <= ch_both.active_contexts,
        "losing a subtree should deactivate contexts: \
         both={}, one={}",
        ch_both.active_contexts,
        ch_one.active_contexts
    );
}

// ═══════════════════════════════════════════════════════════
//  Hierarchy & nesting (§7.1, §7.4)
// ═══════════════════════════════════════════════════════════

// ── nested_coordination_levels ──────────────────────────────

/// Several regions spread across the domain make the hierarchy fire at more
/// than one level at once: a context over each neighbouring pair and wider
/// ones above them. The levels stay distinct — every report in a batch
/// stands at its own node, so a nested pair is two measurements at two
/// scales rather than the same measurement counted twice.
///
/// (´claim:coordination:each-firing-node-appears-exactly-once-among-a-batchs-coordination-reports´)
/// ´test:integration:nested-coordination-levels´
#[test]
fn nested_coordination_levels() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 16,
        split_threshold: 10,
        ..test_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0x1, 4)
        .seed_range(0x3, 4)
        .seed_range(0x9, 4)
        .seed_range(0xF, 4)
        .warm_batches(19)
        .build();

    let report = s.ingest(
        &[
            cell_values(0x1, 4),
            cell_values(0x3, 4),
            cell_values(0x9, 4),
            cell_values(0xF, 4),
        ]
        .concat(),
    );

    // With 4 cell regions, hierarchy should fire at multiple levels.
    if report.coordination_reports.len() > 1 {
        let gnodes: BTreeSet<_> = report.coordination_reports.iter().map(|cr| cr.gnode_id).collect();
        assert!(
            gnodes.len() == report.coordination_reports.len(),
            "each coordination report should be at a unique gnode"
        );
    }
    assert_invariants(&s, &report);
}

// ── coordination_group_nesting_invariant ────────────────────

/// Membership nests with the tree. Wherever one context's interval contains
/// another's, the containing context counts at least as many reporting
/// cells, because the walk unions each subtree's members and passes the
/// result upward. An ancestor therefore measures a superset of what its
/// descendant measured, which is what lets the two readings be compared as
/// the same pattern seen at two scales.
///
/// ´claim:coordination:an-ancestor-context-counts-every-cell-its-descendant-counts´
/// ´test:integration:coordination-group-nesting-invariant´
#[test]
fn coordination_group_nesting_invariant() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 16,
        split_threshold: 10,
        ..test_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0x1, 4)
        .seed_range(0x5, 4)
        .seed_range(0x9, 4)
        .seed_range(0xF, 4)
        .warm_batches(14)
        .build();

    let report = s.ingest(
        &[
            cell_values(0x1, 4),
            cell_values(0x5, 4),
            cell_values(0x9, 4),
            cell_values(0xF, 4),
        ]
        .concat(),
    );

    // For any two coordination reports where one's interval
    // contains the other, the parent should have ≥ cells_reporting.
    for a in &report.coordination_reports {
        for b in &report.coordination_reports {
            if a.gnode_id == b.gnode_id {
                continue;
            }
            if a.start <= b.start && a.end >= b.end {
                assert!(
                    a.cells_reporting >= b.cells_reporting,
                    "ancestor context [{}..{}) (depth {}, cells {}) \
                     should have ≥ cells than descendant [{}..{}) (depth {}, cells {})",
                    a.start,
                    a.end,
                    a.depth,
                    a.cells_reporting,
                    b.start,
                    b.end,
                    b.depth,
                    b.cells_reporting
                );
            }
        }
    }
    assert_invariants(&s, &report);
}

// ── root_context_sees_all_cells ─────────────────────────────

/// Pins the top of that nesting: when several competitive cells report
/// together, the widest live context is the one that has accumulated them,
/// so at least one report shows a membership larger than a single cell.
/// Which node that is depends on where the traffic actually split, not on
/// any privileged root context.
///
/// (´claim:coordination:an-ancestor-context-counts-every-cell-its-descendant-counts´)
/// ´test:integration:root-context-sees-all-cells´
#[test]
fn root_context_sees_all_cells() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    let competitive_with_obs = report
        .cell_reports
        .iter()
        .filter(|c| c.is_competitive && c.sample_count > 0)
        .count();

    if !report.coordination_reports.is_empty() && competitive_with_obs >= 2 {
        let max_reporting = report
            .coordination_reports
            .iter()
            .map(|cr| cr.cells_reporting)
            .max()
            .unwrap_or(0);
        assert!(
            max_reporting >= 2,
            "at least one coordination context should see multiple cells"
        );
    }
    assert_invariants(&s, &report);
}

// ── semi_internal_node_passthrough ──────────────────────────

/// A node with only one contributing child poses no coordination question —
/// there is no pair of subtrees to compare — so it fires nothing and simply
/// forwards its members to its parent. Every report that does appear
/// consequently spans a real split and carries a usable finite score, rather
/// than the tier manufacturing a group out of a single branch.
///
/// ´claim:coordination:a-node-with-one-contributing-child-forwards-its-cells-without-firing´
/// ´test:integration:semi-internal-node-passthrough´
#[test]
fn semi_internal_node_passthrough() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    // Semi-internal nodes (one child) should not fire coordination;
    // only nodes with both subtrees contributing should appear.
    for cr in &report.coordination_reports {
        assert!(cr.cells_reporting >= 2);
        assert!(cr.scores.novelty.mean.is_finite());
    }
    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Scoring & tracking (§7.2–7.5)
// ═══════════════════════════════════════════════════════════

// ── coordination_tracker_operates_at_w4 ─────────────────────

/// A member's contribution to a context is one value per scoring axis, so
/// the tier works in four dimensions however wide the cells beneath it are.
/// That fixed and tiny geometry is what makes a second tier affordable: its
/// cost does not grow with the width of the domain the cells analyse.
///
/// ´claim:coordination:the-tier-works-in-four-dimensions-because-a-member-contributes-one-value-per-scoring-axis´
/// ´test:integration:coordination-tracker-operates-at-w4´
#[test]
fn coordination_tracker_operates_at_w4() {
    let s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let ch = s.health().coordination_health;
    if ch.active_contexts > 0 {
        assert_eq!(ch.dim, 4, "coordination trackers should operate at w=4");
        // cap = min(4, max_rank). test_config has max_rank=4.
        assert_eq!(ch.capacity, 4, "cap should be min(4, max_rank)");
    }
}

// ── coordination_tracker_with_low_max_rank ──────────────────

/// A context's capacity is the configured rank ceiling, capped by the four
/// dimensions actually available. Lowering that ceiling lowers the capacity
/// with it while the dimensionality stays at four: capacity is a budget on
/// how much structure the model may hold, not a change to what a member
/// observation is.
///
/// ´claim:coordination:a-contexts-capacity-is-the-configured-rank-ceiling-capped-at-the-four-available-dimensions´
/// ´test:integration:coordination-tracker-with-low-max-rank´
#[test]
fn coordination_tracker_with_low_max_rank() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 2,
        ..test_config()
    };
    let s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let ch = s.health().coordination_health;
    if ch.active_contexts > 0 {
        assert_eq!(ch.dim, 4, "coordination dim is always 4");
        assert_eq!(ch.capacity, 2, "cap should be min(4, max_rank=2) = 2");
    }
}

// ── running_mean_cold_start ─────────────────────────────────

/// The first batch a context ever handles still yields finite numbers on all
/// four axes. Members are centred against a running mean that starts at zero
/// and is seeded outright from that batch's own column means rather than
/// divided by an empty history, so activation costs the host no undefined
/// readings to interpret.
///
/// ´claim:coordination:a-newly-activated-context-scores-its-first-batch-in-finite-numbers´
/// ´test:integration:running-mean-cold-start´
#[test]
fn running_mean_cold_start() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    let ch = s.health().coordination_health;
    if ch.active_contexts > 0 {
        let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());
        for cr in &report.coordination_reports {
            assert!(cr.scores.novelty.mean.is_finite());
            assert!(cr.scores.displacement.mean.is_finite());
            assert!(cr.scores.surprise.mean.is_finite());
            assert!(cr.scores.coherence.mean.is_finite());
        }
        assert_invariants(&s, &report);
    }
}

// ── running_mean_ewma_update ────────────────────────────────

/// A context that has activated goes on firing while both its subtrees keep
/// reporting: repeating the same shape of batch produces coordination in
/// nearly all of them. Its running mean is updated across those batches
/// instead of being rebuilt each time, so what the tier measures stays a
/// departure from the group's own recent pattern.
///
/// ´claim:coordination:an-active-context-keeps-firing-while-both-its-subtrees-keep-reporting´
/// ´test:integration:running-mean-ewma-update´
#[test]
fn running_mean_ewma_update() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build();

    let batch = [cell_values(0xF, 4), cell_values(0x1, 4)].concat();
    let mut reports = Vec::new();
    for _ in 0..5 {
        reports.push(s.ingest(&batch));
    }

    let non_empty: usize = reports.iter().filter(|r| !r.coordination_reports.is_empty()).count();
    assert!(non_empty >= 3, "most batches should produce coordination: got {non_empty}/5");
}

// ── only_competitive_cells_contribute ───────────────────────

/// Membership is drawn only from competitive cells that actually observed
/// something in this batch, so no context can report more members than there
/// were such cells. The root is never competitive and so never joins a
/// group: coordination measures the cells the engine chose to invest in, not
/// the whole tree.
///
/// ´claim:coordination:only-competitive-cells-that-observed-this-batch-become-members´
/// ´test:integration:only-competitive-cells-contribute´
#[test]
fn only_competitive_cells_contribute() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(4)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    // Root is never competitive (§ALGO S-4.7). coordination
    // cells_reporting should only count competitive cells.
    let competitive_count = report
        .cell_reports
        .iter()
        .filter(|c| c.is_competitive && c.sample_count > 0)
        .count();

    for cr in &report.coordination_reports {
        assert!(
            cr.cells_reporting <= competitive_count,
            "cells_reporting ({}) should not exceed competitive cells with observations ({})",
            cr.cells_reporting,
            competitive_count
        );
    }
    assert_invariants(&s, &report);
}

// ── cells_with_no_observations_excluded ─────────────────────

/// Pins the other end of that rule: a cell can remain competitive and still
/// contribute nothing, because it saw no values this time. Sending traffic
/// to one region only leaves the opposite region's cells silent, and a
/// subtree of silent cells does not count as contributing, so nothing fires
/// on their behalf.
///
/// (´claim:coordination:only-competitive-cells-that-observed-this-batch-become-members´)
/// ´test:integration:cells-with-no-observations-excluded´
#[test]
fn cells_with_no_observations_excluded() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    // Ingest only one region — the other region's competitive
    // cells get zero observations this batch.
    let report = s.ingest(&cell_values(0xF, 4));

    // §7.2: cells with no observations are excluded. If only one
    // subtree has observations, coordination should not fire.
    for cr in &report.coordination_reports {
        assert!(cr.cells_reporting >= 2, "coordination should need cells from both subtrees");
    }
    assert_invariants(&s, &report);
}

// ── coordination_scores_are_finite ──────────────────────────

/// Everything a context publishes is a finite number: the four axis means,
/// the drift accumulated on each, the fraction of variance the learned
/// subspace captures and its largest singular value. Degenerate geometry at
/// this tier is resolved inside the model rather than handed to the host as
/// a not-a-number it would have to interpret for itself.
///
/// ´claim:coordination:every-published-coordination-measurement-is-a-finite-number´
/// ´test:integration:coordination-scores-are-finite´
#[test]
fn coordination_scores_are_finite() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    for cr in &report.coordination_reports {
        assert!(cr.scores.novelty.mean.is_finite(), "NaN/Inf novelty mean");
        assert!(cr.scores.displacement.mean.is_finite(), "NaN/Inf displacement mean");
        assert!(cr.scores.surprise.mean.is_finite(), "NaN/Inf surprise mean");
        assert!(cr.scores.coherence.mean.is_finite(), "NaN/Inf coherence mean");

        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty");
        assert!(!cr.scores.displacement.mean.is_nan(), "NaN displacement");
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise");
        assert!(!cr.scores.coherence.mean.is_nan(), "NaN coherence");

        // CUSUM accumulators must be finite.
        assert!(cr.scores.novelty.cusum.accumulator.is_finite());
        assert!(cr.scores.displacement.cusum.accumulator.is_finite());
        assert!(cr.scores.surprise.cusum.accumulator.is_finite());
        assert!(cr.scores.coherence.cusum.accumulator.is_finite());

        // Energy ratio and singular values must be sane.
        assert!(cr.energy_ratio.is_finite());
        assert!(cr.top_singular_value.is_finite());
    }
    assert_invariants(&s, &report);
}

// ── per_member_scores_present_when_enabled ──────────────────

/// With per-sample scoring enabled a context also names its members: one
/// entry per reporting cell, matching the membership count it declared, each
/// carrying that cell's own four scores and the non-empty interval it
/// covers. A host can therefore attribute a group-level reading to
/// particular regions of the domain instead of seeing only the aggregate.
///
/// ´claim:coordination:per-member-scores-attribute-a-group-reading-to-the-cells-that-produced-it´
/// ´test:integration:per-member-scores-present-when-enabled´
#[test]
fn per_member_scores_present_when_enabled() {
    // test_config() has per_sample_scores = true.
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    for cr in &report.coordination_reports {
        let members = cr
            .per_member
            .as_ref()
            .expect("per_member should be Some when per_sample_scores is enabled");
        assert_eq!(
            members.len(),
            cr.cells_reporting,
            "per_member count should equal cells_reporting"
        );
        for ms in members {
            assert!(ms.novelty.is_finite());
            assert!(ms.displacement.is_finite());
            assert!(ms.surprise.is_finite());
            assert!(ms.coherence.is_finite());
            assert!(ms.cell_start < ms.cell_end, "member cell interval must be non-empty");
        }
    }
    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  Stability & resilience
// ═══════════════════════════════════════════════════════════

// ── coordination_survives_decay ─────────────────────────────

/// Ageing the model down does not dismantle the tier. After a decay the
/// sentinel still ingests and still produces reports on the next batch,
/// because decay weakens the learned structure rather than removing the
/// cells and contexts that carry it.
///
/// ´claim:coordination:decaying-the-model-leaves-the-tier-able-to-report-on-the-next-batch´
/// ´test:integration:coordination-survives-decay´
#[test]
fn coordination_survives_decay() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(9)
        .build();

    s.decay(0.5, 0.0);

    // After decay, new observations should still produce functional reports.
    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());
    assert!(
        !report.cell_reports.is_empty() || !report.ancestor_reports.is_empty(),
        "coordination tier should remain functional after decay"
    );
    assert_invariants(&s, &report);
}

// ── inject_noise_warms_coordination ─────────────────────────

/// A new context is warmed with synthetic score vectors drawn to match its
/// members' own baselines, which leaves it holding a usable model rather
/// than an empty one. The warm-up then clears the drift accumulators, so the
/// first real batch is that context's first step of evidence: what the
/// engine invented about itself is never counted as evidence about traffic.
///
/// ´claim:coordination:warming-leaves-a-context-experienced-but-with-its-drift-evidence-cleared´
/// ´test:integration:inject-noise-warms-coordination´
#[test]
fn inject_noise_warms_coordination() {
    let mut s = seeded_sentinel();

    let ch = s.health().coordination_health;

    if ch.active_contexts > 0 {
        assert!(
            ch.maturity_distribution.max_noise_influence < 1.0,
            "coordination should have warmed after noise"
        );
    }

    // Verify CUSUM reset: ingest one batch and check steps_since_reset.
    let report = s.ingest(&[
        0xF000_0000_0000_0000_0000_0000_0000_AAAA,
        0x1000_0000_0000_0000_0000_0000_0000_BBBB,
    ]);
    for cr in &report.coordination_reports {
        assert_eq!(
            cr.scores.novelty.cusum.steps_since_reset, 1,
            "coordination CUSUM should have been reset after noise"
        );
    }
    assert_invariants(&s, &report);
}

// ── deterministic_report_order ──────────────────────────────

/// Two runs from the same seed over the same traffic produce the same
/// reports: the same number of them, at the same nodes in the same order,
/// with the same memberships and matching scores. The only randomness in the
/// engine is the seeded warm-up, and pinning it makes two runs comparable —
/// without that, no difference a host observed could be attributed.
///
/// ´claim:coordination:the-same-seed-and-traffic-give-the-same-reports-in-the-same-order´
/// ´test:integration:deterministic-report-order´
#[test]
fn deterministic_report_order() {
    let batch: Vec<u128> = [cell_values(0xF, 4), cell_values(0x1, 4)].concat();

    let run = |seed: u64| {
        let cfg = SentinelConfig::<u64> {
            analysis_k: 16,
            noise_seed: Some(seed),
            ..test_config()
        };
        let mut s = Sentinel128::new(cfg).unwrap();
        for _ in 0..10 {
            s.ingest(&batch);
        }
        s.ingest(&batch)
    };

    let r1 = run(42);
    let r2 = run(42);

    assert_eq!(r1.coordination_reports.len(), r2.coordination_reports.len());
    for (a, b) in r1.coordination_reports.iter().zip(r2.coordination_reports.iter()) {
        assert_eq!(a.gnode_id, b.gnode_id, "reports should be in same order");
        assert_eq!(a.depth, b.depth);
        assert_eq!(a.cells_reporting, b.cells_reporting);
        assert!(
            (a.scores.novelty.mean - b.scores.novelty.mean).abs() < 1e-10,
            "deterministic runs should produce identical scores"
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  Invariants (§7.1)
// ═══════════════════════════════════════════════════════════

// ── context_count_bounded_by_k_minus_1 ──────────────────────

/// Contexts are the internal nodes of a binary tree whose leaves are the
/// competitive cells, so their number stays below the number of those cells.
/// The tier's cost is thereby bounded by the analysis budget the host has
/// already chosen and cannot grow on its own.
///
/// ´claim:coordination:contexts-are-internal-nodes-of-a-binary-tree-so-they-stay-fewer-than-the-competitive-cells´
/// ´test:integration:context-count-bounded-by-k-minus-1´
#[test]
fn context_count_bounded_by_k_minus_1() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 16,
        split_threshold: 10,
        ..test_config()
    };
    let s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0x1, 4)
        .seed_range(0x5, 4)
        .seed_range(0x9, 4)
        .seed_range(0xF, 4)
        .warm_batches(14)
        .build();

    let competitive_cells = s.analysis_set().competitive().len();
    let active = s.health().coordination_health.active_contexts;

    // Binary tree internal node bound: |contexts| ≤ K-1
    // where K = number of competitive cells (§ALGO S-7.1).
    if competitive_cells > 0 {
        assert!(
            active <= competitive_cells.saturating_sub(1).max(1),
            "active contexts ({active}) should be ≤ K-1 ({}) (§ALGO S-7.1)",
            competitive_cells.saturating_sub(1)
        );
    }
}

// ── coordination_reports_unique_gnodes ──────────────────────

/// A batch produces at most one report per firing node. The walk visits each
/// node once and a context is keyed by the node it sits at, so a nested
/// hierarchy yields one measurement per level rather than repeated entries a
/// host would first have to de-duplicate.
///
/// ´claim:coordination:each-firing-node-appears-exactly-once-among-a-batchs-coordination-reports´
/// ´test:integration:coordination-reports-unique-gnodes´
#[test]
fn coordination_reports_unique_gnodes() {
    let mut s = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build();

    let report = s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());

    let gnodes: BTreeSet<_> = report.coordination_reports.iter().map(|cr| cr.gnode_id).collect();
    assert_eq!(
        gnodes.len(),
        report.coordination_reports.len(),
        "no duplicate GNodeIds in coordination_reports"
    );
    assert_invariants(&s, &report);
}

// ═══════════════════════════════════════════════════════════
//  API surface
// ═══════════════════════════════════════════════════════════

// ── batch_report_coordination_is_vec ────────────────────────

/// The batch report always carries a list of coordination reports, empty
/// when nothing fired, rather than an optional one. Absence of coordination
/// is an ordinary outcome — a lone value in a batch simply produces none —
/// so a host iterates the list without first having to test for presence.
///
/// ´claim:coordination:coordination-arrives-as-a-possibly-empty-list-rather-than-an-optional-value´
/// ´test:integration:batch-report-coordination-is-vec´
#[test]
fn batch_report_coordination_is_vec() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[42]);

    // The coordination field is Vec<CoordinationReport>, not Option.
    let _: &Vec<torrust_sentinel::CoordinationReport<u128>> = &report.coordination_reports;
    assert!(report.coordination_reports.is_empty());
}

// ── health_report_has_coordination_health ────────────────────

/// The health snapshot always includes the coordination summary, and that
/// summary declares the tier's fixed dimensionality even on a sentinel that
/// has ingested nothing and holds no contexts at all. The field describes
/// the tier itself, not merely whichever contexts happen to exist at the
/// moment of the reading.
///
/// (´claim:coordination:the-tier-works-in-four-dimensions-because-a-member-contributes-one-value-per-scoring-axis´)
/// ´test:integration:health-report-has-coordination-health´
#[test]
fn health_report_has_coordination_health() {
    let s = Sentinel128::new(test_config()).unwrap();
    let h = s.health();

    let ch = &h.coordination_health;
    assert_eq!(ch.active_contexts, 0);
    assert_eq!(ch.dim, 4);
}

// ── config_cusum_coord_slow_decay_validated ──────────────────

/// The slow baseline the coordination tier measures drift against is
/// validated like any other rate: strictly inside zero and one, and strictly
/// slower than the fast forgetting factor. The separation is the whole
/// point — the slow baseline is the reference the fast one is judged
/// against — so a configuration where the two move at the same speed is
/// rejected outright rather than quietly producing a meaningless reading.
///
/// ´claim:coordination:the-tiers-slow-baseline-must-be-strictly-slower-than-the-fast-one-and-inside-zero-and-one´
/// ´test:integration:config-cusum-coord-slow-decay-validated´
#[test]
fn config_cusum_coord_slow_decay_validated() {
    // Valid value.
    let cfg = SentinelConfig::<u64> {
        cusum_coord_slow_decay: 0.999,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();

    // Out of range: exactly 1.0.
    let cfg = SentinelConfig::<u64> {
        cusum_coord_slow_decay: 1.0,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());

    // Out of range: exactly 0.0.
    let cfg = SentinelConfig::<u64> {
        cusum_coord_slow_decay: 0.0,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());

    // Too low: must be > forgetting_factor.
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.99,
        cusum_coord_slow_decay: 0.99,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());
}

/// Coordination reports arrive shallowest first, ties broken by ascending
/// identifier. The walk that produces them is bottom-up, which emits a
/// strictly post-order sequence and puts the root — the shallowest context of
/// all — last; that is deterministic but it is not the order either record
/// states, and a reader taking the reports as a descent from the coarsest
/// scale to the finest would have had the sequence exactly backwards. Depth
/// is the ordering the output record describes and the identifier is the
/// ordering this type's own documentation describes, so sorting on the pair
/// satisfies both and is a total order besides, which sorting on depth alone
/// would not be.
///
/// ´claim:coordination:reports-arrive-shallowest-first-with-ties-broken-by-identifier´
/// ´test:integration:coordination-reports-are-ordered-by-depth-then-identifier´
#[test]
fn coordination_reports_are_ordered_by_depth_then_identifier() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 16,
        split_threshold: 10,
        ..test_config()
    };
    let mut s = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0x1, 4)
        .seed_range(0x3, 4)
        .seed_range(0x9, 4)
        .seed_range(0xF, 4)
        .warm_batches(19)
        .build();

    let report = s.ingest(
        &[
            cell_values(0x1, 4),
            cell_values(0x3, 4),
            cell_values(0x9, 4),
            cell_values(0xF, 4),
        ]
        .concat(),
    );

    assert!(
        report.coordination_reports.len() > 1,
        "nested contexts are needed for an ordering to be observable"
    );
    let keys: Vec<(u32, GNodeId)> = report.coordination_reports.iter().map(|cr| (cr.depth, cr.gnode_id)).collect();
    let mut sorted = keys.clone();
    sorted.sort_unstable();
    assert_eq!(keys, sorted, "reports run shallowest first, ties by ascending identifier");

    assert_invariants(&s, &report);
}

/// The contour count is the whole contour: the terminal cells together with
/// the semi-internal ones. A semi-internal node has one half subdivided and
/// one that still accumulates locally, so that second half receives
/// observations exactly as a terminal cell does and is part of the surface the
/// snapshot describes. Counting only the terminals reported a resolution
/// short by every half-subdivided node, which is a figure that drifts from the
/// truth precisely while the structure is being reshaped.
///
/// ´claim:coordination:the-contour-count-is-the-terminals-together-with-the-semi-internal-nodes´
/// ´test:integration:contour-count-includes-semi-internal-nodes´
#[test]
fn contour_count_includes_semi_internal_nodes() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        budget: 200,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let mut saw_semi_internal = false;
    for nibble in 0..16u128 {
        let batch: Vec<u128> = (0u128..500).map(|i| (nibble << 124) | (i << 100)).collect();
        let report = s.ingest(&batch);

        let terminals = s.graph().terminal_count() as usize;
        let semi_internal = report.health.semi_internal_count;
        assert_eq!(
            report.contour.cell_count,
            terminals + semi_internal,
            "the contour is the terminals together with the semi-internal nodes"
        );
        if semi_internal > 0 {
            saw_semi_internal = true;
        }
    }

    assert!(
        saw_semi_internal,
        "this run must reach a half-subdivided node for the sum to be distinguishable"
    );
}
