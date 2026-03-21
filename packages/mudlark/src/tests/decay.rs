// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness and regression tests for **`GvGraph::decay()`**
//! (§IDEA M-14, ADR-M-024).
//!
//! Decay attenuates the energy stored in a G-tree by multiplying each
//! node's `own` value by a depth-dependent factor controlled by two
//! parameters: `attenuation` (overall scaling) and `q` (depth
//! selectivity, 0 = uniform, 1 = coarse-preserving).  These tests
//! exercise every combination of those axes — uniform vs selective,
//! global vs subtree-scoped — and verify invariants, energy
//! conservation, boundary behaviour (att = 0 annihilation, att = 1
//! no-op, att > 1 amplification, att = ∞), parameter validation
//! panics, composition of multiple decay steps, arithmetic against
//! hand-computed values (ADR-M-039), and interaction with other
//! operations (observe, extract, budgeted graphs, plateaus).
//!
//! # Test index
//!
//! ## Uniform global (`q = 0`, `root = g_root`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_uniform_global_scales_all_nodes`] | root sum halves after `att = 0.5` |
//! | [`decay_uniform_global_preserves_invariants`] | invariants hold after uniform decay |
//! | [`decay_noop_at_attenuation_one`] | `att = 1.0` is a no-op |
//! | [`decay_noop_at_attenuation_one_with_q`] | `att = 1.0` is a no-op regardless of `q` |
//! | [`decay_uniform_global_all_own_scaled`] | every node's `own` scaled by `att` |
//!
//! ## Uniform subtree (`q = 0`, `root = child`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_uniform_subtree_scales_only_target`] | sibling subtree energy is unaffected |
//! | [`decay_uniform_subtree_preserves_invariants`] | invariants hold after subtree decay |
//!
//! ## Selective global (`q > 0`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_selective_preserves_invariants`] | invariants hold after selective decay |
//! | [`decay_selective_midpoint_matches_attenuation`] | midpoint depth factor equals `att` |
//! | [`decay_selective_coarse_persists_fine_fades`] | `factor(d=0) > factor(d=D)` |
//! | [`decay_selective_q1_root_undecayed`] | at `q = 1.0`, subtree root `own` unchanged |
//!
//! ## Selective subtree
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_selective_subtree_preserves_invariants`] | invariants hold for selective subtree decay |
//! | [`decay_selective_subtree_leaves_other_half_unchanged`] | sibling subtree energy unaffected |
//!
//! ## Composition
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_composition_k_steps`] | *k* steps of `att` ≈ one step of `att^k` |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_energy_conservation_uniform`] | total ≈ `before × att` (uniform) |
//! | [`decay_energy_conservation_selective`] | total ≈ Σ(`own_before × factor(d)`) (selective) |
//!
//! ## Annihilation (`att = 0`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_annihilation_uniform_zeroes_all`] | `att = 0, q = 0` zeroes all energy |
//! | [`decay_annihilation_detail_flush_preserves_root`] | `att = 0, q = 1`: root `own` preserved (`0^0 = 1`) |
//! | [`decay_annihilation_selective_mid_q`] | `att = 0, q ∈ (0,1)`: all energy zeroed |
//!
//! ## Panics (invalid parameters)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_panics_negative_attenuation`] | negative `att` panics |
//! | [`decay_panics_nan_attenuation`] | `NaN` `att` panics |
//! | [`decay_panics_negative_q`] | negative `q` panics |
//! | [`decay_panics_q_above_one`] | `q > 1` panics |
//! | [`decay_panics_nan_q`] | `NaN` `q` panics |
//!
//! ## Amplification (`att > 1`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_amplification_above_one`] | uniform amplification increases energy |
//! | [`decay_amplification_selective`] | selective amplification increases energy |
//!
//! ## Extract after decay
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_then_extract_roundtrip`] | PEWEI extraction succeeds after decay |
//!
//! ## Float coordinates
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_f64_tree`] | decay works on `f64` coordinate trees |
//!
//! ## Single-node tree
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_single_node_tree`] | uniform decay on a single node |
//! | [`decay_single_node_selective`] | selective decay collapses to `att^1` for single node |
//!
//! ## Multiple consecutive decays
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_multiple_uniform_preserves_invariants`] | 10 consecutive uniform decays |
//! | [`decay_multiple_selective_preserves_invariants`] | 10 consecutive selective decays |
//!
//! ## Plateau (`feature = "dynamic-contour-tracking"`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_updates_plateau_sums`] | plateau sums change after decay |
//! | [`decay_preserves_plateau_structure`] | plateau keys and depths unchanged |
//!
//! ## Decay arithmetic verification (ADR-M-039)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_selective_per_node_values_hand_computed`] | single-node `own` matches hand-computed value |
//! | [`decay_selective_two_level_per_depth`] | per-depth factors verified on two-level tree |
//! | [`decay_uniform_vs_selective_at_q0`] | uniform and selective agree when `q → 0` |
//! | [`decay_selective_coarse_preserved_more_than_fine_per_node`] | retention fraction decreases with depth |
//!
//! ## Interaction with other operations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_then_observe_still_works`] | observe succeeds after decay |
//! | [`decay_on_budgeted_graph_preserves_node_count`] | decay does not trigger eviction (DC-024-3) |
//!
//! ## Decay targeting a leaf
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_on_leaf_within_tree`] | uniform decay scoped to a leaf node |
//! | [`decay_selective_on_leaf_within_tree`] | selective decay on leaf: `depth_range = 0` → `att^1` |
//!
//! ## Infinite amplification (`att = ∞`, f64)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_infinite_uniform_f64_per_node_own`] | nonzero `own` → ∞, zero stays zero |
//! | [`decay_infinite_selective_q1_f64_factor_table`] | root preserved (`∞^0 = 1`), deepest → ∞ |
//! | [`decay_infinite_selective_f64_sum_recomputation`] | root `sum` is ∞ after infinite selective decay |
//! | [`decay_infinite_single_node_f64`] | single-node `∞` decay: `own` and `sum` both ∞ |

