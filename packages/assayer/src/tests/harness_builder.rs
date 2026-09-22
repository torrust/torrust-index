// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`world_builder_forwards_invalid_interaction_template`] | harness | A scenario's interaction declarations cross the harness boundary into the engine's construction validator, so an invalid aggregate operand is returned as the engine's typed build error instead of being ignored or panicking. |
//! | [`register_identity_with_cells_observes_the_declared_set`] | harness | The gated identity fixture drives its declared observation through assessment, waits for the identity owner to derive the competitive set and waits for the model owner to publish it. Success therefore means the declared cell is observable in the resulting layout, not merely queued. |
//! | [`register_identity_with_cells_rejects_an_unobservable_set`] | harness | A gated identity fixture whose declared set cannot be produced by its declared empty workload is refused with both sets. Registration alone is not mistaken for installation, and a caller cannot continue with a layout that only resembles the fixture it requested. |
//! | [`runtime_layout_rejects_a_declared_width_mismatch`] | harness | A builder-side layout declaration is checked against one published snapshot after a barrier. A width mismatch is reported with both complete layouts, making the guard useful to a fixture that must fail before its scenario begins. |
//! | [`world_builder_guard_rejects_an_unreachable_layout_with_its_measured_baseline`] | harness | A builder carrying a completed-layout declaration refuses a target whose fixed signal width differs from the measured construction state and reports the complete initial baseline beside both construction and completed-layout expectations. Invalid precondition: the target adds an undeclared signal. |
//! | [`cold_guard_rejects_an_advanced_ramp_with_its_measured_baseline`] | harness | The cold-world guard refuses a measured ramp that has already accepted an observation and reports every construction fact beside the cold declaration. Invalid precondition: the measured accepted count is non-zero. |
//! | [`scenario_guard_rejects_a_changed_rng_state_with_its_measured_baseline`] | harness | The ordinary scenario guard refuses a generator state that differs from the declared seed and reports the complete measured construction baseline beside the declaration. Invalid precondition: the measured generator state differs from the seed. |
//! | [`scenario_with_guard_rejects_a_missing_channel_with_its_measured_baseline`] | harness | The configured-scenario guard refuses a measured channel population missing the declared channel and reports all construction facts beside the declaration. Invalid precondition: the measured channel population is empty. |
//! | [`scenario_with_config_guard_rejects_a_changed_instance_with_its_measured_baseline`] | harness | The explicit-config scenario guard refuses a measured engine identifier different from the configured identifier and reports all construction facts beside the declaration. Invalid precondition: the measured identifier names another engine. |
//! | [`trained_state_fixture_returns_its_measured_precondition`] | harness | The trained-state fixture returns a real world only after its balanced training and held-out class counts match their declarations and its measured endpoint gap and pairwise rank clear their strict setup floors. |
//! | [`trained_state_guard_rejects_an_invalid_class_count_with_its_measured_baseline`] | harness | The trained-state guard refuses a held-out class count below its declaration and its message carries the complete measured baseline beside the expected counts and strict floors, identifying setup as the failed boundary. Invalid precondition: the measured benign held-out population is one sample short. |
//! | [`public_scenario_reproduces_reference_layout_p638`] | harness | The reference population is constructed only through scenario verbs: one spatial outcome axis, eight Sentinels, two identity dimensions whose declared workloads publish ten competitive cells apiece, and the fourteen interaction templates retained by the builder. Its published block widths match the existing `rebuild_reference_config_p638` witness, including total width 638. |
//! | [`scenario_advance_moves_both_clock_domains_together`] | harness | One scenario advance moves the persistent and intra-process readings by the same declared interval while their types remain distinct. |
//! | [`scenario_advance_completes_the_ledger_gc_cycle_it_makes_due`] | harness | An entry that becomes collectable only after scenario time advances is absent when the verb returns, proving completion through the Ledger barrier's own state transition rather than elapsed time. |
//! | [`scenario_travel_targets_both_clock_domains_together`] | harness | Targeted scenario travel places the persistent reading at the requested instant and moves the intra-process reading by the same offset. |
//! | [`scenario_travel_refuses_a_backward_target_with_both_instants`] | harness | A backward target is refused without moving time, and its diagnostic names the current and requested persistent instants. Invalid precondition: the target is one hour before the scenario clock. |

