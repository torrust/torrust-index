// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Correctness tests for **multi-violation cascade** resolution.
//!
//! A single `observe()` call can trigger a rebalance that itself
//! produces secondary violations — contractions may shift energy
//! across siblings, splits may create new uncle-shield breaches, and
//! promotions may expose previously hidden imbalances.  The rebalance
//! loop (ADR-M-003) must drain every such violation before returning,
//! regardless of how they chain.
//!
//! These tests exercise that loop across a spectrum of patterns:
//! independent concurrent violations, side-effect chains from
//! contraction and restructuring, adversarial access patterns
//! (zigzag, deep chains, alternating extremes), multi-phase
//! workloads, and scale stress — all verifying that invariants hold
//! and energy is conserved after every mutation.
//!
//! # Test index
//!
//! ## Independent & side-effect violations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`two_independent_violations_resolved`] | two rapid accumulations resolved in one pass |
//! | [`contraction_side_effects_resolved`] | contraction-induced side effects cleaned up |
//! | [`rapid_fire_observations`] | 16 interleaved observations preserve invariants |
//! | [`invariants_hold_after_every_observation`] | checked after every step in a monotone sweep |
//!
//! ## Propagation & restructuring
//!
//! | Test | Focus |
//! |------|-------|
//! | [`ancestor_violation_caught_by_propagation_walk`] | ancestor violation resolved via upward walk |
//! | [`promoted_children_checked_after_restructure`] | promoted children re-checked in new uncle context |
//! | [`escalation_triggered_by_deep_concentration`] | build → spike → recovery keeps invariants |
//! | [`split_preprocessing_violations_caught`] | merged violations from split preprocessing |
//! | [`stress_skewed_200_observations`] | 200 power-law–skewed observations |
//!
//! ## Adversarial & degenerate patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adversarial_zigzag_cascade`] | escalating zigzag between domain extremes |
//! | [`left_deep_chain_cascade`] | all observations at coord 0 (left spine) |
//! | [`right_deep_chain_cascade`] | all observations at max coord (right spine) |
//! | [`alternating_extremes_at_boundary`] | coords 0 and 255 alternating on N=8 |
//!
//! ## Burst & multi-phase
//!
//! | Test | Focus |
//! |------|-------|
//! | [`burst_then_cascade`] | uniform spread followed by concentrated hotspot |
//! | [`composed_plan_preserves_invariants`] | three-phase composed plan stays valid |
//! | [`oscillating_hotspot_cascade`] | oscillating between two hotspots |
//! | [`random_spray_cascade`] | pseudo-random observations across domain |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`energy_conserved_through_cascade`] | `total_sum()` equals sum of all deltas |
//! | [`energy_conserved_after_every_step`] | `total_sum()` correct at each observation |
//!
//! ## Stress / scale
//!
//! | Test | Focus |
//! |------|-------|
//! | [`stress_rapid_fire_500_n8`] | 500 sweep observations on N=8 domain |
//!
//! ## Serialization (`feature = "serde"`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`cascade_plan_serializes_correctly`] | round-trip JSON of a cascade-inducing plan |
//!
//! ## Edge cases
//!
//! | Test | Focus |
//! |------|-------|
//! | [`empty_plan_produces_valid_graph`] | zero observations → valid root-only graph |
//! | [`single_observation_no_cascade`] | one observation below threshold → no split |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, cascade_config, deep_config, low_threshold_config, plan_adversarial, run, run_checked};

mod support;
use support::init_tracing;

// =====================================================================
// Independent & side-effect violations (§4)
// =====================================================================

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

// =====================================================================
// Propagation & restructuring (§5)
// =====================================================================

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

// =====================================================================
// Adversarial & degenerate patterns
// =====================================================================

#[test]
fn adversarial_zigzag_cascade() {
    let _t = init_tracing();
    let plan = plan_adversarial(16, 100);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn left_deep_chain_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(0, 10, 100);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn right_deep_chain_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().hotspot(15, 10, 100);
    let g = run_checked::<u64, u64, 4>(low_threshold_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn alternating_extremes_at_boundary() {
    let _t = init_tracing();
    // coord 0 and coord 255 on N=8 domain.
    let plan = Plan::new().zigzag(0, 255, 10, 100);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Burst & multi-phase
// =====================================================================

#[test]
fn burst_then_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 3, 30).hotspot(7, 100, 20);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

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

#[test]
fn oscillating_hotspot_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().oscillating_hotspot(0, 15, 10, 8, 12);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

#[test]
fn random_spray_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().random_spray(42, 16, 8, 200);
    let g = run_checked::<u64, u64, 4>(cascade_config(), &plan, 1);
    assert_invariants(&g);
}

// =====================================================================
// Energy conservation
// =====================================================================

#[test]
fn energy_conserved_through_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().skewed(16, 100);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g.total_sum(), total, "energy conservation violated through cascade");
    assert_invariants(&g);
}

#[test]
fn energy_conserved_after_every_step() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 4, 20).hotspot(0, 50, 10).zigzag(0, 15, 8, 30);
    let mut g = torrust_mudlark::GvGraph::<u64, u64, 4>::new(cascade_config());
    let mut cumulative = 0u64;
    for &(coord, delta) in &plan.observations {
        cumulative += delta;
        g.observe(coord, delta);
        assert_eq!(
            g.total_sum(),
            cumulative,
            "energy conservation violated at cumulative={cumulative}"
        );
    }
    assert_invariants(&g);
}

// =====================================================================
// Stress / scale
// =====================================================================

#[test]
fn stress_rapid_fire_500_n8() {
    let _t = init_tracing();
    let plan = Plan::new().sweep(256, 500);
    let g = run_checked::<u64, u64, 8>(deep_config(), &plan, 10);
    assert_invariants(&g);
}

// =====================================================================
// Serialization
// =====================================================================

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

// =====================================================================
// Edge cases
// =====================================================================

#[test]
fn empty_plan_produces_valid_graph() {
    let _t = init_tracing();
    let plan = Plan::<u64, u64>::new();
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_eq!(g.total_sum(), 0);
    assert_invariants(&g);
}

#[test]
fn single_observation_no_cascade() {
    let _t = init_tracing();
    let plan = Plan::new().observe(7, 1);
    let g = run::<u64, u64, 4>(cascade_config(), &plan);
    assert_eq!(g.total_sum(), 1);
    assert_invariants(&g);
}
