// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sentinel report types, slot management, and ingestion pipeline.
//!
//! This module provides:
//! - `ReportIndex` — depth-walk routing from coordinates to cell entries
//! - `SentinelSlot` — per-Sentinel state with atomic report swapping
//! - Report entry types: `ReportCellEntry`, `CoordinationEntry`, etc.
//! - `ingest_report` — 6-step ingestion pipeline for `BatchReport<u128>`
//! - Structural and cell-level validation
//!
//! # Module Structure
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`index`] | `ReportIndex`, entry types, `ReportAck` |
//! | [`slot`] | `SentinelSlot` with atomic report swapping |
//! | [`ingestion`] | 6-step ingestion pipeline |
//! | [`validation`] | Structural and data quality validation |
//!
//! # Cross-References
//!
//! - (´dec:ordering:synchronous-reception´) — reception is synchronous and
//!   serialised per Sentinel
//! - (´dec:surface:slot-map´) — slots live in a concurrent map keyed by Sentinel
//! - (´def:extraction:slot´) — what a per-Sentinel slot occupies and holds
//! - (´dec:surface:reception-isolation´) — the four steps reception runs, and
//!   the state it does not touch

mod index;
mod ingestion;
mod slot;
mod validation;

// Re-exports for crate-internal use and integration tests.
// Items are `pub` (not `pub(crate)`) to allow re-export from the crate root,
// but remain hidden from the public API because this module is private.
// TODO ´todo:code:research-renaming-this-re-export-to´: research renaming this re-export to
// `InternalReportAck` or forcing callers through `report::index` before the
// next change to the diagnostics reception acknowledges with
// (´dec:surface:diagnostic-ack´). The crate root exports `api::ReportAck`, and
// the identical short name is easy to import accidentally in test/support code.
#[allow(unused_imports)]
pub use index::{AxisScoreSet, AxisScoreSnapshot, CoordinationEntry, ReportAck, ReportCellEntry, ReportIndex, ReportLevelData};
pub use ingestion::{IngestionConfig, ingest_report};
pub use slot::SentinelSlot;
#[allow(unused_imports)]
pub use validation::{validate_and_extract_cell, validate_structural};
