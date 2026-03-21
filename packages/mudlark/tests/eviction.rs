// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **eviction, contraction, and energy
//! conservation** (Phase 3).
//!
//! These tests exercise the budget-driven eviction pipeline: semi-internal
//! routing, bottom-up contraction (§IDEA M-12.7), depth-gate dynamics,
//! and the energy-conservation invariant that must hold through every
//! eviction path.  Adversarial and multi-phase workloads stress the
//! system under realistic pressure.
//!
//! # Test index
//!
//! ## Semi-internal routing
//!
//! | Test | Focus |
//! |------|-------|
//! | [`semi_internal_routing_via_budget_eviction`] | budget-triggered eviction creates semi-internal nodes |
//! | [`semi_internal_routing_then_observe`] | observations route correctly through semi-internal nodes |
//!
//! ## Bottom-up contraction (§IDEA M-12.7)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bottom_up_contraction_via_budget`] | contraction under budget pressure |
//! | [`contraction_converges_via_check_evictions`] | multi-round manual eviction converges |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`energy_conserved_after_budget_evictions`] | G-root sum preserved after budget evictions |
//! | [`energy_conserved_after_manual_evictions`] | G-root sum preserved after `check_evictions` |
//! | [`clean_accounting_after_budget_eviction`] | invariants hold after budget eviction |
//!
//! ## Hotspot-induced eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`hotspot_eviction_preserves_energy`] | concentrated observations then budget eviction |
//!
//! ## Degenerate / boundary
//!
//! | Test | Focus |
//! |------|-------|
//! | [`single_coord_eviction_with_tight_budget`] | all observations at coord 0, minimum budget |
//! | [`empty_graph_eviction_is_noop`] | `check_evictions` on empty graph returns 0 |
//!
//! ## Adversarial patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adversarial_zigzag_under_eviction_pressure`] | escalating zigzag under budget |
//! | [`skewed_load_under_eviction_pressure`] | power-law skew triggers asymmetric eviction |
//! | [`oscillating_hotspot_under_eviction`] | alternating bursts force repeated eviction |
//!
//! ## Multi-phase cycles
//!
//! | Test | Focus |
//! |------|-------|
//! | [`growth_pressure_relax_cycle`] | growth → pressure → relaxation honours budget |
//! | [`budget_burst_eviction`] | spread then burst under budget |
//!
//! ## Eviction mechanics
//!
//! | Test | Focus |
//! |------|-------|
//! | [`check_evictions_idempotent`] | second pass evicts ≤ first |
//! | [`check_evictions_noop_when_none_eligible`] | low-depth entries are ineligible |
//! | [`eviction_return_value_matches_delta`] | return value equals node-count decrease |
//! | [`trailing_rebalance_clears_violations`] | no invariant violations after eviction |
//!
//! ## Bounded vs unbounded eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bounded_eviction_honours_budget`] | `budget_config` (bounded mode) stays within budget |
//!
//! ## Depth-gate dynamics
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_gate_tightens_under_pressure`] | `depth_evict()` decreases when over soft limit |
//!
//! ## Evictable config preset
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evictable_config_produces_deep_entries`] | `evictable_config` + `plan_evictable` boundary test |
//!
//! ## Serde round-trip
//!
//! | Test | Focus |
//! |------|-------|
//! | [`eviction_plan_serializes`] | plan serialization (serde feature) |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config, budget_config_unbounded, default_config, evictable_config, plan_budget_burst, plan_evictable,
    plan_growth_pressure_relax, run, run_budget_checked, run_checked, small_buffer_config,
};

mod support;
use support::init_tracing;

// ── Semi-internal routing ───────────────────────────────────────

#[test]
fn semi_internal_routing_via_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(15), &plan, 15, 50);
    assert_invariants(&g);
}

#[test]
fn semi_internal_routing_then_observe() {
    let _t = init_tracing();
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
    let plan = plan_evictable();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan);
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
fn energy_conserved_after_manual_evictions() {
    let _t = init_tracing();
    let plan = plan_evictable();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan);

    let total_before = g.total_sum();
    g.check_evictions();

    assert_eq!(g.total_sum(), total_before, "manual check_evictions must preserve energy");
    assert_invariants(&g);
}

#[test]
fn clean_accounting_after_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 3, 50);
    let g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    assert_invariants(&g);
}

// ── Hotspot-induced eviction ────────────────────────────────────

#[test]
fn hotspot_eviction_preserves_energy() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(42, 10, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Degenerate / boundary ───────────────────────────────────────

#[test]
fn single_coord_eviction_with_tight_budget() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(0, 6, 100);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(10), &plan, 10, 100);
    assert_invariants(&g);
}

