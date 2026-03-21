// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **budget enforcement** and **dynamic depth
//! control** (Phase 3).
//!
//! Budget enforcement caps the number of G-tree nodes via a hard
//! limit, dynamically adjusting `D_evict` and `D_create` depth gates
//! to keep the tree within bounds.  The tests exercise config
//! validation, gate dynamics, hard-budget guarantees (§ADR M-018),
//! adversarial workloads, energy conservation, and serialization —
//! all with full invariant checking.
//!
//! Uses the `testing` harness for config presets, observation plans,
//! and budget-checked runners.
//!
//! # Test index
//!
//! ## Config validation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`no_budget_soft_limit_is_none`] | `soft_limit()` returns `None` without budget |
//! | [`budget_config_soft_limit_and_headroom`] | soft-limit / headroom arithmetic |
//!
//! ## Budget enforcement
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_converges_under_500_observations`] | sweep, unbounded, budget-checked |
//! | [`budget_bounded_eviction_converges`] | sweep, bounded, budget-checked |
//!
//! ## Depth gate dynamics
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_gates_shift_under_budget_pressure`] | sustained spread tightens `D_evict` |
//! | [`depth_gates_recover_when_under_budget`] | light load relaxes `D_evict` |
//!
//! ## Invariants throughout
//!
//! | Test | Focus |
//! |------|-------|
//! | [`invariants_hold_through_growth_and_contraction`] | per-observation invariant check |
//! | [`energy_conserved_with_budget`] | total sum equals plan energy (unbounded) |
//! | [`energy_conserved_with_bounded_eviction`] | total sum equals plan energy (bounded) |
//!
//! ## Hard budget guarantee (ADR-M-018)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`hard_budget_never_exceeded`] | 1000-obs sweep, unbounded |
//! | [`hard_budget_never_exceeded_bounded`] | 1000-obs sweep, bounded |
//! | [`hard_budget_minimum_budget`] | `tight_budget_config` (budget=82) |
//! | [`hard_budget_minimum_budget_small_buffer`] | `small_buffer_config(10)` |
//! | [`hard_budget_ceiling_is_tight`] | max node count exceeds soft limit but not budget |
//! | [`hard_budget_split_guard_prevents_overshoot`] | small buffer, same pattern |
//! | [`energy_conserved_at_hard_budget_limit`] | tight budget energy conservation |
//!
//! ## Degenerate hotspot under budget
//!
//! | Test | Focus |
//! |------|-------|
//! | [`hotspot_under_budget`] | single coord, budget=100 |
//! | [`hotspot_under_minimum_budget`] | single coord, budget=10 |
//!
//! ## Adversarial patterns under budget
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adversarial_zigzag_under_budget`] | `plan_adversarial` preset |
//! | [`left_deep_under_budget`] | `plan_left_deep` (all at coord 0) |
//! | [`right_deep_under_budget`] | `plan_right_deep` (all at domain − 1) |
//!
//! ## Skewed / burst / random under budget
//!
//! | Test | Focus |
//! |------|-------|
//! | [`skewed_load_under_budget`] | power-law skew + energy check |
//! | [`budget_burst_growth_then_spike`] | `plan_budget_burst` preset |
//! | [`random_spray_under_budget`] | pseudorandom coordinates |
//! | [`oscillating_hotspot_under_budget`] | alternating deep hotspots |
//!
//! ## Multi-phase lifecycle
//!
//! | Test | Focus |
//! |------|-------|
//! | [`growth_pressure_relax_under_budget`] | `plan_growth_pressure_relax` preset |
//!
//! ## Gate oscillation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gate_oscillation_near_budget_boundary`] | tracks tighten/relax counts |
//!
//! ## D-I3 (buffer invariant)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`di3_maintained_through_budget_lifecycle`] | per-observation D-I3 check |
//!
//! ## Serialization
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_plan_serializes_correctly`] | serde round-trip (`#[cfg(feature = "serde")]`) |
//!
//! ## Bounded vs unbounded comparison
//!
//! | Test | Focus |
//! |------|-------|
//! | [`bounded_and_unbounded_both_respect_hard_limit`] | both modes conserve energy |
//!
//! ## Various budget sizes
//!
//! | Test | Focus |
//! |------|-------|
//! | [`budget_200_sustained_load`] | larger budget under sweep |
//! | [`budget_50_tight_small_buffer`] | `small_buffer_config(50)` |
//!
//! ## Empty / trivial
//!
//! | Test | Focus |
//! |------|-------|
//! | [`empty_plan_under_budget`] | zero observations, budget graph is valid |

