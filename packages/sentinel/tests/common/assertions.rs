// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Assertions and report-inspection helpers for tests.

use std::collections::HashSet;

use torrust_sentinel::{
    AnalysisSetSummary, AnomalyScores, BaselineSnapshot, BatchReport, CellReport, ClipPressureDistribution, ContourSnapshot,
    CoordinationHealth, CoordinationReport, GeometryDistribution, HealthReport, MaturityDistribution, MemberScore,
    RankDistribution, SampleScore, ScoreDistribution, ScoringGeometry, Sentinel128, TrackerMaturity,
};

// ─── assert_invariants() ───────────────────────────────────

/// Assert all structural invariants on a sentinel and its last report.
///
/// Designed to be called after any `ingest()` in any test to verify
/// cross-cutting invariants.
pub fn assert_invariants(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    assert_steiner_tree_bound(report);
    assert_competitive_cap(sentinel, report);
    assert_root_in_full_set(report);
    assert_cell_reports_competitive(report);
    assert_ancestor_reports_non_competitive(report);
    assert_cell_reports_sorted(report);
    assert_ancestor_reports_sorted(report);
    assert_coordination_reports_unique(report);
    assert_no_nan_scores(report);
    assert_analysis_widths(report);
}

/// 1. Analysis set bounds: `full_size <= 1 + K * D_max` (§ALGO S-8.2).
///
/// The *reduced* Steiner tree has at most `2K − 1` nodes, but the
/// materialised investment set includes degree-2 chain intermediaries
/// that push the count higher when competitive cells sit at varying
/// depths.  The correct worst-case bound is `1 + K·D̄` (before
/// sharing); we use `1 + K·D_max` as a practical upper bound.
fn assert_steiner_tree_bound(report: &BatchReport<u128>) {
    let summary = &report.analysis_set_summary;
    let k = summary.competitive_size;
    let d_max = summary.depth_range.1 as usize;
    let bound = 1 + k * d_max;
    assert!(
        summary.full_size <= bound,
        "materialised Steiner tree bound violated: full={}, competitive={}, d_max={}, bound={}",
        summary.full_size,
        k,
        d_max,
        bound,
    );
}

/// 2. Competitive cap: `competitive_size <= analysis_k`.
fn assert_competitive_cap(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    let summary = &report.analysis_set_summary;
    assert!(
        summary.competitive_size <= sentinel.config().analysis_k,
        "competitive set {} exceeds K={}",
        summary.competitive_size,
        sentinel.config().analysis_k,
    );
}

/// 3. Root at depth 0 is always in the full set.
fn assert_root_in_full_set(report: &BatchReport<u128>) {
    let summary = &report.analysis_set_summary;
    if summary.full_size > 0 {
        assert_eq!(summary.depth_range.0, 0, "root (depth 0) must be in the full analysis set");
    }
}

/// 4. `cell_reports` entries are competitive only.
fn assert_cell_reports_competitive(report: &BatchReport<u128>) {
    for cr in &report.cell_reports {
        assert!(
            cr.is_competitive,
            "cell_reports entry at depth {} is not competitive",
            cr.depth
        );
    }
}

/// 5. `ancestor_reports` entries are non-competitive.
fn assert_ancestor_reports_non_competitive(report: &BatchReport<u128>) {
    for ar in &report.ancestor_reports {
        assert!(
            !ar.is_competitive,
            "ancestor_reports entry at depth {} is competitive",
            ar.depth
        );
    }
}

/// 6. Deterministic ordering: `cell_reports` sorted by `gnode_id` ascending.
fn assert_cell_reports_sorted(report: &BatchReport<u128>) {
    for window in report.cell_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "cell_reports not sorted by GNodeId: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// 7. Deterministic ordering: `ancestor_reports` sorted by `gnode_id` ascending.
fn assert_ancestor_reports_sorted(report: &BatchReport<u128>) {
    for window in report.ancestor_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "ancestor_reports not sorted by GNodeId: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// 8. Coordination reports have no duplicate `GNodeId`s.
fn assert_coordination_reports_unique(report: &BatchReport<u128>) {
    let mut seen = HashSet::new();
    for cr in &report.coordination_reports {
        assert!(
            seen.insert(cr.gnode_id),
            "duplicate GNodeId in coordination_reports: {:?}",
            cr.gnode_id,
        );
    }
}

/// 9. No NaN in score fields.
fn assert_no_nan_scores(report: &BatchReport<u128>) {
    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty mean at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement mean at depth {}",
            cr.depth
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise mean at depth {}", cr.depth);
    }
}

/// 10. `analysis_width == 128 - depth` for every cell report.
fn assert_analysis_widths(report: &BatchReport<u128>) {
    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.analysis_width,
            128 - cr.depth as usize,
            "analysis_width mismatch at depth {}",
            cr.depth
        );
    }
}

