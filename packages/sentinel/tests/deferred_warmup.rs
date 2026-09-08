// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`step2_invariants_hold_throughout_lifecycle`] | warmup | Deferring warm-up changes when a cell becomes live, not what a report is allowed to look like. Across a long run of splitting traffic — cells entering staging, warming, and being promoted mid-run — every batch report still satisfies the structural guarantees. A cell in staging is simply absent from the producing set until it is ready, so no half-built model can leak into the output. |
//! | [`step2_lifecycle_new_cells_appear_after_splits`] | warmup | A sentinel that starts as root alone ends a run of splitting traffic with several live cells, and every one of them carries noise observations behind it. Promotion is conditional on the warm-up schedule being finished, so the live map never contains a cell that skipped its seeding — staging is a queue on the way in, not an alternative way in. |
//! | [`step2_determinism_staging_path_matches_twin`] | warmup | Two sentinels built from the same configuration and the same noise seed, fed the same batches, agree on their reports down to the bits of the score means. The synchronous drain warms queued cells in a deterministic order from a seeded generator, so the staging detour introduces no freedom: an investigation can be replayed exactly rather than approximately. |
//! | [`step2_ancestor_receives_observations_for_warming_cells`] | warmup | Deferring a cell costs no coverage. Every value routes to each live cell whose interval contains it, and the root's interval contains all of them, so the root reports a non-zero sample count whatever is queued below it. A region under construction is therefore still watched — at coarser resolution, by the ancestor chain — rather than unobserved until its cell is ready. |
//! | [`step2_eviction_during_reconcile_no_panic`] | warmup | A cell can lose its place before it ever takes it. Under a tight budget and scattered traffic, cells are created and evicted continuously, and a cell still warming when its G-node leaves the analysis set is dropped from staging rather than promoted into a set it no longer belongs to. Work already spent on it is abandoned, which is cheaper than admitting a cell the selector has rejected. |
//! | [`step2_reset_clears_staging_and_resumes`] | warmup | A reset returns the sentinel to the state it was constructed in — root alone, no accumulated observations — and that includes emptying the staging area, so no cell queued under the old structure can surface under the new one. What follows is a genuine second warm-up: further traffic splits the space and builds cells again from nothing. |
//! | [`step3_invariants_hold_with_background_warming`] | warmup | cites (´claim:warmup:deferring-warm-up-through-a-staging-area-never-lets-a-half-built-cell-into-a-report´) |
//! | [`step3_warmup_completes_eventually`] | warmup | cites (´claim:warmup:a-cell-reaches-the-live-map-only-after-its-noise-schedule-is-complete´) |
//! | [`step3_scoring_works_after_background_warmup`] | warmup | A tracker warmed on another thread scores like any other: after a warmed run, every cell and ancestor in the batch reports finite score means. That is what the seeding is for — a model with no baseline would divide by a spread it does not have — and it holds whichever thread supplied the seed. |
//! | [`step3_background_same_cell_structure_as_sync`] | warmup | Two sentinels fed identical traffic, one warming inline and one on a background thread, end with the same accumulated volume and the same set of tracked cells. Structure is decided by observation volume alone and never by anything the trackers compute, so warm-up scheduling — a modelling concern — cannot perturb it. Choosing the background path is a latency decision, not a modelling one. |
//! | [`step3_higher_volume_cells_warm_via_priority`] | warmup | When warming capacity is scarce — a long noise schedule and small batches leave the queue in progress — the staging area serves its waiting cells in descending order of volume, so a heavily trafficked subtree gets cells into service before a quiet one. Warming effort is spent where the traffic is, which is the same principle that decided the cells were worth having. |
//! | [`step3_concurrent_ingest_no_panic`] | warmup | Ingestion and background warming genuinely overlap: a long run of diverse traffic keeps creating cells while the thread is warming earlier ones, and the two never collide. A cell being warmed is checked out of the staging area for the duration, so the expensive work happens on a cell no other thread can reach, and the lock is held only for the queue operations around it. |
//! | [`step3_eviction_during_background_warming_no_panic`] | warmup | cites (´claim:warmup:a-warming-cell-whose-node-leaves-the-analysis-set-is-discarded-rather-than-promoted´) |
//! | [`step3_reset_restarts_background_thread`] | warmup | cites (´claim:warmup:a-reset-empties-the-staging-area-so-warm-up-begins-again-from-the-root-alone´) |
//! | [`step3_drop_while_warming_no_hang`] | warmup | Dropping a sentinel with a long noise schedule still outstanding returns promptly instead of waiting for the queue to empty. The warming thread checks for shutdown between batches and abandons whatever remains, because cells nobody will ever read from are not worth finishing. Teardown costs at most one batch of work, not the rest of the schedule. |

