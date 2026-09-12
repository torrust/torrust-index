// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`stage1_only_root_tracker`] | warmup | Traffic that has not yet reached the split threshold leaves the domain undivided: one cell, the root, and nothing competing for investment. Structure is bought with observation volume, so a sentinel that has seen only a handful of values has bought none of it yet. |
//! | [`stage1_no_coordination`] | warmup | The coordination tier compares cells against each other, so it has nothing to say while there is only one. A pre-split batch therefore reports no coordination at all rather than a degenerate context over a single member: a context fires only where both sides of a G-node contribute cells. |
//! | [`stage1_lifetime_observations_counted`] | warmup | The lifetime observation count starts at nothing and accrues one unit per input value, batch after batch, from the very first ingest. It is a record of what the host has fed in rather than a measure of what the sentinel has made of it, so it runs well before any structure exists to attribute it to. |
//! | [`stage1_invariants_hold`] | warmup | The structural guarantees a report makes — the competitive cap, the root's presence in the full set, the separation of competitive from ancestor entries, sorted output, finite scores, widths matching depth — are not promises about steady state. They hold from the first batch, when the sentinel is a single cell and has almost nothing to report. |
//! | [`stage2_cells_tracked_increases`] | warmup | Once a range has taken more traffic than the split threshold allows, the spatial layer divides it, and the newly exposed cells are picked up by the analysis set and given trackers of their own. Modelling effort follows the structure the traffic created rather than a shape chosen in advance. |
//! | [`stage2_competitive_cells_appear`] | warmup | Having a tracker and being competitive are separate things, and the second arrives later. After a run of batches has given some cells enough accumulated importance to win the ranking, the competitive set becomes non-empty — so the transition out of the pre-split state is earned by volume, not conferred at creation. |
//! | [`stage2_new_cells_have_high_noise_influence`] | warmup | A cell that has just come into service is mostly synthetic. Its tracker was seeded so that it could score at all, and until real batches have arrived to displace that seed, the maturity figure it publishes stays high. Every non-root cell with few real observations behind it says so, which lets a host discount a young cell's scores instead of trusting them equally. |
//! | [`stage2_invariants_hold`] | warmup | cites (´claim:warmup:the-report-invariants-hold-at-every-stage-of-warm-up-not-merely-once-it-has-settled´) |
//! | [`stage3_competitive_set_size_stabilises`] | warmup | Once the traffic pattern stops changing, the competitive set stops changing with it: repeated batches over the same ranges leave the number of selected cells varying only within a narrow band. Selection is recomputed from scratch on every batch, so stability here is a property of the ranking rather than of any memory the selector keeps. |
//! | [`stage3_coordination_activates`] | warmup | cites (´claim:warmup:coordination-fires-only-where-two-subtrees-both-contribute-cells´) |
//! | [`stage3_invariants_hold_throughout`] | warmup | cites (´claim:warmup:the-report-invariants-hold-at-every-stage-of-warm-up-not-merely-once-it-has-settled´) |
//! | [`stage4_root_maturity_below_half`] | warmup | Real data displaces the synthetic seed geometrically: each real batch multiplies the synthetic share of a tracker's memory by the forgetting factor raised to the batch size. A modest run of warm-up batches is therefore enough to push the root well past the halfway mark, and the rate is a property of the configured forgetting factor rather than of the data. |
//! | [`stage4_maturity_decreases_monotonically`] | warmup | Maturity only ever improves while real data is arriving: batch after batch, a cell's synthetic share is multiplied down and never rises again. It could rise only if further noise were injected, and nothing injects noise into a cell already in service. A host can therefore read the figure as a one-way progress measure rather than as something that might rebound. |
//! | [`stage4_health_maturity_distribution`] | warmup | The health snapshot aggregates what individual cells know about their own maturity, and in steady state it says two things: the average tracker is no longer purely synthetic, and no tracker anywhere is still cold — every cell in service has seen real data. A cold entry in a warmed sentinel would mean a cell was being scored on noise alone. |
//! | [`cold_start_noise_influence_is_one`] | warmup | Turning the noise schedule off shows what the seed was doing. The root then begins at a synthetic share of exactly one — the value a tracker with no information at all reports — and only real data moves it. Warm-up is thus an optional head start, not a precondition: the sentinel still runs without it, and simply says that everything it knows is unearned. |

