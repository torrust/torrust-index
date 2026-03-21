// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness tests for **`GvGraph::observe()`** (§IDEA M-8).
//!
//! `observe` is the primary mutation entry-point: it inserts energy at a
//! coordinate, propagates g-sums up the tree, manages the V-layer
//! entries, and drives the full split → violation → eviction pipeline.
//! These tests exercise each stage in isolation and in combination:
//!
//! * **Accumulation** — energy is recorded in `own`/`sum` and mirrored
//!   in the V-entry intensity, both for repeated and distinct coords.
//! * **Routing** — after a split, subsequent observations land in the
//!   correct child and g-sums propagate back to the root.
//! * **Split triggers** — bootstrap and cascading splits fire (or not)
//!   based on the threshold, and V-entry terminal flags flip correctly.
//! * **Violations & eviction** — the violations queue is fully drained
//!   by the time `observe` returns, and the hard budget guarantee
//!   (ADR-M-018) is never breached.
//! * **Boundary coordinates** — domain extremes (0 and max) route to
//!   the expected children.
//! * **Plateaus** — when `dynamic-contour-tracking` is enabled, plateau
//!   sums and structure are maintained.
//!
//! # Test index
//!
//! ## Accumulation — single & additive
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_accumulates_own`] | single observation sets `own` and `sum` on root |
//! | [`observe_updates_v_entry_intensity`] | V-entry intensity mirrors the accumulated value |
//! | [`observe_accumulates_additively`] | same coordinate accumulates additively |
//! | [`observe_accumulates_distinct_coordinates`] | distinct coordinates both land in root pre-split |
//!
//! ## Routing — post-split observation dispatch
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_routes_after_split`] | second observation routes to correct child |
//! | [`observe_propagates_g_sums_correctly`] | g-sums propagate up through left and right children |
//!
//! ## Split triggers
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_no_split_below_threshold`] | energy below threshold leaves root as leaf |
//! | [`observe_triggers_bootstrap_split`] | energy above threshold creates two children |
//! | [`cascading_splits`] | repeated hotspot triggers cascading child splits |
//! | [`v_entry_terminal_flag_flipped_after_split`] | parent V-entry becomes non-exposed after split |
//!
//! ## Violations queue
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_violations_drained`] | violations queue is empty after every observe call |
//!
//! ## Budget & eviction (pipeline step 8)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_triggers_eviction_when_over_budget`] | hard budget (ADR-M-018) never exceeded |
//! | [`observe_violations_empty_after_eviction`] | violations drained after eviction cycle |
//! | [`observe_invariants_hold_after_eviction_cycle`] | full invariants hold across 50 observe+evict rounds |
//!
//! ## Boundary coordinates
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_at_domain_boundary_zero`] | coord 0 routes to left child |
//! | [`observe_at_domain_boundary_max`] | coord `2^N − 1` routes to right child |
//!
//! ## Public API consistency
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_total_sum_matches_root_sum`] | `total_sum()` equals root g-node `sum` |
//!
//! ## Plateau (`feature = "dynamic-contour-tracking"`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_updates_plateau_sum`] | plateau sum tracks accumulated energy |
//! | [`observe_preserves_plateau_structure`] | plateau keys and depths unchanged by observation |

use crate::GvGraph;
use crate::testing::{GraphCreator, default_config};
use crate::vnode::VKind;

/// High-threshold config: prevents splitting so accumulation tests
/// can focus on value propagation without structural changes.
fn no_split_config() -> crate::graph::Config<u64> {
    crate::graph::Config {
        split_threshold: 100,
        ..default_config()
    }
}

// ── Accumulation — single & additive ────────────────────────────

#[test]
fn observe_accumulates_own() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
    g.observe(5u64, 7u64);

    let root = g.gnodes.get(g.g_root.index());
    assert_eq!(root.own, 7);
    assert_eq!(root.sum, 7);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_updates_v_entry_intensity() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
    g.observe(5u64, 7u64);

    let entry_id = g.gnodes.get(g.g_root.index()).entry.unwrap();
    let v = g.vnodes.get(entry_id.index());
    assert_eq!(v.intensity, 7);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_accumulates_additively() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
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
fn observe_accumulates_distinct_coordinates() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
    g.observe(3u64, 5u64);
    g.observe(10u64, 8u64);

    // Both observations land in the root (no split).
    let root = g.gnodes.get(g.g_root.index());
    assert_eq!(root.own, 13);
    assert_eq!(root.sum, 13);
    assert_eq!(g.total_sum(), 13);
    assert_eq!(g.node_count(), 1);
    crate::invariants::assert_invariants(&g);
}