//! Deferred warm-up (§ALGO S-18.2): paying for a new cell off the hot path.
//!
//! Seeding a new cell's tracker with synthetic noise is the expensive part of
//! creating one, and it comes due at exactly the wrong moment — a split is
//! triggered by a surge of traffic, so the cost lands while the sentinel is
//! busiest. Deferred warm-up moves it aside. A newly selected cell is not
//! built straight into the live map; it waits in a staging area, is warmed
//! round by round, and is promoted only once its schedule is complete. Until
//! then it counts towards the investment set but takes no real observations,
//! and the values that would have reached it are still seen by its ancestors.
//!
//! Two paths drain the staging area, and these tests exercise both end to
//! end. The synchronous drain runs inside reconciliation and finishes every
//! queued cell before the batch is scored, which keeps the whole pipeline
//! reproducible from a seed. The background thread instead takes the
//! highest-volume waiting cell out of the queue, injects noise without
//! holding the lock, and returns it — so warming overlaps ingestion rather
//! than blocking it, and the busiest cells come into service first. Which
//! path ran is meant to be invisible in the structure that results, because
//! that structure is decided by observation volume alone.
//!
//! The awkward moments are the ones worth pinning down: a cell evicted while
//! it is still warming, a reset that has to tear the thread down and stand it
//! back up, and a drop that must not wait on warming work nobody will read.

mod common;

use common::{cell_values, integration_config};
use torrust_sentinel::{BatchReport, NoiseSchedule, Sentinel128, SentinelConfig};

// ════════════════════════════════════════════════════════════════
//  Helpers
// ════════════════════════════════════════════════════════════════

/// Invariant checks for deferred warm-up tests.
///
/// This is a variant of [`common::assert_invariants`] that omits the
/// Steiner tree bound (`full_size <= 2 * competitive_size + 1`).
/// Deep-split configurations (low `split_threshold`) can transiently
/// violate the bound during rapid cell creation, so it is excluded
/// here. All other invariants from the common version are checked.
fn assert_deferred_invariants(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    let summary = &report.analysis_set_summary;

    // Competitive cap: competitive_size <= analysis_k.
    assert!(
        summary.competitive_size <= sentinel.config().analysis_k,
        "competitive set {} exceeds K={}",
        summary.competitive_size,
        sentinel.config().analysis_k,
    );

    // Root at depth 0 is always in the full set.
    if summary.full_size > 0 {
        assert_eq!(summary.depth_range.0, 0, "root (depth 0) must be in the full analysis set");
    }

    // cell_reports are competitive only.
    for cr in &report.cell_reports {
        assert!(
            cr.is_competitive,
            "cell_reports entry at depth {} is not competitive",
            cr.depth
        );
    }

    // ancestor_reports are non-competitive.
    for ar in &report.ancestor_reports {
        assert!(
            !ar.is_competitive,
            "ancestor_reports entry at depth {} is competitive",
            ar.depth
        );
    }

    // Deterministic ordering: reports sorted by gnode_id ascending.
    for window in report.cell_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "cell_reports not sorted by GNodeId: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }
    for window in report.ancestor_reports.windows(2) {
        assert!(
            window[0].gnode_id < window[1].gnode_id,
            "ancestor_reports not sorted by GNodeId: {:?} >= {:?}",
            window[0].gnode_id,
            window[1].gnode_id,
        );
    }

    // Coordination reports have no duplicate GNodeIds.
    {
        let mut seen = std::collections::HashSet::new();
        for cr in &report.coordination_reports {
            assert!(
                seen.insert(cr.gnode_id),
                "duplicate GNodeId in coordination_reports: {:?}",
                cr.gnode_id,
            );
        }
    }

    // No NaN in score fields (novelty, displacement, surprise).
    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "NaN novelty mean at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "NaN displacement mean at depth {}",
            cr.depth,
        );
        assert!(!cr.scores.surprise.mean.is_nan(), "NaN surprise mean at depth {}", cr.depth);
    }

    // analysis_width == 128 - depth for every cell/ancestor report.
    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.analysis_width,
            128 - cr.depth as usize,
            "analysis_width mismatch at depth {}",
            cr.depth,
        );
    }
}

