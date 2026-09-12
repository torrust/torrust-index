// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(clippy::print_stdout)]

//! # Advanced Pedagogy Test — Companion to [`pedagogy.rs`]
//!
//! The basic pedagogy test walks through the Sentinel **lifecycle
//! surface**: construction, feed-forward observation, spatial refinement,
//! coordination, host decay, and reset. This companion test exercises the
//! **inspection surface** — the readouts, summaries, and cross-checks a
//! host uses after the lifecycle has produced a non-trivial model.
//!
//! ## Why a separate test?
//!
//! The Sentinel deliberately separates measurement from policy. The basic
//! test shows how measurements are produced. This test shows how to read
//! them without smuggling in decisions: analysis-set membership, ancestor
//! closure, per-sample payloads, geometry flags, coordination contexts,
//! scale profiles, and temporal separation between spatial weight and
//! tracker state.
//!
//! ## What this test demonstrates
//!
//! ### Inspection-oriented configuration (Step 0)
//!
//! The test uses deterministic foreground warm-up so every tracker created
//! by investment reconciliation is online in the same `ingest()` call. That
//! keeps the inspection surface stable: `investment_set_size == full_size`
//! once the graph has settled, and no asynchronous warm-up race can obscure
//! the fields being demonstrated.
//!
//! ### Analysis-set anatomy (Step 1)
//!
//! The public [`AnalysisSet`] is checked against the report summary. Every
//! competitive cell's G-tree parent chain is present in the full set, and
//! the root is permanent context, never a competitive target.
//!
//! ### Report and inspection consistency (Steps 2-3)
//!
//! [`CellReport`] values are cross-checked with `inspect_cell()`. Optional
//! per-sample payloads are verified as raw finite measurements whose counts
//! match the routing counts.
//!
//! ### Geometry edge flags (Step 4)
//!
//! The scoring geometry record is not decoration. It tells the host when an
//! axis is structurally inactive or saturable. A rank-one configuration
//! demonstrates coherence inactivity; a tiny four-bit Sentinel demonstrates
//! novelty-saturability without relying on fragile score values.
//!
//! ### Coordination read surface (Step 5)
//!
//! Coordination contexts are inspected as reports over competitive-cell
//! score vectors. Per-member records are verified against the contributing
//! competitive cells.
//!
//! ### Scale profile and temporal separation (Steps 6-7)
//!
//! The same batch yields measurements at several depths; the test prints a
//! depth-indexed novelty profile without asserting a policy verdict. Then
//! host-controlled decay is shown to rescale spatial importance without
//! mutating tracker baselines or counting traffic.
//!
//! ### Reset read surface (Step 8)
//!
//! Reset drops learned state and recreates the warmed root, clearing the
//! inspection surfaces back to the fresh lifecycle.
//!
//! Run with:
//!
//! ```sh
//! cargo test -p torrust-sentinel --test pedagogy_advanced
//! ```
//!
//! # Test Index
//!
//! ## Inspection Setup (§ALGO S-11, ADR-S-019)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 0 | [`step_0_inspection_setup`] | Foreground warm-up gives deterministic readouts |
//!
//! ## Analysis-Set Read Surface (§ALGO S-8, ADR-S-019)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 1 | [`step_1_analysis_set_anatomy`] | Competitive targets plus ancestors form the full set |
//! | 2 | [`step_2_inspection_mirrors_reports`] | `inspect_cell()` and batch reports expose the same tracker facts |
//!
//! ## Report Payloads and Geometry (§ALGO S-14)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 3 | [`step_3_payload_records`] | Per-sample payloads are count-aligned raw measurements |
//! | 4 | [`step_4_scoring_geometry_edges`] | Geometry flags explain inactive and saturable axes |
//!
//! ## Coordination and Scale (§ALGO S-7, §ALGO S-9)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 5 | [`step_5_coordination_read_surface`] | Coordination reports measure cross-cell score patterns |
//! | 6 | [`step_6_scale_profile`] | Depth-indexed scores are diagnostic measurements, not verdicts |
//!
//! ## Host Policy and Lifecycle (§ALGO S-10, ADR-S-002)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 7 | [`step_7_temporal_separation`] | Spatial decay does not mutate tracker baselines |
//! | 8 | [`step_8_reset_read_surface`] | Reset clears readouts and recreates the warmed root |

