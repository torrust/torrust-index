// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Family 2: Query benchmarks ──────────────────────────────────
//
// Measures point query, range sum, and sampling latency.
// Validates entropy-optimal O(1.44 H) sampling claim via
// cross-distribution comparison.

mod bench_rng;

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use torrust_mudlark::testing::{Plan, default_config};

// ── query/get ───────────────────────────────────────────────────

fn bench_query_get(c: &mut Criterion) {
    let mut group = c.benchmark_group("query/get");

    for size in [100, 1_000, 10_000] {
        let plan = Plan::<u64, u64>::new().spread(256, 6, size);
        let g = torrust_mudlark::testing::run::<u64, u64, 8>(default_config(), &plan);

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, _| {
            b.iter(|| g.get(128));
        });
    }

    group.finish();
}

// ── query/range_sum ─────────────────────────────────────────────

fn bench_query_range_sum(c: &mut Criterion) {
    let mut group = c.benchmark_group("query/range_sum");

    let plan = Plan::<u64, u64>::new().spread(256, 6, 10_000);
    let g = torrust_mudlark::testing::run::<u64, u64, 8>(default_config(), &plan);

    for width in [16u64, 64, 128, 256] {
        // Centre the range around the midpoint of the domain.
        let mid = 128u64;
        let a = mid.saturating_sub(width / 2);
        let b = a + width;

        group.bench_with_input(BenchmarkId::new("width", width), &width, |bench, _| {
            bench.iter(|| g.range_sum(a..b));
        });
    }

    group.finish();
}

// ── query/sample (uniform / skewed / hotspot) ───────────────────

fn bench_query_sample(c: &mut Criterion) {
    let mut group = c.benchmark_group("query/sample");
    group.sample_size(200);

    // ── Uniform distribution ────────────────────────────────────
    {
        let plan = Plan::<u64, u64>::new().spread(256, 6, 5_000);
        let g = torrust_mudlark::testing::run::<u64, u64, 8>(default_config(), &plan);

        group.bench_function("uniform", |b| {
            let mut rng = bench_rng::BenchRng(42);
            b.iter(|| g.sample(&mut rng));
        });
    }

    // ── Skewed distribution ─────────────────────────────────────
    {
        let plan = Plan::<u64, u64>::new().skewed(256, 5_000);
        let g = torrust_mudlark::testing::run::<u64, u64, 8>(default_config(), &plan);

        group.bench_function("skewed", |b| {
            let mut rng = bench_rng::BenchRng(42);
            b.iter(|| g.sample(&mut rng));
        });
    }

    // ── Hotspot distribution ────────────────────────────────────
    {
        let plan = Plan::<u64, u64>::new().hotspot(128, 10, 5_000);
        let g = torrust_mudlark::testing::run::<u64, u64, 8>(default_config(), &plan);

        group.bench_function("hotspot", |b| {
            let mut rng = bench_rng::BenchRng(42);
            b.iter(|| g.sample(&mut rng));
        });
    }

    group.finish();
}

// ── query/sample_f64 ────────────────────────────────────────────

fn bench_query_sample_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("query/sample_f64");
    group.sample_size(200);

    let plan = Plan::<f64, f64>::new().spread(256, 6.0, 5_000);
    let g = torrust_mudlark::testing::run::<f64, f64, 52>(torrust_mudlark::testing::f64_default_config(), &plan);

    group.bench_function("uniform", |b| {
        let mut rng = bench_rng::BenchRng(42);
        b.iter(|| g.sample(&mut rng));
    });

    group.finish();
}

// ── Harness ─────────────────────────────────────────────────────

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_query_get,
    bench_query_range_sum,
    bench_query_sample,
    bench_query_sample_f64,
}
criterion_main!(benches);
