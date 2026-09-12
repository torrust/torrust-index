// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{
    DIVERGENT_OUTCOME_REPLAY_DRIFT, GOLDEN_COORD, LIFECYCLE_SUFFICIENCY_DRIFT, PURITY_DRIFT, Scenario, World,
    assert_core_and_resonance_profile_near_with_drift, assert_health_clean, attach_golden_reporting_sentinel, cycle_request,
    refresh_golden_report, scenario_with, score_verified_request_schema, with_degraded_score_verified,
    with_standard_final_score_verified,
};
use torrust_assayer::types::{Action, ChallengeResult, OutcomeAxisId, SentinelId};
use torrust_assayer::{DerivedReckoning, RequestContext, RewardParameters, Tag};

use super::assertions::CorePayloadExpectation;
use super::derivation::{
    ActionLayerMovePair, CHALLENGE_REWARD_POLICY_MATRIX, ChannelDerivationCase, DECISION_POLICY_MATRIX, DerivationMatrix,
    DerivationMatrixRequestOrder, DerivationReplayComparison, FinalSignalFixture, STANDARD_DERIVATION_MATRIX,
    WorldDerivationBatch, challenge_reward_policy_matrix_scenario, challenge_reward_policy_matrix_scenario_with,
    channel_pair_cases, decision_policy_matrix_scenario, decision_policy_matrix_scenario_with,
    multi_channel_independence_scenario, multi_channel_independence_scenario_with,
    register_distinct_golden_reporting_sentinel_states, register_distinct_live_sentinel_states,
    train_challenge_reward_policy_matrix_fail_evidence_pair_with_requests,
    train_challenge_reward_policy_matrix_fail_evidence_with_requests, train_channel_challenge_fail_evidence_with_requests,
    train_channel_challenge_fail_vs_absent_evidence_with_requests,
};
use super::labels::{
    LabelCycleCase, adverse_challenge_result, alternating_entity_label_stream, alternating_label_stream_for_entity,
    cycle_label_case_pair, cycle_reported_label_case_stream, cycle_request_pair,
};
use super::outcomes::{
    DivergentOutcomeAxisPair, train_standard_divergent_outcome_prediction_pair_with_requests,
    train_standard_live_outcome_prediction_pair_with_requests, train_standard_live_outcome_prediction_with_requests,
};
use super::policy::policy_with_actions;
use super::rewards::reward_perturbation_scenario_with;

pub type EnrichedMatrixWorldBuilder = fn(&Enrichment, &str, u64) -> Scenario;
type EnrichedMatrixPairTrainer = fn(&Enrichment, &World, &World, &str);

/// Which request-signal fixture to attach to final assessments.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SignalMode {
    /// No request-signal schema or final signal payload.
    None,
    /// Valid `score` and `verified` request signals.
    Valid,
    /// Intentionally degraded request signals.
    Degraded,
}

/// One point in the public-request enrichment grid used by the boundary tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[derive(Debug, Clone, Copy)]
pub struct Enrichment {
    /// Human-readable label appended to sub-test names.
    pub label: &'static str,
    /// Final request-signal mode.
    pub signal: SignalMode,
    /// Whether final requests route through a live reported Sentinel.
    pub reported: bool,
    /// Whether setup trains a live outcome-prediction payload.
    pub live_outcome: bool,
    /// Per-variant seed salt.
    pub seed: u64,
}

/// Full public-request enrichment grid used by the boundary integration tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub static ALL_ENRICHMENTS: [Enrichment; 12] = [
    Enrichment {
        label: "plain",
        signal: SignalMode::None,
        reported: false,
        live_outcome: false,
        seed: 0x0000_0000,
    },
    Enrichment {
        label: "signal-rich",
        signal: SignalMode::Valid,
        reported: false,
        live_outcome: false,
        seed: 0x519A_0001,
    },
    Enrichment {
        label: "degraded-signal",
        signal: SignalMode::Degraded,
        reported: false,
        live_outcome: false,
        seed: 0xDE67_0002,
    },
    Enrichment {
        label: "reported",
        signal: SignalMode::None,
        reported: true,
        live_outcome: false,
        seed: 0x5071_0003,
    },
    Enrichment {
        label: "reported-signal-rich",
        signal: SignalMode::Valid,
        reported: true,
        live_outcome: false,
        seed: 0x519A_5004,
    },
    Enrichment {
        label: "reported-degraded-signal",
        signal: SignalMode::Degraded,
        reported: true,
        live_outcome: false,
        seed: 0xDE67_5005,
    },
    Enrichment {
        label: "outcome",
        signal: SignalMode::None,
        reported: false,
        live_outcome: true,
        seed: 0x0A74_0006,
    },
    Enrichment {
        label: "signal-rich-outcome",
        signal: SignalMode::Valid,
        reported: false,
        live_outcome: true,
        seed: 0x519A_7407,
    },
    Enrichment {
        label: "degraded-signal-outcome",
        signal: SignalMode::Degraded,
        reported: false,
        live_outcome: true,
        seed: 0xDE67_7408,
    },
    Enrichment {
        label: "reported-outcome",
        signal: SignalMode::None,
        reported: true,
        live_outcome: true,
        seed: 0x5074_0009,
    },
    Enrichment {
        label: "reported-signal-rich-outcome",
        signal: SignalMode::Valid,
        reported: true,
        live_outcome: true,
        seed: 0x519A_740A,
    },
    Enrichment {
        label: "reported-degraded-outcome",
        signal: SignalMode::Degraded,
        reported: true,
        live_outcome: true,
        seed: 0xDE67_740B,
    },
];

/// Enrichment variants that carry live outcome predictions.
#[must_use]
pub fn enrichments_with_outcome() -> &'static [Enrichment] {
    &ALL_ENRICHMENTS[6..]
}

/// Run a test body over the full enrichment grid
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[track_caller]
pub fn run_enrichment_grid<F>(label_prefix: &str, seed_salt: u64, mut run: F)
where
    F: FnMut(&Enrichment, &str, u64),
{
    for enrichment in &ALL_ENRICHMENTS {
        let label = format!("{label_prefix} [{}]", enrichment.label);
        run(enrichment, &label, seed_salt ^ enrichment.seed);
    }
}

/// Run a test body over the outcome-bearing half of the enrichment grid
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´).
#[track_caller]
pub fn run_outcome_enrichment_grid<F>(label_prefix: &str, seed_salt: u64, mut run: F)
where
    F: FnMut(&Enrichment, &str, u64),
{
    for enrichment in enrichments_with_outcome() {
        let label = format!("{label_prefix} [{}]", enrichment.label);
        run(enrichment, &label, seed_salt ^ enrichment.seed);
    }
}

/// No-op decision-layer training hook for enriched matrix family runners.
pub const fn train_no_enriched_decision_evidence(_: &World, _: &Enrichment, _: &EnrichedSetup, _: &str) {}

/// Train the challenge/reward matrix's canonical channel-local evidence.
#[track_caller]
pub fn train_enriched_challenge_reward_decision_evidence(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    label: &str,
) {
    train_enriched_matrix_fail_evidence(world, enrichment, setup, 8, label);
    enrichment.refresh_if_reported(world);
}

/// Run an enriched same-observation derivation matrix over the full grid.
#[track_caller]
pub fn run_enriched_derivation_matrix_family(
    label_prefix: &str,
    seed_salt: u64,
    build_world: EnrichedMatrixWorldBuilder,
    history: EnrichedCoreHistory,
    train_decision_layer: EnrichedMatrixDecisionTraining,
    matrix: DerivationMatrix<'_>,
) {
    run_enrichment_grid(label_prefix, seed_salt, |enrichment, label, seed| {
        let mut world = build_world(enrichment, label, seed);
        let setup = history.prepare(&mut world, enrichment, label);
        train_decision_layer(&world, enrichment, &setup, label);

        assert_enriched_derivation_matrix(&world, enrichment, &setup, history.entity(), matrix, label);
        enrichment.assert_health(&world);
    });
}