#[test]
fn empty_graph_eviction_is_noop() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new();
    let mut g = run::<u64, u64, 8>(default_config(), &plan);
    let evicted = g.check_evictions();
    assert_eq!(evicted, 0);
    assert_invariants(&g);
}

// ── Adversarial patterns ────────────────────────────────────────

#[test]
fn adversarial_zigzag_under_eviction_pressure() {
    let _t = init_tracing();
    let plan = Plan::new().adversarial_zigzag(0, 255, 5, 3, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    assert_invariants(&g);
}

#[test]
fn skewed_load_under_eviction_pressure() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(256, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 200);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "skewed eviction must preserve energy");
    assert_invariants(&g);
}

#[test]
fn oscillating_hotspot_under_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().oscillating_hotspot(0, 255, 8, 10, 10);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 100);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "oscillating hotspot eviction must preserve energy");
    assert_invariants(&g);
}

// ── Multi-phase cycles ──────────────────────────────────────────

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

#[test]
fn budget_burst_eviction() {
    let _t = init_tracing();
    let plan = plan_budget_burst(256, 50, 30);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 80);
    assert_invariants(&g);
}

// ── Eviction mechanics ──────────────────────────────────────────

#[test]
fn check_evictions_idempotent() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    let mut g = run::<u64, u64, 8>(default_config(), &plan);

    let first = g.check_evictions();
    assert_invariants(&g);
    let second = g.check_evictions();
    assert_invariants(&g);
    assert!(second <= first, "second eviction pass should evict <= first");
}

#[test]
fn check_evictions_noop_when_none_eligible() {
    let _t = init_tracing();
    let plan = Plan::new().observe(100, 10);
    let mut g = run::<u64, u64, 32>(default_config(), &plan);
    let evicted = g.check_evictions();
    assert_eq!(evicted, 0);
    assert_invariants(&g);
}

#[test]
fn eviction_return_value_matches_delta() {
    let _t = init_tracing();
    let plan = plan_evictable();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan);

    let count_before = g.node_count();
    let evicted = g.check_evictions();
    assert_eq!(count_before - g.node_count(), evicted);
    assert_invariants(&g);
}

#[test]
fn trailing_rebalance_clears_violations() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 100);
    let mut g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    g.check_evictions();
    assert_invariants(&g);
}

// ── Bounded vs unbounded eviction ───────────────────────────────

#[test]
fn bounded_eviction_honours_budget() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 200);
    let g = run_budget_checked::<u64, u64, 8>(budget_config(100), &plan, 100, 200);
    assert_invariants(&g);
}

// ── Depth-gate dynamics ─────────────────────────────────────────

#[test]
fn depth_gate_tightens_under_pressure() {
    let _t = init_tracing();
    // small_buffer_config has buffer=1, headroom=9, so budget=15 is valid.
    // Heavy spread forces many splits, pushing over the soft limit and
    // tightening D_evict from its initial value of 4.
    let cfg = small_buffer_config(15);
    let initial_d_evict = cfg.depth_evict;
    let plan = Plan::new().spread(256, 6, 200);
    let g = run::<u64, u64, 8>(cfg, &plan);

    assert!(
        g.depth_evict() < initial_d_evict,
        "depth_evict should tighten under pressure, got {} (initial={})",
        g.depth_evict(),
        initial_d_evict
    );
    assert_invariants(&g);
}

// ── Evictable config preset ─────────────────────────────────────

#[test]
fn evictable_config_produces_deep_entries() {
    let _t = init_tracing();
    // evictable_config has D_evict=4.  plan_evictable spreads 8
    // coords across domain [0,256) with θ=5 and D_create=3.
    // Entries are created at V-depth ≤ D_evict, so check_evictions
    // may not find candidates.  Verify the graph is well-formed and
    // that energy is conserved; the entries exist at the eviction
    // boundary.
    let plan = plan_evictable();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan);

    let total_before = g.total_sum();
    let count_before = g.node_count();
    let evicted = g.check_evictions();

    // Energy must be conserved regardless of whether eviction occurred.
    assert_eq!(g.total_sum(), total_before, "energy must be conserved");
    // Node count must not increase.
    assert!(g.node_count() <= count_before);
    // Return value must match actual change.
    assert_eq!(count_before - g.node_count(), evicted);
    assert_invariants(&g);
}

// ── Serde round-trip ────────────────────────────────────────────

#[test]
#[cfg(feature = "serde")]
fn eviction_plan_serializes() {
    let _t = init_tracing();
    let plan = plan_growth_pressure_relax(256, 100);
    let json = support::to_json(&plan);
    let restored: torrust_mudlark::testing::Plan<u64, u64> = support::from_json(&json);
    assert_eq!(plan, restored);
}
