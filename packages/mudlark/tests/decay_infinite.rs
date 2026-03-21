// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(clippy::float_cmp)]

//! Integration tests for `decay()` with infinite attenuation
//! and regression tests for the factor-table depth bound bug
//! (ADR-M-038).
//!
//! Infinite amplification (`att = ∞`) is the mirror of annihilation
//! (`att = 0`). The selective path special-cases both extremes to
//! avoid NaN from `ln(∞) * 0 = ∞ * 0`.
//!
//! For f64 accumulators, ∞ is a representable value and the graph
//! remains structurally valid after infinite amplification. For
//! integer accumulators, nonzero nodes saturate to `T::MAX` and
//! multi-node sum recomputation overflows via `checked_add` — so
//! only single-node trees survive.
//!
//! # Test index
//!
//! ## ADR-M-038 regression
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adr038_finite_selective_f64_does_not_panic`] | finite selective decay on deep f64 tree |
//! | [`adr038_infinite_selective_f64_does_not_panic`] | infinite selective decay on deep f64 tree |
//! | [`adr038_annihilation_selective_f64_does_not_panic`] | annihilation (`att = 0`) in selective mode |
//! | [`adr038_selective_various_q_values`] | selective decay with various q values |
//! | [`adr038_deep_split_selective_f64`] | selective decay on deep-split (concentrated) f64 tree |
//! | [`adr038_subtree_selective_f64`] | subtree-scoped selective decay on f64 |
//!
//! ## Uniform ∞ (q=0)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_uniform_f64_total_sum_is_infinite`] | total sum → ∞ after uniform infinite amplification |
//! | [`infinite_uniform_f64_zero_own_preserved`] | zero-own nodes stay zero (`0 × ∞ = 0`) |
//! | [`infinite_uniform_f64_get_returns_infinite_intensity`] | point query returns ∞ intensity |
//! | [`infinite_uniform_f64_plateaus_structure_preserved`] | plateau keys and depths unchanged |
//! | [`infinite_uniform_f64_extract_succeeds`] | PEWEI extraction succeeds on ∞-valued tree |
//!
//! ## Selective ∞ (q>0)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_selective_q1_preserves_root_own`] | `q = 1`: root exponent 0 → `∞⁰ = 1`, root own preserved |
//! | [`infinite_selective_q05_all_infinite`] | `q = 0.5`: all exponents positive → everything ∞ |
//! | [`infinite_selective_f64_with_zero_own`] | zero-own nodes stay zero (no NaN) |
//!
//! ## Subtree-scoped ∞
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_subtree_only_affects_target`] | sibling subtree energy unaffected by scoped ∞ |
//!
//! ## Round-trip ∞↔0
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_then_annihilate_zeros_everything`] | ∞ then annihilate → 0 (`0 × ∞ = 0`) |
//! | [`infinite_then_finite_decay_stays_infinite`] | ∞ then finite → still ∞ |
//! | [`annihilate_then_infinite_amplify_is_still_zero`] | 0 then ∞ → still 0 |
//!
//! ## Post-∞ operations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_after_infinite_amplification`] | observe into ∞-valued tree does not panic |
//! | [`contour_range_after_infinite_amplification`] | contour range query on ∞-valued tree |
//! | [`range_sum_after_infinite_amplification`] | full-domain range sum returns ∞ |
//! | [`sample_after_infinite_amplification`] | sampling from ∞-intensity graph succeeds |
//!
//! ## q=1 detail flood
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_detail_flood_preserves_root_amplifies_descendants`] | root own preserved, descendants → ∞ |
//!
//! ## Single-node trees
//!
//! | Test | Focus |
//! |------|-------|
//! | [`infinite_single_node_f64`] | single f64 node amplified to ∞ |
//! | [`infinite_single_node_u64_saturates`] | single u64 node saturates to `u64::MAX` |
//! | [`infinite_single_node_u64_zero_own_stays_zero`] | single u64 node with zero own stays zero |
//!
//! ## Idempotence / stacking
//!
//! | Test | Focus |
//! |------|-------|
//! | [`double_infinite_amplification`] | `∞ × ∞ = ∞`, no NaN or panic |
//!
//! ## Boundary: att=1.0
//!
//! | Test | Focus |
//! |------|-------|
//! | [`noop_at_attenuation_one`] | `att = 1.0` is a no-op at all q values |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{GraphCreator, f64_default_config};
use torrust_mudlark::{BasisEdge, Config, GvGraph};

mod support;
use support::{TestRng, init_tracing};

// ── Helpers ─────────────────────────────────────────────────────

/// Build an f64 graph with multiple depths and all-nonzero own values.
///
/// Every coordinate that receives observations will have nonzero own;
/// internal nodes created by splits start at zero own but immediately
/// become nonzero via further observations.
fn make_f64_graph() -> GvGraph<f64, f64, 4> {
    GraphCreator::new(f64_default_config())
        .observe_n(2.0, 10.0, 10)
        .observe_n(6.0, 8.0, 10)
        .observe_n(10.0, 5.0, 10)
        .observe_n(14.0, 3.0, 10)
        .check_every(0)
        .build()
}

/// Build an f64 graph that is guaranteed to have zero-own internal
/// nodes (split parents that never received direct observations).
fn make_f64_graph_with_zero_own() -> GvGraph<f64, f64, 4> {
    GraphCreator::new(f64_default_config())
        .hotspot(2.0, 10.0, 20)
        .check_every(0)
        .build()
}

// ── ADR-M-038 regression: selective decay on f64 trees ──────────
//
// The factor table in `decay_selective` was sized to `N - d_root + 1`
// entries, assuming G-tree depth ≤ N. For f64 coordinates,
// `attempt_split` gates on V-tree depth (not G-tree depth), so
// nodes at G-depth > N can exist.  Before the fix, any selective
// decay (q > 0) on such a tree panicked with index-out-of-bounds.

/// Regression: finite selective decay on an f64 graph whose
/// structure extends beyond depth N (the original crash).
#[test]
fn adr038_finite_selective_f64_does_not_panic() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), 0.5, 0.5);
    assert_invariants(&g);
}

/// Regression: infinite selective decay on the same f64 graph.
#[test]
fn adr038_infinite_selective_f64_does_not_panic() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.5);
    assert_invariants(&g);
}

/// Regression: annihilation (att=0) in selective mode on f64.
#[test]
fn adr038_annihilation_selective_f64_does_not_panic() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), 0.0, 0.5);
    assert_eq!(g.total_sum(), 0.0);
    assert_invariants(&g);
}

/// Regression: selective decay with various q values on f64.
#[test]
fn adr038_selective_various_q_values() {
    let _t = init_tracing();
    for &q in &[0.1, 0.3, 0.5, 0.7, 1.0] {
        let mut g = make_f64_graph();
        let sum_before = g.total_sum();
        g.decay(g.g_root(), 0.8, q);
        assert!(g.total_sum() < sum_before, "decay should reduce total at q={q}");
        assert_invariants(&g);
    }
}

/// Regression: selective decay on a deep-split f64 graph
/// (concentrated observations force the deepest possible splits).
#[test]
fn adr038_deep_split_selective_f64() {
    let _t = init_tracing();
    let mut g = make_f64_graph_with_zero_own();
    g.decay(g.g_root(), 0.5, 0.8);
    assert_invariants(&g);
}

/// Regression: subtree-scoped selective decay on f64.
#[test]
fn adr038_subtree_selective_f64() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    let root = g.g_root();
    let root_own_before = g.gnode_info(root).unwrap().own;

    // Find a non-root node.
    let sub_root = g
        .layers()
        .find(|(_, node)| node.gnode_id != root && g.is_ancestor_of(root, node.gnode_id))
        .map(|(_, node)| node.gnode_id)
        .expect("should have non-root nodes");

    g.decay(sub_root, 0.5, 0.5);
    assert_eq!(
        g.gnode_info(root).unwrap().own,
        root_own_before,
        "root own unaffected by subtree decay"
    );
    assert_invariants(&g);
}

// ── Uniform infinite amplification (q = 0) ─────────────────────

#[test]
fn infinite_uniform_f64_total_sum_is_infinite() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    assert!(g.total_sum() > 0.0);

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    assert!(g.total_sum().is_infinite(), "total sum should be ∞");
    assert_invariants(&g);
}

#[test]
fn infinite_uniform_f64_zero_own_preserved() {
    let _t = init_tracing();
    let mut g = make_f64_graph_with_zero_own();

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // Zero-own nodes should stay zero (0.0 * ∞ = 0.0 by Attenuatable contract).
    assert_invariants(&g);
}

#[test]
fn infinite_uniform_f64_get_returns_infinite_intensity() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // Point query should return infinite intensity at observed coords.
    let cell = g.get(2.0);
    assert!(cell.intensity.is_infinite(), "cell intensity should be ∞");
}

#[test]
fn infinite_uniform_f64_plateaus_structure_preserved() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_before: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_after: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();
    assert_eq!(keys_before, keys_after, "plateau keys must not change");
    assert_eq!(depths_before, depths_after, "plateau depths must not change");
    assert_invariants(&g);
}

#[test]
fn infinite_uniform_f64_extract_succeeds() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    let pewei = g.extract();
    assert!(!pewei.layers.is_empty(), "PEWEI should have layers");
    assert_invariants(&g);
}

// ── Selective infinite amplification (q > 0) ────────────────────

#[test]
fn infinite_selective_q1_preserves_root_own() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    let root = g.g_root();
    let root_info_before = g.gnode_info(root).unwrap();

    g.decay(root, f64::INFINITY, 1.0);

    // At q=1, root exponent = 1 - q = 0 → ∞^0 = 1 → root unchanged.
    let root_info_after = g.gnode_info(root).unwrap();
    assert_eq!(
        root_info_before.own, root_info_after.own,
        "root own should be preserved at q = 1"
    );
    // But total sum should be infinite (descendants amplified).
    assert!(g.total_sum().is_infinite(), "total sum should be ∞");
    assert_invariants(&g);
}

#[test]
fn infinite_selective_q05_all_infinite() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    // At q = 0.5: all exponents are positive (min exponent = 1 - 0.5 = 0.5),
    // so ∞^(positive) = ∞ for all depths.
    g.decay(g.g_root(), f64::INFINITY, 0.5);

    assert!(g.total_sum().is_infinite());
    assert_invariants(&g);
}

#[test]
fn infinite_selective_f64_with_zero_own() {
    let _t = init_tracing();
    let mut g = make_f64_graph_with_zero_own();

    g.decay(g.g_root(), f64::INFINITY, 0.7);

    // Should not produce NaN — zero-own nodes stay zero.
    assert_invariants(&g);
}

// ── Subtree-scoped infinite amplification ───────────────────────

#[test]
fn infinite_subtree_only_affects_target() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    let root = g.g_root();
    let root_info = g.gnode_info(root).unwrap();

    // Use layers() to find a non-root gnode_id.
    let sub_root = g
        .layers()
        .find(|(_, node)| node.gnode_id != root && g.is_ancestor_of(root, node.gnode_id))
        .map(|(_, node)| node.gnode_id)
        .expect("multi-depth tree should have non-root nodes");

    let total_before = g.total_sum();
    g.decay(sub_root, f64::INFINITY, 0.0);

    // Total sum should have increased (subtree amplified to ∞).
    assert!(g.total_sum() > total_before || g.total_sum().is_infinite());
    // Root own should be unchanged (not in the target subtree).
    assert_eq!(
        g.gnode_info(root).unwrap().own,
        root_info.own,
        "root own should be unaffected by subtree decay"
    );
    assert_invariants(&g);
}

// ── Round-trip: infinite ↔ annihilate ───────────────────────────

#[test]
fn infinite_then_annihilate_zeros_everything() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    g.decay(g.g_root(), f64::INFINITY, 0.0);
    assert!(g.total_sum().is_infinite());

    // Annihilate: 0.0 * ∞ = 0.0 (by Attenuatable contract).
    g.decay(g.g_root(), 0.0, 0.0);
    assert_eq!(g.total_sum(), 0.0, "annihilation after ∞ should zero everything");
    assert_invariants(&g);
}

#[test]
fn infinite_then_finite_decay_stays_infinite() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    g.decay(g.g_root(), f64::INFINITY, 0.0);
    // ∞ * 0.5 = ∞.
    g.decay(g.g_root(), 0.5, 0.0);
    assert!(g.total_sum().is_infinite(), "∞ * finite should remain ∞");
    assert_invariants(&g);
}

#[test]
fn annihilate_then_infinite_amplify_is_still_zero() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    // First annihilate: everything → 0.
    g.decay(g.g_root(), 0.0, 0.0);
    assert_eq!(g.total_sum(), 0.0);

    // Then infinite amplify: 0 * ∞ = 0 (by contract).
    g.decay(g.g_root(), f64::INFINITY, 0.0);
    assert_eq!(g.total_sum(), 0.0, "amplifying zero should remain zero");
    assert_invariants(&g);
}

// ── Post-∞ operations ───────────────────────────────────────────

#[test]
fn observe_after_infinite_amplification() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // Observing into an infinite tree should not panic.
    g.observe(8.0, 42.0);
    // ∞ + 42 = ∞.
    assert!(g.total_sum().is_infinite());
    assert_invariants(&g);
}

#[test]
fn contour_range_after_infinite_amplification() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // Contour range query should not panic.
    let cr = g.contour_range(BasisEdge(0.0), BasisEdge(16.0));
    let cr = cr.expect("contour range should be Some");
    assert!(!cr.basis.is_empty(), "contour range should have basis elements");
    assert!(cr.energy.is_infinite(), "energy should be ∞");
    assert_invariants(&g);
}

#[test]
fn range_sum_after_infinite_amplification() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    let sum = g.range_sum(..);
    assert!(sum.is_infinite(), "full-domain range_sum should be ∞");
    assert_invariants(&g);
}

#[test]
fn sample_after_infinite_amplification() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    let mut rng = TestRng(42);
    let cell = g.sample(&mut rng);
    // Graph has infinite intensity — sample should succeed.
    assert!(cell.is_some(), "sample on ∞-intensity graph should return Some");
    assert!(cell.unwrap().intensity.is_infinite(), "sampled cell intensity should be ∞");
}

// ── q=1 detail flood (mirror of detail flush) ──────────────────

#[test]
fn infinite_detail_flood_preserves_root_amplifies_descendants() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    let root = g.g_root();
    let root_own_before = g.gnode_info(root).unwrap().own;

    // ∞ at q=1: root gets ∞^0 = 1 (preserved), descendants get ∞.
    // Mirror of annihilation at q=1 (detail flush: root preserved,
    // descendants zeroed).
    g.decay(root, f64::INFINITY, 1.0);

    let root_own_after = g.gnode_info(root).unwrap().own;
    assert_eq!(root_own_before, root_own_after, "detail flood: root own preserved");
    assert!(g.total_sum().is_infinite(), "descendants should be ∞");
    assert_invariants(&g);
}

// ── Single-node trees ───────────────────────────────────────────

#[test]
fn infinite_single_node_f64() {
    let _t = init_tracing();
    let cfg = Config {
        split_threshold: 1000.0,
        ..f64_default_config()
    };
    let mut g = GvGraph::<f64, f64, 4>::new(cfg);
    g.observe(5.0, 100.0);
    assert_eq!(g.node_count(), 1);

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    assert!(g.total_sum().is_infinite());
    assert_eq!(g.node_count(), 1);
    assert_invariants(&g);
}

#[test]
fn infinite_single_node_u64_saturates() {
    let _t = init_tracing();
    let cfg = Config {
        split_threshold: 1000u64,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g = GvGraph::<u64, u64, 4>::new(cfg);
    g.observe(5u64, 100u64);
    assert_eq!(g.node_count(), 1);

    // Single-node: no sum recomputation addition, just own * ∞.
    // (100 as f64 * ∞) as u64 = ∞ as u64 = u64::MAX (saturating cast).
    g.decay(g.g_root(), f64::INFINITY, 0.0);

    assert_eq!(g.total_sum(), u64::MAX);
    assert_invariants(&g);
}

#[test]
fn infinite_single_node_u64_zero_own_stays_zero() {
    let _t = init_tracing();
    let cfg = Config {
        split_threshold: 1000u64,
        depth_create: 3,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g = GvGraph::<u64, u64, 4>::new(cfg);
    // Empty tree: root has own=0, sum=0.
    assert_eq!(g.total_sum(), 0);

    g.decay(g.g_root(), f64::INFINITY, 0.0);

    // (0 as f64 * ∞) as u64 = NaN as u64 = 0 (saturating cast).
    assert_eq!(g.total_sum(), 0);
    assert_invariants(&g);
}

// ── Idempotence / stacking ──────────────────────────────────────

#[test]
fn double_infinite_amplification() {
    let _t = init_tracing();
    let mut g = make_f64_graph();

    g.decay(g.g_root(), f64::INFINITY, 0.0);
    assert!(g.total_sum().is_infinite());

    // ∞ * ∞ = ∞ — applying ∞ twice should not panic or produce NaN.
    g.decay(g.g_root(), f64::INFINITY, 0.0);
    assert!(g.total_sum().is_infinite(), "∞ * ∞ should remain ∞");
    assert_invariants(&g);
}

// ── Boundary: att = 1.0 is a no-op ─────────────────────────────

#[test]
fn noop_at_attenuation_one() {
    let _t = init_tracing();
    let mut g = make_f64_graph();
    let sum_before = g.total_sum();

    g.decay(g.g_root(), 1.0, 0.0);
    assert_eq!(g.total_sum(), sum_before, "att=1.0 should be a no-op");

    // Also with selectivity.
    g.decay(g.g_root(), 1.0, 0.5);
    assert_eq!(g.total_sum(), sum_before, "att=1.0 q=0.5 should be a no-op");

    g.decay(g.g_root(), 1.0, 1.0);
    assert_eq!(g.total_sum(), sum_before, "att=1.0 q=1.0 should be a no-op");
    assert_invariants(&g);
}
