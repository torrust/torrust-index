// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(clippy::print_stdout)]

//! # End-to-End Pedagogy Test for the Spectral Sentinel
//!
//! This custom-harness test walks through the Sentinel lifecycle as a
//! narrative: construction, feed-forward observation, spatial refinement,
//! analysis-set closure, hierarchical coordination, score interpretation,
//! host-controlled decay, and reset. Every assertion corresponds to a
//! public contract from the algorithm, API reference, or ADRs.
//!
//! ## What is the Spectral Sentinel?
//!
//! The Sentinel is a three-layer online anomaly spectrometer for
//! positionally structured coordinate streams:
//!
//! - **Layer 1: Spatial substrate.** A Mudlark G-V Graph adapts a dyadic
//!   contour over the coordinate domain using pure observation volume.
//! - **Layer 2: Analysis selector.** The top competitive cells are closed
//!   under G-tree ancestry so every selected region has a full model chain
//!   back to the root.
//! - **Layer 3: Analysis engine.** Per-cell subspace trackers score suffix
//!   bit vectors on four raw axes, and coordination trackers model
//!   cross-cell score patterns.
//!
//! The central teaching point is the design principle from ADR-S-001 and
//! ADR-S-002: **the Sentinel measures; the host decides**. The spatial
//! graph receives `Delta = 1` per input value. Scores flow outward in
//! reports; they never flow back into spatial importance.
//!
//! ## What this test demonstrates
//!
//! Steps 0-2 show construction, automatic noise warm-up, the feed-forward
//! invariant, and spatial refinement under concentrated traffic. Steps 3-4
//! show analysis-set ancestry, suffix widths, report structure, raw score
//! polarity, and hierarchical coordination. Steps 5-6 demonstrate that
//! temporal policy is host-controlled through `decay()` and that `reset()`
//! returns the system to a fresh, warmed root.
//!
//! Run with:
//!
//! ```sh
//! cargo test -p torrust-sentinel --test pedagogy
//! ```
//!
//! # Test Index
//!
//! ## Construction & Warm-Up (§ALGO S-11, ADR-S-007)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 0 | [`step_0_fresh`] | Fresh Sentinel = one spatial root, one warmed tracker, no real observations |
//!
//! ## Feed-Forward Observation (§ALGO S-8.1, ADR-S-002)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 1 | [`step_1_first_batch`] | Each raw value increments the graph by exactly one unit |
//! | 2 | [`step_2_concentrated_traffic`] | Volume alone drives spatial splitting and analysis selection |
//!
//! ## Multi-Scale Analysis (§ALGO S-8, §ALGO S-9)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 3 | [`step_3_multiscale_reports`] | Competitive cells plus ancestors form a complete reporting chain |
//! | 4 | [`step_4_scores_are_measurements`] | Scores are finite, higher-polarity measurements, not verdicts |
//!
//! ## Host Policy & Lifecycle (§ALGO S-10, §ALGO S-13.4)
//!
//! | Step | Function | What it teaches |
//! |------|----------|-----------------|
//! | 5 | [`step_5_host_controlled_decay`] | Decay changes spatial importance, not tracker count or lifetime observations |
//! | 6 | [`step_6_reset`] | Reset clears learned state while preserving the configured warm-up lifecycle |

mod common;

use common::{anomalous_values, assert_invariants, cell_values};
use torrust_sentinel::{
    AnomalyScores, BatchReport, CellReport, CoordinationReport, MemberScore, NoiseSchedule, ScoreDistribution, Sentinel128,
    SentinelConfig, SvdStrategy,
};

// -- Configuration ---------------------------------------------------------

/// Small deterministic configuration for a narrative end-to-end run.
///
/// The values are intentionally close to the shared integration-test
/// presets, but written out here so the test explains itself. The low
/// split threshold makes spatial refinement visible after a few batches;
/// explicit noise keeps warm-up fast and deterministic.
fn pedagogy_config() -> SentinelConfig<u64> {
    SentinelConfig::<u64> {
        max_rank: 2,
        forgetting_factor: 0.90,
        rank_update_interval: 5,
        analysis_k: 16,
        analysis_depth_cutoff: 6,
        energy_threshold: 0.90,
        eps: 1e-6,
        per_sample_scores: true,
        cusum_allowance_sigmas: 0.5,
        cusum_slow_decay: 0.99,
        cusum_coord_slow_decay: 0.99,
        clip_sigmas: 3.0,
        clip_pressure_decay: 0.95,
        split_threshold: 10,
        d_create: 3,
        d_evict: 6,
        budget: 100_000,
        noise_schedule: NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 4,
        noise_seed: Some(2026),
        background_warming: false,
        svd_strategy: SvdStrategy::Brand,
    }
}

