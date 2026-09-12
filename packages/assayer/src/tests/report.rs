// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_report_has_no_report`] | dossier | A freshly built index says plainly that it holds no report. Emptiness is a state the index knows itself to be in rather than something a caller must infer from a cell count, which is what lets "this Sentinel has not reported yet" be distinguished from "this Sentinel reported nothing of interest". |
//! | [`route_on_empty_index_returns_none`] | dossier | Routing through an index that has received nothing yields nothing. Silence is answered with silence rather than with a manufactured root entry whose scores are all zero, because a caller offered such an entry would read a Sentinel that has never spoken as one reporting perfect calm. |
//! | [`route_with_root_only_returns_root`] | dossier | With only the root reported, every coordinate finds it — zero, the largest representable value, the midpoint and an arbitrary one alike all return the same entry. Because the root spans the whole space, an index that has any report at all can always answer, and the depth-walk's fallback never runs off the bottom. |
//! | [`route_depth_walk_returns_deepest`] | dossier | Where several reported cells contain a coordinate, the deepest one wins, and where the deeper ones do not, the walk keeps descending until something does: against cells at depths 0, 4 and 8, one coordinate lands on the depth-8 entry, another that shares only the first four bits lands on depth 4, and one sharing neither falls back to the root. The deepest containing cell is the most specific account anyone has of that coordinate, and the ancestors behind it are what stop the answer from ever being nothing. |
//! | [`depth_walk_uses_dyadic_ancestor_lo`] | dossier | The key the walk probes at each level is the coordinate's own dyadic ancestor at that depth: a cell filed under that ancestor is found, and the key the router builds is identical to the one computed directly. This is why routing costs one map lookup per depth rather than a search — the identity of the cell that could contain a coordinate is arithmetic, not a question to be asked of the data. |
//! | [`sentinel_slot_new_has_empty_report`] | dossier | A slot created for a newly registered Sentinel already holds an index, and that index is an empty one. Registration and first reception are separate events, so the window between them has a defined answer instead of an absent slot that every reader would have to guard against. |
//! | [`sentinel_slot_bootstrap_lifecycle`] | dossier | Bootstrap is off in a new slot, on once an accumulator is installed, and off again when cleared — the flag follows the accumulator rather than drifting from it. Readers decide whether a Sentinel's numbers are yet worth acting on from that one flag, so a slot that has finished warming must never still read as warming. |
//! | [`bootstrap_accumulator_fields`] | dossier | A new accumulator is sized to the number of dimensions it will summarise and empty of observations: running means and second moments with a slot apiece, a count of zero against its target, and no claim to be complete. Warming is something to be earned observation by observation, so the accumulator starts owing its target in full rather than part-way there. |
//! | [`axis_score_types_are_clone_debug`] | dossier | A single axis snapshot and a full four-axis set can both be copied and rendered for inspection. Score data is read far from where it is stored — in a log line, in a health response, in a feature extractor working on its own copy — and requiring a borrow of the live index for any of that would tie readers to the lifetime of a report that is about to be replaced. |
//! | [`report_ack_is_clone_debug`] | dossier | An acknowledgement goes further than the score types: it compares equal to its own clone and prints the counts it carries. Equality is what lets the copy stored on a slot be checked against the one returned to a caller, and the printed form is how those counts reach a log without a bespoke formatter. |
//! | [`coordination_entry_is_clone_debug`] | dossier | cites (´claim:dossier:report-types-can-be-copied-away-from-the-index-and-rendered-for-inspection´) |
//! | [`report_level_data_is_clone_debug`] | dossier | cites (´claim:dossier:report-types-can-be-copied-away-from-the-index-and-rendered-for-inspection´) |
//! | [`ledger_key_from_coordinate`] | dossier | A key derived from a coordinate keeps as much of it as the depth affords: at depth zero none, so every coordinate gives the root; at depth 128 all of it, so the key is the coordinate itself; in between, exactly the leading bits that depth names. Depth is thus a dial between the coarsest and the finest possible account of where something happened, with the same derivation at every setting. |
//! | [`ledger_key_contains`] | dossier | Containment is decided by that same prefix: a depth-8 key admits every coordinate agreeing with it in the first eight bits, right up to the one whose remaining bits are all set, and refuses the coordinate that differs there. Membership is therefore a property of a coordinate's own bits rather than of an arithmetic range that could overflow at the top of the space. |
//! | [`ledger_key_hi`] | dossier | A cell's upper bound is implied by its depth rather than stored: the root reaches the largest representable coordinate, a depth-128 cell reaches only itself, and a depth-8 cell reaches its start plus the width that eight halvings leave. Keeping the width implicit means a key and its span can never disagree, and the root's bound is expressible even though its exclusive end would not be. |
//! | [`route_exact_boundary_lo`] | dossier | A cell owns its own first coordinate: querying exactly at a depth-8 cell's start returns that cell and not the root above it. The endpoints are where an off-by-one would hide, and the one at the bottom would push a whole cell's opening coordinate up to its ancestor, losing the finest account of it for no visible reason. |
//! | [`route_exact_boundary_hi_minus_1`] | dossier | cites (´claim:dossier:a-cell-owns-both-its-first-and-its-last-coordinate´) |
//! | [`route_just_past_boundary`] | dossier | One step past the end and the answer changes: the first coordinate outside a depth-8 cell falls through to the root rather than being absorbed by its neighbour. A cell speaks only for the region it was reported over, so the boundary is sharp and the coordinate beyond it gets the coarser account that is genuinely all anyone has of it. |
//! | [`route_many_depths`] | dossier | cites (´claim:dossier:routing-returns-the-deepest-containing-cell-and-falls-back-through-its-ancestors´) |
//! | [`route_max_depth`] | dossier | At the bottom of the hierarchy a cell has narrowed to a single coordinate: the depth-128 entry answers for the coordinate it was keyed on and for nothing else, and the coordinate one greater falls back to the root. The space runs out of bits before the hierarchy runs out of levels, so the deepest cell is a point and routing at full depth is an exact-match question. |
//! | [`constructed_has_report`] | dossier | Building an index from components marks it as carrying a report. Having a report is a fact about provenance rather than about population — the index is stamped at construction, not inferred from whether any cells happen to be present — which is why a batch that reported almost nothing is still distinguishable from no batch at all. |
//! | [`cells_accessor`] | dossier | Every cell handed to the constructor is still there to be enumerated afterwards, ancestors at successive depths included. Cell-set maintenance works from that enumeration rather than from routing, so a cell quietly dropped on the way in would be a cell the Ledger never learns to track. |
//! | [`coordination_accessor`] | dossier | Coordination entries are kept as a sequence, not a map: both survive, and the one given first is still first when read back. Unlike cells they are not keyed by a region — several may describe the same depth — so order is the only identity they have, and preserving it is what lets a caller match an entry against the report it came from. |
//! | [`level_data_correct`] | dossier | The batch-level summary is carried into the index field for field — the competitive count, the full set size, the range of depths reached, the number of plateaus and the total importance all come back as given. This summary is the Sentinel's own account of the shape of its batch, and it is not recomputable from the cells that were reported, since those are only the ones worth naming individually. |
//! | [`max_depth_tracks`] | dossier | An index reports the greatest depth among the cells it holds. That figure is where the depth-walk starts, so it must not understate the index — a cell filed below the stated maximum would never be probed and would be invisible to routing while sitting in the map. |
//! | [`axis_score_set_all_zeroes`] | dossier | A default score set is zero everywhere — every field of novelty, and the peak of each of the other three axes besides. This is the value sanitisation falls back to, so it must represent the absence of a signal rather than an arbitrary one: zeroes read as "nothing to report here", which is the honest account of an axis whose numbers could not be trusted. |
//! | [`axis_score_set_independent`] | dossier | The four axes are stored side by side and never share storage: writing novelty's peak and accumulator leaves the other three untouched, and writing displacement's mean leaves novelty's own mean at zero. This separation is what makes per-axis sanitisation meaningful — zeroing one axis can only be a containment measure if the axes were independent to begin with. |
//! | [`slot_report_swap`] | dossier | Storing an index into a slot replaces what was there entirely: a slot that read as empty then yields the new report, with its one cell present and routable. The unit of publication is a whole index, so no reader can catch a slot mid-update holding some cells from one batch and some from the next. |
//! | [`slot_ingestion_lock`] | dossier | The ingestion lock is released when its guard is dropped and can be taken again straight afterwards. Every batch for a Sentinel passes through this one lock, so a guard that outlived its ingestion — or a lock that stayed held after a failed one — would stop that Sentinel from ever being heard from again. |

