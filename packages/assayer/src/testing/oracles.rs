// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Independent reference computations for tests (´dec:harness:oracle-tier´).

use faer::Mat;

/// The source of authority an oracle follows.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OracleProvenance {
    /// A formula stated directly by the specification.
    SpecificationFormula,
    /// A derivation independent of the implementation under test.
    IndependentDerivation,
    /// A reference calculation chosen and owned by the harness.
    HarnessReference,
}

/// Provenance of [`pairwise_rank`].
pub const PAIRWISE_RANK_PROVENANCE: OracleProvenance = OracleProvenance::SpecificationFormula;

/// Provenance of [`regularised_schur_complement`].
pub const REGULARISED_SCHUR_PROVENANCE: OracleProvenance = OracleProvenance::SpecificationFormula;

/// Provenance of [`decay_recurrence`].
pub const DECAY_RECURRENCE_PROVENANCE: OracleProvenance = OracleProvenance::SpecificationFormula;

/// Provenance of [`dimension_width`].
pub const DIMENSION_WIDTH_PROVENANCE: OracleProvenance = OracleProvenance::SpecificationFormula;

/// Computes tie-aware rank discrimination by comparing every positive row with every negative row.
///
/// Provenance is [`OracleProvenance::SpecificationFormula`]: this evaluates the pairwise reading of rank discrimination in (´alg:monitoring:auc´), awarding one for a correctly ordered pair and one half for a tie, after applying the configured gate to each class. It deliberately avoids the production single-sort route through `health::published::assign_ranks` and `health::published::auc_from_ranks`; no ranks or rank sums are formed here.
#[must_use]
pub fn pairwise_rank(scores: &[(f64, bool)], min_class_count: usize, tie_tolerance: f64) -> Option<f64> {
    debug_assert!(tie_tolerance.is_finite() && tie_tolerance >= 0.0);
    debug_assert!(scores.iter().all(|(score, _)| score.is_finite()));

    let positives = scores.iter().filter(|(_, positive)| *positive).count();
    let negatives = scores.len() - positives;
    if positives < min_class_count || negatives < min_class_count {
        return None;
    }

    let mut pair_count = 0.0;
    let mut credit = 0.0;
    for (positive_score, _) in scores.iter().filter(|(_, positive)| *positive) {
        for (negative_score, _) in scores.iter().filter(|(_, positive)| !*positive) {
            pair_count += 1.0;
            let difference = positive_score - negative_score;
            if difference.abs() <= tie_tolerance {
                credit += 0.5;
            } else if difference > 0.0 {
                credit += 1.0;
            }
        }
    }

    let result = credit / pair_count;
    debug_assert!((0.0..=1.0).contains(&result));
    Some(result)
}

/// Computes the dense regularised Schur complement by pivoted Gauss–Jordan elimination.
///
/// Provenance is [`OracleProvenance::SpecificationFormula`]: this evaluates $B_{kk} - B_{kr}(B_{rr} + \delta I)^{-1}B_{rk}$ from (´alg:gaussian:regularised-schur´). It deliberately avoids the production route through `model::marginalise::compute_schur_complement`, `linalg::bridge::cholesky`, and `linalg::bridge::schur_half_solve`; the removed system is instead solved as one dense augmented matrix with partial pivoting.
///
/// Returns `None` when the input is not a finite square matrix, the removed indices are not a unique in-bounds set, the offset is invalid, or elimination finds no usable pivot.
#[must_use]
pub fn regularised_schur_complement(precision: &Mat<f64>, remove_indices: &[usize], delta: f64) -> Option<Mat<f64>> {
    let width = precision.nrows();
    if precision.ncols() != width || !delta.is_finite() || delta < 0.0 {
        return None;
    }
    if !(0..width).all(|row| (0..width).all(|column| precision[(row, column)].is_finite())) {
        return None;
    }

    let mut removed = vec![false; width];
    for &index in remove_indices {
        if index >= width || removed[index] {
            return None;
        }
        removed[index] = true;
    }
    let keep_indices: Vec<_> = removed
        .iter()
        .enumerate()
        .filter_map(|(index, is_removed)| (!is_removed).then_some(index))
        .collect();

    let removed_width = remove_indices.len();
    let kept_width = keep_indices.len();
    if removed_width == 0 {
        return Some(Mat::from_fn(width, width, |row, column| precision[(row, column)]));
    }

    let mut augmented = vec![vec![0.0; removed_width + kept_width]; removed_width];
    for (row, (&source_row, values)) in remove_indices.iter().zip(&mut augmented).enumerate() {
        for (column, &source_column) in remove_indices.iter().enumerate() {
            values[column] = precision[(source_row, source_column)];
        }
        values[row] += delta;
        for (column, &source_column) in keep_indices.iter().enumerate() {
            values[removed_width + column] = precision[(source_row, source_column)];
        }
    }

    for pivot_column in 0..removed_width {
        let pivot_row = augmented
            .iter()
            .enumerate()
            .skip(pivot_column)
            .max_by(|(_, left), (_, right)| left[pivot_column].abs().total_cmp(&right[pivot_column].abs()))
            .map(|(row, _)| row)?;
        augmented.swap(pivot_column, pivot_row);

        let pivot = augmented[pivot_column][pivot_column];
        if !pivot.is_finite() || pivot.abs().to_bits() == 0.0f64.to_bits() {
            return None;
        }
        for value in &mut augmented[pivot_column][pivot_column..] {
            *value /= pivot;
        }

        let normalised_pivot = augmented[pivot_column].clone();
        for (row_index, row) in augmented.iter_mut().enumerate() {
            if row_index == pivot_column {
                continue;
            }
            let factor = row[pivot_column];
            for (value, pivot_value) in row[pivot_column..].iter_mut().zip(&normalised_pivot[pivot_column..]) {
                *value -= factor * pivot_value;
            }
        }
    }

    Some(Mat::from_fn(kept_width, kept_width, |row, column| {
        let correction = remove_indices
            .iter()
            .enumerate()
            .map(|(removed_column, &source_column)| {
                precision[(keep_indices[row], source_column)] * augmented[removed_column][removed_width + column]
            })
            .sum::<f64>();
        precision[(keep_indices[row], keep_indices[column])] - correction
    }))
}

