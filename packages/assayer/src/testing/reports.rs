// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Golden report and test utilities for report ingestion through assessment.
//!
//! This module provides:
//!
//! - [`golden_report()`] — A fixed `BatchReport<u128>` with analytically
//!   verifiable scores (3 cells: root + 1 competitive + 1 ancestor).
//! - [`golden_report_4cell()`] — A 4-cell variant (root + 2 competitive
//!   + 1 ancestor) matching the acceptance criterion.
//! - [`GoldenValues`] — Pre-computed feature values for extraction verification.
//! - [`minimal_report()`] — A root-only report for minimal valid testing.
//! - [`degraded_report()`] — A report with NaN in one cell's novelty max_z.
//!
//! # Golden Report Variants
//!
//! ## 3-Cell Golden Report (`golden_report()`)
//!
//! | Depth | lo | sample_count | rank | cap | energy_ratio | noise_influence |
//! |-------|-----|------|------|-----|------|------|
//! | 0 (root) | 0 | 1000 | 4 | 8 | 0.85 | 0.05 |
//! | 4 (ancestor) | 0x0000_1000...0 | 200 | 4 | 8 | 0.80 | 0.08 |
//! | 8 (cell) | 0x0000_1200...0 | 50 | 3 | 8 | 0.72 | 0.15 |
//!
//! ## 4-Cell Golden Report (`golden_report_4cell()`)
//!
//! Adds a depth-12 competitive cell for the acceptance criterion:
//!
//! | Depth | lo | sample_count | rank | cap | energy_ratio | noise_influence |
//! |-------|-----|------|------|-----|------|------|
//! | 0 (root) | 0 | 1000 | 4 | 8 | 0.85 | 0.05 |
//! | 4 (ancestor) | 0x0000_1000...0 | 200 | 4 | 8 | 0.80 | 0.08 |
//! | 8 (cell) | 0x0000_1200...0 | 50 | 3 | 8 | 0.72 | 0.15 |
//! | 12 (cell) | 0x0000_1230...0 | 25 | 3 | 8 | 0.65 | 0.20 |
//!
//! # Cross-References
//!
//! - (´setup:extraction:from-batch-report´) — the batch-report shape these
//!   fixtures instantiate, and the fixed width it is compressed to
//! - Report ingestion — Report ingestion pipeline tests
//! - Feature extraction — Per-Sentinel feature extraction tests

#![allow(
    clippy::doc_markdown,
    clippy::must_use_candidate,
    clippy::wildcard_imports,
    clippy::unreadable_literal,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss,
    clippy::cast_lossless,
    clippy::too_many_arguments
)]

use torrust_mudlark::GNodeId;
use torrust_sentinel::{
    AnalysisSetSummary, AnomalyScores, BaselineSnapshot, BatchReport, CellReport, ClipPressureDistribution, ContourSnapshot,
    CoordinationHealth, CoordinationReport, CusumSnapshot, GeometryDistribution, HealthReport, MaturityDistribution,
    RankDistribution, ScoreDistribution, ScoringGeometry, TrackerMaturity,
};

use super::world::World;
use crate::types::SentinelId;

// ═══════════════════════════════════════════════════════════════════════════════
// Coordinate Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// The golden coordinate that routes through all three cells.
///
/// The bit pattern itself is the harness's own: what the fixtures need is one
/// coordinate whose prefixes land in every cell the scenarios report, and any
/// pattern with that property would serve.
///
/// The harness declares the figure as any host declares one
/// (´tab:assayer:harness-golden-figures´): this pattern's leading four, eight
/// and twelve bits are the three cells' lower bounds, which is the whole of
/// what is required of it.
///
/// ´const:assayer:golden-route-coordinate´ (´alg:const:form´)
/// ´const:assayer:golden-route-coordinate-form-x8ec0eee8´
pub const GOLDEN_COORD: u128 = 0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0;

