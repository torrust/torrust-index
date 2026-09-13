// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`gc_removes_old_low_ewma_entries`] | ledger | Collection takes an entry only when it is both stale and empty: an entry last touched a hundred days ago with every EWMA at zero, against a ninety-day horizon, is removed. Such an entry can only ever answer a read with the neutral values a fallback ancestor would give anyway, so keeping it costs memory and buys nothing. |
//! | [`gc_keeps_entry_with_high_ewma`] | ledger | Age alone is not grounds for collection: an entry a hundred days old whose bad-rate still sits well above the floor survives untouched. Stored EWMAs decay lazily at read time rather than in place, so an old entry may still carry real weight, and collecting it would discard evidence the read path would have honoured. |
//! | [`gc_keeps_recent_entry`] | ledger | Emptiness alone is not grounds either: an entry touched a day ago with all EWMAs at zero stays. A cell whose recent traffic has simply been benign reads as empty, and collecting it would throw away a live cell only for it to be recreated from the next report — churning the ledger exactly where activity is highest. |
//! | [`gc_never_removes_root`] | ledger | The root is outside the collector's reach: made ancient and left empty, then swept with a one-second horizon that would condemn any other entry, it survives and nothing is reported removed. Routing has no destination below the root, so its permanence is a structural invariant rather than a retention preference. |
//! | [`gc_respects_per_axis_ewma`] | ledger | Emptiness is judged across every EWMA an entry holds, per-axis rows included: an old entry whose entry-wide figures are all zero but which carries one axis row above the floor is retained. A host may care only about a registered outcome axis, so an entry whose sole content lives on such an axis is still the answer to somebody's read. |
//! | [`gc_stats_counts_correctly`] | ledger | The statistics pass reports what collection would do without doing it, and separates the two conditions rather than only their conjunction: across an old-and-empty, an old-but-loud and a recent-but-empty entry it counts two below the floor, two beyond the horizon, and just one actually eligible. An operator watching the ledger grow can therefore tell whether the horizon or the floor is the binding constraint before changing either. |

//! Garbage collection for the Outcome Ledger.
//!
//! This module provides time-based garbage collection for stale ledger
//! entries. Entries are removed when:
//!
//! 1. All EWMA values are below a floor threshold
//! 2. The entry has not been updated within a horizon period
//!
//! The root entry is never garbage collected.
//!
//! # Cross-References
//!
//! - (´alg:ledger:garbage-collection´) — the floor and the horizon this
//!   applies
//! - (´dec:memory:root-permanence´) — why the root is never collected

use std::sync::RwLock;
use std::time::Duration;

use super::entry::LedgerEntry;
use super::sentinel_ledger::SentinelLedger;
use crate::types::{LedgerKey, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// Garbage Collection
// ═══════════════════════════════════════════════════════════════════════════════

/// Default maximum entries inspected under one read-lock acquisition.
///
/// The size of one read-side piece of the sweep rather than a limit on the
/// sweep: the collector reacquires and continues until every snapshotted key is
/// inspected (´dec:concurrency:bounded-traversal´). It is a default a caller may
/// replace, and the larger of the collector's two figures because a read
/// acquisition excludes only writers.
///
/// ´const:assayer:collection-scan-bound´ (´alg:const:count´)
/// ´const:assayer:collection-scan-bound-count-512´
pub const DEFAULT_GC_SCAN_LIMIT: usize = 512;

/// Default maximum entries deleted under one write-lock acquisition.
///
/// The size of one write-side piece of the sweep, and the smaller of the
/// collector's two figures because a write acquisition excludes every reader
/// rather than only the writers (´dec:concurrency:bounded-traversal´). Reaching
/// it flushes the batch and takes a fresh acquisition; it is a default a caller
/// may replace.
///
/// ´const:assayer:collection-deletion-bound´ (´alg:const:count´)
/// ´const:assayer:collection-deletion-bound-count-64´
pub const DEFAULT_GC_DELETE_BATCH_LIMIT: usize = 64;

/// Batching limits for Ledger garbage collection.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GcLimits {
    /// Maximum entries inspected under one read-lock acquisition.
    pub scan_limit: usize,
    /// Maximum entries deleted under one write-lock acquisition.
    pub delete_batch_limit: usize,
}

impl GcLimits {
    /// Creates a new GC limit bundle.
    #[must_use]
    pub const fn new(scan_limit: usize, delete_batch_limit: usize) -> Self {
        Self {
            scan_limit,
            delete_batch_limit,
        }
    }