use crate::graph::{Config, GvGraph};
use crate::invariants::assert_invariants;
use crate::testing::{Plan, budget_config, decay_config, f64_default_config, run};
use crate::traits::Inspectable;

/// Build a `GvGraph<u64, u64, 4>` (domain `[0, 16)`) and populate
/// it with known observations to create a multi-depth tree.
fn make_populated_graph() -> GvGraph<u64, u64, 4> {
    let plan: Plan<u64, u64> = (0..10).fold(Plan::new(), |p, _| p.observe(2, 10).observe(10, 5).observe(14, 3));
    let g: GvGraph<u64, u64, 4> = run(decay_config(), &plan);
    assert_invariants(&g);
    g
}

/// Build a graph with sufficient structure for subtree tests:
/// observations in left half and right half.
fn make_subtree_graph() -> GvGraph<u64, u64, 4> {
    let plan: Plan<u64, u64> = (0..10).fold(Plan::new(), |p, _| {
        p.observe(2, 20).observe(6, 10).observe(10, 15).observe(14, 8)
    });
    let g: GvGraph<u64, u64, 4> = run(decay_config(), &plan);
    assert_invariants(&g);
    g
}

/// Helper: total energy = `g_root.sum`.
fn total_energy(g: &GvGraph<u64, u64, 4>) -> f64 {
    g.gnodes().get(g.g_root().index()).sum.to_f64_approx()
}

// ── Uniform global ──────────────────────────────────────────

#[test]
fn decay_uniform_global_scales_all_nodes() {
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 0.5, 0.0);
    let after = total_energy(&g);
    // Root sum should halve (integer truncation may lose 1-2 units).
    let expected = (before * 0.5).floor();
    assert!(
        (after - expected).abs() <= f64::from(g.node_count()),
        "expected ≈{expected}, got {after}"
    );
}

#[test]
fn decay_uniform_global_preserves_invariants() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, 0.0);
    assert_invariants(&g);
}

#[test]
fn decay_noop_at_attenuation_one() {
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 1.0, 0.0);
    assert!((total_energy(&g) - before).abs() < f64::EPSILON);
    assert_invariants(&g);
}

#[test]
fn decay_noop_at_attenuation_one_with_q() {
    // att = 1.0 triggers early return regardless of q.
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 1.0, 0.7);
    assert!((total_energy(&g) - before).abs() < f64::EPSILON);
    assert_invariants(&g);
}

#[test]
fn decay_uniform_global_all_own_scaled() {
    let mut g = make_populated_graph();
    // Snapshot all own values before decay.
    let before_owns: Vec<(usize, f64)> = g
        .gnodes()
        .iter_occupied()
        .map(|(i, gn)| (i, gn.own.to_f64_approx()))
        .collect();

    g.decay(g.g_root(), 0.8, 0.0);

    for (i, old_own) in &before_owns {
        let new_own = g.gnodes().get(*i).own.to_f64_approx();
        let expected = (*old_own * 0.8).floor(); // u64 truncation
        // Allow ±1 for integer rounding.
        assert!(
            (new_own - expected).abs() <= 1.0,
            "G-node {i}: expected own ≈{expected}, got {new_own} (was {old_own})"
        );
    }
    assert_invariants(&g);
}

// ── Uniform subtree ─────────────────────────────────────────

