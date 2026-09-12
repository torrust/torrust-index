// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sufficiency tests for the resonance derivation boundary, where the Core's work ends (´dec:ordering:core-boundary´).
//!
//! These tests sit at crate level so the fixtures can hand the layered
//! pipeline the bare sufficient-statistic scalars directly. The
//! derivation function is the factored boundary: once the core model
//! has reduced its state to the sufficient statistic, the decision
//! layer receives only that statistic plus channel policy.
//!
//! # Cross-References
//!
//! - [`crate::resonance::landscape`] — pure derivation from the risk basis, policy and posterior.
//! - (´dec:ordering:closed-crossing´) — the `RiskAssessment` sufficient statistic is all that crosses into derivation.
//! - (´def:landscape:outcome-neutrality´) — outcome predictions are published beside the landscape, never fed into it.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`same_sufficient_statistic_and_policy_replays_identical_profile`] | resonance | Two entities whose model histories share nothing — one with no labels, no Sentinels and no identity structure at all, the other trained on twelve thousand labels across dozens of identity cells — derive bit-identical tag profiles once they reduce to the same four scalars under the same policy. The statistic really is sufficient: everything the core learned reaches the decision layer through it or not at all. |
//! | [`same_sufficient_statistic_replays_identical_profile_across_policy_shapes`] | resonance | cites (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´) |
//! | [`rich_core_witnesses_with_same_sufficient_statistic_replay_identical_profile`] | resonance | cites (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´) |
//! | [`outcome_prediction_payload_is_not_a_resonance_derivation_input`] | resonance | Outcome-axis predictions may be published beside a risk assessment, but the derivation has no parameter slot for them: a risk-only witness and one carrying two predicted axes with confirmedly different payloads produce the same profile down to the bit. What the model expects the *magnitude* of an adverse outcome to be is information for the host to act on separately, and cannot leak sideways into where the action tags are placed. |
//! | [`same_sufficient_statistic_replays_identical_profile_across_witness_policy_grid`] | resonance | cites (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´) |
//! | [`sufficient_statistic_fields_are_live_derivation_inputs`] | resonance | The statistic is sufficient without being padded: shifting any one of its four scalars — the probability, the effective spread, the condition number, or the challenge-estimate variance — with the policy held fixed moves the tag profile somewhere. Each field is genuinely read, so the replay results above cannot be passing because the derivation has quietly stopped consulting one of its inputs. |
//! | [`policy_reward_and_challenge_estimate_are_live_derivation_inputs`] | resonance | The host's own two levers are live in the same sense: quadrupling the friction reward, or raising the challenge-effectiveness estimate from 0.57 to 0.82, each moves the profile while the core statistic stands still. The replay guarantee is conditional on holding these fixed, and that condition has teeth — a host retuning its rewards genuinely gets a different channel, not the same one relabelled. |

use super::helpers::{KernelScalars, derive_tags};
use crate::resonance::channel::{ChannelPolicy, RewardParameters};
use crate::resonance::derivation::ResonanceConfig;
use crate::resonance::tags::TagResonance;
use crate::testing::{assert_finite, assert_tag_profile_diverges_somewhere};
use crate::types::Action;

const EPS: f64 = 1e-12;

/// Deliberately richer than [`KernelScalars`]: this stands in for
/// model history/state that the derivation layer must not be able to
/// observe once the core has emitted the sufficient statistic.
#[derive(Clone, Debug)]
struct CoreStateWitness {
    model_family: &'static str,
    label_history: &'static str,
    labels_seen: u64,
    registered_sentinels: usize,
    identity_cells: usize,
    outcome_axes: usize,
    outcome_predictions: Vec<OutcomePredictionWitness>,
    sentinel_alarm_fingerprint: u64,
}

impl CoreStateWitness {
    fn fingerprint(&self) -> String {
        let prediction_fingerprint = outcome_prediction_fingerprint(&self.outcome_predictions).join(",");
        format!(
            "{}|{}|{}|{}|{}|{}|{}|{}",
            self.model_family,
            self.label_history,
            self.labels_seen,
            self.registered_sentinels,
            self.identity_cells,
            self.outcome_axes,
            prediction_fingerprint,
            self.sentinel_alarm_fingerprint,
        )
    }
}