//! Tests for report types and Sentinel slots.
//!
//! A received report becomes a `ReportIndex`: a map from dyadic cell keys to the
//! Sentinel's account of each cell, plus the summary data describing the batch as
//! a whole. Its one interesting operation is routing — given a coordinate, walk
//! from the deepest reported level down to the root and return the first cell
//! that claims it — and most of what follows pins down where the edges of a cell
//! lie and what happens just past them.
//!
//! The slot is where such an index lives between reports. It swaps whole indexes
//! rather than editing one in place, so a reader either sees the previous report
//! or the next one and never a half-written mixture of the two.

use std::collections::HashMap;

use crate::feature::bootstrap::BootstrapAccumulator;
use crate::report::{
    AxisScoreSet, AxisScoreSnapshot, CoordinationEntry, ReportAck, ReportCellEntry, ReportIndex, ReportLevelData, SentinelSlot,
};
use crate::testing::{DEFAULT_TOLERANCES, DEPTH_4_LO, DEPTH_8_LO, DEPTH_12_LO, GOLDEN_COORD, assert_near};
use crate::types::{LedgerKey, dyadic_ancestor_lo};

// ═══════════════════════════════════════════════════════════════════════════════
// ReportIndex Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A freshly built index says plainly that it holds no report. Emptiness is a
/// state the index knows itself to be in rather than something a caller must
/// infer from a cell count, which is what lets "this Sentinel has not reported
/// yet" be distinguished from "this Sentinel reported nothing of interest".
///
/// ´claim:dossier:an-index-that-has-received-nothing-says-so´
/// ´test:crate:empty-report-has-no-report´
#[test]
fn empty_report_has_no_report() {
    let index = ReportIndex::empty();
    assert!(!index.has_report(), "empty index should have no report");
}

