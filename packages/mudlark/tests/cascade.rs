// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Multi-violation cascade tests.
//!
//! Verify that the rebalance loop correctly handles multiple
//! independent and side-effect violations in a single `observe()`
//! call. Exercises ADR-M-003 (violation tracking).
//!
//! **Expanded coverage:** adversarial zigzag, power-law skew,
//! plan composition, and serialization round-trip for degenerate
//! cascade-inducing plans.

use torrust_mudlark::Config;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, cascade_config, deep_config, low_threshold_config, plan_adversarial, run, run_checked};

mod support;
use support::init_tracing;

// ── 4.1: Two independent violations from rapid accumulation ─────

#[test]
fn two_independent_violations_resolved() {
    let _t = init_tracing();
    let plan = Plan::new().observe(3, 10).observe(3, 10).observe(10, 10).observe(3, 30);
    let g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    assert_invariants(&g);
}

// ── 4.2: Side-effect violations from contraction ────────────────

#[test]
fn contraction_side_effects_resolved() {
    let _t = init_tracing();
    let plan = Plan::new().observe(1, 5).observe(1, 5).observe(9, 5).observe(1, 100);
    let g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    assert_invariants(&g);
}

// ── 4.3: Rapid-fire observations preserve invariants ────────────

#[test]
fn rapid_fire_observations() {
    let _t = init_tracing();
    let coords = [0u64, 7, 3, 15, 1, 10, 5, 12, 2, 8, 6, 14, 4, 11, 9, 13];
    let mut plan = Plan::new();
    for (i, &coord) in coords.iter().enumerate() {
        plan = plan.observe(coord, (i as u64 + 1) * 3);
    }
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── 4.4: Invariants hold after every observation ────────────────

#[test]
fn invariants_hold_after_every_observation() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(16, 50);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── 5.1: Ancestor violation from propagation ────────────────────

#[test]
fn ancestor_violation_caught_by_propagation_walk() {
    let _t = init_tracing();
    let plan = Plan::new()
        .observe(1, 3)
        .observe(9, 3)
        .observe(1, 3)
        .observe(1, 50)
        .observe(9, 20)
        .observe(5, 10);
    let g = run::<u64, u64, 4>(low_threshold_config(), &plan);
    assert_invariants(&g);
}

// ── 5.2: Promoted children in new uncle context ─────────────────

#[test]
fn promoted_children_checked_after_restructure() {
    let _t = init_tracing();
    let plan = Plan::new()
        .observe(1, 5)
        .observe(9, 5)
        .observe(5, 5)
        .observe(1, 80)
        .observe(1, 40);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_invariants(&g);
}

// ── 5.3: Escalation triggered by concentrated activity ──────────

#[test]
fn escalation_triggered_by_deep_concentration() {
    let _t = init_tracing();
    // Phase 1: build a moderately deep tree.
    let build = Plan::new().spread(16, 4, 8);
    // Phase 2: massive concentration.
    let spike = Plan::new().observe(1, 200);
    // Phase 3: verify tree remains operable.
    let recovery = Plan::new().spread(16, 5, 16);

    let plan = build.then(spike).then(recovery);
    let g = run::<u64, u64, 4>(deep_config(), &plan);
    assert_invariants(&g);
}

// ── 5.4: Split preprocessing with promoted + merged violations ──

#[test]
fn split_preprocessing_violations_caught() {
    let _t = init_tracing();
    let plan = Plan::new()
        .observe(1, 5)
        .observe(9, 5)
        .observe(1, 10)
        .observe(5, 10)
        .observe(13, 10)
        .observe(3, 15)
        .observe(7, 15);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_invariants(&g);
}

// ── 5.5: Stress test — 200 skewed observations ──────────────────

#[test]
fn stress_skewed_200_observations() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(16, 200);
    let g = run_checked::<u64, u64, 4>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: adversarial zigzag cascade ────────────────────────

#[test]
fn adversarial_zigzag_cascade() {
    let _t = init_tracing();
    let plan = plan_adversarial(16, 100);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: left-deep chain cascade ───────────────────────────

#[test]
fn left_deep_chain_cascade() {
    let _t = init_tracing();
    // All observations at coord 0 — maximally left-biased.
    let plan = Plan::new().hotspot(0, 10, 100);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: right-deep chain cascade ──────────────────────────

#[test]
fn right_deep_chain_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(15, 10, 100);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: burst then cascade ────────────────────────────────

#[test]
fn burst_then_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 3, 30).hotspot(7, 100, 20);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: energy conservation through cascade ───────────────

#[test]
fn energy_conserved_through_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(16, 100);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy conservation violated through cascade");
    assert_invariants(&g);
}

// ── Expanded: multi-phase plan composition ──────────────────────

#[test]
fn composed_plan_preserves_invariants() {
    let _t = init_tracing();
    let phase1 = Plan::new().spread(16, 4, 20);
    let phase2 = Plan::new().hotspot(0, 50, 10);
    let phase3 = Plan::new().zigzag(0, 15, 8, 30);
    let plan = phase1.then(phase2).then(phase3);
    let g = run_checked::<u64, u64, 4>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// ── Expanded: serialization round-trip of cascade plan ──────────

#[test]
#[cfg(feature = "serde")]
fn cascade_plan_serializes_correctly() {
    let _t = init_tracing();
    let plan: Plan<u64, u64> = Plan::new().spread(16, 4, 8).observe(1, 200).spread(16, 5, 16);
    let json = support::to_json(&plan);
    let restored = support::from_json(&json);
    assert_eq!(plan, restored);
    assert_eq!(plan.len(), restored.len());
}

// ── Expanded: 500 rapid-fire observations on N=8 ────────────────

#[test]
fn stress_rapid_fire_500_n8() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 500);
    let g = run_checked::<u64, u64, 8>(
        Config {
            split_threshold: 3,
            depth_create: 5,
            depth_evict: 10,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        },
        &plan,
        10,
    );
    assert_invariants(&g);
}

// ── Expanded: alternating extremes at domain boundary ───────────

#[test]
fn alternating_extremes_at_boundary() {
    let _t = init_tracing();
    // coord 0 and coord 255 on N=8 domain.
    let plan = Plan::new().zigzag(0, 255, 10, 100);
    let g = run_checked::<u64, u64, 8>(
        Config {
            split_threshold: 5,
            depth_create: 4,
            depth_evict: 8,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        },
        &plan,
        1,
    );
    assert_invariants(&g);
}
