// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cusum_empty_chain_returns_zeros`] | extract | A chain with no entries extracts as twelve zeros rather than as a failure or a partly-filled vector. Extraction is total: the caller that forgot to check whether a report exists still gets a well-formed feature block, and one absent chain cannot poison the row it sits in. |
//! | [`cusum_single_entry_chain`] | extract | When the chain holds a single entry the three views collapse onto it: deepest cell, root and per-axis maximum all report that one entry's value. The views are positions along a chain, not distinct measurements, so a chain of length one is not a degenerate case needing its own handling — it is the general rule with nowhere else to look. |
//! | [`golden_report_chain_cusums`] | extract | The twelve features are three views of the same four axes, and each view reads the entry it names: the cell view takes the deepest entry, the root view the shallowest, and the max view the largest value per axis across the whole chain. Checked against a hand-computed report where the axes differ from each other at every depth except the root's accumulators, the layout is pinned against index-slot transposition — a caller reading index four is reading the root's novelty and nothing else. |
//! | [`cusums_max_across_chain`] | extract | The maximum view is a genuine per-axis maximum over every depth, not a choice between the two endpoints: when an interior depth is the loudest, that interior value is what the max view reports while the cell view keeps its own. Drift that peaks midway up the ancestry is therefore visible to the model instead of being hidden by quieter ends. |
//! | [`cusums_all_zero`] | extract | A chain that is present but everywhere quiet extracts as twelve zeros: no view manufactures magnitude the entries did not carry. In particular the max view's fold over an all-zero chain settles at zero rather than at the negative infinity it starts from, so a calm ancestry is indistinguishable from calm rather than from a sentinel value. |

//! Chain CUSUM feature extraction.
//!
//! Extracts 12 CUSUM features from the ancestor chain: 3 views × 4 axes.
//!
//! # Feature Layout
//!
//! | Index Range | View | Description |
//! |-------------|------|-------------|
//! | 0–3 | Cell | Deepest cell's CUSUM values (N, D, S, C) |
//! | 4–7 | Root | Root cell's CUSUM values |
//! | 8–11 | Max | Maximum CUSUM across chain per axis |
//!
//! # Cross-References
//!
//! - (´tab:extraction:chain-cusums´) — the three accumulator views this
//!   module lays out across the four scoring axes

// Each axis is used to compute multiple indices; this is clearer than enumerate.

use crate::report::ReportCellEntry;
use crate::types::SCORING_AXIS_COUNT;

