// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`make_ledger_key_normal`] | dossier | A Ledger key is nothing more than the pair a cell already reports: its start coordinate and its depth, carried through unchanged. Keying on the reported interval rather than on a hash of the cell's contents is what lets the same physical region be recognised across successive reports. |
//! | [`make_ledger_key_overflow`] | dossier | A depth beyond 128 cannot occur in a well-formed report, so it is data-quality input rather than data: the key construction clamps it to 128, in every build profile alike. Clamping matters because the narrowing to `u8` would otherwise wrap a depth of 200 round to 72 and silently file the cell against a real but wrong region of the space — and the cell extraction reports the same clamp through its degraded flag, so the sanitisation is counted rather than silent. |
//! | [`ingestion_config_default`] | dossier | Out of the box a cell must go missing from three consecutive reports before its Ledger entry is discarded. A threshold above one is what makes the cell set tolerant of a single report in which a region happened not to be competitive, rather than forgetting its history at the first gap. |

//! Sentinel report ingestion pipeline.
//!
//! This module implements the 6-step ingestion pipeline for `BatchReport<u128>`:
//!
//! 1. Slot lookup via `DashMap::get()` → `UnknownSentinel` if absent
//! 2. Acquire per-Sentinel ingestion `Mutex<()>`
//! 3. `validate_structural(&report)` → hard error on failure
//! 4. Build `ReportIndex` from cell and ancestor reports
//! 5. Cell-set maintenance — update Ledger with appearing/disappearing cells
//! 6. `ArcSwap::store` — publish new `ReportIndex` and internal `ReportAck`
//!
//! # Cross-References
//!
//! - (´dec:ownership:feed-forward´) — the one direction information crosses
//!   the value-transfer boundary
//! - (´dec:ordering:synchronous-reception´) — reception is synchronous and
//!   serialised per Sentinel
//! - (´dec:surface:diagnostic-ack´) — what the acknowledgement this pipeline
//!   retains carries
//! - (´dec:surface:reception-isolation´) — the steps this pipeline runs, and
//!   the state it does not touch

// Items in this module are re-exported via crate::report and used by tests.

use std::collections::{HashMap, HashSet};
use std::sync::Arc;
use std::time::Instant;

use dashmap::DashMap;
use torrust_sentinel::BatchReport;