//! Crate tests for the scenario builder's complete construction contract.
//!
//! These checks stay above the real engine rather than rebuilding a dimension map directly: every population member arrives through a lifecycle verb, identity cells arise from declared observations, and the final widths are read from the engine's published snapshot.

use std::time::{Duration, UNIX_EPOCH};

use crate::ChannelPolicy;
use crate::error::BuildError;
use crate::ledger::entry::LedgerEntry;
use crate::testing::{
    Clock, CompetitiveCellId, CompetitiveCellSpec, ConstructionFixture, FeatureSelector, InteractionTemplate, RuntimeLayout,
    TrainedStateBaseline, World, WorldBuildError, WorldConstructionBaseline, WorldConstructionDeclaration, WorldFixtureError,
};
use crate::types::{IdentityBudget, LedgerKey, PersistentTimestamp, SentinelId};

const REFERENCE_LAYOUT: RuntimeLayout = RuntimeLayout {
    dimension_map_width: 638,
    bias_width: 1,
    aggregate_width: 15,
    identity_dimensions_width: 22,
    identity_cross_dimension_width: 8,
    signal_width: 0,
    sentinel_slots_width: 504,
    interaction_width: 68,
    competitive_cells_width: 20,
};

fn default_world_builder() -> crate::testing::WorldBuilder {
    World::builder(crate::AssayerConfig {
        instance_id: "test".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    })
    .channel("default", ChannelPolicy::default())
}

fn reference_templates() -> Vec<InteractionTemplate> {
    let mut templates: Vec<_> = (0..5)
        .map(|offset| InteractionTemplate::type1(offset, FeatureSelector::AggregateFeature(0)))
        .collect();
    templates.extend((0..8).map(|offset| InteractionTemplate::type4(offset, FeatureSelector::AggregateFeature(0))));
    templates.push(InteractionTemplate::type5(FeatureSelector::AggregateFeature(0)));
    templates
}

fn reference_cells(entity: &str) -> CompetitiveCellSpec {
    let expected = vec![
        CompetitiveCellId::new(0, 0),
        CompetitiveCellId::new(0, 1),
        CompetitiveCellId::new(0, 2),
        CompetitiveCellId::new(0, 3),
        CompetitiveCellId::new(0, 4),
        CompetitiveCellId::new(0, 5),
        CompetitiveCellId::new((1_u128 << 123) - 1, 5),
        CompetitiveCellId::new((1_u128 << 124) - 1, 4),
        CompetitiveCellId::new((1_u128 << 126) - 1, 2),
        CompetitiveCellId::new((1_u128 << 127) - 1, 1),
    ];
    let mut budget = IdentityBudget::for_depth_cutoff(10);
    budget.max_cells = 19;
    budget.depth_create = 3;
    budget.depth_evict = 4;
    CompetitiveCellSpec::new(expected)
        .budget(budget)
        .observe_n(World::entity(entity), 110)
}

fn cold_construction_baselines() -> (WorldConstructionDeclaration, WorldConstructionBaseline) {
    let world = World::cold("harness-construction-baseline", 0xBA5E_11AE);
    (world.construction_declaration().clone(), world.construction_baseline())
}

/// A scenario's interaction declarations cross the harness boundary into the engine's construction validator, so an invalid aggregate operand is returned as the engine's typed build error instead of being ignored or panicking.
///
/// ´claim:harness:interaction-declarations-reach-the-engine-construction-validator´
/// ´test:crate:world-builder-forwards-invalid-interaction-template´
#[test]
fn world_builder_forwards_invalid_interaction_template() {
    let result = default_world_builder()
        .interaction_templates(vec![InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(15))])
        .seed(1)
        .build();

    assert!(matches!(
        result,
        Err(WorldBuildError::Build(BuildError::InvalidInteractionTemplate {
            template_index: 0,
            ..
        }))
    ));
}

