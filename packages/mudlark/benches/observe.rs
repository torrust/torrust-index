// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Family 1 — Observe benchmarks
//!
//! Measures observation throughput across workload shapes and validates
//! the amortised O(depth) claim.
//!
//! Four benchmark groups:
//! - `observe/spread`           — uniform spread, default config
//! - `observe/spread_budgeted`  — uniform spread, budgeted config
//! - `observe/skewed`           — power-law skew, default config
//! - `observe/deep`             — monotone sweep, deep config

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_mudlark::GvGraph;
use torrust_mudlark::testing::{Plan, budget_config, deep_config, default_config, f64_default_config, run};

const SWEEP: [usize; 4] = [100, 1_000, 10_000, 100_000];

// ── observe/spread ──────────────────────────────────────────────

fn bench_observe_spread(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/spread");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().spread(256, 6, n),
                |plan| {
                    let mut g = GvGraph::<u64, u64, 8>::new(default_config());
                    for &(coord, delta) in &plan.observations {
                        g.observe(coord, delta);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/spread_budgeted ─────────────────────────────────────

fn bench_observe_spread_budgeted(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/spread_budgeted");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().spread(256, 6, n),
                |plan| {
                    let mut g = GvGraph::<u64, u64, 8>::new(budget_config(500));
                    for &(coord, delta) in &plan.observations {
                        g.observe(coord, delta);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/skewed ──────────────────────────────────────────────

fn bench_observe_skewed(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/skewed");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().skewed(256, n),
                |plan| {
                    let mut g = GvGraph::<u64, u64, 8>::new(default_config());
                    for &(coord, delta) in &plan.observations {
                        g.observe(coord, delta);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/deep ────────────────────────────────────────────────

fn bench_observe_deep(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/deep");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().sweep(256, n),
                |plan| {
                    let mut g = GvGraph::<u64, u64, 8>::new(deep_config());
                    for &(coord, delta) in &plan.observations {
                        g.observe(coord, delta);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/spread_f64 ──────────────────────────────────────────
//
// Phase 8 representative: validates that the floating-point
// coordinate/accumulator path has no unexpected performance cliffs.

fn bench_observe_spread_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/spread_f64");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<f64, f64>::new().spread(256, 6.0, n),
                |plan| {
                    let mut g = GvGraph::<f64, f64, 52>::new(f64_default_config());
                    for &(coord, delta) in &plan.observations {
                        g.observe(coord, delta);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/zigzag ──────────────────────────────────────────────
//
// Alternating extremes: exercises repeated deep refinement on
// opposite sides of the tree.

fn bench_observe_zigzag(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/zigzag");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().zigzag(0, 255, 6, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/diamond ─────────────────────────────────────────────
//
// Converge from domain extremes then diverge: exercises
// restructuring in contraction and expansion directions.

fn bench_observe_diamond(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/diamond");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().diamond(256, 6, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── observe/golden_spiral ───────────────────────────────────────
//
// Low-discrepancy quasi-uniform coverage: each observation lands
// as far as possible from all predecessors.

fn bench_observe_golden_spiral(c: &mut Criterion) {
    let mut group = c.benchmark_group("observe/golden_spiral");

    for &n in &SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().golden_spiral(256, 6, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── Harness ─────────────────────────────────────────────────────

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_observe_spread,
    bench_observe_spread_budgeted,
    bench_observe_skewed,
    bench_observe_deep,
    bench_observe_spread_f64,
    bench_observe_zigzag,
    bench_observe_diamond,
    bench_observe_golden_spiral,
}
criterion_main!(benches);
