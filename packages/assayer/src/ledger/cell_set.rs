// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`appearances_create_neutral_entries`] | ledger | Cells named by a report that the ledger has never seen are taken up as neutral entries, and the summary distinguishes how many appeared from how many are now tracked. A Sentinel's reported geometry is what decides which cells exist, so the ledger follows the report rather than requiring cells to be declared ahead of the traffic that occupies them. |
//! | [`absence_increments_counter`] | ledger | A tracked cell that the current report omits accrues one absence, and successive omissions accumulate. Absence is counted in report cycles rather than acted on immediately, which is what lets a cell that is merely quiet for a cycle keep its history instead of being dropped by a single unlucky reporting window. |
//! | [`deletion_after_threshold`] | ledger | A cell survives absences up to the threshold and is removed on the one that reaches it — with a threshold of three, two omissions leave the entry in place carrying a count of two, and the third takes it away. The threshold is the ledger's patience budget: it bounds how long state for a vanished region is carried without discarding a region that is merely intermittent. |
//! | [`root_never_deleted`] | ledger | The root is exempt from the absence rule entirely: ten empty reports against a threshold of zero — the harshest setting available — leave it standing. Every read terminates at the root, so deleting it would leave the Sentinel with no routable destination at all; its permanence is what makes the rest of the cell set safe to prune aggressively. |
//! | [`presence_in_report_resets_counter`] | ledger | Reappearing in a report clears a cell's absence run outright rather than decrementing it: after two absences a single presence returns the count to zero, and the cell then survives two fresh absences before the third removes it. Deletion therefore requires an unbroken run, so a cell that reports intermittently is never worn down across widely separated quiet cycles. |
//! | [`deletion_no_merge`] | ledger | Removing a cell does not fold its history into its parent: the depth-8 child goes, and the depth-4 ancestor's tally is exactly what it was. The ancestor already received its own share of every write through the all-layers path, so merging on deletion would double-count the very observations it has already absorbed. |
//! | [`reset_absent_counters_works`] | ledger | Acknowledging presence on its own touches only the cells actually named: the listed cell's absence count returns to zero while an unlisted one keeps its five. This is the half of cell-set maintenance that neither creates nor deletes, so a caller can record that a partial report arrived without that partiality being read as evidence of absence. |

//! Cell-set maintenance for the Outcome Ledger.
//!
//! This module handles the lifecycle of ledger entries based on Sentinel
//! report data:
//!
//! - **Appearances** — New cells from the report are added as neutral entries
//! - **Absences** — Existing entries not in the report have `consecutive_absent` incremented
//! - **Deletions** — Entries absent for too many cycles are removed
//!
//! # Deletion Policy
//!
//! Entries are deleted (not merged) when absent for `n_absent_threshold` cycles.
//! The parent entry is unaffected. The root entry is never deleted.
//!
//! # Cross-References
//!
//! - (´alg:ledger:entry-creation´) — what an appearance installs
//! - (´alg:ledger:entry-deletion´) — what a deletion removes, and what it
//!   leaves the parent
//! - (´dec:memory:root-permanence´) — why the root is never among them
//! - (´dec:memory:periodic-sweep´) — why this runs as a bounded sweep rather
//!   than lazily on the access path

use std::collections::HashSet;

use super::entry::LedgerEntry;
use super::sentinel_ledger::SentinelLedger;
use crate::types::LedgerKey;

// ═══════════════════════════════════════════════════════════════════════════════
// Cell-Set Changes
// ═══════════════════════════════════════════════════════════════════════════════