/// The gated identity fixture drives its declared observation through assessment, waits for the identity owner to derive the competitive set and waits for the model owner to publish it. Success therefore means the declared cell is observable in the resulting layout, not merely queued.
///
/// ´claim:harness:a-gated-identity-fixture-returns-only-after-its-declared-cells-are-published´
/// ´test:crate:register-identity-with-cells-observes-the-declared-set´
#[test]
fn register_identity_with_cells_observes_the_declared_set() {
    let mut world = World::cold("harness-cells-observed", 2);
    let root = CompetitiveCellId::new(0, 0);
    let spec = CompetitiveCellSpec::new(vec![root]).observe_n(World::entity("observed"), 1);

    let dimension = world
        .register_identity_with_cells("identity", spec)
        .expect("one observation should publish the root cell");
    let layout = world.runtime_layout().expect("published layout should be readable");

    assert_eq!(world.identity("identity"), Some(dimension));
    assert_eq!(layout.competitive_cells_width, 1);
}

/// A gated identity fixture whose declared set cannot be produced by its declared empty workload is refused with both sets. Registration alone is not mistaken for installation, and a caller cannot continue with a layout that only resembles the fixture it requested.
///
/// ´claim:harness:an-unobservable-declared-cell-set-is-a-fixture-error´
/// ´test:crate:register-identity-with-cells-rejects-an-unobservable-set´
#[test]
fn register_identity_with_cells_rejects_an_unobservable_set() {
    let mut world = World::cold("harness-cells-unobservable", 3);
    let expected = vec![CompetitiveCellId::new(0, 8)];

    let result = world.register_identity_with_cells("identity", CompetitiveCellSpec::new(expected.clone()));

    assert!(matches!(
        result,
        Err(WorldFixtureError::CompetitiveCellsNotObserved {
            expected: error_expected,
            observed,
            ..
        }) if error_expected == expected && observed.is_empty()
    ));
}

/// A builder-side layout declaration is checked against one published snapshot after a barrier. A width mismatch is reported with both complete layouts, making the guard useful to a fixture that must fail before its scenario begins.
///
/// ´claim:harness:a-declared-runtime-layout-is-checked-against-the-published-layout´
/// ´test:crate:runtime-layout-rejects-a-declared-width-mismatch´
#[test]
fn runtime_layout_rejects_a_declared_width_mismatch() {
    let expected = RuntimeLayout {
        dimension_map_width: 17,
        bias_width: 1,
        aggregate_width: 15,
        identity_dimensions_width: 0,
        identity_cross_dimension_width: 0,
        signal_width: 0,
        sentinel_slots_width: 0,
        interaction_width: 0,
        competitive_cells_width: 1,
    };
    let world = default_world_builder()
        .expected_runtime_layout(expected)
        .seed(4)
        .build()
        .expect("scenario should build before the layout guard runs");

    let result = world.runtime_layout();

    assert!(matches!(
        result,
        Err(WorldFixtureError::RuntimeLayoutMismatch {
            expected: error_expected,
            observed,
        }) if *error_expected == expected && observed.dimension_map_width == 16
    ));
}

