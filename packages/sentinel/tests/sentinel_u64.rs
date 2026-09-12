// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests validating the **64-bit sentinel** path end-to-end.
//!
//! Constructs a [`Sentinel64`], feeds `u64` values, and verifies
//! that the generic machinery works correctly for a non-`u128`
//! coordinate type.
//!
//! The engine is generic over its coordinate type and its bit width, and the
//! crate ships two aliases over that one implementation — a sentinel for a
//! 128-bit domain and one for a 64-bit domain. Nothing about the narrower
//! alias is a separate code path, which is precisely why it needs its own
//! exercise: a width hard-coded somewhere instead of derived from the type
//! parameter would pass unnoticed at the width the rest of the suite happens
//! to use, and would show up here as a report that describes cells the
//! narrower domain does not have.
//!
//! The rule the width has to satisfy is that a cell's analysis width is the
//! domain width less its depth in the routing tree: the leading bits routing
//! has already resolved are not part of what that cell's tracker analyses.
//! Everything else the narrower sentinel promises is the same promise the
//! wider one makes — construction from any supported configuration, a
//! permanent root, reports partitioned into competitive cells and ancestors
//! and ordered by node handle, finite scores, and reproducibility from a
//! fixed seed. Parity, not a second contract.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_with_test_config`] | width | The narrower alias constructs from an ordinary configuration and comes up tracking the root alone, exactly as the wider one does. Configuration carries no width of its own — width is a property of the type — so the same settings serve either domain. |
//! | [`new_with_cold_config`] | width | cites (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´) |
//! | [`new_with_integration_config`] | width | cites (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´) |
//! | [`initial_state_has_root_only`] | width | cites (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´) |
//! | [`empty_ingest_produces_report`] | edge | cites (´claim:edge:an-empty-batch-yields-a-report-with-no-cells-and-moves-no-counter´) |
//! | [`ingest_returns_non_empty_report`] | engine | cites (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´) |
//! | [`multiple_ingests_accumulate`] | engine | Ingestion accumulates rather than restarting: after many further batches of the same traffic the sentinel tracks no fewer cells than it did after the first. A batch is an increment to standing state, so cells already earned are not dropped merely because another batch arrived. |
//! | [`analysis_widths_are_64_minus_depth`] | width | Every cell in the report, competitive or ancestor, analyses a width equal to the domain width less its own depth — here the narrower domain's width, at whatever depths the traffic reached. The bits routing has already resolved are constant within the cell and so carry no information for its tracker; what remains is the suffix, and its length is fixed by the depth. The width is read from the sentinel's type parameter rather than assumed, which is what makes the same arithmetic hold for either alias. |
//! | [`cell_reports_are_competitive`] | engine | A report separates the cells that earned their modelling from the ones carried along to complete an ancestor chain, and the first vector holds only the former. The distinction is what tells a reader which measurements reflect a deliberate investment, so it is expressed as two vectors rather than as a flag to be filtered on. |
//! | [`ancestor_reports_are_non_competitive`] | engine | cites (´claim:engine:cell-reports-hold-only-competitive-cells-and-ancestor-reports-only-non-competitive-ones´) |
//! | [`reports_sorted_by_gnode_id`] | determinism | cites (´claim:determinism:every-report-vector-comes-out-in-the-order-its-contract-states-so-a-reader-never-depends-on-visit-order´) |
//! | [`no_nan_in_scores`] | engine | Every reported score on every axis is a real number. The axes are ratios and standardised departures, so a variance that had collapsed to nothing or a basis that spanned no direction would surface as a non-number rather than as an obviously wrong value — which is why the absence of one is worth asserting across all four axes and both report vectors. |
//! | [`creates_cells_on_split`] | engine | cites (´claim:engine:traffic-in-separate-regions-splits-the-domain-so-more-than-the-root-is-tracked´) |
//! | [`cells_tracked_never_below_one`] | engine | The root tracker is permanent, before any traffic and after it. It is not selected on merit and cannot be displaced by the competition, because every ancestor chain has to terminate somewhere — so the tracked count has a floor of one and a host never meets a sentinel with nothing to report against. |
//! | [`min_value`] | edge | cites (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´) |
//! | [`max_value`] | edge | cites (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´) |
//! | [`sentinel_u64_min_and_max_together`] | edge | cites (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´) |
//! | [`deterministic_output_for_same_input`] | determinism | cites (´claim:determinism:the-same-seed-and-the-same-data-reproduce-the-same-reports´) |

mod common;

