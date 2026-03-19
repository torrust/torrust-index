// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Integration tests for Step 2G: live plateau mirror.
//!
//! Validates plateau `BTreeMap` invariants across the four mutation
//! paths (observe, split, evict, decay) using the `graph_creator`
//! harness and dedicated thatching lifecycle tests.
//!
//! Note: plateau sums do NOT necessarily equal `root.sum`. Basis
//! elements sit at the contour; `own` values at internal nodes
//! above the contour are included in each basis element's `g.sum`
//! but may overlap when a parent's `own` is nonzero. Use
//! `assert_invariants` (which checks P-I1..P-I5 plus sum/depth
//! consistency) rather than asserting global sum conservation.

use torrust_mudlark::invariants::{assert_invariants, check_all_invariants};
use torrust_mudlark::testing::{Plan, cascade_config, default_config, low_threshold_config, run, run_checked};
use torrust_mudlark::{BasisEdge, Config, GvGraph};

mod support;
use support::init_tracing;

// ── Worked example: step-by-step plateau evolution ──────────────

/// Build a small tree step-by-step and verify plateau structure
/// at each observation. Uses N=3 (domain `[0, 8)`).
#[test]
fn worked_example_plateau_states() {
    let _t = init_tracing();
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };

    // Step 0: fresh graph has exactly one plateau at depth 0.
    let mut g: GvGraph<u64, u64, 3> = GvGraph::new(config);
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

    let budgeted_config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let g_budget = run_checked::<u64, u64, 8>(budgeted_config, &plan, 10);

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
    // buffer=3, headroom=81, budget=100 > 81 ✓
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new()
        .spread(16, 6, 100) // grow
        .hotspot(8, 8, 60); // concentrate → evictions elsewhere
    let g = run_checked::<u64, u64, 4>(config, &plan, 1);
    assert_invariants(&g);
}

// ── Thatching-specific tests ────────────────────────────────────

/// Low budget forces aggressive eviction → semi-internals (thatch).
/// P-I4 (thatch one-hop) must hold throughout.
#[test]
fn simple_thatch_one_hop() {
    let _t = init_tracing();
    // buffer=3, headroom=81, budget=100 > 81 ✓
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: false,
    };
    let plan = Plan::new()
        .spread(16, 6, 80) // build a reasonably deep tree
        .spread(16, 1, 40); // light observations → trigger evictions
    let g = run_checked::<u64, u64, 4>(config, &plan, 1);
    assert_invariants(&g);
}

/// Drive a hotspot to create deep splits, then shift to another
/// region to trigger evictions in the original hotspot. The
/// plateau mirror must remain valid throughout the thatch lifecycle.
#[test]
fn thatch_lifecycle_hotspot_shift() {
    let _t = init_tracing();
    // buffer=3, headroom=81, budget=100 > 81 ✓
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new()
        .hotspot(3, 10, 40) // deep splits at coord 3
        .hotspot(12, 10, 40); // shift → evictions in coord 3's region
    let g = run_checked::<u64, u64, 4>(config, &plan, 1);
    assert_invariants(&g);
}

/// Two hotspots on opposite ends create semi-internals that thatch
/// from both sides of a plateau (bilateral thatching).
#[test]
fn bilateral_thatch() {
    let _t = init_tracing();
    // buffer=3, headroom=81, budget=100 > 81 ✓
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new()
        .hotspot(1, 10, 30) // left side deep
        .hotspot(14, 10, 30) // right side deep
        .spread(16, 1, 40); // light spread → evictions on both sides
    let g = run_checked::<u64, u64, 4>(config, &plan, 1);
    assert_invariants(&g);
}

/// Deep tree under tight budget: nested evictions create layered
/// semi-internals. P-I5 (thatch depth bound) must hold.
#[test]
fn transitive_thatch_stacking() {
    let _t = init_tracing();
    // buffer=5, headroom=3^6=729, budget=800 > 729 ✓
    let config = Config {
        split_threshold: 3,
        depth_create: 3,
        depth_evict: 8,
        budget: Some(800),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new()
        .spread(256, 8, 150) // deep tree
        .hotspot(128, 5, 100) // concentrate → evict periphery
        .spread(256, 1, 50); // light → more evictions
    let g = run_checked::<u64, u64, 8>(config, &plan, 5);
    assert_invariants(&g);
}

// ── BTreeMap surface tests ──────────────────────────────────────

/// Range query returns a meaningful subset.
#[test]
fn range_query() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 100);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_invariants(&g);

    let range_count = g.plateaus().range(BasisEdge(4u64)..BasisEdge(12u64)).count();
    assert!(range_count <= g.plateaus().len());
    // Range should be a strict subset (domain is wider than [4, 12)).
    assert!(
        range_count < g.plateaus().len(),
        "range [4,12) should be a strict subset of the full plateau set",
    );
}

/// Floor-key lookup covers the entire domain: every coordinate
/// in `[0, 2^N)` has a covering plateau.
#[test]
fn floor_key_covers_domain() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 50);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_invariants(&g);

    for coord in 0..16u64 {
        let plateaus = g.plateaus();
        let result = plateaus.range(..=BasisEdge(coord)).next_back();
        assert!(result.is_some(), "floor-key query at coord={coord} should find a plateau");
    }
}