/// Extract the maximum CUSUM accumulator value across all cell and
/// ancestor reports in a [`BatchReport`].
pub fn max_cusum(report: &BatchReport<u128>) -> f64 {
    report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .flat_map(|cr| {
            [
                cr.scores.novelty.cusum.accumulator,
                cr.scores.displacement.cusum.accumulator,
                cr.scores.surprise.cusum.accumulator,
                cr.scores.coherence.cusum.accumulator,
            ]
        })
        .fold(0.0f64, f64::max)
}

/// Extract the maximum novelty z-score across all cell and
/// ancestor reports in a [`BatchReport`].
pub fn max_novelty_z(report: &BatchReport<u128>) -> f64 {
    report
        .cell_reports
        .iter()
        .chain(report.ancestor_reports.iter())
        .map(|cr| cr.scores.novelty.max_z_score)
        .fold(0.0f64, f64::max)
}

/// Extract the novelty z-score from the root (depth 0) ancestor
/// report, returning `0.0` if no root is present.
pub fn root_novelty_z(report: &BatchReport<u128>) -> f64 {
    report
        .ancestor_reports
        .iter()
        .find(|r| r.depth == 0)
        .map_or(0.0, |r| r.scores.novelty.max_z_score)
}

// ─── assert_reports_identical() ─────────────────────────────

/// Assert that two batch reports are the same report, field for field.
///
/// Every number is compared by its bit pattern rather than by value, because
/// the claim being checked is that two runs computed the same thing and not
/// merely that they landed near each other: equality on values would accept a
/// difference smaller than a printed digit, and would call the two signed
/// zeroes equal while calling two non-numbers different.
///
/// One field is deliberately not compared for its value. The age of a
/// report's oldest observation is an interval between two readings of the
/// sentinel's own clock, so two runs differ in it by construction and no
/// arrangement of the inputs can make them agree. What is compared is whether
/// each report carries an age at all, which is the part that follows from the
/// batch rather than from when the batch happened to be processed.
pub fn assert_reports_identical(left: &BatchReport<u128>, right: &BatchReport<u128>) {
    assert_eq!(
        left.cell_reports.len(),
        right.cell_reports.len(),
        "the two runs reported different numbers of competitive cells"
    );
    for (a, b) in left.cell_reports.iter().zip(&right.cell_reports) {
        assert_cells_identical(a, b, "cell");
    }

    assert_eq!(
        left.ancestor_reports.len(),
        right.ancestor_reports.len(),
        "the two runs reported different numbers of ancestor cells"
    );
    for (a, b) in left.ancestor_reports.iter().zip(&right.ancestor_reports) {
        assert_cells_identical(a, b, "ancestor");
    }

    assert_eq!(
        left.coordination_reports.len(),
        right.coordination_reports.len(),
        "the two runs reported different numbers of coordination contexts"
    );
    for (a, b) in left.coordination_reports.iter().zip(&right.coordination_reports) {
        assert_coordination_identical(a, b);
    }

    assert_contours_identical(&left.contour, &right.contour);
    assert_health_identical(&left.health, &right.health);
    assert_summaries_identical(&left.analysis_set_summary, &right.analysis_set_summary);

    assert_eq!(
        left.oldest_observation_age_micros.is_some(),
        right.oldest_observation_age_micros.is_some(),
        "one run reported an observation age and the other reported none"
    );
}

/// Assert two floating-point figures have the same bit pattern.
fn assert_bits(left: f64, right: f64, what: &str) {
    assert_eq!(left.to_bits(), right.to_bits(), "{what} differs: {left} against {right}");
}

