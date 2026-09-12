// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`one_core_assessment_derives_three_channel_policies`] | channel | A single core assessment is enough for as many channels as a host cares to ask about: one assessment of a login request derives reckonings for the login, API and transaction policies, each carrying that same assessment id, each with the action tags its own policy declares, and all three agreeing on their classification tags. The expensive part of a reckoning is paid once, so consulting a second channel costs a derivation and not another trip through the models. |
//! | [`three_channel_batch_core_independence`] | channel | The seam between the core model and the decision layer holds under every shape of request the public surface admits: across the whole enrichment grid — no signals, valid signals or deliberately degraded ones, crossed with a live reporting sentinel or none, crossed with a live outcome prediction or none — one entity assessed on three differently-shaped channels shares its risk basis and its classification tags, and differs only where the action sets differ. What a channel is for cannot bend what the engine believes about the entity behind it. |
//! | [`three_channel_request_order_replays`] | channel | Deriving one channel leaves nothing behind that changes the next: running the same three channels in reversed order reproduces each channel's reckoning by name, across the full enrichment grid. Without this a host's answer for a transaction would depend on whether it happened to consult the login channel first, and the batch a service issues would silently become part of its own input. |
//! | [`pre_conditioned_three_channel_batch`] | channel | Channel independence is a property of the architecture, not an artefact of an untrained engine. After sixteen alternating labels have moved the models on one channel, and again after eighteen labels routed through a registered identity dimension, the three channels still share one core across the whole enrichment grid. The separation being tested only matters once the engine has learned something, which is exactly when a leak would show. Every pre-conditioned cell derives once and compares channels: the cell's enrichment is assessed one time and every channel derives from that single assessment, so the comparison cannot straddle the model owner republishing mid-batch as it applies the history — a core disagreement in this family is a real leak from channel into core, never a race with the cell's own training. |
//! | [`pre_conditioned_three_channel_request_order`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`policy_matrix_core_independence`] | channel | cites (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´) |
//! | [`policy_matrix_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_policy_matrix`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`reward_perturbation_core_independence`] | channel | Two channels sharing an action set but priced differently — the cost of a miss doubled and the cost of friction quadrupled — reach different action tags from an identical risk basis and identical classification tags, across the whole enrichment grid. Rewards say what an outcome is worth to the operator, and an operator revising that valuation must be able to change what the system does without changing what it believes. |
//! | [`reward_axis_sweep`] | channel | cites (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´) |
//! | [`reward_axis_sweep_enriched_core_independence`] | channel | cites (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´) |
//! | [`reward_axis_sweep_enriched_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`challenge_reward_axis_sweep_enriched_core_independence`] | channel | cites (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´) |
//! | [`challenge_reward_axis_sweep_enriched_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`reward_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_reward_perturbation`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`two_world_reward_label_path`] | channel | Rewards stay out of the core on the way in as well as on the way out: two separately-built worlds seeded alike, one with default rewards and one with a perturbed block, are taught the same label stream through their own channels and end up agreeing on the core while differing in the action layer. The same holds when the requests carry signals and when the action sets are widened. Since a label is what actually moves the models, this is the path by which a reward could contaminate learning rather than merely colour a reading. |
//! | [`challenge_pass_fail_core_independence`] | channel | Whether an entity passed or failed a challenge changes what the decision layer recommends and not what the core believes: eight rounds of failures against eight rounds of passes leave the risk basis and classification tags in agreement while the action tags separate. This holds through five delivery shapes — plain, marked as ground truth, carried on valid signals, carried on deliberately degraded ones, and delivered via a live reporting sentinel. A challenge outcome is evidence about the challenge, and treating it as evidence about the entity would double-count the engine's own intervention. |
//! | [`challenge_tracker_channel_scope`] | channel | A companion tracker belongs to its channel and to nothing else: feeding eight failures to one channel moves that channel's action layer while an untouched sibling under the identical policy still shares the evolved core and stays at its own prior, across the whole enrichment grid. Whether challenges work is a fact about a particular channel's challenge mechanism, so a channel that never issues one must not inherit another's experience of it. |
//! | [`challenge_pass_vs_fail_shared_core`] | channel | cites (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´) |
//! | [`challenge_pass_vs_fail_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_challenge_pass_vs_fail_shared_core`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`challenge_tracker_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`challenge_fail_vs_absent_evidence`] | channel | cites (´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´) |
//! | [`challenge_fail_vs_absent_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`challenge_reward_matrix_core_independence`] | channel | cites (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´) |
//! | [`challenge_reward_matrix_request_order_replays`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_challenge_reward_matrix`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`divergent_outcome_standard_matrix`] | channel | An outcome prediction is something the engine reports, not something it reasons from: two worlds trained to predict opposite values on the same axis still replay the three-channel derivation from the same risk basis, across the enrichments that carry a live prediction. Predictions ride out with a reckoning for the host to use, and if they fed back into the derivation the engine's forecast would start justifying itself. |
//! | [`divergent_outcome_policy_matrix`] | channel | cites (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´) |
//! | [`divergent_outcome_challenge_reward_matrix`] | channel | cites (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´) |
//! | [`lifecycle_sufficiency_standard_matrix`] | channel | The risk assessment is a sufficient statistic for everything downstream of it: two worlds that arrived at the same one by different lifecycle routes derive the same three-channel matrix, across the whole enrichment grid. How an entity came to look the way it does is the core's business alone, which is what lets an assessment be stored, moved or re-derived later and still mean what it meant. |
//! | [`lifecycle_sufficiency_policy_matrix`] | channel | cites (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´) |
//! | [`lifecycle_sufficiency_challenge_reward_matrix`] | channel | cites (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´) |
//! | [`tag_count_matches_action_set_per_channel`] | channel | A reckoning carries one action tag per action its channel declares and no more: three for login, two for the API, four for transactions, with the slow tag present only on the channel that offers slowing. The classification side is untouched by that — all three channels emit the same three classification tags of the same kinds — and each reckoning is stamped with the channel it was derived for. A host can therefore read the action tags positionally against its own policy without discovering an option it never declared. |
//! | [`risk_basis_identical_across_channels`] | channel | cites (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´) |
//! | [`classification_tag_parameters_identical_across_channels`] | channel | Classification tags carry their full parameters across from the core unchanged: read on three differently-shaped channels, the same entity's classification tags agree not merely in kind and count but in location, magnitude and q. The classification half of a reckoning describes the entity and the action half describes what to do about it, and only the second is the channel's to shape. |
//! | [`risk_basis_invariant_when_action_set_extends`] | channel | cites (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´) |
//! | [`action_regime_widths_follow_channel_policy_without_moving_core`] | channel | The action layer divides a fixed quantity rather than creating one: the total action magnitude is the same on all three channels, so the four-action channel's challenge regime is no wider than the three-action channel's, the two-action channel's block regime is no narrower than the three-action one's, and the extra slow regime is live where it is declared. Adding an action takes its room from the others, which is why the risk basis can stay put while the recommendation changes. |
//! | [`action_tag_order_matches_channel_policy_actions`] | channel | cites (´claim:channel:the-action-tags-of-a-reckoning-are-exactly-the-channels-declared-actions-in-order´) |
//! | [`identical_policy_channel_aliases_replay_core_and_resonance_profile`] | channel | Two channels registered under different names but the same policy are the same channel for every purpose that matters: they hold distinct channel identifiers yet replay not only the core assessment but the entire resonance profile. Identity of behaviour is keyed on the policy, so a host splitting one channel into two names for its own bookkeeping gets no difference in what the engine tells it. |
//! | [`identical_policy_channel_aliases_replay_across_enrichment_grid`] | channel | cites (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´) |
//! | [`identical_policy_channel_aliases_request_order_across_enrichment_grid`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_identical_policy_channel_aliases_replay_across_enrichment_grid`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`pre_conditioned_identical_policy_channel_aliases_request_order_across_enrichment_grid`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`same_policy_channels_with_matching_challenge_evidence_replay`] | channel | cites (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´) |
//! | [`same_policy_channels_with_matching_challenge_evidence_replay_across_enrichment_grid`] | channel | cites (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´) |
//! | [`same_policy_channels_with_matching_challenge_evidence_request_order_across_enrichment_grid`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`pre_conditioned_same_policy_channels_with_matching_challenge_evidence_replay_across_enrichment_grid`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`pre_conditioned_same_policy_channels_with_matching_challenge_evidence_request_order_across_enrichment_grid`] | channel | cites (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´) |
//! | [`challenge_pass_vs_fail_request_order_replays_channel_profiles`] | channel | cites (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´) |
//! | [`stray_challenge_result_on_allow_label_is_ignored`] | channel | A challenge result only means anything on a label whose action was to challenge. Eight rounds of allow-labels carrying a spurious failure result leave a world indistinguishable from a matched control whose allow-labels carried none: same core, same resonance profile. The engine never issued a challenge, so there is no outcome to record, and a host that populates the field defensively on every label pays nothing for it. |
//! | [`challenge_without_result_follows_eligibility_policy`] | channel | What a result-free challenge label teaches is the declared eligibility policy's to decide, not a hard-coded refusal. Under the default policy the challenge proceeded and its label trains the inherent-risk models, so eight result-free challenge labels pull a world measurably away from a block-trained control. A deployment that declares the policy false withdraws the row, and the same stream then matches the block control exactly — a non-ground-truth challenge label is excluded like the blocks beside it. The challenge verdict stays companion-owned either way; the policy governs the label's outcome, never the verdict. |
//! | [`challenge_without_result_leaves_channel_alias_at_prior`] | channel | Result-free challenge labels leave the companion tracker where it started: after eight of them on one channel, a same-policy sibling that saw nothing at all still replays the entire profile, tracker included. Had the absent result been read as a pass or a fail the two channels would have parted, and a host with an unobservable challenge mechanism would be drifting its tracker on evidence it never had. |
//! | [`challenge_fail_vs_absent_in_shared_core_world`] | channel | cites (´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´) |
//! | [`distinct_public_core_state_with_same_risk_basis_derives_same_profile`] | channel | cites (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´) |
//! | [`non_spatial_outcome_axis_with_live_sentinel_report`] | channel | An outcome axis need not be spatial to work: an axis registered as non-spatial and trained on a magnitude over twelve cycles still produces the expected reported-outcome payload when the request also carries a live reporting sentinel, and the world stays clean. Spatial axes travel with a coordinate that a sentinel report can be located against, so pairing the two is the case where a non-spatial axis would be asked for a coordinate it has not got. |
//! | [`borrowed_share_identical_across_channels`] | channel | The share of borrowed confidence a verdict carries is the same on every channel that asks about the same request, and the same before and after the core has been moved by history. It is a property of the models and of the request's own features: the two floors put the mass there, the features choose the direction, and a channel is neither. The comparison is on the bits rather than within a drift budget, because there is no arithmetic between the core reading and the derived reckoning that could legitimately move it — a share that differed in its last place across two channels would mean the reading had been recomputed somewhere it should have been carried. The share may legitimately read zero here, and the invariant is the same either way: a healthy engine whose floors have never acted has borrowed nothing, and what this test forbids is a channel changing the answer, not a particular answer. The trained half of the test is where a leak would first appear, since an untrained core has less state to leak. |

