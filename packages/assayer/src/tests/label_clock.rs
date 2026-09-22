// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Engine-clock witnesses for cold construction and isolated checkpoint restore.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`persistence_fork_keeps_the_label_decay_clock`] | persistence | An isolated checkpoint copy restored at the same scenario instant must run the next label with the same elapsed decay as the uninterrupted arm. The prefix already advanced and labelled at that instant, so a second application of its elapsed interval changes durable precision. # Panics Panics if construction, a barrier, durable I/O, or the equality witness fails. |
//! | [`cold_start_keeps_the_label_decay_clock`] | persistence | Advancing the injected clock before building a cold World contributes no elapsed decay to its first label. A clock created directly at that same persistent instant is the control: both worlds start from the same priors and process the same label without advancing either clock after construction. # Panics Panics if construction, label processing, durable I/O, or the equality witness fails. |
//! | [`numeric_revert_keeps_the_label_decay_clock`] | persistence | A numeric-error revert rebuilds from the published snapshot at the engine's current monotonic reading. After both arms have processed the same advanced-time prefix, forcing one arm through CP5 must leave its next valid label with the same elapsed decay as the arm that did not revert. # Panics Panics if construction, registration, a barrier, the CP5 stimulus, durable I/O, or the equality witness fails. |

use std::collections::HashMap;
use std::time::Duration;

use crate::config::types::AssayerConfig;
use crate::health::DegradationContext;
use crate::owner::commands::{LabelContext, PendingAssessment, SequencedLabel};
use crate::pending::PendingRiskBasis;
use crate::persistence::checkpoint;
use crate::testing::{Clock, LabelSpec, VirtualClock, World};
use crate::types::AssessmentId;

fn test_dir() -> tempfile::TempDir {
    tempfile::tempdir().expect("failed to create temp dir")
}

fn dummy_label(assessment_id: AssessmentId, seq: Option<u64>, clock: &dyn Clock) -> SequencedLabel {
    SequencedLabel {
        seq,
        label: LabelSpec::new(assessment_id).valence(1.0).ground_truth().build(),
        context: LabelContext::Full(Box::new(PendingAssessment {
            spatial_axis_ids: Vec::new(),
            id: assessment_id,
            timestamp: clock.now_monotonic(),
            persistent_timestamp: clock.now(),
            entity: World::entity(&format!("pending-assessment-{}", assessment_id.0)),
            sentinel_extractions: HashMap::new(),
            identity_coordinates: HashMap::new(),
            identity_active_cells: HashMap::new(),
            active_sentinels: Vec::new(),
            reporting_sentinels: Vec::new(),
            entity_base_features: HashMap::new(),
            entity_axis_features: HashMap::new(),
            signal_features: crate::pending::StoredFeatures::default(),
            risk_basis: PendingRiskBasis::default(),
            outcome_predictions: HashMap::new(),
            degradation: DegradationContext::default(),
            report_origin: None,
        })),
        arrived_at: None,
    }
}