/// Depth-4 cell lo: top 4 bits are 0x1.
///
/// The golden coordinate's own prefix at that depth, zero-filled below it: a
/// cell at a given depth groups the values sharing a prefix of that many bits
/// (´def:encoding:question-meaning´).
///
/// ´const:assayer:golden-depth-four-prefix´ (´alg:const:form´)
/// ´const:assayer:golden-depth-four-prefix-form-xf800949b´
pub const DEPTH_4_LO: u128 = 0x1000_0000_0000_0000_0000_0000_0000_0000;

/// Depth-8 cell lo: top 8 bits are 0x12.
///
/// The golden coordinate's own prefix at that depth, zero-filled below it
/// (´def:encoding:question-meaning´).
///
/// ´const:assayer:golden-depth-eight-prefix´ (´alg:const:form´)
/// ´const:assayer:golden-depth-eight-prefix-form-xa4645789´
pub const DEPTH_8_LO: u128 = 0x1200_0000_0000_0000_0000_0000_0000_0000;

/// Depth-12 cell lo: top 12 bits are 0x123.
///
/// The golden coordinate's own prefix at that depth, zero-filled below it
/// (´def:encoding:question-meaning´).
///
/// ´const:assayer:golden-depth-twelve-prefix´ (´alg:const:form´)
/// ´const:assayer:golden-depth-twelve-prefix-form-x897c256c´
pub const DEPTH_12_LO: u128 = 0x1230_0000_0000_0000_0000_0000_0000_0000;

// ═══════════════════════════════════════════════════════════════════════════════
// Score Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Golden max_z scores by (axis, depth): `[N, D, S, C]` per depth.
pub mod scores {
    /// Root (d=0) max_z: `[N:0.5, D:0.3, S:1.0, C:0.0]`
    ///
    /// One reading per scoring axis (´tab:architecture:sentinel-properties´);
    /// the readings themselves are the fixture's own, chosen to sit quiet at
    /// the root so the depths above it have somewhere to rise from.
    ///
    /// Stimulus the harness authors rather than an output it recorded
    /// (´dec:assayer:golden-report-stimulus´), declared with its reason
    /// (´tab:assayer:harness-golden-figures´). Coherence sits at zero here so
    /// the gradient over that axis is the whole of its rise.
    ///
    /// ´const:assayer:golden-root-peaks´ (´alg:const:form´)
    /// ´const:assayer:golden-root-peaks-form-xcef69bea´
    pub const ROOT_MAX_Z: [f64; crate::types::SCORING_AXIS_COUNT] = [0.5, 0.3, 1.0, 0.0];

    /// Ancestor (d=4) max_z: `[N:1.2, D:0.7, S:1.0, C:0.4]`
    ///
    /// One reading per scoring axis (´tab:architecture:sentinel-properties´).
    ///
    /// The ladder's interior rung, above the root and below the cell on every
    /// axis but surprise: an interior reading is what separates a mean or a
    /// spread view from either endpoint (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-four-peaks´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-four-peaks-form-x1bb8a3e0´
    pub const D4_MAX_Z: [f64; crate::types::SCORING_AXIS_COUNT] = [1.2, 0.7, 1.0, 0.4];

    /// Cell (d=8) max_z: `[N:2.8, D:1.5, S:1.0, C:0.9]`
    ///
    /// One reading per scoring axis (´tab:architecture:sentinel-properties´).
    ///
    /// The loud end of the three-cell report and the deepest entry its views
    /// read; the rise from the root is large enough that a gradient reading is
    /// unmistakable rather than a rounding difference
    /// (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-eight-peaks´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-eight-peaks-form-x731285e9´
    pub const D8_MAX_Z: [f64; crate::types::SCORING_AXIS_COUNT] = [2.8, 1.5, 1.0, 0.9];