#[derive(Clone, Debug)]
struct OutcomePredictionWitness {
    axis_name: &'static str,
    predicted_raw: f64,
    uncertainty: f64,
}

impl OutcomePredictionWitness {
    fn fingerprint(&self) -> String {
        format!(
            "{}:{}:{}",
            self.axis_name,
            self.predicted_raw.to_bits(),
            self.uncertainty.to_bits(),
        )
    }
}

#[derive(Clone, Debug)]
struct SufficientStatisticCase {
    left: CoreStateWitness,
    right: CoreStateWitness,
    statistic: KernelScalars,
}

#[derive(Clone, Debug)]
struct PolicyFixture {
    label: &'static str,
    policy: ChannelPolicy,
    q_hat_c: f64,
}

fn baseline_statistic() -> KernelScalars {
    KernelScalars {
        p_hat: 0.37,
        sigma_eff: 0.58,
        kappa_eff: 1.25,
        sigma2_q_hat_c: 0.031,
    }
}

fn sufficient_statistic_from(_witness: &CoreStateWitness, statistic: &KernelScalars) -> KernelScalars {
    statistic.clone()
}

fn outcome_prediction_fingerprint(predictions: &[OutcomePredictionWitness]) -> Vec<String> {
    predictions.iter().map(OutcomePredictionWitness::fingerprint).collect()
}

fn derive_profile(input: &KernelScalars, policy: &ChannelPolicy, q_hat_c: f64) -> Vec<TagResonance> {
    derive_tags(input, q_hat_c, policy, &ResonanceConfig::default())
}

fn policy_fixtures() -> Vec<PolicyFixture> {
    let mut high_friction = RewardParameters::default();
    high_friction.friction *= 4.0;

    let mut high_catch_reward = RewardParameters::default();
    high_catch_reward.caught *= 2.5;

    vec![
        PolicyFixture {
            label: "default four-action policy",
            policy: ChannelPolicy::default(),
            q_hat_c: 0.57,
        },
        PolicyFixture {
            label: "two-action policy",
            policy: ChannelPolicy {
                actions: vec![Action::Allow, Action::Block],
                reward: RewardParameters::default(),
                ..ChannelPolicy::default()
            },
            q_hat_c: 0.57,
        },
        PolicyFixture {
            label: "three-action policy",
            policy: ChannelPolicy {
                actions: vec![Action::Allow, Action::Challenge, Action::Block],
                reward: RewardParameters::default(),
                ..ChannelPolicy::default()
            },
            q_hat_c: 0.57,
        },
        PolicyFixture {
            label: "four-action high-friction policy",
            policy: ChannelPolicy {
                actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
                reward: high_friction,
                ..ChannelPolicy::default()
            },
            q_hat_c: 0.72,
        },
        PolicyFixture {
            label: "three-action high-catch policy",
            policy: ChannelPolicy {
                actions: vec![Action::Allow, Action::Challenge, Action::Block],
                reward: high_catch_reward,
                ..ChannelPolicy::default()
            },
            q_hat_c: 0.31,
        },
    ]
}

fn sparse_vs_trained_case() -> SufficientStatisticCase {
    let sparse_core = CoreStateWitness {
        model_family: "prior-only",
        label_history: "none",
        labels_seen: 0,
        registered_sentinels: 0,
        identity_cells: 0,
        outcome_axes: 0,
        outcome_predictions: Vec::new(),
        sentinel_alarm_fingerprint: 0,
    };
    let trained_core = CoreStateWitness {
        model_family: "trained-with-axes",
        label_history: "mixed-benign-adverse",
        labels_seen: 12_000,
        registered_sentinels: 8,
        identity_cells: 96,
        outcome_axes: 3,
        outcome_predictions: vec![OutcomePredictionWitness {
            axis_name: "magnitude",
            predicted_raw: 4.25,
            uncertainty: 0.38,
        }],
        sentinel_alarm_fingerprint: 0x5E17_1A1A,
    };
    SufficientStatisticCase {
        left: sparse_core,
        right: trained_core,
        statistic: baseline_statistic(),
    }
}