/// An isolated checkpoint copy restored at the same scenario instant must run the next label with the same elapsed decay as the uninterrupted arm. The prefix already advanced and labelled at that instant, so a second application of its elapsed interval changes durable precision.
///
/// # Panics
///
/// Panics if construction, a barrier, durable I/O, or the equality witness fails.
///
/// ´claim:persistence:an-isolated-fork-does-not-repeat-pre-checkpoint-label-decay´
/// ´test:crate:persistence-fork-keeps-the-label-decay-clock´
#[test]
fn persistence_fork_keeps_the_label_decay_clock() {
    use std::path::Path;
    use std::sync::Arc;
    use std::time::Instant;

    let witness_started = Instant::now();
    let original_root = test_dir();
    let clock = Arc::new(VirtualClock::epoch());
    let build = |root: &Path| {
        let mut config = AssayerConfig {
            instance_id: "fork-decay-clock".to_owned(),
            infrastructure: crate::testing::test_infrastructure(),
            ..AssayerConfig::default()
        };
        config.temporal.gamma_t_core = 0.9;
        World::builder(config)
            .seed(0xF04C)
            .clock(Arc::clone(&clock))
            .persistence_dirs(root, root)
            .build()
            .expect("persistent world builds")
    };
    let uninterrupted = build(original_root.path());
    uninterrupted
        .advance(Duration::from_secs(24 * 3600))
        .expect("advance settles");
    uninterrupted
        .assayer()
        .label_tx()
        .send(dummy_label(AssessmentId(1), Some(1), clock.as_ref()))
        .unwrap();
    uninterrupted.flush_observations().expect("observations settle");
    uninterrupted.flush_identity_maintenance().expect("identity work settles");
    uninterrupted
        .flush_checkpoint_scheduler()
        .expect("checkpoint trigger settles");
    uninterrupted.flush_labels().expect("prefix checkpoint completes");

    let captured = crate::testing::durable_checkpoint(&uninterrupted).expect("read prefix checkpoint");
    assert_eq!(captured.checkpoint_timestamp, clock.now(), "restore has no downtime");
    let fork = uninterrupted.fork_persistence().expect("isolated label-clock fork");
    let (uninterrupted, restored) = fork.arms();
    let fork_wall = fork.wall();
    assert!(Arc::ptr_eq(uninterrupted.clock(), restored.clock()), "one injected clock");
    assert_ne!(fork.roots()[0], fork.roots()[1], "isolated roots");

    let suffix_started = Instant::now();
    for world in [uninterrupted, restored] {
        world
            .assayer()
            .label_tx()
            .send(dummy_label(AssessmentId(2), Some(2), clock.as_ref()))
            .unwrap();
        world.flush_labels().expect("suffix label and checkpoint complete");
    }
    let suffix_wall = suffix_started.elapsed();
    let left = crate::testing::durable_checkpoint(uninterrupted).expect("read uninterrupted checkpoint");
    let right = crate::testing::durable_checkpoint(restored).expect("read restored checkpoint");
    assert_eq!(left.owner_state.label_count, 2, "uninterrupted suffix ran");
    assert_eq!(right.owner_state.label_count, 2, "restored suffix ran");
    eprintln!(
        "fork_wall={fork_wall:?} suffix_wall={suffix_wall:?} witness_wall={:?}",
        witness_started.elapsed()
    );
    for index in 0..left.operational.precision.dim() {
        let uninterrupted_value = left.operational.precision.as_inner()[(index, index)];
        let restored_value = right.operational.precision.as_inner()[(index, index)];
        assert!(
            (uninterrupted_value - restored_value).abs() < 1e-10,
            "operational.precision[{index},{index}]: uninterrupted={uninterrupted_value:?}, restored={restored_value:?}; restore repeated time before the checkpoint"
        );
    }
}

/// Advancing the injected clock before building a cold World contributes no elapsed decay to its first label. A clock created directly at that same persistent instant is the control: both worlds start from the same priors and process the same label without advancing either clock after construction.
///
/// # Panics
///
/// Panics if construction, label processing, durable I/O, or the equality witness fails.
///
/// ´claim:persistence:cold-construction-excludes-time-before-the-working-copy-existed´
/// ´test:crate:cold-start-keeps-the-label-decay-clock´
#[test]
fn cold_start_keeps_the_label_decay_clock() {
    use std::sync::Arc;

    let advanced = Arc::new(VirtualClock::epoch());
    advanced.advance(Duration::from_secs(24 * 3600));
    let fresh = Arc::new(VirtualClock::at_secs(VirtualClock::EPOCH_SECS + 24 * 3600));
    assert_eq!(advanced.now(), fresh.now(), "one persistent present");
    let build = |clock: Arc<VirtualClock>, root: &std::path::Path| {
        let mut config = AssayerConfig {
            instance_id: "cold-decay-clock".to_owned(),
            infrastructure: crate::testing::test_infrastructure(),
            ..AssayerConfig::default()
        };
        config.temporal.gamma_t_core = 0.9;
        World::builder(config)
            .seed(0xF04C)
            .clock(clock)
            .persistence_dirs(root, root)
            .build()
            .expect("cold world builds")
    };
    let advanced_root = test_dir();
    let fresh_root = test_dir();
    let advanced_world = build(Arc::clone(&advanced), advanced_root.path());
    let fresh_world = build(Arc::clone(&fresh), fresh_root.path());
    for world in [&advanced_world, &fresh_world] {
        world
            .assayer()
            .label_tx()
            .send(dummy_label(AssessmentId(1), Some(1), world.clock().as_ref()))
            .unwrap();
        world.flush_labels().expect("first label and checkpoint complete");
    }
    let left = checkpoint::read_checkpoint(&advanced_root.path().join("checkpoint.bin")).expect("read advanced-clock checkpoint");
    let right = checkpoint::read_checkpoint(&fresh_root.path().join("checkpoint.bin")).expect("read fresh-clock checkpoint");
    assert_eq!(left.owner_state.label_count, 1, "advanced-clock first label ran");
    assert_eq!(right.owner_state.label_count, 1, "fresh-clock first label ran");
    for index in 0..left.operational.precision.dim() {
        let advanced_value = left.operational.precision.as_inner()[(index, index)];
        let fresh_value = right.operational.precision.as_inner()[(index, index)];
        assert!(
            (advanced_value - fresh_value).abs() < 1e-10,
            "operational.precision[{index},{index}]: advanced={advanced_value:?}, fresh={fresh_value:?}; first label charged time before construction"
        );
    }
}

