// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{
    PURITY_DRIFT, Scenario, World, WorldBuildError, WorldBuilder, assert_core_and_resonance_profile_near_with_drift,
    assert_core_assessment_near_with_drift, assert_reckoning_well_formed, cycle_request, refresh_golden_report, scenario_with,
    with_degraded_score_verified, with_standard_training_score_verified,
};
use torrust_assayer::types::{ChallengeResult, SentinelId};
use torrust_assayer::{DerivedReckoning, RequestContext, RewardParameters, RiskAssessment, Tag};

use super::assertions::{
    CorePayloadExpectation, action_tag_count, assert_action_layer_moves_without_core, assert_action_tag_sequence,
    assert_classification_tags_near_with_drift, assert_core_payload_expectation,
    assert_risk_basis_and_resonance_profile_near_with_drift,
};
use super::constants::{API_ACTION_TAGS, FOUR_ACTIONS, LOGIN_ACTION_TAGS, THREE_ACTIONS, TRANSACTION_ACTION_TAGS, TWO_ACTIONS};
use super::labels::{adverse_challenge_label, adverse_challenge_result, cycle_request_pair};
use super::outcomes::{
    DivergentOutcomeAxisPair, assert_outcome_prediction_raw_ordered, assert_outcome_prediction_shared_by_all_with_drift,
};
use super::policy::{high_decision_cost_reward, policy_with, policy_with_actions};

/// Expected channel-specific action tags for a same-observation derivation probe.
#[derive(Clone, Copy, Debug)]
pub struct ChannelDerivationCase<'a> {
    /// Channel name registered in the [`World`].
    pub name: &'static str,
    /// Action tags expected from this channel, in policy order.
    pub action_tags: &'a [Tag],
}

/// Observed reckoning plus its expected channel-specific action tags.
#[derive(Clone, Copy, Debug)]
pub struct ObservedDerivationCase<'a> {
    /// Human-readable case label, usually the channel name.
    pub name: &'static str,
    /// Reckoning already produced by the engine.
    pub reckoning: &'a DerivedReckoning,
    /// Action tags expected from this channel, in policy order.
    pub action_tags: &'a [Tag],
}

/// Caller-built public request batch for one world in a cross-world probe
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´).
///
/// Cross-world sufficiency and outcome-prediction tests often need custom
/// request payloads on each side: request-scoped signals, expected degradation,
/// or world-local outcome-axis ids. Bundling the world, request slice, and
/// expected core payload keeps those helpers from growing long parallel
/// argument lists.
#[derive(Clone, Copy)]
pub struct WorldDerivationBatch<'a> {
    /// World that will evaluate the request batch.
    pub world: &'a World,
    /// Requests ordered to match the shared derivation-case list.
    pub requests: &'a [RequestContext],
    /// Expected core payload for every reckoning produced by this batch.
    pub core_payload: CorePayloadExpectation,
}

impl<'a> WorldDerivationBatch<'a> {
    /// Build a cross-world derivation batch with no rich core-payload witness.
    #[must_use]
    pub const fn new(world: &'a World, requests: &'a [RequestContext]) -> Self {
        Self {
            world,
            requests,
            core_payload: CorePayloadExpectation {
                sentinel: None,
                outcome_axis: None,
                signal_degradation: None,
            },
        }
    }

    /// Attach the expected public core payload for this world's batch.
    #[must_use]
    pub const fn with_core_payload(mut self, core_payload: CorePayloadExpectation) -> Self {
        self.core_payload = core_payload;
        self
    }
}

/// Standard final request-signal fixtures used by the matrix probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[derive(Clone, Copy, Debug)]
pub enum FinalSignalFixture {
    /// Valid score/verified payload used for signal-rich final assessments.
    StandardScoreVerified,
    /// Deliberately degraded score/verified payload used for health-footprint probes.
    DegradedScoreVerified,
}

impl FinalSignalFixture {
    /// Attach this fixture's training request-scoped signals to a request.
    #[must_use]
    pub fn apply_training(self, request: RequestContext, cycle: usize) -> RequestContext {
        match self {
            Self::StandardScoreVerified => with_standard_training_score_verified(request, cycle),
            Self::DegradedScoreVerified => with_degraded_score_verified(request),
        }
    }
}

/// A named pair of same-action-shape assessments whose action layer should move.
///
/// Composition tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// often submit a whole channel matrix in one batch and
/// then need liveness checks for selected same-shape pairs: one pair proves a
/// reward perturbation was visible, another proves challenge-tracker evidence
/// was visible. Naming pairs by channel keeps those tests independent of batch
/// ordering.
#[derive(Clone, Copy, Debug)]
pub struct ActionLayerMovePair {
    /// Left channel name in the derivation-case list.
    pub left: &'static str,
    /// Right channel name in the derivation-case list.
    pub right: &'static str,
    /// Human-readable pair label appended to assertion failures.
    pub label: &'static str,
}

/// A named derivation matrix plus the same-action pairs that must move.
///
/// Standard matrix probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// always carry these two pieces together: the
/// ordered channel cases and the selected liveness pairs. Bundling them avoids
/// call sites accidentally pairing one matrix's cases with another matrix's
/// liveness assertions.
#[derive(Clone, Copy, Debug)]
pub struct DerivationMatrix<'a> {
    /// Cases for a single same-observation derivation batch.
    pub cases: &'a [ChannelDerivationCase<'a>],
    /// Same-action-shape pairs whose action layer should move.
    pub move_pairs: &'a [ActionLayerMovePair],
}

impl<'a> DerivationMatrix<'a> {
    /// Bundle one derivation matrix's case and liveness metadata.
    #[must_use]
    pub const fn new(cases: &'a [ChannelDerivationCase<'a>], move_pairs: &'a [ActionLayerMovePair]) -> Self {
        Self { cases, move_pairs }
    }
}

/// Ordered case lists and liveness pairs for a request-order matrix probe.
///
/// Matrix request-order tests need the same three pieces of metadata every
/// time: the forward cases, the reordered cases, and the selected same-shape
/// pairs that must prove the action layer is still live. Bundling them keeps
/// decision-policy and challenge/reward wrappers from passing parallel lists
/// through several helper layers.
#[derive(Clone, Copy, Debug)]
pub struct DerivationMatrixRequestOrder<'a> {
    /// Cases for the forward batch.
    pub forward_cases: &'a [ChannelDerivationCase<'a>],
    /// Cases for the reordered batch.
    pub reordered_cases: &'a [ChannelDerivationCase<'a>],
    /// Same-action-shape pairs whose action layer should move.
    pub move_pairs: &'a [ActionLayerMovePair],
}

/// Which stable payload should be compared when replaying two derivation batches.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DerivationReplayComparison {
    /// Compare the full public core assessment plus resonance profile.
    FullProfile,
    /// Compare only the sufficient public risk statistic plus resonance profile.
    RiskBasis,
}

impl<'a> DerivationMatrixRequestOrder<'a> {
    /// Bundle a matrix request-order probe's case and liveness metadata.
    #[must_use]
    pub const fn new(
        forward_cases: &'a [ChannelDerivationCase<'a>],
        reordered_cases: &'a [ChannelDerivationCase<'a>],
        move_pairs: &'a [ActionLayerMovePair],
    ) -> Self {
        Self {
            forward_cases,
            reordered_cases,
            move_pairs,
        }
    }
}