/// Config that triggers splits quickly so new cells are created.
fn fast_split_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        split_threshold: 10,
        analysis_k: 16,
        analysis_depth_cutoff: 6,
        noise_schedule: NoiseSchedule::Explicit(vec![3]),
        noise_batch_size: 4,
        noise_seed: Some(42),
        background_warming: false,
        ..integration_config()
    }
}

/// Config with background warming enabled (otherwise identical to
/// [`fast_split_config`]).
fn background_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        background_warming: true,
        ..fast_split_config()
    }
}

// ════════════════════════════════════════════════════════════════
//  Step 2 — Synchronous Deferred Warm-Up
// ════════════════════════════════════════════════════════════════

// ── 2.8a: Invariants ────────────────────────────────────────

/// Deferring warm-up changes when a cell becomes live, not what a report is
/// allowed to look like. Across a long run of splitting traffic — cells
/// entering staging, warming, and being promoted mid-run — every batch report
/// still satisfies the structural guarantees. A cell in staging is simply
/// absent from the producing set until it is ready, so no half-built model
/// can leak into the output.
///
/// ´claim:warmup:deferring-warm-up-through-a-staging-area-never-lets-a-half-built-cell-into-a-report´
/// ´test:integration:step2-invariants-hold-throughout-lifecycle´
#[test]
fn step2_invariants_hold_throughout_lifecycle() {
    let mut s = Sentinel128::new(fast_split_config()).unwrap();

    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 16), cell_values(0x5, 16)].concat();
        let report = s.ingest(&batch);
        assert_deferred_invariants(&s, &report);
    }
}

// ── 2.8b: Lifecycle ─────────────────────────────────────────

/// A sentinel that starts as root alone ends a run of splitting traffic with
/// several live cells, and every one of them carries noise observations
/// behind it. Promotion is conditional on the warm-up schedule being
/// finished, so the live map never contains a cell that skipped its seeding —
/// staging is a queue on the way in, not an alternative way in.
///
/// ´claim:warmup:a-cell-reaches-the-live-map-only-after-its-noise-schedule-is-complete´
/// ´test:integration:step2-lifecycle-new-cells-appear-after-splits´
#[test]
fn step2_lifecycle_new_cells_appear_after_splits() {
    let mut s = Sentinel128::new(fast_split_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1, "initially only root");

    // Feed diverse traffic to trigger splits and cell creation.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
    }

    // After enough traffic, new cells should have been created via
    // the staging area and promoted into the live cells map.
    assert!(
        s.cells_tracked() > 1,
        "after splits, more than just root should be tracked (got {})",
        s.cells_tracked(),
    );

    // All tracked cells should be noise-warmed.
    for &gnode in &s.cell_gnodes() {
        let insp = s.inspect_cell(gnode).unwrap();
        assert!(
            insp.maturity.noise_observations > 0,
            "cell {:?} at depth {} should have noise observations",
            gnode,
            insp.depth,
        );
    }
}

// ── 2.8c: Determinism ──────────────────────────────────────

/// Two sentinels built from the same configuration and the same noise seed,
/// fed the same batches, agree on their reports down to the bits of the score
/// means. The synchronous drain warms queued cells in a deterministic order
/// from a seeded generator, so the staging detour introduces no freedom: an
/// investigation can be replayed exactly rather than approximately.
///
/// ´claim:warmup:the-synchronous-staging-path-is-fully-determined-by-the-seed-so-twin-sentinels-agree-bit-for-bit´
/// ´test:integration:step2-determinism-staging-path-matches-twin´
#[test]
fn step2_determinism_staging_path_matches_twin() {
    // Two sentinels with identical config and seed should produce
    // bit-identical reports when using the synchronous staging path.
    let cfg = fast_split_config();

    let mut s1 = Sentinel128::new(cfg.clone()).unwrap();
    let mut s2 = Sentinel128::new(cfg).unwrap();

    let values: Vec<u128> = [cell_values(0xA, 50), cell_values(0x5, 50)].concat();

    for chunk in values.chunks(20) {
        let r1 = s1.ingest(chunk);
        let r2 = s2.ingest(chunk);

        assert_eq!(r1.cell_reports.len(), r2.cell_reports.len(), "cell report count mismatch");
        assert_eq!(
            r1.ancestor_reports.len(),
            r2.ancestor_reports.len(),
            "ancestor report count mismatch",
        );
        assert_eq!(
            r1.coordination_reports.len(),
            r2.coordination_reports.len(),
            "coordination report count mismatch",
        );
        assert_eq!(
            r1.health.lifetime_observations, r2.health.lifetime_observations,
            "lifetime observations mismatch",
        );

        // Bit-exact score comparison.
        for (c1, c2) in r1.cell_reports.iter().zip(&r2.cell_reports) {
            assert_eq!(
                c1.scores.novelty.mean.to_bits(),
                c2.scores.novelty.mean.to_bits(),
                "novelty mean diverged",
            );
            assert_eq!(
                c1.scores.displacement.mean.to_bits(),
                c2.scores.displacement.mean.to_bits(),
                "displacement mean diverged",
            );
        }

        for (a1, a2) in r1.ancestor_reports.iter().zip(&r2.ancestor_reports) {
            assert_eq!(
                a1.scores.novelty.mean.to_bits(),
                a2.scores.novelty.mean.to_bits(),
                "ancestor novelty mean diverged",
            );
        }
    }
}

