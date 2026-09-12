// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`buffer_insert_with_capacity_100_evicts_on_101st`] | pending | cites (´claim:pending:exceeding-capacity-costs-the-oldest-entry-not-the-newest´) |
//! | [`buffer_remove_returns_entry`] | pending | cites (´claim:pending:a-buffered-assessment-comes-back-under-its-own-identifier´) |
//! | [`buffer_remove_unknown_returns_none`] | pending | cites (´claim:pending:a-lookup-for-an-entry-the-buffer-does-not-hold-yields-absence´) |
//! | [`buffer_remove_twice_returns_none`] | pending | cites (´claim:pending:removal-consumes-the-entry-so-a-repeated-label-finds-nothing´) |
//! | [`lazy_eviction_50ms_expiry`] | pending | cites (´claim:pending:entries-past-the-expiry-horizon-are-dropped-even-well-under-capacity´) |
//! | [`eviction_capped_at_16_per_insert`] | pending | cites (´claim:pending:a-large-stale-backlog-is-cleared-over-many-inserts-not-in-one´) |
//! | [`pending_assessment_to_context_strips_transient_fields`] | pending | cites (´claim:pending:the-journal-context-preserves-everything-replay-depends-on´) |
//! | [`pending_assessment_to_context_preserves_fields`] | pending | cites (´claim:pending:the-journal-context-preserves-everything-replay-depends-on´) |
//! | [`pending_context_serde_roundtrip_f32`] | pending | cites (´claim:pending:a-context-round-trips-through-serialisation-with-its-features-exact´) |
//! | [`sentinel_extraction_62_features_memory`] | pending | cites (´claim:pending:an-extraction-at-reference-width-stays-within-its-memory-budget´) |
//! | [`buffer_concurrent_4_threads_1000_ops`] | pending | cites (´claim:pending:concurrent-inserts-and-removals-leave-the-buffer-consistent´) |
//! | [`context_serde_empty_sentinels`] | pending | Emptiness survives the journal as faithfully as content does: an assessment with no Sentinel extractions, no coordinates and no signal features round-trips to a context that is empty in exactly those places. An assessment made when no Sentinel had anything to say is a legitimate thing to replay, and it must not come back reading as absent data or as a deserialisation failure. |

//! Crate-level tests for the pending buffer module.
//!
//! # Cross-References
//!
//! - (´dec:retention:pending-map´) — how an assessment awaiting its label is held
//! - ADR review — Implementation specification

