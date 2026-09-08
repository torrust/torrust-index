// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`root_suffix_width_is_full_bit_width`] | suffix | cites (´claim:suffix:a-cells-analysis-width-is-the-domain-width-less-its-depth´) |
//! | [`cell_analysis_width_equals_n_minus_depth`] | suffix | Across every cell a graph under traffic has produced, the width a cell analyses is exactly the domain width less its depth. The leading bits its depth stands for were fixed by routing and are identical for every value that reaches it, so modelling them would add a constant column and no information; the width is a consequence of position rather than a per-cell setting anyone can get wrong. |
//! | [`geometry_dim_equals_analysis_width`] | suffix | The space a cell's model works in is its own suffix, not the domain: the dimension reported with its scoring geometry is the cell's analysis width at every depth the tree reaches. The two are not independently maintained numbers that happen to agree — the tracker is constructed at the cell's width — so a host reading the geometry is reading the same fact as one reading the width. |
//! | [`geometry_cap_is_min_of_dim_and_max_rank`] | suffix | How much structure a cell may learn is limited by its own suffix as well as by configuration: the ceiling is whichever of the two is smaller. A model cannot hold more directions than the space it lives in has, so a cell deep enough to be narrower than the configured maximum is capped by its depth instead — the configured maximum is a budget, never a promise of capacity. |
//! | [`residual_dof_equals_dim_minus_rank`] | suffix | What is left over for a cell to be surprised by is its width less the structure it has already learned. Those residual directions are the room in which an unexplained departure can register at all, so the same absolute departure means more in a narrow cell than in a wide one — and a cell whose model has grown to fill its width has no room left, which is precisely the degenerate case the geometry lets a host detect rather than hiding. |
//! | [`deeper_cells_have_smaller_suffix_width`] | suffix | cites (´claim:suffix:a-cells-analysis-width-is-the-domain-width-less-its-depth´) |
//! | [`cell_reports_carry_correct_suffix_widths`] | suffix | The width a cell analysed travels out with its numbers: each entry in a batch report states the width it was computed at, and that width still agrees with the cell's depth and with the geometry the scores came from. Scores from different depths are not commensurable, so a report that carried only the numbers would invite a host to compare them as though they were. |
//! | [`scores_are_valid_across_suffix_widths`] | suffix | Every score axis yields a real number at every width the tree produced, across a graph warmed on separated ranges and then driven into one of them. Narrow geometries are where the divisions in the scoring arithmetic come closest to degenerating, so this is the property that lets a host treat a report as data rather than checking each figure for a non-number first. |
//! | [`noise_injected_at_every_suffix_width`] | suffix | No cell begins scoring cold. Every cell the graph created under traffic has synthetic observations behind it, generated at that cell's own width, so the warm-up schedule reaches cells born deep in the tree and not only the root it started from. A cell that had never seen anything would find its first real batch infinitely surprising, and the sentinel would report the arrival of a new region as an anomaly in it. |

//! Suffix analysis — which part of an observation a cell actually models
//! (§ALGO S-3.2).
//!
//! A cell's position in the G-tree is a prefix of the coordinate: routing has
//! already decided the leading bits by the time a value arrives, and within
//! the cell those bits are the same for every observation. They therefore
//! carry no statistical content and are not handed to the cell's model at all.
//! What the model sees is the suffix behind them, so a cell's analysis width
//! is the domain width less its depth — the deeper a cell, the narrower and
//! more specialised the thing it is modelling, and the shallower a cell, the
//! wider the view it keeps.
//!
//! That single identity propagates outward rather than being recomputed
//! anywhere. It fixes the working dimension of the cell's subspace tracker,
//! and through it the geometry the scores are read against: the rank a cell
//! can reach is capped by its own width where that is narrower than the
//! configured maximum, and the residual degrees of freedom against which
//! novelty is judged are whatever the width leaves once the modelled rank is
//! taken out. A narrow cell is thus not a wide cell with fewer observations;
//! it is a smaller geometry, and its scores mean something correspondingly
//! smaller.
//!
//! Because the width varies from cell to cell, it has to travel with the
//! numbers. Every cell report carries the width it was computed at, so a host
//! comparing two cells knows it is comparing different geometries, and the
//! machinery around the model — warming a new cell with synthetic data,
//! producing finite scores on every axis — has to work at every width the tree
//! can produce rather than only at the root's.

mod common;

use common::{ScenarioBuilder, assert_invariants, cell_values, seeded_sentinel, test_config};
use torrust_sentinel::Sentinel128;

// ═══════════════════════════════════════════════════════════
//  Suffix width identity
// ═══════════════════════════════════════════════════════════