// -- Helpers ---------------------------------------------------------------

fn heading(s: &str) {
    let rule = "=".repeat(72);
    println!("\n{rule}");
    println!("  {s}");
    println!("{rule}");
}

fn subheading(s: &str) {
    println!("\n-- {s} --");
}

fn all_cell_reports(report: &BatchReport<u128>) -> impl Iterator<Item = &CellReport<u128>> {
    report.cell_reports.iter().chain(report.ancestor_reports.iter())
}

fn root_ancestor(report: &BatchReport<u128>) -> &CellReport<u128> {
    report
        .ancestor_reports
        .iter()
        .find(|cell| cell.depth == 0)
        .expect("root tracker must report as an ancestor when the batch is non-empty")
}

fn max_novelty_z(report: &BatchReport<u128>) -> f64 {
    all_cell_reports(report)
        .map(|cell| cell.scores.novelty.max_z_score)
        .fold(0.0_f64, f64::max)
}

fn assert_score_distribution(name: &str, score: &ScoreDistribution) {
    assert!(score.min.is_finite(), "{name}.min must be finite");
    assert!(score.max.is_finite(), "{name}.max must be finite");
    assert!(score.mean.is_finite(), "{name}.mean must be finite");
    assert!(score.max_z_score.is_finite(), "{name}.max_z_score must be finite");
    assert!(score.mean_z_score.is_finite(), "{name}.mean_z_score must be finite");
    assert!(score.baseline.mean.is_finite(), "{name}.baseline.mean must be finite");
    assert!(score.baseline.variance.is_finite(), "{name}.baseline.variance must be finite");
    assert!(score.cusum.accumulator.is_finite(), "{name}.cusum.accumulator must be finite");
    assert!(
        score.cusum.slow_baseline.mean.is_finite(),
        "{name}.cusum.slow_baseline.mean must be finite"
    );
    assert!(
        score.cusum.slow_baseline.variance.is_finite(),
        "{name}.cusum.slow_baseline.variance must be finite"
    );
    assert!(score.clip_pressure.is_finite(), "{name}.clip_pressure must be finite");

    assert!(score.max + 1e-12 >= score.min, "{name}.max must be >= min");
    assert!(score.mean + 1e-12 >= score.min, "{name}.mean must be >= min");
    assert!(score.mean <= score.max + 1e-12, "{name}.mean must be <= max");
    assert!(
        score.baseline.variance >= -1e-12,
        "{name}.baseline variance must be non-negative"
    );
    assert!(
        score.cusum.slow_baseline.variance >= -1e-12,
        "{name}.slow baseline variance must be non-negative"
    );
    assert!(
        score.cusum.accumulator >= -1e-12,
        "{name}.CUSUM is one-sided and non-negative"
    );
    assert!(
        (-1e-12..=1.0 + 1e-12).contains(&score.clip_pressure),
        "{name}.clip_pressure must be in [0, 1]"
    );
}

fn assert_scores_are_measurements(scores: &AnomalyScores) {
    assert_score_distribution("novelty", &scores.novelty);
    assert_score_distribution("displacement", &scores.displacement);
    assert_score_distribution("surprise", &scores.surprise);
    assert_score_distribution("coherence", &scores.coherence);

    assert!(
        scores.novelty.min >= -1e-12,
        "novelty is residual energy and must be non-negative"
    );
    assert!(
        scores.displacement.min >= -1e-12 && scores.displacement.max <= 1.0 + 1e-12,
        "displacement is bounded in [0, 1] (closed under float tolerance)"
    );
    assert!(scores.surprise.min >= -1e-12, "surprise must be non-negative");
    assert!(scores.coherence.min >= -1e-12, "coherence must be non-negative");
}

