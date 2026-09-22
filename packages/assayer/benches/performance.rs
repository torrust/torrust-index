// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Criterion measurements of the Assayer cost model on the shared scenario.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`bench_assessment`] | audit | Core assessment and host derivation are measured separately over cold and reference-width worlds, preserving the former operating-cycle witness without concealing the read/write split; the documented comparison floor is six milliseconds per completed assess–derive–label round trip, and the baseline is compared rather than asserted. |
//! | [`bench_label_publication`] | audit | Complete real-engine label publication and its sequential dense model-update region are measured separately on the representative workload, preserving the former attribution witness on one publication path; the attribution split has no asserted floor, while the documented cross-group floor is six milliseconds per completed round trip and is compared rather than asserted. |
//! | [`bench_construction_shutdown`] | audit | Building and dropping a guarded real-engine world is measured across every declared performance configuration, including the owner-thread join; the documented comparison floor is three hundred milliseconds per complete build/drop cycle, and the baseline is compared rather than asserted. |
//! | [`bench_pre_seed`] | audit | One synchronous pre-seed call is measured over the guarded thousand-entry population and returns only after every entry has been processed; the documented comparison floor is one thousand accepted entries within thirty seconds, and the baseline is compared rather than asserted. |
//! | [`bench_health_summary`] | audit | Health-summary polling is measured without mutation over cold and populated worlds; the documented comparison floor is one millisecond per call, and the baseline is compared rather than asserted. |
//! | [`bench_full_health_report`] | audit | Full-health-report assembly is measured separately from its summary over cold and populated worlds so its per-Sentinel, per-dimension, and per-model walk remains visible; the documented comparison floor is fifty milliseconds per call, and the baseline is compared rather than asserted. |
//! | [`bench_reference_marginalisation`] | bayes | Deregistering one Sentinel is measured on a guarded, trained reference-width world through the real lifecycle barrier, replacing the private-model Schur witness; the documented comparison floor is three hundred milliseconds per completed removal, and the baseline is compared rather than asserted. |

use std::fmt::Display;
use std::hint::black_box;
use std::time::Duration;