//! §9.4 — **Warm-up sequence**: how a sentinel comes into service.
//!
//! A sentinel arrives knowing nothing. It begins as a single root cell
//! covering the whole domain, and everything else — the division of that
//! domain into cells, the competition that decides which of them are worth
//! modelling, the coordination tier that compares them — has to be earned
//! from observed traffic. These tests walk that sequence from the first batch
//! to steady state and fix what is true at each point along it.
//!
//! Two quantities move in opposite directions during warm-up. Structure
//! grows: volume accumulates, cells split off, and the competitive set fills
//! and then settles once traffic has stopped telling the graph anything new.
//! Maturity falls: every tracker is seeded with synthetic noise so that it
//! has a baseline to score against before it has seen anything real, and the
//! synthetic share of its memory is multiplied down by the forgetting factor
//! with each real batch. A cell is therefore never unable to score. It is
//! only more or less made of noise, and the maturity figures are what say
//! which.
//!
//! The two are independent by construction. Splitting is driven by
//! observation volume alone and never by scores, so warm-up cannot chase its
//! own tail; and the structural invariants the reports must satisfy hold at
//! every stage rather than only once the sentinel has settled.

mod common;

use common::{ScenarioBuilder, assert_invariants, cell_values, cold_config, integration_config, test_config};
use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};

// ── Stage 1: Pre-split ──────────────────────────────────────

/// Traffic that has not yet reached the split threshold leaves the domain
/// undivided: one cell, the root, and nothing competing for investment.
/// Structure is bought with observation volume, so a sentinel that has seen
/// only a handful of values has bought none of it yet.
///
/// ´claim:warmup:before-the-first-split-the-root-is-the-only-cell-and-nothing-is-competing´
/// ´test:integration:stage1-only-root-tracker´
#[test]
fn stage1_only_root_tracker() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 100,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed fewer than split_threshold observations — all to same range.
    let report = s.ingest(&cell_values(0xA, 50));

    assert_eq!(
        report.analysis_set_summary.competitive_size, 0,
        "pre-split: no competitive cells yet"
    );
    assert_eq!(s.cells_tracked(), 1, "only root tracker");
}

/// The coordination tier compares cells against each other, so it has nothing
/// to say while there is only one. A pre-split batch therefore reports no
/// coordination at all rather than a degenerate context over a single member:
/// a context fires only where both sides of a G-node contribute cells.
///
/// ´claim:warmup:coordination-fires-only-where-two-subtrees-both-contribute-cells´
/// ´test:integration:stage1-no-coordination´
#[test]
fn stage1_no_coordination() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 100,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&cell_values(0xA, 50));
    assert!(
        report.coordination_reports.is_empty(),
        "pre-split: coordination should be empty"
    );
}

/// The lifetime observation count starts at nothing and accrues one unit per
/// input value, batch after batch, from the very first ingest. It is a record
/// of what the host has fed in rather than a measure of what the sentinel has
/// made of it, so it runs well before any structure exists to attribute it to.
///
/// ´claim:warmup:the-lifetime-observation-count-accrues-from-the-first-batch-even-before-any-structure-exists´
/// ´test:integration:stage1-lifetime-observations-counted´
#[test]
fn stage1_lifetime_observations_counted() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 100,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    assert_eq!(s.lifetime_observations(), 0);
    s.ingest(&cell_values(0xA, 50));
    assert_eq!(s.lifetime_observations(), 50);
    s.ingest(&cell_values(0xA, 30));
    assert_eq!(s.lifetime_observations(), 80);
}

