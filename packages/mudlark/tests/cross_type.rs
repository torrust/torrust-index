// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Cross-type instantiation tests.
//!
//! Verify that the generic machinery works for non-`u64` type
//! combinations. Exercises ADR-M-006 (generic parameters), ADR-M-010
//! (observation generics), and ADR-M-011 (overflow/narrowing).
//!
//! **Expanded coverage:** f32 coordinate, multi-observation mixed-type
//! sequences, energy conservation across types, and plan reuse across
//! different graph instantiations.
//!
//! Note: `Plan` is `(u64, u64)` only, so cross-type tests manually
//! call `observe()`. The plan infrastructure is tested for `u64` graphs
//! in other test files.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, run};
use torrust_mudlark::{Config, GvGraph};

mod support;
use support::init_tracing;

// ── Helpers ─────────────────────────────────────────────────────

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

const fn config_f64() -> Config<f64> {
    Config {
        split_threshold: 5.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

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

// ── 3.1: u32 coordinate, u32 accumulator ────────────────────────

#[test]
fn u32_u32_observe_and_split() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(config_u32());
    g.observe(100u32, 10u32);
    assert_eq!(g.node_count(), 3); // bootstrap split
    assert_eq!(g.total_sum(), 10);
    assert_invariants(&g);
}

// ── 3.2: u128 coordinate, f64 accumulator ───────────────────────

#[test]
fn u128_f64_observe_and_split() {
    let _t = init_tracing();
    let mut g: GvGraph<u128, f64, 64> = GvGraph::new(config_f64());
    g.observe(42u128, 10.5f64);
    assert_eq!(g.node_count(), 3);
    assert!((g.total_sum() - 10.5).abs() < f64::EPSILON);
    assert_invariants(&g);
}

// ── 3.3: f64 coordinate, f64 accumulator ────────────────────────

#[test]
fn f64_f64_observe_and_split() {
    let _t = init_tracing();
    let mut g: GvGraph<f64, f64, 52> = GvGraph::new(config_f64());
    g.observe(3.0f64, 10.0f64);
    assert_eq!(g.node_count(), 3);
    assert_invariants(&g);
}

// ── 3.4: f64 observation on u16 accumulator (cross-type truncation) ─

#[test]
fn f64_observation_on_u16_accumulator() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u16, 32> = GvGraph::new(config_u16());
    // 0 + 10.7 → truncates to 10
    g.observe(100u64, 10.7f64);
    assert_eq!(g.total_sum(), 10);
    assert_eq!(g.node_count(), 3); // 10 > 5 → split
    assert_invariants(&g);
}

// ── 3.5: f32 observation on u32 accumulator (cross-type narrowing) ──

#[test]
fn f32_observation_on_u32_accumulator() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(config_u32());
    g.observe(50u32, 8.9f32);
    assert_eq!(g.total_sum(), 8); // truncated
    assert_eq!(g.node_count(), 3); // 8 > 5 → split
    assert_invariants(&g);
}

// ── 3.6: Multiple observations across types ─────────────────────

#[test]
fn mixed_observations_preserve_invariants() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    // Same-type observations
    g.observe(10u64, 6u64);
    g.observe(10u64, 4u64);
    // Cross-type observations
    g.observe(100u64, 7.5f64);
    g.observe(200u64, 3.2f64);
    assert_invariants(&g);
}

// ── Expanded: f32 coordinate, f32 accumulator ───────────────────

#[test]
fn f32_f32_observe_and_split() {
    let _t = init_tracing();
    let mut g: GvGraph<f32, f32, 24> = GvGraph::new(Config {
        split_threshold: 5.0f32,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(3.0f32, 10.0f32);
    assert_eq!(g.node_count(), 3);
    assert_invariants(&g);
}

// ── Expanded: u8 coordinate, u64 accumulator ────────────────────

#[test]
fn u8_u64_observe_and_split() {
    let _t = init_tracing();
    let mut g: GvGraph<u8, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u8, 10u64);
    assert_eq!(g.node_count(), 3);
    assert_eq!(g.total_sum(), 10);
    assert_invariants(&g);
}

// ── Expanded: u8 coordinate, many observations ──────────────────

#[test]
fn u8_u64_many_observations() {
    let _t = init_tracing();
    let mut g: GvGraph<u8, u64, 8> = GvGraph::new(Config {
        split_threshold: 3,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    let mut total: u64 = 0;
    for i in 0..50u64 {
        let coord = u8::try_from(i % 256).unwrap();
        let delta = (i % 7) + 1;
        g.observe(coord, delta);
        total += delta;
    }
    assert_eq!(g.total_sum(), total);
    assert_invariants(&g);
}

// ── Expanded: u32 with many splits ──────────────────────────────

#[test]
fn u32_u32_many_splits() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(Config {
        split_threshold: 2,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    for i in 0..100u32 {
        g.observe(i % 65536, 5u32);
    }
    assert_invariants(&g);
}

// ── Expanded: cross-type energy conservation ────────────────────

#[test]
fn f64_energy_conservation() {
    let _t = init_tracing();
    let mut g: GvGraph<u128, f64, 64> = GvGraph::new(config_f64());
    let mut total: f64 = 0.0;
    for i in 0..30u64 {
        #[allow(clippy::cast_precision_loss)]
        let delta = (i as f64 + 1.0) * 0.5;
        g.observe(u128::from(i) * 1000, delta);
        total += delta;
    }
    assert!(
        (g.total_sum() - total).abs() < 1e-9,
        "f64 energy conservation: expected {total}, got {}",
        g.total_sum()
    );
    assert_invariants(&g);
}

// ── Expanded: truncation accumulation over many observations ────

#[test]
fn truncation_accumulation_over_many_observations() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u16, 32> = GvGraph::new(config_u16());
    // Each f64 observation truncates; verify accumulated values.
    for i in 0..20u64 {
        g.observe(i % 256, 3.9f64); // truncates to 3
    }
    // Total: 20 * 3 = 60.
    assert_eq!(g.total_sum(), 60);
    assert_invariants(&g);
}

// ── Expanded: verify Plan reuse produces deterministic u64 graphs ──

#[test]
fn plan_reuse_deterministic() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 6, 50);
    let cfg = Config {
        split_threshold: 5u64,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let g1 = run::<u64, u64, 8>(cfg.clone(), &plan);
    let g2 = run::<u64, u64, 8>(cfg, &plan);
    assert_eq!(g1.node_count(), g2.node_count());
    assert_eq!(g1.total_sum(), g2.total_sum());
}

// ── Expanded: f64 coord with cross-type observation (f64 delta) ─

#[test]
fn f64_coord_f64_observation() {
    let _t = init_tracing();
    let mut g: GvGraph<f64, f64, 52> = GvGraph::new(config_f64());
    g.observe(1.5f64, 8.5f64);
    assert!((g.total_sum() - 8.5).abs() < 1e-5);
    assert_eq!(g.node_count(), 3); // 8.5 > 5.0 → split
    assert_invariants(&g);
}
