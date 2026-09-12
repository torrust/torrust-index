// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`competitive_set_containment_chain`] | identity | cites (´claim:identity:a-coordinates-active-cells-are-returned-as-a-containment-chain-deepest-first´) |
//! | [`competitive_set_empty`] | identity | cites (´claim:identity:an-empty-competitive-set-matches-no-coordinate´) |
//! | [`competitive_set_exact_boundary`] | identity | cites (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´) |
//! | [`competitive_set_root_only`] | identity | The root cell spans the entire domain, so while it is the only competitive cell every entity routes to it and no other. This is the coarsest useful state of a dimension: one population containing everybody, which the model can still learn from while finer structure is being earned. |
//! | [`cell_outcome_state_neutral`] | identity | cites (´claim:identity:a-newly-tracked-cell-starts-neutral-with-no-outcome-history-of-its-own´) |
//! | [`cell_outcome_state_ewma_formula`] | identity | cites (´claim:identity:a-single-outcome-moves-a-cells-ewmas-by-the-complement-of-the-smoothing-factor´) |
//! | [`observation_try_send_success`] | identity | cites (´claim:identity:an-observation-handed-over-by-an-assessment-thread-reaches-the-maintenance-side-unchanged´) |
//! | [`observation_overflow_counter`] | identity | cites (´claim:identity:every-refused-observation-is-counted-so-shed-load-is-visible-rather-than-silent´) |
//! | [`observation_feed_forward`] | identity | The only thing an assessment thread can say about an observation is where it landed: the submission surface takes a coordinate and nothing else, so the weight applied to the graph is chosen by the maintenance loop alone. The boundary is held by the type system rather than by validation, which is why the demonstration is that the code compiles at all with no weighted variant to call. |
//! | [`maintenance_thread_lifecycle`] | identity | cites (´claim:identity:the-maintenance-thread-obeys-a-shutdown-command-and-joins-cleanly´) |
//! | [`infra_competitive_set_arcswap`] | identity | Four threads hammering a dimension's published competitive set neither deadlock nor race one another: reading it takes no lock any other reader can hold. Every assessment consults this set, so a shared lock here would serialise the whole request path on a structure that changes rarely. |
//! | [`cell_mutable_state_has_measurement`] | identity | cites (´claim:identity:every-competitive-cell-carries-measurement-state-from-the-moment-it-is-created´) |

//! Crate-level tests for the identity layer.
//!
//! # Cross-References
//!
//! - ADR review — Identity layer acceptance criteria
//! - (´dec:memory:graph-owner´) — the one dedicated thread that owns every identity graph

use std::collections::HashMap;
use std::sync::Arc;
use std::time::Duration;

use crate::identity::{
    CellMutableState, CellOutcomeState, CellOutcomeUpdate, CompetitiveCellId, CompetitiveSetIndex, IdentityDimension,
    IdentityDimensionInfra, MaintenanceCommand, ObservationChannel,
};
use crate::owner::commands::ModelOwnerCommand;
use crate::testing::{Clock, DEFAULT_TOLERANCES, VirtualClock, assert_near};
use crate::types::DimensionId;

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Set Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a cell at the given lo and depth.
fn cell(lo: u128, depth: u8) -> CompetitiveCellId {
    CompetitiveCellId::new(lo, depth)
}

/// Read through the crate's public identity surface, a coordinate covered by
/// three nested cells resolves to all three, ordered from the finest region to
/// the root. The assessment path consumes this chain as a specific-to-general
/// sequence, so the ordering is part of the contract and not an artefact of how
/// the index happens to scan.
///
/// (´claim:identity:a-coordinates-active-cells-are-returned-as-a-containment-chain-deepest-first´)
/// ´test:crate:competitive-set-containment-chain´
#[test]
fn competitive_set_containment_chain() {
    // Create cells at depths 0, 8, 16 that all contain coordinate 0x100
    let index = CompetitiveSetIndex::from_cells([cell(0, 0), cell(0, 8), cell(0, 16)]);

    let coord = 0x100;
    let result = index.find_active(coord);

    // Should return all three in depth-descending order
    assert_eq!(result.len(), 3, "expected 3 cells in containment chain");
    assert_eq!(result[0].depth, 16, "deepest cell should be first");
    assert_eq!(result[1].depth, 8, "middle cell second");
    assert_eq!(result[2].depth, 0, "root last");
}

/// A dimension whose competitive set is still empty answers any lookup with an
/// empty chain. Since every dimension starts here, the assessment path must be
/// able to route against a dimension that has learned nothing yet and simply
/// get no identity-specific evidence back.
///
/// (´claim:identity:an-empty-competitive-set-matches-no-coordinate´)
/// ´test:crate:competitive-set-empty´
#[test]
fn competitive_set_empty() {
    let index = CompetitiveSetIndex::empty();
    let result = index.find_active(0x12345);
    assert!(result.is_empty(), "empty set should return no cells");
}