/// Routing through an index that has received nothing yields nothing. Silence is
/// answered with silence rather than with a manufactured root entry whose scores
/// are all zero, because a caller offered such an entry would read a Sentinel
/// that has never spoken as one reporting perfect calm.
///
/// ´claim:dossier:an-empty-index-routes-nothing-rather-than-inventing-a-cell´
/// ´test:crate:route-on-empty-index-returns-none´
#[test]
fn route_on_empty_index_returns_none() {
    let index = ReportIndex::empty();
    assert!(index.route(GOLDEN_COORD).is_none(), "route on empty index should return None");
}

/// With only the root reported, every coordinate finds it — zero, the largest
/// representable value, the midpoint and an arbitrary one alike all return the
/// same entry. Because the root spans the whole space, an index that has any
/// report at all can always answer, and the depth-walk's fallback never runs off
/// the bottom.
///
/// ´claim:dossier:the-root-cell-covers-the-whole-space-so-any-coordinate-finds-it´
/// ´test:crate:route-with-root-only-returns-root´
#[test]
fn route_with_root_only_returns_root() {
    // Build an index with only the root cell (depth 0)
    let mut cells = HashMap::new();
    let root_key = LedgerKey::new(0, 0); // Root: lo=0, depth=0
    let root_entry = ReportCellEntry {
        depth: 0,
        sample_count: 100,
        is_competitive: true,
        ..Default::default()
    };
    cells.insert(root_key, root_entry);

    let index = ReportIndex::from_components(
        cells,
        Vec::new(),
        ReportLevelData::default(),
        0, // max_depth = 0
    );

    // Any coordinate should match the root
    let test_coords = [
        0_u128,
        u128::MAX,
        0x8000_0000_0000_0000_0000_0000_0000_0000_u128,
        GOLDEN_COORD,
    ];

    for coord in test_coords {
        let entry = index.route(coord).expect("root-only index should match all coords");
        assert_eq!(entry.depth, 0, "should return root entry");
        assert_eq!(entry.sample_count, 100);
    }
}

/// Where several reported cells contain a coordinate, the deepest one wins, and
/// where the deeper ones do not, the walk keeps descending until something does:
/// against cells at depths 0, 4 and 8, one coordinate lands on the depth-8 entry,
/// another that shares only the first four bits lands on depth 4, and one sharing
/// neither falls back to the root. The deepest containing cell is the most
/// specific account anyone has of that coordinate, and the ancestors behind it are
/// what stop the answer from ever being nothing.
///
/// ´claim:dossier:routing-returns-the-deepest-containing-cell-and-falls-back-through-its-ancestors´
/// ´test:crate:route-depth-walk-returns-deepest´
#[test]
fn route_depth_walk_returns_deepest() {
    // Build an index with cells at depths 0, 4, and 8 keyed off GOLDEN_COORD.
    // At depth 4: top 4 bits of GOLDEN_COORD (0x1) yield DEPTH_4_LO.
    // At depth 8: top 8 bits (0x12) yield DEPTH_8_LO.

    // Build cells at depths 0, 4, 8
    let mut cells = HashMap::new();

    // Root (depth 0)
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 10,
            ..Default::default()
        },
    );

    // Depth 4: covers top 4 bits
    cells.insert(
        LedgerKey::new(DEPTH_4_LO, 4),
        ReportCellEntry {
            depth: 4,
            sample_count: 40,
            ..Default::default()
        },
    );

    // Depth 8: covers top 8 bits
    cells.insert(
        LedgerKey::new(DEPTH_8_LO, 8),
        ReportCellEntry {
            depth: 8,
            sample_count: 80,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(
        cells,
        Vec::new(),
        ReportLevelData::default(),
        8, // max_depth = 8
    );

    // Coordinate inside depth-8 cell should return depth-8 entry
    let entry = index.route(GOLDEN_COORD).expect("should find entry");
    assert_eq!(entry.depth, 8, "should return depth-8 entry (deepest)");
    assert_eq!(entry.sample_count, 80);

    // Coordinate inside depth-4 (top 4 bits = 0x1) but outside depth-8 (top 8 bits != 0x12)
    let coord_depth_4_only = 0x1F00_0000_0000_0000_0000_0000_0000_0000_u128;
    let entry = index.route(coord_depth_4_only).expect("should find entry");
    assert_eq!(entry.depth, 4, "should return depth-4 entry");
    assert_eq!(entry.sample_count, 40);

    // Coordinate outside both depth-4 and depth-8 should return root
    // Use a coordinate with different top 4 bits (not 0x1)
    let coord_root_only = 0xF000_0000_0000_0000_0000_0000_0000_0000_u128;
    let entry = index.route(coord_root_only).expect("should find root entry");
    assert_eq!(entry.depth, 0, "should return root entry");
    assert_eq!(entry.sample_count, 10);
}

/// The key the walk probes at each level is the coordinate's own dyadic ancestor
/// at that depth: a cell filed under that ancestor is found, and the key the
/// router builds is identical to the one computed directly. This is why routing
/// costs one map lookup per depth rather than a search — the identity of the cell
/// that could contain a coordinate is arithmetic, not a question to be asked of
/// the data.
///
/// ´claim:dossier:the-key-a-route-probes-is-the-coordinates-dyadic-ancestor-at-that-depth´
/// ´test:crate:depth-walk-uses-dyadic-ancestor-lo´
#[test]
fn depth_walk_uses_dyadic_ancestor_lo() {
    // Verify that route() uses dyadic_ancestor_lo correctly
    // by checking that the key construction matches

    let coord = GOLDEN_COORD;

    // Build index with a cell at depth 16
    let mut cells = HashMap::new();

    // Root
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            ..Default::default()
        },
    );

    // Depth 16: use dyadic_ancestor_lo to compute the expected key
    let expected_lo = dyadic_ancestor_lo(coord, 16);
    let depth_16_key = LedgerKey::new(expected_lo, 16);
    cells.insert(
        depth_16_key,
        ReportCellEntry {
            depth: 16,
            sample_count: 160,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 16);

    // Route should find the depth-16 cell
    let entry = index.route(coord).expect("should find depth-16 entry");
    assert_eq!(entry.depth, 16);
    assert_eq!(entry.sample_count, 160);

    // Verify the key computation matches
    let key_from_route = LedgerKey::from_coordinate(coord, 16);
    assert_eq!(key_from_route.lo, expected_lo);
    assert_eq!(key_from_route.depth, 16);
}

