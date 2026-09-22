// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Declarative playback for long test stimuli.
//!
//! Subject modules own their row types and implement [`PlaybackRow`]. This
//! module owns the execution semantics around those rows: the ordered barrier
//! policy, exact checkpoint placement, per-row progress, and bounded
//! materialisation. A batch is only an allocation and dispatch-cost boundary;
//! every row still crosses its selected barriers before its checkpoint and
//! progress boundary.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`held_barrier_completes_before_its_checkpoint`] | harness | A held barrier completes before its row checkpoint, so a settled comparison cannot run early. |
//! | [`skipped_checkpoint_is_rejected`] | harness | cites (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´) |
//! | [`duplicated_checkpoint_is_rejected`] | harness | cites (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´) |
//! | [`early_checkpoint_is_rejected`] | harness | cites (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´) |
//! | [`later_boundary_checkpoint_is_rejected`] | harness | An invocation after the declared row is rejected as a later boundary. |
//! | [`stalled_progress_names_the_first_incomplete_row`] | harness | Progress advances once for a completed row and names the next row while that row is deliberately blocked. |
//! | [`batch_size_preserves_mid_batch_publication_boundaries`] | harness | A row after a mid-batch state change sees the preceding row's publication for every bounded batch size. |

use std::fmt;
use std::num::NonZeroUsize;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{Arc, Mutex, MutexGuard};

use super::World;

/// One subject-owned row that the shared runner can interpret.
pub trait PlaybackRow {
    /// Error returned when this subject cannot apply a row.
    type Error: fmt::Display;

    /// Stable diagnostic name for this row.
    fn name(&self) -> &str;

    /// Apply this row to the real scenario subject without choosing a barrier.
    ///
    /// The runner crosses the selected barriers after this method returns, so
    /// an implementation must not add its own settling cadence.
    ///
    /// # Errors
    ///
    /// Returns the subject-specific failure to apply the row.
    fn play(&self, world: &World) -> Result<(), Self::Error>;
}

/// One queue boundary available to a playback policy.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlaybackBarrier {
    /// Complete model-owner commands and labels through publication.
    FlushLabels,
    /// Complete cold-ramp observations through publication.
    FlushObservations,
    /// Complete identity maintenance and publish accepted lifecycle work.
    FlushIdentityMaintenance,
    /// Complete one explicitly triggered Ledger collection cycle.
    FlushLedgerGc,
    /// Complete one checkpoint-scheduler trigger cycle.
    FlushCheckpointScheduler,
    /// Complete every health-event producer and drain accepted events.
    DrainHealthEvents,
}

impl fmt::Display for PlaybackBarrier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(match self {
            Self::FlushLabels => "flush_labels",
            Self::FlushObservations => "flush_observations",
            Self::FlushIdentityMaintenance => "flush_identity_maintenance",
            Self::FlushLedgerGc => "flush_ledger_gc",
            Self::FlushCheckpointScheduler => "flush_checkpoint_scheduler",
            Self::DrainHealthEvents => "drain_health_events",
        })
    }
}

/// Ordered set of existing [`World`] barriers required at every row boundary.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct PlaybackBarrierPolicy {
    barriers: Vec<PlaybackBarrier>,
}

impl PlaybackBarrierPolicy {
    /// Select barriers in the exact order playback must cross them.
    #[must_use]
    pub fn new(barriers: impl IntoIterator<Item = PlaybackBarrier>) -> Self {
        Self {
            barriers: barriers.into_iter().collect(),
        }
    }

    /// Select no asynchronous completion boundary.
    #[must_use]
    pub const fn none() -> Self {
        Self { barriers: Vec::new() }
    }

    fn settle(&self, world: &World, location: &PlaybackRowLocation) -> Result<(), PlaybackError> {
        for barrier in &self.barriers {
            let result = match barrier {
                PlaybackBarrier::FlushLabels => world.flush_labels().map_err(|error| error.to_string()),
                PlaybackBarrier::FlushObservations => world.flush_observations().map_err(|error| error.to_string()),
                PlaybackBarrier::FlushIdentityMaintenance => {
                    world.flush_identity_maintenance().map_err(|error| error.to_string())
                }
                PlaybackBarrier::FlushLedgerGc => world.flush_ledger_gc().map(drop).map_err(|error| error.to_string()),
                PlaybackBarrier::FlushCheckpointScheduler => {
                    world.flush_checkpoint_scheduler().map_err(|error| error.to_string())
                }
                PlaybackBarrier::DrainHealthEvents => world.drain_health_events().map(drop).map_err(|error| error.to_string()),
            };
            if let Err(message) = result {
                return Err(PlaybackError::Barrier {
                    location: location.clone(),
                    barrier: *barrier,
                    message,
                });
            }
        }
        Ok(())
    }
}