mod common;

use std::collections::{BTreeMap, BTreeSet};

use common::{anomalous_values, assert_invariants, cell_values};
use torrust_sentinel::{
    AxisBaselineSnapshots, BaselineSnapshot, BatchReport, CellInspection, CellReport, CoordinationReport, MemberScore,
    NoiseSchedule, SampleScore, ScoreDistribution, Sentinel128, SentinelConfig, SpectralSentinel, SvdStrategy,
};

// -- Configuration ---------------------------------------------------------

/// Deterministic configuration tuned for inspection rather than suspense.
///
/// Compared with the basic pedagogy test, this uses a slightly larger
/// analysis budget and rank cap so the read surface has more structure to
/// inspect. Background warming stays disabled: this file teaches the public
/// readouts, not scheduler timing.
fn advanced_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 4,
        forgetting_factor: 0.90,
        rank_update_interval: 4,
        analysis_k: 24,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: true,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.99,
        cusum_coord_slow_decay: 0.99,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 8,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: NoiseSchedule::Explicit(vec![6]),
        noise_batch_size: 4,
        noise_seed: Some(2026),
        background_warming: false,
        svd_strategy: SvdStrategy::Brand,
    }
}

fn rank_one_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 1,
        noise_seed: Some(7),
        ..advanced_config()
    }
}

fn tiny_geometry_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 4,
        rank_update_interval: 1,
        noise_seed: Some(11),
        ..advanced_config()
    }
}

// -- Helpers ---------------------------------------------------------------

const MULTISCALE_MAX_ROUNDS: usize = 40;

type TinySentinel = SpectralSentinel<u64, u64, 4>;

fn heading(s: &str) {
    let rule = "=".repeat(72);
    println!("\n{rule}");
    println!("  {s}");
    println!("{rule}");
}

fn subheading(s: &str) {
    println!("\n-- {s} --");
}

fn mixed_normal_batch() -> Vec<u128> {
    [
        cell_values(0x1, 8),
        cell_values(0x5, 8),
        cell_values(0xA, 8),
        cell_values(0xF, 8),
    ]
    .concat()
}

fn all_cell_reports(report: &BatchReport<u128>) -> impl Iterator<Item = &CellReport<u128>> {
    report.cell_reports.iter().chain(report.ancestor_reports.iter())
}

fn root_report(report: &BatchReport<u128>) -> &CellReport<u128> {
    report
        .ancestor_reports
        .iter()
        .find(|cell| cell.depth == 0)
        .expect("non-empty batch reports include the root ancestor")
}

fn assert_score_distribution(name: &str, score: &ScoreDistribution) {
    for (field, value) in [
        ("min", score.min),
        ("max", score.max),
        ("mean", score.mean),
        ("max_z_score", score.max_z_score),
        ("mean_z_score", score.mean_z_score),
        ("baseline.mean", score.baseline.mean),
        ("baseline.variance", score.baseline.variance),
        ("cusum.accumulator", score.cusum.accumulator),
        ("cusum.slow_baseline.mean", score.cusum.slow_baseline.mean),
        ("cusum.slow_baseline.variance", score.cusum.slow_baseline.variance),
        ("clip_pressure", score.clip_pressure),
    ] {
        assert!(value.is_finite(), "{name}.{field} must be finite");
    }

    assert!(score.max + 1e-12 >= score.min, "{name}.max must be >= min");
    assert!(score.mean + 1e-12 >= score.min, "{name}.mean must be >= min");
    assert!(score.mean <= score.max + 1e-12, "{name}.mean must be <= max");
    assert!(score.baseline.variance >= -1e-12, "{name}.baseline variance is non-negative");
    assert!(
        score.cusum.slow_baseline.variance >= -1e-12,
        "{name}.slow baseline variance is non-negative"
    );
    assert!(score.cusum.accumulator >= -1e-12, "{name}.CUSUM is one-sided");
    assert!(
        (-1e-12..=1.0 + 1e-12).contains(&score.clip_pressure),
        "{name}.clip pressure is a fraction"
    );
}