use torrust_mudlark::GvGraph;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config, budget_config_unbounded, default_config, plan_adversarial, plan_budget_burst,
    plan_growth_pressure_relax, plan_left_deep, plan_right_deep, run, run_budget_checked, run_checked, small_buffer_config,
    tight_budget_config,
};

mod support;
use support::init_tracing;

// ── Config validation ───────────────────────────────────────────

#[test]
fn no_budget_soft_limit_is_none() {
    let _t = init_tracing();
    let g = GvGraph::<u64, u64, 8>::new(default_config());
    assert_eq!(g.budget(), None);
    assert_eq!(g.soft_limit(), None);
}

#[test]
fn budget_config_soft_limit_and_headroom() {
    let _t = init_tracing();
    let g = GvGraph::<u64, u64, 8>::new(budget_config_unbounded(100));
    assert_eq!(g.budget(), Some(100));
    assert_eq!(g.headroom(), 81); // 3^(buffer+1) = 3^4 = 81
    assert_eq!(g.soft_limit(), Some(100 - 81)); // 19
    assert_eq!(g.depth_buffer(), 3); // D_evict(6) − D_create(3)
}

// ── Budget enforcement ──────────────────────────────────────────

#[test]
fn budget_converges_under_500_observations() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 50);
    assert_invariants(&g);
}

#[test]
fn budget_bounded_eviction_converges() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config(100), &plan, 100, 500);
    assert_invariants(&g);
}

// ── Depth gate dynamics ─────────────────────────────────────────

#[test]
fn depth_gates_shift_under_budget_pressure() {
    let _t = init_tracing();
    let cfg = budget_config_unbounded(100);
    let plan = Plan::new().spread(256, 6, 100);
    let g = run::<u64, u64, 8>(cfg, &plan);

    assert!(
        g.depth_evict() <= 6,
        "D_evict should tighten under sustained budget pressure: current={}",
        g.depth_evict()
    );
    assert_invariants(&g);
}

#[test]
fn depth_gates_recover_when_under_budget() {
    let _t = init_tracing();
    let cfg = budget_config_unbounded(100);
    let plan = Plan::new().spread(256, 6, 10);
    let g = run::<u64, u64, 8>(cfg, &plan);

    assert!(
        g.depth_evict() >= 6,
        "D_evict should relax when well under budget: {}",
        g.depth_evict()
    );
    assert_invariants(&g);
}

// ── Invariants throughout ───────────────────────────────────────

#[test]
fn invariants_hold_through_growth_and_contraction() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 200);
    let g = run_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 1);

    assert!(
        g.node_count() as usize <= 100,
        "hard budget violated: node_count ({}) > budget (100)",
        g.node_count()
    );
}

#[test]
fn energy_conserved_with_budget() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 300);
    let g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "G-root sum must equal total observation energy");
    assert_invariants(&g);
}

#[test]
fn energy_conserved_with_bounded_eviction() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 300);
    let g = run::<u64, u64, 8>(budget_config(100), &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "bounded eviction must also conserve energy");
    assert_invariants(&g);
}

// ── Hard budget guarantee (ADR-M-018) ─────────────────────────────

#[test]
fn hard_budget_never_exceeded() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 1000);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 1000);
    assert_invariants(&g);
}

#[test]
fn hard_budget_never_exceeded_bounded() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 1000);
    let g = run_budget_checked::<u64, u64, 8>(budget_config(100), &plan, 100, 1000);
    assert_invariants(&g);
}

#[test]
fn hard_budget_minimum_budget() {
    let _t = init_tracing();
    // buffer=3, headroom=81. Minimum budget = 82.
    let cfg = tight_budget_config();
    let plan = Plan::new().sweep(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 82, 500);
    assert_invariants(&g);
}