/// Non-zero upper bound on rows materialised together by playback.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlaybackBatchSize(NonZeroUsize);

impl PlaybackBatchSize {
    /// Build a bounded batch size from an already validated non-zero value.
    #[must_use]
    pub const fn new(size: NonZeroUsize) -> Self {
        Self(size)
    }

    /// Number of rows materialised at once.
    #[must_use]
    pub const fn get(self) -> usize {
        self.0.get()
    }
}

impl Default for PlaybackBatchSize {
    fn default() -> Self {
        Self(NonZeroUsize::MIN)
    }
}

/// Zero-based row position and subject-supplied diagnostic name.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackRowLocation {
    index: usize,
    name: String,
}

impl PlaybackRowLocation {
    fn new(index: usize, name: impl Into<String>) -> Self {
        Self {
            index,
            name: name.into(),
        }
    }

    /// Zero-based row index.
    #[must_use]
    pub const fn index(&self) -> usize {
        self.index
    }

    /// Subject-supplied row name.
    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }
}

impl fmt::Display for PlaybackRowLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "row {} '{}'", self.index, self.name)
    }
}

#[derive(Debug, Default)]
struct PlaybackProgressState {
    first_incomplete: Option<PlaybackRowLocation>,
    finished: bool,
}

/// Shareable liveness reading advanced once after each completed row.
#[derive(Clone, Debug, Default)]
pub struct PlaybackProgress {
    completed: Arc<AtomicUsize>,
    state: Arc<Mutex<PlaybackProgressState>>,
}

impl PlaybackProgress {
    /// Start an empty progress reading.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Number of rows whose dispatch, barriers, and checkpoints completed.
    #[must_use]
    pub fn completed_rows(&self) -> usize {
        self.completed.load(Ordering::Acquire)
    }

    /// First row whose complete boundary has not yet been crossed.
    #[must_use]
    pub fn first_incomplete_row(&self) -> Option<PlaybackRowLocation> {
        self.lock_state().first_incomplete.clone()
    }

    /// Liveness diagnostic for the row currently preventing advancement.
    #[must_use]
    pub fn stalled(&self) -> Option<PlaybackStall> {
        self.first_incomplete_row().map(|location| PlaybackStall {
            completed_rows: self.completed_rows(),
            location,
        })
    }

    /// Whether playback completed every supplied row and checkpoint.
    #[must_use]
    pub fn is_finished(&self) -> bool {
        self.lock_state().finished
    }

    fn begin(&self, location: PlaybackRowLocation) {
        let mut state = self.lock_state();
        state.first_incomplete = Some(location);
        state.finished = false;
    }

    fn advanced(&self) {
        self.completed.fetch_add(1, Ordering::Release);
        self.lock_state().first_incomplete = None;
    }

    fn finished(&self) {
        let mut state = self.lock_state();
        state.first_incomplete = None;
        state.finished = true;
    }

    fn lock_state(&self) -> MutexGuard<'_, PlaybackProgressState> {
        self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner)
    }
}

/// Snapshot of playback progress while one row remains incomplete.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PlaybackStall {
    completed_rows: usize,
    location: PlaybackRowLocation,
}

impl PlaybackStall {
    /// Rows completed before the stalled row.
    #[must_use]
    pub const fn completed_rows(&self) -> usize {
        self.completed_rows
    }

    /// First row whose full playback boundary has not completed.
    #[must_use]
    pub const fn first_incomplete_row(&self) -> &PlaybackRowLocation {
        &self.location
    }
}

impl fmt::Display for PlaybackStall {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let noun = if self.completed_rows == 1 { "row" } else { "rows" };
        write!(
            f,
            "playback stalled after {} completed {noun}; first incomplete {}",
            self.completed_rows, self.location,
        )
    }
}

type CheckpointCallback<'a> = dyn FnMut(&World, &PlaybackRowLocation) -> Result<(), String> + 'a;

/// Callback declared for exactly one completed row boundary.
pub struct PlaybackCheckpoint<'a> {
    name: String,
    after_row: usize,
    callback: Box<CheckpointCallback<'a>>,
    calls: usize,
}

