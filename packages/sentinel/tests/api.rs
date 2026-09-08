// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`new_validates_config`] | engine | A configuration that could not produce a working model is rejected at construction rather than carried into the run: a rank budget of zero leaves the subspace tracker nothing to hold, and the constructor returns an error instead of a sentinel. Validation happens once, so every later method may assume its parameters are coherent. |
//! | [`new_with_default_config_succeeds`] | engine | The configuration the crate ships as its default passes its own validation, so a host that supplies nothing of its own starts from a usable engine rather than from an error. Only validation is exercised here: the default noise schedule warms the root through many rounds, and paying for that construction adds nothing the rest of the suite does not already pay for. |
//! | [`new_starts_with_root_cell`] | engine | A newly constructed sentinel holds exactly one tracker — the root — and has observed nothing. Cells below the root are created only when traffic justifies them, so construction commits to the single model that is structurally obligatory and to no other investment. |
//! | [`empty_ingest_returns_empty_report`] | edge | A batch with no values in it is answered with a report that names no cells and no cross-cell contexts, and the observation counter does not move. The health section is still filled in — the root tracker is alive whether or not anything arrived — so an idle interval reads as an engine with nothing to say rather than as a gap in the record. |
//! | [`single_value_ingest`] | engine | Every value is delivered to the root as well as to whatever cell it routes into, so even a lone observation produces an ancestor report and advances the lifetime count. The root is the one cell guaranteed to contain any coordinate, which is why a batch can never be scored against nothing. |
//! | [`batch_counter_increments`] | engine | The lifetime counter accumulates real observations rather than batches: after a single value and then a batch of several it stands at their sum. Batch boundaries are a delivery convenience for the host and carry no weight in the record of what was actually seen. |
//! | [`repeated_ingest_does_not_panic`] | engine | Feeding the same traffic round after round leaves every structural invariant standing at each step: the reported cells stay within the competitive budget, the two report vectors stay partitioned and ordered, widths still match depths, and no score turns into a non-number. The engine is a steady-state machine, so repetition is the ordinary case and not a stress case. |
//! | [`config_accessor_returns_construction_config`] | engine | The configuration reads back exactly as it was handed in — rank budget, competitive budget and forgetting factor all unchanged. Construction validates the parameters but does not silently normalise or substitute them, so a host can trust the accessor as the authority on how this sentinel is behaving. |
//! | [`graph_accessor_starts_with_single_root`] | engine | The spatial substrate underneath a fresh sentinel is a bare root with no accumulated importance at all. Construction does not seed the graph with anything, so the first real batch is also the first thing the spatial layer has ever ranked — there is no synthetic history for the selector to mistake for traffic. |
//! | [`analysis_set_accessible`] | engine | The set of cells the sentinel is investing in is readable from outside, and after traffic it holds at least the root. The root's membership is unconditional — it is what every ancestor chain terminates at — so the set is never empty and a host inspecting it never has to handle the no-cells case. |
//! | [`cell_gnodes_returns_all_tracked`] | engine | The list of cell handles and the count of tracked cells are two views of one map, never two records that could disagree. A host can enumerate the handles and know it has enumerated everything the engine is modelling. |
//! | [`cells_tracked_matches_cell_gnodes_len`] | engine | cites (´claim:engine:the-listed-cell-handles-and-the-tracked-count-are-two-views-of-one-map´) |
//! | [`lifetime_observations_reflects_real_input`] | engine | cites (´claim:engine:the-lifetime-counter-accumulates-observations-not-batches´) |
//! | [`degenerate_cells_skipped_starts_at_zero`] | engine | A cell whose suffix is too narrow to support a subspace model is skipped and counted rather than modelled, and on a sentinel that has observed nothing that count is zero. The counter is therefore a record of something that happened, not a constant the engine carries around — a non-zero reading always means real traffic drove the domain that deep. |
//! | [`health_accessible_on_fresh_sentinel`] | engine | A health snapshot can be taken before any observation arrives, and it describes the engine truthfully at that moment: one live tracker, nothing observed. Health is a readout of present state rather than a summary accumulated during ingestion, so a host may poll it on a schedule of its own without having to feed the engine first. |
//! | [`inspect_cell_returns_state_for_root`] | engine | A tracked cell can be inspected individually, and what comes back describes that cell: its depth in the routing tree, the width its tracker analyses, and the rank of the subspace it has learned. The root reports depth zero and the full domain width because nothing has been resolved above it, and its rank is at least one from the moment it exists, since a tracker with no direction at all could produce no residual. |
//! | [`inspect_cell_returns_none_for_unknown_gnode`] | engine | A cell handle is meaningful only to the sentinel that issued it. Handed a handle minted by a different sentinel, inspection returns nothing rather than the state of whichever local cell happens to sit at that index — the lookup is a membership question, so a stale or foreign handle is an absence and never a plausible-looking wrong answer. |
//! | [`per_sample_scores_present_when_enabled`] | engine | Scores for individual values are attached to a cell's report only when the host asked for them, and then there is exactly one entry per observation in the batch. Per-sample detail costs memory proportional to the traffic, so it is opt-in rather than always paid for, and the one-to-one correspondence is what makes an entry attributable back to the value that produced it. |
//! | [`per_sample_scores_absent_when_disabled`] | engine | cites (´claim:engine:per-sample-scores-appear-only-when-asked-for-and-then-carry-one-entry-per-observation´) |
//! | [`reset_restores_initial_state`] | engine | Reset returns a used sentinel to the state it was constructed in: the spatial graph is rebuilt as a bare root, the observation counter is zero, and the root tracker alone is tracked. Learned structure is dropped wholesale rather than aged out, because reset exists for the case where the host knows the past no longer describes the future. The configuration is not part of what is cleared. |

