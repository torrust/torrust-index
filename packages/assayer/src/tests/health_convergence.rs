// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for convergence tracking.
//!
//! The Assayer watches its own models converge. Four trackers contribute: the
//! concordance tracker recalibrates its per-axis thresholds from a bounded
//! window of recent observations; the Platt tracker counts refits and decides
//! when calibration has settled; the identity tracker measures how much the
//! competitive cell set churns; and the composite stage combines the gating
//! ones conservatively so the system never claims to be further along than its
//! least converged part.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`concordance_initial_thresholds`] | wellness | A concordance tracker that has seen nothing holds neutral placeholder thresholds on every axis rather than zero or infinity. Thresholds are learned from observation, so before any have arrived the tracker offers a value that neither passes everything nor rejects everything. |
//! | [`concordance_1001_observations_one_recalibration`] | wellness | Recalibration is paced by completed intervals of observation, not by every arrival: a thousand-and-one observations against an interval of a thousand leaves exactly one recalibration behind. Thresholds are therefore a periodic decision a host can reason about, rather than something that shifts under it on each sample. |
//! | [`concordance_thresholds_readable_during_observe`] | wellness | Reading the current thresholds is always available and never waits on the observation path: a read before any observation and a read straight after one both return immediately, and both return the placeholders because no interval has completed. The scoring hot path can consult thresholds without ever contending with the tracker that maintains them. |
//! | [`concordance_80th_percentile`] | wellness | A recalibrated threshold is the configured percentile of what the window actually holds: fed a spread evenly covering nought to three, the eightieth-percentile setting lands near two point four on every axis. The threshold adapts to the observed scale instead of being an absolute number that means something different in each deployment. |
//! | [`concordance_window_wraps`] | wellness | The observation window keeps only the most recent samples: once a windowful of high scores has displaced an earlier windowful of low ones, the recalibrated thresholds reflect the high regime alone. Superseded history stops voting, so a tracker that has lived through a regime change calibrates to the regime it is in rather than to an average of both. |
//! | [`platt_should_refit_at_n`] | wellness | A refit becomes due only when the configured number of labels has actually accumulated, and on exactly that label rather than one early: ninety-nine labels against a budget of a hundred leaves the tracker still waiting, and the hundredth makes it ready. Refitting is expensive, so the trigger is a counted quantity of new evidence, not the mere passage of work. |
//! | [`platt_early_refit_immediate`] | wellness | Marking an early refit overrides the label budget outright: a tracker with no labels at all and a budget of a thousand reports a refit due the moment the mark is set. Something outside the counter — a detected shift, an operator's request — can force calibration to be redone without waiting out an interval that has become meaningless. |
//! | [`platt_record_refit_clears_pending`] | wellness | Recording a completed refit settles every reason the refit was owed: the early-refit mark is cleared, the labels-since-refit count returns to zero and the completed-refit tally advances. Without all three moving together a satisfied request would keep firing, and calibration would refit on every subsequent opportunity. |
//! | [`platt_convergence_state_transitions`] | wellness | Calibration convergence is read off two quantities at once — how many refits have been completed and how much the last one moved the calibration. No refits reads as Initial, one as FirstFit, and further refits as Converging for as long as the calibration keeps shifting; only when the refit count is met *and* the shift falls under the tolerance does it read Converged. A run of large corrections cannot be counted into convergence, and a single small correction cannot short-cut to it. |
//! | [`identity_entries_exits_change_rate`] | wellness | Churn in the competitive cell set is recorded from both directions and kept apart: five cells arriving lift the smoothed change rate off its floor and land in the entry tally, and two later departures land in the exit tally while moving the same smoothed rate. Keeping entries and exits separate means a set that is growing and a set that is shedding are distinguishable even when their churn rates match. |
//! | [`identity_no_changes_jaccard_stays_high`] | wellness | An identity set observed twice without change drives its smoothed overlap measure upwards rather than letting it decay. Stability has to be rewarded by the same statistic that punishes churn, or a settled set would look indistinguishable from one that had simply stopped being observed. |
//! | [`identity_stages_are_diagnostic`] | wellness | The identity stage is a reading derived from the tracker's own statistics, not an input to them: it begins at Initial and, once change events have been recorded, reports whichever of the settled stages the metrics warrant, while the tracker keeps recording exactly as before. Nothing downstream of the tracker changes behaviour because the label moved. |
//! | [`composite_0_labels_prior_only`] | wellness | With no labels seen and no calibration fitted, the composite stage is ColdStart. Whatever the models are emitting at that point rests on priors alone, and the stage says so rather than letting an unlabelled system present itself as merely early. |
//! | [`composite_30_labels_anchor`] | wellness | Labels accumulating without a calibration fit behind them carry the composite off ColdStart but no further than AnchorEmerging. Evidence is arriving and the anchor is forming, yet nothing has been calibrated against it — so the stage reflects data gathered rather than a model that has been fitted to it. |
//! | [`composite_first_platt_sister`] | wellness | The first completed calibration fit is what advances the composite to PreCalibration — a single refit on a tracker carrying labels is enough. One fit means the mapping from score to probability now exists, which is a different condition from having merely collected the labels to fit it with. |
//! | [`composite_conservative_gating`] | wellness | The composite takes the least advanced of its gating inputs, never the most advanced: fully converged calibration paired with a volatile identity set reads as InteractionMaturing, and only when the same calibration is paired with a stable, mature identity set does it reach SteadyState. A system is exactly as converged as its worst-converged part, so no single settled component can talk the whole assembly into claiming readiness. |
//! | [`event_stage_changed_emitted`] | wellness | A composite stage that has moved is announced, and the announcement carries both endpoints — the stage left behind and the stage arrived at. A subscriber that missed earlier events can therefore tell whether the transition it is reading follows on from the last one it saw. |
//! | [`event_no_emit_when_unchanged`] | wellness | A stage that has not moved produces nothing at all: checked against itself, the transition path emits no event and the channel stays empty. The check runs on every evaluation, so an event per evaluation would drown the channel in restatements and evict the transitions that actually mattered. |
//! | [`event_channel_overflow`] | wellness | When the event channel is full the emitting side neither blocks nor panics: the overflowing event is discarded and a drop counter advances, while the events already queued survive intact and remain receivable. Model health reporting is a bystander to the work it observes, so a slow or absent subscriber must cost a countable loss of visibility rather than stalling the system being watched. |
//! | [`concordance_nan_filtered_from_window`] | wellness | Non-finite observations are excluded when thresholds are recomputed: a window seeded with NaN and both infinities among ordinary values still recalibrates to finite, positive thresholds. A percentile taken over unordered values would otherwise poison every subsequent comparison, and a single upstream division by zero would silently disable the axis. |
//! | [`concordance_threshold_floor_applied`] | wellness | A floor holds the thresholds away from zero even when the window is degenerate: a hundred observations all sitting at a thousandth would put the percentile essentially at nothing, and the calibrated threshold still comes out at the floor. A threshold at zero would treat every subsequent score as exceptional, so a quiet stretch cannot be allowed to calibrate the axis into permanent alarm. |
//! | [`identity_maturing_blocks_composite_fully_converged`] | wellness | Stable-looking metrics are not by themselves maturity: an identity set with a low change rate, high overlap and a healthy entry count, but first seen only moments ago, still reads as Maturing and holds the composite at InteractionMaturing despite fully converged calibration. A set that has barely existed has had no opportunity to churn, so its calm is absence of evidence rather than evidence of stability, and elapsed existence is the gate that distinguishes the two. |
//! | [`platt_tracker_serde`] | wellness | A calibration tracker survives being written out and read back with everything that governs its next decision intact: the completed-refit tally, the last fit's calibration shift and both slope terms, and the pending early-refit mark. A restarted Assayer resumes the convergence judgement it had reached instead of re-earning it from Initial. |
//! | [`identity_tracker_serde`] | wellness | An identity tracker likewise round-trips with both its raw tallies — entries, exits, current occupancy — and its smoothed churn and overlap measures. The smoothed measures take many change events to build, so losing them across a restart would reset the identity set to Volatile and gate the composite back down for no reason connected to the set itself. |
//! | [`concordance_observe_below_interval`] | wellness | cites (´claim:wellness:recalibration-runs-once-per-completed-interval-of-observations´) |
//! | [`concordance_second_recalibration`] | wellness | cites (´claim:wellness:recalibration-runs-once-per-completed-interval-of-observations´) |
//! | [`concordance_recalibration_during_read`] | wellness | A reader running concurrently with a writer that keeps crossing recalibration boundaries always sees a whole, finite, non-negative threshold set, and completes every one of its reads without being starved. Thresholds are swapped as a unit rather than mutated in place, so a scoring thread can never catch a set half-updated — part old regime, part new. |
//! | [`concordance_health`] | wellness | The health snapshot reports what an operator needs to judge whether the thresholds can be trusted: how full the window is against its capacity, how many recalibrations have run, how many observations have been seen in total, and the thresholds currently in force. Observations beyond capacity raise the total and the calibration count while the window stays pinned at capacity — the snapshot distinguishes how much has been seen from how much is still being weighed. |
//! | [`concordance_checkpoint_state_round_trip`] | wellness | A checkpoint carries the live window, the thresholds in force and the counters, and a tracker rebuilt from it reports all of them unchanged. The captured window is bounded by capacity rather than by everything ever observed, so the checkpoint stays a fixed size however long the tracker has run — and a restarted Assayer resumes with calibrated thresholds instead of falling back to placeholders while it re-fills a window. |
//! | [`composite_concordance_not_gating`] | wellness | Concordance is measured but never gates: with calibration converged and the identity set stable and mature, the composite reaches SteadyState even though no concordance observation has ever been made. Concordance describes agreement between axes rather than whether a model has been learned, so holding the whole system back on it would leave the composite hostage to a diagnostic. |
//! | [`composite_challenge_not_gating`] | wellness | Challenge sufficiency is likewise outside the gate: with calibration converged and the identity set stable and mature, the composite reads SteadyState whether challenges are reported sufficient or not. The flag makes no difference in either direction, so a deployment that never generates challenges is not thereby stuck short of steady state. |
//! | [`event_batch_init_complete`] | wellness | An emitted health event arrives at the receiver as the same variant that was sent — here a payload-free milestone marking batch initialisation finished. The channel is a transport and not a translator: a subscriber matching on the variant it cares about gets exactly what the model reported. |
//! | [`event_bootstrap_complete`] | wellness | cites (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´) |
//! | [`event_platt_first_fit`] | wellness | cites (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´) |
//! | [`event_platt_converged`] | wellness | cites (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´) |
//! | [`event_identity_stabilised`] | wellness | cites (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´) |
//! | [`event_drain`] | wellness | Events are received in the order they were emitted: a run of stage transitions climbing from ColdStart to SteadyState drains as that same chain, each event's starting stage matching the previous one's arrival. Convergence is a narrative, and a reordered drain would let a subscriber reconstruct a history the system never lived through. |