/// Every field of a cell report, for a competitive or an ancestor cell alike.
fn assert_cells_identical(left: &CellReport<u128>, right: &CellReport<u128>, tier: &str) {
    let at = format!("{tier} {:?}", left.gnode_id);

    assert_eq!(left.gnode_id, right.gnode_id, "{at}: handle");
    assert_eq!(left.start, right.start, "{at}: interval start");
    assert_eq!(left.end, right.end, "{at}: interval end");
    assert_eq!(left.depth, right.depth, "{at}: depth");
    assert_eq!(left.analysis_width, right.analysis_width, "{at}: analysis width");
    assert_eq!(left.is_competitive, right.is_competitive, "{at}: competitiveness");
    assert_eq!(left.sample_count, right.sample_count, "{at}: sample count");
    assert_eq!(left.rank, right.rank, "{at}: rank");
    assert_bits(left.energy_ratio, right.energy_ratio, &format!("{at}: energy ratio"));
    assert_bits(
        left.top_singular_value,
        right.top_singular_value,
        &format!("{at}: top singular value"),
    );

    assert_scores_identical(&left.scores, &right.scores, &at);
    assert_maturity_identical(&left.maturity, &right.maturity, &at);
    assert_geometry_identical(&left.geometry, &right.geometry, &at);

    match (&left.per_sample, &right.per_sample) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            assert_eq!(a.len(), b.len(), "{at}: per-sample score count");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                assert_sample_scores_identical(x, y, &format!("{at}: sample {i}"));
            }
        }
        _ => panic!("{at}: one run carried per-sample scores and the other did not"),
    }
}

/// Every field of a coordination report.
fn assert_coordination_identical(left: &CoordinationReport<u128>, right: &CoordinationReport<u128>) {
    let at = format!("context {:?}", left.gnode_id);

    assert_eq!(left.gnode_id, right.gnode_id, "{at}: handle");
    assert_eq!(left.start, right.start, "{at}: interval start");
    assert_eq!(left.end, right.end, "{at}: interval end");
    assert_eq!(left.depth, right.depth, "{at}: depth");
    assert_eq!(left.cells_reporting, right.cells_reporting, "{at}: contributing cells");
    assert_eq!(left.rank, right.rank, "{at}: rank");
    assert_bits(left.energy_ratio, right.energy_ratio, &format!("{at}: energy ratio"));
    assert_bits(
        left.top_singular_value,
        right.top_singular_value,
        &format!("{at}: top singular value"),
    );

    assert_scores_identical(&left.scores, &right.scores, &at);
    assert_maturity_identical(&left.maturity, &right.maturity, &at);
    assert_geometry_identical(&left.geometry, &right.geometry, &at);

    match (&left.per_member, &right.per_member) {
        (None, None) => {}
        (Some(a), Some(b)) => {
            assert_eq!(a.len(), b.len(), "{at}: member score count");
            for (i, (x, y)) in a.iter().zip(b).enumerate() {
                assert_member_scores_identical(x, y, &format!("{at}: member {i}"));
            }
        }
        _ => panic!("{at}: one run carried member scores and the other did not"),
    }
}

/// All four scoring axes.
fn assert_scores_identical(left: &AnomalyScores, right: &AnomalyScores, at: &str) {
    assert_distributions_identical(&left.novelty, &right.novelty, &format!("{at}: novelty"));
    assert_distributions_identical(&left.displacement, &right.displacement, &format!("{at}: displacement"));
    assert_distributions_identical(&left.surprise, &right.surprise, &format!("{at}: surprise"));
    assert_distributions_identical(&left.coherence, &right.coherence, &format!("{at}: coherence"));
}

/// Every figure one axis publishes, baselines and drift evidence included.
fn assert_distributions_identical(left: &ScoreDistribution, right: &ScoreDistribution, at: &str) {
    assert_bits(left.min, right.min, &format!("{at} minimum"));
    assert_bits(left.max, right.max, &format!("{at} maximum"));
    assert_bits(left.mean, right.mean, &format!("{at} mean"));
    assert_bits(left.max_z_score, right.max_z_score, &format!("{at} maximum z-score"));
    assert_bits(left.mean_z_score, right.mean_z_score, &format!("{at} mean z-score"));
    assert_bits(left.clip_pressure, right.clip_pressure, &format!("{at} clip pressure"));

    assert_baselines_identical(&left.baseline, &right.baseline, &format!("{at} baseline"));

    assert_bits(
        left.cusum.accumulator,
        right.cusum.accumulator,
        &format!("{at} cusum accumulator"),
    );
    assert_eq!(
        left.cusum.steps_since_reset, right.cusum.steps_since_reset,
        "{at} cusum steps since reset"
    );
    assert_baselines_identical(
        &left.cusum.slow_baseline,
        &right.cusum.slow_baseline,
        &format!("{at} cusum slow baseline"),
    );
}

/// A baseline's centre and spread.
fn assert_baselines_identical(left: &BaselineSnapshot, right: &BaselineSnapshot, at: &str) {
    assert_bits(left.mean, right.mean, &format!("{at} mean"));
    assert_bits(left.variance, right.variance, &format!("{at} variance"));
}