/// Standard three-channel fixture for the core/decision independence tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub const STANDARD_DERIVATION_CASES: [ChannelDerivationCase<'static>; 3] = [
    ChannelDerivationCase {
        name: "login",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "api",
        action_tags: &API_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "transaction",
        action_tags: &TRANSACTION_ACTION_TAGS,
    },
];

/// Reverse order for the standard three-channel request-order probe
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´).
pub const REVERSED_STANDARD_DERIVATION_CASES: [ChannelDerivationCase<'static>; 3] = [
    ChannelDerivationCase {
        name: "transaction",
        action_tags: &TRANSACTION_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "api",
        action_tags: &API_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "login",
        action_tags: &LOGIN_ACTION_TAGS,
    },
];

const STANDARD_DERIVATION_MOVE_PAIRS: [ActionLayerMovePair; 0] = [];

/// Standard three-channel fixture as a derivation matrix.
///
/// Unlike the policy and challenge/reward matrices, this fixture intentionally
/// has no same-action liveness pairs: the action sets differ by policy shape,
/// so fixed-core derivation and action-tag sequence checks are the liveness
/// witness.
pub const STANDARD_DERIVATION_MATRIX: DerivationMatrix<'static> =
    DerivationMatrix::new(&STANDARD_DERIVATION_CASES, &STANDARD_DERIVATION_MOVE_PAIRS);

/// Forward/reversed request-order metadata for the standard three-channel probe
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´).
pub const STANDARD_DERIVATION_MATRIX_REQUEST_ORDER: DerivationMatrixRequestOrder<'static> = DerivationMatrixRequestOrder::new(
    &STANDARD_DERIVATION_CASES,
    &REVERSED_STANDARD_DERIVATION_CASES,
    &STANDARD_DERIVATION_MOVE_PAIRS,
);

/// Four-channel fixture for the policy-matrix probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
///
/// The matrix combines action-set shape (`api`, `login`, `transaction_high_cost`)
/// with a same-shape reward perturbation (`login` vs `login_high_cost`).
pub const DECISION_POLICY_MATRIX_CASES: [ChannelDerivationCase<'static>; 4] = [
    ChannelDerivationCase {
        name: "login",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "login_high_cost",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "api",
        action_tags: &API_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "transaction_high_cost",
        action_tags: &TRANSACTION_ACTION_TAGS,
    },
];

/// Reverse order for the standard policy-matrix request-order probe
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´).
pub const REVERSED_DECISION_POLICY_MATRIX_CASES: [ChannelDerivationCase<'static>; 4] = [
    ChannelDerivationCase {
        name: "transaction_high_cost",
        action_tags: &TRANSACTION_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "api",
        action_tags: &API_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "login_high_cost",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "login",
        action_tags: &LOGIN_ACTION_TAGS,
    },
];

pub const DECISION_POLICY_MATRIX_MOVE_PAIRS: [ActionLayerMovePair; 1] = [ActionLayerMovePair {
    left: "login",
    right: "login_high_cost",
    label: "same action set reward perturbation",
}];

pub const DECISION_POLICY_MATRIX: DerivationMatrix<'static> =
    DerivationMatrix::new(&DECISION_POLICY_MATRIX_CASES, &DECISION_POLICY_MATRIX_MOVE_PAIRS);

pub const DECISION_POLICY_MATRIX_REQUEST_ORDER: DerivationMatrixRequestOrder<'static> = DerivationMatrixRequestOrder::new(
    &DECISION_POLICY_MATRIX_CASES,
    &REVERSED_DECISION_POLICY_MATRIX_CASES,
    &DECISION_POLICY_MATRIX_MOVE_PAIRS,
);

/// Three-channel fixture for challenge-tracker + reward composition probes.
///
/// `challenged` and `untouched` share policy so companion-tracker state is the
/// only moving part. `challenged_high_cost` receives the same challenge evidence
/// as `challenged` but with perturbed rewards, isolating reward liveness.
pub const CHALLENGE_REWARD_POLICY_MATRIX_CASES: [ChannelDerivationCase<'static>; 3] = [
    ChannelDerivationCase {
        name: "challenged",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "challenged_high_cost",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "untouched",
        action_tags: &LOGIN_ACTION_TAGS,
    },
];

/// Reverse order for the standard challenge-tracker and reward matrix
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´).
pub const REVERSED_CHALLENGE_REWARD_POLICY_MATRIX_CASES: [ChannelDerivationCase<'static>; 3] = [
    ChannelDerivationCase {
        name: "untouched",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "challenged_high_cost",
        action_tags: &LOGIN_ACTION_TAGS,
    },
    ChannelDerivationCase {
        name: "challenged",
        action_tags: &LOGIN_ACTION_TAGS,
    },
];

pub const CHALLENGE_REWARD_POLICY_MATRIX_MOVE_PAIRS: [ActionLayerMovePair; 2] = [
    ActionLayerMovePair {
        left: "challenged",
        right: "untouched",
        label: "challenge tracker evidence",
    },
    ActionLayerMovePair {
        left: "challenged",
        right: "challenged_high_cost",
        label: "reward perturbation under matched challenge evidence",
    },
];

pub const CHALLENGE_REWARD_POLICY_MATRIX: DerivationMatrix<'static> = DerivationMatrix::new(
    &CHALLENGE_REWARD_POLICY_MATRIX_CASES,
    &CHALLENGE_REWARD_POLICY_MATRIX_MOVE_PAIRS,
);

pub const CHALLENGE_REWARD_POLICY_MATRIX_REQUEST_ORDER: DerivationMatrixRequestOrder<'static> = DerivationMatrixRequestOrder::new(
    &CHALLENGE_REWARD_POLICY_MATRIX_CASES,
    &REVERSED_CHALLENGE_REWARD_POLICY_MATRIX_CASES,
    &CHALLENGE_REWARD_POLICY_MATRIX_MOVE_PAIRS,
);

/// Build two same-action-shape derivation cases.
///
/// Independence tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// often compare two channels that intentionally share the same action set
/// while differing in one decision-layer input. This helper
/// keeps those tests focused on the varying input instead of repeating the
/// same pair of [`ChannelDerivationCase`] literals.
#[must_use]
pub const fn channel_pair_cases<'a>(
    left: &'static str,
    right: &'static str,
    action_tags: &'a [Tag],
) -> [ChannelDerivationCase<'a>; 2] {
    [
        ChannelDerivationCase { name: left, action_tags },
        ChannelDerivationCase {
            name: right,
            action_tags,
        },
    ]
}