// ═══════════════════════════════════════════════════════════════════════════════
// Chain CUSUM Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts 12 CUSUM features from an ancestor chain.
///
/// # Arguments
///
/// * `chain` — Ancestor chain entries, deepest first. Must be non-empty.
///
/// # Returns
///
/// An array of 12 f64 values representing the 3 views × 4 axes.
///
/// # Degenerate Cases
///
/// - Chain length 1: root = cell, max = cell.
/// - Empty chain: all zeros (caller should check `has_report` first).
///
/// # Cross-References
///
/// - (´tab:extraction:chain-cusums´) — the twelve accumulator features
#[must_use]
pub fn extract_chain_cusums(chain: &[(u8, &ReportCellEntry)]) -> [f64; 12] {
    let mut features = [0.0f64; 12];

    if chain.is_empty() {
        return features;
    }

    // Each entry's axes read once, through the report's own projection, so the
    // views below index an arity-wide row rather than re-spelling the axis order.
    let per_entry: Vec<[f64; SCORING_AXIS_COUNT]> = chain.iter().map(|(_, e)| e.scores.cusum_per_axis()).collect();

    // Cell view (indices 0–3): deepest cell (chain[0])
    let cell = per_entry[0];
    features[..SCORING_AXIS_COUNT].copy_from_slice(&cell);

    // Root view (indices 4–7): shallowest cell (chain.last())
    let root = *per_entry.last().unwrap_or(&cell);
    features[SCORING_AXIS_COUNT..2 * SCORING_AXIS_COUNT].copy_from_slice(&root);

    // Max view (indices 8–11): maximum across chain per axis
    for axis in 0..SCORING_AXIS_COUNT {
        let max_val = per_entry.iter().map(|row| row[axis]).fold(f64::NEG_INFINITY, f64::max);
        features[2 * SCORING_AXIS_COUNT + axis] = if max_val.is_finite() { max_val } else { 0.0 };
    }

    features
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::items_after_statements)]
    #![allow(clippy::uninlined_format_args)]

    use super::*;
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry};

    /// Creates a test entry with specified CUSUM values for each axis.
    fn make_entry_cusum(depth: u8, n: f64, d: f64, s: f64, c: f64) -> ReportCellEntry {
        ReportCellEntry {
            depth,
            sample_count: 100,
            is_competitive: true,
            rank: 1,
            cap: 10,
            energy_ratio: 1.0,
            noise_influence: 0.0,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    cusum: n,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    cusum: d,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    cusum: s,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    cusum: c,
                    ..Default::default()
                },
            },
            degraded: false,
        }
    }

    /// A chain with no entries extracts as twelve zeros rather than as a
    /// failure or a partly-filled vector. Extraction is total: the caller that
    /// forgot to check whether a report exists still gets a well-formed feature
    /// block, and one absent chain cannot poison the row it sits in.
    ///
    /// ´claim:extract:an-empty-ancestor-chain-extracts-as-twelve-zero-features´
    /// ´test:unit:cusum-empty-chain-returns-zeros´
    #[test]
    fn cusum_empty_chain_returns_zeros() {
        let chain: Vec<(u8, &ReportCellEntry)> = vec![];
        let features = extract_chain_cusums(&chain);
        assert!(features.iter().all(|&v| v == 0.0));
    }

    /// When the chain holds a single entry the three views collapse onto it:
    /// deepest cell, root and per-axis maximum all report that one entry's
    /// value. The views are positions along a chain, not distinct measurements,
    /// so a chain of length one is not a degenerate case needing its own
    /// handling — it is the general rule with nowhere else to look.
    ///
    /// ´claim:extract:a-single-entry-chain-makes-the-three-views-coincide´
    /// ´test:unit:cusum-single-entry-chain´
    #[test]
    fn cusum_single_entry_chain() {
        let entry = make_entry_cusum(8, 0.7, 0.4, 0.5, 0.3);
        let chain = vec![(8, &entry)];
        let features = extract_chain_cusums(&chain);

        // Cell and root are the same
        assert!((features[0] - 0.7).abs() < 1e-14, "Cell N CUSUM");
        assert!((features[4] - 0.7).abs() < 1e-14, "Root N CUSUM");
        assert!((features[8] - 0.7).abs() < 1e-14, "Max N CUSUM");
    }

    /// The twelve features are three views of the same four axes, and each view
    /// reads the entry it names: the cell view takes the deepest entry, the root
    /// view the shallowest, and the max view the largest value per axis across
    /// the whole chain. Checked against a hand-computed report where the axes
    /// differ from each other at every depth except the root's accumulators,
    /// the layout is pinned against index-slot transposition — a caller reading
    /// index four is reading the root's novelty and nothing else.
    ///
    /// ´claim:extract:the-three-chain-views-read-deepest-shallowest-and-per-axis-maximum´
    /// ´test:unit:golden-report-chain-cusums´
    #[test]
    fn golden_report_chain_cusums() {
        use crate::testing::scores::{D4_CUSUM, D8_CUSUM, ROOT_CUSUM};

        // Golden report figures from (´tab:assayer:harness-golden-figures´)
        //
        // | Axis | Root (d=0) CUSUM | d=4 CUSUM | Cell (d=8) CUSUM |
        // |------|------------------|-----------|------------------|
        // | N    | 0.1              | 0.3       | 0.7              |
        // | D    | 0.0              | 0.2       | 0.4              |
        // | S    | 0.5              | 0.5       | 0.5              |
        // | C    | 0.0              | 0.1       | 0.3              |

        let root = make_entry_cusum(0, ROOT_CUSUM[0], ROOT_CUSUM[1], ROOT_CUSUM[2], ROOT_CUSUM[3]);
        let mid = make_entry_cusum(4, D4_CUSUM[0], D4_CUSUM[1], D4_CUSUM[2], D4_CUSUM[3]);
        let cell = make_entry_cusum(8, D8_CUSUM[0], D8_CUSUM[1], D8_CUSUM[2], D8_CUSUM[3]);

        // Chain is deepest-first: cell, mid, root
        let chain = vec![(8, &cell), (4, &mid), (0, &root)];
        let features = extract_chain_cusums(&chain);

        const TOL: f64 = 1e-14;

        // Cell view (indices 0–3)
        assert!((features[0] - D8_CUSUM[0]).abs() < TOL, "Cell N CUSUM");
        assert!((features[1] - D8_CUSUM[1]).abs() < TOL, "Cell D CUSUM");
        assert!((features[2] - D8_CUSUM[2]).abs() < TOL, "Cell S CUSUM");
        assert!((features[3] - D8_CUSUM[3]).abs() < TOL, "Cell C CUSUM");

        // Root view (indices 4–7)
        assert!((features[4] - ROOT_CUSUM[0]).abs() < TOL, "Root N CUSUM");
        assert!((features[5] - ROOT_CUSUM[1]).abs() < TOL, "Root D CUSUM");
        assert!((features[6] - ROOT_CUSUM[2]).abs() < TOL, "Root S CUSUM");
        assert!((features[7] - ROOT_CUSUM[3]).abs() < TOL, "Root C CUSUM");

        // Max view (indices 8–11): the fixture's monotone ladder means the
        // deepest cell already carries the per-axis maximum (see
        // `cusums_max_across_chain` for a fixture where an interior depth wins).
        assert!((features[8] - D8_CUSUM[0]).abs() < TOL, "Max N CUSUM");
        assert!((features[9] - D8_CUSUM[1]).abs() < TOL, "Max D CUSUM");
        assert!((features[10] - D8_CUSUM[2]).abs() < TOL, "Max S CUSUM");
        assert!((features[11] - D8_CUSUM[3]).abs() < TOL, "Max C CUSUM");
    }

    /// The maximum view is a genuine per-axis maximum over every depth, not a
    /// choice between the two endpoints: when an interior depth is the loudest,
    /// that interior value is what the max view reports while the cell view
    /// keeps its own. Drift that peaks midway up the ancestry is therefore
    /// visible to the model instead of being hidden by quieter ends.
    ///
    /// ´claim:extract:the-maximum-view-can-be-carried-by-an-interior-depth´
    /// ´test:unit:cusums-max-across-chain´
    #[test]
    fn cusums_max_across_chain() {
        // Test that max picks the loudest depth, not necessarily cell or root
        // CUSUMs: root=0.1, d=4=0.5, cell=0.3 → Max = 0.5 (d=4 is loudest)
        let root = make_entry_cusum(0, 0.1, 0.0, 0.0, 0.0);
        let mid = make_entry_cusum(4, 0.5, 0.8, 0.0, 0.0); // mid has highest N and D
        let cell = make_entry_cusum(8, 0.3, 0.2, 0.0, 0.0);

        let chain = vec![(8, &cell), (4, &mid), (0, &root)];
        let features = extract_chain_cusums(&chain);

        const TOL: f64 = 1e-14;

        // Max view should pick d=4's values for N and D
        assert!((features[8] - 0.5).abs() < TOL, "Max N CUSUM should be 0.5 from d=4");
        assert!((features[9] - 0.8).abs() < TOL, "Max D CUSUM should be 0.8 from d=4");

        // Cell view should still be from cell (d=8)
        assert!((features[0] - 0.3).abs() < TOL, "Cell N CUSUM should be 0.3");
    }

    /// A chain that is present but everywhere quiet extracts as twelve zeros:
    /// no view manufactures magnitude the entries did not carry. In particular
    /// the max view's fold over an all-zero chain settles at zero rather than at
    /// the negative infinity it starts from, so a calm ancestry is
    /// indistinguishable from calm rather than from a sentinel value.
    ///
    /// ´claim:extract:an-everywhere-quiet-chain-extracts-to-zero-in-every-view´
    /// ´test:unit:cusums-all-zero´
    #[test]
    fn cusums_all_zero() {
        // All CUSUMs = 0.0 should produce all zeros
        let root = make_entry_cusum(0, 0.0, 0.0, 0.0, 0.0);
        let cell = make_entry_cusum(8, 0.0, 0.0, 0.0, 0.0);

        let chain = vec![(8, &cell), (0, &root)];
        let features = extract_chain_cusums(&chain);

        // All 12 features should be 0.0
        for (i, &f) in features.iter().enumerate() {
            assert!(f.abs() < 1e-14, "CUSUM feature {} should be 0, got {}", i, f);
        }
    }
}