fn assert_cell_scores(cell: &CellReport<u128>) {
    assert_score_distribution("novelty", &cell.scores.novelty);
    assert_score_distribution("displacement", &cell.scores.displacement);
    assert_score_distribution("surprise", &cell.scores.surprise);
    assert_score_distribution("coherence", &cell.scores.coherence);

    assert!(cell.scores.novelty.min >= -1e-12, "novelty is non-negative");
    assert!(
        cell.scores.displacement.min >= -1e-12 && cell.scores.displacement.max <= 1.0 + 1e-12,
        "displacement is bounded in [0, 1]"
    );
    assert!(cell.scores.surprise.min >= -1e-12, "surprise is non-negative");
    assert!(cell.scores.coherence.min >= -1e-12, "coherence is non-negative");
}

fn assert_sample_score(sample: &SampleScore) {
    for (name, value) in [
        ("novelty", sample.novelty),
        ("displacement", sample.displacement),
        ("surprise", sample.surprise),
        ("coherence", sample.coherence),
        ("novelty_z", sample.novelty_z),
        ("displacement_z", sample.displacement_z),
        ("surprise_z", sample.surprise_z),
        ("coherence_z", sample.coherence_z),
    ] {
        assert!(value.is_finite(), "sample.{name} must be finite");
    }

    assert!(sample.novelty >= -1e-12, "sample novelty is non-negative");
    assert!(
        sample.displacement >= -1e-12 && sample.displacement <= 1.0 + 1e-12,
        "sample displacement is bounded in [0, 1]"
    );
    assert!(sample.surprise >= -1e-12, "sample surprise is non-negative");
    assert!(sample.coherence >= -1e-12, "sample coherence is non-negative");
}

fn assert_member_score(member: &MemberScore<u128>) {
    assert!(member.cell_start < member.cell_end, "member cell interval is non-empty");
    for (name, value) in [
        ("novelty", member.novelty),
        ("displacement", member.displacement),
        ("surprise", member.surprise),
        ("coherence", member.coherence),
        ("novelty_z", member.novelty_z),
        ("displacement_z", member.displacement_z),
        ("surprise_z", member.surprise_z),
        ("coherence_z", member.coherence_z),
    ] {
        assert!(value.is_finite(), "member.{name} must be finite");
    }
    assert!(member.novelty >= -1e-12, "member novelty is non-negative");
    assert!(
        member.displacement >= -1e-12 && member.displacement <= 1.0 + 1e-12,
        "member displacement is bounded in [0, 1]"
    );
    assert!(member.surprise >= -1e-12, "member surprise is non-negative");
    assert!(member.coherence >= -1e-12, "member coherence is non-negative");
}

fn assert_cell_geometry(cell: &CellReport<u128>, config: &SentinelConfig<u64>) {
    assert_eq!(cell.analysis_width, 128 - cell.depth as usize);
    assert_eq!(cell.geometry.dim, cell.analysis_width);
    assert_eq!(cell.geometry.cap, cell.analysis_width.min(config.max_rank));
    assert!(cell.rank >= 1);
    assert!(cell.rank <= cell.geometry.cap);
    assert_eq!(cell.geometry.residual_dof, cell.geometry.dim - cell.rank);
}

fn assert_report_surface(sentinel: &Sentinel128, report: &BatchReport<u128>, batch_size: usize) {
    let config = sentinel.config();

    assert_invariants(sentinel, report);
    assert_eq!(report.health.lifetime_observations, sentinel.lifetime_observations());
    assert_eq!(report.health.cells_tracked, sentinel.cells_tracked());
    assert_eq!(report.health.active_trackers, sentinel.cell_gnodes().len());
    assert_eq!(
        report.analysis_set_summary.competitive_size,
        sentinel.analysis_set().competitive_count()
    );
    assert_eq!(report.analysis_set_summary.full_size, sentinel.analysis_set().total_count());
    assert!(report.analysis_set_summary.competitive_size <= config.analysis_k);
    assert!(report.analysis_set_summary.investment_set_size >= report.analysis_set_summary.full_size);

    let root = root_report(report);
    assert_eq!(root.sample_count, batch_size, "root receives the full ingestion batch");

    for cell in &report.cell_reports {
        assert!(cell.is_competitive, "cell_reports are competitive reports");
        assert!(cell.sample_count > 0);
        assert_cell_geometry(cell, config);
        assert_cell_scores(cell);
    }
    for cell in &report.ancestor_reports {
        assert!(!cell.is_competitive, "ancestor_reports are non-competitive reports");
        assert!(cell.sample_count > 0);
        assert_cell_geometry(cell, config);
        assert_cell_scores(cell);
    }
}