// ── Decay + plateau structure stability ─────────────────────────

/// Repeated decay does not change plateau keys or depths (decay is
/// magnitude-only — no structural changes).
#[test]
fn repeated_decay_preserves_structure() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 50);
    let mut g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    assert_invariants(&g);

    let keys_before: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_before: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();

    for _ in 0..5 {
        g.decay(g.g_root(), 0.8, 0.0);
        assert_invariants(&g);
    }

    let keys_after: Vec<_> = g.plateaus().keys().copied().collect();
    let depths_after: Vec<_> = g.plateaus().values().map(|p| p.depth).collect();
    assert_eq!(keys_before, keys_after, "decay must not change plateau keys");
    assert_eq!(depths_before, depths_after, "decay must not change plateau depths");
}

/// Decay reduces plateau sums (but doesn't zero them entirely
/// unless threshold is 0).
#[test]
fn decay_reduces_plateau_sums() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 50);
    let mut g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    assert_invariants(&g);

    let sums_before: Vec<u64> = g.plateaus().values().map(|p| p.sum).collect();
    g.decay(g.g_root(), 0.5, 0.0);
    assert_invariants(&g);
    let sums_after: Vec<u64> = g.plateaus().values().map(|p| p.sum).collect();

    // Every sum should be <= what it was before decay.
    for (before, after) in sums_before.iter().zip(&sums_after) {
        assert!(after <= before, "decay should not increase sums: {after} > {before}");
    }
}

// ── Adversarial patterns ────────────────────────────────────────

/// Adversarial zigzag under budget: alternating observations at
/// opposite ends of the domain force many splits and evictions.
#[test]
fn adversarial_zigzag_plateau_invariants() {
    let _t = init_tracing();
    // buffer=3, headroom=81, budget=100 > 81 ✓
    let config = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: Some(100),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new().adversarial_zigzag(0, 15, 10, 1, 40);
    let g = run_checked::<u64, u64, 4>(config, &plan, 1);
    assert_invariants(&g);
    assert!(g.plateaus().contains_key(&BasisEdge(0u64)));
}

// ── Cross-type tests ────────────────────────────────────────────

/// `GvGraph<u128, u64, 16>` compiles and maintains plateaus.
#[test]
fn u128_coordinates_plateaus() {
    let _t = init_tracing();
    let config = Config {
        split_threshold: 5u64,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u128, u64, 16> = GvGraph::new(config);
    g.observe(1000u128, 10u64);
    g.observe(1000u128, 10u64);
    assert_invariants(&g);
    assert!(!g.plateaus().is_empty());
}

// ── `check_all_invariants` surface ──────────────────────────────

#[test]
fn check_all_invariants_empty_on_healthy_graph() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 6, 50);
    let g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    let errors = check_all_invariants(&g);
    assert!(
        errors.is_empty(),
        "healthy graph should have no invariant violations: {errors:?}",
    );
}

// ── Migrated from graph.rs inline tests ─────────────────────────

#[test]
fn new_graph_has_one_plateau() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 1);
    let (&key, p) = plateaus.iter().next().unwrap();
    assert_eq!(key, BasisEdge(0u64));
    assert_eq!(p.depth, 0);
    assert_eq!(p.sum, 0u64);
}

#[test]
fn new_graph_plateau_covers_domain() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let plateaus = g.plateaus();
    let p = &plateaus[&BasisEdge(0u64)];
    assert_eq!(p.start, 0u64);
    assert_eq!(p.end, 1u64 << 8);
}

#[test]
fn plateaus_accessor_returns_btreemap() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Type check: the accessor returns a Cow wrapping a BTreeMap.
    let map = g.plateaus();
    assert!(!map.is_empty());
}

#[test]
fn floor_key_point_query() {
    // After splits, a floor-key lookup should find the correct plateau.
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    for _ in 0..5 {
        g.observe(3u64, 10u64);
        g.observe(12u64, 10u64);
    }
    assert_invariants(&g);

    // Point query at coord=5 via floor-key.
    let query = BasisEdge(5u64);
    let plateaus = g.plateaus();
    let result = plateaus.range(..=query).next_back();
    assert!(result.is_some(), "floor-key query should find a plateau");
}

#[test]
fn plateau_iteration_order() {
    // BTreeMap iteration is left-to-right spatial order.
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    for _ in 0..5 {
        g.observe(3u64, 10u64);
        g.observe(12u64, 10u64);
    }
    assert_invariants(&g);

    let plateaus = g.plateaus();
    let keys: Vec<_> = plateaus.keys().collect();
    for w in keys.windows(2) {
        assert!(w[0] < w[1], "plateau keys must be sorted: {w:?}");
    }
}

#[test]
fn plateau_len_equals_count() {
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(default_config());
    for _ in 0..10 {
        g.observe(3u64, 10u64);
        g.observe(12u64, 10u64);
    }
    assert_invariants(&g);
    assert!(!g.plateaus().is_empty());
}

#[test]
fn f64_coordinates_compile_with_plateaus() {
    let mut g: GvGraph<f64, f64, 8> = GvGraph::new(Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(3.0_f64, 10.0_f64);
    assert_invariants(&g);
    assert!(!g.plateaus().is_empty());
}
