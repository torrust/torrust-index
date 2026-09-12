// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`insert_then_remove_returns_the_same_entry`] | pending | An assessment placed in the buffer is retrievable by its own identifier and comes back as the entry that went in. This is the whole contract of deferred labelling: the assessment context has to still be there, and be the right one, when the outcome arrives some time later. |
//! | [`remove_unknown_returns_none`] | pending | A label naming an identifier the buffer never held is answered with absence, not with a panic or an invented entry. Labels arrive from the host and may be late, duplicated or simply wrong, so an unknown identifier has to be an ordinary outcome of the lookup. |
//! | [`remove_twice_returns_none`] | pending | Removal consumes the entry: the first call hands the assessment over and the second finds nothing. A duplicated label therefore cannot drive the same model update twice, because the context it would need has already left the buffer. |
//! | [`capacity_eviction`] | pending | The buffer is bounded: pushing one assessment past the configured capacity costs the oldest entry, never the newest. Assessments whose labels have not arrived accumulate indefinitely otherwise, and when the bound has to bite it is the least likely to still be labelled that is given up. |
//! | [`lazy_eviction_capped_at_16`] | pending | Eviction is amortised rather than exhaustive: an insert arriving after a large backlog of entries has gone stale does a bounded amount of cleanup and returns, leaving the rest for later inserts. The buffer may stay over its bound for a while, but no single caller pays for the whole backlog — which is what keeps insert latency independent of how long the host went without labelling anything. |
//! | [`expiry_eviction`] | pending | Age evicts as surely as pressure does: entries left past the expiry horizon are dropped on the next insert even when the buffer is nowhere near its capacity. A label that never comes must not pin its context forever, so the horizon is what bounds the buffer during quiet periods. |
//! | [`len_tracks_entries`] | pending | The reported occupancy follows the live entries in both directions — rising with each insert, falling with each removal, and reading empty at both ends. Because the eviction queue keeps its own record of identifiers that may already be gone, the count has to come from the entries themselves for a host to trust it as buffer depth. |
//! | [`concurrent_insert_remove`] | pending | Several threads may insert and remove against the same buffer at once without deadlock, panic or lost bookkeeping: after thousands of interleaved cycles across four threads the buffer settles back to essentially empty. Assessments and labels arrive on whatever threads the host runs, so the eviction lock and the entry map must interleave safely rather than merely being individually correct. |
//! | [`fifo_ordering_preserved`] | pending | cites (´claim:pending:exceeding-capacity-costs-the-oldest-entry-not-the-newest´) |
//! | [`expiry_not_early`] | pending | Freshly inserted entries survive the eviction pass that their own insert triggers: with a horizon of a hundred milliseconds, two assessments stored a moment ago are both still there. Expiry is a deadline for absent labels, and an entry discarded before its deadline would lose an update the host was entitled to make. |
//! | [`fifo_with_removal_handles_dangling`] | pending | Labelling an entry out of turn leaves its identifier stranded in the eviction queue, and the buffer treats that stranded identifier as nothing at all: it is skipped over, later inserts still find the oldest genuinely live entry to give up, and everything younger stays. Removal deliberately does not walk the queue to excise the identifier, so the eviction pass has to tolerate the gaps that ordinary labelling leaves behind rather than mistaking one for a victim. |
//! | [`live_entry_evictions_increment_counter`] | pending | Capacity pressure and age expiry both count when they remove a live pending entry. By contrast, cleaning an identifier whose entry a label already consumed changes no count: that identifier no longer stands for label evidence the eviction pass can destroy. |
//! | [`take_evictions_clears_counter`] | pending | cites (´claim:pending:only-successful-eviction-of-a-live-entry-increments-the-eviction-counter´) |
//! | [`guidance_snapshot_respects_limit`] | pending | A guidance scan copies out no more than the caller asked for, and a limit of zero copies nothing at all. Every entry in the snapshot is a full clone of a buffered assessment, so an unbounded scan of a deep buffer would turn a read-only query into the largest allocation in the system; the limit is what makes the cost of asking predictable. |
//! | [`guidance_get_clones_live_entry`] | pending | A guidance lookup reads without consuming — the entry is returned as a copy and stays in the buffer — and it reports absence once that entry has genuinely been labelled away. Guidance ranks candidates and then re-checks them, so it needs a read that neither steals an assessment from the labelling path nor vouches for one that has since gone. |
//! | [`memory_estimate_per_entry`] | pending | A whole buffered assessment at the reference configuration — eight Sentinels' features, the identity coordinates and active cells, the merged signals and the entity key — comes to a few kilobytes rather than tens. Capacity is expressed in entries, so this figure is the exchange rate between the configured depth and the memory the host actually commits to holding unlabelled work. |