    /// Root (d=0) CUSUM: `[N:0.1, D:0.0, S:0.5, C:0.0]`
    ///
    /// One accumulator per scoring axis
    /// (´tab:architecture:sentinel-properties´).
    ///
    /// A second ladder on the same axes, deliberately not proportional to the
    /// peaks so a view confusing the two families would not land on a matching
    /// number (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´). Displacement and coherence are
    /// both zero here, the one repeated reading in either family
    /// (´cav:assayer:golden-discrimination-limits´).
    ///
    /// ´const:assayer:golden-root-accumulators´ (´alg:const:form´)
    /// ´const:assayer:golden-root-accumulators-form-x9c0721f9´
    pub const ROOT_CUSUM: [f64; crate::types::SCORING_AXIS_COUNT] = [0.1, 0.0, 0.5, 0.0];

    /// Ancestor (d=4) CUSUM: `[N:0.3, D:0.2, S:0.5, C:0.1]`
    ///
    /// One accumulator per scoring axis
    /// (´tab:architecture:sentinel-properties´).
    ///
    /// The accumulator ladder's interior rung, and the first depth at which all
    /// four axes differ (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-four-accumulators´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-four-accumulators-form-x13adce1c´
    pub const D4_CUSUM: [f64; crate::types::SCORING_AXIS_COUNT] = [0.3, 0.2, 0.5, 0.1];

    /// Cell (d=8) CUSUM: `[N:0.7, D:0.4, S:0.5, C:0.3]`
    ///
    /// One accumulator per scoring axis
    /// (´tab:architecture:sentinel-properties´).
    ///
    /// The deepest accumulator reading of the three-cell report, with surprise
    /// still held at its root value (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-eight-accumulators´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-eight-accumulators-form-x72d581d8´
    pub const D8_CUSUM: [f64; crate::types::SCORING_AXIS_COUNT] = [0.7, 0.4, 0.5, 0.3];

    /// Cell (d=12) max_z: `[N:3.2, D:1.8, S:1.1, C:1.0]`
    ///
    /// One reading per scoring axis (´tab:architecture:sentinel-properties´).
    ///
    /// The four-cell variant's added rung. Surprise steps once here, so the
    /// acceptance scenario has an axis that is flat over three depths and not
    /// over four (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-twelve-peaks´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-twelve-peaks-form-xb3bbee46´
    pub const D12_MAX_Z: [f64; crate::types::SCORING_AXIS_COUNT] = [3.2, 1.8, 1.1, 1.0];

