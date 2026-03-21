// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Regression and correctness tests for **f64 depth computation** and
//! **f64 decay**.
//!
//! The depth tests were introduced to guard against a specific bug in
//! `gnode_depth_from_interval`: when the interval width is sub-unit
//! (i.e. `hi - lo < 1.0`), the `log2(width)` result is negative, and
//! the original `as u32` cast saturated it to 0 — collapsing deep
//! nodes onto the root level.  The fix casts via `as i32` first so
//! that depths beyond *N* are represented correctly.
//!
//! The decay tests verify the `GvGraph::decay` operation over f64
//! trees: boundary attenuation values (0.0 and 1.0), selective decay
//! (`q > 0`), subtree scoping, energy conservation, and repeated
//! application — all with full invariant checking after every
//! mutation.
//!
//! # Test index
//!
//! ## Depth computation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`f64_depth_regression_sub_unit_intervals`] | `i32` cast fix for sub-unit widths (depth > *N*) |
//! | [`f64_depth_alternating_observations_invariants`] | alternating observations replay + post-decay invariants |
//! | [`f64_depth_hotspot_deep_splits`] | deep-split stress via single-hotspot plan |
//!
//! ## Decay
//!
//! | Test | Focus |
//! |------|-------|
//! | [`f64_decay_noop_at_attenuation_one`] | `att = 1.0` is a no-op |
//! | [`f64_decay_annihilation_zeroes_all`] | `att = 0.0` zeroes all energy |
//! | [`f64_decay_selective_preserves_invariants`] | selective decay (`q > 0`) preserves invariants |
//! | [`f64_decay_multiple_consecutive`] | 10 consecutive selective decays |
//! | [`f64_decay_energy_conservation_uniform`] | uniform decay: total ≈ before × attenuation |
//! | [`f64_decay_subtree_scoped`] | sibling subtree energy is unaffected |
//! | [`f64_decay_selective_q1_root_undecayed`] | at `q = 1.0`, subtree root's own energy is unchanged |

use crate::graph::GvGraph;
use crate::gtree::gnode_depth_from_interval;
use crate::invariants::{assert_invariants, check_all_invariants, dump_gtree, dump_plateaus};
use crate::testing::{Plan, f64_deep_config, f64_default_config, run, run_checked, run_soft_checked};
use crate::tests::init_tracing;
use crate::traits::Coordinate;

// ── Helpers ─────────────────────────────────────────────────────

/// Alternating observation plan: 10 rounds of (2.0, +10.0) then
/// (10.0, +5.0).  Shared across most tests in this module.
fn f64_alternating_plan() -> Plan<f64, f64> {
    (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(10.0, 5.0))
}

/// Build and validate a populated `GvGraph<f64, f64, 4>` from the
/// alternating plan.
fn make_f64_graph() -> GvGraph<f64, f64, 4> {
    let g = run::<f64, f64, 4>(f64_default_config(), &f64_alternating_plan());
    assert_invariants(&g);
    g
}

// ── Depth computation regression ────────────────────────────────

/// Verify that `gnode_depth_from_interval` returns correct depths
/// for f64 intervals, including sub-unit widths that previously
/// saturated at N due to an unsigned cast.
#[test]
fn f64_depth_regression_sub_unit_intervals() {
    let _t = init_tracing();

    // N = 4, domain [0, 16).  Width 2^k → depth N-k.
    let cases: &[(f64, f64, u32)] = &[
        (0.0, 16.0, 0), // width=16 → depth 0
        (0.0, 8.0, 1),  // width=8  → depth 1
        (0.0, 4.0, 2),  // width=4  → depth 2
        (2.0, 4.0, 3),  // width=2  → depth 3
        (2.0, 3.0, 4),  // width=1  → depth 4 (= N)
        (2.0, 2.5, 5),  // width=0.5  → depth 5 (> N, was the bug)
        (2.5, 3.0, 5),  // width=0.5  → depth 5
        (2.0, 2.25, 6), // width=0.25 → depth 6
    ];

    for &(lo, hi, expected_depth) in cases {
        let depth = gnode_depth_from_interval(lo, hi, 4);
        assert_eq!(
            depth,
            expected_depth,
            "[{lo}, {hi}) width={}: expected depth {expected_depth}, got {depth}",
            hi - lo,
        );
    }

    // Key regression check: parent and child must differ in depth.
    let depth_parent = gnode_depth_from_interval(2.0_f64, 3.0_f64, 4);
    let depth_child = gnode_depth_from_interval(2.0_f64, 2.5_f64, 4);
    assert_ne!(
        depth_parent, depth_child,
        "parent [2.0, 3.0) and child [2.0, 2.5) must have different depths"
    );
    assert_eq!(depth_child, depth_parent + 1);
}

// ── Invariant-checked f64 tree construction + decay ─────────────

