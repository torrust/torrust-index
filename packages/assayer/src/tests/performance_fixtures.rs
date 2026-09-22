// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`performance_case_table_declares_the_six_plan_rows_once`] | performance | The shared case table contains the six planned workloads exactly once, and the dense row retains its warm-up and measured populations separately. |
//! | [`every_performance_case_returns_its_declared_runtime_layout`] | performance | Every guarded case constructor reaches the real scenario and returns only with the complete published layout declared by its own row. |
//! | [`seeded_populations_report_every_declared_count_and_mix`] | performance | Label and pre-seed generators report the size, adverse and benign mix, eligibility, ground truth, outcome values, and entity cardinality declared by their case rows. |
//! | [`fixed_clock_origin_guard_rejects_a_different_origin`] | performance | The fixed-clock guard refuses a world whose existing virtual clock did not begin at the harness epoch and reports both timestamps. Invalid precondition: the world starts one second after the declared epoch. |
//! | [`fixed_clock_motion_guard_rejects_a_sample_that_advances_time`] | performance | The fixed-clock guard refuses a sample that moves the world's existing clock and reports its entry and exit readings. Invalid precondition: the sampled operation advances the clock by one second. |
//! | [`no_persistence_guard_rejects_configured_durable_paths`] | performance | The named no-op persistence guard refuses configured checkpoint and journal paths and carries both paths in its typed failure. Invalid precondition: both durable directories are configured. |
//! | [`channel_policy_guard_rejects_an_extra_action`] | performance | The performance channel guard refuses the ordinary four-action default because the case table declares Allow, Challenge, and Block only. Invalid precondition: the policy includes the additional Slow action. |
//! | [`population_guard_rejects_an_invalid_mix`] | performance | The population guard refuses an adverse count that differs from the declared seeded mix and returns the complete expected and observed summaries. Invalid precondition: the reference population reports one fewer adverse item. |
//! | [`synthetic_report_guard_rejects_an_invalid_acknowledgement`] | performance | The report guard refuses a structural acknowledgement with the wrong cell count and returns every declared and measured report precondition. Invalid precondition: one cell was declared and two were acknowledged. |
//! | [`synthetic_report_generator_installs_a_guarded_population`] | performance | The synthetic generator accepts its Sentinel count, slot width, cell count, and outcome-axis width as parameters, then returns only after structural acknowledgement and clean health. |
//! | [`synthetic_report_generator_rejects_an_empty_report`] | performance | The generator refuses a zero-cell declaration before ingestion because no structurally valid report can acknowledge it. Invalid precondition: the report declares zero cells. |
//! | [`clean_health_guard_rejects_a_degraded_precondition`] | performance | The health guard refuses a non-zero signal-sanitisation count and returns all health fields it checked. Invalid precondition: one signal was sanitised. |
//! | [`health_model_row_guard_rejects_a_missing_row`] | performance | The health fixture refuses a full-report model-row count below its declaration and carries both counts. Invalid precondition: three rows were observed where four were declared. |
//! | [`pre_seed_completion_guard_rejects_a_partial_result`] | performance | The bulk fixture refuses a pre-seed result that processed fewer entries than declared and reports processed and failed counts. Invalid precondition: nine of ten entries were reported as processed. |
//! | [`reference_removal_guard_rejects_trivial_training`] | performance | The lifecycle guard refuses a reference removal with no eligible training even when its widths and correction result otherwise match. Invalid precondition: the trained-label count is zero. |

//! Guard coverage for the six performance cases and their seeded fixtures.
//!
//! Invalid inputs are stated at each guard: a wrong clock origin, clock motion,
//! durable paths, an extra channel action, a mismatched population count, a bad
//! structural acknowledgement, an empty report, dirty health, a missing health
//! row, an incomplete pre-seed result, and trivial lifecycle training.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use crate::api::PreSeedResult;
use crate::config::types::PersistenceConfig;
use crate::testing::performance::{
    HealthObservation, PERFORMANCE_CASES, PerformanceCase, PerformanceFixtureError, ReferenceRemovalObservation,
    SeededLabelPopulation, SeededPreSeedPopulation, SyntheticReportDeclaration, SyntheticReportObservation, guard_channel_policy,
    guard_clean_health, guard_health_model_rows, guard_no_persistence, guard_population, guard_pre_seed_result,
    guard_reference_removal, guard_synthetic_reports, install_synthetic_reports, with_fixed_clock_sample,
};
use crate::testing::{VirtualClock, World};
use crate::types::Action;
use crate::{AssayerConfig, ChannelPolicy};

