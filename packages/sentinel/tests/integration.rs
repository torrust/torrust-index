// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! End-to-end integration tests for the sentinel.
//!
//! These tests exercise multi-subsystem interactions that span the full
//! sentinel lifecycle.  Each test constructs a sentinel, feeds realistic
//! traffic, and verifies cross-cutting concerns — things that only become
//! visible when construction, ingestion, scoring, coordination, decay,
//! and health reporting all run together.
//!
//! Focused unit-level concerns live in dedicated files:
//!
//! | Concern               | File                            |
//! |-----------------------|---------------------------------|
//! | Public API contracts  | `api.rs`                        |
//! | Report structure      | `report_structure.rs`           |
//! | Reset behaviour       | `api.rs`, `edge_cases.rs`, `noise.rs` |
//! | CUSUM accumulation    | `coverage_matrix.rs`            |
//! | Warm-vs-cold scoring  | `noise.rs`, `health.rs`         |
//! | Spatial decay         | `spatial_decay.rs`              |
//! | Ancestor chains       | `ancestor_chain.rs`             |
//! | Invariants            | `invariants.rs`                 |
//! | Determinism           | `determinism.rs`                |
//!
//! What a lifecycle test is for is agreement between the layers. The spatial
//! substrate splits the domain as traffic concentrates; the selector invests
//! in the cells that earned it and closes them under ancestry; each cell's
//! tracker scores the suffix left to it; and a second tier scores the pattern
//! across cells. A disagreement between those layers — a report naming a cell
//! the engine no longer tracks, a width that does not match a depth, an
//! ancestor listed as though it had competed — is invisible to any one
//! subsystem's own tests and shows up only when they run together over a
//! whole run.
//!
//! Because the sentinel measures and the host decides, the end-to-end claims
//! here are comparative rather than categorical. An unfamiliar batch scores
//! higher than the familiar one immediately before it; unfamiliarity
//! sustained over many batches accumulates instead of being forgotten;
//! decay lowers spatial standing without disturbing the observation record or
//! the trackers' learned state. No threshold is asserted anywhere, because
//! the engine does not own one.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`single_range_lifecycle`] | engine | A sentinel taken through a full life — constructed, seeded, warmed, driven at steady state, then handed a structurally unfamiliar batch — holds every structural invariant at each of those stages, not merely at the end. The health snapshot afterwards still describes a working engine: trackers are live, and the smallest learned rank is at least one, so no cell has collapsed to a model with no directions in it. |
//! | [`multi_range_lifecycle`] | engine | Traffic concentrated in two well-separated regions of the domain drives the spatial layer to split, and the sentinel ends up tracking more than the root — separate models for separate structure, which is the whole point of a hierarchy. The invariants continue to hold through steady state and through an unfamiliar batch confined to one of the two regions, so the cells coexist rather than interfering. |
//! | [`anomalous_batch_elevates_novelty`] | engine | Once a sentinel has settled on the structure it keeps being shown, a batch built on a different bit pattern scores higher than the ordinary batch immediately before it. The comparison is against that neighbour rather than against a fixed number, because the engine reports how far an observation departs from what this cell learned and leaves the question of how far is too far to the host. |
//! | [`anomalous_batch_elevates_cusum`] | engine | Unfamiliarity that persists across many batches accumulates: the drift accumulator stands higher after a long run of unfamiliar traffic than it did at the baseline batch. A single surprising batch and a regime that has genuinely shifted look alike instant by instant, and the accumulator is what tells them apart — a small departure repeated is allowed to add up rather than being forgotten each round. |
//! | [`decay_then_ingest_maintains_invariants`] | engine | Decay applied in the middle of a run attenuates accumulated spatial standing and nothing else: ingestion continues afterwards with every invariant intact and the observation counter still climbing. Temporal policy belongs to the host, and the engine implements it by lowering what cells have earned rather than by discarding what they have learned, so a decayed sentinel is a going concern and not a half-reset one. |
//! | [`inspect_cells_after_multi_range_traffic`] | width | cites (´claim:width:a-cells-analysis-width-is-the-domain-width-less-its-depth´) |
//! | [`coordination_activates_with_multi_range_traffic`] | engine | Once two separate regions are being modelled in earnest, a second tier of reporting appears without the host asking for it: a context that covers several cells at once and carries scores of its own, finite like any other. A pattern spread across sibling cells is invisible to each of them individually, so the engine models the pattern itself as soon as there are enough cells for one to exist. |

mod common;

use common::{ScenarioBuilder, anomalous_values, assert_invariants, cell_values, integration_config, max_cusum, max_novelty_z};
use torrust_sentinel::SentinelConfig;

// ── Full lifecycle ──────────────────────────────────────────