/// Run an enriched request-order derivation matrix over the full grid.
#[track_caller]
pub fn run_enriched_derivation_matrix_request_order_family(
    label_prefix: &str,
    seed_salt: u64,
    build_world: EnrichedMatrixWorldBuilder,
    history: EnrichedCoreHistory,
    train_decision_layer: EnrichedMatrixDecisionTraining,
    matrix: DerivationMatrixRequestOrder<'_>,
) {
    run_enrichment_grid(label_prefix, seed_salt, |enrichment, label, seed| {
        let mut world = build_world(enrichment, label, seed);
        let setup = history.prepare(&mut world, enrichment, label);
        train_decision_layer(&world, enrichment, &setup, label);

        assert_enriched_derivation_matrix_request_order(&world, enrichment, &setup, history.entity(), matrix, label);
        enrichment.assert_health(&world);
    });
}

/// World-local state registered by [`Enrichment::setup_world`].
#[derive(Debug, Clone, Copy, Default)]
pub struct EnrichedSetup {
    /// Live reported Sentinel id, when the enrichment uses reported requests.
    pub sentinel: Option<SentinelId>,
    /// Live outcome-axis id, when the enrichment trains outcome predictions.
    pub axis: Option<OutcomeAxisId>,
}

/// Two enriched same-action channels used by the pair probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[derive(Clone, Copy, Debug)]
pub struct EnrichedChannelPair<'a> {
    /// Left channel name. In challenge-evidence helpers this is the evidence-fed channel.
    pub left: &'static str,
    /// Right channel name. In challenge-evidence helpers this is the comparison channel.
    pub right: &'static str,
    /// Expected action tags for both channels, in policy order.
    pub action_tags: &'a [Tag],
}

/// Final assertion shape for an enriched same-action channel-pair family.
#[derive(Clone, Copy, Debug)]
pub enum EnrichedChannelPairAssertion {
    /// The two channels must share the core assessment while their action layer moves.
    ActionOnly,
    /// Forward and reversed request batches must replay each channel by name.
    RequestOrder,
    /// The two channels must replay the full core assessment and resonance profile.
    CoreAndResonanceProfile,
    /// Forward and reversed request batches must replay, with each batch's aliases sharing the full profile.
    CoreAndResonanceProfileRequestOrder,
}

/// Core-side history used before an enriched derivation matrix is replayed
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´).
#[derive(Debug, Clone, Copy)]
pub enum EnrichedCoreHistory {
    /// Fresh world: only the enrichment-specific Sentinel/outcome setup runs.
    Fresh {
        /// Channel used for enrichment setup such as outcome-prediction training.
        channel: &'static str,
        /// Entity used for enrichment setup and final matrix replay.
        entity: &'static str,
    },
    /// Alternating label stream moves the core before final matrix replay.
    LabelHistory {
        /// Channel that receives the training labels.
        channel: &'static str,
        /// Entity used for enrichment setup and final matrix replay.
        entity: &'static str,
        /// Number of alternating label cycles.
        cycles: usize,
        /// Every Nth label is adverse.
        adverse_every: usize,
    },
    /// Identity dimension plus alternating label stream moves the core.
    IdentityHistory {
        /// Identity dimension registered before training.
        identity_dimension: &'static str,
        /// Channel that receives the training labels.
        channel: &'static str,
        /// Entity used for enrichment setup and final matrix replay.
        entity: &'static str,
        /// Number of alternating label cycles.
        cycles: usize,
        /// Every Nth label is adverse.
        adverse_every: usize,
    },
}

/// One named core-history precondition used by the enriched family runners
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´).
#[derive(Debug, Clone, Copy)]
pub struct EnrichedHistoryCase {
    /// Label prefix passed to the enrichment-grid runner.
    pub label_prefix: &'static str,
    /// Stable seed salt for this history family.
    pub seed_salt: u64,
    /// Core-side history prepared before final replay.
    pub history: EnrichedCoreHistory,
}

impl EnrichedHistoryCase {
    /// Build a named enriched core-history case.
    #[must_use]
    pub const fn new(label_prefix: &'static str, seed_salt: u64, history: EnrichedCoreHistory) -> Self {
        Self {
            label_prefix,
            seed_salt,
            history,
        }
    }
}

impl EnrichedCoreHistory {
    /// Fresh enriched setup for a final derivation matrix.
    #[must_use]
    pub const fn fresh(channel: &'static str, entity: &'static str) -> Self {
        Self::Fresh { channel, entity }
    }

    /// Label-history enriched setup for a final derivation matrix.
    #[must_use]
    pub const fn label_history(channel: &'static str, entity: &'static str, cycles: usize, adverse_every: usize) -> Self {
        Self::LabelHistory {
            channel,
            entity,
            cycles,
            adverse_every,
        }
    }

    /// Identity-history enriched setup for a final derivation matrix.
    #[must_use]
    pub const fn identity_history(
        identity_dimension: &'static str,
        channel: &'static str,
        entity: &'static str,
        cycles: usize,
        adverse_every: usize,
    ) -> Self {
        Self::IdentityHistory {
            identity_dimension,
            channel,
            entity,
            cycles,
            adverse_every,
        }
    }

    /// Entity used by the final matrix replay.
    #[must_use]
    pub const fn entity(self) -> &'static str {
        match self {
            Self::Fresh { entity, .. } | Self::LabelHistory { entity, .. } | Self::IdentityHistory { entity, .. } => entity,
        }
    }

    /// Prepare the world according to this core-history fixture.
    #[track_caller]
    pub fn prepare(self, world: &mut Scenario, enrichment: &Enrichment, label: &str) -> EnrichedSetup {
        match self {
            Self::Fresh { channel, entity } => enrichment.setup_world(world, channel, entity),
            Self::LabelHistory {
                channel,
                entity,
                cycles,
                adverse_every,
            } => setup_enriched_label_history(world, enrichment, channel, entity, cycles, adverse_every, label),
            Self::IdentityHistory {
                identity_dimension,
                channel,
                entity,
                cycles,
                adverse_every,
            } => setup_enriched_identity_history(
                world,
                enrichment,
                identity_dimension,
                channel,
                entity,
                cycles,
                adverse_every,
                label,
            ),
        }
    }
}

/// Optional decision-layer training between core-history setup and replay.
pub type EnrichedMatrixDecisionTraining = fn(&World, &Enrichment, &EnrichedSetup, &str);

/// One enriched derivation-matrix fixture used by the cross-world probes
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´).
///
/// Divergent-outcome and lifecycle-sufficiency tests share the same ceremony:
/// build a pair of worlds, prepare the core payload, optionally train extra
/// decision-layer evidence, and finally replay a derivation matrix. Keeping the
/// matrix metadata and its setup hooks together lets new matrix shapes reuse
/// that ceremony without open-coding another runner.
#[derive(Clone, Copy)]
struct EnrichedMatrixFixture<'a> {
    /// Builds one world for this matrix shape.
    build_world: EnrichedMatrixWorldBuilder,
    /// Matrix asserted at the final derivation point.
    matrix: DerivationMatrix<'a>,
    /// Channel used for core-side outcome/lifecycle training.
    training_channel: &'static str,
    /// Optional extra matched-world decision-layer training before replay.
    train_pair_evidence: EnrichedMatrixPairTrainer,
}

impl EnrichedMatrixFixture<'_> {
    #[track_caller]
    fn build_world(self, enrichment: &Enrichment, label: &str, seed: u64) -> Scenario {
        (self.build_world)(enrichment, label, seed)
    }
}

