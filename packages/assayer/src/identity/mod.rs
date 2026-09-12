// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`identity_dimension_infra_construction`] | identity | A dimension's shared infrastructure starts blank in all three of its parts: an empty published competitive set, no per-cell state, and no observations recorded as dropped. Nothing about a dimension is inherited from whatever the process was doing before — its competitive geometry is built entirely from the traffic it goes on to see. |
//! | [`identity_dimension_infra_find_active_empty`] | identity | cites (´claim:identity:an-empty-competitive-set-matches-no-coordinate´) |
//! | [`identity_dimension_infra_observe`] | identity | cites (´claim:identity:an-observation-handed-over-by-an-assessment-thread-reaches-the-maintenance-side-unchanged´) |
//! | [`identity_dimension_infra_observe_overflow`] | identity | cites (´claim:identity:every-refused-observation-is-counted-so-shed-load-is-visible-rather-than-silent´) |

//! Identity layer for entity-specific baseline tracking.
//!
// Data structures only. A dedicated owner holds every identity graph, and the
// assessment path never mutates one (´dec:memory:graph-owner´).
#![allow(unused_imports)]
//!
//! # Module Index
//!
//! | Module | Purpose |
//! |--------|---------|
//! | [`competitive`] | Competitive cell types and indexing |
//! | [`cell_state`] | Per-cell mutable state (measurement, outcome) |
//! | [`dimension`] | Dimension configuration |
//! | [`observation`] | Observation deferral channel |
//! | [`maintenance_loop`] | Maintenance thread and commands |
//! | [`snapshot`] | Graph snapshot for checkpointing |
//!
//! # Architecture
//!
//! The identity layer enables entity-specific baseline tracking through
//! competitive cells. Each identity dimension partitions the coordinate
//! space via a G-V graph, identifying important regions (competitive cells)
//! that receive dedicated tracking.
//!
//! ```text
//!                     ┌──────────────────────────────────────┐
//!                     │        assessment threads            │
//!                     │   (concurrent coordinate lookups)    │
//!                     └───────────────┬──────────────────────┘
//!                                     │ try_send(coord)
//!                                     ▼
//!               ┌─────────────────────────────────────────────┐
//!               │         Observation Channel                 │
//!               │  (bounded, per-dimension, 100K capacity)    │
//!               └───────────────────┬─────────────────────────┘
//!                                   │ drain
//!                                   ▼
//!     ┌─────────────────────────────────────────────────────────────┐
//!     │                Identity Maintenance Loop                     │
//!     │  (dedicated thread, manages G-V graphs, emits lifecycle)    │
//!     └────────────────────────────┬────────────────────────────────┘
//!                                  │ LifecycleEvent
//!                                  ▼
//!                      ┌────────────────────────┐
//!                      │  Model Owner Thread    │
//!                      │  (processes events)    │
//!                      └────────────────────────┘
//! ```
//!
//! # The Feed-Forward Boundary
//!
//! Observations are feed-forward only (´dec:ownership:feed-forward´):
//! - `assess()` threads can only submit coordinates (no custom deltas)
//! - The maintenance loop applies `Δ=1` internally
//! - No path exists for callers to influence graph weights directly
//!
//! # Cross-References
//!
//! - (´dec:ownership:graph-custody´) — the package owns its identity graphs
//!   outright
//! - (´dec:memory:graph-owner´) — the dedicated owner that holds each one
//! - (´schema:keyspace:dimension-record´) — what a registered dimension
//!   declares
//! - (´mot:keyspace:purpose´) — what the identity layer is for

mod cell_state;
mod competitive;
mod dimension;
mod maintenance_loop;
mod observation;
mod snapshot;

use std::collections::{BTreeMap, HashMap};
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex, RwLock};