    /// Cell (d=12) CUSUM: `[N:0.9, D:0.5, S:0.6, C:0.4]`
    ///
    /// One accumulator per scoring axis
    /// (´tab:architecture:sentinel-properties´).
    ///
    /// The four-cell variant's rung, its single surprise step matching the
    /// peaks' so both families describe the same scenario
    /// (´dec:assayer:golden-report-stimulus´),
    /// (´tab:assayer:harness-golden-figures´).
    ///
    /// ´const:assayer:golden-depth-twelve-accumulators´ (´alg:const:form´)
    /// ´const:assayer:golden-depth-twelve-accumulators-form-xef1bf165´
    pub const D12_CUSUM: [f64; crate::types::SCORING_AXIS_COUNT] = [0.9, 0.5, 0.6, 0.4];
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Builders
// ═══════════════════════════════════════════════════════════════════════════════

/// Builds a `ScoreDistribution` with the given max_z and cusum values.
fn make_score(max_z: f64, cusum: f64) -> ScoreDistribution {
    ScoreDistribution {
        min: 0.0,
        max: max_z,
        mean: max_z * 0.5,
        max_z_score: max_z,
        mean_z_score: max_z * 0.5,
        baseline: BaselineSnapshot {
            mean: 0.0,
            variance: 1.0,
        },
        cusum: CusumSnapshot {
            accumulator: cusum,
            slow_baseline: BaselineSnapshot {
                mean: 0.0,
                variance: 1.0,
            },
            steps_since_reset: 100,
        },
        clip_pressure: 0.0,
    }
}

/// Builds an `AnomalyScores` from per-axis max_z and cusum arrays.
fn make_anomaly_scores(
    max_z: [f64; crate::types::SCORING_AXIS_COUNT],
    cusum: [f64; crate::types::SCORING_AXIS_COUNT],
) -> AnomalyScores {
    AnomalyScores {
        novelty: make_score(max_z[0], cusum[0]),
        displacement: make_score(max_z[1], cusum[1]),
        surprise: make_score(max_z[2], cusum[2]),
        coherence: make_score(max_z[3], cusum[3]),
    }
}

/// Builds a `CellReport` at the given depth with specified parameters.
#[allow(clippy::too_many_arguments)]
pub fn make_cell_report(
    start: u128,
    depth: u32,
    sample_count: usize,
    rank: usize,
    cap: usize,
    energy_ratio: f64,
    noise_influence: f64,
    max_z: [f64; crate::types::SCORING_AXIS_COUNT],
    cusum: [f64; crate::types::SCORING_AXIS_COUNT],
    is_competitive: bool,
) -> CellReport<u128> {
    // Compute exclusive end
    // For depth 0: exclusive end = 0 (wraps around to represent full u128 range)
    // For depth > 0: end = start + 2^(128 - depth)
    let end = if depth == 0 {
        0 // Exclusive end of 0 for depth 0 means [0, u128::MAX]
    } else if depth >= 128 {
        start.saturating_add(1)
    } else {
        let width = 1_u128 << (128 - depth);
        start.saturating_add(width)
    };

    CellReport {
        gnode_id: GNodeId::from_parts(depth as usize, 0),
        start,
        end,
        depth,
        analysis_width: (128 - depth.min(128)) as usize,
        is_competitive,
        sample_count,
        rank,
        energy_ratio,
        top_singular_value: 1.0,
        scores: make_anomaly_scores(max_z, cusum),
        maturity: TrackerMaturity {
            real_observations: sample_count as u64,
            noise_observations: 0,
            noise_influence,
        },
        geometry: ScoringGeometry {
            dim: 128,
            cap,
            residual_dof: 128 - rank,
        },
        per_sample: None,
    }
}

/// Default health report for golden reports.
pub const fn default_health() -> HealthReport {
    HealthReport {
        total_g_nodes: 10,
        semi_internal_count: 2,
        active_trackers: 3,
        active_competitive_trackers: 2,
        active_ancestor_trackers: 1,
        active_coordination_contexts: 0,
        investment_set_size: 3,
        warming_trackers: 0,
        warming_competitive_targets: 0,
        lifetime_observations: 1000,
        cells_tracked: 3,
        rank_distribution: RankDistribution {
            min: 3,
            max: 4,
            mean: 3.5,
        },
        maturity_distribution: MaturityDistribution {
            max_noise_influence: 0.2,
            min_noise_influence: 0.05,
            mean_noise_influence: 0.1,
            cold_trackers: 0,
        },
        geometry_distribution: GeometryDistribution {
            novelty_saturated: 0,
            novelty_saturable: 0,
            coherence_inactive: 0,
        },
        coordination_health: CoordinationHealth {
            active_contexts: 0,
            capacity: 4,
            rank_distribution: RankDistribution {
                min: 0,
                max: 0,
                mean: 0.0,
            },
            maturity_distribution: MaturityDistribution {
                max_noise_influence: 0.0,
                min_noise_influence: 0.0,
                mean_noise_influence: 0.0,
                cold_trackers: 0,
            },
            dim: 4,
            geometry_distribution: GeometryDistribution {
                novelty_saturated: 0,
                novelty_saturable: 0,
                coherence_inactive: 0,
            },
        },
        clip_pressure_distribution: ClipPressureDistribution {
            min: 0.0,
            max: 0.0,
            mean: 0.0,
        },
    }
}

/// Default contour snapshot for golden reports.
pub const fn default_contour() -> ContourSnapshot {
    ContourSnapshot {
        plateau_count: 2,
        cell_count: 3,
        total_importance: 1250.0,
        splits_since_last_report: 0,
        net_removals_since_last_report: 0,
    }
}

/// Default analysis set summary for golden reports.
pub const fn default_analysis_set_summary() -> AnalysisSetSummary {
    AnalysisSetSummary {
        competitive_size: 2,
        full_size: 3,
        investment_set_size: 3,
        depth_range: (0, 8),
        importance_range: (50.0, 1000.0),
        v_depth_range: (0, 2),
        degenerate_cells_skipped: 0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// The golden `BatchReport<u128>` with analytically verifiable scores.
///
/// Contains cells at depths 0, 4, and 8. The coordinate [`GOLDEN_COORD`]
/// routes through all three cells.
#[must_use]
pub fn golden_report() -> BatchReport<u128> {
    use scores::*;

    // Root cell (depth 0) - in cell_reports as competitive
    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, true);

    // Depth-8 cell - in cell_reports as competitive
    let cell_d8 = make_cell_report(DEPTH_8_LO, 8, 50, 3, 8, 0.72, 0.15, D8_MAX_Z, D8_CUSUM, true);

    // Depth-4 ancestor - in ancestor_reports
    let ancestor_d4 = make_cell_report(DEPTH_4_LO, 4, 200, 4, 8, 0.80, 0.08, D4_MAX_Z, D4_CUSUM, false);

    BatchReport {
        cell_reports: vec![root, cell_d8],
        ancestor_reports: vec![ancestor_d4],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 2,
            cell_count: 3,
            total_importance: 1250.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 2,
            full_size: 3,
            investment_set_size: 3,
            depth_range: (0, 8),
            importance_range: (50.0, 1000.0),
            v_depth_range: (0, 2),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A 4-cell golden report: root + 2 competitive cells + 1 ancestor.
///
/// Matches the acceptance criterion for `cells_ingested: 4`.
/// The coordinate [`GOLDEN_COORD`] routes through all four cells
/// (depths 0, 4, 8, 12).
#[must_use]
pub fn golden_report_4cell() -> BatchReport<u128> {
    use scores::*;

    // Root cell (depth 0) - competitive
    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, true);

    // Depth-8 cell - competitive
    let cell_d8 = make_cell_report(DEPTH_8_LO, 8, 50, 3, 8, 0.72, 0.15, D8_MAX_Z, D8_CUSUM, true);

    // Depth-12 cell - competitive
    let cell_d12 = make_cell_report(DEPTH_12_LO, 12, 25, 3, 8, 0.65, 0.20, D12_MAX_Z, D12_CUSUM, true);

    // Depth-4 ancestor
    let ancestor_d4 = make_cell_report(DEPTH_4_LO, 4, 200, 4, 8, 0.80, 0.08, D4_MAX_Z, D4_CUSUM, false);

    BatchReport {
        cell_reports: vec![root, cell_d8, cell_d12],
        ancestor_reports: vec![ancestor_d4],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 3,
            cell_count: 4,
            total_importance: 1275.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 3,
            full_size: 4,
            investment_set_size: 4,
            depth_range: (0, 12),
            importance_range: (25.0, 1000.0),
            v_depth_range: (0, 3),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A minimal valid report with only the root cell.
#[must_use]
pub fn minimal_report() -> BatchReport<u128> {
    use scores::*;

    let root = make_cell_report(0, 0, 100, 2, 8, 0.5, 0.1, ROOT_MAX_Z, ROOT_CUSUM, true);

    BatchReport {
        cell_reports: vec![root],
        ancestor_reports: vec![],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 1,
            cell_count: 1,
            total_importance: 100.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 1,
            full_size: 1,
            investment_set_size: 1,
            depth_range: (0, 0),
            importance_range: (100.0, 100.0),
            v_depth_range: (0, 0),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A report with a NaN in one cell's novelty max_z score.
#[must_use]
pub fn degraded_report() -> BatchReport<u128> {
    let mut report = golden_report();

    // Inject NaN into the depth-8 cell's novelty max_z
    if let Some(cell) = report.cell_reports.get_mut(1) {
        cell.scores.novelty.max_z_score = f64::NAN;
    }

    report
}

/// A report missing the root cell (invalid).
#[must_use]
pub fn report_missing_root() -> BatchReport<u128> {
    use scores::*;

    // Only the depth-8 cell, no root
    let cell_d8 = make_cell_report(DEPTH_8_LO, 8, 50, 3, 8, 0.72, 0.15, D8_MAX_Z, D8_CUSUM, true);

    BatchReport {
        cell_reports: vec![cell_d8],
        ancestor_reports: vec![],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 1,
            cell_count: 1,
            total_importance: 50.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 1,
            full_size: 1,
            investment_set_size: 1,
            depth_range: (8, 8),
            importance_range: (50.0, 50.0),
            v_depth_range: (0, 0),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A report with duplicate cells at the same (lo, depth).
#[must_use]
pub fn report_with_duplicates() -> BatchReport<u128> {
    use scores::*;

    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, true);

    // Two cells at the same depth 8 with same lo
    let cell_d8_a = make_cell_report(DEPTH_8_LO, 8, 50, 3, 8, 0.72, 0.15, D8_MAX_Z, D8_CUSUM, true);
    let cell_d8_b = make_cell_report(DEPTH_8_LO, 8, 60, 4, 8, 0.75, 0.12, D8_MAX_Z, D8_CUSUM, true);

    BatchReport {
        cell_reports: vec![root, cell_d8_a, cell_d8_b],
        ancestor_reports: vec![],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 2,
            cell_count: 3,
            total_importance: 1110.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 3,
            full_size: 3,
            investment_set_size: 3,
            depth_range: (0, 8),
            importance_range: (50.0, 1000.0),
            v_depth_range: (0, 2),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A report with a non-dyadic interval.
#[must_use]
pub fn report_non_dyadic() -> BatchReport<u128> {
    use scores::*;

    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, true);

    // Cell with misaligned start for its depth (start=1, depth=8 is not dyadic)
    let bad_cell = make_cell_report(1, 8, 50, 3, 8, 0.72, 0.15, D8_MAX_Z, D8_CUSUM, true);

    BatchReport {
        cell_reports: vec![root, bad_cell],
        ancestor_reports: vec![],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 2,
            cell_count: 2,
            total_importance: 1050.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 2,
            full_size: 2,
            investment_set_size: 2,
            depth_range: (0, 8),
            importance_range: (50.0, 1000.0),
            v_depth_range: (0, 1),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// A report with many cells (100 cells at various depths).
#[must_use]
pub fn report_many_cells() -> BatchReport<u128> {
    use scores::*;

    let root = make_cell_report(0, 0, 1000, 4, 8, 0.85, 0.05, ROOT_MAX_Z, ROOT_CUSUM, true);

    // Generate 99 more cells at depths 4, 8, 12, 16, etc.
    let mut cells = vec![root];

    for i in 0..99 {
        let depth = 4 + (i % 28) * 4; // depths 4, 8, 12, ..., 112
        // Compute a valid lo for this depth
        let shift = 128 - depth;
        let lo = (i as u128) << shift;

        let cell = make_cell_report(lo, depth, 50, 3, 8, 0.7, 0.1, D8_MAX_Z, D8_CUSUM, true);
        cells.push(cell);
    }

    BatchReport {
        cell_reports: cells,
        ancestor_reports: vec![],
        coordination_reports: vec![],
        contour: ContourSnapshot {
            plateau_count: 10,
            cell_count: 100,
            total_importance: 6000.0,
            splits_since_last_report: 0,
            net_removals_since_last_report: 0,
        },
        health: default_health(),
        analysis_set_summary: AnalysisSetSummary {
            competitive_size: 100,
            full_size: 100,
            investment_set_size: 100,
            depth_range: (0, 112),
            importance_range: (50.0, 1000.0),
            v_depth_range: (0, 99),
            degenerate_cells_skipped: 0,
        },
        oldest_observation_age_micros: None,
    }
}

/// Build a cell report for injection with custom score values.
#[must_use]
pub fn make_test_cell(
    start: u128,
    depth: u32,
    sample_count: usize,
    max_z: [f64; crate::types::SCORING_AXIS_COUNT],
    cusum: [f64; crate::types::SCORING_AXIS_COUNT],
    noise_influence: f64,
) -> CellReport<u128> {
    make_cell_report(start, depth, sample_count, 4, 8, 0.8, noise_influence, max_z, cusum, true)
}

/// Build a cell report with NaN in a specific axis's max_z.
#[must_use]
pub fn make_nan_cell(axis: usize) -> CellReport<u128> {
    let mut cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);

    match axis {
        0 => cell.scores.novelty.max_z_score = f64::NAN,
        1 => cell.scores.displacement.max_z_score = f64::NAN,
        2 => cell.scores.surprise.max_z_score = f64::NAN,
        3 => cell.scores.coherence.max_z_score = f64::NAN,
        _ => {}
    }

    cell
}

/// Build a cell report with Inf in the CUSUM.
#[must_use]
pub fn make_inf_cusum_cell() -> CellReport<u128> {
    let mut cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    cell.scores.surprise.cusum.accumulator = f64::INFINITY;
    cell
}

/// Build a cell report with negative baseline variance.
#[must_use]
pub fn make_negative_variance_cell() -> CellReport<u128> {
    let mut cell = make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 0.1);
    cell.scores.novelty.baseline.variance = -0.5;
    cell
}

/// Build a cell report with noise_influence > 1.0.
#[must_use]
pub fn make_high_noise_cell() -> CellReport<u128> {
    make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, 1.5)
}

/// Build a cell report with negative noise_influence.
#[must_use]
pub fn make_negative_noise_cell() -> CellReport<u128> {
    make_test_cell(DEPTH_8_LO, 8, 50, scores::D8_MAX_Z, scores::D8_CUSUM, -0.1)
}

/// Build a coordination report for testing.
#[must_use]
pub fn make_coordination_report(start: u128, depth: u32, cells_reporting: usize) -> CoordinationReport<u128> {
    let end = if depth == 0 {
        0 // Exclusive end of 0 for depth 0 means [0, u128::MAX]
    } else if depth >= 128 {
        start.saturating_add(1)
    } else {
        let width = 1_u128 << (128 - depth);
        start.saturating_add(width)
    };

    CoordinationReport {
        gnode_id: GNodeId::from_parts(depth as usize, 0),
        start,
        end,
        depth,
        cells_reporting,
        rank: 2,
        energy_ratio: 0.7,
        top_singular_value: 1.0,
        scores: make_anomaly_scores([1.0, 0.5, 0.8, 0.3], [0.2, 0.1, 0.3, 0.1]),
        maturity: TrackerMaturity {
            real_observations: 100,
            noise_observations: 0,
            noise_influence: 0.1,
        },
        geometry: ScoringGeometry {
            dim: 16,
            cap: 4,
            residual_dof: 12,
        },
        per_member: None,
    }
}

/// Ingest the canonical golden report for an already-registered Sentinel.
///
/// Cross-world reported-Sentinel tests use this immediately before final
/// assessment so snapshot staleness does not dominate the decision
/// layer invariant under test.
#[track_caller]
pub fn refresh_golden_report(world: &World, name: &'static str) {
    let ack = world
        .receive_report(name, golden_report())
        .expect("refresh golden Sentinel report");
    assert_eq!(ack.cells_in_report, 3, "golden report cell count");
    assert_eq!(ack.degraded_cells, 0, "golden report should not degrade");
}

/// Register a Sentinel and ingest the canonical golden report.
///
/// The golden report is the compact public fixture whose
/// [`GOLDEN_COORD`] routes
/// through root, ancestor, and competitive cells. Returning the
/// Sentinel id lets tests build custom requests while keeping the
/// report-setup ceremony in one place.
#[track_caller]
pub fn attach_golden_reporting_sentinel(world: &mut World, name: &'static str) -> SentinelId {
    let sentinel = world.register_sentinel(name).expect("register golden Sentinel");
    // `flush_labels` is also the harness barrier for queued lifecycle commands.
    world
        .flush_labels()
        .expect("publish golden Sentinel lifecycle before report ingestion");
    refresh_golden_report(world, name);
    sentinel
}
