// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Report index types for Sentinel report routing.
//!
//! This module provides the `ReportIndex` structure that enables depth-walk
//! routing from coordinates to `ReportCellEntry` values. Each entry captures
//! extracted fields from Sentinel `CellReport<u128>` needed by Layer 3's
//! feature extraction, which reads the per-axis maxima along the ancestor
//! chain (´tab:extraction:chain-z-scores´).
//!
//! # Cross-References
//!
//! - (´dec:ordering:synchronous-reception´) — reception is synchronous and
//!   serialised per Sentinel
//! - (´setup:extraction:from-batch-report´) — what a batch report offers, and
//!   the fixed width it is compressed to
//! - Spike 2 (Layer 2 Plan) — Type mappings from Sentinel

use std::collections::HashMap;
use std::time::Instant;

use crate::types::{LedgerKey, SCORING_AXIS_COUNT};

// ═══════════════════════════════════════════════════════════════════════════════
// Score Snapshot Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Snapshot of scores for a single anomaly axis.
///
/// Captures z-scores, CUSUM state, and baseline statistics extracted from
/// Sentinel's `ScoreDistribution`, along the extraction path the spike
/// confirmed.
// Layer 1 scaffolding; Layer 2 fills it at reception
// (´dec:ordering:synchronous-reception´).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisScoreSnapshot {
    /// Maximum z-score in the batch.
    pub max_z: f64,
    /// Mean z-score in the batch.
    pub mean_z: f64,
    /// CUSUM accumulator value.
    pub cusum: f64,
    /// Baseline mean from the tracker.
    pub baseline_mean: f64,
    /// Baseline variance from the tracker.
    pub baseline_variance: f64,
    /// Clip pressure indicator (health diagnostic).
    pub clip_pressure: f64,
}

/// Score snapshots for all four anomaly axes.
///
/// Novelty, displacement, surprise, coherence — four axes each, as the chain
/// views read them (´tab:extraction:chain-z-scores´).
// Layer 1 scaffolding; Layer 2 fills it at reception
// (´dec:ordering:synchronous-reception´).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisScoreSet {
    /// Novelty axis scores.
    pub novelty: AxisScoreSnapshot,
    /// Displacement axis scores.
    pub displacement: AxisScoreSnapshot,
    /// Surprise axis scores.
    pub surprise: AxisScoreSnapshot,
    /// Coherence axis scores.
    pub coherence: AxisScoreSnapshot,
}

impl AxisScoreSet {
    /// The maximum z-score of every scoring axis, in the order the set declares them.
    ///
    /// The extraction reads a report's axes only through this projection and its
    /// CUSUM twin, so the axis order is written down once here rather than
    /// re-spelled at each per-axis loop. The width is the arity itself
    /// (´const:assayer:scoring-axis-arity´): an axis added to or removed from the
    /// set above leaves this array literal the wrong length, which is a compile
    /// error at this line rather than a neighbour's slot read as an axis's own.
    #[must_use]
    pub const fn max_z_per_axis(&self) -> [f64; SCORING_AXIS_COUNT] {
        [
            self.novelty.max_z,
            self.displacement.max_z,
            self.surprise.max_z,
            self.coherence.max_z,
        ]
    }

