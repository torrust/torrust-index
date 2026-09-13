// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`structure_empty_chain_returns_zeros`] | extract | A chain with no entries yields eight zeroed structure slots rather than a panic or a short vector, even when the caller still supplies a normalisation depth and a staleness. Structure extraction is total, so a cell whose report never arrived contributes a silent block that the rest of the row can be assembled around. |
//! | [`structure_single_entry_chain`] | extract | Six of the eight slots come in cell-and-root pairs, and on a chain of one entry both halves of each pair report that entry: rank share, energy ratio and maturity all agree. The pair is two positions along an ancestry rather than two separate measurements, so a cell that is its own root needs no special case — the general rule simply has nowhere else to look. |
//! | [`chain_length_normalisation`] | extract | Depth reaches the model as a ratio of logarithms — the cell's own log-depth divided by the log of the deepest depth the tree allows — and not as a raw count of levels. The log flattens the difference between deep siblings, where one further level means little, while keeping the early levels distinguishable; dividing by the tree's own maximum makes the figure comparable across trees of different configured depth. |
//! | [`report_staleness_log`] | extract | Age enters the vector as the logarithm of one plus the elapsed seconds, so the difference between a fresh report and a minute-old one weighs far more than the difference between an hour and two. The model needs to discount stale evidence without letting a single very old report dominate the row through sheer magnitude. |
//! | [`zero_staleness_is_zero`] | extract | cites (´claim:extract:report-staleness-enters-as-the-log-of-one-plus-the-elapsed-seconds´) |
//! | [`zero_cap_handled`] | extract | A cell reporting no capacity yields a rank share of zero rather than a division by zero, for the cell and root halves alike, even though the entry does claim a non-zero rank. A capacity of nothing means the cell holds no contest to be ranked within, and a non-finite value here would propagate through standardisation and poison the entire feature row. |
//! | [`three_entry_chain`] | extract | On a chain spanning several depths the paired slots read from opposite ends and nothing in between: the cell half takes the deepest entry, the root half the shallowest, and the interior entry contributes to neither. The model is being shown local conditions beside the conditions of the whole region the cell sits in, so the two must be read from genuinely different places for the contrast between them to mean anything. |
//! | [`chain_length_deep`] | extract | A cell sitting at the tree's configured maximum depth reports a normalised chain length of exactly one — the upper end of the scale is reached, not merely approached. The feature is therefore a genuine fraction of the available depth, so a model trained against one tree configuration reads the same span of values under another. |
//! | [`chain_length_root_only`] | extract | cites (´claim:extract:normalised-chain-length-reaches-one-exactly-at-the-maximum-depth´) |

//! Chain structure feature extraction.
//!
//! Extracts 8 structural features from the ancestor chain.
//!
//! # Feature Layout
//!
//! | Index | Feature | Description |
//! |-------|---------|-------------|
//! | 0 | Cell rank / cap | Rank normalised by capacity for deepest cell |
//! | 1 | Root rank / cap | Rank normalised by capacity for root cell |
//! | 2 | Cell energy ratio | Energy ratio for deepest cell |
//! | 3 | Root energy ratio | Energy ratio for root cell |
//! | 4 | Cell maturity | `1 - noise_influence` for deepest cell |
//! | 5 | Root maturity | `1 - noise_influence` for root cell |
//! | 6 | Chain length | Normalised log depth |
//! | 7 | Report staleness | `ln(1 + staleness_seconds)` |
//!
//! # Cross-References
//!
//! - (´tab:extraction:chain-structure´) — the structural features this module
//!   reads off the shape of the ancestor chain

// Rank and cap are always small values; no precision loss.
#![allow(clippy::cast_precision_loss)]

use crate::report::ReportCellEntry;

