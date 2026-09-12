// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_graph_produces_root_only`] | selection | A graph that has observed nothing has nothing worth competing over, so no cell is competitively selected and the set holds a single entry. Selection is driven by accumulated importance, and where there is none the sentinel invests in nothing rather than picking arbitrarily among equals. |
//! | [`root_always_present`] | selection | The root is in the full set unconditionally — here even when nothing has been observed and no cell competed at all. Ancestor closure walks upward from each selected cell, so the root's presence is what guarantees every such walk terminates at a cell that has a model rather than running off the top of the tree. |
//! | [`root_is_never_competitive`] | selection | However much traffic the graph has seen, and however generous the budget, the root is never competitively selected. It accumulates every observation by construction and would win any importance contest automatically, crowding out the cells whose behaviour is actually informative. Its place in the set is structural, and it is held apart from the cells that earned theirs. |
//! | [`competitive_set_respects_k`] | selection | The competitive set never exceeds the budget it was asked for, however many cells would qualify on their merits. The budget is what bounds the sentinel's modelling cost, so it is a ceiling rather than a target that a sufficiently busy graph could push past. |
//! | [`budget_of_one_selects_one_cell`] | selection | A budget of one selects a cell rather than nothing, because the root leaves the field before the cut rather than after it. Taken the other way round the root wins the only slot and is then discarded for being the root, so the smallest budget the configuration admits selects nothing at all however busy the graph is. The root wins that contest on a total it accumulated before its first split and stopped adding to at the split, while its children start from zero — so the emptiness persists until a child's own total passes a figure that is no longer growing, and every slot spent on the root is a slot spent on an entry that cannot be selected. |
//! | [`a_larger_budget_fills_every_slot_with_selectable_cells`] | selection | cites (´claim:selection:the-root-leaves-the-field-before-the-cut-so-every-slot-goes-to-a-selectable-cell´) |
//! | [`k_zero_yields_no_competitive_entries`] | selection | cites (´claim:selection:the-competitive-set-never-exceeds-the-budget-it-was-asked-for´) |
//! | [`depth_cutoff_zero_excludes_all_non_root`] | selection | The depth cutoff bounds the V-depth of every competitive cell: with the cutoff at zero, no selected cell sits deeper than zero however busy the graph. The bound is enforced while the tree is being walked rather than by discarding candidates afterwards, so the cutoff limits the work done as well as the cells returned. |
//! | [`overdeep_float_cells_are_excluded`] | selection | A floating-coordinate graph can contain cells deeper than its model width because splitting is gated by V-tree depth. Their modeled suffix has no remaining width, so selection excludes them rather than overflowing the width subtraction or admitting them as enormous candidates. |
//! | [`tie_breaking_is_deterministic`] | selection | Recomputing over an unchanged graph selects the same cells in the same order. Nothing in selection depends on iteration order, hashing, or timing, so two sentinels fed identical observations reach identical analysis sets — the foundation the reproducibility of every downstream score rests on. |
//! | [`competitive_ordering_by_importance_then_start`] | selection | Competitive cells come back in a total order: importance descending, and among cells of equal importance, interval start ascending. Ties are therefore settled by a property of the coordinate domain rather than by whatever order the tree walk happened to produce, which is what makes the ordering reproducible and not merely stable within one run. |
//! | [`internal_nodes_eligible_for_competitive_set`] | selection | A graph driven hard enough to split still yields competitive cells. The selector ranks by V-Tree importance alone and applies no filter on G-tree state, so a cell that has since become internal keeps its V-Tree position and remains eligible. Splitting refines the spatial structure; it does not silently remove cells from consideration. |
//! | [`full_set_is_superset_of_competitive`] | selection | Every competitive cell is also in the full set, and the full set is never the smaller of the two. Winning the competition confers membership rather than replacing it, so a cell can be looked up by either question without the two answers contradicting each other. |
//! | [`contains_returns_false_for_absent_node`] | selection | Membership is decided by what the set actually holds, not by whether a handle looks plausible: a fabricated handle the graph never allocated is simply absent. A caller holding a stale or invented cell identifier gets a negative answer rather than an accidental match on a reused slot. |
//! | [`is_competitive_true_for_selected_entries`] | selection | The competitiveness predicate agrees with the competitive list: every cell the set lists as competitive answers to that question as well. Asking by handle and reading the list are two views of one fact, so the two ways a caller can learn a cell's standing cannot disagree. |
//! | [`is_competitive_false_for_ancestor_only`] | selection | cites (´claim:selection:the-competitiveness-predicate-agrees-with-the-competitive-list´) |
//! | [`summary_empty_graph`] | selection | A summary of a set with nothing selected reports zeroes throughout — sizes, depth span, importance span and V-depth span alike — rather than omitting the ranges or filling them with sentinels. The full size is one, because the root is there. A host parsing summaries gets the same shape whether or not anything was selected. |
//! | [`summary_with_competitive_cells`] | selection | A summary counts the cells that competed and the cells the closure added as separate figures, and on a populated graph the full count strictly exceeds the competitive one. The cost of ancestry is therefore visible: a host can see how much modelling it is paying for beyond the cells it actually chose to invest in. |
//! | [`summary_online_keeps_the_investment_count_whole`] | selection | The producing sets shrink to whatever is online, but the investment does not: a cell still being warmed has been paid for and has produced nothing yet, and that gap is the whole difference between the two readings. A summary taken over the online cells therefore filters the producing count and leaves the investment count whole, so a host watching a warm-up sees what it has committed to as well as what is answering. |
//! | [`summary_depth_range_includes_root`] | selection | cites (´claim:selection:the-root-is-always-in-the-full-set-so-every-ancestor-chain-terminates´) |
//! | [`summary_importance_range_positive`] | selection | Where cells were selected at all, the least important of them still carries importance above zero, and the reported span runs the right way round. A cell can only win the competition on accumulated observation, so nothing with no traffic behind it appears in the summary as though it had been chosen. |
//! | [`summary_v_depth_range_nonzero`] | selection | cites (´claim:selection:the-root-is-never-competitive-however-important-it-is´) |

