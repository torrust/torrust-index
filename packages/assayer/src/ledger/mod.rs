// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`create_sentinel_has_root`] | ledger | Registering a Sentinel hands back a ledger that already has its root in place, not an empty shell awaiting a second initialisation step. The container is the only thing that constructs per-Sentinel ledgers, so putting the root in at that moment means no caller can ever observe one in the unroutable state a bare construction leaves behind. |
//! | [`remove_sentinel`] | ledger | Removing a Sentinel hands its ledger back to the caller and leaves nothing registered under that identity. The state is surrendered rather than released inside the container, so letting the accumulated outcome memory go is the caller's own act at its own call site rather than something the ledger does quietly, and a later registration of the same id starts from nothing whatever the caller does with what it was handed. |
//! | [`snapshot_roundtrip`] | ledger | A snapshot captures every registered Sentinel and the contents of each one's ledger, and restoring it into a fresh container reproduces both the registrations and the recorded values. Outcome history is accumulated over months of traffic, so a checkpoint that lost a Sentinel — or kept the Sentinel but reset its root — would silently forfeit exactly the long-memory state the ledger exists to hold. |
//! | [`prune_axis_removes_rows_from_every_entry`] | ledger | Pruning an outcome axis reaches every entry of every Sentinel's ledger — roots and deeper cells alike — and removes only that axis's rows, leaving other axes' rows in place. Axis ids are reusable, so a re-registration under a recycled id would otherwise inherit the deregistered axis's EWMAs and read as history it never had. |
//! | [`prune_axis_is_idempotent_for_absent_axis`] | ledger | Pruning an axis that no entry ever recorded leaves the ledger exactly as it was — the unrelated axis row still stands. Deregistration events can arrive for axes that never received a value, or arrive twice, and the sweep tolerates both without the caller having to know which case it is in. |

//! Outcome Ledger — hierarchical outcome tracking.
//!
//! The Outcome Ledger maintains per-Sentinel outcome statistics at multiple
//! scales in a dyadic hierarchy. Each Sentinel has its own [`SentinelLedger`]
//! containing EWMA statistics at various depths.
//!
//! # Module Structure
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`entry`] | `LedgerEntry`, `DecayedView`, `LedgerUpdate`, `ImmaturityCriteria` |
//! | [`sentinel_ledger`] | `SentinelLedger` |
//! | [`routing`] | `read_route`, `update_all_layers` |
//! | [`cell_set`] | `apply_cell_set_changes`, `CellSetChanges` |
//! | [`gc`] | `garbage_collect`, `GcStats` |
//!
//! # Phase 1 Status
//!
//! Type definitions and operations are complete. Phase 2 adds:
//! - Integration with `label()` path
//! - Cell-set maintenance triggered by Sentinel reports
//!
//! # Cross-References
//!
//! - (´def:ledger:purpose´) — what the Outcome Ledger is
//! - (´chap:spec:outcome-ledger´) — the chapter that specifies it
//! - (´dec:memory:coordinate-depth-key´) — why entries are keyed by
//!   coordinate and depth together

// Phase 1: All types are used in Phase 2's label() path and cell-set maintenance.
#![allow(dead_code)]

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::types::{OutcomeAxisId, SentinelId};

pub mod cell_set;
pub mod entry;
pub mod gc;
pub mod routing;
pub mod scheduler;
pub mod sentinel_ledger;

// Re-exports for convenience
// Phase 1: These are used in Phase 2's ingestion pipeline and label() path.
#[allow(unused_imports)] // Phase 1: used in Phase 2
pub use cell_set::{CellSetChanges, apply_cell_set_changes, reset_absent_counters};
pub use entry::{DecayedView, ImmaturityCriteria, LedgerEntry, LedgerUpdate};
#[allow(unused_imports)] // Phase 1: used in Phase 2
pub use gc::{GcLimits, GcOutcome, GcStats, garbage_collect, garbage_collect_batched, garbage_collect_with_limits, gc_stats};
#[allow(unused_imports)] // Phase 1: used in Phase 2
pub use routing::{keys_containing, read_route, read_route_key, update_all_layers};
pub use scheduler::{SWEEP_INTERVAL, spawn_ledger_gc_scheduler};
pub use sentinel_ledger::SentinelLedger;

