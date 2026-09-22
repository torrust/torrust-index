// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Shared state holder for lock-free snapshot publication.
//!
//! `SharedState` wraps an `ArcSwap<ModelSnapshot>` that the model-owner
//! thread stores into and reader threads load from. `ArcSwap` guarantees
//! lock-free reads and wait-free loads.
//!
//! # Reader Protocol
//!
//! ```ignore
//! let guard = shared.published.load();
//! let version = guard.version;
//! // guard keeps the snapshot alive until dropped
//! ```
//!
//! # Writer Protocol
//!
//! ```ignore
//! shared.published.store(Arc::new(new_snapshot));
//! ```
//!
//! # Cross-References
//!
//! - (´dec:concurrency:snapshot-swap´) — lock-free publication by one swap
//! - (´dec:retention:monolithic-snapshot´) — one allocation swapped whole; the
//!   transient second copy is the ~18 MB this costs

use std::sync::Arc;
#[cfg(any(test, feature = "test-support"))]
use std::sync::Mutex;
#[cfg(any(test, feature = "test-support"))]
use std::time::Duration;

use arc_swap::{ArcSwap, ArcSwapOption};

use super::published::ModelSnapshot;
use crate::error::LabelPathStop;

/// State errors from the test-support model-update region recorder.
#[cfg(any(test, feature = "test-support"))]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUpdateRecorderError {
    /// Activation was requested while a sample was already active.
    AlreadyActive,
    /// Drain was requested without an active sample.
    Inactive,
    /// Drain was requested before any publication completed its update region.
    Incomplete,
}

/// Read-only test-support recorder for the joined label model-update region.
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, Default)]
pub struct LabelUpdateRecorder {
    /// Activation and completed-publication accumulation for one sample.
    state: Mutex<LabelUpdateRecorderState>,
}

/// Mutable accumulation owned only by [`LabelUpdateRecorder`].
#[cfg(any(test, feature = "test-support"))]
#[derive(Debug, Default)]
struct LabelUpdateRecorderState {
    /// Whether a sample currently accepts completed publication durations.
    active: bool,
    /// Successful snapshot publications recorded after activation.
    completed_publications: usize,
    /// Sum of the model-update regions for those publications.
    elapsed: Duration,
}

#[cfg(any(test, feature = "test-support"))]
impl LabelUpdateRecorder {
    /// Starts an empty sample without changing product state.
    pub(crate) fn activate(&self) -> Result<(), LabelUpdateRecorderError> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.active {
            return Err(LabelUpdateRecorderError::AlreadyActive);
        }
        *state = LabelUpdateRecorderState {
            active: true,
            ..LabelUpdateRecorderState::default()
        };
        drop(state);
        Ok(())
    }

    /// Adds one completed owner-thread update region when a sample is active.
    pub(crate) fn record(&self, elapsed: Duration) {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if state.active {
            state.completed_publications += 1;
            state.elapsed += elapsed;
        }
    }

    /// Ends a completed sample and returns only its accumulated region time.
    pub(crate) fn drain(&self) -> Result<Duration, LabelUpdateRecorderError> {
        let mut state = self.state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
        if !state.active {
            return Err(LabelUpdateRecorderError::Inactive);
        }
        if state.completed_publications == 0 {
            return Err(LabelUpdateRecorderError::Incomplete);
        }
        let elapsed = state.elapsed;
        *state = LabelUpdateRecorderState::default();
        drop(state);
        Ok(elapsed)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SharedState
// ═══════════════════════════════════════════════════════════════════════════════

/// Shared state between the model-owner thread and readers.
///
/// Holds the published snapshot behind an `ArcSwap` for lock-free
/// concurrent access. The model-owner thread is the sole writer;
/// any number of reader threads may call `load()` concurrently.
///
/// Layer 5 (contracts and boundaries) adds the second, independent swap
/// (´dec:health:independent-publication´):
/// `pub health: ArcSwap<PublishedHealthSummary>`.
pub struct SharedState {
    /// The most recently published model snapshot.
    ///
    /// Readers call `load()` to get a `Guard<Arc<ModelSnapshot>>`.
    /// The model-owner thread calls `store()` after each label
    /// that updates the working copy.
    pub published: ArcSwap<ModelSnapshot>,

    /// The most recently published health summary (´dec:health:independent-publication´).
    ///
    /// Published alongside `published` at step 17 (snapshot first,
    /// health second — intermediate state is benign).
    ///
    /// **Ordering note:** Between the two stores a concurrent reader
    /// may observe the *new* snapshot but the *old* health summary
    /// (snapshot "leads" health).  The reverse is not possible.
    pub health: ArcSwap<crate::health::PublishedHealthSummary>,

    /// Why the label path stopped, or empty while it is running.
    ///
    /// A third swap rather than a field of either of the other two, because
    /// what it says is true of the owner rather than of any snapshot: the
    /// published snapshot the assessments keep reading is the one that was
    /// current before the stop and must not be rewritten to carry it, and the
    /// health summary is republished on a label the stopped path will never
    /// take. The model-owner thread is the sole writer here as it is there,
    /// and it writes once — a stop is not lifted (´dec:concurrency:snapshot-swap´).
    ///
    /// Both the label surface and the health surface read it: the first to
    /// refuse a submission with its cause, the second to raise
    /// `label_path_stopped` in both tiers.
    pub label_path_stop: ArcSwapOption<LabelPathStop>,

    /// Test-support timing probe for the joined label-update region.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) label_update_recorder: LabelUpdateRecorder,
}

impl SharedState {
    /// Creates a new `SharedState` with the given initial snapshot.
    #[must_use]
    pub fn new(initial: ModelSnapshot) -> Self {
        Self {
            published: ArcSwap::from_pointee(initial),
            health: ArcSwap::from_pointee(crate::health::PublishedHealthSummary::default()),
            label_path_stop: ArcSwapOption::empty(),
            #[cfg(any(test, feature = "test-support"))]
            label_update_recorder: LabelUpdateRecorder::default(),
        }
    }

    /// Creates a new `SharedState` from an existing `Arc<ModelSnapshot>`.
    #[must_use]
    pub fn from_arc(initial: Arc<ModelSnapshot>) -> Self {
        Self {
            published: ArcSwap::new(initial),
            health: ArcSwap::from_pointee(crate::health::PublishedHealthSummary::default()),
            label_path_stop: ArcSwapOption::empty(),
            #[cfg(any(test, feature = "test-support"))]
            label_update_recorder: LabelUpdateRecorder::default(),
        }
    }
}

impl std::fmt::Debug for SharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.published.load();
        let mut debug = f.debug_struct("SharedState");
        debug
            .field("published_version", &guard.version)
            .field("health", &"<ArcSwap<PublishedHealthSummary>>")
            .field("label_path_stop", &self.label_path_stop.load_full());
        #[cfg(any(test, feature = "test-support"))]
        debug.field("label_update_recorder", &self.label_update_recorder);
        debug.finish()
    }
}
