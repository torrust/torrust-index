// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`dyadic_full_range`] | dossier | Depth zero is the whole coordinate space: the single interval running from zero to the largest representable coordinate is dyadic there. The root is therefore not a special case bolted onto the hierarchy but its first member, which is why every coordinate has somewhere to land. |
//! | [`dyadic_half_range`] | dossier | Each descent halves: at depth one both the lower and the upper half of the space qualify, and nothing between them does. The two halves partition the space exactly, so a coordinate has one ancestor per depth rather than a choice of overlapping candidates. |
//! | [`non_dyadic_misaligned`] | dossier | An interval that does not begin on a boundary of its claimed depth is not dyadic, however plausible its endpoints look on their own: starting one past zero disqualifies it at depth eight. Alignment is what makes a cell's ancestry computable by masking bits, so an unaligned interval could not be placed in the hierarchy at all. |

//! Structural validation for Sentinel batch reports.
//!
//! This module provides validation functions for incoming `BatchReport<u128>`:
//!
//! - [`validate_structural`] — Ensures the report has a root cell, dyadic
//!   intervals, and no duplicate cells.
//! - [`validate_and_extract_cell`] — Validates and extracts a single cell,
//!   sanitising NaN/Inf values.
//!
//! # Cross-References
//!
//! - (´req:publication:report-reception´) — what reception must do with a
//!   report, and in what order
//! - (´def:encoding:question-meaning´) — the dyadic structure the reported
//!   intervals are required to have
//! - (´dec:surface:reception-isolation´) — the steps the pipeline runs, and
//!   the state it does not touch

// Items in this module are re-exported via crate::report and used by tests.

use std::collections::HashSet;

use torrust_sentinel::{AnomalyScores, BatchReport, CellReport, CoordinationReport, ScoreDistribution};

use super::index::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry};
use crate::error::ReportError;
use crate::types::{CellInterval, LedgerKey, is_dyadic};

// ═══════════════════════════════════════════════════════════════════════════════
// Structural Validation
// ═══════════════════════════════════════════════════════════════════════════════

