// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Worked example integration test (§IDEA M-16).
//!
//! Exercises the three-observation sequence from the spec on a
//! `GvGraph<u64, u64, 3>` covering domain `[0, 8)` with θ = 5,
//! `D_create` = 3, `D_evict` = 6.  Each step is tested in isolation
//! for G-tree structure, V-tree shape, and node counts, and then
//! the full sequence is replayed in one shot as a cross-check.
//!
//! | Step | Observation      | Mechanism                                      |
//! |------|------------------|------------------------------------------------|
//! |  1   | `observe(3, 10)` | bootstrap split                                |
//! |  2   | `observe(3, 15)` | catalytic split + violation + skip-promote      |
//! |  3   | `observe(6,  8)` | catalytic split, no violation (uncle shield)    |
//!
//! **Expanded coverage:** additional steps exercising routing into
//! deeper subtrees, energy conservation across all steps, atomic-
//! interval splits, serialisation round-trips, and depth-gate
//! stability.
//!
//! # Test index
//!
//! ## Step 1 — bootstrap split
//!
//! | Test | Focus |
//! |------|-------|
//! | [`step1_g_tree_structure`] | root own/sum, two children at `[0,4)` and `[4,8)` |
//! | [`step1_v_tree_structure`] | SR is a 2-node; root entry frozen, L/R entries terminal |
//! | [`step1_node_count_and_violations`] | 3 nodes, no pending violations |
//!
//! ## Step 2 — catalytic split + skip-promote
//!
//! | Test | Focus |
//! |------|-------|
//! | [`step2_g_tree_structure`] | L gains children LL/LR; R still terminal |
//! | [`step2_v_tree_after_skip_promote`] | SR becomes 3-node; L.entry promoted to depth 1 |
//! | [`step2_node_count_and_violations`] | 5 nodes, violations drained by rebalance |
//!
//! ## Step 3 — catalytic split, uncle shield
//!
//! | Test | Focus |
//! |------|-------|
//! | [`step3_g_tree_structure`] | R gains children RL/RR; total sum = 33 |
//! | [`step3_v_tree_uncle_shield`] | L.entry stays at depth 1 (uncle shield); R.entry frozen |
//! | [`step3_final_node_count`] | 7 nodes, no pending violations |
//!
//! ## Full sequence & energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`worked_example_full_sequence`] | end-to-end replay; aggregate sums, depths, V-root shape |
//! | [`energy_conserved_at_every_step`] | `total_sum()` matches cumulative delta after each observe |
//!
//! ## Continued observations (step 4)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`step4_routing_into_deep_subtree`] | route into LL without split (4 < θ) |
//! | [`step4_split_into_atomic_interval`] | LL splits into `[0,1)` and `[1,2)` (atomic for u64) |
//! | [`step4_v_tree_after_atomic_split`] | LL.entry frozen; LLL/LLR terminal at max depth |
//!
//! ## Serialisation & invariants
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plan_serializes_and_deserializes`] | JSON round-trip produces identical graph (feature = "serde") |
//! | [`invariants_hold_at_every_step`] | `run_checked` validates invariants after every observation |
//! | [`depth_gates_unchanged_without_budget`] | `D_create`, `D_evict`, soft limit stay at initial values |

use crate::graph::GvGraph;
use crate::handle::VNodeId;
use crate::invariants::assert_invariants;
use crate::testing::{Plan, run, run_checked, worked_example_config};
use crate::tests::init_tracing;
use crate::vnode::VKind;
use crate::vtree::v_depth;

/// Build the plan for §IDEA M-16 step 1.
fn step1_plan() -> Plan<u64, u64> {
    Plan::new().observe(3, 10)
}

/// Build the plan for steps 1–2.
fn step2_plan() -> Plan<u64, u64> {
    step1_plan().observe(3, 15)
}

/// Build the plan for steps 1–3.
fn step3_plan() -> Plan<u64, u64> {
    step2_plan().observe(6, 8)
}

/// Return the child count of a V-structural node (0 for entries).
fn v_child_count(graph: &GvGraph<u64, u64, 3>, v_id: VNodeId) -> usize {
    match &graph.vnodes().get(v_id.index()).kind {
        VKind::Structural { children, .. } => children.len(),
        VKind::Entry { .. } => 0,
    }
}

/// Assert a V-entry has the expected exposed flag.
fn assert_terminal(graph: &GvGraph<u64, u64, 3>, v_id: VNodeId, expected: bool) {
    match &graph.vnodes().get(v_id.index()).kind {
        VKind::Entry { is_exposed, .. } => {
            assert_eq!(*is_exposed, expected, "exposed flag mismatch at {v_id:?}");
        }
        VKind::Structural { .. } => panic!("expected VKind::Entry at {v_id:?}"),
    }
}

// ── Step 1: observe(3, 10) — bootstrap split ────────────────────

#[test]
fn step1_g_tree_structure() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step1_plan());

    let root = g.gnodes().get(g.g_root().index());
    assert_eq!(root.own, 10, "root.own after observe(3,10)");
    assert_eq!(root.sum, 10, "root.sum after observe(3,10)");

    // Two G-children created by bootstrap split.
    let l_id = root.left.expect("root should have left child");
    let r_id = root.right.expect("root should have right child");
    let l = g.gnodes().get(l_id.index());
    let r = g.gnodes().get(r_id.index());

    assert_eq!((l.lo, l.hi), (0, 4));
    assert_eq!((r.lo, r.hi), (4, 8));
    assert_eq!((l.own, l.sum), (0, 0));
    assert_eq!((r.own, r.sum), (0, 0));
    assert_invariants(&g);
}

#[test]
fn step1_v_tree_structure() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step1_plan());

    // V-root is a structural 2-node (SR).
    let sr = g.v_root().expect("v_root should exist");
    assert_eq!(v_child_count(&g, sr), 2, "SR should be a 2-node");
    assert_eq!(g.vnodes().get(sr.index()).intensity, 10);

    // Root entry: intensity 10, frozen (terminal=false).
    let root_entry = g.gnodes().get(g.g_root().index()).entry.unwrap();
    assert_eq!(g.vnodes().get(root_entry.index()).intensity, 10);
    assert_terminal(&g, root_entry, false);

    // L and R entries: intensity 0, terminal=true.
    let l_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let r_id = g.gnodes().get(g.g_root().index()).right.unwrap();
    let l_entry = g.gnodes().get(l_id.index()).entry.unwrap();
    let r_entry = g.gnodes().get(r_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(l_entry.index()).intensity, 0);
    assert_eq!(g.vnodes().get(r_entry.index()).intensity, 0);
    assert_terminal(&g, l_entry, true);
    assert_terminal(&g, r_entry, true);
    assert_invariants(&g);
}

#[test]
fn step1_node_count_and_violations() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step1_plan());
    assert_eq!(g.node_count(), 3, "root + 2 children");
    assert!(!g.has_pending_violations(), "no violations after step 1");
    assert_invariants(&g);
}

// ── Step 2: observe(3, 15) — catalytic split + skip-promote ─────

#[test]
fn step2_g_tree_structure() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step2_plan());

    let root = g.gnodes().get(g.g_root().index());
    assert_eq!(root.own, 10);
    assert_eq!(root.sum, 25, "10 + 15 + 0");

    let l = g.gnodes().get(root.left.unwrap().index());
    assert_eq!((l.own, l.sum), (15, 15));

    // L now has children LL=[0,2) and LR=[2,4).
    let ll = g.gnodes().get(l.left.expect("L should have left child").index());
    let lr = g.gnodes().get(l.right.expect("L should have right child").index());
    assert_eq!((ll.lo, ll.hi), (0, 2));
    assert_eq!((lr.lo, lr.hi), (2, 4));
    assert_eq!((ll.own, ll.sum), (0, 0));
    assert_eq!((lr.own, lr.sum), (0, 0));

    // R still terminal.
    let r = g.gnodes().get(root.right.unwrap().index());
    assert!(r.left.is_none());
    assert!(r.right.is_none());
    assert_eq!((r.own, r.sum), (0, 0));
    assert_invariants(&g);
}

#[test]
fn step2_v_tree_after_skip_promote() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step2_plan());

    // SR is now a 3-node (absorbed L and m₁ after skip-promote).
    let sr = g.v_root().unwrap();
    assert_eq!(v_child_count(&g, sr), 3);
    assert_eq!(g.vnodes().get(sr.index()).intensity, 25);

    // L.entry: depth 1, intensity 15, frozen.
    let l_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let l_entry = g.gnodes().get(l_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(l_entry.index()).intensity, 15);
    assert_eq!(v_depth(g.vnodes(), l_entry), 1, "L promoted to depth 1");
    assert_terminal(&g, l_entry, false);

    // root.entry: depth 1, intensity 10, frozen.
    let root_entry = g.gnodes().get(g.g_root().index()).entry.unwrap();
    assert_eq!(g.vnodes().get(root_entry.index()).intensity, 10);
    assert_eq!(v_depth(g.vnodes(), root_entry), 1);
    assert_terminal(&g, root_entry, false);

    // R.entry: depth 2, intensity 0, terminal.
    let r_id = g.gnodes().get(g.g_root().index()).right.unwrap();
    let r_entry = g.gnodes().get(r_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(r_entry.index()).intensity, 0);
    assert_eq!(v_depth(g.vnodes(), r_entry), 2);
    assert_terminal(&g, r_entry, true);

    // LL.entry, LR.entry: depth 3, intensity 0, terminal.
    let left_node = g.gnodes().get(l_id.index());
    let left_left_entry = g.gnodes().get(left_node.left.unwrap().index()).entry.unwrap();
    let left_right_entry = g.gnodes().get(left_node.right.unwrap().index()).entry.unwrap();
    assert_eq!(v_depth(g.vnodes(), left_left_entry), 3);
    assert_eq!(v_depth(g.vnodes(), left_right_entry), 3);
    assert_terminal(&g, left_left_entry, true);
    assert_terminal(&g, left_right_entry, true);
    assert_invariants(&g);
}

#[test]
fn step2_node_count_and_violations() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step2_plan());
    assert_eq!(g.node_count(), 5, "root + L + R + LL + LR");
    assert!(!g.has_pending_violations(), "violations drained by rebalance");
    assert_invariants(&g);
}

// ── Step 3: observe(6, 8) — catalytic split, uncle shield ───────

#[test]
fn step3_g_tree_structure() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step3_plan());

    let root = g.gnodes().get(g.g_root().index());
    assert_eq!(root.own, 10);
    assert_eq!(root.sum, 33, "10 + 15 + 8");

    let r = g.gnodes().get(root.right.unwrap().index());
    assert_eq!((r.own, r.sum), (8, 8));

    // R now has children RL=[4,6) and RR=[6,8).
    let rl = g.gnodes().get(r.left.expect("R should have children").index());
    let rr = g.gnodes().get(r.right.unwrap().index());
    assert_eq!((rl.lo, rl.hi), (4, 6));
    assert_eq!((rr.lo, rr.hi), (6, 8));
    assert_eq!((rl.own, rl.sum), (0, 0));
    assert_eq!((rr.own, rr.sum), (0, 0));
    assert_invariants(&g);
}

#[test]
fn step3_v_tree_uncle_shield() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step3_plan());

    // SR still a 3-node, intensity updated to 33.
    let sr = g.v_root().unwrap();
    assert_eq!(v_child_count(&g, sr), 3);
    assert_eq!(g.vnodes().get(sr.index()).intensity, 33);

    // L.entry still at depth 1 — the uncle shield.
    let l_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let l_entry = g.gnodes().get(l_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(l_entry.index()).intensity, 15);
    assert_eq!(v_depth(g.vnodes(), l_entry), 1);

    // R.entry: intensity 8, frozen after split.
    let r_id = g.gnodes().get(g.g_root().index()).right.unwrap();
    let r_entry = g.gnodes().get(r_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(r_entry.index()).intensity, 8);
    assert_terminal(&g, r_entry, false);

    // R's parent (m₁) is now a 3-node (had R + s₁, added s₂).
    let m1 = g.vnodes().get(r_entry.index()).parent.unwrap();
    assert_eq!(v_child_count(&g, m1), 3);
    assert_eq!(g.vnodes().get(m1.index()).intensity, 8);

    // RL, RR entries at depth 3, intensity 0, terminal.
    let right_node = g.gnodes().get(r_id.index());
    let right_left_entry = g.gnodes().get(right_node.left.unwrap().index()).entry.unwrap();
    let right_right_entry = g.gnodes().get(right_node.right.unwrap().index()).entry.unwrap();
    assert_eq!(g.vnodes().get(right_left_entry.index()).intensity, 0);
    assert_eq!(g.vnodes().get(right_right_entry.index()).intensity, 0);
    assert_eq!(v_depth(g.vnodes(), right_left_entry), 3);
    assert_eq!(v_depth(g.vnodes(), right_right_entry), 3);
    assert_terminal(&g, right_left_entry, true);
    assert_terminal(&g, right_right_entry, true);
    assert_invariants(&g);
}

#[test]
fn step3_final_node_count() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step3_plan());
    assert_eq!(g.node_count(), 7, "root + L + R + LL + LR + RL + RR");
    assert!(!g.has_pending_violations());
    assert_invariants(&g);
}

// ── Full sequence in one shot ───────────────────────────────────

#[test]
fn worked_example_full_sequence() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step3_plan());

    // Aggregate outcomes (public API + internal cross-check).
    assert_eq!(g.total_sum(), 33);
    assert_eq!(g.gnodes().get(g.g_root().index()).sum, 33);
    assert_eq!(g.node_count(), 7);

    // L.entry rose to depth 1 (skip-promote resolved violation).
    let l_entry = g
        .gnodes()
        .get(g.gnodes().get(g.g_root().index()).left.unwrap().index())
        .entry
        .unwrap();
    assert_eq!(v_depth(g.vnodes(), l_entry), 1);

    // R.entry at depth 2, shielded by L(15).
    let r_entry = g
        .gnodes()
        .get(g.gnodes().get(g.g_root().index()).right.unwrap().index())
        .entry
        .unwrap();
    assert_eq!(g.vnodes().get(r_entry.index()).intensity, 8);
    assert_eq!(v_depth(g.vnodes(), r_entry), 2);

    // Final V-root: 3-node with intensity 33.
    let sr = g.v_root().unwrap();
    assert_eq!(v_child_count(&g, sr), 3);
    assert_eq!(g.vnodes().get(sr.index()).intensity, 33);

    assert!(!g.has_pending_violations());
    assert_invariants(&g);
}

// ── Expanded coverage: energy conservation ──────────────────────

#[test]
fn energy_conserved_at_every_step() {
    let _t = init_tracing();
    let plan = step3_plan();
    let cfg = worked_example_config();

    // Check after each observation via the public total_sum() API.
    let mut g = GvGraph::<u64, u64, 3>::new(cfg);
    let mut total: u64 = 0;
    for &(coord, delta) in &plan.observations {
        total += delta;
        g.observe(coord, delta);
        assert_eq!(g.total_sum(), total, "energy conservation violated at total={total}");
    }
    assert_invariants(&g);
}

// ── Expanded: continued observations after the spec sequence ────

#[test]
fn step4_routing_into_deep_subtree() {
    let _t = init_tracing();
    // After steps 1–3, observe into the deepest terminal (LL=[0,2)).
    let plan = step3_plan().observe(1, 4);
    let g = run::<u64, u64, 3>(worked_example_config(), &plan);

    // LL.entry should have intensity=4, still terminal, still at depth 3.
    let left_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let left_left_id = g.gnodes().get(left_id.index()).left.unwrap();
    let ll = g.gnodes().get(left_left_id.index());
    assert_eq!(ll.own, 4);
    assert_eq!(ll.sum, 4);
    // No split (4 < θ=5).
    assert!(ll.left.is_none());

    // Total energy: 10 + 15 + 8 + 4 = 37.
    assert_eq!(g.total_sum(), 37);
    assert_invariants(&g);
}

#[test]
fn step4_split_into_atomic_interval() {
    let _t = init_tracing();
    // Observe enough into LL=[0,2) to trigger a split.
    // LL.entry is at v_depth=3, D_create=3, so depth gate passes (3 > 3 is false).
    // After split: LL has children [0,1) and [1,2) — atomic for u64.
    let plan = step3_plan().observe(1, 6);
    let g = run::<u64, u64, 3>(worked_example_config(), &plan);

    let left_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let left_left_id = g.gnodes().get(left_id.index()).left.unwrap();
    let ll = g.gnodes().get(left_left_id.index());

    // Split must occur: 6 > θ=5 and depth gate allows it.
    let left_child = ll.left.expect("LL should have split — left child missing");
    let right_child = ll.right.expect("LL should have split — right child missing");
    let lll = g.gnodes().get(left_child.index());
    let llr = g.gnodes().get(right_child.index());
    assert_eq!((lll.lo, lll.hi), (0, 1));
    assert_eq!((llr.lo, llr.hi), (1, 2));
    assert_eq!((lll.own, lll.sum), (0, 0));
    assert_eq!((llr.own, llr.sum), (0, 0));
    assert_eq!(g.node_count(), 9, "root + L + R + LL + LR + RL + RR + LLL + LLR");
    assert_invariants(&g);
}

#[test]
fn step4_v_tree_after_atomic_split() {
    let _t = init_tracing();
    let plan = step3_plan().observe(1, 6);
    let g = run::<u64, u64, 3>(worked_example_config(), &plan);

    // LL.entry: frozen (split occurred), intensity 6.
    let left_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let left_left_id = g.gnodes().get(left_id.index()).left.unwrap();
    let ll_entry = g.gnodes().get(left_left_id.index()).entry.unwrap();
    assert_eq!(g.vnodes().get(ll_entry.index()).intensity, 6);
    assert_terminal(&g, ll_entry, false);

    // LLL and LLR entries: terminal, intensity 0, at maximum depth.
    let ll = g.gnodes().get(left_left_id.index());
    let left_left_left_entry = g.gnodes().get(ll.left.unwrap().index()).entry.unwrap();
    let left_left_right_entry = g.gnodes().get(ll.right.unwrap().index()).entry.unwrap();
    assert_eq!(g.vnodes().get(left_left_left_entry.index()).intensity, 0);
    assert_eq!(g.vnodes().get(left_left_right_entry.index()).intensity, 0);
    assert_terminal(&g, left_left_left_entry, true);
    assert_terminal(&g, left_left_right_entry, true);

    // Total energy: 10 + 15 + 8 + 6 = 39.
    assert_eq!(g.total_sum(), 39);
    assert_invariants(&g);
}

// ── Expanded: plan serialization round-trip ─────────────────────

#[test]
#[cfg(feature = "serde")]
fn plan_serializes_and_deserializes() {
    let _t = init_tracing();
    let plan = step3_plan();
    let json = serde_json::to_string(&plan).unwrap();
    let restored: Plan<u64, u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(plan, restored);

    // Execute both and verify identical outcome.
    let g1 = run::<u64, u64, 3>(worked_example_config(), &plan);
    let g2 = run::<u64, u64, 3>(worked_example_config(), &restored);
    assert_eq!(g1.node_count(), g2.node_count());
    assert_eq!(g1.total_sum(), g2.total_sum());
}

// ── Expanded: invariants checked after every observation ────────

#[test]
fn invariants_hold_at_every_step() {
    let _t = init_tracing();
    let plan = step3_plan();
    let g = run_checked::<u64, u64, 3>(worked_example_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: depth gates remain static without budget ──────────

#[test]
fn depth_gates_unchanged_without_budget() {
    let _t = init_tracing();
    let g = run::<u64, u64, 3>(worked_example_config(), &step3_plan());
    assert_eq!(g.depth_evict(), 6, "D_evict should stay at initial");
    assert_eq!(g.depth_create(), 3, "D_create should stay at initial");
    assert_eq!(g.soft_limit(), None, "no soft limit without budget");
}