impl<'a> EnrichedChannelPair<'a> {
    /// Build a same-action channel pair descriptor.
    #[must_use]
    pub const fn new(left: &'static str, right: &'static str, action_tags: &'a [Tag]) -> Self {
        Self {
            left,
            right,
            action_tags,
        }
    }

    #[must_use]
    const fn forward_cases(&self) -> [ChannelDerivationCase<'a>; 2] {
        channel_pair_cases(self.left, self.right, self.action_tags)
    }

    #[must_use]
    const fn reversed_cases(&self) -> [ChannelDerivationCase<'a>; 2] {
        channel_pair_cases(self.right, self.left, self.action_tags)
    }

    #[must_use]
    const fn move_pair(&self) -> [ActionLayerMovePair; 1] {
        [ActionLayerMovePair {
            left: self.left,
            right: self.right,
            label: "action layer",
        }]
    }
}

impl Enrichment {
    /// Return `true` when this enrichment needs the score/verified schema.
    #[must_use]
    pub const fn needs_signal_schema(&self) -> bool {
        !matches!(self.signal, SignalMode::None)
    }

    /// Final signal fixture for this enrichment, if any.
    #[must_use]
    pub const fn final_signal(&self) -> Option<FinalSignalFixture> {
        match self.signal {
            SignalMode::None => Option::None,
            SignalMode::Valid => Some(FinalSignalFixture::StandardScoreVerified),
            SignalMode::Degraded => Some(FinalSignalFixture::DegradedScoreVerified),
        }
    }

    /// Attach a signal schema when this enrichment needs one.
    #[must_use]
    pub fn maybe_signal_schema<B>(&self, builder: B) -> B
    where
        B: super::ScenarioBuilderExt,
    {
        if self.needs_signal_schema() {
            builder.signal_schema(score_verified_request_schema())
        } else {
            builder
        }
    }

    /// Attach a Sentinel and/or train an outcome axis according to this enrichment.
    #[track_caller]
    pub fn setup_world(&self, world: &mut Scenario, train_channel: &'static str, entity: &str) -> EnrichedSetup {
        let sentinel = if self.reported {
            Some(attach_golden_reporting_sentinel(world, "S1"))
        } else {
            Option::None
        };

        let axis = if self.live_outcome {
            let spatial = self.reported;
            let axis = world.register_axis("magnitude", spatial).expect("register outcome axis");
            self.train_outcome(world, train_channel, entity, axis);
            Some(axis)
        } else {
            Option::None
        };

        EnrichedSetup { sentinel, axis }
    }

    /// Train one live outcome axis through this enrichment's public request shape.
    #[track_caller]
    pub fn train_outcome(&self, world: &World, channel: &'static str, entity: &str, axis: OutcomeAxisId) {
        let label = format!("{} outcome training", self.label);
        train_standard_live_outcome_prediction_with_requests(
            world,
            axis,
            |cycle, world| self.decorate_training(self.base_request(world, channel, entity), cycle),
            &label,
        );
        self.refresh_if_reported(world);
    }

    /// Expected core payload for this enrichment and world-local setup.
    #[must_use]
    pub const fn core_payload(&self, setup: &EnrichedSetup) -> CorePayloadExpectation {
        let base = match (setup.sentinel, setup.axis) {
            (Some(sentinel), Some(axis)) => CorePayloadExpectation::reported_outcome(sentinel, axis),
            (Some(sentinel), Option::None) => CorePayloadExpectation::live_sentinel(sentinel),
            (Option::None, Some(axis)) => CorePayloadExpectation::outcome(axis),
            (Option::None, Option::None) => CorePayloadExpectation {
                sentinel: Option::None,
                outcome_axis: Option::None,
                signal_degradation: Option::None,
            },
        };

        if matches!(self.signal, SignalMode::Degraded) {
            base.with_signal_degradation(1, 1)
        } else {
            base
        }
    }

    /// Attach the final request-signal payload for this enrichment.
    #[must_use]
    pub fn decorate_final(&self, request: RequestContext) -> RequestContext {
        match self.signal {
            SignalMode::None => request,
            SignalMode::Valid => with_standard_final_score_verified(request),
            SignalMode::Degraded => with_degraded_score_verified(request),
        }
    }

    /// Attach this enrichment's training request-signal payload.
    #[must_use]
    pub fn decorate_training(&self, request: RequestContext, cycle: usize) -> RequestContext {
        match self.final_signal() {
            Some(signal_fixture) => signal_fixture.apply_training(request, cycle),
            Option::None => request,
        }
    }

    /// Build the undecorated public request shape for this enrichment.
    #[must_use]
    pub fn base_request(&self, world: &World, channel: &'static str, entity: &str) -> RequestContext {
        if self.reported {
            world.request_with_sentinel(channel, entity, "S1", GOLDEN_COORD)
        } else {
            world.request(channel, entity)
        }
    }

    /// Build the final public request shape for this enrichment.
    #[must_use]
    pub fn final_request(&self, world: &World, channel: &'static str, entity: &str) -> RequestContext {
        self.decorate_final(self.base_request(world, channel, entity))
    }

    /// Drift for same-observation fixed-core batches.
    #[must_use]
    pub const fn same_batch_core_drift(&self) -> f64 {
        if self.live_outcome {
            DIVERGENT_OUTCOME_REPLAY_DRIFT
        } else {
            PURITY_DRIFT
        }
    }

    /// Whether replay should compare from `RiskBasis` instead of the full payload.
    #[must_use]
    pub const fn replay_from_risk_basis(&self) -> bool {
        self.live_outcome || self.reported
    }

    /// Replay comparison appropriate for this enrichment's public payload.
    #[must_use]
    pub const fn replay_comparison(&self) -> DerivationReplayComparison {
        if self.replay_from_risk_basis() {
            DerivationReplayComparison::RiskBasis
        } else {
            DerivationReplayComparison::FullProfile
        }
    }

    /// Refresh the golden report when this enrichment uses reported requests.
    #[track_caller]
    pub fn refresh_if_reported(&self, world: &World) {
        if self.reported {
            refresh_golden_report(world, "S1");
        }
    }

    /// Assert world health for this enrichment's expected public request shape.
    #[track_caller]
    pub fn assert_health(&self, world: &World) {
        if matches!(self.signal, SignalMode::Degraded) {
            assert_signal_degradation_health(world);
        } else {
            assert_health_clean(world);
        }
    }
}

/// Assert that only expected request-signal degradation reached global health.
#[track_caller]
pub fn assert_signal_degradation_health(world: &World) {
    let health = world.assayer().health_summary();
    let degradation = &health.degradation;

    assert!(
        degradation.total_degraded > 0,
        "health: degraded signal path should record at least one degraded assessment",
    );
    assert!(
        degradation.signals_sanitised > 0,
        "health: degraded signal path should record sanitised signals",
    );
    assert_eq!(
        degradation.sentinel_slots_zeroed, 0,
        "health: sentinel_slots_zeroed = {}",
        degradation.sentinel_slots_zeroed,
    );
    assert_eq!(
        degradation.features_sanitised, 0,
        "health: features_sanitised = {}",
        degradation.features_sanitised,
    );
    assert_eq!(
        degradation.model_fallbacks, 0,
        "health: model_fallbacks = {}",
        degradation.model_fallbacks,
    );
    assert!(!health.any_cascade_terminus, "health: any_cascade_terminus set");
    assert_eq!(
        health.health_events_dropped, 0,
        "health: health_events_dropped = {}",
        health.health_events_dropped,
    );
}

/// Build the standard three-channel fixture for an enrichment.
#[track_caller]
pub fn build_enriched_standard_world(enrichment: &Enrichment, label: &str, seed: u64) -> Scenario {
    if enrichment.needs_signal_schema() {
        multi_channel_independence_scenario_with(label, seed, |builder| enrichment.maybe_signal_schema(builder))
    } else {
        multi_channel_independence_scenario(label, seed)
    }
    .expect("world should build")
}

