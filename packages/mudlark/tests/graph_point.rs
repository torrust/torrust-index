// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `get()` point queries.

mod support;

use support::TestRng;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{default_config, evictable_config, plan_evictable, run};
use torrust_mudlark::{Config, GvGraph};

// Note: `get_semi_internal_*` and `get_semi_internal_depth_is_trimmed_depth`
// tests remain inline in graph.rs because they require `build_evictable_graph()`
// which mutates private fields.

#[test]
fn get_single_node_returns_root() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let cell = g.get(0u64);
    assert_eq!(cell.start, 0);
    assert_eq!(cell.end, 256); // 2^8
    assert_eq!(cell.intensity, 0);
    assert_eq!(cell.depth, 0);
}

#[test]
fn get_single_node_any_coord() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Any coordinate in the domain should return the root cell.
    for &c in &[0u64, 42, 128, 255] {
        let cell = g.get(c);
        assert_eq!(cell.start, 0);
        assert_eq!(cell.end, 256);
        assert_eq!(cell.intensity, 0);
        assert_eq!(cell.depth, 0);
    }
}

#[test]
fn get_after_observe_returns_receiver() {
    // Use a high threshold to avoid triggering a split.
    let cfg = Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(42u64, 10u64);
    assert_invariants(&g);
    let cell = g.get(42u64);
    assert!(cell.start <= 42 && 42 < cell.end);
    assert_eq!(cell.intensity, 10);
}

#[test]
fn get_after_split_returns_child() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Observe enough at two distant coordinates to trigger a split.
    for _ in 0..6 {
        g.observe(10u64, 1u64);
        g.observe(200u64, 1u64);
    }
    assert_invariants(&g);
    // Should have more than one node now.
    assert!(g.node_count() > 1);
    let left_cell = g.get(10u64);
    let right_cell = g.get(200u64);
    assert!(left_cell.start <= 10 && 10 < left_cell.end);
    assert!(right_cell.start <= 200 && 200 < right_cell.end);
    // They should be in different cells.
    assert!(left_cell.end <= right_cell.start || right_cell.end <= left_cell.start);
}

#[test]
fn get_out_of_domain_high_clamps() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    g.observe(250u64, 10u64);
    assert_invariants(&g);
    // 256 is out of domain [0, 256). Should clamp and return a
    // valid cell.
    let cell = g.get(256u64);
    assert!(cell.start < cell.end);
    // Should be same as getting the rightmost valid coordinate.
    let cell_max = g.get(255u64);
    assert_eq!(cell.start, cell_max.start);
    assert_eq!(cell.end, cell_max.end);
}

#[test]
fn get_out_of_domain_low_clamps_f64() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(1.0_f64, 10.0_f64);
    assert_invariants(&g);
    let cell_neg = g.get(-5.0_f64);
    let cell_zero = g.get(0.0_f64);
    // Clamped to zero, same result.
    #[allow(clippy::float_cmp)]
    {
        assert_eq!(cell_neg.start, cell_zero.start);
        assert_eq!(cell_neg.end, cell_zero.end);
    }
}

#[test]
#[should_panic(expected = "get(): coordinate is NaN")]
fn get_nan_panics() {
    let g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    let _cell = g.get(f64::NAN);
}

#[test]
fn get_domain_boundary_zero() {
    let cfg = Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(0u64, 10u64);
    assert_invariants(&g);
    let cell = g.get(0u64);
    assert_eq!(cell.start, 0);
    assert!(cell.intensity >= 10);
}

#[test]
fn get_domain_boundary_max_minus_one() {
    let cfg = Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(255u64, 10u64);
    assert_invariants(&g);
    let cell = g.get(255u64);
    assert!(cell.start <= 255 && 255 < cell.end);
    assert!(cell.intensity >= 10);
}

#[test]
fn get_consistent_with_sample() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    g.observe(30u64, 10u64);
    g.observe(200u64, 10u64);
    assert_invariants(&g);

    let mut rng = TestRng(42);
    if let Some(sampled) = g.sample(&mut rng) {
        // get() at the sample's start should return a cell whose
        // interval contains that start.
        let cell = g.get(sampled.start);
        assert!(cell.start <= sampled.start);
        assert!(sampled.start < cell.end);
    }
}

#[test]
fn get_u32_coordinates() {
    let g: GvGraph<u32, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    let cell = g.get(42u32);
    assert!(cell.start <= 42 && 42 < cell.end);
    assert_eq!(cell.depth, 0);
}

#[test]
fn get_f64_coordinates() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(2.5_f64, 10.0_f64);
    assert_invariants(&g);
    let cell = g.get(2.5_f64);
    assert!(cell.start <= 2.5 && 2.5 < cell.end);
}

#[test]
fn get_after_eviction() {
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    // Record a coordinate that routes to a deep terminal.
    let deep_coord = 32u64;
    g.observe(deep_coord, 1u64);
    assert_invariants(&g);

    // Evict — the deep terminal may be absorbed into its parent.
    let _evicted = g.check_evictions();
    assert_invariants(&g);

    // get() should still work and return a valid cell.
    let cell = g.get(deep_coord);
    assert!(cell.start <= deep_coord && deep_coord < cell.end);
}
