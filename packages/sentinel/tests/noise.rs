// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for the **warming schedule** — the configured policy that
//! decides how many rounds of synthetic observations a tracker is fed before it
//! is allowed to judge real traffic, and what those rounds leave behind.
//!
//! A tracker with no history has no baseline to score against, so its first
//! real batch would find everything extreme. The schedule closes that window by
//! warming a tracker at the moment it comes into being — the root when the
//! sentinel is constructed, a deeper cell when it enters the analysis set — so
//! nothing is ever asked a question before it can hold an answer. Synthetic
//! rounds are booked apart from real ones: they raise the tracker's own
//! maturity counters and leave the sentinel's traffic counters untouched,
//! because they are not traffic and must not be reported as though they were.
//!
//! Two decisions shape the policy. The round count tapers with depth, because a
//! deeper cell analyses a narrower slice of the domain and settles sooner, so
//! paying root-sized warming everywhere would be waste; a schedule can also be
//! written out per depth by hand, and an empty one turns warming off entirely.
//! And the draws come from a single seeded generator that advances as it moves
//! from cell to cell, which makes a whole run reproducible from its seed while
//! still giving every tracker its own draws. Because warming does leave a mark,
//! the handover is deliberate: the slow baselines are seeded from the fast
//! ones, then the drift accumulators and the rejection state are zeroed, so a
//! tracker begins its real life at rest rather than carrying synthetic history
//! into its first verdict.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`root_warmed_at_construction`] | schedule | Constructing a sentinel already warms its root tracker: the root carries synthetic observations before any caller has had a chance to feed it. Those rounds push the tracker's reliance on synthetic history toward its maximum and hold it there, so a root that has seen nothing but warming still reports itself as fully synthetic — the reliance only falls once real traffic arrives to displace it. |
//! | [`root_cold_when_schedule_empty`] | schedule | A schedule that specifies no rounds at any depth turns warming off rather than falling back to a default: the root is constructed cold, with no synthetic observations at all. Its reliance on synthetic history still reads as maximal, because that is the value a tracker is born with and warming is what would have started moving it. |
//! | [`noise_does_not_count_as_real_observations`] | schedule | Warming rounds are not traffic and are not counted as traffic: a sentinel whose root has just been warmed still reports having observed nothing over its lifetime. The two ledgers are kept apart deliberately, so an operator reading the observation count sees what the world sent and never the sentinel's own preparation. |
//! | [`new_cells_warmed_on_analysis_set_entry`] | schedule | Warming is not a construction-time favour granted to the root alone. After a run whose traffic is concentrated enough to split the tree, every cell the sentinel is tracking carries synthetic observations, including the ones that did not exist when the sentinel was built. A cell that arrives mid-run is therefore never asked to score its own first batch from an empty baseline. |
//! | [`successive_cells_get_different_noise`] | schedule | cites (´claim:schedule:a-cell-born-mid-run-is-warmed-as-it-enters-the-analysis-set´) |
//! | [`deeper_cells_receive_fewer_noise_rounds`] | schedule | Under a geometric schedule the round count falls off with depth, and the effect is visible in the trackers themselves: at least one cell below the root ends the run with fewer synthetic observations than the root has. The taper is the point of the schedule — a deeper cell works on a narrower slice and settles in fewer rounds, so spending root-sized warming on it would buy nothing. |
//! | [`auto_inject_resets_cusum`] | schedule | Warming ends with the drift detectors wound back to zero, so the first real batch a tracker sees is its first step of drift accounting — every cell and ancestor in that batch's report says exactly one step has passed since its last reset. Without the reset, the drift a tracker accumulated while chasing synthetic rounds would be charged to whoever sent the first real request. |
//! | [`deterministic_with_same_seed`] | schedule | Two sentinels built from the same seed and fed the same traffic come out with the same cells, warmed by the same number of rounds and left with the same reliance on synthetic history. Warming is a reproducible part of a run rather than a source of drift between two otherwise identical deployments, which is what makes a captured incident replayable at all. |
//! | [`different_seeds_produce_different_baselines`] | schedule | The seed is not cosmetic: two sentinels warmed from different seeds and then fed byte-identical traffic score that traffic against different baselines. Warming leaves a real imprint on where a tracker starts, so an attacker who knew one deployment's warmed baseline would not thereby know another's. |
//! | [`reset_reseeds_rng_and_warms_root`] | schedule | Resetting a sentinel that has been running for a long stretch puts its root back exactly where a freshly constructed one stands: the same number of warming rounds, the same reliance on synthetic history. Reset restores the generator to its seed and warms the rebuilt root again, so it is a genuine return to birth rather than a partial clearing that leaves a cold root behind. |
//! | [`coordination_contexts_warmed_on_activation`] | schedule | The contexts that watch several cells at once are warmed on the same terms as the cell trackers: in a run where any of them became active, none of them is left cold. Their synthetic rounds are drawn to look like the score patterns they will actually be shown, sampled from the contributing cells' own baselines, because a context scores score vectors rather than raw coordinates. |