use arc_swap::ArcSwap;
pub use cell_state::{CellMutableState, CellOutcomeState, CellOutcomeUpdate, MeasurementState};
pub use competitive::{CompetitiveCellId, CompetitiveCellInfo, CompetitiveSetIndex};
use crossbeam_channel::TrySendError;
pub use dimension::IdentityDimension;
pub use maintenance_loop::{IdentityMaintenanceLoop, MaintenanceCommand, spawn_identity_maintenance_thread};
pub use observation::{DEFAULT_OBSERVATION_CHANNEL_CAPACITY, ObservationChannel};
pub use snapshot::IdentityGraphSnapshot;

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension Infrastructure
// ═══════════════════════════════════════════════════════════════════════════════

/// Shared infrastructure for an identity dimension.
///
/// This struct is held by the Assayer and shared with the maintenance loop.
/// It provides lock-free access to the competitive set index (for assessment threads)
/// and locked access to per-cell mutable state.
///
/// # Thread Safety
///
/// - `competitive_set`: Lock-free reads via `ArcSwap`. Only the maintenance
///   loop writes (via `store()`).
/// - `cell_state`: Read-write lock on the outer map, mutex on each cell.
///   `assess()` threads acquire read locks; maintenance loop acquires write
///   locks when adding/removing cells.
/// - `observation_tx`: Multiple senders are Clone; thread-safe by design.
/// - `graph_snapshot`: Lock-free reads; written by maintenance loop during
///   checkpoint preparation.
pub struct IdentityDimensionInfra {
    /// The dimension configuration.
    pub dimension: IdentityDimension,

    /// Current competitive set (lock-free reads).
    ///
    /// Published by the maintenance loop after competitive set changes.
    pub competitive_set: ArcSwap<CompetitiveSetIndex>,

    /// Per-cell mutable state.
    ///
    /// Keyed by `CompetitiveCellId`. Each cell's state is protected by
    /// a mutex for fine-grained locking.
    pub cell_state: RwLock<HashMap<CompetitiveCellId, Mutex<CellMutableState>>>,

    /// Sender for observation coordinates.
    ///
    /// Clone this sender for use by assessment threads.
    pub observation_tx: crossbeam_channel::Sender<u128>,

    /// Number of observations dropped because the assessment path could not
    /// enqueue them for maintenance.
    observations_dropped: AtomicU64,

    /// Lifetime assessment counts grouped by how many indicators were active.
    active_indicator_counts: Mutex<BTreeMap<usize, u64>>,

    /// Graph snapshot for checkpointing.
    ///
    /// Published by the maintenance loop during checkpoint preparation.
    pub graph_snapshot: ArcSwap<Option<IdentityGraphSnapshot>>,
}

impl IdentityDimensionInfra {
    /// Creates new dimension infrastructure.
    ///
    /// # Arguments
    ///
    /// * `dimension` — The dimension configuration
    /// * `observation_tx` — Sender end of the observation channel
    pub fn new(dimension: IdentityDimension, observation_tx: crossbeam_channel::Sender<u128>) -> Self {
        Self {
            dimension,
            competitive_set: ArcSwap::from_pointee(CompetitiveSetIndex::empty()),
            cell_state: RwLock::new(HashMap::new()),
            observation_tx,
            observations_dropped: AtomicU64::new(0),
            active_indicator_counts: Mutex::new(BTreeMap::new()),
            graph_snapshot: ArcSwap::from_pointee(None),
        }
    }

    /// Loads the current competitive set (lock-free).
    #[must_use]
    pub fn load_competitive_set(&self) -> arc_swap::Guard<Arc<CompetitiveSetIndex>> {
        self.competitive_set.load()
    }

    /// Finds active cells for a coordinate.
    ///
    /// Returns all competitive cells whose interval contains the coordinate,
    /// ordered from deepest to shallowest.
    #[must_use]
    pub fn find_active_cells(&self, coord: u128) -> smallvec::SmallVec<[CompetitiveCellId; 4]> {
        self.competitive_set.load().find_active(coord)
    }