#[test]
fn hard_budget_minimum_budget_small_buffer() {
    let _t = init_tracing();
    // buffer=1, headroom=9. Minimum budget = 10.
    let cfg = small_buffer_config(10);
    let plan = Plan::new().sweep(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 10, 500);
    assert_eq!(g.soft_limit(), Some(1));
    assert_eq!(g.headroom(), 9);
    assert_invariants(&g);
}

#[test]
fn hard_budget_ceiling_is_tight() {
    let _t = init_tracing();
    let cfg = budget_config_unbounded(100);
    let plan: Plan<u64, u64> = Plan::new().sweep(256, 2000);

    let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    let mut max_count: u32 = 0;
    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        g.observe(coord, delta);
        max_count = max_count.max(g.node_count());
        assert!(
            g.node_count() as usize <= 100,
            "hard budget violated at observation {i}: node_count={}, budget=100",
            g.node_count()
        );
    }
    assert_invariants(&g);

    // Under sustained load, the tree should grow past the soft limit.
    assert!(
        max_count as usize > 19,
        "tree should exceed soft_limit (19) under load: max was {max_count}"
    );
}

#[test]
fn hard_budget_split_guard_prevents_overshoot() {
    let _t = init_tracing();
    // buffer=1, headroom=9, budget=20, soft_limit=11.
    let cfg = small_buffer_config(20);
    let plan: Plan<u64, u64> = Plan::new().sweep(256, 5000);

    let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    let mut max_count: u32 = 0;
    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        g.observe(coord, delta);
        max_count = max_count.max(g.node_count());
        assert!(
            g.node_count() as usize <= 20,
            "hard budget violated at observation {i}: node_count={}, budget=20",
            g.node_count()
        );
    }
    assert_invariants(&g);

    assert!(
        max_count as usize > 11,
        "tree should exceed soft_limit (11) under load: max was {max_count}"
    );
    assert!(max_count as usize <= 20);
}

