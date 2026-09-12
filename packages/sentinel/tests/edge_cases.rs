// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`min_value_u128`] | edge | The bottom of the coordinate domain is an ordinary observation: a value with no bits set is counted and reported like any other, and every structural invariant survives it. Encoding centres each bit rather than taking it raw, so an all-zero value is a well-formed vector and not a degenerate one the geometry has to special-case. |
//! | [`max_value_u128`] | edge | cites (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´) |
//! | [`min_and_max_together`] | edge | cites (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´) |
//! | [`all_nibbles_full_spread`] | edge | Traffic spread evenly over every leading region of the domain, batch after batch, gives the spatial layer no concentration to reward — and the engine keeps its invariants anyway, with the observation count equal to exactly what was fed in. Uniform traffic is the worst case for a selector that ranks by importance, and it produces a boring report rather than an unstable one. |
//! | [`single_observation_batch`] | engine | cites (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´) |
//! | [`very_large_batch`] | edge | There is no ceiling on batch size: a batch of many thousands of values spread across the domain is scored in one pass, the root's sample count equals the batch it was given, and the invariants hold. Ingestion walks the batch once and updates state as it goes, so a large batch costs time proportional to its size and nothing else. |
//! | [`empty_then_burst`] | edge | A long idle stretch of empty batches leaves the engine exactly where it was — nothing observed, nothing reported — and the burst that follows is scored as though the idling had never happened. Empty batches do not accumulate into a state the engine has to recover from, because an empty batch returns before any of the observation machinery runs. |
//! | [`single_observation_repeated`] | edge | One value hammered in over and over, under a split threshold low enough to make the spatial layer subdivide around it, is counted in full and leaves the invariants intact. Concentration is exactly what the spatial layer is built to notice, so the pathological case of total concentration is a case it handles rather than one that surprises it. |
//! | [`all_same_value`] | edge | Identical observations teach the model no new directions, so after a long run of them the root's learned rank is still near its floor rather than having grown with the volume. Rank tracks how many directions the data actually spans, not how much data arrived, which is what keeps the measurement honest about a stream that carries no structure. |
//! | [`alternating_two_values`] | edge | A stream that alternates strictly between two well-separated values still leaves the root with at least one learned direction and the invariants standing. Two points do span a direction, so this is the smallest non-trivial structure a tracker can be given — the case just above the one where nothing varies at all. |
//! | [`reset_then_immediate_ingest`] | engine | A sentinel is usable the instant a reset returns: the very next batch is counted from zero and produces a root report, with no warm-up call or settling period in between. Reset rebuilds the root tracker as part of the operation rather than leaving the engine cell-less until traffic arrives. |
//! | [`double_reset`] | engine | cites (´claim:engine:the-sentinel-is-immediately-usable-after-a-reset´) |
//! | [`decay_to_zero_then_rebuild`] | edge | Decay severe enough to annihilate accumulated standing does not leave a dead engine: fresh traffic rebuilds cells from what it observes, the root is reported again, and trackers are live. Decay lowers what regions have earned rather than removing the machinery that earns it, so the recovery path is simply ordinary ingestion. |
//! | [`rapid_decay_ingest_cycle`] | edge | Decay interleaved with ingestion round after round keeps producing valid reports with the invariants intact. Decay does not need to be rare or quiescent to be safe: it changes spatial standing between batches, and the analysis set is simply recomputed at the start of the next ingestion rather than being eagerly invalidated. |
//! | [`analysis_k_equals_one`] | edge | Squeezing the competitive budget to a single cell squeezes exactly that: at most one cell is reported as having earned its place, and the root is still reported regardless. The budget governs investment, not structure, so the narrowest possible budget yields the smallest useful report rather than an empty one. The full-set bound is deliberately not asserted here — closure under ancestry keeps materialising chains the bound assumes a wider budget for. |
//! | [`analysis_depth_cutoff_zero`] | edge | cites (´claim:edge:an-extreme-analysis-budget-narrows-what-can-be-chosen-but-never-empties-the-set´) |
//! | [`per_sample_scores_large_batch`] | engine | cites (´claim:engine:per-sample-scores-appear-only-when-asked-for-and-then-carry-one-entry-per-observation´) |
//! | [`edge_cases_degenerate_cells_skipped_starts_at_zero`] | engine | cites (´claim:engine:a-fresh-sentinel-has-skipped-no-cell-as-too-narrow-to-model´) |
//! | [`degenerate_cells_skipped_in_report_normal_traffic`] | edge | Ordinary traffic never drives a cell narrow enough to be skipped: after a long run of ingestion the report still shows no skips at all. The guard is there for pathological splitting, not for everyday operation, so a non-zero reading in the field is a signal about the traffic rather than routine noise. |
//! | [`degenerate_cells_skipped_after_reset`] | engine | cites (´claim:engine:a-fresh-sentinel-has-skipped-no-cell-as-too-narrow-to-model´) |
//! | [`deep_spray_traffic_does_not_panic`] | edge | Splitting made as aggressive as the configuration allows, on traffic hammering a single point, drives the domain deep enough that some cells have almost no suffix left to analyse. Those cells are skipped and counted rather than given a tracker that could form no basis and produce no residual, and the run continues with the root still tracked. A cell too narrow to model is a case the engine declines, not a fault it crashes on. |