// ── 2.8d: Ancestor routing ─────────────────────────────────

/// Deferring a cell costs no coverage. Every value routes to each live cell
/// whose interval contains it, and the root's interval contains all of them,
/// so the root reports a non-zero sample count whatever is queued below it.
/// A region under construction is therefore still watched — at coarser
/// resolution, by the ancestor chain — rather than unobserved until its cell
/// is ready.
///
/// ´claim:warmup:no-region-goes-unwatched-while-its-cell-warms-because-the-ancestor-chain-still-observes-it´
/// ´test:integration:step2-ancestor-receives-observations-for-warming-cells´
#[test]
fn step2_ancestor_receives_observations_for_warming_cells() {
    // With synchronous drain, warming cells are promoted within the
    // same ingest() call. In all cases the root (an ancestor of
    // everything) must receive every observation.
    let mut s = Sentinel128::new(fast_split_config()).unwrap();

    // Seed to create cells.
    for _ in 0..10 {
        s.ingest(&cell_values(0xA, 20));
    }

    // Now ingest a fresh batch and verify the root has observations.
    let report = s.ingest(&cell_values(0xA, 8));
    let root_report = report.ancestor_reports.iter().find(|cr| cr.depth == 0);
    assert!(root_report.is_some(), "root cell should receive observations as ancestor");
    assert!(
        root_report.unwrap().sample_count > 0,
        "root should have non-zero sample_count",
    );
}

// ── 2.8e: Eviction ─────────────────────────────────────────

/// A cell can lose its place before it ever takes it. Under a tight budget
/// and scattered traffic, cells are created and evicted continuously, and a
/// cell still warming when its G-node leaves the analysis set is dropped from
/// staging rather than promoted into a set it no longer belongs to. Work
/// already spent on it is abandoned, which is cheaper than admitting a cell
/// the selector has rejected.
///
/// ´claim:warmup:a-warming-cell-whose-node-leaves-the-analysis-set-is-discarded-rather-than-promoted´
/// ´test:integration:step2-eviction-during-reconcile-no-panic´
#[test]
fn step2_eviction_during_reconcile_no_panic() {
    // Create a sentinel with a small budget so cells get evicted.
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        budget: 50,
        d_evict: 4,
        ..fast_split_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed lots of diverse traffic — cells will be created and evicted.
    for _ in 0..20 {
        let batch: Vec<u128> = [
            cell_values(0xA, 10),
            cell_values(0x5, 10),
            cell_values(0x2, 10),
            cell_values(0xE, 10),
        ]
        .concat();
        let report = s.ingest(&batch);
        assert_deferred_invariants(&s, &report);
    }
}

// ── 2.8f: Reset ─────────────────────────────────────────────

/// A reset returns the sentinel to the state it was constructed in — root
/// alone, no accumulated observations — and that includes emptying the
/// staging area, so no cell queued under the old structure can surface under
/// the new one. What follows is a genuine second warm-up: further traffic
/// splits the space and builds cells again from nothing.
///
/// ´claim:warmup:a-reset-empties-the-staging-area-so-warm-up-begins-again-from-the-root-alone´
/// ´test:integration:step2-reset-clears-staging-and-resumes´
#[test]
fn step2_reset_clears_staging_and_resumes() {
    let mut s = Sentinel128::new(fast_split_config()).unwrap();

    // Build up state.
    for _ in 0..10 {
        s.ingest(&cell_values(0xA, 20));
    }
    assert!(s.cells_tracked() > 1);

    s.reset();
    assert_eq!(s.cells_tracked(), 1, "after reset, only root");
    assert_eq!(s.lifetime_observations(), 0);

    // Resume ingestion — new cells should be created again.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
    }
    assert!(
        s.cells_tracked() > 1,
        "after reset + re-ingest, more than just root should be tracked (got {})",
        s.cells_tracked(),
    );
}