fn rich_core_history_case() -> SufficientStatisticCase {
    let report_heavy_core = CoreStateWitness {
        model_family: "reported-sentinel-heavy",
        label_history: "allow-challenge-block-ground-truth",
        labels_seen: 2_400,
        registered_sentinels: 5,
        identity_cells: 48,
        outcome_axes: 2,
        outcome_predictions: vec![OutcomePredictionWitness {
            axis_name: "loss",
            predicted_raw: 9.5,
            uncertainty: 0.72,
        }],
        sentinel_alarm_fingerprint: 0xA1A1_0005,
    };
    let identity_heavy_core = CoreStateWitness {
        model_family: "identity-heavy",
        label_history: "identity-cycle-benign-adverse",
        labels_seen: 2_400,
        registered_sentinels: 2,
        identity_cells: 192,
        outcome_axes: 1,
        outcome_predictions: vec![OutcomePredictionWitness {
            axis_name: "loss",
            predicted_raw: -3.25,
            uncertainty: 1.41,
        }],
        sentinel_alarm_fingerprint: 0x1D1D_0002,
    };
    SufficientStatisticCase {
        left: report_heavy_core,
        right: identity_heavy_core,
        statistic: baseline_statistic(),
    }
}

fn divergent_prediction_payload_case() -> SufficientStatisticCase {
    let no_prediction_core = CoreStateWitness {
        model_family: "risk-only",
        label_history: "adverse-risk-labels-only",
        labels_seen: 96,
        registered_sentinels: 1,
        identity_cells: 12,
        outcome_axes: 0,
        outcome_predictions: Vec::new(),
        sentinel_alarm_fingerprint: 0x0074_0001,
    };
    let prediction_rich_core = CoreStateWitness {
        model_family: "risk-plus-outcome-predictions",
        label_history: "same-risk-labels-plus-outcome-values",
        labels_seen: 96,
        registered_sentinels: 1,
        identity_cells: 12,
        outcome_axes: 2,
        outcome_predictions: vec![
            OutcomePredictionWitness {
                axis_name: "magnitude",
                predicted_raw: 11.75,
                uncertainty: 0.19,
            },
            OutcomePredictionWitness {
                axis_name: "duration",
                predicted_raw: -4.5,
                uncertainty: 2.25,
            },
        ],
        sentinel_alarm_fingerprint: 0x0074_0001,
    };
    SufficientStatisticCase {
        left: no_prediction_core,
        right: prediction_rich_core,
        statistic: baseline_statistic(),
    }
}

fn sufficient_statistic_cases() -> Vec<(&'static str, SufficientStatisticCase)> {
    vec![
        ("sparse vs trained", sparse_vs_trained_case()),
        ("rich core histories", rich_core_history_case()),
        ("divergent outcome payloads", divergent_prediction_payload_case()),
    ]
}

#[track_caller]
fn assert_same_statistic_replays_for_policy(case: &SufficientStatisticCase, policy: &ChannelPolicy, q_hat_c: f64, label: &str) {
    assert_ne!(
        case.left.fingerprint(),
        case.right.fingerprint(),
        "{label}: witnesses must represent distinct core states"
    );

    let left_input = sufficient_statistic_from(&case.left, &case.statistic);
    let right_input = sufficient_statistic_from(&case.right, &case.statistic);

    let left_tags = derive_profile(&left_input, policy, q_hat_c);
    let right_tags = derive_profile(&right_input, policy, q_hat_c);

    let numeric_payload: Vec<f64> = left_tags
        .iter()
        .chain(right_tags.iter())
        .flat_map(|tag| [tag.location, tag.magnitude, tag.q])
        .collect();
    assert_finite(&numeric_payload, label);

    assert_profile_bit_identical(&left_tags, &right_tags, label);
}