/// A builder carrying a completed-layout declaration refuses a target whose fixed signal width differs from the measured construction state and reports the complete initial baseline beside both construction and completed-layout expectations.
///
/// Invalid precondition: the target adds an undeclared signal.
///
/// ´claim:harness:a-builder-guard-refusal-carries-the-measured-construction-baseline´
/// ´test:crate:world-builder-guard-rejects-an-unreachable-layout-with-its-measured-baseline´
#[test]
fn world_builder_guard_rejects_an_unreachable_layout_with_its_measured_baseline() {
    let expected_layout = RuntimeLayout {
        dimension_map_width: 17,
        bias_width: 1,
        aggregate_width: 15,
        identity_dimensions_width: 0,
        identity_cross_dimension_width: 0,
        signal_width: 1,
        sentinel_slots_width: 0,
        interaction_width: 0,
        competitive_cells_width: 0,
    };
    let Err(error) = default_world_builder()
        .expected_runtime_layout(expected_layout)
        .seed(0xB011_DE42)
        .build()
    else {
        panic!("a target requiring an undeclared signal must be refused as invalid setup");
    };
    let message = error.to_string();
    let (expected, measured) = match error {
        WorldBuildError::ConstructionBaselineMismatch {
            fixture: ConstructionFixture::WorldBuilder,
            expected,
            measured,
        } => (expected, measured),
        other => panic!("unexpected builder refusal: {other}"),
    };

    assert_eq!(expected.expected_runtime_layout, Some(expected_layout));
    assert_eq!(expected.initial.runtime_layout.signal_width, 0);
    assert_eq!(measured.runtime_layout.signal_width, 0);
    assert_eq!(
        message,
        format!("WorldBuilder::build measured construction baseline {measured:?} does not satisfy declaration {expected:?}")
    );
}

/// The cold-world guard refuses a measured ramp that has already accepted an observation and reports every construction fact beside the cold declaration.
///
/// Invalid precondition: the measured accepted count is non-zero.
///
/// ´claim:harness:a-cold-world-guard-refusal-carries-the-measured-construction-baseline´
/// ´test:crate:cold-guard-rejects-an-advanced-ramp-with-its-measured-baseline´
#[test]
fn cold_guard_rejects_an_advanced_ramp_with_its_measured_baseline() {
    let (expected, mut measured) = cold_construction_baselines();
    measured.standardisation_observations = 1;

    let error = ConstructionFixture::Cold
        .guard_baseline(&expected, measured.clone())
        .expect_err("an advanced ramp must be refused as invalid cold setup");
    let message = error.to_string();

    assert!(matches!(
        error,
        WorldBuildError::ConstructionBaselineMismatch {
            fixture: ConstructionFixture::Cold,
            expected: error_expected,
            measured: error_measured,
        } if *error_expected == expected && *error_measured == measured
    ));
    assert_eq!(
        message,
        format!("World::cold measured construction baseline {measured:?} does not satisfy declaration {expected:?}")
    );
}

/// The ordinary scenario guard refuses a generator state that differs from the declared seed and reports the complete measured construction baseline beside the declaration.
///
/// Invalid precondition: the measured generator state differs from the seed.
///
/// ´claim:harness:a-scenario-guard-refusal-carries-the-measured-construction-baseline´
/// ´test:crate:scenario-guard-rejects-a-changed-rng-state-with-its-measured-baseline´
#[test]
fn scenario_guard_rejects_a_changed_rng_state_with_its_measured_baseline() {
    let (expected, mut measured) = cold_construction_baselines();
    measured.rng_state = measured.rng_state.wrapping_add(1);

    let error = ConstructionFixture::Scenario
        .guard_baseline(&expected, measured.clone())
        .expect_err("a changed generator state must be refused as invalid setup");
    let message = error.to_string();

    assert!(matches!(
        error,
        WorldBuildError::ConstructionBaselineMismatch {
            fixture: ConstructionFixture::Scenario,
            expected: error_expected,
            measured: error_measured,
        } if *error_expected == expected && *error_measured == measured
    ));
    assert_eq!(
        message,
        format!("scenario measured construction baseline {measured:?} does not satisfy declaration {expected:?}")
    );
}

/// The configured-scenario guard refuses a measured channel population missing the declared channel and reports all construction facts beside the declaration.
///
/// Invalid precondition: the measured channel population is empty.
///
/// ´claim:harness:a-configured-scenario-guard-refusal-carries-the-measured-construction-baseline´
/// ´test:crate:scenario-with-guard-rejects-a-missing-channel-with-its-measured-baseline´
#[test]
fn scenario_with_guard_rejects_a_missing_channel_with_its_measured_baseline() {
    let (expected, mut measured) = cold_construction_baselines();
    measured.channels.clear();

    let error = ConstructionFixture::ScenarioWith
        .guard_baseline(&expected, measured.clone())
        .expect_err("a missing channel must be refused as invalid setup");
    let message = error.to_string();

    assert!(matches!(
        error,
        WorldBuildError::ConstructionBaselineMismatch {
            fixture: ConstructionFixture::ScenarioWith,
            expected: error_expected,
            measured: error_measured,
        } if *error_expected == expected && *error_measured == measured
    ));
    assert_eq!(
        message,
        format!("scenario_with measured construction baseline {measured:?} does not satisfy declaration {expected:?}")
    );
}