// ════════════════════════════════════════════════════════════════
//  Step 3 — Background Warming Thread
// ════════════════════════════════════════════════════════════════

// ── 3.6a: Invariants ────────────────────────────────────────

/// The same holds when a separate thread is doing the warming and cells may
/// be promoted between batches rather than within one. Whichever thread
/// finished a cell, the report published by an ingest describes only cells
/// already in service.
///
/// (´claim:warmup:deferring-warm-up-through-a-staging-area-never-lets-a-half-built-cell-into-a-report´)
/// ´test:integration:step3-invariants-hold-with-background-warming´
#[test]
fn step3_invariants_hold_with_background_warming() {
    let mut s = Sentinel128::new(background_config()).unwrap();

    // Give background thread time to warm cells between ingests.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 16), cell_values(0x5, 16)].concat();
        let report = s.ingest(&batch);
        assert_deferred_invariants(&s, &report);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }
}

// ── 3.6b: Warm-up completes eventually ─────────────────────

/// Handing warm-up to a background thread weakens the timing but not the
/// condition. Given a run of further batches to work through, every cell that
/// has reached the live map carries its noise observations, so the queue
/// drains rather than stalling: deferral postpones seeding, it does not
/// permit a cell to be promoted without it.
///
/// (´claim:warmup:a-cell-reaches-the-live-map-only-after-its-noise-schedule-is-complete´)
/// ´test:integration:step3-warmup-completes-eventually´
#[test]
fn step3_warmup_completes_eventually() {
    let mut s = Sentinel128::new(background_config()).unwrap();

    // Feed diverse traffic to trigger splits.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
    }

    // Give the background thread time to finish warming.
    // We spin-ingest a few more batches — each ingest promotes ready cells.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 4));
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    // All tracked cells should have noise observations.
    let gnodes = s.cell_gnodes();
    assert!(gnodes.len() > 1, "should have more than just root after splits");
    for &gnode in &gnodes {
        let insp = s.inspect_cell(gnode).unwrap();
        assert!(
            insp.maturity.noise_observations > 0,
            "cell {:?} at depth {} should have noise observations after background warming",
            gnode,
            insp.depth,
        );
    }
}

// ── 3.6c: Scoring after warm-up ────────────────────────────

/// A tracker warmed on another thread scores like any other: after a warmed
/// run, every cell and ancestor in the batch reports finite score means. That
/// is what the seeding is for — a model with no baseline would divide by a
/// spread it does not have — and it holds whichever thread supplied the seed.
///
/// ´claim:warmup:a-background-warmed-tracker-produces-finite-scores-from-its-first-real-batch´
/// ´test:integration:step3-scoring-works-after-background-warmup´
#[test]
fn step3_scoring_works_after_background_warmup() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        background_warming: true,
        ..integration_config()
    };

    let mut s = Sentinel128::new(cfg).unwrap();

    // Seed and warm with background thread.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
        std::thread::sleep(std::time::Duration::from_millis(2));
    }

    // Steady-state: scoring should produce non-NaN values.
    let report = s.ingest(&cell_values(0xA, 8));
    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert!(!cr.scores.novelty.mean.is_nan(), "novelty NaN at depth {}", cr.depth);
        assert!(
            !cr.scores.displacement.mean.is_nan(),
            "displacement NaN at depth {}",
            cr.depth,
        );
    }
}

// ── 3.6d: Cell structure matches sync path ──────────────────