/// Build the decision policy matrix fixture for an enrichment.
#[track_caller]
pub fn build_enriched_policy_matrix_world(enrichment: &Enrichment, label: &str, seed: u64) -> Scenario {
    if enrichment.needs_signal_schema() {
        decision_policy_matrix_scenario_with(label, seed, |builder| enrichment.maybe_signal_schema(builder))
    } else {
        decision_policy_matrix_scenario(label, seed)
    }
    .expect("world should build")
}

/// Build a same-action reward-pair fixture for an enrichment.
#[track_caller]
pub fn build_enriched_reward_world(
    enrichment: &Enrichment,
    label: &str,
    seed: u64,
    actions: &[Action],
    perturb: impl FnOnce(&mut RewardParameters),
) -> Scenario {
    reward_perturbation_scenario_with(label, seed, actions, perturb, |builder| {
        enrichment.maybe_signal_schema(builder)
    })
    .expect("world should build")
}

/// Build two same-policy channels for an enrichment.
#[track_caller]
pub fn build_enriched_same_policy_pair_world(
    enrichment: &Enrichment,
    label: &str,
    seed: u64,
    left_channel: &'static str,
    right_channel: &'static str,
    actions: &[Action],
) -> Scenario {
    let policy = policy_with_actions(actions.iter().copied());
    scenario_with(label, seed, |builder| {
        enrichment
            .maybe_signal_schema(builder)
            .channel(left_channel, policy.clone())
            .channel(right_channel, policy)
    })
    .expect("world should build")
}

/// No-op pair-training hook for enriched channel-pair runners.
pub const fn train_no_enriched_channel_pair_evidence(
    _: &World,
    _: &Enrichment,
    _: &EnrichedSetup,
    _: EnrichedChannelPair<'_>,
    _: &str,
    _: &str,
) {
}

/// Run a same-action channel-pair fixture with core-history preconditioning.
///
/// This is the pair-level companion to the enriched derivation-matrix runners:
/// callers provide world construction, optional core-side history, optional
/// decision-layer training, and the final assertion shape while the helper owns
/// the repeated build/setup / refresh/health ceremony.
#[track_caller]
pub fn run_enriched_channel_pair_history_family<'a, B, T>(
    label_prefix: &str,
    seed_salt: u64,
    mut build_world: B,
    history: EnrichedCoreHistory,
    pair: EnrichedChannelPair<'a>,
    mut train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    B: FnMut(&Enrichment, &str, u64) -> Scenario,
    T: FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'a>, &str, &str),
{
    run_enrichment_grid(label_prefix, seed_salt, |enrichment, label, seed| {
        let mut world = build_world(enrichment, label, seed);
        let setup = history.prepare(&mut world, enrichment, label);
        let entity = history.entity();

        train_pair(&world, enrichment, &setup, pair, entity, label);
        enrichment.refresh_if_reported(&world);

        assert_enriched_channel_pair_by_mode(&world, enrichment, &setup, entity, pair, assertion, label);
        enrichment.assert_health(&world);
    });
}

/// Run a same-action channel-pair fixture across the full enrichment grid.
#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: pair-family runner keeps fixture dimensions explicit at the boundary call site.
pub fn run_enriched_channel_pair_family<'a, B, T>(
    label_prefix: &str,
    seed_salt: u64,
    build_world: B,
    setup_channel: &'static str,
    entity: &'static str,
    pair: EnrichedChannelPair<'a>,
    train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    B: FnMut(&Enrichment, &str, u64) -> Scenario,
    T: FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'a>, &str, &str),
{
    run_enriched_channel_pair_history_family(
        label_prefix,
        seed_salt,
        build_world,
        EnrichedCoreHistory::fresh(setup_channel, entity),
        pair,
        train_pair,
        assertion,
    );
}

/// Run a same-policy channel-pair fixture across the full enrichment grid.
#[track_caller]
pub fn run_enriched_same_policy_pair_family<'a, T>(
    label_prefix: &str,
    seed_salt: u64,
    pair: EnrichedChannelPair<'a>,
    actions: &[Action],
    entity: &'static str,
    train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    T: FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'a>, &str, &str),
{
    run_enriched_channel_pair_family(
        label_prefix,
        seed_salt,
        |enrichment, label, seed| build_enriched_same_policy_pair_world(enrichment, label, seed, pair.left, pair.right, actions),
        pair.left,
        entity,
        pair,
        train_pair,
        assertion,
    );
}

/// Run a same-policy channel-pair fixture after core-history preconditioning.
#[track_caller]
pub fn run_enriched_same_policy_pair_history_family<'a, T>(
    label_prefix: &str,
    seed_salt: u64,
    pair: EnrichedChannelPair<'a>,
    actions: &[Action],
    history: EnrichedCoreHistory,
    train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    T: FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'a>, &str, &str),
{
    run_enriched_channel_pair_history_family(
        label_prefix,
        seed_salt,
        |enrichment, label, seed| build_enriched_same_policy_pair_world(enrichment, label, seed, pair.left, pair.right, actions),
        history,
        pair,
        train_pair,
        assertion,
    );
}

/// Run one same-policy channel-pair fixture across several core-history cases.
#[track_caller]
pub fn run_enriched_same_policy_pair_history_cases<'a, T>(
    cases: &[EnrichedHistoryCase],
    pair: EnrichedChannelPair<'a>,
    actions: &[Action],
    train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    T: Copy + FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'a>, &str, &str),
{
    for case in cases {
        run_enriched_same_policy_pair_history_family(
            case.label_prefix,
            case.seed_salt,
            pair,
            actions,
            case.history,
            train_pair,
            assertion,
        );
    }
}

/// Run every canonical reward-axis perturbation through the enriched channel-pair runner.
///
/// This is the reward-parameter companion to [`run_enriched_same_policy_pair_family`]:
/// each reward knob gets its own baseline/perturbed same-action channel pair,
/// while callers choose whether the final assertion is action-layer movement,
/// request-order replay, or full-profile replay after additional evidence.
#[track_caller]
pub fn run_enriched_reward_axis_sweep_family<T>(
    label_prefix: &str,
    seed_salt: u64,
    train_pair: T,
    assertion: EnrichedChannelPairAssertion,
) where
    T: Copy + FnMut(&World, &Enrichment, &EnrichedSetup, EnrichedChannelPair<'_>, &str, &str),
{
    for case in super::REWARD_AXIS_SWEEP_CASES {
        let case_prefix = format!("{label_prefix}-{}", case.name);
        run_enriched_channel_pair_family(
            &case_prefix,
            seed_salt ^ case.seed,
            |enrichment, label, seed| build_enriched_reward_world(enrichment, label, seed, case.actions, case.perturb),
            "baseline",
            "alice",
            EnrichedChannelPair::new("baseline", "perturbed", case.action_tags),
            train_pair,
            assertion,
        );
    }
}

#[track_caller]
fn assert_enriched_channel_pair_by_mode(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    pair: EnrichedChannelPair<'_>,
    assertion: EnrichedChannelPairAssertion,
    label: &str,
) {
    match assertion {
        EnrichedChannelPairAssertion::ActionOnly => {
            let assessments = assert_enriched_channel_pair_action_only(world, enrichment, setup, entity, pair, label);
            assert_distinct_pair_channel_ids(&assessments, label);
        }
        EnrichedChannelPairAssertion::RequestOrder => {
            let (forward, reordered) = assert_enriched_channel_pair_request_order(world, enrichment, setup, entity, pair, label);
            assert_distinct_pair_channel_ids(&forward, &format!("{label}: forward"));
            assert_distinct_pair_channel_ids(&reordered, &format!("{label}: reordered"));
        }
        EnrichedChannelPairAssertion::CoreAndResonanceProfile => {
            let assessments =
                assert_enriched_channel_pair_replays_core_and_resonance_profile(world, enrichment, setup, entity, pair, label);
            assert_distinct_pair_channel_ids(&assessments, label);
        }
        EnrichedChannelPairAssertion::CoreAndResonanceProfileRequestOrder => {
            let (forward, reordered) = assert_enriched_channel_pair_replays_core_and_resonance_profile_request_order(
                world, enrichment, setup, entity, pair, label,
            );
            assert_distinct_pair_channel_ids(&forward, &format!("{label}: forward"));
            assert_distinct_pair_channel_ids(&reordered, &format!("{label}: reordered"));
        }
    }
}

#[track_caller]
fn assert_distinct_pair_channel_ids(assessments: &[DerivedReckoning], label: &str) {
    assert_eq!(assessments.len(), 2, "{label}: expected exactly two channel-pair assessments");
    assert_ne!(
        assessments[0].channel, assessments[1].channel,
        "{label}: channel ids should differ"
    );
}

/// Build the challenge/reward matrix fixture for an enrichment.
#[track_caller]
pub fn build_enriched_challenge_reward_world(enrichment: &Enrichment, label: &str, seed: u64) -> Scenario {
    if enrichment.needs_signal_schema() {
        challenge_reward_policy_matrix_scenario_with(label, seed, |builder| enrichment.maybe_signal_schema(builder))
    } else {
        challenge_reward_policy_matrix_scenario(label, seed)
    }
    .expect("world should build")
}

const ENRICHED_STANDARD_MATRIX_FIXTURE: EnrichedMatrixFixture<'static> = EnrichedMatrixFixture {
    build_world: build_enriched_standard_world,
    matrix: STANDARD_DERIVATION_MATRIX,
    training_channel: "login",
    train_pair_evidence: train_no_extra_matrix_pair_evidence,
};

