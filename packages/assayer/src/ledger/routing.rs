// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`read_route_root_only`] | ledger | With only the root installed, every coordinate — zero and the largest representable alike — resolves to the root's entry. Routing is total from the moment a Sentinel exists, so an assessment arriving before any finer cell has ever been reported still reads a real outcome history rather than failing or reading a neutral placeholder. |
//! | [`read_route_deepest_wins`] | ledger | When several nested cells contain a coordinate, the read returns the deepest of them rather than any ancestor. Depth is specificity: the narrowest cell that has been reported is the one whose history actually describes this neighbourhood, and letting a broad ancestor answer would dilute a locally-concentrated signal into the population average. |
//! | [`read_route_falls_to_shallower`] | ledger | A coordinate that lies outside the deep cells still resolves, to the deepest ancestor that does contain it: the test picks a coordinate whose depth-4 ancestor is the tracked cell but whose depth-8 ancestor is not, and the depth-4 entry answers. The hierarchy is sparse by design, so falling back through it is the normal case rather than an error path. |
//! | [`read_route_exact_boundary`] | ledger | A cell's own lower endpoint belongs to that cell: routing the coordinate equal to the cell's `lo` lands on the cell rather than slipping past it to an ancestor. Dyadic intervals tile the coordinate space, so an off-by-one at the seam would leave a coordinate systematically misrouted at every depth boundary. |
//! | [`update_all_layers_hits_all`] | ledger | A write is not routed like a read: instead of picking one cell, it updates every tracked cell whose interval contains the coordinate, and reports how many it touched. Each scale keeps its own honest count, so a broad ancestor accumulates the aggregate view of its region at the same time as the narrow cell accumulates the local one. |
//! | [`update_all_layers_partial`] | ledger | Cells that do not contain the coordinate are left entirely alone: a coordinate inside the depth-4 cell but outside the depth-8 one updates two entries and leaves the depth-8 tally at zero. Containment, not mere presence in the ledger, is what earns a cell a share of an observation. |
//! | [`keys_containing_returns_all_matching`] | ledger | The containing cells can be enumerated for a coordinate, and they come back deepest-first — depth 8, then 4, then the root. That ordering is the same walk the read and write paths take, so a diagnostic listing shows the ancestry in the order routing would consider it rather than in whatever order the underlying map happens to hold. |

//! Read and write routing for the Outcome Ledger.
//!
//! This module provides hierarchical routing functions:
//!
//! - [`read_route`] — Depth-walk to find the deepest matching entry (read path)
//! - [`update_all_layers`] — Update all entries containing a coordinate (write path)
//!
//! # Routing Model
//!
//! The ledger uses a depth-walk routing model. For reads, we find the
//! deepest entry containing the coordinate. For writes, we update *all*
//! entries containing the coordinate from deepest to root.
//!
//! # Cross-References
//!
//! - (´dec:memory:depth-walk´) — reads exit at the first hit; writes visit
//!   every containing layer
//! - (´alg:ledger:all-layers-update´) — the update each visited layer takes
//! - (´dec:memory:root-permanence´) — why the walk always ends somewhere