// ═══════════════════════════════════════════════════════════════════════════════
// SentinelSlot Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A slot created for a newly registered Sentinel already holds an index, and
/// that index is an empty one. Registration and first reception are separate
/// events, so the window between them has a defined answer instead of an absent
/// slot that every reader would have to guard against.
///
/// ´claim:dossier:a-fresh-slot-holds-an-empty-index-until-a-report-arrives´
/// ´test:crate:sentinel-slot-new-has-empty-report´
#[test]
fn sentinel_slot_new_has_empty_report() {
    let slot = SentinelSlot::new();
    let report = slot.report_index.load();
    assert!(!report.has_report(), "new SentinelSlot should have empty report");
}

/// Bootstrap is off in a new slot, on once an accumulator is installed, and off
/// again when cleared — the flag follows the accumulator rather than drifting
/// from it. Readers decide whether a Sentinel's numbers are yet worth acting on
/// from that one flag, so a slot that has finished warming must never still read
/// as warming.
///
/// ´claim:dossier:the-bootstrap-flag-follows-the-accumulator-on-when-started-and-off-when-cleared´
/// ´test:crate:sentinel-slot-bootstrap-lifecycle´
#[test]
fn sentinel_slot_bootstrap_lifecycle() {
    let slot = SentinelSlot::new();

    // Initially bootstrap is not active
    assert!(!slot.is_bootstrap_active());

    // Start bootstrap
    let accumulator = BootstrapAccumulator::new(10, 100);
    slot.start_bootstrap(accumulator);
    assert!(slot.is_bootstrap_active());

    // Clear bootstrap
    slot.clear_bootstrap();
    assert!(!slot.is_bootstrap_active());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Type Trait Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A new accumulator is sized to the number of dimensions it will summarise and
/// empty of observations: running means and second moments with a slot apiece, a
/// count of zero against its target, and no claim to be complete. Warming is
/// something to be earned observation by observation, so the accumulator starts
/// owing its target in full rather than part-way there.
///
/// ´claim:dossier:a-new-bootstrap-accumulator-is-sized-to-its-dimensions-and-empty-of-observations´
/// ´test:crate:bootstrap-accumulator-fields´
#[test]
fn bootstrap_accumulator_fields() {
    let acc = BootstrapAccumulator::new(5, 100);
    assert_eq!(acc.running_mean.len(), 5);
    assert_eq!(acc.running_m2.len(), 5);
    assert_eq!(acc.count(), 0);
    assert_eq!(acc.target(), 100);
    assert!(!acc.is_complete());
}

/// A single axis snapshot and a full four-axis set can both be copied and
/// rendered for inspection. Score data is read far from where it is stored — in a
/// log line, in a health response, in a feature extractor working on its own copy
/// — and requiring a borrow of the live index for any of that would tie readers
/// to the lifetime of a report that is about to be replaced.
///
/// ´claim:dossier:report-types-can-be-copied-away-from-the-index-and-rendered-for-inspection´
/// ´test:crate:axis-score-types-are-clone-debug´
#[test]
fn axis_score_types_are_clone_debug() {
    // AxisScoreSnapshot
    let snapshot = AxisScoreSnapshot::default();
    let _cloned = snapshot.clone();
    let _debug = format!("{snapshot:?}");

    // AxisScoreSet
    let set = AxisScoreSet::default();
    let _cloned = set.clone();
    let _debug = format!("{set:?}");
}

/// An acknowledgement goes further than the score types: it compares equal to its
/// own clone and prints the counts it carries. Equality is what lets the copy
/// stored on a slot be checked against the one returned to a caller, and the
/// printed form is how those counts reach a log without a bespoke formatter.
///
/// ´claim:dossier:an-acknowledgement-compares-equal-to-its-clone-and-prints-its-counts´
/// ´test:crate:report-ack-is-clone-debug´
#[test]
fn report_ack_is_clone_debug() {
    let ack = ReportAck {
        cells_ingested: 10,
        degraded_cells: 2,
        coordination_contexts: 3,
        cells_created: 0,
        cells_deleted: 0,
    };
    let cloned = ack.clone();
    assert_eq!(ack, cloned);

    let debug = format!("{ack:?}");
    assert!(debug.contains("10"));
    assert!(debug.contains('2'));
    assert!(debug.contains('3'));
}

/// Coordination entries travel and print on the same terms as cell scores do.
/// They are the part of a report describing cells acting in concert, and a
/// diagnostic that could show the per-cell picture but not the coordinated one
/// would be blind to exactly the behaviour coordination exists to surface.
///
/// (´claim:dossier:report-types-can-be-copied-away-from-the-index-and-rendered-for-inspection´)
/// ´test:crate:coordination-entry-is-clone-debug´
#[test]
fn coordination_entry_is_clone_debug() {
    let entry = CoordinationEntry::default();
    let _cloned = entry.clone();
    let _debug = format!("{entry:?}");
}

/// The batch-level summary is portable and printable too. It is the smallest
/// useful description of a report — how many cells were competitive, how deep the
/// analysis reached, how much importance was in play — and it is often all a
/// caller wants, so it must be obtainable without dragging the cell map along
/// with it.
///
/// (´claim:dossier:report-types-can-be-copied-away-from-the-index-and-rendered-for-inspection´)
/// ´test:crate:report-level-data-is-clone-debug´
#[test]
fn report_level_data_is_clone_debug() {
    let data = ReportLevelData::default();
    let _cloned = data.clone();
    let _debug = format!("{data:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// LedgerKey Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A key derived from a coordinate keeps as much of it as the depth affords: at
/// depth zero none, so every coordinate gives the root; at depth 128 all of it,
/// so the key is the coordinate itself; in between, exactly the leading bits that
/// depth names. Depth is thus a dial between the coarsest and the finest possible
/// account of where something happened, with the same derivation at every setting.
///
/// ´claim:dossier:depth-decides-how-many-leading-bits-of-a-coordinate-its-key-keeps´
/// ´test:crate:ledger-key-from-coordinate´
#[test]
fn ledger_key_from_coordinate() {
    let coord = GOLDEN_COORD;

    // Depth 0 should always give lo=0
    let key_0 = LedgerKey::from_coordinate(coord, 0);
    assert_eq!(key_0.lo, 0);
    assert_eq!(key_0.depth, 0);

    // Depth 8 should give top 8 bits
    let key_8 = LedgerKey::from_coordinate(coord, 8);
    assert_eq!(key_8.lo, dyadic_ancestor_lo(coord, 8));
    assert_eq!(key_8.depth, 8);

    // Depth 128 should give the coordinate itself
    let key_128 = LedgerKey::from_coordinate(coord, 128);
    assert_eq!(key_128.lo, coord);
    assert_eq!(key_128.depth, 128);
}

/// Containment is decided by that same prefix: a depth-8 key admits every
/// coordinate agreeing with it in the first eight bits, right up to the one whose
/// remaining bits are all set, and refuses the coordinate that differs there.
/// Membership is therefore a property of a coordinate's own bits rather than of
/// an arithmetic range that could overflow at the top of the space.
///
/// ´claim:dossier:a-key-contains-exactly-the-coordinates-sharing-its-leading-bits´
/// ´test:crate:ledger-key-contains´
#[test]
fn ledger_key_contains() {
    let coord = DEPTH_8_LO;
    let key = LedgerKey::from_coordinate(coord, 8);

    // The coordinate should be contained
    assert!(key.contains(coord));

    // Any coordinate with the same top 8 bits (0x12) should be contained
    let same_prefix = 0x12FF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_FFFF_u128;
    assert!(key.contains(same_prefix));

    // Coordinate with different top 8 bits should not be contained
    let different_prefix = 0x1300_0000_0000_0000_0000_0000_0000_0000_u128;
    assert!(!key.contains(different_prefix));
}

/// A cell's upper bound is implied by its depth rather than stored: the root
/// reaches the largest representable coordinate, a depth-128 cell reaches only
/// itself, and a depth-8 cell reaches its start plus the width that eight
/// halvings leave. Keeping the width implicit means a key and its span can never
/// disagree, and the root's bound is expressible even though its exclusive end
/// would not be.
///
/// ´claim:dossier:a-cells-upper-bound-follows-from-its-depth-rather-than-being-stored´
/// ´test:crate:ledger-key-hi´
#[test]
fn ledger_key_hi() {
    // Root key covers entire space
    let root = LedgerKey::new(0, 0);
    assert_eq!(root.hi(), u128::MAX);

    // Depth-128 key covers single point
    let point = 0x1234_u128;
    let single = LedgerKey::new(point, 128);
    assert_eq!(single.hi(), point);

    // Depth-8 key: hi = lo + 2^120 - 1
    let key_8 = LedgerKey::new(DEPTH_8_LO, 8);
    let expected_hi = DEPTH_8_LO + (1_u128 << 120) - 1;
    assert_eq!(key_8.hi(), expected_hi);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Routing Tests (completion)
// ═══════════════════════════════════════════════════════════════════════════════

/// A cell owns its own first coordinate: querying exactly at a depth-8 cell's
/// start returns that cell and not the root above it. The endpoints are where an
/// off-by-one would hide, and the one at the bottom would push a whole cell's
/// opening coordinate up to its ancestor, losing the finest account of it for no
/// visible reason.
///
/// ´claim:dossier:a-cell-owns-both-its-first-and-its-last-coordinate´
/// ´test:crate:route-exact-boundary-lo´
#[test]
fn route_exact_boundary_lo() {
    // Verify routing when coord exactly equals cell.lo
    let lo = DEPTH_8_LO;
    let mut cells = HashMap::new();

    // Root
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 10,
            ..Default::default()
        },
    );

    // Depth-8 cell
    cells.insert(
        LedgerKey::new(lo, 8),
        ReportCellEntry {
            depth: 8,
            sample_count: 80,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 8);

    // Query exactly at cell.lo should match the depth-8 cell
    let entry = index.route(lo).expect("should find entry at boundary");
    assert_eq!(entry.depth, 8);
    assert_eq!(entry.sample_count, 80);
}

/// The far end is owned too: the last coordinate before the next depth-8 cell
/// begins still routes into this one. Taken with the lower endpoint, the cell's
/// span is closed at both ends and its width is exactly the one its depth
/// promises — no coordinate in it is orphaned to an ancestor.
///
/// (´claim:dossier:a-cell-owns-both-its-first-and-its-last-coordinate´)
/// ´test:crate:route-exact-boundary-hi-minus-1´
#[test]
fn route_exact_boundary_hi_minus_1() {
    // Verify routing for the last coordinate in a cell (hi boundary)
    let lo = DEPTH_8_LO;
    let width = 1_u128 << 120; // Cell width at depth 8
    let last_coord = lo + width - 1;

    let mut cells = HashMap::new();

    // Root
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 10,
            ..Default::default()
        },
    );

    // Depth-8 cell
    cells.insert(
        LedgerKey::new(lo, 8),
        ReportCellEntry {
            depth: 8,
            sample_count: 80,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 8);

    // Last coordinate in cell should still match depth-8
    let entry = index.route(last_coord).expect("should find entry at hi-1");
    assert_eq!(entry.depth, 8, "last coord in cell should route to depth-8");
    assert_eq!(entry.sample_count, 80);
}

/// One step past the end and the answer changes: the first coordinate outside a
/// depth-8 cell falls through to the root rather than being absorbed by its
/// neighbour. A cell speaks only for the region it was reported over, so the
/// boundary is sharp and the coordinate beyond it gets the coarser account that
/// is genuinely all anyone has of it.
///
/// ´claim:dossier:the-coordinate-just-past-a-cell-falls-through-to-an-ancestor´
/// ´test:crate:route-just-past-boundary´
#[test]
fn route_just_past_boundary() {
    // First coordinate past the cell boundary should fall back to root
    let lo = DEPTH_8_LO;
    let width = 1_u128 << 120;
    let first_outside = lo + width;

    let mut cells = HashMap::new();

    // Root
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 10,
            ..Default::default()
        },
    );

    // Depth-8 cell
    cells.insert(
        LedgerKey::new(lo, 8),
        ReportCellEntry {
            depth: 8,
            sample_count: 80,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 8);

    // First coord outside cell should fall back to root
    let entry = index.route(first_outside).expect("should fall back to root");
    assert_eq!(entry.depth, 0, "past boundary should route to root");
    assert_eq!(entry.sample_count, 10);
}

/// A deep stack of ancestors changes nothing about which one answers: with the
/// coordinate covered at every even depth from zero to ten, it is the depth-ten
/// entry that comes back. Six candidates or two, the walk starts at the bottom
/// and stops at the first hit, so a richly reported region is described by its
/// finest cell rather than by an arbitrary one of its ancestors.
///
/// (´claim:dossier:routing-returns-the-deepest-containing-cell-and-falls-back-through-its-ancestors´)
/// ´test:crate:route-many-depths´
#[test]
fn route_many_depths() {
    // Test routing with entries at depths 0, 2, 4, 6, 8, 10
    let coord = GOLDEN_COORD;
    let mut cells = HashMap::new();

    let depths = [0, 2, 4, 6, 8, 10];
    for depth in depths {
        let lo = dyadic_ancestor_lo(coord, depth);
        cells.insert(
            LedgerKey::new(lo, depth),
            ReportCellEntry {
                depth,
                sample_count: depth as usize * 10,
                ..Default::default()
            },
        );
    }

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 10);

    // coord should route to depth 10 (deepest)
    let entry = index.route(coord).expect("should find deepest entry");
    assert_eq!(entry.depth, 10);
    assert_eq!(entry.sample_count, 100);
}