//! Integration tests for the independence of the core model from the
//! decision layer.
//!
//! The architectural seam under test factors every reckoning into
//!
//! ```text
//!     core assess()  →  RiskAssessment  →  derive_reckoning(channel, …)
//! ```
//!
//! Tests are organised into *parameterized families* plus a few fixed
//! witnesses. Grid-backed families sweep an **enrichment grid** — every
//! combination of signal mode (none / valid / degraded), reported
//! Sentinel (yes / no), and live outcome prediction (yes / no) — across
//! each proof strategy, which is twelve configurations. Outcome-isolation
//! families use only the live-prediction half of the grid, which is six,
//! because a prediction payload is the condition under test there.
//! Lifecycle-sufficiency families keep all twelve, because a prediction is
//! one optional payload shape rather than the condition.
//!
//! Each family's own sweep shape and the history it runs against are
//! stated in that family's gloss rather than tabulated here, which is
//! what keeps the two from drifting: the shape a test sweeps is a fact
//! about that test, and a table of shapes maintained beside the tests was
//! a second copy of it that had already diverged in most of its cells
//! before it was retired.
//!
//! # Drift budgets, and why they are here
//!
//! Cross-world slices compare against named drift budgets from
//! [`torrust_assayer::testing`] rather than against bit identity, and every
//! budget they use now covers floating-point summation residue alone; each
//! constant names the source it covers. They cover no elapsed-time difference:
//! the engine and the live Companion trackers share the harness clock.
//!
//! Two arrangements are what let those budgets be that narrow, and both are
//! about removing a difference nothing here was ever asking about. Within a
//! world, a cross-channel comparison derives once: the cell's enrichment is
//! assessed one time and every channel derives from that single assessment,
//! so no comparison can straddle the model owner republishing between two
//! requests. Across worlds, the two matrix witnesses settle both of their
//! worlds before anything makes them differ, because a standardisation ramp
//! still transitioning advances on accepted observations that the steward
//! applies asynchronously, and two worlds compared mid-ramp stand at
//! different accepted counts for no reason but scheduling. Live Companion
//! state adds no allowance either: repeated reads take one fixed persistent
//! present from the harness clock.
//!
//! # Boundary properties
//!
//! The five properties this file exists for, each stated once as the
//! claim its family mints:
//!
//! - Channel independence — one assessment across differently-shaped
//!   channels (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´).
//! - Reward independence — reward parameters configure the action layer
//!   alone (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´).
//! - Challenge isolation — a challenge result feeds the companion tracker
//!   alone (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´).
//! - Risk-triple sufficiency — distinct histories agreeing on the
//!   assessment derive alike (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´).
//! - Outcome-prediction isolation — predictions ride along as payload
//!   (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´).

mod support;

use support::{
    API_ACTION_TAGS, CHALLENGE_REWARD_POLICY_MATRIX, CHALLENGE_REWARD_POLICY_MATRIX_REQUEST_ORDER, CorePayloadExpectation,
    DECISION_POLICY_MATRIX, DECISION_POLICY_MATRIX_REQUEST_ORDER, EnrichedChannelPair, EnrichedChannelPairAssertion,
    EnrichedCoreHistory, EnrichedHistoryCase, FOUR_ACTIONS, LOGIN_ACTION_TAGS, REWARD_AXIS_SWEEP_CASES,
    STANDARD_DERIVATION_MATRIX, STANDARD_DERIVATION_MATRIX_REQUEST_ORDER, THREE_ACTIONS, TRANSACTION_ACTION_TAGS, TWO_ACTIONS,
    action_magnitude_sum, action_tag_count, adverse_challenge_label, adverse_challenge_result, alternating_entity_label_stream,
    alternating_label_stream_for_entity, assert_action_layer_moves_without_core, assert_action_tag_sequence,
    assert_channel_pair_moves_only_action_layer, assert_channel_pair_replays_core_and_resonance_profile,
    assert_core_payload_expectation, assert_reward_perturbation_case_does_not_perturb_risk_basis,
    assert_risk_basis_and_resonance_profile_near_with_drift, assert_world_pair_action_layer_moves_without_core,
    challenge_label_pair, channel_pair_cases, classification_tag_count, classification_tag_kinds, classification_tags,
    cycle_label_case_stream, cycle_label_case_variant_stream_pair, multi_channel_independence_scenario, policy_with,
    policy_with_actions, register_distinct_golden_reporting_sentinel_states, train_channel_challenge_fail_vs_absent_evidence,
    train_enriched_challenge_reward_decision_evidence, train_no_enriched_decision_evidence, train_outcome_prediction,
};
use torrust_assayer::testing::{
    GOLDEN_COORD, LIFECYCLE_SUFFICIENCY_DRIFT, LabelSpec, PURITY_DRIFT, Scenario, assert_core_and_resonance_profile_near,
    assert_core_risk_basis_near, assert_health_clean, assert_near, assert_reckoning_well_formed,
    assert_resonance_profile_near_with_drift, assert_risk_basis_near, assert_tag_profile_near, attach_golden_reporting_sentinel,
    cycle_request, find_tag, scenario, scenario_with, scenario_with_config, score_verified_request_schema,
    with_degraded_score_verified, with_score_verified,
};
use torrust_assayer::types::{Action, ChallengeResult};
use torrust_assayer::{RewardParameters, Tag};

// ================================================================
// Parameterized test families
// ================================================================

// ---- Three-channel batch / request-order ----

/// A single core assessment is enough for as many channels as a host cares to
/// ask about: one assessment of a login request derives reckonings for the
/// login, API and transaction policies, each carrying that same assessment id,
/// each with the action tags its own policy declares, and all three agreeing
/// on their classification tags. The expensive part of a reckoning is paid
/// once, so consulting a second channel costs a derivation and not another
/// trip through the models.
///
/// ´claim:channel:one-core-assessment-serves-every-channel-without-a-second-core-read´
/// ´test:integration:one-core-assessment-derives-three-channel-policies´
#[test]
fn one_core_assessment_derives_three_channel_policies() {
    let world = multi_channel_independence_scenario("one-assessment-three-policies", 0x5EED_0200)
        .expect("world with three host policies");

    let assessment = world.assess(world.request("login", "alice"));
    let login = world.derive(&assessment, "login");
    let api = world.derive(&assessment, "api");
    let transaction = world.derive(&assessment, "transaction");

    for (name, reckoning) in [("login", &login), ("api", &api), ("transaction", &transaction)] {
        assert_eq!(reckoning.assessment.id, assessment.id, "{name}: reused assessment id");
        assert_reckoning_well_formed(reckoning, &format!("{name} reused assessment"));
    }

    assert_action_tag_sequence(&login, &LOGIN_ACTION_TAGS, "login reused assessment");
    assert_action_tag_sequence(&api, &API_ACTION_TAGS, "api reused assessment");
    assert_action_tag_sequence(&transaction, &TRANSACTION_ACTION_TAGS, "transaction reused assessment");
    assert_tag_profile_near(
        &classification_tags(&login),
        &classification_tags(&api),
        PURITY_DRIFT,
        "classification tags login vs api reused assessment",
    );
    assert_tag_profile_near(
        &classification_tags(&login),
        &classification_tags(&transaction),
        PURITY_DRIFT,
        "classification tags login vs transaction reused assessment",
    );
    assert_health_clean(&world);
}

/// The seam between the core model and the decision layer holds under every
/// shape of request the public surface admits: across the whole enrichment
/// grid — no signals, valid signals or deliberately degraded ones, crossed
/// with a live reporting sentinel or none, crossed with a live outcome
/// prediction or none — one entity assessed on three differently-shaped
/// channels shares its risk basis and its classification tags, and differs
/// only where the action sets differ. What a channel is for cannot bend what
/// the engine believes about the entity behind it.
///
/// ´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´
/// ´test:integration:three-channel-batch-core-independence´
#[test]
fn three_channel_batch_core_independence() {
    support::run_enriched_derivation_matrix_family(
        "three-channel-batch",
        0x5EED_0001,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::fresh("login", "alice"),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX,
    );
}

/// Deriving one channel leaves nothing behind that changes the next: running
/// the same three channels in reversed order reproduces each channel's
/// reckoning by name, across the full enrichment grid. Without this a host's
/// answer for a transaction would depend on whether it happened to consult the
/// login channel first, and the batch a service issues would silently become
/// part of its own input.
///
/// ´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´
/// ´test:integration:three-channel-request-order-replays´
#[test]
fn three_channel_request_order_replays() {
    support::run_enriched_derivation_matrix_request_order_family(
        "three-channel-request-order",
        0x5EED_0002,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::fresh("login", "alice"),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX_REQUEST_ORDER,
    );
}