fn assert_inspection_matches_report(sentinel: &Sentinel128, report: &CellReport<u128>) {
    let inspection = sentinel
        .inspect_cell(report.gnode_id)
        .expect("reported cells are inspectable after the batch");

    assert_eq!(inspection.gnode_id, report.gnode_id);
    assert_eq!((inspection.start, inspection.end), (report.start, report.end));
    assert_eq!(inspection.depth, report.depth);
    assert_eq!(inspection.analysis_width, report.analysis_width);
    assert_eq!(inspection.is_competitive, report.is_competitive);
    assert_eq!(inspection.rank, report.rank);
    assert_eq!(inspection.geometry.dim, report.geometry.dim);
    assert_eq!(inspection.geometry.cap, report.geometry.cap);
    assert_eq!(inspection.geometry.residual_dof, report.geometry.residual_dof);
    assert_eq!(inspection.maturity.real_observations, report.maturity.real_observations);
    assert_eq!(inspection.maturity.noise_observations, report.maturity.noise_observations);
}

fn assert_coordination_report(coordination: &CoordinationReport<u128>, config: &SentinelConfig<u64>) {
    assert!(coordination.start < coordination.end);
    assert!(coordination.cells_reporting >= 2);
    assert_eq!(coordination.geometry.dim, 4);
    assert_eq!(coordination.geometry.cap, config.max_rank.min(4));
    assert!(coordination.rank >= 1);
    assert!(coordination.rank <= coordination.geometry.cap);
    assert_eq!(
        coordination.geometry.residual_dof,
        coordination.geometry.dim - coordination.rank
    );
    assert_score_distribution("coord.novelty", &coordination.scores.novelty);
    assert_score_distribution("coord.displacement", &coordination.scores.displacement);
    assert_score_distribution("coord.surprise", &coordination.scores.surprise);
    assert_score_distribution("coord.coherence", &coordination.scores.coherence);
}

fn assert_baseline_same(name: &str, before: BaselineSnapshot, after: BaselineSnapshot) {
    assert_eq!(
        before.mean.to_bits(),
        after.mean.to_bits(),
        "{name} mean changed during spatial decay"
    );
    assert_eq!(
        before.variance.to_bits(),
        after.variance.to_bits(),
        "{name} variance changed during spatial decay"
    );
}

fn assert_baselines_same(before: AxisBaselineSnapshots, after: AxisBaselineSnapshots) {
    assert_baseline_same("novelty", before.novelty, after.novelty);
    assert_baseline_same("displacement", before.displacement, after.displacement);
    assert_baseline_same("surprise", before.surprise, after.surprise);
    assert_baseline_same("coherence", before.coherence, after.coherence);
}

fn drive_to_multiscale(sentinel: &mut Sentinel128) -> BatchReport<u128> {
    let seed = cell_values(0xA, 16);
    let seed_report = sentinel.ingest(&seed);
    assert_report_surface(sentinel, &seed_report, seed.len());

    let concentrated = cell_values(0xA, 48);
    let concentrated_report = sentinel.ingest(&concentrated);
    assert_report_surface(sentinel, &concentrated_report, concentrated.len());

    let values = mixed_normal_batch();
    let mut report = sentinel.ingest(&values);
    assert_report_surface(sentinel, &report, values.len());

    for round in 1..=MULTISCALE_MAX_ROUNDS {
        if report.cell_reports.len() >= 2 && !report.coordination_reports.is_empty() {
            println!("Multiscale state reached after {round} mixed batch(es).");
            return report;
        }
        report = sentinel.ingest(&values);
        assert_report_surface(sentinel, &report, values.len());
    }

    panic!(
        "coordination did not activate within {MULTISCALE_MAX_ROUNDS} rounds: cell_reports={}, coordination_reports={}",
        report.cell_reports.len(),
        report.coordination_reports.len(),
    );
}

