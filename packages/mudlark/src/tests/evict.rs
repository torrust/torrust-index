// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness tests for **tip eviction** (`evict_tip`) and
//! **candidate scanning** (`scan_for_candidates`) (§IDEA M-12).
//!
//! Eviction removes a terminal V-entry from the tree, folding its
//! energy back into the parent G-node so that the root sum is
//! preserved.  These tests verify energy conservation across single
//! and sequential evictions, correct `terminal_count` bookkeeping,
//! expected panics on illegal targets, and the depth-based candidate
//! scan that decides *which* tips are eligible for removal.
//!
//! The plateau tests (behind `feature = "dynamic-contour-tracking"`)
//! confirm that thatch structures are correctly created or destroyed
//! when eviction changes a node between terminal and semi-internal
//! status.
//!
//! # Test index
//!
//! ## `evict_tip` — basic
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evict_single_left_child`] | evict left child; parent keeps right, energy conserved |
//! | [`evict_single_right_child`] | evict right child; parent keeps left, energy conserved |
//! | [`evict_both_children_makes_parent_terminal`] | both children evicted → parent regains terminal + exposed status |
//! | [`evict_after_observations_conserves_energy`] | eviction after additional observations still conserves energy |
//!
//! ## `evict_tip` — deeper trees
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evict_grandchild_conserves_energy`] | evict a deep terminal found by scan; root sum unchanged |
//! | [`sequential_evictions_conserve_energy`] | loop-evict all candidates; energy + invariants hold at every step |
//!
//! ## `evict_tip` — `terminal_count` tracking
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_decreases_on_eviction`] | evict one child of 3-node tree → count decreases by 1 |
//! | [`terminal_count_unchanged_when_parent_becomes_terminal`] | second eviction: −1 terminal + parent gains terminal = net 0 |
//!
//! ## `evict_tip` — panics
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evict_root_panics`] | evicting the G-root entry panics |
//! | [`evict_structural_panics`] | evicting a structural V-node panics |
//!
//! ## `scan_for_candidates`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`no_candidates_when_within_d_evict`] | entries within `D_evict` depth yield no candidates |
//! | [`finds_candidates_past_d_evict`] | lowered `D_evict` surfaces non-root evictable entries |
//! | [`prunes_subtrees_without_evictable`] | only evictable entries appear in results |
//! | [`scan_on_single_node_graph`] | single-node graph trivially has no candidates |
//! | [`scan_deep_tree_finds_candidates`] | deep N=8 tree with lowered `D_evict` finds non-root candidates |
//!
//! ## Plateau (`feature = "dynamic-contour-tracking"`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evict_to_terminal_destroys_thatch`] | evicting both children destroys thatch, leaves 1 plateau |
//! | [`evict_to_semi_internal_creates_thatch`] | evicting one child → semi-internal maintains/increases plateaus |

use crate::evict::{evict_tip, scan_for_candidates};
use crate::graph::{Config, GvGraph};
use crate::handle::VNodeId;
use crate::invariants::assert_invariants;
use crate::testing::{GraphCreator, default_config, evictable_config, plan_evictable, run_checked};
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::VKind;

// ── Helpers ─────────────────────────────────────────────────────

/// Build a graph with a bootstrap split (root + 2 children), N=4.
fn graph_with_split() -> GvGraph<u64, u64, 4> {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(default_config())
        .observe(3, 10) // triggers bootstrap split
        .check_every(1)
        .build();
    assert_eq!(g.node_count(), 3);
    g
}

/// Evict a V-entry, rebalance, and handle legacy promotes.
fn evict_and_rebalance<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(g: &mut GvGraph<C, V, N>, v_id: VNodeId) {
    evict_tip(g, v_id);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);
}

/// Return the V-entry id for a G-node's left child.
fn left_entry(g: &GvGraph<u64, u64, 4>) -> VNodeId {
    let root = g.g_root();
    let left_id = g.gnodes.get(root.index()).left.expect("root must have a left child");
    g.gnodes.get(left_id.index()).entry.expect("left child must have V-entry")
}

/// Return the V-entry id for a G-node's right child.
fn right_entry(g: &GvGraph<u64, u64, 4>) -> VNodeId {
    let root = g.g_root();
    let right_id = g.gnodes.get(root.index()).right.expect("root must have a right child");
    g.gnodes.get(right_id.index()).entry.expect("right child must have V-entry")
}

