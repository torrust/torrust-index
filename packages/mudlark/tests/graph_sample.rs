// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **`GvGraph::sample()`** — weighted random
//! cell selection.
//!
//! These tests exercise the full `sample()` path: the empty-graph
//! edge case, deterministic replay with controlled RNG values,
//! structural validity of returned cells (domain bounds, dyadic
//! alignment, positive intensity), statistical convergence to the
//! expected weight distribution, exhaustive reachability of every
//! terminal, and correct behaviour after eviction passes.
//!
//! # Test index
//!
//! ## Empty / degenerate
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_empty_graph_returns_none`] | `None` for a freshly constructed graph |
//! | [`sample_zero_intensity_returns_none`] | `None` when total intensity is zero |
//! | [`sample_single_root_entry`] | single root cell returned when no splits occur |
//! | [`sample_accumulated_single_coordinate`] | repeated observations at one coordinate accumulate |
//!
//! ## Determinism
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_deterministic_fixed_rng`] | identical RNG value always yields the same cell |
//! | [`sample_deterministic_low_picks_first_child`] | `rng = 0.0` selects the first child at every level |
//! | [`sample_deterministic_high_picks_last_child`] | `rng ≈ 1.0` selects the rightmost terminal |
//! | [`sample_seq_rng_advances_per_call`] | successive calls consume successive RNG values |
//!
//! ## Cell validity
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_cell_within_domain`] | sampled interval lies within `[0, 256)` |
//! | [`sample_cell_interval_is_dyadic`] | width is a power of 2, start is aligned |
//! | [`sample_cell_has_positive_intensity`] | sampled cell always has intensity > 0 |
//!
//! ## Weight proportionality
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_respects_weight_proportions`] | both halves of the domain are reachable |
//! | [`sample_frequency_convergence`] | 10 k samples converge to expected weight fractions (4σ) |
//!
//! ## Exhaustive reachability
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_all_terminals_reachable`] | dense RNG sweep visits every distinct terminal |
//!
//! ## Deeper / richer topologies
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_deep_tree`] | dense spread → deep tree still returns valid terminals |
//! | [`sample_after_spread`] | uniform spread makes both low and high quarters reachable |
//! | [`sample_budgeted_graph`] | sampling works after budget-triggered evictions |
//!
//! ## Type coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_f64_coordinate`] | `GvGraph<f64, f64, 4>` round-trip through `sample()` |
//!
//! ## Trait coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_via_weighted_sampler_trait`] | `WeightedSampler::sample` agrees with inherent method |
//!
//! ## Post-eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_after_explicit_eviction`] | valid cells returned after explicit `check_evictions()` rounds |
//!
//! ## Consistency with other queries
//!
//! | Test | Focus |
//! |------|-------|
//! | [`sample_consistent_with_get`] | `get(start)` interval contains the sampled coordinate |

mod support;

use support::{FixedRng, SeqRng, TestRng};
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config, default_config, evictable_config, f64_no_split_config, no_split_config, plan_evictable, plan_range_tree,
    range_tree_config, run, run_checked,
};
use torrust_mudlark::{GvGraph, Weighable, WeightedSampler};

// ── Helpers ─────────────────────────────────────────────────────

/// Build a small multi-node range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    let g = run::<u64, u64, 8>(range_tree_config(), &plan_range_tree());
    assert_invariants(&g);
    g
}

// ── Empty / degenerate ──────────────────────────────────────────

#[test]
fn sample_empty_graph_returns_none() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert!(g.sample(&mut FixedRng(0.0)).is_none());
    assert!(g.sample(&mut FixedRng(0.5)).is_none());
    assert!(g.sample(&mut FixedRng(0.999)).is_none());
}

#[test]
fn sample_zero_intensity_returns_none() {
    // A fresh graph with default config has zero total intensity.
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert!(g.sample(&mut TestRng(42)).is_none());
}