// ========================================================================
//  STEP 0: INSPECTION-ORIENTED SETUP
//  §ALGO S-11, ADR-S-019
// ========================================================================

fn step_0_inspection_setup(sentinel: &Sentinel128) {
    heading("Step 0: Inspection-Oriented Setup");
    println!("  (§ALGO S-11, ADR-S-019)");

    let health = sentinel.health();
    assert_eq!(sentinel.graph().node_count(), 1);
    assert_eq!(sentinel.graph().total_sum(), 0);
    assert_eq!(sentinel.cells_tracked(), 1);
    assert_eq!(sentinel.analysis_set().competitive_count(), 0);
    assert_eq!(sentinel.analysis_set().total_count(), 1);
    assert_eq!(health.warming_trackers, 0, "foreground warm-up leaves no staging backlog");
    assert_eq!(health.investment_set_size, 1);

    let root = sentinel
        .inspect_cell(sentinel.graph().g_root())
        .expect("fresh Sentinel exposes its root tracker");
    assert_eq!(root.depth, 0);
    assert_eq!(root.analysis_width, 128);
    assert!(!root.is_competitive);
    assert!(root.maturity.noise_observations > 0);
    assert_eq!(root.maturity.real_observations, 0);

    println!("Fresh inspection surface:");
    println!("  graph nodes = {}", sentinel.graph().node_count());
    println!("  active trackers = {}", health.active_trackers);
    println!("  warming trackers = {}", health.warming_trackers);
    println!("  root noise observations = {}", root.maturity.noise_observations);
    println!("  foreground warm-up makes tracker readouts immediately inspectable");
}

// ========================================================================
//  STEP 1: ANALYSIS SET ANATOMY
//  §ALGO S-8, ADR-S-019
// ========================================================================

fn step_1_analysis_set_anatomy(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    heading("Step 1: Analysis-Set Anatomy");
    println!("  (§ALGO S-8, ADR-S-019)");

    let analysis = sentinel.analysis_set();
    let summary = report.analysis_set_summary;

    assert_eq!(analysis.competitive_count(), summary.competitive_size);
    assert_eq!(analysis.total_count(), summary.full_size);
    assert!(analysis.total_count() > analysis.competitive_count());

    let root = sentinel.graph().g_root();
    assert!(analysis.contains(root), "root is always in the full analysis set");
    assert!(
        !analysis.is_competitive(root),
        "root provides context, not a competitive slot"
    );

    for entry in analysis.competitive() {
        let mut current = Some(entry.gnode);
        while let Some(gnode) = current {
            assert!(
                analysis.contains(gnode),
                "ancestor closure missing GNode {:?} for competitive [{:#034x}, {:#034x})",
                gnode,
                entry.start,
                entry.end,
            );
            let info = sentinel.graph().gnode_info(gnode).expect("analysis cells are live G-nodes");
            current = info.parent;
        }
    }

    subheading("Set sizes");
    println!("  competitive targets = {}", analysis.competitive_count());
    println!("  full analysis set = {}", analysis.total_count());
    println!("  investment set = {}", summary.investment_set_size);
    println!("  depth range = {:?}", summary.depth_range);

    subheading("Competitive cells");
    for entry in analysis.competitive() {
        println!(
            "  [{:#034x}, {:#034x}) depth={} v_depth={} importance={}",
            entry.start, entry.end, entry.depth, entry.v_depth, entry.importance,
        );
    }

    println!();
    println!("Every competitive cell's parent chain is present in the full set.");
    println!("The host can inspect local cells and their shared ancestors separately.");
}

// ========================================================================
//  STEP 2: INSPECTION MIRRORS REPORTS
//  §ALGO S-14, ADR-S-014
// ========================================================================

