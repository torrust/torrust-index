// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::graph::Config;
use crate::rebalance::is_violated;
use crate::split::attempt_split;
use crate::vnode::VKind;
use crate::{GvGraph, VNodeId};

fn test_config() -> Config<u64> {
    Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

// ── attempt_split guards ────────────────────────────────────

#[test]
fn no_split_when_not_terminal() {
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(test_config());
    // Manually give the root enough intensity to split.
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    // Bootstrap split creates children — root is no longer terminal.
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 3);

    // Second split on the same (now non-terminal) node is rejected.
    let count_before = graph.node_count();
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), count_before);
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn no_split_when_below_threshold() {
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(test_config());
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 3;
    graph.gnodes.get_mut(root.index()).sum = 3;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 3;
    // Manual sum change → keep plateau consistent.
    #[cfg(feature = "dynamic-contour-tracking")]
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));
    attempt_split(&mut graph, root);
    // sum=3 <= θ=5, no split.
    assert_eq!(graph.node_count(), 1);
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn no_split_when_width_1() {
    // N=3, domain [0,8). Create a graph, manually shrink a node to width 1.
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(test_config());
    let root = graph.g_root();
    // Manually set range to [3,4) — width 1, can't split.
    let g = graph.gnodes.get_mut(root.index());
    g.lo = 3;
    g.hi = 4;
    g.own = 100;
    g.sum = 100;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 100;
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 1);
    // Note: full invariant check skipped — the manual interval
    // mutation [0,8)→[3,4) intentionally breaks domain coverage
    // (P-I1). This test only verifies the width-1 split guard.
}

// ── bootstrap split ─────────────────────────────────────────

#[test]
fn bootstrap_split_creates_correct_structure() {
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(test_config());
    let root = graph.g_root();

    // Set root intensity above θ.
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;

    attempt_split(&mut graph, root);

    // G-Tree: root + 2 children = 3 nodes.
    assert_eq!(graph.node_count(), 3);

    // G-children cover correct ranges.
    let g = graph.gnodes.get(root.index());
    let left_id = g.left.expect("should have left child");
    let right_id = g.right.expect("should have right child");
    let left = graph.gnodes.get(left_id.index());
    let right = graph.gnodes.get(right_id.index());
    assert_eq!(left.lo, 0);
    assert_eq!(left.hi, 4);
    assert_eq!(right.lo, 4);
    assert_eq!(right.hi, 8);
    assert_eq!(left.sum, 0);
    assert_eq!(right.sum, 0);

    // V-Tree: root_s(2-node) -> [entry(10,F), cs(2-node)] -> [le(0,T), re(0,T)]
    // 5 V-nodes total: root_s, entry, cs, le, re
    assert_eq!(graph.vnodes.count(), 5);

    let v_root = graph.v_root().unwrap();
    let vr = graph.vnodes.get(v_root.index());
    assert_eq!(vr.intensity, 10);
    if let VKind::Structural { children, .. } = &vr.kind {
        assert_eq!(children.len(), 2);
    } else {
        panic!("V-root should be structural");
    }

    // Entry is frozen (internal: not exposed, not evictable).
    let entry = graph.vnodes.get(entry_id.index());
    if let VKind::Entry {
        is_exposed,
        is_evictable,
        ..
    } = &entry.kind
    {
        assert!(!is_exposed);
        assert!(!is_evictable);
    } else {
        panic!("should be entry");
    }
    crate::invariants::assert_invariants(&graph);
}

// ── catalytic split ─────────────────────────────────────────