    /// Submits an observation for this dimension.
    ///
    /// Returns `true` if the observation was enqueued. Returns `false` and
    /// increments the drop counter if the queue is full or disconnected.
    pub fn observe(&self, coord: u128) -> bool {
        match self.observation_tx.try_send(coord) {
            Ok(()) => true,
            Err(TrySendError::Full(_) | TrySendError::Disconnected(_)) => {
                self.observations_dropped.fetch_add(1, Ordering::Relaxed);
                false
            }
        }
    }

    /// Returns the number of dropped observations for this dimension.
    #[must_use]
    pub fn observations_dropped(&self) -> u64 {
        self.observations_dropped.load(Ordering::Relaxed)
    }

    /// Records how many competitive indicators fired for one assessment.
    pub fn record_active_indicator_count(&self, active: usize) {
        let mut counts = self
            .active_indicator_counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner);
        let count = counts.entry(active).or_default();
        *count = count.saturating_add(1);
        drop(counts);
    }

    /// Returns the lifetime active-indicator count distribution.
    #[must_use]
    pub fn active_indicator_count_distribution(&self) -> BTreeMap<usize, u64> {
        self.active_indicator_counts
            .lock()
            .unwrap_or_else(std::sync::PoisonError::into_inner)
            .clone()
    }

    /// Returns the cell state for a given cell, if it exists.
    ///
    /// Acquires a read lock on the cell state map.
    pub fn with_cell_state<F, R>(&self, cell: &CompetitiveCellId, f: F) -> Option<R>
    where
        F: FnOnce(&Mutex<CellMutableState>) -> R,
    {
        let guard = self.cell_state.read().ok()?;
        guard.get(cell).map(f)
    }
}

/// The share of assessed traffic that fell in a competitive cell, folded from
/// a lifetime active-indicator tally (´tab:monitoring:dimension-health´).
///
/// The tally already separates the two populations the coverage row asks
/// about: the assessment path records one entry per assessment it routed
/// through the dimension, keyed by how many of the dimension's competitive
/// indicators fired, so the zero key counts exactly the assessments that
/// matched no cell and every other key counts an assessment that matched at
/// least one. The share is therefore the complement of the zero key's weight,
/// and no second counter is accumulated for it. That is deliberate rather than
/// merely economical: two counters incremented at one site can disagree after
/// a partial failure or a missed call, where one counter read two ways cannot,
/// and the shape row and the coverage row are then arithmetically consistent
/// by construction.
///
/// The reading is over the dimension's whole life, matching the tally it is
/// taken from and the shape rows it is reported beside. Windowing is already
/// represented on that surface by the churn reading, which is an EWMA
/// precisely because set membership turns over; coverage does not need a
/// second windowed accumulator and a second decay constant to say what the
/// lifetime share already says exactly.
///
/// `None` before any traffic has been assessed against the dimension: a
/// dimension nothing has arrived for has no share to report, and stays
/// distinguishable from one whose traffic arrived and missed the set
/// entirely, which reads zero.
#[must_use]
#[allow(clippy::cast_precision_loss)] // Justified: assessment counts stay far below f64's exact integer range.
pub fn competitive_coverage_fraction(distribution: &BTreeMap<usize, u64>) -> Option<f64> {
    let assessed: u64 = distribution.values().copied().sum();
    if assessed == 0 {
        return None;
    }
    let missed = distribution.get(&0).copied().unwrap_or(0);
    Some((assessed - missed) as f64 / assessed as f64)
}

impl std::fmt::Debug for IdentityDimensionInfra {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let competitive_len = self.competitive_set.load().len();
        let cell_count = self.cell_state.read().map_or(0, |g| g.len());