/// Summary of cell-set maintenance changes.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct CellSetChanges {
    /// Number of new cells that appeared.
    pub appeared: usize,
    /// Number of cells that were deleted.
    pub deleted: usize,
    /// Total cells being tracked after changes.
    pub tracked: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell-Set Maintenance
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies cell-set changes from a Sentinel report.
///
/// Three-pass algorithm:
///
/// 1. **Appearances** — Insert neutral entries for cells in `current_report_cells`
///    that don't exist in the ledger.
///
/// 2. **Absences** — Increment `consecutive_absent` for non-root entries
///    that exist in the ledger but not in `current_report_cells`.
///
/// 3. **Deletions** — Remove non-root entries where
///    `consecutive_absent >= n_absent_threshold`.
///
/// # Arguments
///
/// * `ledger` — The Sentinel ledger to modify
/// * `current_report_cells` — Set of cell keys from the current report
/// * `n_absent_threshold` — Number of absent cycles before deletion
///
/// # Returns
///
/// A [`CellSetChanges`] summarising the modifications.
///
/// # Invariants
///
/// - The root entry (depth 0) is never deleted.
/// - Deletion produces no merge — parent entries are unaffected.
pub fn apply_cell_set_changes(
    ledger: &mut SentinelLedger,
    current_report_cells: &HashSet<LedgerKey>,
    n_absent_threshold: u8,
) -> CellSetChanges {
    let root_key = LedgerKey::new(0, 0);
    let mut appeared = 0;
    let mut deleted = 0;

    // Pass 1: Appearances and presence reset (´alg:ledger:entry-creation´)
    //
    // - New cells: insert neutral entries.
    // - Existing cells present in the report: reset consecutive_absent to 0.
    for &key in current_report_cells {
        if let Some(entry) = ledger.get_mut(&key) {
            entry.consecutive_absent = 0;
        } else {
            ledger.insert(key, LedgerEntry::new_neutral());
            appeared += 1;
        }
    }

    // Pass 2: Absences — increment consecutive_absent for missing cells
    // Collect keys first to avoid borrow issues
    let existing_keys: Vec<LedgerKey> = ledger.entries().keys().copied().collect();

    for key in &existing_keys {
        // Skip root
        if *key == root_key {
            continue;
        }

        if !current_report_cells.contains(key)
            && let Some(entry) = ledger.get_mut(key)
        {
            entry.consecutive_absent = entry.consecutive_absent.saturating_add(1);
        }
    }

    // Pass 3: Deletions — remove entries that have been absent too long
    let keys_to_delete: Vec<LedgerKey> = ledger
        .entries()
        .iter()
        .filter(|(key, entry)| {
            **key != root_key // Never delete root
                && entry.consecutive_absent >= n_absent_threshold
        })
        .map(|(key, _)| *key)
        .collect();

    let need_recalc = keys_to_delete.iter().any(|k| k.depth == ledger.max_depth());

    for key in keys_to_delete {
        ledger.remove(&key);
        deleted += 1;
    }

    // Recalculate max_depth if we deleted at the former maximum
    // (´alg:ledger:entry-deletion´).
    if need_recalc {
        ledger.recalculate_max_depth();
    }

    // Update entry count is handled by ledger.insert/remove
    let tracked = ledger.entry_count();

    CellSetChanges {
        appeared,
        deleted,
        tracked,
    }
}

