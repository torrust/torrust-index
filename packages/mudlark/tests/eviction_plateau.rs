// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Integration tests for eviction ↔ plateau invariant maintenance.
//!
//! These tests exercise the plateau invariants (P-I2 basis
//! minimality, P-I4 thatch one-hop, P-I5 thatch depth) through
//! eviction-triggering scenarios.  They complement `eviction.rs`
//! (general eviction correctness) and `plateau.rs` (plateau
//! structure without budget pressure).
//!
//! # Test index
//!
//! ## Evictable-graph reproduction (original P-I4 regression)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evictable_graph_checked`] | 8-coord evictable plan, invariants every step |
//! | [`evictable_graph_budget_checked`] | same plan with hard budget assertion |
//!
//! ## Energy conservation through eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`energy_conserved_after_evictable_plan`] | `total_sum` matches input |
//! | [`energy_conserved_spread_under_budget`] | spread plan + budget evictions |
//!
//! ## Plateau count and tiling after eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateaus_valid_after_budget_eviction`] | plateaus structurally valid |
//! | [`plateau_count_nonzero_after_eviction`] | at least one plateau survives |
//! | [`plateau_depths_plausible_after_eviction`] | depths bounded by *N* |
//!
//! ## Manual `check_evictions` + plateau stability
//!
//! | Test | Focus |
//! |------|-------|
//! | [`manual_check_evictions_preserves_plateaus`] | plateaus valid after manual evict |
//! | [`repeated_check_evictions_stable`] | idempotence of eviction on plateaus |
//!
//! ## Stress / adversarial patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`spread_stress_plateaus_valid`] | 50-observation spread under tight budget |
//! | [`adversarial_zigzag_plateau_survives`] | extreme zigzag under budget |
//! | [`oscillating_hotspot_plateau_survives`] | alternating hotspot eviction cycles |
//! | [`burst_then_evict_plateaus_valid`] | spread + hotspot burst under budget |
//!
//! ## Soft-checked diagnostic
//!
//! | Test | Focus |
//! |------|-------|
//! | [`soft_checked_no_violations`] | `run_soft_checked` detects zero violations |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config_unbounded, plan_evictable, run, run_budget_checked, run_checked, run_soft_checked, small_buffer_config,
};
mod support;
use support::init_tracing;

// ── Evictable-graph reproduction (original P-I4 regression) ─────

/// Reproduce the exact observation sequence from `build_evictable_graph`:
/// spread 8 observations at coords [0, 128, 64, 192, 32, 96, 160, 224]
/// with delta=6, using `depth_create=3`, `depth_evict=4`, and a tight
/// budget so evictions are automatically triggered.
///
/// Invariants are checked after every single observation.
#[test]
fn evictable_graph_checked() {
    let _t = init_tracing();

    // small_buffer_config: depth_create=3, depth_evict=4, buffer=1, headroom=9.
    // Budget 15 → soft_limit = 15 - 9 = 6.
    let cfg = small_buffer_config(15);
    let plan = plan_evictable();
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);
}

/// Same scenario with hard budget assertion at every step.
#[test]
fn evictable_graph_budget_checked() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = plan_evictable();
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 15, 1);
    assert_invariants(&g);
}

// ── Energy conservation through eviction ────────────────────────

/// The evictable plan's total energy must be preserved even after
/// budget-triggered evictions merge nodes.
#[test]
fn energy_conserved_after_evictable_plan() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = plan_evictable();
    let g = run::<u64, u64, 8>(cfg, &plan);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected, "energy must be conserved");
    assert_invariants(&g);
}

/// A broader spread under budget pressure must conserve energy.
#[test]
fn energy_conserved_spread_under_budget() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().spread(256, 6, 50);
    let g = run::<u64, u64, 8>(cfg, &plan);
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected);
    assert_invariants(&g);
}

// ── Plateau count and tiling after eviction ─────────────────────

/// After budget-triggered evictions, the plateau map must have at
/// least one entry whose key starts at the domain origin, and all
/// structural invariants (including P-I1 tiling) must hold.
#[test]
fn plateaus_valid_after_budget_eviction() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 15, 50);
    let plateaus = g.plateaus();

    // Domain for N=8 is [0, 256).
    let first_key = plateaus.keys().next().expect("at least one plateau");
    assert_eq!(first_key.0, 0, "first plateau must start at domain origin");

    // All plateau sums must be non-negative.
    for p in plateaus.values() {
        assert!(p.sum > 0 || p.depth == 0, "plateau sum should be positive (or root-level)");
    }

    // Full invariant suite (includes P-I1 tiling checks).
    assert_invariants(&g);
}