/// Assert that a V-entry is evictable.
fn assert_evictable(g: &GvGraph<u64, u64, 4>, v_id: VNodeId) {
    let VKind::Entry { is_evictable, .. } = &g.vnodes.get(v_id.index()).kind else {
        panic!("V-node {} should be an entry, not structural", v_id.index());
    };
    assert!(is_evictable, "V-entry {} should be evictable", v_id.index());
}

// ── evict_tip — basic ───────────────────────────────────────────

#[test]
fn evict_single_left_child() {
    let mut g = graph_with_split();
    let root = g.g_root();
    let root_sum_before = g.gnodes.get(root.index()).sum;

    let entry = left_entry(&g);
    assert_evictable(&g, entry);

    evict_and_rebalance(&mut g, entry);

    assert_eq!(g.node_count(), 2);
    assert!(g.gnodes.get(root.index()).left.is_none());
    assert!(g.gnodes.get(root.index()).right.is_some());
    assert_eq!(g.gnodes.get(root.index()).sum, root_sum_before, "energy conservation");
    assert_invariants(&g);
}

#[test]
fn evict_single_right_child() {
    let mut g = graph_with_split();
    let root = g.g_root();
    let root_sum_before = g.gnodes.get(root.index()).sum;

    let entry = right_entry(&g);
    assert_evictable(&g, entry);

    evict_and_rebalance(&mut g, entry);

    assert_eq!(g.node_count(), 2);
    assert!(g.gnodes.get(root.index()).right.is_none());
    assert!(g.gnodes.get(root.index()).left.is_some());
    assert_eq!(g.gnodes.get(root.index()).sum, root_sum_before, "energy conservation");
    assert_invariants(&g);
}

#[test]
fn evict_both_children_makes_parent_terminal() {
    let mut g = graph_with_split();
    let root = g.g_root();
    let root_sum_before = g.gnodes.get(root.index()).sum;

    let le = left_entry(&g);
    let re = right_entry(&g);

    // Evict left.
    evict_and_rebalance(&mut g, le);
    assert_eq!(g.node_count(), 2);
    assert_invariants(&g);

    // Evict right.
    evict_and_rebalance(&mut g, re);
    assert_eq!(g.node_count(), 1);

    // Root is now terminal again.
    let root_g = g.gnodes.get(root.index());
    assert!(root_g.left.is_none());
    assert!(root_g.right.is_none());
    assert_eq!(root_g.sum, root_sum_before, "energy conservation");

    // Root's entry: exposed + evictable.
    let root_entry = root_g.entry.expect("root must have V-entry");
    let VKind::Entry {
        is_evictable,
        is_exposed,
        ..
    } = &g.vnodes.get(root_entry.index()).kind
    else {
        panic!("root V-node should be an entry");
    };
    assert!(is_exposed, "root should be exposed after evicting both children");
    assert!(is_evictable, "root should be evictable after evicting both children");

    assert_invariants(&g);
}

#[test]
fn evict_after_observations_conserves_energy() {
    let mut g = graph_with_split();
    // Small deltas below split threshold (5) keep children terminal.
    g.observe(3u64, 3u64);
    g.observe(12u64, 4u64);
    let root_sum_before = g.gnodes.get(g.g_root().index()).sum;
    assert_invariants(&g);

    let entry = left_entry(&g);
    evict_and_rebalance(&mut g, entry);

    assert_eq!(g.gnodes.get(g.g_root().index()).sum, root_sum_before, "energy conservation");
    assert_invariants(&g);
}

// ── evict_tip — deeper trees ────────────────────────────────────

#[test]
fn evict_grandchild_conserves_energy() {
    // Build a deeper tree via plan_evictable (N=8) and evict a
    // terminal descendant found by scan_for_candidates.
    let mut g: GvGraph<u64, u64, 8> = run_checked(evictable_config(), &plan_evictable(), 1);
    assert!(g.node_count() > 3, "should have more than 3 nodes after plan_evictable");

    // Lower live_depth_evict so deep terminals become candidates.
    // Must also adjust live_depth_create to satisfy D-I3: create < evict.
    g.live_depth_evict = 2;
    g.live_depth_create = 2 - g.depth_buffer();

    let candidates = scan_for_candidates(&g);
    assert!(!candidates.is_empty(), "should find evictable candidates in deep tree");

    let root_sum_before = g.gnodes.get(g.g_root().index()).sum;
    let target = candidates[0];

    evict_and_rebalance(&mut g, target);

    assert_eq!(g.gnodes.get(g.g_root().index()).sum, root_sum_before, "energy conservation");
    assert_invariants(&g);
}

