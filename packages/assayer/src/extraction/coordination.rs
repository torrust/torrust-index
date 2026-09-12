// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`empty_entries_returns_zeros`] | extract | A report carrying no coordination contexts extracts as twelve zeros, including the fractions that would otherwise divide by an entry count of nothing. A Sentinel whose analysis sets are all quiet contributes an honestly silent block rather than a non-finite value that would spread through the rest of the row. |
//! | [`single_entry`] | extract | A lone context fills all four kinds of coordination feature at once: its own z-scores become the per-axis maxima, it is both the deepest and the shallowest context, it is wholly concordant with itself if it clears the threshold, and it occupies one slot of the configured capacity. The four kinds are different questions asked of the same population, so a population of one answers every one of them consistently rather than leaving some slots undefined. |
//! | [`concordance_threshold`] | extract | Concordance counts contexts, not magnitude: a context clears the bar if its loudest axis passes the threshold, and the feature is the fraction of contexts that do. Two of three loud contexts read as two-thirds however far past the bar those two ran, which is what distinguishes broad agreement across an analysis set from one context shouting alone. |
//! | [`cusum_features`] | extract | Accumulated drift gets its own four slots, maximised per axis across every context independently of the z-score slots above them. Each axis keeps its own maximum, so the context that drifted furthest on one axis does not thereby claim the other three, and sustained coordinated drift stays visible even where no single batch was loud enough to raise a z-score. |
//! | [`root_is_shallowest`] | extract | The root context slot is filled by whichever context sits shallowest, found by depth rather than by position in the list, and it reports that context's own loudest axis even while a deeper context is far louder and dominates the per-axis maxima. Broad shallow coordination and sharp deep coordination are separately readable, so the model can tell a region-wide shift from a local one. |
//! | [`concordance_threshold_exact`] | extract | The threshold is strict: contexts sitting exactly on it are not counted, so an analysis set poised precisely at the bar reads as no concordance rather than as total concordance. The boundary belongs to the quiet side, which keeps a threshold configured to a value the scores can land on exactly from flipping the feature between its extremes. |
//! | [`concordance_above_threshold`] | extract | When every context clears the bar concordance reaches one exactly, pinning the top of a scale whose bottom the exactly-at-threshold case pins at zero. The feature is therefore a true proportion over the contexts present, comparable between Sentinels that hold different numbers of them. |
//! | [`peak_depth_normalisation`] | extract | Peak context depth is the deepest context's depth expressed as a fraction of the tree's maximum depth, so the slot says how far down coordination reaches rather than how many levels that happens to be. Sentinels configured with different tree depths therefore report the same figure for coordination that penetrates equally far. |

//! Coordination context feature extraction.
//!
//! Extracts 12 features from coordination contexts across analysis sets.
//!
//! # Feature Layout
//!
//! | Index Range | Feature | Description |
//! |-------------|---------|-------------|
//! | 0–3 | Per-axis max coord z | Max `max_z` across entries per axis |
//! | 4–7 | Per-axis max coord CUSUM | Max CUSUM across entries per axis |
//! | 8 | Concordance | Fraction of entries with max z above threshold |
//! | 9 | Active context fraction | `entries.len() / capacity` |
//! | 10 | Peak context depth | Max depth normalised |
//! | 11 | Root context max z | Max z across axes for shallowest entry |
//!
//! # Cross-References
//!
//! - (´tab:extraction:coordination´) — the coordination features this module
//!   extracts from the analysis-set entries

// Each axis is used to compute multiple indices; this is clearer than enumerate.
#![allow(clippy::needless_range_loop)]
// Entry counts and lengths are always small; no precision loss.
#![allow(clippy::cast_precision_loss)]

use crate::report::CoordinationEntry;
use crate::types::SCORING_AXIS_COUNT;