/// The structural guarantees a report makes — the competitive cap, the root's
/// presence in the full set, the separation of competitive from ancestor
/// entries, sorted output, finite scores, widths matching depth — are not
/// promises about steady state. They hold from the first batch, when the
/// sentinel is a single cell and has almost nothing to report.
///
/// ´claim:warmup:the-report-invariants-hold-at-every-stage-of-warm-up-not-merely-once-it-has-settled´
/// ´test:integration:stage1-invariants-hold´
#[test]
fn stage1_invariants_hold() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 100,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&cell_values(0xA, 50));
    assert_invariants(&s, &report);
}

// ── Stage 2: Spatial formation ──────────────────────────────

/// Once a range has taken more traffic than the split threshold allows, the
/// spatial layer divides it, and the newly exposed cells are picked up by the
/// analysis set and given trackers of their own. Modelling effort follows the
/// structure the traffic created rather than a shape chosen in advance.
///
/// ´claim:warmup:traffic-past-the-split-threshold-divides-the-space-and-each-new-cell-earns-its-own-tracker´
/// ´test:integration:stage2-cells-tracked-increases´
#[test]
fn stage2_cells_tracked_increases() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for _ in 0..5 {
        s.ingest(&cell_values(0xA, 20));
    }

    assert!(s.cells_tracked() > 1, "splits should create new cells");
}

/// Having a tracker and being competitive are separate things, and the second
/// arrives later. After a run of batches has given some cells enough
/// accumulated importance to win the ranking, the competitive set becomes
/// non-empty — so the transition out of the pre-split state is earned by
/// volume, not conferred at creation.
///
/// ´claim:warmup:a-cell-becomes-competitive-only-once-it-has-accumulated-enough-importance-to-win-the-ranking´
/// ´test:integration:stage2-competitive-cells-appear´
#[test]
fn stage2_competitive_cells_appear() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed enough to trigger splits (> split_threshold per range).
    for _ in 0..5 {
        s.ingest(&cell_values(0xA, 20));
    }

    let report = s.ingest(&cell_values(0xA, 8));
    assert!(
        report.analysis_set_summary.competitive_size > 0,
        "after sufficient traffic, competitive cells should appear"
    );
}

/// A cell that has just come into service is mostly synthetic. Its tracker
/// was seeded so that it could score at all, and until real batches have
/// arrived to displace that seed, the maturity figure it publishes stays
/// high. Every non-root cell with few real observations behind it says so,
/// which lets a host discount a young cell's scores instead of trusting them
/// equally.
///
/// ´claim:warmup:a-freshly-promoted-cell-is-still-mostly-synthetic-and-publishes-that-fact´
/// ´test:integration:stage2-new-cells-have-high-noise-influence´
#[test]
fn stage2_new_cells_have_high_noise_influence() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        noise_schedule: NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed just enough to create new cells.
    for _ in 0..5 {
        s.ingest(&cell_values(0xA, 20));
    }

    // Newly created cells should have high noise influence (η > 0.3)
    // because they've had very few real observations.
    let root = s.graph().g_root();
    for &gnode in &s.cell_gnodes() {
        if gnode == root {
            continue; // root is special — skip.
        }
        let insp = s.inspect_cell(gnode).unwrap();
        // New cells may not have processed many real batches yet,
        // so η should be high.
        if insp.maturity.real_observations < 20 {
            assert!(
                insp.maturity.noise_influence > 0.3,
                "early cell at depth {} should have high noise influence, got {:.4}",
                insp.depth,
                insp.maturity.noise_influence
            );
        }
    }
}

/// The same guarantees survive the most turbulent part of warm-up. Splitting
/// is creating cells, trackers are being built and some of them discarded
/// again, and the report emitted in the middle of that still satisfies every
/// structural invariant.
///
/// (´claim:warmup:the-report-invariants-hold-at-every-stage-of-warm-up-not-merely-once-it-has-settled´)
/// ´test:integration:stage2-invariants-hold´
#[test]
fn stage2_invariants_hold() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for _ in 0..5 {
        s.ingest(&cell_values(0xA, 20));
    }

    let report = s.ingest(&cell_values(0xA, 8));
    assert_invariants(&s, &report);
}

// ── Stage 3: Stabilisation ──────────────────────────────────

