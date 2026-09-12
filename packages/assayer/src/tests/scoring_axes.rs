// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`scoring_axis_count_matches_the_report_axis_set`] | extract | The arity is recomputed from the axis set the report carries rather than restated: the snapshots `AxisScoreSet` holds are destructured by name, counted, and checked against the constant. The destructuring is exhaustive, so an axis added to or removed from the report cannot compile past this test at all, and a constant moved on its own fails here by name. Every per-axis loop in the extraction reads the constant while indexing that struct's fields, so the two agreeing is the premise the whole extraction rests on. |
//! | [`scoring_axis_count_fixes_every_per_axis_width`] | extract | Every per-axis width in the extraction is recomputed from the arity and checked against the block it governs: six chain z-score views and three chain CUSUM views at four axes to a view, the coordination block's two per-axis maxima standing above its four scalars, and the aggregate block's per-axis maxima and concordances above its seven. The extraction chapter derives each of those widths from a report carrying scores across these axes rather than fixing them independently, so a width that stopped tracking the arity would leave an axis unread at that view or read a neighbour's slot as its own — and every offset downstream would still add up, which is what makes the drift silent without this test. |

//! Crate-level tests for the scoring axis arity (´tab:architecture:sentinel-properties´).
//!
//! One constant now fixes how many scoring axes a batch report carries, and
//! every per-axis width in the extraction is derived from it. Nothing in the
//! compiler makes that constant answerable to the report it describes: the
//! per-axis loops would run happily over three axes or five, reading a
//! neighbour's slot or leaving an axis unread, and the feature vector would
//! still be the width the layout expects. These tests are what hold the
//! constant to its two obligations — agreeing with the axis set the report
//! actually carries, and fixing the widths the extraction chapter derives
//! from it.

use crate::extraction::chain::extract_chain_zscores;
use crate::extraction::coordination::extract_coordination;
use crate::extraction::cusum::extract_chain_cusums;
use crate::feature::aggregate::AGGREGATE_FEATURE_COUNT;
use crate::report::AxisScoreSet;
use crate::types::SCORING_AXIS_COUNT;

/// The arity is recomputed from the axis set the report carries rather than
/// restated: the snapshots `AxisScoreSet` holds are destructured by name,
/// counted, and checked against the constant. The destructuring is exhaustive,
/// so an axis added to or removed from the report cannot compile past this
/// test at all, and a constant moved on its own fails here by name. Every
/// per-axis loop in the extraction reads the constant while indexing that
/// struct's fields, so the two agreeing is the premise the whole extraction
/// rests on.
///
/// ´claim:extract:the-scoring-axis-arity-is-recomputed-from-the-axis-set-the-report-carries´
/// ´test:crate:scoring-axis-count-matches-the-report-axis-set´
#[test]
fn scoring_axis_count_matches_the_report_axis_set() {
    // Exhaustive by construction: no `..` rest pattern, so a field added to or
    // removed from the report's axis set breaks this line rather than passing.
    let AxisScoreSet {
        novelty,
        displacement,
        surprise,
        coherence,
    } = AxisScoreSet::default();

    let axes = [novelty, displacement, surprise, coherence];

    assert_eq!(
        axes.len(),
        SCORING_AXIS_COUNT,
        "the report carries {} scoring axes but the arity says {SCORING_AXIS_COUNT}",
        axes.len(),
    );
}

/// Every per-axis width in the extraction is recomputed from the arity and
/// checked against the block it governs: six chain z-score views and three
/// chain CUSUM views at four axes to a view, the coordination block's two
/// per-axis maxima standing above its four scalars, and the aggregate block's
/// per-axis maxima and concordances above its seven. The extraction chapter
/// derives each of those widths from a report carrying scores across these
/// axes rather than fixing them independently, so a width that stopped
/// tracking the arity would leave an axis unread at that view or read a
/// neighbour's slot as its own — and every offset downstream would still add
/// up, which is what makes the drift silent without this test.
///
/// ´claim:extract:every-per-axis-width-is-recomputed-from-the-scoring-axis-arity´
/// ´test:crate:scoring-axis-count-fixes-every-per-axis-width´
#[test]
fn scoring_axis_count_fixes_every_per_axis_width() {
    // The degenerate inputs are deliberate: each extractor returns its full
    // fixed width from an empty report, so the widths are read from the
    // layout itself rather than from anything a fixture happened to carry.
    let chain_zscores = extract_chain_zscores(&[]);
    assert_eq!(
        chain_zscores.len(),
        6 * SCORING_AXIS_COUNT,
        "six chain z-score views, four axes to a view, twenty-four features",
    );

    let chain_cusums = extract_chain_cusums(&[]);
    assert_eq!(
        chain_cusums.len(),
        3 * SCORING_AXIS_COUNT,
        "three chain CUSUM views, four axes to a view, twelve features",
    );

    // Per-axis maximum coordination z and CUSUM, then concordance, active
    // context fraction, peak context depth and the root context maximum.
    let coordination = extract_coordination(&[], &[0.0; 4], 1, 1);
    assert_eq!(
        coordination.len(),
        2 * SCORING_AXIS_COUNT + 4,
        "two per-axis coordination maxima above four scalars, twelve features",
    );

    // Three distribution slots, then the per-axis maxima and concordances,
    // then the cross-axis product, breadth, coverage and maximum CUSUM.
    assert_eq!(
        AGGREGATE_FEATURE_COUNT,
        3 + 2 * SCORING_AXIS_COUNT + 4,
        "two per-axis aggregate runs between three leading and four trailing slots",
    );
}
