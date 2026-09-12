// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Pending assessment buffer for deferred labelling.
//!
//! # Overview
//!
//! This module provides the data structures for storing pending assessments
//! awaiting outcome labels. When an assessment completes, its extracted features
//! and metadata are stored in the pending buffer. When a label arrives, the
//! stored context is retrieved and used for the model update sequence.
//!
//! # Key Types
//!
//! - [`SentinelExtraction`] — Per-Sentinel extracted features
//! - [`PendingAssessment`] — Full in-memory entry with all assessment context
//! - [`ReportOrigin`] — Where the oldest Sentinel evidence one assessment read came from
//! - [`PendingContext`] — Journal-stripped version for persistence
//! - [`PendingBuffer`] — `DashMap` + FIFO storage with capacity limits
//!
//! # Capacity Management
//!
//! The buffer has a configurable capacity (`capacity`) and expiry horizon
//! (`expiry_horizon`). On each insert, up to 16 stale entries are lazily
//! evicted (expired or over-capacity). This amortised eviction keeps insert
//! latency bounded.
//!
//! # Cross-References
//!
//! - (´dec:retention:pending-map´) — how an assessment awaiting its label is held
//! - (´def:runtime:pending-entry´) — what one pending entry carries
//! - (´req:publication:pending-buffer´) — the buffer read concurrently with the paths that write it

// Layer 2 data structures realising the pending map (´dec:retention:pending-map´).
// Wired into Assayer in Layer 3.
#![allow(dead_code)]

mod buffer;
mod entry;

pub use buffer::PendingBuffer;
pub use entry::{
    PendingAssessment, PendingContext, PendingRiskBasis, ReportOrigin, SentinelExtraction, StoragePrecision, StoredFeatures,
};