#[test]
fn catalytic_split_after_bootstrap() {
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(test_config());
    let root = graph.g_root();

    // Bootstrap: observe enough to split root.
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 3);

    // Now give left child enough to split.
    let left_id = graph.gnodes.get(root.index()).left.unwrap();
    graph.gnodes.get_mut(left_id.index()).own = 15;
    graph.gnodes.get_mut(left_id.index()).sum = 15;
    let le_entry = graph.gnodes.get(left_id.index()).entry.unwrap();
    graph.vnodes.get_mut(le_entry.index()).intensity = 15;

    // Propagate G-sums and V-sums to maintain invariants.
    crate::gtree::recompute_g_sums(&mut graph.gnodes, left_id);
    crate::vtree::update_parent_cached_intensity(&mut graph.vnodes, le_entry, 15);
    crate::vtree::propagate_v_sums(&mut graph.vnodes, le_entry);

    // Manual state setup may create max-uncle violations; enqueue
    // and rebalance just as observe() would.
    if crate::rebalance::is_violated(&graph.vnodes, le_entry) {
        tracing::debug!(
            entry = %crate::rebalance::Ctx(&graph.vnodes, le_entry),
            "enqueuing test violation",
        );
        graph.violations.push(le_entry);
    }

    attempt_split(&mut graph, left_id);

    let new_gnodes = crate::rebalance::rebalance(
        &mut graph.vnodes,
        &mut graph.gnodes,
        &mut graph.violations,
        graph.live_depth_evict,
    );
    graph.handle_legacy_promotes(&new_gnodes);

    // G-Tree: root + left + right + left.left + left.right = 5 nodes.
    assert_eq!(graph.node_count(), 5);

    // Left child is no longer terminal.
    let left = graph.gnodes.get(left_id.index());
    assert!(left.left.is_some());
    assert!(left.right.is_some());

    // Left's V-entry is frozen (internal: not exposed, not evictable).
    let le = graph.vnodes.get(le_entry.index());
    if let VKind::Entry {
        is_exposed,
        is_evictable,
        ..
    } = &le.kind
    {
        assert!(!is_exposed);
        assert!(!is_evictable);
    }
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn catalytic_split_is_violation_free() {
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(test_config());
    let root = graph.g_root();

    // Bootstrap split.
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    attempt_split(&mut graph, root);

    // Catalytic split on left child.
    let left_id = graph.gnodes.get(root.index()).left.unwrap();
    graph.gnodes.get_mut(left_id.index()).own = 8;
    graph.gnodes.get_mut(left_id.index()).sum = 8;
    let le_entry = graph.gnodes.get(left_id.index()).entry.unwrap();
    graph.vnodes.get_mut(le_entry.index()).intensity = 8;
    crate::gtree::recompute_g_sums(&mut graph.gnodes, left_id);
    crate::vtree::update_parent_cached_intensity(&mut graph.vnodes, le_entry, 8);
    crate::vtree::propagate_v_sums(&mut graph.vnodes, le_entry);
    attempt_split(&mut graph, left_id);

    // Check: no V-node is violated.
    for i in 0..20 {
        if graph.vnodes.is_occupied(i) {
            let id = VNodeId::from_index(i);
            assert!(!is_violated(&graph.vnodes, id), "node at index {i} should not be violated");
        }
    }
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn depth_gate_rejects_deep_split() {
    // Config with D_create=1 so only very shallow entries can split.
    let config = Config {
        split_threshold: 5,
        depth_create: 1,
        depth_evict: 4,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(config);
    let root = graph.g_root();

    // Bootstrap split (entry at depth 0, allowed since 0 <= 1 — but
    // after bootstrap, entry moves to depth 1).
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 3);

    // Left child's entry is at V-depth 2 (root_s -> cs -> le).
    // D_create=1, so 2 > 1 → rejected.
    let left_id = graph.gnodes.get(root.index()).left.unwrap();
    graph.gnodes.get_mut(left_id.index()).own = 100;
    graph.gnodes.get_mut(left_id.index()).sum = 100;
    let le_entry = graph.gnodes.get(left_id.index()).entry.unwrap();
    graph.vnodes.get_mut(le_entry.index()).intensity = 100;
    crate::gtree::recompute_g_sums(&mut graph.gnodes, left_id);
    crate::vtree::update_parent_cached_intensity(&mut graph.vnodes, le_entry, 100);
    crate::vtree::propagate_v_sums(&mut graph.vnodes, le_entry);
    // Manual sum change → keep plateau consistent.
    #[cfg(feature = "dynamic-contour-tracking")]
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));

    // Manual state setup may create max-uncle violations; enqueue
    // and rebalance just as observe() would.
    if crate::rebalance::is_violated(&graph.vnodes, le_entry) {
        tracing::debug!(
            entry = %crate::rebalance::Ctx(&graph.vnodes, le_entry),
            "enqueuing test violation (depth gate)",
        );
        graph.violations.push(le_entry);
    }

    attempt_split(&mut graph, left_id);

    let new_gnodes = crate::rebalance::rebalance(
        &mut graph.vnodes,
        &mut graph.gnodes,
        &mut graph.violations,
        graph.live_depth_evict,
    );
    graph.handle_legacy_promotes(&new_gnodes);
    // Should NOT have split — depth gate denied.
    assert_eq!(graph.node_count(), 3);
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn preprocessing_contraction_before_catalytic_split() {
    // After bootstrap + one catalytic split, the child structural cs
    // becomes a 3-node. A second catalytic split on its child triggers
    // preprocessing contraction.
    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(test_config());
    let root = graph.g_root();

    // Bootstrap.
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 3); // root, left, right

    // Catalytic split on left [0,8) -> [0,4), [4,8).
    let left_id = graph.gnodes.get(root.index()).left.unwrap();
    graph.gnodes.get_mut(left_id.index()).own = 8;
    graph.gnodes.get_mut(left_id.index()).sum = 8;
    let le_entry = graph.gnodes.get(left_id.index()).entry.unwrap();
    graph.vnodes.get_mut(le_entry.index()).intensity = 8;
    crate::gtree::recompute_g_sums(&mut graph.gnodes, left_id);
    crate::vtree::update_parent_cached_intensity(&mut graph.vnodes, le_entry, 8);
    crate::vtree::propagate_v_sums(&mut graph.vnodes, le_entry);
    attempt_split(&mut graph, left_id);
    assert_eq!(graph.node_count(), 5);

    // Now the right child [8,16) should also be splittable.
    // Its parent in V-tree (cs or wherever it ended up) might be a
    // 3-node after the catalytic split added a child.
    let right_id = graph.gnodes.get(root.index()).right.unwrap();
    graph.gnodes.get_mut(right_id.index()).own = 7;
    graph.gnodes.get_mut(right_id.index()).sum = 7;
    let re_entry = graph.gnodes.get(right_id.index()).entry.unwrap();
    graph.vnodes.get_mut(re_entry.index()).intensity = 7;
    crate::gtree::recompute_g_sums(&mut graph.gnodes, right_id);
    crate::vtree::update_parent_cached_intensity(&mut graph.vnodes, re_entry, 7);
    crate::vtree::propagate_v_sums(&mut graph.vnodes, re_entry);
    attempt_split(&mut graph, right_id);

    // Should have split (after possible preprocessing contraction).
    assert_eq!(graph.node_count(), 7);
    crate::invariants::assert_invariants(&graph);
}