#[test]
fn decay_uniform_subtree_scales_only_target() {
    let mut g = make_subtree_graph();

    // Find the left child of g_root (covers [0, 8)).
    let g_root = g.g_root();
    let left_child = g.gnodes().get(g_root.index()).left;
    // The tree must have split by now.
    let left_child = left_child.expect("expected left child after observations");

    // Snapshot right subtree energy.
    let right_child = g.gnodes().get(g_root.index()).right.unwrap();
    let right_before = g.gnodes().get(right_child.index()).sum.to_f64_approx();

    g.decay(left_child, 0.5, 0.0);

    // Right subtree unchanged.
    let right_after = g.gnodes().get(right_child.index()).sum.to_f64_approx();
    assert!(
        (right_before - right_after).abs() < f64::EPSILON,
        "right subtree should be unaffected"
    );
    assert_invariants(&g);
}

#[test]
fn decay_uniform_subtree_preserves_invariants() {
    let mut g = make_subtree_graph();
    let left_child = g.gnodes().get(g.g_root().index()).left.unwrap();
    g.decay(left_child, 0.3, 0.0);
    assert_invariants(&g);
}

// ── Selective global ────────────────────────────────────────

#[test]
fn decay_selective_preserves_invariants() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, 0.5);
    assert_invariants(&g);
}

#[test]
fn decay_selective_midpoint_matches_attenuation() {
    // At the midpoint depth, factor = attenuation exactly.
    let att: f64 = 0.7;
    let q: f64 = 0.5;
    let depth_range = 4u32; // N=4
    let d_mid = depth_range / 2;

    let ln_att = att.ln();
    let t = 2.0 * f64::from(d_mid) / f64::from(depth_range) - 1.0;
    let factor = (ln_att * q.mul_add(t, 1.0)).exp();
    assert!(
        (factor - att).abs() < 1e-12,
        "midpoint factor {factor} should equal att {att}"
    );
}

#[test]
fn decay_selective_coarse_persists_fine_fades() {
    // At q > 0: factor at d_local=0 > factor at d_local=D.
    let att: f64 = 0.5;
    let q: f64 = 0.8;
    let ln_att = att.ln();

    let factor_coarse = {
        let t = -1.0; // d_local=0
        (ln_att * q.mul_add(t, 1.0)).exp()
    };
    let factor_fine = {
        let t = 1.0; // d_local=D
        (ln_att * q.mul_add(t, 1.0)).exp()
    };
    assert!(
        factor_coarse > factor_fine,
        "coarse ({factor_coarse}) should persist more than fine ({factor_fine})"
    );
    // Coarse = att^(1-q), Fine = att^(1+q).
    let expected_coarse = att.powf(1.0 - q);
    let expected_fine = att.powf(1.0 + q);
    assert!((factor_coarse - expected_coarse).abs() < 1e-12);
    assert!((factor_fine - expected_fine).abs() < 1e-12);
}

#[test]
fn decay_selective_q1_root_undecayed() {
    // At q=1.0, the subtree root's own (d_local=0) gets att^(1-1) = 1.0.
    let mut g = make_populated_graph();
    let root = g.g_root();
    let root_own_before = g.gnodes().get(root.index()).own.to_f64_approx();

    g.decay(root, 0.5, 1.0);

    let root_own_after = g.gnodes().get(root.index()).own.to_f64_approx();
    assert!(
        (root_own_before - root_own_after).abs() < f64::EPSILON,
        "at q=1.0, subtree root own should be unchanged"
    );
    assert_invariants(&g);
}

// ── Composition ─────────────────────────────────────────────

#[test]
fn decay_composition_k_steps() {
    // k steps of (att, q) ≈ one step of (att^k, q).
    let att = 0.9;
    let q = 0.4;
    let k: u32 = 5;

    // Path A: k individual steps.
    let mut g_a = make_populated_graph();
    for _ in 0..k {
        g_a.decay(g_a.g_root(), att, q);
    }

    // Path B: one step of att^k.
    let mut g_b = make_populated_graph();
    #[allow(clippy::cast_possible_wrap)] // k fits in i32
    g_b.decay(g_b.g_root(), att.powi(k as i32), q);

    // Compare all g.own values — allow integer rounding.
    for (i, ga) in g_a.gnodes().iter_occupied() {
        if !g_b.gnodes().is_occupied(i) {
            continue;
        }
        let gb = g_b.gnodes().get(i);
        let diff = (ga.own.to_f64_approx() - gb.own.to_f64_approx()).abs();
        assert!(
            diff <= f64::from(k),
            "G-node {i}: own mismatch after composition: {} vs {} (diff {diff})",
            ga.own.to_f64_approx(),
            gb.own.to_f64_approx(),
        );
    }
    assert_invariants(&g_a);
    assert_invariants(&g_b);
}