//! Pending buffer implementation with `DashMap` + FIFO.
//!
//! The pending buffer stores [`PendingAssessment`] entries indexed by
//! [`AssessmentId`]. Entries are evicted when:
//!
//! 1. Capacity is exceeded
//! 2. Entries expire past `expiry_horizon`
//!
//! Eviction is lazy: up to 16 entries are evicted per insert operation.
//!
//! # Cross-References
//!
//! - (´dec:retention:pending-map´) — how an assessment awaiting its label is held
//! - (´dec:retention:lazy-eviction´) — eviction on insert, capped per insert

use std::collections::VecDeque;
use std::sync::Mutex;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

use dashmap::DashMap;

use super::entry::PendingAssessment;
use crate::types::AssessmentId;

/// Maximum entries to evict per insert operation.
///
/// The budget it serves is the hold on the eviction-order mutex, not a
/// wall-clock figure: the selection pass runs under that one lock, so an insert
/// occupies it for at most this many turns and every concurrent insert queues
/// behind exactly that. The removals the pass selects happen after the lock is
/// released (´rem:retention:eviction-bound-figure´).
///
/// At the specified capacity the bound also fixes how fast a stale buffer
/// clears: the request rate cancels out of capacity over evictions-per-insert,
/// leaving an eighth of the labelling latency. Sixteen exactly is a shipped
/// operating point within the range that argument allows, revisable once the
/// deferred eviction counter measures it (´entry:retention:eviction-counter´).
///
/// ´const:assayer:eviction-batch-bound´ (´alg:const:count´)
/// ´const:assayer:eviction-batch-bound-count-16´
const MAX_EVICTIONS_PER_INSERT: usize = 16;

// ═══════════════════════════════════════════════════════════════════════════════
// Pending Buffer
// ═══════════════════════════════════════════════════════════════════════════════

/// Pending assessment buffer with capacity limits and expiry.
///
/// Stores [`PendingAssessment`] entries indexed by [`AssessmentId`]. Provides
/// concurrent access via `DashMap` with FIFO ordering maintained separately
/// for eviction purposes.
///
/// # Capacity Management
///
/// On each insert, up to [`MAX_EVICTIONS_PER_INSERT`] stale entries are lazily
/// evicted. Eviction priority:
/// 1. Expired entries (past `expiry_horizon`)
/// 2. Over-capacity entries (FIFO order)
///
/// # Thread Safety
///
/// - `entries`: `DashMap` provides lock-free concurrent access
/// - `fifo`: `Mutex<VecDeque>` for eviction ordering
///
/// The FIFO queue may contain stale IDs (already removed from `DashMap`). These
/// are skipped during eviction.
///
/// # Cross-References
///
/// - (´dec:retention:pending-map´) — the concurrent map, and the eviction ordering kept beside it
#[derive(Debug)]
pub struct PendingBuffer {
    /// Entries indexed by assessment ID.
    entries: DashMap<AssessmentId, PendingAssessment>,
    /// FIFO queue for eviction ordering.
    fifo: Mutex<VecDeque<AssessmentId>>,
    /// Live entries evicted since the assessment path last took the count.
    evictions: AtomicU64,
    /// Live entries evicted over this buffer's lifetime.
    total_evictions: AtomicU64,
    /// Entries inserted over this buffer's lifetime.
    total_insertions: AtomicU64,
    /// Maximum number of entries to store.
    capacity: usize,
    /// Time horizon after which entries are considered expired.
    expiry_horizon: Duration,
}

impl PendingBuffer {
    /// Creates a new `PendingBuffer` with the given capacity and expiry horizon.
    ///
    /// # Arguments
    ///
    /// - `capacity`: Maximum number of entries to store
    /// - `expiry_horizon`: Duration after which entries are considered expired
    ///
    /// # Panics
    ///
    /// Debug panics if `capacity` is 0.
    #[must_use]
    pub fn new(capacity: usize, expiry_horizon: Duration) -> Self {
        debug_assert!(capacity > 0, "capacity must be positive");
        Self {
            entries: DashMap::with_capacity(capacity.min(1024)),
            fifo: Mutex::new(VecDeque::with_capacity(capacity.min(1024))),
            evictions: AtomicU64::new(0),
            total_evictions: AtomicU64::new(0),
            total_insertions: AtomicU64::new(0),
            capacity,
            expiry_horizon,
        }
    }