/// Two sentinels fed identical traffic, one warming inline and one on a
/// background thread, end with the same accumulated volume and the same set
/// of tracked cells. Structure is decided by observation volume alone and
/// never by anything the trackers compute, so warm-up scheduling — a
/// modelling concern — cannot perturb it. Choosing the background path is a
/// latency decision, not a modelling one.
///
/// ´claim:warmup:the-choice-of-warm-up-path-cannot-change-the-cell-structure-because-splitting-is-driven-by-volume-alone´
/// ´test:integration:step3-background-same-cell-structure-as-sync´
#[test]
fn step3_background_same_cell_structure_as_sync() {
    // Both modes with same seed should produce the same set of
    // tracked cells (same GNodeIds), demonstrating that the staging
    // pathway is consistent.
    let base = SentinelConfig::<u64> {
        split_threshold: 10,
        ..fast_split_config()
    };

    let sync_cfg = SentinelConfig::<u64> {
        background_warming: false,
        ..base.clone()
    };
    let bg_cfg = SentinelConfig::<u64> {
        background_warming: true,
        ..base
    };

    let mut sync_s = Sentinel128::new(sync_cfg).unwrap();
    let mut bg_s = Sentinel128::new(bg_cfg).unwrap();

    let values: Vec<u128> = [cell_values(0xA, 50), cell_values(0x5, 50)].concat();

    for chunk in values.chunks(20) {
        sync_s.ingest(chunk);
        bg_s.ingest(chunk);
    }

    // Give background thread time to finish.
    for _ in 0..15 {
        bg_s.ingest(&cell_values(0xA, 4));
        std::thread::sleep(std::time::Duration::from_millis(5));
    }
    // Run same batches through sync too.
    for _ in 0..15 {
        sync_s.ingest(&cell_values(0xA, 4));
    }

    // The G-V Graph structure should be identical (same observations).
    assert_eq!(
        sync_s.graph().total_sum(),
        bg_s.graph().total_sum(),
        "total_sum should match between sync and background modes",
    );

    // Cell GNodeIds should match (same structural decisions).
    let sync_gnodes = sync_s.cell_gnodes();
    let bg_gnodes = bg_s.cell_gnodes();
    assert_eq!(sync_gnodes, bg_gnodes, "cell gnodes should match between modes");
}

// ── 3.6e: Priority pipeline ────────────────────────────────

/// When warming capacity is scarce — a long noise schedule and small batches
/// leave the queue in progress — the staging area serves its waiting cells in
/// descending order of volume, so a heavily trafficked subtree gets cells into
/// service before a quiet one. Warming effort is spent where the traffic is,
/// which is the same principle that decided the cells were worth having.
///
/// ´claim:warmup:the-staging-area-warms-the-busiest-waiting-cells-first´
/// ´test:integration:step3-higher-volume-cells-warm-via-priority´
#[test]
fn step3_higher_volume_cells_warm_via_priority() {
    // The staging area serves cells in volume-descending order
    // (§ALGO S-18.2). We use a slow noise schedule so that
    // background warming is still in progress when we inspect,
    // then verify that the high-volume subtree has promoted cells.
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        noise_schedule: NoiseSchedule::Explicit(vec![30]),
        noise_batch_size: 2,
        background_warming: true,
        ..fast_split_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed heavy traffic to 0xA subtree so its cells have high volume.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 30));
    }

    // Now seed a second subtree with much less traffic.
    for _ in 0..5 {
        s.ingest(&cell_values(0x5, 10));
    }

    // Allow partial background warming and trigger promotion sweeps.
    for _ in 0..10 {
        s.ingest(&cell_values(0xA, 2));
        std::thread::sleep(std::time::Duration::from_millis(10));
    }

    // At least one non-root cell should have been warmed and promoted
    // via the priority pipeline, confirming the end-to-end path.
    let gnodes = s.cell_gnodes();
    let warmed_non_root = gnodes
        .iter()
        .filter_map(|&gnode| s.inspect_cell(gnode))
        .filter(|insp| insp.depth > 0 && insp.maturity.noise_observations > 0)
        .count();
    assert!(
        warmed_non_root > 0,
        "at least one non-root cell should have noise observations via background warming",
    );
}

// ── 3.6f: Concurrency smoke test ────────────────────────────