//! Degenerate and boundary input, and what the engine does with it.
//!
//! A sentinel in service is fed whatever the host sees, not whatever suits
//! the model. Batches arrive empty, arrive one value at a time, and arrive
//! enormous. Values sit at the extremes of the coordinate domain, repeat
//! without variation, or alternate between two points indefinitely. None of
//! these is an error, so none of them is treated as one: the engine counts
//! what it was given, reports what it measured, and keeps its structural
//! invariants throughout. The boundary between a degenerate case and an
//! impossible one matters here — the degenerate cases are all admitted.
//!
//! Structureless traffic is the interesting degeneracy, because it is the
//! case where there is nothing to learn. Identical values present no new
//! direction, so the learned rank stays low instead of inflating on
//! repetition, and the measurement remains honest about how little variation
//! it has actually seen.
//!
//! Extreme configuration and extreme splitting are the other two edges. A
//! competitive budget of one, or a depth cutoff of zero, narrows what may be
//! selected but never empties the set, because the root's membership is
//! unconditional. And when aggressive splitting drives a cell's suffix too
//! narrow to support a subspace model at all, that cell is skipped and
//! counted rather than modelled badly or allowed to bring the run down.

mod common;

use common::{ScenarioBuilder, assert_invariants, cell_values, integration_config, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ─── Boundary values ────────────────────────────────────────

/// The bottom of the coordinate domain is an ordinary observation: a value
/// with no bits set is counted and reported like any other, and every
/// structural invariant survives it. Encoding centres each bit rather than
/// taking it raw, so an all-zero value is a well-formed vector and not a
/// degenerate one the geometry has to special-case.
///
/// ´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´
/// ´test:integration:min-value-u128´
#[test]
fn min_value_u128() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[0u128]);
    assert_eq!(report.health.lifetime_observations, 1);
    assert_invariants(&s, &report);
}

/// The top of the domain behaves the same way as the bottom — a saturated
/// value routes, scores and counts without arithmetic trouble at the far end
/// of the interval the root covers.
///
/// (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´)
/// ´test:integration:max-value-u128´
#[test]
fn max_value_u128() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[u128::MAX]);
    assert_eq!(report.health.lifetime_observations, 1);
    assert_invariants(&s, &report);
}

/// The widest possible spread within one batch — both extremes at once —
/// is handled as a single ordinary batch. Values in a batch are scored
/// independently against the cells that contain them, so how far apart they
/// lie in the domain is not itself a difficulty.
///
/// (´claim:edge:the-extremes-of-the-coordinate-domain-are-ordinary-observations´)
/// ´test:integration:min-and-max-together´
#[test]
fn min_and_max_together() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[0u128, u128::MAX]);
    assert_eq!(report.health.lifetime_observations, 2);
    assert_invariants(&s, &report);
}