//! Crate-level tests for **`AnalysisSet`** — which cells the sentinel spends
//! its modelling effort on.
//!
//! Selection happens in two movements. First a competition: V-Tree entries
//! within the depth cutoff and wide enough to support a tracker are ranked by
//! importance, and the top few win. Then a closure: every winner's G-tree
//! ancestors are pulled in whether or not they competed, so that each selected
//! cell has an unbroken chain of models back to the root.
//!
//! The two movements make two kinds of membership, and the accessors keep
//! them apart. A cell in the full set has a tracker; a cell in the
//! competitive set additionally earned its place. The root belongs to the
//! first and never to the second — it is present unconditionally so that
//! every ancestor chain terminates, which is a structural obligation rather
//! than a claim that the root deserved investment.
//!
//! Ranking is by V-Tree importance alone. The selector applies no filter on
//! G-tree state, so an internal cell is as eligible as a terminal one, and
//! ties are broken by interval start so that recomputing over an unchanged
//! graph returns the same cells in the same order.

use std::collections::BTreeSet;

use torrust_mudlark::{Config as GvConfig, GNodeId, GvGraph};

use crate::analysis_set::*;

// ── Helpers ─────────────────────────────────────────────

fn test_graph() -> GvGraph<u128, u64, 128> {
    let cfg = GvConfig {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100_000),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    GvGraph::new(cfg)
}

/// Feed uniform traffic to force splits.
fn populated_graph() -> GvGraph<u128, u64, 128> {
    let mut graph = test_graph();
    for i in 0u128..2000 {
        graph.observe(i * (u128::MAX / 2000), 1u64);
    }
    graph
}

// ── recompute() — construction ──────────────────────────

/// A graph that has observed nothing has nothing worth competing over, so no
/// cell is competitively selected and the set holds a single entry. Selection
/// is driven by accumulated importance, and where there is none the sentinel
/// invests in nothing rather than picking arbitrarily among equals.
///
/// ´claim:selection:an-unobserved-graph-selects-nothing-competitively´
/// ´test:crate:empty-graph-produces-root-only´
#[test]
fn empty_graph_produces_root_only() {
    let graph = test_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    assert_eq!(set.competitive_count(), 0);
    assert_eq!(set.total_count(), 1); // root only
}

