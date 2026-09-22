// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Durable forks exercise the same typed suffix under one injected clock.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`persistence_fork_common_suffix`] | persistence | Identical typed labels and cold-ramp observations leave both isolated arms equal in every declared durable field. # Panics Panics if durable construction, playback, projection or the equality contract fails. |
//! | [`persistence_fork_decay_once`] | persistence | A copied checkpoint precedes one shared-clock advance; restore ages precision once, then both arms cross another checkpoint boundary and play one suffix. An independently double-decayed payload must differ by precision. # Panics Panics if the numeric decay factor, complete comparison or double-decay rejection fails. |
//! | [`persistence_fork_journals_are_isolated`] | persistence | Each arm's journal contains its own suffix record and no record written by the other; either checkpoint truncates only its own journal. The restored World owns its root even after leaving the fork wrapper. # Panics Panics if an append or truncation changes foreign bytes, records are indistinguishable, or root ownership is lost. |
//! | [`persistence_fork_comparator_names_field`] | persistence | A single altered durable scalar is reported by its field name and both values; a newly declared but unprojected restore field is an explicit refusal. # Panics Panics if the comparator accepts the perturbation, loses the field name or values, or silently omits a declared field. |

use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::config::types::AssayerConfig;
use crate::testing::{
    DurableProjection, LabelSpec, PlaybackBarrier, PlaybackBarrierPolicy, PlaybackBatchSize, PlaybackProgress, PlaybackRow,
    World, durable_checkpoint, playback,
};