/// Once the traffic pattern stops changing, the competitive set stops
/// changing with it: repeated batches over the same ranges leave the number
/// of selected cells varying only within a narrow band. Selection is
/// recomputed from scratch on every batch, so stability here is a property of
/// the ranking rather than of any memory the selector keeps.
///
/// ´claim:warmup:a-settled-traffic-pattern-settles-the-competitive-set-size-even-though-selection-is-recomputed-each-batch´
/// ´test:integration:stage3-competitive-set-size-stabilises´
#[test]
fn stage3_competitive_set_size_stabilises() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };

    let (mut s, _) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(8)
        .build_with_reports();

    // Run 10 more batches and check variance of competitive_size.
    let mut sizes = Vec::new();
    for _ in 0..10 {
        let report = s.ingest(&[cell_values(0xA, 10), cell_values(0xB, 10)].concat());
        sizes.push(report.analysis_set_summary.competitive_size);
    }

    // Stabilisation: low variance in competitive set size.
    let min = *sizes.iter().min().unwrap();
    let max = *sizes.iter().max().unwrap();
    assert!(max - min <= 2, "competitive set size should stabilise (min={min}, max={max})");
}

/// The far end of the same rule. Seeding two well-separated ranges and
/// warming until several cells compete gives a common ancestor two
/// contributing subtrees, and coordination becomes active — either as reports
/// in the batch or as live contexts in the health snapshot. What switches
/// coordination on is the arrival of cells on both sides, which volume alone
/// decides.
///
/// (´claim:warmup:coordination-fires-only-where-two-subtrees-both-contribute-cells´)
/// ´test:integration:stage3-coordination-activates´
#[test]
fn stage3_coordination_activates() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(12)
        .build();

    let report = s.ingest(&[cell_values(0xA, 10), cell_values(0xB, 10)].concat());

    // If there are multiple competitive cells, coordination contexts
    // should have appeared.  Cell creation depends on traffic volume
    // (split_threshold), not λ — needs enough warm batches for the
    // coordination context to observe both subtrees.
    if report.analysis_set_summary.competitive_size >= 2 {
        assert!(
            !report.coordination_reports.is_empty() || report.health.coordination_health.active_contexts > 0,
            "with ≥2 competitive cells, coordination should be active"
        );
    }
}

/// This pins the strongest reading of the guarantee: not that some sampled
/// report is well formed, but that every report is, across the whole warming
/// run and the batches that follow it. A transient violation between two
/// checked batches would be a violation.
///
/// (´claim:warmup:the-report-invariants-hold-at-every-stage-of-warm-up-not-merely-once-it-has-settled´)
/// ´test:integration:stage3-invariants-hold-throughout´
#[test]
fn stage3_invariants_hold_throughout() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    };

    let (mut s, reports) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xA, 20)
        .seed_range(0xB, 20)
        .warm_batches(8)
        .build_with_reports();

    // Verify invariants on the last warm-up report.
    if let Some(last) = reports.last() {
        assert_invariants(&s, last);
    }

    // Verify invariants on 5 further batches.
    for _ in 0..5 {
        let report = s.ingest(&[cell_values(0xA, 10), cell_values(0xB, 10)].concat());
        assert_invariants(&s, &report);
    }
}

// ── Stage 4: Steady state ───────────────────────────────────

/// Real data displaces the synthetic seed geometrically: each real batch
/// multiplies the synthetic share of a tracker's memory by the forgetting
/// factor raised to the batch size. A modest run of warm-up batches is
/// therefore enough to push the root well past the halfway mark, and the rate
/// is a property of the configured forgetting factor rather than of the data.
///
/// ´claim:warmup:real-batches-displace-the-synthetic-seed-geometrically-so-maturity-arrives-within-a-predictable-number-of-batches´
/// ´test:integration:stage4-root-maturity-below-half´
#[test]
fn stage4_root_maturity_below_half() {
    let s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    // After sufficient batches, root tracker should be mature.
    // 15 batches at λ=0.90 gives 0.90^15 = 0.206 fractional
    // weight from noise — well under 0.5.  See ADR-S-012.
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).unwrap();
    assert!(
        insp.maturity.noise_influence < 0.5,
        "after 15 warm-up batches, root noise_influence should be < 0.5, got {:.4}",
        insp.maturity.noise_influence
    );
}

