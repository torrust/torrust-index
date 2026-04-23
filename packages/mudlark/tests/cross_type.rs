// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Cross-type instantiation tests.
//!
//! Verify that the generic machinery works for non-`u64` type
//! combinations.  Exercises ADR-M-006 (generic parameters), ADR-M-010
//! (observation generics), ADR-M-011 (overflow/narrowing), and
//! ADR-M-012 (g-sum recomputation for cross-type deltas).
//!
//! `Plan<C, V>` is fully generic — the convenience builders
//! (`spread`, `sweep`, `zigzag`, …) work for any `C: Coordinate`,
//! `V: Accumulator + Inspectable`.  Tests in this file use `Plan`
//! and `run` / `run_checked` wherever possible; manual `observe()`
//! calls are reserved for cross-type *observation* tests (`O ≠ V`).
//!
//! # Test index
//!
//! ## Same-type instantiations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u32_u32_single_observation`] | u32/u32 single observe + split |
//! | [`u32_u32_spread`] | u32/u32 spread via Plan |
//! | [`u32_u32_many_splits`] | u32/u32 low threshold, many splits |
//! | [`u8_u64_single_observation`] | u8 coord, u64 accumulator |
//! | [`u8_u64_sweep`] | u8/u64 sweep via Plan |
//! | [`u128_f64_single_observation`] | u128 coord, f64 accumulator |
//! | [`u128_f64_energy_conservation`] | f64 energy over many observations |
//! | [`f64_f64_single_observation`] | f64/f64 single observe + split |
//! | [`f32_f32_single_observation`] | f32/f32 single observe + split |
//! | [`f32_f32_cascading_splits`] | f32/f32 multi-split cascade |
//! | [`f32_f32_range_sum`] | f32 `range_sum` (Proratable) |
//! | [`f32_f32_sampling`] | f32 sampling (Weighable) |
//! | [`f32_f32_point_query`] | f32 point query |
//! | [`f32_f32_decay`] | f32 decay (Attenuatable) |
//! | [`f32_f32_extract_and_reconstruct`] | f32 extract + reconstruct |
//! | [`f32_f32_energy_conservation`] | f32 energy over many observations |
//!
//! ## Cross-type observations (O ≠ V)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`f64_observation_on_u16_accumulator`] | f64→u16 truncation |
//! | [`f32_observation_on_u32_accumulator`] | f32→u32 truncation |
//! | [`f64_observation_on_u64_accumulator`] | f64→u64 truncation |
//! | [`mixed_same_and_cross_type_observations`] | interleaved same/cross |
//! | [`truncation_accumulation_many_observations`] | accumulated truncation |
//! | [`from_observations_cross_type`] | `from_observations` with f64→u64 |
//! | [`extend_cross_type`] | `Extend` with f64→u64 pairs |
//!
//! ## Determinism
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plan_deterministic_across_runs`] | same Plan → same graph (u64) |
//! | [`plan_deterministic_u32`] | same Plan → same graph (u32) |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, default_config, f64_default_config, run, run_checked};
use torrust_mudlark::{Config, GvGraph};

mod support;
use support::init_tracing;

// ── Helpers ─────────────────────────────────────────────────────

/// Default-shaped config for u32 graphs (same parameters as
/// `default_config()` but with `u32` split threshold).
const fn config_u32() -> Config<u32> {
    Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

/// Default-shaped config for u16 graphs.
const fn config_u16() -> Config<u16> {
    Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

/// Default-shaped config for f32 graphs.
const fn config_f32() -> Config<f32> {
    Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

// ═══════════════════════════════════════════════════════════════
// Same-type instantiations
// ═══════════════════════════════════════════════════════════════

// ── u32 coordinate, u32 accumulator ─────────────────────────────

#[test]
fn u32_u32_single_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(config_u32());
    g.observe(100u32, 10u32);
    assert_eq!(g.node_count(), 3); // 10 > θ=5 → bootstrap split
    assert_eq!(g.total_sum(), 10);
    assert_invariants(&g);
}

#[test]
fn u32_u32_spread() {
    let _t = init_tracing();
    let plan: Plan<u32, u32> = Plan::new().spread(256, 6, 50);
    let g = run_checked::<u32, u32, 16>(config_u32(), &plan, 10);
    assert_eq!(g.total_sum(), 50 * 6);
    assert_invariants(&g);
}

#[test]
fn u32_u32_many_splits() {
    let _t = init_tracing();
    let plan: Plan<u32, u32> = Plan::new().spread(65536, 5, 100);
    let cfg = Config {
        split_threshold: 2,
        depth_create: 4,
        depth_evict: 8,
        ..config_u32()
    };
    let g = run_checked::<u32, u32, 16>(cfg, &plan, 20);
    assert_invariants(&g);
}

// ── u8 coordinate, u64 accumulator ──────────────────────────────

#[test]
fn u8_u64_single_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<u8, u64, 8> = GvGraph::new(default_config());
    g.observe(100u8, 10u64);
    assert_eq!(g.node_count(), 3);
    assert_eq!(g.total_sum(), 10);
    assert_invariants(&g);
}

#[test]
fn u8_u64_sweep() {
    let _t = init_tracing();
    let plan: Plan<u8, u64> = Plan::new().sweep(256, 50);
    let cfg = Config {
        split_threshold: 3,
        depth_create: 4,
        depth_evict: 8,
        ..default_config()
    };
    let g = run_checked::<u8, u64, 8>(cfg, &plan, 10);
    // Verify energy conservation: sweep generates delta = (i%7)+1.
    let expected: u64 = (0..50u64).map(|i| (i % 7) + 1).sum();
    assert_eq!(g.total_sum(), expected);
    assert_invariants(&g);
}

// ── u128 coordinate, f64 accumulator ────────────────────────────

#[test]
fn u128_f64_single_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<u128, f64, 64> = GvGraph::new(f64_default_config());
    g.observe(42u128, 10.5f64);
    assert_eq!(g.node_count(), 3);
    assert!((g.total_sum() - 10.5).abs() < f64::EPSILON);
    assert_invariants(&g);
}

#[test]
fn u128_f64_energy_conservation() {
    let _t = init_tracing();
    let plan: Plan<u128, f64> = Plan::new().spread(1000, 3.5, 30);
    let g = run_checked::<u128, f64, 64>(f64_default_config(), &plan, 5);
    let expected = 30.0 * 3.5;
    assert!(
        (g.total_sum() - expected).abs() < 1e-9,
        "f64 energy: expected {expected}, got {}",
        g.total_sum()
    );
    assert_invariants(&g);
}

// ── f64 coordinate, f64 accumulator ─────────────────────────────

#[test]
fn f64_f64_single_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<f64, f64, 52> = GvGraph::new(f64_default_config());
    g.observe(3.0f64, 10.0f64);
    assert_eq!(g.node_count(), 3);
    assert_invariants(&g);
}

// ── f32 coordinate, f32 accumulator ─────────────────────────────

#[test]
fn f32_f32_single_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 24> = GvGraph::new(config_f32());
    g.observe(3.0f32, 10.0f32);
    assert_eq!(g.node_count(), 3);
    assert_invariants(&g);
}

#[test]
fn f32_f32_cascading_splits() {
    let _t = init_tracing();
    let plan: Plan<f32, f32> = Plan::new().spread(256, 10.0f32, 10);
    let g = run_checked::<f32, f32, 8>(config_f32(), &plan, 2);
    assert!(g.node_count() > 3, "should have split multiple times");
    assert_invariants(&g);
}

#[test]
fn f32_f32_range_sum() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 8> = GvGraph::new(config_f32());
    g.observe(10.0f32, 20.0f32);
    g.observe(200.0f32, 30.0f32);

    let total = g.range_sum(..);
    assert!((total - 50.0).abs() < 0.01, "full range_sum: expected 50.0, got {total}");

    let left = g.range_sum(0.0f32..128.0f32);
    assert!(left > 0.0, "left half should have nonzero sum");

    let right = g.range_sum(128.0f32..256.0f32);
    assert!(
        (left + right - total).abs() < 1.0,
        "sub-ranges should sum to total: {left} + {right} vs {total}",
    );
}

#[test]
fn f32_f32_sampling() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 8> = GvGraph::new(config_f32());
    g.observe(42.0f32, 20.0f32);
    g.observe(200.0f32, 10.0f32);

    let cell = g.sample(&mut support::FixedRng(0.5)).unwrap();
    assert!(cell.start >= 0.0f32);
    assert!(cell.end <= 256.0f32);
    assert!(cell.start < cell.end);
    assert!(cell.intensity >= 0.0f32);
}

#[test]
fn f32_f32_point_query() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 8> = GvGraph::new(config_f32());
    g.observe(42.0f32, 20.0f32);

    let cell = g.get(42.0f32);
    assert!(cell.start <= 42.0f32);
    assert!(cell.end > 42.0f32);
    assert!(cell.start < cell.end);
}

#[test]
fn f32_f32_decay() {
    let _t = init_tracing();
    let plan: Plan<f32, f32> = Plan::new().hotspot(42.0f32, 10.0f32, 10);
    let mut g = run::<f32, f32, 8>(config_f32(), &plan);
    let pre_decay = g.total_sum();
    assert!((pre_decay - 100.0).abs() < 0.01);

    g.decay(g.g_root(), 0.5, 0.0);
    let post_decay = g.total_sum();

    assert!(post_decay < pre_decay, "decay should reduce total");
    assert!(
        (post_decay - 50.0).abs() < 5.0,
        "50% decay should yield ~50.0, got {post_decay}",
    );
    assert_invariants(&g);
}

#[test]
fn f32_f32_extract_and_reconstruct() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 8> = GvGraph::new(config_f32());
    g.observe(10.0f32, 15.0f32);
    g.observe(200.0f32, 25.0f32);

    let total = g.total_sum();
    let pewei = g.extract();
    assert!(pewei.layer_count() >= 1);

    let spans = pewei.reconstruct(pewei.layer_count().saturating_sub(1));

    // Partition check: spans tile [0, 256).
    assert!((spans.first().unwrap().start - 0.0f32).abs() < f32::EPSILON);
    assert!((spans.last().unwrap().end - 256.0f32).abs() < f32::EPSILON);
    for w in spans.windows(2) {
        assert!(
            (w[0].end - w[1].start).abs() < f32::EPSILON,
            "gap: {} vs {}",
            w[0].end,
            w[1].start,
        );
    }

    // Energy conservation (f32 rounding — wider tolerance).
    let span_sum: f32 = spans.iter().map(|s| s.intensity).sum();
    assert!(
        (span_sum - total).abs() < 2.0,
        "f32 energy conservation: total={total}, span_sum={span_sum}",
    );
}

#[test]
fn f32_f32_energy_conservation() {
    let _t = init_tracing();
    let plan: Plan<f32, f32> = Plan::new().spread(256, 3.5f32, 20);
    let g = run_checked::<f32, f32, 8>(config_f32(), &plan, 5);
    let expected = 20.0f32 * 3.5;
    assert!(
        (g.total_sum() - expected).abs() < 0.1,
        "f32 energy: expected {expected}, got {}",
        g.total_sum()
    );
    assert_invariants(&g);
}

// ═══════════════════════════════════════════════════════════════
// Cross-type observations (O ≠ V)
// ═══════════════════════════════════════════════════════════════

#[test]
fn f64_observation_on_u16_accumulator() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u16, 32> = GvGraph::new(config_u16());
    // 0 + 10.7 → truncates to 10.
    g.observe(100u64, 10.7f64);
    assert_eq!(g.total_sum(), 10);
    assert_eq!(g.node_count(), 3); // 10 > θ=5 → split
    assert_invariants(&g);
}

#[test]
fn f32_observation_on_u32_accumulator() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(config_u32());
    g.observe(50u32, 8.9f32);
    assert_eq!(g.total_sum(), 8); // truncated
    assert_eq!(g.node_count(), 3); // 8 > θ=5 → split
    assert_invariants(&g);
}

#[test]
fn f64_observation_on_u64_accumulator() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    g.observe(42u64, 7.9f64);
    assert_eq!(g.total_sum(), 7); // truncated
    assert_eq!(g.node_count(), 3); // 7 > θ=5 → split
    assert_invariants(&g);
}

#[test]
fn mixed_same_and_cross_type_observations() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        depth_create: 4,
        depth_evict: 8,
        ..default_config()
    });
    // Same-type.
    g.observe(10u64, 6u64);
    g.observe(10u64, 4u64);
    // Cross-type (f64 → u64).
    g.observe(100u64, 7.5f64); // truncates to 7
    g.observe(200u64, 3.2f64); // truncates to 3
    assert_eq!(g.total_sum(), 6 + 4 + 7 + 3);
    assert_invariants(&g);
}

#[test]
fn truncation_accumulation_many_observations() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u16, 32> = GvGraph::new(config_u16());
    for i in 0..20u64 {
        g.observe(i % 256, 3.9f64); // each truncates to 3
    }
    assert_eq!(g.total_sum(), 60); // 20 × 3
    assert_invariants(&g);
}

#[test]
fn from_observations_cross_type() {
    let _t = init_tracing();
    // f64 deltas into a u64 accumulator via `from_observations`.
    let pairs: Vec<(u64, f64)> = vec![(10, 5.9), (20, 3.1), (30, 7.7)];
    let g = GvGraph::<u64, u64, 8>::from_observations(default_config(), pairs);
    // 5.9→5, 3.1→3, 7.7→7 = 15.
    assert_eq!(g.total_sum(), 15);
    assert_invariants(&g);
}

#[test]
fn extend_cross_type() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Same-type seed.
    g.observe(0u64, 10u64);
    // Cross-type extend.
    let pairs: Vec<(u64, f64)> = vec![(50, 4.8), (100, 6.2)];
    g.extend(pairs);
    // 10 + 4 + 6 = 20.
    assert_eq!(g.total_sum(), 20);
    assert_invariants(&g);
}

// ═══════════════════════════════════════════════════════════════
// Determinism
// ═══════════════════════════════════════════════════════════════

#[test]
fn plan_deterministic_across_runs() {
    let _t = init_tracing();
    let plan: Plan<u64, u64> = Plan::new().spread(256, 6, 50);
    let cfg = Config {
        depth_create: 4,
        depth_evict: 8,
        ..default_config()
    };
    let g1 = run::<u64, u64, 8>(cfg.clone(), &plan);
    let g2 = run::<u64, u64, 8>(cfg, &plan);
    assert_eq!(g1.node_count(), g2.node_count());
    assert_eq!(g1.total_sum(), g2.total_sum());
}

#[test]
fn plan_deterministic_u32() {
    let _t = init_tracing();
    let plan: Plan<u32, u32> = Plan::new().spread(256, 6, 50);
    let g1 = run::<u32, u32, 16>(config_u32(), &plan);
    let g2 = run::<u32, u32, 16>(config_u32(), &plan);
    assert_eq!(g1.node_count(), g2.node_count());
    assert_eq!(g1.total_sum(), g2.total_sum());
}
