// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(clippy::doc_markdown, clippy::cast_precision_loss)]

//! Aggregate feature computation across Sentinels.
//!
//! Computes the 15 cross-Sentinel summary statistics that form the aggregate
//! feature block in φ. These features capture the overall anomaly profile
//! across all reporting Sentinels.
//!
//! # Feature Layout
//!
//! Fifteen features, in the order the aggregate block states
//! (´tab:feature:aggregate-block´).
//!
//! | Index | Feature | Description |
//! |-------|---------|-------------|
//! | 0 | Max cell-level max z | `max_s(max_a(z_{s,a}))` |
//! | 1 | Mean cell-level max z | `mean_s(max_a(z_{s,a}))` |
//! | 2 | Std cell-level max z | `std_s(max_a(z_{s,a}))` |
//! | 3–6 | Per-axis max z (N,D,S,C) | `max_s(z_{s,a})` per axis |
//! | 7–10 | Per-axis concordance | Fraction of Sentinels with `z_{s,a} > θ_a` |
//! | 11 | Cross-axis product | `max_s(z_{s,N}) × max_s(z_{s,D})` |
//! | 12 | Axis breadth | Count of axes where `max_s(z_{s,a}) > θ_a` |
//! | 13 | Coverage | `n_reporting / n_registered` |
//! | 14 | Max CUSUM | `max_s(max_a(cusum_{s,a}))` |
//!
//! # Tests
//!
//! Crate-level tests: `src/tests/aggregate.rs`
//!
//! # Cross-References
//!
//! - (´sec:feature:aggregate´) — the aggregate block's purpose, the fifteen
//!   features it holds, and the recalibration of the thresholds five of them
//!   are counted against

use crate::types::{SCORING_AXIS_COUNT, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Number of aggregate features.
///
/// The fifteen summaries of the current batch taken across every reporting
/// Sentinel (´tab:feature:aggregate-block´).
///
/// ´const:assayer:aggregate-feature-arity´ (´alg:const:count´)
/// ´const:assayer:aggregate-feature-arity-count-15´
pub const AGGREGATE_FEATURE_COUNT: usize = 15;

// ═══════════════════════════════════════════════════════════════════════════════
// Sub-Scores Type
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-Sentinel cell-level sub-scores for aggregate computation.
///
/// Contains the cell-level z-scores and CUSUMs extracted from each Sentinel's
/// deepest containing cell. Used to compute aggregate features.
#[derive(Clone, Debug, Default)]
pub struct SentinelSubScores {
    /// Cell-level z-scores per axis (N, D, S, C).
    pub z_scores: [f64; SCORING_AXIS_COUNT],
    /// Cell-level CUSUMs per axis (N, D, S, C).
    pub cusums: [f64; SCORING_AXIS_COUNT],
}

impl SentinelSubScores {
    /// Creates new sub-scores with all zeros.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            z_scores: [0.0; SCORING_AXIS_COUNT],
            cusums: [0.0; SCORING_AXIS_COUNT],
        }
    }

    /// Creates sub-scores from z-scores and CUSUMs.
    #[must_use]
    pub const fn from_values(z_scores: [f64; SCORING_AXIS_COUNT], cusums: [f64; SCORING_AXIS_COUNT]) -> Self {
        Self { z_scores, cusums }
    }

    /// Returns the maximum z-score across all axes.
    #[must_use]
    pub fn max_z(&self) -> f64 {
        self.z_scores.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }

    /// Returns the maximum CUSUM across all axes.
    #[must_use]
    pub fn max_cusum(&self) -> f64 {
        self.cusums.iter().copied().fold(f64::NEG_INFINITY, f64::max)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Aggregate Computation
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the 15 aggregate features from per-Sentinel sub-scores.
///
/// # Arguments
///
/// * `sentinel_sub_scores` — Per-Sentinel cell-level scores (only reporting Sentinels)
/// * `concordance_thresholds` — Per-axis z-score thresholds for concordance (θ_a)
/// * `n_registered` — Total number of registered Sentinels (for coverage)
///
/// # Returns
///
/// Array of 15 aggregate features as described in the module documentation.
///
/// # Degenerate Cases
///
/// - Zero reporting Sentinels: all features = 0.0 (including coverage)
/// - Zero registered Sentinels: coverage undefined, set to 0.0
///
/// # Cross-References
///
/// - (´tab:feature:aggregate-block´) — the fifteen features this computes
#[must_use]
pub fn compute_aggregates(
    sentinel_sub_scores: &[(SentinelId, SentinelSubScores)],
    concordance_thresholds: &[f64; SCORING_AXIS_COUNT],
    n_registered: usize,
) -> [f64; AGGREGATE_FEATURE_COUNT] {
    let mut features = [0.0f64; AGGREGATE_FEATURE_COUNT];

    let n_reporting = sentinel_sub_scores.len();
    if n_reporting == 0 {
        return features;
    }

    // Collect max z per Sentinel (for indices 0–2)
    let max_zs: Vec<f64> = sentinel_sub_scores.iter().map(|(_, s)| s.max_z()).collect();

    // Index 0: Max cell-level max z
    features[0] = max_zs.iter().copied().fold(f64::NEG_INFINITY, f64::max);
    if !features[0].is_finite() {
        features[0] = 0.0;
    }

    // Index 1: Mean cell-level max z
    let sum: f64 = max_zs.iter().sum();
    features[1] = sum / n_reporting as f64;

    // Index 2: Std cell-level max z
    if n_reporting > 1 {
        let mean = features[1];
        let variance: f64 = max_zs.iter().map(|&v| (v - mean).powi(2)).sum::<f64>() / n_reporting as f64;
        features[2] = variance.sqrt();
    }

    // Indices 3–6: Per-axis max z
    for axis in 0..SCORING_AXIS_COUNT {
        let max_for_axis = sentinel_sub_scores
            .iter()
            .map(|(_, s)| s.z_scores[axis])
            .fold(f64::NEG_INFINITY, f64::max);
        features[3 + axis] = if max_for_axis.is_finite() { max_for_axis } else { 0.0 };
    }

    // Indices 7–10: Per-axis concordance (fraction above threshold)
    for axis in 0..SCORING_AXIS_COUNT {
        let threshold = concordance_thresholds[axis];
        let count_above = sentinel_sub_scores
            .iter()
            .filter(|(_, s)| s.z_scores[axis] > threshold)
            .count();
        features[7 + axis] = count_above as f64 / n_reporting as f64;
    }

    // Index 11: Cross-axis product (Novelty × Displacement)
    features[11] = features[3] * features[4]; // max_N × max_D

    // Index 12: Axis breadth (count of axes with max > threshold)
    let mut breadth = 0usize;
    for axis in 0..SCORING_AXIS_COUNT {
        if features[3 + axis] > concordance_thresholds[axis] {
            breadth += 1;
        }
    }
    features[12] = breadth as f64;

    // Index 13: Coverage
    if n_registered > 0 {
        features[13] = n_reporting as f64 / n_registered as f64;
    }

    // Index 14: Max CUSUM
    let max_cusum = sentinel_sub_scores
        .iter()
        .map(|(_, s)| s.max_cusum())
        .fold(f64::NEG_INFINITY, f64::max);
    features[14] = if max_cusum.is_finite() { max_cusum } else { 0.0 };

    features
}