/// Register matching live Sentinel names through distinct lifecycle histories.
///
/// The public sufficiency probes
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´)
/// compare two worlds whose published core statistic matches even though one world has lifecycle churn in its history.
/// Both worlds finish with a live `S1`, but the cycled world first registers
/// and deregisters a retired Sentinel so the final live ids differ.
#[track_caller]
pub fn register_distinct_live_sentinel_states(
    fresh_world: &mut Scenario,
    cycled_world: &mut Scenario,
    label: &str,
) -> (SentinelId, SentinelId) {
    let fresh_id = fresh_world
        .register_sentinel("S1")
        .unwrap_or_else(|error| panic!("{label}: register fresh Sentinel failed: {error:?}"));
    cycled_world
        .register_sentinel("retired")
        .unwrap_or_else(|error| panic!("{label}: register retired Sentinel failed: {error:?}"));
    cycled_world
        .deregister_sentinel("retired")
        .unwrap_or_else(|error| panic!("{label}: deregister retired Sentinel failed: {error:?}"));
    let cycled_id = cycled_world
        .register_sentinel("S1")
        .unwrap_or_else(|error| panic!("{label}: register cycled Sentinel failed: {error:?}"));

    assert_ne!(
        fresh_id, cycled_id,
        "{label}: fixture worlds should have distinct live Sentinel ids"
    );
    assert!(
        fresh_world.sentinel("S1").is_some(),
        "{label}: fresh world should have a live Sentinel",
    );
    assert!(
        cycled_world.sentinel("S1").is_some(),
        "{label}: cycled world should have a live Sentinel after lifecycle churn",
    );

    fresh_world
        .flush_labels()
        .unwrap_or_else(|error| panic!("{label}: fresh lifecycle barrier failed: {error:?}"));
    cycled_world
        .flush_labels()
        .unwrap_or_else(|error| panic!("{label}: cycled lifecycle barrier failed: {error:?}"));

    (fresh_id, cycled_id)
}

/// Register matching live Sentinel names through distinct lifecycle histories
/// and ingest the canonical golden report in both worlds.
///
/// The returned ids are intentionally different. The reported sufficiency probes
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´)
/// use
/// them to assert each world's rich core payload is live, then compare the
/// decision layer from the shared public `RiskBasis` because the per-Sentinel
/// payload keys are world-local by design.
#[track_caller]
pub fn register_distinct_golden_reporting_sentinel_states(
    fresh_world: &mut Scenario,
    cycled_world: &mut Scenario,
    label: &str,
) -> (SentinelId, SentinelId) {
    let (fresh_id, cycled_id) = register_distinct_live_sentinel_states(fresh_world, cycled_world, label);

    refresh_golden_report(fresh_world, "S1");
    refresh_golden_report(cycled_world, "S1");

    (fresh_id, cycled_id)
}

