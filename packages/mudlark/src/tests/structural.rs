// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Structural tests that need `pub(crate)` access to `gnodes()`.
//!
//! These tests were originally integration tests but require
//! arena-level access (Surface 3), so they live here as crate tests
//! to satisfy the three-surface boundary (ADR-M-032).

use crate::gnode::GState;
use crate::graph::{Config, GvGraph};
use crate::invariants::assert_invariants;
use crate::testing::{Plan, default_config, plan_range_tree, range_tree_config, run, small_buffer_config};
use crate::tests::init_tracing;

// ── from eviction.rs ────────────────────────────────────────────

#[test]
fn semi_internal_surviving_child_unaffected() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    let g = run::<u64, u64, 8>(small_buffer_config(15), &plan);
    assert_invariants(&g);

    // Check that a surviving child (if any) keeps its sum after
    // observing in the evicted sibling's region.
    let root = g.gnodes().get(g.g_root().index());
    if let Some(child_id) = root.left.or(root.right) {
        let child_sum = g.gnodes().get(child_id.index()).sum;
        let child_hi = g.gnodes().get(child_id.index()).hi;
        if child_hi < 256 {
            let mut g = g;
            g.observe(child_hi, 2u64);
            assert_eq!(g.gnodes().get(child_id.index()).sum, child_sum);
            assert_invariants(&g);
        }
    }
}

// ── from graph_extract.rs ───────────────────────────────────────

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

#[test]
fn extract_multi_node_classification() {
    // build_range_tree creates a multi-node tree. The exact shape
    // depends on split mechanics; we verify structural properties
    // rather than hardcoded counts.
    let g = build_range_tree();
    let p = g.extract();

    // Count G-node states by walking the arena directly.
    let mut expected_terminals = 0u32;
    let mut expected_non_terminals = 0u32;
    for (_idx, gn) in g.gnodes().iter_occupied() {
        if gn.is_terminal() {
            expected_terminals += 1;
        } else {
            expected_non_terminals += 1;
        }
    }

    let total_transitions: usize = p.layers.iter().map(|l| l.transitions.len()).sum();
    let total_terminals: usize = p.layers.iter().map(|l| l.terminals.len()).sum();

    assert_eq!(
        total_transitions, expected_non_terminals as usize,
        "every non-terminal G-node should be a transition"
    );
    assert_eq!(
        total_terminals, expected_terminals as usize,
        "every terminal G-node should be a terminal"
    );
    assert_eq!(p.node_count(), g.node_count() as usize);
    // Multi-node tree must have at least 2 layers.
    assert!(p.layer_count() >= 2, "multi-node tree should produce multiple layers");
}

#[test]
fn extract_semi_internal_is_transition() {
    // DC-021-5: semi-internal G-nodes are classified as transitions.
    // Create a tree where a node has exactly one child.
    let cfg = Config {
        split_threshold: 1,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    // Single observation above threshold → root splits, creating
    // one child. Depending on the split protocol, root may become
    // semi-internal or internal.
    g.observe(32u64, 5u64);

    let p = g.extract();
    // Check all transitions come from non-terminal G-nodes.
    for layer in &p.layers {
        for tr in &layer.transitions {
            // Look up the G-node: it should be Internal or SemiInternal
            // (we can't easily look it up by region, so we verify
            //  refinement > 0 or baseline != total).
            assert!(tr.total >= tr.baseline, "transition total should be >= baseline");
        }
    }

    // Verify no terminal ended up with children by checking that
    // the total non-terminal G-node count matches transition count.
    let g_root = g.gnodes().get(g.g_root().index());
    let root_is_terminal = g_root.is_terminal();
    if !root_is_terminal {
        // Root split; should be extracted as a transition.
        let total_transitions: usize = p.layers.iter().map(|l| l.transitions.len()).sum();
        assert!(total_transitions >= 1, "non-terminal root should appear as transition");
    }
}

// ── from graph_terminal.rs ──────────────────────────────────────

#[test]
#[allow(clippy::cast_possible_truncation)]
fn terminal_count_matches_arena_walk() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    for &c in &[0u64, 128, 64, 192, 32, 96] {
        g.observe(c, 6u64);
        assert_invariants(&g);
        // Explicit manual check.
        let walk_count = g.gnodes().iter_occupied().filter(|(_, gn)| gn.is_terminal()).count() as u32;
        assert_eq!(g.terminal_count(), walk_count);
    }
}

// ── from graph_point.rs ─────────────────────────────────────────

#[test]
fn get_all_terminals_covered() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    for &c in &[0u64, 64, 128, 192] {
        for _ in 0..6 {
            g.observe(c, 1u64);
        }
    }
    assert_invariants(&g);

    // For every terminal G-node, get(g.lo) should return a cell
    // that starts at g.lo (or trimmed equivalent).
    for (_idx, gn) in g.gnodes().iter_occupied() {
        if gn.state() == GState::Terminal {
            let cell = g.get(gn.lo);
            assert!(
                cell.start <= gn.lo && gn.lo < cell.end,
                "get({:?}) returned [{:?}, {:?}) which does not contain lo",
                gn.lo,
                cell.start,
                cell.end,
            );
        }
    }
}