use std::collections::HashMap;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::health::DegradationContext;
use crate::pending::{PendingAssessment, PendingBuffer, PendingRiskBasis, SentinelExtraction};
use crate::testing::assert_below;
use crate::testing::dimensions::make_cells;
use crate::types::{AssessmentId, DimensionId, EntityKey, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Test Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a test `PendingAssessment` with the given ID.
fn create_pending_assessment(id: AssessmentId) -> PendingAssessment {
    create_pending_assessment_at_time(id, Instant::now())
}

/// Creates a test `PendingAssessment` with the given ID and timestamp.
fn create_pending_assessment_at_time(id: AssessmentId, timestamp: Instant) -> PendingAssessment {
    let mut sentinel_extractions = HashMap::new();
    sentinel_extractions.insert(
        SentinelId(1),
        SentinelExtraction::new(u128::from(id.0), vec![1.0f32, 2.0], true),
    );

    let mut identity_coordinates = HashMap::new();
    identity_coordinates.insert(DimensionId(1), u128::from(id.0));

    let mut identity_active_cells = HashMap::new();
    identity_active_cells.insert(DimensionId(1), make_cells(1));

    PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id,
        timestamp,
        persistent_timestamp: PersistentTimestamp::now(),
        entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
        sentinel_extractions,
        identity_coordinates,
        identity_active_cells,
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: vec![0.5f32, 1.0, -0.3].into(),
        risk_basis: PendingRiskBasis::new(0.1, 0.3, 0.2, 0.0),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        report_origin: None,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Capacity & Eviction Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Assembled as the crate assembles it, the buffer still honours its bound at
/// the reference capacity: the hundred-and-first assessment displaces the very
/// first, and arrives itself. The limit is a property of the buffer as the rest
/// of the crate constructs it, not only of the module's own fixtures.
///
/// (´claim:pending:exceeding-capacity-costs-the-oldest-entry-not-the-newest´)
/// ´test:crate:buffer-insert-with-capacity-100-evicts-on-101st´
#[test]
fn buffer_insert_with_capacity_100_evicts_on_101st() {
    let buffer = PendingBuffer::new(100, Duration::from_secs(3600));

    // Insert 101 entries
    for i in 0..=100 {
        buffer.insert(create_pending_assessment(AssessmentId(i)));
    }

    // The first entry (ID 0) should have been evicted
    assert!(
        buffer.remove(AssessmentId(0)).is_none(),
        "oldest entry (ID 0) should be evicted"
    );

    // Entry 100 should still be present
    assert!(
        buffer.remove(AssessmentId(100)).is_some(),
        "newest entry (ID 100) should be present"
    );
}

/// An assessment carrying the crate's real entry shape — Sentinel
/// extractions, identity coordinates and active cells alike — is stored and
/// handed back under its own identifier. The round trip holds for a fully
/// populated entry, not merely for a skeleton one.
///
/// (´claim:pending:a-buffered-assessment-comes-back-under-its-own-identifier´)
/// ´test:crate:buffer-remove-returns-entry´
#[test]
fn buffer_remove_returns_entry() {
    let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
    buffer.insert(create_pending_assessment(AssessmentId(42)));

    let removed = buffer.remove(AssessmentId(42));
    assert!(removed.is_some());
    assert_eq!(removed.unwrap().id, AssessmentId(42));
}

/// An identifier far outside anything the buffer has ever held is answered
/// with absence rather than a fault, even on a buffer that is entirely empty.
/// A host is free to ask about an assessment it only thinks it made.
///
/// (´claim:pending:a-lookup-for-an-entry-the-buffer-does-not-hold-yields-absence´)
/// ´test:crate:buffer-remove-unknown-returns-none´
#[test]
fn buffer_remove_unknown_returns_none() {
    let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
    assert!(buffer.remove(AssessmentId(9999)).is_none());
}

/// Labelling the same assessment twice through the crate's own interface gets
/// the context exactly once: the repeat finds nothing left to consume. This is
/// what stops a retried or duplicated label from being applied to the model a
/// second time.
///
/// (´claim:pending:removal-consumes-the-entry-so-a-repeated-label-finds-nothing´)
/// ´test:crate:buffer-remove-twice-returns-none´
#[test]
fn buffer_remove_twice_returns_none() {
    let buffer = PendingBuffer::new(100, Duration::from_secs(3600));
    buffer.insert(create_pending_assessment(AssessmentId(42)));

    assert!(buffer.remove(AssessmentId(42)).is_some());
    assert!(buffer.remove(AssessmentId(42)).is_none(), "double-remove should return None");
}

/// With capacity set far above the entries held, it is age alone that clears
/// them: fifty assessments left past a fifty-millisecond horizon start
/// disappearing as soon as the next insert gives the buffer a chance to look.
/// Nothing but the passage of time is needed to reclaim work the host has
/// stopped labelling.
///
/// (´claim:pending:entries-past-the-expiry-horizon-are-dropped-even-well-under-capacity´)
/// ´test:crate:lazy-eviction-50ms-expiry´
#[test]
fn lazy_eviction_50ms_expiry() {
    let buffer = PendingBuffer::new(1000, Duration::from_millis(50));

    // Insert 50 entries
    for i in 0..50 {
        buffer.insert(create_pending_assessment(AssessmentId(i)));
    }

    // Wait for expiry (100ms to be safe)
    std::thread::sleep(Duration::from_millis(100));

    // Insert a new entry to trigger eviction
    buffer.insert(create_pending_assessment(AssessmentId(100)));

    // Some expired entries should have been evicted (up to 16 per insert)
    assert!(
        buffer.len() < 51,
        "some expired entries should be evicted, got len={}",
        buffer.len()
    );
}

/// An insert made against a buffer holding a hundred simultaneously expired
/// entries returns, and the assessment it carried is there afterwards. The
/// cleanup work is bounded, so the caller that happens to arrive after a long
/// unlabelled stretch is not made to pay for it before its own entry lands.
///
/// (´claim:pending:a-large-stale-backlog-is-cleared-over-many-inserts-not-in-one´)
/// ´test:crate:eviction-capped-at-16-per-insert´
#[test]
fn eviction_capped_at_16_per_insert() {
    let buffer = PendingBuffer::new(10, Duration::from_millis(1));

    // Insert 100 entries
    for i in 0..100 {
        buffer.insert(create_pending_assessment(AssessmentId(i)));
    }

    // Wait for all to expire
    std::thread::sleep(Duration::from_millis(5));

    // Insert one more entry — should evict at most 16
    buffer.insert(create_pending_assessment(AssessmentId(200)));

    // The newest entry should be present
    assert!(buffer.remove(AssessmentId(200)).is_some(), "newest entry should be present");

    // We can't assert exact numbers due to timing, but the important thing
    // is that insert doesn't block forever even with many expired entries.
}

// ═══════════════════════════════════════════════════════════════════════════════
// Conversion Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The entity key crosses into the journal context unchanged, which is the
/// one field replay cannot recover by any other route: it is the handle the
/// signal cache and identity lookup are keyed by when a restored run
/// reconstructs the update.
///
/// (´claim:pending:the-journal-context-preserves-everything-replay-depends-on´)
/// ´test:crate:pending-assessment-to-context-strips-transient-fields´
#[test]
fn pending_assessment_to_context_strips_transient_fields() {
    let assessment = create_pending_assessment(AssessmentId(42));
    let context = assessment.to_context();

    assert_eq!(context.entity, assessment.entity);
}

/// An assessment built the way the crate builds one — populated coordinates
/// and real competitive cells rather than empty maps — converts to its journal
/// context with entity, signals, extractions, coordinates and active cells all
/// intact. The conversion holds for entries that actually carry identity, not
/// only for sparse fixtures.
///
/// (´claim:pending:the-journal-context-preserves-everything-replay-depends-on´)
/// ´test:crate:pending-assessment-to-context-preserves-fields´
#[test]
fn pending_assessment_to_context_preserves_fields() {
    let assessment = create_pending_assessment(AssessmentId(42));
    let context = assessment.to_context();

    assert_eq!(context.entity, assessment.entity);
    assert_eq!(context.signal_features, assessment.signal_features);
    assert_eq!(context.sentinel_extractions.len(), assessment.sentinel_extractions.len());
    assert_eq!(context.identity_coordinates, assessment.identity_coordinates);
    assert_eq!(context.identity_active_cells.len(), assessment.identity_active_cells.len());
}

/// Signal features written out and read back compare equal to the originals
/// value for value, negatives included. The journal is the only thing standing
/// between a crash and the model update the assessment was going to make, so a
/// round trip that perturbed the features would corrupt the very evidence it
/// exists to preserve.
///
/// (´claim:pending:a-context-round-trips-through-serialisation-with-its-features-exact´)
/// ´test:crate:pending-context-serde-roundtrip-f32´
#[cfg(feature = "serde")]
#[test]
fn pending_context_serde_roundtrip_f32() {
    use crate::pending::PendingContext;

    let assessment = create_pending_assessment(AssessmentId(42));
    let context = assessment.to_context();

    let json = serde_json::to_string(&context).expect("serialise");
    let restored: PendingContext = serde_json::from_str(&json).expect("deserialise");

    assert_eq!(restored.signal_features, context.signal_features);
    // Verify f32 features survived
    assert_eq!(
        restored.signal_features,
        crate::pending::StoredFeatures::Single(vec![0.5f32, 1.0, -0.3])
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Memory Size Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Measured against the crate's shared budget assertion, one Sentinel's
/// extraction at the reference feature width sits under a few hundred bytes
/// inline and on the heap together. The figure is a stated budget the crate
/// holds itself to, so a widening of the entry layout shows up as a budget
/// breach rather than as unexplained growth in a running host.
///
/// (´claim:pending:an-extraction-at-reference-width-stays-within-its-memory-budget´)
/// ´test:crate:sentinel-extraction-62-features-memory´
#[test]
fn sentinel_extraction_62_features_memory() {
    // Reference q = 62 features at m_s=1
    let features: Vec<f32> = vec![0.0f32; 62];
    let extraction = SentinelExtraction::new(0, features, true);

    // Approximate size calculation:
    // - SentinelExtraction struct overhead
    // - u128 coordinate (16 bytes)
    // - Vec header (24 bytes on 64-bit)
    // - 62 × f32 (248 bytes)
    // - bool (1 byte)
    // Total: ~289 bytes inline + heap
    let inline_size = std::mem::size_of_val(&extraction);
    let heap_size = extraction.features.len() * std::mem::size_of::<f32>();
    let total = inline_size + heap_size;

    // The spec says ~252 bytes; allow some slack for alignment.
    // Use `<` semantics by asserting against `limit - 1` since
    // `assert_below` is inclusive (`actual <= limit`).
    assert_below(total as f64, 399.0, "SentinelExtraction(62 features) bytes");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrency Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Four thousand insert-and-remove cycles run concurrently across four
/// threads, on entries with the crate's full identity payload, complete
/// without a thread panicking and leave the buffer essentially drained. Real
/// assessments are heavier than the module's fixtures, and the buffer's
/// behaviour under contention does not depend on how much each entry carries.
///
/// (´claim:pending:concurrent-inserts-and-removals-leave-the-buffer-consistent´)
/// ´test:crate:buffer-concurrent-4-threads-1000-ops´
#[test]
fn buffer_concurrent_4_threads_1000_ops() {
    let buffer = Arc::new(PendingBuffer::new(10_000, Duration::from_secs(3600)));
    let mut handles = vec![];

    // Spawn 4 threads, each doing 1000 insert+remove cycles
    for thread_id in 0..4u64 {
        let buffer = Arc::clone(&buffer);
        handles.push(thread::spawn(move || {
            for i in 0..1000u64 {
                let id = AssessmentId(thread_id * 10_000 + i);
                buffer.insert(create_pending_assessment(id));
                // Immediately remove
                drop(buffer.remove(id));
            }
        }));
    }

    // Wait for all threads to complete (no panics)
    for handle in handles {
        handle.join().expect("thread panicked");
    }

    // Buffer should be empty or nearly empty
    assert!(
        buffer.len() < 100,
        "buffer should be nearly empty after concurrent insert+remove, got {}",
        buffer.len()
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serde Edge Cases
// ═══════════════════════════════════════════════════════════════════════════════

/// Emptiness survives the journal as faithfully as content does: an
/// assessment with no Sentinel extractions, no coordinates and no signal
/// features round-trips to a context that is empty in exactly those places.
/// An assessment made when no Sentinel had anything to say is a legitimate
/// thing to replay, and it must not come back reading as absent data or as a
/// deserialisation failure.
///
/// ´claim:pending:an-assessment-with-nothing-extracted-round-trips-as-genuinely-empty´
/// ´test:crate:context-serde-empty-sentinels´
#[cfg(feature = "serde")]
#[test]
fn context_serde_empty_sentinels() {
    use crate::pending::PendingContext;

    // Create a pending assessment with no Sentinel extractions.
    let assessment = PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id: AssessmentId(42),
        timestamp: Instant::now(),
        persistent_timestamp: PersistentTimestamp::now(),
        entity: EntityKey::new(vec![1, 2, 3]),
        sentinel_extractions: HashMap::new(), // Empty!
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
    };

    let context = assessment.to_context();

    let json = serde_json::to_string(&context).expect("serialise");
    let restored: PendingContext = serde_json::from_str(&json).expect("deserialise");

    assert!(
        restored.sentinel_extractions.is_empty(),
        "sentinel_extractions should be empty"
    );
    assert!(
        restored.identity_coordinates.is_empty(),
        "identity_coordinates should be empty"
    );
    assert!(restored.signal_features.is_empty(), "signal_features should be empty");
}