/// The shared case table contains the six planned workloads exactly once, and the dense row retains its warm-up and measured populations separately.
///
/// ´claim:performance:the-performance-case-table-is-the-single-declaration-of-all-six-workloads´
/// ´test:crate:performance-case-table-declares-the-six-plan-rows-once´
#[test]
fn performance_case_table_declares_the_six_plan_rows_once() {
    let cases: Vec<_> = PERFORMANCE_CASES.iter().map(|row| row.case).collect();
    assert_eq!(
        cases,
        vec![
            PerformanceCase::Cold,
            PerformanceCase::ReferenceSteady,
            PerformanceCase::DenseLabelProfile,
            PerformanceCase::PreseedThousand,
            PerformanceCase::HealthReference,
            PerformanceCase::ReferenceRemoval,
        ]
    );
    let dense = PerformanceCase::DenseLabelProfile.declaration();
    assert_eq!(dense.population.warmup_labels, 8);
    assert_eq!(dense.population.labels, 128);
    assert_eq!(dense.mix.items, 136);
    for row in PERFORMANCE_CASES {
        assert_eq!(row.channel_policy, [Action::Allow, Action::Challenge, Action::Block]);
    }
}

/// Every guarded case constructor reaches the real scenario and returns only with the complete published layout declared by its own row.
///
/// ´claim:performance:every-performance-case-returns-only-after-its-runtime-layout-matches-its-row´
/// ´test:crate:every-performance-case-returns-its-declared-runtime-layout´
#[test]
fn every_performance_case_returns_its_declared_runtime_layout() {
    for row in PERFORMANCE_CASES {
        let world = row
            .case
            .world(&format!("performance-layout-{:?}", row.case))
            .expect("declared performance case should establish every precondition");
        assert_eq!(
            world.runtime_layout().expect("guarded world layout remains readable"),
            row.dimensions.runtime
        );
    }
}

/// Label and pre-seed generators report the size, adverse and benign mix, eligibility, ground truth, outcome values, and entity cardinality declared by their case rows.
///
/// ´claim:performance:seeded-populations-report-every-declared-count-and-mix-before-use´
/// ´test:crate:seeded-populations-report-every-declared-count-and-mix´
#[test]
fn seeded_populations_report_every_declared_count_and_mix() {
    for case in [
        PerformanceCase::Cold,
        PerformanceCase::ReferenceSteady,
        PerformanceCase::DenseLabelProfile,
        PerformanceCase::HealthReference,
        PerformanceCase::ReferenceRemoval,
    ] {
        let summary = SeededLabelPopulation::for_case(case)
            .expect("declared label population should guard cleanly")
            .summary();
        assert_eq!(summary.items, case.declaration().mix.items);
        assert_eq!(summary.adverse, case.declaration().mix.adverse);
        assert_eq!(summary.benign, case.declaration().mix.benign);
        assert_eq!(summary.eligible, case.declaration().mix.eligible);
        assert_eq!(summary.ground_truth, case.declaration().mix.ground_truth);
        assert_eq!(
            summary.outcome_values,
            case.declaration().mix.items * case.declaration().mix.outcome_axis_width
        );
        assert_eq!(summary.entity_cardinality, case.declaration().mix.entity_cardinality);
    }
    let preseed = SeededPreSeedPopulation::for_case(PerformanceCase::PreseedThousand)
        .expect("declared pre-seed population should guard cleanly")
        .summary();
    assert_eq!(preseed.items, 1_000);
    assert_eq!(preseed.adverse, 200);
    assert_eq!(preseed.benign, 800);
    assert_eq!(preseed.entity_cardinality, 1_000);
}

/// The fixed-clock guard refuses a world whose existing virtual clock did not begin at the harness epoch and reports both timestamps.
///
/// Invalid precondition: the world starts one second after the declared epoch.
///
/// ´claim:performance:the-fixed-clock-guard-requires-the-declared-origin´
/// ´test:crate:fixed-clock-origin-guard-rejects-a-different-origin´
#[test]
fn fixed_clock_origin_guard_rejects_a_different_origin() {
    let clock = Arc::new(VirtualClock::at_secs(VirtualClock::EPOCH_SECS + 1));
    let world = World::builder(AssayerConfig {
        instance_id: "test".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    })
    .channel("default", ChannelPolicy::default())
    .clock(clock)
    .seed(1)
    .build()
    .expect("invalid-origin world should still build");

    let error = with_fixed_clock_sample(&world, |_| ()).expect_err("wrong origin must be refused");

    assert!(matches!(error, PerformanceFixtureError::ClockOrigin { expected, observed } if expected != observed));
}

