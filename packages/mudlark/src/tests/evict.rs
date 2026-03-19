// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::evict::{evict_tip, scan_for_candidates};
use crate::graph::{Config, GvGraph};
use crate::invariants::assert_invariants;
use crate::vnode::VKind;

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

/// Build a graph with a bootstrap split (root + 2 children).
fn graph_with_split() -> GvGraph<u64, u64, 4> {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(test_config());
    g.observe(3u64, 10u64); // triggers bootstrap split
    assert_eq!(g.node_count(), 3);
    assert_invariants(&g);
    g
}

#[test]
fn evict_single_terminal_child() {
    let mut g = graph_with_split();
    let root = g.g_root();
    let root_sum_before = g.gnodes.get(root.index()).sum;

    // Find the left child's V-entry.
    let left_id = g.gnodes.get(root.index()).left.unwrap();
    let left_entry = g.gnodes.get(left_id.index()).entry.unwrap();

    // Verify it's an evictable entry.
    if let VKind::Entry { is_evictable, .. } = &g.vnodes.get(left_entry.index()).kind {
        assert!(is_evictable);
    }

    evict_tip(&mut g, left_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);

    // Node count decreased.
    assert_eq!(g.node_count(), 2);

    // Root left child is now None.
    assert!(g.gnodes.get(root.index()).left.is_none());
    // Root right child still exists.
    assert!(g.gnodes.get(root.index()).right.is_some());

    // G-root sum is conserved.
    let root_sum_after = g.gnodes.get(root.index()).sum;
    assert_eq!(root_sum_before, root_sum_after, "energy conservation");

    assert_invariants(&g);
}

#[test]
fn evict_both_children_makes_parent_terminal() {
    let mut g = graph_with_split();
    let root = g.g_root();
    let root_sum_before = g.gnodes.get(root.index()).sum;

    let left_id = g.gnodes.get(root.index()).left.unwrap();
    let right_id = g.gnodes.get(root.index()).right.unwrap();
    let left_entry = g.gnodes.get(left_id.index()).entry.unwrap();
    let right_entry = g.gnodes.get(right_id.index()).entry.unwrap();

    // Evict left.
    evict_tip(&mut g, left_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);
    assert_eq!(g.node_count(), 2);
    assert_invariants(&g);

    // Evict right.
    evict_tip(&mut g, right_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);
    assert_eq!(g.node_count(), 1);

    // Root is now terminal again.
    let root_g = g.gnodes.get(root.index());
    assert!(root_g.left.is_none());
    assert!(root_g.right.is_none());

    // Root sum is conserved.
    assert_eq!(g.gnodes.get(root.index()).sum, root_sum_before, "energy conservation");

    // Root's entry becomes evictable.
    let root_entry = root_g.entry.unwrap();
    if let VKind::Entry {
        is_evictable,
        is_exposed,
        ..
    } = &g.vnodes.get(root_entry.index()).kind
    {
        assert!(is_exposed, "root should be exposed after evicting both children");
        assert!(is_evictable, "root should be evictable after evicting both children");
    }

    assert_invariants(&g);
}

#[test]
fn evict_after_observations_conserves_energy() {
    let mut g = graph_with_split();
    // Use small deltas that stay below the split threshold (5)
    // so children remain terminal.
    g.observe(3u64, 3u64); // into left child (sum 3 ≤ 5)
    g.observe(12u64, 4u64); // into right child (sum 4 ≤ 5)
    let root_sum_before = g.gnodes.get(g.g_root().index()).sum;
    assert_invariants(&g);

    // Evict left.
    let left_id = g.gnodes.get(g.g_root().index()).left.unwrap();
    let left_entry = g.gnodes.get(left_id.index()).entry.unwrap();
    evict_tip(&mut g, left_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);

    // G-root sum is unchanged.
    assert_eq!(g.gnodes.get(g.g_root().index()).sum, root_sum_before, "energy conservation");
    assert_invariants(&g);
}