// ── Routing — post-split observation dispatch ───────────────────

#[test]
fn observe_routes_after_split() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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

    assert_eq!(g.total_sum(), 15);
    crate::invariants::assert_invariants(&g);
}

// ── Split triggers ──────────────────────────────────────────────

#[test]
fn observe_no_split_below_threshold() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
    g.observe(5u64, 3u64);

    // 3 < 100 → no split.
    let root = g.gnodes.get(g.g_root.index());
    assert!(root.left.is_none());
    assert!(root.right.is_none());
    assert_eq!(g.node_count(), 1);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_triggers_bootstrap_split() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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

    assert_eq!(g.node_count(), 3);
    crate::invariants::assert_invariants(&g);
}

#[test]
fn cascading_splits() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    // First split: root [0,16) → [0,8) + [8,16)
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count(), 3);

    // Second observation on left child [0,8): own = 10 > 5 → split
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count(), 5);

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

#[test]
fn v_entry_terminal_flag_flipped_after_split() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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

// ── Violations queue ────────────────────────────────────────────

#[test]
fn observe_violations_drained() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
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

// ── Budget & eviction (pipeline step 8) ─────────────────────────

#[test]
fn observe_triggers_eviction_when_over_budget() {
    // buffer=1, headroom=9, soft_limit=12-9=3.
    // After bootstrap split → 3 nodes. Another split pushes
    // past soft_limit, triggering eviction.
    let cfg = crate::graph::Config {
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        bounded_eviction: false,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Trigger bootstrap split: 3 nodes.
    g.observe(3u64, 10u64);
    assert_eq!(g.node_count(), 3);

    // Large observation on left child: triggers catalytic split.
    // Pipeline step 8 should evict back down.
    g.observe(1u64, 20u64);
    // Hard budget guarantee (ADR-M-018): never exceeds budget.
    assert!(
        g.node_count() as usize <= 12,
        "hard budget violated: node_count={}, budget=12",
        g.node_count()
    );
    assert!(g.violations.is_empty(), "violations queue should be empty");
    crate::invariants::assert_invariants(&g);
}

#[test]
fn observe_violations_empty_after_eviction() {
    let cfg = crate::graph::Config {
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        ..default_config()
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
    let cfg = crate::graph::Config {
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        bounded_eviction: false,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Many observations to trigger splits and evictions.
    for i in 0..50u64 {
        g.observe(i % 16, 3u64);
        crate::invariants::assert_invariants(&g);
    }
}

// ── Boundary coordinates ────────────────────────────────────────

#[test]
fn observe_at_domain_boundary_zero() {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(default_config())
        .observe(0, 10) // triggers bootstrap split
        .observe(0, 5) // routes to left child [0,8)
        .check_every(1)
        .build();

    assert_eq!(g.total_sum(), 15);
    // Observation at coord 0 lands in the left child.
    let root = g.gnodes.get(g.g_root.index());
    let left = g.gnodes.get(root.left.unwrap().index());
    assert_eq!(left.own, 5);
}

#[test]
fn observe_at_domain_boundary_max() {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(default_config())
        .observe(15, 10) // triggers bootstrap split (domain max = 2^4 - 1)
        .observe(15, 5) // routes to right child [8,16)
        .check_every(1)
        .build();

    assert_eq!(g.total_sum(), 15);
    // Observation at coord 15 lands in the right child.
    let root = g.gnodes.get(g.g_root.index());
    let right = g.gnodes.get(root.right.unwrap().index());
    assert_eq!(right.own, 5);
}

// ── Public API consistency ──────────────────────────────────────

#[test]
fn observe_total_sum_matches_root_sum() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    g.observe(3u64, 10u64); // bootstrap split
    g.observe(3u64, 2u64);
    g.observe(10u64, 3u64);
    g.observe(7u64, 1u64);

    let root_sum = g.gnodes.get(g.g_root.index()).sum;
    assert_eq!(g.total_sum(), root_sum);
    assert_eq!(g.total_sum(), 16);
    crate::invariants::assert_invariants(&g);
}

// ── Plateau unit tests (Step 2G) ────────────────────────────────

#[test]
#[cfg(feature = "dynamic-contour-tracking")]
fn observe_updates_plateau_sum() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
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
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(no_split_config());
    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    g.observe(5u64, 7u64);
    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    // No structural change: same keys, same depths.
    assert_eq!(keys_before, keys_after);
    assert_eq!(g.plateaus()[&keys_after[0]].depth, 0);
}