/// Sweep parameters read from the Ledger configuration
/// (´alg:ledger:garbage-collection´).
#[derive(Clone, Copy, Debug)]
pub struct SweepConfig {
    /// Minimum EWMA magnitude to retain an entry.
    pub floor: f64,
    /// Maximum age since last update.
    pub horizon: std::time::Duration,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Ledger
// ═══════════════════════════════════════════════════════════════════════════════

/// Container for all per-Sentinel outcome ledgers.
///
/// Each Sentinel has its own [`SentinelLedger`] stored behind a [`RwLock`]
/// for concurrent read access with exclusive write access.
///
/// # Thread Safety
///
/// - Container mutations acquire a short write lock on the Sentinel map.
/// - Readers clone the per-Sentinel ledger handle under a short read lock.
/// - Ledger readers/writers acquire the specific Sentinel's ledger lock.
#[derive(Debug, Default)]
pub struct OutcomeLedger {
    /// Per-Sentinel ledgers with read-write locks.
    ledgers: RwLock<HashMap<SentinelId, Arc<RwLock<SentinelLedger>>>>,
}

impl OutcomeLedger {
    /// Creates an empty outcome ledger.
    #[must_use]
    pub fn new() -> Self {
        Self {
            ledgers: RwLock::new(HashMap::new()),
        }
    }

    /// Creates a new Sentinel ledger.
    ///
    /// Initialises the ledger with a root entry. If the Sentinel already has a
    /// ledger this is a no-op, preserving any reports ingested during Phase 1
    /// before the model-owner lifecycle event is processed.
    pub fn create_sentinel(&self, id: SentinelId) {
        let mut ledgers = self.ledgers.write().expect("ledger map RwLock poisoned");
        ledgers.entry(id).or_insert_with(|| {
            let mut ledger = SentinelLedger::new();
            ledger.ensure_root();
            Arc::new(RwLock::new(ledger))
        });
    }

    /// Removes a Sentinel ledger.
    ///
    /// Returns the removed ledger if present.
    pub fn remove_sentinel(&self, id: SentinelId) -> Option<SentinelLedger> {
        let removed = self.ledgers.write().expect("ledger map RwLock poisoned").remove(&id)?;
        Arc::try_unwrap(removed)
            .ok()
            .map(|lock| lock.into_inner().expect("RwLock poisoned"))
    }

    /// Returns the ledger for a Sentinel, if present.
    #[must_use]
    pub fn get(&self, id: SentinelId) -> Option<Arc<RwLock<SentinelLedger>>> {
        self.ledgers.read().expect("ledger map RwLock poisoned").get(&id).cloned()
    }

    /// Returns a cloned `Arc` to the ledger for a Sentinel, if present.
    #[must_use]
    pub fn get_arc(&self, id: SentinelId) -> Option<Arc<RwLock<SentinelLedger>>> {
        self.get(id)
    }

    /// Returns `true` if a Sentinel ledger exists.
    #[must_use]
    pub fn contains(&self, id: SentinelId) -> bool {
        self.ledgers.read().expect("ledger map RwLock poisoned").contains_key(&id)
    }

    /// Returns the number of Sentinel ledgers.
    #[must_use]
    pub fn len(&self) -> usize {
        self.ledgers.read().expect("ledger map RwLock poisoned").len()
    }

    /// Returns `true` if there are no Sentinel ledgers.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.ledgers.read().expect("ledger map RwLock poisoned").is_empty()
    }

    /// Returns the registered Sentinel IDs.
    #[must_use]
    pub fn sentinel_ids(&self) -> Vec<SentinelId> {
        self.ledgers
            .read()
            .expect("ledger map RwLock poisoned")
            .keys()
            .copied()
            .collect()
    }