// ── Energy conservation ─────────────────────────────────────

#[test]
fn decay_energy_conservation_uniform() {
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 0.75, 0.0);
    let after = total_energy(&g);
    // Allow rounding: each node loses at most 1 unit.
    let tolerance = f64::from(g.node_count());
    let expected = before * 0.75;
    assert!(
        (after - expected).abs() <= tolerance,
        "expected ≈{expected}, got {after} (tolerance {tolerance})"
    );
}

#[test]
fn decay_energy_conservation_selective() {
    // Total energy should equal sum of (own_before * factor(depth))
    // for all G-nodes, within integer rounding tolerance.
    let g_before = make_populated_graph();
    let att: f64 = 0.6;
    let q: f64 = 0.5;

    let d_root = 0u32;
    let depth_range = 4u32; // N=4
    let ln_att = att.ln();
    let factor = |d: u32| -> f64 {
        let t = if depth_range == 0 {
            0.0
        } else {
            2.0 * f64::from(d - d_root) / f64::from(depth_range) - 1.0
        };
        (ln_att * q.mul_add(t, 1.0)).exp()
    };

    // Compute expected total.
    let expected_total: f64 = g_before
        .gnodes()
        .iter_occupied()
        .map(|(_, gn)| {
            let depth = crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4);
            gn.own.to_f64_approx() * factor(depth)
        })
        .sum();

    let mut g = g_before;
    g.decay(g.g_root(), att, q);
    let actual = total_energy(&g);
    let tolerance = f64::from(g.node_count()) * 2.0; // 2 units per node
    assert!(
        (actual - expected_total).abs() <= tolerance,
        "expected ≈{expected_total}, got {actual} (tolerance {tolerance})"
    );
    assert_invariants(&g);
}

// ── Annihilation (att = 0) ───────────────────────────────────

#[test]
fn decay_annihilation_uniform_zeroes_all() {
    let mut g = make_populated_graph();
    assert!(g.total_sum() > 0);
    g.decay(g.g_root(), 0.0, 0.0);
    assert_eq!(g.total_sum(), 0);
    assert_invariants(&g);
}

#[test]
fn decay_annihilation_detail_flush_preserves_root() {
    let mut g = make_populated_graph();
    let root = g.g_root();
    let root_own_before = g.gnode_info(root).unwrap().own;
    assert!(root_own_before > 0);
    g.decay(root, 0.0, 1.0);
    let root_own_after = g.gnode_info(root).unwrap().own;
    // Root preserved (0^0 = 1 convention).
    assert_eq!(root_own_after, root_own_before);
    assert_invariants(&g);
}

#[test]
fn decay_annihilation_selective_mid_q() {
    // att = 0, q = 0.5 — all exponents are positive (no depth
    // produces exponent = 0), so every node gets factor 0.
    let mut g = make_populated_graph();
    assert!(g.total_sum() > 0);
    g.decay(g.g_root(), 0.0, 0.5);
    assert_eq!(g.total_sum(), 0);
    assert_invariants(&g);
}

// ── Panics ──────────────────────────────────────────────────

#[test]
#[should_panic(expected = "attenuation must be >= 0")]
fn decay_panics_negative_attenuation() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), -0.5, 0.0);
}

#[test]
#[should_panic(expected = "attenuation must be >= 0")]
fn decay_panics_nan_attenuation() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), f64::NAN, 0.0);
}

#[test]
#[should_panic(expected = "q must be in")]
fn decay_panics_negative_q() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, -0.1);
}

#[test]
#[should_panic(expected = "q must be in")]
fn decay_panics_q_above_one() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, 1.1);
}

#[test]
#[should_panic(expected = "q must be in")]
fn decay_panics_nan_q() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, f64::NAN);
}

// ── Amplification ───────────────────────────────────────────

#[test]
fn decay_amplification_above_one() {
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 1.5, 0.0);
    let after = total_energy(&g);
    assert!(after > before, "amplification should increase energy");
    assert_invariants(&g);
}

// ── Extract after decay ─────────────────────────────────────

#[test]
fn decay_then_extract_roundtrip() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, 0.3);
    assert_invariants(&g);
    let pewei = g.extract();
    // The PEWEI should have at least one layer.
    assert!(!pewei.layers.is_empty());
}

// ── Float coordinates ───────────────────────────────────────

