// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Criterion benchmarks for the Spectral Sentinel.
//!
//! Run with:
//!
//! ```sh
//! cargo bench -p torrust-sentinel
//! ```

use std::hint::black_box;

use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_sentinel::{CentredBits, Sentinel128, SentinelConfig};

// ─── Helpers ────────────────────────────────────────────────

/// Lightweight config for micro-benchmarks: small rank, small k.
fn bench_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 4,
        forgetting_factor: 0.95,
        rank_update_interval: 10,
        analysis_k: 16,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: false,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.999,
        cusum_coord_slow_decay: 0.999,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 100,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        noise_seed: Some(42),
        svd_strategy: torrust_sentinel::SvdStrategy::Brand,
        background_warming: false,
    }
}

/// Realistic config with higher rank and larger analysis set.
fn realistic_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 16,
        forgetting_factor: 0.99,
        rank_update_interval: 100,
        analysis_k: 1024,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: false,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.999,
        cusum_coord_slow_decay: 0.999,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 100,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![50]),
        noise_batch_size: 16,
        noise_seed: Some(42),
        svd_strategy: torrust_sentinel::SvdStrategy::Brand,
        background_warming: false,
    }
}

/// Generate `count` sequential values in a single narrow range.
fn single_range_values(count: usize) -> Vec<u128> {
    (0..count).map(|i| (0xAB_u128 << 120) | (i as u128 + 1)).collect()
}

/// Generate `count` values spread across many leading-nibble ranges.
fn multi_range_values(count: usize) -> Vec<u128> {
    (0..count)
        .map(|i| {
            let nibble = (i % 16) as u128;
            (nibble << 124) | (i as u128 + 1)
        })
        .collect()
}

/// Create a warmed sentinel ready for steady-state benchmarking.
fn warmed_sentinel(cfg: &SentinelConfig<u64>) -> Sentinel128 {
    let mut s = Sentinel128::new(cfg.clone()).unwrap();

    // Seed cells by ingesting diverse values.
    let seed: Vec<u128> = (0..4_u128)
        .flat_map(|nibble| (0..4_u128).map(move |i| (nibble << 124) | (i + 1)))
        .collect();
    s.ingest(&seed);

    // Noise is auto-injected at construction (§ALGO S-11).

    // One real batch to enter the warm code path.
    let batch: Vec<u128> = (0..4_u128)
        .flat_map(|nibble| (0..4_u128).map(move |i| (nibble << 124) | (i + 100)))
        .collect();
    s.ingest(&batch);

    s
}

// ─── Observation encoding ───────────────────────────────────

fn bench_centred_bits(c: &mut Criterion) {
    c.bench_function("CentredBits::from_u128", |b| {
        b.iter(|| CentredBits::from_u128(black_box(0xDEAD_BEEF_CAFE_BABE_1234_5678_9ABC_DEF0)));
    });
}

// ─── Ingest (core hot path) ─────────────────────────────────

fn bench_ingest_cold(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_cold");

    for batch_size in [1, 8, 32] {
        let values = single_range_values(batch_size);
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("single_range", batch_size), &values, |b, vals| {
            b.iter_with_setup(
                || Sentinel128::new(bench_config()).unwrap(),
                |mut s| s.ingest(black_box(vals)),
            );
        });
    }

    group.finish();
}

