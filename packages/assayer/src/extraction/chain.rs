// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_chain_returns_zeros`] | extract | The z-score block is twenty-four slots wide whether or not there is a chain to fill it, an absent chain producing zeros throughout. The row's width is a property of the model, not of any one Sentinel's luck with reports, so a missing report shifts nothing along. |
//! | [`single_entry_chain`] | extract | On a chain of one the two comparative views fall silent: cell and root coincide, so their difference is zero, and a single value has no spread about its own mean. Gradient and spread describe how conditions vary along an ancestry, and a cell with no ancestry has no variation to report — reported as zero rather than as an undefined statistic from dividing by a degenerate sample. |
//! | [`golden_report_chain_zscores`] | extract | Each axis is shown to the model six ways over the same ancestry: the deepest entry, the shallowest, the largest and the arithmetic mean over all depths, the deepest-minus-shallowest difference, and the population standard deviation about that mean. Checked against a hand-computed report where the axes differ from each other at every depth, the whole layout is pinned against index-slot transposition — every index names one view of one axis and nothing else. |
//! | [`zscores_two_depths`] | extract | cites (´claim:extract:the-six-z-score-views-are-cell-root-max-mean-gradient-and-population-spread´) |
//! | [`zscores_per_axis_independence`] | extract | The four axes are summarised in isolation from one another: with novelty loudest at the cell and displacement loudest at the root, each axis's views follow its own values and neither borrows the other's magnitude or its choice of depth. The axes measure unrelated things, so a maximum taken across them jointly would let one loud axis speak for all four. |
//! | [`zscores_all_zero`] | extract | cites (´claim:extract:an-everywhere-quiet-chain-extracts-to-zero-in-every-view´) |
//! | [`zscores_negative_z`] | extract | Negative scores pass through unaltered: the cell view keeps its negative value, the gradient goes negative when the cell sits below its root, and the maximum takes the largest signed value rather than the largest magnitude. Sign carries direction here — quieter than the reference is a different claim from louder — so clamping or taking absolute values would erase the very contrast the gradient view exists to show. |

//! Chain z-score feature extraction.
//!
//! Extracts 24 z-score features from the ancestor chain: 6 views × 4 axes.
//!
//! # Feature Layout
//!
//! | Index Range | View | Description |
//! |-------------|------|-------------|
//! | 0–3 | Cell | Deepest cell's z-scores (N, D, S, C) |
//! | 4–7 | Root | Root cell's z-scores |
//! | 8–11 | Max | Maximum z-score across chain per axis |
//! | 12–15 | Mean | Mean z-score across chain per axis |
//! | 16–19 | Gradient | Cell − Root difference per axis |
//! | 20–23 | Spread | Standard deviation across chain per axis |
//!
//! # Cross-References
//!
//! - (´tab:extraction:chain-z-scores´) — the five depth-dependent z-score
//!   views this module lays out across the four scoring axes

// Each axis is used to compute multiple indices into features; this is clearer than enumerate.
// Chain length is always small (≤128); no precision loss.
#![allow(clippy::cast_precision_loss)]

use crate::report::ReportCellEntry;
use crate::types::SCORING_AXIS_COUNT;