    /// Returns a non-zero scan limit.
    const fn effective_scan_limit(self) -> usize {
        if self.scan_limit == 0 { 1 } else { self.scan_limit }
    }

    /// Returns a non-zero delete batch limit.
    const fn effective_delete_batch_limit(self) -> usize {
        if self.delete_batch_limit == 0 {
            1
        } else {
            self.delete_batch_limit
        }
    }
}

impl Default for GcLimits {
    fn default() -> Self {
        Self {
            scan_limit: DEFAULT_GC_SCAN_LIMIT,
            delete_batch_limit: DEFAULT_GC_DELETE_BATCH_LIMIT,
        }
    }
}

/// Outcome telemetry from a batched GC sweep.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct GcOutcome {
    /// Number of entries removed.
    pub removed: usize,
    /// Number of entries inspected during read-side scans.
    pub scanned_entries: usize,
    /// Number of read-lock acquisitions used for scan windows.
    pub read_lock_acquisitions: usize,
    /// Number of write-lock acquisitions used for deletion batches.
    pub write_lock_acquisitions: usize,
    /// Largest read-side scan window observed.
    pub max_scan_window: usize,
    /// Largest write-side deletion batch observed.
    pub max_delete_batch: usize,
}

impl GcOutcome {
    /// Records one read-side scan window.
    fn record_read_window(&mut self, window_len: usize) {
        self.read_lock_acquisitions += 1;
        self.scanned_entries += window_len;
        self.max_scan_window = self.max_scan_window.max(window_len);
    }

    /// Records one write-side deletion batch.
    fn record_delete_batch(&mut self, batch_len: usize) {
        self.write_lock_acquisitions += 1;
        self.max_delete_batch = self.max_delete_batch.max(batch_len);
    }
}

/// Garbage collects stale entries from the ledger.
///
/// Removes non-root entries where:
/// - All EWMA values are below `ewma_floor`
/// - `last_updated` is older than `gc_horizon` from `now`
///
/// # Arguments
///
/// * `ledger` — The Sentinel ledger to clean
/// * `ewma_floor` — Minimum EWMA magnitude to retain an entry
/// * `gc_horizon` — Maximum age since last update
/// * `now` — Current timestamp
///
/// # Returns
///
/// The number of entries removed.
///
/// # Invariants
///
/// - The root entry (depth 0) is never removed.
pub fn garbage_collect(ledger: &mut SentinelLedger, ewma_floor: f64, gc_horizon: Duration, now: &PersistentTimestamp) -> usize {
    garbage_collect_with_limits(ledger, GcLimits::default(), ewma_floor, gc_horizon, now).removed
}

/// Garbage collects stale entries from an already-borrowed ledger using bounded batches.
///
/// This compatibility helper preserves the historical `&mut SentinelLedger`
/// call shape. Lock-aware callers should prefer [`garbage_collect_batched`]
/// so read scans and write deletions are separated by actual lock acquisition.
#[must_use]
pub fn garbage_collect_with_limits(
    ledger: &mut SentinelLedger,
    limits: GcLimits,
    ewma_floor: f64,
    gc_horizon: Duration,
    now: &PersistentTimestamp,
) -> GcOutcome {
    let cutoff = cutoff_timestamp(gc_horizon, now);
    let scan_limit = limits.effective_scan_limit();
    let delete_batch_limit = limits.effective_delete_batch_limit();

    let mut outcome = GcOutcome::default();
    let mut scanned_survivors = 0usize;
    let mut pending_deletes = Vec::with_capacity(delete_batch_limit);

    loop {
        let window_candidates = scan_candidate_window(ledger, scanned_survivors, scan_limit, ewma_floor, &cutoff);
        if window_candidates.window_len == 0 {
            break;
        }

        outcome.record_read_window(window_candidates.window_len);
        scanned_survivors += window_candidates.window_len;

        for key in window_candidates.candidates {
            pending_deletes.push(key);
            if pending_deletes.len() == delete_batch_limit {
                outcome.record_delete_batch(pending_deletes.len());
                let removed = delete_candidate_batch(ledger, &pending_deletes, ewma_floor, &cutoff);
                outcome.removed += removed;
                scanned_survivors = scanned_survivors.saturating_sub(removed);
                pending_deletes.clear();
            }
        }
    }

    if !pending_deletes.is_empty() {
        outcome.record_delete_batch(pending_deletes.len());
        let removed = delete_candidate_batch(ledger, &pending_deletes, ewma_floor, &cutoff);
        outcome.removed += removed;
    }

    outcome
}