fn bench_ingest_warm(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_warm");
    let cfg = bench_config();

    for batch_size in [1, 8, 32, 64] {
        let values = single_range_values(batch_size);
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("single_range", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    // Multi-range: triggers coordination tier.
    for batch_size in [16, 64] {
        let values = multi_range_values(batch_size);
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("multi_range", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

// ─── Ingest at realistic scale ──────────────────────────────

fn bench_ingest_realistic(c: &mut Criterion) {
    let mut group = c.benchmark_group("ingest_realistic");
    let cfg = realistic_config();

    for batch_size in [16, 64, 256] {
        let values = multi_range_values(batch_size);
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("multi_range", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

// ─── Construction (includes automatic noise injection) ──────

fn bench_construction(c: &mut Criterion) {
    let mut group = c.benchmark_group("construction");

    for rounds in [10, 50] {
        let cfg = SentinelConfig::<u64> {
            noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![rounds]),
            noise_batch_size: 16,
            noise_seed: Some(42),
            ..bench_config()
        };
        group.bench_with_input(BenchmarkId::new("noise_rounds", rounds), &cfg, |b, cfg| {
            b.iter(|| Sentinel128::new(black_box(cfg.clone())).unwrap());
        });
    }

    group.finish();
}

// ─── ADR-S-013 §1: Warm-up convergence benchmarks ──────────

/// Measure construction cost at candidate noise schedule values.
///
/// This answers: "how many milliseconds does it cost to increase
/// noise rounds from 50 to 100, 200, or 400?"
///
/// Two configs are tested:
/// - **bench**: `max_rank=4`, `analysis_k=16`, `b=4`, `λ=0.95`
/// - **realistic**: `max_rank=16`, `analysis_k=1024`, `b=16`, `λ=0.99`
fn bench_noise_round_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("noise_round_scaling");
    group.sample_size(20); // construction is expensive at high rounds

    for rounds in [10, 50, 100, 200, 400] {
        // Bench config (small)
        let cfg = SentinelConfig::<u64> {
            noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![rounds]),
            noise_seed: Some(42),
            ..bench_config()
        };
        group.bench_with_input(BenchmarkId::new("bench", rounds), &cfg, |b, cfg| {
            b.iter(|| Sentinel128::new(black_box(cfg.clone())).unwrap());
        });

        // Realistic config (production-like)
        let cfg = SentinelConfig::<u64> {
            noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![rounds]),
            noise_seed: Some(42),
            ..realistic_config()
        };
        group.bench_with_input(BenchmarkId::new("realistic", rounds), &cfg, |b, cfg| {
            b.iter(|| Sentinel128::new(black_box(cfg.clone())).unwrap());
        });
    }

    group.finish();
}

/// Detailed warm-up convergence cost (ADR-S-013 §4).
///
/// Measures the actual wall-clock cost of construction + noise
/// injection at a dense range of noise schedule values for the
/// production-like config.  This extends `bench_noise_round_scaling`
/// with finer granularity to identify the cost knee-point.
///
/// Ported from the diagnostic `wall_clock_convergence_cost` test in
/// `convergence_benchmark.rs`.
fn bench_warmup_cost_detailed(c: &mut Criterion) {
    let mut group = c.benchmark_group("warmup_cost_detailed");
    group.sample_size(10); // construction is expensive at high rounds

    for rounds in [5, 10, 20, 50, 65, 100, 150, 200, 300, 400, 500] {
        let cfg = SentinelConfig::<u64> {
            noise_schedule: torrust_sentinel::NoiseSchedule::Explicit(vec![rounds]),
            noise_seed: Some(42),
            ..realistic_config()
        };
        group.bench_with_input(BenchmarkId::new("production", rounds), &cfg, |b, cfg| {
            b.iter(|| Sentinel128::new(black_box(cfg.clone())).unwrap());
        });
    }

    group.finish();
}

/// Measure per-round `ingest()` cost on a warmed sentinel.
///
/// This answers: "if we increase noise rounds by 100, how many
/// additional ms does that cost?" — by measuring the marginal cost
/// of one `ingest()` call at various batch sizes.
fn bench_per_round_ingest(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_round_ingest");

    // Bench config — single range, varying batch size.
    for batch_size in [4, 16] {
        let values = single_range_values(batch_size);
        let cfg = bench_config();
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("bench", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    // Realistic config — single range, varying batch size.
    for batch_size in [4, 16] {
        let values = single_range_values(batch_size);
        let cfg = realistic_config();
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("realistic", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

// ─── Health / inspection ────────────────────────────────────

fn bench_health(c: &mut Criterion) {
    let cfg = bench_config();
    let s = warmed_sentinel(&cfg);

    c.bench_function("health", |b| {
        b.iter(|| s.health());
    });
}

fn bench_analysis_set_summary(c: &mut Criterion) {
    let cfg = bench_config();
    let s = warmed_sentinel(&cfg);

    c.bench_function("analysis_set_summary", |b| {
        b.iter(|| s.analysis_set().summary());
    });
}

// ─── Per-sample scoring overhead ────────────────────────────

fn bench_per_sample_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("per_sample_scores");

    let batch_size = 32;
    let values = single_range_values(batch_size);

    for enabled in [false, true] {
        let cfg = SentinelConfig::<u64> {
            per_sample_scores: enabled,
            ..bench_config()
        };

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("enabled", enabled), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

// ─── Analysis K scaling ─────────────────────────────────────

fn bench_analysis_k_scaling(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_k_scaling");
    let batch_size = 16;
    let values = single_range_values(batch_size);

    for k in [4, 16, 64, 256] {
        let cfg = SentinelConfig::<u64> {
            analysis_k: k,
            ..bench_config()
        };

        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("k", k), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

// ─── Registration ───────────────────────────────────────────

criterion_group!(encoding, bench_centred_bits);

criterion_group!(ingest, bench_ingest_cold, bench_ingest_warm, bench_ingest_realistic);

criterion_group!(auxiliary, bench_construction, bench_health, bench_analysis_set_summary);

criterion_group!(
    convergence,
    bench_noise_round_scaling,
    bench_warmup_cost_detailed,
    bench_per_round_ingest,
);

criterion_group!(scaling, bench_per_sample_overhead, bench_analysis_k_scaling);

// ─── §9.11 — Temporal and analysis benchmarks ──────────────

/// Generate `count` values in a narrow range with leading nibble
/// `nibble` and sequential low bits.
fn cell_values(nibble: u128, count: usize) -> Vec<u128> {
    (0..count).map(|i| (nibble << 124) | (i as u128 + 1)).collect()
}

fn bench_decay(c: &mut Criterion) {
    let mut group = c.benchmark_group("decay");
    let cfg = bench_config();

    for obs_count in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("observations", obs_count), &obs_count, |b, &n| {
            b.iter_with_setup(
                || {
                    let mut s = Sentinel128::new(cfg.clone()).unwrap();
                    s.ingest(&multi_range_values(n));
                    s
                },
                |mut s| s.decay(black_box(0.5), black_box(0.0)),
            );
        });
    }

    group.finish();
}

fn bench_decay_subtree(c: &mut Criterion) {
    let mut group = c.benchmark_group("decay_subtree");
    let cfg = bench_config();

    for obs_count in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("observations", obs_count), &obs_count, |b, &n| {
            b.iter_with_setup(
                || {
                    let mut s = Sentinel128::new(cfg.clone()).unwrap();
                    s.ingest(&multi_range_values(n));
                    s
                },
                |mut s| {
                    let root = s.graph().g_root();
                    s.decay_subtree(root, black_box(0.5), black_box(0.0));
                },
            );
        });
    }

    group.finish();
}

fn bench_analysis_set_recompute(c: &mut Criterion) {
    let mut group = c.benchmark_group("analysis_set_recompute");

    for k in [16, 64, 256] {
        let cfg = SentinelConfig::<u64> {
            analysis_k: k,
            split_threshold: 10,
            budget: 50_000,
            ..bench_config()
        };

        group.throughput(Throughput::Elements(8));
        group.bench_with_input(BenchmarkId::new("k", k), &cfg, |b, cfg| {
            b.iter_with_setup(
                || {
                    let mut s = Sentinel128::new(cfg.clone()).unwrap();
                    // Pre-populate graph with diverse traffic.
                    for nibble in 0..16u128 {
                        s.ingest(&cell_values(nibble, 100));
                    }
                    s
                },
                |mut s| s.ingest(black_box(&cell_values(0xA, 8))),
            );
        });
    }

    group.finish();
}

fn bench_report_assembly(c: &mut Criterion) {
    let mut group = c.benchmark_group("report_assembly");
    let cfg = bench_config();

    for batch_size in [8, 32, 128] {
        let values = multi_range_values(batch_size);
        group.throughput(Throughput::Elements(batch_size as u64));
        group.bench_with_input(BenchmarkId::new("batch_size", batch_size), &values, |b, vals| {
            b.iter_with_setup(|| warmed_sentinel(&cfg), |mut s| s.ingest(black_box(vals)));
        });
    }

    group.finish();
}

criterion_group!(temporal, bench_decay, bench_decay_subtree);

criterion_group!(analysis, bench_analysis_set_recompute, bench_report_assembly);

criterion_main!(encoding, ingest, auxiliary, convergence, scaling, temporal, analysis);
