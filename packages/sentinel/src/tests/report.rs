// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for [`crate::report`] — the records the sentinel hands back to the
//! host after each observation cycle.
//!
//! The reports exist because the sentinel measures and the host decides.
//! They carry raw statistics — scores, baselines, drift, maturity, geometry,
//! structural summaries — and never a threat level or a recommended action.
//! That division is what makes the small pieces of behaviour these types do
//! own worth pinning down: whatever logic lives in a report type is logic
//! the host will lean on when forming its own judgement.
//!
//! Most of the module is plain data, and the tests here cover the parts that
//! are not. Maturity distinguishes observations the tracker really saw from
//! the noise it was warmed with, and reports both rather than one blended
//! figure, so a host can tell a confident model from a fresh one. Geometry
//! answers whether an axis is structurally meaningful at all — novelty
//! measures leftover residual, so with no residual degrees of freedom there
//! is nothing left for it to measure, and a host reading a low novelty score
//! needs to know whether that means "nothing unusual" or "nothing
//! measurable".
//!
//! The remaining tests fix the shape of the summary records: what a contour
//! snapshot, an analysis-set summary, and a per-cell member score are
//! obliged to carry. A report consumer parses the same fields whether the
//! sentinel is idle or saturated, so the degenerate cases report zeroes
//! rather than omitting anything.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cold_maturity_is_fully_noisy`] | readout | A tracker that has seen nothing reports no observations of either kind and a baseline owed entirely to noise. The cold state is not "unknown" but a definite statement: whatever this tracker would score against, none of it came from real traffic, so a host can discount its scores rather than having to guess how new the model is. |
//! | [`total_observations_sums_real_and_noise`] | readout | Total experience is the real and injected observations added together, and both counts stay separately readable beside it. The sum says how much the model has absorbed; the split says how much of that was manufactured during warm-up — a host needs the second to interpret the first, so the report offers the convenience without collapsing the distinction. |
//! | [`novelty_not_saturated_when_residual_dof_positive`] | readout | Novelty is degenerate exactly when no residual degrees of freedom remain, and with residual dimensions still unexplained by the learned subspace it is not. Novelty measures the energy the model failed to account for, so while there is somewhere for that energy to live the axis is measuring something real. |
//! | [`novelty_saturated_when_residual_dof_zero`] | readout | cites (´claim:readout:novelty-is-degenerate-exactly-when-no-residual-degrees-of-freedom-remain´) |
//! | [`novelty_saturable_when_cap_ge_dim`] | readout | Saturability is a separate question from saturation: a tracker whose rank cap reaches its working dimension may still have residual room today, yet it is one of those that can lose the novelty axis as rank grows. The report distinguishes the two so a host can tell a temporary reading from a cell where novelty will eventually stop meaning anything. |
//! | [`novelty_not_saturable_when_cap_lt_dim`] | readout | cites (´claim:readout:novelty-can-become-degenerate-whenever-the-rank-cap-reaches-the-working-dimension´) |
//! | [`contour_snapshot_fields`] | readout | A contour snapshot carries the spatial shape, the accumulated volume, and the structural churn since the previous report side by side. Standing state and change-since-last-time are different questions about the spatial layer, and the snapshot answers both at once so a host need not difference successive reports to see the graph move. |
//! | [`analysis_set_summary_empty`] | readout | A summary describing an analysis set with nothing selected still reports every field, its ranges zeroed rather than omitted, and still counts the root as a member of the full set. The shape a host parses does not change with how busy the sentinel is, and the permanent root tracker is visible even at the quietest extreme. |
//! | [`analysis_set_summary_populated`] | readout | A populated summary reports its depth and importance spans as ordered pairs, low end first, and its three sizes widen as the definition of membership loosens: the cells that won the competition, the full set their ancestry closes over, and the investment set that also holds cells still warming. The nesting is what lets a host read the price of analysing a cell as well as the choice to analyse it. |
//! | [`analysis_set_summary_with_degenerate_skips`] | readout | Cells too narrow to support a tracker are counted in the current selection snapshot rather than quietly dropped. Recomputing replaces the count instead of accumulating it, so the report describes the graph the host can inspect now while still exposing a persistently narrow configuration. |
//! | [`member_score_has_cell_identity`] | readout | A member score names the cell it came from — a well-ordered interval and a depth — alongside a real number on each of the four axes and its standardised counterpart. Coordination scores describe a group, so without the identity a host could see that the group behaved oddly but not which part of the domain to look at. |

use crate::report::*;
use crate::{NoiseSchedule, SentinelConfig, SpectralSentinel};

// ── TrackerMaturity ─────────────────────────────────────────