#[test]
fn sequential_evictions_conserve_energy() {
    // Build a deep tree (N=8) and evict multiple terminals one by
    // one, verifying energy conservation on every step.
    let mut g: GvGraph<u64, u64, 8> = run_checked(evictable_config(), &plan_evictable(), 1);
    let initial_count = g.node_count();
    assert!(initial_count > 3);

    // Lower live_depth_evict so scan finds candidates.
    // Must also adjust live_depth_create to satisfy D-I3: create < evict.
    g.live_depth_evict = 2;
    g.live_depth_create = 2 - g.depth_buffer();

    let root_sum = g.gnodes.get(g.g_root().index()).sum;
    let mut evicted = 0u32;

    loop {
        let candidates = scan_for_candidates(&g);
        if candidates.is_empty() {
            break;
        }
        evict_and_rebalance(&mut g, candidates[0]);
        evicted += 1;

        assert_eq!(
            g.gnodes.get(g.g_root().index()).sum,
            root_sum,
            "energy conservation after eviction #{evicted}"
        );
        assert_invariants(&g);
    }

    assert!(evicted >= 2, "should have evicted at least 2 nodes, got {evicted}");
    assert!(g.node_count() < initial_count, "node count should decrease after evictions");
}

// ── evict_tip — terminal_count tracking ─────────────────────────

#[test]
fn terminal_count_decreases_on_eviction() {
    // After evicting one child of a 3-node tree, the parent
    // stays semi-internal → net terminal_count decreases by 1.
    let mut g = graph_with_split();
    let tc_before = g.terminal_count();

    let entry = left_entry(&g);
    evict_and_rebalance(&mut g, entry);

    // Parent still has a right child, so it's semi-internal, not terminal.
    assert_eq!(g.terminal_count(), tc_before - 1, "terminal count should decrease by 1");
    assert_invariants(&g);
}

#[test]
fn terminal_count_unchanged_when_parent_becomes_terminal() {
    // After evicting both children, the parent becomes terminal.
    // The second eviction removes a terminal (−1) but the parent
    // gains terminal status (+1) → net change is 0.
    let mut g = graph_with_split();
    let le = left_entry(&g);
    evict_and_rebalance(&mut g, le);
    let tc_after_first = g.terminal_count();

    let root = g.g_root();
    let right_id = g.gnodes.get(root.index()).right.expect("right child should still exist");
    let re = g.gnodes.get(right_id.index()).entry.expect("right child must have V-entry");
    evict_and_rebalance(&mut g, re);

    assert_eq!(
        g.terminal_count(),
        tc_after_first,
        "terminal count should be unchanged (parent became terminal)"
    );
    assert_eq!(g.terminal_count(), 1, "single root terminal");
    assert_invariants(&g);
}

// ── evict_tip — panics ──────────────────────────────────────────

#[test]
#[should_panic(expected = "cannot evict the G-root")]
fn evict_root_panics() {
    let mut g = graph_with_split();
    let root_entry = g.gnodes.get(g.g_root().index()).entry.expect("root must have V-entry");

    // Artificially mark the root entry as evictable so we reach
    // the G-root guard (not the debug_assert on is_evictable).
    if let VKind::Entry { is_evictable, .. } = &mut g.vnodes.get_mut(root_entry.index()).kind {
        *is_evictable = true;
    }

    evict_tip(&mut g, root_entry);
}

#[test]
#[should_panic(expected = "is structural, not an entry")]
fn evict_structural_panics() {
    let mut g = graph_with_split();
    // The V-root (after a split) should be structural.
    let v_root = g.v_root.expect("graph must have a V-root");
    let is_structural = matches!(&g.vnodes.get(v_root.index()).kind, VKind::Structural { .. });
    assert!(is_structural, "V-root should be structural after a split");

    evict_tip(&mut g, v_root);
}

// ── scan_for_candidates ─────────────────────────────────────────

#[test]
fn no_candidates_when_within_d_evict() {
    // Default D_evict = 6. After a bootstrap split, child entries
    // are at V-depth 2 or 3 — well within bounds.
    let g = graph_with_split();
    let candidates = scan_for_candidates(&g);
    assert!(candidates.is_empty());
}