/// What a tracker has seen, and how much of it was synthetic.
fn assert_maturity_identical(left: &TrackerMaturity, right: &TrackerMaturity, at: &str) {
    assert_eq!(
        left.real_observations, right.real_observations,
        "{at}: real observation count"
    );
    assert_eq!(
        left.noise_observations, right.noise_observations,
        "{at}: synthetic observation count"
    );
    assert_bits(left.noise_influence, right.noise_influence, &format!("{at}: noise influence"));
}

/// The shape of the space a tracker is working in.
fn assert_geometry_identical(left: &ScoringGeometry, right: &ScoringGeometry, at: &str) {
    assert_eq!(left.dim, right.dim, "{at}: dimension");
    assert_eq!(left.cap, right.cap, "{at}: rank cap");
    assert_eq!(left.residual_dof, right.residual_dof, "{at}: residual degrees of freedom");
}

/// One observation's four axes, raw and standardised.
fn assert_sample_scores_identical(left: &SampleScore, right: &SampleScore, at: &str) {
    assert_bits(left.novelty, right.novelty, &format!("{at}: novelty"));
    assert_bits(left.displacement, right.displacement, &format!("{at}: displacement"));
    assert_bits(left.surprise, right.surprise, &format!("{at}: surprise"));
    assert_bits(left.coherence, right.coherence, &format!("{at}: coherence"));
    assert_bits(left.novelty_z, right.novelty_z, &format!("{at}: novelty z-score"));
    assert_bits(
        left.displacement_z,
        right.displacement_z,
        &format!("{at}: displacement z-score"),
    );
    assert_bits(left.surprise_z, right.surprise_z, &format!("{at}: surprise z-score"));
    assert_bits(left.coherence_z, right.coherence_z, &format!("{at}: coherence z-score"));
}

/// One contributing cell's four axes, and the cell they are attributed to.
fn assert_member_scores_identical(left: &MemberScore<u128>, right: &MemberScore<u128>, at: &str) {
    assert_eq!(left.cell_start, right.cell_start, "{at}: cell interval start");
    assert_eq!(left.cell_end, right.cell_end, "{at}: cell interval end");
    assert_eq!(left.cell_depth, right.cell_depth, "{at}: cell depth");
    assert_bits(left.novelty, right.novelty, &format!("{at}: novelty"));
    assert_bits(left.displacement, right.displacement, &format!("{at}: displacement"));
    assert_bits(left.surprise, right.surprise, &format!("{at}: surprise"));
    assert_bits(left.coherence, right.coherence, &format!("{at}: coherence"));
    assert_bits(left.novelty_z, right.novelty_z, &format!("{at}: novelty z-score"));
    assert_bits(
        left.displacement_z,
        right.displacement_z,
        &format!("{at}: displacement z-score"),
    );
    assert_bits(left.surprise_z, right.surprise_z, &format!("{at}: surprise z-score"));
    assert_bits(left.coherence_z, right.coherence_z, &format!("{at}: coherence z-score"));
}

/// The spatial layer as the report describes it.
fn assert_contours_identical(left: &ContourSnapshot, right: &ContourSnapshot) {
    assert_eq!(left.plateau_count, right.plateau_count, "contour: plateau count");
    assert_eq!(left.cell_count, right.cell_count, "contour: cell count");
    assert_bits(left.total_importance, right.total_importance, "contour: total importance");
    assert_eq!(
        left.splits_since_last_report, right.splits_since_last_report,
        "contour: splits since the last report"
    );
    assert_eq!(
        left.net_removals_since_last_report, right.net_removals_since_last_report,
        "contour: net removals since the last report"
    );
}

