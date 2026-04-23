// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Family 4 — Lifecycle benchmarks: decay and eviction cost.
//!
//! Validates O(G) decay and eviction scan claims from ADR-M-035.
//!
//! All benchmarks use `iter_batched` because `decay()` and
//! `check_evictions()` are `&mut self` — each iteration must start
//! from a fresh graph.

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_mudlark::testing::{Plan, budget_config, default_config, f64_default_config, run};

const SIZES: [usize; 3] = [100, 1_000, 10_000];

// ── bench_lifecycle_decay ───────────────────────────────────────
//
// Measure decay cost across graph sizes with default (unbounded)
// config.  q = 0.5 exercises the selective path.

fn bench_lifecycle_decay(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/decay");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().spread(256, 6, size);
                    run::<u64, u64, 8>(default_config(), &plan)
                },
                |mut g| {
                    let root = g.g_root();
                    g.decay(root, 0.95, 0.5);
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── bench_lifecycle_eviction ────────────────────────────────────
//
// Measure eviction scan cost on a budgeted graph.

fn bench_lifecycle_eviction(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/eviction");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().spread(256, 6, size);
                    run::<u64, u64, 8>(budget_config(500), &plan)
                },
                |mut g| {
                    g.check_evictions();
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── bench_lifecycle_decay_evict ─────────────────────────────────
//
// Combined decay + eviction pass — typical real-world maintenance
// pattern on a budgeted graph.

fn bench_lifecycle_decay_evict(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/decay_evict");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().spread(256, 6, size);
                    run::<u64, u64, 8>(budget_config(500), &plan)
                },
                |mut g| {
                    let root = g.g_root();
                    g.decay(root, 0.95, 0.5);
                    g.check_evictions();
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── bench_lifecycle_decay_selective ─────────────────────────────
//
// Selectivity sweep: vary q (quantile threshold) while holding
// graph size constant at 5 000 observations.
//
// q = 0.0 → nothing decays (cheapest).
// q = 1.0 → everything decays (costliest).
// Confirms monotone-increasing cost with q.

fn bench_lifecycle_decay_selective(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/decay_selective");
    group.sample_size(200);

    for q in [0.0, 0.25, 0.5, 0.75, 1.0] {
        group.bench_with_input(BenchmarkId::new("q", format!("{q}")), &q, |b, &q| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().spread(256, 6, 5_000);
                    run::<u64, u64, 8>(default_config(), &plan)
                },
                |mut g| {
                    let root = g.g_root();
                    g.decay(root, 0.95, q);
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── lifecycle/decay_f64 ─────────────────────────────────────────
//
// Phase 8 representative: f64 secondary configuration.
// Validates no unexpected performance cliffs in the floating-point
// decay path.

fn bench_lifecycle_decay_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/decay_f64");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<f64, f64>::new().spread(256, 6.0, size);
                    run::<f64, f64, 52>(f64_default_config(), &plan)
                },
                |mut g| {
                    let root = g.g_root();
                    g.decay(root, 0.95, 0.5);
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── lifecycle/decay_skewed ─────────────────────────────────────
//
// Decay over an asymmetric tree (power-law distribution).
// Validates that selective decay handles unbalanced V-Trees
// without pathological slowdown.

fn bench_lifecycle_decay_skewed(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/decay_skewed");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().skewed(256, size);
                    run::<u64, u64, 8>(default_config(), &plan)
                },
                |mut g| {
                    let root = g.g_root();
                    g.decay(root, 0.95, 0.5);
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── lifecycle/eviction_skewed ──────────────────────────────────
//
// Eviction scan on an asymmetric budgeted tree.
// The uneven weight distribution may change which nodes are
// eligible for eviction.

fn bench_lifecycle_eviction_skewed(c: &mut Criterion) {
    let mut group = c.benchmark_group("lifecycle/eviction_skewed");

    for &size in &SIZES {
        group.throughput(Throughput::Elements(size as u64));
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            b.iter_batched(
                || {
                    let plan = Plan::<u64, u64>::new().skewed(256, size);
                    run::<u64, u64, 8>(budget_config(500), &plan)
                },
                |mut g| {
                    g.check_evictions();
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_lifecycle_decay,
    bench_lifecycle_eviction,
    bench_lifecycle_decay_evict,
    bench_lifecycle_decay_selective,
    bench_lifecycle_decay_f64,
    bench_lifecycle_decay_skewed,
    bench_lifecycle_eviction_skewed,
}
criterion_main!(benches);