use super::index::{CoordinationEntry, ReportAck, ReportCellEntry, ReportIndex, ReportLevelData};
use super::slot::SentinelSlot;
use super::validation::{depth_to_u8_clamped, validate_and_extract_cell, validate_and_extract_coordination, validate_structural};
use crate::error::ReportError;
use crate::ledger::{OutcomeLedger, apply_cell_set_changes};
use crate::types::{LedgerKey, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Ingestion Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for report ingestion and Ledger maintenance.
///
/// Part of the Ledger configuration hierarchy: one of the parameters the
/// surfaces defer to rather than fix (´tab:construction:parameters´).
#[derive(Clone, Debug)]
pub struct IngestionConfig {
    /// Number of consecutive absent cycles before a Ledger entry is deleted.
    ///
    /// Default: 3. Root entry is never deleted regardless of this setting.
    pub n_absent_threshold: u8,
}

impl Default for IngestionConfig {
    fn default() -> Self {
        Self { n_absent_threshold: 3 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Ingestion Pipeline
// ═══════════════════════════════════════════════════════════════════════════════

/// Ingests a Sentinel batch report into the system.
///
/// This is the main entry point for report processing. It implements a 6-step
/// pipeline:
///
/// 1. **Slot lookup** — Find the Sentinel's slot in the `DashMap`
/// 2. **Lock acquisition** — Acquire the per-Sentinel ingestion mutex
/// 3. **Structural validation** — Validate the report structure
/// 4. **Index construction** — Build a `ReportIndex` from the report
/// 5. **Cell-set maintenance** — Update the Ledger with cell changes
/// 6. **Publication** — Store new index and internal `ReportAck`
///
/// # Arguments
///
/// * `sentinel_id` — The Sentinel that produced this report
/// * `report` — The batch report to ingest
/// * `sentinel_slots` — Map of Sentinel IDs to their slots
/// * `outcome_ledger` — The outcome ledger for cell-set maintenance
/// * `config` — Ingestion configuration
///
/// # Returns
///
/// A `ReportAck` on success, containing:
/// - `cells_ingested` — Total number of cells processed
/// - `degraded_cells` — Number of cells with sanitised values
/// - `coordination_contexts` — Number of coordination entries
///
/// # Errors
///
/// - [`ReportError::UnknownSentinel`] — Sentinel ID not found in slot map
/// - [`ReportError::MissingRootCell`] — Report has no depth-0 cell
/// - [`ReportError::NonDyadicInterval`] — A cell interval is not dyadic
/// - [`ReportError::DuplicateCell`] — Two cells share the same `(lo, depth)` key
///
/// # Panics
///
/// Panics if the ingestion mutex or Ledger `RwLock` is poisoned.
///
/// # Concurrency
///
/// The ingestion mutex ensures that concurrent reports for the same Sentinel
/// are serialised. The Ledger `RwLock` is held only for the cell-set maintenance
/// step. The `ArcSwap` update is lock-free.
///
/// **Lock ordering:** ingestion Mutex → Ledger `RwLock` (never reversed).
pub fn ingest_report(
    sentinel_id: SentinelId,
    report: &BatchReport<u128>,
    sentinel_slots: &DashMap<SentinelId, SentinelSlot>,
    outcome_ledger: &OutcomeLedger,
    config: &IngestionConfig,
    clock: &dyn crate::testing::Clock,
) -> Result<ReportAck, ReportError> {
    // Stamp report arrival time at the API boundary. Threaded through index
    // construction so `ReportIndex.received_at` reflects arrival, not
    // materialisation.
    //
    // This stamp is one half of report staleness
    // (´def:extraction:report-staleness´); the assessment's batch reading is
    // the other. Both come from the same injected clock, so their difference
    // is a property of the scenario rather than of how busy the machine was.
    let received_at = clock.now_monotonic();

    // ─── Step 1: Slot lookup ────────────────────────────────────────────────
    let slot_ref = sentinel_slots
        .get(&sentinel_id)
        .ok_or(ReportError::UnknownSentinel { id: sentinel_id })?;

    // ─── Step 2: Acquire ingestion lock ─────────────────────────────────────
    // This serialises concurrent reports for the same Sentinel.
    // Poison recovery is safe: the guarded value is () and the ArcSwap
    // swap (final step) has not executed, so readers still see the
    // previous valid index; reports from one Sentinel cannot interleave
    // (´dec:ordering:synchronous-reception´).
    let ingestion_guard = slot_ref
        .ingestion_lock
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);

    // ─── Step 3: Structural validation ──────────────────────────────────────
    validate_structural(report)?;

    // ─── Step 4: Build ReportIndex ──────────────────────────────────────────
    let (index, mut ack) = build_report_index(report, received_at);

    // ─── Step 5: Cell-set maintenance ───────────────────────────────────────
    // Compute the set of cell keys from the new index
    let current_cells: HashSet<LedgerKey> = index.cells().keys().copied().collect();

    // Acquire Ledger write lock and apply cell-set changes
    if let Some(ledger_lock) = outcome_ledger.get(sentinel_id) {
        let mut ledger = ledger_lock.write().expect("ledger RwLock poisoned");
        let changes = apply_cell_set_changes(&mut ledger, &current_cells, config.n_absent_threshold);
        ack.cells_created = changes.appeared;
        ack.cells_deleted = changes.deleted;
        slot_ref.accumulate_ledger_cell_deltas(changes.appeared, changes.deleted);
        drop(ledger);
        // Lock released here
    }

    // ─── Step 6: Publication ────────────────────────────────────────────────
    slot_ref.report_index.store(Arc::new(index));
    // TODO ´todo:code:research-whether´: Research whether
    // health needs a generation tag to pair this internal ack with the
    // independently stored report index. Keep that pairing internal;
    // the public ReportAck remains the per-call host acknowledgement
    // (´dec:surface:diagnostic-ack´) and should continue to be produced only
    // by the API-layer mapper.
    slot_ref.last_report_ack.store(Arc::new(ack.clone()));

    // Drop the ingestion guard first (since it borrows slot_ref), then slot_ref
    drop(ingestion_guard);
    drop(slot_ref);

    Ok(ack)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Index Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Builds a `ReportIndex` from a validated batch report.
///
/// Iterates through `cell_reports` and `ancestor_reports`, extracting and
/// validating each cell. Constructs the `ReportLevelData` from the contour
/// and analysis set summary.
///
/// # Arguments
///
/// * `report` — The validated batch report
/// * `received_at` — Timestamp stamped at the ingestion API boundary, from
///   which staleness is measured (´def:extraction:report-staleness´); threaded
///   through from [`ingest_report`].
///
/// # Returns
///
/// A tuple containing the constructed `ReportIndex` and a `ReportAck`
/// with ingestion statistics.
fn build_report_index(report: &BatchReport<u128>, received_at: Instant) -> (ReportIndex, ReportAck) {
    let mut cells: HashMap<LedgerKey, ReportCellEntry> = HashMap::new();
    let mut coordination: Vec<CoordinationEntry> = Vec::new();
    let mut max_depth: u8 = 0;
    let mut degraded_cells = 0;

    // Process competitive cells
    for cell in &report.cell_reports {
        let key = make_ledger_key(cell.start, cell.depth);
        let (entry, degraded) = validate_and_extract_cell(cell);

        if degraded {
            degraded_cells += 1;
        }

        max_depth = max_depth.max(entry.depth);
        cells.insert(key, entry);
    }

    // Process ancestor cells
    for cell in &report.ancestor_reports {
        let key = make_ledger_key(cell.start, cell.depth);
        let (entry, degraded) = validate_and_extract_cell(cell);

        if degraded {
            degraded_cells += 1;
        }

        max_depth = max_depth.max(entry.depth);
        cells.insert(key, entry);
    }

    // Process coordination entries
    //
    // Coordination-level degradation is not counted towards `degraded_cells`,
    // which tracks the cell entries the acknowledgement reports
    // (´dec:surface:diagnostic-ack´).
    for coord in &report.coordination_reports {
        let (entry, _degraded) = validate_and_extract_coordination(coord);
        coordination.push(entry);
    }

    // Build level data from contour and analysis set summary
    let level_data = ReportLevelData {
        competitive_cell_count: report.analysis_set_summary.competitive_size,
        full_set_size: report.analysis_set_summary.full_size,
        depth_range: (
            depth_to_u8_clamped(report.analysis_set_summary.depth_range.0),
            depth_to_u8_clamped(report.analysis_set_summary.depth_range.1),
        ),
        plateau_count: report.contour.plateau_count,
        total_importance: report.contour.total_importance,
        contour_cell_count: report.contour.cell_count,
        splits_since_last_report: report.contour.splits_since_last_report,
        net_removals_since_last_report: report.contour.net_removals_since_last_report,
        importance_range: report.analysis_set_summary.importance_range,
        v_depth_range: report.analysis_set_summary.v_depth_range,
        degenerate_cells_skipped: report.analysis_set_summary.degenerate_cells_skipped,
    };

    let cells_ingested = cells.len();
    let coordination_contexts = coordination.len();

    // The age travels with the arrival stamp. The Sentinel measured it on its
    // own monotonic clock and this pipeline neither adjusts nor re-bases it:
    // the two figures belong to different clocks and are kept apart, which is
    // exactly why the stage they bound is reported as a lower bound
    // (´def:monitoring:feedback-latency´).
    let index = ReportIndex::from_components_at(
        cells,
        coordination,
        level_data,
        max_depth,
        received_at,
        report.oldest_observation_age_micros,
    );

    let ack = ReportAck {
        cells_ingested,
        degraded_cells,
        coordination_contexts,
        cells_created: 0,
        cells_deleted: 0,
    };

    (index, ack)
}

/// Creates a `LedgerKey` from a cell's start coordinate and depth.
///
/// Delegates to [`depth_to_u8_clamped`] for depth overflow handling.
#[must_use]
const fn make_ledger_key(start: u128, depth: u32) -> LedgerKey {
    LedgerKey::new(start, depth_to_u8_clamped(depth))
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A Ledger key is nothing more than the pair a cell already reports: its
    /// start coordinate and its depth, carried through unchanged. Keying on the
    /// reported interval rather than on a hash of the cell's contents is what
    /// lets the same physical region be recognised across successive reports.
    ///
    /// ´claim:dossier:a-ledger-key-carries-a-cells-start-coordinate-and-depth-unchanged´
    /// ´test:unit:make-ledger-key-normal´
    #[test]
    fn make_ledger_key_normal() {
        let key = make_ledger_key(0x1234, 8);
        assert_eq!(key.lo, 0x1234);
        assert_eq!(key.depth, 8);
    }

    /// A depth beyond 128 cannot occur in a well-formed report, so it is
    /// data-quality input rather than data: the key construction clamps it to
    /// 128, in every build profile alike. Clamping matters because the
    /// narrowing to `u8` would otherwise wrap a depth of 200 round to 72 and
    /// silently file the cell against a real but wrong region of the space —
    /// and the cell extraction reports the same clamp through its degraded
    /// flag, so the sanitisation is counted rather than silent.
    ///
    /// ´claim:dossier:a-depth-beyond-128-is-clamped-rather-than-wrapped-in-every-build-profile´
    /// ´test:unit:make-ledger-key-overflow´
    #[test]
    fn make_ledger_key_overflow() {
        let key = make_ledger_key(0x1234, 200);
        assert_eq!(key.lo, 0x1234);
        assert_eq!(key.depth, 128);
    }

    /// Out of the box a cell must go missing from three consecutive reports
    /// before its Ledger entry is discarded. A threshold above one is what makes
    /// the cell set tolerant of a single report in which a region happened not to
    /// be competitive, rather than forgetting its history at the first gap.
    ///
    /// ´claim:dossier:by-default-a-cell-must-be-absent-three-times-before-its-ledger-entry-goes´
    /// ´test:unit:ingestion-config-default´
    #[test]
    fn ingestion_config_default() {
        let config = IngestionConfig::default();
        assert_eq!(config.n_absent_threshold, 3);
    }
}