/// A sentinel taken through a full life — constructed, seeded, warmed,
/// driven at steady state, then handed a structurally unfamiliar batch —
/// holds every structural invariant at each of those stages, not merely at
/// the end. The health snapshot afterwards still describes a working engine:
/// trackers are live, and the smallest learned rank is at least one, so no
/// cell has collapsed to a model with no directions in it.
///
/// ´claim:engine:a-sentinel-holds-its-invariants-through-a-whole-lifecycle-from-first-seed-to-anomaly´
/// ´test:integration:single-range-lifecycle´
#[test]
fn single_range_lifecycle() {
    let (mut s, warm_reports) = ScenarioBuilder::new()
        .seed_range(0xF, 8)
        .warm_batches(15)
        .build_with_reports();

    // Invariants hold throughout warm-up.
    for r in &warm_reports {
        assert_invariants(&s, r);
    }
    assert!(s.cells_tracked() >= 1);

    // Steady-state ingestion.
    for _ in 0..10 {
        let report = s.ingest(&cell_values(0xF, 8));
        assert_invariants(&s, &report);
    }

    // Anomalous batch.
    let anomaly_report = s.ingest(&anomalous_values(0xF, 8));
    assert_invariants(&s, &anomaly_report);
    assert!(!anomaly_report.ancestor_reports.is_empty());

    // Health check.
    let h = s.health();
    assert!(h.active_trackers >= 1);
    assert!(h.rank_distribution.min >= 1);
}

/// Traffic concentrated in two well-separated regions of the domain drives
/// the spatial layer to split, and the sentinel ends up tracking more than
/// the root — separate models for separate structure, which is the whole
/// point of a hierarchy. The invariants continue to hold through steady
/// state and through an unfamiliar batch confined to one of the two regions,
/// so the cells coexist rather than interfering.
///
/// ´claim:engine:traffic-in-separate-regions-splits-the-domain-so-more-than-the-root-is-tracked´
/// ´test:integration:multi-range-lifecycle´
#[test]
fn multi_range_lifecycle() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..integration_config()
        })
        .seed_range(0xF, 12)
        .seed_range(0x1, 12)
        .warm_batches(15)
        .build();

    // Two disjoint ranges should produce more than just the root cell.
    assert!(s.cells_tracked() > 1, "expected multiple cells from disjoint ranges");

    // Steady-state.
    for _ in 0..10 {
        let batch: Vec<u128> = [cell_values(0xF, 8), cell_values(0x1, 8)].concat();
        let report = s.ingest(&batch);
        assert_invariants(&s, &report);
    }

    // Anomaly in one range only.
    let mixed: Vec<u128> = [anomalous_values(0xF, 8), cell_values(0x1, 8)].concat();
    let report = s.ingest(&mixed);
    assert_invariants(&s, &report);

    let h = s.health();
    assert!(h.lifetime_observations > 0);
    assert!(h.active_trackers >= 2);
}

// ── Anomaly detection (end-to-end) ──────────────────────────

/// Once a sentinel has settled on the structure it keeps being shown, a
/// batch built on a different bit pattern scores higher than the ordinary
/// batch immediately before it. The comparison is against that neighbour
/// rather than against a fixed number, because the engine reports how far an
/// observation departs from what this cell learned and leaves the question
/// of how far is too far to the host.
///
/// ´claim:engine:a-structurally-unfamiliar-batch-scores-higher-than-the-familiar-one-that-preceded-it´
/// ´test:integration:anomalous-batch-elevates-novelty´
#[test]
fn anomalous_batch_elevates_novelty() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 16).warm_batches(20).build();

    // Final normal batch as baseline.
    let normal = s.ingest(&cell_values(0xA, 8));
    assert_invariants(&s, &normal);

    // Anomalous batch.
    let anomaly = s.ingest(&anomalous_values(0xA, 8));
    assert_invariants(&s, &anomaly);

    let normal_z = max_novelty_z(&normal);
    let anomaly_z = max_novelty_z(&anomaly);
    assert!(
        anomaly_z > normal_z,
        "anomalous batch should produce higher novelty z-score: \
         anomaly={anomaly_z:.4}, normal={normal_z:.4}"
    );
}

/// Unfamiliarity that persists across many batches accumulates: the drift
/// accumulator stands higher after a long run of unfamiliar traffic than it
/// did at the baseline batch. A single surprising batch and a regime that
/// has genuinely shifted look alike instant by instant, and the accumulator
/// is what tells them apart — a small departure repeated is allowed to add
/// up rather than being forgotten each round.
///
/// ´claim:engine:sustained-unfamiliarity-accumulates-instead-of-being-forgotten-batch-by-batch´
/// ´test:integration:anomalous-batch-elevates-cusum´
#[test]
fn anomalous_batch_elevates_cusum() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            max_rank: 1,
            cusum_slow_decay: 0.96,
            cusum_coord_slow_decay: 0.96,
            ..integration_config()
        })
        .seed_range(0xF, 8)
        .warm_batches(30)
        .build();

    let baseline = s.ingest(&cell_values(0xF, 8));
    assert_invariants(&s, &baseline);
    let cusum_before = max_cusum(&baseline);

    // Sustained anomalous traffic.
    let mut last_report = baseline;
    for _ in 0..15 {
        last_report = s.ingest(&anomalous_values(0xF, 8));
        assert_invariants(&s, &last_report);
    }

    let cusum_after = max_cusum(&last_report);
    assert!(
        cusum_after > cusum_before,
        "CUSUM should grow under sustained anomalous traffic: \
         before={cusum_before:.6}, after={cusum_after:.6}"
    );
}