/// Every operational figure the report carries.
fn assert_health_identical(left: &HealthReport, right: &HealthReport) {
    assert_eq!(left.total_g_nodes, right.total_g_nodes, "health: total nodes");
    assert_eq!(
        left.semi_internal_count, right.semi_internal_count,
        "health: semi-internal nodes"
    );
    assert_eq!(left.active_trackers, right.active_trackers, "health: active trackers");
    assert_eq!(
        left.active_competitive_trackers, right.active_competitive_trackers,
        "health: active competitive trackers"
    );
    assert_eq!(
        left.active_ancestor_trackers, right.active_ancestor_trackers,
        "health: active ancestor trackers"
    );
    assert_eq!(
        left.active_coordination_contexts, right.active_coordination_contexts,
        "health: active coordination contexts"
    );
    assert_eq!(
        left.investment_set_size, right.investment_set_size,
        "health: investment set size"
    );
    assert_eq!(left.warming_trackers, right.warming_trackers, "health: warming trackers");
    assert_eq!(
        left.warming_competitive_targets, right.warming_competitive_targets,
        "health: warming competitive targets"
    );
    assert_eq!(
        left.lifetime_observations, right.lifetime_observations,
        "health: lifetime observations"
    );
    assert_eq!(left.cells_tracked, right.cells_tracked, "health: cells tracked");

    assert_ranks_identical(&left.rank_distribution, &right.rank_distribution, "health");
    assert_maturities_identical(&left.maturity_distribution, &right.maturity_distribution, "health");
    assert_geometries_identical(&left.geometry_distribution, &right.geometry_distribution, "health");
    assert_pressures_identical(&left.clip_pressure_distribution, &right.clip_pressure_distribution);
    assert_coordination_health_identical(&left.coordination_health, &right.coordination_health);
}

/// The rank spread over the live trackers.
fn assert_ranks_identical(left: &RankDistribution, right: &RankDistribution, at: &str) {
    assert_eq!(left.min, right.min, "{at}: minimum rank");
    assert_eq!(left.max, right.max, "{at}: maximum rank");
    assert_bits(left.mean, right.mean, &format!("{at}: mean rank"));
}

/// How much of what the live trackers hold is synthetic.
fn assert_maturities_identical(left: &MaturityDistribution, right: &MaturityDistribution, at: &str) {
    assert_bits(
        left.max_noise_influence,
        right.max_noise_influence,
        &format!("{at}: maximum noise influence"),
    );
    assert_bits(
        left.min_noise_influence,
        right.min_noise_influence,
        &format!("{at}: minimum noise influence"),
    );
    assert_bits(
        left.mean_noise_influence,
        right.mean_noise_influence,
        &format!("{at}: mean noise influence"),
    );
    assert_eq!(left.cold_trackers, right.cold_trackers, "{at}: cold trackers");
}

/// Where the live trackers stand against their own geometric limits.
fn assert_geometries_identical(left: &GeometryDistribution, right: &GeometryDistribution, at: &str) {
    assert_eq!(left.novelty_saturated, right.novelty_saturated, "{at}: novelty saturated");
    assert_eq!(left.novelty_saturable, right.novelty_saturable, "{at}: novelty saturable");
    assert_eq!(left.coherence_inactive, right.coherence_inactive, "{at}: coherence inactive");
}

/// The rejection rate spread over the live trackers.
fn assert_pressures_identical(left: &ClipPressureDistribution, right: &ClipPressureDistribution) {
    assert_bits(left.min, right.min, "health: minimum clip pressure");
    assert_bits(left.max, right.max, "health: maximum clip pressure");
    assert_bits(left.mean, right.mean, "health: mean clip pressure");
}

/// The coordination tier's own health section.
fn assert_coordination_health_identical(left: &CoordinationHealth, right: &CoordinationHealth) {
    assert_eq!(
        left.active_contexts, right.active_contexts,
        "coordination health: active contexts"
    );
    assert_eq!(left.capacity, right.capacity, "coordination health: capacity");
    assert_eq!(left.dim, right.dim, "coordination health: dimension");
    assert_ranks_identical(&left.rank_distribution, &right.rank_distribution, "coordination health");
    assert_maturities_identical(
        &left.maturity_distribution,
        &right.maturity_distribution,
        "coordination health",
    );
    assert_geometries_identical(
        &left.geometry_distribution,
        &right.geometry_distribution,
        "coordination health",
    );
}

/// The summary of what the sentinel is currently investing in.
fn assert_summaries_identical(left: &AnalysisSetSummary, right: &AnalysisSetSummary) {
    assert_eq!(left.competitive_size, right.competitive_size, "summary: competitive size");
    assert_eq!(left.full_size, right.full_size, "summary: full size");
    assert_eq!(
        left.investment_set_size, right.investment_set_size,
        "summary: investment set size"
    );
    assert_eq!(left.depth_range, right.depth_range, "summary: depth range");
    assert_bits(
        left.importance_range.0,
        right.importance_range.0,
        "summary: lowest importance",
    );
    assert_bits(
        left.importance_range.1,
        right.importance_range.1,
        "summary: highest importance",
    );
    assert_eq!(left.v_depth_range, right.v_depth_range, "summary: v-depth range");
    assert_eq!(
        left.degenerate_cells_skipped, right.degenerate_cells_skipped,
        "summary: degenerate cells skipped"
    );
}