use std::sync::atomic::{AtomicU64, Ordering};

use crate::health::{
    CompositeConvergenceStage, ConcordanceConfig, ConcordanceTracker, ConvergenceEvent, DEFAULT_PLATT_CONVERGED_DELTA_CAL,
    DEFAULT_PLATT_CONVERGED_REFIT_COUNT, HealthEvent, IdentityConvergenceStage, IdentityConvergenceTracker,
    PlattConvergenceState, PlattConvergenceTracker, PlattRefitResult, check_convergence_event, compute_composite_stage,
    create_event_channel,
};
use crate::identity::CompetitiveCellId;
use crate::testing::{DEFAULT_TOLERANCES, assert_finite, assert_near};

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance Tracker Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A concordance tracker that has seen nothing holds neutral placeholder
/// thresholds on every axis rather than zero or infinity. Thresholds are
/// learned from observation, so before any have arrived the tracker offers a
/// value that neither passes everything nor rejects everything.
///
/// ´claim:wellness:a-fresh-concordance-tracker-holds-neutral-placeholder-thresholds´
/// ´test:crate:concordance-initial-thresholds´
#[test]
fn concordance_initial_thresholds() {
    let tracker = ConcordanceTracker::with_defaults();
    let thresholds = tracker.current_thresholds();
    assert_eq!(thresholds, [0.5; 4]);
}

