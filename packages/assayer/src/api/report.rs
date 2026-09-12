// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Public `receive_sentinel_report()` method on `Assayer` and `ReportAck`.
//!
//! Delegates to the existing 6-step ingestion pipeline in
//! `crate::report::ingestion`.
//!
//! # Cross-References
//!
//! - Reception touches no model, graph, cache, or calibration state
//!   (´dec:surface:reception-isolation´)
//! - Reception is synchronous and serialised per Sentinel
//!   (´dec:ordering:synchronous-reception´)
//! - Reception acknowledges with maintenance diagnostics, so a caller learns
//!   what its report cost (´dec:surface:diagnostic-ack´)

use torrust_sentinel::BatchReport;

use crate::Assayer;
use crate::error::ReportError;
use crate::report::ingest_report;
use crate::types::SentinelId;

// ═══════════════════════════════════════════════════════════════════════════════
// ReportAck
// ═══════════════════════════════════════════════════════════════════════════════

/// Acknowledgement returned by `Assayer::receive_sentinel_report()`.
///
/// Provides diagnostic counters about the ingested report.
///
/// TODO ´todo:code:keep-this-host-facing-ack´: Keep this host-facing ack
/// distinct from `crate::report::ReportAck`. Research before either
/// type changes: whether the internal type should be renamed or aliased
/// as `InternalReportAck`, whether the mapper should be a named tested
/// function, and whether health needs its own public health type rather
/// than reusing either acknowledgement type. Reception acknowledges with
/// maintenance diagnostics (´dec:surface:diagnostic-ack´), and reception
/// itself is serialised per Sentinel (´dec:ordering:synchronous-reception´).
#[derive(Clone, Debug)]
#[non_exhaustive]
pub struct ReportAck {
    /// Number of cells in the ingested report.
    pub cells_in_report: usize,
    /// Number of new cells created in the Ledger.
    pub cells_created_in_ledger: usize,
    /// Number of cells deleted from the Ledger.
    pub cells_deleted_from_ledger: usize,
    /// Number of cells with sanitised (degraded) values.
    pub degraded_cells: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assayer::receive_sentinel_report()
// ═══════════════════════════════════════════════════════════════════════════════

impl Assayer {
    /// Ingests a Sentinel batch report.
    ///
    /// Delegates to the 6-step ingestion pipeline:
    ///
    /// 1. Slot lookup — find the Sentinel's slot
    /// 2. Lock acquisition — per-Sentinel ingestion mutex
    /// 3. Structural validation — root cell, dyadic intervals
    /// 4. Index construction — build `ReportIndex`
    /// 5. Cell-set maintenance — update Ledger
    /// 6. Publication — store new index and internal acknowledgement
    ///
    /// Concurrent reports for different Sentinels proceed without
    /// deadlock (independent per-Sentinel locks).
    ///
    /// # Errors
    ///
    /// - [`ReportError::UnknownSentinel`] — Sentinel ID not registered
    /// - [`ReportError::MissingRootCell`] — no depth-0 cell in report
    /// - [`ReportError::DuplicateCell`] — two cells share the same `(lo, depth)` key
    /// - [`ReportError::NonDyadicInterval`] — non-dyadic cell interval
    ///
    /// # Cross-References
    ///
    /// - Information crosses the boundary in one direction (´dec:ownership:feed-forward´)
    /// - Reception is synchronous and serialised per Sentinel (´dec:ordering:synchronous-reception´)
    // Justified: an owned `BatchReport` is what enforces the feed-forward
    // boundary (´inv:guarantee:feed-forward´).
    #[allow(clippy::needless_pass_by_value)]
    pub fn receive_sentinel_report(&self, sentinel_id: SentinelId, report: BatchReport<u128>) -> Result<ReportAck, ReportError> {
        let internal_ack = ingest_report(
            sentinel_id,
            &report,
            &self.sentinel_slots,
            &self.outcome_ledger,
            &self.ingestion,
            &*self.clock,
        )?;

        // TODO ´todo:code:move-this-to-a-named-mapper´: move this to a named mapper
        // before either ack grows again. Research the field contract at
        // the same time: `coordination_contexts` is intentionally internal
        // unless the acknowledgement contract promotes it
        // (´dec:surface:diagnostic-ack´), and public field names should
        // remain host-facing rather than ingestion-pipeline names.
        Ok(ReportAck {
            cells_in_report: internal_ack.cells_ingested,
            cells_created_in_ledger: internal_ack.cells_created,
            cells_deleted_from_ledger: internal_ack.cells_deleted,
            degraded_cells: internal_ack.degraded_cells,
        })
    }
}