/// Garbage collects stale entries with bounded read and write lock holds.
///
/// The key set is snapshotted under one initial read lock and consumed
/// across windows, so candidate discovery survives the rehashing a
/// concurrent insertion may cause between lock acquisitions — a sweep
/// must scan the Ledger (´alg:ledger:garbage-collection´), and a
/// positional cursor into hash-map iteration order does not survive a
/// resize. Each subsequent read-lock acquisition inspects at most
/// `limits.scan_limit` snapshotted keys. Each write-lock acquisition
/// deletes at most `limits.delete_batch_limit` entries, revalidating
/// every candidate under the write lock so entries that were refreshed
/// or removed concurrently are skipped.
#[must_use]
pub fn garbage_collect_batched(
    ledger: &RwLock<SentinelLedger>,
    limits: GcLimits,
    ewma_floor: f64,
    gc_horizon: Duration,
    now: &PersistentTimestamp,
) -> GcOutcome {
    let cutoff = cutoff_timestamp(gc_horizon, now);
    let scan_limit = limits.effective_scan_limit();
    let delete_batch_limit = limits.effective_delete_batch_limit();

    let mut outcome = GcOutcome::default();

    // One key-snapshot acquisition; keys are cloned without evaluating
    // any entry predicate, so it is counted as a lock acquisition but
    // not as a scan window.
    let keys: Vec<LedgerKey> = {
        let guard = ledger.read().expect("ledger RwLock poisoned");
        outcome.read_lock_acquisitions += 1;
        guard.entries().keys().copied().collect()
    };

    let mut pending_deletes = Vec::with_capacity(delete_batch_limit);

    for window in keys.chunks(scan_limit) {
        let candidates = {
            let guard = ledger.read().expect("ledger RwLock poisoned");
            let mut inspected = 0usize;
            let mut candidates = Vec::new();
            for key in window {
                // Revalidated by key: a key removed since the snapshot
                // is skipped rather than misdirecting the window.
                if let Some(entry) = guard.get(key) {
                    inspected += 1;
                    if is_gc_candidate(*key, entry, ewma_floor, &cutoff) {
                        candidates.push(*key);
                    }
                }
            }
            outcome.record_read_window(inspected);
            candidates
        };

        for key in candidates {
            pending_deletes.push(key);
            if pending_deletes.len() == delete_batch_limit {
                outcome.record_delete_batch(pending_deletes.len());
                let removed = {
                    let mut guard = ledger.write().expect("ledger RwLock poisoned");
                    delete_candidate_batch(&mut guard, &pending_deletes, ewma_floor, &cutoff)
                };
                outcome.removed += removed;
                pending_deletes.clear();
            }
        }
    }

    if !pending_deletes.is_empty() {
        outcome.record_delete_batch(pending_deletes.len());
        let removed = {
            let mut guard = ledger.write().expect("ledger RwLock poisoned");
            delete_candidate_batch(&mut guard, &pending_deletes, ewma_floor, &cutoff)
        };
        outcome.removed += removed;
    }

    outcome
}

/// Candidate keys found in one bounded scan window.
struct CandidateWindow {
    /// Number of live entries inspected in the window.
    window_len: usize,
    /// Entries eligible for revalidation and possible deletion.
    candidates: Vec<LedgerKey>,
}

/// Computes the oldest retained timestamp for the GC horizon.
fn cutoff_timestamp(gc_horizon: Duration, now: &PersistentTimestamp) -> PersistentTimestamp {
    // Use saturating conversion to avoid overflow on very long horizons.
    let horizon_seconds = i64::try_from(gc_horizon.as_secs()).unwrap_or(i64::MAX);
    let cutoff_seconds = now.seconds.saturating_sub(horizon_seconds);
    PersistentTimestamp::new(cutoff_seconds, 0)
}

/// Scans one bounded live-entry window and returns deletion candidates.
fn scan_candidate_window(
    ledger: &SentinelLedger,
    start: usize,
    scan_limit: usize,
    ewma_floor: f64,
    cutoff: &PersistentTimestamp,
) -> CandidateWindow {
    let root_key = LedgerKey::new(0, 0);
    let mut window_len = 0usize;
    let mut candidates = Vec::new();

    for (&key, entry) in ledger.entries().iter().skip(start).take(scan_limit) {
        window_len += 1;
        if key != root_key && is_gc_candidate(key, entry, ewma_floor, cutoff) {
            candidates.push(key);
        }
    }

    CandidateWindow { window_len, candidates }
}

