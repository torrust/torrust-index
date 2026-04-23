// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Diagnostic integration tests for **eviction** and **plateau
//! maintenance**.
//!
//! Each test replays an observation plan using the diagnostic
//! runners (`run_diagnostic` / `run_diagnostic_budgeted`) that
//! check **every** invariant after **every** observation and dump
//! full G-tree + plateau state on the first violation.
//!
//! These complement `eviction.rs` (which uses `run_checked` /
//! `run_budget_checked`) by catching failures at the exact
//! observation that triggers them — invaluable when a bug hides
//! behind a long plan and the final-state check alone is not enough
//! to pinpoint the cause.
//!
//! # Test index
//!
//! ## Semi-internal routing
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_semi_internal_routing_via_budget`] | budgeted spread triggers semi-internal routing |
//! | [`diag_semi_internal_routing_then_observe`] | semi-internal routing followed by a single observation |
//! | [`diag_semi_internal_surviving_child`] | surviving child after semi-internal merge |
//!
//! ## Budget eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_budget_burst_eviction`] | burst plan under tight budget |
//! | [`diag_clean_accounting_after_budget_eviction`] | node counts and sums consistent after budget eviction |
//! | [`diag_single_coord_tight_budget`] | hotspot at a single coordinate under tight budget |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_energy_conserved_after_budget_evictions`] | total energy equals plan sum after evictions |
//!
//! ## Growth / pressure / relaxation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_growth_pressure_relax_cycle`] | grow → pressure → relax cycle with diagnostics |
//!
//! ## Adversarial patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_adversarial_zigzag_under_pressure`] | zigzag adversarial plan under budget pressure |
//!
//! ## Hotspot eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_hotspot_eviction_preserves_energy`] | energy conserved after hotspot eviction |
//!
//! ## Eviction idempotence
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_eviction_idempotent`] | second eviction pass evicts ≤ first |
//!
//! ## Contraction convergence
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_contraction_converges`] | repeated eviction rounds converge (monotonically non-increasing) |
//!
//! ## Eviction no-op
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diag_eviction_noop_when_none_eligible`] | no evictions when no nodes are deep enough |

use torrust_mudlark::Config;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config_unbounded, default_config, plan_budget_burst, plan_growth_pressure_relax, run_diagnostic,
    run_diagnostic_budgeted, small_buffer_config,
};

mod support;
use support::init_tracing;

// ── Semi-internal routing ───────────────────────────────────────

#[test]
fn diag_semi_internal_routing_via_budget() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "semi_internal_routing_via_budget");
}

#[test]
fn diag_semi_internal_routing_then_observe() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50).observe(200, 5);
    let g = run_diagnostic::<u64, u64, 8>(small_buffer_config(15), &plan, "semi_internal_routing_then_observe");
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy must be conserved through semi-internal routing");
}

#[test]
fn diag_semi_internal_surviving_child() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    let _g = run_diagnostic::<u64, u64, 8>(small_buffer_config(15), &plan, "semi_internal_surviving_child");
}

// ── Budget eviction ─────────────────────────────────────────────

#[test]
fn diag_budget_burst_eviction() {
    let _t = init_tracing();
    let plan = plan_budget_burst(256, 20, 10);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, "budget_burst");
}

#[test]
fn diag_clean_accounting_after_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "clean_accounting");
}

#[test]
fn diag_single_coord_tight_budget() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(0, 6, 100);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(small_buffer_config(10), &plan, 10, "single_coord_tight_budget");
}

// ── Energy conservation ─────────────────────────────────────────

#[test]
fn diag_energy_conserved_after_budget_evictions() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 100);
    let g = run_diagnostic::<u64, u64, 8>(budget_config_unbounded(100), &plan, "energy_conserved");
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "G-root sum must be preserved (energy conservation)");
}

// ── Growth / pressure / relaxation ──────────────────────────────

#[test]
fn diag_growth_pressure_relax_cycle() {
    let _t = init_tracing();
    let plan = plan_growth_pressure_relax(256, 100);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "growth_pressure_relax");
}

// ── Adversarial patterns ────────────────────────────────────────

#[test]
fn diag_adversarial_zigzag_under_pressure() {
    let _t = init_tracing();
    let plan = Plan::new().adversarial_zigzag(0, 255, 5, 3, 200);
    let _g = run_diagnostic_budgeted::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, "adversarial_zigzag");
}

// ── Hotspot eviction ────────────────────────────────────────────

#[test]
fn diag_hotspot_eviction_preserves_energy() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(42, 10, 200);
    let g = run_diagnostic_budgeted::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, "hotspot_eviction");
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy must be conserved after hotspot eviction");
}

// ── Eviction idempotence ────────────────────────────────────────

#[test]
fn diag_eviction_idempotent() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    let mut g = run_diagnostic::<u64, u64, 8>(default_config(), &plan, "eviction_idempotent_build");

    let first = g.check_evictions();
    assert_invariants(&g);
    let second = g.check_evictions();
    assert_invariants(&g);
    assert!(
        second <= first,
        "second eviction pass should evict <= first (got {second} > {first})"
    );
}

// ── Contraction convergence ─────────────────────────────────────

#[test]
fn diag_contraction_converges() {
    let _t = init_tracing();
    let cfg = Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 4,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = Plan::new()
        .observe(0u64, 6u64)
        .observe(128, 6)
        .observe(64, 6)
        .observe(192, 6)
        .observe(32, 6)
        .observe(96, 6)
        .observe(160, 6)
        .observe(224, 6);

    let mut g = run_diagnostic::<u64, u64, 8>(cfg, &plan, "contraction_build");
    let count_before = g.node_count();
    assert!(count_before > 3);

    // Multiple rounds should converge (monotonically non-increasing).
    let mut prev = count_before;
    for _ in 0..5 {
        g.check_evictions();
        assert_invariants(&g);
        let now = g.node_count();
        assert!(now <= prev, "eviction should not increase node count ({now} > {prev})");
        prev = now;
    }
}

// ── Eviction noop ───────────────────────────────────────────────

#[test]
fn diag_eviction_noop_when_none_eligible() {
    let _t = init_tracing();
    let plan = Plan::new().observe(100u64, 10u64);
    let mut g = run_diagnostic::<u64, u64, 32>(default_config(), &plan, "eviction_noop");
    let evicted = g.check_evictions();
    assert_eq!(evicted, 0, "no entries should be deep enough to evict");
    assert_invariants(&g);
}