struct LabelRow(&'static str, f64);

impl PlaybackRow for LabelRow {
    type Error = String;

    fn name(&self) -> &str {
        self.0
    }

    fn play(&self, world: &World) -> Result<(), Self::Error> {
        let assessment = world.assess(crate::RequestContext::new(World::entity(self.0)));
        world
            .label(LabelSpec::new(assessment.id).valence(self.1).ground_truth().build())
            .map(drop)
            .map_err(|error| error.to_string())
    }
}

fn play(world: &World, rows: impl IntoIterator<Item = LabelRow>, settle: bool) {
    let barriers = if settle {
        PlaybackBarrierPolicy::new([PlaybackBarrier::FlushObservations, PlaybackBarrier::FlushLabels])
    } else {
        PlaybackBarrierPolicy::none()
    };
    playback(
        world,
        rows,
        &barriers,
        &mut [],
        &PlaybackProgress::new(),
        PlaybackBatchSize::default(),
    )
    .expect("declared persistence rows complete");
}

fn build(root: &std::path::Path) -> World {
    let mut config = AssayerConfig {
        instance_id: "persistence-fork".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..AssayerConfig::default()
    };
    config.temporal.gamma_t_core = 0.9;
    World::builder(config)
        .seed(0xF04C)
        .persistence_dirs(root, root)
        .build()
        .expect("persistent World builds")
}

fn projection(world: &World) -> DurableProjection {
    DurableProjection::capture(world, &super::persistence::restore_validation_fields()).expect("complete projection")
}

/// Identical typed labels and cold-ramp observations leave both isolated arms equal in every declared durable field.
///
/// # Panics
/// Panics if durable construction, playback, projection or the equality contract fails.
///
/// ´claim:persistence:fork-arms-equal-after-a-common-suffix´
/// ´test:crate:persistence-fork-common-suffix´
#[test]
fn persistence_fork_common_suffix() {
    let started = Instant::now();
    let root = tempfile::tempdir().unwrap();
    let original = build(root.path());
    play(
        &original,
        [LabelRow("prefix-benign", 0.0), LabelRow("prefix-adverse", 1.0)],
        true,
    );
    let fork = original.fork_persistence().expect("isolated fork");
    let (left, right) = fork.arms();
    assert!(Arc::ptr_eq(left.clock(), right.clock()), "fork shares the injected clock");
    assert_ne!(fork.roots()[0], fork.roots()[1], "fork records isolated roots");
    let suffix = Instant::now();
    for world in [left, right] {
        play(world, [LabelRow("suffix-adverse", 1.0), LabelRow("suffix-benign", 0.0)], true);
    }
    let suffix_wall = suffix.elapsed();
    let result = projection(left).compare(&projection(right));
    eprintln!(
        "common_suffix durable_comparison={result:?} fork_wall={:?} suffix_wall={suffix_wall:?} witness_wall={:?}",
        fork.wall(),
        started.elapsed()
    );
    result.expect("complete common-suffix durable equality");
    assert_eq!(durable_checkpoint(right).unwrap().owner_state.label_count, 4);
}

/// A copied checkpoint precedes one shared-clock advance; restore ages precision once, then both arms cross another checkpoint boundary and play one suffix. An independently double-decayed payload must differ by precision.
///
/// # Panics
/// Panics if the numeric decay factor, complete comparison or double-decay rejection fails.
///
/// ´claim:persistence:fork-restore-applies-elapsed-decay-exactly-once´
/// ´test:crate:persistence-fork-decay-once´
#[test]
fn persistence_fork_decay_once() {
    let started = Instant::now();
    let root = tempfile::tempdir().unwrap();
    let original = build(root.path());
    play(&original, [LabelRow("decay-prefix", 1.0)], true);
    let before = durable_checkpoint(&original).unwrap();
    let fork = original
        .fork_persistence_after(Duration::from_secs(3600), |_| Ok(()))
        .expect("elapsed fork");
    let (left, right) = fork.arms();
    let mut restored = durable_checkpoint(right).unwrap();
    let factor = crate::numerics::decay_factor(0.9, 1.0);
    for index in 0..before.operational.precision.dim() {
        let expected = before.operational.precision.as_inner()[(index, index)] * factor;
        let actual = restored.operational.precision.as_inner()[(index, index)];
        assert!(
            (actual - expected).abs() < 1e-12,
            "operational.precision[{index},{index}]: once={expected}, restored={actual}"
        );
    }
    let fields = super::persistence::restore_validation_fields();
    let once = DurableProjection::from_payload(&restored, &fields).unwrap();
    restored.operational.precision.scale(factor);
    let twice = DurableProjection::from_payload(&restored, &fields).unwrap();
    let rejection = once.compare(&twice).expect_err("double decay must be rejected");
    assert!(rejection.starts_with("operational.precision"), "{rejection}");
    let suffix = Instant::now();
    fork.advance(Duration::from_secs(3600))
        .expect("common clock crosses checkpoint boundary");
    for world in [left, right] {
        play(world, [LabelRow("decay-suffix", 0.0)], true);
    }
    let suffix_wall = suffix.elapsed();
    let result = projection(left).compare(&projection(right));
    eprintln!(
        "decay_once durable_comparison={result:?} rejection={rejection}; fork_wall={:?} suffix_wall={suffix_wall:?} witness_wall={:?}",
        fork.wall(),
        started.elapsed()
    );
    result.expect("one decay along either durable path");
}

/// Each arm's journal contains its own suffix record and no record written by the other; either checkpoint truncates only its own journal. The restored World owns its root even after leaving the fork wrapper.
///
/// # Panics
/// Panics if an append or truncation changes foreign bytes, records are indistinguishable, or root ownership is lost.
///
/// ´claim:persistence:fork-journals-never-consume-or-extend-each-other´
/// ´test:crate:persistence-fork-journals-are-isolated´
#[test]
fn persistence_fork_journals_are_isolated() {
    let started = Instant::now();
    let root = tempfile::tempdir().unwrap();
    let original = build(root.path());
    play(&original, [LabelRow("isolation-prefix", 1.0)], true);
    let fork = original.fork_persistence().expect("isolated fork");
    let owned_root = fork.roots()[1].0.parent().unwrap().to_path_buf();
    let empty = fork.journal_bytes().unwrap();
    let (left, right) = fork.arms();
    let suffix = Instant::now();
    play(left, [LabelRow("isolation-suffix", 0.0)], false);
    let first = fork.journal_bytes().unwrap();
    assert_eq!(first[1], empty[1], "uninterrupted append cannot extend restored journal");
    assert!(first[0].len() > empty[0].len(), "uninterrupted suffix was journalled");
    play(right, [LabelRow("isolation-suffix", 0.0)], false);
    let both = fork.journal_bytes().unwrap();
    assert_eq!(both[0], first[0], "restored append cannot extend uninterrupted journal");
    assert!(both[1].len() > empty[1].len(), "restored suffix was journalled");
    let left_record = &both[0][empty[0].len()..];
    let right_record = &both[1][empty[1].len()..];
    // Assessment identifiers are process-local, so the same semantic suffix has distinguishable journal records after a nonempty prefix.
    assert_ne!(left_record, right_record, "byte witnesses must distinguish the writers");
    assert!(
        !both[1].windows(left_record.len()).any(|bytes| bytes == left_record),
        "restored journal contains a foreign record"
    );
    assert!(
        !both[0].windows(right_record.len()).any(|bytes| bytes == right_record),
        "uninterrupted journal contains a foreign record"
    );
    left.flush_labels().unwrap();
    let left_checkpointed = fork.journal_bytes().unwrap();
    assert_eq!(
        left_checkpointed[1], both[1],
        "uninterrupted checkpoint cannot consume restored journal"
    );
    play(left, [LabelRow("isolation-next", 1.0)], false);
    let left_appended = fork.journal_bytes().unwrap();
    right.flush_labels().unwrap();
    assert_eq!(
        fork.journal_bytes().unwrap()[0],
        left_appended[0],
        "restored checkpoint cannot consume uninterrupted journal"
    );
    eprintln!(
        "journal_isolation fork_wall={:?} suffix_wall={:?} witness_wall={:?}",
        fork.wall(),
        suffix.elapsed(),
        started.elapsed()
    );
    let (left, right) = fork.into_arms();
    assert!(owned_root.exists(), "restored World retains the root");
    drop(left);
    drop(right);
    assert!(!owned_root.exists(), "restored World removes its owned root");
}

/// A single altered durable scalar is reported by its field name and both values; a newly declared but unprojected restore field is an explicit refusal.
///
/// # Panics
/// Panics if the comparator accepts the perturbation, loses the field name or values, or silently omits a declared field.
///
/// ´claim:persistence:durable-comparison-names-the-first-differing-field´
/// ´test:crate:persistence-fork-comparator-names-field´
#[test]
fn persistence_fork_comparator_names_field() {
    let started = Instant::now();
    let root = tempfile::tempdir().unwrap();
    let world = build(root.path());
    let fork = world.fork_persistence().expect("comparator fork");
    let mut payload = durable_checkpoint(fork.arms().1).unwrap();
    let mut fields = super::persistence::restore_validation_fields();
    let before = DurableProjection::from_payload(&payload, &fields).unwrap();
    let original = payload.p_positive_global;
    payload.p_positive_global = 0.125;
    let after = DurableProjection::from_payload(&payload, &fields).unwrap();
    let rejection = before.compare(&after).expect_err("perturbed durable field must differ");
    assert_eq!(
        rejection,
        format!("p_positive_global: uninterrupted={original:?}, restored=0.125")
    );
    fields.insert("future_restore_field".to_owned());
    assert!(
        DurableProjection::from_payload(&payload, &fields)
            .unwrap_err()
            .to_string()
            .contains("schema mismatch")
    );
    eprintln!(
        "comparator rejection={rejection}; fork_wall={:?} witness_wall={:?}",
        fork.wall(),
        started.elapsed()
    );
}
