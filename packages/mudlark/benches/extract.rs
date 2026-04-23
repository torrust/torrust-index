// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_mudlark::testing::{Plan, default_config, f64_default_config, run};

// ── Family 3: Extract benchmarks ────────────────────────────────
//
// Measures PEWEI extraction, streaming layer iteration, and plateau
// retrieval.  All three operations are `&self`, so we build the
// graph once and benchmark repeatedly.

// ── Size sweep constants ────────────────────────────────────────

const SIZES: &[usize] = &[100, 1_000, 10_000];

// ── extract/full — allocating PEWEI extraction ──────────────────

fn bench_extract_full(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/full");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<u64, u64>::new().spread(256, 6, size);
            let g = run::<u64, u64, 8>(default_config(), &plan);

            b.iter(|| g.extract());
        });
    }

    group.finish();
}

// ── extract/layers — streaming iterator overhead ────────────────

fn bench_extract_layers(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/layers");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<u64, u64>::new().spread(256, 6, size);
            let g = run::<u64, u64, 8>(default_config(), &plan);

            b.iter(|| g.layers().count());
        });
    }

    group.finish();
}

// ── extract/plateaus — O(1) borrow with dynamic-contour-tracking ─

fn bench_extract_plateaus(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/plateaus");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<u64, u64>::new().spread(256, 6, size);
            let g = run::<u64, u64, 8>(default_config(), &plan);

            b.iter(|| g.plateaus());
        });
    }

    group.finish();
}

// ── extract/full_f64 — f64 secondary configuration ─────────────
//
// Phase 8 representative: validates that the floating-point path
// has no unexpected performance cliffs relative to the u64 variant.

fn bench_extract_full_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/full_f64");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<f64, f64>::new().spread(256, 6.0, size);
            let g = run::<f64, f64, 52>(f64_default_config(), &plan);

            b.iter(|| g.extract());
        });
    }

    group.finish();
}

// ── extract/full_skewed — non-uniform tree shape ────────────────
//
// Power-law distribution concentrates weight in the lower quarter,
// producing an asymmetric tree.  Validates extract under unbalanced
// V-Trees.

fn bench_extract_full_skewed(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/full_skewed");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<u64, u64>::new().skewed(256, size);
            let g = run::<u64, u64, 8>(default_config(), &plan);

            b.iter(|| g.extract());
        });
    }

    group.finish();
}

// ── extract/full_hotspot — extreme concentration ────────────────
//
// Single-coordinate concentration produces a deep, narrow V-Tree.
// Validates extract at the degenerate extreme.

fn bench_extract_full_hotspot(c: &mut Criterion) {
    let mut group = c.benchmark_group("extract/full_hotspot");

    for &size in SIZES {
        group.throughput(Throughput::Elements(size as u64));

        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |b, &size| {
            let plan = Plan::<u64, u64>::new().hotspot(128, 6, size);
            let g = run::<u64, u64, 8>(default_config(), &plan);

            b.iter(|| g.extract());
        });
    }

    group.finish();
}

// ── Harness registration ────────────────────────────────────────

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_extract_full,
    bench_extract_layers,
    bench_extract_plateaus,
    bench_extract_full_f64,
    bench_extract_full_skewed,
    bench_extract_full_hotspot,
}
criterion_main!(benches);