/// A coordinate landing precisely on a cell's opening bound routes into that
/// cell. Entity encoders are hashes, so boundary coordinates are as likely as
/// any other, and a cell that excluded its own first coordinate would lose
/// evidence at a predictable, repeatable spot.
///
/// (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´)
/// ´test:crate:competitive-set-exact-boundary´
#[test]
fn competitive_set_exact_boundary() {
    // Cell at depth 8 covering [0, 2^120)
    let index = CompetitiveSetIndex::from_cells([cell(0, 8)]);

    // Coordinate exactly at lo should be contained
    let result = index.find_active(0);
    assert_eq!(result.len(), 1, "coord at lo should be contained");
    assert_eq!(result[0], cell(0, 8));
}

/// The root cell spans the entire domain, so while it is the only competitive
/// cell every entity routes to it and no other. This is the coarsest useful
/// state of a dimension: one population containing everybody, which the model
/// can still learn from while finer structure is being earned.
///
/// ´claim:identity:the-root-cell-covers-the-whole-domain-so-every-coordinate-is-active-in-it´
/// ´test:crate:competitive-set-root-only´
#[test]
fn competitive_set_root_only() {
    let index = CompetitiveSetIndex::from_cells([cell(0, 0)]);

    // Any coordinate should find the root
    let result = index.find_active(u128::MAX / 2);
    assert_eq!(result.len(), 1);
    assert_eq!(result[0].depth, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell Outcome State Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Seen from outside the identity module, a neutral cell really is neutral:
/// every aggregate EWMA reads zero and both per-axis maps are empty. A cell
/// promoted into the competitive set therefore tilts nothing until outcomes are
/// attributed to it.
///
/// (´claim:identity:a-newly-tracked-cell-starts-neutral-with-no-outcome-history-of-its-own´)
/// ´test:crate:cell-outcome-state-neutral´
#[test]
fn cell_outcome_state_neutral() {
    let state = CellOutcomeState::new_neutral();
    assert_eq!(state.adverse_rate_ewma, 0.0);
    assert_eq!(state.compressed_valence_ewma, 0.0);
    assert_eq!(state.raw_valence_ewma, 0.0);
    assert!(state.per_axis_compressed.is_empty());
    assert!(state.per_axis_raw.is_empty());
}

/// Driven from a virtual clock so no real time can pass between construction
/// and update, a single adverse outcome lands the cell's adverse rate exactly
/// at the complement of the smoothing factor. The identity layer's cells smooth
/// on the same terms as the ledger's entries, which is what makes a cell-level
/// rate and a ledger-level rate comparable quantities.
///
/// (´claim:identity:a-single-outcome-moves-a-cells-ewmas-by-the-complement-of-the-smoothing-factor´)
/// ´test:crate:cell-outcome-state-ewma-formula´
#[test]
fn cell_outcome_state_ewma_formula() {
    let mut state = CellOutcomeState::new_neutral();
    let clock = VirtualClock::epoch();
    let now = clock.now();

    // Adverse outcome: positive valence (´conv:valence:sign´).
    let update = CellOutcomeUpdate {
        is_positive: true,
        compressed_valence: 0.5,
        raw_valence: 10.0,
        axis_compressed: HashMap::new(),
        axis_raw: HashMap::new(),
    };

    let lambda_l = 0.999;
    state.apply_write_decay_and_update(0.999, &now, &update, lambda_l);

    // adverse_rate = 0.999 * 0.0 + 0.001 * 1.0 = 0.001
    assert_near(
        state.adverse_rate_ewma,
        0.001,
        DEFAULT_TOLERANCES.ewma,
        "adverse_rate_ewma after single adverse update",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Observation Channel Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A coordinate offered to a dimension with room to spare is accepted, counted
/// among the pending work, and read back as submitted. Acceptance is reported
/// per observation, which is what allows the assessment path to distinguish a
/// deferred observation from a discarded one.
///
/// (´claim:identity:an-observation-handed-over-by-an-assessment-thread-reaches-the-maintenance-side-unchanged´)
/// ´test:crate:observation-try-send-success´
#[test]
fn observation_try_send_success() {
    let channel = ObservationChannel::with_capacity(10);

    // Should succeed
    assert!(channel.try_send(0x12345));
    assert_eq!(channel.len(), 1);

    // Should be retrievable
    let coord = channel.receiver().try_recv().unwrap();
    assert_eq!(coord, 0x12345);
}

/// Sends succeed up to the queue's capacity and the first one beyond it is
/// refused, moving the dimension's overflow count off zero. The count is the
/// only trace a shed observation leaves, and it is what a health report later
/// reads to tell an operator that this dimension's picture is incomplete.
///
/// (´claim:identity:every-refused-observation-is-counted-so-shed-load-is-visible-rather-than-silent´)
/// ´test:crate:observation-overflow-counter´
#[test]
fn observation_overflow_counter() {
    let channel = ObservationChannel::with_capacity(2);

    assert!(channel.try_send(1), "first send should succeed");
    assert!(channel.try_send(2), "second send should succeed");
    assert!(!channel.try_send(3), "third send should fail (overflow)");

    assert_eq!(channel.overflow_count(), 1, "overflow counter should be 1");
}

/// The only thing an assessment thread can say about an observation is where it
/// landed: the submission surface takes a coordinate and nothing else, so the
/// weight applied to the graph is chosen by the maintenance loop alone. The
/// boundary is held by the type system rather than by validation, which is why
/// the demonstration is that the code compiles at all with no weighted variant
/// to call.
///
/// ´claim:identity:the-observation-surface-carries-only-a-coordinate-so-no-caller-can-choose-a-graph-weight´
/// ´test:crate:observation-feed-forward´
#[test]
fn observation_feed_forward() {
    let channel = ObservationChannel::new();

    // The only method to send observations is try_send(u128)
    // There is no method like try_send_with_delta(coord, delta)
    // This is enforced at compile time by the type system

    // We can only send coordinates
    assert!(channel.try_send(42u128));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Maintenance Thread Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Shutdown is not merely honoured but honoured promptly: the thread joins
/// cleanly well inside a fifth of a second. The loop wakes on its own cycle
/// interval, so a shutdown that had to wait for the next tick would make
/// tearing an engine down cost a visible pause for every dimension it carried.
///
/// (´claim:identity:the-maintenance-thread-obeys-a-shutdown-command-and-joins-cleanly´)
/// ´test:crate:maintenance-thread-lifecycle´
#[test]
fn maintenance_thread_lifecycle() {
    let (model_owner_tx, _model_owner_rx) = crossbeam_channel::bounded::<ModelOwnerCommand>(64);
    let (handle, command_tx) = crate::identity::spawn_identity_maintenance_thread(
        "test-lifecycle",
        model_owner_tx,
        std::sync::Arc::new(crate::testing::SystemClock),
        std::sync::Arc::new(crate::signal::SignalCache::new(
            1,
            std::sync::Arc::new(crate::signal::SignalSchemaIndex::from_declarations(&[]).expect("empty schema")),
        )),
    );

    // Send shutdown
    command_tx.send(MaintenanceCommand::Shutdown).unwrap();

    // Thread should exit within 200ms (generous timeout)
    let start = std::time::Instant::now();
    let result = handle.join();
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "thread should exit cleanly");
    assert!(elapsed < Duration::from_millis(200), "shutdown should be fast");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension Infra Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn test_dimension() -> IdentityDimension {
    IdentityDimension::new(DimensionId(1), "test", "test hierarchy", "prefix groups", 128, 24, |_| 0)
}

/// Four threads hammering a dimension's published competitive set neither
/// deadlock nor race one another: reading it takes no lock any other reader can
/// hold. Every assessment consults this set, so a shared lock here would
/// serialise the whole request path on a structure that changes rarely.
///
/// ´claim:identity:many-assessment-threads-read-the-published-competitive-set-at-once-without-contending´
/// ´test:crate:infra-competitive-set-arcswap´
#[test]
fn infra_competitive_set_arcswap() {
    let (tx, _rx) = crossbeam_channel::bounded(100);
    let infra = Arc::new(IdentityDimensionInfra::new(test_dimension(), tx));

    // Simulate concurrent loads from multiple threads
    let handles: Vec<_> = (0..4)
        .map(|_| {
            let infra = Arc::clone(&infra);
            std::thread::spawn(move || {
                for _ in 0..100 {
                    let _guard = infra.load_competitive_set();
                    // Just load and drop
                }
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }

    // No deadlocks or panics
}

/// Cell state as the rest of the crate sees it always includes a measurement
/// side, arriving with its step count at zero. There is no second, lighter kind
/// of tracked cell to branch on: anything in the competitive set can record
/// what Sentinels observe about it.
///
/// (´claim:identity:every-competitive-cell-carries-measurement-state-from-the-moment-it-is-created´)
/// ´test:crate:cell-mutable-state-has-measurement´
#[test]
fn cell_mutable_state_has_measurement() {
    let state = CellMutableState::new();
    assert_eq!(state.measurement.step_count, 0);
}