/// The root is in the full set unconditionally — here even when nothing has
/// been observed and no cell competed at all. Ancestor closure walks upward
/// from each selected cell, so the root's presence is what guarantees every
/// such walk terminates at a cell that has a model rather than running off
/// the top of the tree.
///
/// ´claim:selection:the-root-is-always-in-the-full-set-so-every-ancestor-chain-terminates´
/// ´test:crate:root-always-present´
#[test]
fn root_always_present() {
    let graph = test_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    assert!(set.contains(graph.g_root()));
}

/// However much traffic the graph has seen, and however generous the budget,
/// the root is never competitively selected. It accumulates every observation
/// by construction and would win any importance contest automatically,
/// crowding out the cells whose behaviour is actually informative. Its place
/// in the set is structural, and it is held apart from the cells that earned
/// theirs.
///
/// ´claim:selection:the-root-is-never-competitive-however-important-it-is´
/// ´test:crate:root-is-never-competitive´
#[test]
fn root_is_never_competitive() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 100, 6);
    assert!(
        !set.is_competitive(graph.g_root()),
        "root must never appear in the competitive set (§ALGO S-4.7)",
    );
}

/// The competitive set never exceeds the budget it was asked for, however
/// many cells would qualify on their merits. The budget is what bounds the
/// sentinel's modelling cost, so it is a ceiling rather than a target that a
/// sufficiently busy graph could push past.
///
/// ´claim:selection:the-competitive-set-never-exceeds-the-budget-it-was-asked-for´
/// ´test:crate:competitive-set-respects-k´
#[test]
fn competitive_set_respects_k() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 2, 6);
    assert!(set.competitive_count() <= 2);
}

/// A budget of one selects a cell rather than nothing, because the root leaves
/// the field before the cut rather than after it. Taken the other way round the
/// root wins the only slot and is then discarded for being the root, so the
/// smallest budget the configuration admits selects nothing at all however busy
/// the graph is. The root wins that contest on a total it accumulated before
/// its first split and stopped adding to at the split, while its children start
/// from zero — so the emptiness persists until a child's own total passes a
/// figure that is no longer growing, and every slot spent on the root is a slot
/// spent on an entry that cannot be selected.
///
/// ´claim:selection:the-root-leaves-the-field-before-the-cut-so-every-slot-goes-to-a-selectable-cell´
/// ´test:crate:budget-of-one-selects-one-cell´
#[test]
fn budget_of_one_selects_one_cell() {
    let graph = populated_graph();
    let available = AnalysisSet::recompute(&graph, 1000, 6).competitive_count();
    assert!(available >= 1, "the populated graph must offer at least one candidate");

    let set = AnalysisSet::recompute(&graph, 1, 6);
    assert_eq!(set.competitive_count(), 1);
    assert!(!set.is_competitive(graph.g_root()));
}

/// The same rule at a budget the root could not have exhausted on its own: the
/// competitive set fills to the whole budget rather than to one less than it.
/// Removing the root after the cut would cost exactly one slot at every budget,
/// which is invisible at a large one and total at a budget of one.
///
/// (´claim:selection:the-root-leaves-the-field-before-the-cut-so-every-slot-goes-to-a-selectable-cell´)
/// ´test:crate:a-larger-budget-fills-every-slot-with-selectable-cells´
#[test]
fn a_larger_budget_fills_every_slot_with_selectable_cells() {
    let graph = populated_graph();
    let available = AnalysisSet::recompute(&graph, 1000, 6).competitive_count();
    assert!(available >= 3, "the populated graph must offer at least three candidates");

    let set = AnalysisSet::recompute(&graph, 3, 6);
    assert_eq!(set.competitive_count(), 3);
    assert!(!set.is_competitive(graph.g_root()));
}

/// A budget of zero is the boundary of the same ceiling: a well-populated
/// graph yields no competitive cells at all, while the root remains in the
/// full set. Selection can be turned off entirely without the structural
/// guarantee going with it.
///
/// (´claim:selection:the-competitive-set-never-exceeds-the-budget-it-was-asked-for´)
/// ´test:crate:k-zero-yields-no-competitive-entries´
#[test]
fn k_zero_yields_no_competitive_entries() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 0, 6);
    assert_eq!(set.competitive_count(), 0);
    // Full set still has at least the root.
    assert!(set.total_count() >= 1);
    assert!(set.contains(graph.g_root()));
}

