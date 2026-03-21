// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! D5: Invariant checker self-tests (ADR-M-039).
//!
//! "Who watches the watchmen?" — these tests construct deliberately
//! invalid graph states and assert that [`check_all_invariants()`]
//! reports the specific expected violation.  One test per invariant
//! family.
//!
//! Two helper graphs are used throughout: [`split_graph`] (minimal,
//! 3+ G-nodes after a single observation) and [`complex_graph`]
//! (richer, with structural V-nodes from a 256-coordinate spread).
//! Each test mutates exactly one field and checks that the returned
//! error list contains the expected invariant tag.
//!
//! # Test index
//!
//! ## Sanity
//!
//! | Test | Focus |
//! |------|-------|
//! | [`clean_graph_no_errors`] | clean split graph passes all checks |
//! | [`clean_complex_graph_no_errors`] | clean complex graph passes all checks |
//!
//! ## G-tree invariants
//!
//! | Test | Invariant |
//! |------|-----------|
//! | [`catches_g_i1_summation`] | G-I1 (summation) |
//! | [`catches_g_i4_entry_consistency`] | G-I4 (entry consistency) |
//!
//! ## V-tree invariants
//!
//! | Test | Invariant |
//! |------|-----------|
//! | [`catches_v_i1_structural_sum`] | V-I1 (structural sum) |
//! | [`catches_v_i2_branching_factor`] | V-I2 (branching factor) |
//! | [`catches_v_i3_max_uncle`] | V-I3 (max-uncle) |
//! | [`catches_v_i6_exposed_flag`] | V-I6 (exposed flag) |
//! | [`catches_v_i6b_evictable_flag`] | V-I6b (evictable flag) |
//! | [`catches_v_i7_structural_has_evictable`] | V-I7 (`has_evictable` flag) |
//!
//! ## Consistency & accounting
//!
//! | Test | Focus |
//! |------|-------|
//! | [`catches_clean_accounting_violation`] | root sum vs total V-entry intensity |
//! | [`catches_g_parent_link_violation`] | G-tree parent link consistency |
//! | [`catches_v_parent_link_violation`] | V-tree parent link consistency |
//! | [`catches_v_root_consistency_violation`] | V-root must have no parent |
//! | [`catches_node_count_mismatch`] | cached node count |
//! | [`catches_terminal_count_mismatch`] | cached terminal count |
//!
//! ## Depth gates & budget
//!
//! | Test | Invariant |
//! |------|-----------|
//! | [`catches_depth_gate_violation`] | D-I3 (depth gate ordering) |
//! | [`catches_hard_budget_violation`] | hard budget (ADR-M-018) |

use crate::GvGraph;
use crate::invariants::check_all_invariants;
use crate::testing::{Plan, budget_config, default_config, run};
use crate::vnode::VKind;

// ── Helpers ─────────────────────────────────────────────────────

/// Build a graph that has had at least one split (3+ G-nodes).
fn split_graph() -> GvGraph<u64, u64, 8> {
    let plan = Plan::new().observe(42, 10u64);
    let g: GvGraph<u64, u64, 8> = run(default_config(), &plan);
    assert!(g.node_count() >= 3, "need a split for this test");
    g
}

/// Build a richer graph with structural V-nodes and multiple splits.
fn complex_graph() -> GvGraph<u64, u64, 8> {
    let plan = Plan::new().spread(256, 6, 20);
    let g: GvGraph<u64, u64, 8> = run(default_config(), &plan);
    assert!(g.node_count() >= 5, "need multiple splits for this test");
    g
}

/// Find the first structural V-node index, panicking if none exists.
fn find_structural(g: &GvGraph<u64, u64, 8>) -> usize {
    g.vnodes
        .iter_occupied()
        .find(|(_, v)| matches!(&v.kind, VKind::Structural { .. }))
        .map(|(idx, _)| idx)
        .expect("graph should contain at least one structural V-node")
}

/// Find the first entry V-node that has a parent, panicking if none.
fn find_entry_with_parent(g: &GvGraph<u64, u64, 8>) -> usize {
    g.vnodes
        .iter_occupied()
        .find(|(_, v)| matches!(&v.kind, VKind::Entry { .. }) && v.parent.is_some())
        .map(|(idx, _)| idx)
        .expect("graph should contain at least one entry V-node with a parent")
}

// ── Sanity: Clean graphs produce no errors ──────────────────────

