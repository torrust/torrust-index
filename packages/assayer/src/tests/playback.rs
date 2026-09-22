// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`shared_runner_accepts_unrelated_subject_row_types`] | harness | The generic runner accepts two unrelated caller-owned row types, while a source recognizer proves that its own module implements no subject row. |
//! | [`selected_label_barrier_completes_before_checkpoint`] | harness | cites (´claim:harness:playback-checkpoints-follow-every-selected-barrier´) |
//! | [`checkpoints_observe_each_declared_row_once`] | harness | cites (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´) |
//! | [`progress_advances_once_after_each_complete_boundary`] | harness | Progress counts each completed row once and reports finished only after the final checkpoint succeeds. |

use std::convert::Infallible;
use std::fs;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};

use crate::testing::{
    LabelSpec, PlaybackBarrier, PlaybackBarrierPolicy, PlaybackBatchSize, PlaybackCheckpoint, PlaybackProgress, PlaybackRow,
    World, manifest_dir, playback,
};

struct AlphaRow;

impl PlaybackRow for AlphaRow {
    type Error = Infallible;

    fn name(&self) -> &'static str {
        "alpha-row"
    }

    fn play(&self, _world: &World) -> Result<(), Self::Error> {
        Ok(())
    }
}

struct BetaRow;

impl PlaybackRow for BetaRow {
    type Error = Infallible;

    fn name(&self) -> &'static str {
        "beta-row"
    }

    fn play(&self, _world: &World) -> Result<(), Self::Error> {
        Ok(())
    }
}

fn run_rows<R: PlaybackRow>(world: &World, rows: impl IntoIterator<Item = R>) {
    playback(
        world,
        rows,
        &PlaybackBarrierPolicy::none(),
        &mut [],
        &PlaybackProgress::new(),
        PlaybackBatchSize::default(),
    )
    .expect("compile-shape fixture rows should play");
}

/// The generic runner accepts two unrelated caller-owned row types, while a source recognizer proves that its own module implements no subject row.
///
/// ´claim:harness:playback-rows-are-subject-owned-typed-values´
/// ´test:crate:shared-runner-accepts-unrelated-subject-row-types´
#[test]
fn shared_runner_accepts_unrelated_subject_row_types() {
    let world = World::cold("playback-typed-rows", 0xA11C_E001);
    run_rows(&world, [AlphaRow]);
    run_rows(&world, [BetaRow]);

    let source = fs::read_to_string(manifest_dir().join("src/testing/playback.rs"))
        .expect("playback source should be readable by its ownership recognizer");
    let runner_source = source
        .split("#[cfg(test)]")
        .next()
        .expect("playback module should have source before its tests");
    let implementation_marker = ["impl Playback", "Row for"].concat();
    assert!(
        !runner_source.lines().any(|line| line.contains(&implementation_marker)),
        "the shared runner must not own a subject-specific PlaybackRow implementation"
    );
}

struct LabelRow;

impl PlaybackRow for LabelRow {
    type Error = String;

    fn name(&self) -> &'static str {
        "published-label-row"
    }

    fn play(&self, world: &World) -> Result<(), Self::Error> {
        let reckoning = world
            .derive_on("default", "alice")
            .map_err(|error| format!("assessment failed: {error}"))?;
        world
            .label(LabelSpec::adverse(reckoning.assessment.id).build())
            .map(drop)
            .map_err(|error| format!("label failed: {error}"))
    }
}

/// The selected label barrier publishes a row before its checkpoint reads the label count.
///
/// (´claim:harness:playback-checkpoints-follow-every-selected-barrier´)
/// ´test:crate:selected-label-barrier-completes-before-checkpoint´
#[test]
fn selected_label_barrier_completes_before_checkpoint() {
    let world = World::cold("playback-label-barrier", 0xA11C_E002);
    let mut checkpoints = [PlaybackCheckpoint::after_row("published label", 0, |world, location| {
        let observed = world.construction_baseline().total_labels;
        if observed == 1 {
            Ok(())
        } else {
            Err(format!("{location} checkpoint observed {observed} labels before publication"))
        }
    })];

    playback(
        &world,
        [LabelRow],
        &PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels]),
        &mut checkpoints,
        &PlaybackProgress::new(),
        PlaybackBatchSize::default(),
    )
    .expect("the selected label barrier should complete before the checkpoint");
}