/// The depth cutoff bounds the V-depth of every competitive cell: with the
/// cutoff at zero, no selected cell sits deeper than zero however busy the
/// graph. The bound is enforced while the tree is being walked rather than by
/// discarding candidates afterwards, so the cutoff limits the work done as
/// well as the cells returned.
///
/// ´claim:selection:the-depth-cutoff-bounds-the-v-depth-of-every-competitive-cell´
/// ´test:crate:depth-cutoff-zero-excludes-all-non-root´
#[test]
fn depth_cutoff_zero_excludes_all_non_root() {
    let graph = populated_graph();
    // depth_cutoff=0 means only V-depth 0 entries qualify.
    // In practice this is restrictive enough that the
    // competitive set should be small or empty.
    let set = AnalysisSet::recompute(&graph, 100, 0);
    // Root is always present but never competitive.
    assert!(set.contains(graph.g_root()));
    // All competitive entries (if any) must have v_depth == 0.
    for entry in set.competitive() {
        assert_eq!(entry.v_depth, 0, "depth_cutoff=0 should only admit v_depth=0");
    }
}

/// A floating-coordinate graph can contain cells deeper than its model width because splitting is gated by V-tree depth. Their modeled suffix has no remaining width, so selection excludes them rather than overflowing the width subtraction or admitting them as enormous candidates.
///
/// ´claim:selection:overdeep-float-cells-have-zero-remaining-width-and-are-excluded´
/// ´test:crate:overdeep-float-cells-are-excluded´
#[test]
fn overdeep_float_cells_are_excluded() {
    const MODEL_WIDTH: u32 = 4;
    let cfg = GvConfig {
        split_threshold: 5.0,
        depth_create: 8,
        depth_evict: 16,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut graph = GvGraph::<f64, f64, MODEL_WIDTH>::new(cfg);
    for _ in 0..100 {
        graph.observe(2.0, 10.0);
    }

    let deepest = graph.layers().map(|(_, node)| node.depth).max().unwrap_or_default();
    assert!(deepest > MODEL_WIDTH, "the fixture must contain a node deeper than N");

    let set = AnalysisSet::recompute(&graph, usize::MAX, usize::MAX);
    assert!(
        set.competitive().iter().all(|entry| entry.depth <= MODEL_WIDTH),
        "selection must exclude every node deeper than N",
    );
}

/// Recomputing over an unchanged graph selects the same cells in the same
/// order. Nothing in selection depends on iteration order, hashing, or
/// timing, so two sentinels fed identical observations reach identical
/// analysis sets — the foundation the reproducibility of every downstream
/// score rests on.
///
/// ´claim:selection:recomputing-an-unchanged-graph-selects-the-same-cells-in-the-same-order´
/// ´test:crate:tie-breaking-is-deterministic´
#[test]
fn tie_breaking_is_deterministic() {
    let graph = test_graph();
    let set1 = AnalysisSet::recompute(&graph, 3, 6);
    let set2 = AnalysisSet::recompute(&graph, 3, 6);
    assert_eq!(
        set1.competitive().iter().map(|e| e.start).collect::<Vec<_>>(),
        set2.competitive().iter().map(|e| e.start).collect::<Vec<_>>(),
    );
}

/// Competitive cells come back in a total order: importance descending,
/// and among cells of equal importance, interval start ascending. Ties are
/// therefore settled by a property of the coordinate domain rather than by
/// whatever order the tree walk happened to produce, which is what makes the
/// ordering reproducible and not merely stable within one run.
///
/// ´claim:selection:competitive-cells-are-ordered-by-importance-with-ties-broken-by-interval-start´
/// ´test:crate:competitive-ordering-by-importance-then-start´
#[test]
fn competitive_ordering_by_importance_then_start() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 20, 6);
    let comp = set.competitive();
    for pair in comp.windows(2) {
        let (a, b) = (&pair[0], &pair[1]);
        assert!(
            a.importance > b.importance || (a.importance == b.importance && a.start <= b.start),
            "competitive entries must be ordered by importance desc, start asc",
        );
    }
}