/// Resets `consecutive_absent` for entries present in the report.
///
/// This is a lightweight alternative to [`apply_cell_set_changes`] when
/// you only need to acknowledge presence without creating new entries.
pub fn reset_absent_counters(ledger: &mut SentinelLedger, present_cells: &HashSet<LedgerKey>) {
    for key in present_cells {
        if let Some(entry) = ledger.get_mut(key) {
            entry.consecutive_absent = 0;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_key(lo: u128, depth: u8) -> LedgerKey {
        LedgerKey::new(lo, depth)
    }

    fn make_ledger_with_keys(keys: &[LedgerKey]) -> SentinelLedger {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        for &key in keys {
            if key.depth == 0 && key.lo == 0 {
                continue; // Root already exists
            }
            ledger.insert(key, LedgerEntry::new_neutral());
        }

        ledger
    }

    /// Cells named by a report that the ledger has never seen are taken up as
    /// neutral entries, and the summary distinguishes how many appeared from
    /// how many are now tracked. A Sentinel's reported geometry is what decides
    /// which cells exist, so the ledger follows the report rather than
    /// requiring cells to be declared ahead of the traffic that occupies them.
    ///
    /// ´claim:ledger:a-cell-newly-named-by-a-report-starts-being-tracked-from-neutral´
    /// ´test:unit:appearances-create-neutral-entries´
    #[test]
    fn appearances_create_neutral_entries() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        let new_cells: HashSet<LedgerKey> = vec![make_key(0, 4), make_key(0, 8), make_key(0, 12)].into_iter().collect();

        let changes = apply_cell_set_changes(&mut ledger, &new_cells, 3);

        assert_eq!(changes.appeared, 3);
        assert_eq!(changes.deleted, 0);
        assert_eq!(changes.tracked, 4); // root + 3 new
    }

    /// A tracked cell that the current report omits accrues one absence, and
    /// successive omissions accumulate. Absence is counted in report cycles
    /// rather than acted on immediately, which is what lets a cell that is
    /// merely quiet for a cycle keep its history instead of being dropped by a
    /// single unlucky reporting window.
    ///
    /// ´claim:ledger:a-cell-missing-from-a-report-accrues-a-consecutive-absence´
    /// ´test:unit:absence-increments-counter´
    #[test]
    fn absence_increments_counter() {
        let key4 = make_key(0, 4);
        let mut ledger = make_ledger_with_keys(&[key4]);

        // Report doesn't include key4
        let empty_report: HashSet<LedgerKey> = HashSet::new();
        apply_cell_set_changes(&mut ledger, &empty_report, 3);

        let entry = ledger.get(&key4).unwrap();
        assert_eq!(entry.consecutive_absent, 1);

        // Second absence
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        let entry = ledger.get(&key4).unwrap();
        assert_eq!(entry.consecutive_absent, 2);
    }

    /// A cell survives absences up to the threshold and is removed on the one
    /// that reaches it — with a threshold of three, two omissions leave the
    /// entry in place carrying a count of two, and the third takes it away. The
    /// threshold is the ledger's patience budget: it bounds how long state for
    /// a vanished region is carried without discarding a region that is merely
    /// intermittent.
    ///
    /// ´claim:ledger:a-cell-is-dropped-on-the-absence-that-reaches-the-threshold´
    /// ´test:unit:deletion-after-threshold´
    #[test]
    fn deletion_after_threshold() {
        let key4 = make_key(0, 4);
        let mut ledger = make_ledger_with_keys(&[key4]);

        let empty_report: HashSet<LedgerKey> = HashSet::new();

        // Absent for 2 cycles (threshold is 3)
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        assert!(ledger.get(&key4).is_some());
        assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 2);

        // Third absence triggers deletion
        let changes = apply_cell_set_changes(&mut ledger, &empty_report, 3);
        assert_eq!(changes.deleted, 1);
        assert!(ledger.get(&key4).is_none());
    }

    /// The root is exempt from the absence rule entirely: ten empty reports
    /// against a threshold of zero — the harshest setting available — leave it
    /// standing. Every read terminates at the root, so deleting it would leave
    /// the Sentinel with no routable destination at all; its permanence is what
    /// makes the rest of the cell set safe to prune aggressively.
    ///
    /// ´claim:ledger:the-root-cell-is-never-deleted-however-long-it-is-absent´
    /// ´test:unit:root-never-deleted´
    #[test]
    fn root_never_deleted() {
        let mut ledger = SentinelLedger::new();
        ledger.ensure_root();

        let empty_report: HashSet<LedgerKey> = HashSet::new();

        // Even with threshold 0, root should never be deleted
        for _ in 0..10 {
            apply_cell_set_changes(&mut ledger, &empty_report, 0);
        }

        assert!(ledger.has_root());
    }

    /// Reappearing in a report clears a cell's absence run outright rather than
    /// decrementing it: after two absences a single presence returns the count
    /// to zero, and the cell then survives two fresh absences before the third
    /// removes it. Deletion therefore requires an unbroken run, so a cell that
    /// reports intermittently is never worn down across widely separated
    /// quiet cycles.
    ///
    /// ´claim:ledger:a-cell-that-reappears-has-its-absence-run-reset-to-zero´
    /// ´test:unit:presence-in-report-resets-counter´
    #[test]
    fn presence_in_report_resets_counter() {
        let key4 = make_key(0, 4);
        let mut ledger = make_ledger_with_keys(&[key4]);

        // Mark as absent twice
        let empty_report: HashSet<LedgerKey> = HashSet::new();
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 2);

        // Report includes this cell — counter must reset to 0
        // (´alg:ledger:entry-creation´).
        let report_with_key: HashSet<LedgerKey> = vec![key4].into_iter().collect();
        apply_cell_set_changes(&mut ledger, &report_with_key, 3);
        assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 0);

        // After reset, 3 fresh absences are needed before deletion.
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        assert!(ledger.get(&key4).is_some(), "should survive 2 absences after reset");

        apply_cell_set_changes(&mut ledger, &empty_report, 3);
        assert!(ledger.get(&key4).is_none(), "deleted after 3 fresh absences");
    }

    /// Removing a cell does not fold its history into its parent: the depth-8
    /// child goes, and the depth-4 ancestor's tally is exactly what it was.
    /// The ancestor already received its own share of every write through the
    /// all-layers path, so merging on deletion would double-count the very
    /// observations it has already absorbed.
    ///
    /// ´claim:ledger:deleting-a-cell-leaves-its-parent-untouched-rather-than-merging-into-it´
    /// ´test:unit:deletion-no-merge´
    #[test]
    fn deletion_no_merge() {
        // Create a parent (depth 4) and child (depth 8)
        let key4 = make_key(0, 4);
        let key8 = make_key(0, 8);
        let mut ledger = make_ledger_with_keys(&[key4, key8]);

        // Mark parent with some data
        if let Some(entry) = ledger.get_mut(&key4) {
            entry.total_assessments = 100;
        }

        // Keep key4 in report, only key8 is absent
        let report_with_key4: HashSet<LedgerKey> = vec![key4].into_iter().collect();
        for _ in 0..3 {
            apply_cell_set_changes(&mut ledger, &report_with_key4, 3);
        }

        // Depth-8 should be deleted (absent 3 times)
        assert!(ledger.get(&key8).is_none());

        // Depth-4 should be unchanged (was in report, no merge happened)
        assert_eq!(ledger.get(&key4).unwrap().total_assessments, 100);
    }

    /// Acknowledging presence on its own touches only the cells actually named:
    /// the listed cell's absence count returns to zero while an unlisted one
    /// keeps its five. This is the half of cell-set maintenance that neither
    /// creates nor deletes, so a caller can record that a partial report
    /// arrived without that partiality being read as evidence of absence.
    ///
    /// ´claim:ledger:acknowledging-presence-clears-the-absence-run-of-only-the-cells-named´
    /// ´test:unit:reset-absent-counters-works´
    #[test]
    fn reset_absent_counters_works() {
        let key4 = make_key(0, 4);
        let key8 = make_key(0, 8);
        let mut ledger = make_ledger_with_keys(&[key4, key8]);

        // Manually set absence counters
        ledger.get_mut(&key4).unwrap().consecutive_absent = 2;
        ledger.get_mut(&key8).unwrap().consecutive_absent = 5;

        // Reset only key4
        let present: HashSet<LedgerKey> = vec![key4].into_iter().collect();
        reset_absent_counters(&mut ledger, &present);

        assert_eq!(ledger.get(&key4).unwrap().consecutive_absent, 0);
        assert_eq!(ledger.get(&key8).unwrap().consecutive_absent, 5);
    }
}