//! Contract tests for the sentinel's public operational surface — the
//! handful of methods a host actually calls: construct it, feed it batches,
//! read what it has measured, and reset it.
//!
//! That surface is deliberately narrow. The configuration is validated once,
//! at construction, so a sentinel that exists at all is one whose parameters
//! were coherent; nothing afterwards can put it into a state its config
//! forbade. Everything after construction either observes or reports. The
//! accessors are read-only views onto state the engine already holds — the
//! tracked cells, the spatial substrate, the analysis set, the health
//! snapshot — rather than computations a caller can perturb by asking. What
//! comes back is measurement: counts, scores, distributions, geometry, with
//! no verdict attached, because the sentinel measures and the host decides.
//!
//! Two structures anchor the whole surface. The root tracker is permanent:
//! it exists from construction, receives every observation as an ancestor of
//! whatever cell the value routed to, and is recreated by reset — so there
//! is always something to report against, and the tracked count never falls
//! to zero. And reset is a return to the freshly-constructed state rather
//! than a partial clearing: trackers, spatial graph and counters all go
//! back, while the configuration the sentinel was built with stays.

mod common;

use common::{assert_invariants, cell_values, seeded_sentinel, test_config};
use torrust_sentinel::{Sentinel128, SentinelConfig};

// ═══════════════════════════════════════════════════════════
//  Construction
// ═══════════════════════════════════════════════════════════

/// A configuration that could not produce a working model is rejected at
/// construction rather than carried into the run: a rank budget of zero
/// leaves the subspace tracker nothing to hold, and the constructor returns
/// an error instead of a sentinel. Validation happens once, so every later
/// method may assume its parameters are coherent.
///
/// ´claim:engine:construction-refuses-an-incoherent-configuration-instead-of-running-with-it´
/// ´test:integration:new-validates-config´
#[test]
fn new_validates_config() {
    let bad = SentinelConfig::<u64> {
        max_rank: 0,
        ..test_config()
    };
    assert!(Sentinel128::new(bad).is_err());
}

/// The configuration the crate ships as its default passes its own
/// validation, so a host that supplies nothing of its own starts from a
/// usable engine rather than from an error. Only validation is exercised
/// here: the default noise schedule warms the root through many rounds, and
/// paying for that construction adds nothing the rest of the suite does not
/// already pay for.
///
/// ´claim:engine:the-default-configuration-passes-its-own-validation´
/// ´test:integration:new-with-default-config-succeeds´
#[test]
fn new_with_default_config_succeeds() {
    // Full construction with the default noise schedule (450 root rounds)
    // takes several seconds.  Validate instead — construction is tested
    // end-to-end by every integration test that calls `Sentinel128::new()`.
    let cfg = SentinelConfig::<u64>::default();
    assert!(cfg.validate().is_ok());
}

/// A newly constructed sentinel holds exactly one tracker — the root — and
/// has observed nothing. Cells below the root are created only when traffic
/// justifies them, so construction commits to the single model that is
/// structurally obligatory and to no other investment.
///
/// ´claim:engine:a-fresh-sentinel-holds-the-root-tracker-alone-and-has-observed-nothing´
/// ´test:integration:new-starts-with-root-cell´
#[test]
fn new_starts_with_root_cell() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.cells_tracked(), 1);
    assert_eq!(s.lifetime_observations(), 0);
}

// ═══════════════════════════════════════════════════════════
//  Ingest — basic contract
// ═══════════════════════════════════════════════════════════