struct CountingRow<'a> {
    name: &'static str,
    applied: &'a AtomicUsize,
}

impl PlaybackRow for CountingRow<'_> {
    type Error = Infallible;

    fn name(&self) -> &str {
        self.name
    }

    fn play(&self, _world: &World) -> Result<(), Self::Error> {
        self.applied.fetch_add(1, Ordering::Release);
        Ok(())
    }
}

/// Checkpoints declared at separate row boundaries each run once and observe every applied row through their boundary.
///
/// (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´)
/// ´test:crate:checkpoints-observe-each-declared-row-once´
#[test]
fn checkpoints_observe_each_declared_row_once() {
    let world = World::cold("playback-checkpoints", 0xA11C_E003);
    let applied = AtomicUsize::new(0);
    let first_calls = AtomicUsize::new(0);
    let third_calls = AtomicUsize::new(0);
    let rows = [
        CountingRow {
            name: "first",
            applied: &applied,
        },
        CountingRow {
            name: "second",
            applied: &applied,
        },
        CountingRow {
            name: "third",
            applied: &applied,
        },
    ];
    let mut checkpoints = [
        PlaybackCheckpoint::after_row("after first", 0, |_, location| {
            first_calls.fetch_add(1, Ordering::Relaxed);
            let observed = applied.load(Ordering::Acquire);
            if observed == 1 {
                Ok(())
            } else {
                Err(format!("{location} observed {observed} applied rows, expected 1"))
            }
        }),
        PlaybackCheckpoint::after_row("after third", 2, |_, location| {
            third_calls.fetch_add(1, Ordering::Relaxed);
            let observed = applied.load(Ordering::Acquire);
            if observed == 3 {
                Ok(())
            } else {
                Err(format!("{location} observed {observed} applied rows, expected 3"))
            }
        }),
    ];

    playback(
        &world,
        rows,
        &PlaybackBarrierPolicy::none(),
        &mut checkpoints,
        &PlaybackProgress::new(),
        PlaybackBatchSize::new(NonZeroUsize::new(2).expect("two is non-zero")),
    )
    .expect("every checkpoint should run at its declared boundary");

    assert_eq!(first_calls.load(Ordering::Relaxed), 1);
    assert_eq!(third_calls.load(Ordering::Relaxed), 1);
}

/// Progress counts each completed row once and reports finished only after the final checkpoint succeeds.
///
/// ´claim:harness:playback-progress-advances-once-after-each-completed-row´
/// ´test:crate:progress-advances-once-after-each-complete-boundary´
#[test]
fn progress_advances_once_after_each_complete_boundary() {
    let world = World::cold("playback-progress", 0xA11C_E004);
    let applied = AtomicUsize::new(0);
    let progress = PlaybackProgress::new();
    let rows = [
        CountingRow {
            name: "first",
            applied: &applied,
        },
        CountingRow {
            name: "second",
            applied: &applied,
        },
        CountingRow {
            name: "third",
            applied: &applied,
        },
    ];
    let mut checkpoints = [PlaybackCheckpoint::after_row("last boundary", 2, |_, _| Ok(()))];

    playback(
        &world,
        rows,
        &PlaybackBarrierPolicy::none(),
        &mut checkpoints,
        &progress,
        PlaybackBatchSize::new(NonZeroUsize::new(2).expect("two is non-zero")),
    )
    .expect("all rows should complete");

    assert_eq!(progress.completed_rows(), 3);
    assert_eq!(progress.first_incomplete_row(), None);
    assert!(progress.is_finished());
}
