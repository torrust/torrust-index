// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`find_active_returns_containment_chain`] | identity | A coordinate does not belong to one competitive cell but to a chain of them — every nested cell that covers it, reported deepest first and the root last. Because dyadic intervals either nest or stay disjoint, this chain is exactly the sequence of ever-coarser populations the entity is a member of, ordered so a reader takes the most specific evidence first and falls back outwards. |
//! | [`find_active_partial_containment`] | identity | Membership in the index is not membership in the chain: a cell whose interval lies elsewhere in the domain is passed over, however deep it is, and only the cells that actually cover the coordinate come back. An entity is therefore attributed the history of the populations it belongs to and no others. |
//! | [`find_active_on_empty_set`] | identity | A dimension with no competitive cells matches nothing: a lookup returns an empty chain rather than failing. Every dimension passes through this state on the way up, before enough traffic has arrived to make any region worth tracking, so the assessment path has to treat it as an ordinary answer. |
//! | [`find_active_at_exact_boundary`] | identity | cites (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´) |
//! | [`find_active_outside_all_cells`] | identity | cites (´claim:identity:only-cells-whose-interval-covers-the-coordinate-are-returned´) |
//! | [`cell_ids_iterator`] | identity | The index enumerates every cell it was built from, so the published competitive set can be read back whole rather than only probed a coordinate at a time. That is what lets a caller check a routed cell against the set it came from instead of taking the routing on trust. |
//! | [`competitive_cell_id_hi_calculation`] | identity | A cell's extent is fixed entirely by its depth: the root spans the whole domain to its last coordinate, each further level halves the span, and at the finest depth a cell covers a single coordinate. A cell is therefore identified by where it starts and how deep it sits, with nothing else to keep consistent. |
//! | [`competitive_cell_id_contains`] | identity | A cell owns its interval inclusively at both ends and nothing beyond it: the first and last coordinates are inside, the one below and the one above are not, and an adjacent cell picks up exactly where this one stops. The domain is thus partitioned with no coordinate falling in two sibling cells and none falling in the gap between them. |
//! | [`index_len_and_is_empty`] | identity | The index's reported size and its emptiness agree with the cells it actually holds, and the two agree with each other. The maintenance loop publishes a new index on every competitive-set change, so a size that drifted from the contents would misreport how much of the dimension is under dedicated tracking. |
//! | [`index_carries_published_total_importance`] | identity | The graph total published beside a competitive set is retained exactly, including for an empty set, so assessment reads the scalar from the same snapshot as the cells rather than reconstructing it from checkpoint state. |
//! | [`find_active_disjoint`] | identity | cites (´claim:identity:only-cells-whose-interval-covers-the-coordinate-are-returned´) |
//! | [`find_active_boundary_hi`] | identity | cites (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´) |
//! | [`find_active_just_past`] | identity | cites (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´) |
//! | [`find_active_sorts_by_depth_desc`] | identity | The deepest-first order is imposed on the result, not inherited from how the cells arrived: cells supplied shallowest-first still come back deepest-first. The maintenance loop builds each index from an unordered set, so a reader could otherwise get the chain in a different order after every republication. |

//! Competitive cell identification and indexing.
//!
// Data structures only. A dedicated owner holds the graph these cells are read
// from, and the assessment path never mutates it (´dec:memory:graph-owner´).
#![allow(dead_code)]
//!
//! This module provides the types for identifying and indexing competitive
//! cells within an identity dimension:
//!
//! - [`CompetitiveCellId`] — Unique identifier for a competitive cell
//! - [`CompetitiveCellInfo`] — Cell ID with interval bounds
//! - [`CompetitiveSetIndex`] — Index for fast containment lookup
//!
//! # Cross-References
//!
//! - (´def:keyspace:competitive-set´) — which graph entries are competitive,
//!   and why the set nests into a chain rather than tiling
//! - (´def:keyspace:competitive-indicators´) — the one indicator each cell
//!   contributes, which is what this index is scanned for
//! - (´dec:memory:competitive-publication´) — the per-dimension swap that
//!   republishes an index

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use smallvec::SmallVec;

use crate::types::dyadic_ancestor_hi;

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Cell ID
// ═══════════════════════════════════════════════════════════════════════════════