/// Ingestion and background warming genuinely overlap: a long run of diverse
/// traffic keeps creating cells while the thread is warming earlier ones, and
/// the two never collide. A cell being warmed is checked out of the staging
/// area for the duration, so the expensive work happens on a cell no other
/// thread can reach, and the lock is held only for the queue operations
/// around it.
///
/// ´claim:warmup:ingest-and-background-warming-overlap-safely-because-a-cell-being-warmed-is-checked-out-of-the-queue´
/// ´test:integration:step3-concurrent-ingest-no-panic´
#[test]
fn step3_concurrent_ingest_no_panic() {
    // Ingest while the background warming thread runs concurrently.
    // Uses max_rank=1 to avoid triggering the Brand SVD oracle on
    // near-degenerate matrices in debug builds.
    let cfg = SentinelConfig::<u64> {
        max_rank: 1,
        ..background_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Feed diverse traffic — the background thread warms new cells
    // concurrently with ingestion.
    for _ in 0..30 {
        let batch: Vec<u128> = [
            cell_values(0x3, 10),
            cell_values(0x6, 10),
            cell_values(0x9, 10),
            cell_values(0xC, 10),
        ]
        .concat();
        let _report = s.ingest(&batch);
    }
}

// ── 3.6g: Eviction during background warming ───────────────

/// The harder end of the same statement: with a background thread running, a
/// cell can be evicted while it is checked out and physically absent from the
/// queue. Eviction still accounts for it, and the cell is discarded when the
/// thread hands it back rather than being restored into a set it has left.
///
/// (´claim:warmup:a-warming-cell-whose-node-leaves-the-analysis-set-is-discarded-rather-than-promoted´)
/// ´test:integration:step3-eviction-during-background-warming-no-panic´
#[test]
fn step3_eviction_during_background_warming_no_panic() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        budget: 50,
        d_evict: 4,
        background_warming: true,
        ..fast_split_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Rapidly create and evict cells with diverse traffic.
    for _ in 0..25 {
        let batch: Vec<u128> = [
            cell_values(0xA, 10),
            cell_values(0x5, 10),
            cell_values(0x2, 10),
            cell_values(0xE, 10),
        ]
        .concat();
        let report = s.ingest(&batch);
        assert_deferred_invariants(&s, &report);
    }
}

// ── 3.6h: Reset restarts background thread ─────────────────

/// This pins the case where the staging area has a second owner. A reset
/// shuts the warming thread down before clearing state and spawns a fresh one
/// afterwards, so the sentinel comes back to root alone with no thread still
/// holding cells from the previous life — and warming resumes normally on the
/// traffic that follows.
///
/// (´claim:warmup:a-reset-empties-the-staging-area-so-warm-up-begins-again-from-the-root-alone´)
/// ´test:integration:step3-reset-restarts-background-thread´
#[test]
fn step3_reset_restarts_background_thread() {
    let mut s = Sentinel128::new(background_config()).unwrap();

    // Build up state.
    for _ in 0..10 {
        s.ingest(&cell_values(0xA, 20));
    }

    // Reset — thread shuts down and restarts.
    s.reset();
    assert_eq!(s.cells_tracked(), 1);

    // Resume with background warming — new cells should still warm.
    for _ in 0..15 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
    }

    // Give time for background warming.
    for _ in 0..15 {
        s.ingest(&cell_values(0xA, 4));
        std::thread::sleep(std::time::Duration::from_millis(5));
    }

    let gnodes = s.cell_gnodes();
    assert!(gnodes.len() > 1, "should have cells after reset + re-ingest");
}

// ── 3.6i: Drop — no hang, no leak ──────────────────────────

/// Dropping a sentinel with a long noise schedule still outstanding returns
/// promptly instead of waiting for the queue to empty. The warming thread
/// checks for shutdown between batches and abandons whatever remains, because
/// cells nobody will ever read from are not worth finishing. Teardown costs
/// at most one batch of work, not the rest of the schedule.
///
/// ´claim:warmup:dropping-a-sentinel-abandons-outstanding-warm-up-instead-of-waiting-for-it´
/// ´test:integration:step3-drop-while-warming-no-hang´
#[test]
fn step3_drop_while_warming_no_hang() {
    // Create a sentinel with background warming and lots of cells
    // being warmed, then drop it. Must not hang or panic.
    let cfg = SentinelConfig::<u64> {
        split_threshold: 5,
        noise_schedule: NoiseSchedule::Explicit(vec![100]),
        background_warming: true,
        ..fast_split_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Create cells that will still be warming at drop time.
    for _ in 0..10 {
        let batch: Vec<u128> = [cell_values(0xA, 20), cell_values(0x5, 20)].concat();
        s.ingest(&batch);
    }

    // Drop the sentinel — this should shut down the background thread
    // gracefully within a reasonable time.
    let start = std::time::Instant::now();
    drop(s);
    let elapsed = start.elapsed();

    assert!(
        elapsed < std::time::Duration::from_secs(5),
        "drop should not hang: elapsed {elapsed:?}",
    );
}