/// A batch with no values in it is answered with a report that names no
/// cells and no cross-cell contexts, and the observation counter does not
/// move. The health section is still filled in — the root tracker is alive
/// whether or not anything arrived — so an idle interval reads as an engine
/// with nothing to say rather than as a gap in the record.
///
/// ´claim:edge:an-empty-batch-yields-a-report-with-no-cells-and-moves-no-counter´
/// ´test:integration:empty-ingest-returns-empty-report´
#[test]
fn empty_ingest_returns_empty_report() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[]);

    assert!(report.cell_reports.is_empty());
    assert!(report.coordination_reports.is_empty());
    assert_eq!(report.health.lifetime_observations, 0);
    assert_eq!(report.health.active_trackers, 1); // root cell
}

/// Every value is delivered to the root as well as to whatever cell it
/// routes into, so even a lone observation produces an ancestor report and
/// advances the lifetime count. The root is the one cell guaranteed to
/// contain any coordinate, which is why a batch can never be scored against
/// nothing.
///
/// ´claim:engine:the-root-tracker-receives-every-observation-in-every-batch´
/// ´test:integration:single-value-ingest´
#[test]
fn single_value_ingest() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let report = s.ingest(&[0xABCD_0000_0000_0000_0000_0000_0000_0001]);

    // Root cell always receives all observations (as an ancestor).
    assert!(!report.ancestor_reports.is_empty());
    assert_eq!(report.health.lifetime_observations, 1);
    assert_invariants(&s, &report);
}

/// The lifetime counter accumulates real observations rather than batches:
/// after a single value and then a batch of several it stands at their sum.
/// Batch boundaries are a delivery convenience for the host and carry no
/// weight in the record of what was actually seen.
///
/// ´claim:engine:the-lifetime-counter-accumulates-observations-not-batches´
/// ´test:integration:batch-counter-increments´
#[test]
fn batch_counter_increments() {
    let mut s = Sentinel128::new(test_config()).unwrap();

    s.ingest(&[1]);
    assert_eq!(s.lifetime_observations(), 1);

    s.ingest(&[2, 3, 4]);
    assert_eq!(s.lifetime_observations(), 4);
}

/// Feeding the same traffic round after round leaves every structural
/// invariant standing at each step: the reported cells stay within the
/// competitive budget, the two report vectors stay partitioned and ordered,
/// widths still match depths, and no score turns into a non-number. The
/// engine is a steady-state machine, so repetition is the ordinary case and
/// not a stress case.
///
/// ´claim:engine:repeated-ingestion-of-the-same-traffic-keeps-every-structural-invariant´
/// ´test:integration:repeated-ingest-does-not-panic´
#[test]
fn repeated_ingest_does_not_panic() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    let values = cell_values(0xA, 20);

    for _ in 0..10 {
        let report = s.ingest(&values);
        assert_invariants(&s, &report);
    }
}

// ═══════════════════════════════════════════════════════════
//  Accessors — read-only queries
// ═══════════════════════════════════════════════════════════

/// The configuration reads back exactly as it was handed in — rank budget,
/// competitive budget and forgetting factor all unchanged. Construction
/// validates the parameters but does not silently normalise or substitute
/// them, so a host can trust the accessor as the authority on how this
/// sentinel is behaving.
///
/// ´claim:engine:the-configuration-reads-back-as-given-because-construction-never-rewrites-it´
/// ´test:integration:config-accessor-returns-construction-config´
#[test]
fn config_accessor_returns_construction_config() {
    let cfg = test_config();
    let s = Sentinel128::new(cfg.clone()).unwrap();

    assert_eq!(s.config().max_rank, cfg.max_rank);
    assert_eq!(s.config().analysis_k, cfg.analysis_k);
    assert!((s.config().forgetting_factor - cfg.forgetting_factor).abs() < f64::EPSILON);
}

/// The spatial substrate underneath a fresh sentinel is a bare root with no
/// accumulated importance at all. Construction does not seed the graph with
/// anything, so the first real batch is also the first thing the spatial
/// layer has ever ranked — there is no synthetic history for the selector to
/// mistake for traffic.
///
/// ´claim:engine:construction-leaves-the-spatial-substrate-a-bare-root-with-no-accumulated-importance´
/// ´test:integration:graph-accessor-starts-with-single-root´
#[test]
fn graph_accessor_starts_with_single_root() {
    let s = Sentinel128::new(test_config()).unwrap();

    assert_eq!(s.graph().node_count(), 1);
    assert_eq!(s.graph().terminal_count(), 1);
    assert_eq!(s.graph().total_sum(), 0u64);
}