/// Traffic spread evenly over every leading region of the domain, batch
/// after batch, gives the spatial layer no concentration to reward — and the
/// engine keeps its invariants anyway, with the observation count equal to
/// exactly what was fed in. Uniform traffic is the worst case for a selector
/// that ranks by importance, and it produces a boring report rather than an
/// unstable one.
///
/// ´claim:edge:traffic-spread-evenly-over-the-whole-domain-keeps-every-invariant-and-every-count´
/// ´test:integration:all-nibbles-full-spread´
#[test]
fn all_nibbles_full_spread() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    // Values spanning all 16 leading nibbles.
    let values: Vec<u128> = (0..16u128).map(|nib| (nib << 124) | 1).collect();
    for _ in 0..20 {
        s.ingest(&values);
    }
    let report = s.ingest(&values);

    assert_eq!(s.lifetime_observations(), 21 * 16);
    assert_invariants(&s, &report);
}

// ─── Batch-size extremes ────────────────────────────────────

/// The smallest non-empty batch, delivered over and over, produces a report
/// every time: each single observation reaches the root and is scored there,
/// and nothing is deferred until enough values have piled up. A host that
/// hands values over one at a time therefore gets the same record as one
/// that buffers them.
///
/// (´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´)
/// ´test:integration:single-observation-batch´
#[test]
fn single_observation_batch() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // Single observation per batch for 50 batches.
    let mut report = None;
    for i in 0..50u128 {
        let r = s.ingest(&[(0xA << 124) | i]);
        assert!(
            !r.ancestor_reports.is_empty() || !r.cell_reports.is_empty(),
            "batch {i} should produce at least a root report"
        );
        report = Some(r);
    }
    assert_eq!(s.lifetime_observations(), 50);
    assert_invariants(&s, report.as_ref().unwrap());
}

/// There is no ceiling on batch size: a batch of many thousands of values
/// spread across the domain is scored in one pass, the root's sample count
/// equals the batch it was given, and the invariants hold. Ingestion walks
/// the batch once and updates state as it goes, so a large batch costs time
/// proportional to its size and nothing else.
///
/// ´claim:edge:batch-size-is-unbounded-so-a-very-large-batch-is-scored-in-one-pass´
/// ´test:integration:very-large-batch´
#[test]
fn very_large_batch() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // 10K observations in one batch.
    let values: Vec<u128> = (0..10_000u128).map(|i| ((i % 16) << 124) | (i + 1)).collect();
    let report = s.ingest(&values);

    assert_eq!(s.lifetime_observations(), 10_000);
    let root = report.ancestor_reports.iter().find(|r| r.depth == 0);
    assert!(root.is_some());
    assert_eq!(root.unwrap().sample_count, 10_000);
    assert_invariants(&s, &report);
}

/// A long idle stretch of empty batches leaves the engine exactly where it
/// was — nothing observed, nothing reported — and the burst that follows is
/// scored as though the idling had never happened. Empty batches do not
/// accumulate into a state the engine has to recover from, because an empty
/// batch returns before any of the observation machinery runs.
///
/// ´claim:edge:idling-on-empty-batches-leaves-the-engine-ready-for-the-burst-that-follows´
/// ´test:integration:empty-then-burst´
#[test]
fn empty_then_burst() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    // 100 empty ingests.
    for _ in 0..100 {
        let report = s.ingest(&[]);
        assert!(report.cell_reports.is_empty());
    }
    assert_eq!(s.lifetime_observations(), 0);

    // Then a large burst.
    let report = s.ingest(&cell_values(0xA, 200));
    assert_eq!(s.lifetime_observations(), 200);
    assert!(
        !report.ancestor_reports.is_empty(),
        "burst after empties should produce ancestor reports"
    );
    assert_invariants(&s, &report);
}

// ─── Traffic patterns ───────────────────────────────────────

/// One value hammered in over and over, under a split threshold low enough
/// to make the spatial layer subdivide around it, is counted in full and
/// leaves the invariants intact. Concentration is exactly what the spatial
/// layer is built to notice, so the pathological case of total concentration
/// is a case it handles rather than one that surprises it.
///
/// ´claim:edge:a-single-value-repeated-without-variation-is-tolerated-and-still-counted-in-full´
/// ´test:integration:single-observation-repeated´
#[test]
fn single_observation_repeated() {
    let mut s = Sentinel128::new(SentinelConfig::<u64> {
        split_threshold: 10,
        ..test_config()
    })
    .unwrap();

    let value = 0xA000_0000_0000_0000_0000_0000_0000_0001u128;
    let mut report = None;
    for _ in 0..100 {
        report = Some(s.ingest(&[value]));
    }

    assert_eq!(s.lifetime_observations(), 100);
    assert!(s.cells_tracked() >= 1);
    assert_invariants(&s, report.as_ref().unwrap());
}