/// The fixed-clock guard refuses a sample that moves the world's existing clock and reports its entry and exit readings.
///
/// Invalid precondition: the sampled operation advances the clock by one second.
///
/// ´claim:performance:the-fixed-clock-guard-requires-time-to-remain-frozen-through-a-sample´
/// ´test:crate:fixed-clock-motion-guard-rejects-a-sample-that-advances-time´
#[test]
fn fixed_clock_motion_guard_rejects_a_sample_that_advances_time() {
    let world = World::cold("performance-clock-motion", 2);

    let error = with_fixed_clock_sample(&world, |sample| sample.clock().advance(Duration::from_secs(1)))
        .expect_err("clock motion must be refused");

    assert!(matches!(error, PerformanceFixtureError::ClockMoved { before, after } if after > before));
}

/// The named no-op persistence guard refuses configured checkpoint and journal paths and carries both paths in its typed failure.
///
/// Invalid precondition: both durable directories are configured.
///
/// ´claim:performance:the-no-op-persistence-fixture-requires-the-absence-of-durable-paths´
/// ´test:crate:no-persistence-guard-rejects-configured-durable-paths´
#[test]
fn no_persistence_guard_rejects_configured_durable_paths() {
    let mut config = AssayerConfig {
        instance_id: "test".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    config.persistence = Some(PersistenceConfig::new(
        PathBuf::from("checkpoint-configured"),
        PathBuf::from("journal-configured"),
    ));

    let error = guard_no_persistence(&config).expect_err("configured persistence must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::PersistenceConfigured {
            checkpoint_dir,
            journal_dir,
        } if checkpoint_dir.as_os_str() == "checkpoint-configured"
            && journal_dir.as_os_str() == "journal-configured"
    ));
}

/// The performance channel guard refuses the ordinary four-action default because the case table declares Allow, Challenge, and Block only.
///
/// Invalid precondition: the policy includes the additional Slow action.
///
/// ´claim:performance:the-performance-policy-guard-requires-the-declared-three-action-order´
/// ´test:crate:channel-policy-guard-rejects-an-extra-action´
#[test]
fn channel_policy_guard_rejects_an_extra_action() {
    let declaration = PerformanceCase::Cold.declaration();
    let error = guard_channel_policy(&declaration.channel_policy, &ChannelPolicy::default())
        .expect_err("a fourth action must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::ChannelPolicyMismatch { expected, observed }
            if expected.len() == 3 && observed.len() == 4
    ));
}

/// The population guard refuses an adverse count that differs from the declared seeded mix and returns the complete expected and observed summaries.
///
/// Invalid precondition: the reference population reports one fewer adverse item.
///
/// ´claim:performance:the-population-guard-requires-every-declared-count-and-mix´
/// ´test:crate:population-guard-rejects-an-invalid-mix´
#[test]
fn population_guard_rejects_an_invalid_mix() {
    let declaration = PerformanceCase::ReferenceSteady.declaration().mix;
    let mut observed = SeededLabelPopulation::for_case(PerformanceCase::ReferenceSteady)
        .expect("reference population declaration is valid")
        .summary();
    observed.adverse -= 1;

    let error = guard_population(declaration, observed).expect_err("wrong adverse mix must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::PopulationMismatch { expected, observed }
            if expected.adverse == observed.adverse + 1
    ));
}

/// The report guard refuses a structural acknowledgement with the wrong cell count and returns every declared and measured report precondition.
///
/// Invalid precondition: one cell was declared and two were acknowledged.
///
/// ´claim:performance:the-synthetic-report-guard-requires-structural-acknowledgement-of-its-declared-shape´
/// ´test:crate:synthetic-report-guard-rejects-an-invalid-acknowledgement´
#[test]
fn synthetic_report_guard_rejects_an_invalid_acknowledgement() {
    let declaration = SyntheticReportDeclaration {
        sentinel_count: 1,
        slot_width: 61,
        cell_count: 1,
        outcome_axis_width: 0,
    };
    let observed = SyntheticReportObservation {
        sentinel_count: 1,
        slot_width: 61,
        cell_count: 2,
        outcome_axis_width: 0,
        acknowledgements: 1,
        degraded_cells: 0,
    };

    let error = guard_synthetic_reports(declaration, observed).expect_err("wrong acknowledgement must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::SyntheticReportMismatch { expected, observed }
            if expected.cell_count == 1 && observed.cell_count == 2
    ));
}