fn step_2_inspection_mirrors_reports(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    heading("Step 2: inspect_cell Mirrors Batch Reports");
    println!("  (§ALGO S-14, ADR-S-014)");

    let mut checked = 0usize;
    for cell in all_cell_reports(report) {
        assert_inspection_matches_report(sentinel, cell);
        checked += 1;
    }

    let inspected_ids: BTreeSet<_> = sentinel.cell_gnodes().into_iter().collect();
    assert_eq!(inspected_ids.len(), sentinel.cells_tracked());
    for id in inspected_ids {
        let inspection: CellInspection<u128> = sentinel.inspect_cell(id).expect("cell_gnodes are inspectable");
        assert_eq!(inspection.geometry.dim, inspection.analysis_width);
        assert_eq!(
            inspection.geometry.cap,
            inspection.analysis_width.min(sentinel.config().max_rank)
        );
    }

    println!("Checked {checked} reported cell snapshots against inspect_cell().");
    println!("`inspect_cell()` is the stable read path for tracker metadata between batches.");
}

// ========================================================================
//  STEP 3: PAYLOAD RECORDS
//  §ALGO S-14.8, §ALGO S-14.9
// ========================================================================

fn step_3_payload_records(report: &BatchReport<u128>) {
    heading("Step 3: Per-Sample and Per-Member Payload Records");
    println!("  (§ALGO S-14.8, §ALGO S-14.9)");

    let mut sample_records = 0usize;
    for cell in all_cell_reports(report) {
        let samples = cell.per_sample.as_ref().expect("advanced config enables per-sample scores");
        assert_eq!(samples.len(), cell.sample_count);
        for sample in samples {
            assert_sample_score(sample);
        }
        sample_records += samples.len();
    }

    let mut member_records = 0usize;
    for coordination in &report.coordination_reports {
        let members = coordination
            .per_member
            .as_ref()
            .expect("advanced config enables per-member coordination scores");
        assert_eq!(members.len(), coordination.cells_reporting);
        for member in members {
            assert_member_score(member);
        }
        member_records += members.len();
    }

    println!("Per-sample records checked: {sample_records}");
    println!("Per-member coordination records checked: {member_records}");
    println!("These payloads are raw measurements aligned with routing, not decisions.");
}

// ========================================================================
//  STEP 4: SCORING GEOMETRY EDGE FLAGS
//  §ALGO S-5.2, §ALGO S-14.7
// ========================================================================

fn step_4_scoring_geometry_edges() {
    heading("Step 4: Scoring Geometry Edge Flags");
    println!("  (§ALGO S-5.2, §ALGO S-14.7)");

    let rank_one = Sentinel128::new(rank_one_config()).expect("rank-one config is valid");
    let rank_one_health = rank_one.health();
    assert_eq!(
        rank_one_health.geometry_distribution.coherence_inactive,
        rank_one_health.active_trackers
    );

    let root = rank_one
        .inspect_cell(rank_one.graph().g_root())
        .expect("rank-one root is inspectable");
    assert_eq!(root.rank, 1);
    assert_eq!(root.geometry.cap, 1);
    assert_eq!(root.geometry.residual_dof, root.geometry.dim - 1);
    assert!(!root.geometry.is_novelty_saturated());

    let tiny = TinySentinel::new(tiny_geometry_config()).expect("tiny geometry config is valid");
    let tiny_root = tiny.inspect_cell(tiny.graph().g_root()).expect("tiny root is inspectable");
    assert_eq!(tiny_root.geometry.dim, 4);
    assert_eq!(tiny_root.geometry.cap, 4);
    assert!(tiny_root.geometry.is_novelty_saturable());

    println!("Rank-one Sentinel:");
    println!("  active trackers = {}", rank_one_health.active_trackers);
    println!(
        "  coherence-inactive trackers = {}",
        rank_one_health.geometry_distribution.coherence_inactive
    );
    println!(
        "  root dim = {}, cap = {}, residual DOF = {}",
        root.geometry.dim, root.geometry.cap, root.geometry.residual_dof
    );

    println!("Tiny four-bit Sentinel:");
    println!("  root dim = {}, cap = {}", tiny_root.geometry.dim, tiny_root.geometry.cap);
    println!("  novelty-saturable = {}", tiny_root.geometry.is_novelty_saturable());
    println!("Geometry fields tell the host whether an axis is structurally meaningful.");
}

// ========================================================================
//  STEP 5: COORDINATION READ SURFACE
//  §ALGO S-7, §ALGO S-9
// ========================================================================