    /// Removes per-axis EWMA rows for `axis_id` from every entry of every
    /// Sentinel ledger.
    ///
    /// Called by the model owner when handling [`LifecycleEvent::DeregisterOutcomeAxis`]
    /// so that a subsequent re-registration of the same `OutcomeAxisId` cannot
    /// inherit stale EWMAs. Dropping the rows is the whole of step 4 of the
    /// deregistration protocol: the averages end with the axis and are not kept
    /// anywhere for a later return (´alg:registry:axis-deregistration´). No
    /// state transfers between entries (´just:ledger:no-state-transfer´).
    ///
    /// Acquires each per-Sentinel write lock sequentially. At 20K entries × 8
    /// Sentinels the total cost is approximately 16 ms, spread across 8 short
    /// write-lock holds (~2 ms each) — well inside the loop's budget
    /// (´tab:ledger:loop-timescales´).
    ///
    /// [`LifecycleEvent::DeregisterOutcomeAxis`]: crate::owner::commands::LifecycleEvent::DeregisterOutcomeAxis
    pub fn prune_axis(&self, axis_id: OutcomeAxisId) {
        let ledgers: Vec<_> = self
            .ledgers
            .read()
            .expect("ledger map RwLock poisoned")
            .values()
            .cloned()
            .collect();

        for lock in ledgers {
            let mut guard = lock.write().expect("ledger RwLock poisoned");
            for entry in guard.entries_mut().values_mut() {
                entry.per_axis.retain(|(id, _, _)| *id != axis_id);
            }
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serialisation Support
// ═══════════════════════════════════════════════════════════════════════════════

/// Serialisable snapshot of the Outcome Ledger.
///
/// Used for checkpoint persistence.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct OutcomeLedgerSnapshot {
    /// Per-Sentinel ledger snapshots.
    pub ledgers: HashMap<SentinelId, SentinelLedger>,
}

impl OutcomeLedger {
    /// Creates a snapshot of all ledgers.
    ///
    /// Acquires read locks on all ledgers.
    #[must_use]
    pub fn snapshot(&self) -> OutcomeLedgerSnapshot {
        let ledger_handles = {
            let guard = self.ledgers.read().expect("ledger map RwLock poisoned");
            let mut handles = Vec::with_capacity(guard.len());
            for (&id, lock) in guard.iter() {
                handles.push((id, Arc::clone(lock)));
            }
            drop(guard);
            handles
        };

        let mut ledgers = HashMap::with_capacity(ledger_handles.len());
        for (id, lock) in ledger_handles {
            let ledger = lock.read().expect("RwLock poisoned").clone();
            ledgers.insert(id, ledger);
        }

        OutcomeLedgerSnapshot { ledgers }
    }

    /// Restores from a snapshot.
    ///
    /// Replaces all existing ledgers.
    pub fn restore(&mut self, snapshot: OutcomeLedgerSnapshot) {
        let ledgers = self.ledgers.get_mut().expect("ledger map RwLock poisoned");
        ledgers.clear();
        for (id, ledger) in snapshot.ledgers {
            ledgers.insert(id, Arc::new(RwLock::new(ledger)));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Registering a Sentinel hands back a ledger that already has its root in
    /// place, not an empty shell awaiting a second initialisation step. The
    /// container is the only thing that constructs per-Sentinel ledgers, so
    /// putting the root in at that moment means no caller can ever observe one
    /// in the unroutable state a bare construction leaves behind.
    ///
    /// ´claim:ledger:a-sentinels-ledger-is-created-with-its-root-already-in-place´
    /// ´test:unit:create-sentinel-has-root´
    #[test]
    fn create_sentinel_has_root() {
        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(SentinelId(1));

        let lock = ledger.get(SentinelId(1)).unwrap();
        let guard = lock.read().unwrap();
        assert!(guard.has_root());
        drop(guard);
    }

    /// Removing a Sentinel hands its ledger back to the caller and leaves
    /// nothing registered under that identity. The state is surrendered rather
    /// than released inside the container, so letting the accumulated outcome
    /// memory go is the caller's own act at its own call site rather than
    /// something the ledger does quietly, and a later registration of the same
    /// id starts from nothing whatever the caller does with what it was handed.
    ///
    /// ´claim:ledger:removing-a-sentinel-yields-its-ledger-and-leaves-nothing-registered´
    /// ´test:unit:remove-sentinel´
    #[test]
    fn remove_sentinel() {
        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(SentinelId(1));

        let removed = ledger.remove_sentinel(SentinelId(1));
        assert!(removed.is_some());
        assert!(!ledger.contains(SentinelId(1)));
    }

    /// A snapshot captures every registered Sentinel and the contents of each
    /// one's ledger, and restoring it into a fresh container reproduces both the
    /// registrations and the recorded values. Outcome history is accumulated
    /// over months of traffic, so a checkpoint that lost a Sentinel — or kept
    /// the Sentinel but reset its root — would silently forfeit exactly the
    /// long-memory state the ledger exists to hold.
    ///
    /// ´claim:ledger:a-snapshot-restores-every-registered-sentinel-with-its-recorded-state´
    /// ´test:unit:snapshot-roundtrip´
    #[test]
    fn snapshot_roundtrip() {
        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(SentinelId(1));
        ledger.create_sentinel(SentinelId(2));

        // Modify a ledger
        {
            let lock = ledger.get(SentinelId(1)).unwrap();
            let mut guard = lock.write().unwrap();
            guard.root_mut().total_assessments = 42;
        }

        let snapshot = ledger.snapshot();

        // Create a new ledger and restore
        let mut restored = OutcomeLedger::new();
        restored.restore(snapshot);

        assert!(restored.contains(SentinelId(1)));
        assert!(restored.contains(SentinelId(2)));

        let lock = restored.get(SentinelId(1)).unwrap();
        let guard = lock.read().unwrap();
        assert_eq!(guard.root().total_assessments, 42);
        drop(guard);
    }

    /// Pruning an outcome axis reaches every entry of every Sentinel's ledger —
    /// roots and deeper cells alike — and removes only that axis's rows,
    /// leaving other axes' rows in place. Axis ids are reusable, so a
    /// re-registration under a recycled id would otherwise inherit the
    /// deregistered axis's EWMAs and read as history it never had.
    ///
    /// ´claim:ledger:pruning-an-axis-strips-its-rows-from-every-entry-and-leaves-other-axes-alone´
    /// ´test:unit:prune-axis-removes-rows-from-every-entry´
    #[test]
    fn prune_axis_removes_rows_from_every_entry() {
        use crate::ledger::entry::LedgerEntry;
        use crate::types::LedgerKey;

        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(SentinelId(1));
        ledger.create_sentinel(SentinelId(2));

        // Seed per-axis rows for two axes across both Sentinels,
        // in both the root and a depth-8 entry.
        let axis_to_prune = OutcomeAxisId(7);
        let axis_to_keep = OutcomeAxisId(11);

        for sid in [SentinelId(1), SentinelId(2)] {
            let lock = ledger.get(sid).unwrap();
            let mut guard = lock.write().unwrap();
            guard.root_mut().per_axis.push((axis_to_prune, 0.5, 0.25));
            guard.root_mut().per_axis.push((axis_to_keep, 0.9, 0.4));

            let child_key = LedgerKey::new(0, 8);
            let mut child = LedgerEntry::new_neutral();
            child.per_axis.push((axis_to_prune, 0.1, 0.2));
            child.per_axis.push((axis_to_keep, 0.3, 0.4));
            guard.insert(child_key, child);
        }

        ledger.prune_axis(axis_to_prune);

        for sid in [SentinelId(1), SentinelId(2)] {
            let lock = ledger.get(sid).unwrap();
            let guard = lock.read().unwrap();
            for entry in guard.entries().values() {
                assert!(
                    !entry.per_axis.iter().any(|(id, _, _)| *id == axis_to_prune),
                    "pruned axis row must not survive on sentinel {sid:?}"
                );
                assert!(
                    entry.per_axis.iter().any(|(id, _, _)| *id == axis_to_keep),
                    "untouched axis row must remain on sentinel {sid:?}"
                );
            }
        }
    }

    /// Pruning an axis that no entry ever recorded leaves the ledger exactly as
    /// it was — the unrelated axis row still stands. Deregistration events can
    /// arrive for axes that never received a value, or arrive twice, and the
    /// sweep tolerates both without the caller having to know which case it is
    /// in.
    ///
    /// ´claim:ledger:pruning-an-axis-no-entry-recorded-changes-nothing´
    /// ´test:unit:prune-axis-is-idempotent-for-absent-axis´
    #[test]
    fn prune_axis_is_idempotent_for_absent_axis() {
        let ledger = OutcomeLedger::new();
        ledger.create_sentinel(SentinelId(1));

        {
            let lock = ledger.get(SentinelId(1)).unwrap();
            let mut guard = lock.write().unwrap();
            guard.root_mut().per_axis.push((OutcomeAxisId(1), 0.5, 0.5));
        }

        // Pruning an unrelated axis leaves state untouched.
        ledger.prune_axis(OutcomeAxisId(999));

        let lock = ledger.get(SentinelId(1)).unwrap();
        let guard = lock.read().unwrap();
        assert_eq!(guard.root().per_axis.len(), 1);
        drop(guard);
    }
}