/// The base case of the depth identity: the root has had nothing resolved by
/// routing, so it models the entire domain width. A sentinel that has observed
/// nothing therefore already has one cell watching everything, which is why
/// there is never a value the sentinel cannot score.
///
/// (´claim:suffix:a-cells-analysis-width-is-the-domain-width-less-its-depth´)
/// ´test:integration:root-suffix-width-is-full-bit-width´
#[test]
fn root_suffix_width_is_full_bit_width() {
    let s = Sentinel128::new(test_config()).unwrap();

    let gnodes = s.cell_gnodes();
    assert!(!gnodes.is_empty(), "sentinel must have at least the root cell");

    let root = s.inspect_cell(gnodes[0]).unwrap();
    assert_eq!(root.depth, 0);
    assert_eq!(root.analysis_width, 128);
}

/// Across every cell a graph under traffic has produced, the width a cell
/// analyses is exactly the domain width less its depth. The leading bits its
/// depth stands for were fixed by routing and are identical for every value
/// that reaches it, so modelling them would add a constant column and no
/// information; the width is a consequence of position rather than a
/// per-cell setting anyone can get wrong.
///
/// ´claim:suffix:a-cells-analysis-width-is-the-domain-width-less-its-depth´
/// ´test:integration:cell-analysis-width-equals-n-minus-depth´
#[test]
fn cell_analysis_width_equals_n_minus_depth() {
    let mut s = seeded_sentinel();
    s.ingest(&cell_values(0xA, 200));

    for &gnode in &s.cell_gnodes() {
        let ins = s.inspect_cell(gnode).unwrap();
        let expected = 128 - ins.depth as usize;
        assert_eq!(
            ins.analysis_width, expected,
            "depth {}: expected analysis_width={expected}, got {}",
            ins.depth, ins.analysis_width,
        );
    }
}