#[test]
fn clean_graph_no_errors() {
    let g = split_graph();
    let errors = check_all_invariants(&g);
    assert!(errors.is_empty(), "clean split graph should have no errors: {errors:?}");
}

#[test]
fn clean_complex_graph_no_errors() {
    let g = complex_graph();
    let errors = check_all_invariants(&g);
    assert!(errors.is_empty(), "clean complex graph should have no errors: {errors:?}");
}

// ── G-I1: Summation invariant ───────────────────────────────────

/// Corrupt a G-node's `sum` and verify the checker catches it.
#[test]
fn catches_g_i1_summation() {
    let mut g = split_graph();
    let root_idx = g.g_root.index();
    g.gnodes.get_mut(root_idx).sum = 999;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("G-I1")),
        "expected G-I1 violation, got: {errors:?}",
    );
}

// ── G-I4: Entry consistency ─────────────────────────────────────

/// Corrupt a G-node's `own` so it diverges from its V-entry's
/// intensity.
#[test]
fn catches_g_i4_entry_consistency() {
    let mut g = split_graph();

    let gnode_idx = g
        .gnodes
        .iter_occupied()
        .find_map(|(idx, gn)| gn.entry.map(|_| idx))
        .expect("graph should have at least one G-node with an entry");

    g.gnodes.get_mut(gnode_idx).own = 777;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("G-I4")),
        "expected G-I4 violation, got: {errors:?}",
    );
}

// ── V-I1: Structural sum consistency ────────────────────────────

/// Corrupt a structural V-node's intensity so it no longer equals
/// the sum of its children's intensities.
#[test]
fn catches_v_i1_structural_sum() {
    let mut g = complex_graph();
    let idx = find_structural(&g);
    g.vnodes.get_mut(idx).intensity = 77777;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-I1")),
        "expected V-I1 violation, got: {errors:?}",
    );
}

// ── V-I2: Branching factor ──────────────────────────────────────

/// Set a structural V-node's child count to 1 (must be 2 or 3).
#[test]
fn catches_v_i2_branching_factor() {
    let mut g = complex_graph();
    let idx = find_structural(&g);

    if let VKind::Structural { children, .. } = &mut g.vnodes.get_mut(idx).kind {
        children.len = 1;
    }

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-I2")),
        "expected V-I2 violation, got: {errors:?}",
    );
}

// ── V-I3: Max-uncle constraint ──────────────────────────────────

/// Inflate an entry's intensity far beyond any uncle's value.
#[test]
fn catches_v_i3_max_uncle() {
    let mut g = complex_graph();
    let idx = find_entry_with_parent(&g);
    g.vnodes.get_mut(idx).intensity = 99999;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-I3") || e.contains("V-I1")),
        "expected V-I3 or V-I1 violation after inflating entry intensity, got: {errors:?}",
    );
}

// ── V-I6: Exposed flag ─────────────────────────────────────────

/// Flip `is_exposed` on an entry and verify the checker catches the
/// mismatch.
#[test]
fn catches_v_i6_exposed_flag() {
    let mut g = split_graph();

    let (idx, current) = g
        .vnodes
        .iter_occupied()
        .find_map(|(idx, v)| {
            if let VKind::Entry { is_exposed, .. } = &v.kind {
                Some((idx, *is_exposed))
            } else {
                None
            }
        })
        .expect("graph should contain at least one entry V-node");

    if let VKind::Entry { is_exposed, .. } = &mut g.vnodes.get_mut(idx).kind {
        *is_exposed = !current;
    }

    let errors = check_all_invariants(&g);
    // Use the specific error prefix to distinguish V-I6 from V-I6b.
    assert!(
        errors.iter().any(|e| e.contains("V-I6 violated")),
        "expected V-I6 violation after flipping is_exposed, got: {errors:?}",
    );
}

// ── V-I6b: Evictable flag ──────────────────────────────────────

/// Flip `is_evictable` on an entry and verify the checker catches
/// the mismatch.
#[test]
fn catches_v_i6b_evictable_flag() {
    let mut g = split_graph();

    let (idx, current) = g
        .vnodes
        .iter_occupied()
        .find_map(|(idx, v)| {
            if let VKind::Entry { is_evictable, .. } = &v.kind {
                Some((idx, *is_evictable))
            } else {
                None
            }
        })
        .expect("graph should contain at least one entry V-node");

    if let VKind::Entry { is_evictable, .. } = &mut g.vnodes.get_mut(idx).kind {
        *is_evictable = !current;
    }

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-I6b")),
        "expected V-I6b violation after flipping is_evictable, got: {errors:?}",
    );
}