/// Deletes one candidate batch after revalidating every key.
fn delete_candidate_batch(
    ledger: &mut SentinelLedger,
    candidates: &[LedgerKey],
    ewma_floor: f64,
    cutoff: &PersistentTimestamp,
) -> usize {
    let mut removed = 0usize;
    let previous_max_depth = ledger.max_depth();
    let mut removed_at_max_depth = false;

    for key in candidates {
        let should_remove = ledger
            .get(key)
            .is_some_and(|entry| is_gc_candidate(*key, entry, ewma_floor, cutoff));

        if should_remove && ledger.remove(key).is_some() {
            removed += 1;
            removed_at_max_depth |= key.depth == previous_max_depth;
        }
    }

    if removed_at_max_depth {
        ledger.recalculate_max_depth();
    }

    removed
}

/// Returns whether one ledger entry is currently eligible for GC.
fn is_gc_candidate(key: LedgerKey, entry: &LedgerEntry, ewma_floor: f64, cutoff: &PersistentTimestamp) -> bool {
    key.depth != 0 && entry.all_ewmas_below(ewma_floor) && entry.last_updated < *cutoff
}

/// Computes GC statistics without removing anything.
///
/// Useful for monitoring and reporting.
#[derive(Clone, Debug, Default)]
pub struct GcStats {
    /// Total entries in the ledger.
    pub total_entries: usize,
    /// Entries that would be removed by GC.
    pub eligible_for_gc: usize,
    /// Entries with all EWMAs below floor.
    pub below_floor: usize,
    /// Entries older than horizon.
    pub older_than_horizon: usize,
}