/// At the bottom of the hierarchy a cell has narrowed to a single coordinate: the
/// depth-128 entry answers for the coordinate it was keyed on and for nothing
/// else, and the coordinate one greater falls back to the root. The space runs out
/// of bits before the hierarchy runs out of levels, so the deepest cell is a point
/// and routing at full depth is an exact-match question.
///
/// ´claim:dossier:at-full-depth-a-cell-is-a-single-coordinate-and-its-neighbour-lies-outside-it´
/// ´test:crate:route-max-depth´
#[test]
fn route_max_depth() {
    // Test routing at maximum depth (128)
    let coord = GOLDEN_COORD;
    let mut cells = HashMap::new();

    // Root
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 10,
            ..Default::default()
        },
    );

    // Depth-128: the cell is exactly the coordinate itself
    cells.insert(
        LedgerKey::new(coord, 128),
        ReportCellEntry {
            depth: 128,
            sample_count: 1280,
            ..Default::default()
        },
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 128);

    // Should route to depth-128
    let entry = index.route(coord).expect("should find depth-128 entry");
    assert_eq!(entry.depth, 128);
    assert_eq!(entry.sample_count, 1280);

    // A different coordinate should fall back to root
    let other = coord.wrapping_add(1);
    let entry = index.route(other).expect("should fall back to root");
    assert_eq!(entry.depth, 0);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Construction Tests (completion)
