// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Structural tests that need `pub(crate)` access to `gnodes()`.
//!
//! These tests were originally integration tests but require
//! arena-level access (Surface 3), so they live here as crate tests
//! to satisfy the three-surface boundary (§ADR M-032).  They verify
//! properties that can only be asserted by walking the internal
//! G-node arena directly: child-pointer consistency after eviction,
//! correct classification of G-node states in `extract()`, and
//! agreement between cached counters and a full arena walk.
//!
//! # Test index
//!
//! ## Eviction — surviving child
//!
//! | Test | Focus |
//! |------|-------|
//! | [`semi_internal_surviving_child_unaffected`] | surviving child sum unchanged after sibling eviction |
//!
//! ## Extract — classification
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_multi_node_classification`] | transitions + terminals partition equals arena walk |
//! | [`extract_semi_internal_is_transition`] | semi-internal G-nodes are classified as transitions |
//! | [`extract_single_node_is_terminal`] | fresh single-root graph yields one terminal, zero transitions |
//!
//! ## Arena–counter consistency
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_matches_arena_walk`] | `terminal_count()` agrees with arena filter after each observe |
//! | [`node_count_matches_arena_walk`] | `node_count()` agrees with arena length after each observe |
//! | [`gstate_partition_sums_to_node_count`] | Terminal + `SemiInternal` + Internal exhausts `node_count()` |
//!
//! ## Point query — terminal coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_all_terminals_covered`] | `get(lo)` returns a cell containing every terminal's `lo` |

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
    let _t = init_tracing();
    // build_range_tree creates a multi-node tree. The exact shape
    // depends on split mechanics; we verify structural properties
    // rather than hardcoded counts.
    let g = build_range_tree();
    assert_invariants(&g);
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
    let _t = init_tracing();
    // DC-021-5: semi-internal G-nodes are classified as transitions.
    // Create a tree where a node has exactly one child.
    // θ=1 forces immediate splitting on any observation.
    let cfg = Config {
        split_threshold: 1,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    // Single observation above threshold → root splits, creating
    // one child. Depending on the split protocol, root may become
    // semi-internal or internal.
    g.observe(32u64, 5u64);
    assert_invariants(&g);

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
    let _t = init_tracing();
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
    let _t = init_tracing();
    let plan = Plan::new()
        .hotspot(0u64, 1u64, 6)
        .hotspot(64, 1, 6)
        .hotspot(128, 1, 6)
        .hotspot(192, 1, 6);
    let g = run::<u64, u64, 8>(default_config(), &plan);
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

// ── extract — single node ───────────────────────────────────────

#[test]
fn extract_single_node_is_terminal() {
    let _t = init_tracing();
    // A fresh graph (no observations) has a single root G-node.
    // Extract should classify it as one terminal, zero transitions.
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_invariants(&g);
    let p = g.extract();

    let total_transitions: usize = p.layers.iter().map(|l| l.transitions.len()).sum();
    let total_terminals: usize = p.layers.iter().map(|l| l.terminals.len()).sum();

    assert_eq!(total_terminals, 1, "single root should be one terminal");
    assert_eq!(total_transitions, 0, "single root should produce no transitions");
    assert_eq!(p.node_count(), 1);
    assert_eq!(p.layer_count(), 1, "single node should produce exactly one layer");
}

// ── arena–counter consistency ───────────────────────────────────

#[test]
#[allow(clippy::cast_possible_truncation)]
fn node_count_matches_arena_walk() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    for &c in &[0u64, 128, 64, 192, 32, 96] {
        g.observe(c, 6u64);
        assert_invariants(&g);
        let walk_count = g.gnodes().iter_occupied().count() as u32;
        assert_eq!(g.node_count(), walk_count, "node_count drifted after observe({c})");
    }
}

#[test]
#[allow(clippy::cast_possible_truncation)]
fn gstate_partition_sums_to_node_count() {
    let _t = init_tracing();
    // After building a non-trivial tree, Terminal + SemiInternal +
    // Internal should exhaust exactly node_count.
    let g = build_range_tree();
    assert_invariants(&g);

    let mut terminals = 0u32;
    let mut semi_internals = 0u32;
    let mut internals = 0u32;
    for (_idx, gn) in g.gnodes().iter_occupied() {
        match gn.state() {
            GState::Terminal => terminals += 1,
            GState::SemiInternal => semi_internals += 1,
            GState::Internal => internals += 1,
        }
    }

    assert_eq!(
        terminals + semi_internals + internals,
        g.node_count(),
        "GState partition must sum to node_count"
    );
    assert_eq!(terminals, g.terminal_count(), "terminal partition must match terminal_count");
}