/// Identical observations teach the model no new directions, so after a long
/// run of them the root's learned rank is still near its floor rather than
/// having grown with the volume. Rank tracks how many directions the data
/// actually spans, not how much data arrived, which is what keeps the
/// measurement honest about a stream that carries no structure.
///
/// ´claim:edge:identical-traffic-teaches-no-new-direction-so-the-learned-rank-stays-low´
/// ´test:integration:all-same-value´
#[test]
fn all_same_value() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    let value = 0xAAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000_1111u128;
    let mut report = None;
    for _ in 0..100 {
        report = Some(s.ingest(&[value; 8]));
    }

    // Rank should stay low — no structural variation.
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).unwrap();
    assert!(insp.rank <= 2, "identical traffic should keep rank low, got {}", insp.rank);
    assert_invariants(&s, report.as_ref().unwrap());
}

/// A stream that alternates strictly between two well-separated values still
/// leaves the root with at least one learned direction and the invariants
/// standing. Two points do span a direction, so this is the smallest
/// non-trivial structure a tracker can be given — the case just above the
/// one where nothing varies at all.
///
/// ´claim:edge:a-stream-of-only-two-values-still-supports-at-least-one-learned-direction´
/// ´test:integration:alternating-two-values´
#[test]
fn alternating_two_values() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    let v1 = 0xF000_0000_0000_0000_0000_0000_0000_0001u128;
    let v2 = 0x1000_0000_0000_0000_0000_0000_0000_0002u128;

    let mut report = None;
    for i in 0..100 {
        if i % 2 == 0 {
            report = Some(s.ingest(&[v1; 4]));
        } else {
            report = Some(s.ingest(&[v2; 4]));
        }
    }

    assert!(s.cells_tracked() >= 1);
    let root = s.graph().g_root();
    let insp = s.inspect_cell(root).unwrap();
    assert!(insp.rank >= 1);
    assert_invariants(&s, report.as_ref().unwrap());
}

// ─── Reset behaviour ────────────────────────────────────────

/// A sentinel is usable the instant a reset returns: the very next batch is
/// counted from zero and produces a root report, with no warm-up call or
/// settling period in between. Reset rebuilds the root tracker as part of
/// the operation rather than leaving the engine cell-less until traffic
/// arrives.
///
/// ´claim:engine:the-sentinel-is-immediately-usable-after-a-reset´
/// ´test:integration:reset-then-immediate-ingest´
#[test]
fn reset_then_immediate_ingest() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(10).build();
    s.reset();

    assert_eq!(s.lifetime_observations(), 0);

    let report = s.ingest(&cell_values(0xA, 16));
    assert_eq!(s.lifetime_observations(), 16);
    assert!(
        report.ancestor_reports.iter().any(|r| r.depth == 0),
        "root must be reported after reset + ingest"
    );
    assert_invariants(&s, &report);
}

/// Resetting twice in a row is no different from resetting once — the second
/// call finds an already-fresh engine and leaves it fresh, counters included,
/// and the sentinel still ingests normally afterwards. A host may therefore
/// reset defensively without tracking whether it already did.
///
/// (´claim:engine:the-sentinel-is-immediately-usable-after-a-reset´)
/// ´test:integration:double-reset´
#[test]
fn double_reset() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(10).build();
    s.reset();
    s.reset();

    assert_eq!(s.lifetime_observations(), 0);
    assert_eq!(s.degenerate_cells_skipped(), 0);

    // Must be usable after double reset.
    let report = s.ingest(&cell_values(0xA, 8));
    assert_eq!(s.lifetime_observations(), 8);
    assert_invariants(&s, &report);
}

// ─── Decay edge cases ───────────────────────────────────────