// ═══════════════════════════════════════════════════════════════════════════════

/// Building an index from components marks it as carrying a report. Having a
/// report is a fact about provenance rather than about population — the index is
/// stamped at construction, not inferred from whether any cells happen to be
/// present — which is why a batch that reported almost nothing is still
/// distinguishable from no batch at all.
///
/// ´claim:dossier:an-index-built-from-components-is-stamped-as-carrying-a-report´
/// ´test:crate:constructed-has-report´
#[test]
fn constructed_has_report() {
    let mut cells = HashMap::new();
    cells.insert(LedgerKey::new(0, 0), ReportCellEntry::default());

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 0);
    assert!(index.has_report(), "constructed index should have report");
}

/// Every cell handed to the constructor is still there to be enumerated
/// afterwards, ancestors at successive depths included. Cell-set maintenance
/// works from that enumeration rather than from routing, so a cell quietly
/// dropped on the way in would be a cell the Ledger never learns to track.
///
/// ´claim:dossier:the-cell-map-holds-every-cell-the-index-was-built-from´
/// ´test:crate:cells-accessor´
#[test]
fn cells_accessor() {
    let mut cells = HashMap::new();
    cells.insert(LedgerKey::new(0, 0), ReportCellEntry::default());
    cells.insert(
        LedgerKey::new(0x8000_0000_0000_0000_0000_0000_0000_0000_u128, 1),
        ReportCellEntry::default(),
    );
    cells.insert(
        LedgerKey::new(0xC000_0000_0000_0000_0000_0000_0000_0000_u128, 2),
        ReportCellEntry::default(),
    );

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 2);
    assert_eq!(index.cells().len(), 3, "cells accessor should return all cells");
}