use criterion::{BatchSize, BenchmarkGroup, BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use torrust_assayer::testing::performance::{
    PerformanceCase, SeededLabelPopulation, SeededPreSeedPopulation, reference_removal_world, with_fixed_clock_sample,
};

const CHEAP_SAMPLE_SIZE: usize = 100;
const EXPENSIVE_SAMPLE_SIZE: usize = 20;
const WARM_UP: Duration = Duration::from_secs(3);
const ASSESSMENT_CASES: [(PerformanceCase, &str); 2] = [
    (PerformanceCase::Cold, "cold"),
    (PerformanceCase::ReferenceSteady, "reference_steady"),
];
const CONSTRUCTION_CASES: [(PerformanceCase, &str); 6] = [
    (PerformanceCase::Cold, "cold"),
    (PerformanceCase::ReferenceSteady, "reference_steady"),
    (PerformanceCase::DenseLabelProfile, "dense_label_profile"),
    (PerformanceCase::PreseedThousand, "preseed_thousand"),
    (PerformanceCase::HealthReference, "health_reference"),
    (PerformanceCase::ReferenceRemoval, "reference_removal"),
];
const HEALTH_CASES: [(PerformanceCase, &str); 2] = [
    (PerformanceCase::Cold, "cold"),
    (PerformanceCase::HealthReference, "health_reference"),
];

fn fixture<T, E: Display>(result: Result<T, E>) -> T {
    match result {
        Ok(value) => value,
        Err(error) => panic!("performance fixture rejected the sample: {error}"),
    }
}

fn count(value: usize) -> u64 {
    match u64::try_from(value) {
        Ok(value) => value,
        Err(error) => panic!("performance fixture count does not fit the Criterion throughput unit: {error}"),
    }
}

fn command_line_overrides(option: &str) -> bool {
    let assignment = format!("{option}=");
    std::env::args_os().any(|argument| {
        let argument = argument.as_encoded_bytes();
        argument == option.as_bytes() || argument.starts_with(assignment.as_bytes())
    })
}

fn configure_group(group: &mut BenchmarkGroup<'_, criterion::measurement::WallTime>, sample_size: usize) {
    if !command_line_overrides("--warm-up-time") {
        group.warm_up_time(WARM_UP);
    }
    if !command_line_overrides("--sample-size") {
        group.sample_size(sample_size);
    }
}

/// Core assessment and host derivation are measured separately over cold and reference-width worlds, preserving the former operating-cycle witness without concealing the read/write split; the documented comparison floor is six milliseconds per completed assess–derive–label round trip, and the baseline is compared rather than asserted.
///
/// ´claim:audit:the-full-assess-derive-label-cycle-sustains-thousands-of-round-trips-a-minute´
/// ´test:bench:bench-assessment´
fn bench_assessment(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("assessment");
    configure_group(&mut group, CHEAP_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    for &(case, case_name) in &ASSESSMENT_CASES {
        group.bench_with_input(BenchmarkId::new("core_assess", case_name), &case, |bencher, &case| {
            bencher.iter_batched(
                || {
                    let world = fixture(case.world(&format!("performance-assessment-core-{case_name}")));
                    let request = world.request("default", "assessment-entity");
                    (world, request)
                },
                |(world, request)| {
                    let assessment = with_fixed_clock_sample(&world, |world| world.core_assess(black_box(request)));
                    (world, assessment)
                },
                BatchSize::PerIteration,
            );
        });
        group.bench_with_input(BenchmarkId::new("derive_for_request", case_name), &case, |bencher, &case| {
            bencher.iter_batched(
                || {
                    let world = fixture(case.world(&format!("performance-assessment-derived-{case_name}")));
                    let request = world.request("default", "assessment-entity");
                    (world, request)
                },
                |(world, request)| {
                    let reckoning = with_fixed_clock_sample(&world, |world| world.derive_for_request(black_box(request)));
                    (world, reckoning)
                },
                BatchSize::PerIteration,
            );
        });
    }

    group.finish();
}

/// Complete real-engine label publication and its sequential dense model-update region are measured separately on the representative workload, preserving the former attribution witness on one publication path; the attribution split has no asserted floor, while the documented cross-group floor is six milliseconds per completed round trip and is compared rather than asserted.
///
/// ´claim:audit:the-label-profile-attributes-publication-latency-to-the-sequential-model-update-block´
/// ´test:bench:bench-label-publication´
fn bench_label_publication(criterion: &mut Criterion) {
    let case = PerformanceCase::DenseLabelProfile;
    let declaration = case.declaration();
    let population = fixture(SeededLabelPopulation::for_case(case));
    let mut group = criterion.benchmark_group("label_publication");
    configure_group(&mut group, EXPENSIVE_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(count(declaration.population.labels)));

    group.bench_function(BenchmarkId::new("complete_publication", "dense_label_profile"), |bencher| {
        bencher.iter_batched(
            || {
                let world = fixture(case.world("performance-label-publication"));
                (world, population)
            },
            |(world, population)| {
                let publication = with_fixed_clock_sample(&world, |world| population.publish_measured(world).is_ok()).is_ok();
                (world, publication)
            },
            BatchSize::PerIteration,
        );
    });

    group.bench_function(BenchmarkId::new("dense_model_update", "dense_label_profile"), |bencher| {
        bencher.iter_custom(|iterations| {
            let mut elapsed = Duration::ZERO;
            for _ in 0..iterations {
                let world = fixture(case.world("performance-label-model-update"));
                fixture(world.activate_label_update_recorder());
                let publication = with_fixed_clock_sample(&world, |world| population.publish_measured(world).is_ok()).is_ok();
                elapsed += world.drain_label_update_duration().unwrap_or_default();
                black_box(publication);
            }
            elapsed
        });
    });

    group.finish();
}

/// Building and dropping a guarded real-engine world is measured across every declared performance configuration, including the owner-thread join; the documented comparison floor is three hundred milliseconds per complete build/drop cycle, and the baseline is compared rather than asserted.
///
/// ´claim:audit:building-and-shutting-down-an-engine-is-bounded-work-that-can-be-repeated-freely´
/// ´test:bench:bench-construction-shutdown´
fn bench_construction_shutdown(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("construction_shutdown");
    configure_group(&mut group, EXPENSIVE_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    for &(case, case_name) in &CONSTRUCTION_CASES {
        drop(fixture(case.world(&format!("performance-construction-guard-{case_name}"))));
        group.bench_with_input(BenchmarkId::from_parameter(case_name), &case, |bencher, &case| {
            bencher.iter(|| drop(black_box(case.world(&format!("performance-construction-{case_name}")))));
        });
    }

    group.finish();
}

/// One synchronous pre-seed call is measured over the guarded thousand-entry population and returns only after every entry has been processed; the documented comparison floor is one thousand accepted entries within thirty seconds, and the baseline is compared rather than asserted.
///
/// ´claim:audit:bulk-pre-seeding-accepts-a-thousand-historical-entries-in-one-call-and-processes-every-one´
/// ´test:bench:bench-pre-seed´
fn bench_pre_seed(criterion: &mut Criterion) {
    let case = PerformanceCase::PreseedThousand;
    let declaration = case.declaration();
    let population = fixture(SeededPreSeedPopulation::for_case(case));
    let mut group = criterion.benchmark_group("pre_seed");
    configure_group(&mut group, EXPENSIVE_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(count(declaration.population.preseed_entries)));
    group.bench_function("preseed_thousand", |bencher| {
        bencher.iter_batched(
            || {
                let world = fixture(case.world("performance-pre-seed"));
                let entries = fixture(population.entries(&world));
                (world, entries)
            },
            |(world, entries)| {
                let result = with_fixed_clock_sample(&world, |world| world.assayer().pre_seed(black_box(&entries)));
                (world, entries, result)
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

/// Health-summary polling is measured without mutation over cold and populated worlds; the documented comparison floor is one millisecond per call, and the baseline is compared rather than asserted.
///
/// ´claim:audit:the-health-summary-is-cheap-enough-to-poll-on-every-monitoring-scrape´
/// ´test:bench:bench-health-summary´
fn bench_health_summary(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("health");
    configure_group(&mut group, CHEAP_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    for &(case, case_name) in &HEALTH_CASES {
        let world = fixture(case.world(&format!("performance-health-summary-{case_name}")));
        group.bench_function(BenchmarkId::new("health_summary", case_name), |bencher| {
            bencher
                .iter(|| black_box(with_fixed_clock_sample(&world, |world| black_box(world.assayer().health_summary())).is_ok()));
        });
    }

    group.finish();
}

/// Full-health-report assembly is measured separately from its summary over cold and populated worlds so its per-Sentinel, per-dimension, and per-model walk remains visible; the documented comparison floor is fifty milliseconds per call, and the baseline is compared rather than asserted.
///
/// ´claim:audit:the-full-health-report-stays-affordable-to-assemble-on-demand-despite-walking-every-entity´
/// ´test:bench:bench-full-health-report´
fn bench_full_health_report(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("health");
    configure_group(&mut group, CHEAP_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));

    for &(case, case_name) in &HEALTH_CASES {
        let world = fixture(case.world(&format!("performance-full-health-report-{case_name}")));
        group.bench_function(BenchmarkId::new("full_health_report", case_name), |bencher| {
            bencher.iter(|| {
                black_box(with_fixed_clock_sample(&world, |world| black_box(world.assayer().full_health_report())).is_ok())
            });
        });
    }

    group.finish();
}

/// Deregistering one Sentinel is measured on a guarded, trained reference-width world through the real lifecycle barrier, replacing the private-model Schur witness; the documented comparison floor is three hundred milliseconds per completed removal, and the baseline is compared rather than asserted.
///
/// ´claim:bayes:marginalising-a-large-block-at-the-reference-dimension-stays-within-its-time-budget´
/// ´test:bench:bench-reference-marginalisation´
fn bench_reference_marginalisation(criterion: &mut Criterion) {
    let mut group = criterion.benchmark_group("reference_marginalisation");
    configure_group(&mut group, EXPENSIVE_SAMPLE_SIZE);
    group.throughput(Throughput::Elements(1));
    group.bench_function("reference_removal", |bencher| {
        bencher.iter_batched(
            || fixture(reference_removal_world("performance-reference-removal")),
            |mut world| {
                let removal = world.deregister_sentinel("S1");
                (world, removal)
            },
            BatchSize::PerIteration,
        );
    });
    group.finish();
}

criterion_group!(
    performance,
    bench_assessment,
    bench_label_publication,
    bench_construction_shutdown,
    bench_pre_seed,
    bench_health_summary,
    bench_full_health_report,
    bench_reference_marginalisation,
);
criterion_main!(performance);
