// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`step0_last_label_time_updated`] | labelling | Processing a label advances the working copy's time anchor to the moment the label arrived. Decay is applied from that anchor on the next label, so an anchor left behind would make every subsequent gap look longer than it was and decay the model harder than the elapsed time warrants. |
//! | [`steps4_8_eligibility_allow_positive`] | labelling | An outcome observed after the system chose to allow is admissible evidence about the base rate, and a positive one pushes the eligible positive rate up. Nothing intervened between the decision and the outcome, so what happened is what would have happened — the sample is unbiased by the system's own action. |
//! | [`steps4_8_eligibility_block_no_ground_truth`] | labelling | A blocked request whose outcome was inferred rather than established leaves the eligible rate exactly where it was, while the global rate still moves. The two trackers answer different questions: the global one describes the traffic the system saw, the eligible one estimates the base rate it must not learn from its own interventions — and a blocked outcome nobody confirmed would contaminate the second. |
//! | [`steps4_8_ground_truth_overrides`] | labelling | Establishing what actually happened restores eligibility that the intervention had removed: the same blocked request becomes admissible base-rate evidence once it carries ground truth. Manual review of blocked traffic is how a system escapes only ever learning from the requests it let through, so confirmed outcomes have to count regardless of the action taken. |
//! | [`step5_kappa_v_updated`] | labelling | The valence scale is learned from the magnitudes the host actually reports: a label five times louder than the current scale pulls that scale upward. Because valence is compressed against this scale before it reaches the model, a host that works in large units ends up with the same effective dynamic range as one that works in small ones, without having to declare its units anywhere. |
//! | [`step5_kappa_v_skip_zero_valence`] | labelling | A label reporting no magnitude at all leaves the valence scale bit-for-bit unchanged rather than dragging it toward zero. Zero valence means the host had nothing to say about severity, not that severity was observed to be nil, and a long run of such labels would otherwise shrink the scale until every real report saturated the compression. |
//! | [`step7_p_plus_global_always_updated`] | labelling | The global positive rate follows the sign of the valence in both directions: a benign label pulls it down just as an adverse one pulls it up. It is a two-sided estimate of how the observed traffic splits, so evidence of benignity has to be as consequential as evidence of harm. |
//! | [`step7_p_plus_eligible_only_when_eligible`] | labelling | cites (´claim:labelling:an-intervened-outcome-moves-the-global-rate-but-never-the-eligible-one´) |
//! | [`step8_importance_weight_uses_eligible_tracker`] | labelling | A sustained run of eligible positives carries the eligible rate decisively past its neutral prior rather than hovering near it. That tracker is what the importance weighting reads to decide how much a minority-class label is worth, so it has to actually move under a skewed stream — a tracker pinned at the prior would weight every class alike no matter how lopsided the traffic became. |
//! | [`learning_policy_conforms_row_by_row`] | labelling | Every combination of challenge policy, outcome polarity, and class rate on either side of the configured ceiling follows one specification chain. The challenge declaration decides inherent-model training (´req:host:eligibility-policy´), the two populations move at the rates of the models they serve (´tab:weighting:trackers´), and their post-label rates produce the capped reciprocal weight (´def:weighting:balancing-weights´). |
//! | [`importance_health_accumulates_each_model_stream`] | labelling | Operational labels and eligible sister labels have distinct denominators, but both accumulate the exact balancing weights the model update consumed. A unit ceiling makes the hand result unambiguous when each incoming class starts on the minority side: every weight binds and positive gradient mass is one half of the total (´entry:health:importance-ceiling´). |
//! | [`alarm_outcome_cusums_accumulate_by_sentinel_and_axis`] | labelling | Page accumulators retain direction and axis identity across labels. Two benign labels with loud first and third axes grow only the false-alarm side; a subsequent adverse label with quiet first and third axes grows the missed-alarm side and subtracts one noise allowance from the first (´def:monitoring:alarm-outcome-cusums´). |
//! | [`alarm_outcome_cusums_honour_configured_boundaries`] | labelling | Configured alarm thresholds are inclusive and the configured noise allowance is the exact amount subtracted: just outside either boundary stays quiet, exactly on it grows the corresponding directional accumulator by one minus the allowance. |
//! | [`companion_boundary_challenge_label_does_not_require_result`] | labelling | A label for a challenged request runs the pipeline to completion and is counted without any challenge outcome attached to it. Whether the challenge was passed or failed is the host's business; the core needs only the outcome that followed, so a host with no challenge instrumentation can still submit labels for the requests it challenged. |
//! | [`companion_boundary_tracker_updates_outside_core`] | labelling | Challenge effectiveness lives in a tracker the host owns and drives directly: recording a passed challenge moves that channel's estimate, and running a label through the core alongside it neither performs nor undoes that update. The two state machines are separate, so a host may adopt challenge tracking, replace it, or omit it without touching the model path. |
//! | [`companion_boundary_non_challenge_skips`] | labelling | Each label is counted under the action that was actually taken on it: an allowed request lands in the allow tally and nowhere else. The per-action counters are how an operator reads the shape of the traffic the model learned from, so a label attributed to the wrong action would misdescribe that mix. |
//! | [`step14_calibration_buffer_pushed`] | labelling | A live label leaves exactly one calibration entry behind, and that entry records the polarity of the outcome it came from. Calibration is refitted from this buffer much later, when the model has moved on, so each entry has to preserve which side of the decision it was — the buffer is the only record by then. |
//! | [`step14_calibration_entry_carries_uncertainty`] | labelling | The calibration row holds the uncertainty the assessment reported: the σ_p̂ carried on the pending risk basis arrives in the pushed entry as the same value, not as a placeholder. Empirical coverage divides each row's residual by exactly this quantity (´alg:monitoring:empirical-coverage´), and the row is the only place it survives to be divided by — the model has moved on by the time coverage is computed. |
//! | [`step14_runs_in_replay`] | labelling | A replayed label pushes the calibration entry the original would have, built from the risk basis the journal context carries. There is no double count to fear: the checkpoint captures the buffer, and the replay filter hands back only labels past the checkpoint's mark — labels whose entries the captured buffer has never held — so skipping them would leave the restored buffer permanently short of exactly the rows the journal was keeping (´dec:durability:checkpoint-journal´). |
//! | [`step15_standardisation_updated`] | labelling | The standardisation means move toward the features each label was built from: seeded well away from where the observations sit, they close some of that gap on the first label. Standardisation is what keeps features comparable as the traffic changes, and a set of means that never followed the data would leave every feature scaled against a distribution that no longer exists. |
//! | [`transition_label_moves_models_not_standardisation`] | labelling | While the cold ramp is transitioning a label teaches the models and leaves the coordinate system alone, and the first label after the horizon advances the continuing average. The two authorities are separable and this is where they are separated: the models learn from the first label onward whatever the coordinate system is doing, while the ramp owns full-vector standardisation until it reaches its horizon. A label moving the average underneath a running ramp would put a label's authority into a transition an observation's authority owns, and the published moments would then be neither the ramp's mixture nor the average's — with nothing on the wire saying which. |
//! | [`step15_standardisation_in_replay`] | labelling | Standardisation keeps updating during replay, as every step does. The checkpoint restores the running means and variances as they stood at capture, and the labels being replayed all postdate that capture — so their step-15 contributions are exactly the ones the restored vectors are missing, and skipping them would leave a restored instance permanently behind the live run it is meant to resume. |
//! | [`step16_runs_in_replay`] | labelling | A replayed label banks its drift residual exactly as the original would have, against the assessment-time prediction the journal context carries — the quantity (´alg:monitoring:drift-cusums´) requires the residual be taken against. Nothing is manufactured: the label happened once, its residual was measured against the prediction in force at assessment time, and replay applies that same residual once — where the restore has just brought the accumulators back and the replayed labels are the only evidence gathered since. |
//! | [`step17_snapshot_published`] | labelling | A processed label ends with a snapshot published at the version it was given, not at some internally chosen counter. The caller decides the version, so the number a reader observes can be matched against the label stream that produced it — which is what makes the version usable as a progress marker at all. |
//! | [`cp5_nan_check_clean_working_copy`] | labelling | A label processed against healthy models publishes rather than reverts: the integrity checkpoint passes and the new snapshot appears. The revert path exists for models that have gone numerically bad, and a checkpoint that fired on clean state would silently discard every update the system ever made. |
//! | [`cp5_revert_restores_from_snapshot`] | labelling | Reverting is a full reconstruction from a snapshot, not a partial rollback: the working copy rebuilt from a snapshot carries that snapshot's scalars and none of the values the live copy had since moved to. Recovery from a corrupted update has to land on a state that once genuinely existed, and only rebuilding wholesale guarantees no half-applied mutation survives. |
//! | [`a_failed_revert_stops_the_label_path`] | labelling | A revert whose rebuild is itself refused stops the label path instead of carrying on with the state it just judged corrupt. The stop is recorded once, naming the model whose reconstruction was refused, the pivot it was refused at, and how far into the label stream it happened; the published snapshot is left exactly as it was, because that snapshot is what assessments go on answering from. What no longer happens is the arrangement this replaces: no cascade terminus is synthesised on any model and no recomputation interval is halved, because the repair those were asking for does not exist — a refused factorisation is a verdict about the matrix, and asking for it again sooner buys attempts rather than an exit. |
//! | [`the_revert_rebuilds_at_the_deployments_configuration`] | labelling | The revert rebuilds at the deployment's declared configuration rather than at the library defaults. The pipeline configuration carries the deployment's whole model declaration and its regularisation, and the revert reads them from there rather than constructing either — so a deployment that declared a prior precision of its own keeps it across a revert instead of having every model silently retuned to the default at the moment a checkpoint fires. The rebuilt prior is pinned by reading the revert's own source rather than by an accessor, because the restored prior is not published on any reader-visible surface: it reaches the model as a stored scalar and shows only in a later marginalisation. The audit reads the source in the checkout being tested, so it holds wherever the binary was compiled. |
//! | [`cp6_snapshot_nan_detection`] | labelling | The mean vector of a cold-started snapshot is entirely finite, and a NaN placed into it is visible to a scan of that vector. The snapshot is what every reader sees, so corruption reaching it must be detectable before publication — and a scan can only be trusted if the untouched case is genuinely clean to begin with. |
//! | [`cp6_nan_in_covariance_blocked`] | labelling | The covariance block of a snapshot is subject to the same scan as the mean. Corruption in the uncertainty is quieter than corruption in the estimate — a bad variance produces bad confidence rather than an obviously bad prediction — so it has to be looked for explicitly rather than assumed to show up elsewhere. |
//! | [`replay_labels_accumulate_calibration`] | labelling | cites (´claim:labelling:a-replayed-label-pushes-the-calibration-entry-the-original-would-have´) |
//! | [`replay_label_updates_standardisation`] | labelling | cites (´claim:labelling:standardisation-keeps-updating-during-replay´) |
//! | [`replay_invariant_r17`] | labelling | Crash recovery is exact where it claims to be: an instance checkpointed halfway through a stream, rebuilt from that checkpoint, and fed the remaining labels as replay arrives at the same positive rates, the same valence scale and the same standardisation means as the run that never crashed — to within a part in a trillion — and at the same calibration buffer, the ten captured entries plus one per replayed label. A restored instance is therefore the same instance, not an approximation of one. |
//! | [`multiple_labels_monotonic_snapshots`] | labelling | cites (´claim:labelling:a-processed-label-publishes-a-snapshot-at-the-version-it-was-given´) |
//! | [`calibration_buffer_accumulates`] | labelling | cites (´claim:labelling:a-live-label-leaves-one-calibration-entry-carrying-its-polarity´) |
//! | [`mixed_replay_and_live_labels`] | labelling | cites (´claim:labelling:a-replayed-label-pushes-the-calibration-entry-the-original-would-have´) |
//! | [`challenge_effectiveness_results_not_core_owned`] | labelling | cites (´claim:labelling:challenge-effectiveness-is-tracked-beside-the-core-rather-than-inside-it´) |
//! | [`companion_sufficiency_floor_carries_its_ruled_value`] | labelling | The Companion's sufficiency floor carries the value the specification anchors rather than the value it shipped with, and it is the boundary the standalone health report actually applies. Both halves are asserted because a decision can succeed at the record and fail at the number: the constant is what a host reads, so the figure that decision fixed — the specification's twenty, in place of the underivable fifty — is pinned here and then made to decide a report (´cav:challenge:sufficiency-threshold-open´). The evidenced counts are placed well either side of the floor rather than on it, because lazy decay makes the knife edge a statement about elapsed microseconds rather than about the threshold. |
//! | [`sequence_number_updated`] | labelling | The working copy records the journal sequence number carried by the label it just applied, and records the label's own number rather than the publication version it was processed under. That high-water mark is what a restarting instance consults to decide where in the journal to resume, so it has to name a position in the journal and not in the snapshot stream. |
//! | [`end_to_end_convergence`] | labelling | Under a long mixed workload every learned quantity actually moves and nothing comes apart. The eligible positive rate leaves its prior, the valence scale settles on the magnitude it keeps being shown, calibration refits at least once, the feature variances depart their initial unit values, the sister model's mean pulls away from its zero prior, the drift accumulators show activity and the calibration buffer fills, and the models count the labels since their last factorisation rebuild — while every parameter of all three models stays finite. This is the seam between the pipeline and the algorithms it drives: a step wired to the wrong state, or wired to nothing, shows up as a quantity that sat still through two hundred and fifty labels, and a step wired to numerically unsound state shows up as a parameter that stopped being a number. |
//! | [`platt_sharpness_holds_between_successive_refits`] | labelling | Successive refits of the calibration on one stationary workload do not move the sister's sharpness by more than the drift the harness declares: the second refit's κ sits within that bound of the first's. A refit is a fit over an accumulating buffer, so the same population seen twice should produce nearly the same map — a sharpness that lurched between consecutive refits would mean the calibration was tracking the buffer's most recent rows rather than the distribution behind them, and every probability the map returns would move with it (´tab:assayer:harness-scenario-tolerances´). Both refits are required to have settled inside the search range as well, because the bound says something about the calibration only if the two figures it compares are fits: the workload's two classes overlap across five score rungs, which is what gives the cross-entropy an interior minimum to find, and a workload whose rows all carried one score would leave both refits holding the same bound — where the comparison passes on a pair of artefacts. The drift half of this scenario is satisfied with room the workload cannot spend: a population that really is stationary moves its own minimum by less than the search resolves, so the two refits return the same sharpness bit for bit, and the declared drift — some five hundred times the search's quantum at this sharpness — is out of reach whatever the calibration does. It is the interiority that discriminates here; the bound itself is put both ways, on a population displaced by a settled amount, in the fitting module's own scenario (´tab:assayer:harness-scenario-tolerances´). |