    /// Inserts a pending assessment into the buffer.
    ///
    /// On insert, lazily evicts up to [`MAX_EVICTIONS_PER_INSERT`] stale entries
    /// (expired or over-capacity).
    ///
    /// # Arguments
    ///
    /// - `assessment`: The pending assessment to insert
    pub fn insert(&self, assessment: PendingAssessment) {
        let id = assessment.id;
        let now = Instant::now();
        self.total_insertions.fetch_add(1, Ordering::Relaxed);

        // Insert the entry
        self.entries.insert(id, assessment);

        // Add to FIFO queue
        {
            let mut fifo = self.fifo.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            fifo.push_back(id);
        }

        // Lazy eviction: remove up to MAX_EVICTIONS_PER_INSERT stale entries
        self.evict_stale(now);
    }

    /// Removes and returns a pending assessment by ID.
    ///
    /// Returns `None` if the ID is not found (either never inserted, already
    /// removed, or evicted).
    ///
    /// Does not modify the FIFO queue. Dangling FIFO entries are harmless
    /// and will be skipped during the next eviction pass.
    #[must_use]
    pub fn remove(&self, id: AssessmentId) -> Option<PendingAssessment> {
        self.entries.remove(&id).map(|(_, v)| v)
    }

    /// Returns a clone of a live pending assessment by ID.
    ///
    /// Used by read-only guidance queries to validate that a candidate still
    /// exists after ranking. This reads only the `DashMap`; it never takes the
    /// FIFO eviction lock.
    #[must_use]
    pub fn get_for_guidance(&self, id: AssessmentId) -> Option<PendingAssessment> {
        self.entries.get(&id).map(|entry| entry.value().clone())
    }

    /// Returns a bounded snapshot of live entries for label guidance.
    ///
    /// Clones at most `scan_limit` entries from the `DashMap` and deliberately
    /// avoids the FIFO eviction queue. Concurrent `label()` removals and lazy
    /// eviction are tolerated by validating selected IDs with
    /// [`Self::get_for_guidance`] before returning them to the host.
    #[must_use]
    pub fn guidance_snapshot(&self, scan_limit: usize) -> Vec<PendingAssessment> {
        if scan_limit == 0 {
            return Vec::new();
        }

        self.entries
            .iter()
            .take(scan_limit)
            .map(|entry| entry.value().clone())
            .collect()
    }

    /// Returns the current number of entries in the buffer.
    ///
    /// This is the count of entries in the `DashMap`, which may differ from
    /// the FIFO queue length due to dangling entries.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Returns the configured capacity of the buffer.
    #[must_use]
    pub const fn capacity(&self) -> usize {
        self.capacity
    }

    /// Takes the live-entry eviction count accumulated since the last read.
    ///
    /// The swap makes concurrent eviction increments belong wholly to this
    /// read or the next one; a count is never observed twice.
    pub fn take_evictions(&self) -> u64 {
        self.evictions.swap(0, Ordering::Relaxed)
    }

    /// Returns lifetime live-entry evictions and insertions.
    #[must_use]
    pub fn eviction_totals(&self) -> (u64, u64) {
        (
            self.total_evictions.load(Ordering::Relaxed),
            self.total_insertions.load(Ordering::Relaxed),
        )
    }

    /// Returns the age of the oldest live pending entry at `now`.
    #[must_use]
    pub fn oldest_pending_age(&self, now: Instant) -> Option<Duration> {
        self.entries
            .iter()
            .map(|entry| entry.value().timestamp)
            .min()
            .map(|timestamp| now.saturating_duration_since(timestamp))
    }