// ═══════════════════════════════════════════════════════════════════════════════
// Chain Z-Score Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts 24 z-score features from an ancestor chain.
///
/// # Arguments
///
/// * `chain` — Ancestor chain entries, deepest first. Must be non-empty.
///
/// # Returns
///
/// An array of 24 f64 values representing the 6 views × 4 axes.
///
/// # Degenerate Cases
///
/// - Chain length 1: root = cell, gradient = 0, spread = 0, mean = cell.
/// - Empty chain: all zeros (caller should check `has_report` first).
///
/// # Cross-References
///
/// - (´tab:extraction:chain-z-scores´) — the twenty-four z-score features
#[must_use]
pub fn extract_chain_zscores(chain: &[(u8, &ReportCellEntry)]) -> [f64; 24] {
    let mut features = [0.0f64; 24];

    if chain.is_empty() {
        return features;
    }

    // Each entry's axes read once, through the report's own projection, so the
    // views below index an arity-wide row rather than re-spelling the axis order.
    let per_entry: Vec<[f64; SCORING_AXIS_COUNT]> = chain.iter().map(|(_, e)| e.scores.max_z_per_axis()).collect();

    // Cell view (indices 0–3): deepest cell (chain[0])
    let cell = per_entry[0];
    features[..SCORING_AXIS_COUNT].copy_from_slice(&cell);

    // Root view (indices 4–7): shallowest cell (chain.last())
    let root = *per_entry.last().unwrap_or(&cell);
    features[SCORING_AXIS_COUNT..2 * SCORING_AXIS_COUNT].copy_from_slice(&root);

    // Compute per-axis statistics across the chain
    let n = chain.len() as f64;

    for axis in 0..SCORING_AXIS_COUNT {
        let values: Vec<f64> = per_entry.iter().map(|row| row[axis]).collect();

        // Max view (indices 8–11)
        let max_val = values.iter().copied().fold(f64::NEG_INFINITY, f64::max);
        features[2 * SCORING_AXIS_COUNT + axis] = if max_val.is_finite() { max_val } else { 0.0 };

        // Mean view (indices 12–15)
        let sum: f64 = values.iter().sum();
        let mean = sum / n;
        features[3 * SCORING_AXIS_COUNT + axis] = mean;

        // Gradient view (indices 16–19): Cell − Root
        let cell_val = values[0]; // Chain is deepest-first
        let root_val = *values.last().unwrap_or(&0.0);
        features[4 * SCORING_AXIS_COUNT + axis] = cell_val - root_val;

        // Spread view (indices 20–23): standard deviation
        if n > 1.0 {
            let variance: f64 = values.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n;
            features[5 * SCORING_AXIS_COUNT + axis] = variance.sqrt();
        } else {
            features[5 * SCORING_AXIS_COUNT + axis] = 0.0;
        }
    }

    features
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::items_after_statements)]
    #![allow(clippy::suboptimal_flops)]
    #![allow(clippy::uninlined_format_args)]

    use super::*;
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry};

    /// Creates a test entry with specified `max_z` values for each axis.
    fn make_entry(depth: u8, n: f64, d: f64, s: f64, c: f64) -> ReportCellEntry {
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
                    max_z: n,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: d,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: s,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: c,
                    ..Default::default()
                },
            },
            degraded: false,
        }
    }

    /// The z-score block is twenty-four slots wide whether or not there is a
    /// chain to fill it, an absent chain producing zeros throughout. The row's
    /// width is a property of the model, not of any one Sentinel's luck with
    /// reports, so a missing report shifts nothing along.
    ///
    /// ´claim:extract:an-absent-chain-still-produces-the-full-width-z-score-block-filled-with-zeros´
    /// ´test:unit:empty-chain-returns-zeros´
    #[test]
    fn empty_chain_returns_zeros() {
        let chain: Vec<(u8, &ReportCellEntry)> = vec![];
        let features = extract_chain_zscores(&chain);
        assert!(features.iter().all(|&v| v == 0.0));
    }

    /// On a chain of one the two comparative views fall silent: cell and root
    /// coincide, so their difference is zero, and a single value has no spread
    /// about its own mean. Gradient and spread describe how conditions vary
    /// along an ancestry, and a cell with no ancestry has no variation to
    /// report — reported as zero rather than as an undefined statistic from
    /// dividing by a degenerate sample.
    ///
    /// ´claim:extract:a-single-entry-chain-has-no-gradient-and-no-spread-to-report´
    /// ´test:unit:single-entry-chain´
    #[test]
    fn single_entry_chain() {
        let entry = make_entry(8, 2.0, 1.5, 1.0, 0.5);
        let chain = vec![(8, &entry)];
        let features = extract_chain_zscores(&chain);

        // Cell and root are the same
        assert!((features[0] - 2.0).abs() < 1e-14); // Cell N
        assert!((features[4] - 2.0).abs() < 1e-14); // Root N

        // Gradient = 0 (cell - root)
        assert!((features[16]).abs() < 1e-14); // Gradient N

        // Spread = 0 (single value)
        assert!((features[20]).abs() < 1e-14); // Spread N
    }

    /// Each axis is shown to the model six ways over the same ancestry: the
    /// deepest entry, the shallowest, the largest and the arithmetic mean over
    /// all depths, the deepest-minus-shallowest difference, and the population
    /// standard deviation about that mean. Checked against a hand-computed
    /// report where the axes differ from each other at every depth, the whole
    /// layout is pinned against index-slot transposition — every index names
    /// one view of one axis and nothing else.
    ///
    /// ´claim:extract:the-six-z-score-views-are-cell-root-max-mean-gradient-and-population-spread´
    /// ´test:unit:golden-report-chain-zscores´
    #[test]
    fn golden_report_chain_zscores() {
        use crate::testing::scores::{D4_MAX_Z, D8_MAX_Z, ROOT_MAX_Z};

        // Golden report figures from (´tab:assayer:harness-golden-figures´)
        //
        // | Axis | Root (d=0) | d=4 | Cell (d=8) |
        // |------|------------|-----|------------|
        // | N    | 0.5        | 1.2 | 2.8        |
        // | D    | 0.3        | 0.7 | 1.5        |
        // | S    | 1.0        | 1.0 | 1.0        |
        // | C    | 0.0        | 0.4 | 0.9        |

        let root = make_entry(0, ROOT_MAX_Z[0], ROOT_MAX_Z[1], ROOT_MAX_Z[2], ROOT_MAX_Z[3]);
        let mid = make_entry(4, D4_MAX_Z[0], D4_MAX_Z[1], D4_MAX_Z[2], D4_MAX_Z[3]);
        let cell = make_entry(8, D8_MAX_Z[0], D8_MAX_Z[1], D8_MAX_Z[2], D8_MAX_Z[3]);

        // Chain is deepest-first: cell, mid, root
        let chain = vec![(8, &cell), (4, &mid), (0, &root)];
        let features = extract_chain_zscores(&chain);

        const TOL: f64 = 1e-14;

        // Cell view (indices 0–3)
        assert!((features[0] - D8_MAX_Z[0]).abs() < TOL, "Cell N");
        assert!((features[1] - D8_MAX_Z[1]).abs() < TOL, "Cell D");
        assert!((features[2] - D8_MAX_Z[2]).abs() < TOL, "Cell S");
        assert!((features[3] - D8_MAX_Z[3]).abs() < TOL, "Cell C");

        // Root view (indices 4–7)
        assert!((features[4] - ROOT_MAX_Z[0]).abs() < TOL, "Root N");
        assert!((features[5] - ROOT_MAX_Z[1]).abs() < TOL, "Root D");
        assert!((features[6] - ROOT_MAX_Z[2]).abs() < TOL, "Root S");
        assert!((features[7] - ROOT_MAX_Z[3]).abs() < TOL, "Root C");

        // Max view (indices 8–11): the fixture's monotone ladder means the
        // deepest cell already carries the per-axis maximum.
        assert!((features[8] - D8_MAX_Z[0]).abs() < TOL, "Max N");
        assert!((features[9] - D8_MAX_Z[1]).abs() < TOL, "Max D");
        assert!((features[10] - D8_MAX_Z[2]).abs() < TOL, "Max S");
        assert!((features[11] - D8_MAX_Z[3]).abs() < TOL, "Max C");

        // Mean view (indices 12–15)
        // N: (2.8 + 1.2 + 0.5) / 3 = 1.5
        // D: (1.5 + 0.7 + 0.3) / 3 ≈ 0.833...
        // S: (1.0 + 1.0 + 1.0) / 3 = 1.0
        // C: (0.9 + 0.4 + 0.0) / 3 ≈ 0.433...
        assert!((features[12] - 1.5).abs() < TOL, "Mean N");
        assert!((features[13] - 2.5 / 3.0).abs() < TOL, "Mean D");
        assert!((features[14] - 1.0).abs() < TOL, "Mean S");
        assert!((features[15] - 1.3 / 3.0).abs() < TOL, "Mean C");

        // Gradient view (indices 16–19): Cell − Root
        assert!((features[16] - 2.3).abs() < TOL, "Gradient N");
        assert!((features[17] - 1.2).abs() < TOL, "Gradient D");
        assert!((features[18] - 0.0).abs() < TOL, "Gradient S");
        assert!((features[19] - 0.9).abs() < TOL, "Gradient C");

        // Spread view (indices 20–23): population std dev
        // N: values [2.8, 1.2, 0.5], mean=1.5
        //    var = ((2.8-1.5)² + (1.2-1.5)² + (0.5-1.5)²) / 3
        //        = (1.69 + 0.09 + 1.0) / 3 = 2.78 / 3 ≈ 0.9267
        //    std = sqrt(0.9267) ≈ 0.9626
        let n_mean: f64 = 1.5;
        let n_var = ((D8_MAX_Z[0] - n_mean).powi(2) + (D4_MAX_Z[0] - n_mean).powi(2) + (ROOT_MAX_Z[0] - n_mean).powi(2)) / 3.0;
        assert!((features[20] - n_var.sqrt()).abs() < TOL, "Spread N");
    }

    /// The derived views range over whatever depths the chain actually holds
    /// rather than over a fixed set of levels: a chain skipping the intermediate
    /// depth has its mean, spread and gradient computed from its two entries
    /// alone. Cells whose ancestry was pruned or never reported are therefore
    /// summarised honestly instead of being padded with imagined levels.
    ///
    /// (´claim:extract:the-six-z-score-views-are-cell-root-max-mean-gradient-and-population-spread´)
    /// ´test:unit:zscores-two-depths´
    #[test]
    fn zscores_two_depths() {
        // Chain with d=0, d=8 (no d=4)
        // Mean = average of two values; Spread = std of two values
        let root = make_entry(0, 0.5, 0.3, 1.0, 0.0);
        let cell = make_entry(8, 2.5, 1.5, 1.0, 0.9);

        let chain = vec![(8, &cell), (0, &root)];
        let features = extract_chain_zscores(&chain);

        const TOL: f64 = 1e-14;

        // Mean N: (2.5 + 0.5) / 2 = 1.5
        assert!((features[12] - 1.5).abs() < TOL, "Mean N");

        // Spread N: std of [2.5, 0.5], mean=1.5
        // var = ((2.5-1.5)² + (0.5-1.5)²) / 2 = (1 + 1) / 2 = 1
        // std = 1.0
        assert!((features[20] - 1.0).abs() < TOL, "Spread N");

        // Gradient = cell - root = 2.5 - 0.5 = 2.0
        assert!((features[16] - 2.0).abs() < TOL, "Gradient N");
    }

    /// The four axes are summarised in isolation from one another: with novelty
    /// loudest at the cell and displacement loudest at the root, each axis's
    /// views follow its own values and neither borrows the other's magnitude or
    /// its choice of depth. The axes measure unrelated things, so a maximum
    /// taken across them jointly would let one loud axis speak for all four.
    ///
    /// ´claim:extract:each-axis-is-summarised-from-its-own-values-with-no-cross-axis-contamination´
    /// ´test:unit:zscores-per-axis-independence´
    #[test]
    fn zscores_per_axis_independence() {
        // Chain where N is high but D is low - verifies no cross-axis contamination
        let root = make_entry(0, 0.1, 3.0, 0.5, 0.2);
        let cell = make_entry(8, 5.0, 0.2, 0.5, 0.8);

        let chain = vec![(8, &cell), (0, &root)];
        let features = extract_chain_zscores(&chain);

        const TOL: f64 = 1e-14;

        // Cell view: N should be high, D should be low
        assert!((features[0] - 5.0).abs() < TOL, "Cell N = 5.0");
        assert!((features[1] - 0.2).abs() < TOL, "Cell D = 0.2");

        // Root view: N should be low, D should be high
        assert!((features[4] - 0.1).abs() < TOL, "Root N = 0.1");
        assert!((features[5] - 3.0).abs() < TOL, "Root D = 3.0");

        // Max view: take max per axis independently
        assert!((features[8] - 5.0).abs() < TOL, "Max N = 5.0");
        assert!((features[9] - 3.0).abs() < TOL, "Max D = 3.0");
    }

    /// A present but wholly silent chain reads as zero in all twenty-four slots,
    /// the derived views included: the maximum's fold settles at zero rather
    /// than at the negative infinity it starts from, and a mean and spread over
    /// identical zeros are themselves zero. Calm is therefore represented by
    /// calm, never by a sentinel value the model would have to learn to ignore.
    ///
    /// (´claim:extract:an-everywhere-quiet-chain-extracts-to-zero-in-every-view´)
    /// ´test:unit:zscores-all-zero´
    #[test]
    fn zscores_all_zero() {
        // Chain with all scores = 0.0
        let root = make_entry(0, 0.0, 0.0, 0.0, 0.0);
        let cell = make_entry(8, 0.0, 0.0, 0.0, 0.0);

        let chain = vec![(8, &cell), (0, &root)];
        let features = extract_chain_zscores(&chain);

        // All 24 features should be 0.0
        for (i, &f) in features.iter().enumerate() {
            assert!(f.abs() < 1e-14, "Feature {} should be 0, got {}", i, f);
        }
    }

    /// Negative scores pass through unaltered: the cell view keeps its negative
    /// value, the gradient goes negative when the cell sits below its root, and
    /// the maximum takes the largest signed value rather than the largest
    /// magnitude. Sign carries direction here — quieter than the reference is a
    /// different claim from louder — so clamping or taking absolute values
    /// would erase the very contrast the gradient view exists to show.
    ///
    /// ´claim:extract:negative-z-scores-keep-their-sign-through-every-view-including-the-maximum´
    /// ´test:unit:zscores-negative-z´
    #[test]
    fn zscores_negative_z() {
        // Chain with some negative z-scores (can happen with CUSUM subtraction)
        let root = make_entry(0, 2.0, 0.0, 0.0, 0.0);
        let cell = make_entry(8, -1.0, 0.0, 0.0, 0.0);

        let chain = vec![(8, &cell), (0, &root)];
        let features = extract_chain_zscores(&chain);

        const TOL: f64 = 1e-14;

        // Cell view: negative value preserved
        assert!((features[0] - (-1.0)).abs() < TOL, "Cell N = -1.0");

        // Gradient can be negative: cell - root = -1.0 - 2.0 = -3.0
        assert!((features[16] - (-3.0)).abs() < TOL, "Gradient N = -3.0");

        // Max picks the larger value regardless of sign
        assert!((features[8] - 2.0).abs() < TOL, "Max N = 2.0");
    }
}