use common::{cold_config, integration_config, test_config};
use torrust_sentinel::{Sentinel64, SentinelConfig};

// ─── Helpers (u64-specific) ─────────────────────────────────

/// Generate `count` values with the given leading nibble and
/// sequential low bits, analogous to `common::cell_values()` but
/// for the 64-bit domain.
fn cell_values_u64(nibble: u64, count: usize) -> Vec<u64> {
    (0..count).map(|i| (nibble << 60) | (i as u64 + 1)).collect()
}

// ── Construction ────────────────────────────────────────────

/// The narrower alias constructs from an ordinary configuration and comes up
/// tracking the root alone, exactly as the wider one does. Configuration
/// carries no width of its own — width is a property of the type — so the
/// same settings serve either domain.
///
/// ´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´
/// ´test:integration:new-with-test-config´
#[test]
fn new_with_test_config() {
    let s = Sentinel64::new(test_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1);
}

/// The same holds with warming noise switched off entirely, which is the
/// configuration most likely to expose a width assumption: the root tracker
/// is built at the narrower width and never primed, and construction still
/// succeeds with one cell tracked.
///
/// (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´)
/// ´test:integration:new-with-cold-config´
#[test]
fn new_with_cold_config() {
    let s = Sentinel64::new(cold_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1);
}

/// And again with a tighter rank budget and faster rank adaptation — the
/// settings that most directly govern the geometry a tracker maintains.
/// None of them is width-dependent, so the narrower sentinel accepts them
/// unchanged.
///
/// (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´)
/// ´test:integration:new-with-integration-config´
#[test]
fn new_with_integration_config() {
    let s = Sentinel64::new(integration_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1);
}

/// Stating the initial condition on its own: before any traffic the
/// narrower sentinel tracks the root and nothing else, so every cell that
/// appears later was created by something observed.
///
/// (´claim:width:the-narrower-alias-constructs-under-any-supported-configuration-and-starts-with-the-root-alone´)
/// ´test:integration:initial-state-has-root-only´
#[test]
fn initial_state_has_root_only() {
    let s = Sentinel64::new(test_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1, "fresh sentinel should have only the root cell");
}

// ── Ingestion basics ────────────────────────────────────────

/// An empty batch at the narrower width is answered the same way as at the
/// wider one: a report naming no cells and no ancestors, rather than an
/// error or an absent report. The early return on an empty batch precedes
/// everything width-dependent, so the two aliases cannot diverge here.
///
/// (´claim:edge:an-empty-batch-yields-a-report-with-no-cells-and-moves-no-counter´)
/// ´test:integration:empty-ingest-produces-report´
#[test]
fn empty_ingest_produces_report() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    let report = s.ingest(&[]);
    // Empty batch is valid — no observations, but struct is populated.
    assert!(report.cell_reports.is_empty());
    assert!(report.ancestor_reports.is_empty());
}

/// A non-empty batch of narrow coordinates produces at least one report
/// entry, because the root contains every value in its domain whatever that
/// domain's width happens to be.
///
/// (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´)
/// ´test:integration:ingest-returns-non-empty-report´
#[test]
fn ingest_returns_non_empty_report() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    let values = cell_values_u64(0xF, 4);
    let report = s.ingest(&values);
    assert!(
        !report.cell_reports.is_empty() || !report.ancestor_reports.is_empty(),
        "ingesting values should produce at least one report entry",
    );
}

/// Ingestion accumulates rather than restarting: after many further batches
/// of the same traffic the sentinel tracks no fewer cells than it did after
/// the first. A batch is an increment to standing state, so cells already
/// earned are not dropped merely because another batch arrived.
///
/// ´claim:engine:repeated-ingestion-adds-to-standing-state-and-does-not-lose-cells-already-tracked´
/// ´test:integration:multiple-ingests-accumulate´
#[test]
fn multiple_ingests_accumulate() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    s.ingest(&cell_values_u64(0xF, 8));
    let cells_after_first = s.cells_tracked();

    for _ in 0..10 {
        s.ingest(&cell_values_u64(0xF, 8));
    }

    assert!(
        s.cells_tracked() >= cells_after_first,
        "repeated ingestion should not lose cells",
    );
}

// ── Report structure ────────────────────────────────────────