/// Recalibration is paced by completed intervals of observation, not by every
/// arrival: a thousand-and-one observations against an interval of a thousand
/// leaves exactly one recalibration behind. Thresholds are therefore a
/// periodic decision a host can reason about, rather than something that
/// shifts under it on each sample.
///
/// ´claim:wellness:recalibration-runs-once-per-completed-interval-of-observations´
/// ´test:crate:concordance-1001-observations-one-recalibration´
#[test]
fn concordance_1001_observations_one_recalibration() {
    let config = ConcordanceConfig {
        window_capacity: 5_000,
        recalibration_interval: 1_000,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Observe 1,001 times
    for i in 0..1_001 {
        let z = [i as f64 % 3.0; 4];
        tracker.observe(z);
    }

    let health = tracker.health();
    assert_eq!(health.calibrations_completed, 1);
}

/// Reading the current thresholds is always available and never waits on the
/// observation path: a read before any observation and a read straight after
/// one both return immediately, and both return the placeholders because no
/// interval has completed. The scoring hot path can consult thresholds
/// without ever contending with the tracker that maintains them.
///
/// ´claim:wellness:thresholds-can-be-read-at-any-moment-without-waiting-on-the-observation-path´
/// ´test:crate:concordance-thresholds-readable-during-observe´
#[test]
fn concordance_thresholds_readable_during_observe() {
    let tracker = ConcordanceTracker::with_defaults();

    // Read before any observations
    let t1 = tracker.current_thresholds();
    assert_eq!(t1, [0.5; 4]);

    // Observe some values
    tracker.observe([1.0, 2.0, 3.0, 4.0]);

    // Read is still available (no blocking)
    let t2 = tracker.current_thresholds();
    assert_eq!(t2, [0.5; 4]); // Not recalibrated yet
}

/// A recalibrated threshold is the configured percentile of what the window
/// actually holds: fed a spread evenly covering nought to three, the
/// eightieth-percentile setting lands near two point four on every axis. The
/// threshold adapts to the observed scale instead of being an absolute number
/// that means something different in each deployment.
///
/// ´claim:wellness:a-recalibrated-threshold-is-the-configured-percentile-of-the-observed-window´
/// ´test:crate:concordance-80th-percentile´
#[test]
fn concordance_80th_percentile() {
    let config = ConcordanceConfig {
        window_capacity: 1_000,
        recalibration_interval: 100,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Observe uniform distribution [0, 3]
    for i in 0..100 {
        let z = [i as f64 * 3.0 / 99.0; 4]; // 0.0 to 3.0
        tracker.observe(z);
    }

    // 80th percentile of [0, 3] uniform ≈ 2.4
    let thresholds = tracker.current_thresholds();
    for &t in &thresholds {
        assert_near(t, 2.4, 0.1, "80th percentile threshold");
    }
}

/// The observation window keeps only the most recent samples: once a windowful
/// of high scores has displaced an earlier windowful of low ones, the
/// recalibrated thresholds reflect the high regime alone. Superseded history
/// stops voting, so a tracker that has lived through a regime change
/// calibrates to the regime it is in rather than to an average of both.
///
/// ´claim:wellness:the-window-keeps-only-recent-observations-so-superseded-history-stops-voting´
/// ´test:crate:concordance-window-wraps´
#[test]
fn concordance_window_wraps() {
    let config = ConcordanceConfig {
        window_capacity: 10,
        recalibration_interval: 5,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Fill window with low values
    for _ in 0..10 {
        tracker.observe([0.0; 4]);
    }

    // Fill with high values (should evict low values)
    for _ in 0..10 {
        tracker.observe([10.0; 4]);
    }

    // After recalibration, thresholds should be high (only high values remain)
    let thresholds = tracker.current_thresholds();
    assert_finite(&thresholds, "thresholds after wraparound");
    for &t in &thresholds {
        assert_near(t, 10.0, DEFAULT_TOLERANCES.bit_identical, "threshold after wraparound");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platt Tracker Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A refit becomes due only when the configured number of labels has actually
/// accumulated, and on exactly that label rather than one early: ninety-nine
/// labels against a budget of a hundred leaves the tracker still waiting, and
/// the hundredth makes it ready. Refitting is expensive, so the trigger is a
/// counted quantity of new evidence, not the mere passage of work.
///
/// ´claim:wellness:a-refit-falls-due-only-once-the-configured-label-count-has-accumulated´
/// ´test:crate:platt-should-refit-at-n´
#[test]
fn platt_should_refit_at_n() {
    let mut tracker = PlattConvergenceTracker::new();
    let n_refit = 100;

    // Not ready initially
    assert!(!tracker.should_refit(n_refit));

    // Record labels
    for _ in 0..99 {
        tracker.record_label();
    }
    assert!(!tracker.should_refit(n_refit));

    tracker.record_label();
    assert!(tracker.should_refit(n_refit));
}

/// Marking an early refit overrides the label budget outright: a tracker with
/// no labels at all and a budget of a thousand reports a refit due the moment
/// the mark is set. Something outside the counter — a detected shift, an
/// operator's request — can force calibration to be redone without waiting out
/// an interval that has become meaningless.
///
/// ´claim:wellness:an-early-refit-mark-makes-a-refit-due-regardless-of-the-label-budget´
/// ´test:crate:platt-early-refit-immediate´
#[test]
fn platt_early_refit_immediate() {
    let mut tracker = PlattConvergenceTracker::new();

    assert!(!tracker.should_refit(1000));

    tracker.mark_early_refit();
    assert!(tracker.should_refit(1000));
}

/// Recording a completed refit settles every reason the refit was owed: the
/// early-refit mark is cleared, the labels-since-refit count returns to zero
/// and the completed-refit tally advances. Without all three moving together a
/// satisfied request would keep firing, and calibration would refit on every
/// subsequent opportunity.
///
/// ´claim:wellness:recording-a-refit-clears-the-early-mark-restarts-the-label-count-and-advances-the-tally´
/// ´test:crate:platt-record-refit-clears-pending´
#[test]
fn platt_record_refit_clears_pending() {
    let mut tracker = PlattConvergenceTracker::new();

    tracker.mark_early_refit();
    assert!(tracker.early_refit_pending);

    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.1,
            kappa_sister: 1.0,
            kappa_anchor: 1.0,
            ..Default::default()
        },
        100,
    );

    assert!(!tracker.early_refit_pending);
    assert_eq!(tracker.labels_since_refit, 0);
    assert_eq!(tracker.refits_completed, 1);
}

/// Calibration convergence is read off two quantities at once — how many
/// refits have been completed and how much the last one moved the calibration.
/// No refits reads as Initial, one as FirstFit, and further refits as
/// Converging for as long as the calibration keeps shifting; only when the
/// refit count is met *and* the shift falls under the tolerance does it read
/// Converged. A run of large corrections cannot be counted into convergence,
/// and a single small correction cannot short-cut to it.
///
/// ´claim:wellness:calibration-counts-as-converged-only-when-enough-refits-have-run-and-the-last-one-barely-moved-it´
/// ´test:crate:platt-convergence-state-transitions´
#[test]
fn platt_convergence_state_transitions() {
    let mut tracker = PlattConvergenceTracker::new();

    // Initial: no refits
    assert_eq!(
        tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
        PlattConvergenceState::Initial
    );

    // FirstFit: 1 refit
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.5,
            ..Default::default()
        },
        0,
    );
    assert_eq!(
        tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
        PlattConvergenceState::FirstFit
    );

    // Converging: 2+ refits but high delta_cal
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.5,
            ..Default::default()
        },
        0,
    );
    assert_eq!(
        tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
        PlattConvergenceState::Converging
    );

    // Still converging at 4 refits
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.5,
            ..Default::default()
        },
        0,
    );
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.5,
            ..Default::default()
        },
        0,
    );
    assert_eq!(
        tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
        PlattConvergenceState::Converging
    );

    // Converged: 5+ refits and low delta_cal
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.005,
            ..Default::default()
        },
        0,
    );
    assert_eq!(
        tracker.convergence_state(DEFAULT_PLATT_CONVERGED_DELTA_CAL, DEFAULT_PLATT_CONVERGED_REFIT_COUNT,),
        PlattConvergenceState::Converged
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Tracker Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn make_cell(lo: u128, depth: u8) -> CompetitiveCellId {
    CompetitiveCellId::new(lo, depth)
}

/// Churn in the competitive cell set is recorded from both directions and kept
/// apart: five cells arriving lift the smoothed change rate off its floor and
/// land in the entry tally, and two later departures land in the exit tally
/// while moving the same smoothed rate. Keeping entries and exits separate
/// means a set that is growing and a set that is shedding are distinguishable
/// even when their churn rates match.
///
/// ´claim:wellness:entries-and-exits-are-tallied-separately-and-both-move-the-smoothed-change-rate´
/// ´test:crate:identity-entries-exits-change-rate´
#[test]
fn identity_entries_exits_change_rate() {
    let mut tracker = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    let initial_change_rate = tracker.change_rate_ewma;

    // Initial state: no previous cells
    let entries = vec![
        make_cell(0, 4),
        make_cell(100, 4),
        make_cell(200, 4),
        make_cell(300, 4),
        make_cell(400, 4),
    ];
    let exits: Vec<CompetitiveCellId> = vec![];
    let current = entries.clone();

    tracker.record_change_event(&entries, &exits, &current);

    // Change rate should increase (5 entries from empty)
    assert!(tracker.change_rate_ewma > initial_change_rate);
    assert_eq!(tracker.total_entries, 5);
    assert_eq!(tracker.total_exits, 0);

    // Now simulate exits
    let entries2 = vec![];
    let exits2 = vec![make_cell(0, 4), make_cell(100, 4)];
    let current2 = vec![make_cell(200, 4), make_cell(300, 4), make_cell(400, 4)];

    let change_rate_before = tracker.change_rate_ewma;
    tracker.record_change_event(&entries2, &exits2, &current2);

    assert_eq!(tracker.total_exits, 2);
    // Change rate EWMA updates (exact value depends on alpha)
    assert!(tracker.change_rate_ewma != change_rate_before || change_rate_before == 0.0);
}

/// An identity set observed twice without change drives its smoothed overlap
/// measure upwards rather than letting it decay. Stability has to be
/// rewarded by the same statistic that punishes churn, or a settled set would
/// look indistinguishable from one that had simply stopped being observed.
///
/// ´claim:wellness:an-unchanged-identity-set-drives-the-smoothed-overlap-upwards´
/// ´test:crate:identity-no-changes-jaccard-stays-high´
#[test]
fn identity_no_changes_jaccard_stays_high() {
    let mut tracker = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));

    let cells = vec![make_cell(0, 4), make_cell(100, 4), make_cell(200, 4)];

    // First observation sets previous_cell_ids
    tracker.record_change_event(&cells, &[], &cells);

    // No changes: same set
    let jaccard_before = tracker.jaccard_ewma;
    tracker.record_change_event(&[], &[], &cells);

    // Jaccard should stay high (approaching 1.0)
    assert!(tracker.jaccard_ewma >= jaccard_before);
}