// ═══════════════════════════════════════════════════════════════════════════════
// Axis Accessors
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the maximum z-score across all axes for an entry.
#[inline]
fn max_z_across_axes(entry: &CoordinationEntry) -> f64 {
    entry.scores.max_z_per_axis().into_iter().fold(f64::NEG_INFINITY, f64::max)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Coordination Feature Extraction
// ═══════════════════════════════════════════════════════════════════════════════

/// Extracts 12 coordination features.
///
/// # Arguments
///
/// * `entries` — Coordination context entries from the report.
/// * `thresholds` — Concordance thresholds per axis `[θ_N, θ_D, θ_S, θ_C]`.
/// * `d_chain` — Maximum chain depth for normalisation.
/// * `capacity` — Total analysis-set size (denominator for active context fraction).
///
/// # Returns
///
/// An array of 12 f64 values representing coordination features.
///
/// # Degenerate Cases
///
/// - Empty entries: all zeros.
///
/// # Cross-References
///
/// - (´tab:extraction:coordination´) — the twelve coordination features
#[must_use]
pub fn extract_coordination(
    entries: &[CoordinationEntry],
    thresholds: &[f64; SCORING_AXIS_COUNT],
    d_chain: u8,
    capacity: usize,
) -> [f64; 12] {
    let mut features = [0.0f64; 12];

    if entries.is_empty() {
        return features;
    }

    // Features 0–3: Per-axis max coordination z
    for axis in 0..SCORING_AXIS_COUNT {
        let max_z = entries
            .iter()
            .map(|e| e.scores.max_z_per_axis()[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        features[axis] = if max_z.is_finite() { max_z } else { 0.0 };
    }

    // Features 4–7: Per-axis max coordination CUSUM
    for axis in 0..SCORING_AXIS_COUNT {
        let max_cusum = entries
            .iter()
            .map(|e| e.scores.cusum_per_axis()[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        features[SCORING_AXIS_COUNT + axis] = if max_cusum.is_finite() { max_cusum } else { 0.0 };
    }

    // Feature 8: Concordance — fraction of entries with max z > θ_coord
    // Use the first threshold as the coordination threshold (typically all are equal)
    let theta_coord = thresholds[0];
    let concordant_count = entries.iter().filter(|e| max_z_across_axes(e) > theta_coord).count();
    features[8] = concordant_count as f64 / entries.len() as f64;

    // Feature 9: Active context fraction
    features[9] = entries.len() as f64 / capacity.max(1) as f64;

    // Feature 10: Peak context depth / d_chain
    let max_depth = entries.iter().map(|e| e.depth).max().unwrap_or(0);
    features[10] = f64::from(max_depth) / f64::from(d_chain.max(1));

    // Feature 11: Root context max z — max z across axes for shallowest entry
    let shallowest = entries.iter().min_by_key(|e| e.depth);
    features[11] = shallowest.map_or(0.0, |e| {
        let max_z = max_z_across_axes(e);
        if max_z.is_finite() { max_z } else { 0.0 }
    });

    features
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    #![allow(clippy::items_after_statements)]

    use super::*;
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, CoordinationEntry};

    /// Creates a test coordination entry with specified z-scores per axis.
    fn make_coord_entry(depth: u8, n_z: f64, d_z: f64, s_z: f64, c_z: f64) -> CoordinationEntry {
        CoordinationEntry {
            depth,
            cells_reporting: 10,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: n_z,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    max_z: d_z,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: s_z,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: c_z,
                    ..Default::default()
                },
            },
        }
    }

    /// Creates a test coordination entry with specified CUSUM values.
    fn make_coord_entry_cusum(depth: u8, n_c: f64, d_c: f64, s_c: f64, c_c: f64) -> CoordinationEntry {
        CoordinationEntry {
            depth,
            cells_reporting: 10,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    cusum: n_c,
                    ..Default::default()
                },
                displacement: AxisScoreSnapshot {
                    cusum: d_c,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    cusum: s_c,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    cusum: c_c,
                    ..Default::default()
                },
            },
        }
    }

    /// A report carrying no coordination contexts extracts as twelve zeros,
    /// including the fractions that would otherwise divide by an entry count of
    /// nothing. A Sentinel whose analysis sets are all quiet contributes an
    /// honestly silent block rather than a non-finite value that would spread
    /// through the rest of the row.
    ///
    /// ´claim:extract:a-report-with-no-coordination-contexts-extracts-as-twelve-zeros´
    /// ´test:unit:empty-entries-returns-zeros´
    #[test]
    fn empty_entries_returns_zeros() {
        let entries: Vec<CoordinationEntry> = vec![];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);
        assert!(features.iter().all(|&v| v == 0.0));
    }

    /// A lone context fills all four kinds of coordination feature at once: its
    /// own z-scores become the per-axis maxima, it is both the deepest and the
    /// shallowest context, it is wholly concordant with itself if it clears the
    /// threshold, and it occupies one slot of the configured capacity. The four
    /// kinds are different questions asked of the same population, so a
    /// population of one answers every one of them consistently rather than
    /// leaving some slots undefined.
    ///
    /// ´claim:extract:a-lone-coordination-context-answers-every-one-of-the-twelve-slots-consistently´
    /// ´test:unit:single-entry´
    #[test]
    fn single_entry() {
        let entry = make_coord_entry(4, 2.0, 1.5, 1.0, 0.5);
        let entries = vec![entry];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        const TOL: f64 = 1e-10;

        // Per-axis max z
        assert!((features[0] - 2.0).abs() < TOL, "Max N z");
        assert!((features[1] - 1.5).abs() < TOL, "Max D z");
        assert!((features[2] - 1.0).abs() < TOL, "Max S z");
        assert!((features[3] - 0.5).abs() < TOL, "Max C z");

        // Concordance: max(2.0, 1.5, 1.0, 0.5) = 2.0 > 0.5 → 1/1 = 1.0
        assert!((features[8] - 1.0).abs() < TOL, "Concordance");

        // Active context fraction: 1 / 100 = 0.01
        assert!((features[9] - 0.01).abs() < TOL, "Active context fraction");

        // Peak depth: 4 / 128
        assert!((features[10] - 4.0 / 128.0).abs() < TOL, "Peak depth");

        // Root context max z (only entry is root)
        assert!((features[11] - 2.0).abs() < TOL, "Root context max z");
    }

    /// Concordance counts contexts, not magnitude: a context clears the bar if
    /// its loudest axis passes the threshold, and the feature is the fraction of
    /// contexts that do. Two of three loud contexts read as two-thirds however
    /// far past the bar those two ran, which is what distinguishes broad
    /// agreement across an analysis set from one context shouting alone.
    ///
    /// ´claim:extract:concordance-is-the-fraction-of-contexts-clearing-the-threshold-not-their-magnitude´
    /// ´test:unit:concordance-threshold´
    #[test]
    fn concordance_threshold() {
        // 3 entries, only 2 have max z > 0.5
        let e1 = make_coord_entry(0, 0.3, 0.2, 0.1, 0.4); // max = 0.4 ≤ 0.5
        let e2 = make_coord_entry(4, 0.6, 0.1, 0.2, 0.3); // max = 0.6 > 0.5
        let e3 = make_coord_entry(8, 1.0, 0.5, 0.3, 0.2); // max = 1.0 > 0.5

        let entries = vec![e1, e2, e3];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        // Concordance: 2/3
        assert!((features[8] - 2.0 / 3.0).abs() < 1e-10, "Concordance = 2/3");
    }

    /// Accumulated drift gets its own four slots, maximised per axis across
    /// every context independently of the z-score slots above them. Each axis
    /// keeps its own maximum, so the context that drifted furthest on one axis
    /// does not thereby claim the other three, and sustained coordinated drift
    /// stays visible even where no single batch was loud enough to raise a
    /// z-score.
    ///
    /// ´claim:extract:coordination-drift-is-maximised-per-axis-across-contexts-in-its-own-four-slots´
    /// ´test:unit:cusum-features´
    #[test]
    fn cusum_features() {
        let e1 = make_coord_entry_cusum(0, 0.1, 0.2, 0.3, 0.4);
        let e2 = make_coord_entry_cusum(4, 0.5, 0.6, 0.7, 0.8);

        let entries = vec![e1, e2];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        const TOL: f64 = 1e-10;

        // Per-axis max CUSUM
        assert!((features[4] - 0.5).abs() < TOL, "Max N CUSUM");
        assert!((features[5] - 0.6).abs() < TOL, "Max D CUSUM");
        assert!((features[6] - 0.7).abs() < TOL, "Max S CUSUM");
        assert!((features[7] - 0.8).abs() < TOL, "Max C CUSUM");
    }

    /// The root context slot is filled by whichever context sits shallowest,
    /// found by depth rather than by position in the list, and it reports that
    /// context's own loudest axis even while a deeper context is far louder and
    /// dominates the per-axis maxima. Broad shallow coordination and sharp deep
    /// coordination are separately readable, so the model can tell a
    /// region-wide shift from a local one.
    ///
    /// ´claim:extract:the-root-context-slot-follows-the-shallowest-depth-even-when-a-deeper-context-is-louder´
    /// ´test:unit:root-is-shallowest´
    #[test]
    fn root_is_shallowest() {
        let e1 = make_coord_entry(8, 3.0, 0.0, 0.0, 0.0); // Deepest
        let e2 = make_coord_entry(4, 2.0, 0.0, 0.0, 0.0);
        let e3 = make_coord_entry(0, 1.0, 0.0, 0.0, 0.0); // Shallowest (root)

        let entries = vec![e1, e2, e3];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        // Root context max z should be from e3 (depth 0)
        assert!((features[11] - 1.0).abs() < 1e-10, "Root context max z = 1.0");

        // But max z overall should be 3.0
        assert!((features[0] - 3.0).abs() < 1e-10, "Max N z = 3.0");
    }

    /// The threshold is strict: contexts sitting exactly on it are not counted,
    /// so an analysis set poised precisely at the bar reads as no concordance
    /// rather than as total concordance. The boundary belongs to the quiet side,
    /// which keeps a threshold configured to a value the scores can land on
    /// exactly from flipping the feature between its extremes.
    ///
    /// ´claim:extract:a-context-exactly-at-the-concordance-threshold-does-not-count-as-concordant´
    /// ´test:unit:concordance-threshold-exact´
    #[test]
    fn concordance_threshold_exact() {
        // 2 entries with max_z exactly at threshold
        // Implementation uses > (strictly greater), so neither counts
        let e1 = make_coord_entry(4, 0.5, 0.5, 0.5, 0.5);
        let e2 = make_coord_entry(8, 0.5, 0.5, 0.5, 0.5);

        let entries = vec![e1, e2];
        let thresholds = [0.5; 4]; // threshold = 0.5
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        // Since implementation uses >, entries exactly at threshold don't count
        // Concordance = 0/2 = 0.0
        assert!((features[8]).abs() < 1e-10, "Concordance = 0.0 (exactly at threshold)");
    }

    /// When every context clears the bar concordance reaches one exactly,
    /// pinning the top of a scale whose bottom the exactly-at-threshold case
    /// pins at zero. The feature is therefore a true proportion over the
    /// contexts present, comparable between Sentinels that hold different
    /// numbers of them.
    ///
    /// ´claim:extract:concordance-reaches-one-exactly-when-every-context-clears-the-threshold´
    /// ´test:unit:concordance-above-threshold´
    #[test]
    fn concordance_above_threshold() {
        // 2 entries with max_z above threshold
        let e1 = make_coord_entry(4, 0.6, 0.4, 0.4, 0.4); // max_z = 0.6 > 0.5
        let e2 = make_coord_entry(8, 0.7, 0.3, 0.3, 0.3); // max_z = 0.7 > 0.5

        let entries = vec![e1, e2];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 128, 100);

        // Both entries are above threshold → concordance = 2/2 = 1.0
        assert!((features[8] - 1.0).abs() < 1e-10, "Concordance = 1.0 (both above threshold)");
    }

    /// Peak context depth is the deepest context's depth expressed as a fraction
    /// of the tree's maximum depth, so the slot says how far down coordination
    /// reaches rather than how many levels that happens to be. Sentinels
    /// configured with different tree depths therefore report the same figure
    /// for coordination that penetrates equally far.
    ///
    /// ´claim:extract:peak-context-depth-is-the-deepest-context-as-a-fraction-of-the-maximum-depth´
    /// ´test:unit:peak-depth-normalisation´
    #[test]
    fn peak_depth_normalisation() {
        // Entries at d=2, d=6; d_chain=16 → peak_depth = 6/16 = 0.375
        let e1 = make_coord_entry(2, 1.0, 0.0, 0.0, 0.0);
        let e2 = make_coord_entry(6, 1.0, 0.0, 0.0, 0.0);

        let entries = vec![e1, e2];
        let thresholds = [0.5; 4];
        let features = extract_coordination(&entries, &thresholds, 16, 100);

        // Peak depth = 6, d_chain = 16 → 6/16 = 0.375
        assert!((features[10] - 0.375).abs() < 1e-10, "Peak depth = 6/16 = 0.375");
    }
}