fn step_5_coordination_read_surface(sentinel: &Sentinel128, report: &BatchReport<u128>) {
    heading("Step 5: Coordination Read Surface");
    println!("  (§ALGO S-7, §ALGO S-9)");

    assert!(
        !report.coordination_reports.is_empty(),
        "multiscale setup activated coordination"
    );
    assert!(sentinel.health().active_coordination_contexts >= report.coordination_reports.len());

    let competitive_cells: BTreeSet<_> = report
        .cell_reports
        .iter()
        .map(|cell| (cell.start, cell.end, cell.depth))
        .collect();

    for coordination in &report.coordination_reports {
        assert_coordination_report(coordination, sentinel.config());
        let members = coordination.per_member.as_ref().expect("per-member payloads are enabled");
        for member in members {
            assert!(member.cell_start >= coordination.start);
            assert!(member.cell_end <= coordination.end);
            assert!(
                competitive_cells.contains(&(member.cell_start, member.cell_end, member.cell_depth)),
                "coordination member must be a reporting competitive cell"
            );
        }
    }

    subheading("Coordination contexts");
    for coordination in &report.coordination_reports {
        println!(
            "  [{:#034x}, {:#034x}) depth={} cells={} rank={} novelty_mean={:.6}",
            coordination.start,
            coordination.end,
            coordination.depth,
            coordination.cells_reporting,
            coordination.rank,
            coordination.scores.novelty.mean,
        );
    }

    println!();
    println!("Coordination rows are competitive-cell score summaries.");
    println!("The tier measures cross-cell structure; it does not classify it.");
}

// ========================================================================
//  STEP 6: SCALE PROFILE
//  §ALGO S-9.4, §ALGO S-16.5
// ========================================================================

fn step_6_scale_profile(sentinel: &mut Sentinel128) {
    heading("Step 6: Depth-Indexed Scale Profile");
    println!("  (§ALGO S-9.4, §ALGO S-16.5)");

    let values = anomalous_values(0xA, 8);
    let report = sentinel.ingest(&values);
    assert_report_surface(sentinel, &report, values.len());

    let mut max_novelty_by_depth: BTreeMap<u32, f64> = BTreeMap::new();
    for cell in all_cell_reports(&report) {
        let entry = max_novelty_by_depth.entry(cell.depth).or_insert(0.0);
        *entry = (*entry).max(cell.scores.novelty.max_z_score);
    }

    assert!(max_novelty_by_depth.contains_key(&0), "root depth participates");
    assert!(
        max_novelty_by_depth.keys().any(|depth| *depth > 0),
        "non-root depths participate"
    );
    assert!(max_novelty_by_depth.len() >= 2, "scale profile has multiple depths");

    subheading("Max novelty z-score by G-tree depth");
    for (depth, z) in &max_novelty_by_depth {
        assert!(z.is_finite());
        println!("  depth {depth}: {z:.6}");
    }

    println!();
    println!("The host can compare depth-local and root-level measurements.");
    println!("This test asserts the profile exists, not what policy should conclude.");
}

// ========================================================================
//  STEP 7: TEMPORAL SEPARATION
//  §ALGO S-10, ADR-S-002
// ========================================================================

fn step_7_temporal_separation(sentinel: &mut Sentinel128) {
    heading("Step 7: Temporal Separation");
    println!("  (§ALGO S-10, ADR-S-002)");

    let root = sentinel.graph().g_root();
    let before_sum = sentinel.graph().total_sum();
    let before_lifetime = sentinel.lifetime_observations();
    let before_cells = sentinel.cells_tracked();
    let before_root = sentinel.inspect_cell(root).expect("root is inspectable before decay");
    let before_baselines = before_root.baselines;

    sentinel.decay(0.5, 0.0);

    let after_sum = sentinel.graph().total_sum();
    let after_root = sentinel.inspect_cell(root).expect("root remains inspectable after decay");
    assert!(after_sum < before_sum, "uniform attenuation reduces spatial importance");
    assert_eq!(sentinel.lifetime_observations(), before_lifetime, "decay is not traffic");
    assert_eq!(
        sentinel.cells_tracked(),
        before_cells,
        "decay does not directly destroy trackers"
    );
    assert_baselines_same(before_baselines, after_root.baselines);

    println!("Spatial importance: {before_sum} -> {after_sum}");
    println!(
        "Lifetime observations: {before_lifetime} -> {}",
        sentinel.lifetime_observations()
    );
    println!("Active trackers: {before_cells} -> {}", sentinel.cells_tracked());
    println!("Root tracker baselines unchanged by spatial decay.  ✓");
}