/// Applies label-indexed and time-indexed forgetting as one recurrence.
///
/// Provenance is [`OracleProvenance::SpecificationFormula`]: this follows the multiplicative composition in (´def:temporal:two-mechanisms´). It deliberately avoids the production `numerics::label_decay_factor` and `numerics::decay_factor` routes and their `f64::powf` evaluation: label steps and whole hours are multiplied one at a time, with only a residual fractional hour evaluated in logarithmic form.
#[must_use]
pub fn decay_recurrence(initial: f64, label_rate: f64, label_count: u64, hourly_rate: f64, elapsed_hours: f64) -> f64 {
    debug_assert!(initial.is_finite());
    debug_assert!(label_rate > 0.0 && label_rate <= 1.0);
    debug_assert!(hourly_rate > 0.0 && hourly_rate <= 1.0);
    debug_assert!(elapsed_hours.is_finite() && elapsed_hours >= 0.0);

    let mut value = initial;
    for _ in 0..label_count {
        value *= label_rate;
    }

    let mut remaining_hours = elapsed_hours;
    loop {
        if remaining_hours < 1.0 {
            break;
        }
        value *= hourly_rate;
        remaining_hours -= 1.0;
    }
    if remaining_hours > 0.0 {
        value *= (remaining_hours * hourly_rate.ln()).exp();
    }
    value
}

/// Population counts for the eight feature-vector block types.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct DimensionBlocks {
    /// Registered identity dimensions.
    pub identity_dimensions: usize,
    /// Active outcome axes.
    pub outcome_axes: usize,
    /// Active outcome axes with spatial features.
    pub spatial_axes: usize,
    /// Declared signal features.
    pub signals: usize,
    /// Registered Sentinels.
    pub sentinels: usize,
    /// Expanded interaction features.
    pub interactions: usize,
    /// Competitive indicators across all identity dimensions.
    pub competitive_indicators: usize,
}

/// Computes the feature-vector width directly from its specification blocks.
///
/// Provenance is [`OracleProvenance::SpecificationFormula`]: this evaluates (´tab:feature:dimension-formula´) block by block. It deliberately avoids `feature::dimension_map::DimensionMap::rebuild` and its shared mutable cursor; every term is formed independently from the supplied population counts before the terms are summed.
#[must_use]
pub const fn dimension_width(blocks: DimensionBlocks) -> usize {
    let identity = (8 + 3 * blocks.outcome_axes) * blocks.identity_dimensions;
    let cross_dimension = if blocks.identity_dimensions > 0 { 8 } else { 0 };
    let sentinel_slots = blocks.sentinels * (61 + 2 * blocks.spatial_axes);
    1 + 15 + identity + cross_dimension + blocks.signals + sentinel_slots + blocks.interactions + blocks.competitive_indicators
}