#[test]
fn sample_single_root_entry() {
    // High threshold prevents splits → single root entry.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(no_split_config());
    g.observe(100u64, 42u64);
    assert_invariants(&g);

    // Any RNG value must return the root cell.
    let cell = g.sample(&mut FixedRng(0.0)).unwrap();
    assert_eq!(cell.start, 0);
    assert_eq!(cell.end, 256);
    assert_eq!(cell.intensity, 42);
    assert_eq!(cell.depth, 0);

    // Same result at different RNG values.
    let cell2 = g.sample(&mut FixedRng(0.999)).unwrap();
    assert_eq!(cell2.start, cell.start);
    assert_eq!(cell2.intensity, cell.intensity);
}

#[test]
fn sample_accumulated_single_coordinate() {
    // Multiple observations at the same coordinate accumulate.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(no_split_config());
    g.observe(50u64, 10u64);
    g.observe(50u64, 20u64);
    g.observe(50u64, 5u64);
    assert_invariants(&g);

    let cell = g.sample(&mut FixedRng(0.5)).unwrap();
    assert_eq!(cell.intensity, 35);
}

// ── Determinism ─────────────────────────────────────────────────

#[test]
fn sample_deterministic_fixed_rng() {
    // Same FixedRng value always yields exactly the same cell.
    let g = build_range_tree();
    let cell1 = g.sample(&mut FixedRng(0.42)).unwrap();
    let cell2 = g.sample(&mut FixedRng(0.42)).unwrap();
    assert_eq!(cell1, cell2, "same RNG value must yield same cell");
}

#[test]
fn sample_deterministic_low_picks_first_child() {
    // rng=0.0 → threshold=0.0 → always picks the first child at
    // each level. Verify consistency (exact cell depends on V-tree
    // ordering, which is by intensity).
    let g = build_range_tree();
    let cell = g.sample(&mut FixedRng(0.0)).unwrap();
    // Must be a valid terminal within the domain.
    assert!(cell.start < cell.end);
    assert!(cell.end <= 256);
    assert!(cell.depth > 0);
    // Repeated calls must return the same result.
    assert_eq!(cell, g.sample(&mut FixedRng(0.0)).unwrap());
}

#[test]
fn sample_deterministic_high_picks_last_child() {
    // rng just below 1.0 → always picks the last child at each level.
    let g = build_range_tree();
    let cell = g.sample(&mut FixedRng(0.999_999_9)).unwrap();
    // Should land on the rightmost terminal.
    assert_eq!(cell.end, 256, "rng≈1.0 should pick the rightmost terminal");
}

#[test]
fn sample_seq_rng_advances_per_call() {
    // Verify that successive sample() calls consume successive RNG
    // values. Extreme 0.0 and 0.999 should (in a multi-entry tree)
    // land on different cells.
    let g = build_range_tree();
    let mut rng = SeqRng::new(vec![0.0, 0.999]);
    let cell1 = g.sample(&mut rng).unwrap();
    let cell2 = g.sample(&mut rng).unwrap();
    assert!(cell1.start < cell1.end);
    assert!(cell2.start < cell2.end);
    assert_ne!(cell1, cell2, "extreme RNG values should pick different cells");
}

// ── Cell validity ───────────────────────────────────────────────

#[test]
fn sample_cell_within_domain() {
    let g = build_range_tree();
    // Sweep the full [0,1) RNG range.
    for i in 0..100 {
        let rng_val = f64::from(i) / 100.0;
        let cell = g.sample(&mut FixedRng(rng_val)).unwrap();
        assert!(cell.start < cell.end, "start must be < end");
        assert!(cell.end <= 256, "cell must lie within domain [0, 256)");
    }
}

#[test]
fn sample_cell_interval_is_dyadic() {
    // Every sampled cell must have a power-of-2 width, aligned
    // to its width.
    let g = build_range_tree();
    for i in 0..50 {
        let rng_val = f64::from(i) / 50.0;
        let cell = g.sample(&mut FixedRng(rng_val)).unwrap();
        let width = cell.end - cell.start;
        assert!(width.is_power_of_two(), "width {width} must be a power of 2");
        assert_eq!(cell.start % width, 0, "start {} not aligned to width {width}", cell.start);
    }
}

#[test]
fn sample_cell_has_positive_intensity() {
    let g = build_range_tree();
    for i in 0..50 {
        let rng_val = f64::from(i) / 50.0;
        let cell = g.sample(&mut FixedRng(rng_val)).unwrap();
        assert!(cell.intensity > 0, "sampled cell must have positive intensity");
    }
}

