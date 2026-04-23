// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Family 5: Spray benchmarks ──────────────────────────────────
//
// Measures realistic sustained throughput under random workloads
// modelling Torrust's actual access pattern.
//
// Depends on: `Plan::random_spray` (Phase 1), `BenchRng` (Phase 1).

mod bench_rng;

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_mudlark::GvGraph;
use torrust_mudlark::testing::{Plan, budget_config, default_config, f64_default_config, run};

// ── Sweep sizes ─────────────────────────────────────────────────

const SWEEP: &[usize] = &[100, 1_000, 10_000, 100_000];

// ── Cold start ──────────────────────────────────────────────────
//
// Fresh graph + `n` random observations.
// Growth-only: splits dominate, no eviction.

fn bench_spray_cold_start(c: &mut Criterion) {
    let mut group = c.benchmark_group("spray/cold_start");

    for &n in SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<u64, u64>::new().random_spray(42, 256, 6, n),
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

// ── Steady state ────────────────────────────────────────────────
//
// Warmup 10K observations → then `n` more under a budget.
// Exercises eviction and rebalance at steady state.

fn bench_spray_steady_state(c: &mut Criterion) {
    let mut group = c.benchmark_group("spray/steady_state");

    for &n in SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    // Warmup phase (not timed).
                    let warmup = Plan::<u64, u64>::new().random_spray(42, 256, 6, 10_000);
                    let g = run::<u64, u64, 8>(budget_config(500), &warmup);
                    let spray = Plan::<u64, u64>::new().random_spray(99, 256, 6, n);
                    (g, spray)
                },
                |(mut g, spray)| {
                    for &(coord, delta) in &spray.observations {
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

// ── Interleave ──────────────────────────────────────────────────
//
// Alternating observe() + sample() at steady state.
// Measures query latency under continuous mutation.

fn bench_spray_interleave(c: &mut Criterion) {
    let mut group = c.benchmark_group("spray/interleave");

    for &n in SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || {
                    // Warmup to steady state (not timed).
                    let warmup = Plan::<u64, u64>::new().random_spray(42, 256, 6, 10_000);
                    let g = run::<u64, u64, 8>(budget_config(500), &warmup);
                    let spray = Plan::<u64, u64>::new().random_spray(99, 256, 6, n);
                    (g, spray)
                },
                |(mut g, spray)| {
                    let mut rng = bench_rng::BenchRng(123);
                    for &(coord, delta) in &spray.observations {
                        g.observe(coord, delta);
                        let _ = g.sample(&mut rng);
                    }
                    g
                },
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

// ── Budget scaling ──────────────────────────────────────────────
//
// Fixed spray (5K), varying budget.
// Reveals budget impact on per-observation cost.

fn bench_spray_scaling_budget(c: &mut Criterion) {
    let mut group = c.benchmark_group("spray/scaling_budget");

    for budget in [200, 500, 2_000, 10_000] {
        group.throughput(Throughput::Elements(5_000));
        group.bench_with_input(BenchmarkId::new("budget", budget), &budget, |b, &budget| {
            b.iter_batched(
                || Plan::<u64, u64>::new().random_spray(42, 256, 6, 5_000),
                |plan| {
                    let mut g = GvGraph::<u64, u64, 8>::new(budget_config(budget));
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

// ── f64 secondary ───────────────────────────────────────────────
//
// Single f64 variant to flag unexpected performance cliffs.

fn bench_spray_cold_start_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("spray/cold_start_f64");

    for &n in SWEEP {
        group.throughput(Throughput::Elements(n as u64));
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<f64, f64>::new().random_spray(42, 256, 6.0, n),
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

// ── Registration ────────────────────────────────────────────────

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_spray_cold_start,
    bench_spray_steady_state,
    bench_spray_interleave,
    bench_spray_scaling_budget,
    bench_spray_cold_start_f64,
}
criterion_main!(benches);