// ── Decay → ingest round-trip ───────────────────────────────

/// Decay applied in the middle of a run attenuates accumulated spatial
/// standing and nothing else: ingestion continues afterwards with every
/// invariant intact and the observation counter still climbing. Temporal
/// policy belongs to the host, and the engine implements it by lowering what
/// cells have earned rather than by discarding what they have learned, so a
/// decayed sentinel is a going concern and not a half-reset one.
///
/// ´claim:engine:decay-attenuates-standing-only-so-ingestion-continues-and-the-observation-record-keeps-growing´
/// ´test:integration:decay-then-ingest-maintains-invariants´
#[test]
fn decay_then_ingest_maintains_invariants() {
    let mut s = ScenarioBuilder::new()
        .seed_range(0xF, 8)
        .seed_range(0x1, 8)
        .warm_batches(10)
        .build();

    // Pre-decay steady state.
    for _ in 0..5 {
        let batch: Vec<u128> = [cell_values(0xF, 4), cell_values(0x1, 4)].concat();
        let r = s.ingest(&batch);
        assert_invariants(&s, &r);
    }

    let obs_before = s.lifetime_observations();

    // Decay: uniform 50% attenuation.
    s.decay(0.5, 0.0);

    // Post-decay ingestion must succeed with invariants intact.
    for _ in 0..10 {
        let batch: Vec<u128> = [cell_values(0xF, 4), cell_values(0x1, 4)].concat();
        let r = s.ingest(&batch);
        assert_invariants(&s, &r);
    }

    assert!(
        s.lifetime_observations() > obs_before,
        "observations should continue accumulating after decay"
    );
}

// ── Inspection after realistic traffic ──────────────────────

/// After traffic deep enough to create cells at several depths, every handle
/// the engine lists is still inspectable, and each cell's analysis width is
/// the domain width less its own depth — the rule holds at whatever depths
/// real splitting produced, not just at the root. Each has at least one
/// learned direction, and the root is among them, as it must be for the
/// ancestor chains to terminate.
///
/// (´claim:width:a-cells-analysis-width-is-the-domain-width-less-its-depth´)
/// ´test:integration:inspect-cells-after-multi-range-traffic´
#[test]
fn inspect_cells_after_multi_range_traffic() {
    let mut s = ScenarioBuilder::new()
        .seed_range(0xF, 8)
        .seed_range(0x1, 8)
        .warm_batches(10)
        .build();

    for _ in 0..10 {
        s.ingest(&[cell_values(0xF, 4), cell_values(0x1, 4)].concat());
    }

    let gnodes = s.cell_gnodes();
    assert_ne!(gnodes, [] as [torrust_sentinel::GNodeId; 0]);

    let mut found_root = false;
    for gnode in &gnodes {
        let insp = s.inspect_cell(*gnode).expect("tracked cell must be inspectable");
        assert_eq!(insp.analysis_width, 128 - insp.depth as usize);
        assert!(insp.rank >= 1, "tracked cell should have rank >= 1");
        if insp.depth == 0 {
            found_root = true;
        }
    }
    assert!(found_root, "root cell (depth 0) must always be tracked");
}

// ── Coordination (end-to-end) ───────────────────────────────

/// Once two separate regions are being modelled in earnest, a second tier of
/// reporting appears without the host asking for it: a context that covers
/// several cells at once and carries scores of its own, finite like any
/// other. A pattern spread across sibling cells is invisible to each of them
/// individually, so the engine models the pattern itself as soon as there
/// are enough cells for one to exist.
///
/// ´claim:engine:a-second-tier-of-reporting-appears-once-several-cells-are-modelled-together´
/// ´test:integration:coordination-activates-with-multi-range-traffic´
#[test]
fn coordination_activates_with_multi_range_traffic() {
    let mut s = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            per_sample_scores: true,
            ..integration_config()
        })
        .seed_range(0xF, 20)
        .seed_range(0x1, 20)
        .warm_batches(20)
        .build();

    // Continue feeding both ranges to keep competitive cells active.
    let mut saw_coordination = false;
    for _ in 0..30 {
        let batch: Vec<u128> = [cell_values(0xF, 8), cell_values(0x1, 8)].concat();
        let report = s.ingest(&batch);

        if !report.coordination_reports.is_empty() {
            saw_coordination = true;
            // Coordination reports should have valid scores.
            for cr in &report.coordination_reports {
                assert!(cr.cells_reporting >= 2);
                assert!(!cr.scores.novelty.mean.is_nan());
            }
            break;
        }
    }

    assert!(
        saw_coordination,
        "coordination should activate with two disjoint competitive ranges"
    );
}