/// The set of cells the sentinel is investing in is readable from outside,
/// and after traffic it holds at least the root. The root's membership is
/// unconditional — it is what every ancestor chain terminates at — so the
/// set is never empty and a host inspecting it never has to handle the
/// no-cells case.
///
/// ´claim:engine:the-analysis-set-is-readable-and-always-holds-at-least-the-root´
/// ´test:integration:analysis-set-accessible´
#[test]
fn analysis_set_accessible() {
    let s = seeded_sentinel();

    let aset = s.analysis_set();
    // The full set always includes the root.
    assert!(aset.total_count() >= 1);
}

/// The list of cell handles and the count of tracked cells are two views of
/// one map, never two records that could disagree. A host can enumerate the
/// handles and know it has enumerated everything the engine is modelling.
///
/// ´claim:engine:the-listed-cell-handles-and-the-tracked-count-are-two-views-of-one-map´
/// ´test:integration:cell-gnodes-returns-all-tracked´
#[test]
fn cell_gnodes_returns_all_tracked() {
    let s = seeded_sentinel();

    let gnodes = s.cell_gnodes();
    assert_eq!(gnodes.len(), s.cells_tracked());
}

/// The same agreement holds on both sides of the event that could break it.
/// Traffic heavy enough to split the domain adds cells to the map, and the
/// count and the handle list move together through that churn rather than
/// one of them being refreshed a moment later than the other.
///
/// (´claim:engine:the-listed-cell-handles-and-the-tracked-count-are-two-views-of-one-map´)
/// ´test:integration:cells-tracked-matches-cell-gnodes-len´
#[test]
fn cells_tracked_matches_cell_gnodes_len() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.cells_tracked(), s.cell_gnodes().len());

    // After traffic that may create cells.
    for _ in 0..5 {
        s.ingest(&cell_values(0xF, 8));
    }
    assert_eq!(s.cells_tracked(), s.cell_gnodes().len());
}

/// Pinning the other end of the same statement: the counter starts at zero
/// and each batch adds exactly as many observations as it carried, so its
/// value is the running total of real input and not of anything the engine
/// generated for itself while warming a tracker.
///
/// (´claim:engine:the-lifetime-counter-accumulates-observations-not-batches´)
/// ´test:integration:lifetime-observations-reflects-real-input´
#[test]
fn lifetime_observations_reflects_real_input() {
    let mut s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.lifetime_observations(), 0);

    let batch_a = cell_values(0xA, 5);
    s.ingest(&batch_a);
    assert_eq!(s.lifetime_observations(), 5);

    let batch_b = cell_values(0xB, 3);
    s.ingest(&batch_b);
    assert_eq!(s.lifetime_observations(), 8);
}

/// A cell whose suffix is too narrow to support a subspace model is skipped
/// and counted rather than modelled, and on a sentinel that has observed
/// nothing that count is zero. The counter is therefore a record of
/// something that happened, not a constant the engine carries around — a
/// non-zero reading always means real traffic drove the domain that deep.
///
/// ´claim:engine:a-fresh-sentinel-has-skipped-no-cell-as-too-narrow-to-model´
/// ´test:integration:degenerate-cells-skipped-starts-at-zero´
#[test]
fn degenerate_cells_skipped_starts_at_zero() {
    let s = Sentinel128::new(test_config()).unwrap();
    assert_eq!(s.degenerate_cells_skipped(), 0);
}

/// A health snapshot can be taken before any observation arrives, and it
/// describes the engine truthfully at that moment: one live tracker, nothing
/// observed. Health is a readout of present state rather than a summary
/// accumulated during ingestion, so a host may poll it on a schedule of its
/// own without having to feed the engine first.
///
/// ´claim:engine:a-health-snapshot-is-available-before-any-observation-arrives´
/// ´test:integration:health-accessible-on-fresh-sentinel´
#[test]
fn health_accessible_on_fresh_sentinel() {
    let s = Sentinel128::new(test_config()).unwrap();
    let h = s.health();

    assert_eq!(h.active_trackers, 1);
    assert_eq!(h.lifetime_observations, 0);
}

/// A tracked cell can be inspected individually, and what comes back
/// describes that cell: its depth in the routing tree, the width its tracker
/// analyses, and the rank of the subspace it has learned. The root reports
/// depth zero and the full domain width because nothing has been resolved
/// above it, and its rank is at least one from the moment it exists, since a
/// tracker with no direction at all could produce no residual.
///
/// ´claim:engine:a-tracked-cell-is-inspectable-and-carries-its-own-depth-width-and-rank´
/// ´test:integration:inspect-cell-returns-state-for-root´
#[test]
fn inspect_cell_returns_state_for_root() {
    let s = Sentinel128::new(test_config()).unwrap();
    let root = s.graph().g_root();

    let inspection = s.inspect_cell(root).expect("root cell should exist");
    assert_eq!(inspection.depth, 0);
    assert_eq!(inspection.analysis_width, 128);
    assert!(inspection.rank >= 1);
}