/// Channel independence is a property of the architecture, not an artefact of
/// an untrained engine. After sixteen alternating labels have moved the models
/// on one channel, and again after eighteen labels routed through a registered
/// identity dimension, the three channels still share one core across the
/// whole enrichment grid. The separation being tested only matters once the
/// engine has learned something, which is exactly when a leak would show.
/// Every pre-conditioned cell derives once and compares channels: the cell's
/// enrichment is assessed one time and every channel derives from that single
/// assessment, so the comparison cannot straddle the model owner republishing
/// mid-batch as it applies the history — a core disagreement in this family
/// is a real leak from channel into core, never a race with the cell's own
/// training.
///
/// ´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´
/// ´test:integration:pre-conditioned-three-channel-batch´
#[test]
fn pre_conditioned_three_channel_batch() {
    support::run_enriched_derivation_matrix_family(
        "post-history-batch",
        0x5EED_1001,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::label_history("login", "gina", 16, 3),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX,
    );

    support::run_enriched_derivation_matrix_family(
        "identity-history-batch",
        0x5EED_1002,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::identity_history("account", "login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX,
    );
}

/// Order-independence likewise outlives a trained core: with label history or
/// identity history already behind it, the three-channel batch still replays
/// under reversal across every enrichment. A learned model carries state that
/// a stateless one cannot leak, so this is where a derivation that quietly
/// wrote back into the core would first betray itself.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-three-channel-request-order´
#[test]
fn pre_conditioned_three_channel_request_order() {
    support::run_enriched_derivation_matrix_request_order_family(
        "post-history-order",
        0x5EED_0A11,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::label_history("login", "gina", 16, 3),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX_REQUEST_ORDER,
    );

    support::run_enriched_derivation_matrix_request_order_family(
        "identity-history-order",
        0x5EED_0A12,
        support::build_enriched_standard_world,
        EnrichedCoreHistory::identity_history("account", "login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        STANDARD_DERIVATION_MATRIX_REQUEST_ORDER,
    );
}

// ---- Policy matrix ----

/// Varying two policy dimensions at once does not open a path the single
/// dimensions closed: a matrix mixing differently-shaped action sets with
/// differently-parameterised reward blocks still resolves to one shared core
/// per entity, while the pair that differs only in reward visibly moves its
/// action tags. Channel policy is a compound thing in practice, and its parts
/// have to stay core-neutral in combination and not merely one at a time.
///
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// ´test:integration:policy-matrix-core-independence´
#[test]
fn policy_matrix_core_independence() {
    support::run_enriched_derivation_matrix_family(
        "policy-matrix",
        0xD3C1_0001,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::fresh("login", "alice"),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX,
    );
}

/// The mixed action-shape and reward matrix replays under reversal too: every
/// channel in it comes back the same by name whichever end of the batch it is
/// derived from, and the reward-only pair keeps its action-layer difference in
/// both directions. Reward parameters are per-channel configuration, so they
/// must not become a hidden ordering between channels either.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:policy-matrix-request-order-replays´
#[test]
fn policy_matrix_request_order_replays() {
    support::run_enriched_derivation_matrix_request_order_family(
        "policy-matrix-order",
        0xD3C1_0002,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::fresh("login", "alice"),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX_REQUEST_ORDER,
    );
}

/// The compound action-shape and reward matrix stays core-neutral after
/// history, in both the batch and the reversed-order form, under label history
/// and under identity history alike. Combining every axis at once — two policy
/// dimensions, two kinds of prior training, two orderings, twelve enrichments
/// — is the widest sweep in the family and the closest thing to an exhaustive
/// case against a leak from policy into the core.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-policy-matrix´
#[test]
fn pre_conditioned_policy_matrix() {
    support::run_enriched_derivation_matrix_family(
        "post-history-policy-matrix",
        0xD3C1_7110,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::label_history("login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX,
    );

    support::run_enriched_derivation_matrix_request_order_family(
        "post-history-policy-order",
        0xD3C1_0710,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::label_history("login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX_REQUEST_ORDER,
    );

    support::run_enriched_derivation_matrix_family(
        "identity-history-policy-matrix",
        0x1D37_D3C1,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::identity_history("account", "login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX,
    );

    support::run_enriched_derivation_matrix_request_order_family(
        "identity-history-policy-order",
        0x1D37_0710,
        support::build_enriched_policy_matrix_world,
        EnrichedCoreHistory::identity_history("account", "login", "alice", 18, 3),
        train_no_enriched_decision_evidence,
        DECISION_POLICY_MATRIX_REQUEST_ORDER,
    );
}

// ---- Reward perturbation ----

/// Two channels sharing an action set but priced differently — the cost of a
/// miss doubled and the cost of friction quadrupled — reach different action
/// tags from an identical risk basis and identical classification tags, across
/// the whole enrichment grid. Rewards say what an outcome is worth to the
/// operator, and an operator revising that valuation must be able to change
/// what the system does without changing what it believes.
///
/// ´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´
/// ´test:integration:reward-perturbation-core-independence´
#[test]
fn reward_perturbation_core_independence() {
    support::run_enriched_channel_pair_family(
        "reward-perturbation",
        0xCAFE_0001,
        |e, tag, seed| {
            support::build_enriched_reward_world(e, tag, seed, &THREE_ACTIONS, |r| {
                r.missed *= 2.0;
                r.friction *= 4.0;
            })
        },
        "baseline",
        "alice",
        EnrichedChannelPair::new("baseline", "perturbed", &LOGIN_ACTION_TAGS),
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// Core-neutrality is not a property of one convenient reward knob: each of
/// the nine parameters in the reward block is perturbed on its own — the five
/// outcome values, the rate at which a block is assumed to catch, both
/// sensitivity coefficients, and the severity of slowing — and every one of
/// them moves the action layer while leaving the risk basis where it was.
/// A sweep over the whole surface is what makes the claim about the reward
/// block rather than about a sampled parameter.
///
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´)
/// ´test:integration:reward-axis-sweep´
#[test]
fn reward_axis_sweep() {
    for case in REWARD_AXIS_SWEEP_CASES {
        assert_reward_perturbation_case_does_not_perturb_risk_basis(&format!("reward-axis-{}", case.name), case, "alice");
    }
}

/// The nine-parameter reward sweep is repeated against richly-enriched
/// requests rather than bare ones — with signals present, degraded or absent,
/// with a live reporting sentinel, with a live outcome prediction. A payload
/// that reaches the core is exactly the kind of thing that could give a reward
/// parameter a route into it, so the sweep is only conclusive once every such
/// payload is in play alongside it.
///
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´)
/// ´test:integration:reward-axis-sweep-enriched-core-independence´
#[test]
fn reward_axis_sweep_enriched_core_independence() {
    support::run_enriched_reward_axis_sweep_family(
        "reward-axis",
        0xE711_0000,
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// Each of the nine reward parameters also survives reversal: for every one of
/// them, and at every enrichment, the baseline and perturbed channels replay
/// by name whichever is derived first. A parameter that only looked
/// core-neutral because the baseline was always evaluated first would be
/// caught here and nowhere else in the sweep.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:reward-axis-sweep-enriched-request-order-replays´
#[test]
fn reward_axis_sweep_enriched_request_order_replays() {
    support::run_enriched_reward_axis_sweep_family(
        "reward-axis-order",
        0xE711_0A11,
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

/// The reward sweep holds even with the companion challenge trackers awake:
/// both channels are first fed eight rounds of matched challenge evidence, so
/// each has a live tracker of its own, and every reward parameter is then still
/// confined to the action layer. Matched evidence is the sharpest setting for
/// this, because any difference that appears can only have come from the
/// reward block the two channels do not share.
///
/// (´claim:channel:reward-parameters-configure-the-action-layer-alone-and-cannot-reach-the-core´)
/// ´test:integration:challenge-reward-axis-sweep-enriched-core-independence´
#[test]
fn challenge_reward_axis_sweep_enriched_core_independence() {
    support::run_enriched_reward_axis_sweep_family(
        "challenge-reward-axis",
        0xC72C_0000,
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// Order-independence survives the combination of a live companion tracker on
/// each channel and a perturbed reward block: at every reward parameter and
/// every enrichment, reversing the batch reproduces both channels by name.
/// Tracker state is per-channel and mutable, which is the ingredient most
/// likely to make one derivation depend on a neighbour that ran before it.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-reward-axis-sweep-enriched-request-order-replays´
#[test]
fn challenge_reward_axis_sweep_enriched_request_order_replays() {
    support::run_enriched_reward_axis_sweep_family(
        "challenge-reward-axis-order",
        0xC72C_0A11,
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

/// A baseline channel and one priced with a doubled miss cost and quadrupled
/// friction replay by name under reversal at every enrichment, each keeping
/// its own action tags in both directions. Whichever of the two a host happens
/// to consult first, it gets that channel's own answer.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:reward-request-order-replays´
#[test]
fn reward_request_order_replays() {
    support::run_enriched_channel_pair_family(
        "reward-request-order",
        0x0710_0001,
        |e, tag, seed| {
            support::build_enriched_reward_world(e, tag, seed, &THREE_ACTIONS, |r| {
                r.missed *= 2.0;
                r.friction *= 4.0;
            })
        },
        "baseline",
        "alice",
        EnrichedChannelPair::new("baseline", "perturbed", &LOGIN_ACTION_TAGS),
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

/// Reward independence holds across three different kinds of accumulated
/// history: twenty alternating labels on the baseline channel, eighteen labels
/// delivered through a live reporting sentinel, and sixteen labels routed
/// through a registered identity dimension. In each the two channels still
/// differ only in their action tags. Repricing a channel therefore stays safe
/// on a system that has been running and learning, which is the only state a
/// production operator ever gets to reprice from.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-reward-perturbation´
#[test]
fn pre_conditioned_reward_perturbation() {
    let mut perturbed_reward = RewardParameters::default();
    perturbed_reward.missed *= 2.0;

    // After label history
    {
        let mut world = scenario_with("reward-after-labels", 0xF00D_F00D, |builder| {
            builder
                .channel("baseline", policy_with(THREE_ACTIONS, RewardParameters::default()))
                .channel("perturbed", policy_with(THREE_ACTIONS, perturbed_reward))
        })
        .expect("world should build");
        world.register_sentinel("S1").expect("register S1");
        cycle_label_case_stream(
            &world,
            "baseline",
            alternating_label_stream_for_entity("alice", 20, 2),
            "post-label reward label",
        );
        let cases = channel_pair_cases("baseline", "perturbed", &LOGIN_ACTION_TAGS);
        support::assert_request_pair_moves_only_action_layer_derive_once(
            &world,
            world.request("baseline", "alice"),
            &cases,
            CorePayloadExpectation::default(),
            "post-label reward perturbation",
        );
    }

    // After reported label history
    {
        let mut perturbed_full = RewardParameters::default();
        perturbed_full.missed *= 2.0;
        perturbed_full.friction *= 4.0;
        perturbed_full.caught *= 1.5;

        let mut world = scenario_with("reward-reported-labels", 0x5071_0071, |builder| {
            builder
                .channel("baseline", policy_with(THREE_ACTIONS, RewardParameters::default()))
                .channel("perturbed", policy_with(THREE_ACTIONS, perturbed_full))
        })
        .expect("world should build");
        let sentinel = attach_golden_reporting_sentinel(&mut world, "S1");
        support::train_reported_reward_label_history(&world, "baseline", "S1", 18);
        let cases = channel_pair_cases("baseline", "perturbed", &LOGIN_ACTION_TAGS);
        support::assert_request_pair_moves_only_action_layer_derive_once(
            &world,
            world.request_with_sentinel("baseline", "alice", "S1", GOLDEN_COORD),
            &cases,
            CorePayloadExpectation::live_sentinel(sentinel),
            "reported reward label history",
        );
    }

    // After identity history
    {
        let mut perturbed_id = RewardParameters::default();
        perturbed_id.missed *= 2.0;
        perturbed_id.friction *= 4.0;

        let mut world = scenario_with("reward-identity-history", 0x1D71_0071, |builder| {
            builder
                .channel("baseline", policy_with(THREE_ACTIONS, RewardParameters::default()))
                .channel("perturbed", policy_with(THREE_ACTIONS, perturbed_id))
        })
        .expect("world should build");
        world.register_identity("account").expect("register identity");
        cycle_label_case_stream(
            &world,
            "baseline",
            alternating_entity_label_stream(16, 4),
            "identity reward label",
        );
        let cases = channel_pair_cases("baseline", "perturbed", &LOGIN_ACTION_TAGS);
        support::assert_request_pair_moves_only_action_layer_derive_once(
            &world,
            world.request("baseline", "alice"),
            &cases,
            CorePayloadExpectation::default(),
            "identity-history reward perturbation",
        );
    }
}

/// Rewards stay out of the core on the way in as well as on the way out: two
/// separately-built worlds seeded alike, one with default rewards and one with
/// a perturbed block, are taught the same label stream through their own
/// channels and end up agreeing on the core while differing in the action
/// layer. The same holds when the requests carry signals and when the action
/// sets are widened. Since a label is what actually moves the models, this is
/// the path by which a reward could contaminate learning rather than merely
/// colour a reading.
///
/// ´claim:channel:routing-labels-through-differently-rewarded-channels-teaches-the-core-the-same-thing´
/// ´test:integration:two-world-reward-label-path´
#[test]
fn two_world_reward_label_path() {
    let mut perturbed = RewardParameters::default();
    perturbed.missed *= 2.0;
    perturbed.friction *= 4.0;

    let cases: &[(&str, u64, bool, bool)] = &[
        // (label, seed, use_signals, mixed_actions)
        ("plain", 0x7100_7100, false, false),
        ("signal-rich", 0x519A_7100, true, false),
        ("mixed-action", 0xA711_0071, false, true),
    ];

    for &(label, seed, use_signals, mixed_actions) in cases {
        let tag = format!("two-world-reward-path [{label}]");
        let actions: &[Action] = if mixed_actions { &FOUR_ACTIONS } else { &THREE_ACTIONS };
        let reward_perturbed = if mixed_actions {
            let mut r = perturbed.clone();
            r.caught *= 1.5;
            r
        } else {
            perturbed.clone()
        };

        let build = |name: &str, reward: RewardParameters| {
            if use_signals {
                scenario_with(name, seed, |b| {
                    b.signal_schema(score_verified_request_schema())
                        .channel("observed", policy_with(actions.iter().copied(), reward))
                })
                .expect("world should build")
            } else {
                scenario_with(name, seed, |b| {
                    b.channel("observed", policy_with(actions.iter().copied(), reward))
                })
                .expect("world should build")
            }
        };

        let baseline_world = build(&format!("{tag}-baseline"), RewardParameters::default());
        let perturbed_world = build(&format!("{tag}-perturbed"), reward_perturbed);

        if mixed_actions {
            support::train_matched_mixed_action_label_history(&baseline_world, &perturbed_world, "observed", 16, &tag);
        } else {
            support::train_matched_reward_label_history(&baseline_world, &perturbed_world, "observed", use_signals, 12, &tag);
        }

        if use_signals {
            support::assert_world_pair_request_action_layer_moves_without_core(
                &baseline_world,
                with_score_verified(baseline_world.request("observed", "alice"), 0.77, true),
                &perturbed_world,
                with_score_verified(perturbed_world.request("observed", "alice"), 0.77, true),
                &tag,
            );
        } else {
            assert_world_pair_action_layer_moves_without_core(&baseline_world, &perturbed_world, "observed", "alice", &tag);
        }
        assert_health_clean(&baseline_world);
        assert_health_clean(&perturbed_world);
    }
}

// ---- Challenge result isolation ----

/// Whether an entity passed or failed a challenge changes what the decision
/// layer recommends and not what the core believes: eight rounds of failures
/// against eight rounds of passes leave the risk basis and classification tags
/// in agreement while the action tags separate. This holds through five
/// delivery shapes — plain, marked as ground truth, carried on valid signals,
/// carried on deliberately degraded ones, and delivered via a live reporting
/// sentinel. A challenge outcome is evidence about the challenge, and treating
/// it as evidence about the entity would double-count the engine's own
/// intervention.
///
/// ´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´
/// ´test:integration:challenge-pass-fail-core-independence´
#[test]
#[allow(clippy::too_many_lines)] // Justified: one table-driven witness keeps the Pass/Fail variant matrix co-located.
fn challenge_pass_fail_core_independence() {
    struct ChallengeVariant {
        label: &'static str,
        seed: u64,
        use_signals: bool,
        reported: bool,
        ground_truth: bool,
    }
    let variants = [
        ChallengeVariant {
            label: "plain",
            seed: 0xA11C_E072,
            use_signals: false,
            reported: false,
            ground_truth: false,
        },
        ChallengeVariant {
            label: "ground-truth",
            seed: 0xA11C_7272,
            use_signals: false,
            reported: false,
            ground_truth: true,
        },
        ChallengeVariant {
            label: "signal-rich",
            seed: 0x519A_0072,
            use_signals: true,
            reported: false,
            ground_truth: false,
        },
        ChallengeVariant {
            label: "degraded",
            seed: 0xDE67_AD72,
            use_signals: true,
            reported: false,
            ground_truth: false,
        },
        ChallengeVariant {
            label: "reported",
            seed: 0xA11C_5072,
            use_signals: false,
            reported: true,
            ground_truth: false,
        },
    ];

    for v in &variants {
        let tag = format!("challenge-pass-fail [{}]", v.label);

        if v.reported {
            // Single-world reported variant: two same-policy channels
            let policy = policy_with_actions(THREE_ACTIONS);
            let mut world = scenario_with(&tag, v.seed, |builder| {
                builder.channel("fail", policy.clone()).channel("pass", policy)
            })
            .expect("world should build");
            let sentinel = attach_golden_reporting_sentinel(&mut world, "S1");
            let with_fail = adverse_challenge_result(ChallengeResult::Fail);
            let with_pass = adverse_challenge_result(ChallengeResult::Pass);
            for _ in 0..8 {
                cycle_request(
                    &world,
                    world.request_with_sentinel("fail", "alice", "S1", GOLDEN_COORD),
                    with_fail,
                );
                cycle_request(
                    &world,
                    world.request_with_sentinel("pass", "alice", "S1", GOLDEN_COORD),
                    with_pass,
                );
            }
            let requests = [
                world.request_with_sentinel("fail", "alice", "S1", GOLDEN_COORD),
                world.request_with_sentinel("pass", "alice", "S1", GOLDEN_COORD),
            ];
            let cases = channel_pair_cases("fail", "pass", &LOGIN_ACTION_TAGS);
            support::assert_request_pair_moves_only_action_layer_with_core_payload(
                &world,
                &requests,
                &cases,
                CorePayloadExpectation::live_sentinel(sentinel),
                &tag,
            );
            assert_health_clean(&world);
        } else if v.use_signals {
            // Two-world signal variant (valid or degraded)
            let is_degraded = v.label == "degraded";
            let fail_world = scenario_with(&format!("{tag}-fail"), v.seed, |b| {
                b.signal_schema(score_verified_request_schema())
                    .channel("observed", policy_with_actions(THREE_ACTIONS))
            })
            .expect("fail world");
            let pass_world = scenario_with(&format!("{tag}-pass"), v.seed, |b| {
                b.signal_schema(score_verified_request_schema())
                    .channel("observed", policy_with_actions(THREE_ACTIONS))
            })
            .expect("pass world");
            // Settle both ramps before the streams diverge, for the reason the
            // plain variant settles below: two ramps fed from two labelled
            // histories end in coordinate systems that genuinely differ, and
            // this comparison is about the action layer rather than about the
            // ramp (´inv:guarantee:evidence-authority´).
            fail_world.settle_cold_ramp_with(&[fail_world.request("observed", "alice")]);
            pass_world.settle_cold_ramp_with(&[pass_world.request("observed", "alice")]);
            support::train_signal_challenge_pair(&fail_world, &pass_world, "observed", "alice", 8, is_degraded, &tag);
            let make_request = |w: &Scenario| {
                if is_degraded {
                    with_degraded_score_verified(w.request("observed", "alice"))
                } else {
                    with_score_verified(w.request("observed", "alice"), 0.64, true)
                }
            };
            let fail_r = fail_world
                .derive_for_request(make_request(&fail_world))
                .expect("fail reckoning");
            let pass_r = pass_world
                .derive_for_request(make_request(&pass_world))
                .expect("pass reckoning");
            assert_reckoning_well_formed(&fail_r, &format!("{tag} fail"));
            assert_reckoning_well_formed(&pass_r, &format!("{tag} pass"));
            assert_action_layer_moves_without_core(&fail_r, &pass_r, &tag);
            if is_degraded {
                assert_core_payload_expectation(&fail_r, CorePayloadExpectation::signal_degradation(1, 1), &tag);
            }
            if is_degraded {
                support::assert_signal_degradation_health(&fail_world);
                support::assert_signal_degradation_health(&pass_world);
            } else {
                assert_health_clean(&fail_world);
                assert_health_clean(&pass_world);
            }
        } else {
            // Plain / ground-truth two-world variant
            let fail_world = scenario(&format!("{tag}-fail"), v.seed);
            let pass_world = scenario(&format!("{tag}-pass"), v.seed);
            // Both worlds settle before the label stream starts, so the two
            // coordinate systems are identical when the streams diverge. Left
            // transitioning, each world's ramp would take its eight
            // observations from its own labelled history and the two would
            // end the comparison in coordinate systems that genuinely differ
            // — which is the ramp working, not the action layer leaking into
            // the core (´inv:guarantee:evidence-authority´).
            fail_world.settle_cold_ramp_with(&[fail_world.request("default", "alice")]);
            pass_world.settle_cold_ramp_with(&[pass_world.request("default", "alice")]);
            let pair = challenge_label_pair(Some(ChallengeResult::Fail), Some(ChallengeResult::Pass), v.ground_truth);
            cycle_label_case_variant_stream_pair(&fail_world, &pass_world, "default", std::iter::repeat_n(pair, 8), &tag);
            assert_world_pair_action_layer_moves_without_core(&fail_world, &pass_world, "default", "alice", &tag);
            assert_health_clean(&fail_world);
            assert_health_clean(&pass_world);
        }
    }
}

/// A companion tracker belongs to its channel and to nothing else: feeding
/// eight failures to one channel moves that channel's action layer while an
/// untouched sibling under the identical policy still shares the evolved core
/// and stays at its own prior, across the whole enrichment grid. Whether
/// challenges work is a fact about a particular channel's challenge
/// mechanism, so a channel that never issues one must not inherit another's
/// experience of it.
///
/// ´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´
/// ´test:integration:challenge-tracker-channel-scope´
#[test]
fn challenge_tracker_channel_scope() {
    support::run_enriched_same_policy_pair_family(
        "challenge-scope",
        0xC0DE_0001,
        EnrichedChannelPair::new("challenged", "untouched", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, setup, pair, entity, label| {
            support::train_enriched_channel_fail_evidence(world, enrichment, setup, pair.left, entity, 8, label);
        },
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// Two channels of one world, fed opposite challenge outcomes for the same
/// entity, hold a single shared belief about that entity between them while
/// their trackers pull apart. Putting both halves inside one engine is
/// stricter than comparing two worlds: there is a live core here that both
/// evidence streams could have moved, and neither does.
///
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´)
/// ´test:integration:challenge-pass-vs-fail-shared-core´
#[test]
fn challenge_pass_vs_fail_shared_core() {
    support::run_enriched_same_policy_pair_family(
        "pass-vs-fail-shared-core",
        0xFA17_7272,
        EnrichedChannelPair::new("fail", "pass", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_pass_vs_fail_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// Reversing which of the pass-fed and fail-fed channels is derived first
/// returns each its own tracker profile unchanged, at every enrichment. The
/// two trackers hold opposite evidence in one world, so if any tracker state
/// were being read across channels this reversal would swap the answers.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-pass-vs-fail-request-order-replays´
#[test]
fn challenge_pass_vs_fail_request_order_replays() {
    support::run_enriched_same_policy_pair_family(
        "pass-vs-fail-order",
        0xFA17_0A11,
        EnrichedChannelPair::new("fail", "pass", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_pass_vs_fail_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

/// Challenge isolation is not an artefact of a quiet engine: after twelve
/// labels on the fail-fed channel, and again after twelve routed through an
/// identity dimension, the opposite challenge outcomes still leave the shared
/// core alone. The core has genuinely moved before the challenge evidence
/// arrives, so there is a moving target for the evidence to disturb.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-challenge-pass-vs-fail-shared-core´
#[test]
fn pre_conditioned_challenge_pass_vs_fail_shared_core() {
    let pair = EnrichedChannelPair::new("fail", "pass", &LOGIN_ACTION_TAGS);
    let histories = [
        EnrichedHistoryCase::new(
            "post-history-pass-vs-fail",
            0xFA17_7110,
            EnrichedCoreHistory::label_history("fail", "alice", 12, 3),
        ),
        EnrichedHistoryCase::new(
            "identity-pass-vs-fail",
            0xFA17_1D37,
            EnrichedCoreHistory::identity_history("account", "fail", "alice", 12, 3),
        ),
    ];

    support::run_enriched_same_policy_pair_history_cases(
        &histories,
        pair,
        &THREE_ACTIONS,
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_pass_vs_fail_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// The channel that was never challenged keeps its prior whichever end of the
/// batch it is derived from: reversing the challenged and untouched pair
/// returns each its own profile at every enrichment. An untouched channel
/// picking up its neighbour's evidence merely because it was evaluated second
/// is precisely the residue this reversal is shaped to catch.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-tracker-request-order-replays´
#[test]
fn challenge_tracker_request_order_replays() {
    support::run_enriched_same_policy_pair_family(
        "challenge-order",
        0xC0DE_0002,
        EnrichedChannelPair::new("challenged", "untouched", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, setup, pair, entity, label| {
            support::train_enriched_channel_fail_evidence(world, enrichment, setup, pair.left, entity, 8, label);
        },
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

/// Having no challenge result is a different state from having a failing one,
/// and only the latter moves anything: across the enrichment grid a channel
/// fed eight failures separates in its action layer from one whose labels
/// carry no result at all, while both keep the shared core. Absence must read
/// as silence rather than as a quiet vote either way, or a host that cannot
/// report a challenge outcome would be teaching the tracker by omission.
///
/// (´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´)
/// ´test:integration:challenge-fail-vs-absent-evidence´
#[test]
fn challenge_fail_vs_absent_evidence() {
    support::run_enriched_same_policy_pair_family(
        "fail-vs-absent",
        0xFA11_0001,
        EnrichedChannelPair::new("fail", "absent", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_fail_vs_absent_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::ActionOnly,
    );
}

/// The fail-fed and result-free channels replay by name under reversal at
/// every enrichment, each keeping its own tracker profile. The result-free
/// channel is the one with nothing of its own to report, and so the one most
/// easily filled in from whatever ran alongside it.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-fail-vs-absent-request-order-replays´
#[test]
fn challenge_fail_vs_absent_request_order_replays() {
    support::run_enriched_same_policy_pair_family(
        "fail-vs-absent-order",
        0xFA11_0A11,
        EnrichedChannelPair::new("fail", "absent", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_fail_vs_absent_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::RequestOrder,
    );
}

// ---- Challenge/reward matrix ----

/// All three channel-local dimensions at once — the shape of the action set,
/// the reward block that prices it, and the companion tracker's accumulated
/// challenge evidence — still resolve to one core per entity across the whole
/// enrichment grid. Independence proved one dimension at a time would leave
/// room for a leak that needs two of them together; this is the matrix that
/// closes it.
///
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// ´test:integration:challenge-reward-matrix-core-independence´
#[test]
fn challenge_reward_matrix_core_independence() {
    support::run_enriched_derivation_matrix_family(
        "challenge-reward-matrix",
        0xC0DE_7172,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::fresh("challenged", "alice"),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX,
    );
}

/// The full challenge-and-reward matrix replays by name under reversal at
/// every enrichment. This is the batch with the most per-channel state in it —
/// distinct action shapes, distinct pricing, and a live tracker apiece — and
/// therefore the most opportunity for one channel's derivation to be coloured
/// by another that ran first.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-reward-matrix-request-order-replays´
#[test]
fn challenge_reward_matrix_request_order_replays() {
    support::run_enriched_derivation_matrix_request_order_family(
        "challenge-reward-order",
        0xC0DE_7207,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::fresh("challenged", "alice"),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX_REQUEST_ORDER,
    );
}

/// Prior training composes with the challenge-and-reward matrix rather than
/// weakening it: after twelve labels, and again after twelve routed through an
/// identity dimension, the matrix stays core-neutral both in batch and under
/// reversal, at every enrichment. Every axis the file varies is switched on
/// simultaneously here, which is what makes it the family's closing argument
/// rather than another sample.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-challenge-reward-matrix´
#[test]
fn pre_conditioned_challenge_reward_matrix() {
    support::run_enriched_derivation_matrix_family(
        "post-history-challenge-reward",
        0xC0DE_7110,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::label_history("challenged", "alice", 12, 3),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX,
    );

    support::run_enriched_derivation_matrix_request_order_family(
        "post-history-challenge-reward-order",
        0xC0DE_7208,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::label_history("challenged", "alice", 12, 3),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX_REQUEST_ORDER,
    );

    support::run_enriched_derivation_matrix_family(
        "identity-challenge-reward",
        0x1D37_7172,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::identity_history("account", "challenged", "alice", 12, 3),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX,
    );

    support::run_enriched_derivation_matrix_request_order_family(
        "identity-challenge-reward-order",
        0x1D37_7207,
        support::build_enriched_challenge_reward_world,
        EnrichedCoreHistory::identity_history("account", "challenged", "alice", 12, 3),
        train_enriched_challenge_reward_decision_evidence,
        CHALLENGE_REWARD_POLICY_MATRIX_REQUEST_ORDER,
    );
}

// ---- Divergent outcome predictions ----

/// An outcome prediction is something the engine reports, not something it
/// reasons from: two worlds trained to predict opposite values on the same
/// axis still replay the three-channel derivation from the same risk basis,
/// across the enrichments that carry a live prediction. Predictions ride out
/// with a reckoning for the host to use, and if they fed back into the
/// derivation the engine's forecast would start justifying itself.
///
/// ´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´
/// ´test:integration:divergent-outcome-standard-matrix´
#[test]
fn divergent_outcome_standard_matrix() {
    support::run_outcome_enrichment_grid("divergent-outcome-standard", 0x0A74_5EED, |e, tag, seed| {
        support::run_divergent_outcome_standard_matrix(e, tag, seed);
    });
}

/// Opposite learned predictions leave the mixed action-shape and reward matrix
/// replaying identically as well. Diverging predictions plus diverging policy
/// is the combination in which a prediction could plausibly be read as one
/// more decision input, and the matrix comes back the same regardless.
///
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// ´test:integration:divergent-outcome-policy-matrix´
#[test]
fn divergent_outcome_policy_matrix() {
    support::run_outcome_enrichment_grid("divergent-outcome-policy", 0x0A74_D3C1, |e, tag, seed| {
        support::run_divergent_outcome_policy_matrix(e, tag, seed);
    });
}

/// Even with companion challenge trackers live and reward blocks perturbed,
/// two worlds holding opposite outcome predictions replay the same matrix from
/// the same risk basis. Trackers and rewards both legitimately move the action
/// layer, so this is the setting where a prediction leaking in would be
/// easiest to mistake for one of them doing its job.
///
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// ´test:integration:divergent-outcome-challenge-reward-matrix´
#[test]
fn divergent_outcome_challenge_reward_matrix() {
    support::run_outcome_enrichment_grid("divergent-outcome-cr", 0x0A74_7271, |e, tag, seed| {
        support::run_divergent_outcome_challenge_reward_matrix(e, tag, seed);
    });
}

// ---- Lifecycle sufficiency ----

/// The risk assessment is a sufficient statistic for everything downstream of
/// it: two worlds that arrived at the same one by different lifecycle routes
/// derive the same three-channel matrix, across the whole enrichment grid. How
/// an entity came to look the way it does is the core's business alone, which
/// is what lets an assessment be stored, moved or re-derived later and still
/// mean what it meant.
///
/// ´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´
/// ´test:integration:lifecycle-sufficiency-standard-matrix´
#[test]
fn lifecycle_sufficiency_standard_matrix() {
    support::run_enrichment_grid("lifecycle-standard", 0x7373_5EED, |e, tag, seed| {
        support::run_lifecycle_sufficiency_standard_matrix(e, tag, seed);
    });
}

/// Sufficiency survives a richer decision layer: worlds with distinct
/// lifecycle histories but a matching risk basis also replay the mixed
/// action-shape and reward matrix. The derivation reads more of the policy
/// here and still nothing more of the history.
///
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´)
/// ´test:integration:lifecycle-sufficiency-policy-matrix´
#[test]
fn lifecycle_sufficiency_policy_matrix() {
    support::run_enrichment_grid("lifecycle-policy", 0x7373_7102, |e, tag, seed| {
        support::run_lifecycle_sufficiency_policy_matrix(e, tag, seed);
    });
}

/// Sufficiency holds with companion challenge evidence and a perturbed reward
/// block in play as well. The companion tracker is the one piece of per-channel
/// memory the decision layer keeps, so it is the obvious candidate for a route
/// by which lifecycle history could re-enter a derivation that is supposed to
/// read only the risk basis.
///
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´)
/// ´test:integration:lifecycle-sufficiency-challenge-reward-matrix´
#[test]
fn lifecycle_sufficiency_challenge_reward_matrix() {
    support::run_enrichment_grid("lifecycle-cr", 0x7373_7271, |e, tag, seed| {
        support::run_lifecycle_sufficiency_challenge_reward_matrix(e, tag, seed);
    });
}

// ================================================================
// One-off tests (structurally unique)
// ================================================================

/// A reckoning carries one action tag per action its channel declares and no
/// more: three for login, two for the API, four for transactions, with the
/// slow tag present only on the channel that offers slowing. The
/// classification side is untouched by that — all three channels emit the same
/// three classification tags of the same kinds — and each reckoning is stamped
/// with the channel it was derived for. A host can therefore read the action
/// tags positionally against its own policy without discovering an option it
/// never declared.
///
/// ´claim:channel:the-action-tags-of-a-reckoning-are-exactly-the-channels-declared-actions-in-order´
/// ´test:integration:tag-count-matches-action-set-per-channel´
#[test]
fn tag_count_matches_action_set_per_channel() {
    let world = multi_channel_independence_scenario("multi-channel", 0x5EED_5EED).expect("world should build");
    let entity = "alice";

    let login = world.derive_on("login", entity).expect("login");
    let api = world.derive_on("api", entity).expect("api");
    let txn = world.derive_on("transaction", entity).expect("transaction");

    assert_eq!(action_tag_count(&login), 3, "login: 3 action tags");
    assert_eq!(action_tag_count(&api), 2, "api: 2 action tags");
    assert_eq!(action_tag_count(&txn), 4, "transaction: 4 action tags");

    for (name, r) in [("login", &login), ("api", &api), ("transaction", &txn)] {
        assert_eq!(classification_tag_count(r), 3, "{name}: 3 classification tags");
    }
    assert_eq!(classification_tag_kinds(&login), classification_tag_kinds(&api));
    assert_eq!(classification_tag_kinds(&login), classification_tag_kinds(&txn));

    let has_slow = |r: &torrust_assayer::DerivedReckoning| r.profile.tags.iter().any(|t| t.tag == Tag::Slow);
    assert!(!has_slow(&login));
    assert!(!has_slow(&api));
    assert!(has_slow(&txn));

    assert_eq!(login.channel, world.channel("login").expect("login id"));
    assert_eq!(api.channel, world.channel("api").expect("api id"));
    assert_eq!(txn.channel, world.channel("transaction").expect("txn id"));
    assert_health_clean(&world);
}

/// The plain witness for the seam: one entity read through the login, API and
/// transaction channels yields three well-formed reckonings whose risk bases
/// agree to within the time-decay band. Stripped of the enrichment sweeps and
/// the matrices, this is the single fact the whole file elaborates.
///
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// ´test:integration:risk-basis-identical-across-channels´
#[test]
fn risk_basis_identical_across_channels() {
    let world = multi_channel_independence_scenario("multi-channel", 0x5EED_5EED).expect("world should build");
    let entity = "bob";
    // Three separate assessments, deliberately. The matrices elsewhere in this
    // file hold one assessment and vary the policy, which is the right shape
    // for a claim about derivation; this scenario is the other half, and asks
    // whether naming a channel on the request reaches the core assessment at
    // all (´claim:api:core-assessment-needs-no-channel-because-policy-belongs-to-derivation´).
    // Holding one assessment here would answer that question by construction
    // and assert nothing. Three reads it must be — so they must be taken in one
    // coordinate state to be comparable, because while the cold ramp transitions
    // each would be answered one advance later than the last
    // (´inv:guarantee:evidence-authority´).
    world.settle_cold_ramp_with(&[world.request("login", entity)]);

    let login = world.derive_on("login", entity).expect("login");
    let api = world.derive_on("api", entity).expect("api");
    let txn = world.derive_on("transaction", entity).expect("transaction");

    for (name, r) in [("login", &login), ("api", &api), ("transaction", &txn)] {
        assert_reckoning_well_formed(r, &format!("{name} reckoning"));
    }
    assert_core_risk_basis_near(&login, &api, "RiskAssessment: login vs api");
    assert_core_risk_basis_near(&login, &txn, "RiskAssessment: login vs transaction");
    assert_health_clean(&world);
}

/// Classification tags carry their full parameters across from the core
/// unchanged: read on three differently-shaped channels, the same entity's
/// classification tags agree not merely in kind and count but in location,
/// magnitude and q. The classification half of a reckoning describes the
/// entity and the action half describes what to do about it, and only the
/// second is the channel's to shape.
///
/// ´claim:channel:classification-tags-are-core-derived-and-therefore-identical-across-channels´
/// ´test:integration:classification-tag-parameters-identical-across-channels´
#[test]
fn classification_tag_parameters_identical_across_channels() {
    let world = multi_channel_independence_scenario("multi-channel", 0x5EED_5EED).expect("world should build");
    let entity = "carol";

    // One assessment read on three channels, not three assessments compared.
    // The claim's "therefore" is the thing under test: the tags are core-derived,
    // so what has to be shown is that the derivation carries them across
    // untouched whatever policy it is handed. Reading the entity three times
    // would put the three answers in coordinate systems an accepted observation
    // apart (´dec:concurrency:per-request-load´), and the tolerance below would
    // then be standing in for a legitimate coordinate change rather than for the
    // thing under test (´inv:guarantee:evidence-authority´). Held, the core is
    // the constant and the channel policy is the only thing that varies.
    let assessment = world.assess(world.request("login", entity));
    let login = world.derive(&assessment, "login");
    let api = world.derive(&assessment, "api");
    let txn = world.derive(&assessment, "transaction");

    assert_tag_profile_near(
        &classification_tags(&login),
        &classification_tags(&api),
        PURITY_DRIFT,
        "classification tags login vs api",
    );
    assert_tag_profile_near(
        &classification_tags(&login),
        &classification_tags(&txn),
        PURITY_DRIFT,
        "classification tags login vs transaction",
    );
    assert_health_clean(&world);
}

/// Extending a channel's repertoire adds options without revising beliefs: a
/// ladder of two-, three- and four-action channels produces two, three and
/// four action tags from a risk basis that is the same at every rung. Offering
/// the operator a further response is a decision-layer change, so it must not
/// come with a quiet reassessment of the entity attached.
///
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// ´test:integration:risk-basis-invariant-when-action-set-extends´
#[test]
fn risk_basis_invariant_when_action_set_extends() {
    let world = scenario_with("action-ladder", 0x1ADD_AC11, |builder| {
        builder
            .channel("two", policy_with_actions(TWO_ACTIONS))
            .channel("three", policy_with_actions(THREE_ACTIONS))
            .channel("four", policy_with_actions(FOUR_ACTIONS))
    })
    .expect("world should build");

    let entity = "dana";
    // Separate assessments, deliberately: "without revising beliefs" is a claim
    // about the belief a fresh read returns on each rung, and holding one
    // assessment across the three would satisfy it by construction. They must
    // therefore be taken in one coordinate state to be comparable, because
    // while the cold ramp transitions each would be answered one advance later
    // than the last (´inv:guarantee:evidence-authority´).
    world.settle_cold_ramp_with(&[world.request("two", entity)]);

    let r2 = world.derive_on("two", entity).expect("two");
    let r3 = world.derive_on("three", entity).expect("three");
    let r4 = world.derive_on("four", entity).expect("four");

    assert_eq!(action_tag_count(&r2), 2);
    assert_eq!(action_tag_count(&r3), 3);
    assert_eq!(action_tag_count(&r4), 4);
    assert_core_risk_basis_near(&r2, &r3, "RiskAssessment: 2 vs 3 action");
    assert_core_risk_basis_near(&r3, &r4, "RiskAssessment: 3 vs 4 action");
    assert_health_clean(&world);
}

/// The action layer divides a fixed quantity rather than creating one: the
/// total action magnitude is the same on all three channels, so the
/// four-action channel's challenge regime is no wider than the three-action
/// channel's, the two-action channel's block regime is no narrower than the
/// three-action one's, and the extra slow regime is live where it is declared.
/// Adding an action takes its room from the others, which is why the risk
/// basis can stay put while the recommendation changes.
///
/// ´claim:channel:action-regimes-share-a-fixed-mass-so-a-richer-action-set-narrows-each-regime´
/// ´test:integration:action-regime-widths-follow-channel-policy-without-moving-core´
#[test]
fn action_regime_widths_follow_channel_policy_without_moving_core() {
    let world = multi_channel_independence_scenario("multi-channel", 0x5EED_5EED).expect("world should build");
    let entity = "frank";
    // Separate assessments, deliberately: alongside the regime-mass claim this
    // scenario carries the "without moving core" half of its own name, which is
    // a statement about three fresh reads agreeing rather than about one held
    // answer. They must therefore be taken in one coordinate state to be
    // comparable, because while the cold ramp transitions each would be
    // answered one advance later than the last
    // (´inv:guarantee:evidence-authority´).
    world.settle_cold_ramp_with(&[world.request("login", entity)]);

    let login = world.derive_on("login", entity).expect("login");
    let api = world.derive_on("api", entity).expect("api");
    let txn = world.derive_on("transaction", entity).expect("transaction");

    for (name, r) in [("login", &login), ("api", &api), ("transaction", &txn)] {
        assert_reckoning_well_formed(r, &format!("{name} regime-width"));
    }
    assert_core_risk_basis_near(&login, &api, "regime-width: login vs api");
    assert_core_risk_basis_near(&login, &txn, "regime-width: login vs transaction");

    let login_mass = action_magnitude_sum(&login);
    let api_mass = action_magnitude_sum(&api);
    let txn_mass = action_magnitude_sum(&txn);
    assert_near(login_mass, api_mass, PURITY_DRIFT, "action mass: login vs api");
    assert_near(login_mass, txn_mass, PURITY_DRIFT, "action mass: login vs txn");

    let login_ch = find_tag(&login.profile.tags, Tag::Challenge);
    let txn_ch = find_tag(&txn.profile.tags, Tag::Challenge);
    let txn_slow = find_tag(&txn.profile.tags, Tag::Slow);
    assert!(
        login_ch.magnitude >= txn_ch.magnitude,
        "3-action Challenge ({}) >= 4-action ({})",
        login_ch.magnitude,
        txn_ch.magnitude
    );
    assert!(txn_slow.magnitude > 0.0, "4-action Slow should be live");

    let api_block = find_tag(&api.profile.tags, Tag::Block);
    let login_block = find_tag(&login.profile.tags, Tag::Block);
    assert!(
        api_block.magnitude >= login_block.magnitude,
        "2-action Block ({}) >= 3-action ({})",
        api_block.magnitude,
        login_block.magnitude
    );
    assert_health_clean(&world);
}

/// Action tags arrive in the order the policy declared them, not in some
/// canonical order of severity the engine prefers: allow-challenge-block for
/// login, allow-block for the API, allow-challenge-slow-block for
/// transactions. Sequence is part of the contract, so a host can zip the tags
/// against its own action list rather than searching by kind.
///
/// (´claim:channel:the-action-tags-of-a-reckoning-are-exactly-the-channels-declared-actions-in-order´)
/// ´test:integration:action-tag-order-matches-channel-policy-actions´
#[test]
fn action_tag_order_matches_channel_policy_actions() {
    let world = multi_channel_independence_scenario("multi-channel", 0x5EED_5EED).expect("world should build");
    let cases = [
        ("login", vec![Tag::Allow, Tag::Challenge, Tag::Block]),
        ("api", vec![Tag::Allow, Tag::Block]),
        ("transaction", vec![Tag::Allow, Tag::Challenge, Tag::Slow, Tag::Block]),
    ];
    for (name, expected) in cases {
        let r = world.derive_on(name, "erin").expect("reckoning");
        assert_action_tag_sequence(&r, &expected, name);
    }
    assert_health_clean(&world);
}

/// Two channels registered under different names but the same policy are the
/// same channel for every purpose that matters: they hold distinct channel
/// identifiers yet replay not only the core assessment but the entire
/// resonance profile. Identity of behaviour is keyed on the policy, so a host
/// splitting one channel into two names for its own bookkeeping gets no
/// difference in what the engine tells it.
///
/// ´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´
/// ´test:integration:identical-policy-channel-aliases-replay-core-and-resonance-profile´
#[test]
fn identical_policy_channel_aliases_replay_core_and_resonance_profile() {
    let world = scenario_with("identical-aliases", 0x1D59_A11A, |builder| {
        let policy = policy_with_actions(THREE_ACTIONS);
        builder.channel("login_a", policy.clone()).channel("login_b", policy)
    })
    .expect("world should build");

    let assessments = assert_channel_pair_replays_core_and_resonance_profile(
        &world,
        "login_a",
        "login_b",
        "alice",
        &LOGIN_ACTION_TAGS,
        "identical-policy aliases",
    );
    assert_ne!(assessments[0].channel, assessments[1].channel);
    assert_health_clean(&world);
}

/// Policy aliases stay indistinguishable across every enrichment the public
/// surface admits — signals absent, valid or degraded, with or without a live
/// reporting sentinel, with or without a live outcome prediction. Each of
/// those payloads is a place where a channel name could quietly become part of
/// the computation, and none of them makes the two names diverge.
///
/// (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´)
/// ´test:integration:identical-policy-channel-aliases-replay-across-enrichment-grid´
#[test]
fn identical_policy_channel_aliases_replay_across_enrichment_grid() {
    support::run_enriched_same_policy_pair_family(
        "identical-aliases",
        0x1D59_A11A,
        EnrichedChannelPair::new("login_a", "login_b", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::CoreAndResonanceProfile,
    );
}

/// Aliases of one policy replay under reversal too, with both members of each
/// batch still sharing the whole profile whichever is derived first. Two
/// channels that should be identical are the most sensitive detector of
/// ordering residue there is: any difference at all between them is a defect,
/// with no legitimate divergence to hide in.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:identical-policy-channel-aliases-request-order-across-enrichment-grid´
#[test]
fn identical_policy_channel_aliases_request_order_across_enrichment_grid() {
    support::run_enriched_same_policy_pair_family(
        "identical-aliases-order",
        0x1D59_0A11,
        EnrichedChannelPair::new("login_a", "login_b", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::CoreAndResonanceProfileRequestOrder,
    );
}

/// Aliases stay identical after the core has been moved: twelve labels
/// delivered through one of the two names, and separately twelve routed
/// through an identity dimension, leave both names still replaying the full
/// profile at every enrichment. Training arrives through a named channel, so
/// this is the case in which the engine could most easily have attributed what
/// it learned to that name rather than to the entity.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-identical-policy-channel-aliases-replay-across-enrichment-grid´
#[test]
fn pre_conditioned_identical_policy_channel_aliases_replay_across_enrichment_grid() {
    let pair = EnrichedChannelPair::new("login_a", "login_b", &LOGIN_ACTION_TAGS);
    let histories = [
        EnrichedHistoryCase::new(
            "post-history-identical-aliases",
            0x1D59_7110,
            EnrichedCoreHistory::label_history("login_a", "alice", 12, 3),
        ),
        EnrichedHistoryCase::new(
            "identity-identical-aliases",
            0x1D59_1D37,
            EnrichedCoreHistory::identity_history("account", "login_a", "alice", 12, 3),
        ),
    ];

    support::run_enriched_same_policy_pair_history_cases(
        &histories,
        pair,
        &THREE_ACTIONS,
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::CoreAndResonanceProfile,
    );
}

/// Trained aliases also replay under reversal, at every enrichment and under
/// both label and identity history. Combining a moved core with a reordered
/// batch stacks the two ways a derivation could pick up something it should
/// not — accumulated state and a neighbour derived first — and the two names
/// remain indistinguishable through both.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-identical-policy-channel-aliases-request-order-across-enrichment-grid´
#[test]
fn pre_conditioned_identical_policy_channel_aliases_request_order_across_enrichment_grid() {
    let pair = EnrichedChannelPair::new("login_a", "login_b", &LOGIN_ACTION_TAGS);
    let histories = [
        EnrichedHistoryCase::new(
            "post-history-identical-aliases-order",
            0x1D59_0A10,
            EnrichedCoreHistory::label_history("login_a", "alice", 12, 3),
        ),
        EnrichedHistoryCase::new(
            "identity-identical-aliases-order",
            0x1D59_0A37,
            EnrichedCoreHistory::identity_history("account", "login_a", "alice", 12, 3),
        ),
    ];

    support::run_enriched_same_policy_pair_history_cases(
        &histories,
        pair,
        &THREE_ACTIONS,
        support::train_no_enriched_channel_pair_evidence,
        EnrichedChannelPairAssertion::CoreAndResonanceProfileRequestOrder,
    );
}

/// Sameness extends to accumulated companion state: two same-policy channels
/// each fed the same eight challenge failures replay the whole profile,
/// trackers included, despite holding separate channel identifiers. The
/// trackers here are live and non-trivial, so agreement is a statement about
/// how the evidence was accumulated and not about both sides being empty.
///
/// (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´)
/// ´test:integration:same-policy-channels-with-matching-challenge-evidence-replay´
#[test]
fn same_policy_channels_with_matching_challenge_evidence_replay() {
    let policy = policy_with_actions(THREE_ACTIONS);
    let world = scenario_with("matching-evidence-aliases", 0x7272_A11A, |builder| {
        builder.channel("left", policy.clone()).channel("right", policy)
    })
    .expect("world should build");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    for _ in 0..8 {
        world.cycle_on("left", "alice", with_fail);
        world.cycle_on("right", "alice", with_fail);
    }
    let assessments = support::assert_channel_pair_replays_core_and_resonance_profile(
        &world,
        "left",
        "right",
        "alice",
        &LOGIN_ACTION_TAGS,
        "matching challenge evidence replay",
    );
    assert_ne!(assessments[0].channel, assessments[1].channel);
    assert_health_clean(&world);
}

/// Matched challenge evidence keeps two same-policy channels identical across
/// every enrichment, not only on bare requests. A tracker fed the same
/// evidence through a live sentinel or alongside degraded signals must still
/// land in the same place as its twin, or the enrichment rather than the
/// evidence would be deciding what the tracker holds.
///
/// (´claim:channel:channels-with-identical-policy-and-identical-evidence-derive-identical-reckonings´)
/// ´test:integration:same-policy-channels-with-matching-challenge-evidence-replay-across-enrichment-grid´
#[test]
fn same_policy_channels_with_matching_challenge_evidence_replay_across_enrichment_grid() {
    support::run_enriched_same_policy_pair_family(
        "matching-evidence-aliases",
        0x7272_A11A,
        EnrichedChannelPair::new("left", "right", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::CoreAndResonanceProfile,
    );
}

/// Two channels holding matched tracker evidence replay by name under
/// reversal at every enrichment, and each batch's pair still shares the full
/// profile. Live per-channel state plus a reordered batch is the combination
/// that would let one tracker be read where the other was meant, and it does
/// not happen.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:same-policy-channels-with-matching-challenge-evidence-request-order-across-enrichment-grid´
#[test]
fn same_policy_channels_with_matching_challenge_evidence_request_order_across_enrichment_grid() {
    support::run_enriched_same_policy_pair_family(
        "matching-evidence-aliases-order",
        0x7272_0A11,
        EnrichedChannelPair::new("left", "right", &LOGIN_ACTION_TAGS),
        &THREE_ACTIONS,
        "alice",
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::CoreAndResonanceProfileRequestOrder,
    );
}

/// Matched-evidence aliases stay identical after twelve labels, and after
/// twelve routed through an identity dimension, at every enrichment. Here the
/// core has moved and both trackers are live at once, so agreement between the
/// two names has to survive both kinds of accumulated state rather than one.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-same-policy-channels-with-matching-challenge-evidence-replay-across-enrichment-grid´
#[test]
fn pre_conditioned_same_policy_channels_with_matching_challenge_evidence_replay_across_enrichment_grid() {
    let pair = EnrichedChannelPair::new("left", "right", &LOGIN_ACTION_TAGS);
    let histories = [
        EnrichedHistoryCase::new(
            "post-history-matching-evidence-aliases",
            0x7272_7110,
            EnrichedCoreHistory::label_history("left", "alice", 12, 3),
        ),
        EnrichedHistoryCase::new(
            "identity-matching-evidence-aliases",
            0x7272_1D37,
            EnrichedCoreHistory::identity_history("account", "left", "alice", 12, 3),
        ),
    ];

    support::run_enriched_same_policy_pair_history_cases(
        &histories,
        pair,
        &THREE_ACTIONS,
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::CoreAndResonanceProfile,
    );
}

/// The strictest cell of the alias family: a moved core, live matched trackers
/// on both channels, a reversed batch, and the full enrichment grid, with the
/// two names still deriving the same reckoning throughout. Every source of
/// state the file knows how to create is switched on here simultaneously.
///
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´)
/// ´test:integration:pre-conditioned-same-policy-channels-with-matching-challenge-evidence-request-order-across-enrichment-grid´
#[test]
fn pre_conditioned_same_policy_channels_with_matching_challenge_evidence_request_order_across_enrichment_grid() {
    let pair = EnrichedChannelPair::new("left", "right", &LOGIN_ACTION_TAGS);
    let histories = [
        EnrichedHistoryCase::new(
            "post-history-matching-evidence-aliases-order",
            0x7272_0A10,
            EnrichedCoreHistory::label_history("left", "alice", 12, 3),
        ),
        EnrichedHistoryCase::new(
            "identity-matching-evidence-aliases-order",
            0x7272_0A37,
            EnrichedCoreHistory::identity_history("account", "left", "alice", 12, 3),
        ),
    ];

    support::run_enriched_same_policy_pair_history_cases(
        &histories,
        pair,
        &THREE_ACTIONS,
        |world, enrichment, _setup, pair, entity, label| {
            support::train_enriched_matching_challenge_evidence(world, enrichment, pair, entity, 8, label);
        },
        EnrichedChannelPairAssertion::CoreAndResonanceProfileRequestOrder,
    );
}

/// The bare witness for order-independence under opposed evidence: one world,
/// two same-policy channels fed eight failures and eight passes respectively,
/// derived forwards and then backwards, with each direction returning each
/// channel its own action layer over the shared core. No enrichment, no
/// history — just the reversal itself, readable end to end in one function.
///
/// (´claim:channel:reversing-a-request-batch-replays-each-channel-by-name-without-cross-derivation-residue´)
/// ´test:integration:challenge-pass-vs-fail-request-order-replays-channel-profiles´
#[test]
fn challenge_pass_vs_fail_request_order_replays_channel_profiles() {
    let policy = policy_with_actions(THREE_ACTIONS);
    let world = scenario_with("pass-fail-order", 0x7272_0A11, |builder| {
        builder.channel("fail", policy.clone()).channel("pass", policy)
    })
    .expect("world should build");

    let with_fail = adverse_challenge_result(ChallengeResult::Fail);
    let with_pass = adverse_challenge_result(ChallengeResult::Pass);
    for _ in 0..8 {
        world.cycle_on("fail", "alice", with_fail);
        world.cycle_on("pass", "alice", with_pass);
    }
    let forward = channel_pair_cases("fail", "pass", &LOGIN_ACTION_TAGS);
    let reverse = channel_pair_cases("pass", "fail", &LOGIN_ACTION_TAGS);
    support::assert_request_pair_order_replays_only_action_layer(
        &world,
        "alice",
        &forward,
        &reverse,
        "Pass-vs-Fail request-order",
    );
    assert_health_clean(&world);
}

/// A challenge result only means anything on a label whose action was to
/// challenge. Eight rounds of allow-labels carrying a spurious failure result
/// leave a world indistinguishable from a matched control whose allow-labels
/// carried none: same core, same resonance profile. The engine never issued a
/// challenge, so there is no outcome to record, and a host that populates the
/// field defensively on every label pays nothing for it.
///
/// ´claim:channel:a-challenge-result-attached-to-a-non-challenge-action-is-ignored-entirely´
/// ´test:integration:stray-challenge-result-on-allow-label-is-ignored´
#[test]
fn stray_challenge_result_on_allow_label_is_ignored() {
    let stray_world = scenario("stray-result", 0xA110_0072);
    let control_world = scenario("control-allow", 0xA110_0072);

    // Both worlds settle before their label streams start, as every other
    // parallel-trained pair in this file does. The ramp advances on the
    // assessment clock and the steward applies it asynchronously, so two worlds
    // cycled in one interleaved loop accept different samples depending only on
    // scheduling and arrive at the comparison on two different sets of empirical
    // moments. That difference is the ramp working, not a stray challenge result
    // being read, and asserting through it measures the machine the suite is
    // running on (´inv:guarantee:evidence-authority´). Settled first, the
    // spurious result field is the only thing that differs between them.
    stray_world.settle_cold_ramp_with(&[stray_world.request("default", "alice")]);
    control_world.settle_cold_ramp_with(&[control_world.request("default", "alice")]);

    for _ in 0..8 {
        stray_world.cycle_default("alice", |id| {
            LabelSpec::adverse(id)
                .action(Action::Allow)
                .challenge_result(ChallengeResult::Fail)
        });
        control_world.cycle_default("alice", |id| LabelSpec::adverse(id).action(Action::Allow));
    }

    let stray = stray_world.derive_default("alice").expect("stray reckoning");
    let control = control_world.derive_default("alice").expect("control reckoning");
    assert_reckoning_well_formed(&stray, "stray Allow");
    assert_reckoning_well_formed(&control, "control Allow");
    assert_core_and_resonance_profile_near(&stray, &control, "stray Allow vs control");
    assert_health_clean(&stray_world);
    assert_health_clean(&control_world);
}

/// What a result-free challenge label teaches is the declared eligibility
/// policy's to decide, not a hard-coded refusal. Under the default policy the
/// challenge proceeded and its label trains the inherent-risk models, so
/// eight result-free challenge labels pull a world measurably away from a
/// block-trained control. A deployment that declares the policy false
/// withdraws the row, and the same stream then matches the block control
/// exactly — a non-ground-truth challenge label is excluded like the blocks
/// beside it. The challenge verdict stays companion-owned either way; the
/// policy governs the label's outcome, never the verdict.
///
/// ´claim:channel:a-result-free-challenge-label-trains-by-default-and-the-declared-policy-can-withdraw-it´
/// ´test:integration:challenge-without-result-follows-eligibility-policy´
#[test]
fn challenge_without_result_follows_eligibility_policy() {
    const DRIFT: f64 = 2e-9;

    // Default policy: challenge labels train (´tab:eligibility:training´),
    // so the challenge-fed world separates from the block-fed control.
    let challenge_world = scenario("absent-result-default", 0xA850_0072);
    let block_world = scenario("absent-result-block-control", 0xA850_0072);

    for _ in 0..8 {
        challenge_world.cycle_default("alice", |id| LabelSpec::adverse(id).action(Action::Challenge));
        block_world.cycle_default("alice", |id| LabelSpec::adverse(id).action(Action::Block));
    }

    let ch = challenge_world.derive_default("alice").expect("Challenge+None reckoning");
    let bl = block_world.derive_default("alice").expect("Block reckoning");
    assert_reckoning_well_formed(&ch, "Challenge+None default policy");
    assert_reckoning_well_formed(&bl, "Block control");
    assert!(
        (ch.assessment.risk.p_bad - bl.assessment.risk.p_bad).abs() > DRIFT,
        "default policy: challenge labels should train where block labels cannot; \
         p_bad {} vs {}",
        ch.assessment.risk.p_bad,
        bl.assessment.risk.p_bad,
    );

    // Policy declared false: the row is withdrawn and the same stream
    // matches the block control.
    let strict = |name: &str| {
        let mut config = torrust_assayer::AssayerConfig {
            instance_id: name.to_owned(),
            // The capacity is host-set with no default; the fixture declares it.
            infrastructure: torrust_assayer::testing::test_infrastructure(),
            ..Default::default()
        };
        config.eligibility.challenge_fail_is_unconfounded = false;
        scenario_with_config(config, 0xA850_0073, |builder| {
            builder.channel("default", torrust_assayer::ChannelPolicy::default())
        })
        .expect("strict world should build")
    };
    let strict_challenge_world = strict("absent-result-strict");
    let strict_block_world = strict("absent-result-strict-block");

    // Both worlds settle before their streams diverge, not after. The ramp
    // advances on the assessment clock and the steward applies it
    // asynchronously, so two worlds cycling in parallel accept different
    // samples depending only on scheduling — and settling afterwards would
    // land them on two different sets of empirical moments rather than on one.
    // The claim here is that a withdrawn eligibility row leaves the two
    // streams agreeing (´inv:guarantee:evidence-authority´).
    strict_challenge_world.settle_cold_ramp_with(&[strict_challenge_world.request("default", "alice")]);
    strict_block_world.settle_cold_ramp_with(&[strict_block_world.request("default", "alice")]);

    for _ in 0..8 {
        strict_challenge_world.cycle_default("alice", |id| LabelSpec::adverse(id).action(Action::Challenge));
        strict_block_world.cycle_default("alice", |id| LabelSpec::adverse(id).action(Action::Block));
    }

    let sch = strict_challenge_world
        .derive_default("alice")
        .expect("strict Challenge+None reckoning");
    let sbl = strict_block_world.derive_default("alice").expect("strict Block reckoning");
    assert_reckoning_well_formed(&sch, "Challenge+None strict policy");
    assert_risk_basis_near(&sch.assessment.risk, &sbl.assessment.risk, DRIFT, "strict policy vs Block");
    assert_resonance_profile_near_with_drift(&sch, &sbl, DRIFT, "strict policy resonance");

    assert_health_clean(&challenge_world);
    assert_health_clean(&block_world);
    assert_health_clean(&strict_challenge_world);
    assert_health_clean(&strict_block_world);
}

/// Result-free challenge labels leave the companion tracker where it started:
/// after eight of them on one channel, a same-policy sibling that saw nothing
/// at all still replays the entire profile, tracker included. Had the absent
/// result been read as a pass or a fail the two channels would have parted,
/// and a host with an unobservable challenge mechanism would be drifting its
/// tracker on evidence it never had.
///
/// ´claim:channel:a-result-free-challenge-label-leaves-the-companion-tracker-untouched´
/// ´test:integration:challenge-without-result-leaves-channel-alias-at-prior´
#[test]
fn challenge_without_result_leaves_channel_alias_at_prior() {
    let policy = policy_with_actions(THREE_ACTIONS);
    let world = scenario_with("absent-result-alias", 0xA850_A11A, |builder| {
        builder.channel("absent", policy.clone()).channel("untouched", policy)
    })
    .expect("world should build");

    let absent_result = adverse_challenge_label(None);
    for _ in 0..8 {
        world.cycle_on("absent", "alice", absent_result);
    }
    let assessments = assert_channel_pair_replays_core_and_resonance_profile(
        &world,
        "absent",
        "untouched",
        "alice",
        &LOGIN_ACTION_TAGS,
        "Challenge+None alias replay",
    );
    assert_ne!(assessments[0].channel, assessments[1].channel);
    assert_health_clean(&world);
}

/// Failure and absence are separated inside one engine: two same-policy
/// channels of a single world, one fed recorded failures and the other labels
/// carrying no result, share the core while their action layers part. The bare
/// single-world form of the fail-versus-absent distinction, with both channels
/// reading the same live models as they diverge.
///
/// (´claim:channel:challenge-evidence-is-scoped-to-the-channel-that-received-it´)
/// ´test:integration:challenge-fail-vs-absent-in-shared-core-world´
#[test]
fn challenge_fail_vs_absent_in_shared_core_world() {
    let policy = policy_with_actions(THREE_ACTIONS);
    let world = scenario_with("fail-vs-none-shared", 0xFA11_0A85, |builder| {
        builder.channel("fail", policy.clone()).channel("absent", policy)
    })
    .expect("world should build");

    train_channel_challenge_fail_vs_absent_evidence(&world, "fail", "absent", "alice", 8, "Fail vs absent evidence");
    assert_channel_pair_moves_only_action_layer(
        &world,
        "fail",
        "absent",
        "alice",
        &LOGIN_ACTION_TAGS,
        "Fail vs absent evidence",
    );
    assert_health_clean(&world);
}

/// The single-channel witness for sufficiency: a fresh world and a cycled one,
/// each carrying one live reporting sentinel but reaching it by different
/// lifecycles, derive matching core assessments and resonance profiles within
/// the lifecycle drift band. The sentinel genuinely reports — a golden report
/// ingested and the request routed through its coordinate — so the
/// contributor count of one is a measured fact rather than an occupancy
/// fiction. Reduced to two worlds and one channel, this is the fact the
/// lifecycle matrices sweep — that what the engine has been through leaves no
/// trace beyond the statistic it publishes.
///
/// (´claim:channel:the-risk-assessment-is-sufficient-so-distinct-histories-agreeing-on-it-derive-alike´)
/// ´test:integration:distinct-public-core-state-with-same-risk-basis-derives-same-profile´
#[test]
fn distinct_public_core_state_with_same_risk_basis_derives_same_profile() {
    let mut fresh_world = scenario("sufficiency-fresh", 0x73_73_73_73);
    let mut cycled_world = scenario("sufficiency-cycled", 0x73_73_73_73);
    register_distinct_golden_reporting_sentinel_states(&mut fresh_world, &mut cycled_world, "single-channel sufficiency");

    let fresh = fresh_world
        .derive_for_request(fresh_world.request_with_sentinel("default", "alice", "S1", GOLDEN_COORD))
        .expect("fresh reckoning");
    let cycled = cycled_world
        .derive_for_request(cycled_world.request_with_sentinel("default", "alice", "S1", GOLDEN_COORD))
        .expect("cycled reckoning");
    assert_reckoning_well_formed(&fresh, "fresh sufficiency");
    assert_reckoning_well_formed(&cycled, "cycled sufficiency");
    assert_eq!(fresh.assessment.risk.n_sentinels_reporting, 1);
    assert_eq!(cycled.assessment.risk.n_sentinels_reporting, 1);
    // The per-Sentinel payload keys are world-local by design, so the
    // comparison lands on the shared public risk statistic and the
    // decision layer, as the golden reporting probes compare.
    assert_risk_basis_and_resonance_profile_near_with_drift(
        &cycled,
        &fresh,
        LIFECYCLE_SUFFICIENCY_DRIFT,
        "same RiskBasis from distinct lifecycle",
    );
    assert_health_clean(&fresh_world);
    assert_health_clean(&cycled_world);
}

/// An outcome axis need not be spatial to work: an axis registered as
/// non-spatial and trained on a magnitude over twelve cycles still produces
/// the expected reported-outcome payload when the request also carries a live
/// reporting sentinel, and the world stays clean. Spatial axes travel with a
/// coordinate that a sentinel report can be located against, so pairing the
/// two is the case where a non-spatial axis would be asked for a coordinate it
/// has not got.
///
/// ´claim:channel:a-non-spatial-outcome-axis-derives-cleanly-alongside-a-live-sentinel-report´
/// ´test:integration:non-spatial-outcome-axis-with-live-sentinel-report´
#[test]
fn non_spatial_outcome_axis_with_live_sentinel_report() {
    let mut world = multi_channel_independence_scenario("non-spatial-regression", 0x5EED_5EED).expect("world should build");
    let axis = world.register_axis("magnitude", false).expect("register non-spatial axis");
    train_outcome_prediction(&world, "login", "alice", axis, 4.0, 12);
    let sentinel = attach_golden_reporting_sentinel(&mut world, "S1");
    world.cycle_on("login", "lifecycle-flush", LabelSpec::benign);

    let reckoning = world
        .derive_for_request(world.request_with_sentinel("login", "alice", "S1", GOLDEN_COORD))
        .expect("non-spatial should not break");
    assert_core_payload_expectation(
        &reckoning,
        CorePayloadExpectation::reported_outcome(sentinel, axis),
        "non-spatial regression witness",
    );
    assert_health_clean(&world);
}

/// The share of borrowed confidence a verdict carries is the same on every
/// channel that asks about the same request, and the same before and after the
/// core has been moved by history. It is a property of the models and of the
/// request's own features: the two floors put the mass there, the features
/// choose the direction, and a channel is neither. The comparison is on the
/// bits rather than within a drift budget, because there is no arithmetic
/// between the core reading and the derived reckoning that could legitimately
/// move it — a share that differed in its last place across two channels would
/// mean the reading had been recomputed somewhere it should have been carried.
///
/// The share may legitimately read zero here, and the invariant is the same
/// either way: a healthy engine whose floors have never acted has borrowed
/// nothing, and what this test forbids is a channel changing the answer, not a
/// particular answer. The trained half of the test is where a leak would first
/// appear, since an untrained core has less state to leak.
///
/// ´claim:channel:the-borrowed-share-a-verdict-carries-is-identical-on-every-channel-that-asks-about-the-request´
/// ´test:integration:borrowed-share-identical-across-channels´
#[test]
fn borrowed_share_identical_across_channels() {
    let world = multi_channel_independence_scenario("borrowed-share", 0x5EED_5EED).expect("world should build");
    let entity = "bob";

    // Three separate assessments, taken in one coordinate state so that they
    // are comparable (´inv:guarantee:evidence-authority´) — the same shape the
    // risk-basis reading beside this one uses, and for the same reason.
    world.settle_cold_ramp_with(&[world.request("login", entity)]);

    let cold: Vec<_> = ["login", "api", "transaction"]
        .into_iter()
        .map(|name| (name, world.derive_on(name, entity).expect("derive")))
        .collect();
    let reference = cold[0].1.assessment.risk.borrowed_share;
    assert!((0.0..=1.0).contains(&reference), "the share is a share: {reference}");
    for (name, reckoning) in &cold[1..] {
        let share = reckoning.assessment.risk.borrowed_share;
        assert!(
            share.to_bits() == reference.to_bits(),
            "{name} reads the same borrowed share as login: {share} against {reference}"
        );
    }

    // And again once the core has been moved by history, which is where a leak
    // from channel into core would first have something to leak.
    let trained = multi_channel_independence_scenario("borrowed-share-trained", 0x5EED_5EED).expect("world should build");
    for _ in 0..16 {
        trained.cycle_on("login", entity, LabelSpec::benign);
    }
    trained.settle_cold_ramp_with(&[trained.request("login", entity)]);

    let warm: Vec<_> = ["login", "api", "transaction"]
        .into_iter()
        .map(|name| (name, trained.derive_on(name, entity).expect("derive")))
        .collect();
    let warm_reference = warm[0].1.assessment.risk.borrowed_share;
    for (name, reckoning) in &warm[1..] {
        let share = reckoning.assessment.risk.borrowed_share;
        assert!(
            share.to_bits() == warm_reference.to_bits(),
            "{name} reads the same borrowed share as login on a trained core: {share} against {warm_reference}"
        );
    }

    // The uncertainty the share tempered agrees across channels too, which is
    // the consequence the ruling is actually about: the widening is applied
    // once, in the core, and every channel reads the same widened figure.
    for (name, reckoning) in &warm[1..] {
        let uncertainty = reckoning.assessment.risk.uncertainty;
        let expected = warm[0].1.assessment.risk.uncertainty;
        assert!(
            uncertainty.to_bits() == expected.to_bits(),
            "{name} reads the same tempered uncertainty as login: {uncertainty} against {expected}"
        );
    }

    assert_health_clean(&world);
    assert_health_clean(&trained);
}