#[test]
fn finds_candidates_past_d_evict() {
    let config = Config {
        depth_create: 1,
        depth_evict: 2,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(config);
    g.observe(3u64, 10u64); // bootstrap split

    // Lower live_depth_evict so entries at depth 2 satisfy `depth > D_evict`.
    g.live_depth_evict = 1;
    assert_eq!(g.node_count(), 3);

    let candidates = scan_for_candidates(&g);
    assert!(!candidates.is_empty(), "should find candidates past D_evict");
    // All candidates must be evictable entries (not root).
    for &c in &candidates {
        let VKind::Entry { gnode, is_evictable, .. } = &g.vnodes.get(c.index()).kind else {
            panic!("candidate {} should be an entry, not structural", c.index());
        };
        assert!(is_evictable);
        assert_ne!(*gnode, g.g_root());
    }
}

#[test]
fn prunes_subtrees_without_evictable() {
    let config = Config {
        depth_create: 1,
        depth_evict: 2,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(config);
    g.observe(3u64, 10u64);
    g.live_depth_evict = 0; // everything past depth 0 is eligible

    let candidates = scan_for_candidates(&g);
    // No candidate should be a non-evictable entry.
    for &c in &candidates {
        let VKind::Entry { is_evictable, .. } = &g.vnodes.get(c.index()).kind else {
            panic!("candidate {} should be an entry, not structural", c.index());
        };
        assert!(is_evictable, "only evictable entries should be candidates");
    }
}

#[test]
fn scan_on_single_node_graph() {
    // Before any observation: single root, trivially no candidates.
    let g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    assert_eq!(g.node_count(), 1);
    let candidates = scan_for_candidates(&g);
    assert!(candidates.is_empty(), "single-node graph should have no candidates");
}

#[test]
fn scan_deep_tree_finds_candidates() {
    // Build a deep tree with plan_evictable and verify scan
    // discovers candidates once live_depth_evict is lowered.
    let mut g: GvGraph<u64, u64, 8> = run_checked(evictable_config(), &plan_evictable(), 1);

    // At the default live_depth_evict, there may or may not be
    // candidates. Lower it to force candidates.
    g.live_depth_evict = 2;
    g.live_depth_create = 2 - g.depth_buffer();

    let candidates = scan_for_candidates(&g);
    assert!(!candidates.is_empty(), "deep tree should have candidates at low D_evict");

    // Every candidate must be a non-root evictable entry.
    for &c in &candidates {
        let VKind::Entry { gnode, is_evictable, .. } = &g.vnodes.get(c.index()).kind else {
            panic!("candidate {} should be an entry", c.index());
        };
        assert!(is_evictable, "candidate {} must be evictable", c.index());
        assert_ne!(*gnode, g.g_root(), "candidate must not be the G-root");
    }
}

// ── Plateau unit tests (feature = "dynamic-contour-tracking") ───

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn evict_to_terminal_destroys_thatch() {
    let mut g = graph_with_split();
    let root = g.g_root();

    // Evict left child.
    let le = left_entry(&g);
    evict_and_rebalance(&mut g, le);
    assert_invariants(&g);

    let plateaus_after_first = g.plateaus.len();

    // Evict right child to make root terminal again.
    let right_id = g.gnodes.get(root.index()).right.expect("right child should still exist");
    let re = g.gnodes.get(right_id.index()).entry.expect("right child must have V-entry");
    evict_and_rebalance(&mut g, re);
    assert_invariants(&g);

    // Root is terminal → exactly 1 plateau.
    assert_eq!(g.plateaus.len(), 1);
    assert!(
        g.plateaus.len() <= plateaus_after_first,
        "thatch should be destroyed when parent becomes terminal",
    );
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn evict_to_semi_internal_creates_thatch() {
    let mut g = graph_with_split();
    assert_eq!(g.node_count(), 3);

    let plateaus_before = g.plateaus.len();

    // Evict left child → root becomes semi-internal.
    let le = left_entry(&g);
    evict_and_rebalance(&mut g, le);
    assert_invariants(&g);

    // Root is now semi-internal — it should be a basis element
    // of a plateau (the uncovered half's plateau). P-I4 checked
    // by assert_invariants.
    assert!(
        g.plateaus.len() >= plateaus_before,
        "eviction to semi-internal should maintain or increase plateau count",
    );
}