/// Computes GC statistics without modifying the ledger.
#[must_use]
pub fn gc_stats(ledger: &SentinelLedger, ewma_floor: f64, gc_horizon: Duration, now: &PersistentTimestamp) -> GcStats {
    let root_key = LedgerKey::new(0, 0);
    // Use saturating conversion to avoid overflow on very long horizons
    let horizon_seconds = i64::try_from(gc_horizon.as_secs()).unwrap_or(i64::MAX);
    let cutoff_seconds = now.seconds.saturating_sub(horizon_seconds);
    let cutoff = PersistentTimestamp::new(cutoff_seconds, 0);

    let mut below_floor = 0;
    let mut older_than_horizon = 0;
    let mut eligible_for_gc = 0;

    for (&key, entry) in ledger.entries() {
        if key == root_key {
            continue;
        }

        let is_below = entry.all_ewmas_below(ewma_floor);
        let is_old = entry.last_updated < cutoff;

        if is_below {
            below_floor += 1;
        }
        if is_old {
            older_than_horizon += 1;
        }
        if is_below && is_old {
            eligible_for_gc += 1;
        }
    }

    GcStats {
        total_entries: ledger.entry_count(),
        eligible_for_gc,
        below_floor,
        older_than_horizon,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ledger::entry::LedgerEntry;

    fn make_key(lo: u128, depth: u8) -> LedgerKey {
        LedgerKey::new(lo, depth)
    }

    fn old_timestamp() -> PersistentTimestamp {
        // 100 days ago
        let now = PersistentTimestamp::now();
        PersistentTimestamp::new(now.seconds - 100 * 24 * 3600, 0)
    }

    fn recent_timestamp() -> PersistentTimestamp {
        // 1 day ago
        let now = PersistentTimestamp::now();
        PersistentTimestamp::new(now.seconds - 24 * 3600, 0)
    }

    /// Collection takes an entry only when it is both stale and empty: an entry
    /// last touched a hundred days ago with every EWMA at zero, against a
    /// ninety-day horizon, is removed. Such an entry can only ever answer a
    /// read with the neutral values a fallback ancestor would give anyway, so
    /// keeping it costs memory and buys nothing.
    ///
    /// ´claim:ledger:an-entry-that-is-both-stale-and-empty-is-collected´
    /// ´test:unit:gc-removes-old-low-ewma-entries´
    #[test]
    fn gc_removes_old_low_ewma_entries() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add an old entry with zero EWMAs
        let key4 = make_key(0, 4);
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = old_timestamp();
        ledger.insert(key4, entry);

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(90 * 24 * 3600); // 90 days

        let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
        assert_eq!(removed, 1);
        assert!(ledger.get(&key4).is_none());
    }

    /// Age alone is not grounds for collection: an entry a hundred days old
    /// whose bad-rate still sits well above the floor survives untouched.
    /// Stored EWMAs decay lazily at read time rather than in place, so an old
    /// entry may still carry real weight, and collecting it would discard
    /// evidence the read path would have honoured.
    ///
    /// ´claim:ledger:an-entry-still-carrying-signal-survives-collection-however-old-it-is´
    /// ´test:unit:gc-keeps-entry-with-high-ewma´
    #[test]
    fn gc_keeps_entry_with_high_ewma() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add an old entry with non-zero EWMA
        let key4 = make_key(0, 4);
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = old_timestamp();
        entry.ewma_bad_rate = 0.5; // Above floor
        ledger.insert(key4, entry);

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(90 * 24 * 3600);

        let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
        assert_eq!(removed, 0);
        assert!(ledger.get(&key4).is_some());
    }

    /// Emptiness alone is not grounds either: an entry touched a day ago with
    /// all EWMAs at zero stays. A cell whose recent traffic has simply been
    /// benign reads as empty, and collecting it would throw away a live cell
    /// only for it to be recreated from the next report — churning the ledger
    /// exactly where activity is highest.
    ///
    /// ´claim:ledger:a-recently-touched-entry-survives-collection-however-empty-it-is´
    /// ´test:unit:gc-keeps-recent-entry´
    #[test]
    fn gc_keeps_recent_entry() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add a recent entry with zero EWMAs
        let key4 = make_key(0, 4);
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = recent_timestamp();
        ledger.insert(key4, entry);

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(90 * 24 * 3600);

        let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
        assert_eq!(removed, 0);
        assert!(ledger.get(&key4).is_some());
    }

    /// The root is outside the collector's reach: made ancient and left empty,
    /// then swept with a one-second horizon that would condemn any other entry,
    /// it survives and nothing is reported removed. Routing has no destination
    /// below the root, so its permanence is a structural invariant rather than
    /// a retention preference.
    ///
    /// ´claim:ledger:the-root-entry-is-never-garbage-collected´
    /// ´test:unit:gc-never-removes-root´
    #[test]
    fn gc_never_removes_root() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Make root old
        ledger.root_mut().last_updated = old_timestamp();

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(1); // Very short horizon

        let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
        assert_eq!(removed, 0);
        assert!(ledger.has_root());
    }

    /// Emptiness is judged across every EWMA an entry holds, per-axis rows
    /// included: an old entry whose entry-wide figures are all zero but which
    /// carries one axis row above the floor is retained. A host may care only
    /// about a registered outcome axis, so an entry whose sole content lives on
    /// such an axis is still the answer to somebody's read.
    ///
    /// ´claim:ledger:a-per-axis-ewma-above-the-floor-is-enough-to-retain-an-entry´
    /// ´test:unit:gc-respects-per-axis-ewma´
    #[test]
    fn gc_respects_per_axis_ewma() {
        use crate::types::OutcomeAxisId;

        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add an old entry with only per-axis EWMA above floor
        let key4 = make_key(0, 4);
        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = old_timestamp();
        entry.per_axis.push((OutcomeAxisId(1), 0.5, 0.0));
        ledger.insert(key4, entry);

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(90 * 24 * 3600);

        let removed = garbage_collect(&mut ledger, 1e-6, horizon, &now);
        assert_eq!(removed, 0);
        assert!(ledger.get(&key4).is_some());
    }

    /// The statistics pass reports what collection would do without doing it,
    /// and separates the two conditions rather than only their conjunction:
    /// across an old-and-empty, an old-but-loud and a recent-but-empty entry it
    /// counts two below the floor, two beyond the horizon, and just one
    /// actually eligible. An operator watching the ledger grow can therefore
    /// tell whether the horizon or the floor is the binding constraint before
    /// changing either.
    ///
    /// ´claim:ledger:the-statistics-pass-reports-what-collection-would-do-without-doing-it´
    /// ´test:unit:gc-stats-counts-correctly´
    #[test]
    fn gc_stats_counts_correctly() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Add entries in various states
        let key_old_low = make_key(0, 4);
        let key_old_high = make_key(0, 8);
        let key_recent_low = make_key(0, 12);

        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = old_timestamp();
        ledger.insert(key_old_low, entry.clone());

        entry.ewma_bad_rate = 0.5;
        ledger.insert(key_old_high, entry);

        let mut entry = LedgerEntry::new_neutral();
        entry.last_updated = recent_timestamp();
        ledger.insert(key_recent_low, entry);

        let now = PersistentTimestamp::now();
        let horizon = Duration::from_secs(90 * 24 * 3600);

        let stats = gc_stats(&ledger, 1e-6, horizon, &now);

        assert_eq!(stats.total_entries, 4); // root + 3
        assert_eq!(stats.below_floor, 2); // old_low and recent_low
        assert_eq!(stats.older_than_horizon, 2); // old_low and old_high
        assert_eq!(stats.eligible_for_gc, 1); // only old_low
    }
}