/// Coordination entries are kept as a sequence, not a map: both survive, and the
/// one given first is still first when read back. Unlike cells they are not keyed
/// by a region — several may describe the same depth — so order is the only
/// identity they have, and preserving it is what lets a caller match an entry
/// against the report it came from.
///
/// ´claim:dossier:coordination-entries-are-kept-in-the-order-they-arrived´
/// ´test:crate:coordination-accessor´
#[test]
fn coordination_accessor() {
    let coordination = vec![
        CoordinationEntry {
            depth: 4,
            cells_reporting: 10,
            ..Default::default()
        },
        CoordinationEntry {
            depth: 8,
            cells_reporting: 20,
            ..Default::default()
        },
    ];

    let index = ReportIndex::from_components(HashMap::new(), coordination, ReportLevelData::default(), 8);

    assert_eq!(
        index.coordination().len(),
        2,
        "coordination accessor should return all entries"
    );
    assert_eq!(index.coordination()[0].depth, 4);
    assert_eq!(index.coordination()[1].cells_reporting, 20);
}

/// The batch-level summary is carried into the index field for field — the
/// competitive count, the full set size, the range of depths reached, the number
/// of plateaus and the total importance all come back as given. This summary is
/// the Sentinel's own account of the shape of its batch, and it is not
/// recomputable from the cells that were reported, since those are only the ones
/// worth naming individually.
///
/// ´claim:dossier:the-batch-level-summary-is-carried-into-the-index-field-for-field´
/// ´test:crate:level-data-correct´
#[test]
fn level_data_correct() {
    let level_data = ReportLevelData {
        competitive_cell_count: 15,
        full_set_size: 100,
        depth_range: (2, 12),
        plateau_count: 3,
        total_importance: 999.0,
        contour_cell_count: 44,
        splits_since_last_report: 5,
        net_removals_since_last_report: 2,
        importance_range: (0.25, 4.0),
        v_depth_range: (1, 7),
        degenerate_cells_skipped: 6,
    };

    let index = ReportIndex::from_components(HashMap::new(), Vec::new(), level_data, 12);

    let data = index.level_data();
    assert_eq!(data.competitive_cell_count, 15);
    assert_eq!(data.full_set_size, 100);
    assert_eq!(data.depth_range, (2, 12));
    assert_eq!(data.plateau_count, 3);
    assert_near(data.total_importance, 999.0, DEFAULT_TOLERANCES.default, "total_importance");
    assert_eq!(data.contour_cell_count, 44);
    assert_eq!(data.splits_since_last_report, 5);
    assert_eq!(data.net_removals_since_last_report, 2);
    assert_eq!(data.importance_range, (0.25, 4.0));
    assert_eq!(data.v_depth_range, (1, 7));
    assert_eq!(data.degenerate_cells_skipped, 6);
}