/// A cell handle is meaningful only to the sentinel that issued it. Handed a
/// handle minted by a different sentinel, inspection returns nothing rather
/// than the state of whichever local cell happens to sit at that index — the
/// lookup is a membership question, so a stale or foreign handle is an
/// absence and never a plausible-looking wrong answer.
///
/// ´claim:engine:a-handle-from-another-sentinel-inspects-to-nothing-rather-than-to-a-wrong-cell´
/// ´test:integration:inspect-cell-returns-none-for-unknown-gnode´
#[test]
fn inspect_cell_returns_none_for_unknown_gnode() {
    let s = Sentinel128::new(test_config()).unwrap();
    // Build a second sentinel so its non-root GNodeIds are foreign.
    let mut other = Sentinel128::new(test_config()).unwrap();
    other.ingest(&[0xF000_0000_0000_0000_0000_0000_0000_0001]);

    let other_gnodes = other.cell_gnodes();
    if other_gnodes.len() > 1 {
        let non_root = other_gnodes.iter().find(|&&g| g != other.graph().g_root()).unwrap();
        assert!(s.inspect_cell(*non_root).is_none());
    }
}

// ═══════════════════════════════════════════════════════════
//  Per-sample scores
// ═══════════════════════════════════════════════════════════

/// Scores for individual values are attached to a cell's report only when
/// the host asked for them, and then there is exactly one entry per
/// observation in the batch. Per-sample detail costs memory proportional to
/// the traffic, so it is opt-in rather than always paid for, and the
/// one-to-one correspondence is what makes an entry attributable back to the
/// value that produced it.
///
/// ´claim:engine:per-sample-scores-appear-only-when-asked-for-and-then-carry-one-entry-per-observation´
/// ´test:integration:per-sample-scores-present-when-enabled´
#[test]
fn per_sample_scores_present_when_enabled() {
    let mut s = Sentinel128::new(test_config()).unwrap(); // per_sample_scores = true

    let report = s.ingest(&[0xF000_0000_0000_0000_0000_0000_0000_0001]);

    let root_report = report.ancestor_reports.iter().find(|cr| cr.depth == 0).unwrap();
    assert!(root_report.per_sample.is_some());
    assert_eq!(root_report.per_sample.as_ref().unwrap().len(), 1);
}

/// The other end of the same switch: with the option off the field is
/// absent, not an empty list. A host that did not ask cannot mistake missing
/// detail for a batch in which nothing scored, and the engine does not
/// allocate for detail nobody wanted.
///
/// (´claim:engine:per-sample-scores-appear-only-when-asked-for-and-then-carry-one-entry-per-observation´)
/// ´test:integration:per-sample-scores-absent-when-disabled´
#[test]
fn per_sample_scores_absent_when_disabled() {
    let cfg = SentinelConfig::<u64> {
        per_sample_scores: false,
        ..test_config()
    };
    let mut s = Sentinel128::new(cfg).unwrap();

    let report = s.ingest(&[0xF000_0000_0000_0000_0000_0000_0000_0001]);
    let root_report = report.ancestor_reports.iter().find(|cr| cr.depth == 0).unwrap();
    assert!(root_report.per_sample.is_none());
}

// ═══════════════════════════════════════════════════════════
//  Reset
// ═══════════════════════════════════════════════════════════

/// Reset returns a used sentinel to the state it was constructed in: the
/// spatial graph is rebuilt as a bare root, the observation counter is zero,
/// and the root tracker alone is tracked. Learned structure is dropped
/// wholesale rather than aged out, because reset exists for the case where
/// the host knows the past no longer describes the future. The configuration
/// is not part of what is cleared.
///
/// ´claim:engine:reset-returns-the-sentinel-to-its-freshly-constructed-state-and-keeps-the-configuration´
/// ´test:integration:reset-restores-initial-state´
#[test]
fn reset_restores_initial_state() {
    let mut s = seeded_sentinel();

    s.reset();

    assert_eq!(s.graph().node_count(), 1);
    assert_eq!(s.graph().total_sum(), 0u64);
    assert_eq!(s.lifetime_observations(), 0);
    assert_eq!(s.cells_tracked(), 1);
}