/// The explicit-config scenario guard refuses a measured engine identifier different from the configured identifier and reports all construction facts beside the declaration.
///
/// Invalid precondition: the measured identifier names another engine.
///
/// ´claim:harness:an-explicit-config-scenario-guard-refusal-carries-the-measured-construction-baseline´
/// ´test:crate:scenario-with-config-guard-rejects-a-changed-instance-with-its-measured-baseline´
#[test]
fn scenario_with_config_guard_rejects_a_changed_instance_with_its_measured_baseline() {
    let (expected, mut measured) = cold_construction_baselines();
    measured.instance_id = "another-engine".to_owned();

    let error = ConstructionFixture::ScenarioWithConfig
        .guard_baseline(&expected, measured.clone())
        .expect_err("a changed engine identifier must be refused as invalid setup");
    let message = error.to_string();

    assert!(matches!(
        error,
        WorldBuildError::ConstructionBaselineMismatch {
            fixture: ConstructionFixture::ScenarioWithConfig,
            expected: error_expected,
            measured: error_measured,
        } if *error_expected == expected && *error_measured == measured
    ));
    assert_eq!(
        message,
        format!("scenario_with_config measured construction baseline {measured:?} does not satisfy declaration {expected:?}")
    );
}

/// The trained-state fixture returns a real world only after its balanced training and held-out class counts match their declarations and its measured endpoint gap and pairwise rank clear their strict setup floors.
///
/// ´claim:harness:the-trained-state-fixture-reports-the-precondition-that-admitted-its-world´
/// ´test:crate:trained-state-fixture-returns-its-measured-precondition´
#[test]
fn trained_state_fixture_returns_its_measured_precondition() {
    let fixture = World::trained_state("harness-trained-state", 5).expect("standard training should establish its precondition");
    let expected = TrainedStateBaseline::expected();

    assert_eq!(
        fixture.baseline.benign_training_class_count,
        expected.benign_training_class_count
    );
    assert_eq!(
        fixture.baseline.adverse_training_class_count,
        expected.adverse_training_class_count
    );
    assert_eq!(
        fixture.baseline.benign_held_out_class_count,
        expected.benign_held_out_class_count
    );
    assert_eq!(
        fixture.baseline.adverse_held_out_class_count,
        expected.adverse_held_out_class_count
    );
    assert!(
        fixture.baseline.held_out_gap > expected.held_out_gap,
        "measured held-out gap: {}",
        fixture.baseline.held_out_gap
    );
    assert!(
        fixture.baseline.pairwise_rank > expected.pairwise_rank,
        "measured pairwise rank: {}",
        fixture.baseline.pairwise_rank
    );
}