/// Drift-aware companion to [`assert_assessments_derive_from_fixed_core`].
#[track_caller]
pub fn assert_assessments_derive_from_fixed_core_with_drift(cases: &[ObservedDerivationCase<'_>], drift: f64, label: &str) {
    assert!(!cases.is_empty(), "{label}: at least one observed case is required");

    for case in cases {
        assert_reckoning_well_formed(case.reckoning, &format!("{label}: {} reckoning", case.name));
        assert_eq!(
            action_tag_count(case.reckoning),
            case.action_tags.len(),
            "{label}: {} action-tag count",
            case.name,
        );
        assert_action_tag_sequence(case.reckoning, case.action_tags, &format!("{label}: {}", case.name));
    }

    let first = cases[0];
    for case in cases.iter().skip(1) {
        assert_core_assessment_near_with_drift(
            first.reckoning,
            case.reckoning,
            drift,
            &format!("{label}: core assessment first vs {}", case.name),
        );
        assert_classification_tags_near_with_drift(
            first.reckoning,
            case.reckoning,
            drift,
            &format!("{label}: classification tags first vs {}", case.name),
        );
    }
}

/// Drift-aware companion to [`assert_requests_derive_from_fixed_core`].
///
/// One held assessment, many policies. The shared observation — `requests[0]`,
/// which every entry of `requests` repeats with only its channel changed — is
/// assessed once, and each case derives from that single assessment.
///
/// Submitting the batch and comparing the results would measure something the
/// engine does not promise. A reader loads the published snapshot once per
/// request rather than once per batch (´dec:concurrency:per-request-load´), and
/// an accepted observation advances a later one
/// (´dec:ordering:score-before-evolve´), so two requests in one batch can be
/// answered in coordinate systems one advance apart. A batch therefore does not
/// share a coordinate state, and a family that assumed it did was reading the
/// steward's scheduling rather than the derivation
/// (´inv:guarantee:evidence-authority´). Holding the assessment removes the
/// coordinate change from the comparison instead of waiting for it to stop.
///
/// What the fixed-core assertion still carries once the assessment is held:
/// derivation must pass the core through untouched rather than reassess, the
/// classification tags must be identical across policies, and each channel must
/// produce its own action-tag shape. The core assessment is a host's to hold and
/// re-derive against several channels
/// (´claim:api:core-assessment-needs-no-channel-because-policy-belongs-to-derivation´),
/// so this is the arrangement the surface was designed for.
#[track_caller]
pub fn assert_requests_derive_from_fixed_core_with_drift(
    world: &World,
    requests: &[RequestContext],
    cases: &[ChannelDerivationCase<'_>],
    drift: f64,
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_eq!(requests.len(), cases.len(), "{label}: request/case count mismatch");
    let shared = requests
        .first()
        .unwrap_or_else(|| panic!("{label}: at least one request is required"));

    let assessment = world.assess(shared.clone());
    let assessments = derive_cases_from_one_assessment(world, &assessment, cases);

    let observed: Vec<_> = cases
        .iter()
        .zip(&assessments)
        .map(|(case, reckoning)| ObservedDerivationCase {
            name: case.name,
            reckoning,
            action_tags: case.action_tags,
        })
        .collect();
    assert_assessments_derive_from_fixed_core_with_drift(&observed, drift, label);

    assessments
}

/// Submit a prepared request batch, assert fixed-core derivation, and check the
/// expected public core payload.
///
/// This is the rich-payload companion to [`assert_requests_derive_from_fixed_core`].
/// Use it when a same-observation batch must prove that a live Sentinel report,
/// outcome prediction, or signal-degradation footprint was actually present in
/// the shared core assessment.
#[track_caller]
pub fn assert_requests_derive_from_fixed_core_with_core_payload(
    world: &World,
    requests: &[RequestContext],
    cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_requests_derive_from_fixed_core_with_core_payload_and_drift(world, requests, cases, core_payload, PURITY_DRIFT, label)
}

/// Drift-aware companion to [`assert_requests_derive_from_fixed_core_with_core_payload`].
#[track_caller]
pub fn assert_requests_derive_from_fixed_core_with_core_payload_and_drift(
    world: &World,
    requests: &[RequestContext],
    cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    drift: f64,
    label: &str,
) -> Vec<DerivedReckoning> {
    let assessments = assert_requests_derive_from_fixed_core_with_drift(world, requests, cases, drift, label);
    assert_core_payload_for_derivation_cases_with_drift(cases, &assessments, core_payload, drift, label);
    assessments
}

/// Build plain named-channel requests for an ordered derivation-case list.
///
/// This is useful for request-order probes: the case list is the source of
/// truth for both channel names and expected action-tag sequences, while the
/// request payload remains the same observation on every channel.
#[must_use]
pub fn requests_for_derivation_cases(world: &World, entity: &str, cases: &[ChannelDerivationCase<'_>]) -> Vec<RequestContext> {
    cases.iter().map(|case| world.request(case.name, entity)).collect()
}

/// Derive every case's reckoning from ONE core assessment.
///
/// This is the derive-once-compare-channels primitive the pre-conditioned
/// pre-conditioned families
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// use: the cell's expected enrichment is derived once and
/// every channel is compared against that single derivation, so the
/// cross-channel comparison cannot straddle a model-owner republish.
#[must_use]
pub fn derive_cases_from_one_assessment(
    world: &World,
    assessment: &RiskAssessment,
    cases: &[ChannelDerivationCase<'_>],
) -> Vec<DerivedReckoning> {
    cases.iter().map(|case| world.derive(assessment, case.name)).collect()
}

/// Assert a derivation matrix derived once: one core assessment per cell,
/// every channel derived from it and checked against it.
#[track_caller]
pub fn assert_derivation_matrix_derives_once_with_drift(
    world: &World,
    request: RequestContext,
    matrix: DerivationMatrix<'_>,
    core_payload: CorePayloadExpectation,
    drift: f64,
    label: &str,
) -> Vec<DerivedReckoning> {
    let assessment = world.assess(request);
    let assessments = derive_cases_from_one_assessment(world, &assessment, matrix.cases);

    let observed: Vec<_> = matrix
        .cases
        .iter()
        .zip(&assessments)
        .map(|(case, reckoning)| ObservedDerivationCase {
            name: case.name,
            reckoning,
            action_tags: case.action_tags,
        })
        .collect();
    assert_assessments_derive_from_fixed_core_with_drift(&observed, drift, label);
    assert_core_payload_for_derivation_cases_with_drift(matrix.cases, &assessments, core_payload, drift, label);
    assert_named_action_layer_pairs_move(matrix.cases, &assessments, matrix.move_pairs, label);

    assessments
}

/// Assert a request-order matrix derived once: both orders derive from the
/// same single core assessment and must replay each channel by name.
#[track_caller]
pub fn assert_derivation_matrix_request_order_derives_once_with_comparison_and_drift(
    world: &World,
    request: RequestContext,
    matrix: DerivationMatrixRequestOrder<'_>,
    core_payload: CorePayloadExpectation,
    comparison: DerivationReplayComparison,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let assessment = world.assess(request);
    let (forward, reordered) = derive_case_orders_from_one_assessment(
        world,
        &assessment,
        matrix.forward_cases,
        matrix.reordered_cases,
        core_payload,
        drift,
        label,
    );

    match comparison {
        DerivationReplayComparison::FullProfile => {
            assert_derivation_batches_match_by_name(matrix.forward_cases, &forward, matrix.reordered_cases, &reordered, label);
        }
        DerivationReplayComparison::RiskBasis => assert_derivation_batches_replay_from_risk_basis_by_name(
            matrix.forward_cases,
            &forward,
            matrix.reordered_cases,
            &reordered,
            label,
        ),
    }

    assert_named_action_layer_pairs_move(
        matrix.forward_cases,
        &forward,
        matrix.move_pairs,
        &format!("{label}: forward matrix"),
    );
    assert_named_action_layer_pairs_move(
        matrix.reordered_cases,
        &reordered,
        matrix.move_pairs,
        &format!("{label}: reordered matrix"),
    );

    (forward, reordered)
}

/// Assert two channels derived from ONE core assessment move only the
/// action layer, with the expected core payload present on both.
#[track_caller]
pub fn assert_request_pair_moves_only_action_layer_derive_once(
    world: &World,
    request: RequestContext,
    cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_eq!(cases.len(), 2, "{label}: expected exactly two channel cases");

    let assessment = world.assess(request);
    let assessments = derive_cases_from_one_assessment(world, &assessment, cases);

    let observed: Vec<_> = cases
        .iter()
        .zip(&assessments)
        .map(|(case, reckoning)| ObservedDerivationCase {
            name: case.name,
            reckoning,
            action_tags: case.action_tags,
        })
        .collect();
    assert_assessments_derive_from_fixed_core_with_drift(&observed, PURITY_DRIFT, label);
    assert_core_payload_for_derivation_cases_with_drift(cases, &assessments, core_payload, PURITY_DRIFT, label);
    assert_action_layer_moves_without_core(&assessments[0], &assessments[1], label);

    assessments
}

/// Assert that two ordered derivation batches produce the same per-channel
/// core and resonance profile, independent of request order.
///
/// The two batches may list the channels in different orders. Each channel is
/// matched by case name; its action-tag expectation must be identical in both
/// batches, and the public stable payload (`RiskBasis` plus resonance profile)
/// must replay within [`PURITY_DRIFT`].
///
/// That budget is the function's, not a caller's. It was a caller's while
/// request-order probes carrying live Sentinel reports and outcome predictions
/// were held to be wider than floating-point residue; they are not, because
/// deriving both orders from one held assessment leaves nothing but arithmetic
/// between them, and measurement under load retired the wider budgets. What was
/// left afterwards was a parameter every caller filled with the same constant,
/// which reads as an open question about the shape under test where there is
/// none.
#[track_caller]
pub fn assert_derivation_batches_match_by_name(
    left_cases: &[ChannelDerivationCase<'_>],
    left_assessments: &[DerivedReckoning],
    right_cases: &[ChannelDerivationCase<'_>],
    right_assessments: &[DerivedReckoning],
    label: &str,
) {
    assert_eq!(
        left_cases.len(),
        left_assessments.len(),
        "{label}: left case/reckoning count mismatch",
    );
    assert_eq!(
        right_cases.len(),
        right_assessments.len(),
        "{label}: right case/reckoning count mismatch",
    );
    assert_eq!(left_cases.len(), right_cases.len(), "{label}: batch case count mismatch");

    for (left_index, left_case) in left_cases.iter().enumerate() {
        let Some((right_index, right_case)) = right_cases
            .iter()
            .enumerate()
            .find(|(_, right_case)| right_case.name == left_case.name)
        else {
            panic!("{label}: no right-batch case named {}", left_case.name);
        };

        assert_eq!(
            left_case.action_tags, right_case.action_tags,
            "{label}: {} expected action tags changed between batches",
            left_case.name,
        );
        assert_core_and_resonance_profile_near_with_drift(
            &left_assessments[left_index],
            &right_assessments[right_index],
            PURITY_DRIFT,
            &format!("{label}: {}", left_case.name),
        );
    }
}

/// Assert that two ordered derivation batches replay the same per-channel
/// sufficient statistic and resonance profile while allowing richer core
/// payloads to differ.
///
/// Use this for request-order probes whose observations carry a core payload
/// richer than two batches can be expected to share: reported Sentinel
/// coordinates are world-local keys, and a live outcome prediction carries its
/// own uncertainty. Each batch proves that payload live on its own, as its
/// fixed-core check does; only the comparison across the two drops to the
/// sufficient statistic.
///
/// The budget is [`PURITY_DRIFT`], as it is for the full-profile comparison
/// beside it: dropping to the sufficient statistic narrows what is compared,
/// not how closely.
#[track_caller]
pub fn assert_derivation_batches_replay_from_risk_basis_by_name(
    left_cases: &[ChannelDerivationCase<'_>],
    left_assessments: &[DerivedReckoning],
    right_cases: &[ChannelDerivationCase<'_>],
    right_assessments: &[DerivedReckoning],
    label: &str,
) {
    assert_eq!(
        left_cases.len(),
        left_assessments.len(),
        "{label}: left case/reckoning count mismatch",
    );
    assert_eq!(
        right_cases.len(),
        right_assessments.len(),
        "{label}: right case/reckoning count mismatch",
    );
    assert_eq!(left_cases.len(), right_cases.len(), "{label}: batch case count mismatch");

    for (left_index, left_case) in left_cases.iter().enumerate() {
        let Some((right_index, right_case)) = right_cases
            .iter()
            .enumerate()
            .find(|(_, right_case)| right_case.name == left_case.name)
        else {
            panic!("{label}: no right-batch case named {}", left_case.name);
        };

        assert_eq!(
            left_case.action_tags, right_case.action_tags,
            "{label}: {} expected action tags changed between batches",
            left_case.name,
        );
        assert_risk_basis_and_resonance_profile_near_with_drift(
            &left_assessments[left_index],
            &right_assessments[right_index],
            PURITY_DRIFT,
            &format!("{label}: {}", left_case.name),
        );
    }
}

/// Derive two case orders from ONE held assessment, checking each order as a
/// fixed-core derivation carrying the expected core payload.
///
/// This is the shared body of the order-replay family. Both orders must come
/// from the same assessment or the comparison is not about order: assessing
/// each order separately would put the two in coordinate systems an accepted
/// observation apart, because the snapshot is loaded per request
/// (´dec:concurrency:per-request-load´) and an accepted observation advances a
/// later one (´dec:ordering:score-before-evolve´). The difference a reader would
/// then be measuring is the steward's, not the derivation's
/// (´inv:guarantee:evidence-authority´).
#[track_caller]
fn derive_case_orders_from_one_assessment(
    world: &World,
    assessment: &RiskAssessment,
    forward_cases: &[ChannelDerivationCase<'_>],
    reordered_cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let mut batches = Vec::with_capacity(2);
    for (cases, tag) in [(forward_cases, "forward batch"), (reordered_cases, "reordered batch")] {
        let batch = derive_cases_from_one_assessment(world, assessment, cases);
        let observed: Vec<_> = cases
            .iter()
            .zip(&batch)
            .map(|(case, reckoning)| ObservedDerivationCase {
                name: case.name,
                reckoning,
                action_tags: case.action_tags,
            })
            .collect();
        assert_assessments_derive_from_fixed_core_with_drift(&observed, drift, &format!("{label}: {tag}"));
        assert_core_payload_for_derivation_cases_with_drift(cases, &batch, core_payload, drift, &format!("{label}: {tag}"));
        batches.push(batch);
    }

    let reordered = batches.pop().expect("reordered batch");
    let forward = batches.pop().expect("forward batch");
    (forward, reordered)
}

/// Assert that every named channel replays its own core and resonance profile
/// whichever order the case list puts it in.
///
/// This is the rich order-replay primitive
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// for probes whose observations
/// carry request-scoped signals, reported Sentinel coordinates, outcome
/// predictions, or other public payload. The shared observation —
/// `forward_requests[0]` — is assessed once; each order is then checked as a
/// fixed-core derivation on its own, and channels are matched by name across
/// the two to prove the order left no residue.
///
/// The budget is [`PURITY_DRIFT`] and no caller chooses it: no fixture in this
/// suite drifts beyond the purity floor under request-order reversal. Reported
/// Sentinel payloads and live outcome predictions were the last held to a wider
/// budget, and those budgets were measured away once both orders came to derive
/// from one held assessment.
#[track_caller]
pub fn assert_request_order_replays_by_name_with_requests(
    world: &World,
    forward_requests: &[RequestContext],
    forward_cases: &[ChannelDerivationCase<'_>],
    reordered_requests: &[RequestContext],
    reordered_cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    assert_eq!(
        forward_requests.len(),
        reordered_requests.len(),
        "{label}: forward/reordered request count mismatch"
    );
    let shared = forward_requests
        .first()
        .unwrap_or_else(|| panic!("{label}: at least one request is required"));

    let assessment = world.assess(shared.clone());
    let (forward, reordered) = derive_case_orders_from_one_assessment(
        world,
        &assessment,
        forward_cases,
        reordered_cases,
        core_payload,
        PURITY_DRIFT,
        label,
    );

    assert_derivation_batches_match_by_name(forward_cases, &forward, reordered_cases, &reordered, label);

    (forward, reordered)
}

/// Drift-aware cross-world full-profile replay helper.
#[track_caller]
pub fn assert_world_pair_derivation_batches_replay_profile_with_drift(
    left: WorldDerivationBatch<'_>,
    right: WorldDerivationBatch<'_>,
    cases: &[ChannelDerivationCase<'_>],
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let left_assessments = assert_requests_derive_from_fixed_core_with_core_payload(
        left.world,
        left.requests,
        cases,
        left.core_payload,
        &format!("{label}: left world"),
    );
    let right_assessments = assert_requests_derive_from_fixed_core_with_core_payload(
        right.world,
        right.requests,
        cases,
        right.core_payload,
        &format!("{label}: right world"),
    );

    for (index, case) in cases.iter().enumerate() {
        assert_core_and_resonance_profile_near_with_drift(
            &left_assessments[index],
            &right_assessments[index],
            drift,
            &format!("{label}: {} cross-world replay", case.name),
        );
    }

    (left_assessments, right_assessments)
}

/// Drift-aware cross-world full-profile matrix replay helper.
///
/// This keeps the selected action-layer liveness checks tied to the same
/// cross-world replay path while allowing lifecycle witnesses to pass their
/// current parallel-load drift budget explicitly.
#[track_caller]
pub fn assert_world_pair_derivation_matrix_replays_profile_with_drift(
    left: WorldDerivationBatch<'_>,
    right: WorldDerivationBatch<'_>,
    matrix: DerivationMatrix<'_>,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let (left_assessments, right_assessments) =
        assert_world_pair_derivation_batches_replay_profile_with_drift(left, right, matrix.cases, drift, label);

    assert_named_action_layer_pairs_move(
        matrix.cases,
        &left_assessments,
        matrix.move_pairs,
        &format!("{label}: left world"),
    );
    assert_named_action_layer_pairs_move(
        matrix.cases,
        &right_assessments,
        matrix.move_pairs,
        &format!("{label}: right world"),
    );

    (left_assessments, right_assessments)
}

/// Drift-aware cross-world replay from the same observed risk statistic.
#[track_caller]
pub fn assert_world_pair_derivation_batches_replay_from_risk_basis_with_drift(
    left: WorldDerivationBatch<'_>,
    right: WorldDerivationBatch<'_>,
    cases: &[ChannelDerivationCase<'_>],
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let left_assessments = assert_requests_derive_from_fixed_core_with_core_payload(
        left.world,
        left.requests,
        cases,
        left.core_payload,
        &format!("{label}: left world"),
    );
    let right_assessments = assert_requests_derive_from_fixed_core_with_core_payload(
        right.world,
        right.requests,
        cases,
        right.core_payload,
        &format!("{label}: right world"),
    );

    for (index, case) in cases.iter().enumerate() {
        assert_risk_basis_and_resonance_profile_near_with_drift(
            &left_assessments[index],
            &right_assessments[index],
            drift,
            &format!("{label}: {} cross-world replay", case.name),
        );
    }

    (left_assessments, right_assessments)
}

/// Drift-aware cross-world derivation matrix replay from the same `RiskBasis`.
#[track_caller]
pub fn assert_world_pair_derivation_matrix_replays_from_risk_basis_with_drift(
    left: WorldDerivationBatch<'_>,
    right: WorldDerivationBatch<'_>,
    matrix: DerivationMatrix<'_>,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let (left_assessments, right_assessments) =
        assert_world_pair_derivation_batches_replay_from_risk_basis_with_drift(left, right, matrix.cases, drift, label);

    assert_named_action_layer_pairs_move(
        matrix.cases,
        &left_assessments,
        matrix.move_pairs,
        &format!("{label}: left world"),
    );
    assert_named_action_layer_pairs_move(
        matrix.cases,
        &right_assessments,
        matrix.move_pairs,
        &format!("{label}: right world"),
    );

    (left_assessments, right_assessments)
}

/// Drift-aware cross-world matrix replay helper with caller-selected replay semantics.
#[track_caller]
pub fn assert_world_pair_derivation_matrix_replays_with_comparison_and_drift(
    left: WorldDerivationBatch<'_>,
    right: WorldDerivationBatch<'_>,
    matrix: DerivationMatrix<'_>,
    comparison: DerivationReplayComparison,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    match comparison {
        DerivationReplayComparison::FullProfile => {
            assert_world_pair_derivation_matrix_replays_profile_with_drift(left, right, matrix, drift, label)
        }
        DerivationReplayComparison::RiskBasis => {
            assert_world_pair_derivation_matrix_replays_from_risk_basis_with_drift(left, right, matrix, drift, label)
        }
    }
}

/// Drift-aware divergent-output matrix replay from the same risk statistic.
#[track_caller]
pub fn assert_divergent_outcome_matrix_replays_from_risk_basis_with_drift(
    positive: WorldDerivationBatch<'_>,
    negative: WorldDerivationBatch<'_>,
    axes: DivergentOutcomeAxisPair,
    matrix: DerivationMatrix<'_>,
    drift: f64,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let (positive_assessments, negative_assessments) =
        assert_world_pair_derivation_matrix_replays_from_risk_basis_with_drift(positive, negative, matrix, drift, label);

    let positive_first = positive_assessments
        .first()
        .unwrap_or_else(|| panic!("{label}: positive matrix produced no assessments"));
    let negative_first = negative_assessments
        .first()
        .unwrap_or_else(|| panic!("{label}: negative matrix produced no assessments"));
    assert_outcome_prediction_raw_ordered(
        positive_first,
        axes.positive,
        negative_first,
        axes.negative,
        1e-6,
        &format!("{label}: divergent outcome predictions"),
    );

    (positive_assessments, negative_assessments)
}

/// Request-order replay helper for two same-action-shape variants whose action
/// layer should move while the core stays fixed.
///
/// This wraps [`assert_request_order_replays_by_name`] with the liveness check
/// used by the reward and challenge-tracker probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
#[track_caller]
pub fn assert_request_pair_order_replays_only_action_layer(
    world: &World,
    entity: &str,
    forward_cases: &[ChannelDerivationCase<'_>],
    reordered_cases: &[ChannelDerivationCase<'_>],
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    let forward_requests = requests_for_derivation_cases(world, entity, forward_cases);
    let reordered_requests = requests_for_derivation_cases(world, entity, reordered_cases);

    assert_request_pair_order_replays_only_action_layer_with_requests(
        world,
        &forward_requests,
        forward_cases,
        &reordered_requests,
        reordered_cases,
        CorePayloadExpectation::default(),
        label,
    )
}

/// Request-order replay helper for two same-action-shape custom request batches
/// whose action layer should move while the core stays fixed.
///
/// This is the richer companion to
/// [`assert_request_pair_order_replays_only_action_layer`]. It lets request-order
/// tests exercise the same liveness property while carrying request-scoped
/// signals, reported Sentinel coordinates, outcome predictions, or expected
/// degradation diagnostics through the public request payload.
#[track_caller]
pub fn assert_request_pair_order_replays_only_action_layer_with_requests(
    world: &World,
    forward_requests: &[RequestContext],
    forward_cases: &[ChannelDerivationCase<'_>],
    reordered_requests: &[RequestContext],
    reordered_cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    label: &str,
) -> (Vec<DerivedReckoning>, Vec<DerivedReckoning>) {
    assert_eq!(forward_cases.len(), 2, "{label}: expected exactly two forward cases");
    assert_eq!(reordered_cases.len(), 2, "{label}: expected exactly two reordered cases");

    let (forward, reordered) = assert_request_order_replays_by_name_with_requests(
        world,
        forward_requests,
        forward_cases,
        reordered_requests,
        reordered_cases,
        core_payload,
        label,
    );
    assert_action_layer_moves_without_core(&forward[0], &forward[1], &format!("{label}: forward action layer"));
    assert_action_layer_moves_without_core(&reordered[0], &reordered[1], &format!("{label}: reordered action layer"));

    (forward, reordered)
}

/// Assert named same-action-shape pairs from a derivation matrix move only the
/// action layer.
///
/// Call this after [`assert_requests_derive_from_fixed_core`] when a test has
/// already proven the whole batch shares one core fingerprint and now wants to
/// prove selected decision-layer inputs are live. Pair lookup is by channel
/// name, so request-order probes and larger channel matrices can reuse the same
/// assertion without relying on positional indices in the test body.
#[track_caller]
pub fn assert_named_action_layer_pairs_move(
    cases: &[ChannelDerivationCase<'_>],
    assessments: &[DerivedReckoning],
    pairs: &[ActionLayerMovePair],
    label: &str,
) {
    assert_eq!(cases.len(), assessments.len(), "{label}: case/reckoning count mismatch");

    for pair in pairs {
        let left_index = cases
            .iter()
            .position(|case| case.name == pair.left)
            .unwrap_or_else(|| panic!("{label}: no case named {}", pair.left));
        let right_index = cases
            .iter()
            .position(|case| case.name == pair.right)
            .unwrap_or_else(|| panic!("{label}: no case named {}", pair.right));

        assert_action_layer_moves_without_core(
            &assessments[left_index],
            &assessments[right_index],
            &format!("{label}: {}", pair.label),
        );
    }
}

/// Drift-aware companion to [`assert_core_payload_for_derivation_cases`].
#[track_caller]
pub fn assert_core_payload_for_derivation_cases_with_drift(
    cases: &[ChannelDerivationCase<'_>],
    assessments: &[DerivedReckoning],
    expectation: CorePayloadExpectation,
    drift: f64,
    label: &str,
) {
    assert_eq!(cases.len(), assessments.len(), "{label}: case/reckoning count mismatch");

    for (case, reckoning) in cases.iter().zip(assessments) {
        assert_core_payload_expectation(reckoning, expectation, &format!("{label}: {}", case.name));
    }

    if let Some(axis) = expectation.outcome_axis {
        assert_outcome_prediction_shared_by_all_with_drift(assessments, axis, drift, label);
    }
}

/// Assert that two request variants share the same core assessment while
/// the action side of the derivation layer moves.
///
/// This is the common paired-request fixture shape
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´):
/// two requests have the same observation and the same action-tag shape, but
/// differ in a decision-layer input such as reward parameters or per-channel
/// challenge effectiveness.
/// The helper keeps the batch submission, structural checks, core-invariance
/// checks, and action-layer liveness assertion in one place.
#[track_caller]
pub fn assert_request_pair_moves_only_action_layer(
    world: &World,
    requests: &[RequestContext],
    cases: &[ChannelDerivationCase<'_>],
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_request_pair_moves_only_action_layer_with_core_payload(
        world,
        requests,
        cases,
        CorePayloadExpectation::default(),
        label,
    )
}

/// Assert that two request variants share the same core assessment and expected
/// core payload while the action side of the derivation layer moves.
///
/// This is the rich-core companion to
/// [`assert_request_pair_moves_only_action_layer`]. Use it when the same
/// two-channel derivation probe must also prove that reported Sentinel alarms,
/// outcome predictions, or degraded-signal diagnostics were part of the shared
/// core payload.
#[track_caller]
pub fn assert_request_pair_moves_only_action_layer_with_core_payload(
    world: &World,
    requests: &[RequestContext],
    cases: &[ChannelDerivationCase<'_>],
    core_payload: CorePayloadExpectation,
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_eq!(requests.len(), 2, "{label}: expected exactly two request variants");
    assert_eq!(cases.len(), 2, "{label}: expected exactly two channel cases");

    let assessments = assert_requests_derive_from_fixed_core_with_core_payload(world, requests, cases, core_payload, label);
    assert_action_layer_moves_without_core(&assessments[0], &assessments[1], label);
    assessments
}

/// Convenience wrapper for the common case where the two variants are plain
/// named-channel requests for the same entity.
#[track_caller]
pub fn assert_channel_pair_moves_only_action_layer(
    world: &World,
    baseline_channel: &'static str,
    variant_channel: &'static str,
    entity: &str,
    action_tags: &[Tag],
    label: &str,
) -> Vec<DerivedReckoning> {
    let requests = [
        world.request(baseline_channel, entity),
        world.request(variant_channel, entity),
    ];
    let cases = [
        ChannelDerivationCase {
            name: baseline_channel,
            action_tags,
        },
        ChannelDerivationCase {
            name: variant_channel,
            action_tags,
        },
    ];

    assert_request_pair_moves_only_action_layer(world, &requests, &cases, label)
}

/// Assert that two same-policy channel aliases replay both the core assessment
/// and the complete resonance profile for the same observation.
///
/// This is the neutral-channel counterpart to
/// [`assert_channel_pair_moves_only_action_layer`]. It is useful when a test
/// first feeds labels through one channel and then needs to prove that the
/// channel's companion decision-layer state stayed at the same value as an
/// untouched alias.
#[track_caller]
pub fn assert_channel_pair_replays_core_and_resonance_profile(
    world: &World,
    left_channel: &'static str,
    right_channel: &'static str,
    entity: &str,
    action_tags: &[Tag],
    label: &str,
) -> Vec<DerivedReckoning> {
    assert_channel_pair_replays_core_and_resonance_profile_with_drift(
        world,
        left_channel,
        right_channel,
        entity,
        action_tags,
        PURITY_DRIFT,
        label,
    )
}

/// Drift-aware companion to
/// [`assert_channel_pair_replays_core_and_resonance_profile`] for comparisons
/// with an independently justified numerical allowance.
#[track_caller]
pub fn assert_channel_pair_replays_core_and_resonance_profile_with_drift(
    world: &World,
    left_channel: &'static str,
    right_channel: &'static str,
    entity: &str,
    action_tags: &[Tag],
    drift: f64,
    label: &str,
) -> Vec<DerivedReckoning> {
    let requests = [world.request(left_channel, entity), world.request(right_channel, entity)];
    let cases = channel_pair_cases(left_channel, right_channel, action_tags);
    let assessments = assert_requests_derive_from_fixed_core_with_drift(world, &requests, &cases, drift, label);
    assert_core_and_resonance_profile_near_with_drift(&assessments[0], &assessments[1], drift, label);
    assessments
}

/// Build the standard three-channel independence scenario
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
///
/// The fixture declares:
///
/// - `login`: `[Allow, Challenge, Block]`
/// - `api`: `[Allow, Block]`
/// - `transaction`: `[Allow, Challenge, Slow, Block]`
///
/// All three channels use default reward parameters, so action-set
/// shape is the only decision-layer variable.
pub fn multi_channel_independence_scenario(instance_id: &str, seed: u64) -> Result<Scenario, WorldBuildError> {
    multi_channel_independence_scenario_with(instance_id, seed, |builder| builder)
}

/// Build the standard three-channel independence scenario with shared setup
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub fn multi_channel_independence_scenario_with<G>(
    instance_id: &str,
    seed: u64,
    configure: G,
) -> Result<Scenario, WorldBuildError>
where
    G: FnOnce(WorldBuilder) -> WorldBuilder,
{
    scenario_with(instance_id, seed, |builder| {
        configure(builder)
            .channel("login", policy_with_actions(THREE_ACTIONS))
            .channel("api", policy_with_actions(TWO_ACTIONS))
            .channel("transaction", policy_with_actions(FOUR_ACTIONS))
    })
}

/// Build the standard policy matrix
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub fn decision_policy_matrix_scenario(instance_id: &str, seed: u64) -> Result<Scenario, WorldBuildError> {
    decision_policy_matrix_scenario_with(instance_id, seed, |builder| builder)
}

/// Build the standard policy matrix with shared world setup
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
pub fn decision_policy_matrix_scenario_with<G>(instance_id: &str, seed: u64, configure: G) -> Result<Scenario, WorldBuildError>
where
    G: FnOnce(WorldBuilder) -> WorldBuilder,
{
    scenario_with(instance_id, seed, |builder| {
        configure(builder)
            .channel("login", policy_with(THREE_ACTIONS, RewardParameters::default()))
            .channel("login_high_cost", policy_with(THREE_ACTIONS, high_decision_cost_reward()))
            .channel("api", policy_with(TWO_ACTIONS, RewardParameters::default()))
            .channel(
                "transaction_high_cost",
                policy_with(FOUR_ACTIONS, high_decision_cost_reward()),
            )
    })
}

/// Build the standard challenge-tracker and reward policy matrix
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´).
pub fn challenge_reward_policy_matrix_scenario(instance_id: &str, seed: u64) -> Result<Scenario, WorldBuildError> {
    challenge_reward_policy_matrix_scenario_with(instance_id, seed, |builder| builder)
}

/// Build the standard challenge-tracker and reward policy matrix with shared
/// setup
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´).
pub fn challenge_reward_policy_matrix_scenario_with<G>(
    instance_id: &str,
    seed: u64,
    configure: G,
) -> Result<Scenario, WorldBuildError>
where
    G: FnOnce(WorldBuilder) -> WorldBuilder,
{
    scenario_with(instance_id, seed, |builder| {
        let policy = policy_with_actions(THREE_ACTIONS);
        configure(builder)
            .channel("challenged", policy.clone())
            .channel(
                "challenged_high_cost",
                policy_with(THREE_ACTIONS, high_decision_cost_reward()),
            )
            .channel("untouched", policy)
    })
}

/// Train the challenge/reward matrix Fail-evidence channels with custom requests.
///
/// Use this variant when the training path must also carry request-scoped
/// signals, reported Sentinel coordinates, or other public payload.
#[track_caller]
pub fn train_challenge_reward_policy_matrix_fail_evidence_with_requests<F>(
    world: &World,
    cycles: usize,
    mut request_for_channel_cycle: F,
    label: &str,
) where
    F: FnMut(&World, &'static str, usize) -> RequestContext,
{
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    for cycle in 0..cycles {
        for channel in ["challenged", "challenged_high_cost"] {
            cycle_request(world, request_for_channel_cycle(world, channel, cycle), with_fail);
        }
    }
}

/// Train one channel-local challenge tracker with Fail evidence.
///
/// This is the single-channel companion to
/// [`train_challenge_reward_policy_matrix_fail_evidence_with_requests`]. Use it
/// for the probes
/// (´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´)
/// that compare one challenged channel against an untouched same-policy alias while still needing custom request payloads such as
/// request-scoped signals or reported Sentinel coordinates.
#[track_caller]
pub fn train_channel_challenge_fail_evidence_with_requests<F>(
    world: &World,
    channel: &'static str,
    cycles: usize,
    mut request_for_cycle: F,
    label: &str,
) where
    F: FnMut(&World, &'static str, usize) -> RequestContext,
{
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    for cycle in 0..cycles {
        cycle_request(world, request_for_cycle(world, channel, cycle), with_fail);
    }
}

/// Train paired channels with present Fail evidence versus absent challenge evidence.
///
/// This is the same-world public fixture for the current `Some(Fail)` vs `None`
/// semantics
/// (´claim:channel:a-result-free-challenge-label-leaves-the-companion-tracker-untouched´):
/// both channels contribute to one shared core history, but only the
/// present-result channel feeds the companion tracker. Callers provide
/// the request shape so the same loop can carry plain, signal-rich, degraded,
/// or reported-Sentinel payloads.
#[track_caller]
pub fn train_channel_challenge_fail_vs_absent_evidence_with_requests<F>(
    world: &World,
    fail_channel: &'static str,
    absent_channel: &'static str,
    cycles: usize,
    mut request_for_channel_cycle: F,
    label: &str,
) where
    F: FnMut(&World, &'static str, usize) -> RequestContext,
{
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let fail_result = adverse_challenge_result(ChallengeResult::Fail);
    let absent_result = adverse_challenge_label(None);
    for cycle in 0..cycles {
        cycle_request(world, request_for_channel_cycle(world, fail_channel, cycle), fail_result);
        cycle_request(world, request_for_channel_cycle(world, absent_channel, cycle), absent_result);
    }
}

/// Train plain `(channel, entity)` Fail-vs-absent challenge evidence.
#[track_caller]
pub fn train_channel_challenge_fail_vs_absent_evidence(
    world: &World,
    fail_channel: &'static str,
    absent_channel: &'static str,
    entity: &'static str,
    cycles: usize,
    label: &str,
) {
    train_channel_challenge_fail_vs_absent_evidence_with_requests(
        world,
        fail_channel,
        absent_channel,
        cycles,
        |world, channel, _cycle| world.request(channel, entity),
        label,
    );
}

/// Train matched worlds with challenge/reward Fail evidence using custom requests.
///
/// The request builder is shared by both worlds so rich public payloads stay
/// matched while the helper owns the repeated challenged/challenged-high-cost
/// evidence loop on each side.
#[track_caller]
pub fn train_challenge_reward_policy_matrix_fail_evidence_pair_with_requests<F>(
    positive_world: &World,
    negative_world: &World,
    cycles: usize,
    request_for_channel_cycle: F,
    label: &str,
) where
    F: Copy + Fn(&World, &'static str, usize) -> RequestContext,
{
    assert!(cycles > 0, "{label}: at least one challenge-evidence cycle is required");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    for cycle in 0..cycles {
        for channel in ["challenged", "challenged_high_cost"] {
            cycle_request_pair(
                positive_world,
                request_for_channel_cycle(positive_world, channel, cycle),
                with_fail,
                negative_world,
                request_for_channel_cycle(negative_world, channel, cycle),
                with_fail,
                &format!("{label}: {channel} challenge/reward evidence cycle {cycle}"),
            );
        }
    }
}

/// Assess two matched custom requests and assert that only the action layer moves.
///
/// This is the common final assertion
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// for two-world probes whose public
/// request shape carries more than just `(channel, entity)`: request-scoped
/// signals, live Sentinel coordinates, and future fixture tapes all reuse this
/// one comparison once the worlds have seen matched core histories.
#[track_caller]
pub fn assert_world_pair_request_action_layer_moves_without_core(
    left_world: &World,
    left_request: RequestContext,
    right_world: &World,
    right_request: RequestContext,
    label: &str,
) -> (DerivedReckoning, DerivedReckoning) {
    // Both worlds settle their cold ramps first: this comparison is about two
    // lifecycle routes agreeing on a core, and a coordinate system still in
    // transition would have the two worlds disagreeing about how far along
    // their own ramps were rather than about their histories.
    left_world.settle_cold_ramp_with(std::slice::from_ref(&left_request));
    right_world.settle_cold_ramp_with(std::slice::from_ref(&right_request));

    let left = left_world
        .derive_for_request(left_request)
        .unwrap_or_else(|error| panic!("{label}: left final reckon failed: {error:?}"));
    let right = right_world
        .derive_for_request(right_request)
        .unwrap_or_else(|error| panic!("{label}: right final reckon failed: {error:?}"));

    assert_reckoning_well_formed(&left, &format!("{label}: left final reckoning"));
    assert_reckoning_well_formed(&right, &format!("{label}: right final reckoning"));
    assert_action_layer_moves_without_core(&left, &right, label);

    (left, right)
}

/// Assess two matched worlds and assert that only the action layer moves.
///
/// This is the common final assertion
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// for plain `(channel, entity)` probes:
/// same seed, same observation/label history, different decision-layer
/// configuration.
#[track_caller]
pub fn assert_world_pair_action_layer_moves_without_core(
    left_world: &World,
    right_world: &World,
    channel: &'static str,
    entity: &str,
    label: &str,
) -> (DerivedReckoning, DerivedReckoning) {
    assert_world_pair_request_action_layer_moves_without_core(
        left_world,
        left_world.request(channel, entity),
        right_world,
        right_world.request(channel, entity),
        label,
    )
}