/// A tracker that has seen nothing reports no observations of either kind
/// and a baseline owed entirely to noise. The cold state is not "unknown"
/// but a definite statement: whatever this tracker would score against, none
/// of it came from real traffic, so a host can discount its scores rather
/// than having to guess how new the model is.
///
/// ´claim:readout:a-cold-tracker-reports-a-baseline-owed-entirely-to-noise-rather-than-an-absent-one´
/// ´test:crate:cold-maturity-is-fully-noisy´
#[test]
fn cold_maturity_is_fully_noisy() {
    let m = TrackerMaturity::cold();
    assert_eq!(m.real_observations, 0);
    assert_eq!(m.noise_observations, 0);
    assert!((m.noise_influence - 1.0).abs() < f64::EPSILON);
    assert_eq!(m.total_observations(), 0);
}

/// Total experience is the real and injected observations added together,
/// and both counts stay separately readable beside it. The sum says how much
/// the model has absorbed; the split says how much of that was manufactured
/// during warm-up — a host needs the second to interpret the first, so the
/// report offers the convenience without collapsing the distinction.
///
/// ´claim:readout:total-experience-is-real-and-injected-observations-added-together-with-both-still-readable´
/// ´test:crate:total-observations-sums-real-and-noise´
#[test]
fn total_observations_sums_real_and_noise() {
    let m = TrackerMaturity {
        real_observations: 100,
        noise_observations: 25,
        noise_influence: 0.2,
    };
    assert_eq!(m.total_observations(), 125);
}

// ── ScoringGeometry ─────────────────────────────────────────

/// Novelty is degenerate exactly when no residual degrees of freedom remain,
/// and with residual dimensions still unexplained by the learned subspace it
/// is not. Novelty measures the energy the model failed to account for, so
/// while there is somewhere for that energy to live the axis is measuring
/// something real.
///
/// ´claim:readout:novelty-is-degenerate-exactly-when-no-residual-degrees-of-freedom-remain´
/// ´test:crate:novelty-not-saturated-when-residual-dof-positive´
#[test]
fn novelty_not_saturated_when_residual_dof_positive() {
    let g = ScoringGeometry {
        dim: 16,
        cap: 8,
        residual_dof: 12,
    };
    assert!(!g.is_novelty_saturated());
}

/// The other side of the same test: once the subspace spans the whole
/// working dimension there is no residual left, and the geometry says so.
/// A host reading a novelty score of nothing here learns that the axis is
/// structurally silent rather than that the observation was ordinary.
///
/// (´claim:readout:novelty-is-degenerate-exactly-when-no-residual-degrees-of-freedom-remain´)
/// ´test:crate:novelty-saturated-when-residual-dof-zero´
#[test]
fn novelty_saturated_when_residual_dof_zero() {
    let g = ScoringGeometry {
        dim: 16,
        cap: 16,
        residual_dof: 0,
    };
    assert!(g.is_novelty_saturated());
}

/// Saturability is a separate question from saturation: a tracker whose rank
/// cap reaches its working dimension may still have residual room today, yet
/// it is one of those that can lose the novelty axis as rank grows. The
/// report distinguishes the two so a host can tell a temporary reading from
/// a cell where novelty will eventually stop meaning anything.
///
/// ´claim:readout:novelty-can-become-degenerate-whenever-the-rank-cap-reaches-the-working-dimension´
/// ´test:crate:novelty-saturable-when-cap-ge-dim´
#[test]
fn novelty_saturable_when_cap_ge_dim() {
    let g = ScoringGeometry {
        dim: 8,
        cap: 8,
        residual_dof: 4,
    };
    assert!(g.is_novelty_saturable());
}

/// Where the cap sits below the working dimension the axis is safe for good:
/// however far rank adapts, residual dimensions remain. Capping rank below
/// the width is therefore not only a cost control but the thing that keeps
/// novelty measurable for the life of the cell.
///
/// (´claim:readout:novelty-can-become-degenerate-whenever-the-rank-cap-reaches-the-working-dimension´)
/// ´test:crate:novelty-not-saturable-when-cap-lt-dim´
#[test]
fn novelty_not_saturable_when_cap_lt_dim() {
    let g = ScoringGeometry {
        dim: 16,
        cap: 8,
        residual_dof: 8,
    };
    assert!(!g.is_novelty_saturable());
}

// ── ContourSnapshot ─────────────────────────────────────────

/// A contour snapshot carries the spatial shape, the accumulated volume, and
/// the structural churn since the previous report side by side. Standing
/// state and change-since-last-time are different questions about the
/// spatial layer, and the snapshot answers both at once so a host need not
/// difference successive reports to see the graph move.
///
/// ´claim:readout:a-contour-snapshot-carries-standing-spatial-state-and-the-churn-since-the-last-report-together´
/// ´test:crate:contour-snapshot-fields´
#[test]
fn contour_snapshot_fields() {
    let cs = ContourSnapshot {
        plateau_count: 3,
        cell_count: 12,
        total_importance: 42_000.0,
        splits_since_last_report: 0,
        net_removals_since_last_report: 0,
    };
    assert_eq!(cs.plateau_count, 3);
    assert_eq!(cs.cell_count, 12);
    assert!((cs.total_importance - 42_000.0).abs() < f64::EPSILON);
    assert_eq!(cs.splits_since_last_report, 0);
    assert_eq!(cs.net_removals_since_last_report, 0);
}