const ENRICHED_POLICY_MATRIX_FIXTURE: EnrichedMatrixFixture<'static> = EnrichedMatrixFixture {
    build_world: build_enriched_policy_matrix_world,
    matrix: DECISION_POLICY_MATRIX,
    training_channel: "login",
    train_pair_evidence: train_no_extra_matrix_pair_evidence,
};

const ENRICHED_CHALLENGE_REWARD_MATRIX_FIXTURE: EnrichedMatrixFixture<'static> = EnrichedMatrixFixture {
    build_world: build_enriched_challenge_reward_world,
    matrix: CHALLENGE_REWARD_POLICY_MATRIX,
    training_channel: "challenged",
    train_pair_evidence: train_challenge_reward_matrix_pair_evidence,
};

/// Build final same-observation requests for a derivation matrix.
#[must_use]
pub fn build_enriched_matrix_requests(
    world: &World,
    enrichment: &Enrichment,
    entity: &str,
    matrix: DerivationMatrix<'_>,
) -> Vec<RequestContext> {
    matrix
        .cases
        .iter()
        .map(|case| enrichment.final_request(world, case.name, entity))
        .collect()
}

/// Train one channel with adverse `Challenge + Fail` evidence.
#[track_caller]
pub fn train_enriched_channel_fail_evidence(
    world: &World,
    enrichment: &Enrichment,
    _setup: &EnrichedSetup,
    channel: &'static str,
    entity: &str,
    cycles: usize,
    label: &str,
) {
    train_channel_challenge_fail_evidence_with_requests(
        world,
        channel,
        cycles,
        |world, channel, cycle| enrichment.decorate_training(enrichment.base_request(world, channel, entity), cycle),
        label,
    );
}

/// Train matching `Challenge + Fail` evidence on two same-policy channels.
///
/// This is the exact-replay companion to the Pass/Fail and Fail/absent
/// fixtures: both channels walk the same core and companion-tracker path, so a
/// later same-observation assessment should replay the complete resonance
/// profile even when the request shape carries signals, reports, or outcome
/// predictions.
#[track_caller]
pub fn train_enriched_matching_challenge_evidence(
    world: &World,
    enrichment: &Enrichment,
    pair: EnrichedChannelPair<'_>,
    entity: &str,
    cycles: usize,
    label: &str,
) {
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    for cycle in 0..cycles {
        cycle_request(
            world,
            enrichment.decorate_training(enrichment.base_request(world, pair.left, entity), cycle),
            with_fail,
        );
        cycle_request(
            world,
            enrichment.decorate_training(enrichment.base_request(world, pair.right, entity), cycle),
            with_fail,
        );
    }
}

/// Train Fail evidence on one channel and absent-result evidence on another.
#[track_caller]
pub fn train_enriched_fail_vs_absent_evidence(
    world: &World,
    enrichment: &Enrichment,
    pair: EnrichedChannelPair<'_>,
    entity: &str,
    cycles: usize,
    label: &str,
) {
    train_channel_challenge_fail_vs_absent_evidence_with_requests(
        world,
        pair.left,
        pair.right,
        cycles,
        |world, channel, cycle| enrichment.decorate_training(enrichment.base_request(world, channel, entity), cycle),
        label,
    );
}

/// Train paired channels with present Fail evidence versus present Pass evidence.
///
/// Both labels follow the same Challenge-result eligibility path into the shared
/// core model. Only the channel-local companion tracker receives opposite
/// decision-layer evidence.
#[track_caller]
pub fn train_enriched_pass_vs_fail_evidence(
    world: &World,
    enrichment: &Enrichment,
    pair: EnrichedChannelPair<'_>,
    entity: &str,
    cycles: usize,
    label: &str,
) {
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let fail_result = adverse_challenge_result(ChallengeResult::Fail);
    let pass_result = adverse_challenge_result(ChallengeResult::Pass);
    for cycle in 0..cycles {
        cycle_request(
            world,
            enrichment.decorate_training(enrichment.base_request(world, pair.left, entity), cycle),
            fail_result,
        );
        cycle_request(
            world,
            enrichment.decorate_training(enrichment.base_request(world, pair.right, entity), cycle),
            pass_result,
        );
    }
}

/// Train Fail evidence across the challenged channels in the challenge/reward matrix.
#[track_caller]
pub fn train_enriched_matrix_fail_evidence(
    world: &World,
    enrichment: &Enrichment,
    _setup: &EnrichedSetup,
    cycles: usize,
    label: &str,
) {
    train_challenge_reward_policy_matrix_fail_evidence_with_requests(
        world,
        cycles,
        |world, channel, cycle| enrichment.decorate_training(enrichment.base_request(world, channel, "alice"), cycle),
        label,
    );
}

/// Prepare an enriched world with the canonical post-label-history core state.
///
/// The helper keeps the preconditioned families
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// aligned with the same public
/// request enrichment grid as the cold-batch families: request signals are
/// present on training cycles when declared, reported variants train through the
/// live Sentinel request shape, and outcome variants keep their prediction
/// payload live before the final derivation probe.
#[track_caller]
pub fn setup_enriched_label_history(
    world: &mut Scenario,
    enrichment: &Enrichment,
    channel: &'static str,
    entity: &str,
    cycles: usize,
    adverse_every: usize,
    label: &str,
) -> EnrichedSetup {
    if !enrichment.reported {
        world
            .register_sentinel("S1")
            .unwrap_or_else(|error| panic!("{label}: register history Sentinel failed: {error:?}"));
    }

    let setup = enrichment.setup_world(world, channel, entity);
    train_enriched_label_history(world, enrichment, channel, cycles, adverse_every, label);
    setup
}

/// Prepare an enriched world with the canonical post-identity-history core state.
#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: names each independent fixture dimension at the boundary call site.
pub fn setup_enriched_identity_history(
    world: &mut Scenario,
    enrichment: &Enrichment,
    identity_dimension: &'static str,
    channel: &'static str,
    entity: &str,
    cycles: usize,
    adverse_every: usize,
    label: &str,
) -> EnrichedSetup {
    world
        .register_identity(identity_dimension)
        .unwrap_or_else(|error| panic!("{label}: register identity {identity_dimension}: {error:?}"));

    let setup = enrichment.setup_world(world, channel, entity);
    train_enriched_label_history(world, enrichment, channel, cycles, adverse_every, label);
    setup
}