/// The trained-state guard refuses a held-out class count below its declaration and its message carries the complete measured baseline beside the expected counts and strict floors, identifying setup as the failed boundary.
///
/// Invalid precondition: the measured benign held-out population is one sample short.
///
/// ´claim:harness:a-trained-state-guard-refusal-carries-the-measured-baseline´
/// ´test:crate:trained-state-guard-rejects-an-invalid-class-count-with-its-measured-baseline´
#[test]
fn trained_state_guard_rejects_an_invalid_class_count_with_its_measured_baseline() {
    let expected = TrainedStateBaseline::expected();
    let measured = TrainedStateBaseline {
        benign_held_out_class_count: expected.benign_held_out_class_count - 1,
        held_out_gap: 0.20,
        pairwise_rank: 0.80,
        ..expected
    };

    let error = crate::testing::TrainedStateFixture::guard_baseline(measured)
        .expect_err("a short held-out class must be refused as invalid setup");
    let message = error.to_string();

    assert!(matches!(
        error,
        crate::testing::TrainedStateFixtureError::BaselineMismatch {
            expected: error_expected,
            measured: error_measured,
        } if error_expected.benign_training_class_count == expected.benign_training_class_count
            && error_expected.adverse_training_class_count == expected.adverse_training_class_count
            && error_expected.benign_held_out_class_count == expected.benign_held_out_class_count
            && error_expected.adverse_held_out_class_count == expected.adverse_held_out_class_count
            && error_expected.held_out_gap.to_bits() == expected.held_out_gap.to_bits()
            && error_expected.pairwise_rank.to_bits() == expected.pairwise_rank.to_bits()
            && error_measured.benign_training_class_count == measured.benign_training_class_count
            && error_measured.adverse_training_class_count == measured.adverse_training_class_count
            && error_measured.benign_held_out_class_count == measured.benign_held_out_class_count
            && error_measured.adverse_held_out_class_count == measured.adverse_held_out_class_count
            && error_measured.held_out_gap.to_bits() == measured.held_out_gap.to_bits()
            && error_measured.pairwise_rank.to_bits() == measured.pairwise_rank.to_bits()
    ));
    assert_eq!(
        message,
        format!("trained-state measured baseline {measured:?} does not satisfy expected counts and strict floors {expected:?}")
    );
}

/// The reference population is constructed only through scenario verbs: one spatial outcome axis, eight Sentinels, two identity dimensions whose declared workloads publish ten competitive cells apiece, and the fourteen interaction templates retained by the builder. Its published block widths match the existing `rebuild_reference_config_p638` witness, including total width 638.
///
/// ´claim:harness:the-public-scenario-path-reproduces-the-reference-runtime-layout´
/// ´test:crate:public-scenario-reproduces-reference-layout-p638´
#[test]
fn public_scenario_reproduces_reference_layout_p638() {
    let mut world = default_world_builder()
        .interaction_templates(reference_templates())
        .expected_runtime_layout(REFERENCE_LAYOUT)
        .seed(5)
        .build()
        .expect("reference scenario should build");

    world.register_axis("outcome", true).expect("reference axis should register");
    for name in ["S1", "S2", "S3", "S4", "S5", "S6", "S7", "S8"] {
        world.register_sentinel(name).expect("reference Sentinel should register");
    }
    world
        .register_identity_with_cells("identity-a", reference_cells("identity-a"))
        .expect("first reference identity should install ten cells");
    world
        .register_identity_with_cells("identity-b", reference_cells("identity-b"))
        .expect("second reference identity should install ten cells");

    let observed = world
        .runtime_layout()
        .expect("reference widths should match their declaration");

    assert_eq!(observed, REFERENCE_LAYOUT);
}

/// One scenario advance moves the persistent and intra-process readings by the same declared interval while their types remain distinct.
///
/// ´claim:harness:one-scenario-advance-moves-both-separated-clock-domains-together´
/// ´test:crate:scenario-advance-moves-both-clock-domains-together´
#[test]
fn scenario_advance_moves_both_clock_domains_together() {
    let world = World::cold("harness-time-domains", 0x71AE);
    let persistent_before = world.clock().now();
    let monotonic_before = world.clock().now_monotonic();
    let delta = Duration::from_secs(7 * 60 * 60);

    world.advance(delta).expect("both time-domain barriers should complete");

    let persistent_after = world.clock().now();
    let monotonic_after = world.clock().now_monotonic();
    assert_eq!(persistent_after.hours_since(&persistent_before).to_bits(), 7.0_f64.to_bits());
    assert_eq!(monotonic_after.duration_since(monotonic_before), delta);
}