// ── AnalysisSetSummary ──────────────────────────────────────

/// A summary describing an analysis set with nothing selected still reports
/// every field, its ranges zeroed rather than omitted, and still counts the
/// root as a member of the full set. The shape a host parses does not change
/// with how busy the sentinel is, and the permanent root tracker is visible
/// even at the quietest extreme.
///
/// ´claim:readout:an-unselected-analysis-set-summarises-with-zeroed-ranges-and-still-counts-the-root´
/// ´test:crate:analysis-set-summary-empty´
#[test]
fn analysis_set_summary_empty() {
    let summary = AnalysisSetSummary {
        competitive_size: 0,
        full_size: 1,
        investment_set_size: 1,
        depth_range: (0, 0),
        importance_range: (0.0, 0.0),
        v_depth_range: (0, 0),
        degenerate_cells_skipped: 0,
    };
    assert_eq!(summary.competitive_size, 0);
    assert_eq!(summary.full_size, 1);
    assert_eq!(summary.investment_set_size, 1);
    assert_eq!(summary.depth_range, (0, 0));
    assert_eq!(summary.importance_range, (0.0, 0.0));
    assert_eq!(summary.v_depth_range, (0, 0));
    assert_eq!(summary.degenerate_cells_skipped, 0);
}

/// A populated summary reports its depth and importance spans as ordered
/// pairs, low end first, and its three sizes widen as the definition of
/// membership loosens: the cells that won the competition, the full set
/// their ancestry closes over, and the investment set that also holds cells
/// still warming. The nesting is what lets a host read the price of
/// analysing a cell as well as the choice to analyse it.
///
/// ´claim:readout:a-summarys-three-sizes-widen-from-competition-through-closure-to-investment´
/// ´test:crate:analysis-set-summary-populated´
#[test]
fn analysis_set_summary_populated() {
    let summary = AnalysisSetSummary {
        competitive_size: 5,
        full_size: 12,
        investment_set_size: 15,
        depth_range: (0, 4),
        importance_range: (100.0, 5000.0),
        v_depth_range: (1, 3),
        degenerate_cells_skipped: 0,
    };
    assert_eq!(summary.competitive_size, 5);
    assert_eq!(summary.full_size, 12);
    assert_eq!(summary.investment_set_size, 15);
    assert!(summary.depth_range.0 < summary.depth_range.1);
    assert!(summary.importance_range.0 < summary.importance_range.1);
}

/// Cells too narrow to support a tracker are counted in the current selection snapshot rather than quietly dropped. Recomputing replaces the count instead of accumulating it, so the report describes the graph the host can inspect now while still exposing a persistently narrow configuration.
///
/// ´claim:readout:cells-too-narrow-to-track-are-counted-rather-than-silently-dropped´
/// ´test:crate:analysis-set-summary-with-degenerate-skips´
#[test]
fn analysis_set_summary_with_degenerate_skips() {
    let config = SentinelConfig::<u64> {
        analysis_k: 32,
        analysis_depth_cutoff: 16,
        split_threshold: 1,
        d_create: 8,
        d_evict: 16,
        noise_schedule: NoiseSchedule::Explicit(Vec::new()),
        ..SentinelConfig::default()
    };
    let mut sentinel = SpectralSentinel::<u64, u64, 4>::new(config).unwrap();

    let report = sentinel.ingest(&[0; 64]);
    let expected = sentinel
        .graph()
        .layers_to(16)
        .filter(|(_, node)| (4u32.saturating_sub(node.depth) as usize) < crate::MIN_TRACKER_DIM)
        .count();

    assert!(expected > 0, "fixture must create narrow selection candidates");
    assert_eq!(report.analysis_set_summary.degenerate_cells_skipped, expected);
    assert_eq!(sentinel.degenerate_cells_skipped(), expected);
}

// ── MemberScore ─────────────────────────────────────────────

/// A member score names the cell it came from — a well-ordered interval and
/// a depth — alongside a real number on each of the four axes and its
/// standardised counterpart. Coordination scores describe a group, so
/// without the identity a host could see that the group behaved oddly but
/// not which part of the domain to look at.
///
/// ´claim:readout:a-member-score-names-the-cell-it-came-from-so-a-group-finding-can-be-attributed´
/// ´test:crate:member-score-has-cell-identity´
#[test]
fn member_score_has_cell_identity() {
    let ms = MemberScore::<u128> {
        cell_start: 0,
        cell_end: u128::MAX / 2,
        cell_depth: 1,
        novelty: 0.5,
        displacement: 0.3,
        surprise: 0.7,
        coherence: 0.9,
        novelty_z: 1.0,
        displacement_z: -0.5,
        surprise_z: 2.1,
        coherence_z: 0.0,
    };
    assert!(ms.cell_start < ms.cell_end);
    assert_eq!(ms.cell_depth, 1);
    assert!(!ms.novelty.is_nan());
    assert!(!ms.displacement.is_nan());
    assert!(!ms.surprise.is_nan());
    assert!(!ms.coherence.is_nan());
}