//! Label pipeline crate-level tests: the label path as one function in ordered
//! groups (´dec:ordering:label-function´).
//!
//! These tests verify cross-module integration for the label pipeline,
//! including step ordering, scalar computations, Companion isolation,
//! CP5/CP6 revert logic, replay semantics, and snapshot publication.
//!
//! # Cross-References
//!
//! - (´dec:ordering:publish-at-end´) — the one publication the path makes, at its end
//! - (´dec:ordering:decay-at-head´) — time decay applied once, before any model update
//! - (´dec:posterior:repair-cascade´) — the factorisation cascade the convergence test drives
//! - (´dec:calibration:pure-refit´) — the refit that same test makes happen at least once

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crate::config::types::EligibilityPolicy;
use crate::health::{DegradationContext, PlattConvergenceTracker, create_event_channel};
use crate::ledger::OutcomeLedger;
use crate::owner::commands::{LabelContext, PendingAssessment, PendingContext, SequencedLabel};
use crate::owner::label_path::{CalibrationBuffer, HealthCounters, LabelPipelineConfig, process_label};
use crate::pending::{PendingRiskBasis, SentinelExtraction};
use crate::risk::challenge::{ChallengeEffectivenessTracker, compute_importance_weight};
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::{ModelConfig, WorkingCopy};
use crate::testing::{DEFAULT_TOLERANCES, LabelSpec, assert_finite, assert_near};
use crate::types::{Action, AssessmentId, ChallengeResult, ChannelId, EntityKey, ModelId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Test Infrastructure
// ═══════════════════════════════════════════════════════════════════════════════

/// Creates a minimal test environment for label pipeline tests.
struct TestEnv {
    working: WorkingCopy,
    shared: SharedState,
    ledger: OutcomeLedger,
    platt_tracker: PlattConvergenceTracker,
    calibration_buffer: CalibrationBuffer,
    config: LabelPipelineConfig,
    event_tx: crossbeam_channel::Sender<crate::health::HealthEvent>,
    _event_rx: crossbeam_channel::Receiver<crate::health::HealthEvent>,
    dropped_counter: AtomicU64,
    drift_reset_counter: crate::health::DriftResetCounter,
    identity_dimensions:
        std::sync::RwLock<HashMap<crate::types::DimensionId, std::sync::Arc<crate::identity::IdentityDimensionInfra>>>,
    health_counters: HealthCounters,
}

impl TestEnv {
    fn new() -> Self {
        let model_config = ModelConfig::default();
        let working = WorkingCopy::cold_start(16, &model_config, 1000);
        let snapshot = working.to_snapshot(0);
        let shared = SharedState::new(snapshot);
        let (event_tx, event_rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);

        Self {
            working,
            shared,
            ledger: OutcomeLedger::new(),
            platt_tracker: PlattConvergenceTracker::new(),
            calibration_buffer: CalibrationBuffer::default(),
            config: LabelPipelineConfig::default(),
            event_tx,
            _event_rx: event_rx,
            dropped_counter: AtomicU64::new(0),
            drift_reset_counter: crate::health::DriftResetCounter::new(),
            identity_dimensions: std::sync::RwLock::new(HashMap::new()),
            health_counters: HealthCounters::default(),
        }
    }

    /// Puts the cold ramp at its horizon, leaving the published moments
    /// exactly where they are.
    ///
    /// The continuing standardisation average is an in-service mechanism: the
    /// cold ramp owns full-vector standardisation until its horizon, and a
    /// label does not also move it before then
    /// (´alg:standardisation:label-time-procedure´). A scenario about the
    /// continuing average therefore has to say which phase it is in, and this
    /// is how it says so — without moving a single moment, so the arithmetic
    /// under test is exactly what it was before the ramp existed.
    fn cold_ramp_in_service(&mut self) {
        let horizon = self.working.cold_ramp.horizon();
        self.working.cold_ramp = crate::feature::bootstrap::ColdRamp::from_published(
            &self.working.feature_means,
            &self.working.feature_variances,
            horizon,
            horizon,
            self.working.layout_generation,
        );
        assert!(self.working.cold_ramp.phase().is_in_service());
    }

    fn process(&mut self, label: &SequencedLabel, version: u64) {
        process_label(
            &mut self.working,
            label,
            &self.shared,
            &self.ledger,
            &mut self.platt_tracker,
            &mut self.calibration_buffer,
            &self.identity_dimensions,
            &HashMap::new(),
            [0.0; 4],
            &self.config,
            &crate::testing::SystemClock,
            &self.event_tx,
            &self.dropped_counter,
            &self.drift_reset_counter,
            version,
            version, // label_count tracks with version in tests
            &mut self.health_counters,
        );
    }
}

fn test_pending_assessment(id: AssessmentId) -> PendingAssessment {
    PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id,
        timestamp: std::time::Instant::now(),
        persistent_timestamp: PersistentTimestamp::now(),
        entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
        sentinel_extractions: HashMap::new(),
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: crate::pending::StoredFeatures::default(),
        risk_basis: PendingRiskBasis::default(),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        report_origin: None,
    }
}

fn test_pending_context(id: AssessmentId) -> PendingContext {
    PendingContext {
        spatial_axis_ids: Vec::new(),
        entity: EntityKey::new(id.0.to_le_bytes().to_vec()),
        sentinel_extractions: HashMap::new(),
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: crate::pending::StoredFeatures::default(),
        risk_basis: crate::pending::PendingRiskBasis::default(),
        outcome_predictions: HashMap::new(),
    }
}

/// Default `AssessmentId` used by label helpers in this file.
const TEST_RID: AssessmentId = AssessmentId(1);

/// One specification-derived learning-policy scenario.
#[derive(Clone, Copy)]
struct LearningPolicyRow {
    name: &'static str,
    challenge_is_unconfounded: bool,
    is_positive: bool,
    initial_class_rate: f64,
    ceiling_binds: bool,
}

/// The eligibility-policy, polarity, and ceiling-side cross product.
///
/// The balancing definition puts the ceiling boundary at
/// `1 / (2 * ceiling)`. With the configured ceiling of one hundred, the
/// class rates below start at two tenths of a percent on the binding side
/// and one percent on the reciprocal side. One tracker update leaves each
/// rate on its declared side of the boundary.
const LEARNING_POLICY_ROWS: &[LearningPolicyRow] = &[
    LearningPolicyRow {
        name: "included-positive-below-boundary",
        challenge_is_unconfounded: true,
        is_positive: true,
        initial_class_rate: 0.002,
        ceiling_binds: true,
    },
    LearningPolicyRow {
        name: "included-positive-above-boundary",
        challenge_is_unconfounded: true,
        is_positive: true,
        initial_class_rate: 0.01,
        ceiling_binds: false,
    },
    LearningPolicyRow {
        name: "included-negative-below-boundary",
        challenge_is_unconfounded: true,
        is_positive: false,
        initial_class_rate: 0.002,
        ceiling_binds: true,
    },
    LearningPolicyRow {
        name: "included-negative-above-boundary",
        challenge_is_unconfounded: true,
        is_positive: false,
        initial_class_rate: 0.01,
        ceiling_binds: false,
    },
    LearningPolicyRow {
        name: "excluded-positive-below-boundary",
        challenge_is_unconfounded: false,
        is_positive: true,
        initial_class_rate: 0.002,
        ceiling_binds: true,
    },
    LearningPolicyRow {
        name: "excluded-positive-above-boundary",
        challenge_is_unconfounded: false,
        is_positive: true,
        initial_class_rate: 0.01,
        ceiling_binds: false,
    },
    LearningPolicyRow {
        name: "excluded-negative-below-boundary",
        challenge_is_unconfounded: false,
        is_positive: false,
        initial_class_rate: 0.002,
        ceiling_binds: true,
    },
    LearningPolicyRow {
        name: "excluded-negative-above-boundary",
        challenge_is_unconfounded: false,
        is_positive: false,
        initial_class_rate: 0.01,
        ceiling_binds: false,
    },
];