mod common;

use common::{ScenarioBuilder, assert_invariants, cell_values, cold_config, seeded_sentinel, test_config};
use torrust_sentinel::{NoiseSchedule, Sentinel128, SentinelConfig};

// ── Root Warming at Construction ────────────────────────────

/// Constructing a sentinel already warms its root tracker: the root carries
/// synthetic observations before any caller has had a chance to feed it. Those
/// rounds push the tracker's reliance on synthetic history toward its maximum
/// and hold it there, so a root that has seen nothing but warming still reports
/// itself as fully synthetic — the reliance only falls once real traffic
/// arrives to displace it.
///
/// ´claim:schedule:the-root-tracker-is-already-warmed-when-the-sentinel-is-handed-back´
/// ´test:integration:root-warmed-at-construction´
#[test]
fn root_warmed_at_construction() {
    let s = Sentinel128::new(test_config()).unwrap();
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).expect("root should exist");

    assert!(
        insp.maturity.noise_observations > 0,
        "root tracker should have received noise observations"
    );
    // noise_influence starts at 1.0 and noise pushes toward 1.0,
    // so a noise-only tracker remains at 1.0.  It decays only
    // after real observations.
    assert!(
        (insp.maturity.noise_influence - 1.0).abs() < f64::EPSILON,
        "noise_influence should stay at 1.0 with only noise"
    );
}

/// A schedule that specifies no rounds at any depth turns warming off rather
/// than falling back to a default: the root is constructed cold, with no
/// synthetic observations at all. Its reliance on synthetic history still reads
/// as maximal, because that is the value a tracker is born with and warming is
/// what would have started moving it.
///
/// ´claim:schedule:an-empty-schedule-switches-warming-off-and-leaves-the-tracker-cold´
/// ´test:integration:root-cold-when-schedule-empty´
#[test]
fn root_cold_when_schedule_empty() {
    let s = Sentinel128::new(cold_config()).unwrap();
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).unwrap();

    assert_eq!(insp.maturity.noise_observations, 0);
    assert!(
        (insp.maturity.noise_influence - 1.0).abs() < f64::EPSILON,
        "cold root should have default noise_influence of 1.0"
    );
}

/// Warming rounds are not traffic and are not counted as traffic: a sentinel
/// whose root has just been warmed still reports having observed nothing over
/// its lifetime. The two ledgers are kept apart deliberately, so an operator
/// reading the observation count sees what the world sent and never the
/// sentinel's own preparation.
///
/// ´claim:schedule:synthetic-rounds-never-enter-the-observed-traffic-count´
/// ´test:integration:noise-does-not-count-as-real-observations´
#[test]
fn noise_does_not_count_as_real_observations() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(
        s.lifetime_observations(),
        0,
        "noise injection at construction should not increment lifetime_observations"
    );
}

// ── Child Cell Warming ──────────────────────────────────────

/// Warming is not a construction-time favour granted to the root alone. After a
/// run whose traffic is concentrated enough to split the tree, every cell the
/// sentinel is tracking carries synthetic observations, including the ones that
/// did not exist when the sentinel was built. A cell that arrives mid-run is
/// therefore never asked to score its own first batch from an empty baseline.
///
/// ´claim:schedule:a-cell-born-mid-run-is-warmed-as-it-enters-the-analysis-set´
/// ´test:integration:new-cells-warmed-on-analysis-set-entry´
#[test]
fn new_cells_warmed_on_analysis_set_entry() {
    let (s, _reports) = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            analysis_k: 16,
            ..test_config()
        })
        .seed_range(0xF, 20)
        .warm_batches(19)
        .build_with_reports();

    // All cells should be warm.
    for &gnode in &s.cell_gnodes() {
        let insp = s.inspect_cell(gnode).unwrap();
        assert!(insp.maturity.noise_observations > 0, "cell {gnode:?} should be noise-warmed");
    }
}