#[track_caller]
fn assert_same_statistic_replays_for_policy_grid(case: &SufficientStatisticCase, label: &str) {
    for fixture in policy_fixtures() {
        assert_same_statistic_replays_for_policy(case, &fixture.policy, fixture.q_hat_c, &format!("{label}: {}", fixture.label));
    }
}

#[track_caller]
fn assert_outcome_prediction_payloads_diverge(left: &CoreStateWitness, right: &CoreStateWitness, label: &str) {
    let left_predictions = outcome_prediction_fingerprint(&left.outcome_predictions);
    let right_predictions = outcome_prediction_fingerprint(&right.outcome_predictions);
    assert_ne!(
        left_predictions, right_predictions,
        "{label}: outcome-prediction payloads should differ"
    );
}

#[track_caller]
fn assert_profile_bit_identical(actual: &[TagResonance], expected: &[TagResonance], label: &str) {
    assert_eq!(actual.len(), expected.len(), "{label}: tag count mismatch");

    for (index, (actual_tag, expected_tag)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(actual_tag.tag, expected_tag.tag, "{label}: tag #{index} kind mismatch");
        assert_eq!(
            actual_tag.location.to_bits(),
            expected_tag.location.to_bits(),
            "{label}: tag #{index} {:?} location mismatch",
            actual_tag.tag,
        );
        assert_eq!(
            actual_tag.magnitude.to_bits(),
            expected_tag.magnitude.to_bits(),
            "{label}: tag #{index} {:?} magnitude mismatch",
            actual_tag.tag,
        );
        assert_eq!(
            actual_tag.q.to_bits(),
            expected_tag.q.to_bits(),
            "{label}: tag #{index} {:?} q mismatch",
            actual_tag.tag,
        );
        assert_eq!(
            actual_tag.dominated, expected_tag.dominated,
            "{label}: tag #{index} {:?} dominated mismatch",
            actual_tag.tag,
        );
    }
}

/// Two entities whose model histories share nothing — one with no labels, no
/// Sentinels and no identity structure at all, the other trained on twelve
/// thousand labels across dozens of identity cells — derive bit-identical tag
/// profiles once they reduce to the same four scalars under the same policy. The
/// statistic really is sufficient: everything the core learned reaches the
/// decision layer through it or not at all.
///
/// ´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´
/// ´test:crate:same-sufficient-statistic-and-policy-replays-identical-profile´
#[test]
fn same_sufficient_statistic_and_policy_replays_identical_profile() {
    // The derivation layer cannot observe core model state directly
    // (´dec:ordering:closed-crossing´). These witnesses differ in every field
    // that would belong to the core side of the architecture, then
    // reduce to the same sufficient statistic before entering
    // the layered pipeline.
    let policy = ChannelPolicy::default();
    let case = sparse_vs_trained_case();
    assert_same_statistic_replays_for_policy(&case, &policy, 0.57, "same sufficient statistic + same policy");
}

/// Sufficiency is not an artefact of the default four-action channel. The same
/// pair of witnesses replays identically under a two-action permit-or-deny
/// channel, a three-action channel, a four-action channel whose friction is
/// quadrupled, and one whose catch reward is raised — five shapes with three
/// different challenge estimates. Changing what the host asks for changes the
/// answer, but never reopens a path back into the core's private state.
///
/// (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´)
/// ´test:crate:same-sufficient-statistic-replays-identical-profile-across-policy-shapes´
#[test]
fn same_sufficient_statistic_replays_identical_profile_across_policy_shapes() {
    // Policy-shape sweep (´dec:ordering:closed-crossing´): sufficiency is not a quirk of the
    // default four-action policy. Once two distinct core histories reduce
    // to the same `KernelScalars`, derivation must replay bit-identically
    // for every fixed explicit decision-layer input: two-action API shape,
    // three-action challenge shape, and a four-action policy whose reward
    // block is deliberately perturbed.
    let case = sparse_vs_trained_case();
    for fixture in policy_fixtures() {
        assert_same_statistic_replays_for_policy(&case, &fixture.policy, fixture.q_hat_c, fixture.label);
    }
}

