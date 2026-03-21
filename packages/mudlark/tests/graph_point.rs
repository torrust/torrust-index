// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **`GvGraph::get()` point queries**.
//!
//! `get(coord)` is the primary read path: given a coordinate it
//! descends the g-tree and returns the leaf `Cell` that contains it.
//! These tests verify correct routing through empty, shallow, and deep
//! trees, intensity accumulation, domain-boundary clamping, dyadic
//! cell alignment, consistency with `sample()`, and behaviour after
//! eviction — across both `u64` and `f64` coordinate types.
//!
//! # Test index
//!
//! ## Empty / single-node graph
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_single_node_returns_root`] | fresh graph returns full-domain root cell |
//! | [`get_single_node_any_coord`] | any coordinate maps to the root in an empty graph |
//!
//! ## After observation (no split)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_after_observe_returns_receiver`] | returned cell contains observed coordinate with correct intensity |
//! | [`get_accumulates_intensity`] | repeated observations at same coordinate accumulate |
//!
//! ## After splits
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_after_split_returns_child`] | distant observations split; cells are disjoint |
//! | [`get_deep_tree_narrows_interval`] | low threshold → deeper tree → narrower cells |
//! | [`get_range_tree_preset`] | preset range-tree plan routes all observed coords correctly |
//! | [`get_domain_midpoint_routes_correctly`] | midpoint coordinate routes to correct half after split |
//!
//! ## Boundary / clamping
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_domain_boundary_zero`] | coordinate 0 (domain start) returns valid cell |
//! | [`get_domain_boundary_max_minus_one`] | coordinate 255 (domain end − 1) returns valid cell |
//! | [`get_out_of_domain_high_clamps`] | out-of-domain high coordinate clamps to rightmost cell |
//! | [`get_out_of_domain_low_clamps_f64`] | negative f64 coordinate clamps to zero |
//! | [`get_nan_panics`] | NaN coordinate panics with descriptive message |
//!
//! ## Consistency with other queries
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_consistent_with_sample`] | `get()` at a sampled start returns a cell containing it |
//!
//! ## Cell invariants (property tests)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_cell_contains_queried_coordinate`] | every queried coordinate lies within the returned cell |
//! | [`get_cell_interval_is_dyadic`] | every returned cell has power-of-2 width and aligned start |
//!
//! ## After eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_after_eviction`] | `get()` returns valid cell after eviction absorbs a deep terminal |
//!
//! ## Type coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_u32_coordinates`] | u32 coordinate type works end-to-end |
//! | [`get_f64_coordinates`] | f64 coordinate type works end-to-end |

// Note: `get_semi_internal_*` and `get_semi_internal_depth_is_trimmed_depth`
// tests remain inline in graph.rs because they require `build_evictable_graph()`
// which mutates private fields.

mod support;

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, default_config, evictable_config, f64_default_config, low_threshold_config, plan_evictable, plan_range_tree,
    range_tree_config, run, run_checked,
};
use torrust_mudlark::{Config, GvGraph};

// ── Helpers ─────────────────────────────────────────────────────

/// High-threshold u64 config that avoids triggering splits.
const fn no_split_config() -> Config<u64> {
    Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

// ── Empty / single-node graph ───────────────────────────────────

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

// ── After observation (no split) ────────────────────────────────

#[test]
fn get_after_observe_returns_receiver() {
    let plan = Plan::new().observe(42u64, 10u64);
    let g = run::<u64, u64, 8>(no_split_config(), &plan);
    assert_invariants(&g);
    let cell = g.get(42u64);
    assert!(cell.start <= 42 && 42 < cell.end);
    assert_eq!(cell.intensity, 10);
}

#[test]
fn get_accumulates_intensity() {
    let plan = Plan::new().observe(42u64, 10u64).observe(42u64, 5u64).observe(42u64, 3u64);
    let g = run::<u64, u64, 8>(no_split_config(), &plan);
    assert_invariants(&g);
    let cell = g.get(42u64);
    assert_eq!(cell.intensity, 18);
}

// ── After splits ────────────────────────────────────────────────

#[test]
fn get_after_split_returns_child() {
    // Observe enough at two distant coordinates to trigger a split.
    let plan = Plan::new().observe_n(10u64, 1u64, 6).observe_n(200u64, 1u64, 6);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 1);
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
fn get_deep_tree_narrows_interval() {
    // Low threshold → frequent splits → deeper tree → narrower cells.
    let plan = Plan::new().spread(256, 3, 80);
    let g = run_checked::<u64, u64, 8>(low_threshold_config(), &plan, 20);
    let cell = g.get(42u64);
    // Cell must still contain the coordinate…
    assert!(cell.start <= 42 && 42 < cell.end);
    // …but should be narrower than the full domain.
    assert!(cell.end - cell.start < 256);
    assert!(cell.depth > 0);
}

#[test]
fn get_range_tree_preset() {
    let g = run::<u64, u64, 8>(range_tree_config(), &plan_range_tree());
    assert_invariants(&g);
    // The range-tree plan observes at 32, 96, 200.
    for &coord in &[32u64, 96, 200] {
        let cell = g.get(coord);
        assert!(
            cell.start <= coord && coord < cell.end,
            "cell [{}, {}) does not contain {coord}",
            cell.start,
            cell.end
        );
    }
}

#[test]
fn get_domain_midpoint_routes_correctly() {
    // After splitting, 128 is the midpoint — should route to the right half.
    let plan = Plan::new().observe_n(10u64, 1u64, 6).observe_n(200u64, 1u64, 6);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 1);
    let cell = g.get(128u64);
    assert!(cell.start <= 128 && 128 < cell.end);
}