/// Scalar polarity contract for a single cell's contribution to a
/// coordination report.
///
/// Unlike [`assert_scores_are_measurements`], which checks a full
/// [`ScoreDistribution`] per axis, a [`MemberScore`] carries only one
/// scalar value (plus a z-score) per axis. We assert the same polarity
/// invariants as the distribution form: finiteness, non-negativity for
/// novelty/surprise/coherence, and the closed-tolerance `[0, 1]` bound
/// for displacement.
fn assert_member_polarity(member: &MemberScore<u128>) {
    assert!(member.cell_start < member.cell_end, "member score identifies a real cell");

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

    assert!(member.novelty >= -1e-12, "member novelty must be non-negative");
    assert!(
        member.displacement >= -1e-12 && member.displacement <= 1.0 + 1e-12,
        "member displacement is bounded in [0, 1] (closed under float tolerance)"
    );
    assert!(member.surprise >= -1e-12, "member surprise must be non-negative");
    assert!(member.coherence >= -1e-12, "member coherence must be non-negative");
}

fn assert_cell_report_contract(cell: &CellReport<u128>, config: &SentinelConfig<u64>) {
    assert!(cell.start < cell.end, "cell interval must be non-empty");
    assert_eq!(
        cell.analysis_width,
        128 - cell.depth as usize,
        "analysis width is the suffix width N - depth"
    );
    assert_eq!(cell.geometry.dim, cell.analysis_width, "geometry dim tracks suffix width");
    assert_eq!(
        cell.geometry.cap,
        cell.analysis_width.min(config.max_rank),
        "rank cap is min(width, max_rank)"
    );
    assert!(cell.rank >= 1, "trackers keep at least one active basis vector");
    assert!(cell.rank <= cell.geometry.cap, "rank must respect the geometry cap");
    assert_eq!(
        cell.geometry.residual_dof,
        cell.geometry.dim - cell.rank,
        "residual degrees of freedom are dim - rank"
    );
    assert!(cell.sample_count > 0, "reported cells received observations this batch");
    assert!(
        cell.maturity.total_observations() > 0,
        "reported cells have a maturity history"
    );
    assert!(
        (0.0..=1.0).contains(&cell.maturity.noise_influence),
        "noise influence is a fraction"
    );

    if config.per_sample_scores {
        let per_sample = cell.per_sample.as_ref().expect("per-sample scores are enabled");
        assert_eq!(per_sample.len(), cell.sample_count, "one sample score per routed observation");
    } else {
        assert!(cell.per_sample.is_none(), "per-sample scores are disabled");
    }

    assert_scores_are_measurements(&cell.scores);
}

fn assert_coordination_contract(coordination: &CoordinationReport<u128>, config: &SentinelConfig<u64>) {
    assert!(
        coordination.start < coordination.end,
        "coordination interval must be non-empty"
    );
    assert!(
        coordination.cells_reporting >= 2,
        "coordination needs cells from both subtrees"
    );
    assert_eq!(
        coordination.geometry.dim, 4,
        "coordination trackers operate on four score axes"
    );
    assert_eq!(
        coordination.geometry.cap,
        config.max_rank.min(4),
        "coordination rank cap is min(4, max_rank)"
    );
    assert!(coordination.rank >= 1, "coordination rank must be at least one");
    assert!(
        coordination.rank <= coordination.geometry.cap,
        "coordination rank respects cap"
    );
    assert_eq!(
        coordination.geometry.residual_dof,
        coordination.geometry.dim - coordination.rank,
        "coordination residual DOF is dim - rank"
    );

    if config.per_sample_scores {
        let members = coordination.per_member.as_ref().expect("per-member scores are enabled");
        assert_eq!(
            members.len(),
            coordination.cells_reporting,
            "one member score per contributing cell"
        );
        for member in members {
            assert_member_polarity(member);
        }
    }

    assert_scores_are_measurements(&coordination.scores);
}