/// Replay alternating f64 observations with soft invariant checks
/// on every step.  On failure, dumps full G-tree + plateau state
/// for diagnosis.  Then applies uniform decay and re-checks.
#[test]
fn f64_depth_alternating_observations_invariants() {
    let _t = init_tracing();

    let config = f64_default_config();
    let plan = f64_alternating_plan();

    let (mut g, errors) = run_soft_checked::<f64, f64, 4>(config, &plan, 1);

    if !errors.is_empty() {
        for (step, errs) in &errors {
            tracing::debug!("INVARIANT VIOLATION at step {step}:");
            for e in errs {
                tracing::debug!("  ✗ {e}");
            }
        }
        tracing::debug!("\n{}", dump_gtree(&g));
        tracing::debug!("{}", dump_plateaus(&g));

        // Extra diagnosis: flag sub-unit intervals.
        for (i, gn) in g.gnodes().iter_occupied() {
            let lo_f = Coordinate::to_f64(gn.lo);
            let hi_f = Coordinate::to_f64(gn.hi);
            let w = hi_f - lo_f;
            if w < 1.0 {
                let d = gnode_depth_from_interval(gn.lo, gn.hi, 4);
                tracing::debug!("  ⚠ Sub-unit: G({i}) [{lo_f:.3}, {hi_f:.3}) w={w:.4} depth={d}",);
            }
        }

        panic!("Invariant violations during observation: {} failing step(s)", errors.len());
    }

    // Uniform decay — the original trigger for the depth bug.
    g.decay(g.g_root(), 0.5, 0.0);
    let post_decay_errs = check_all_invariants(&g);
    assert!(
        post_decay_errs.is_empty(),
        "Post-decay invariant violations: {post_decay_errs:?}"
    );
}

// ── Deep-split hotspot ──────────────────────────────────────────

/// Feeds 100 observations at a single f64 coordinate to force
/// deep splits into sub-unit-width intervals.  Invariants are
/// checked every step.
#[test]
fn f64_depth_hotspot_deep_splits() {
    let _t = init_tracing();

    let config = f64_deep_config();
    let plan: Plan<f64, f64> = Plan::new().hotspot(2.0, 10.0, 100);

    // run_checked panics on first violation with a clear message.
    let g = run_checked::<f64, f64, 4>(config, &plan, 1);

    // Verify the tree actually grew beyond the root.
    assert!(g.node_count() > 1, "hotspot should have triggered splits");
}

// ── f64 decay noop ──────────────────────────────────────────────

/// Decay with attenuation = 1.0 must leave the tree unchanged.
#[test]
fn f64_decay_noop_at_attenuation_one() {
    let _t = init_tracing();

    let mut g = make_f64_graph();
    let before = g.total_sum();
    g.decay(g.g_root(), 1.0, 0.0);

    assert!(
        (g.total_sum() - before).abs() < f64::EPSILON,
        "decay at att=1.0 should be a no-op"
    );
    assert_invariants(&g);
}

// ── f64 decay annihilation ──────────────────────────────────────

/// Decay with attenuation = 0.0 must zero all energy.
#[test]
fn f64_decay_annihilation_zeroes_all() {
    let _t = init_tracing();

    let mut g = make_f64_graph();
    assert!(g.total_sum() > 0.0);

    g.decay(g.g_root(), 0.0, 0.0);

    assert!(
        g.total_sum().abs() < f64::EPSILON,
        "decay at att=0.0 should zero all energy, got {}",
        g.total_sum()
    );
    assert_invariants(&g);
}

// ── f64 selective decay (q > 0) ─────────────────────────────────

/// Selective decay on an f64 tree preserves all invariants.
#[test]
fn f64_decay_selective_preserves_invariants() {
    let _t = init_tracing();

    let mut g = make_f64_graph();

    g.decay(g.g_root(), 0.6, 0.7);
    assert_invariants(&g);
}

// ── f64 multiple consecutive decays ─────────────────────────────

/// 10 consecutive selective decays on an f64 tree.
#[test]
fn f64_decay_multiple_consecutive() {
    let _t = init_tracing();

    let mut g = make_f64_graph();

    for _ in 0..10 {
        g.decay(g.g_root(), 0.9, 0.4);
        assert_invariants(&g);
    }
}

// ── f64 energy conservation ─────────────────────────────────────

/// After uniform decay (q=0), total energy ≈ before × attenuation.
#[test]
fn f64_decay_energy_conservation_uniform() {
    let _t = init_tracing();

    let mut g = make_f64_graph();

    let before = g.total_sum();
    g.decay(g.g_root(), 0.75, 0.0);
    let after = g.total_sum();

    // f64 accumulator: no integer truncation, tolerance is just
    // floating-point rounding.
    let expected = before * 0.75;
    assert!((after - expected).abs() < 1e-6, "expected ≈{expected}, got {after}");
    assert_invariants(&g);
}

// ── f64 subtree-scoped decay ────────────────────────────────────

/// Decay scoped to one subtree must leave the sibling untouched.
#[test]
fn f64_decay_subtree_scoped() {
    let _t = init_tracing();

    let mut g = make_f64_graph();

    let g_root = g.g_root();
    let left_child = g
        .gnodes()
        .get(g_root.index())
        .left
        .expect("expected left child after observations");
    let right_child = g
        .gnodes()
        .get(g_root.index())
        .right
        .expect("expected right child after observations");

    let right_before = g.gnodes().get(right_child.index()).sum;

    g.decay(left_child, 0.5, 0.0);

    let right_after = g.gnodes().get(right_child.index()).sum;
    assert!(
        (right_before - right_after).abs() < f64::EPSILON,
        "right subtree should be unaffected by left-subtree decay"
    );
    assert_invariants(&g);
}

// ── f64 selective decay q=1 root undecayed ──────────────────────

/// At q = 1.0, the subtree root's own (`d_local` = 0) gets
/// `att^(1-1) = 1.0` — it should be unchanged.
#[test]
fn f64_decay_selective_q1_root_undecayed() {
    let _t = init_tracing();

    let mut g = make_f64_graph();
    let root = g.g_root();
    let root_own_before = g.gnodes().get(root.index()).own;

    g.decay(root, 0.5, 1.0);

    let root_own_after = g.gnodes().get(root.index()).own;
    assert!(
        (root_own_before - root_own_after).abs() < f64::EPSILON,
        "at q=1.0, subtree root own should be unchanged"
    );
    assert_invariants(&g);
}