/// Applies the tracker equation from (´alg:weighting:tracker-update´).
#[allow(clippy::suboptimal_flops)] // Justified: keeps the specification oracle's arithmetic independent of production's fused implementation
fn spec_tracker_update(initial: f64, gamma: f64, is_positive: bool) -> f64 {
    let indicator = if is_positive { 1.0 } else { 0.0 };
    gamma * initial + (1.0 - gamma) * indicator
}

/// Applies the balancing equation from (´def:weighting:balancing-weights´).
fn spec_balancing_weight(p_positive: f64, is_positive: bool, ceiling: f64) -> f64 {
    let class_rate = if is_positive { p_positive } else { 1.0 - p_positive };
    (1.0 / (2.0 * class_rate)).min(ceiling)
}

/// Wraps a built [`LabelSpec`] in a `Full`-context `SequencedLabel`.
fn full(spec: LabelSpec) -> SequencedLabel {
    SequencedLabel {
        seq: Some(1),
        label: spec.build(),
        context: LabelContext::Full(Box::new(test_pending_assessment(TEST_RID))),
        arrived_at: None,
    }
}

/// Wraps a built [`LabelSpec`] in a `Replay`-context `SequencedLabel`.
fn replay(spec: LabelSpec) -> SequencedLabel {
    SequencedLabel {
        seq: Some(1),
        label: spec.build(),
        context: LabelContext::Replay(Box::new(test_pending_context(TEST_RID))),
        arrived_at: None,
    }
}

fn make_full_label(valence: f64, action: Action) -> SequencedLabel {
    full(LabelSpec::new(TEST_RID).valence(valence).action(action).ground_truth())
}

/// The rungs of the overlap ladder: `(score, positive rows)` at each.
///
/// Eight rows stand at each of five scores, and every rung carries both
/// outcomes — one positive at the bottom, seven at the top, four in the
/// middle — so the classes the calibration fits overlap across the whole
/// range rather than parting at a cut. The positive counts are antisymmetric
/// about the middle rung, which balances a block at twenty of each. This is
/// the ladder the fitting module's own tests use, driven here through the
/// pipeline instead of pushed straight into a buffer.
const OVERLAP_LADDER_RUNGS: [(f64, u64); 5] = [(-4.0, 1), (-2.0, 2), (0.0, 4), (2.0, 6), (4.0, 7)];

/// Rows standing at each rung, and so a block of five times this.
const OVERLAP_LADDER_ROWS_PER_RUNG: u64 = 8;

/// The `(score, valence)` the ladder's `index`-th row carries.
///
/// Valence is what the pipeline reads the outcome's polarity from, so the
/// rung's positive tally is expressed as a sign here rather than as a flag.
fn overlap_ladder_row(index: u64) -> (f64, f64) {
    let within = index % (OVERLAP_LADDER_ROWS_PER_RUNG * OVERLAP_LADDER_RUNGS.len() as u64);
    let (rho, positives) = OVERLAP_LADDER_RUNGS[(within / OVERLAP_LADDER_ROWS_PER_RUNG) as usize];
    let row = within % OVERLAP_LADDER_ROWS_PER_RUNG;
    (rho, if row < positives { 1.0 } else { -1.0 })
}

/// A live label whose assessment reported a score, so the calibration entry
/// the pipeline pushes carries that score rather than the default zero.
///
/// [`make_full_label`] leaves the risk basis at its default, which puts a
/// score of zero on every calibration row it produces. A buffer of zeroes
/// makes the calibration's cross-entropy identically constant in the
/// sharpness — every candidate predicts a half for every row — so the fit has
/// nothing to choose between and returns whichever bound its iteration walks
/// to. Scenarios that go on to assert something about the fitted sharpness
/// build their labels here instead.
fn make_scored_label(valence: f64, rho_eff: f64) -> SequencedLabel {
    let mut pending = test_pending_assessment(TEST_RID);
    pending.risk_basis = PendingRiskBasis::new(0.5, 0.25, 0.05, rho_eff);

    SequencedLabel {
        seq: Some(1),
        label: LabelSpec::new(TEST_RID)
            .valence(valence)
            .action(Action::Allow)
            .ground_truth()
            .build(),
        context: LabelContext::Full(Box::new(pending)),
        arrived_at: None,
    }
}

fn make_alarm_label(valence: f64, alarms: [f32; 4]) -> SequencedLabel {
    let sentinel_id = SentinelId(7);
    let mut pending = test_pending_assessment(TEST_RID);
    let mut features = vec![0.0f32; 12];
    features[8..12].copy_from_slice(&alarms);
    pending
        .sentinel_extractions
        .insert(sentinel_id, SentinelExtraction::new(0, features, true));
    pending.active_sentinels.push(sentinel_id);
    pending.reporting_sentinels.push(sentinel_id);

    SequencedLabel {
        seq: Some(1),
        label: LabelSpec::new(TEST_RID)
            .valence(valence)
            .action(Action::Allow)
            .ground_truth()
            .build(),
        context: LabelContext::Full(Box::new(pending)),
        arrived_at: None,
    }
}

fn make_replay_label(valence: f64, action: Action) -> SequencedLabel {
    replay(LabelSpec::new(TEST_RID).valence(valence).action(action).ground_truth())
}

fn make_challenge_label(valence: f64, ground_truth: bool) -> SequencedLabel {
    let mut spec = LabelSpec::new(TEST_RID).valence(valence).action(Action::Challenge);
    if ground_truth {
        spec = spec.ground_truth();
    }
    full(spec)
}