/// An index reports the greatest depth among the cells it holds. That figure is
/// where the depth-walk starts, so it must not understate the index — a cell
/// filed below the stated maximum would never be probed and would be invisible to
/// routing while sitting in the map.
///
/// ´claim:dossier:the-stated-maximum-depth-is-where-the-depth-walk-begins´
/// ´test:crate:max-depth-tracks´
#[test]
fn max_depth_tracks() {
    // Build index with cells at depths 0, 4, 12 keyed off GOLDEN_COORD.
    let mut cells = HashMap::new();

    for (lo, depth) in [(0_u128, 0_u8), (DEPTH_4_LO, 4), (DEPTH_12_LO, 12)] {
        cells.insert(LedgerKey::new(lo, depth), ReportCellEntry::default());
    }

    let index = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 12);
    assert_eq!(index.max_depth(), 12, "max_depth should track highest depth");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Score Type Tests (completion)
// ═══════════════════════════════════════════════════════════════════════════════

/// A default score set is zero everywhere — every field of novelty, and the peak
/// of each of the other three axes besides. This is the value sanitisation falls
/// back to, so it must represent the absence of a signal rather than an
/// arbitrary one: zeroes read as "nothing to report here", which is the honest
/// account of an axis whose numbers could not be trusted.
///
/// ´claim:dossier:a-default-score-set-is-zero-on-every-axis´
/// ´test:crate:axis-score-set-all-zeroes´
#[test]
fn axis_score_set_all_zeroes() {
    let set = AxisScoreSet::default();

    // Check all axes start at zero
    assert_eq!(set.novelty.max_z, 0.0);
    assert_eq!(set.novelty.mean_z, 0.0);
    assert_eq!(set.novelty.cusum, 0.0);
    assert_eq!(set.novelty.baseline_mean, 0.0);
    assert_eq!(set.novelty.baseline_variance, 0.0);
    assert_eq!(set.novelty.clip_pressure, 0.0);

    assert_eq!(set.displacement.max_z, 0.0);
    assert_eq!(set.surprise.max_z, 0.0);
    assert_eq!(set.coherence.max_z, 0.0);
}

/// The four axes are stored side by side and never share storage: writing
/// novelty's peak and accumulator leaves the other three untouched, and writing
/// displacement's mean leaves novelty's own mean at zero. This separation is what
/// makes per-axis sanitisation meaningful — zeroing one axis can only be a
/// containment measure if the axes were independent to begin with.
///
/// ´claim:dossier:the-four-axes-are-stored-separately-so-writing-one-never-disturbs-another´
/// ´test:crate:axis-score-set-independent´
#[test]
fn axis_score_set_independent() {
    // Setting one axis should not affect others
    let mut set = AxisScoreSet::default();

    set.novelty.max_z = 1.5;
    set.novelty.cusum = 2.0;

    // Other axes should remain zero
    assert_eq!(set.displacement.max_z, 0.0, "displacement should be independent");
    assert_eq!(set.surprise.max_z, 0.0, "surprise should be independent");
    assert_eq!(set.coherence.max_z, 0.0, "coherence should be independent");

    // Set another axis
    set.displacement.mean_z = 3.0;
    assert_eq!(set.novelty.mean_z, 0.0, "novelty.mean_z should be independent");
}

// ═══════════════════════════════════════════════════════════════════════════════
// SentinelSlot Tests (completion)
// ═══════════════════════════════════════════════════════════════════════════════

/// Storing an index into a slot replaces what was there entirely: a slot that
/// read as empty then yields the new report, with its one cell present and
/// routable. The unit of publication is a whole index, so no reader can catch a
/// slot mid-update holding some cells from one batch and some from the next.
///
/// ´claim:dossier:a-slot-publishes-whole-indexes-so-a-reader-never-sees-a-half-written-one´
/// ´test:crate:slot-report-swap´
#[test]
fn slot_report_swap() {
    use std::sync::Arc;

    let slot = SentinelSlot::new();

    // Initially empty
    let report1 = slot.report_index.load();
    assert!(!report1.has_report(), "initial report should be empty");

    // Store a real report
    let mut cells = HashMap::new();
    cells.insert(
        LedgerKey::new(0, 0),
        ReportCellEntry {
            depth: 0,
            sample_count: 42,
            ..Default::default()
        },
    );
    let new_report = ReportIndex::from_components(cells, Vec::new(), ReportLevelData::default(), 0);
    slot.report_index.store(Arc::new(new_report));

    // Load again - should see the new report
    let report2 = slot.report_index.load();
    assert!(report2.has_report(), "swapped report should have report");
    assert_eq!(report2.cells().len(), 1);
    let entry = report2.route(0).expect("root should exist");
    assert_eq!(entry.sample_count, 42);
}

/// The ingestion lock is released when its guard is dropped and can be taken
/// again straight afterwards. Every batch for a Sentinel passes through this one
/// lock, so a guard that outlived its ingestion — or a lock that stayed held after
/// a failed one — would stop that Sentinel from ever being heard from again.
///
/// ´claim:dossier:the-ingestion-lock-is-released-on-drop-and-can-be-taken-again´
/// ´test:crate:slot-ingestion-lock´
#[test]
fn slot_ingestion_lock() {
    let slot = SentinelSlot::new();

    // Acquire lock
    let guard = slot.lock_ingestion();

    // Try to check lock is held (we can't actually try_lock from same thread,
    // but we can verify the mutex is functional)
    drop(guard);

    // Lock can be reacquired
    let _guard2 = slot.lock_ingestion();
}