// ── V-I7: Structural has_evictable flag ─────────────────────────

/// Flip `has_evictable` on a structural V-node.
#[test]
fn catches_v_i7_structural_has_evictable() {
    let mut g = complex_graph();
    let idx = find_structural(&g);

    if let VKind::Structural { has_evictable, .. } = &mut g.vnodes.get_mut(idx).kind {
        *has_evictable = !*has_evictable;
    }

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-I7")),
        "expected V-I7 violation after flipping has_evictable, got: {errors:?}",
    );
}

// ── Clean accounting ────────────────────────────────────────────

/// Inflate the G-root's sum so it diverges from the total of all
/// V-entry intensities. (Also triggers G-I1 as a side-effect.)
#[test]
fn catches_clean_accounting_violation() {
    let mut g = split_graph();
    let root_idx = g.g_root.index();
    g.gnodes.get_mut(root_idx).sum = 9999;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("Clean accounting")),
        "expected clean accounting violation, got: {errors:?}",
    );
}

// ── Parent link consistency (G-tree) ────────────────────────────

/// Null-out a child G-node's parent pointer and verify the checker
/// catches the dangling link.
#[test]
fn catches_g_parent_link_violation() {
    let mut g = split_graph();

    let target_idx = g
        .gnodes
        .iter_occupied()
        .find(|(_, gn)| gn.parent.is_some())
        .map(|(idx, _)| idx)
        .expect("split graph should have child G-nodes");

    g.gnodes.get_mut(target_idx).parent = None;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("G-parent link")),
        "expected G-parent link violation, got: {errors:?}",
    );
}

// ── Parent link consistency (V-tree) ────────────────────────────

/// Null-out a child V-node's parent pointer and verify the checker
/// catches the dangling link.
#[test]
fn catches_v_parent_link_violation() {
    let mut g = complex_graph();
    let idx = find_entry_with_parent(&g);
    g.vnodes.get_mut(idx).parent = None;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-parent link")),
        "expected V-parent link violation, got: {errors:?}",
    );
}

// ── V-root consistency ──────────────────────────────────────────

/// Give the V-tree root a parent (should always be `None`).
#[test]
fn catches_v_root_consistency_violation() {
    let mut g = split_graph();

    let v_root_id = g.v_root.expect("split graph should have a v_root");
    g.vnodes.get_mut(v_root_id.index()).parent = Some(v_root_id);

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("V-root") || e.contains("v_root")),
        "expected V-root consistency violation, got: {errors:?}",
    );
}

// ── Node count consistency ──────────────────────────────────────

/// Corrupt the cached `node_count` and verify the checker catches it.
#[test]
fn catches_node_count_mismatch() {
    let mut g = split_graph();
    g.node_count = 999;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("Node count")),
        "expected node count mismatch, got: {errors:?}",
    );
}

// ── Terminal count consistency ───────────────────────────────────

/// Corrupt the cached `terminal_count`.
#[test]
fn catches_terminal_count_mismatch() {
    let mut g = split_graph();
    g.terminal_count = 999;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("Terminal count")),
        "expected terminal count mismatch, got: {errors:?}",
    );
}

// ── D-I3: Depth gate invariants ─────────────────────────────────

/// Violate D-I3 by setting `live_depth_create >= live_depth_evict`.
#[test]
fn catches_depth_gate_violation() {
    let mut g = split_graph();
    g.live_depth_create = g.live_depth_evict;

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("D-I3")),
        "expected D-I3 violation, got: {errors:?}",
    );
}

// ── Hard budget (ADR-M-018) ─────────────────────────────────────

/// Set a tiny budget below the actual node count.
#[test]
fn catches_hard_budget_violation() {
    let plan = Plan::new().observe(42, 10u64);
    let mut g: GvGraph<u64, u64, 8> = run(budget_config(200), &plan);

    // Shrink the budget below the actual node count.
    g.config.budget = Some(1);

    let errors = check_all_invariants(&g);
    assert!(
        errors.iter().any(|e| e.contains("Hard budget")),
        "expected hard budget violation, got: {errors:?}",
    );
}