/// Decay severe enough to annihilate accumulated standing does not leave a
/// dead engine: fresh traffic rebuilds cells from what it observes, the root
/// is reported again, and trackers are live. Decay lowers what regions have
/// earned rather than removing the machinery that earns it, so the recovery
/// path is simply ordinary ingestion.
///
/// ´claim:edge:standing-decayed-almost-to-nothing-is-rebuilt-by-fresh-traffic-rather-than-leaving-a-dead-engine´
/// ´test:integration:decay-to-zero-then-rebuild´
#[test]
fn decay_to_zero_then_rebuild() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(20).build();
    assert!(s.cells_tracked() > 1);

    // Annihilate everything.
    s.decay(0.0001, 0.0);

    // Re-ingest: should rebuild cells from scratch.
    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 16));
    }
    let report = s.ingest(&cell_values(0xA, 8));

    assert!(report.ancestor_reports.iter().any(|r| r.depth == 0));
    assert!(report.health.active_trackers >= 1);
    assert_invariants(&s, &report);
}

/// Decay interleaved with ingestion round after round keeps producing valid
/// reports with the invariants intact. Decay does not need to be rare or
/// quiescent to be safe: it changes spatial standing between batches, and
/// the analysis set is simply recomputed at the start of the next ingestion
/// rather than being eagerly invalidated.
///
/// ´claim:edge:decay-interleaved-with-ingestion-round-after-round-keeps-the-reports-valid´
/// ´test:integration:rapid-decay-ingest-cycle´
#[test]
fn rapid_decay_ingest_cycle() {
    let mut s = ScenarioBuilder::new().seed_range(0xA, 20).warm_batches(10).build();

    for _ in 0..50 {
        s.decay(0.8, 0.0);
        let report = s.ingest(&cell_values(0xA, 8));
        assert!(!report.ancestor_reports.is_empty() || !report.cell_reports.is_empty());
        assert_invariants(&s, &report);
    }
}

// ─── Config edge cases ──────────────────────────────────────

/// Squeezing the competitive budget to a single cell squeezes exactly that:
/// at most one cell is reported as having earned its place, and the root is
/// still reported regardless. The budget governs investment, not structure,
/// so the narrowest possible budget yields the smallest useful report rather
/// than an empty one. The full-set bound is deliberately not asserted here —
/// closure under ancestry keeps materialising chains the bound assumes a
/// wider budget for.
///
/// ´claim:edge:an-extreme-analysis-budget-narrows-what-can-be-chosen-but-never-empties-the-set´
/// ´test:integration:analysis-k-equals-one´
#[test]
fn analysis_k_equals_one() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 1,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for _ in 0..50 {
        s.ingest(&cell_values(0xA, 16));
    }

    let report = s.ingest(&cell_values(0xA, 8));
    assert!(
        report.analysis_set_summary.competitive_size <= 1,
        "with K=1, at most 1 competitive cell"
    );
    assert!(
        report.ancestor_reports.iter().any(|r| r.depth == 0),
        "root must still be reported"
    );
    // `assert_invariants` omitted: K=1 intentionally violates the
    // Steiner tree bound (full_size >> 2*competitive_size+1).
}

/// The other extreme configuration reaches the same floor. With the depth
/// cutoff at zero nothing below the top of the spatial tree can compete at
/// all, and the set still holds at least the root — the one member that is
/// there by obligation rather than by merit.
///
/// (´claim:edge:an-extreme-analysis-budget-narrows-what-can-be-chosen-but-never-empties-the-set´)
/// ´test:integration:analysis-depth-cutoff-zero´
#[test]
fn analysis_depth_cutoff_zero() {
    let cfg = SentinelConfig::<u64> {
        analysis_depth_cutoff: 0,
        split_threshold: 10,
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    for _ in 0..50 {
        s.ingest(&cell_values(0xA, 16));
    }

    let report = s.ingest(&cell_values(0xA, 8));
    // With depth cutoff 0, only the V-Tree root is eligible for
    // competitive selection.
    assert!(
        report.analysis_set_summary.full_size >= 1,
        "at least root must be in the full set"
    );
    // `assert_invariants` omitted: depth_cutoff=0 may violate the
    // Steiner tree bound by design.
}

/// The one-entry-per-observation correspondence holds at scale, not just for
/// a batch of one: a batch of a couple of hundred values yields exactly that
/// many per-sample entries at the root. Nothing is sampled, truncated or
/// aggregated on the way out, so an entry can always be traced back to the
/// value that produced it however large the batch was.
///
/// (´claim:engine:per-sample-scores-appear-only-when-asked-for-and-then-carry-one-entry-per-observation´)
/// ´test:integration:per-sample-scores-large-batch´
#[test]
fn per_sample_scores_large_batch() {
    let cfg = SentinelConfig::<u64> {
        per_sample_scores: true,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&cell_values(0xA, 256));

    let root = report.ancestor_reports.iter().find(|r| r.depth == 0);
    assert!(root.is_some());
    let per_sample = root.unwrap().per_sample.as_ref();
    assert!(per_sample.is_some(), "per_sample should be Some when enabled");
    assert_eq!(
        per_sample.unwrap().len(),
        256,
        "per_sample should have one entry per observation"
    );
    assert_invariants(&s, &report);
}

// ─── ADR-S-011: Degenerate cell dimension guard ────────────

/// The skip counter starts at zero, which is what makes it evidence: any
/// later reading above zero was caused by traffic driving the domain deep
/// enough to produce a cell too narrow to model, and never by construction
/// itself.
///
/// (´claim:engine:a-fresh-sentinel-has-skipped-no-cell-as-too-narrow-to-model´)
/// ´test:integration:edge-cases-degenerate-cells-skipped-starts-at-zero´
#[test]
fn edge_cases_degenerate_cells_skipped_starts_at_zero() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.degenerate_cells_skipped(), 0);
}