/// Unique identifier for a competitive cell in an identity dimension.
///
/// Mirrors [`LedgerKey`](crate::LedgerKey) pattern but is a distinct type
/// for type safety. The cell covers the dyadic interval `[lo, hi)` where
/// `hi = lo + 2^(128 - depth)`.
///
/// # Invariants
///
/// - `depth <= 128` (128-bit domain)
/// - `lo` is aligned to the cell size at `depth`
#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompetitiveCellId {
    /// Lower bound of the dyadic interval (inclusive).
    pub lo: u128,
    /// Depth in the G-tree (0 = root, 128 = finest resolution).
    pub depth: u8,
}

impl CompetitiveCellId {
    /// Creates a new competitive cell ID.
    ///
    /// # Panics
    ///
    /// Debug panics if `depth > 128`.
    #[must_use]
    pub const fn new(lo: u128, depth: u8) -> Self {
        debug_assert!(depth <= 128, "depth must be <= 128");
        Self { lo, depth }
    }

    /// Returns the inclusive upper bound of this cell's interval.
    #[must_use]
    pub const fn hi(&self) -> u128 {
        dyadic_ancestor_hi(self.lo, self.depth)
    }

    /// Returns `true` if this cell contains the given coordinate.
    #[must_use]
    pub const fn contains(&self, coord: u128) -> bool {
        let hi = dyadic_ancestor_hi(self.lo, self.depth);
        coord >= self.lo && coord <= hi
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Cell Info
// ═══════════════════════════════════════════════════════════════════════════════

/// Competitive cell with precomputed interval bounds.
///
/// Stores `hi` explicitly to avoid recomputation during lookup.
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompetitiveCellInfo {
    /// The cell identifier.
    pub id: CompetitiveCellId,
    /// Inclusive upper bound of the dyadic interval.
    pub hi: u128,
}

impl CompetitiveCellInfo {
    /// Creates a new cell info from an ID.
    #[must_use]
    pub const fn from_id(id: CompetitiveCellId) -> Self {
        Self { id, hi: id.hi() }
    }

    /// Returns `true` if this cell's interval contains the coordinate.
    #[must_use]
    pub const fn contains(&self, coord: u128) -> bool {
        coord >= self.id.lo && coord <= self.hi
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Set Index
// ═══════════════════════════════════════════════════════════════════════════════

/// Index of competitive cells for fast containment lookup.
///
/// The competitive set is the set of cells in an identity dimension's G-V
/// graph that have sufficient importance to track entity-specific baselines.
///
/// # Lookup Complexity
///
/// `find_active()` performs a linear scan of 10–25 cells (typical competitive
/// set size), completing in approximately 250 ns.
///
/// # Dyadic Interval Property
///
/// Because cells are dyadic intervals, they either nest or are disjoint.
/// The result of `find_active()` is always a containment chain from deepest
/// to root.
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct CompetitiveSetIndex {
    /// Cells sorted by `(lo asc, depth desc)`.
    ///
    /// This ordering ensures that for any coordinate, we encounter deeper
    /// (finer) cells before shallower (coarser) ones when scanning.
    cells: Vec<CompetitiveCellInfo>,
    /// Total graph importance at the publication that produced these cells.
    total_importance: f64,
}

impl CompetitiveSetIndex {
    /// Creates an empty competitive set index.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            cells: Vec::new(),
            total_importance: 0.0,
        }
    }

    /// Creates a new index from a vec of cell IDs.
    ///
    /// Cells are sorted by `(lo asc, depth desc)`.
    #[must_use]
    pub fn from_cells(cell_ids: impl IntoIterator<Item = CompetitiveCellId>) -> Self {
        Self::from_cells_and_total(cell_ids, 0.0)
    }

    /// Creates a new index from cells and the graph total observed with them.
    ///
    /// Cells are sorted by `(lo asc, depth desc)`.
    #[must_use]
    pub fn from_cells_and_total(cell_ids: impl IntoIterator<Item = CompetitiveCellId>, total_importance: f64) -> Self {
        let mut cells: Vec<CompetitiveCellInfo> = cell_ids.into_iter().map(CompetitiveCellInfo::from_id).collect();

        // Sort by (lo ascending, depth descending) so deeper cells come first
        // when multiple cells share the same lo bound.
        cells.sort_by(|a, b| a.id.lo.cmp(&b.id.lo).then_with(|| b.id.depth.cmp(&a.id.depth)));

        Self { cells, total_importance }
    }

    /// Finds all competitive cells whose interval contains the coordinate.
    ///
    /// Returns a containment chain from deepest to shallowest. Because dyadic
    /// intervals either nest or are disjoint, the result is ordered by
    /// decreasing depth.
    ///
    /// # Performance
    ///
    /// Linear scan of typically 10–25 cells (~250 ns). Returns a `SmallVec`
    /// to avoid heap allocation for typical containment chains of ≤4 cells.
    #[must_use]
    pub fn find_active(&self, coord: u128) -> SmallVec<[CompetitiveCellId; 4]> {
        let mut result: SmallVec<[CompetitiveCellId; 4]> = SmallVec::new();

        for cell in &self.cells {
            if cell.contains(coord) {
                result.push(cell.id);
            }
        }

        // Sort by depth descending (deepest first)
        result.sort_by_key(|c| std::cmp::Reverse(c.depth));

        result
    }

    /// Returns an iterator over all cell IDs in the index.
    pub fn cell_ids(&self) -> impl Iterator<Item = &CompetitiveCellId> {
        self.cells.iter().map(|info| &info.id)
    }

    /// Returns the number of cells in the index.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.cells.len()
    }

    /// Returns `true` if the index is empty.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.cells.is_empty()
    }