use super::entry::{LedgerEntry, LedgerUpdate};
use super::sentinel_ledger::SentinelLedger;
use crate::types::{LedgerKey, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// Read Routing
// ═══════════════════════════════════════════════════════════════════════════════

/// Finds the deepest entry containing the given coordinate.
///
/// Performs a depth-walk from `max_depth` down to 0, returning the first
/// (deepest) entry whose dyadic interval contains the coordinate.
///
/// # Arguments
///
/// * `ledger` — The Sentinel ledger to search
/// * `coord` — The 128-bit coordinate to route
///
/// # Returns
///
/// A reference to the deepest matching entry. Since the root entry
/// always exists (after [`SentinelLedger::ensure_root`]), this always
/// succeeds.
///
/// # Algorithm
///
/// ```text
/// for depth in max_depth..=0 (descending):
///     key = LedgerKey::from_coordinate(coord, depth)
///     if ledger.contains(key):
///         return ledger[key]
/// ```
///
/// # Panics
///
/// Panics if the root entry does not exist.
#[must_use]
pub fn read_route(ledger: &SentinelLedger, coord: u128) -> &LedgerEntry {
    let max_depth = ledger.max_depth();

    // Walk from deepest to root
    for depth in (0..=max_depth).rev() {
        let key = LedgerKey::from_coordinate(coord, depth);
        if let Some(entry) = ledger.get(&key) {
            return entry;
        }
    }

    // Root must exist
    ledger.root()
}

/// Finds the deepest entry and returns its key.
///
/// Like [`read_route`], but returns the key instead of the entry.
#[must_use]
pub fn read_route_key(ledger: &SentinelLedger, coord: u128) -> LedgerKey {
    let max_depth = ledger.max_depth();

    for depth in (0..=max_depth).rev() {
        let key = LedgerKey::from_coordinate(coord, depth);
        if ledger.get(&key).is_some() {
            return key;
        }
    }

    LedgerKey::new(0, 0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Write Routing
// ═══════════════════════════════════════════════════════════════════════════════

/// Updates all entries whose intervals contain the given coordinate.
///
/// Visits every entry from deepest to root, applying write-time decay
/// and EWMA updates to each. The root entry is always updated.
///
/// # Arguments
///
/// * `ledger` — The Sentinel ledger to update
/// * `coord` — The 128-bit coordinate
/// * `gamma_t` — Hourly decay rate
/// * `now` — Current timestamp for decay
/// * `update` — Update data (valence, action, etc.)
/// * `lambda_l` — EWMA smoothing factor (`λ_L`)
///
/// # Algorithm (´alg:ledger:all-layers-update´)
///
/// ```text
/// for depth in max_depth..=0 (descending):
///     key = LedgerKey::from_coordinate(coord, depth)
///     if ledger.contains(key):
///         entry.apply_write_decay_and_update(gamma_t, now, update, lambda_l)
/// ```
///
/// # Returns
///
/// The number of entries updated (always ≥ 1 if root exists).
pub fn update_all_layers(
    ledger: &mut SentinelLedger,
    coord: u128,
    gamma_t: f64,
    now: &PersistentTimestamp,
    update: &LedgerUpdate,
    lambda_l: f64,
) -> usize {
    let max_depth = ledger.max_depth();
    let mut updated_count = 0;

    // Walk from deepest to root
    for depth in (0..=max_depth).rev() {
        let key = LedgerKey::from_coordinate(coord, depth);
        if let Some(entry) = ledger.get_mut(&key) {
            entry.apply_write_decay_and_update(gamma_t, now, update, lambda_l);
            updated_count += 1;
        }
    }

    updated_count
}

/// Returns all keys whose intervals contain the given coordinate.
///
/// Useful for diagnostics and testing.
#[must_use]
pub fn keys_containing(ledger: &SentinelLedger, coord: u128) -> Vec<LedgerKey> {
    let max_depth = ledger.max_depth();
    let mut keys = Vec::new();

    for depth in (0..=max_depth).rev() {
        let key = LedgerKey::from_coordinate(coord, depth);
        if ledger.get(&key).is_some() {
            keys.push(key);
        }
    }

    keys
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Action, dyadic_ancestor_lo};

    fn make_ledger_with_depths(depths: &[u8]) -> SentinelLedger {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        for &depth in depths {
            if depth == 0 {
                continue; // Root already exists
            }
            // Create entry at lo=0 for this depth
            let key = LedgerKey::new(0, depth);
            ledger.insert(key, LedgerEntry::new_neutral());
        }

        ledger
    }

    /// With only the root installed, every coordinate — zero and the largest
    /// representable alike — resolves to the root's entry. Routing is total
    /// from the moment a Sentinel exists, so an assessment arriving before any
    /// finer cell has ever been reported still reads a real outcome history
    /// rather than failing or reading a neutral placeholder.
    ///
    /// ´claim:ledger:a-root-only-ledger-routes-every-coordinate-to-the-root´
    /// ´test:unit:read-route-root-only´
    #[test]
    fn read_route_root_only() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        // Any coordinate should route to root
        let entry = read_route(&ledger, 0);
        assert_eq!(ledger.root().total_assessments, entry.total_assessments);

        let entry = read_route(&ledger, u128::MAX);
        assert_eq!(ledger.root().total_assessments, entry.total_assessments);
    }

    /// When several nested cells contain a coordinate, the read returns the
    /// deepest of them rather than any ancestor. Depth is specificity: the
    /// narrowest cell that has been reported is the one whose history actually
    /// describes this neighbourhood, and letting a broad ancestor answer would
    /// dilute a locally-concentrated signal into the population average.
    ///
    /// ´claim:ledger:the-read-path-returns-the-deepest-cell-containing-the-coordinate´
    /// ´test:unit:read-route-deepest-wins´
    #[test]
    fn read_route_deepest_wins() {
        let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

        // Coordinate in depth-8 cell (lo=0) should return depth-8 entry
        // Mark depth-8 entry distinctively
        let key8 = LedgerKey::new(0, 8);
        if let Some(entry) = ledger.get_mut(&key8) {
            entry.total_assessments = 888;
        }

        let coord = 0x00AB_0000_0000_0000_0000_0000_0000_0000_u128;
        let entry = read_route(&ledger, coord);
        assert_eq!(entry.total_assessments, 888);
    }

    /// A coordinate that lies outside the deep cells still resolves, to the
    /// deepest ancestor that does contain it: the test picks a coordinate whose
    /// depth-4 ancestor is the tracked cell but whose depth-8 ancestor is not,
    /// and the depth-4 entry answers. The hierarchy is sparse by design, so
    /// falling back through it is the normal case rather than an error path.
    ///
    /// ´claim:ledger:a-coordinate-outside-the-deep-cells-falls-back-to-the-deepest-ancestor-containing-it´
    /// ´test:unit:read-route-falls-to-shallower´
    #[test]
    fn read_route_falls_to_shallower() {
        let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

        // Mark depth-4 entry
        let key4 = LedgerKey::new(0, 4);
        if let Some(entry) = ledger.get_mut(&key4) {
            entry.total_assessments = 444;
        }

        // Coordinate in depth-4 but NOT in depth-8 (depth-8 covers only lo=0..2^120)
        // A coord with top 4 bits = 0 but top 8 bits != 0 will match depth-4 but not depth-8
        let coord = 0x0F00_0000_0000_0000_0000_0000_0000_0000_u128;

        // Verify this coord maps to lo=0 at depth 4
        assert_eq!(dyadic_ancestor_lo(coord, 4), 0);
        // But does NOT map to lo=0 at depth 8
        assert_ne!(dyadic_ancestor_lo(coord, 8), 0);

        let entry = read_route(&ledger, coord);
        assert_eq!(entry.total_assessments, 444);
    }

    /// A cell's own lower endpoint belongs to that cell: routing the coordinate
    /// equal to the cell's `lo` lands on the cell rather than slipping past it
    /// to an ancestor. Dyadic intervals tile the coordinate space, so an
    /// off-by-one at the seam would leave a coordinate systematically misrouted
    /// at every depth boundary.
    ///
    /// ´claim:ledger:a-cells-lower-endpoint-routes-into-that-cell´
    /// ´test:unit:read-route-exact-boundary´
    #[test]
    fn read_route_exact_boundary() {
        let mut ledger = make_ledger_with_depths(&[0, 8]);

        // Mark depth-8 entry
        let key8 = LedgerKey::new(0, 8);
        if let Some(entry) = ledger.get_mut(&key8) {
            entry.total_assessments = 808;
        }

        // Exact boundary: coord = lo of depth-8 cell = 0
        let entry = read_route(&ledger, 0);
        assert_eq!(entry.total_assessments, 808);
    }

    /// A write is not routed like a read: instead of picking one cell, it
    /// updates every tracked cell whose interval contains the coordinate, and
    /// reports how many it touched. Each scale keeps its own honest count, so a
    /// broad ancestor accumulates the aggregate view of its region at the same
    /// time as the narrow cell accumulates the local one.
    ///
    /// ´claim:ledger:a-write-updates-every-tracked-cell-containing-the-coordinate´
    /// ´test:unit:update-all-layers-hits-all´
    #[test]
    fn update_all_layers_hits_all() {
        let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

        let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
        let now = PersistentTimestamp::now();

        // Coordinate that matches all three depths (lo=0)
        let count = update_all_layers(&mut ledger, 0, 0.999, &now, &update, 0.999);
        assert_eq!(count, 3);

        // All entries should have been updated
        assert_eq!(ledger.get(&LedgerKey::new(0, 0)).unwrap().total_assessments, 1);
        assert_eq!(ledger.get(&LedgerKey::new(0, 4)).unwrap().total_assessments, 1);
        assert_eq!(ledger.get(&LedgerKey::new(0, 8)).unwrap().total_assessments, 1);
    }

    /// Cells that do not contain the coordinate are left entirely alone: a
    /// coordinate inside the depth-4 cell but outside the depth-8 one updates
    /// two entries and leaves the depth-8 tally at zero. Containment, not mere
    /// presence in the ledger, is what earns a cell a share of an observation.
    ///
    /// ´claim:ledger:a-write-skips-cells-that-do-not-contain-the-coordinate´
    /// ´test:unit:update-all-layers-partial´
    #[test]
    fn update_all_layers_partial() {
        let mut ledger = make_ledger_with_depths(&[0, 4, 8]);

        let update = LedgerUpdate::new(true, 0.5, 1.0, Action::Allow, true);
        let now = PersistentTimestamp::now();

        // Coordinate that matches depth-0 and depth-4, but not depth-8
        let coord = 0x0F00_0000_0000_0000_0000_0000_0000_0000_u128;
        let count = update_all_layers(&mut ledger, coord, 0.999, &now, &update, 0.999);
        assert_eq!(count, 2);

        // Depth-8 entry should NOT have been updated
        assert_eq!(ledger.get(&LedgerKey::new(0, 8)).unwrap().total_assessments, 0);
    }

    /// The containing cells can be enumerated for a coordinate, and they come
    /// back deepest-first — depth 8, then 4, then the root. That ordering is
    /// the same walk the read and write paths take, so a diagnostic listing
    /// shows the ancestry in the order routing would consider it rather than in
    /// whatever order the underlying map happens to hold.
    ///
    /// ´claim:ledger:the-cells-containing-a-coordinate-are-enumerated-deepest-first´
    /// ´test:unit:keys-containing-returns-all-matching´
    #[test]
    fn keys_containing_returns_all_matching() {
        let ledger = make_ledger_with_depths(&[0, 4, 8]);

        let keys = keys_containing(&ledger, 0);
        assert_eq!(keys.len(), 3);

        // Should be in descending depth order
        assert_eq!(keys[0].depth, 8);
        assert_eq!(keys[1].depth, 4);
        assert_eq!(keys[2].depth, 0);
    }
}
