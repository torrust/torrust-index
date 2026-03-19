// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Diagnostic integration tests for eviction plateau maintenance.
//!
//! These tests replay the exact observation sequences from failing
//! tests but check invariants after EVERY observation, dumping full
//! plateau + G-tree state when the first violation is detected.
//!
//! Run with:
//!   `cargo test --test eviction_debug -- --nocapture`

use torrust_mudlark::invariants::{check_all_invariants, dump_gtree, dump_plateaus};
use torrust_mudlark::testing::{
    Plan, budget_config_unbounded, plan_budget_burst, plan_growth_pressure_relax, small_buffer_config,
};
use torrust_mudlark::{Accumulator, Config, Coordinate, GvGraph, Inspectable};

mod support;
use support::init_tracing;

/// Run a plan one observation at a time, checking invariants after each.
/// On first failure, dump full diagnostic state and panic.
fn run_one_at_a_time<C, V, const N: u32>(config: Config<V>, plan: &Plan<C, V>, label: &str) -> GvGraph<C, V, N>
where
    C: Coordinate + std::fmt::Display + std::fmt::Debug,
    V: Accumulator + Inspectable + std::fmt::Display + std::fmt::Debug,
{
    let mut g = GvGraph::new(config);

    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        // Snapshot state BEFORE this observation.
        let pre_gtree = dump_gtree::<C, V, N>(&g);
        let pre_plateaus = dump_plateaus::<C, V, N>(&g);

        g.observe(coord, delta);

        let errors = check_all_invariants::<C, V, N>(&g);
        if !errors.is_empty() {
            let post_gtree = dump_gtree::<C, V, N>(&g);
            let post_plateaus = dump_plateaus::<C, V, N>(&g);

            eprintln!("\n╔══════════════════════════════════════════════════╗");
            eprintln!("║  FIRST VIOLATION in [{label}]                    ║");
            eprintln!("║  observation #{i}: observe({coord}, {delta})     ║");
            eprintln!("║  node_count = {}                                 ║", g.node_count());
            eprintln!("╚══════════════════════════════════════════════════╝");
            eprintln!("\n── BEFORE observation #{i} ──");
            eprintln!("{pre_gtree}");
            eprintln!("{pre_plateaus}");
            eprintln!("\n── AFTER observation #{i} ──");
            eprintln!("{post_gtree}");
            eprintln!("{post_plateaus}");
            eprintln!("\n── Violations ({}) ──", errors.len());
            for e in &errors {
                eprintln!("  • {e}");
            }
            panic!(
                "[{label}] invariant violated after observation #{i}: \
                 observe({coord}, {delta}). {} violations. \
                 See stderr for full dump.",
                errors.len()
            );
        }
    }
    g
}

/// Same as `run_one_at_a_time` but with a hard budget — uses
/// `check_evictions` after each observation when over budget.
fn run_budgeted_one_at_a_time<C, V, const N: u32>(
    config: Config<V>,
    plan: &Plan<C, V>,
    budget: usize,
    label: &str,
) -> GvGraph<C, V, N>
where
    C: Coordinate + std::fmt::Display + std::fmt::Debug,
    V: Accumulator + Inspectable + std::fmt::Display + std::fmt::Debug,
{
    let mut g = GvGraph::new(config);

    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        // Snapshot state BEFORE this observation.
        let pre_gtree = dump_gtree::<C, V, N>(&g);
        let pre_plateaus = dump_plateaus::<C, V, N>(&g);

        g.observe(coord, delta);

        // Check invariants after observe, before any eviction.
        let post_observe_errors = check_all_invariants::<C, V, N>(&g);
        if !post_observe_errors.is_empty() {
            let post_gtree = dump_gtree::<C, V, N>(&g);
            let post_plateaus = dump_plateaus::<C, V, N>(&g);
            eprintln!("\n╔══════════════════════════════════════════════════╗");
            eprintln!("║  VIOLATION after observe (pre-eviction)           ║");
            eprintln!("║  [{label}] obs #{i}: observe({coord}, {delta})    ║");
            eprintln!("╚══════════════════════════════════════════════════╝");
            eprintln!("\n── BEFORE ──\n{pre_gtree}\n{pre_plateaus}");
            eprintln!("\n── AFTER observe ──\n{post_gtree}\n{post_plateaus}");
            eprintln!("\n── Violations ({}) ──", post_observe_errors.len());
            for e in &post_observe_errors {
                eprintln!("  • {e}");
            }
            panic!(
                "[{label}] invariant violated after observe #{i}. \
                 {} violations.",
                post_observe_errors.len()
            );
        }

        // If over budget, evict one-at-a-time to find the culprit.
        if g.node_count() as usize > budget {
            let pre_evict_gtree = dump_gtree::<C, V, N>(&g);
            let pre_evict_plateaus = dump_plateaus::<C, V, N>(&g);

            g.check_evictions();

            let post_evict_errors = check_all_invariants::<C, V, N>(&g);
            if !post_evict_errors.is_empty() {
                let post_evict_gtree = dump_gtree::<C, V, N>(&g);
                let post_evict_plateaus = dump_plateaus::<C, V, N>(&g);
                eprintln!("\n╔══════════════════════════════════════════════════╗");
                eprintln!("║  VIOLATION after check_evictions                  ║");
                eprintln!("║  [{label}] obs #{i}: observe({coord}, {delta})    ║");
                eprintln!("╚══════════════════════════════════════════════════╝");
                eprintln!("\n── BEFORE eviction ──\n{pre_evict_gtree}\n{pre_evict_plateaus}");
                eprintln!("\n── AFTER eviction ──\n{post_evict_gtree}\n{post_evict_plateaus}");
                eprintln!("\n── Violations ({}) ──", post_evict_errors.len());
                for e in &post_evict_errors {
                    eprintln!("  • {e}");
                }
                panic!(
                    "[{label}] invariant violated after check_evictions at obs #{i}. \
                     {} violations.",
                    post_evict_errors.len()
                );
            }
        }
    }
    g
}

// ── Replays of failing tests ────────────────────────────────────

#[test]
fn debug_semi_internal_routing_via_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    run_budgeted_one_at_a_time::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "semi_internal_routing_via_budget");
}

#[test]
fn debug_semi_internal_routing_then_observe() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50).observe(200, 5);
    run_one_at_a_time::<u64, u64, 8>(small_buffer_config(15), &plan, "semi_internal_routing_then_observe");
}

#[test]
fn debug_semi_internal_surviving_child_unaffected() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 30);
    run_one_at_a_time::<u64, u64, 8>(small_buffer_config(15), &plan, "semi_internal_surviving_child");
}

#[test]
fn debug_budget_burst_eviction() {
    let _t = init_tracing();
    let plan = plan_budget_burst(256, 20, 10);
    run_budgeted_one_at_a_time::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, "budget_burst");
}

#[test]
fn debug_energy_conserved_after_budget_evictions() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 100);
    run_one_at_a_time::<u64, u64, 8>(budget_config_unbounded(100), &plan, "energy_conserved");
}

#[test]
fn debug_growth_pressure_relax_cycle() {
    let _t = init_tracing();
    let plan = plan_growth_pressure_relax(256, 100);
    run_budgeted_one_at_a_time::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "growth_pressure_relax");
}

#[test]
fn debug_clean_accounting_after_budget_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    run_budgeted_one_at_a_time::<u64, u64, 8>(small_buffer_config(15), &plan, 15, "clean_accounting");
}
