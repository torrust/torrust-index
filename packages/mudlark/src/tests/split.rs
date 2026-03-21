// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness tests for **`attempt_split`** (§IDEA M-10).
//!
//! Splitting is the primary mechanism for refining the G-tree: when a
//! terminal G-node accumulates energy above the split threshold *θ*,
//! it is bisected into two children that cover its left and right
//! half-intervals.  The first split on a fresh root is the
//! *bootstrap* split (§IDEA M-10.3); all subsequent splits are
//! *catalytic* (§IDEA M-10.2) and may trigger preprocessing
//! contraction when the parent structural V-node is already a 3-node.
//!
//! The tests are arranged in four tiers:
//! 1. **Guard tests** — verify every early-return path in
//!    `attempt_split` (non-terminal, below threshold, width-1
//!    interval, missing V-entry).
//! 2. **Bootstrap split** — shape checks on the G-tree and V-tree
//!    produced by the very first split.
//! 3. **Catalytic split** — deeper splits, depth-gate rejection,
//!    and preprocessing contraction.
//! 4. **High-level / `observe()`** — splits triggered through the
//!    public API to confirm end-to-end behaviour.
//!
//! An additional section covers **plateau tracking** behind the
//! `dynamic-contour-tracking` feature gate.
//!
//! # Test index
//!
//! ## `attempt_split` guards
//!
//! | Test | Focus |
//! |------|-------|
//! | [`no_split_when_not_terminal`] | guard 1: already has G-children |
//! | [`no_split_when_below_threshold`] | guard 3: sum ≤ θ |
//! | [`no_split_when_at_exact_threshold`] | guard 3 boundary: sum == θ (strict `>`) |
//! | [`no_split_when_width_1`] | guard 2: interval indivisible |
//! | [`no_split_when_entry_is_none`] | guard 4: missing V-entry |
//!
//! ## Bootstrap split (§IDEA M-10.3)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bootstrap_split_creates_correct_structure`] | G-tree + V-tree shape after first split |
//!
//! ## Catalytic split (§IDEA M-10.2)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`catalytic_split_after_bootstrap`] | structure after 2nd split |
//! | [`catalytic_split_is_violation_free`] | no max-uncle violations |
//! | [`depth_gate_rejects_deep_split`] | `D_create` gate rejects too-deep entries |
//! | [`preprocessing_contraction_before_catalytic_split`] | 3-node contraction before split |
//!
//! ## High-level (via `observe()`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_triggers_bootstrap_split`] | split through public API |
//! | [`observe_triggers_catalytic_split`] | deeper split through repeated hotspot |
//!
//! ## Plateau (`feature = "dynamic-contour-tracking"`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bootstrap_split_plateau_count`] | single plateau after bootstrap |
//! | [`bootstrap_split_plateau_depth`] | plateau depth == 1 |
//! | [`catalytic_split_creates_boundary`] | >1 plateau after catalytic splits |

use crate::GvGraph;
use crate::graph::Config;
use crate::handle::VNodeId;
use crate::rebalance::is_violated;
use crate::split::attempt_split;
use crate::testing::{GraphCreator, Plan, default_config, run};
use crate::tests::init_tracing;
use crate::vnode::VKind;

// ── attempt_split guards ────────────────────────────────────

#[test]
fn no_split_when_not_terminal() {
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
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
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 3;
    graph.gnodes.get_mut(root.index()).sum = 3;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 3;
    // Manual sum change → keep plateau consistent.
    #[cfg(feature = "dynamic-contour-tracking")]
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));
    attempt_split(&mut graph, root);
    // sum=3 < θ=5, no split.
    assert_eq!(graph.node_count(), 1);
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn no_split_when_at_exact_threshold() {
    // Guard 3 uses strict `>`, so sum == θ must NOT split.
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let root = graph.g_root();
    graph.gnodes.get_mut(root.index()).own = 5;
    graph.gnodes.get_mut(root.index()).sum = 5;
    let entry_id = graph.gnodes.get(root.index()).entry.unwrap();
    graph.vnodes.get_mut(entry_id.index()).intensity = 5;
    #[cfg(feature = "dynamic-contour-tracking")]
    graph.recompute_plateau(&crate::plateau::BasisEdge(0u64));
    attempt_split(&mut graph, root);
    // sum=5 == θ=5, guard uses `>` so no split.
    assert_eq!(graph.node_count(), 1);
    crate::invariants::assert_invariants(&graph);
}

#[test]
fn no_split_when_width_1() {
    // N=3, domain [0,8). Create a graph, manually shrink a node to width 1.
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(default_config());
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

#[test]
fn no_split_when_entry_is_none() {
    // Guard 4: if g.entry is None, attempt_split returns early.
    let mut graph: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let root = graph.g_root();
    // Remove the entry link.
    graph.gnodes.get_mut(root.index()).entry = None;
    graph.gnodes.get_mut(root.index()).own = 100;
    graph.gnodes.get_mut(root.index()).sum = 100;
    attempt_split(&mut graph, root);
    assert_eq!(graph.node_count(), 1);
}

// ── bootstrap split ─────────────────────────────────────────

#[test]
fn bootstrap_split_creates_correct_structure() {
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(default_config());
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
    let _t = init_tracing();
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(default_config());
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
    if is_violated(&graph.vnodes, le_entry) {
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
    let mut graph: GvGraph<u64, u64, 3> = GvGraph::new(default_config());
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
    let _t = init_tracing();
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
    if is_violated(&graph.vnodes, le_entry) {
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
    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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

// ── High-level (via observe()) ──────────────────────────────

#[test]
fn observe_triggers_bootstrap_split() {
    // A single observation above θ should trigger bootstrap split
    // through the public API, producing 3 G-nodes.
    let g: GvGraph<u64, u64, 3> = run(default_config(), &Plan::new().observe(3, 10));
    assert_eq!(g.node_count(), 3);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_triggers_catalytic_split() {
    // Repeated hotspot should drive the tree deeper via catalytic
    // splits, checked at every step.
    let g: GvGraph<u64, u64, 3> = GraphCreator::default_u64().hotspot(3, 10, 5).check_every(1).build();
    assert!(g.node_count() > 3, "repeated hotspot should trigger catalytic split");
    crate::invariants::assert_invariants(&g);
}

// ── Plateau unit tests (Step 2G) ────────────────────────────

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn bootstrap_split_plateau_count() {
    // After bootstrap split, the tree is uniformly one level deeper.
    // The root now has both children at the same depth → still
    // one plateau (or more, depending on whether adjacent regions
    // are at the same depth).
    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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
    let mut graph: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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
