// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::graph::{Config, GvGraph};
use crate::invariants::assert_invariants;
use crate::traits::Inspectable;

/// Build a `GvGraph<u64, u64, 4>` (domain `[0, 16)`) and populate
/// it with known observations to create a multi-depth tree.
fn make_populated_graph() -> GvGraph<u64, u64, 4> {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    // Accumulate enough to trigger splits at multiple depths.
    for _ in 0..10 {
        g.observe(2u64, 10u64);
        g.observe(10u64, 5u64);
        g.observe(14u64, 3u64);
    }
    assert_invariants(&g);
    g
}

/// Build a graph with sufficient structure for subtree tests:
/// observations in left half and right half.
fn make_subtree_graph() -> GvGraph<u64, u64, 4> {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    for _ in 0..10 {
        g.observe(2u64, 20u64);
        g.observe(6u64, 10u64);
        g.observe(10u64, 15u64);
        g.observe(14u64, 8u64);
    }
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
            Inspectable::to_f64_approx(gn.own) * factor(depth)
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

// ── Panics ──────────────────────────────────────────────────

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
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    for _ in 0..10 {
        g.observe(2.0, 10.0);
        g.observe(10.0, 5.0);
    }
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
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(Config {
        split_threshold: 1000,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(5u64, 100u64);
    assert_invariants(&g);
    assert_eq!(g.node_count(), 1);

    g.decay(g.g_root(), 0.5, 0.0);
    assert_eq!(g.gnodes().get(g.g_root().index()).own, 50);
    assert_eq!(g.gnodes().get(g.g_root().index()).sum, 50);
    assert_invariants(&g);
}

#[test]
fn decay_single_node_selective() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(Config {
        split_threshold: 1000,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(5u64, 100u64);
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