// ── scan_for_candidates tests ───────────────────────────────

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
    // Set D_evict very low so bootstrap-split entries (depth 2) are eligible.
    // depth > D_evict means depth > 1, which is true for depth=2 entries.
    let config = Config {
        split_threshold: 5,
        depth_create: 1,
        depth_evict: 2,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(config);
    g.observe(3u64, 10u64); // bootstrap split

    // Manually lower live_depth_evict to 1 so entries at depth 2
    // satisfy `depth > D_evict`.
    g.live_depth_evict = 1;
    assert_eq!(g.node_count(), 3);

    let candidates = scan_for_candidates(&g);
    // Both child entries should be eligible (depth > 1, evictable).
    // Actual depth depends on V-tree layout — at least 2 after
    // the bootstrap structural nodes.
    assert!(!candidates.is_empty(), "should find candidates past D_evict=2");
    // All candidates must be evictable entries (not root).
    for &c in &candidates {
        match &g.vnodes.get(c.index()).kind {
            VKind::Entry { gnode, is_evictable, .. } => {
                assert!(is_evictable);
                assert_ne!(*gnode, g.g_root());
            }
            VKind::Structural { .. } => panic!("candidate should be an entry"),
        }
    }
}

#[test]
fn prunes_subtrees_without_evictable() {
    // After a split, the root entry becomes non-terminal (is_exposed=false).
    // The scan should not collect it even if it's past D_evict.
    let config = Config {
        split_threshold: 5,
        depth_create: 1,
        depth_evict: 2,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(config);
    g.observe(3u64, 10u64);
    g.live_depth_evict = 0; // everything past depth 0 is eligible

    let candidates = scan_for_candidates(&g);
    // No candidate should be a non-terminal entry.
    for &c in &candidates {
        if let VKind::Entry { is_evictable, .. } = &g.vnodes.get(c.index()).kind {
            assert!(is_evictable, "only evictable entries should be candidates");
        }
    }
}

// ── Plateau unit tests (Step 2G) ────────────────────────────

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn evict_to_terminal_destroys_thatch() {
    // Build graph with split, then evict. After eviction the parent
    // was semi-internal (if one child survived). Evict the survivor
    // to make parent terminal — the thatch should be destroyed.
    let mut g = graph_with_split();
    let root = g.g_root();

    // Evict left child.
    let left_id = g.gnodes.get(root.index()).left.unwrap();
    let left_entry = g.gnodes.get(left_id.index()).entry.unwrap();
    evict_tip(&mut g, left_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);
    assert_invariants(&g);

    let plateaus_after_first = g.plateaus.len();

    // Now evict the right child to make root terminal again.
    if let Some(right_id) = g.gnodes.get(root.index()).right
        && let Some(right_entry) = g.gnodes.get(right_id.index()).entry
    {
        evict_tip(&mut g, right_entry);
        let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
        g.handle_legacy_promotes(&new_gnodes);
        assert_invariants(&g);

        // After both children evicted: root is terminal, 1 plateau.
        assert_eq!(g.plateaus.len(), 1);
        assert!(
            g.plateaus.len() <= plateaus_after_first,
            "thatch should be destroyed when parent becomes terminal",
        );
    }
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn evict_to_semi_internal_creates_thatch() {
    use crate::evict::evict_tip;

    let mut g = graph_with_split();
    let root = g.g_root();
    assert_eq!(g.node_count(), 3);

    // Record plateau count before eviction.
    let plateaus_before = g.plateaus.len();

    // Evict left child → root becomes semi-internal.
    let left_id = g.gnodes.get(root.index()).left.unwrap();
    let left_entry = g.gnodes.get(left_id.index()).entry.unwrap();
    evict_tip(&mut g, left_entry);
    let new_gnodes = crate::rebalance::rebalance(&mut g.vnodes, &mut g.gnodes, &mut g.violations, g.live_depth_evict);
    g.handle_legacy_promotes(&new_gnodes);
    assert_invariants(&g);

    // Root is now semi-internal — it should be a basis element
    // of a plateau (the uncovered half's plateau). P-I4 checked
    // by assert_invariants.
    assert!(
        g.plateaus.len() >= plateaus_before,
        "eviction to semi-internal should maintain or increase plateau count",
    );
}