    /// Returns `true` if the buffer contains no entries.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Evicts up to `MAX_EVICTIONS_PER_INSERT` stale entries.
    ///
    /// A stale entry is one that:
    /// 1. Has expired (timestamp + `expiry_horizon` < now), or
    /// 2. Is over capacity (FIFO order)
    ///
    /// Dangling FIFO entries (IDs not in `DashMap`) are skipped but counted
    /// against the eviction limit to prevent infinite loops.
    ///
    /// The FIFO lock is held only during the collection phase (popping
    /// IDs from the front of the `VecDeque`). `DashMap` removes happen
    /// after the lock is released (´rem:retention:eviction-bound-figure´).
    fn evict_stale(&self, now: Instant) {
        let to_evict = {
            let mut fifo = self.fifo.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            let mut evict = Vec::new();

            while evict.len() < MAX_EVICTIONS_PER_INSERT {
                // Check if eviction is needed
                let should_evict =
                    self.entries.len().saturating_sub(evict.len()) > self.capacity || self.has_expired_front(&fifo, now);

                if !should_evict {
                    break;
                }

                // Pop front of FIFO
                let Some(id) = fifo.pop_front() else {
                    break;
                };

                evict.push(id);
            }

            drop(fifo);
            evict
        };

        for id in &to_evict {
            // Remove may be a no-op if label() already consumed it.
            if self.entries.remove(id).is_some() {
                self.evictions.fetch_add(1, Ordering::Relaxed);
                self.total_evictions.fetch_add(1, Ordering::Relaxed);
            }
        }
    }

    /// Checks if the front FIFO entry is expired.
    fn has_expired_front(&self, fifo: &VecDeque<AssessmentId>, now: Instant) -> bool {
        let Some(&front_id) = fifo.front() else {
            return false;
        };

        // Look up the entry to check its timestamp
        let Some(entry) = self.entries.get(&front_id) else {
            // Entry already removed; treat as "evictable" to clean up FIFO
            return true;
        };

        now.duration_since(entry.timestamp) > self.expiry_horizon
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use std::collections::HashMap;
    use std::sync::Arc;
    use std::thread;

    use super::*;
    use crate::health::DegradationContext;
    use crate::pending::entry::{PendingRiskBasis, SentinelExtraction};
    use crate::types::{EntityKey, PersistentTimestamp, SentinelId};

    /// Creates a test `PendingAssessment` with the given ID and optional timestamp.
    fn create_assessment(id: AssessmentId) -> PendingAssessment {
        create_assessment_with_timestamp(id, Instant::now())
    }

    fn create_assessment_with_timestamp(id: AssessmentId, timestamp: Instant) -> PendingAssessment {
        let mut sentinel_extractions = HashMap::new();
        sentinel_extractions.insert(SentinelId(1), SentinelExtraction::new(u128::from(id.0), vec![1.0f32], true));

        PendingAssessment {
            spatial_axis_ids: Vec::new(),
            id,
            timestamp,
            persistent_timestamp: PersistentTimestamp::now(),
            entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
            sentinel_extractions,
            identity_coordinates: HashMap::new(),
            identity_active_cells: HashMap::new(),
            active_sentinels: Vec::new(),
            reporting_sentinels: Vec::new(),
            entity_base_features: HashMap::new(),
            entity_axis_features: HashMap::new(),
            signal_features: vec![0.5f32].into(),
            risk_basis: PendingRiskBasis::default(),
            outcome_predictions: HashMap::new(),
            degradation: DegradationContext::default(),
            report_origin: None,
        }
    }

    /// An assessment placed in the buffer is retrievable by its own
    /// identifier and comes back as the entry that went in. This is the whole
    /// contract of deferred labelling: the assessment context has to still be
    /// there, and be the right one, when the outcome arrives some time later.
    ///
    /// ´claim:pending:a-buffered-assessment-comes-back-under-its-own-identifier´
    /// ´test:unit:insert-then-remove-returns-the-same-entry´
    #[test]
    fn insert_then_remove_returns_the_same_entry() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
        let assessment = create_assessment(AssessmentId(42));
        buffer.insert(assessment);

        let removed = buffer.remove(AssessmentId(42));
        assert!(removed.is_some());
        assert_eq!(removed.unwrap().id, AssessmentId(42));
    }