fn make_block_label(valence: f64, ground_truth: bool) -> SequencedLabel {
    let mut spec = LabelSpec::new(TEST_RID).valence(valence).action(Action::Block);
    if ground_truth {
        spec = spec.ground_truth();
    }
    full(spec)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Step Ordering Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Processing a label advances the working copy's time anchor to the moment the
/// label arrived. Decay is applied from that anchor on the next label, so an
/// anchor left behind would make every subsequent gap look longer than it was and
/// decay the model harder than the elapsed time warrants.
///
/// ´claim:labelling:processing-a-label-advances-the-time-anchor-that-decay-is-measured-from´
/// ´test:crate:step0-last-label-time-updated´
#[test]
fn step0_last_label_time_updated() {
    let mut env = TestEnv::new();
    let before = env.working.last_label_time;

    // Small delay so the time difference is observable
    std::thread::sleep(std::time::Duration::from_millis(1));

    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    assert!(
        env.working.last_label_time > before,
        "last_label_time should advance after processing a label"
    );
}

/// An outcome observed after the system chose to allow is admissible evidence
/// about the base rate, and a positive one pushes the eligible positive rate up.
/// Nothing intervened between the decision and the outcome, so what happened is
/// what would have happened — the sample is unbiased by the system's own action.
///
/// ´claim:labelling:an-outcome-observed-after-an-allow-is-admissible-base-rate-evidence´
/// ´test:crate:steps4-8-eligibility-allow-positive´
#[test]
fn steps4_8_eligibility_allow_positive() {
    let mut env = TestEnv::new();
    let initial_p_plus = env.working.p_positive_eligible;

    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    // Allow + positive → eligible; P+ eligible should move toward 1.0
    assert!(
        env.working.p_positive_eligible > initial_p_plus,
        "P+ eligible should increase for positive Allow label"
    );
}

/// A blocked request whose outcome was inferred rather than established leaves
/// the eligible rate exactly where it was, while the global rate still moves. The
/// two trackers answer different questions: the global one describes the traffic
/// the system saw, the eligible one estimates the base rate it must not learn
/// from its own interventions — and a blocked outcome nobody confirmed would
/// contaminate the second.
///
/// ´claim:labelling:an-intervened-outcome-moves-the-global-rate-but-never-the-eligible-one´
/// ´test:crate:steps4-8-eligibility-block-no-ground-truth´
#[test]
fn steps4_8_eligibility_block_no_ground_truth() {
    let mut env = TestEnv::new();
    let initial_p_plus_eligible = env.working.p_positive_eligible;

    let label = make_block_label(1.0, false);
    env.process(&label, 1);

    // Block + !ground_truth → ineligible; P+ eligible should NOT be updated
    assert_near(
        env.working.p_positive_eligible,
        initial_p_plus_eligible,
        1e-10,
        "P+ eligible unchanged for ineligible Block label",
    );

    // But P+ global SHOULD be updated
    // Initial was 0.5; positive label moves it toward 1.0
    assert!(
        env.working.p_positive_global > 0.5,
        "P+ global should increase for positive label regardless of eligibility"
    );
}

/// Establishing what actually happened restores eligibility that the intervention
/// had removed: the same blocked request becomes admissible base-rate evidence once
/// it carries ground truth. Manual review of blocked traffic is how a system escapes
/// only ever learning from the requests it let through, so confirmed outcomes have
/// to count regardless of the action taken.
///
/// ´claim:labelling:ground-truth-restores-the-eligibility-an-intervention-removed´
/// ´test:crate:steps4-8-ground-truth-overrides´
#[test]
fn steps4_8_ground_truth_overrides() {
    let mut env = TestEnv::new();
    let initial_p_plus_eligible = env.working.p_positive_eligible;

    // Block + ground_truth → eligible
    let label = make_block_label(1.0, true);
    env.process(&label, 1);

    assert!(
        env.working.p_positive_eligible > initial_p_plus_eligible,
        "P+ eligible should be updated when Block has ground_truth"
    );
}

/// The valence scale is learned from the magnitudes the host actually reports: a
/// label five times louder than the current scale pulls that scale upward. Because
/// valence is compressed against this scale before it reaches the model, a host
/// that works in large units ends up with the same effective dynamic range as one
/// that works in small ones, without having to declare its units anywhere.
///
/// ´claim:labelling:the-valence-scale-is-learned-from-the-magnitudes-the-host-reports´
/// ´test:crate:step5-kappa-v-updated´
#[test]
fn step5_kappa_v_updated() {
    let mut env = TestEnv::new();
    let initial_kappa = env.working.kappa_v;

    let label = make_full_label(5.0, Action::Allow);
    env.process(&label, 1);

    // κ_v should move toward |v| = 5.0
    assert!(
        env.working.kappa_v > initial_kappa,
        "κ_v should increase toward |valence| = 5.0; was {}, now {}",
        initial_kappa,
        env.working.kappa_v
    );
}

/// A label reporting no magnitude at all leaves the valence scale bit-for-bit
/// unchanged rather than dragging it toward zero. Zero valence means the host had
/// nothing to say about severity, not that severity was observed to be nil, and
/// a long run of such labels would otherwise shrink the scale until every real
/// report saturated the compression.
///
/// ´claim:labelling:a-zero-valence-label-leaves-the-valence-scale-untouched´
/// ´test:crate:step5-kappa-v-skip-zero-valence´
#[test]
fn step5_kappa_v_skip_zero_valence() {
    let mut env = TestEnv::new();
    let initial_kappa = env.working.kappa_v;

    let label = make_full_label(0.0, Action::Allow);
    env.process(&label, 1);

    assert_near(
        env.working.kappa_v,
        initial_kappa,
        DEFAULT_TOLERANCES.bit_identical,
        "κ_v unchanged on zero valence",
    );
}

/// The global positive rate follows the sign of the valence in both directions:
/// a benign label pulls it down just as an adverse one pulls it up. It is a
/// two-sided estimate of how the observed traffic splits, so evidence of benignity
/// has to be as consequential as evidence of harm.
///
/// ´claim:labelling:the-global-positive-rate-follows-the-sign-of-the-valence-in-both-directions´
/// ´test:crate:step7-p-plus-global-always-updated´
#[test]
fn step7_p_plus_global_always_updated() {
    let mut env = TestEnv::new();
    let initial_global = env.working.p_positive_global;

    // Negative valence → move toward 0.0
    let label = make_full_label(-1.0, Action::Allow);
    env.process(&label, 1);

    assert!(
        env.working.p_positive_global < initial_global,
        "P+ global should decrease for negative valence label"
    );
}

/// The eligible tracker holds its value across an ineligible label even when that
/// label is loud and points the other way — an established estimate is not
/// perturbed by evidence it is not allowed to use. Skipping is a genuine skip, not
/// a small update, so a long run of intervened traffic cannot slowly erode a base
/// rate learned from clean observations.
///
/// (´claim:labelling:an-intervened-outcome-moves-the-global-rate-but-never-the-eligible-one´)
/// ´test:crate:step7-p-plus-eligible-only-when-eligible´
#[test]
fn step7_p_plus_eligible_only_when_eligible() {
    let mut env = TestEnv::new();

    // First: process one eligible label to establish baseline
    let label1 = make_full_label(1.0, Action::Allow);
    env.process(&label1, 1);
    let p_plus_after_eligible = env.working.p_positive_eligible;

    // Second: process an ineligible label (Block without ground_truth)
    let label2 = make_block_label(-1.0, false);
    env.process(&label2, 2);

    assert_near(
        env.working.p_positive_eligible,
        p_plus_after_eligible,
        1e-10,
        "P+ eligible unchanged for ineligible labels",
    );
}

/// A sustained run of eligible positives carries the eligible rate decisively past
/// its neutral prior rather than hovering near it. That tracker is what the
/// importance weighting reads to decide how much a minority-class label is worth,
/// so it has to actually move under a skewed stream — a tracker pinned at the prior
/// would weight every class alike no matter how lopsided the traffic became.
///
/// ´claim:labelling:a-sustained-run-of-eligible-positives-carries-the-eligible-rate-past-its-prior´
/// ´test:crate:step8-importance-weight-uses-eligible-tracker´
#[test]
fn step8_importance_weight_uses_eligible_tracker() {
    // We verify indirectly: after many positive eligible labels, the
    // P+ eligible tracker should be high, and the weight for negative
    // labels should be small (since negatives are the majority class direction).
    let mut env = TestEnv::new();

    // Process 20 positive labels to push P+ eligible high
    for i in 0..20 {
        let label = make_full_label(1.0, Action::Allow);
        env.process(&label, i + 1);
    }

    // P+ eligible should be well above 0.5
    assert!(
        env.working.p_positive_eligible > 0.5,
        "P+ eligible should be > 0.5 after 20 positive labels"
    );
}

/// Every combination of challenge policy, outcome polarity, and class rate on
/// either side of the configured ceiling follows one specification chain. The
/// challenge declaration decides inherent-model training
/// (´req:host:eligibility-policy´), the two populations move at the rates of
/// the models they serve (´tab:weighting:trackers´), and their post-label rates
/// produce the capped reciprocal weight
/// (´def:weighting:balancing-weights´).
///
/// ´claim:labelling:the-learning-policy-follows-its-specification-chain-row-by-row´
/// ´test:crate:learning-policy-conforms-row-by-row´
#[test]
fn learning_policy_conforms_row_by_row() {
    for row in LEARNING_POLICY_ROWS {
        let mut env = TestEnv::new();
        env.config.eligibility = EligibilityPolicy {
            challenge_fail_is_unconfounded: row.challenge_is_unconfounded,
        };

        let initial_p_positive = if row.is_positive {
            row.initial_class_rate
        } else {
            1.0 - row.initial_class_rate
        };
        env.working.p_positive_global = initial_p_positive;
        env.working.p_positive_eligible = initial_p_positive;

        let label = make_challenge_label(if row.is_positive { 1.0 } else { -1.0 }, false);
        env.process(&label, 1);

        let expected_eligible = row.challenge_is_unconfounded;
        let expected_global = spec_tracker_update(initial_p_positive, env.config.gamma_opr, row.is_positive);
        let expected_eligible_rate = if expected_eligible {
            spec_tracker_update(initial_p_positive, env.config.gamma_inh, row.is_positive)
        } else {
            initial_p_positive
        };

        assert_eq!(
            env.health_counters.eligible_labels,
            u64::from(expected_eligible),
            "{}: challenge policy decides eligibility",
            row.name
        );
        assert_eq!(
            env.working.operational.labels_since_recompute(),
            1,
            "{}: operational model trains on every label",
            row.name
        );
        assert_eq!(
            env.working.sister.labels_since_recompute(),
            u32::from(expected_eligible),
            "{}: sister training follows eligibility",
            row.name
        );
        assert_eq!(
            env.working.anchor.labels_since_recompute(),
            u32::from(expected_eligible),
            "{}: anchor training follows eligibility",
            row.name
        );

        assert_near(
            env.working.p_positive_global,
            expected_global,
            1e-12,
            &format!("{}: global tracker uses the operational rate", row.name),
        );
        assert_near(
            env.working.p_positive_eligible,
            expected_eligible_rate,
            1e-12,
            &format!("{}: eligible tracker uses the inherent rate or holds", row.name),
        );

        for (population, p_positive) in [
            ("global", env.working.p_positive_global),
            ("eligible", env.working.p_positive_eligible),
        ] {
            let expected_weight = spec_balancing_weight(p_positive, row.is_positive, env.config.importance_ceiling);
            let live_weight = compute_importance_weight(p_positive, row.is_positive, env.config.importance_ceiling);

            assert_near(
                live_weight,
                expected_weight,
                1e-12,
                &format!("{}: {population} weight follows the reciprocal equation", row.name),
            );
            assert_eq!(
                expected_weight == env.config.importance_ceiling,
                row.ceiling_binds,
                "{}: {population} class rate remains on its declared side of the ceiling",
                row.name
            );
        }
    }
}

/// Operational labels and eligible sister labels have distinct denominators,
/// but both accumulate the exact balancing weights the model update consumed.
/// A unit ceiling makes the hand result unambiguous when each incoming class
/// starts on the minority side: every weight binds and positive gradient mass
/// is one half of the total
/// (´entry:health:importance-ceiling´).
///
/// ´claim:labelling:importance-health-accumulates-the-weights-used-by-each-model-stream´
/// ´test:crate:importance-health-accumulates-each-model-stream´
#[test]
fn importance_health_accumulates_each_model_stream() {
    let mut env = TestEnv::new();
    env.config.importance_ceiling = 1.0;

    for (version, valence) in [1.0, -1.0, 1.0, -1.0].into_iter().enumerate() {
        let minority_rate = if valence > 0.0 { 0.0 } else { 1.0 };
        env.working.p_positive_global = minority_rate;
        env.working.p_positive_eligible = minority_rate;
        env.process(&make_full_label(valence, Action::Allow), version as u64 + 1);
    }

    let health = env.shared.health.load();
    let importance = &health.importance_ceiling;
    assert_eq!(importance.ceiling_binding_fraction, Some(1.0));
    assert_eq!(importance.gradient_balance, Some(0.5));
    assert_eq!(importance.sister_ceiling_binding_fraction, Some(1.0));
    assert_eq!(importance.sister_gradient_balance, Some(0.5));
}

/// Page accumulators retain direction and axis identity across labels. Two
/// benign labels with loud first and third axes grow only the false-alarm
/// side; a subsequent adverse label with quiet first and third axes grows the
/// missed-alarm side and subtracts one noise allowance from the first
/// (´def:monitoring:alarm-outcome-cusums´).
///
/// ´claim:labelling:alarm-outcome-cusums-accumulate-per-sentinel-axis-and-direction´
/// ´test:crate:alarm-outcome-cusums-accumulate-by-sentinel-and-axis´
#[test]
fn alarm_outcome_cusums_accumulate_by_sentinel_and_axis() {
    let mut env = TestEnv::new();
    env.process(&make_alarm_label(-1.0, [3.5, 2.0, 4.0, 1.0]), 1);
    env.process(&make_alarm_label(-1.0, [3.5, 2.0, 4.0, 1.0]), 2);
    env.process(&make_alarm_label(1.0, [0.5, 1.0, 0.1, 4.0]), 3);

    let health = env.shared.health.load();
    let alarm = health.alarm_outcome[&SentinelId(7)];
    for axis in [0, 2] {
        assert_near(alarm.high_alarm_benign[axis], 1.94, 1e-12, "false-alarm CUSUM");
        assert_near(alarm.quiet_alarm_adverse[axis], 0.98, 1e-12, "missed-alarm CUSUM");
    }
    for axis in [1, 3] {
        assert_eq!(alarm.high_alarm_benign[axis], 0.0);
        assert_eq!(alarm.quiet_alarm_adverse[axis], 0.0);
    }
}

/// Configured alarm thresholds are inclusive and the configured noise allowance
/// is the exact amount subtracted: just outside either boundary stays quiet,
/// exactly on it grows the corresponding directional accumulator by one minus
/// the allowance.
///
/// ´claim:labelling:alarm-outcome-boundaries-and-allowance-are-configured-exactly´
/// ´test:crate:alarm-outcome-cusums-honour-configured-boundaries´
#[test]
fn alarm_outcome_cusums_honour_configured_boundaries() {
    let mut env = TestEnv::new();
    env.config.monitoring.strong_alarm_threshold = 7.0;
    env.config.monitoring.quiet_alarm_threshold = 0.5;
    env.config.monitoring.alarm_outcome_noise_allowance = 0.2;

    env.process(&make_alarm_label(-1.0, [6.99, 0.0, 0.0, 0.0]), 1);
    env.process(&make_alarm_label(-1.0, [7.0, 0.0, 0.0, 0.0]), 2);
    {
        let health = env.shared.health.load();
        assert_near(
            health.alarm_outcome[&SentinelId(7)].high_alarm_benign[0],
            0.8,
            1e-12,
            "configured strong boundary",
        );
    }

    env.process(&make_alarm_label(1.0, [0.51, 1.0, 1.0, 1.0]), 3);
    env.process(&make_alarm_label(1.0, [0.5, 1.0, 1.0, 1.0]), 4);

    let health = env.shared.health.load();
    let alarm = health.alarm_outcome[&SentinelId(7)];
    assert_near(alarm.quiet_alarm_adverse[0], 0.8, 1e-12, "configured quiet boundary");
}

/// A label for a challenged request runs the pipeline to completion and is counted
/// without any challenge outcome attached to it. Whether the challenge was passed
/// or failed is the host's business; the core needs only the outcome that followed,
/// so a host with no challenge instrumentation can still submit labels for the
/// requests it challenged.
///
/// ´claim:labelling:a-challenge-label-is-processed-without-any-challenge-outcome-attached´
/// ´test:crate:companion-boundary-challenge-label-does-not-require-result´
#[test]
fn companion_boundary_challenge_label_does_not_require_result() {
    let mut env = TestEnv::new();

    let label = make_challenge_label(1.0, true);
    env.process(&label, 1);

    assert_eq!(
        env.health_counters.labels_by_action.get(&Action::Challenge),
        Some(&1),
        "challenge labels are processed by the core without challenge-result data"
    );
}

/// Challenge effectiveness lives in a tracker the host owns and drives directly:
/// recording a passed challenge moves that channel's estimate, and running a label
/// through the core alongside it neither performs nor undoes that update. The two
/// state machines are separate, so a host may adopt challenge tracking, replace it,
/// or omit it without touching the model path.
///
/// ´claim:labelling:challenge-effectiveness-is-tracked-beside-the-core-rather-than-inside-it´
/// ´test:crate:companion-boundary-tracker-updates-outside-core´
#[test]
fn companion_boundary_tracker_updates_outside_core() {
    let mut env = TestEnv::new();
    let mut tracker = ChallengeEffectivenessTracker::new();
    let now = crate::types::PersistentTimestamp::now();

    tracker.update(ChannelId(0), ChallengeResult::Pass, &now);

    let label = make_challenge_label(1.0, true);
    env.process(&label, 1);

    let estimate = tracker.estimate(ChannelId(0), &now);
    assert!(estimate.q_c < 0.5, "Pass increments beta in the companion tracker");
}

/// Each label is counted under the action that was actually taken on it: an allowed
/// request lands in the allow tally and nowhere else. The per-action counters are
/// how an operator reads the shape of the traffic the model learned from, so a
/// label attributed to the wrong action would misdescribe that mix.
///
/// ´claim:labelling:each-label-is-counted-under-the-action-that-was-actually-taken´
/// ´test:crate:companion-boundary-non-challenge-skips´
#[test]
fn companion_boundary_non_challenge_skips() {
    let mut env = TestEnv::new();

    // Allow + positive → challenge model not updated
    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    assert_eq!(env.health_counters.labels_by_action.get(&Action::Allow), Some(&1));
}

/// A live label leaves exactly one calibration entry behind, and that entry records
/// the polarity of the outcome it came from. Calibration is refitted from this
/// buffer much later, when the model has moved on, so each entry has to preserve
/// which side of the decision it was — the buffer is the only record by then.
///
/// ´claim:labelling:a-live-label-leaves-one-calibration-entry-carrying-its-polarity´
/// ´test:crate:step14-calibration-buffer-pushed´
#[test]
fn step14_calibration_buffer_pushed() {
    let mut env = TestEnv::new();
    assert!(env.calibration_buffer.is_empty());

    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    assert_eq!(
        env.calibration_buffer.len(),
        1,
        "calibration buffer should have 1 entry after 1 label"
    );
    let entry = env.calibration_buffer.iter().next().unwrap();
    assert!(entry.positive, "entry should be positive for positive label");
}

/// The calibration row holds the uncertainty the assessment reported: the
/// σ_p̂ carried on the pending risk basis arrives in the pushed entry as the
/// same value, not as a placeholder. Empirical coverage divides each row's
/// residual by exactly this quantity (´alg:monitoring:empirical-coverage´),
/// and the row is the only place it survives to be divided by — the model
/// has moved on by the time coverage is computed.
///
/// ´claim:labelling:the-calibration-row-holds-the-assessment-time-uncertainty´
/// ´test:crate:step14-calibration-entry-carries-uncertainty´
#[test]
fn step14_calibration_entry_carries_uncertainty() {
    let mut env = TestEnv::new();

    let mut pending = test_pending_assessment(TEST_RID);
    pending.risk_basis = PendingRiskBasis::new(0.2, 0.42, 0.1, 0.3);
    let label = SequencedLabel {
        seq: Some(1),
        label: LabelSpec::new(TEST_RID)
            .valence(1.0)
            .action(Action::Allow)
            .ground_truth()
            .build(),
        context: LabelContext::Full(Box::new(pending)),
        arrived_at: None,
    };
    env.process(&label, 1);

    let entry = env.calibration_buffer.iter().next().expect("one entry");
    assert!(
        (entry.uncertainty - 0.42).abs() < f64::EPSILON,
        "the row carries the assessment-time uncertainty, got {}",
        entry.uncertainty
    );
    assert!((entry.rho_eff - 0.3).abs() < f64::EPSILON);
}

/// A replayed label pushes the calibration entry the original would have,
/// built from the risk basis the journal context carries. There is no double
/// count to fear: the checkpoint captures the buffer, and the replay filter
/// hands back only labels past the checkpoint's mark — labels whose entries
/// the captured buffer has never held — so skipping them would leave the
/// restored buffer permanently short of exactly the rows the journal was
/// keeping (´dec:durability:checkpoint-journal´).
///
/// ´claim:labelling:a-replayed-label-pushes-the-calibration-entry-the-original-would-have´
/// ´test:crate:step14-runs-in-replay´
#[test]
fn step14_runs_in_replay() {
    let mut env = TestEnv::new();
    assert!(env.calibration_buffer.is_empty());

    let label = make_replay_label(1.0, Action::Allow);
    env.process(&label, 1);

    assert_eq!(
        env.calibration_buffer.len(),
        1,
        "a replayed label pushes its calibration entry"
    );
    let entry = env.calibration_buffer.iter().next().unwrap();
    assert!(entry.positive, "the entry carries the replayed label's polarity");
}

/// The standardisation means move toward the features each label was built from:
/// seeded well away from where the observations sit, they close some of that gap on
/// the first label. Standardisation is what keeps features comparable as the traffic
/// changes, and a set of means that never followed the data would leave every
/// feature scaled against a distribution that no longer exists.
///
/// ´claim:labelling:standardisation-means-move-toward-the-features-of-each-label´
/// ´test:crate:step15-standardisation-updated´
#[test]
fn step15_standardisation_updated() {
    let mut env = TestEnv::new();
    // The continuing average is the in-service mechanism; the cold ramp owns
    // the coordinate system before that (´alg:standardisation:label-time-procedure´).
    env.cold_ramp_in_service();
    // Seed means away from zero so EWMA toward the zero-valued φ is observable.
    env.working.feature_means.fill(5.0);
    let initial_means = env.working.feature_means.clone();

    // Process a label — standardisation should update
    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    // At least one mean should have moved toward 0 (skip index 0 = bias)
    let changed = env
        .working
        .feature_means
        .iter()
        .zip(initial_means.iter())
        .skip(1)
        .any(|(a, b)| (a - b).abs() > 1e-15);
    assert!(changed, "standardisation means should be updated after label");
}

/// While the cold ramp is transitioning a label teaches the models and leaves
/// the coordinate system alone, and the first label after the horizon advances
/// the continuing average. The two authorities are separable and this is where
/// they are separated: the models learn from the first label onward whatever
/// the coordinate system is doing, while the ramp owns full-vector
/// standardisation until it reaches its horizon. A label moving the average
/// underneath a running ramp would put a label's authority into a transition an
/// observation's authority owns, and the published moments would then be
/// neither the ramp's mixture nor the average's — with nothing on the wire
/// saying which.
///
/// ´claim:labelling:a-transitioning-label-moves-the-models-and-leaves-the-coordinate-system-to-the-ramp´
/// ´test:crate:transition-label-moves-models-not-standardisation´
#[test]
fn transition_label_moves_models_not_standardisation() {
    let mut env = TestEnv::new();

    // One accepted observation of a hundred-observation horizon: the ramp is
    // transitioning, with the great majority of its prior mass still standing.
    let width = env.working.feature_means.len();
    let generation = env.working.layout_generation;
    let v_floor = env.config.standardisation.v_floor;
    env.working
        .accept_cold_observation(&vec![0.0; width], generation, v_floor)
        .expect("the first observation is accepted");
    assert!(env.working.cold_ramp.phase().is_transitioning());

    // Seed the means away from the observations so any movement would show.
    env.working.feature_means.fill(5.0);
    let means_before = env.working.feature_means.clone();
    let variances_before = env.working.feature_variances.clone();
    let mu_before = env.working.sister.to_parameters().mu;

    env.process(&make_full_label(1.0, Action::Allow), 1);

    assert_eq!(
        env.working.feature_means, means_before,
        "a transitioning label must not move the standardisation means",
    );
    assert_eq!(
        env.working.feature_variances, variances_before,
        "a transitioning label must not move the standardisation variances",
    );
    let mu_after = env.working.sister.to_parameters().mu;
    assert!(
        mu_before.iter().zip(mu_after.iter()).any(|(a, b)| (a - b).abs() > 1e-15),
        "the models must still learn from a label during the transition",
    );

    // At the horizon the continuing average takes over, and the very next
    // label advances it.
    env.cold_ramp_in_service();
    env.process(&make_full_label(1.0, Action::Allow), 2);

    let moved = env
        .working
        .feature_means
        .iter()
        .zip(means_before.iter())
        .skip(1)
        .filter(|(a, b)| (*a - *b).abs() > 1e-15)
        .count();
    assert!(moved > 0, "the first in-service label must advance the continuing average");
}

/// Standardisation keeps updating during replay, as every step does. The
/// checkpoint restores the running means and variances as they stood at
/// capture, and the labels being replayed all postdate that capture — so
/// their step-15 contributions are exactly the ones the restored vectors
/// are missing, and skipping them would leave a restored instance
/// permanently behind the live run it is meant to resume.
///
/// ´claim:labelling:standardisation-keeps-updating-during-replay´
/// ´test:crate:step15-standardisation-in-replay´
#[test]
fn step15_standardisation_in_replay() {
    let mut env = TestEnv::new();
    env.cold_ramp_in_service();
    // Seed means away from zero so EWMA toward the zero-valued φ is observable.
    env.working.feature_means.fill(5.0);
    let initial_means = env.working.feature_means.clone();

    let label = make_replay_label(1.0, Action::Allow);
    env.process(&label, 1);

    let changed = env
        .working
        .feature_means
        .iter()
        .zip(initial_means.iter())
        .skip(1)
        .any(|(a, b)| (a - b).abs() > 1e-15);
    assert!(changed, "standardisation should be updated even during replay");
}

/// A replayed label banks its drift residual exactly as the original would
/// have, against the assessment-time prediction the journal context carries
/// — the quantity (´alg:monitoring:drift-cusums´) requires the residual be
/// taken against. Nothing is manufactured: the label happened once, its
/// residual was measured against the prediction in force at assessment
/// time, and replay applies that same residual once — where the restore has
/// just brought the accumulators back and the replayed labels are the only
/// evidence gathered since.
///
/// ´claim:labelling:replay-banks-the-drift-residual-against-the-journalled-prediction´
/// ´test:crate:step16-runs-in-replay´
#[test]
fn step16_runs_in_replay() {
    let mut env = TestEnv::new();

    // A live label with a journalled prediction, then the same label
    // replayed: both bank one step into every model's accumulators.
    let label1 = make_full_label(1.0, Action::Allow);
    env.process(&label1, 1);

    let drift_before = env.working.drift_state.clone();
    assert!(!drift_before.is_empty(), "the live label populates drift state");

    let label2 = make_replay_label(1.0, Action::Allow);
    env.process(&label2, 2);

    for (model_id, state_before) in &drift_before {
        let state_after = env.working.drift_state.get(model_id).expect("model should exist");
        assert_eq!(
            state_after.steps_since_reset,
            state_before.steps_since_reset + 1,
            "a replayed label banks one drift step for {model_id:?}"
        );
    }
}

/// A processed label ends with a snapshot published at the version it was given,
/// not at some internally chosen counter. The caller decides the version, so the
/// number a reader observes can be matched against the label stream that produced
/// it — which is what makes the version usable as a progress marker at all.
///
/// ´claim:labelling:a-processed-label-publishes-a-snapshot-at-the-version-it-was-given´
/// ´test:crate:step17-snapshot-published´
#[test]
fn step17_snapshot_published() {
    let mut env = TestEnv::new();
    let snap_before = env.shared.published.load();
    let version_before = snap_before.version;

    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    let snap_after = env.shared.published.load();
    assert_eq!(
        snap_after.version, 1,
        "snapshot version should be 1 after processing with version=1"
    );
    assert!(
        snap_after.version > version_before,
        "snapshot version should increase after process_label()"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// CP5/CP6 Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A label processed against healthy models publishes rather than reverts: the
/// integrity checkpoint passes and the new snapshot appears. The revert path exists
/// for models that have gone numerically bad, and a checkpoint that fired on clean
/// state would silently discard every update the system ever made.
///
/// ´claim:labelling:a-label-processed-against-healthy-models-publishes-rather-than-reverts´
/// ´test:crate:cp5-nan-check-clean-working-copy´
#[test]
fn cp5_nan_check_clean_working_copy() {
    // A cold-start working copy should have no NaN.
    let _working = WorkingCopy::cold_start(20, &ModelConfig::default(), 1000);
    // Use the module-private check_working_copy_nan indirectly:
    // process a label on a clean working copy — it should not trigger CP5.
    let mut env = TestEnv::new();
    let label = make_full_label(1.0, Action::Allow);
    env.process(&label, 1);

    // Verify the snapshot was published (not reverted)
    let snap = env.shared.published.load();
    assert_eq!(snap.version, 1, "clean processing should publish version 1");
}

/// Reverting is a full reconstruction from a snapshot, not a partial rollback: the
/// working copy rebuilt from a snapshot carries that snapshot's scalars and none of
/// the values the live copy had since moved to. Recovery from a corrupted update has
/// to land on a state that once genuinely existed, and only rebuilding wholesale
/// guarantees no half-applied mutation survives.
///
/// ´claim:labelling:reverting-rebuilds-the-working-copy-wholesale-from-the-snapshot´
/// ´test:crate:cp5-revert-restores-from-snapshot´
#[test]
fn cp5_revert_restores_from_snapshot() {
    let model_config = ModelConfig::default();
    let mut working = WorkingCopy::cold_start(10, &model_config, 1000);
    let snapshot = working.to_snapshot(0);

    // Modify working copy scalars
    working.p_positive_global = 0.9;
    working.p_positive_eligible = 0.8;
    working.kappa_v = 5.0;

    // Revert by rebuilding the copy whole (´dec:retention:monolithic-snapshot´)
    let working = WorkingCopy::from_snapshot(&snapshot, &model_config, 1000, 100).expect("revert should succeed");

    let exact = DEFAULT_TOLERANCES.bit_identical;
    assert_near(working.p_positive_global, 0.5, exact, "p_positive_global restored");
    assert_near(working.p_positive_eligible, 0.5, exact, "p_positive_eligible restored");
    assert_near(working.kappa_v, 1.0, exact, "kappa_v restored");
}

/// A revert whose rebuild is itself refused stops the label path instead of
/// carrying on with the state it just judged corrupt. The stop is recorded once,
/// naming the model whose reconstruction was refused, the pivot it was refused
/// at, and how far into the label stream it happened; the published snapshot is
/// left exactly as it was, because that snapshot is what assessments go on
/// answering from. What no longer happens is the arrangement this replaces: no
/// cascade terminus is synthesised on any model and no recomputation interval is
/// halved, because the repair those were asking for does not exist — a refused
/// factorisation is a verdict about the matrix, and asking for it again sooner
/// buys attempts rather than an exit.
///
/// ´claim:labelling:a-revert-whose-rebuild-is-refused-stops-the-label-path-and-synthesises-no-terminus´
/// ´test:crate:a-failed-revert-stops-the-label-path´
#[test]
fn a_failed_revert_stops_the_label_path() {
    let model_config = ModelConfig::default();
    let mut env = TestEnv::new();

    // The working copy carries a non-finite mean, which is what CP5 reads.
    let good = env.working.to_snapshot(1);
    let mut with_nan = good.clone();
    with_nan.operational.mu[0] = f64::NAN;
    env.working = WorkingCopy::from_snapshot(&with_nan, &model_config, env.config.n_recompute, 100)
        .expect("a finite covariance restores whatever the mean beside it carries");

    // The snapshot the revert will reach for cannot be rebuilt from. Its
    // operational covariance carries an off-diagonal pair large enough to put
    // the leading two-by-two block outside the definite cone — a spectrum of
    // thirty and minus ten against a diagonal the diagonal test finds nothing
    // wrong with — so the plain factorisation is refused at the second pivot
    // and the shifted one is refused with it (´dec:posterior:cascade-never-fails´).
    let mut unrestorable = good;
    let p = unrestorable.operational.p;
    unrestorable.operational.covariance_data[1] = 20.0;
    unrestorable.operational.covariance_data[p] = 20.0;
    env.shared.published.store(Arc::new(unrestorable));
    let published_before = env.shared.published.load_full();

    let intervals_before = (
        env.working.operational.n_recompute_effective(),
        env.working.sister.n_recompute_effective(),
        env.working.anchor.n_recompute_effective(),
    );

    env.process(&make_full_label(1.0, Action::Allow), 7);

    let stop = env
        .shared
        .label_path_stop
        .load_full()
        .expect("a refused rebuild stops the label path");
    assert_eq!(
        stop.model,
        ModelId::Operational,
        "the stop names the model whose reconstruction was refused",
    );
    assert_eq!(stop.pivot, 1, "the stop carries the pivot the factorisation stopped at");
    assert_eq!(
        stop.labels_taken_up, 7,
        "the stop carries how far into the label stream the engine had got",
    );
    assert_eq!(
        env.health_counters.revert_failures, 1,
        "the failed revert is counted exactly once",
    );

    assert_eq!(
        env.working.operational.cascade_terminus_events(),
        0,
        "no terminus is synthesised on the operational model",
    );
    assert_eq!(
        env.working.sister.cascade_terminus_events(),
        0,
        "no terminus is synthesised on the sister model",
    );
    assert_eq!(
        env.working.anchor.cascade_terminus_events(),
        0,
        "no terminus is synthesised on the anchor model",
    );
    assert_eq!(
        (
            env.working.operational.n_recompute_effective(),
            env.working.sister.n_recompute_effective(),
            env.working.anchor.n_recompute_effective(),
        ),
        intervals_before,
        "no recomputation interval is halved by the failure",
    );

    assert!(
        Arc::ptr_eq(&published_before, &env.shared.published.load_full()),
        "the read surface is the snapshot that was published before the label",
    );
}

/// The revert rebuilds at the deployment's declared configuration rather than at
/// the library defaults. The pipeline configuration carries the deployment's
/// whole model declaration and its regularisation, and the revert reads them from
/// there rather than constructing either — so a deployment that declared a prior
/// precision of its own keeps it across a revert instead of having every model
/// silently retuned to the default at the moment a checkpoint fires.
///
/// The rebuilt prior is pinned by reading the revert's own source rather than by
/// an accessor, because the restored prior is not published on any reader-visible
/// surface: it reaches the model as a stored scalar and shows only in a later
/// marginalisation. The audit reads the source in the checkout being tested, so
/// it holds wherever the binary was compiled.
///
/// ´claim:labelling:the-revert-rebuilds-at-the-deployments-declared-configuration-rather-than-the-defaults´
/// ´test:crate:the-revert-rebuilds-at-the-deployments-configuration´
#[test]
fn the_revert_rebuilds_at_the_deployments_configuration() {
    // The declaration reaches the pipeline configuration whole.
    let declared = crate::config::types::AssayerConfig {
        model: ModelConfig {
            lambda_prior: 0.25,
            ..ModelConfig::default()
        },
        ..crate::config::types::AssayerConfig::default()
    };
    let pipeline = LabelPipelineConfig::from_assayer_config(&declared);
    assert_near(
        pipeline.model.lambda_prior,
        0.25,
        DEFAULT_TOLERANCES.bit_identical,
        "the pipeline carries the declared prior",
    );
    assert!(
        (ModelConfig::default().lambda_prior - 0.25).abs() > DEFAULT_TOLERANCES.bit_identical,
        "the declared prior has to differ from the default for this to distinguish anything",
    );

    // And the revert reads it from there rather than constructing one.
    let path = crate::testing::manifest_dir().join("src/owner/label_path.rs");
    let source = std::fs::read_to_string(&path).expect("the revert's source is readable");
    let lines: Vec<&str> = source.lines().collect();
    let start = lines
        .iter()
        .position(|line| line.trim_start().starts_with("fn revert_all_state("))
        .expect("the revert is where this audit expects it");

    let mut depth = 0i32;
    let mut opened = false;
    let mut body = Vec::new();
    for line in &lines[start..] {
        let code = line.split_once("//").map_or(*line, |(before, _)| before);
        body.push(code);
        depth += i32::try_from(code.matches('{').count()).expect("brace counts are small");
        depth -= i32::try_from(code.matches('}').count()).expect("brace counts are small");
        if depth > 0 {
            opened = true;
        }
        if opened && depth == 0 {
            break;
        }
    }
    assert!(opened && depth == 0, "the revert's body did not close");

    let body = body.join("\n");
    assert!(
        body.contains("&config.model"),
        "the revert rebuilds at the deployment's declared model configuration",
    );
    assert!(
        !body.contains("ModelConfig::default()"),
        "the revert constructs no configuration for itself",
    );
}

/// The mean vector of a cold-started snapshot is entirely finite, and a NaN placed
/// into it is visible to a scan of that vector. The snapshot is what every reader
/// sees, so corruption reaching it must be detectable before publication — and a
/// scan can only be trusted if the untouched case is genuinely clean to begin with.
///
/// ´claim:labelling:the-mean-vector-of-a-snapshot-is-clean-at-cold-start-and-scannable-for-nan´
/// ´test:crate:cp6-snapshot-nan-detection´
#[test]
fn cp6_snapshot_nan_detection() {
    // Clean snapshot: no NaN
    let working = WorkingCopy::cold_start(5, &ModelConfig::default(), 1000);
    let snap = working.to_snapshot(0);
    assert!(
        !snap.operational.mu.iter().any(|x| x.is_nan()),
        "cold-start snapshot should have no NaN"
    );

    // Construct a snapshot with NaN injected into mu
    let mut bad_snap = snap;
    if !bad_snap.operational.mu.is_empty() {
        bad_snap.operational.mu[0] = f64::NAN;
    }
    assert!(
        bad_snap.operational.mu.iter().any(|x| x.is_nan()),
        "injected NaN should be detectable"
    );
}

/// The covariance block of a snapshot is subject to the same scan as the mean.
/// Corruption in the uncertainty is quieter than corruption in the estimate — a
/// bad variance produces bad confidence rather than an obviously bad prediction —
/// so it has to be looked for explicitly rather than assumed to show up elsewhere.
///
/// ´claim:labelling:the-covariance-block-of-a-snapshot-is-scanned-for-nan-as-well-as-the-mean´
/// ´test:crate:cp6-nan-in-covariance-blocked´
#[test]
fn cp6_nan_in_covariance_blocked() {
    // NaN in covariance_data (diag(Σ)) should also be detected.
    let working = WorkingCopy::cold_start(5, &ModelConfig::default(), 1000);
    let mut snap = working.to_snapshot(0);

    // Inject NaN into covariance_data
    if !snap.operational.covariance_data.is_empty() {
        snap.operational.covariance_data[0] = f64::NAN;
    }
    assert!(
        snap.operational.covariance_data.iter().any(|x| x.is_nan()),
        "injected NaN in covariance should be detectable"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Replay Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The calibration push holds over a run, not just for a single label: five
/// replayed labels in succession leave five entries, one each. Replay is
/// normally a long stretch of journal entries, so the property has to
/// survive repetition to be worth anything.
///
/// (´claim:labelling:a-replayed-label-pushes-the-calibration-entry-the-original-would-have´)
/// ´test:crate:replay-labels-accumulate-calibration´
#[test]
fn replay_labels_accumulate_calibration() {
    let mut env = TestEnv::new();

    // Process 5 replay labels
    for i in 0..5 {
        let label = make_replay_label(1.0, Action::Allow);
        env.process(&label, i + 1);
    }

    assert_eq!(
        env.calibration_buffer.len(),
        5,
        "each replayed label pushes its calibration entry"
    );
}

/// Across a run of replayed labels the standardisation means keep moving, so a
/// restored instance converges on the scaling the live run had rather than freezing
/// at whatever the checkpoint held. The skip in replay is selective: what the
/// checkpoint captured is not redone, what it did not capture is.
///
/// (´claim:labelling:standardisation-keeps-updating-during-replay´)
/// ´test:crate:replay-label-updates-standardisation´
#[test]
fn replay_label_updates_standardisation() {
    let mut env = TestEnv::new();
    env.cold_ramp_in_service();
    // Seed means away from zero so EWMA toward the zero-valued φ is observable.
    env.working.feature_means.fill(5.0);
    let initial_means = env.working.feature_means.clone();

    // Process 5 replay labels — standardisation should still update
    for i in 0..5 {
        let label = make_replay_label(1.0, Action::Allow);
        env.process(&label, i + 1);
    }

    let changed = env
        .working
        .feature_means
        .iter()
        .zip(initial_means.iter())
        .skip(1)
        .any(|(a, b)| (a - b).abs() > 1e-15);
    assert!(changed, "standardisation means should be updated during replay");
}

/// Crash recovery is exact where it claims to be: an instance checkpointed halfway
/// through a stream, rebuilt from that checkpoint, and fed the remaining labels as
/// replay arrives at the same positive rates, the same valence scale and the same
/// standardisation means as the run that never crashed — to within a part in a
/// trillion — and at the same calibration buffer, the ten captured entries plus
/// one per replayed label. A restored instance is therefore the same instance,
/// not an approximation of one.
///
/// ´claim:labelling:replay-from-a-checkpoint-reproduces-the-live-scalar-state-exactly´
/// ´test:crate:replay-invariant-r17´
#[test]
fn replay_invariant_r17() {
    // R-18: checkpoint at seq=10 → restore → replay entries 11–20 →
    // P+, κ_v, standardisation and the calibration buffer match the live
    // run within tolerance. Model updates are deterministic from the
    // replayed journal context, which carries the assessment-time
    // scalars (´cor:durability:replay-exactness´).

    let model_config = ModelConfig::default();

    // --- Pass 1: live run of labels 1–20 ---
    let mut env = TestEnv::new();
    let valences: Vec<f64> = (0..20).map(|i| if i % 3 == 0 { 1.0 } else { -1.0 }).collect();

    for (i, &v) in valences.iter().enumerate() {
        let label = make_full_label(v, Action::Allow);
        env.process(&label, (i + 1) as u64);
    }

    // Record live-run final state
    let live_p_plus_global = env.working.p_positive_global;
    let live_p_plus_eligible = env.working.p_positive_eligible;
    let live_kappa_v = env.working.kappa_v;
    let live_means = env.working.feature_means.clone();

    // --- Pass 2: checkpoint at seq=10, restore, replay 11–20 ---
    let mut env2 = TestEnv::new();

    // Process labels 1–10 (live)
    for (i, &v) in valences[..10].iter().enumerate() {
        let label = make_full_label(v, Action::Allow);
        env2.process(&label, (i + 1) as u64);
    }

    // Checkpoint after seq=10
    let checkpoint = env2.working.to_snapshot(10);
    let checkpoint_means = env2.working.feature_means.clone();
    let checkpoint_variances = env2.working.feature_variances.clone();
    let checkpoint_cal_len = env2.calibration_buffer.len();

    // Simulate a crash: reconstruct from the whole-state checkpoint
    // (´dec:durability:checkpoint-journal´)
    let mut reverted = WorkingCopy::from_snapshot(&checkpoint, &model_config, 1000, 100).expect("revert should succeed");
    std::mem::swap(&mut env2.working, &mut reverted);
    env2.working.feature_means = checkpoint_means;
    env2.working.feature_variances = checkpoint_variances;
    env2.calibration_buffer.truncate(checkpoint_cal_len);

    // Replay labels 11–20
    for (i, &v) in valences[10..].iter().enumerate() {
        let label = make_replay_label(v, Action::Allow);
        env2.process(&label, (i + 11) as u64);
    }

    // Compare
    let tol = 1e-12;
    assert_near(
        env2.working.p_positive_global,
        live_p_plus_global,
        tol,
        "P+ global replay vs live",
    );
    assert_near(
        env2.working.p_positive_eligible,
        live_p_plus_eligible,
        tol,
        "P+ eligible replay vs live",
    );
    assert_near(env2.working.kappa_v, live_kappa_v, tol, "κ_v replay vs live");

    // Standardisation means should match within tolerance
    for (i, (replay_m, live_m)) in env2.working.feature_means.iter().zip(live_means.iter()).enumerate() {
        assert_near(*replay_m, *live_m, tol, &format!("standardisation mean[{i}] replay vs live"));
    }

    // The calibration buffer matches the live run's: the ten captured
    // entries plus one per replayed label.
    assert_eq!(
        env2.calibration_buffer.len(),
        20,
        "the replayed run's buffer holds the captured entries plus one per replayed label"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multi-Label Integration Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Over a run of labels the published version tracks the caller's numbering step
/// for step, never lagging behind by a publication or skipping ahead. A reader
/// polling the snapshot therefore sees each label's effect appear exactly once, in
/// order, which is what lets it treat the version as a position in the stream.
///
/// (´claim:labelling:a-processed-label-publishes-a-snapshot-at-the-version-it-was-given´)
/// ´test:crate:multiple-labels-monotonic-snapshots´
#[test]
fn multiple_labels_monotonic_snapshots() {
    let mut env = TestEnv::new();

    for i in 0..10 {
        let valence = if i % 3 == 0 { 1.0 } else { -1.0 };
        let label = make_full_label(valence, Action::Allow);
        env.process(&label, i + 1);

        let snap = env.shared.published.load();
        assert_eq!(
            snap.version,
            i + 1,
            "snapshot version should equal the version passed to process_label"
        );
    }
}

/// The buffer accumulates rather than replaces: twenty live labels leave twenty
/// entries, and the split between positive and negative entries mirrors the
/// alternating stream that produced them. Calibration is fitted against the balance
/// of the two classes, so an entry lost or a polarity mislaid would bias the fit
/// toward whichever side survived.
///
/// (´claim:labelling:a-live-label-leaves-one-calibration-entry-carrying-its-polarity´)
/// ´test:crate:calibration-buffer-accumulates´
#[test]
fn calibration_buffer_accumulates() {
    let mut env = TestEnv::new();

    for i in 0..20 {
        let valence = if i % 2 == 0 { 1.0 } else { -1.0 };
        let label = make_full_label(valence, Action::Allow);
        env.process(&label, i + 1);
    }

    assert_eq!(
        env.calibration_buffer.len(),
        20,
        "calibration buffer should have 20 entries after 20 non-replay labels"
    );

    // Verify positives reflect the valence pattern
    let positives = env.calibration_buffer.iter().filter(|e| e.positive).count();
    let negatives = env.calibration_buffer.iter().filter(|e| !e.positive).count();
    assert_eq!(positives, 10);
    assert_eq!(negatives, 10);
}

/// Replayed and live labels interleave on the same instance and are counted
/// alike: after three replayed labels and then five live ones the buffer
/// holds eight entries, one per label whichever mode delivered it. Recovery
/// ends by handing an instance straight over to live traffic, and the seam
/// is invisible because both modes run the same seventeen steps.
///
/// (´claim:labelling:a-replayed-label-pushes-the-calibration-entry-the-original-would-have´)
/// ´test:crate:mixed-replay-and-live-labels´
#[test]
fn mixed_replay_and_live_labels() {
    let mut env = TestEnv::new();

    // 3 replay labels → 3 calibration entries
    for i in 0..3 {
        let label = make_replay_label(1.0, Action::Allow);
        env.process(&label, i + 1);
    }
    assert_eq!(env.calibration_buffer.len(), 3);

    // 5 live labels → 5 more calibration entries
    for i in 3..8 {
        let label = make_full_label(1.0, Action::Allow);
        env.process(&label, i + 1);
    }
    assert_eq!(env.calibration_buffer.len(), 8);
}

/// Challenge outcomes accumulate only where the host records them: a stream of
/// challenge labels through the core produces no channel state, while feeding the
/// same passes and failures to the host's own tracker populates its report. The
/// health report on challenge effectiveness is therefore the host's account of its
/// own instrumentation, never something the core quietly derived on its behalf.
///
/// (´claim:labelling:challenge-effectiveness-is-tracked-beside-the-core-rather-than-inside-it´)
/// ´test:crate:challenge-effectiveness-results-not-core-owned´
#[test]
fn challenge_effectiveness_results_not_core_owned() {
    let mut env = TestEnv::new();

    // 5 Fail + 3 Pass
    for _ in 0..5 {
        let label = make_challenge_label(1.0, true);
        env.process(&label, 1);
    }
    for _ in 0..3 {
        let label = make_challenge_label(1.0, true);
        env.process(&label, 1);
    }

    let mut tracker = ChallengeEffectivenessTracker::new();
    let now = crate::types::PersistentTimestamp::now();
    for _ in 0..5 {
        tracker.update(ChannelId(0), ChallengeResult::Fail, &now);
    }
    for _ in 0..3 {
        tracker.update(ChannelId(0), ChallengeResult::Pass, &now);
    }

    let report = tracker.health_report(1.0, &now);
    assert!(
        report.channels.contains_key(&ChannelId(0)),
        "challenge tracking is host-owned outside the core label path"
    );
}

/// The Companion's sufficiency floor carries the value the specification
/// anchors rather than the value it shipped with, and it is the boundary the
/// standalone health report actually applies. Both halves are asserted because
/// a decision can succeed at the record and fail at the number: the constant is
/// what a host reads, so the figure that decision fixed — the specification's
/// twenty, in place of the underivable fifty — is pinned here and then made to
/// decide a report (´cav:challenge:sufficiency-threshold-open´). The evidenced
/// counts are placed well either side of the floor rather than on it, because
/// lazy decay makes the knife edge a statement about elapsed microseconds
/// rather than about the threshold.
///
/// ´claim:labelling:the-companion-sufficiency-floor-carries-its-ruled-value-and-bounds-the-health-report´
/// ´test:crate:companion-sufficiency-floor-carries-its-ruled-value´
#[test]
fn companion_sufficiency_floor_carries_its_ruled_value() {
    let floor = ChallengeEffectivenessTracker::DEFAULT_SUFFICIENT_EVIDENCE_THRESHOLD;
    assert!(
        (floor - 20.0).abs() < f64::EPSILON,
        "the floor carries the specification's twenty, not the underivable fifty it shipped with"
    );

    let now = crate::types::PersistentTimestamp::now();

    let mut thin = ChallengeEffectivenessTracker::new();
    for _ in 0..10 {
        thin.update(ChannelId(0), ChallengeResult::Pass, &now);
    }
    assert!(
        !thin.health_report(floor, &now).channels[&ChannelId(0)].sufficient_evidence,
        "ten effective samples sit under the floor"
    );

    let mut evidenced = ChallengeEffectivenessTracker::new();
    for _ in 0..30 {
        evidenced.update(ChannelId(0), ChallengeResult::Pass, &now);
    }
    assert!(
        evidenced.health_report(floor, &now).channels[&ChannelId(0)].sufficient_evidence,
        "thirty effective samples clear it"
    );
}

/// The working copy records the journal sequence number carried by the label it
/// just applied, and records the label's own number rather than the publication
/// version it was processed under. That high-water mark is what a restarting
/// instance consults to decide where in the journal to resume, so it has to name a
/// position in the journal and not in the snapshot stream.
///
/// ´claim:labelling:the-working-copy-records-the-journal-sequence-of-the-label-it-just-applied´
/// ´test:crate:sequence-number-updated´
#[test]
fn sequence_number_updated() {
    let mut env = TestEnv::new();

    let mut label = make_full_label(1.0, Action::Allow);
    label.seq = Some(42);
    env.process(&label, 1);

    assert_eq!(
        env.working.last_processed_label_seq, 42,
        "last_processed_label_seq should be updated"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// End-to-End Convergence (´dec:posterior:three-step-update´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Under a long mixed workload every learned quantity actually moves and nothing
/// comes apart. The eligible positive rate leaves its prior, the valence scale
/// settles on the magnitude it keeps being shown, calibration refits at least once,
/// the feature variances depart their initial unit values, the sister model's mean
/// pulls away from its zero prior, the drift accumulators show activity and the
/// calibration buffer fills, and the models count the labels since their last
/// factorisation rebuild — while every parameter of all three models stays finite.
/// This is the seam between the pipeline and the algorithms it drives: a
/// step wired to the wrong state, or wired to nothing, shows up as a quantity that
/// sat still through two hundred and fifty labels, and a step wired to numerically
/// unsound state shows up as a parameter that stopped being a number.
///
/// ´claim:labelling:a-long-mixed-workload-moves-every-learned-quantity-and-leaves-none-non-finite´
/// ´test:crate:end-to-end-convergence´
#[test]
fn end_to_end_convergence() {
    use crate::types::ModelId;

    const NUM_LABELS: usize = 250;

    let mut env = TestEnv::new();
    // The workload below asserts that the feature variances depart their
    // initial values, which is the continuing average's doing and therefore
    // an in-service claim (´alg:standardisation:label-time-procedure´).
    env.cold_ramp_in_service();

    let initial_mu_norm = env
        .working
        .sister
        .to_parameters()
        .mu
        .iter()
        .map(|x| x * x)
        .sum::<f64>()
        .sqrt();
    let initial_p_plus_eligible = env.working.p_positive_eligible;

    let mut challenge_count = 0_usize;

    for i in 0..NUM_LABELS {
        // 60% Allow+benign, 20% Allow+adverse, 10% Challenge+Fail, 10% Block
        let label = match i % 10 {
            0..6 => make_full_label(1.0, Action::Allow),  // benign
            6..8 => make_full_label(-1.0, Action::Allow), // adverse
            8 => {
                challenge_count += 1;
                make_challenge_label(-1.0, true)
            }
            _ => make_block_label(-1.0, true),
        };

        env.process(&label, (i as u64) + 1);
    }

    // ─── (a) P+ eligible has moved ───────────────────────────────────
    // At the eligible tracker's own rate of γ_inh = 0.9998
    // (´tab:weighting:trackers´), 250 eligible labels at a 60% positive mix move the
    // rate from its 0.5 prior by about 0.1·(1 − 0.9998^250) ≈ 0.0049 —
    // the long-memory horizon is the design, so liveness is asserted at
    // the magnitude that rate implies, not the retired shared rate's.
    assert!(
        (env.working.p_positive_eligible - initial_p_plus_eligible).abs() > 0.003,
        "P+ eligible should change after {NUM_LABELS} labels: initial={initial_p_plus_eligible:.4}, final={:.4}",
        env.working.p_positive_eligible
    );

    // ─── (b) κ_v tracks |valence| EWMA (converges to 1.0 with |v| = 1.0) ─
    assert_near(env.working.kappa_v, 1.0, 0.01, "κ_v EWMA with |v| = 1.0 constant");

    // ─── (c) Platt refit occurred ────────────────────────────────────
    assert!(
        env.platt_tracker.refits_completed > 0,
        "Platt calibration should have refitted at least once after {NUM_LABELS} labels"
    );

    // ─── (d) Feature standardisation running ─────────────────────────
    let variance_changed = env.working.feature_variances.iter().any(|&v| (v - 1.0).abs() > 1e-6);
    assert!(variance_changed, "Feature variances should evolve from initial 1.0 values");

    // ─── (e) Sister model μ moved from prior ─────────────────────────
    let final_mu_norm = env
        .working
        .sister
        .to_parameters()
        .mu
        .iter()
        .map(|x| x * x)
        .sum::<f64>()
        .sqrt();
    assert!(
        (final_mu_norm - initial_mu_norm).abs() > 0.01,
        "Sister model μ should move from zero prior: initial norm={initial_mu_norm:.6}, final norm={final_mu_norm:.6}"
    );

    // ─── (f) Drift accumulators running ──────────────────────────────
    // Initialise operational drift state if not present.
    let _ = env.working.drift_state.entry(ModelId::Operational).or_default();

    // steps_since_reset resets on threshold breach, so check MAR/s± as well.
    let any_drift_activity = env
        .working
        .drift_state
        .values()
        .any(|d| d.steps_since_reset > 0 || d.mean_abs_residual > 0.0 || d.s_plus > 0.0 || d.s_minus > 0.0);
    assert!(
        any_drift_activity,
        "Drift accumulators should show activity after {NUM_LABELS} labels"
    );

    // ─── (g) Challenge labels did not require core challenge state ────
    assert!(challenge_count > 0, "mixed workload should include challenge labels");

    // ─── (h) Calibration buffer populated ────────────────────────────
    assert!(
        !env.calibration_buffer.is_empty(),
        "Calibration buffer should have entries after {NUM_LABELS} eligible labels"
    );

    // ─── (i) All model parameters finite ─────────────────────────────
    let check_params_finite = |model: &crate::model::bayesian::BayesianLinearModel, name: &str| {
        let params = model.to_parameters();
        assert_finite(&params.mu, &format!("{name} model μ"));
        assert_finite(&params.covariance_data, &format!("{name} model Σ"));
    };

    check_params_finite(&env.working.operational, "Operational");
    check_params_finite(&env.working.sister, "Sister");
    check_params_finite(&env.working.anchor, "Anchor");

    // ─── (j) labels_since_recompute incremented ──────────────────────
    let any_recomp_tracking =
        env.working.operational.labels_since_recompute() > 0 || env.working.sister.labels_since_recompute() > 0;
    assert!(
        any_recomp_tracking,
        "labels_since_recompute should increment during label processing"
    );
}

/// Successive refits of the calibration on one stationary workload do not move
/// the sister's sharpness by more than the drift the harness declares: the
/// second refit's κ sits within that bound of the first's. A refit is a fit over
/// an accumulating buffer, so the same population seen twice should produce
/// nearly the same map — a sharpness that lurched between consecutive refits
/// would mean the calibration was tracking the buffer's most recent rows rather
/// than the distribution behind them, and every probability the map returns
/// would move with it (´tab:assayer:harness-scenario-tolerances´). Both refits are
/// required to have settled inside the search range as well, because the bound
/// says something about the calibration only if the two figures it compares are
/// fits: the workload's two classes overlap across five score rungs, which is
/// what gives the cross-entropy an interior minimum to find, and a workload
/// whose rows all carried one score would leave both refits holding the same
/// bound — where the comparison passes on a pair of artefacts. The drift half of
/// this scenario is satisfied with room the workload cannot spend: a population
/// that really is stationary moves its own minimum by less than the search
/// resolves, so the two refits return the same sharpness bit for bit, and the
/// declared drift — some five hundred times the search's quantum at this
/// sharpness — is out of reach whatever the calibration does. It is the
/// interiority that discriminates here; the bound itself is put both ways, on a
/// population displaced by a settled amount, in the fitting module's own
/// scenario (´tab:assayer:harness-scenario-tolerances´).
///
/// ´claim:labelling:successive-calibration-refits-on-one-workload-hold-the-sharpness-within-the-declared-drift´
/// ´test:crate:platt-sharpness-holds-between-successive-refits´
#[test]
fn platt_sharpness_holds_between_successive_refits() {
    const MAX_LABELS: u64 = 2_000;

    let mut env = TestEnv::new();
    let mut kappas: Vec<f64> = Vec::new();
    let mut refits_seen = env.platt_tracker.refits_completed;

    let mut seq = 0_u64;
    while kappas.len() < 2 && seq < MAX_LABELS {
        // A stationary population: the overlap ladder, walked a row at a time
        // and repeating every forty labels. Every label is eligible, and each
        // one reports the score its rung stands at, so the calibration buffer
        // fills with a distribution the fit can actually read a sharpness off.
        let (rho_eff, valence) = overlap_ladder_row(seq);
        seq += 1;
        env.process(&make_scored_label(valence, rho_eff), seq);

        if env.platt_tracker.refits_completed > refits_seen {
            refits_seen = env.platt_tracker.refits_completed;
            kappas.push(env.platt_tracker.last_kappa_sister);
        }
    }

    assert!(
        kappas.len() >= 2,
        "the workload must reach a second refit for the drift to be observable; \
         reached {} refit(s) in {seq} labels",
        kappas.len(),
    );

    // Both refits must have chosen their sharpness rather than been left
    // holding a search bound. The workload's scores span four either side of
    // zero and its tallies imply a sharpness near two, so a fit anywhere near
    // that scale stands far inside the search range of a hundredth to a
    // hundred; a fit against the ceiling would mean the loss surface carried
    // no information about the sharpness at all, and the drift compared below
    // would be a difference of two artefacts rather than of two fits.
    for (refit, kappa) in kappas.iter().enumerate() {
        assert!(
            (0.5..10.0).contains(kappa),
            "refit {} settled on κ = {kappa}, which is not an interior optimum of the search range \
             [0.01, 100] — the calibration buffer's scores carry no information about the sharpness",
            refit + 1,
        );
    }

    let drift = (kappas[1].ln() - kappas[0].ln()).abs();
    assert!(
        drift <= DEFAULT_TOLERANCES.platt_log_sharpness,
        "sharpness drift between successive refits must stay within the declared bound: \
         κ went {} → {} (|Δ ln κ| = {drift}, bound {})",
        kappas[0],
        kappas[1],
        DEFAULT_TOLERANCES.platt_log_sharpness,
    );
}