// ═══════════════════════════════════════════════════════════════════════════════
// Chain Structure Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts 8 structural features from an ancestor chain.
///
/// # Arguments
///
/// * `chain` — Ancestor chain entries, deepest first. Must be non-empty.
/// * `d_chain` — Maximum chain depth for normalisation.
/// * `report_staleness` — Time since report was received, in seconds.
///
/// # Returns
///
/// An array of 8 f64 values representing the structural features.
///
/// # Degenerate Cases
///
/// - Empty chain: all zeros.
/// - Zero capacity: rank/cap = 0.0 (division guarded).
///
/// # Cross-References
///
/// - (´tab:extraction:chain-structure´) — the eight structural features
#[must_use]
pub fn extract_chain_structure(chain: &[(u8, &ReportCellEntry)], d_chain: u8, report_staleness: f64) -> [f64; 8] {
    let mut features = [0.0f64; 8];

    if chain.is_empty() {
        return features;
    }

    let cell = chain[0].1;
    let cell_depth = chain[0].0;
    let root = chain.last().map_or(cell, |(_, e)| *e);

    // Feature 0: Cell rank / cap
    features[0] = if cell.cap > 0 {
        cell.rank as f64 / cell.cap as f64
    } else {
        0.0
    };

    // Feature 1: Root rank / cap
    features[1] = if root.cap > 0 {
        root.rank as f64 / root.cap as f64
    } else {
        0.0
    };

    // Feature 2: Cell energy ratio
    features[2] = cell.energy_ratio;

    // Feature 3: Root energy ratio
    features[3] = root.energy_ratio;

    // Feature 4: Cell maturity (1 - noise_influence)
    features[4] = 1.0 - cell.noise_influence;

    // Feature 5: Root maturity (1 - noise_influence)
    features[5] = 1.0 - root.noise_influence;

    // Feature 6: Chain length normalised
    // log₂(1 + cell_depth) / log₂(1 + d_chain)
    let log_cell = (1.0 + f64::from(cell_depth)).log2();
    let log_d_chain = (1.0 + f64::from(d_chain)).log2();
    features[6] = if log_d_chain > 0.0 { log_cell / log_d_chain } else { 0.0 };

    // Feature 7: Report staleness
    // ln(1 + staleness_seconds)
    features[7] = report_staleness.ln_1p();

    features
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::items_after_statements)]
    #![allow(clippy::suboptimal_flops)]

    use super::*;
    use crate::report::{AxisScoreSet, ReportCellEntry};

    /// Creates a test entry with specified structure values.
    fn make_entry_structure(depth: u8, rank: usize, cap: usize, energy_ratio: f64, noise_influence: f64) -> ReportCellEntry {
        ReportCellEntry {
            depth,
            sample_count: 100,
            is_competitive: true,
            rank,
            cap,
            energy_ratio,
            noise_influence,
            scores: AxisScoreSet::default(),
            degraded: false,
        }
    }

    /// A chain with no entries yields eight zeroed structure slots rather than
    /// a panic or a short vector, even when the caller still supplies a
    /// normalisation depth and a staleness. Structure extraction is total, so a
    /// cell whose report never arrived contributes a silent block that the rest
    /// of the row can be assembled around.
    ///
    /// ´claim:extract:an-empty-chain-yields-zero-structure-features-rather-than-a-failure´
    /// ´test:unit:structure-empty-chain-returns-zeros´
    #[test]
    fn structure_empty_chain_returns_zeros() {
        let chain: Vec<(u8, &ReportCellEntry)> = vec![];
        let features = extract_chain_structure(&chain, 128, 0.0);
        assert!(features.iter().all(|&v| v == 0.0));
    }

    /// Six of the eight slots come in cell-and-root pairs, and on a chain of one
    /// entry both halves of each pair report that entry: rank share, energy
    /// ratio and maturity all agree. The pair is two positions along an
    /// ancestry rather than two separate measurements, so a cell that is its own
    /// root needs no special case — the general rule simply has nowhere else to
    /// look.
    ///
    /// ´claim:extract:a-single-entry-chain-makes-the-cell-and-root-halves-of-each-pair-coincide´
    /// ´test:unit:structure-single-entry-chain´
    #[test]
    fn structure_single_entry_chain() {
        let entry = make_entry_structure(8, 5, 10, 0.8, 0.2);
        let chain = vec![(8, &entry)];
        let features = extract_chain_structure(&chain, 128, 0.0);

        const TOL: f64 = 1e-10;

        // Cell and root are the same
        assert!((features[0] - 0.5).abs() < TOL, "Cell rank/cap");
        assert!((features[1] - 0.5).abs() < TOL, "Root rank/cap");
        assert!((features[2] - 0.8).abs() < TOL, "Cell energy ratio");
        assert!((features[3] - 0.8).abs() < TOL, "Root energy ratio");
        assert!((features[4] - 0.8).abs() < TOL, "Cell maturity (1 - 0.2)");
        assert!((features[5] - 0.8).abs() < TOL, "Root maturity");
    }

    /// Depth reaches the model as a ratio of logarithms — the cell's own
    /// log-depth divided by the log of the deepest depth the tree allows — and
    /// not as a raw count of levels. The log flattens the difference between
    /// deep siblings, where one further level means little, while keeping the
    /// early levels distinguishable; dividing by the tree's own maximum makes
    /// the figure comparable across trees of different configured depth.
    ///
    /// ´claim:extract:chain-length-enters-as-log-depth-divided-by-log-maximum-depth´
    /// ´test:unit:chain-length-normalisation´
    #[test]
    fn chain_length_normalisation() {
        let entry = make_entry_structure(64, 1, 10, 1.0, 0.0);
        let chain = vec![(64, &entry)];
        let features = extract_chain_structure(&chain, 128, 0.0);

        // log₂(1 + 64) / log₂(1 + 128) = log₂(65) / log₂(129)
        let expected = (65.0_f64).log2() / (129.0_f64).log2();
        assert!((features[6] - expected).abs() < 1e-10, "Chain length normalisation");
    }

    /// Age enters the vector as the logarithm of one plus the elapsed seconds,
    /// so the difference between a fresh report and a minute-old one weighs far
    /// more than the difference between an hour and two. The model needs to
    /// discount stale evidence without letting a single very old report
    /// dominate the row through sheer magnitude.
    ///
    /// ´claim:extract:report-staleness-enters-as-the-log-of-one-plus-the-elapsed-seconds´
    /// ´test:unit:report-staleness-log´
    #[test]
    fn report_staleness_log() {
        let entry = make_entry_structure(8, 1, 10, 1.0, 0.0);
        let chain = vec![(8, &entry)];

        // Staleness of 100 seconds → ln(101) ≈ 4.615
        let features = extract_chain_structure(&chain, 128, 100.0);
        let expected = (101.0_f64).ln();
        assert!((features[7] - expected).abs() < 1e-10, "Staleness ln(1 + 100)");
    }

    /// A report received at this instant carries no staleness at all: the
    /// transform's fixed point at zero means fresh evidence contributes nothing
    /// to the age slot rather than a small constant offset, so the feature is a
    /// measure of decay and not of mere existence.
    ///
    /// (´claim:extract:report-staleness-enters-as-the-log-of-one-plus-the-elapsed-seconds´)
    /// ´test:unit:zero-staleness-is-zero´
    #[test]
    fn zero_staleness_is_zero() {
        let entry = make_entry_structure(8, 1, 10, 1.0, 0.0);
        let chain = vec![(8, &entry)];
        let features = extract_chain_structure(&chain, 128, 0.0);

        // ln(1 + 0) = ln(1) = 0
        assert!((features[7]).abs() < 1e-14, "Zero staleness");
    }

    /// A cell reporting no capacity yields a rank share of zero rather than a
    /// division by zero, for the cell and root halves alike, even though the
    /// entry does claim a non-zero rank. A capacity of nothing means the cell
    /// holds no contest to be ranked within, and a non-finite value here would
    /// propagate through standardisation and poison the entire feature row.
    ///
    /// ´claim:extract:a-zero-capacity-cell-yields-a-zero-rank-share-rather-than-a-non-finite-feature´
    /// ´test:unit:zero-cap-handled´
    #[test]
    fn zero_cap_handled() {
        let entry = make_entry_structure(8, 5, 0, 0.8, 0.2);
        let chain = vec![(8, &entry)];
        let features = extract_chain_structure(&chain, 128, 0.0);

        // rank/cap with cap=0 should be 0.0, not NaN or panic
        assert!((features[0]).abs() < 1e-14, "Zero cap handled for cell");
        assert!((features[1]).abs() < 1e-14, "Zero cap handled for root");
    }

    /// On a chain spanning several depths the paired slots read from opposite
    /// ends and nothing in between: the cell half takes the deepest entry, the
    /// root half the shallowest, and the interior entry contributes to neither.
    /// The model is being shown local conditions beside the conditions of the
    /// whole region the cell sits in, so the two must be read from genuinely
    /// different places for the contrast between them to mean anything.
    ///
    /// ´claim:extract:the-paired-structure-slots-read-the-deepest-and-shallowest-entries-and-nothing-between´
    /// ´test:unit:three-entry-chain´
    #[test]
    fn three_entry_chain() {
        let root = make_entry_structure(0, 1, 100, 0.5, 0.5);
        let mid = make_entry_structure(4, 3, 50, 0.7, 0.3);
        let cell = make_entry_structure(8, 5, 10, 0.9, 0.1);

        // Chain is deepest-first
        let chain = vec![(8, &cell), (4, &mid), (0, &root)];
        let features = extract_chain_structure(&chain, 128, 60.0);

        const TOL: f64 = 1e-10;

        // Cell features
        assert!((features[0] - 0.5).abs() < TOL, "Cell rank/cap = 5/10");
        assert!((features[2] - 0.9).abs() < TOL, "Cell energy ratio");
        assert!((features[4] - 0.9).abs() < TOL, "Cell maturity (1 - 0.1)");

        // Root features
        assert!((features[1] - 0.01).abs() < TOL, "Root rank/cap = 1/100");
        assert!((features[3] - 0.5).abs() < TOL, "Root energy ratio");
        assert!((features[5] - 0.5).abs() < TOL, "Root maturity (1 - 0.5)");

        // Chain length: log₂(9) / log₂(129)
        let expected_len = (9.0_f64).log2() / (129.0_f64).log2();
        assert!((features[6] - expected_len).abs() < TOL, "Chain length");

        // Staleness: ln(61)
        let expected_stale = (61.0_f64).ln();
        assert!((features[7] - expected_stale).abs() < TOL, "Staleness");
    }

    /// A cell sitting at the tree's configured maximum depth reports a
    /// normalised chain length of exactly one — the upper end of the scale is
    /// reached, not merely approached. The feature is therefore a genuine
    /// fraction of the available depth, so a model trained against one tree
    /// configuration reads the same span of values under another.
    ///
    /// ´claim:extract:normalised-chain-length-reaches-one-exactly-at-the-maximum-depth´
    /// ´test:unit:chain-length-deep´
    #[test]
    fn chain_length_deep() {
        // When depth = d_chain, normalised chain length should be 1.0
        // log₂(1 + d_chain) / log₂(1 + d_chain) = 1.0
        let entry = make_entry_structure(16, 1, 10, 1.0, 0.0);
        let chain = vec![(16, &entry)];
        let features = extract_chain_structure(&chain, 16, 0.0);

        // log₂(17) / log₂(17) = 1.0
        assert!((features[6] - 1.0).abs() < 1e-10, "Chain length at max depth = 1.0");
    }

    /// A cell that is the root sits at the bottom of the same scale, reporting
    /// zero. Together with the maximum-depth end this pins the feature to the
    /// unit interval, so depth is expressed as a position between the two
    /// extremes rather than as an open-ended count.
    ///
    /// (´claim:extract:normalised-chain-length-reaches-one-exactly-at-the-maximum-depth´)
    /// ´test:unit:chain-length-root-only´
    #[test]
    fn chain_length_root_only() {
        // When depth = 0, normalised chain length should be 0.0
        // log₂(1 + 0) / log₂(1 + d_chain) = log₂(1) / log₂(17) = 0.0
        let entry = make_entry_structure(0, 1, 10, 1.0, 0.0);
        let chain = vec![(0, &entry)];
        let features = extract_chain_structure(&chain, 16, 0.0);

        // log₂(1) = 0
        assert!((features[6]).abs() < 1e-10, "Chain length at root = 0.0");
    }
}