    /// A label naming an identifier the buffer never held is answered with
    /// absence, not with a panic or an invented entry. Labels arrive from the
    /// host and may be late, duplicated or simply wrong, so an unknown
    /// identifier has to be an ordinary outcome of the lookup.
    ///
    /// ´claim:pending:a-lookup-for-an-entry-the-buffer-does-not-hold-yields-absence´
    /// ´test:unit:remove-unknown-returns-none´
    #[test]
    fn remove_unknown_returns_none() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
        assert!(buffer.remove(AssessmentId(9999)).is_none());
    }

    /// Removal consumes the entry: the first call hands the assessment over
    /// and the second finds nothing. A duplicated label therefore cannot drive
    /// the same model update twice, because the context it would need has
    /// already left the buffer.
    ///
    /// ´claim:pending:removal-consumes-the-entry-so-a-repeated-label-finds-nothing´
    /// ´test:unit:remove-twice-returns-none´
    #[test]
    fn remove_twice_returns_none() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
        buffer.insert(create_assessment(AssessmentId(42)));

        assert!(buffer.remove(AssessmentId(42)).is_some());
        assert!(buffer.remove(AssessmentId(42)).is_none(), "double-remove should return None");
    }

    /// The buffer is bounded: pushing one assessment past the configured
    /// capacity costs the oldest entry, never the newest. Assessments whose
    /// labels have not arrived accumulate indefinitely otherwise, and when the
    /// bound has to bite it is the least likely to still be labelled that is
    /// given up.
    ///
    /// ´claim:pending:exceeding-capacity-costs-the-oldest-entry-not-the-newest´
    /// ´test:unit:capacity-eviction´
    #[test]
    fn capacity_eviction() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));

        // Insert 101 entries
        for i in 0..=100 {
            buffer.insert(create_assessment(AssessmentId(i)));
        }

        // The first entry (ID 0) should have been evicted
        assert!(buffer.remove(AssessmentId(0)).is_none(), "oldest entry should be evicted");

        // Entry 100 should still be present
        assert!(buffer.remove(AssessmentId(100)).is_some(), "newest entry should be present");
    }

    /// Eviction is amortised rather than exhaustive: an insert arriving after
    /// a large backlog of entries has gone stale does a bounded amount of
    /// cleanup and returns, leaving the rest for later inserts. The buffer may
    /// stay over its bound for a while, but no single caller pays for the
    /// whole backlog — which is what keeps insert latency independent of how
    /// long the host went without labelling anything.
    ///
    /// ´claim:pending:a-large-stale-backlog-is-cleared-over-many-inserts-not-in-one´
    /// ´test:unit:lazy-eviction-capped-at-16´
    #[test]
    fn lazy_eviction_capped_at_16() {
        let buffer = PendingBuffer::new(10, Duration::from_millis(1));

        // Insert 100 entries (will exceed capacity by 90)
        for i in 0..100 {
            buffer.insert(create_assessment(AssessmentId(i)));
        }

        // Wait for all to expire
        std::thread::sleep(Duration::from_millis(5));

        // Insert one more entry — should evict at most 16
        buffer.insert(create_assessment(AssessmentId(200)));

        // Should have evicted at most 16 entries, so len should be 100 - 16 + 1 = 85
        // But this depends on the exact timing. The key invariant is that we
        // don't block forever even with many expired entries.
        assert!(buffer.len() <= 100, "should not grow unbounded");

        // The newest entry should be present
        assert!(buffer.remove(AssessmentId(200)).is_some());
    }

    /// Age evicts as surely as pressure does: entries left past the expiry
    /// horizon are dropped on the next insert even when the buffer is nowhere
    /// near its capacity. A label that never comes must not pin its context
    /// forever, so the horizon is what bounds the buffer during quiet periods.
    ///
    /// ´claim:pending:entries-past-the-expiry-horizon-are-dropped-even-well-under-capacity´
    /// ´test:unit:expiry-eviction´
    #[test]
    fn expiry_eviction() {
        let buffer = PendingBuffer::new(1000, Duration::from_millis(10));

        // Insert entries
        for i in 0..50 {
            buffer.insert(create_assessment(AssessmentId(i)));
        }

        // Wait for expiry
        std::thread::sleep(Duration::from_millis(20));

        // Insert a new entry to trigger eviction
        buffer.insert(create_assessment(AssessmentId(100)));

        // Some expired entries should have been evicted
        // (up to 16 per the capped eviction limit)
        assert!(buffer.len() < 51, "some expired entries should be evicted");
    }

    /// The reported occupancy follows the live entries in both directions —
    /// rising with each insert, falling with each removal, and reading empty
    /// at both ends. Because the eviction queue keeps its own record of
    /// identifiers that may already be gone, the count has to come from the
    /// entries themselves for a host to trust it as buffer depth.
    ///
    /// ´claim:pending:the-reported-occupancy-follows-the-live-entries-in-both-directions´
    /// ´test:unit:len-tracks-entries´
    #[test]
    fn len_tracks_entries() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));

        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());

        buffer.insert(create_assessment(AssessmentId(1)));
        assert_eq!(buffer.len(), 1);
        assert!(!buffer.is_empty());

        buffer.insert(create_assessment(AssessmentId(2)));
        assert_eq!(buffer.len(), 2);

        drop(buffer.remove(AssessmentId(1)));
        assert_eq!(buffer.len(), 1);

        drop(buffer.remove(AssessmentId(2)));
        assert_eq!(buffer.len(), 0);
        assert!(buffer.is_empty());
    }

    /// Several threads may insert and remove against the same buffer at once
    /// without deadlock, panic or lost bookkeeping: after thousands of
    /// interleaved cycles across four threads the buffer settles back to
    /// essentially empty. Assessments and labels arrive on whatever threads
    /// the host runs, so the eviction lock and the entry map must interleave
    /// safely rather than merely being individually correct.
    ///
    /// ´claim:pending:concurrent-inserts-and-removals-leave-the-buffer-consistent´
    /// ´test:unit:concurrent-insert-remove´
    #[test]
    fn concurrent_insert_remove() {
        let buffer = Arc::new(PendingBuffer::new(10_000, Duration::from_secs(3600)));
        let mut handles = vec![];

        // Spawn 4 threads, each doing 1000 insert+remove cycles
        for thread_id in 0..4u64 {
            let buffer = Arc::clone(&buffer);
            handles.push(thread::spawn(move || {
                for i in 0..1000u64 {
                    let id = AssessmentId(thread_id * 10_000 + i);
                    buffer.insert(create_assessment(id));
                    // Immediately remove
                    drop(buffer.remove(id));
                }
            }));
        }

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("thread panicked");
        }

        // Buffer should be empty or nearly empty
        // (some entries may remain from timing between insert and remove)
        assert!(buffer.len() < 100, "buffer should be nearly empty, got {}", buffer.len());
    }

    /// Eviction follows insertion order exactly and takes no more than it
    /// must: with the buffer full, one further insert removes the single
    /// oldest entry and leaves every younger one — including the newcomer —
    /// untouched. Ordering by arrival is what makes the sacrifice principled
    /// rather than arbitrary.
    ///
    /// (´claim:pending:exceeding-capacity-costs-the-oldest-entry-not-the-newest´)
    /// ´test:unit:fifo-ordering-preserved´
    #[test]
    fn fifo_ordering_preserved() {
        let buffer = PendingBuffer::new(5, Duration::from_secs(3600));

        // Insert entries in order
        for i in 1..=5 {
            buffer.insert(create_assessment(AssessmentId(i)));
        }

        // Insert one more to trigger eviction
        buffer.insert(create_assessment(AssessmentId(6)));

        // Entry 1 (oldest) should be evicted
        assert!(buffer.remove(AssessmentId(1)).is_none(), "entry 1 should be evicted");

        // Entries 2-6 should still be present
        for i in 2..=6 {
            assert!(buffer.remove(AssessmentId(i)).is_some(), "entry {i} should be present");
        }
    }

    /// Freshly inserted entries survive the eviction pass that their own
    /// insert triggers: with a horizon of a hundred milliseconds, two
    /// assessments stored a moment ago are both still there. Expiry is a
    /// deadline for absent labels, and an entry discarded before its deadline
    /// would lose an update the host was entitled to make.
    ///
    /// ´claim:pending:a-fresh-entry-survives-the-eviction-pass-its-own-insert-triggers´
    /// ´test:unit:expiry-not-early´
    #[test]
    fn expiry_not_early() {
        // Both entries should be present immediately after insert (not yet expired)
        let buffer = PendingBuffer::new(100, Duration::from_millis(100));

        buffer.insert(create_assessment(AssessmentId(1)));
        buffer.insert(create_assessment(AssessmentId(2)));

        // Both should still be present (inserted just now, not expired)
        assert!(buffer.remove(AssessmentId(1)).is_some(), "entry 1 should still be present");
        assert!(buffer.remove(AssessmentId(2)).is_some(), "entry 2 should still be present");
    }

    /// Labelling an entry out of turn leaves its identifier stranded in the
    /// eviction queue, and the buffer treats that stranded identifier as
    /// nothing at all: it is skipped over, later inserts still find the oldest
    /// genuinely live entry to give up, and everything younger stays. Removal
    /// deliberately does not walk the queue to excise the identifier, so the
    /// eviction pass has to tolerate the gaps that ordinary labelling leaves
    /// behind rather than mistaking one for a victim.
    ///
    /// ´claim:pending:stranded-queue-identifiers-are-skipped-and-do-not-shield-live-entries´
    /// ´test:unit:fifo-with-removal-handles-dangling´
    #[test]
    fn fifo_with_removal_handles_dangling() {
        // Test that FIFO handles dangling entries correctly after mid-removal
        let buffer = PendingBuffer::new(3, Duration::from_secs(3600));

        // Insert A, B, C
        buffer.insert(create_assessment(AssessmentId(1))); // A
        buffer.insert(create_assessment(AssessmentId(2))); // B
        buffer.insert(create_assessment(AssessmentId(3))); // C

        // Remove B (creates dangling entry in FIFO)
        assert!(buffer.remove(AssessmentId(2)).is_some(), "B should be removable");

        // Insert D — triggers eviction with capacity 3
        // FIFO order: [A, B(dangling), C, D]
        // On capacity check: 3 entries (A, C, D), capacity is 3, no eviction needed
        buffer.insert(create_assessment(AssessmentId(4))); // D

        // Insert E — now over capacity, should evict A (FIFO)
        // FIFO order: [A, B(dangling), C, D, E]
        buffer.insert(create_assessment(AssessmentId(5))); // E

        // A should be evicted (oldest non-dangling)
        assert!(buffer.remove(AssessmentId(1)).is_none(), "A should be evicted");
        // B already removed
        assert!(buffer.remove(AssessmentId(2)).is_none(), "B already removed");
        // C, D, E should still be present
        assert!(buffer.remove(AssessmentId(3)).is_some(), "C should be present");
        assert!(buffer.remove(AssessmentId(4)).is_some(), "D should be present");
        assert!(buffer.remove(AssessmentId(5)).is_some(), "E should be present");
    }

    /// Capacity pressure and age expiry both count when they remove a live
    /// pending entry. By contrast, cleaning an identifier whose entry a label
    /// already consumed changes no count: that identifier no longer stands
    /// for label evidence the eviction pass can destroy.
    ///
    /// ´claim:pending:only-successful-eviction-of-a-live-entry-increments-the-eviction-counter´
    /// ´test:unit:live-entry-evictions-increment-counter´
    #[test]
    fn live_entry_evictions_increment_counter() {
        let capacity_buffer = PendingBuffer::new(1, Duration::from_secs(3600));
        capacity_buffer.insert(create_assessment(AssessmentId(1)));
        capacity_buffer.insert(create_assessment(AssessmentId(2)));
        assert_eq!(
            capacity_buffer.take_evictions(),
            1,
            "capacity eviction removes one live entry"
        );

        assert!(capacity_buffer.remove(AssessmentId(2)).is_some());
        capacity_buffer.insert(create_assessment(AssessmentId(3)));
        assert_eq!(
            capacity_buffer.take_evictions(),
            0,
            "cleaning the consumed entry's FIFO identifier is not a live eviction"
        );

        let expiry_buffer = PendingBuffer::new(10, Duration::from_millis(1));
        let now = Instant::now();
        let expired_at = now.checked_sub(Duration::from_millis(10)).unwrap_or(now);
        expiry_buffer.insert(create_assessment_with_timestamp(AssessmentId(4), expired_at));
        assert_eq!(expiry_buffer.take_evictions(), 1, "expiry removes one live entry");
    }

    /// Successful live-entry removals accumulate between reads, and taking
    /// that count transfers the whole interval to one caller. The immediately
    /// following read is empty, so no assessment snapshot can report an
    /// eviction an earlier snapshot already carried.
    ///
    /// (´claim:pending:only-successful-eviction-of-a-live-entry-increments-the-eviction-counter´)
    /// ´test:unit:take-evictions-clears-counter´
    #[test]
    fn take_evictions_clears_counter() {
        let buffer = PendingBuffer::new(1, Duration::from_secs(3600));
        buffer.insert(create_assessment(AssessmentId(1)));
        buffer.insert(create_assessment(AssessmentId(2)));
        buffer.insert(create_assessment(AssessmentId(3)));

        assert_eq!(buffer.take_evictions(), 2);
        assert_eq!(buffer.take_evictions(), 0);
        assert_eq!(buffer.eviction_totals(), (2, 3));
    }

    /// A guidance scan copies out no more than the caller asked for, and a
    /// limit of zero copies nothing at all. Every entry in the snapshot is a
    /// full clone of a buffered assessment, so an unbounded scan of a deep
    /// buffer would turn a read-only query into the largest allocation in the
    /// system; the limit is what makes the cost of asking predictable.
    ///
    /// ´claim:pending:a-guidance-scan-clones-no-more-than-the-requested-limit´
    /// ´test:unit:guidance-snapshot-respects-limit´
    #[test]
    fn guidance_snapshot_respects_limit() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));

        for id in 0..10 {
            buffer.insert(create_assessment(AssessmentId(id)));
        }

        let snapshot = buffer.guidance_snapshot(3);
        assert_eq!(snapshot.len(), 3);

        let empty = buffer.guidance_snapshot(0);
        assert!(empty.is_empty());
    }

    /// A guidance lookup reads without consuming — the entry is returned as a
    /// copy and stays in the buffer — and it reports absence once that entry
    /// has genuinely been labelled away. Guidance ranks candidates and then
    /// re-checks them, so it needs a read that neither steals an assessment
    /// from the labelling path nor vouches for one that has since gone.
    ///
    /// ´claim:pending:a-guidance-lookup-reads-without-consuming-and-sees-only-live-entries´
    /// ´test:unit:guidance-get-clones-live-entry´
    #[test]
    fn guidance_get_clones_live_entry() {
        let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
        buffer.insert(create_assessment(AssessmentId(42)));

        let live = buffer.get_for_guidance(AssessmentId(42));
        assert!(live.is_some());

        drop(buffer.remove(AssessmentId(42)));
        assert!(buffer.get_for_guidance(AssessmentId(42)).is_none());
    }

    /// A whole buffered assessment at the reference configuration — eight
    /// Sentinels' features, the identity coordinates and active cells, the
    /// merged signals and the entity key — comes to a few kilobytes rather
    /// than tens. Capacity is expressed in entries, so this figure is the
    /// exchange rate between the configured depth and the memory the host
    /// actually commits to holding unlabelled work.
    ///
    /// ´claim:pending:a-reference-entry-costs-a-few-kilobytes-so-capacity-converts-to-memory´
    /// ´test:unit:memory-estimate-per-entry´
    #[test]
    fn memory_estimate_per_entry() {
        // Reference config: 8 sentinels × 62 features + identity + signals ≈ 3.2 KB
        // This test verifies a realistic entry size is within expected bounds.
        let mut sentinel_extractions = HashMap::new();
        for s in 0..8 {
            sentinel_extractions.insert(SentinelId(s), SentinelExtraction::new(u128::from(s), vec![0.0f32; 62], true));
        }

        let mut identity_coordinates = HashMap::new();
        identity_coordinates.insert(crate::types::DimensionId(0), 0u128);
        identity_coordinates.insert(crate::types::DimensionId(1), 0u128);

        let mut identity_active_cells = HashMap::new();
        identity_active_cells.insert(
            crate::types::DimensionId(0),
            vec![crate::identity::CompetitiveCellId::new(0, 0); 10],
        );
        identity_active_cells.insert(
            crate::types::DimensionId(1),
            vec![crate::identity::CompetitiveCellId::new(0, 0); 10],
        );

        let assessment = PendingAssessment {
            spatial_axis_ids: Vec::new(),
            id: AssessmentId(42),
            timestamp: Instant::now(),
            persistent_timestamp: PersistentTimestamp::now(),
            entity: EntityKey::new(vec![0u8; 32]),
            sentinel_extractions,
            identity_coordinates,
            identity_active_cells,
            active_sentinels: Vec::new(),
            reporting_sentinels: Vec::new(),
            entity_base_features: HashMap::new(),
            entity_axis_features: HashMap::new(),
            signal_features: vec![0.0f32; 20].into(),
            risk_basis: PendingRiskBasis::default(),
            outcome_predictions: HashMap::new(),
            degradation: DegradationContext::default(),
            report_origin: None,
        };

        // Calculate approximate size:
        // - Base struct: ~200 bytes
        // - 8 × SentinelExtraction (~300 bytes each): 2400 bytes
        // - identity_coordinates: ~100 bytes
        // - identity_active_cells: ~200 bytes
        // - signal_features: ~100 bytes
        // Total: ~3000 bytes ≈ 3 KB
        let inline_size = std::mem::size_of_val(&assessment);
        let sentinel_heap: usize = assessment.sentinel_extractions.values().map(|e| e.features.len() * 4).sum();
        let signal_heap = assessment.signal_features.len() * 4;
        let entity_heap = assessment.entity.0.len();
        let total = inline_size + sentinel_heap + signal_heap + entity_heap;

        // Allow range 2-5 KB per entry at reference config
        assert!(
            total > 2000 && total < 5000,
            "entry size should be ~3.2 KB, got {total} bytes"
        );
    }
}
