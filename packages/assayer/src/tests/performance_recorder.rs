// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`label_update_recorder_rejects_a_second_activation`] | performance | Activation refuses an already-active sample and reports that invalid precondition. Invalid precondition: a sample is active when activation is requested again. |
//! | [`label_update_recorder_rejects_an_inactive_drain`] | performance | Drain refuses an inactive recorder and reports that invalid precondition. Invalid precondition: no sample has been activated. |
//! | [`label_update_recorder_rejects_an_incomplete_publication`] | performance | Drain refuses an active sample until at least one publication completes the timed region. Invalid precondition: the sample is active but no label has completed publication. |
//! | [`label_update_recorder_returns_only_a_completed_dense_publication_duration`] | performance | The scenario surface returns a positive duration only after the real dense label publication crosses the joined model-update region. |

//! The performance recorder is reached only through [`World`], measures the
//! existing dense update block, and never mutates product state.

use std::time::Duration;

use crate::testing::performance::{PerformanceCase, SeededLabelPopulation};
use crate::testing::{World, WorldFixtureError};

/// Activation refuses an already-active sample and reports that invalid precondition.
///
/// Invalid precondition: a sample is active when activation is requested again.
///
/// ´claim:performance:the-label-update-recorder-has-at-most-one-active-sample´
/// ´test:crate:label-update-recorder-rejects-a-second-activation´
#[test]
fn label_update_recorder_rejects_a_second_activation() {
    let world = World::cold("performance-recorder-double-activation", 31);
    world
        .activate_label_update_recorder()
        .expect("first activation should start an empty sample");

    let error = world
        .activate_label_update_recorder()
        .expect_err("a second activation must be refused");

    assert!(matches!(error, WorldFixtureError::LabelUpdateRecorderAlreadyActive));
}

/// Drain refuses an inactive recorder and reports that invalid precondition.
///
/// Invalid precondition: no sample has been activated.
///
/// ´claim:performance:the-label-update-recorder-drains-only-an-active-sample´
/// ´test:crate:label-update-recorder-rejects-an-inactive-drain´
#[test]
fn label_update_recorder_rejects_an_inactive_drain() {
    let world = World::cold("performance-recorder-inactive", 32);

    let error = world
        .drain_label_update_duration()
        .expect_err("an inactive recorder must be refused");

    assert!(matches!(error, WorldFixtureError::LabelUpdateRecorderInactive));
}

/// Drain refuses an active sample until at least one publication completes the timed region.
///
/// Invalid precondition: the sample is active but no label has completed publication.
///
/// ´claim:performance:the-label-update-recorder-requires-a-completed-publication´
/// ´test:crate:label-update-recorder-rejects-an-incomplete-publication´
#[test]
fn label_update_recorder_rejects_an_incomplete_publication() {
    let world = World::cold("performance-recorder-incomplete", 33);
    world
        .activate_label_update_recorder()
        .expect("activation should start an empty sample");

    let error = world
        .drain_label_update_duration()
        .expect_err("a sample without a completed publication must be refused");

    assert!(matches!(error, WorldFixtureError::LabelUpdateRecorderIncomplete));
}

/// The scenario surface returns a positive duration only after the real dense label publication crosses the joined model-update region.
///
/// ´claim:performance:the-label-update-recorder-times-the-real-dense-model-update-region´
/// ´test:crate:label-update-recorder-returns-only-a-completed-dense-publication-duration´
#[test]
fn label_update_recorder_returns_only_a_completed_dense_publication_duration() {
    let world = PerformanceCase::DenseLabelProfile
        .world("performance-recorder-dense-publication")
        .expect("dense performance world should establish its guarded preconditions");
    let population = SeededLabelPopulation::for_case(PerformanceCase::DenseLabelProfile)
        .expect("dense label population should match its declaration");
    world
        .activate_label_update_recorder()
        .expect("activation should start an empty sample");

    population
        .publish_measured(&world)
        .expect("measured dense labels should publish through the real engine");
    let elapsed = world
        .drain_label_update_duration()
        .expect("completed dense publications should yield one duration");

    assert!(elapsed > Duration::ZERO);
    assert!(matches!(
        world.drain_label_update_duration(),
        Err(WorldFixtureError::LabelUpdateRecorderInactive)
    ));
}