/// A graph driven hard enough to split still yields competitive cells. The
/// selector ranks by V-Tree importance alone and applies no filter on G-tree
/// state, so a cell that has since become internal keeps its V-Tree position
/// and remains eligible. Splitting refines the spatial structure; it does not
/// silently remove cells from consideration.
///
/// ´claim:selection:ranking-is-by-v-tree-importance-alone-so-internal-cells-stay-eligible´
/// ´test:crate:internal-nodes-eligible-for-competitive-set´
#[test]
fn internal_nodes_eligible_for_competitive_set() {
    // After splits, internal G-Tree nodes retain their V-Tree
    // position and frozen importance. The competitive set uses
    // V-Tree ranking exclusively (§ALGO S-4.1) — no G-Tree state
    // filter — so internal nodes with sufficient importance remain
    // eligible.
    let mut graph = test_graph();
    for _ in 0..500 {
        graph.observe(0u128, 1u64);
        graph.observe(u128::MAX / 2, 1u64);
    }

    let set = AnalysisSet::recompute(&graph, 100, 20);
    assert!(
        set.competitive_count() > 0,
        "graph with splits should have competitive entries",
    );
}

// ── Accessor methods ────────────────────────────────────

/// Every competitive cell is also in the full set, and the full set is never
/// the smaller of the two. Winning the competition confers membership rather
/// than replacing it, so a cell can be looked up by either question without
/// the two answers contradicting each other.
///
/// ´claim:selection:every-competitive-cell-is-also-in-the-full-set´
/// ´test:crate:full-set-is-superset-of-competitive´
#[test]
fn full_set_is_superset_of_competitive() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    for entry in set.competitive() {
        assert!(set.contains(entry.gnode), "competitive entry must appear in full set");
    }
    assert!(set.total_count() >= set.competitive_count());
}

/// Membership is decided by what the set actually holds, not by whether a
/// handle looks plausible: a fabricated handle the graph never allocated is
/// simply absent. A caller holding a stale or invented cell identifier gets a
/// negative answer rather than an accidental match on a reused slot.
///
/// ´claim:selection:a-handle-the-graph-never-allocated-is-absent-from-the-set´
/// ´test:crate:contains-returns-false-for-absent-node´
#[test]
fn contains_returns_false_for_absent_node() {
    let graph = test_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    // GNodeId is an opaque arena handle. A fabricated id that was
    // never allocated by the graph cannot appear in the set.
    let bogus = GNodeId::from_parts(999_999, 0);
    assert!(!set.contains(bogus));
}

/// The competitiveness predicate agrees with the competitive list: every cell
/// the set lists as competitive answers to that question as well. Asking by
/// handle and reading the list are two views of one fact, so the two ways a
/// caller can learn a cell's standing cannot disagree.
///
/// ´claim:selection:the-competitiveness-predicate-agrees-with-the-competitive-list´
/// ´test:crate:is-competitive-true-for-selected-entries´
#[test]
fn is_competitive_true_for_selected_entries() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    for entry in set.competitive() {
        assert!(set.is_competitive(entry.gnode));
    }
}

/// The same agreement holds in the negative direction: a cell drawn in only
/// by ancestor closure is never reported as competitive. Cells the closure
/// added are therefore distinguishable from cells that earned their place,
/// which matters because the two were selected for entirely different
/// reasons.
///
/// (´claim:selection:the-competitiveness-predicate-agrees-with-the-competitive-list´)
/// ´test:crate:is-competitive-false-for-ancestor-only´
#[test]
fn is_competitive_false_for_ancestor_only() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    for entry in set.full() {
        if !entry.is_competitive {
            assert!(
                !set.is_competitive(entry.gnode),
                "ancestor-only entry must not be reported as competitive",
            );
        }
    }
}

// ── summary() ───────────────────────────────────────────

/// A summary of a set with nothing selected reports zeroes throughout — sizes,
/// depth span, importance span and V-depth span alike — rather than omitting
/// the ranges or filling them with sentinels. The full size is one, because
/// the root is there. A host parsing summaries gets the same shape whether or
/// not anything was selected.
///
/// ´claim:selection:a-summary-with-nothing-selected-reports-zeroed-ranges-rather-than-omitting-them´
/// ´test:crate:summary-empty-graph´
#[test]
fn summary_empty_graph() {
    let graph = test_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    let s = set.summary();
    assert_eq!(s.competitive_size, 0);
    assert_eq!(s.full_size, 1); // root only
    assert_eq!(s.depth_range, (0, 0));
    assert_eq!(s.importance_range, (0.0, 0.0));
    assert_eq!(s.v_depth_range, (0, 0));
}