/// The space a cell's model works in is its own suffix, not the domain: the
/// dimension reported with its scoring geometry is the cell's analysis width
/// at every depth the tree reaches. The two are not independently maintained
/// numbers that happen to agree — the tracker is constructed at the cell's
/// width — so a host reading the geometry is reading the same fact as one
/// reading the width.
///
/// ´claim:suffix:a-cells-model-works-in-its-own-suffix-space-and-not-the-whole-domain´
/// ´test:integration:geometry-dim-equals-analysis-width´
#[test]
fn geometry_dim_equals_analysis_width() {
    let mut s = seeded_sentinel();
    s.ingest(&cell_values(0xA, 200));

    for &gnode in &s.cell_gnodes() {
        let ins = s.inspect_cell(gnode).unwrap();
        assert_eq!(
            ins.geometry.dim, ins.analysis_width,
            "depth {}: geometry.dim={} != analysis_width={}",
            ins.depth, ins.geometry.dim, ins.analysis_width,
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  Geometry cap
// ═══════════════════════════════════════════════════════════

/// How much structure a cell may learn is limited by its own suffix as well as
/// by configuration: the ceiling is whichever of the two is smaller. A model
/// cannot hold more directions than the space it lives in has, so a cell deep
/// enough to be narrower than the configured maximum is capped by its depth
/// instead — the configured maximum is a budget, never a promise of capacity.
///
/// ´claim:suffix:a-cells-structural-ceiling-is-its-own-width-when-that-is-narrower-than-the-configured-maximum´
/// ´test:integration:geometry-cap-is-min-of-dim-and-max-rank´
#[test]
fn geometry_cap_is_min_of_dim_and_max_rank() {
    let cfg = test_config();
    let max_rank = cfg.max_rank;
    let mut s = Sentinel128::new(cfg).unwrap();
    s.ingest(&cell_values(0xF, 200));

    for &gnode in &s.cell_gnodes() {
        let ins = s.inspect_cell(gnode).unwrap();
        let expected_cap = ins.geometry.dim.min(max_rank);
        assert_eq!(
            ins.geometry.cap, expected_cap,
            "depth {}: expected cap={expected_cap}, got {}",
            ins.depth, ins.geometry.cap,
        );
    }
}

/// What is left over for a cell to be surprised by is its width less the
/// structure it has already learned. Those residual directions are the room in
/// which an unexplained departure can register at all, so the same absolute
/// departure means more in a narrow cell than in a wide one — and a cell whose
/// model has grown to fill its width has no room left, which is precisely the
/// degenerate case the geometry lets a host detect rather than hiding.
///
/// ´claim:suffix:what-a-cell-can-still-be-surprised-by-is-its-width-less-the-structure-it-has-learned´
/// ´test:integration:residual-dof-equals-dim-minus-rank´
#[test]
fn residual_dof_equals_dim_minus_rank() {
    let mut s = seeded_sentinel();
    s.ingest(&cell_values(0xA, 200));

    for &gnode in &s.cell_gnodes() {
        let ins = s.inspect_cell(gnode).unwrap();
        let expected_dof = ins.geometry.dim - ins.rank;
        assert_eq!(
            ins.geometry.residual_dof, expected_dof,
            "depth {}: expected residual_dof={expected_dof}, got {}",
            ins.depth, ins.geometry.residual_dof,
        );
    }
}

// ═══════════════════════════════════════════════════════════
//  Deeper cells have narrower suffixes
// ═══════════════════════════════════════════════════════════

/// The ordering consequence of the same identity: sort a graph's cells by
/// depth and their widths fall strictly the other way. Refinement is a trade —
/// each split buys a more specific region at the cost of a narrower view
/// inside it — so no cell in the tree is both deeper and wider than another.
///
/// (´claim:suffix:a-cells-analysis-width-is-the-domain-width-less-its-depth´)
/// ´test:integration:deeper-cells-have-smaller-suffix-width´
#[test]
fn deeper_cells_have_smaller_suffix_width() {
    let mut s = seeded_sentinel();
    s.ingest(&cell_values(0xF, 200));

    let mut widths: Vec<(u32, usize)> = s
        .cell_gnodes()
        .iter()
        .map(|&g| {
            let ins = s.inspect_cell(g).unwrap();
            (ins.depth, ins.analysis_width)
        })
        .collect();
    widths.sort_by_key(|&(depth, _)| depth);

    for pair in widths.windows(2) {
        let (d1, w1) = pair[0];
        let (d2, w2) = pair[1];
        if d2 > d1 {
            assert!(
                w2 < w1,
                "depth {d1} (width {w1}) should be wider than depth {d2} (width {w2})",
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════
//  Report suffix widths
// ═══════════════════════════════════════════════════════════

/// The width a cell analysed travels out with its numbers: each entry in a
/// batch report states the width it was computed at, and that width still
/// agrees with the cell's depth and with the geometry the scores came from.
/// Scores from different depths are not commensurable, so a report that
/// carried only the numbers would invite a host to compare them as though they
/// were.
///
/// ´claim:suffix:a-batch-report-states-the-width-each-cell-was-scored-at´
/// ´test:integration:cell-reports-carry-correct-suffix-widths´
#[test]
fn cell_reports_carry_correct_suffix_widths() {
    let mut s = seeded_sentinel();
    let report = s.ingest(&cell_values(0xF, 100));

    assert_invariants(&s, &report);

    for cr in &report.cell_reports {
        let expected = 128 - cr.depth as usize;
        assert_eq!(
            cr.analysis_width, expected,
            "cell report depth {}: expected analysis_width={expected}, got {}",
            cr.depth, cr.analysis_width,
        );
        assert_eq!(
            cr.geometry.dim, expected,
            "cell report depth {}: geometry.dim={} != {expected}",
            cr.depth, cr.geometry.dim,
        );
    }
}

/// Every score axis yields a real number at every width the tree produced,
/// across a graph warmed on separated ranges and then driven into one of them.
/// Narrow geometries are where the divisions in the scoring arithmetic come
/// closest to degenerating, so this is the property that lets a host treat a
/// report as data rather than checking each figure for a non-number first.
///
/// ´claim:suffix:every-score-axis-yields-a-real-number-at-every-width-the-tree-produces´
/// ´test:integration:scores-are-valid-across-suffix-widths´
#[test]
fn scores_are_valid_across_suffix_widths() {
    let (mut s, _) = ScenarioBuilder::new()
        .config(test_config())
        .seed_range(0xF, 50)
        .seed_range(0x1, 50)
        .warm_batches(3)
        .build_with_reports();

    let report = s.ingest(&cell_values(0xF, 100));

    assert_invariants(&s, &report);

    for cr in &report.cell_reports {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement at depth {}",
            cr.depth
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise at depth {}", cr.depth);
        assert!(!cr.scores.coherence.mean.is_nan(), "NaN coherence at depth {}", cr.depth);
    }
}

// ═══════════════════════════════════════════════════════════
//  Noise at all suffix widths
// ═══════════════════════════════════════════════════════════

/// No cell begins scoring cold. Every cell the graph created under traffic has
/// synthetic observations behind it, generated at that cell's own width, so
/// the warm-up schedule reaches cells born deep in the tree and not only the
/// root it started from. A cell that had never seen anything would find its
/// first real batch infinitely surprising, and the sentinel would report the
/// arrival of a new region as an anomaly in it.
///
/// ´claim:suffix:no-cell-starts-scoring-cold-however-narrow-its-suffix´
/// ´test:integration:noise-injected-at-every-suffix-width´
#[test]
fn noise_injected_at_every_suffix_width() {
    let mut s = seeded_sentinel();
    s.ingest(&cell_values(0xA, 200));

    for &gnode in &s.cell_gnodes() {
        let ins = s.inspect_cell(gnode).unwrap();
        assert!(
            ins.maturity.noise_observations > 0,
            "cell at depth {} (width {}) received no noise",
            ins.depth,
            ins.analysis_width,
        );
    }
}