impl fmt::Debug for PlaybackCheckpoint<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("PlaybackCheckpoint")
            .field("name", &self.name)
            .field("after_row", &self.after_row)
            .field("calls", &self.calls)
            .finish_non_exhaustive()
    }
}

impl<'a> PlaybackCheckpoint<'a> {
    /// Declare a named callback after one zero-based row index.
    pub fn after_row(
        name: impl Into<String>,
        row: usize,
        callback: impl FnMut(&World, &PlaybackRowLocation) -> Result<(), String> + 'a,
    ) -> Self {
        Self {
            name: name.into(),
            after_row: row,
            callback: Box::new(callback),
            calls: 0,
        }
    }

    fn invoke(&mut self, world: &World, actual: &PlaybackRowLocation) -> Result<(), PlaybackCheckpointError> {
        self.accept_boundary(actual)?;
        (self.callback)(world, actual).map_err(|message| PlaybackCheckpointError::Callback {
            checkpoint: self.name.clone(),
            location: actual.clone(),
            message,
        })
    }

    fn accept_boundary(&mut self, actual: &PlaybackRowLocation) -> Result<(), PlaybackCheckpointError> {
        if actual.index != self.after_row {
            return Err(PlaybackCheckpointError::WrongBoundary {
                checkpoint: self.name.clone(),
                declared_row: self.after_row,
                actual: actual.clone(),
            });
        }
        if self.calls != 0 {
            return Err(PlaybackCheckpointError::Duplicate {
                checkpoint: self.name.clone(),
                location: actual.clone(),
            });
        }
        self.calls += 1;
        Ok(())
    }

    fn verify_called(&self) -> Result<(), PlaybackCheckpointError> {
        if self.calls == 1 {
            Ok(())
        } else {
            Err(PlaybackCheckpointError::Skipped {
                checkpoint: self.name.clone(),
                declared_row: self.after_row,
            })
        }
    }
}

/// A checkpoint invocation violated its declared boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackCheckpointError {
    /// The runner completed without invoking a declared checkpoint.
    Skipped {
        /// Diagnostic checkpoint name.
        checkpoint: String,
        /// Declared zero-based row boundary.
        declared_row: usize,
    },
    /// The runner invoked a checkpoint more than once.
    Duplicate {
        /// Diagnostic checkpoint name.
        checkpoint: String,
        /// Row at which the duplicate was attempted.
        location: PlaybackRowLocation,
    },
    /// The runner invoked a checkpoint at another row boundary.
    WrongBoundary {
        /// Diagnostic checkpoint name.
        checkpoint: String,
        /// Declared zero-based row boundary.
        declared_row: usize,
        /// Row at which invocation was attempted.
        actual: PlaybackRowLocation,
    },
    /// The checkpoint callback rejected the settled state it observed.
    Callback {
        /// Diagnostic checkpoint name.
        checkpoint: String,
        /// Settled row observed by the callback.
        location: PlaybackRowLocation,
        /// Subject-specific checkpoint failure.
        message: String,
    },
}

impl fmt::Display for PlaybackCheckpointError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Skipped {
                checkpoint,
                declared_row,
            } => write!(f, "checkpoint '{checkpoint}' was skipped at declared row {declared_row}"),
            Self::Duplicate { checkpoint, location } => write!(f, "checkpoint '{checkpoint}' ran more than once at {location}"),
            Self::WrongBoundary {
                checkpoint,
                declared_row,
                actual,
            } => write!(f, "checkpoint '{checkpoint}' declared for row {declared_row} ran at {actual}"),
            Self::Callback {
                checkpoint,
                location,
                message,
            } => write!(f, "checkpoint '{checkpoint}' rejected {location}: {message}"),
        }
    }
}

impl std::error::Error for PlaybackCheckpointError {}

/// Failure to complete a declarative playback boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PlaybackError {
    /// A subject row could not be applied.
    Row {
        /// First incomplete row.
        location: PlaybackRowLocation,
        /// Subject-specific failure.
        message: String,
    },
    /// A selected barrier failed before the row checkpoint.
    Barrier {
        /// Row blocked at this barrier.
        location: PlaybackRowLocation,
        /// Selected barrier that failed.
        barrier: PlaybackBarrier,
        /// Barrier-specific failure.
        message: String,
    },
    /// A checkpoint violated its declaration or rejected observed state.
    Checkpoint(PlaybackCheckpointError),
}