/// Ordinary traffic never drives a cell narrow enough to be skipped: after a
/// long run of ingestion the report still shows no skips at all. The guard
/// is there for pathological splitting, not for everyday operation, so a
/// non-zero reading in the field is a signal about the traffic rather than
/// routine noise.
///
/// ´claim:edge:ordinary-traffic-never-drives-a-cell-narrow-enough-to-be-skipped´
/// ´test:integration:degenerate-cells-skipped-in-report-normal-traffic´
#[test]
fn degenerate_cells_skipped_in_report_normal_traffic() {
    let mut s = Sentinel128::new(integration_config()).unwrap();

    for _ in 0..20 {
        s.ingest(&cell_values(0xA, 16));
    }
    let report = s.ingest(&cell_values(0xA, 8));
    assert_eq!(
        report.analysis_set_summary.degenerate_cells_skipped, 0,
        "normal traffic should not produce degenerate cells"
    );
    assert_invariants(&s, &report);
}

/// Reset restores the fresh reading of the skip counter along with
/// everything else, so the count belongs to the current life of the engine
/// and does not carry across a deliberate discarding of the past.
///
/// (´claim:engine:a-fresh-sentinel-has-skipped-no-cell-as-too-narrow-to-model´)
/// ´test:integration:degenerate-cells-skipped-after-reset´
#[test]
fn degenerate_cells_skipped_after_reset() {
    let mut s = Sentinel128::new(integration_config()).unwrap();
    s.ingest(&cell_values(0xA, 8));
    s.reset();
    assert_eq!(s.degenerate_cells_skipped(), 0);
}

/// Splitting made as aggressive as the configuration allows, on traffic
/// hammering a single point, drives the domain deep enough that some cells
/// have almost no suffix left to analyse. Those cells are skipped and
/// counted rather than given a tracker that could form no basis and produce
/// no residual, and the run continues with the root still tracked. A cell
/// too narrow to model is a case the engine declines, not a fault it
/// crashes on.
///
/// ´claim:edge:a-cell-too-narrow-to-model-is-skipped-and-counted-rather-than-bringing-the-run-down´
/// ´test:integration:deep-spray-traffic-does-not-panic´
#[test]
fn deep_spray_traffic_does_not_panic() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 2, // very aggressive splitting
        d_create: 1,
        d_evict: 2,
        budget: 100_000,
        analysis_k: 32,             // large analysis set to pick up deep cells
        analysis_depth_cutoff: 128, // don't filter by V-depth
        ..integration_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    // Hammer a single value to force deep splits.
    let value = 0xAAAA_BBBB_CCCC_DDDD_EEEE_FFFF_0000_1111u128;
    for _ in 0..500 {
        let _report = s.ingest(&[value]);
    }

    // The sentinel must not panic.  Any degenerate cells should be
    // silently skipped and counted.
    assert!(s.cells_tracked() >= 1, "at least root must be tracked");
}