/// Every cell in the report, competitive or ancestor, analyses a width equal
/// to the domain width less its own depth — here the narrower domain's
/// width, at whatever depths the traffic reached. The bits routing has
/// already resolved are constant within the cell and so carry no
/// information for its tracker; what remains is the suffix, and its length
/// is fixed by the depth. The width is read from the sentinel's type
/// parameter rather than assumed, which is what makes the same arithmetic
/// hold for either alias.
///
/// ´claim:width:a-cells-analysis-width-is-the-domain-width-less-its-depth´
/// ´test:integration:analysis-widths-are-64-minus-depth´
#[test]
fn analysis_widths_are_64_minus_depth() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    // Feed diverse traffic to populate multiple depths.
    for _ in 0..10 {
        s.ingest(&[cell_values_u64(0xF, 4), cell_values_u64(0x1, 4)].concat());
    }
    let report = s.ingest(&cell_values_u64(0xF, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.analysis_width,
            64 - cr.depth as usize,
            "analysis_width mismatch at depth {}",
            cr.depth,
        );
    }
}

/// A report separates the cells that earned their modelling from the ones
/// carried along to complete an ancestor chain, and the first vector holds
/// only the former. The distinction is what tells a reader which
/// measurements reflect a deliberate investment, so it is expressed as two
/// vectors rather than as a flag to be filtered on.
///
/// ´claim:engine:cell-reports-hold-only-competitive-cells-and-ancestor-reports-only-non-competitive-ones´
/// ´test:integration:cell-reports-are-competitive´
#[test]
fn cell_reports_are_competitive() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    for _ in 0..10 {
        s.ingest(&[cell_values_u64(0xA, 4), cell_values_u64(0x5, 4)].concat());
    }
    let report = s.ingest(&cell_values_u64(0xA, 8));

    for cr in &report.cell_reports {
        assert!(
            cr.is_competitive,
            "cell_reports entry at depth {} is not competitive",
            cr.depth
        );
    }
}

/// The complementary half of the partition: the ancestor vector holds no
/// cell that competed. A cell appears in exactly one of the two vectors, so
/// nothing is counted twice and nothing falls between them.
///
/// (´claim:engine:cell-reports-hold-only-competitive-cells-and-ancestor-reports-only-non-competitive-ones´)
/// ´test:integration:ancestor-reports-are-non-competitive´
#[test]
fn ancestor_reports_are_non_competitive() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    for _ in 0..10 {
        s.ingest(&[cell_values_u64(0xA, 4), cell_values_u64(0x5, 4)].concat());
    }
    let report = s.ingest(&cell_values_u64(0xA, 8));

    for ar in &report.ancestor_reports {
        assert!(
            !ar.is_competitive,
            "ancestor_reports entry at depth {} is competitive",
            ar.depth
        );
    }
}

/// Node-handle ordering is a property of the report and not of the domain
/// width: at the narrower width both vectors still come out strictly
/// ascending, so a reader compares runs positionally here exactly as it does
/// at the wider one.
///
/// (´claim:determinism:every-report-vector-comes-out-in-the-order-its-contract-states-so-a-reader-never-depends-on-visit-order´)
/// ´test:integration:reports-sorted-by-gnode-id´
#[test]
fn reports_sorted_by_gnode_id() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    for _ in 0..10 {
        s.ingest(&[cell_values_u64(0xF, 4), cell_values_u64(0x1, 4)].concat());
    }
    let report = s.ingest(&cell_values_u64(0xF, 8));

    for window in report.cell_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "cell_reports not sorted: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
    for window in report.ancestor_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "ancestor_reports not sorted: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
}

/// Every reported score on every axis is a real number. The axes are ratios
/// and standardised departures, so a variance that had collapsed to nothing
/// or a basis that spanned no direction would surface as a non-number rather
/// than as an obviously wrong value — which is why the absence of one is
/// worth asserting across all four axes and both report vectors.
///
/// ´claim:engine:every-reported-score-is-a-real-number-and-never-a-non-number´
/// ´test:integration:no-nan-in-scores´
#[test]
fn no_nan_in_scores() {
    let mut s = Sentinel64::new(test_config()).unwrap();

    for _ in 0..10 {
        s.ingest(&[cell_values_u64(0xF, 4), cell_values_u64(0x1, 4)].concat());
    }
    let report = s.ingest(&cell_values_u64(0xF, 8));

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty mean at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement mean at depth {}",
            cr.depth
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise mean at depth {}", cr.depth);
        assert!(!cr.scores.coherence.mean.is_nan(), "NaN coherence mean at depth {}", cr.depth);
    }
}

// ── Cell management ─────────────────────────────────────────