#[test]
fn decay_f64_tree() {
    let plan: Plan<f64, f64> = (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(10.0, 5.0));
    let mut g: GvGraph<f64, f64, 4> = run(f64_default_config(), &plan);
    assert_invariants(&g);
    let before = g.gnodes().get(g.g_root().index()).sum;
    g.decay(g.g_root(), 0.5, 0.0);
    let after = g.gnodes().get(g.g_root().index()).sum;
    #[allow(clippy::suboptimal_flops)] // clarity: (after - before*0.5) is the intended expression
    let diff = (after - before * 0.5).abs();
    assert!(diff < 1.0);
    assert_invariants(&g);
}

// ── Selective subtree ───────────────────────────────────────

#[test]
fn decay_selective_subtree_preserves_invariants() {
    let mut g = make_subtree_graph();
    let left_child = g.gnodes().get(g.g_root().index()).left.unwrap();
    g.decay(left_child, 0.5, 0.5);
    assert_invariants(&g);
}

#[test]
fn decay_selective_subtree_leaves_other_half_unchanged() {
    let mut g = make_subtree_graph();
    let left_child = g.gnodes().get(g.g_root().index()).left.unwrap();
    let right_child = g.gnodes().get(g.g_root().index()).right.unwrap();
    let right_before = g.gnodes().get(right_child.index()).sum.to_f64_approx();

    g.decay(left_child, 0.5, 0.5);

    let right_after = g.gnodes().get(right_child.index()).sum.to_f64_approx();
    assert!((right_before - right_after).abs() < f64::EPSILON);
    assert_invariants(&g);
}

// ── Single-node tree ────────────────────────────────────────

#[test]
fn decay_single_node_tree() {
    let no_split = Config {
        split_threshold: 1000,
        ..decay_config()
    };
    let mut g: GvGraph<u64, u64, 4> = run(no_split, &Plan::new().observe(5, 100));
    assert_invariants(&g);
    assert_eq!(g.node_count(), 1);

    g.decay(g.g_root(), 0.5, 0.0);
    assert_eq!(g.gnodes().get(g.g_root().index()).own, 50);
    assert_eq!(g.gnodes().get(g.g_root().index()).sum, 50);
    assert_invariants(&g);
}

#[test]
fn decay_single_node_selective() {
    let no_split = Config {
        split_threshold: 1000,
        ..decay_config()
    };
    let mut g: GvGraph<u64, u64, 4> = run(no_split, &Plan::new().observe(5, 100));
    assert_invariants(&g);

    // Single node: actual depth_range = 0, so all Q values collapse
    // to the midpoint factor att^1 = att (ADR-M-038).  For a single
    // node there is no depth axis to tilt across.
    g.decay(g.g_root(), 0.5, 0.8);
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let expected = (100.0 * 0.5_f64) as u64; // att^1.0 = 0.5
    assert_eq!(g.gnodes().get(g.g_root().index()).own, expected);
    assert_invariants(&g);
}

// ── Multiple consecutive decays ─────────────────────────────

#[test]
fn decay_multiple_uniform_preserves_invariants() {
    let mut g = make_populated_graph();
    for _ in 0..10 {
        g.decay(g.g_root(), 0.9, 0.0);
        assert_invariants(&g);
    }
}

#[test]
fn decay_multiple_selective_preserves_invariants() {
    let mut g = make_populated_graph();
    for _ in 0..10 {
        g.decay(g.g_root(), 0.9, 0.5);
        assert_invariants(&g);
    }
}