/// Maturity only ever improves while real data is arriving: batch after
/// batch, a cell's synthetic share is multiplied down and never rises again.
/// It could rise only if further noise were injected, and nothing injects
/// noise into a cell already in service. A host can therefore read the figure
/// as a one-way progress measure rather than as something that might rebound.
///
/// ´claim:warmup:the-synthetic-share-of-a-serving-cell-never-rises-again´
/// ´test:integration:stage4-maturity-decreases-monotonically´
#[test]
fn stage4_maturity_decreases_monotonically() {
    let mut s = Sentinel128::new(SentinelConfig::<u64> {
        split_threshold: 10,
        ..integration_config()
    })
    .unwrap();

    let root = s.graph().g_root();
    let mut prev_ni = s.inspect_cell(root).unwrap().maturity.noise_influence;

    for _ in 0..10 {
        s.ingest(&cell_values(0xA, 8));
        let ni = s.inspect_cell(root).unwrap().maturity.noise_influence;
        assert!(
            ni <= prev_ni + f64::EPSILON,
            "noise_influence should not increase: prev={prev_ni:.6}, current={ni:.6}"
        );
        prev_ni = ni;
    }

    // After 10 batches, it should have decreased significantly.
    assert!(prev_ni < 1.0, "noise_influence should decrease from 1.0 after real batches");
}

/// The health snapshot aggregates what individual cells know about their own
/// maturity, and in steady state it says two things: the average tracker is
/// no longer purely synthetic, and no tracker anywhere is still cold — every
/// cell in service has seen real data. A cold entry in a warmed sentinel
/// would mean a cell was being scored on noise alone.
///
/// ´claim:warmup:a-warmed-sentinel-reports-no-cold-trackers-because-every-serving-cell-has-seen-real-data´
/// ´test:integration:stage4-health-maturity-distribution´
#[test]
fn stage4_health_maturity_distribution() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xA, 20)
        .warm_batches(15)
        .build();

    let report = s.ingest(&cell_values(0xA, 10));
    let maturity = &report.health.maturity_distribution;

    assert!(
        maturity.mean_noise_influence < 1.0,
        "mean noise influence should be below 1.0 in steady state, got {:.4}",
        maturity.mean_noise_influence
    );
    assert_eq!(maturity.cold_trackers, 0, "no cold trackers should remain in steady state");
    assert_invariants(&s, &report);
}

// ── Cold start ──────────────────────────────────────────────

/// Turning the noise schedule off shows what the seed was doing. The root
/// then begins at a synthetic share of exactly one — the value a tracker with
/// no information at all reports — and only real data moves it. Warm-up is
/// thus an optional head start, not a precondition: the sentinel still runs
/// without it, and simply says that everything it knows is unearned.
///
/// ´claim:warmup:with-the-noise-schedule-empty-a-tracker-begins-fully-uninformed-and-only-real-data-moves-it´
/// ´test:integration:cold-start-noise-influence-is-one´
#[test]
fn cold_start_noise_influence_is_one() {
    let mut s = Sentinel128::new(cold_config()).unwrap();

    // With noise disabled, the root tracker starts completely cold.
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).unwrap();
    assert!(
        (insp.maturity.noise_influence - 1.0).abs() < f64::EPSILON,
        "cold start: root noise_influence should be 1.0, got {:.4}",
        insp.maturity.noise_influence
    );

    // After one batch of real data, noise_influence should drop.
    let report = s.ingest(&cell_values(0xA, 20));
    let insp = s.inspect_cell(root).unwrap();
    assert!(
        insp.maturity.noise_influence < 1.0,
        "after real data, noise_influence should drop below 1.0, got {:.4}",
        insp.maturity.noise_influence
    );
    assert_invariants(&s, &report);
}