/// Train the canonical alternating entity history through an enrichment's
/// request shape.
#[track_caller]
pub fn train_enriched_label_history(
    world: &World,
    enrichment: &Enrichment,
    channel: &'static str,
    cycles: usize,
    adverse_every: usize,
    label: &str,
) {
    assert!(cycles > 0, "{label}: at least one label-history cycle is required");
    assert!(adverse_every > 0, "{label}: adverse_every must be non-zero");

    for (cycle, case) in alternating_entity_label_stream(cycles, adverse_every).enumerate() {
        cycle_request(
            world,
            enrichment.decorate_training(enrichment.base_request(world, channel, case.entity), cycle),
            |assessment_id| case.label_spec(assessment_id),
        );
    }
    enrichment.refresh_if_reported(world);
}

/// Train a reported reward-label history through one channel.
#[track_caller]
pub fn train_reported_reward_label_history(world: &World, channel: &'static str, sentinel_name: &'static str, cycles: usize) {
    cycle_reported_label_case_stream(
        world,
        channel,
        sentinel_name,
        GOLDEN_COORD,
        alternating_label_stream_for_entity("alice", cycles, 3),
        "reported reward label history",
    );
    refresh_golden_report(world, sentinel_name);
}

/// Train matching label streams through two same-seed reward worlds.
#[track_caller]
pub fn train_matched_reward_label_history(
    baseline_world: &World,
    perturbed_world: &World,
    channel: &'static str,
    use_signals: bool,
    cycles: usize,
    label: &str,
) {
    // Both worlds settle their cold ramps before their histories diverge.
    // The ramp advances on the assessment clock and the steward applies it
    // asynchronously, so two worlds trained in parallel reach different
    // accepted counts depending only on how the threads were scheduled — and
    // a comparison taken afterwards would be reading that scheduling rather
    // than the thing under test (´inv:guarantee:evidence-authority´). Settled
    // first, both stand on the same empirical moments and the matched label
    // histories move them alike from there.
    baseline_world.settle_cold_ramp_with(&[baseline_world.request(channel, "alice")]);
    perturbed_world.settle_cold_ramp_with(&[perturbed_world.request(channel, "alice")]);

    for (cycle, case) in alternating_label_stream_for_entity("alice", cycles, 3).enumerate() {
        if use_signals {
            cycle_request_pair(
                baseline_world,
                FinalSignalFixture::StandardScoreVerified.apply_training(baseline_world.request(channel, case.entity), cycle),
                |assessment_id| case.label_spec(assessment_id),
                perturbed_world,
                FinalSignalFixture::StandardScoreVerified.apply_training(perturbed_world.request(channel, case.entity), cycle),
                |assessment_id| case.label_spec(assessment_id),
                &format!("{label}: cycle {cycle}"),
            );
        } else {
            cycle_label_case_pair(
                baseline_world,
                perturbed_world,
                channel,
                case,
                &format!("{label}: cycle {cycle}"),
            );
        }
    }
}

/// Train a four-action mixed-action label history through two same-seed worlds.
#[track_caller]
pub fn train_matched_mixed_action_label_history(
    baseline_world: &World,
    perturbed_world: &World,
    channel: &'static str,
    cycles: usize,
    label: &str,
) {
    // Both worlds settle their cold ramps before their histories diverge.
    // The ramp advances on the assessment clock and the steward applies it
    // asynchronously, so two worlds trained in parallel reach different
    // accepted counts depending only on how the threads were scheduled — and
    // a comparison taken afterwards would be reading that scheduling rather
    // than the thing under test (´inv:guarantee:evidence-authority´). Settled
    // first, both stand on the same empirical moments and the matched label
    // histories move them alike from there.
    baseline_world.settle_cold_ramp_with(&[baseline_world.request(channel, "alice")]);
    perturbed_world.settle_cold_ramp_with(&[perturbed_world.request(channel, "alice")]);

    for cycle in 0..cycles {
        let action = match cycle % 4 {
            0 => Action::Allow,
            1 => Action::Challenge,
            2 => Action::Slow,
            _ => Action::Block,
        };
        let case = LabelCycleCase {
            entity: if cycle % 2 == 0 { "alice" } else { "bob" },
            action,
            adverse: cycle % 3 == 0,
            ground_truth: false,
            challenge_result: (action == Action::Challenge).then_some(ChallengeResult::Fail),
        };
        cycle_label_case_pair(
            baseline_world,
            perturbed_world,
            channel,
            case,
            &format!("{label}: cycle {cycle}"),
        );
    }
}

/// Train matching Challenge+Fail / Challenge+Pass streams through signal-bearing worlds.
#[track_caller]
pub fn train_signal_challenge_pair(
    fail_world: &World,
    pass_world: &World,
    channel: &'static str,
    entity: &str,
    cycles: usize,
    degraded: bool,
    label: &str,
) {
    let fixture = if degraded {
        FinalSignalFixture::DegradedScoreVerified
    } else {
        FinalSignalFixture::StandardScoreVerified
    };
    let fail_label = adverse_challenge_result(ChallengeResult::Fail);
    let pass_label = adverse_challenge_result(ChallengeResult::Pass);

    for cycle in 0..cycles {
        cycle_request_pair(
            fail_world,
            fixture.apply_training(fail_world.request(channel, entity), cycle),
            fail_label,
            pass_world,
            fixture.apply_training(pass_world.request(channel, entity), cycle),
            pass_label,
            &format!("{label}: cycle {cycle}"),
        );
    }
}

/// Assert any enriched same-observation derivation matrix.
///
/// The cell's enrichment is assessed once and every channel derives from that
/// single assessment. There used to be a second, per-request-batch form of this
/// helper, chosen by a flag on the history fixture: pre-conditioned worlds
/// derived once because their in-flight label history could land in the middle
/// of a batch, and fresh worlds submitted the batch. The distinction did not
/// survive inspection — a fresh world's cold standardisation ramp straddles a
/// batch for exactly the same reason, since the snapshot is loaded per request
/// (´dec:concurrency:per-request-load´) and an accepted observation advances a
/// later one (´dec:ordering:score-before-evolve´). Both arms now name the same
/// computation and only this one remains.
#[track_caller]
pub fn assert_enriched_derivation_matrix(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    matrix: DerivationMatrix<'_>,
    label: &str,
) -> Vec<DerivedReckoning> {
    let request = enrichment.final_request(world, matrix.cases[0].name, entity);
    super::derivation::assert_derivation_matrix_derives_once_with_drift(
        world,
        request,
        matrix,
        enrichment.core_payload(setup),
        enrichment.same_batch_core_drift(),
        label,
    )
}

/// Assert enriched request-order replay for any derivation matrix
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´).
///
/// Both orders derive from one assessment of the cell's enrichment, for the
/// reason [`assert_enriched_derivation_matrix`] gives: submitting each order as
/// its own batch would put the two in coordinate systems an accepted
/// observation apart and compare the steward rather than the derivation
/// (´inv:guarantee:evidence-authority´).
///
/// Holding one assessment is also why the budget here is the plain purity one
/// for every cell of the grid: with no second call there is no window a
/// publication can land in, and measurement under load found nothing left for a
/// wider budget to cover.
#[track_caller]
pub fn assert_enriched_derivation_matrix_request_order(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    matrix: DerivationMatrixRequestOrder<'_>,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let request = enrichment.final_request(world, matrix.forward_cases[0].name, entity);
    super::derivation::assert_derivation_matrix_request_order_derives_once_with_comparison_and_drift(
        world,
        request,
        matrix,
        enrichment.core_payload(setup),
        enrichment.replay_comparison(),
        PURITY_DRIFT,
        label,
    )
}

