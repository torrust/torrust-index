// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sentinel slot management.
//!
//! Each Sentinel is assigned a `SentinelSlot` that holds:
//! - The current `ReportIndex` (stored atomically via `ArcSwap`)
//! - The most recent internal `ReportAck` (stored atomically via `ArcSwap`)
//! - An ingestion lock (serialises multi-step ingestion pipeline)
//! - Bootstrap state (active flag and accumulator)
//!
//! # Thread Safety
//!
//! The `ReportIndex` is read via `ArcSwap::load()` (lock-free) and written
//! via the ingestion pipeline while holding `ingestion_lock`. This ensures
//! readers never block and writers are serialised.
//!
//! # Cross-References
//!
//! - (´def:extraction:slot´) — the canonical per-Sentinel slot: its width and
//!   what occupies it
//! - (´dec:vector:transient-accumulators´) — why the bootstrap accumulator is
//!   held here and never checkpointed
//! - (´dec:surface:diagnostic-ack´) — what the retained acknowledgement carries
//! - (´dec:surface:slot-map´) — the concurrent map these slots live in
//! - Built to the plan's `SentinelSlot` shape

use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};

use arc_swap::ArcSwap;

use super::index::{ReportAck, ReportIndex};
use crate::feature::bootstrap::BootstrapAccumulator;

// ═══════════════════════════════════════════════════════════════════════════════
// Sentinel Slot
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-Sentinel slot holding report state and bootstrap accumulator.
///
/// # Fields
///
/// - `report_index` — Current `ReportIndex`, stored atomically
/// - `last_report_ack` — Most recent internal ingestion acknowledgement
/// - `ingestion_lock` — Serialises the multi-step ingestion pipeline
/// - `bootstrap_active` — Whether bootstrap phase is in progress
/// - `bootstrap` — Optional bootstrap accumulator
// Layer 1 scaffolding; Layer 2 reaches it through the slot map (´dec:surface:slot-map´).
pub struct SentinelSlot {
    /// Current report index, stored atomically.
    pub report_index: ArcSwap<ReportIndex>,
    /// Most recent internal report acknowledgement, stored atomically.
    ///
    /// Updated by `ingest_report()` on every successful ingestion.
    /// Starts as the default (all zeros) until the first report.
    /// TODO ´todo:code:if-the-internal´: if the internal
    /// ack is renamed or wrapped after the internal/public mapping
    /// research, keep this field tied to the internal storage type, not
    /// the public `api::ReportAck` returned per call — the diagnostics
    /// reception answers with (´dec:surface:diagnostic-ack´).
    pub last_report_ack: ArcSwap<ReportAck>,
    /// Cells created by successful report ingestions over this process lifetime.
    cells_created: AtomicUsize,
    /// Cells deleted by successful report ingestions over this process lifetime.
    cells_deleted: AtomicUsize,
    /// Lock for serialising the ingestion pipeline.
    ///
    /// Only accessed by `crate::report::ingestion::ingest_report()`.
    pub(super) ingestion_lock: Mutex<()>,
    /// Whether bootstrap phase is active.
    pub bootstrap_active: AtomicBool,
    /// Bootstrap accumulator (if bootstrap is active).
    pub bootstrap: Mutex<Option<BootstrapAccumulator>>,
}

// Layer 1 scaffolding; Layer 2 and the tests reach it through reception
// (´dec:ordering:synchronous-reception´).
#[allow(dead_code)]
impl SentinelSlot {
    /// Creates a new `SentinelSlot` with an empty report index.
    ///
    /// Bootstrap is inactive until explicitly started.
    #[must_use]
    pub fn new() -> Self {
        Self {
            report_index: ArcSwap::from_pointee(ReportIndex::empty()),
            last_report_ack: ArcSwap::from_pointee(ReportAck::default()),
            cells_created: AtomicUsize::new(0),
            cells_deleted: AtomicUsize::new(0),
            ingestion_lock: Mutex::new(()),
            bootstrap_active: AtomicBool::new(false),
            bootstrap: Mutex::new(None),
        }
    }

    /// Returns `true` if bootstrap phase is active.
    #[must_use]
    pub fn is_bootstrap_active(&self) -> bool {
        self.bootstrap_active.load(std::sync::atomic::Ordering::Acquire)
    }

    /// Accumulates one successful report's Ledger cell-set deltas.
    pub fn accumulate_ledger_cell_deltas(&self, created: usize, deleted: usize) {
        self.cells_created.fetch_add(created, Ordering::Relaxed);
        self.cells_deleted.fetch_add(deleted, Ordering::Relaxed);
    }

    /// Returns lifetime Ledger cell creations and deletions for health.
    #[must_use]
    pub fn ledger_cell_delta_totals(&self) -> (usize, usize) {
        (
            self.cells_created.load(Ordering::Relaxed),
            self.cells_deleted.load(Ordering::Relaxed),
        )
    }

    /// Starts bootstrap phase with the given accumulator.
    ///
    /// # Panics
    ///
    /// Panics if bootstrap is already active or if the mutex is poisoned.
    pub fn start_bootstrap(&self, accumulator: BootstrapAccumulator) {
        let mut guard = self.bootstrap.lock().expect("bootstrap mutex poisoned");
        assert!(guard.is_none(), "bootstrap already active");
        *guard = Some(accumulator);
        drop(guard);
        self.bootstrap_active.store(true, std::sync::atomic::Ordering::Release);
    }

    /// Clears bootstrap phase.
    ///
    /// # Panics
    ///
    /// Panics if the mutex is poisoned.
    pub fn clear_bootstrap(&self) {
        let mut guard = self.bootstrap.lock().expect("bootstrap mutex poisoned");
        *guard = None;
        drop(guard);
        self.bootstrap_active.store(false, std::sync::atomic::Ordering::Release);
    }

    /// Acquires the ingestion lock. Test-only accessor.
    #[cfg(test)]
    pub fn lock_ingestion(&self) -> std::sync::MutexGuard<'_, ()> {
        self.ingestion_lock.lock().expect("ingestion mutex poisoned")
    }
}

impl Default for SentinelSlot {
    fn default() -> Self {
        Self::new()
    }
}

impl std::fmt::Debug for SentinelSlot {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let report_guard = self.report_index.load();
        let ack_guard = self.last_report_ack.load();
        f.debug_struct("SentinelSlot")
            .field("has_report", &report_guard.has_report())
            .field("last_report_ack", &*ack_guard)
            .field("bootstrap_active", &self.is_bootstrap_active())
            .finish_non_exhaustive()
    }
}