        f.debug_struct("IdentityDimensionInfra")
            .field("dimension_id", &self.dimension.id)
            .field("competitive_cells", &competitive_len)
            .field("tracked_cells", &cell_count)
            .field("observations_dropped", &self.observations_dropped())
            .field("active_indicator_counts", &self.active_indicator_count_distribution())
            .field("observation_tx", &self.observation_tx)
            .field("graph_snapshot", &self.graph_snapshot)
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::DimensionId;

    fn test_dimension() -> IdentityDimension {
        IdentityDimension::new(DimensionId(1), "test", "test hierarchy", "prefix groups", 128, 24, |_| 0)
    }

    /// A dimension's shared infrastructure starts blank in all three of its
    /// parts: an empty published competitive set, no per-cell state, and no
    /// observations recorded as dropped. Nothing about a dimension is inherited
    /// from whatever the process was doing before — its competitive geometry is
    /// built entirely from the traffic it goes on to see.
    ///
    /// ´claim:identity:a-fresh-dimension-starts-with-no-competitive-cells-no-cell-state-and-nothing-dropped´
    /// ´test:unit:identity-dimension-infra-construction´
    #[test]
    fn identity_dimension_infra_construction() {
        let (tx, _rx) = crossbeam_channel::bounded(100);
        let infra = IdentityDimensionInfra::new(test_dimension(), tx);

        assert!(infra.load_competitive_set().is_empty());
        assert!(infra.cell_state.read().unwrap().is_empty());
        assert_eq!(infra.observations_dropped(), 0);
    }

    /// Routing a coordinate through a dimension that has not yet published any
    /// competitive cells yields nothing rather than misbehaving. Assessment
    /// begins before the maintenance loop has learned anything, so this is the
    /// state the routing path meets first in every dimension's life.
    ///
    /// (´claim:identity:an-empty-competitive-set-matches-no-coordinate´)
    /// ´test:unit:identity-dimension-infra-find-active-empty´
    #[test]
    fn identity_dimension_infra_find_active_empty() {
        let (tx, _rx) = crossbeam_channel::bounded(100);
        let infra = IdentityDimensionInfra::new(test_dimension(), tx);

        let cells = infra.find_active_cells(0x12345);
        assert!(cells.is_empty());
    }

    /// Submitting an observation through a dimension's shared infrastructure
    /// reports success and leaves that exact coordinate waiting on the
    /// maintenance side. The assessment path's whole contribution to the
    /// identity layer travels this way, so the acknowledgement it gets has to
    /// mean the coordinate is genuinely in hand.
    ///
    /// (´claim:identity:an-observation-handed-over-by-an-assessment-thread-reaches-the-maintenance-side-unchanged´)
    /// ´test:unit:identity-dimension-infra-observe´
    #[test]
    fn identity_dimension_infra_observe() {
        let (tx, rx) = crossbeam_channel::bounded(10);
        let infra = IdentityDimensionInfra::new(test_dimension(), tx);

        // Observation should succeed
        assert!(infra.observe(0x00AB_CDEF));

        // Should be in the channel
        let coord = rx.try_recv().unwrap();
        assert_eq!(coord, 0x00AB_CDEF);
    }

    /// Once a dimension's queue is full, a further observation is refused
    /// rather than blocking the assessing thread, and the refusal is added to
    /// that dimension's dropped count. Assessment never waits on maintenance:
    /// under pressure the identity layer loses resolution, and says so, instead
    /// of imposing back-pressure on the request path.
    ///
    /// (´claim:identity:every-refused-observation-is-counted-so-shed-load-is-visible-rather-than-silent´)
    /// ´test:unit:identity-dimension-infra-observe-overflow´
    #[test]
    fn identity_dimension_infra_observe_overflow() {
        let (tx, _rx) = crossbeam_channel::bounded(2);
        let infra = IdentityDimensionInfra::new(test_dimension(), tx);

        // Fill the channel
        assert!(infra.observe(1));
        assert!(infra.observe(2));

        // Overflow
        assert!(!infra.observe(3));
        assert_eq!(infra.observations_dropped(), 1);
    }
}