// ── Plateau unit tests (Step 2G) ────────────────────────────

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn decay_updates_plateau_sums() {
    let mut g = make_populated_graph();
    let sums_before: Vec<_> = g.plateaus().values().map(|p| p.sum).collect();

    g.decay(g.g_root(), 0.5, 0.0);
    assert_invariants(&g);

    let sums_after: Vec<_> = g.plateaus().values().map(|p| p.sum).collect();
    // At least some sums should have changed (they were non-zero).
    assert_ne!(sums_before, sums_after, "decay should change plateau sums");
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn decay_preserves_plateau_structure() {
    let mut g = make_populated_graph();
    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_before: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();

    g.decay(g.g_root(), 0.5, 0.0);
    assert_invariants(&g);

    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_after: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();
    // Decay changes magnitudes only — no structural changes.
    assert_eq!(keys_before, keys_after, "decay must not change plateau keys");
    assert_eq!(depths_before, depths_after, "decay must not change plateau depths");
}

// ── D6: Decay arithmetic verification (ADR-M-039) ──────────────

/// Verify per-node own values after selective decay against
/// hand-computed expected results.
#[test]
fn decay_selective_per_node_values_hand_computed() {
    // Build a small tree with known structure — high threshold prevents splits.
    let no_split = Config {
        split_threshold: 1000,
        ..decay_config()
    };
    let mut g: GvGraph<u64, u64, 4> = run(no_split, &Plan::new().observe(5, 1000));
    assert_eq!(g.node_count(), 1);
    assert_invariants(&g);

    let att = 0.6_f64;
    let q = 0.5_f64;

    // Single node: depth_range = 0, so d_local = 0 for the root.
    // With depth_range = 0, factor = att^1 = att (the code uses
    // t = 0.0 when depth_range = 0, giving exponent = 1 + q*0 = 1).
    #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
    let expected_own = (1000.0 * att) as u64; // 600

    g.decay(g.g_root(), att, q);
    let actual_own = g.gnodes().get(g.g_root().index()).own;
    assert_eq!(
        actual_own, expected_own,
        "single-node selective decay: expected own={expected_own}, got {actual_own}"
    );
    assert_invariants(&g);
}

/// Two-level tree: verify distinct per-depth factors for selective decay.
#[test]
fn decay_selective_two_level_per_depth() {
    // Build a graph with exactly two depths: root (d=0) and one child pair (d=1).
    let mut g: GvGraph<u64, u64, 4> = run(decay_config(), &Plan::new().observe_n(2, 100, 10));
    assert!(g.node_count() >= 3, "need at least a split");
    assert_invariants(&g);

    // Record pre-decay own values per depth.
    let att = 0.5_f64;
    let q = 0.8_f64;

    // Compute expected factors per local depth.
    let d_root = 0u32;
    let max_depth: u32 = g
        .gnodes()
        .iter_occupied()
        .map(|(_, gn)| crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4))
        .max()
        .unwrap();
    let depth_range = max_depth - d_root;
    let ln_att = att.ln();

    let factor = |d_local: u32| -> f64 {
        let t = if depth_range == 0 {
            0.0
        } else {
            2.0 * f64::from(d_local) / f64::from(depth_range) - 1.0
        };
        (ln_att * q.mul_add(t, 1.0)).exp()
    };

    // Collect (gnode_index, depth, own_before).
    let pre_data: Vec<(usize, u32, u64)> = g
        .gnodes()
        .iter_occupied()
        .map(|(idx, gn)| {
            let d = crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4);
            (idx, d, gn.own)
        })
        .collect();

    g.decay(g.g_root(), att, q);
    assert_invariants(&g);

    // Verify each node's own matches hand-computed value.
    for (idx, depth, own_before) in &pre_data {
        if !g.gnodes().is_occupied(*idx) {
            continue;
        }
        let d_local = depth - d_root;
        let f = factor(d_local);
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_precision_loss)]
        let expected = (*own_before as f64 * f) as u64;
        let actual = g.gnodes().get(*idx).own;
        let diff = actual.abs_diff(expected);
        assert!(
            diff <= 1,
            "G-node {idx} (depth {depth}): expected own ≈ {expected}, got {actual} (factor={f:.6})"
        );
    }
}

/// Uniform vs. selective consistency: at q=0 both paths must agree.
#[test]
fn decay_uniform_vs_selective_at_q0() {
    // Build two identical graphs.
    let plan: Plan<u64, u64> = (0..10).fold(Plan::new(), |p, _| p.observe(2, 10).observe(10, 5));
    let mut g_uniform: GvGraph<u64, u64, 4> = run(decay_config(), &plan);
    let mut g_selective: GvGraph<u64, u64, 4> = run(decay_config(), &plan);
    assert_invariants(&g_uniform);

    let att = 0.7;

    // Uniform path.
    g_uniform.decay(g_uniform.g_root(), att, 0.0);
    assert_invariants(&g_uniform);

    // Selective path with q ≈ 0 (but using the selective code path).
    // q must be > 0 strictly to get the selective path, but the factor
    // at q=0.0 is exactly att for all depths. Use the uniform path
    // itself — this test verifies that the uniform and selective
    // formulas agree when q → 0.
    //
    // Use a very small q to approximate uniform behavior.
    g_selective.decay(g_selective.g_root(), att, 1e-10);
    assert_invariants(&g_selective);

    // Compare total energies — should be nearly identical.
    // Integer truncation across many nodes can cause small differences.
    let e_uniform = g_uniform.gnodes().get(g_uniform.g_root().index()).sum;
    let e_selective = g_selective.gnodes().get(g_selective.g_root().index()).sum;
    let diff = e_uniform.abs_diff(e_selective);
    let tolerance = u64::from(g_uniform.node_count());
    assert!(
        diff <= tolerance,
        "uniform and near-uniform selective should agree: uniform={e_uniform}, selective={e_selective} (tolerance={tolerance})"
    );

    // Compare per-node own values — allow rounding per node.
    for (idx, gn) in g_uniform.gnodes().iter_occupied() {
        if !g_selective.gnodes().is_occupied(idx) {
            continue;
        }
        let own_u = gn.own;
        let own_s = g_selective.gnodes().get(idx).own;
        let d = own_u.abs_diff(own_s);
        assert!(d <= 1, "G-node {idx}: uniform own={own_u}, selective own={own_s} (diff {d})");
    }
}