    /// Returns the total graph importance published with this index.
    #[must_use]
    pub const fn total_importance(&self) -> f64 {
        self.total_importance
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Creates a cell at the given lo and depth.
    fn cell(lo: u128, depth: u8) -> CompetitiveCellId {
        CompetitiveCellId::new(lo, depth)
    }

    /// A coordinate does not belong to one competitive cell but to a chain of
    /// them — every nested cell that covers it, reported deepest first and the
    /// root last. Because dyadic intervals either nest or stay disjoint, this
    /// chain is exactly the sequence of ever-coarser populations the entity is
    /// a member of, ordered so a reader takes the most specific evidence first
    /// and falls back outwards.
    ///
    /// ´claim:identity:a-coordinates-active-cells-are-returned-as-a-containment-chain-deepest-first´
    /// ´test:unit:find-active-returns-containment-chain´
    #[test]
    fn find_active_returns_containment_chain() {
        // Create cells at depths 0, 8, 16 that all contain coordinate 0x100
        // Depth 0: [0, 2^128) — contains everything
        // Depth 8: [0, 2^120) — contains 0x100
        // Depth 16: [0, 2^112) — contains 0x100
        let index = CompetitiveSetIndex::from_cells([cell(0, 0), cell(0, 8), cell(0, 16)]);

        let coord = 0x100;
        let result = index.find_active(coord);

        // Should return all three in depth-descending order
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].depth, 16); // deepest first
        assert_eq!(result[1].depth, 8);
        assert_eq!(result[2].depth, 0); // root last
    }

    /// Membership in the index is not membership in the chain: a cell whose
    /// interval lies elsewhere in the domain is passed over, however deep it
    /// is, and only the cells that actually cover the coordinate come back. An
    /// entity is therefore attributed the history of the populations it belongs
    /// to and no others.
    ///
    /// ´claim:identity:only-cells-whose-interval-covers-the-coordinate-are-returned´
    /// ´test:unit:find-active-partial-containment´
    #[test]
    fn find_active_partial_containment() {
        // Create cells where only some contain the coordinate
        // Depth 0: [0, 2^128) — contains everything
        // Depth 8: [0, 2^120) — contains small coords only
        // Depth 16: [2^112, 2^113) — does NOT contain 0x100

        let index = CompetitiveSetIndex::from_cells([
            cell(0, 0),
            cell(0, 8),
            cell(1u128 << 112, 16), // This cell starts at 2^112, won't contain 0x100
        ]);

        let coord = 0x100;
        let result = index.find_active(coord);

        // Should return depths 0 and 8, but not 16
        assert_eq!(result.len(), 2);
        assert_eq!(result[0].depth, 8); // deepest that contains coord
        assert_eq!(result[1].depth, 0);
    }

    /// A dimension with no competitive cells matches nothing: a lookup returns
    /// an empty chain rather than failing. Every dimension passes through this
    /// state on the way up, before enough traffic has arrived to make any
    /// region worth tracking, so the assessment path has to treat it as an
    /// ordinary answer.
    ///
    /// ´claim:identity:an-empty-competitive-set-matches-no-coordinate´
    /// ´test:unit:find-active-on-empty-set´
    #[test]
    fn find_active_on_empty_set() {
        let index = CompetitiveSetIndex::empty();
        let result = index.find_active(0x12345);
        assert!(result.is_empty());
    }

    /// The coordinate sitting exactly on a cell's lower bound is inside it, not
    /// beside it. The first entity in a region must land in that region's cell,
    /// or the population's own edge would be the one place its evidence went
    /// missing.
    ///
    /// (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´)
    /// ´test:unit:find-active-at-exact-boundary´
    #[test]
    fn find_active_at_exact_boundary() {
        // Cell at depth 8 covering [0, 2^120)
        let index = CompetitiveSetIndex::from_cells([cell(0, 8)]);

        // Coordinate exactly at lo should be contained
        let result = index.find_active(0);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], cell(0, 8));
    }

    /// A coordinate at the far end of the domain from the only tracked region
    /// belongs to no cell, and the chain comes back empty. An entity encoded
    /// into untracked space simply has no identity-specific evidence, which is
    /// a different thing from inheriting a stranger's.
    ///
    /// (´claim:identity:only-cells-whose-interval-covers-the-coordinate-are-returned´)
    /// ´test:unit:find-active-outside-all-cells´
    #[test]
    fn find_active_outside_all_cells() {
        // Cell at depth 16 covering only a small range
        let small_cell = cell(0, 16); // [0, 2^112)
        let index = CompetitiveSetIndex::from_cells([small_cell]);

        // Coordinate well outside the cell
        let large_coord = u128::MAX;
        let result = index.find_active(large_coord);

        // Cell at depth 16 starting at 0 has hi = 2^112, so MAX is outside
        assert!(result.is_empty());
    }

    /// The index enumerates every cell it was built from, so the published
    /// competitive set can be read back whole rather than only probed a
    /// coordinate at a time. That is what lets a caller check a routed cell
    /// against the set it came from instead of taking the routing on trust.
    ///
    /// ´claim:identity:the-index-enumerates-every-cell-it-was-built-from´
    /// ´test:unit:cell-ids-iterator´
    #[test]
    fn cell_ids_iterator() {
        let cells = [cell(0, 0), cell(0, 8), cell(1u128 << 120, 8)];
        let index = CompetitiveSetIndex::from_cells(cells);

        assert_eq!(index.cell_ids().count(), 3);
    }

    /// A cell's extent is fixed entirely by its depth: the root spans the whole
    /// domain to its last coordinate, each further level halves the span, and at
    /// the finest depth a cell covers a single coordinate. A cell is therefore
    /// identified by where it starts and how deep it sits, with nothing else to
    /// keep consistent.
    ///
    /// ´claim:identity:a-cells-extent-is-fixed-by-its-depth-from-the-whole-domain-at-the-root-to-a-single-coordinate-at-the-finest´
    /// ´test:unit:competitive-cell-id-hi-calculation´
    #[test]
    fn competitive_cell_id_hi_calculation() {
        // Depth 0: interval is [0, u128::MAX] (inclusive)
        let root = cell(0, 0);
        assert_eq!(root.hi(), u128::MAX);

        // Depth 8: interval is [0, 2^120 - 1] (inclusive)
        let d8 = cell(0, 8);
        assert_eq!(d8.hi(), (1u128 << 120) - 1);

        // Depth 128 (finest): interval is [lo, lo] (single value)
        let finest = cell(42, 128);
        assert_eq!(finest.hi(), 42);
    }

    /// A cell owns its interval inclusively at both ends and nothing beyond it:
    /// the first and last coordinates are inside, the one below and the one
    /// above are not, and an adjacent cell picks up exactly where this one
    /// stops. The domain is thus partitioned with no coordinate falling in two
    /// sibling cells and none falling in the gap between them.
    ///
    /// ´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´
    /// ´test:unit:competitive-cell-id-contains´
    #[test]
    fn competitive_cell_id_contains() {
        // Depth 120: width = 2^8 = 256, so [0, 255] (inclusive)
        // Use lo=0 which is aligned to depth 120
        let c = cell(0, 120);
        assert!(c.contains(0)); // lo
        assert!(c.contains(128)); // middle
        assert!(c.contains(255)); // hi (inclusive)
        assert!(!c.contains(256)); // above hi

        // Test with a non-zero aligned lo
        // At depth 120, cells are aligned to 256
        let c2 = cell(256, 120); // [256, 511]
        assert!(c2.contains(256));
        assert!(c2.contains(400));
        assert!(c2.contains(511));
        assert!(!c2.contains(255)); // below lo
        assert!(!c2.contains(512)); // above hi
    }

    /// The index's reported size and its emptiness agree with the cells it
    /// actually holds, and the two agree with each other. The maintenance loop
    /// publishes a new index on every competitive-set change, so a size that
    /// drifted from the contents would misreport how much of the dimension is
    /// under dedicated tracking.
    ///
    /// ´claim:identity:the-indexs-reported-size-and-emptiness-agree-with-the-cells-it-holds´
    /// ´test:unit:index-len-and-is-empty´
    #[test]
    fn index_len_and_is_empty() {
        let empty = CompetitiveSetIndex::empty();
        assert!(empty.is_empty());
        assert_eq!(empty.len(), 0);

        let non_empty = CompetitiveSetIndex::from_cells([cell(0, 0)]);
        assert!(!non_empty.is_empty());
        assert_eq!(non_empty.len(), 1);
    }

    /// The graph total published beside a competitive set is retained exactly,
    /// including for an empty set, so assessment reads the scalar from the same
    /// snapshot as the cells rather than reconstructing it from checkpoint
    /// state.
    ///
    /// ´claim:identity:a-competitive-index-publishes-the-graph-total-beside-its-cells´
    /// ´test:unit:index-carries-published-total-importance´
    #[test]
    fn index_carries_published_total_importance() {
        let index = CompetitiveSetIndex::from_cells_and_total([], 1234.5);
        assert!(index.is_empty());
        assert!((index.total_importance() - 1234.5).abs() < f64::EPSILON);
    }

    /// Two cells at the same depth in different halves of the domain are told
    /// apart: a coordinate in one finds only that one, and a coordinate in the
    /// other finds only the other. Siblings are alternatives, never a chain, so
    /// one population's evidence never leaks into a peer's.
    ///
    /// (´claim:identity:only-cells-whose-interval-covers-the-coordinate-are-returned´)
    /// ´test:unit:find-active-disjoint´
    #[test]
    fn find_active_disjoint() {
        // Two disjoint depth-8 cells: A=[0, 2^120), C=[2^120, 2^121)
        let a_lo = 0u128;
        let c_lo = 1u128 << 120;
        let index = CompetitiveSetIndex::from_cells([cell(a_lo, 8), cell(c_lo, 8)]);

        // Coord in A should find only A
        let result = index.find_active(42);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], cell(a_lo, 8));

        // Coord in C should find only C
        let result = index.find_active(c_lo + 500);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], cell(c_lo, 8));
    }

    /// The last coordinate a cell covers is still inside it. The upper bound is
    /// inclusive, so the topmost entity in a region is tracked by that region
    /// rather than falling into the gap between it and its neighbour.
    ///
    /// (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´)
    /// ´test:unit:find-active-boundary-hi´
    #[test]
    fn find_active_boundary_hi() {
        // Cell at depth 8: [0, 2^120 - 1] (inclusive)
        let index = CompetitiveSetIndex::from_cells([cell(0, 8)]);
        let hi = (1u128 << 120) - 1;

        // Last coord in interval should be contained
        let result = index.find_active(hi);
        assert_eq!(result.len(), 1);
        assert_eq!(result[0], cell(0, 8));
    }

    /// The very next coordinate past a cell's upper bound belongs to the
    /// neighbouring cell, not this one. Cells stop exactly where the next
    /// begins, so an off-by-one at the boundary cannot hand one population's
    /// first entity to the population below it.
    ///
    /// (´claim:identity:a-cells-dyadic-interval-is-inclusive-at-both-ends-and-excludes-its-neighbours´)
    /// ´test:unit:find-active-just-past´
    #[test]
    fn find_active_just_past() {
        // Cell at depth 8: [0, 2^120 - 1] (inclusive)
        let index = CompetitiveSetIndex::from_cells([cell(0, 8)]);
        let just_past = 1u128 << 120; // first coord past hi

        let result = index.find_active(just_past);
        assert!(result.is_empty());
    }

    /// The deepest-first order is imposed on the result, not inherited from how
    /// the cells arrived: cells supplied shallowest-first still come back
    /// deepest-first. The maintenance loop builds each index from an unordered
    /// set, so a reader could otherwise get the chain in a different order after
    /// every republication.
    ///
    /// ´claim:identity:the-deepest-first-order-does-not-depend-on-the-order-cells-were-supplied´
    /// ´test:unit:find-active-sorts-by-depth-desc´
    #[test]
    fn find_active_sorts_by_depth_desc() {
        // Supply cells in ascending depth order (0, 4, 8)
        // All containing the same coordinate
        let a_lo = 0u128;
        let index = CompetitiveSetIndex::from_cells([cell(a_lo, 0), cell(a_lo, 4), cell(a_lo, 8)]);

        let result = index.find_active(42);
        assert_eq!(result.len(), 3);
        assert_eq!(result[0].depth, 8); // deepest first
        assert_eq!(result[1].depth, 4);
        assert_eq!(result[2].depth, 0);
    }
}