/// Assert two enriched same-action channels move only the action layer.
#[track_caller]
pub fn assert_enriched_channel_pair_action_only(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    pair: EnrichedChannelPair<'_>,
    label: &str,
) -> Vec<DerivedReckoning> {
    let cases = pair.forward_cases();
    let move_pairs = pair.move_pair();
    let matrix = DerivationMatrix::new(&cases, &move_pairs);

    assert_enriched_derivation_matrix(world, enrichment, setup, entity, matrix, label)
}

/// Assert two enriched same-policy channels replay the complete public profile.
///
/// This is the enriched companion to
/// [`assert_channel_pair_replays_core_and_resonance_profile`]. It first proves
/// that the rich core payload is present and shared, then checks that no
/// channel-id residue or companion-tracker difference moved the resonance
/// profile.
#[track_caller]
pub fn assert_enriched_channel_pair_replays_core_and_resonance_profile(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    pair: EnrichedChannelPair<'_>,
    label: &str,
) -> Vec<DerivedReckoning> {
    let cases = pair.forward_cases();
    let move_pairs: [ActionLayerMovePair; 0] = [];
    let matrix = DerivationMatrix::new(&cases, &move_pairs);

    let assessments = assert_enriched_derivation_matrix(world, enrichment, setup, entity, matrix, label);
    assert_eq!(assessments.len(), 2, "{label}: expected exactly two alias assessments");
    assert_core_and_resonance_profile_near_with_drift(
        &assessments[0],
        &assessments[1],
        enrichment.same_batch_core_drift(),
        label,
    );

    assessments
}

/// Assert two enriched same-policy aliases replay by request order and share the full profile in each batch.
#[track_caller]
pub fn assert_enriched_channel_pair_replays_core_and_resonance_profile_request_order(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    pair: EnrichedChannelPair<'_>,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let forward_cases = pair.forward_cases();
    let reordered_cases = pair.reversed_cases();
    let move_pairs: [ActionLayerMovePair; 0] = [];
    let matrix = DerivationMatrixRequestOrder::new(&forward_cases, &reordered_cases, &move_pairs);

    let (forward, reordered) = assert_enriched_derivation_matrix_request_order(world, enrichment, setup, entity, matrix, label);

    assert_eq!(forward.len(), 2, "{label}: expected exactly two forward alias assessments");
    assert_eq!(
        reordered.len(),
        2,
        "{label}: expected exactly two reordered alias assessments"
    );
    assert_core_and_resonance_profile_near_with_drift(
        &forward[0],
        &forward[1],
        PURITY_DRIFT,
        &format!("{label}: forward aliases"),
    );
    assert_core_and_resonance_profile_near_with_drift(
        &reordered[0],
        &reordered[1],
        PURITY_DRIFT,
        &format!("{label}: reordered aliases"),
    );

    (forward, reordered)
}

/// Assert request-order replay for an enriched same-action channel pair.
#[track_caller]
pub fn assert_enriched_channel_pair_request_order(
    world: &World,
    enrichment: &Enrichment,
    setup: &EnrichedSetup,
    entity: &str,
    pair: EnrichedChannelPair<'_>,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let forward_cases = pair.forward_cases();
    let reordered_cases = pair.reversed_cases();
    let move_pairs = pair.move_pair();
    let matrix = DerivationMatrixRequestOrder::new(&forward_cases, &reordered_cases, &move_pairs);

    assert_enriched_derivation_matrix_request_order(world, enrichment, setup, entity, matrix, label)
}

/// Run the divergent-outcome standard three-channel matrix witness.
#[track_caller]
pub fn run_divergent_outcome_standard_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_divergent_outcome_matrix(enrichment, ENRICHED_STANDARD_MATRIX_FIXTURE, label, seed);
}

/// Run the divergent-outcome decision policy matrix witness.
#[track_caller]
pub fn run_divergent_outcome_policy_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_divergent_outcome_matrix(enrichment, ENRICHED_POLICY_MATRIX_FIXTURE, label, seed);
}

/// Run the divergent-outcome challenge/reward matrix witness.
#[track_caller]
pub fn run_divergent_outcome_challenge_reward_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_divergent_outcome_matrix(enrichment, ENRICHED_CHALLENGE_REWARD_MATRIX_FIXTURE, label, seed);
}

/// Run a divergent-outcome replay witness for any enriched derivation matrix.
#[track_caller]
fn run_divergent_outcome_matrix(enrichment: &Enrichment, fixture: EnrichedMatrixFixture<'_>, label: &str, seed: u64) {
    let mut positive_world = fixture.build_world(enrichment, &format!("{label}-positive"), seed);
    let mut negative_world = fixture.build_world(enrichment, &format!("{label}-negative"), seed);
    settle_matrix_world_pair(&positive_world, &negative_world, fixture.training_channel, "alice");

    let (positive_sentinel, negative_sentinel) =
        attach_reported_pair_if_needed(enrichment, &mut positive_world, &mut negative_world);
    let axes = register_divergent_axes(&mut positive_world, &mut negative_world, enrichment.reported, label);
    train_divergent_pair(
        enrichment,
        &positive_world,
        axes.positive,
        &negative_world,
        axes.negative,
        fixture.training_channel,
        "alice",
        label,
    );
    (fixture.train_pair_evidence)(enrichment, &positive_world, &negative_world, label);
    refresh_reported_pair_if_needed(enrichment, &positive_world, &negative_world);

    let positive_requests = build_enriched_matrix_requests(&positive_world, enrichment, "alice", fixture.matrix);
    let negative_requests = build_enriched_matrix_requests(&negative_world, enrichment, "alice", fixture.matrix);
    let positive_payload = enrichment.core_payload(&EnrichedSetup {
        sentinel: positive_sentinel,
        axis: Some(axes.positive),
    });
    let negative_payload = enrichment.core_payload(&EnrichedSetup {
        sentinel: negative_sentinel,
        axis: Some(axes.negative),
    });

    super::derivation::assert_divergent_outcome_matrix_replays_from_risk_basis_with_drift(
        WorldDerivationBatch::new(&positive_world, &positive_requests).with_core_payload(positive_payload),
        WorldDerivationBatch::new(&negative_world, &negative_requests).with_core_payload(negative_payload),
        axes,
        fixture.matrix,
        DIVERGENT_OUTCOME_REPLAY_DRIFT,
        label,
    );
    enrichment.assert_health(&positive_world);
    enrichment.assert_health(&negative_world);
}

/// Settle both worlds of a cross-world matrix witness before their histories diverge.
///
/// A cross-world witness ends by comparing one world against the other, so anything that moves one of them and not
/// the other lands inside the compared quantity. The standardisation ramp is exactly that kind of thing while a world
/// is still transitioning: it advances on accepted observations, the steward applies them asynchronously, and two
/// worlds driven in parallel therefore reach different accepted counts depending only on how the threads were
/// scheduled. A comparison taken afterwards reads that scheduling as a difference in the statistic under test
/// (´inv:guarantee:evidence-authority´). Settled first, both worlds stand at the horizon, where the prior mass is gone
/// and the coordinate system moves only on labels, and the matched histories move them alike from there.
///
/// The paired trainers that drive two worlds through one label stream settle for this reason already. The matrix
/// witnesses build their own worlds instead of borrowing a trainer's, so the barrier belongs at their call sites, and
/// its absence there is what left them reading the scheduler under load.
#[track_caller]
fn settle_matrix_world_pair(left: &World, right: &World, channel: &'static str, entity: &str) {
    left.settle_cold_ramp_with(&[left.request(channel, entity)]);
    right.settle_cold_ramp_with(&[right.request(channel, entity)]);
}