/// A numeric-error revert rebuilds from the published snapshot at the engine's current monotonic reading. After both arms have processed the same advanced-time prefix, forcing one arm through CP5 must leave its next valid label with the same elapsed decay as the arm that did not revert.
///
/// # Panics
///
/// Panics if construction, registration, a barrier, the CP5 stimulus, durable I/O, or the equality witness fails.
///
/// ´claim:persistence:a-numeric-revert-does-not-repeat-the-label-clock-offset´
/// ´test:crate:numeric-revert-keeps-the-label-decay-clock´
#[test]
fn numeric_revert_keeps_the_label_decay_clock() {
    use std::sync::Arc;

    use crate::testing::registrations::spatial_axis_reg;
    use crate::types::OutcomeAxisId;

    let clock = Arc::new(VirtualClock::epoch());
    let reverted_root = test_dir();
    let control_root = test_dir();
    let build = |root: &std::path::Path| {
        let mut config = AssayerConfig {
            instance_id: "revert-decay-clock".to_owned(),
            infrastructure: crate::testing::test_infrastructure(),
            ..AssayerConfig::default()
        };
        config.temporal.gamma_t_core = 0.9;
        World::builder(config)
            .seed(0xF04C)
            .clock(Arc::clone(&clock))
            .persistence_dirs(root, root)
            .build()
            .expect("revert world builds")
    };
    let reverted = build(reverted_root.path());
    let control = build(control_root.path());
    let axis = OutcomeAxisId(1);
    for world in [&reverted, &control] {
        world
            .assayer()
            .register_outcome_axis(spatial_axis_reg(axis))
            .expect("axis registers");
        world.flush_labels().expect("registration publishes");
        world
            .assayer()
            .label_tx()
            .send(dummy_label(AssessmentId(1), Some(1), clock.as_ref()))
            .unwrap();
        world.flush_labels().expect("initial label publishes");
    }
    reverted.advance(Duration::from_secs(24 * 3600)).expect("advance settles");
    for world in [&reverted, &control] {
        world
            .assayer()
            .label_tx()
            .send(dummy_label(AssessmentId(2), Some(2), clock.as_ref()))
            .unwrap();
        world.flush_labels().expect("advanced-time prefix publishes");
    }

    // The crate-level channel admits a numeric fault past public sanitisation.
    // A non-finite axis target corrupts its mean and forces the actual CP5
    // reconstruction; no working-copy constructor is called by this witness.
    let mut corrupt = dummy_label(AssessmentId(3), Some(3), clock.as_ref());
    corrupt.label.outcomes.insert(axis, f64::NAN);
    reverted.assayer().label_tx().send(corrupt).unwrap();
    reverted.flush_labels().expect("numeric-error revert completes");
    assert_eq!(
        reverted.assayer().shared().health.load().cp5_reverts,
        1,
        "the snapshot constructor ran through CP5"
    );
    assert!(
        reverted.assayer().shared().label_path_stop.load().is_none(),
        "the rebuild succeeded"
    );

    for world in [&reverted, &control] {
        world
            .assayer()
            .label_tx()
            .send(dummy_label(AssessmentId(4), Some(4), clock.as_ref()))
            .unwrap();
        world.flush_labels().expect("post-revert suffix publishes");
    }
    let left = checkpoint::read_checkpoint(&reverted_root.path().join("checkpoint.bin")).expect("read reverted checkpoint");
    let right = checkpoint::read_checkpoint(&control_root.path().join("checkpoint.bin")).expect("read control checkpoint");
    for index in 0..left.operational.precision.dim() {
        let reverted_value = left.operational.precision.as_inner()[(index, index)];
        let control_value = right.operational.precision.as_inner()[(index, index)];
        assert!(
            (reverted_value - control_value).abs() < 1e-10,
            "operational.precision[{index},{index}]: reverted={reverted_value:?}, control={control_value:?}; numeric revert repeated the injected clock offset"
        );
    }
}