/// A summary counts the cells that competed and the cells the closure added
/// as separate figures, and on a populated graph the full count strictly
/// exceeds the competitive one. The cost of ancestry is therefore visible: a
/// host can see how much modelling it is paying for beyond the cells it
/// actually chose to invest in.
///
/// ´claim:selection:a-summary-counts-competition-and-closure-separately-so-the-cost-of-ancestry-is-visible´
/// ´test:crate:summary-with-competitive-cells´
#[test]
fn summary_with_competitive_cells() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 4, 6);
    let s = set.summary();
    assert!(s.competitive_size > 0);
    assert!(s.competitive_size <= 4);
    assert!(s.full_size > s.competitive_size); // at least root + competitive
}

/// The producing sets shrink to whatever is online, but the investment does
/// not: a cell still being warmed has been paid for and has produced nothing
/// yet, and that gap is the whole difference between the two readings. A
/// summary taken over the online cells therefore filters the producing count
/// and leaves the investment count whole, so a host watching a warm-up sees
/// what it has committed to as well as what is answering.
///
/// ´claim:selection:a-summary-over-the-online-cells-leaves-the-investment-count-whole´
/// ´test:crate:summary-online-keeps-the-investment-count-whole´
#[test]
fn summary_online_keeps_the_investment_count_whole() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 4, 6);
    assert!(set.total_count() > 1, "the fixture must select more than the root");

    // One cell online; every other selected cell stands for one still being
    // warmed in staging.
    let online: BTreeSet<GNodeId> = set.full().iter().map(|e| e.gnode).take(1).collect();
    let s = set.summary_online(&online);

    assert_eq!(s.full_size, 1, "the producing set is the online part of the selection");
    assert_eq!(
        s.investment_set_size,
        set.total_count(),
        "the investment is the whole selection, warming cells included"
    );
}

/// The depth span of a populated set always begins at zero, because the root
/// sits at depth zero and is always a member. The span therefore reports the
/// reach of the whole modelled chain rather than only the band the selected
/// cells happen to occupy.
///
/// (´claim:selection:the-root-is-always-in-the-full-set-so-every-ancestor-chain-terminates´)
/// ´test:crate:summary-depth-range-includes-root´
#[test]
fn summary_depth_range_includes_root() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 4, 6);
    let s = set.summary();
    assert_eq!(s.depth_range.0, 0, "root at depth 0 is always in the full set");
}

/// Where cells were selected at all, the least important of them still
/// carries importance above zero, and the reported span runs the right way
/// round. A cell can only win the competition on accumulated observation, so
/// nothing with no traffic behind it appears in the summary as though it had
/// been chosen.
///
/// ´claim:selection:a-selected-cell-carries-importance-above-zero-and-the-reported-span-runs-the-right-way-round´
/// ´test:crate:summary-importance-range-positive´
#[test]
fn summary_importance_range_positive() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    let s = set.summary();
    if s.competitive_size > 0 {
        assert!(s.importance_range.0 > 0.0, "min importance should be positive");
        assert!(s.importance_range.1 >= s.importance_range.0, "max >= min");
    }
}

/// Every competitively selected cell sits below the top of the V-Tree: the
/// reported V-depth span starts above zero. The only entry at depth zero is
/// the root, and it is barred from the competition, so what the summary
/// describes is genuinely the refined structure rather than the whole domain
/// counted once more.
///
/// (´claim:selection:the-root-is-never-competitive-however-important-it-is´)
/// ´test:crate:summary-v-depth-range-nonzero´
#[test]
fn summary_v_depth_range_nonzero() {
    let graph = populated_graph();
    let set = AnalysisSet::recompute(&graph, 10, 6);
    let s = set.summary();
    if s.competitive_size > 0 {
        assert!(s.v_depth_range.0 > 0, "competitive entries should have v_depth > 0");
        assert!(s.v_depth_range.1 >= s.v_depth_range.0, "max >= min");
    }
}