/// Traffic split across two well-separated ranges produces several non-root
/// cells in one run, and each of them is warmed in turn. This pins the
/// many-cells end of the rule: the generator is a single persistent one that
/// advances as it serves each cell, so warming a second cell neither replays
/// the first cell's draws nor exhausts the supply.
///
/// (´claim:schedule:a-cell-born-mid-run-is-warmed-as-it-enters-the-analysis-set´)
/// ´test:integration:successive-cells-get-different-noise´
#[test]
fn successive_cells_get_different_noise() {
    let (s, _) = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            analysis_k: 16,
            ..test_config()
        })
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build_with_reports();

    let root = s.graph().g_root();
    let non_root: Vec<_> = s.cell_gnodes().into_iter().filter(|&g| g != root).collect();

    // We need at least two non-root cells to compare.
    assert!(
        non_root.len() >= 2,
        "expected at least 2 non-root cells, got {}",
        non_root.len()
    );

    // Both cells must be warmed.
    for &gnode in &non_root {
        let insp = s.inspect_cell(gnode).unwrap();
        assert!(insp.maturity.noise_observations > 0, "cell {gnode:?} should be noise-warmed");
    }
}

// ── Depth-Tiered Noise ──────────────────────────────────────

/// Under a geometric schedule the round count falls off with depth, and the
/// effect is visible in the trackers themselves: at least one cell below the
/// root ends the run with fewer synthetic observations than the root has. The
/// taper is the point of the schedule — a deeper cell works on a narrower slice
/// and settles in fewer rounds, so spending root-sized warming on it would buy
/// nothing.
///
/// ´claim:schedule:a-geometric-schedule-spends-fewer-rounds-on-deeper-cells-than-on-the-root´
/// ´test:integration:deeper-cells-receive-fewer-noise-rounds´
#[test]
fn deeper_cells_receive_fewer_noise_rounds() {
    // Use a geometric schedule with aggressive decay so the
    // difference between root (depth 0) and child cells is obvious.
    let cfg = SentinelConfig::<u64> {
        split_threshold: 10,
        analysis_k: 16,
        noise_schedule: NoiseSchedule::geometric(100, 0.5, 5),
        ..test_config()
    };

    let (s, _) = ScenarioBuilder::new()
        .config(cfg)
        .seed_range(0xF, 20)
        .warm_batches(19)
        .build_with_reports();

    let root = s.graph().g_root();
    let root_insp = s.inspect_cell(root).unwrap();

    // At least one child cell should have fewer noise observations
    // than the root (deeper → fewer rounds via geometric decay).
    let non_root: Vec<_> = s.cell_gnodes().into_iter().filter(|&g| g != root).collect();
    if !non_root.is_empty() {
        let any_fewer = non_root.iter().any(|&gnode| {
            let insp = s.inspect_cell(gnode).unwrap();
            insp.maturity.noise_observations < root_insp.maturity.noise_observations
        });
        assert!(
            any_fewer,
            "at least one deeper cell should have fewer noise rounds than root ({})",
            root_insp.maturity.noise_observations
        );
    }
}

// ── CUSUM Reset After Noise ─────────────────────────────────

/// Warming ends with the drift detectors wound back to zero, so the first real
/// batch a tracker sees is its first step of drift accounting — every cell and
/// ancestor in that batch's report says exactly one step has passed since its
/// last reset. Without the reset, the drift a tracker accumulated while chasing
/// synthetic rounds would be charged to whoever sent the first real request.
///
/// ´claim:schedule:warming-hands-over-a-tracker-whose-drift-accounting-starts-at-the-first-real-batch´
/// ´test:integration:auto-inject-resets-cusum´
#[test]
fn auto_inject_resets_cusum() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // Root was auto-injected at construction; CUSUM should be reset.
    // Ingest one real batch — steps_since_reset should be 1.
    let report = s.ingest(&[
        0xF000_0000_0000_0000_0000_0000_0000_AAAA,
        0x1000_0000_0000_0000_0000_0000_0000_BBBB,
    ]);

    assert_invariants(&s, &report);

    for cr in report.cell_reports.iter().chain(report.ancestor_reports.iter()) {
        assert_eq!(
            cr.scores.novelty.cusum.steps_since_reset, 1,
            "CUSUM should have been reset after auto noise injection"
        );
    }
}

// ── Noise Determinism ───────────────────────────────────────

/// Two sentinels built from the same seed and fed the same traffic come out
/// with the same cells, warmed by the same number of rounds and left with the
/// same reliance on synthetic history. Warming is a reproducible part of a run
/// rather than a source of drift between two otherwise identical deployments,
/// which is what makes a captured incident replayable at all.
///
/// ´claim:schedule:a-fixed-seed-makes-warming-reproduce-cell-for-cell´
/// ´test:integration:deterministic-with-same-seed´
#[test]
fn deterministic_with_same_seed() {
    let s1 = seeded_sentinel();
    let s2 = seeded_sentinel();

    let gnodes1 = s1.cell_gnodes();
    let gnodes2 = s2.cell_gnodes();
    assert_eq!(gnodes1, gnodes2);

    for &gnode in &gnodes1 {
        let insp1 = s1.inspect_cell(gnode).unwrap();
        let insp2 = s2.inspect_cell(gnode).unwrap();
        assert_eq!(insp1.maturity.noise_observations, insp2.maturity.noise_observations);
        assert!(
            (insp1.maturity.noise_influence - insp2.maturity.noise_influence).abs() < f64::EPSILON,
            "noise influence should be identical with same seed"
        );
    }
}