/// Run the lifecycle-sufficiency standard three-channel matrix witness.
#[track_caller]
pub fn run_lifecycle_sufficiency_standard_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_lifecycle_sufficiency_matrix(enrichment, ENRICHED_STANDARD_MATRIX_FIXTURE, label, seed);
}

/// Run the lifecycle-sufficiency decision policy matrix witness.
#[track_caller]
pub fn run_lifecycle_sufficiency_policy_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_lifecycle_sufficiency_matrix(enrichment, ENRICHED_POLICY_MATRIX_FIXTURE, label, seed);
}

/// Run the lifecycle-sufficiency challenge/reward matrix witness.
#[track_caller]
pub fn run_lifecycle_sufficiency_challenge_reward_matrix(enrichment: &Enrichment, label: &str, seed: u64) {
    run_lifecycle_sufficiency_matrix(enrichment, ENRICHED_CHALLENGE_REWARD_MATRIX_FIXTURE, label, seed);
}

/// Run a lifecycle-sufficiency witness for any enriched derivation matrix.
#[track_caller]
fn run_lifecycle_sufficiency_matrix(enrichment: &Enrichment, fixture: EnrichedMatrixFixture<'_>, label: &str, seed: u64) {
    let mut fresh_world = fixture.build_world(enrichment, &format!("{label}-fresh"), seed);
    let mut cycled_world = fixture.build_world(enrichment, &format!("{label}-cycled"), seed);
    settle_matrix_world_pair(&fresh_world, &cycled_world, fixture.training_channel, "alice");

    let (fresh_setup, cycled_setup) = setup_lifecycle_pair(
        enrichment,
        &mut fresh_world,
        &mut cycled_world,
        fixture.training_channel,
        "alice",
        label,
    );
    (fixture.train_pair_evidence)(enrichment, &fresh_world, &cycled_world, label);
    refresh_reported_pair_if_needed(enrichment, &fresh_world, &cycled_world);

    assert_lifecycle_enriched_matrix(
        enrichment,
        &fresh_world,
        &fresh_setup,
        &cycled_world,
        &cycled_setup,
        "alice",
        fixture.matrix,
        LIFECYCLE_SUFFICIENCY_DRIFT,
        label,
    );
    enrichment.assert_health(&fresh_world);
    enrichment.assert_health(&cycled_world);
}

const fn train_no_extra_matrix_pair_evidence(_: &Enrichment, _: &World, _: &World, _: &str) {}

#[track_caller]
fn train_challenge_reward_matrix_pair_evidence(enrichment: &Enrichment, left_world: &World, right_world: &World, label: &str) {
    train_challenge_reward_policy_matrix_fail_evidence_pair_with_requests(
        left_world,
        right_world,
        8,
        |world, channel, cycle| enrichment.decorate_training(enrichment.base_request(world, channel, "alice"), cycle),
        label,
    );
}

#[track_caller]
fn attach_reported_pair_if_needed(
    enrichment: &Enrichment,
    positive_world: &mut Scenario,
    negative_world: &mut Scenario,
) -> (Option<SentinelId>, Option<SentinelId>) {
    if enrichment.reported {
        (
            Some(attach_golden_reporting_sentinel(positive_world, "S1")),
            Some(attach_golden_reporting_sentinel(negative_world, "S1")),
        )
    } else {
        (Option::None, Option::None)
    }
}

#[track_caller]
fn refresh_reported_pair_if_needed(enrichment: &Enrichment, left_world: &World, right_world: &World) {
    if enrichment.reported {
        refresh_golden_report(left_world, "S1");
        refresh_golden_report(right_world, "S1");
    }
}

#[track_caller]
fn register_divergent_axes(
    positive_world: &mut Scenario,
    negative_world: &mut Scenario,
    spatial: bool,
    label: &str,
) -> DivergentOutcomeAxisPair {
    let positive = positive_world
        .register_axis("magnitude", spatial)
        .unwrap_or_else(|error| panic!("{label}: register positive outcome axis failed: {error:?}"));
    let negative = negative_world
        .register_axis("magnitude", spatial)
        .unwrap_or_else(|error| panic!("{label}: register negative outcome axis failed: {error:?}"));

    DivergentOutcomeAxisPair { positive, negative }
}

#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: each world is named beside the axis it is trained on, and the pairing is the fixture — a struct holding the four would let a caller pass the positive axis with the negative world.
fn train_divergent_pair(
    enrichment: &Enrichment,
    positive_world: &World,
    positive_axis: OutcomeAxisId,
    negative_world: &World,
    negative_axis: OutcomeAxisId,
    channel: &'static str,
    entity: &str,
    label: &str,
) {
    train_standard_divergent_outcome_prediction_pair_with_requests(
        positive_world,
        positive_axis,
        negative_world,
        negative_axis,
        |cycle, world| enrichment.decorate_training(enrichment.base_request(world, channel, entity), cycle),
        label,
    );
}

#[track_caller]
fn setup_lifecycle_pair(
    enrichment: &Enrichment,
    fresh_world: &mut Scenario,
    cycled_world: &mut Scenario,
    training_channel: &'static str,
    entity: &str,
    label: &str,
) -> (EnrichedSetup, EnrichedSetup) {
    let (fresh_sentinel, cycled_sentinel) = if enrichment.reported {
        let (fresh, cycled) = register_distinct_golden_reporting_sentinel_states(fresh_world, cycled_world, label);
        (Some(fresh), Some(cycled))
    } else {
        register_distinct_live_sentinel_states(fresh_world, cycled_world, label);
        (Option::None, Option::None)
    };

    let (fresh_axis, cycled_axis) = if enrichment.live_outcome {
        let fresh_axis = fresh_world
            .register_axis("magnitude", enrichment.reported)
            .unwrap_or_else(|error| panic!("{label}: register fresh outcome axis failed: {error:?}"));
        let cycled_axis = cycled_world
            .register_axis("magnitude", enrichment.reported)
            .unwrap_or_else(|error| panic!("{label}: register cycled outcome axis failed: {error:?}"));
        train_standard_live_outcome_prediction_pair_with_requests(
            fresh_world,
            fresh_axis,
            cycled_world,
            cycled_axis,
            |cycle, world| enrichment.decorate_training(enrichment.base_request(world, training_channel, entity), cycle),
            &format!("{label}: paired outcome training"),
        );
        refresh_reported_pair_if_needed(enrichment, fresh_world, cycled_world);
        (Some(fresh_axis), Some(cycled_axis))
    } else {
        (Option::None, Option::None)
    };

    (
        EnrichedSetup {
            sentinel: fresh_sentinel,
            axis: fresh_axis,
        },
        EnrichedSetup {
            sentinel: cycled_sentinel,
            axis: cycled_axis,
        },
    )
}

#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: each world is named beside the setup it was built from, and the witness compares the two — a struct holding both pairs would let a caller cross them at the call site without the compiler noticing.
fn assert_lifecycle_enriched_matrix(
    enrichment: &Enrichment,
    fresh_world: &World,
    fresh_setup: &EnrichedSetup,
    cycled_world: &World,
    cycled_setup: &EnrichedSetup,
    entity: &str,
    matrix: DerivationMatrix<'_>,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let fresh_requests = build_enriched_matrix_requests(fresh_world, enrichment, entity, matrix);
    let cycled_requests = build_enriched_matrix_requests(cycled_world, enrichment, entity, matrix);
    let fresh_batch =
        WorldDerivationBatch::new(fresh_world, &fresh_requests).with_core_payload(enrichment.core_payload(fresh_setup));
    let cycled_batch =
        WorldDerivationBatch::new(cycled_world, &cycled_requests).with_core_payload(enrichment.core_payload(cycled_setup));

    super::derivation::assert_world_pair_derivation_matrix_replays_with_comparison_and_drift(
        fresh_batch,
        cycled_batch,
        matrix,
        enrichment.replay_comparison(),
        drift,
        label,
    )
}
