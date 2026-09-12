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

use arc_swap::{ArcSwap, ArcSwapOption};

use super::published::ModelSnapshot;
use crate::error::LabelPathStop;

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
}

impl SharedState {
    /// Creates a new `SharedState` with the given initial snapshot.
    #[must_use]
    pub fn new(initial: ModelSnapshot) -> Self {
        Self {
            published: ArcSwap::from_pointee(initial),
            health: ArcSwap::from_pointee(crate::health::PublishedHealthSummary::default()),
            label_path_stop: ArcSwapOption::empty(),
        }
    }

    /// Creates a new `SharedState` from an existing `Arc<ModelSnapshot>`.
    #[must_use]
    pub fn from_arc(initial: Arc<ModelSnapshot>) -> Self {
        Self {
            published: ArcSwap::new(initial),
            health: ArcSwap::from_pointee(crate::health::PublishedHealthSummary::default()),
            label_path_stop: ArcSwapOption::empty(),
        }
    }
}

impl std::fmt::Debug for SharedState {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let guard = self.published.load();
        f.debug_struct("SharedState")
            .field("published_version", &guard.version)
            .field("health", &"<ArcSwap<PublishedHealthSummary>>")
            .field("label_path_stop", &self.label_path_stop.load_full())
            .finish()
    }
}
