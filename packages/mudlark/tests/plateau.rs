// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Integration tests for the **live plateau mirror** (Step 2G).
//!
//! The plateau mirror maintains a `BTreeMap` of contour-level basis
//! elements that tile the full domain `[0, 2^N)`.  Every mutation
//! path — observe, split, evict, decay — must keep this map
//! consistent with the underlying G-tree.  These tests exercise all
//! four paths, plus dedicated coverage for the thatching lifecycle
//! (semi-internal nodes created during eviction) and the public
//! surface API (`width`, `to_span`, floor-key lookup).
//!
//! **Important:** plateau sums do NOT necessarily equal `root.sum`.
//! Basis elements sit at the contour; `own` values at internal nodes
//! above the contour are included in each basis element's `g.sum`
//! but may overlap when a parent's `own` is nonzero.  Use
//! `assert_invariants` (which checks P-I1..P-I5 plus sum/depth
//! consistency) rather than asserting global sum conservation.
//!
//! # Test index
//!
//! ## Fresh / trivial
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fresh_graph_single_plateau`] | new graph has exactly one plateau at depth 0 |
//! | [`no_split_preserves_single_plateau`] | high split threshold keeps a single plateau |
//!
//! ## Worked example
//!
//! | Test | Focus |
//! |------|-------|
//! | [`worked_example_plateau_states`] | step-by-step plateau evolution (N=3) |
//!
//! ## Cascade / tiling
//!
//! | Test | Focus |
//! |------|-------|
//! | [`cascade_plateau_tiling`] | P-I1 tiling and key ordering after cascade |
//!
//! ## Budget / shrinkage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_plateau_shrinkage`] | tight budget reduces plateau count |
//!
//! ## Eviction / thatch lifecycle
//!
//! | Test | Focus |
//! |------|-------|
//! | [`eviction_thatch_lifecycle`] | growth then hotspot eviction |
//! | [`simple_thatch_one_hop`] | P-I4 thatch one-hop with unbounded eviction |
//! | [`thatch_lifecycle_hotspot_shift`] | hotspot shift triggers thatch creation |
//!
//! ## Decay
//!
//! | Test | Focus |
//! |------|-------|
//! | [`decay_preserves_plateau_structure`] | uniform decay (q=0) preserves keys and depths |
//! | [`selective_decay_preserves_plateaus`] | selective decay (q>0) preserves plateau topology |
//!
//! ## Domain tiling completeness
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateaus_tile_full_domain_n4`] | plateaus cover `[0, 2^4)` with no gaps or overlaps |
//! | [`plateaus_tile_full_domain_n8`] | plateaus cover `[0, 2^8)` with no gaps or overlaps |
//!
//! ## Surface API
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateau_width_consistent`] | `width() == end - start` for every plateau |
//! | [`plateau_to_span_round_trip`] | `to_span()` preserves start/end/sum/depth |
//! | [`floor_key_lookup`] | floor-key lookup locates the containing plateau |
//!
//! ## Adversarial / stress
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adversarial_zigzag_plateaus`] | zigzag between domain extremes |
//! | [`skewed_pattern_plateaus`] | power-law skew preserves invariants |
//! | [`plan_multi_plateau_preset`] | preset plan produces multiple plateaus |
//! | [`large_domain_n8_stress`] | high-volume spread on N=8 under budget pressure |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config, budget_config_unbounded, cascade_config, default_config, low_threshold_config, no_split_config,
    plan_multi_plateau, run, run_checked, worked_example_config,
};
use torrust_mudlark::{BasisEdge, GvGraph};

mod support;
use support::init_tracing;

// ── Fresh / trivial ─────────────────────────────────────────────

/// A freshly created graph has exactly one plateau covering the
/// full domain at depth 0.
#[test]
fn fresh_graph_single_plateau() {
    let _t = init_tracing();
    let g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    let plateaus = g.plateaus();

    assert_eq!(plateaus.len(), 1, "fresh graph must have exactly one plateau");
    let (&key, p) = plateaus.iter().next().unwrap();
    assert_eq!(key, BasisEdge(0u64), "single plateau must start at origin");
    assert_eq!(p.depth, 0, "fresh plateau must be depth 0");
    assert_eq!(p.start, 0);
    assert_eq!(p.end, 16, "N=4 → domain [0, 16)");
    assert_invariants(&g);
}

/// With a high split threshold, observations never trigger splits,
/// so the single root plateau is preserved.
#[test]
fn no_split_preserves_single_plateau() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 10, 50);
    let g = run_checked::<u64, u64, 4>(no_split_config(), &plan, 5);

    assert_eq!(g.plateaus().len(), 1, "no-split config must keep single plateau");
    assert_invariants(&g);
}

// ── Worked example: step-by-step plateau evolution ──────────────

/// Build a small tree step-by-step and verify plateau structure
/// at each observation. Uses N=3 (domain `[0, 8)`).
#[test]
fn worked_example_plateau_states() {
    let _t = init_tracing();

    // Step 0: fresh graph has exactly one plateau at depth 0.
    let mut g: GvGraph<u64, u64, 3> = GvGraph::new(worked_example_config());
    assert_eq!(g.plateaus().len(), 1);
    assert_eq!(g.plateaus()[&BasisEdge(0u64)].depth, 0);

    // Step 1: observe(3, 10) triggers bootstrap split.
    g.observe(3u64, 10u64);
    assert_invariants(&g);
    assert_eq!(g.plateaus().len(), 1, "bootstrap split keeps one uniform-depth plateau");
    let plateaus = g.plateaus();
    let p = plateaus.values().next().unwrap();
    assert_eq!(p.depth, 1, "bootstrap split → depth-1 plateau");

    // Step 2: observe(3, 15) triggers catalytic split.
    g.observe(3u64, 15u64);
    assert_invariants(&g);
    assert!(
        g.plateaus().len() >= 2,
        "catalytic split should create a plateau boundary: got {}",
        g.plateaus().len(),
    );

    // Step 3: observe(6, 8) → another catalytic split in the right half.
    g.observe(6u64, 8u64);
    assert_invariants(&g);

    // Tiling: domain origin must always have a plateau key.
    assert!(g.plateaus().contains_key(&BasisEdge(0u64)));

    // Depths should be monotonically non-negative.
    for p in g.plateaus().values() {
        assert!(p.depth >= 1, "after splits, all plateaus should be depth >= 1");
    }
}

// ── Cascade plateau tiling ──────────────────────────────────────

#[test]
fn cascade_plateau_tiling() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 100);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);

    // P-I1: tiling must start at domain origin.
    assert!(
        g.plateaus().contains_key(&BasisEdge(0u64)),
        "tiling must start at domain origin",
    );

    // Keys must be strictly increasing (BTreeMap guarantees this,
    // but verify the spatial ordering makes sense).
    let plateaus = g.plateaus();
    let keys: Vec<_> = plateaus.keys().collect();
    for w in keys.windows(2) {
        assert!(w[0] < w[1], "plateau keys must be strictly increasing");
    }
}

// ── Budget plateau shrinkage ────────────────────────────────────

#[test]
fn budget_plateau_shrinkage() {
    let _t = init_tracing();
    // buffer = depth_evict - depth_create = 3, headroom = 3^4 = 81.
    // Budget must be > 81.
    let plan = Plan::new().spread(256, 6, 200);
    let g_no_budget = run::<u64, u64, 8>(default_config(), &plan);
    let g_budget = run_checked::<u64, u64, 8>(budget_config(100), &plan, 10);

    assert_invariants(&g_no_budget);
    assert_invariants(&g_budget);

    // With a tight budget, we expect fewer or equal plateaus.
    assert!(
        g_budget.plateaus().len() <= g_no_budget.plateaus().len(),
        "budget should reduce plateau count: budget={}, no_budget={}",
        g_budget.plateaus().len(),
        g_no_budget.plateaus().len(),
    );
}

// ── Eviction thatch lifecycle ───────────────────────────────────

/// Tree grows (splits) then budget pressure causes evictions.
/// Plateau invariants hold throughout including thatch
/// creation and destruction.
#[test]
fn eviction_thatch_lifecycle() {
    let _t = init_tracing();
    let plan = Plan::new()
        .spread(16, 6, 100) // grow
        .hotspot(8, 8, 60); // concentrate → evictions elsewhere
    let g = run_checked::<u64, u64, 4>(budget_config(100), &plan, 1);
    assert_invariants(&g);
}

// ── Thatching-specific tests ────────────────────────────────────

/// Low budget forces aggressive eviction → semi-internals (thatch).
/// P-I4 (thatch one-hop) must hold throughout.
#[test]
fn simple_thatch_one_hop() {
    let _t = init_tracing();
    let plan = Plan::new()
        .spread(16, 6, 80) // build a reasonably deep tree
        .spread(16, 1, 40); // light observations → trigger evictions
    let g = run_checked::<u64, u64, 4>(budget_config_unbounded(100), &plan, 1);
    assert_invariants(&g);
}

/// Drive a hotspot to create deep splits, then shift to another
/// region to trigger evictions in the original hotspot. The
/// plateau mirror must remain valid throughout the thatch lifecycle.
#[test]
fn thatch_lifecycle_hotspot_shift() {
    let _t = init_tracing();
    let plan = Plan::new()
        .hotspot(3, 10, 40) // deep splits at coord 3
        .hotspot(12, 10, 40); // shift → evictions in coord 3's region
    let g = run_checked::<u64, u64, 4>(budget_config(100), &plan, 1);
    assert_invariants(&g);
}

// ── Decay ───────────────────────────────────────────────────────

/// Uniform decay (q=0) should preserve plateau keys and depths
/// while reducing sums.
#[test]
fn decay_preserves_plateau_structure() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 60);
    let mut g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 5);
    assert_invariants(&g);

    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_before: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();

    let root = g.g_root();
    g.decay(root, 0.5, 0.0);
    assert_invariants(&g);

    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_after: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();
    assert_eq!(keys_before, keys_after, "uniform decay must not change plateau keys");
    assert_eq!(depths_before, depths_after, "uniform decay must not change plateau depths");
}

/// Selective decay (q > 0) should also preserve plateau topology.
#[test]
fn selective_decay_preserves_plateaus() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 60);
    let mut g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 5);
    assert_invariants(&g);

    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();

    let root = g.g_root();
    g.decay(root, 0.75, 0.5);
    assert_invariants(&g);

    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    assert_eq!(keys_before, keys_after, "selective decay must not change plateau keys");
}

// ── Domain tiling completeness ──────────────────────────────────

/// Verify that plateau ranges tile the full domain `[0, 2^N)` with
/// no gaps and no overlaps.
#[test]
fn plateaus_tile_full_domain_n4() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 80);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 5);
    assert_invariants(&g);
    assert_plateaus_tile_domain(&g, 16);
}

#[test]
fn plateaus_tile_full_domain_n8() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 200);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 10);
    assert_invariants(&g);
    assert_plateaus_tile_domain(&g, 256);
}

/// Assert plateau start/end ranges tile `[0, domain_size)` contiguously.
fn assert_plateaus_tile_domain<const N: u32>(g: &GvGraph<u64, u64, N>, domain_size: u64) {
    let plateaus = g.plateaus();
    let entries: Vec<_> = plateaus.values().collect();

    assert!(!entries.is_empty(), "must have at least one plateau");
    assert_eq!(entries.first().unwrap().start, 0, "first plateau must start at 0");
    assert_eq!(
        entries.last().unwrap().end,
        domain_size,
        "last plateau must end at domain size"
    );

    for w in entries.windows(2) {
        assert_eq!(
            w[0].end, w[1].start,
            "plateau gap/overlap between [{}, {}) and [{}, {})",
            w[0].start, w[0].end, w[1].start, w[1].end,
        );
    }
}

// ── Surface API ─────────────────────────────────────────────────

/// `Plateau::width()` must equal `end - start` for every plateau.
#[test]
fn plateau_width_consistent() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 80);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 5);

    for p in g.plateaus().values() {
        assert_eq!(p.width(), p.end - p.start, "width mismatch for plateau at {:?}", p.basis_edge);
    }
}

/// `Plateau::to_span()` must transfer all fields correctly.
#[test]
fn plateau_to_span_round_trip() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 80);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 5);

    for p in g.plateaus().values() {
        let span = p.to_span();
        assert_eq!(span.start, p.start);
        assert_eq!(span.end, p.end);
        assert_eq!(span.intensity, p.sum);
        assert_eq!(span.depth, p.depth);
    }
}

/// Floor-key lookup: `range(..=BasisEdge(x)).next_back()` must
/// return the plateau whose `[start, end)` contains `x`.
#[test]
fn floor_key_lookup() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 80);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 5);
    let plateaus = g.plateaus();

    // Test every integer coordinate in the domain.
    for x in 0..16u64 {
        let (_edge, p) = plateaus
            .range(..=BasisEdge(x))
            .next_back()
            .unwrap_or_else(|| panic!("no plateau found for coord {x}"));
        assert!(
            p.start <= x && x < p.end,
            "floor-key lookup for {x}: plateau [{}, {}) does not contain it",
            p.start,
            p.end,
        );
    }
}

// ── Adversarial / stress ────────────────────────────────────────

/// Adversarial zigzag between domain extremes with plateau tracking.
#[test]
fn adversarial_zigzag_plateaus() {
    let _t = init_tracing();
    let plan = Plan::new().adversarial_zigzag(0, 15, 5, 3, 80);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
    assert!(
        g.plateaus().len() >= 2,
        "adversarial zigzag should produce multiple plateaus, got {}",
        g.plateaus().len(),
    );
}

/// Power-law skew: lower coordinates get disproportionate intensity.
#[test]
fn skewed_pattern_plateaus() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(16, 100);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

/// The `plan_multi_plateau` preset must produce multiple plateaus.
#[test]
fn plan_multi_plateau_preset() {
    let _t = init_tracing();
    let plan = plan_multi_plateau();
    let g = run_checked::<u64, u64, 8>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
    assert!(
        g.plateaus().len() >= 2,
        "plan_multi_plateau should produce multiple plateaus, got {}",
        g.plateaus().len(),
    );
}

/// High-volume spread on a large domain (N=8) under budget pressure.
#[test]
fn large_domain_n8_stress() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 500);
    let g = run_checked::<u64, u64, 8>(budget_config(200), &plan, 25);
    assert_invariants(&g);
    // Must still tile the domain.
    let plateaus = g.plateaus();
    assert!(plateaus.contains_key(&BasisEdge(0u64)));
    let last = plateaus.values().last().unwrap();
    assert_eq!(last.end, 256, "last plateau must end at 2^8");
}