// ── Boundary / clamping ─────────────────────────────────────────

#[test]
fn get_domain_boundary_zero() {
    let plan = Plan::new().observe(0u64, 10u64);
    let g = run::<u64, u64, 8>(no_split_config(), &plan);
    assert_invariants(&g);
    let cell = g.get(0u64);
    assert_eq!(cell.start, 0);
    assert!(cell.intensity >= 10);
}

#[test]
fn get_domain_boundary_max_minus_one() {
    let plan = Plan::new().observe(255u64, 10u64);
    let g = run::<u64, u64, 8>(no_split_config(), &plan);
    assert_invariants(&g);
    let cell = g.get(255u64);
    assert!(cell.start <= 255 && 255 < cell.end);
    assert!(cell.intensity >= 10);
}

#[test]
fn get_out_of_domain_high_clamps() {
    let plan = Plan::new().observe(250u64, 10u64);
    let mut g = run::<u64, u64, 8>(default_config(), &plan);
    g.observe(250u64, 0u64); // no-op observe just to ensure graph is populated
    assert_invariants(&g);
    // 256 is out of domain [0, 256). Should clamp and return a valid cell.
    let cell = g.get(256u64);
    assert!(cell.start < cell.end);
    // Should be same as getting the rightmost valid coordinate.
    let cell_max = g.get(255u64);
    assert_eq!(cell.start, cell_max.start);
    assert_eq!(cell.end, cell_max.end);
}

#[test]
fn get_out_of_domain_low_clamps_f64() {
    let plan = Plan::new().observe(1.0_f64, 10.0_f64);
    let g = run::<f64, f64, 4>(f64_default_config(), &plan);
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
    let g: GvGraph<f64, f64, 4> = GvGraph::new(f64_default_config());
    let _cell = g.get(f64::NAN);
}

// ── Consistency with other queries ──────────────────────────────

#[test]
fn get_consistent_with_sample() {
    let plan = Plan::new().observe(30u64, 10u64).observe(200u64, 10u64);
    let g = run::<u64, u64, 8>(default_config(), &plan);
    assert_invariants(&g);

    let mut rng = support::TestRng(42);
    if let Some(sampled) = g.sample(&mut rng) {
        // get() at the sample's start should return a cell whose
        // interval contains that start.
        let cell = g.get(sampled.start);
        assert!(cell.start <= sampled.start);
        assert!(sampled.start < cell.end);
    }
}

// ── Cell invariants (property tests) ────────────────────────────

#[test]
fn get_cell_contains_queried_coordinate() {
    // After many observations, every queried coordinate must lie within
    // the returned cell's interval.
    let plan = Plan::new().spread(256, 3, 100);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 25);
    for coord in (0u64..256).step_by(7) {
        let cell = g.get(coord);
        assert!(
            cell.start <= coord && coord < cell.end,
            "cell [{}, {}) does not contain {coord}",
            cell.start,
            cell.end
        );
    }
}

#[test]
fn get_cell_interval_is_dyadic() {
    // Every cell interval must be a power-of-2 aligned dyadic range.
    let plan = Plan::new().spread(256, 3, 100);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 25);
    for coord in (0u64..256).step_by(13) {
        let cell = g.get(coord);
        let width = cell.end - cell.start;
        assert!(
            width.is_power_of_two(),
            "cell [{}, {}) has non-power-of-2 width {width}",
            cell.start,
            cell.end
        );
        // start must be aligned to the width.
        assert_eq!(
            cell.start % width,
            0,
            "cell [{}, {}) is not aligned to width {width}",
            cell.start,
            cell.end
        );
    }
}

// ── After eviction ──────────────────────────────────────────────

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

// ── Type coverage ───────────────────────────────────────────────

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
    let plan = Plan::new().observe(2.5_f64, 10.0_f64);
    let g = run::<f64, f64, 4>(f64_default_config(), &plan);
    assert_invariants(&g);
    let cell = g.get(2.5_f64);
    assert!(cell.start <= 2.5 && 2.5 < cell.end);
}
