// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for the **Sentinel integration API** (§ADR M-036).
//!
//! The sentinel API exposes the internal g-tree structure through
//! stable `GNodeId` handles, allowing external consumers to inspect
//! node intervals, depths, parent/child relationships, and ancestry
//! — without coupling to the internal representation.  These tests
//! verify the three deliverables (D1–D3) defined in the ADR.
//!
//! # Test index
//!
//! ## D1 — `Node.gnode_id`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`node_carries_gnode_id`] | every node has a unique id that round-trips through `gnode_info` |
//! | [`node_gnode_id_empty_tree`] | root-only tree has `gnode_id == g_root()` |
//!
//! ## D2 — `gnode_info()` / `gnode_children()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnode_info_live_node`] | root info returns correct interval and depth |
//! | [`gnode_info_root_has_no_parent`] | root's parent is `None` |
//! | [`gnode_info_children_consistent`] | parent/child back-pointers, interval partitioning, state consistency |
//! | [`gnode_info_dead_handle`] | evicted handles return `None` |
//! | [`gnode_info_sum_equals_own_plus_children`] | `sum == own + Σ children.sum` |
//! | [`gnode_children_dead_handle`] | evicted handles return `None` from `gnode_children` |
//! | [`gnode_info_depth_monotonic`] | children have strictly greater depth than parents |
//! | [`gnode_info_terminal_own_equals_sum`] | terminal nodes satisfy `own == sum` |
//! | [`dfs_walk_matches_layers_count`] | DFS via `gnode_children` visits same nodes as `layers()` |
//!
//! ## D3 — `is_ancestor_of()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`is_ancestor_of_root_ancestors_all`] | root is ancestor of every non-root node |
//! | [`is_ancestor_of_self_is_false`] | proper ancestor excludes self |
//! | [`is_ancestor_of_sibling_is_false`] | siblings are not ancestors of each other |
//! | [`is_ancestor_of_parent_child`] | direct parent ↔ child relationship, asymmetric |
//! | [`is_ancestor_of_stale_handles`] | evicted handles always return `false` |
//! | [`is_ancestor_of_transitive`] | ancestor relation is transitive along leaf-to-root paths |
//!
//! ## Cross-preset
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_tree_gnode_ids_consistent`] | range-tree preset nodes round-trip through `gnode_info` |

mod support;

use std::collections::HashSet;

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{default_config, plan_range_tree, range_tree_config, run};
use torrust_mudlark::{Config, GNodeId, GState, GvGraph};

// ── Helpers ─────────────────────────────────────────────────────

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

/// Build a split tree: low split threshold to force internal nodes.
fn build_split_tree() -> GvGraph<u64, u64, 8> {
    let mut g = GvGraph::<u64, u64, 8>::new(range_tree_config());
    // Spread observations to create several splits.
    for coord in [16u64, 48, 80, 112, 144, 176, 208, 240] {
        g.observe(coord, 10u64);
        g.observe(coord, 10u64);
    }
    assert_invariants(&g);
    g
}

/// Config for dead-handle / stale-handle tests: θ=1, budget=100.
const fn dead_handle_config() -> Config<u64> {
    Config {
        budget: Some(100),
        ..range_tree_config()
    }
}

/// Build a graph with some initial nodes, then force evictions so
/// that some of the initial handles become dead.  Returns the graph
/// and the set of initially-captured `GNodeId`s.
fn build_with_dead_handles() -> (GvGraph<u64, u64, 8>, Vec<GNodeId>) {
    let mut g = GvGraph::<u64, u64, 8>::new(dead_handle_config());

    // Create initial structure.
    g.observe(42u64, 10u64);
    g.observe(42u64, 10u64);
    let initial_ids: Vec<GNodeId> = g.layers().map(|(_, n)| n.gnode_id).collect();

    // Hammer a different area to force evictions.
    for i in 0..50u64 {
        g.observe(200 + (i % 4), 5u64);
    }
    assert_invariants(&g);

    (g, initial_ids)
}

// ── D1: Node.gnode_id ───────────────────────────────────────────

#[test]
fn node_carries_gnode_id() {
    let g = build_split_tree();

    let mut seen_ids = HashSet::new();
    for (_layer, node) in g.layers() {
        // The gnode_id should be unique across all nodes.
        assert!(seen_ids.insert(node.gnode_id), "duplicate gnode_id {:?}", node.gnode_id);

        // The gnode_id should round-trip through gnode_info().
        let info = g.gnode_info(node.gnode_id).expect("gnode_id from layers() should be live");
        assert_eq!(info.start, node.start);
        assert_eq!(info.end, node.end);
        assert_eq!(info.own, node.own);
        assert_eq!(info.sum, node.sum);
        assert_eq!(info.depth, node.depth);
        assert_eq!(info.state, node.state);
    }
}

#[test]
fn node_gnode_id_empty_tree() {
    let g = GvGraph::<u64, u64, 8>::new(default_config());
    let entries: Vec<_> = g.layers().collect();
    assert_eq!(entries.len(), 1);
    let (_layer, node) = &entries[0];
    // The root node's gnode_id should equal g_root().
    assert_eq!(node.gnode_id, g.g_root());
}

// ── D2: gnode_info() / gnode_children() ─────────────────────────

#[test]
fn gnode_info_live_node() {
    let g = build_split_tree();
    let root = g.g_root();

    let info = g.gnode_info(root).expect("root should be live");
    assert_eq!(info.start, 0);
    assert_eq!(info.end, 256); // 2^8
    assert_eq!(info.depth, 0);
}

#[test]
fn gnode_info_root_has_no_parent() {
    let g = build_split_tree();
    let info = g.gnode_info(g.g_root()).unwrap();
    assert!(info.parent.is_none(), "root should have no parent");
}

#[test]
fn gnode_info_children_consistent() {
    let g = build_split_tree();

    for (_layer, node) in g.layers() {
        let info = g.gnode_info(node.gnode_id).unwrap();
        let children = g.gnode_children(node.gnode_id).unwrap();
        let midpoint = u64::midpoint(info.start, info.end);

        // If there's a left child, its parent should point back.
        if let Some(left_id) = children.left {
            let left_info = g.gnode_info(left_id).expect("left child should be live");
            assert_eq!(
                left_info.parent,
                Some(node.gnode_id),
                "left child's parent should be {:?}, got {:?}",
                node.gnode_id,
                left_info.parent,
            );
            // Left child's interval should be [start, midpoint).
            assert_eq!(left_info.start, info.start);
            assert_eq!(left_info.end, midpoint);
        }

        // If there's a right child, its parent should point back.
        if let Some(right_id) = children.right {
            let right_info = g.gnode_info(right_id).expect("right child should be live");
            assert_eq!(
                right_info.parent,
                Some(node.gnode_id),
                "right child's parent should be {:?}, got {:?}",
                node.gnode_id,
                right_info.parent,
            );
            // Right child's interval should be [midpoint, end).
            assert_eq!(right_info.start, midpoint);
            assert_eq!(right_info.end, info.end);
        }

        // State consistency: children presence matches GState.
        match info.state {
            GState::Terminal => {
                assert!(children.left.is_none() && children.right.is_none());
            }
            GState::SemiInternal => {
                assert!(children.left.is_some() ^ children.right.is_some());
            }
            GState::Internal => {
                assert!(children.left.is_some() && children.right.is_some());
            }
        }
    }
}

#[test]
fn gnode_info_dead_handle() {
    let (g, initial_ids) = build_with_dead_handles();
    let live_ids: HashSet<GNodeId> = g.layers().map(|(_, n)| n.gnode_id).collect();

    for id in &initial_ids {
        if !live_ids.contains(id) {
            assert!(
                g.gnode_info(*id).is_none(),
                "evicted handle {id:?} should return None from gnode_info",
            );
        }
    }
}

#[test]
fn gnode_info_sum_equals_own_plus_children() {
    let g = build_split_tree();

    for (_layer, node) in g.layers() {
        let info = g.gnode_info(node.gnode_id).unwrap();
        let children = g.gnode_children(node.gnode_id).unwrap();

        let child_sum: u64 = [children.left, children.right]
            .iter()
            .flatten()
            .map(|cid| g.gnode_info(*cid).unwrap().sum)
            .sum();

        assert_eq!(
            info.sum,
            info.own + child_sum,
            "sum should equal own + children's sums for node {:?}",
            node.gnode_id,
        );
    }
}

#[test]
fn gnode_children_dead_handle() {
    let (g, initial_ids) = build_with_dead_handles();
    let live_ids: HashSet<GNodeId> = g.layers().map(|(_, n)| n.gnode_id).collect();

    for id in &initial_ids {
        if !live_ids.contains(id) {
            assert!(
                g.gnode_children(*id).is_none(),
                "evicted handle {id:?} should return None from gnode_children",
            );
        }
    }
}

#[test]
fn gnode_info_depth_monotonic() {
    let g = build_split_tree();

    for (_layer, node) in g.layers() {
        let children = g.gnode_children(node.gnode_id).unwrap();
        for child_id in [children.left, children.right].into_iter().flatten() {
            let child_info = g.gnode_info(child_id).unwrap();
            assert!(
                child_info.depth > node.depth,
                "child {:?} depth {} should be > parent {:?} depth {}",
                child_id,
                child_info.depth,
                node.gnode_id,
                node.depth,
            );
        }
    }
}

#[test]
fn gnode_info_terminal_own_equals_sum() {
    let g = build_split_tree();

    let mut found_terminal = false;
    for (_layer, node) in g.layers() {
        if node.state == GState::Terminal {
            found_terminal = true;
            assert_eq!(
                node.own, node.sum,
                "terminal node {:?} should have own == sum, got own={}, sum={}",
                node.gnode_id, node.own, node.sum,
            );
        }
    }
    assert!(found_terminal, "split tree should have at least one terminal");
}

#[test]
fn dfs_walk_matches_layers_count() {
    let g = build_split_tree();

    // DFS walk using gnode_children from root.
    let mut stack = vec![g.g_root()];
    let mut dfs_ids = HashSet::new();
    while let Some(id) = stack.pop() {
        dfs_ids.insert(id);
        if let Some(ch) = g.gnode_children(id) {
            stack.extend(ch.left);
            stack.extend(ch.right);
        }
    }

    // layers() walk.
    let layers_ids: HashSet<GNodeId> = g.layers().map(|(_, n)| n.gnode_id).collect();

    assert_eq!(
        dfs_ids, layers_ids,
        "DFS via gnode_children should visit exactly the same nodes as layers()",
    );
}

// ── D3: is_ancestor_of() ───────────────────────────────────────

#[test]
fn is_ancestor_of_root_ancestors_all() {
    let g = build_split_tree();
    let root = g.g_root();

    for (_layer, node) in g.layers() {
        if node.gnode_id != root {
            assert!(
                g.is_ancestor_of(root, node.gnode_id),
                "root should be ancestor of {:?}",
                node.gnode_id,
            );
        }
    }
}

#[test]
fn is_ancestor_of_self_is_false() {
    let g = build_split_tree();
    let root = g.g_root();

    // Root vs itself.
    assert!(!g.is_ancestor_of(root, root));

    // Every node vs itself.
    for (_layer, node) in g.layers() {
        assert!(
            !g.is_ancestor_of(node.gnode_id, node.gnode_id),
            "a node should not be its own proper ancestor: {:?}",
            node.gnode_id,
        );
    }
}

#[test]
fn is_ancestor_of_sibling_is_false() {
    let g = build_split_tree();

    for (_layer, node) in g.layers() {
        let children = g.gnode_children(node.gnode_id).unwrap();

        if let (Some(left), Some(right)) = (children.left, children.right) {
            assert!(!g.is_ancestor_of(left, right), "left sibling should not be ancestor of right");
            assert!(!g.is_ancestor_of(right, left), "right sibling should not be ancestor of left");
        }
    }
}

#[test]
fn is_ancestor_of_parent_child() {
    let g = build_split_tree();

    for (_layer, node) in g.layers() {
        let children = g.gnode_children(node.gnode_id).unwrap();

        if let Some(left) = children.left {
            assert!(g.is_ancestor_of(node.gnode_id, left));
            assert!(!g.is_ancestor_of(left, node.gnode_id));
        }
        if let Some(right) = children.right {
            assert!(g.is_ancestor_of(node.gnode_id, right));
            assert!(!g.is_ancestor_of(right, node.gnode_id));
        }
    }
}

#[test]
fn is_ancestor_of_stale_handles() {
    let (g, initial_ids) = build_with_dead_handles();
    let live_ids: HashSet<GNodeId> = g.layers().map(|(_, n)| n.gnode_id).collect();

    // For any evicted handle, is_ancestor_of should return false.
    for id in &initial_ids {
        if !live_ids.contains(id) {
            assert!(!g.is_ancestor_of(*id, g.g_root()));
            assert!(!g.is_ancestor_of(g.g_root(), *id));
        }
    }
}

#[test]
fn is_ancestor_of_transitive() {
    let g = build_split_tree();

    // Walk a path from any leaf to root and verify transitivity.
    for (_layer, node) in g.layers() {
        if node.state != GState::Terminal {
            continue;
        }

        // Build path from leaf to root.
        let mut path = vec![node.gnode_id];
        let mut current = node.gnode_id;
        loop {
            let info = g.gnode_info(current).unwrap();
            match info.parent {
                Some(p) => {
                    path.push(p);
                    current = p;
                }
                None => break,
            }
        }

        // Every earlier node in the path is a descendant of every
        // later node (path goes leaf → root).
        for i in 0..path.len() {
            for j in (i + 1)..path.len() {
                assert!(
                    g.is_ancestor_of(path[j], path[i]),
                    "{:?} (depth j={j}) should be ancestor of {:?} (depth i={i})",
                    path[j],
                    path[i],
                );
            }
        }

        // Just test one leaf path for efficiency.
        break;
    }
}

// ── Cross-preset ────────────────────────────────────────────────

#[test]
fn range_tree_gnode_ids_consistent() {
    let g = build_range_tree();
    assert_invariants(&g);

    // Every layers() node should have a live gnode_info().
    for (_layer, node) in g.layers() {
        let info = g.gnode_info(node.gnode_id).unwrap();
        assert_eq!(info.start, node.start);
        assert_eq!(info.end, node.end);
    }
}