/// Validates the structural integrity of a batch report.
///
/// Performs three checks:
///
/// 1. **Root cell present** — There must be exactly one cell at depth 0
///    covering the full domain `[0, u128::MAX]`.
///
/// 2. **Dyadic intervals** — Every cell interval must be dyadic at its
///    claimed depth.
///
/// 3. **No overlapping intervals** — No two cells can have the same
///    `(lo, depth)` key.
///
/// # Arguments
///
/// * `report` — The batch report to validate
///
/// # Errors
///
/// Returns an error if any structural check fails:
///
/// - [`ReportError::MissingRootCell`] — No depth-0 cell present
/// - [`ReportError::NonDyadicInterval`] — A cell interval is not dyadic
/// - [`ReportError::DuplicateCell`] — Two cells share the same `(lo, depth)` key
///
/// # Performance
///
/// $O(n)$ where $n$ is the total number of cells. Duplicate detection
/// exploits the dyadic property: same-depth intervals either coincide
/// or are disjoint, so we only need to check `(lo, depth)` uniqueness.
pub fn validate_structural(report: &BatchReport<u128>) -> Result<(), ReportError> {
    let mut seen_keys: HashSet<LedgerKey> = HashSet::new();
    let mut has_root = false;

    // Helper to validate a single cell's interval.
    //
    // The Sentinel reports `end` as an exclusive upper bound. The depth-0 root
    // cell is the single exception: it uses the sentinel value `end == 0` to
    // encode the full domain `[0, u128::MAX]` (an exclusive end of $2^{128}$
    // cannot be represented in `u128`). For any non-root cell, `end == 0`
    // would imply a wrap-around interval, which is not representable and is
    // rejected as `NonDyadicInterval`.
    let validate_cell = |start: u128, end: u128, depth: u32, seen_keys: &mut HashSet<LedgerKey>| -> Result<(), ReportError> {
        let depth_u8 = depth_to_u8_clamped(depth);

        let hi = if depth == 0 && start == 0 && end == 0 {
            u128::MAX
        } else if end == 0 {
            // Non-root cell with sentinel `end == 0`: structurally malformed.
            // Report the cell as-observed (`hi = end`) so the diagnostic
            // reflects the input rather than a contrived fallback.
            return Err(ReportError::NonDyadicInterval {
                cell: CellInterval::new(start, end, depth_u8),
            });
        } else {
            end.saturating_sub(1)
        };

        // Validate dyadic property
        if !is_dyadic(start, hi, depth_u8) {
            return Err(ReportError::NonDyadicInterval {
                cell: CellInterval::new(start, hi, depth_u8),
            });
        }

        // Check for duplicates using LedgerKey.
        //
        // Under the dyadic invariant, same-depth intervals either coincide
        // or are disjoint (´def:encoding:question-meaning´). A `(lo, depth)`
        // key collision therefore implies two interval-identical cells — no
        // second cell need be reported.
        let key = LedgerKey::new(start, depth_u8);
        if !seen_keys.insert(key) {
            return Err(ReportError::DuplicateCell {
                cell: CellInterval::new(start, hi, depth_u8),
            });
        }

        Ok(())
    };

    // Validate competitive cells
    for cell in &report.cell_reports {
        validate_cell(cell.start, cell.end, cell.depth, &mut seen_keys)?;
        if cell.depth == 0 {
            has_root = true;
        }
    }

    // Validate ancestor cells
    for cell in &report.ancestor_reports {
        validate_cell(cell.start, cell.end, cell.depth, &mut seen_keys)?;
        if cell.depth == 0 {
            has_root = true;
        }
    }

    // Ensure root cell was found
    if !has_root {
        return Err(ReportError::MissingRootCell);
    }

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cell Extraction with Validation
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts and sanitises an [`AxisScoreSnapshot`] from a [`ScoreDistribution`].
///
/// Returns the default (all-zero) snapshot if any field is non-finite or the
/// baseline variance is negative; sets `*degraded = true` in that case.
fn extract_axis(score: &ScoreDistribution, degraded: &mut bool) -> AxisScoreSnapshot {
    let snapshot = AxisScoreSnapshot {
        max_z: score.max_z_score,
        mean_z: score.mean_z_score,
        cusum: score.cusum.accumulator,
        baseline_mean: score.baseline.mean,
        baseline_variance: score.baseline.variance,
        clip_pressure: score.clip_pressure,
    };

    let is_invalid = !snapshot.max_z.is_finite()
        || !snapshot.mean_z.is_finite()
        || !snapshot.cusum.is_finite()
        || !snapshot.baseline_mean.is_finite()
        || !snapshot.baseline_variance.is_finite()
        || !snapshot.clip_pressure.is_finite()
        || snapshot.baseline_variance < 0.0;

    if is_invalid {
        *degraded = true;
        AxisScoreSnapshot::default()
    } else {
        snapshot
    }
}

/// Extracts and sanitises all four anomaly axes into an [`AxisScoreSet`].
///
/// Delegates to [`extract_axis`] per axis. Sets `*degraded = true` if any
/// individual axis was sanitised.
fn extract_all_axes(scores: &AnomalyScores, degraded: &mut bool) -> AxisScoreSet {
    AxisScoreSet {
        novelty: extract_axis(&scores.novelty, degraded),
        displacement: extract_axis(&scores.displacement, degraded),
        surprise: extract_axis(&scores.surprise, degraded),
        coherence: extract_axis(&scores.coherence, degraded),
    }
}

/// Validates and extracts a single cell from a `CellReport<u128>`.
///
/// Performs data quality validation and sanitisation:
///
/// - **NaN/Inf in score fields** → Zero the affected axis
/// - **Negative baseline variance** → Zero the affected axis
/// - **`noise_influence` outside \[0, 1\]** → Clamp to bounds
/// - **`depth > 128`** → Clamp to 128
///
/// # Arguments
///
/// * `cell` — The cell report to validate and extract
///
/// # Returns
///
/// A tuple containing:
/// - The extracted `ReportCellEntry` with sanitised values
/// - A `degraded` flag indicating if any value was sanitised — the
///   depth clamp included, so the flag reports every sanitisation this
///   function performs (´dec:degradation:error-partition´)
#[must_use]
pub fn validate_and_extract_cell(cell: &CellReport<u128>) -> (ReportCellEntry, bool) {
    let mut degraded = false;

    let scores = extract_all_axes(&cell.scores, &mut degraded);

    // Sanitise noise_influence: clamp to [0, 1]
    let noise_influence = if !cell.maturity.noise_influence.is_finite() || cell.maturity.noise_influence < 0.0 {
        degraded = true;
        0.0
    } else if cell.maturity.noise_influence > 1.0 {
        degraded = true;
        1.0
    } else {
        cell.maturity.noise_influence
    };

    // Handle depth overflow: the clamp is a sanitisation like the
    // other three, and the flag reports it.
    if cell.depth > MAX_REPORT_DEPTH {
        degraded = true;
    }
    let depth = depth_to_u8_clamped(cell.depth);

    // Sanitise energy_ratio
    let energy_ratio = if cell.energy_ratio.is_finite() {
        cell.energy_ratio
    } else {
        degraded = true;
        0.0
    };

    let entry = ReportCellEntry {
        depth,
        sample_count: cell.sample_count,
        is_competitive: cell.is_competitive,
        rank: cell.rank,
        cap: cell.geometry.cap,
        energy_ratio,
        noise_influence,
        scores,
        degraded,
    };

    (entry, degraded)
}

/// Validates and extracts a coordination entry from a `CoordinationReport<u128>`.
///
/// Performs the same data quality validation as [`validate_and_extract_cell`].
///
/// # Arguments
///
/// * `coord` — The coordination report to validate and extract
///
/// # Returns
///
/// A tuple containing:
/// - The extracted `CoordinationEntry` with sanitised values
/// - A `degraded` flag indicating if any value was sanitised
#[must_use]
pub fn validate_and_extract_coordination(coord: &CoordinationReport<u128>) -> (super::index::CoordinationEntry, bool) {
    let mut degraded = false;

    let scores = extract_all_axes(&coord.scores, &mut degraded);

    // Handle depth overflow: reported through the flag, as in
    // `validate_and_extract_cell`.
    if coord.depth > MAX_REPORT_DEPTH {
        degraded = true;
    }
    let depth = depth_to_u8_clamped(coord.depth);

    let entry = super::index::CoordinationEntry {
        depth,
        cells_reporting: coord.cells_reporting,
        scores,
    };

    (entry, degraded)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Utility Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum representable cell depth in a 128-bit coordinate domain.
///
/// The ceiling is the Core's internal coordinate width, which is fixed
/// at that many bits for every per-Sentinel spatial structure and every
/// internal routing path (´def:encoding:domain-width´).
///
/// ´const:assayer:coordinate-depth-ceiling´ (´alg:const:count´)
/// ´const:assayer:coordinate-depth-ceiling-count-128´
pub(super) const MAX_REPORT_DEPTH: u32 = 128;

/// Converts a depth value to u8, clamping to `MAX_REPORT_DEPTH`.
///
/// The extraction call sites report the clamp through their degraded
/// flag; the clamp itself is uniform across build profiles rather than
/// an assertion, because an out-of-contract report is a data-quality
/// input this module sanitises and reports, never a crash
/// (´dec:degradation:error-partition´).
///
/// # Safety
///
/// The explicit check before the cast ensures no truncation occurs —
/// values above the maximum are clamped to it, and values at or below
/// fit in u8.
#[allow(clippy::cast_possible_truncation)]
pub(super) const fn depth_to_u8_clamped(depth: u32) -> u8 {
    if depth > MAX_REPORT_DEPTH { 128 } else { depth as u8 }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Depth zero is the whole coordinate space: the single interval running from
    /// zero to the largest representable coordinate is dyadic there. The root is
    /// therefore not a special case bolted onto the hierarchy but its first
    /// member, which is why every coordinate has somewhere to land.
    ///
    /// ´claim:dossier:the-whole-coordinate-space-is-the-one-dyadic-cell-at-depth-zero´
    /// ´test:unit:dyadic-full-range´
    #[test]
    fn dyadic_full_range() {
        // Depth 0 should cover [0, u128::MAX]
        assert!(is_dyadic(0, u128::MAX, 0));
    }

    /// Each descent halves: at depth one both the lower and the upper half of the
    /// space qualify, and nothing between them does. The two halves partition the
    /// space exactly, so a coordinate has one ancestor per depth rather than a
    /// choice of overlapping candidates.
    ///
    /// ´claim:dossier:each-depth-halves-the-space-into-two-cells-that-partition-it-exactly´
    /// ´test:unit:dyadic-half-range´
    #[test]
    fn dyadic_half_range() {
        // Depth 1: two halves of the u128 space
        let half = 1_u128 << 127;
        // First half: [0, half - 1]
        assert!(is_dyadic(0, half - 1, 1));
        // Second half: [half, u128::MAX]
        assert!(is_dyadic(half, u128::MAX, 1));
    }

    /// An interval that does not begin on a boundary of its claimed depth is not
    /// dyadic, however plausible its endpoints look on their own: starting one
    /// past zero disqualifies it at depth eight. Alignment is what makes a cell's
    /// ancestry computable by masking bits, so an unaligned interval could not be
    /// placed in the hierarchy at all.
    ///
    /// ´claim:dossier:an-interval-not-aligned-to-its-claimed-depth-is-not-dyadic´
    /// ´test:unit:non-dyadic-misaligned´
    #[test]
    fn non_dyadic_misaligned() {
        // An interval that doesn't start at a valid dyadic boundary
        assert!(!is_dyadic(1, 100, 8));
    }
}