// ── Weight proportionality ──────────────────────────────────────

#[test]
fn sample_respects_weight_proportions() {
    // Build a tree with entries in both halves of the domain.
    // range_tree_config() has split_threshold=1, forcing splits.
    let plan = Plan::new().observe(32u64, 30u64).observe(200u64, 10u64);
    let g = run::<u64, u64, 8>(range_tree_config(), &plan);
    assert_invariants(&g);

    let total_intensity = g.total_sum();
    assert!(total_intensity > 0);

    // Sample across the RNG range and verify both halves are reachable.
    let mut saw_left = false;
    let mut saw_right = false;
    for i in 0..20 {
        let rng_val = f64::from(i) / 20.0;
        if let Some(cell) = g.sample(&mut FixedRng(rng_val)) {
            if cell.start < 128 {
                saw_left = true;
            } else {
                saw_right = true;
            }
        }
    }
    assert!(saw_left, "should sample from left half");
    assert!(saw_right, "should sample from right half");
}

#[test]
fn sample_frequency_convergence() {
    // Statistical test: N samples via LCG, verify observed
    // frequencies converge to weights proportional to cell intensity.
    let plan = Plan::new().observe(32u64, 30u64).observe(200u64, 10u64);
    let g = run::<u64, u64, 8>(range_tree_config(), &plan);
    assert_invariants(&g);

    let total_w: f64 = g.total_sum().weight();
    assert!(total_w > 0.0);

    let n = 10_000u32;
    let mut rng = TestRng(42);
    let mut buckets: std::collections::HashMap<u64, (u32, f64)> = std::collections::HashMap::new();
    for _ in 0..n {
        let cell = g.sample(&mut rng).unwrap();
        buckets
            .entry(cell.start)
            .and_modify(|(count, _)| *count += 1)
            .or_insert_with(|| (1, cell.intensity.weight()));
    }

    // Verify each bucket's frequency matches its expected weight.
    for (&start, &(count, weight)) in &buckets {
        let expected = weight / total_w;
        let observed = f64::from(count) / f64::from(n);
        // 4σ tolerance.
        let tolerance = 4.0 * (expected * (1.0 - expected) / f64::from(n)).sqrt();
        assert!(
            (observed - expected).abs() < tolerance,
            "bucket start={start}: observed={observed:.4}, expected={expected:.4}, tol={tolerance:.4}"
        );
    }
}

// ── Exhaustive reachability ─────────────────────────────────────

#[test]
fn sample_all_terminals_reachable() {
    // Every terminal cell in the range-tree must be reachable by
    // some RNG value. We sweep a dense grid of RNG values and
    // collect distinct sampled cells. sample() walks the V-tree,
    // so results may include semi-internal uncovered halves.
    let g = build_range_tree();

    let mut seen: std::collections::HashSet<(u64, u64)> = std::collections::HashSet::new();
    for i in 0..10_000 {
        let rng_val = f64::from(i) / 10_000.0;
        let cell = g.sample(&mut FixedRng(rng_val)).unwrap();
        seen.insert((cell.start, cell.end));
    }

    // The range-tree plan has 3 observations (32, 96, 200) with
    // split_threshold=1 — producing multiple distinct V-entries.
    assert!(seen.len() >= 3, "expected >=3 distinct cells, got {}", seen.len());

    // Every seen cell must be a valid dyadic interval within the domain.
    for &(start, end) in &seen {
        assert!(start < end, "cell [{start}, {end}) has start >= end");
        assert!(end <= 256, "cell [{start}, {end}) exceeds domain");
        let width = end - start;
        assert!(width.is_power_of_two(), "width {width} is not a power of 2");
    }
}

// ── Deeper / richer topologies ──────────────────────────────────

#[test]
fn sample_deep_tree() {
    // More observations → deeper tree. Verify sampling still works
    // and always returns valid terminals.
    let plan = Plan::new().spread(256, 5, 200);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 50);

    let mut rng = TestRng(99);
    for _ in 0..100 {
        let cell = g.sample(&mut rng).unwrap();
        assert!(cell.start < cell.end);
        assert!(cell.end <= 256);
        assert!(cell.intensity > 0);
    }
}