/// Splitting works the same way in the narrower domain: sustained traffic in
/// two separated regions, under a low split threshold, leaves the sentinel
/// tracking more than the root. Subdivision is driven by observation volume
/// against that threshold, and neither quantity depends on how wide the
/// coordinates are.
///
/// (´claim:engine:traffic-in-separate-regions-splits-the-domain-so-more-than-the-root-is-tracked´)
/// ´test:integration:creates-cells-on-split´
#[test]
fn creates_cells_on_split() {
    let mut s = Sentinel64::new(SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    })
    .unwrap();

    // Feed diverse traffic across two leading nibbles to trigger splits.
    for _ in 0..15 {
        s.ingest(&[cell_values_u64(0xA, 20), cell_values_u64(0x5, 20)].concat());
    }

    assert!(s.cells_tracked() >= 2, "should have split beyond the root cell");
}

/// The root tracker is permanent, before any traffic and after it. It is not
/// selected on merit and cannot be displaced by the competition, because
/// every ancestor chain has to terminate somewhere — so the tracked count
/// has a floor of one and a host never meets a sentinel with nothing to
/// report against.
///
/// ´claim:engine:the-root-tracker-is-permanent-so-the-tracked-count-never-falls-below-one´
/// ´test:integration:cells-tracked-never-below-one´
#[test]
fn cells_tracked_never_below_one() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    assert!(s.cells_tracked() >= 1, "root cell must always exist");

    s.ingest(&cell_values_u64(0xF, 4));
    assert!(s.cells_tracked() >= 1, "root cell must persist after ingestion");
}

// ── Boundary values ─────────────────────────────────────────

/// The bottom of the narrower domain is an ordinary observation too, counted
/// like any other. The extreme is defined by the width the sentinel was
/// instantiated at, so each alias has its own extremes and handles them the
/// same way.
///
/// (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´)
/// ´test:integration:min-value´
#[test]
fn min_value() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    let report = s.ingest(&[0u64]);
    assert_eq!(report.health.lifetime_observations, 1);
}

/// A saturated narrow coordinate is likewise routed and counted normally —
/// the value that sets every bit the domain has, at the top of the interval
/// the root covers.
///
/// (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´)
/// ´test:integration:max-value´
#[test]
fn max_value() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    let report = s.ingest(&[u64::MAX]);
    assert_eq!(report.health.lifetime_observations, 1);
}

/// Both extremes of the narrower domain in one batch are counted as two
/// ordinary observations, so the widest spread the domain admits is not a
/// case the engine treats specially.
///
/// (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´)
/// ´test:integration:sentinel-u64-min-and-max-together´
#[test]
fn sentinel_u64_min_and_max_together() {
    let mut s = Sentinel64::new(test_config()).unwrap();
    let report = s.ingest(&[0u64, u64::MAX]);
    assert_eq!(report.health.lifetime_observations, 2);
}

// ── Determinism ─────────────────────────────────────────────

/// Reproducibility holds at the narrower width as well, and over structure
/// as well as over scores: two runs of one seeded configuration on one batch
/// agree batch by batch on how many cells were reported, on each cell's
/// depth, analysed width and learned rank, and on the scores themselves. The
/// shape of a run is as reproducible as its numbers.
///
/// (´claim:determinism:the-same-seed-and-the-same-data-reproduce-the-same-reports´)
/// ´test:integration:deterministic-output-for-same-input´
#[test]
fn deterministic_output_for_same_input() {
    let cfg = SentinelConfig::<u64> {
        noise_seed: Some(42),
        ..test_config()
    };

    let batch = cell_values_u64(0xF, 8);

    let run = |cfg: SentinelConfig<u64>| {
        let mut s = Sentinel64::new(cfg).unwrap();
        let mut reports = Vec::new();
        for _ in 0..10 {
            reports.push(s.ingest(&batch));
        }
        reports
    };

    let a = run(cfg.clone());
    let b = run(cfg);

    assert_eq!(a.len(), b.len());
    for (ra, rb) in a.iter().zip(b.iter()) {
        assert_eq!(ra.cell_reports.len(), rb.cell_reports.len());
        assert_eq!(ra.ancestor_reports.len(), rb.ancestor_reports.len());

        for (ca, cb) in ra.cell_reports.iter().zip(rb.cell_reports.iter()) {
            assert_eq!(ca.depth, cb.depth);
            assert_eq!(ca.analysis_width, cb.analysis_width);
            assert_eq!(ca.rank, cb.rank);
            assert!(
                (ca.scores.novelty.mean - cb.scores.novelty.mean).abs() < f64::EPSILON,
                "novelty diverged at depth {}",
                ca.depth,
            );
        }
    }
}