/// The sparse-against-trained contrast is the easy case; the property also holds
/// between two histories that are both plausibly live. A report-heavy core with
/// five Sentinels and a forty-eight-cell identity footprint, and an
/// identity-heavy core with two Sentinels and four times the cells — matched on
/// label count but differing in every other respect, including the sign of their
/// outcome predictions — still replay to the same bits.
///
/// (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´)
/// ´test:crate:rich-core-witnesses-with-same-sufficient-statistic-replay-identical-profile´
#[test]
fn rich_core_witnesses_with_same_sufficient_statistic_replay_identical_profile() {
    // Rich-witness slice (´dec:ordering:closed-crossing´): the basic sufficiency test uses a sparse
    // prior-only witness against a trained witness. This companion keeps the
    // same fixed `KernelScalars` but makes both sides look like plausible live
    // core histories: different Sentinel/report mix, identity footprint, label
    // path, and outcome-prediction payload. None of that state may be visible to
    // the layered pipeline once the core has reduced it to the sufficient
    // statistic.
    let policy = ChannelPolicy::default();
    let case = rich_core_history_case();
    assert_outcome_prediction_payloads_diverge(&case.left, &case.right, "rich core witnesses");
    assert_same_statistic_replays_for_policy(&case, &policy, 0.57, "rich core witnesses + same statistic");
}

/// Outcome-axis predictions may be published beside a risk assessment, but the
/// derivation has no parameter slot for them: a risk-only witness and one
/// carrying two predicted axes with confirmedly different payloads produce the
/// same profile down to the bit. What the model expects the *magnitude* of an
/// adverse outcome to be is information for the host to act on separately, and
/// cannot leak sideways into where the action tags are placed.
///
/// ´claim:resonance:outcome-predictions-travel-beside-the-assessment-without-entering-the-derivation´
/// ´test:crate:outcome-prediction-payload-is-not-a-resonance-derivation-input´
#[test]
fn outcome_prediction_payload_is_not_a_resonance_derivation_input() {
    // Pure-boundary slice (´def:landscape:outcome-neutrality´): outcome-axis prediction outputs may be
    // published beside the risk assessment, but the resonance derivation has no
    // parameter slot for them. A risk-only witness and a prediction-rich witness
    // with the same fixed statistic must therefore derive bit-identical tag
    // profiles under the same channel policy.
    let policy = ChannelPolicy::default();
    let case = divergent_prediction_payload_case();
    assert_outcome_prediction_payloads_diverge(&case.left, &case.right, "prediction payload neutrality");
    assert_same_statistic_replays_for_policy(&case, &policy, 0.57, "prediction payload outside resonance input");
}

/// Taking every witness pair against every policy fixture at once closes the
/// matrix: three kinds of core divergence crossed with five channel shapes and
/// three challenge estimates all replay identically. The two halves of the
/// boundary are independent — no combination of a particular history with a
/// particular policy opens a channel that neither opens alone.
///
/// (´claim:resonance:the-derivation-sees-nothing-of-the-core-beyond-the-sufficient-statistic´)
/// ´test:crate:same-sufficient-statistic-replays-identical-profile-across-witness-policy-grid´
#[test]
fn same_sufficient_statistic_replays_identical_profile_across_witness_policy_grid() {
    // Grid slice (´dec:ordering:closed-crossing´): the same fixed-triple boundary should survive
    // both sides of the matrix. Core witnesses vary label history, Sentinel
    // footprint, identity footprint, and outcome-prediction payloads; decision
    // fixtures vary action shape, reward parameters, and q̂_c. Only the explicit
    // `KernelScalars` and fixed decision-layer inputs are allowed to determine
    // the profile for each cell.
    for (case_label, case) in sufficient_statistic_cases() {
        assert_same_statistic_replays_for_policy_grid(&case, case_label);
    }
}