/// At least one plateau must survive eviction.
#[test]
fn plateau_count_nonzero_after_eviction() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = plan_evictable();
    let g = run::<u64, u64, 8>(cfg, &plan);
    assert!(!g.plateaus().is_empty(), "must have at least one plateau");
    assert_invariants(&g);
}

/// Plateau depths must not exceed N (the tree arity — maximum
/// possible G-tree depth for the domain).
#[test]
fn plateau_depths_plausible_after_eviction() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 15, 50);
    for plateau in g.plateaus().values() {
        assert!(plateau.depth <= 8, "plateau depth {} exceeds N=8", plateau.depth);
    }
    assert_invariants(&g);
}

// ── Manual check_evictions + plateau stability ──────────────────

/// Build without budget, then manually trigger `check_evictions` and
/// verify all plateau invariants hold afterward.
#[test]
fn manual_check_evictions_preserves_plateaus() {
    let _t = init_tracing();

    let cfg = torrust_mudlark::Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 4,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let plan = plan_evictable();
    let mut g = run::<u64, u64, 8>(cfg, &plan);
    assert_invariants(&g);

    let count_before = g.node_count();
    let evicted = g.check_evictions();
    tracing::debug!(evicted, count_before, count_after = g.node_count(), "post-eviction");

    // Node count must not increase.
    assert!(g.node_count() <= count_before);
    // Return value must match the delta.
    assert_eq!(count_before - g.node_count(), evicted);

    // Plateaus must still be valid.
    assert_invariants(&g);

    // Energy must be conserved.
    let expected: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), expected);
}

/// Two consecutive `check_evictions` calls must converge: the second
/// call should not evict more than the first, and plateaus must be
/// valid after both.
#[test]
fn repeated_check_evictions_stable() {
    let _t = init_tracing();

    let cfg = torrust_mudlark::Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 4,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    // Use a bigger observation set so there's something to evict.
    let plan = Plan::new().spread(256, 6, 30);
    let mut g = run::<u64, u64, 8>(cfg, &plan);

    let first = g.check_evictions();
    assert_invariants(&g);

    let second = g.check_evictions();
    assert_invariants(&g);

    assert!(
        second <= first,
        "second eviction pass should not evict more than first: {second} > {first}"
    );
}

// ── Stress / adversarial patterns ───────────────────────────────

/// Broader spread that exercises more eviction + re-split cycles.
#[test]
fn spread_stress_plateaus_valid() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().spread(256, 6, 50);
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);
}

/// Adversarial zigzag between domain extremes under tight budget.
#[test]
fn adversarial_zigzag_plateau_survives() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().adversarial_zigzag(0, 255, 5, 3, 40);
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);

    // Plateaus must still tile the domain.
    let plateaus = g.plateaus();
    assert!(!plateaus.is_empty());
    let first_key = plateaus.keys().next().unwrap();
    assert_eq!(first_key.0, 0);
}

/// Oscillating hotspot: alternating bursts at opposite ends of the
/// domain force repeated deep refinement and eviction.
#[test]
fn oscillating_hotspot_plateau_survives() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = Plan::new().oscillating_hotspot(0, 255, 6, 5, 6);
    let g = run_checked::<u64, u64, 8>(cfg, &plan, 1);
    assert_invariants(&g);
}

/// Burst pattern (spread then concentrated spike) under budget.
#[test]
fn burst_then_evict_plateaus_valid() {
    let _t = init_tracing();

    let cfg = budget_config_unbounded(100);
    let plan = Plan::new().burst(256, 6, 30, 42u64, 50, 20);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 100, 50);
    assert_invariants(&g);
}

// ── Soft-checked diagnostic ─────────────────────────────────────

/// Use `run_soft_checked` to verify no invariant violations are
/// collected during the evictable-graph plan.
#[test]
fn soft_checked_no_violations() {
    let _t = init_tracing();

    let cfg = small_buffer_config(15);
    let plan = plan_evictable();
    let (g, errors) = run_soft_checked::<u64, u64, 8>(cfg, &plan, 1);

    if !errors.is_empty() {
        for (step, violations) in &errors {
            eprintln!("step {step}:");
            for v in violations {
                eprintln!("  {v}");
            }
        }
        panic!(
            "soft_checked found {} violation(s) across {} step(s)",
            errors.iter().map(|(_, v)| v.len()).sum::<usize>(),
            errors.len(),
        );
    }
    assert_invariants(&g);
}