// ── Plateau unit tests (Step 2G) ────────────────────────────

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn bootstrap_split_plateau_count() {
    // After bootstrap split, the tree is uniformly one level deeper.
    // The root now has both children at the same depth → still
    // one plateau (or more, depending on whether adjacent regions
    // are at the same depth).
    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(test_config());
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    crate::vtree::propagate_v_sums(&mut graph.vnodes, entry_id);
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));

    attempt_split(&mut graph, root);
    crate::invariants::assert_invariants(&graph);

    // Still exactly 1 plateau (uniform depth after first split).
    assert_eq!(graph.plateaus.len(), 1);
}

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn bootstrap_split_plateau_depth() {
    use crate::split::attempt_split;

    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(test_config());
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 10;
    graph.gnodes.get_mut(root.index()).sum = 10;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 10;
    crate::vtree::propagate_v_sums(&mut graph.vnodes, entry_id);
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));

    attempt_split(&mut graph, root);
    crate::invariants::assert_invariants(&graph);

    // The sole plateau should have depth = 1 (children are at depth 1).
    let p = graph.plateaus.values().next().unwrap();
    assert_eq!(p.depth, 1);
}

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn catalytic_split_creates_boundary() {
    // Drive enough observations through observe() to trigger
    // catalytic splits that create new plateau boundaries.

    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(test_config());
    for _ in 0..10 {
        g.observe(3u64, 10u64);
    }
    crate::invariants::assert_invariants(&g);

    // Multiple splits should have created > 1 plateau.
    assert!(
        g.plateaus.len() > 1,
        "catalytic splits should create plateau boundaries: got {}",
        g.plateaus.len(),
    );
}
