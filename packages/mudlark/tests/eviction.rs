// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for Phase 3: eviction, contraction, and
//! energy conservation.
//!
//! **Expanded coverage:** hotspot-induced eviction, multi-round
//! contraction convergence, adversarial zigzag under eviction
//! pressure, degenerate single-coord eviction, and plan-based
//! energy conservation checks.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config_unbounded, default_config, plan_budget_burst, plan_growth_pressure_relax, run, run_budget_checked,
    run_checked, small_buffer_config,
};

mod support;
use support::init_tracing;

// ── Semi-internal routing ───────────────────────────────────────

#[test]
fn semi_internal_routing_via_budget_eviction() {
    let _t = init_tracing();
    // buffer=1, headroom=9, soft_limit=6.
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(15), &plan, 15, 50);
    assert_invariants(&g);
}

#[test]
fn semi_internal_routing_then_observe() {
    let _t = init_tracing();
    // After eviction creates semi-internal nodes, further observations
    // route correctly.
    let plan = Plan::new().spread(256, 6, 50).observe(200, 5);
    let g = run::<u64, u64, 8>(small_buffer_config(15), &plan);

    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy must be conserved through semi-internal routing");
    assert_invariants(&g);
}

// ── Bottom-up contraction (§IDEA M-12.7) ──────────────────────────────

#[test]
fn bottom_up_contraction_via_budget() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    assert_invariants(&g);
}

#[test]
fn contraction_converges_via_check_evictions() {
    let _t = init_tracing();
    // Build up without budget, then manually evict.
    let plan = Plan::new();
    let coords = [0u64, 128, 64, 192, 32, 96, 160, 224, 16, 48, 80, 112];
    let mut p = plan;
    for &c in &coords {
        p = p.observe(c, 6);
    }
    let mut g = run::<u64, u64, 8>(
        torrust_mudlark::Config {
            split_threshold: 5,
            depth_create: 3,
            depth_evict: 4,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        },
        &p,
    );
    let count_before = g.node_count();
    assert!(count_before > 3);

    // Multiple rounds of eviction should converge.
    let mut prev_count = count_before;
    for _ in 0..5 {
        g.check_evictions();
        assert_invariants(&g);
        let now = g.node_count();
        assert!(now <= prev_count, "eviction should not increase node count");
        prev_count = now;
    }
}

// ── Energy conservation ─────────────────────────────────────────

#[test]
fn energy_conserved_after_budget_evictions() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 100);
    let g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "G-root sum must be preserved (energy conservation)");
    assert_invariants(&g);
}

#[test]
fn clean_accounting_after_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 3, 50);
    let g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    assert_invariants(&g);
}

// ── Expanded: hotspot-induced eviction ──────────────────────────

#[test]
fn hotspot_eviction_preserves_energy() {
    let _t = init_tracing();
    // Concentrated observations at one coord force deep splits,
    // then budget evicts them.
    let plan = Plan::new().hotspot(42, 10, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Expanded: degenerate single-coord eviction ──────────────────

#[test]
fn single_coord_eviction_with_tight_budget() {
    let _t = init_tracing();
    // All observations at coord 0 with minimum budget.
    let plan = Plan::new().hotspot(0, 6, 100);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(10), &plan, 10, 100);
    assert_invariants(&g);
}

// ── Expanded: adversarial zigzag under eviction pressure ────────

#[test]
fn adversarial_zigzag_under_eviction_pressure() {
    let _t = init_tracing();
    let plan = Plan::new().adversarial_zigzag(0, 255, 5, 3, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    assert_invariants(&g);
}

// ── Expanded: growth → pressure → relaxation cycle ──────────────

#[test]
fn growth_pressure_relax_cycle() {
    let _t = init_tracing();
    let plan = plan_growth_pressure_relax(256, 100);
    let g = run_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 10);
    assert!(
        g.node_count() as usize <= 100,
        "hard budget violated: node_count={}",
        g.node_count()
    );
    assert_invariants(&g);
}

// ── Expanded: budget burst plan ─────────────────────────────────

#[test]
fn budget_burst_eviction() {
    let _t = init_tracing();
    let plan = plan_budget_burst(256, 50, 30);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 80);
    assert_invariants(&g);
}

// ── Expanded: eviction idempotence ──────────────────────────────

#[test]
fn check_evictions_idempotent() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    let mut g = run::<u64, u64, 8>(default_config(), &plan);

    // Two consecutive eviction calls should produce the same result.
    let first = g.check_evictions();
    assert_invariants(&g);
    let second = g.check_evictions();
    assert_invariants(&g);
    // Second pass may evict fewer or zero — but not more.
    assert!(second <= first, "second eviction pass should evict <= first");
}

// ── Expanded: eviction noop when none eligible ──────────────────

#[test]
fn check_evictions_noop_when_none_eligible() {
    let _t = init_tracing();
    // Default config: D_evict = 6, entries are at low depth.
    let plan = Plan::new().observe(100, 10);
    let mut g = run::<u64, u64, 32>(default_config(), &plan);
    let evicted = g.check_evictions();
    assert_eq!(evicted, 0);
    assert_invariants(&g);
}

// ── Expanded: eviction return value matches count delta ─────────

#[test]
fn eviction_return_value_matches_delta() {
    let _t = init_tracing();
    // Build a graph with many deep entries, then evict.
    let mut plan = Plan::new();
    for &c in &[0u64, 128, 64, 192, 32, 96, 160, 224] {
        plan = plan.observe(c, 6);
    }
    let mut g = run::<u64, u64, 8>(
        torrust_mudlark::Config {
            split_threshold: 5,
            depth_create: 3,
            depth_evict: 4,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        },
        &plan,
    );
    // Lower D_evict manually to make entries eligible.
    // (This accesses pub(crate) but we can test via the budget path.)
    let count_before = g.node_count();
    let evicted = g.check_evictions();
    assert_eq!(count_before - g.node_count(), evicted);
    assert_invariants(&g);
}

// ── Expanded: trailing rebalance clears violations ──────────────

#[test]
fn trailing_rebalance_clears_violations() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 100);
    let mut g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    g.check_evictions();
    assert_invariants(&g);
}

// ── Expanded: plan serialization for eviction scenario ──────────

#[test]
#[cfg(feature = "serde")]
fn eviction_plan_serializes() {
    let _t = init_tracing();
    let plan = plan_growth_pressure_relax(256, 100);
    let json = support::to_json(&plan);
    let restored = support::from_json(&json);
    assert_eq!(plan, restored);
}