// ========================================================================
//  STEP 8: RESET READ SURFACE
//  Public lifecycle contract
// ========================================================================

fn step_8_reset_read_surface(sentinel: &mut Sentinel128) {
    heading("Step 8: Reset Read Surface");

    assert!(sentinel.graph().total_sum() > 0);
    assert!(sentinel.lifetime_observations() > 0);

    sentinel.reset();

    let health = sentinel.health();
    assert_eq!(sentinel.graph().node_count(), 1);
    assert_eq!(sentinel.graph().terminal_count(), 1);
    assert_eq!(sentinel.graph().total_sum(), 0);
    assert_eq!(sentinel.cells_tracked(), 1);
    assert_eq!(sentinel.lifetime_observations(), 0);
    assert_eq!(sentinel.analysis_set().competitive_count(), 0);
    assert_eq!(sentinel.analysis_set().total_count(), 1);
    assert_eq!(health.active_coordination_contexts, 0);
    assert_eq!(health.warming_trackers, 0);

    let root = sentinel
        .inspect_cell(sentinel.graph().g_root())
        .expect("reset recreates the inspectable root");
    assert_eq!(root.depth, 0);
    assert_eq!(root.analysis_width, 128);
    assert!(root.maturity.noise_observations > 0);
    assert_eq!(root.maturity.real_observations, 0);

    println!("After reset:");
    println!("  graph nodes = {}", sentinel.graph().node_count());
    println!("  active trackers = {}", health.active_trackers);
    println!("  coordination contexts = {}", health.active_coordination_contexts);
    println!("  root noise observations = {}", root.maturity.noise_observations);
}

// ========================================================================
//  MAIN
// ========================================================================

fn main() {
    // Support `--list --format terse` so cargo-nextest can enumerate this
    // custom-harness test binary. With `--ignored`, output nothing.
    let args: Vec<String> = std::env::args().collect();
    if args.iter().any(|arg| arg == "--list") {
        if !args.iter().any(|arg| arg == "--ignored") {
            println!("pedagogy_advanced: test");
        }
        return;
    }

    let config = advanced_config();
    config.validate().expect("advanced pedagogy config must be valid");

    let mut sentinel = Sentinel128::new(config).expect("valid config constructs a Sentinel");

    step_0_inspection_setup(&sentinel);
    let report = drive_to_multiscale(&mut sentinel);
    step_1_analysis_set_anatomy(&sentinel, &report);
    step_2_inspection_mirrors_reports(&sentinel, &report);
    step_3_payload_records(&report);
    step_4_scoring_geometry_edges();
    step_5_coordination_read_surface(&sentinel, &report);
    step_6_scale_profile(&mut sentinel);
    step_7_temporal_separation(&mut sentinel);
    step_8_reset_read_surface(&mut sentinel);

    heading("All Advanced Claims Verified");
    println!();
    println!("  Inspection setup:");
    println!("    [ok] foreground warm-up makes tracker readouts deterministic");
    println!("    [ok] fresh Sentinel exposes a warmed, non-competitive root");
    println!();
    println!("  Analysis-set read surface:");
    println!("    [ok] report summary matches AnalysisSet accessors");
    println!("    [ok] competitive cells are closed under G-tree ancestry");
    println!("    [ok] inspect_cell mirrors reported tracker metadata");
    println!();
    println!("  Report payloads and geometry:");
    println!("    [ok] per-sample payloads match routed sample counts");
    println!("    [ok] per-member coordination payloads match reporting cells");
    println!("    [ok] geometry flags expose inactive and saturable axes");
    println!();
    println!("  Coordination and scale:");
    println!("    [ok] coordination contexts measure cross-cell score patterns");
    println!("    [ok] depth-indexed profiles give scale diagnostics");
    println!();
    println!("  Host policy and lifecycle:");
    println!("    [ok] spatial decay leaves tracker baselines untouched");
    println!("    [ok] reset restores the fresh read surface and warmed root");
}