/// The synthetic generator accepts its Sentinel count, slot width, cell count, and outcome-axis width as parameters, then returns only after structural acknowledgement and clean health.
///
/// ´claim:performance:synthetic-reports-are-guarded-by-shape-acknowledgement-and-clean-health´
/// ´test:crate:synthetic-report-generator-installs-a-guarded-population´
#[test]
fn synthetic_report_generator_installs_a_guarded_population() {
    let mut world = World::cold("performance-synthetic-reports", 3);
    world.register_sentinel("S1").expect("fixture Sentinel should register");
    let declaration = SyntheticReportDeclaration {
        sentinel_count: 1,
        slot_width: 61,
        cell_count: 2,
        outcome_axis_width: 0,
    };

    let observed = install_synthetic_reports(&world, declaration).expect("declared synthetic report should install");

    assert_eq!(
        observed,
        SyntheticReportObservation {
            sentinel_count: 1,
            slot_width: 61,
            cell_count: 2,
            outcome_axis_width: 0,
            acknowledgements: 1,
            degraded_cells: 0,
        }
    );
}

/// The generator refuses a zero-cell declaration before ingestion because no structurally valid report can acknowledge it.
///
/// Invalid precondition: the report declares zero cells.
///
/// ´claim:performance:the-synthetic-report-generator-requires-a-non-empty-structural-shape´
/// ´test:crate:synthetic-report-generator-rejects-an-empty-report´
#[test]
fn synthetic_report_generator_rejects_an_empty_report() {
    let world = World::cold("performance-empty-report", 4);
    let declaration = SyntheticReportDeclaration {
        sentinel_count: 0,
        slot_width: 0,
        cell_count: 0,
        outcome_axis_width: 0,
    };

    let error = install_synthetic_reports(&world, declaration).expect_err("an empty report must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::SyntheticReportUnsupported {
            sentinel_count: 0,
            cell_count: 0,
        }
    ));
}

/// The health guard refuses a non-zero signal-sanitisation count and returns all health fields it checked.
///
/// Invalid precondition: one signal was sanitised.
///
/// ´claim:performance:the-clean-health-guard-refuses-every-degradation-counter-and-stop-condition´
/// ´test:crate:clean-health-guard-rejects-a-degraded-precondition´
#[test]
fn clean_health_guard_rejects_a_degraded_precondition() {
    let observed = HealthObservation {
        signals_sanitised: 1,
        ..HealthObservation::default()
    };

    let error = guard_clean_health(observed).expect_err("dirty health must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::HealthNotClean { observed } if observed.signals_sanitised == 1
    ));
}

/// The health fixture refuses a full-report model-row count below its declaration and carries both counts.
///
/// Invalid precondition: three rows were observed where four were declared.
///
/// ´claim:performance:the-health-reference-guard-requires-every-declared-model-row´
/// ´test:crate:health-model-row-guard-rejects-a-missing-row´
#[test]
fn health_model_row_guard_rejects_a_missing_row() {
    let error = guard_health_model_rows(4, 3).expect_err("missing model row must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::HealthModelRows {
            expected: 4,
            observed: 3,
        }
    ));
}

/// The bulk fixture refuses a pre-seed result that processed fewer entries than declared and reports processed and failed counts.
///
/// Invalid precondition: nine of ten entries were reported as processed.
///
/// ´claim:performance:the-pre-seed-guard-requires-the-whole-population-to-complete´
/// ´test:crate:pre-seed-completion-guard-rejects-a-partial-result´
#[test]
fn pre_seed_completion_guard_rejects_a_partial_result() {
    let result = PreSeedResult {
        processed: 9,
        failed: 0,
        final_p_positive: 0.5,
    };

    let error = guard_pre_seed_result(10, &result).expect_err("partial pre-seed must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::PreSeedIncomplete {
            expected: 10,
            processed: 9,
            failed: 0,
        }
    ));
}

/// The lifecycle guard refuses a reference removal with no eligible training even when its widths and correction result otherwise match.
///
/// Invalid precondition: the trained-label count is zero.
///
/// ´claim:performance:the-reference-removal-guard-requires-nontrivial-training-and-a-nonfallback-correction´
/// ´test:crate:reference-removal-guard-rejects-trivial-training´
#[test]
fn reference_removal_guard_rejects_trivial_training() {
    let row = PerformanceCase::ReferenceRemoval.declaration();
    let observed = ReferenceRemovalObservation {
        starting_width: 638,
        departing_block_width: 68,
        trained_labels: 0,
        retained_width: 570,
        marginalisation_events: 1,
        corrections_skipped: 0,
    };

    let error =
        guard_reference_removal(row.dimensions, row.population.labels, observed).expect_err("trivial training must be refused");

    assert!(matches!(
        error,
        PerformanceFixtureError::ReferenceRemovalMismatch { observed, .. } if observed.trained_labels == 0
    ));
}