impl fmt::Display for PlaybackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Row { location, message } => write!(f, "playback stopped at {location}: {message}"),
            Self::Barrier {
                location,
                barrier,
                message,
            } => write!(f, "playback blocked at {location} crossing {barrier}: {message}"),
            Self::Checkpoint(error) => error.fmt(f),
        }
    }
}

impl std::error::Error for PlaybackError {}

impl From<PlaybackCheckpointError> for PlaybackError {
    fn from(error: PlaybackCheckpointError) -> Self {
        Self::Checkpoint(error)
    }
}

/// Interpret subject-owned rows under shared playback semantics.
///
/// Rows are materialised in bounded chunks, but each row is applied, settled,
/// checkpointed, and counted before the next row begins. Changing `batch_size`
/// therefore changes allocation and iterator cost without changing any
/// observable boundary.
///
/// # Errors
///
/// Returns the first row, barrier, or checkpoint failure. Progress remains on
/// that first incomplete row for liveness reporting.
pub fn playback<R, I>(
    world: &World,
    rows: I,
    barrier_policy: &PlaybackBarrierPolicy,
    checkpoints: &mut [PlaybackCheckpoint<'_>],
    progress: &PlaybackProgress,
    batch_size: PlaybackBatchSize,
) -> Result<(), PlaybackError>
where
    R: PlaybackRow,
    I: IntoIterator<Item = R>,
{
    execute_rows(
        rows.into_iter().enumerate(),
        batch_size,
        progress,
        |row, location| {
            row.play(world).map_err(|error| PlaybackError::Row {
                location: location.clone(),
                message: error.to_string(),
            })
        },
        |location| barrier_policy.settle(world, location),
        |location| {
            for checkpoint in checkpoints
                .iter_mut()
                .filter(|checkpoint| checkpoint.after_row == location.index)
            {
                checkpoint.invoke(world, location)?;
            }
            Ok(())
        },
    )?;

    for checkpoint in checkpoints {
        checkpoint.verify_called()?;
    }
    progress.finished();
    Ok(())
}

fn execute_rows<R, I, D, B, C>(
    mut rows: I,
    batch_size: PlaybackBatchSize,
    progress: &PlaybackProgress,
    mut dispatch: D,
    mut barriers: B,
    mut checkpoints: C,
) -> Result<(), PlaybackError>
where
    R: PlaybackRow,
    I: Iterator<Item = (usize, R)>,
    D: FnMut(&R, &PlaybackRowLocation) -> Result<(), PlaybackError>,
    B: FnMut(&PlaybackRowLocation) -> Result<(), PlaybackError>,
    C: FnMut(&PlaybackRowLocation) -> Result<(), PlaybackError>,
{
    loop {
        let batch: Vec<_> = rows.by_ref().take(batch_size.get()).collect();
        if batch.is_empty() {
            return Ok(());
        }
        for (index, row) in batch {
            let location = PlaybackRowLocation::new(index, row.name());
            progress.begin(location.clone());
            dispatch(&row, &location)?;
            barriers(&location)?;
            checkpoints(&location)?;
            progress.advanced();
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;
    use std::convert::Infallible;
    use std::sync::mpsc;

    use super::*;

    #[derive(Clone, Copy)]
    struct NamedRow(&'static str);

    impl PlaybackRow for NamedRow {
        type Error = Infallible;

        fn name(&self) -> &str {
            self.0
        }

        fn play(&self, _world: &World) -> Result<(), Self::Error> {
            Ok(())
        }
    }

    fn location(index: usize, name: &str) -> PlaybackRowLocation {
        PlaybackRowLocation::new(index, name)
    }

    /// A held barrier completes before its row checkpoint, so a settled comparison cannot run early.
    ///
    /// ´claim:harness:playback-checkpoints-follow-every-selected-barrier´
    /// ´test:unit:held-barrier-completes-before-its-checkpoint´
    #[test]
    fn held_barrier_completes_before_its_checkpoint() {
        let progress = PlaybackProgress::new();
        let events = Arc::new(Mutex::new(Vec::new()));
        let (entered_tx, entered_rx) = mpsc::sync_channel(0);
        let (release_tx, release_rx) = mpsc::sync_channel(0);

        std::thread::scope(|scope| {
            let worker_events = Arc::clone(&events);
            let worker_progress = progress.clone();
            let worker = scope.spawn(move || {
                execute_rows(
                    std::iter::once(NamedRow("held-label-row")).enumerate(),
                    PlaybackBatchSize::default(),
                    &worker_progress,
                    |_, location| {
                        worker_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(format!("dispatch {location}"));
                        Ok(())
                    },
                    |location| {
                        worker_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(format!("barrier entered {location}"));
                        entered_tx.send(()).expect("held barrier should announce entry");
                        release_rx.recv().expect("held barrier should receive release");
                        worker_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(format!("barrier completed {location}"));
                        Ok(())
                    },
                    |location| {
                        worker_events
                            .lock()
                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                            .push(format!("checkpoint {location}"));
                        Ok(())
                    },
                )
            });

            entered_rx.recv().expect("runner should reach the held barrier");
            release_tx.send(()).expect("test should release the held barrier");
            worker
                .join()
                .expect("playback worker should not panic")
                .expect("playback should complete");
        });

        let observed = events.lock().unwrap_or_else(std::sync::PoisonError::into_inner).clone();
        assert_eq!(
            observed,
            [
                "dispatch row 0 'held-label-row'",
                "barrier entered row 0 'held-label-row'",
                "barrier completed row 0 'held-label-row'",
                "checkpoint row 0 'held-label-row'",
            ],
            "checkpoint for row 0 'held-label-row' ran before its held barrier completed"
        );
    }

    fn checkpoint_for(row: usize) -> PlaybackCheckpoint<'static> {
        PlaybackCheckpoint::after_row("boundary witness", row, |_, _| Ok(()))
    }

    /// A missing checkpoint is rejected at its declared row boundary.
    ///
    /// (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´)
    /// ´test:unit:skipped-checkpoint-is-rejected´
    #[test]
    fn skipped_checkpoint_is_rejected() {
        let error = checkpoint_for(1).verify_called().expect_err("a skipped checkpoint must fail");
        assert_eq!(
            error,
            PlaybackCheckpointError::Skipped {
                checkpoint: "boundary witness".to_owned(),
                declared_row: 1,
            }
        );
        assert_eq!(
            error.to_string(),
            "checkpoint 'boundary witness' was skipped at declared row 1"
        );
    }

    /// A second invocation is rejected rather than calling the checkpoint twice.
    ///
    /// (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´)
    /// ´test:unit:duplicated-checkpoint-is-rejected´
    #[test]
    fn duplicated_checkpoint_is_rejected() {
        let mut checkpoint = checkpoint_for(1);
        let boundary = location(1, "declared-row");
        checkpoint
            .accept_boundary(&boundary)
            .expect("first invocation should be accepted");
        let error = checkpoint
            .accept_boundary(&boundary)
            .expect_err("a duplicate checkpoint must fail");
        assert_eq!(
            error,
            PlaybackCheckpointError::Duplicate {
                checkpoint: "boundary witness".to_owned(),
                location: boundary,
            }
        );
        assert_eq!(
            error.to_string(),
            "checkpoint 'boundary witness' ran more than once at row 1 'declared-row'"
        );
    }

    /// An invocation before the declared row is rejected as early.
    ///
    /// (´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´)
    /// ´test:unit:early-checkpoint-is-rejected´
    #[test]
    fn early_checkpoint_is_rejected() {
        let mut checkpoint = checkpoint_for(1);
        let actual = location(0, "earlier-row");
        let error = checkpoint
            .accept_boundary(&actual)
            .expect_err("an early checkpoint must fail");
        assert_eq!(
            error,
            PlaybackCheckpointError::WrongBoundary {
                checkpoint: "boundary witness".to_owned(),
                declared_row: 1,
                actual,
            }
        );
        assert_eq!(
            error.to_string(),
            "checkpoint 'boundary witness' declared for row 1 ran at row 0 'earlier-row'"
        );
    }

    /// An invocation after the declared row is rejected as a later boundary.
    ///
    /// ´claim:harness:a-declared-playback-checkpoint-runs-exactly-once-at-its-row-boundary´
    /// ´test:unit:later-boundary-checkpoint-is-rejected´
    #[test]
    fn later_boundary_checkpoint_is_rejected() {
        let mut checkpoint = checkpoint_for(1);
        let actual = location(2, "later-row");
        let error = checkpoint
            .accept_boundary(&actual)
            .expect_err("a later-boundary checkpoint must fail");
        assert_eq!(
            error,
            PlaybackCheckpointError::WrongBoundary {
                checkpoint: "boundary witness".to_owned(),
                declared_row: 1,
                actual,
            }
        );
        assert_eq!(
            error.to_string(),
            "checkpoint 'boundary witness' declared for row 1 ran at row 2 'later-row'"
        );
    }

    /// Progress advances once for a completed row and names the next row while that row is deliberately blocked.
    ///
    /// ´claim:harness:playback-progress-names-the-first-incomplete-row-without-promising-latency´
    /// ´test:unit:stalled-progress-names-the-first-incomplete-row´
    #[test]
    fn stalled_progress_names_the_first_incomplete_row() {
        let progress = PlaybackProgress::new();
        let (entered_tx, entered_rx) = mpsc::sync_channel(0);
        let (release_tx, release_rx) = mpsc::sync_channel(0);

        std::thread::scope(|scope| {
            let worker_progress = progress.clone();
            let worker = scope.spawn(move || {
                execute_rows(
                    [NamedRow("completed-row"), NamedRow("stalled-row")].into_iter().enumerate(),
                    PlaybackBatchSize::new(NonZeroUsize::new(2).expect("two is non-zero")),
                    &worker_progress,
                    |_, location| {
                        if location.index == 1 {
                            entered_tx.send(()).expect("stalled row should announce entry");
                            release_rx.recv().expect("stalled row should receive release");
                        }
                        Ok(())
                    },
                    |_| Ok(()),
                    |_| Ok(()),
                )
            });

            entered_rx.recv().expect("runner should enter the deliberately stalled row");
            let observed_completed = progress.completed_rows();
            let observed_incomplete = progress.first_incomplete_row();
            let observed_stall = progress.stalled();
            release_tx.send(()).expect("test should release the stalled row");
            worker
                .join()
                .expect("playback worker should not panic")
                .expect("playback should complete");

            assert_eq!(
                (observed_completed, observed_incomplete),
                (1, Some(location(1, "stalled-row"))),
                "stalled playback must report one completed row and identify row 1 'stalled-row' as first incomplete"
            );
            let stalled = observed_stall.expect("the blocked row should produce a stall diagnostic");
            assert_eq!(stalled.completed_rows(), 1);
            assert_eq!(stalled.first_incomplete_row(), &location(1, "stalled-row"));
            assert_eq!(
                stalled.to_string(),
                "playback stalled after 1 completed row; first incomplete row 1 'stalled-row'"
            );
        });

        assert_eq!(progress.completed_rows(), 2);
    }

    #[derive(Debug, Default, PartialEq, Eq)]
    struct PublishedState {
        pending: usize,
        published: usize,
        observations: Vec<usize>,
    }

    fn run_mid_batch(batch_size: NonZeroUsize) -> Result<(PublishedState, usize), PlaybackError> {
        let state = RefCell::new(PublishedState::default());
        let progress = PlaybackProgress::new();
        execute_rows(
            [NamedRow("publish-state"), NamedRow("observe-published-state")]
                .into_iter()
                .enumerate(),
            PlaybackBatchSize::new(batch_size),
            &progress,
            |_, location| {
                let mut state = state.borrow_mut();
                if location.index == 0 {
                    state.pending += 1;
                } else if state.published != 1 {
                    return Err(PlaybackError::Row {
                        location: location.clone(),
                        message: "row 1 'observe-published-state' saw one shared batch view before row 0 publication".to_owned(),
                    });
                } else {
                    let published = state.published;
                    state.observations.push(published);
                }
                Ok(())
            },
            |_| {
                let mut state = state.borrow_mut();
                state.published += state.pending;
                state.pending = 0;
                Ok(())
            },
            |_| Ok(()),
        )?;
        Ok((state.into_inner(), progress.completed_rows()))
    }

    /// A row after a mid-batch state change sees the preceding row's publication for every bounded batch size.
    ///
    /// ´claim:harness:playback-batches-change-cost-without-sharing-a-published-view´
    /// ´test:unit:batch-size-preserves-mid-batch-publication-boundaries´
    #[test]
    fn batch_size_preserves_mid_batch_publication_boundaries() {
        let single = run_mid_batch(NonZeroUsize::MIN).expect("single-row batches should complete");
        let combined = run_mid_batch(NonZeroUsize::new(2).expect("two is non-zero"))
            .expect("a combined batch must preserve row publication boundaries");
        assert_eq!(single, combined);
        assert_eq!(combined.0.observations, [1]);
        assert_eq!(combined.1, 2);
    }
}
