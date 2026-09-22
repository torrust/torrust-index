// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`pairwise_rank_oracle_counts_ties_by_pairs`] | harness | A hand-counted set of four class pairs gives two full credits, one half-credit tie, and one inversion, so the pairwise oracle returns five eighths without constructing ranks. |
//! | [`regularised_schur_oracle_matches_scalar_formula`] | harness | A scalar removed block makes the regularised Schur formula exactly representable: `2 - 1 / (1 + 1)` is three halves, and the dense oracle returns that value. |
//! | [`decay_oracle_applies_each_clock_as_a_recurrence`] | harness | Two label steps and one hourly step at one half reduce eight to one, showing that the recurrence applies every step from both independent clocks. |
//! | [`dimension_width_oracle_sums_the_reference_blocks`] | harness | The specification's reference population contributes its eight named block widths to the documented total. |
//! | [`pairwise_rank_oracle_separates_dropped_tie_credit`] | harness | A tied positive-negative pair earns one half from the oracle and zero from an arm that drops tie credit. |
//! | [`regularised_schur_oracle_separates_unregularised_inverse`] | harness | A non-zero diagonal offset makes the oracle's regularised scalar complement differ from an arm that inverts the unregularised block. |
//! | [`decay_oracle_separates_single_application`] | harness | Three composed half-rate steps reduce eight to one while an arm that applies decay only once returns four. |
//! | [`dimension_width_oracle_separates_wrong_block_read`] | harness | Distinct signal and interaction counts make the oracle disagree with an arm that reads interaction width from the signal block. |

//! Focused witnesses for the shared oracle tier. These tests establish the reference computations themselves rather than product behaviour (´dec:harness:oracle-tier´).

use faer::Mat;

use crate::testing::{DimensionBlocks, decay_recurrence, dimension_width, pairwise_rank, regularised_schur_complement};

/// A hand-counted set of four class pairs gives two full credits, one half-credit tie, and one inversion, so the pairwise oracle returns five eighths without constructing ranks.
///
/// ´claim:harness:the-pairwise-rank-oracle-counts-every-class-pair-and-half-of-ties´
/// ´test:crate:pairwise-rank-oracle-counts-ties-by-pairs´
#[test]
fn pairwise_rank_oracle_counts_ties_by_pairs() {
    let scores = [(2.0, true), (0.0, true), (1.0, false), (0.0, false)];
    let rank = pairwise_rank(&scores, 2, 0.0).expect("both classes meet the gate");
    assert_eq!(rank.to_bits(), 0.625f64.to_bits());
}

/// A scalar removed block makes the regularised Schur formula exactly representable: `2 - 1 / (1 + 1)` is three halves, and the dense oracle returns that value.
///
/// ´claim:harness:the-regularised-schur-oracle-evaluates-the-specification-formula´
/// ´test:crate:regularised-schur-oracle-matches-scalar-formula´
#[test]
fn regularised_schur_oracle_matches_scalar_formula() {
    let precision = Mat::from_fn(2, 2, |row, column| [[2.0, 1.0], [1.0, 1.0]][row][column]);
    let result = regularised_schur_complement(&precision, &[1], 1.0).expect("the regularised pivot is non-zero");
    assert_eq!(result.nrows(), 1);
    assert_eq!(result.ncols(), 1);
    assert_eq!(result[(0, 0)].to_bits(), 1.5f64.to_bits());
}

/// Two label steps and one hourly step at one half reduce eight to one, showing that the recurrence applies every step from both independent clocks.
///
/// ´claim:harness:the-decay-oracle-applies-each-clock-as-a-recurrence´
/// ´test:crate:decay-oracle-applies-each-clock-as-a-recurrence´
#[test]
fn decay_oracle_applies_each_clock_as_a_recurrence() {
    let result = decay_recurrence(8.0, 0.5, 2, 0.5, 1.0);
    assert_eq!(result.to_bits(), 1.0f64.to_bits());
}

/// The specification's reference population contributes its eight named block widths to the documented total.
///
/// ´claim:harness:the-dimension-width-oracle-sums-the-specification-blocks´
/// ´test:crate:dimension-width-oracle-sums-the-reference-blocks´
#[test]
fn dimension_width_oracle_sums_the_reference_blocks() {
    let width = dimension_width(DimensionBlocks {
        identity_dimensions: 2,
        outcome_axes: 1,
        spatial_axes: 1,
        signals: 0,
        sentinels: 8,
        interactions: 68,
        competitive_indicators: 20,
    });
    assert_eq!(width, 638);
}

/// A tied positive-negative pair earns one half from the oracle and zero from an arm that drops tie credit.
///
/// ´claim:harness:the-pairwise-rank-oracle-separates-a-dropped-tie-correction´
/// ´test:crate:pairwise-rank-oracle-separates-dropped-tie-credit´
#[test]
fn pairwise_rank_oracle_separates_dropped_tie_credit() {
    let rank = pairwise_rank(&[(0.0, true), (0.0, false)], 1, 0.0).expect("both classes meet the gate");
    let dropped_tie_credit = 0.0f64;

    assert_eq!(rank.to_bits(), 0.5f64.to_bits());
    assert_ne!(rank.to_bits(), dropped_tie_credit.to_bits());
}

/// A non-zero diagonal offset makes the oracle's regularised scalar complement differ from an arm that inverts the unregularised block.
///
/// ´claim:harness:the-regularised-schur-oracle-separates-an-unregularised-inverse´
/// ´test:crate:regularised-schur-oracle-separates-unregularised-inverse´
#[test]
fn regularised_schur_oracle_separates_unregularised_inverse() {
    let precision = Mat::from_fn(2, 2, |row, column| [[2.0, 1.0], [1.0, 1.0]][row][column]);
    let result = regularised_schur_complement(&precision, &[1], 1.0).expect("the regularised pivot is non-zero");
    let unregularised_inverse = 2.0f64 - 1.0 / 1.0;

    assert_eq!(result[(0, 0)].to_bits(), 1.5f64.to_bits());
    assert_ne!(result[(0, 0)].to_bits(), unregularised_inverse.to_bits());
}

/// Three composed half-rate steps reduce eight to one while an arm that applies decay only once returns four.
///
/// ´claim:harness:the-decay-oracle-separates-a-single-decay-application´
/// ´test:crate:decay-oracle-separates-single-application´
#[test]
fn decay_oracle_separates_single_application() {
    let result = decay_recurrence(8.0, 0.5, 2, 0.5, 1.0);
    let decay_applied_once = 8.0f64 * 0.5;

    assert_eq!(result.to_bits(), 1.0f64.to_bits());
    assert_ne!(result.to_bits(), decay_applied_once.to_bits());
}

/// Distinct signal and interaction counts make the oracle disagree with an arm that reads interaction width from the signal block.
///
/// ´claim:harness:the-dimension-width-oracle-separates-a-wrong-block-read´
/// ´test:crate:dimension-width-oracle-separates-wrong-block-read´
#[test]
fn dimension_width_oracle_separates_wrong_block_read() {
    let blocks = DimensionBlocks {
        identity_dimensions: 2,
        outcome_axes: 1,
        spatial_axes: 1,
        signals: 3,
        sentinels: 1,
        interactions: 7,
        competitive_indicators: 5,
    };
    let width = dimension_width(blocks);
    let width_from_wrong_block = dimension_width(DimensionBlocks {
        interactions: blocks.signals,
        ..blocks
    });

    assert_eq!(width, 124);
    assert_ne!(width, width_from_wrong_block);
}
