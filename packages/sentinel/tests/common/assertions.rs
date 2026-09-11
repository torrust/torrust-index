// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Assertions and report-inspection helpers for tests.

use std::collections::HashSet;

use torrust_sentinel::{BatchReport, Sentinel128};

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