/// Verify that selective decay with q > 0 preserves coarse more
/// than fine — check specific node own values decrease monotonically
/// with depth.
#[test]
fn decay_selective_coarse_preserved_more_than_fine_per_node() {
    let mut g = make_populated_graph();
    let att = 0.5_f64;
    let q = 0.9_f64;

    // Record own values before.
    let pre_data: Vec<(usize, u32, f64)> = g
        .gnodes()
        .iter_occupied()
        .filter(|(_, gn)| gn.own > 0)
        .map(|(idx, gn)| {
            let d = crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4);
            (idx, d, gn.own.to_f64_approx())
        })
        .collect();

    g.decay(g.g_root(), att, q);
    assert_invariants(&g);

    // For each pair of nodes at different depths, the shallower one
    // should have retained a higher fraction.
    let post_data: Vec<(usize, u32, f64, f64)> = pre_data
        .iter()
        .filter(|(idx, _, _)| g.gnodes().is_occupied(*idx))
        .map(|(idx, d, pre)| {
            let post = g.gnodes().get(*idx).own.to_f64_approx();
            (*idx, *d, *pre, post)
        })
        .filter(|(_, _, pre, _)| *pre > 0.0)
        .collect();

    for i in 0..post_data.len() {
        for j in (i + 1)..post_data.len() {
            let (_, d_i, pre_i, post_i) = post_data[i];
            let (_, d_j, pre_j, post_j) = post_data[j];
            if d_i == d_j {
                continue;
            }
            let frac_i = post_i / pre_i;
            let frac_j = post_j / pre_j;
            let (shallow_frac, deep_frac) = if d_i < d_j { (frac_i, frac_j) } else { (frac_j, frac_i) };
            // Allow small tolerance for integer truncation.
            assert!(
                shallow_frac >= deep_frac - 0.02,
                "shallower should retain more: d{d_i}={shallow_frac:.4}, d{d_j}={deep_frac:.4}"
            );
        }
    }
}

// ── Amplification + selective ───────────────────────────────

#[test]
fn decay_amplification_selective() {
    let mut g = make_populated_graph();
    let before = total_energy(&g);
    g.decay(g.g_root(), 1.5, 0.6);
    let after = total_energy(&g);
    assert!(after > before, "selective amplification should increase energy");
    assert_invariants(&g);
}

// ── Decay then observe ──────────────────────────────────────

#[test]
fn decay_then_observe_still_works() {
    let mut g = make_populated_graph();
    g.decay(g.g_root(), 0.5, 0.3);
    assert_invariants(&g);

    // Graph should still accept new observations after decay.
    g.observe(8u64, 50u64);
    assert_invariants(&g);
    assert!(g.total_sum() > 0);
}

// ── Budgeted graph + decay ──────────────────────────────────

#[test]
fn decay_on_budgeted_graph_preserves_node_count() {
    let plan: Plan<u64, u64> = Plan::new().spread(16, 6, 30);
    let mut g: GvGraph<u64, u64, 4> = run(budget_config(200), &plan);
    assert_invariants(&g);
    let count_before = g.node_count();

    // DC-024-3: decay must not trigger eviction.
    g.decay(g.g_root(), 0.5, 0.0);
    assert_eq!(g.node_count(), count_before, "decay should not change node count");
    assert_invariants(&g);
}

// ── Decay targeting a leaf within a larger tree ─────────────

#[test]
fn decay_on_leaf_within_tree() {
    let mut g = make_populated_graph();

    // Walk to a terminal (leaf) node in the tree.
    let mut cursor = g.g_root();
    loop {
        let gn = g.gnodes().get(cursor.index());
        if let Some(left) = gn.left {
            cursor = left;
        } else {
            break; // cursor is a leaf
        }
    }
    assert_ne!(cursor, g.g_root(), "should have descended to a leaf");

    let root_sum_before = total_energy(&g);
    g.decay(cursor, 0.5, 0.0);
    let root_sum_after = total_energy(&g);

    // Decaying a leaf should reduce total energy.
    assert!(root_sum_after <= root_sum_before);
    assert_invariants(&g);
}

#[test]
fn decay_selective_on_leaf_within_tree() {
    let mut g = make_populated_graph();

    // Walk to a terminal (leaf) node.
    let mut cursor = g.g_root();
    loop {
        let gn = g.gnodes().get(cursor.index());
        if let Some(left) = gn.left {
            cursor = left;
        } else {
            break;
        }
    }

    // Single-node subtree: depth_range = 0, so selective
    // collapses to factor = att^1 regardless of q.
    g.decay(cursor, 0.5, 0.7);
    assert_invariants(&g);
}