    /// The CUSUM accumulator of every scoring axis, in the order the set declares them.
    ///
    /// The CUSUM half of the projection [`Self::max_z_per_axis`] describes; the
    /// same arity holds it to the same width for the same reason.
    #[must_use]
    pub const fn cusum_per_axis(&self) -> [f64; SCORING_AXIS_COUNT] {
        [
            self.novelty.cusum,
            self.displacement.cusum,
            self.surprise.cusum,
            self.coherence.cusum,
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Entry Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracted cell entry from a Sentinel `CellReport<u128>`.
///
/// Contains all fields needed by Layer 3's feature extraction, which resolves
/// each one through the single dimension map (´dec:vector:sole-resolver´), by
/// the type mapping and extraction path the spike confirmed.
// Layer 1 scaffolding; Layer 2 fills it at reception
// (´dec:ordering:synchronous-reception´).
#[allow(dead_code)]
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReportCellEntry {
    /// Depth in the dyadic hierarchy (0–128). Cast from `u32` at ingestion.
    pub depth: u8,
    /// Number of samples in the batch.
    pub sample_count: usize,
    /// Whether this cell is in the competitive set.
    pub is_competitive: bool,
    /// Rank within the analysis set.
    pub rank: usize,
    /// Capacity from scoring geometry.
    pub cap: usize,
    /// Energy ratio from cell report.
    pub energy_ratio: f64,
    /// Noise influence from tracker maturity.
    pub noise_influence: f64,
    /// Scores for all four anomaly axes.
    pub scores: AxisScoreSet,
    /// Whether this entry was produced under degraded mode.
    pub degraded: bool,
}

impl Default for ReportCellEntry {
    fn default() -> Self {
        Self {
            depth: 0,
            sample_count: 0,
            is_competitive: false,
            rank: 0,
            cap: 0,
            energy_ratio: 0.0,
            noise_influence: 0.0,
            scores: AxisScoreSet::default(),
            degraded: false,
        }
    }
}

/// Coordination context entry extracted from `CoordinationReport<u128>`.
///
/// Captures the coordination-level anomaly signals the extraction reads from
/// the batch report's coordination summary (´tab:extraction:coordination´).
// Layer 1 scaffolding; Layer 2 fills it at reception
// (´dec:ordering:synchronous-reception´).
#[allow(dead_code)]
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoordinationEntry {
    /// Depth in the dyadic hierarchy (0–128).
    pub depth: u8,
    /// Number of cells reporting at this coordination level.
    pub cells_reporting: usize,
    /// Scores for all four anomaly axes.
    pub scores: AxisScoreSet,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Level Data
// ═══════════════════════════════════════════════════════════════════════════════

/// Summary data for a report level.
///
/// Captures structural information from `AnalysisSetSummary` and
/// `ContourSnapshot`, by the field mappings the spike settled, and supplies
/// the report's structural summary (´tab:extraction:chain-structure´).
// Layer 1 scaffolding; Layer 2 fills it at reception
// (´dec:ordering:synchronous-reception´).
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ReportLevelData {
    /// Number of cells in the competitive set.
    pub competitive_cell_count: usize,
    /// Total number of cells in the analysis set.
    pub full_set_size: usize,
    /// Range of depths in the analysis set `(min, max)`.
    pub depth_range: (u8, u8),
    /// Number of plateaus in the contour.
    pub plateau_count: usize,
    /// Total importance across the analysis set.
    pub total_importance: f64,
    /// Number of terminal cells in the Sentinel contour.
    pub contour_cell_count: usize,
    /// Cells created by structural splits since the previous report.
    pub splits_since_last_report: u32,
    /// Net structural removals since the previous report.
    pub net_removals_since_last_report: u32,
    /// Range of importance across the competitive analysis set.
    pub importance_range: (f64, f64),
    /// Range of V-tree depths across the competitive analysis set.
    pub v_depth_range: (usize, usize),
    /// Cells excluded because their suffix geometry was degenerate.
    pub degenerate_cells_skipped: usize,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Index
// ═══════════════════════════════════════════════════════════════════════════════

/// Index for routing coordinates to report cell entries.
///
/// Provides depth-walk routing: given a coordinate, walks from `max_depth`
/// down to 0, returning the deepest cell entry that contains the coordinate.
///
/// # Construction
///
/// Phase 1 tests construct `ReportIndex` manually. Phase 2 adds the ingestion
/// pipeline that constructs from `BatchReport<u128>`.
///
/// # Cross-References
///
/// - (´dec:memory:depth-walk´) — a read walks depth to the first hit
/// - Built to the summary's `ReportIndex` shape
#[derive(Clone, Debug)]
pub struct ReportIndex {
    /// Cell entries indexed by `LedgerKey`.
    cells: HashMap<LedgerKey, ReportCellEntry>,
    /// Coordination context entries.
    coordination: Vec<CoordinationEntry>,
    /// Summary data for the report level.
    level_data: ReportLevelData,
    /// Maximum depth present in the index.
    max_depth: u8,
    /// Timestamp when the report was received (None = no report yet).
    received_at: Option<Instant>,
    /// How old the report's oldest observation already was when the Sentinel
    /// emitted it, in microseconds, exactly as the wire carried it.
    ///
    /// This is the only figure anything in this process holds about the time
    /// before a report arrived, and it is the first stage of the end-to-end
    /// feedback latency (´def:monitoring:feedback-latency´). It is a lower
    /// bound on that stage and never the stage itself: it stops at the
    /// Sentinel's emission and says nothing about how long the host then held
    /// the report before handing it over, which is an explicitly unmeasured
    /// residual on the producing side.
    ///
    /// Kept in the producer's own microseconds rather than converted here, so
    /// the figure a consumer reads is the figure the Sentinel measured; the
    /// conversion to the seconds the health surface reports happens once, at
    /// that surface.
    ///
    /// `None` for a report that carried no age — an empty batch, or a payload
    /// written before the field existed — and for an index with no report at
    /// all. All three say the same thing: there is nothing here to read.
    oldest_observation_age_micros: Option<u64>,
}

impl Default for ReportIndex {
    fn default() -> Self {
        Self::empty()
    }
}

#[allow(dead_code)] // Phase 1: used in Phase 2 and tests
impl ReportIndex {
    /// Creates an empty `ReportIndex` with no report.
    ///
    /// `has_report()` returns `false` until a report is ingested.
    #[must_use]
    pub fn empty() -> Self {
        Self {
            cells: HashMap::new(),
            coordination: Vec::new(),
            level_data: ReportLevelData::default(),
            max_depth: 0,
            received_at: None,
            oldest_observation_age_micros: None,
        }
    }

    /// Creates a `ReportIndex` from pre-built components, stamping
    /// `received_at` with `Instant::now()` and carrying no observation age.
    ///
    /// Convenience constructor for tests. The ingestion pipeline
    /// uses [`from_components_at`](Self::from_components_at) so
    /// that `received_at` reflects the instant the report arrived
    /// at the Assayer — the point staleness is measured from
    /// (´def:extraction:report-staleness´) — not the later point at
    /// which the index is materialised, and so that the age the report
    /// carried arrives with it.
    #[must_use]
    pub fn from_components(
        cells: HashMap<LedgerKey, ReportCellEntry>,
        coordination: Vec<CoordinationEntry>,
        level_data: ReportLevelData,
        max_depth: u8,
    ) -> Self {
        Self::from_components_at(cells, coordination, level_data, max_depth, Instant::now(), None)
    }

    /// Creates a `ReportIndex` from pre-built components with an
    /// explicit `received_at` timestamp and the age the report carried.
    ///
    /// The ingestion pipeline stamps `received_at` at function
    /// entry, where staleness is measured from
    /// (´def:extraction:report-staleness´), and threads that value through so
    /// the timestamp reflects report arrival rather than index
    /// materialisation. The observation age travels beside it because the two
    /// meet at exactly one point — this index — and only together do they
    /// span the interval from a Sentinel's observation to the Assayer holding
    /// the evidence of it (´def:monitoring:feedback-latency´).
    #[must_use]
    pub const fn from_components_at(
        cells: HashMap<LedgerKey, ReportCellEntry>,
        coordination: Vec<CoordinationEntry>,
        level_data: ReportLevelData,
        max_depth: u8,
        received_at: Instant,
        oldest_observation_age_micros: Option<u64>,
    ) -> Self {
        Self {
            cells,
            coordination,
            level_data,
            max_depth,
            received_at: Some(received_at),
            oldest_observation_age_micros,
        }
    }

    /// Returns `true` if a report has been received.
    #[must_use]
    pub const fn has_report(&self) -> bool {
        self.received_at.is_some()
    }

    /// Routes a coordinate to the deepest matching cell entry.
    ///
    /// Performs a depth-walk from `max_depth` down to 0, probing the
    /// `HashMap` at each depth using `dyadic_ancestor_lo(coord, depth)`
    /// to construct the `LedgerKey`. Returns the first (deepest) hit.
    ///
    /// # Returns
    ///
    /// - `Some(&ReportCellEntry)` — deepest cell containing `coord`
    /// - `None` — no report received (empty index)
    #[must_use]
    pub fn route(&self, coord: u128) -> Option<&ReportCellEntry> {
        if !self.has_report() {
            return None;
        }

        // Depth-walk from max_depth down to 0
        for depth in (0..=self.max_depth).rev() {
            let key = LedgerKey::from_coordinate(coord, depth);
            if let Some(entry) = self.cells.get(&key) {
                return Some(entry);
            }
        }

        None
    }

    /// Returns a reference to the cell map.
    #[must_use]
    pub const fn cells(&self) -> &HashMap<LedgerKey, ReportCellEntry> {
        &self.cells
    }

    /// Returns a slice of coordination entries.
    #[must_use]
    pub fn coordination(&self) -> &[CoordinationEntry] {
        &self.coordination
    }

    /// Returns a reference to the level data.
    #[must_use]
    pub const fn level_data(&self) -> &ReportLevelData {
        &self.level_data
    }

    /// Returns the maximum depth in this index.
    #[must_use]
    pub const fn max_depth(&self) -> u8 {
        self.max_depth
    }

    /// Returns the `Instant` when this report was received, if any.
    #[must_use]
    pub const fn received_at(&self) -> Option<Instant> {
        self.received_at
    }

    /// How old the oldest observation behind this report already was when the
    /// Sentinel emitted it, in microseconds, or `None` when the report carried
    /// no age.
    ///
    /// A lower bound on the first stage of the feedback latency, never the
    /// stage itself (´def:monitoring:feedback-latency´).
    #[must_use]
    pub const fn oldest_observation_age_micros(&self) -> Option<u64> {
        self.oldest_observation_age_micros
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Acknowledgement
// ═══════════════════════════════════════════════════════════════════════════════

/// Acknowledgement returned by `receive_sentinel_report()`.
///
/// Provides summary statistics about the ingested report.
///
/// TODO ´todo:code:this-is-the´: This is the
/// internal ingestion ack stored on `SentinelSlot`, behind the diagnostics
/// reception answers with (´dec:surface:diagnostic-ack´). Before adding fields
/// or changing names, research the final type boundary: update the
/// internal/public mapping table, add mapper tests in `api::report`, and
/// decide whether health consumes a separate public health type. Do not
/// expose ingestion-only counters by accident.
#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct ReportAck {
    /// Number of cells ingested from the report.
    pub cells_ingested: usize,
    /// Number of cells marked as degraded.
    pub degraded_cells: usize,
    /// Number of coordination contexts ingested.
    pub coordination_contexts: usize,
    /// Number of new cells created in the Ledger.
    pub cells_created: usize,
    /// Number of cells deleted from the Ledger.
    pub cells_deleted: usize,
}
