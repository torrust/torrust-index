// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::time::Duration;

use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use torrust_mudlark::GvGraph;
use torrust_mudlark::testing::{
    Plan, budget_config, default_config, f64_default_config, plan_adversarial, plan_budget_burst, plan_cousin_rivalry,
    plan_diamond, plan_fractal_fill, plan_gray_code, plan_growth_pressure_relax, plan_left_deep, plan_phase_shifted,
    plan_right_deep, plan_single_hotspot, run, tight_budget_config,
};

// ── Spine comparison group ──────────────────────────────────────
//
// Left-deep, right-deep, and adversarial zigzag in a single
// BenchmarkGroup for direct violin-plot comparison.

fn bench_path_spine(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/spine");

    for n in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::new("left_deep", n), &n, |b, &n| {
            b.iter_batched(
                || plan_left_deep(6, n),
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

        group.bench_with_input(BenchmarkId::new("right_deep", n), &n, |b, &n| {
            b.iter_batched(
                || plan_right_deep(256, 6, n),
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

        group.bench_with_input(BenchmarkId::new("adversarial_zigzag", n), &n, |b, &n| {
            b.iter_batched(
                || plan_adversarial(256, n),
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

// ── Single hotspot (parameterised) ──────────────────────────────

fn bench_path_single_hotspot(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/single_hotspot");

    for n in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || plan_single_hotspot(128, 6, n),
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

// ── Non-parameterised pathological benchmarks ───────────────────

fn bench_path_budget_burst(c: &mut Criterion) {
    c.bench_function("pathological/budget_burst", |b| {
        b.iter_batched(
            || plan_budget_burst(256, 5_000, 1_000),
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

fn bench_path_growth_pressure_relax(c: &mut Criterion) {
    c.bench_function("pathological/growth_pressure_relax", |b| {
        b.iter_batched(
            || plan_growth_pressure_relax(256, 100),
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

fn bench_path_oscillating_hotspot(c: &mut Criterion) {
    c.bench_function("pathological/oscillating_hotspot", |b| {
        b.iter_batched(
            || Plan::<u64, u64>::new().oscillating_hotspot(0, 255, 10, 200, 20),
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

fn bench_path_tight_budget_spray(c: &mut Criterion) {
    c.bench_function("pathological/tight_budget_spray", |b| {
        b.iter_batched(
            || Plan::<u64, u64>::new().random_spray(42, 256, 6, 10_000),
            |plan| {
                let mut g = GvGraph::<u64, u64, 8>::new(tight_budget_config());
                for &(coord, delta) in &plan.observations {
                    g.observe(coord, delta);
                }
                g
            },
            criterion::BatchSize::SmallInput,
        );
    });
}

// ── Targeted / structural patterns ──────────────────────────────
//
// Exercise the concurrent-violation, fractal, and topology code
// paths that the spine/single-hotspot groups don't reach.

fn bench_path_cousin_rivalry(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/cousin_rivalry");

    for n in [100, 500, 2_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || plan_cousin_rivalry(n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_path_phase_shifted(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/phase_shifted");

    for n in [200, 1_000, 4_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || plan_phase_shifted(32, 96, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_path_fractal_fill(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/fractal_fill");

    for depth in [3u32, 5, 7] {
        group.bench_with_input(BenchmarkId::from_parameter(depth), &depth, |b, &depth| {
            b.iter_batched(
                || plan_fractal_fill(256, depth),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_path_diamond(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/diamond");

    for n in [200, 1_000, 5_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || plan_diamond(256, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_path_gray_code(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/gray_code");

    for n in [256, 1_000, 5_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || plan_gray_code(256, n),
                |plan| run::<u64, u64, 8>(default_config(), &plan),
                criterion::BatchSize::SmallInput,
            );
        });
    }

    group.finish();
}

fn bench_path_centroid_drift(c: &mut Criterion) {
    c.bench_function("pathological/centroid_drift", |b| {
        b.iter_batched(
            || Plan::<u64, u64>::new().centroid_drift(0, 200, 8, 20),
            |plan| run::<u64, u64, 8>(default_config(), &plan),
            criterion::BatchSize::SmallInput,
        );
    });
}

fn bench_path_golden_spiral(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/golden_spiral");

    for n in [500, 2_000, 10_000] {
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

fn bench_path_pincer(c: &mut Criterion) {
    c.bench_function("pathological/pincer", |b| {
        b.iter_batched(
            || Plan::<u64, u64>::new().pincer(0, 255, 6, 10),
            |plan| run::<u64, u64, 8>(default_config(), &plan),
            criterion::BatchSize::SmallInput,
        );
    });
}

// ── pathological/left_deep_f64 ──────────────────────────────────
//
// Phase 8 representative: f64 secondary configuration.
// Uses a single hotspot at 0.0 to exercise the degenerate
// left-spine path with floating-point coordinates.

fn bench_path_left_deep_f64(c: &mut Criterion) {
    let mut group = c.benchmark_group("pathological/left_deep_f64");

    for n in [100, 1_000, 10_000] {
        group.bench_with_input(BenchmarkId::from_parameter(n), &n, |b, &n| {
            b.iter_batched(
                || Plan::<f64, f64>::new().hotspot(0.0, 6.0, n),
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

criterion_group! {
    name = benches;
    config = Criterion::default().warm_up_time(Duration::from_millis(500)).measurement_time(Duration::from_secs(1));
    targets =
    bench_path_spine,
    bench_path_single_hotspot,
    bench_path_budget_burst,
    bench_path_growth_pressure_relax,
    bench_path_oscillating_hotspot,
    bench_path_tight_budget_spray,
    bench_path_cousin_rivalry,
    bench_path_phase_shifted,
    bench_path_fractal_fill,
    bench_path_diamond,
    bench_path_gray_code,
    bench_path_centroid_drift,
    bench_path_golden_spiral,
    bench_path_pincer,
    bench_path_left_deep_f64,
}
criterion_main!(benches);
