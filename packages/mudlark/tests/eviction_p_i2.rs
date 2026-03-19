// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Targeted test reproducing the P-I4 thatch violation discovered
//! during Step 2D (evict plateau maintenance).
//!
//! The original unit test `check_evictions_bounded_respects_limit`
//! manually lowered `live_depth_evict` then called `check_evictions_bounded(1)`.
//! This integration test reproduces the same structural scenario
//! through the public API using budget-triggered eviction.

use torrust_mudlark::Config;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, run, run_budget_checked, run_checked, small_buffer_config};

mod support;
use support::init_tracing;

/// Reproduce the exact observation sequence from `build_evictable_graph`:
/// spread 8 observations at coords [0, 128, 64, 192, 32, 96, 160, 224]
/// with delta=6, using `depth_create=3`, `depth_evict=4`, and a tight budget
/// so evictions are automatically triggered.
#[test]
fn p_i2_eviction_bounded_repro() {
    let _t = init_tracing();

    // small_buffer_config: depth_create=3, depth_evict=4, buffer=1, headroom=9.
    // Budget 15 → soft_limit = 15 - 9 = 6.
    // The 8 observations each trigger splits; the budget forces eviction.
    let cfg = small_buffer_config(15);
    let plan = Plan::new()
        .observe(0u64, 6u64)
        .observe(128, 6)
        .observe(64, 6)
        .observe(192, 6)
        .observe(32, 6)
        .observe(96, 6)
        .observe(160, 6)
        .observe(224, 6);

    // Run with invariant checks after every observation.
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);
}

/// Same scenario but checking budget at every step.
#[test]
fn p_i2_eviction_budget_checked_repro() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new()
        .observe(0u64, 6u64)
        .observe(128, 6)
        .observe(64, 6)
        .observe(192, 6)
        .observe(32, 6)
        .observe(96, 6)
        .observe(160, 6)
        .observe(224, 6);

    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 15, 1);
    assert_invariants(&g);
}

/// Broader spread that exercises more eviction + re-split cycles.
#[test]
fn p_i2_eviction_spread_stress() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);
}

/// Step-by-step reproduction: build graph without budget, then manually
/// trigger `check_evictions` to isolate the eviction path.
#[test]
fn p_i2_eviction_manual_check_evictions() {
    let _t = init_tracing();

    let cfg = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 4,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut plan = Plan::new();
    for &c in &[0u64, 128, 64, 192, 32, 96, 160, 224] {
        plan = plan.observe(c, 6);
    }
    let mut g = run::<u64, u64, 8>(cfg, &plan);
    let count_before = g.node_count();
    tracing::debug!(count_before, "pre-eviction node count");

    assert_invariants(&g);

    // Trigger evictions (entries may or may not be deep enough with
    // default D_evict=4 — check_evictions uses live_depth_evict).
    let evicted = g.check_evictions();
    tracing::debug!(evicted, count_after = g.node_count(), "post-eviction");

    assert_invariants(&g);
}