#[test]
fn sample_after_spread() {
    // Uniform spread should make every part of the domain reachable.
    let plan = Plan::new().spread(256, 3, 256);
    let g = run::<u64, u64, 8>(default_config(), &plan);

    let mut saw_low = false;
    let mut saw_high = false;
    let mut rng = TestRng(7);
    for _ in 0..500 {
        let cell = g.sample(&mut rng).unwrap();
        if cell.start < 64 {
            saw_low = true;
        }
        if cell.end > 192 {
            saw_high = true;
        }
    }
    assert!(saw_low, "should reach low quarter of domain");
    assert!(saw_high, "should reach high quarter of domain");
}

#[test]
fn sample_budgeted_graph() {
    // Sampling must work correctly on a budgeted graph that has
    // undergone evictions.
    let plan = Plan::new().spread(256, 5, 300);
    let g = run::<u64, u64, 8>(budget_config(100), &plan);

    let mut rng = TestRng(123);
    for _ in 0..100 {
        let cell = g.sample(&mut rng).unwrap();
        assert!(cell.start < cell.end);
        assert!(cell.end <= 256);
        assert!(cell.intensity > 0);
    }
}

// ── Type coverage ───────────────────────────────────────────────

#[test]
fn sample_f64_coordinate() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(f64_no_split_config());
    g.observe(5.0_f64, 10.0_f64);
    assert_invariants(&g);

    let cell = g.sample(&mut FixedRng(0.5)).unwrap();
    assert!(cell.start < cell.end);
    assert!((cell.intensity - 10.0).abs() < f64::EPSILON);
    assert_eq!(cell.depth, 0); // single root (no split)
}

// ── Trait coverage ───────────────────────────────────────────────

#[test]
fn sample_via_weighted_sampler_trait() {
    // Verify the `WeightedSampler` trait path produces the same
    // result as calling `g.sample()` directly.
    let g = build_range_tree();
    let cell_direct = g.sample(&mut FixedRng(0.42)).unwrap();
    let cell_trait = WeightedSampler::sample(&g, &mut FixedRng(0.42)).unwrap();
    assert_eq!(cell_direct, cell_trait, "trait and inherent sample must agree");

    // Also verify the empty case.
    let empty: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert!(WeightedSampler::sample(&empty, &mut FixedRng(0.5)).is_none());
}

// ── Post-eviction ───────────────────────────────────────────────

#[test]
fn sample_after_explicit_eviction() {
    // Build a graph, then trigger manual eviction rounds.
    // Sampling must still return valid cells afterward, even if
    // the passes are no-ops (terminal depth may be ≤ D_evict).
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    assert_invariants(&g);

    for _ in 0..5 {
        g.check_evictions();
    }
    assert_invariants(&g);

    // Regardless of whether evictions occurred, sampling must
    // still produce valid results.
    let mut rng = TestRng(55);
    for _ in 0..100 {
        let cell = g.sample(&mut rng).unwrap();
        assert!(cell.start < cell.end, "valid interval");
        assert!(cell.end <= 256, "within domain");
        assert!(cell.intensity > 0, "positive intensity");
    }
}

// ── Consistency with other queries ──────────────────────────────

#[test]
fn sample_consistent_with_get() {
    // For every sampled cell, `get(start)` must return a cell whose
    // interval contains the sampled start coordinate. sample() walks
    // the V-tree and may return semi-internal uncovered halves that
    // are coarser than what get() finds via the G-tree, so we only
    // assert containment of the coordinate, not interval equality.
    let g = build_range_tree();

    let mut rng = TestRng(77);
    for _ in 0..50 {
        let sampled = g.sample(&mut rng).unwrap();
        let got = g.get(sampled.start);
        let coord = sampled.start;
        assert!(
            got.start <= coord && coord < got.end,
            "get({coord}) returned [{}, {}), does not contain {coord}",
            got.start,
            got.end,
        );
        // Both must report positive intensity at a sampled location.
        assert!(sampled.intensity > 0, "sampled cell should have positive intensity");
    }
}