/// The identity stage is a reading derived from the tracker's own statistics,
/// not an input to them: it begins at Initial and, once change events have
/// been recorded, reports whichever of the settled stages the metrics warrant,
/// while the tracker keeps recording exactly as before. Nothing downstream of
/// the tracker changes behaviour because the label moved.
///
/// ´claim:wellness:the-identity-stage-is-derived-from-the-metrics-and-never-alters-what-the-tracker-records´
/// ´test:crate:identity-stages-are-diagnostic´
#[test]
fn identity_stages_are_diagnostic() {
    let mut tracker = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));

    // Initial stage
    assert_eq!(
        tracker.convergence_stage(&crate::types::PersistentTimestamp::new(0, 0)),
        IdentityConvergenceStage::Initial
    );

    // After changes, stage may change, but it's diagnostic only
    let cells = vec![make_cell(0, 4)];
    tracker.record_change_event(&cells, &[], &cells);

    // Stage may be Stabilising, Maturing, or Stable - doesn't affect tracker functionality
    let stage = tracker.convergence_stage(&crate::types::PersistentTimestamp::new(0, 0));
    assert!(matches!(
        stage,
        IdentityConvergenceStage::Volatile
            | IdentityConvergenceStage::Stabilising
            | IdentityConvergenceStage::Maturing
            | IdentityConvergenceStage::Stable
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Composite Stage Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// With no labels seen and no calibration fitted, the composite stage is
/// ColdStart. Whatever the models are emitting at that point rests on priors
/// alone, and the stage says so rather than letting an unlabelled system
/// present itself as merely early.
///
/// ´claim:wellness:with-no-labels-the-composite-stage-is-coldstart´
/// ´test:crate:composite-0-labels-prior-only´
#[test]
fn composite_0_labels_prior_only() {
    let platt = PlattConvergenceTracker::new();
    let stage = compute_composite_stage(
        0,
        &platt,
        &[],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage, CompositeConvergenceStage::ColdStart);
}

/// Labels accumulating without a calibration fit behind them carry the
/// composite off ColdStart but no further than AnchorEmerging. Evidence is
/// arriving and the anchor is forming, yet nothing has been calibrated against
/// it — so the stage reflects data gathered rather than a model that has been
/// fitted to it.
///
/// ´claim:wellness:labels-without-a-first-calibration-fit-carry-the-composite-only-to-anchoremerging´
/// ´test:crate:composite-30-labels-anchor´
#[test]
fn composite_30_labels_anchor() {
    let platt = PlattConvergenceTracker::new(); // No refits
    let stage = compute_composite_stage(
        30,
        &platt,
        &[],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage, CompositeConvergenceStage::AnchorEmerging);
}

/// The first completed calibration fit is what advances the composite to
/// PreCalibration — a single refit on a tracker carrying labels is enough. One
/// fit means the mapping from score to probability now exists, which is a
/// different condition from having merely collected the labels to fit it with.
///
/// ´claim:wellness:the-first-calibration-fit-advances-the-composite-to-precalibration´
/// ´test:crate:composite-first-platt-sister´
#[test]
fn composite_first_platt_sister() {
    let mut platt = PlattConvergenceTracker::new();
    platt.record_refit(&PlattRefitResult::default(), 0);

    let stage = compute_composite_stage(
        50,
        &platt,
        &[],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage, CompositeConvergenceStage::PreCalibration);
}

/// The composite takes the least advanced of its gating inputs, never the most
/// advanced: fully converged calibration paired with a volatile identity set
/// reads as InteractionMaturing, and only when the same calibration is paired
/// with a stable, mature identity set does it reach SteadyState. A system is
/// exactly as converged as its worst-converged part, so no single settled
/// component can talk the whole assembly into claiming readiness.
///
/// ´claim:wellness:the-composite-stage-is-the-least-advanced-of-its-gating-inputs´
/// ´test:crate:composite-conservative-gating´
#[test]
fn composite_conservative_gating() {
    let mut platt = PlattConvergenceTracker::new();

    // Get to SisterConverging
    for i in 0_u64..5 {
        platt.record_refit(
            &PlattRefitResult {
                delta_cal: if i < 4 { 0.5 } else { 0.001 },
                ..Default::default()
            },
            i,
        );
    }

    // Create unstable identity tracker
    let mut identity = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    identity.change_rate_ewma = 0.5; // High change rate = volatile
    identity.jaccard_ewma = 0.3; // Low jaccard = volatile

    let stage = compute_composite_stage(
        500,
        &platt,
        &[&identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    // Should be InteractionMaturing because identity is not stable
    assert_eq!(stage, CompositeConvergenceStage::InteractionMaturing);

    // Now with stable identity (set first_seen far in the past for maturity)
    let mut stable_identity = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    stable_identity.change_rate_ewma = 0.01;
    stable_identity.jaccard_ewma = 0.95;
    stable_identity.total_entries = 10;
    stable_identity.first_seen = crate::types::PersistentTimestamp::new(0, 0);

    let stage2 = compute_composite_stage(
        500,
        &platt,
        &[&stable_identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage2, CompositeConvergenceStage::SteadyState);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Health Event Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A composite stage that has moved is announced, and the announcement carries
/// both endpoints — the stage left behind and the stage arrived at. A
/// subscriber that missed earlier events can therefore tell whether the
/// transition it is reading follows on from the last one it saw.
///
/// ´claim:wellness:a-stage-transition-is-announced-carrying-both-the-stage-left-and-the-stage-reached´
/// ´test:crate:event-stage-changed-emitted´
#[test]
fn event_stage_changed_emitted() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    check_convergence_event(
        CompositeConvergenceStage::ColdStart,
        CompositeConvergenceStage::AnchorEmerging,
        &tx,
        &dropped,
    );

    let event = rx.try_recv().expect("should have event");
    match event {
        HealthEvent::Convergence(ConvergenceEvent::CompositeStageChanged { from, to }) => {
            assert_eq!(from, CompositeConvergenceStage::ColdStart);
            assert_eq!(to, CompositeConvergenceStage::AnchorEmerging);
        }
        _ => panic!("expected CompositeStageChanged event"),
    }
}

/// A stage that has not moved produces nothing at all: checked against itself,
/// the transition path emits no event and the channel stays empty. The check
/// runs on every evaluation, so an event per evaluation would drown the
/// channel in restatements and evict the transitions that actually mattered.
///
/// ´claim:wellness:an-unmoved-stage-produces-no-event-at-all´
/// ´test:crate:event-no-emit-when-unchanged´
#[test]
fn event_no_emit_when_unchanged() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    check_convergence_event(
        CompositeConvergenceStage::ColdStart,
        CompositeConvergenceStage::ColdStart,
        &tx,
        &dropped,
    );

    assert!(rx.try_recv().is_err(), "should not emit event when stage unchanged");
}

/// When the event channel is full the emitting side neither blocks nor
/// panics: the overflowing event is discarded and a drop counter advances,
/// while the events already queued survive intact and remain receivable. Model
/// health reporting is a bystander to the work it observes, so a slow or
/// absent subscriber must cost a countable loss of visibility rather than
/// stalling the system being watched.
///
/// ´claim:wellness:a-full-event-channel-drops-and-counts-rather-than-blocking-or-losing-what-is-queued´
/// ´test:crate:event-channel-overflow´
#[test]
fn event_channel_overflow() {
    let (tx, rx) = crossbeam_channel::bounded::<HealthEvent>(2);
    let dropped = AtomicU64::new(0);

    // Fill channel
    check_convergence_event(
        CompositeConvergenceStage::ColdStart,
        CompositeConvergenceStage::AnchorEmerging,
        &tx,
        &dropped,
    );
    check_convergence_event(
        CompositeConvergenceStage::AnchorEmerging,
        CompositeConvergenceStage::PreCalibration,
        &tx,
        &dropped,
    );

    assert_eq!(dropped.load(Ordering::Relaxed), 0);

    // Overflow
    check_convergence_event(
        CompositeConvergenceStage::PreCalibration,
        CompositeConvergenceStage::SisterConverging,
        &tx,
        &dropped,
    );

    assert_eq!(dropped.load(Ordering::Relaxed), 1, "overflow should increment counter");

    // Drain to verify first two events are there
    assert!(rx.try_recv().is_ok());
    assert!(rx.try_recv().is_ok());
    assert!(rx.try_recv().is_err());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance NaN Defence
// ═══════════════════════════════════════════════════════════════════════════════

/// Non-finite observations are excluded when thresholds are recomputed: a
/// window seeded with NaN and both infinities among ordinary values still
/// recalibrates to finite, positive thresholds. A percentile taken over
/// unordered values would otherwise poison every subsequent comparison, and a
/// single upstream division by zero would silently disable the axis.
///
/// ´claim:wellness:non-finite-observations-are-excluded-so-recalibrated-thresholds-stay-finite´
/// ´test:crate:concordance-nan-filtered-from-window´
#[test]
fn concordance_nan_filtered_from_window() {
    let config = ConcordanceConfig {
        window_capacity: 200,
        recalibration_interval: 100,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Mix finite values with NaN/Inf — recalibration should ignore non-finite.
    for i in 0..80 {
        tracker.observe([i as f64; 4]);
    }
    // Inject NaN and Inf
    tracker.observe([f64::NAN; 4]);
    tracker.observe([f64::INFINITY; 4]);
    tracker.observe([f64::NEG_INFINITY; 4]);
    // Fill rest with finite values
    for i in 83..100 {
        tracker.observe([i as f64; 4]);
    }

    let thresholds = tracker.current_thresholds();
    assert_finite(&thresholds, "thresholds after NaN/Inf injection");
    for &t in &thresholds {
        assert!(t > 0.0, "threshold must be positive, got {t}");
    }
}

/// A floor holds the thresholds away from zero even when the window is
/// degenerate: a hundred observations all sitting at a thousandth would put
/// the percentile essentially at nothing, and the calibrated threshold still
/// comes out at the floor. A threshold at zero would treat every subsequent
/// score as exceptional, so a quiet stretch cannot be allowed to calibrate the
/// axis into permanent alarm.
///
/// ´claim:wellness:a-floor-keeps-thresholds-off-zero-when-the-observed-window-is-degenerate´
/// ´test:crate:concordance-threshold-floor-applied´
#[test]
fn concordance_threshold_floor_applied() {
    let config = ConcordanceConfig {
        window_capacity: 200,
        recalibration_interval: 100,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // All near-zero values — 80th percentile would be ~0.0 without the floor.
    for _ in 0..100 {
        tracker.observe([0.001; 4]);
    }

    let thresholds = tracker.current_thresholds();
    for &t in &thresholds {
        assert!(t >= 0.01, "threshold must respect the 0.01 floor, got {t}");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Maturing Stage
// ═══════════════════════════════════════════════════════════════════════════════

/// Stable-looking metrics are not by themselves maturity: an identity set with
/// a low change rate, high overlap and a healthy entry count, but first seen
/// only moments ago, still reads as Maturing and holds the composite at
/// InteractionMaturing despite fully converged calibration. A set that has
/// barely existed has had no opportunity to churn, so its calm is absence of
/// evidence rather than evidence of stability, and elapsed existence is the
/// gate that distinguishes the two.
///
/// ´claim:wellness:calm-metrics-on-a-newly-registered-identity-set-do-not-count-as-maturity´
/// ´test:crate:identity-maturing-blocks-composite-fully-converged´
#[test]
fn identity_maturing_blocks_composite_fully_converged() {
    let mut platt = PlattConvergenceTracker::new();
    for _ in 0..5 {
        platt.record_refit(
            &PlattRefitResult {
                delta_cal: 0.005,
                ..Default::default()
            },
            0,
        );
    }

    // Stable set metrics but registered JUST NOW — should be Maturing.
    let mut identity = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    identity.change_rate_ewma = 0.01;
    identity.jaccard_ewma = 0.95;
    identity.total_entries = 10;

    // Read at the registration instant — the maturity gate fires.
    let stage = compute_composite_stage(
        500,
        &platt,
        &[&identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(0, 0),
    );
    assert_eq!(
        stage,
        CompositeConvergenceStage::InteractionMaturing,
        "Maturing identity should block SteadyState"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serde Round-Trips
// ═══════════════════════════════════════════════════════════════════════════════

/// A calibration tracker survives being written out and read back with
/// everything that governs its next decision intact: the completed-refit
/// tally, the last fit's calibration shift and both slope terms, and the
/// pending early-refit mark. A restarted Assayer resumes the convergence
/// judgement it had reached instead of re-earning it from Initial.
///
/// ´claim:wellness:a-calibration-tracker-round-trips-with-its-tally-last-fit-and-pending-mark-intact´
/// ´test:crate:platt-tracker-serde´
#[cfg(feature = "serde")]
#[test]
fn platt_tracker_serde() {
    use serde_json;

    let mut tracker = PlattConvergenceTracker::new();
    tracker.record_refit(
        &PlattRefitResult {
            delta_cal: 0.123,
            kappa_sister: 1.5,
            kappa_anchor: 2.0,
            ..Default::default()
        },
        1000,
    );
    tracker.mark_early_refit();

    let json = serde_json::to_string(&tracker).expect("serialize");
    let restored: PlattConvergenceTracker = serde_json::from_str(&json).expect("deserialize");

    let tol = DEFAULT_TOLERANCES;
    assert_eq!(restored.refits_completed, tracker.refits_completed);
    assert_near(
        restored.last_delta_cal,
        tracker.last_delta_cal,
        tol.bit_identical,
        "restored last_delta_cal",
    );
    assert_near(
        restored.last_kappa_sister,
        tracker.last_kappa_sister,
        tol.bit_identical,
        "restored last_kappa_sister",
    );
    assert_near(
        restored.last_kappa_anchor,
        tracker.last_kappa_anchor,
        tol.bit_identical,
        "restored last_kappa_anchor",
    );
    assert_eq!(restored.early_refit_pending, tracker.early_refit_pending);
}

/// An identity tracker likewise round-trips with both its raw tallies —
/// entries, exits, current occupancy — and its smoothed churn and overlap
/// measures. The smoothed measures take many change events to build, so
/// losing them across a restart would reset the identity set to Volatile and
/// gate the composite back down for no reason connected to the set itself.
///
/// ´claim:wellness:an-identity-tracker-round-trips-with-its-tallies-and-smoothed-measures-intact´
/// ´test:crate:identity-tracker-serde´
#[cfg(feature = "serde")]
#[test]
fn identity_tracker_serde() {
    use serde_json;

    let mut tracker = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    let cells = vec![make_cell(0, 4), make_cell(100, 4)];
    tracker.record_change_event(&cells, &[], &cells);

    let json = serde_json::to_string(&tracker).expect("serialize");
    let restored: IdentityConvergenceTracker = serde_json::from_str(&json).expect("deserialize");

    assert_eq!(restored.total_entries, tracker.total_entries);
    assert_eq!(restored.total_exits, tracker.total_exits);
    assert_eq!(restored.current_cell_count, tracker.current_cell_count);
    let tol = DEFAULT_TOLERANCES;
    assert_near(
        restored.change_rate_ewma,
        tracker.change_rate_ewma,
        tol.default,
        "restored change_rate_ewma",
    );
    assert_near(
        restored.jaccard_ewma,
        tracker.jaccard_ewma,
        tol.default,
        "restored jaccard_ewma",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Concordance Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Short of a completed interval nothing is recalibrated: five hundred
/// observations against an interval of a thousand leave the calibration count
/// at zero and the placeholder thresholds untouched. A partial window is a
/// worse estimate than the neutral starting point, so the tracker waits for a
/// full one rather than calibrating to whatever has arrived so far.
///
/// (´claim:wellness:recalibration-runs-once-per-completed-interval-of-observations´)
/// ´test:crate:concordance-observe-below-interval´
#[test]
fn concordance_observe_below_interval() {
    let config = ConcordanceConfig {
        window_capacity: 5_000,
        recalibration_interval: 1_000,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Observe 500 times (below interval)
    for i in 0..500 {
        tracker.observe([i as f64; 4]);
    }

    let health = tracker.health();
    assert_eq!(health.calibrations_completed, 0, "should not recalibrate below interval");
    // Thresholds unchanged
    assert_eq!(tracker.current_thresholds(), [0.5; 4]);
}

/// The pacing keeps holding as observation continues: two thousand
/// observations against an interval of a thousand yield two recalibrations,
/// not one and not a count that has drifted with the arrival pattern.
///
/// (´claim:wellness:recalibration-runs-once-per-completed-interval-of-observations´)
/// ´test:crate:concordance-second-recalibration´
#[test]
fn concordance_second_recalibration() {
    let config = ConcordanceConfig {
        window_capacity: 5_000,
        recalibration_interval: 1_000,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Observe 2,000 times
    for i in 0..2_000 {
        tracker.observe([i as f64 % 3.0; 4]);
    }

    let health = tracker.health();
    assert_eq!(health.calibrations_completed, 2, "should have 2 recalibrations");
}

/// A reader running concurrently with a writer that keeps crossing
/// recalibration boundaries always sees a whole, finite, non-negative
/// threshold set, and completes every one of its reads without being starved.
/// Thresholds are swapped as a unit rather than mutated in place, so a scoring
/// thread can never catch a set half-updated — part old regime, part new.
///
/// ´claim:wellness:a-concurrent-reader-never-catches-a-half-updated-threshold-set´
/// ´test:crate:concordance-recalibration-during-read´
#[test]
fn concordance_recalibration_during_read() {
    use std::sync::Arc;
    use std::thread;

    let config = ConcordanceConfig {
        window_capacity: 1_000,
        recalibration_interval: 100,
        calibration_percentile: 80.0,
    };
    let tracker = Arc::new(ConcordanceTracker::new(config));

    let tracker_writer = Arc::clone(&tracker);
    let tracker_reader = Arc::clone(&tracker);

    // Writer thread: observes values that will trigger recalibration
    let writer = thread::spawn(move || {
        for i in 0..500 {
            tracker_writer.observe([i as f64; 4]);
        }
    });

    // Reader thread: continuously reads thresholds
    let reader = thread::spawn(move || {
        let mut reads = 0;
        for _ in 0..1000 {
            let thresholds = tracker_reader.current_thresholds();
            // Thresholds should be valid (finite, non-negative)
            assert_finite(&thresholds, "concurrent reader thresholds");
            for &t in &thresholds {
                assert!(t >= 0.0, "threshold should be non-negative");
            }
            reads += 1;
        }
        reads
    });

    writer.join().expect("writer thread panicked");
    let reads = reader.join().expect("reader thread panicked");
    assert!(reads == 1000, "reader should complete all reads");
}

/// The health snapshot reports what an operator needs to judge whether the
/// thresholds can be trusted: how full the window is against its capacity, how
/// many recalibrations have run, how many observations have been seen in total,
/// and the thresholds currently in force. Observations beyond capacity raise
/// the total and the calibration count while the window stays pinned at
/// capacity — the snapshot distinguishes how much has been seen from how much
/// is still being weighed.
///
/// ´claim:wellness:the-concordance-snapshot-separates-total-observations-from-what-the-bounded-window-still-weighs´
/// ´test:crate:concordance-health´
#[test]
fn concordance_health() {
    let config = ConcordanceConfig {
        window_capacity: 1_000,
        recalibration_interval: 500,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config);

    // Observe 2,500 times
    for i in 0..2_500 {
        tracker.observe([(i % 10) as f64; 4]);
    }

    let health = tracker.health();

    assert_eq!(health.window_size, 1_000, "window should be at capacity");
    assert_eq!(health.window_capacity, 1_000);
    assert_eq!(health.calibrations_completed, 5, "2500/500 = 5 recalibrations");
    assert_eq!(health.total_observations, 2_500);
    // Thresholds should be populated
    assert_finite(&health.current_thresholds, "calibrated thresholds");
    for &t in &health.current_thresholds {
        assert!(t > 0.0, "threshold should be positive after calibration");
    }
}

/// A checkpoint carries the live window, the thresholds in force and the
/// counters, and a tracker rebuilt from it reports all of them unchanged. The
/// captured window is bounded by capacity rather than by everything ever
/// observed, so the checkpoint stays a fixed size however long the tracker has
/// run — and a restarted Assayer resumes with calibrated thresholds instead of
/// falling back to placeholders while it re-fills a window.
///
/// ´claim:wellness:a-checkpoint-restores-the-bounded-window-the-thresholds-and-the-counters´
/// ´test:crate:concordance-checkpoint-state-round-trip´
#[test]
fn concordance_checkpoint_state_round_trip() {
    let config = ConcordanceConfig {
        window_capacity: 4,
        recalibration_interval: 2,
        calibration_percentile: 80.0,
    };
    let tracker = ConcordanceTracker::new(config.clone());

    for i in 0..6 {
        tracker.observe([f64::from(i), f64::from(i + 1), f64::from(i + 2), f64::from(i + 3)]);
    }

    let original_health = tracker.health();
    let state = tracker.checkpoint_state();
    assert_eq!(state.window.len(), 4, "checkpoint keeps bounded live window");

    let restored = ConcordanceTracker::from_checkpoint(config, state);
    let restored_health = restored.health();

    assert_eq!(restored.current_thresholds(), original_health.current_thresholds);
    assert_eq!(restored_health.window_size, original_health.window_size);
    assert_eq!(restored_health.calibrations_completed, original_health.calibrations_completed);
    assert_eq!(restored_health.total_observations, original_health.total_observations);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Composite Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Concordance is measured but never gates: with calibration converged and the
/// identity set stable and mature, the composite reaches SteadyState even
/// though no concordance observation has ever been made. Concordance describes
/// agreement between axes rather than whether a model has been learned, so
/// holding the whole system back on it would leave the composite hostage to a
/// diagnostic.
///
/// ´claim:wellness:concordance-does-not-gate-composite-advancement´
/// ´test:crate:composite-concordance-not-gating´
#[test]
fn composite_concordance_not_gating() {
    // Concordance tracker state does NOT gate composite stage advancement

    let mut platt = PlattConvergenceTracker::new();
    // Get to converged state
    for _ in 0..5 {
        platt.record_refit(
            &PlattRefitResult {
                delta_cal: 0.005,
                ..Default::default()
            },
            0,
        );
    }

    // Stable identity (set first_seen far in the past for maturity)
    let mut identity = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    identity.change_rate_ewma = 0.01;
    identity.jaccard_ewma = 0.95;
    identity.total_entries = 10;
    identity.first_seen = crate::types::PersistentTimestamp::new(0, 0);

    // Even with no concordance observations, composite should be SteadyState
    // (concordance is not a gating factor)
    let stage = compute_composite_stage(
        500,
        &platt,
        &[&identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage, CompositeConvergenceStage::SteadyState);
}

/// Challenge sufficiency is likewise outside the gate: with calibration
/// converged and the identity set stable and mature, the composite reads
/// SteadyState whether challenges are reported sufficient or not. The flag
/// makes no difference in either direction, so a deployment that never
/// generates challenges is not thereby stuck short of steady state.
///
/// ´claim:wellness:challenge-sufficiency-does-not-gate-composite-advancement´
/// ´test:crate:composite-challenge-not-gating´
#[test]
fn composite_challenge_not_gating() {
    // Challenge effectiveness does NOT gate composite stage advancement

    let mut platt = PlattConvergenceTracker::new();
    for _ in 0..5 {
        platt.record_refit(
            &PlattRefitResult {
                delta_cal: 0.005,
                ..Default::default()
            },
            0,
        );
    }

    let mut identity = IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0));
    identity.change_rate_ewma = 0.01;
    identity.jaccard_ewma = 0.95;
    identity.total_entries = 10;
    identity.first_seen = crate::types::PersistentTimestamp::new(0, 0);

    // challenge_sufficient = false does NOT block advancement
    let stage = compute_composite_stage(
        500,
        &platt,
        &[&identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage, CompositeConvergenceStage::SteadyState);

    // challenge_sufficient = true also works
    let stage2 = compute_composite_stage(
        500,
        &platt,
        &[&identity],
        DEFAULT_PLATT_CONVERGED_DELTA_CAL,
        DEFAULT_PLATT_CONVERGED_REFIT_COUNT,
        &crate::types::PersistentTimestamp::new(400 * 3600, 0),
    );
    assert_eq!(stage2, CompositeConvergenceStage::SteadyState);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Additional Health Event Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// An emitted health event arrives at the receiver as the same variant that
/// was sent — here a payload-free milestone marking batch initialisation
/// finished. The channel is a transport and not a translator: a subscriber
/// matching on the variant it cares about gets exactly what the model
/// reported.
///
/// ´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´
/// ´test:crate:event-batch-init-complete´
#[test]
fn event_batch_init_complete() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    let event = HealthEvent::Convergence(ConvergenceEvent::BatchInitComplete);
    crate::health::emit_health_event(event, &tx, &dropped);

    let received = rx.try_recv().expect("should receive event");
    match received {
        HealthEvent::Convergence(ConvergenceEvent::BatchInitComplete) => {}
        _ => panic!("expected BatchInitComplete"),
    }
}

/// A sentinel finishing bootstrap is reported with the identifier of the
/// sentinel it concerns, and that identifier survives the crossing. Several
/// sentinels bootstrap independently, so an event that lost its subject would
/// tell a subscriber only that something somewhere had finished.
///
/// (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´)
/// ´test:crate:event-bootstrap-complete´
#[test]
fn event_bootstrap_complete() {
    use crate::types::SentinelId;

    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    let sentinel_id = SentinelId(42);
    let event = HealthEvent::Convergence(ConvergenceEvent::SentinelBootstrapComplete { sentinel_id });
    crate::health::emit_health_event(event, &tx, &dropped);

    let received = rx.try_recv().expect("should receive event");
    match received {
        HealthEvent::Convergence(ConvergenceEvent::SentinelBootstrapComplete { sentinel_id: id }) => {
            assert_eq!(id, SentinelId(42));
        }
        _ => panic!("expected SentinelBootstrapComplete"),
    }
}

/// The first calibration fit is reported with the numbers that characterise
/// it — both slope terms and the calibration shift — each arriving unchanged.
/// The milestone on its own says only that a fit happened; the payload is what
/// lets an operator see whether that first fit was plausible or wild.
///
/// (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´)
/// ´test:crate:event-platt-first-fit´
#[test]
fn event_platt_first_fit() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    let event = HealthEvent::Convergence(ConvergenceEvent::PlattFirstFit {
        kappa_sister: 1.5,
        kappa_anchor: 2.0,
        delta_cal: 0.42,
    });
    crate::health::emit_health_event(event, &tx, &dropped);

    let received = rx.try_recv().expect("should receive event");
    match received {
        HealthEvent::Convergence(ConvergenceEvent::PlattFirstFit {
            kappa_sister,
            kappa_anchor,
            delta_cal,
        }) => {
            let tol = DEFAULT_TOLERANCES;
            assert_near(kappa_sister, 1.5, tol.bit_identical, "PlattFirstFit kappa_sister");
            assert_near(kappa_anchor, 2.0, tol.bit_identical, "PlattFirstFit kappa_anchor");
            assert_near(delta_cal, 0.42, tol.bit_identical, "PlattFirstFit delta_cal");
        }
        _ => panic!("expected PlattFirstFit"),
    }
}

/// Calibration reaching convergence is reported with the evidence for the
/// claim: how many refits it took and how small the final calibration shift
/// was. A subscriber can check the declaration against its own thresholds
/// rather than taking the word Converged on trust.
///
/// (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´)
/// ´test:crate:event-platt-converged´
#[test]
fn event_platt_converged() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    let event = HealthEvent::Convergence(ConvergenceEvent::PlattConverged {
        refits_completed: 5,
        delta_cal: 0.005,
    });
    crate::health::emit_health_event(event, &tx, &dropped);

    let received = rx.try_recv().expect("should receive event");
    match received {
        HealthEvent::Convergence(ConvergenceEvent::PlattConverged {
            refits_completed,
            delta_cal,
        }) => {
            assert_eq!(refits_completed, 5);
            assert_near(delta_cal, 0.005, DEFAULT_TOLERANCES.bit_identical, "PlattConverged delta_cal");
        }
        _ => panic!("expected PlattConverged"),
    }
}

/// An identity set settling is reported per dimension, carrying the dimension
/// it belongs to. Dimensions stabilise at their own pace, so the report is
/// only actionable if it names which one has arrived.
///
/// (´claim:wellness:an-emitted-health-event-reaches-the-receiver-as-the-same-variant-with-its-payload-intact´)
/// ´test:crate:event-identity-stabilised´
#[test]
fn event_identity_stabilised() {
    use crate::types::DimensionId;

    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    let dimension_id = DimensionId(7);
    let event = HealthEvent::Convergence(ConvergenceEvent::IdentityStabilised { dimension_id });
    crate::health::emit_health_event(event, &tx, &dropped);

    let received = rx.try_recv().expect("should receive event");
    match received {
        HealthEvent::Convergence(ConvergenceEvent::IdentityStabilised { dimension_id: id }) => {
            assert_eq!(id, DimensionId(7));
        }
        _ => panic!("expected IdentityStabilised"),
    }
}

/// Events are received in the order they were emitted: a run of stage
/// transitions climbing from ColdStart to SteadyState drains as that same
/// chain, each event's starting stage matching the previous one's arrival.
/// Convergence is a narrative, and a reordered drain would let a subscriber
/// reconstruct a history the system never lived through.
///
/// ´claim:wellness:queued-events-drain-in-the-order-they-were-emitted´
/// ´test:crate:event-drain´
#[test]
fn event_drain() {
    let (tx, rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
    let dropped = AtomicU64::new(0);

    // Emit 5 events
    for i in 0..5 {
        let from = match i {
            0 => CompositeConvergenceStage::ColdStart,
            1 => CompositeConvergenceStage::AnchorEmerging,
            2 => CompositeConvergenceStage::PreCalibration,
            3 => CompositeConvergenceStage::SisterConverging,
            _ => CompositeConvergenceStage::InteractionMaturing,
        };
        let to = match i {
            0 => CompositeConvergenceStage::AnchorEmerging,
            1 => CompositeConvergenceStage::PreCalibration,
            2 => CompositeConvergenceStage::SisterConverging,
            3 => CompositeConvergenceStage::InteractionMaturing,
            _ => CompositeConvergenceStage::SteadyState,
        };
        check_convergence_event(from, to, &tx, &dropped);
    }

    // Drain and verify order
    let mut events = Vec::new();
    while let Ok(event) = rx.try_recv() {
        events.push(event);
    }

    assert_eq!(events.len(), 5, "should drain 5 events");

    // Verify order
    for (i, event) in events.iter().enumerate() {
        match event {
            HealthEvent::Convergence(ConvergenceEvent::CompositeStageChanged { from, to }) => {
                let expected_from = match i {
                    0 => CompositeConvergenceStage::ColdStart,
                    1 => CompositeConvergenceStage::AnchorEmerging,
                    2 => CompositeConvergenceStage::PreCalibration,
                    3 => CompositeConvergenceStage::SisterConverging,
                    _ => CompositeConvergenceStage::InteractionMaturing,
                };
                let expected_to = match i {
                    0 => CompositeConvergenceStage::AnchorEmerging,
                    1 => CompositeConvergenceStage::PreCalibration,
                    2 => CompositeConvergenceStage::SisterConverging,
                    3 => CompositeConvergenceStage::InteractionMaturing,
                    _ => CompositeConvergenceStage::SteadyState,
                };
                assert_eq!(*from, expected_from, "event {i} from mismatch");
                assert_eq!(*to, expected_to, "event {i} to mismatch");
            }
            _ => panic!("expected CompositeStageChanged"),
        }
    }
}