/// An entry that becomes collectable only after scenario time advances is absent when the verb returns, proving completion through the Ledger barrier's own state transition rather than elapsed time.
///
/// ´claim:harness:a-scenario-advance-returns-after-the-ledger-cycle-it-makes-due´
/// ´test:crate:scenario-advance-completes-the-ledger-gc-cycle-it-makes-due´
#[test]
fn scenario_advance_completes_the_ledger_gc_cycle_it_makes_due() {
    let world = World::cold("harness-time-ledger-barrier", 0x71AE_6C65_6467_6572);
    let ledger = &world.assayer().outcome_ledger;
    let sentinel = SentinelId(1);
    let key = LedgerKey::new(1, 128);
    ledger.create_sentinel(sentinel, PersistentTimestamp::new(1_700_000_000, 0));
    {
        let sentinel_ledger = ledger.get(sentinel).expect("the scenario Ledger contains its Sentinel");
        let mut entries = sentinel_ledger.write().expect("the scenario owns the Ledger setup");
        let mut entry = LedgerEntry::new_neutral(PersistentTimestamp::new(1_700_000_000, 0));
        entry.last_updated = world.clock().now();
        entries.insert(key, entry);
    }

    world
        .advance(Duration::from_secs(61 * 24 * 60 * 60))
        .expect("time advance should complete the newly due collection cycle");

    let sentinel_ledger = ledger.get(sentinel).expect("the Sentinel root survives collection");
    let entries = sentinel_ledger.read().expect("the collected Ledger remains readable");
    assert!(entries.get(&key).is_none(), "the time-eligible neutral entry was collected");
    assert!(entries.has_root(), "the Ledger root remains outside collection");
}

/// Targeted scenario travel places the persistent reading at the requested instant and moves the intra-process reading by the same offset.
///
/// ´claim:harness:targeted-scenario-travel-moves-both-separated-clock-domains-together´
/// ´test:crate:scenario-travel-targets-both-clock-domains-together´
#[test]
fn scenario_travel_targets_both_clock_domains_together() {
    let world = World::cold("harness-time-target", 0x71AE_7A76);
    let persistent_before = world.clock().now();
    let monotonic_before = world.clock().now_monotonic();
    let delta = Duration::from_secs(9 * 60 * 60);
    let target = UNIX_EPOCH + Duration::from_secs(crate::testing::VirtualClock::EPOCH_SECS) + delta;

    world
        .travel_to(target)
        .expect("the forward target and its barriers should succeed");

    assert_eq!(world.clock().now(), PersistentTimestamp::from_system_time(target));
    assert_eq!(
        world.clock().now().hours_since(&persistent_before).to_bits(),
        9.0_f64.to_bits()
    );
    assert_eq!(world.clock().now_monotonic().duration_since(monotonic_before), delta);
}

/// A backward target is refused without moving time, and its diagnostic names the current and requested persistent instants.
///
/// Invalid precondition: the target is one hour before the scenario clock.
///
/// ´claim:harness:backward-scenario-travel-is-a-diagnostic-refusal-not-a-clock-mutation´
/// ´test:crate:scenario-travel-refuses-a-backward-target-with-both-instants´
#[test]
fn scenario_travel_refuses_a_backward_target_with_both_instants() {
    let world = World::cold("harness-time-backward", 0x71AE_BAC0);
    let current = world.clock().now();
    let monotonic_current = world.clock().now_monotonic();
    let target = UNIX_EPOCH + Duration::from_secs(crate::testing::VirtualClock::EPOCH_SECS) - Duration::from_secs(60 * 60);
    let requested = PersistentTimestamp::from_system_time(target);

    let error = world
        .travel_to(target)
        .expect_err("a target before the current instant must be refused");

    assert!(matches!(
        error,
        WorldFixtureError::BackwardTimeTravel {
            current: error_current,
            requested: error_requested,
        } if error_current == current && error_requested == requested
    ));
    assert_eq!(
        error.to_string(),
        format!("scenario time cannot move backward: current {current:?}, requested {requested:?}")
    );
    assert_eq!(world.clock().now(), current, "a refused target leaves both domains unmoved");
    assert_eq!(
        world.clock().now_monotonic(),
        monotonic_current,
        "a refused target leaves the intra-process domain unmoved"
    );
}