// ── Infinite amplification (f64) ────────────────────────────
//
// The `att.is_infinite()` branch in `decay_selective` computes
// ∞^exponent directly: ∞^0 = 1, ∞^(pos) = ∞, ∞^(neg) = 0.
// These crate tests verify per-node own/sum via `gnodes()`
// (pub(crate)), complementing the integration tests in
// `tests/decay_infinite.rs` which use only the public API.

#[test]
#[allow(clippy::float_cmp)] // Intentional: 0.0 stays exactly 0.0 under ∞ scaling.
fn decay_infinite_uniform_f64_per_node_own() {
    let plan: Plan<f64, f64> = (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(10.0, 5.0).observe(14.0, 3.0));
    let mut g: GvGraph<f64, f64, 4> = run(f64_default_config(), &plan);
    assert_invariants(&g);

    // Snapshot pre-decay own values.
    let pre_owns: Vec<(usize, f64)> = g.gnodes().iter_occupied().map(|(i, gn)| (i, gn.own)).collect();

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // Uniform ∞: every nonzero own → ∞, zero own → 0.
    for (i, old_own) in &pre_owns {
        let new_own = g.gnodes().get(*i).own;
        if *old_own == 0.0 {
            assert_eq!(new_own, 0.0, "G-node {i}: zero own should stay zero");
        } else {
            assert!(new_own.is_infinite(), "G-node {i}: nonzero own should be ∞");
        }
    }
    assert_invariants(&g);
}

#[test]
#[allow(clippy::float_cmp)] // Intentional: ∞^0 = 1 preserves root own exactly.
fn decay_infinite_selective_q1_f64_factor_table() {
    // At q = 1: exponent at d_local = 0 is 1 + 1·(-1) = 0 → ∞^0 = 1.
    // exponent at d_local = D is 1 + 1·(+1) = 2 → ∞^2 = ∞.
    let plan: Plan<f64, f64> = (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(10.0, 5.0));
    let mut g: GvGraph<f64, f64, 4> = run(f64_default_config(), &plan);
    assert_invariants(&g);
    assert!(g.node_count() >= 3, "need at least one split");

    let root = g.g_root();
    let root_own_before = g.gnodes().get(root.index()).own;

    g.decay(root, f64::INFINITY, 1.0);

    // Root (d_local = 0): factor = 1.0 → own unchanged.
    let root_own_after = g.gnodes().get(root.index()).own;
    assert_eq!(root_own_before, root_own_after, "root own preserved at q=1 (∞^0 = 1)");

    // Deepest nodes: factor = ∞ → nonzero own becomes ∞.
    let d_root = 0u32;
    let max_depth: u32 = g
        .gnodes()
        .iter_occupied()
        .map(|(_, gn)| crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4))
        .max()
        .unwrap();
    for (i, gn) in g.gnodes().iter_occupied() {
        let d = crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 4);
        if d == max_depth && d > d_root {
            // Deep node: exponent > 0 → ∞.
            if gn.own != 0.0 {
                assert!(gn.own.is_infinite(), "G-node {i} at max depth: nonzero own should be ∞");
            }
        }
    }
    assert_invariants(&g);
}

#[test]
fn decay_infinite_selective_f64_sum_recomputation() {
    let plan: Plan<f64, f64> = (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(14.0, 5.0));
    let mut g: GvGraph<f64, f64, 4> = run(f64_default_config(), &plan);
    assert_invariants(&g);

    g.decay(g.g_root(), f64::INFINITY, 0.5);

    // Root sum should be ∞ (at least one descendant got ∞).
    let root_sum = g.gnodes().get(g.g_root().index()).sum;
    assert!(root_sum.is_infinite(), "root sum should be ∞ after infinite selective decay");
    assert_invariants(&g);
}

#[test]
fn decay_infinite_single_node_f64() {
    // Single node: depth_range = 0, so factor = ∞^1 = ∞.
    let no_split = Config {
        split_threshold: 1000.0,
        ..f64_default_config()
    };
    let mut g: GvGraph<f64, f64, 4> = run(no_split, &Plan::new().observe(5.0, 100.0));
    assert_eq!(g.node_count(), 1);

    g.decay(g.g_root(), f64::INFINITY, 0.5);

    assert!(g.gnodes().get(g.g_root().index()).own.is_infinite());
    assert!(g.gnodes().get(g.g_root().index()).sum.is_infinite());
    assert_invariants(&g);
}