fn assert_report_contract(sentinel: &Sentinel128, report: &BatchReport<u128>, batch_size: usize) {
    let config = sentinel.config();
    assert_invariants(sentinel, report);

    assert_eq!(
        report.health.lifetime_observations,
        sentinel.lifetime_observations(),
        "inline health mirrors the sentinel counter"
    );
    assert_eq!(
        report.health.cells_tracked,
        sentinel.cells_tracked(),
        "inline health mirrors active cells"
    );
    assert_eq!(
        report.analysis_set_summary.competitive_size,
        sentinel.analysis_set().competitive_count(),
        "summary mirrors the current producing competitive set"
    );
    assert_eq!(
        report.analysis_set_summary.full_size,
        sentinel.analysis_set().total_count(),
        "summary mirrors the current producing full set"
    );
    assert!(
        report.analysis_set_summary.competitive_size <= config.analysis_k,
        "competitive analysis set respects K"
    );
    assert!(
        report.analysis_set_summary.investment_set_size >= report.analysis_set_summary.full_size,
        "investment set covers online plus warming cells"
    );
    assert_eq!(
        report.analysis_set_summary.depth_range.0, 0,
        "depth range starts at the root: the producing set is anchored at \
         depth 0 via ancestor closure (§ALGO S-8.2)"
    );

    for cell in &report.cell_reports {
        assert!(cell.is_competitive, "cell_reports are the producing competitive set");
        assert_cell_report_contract(cell, config);
    }
    for cell in &report.ancestor_reports {
        assert!(!cell.is_competitive, "ancestor_reports are ancestor-only cells");
        assert_cell_report_contract(cell, config);
    }
    for coordination in &report.coordination_reports {
        assert_coordination_contract(coordination, config);
    }

    let root = root_ancestor(report);
    assert_eq!(root.sample_count, batch_size, "root receives every observation in the batch");
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

/// Maximum number of multi-region batches Step 3 will ingest while
/// waiting for hierarchical coordination to activate. Far above the
/// typical activation round under [`pedagogy_config`]; a panic at this
/// bound indicates a real regression, not flakiness.
const MULTISCALE_MAX_ROUNDS: usize = 30;

/// Function-pointer alias used in Step 4 to project an [`AnomalyScores`]
/// value onto one of its four axis distributions without repeating the
/// full type signature.
type ScoreAxis = fn(&AnomalyScores) -> &ScoreDistribution;

// ========================================================================
//  STEP 0: CONSTRUCTION AND AUTOMATIC ROOT WARM-UP
//  §ALGO S-11, ADR-S-007, ADR-S-001
// ========================================================================

/// A fresh Sentinel has one spatial G-node and one permanent root tracker.
///
/// The root tracker is warmed automatically with synthetic noise. Noise is
/// tracker-local: it does not count as a real observation and does not touch
/// the spatial G-V Graph. This is the first visible consequence of the
/// feed-forward design.
fn step_0_fresh(sentinel: &Sentinel128) {
    heading("Step 0: Construction and Automatic Root Warm-Up");
    println!("  (§ALGO S-11, ADR-S-007, ADR-S-001)");

    assert_eq!(sentinel.graph().node_count(), 1, "fresh graph has one G-node");
    assert_eq!(sentinel.graph().terminal_count(), 1, "fresh graph has one terminal cell");
    assert_eq!(sentinel.graph().total_sum(), 0, "fresh graph has no real volume");
    assert_eq!(sentinel.cells_tracked(), 1, "only the root tracker is active");
    assert_eq!(sentinel.lifetime_observations(), 0, "noise is not real traffic");
    assert_eq!(sentinel.analysis_set().competitive_count(), 0, "root is never competitive");
    assert_eq!(
        sentinel.analysis_set().total_count(),
        1,
        "full analysis set contains the root"
    );

    let root = sentinel.graph().g_root();
    let root_cell = sentinel.inspect_cell(root).expect("root cell is always inspectable");
    assert_eq!(root_cell.depth, 0);
    assert_eq!(root_cell.analysis_width, 128);
    assert!(!root_cell.is_competitive, "root provides context, not a competitive target");
    assert!(
        root_cell.maturity.noise_observations > 0,
        "root tracker is noise-warmed at construction"
    );
    assert_eq!(root_cell.maturity.real_observations, 0, "root has no real observations yet");

    let health = sentinel.health();
    assert_eq!(health.active_trackers, 1);
    assert_eq!(health.active_coordination_contexts, 0, "coordination is demand-driven");
    assert_eq!(
        health.maturity_distribution.cold_trackers, 1,
        "noise-only trackers are still cold"
    );

    println!("Fresh state:");
    println!("  spatial graph: one root, total importance = 0");
    println!("  analysis set: root only, no competitive cells yet");
    println!("  root tracker: warmed with synthetic noise, real observations = 0");
    println!("  design point: the Sentinel measures; the host decides");
}

// ========================================================================
//  STEP 1: FIRST REAL BATCH -- FEED-FORWARD OBSERVATION
//  §ALGO S-8.1, ADR-S-002
// ========================================================================

/// The first real batch proves the feed-forward invariant at the public
/// surface: the graph's total importance and the lifetime observation
/// counter increase by exactly the number of submitted values. A second
/// probe of the same length but very different bit structure produces
/// the same accounting delta, showing that score content does not feed
/// back into spatial importance.
fn step_1_first_batch(sentinel: &mut Sentinel128) {
    heading("Step 1: First Real Batch -- Feed-Forward Observation");
    println!("  (§ALGO S-8.1, ADR-S-002)");

    // Probe A: a structurally simple batch.
    let values = cell_values(0xA, 8);
    let before_sum = sentinel.graph().total_sum();
    let report = sentinel.ingest(&values);

    assert_eq!(sentinel.graph().total_sum(), before_sum + values.len() as u64);
    assert_eq!(sentinel.lifetime_observations(), values.len() as u64);
    assert_report_contract(sentinel, &report, values.len());

    let root = root_ancestor(&report);
    let probe_a_novelty_z = max_novelty_z(&report);
    subheading("Root tracker report");
    println!("  sample_count = {}", root.sample_count);
    println!("  novelty mean = {:.6}", root.scores.novelty.mean);
    println!("  displacement mean = {:.6}", root.scores.displacement.mean);
    println!("  surprise mean = {:.6}", root.scores.surprise.mean);
    println!("  coherence mean = {:.6}", root.scores.coherence.mean);

    // Probe B: a same-sized but structurally very different batch.
    //
    // The feed-forward invariant says the graph receives Delta = 1 per
    // input value, regardless of what scores those values produce. We
    // verify it directly: submit a structurally anomalous batch of the
    // same length and assert the same spatial accounting delta. The two
    // batches will differ in score magnitudes; they must not differ in
    // their effect on graph importance per input value.
    let sum_before_b = sentinel.graph().total_sum();
    let obs_before_b = sentinel.lifetime_observations();
    let anomalous = anomalous_values(0xA, values.len());
    let anomaly_report = sentinel.ingest(&anomalous);

    assert_eq!(
        sentinel.graph().total_sum() - sum_before_b,
        values.len() as u64,
        "graph importance increments by batch length irrespective of score content"
    );
    assert_eq!(
        sentinel.lifetime_observations() - obs_before_b,
        values.len() as u64,
        "lifetime observations count input values, not score magnitudes"
    );
    assert_report_contract(sentinel, &anomaly_report, anomalous.len());

    subheading("Same volume, different content");
    println!(
        "  probe A: {} values, max novelty z = {:.6}, graph delta = {}",
        values.len(),
        probe_a_novelty_z,
        values.len(),
    );
    println!(
        "  probe B: {} values, max novelty z = {:.6}, graph delta = {}",
        anomalous.len(),
        max_novelty_z(&anomaly_report),
        anomalous.len(),
    );
    println!("  -> identical graph deltas confirm scores do not feed back into importance");

    println!();
    println!("What happened:");
    println!("  1. Every coordinate was observed by the G-V Graph with Delta = 1.");
    println!("  2. Two batches of equal length produced the same spatial accounting");
    println!("     delta despite very different score profiles.");
    println!("  3. The report contains raw measurements, not a verdict or action.");
}

// ========================================================================
//  STEP 2: CONCENTRATED TRAFFIC CREATES SPATIAL STRUCTURE
//  §ALGO S-3, §ALGO S-8, ADR-S-006
// ========================================================================

/// Concentrated traffic crosses the spatial split threshold. The graph
/// refines under volume alone, then the analysis selector recomputes the
/// competitive targets and closes them under ancestry.
fn step_2_concentrated_traffic(sentinel: &mut Sentinel128) {
    heading("Step 2: Concentrated Traffic -- Spatial Refinement");
    println!("  (§ALGO S-3, §ALGO S-8, ADR-S-006)");

    let values = cell_values(0xA, 40);
    let nodes_before = sentinel.graph().node_count();
    let terminals_before = sentinel.graph().terminal_count();
    let sum_before = sentinel.graph().total_sum();

    let report = sentinel.ingest(&values);

    assert_eq!(sentinel.graph().total_sum(), sum_before + values.len() as u64);
    assert!(
        sentinel.graph().node_count() > nodes_before,
        "concentrated traffic should split G-nodes"
    );
    assert!(
        sentinel.graph().terminal_count() > terminals_before,
        "spatial contour should gain terminal cells"
    );
    assert!(
        report.contour.splits_since_last_report > 0,
        "report exposes the structural split event"
    );
    assert!(
        report.analysis_set_summary.competitive_size > 0,
        "non-root cells become competitive targets"
    );
    assert!(
        report.cell_reports.iter().any(|cell| cell.depth > 0),
        "at least one non-root competitive cell reports"
    );
    assert_report_contract(sentinel, &report, values.len());

    subheading("Contour snapshot");
    println!("  plateaus = {}", report.contour.plateau_count);
    println!("  terminal cells = {}", report.contour.cell_count);
    println!("  total importance = {:.0}", report.contour.total_importance);
    println!("  splits since previous report = {}", report.contour.splits_since_last_report);

    subheading("Analysis set summary");
    println!("  competitive size = {}", report.analysis_set_summary.competitive_size);
    println!("  full size = {}", report.analysis_set_summary.full_size);
    println!("  investment size = {}", report.analysis_set_summary.investment_set_size);
    println!("  depth range = {:?}", report.analysis_set_summary.depth_range);

    println!();
    println!("Key point: the analysis tier follows spatial volume; it does not steer it.");
}

// ========================================================================
//  STEP 3: MULTI-SCALE REPORTS AND HIERARCHICAL COORDINATION
//  §ALGO S-8.2, §ALGO S-9, ADR-S-019
// ========================================================================

/// Multi-region traffic creates multiple competitive cells. Their G-tree
/// ancestors provide shared context, and internal nodes whose left and
/// right subtrees both contribute competitive scores can emit coordination
/// reports.
fn step_3_multiscale_reports(sentinel: &mut Sentinel128) {
    heading("Step 3: Multi-Scale Reports and Coordination");
    println!("  (§ALGO S-8.2, §ALGO S-9, ADR-S-019)");

    let values = mixed_normal_batch();
    let mut report = sentinel.ingest(&values);
    assert_report_contract(sentinel, &report, values.len());

    let mut activated_at: Option<usize> = None;
    for round in 1..=MULTISCALE_MAX_ROUNDS {
        if !report.coordination_reports.is_empty() && report.cell_reports.len() >= 2 {
            activated_at = Some(round);
            break;
        }
        report = sentinel.ingest(&values);
        assert_report_contract(sentinel, &report, values.len());
    }

    let activated_at = activated_at.unwrap_or_else(|| {
        panic!(
            "coordination did not activate within {MULTISCALE_MAX_ROUNDS} multi-region batches: \
             cell_reports={}, coordination_reports={}",
            report.cell_reports.len(),
            report.coordination_reports.len(),
        )
    });
    println!("Coordination activated after {activated_at} multi-region batch(es).");

    assert!(
        report.cell_reports.len() >= 2,
        "multi-region traffic should produce several competitive reports"
    );
    assert!(
        !report.coordination_reports.is_empty(),
        "competitive cells in both subtrees should activate coordination"
    );

    let depths: std::collections::BTreeSet<_> = all_cell_reports(&report).map(|cell| cell.depth).collect();
    assert!(depths.contains(&0), "combined reports include the root depth");
    assert!(depths.len() >= 2, "combined reports show at least two spatial scales");

    subheading("Competitive reports");
    for cell in &report.cell_reports {
        println!(
            "  GNode {:?}: [{:#034x}, {:#034x}) depth={} width={} samples={}",
            cell.gnode_id, cell.start, cell.end, cell.depth, cell.analysis_width, cell.sample_count
        );
    }

    subheading("Ancestor reports");
    for cell in &report.ancestor_reports {
        println!(
            "  GNode {:?}: [{:#034x}, {:#034x}) depth={} width={} samples={}",
            cell.gnode_id, cell.start, cell.end, cell.depth, cell.analysis_width, cell.sample_count
        );
    }

    subheading("Coordination reports");
    for coordination in &report.coordination_reports {
        println!(
            "  GNode {:?}: depth={} cells_reporting={} novelty mean={:.6}",
            coordination.gnode_id, coordination.depth, coordination.cells_reporting, coordination.scores.novelty.mean
        );
    }

    println!();
    println!("Key point: cell reports are local, ancestors are multi-scale context,");
    println!("and coordination reports measure cross-cell score patterns.");
}

// ========================================================================
//  STEP 4: SCORES ARE MEASUREMENTS, NOT OPINIONS
//  §ALGO S-6, ADR-S-001
// ========================================================================

/// The four score axes share a uniform polarity: higher means more
/// anomalous. This step uses a fresh, focused scoring probe so the
/// comparison is about the score contract rather than the long-running
/// narrative sentinel's mixed traffic history. Novelty is the strongest
/// indicator for this kind of structural anomaly and is asserted
/// strictly; the other three axes are reported for inspection.
fn step_4_scores_are_measurements() {
    heading("Step 4: Scores Are Measurements, Not Opinions");
    println!("  (§ALGO S-6, ADR-S-001)");

    let mut scoring_config = pedagogy_config();
    scoring_config.noise_seed = Some(42);
    let mut scoring_sentinel = Sentinel128::new(scoring_config).expect("valid scoring config constructs");

    let seed = cell_values(0xA, 20);
    let seed_report = scoring_sentinel.ingest(&seed);
    assert_report_contract(&scoring_sentinel, &seed_report, seed.len());
    for _ in 0..5 {
        let warm_report = scoring_sentinel.ingest(&seed);
        assert_report_contract(&scoring_sentinel, &warm_report, seed.len());
    }

    let normal = cell_values(0xA, 8);
    let normal_report = scoring_sentinel.ingest(&normal);
    assert_report_contract(&scoring_sentinel, &normal_report, normal.len());

    let anomaly = anomalous_values(0xA, 8);
    let anomaly_report = scoring_sentinel.ingest(&anomaly);
    assert_report_contract(&scoring_sentinel, &anomaly_report, anomaly.len());

    let normal_z = max_novelty_z(&normal_report);
    let anomaly_z = max_novelty_z(&anomaly_report);
    assert!(
        anomaly_z > normal_z,
        "structurally dense batch should elevate novelty: anomaly={anomaly_z:.6}, normal={normal_z:.6}"
    );

    subheading("Novelty comparison (asserted)");
    println!("  normal max novelty z = {normal_z:.6}");
    println!("  anomalous max novelty z = {anomaly_z:.6}");

    // The other three axes share the same *raw-score* polarity invariant
    // (higher means more anomalous departure), but the *z-scores* below
    // compare a batch against each cell's current baseline -- and the
    // seed batch above shaped that baseline. A "normal" batch can
    // therefore show higher z-scores on displacement, surprise, or
    // coherence than the structurally anomalous batch when the
    // anomalous batch happens to land closer to the seeded baseline
    // along those axes. This is not a polarity violation -- both
    // batches' raw scores are non-negative -- it is a reminder that
    // z-scores are relative to whatever the cell has learned. We surface
    // the numbers without asserting an ordering.
    let max_z_per_axis = |report: &BatchReport<u128>, axis: ScoreAxis| -> f64 {
        all_cell_reports(report)
            .map(|cell| axis(&cell.scores).max_z_score)
            .fold(0.0_f64, f64::max)
    };
    let axes: &[(&str, ScoreAxis)] = &[
        ("displacement", |s| &s.displacement),
        ("surprise", |s| &s.surprise),
        ("coherence", |s| &s.coherence),
    ];
    subheading("Other axes (reported, not asserted)");
    for (name, axis) in axes {
        let normal_max = max_z_per_axis(&normal_report, *axis);
        let anomaly_max = max_z_per_axis(&anomaly_report, *axis);
        println!("  {name:<13} normal max z = {normal_max:>10.6}   anomalous max z = {anomaly_max:>10.6}");
    }

    subheading("Score-axis contract");
    println!("  novelty: residual energy per residual degree of freedom, non-negative");
    println!("  displacement: bounded cell displacement, in [0, 1)");
    println!("  surprise: diagonal latent deviation, non-negative");
    println!("  coherence: off-diagonal latent deviation, non-negative");

    println!();
    println!("There is no alert threshold here. The host reads these measurements");
    println!("and decides what, if anything, they mean in its own domain.");
}

// ========================================================================
//  STEP 5: HOST-CONTROLLED TEMPORAL POLICY
//  §ALGO S-10, §ALGO S-13.4, ADR-S-002
// ========================================================================

/// Decay is the host's temporal policy hook. The call itself rescales
/// spatial importance in the G-V Graph; it does not count as real
/// traffic, and it does not directly destroy trackers. Tracker churn
/// happens later, through normal selection reconciliation when cells
/// lose enough standing to drop out of the competitive top-K.
fn step_5_host_controlled_decay(sentinel: &mut Sentinel128) {
    heading("Step 5: Host-Controlled Decay");
    println!("  (§ALGO S-10, §ALGO S-13.4, ADR-S-002)");

    let sum_before = sentinel.graph().total_sum();
    let observations_before = sentinel.lifetime_observations();
    let trackers_before = sentinel.cells_tracked();

    sentinel.decay(0.5, 0.0);

    let sum_after_decay = sentinel.graph().total_sum();
    let trackers_after_decay = sentinel.cells_tracked();
    assert!(
        sum_after_decay < sum_before,
        "uniform attenuation should reduce spatial importance"
    );
    assert_eq!(
        sentinel.lifetime_observations(),
        observations_before,
        "decay is not real traffic"
    );
    assert_eq!(
        trackers_after_decay, trackers_before,
        "decay() itself does not destroy trackers; lifecycle changes happen via later reconciliation"
    );

    let values = cell_values(0xF, 8);
    let report = sentinel.ingest(&values);
    let trackers_after_ingest = sentinel.cells_tracked();
    assert_eq!(sentinel.graph().total_sum(), sum_after_decay + values.len() as u64);
    assert_eq!(sentinel.lifetime_observations(), observations_before + values.len() as u64);
    assert_report_contract(sentinel, &report, values.len());

    subheading("Decay effect");
    println!("  total importance: before decay = {sum_before}, after decay = {sum_after_decay}");
    println!(
        "  active trackers: before decay = {trackers_before}, after decay = {trackers_after_decay} \
         (unchanged), after subsequent ingest = {trackers_after_ingest}"
    );
    println!(
        "  lifetime observations: before = {observations_before}, after ingest = {}",
        sentinel.lifetime_observations()
    );

    println!();
    println!("Key point: decay rescales spatial weight; selection reconciliation");
    println!("decides downstream tracker lifecycle. Feedback, when desired,");
    println!("belongs in host policy.");
}

// ========================================================================
//  STEP 6: RESET
//  Public API lifecycle contract
// ========================================================================

/// Reset clears the spatial graph, trackers, coordination contexts, and
/// counters, then recreates the warmed root according to the same config.
fn step_6_reset(sentinel: &mut Sentinel128) {
    heading("Step 6: Reset Restores the Fresh Lifecycle");

    assert!(
        sentinel.graph().total_sum() > 0,
        "the narrative produced real traffic before reset"
    );
    assert!(
        sentinel.lifetime_observations() > 0,
        "the narrative counted real observations before reset"
    );

    sentinel.reset();

    assert_eq!(sentinel.graph().node_count(), 1);
    assert_eq!(sentinel.graph().terminal_count(), 1);
    assert_eq!(sentinel.graph().total_sum(), 0);
    assert_eq!(sentinel.cells_tracked(), 1);
    assert_eq!(sentinel.lifetime_observations(), 0);
    assert_eq!(sentinel.health().active_coordination_contexts, 0);

    let root = sentinel.graph().g_root();
    let root_cell = sentinel.inspect_cell(root).expect("root is recreated on reset");
    assert_eq!(root_cell.depth, 0);
    assert_eq!(root_cell.analysis_width, 128);
    assert!(root_cell.maturity.noise_observations > 0, "reset recreates the warmed root");

    println!("After reset:");
    println!("  graph: one root, total importance = 0");
    println!("  trackers: root only");
    println!("  coordination contexts: none");
    println!("  root warm-up: reapplied from the configured noise schedule");
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
            println!("pedagogy: test");
        }
        return;
    }

    let config = pedagogy_config();
    config.validate().expect("pedagogy config must be valid");

    let mut sentinel = Sentinel128::new(config).expect("valid config constructs a Sentinel");

    step_0_fresh(&sentinel);
    step_1_first_batch(&mut sentinel);
    step_2_concentrated_traffic(&mut sentinel);
    step_3_multiscale_reports(&mut sentinel);
    step_4_scores_are_measurements();
    step_5_host_controlled_decay(&mut sentinel);
    step_6_reset(&mut sentinel);

    heading("All Claims Verified");
    println!();
    println!("  Feed-forward invariant:");
    println!("    [ok] graph importance increases by one per input value");
    println!("    [ok] same-length batches produce identical spatial accounting");
    println!("         deltas regardless of score content");
    println!();
    println!("  Analysis lifecycle:");
    println!("    [ok] automatic noise warm-up is tracker-local");
    println!("    [ok] volume-driven spatial splits create competitive analysis cells");
    println!("    [ok] ancestors keep a complete multi-scale chain to the root");
    println!("    [ok] coordination activates from cross-cell score patterns");
    println!();
    println!("  Reporting principle:");
    println!("    [ok] report values are raw, finite statistical measurements");
    println!("    [ok] no test asserts a policy verdict or host action");
    println!();
    println!("  Host-controlled lifecycle:");
    println!("    [ok] decay changes spatial importance without counting traffic");
    println!("    [ok] reset restores a fresh graph and warmed root tracker");
}