/// The statistic is sufficient without being padded: shifting any one of its
/// four scalars — the probability, the effective spread, the condition number,
/// or the challenge-estimate variance — with the policy held fixed moves the tag
/// profile somewhere. Each field is genuinely read, so the replay results above
/// cannot be passing because the derivation has quietly stopped consulting one
/// of its inputs.
///
/// ´claim:resonance:every-scalar-in-the-sufficient-statistic-actually-moves-the-tag-profile´
/// ´test:crate:sufficient-statistic-fields-are-live-derivation-inputs´
#[test]
fn sufficient_statistic_fields_are_live_derivation_inputs() {
    // Liveness companion: if the statistic itself moves while the
    // channel policy stays fixed, the tag profile must move. Exercise
    // every scalar in `KernelScalars`, not just `p_hat`, so the
    // sufficiency witness above cannot pass while the derivation has
    // silently stopped reading one field.
    let policy = ChannelPolicy::default();
    let baseline = KernelScalars {
        p_hat: 0.37,
        sigma_eff: 0.58,
        kappa_eff: 1.25,
        sigma2_q_hat_c: 0.031,
    };
    let baseline_tags = derive_profile(&baseline, &policy, 0.57);

    let cases = [
        (
            "p_hat",
            KernelScalars {
                p_hat: 0.62,
                ..baseline.clone()
            },
        ),
        (
            "sigma_eff",
            KernelScalars {
                sigma_eff: 0.91,
                ..baseline.clone()
            },
        ),
        (
            "kappa_eff",
            KernelScalars {
                kappa_eff: 1.80,
                ..baseline.clone()
            },
        ),
        (
            "sigma2_q_hat_c",
            KernelScalars {
                sigma2_q_hat_c: 0.19,
                ..baseline
            },
        ),
    ];

    for (field, shifted) in cases {
        let shifted_tags = derive_profile(&shifted, &policy, 0.57);
        assert_tag_profile_diverges_somewhere(
            &baseline_tags,
            &shifted_tags,
            EPS,
            &format!("{field} perturbation should move the resonance profile"),
        );
    }
}

/// The host's own two levers are live in the same sense: quadrupling the
/// friction reward, or raising the challenge-effectiveness estimate from 0.57 to
/// 0.82, each moves the profile while the core statistic stands still. The
/// replay guarantee is conditional on holding these fixed, and that condition
/// has teeth — a host retuning its rewards genuinely gets a different channel,
/// not the same one relabelled.
///
/// ´claim:resonance:reward-parameters-and-the-challenge-estimate-are-live-inputs-of-the-decision-layer´
/// ´test:crate:policy-reward-and-challenge-estimate-are-live-derivation-inputs´
#[test]
fn policy_reward_and_challenge_estimate_are_live_derivation_inputs() {
    // Liveness companion for the explicit decision-layer inputs: the
    // sufficiency assertions above hold only when the channel policy and
    // challenge-effectiveness estimate are fixed. Perturbing either input,
    // while holding the core statistic fixed, must move the profile.
    let input = KernelScalars {
        p_hat: 0.37,
        sigma_eff: 0.58,
        kappa_eff: 1.25,
        sigma2_q_hat_c: 0.031,
    };
    let policy = ChannelPolicy::default();
    let baseline_tags = derive_profile(&input, &policy, 0.57);

    let mut high_friction_policy = ChannelPolicy::default();
    high_friction_policy.reward.friction *= 4.0;
    let high_friction_tags = derive_profile(&input, &high_friction_policy, 0.57);
    assert_tag_profile_diverges_somewhere(
        &baseline_tags,
        &high_friction_tags,
        EPS,
        "reward perturbation should move the resonance profile",
    );

    let high_challenge_estimate_tags = derive_profile(&input, &policy, 0.82);
    assert_tag_profile_diverges_somewhere(
        &baseline_tags,
        &high_challenge_estimate_tags,
        EPS,
        "challenge-effectiveness estimate should move the resonance profile",
    );
}