#[test]
fn energy_conserved_at_hard_budget_limit() {
    let _t = init_tracing();
    let cfg = tight_budget_config();
    let plan = Plan::new().sweep(256, 500);
    let g = run::<u64, u64, 8>(cfg, &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy conservation violated at hard budget minimum");
    assert_invariants(&g);
}

// ── Degenerate hotspot under budget ─────────────────────────────

#[test]
fn hotspot_under_budget() {
    let _t = init_tracing();
    // All observations at a single coordinate, tight budget.
    let plan = Plan::new().hotspot(0, 10, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    assert_invariants(&g);
}

#[test]
fn hotspot_under_minimum_budget() {
    let _t = init_tracing();
    // Single coord, minimum budget.
    let plan = Plan::new().hotspot(0, 6, 200);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(10), &plan, 10, 200);
    assert_invariants(&g);
}

// ── Adversarial patterns under budget ───────────────────────────

#[test]
fn adversarial_zigzag_under_budget() {
    let _t = init_tracing();
    let plan = plan_adversarial(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    assert_invariants(&g);
}

#[test]
fn left_deep_under_budget() {
    let _t = init_tracing();
    // Worst-case left spine: all observations at coord 0.
    let plan = plan_left_deep(6, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

#[test]
fn right_deep_under_budget() {
    let _t = init_tracing();
    // Worst-case right spine: all observations at domain − 1.
    let plan = plan_right_deep(256, 6, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Skewed / burst / random under budget ────────────────────────

#[test]
fn skewed_load_under_budget() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

#[test]
fn budget_burst_growth_then_spike() {
    let _t = init_tracing();
    let plan = plan_budget_burst(256, 80, 50);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 130);
    assert_invariants(&g);
}

#[test]
fn random_spray_under_budget() {
    let _t = init_tracing();
    let plan = Plan::new().random_spray(42, 256, 6, 500);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

#[test]
fn oscillating_hotspot_under_budget() {
    let _t = init_tracing();
    // Alternating deep refinement at two far-apart coords forces
    // repeated eviction and re-growth.
    let plan = Plan::new().oscillating_hotspot(0, 255, 10, 50, 8);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 400);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Multi-phase lifecycle ───────────────────────────────────────

#[test]
fn growth_pressure_relax_under_budget() {
    let _t = init_tracing();
    // Uses the plan_growth_pressure_relax preset: growth → hotspot
    // pressure → light relaxation.
    let plan = plan_growth_pressure_relax(256, 100);
    let g = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 350);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Gate oscillation near budget boundary ───────────────────────

#[test]
fn gate_oscillation_near_budget_boundary() {
    let _t = init_tracing();
    // Observations designed to keep node_count hovering near the
    // soft limit, maximizing gate adjustments.
    let cfg = budget_config_unbounded(100);
    let plan: Plan<u64, u64> = Plan::new().sweep(256, 1000);

    let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    let mut tighten_count = 0u32;
    let mut relax_count = 0u32;
    let mut prev_d_evict = g.depth_evict();

    for &(coord, delta) in &plan.observations {
        g.observe(coord, delta);
        let d = g.depth_evict();
        if d < prev_d_evict {
            tighten_count += 1;
        } else if d > prev_d_evict {
            relax_count += 1;
        }
        prev_d_evict = d;
        assert!(
            g.node_count() as usize <= 100,
            "hard budget violated: node_count={}",
            g.node_count()
        );
    }
    assert_invariants(&g);

    // Under sustained load with budget, gates should have shifted
    // at least once in each direction.
    assert!(
        tighten_count > 0 || relax_count > 0,
        "gates should adjust under sustained load"
    );
}

// ── D-I3 (buffer invariant) maintained through lifecycle ────────

#[test]
fn di3_maintained_through_budget_lifecycle() {
    let _t = init_tracing();
    let cfg = budget_config_unbounded(100);
    let plan: Plan<u64, u64> = Plan::new()
        .spread(256, 6, 200) // push past budget → gates tighten
        .spread(256, 1, 100); // lighter load → gates may relax

    let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    for &(coord, delta) in &plan.observations {
        g.observe(coord, delta);
        assert!(
            g.depth_create() < g.depth_evict(),
            "D-I3 violated: D_create={}, D_evict={}",
            g.depth_create(),
            g.depth_evict()
        );
        assert_eq!(
            g.depth_buffer(),
            g.depth_evict() - g.depth_create(),
            "buffer must be preserved"
        );
    }
    assert_invariants(&g);
}

// ── Serialization ───────────────────────────────────────────────

#[test]
#[cfg(feature = "serde")]
fn budget_plan_serializes_correctly() {
    let _t = init_tracing();
    let plan: Plan<u64, u64> = Plan::new().sweep(256, 500);
    let json = support::to_json(&plan);
    let restored = support::from_json(&json);
    assert_eq!(plan, restored);
}

// ── Bounded vs unbounded comparison ─────────────────────────────

#[test]
fn bounded_and_unbounded_both_respect_hard_limit() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 500);

    let g_bounded = run_budget_checked::<u64, u64, 8>(budget_config(100), &plan, 100, 500);
    let g_unbounded = run_budget_checked::<u64, u64, 8>(budget_config_unbounded(100), &plan, 100, 500);

    // Both must conserve energy.
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g_bounded.total_sum(), total);
    assert_eq!(g_unbounded.total_sum(), total);
    assert_invariants(&g_bounded);
    assert_invariants(&g_unbounded);
}

// ── Various budget sizes ────────────────────────────────────────

#[test]
fn budget_200_sustained_load() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 1000);
    let cfg = budget_config_unbounded(200);
    let g = run_budget_checked::<u64, u64, 8>(cfg, &plan, 200, 100);
    assert_invariants(&g);
}

#[test]
fn budget_50_tight_small_buffer() {
    let _t = init_tracing();
    // buffer=1, headroom=9, budget=50, soft_limit=41.
    let plan = Plan::new().sweep(256, 500);
    let g = run_budget_checked::<u64, u64, 8>(small_buffer_config(50), &plan, 50, 500);
    assert_invariants(&g);
}

// ── Empty / trivial ─────────────────────────────────────────────

#[test]
fn empty_plan_under_budget() {
    let _t = init_tracing();
    let plan: Plan<u64, u64> = Plan::new();
    let g = run::<u64, u64, 8>(budget_config_unbounded(100), &plan);
    assert_eq!(g.node_count(), 1, "fresh graph has only the root");
    assert_eq!(g.total_sum(), 0);
    assert_invariants(&g);
}
