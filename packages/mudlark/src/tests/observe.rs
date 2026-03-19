// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::GvGraph;
use crate::graph::Config;
#[allow(unused_imports)]
use crate::vnode::VKind;

/// Helper: build a `GvGraph<u64, u64, 4>` with domain `[0, 16)`.
fn make_graph(threshold: u64) -> GvGraph<u64, u64, 4> {
    GvGraph::new(Config {
        split_threshold: threshold,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    })
}

#[test]
fn observe_accumulates_own() {
    let mut g = make_graph(100);
    g.observe(5u64, 7u64);
    let root = g.gnodes.get(g.g_root.index());
    assert_eq!(root.own, 7);
    assert_eq!(root.sum, 7);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_updates_v_entry_intensity() {
    let mut g = make_graph(100);
    g.observe(5u64, 7u64);
    let entry_id = g.gnodes.get(g.g_root.index()).entry.unwrap();
    let v = g.vnodes.get(entry_id.index());
    assert_eq!(v.intensity, 7);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_accumulates_additively() {
    let mut g = make_graph(100);
    g.observe(5u64, 3u64);
    g.observe(5u64, 4u64);
    let root = g.gnodes.get(g.g_root.index());
    assert_eq!(root.own, 7);
    assert_eq!(root.sum, 7);
    let entry_id = root.entry.unwrap();
    assert_eq!(g.vnodes.get(entry_id.index()).intensity, 7);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_triggers_bootstrap_split() {
    let mut g = make_graph(5);
    // 10 > 5 threshold → bootstrap split
    g.observe(3u64, 10u64);

    // Root should now have two G-children.
    let root = g.gnodes.get(g.g_root.index());
    assert!(root.left.is_some());
    assert!(root.right.is_some());

    // Children cover [0,8) and [8,16).
    let left = g.gnodes.get(root.left.unwrap().index());
    let right = g.gnodes.get(root.right.unwrap().index());
    assert_eq!((left.lo, left.hi), (0, 8));
    assert_eq!((right.lo, right.hi), (8, 16));

    // node_count should be 3 (root + 2 children).
    assert_eq!(g.node_count, 3);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_routes_after_split() {
    let mut g = make_graph(5);
    // First observation triggers bootstrap split.
    g.observe(3u64, 10u64);
    // Second observation routes to the left child [0,8).
    g.observe(3u64, 4u64);

    let root = g.gnodes.get(g.g_root.index());
    let left = g.gnodes.get(root.left.unwrap().index());
    assert_eq!(left.own, 4);
    // Root's sum = own(10) + left.sum(4) + right.sum(0) = 14.
    assert_eq!(root.sum, 14);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_propagates_g_sums_correctly() {
    let mut g = make_graph(5);
    g.observe(3u64, 10u64); // bootstrap split
    g.observe(3u64, 2u64); // goes to left child [0,8)
    g.observe(10u64, 3u64); // goes to right child [8,16)

    let root = g.gnodes.get(g.g_root.index());
    // root.sum = root.own(10) + left.sum(2) + right.sum(3) = 15
    assert_eq!(root.sum, 15);
    assert_eq!(root.own, 10);

    let left = g.gnodes.get(root.left.unwrap().index());
    assert_eq!(left.own, 2);
    assert_eq!(left.sum, 2);

    let right = g.gnodes.get(root.right.unwrap().index());
    assert_eq!(right.own, 3);
    assert_eq!(right.sum, 3);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_no_split_below_threshold() {
    let mut g = make_graph(100);
    g.observe(5u64, 3u64);
    // 3 < 100 → no split.
    let root = g.gnodes.get(g.g_root.index());
    assert!(root.left.is_none());
    assert!(root.right.is_none());
    assert_eq!(g.node_count, 1);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_violations_drained() {
    let mut g = make_graph(5);
    // Multiple observations — violations queue should be empty
    // after each observe call returns.
    g.observe(3u64, 10u64);
    assert!(g.violations.is_empty());
    g.observe(3u64, 20u64);
    assert!(g.violations.is_empty());
    g.observe(10u64, 5u64);
    assert!(g.violations.is_empty());
    crate::invariants::assert_invariants(&g);
}

#[test]
fn v_entry_terminal_flag_flipped_after_split() {
    let mut g = make_graph(5);
    g.observe(3u64, 10u64); // triggers bootstrap split

    // The root's V-entry should no longer be exposed (internal node).
    let root_entry_id = g.gnodes.get(g.g_root.index()).entry.unwrap();
    let root_v = g.vnodes.get(root_entry_id.index());
    match &root_v.kind {
        VKind::Entry {
            is_exposed,
            is_evictable,
            ..
        } => {
            assert!(!is_exposed);
            assert!(!is_evictable);
        }
        VKind::Structural { .. } => panic!("expected entry"),
    }

    // Children's V-entries should be exposed and evictable (terminal).
    let left_id = g.gnodes.get(g.g_root.index()).left.unwrap();
    let left_entry_id = g.gnodes.get(left_id.index()).entry.unwrap();
    match &g.vnodes.get(left_entry_id.index()).kind {
        VKind::Entry {
            is_exposed,
            is_evictable,
            ..
        } => {
            assert!(is_exposed);
            assert!(is_evictable);
        }
        VKind::Structural { .. } => panic!("expected entry"),
    }
    crate::invariants::assert_invariants(&g);
}

#[test]
fn cascading_splits() {
    let mut g = make_graph(5);
    // First split: root [0,16) → [0,8) + [8,16)
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count, 3);

    // Second observation on left child [0,8): own = 10 > 5 → split
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count, 5);

    // Verify left child now has children [0,4) and [4,8).
    let left_id = g.gnodes.get(g.g_root.index()).left.unwrap();
    let left = g.gnodes.get(left_id.index());
    assert!(left.left.is_some());
    assert!(left.right.is_some());

    let ll = g.gnodes.get(left.left.unwrap().index());
    let lr = g.gnodes.get(left.right.unwrap().index());
    assert_eq!((ll.lo, ll.hi), (0, 4));
    assert_eq!((lr.lo, lr.hi), (4, 8));
    crate::invariants::assert_invariants(&g);
}

// ── Step 8: observe triggers eviction when over budget ──────

#[test]
fn observe_triggers_eviction_when_over_budget() {
    // buffer=1, headroom=9, soft_limit=12-9=3.
    // After bootstrap split → 3 nodes. Another split pushes
    // past soft_limit, triggering eviction.
    let cfg = Config {
        split_threshold: 5,
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        alpha_relax: 0.75,
        bounded_eviction: false,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Trigger bootstrap split: 3 nodes.
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count, 3);

    // Large observation on left child: triggers catalytic split.
    // Pipeline step 8 should evict back down.
    g.observe(1u64, 20u64);
    // Hard budget guarantee (ADR-M-018): never exceeds budget.
    assert!(
        g.node_count as usize <= 12,
        "hard budget violated: node_count={}, budget=12",
        g.node_count
    );
    assert!(g.violations.is_empty(), "violations queue should be empty");
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_violations_empty_after_eviction() {
    let cfg = Config {
        split_threshold: 5,
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Build up some nodes.
    for i in 0..10u64 {
        g.observe(i % 16, 6u64);
    }
    // After each observe, violations should be fully drained.
    assert!(g.violations.is_empty());
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_invariants_hold_after_eviction_cycle() {
    let cfg = Config {
        split_threshold: 5,
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        alpha_relax: 0.75,
        bounded_eviction: false,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Many observations to trigger splits and evictions.
    for i in 0..50u64 {
        g.observe(i % 16, 3u64);
        crate::invariants::assert_invariants(&g);
    }
}

// ── Plateau unit tests (Step 2G) ────────────────────────────

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn observe_updates_plateau_sum() {
    let mut g = make_graph(100); // high threshold → no split
    g.observe(5u64, 7u64);
    crate::invariants::assert_invariants(&g);

    // Single plateau; its sum should equal root.sum.
    let p = &g.plateaus()[&crate::plateau::BasisEdge(0u64)];
    assert_eq!(p.sum, 7u64);

    g.observe(5u64, 3u64);
    let p = &g.plateaus()[&crate::plateau::BasisEdge(0u64)];
    assert_eq!(p.sum, 10u64);
}

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn observe_preserves_plateau_structure() {
    let mut g = make_graph(100); // high threshold → no split
    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    g.observe(5u64, 7u64);
    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    // No structural change: same keys, same depths.
    assert_eq!(keys_before, keys_after);
    assert_eq!(g.plateaus()[&keys_after[0]].depth, 0);
}