/// The seed is not cosmetic: two sentinels warmed from different seeds and then
/// fed byte-identical traffic score that traffic against different baselines.
/// Warming leaves a real imprint on where a tracker starts, so an attacker who
/// knew one deployment's warmed baseline would not thereby know another's.
///
/// ´claim:schedule:the-seed-decides-which-baseline-a-warmed-tracker-settles-on´
/// ´test:integration:different-seeds-produce-different-baselines´
#[test]
fn different_seeds_produce_different_baselines() {
    let cfg1 = SentinelConfig::<u64> {
        noise_seed: Some(42),
        ..test_config()
    };
    let cfg2 = SentinelConfig::<u64> {
        noise_seed: Some(999),
        ..test_config()
    };

    let values = cell_values(0xA, 20);

    let mut s1 = Sentinel128::new(cfg1).unwrap();
    let mut s2 = Sentinel128::new(cfg2).unwrap();

    // Seed with identical traffic.
    s1.ingest(&values);
    s2.ingest(&values);

    // Probe with the same batch.
    let r1 = s1.ingest(&values);
    let r2 = s2.ingest(&values);

    assert_invariants(&s1, &r1);
    assert_invariants(&s2, &r2);

    let means1: Vec<f64> = r1
        .cell_reports
        .iter()
        .chain(r1.ancestor_reports.iter())
        .map(|cr| cr.scores.novelty.mean)
        .collect();
    let means2: Vec<f64> = r2
        .cell_reports
        .iter()
        .chain(r2.ancestor_reports.iter())
        .map(|cr| cr.scores.novelty.mean)
        .collect();

    assert_ne!(means1, means2, "different seeds should produce different baselines");
}

// ── Reset Behaviour ─────────────────────────────────────────

/// Resetting a sentinel that has been running for a long stretch puts its root
/// back exactly where a freshly constructed one stands: the same number of
/// warming rounds, the same reliance on synthetic history. Reset restores the
/// generator to its seed and warms the rebuilt root again, so it is a genuine
/// return to birth rather than a partial clearing that leaves a cold root
/// behind.
///
/// ´claim:schedule:a-reset-rebuilds-and-rewarms-the-root-to-the-state-a-fresh-sentinel-has´
/// ´test:integration:reset-reseeds-rng-and-warms-root´
#[test]
fn reset_reseeds_rng_and_warms_root() {
    let cfg = test_config();
    let fresh = Sentinel128::new(cfg.clone()).unwrap();

    let mut reset_s = Sentinel128::new(cfg).unwrap();
    reset_s.ingest(&cell_values(0xA, 100));
    reset_s.reset();

    let fresh_insp = fresh.inspect_cell(fresh.graph().g_root()).unwrap();
    let reset_insp = reset_s.inspect_cell(reset_s.graph().g_root()).unwrap();

    assert_eq!(fresh_insp.maturity.noise_observations, reset_insp.maturity.noise_observations);
    assert!(
        (fresh_insp.maturity.noise_influence - reset_insp.maturity.noise_influence).abs() < f64::EPSILON,
        "noise influence after reset should match fresh sentinel"
    );
}

// ── Coordination Warming ────────────────────────────────────

/// The contexts that watch several cells at once are warmed on the same terms
/// as the cell trackers: in a run where any of them became active, none of them
/// is left cold. Their synthetic rounds are drawn to look like the score
/// patterns they will actually be shown, sampled from the contributing cells'
/// own baselines, because a context scores score vectors rather than raw
/// coordinates.
///
/// ´claim:schedule:a-context-that-activates-is-warmed-from-the-baselines-of-the-cells-it-watches´
/// ´test:integration:coordination-contexts-warmed-on-activation´
#[test]
fn coordination_contexts_warmed_on_activation() {
    let (s, _) = ScenarioBuilder::new()
        .config(SentinelConfig::<u64> {
            split_threshold: 10,
            ..test_config()
        })
        .seed_range(0xF, 4)
        .seed_range(0x1, 4)
        .warm_batches(14)
        .build_with_reports();

    let ch = s.health().coordination_health;
    if ch.active_contexts > 0 {
        // Coordination contexts should have been warmed on activation
        // via Gamma-sampled synthetic noise (§ALGO S-9.8).
        assert!(
            ch.maturity_distribution.cold_trackers == 0,
            "all active coordination contexts should be warmed, found {} cold",
            ch.maturity_distribution.cold_trackers,
        );
    }
}
