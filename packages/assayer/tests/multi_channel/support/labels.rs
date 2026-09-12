// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{LabelSpec, World, cycle_request};
use torrust_assayer::types::{Action, AssessmentId, ChallengeResult};
use torrust_assayer::{DerivedReckoning, RequestContext};

/// One public label-cycle fixture used by matched-world probes
/// (´claim:channel:routing-labels-through-differently-rewarded-channels-teaches-the-core-the-same-thing´).
///
/// This represents the observable host feedback knobs that can otherwise
/// make reward/core and challenge/core independence tests noisy: entity,
/// action, valence, optional ground-truth override, and optional challenge
/// result. Keeping the conversion to [`LabelSpec`] here lets scenario tests
/// declare streams as data instead of open-coding label builders per cycle.
#[derive(Clone, Copy, Debug)]
pub struct LabelCycleCase<'a> {
    /// Entity to assess before submitting this label.
    pub entity: &'a str,
    /// Host action reported with the label.
    pub action: Action,
    /// Whether the label is adverse (`true`) or benign (`false`).
    pub adverse: bool,
    /// Whether the label should bypass action-confounding eligibility.
    pub ground_truth: bool,
    /// Optional challenge outcome reported by the host.
    pub challenge_result: Option<ChallengeResult>,
}

impl LabelCycleCase<'_> {
    /// Build the corresponding [`LabelSpec`] for a freshly-issued assessment ID.
    #[must_use]
    pub fn label_spec(self, assessment_id: AssessmentId) -> LabelSpec {
        let label = if self.adverse {
            LabelSpec::adverse(assessment_id)
        } else {
            LabelSpec::benign(assessment_id)
        }
        .action(self.action);

        let label = if self.ground_truth { label.ground_truth() } else { label };

        match self.challenge_result {
            Some(result) => label.challenge_result(result),
            None => label,
        }
    }
}

/// Matched label-cycle variants for two-world probes
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´).
///
/// Use this when the two worlds should see the same observation stream but a
/// deliberate host-feedback difference, such as `challenge_result: Some(_)` vs
/// `None`. Keeping the pair as data makes the difference visible at the call
/// site and lets the shared cycle helper own the assess/label/flush ceremony.
#[derive(Clone, Copy, Debug)]
pub struct LabelCyclePair<'a> {
    /// Label case submitted to the left world.
    pub left: LabelCycleCase<'a>,
    /// Label case submitted to the right world.
    pub right: LabelCycleCase<'a>,
}

/// Build the canonical adverse `Action::Challenge` label case for challenge
/// isolation
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´).
#[must_use]
pub const fn challenge_label_case(challenge_result: Option<ChallengeResult>, ground_truth: bool) -> LabelCycleCase<'static> {
    LabelCycleCase {
        entity: "alice",
        action: Action::Challenge,
        adverse: true,
        ground_truth,
        challenge_result,
    }
}

/// Build the canonical Pass/Fail or present/absent challenge-result pair.
#[must_use]
pub const fn challenge_label_pair(
    left_result: Option<ChallengeResult>,
    right_result: Option<ChallengeResult>,
    ground_truth: bool,
) -> LabelCyclePair<'static> {
    LabelCyclePair {
        left: challenge_label_case(left_result, ground_truth),
        right: challenge_label_case(right_result, ground_truth),
    }
}

/// Alternating Alice/Bob label stream used by the post-history probes
/// (´claim:channel:the-separation-of-channel-from-core-survives-a-core-already-moved-by-history´).
///
/// Labels use the default public action (`Allow`) and mark every
/// `adverse_every`th cycle adverse. This matches the hand-written streams those
/// tests used before the fixture was centralised.
pub fn alternating_entity_label_stream(cycles: usize, adverse_every: usize) -> impl Iterator<Item = LabelCycleCase<'static>> {
    assert!(adverse_every > 0, "adverse_every must be non-zero");

    (0..cycles).map(move |cycle| LabelCycleCase {
        entity: if cycle % 2 == 0 { "alice" } else { "bob" },
        action: Action::Allow,
        adverse: cycle % adverse_every == 0,
        ground_truth: false,
        challenge_result: None,
    })
}

/// Single-entity alternating-valence stream for post-history reward probes.
pub fn alternating_label_stream_for_entity(
    entity: &'static str,
    cycles: usize,
    adverse_every: usize,
) -> impl Iterator<Item = LabelCycleCase<'static>> {
    assert!(adverse_every > 0, "adverse_every must be non-zero");

    (0..cycles).map(move |cycle| LabelCycleCase {
        entity,
        action: Action::Allow,
        adverse: cycle % adverse_every == 0,
        ground_truth: false,
        challenge_result: None,
    })
}

/// Build an adverse `Action::Challenge` label builder with an optional outcome.
///
/// This is the common absent-result primitive
/// (´claim:channel:a-result-free-challenge-label-leaves-the-companion-tracker-untouched´). `Some(_)` feeds the
/// companion challenge tracker and follows the current eligible Challenge path;
/// `None` leaves the companion tracker untouched and follows the current
/// confounded Challenge path.
pub fn adverse_challenge_label(result: Option<ChallengeResult>) -> impl Copy + Fn(AssessmentId) -> LabelSpec {
    move |assessment_id| {
        let label = LabelSpec::adverse(assessment_id).action(Action::Challenge);
        match result {
            Some(result) => label.challenge_result(result),
            None => label,
        }
    }
}

/// Build an adverse `Action::Challenge` label builder with a fixed present outcome.
///
/// This is the common training primitive for Pass-vs-Fail probes
/// (´claim:channel:a-challenge-result-feeds-the-companion-tracker-alone-and-leaves-the-core-untouched´): it
/// holds model-update eligibility constant while varying only the
/// companion-tracker evidence.
pub fn adverse_challenge_result(result: ChallengeResult) -> impl Copy + Fn(AssessmentId) -> LabelSpec {
    adverse_challenge_label(Some(result))
}

/// One public label cycle for a named channel/entity request.
#[track_caller]
pub fn cycle_label_case(world: &World, channel: &'static str, case: LabelCycleCase<'_>, _label: &str) -> DerivedReckoning {
    cycle_request(world, world.request(channel, case.entity), |assessment_id| {
        case.label_spec(assessment_id)
    })
}

/// Drive a stream of public label cases through one world and channel.
#[track_caller]
pub fn cycle_label_case_stream<'a, I>(world: &World, channel: &'static str, stream: I, label: &str)
where
    I: IntoIterator<Item = LabelCycleCase<'a>>,
{
    for (cycle, case) in stream.into_iter().enumerate() {
        cycle_label_case(world, channel, case, &format!("{label} cycle {cycle}"));
    }
}

/// Matched `assess(request) -> label(...) -> flush` for two worlds.
///
/// Independence scenarios
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// often compare two same-seed worlds that see identical observations but
/// differ in one decision-layer input. This helper
/// keeps the paired cycle atomic at the test-call site and produces better
/// failure labels than two open-coded [`cycle_request`] calls.
#[track_caller]
pub fn cycle_request_pair<FL, FR>(
    left_world: &World,
    left_request: RequestContext,
    build_left_label: FL,
    right_world: &World,
    right_request: RequestContext,
    build_right_label: FR,
    label: &str,
) -> (DerivedReckoning, DerivedReckoning)
where
    FL: FnOnce(AssessmentId) -> LabelSpec,
    FR: FnOnce(AssessmentId) -> LabelSpec,
{
    let left_reckoning = left_world
        .derive_for_request(left_request)
        .unwrap_or_else(|error| panic!("{label}: left assess failed: {error:?}"));
    let right_reckoning = right_world
        .derive_for_request(right_request)
        .unwrap_or_else(|error| panic!("{label}: right assess failed: {error:?}"));

    let left_spec = build_left_label(left_reckoning.assessment.id);
    if left_spec.action_taken() == Action::Challenge
        && let Some(result) = left_spec.challenge_outcome()
    {
        let _recorded = left_world.record_challenge_result(left_reckoning.assessment.id, result);
    }
    let right_spec = build_right_label(right_reckoning.assessment.id);
    if right_spec.action_taken() == Action::Challenge
        && let Some(result) = right_spec.challenge_outcome()
    {
        let _recorded = right_world.record_challenge_result(right_reckoning.assessment.id, result);
    }

    left_world
        .label(left_spec.build())
        .unwrap_or_else(|error| panic!("{label}: left label failed: {error:?}"));
    right_world
        .label(right_spec.build())
        .unwrap_or_else(|error| panic!("{label}: right label failed: {error:?}"));

    left_world
        .flush_labels()
        .unwrap_or_else(|error| panic!("{label}: left flush failed: {error:?}"));
    right_world
        .flush_labels()
        .unwrap_or_else(|error| panic!("{label}: right flush failed: {error:?}"));

    (left_reckoning, right_reckoning)
}

/// One matched label cycle for two worlds with the same named channel.
///
/// The two worlds receive the same public observation and the same label
/// payload. Reward-independence tests use this to prove that differing channel
/// policies do not change the core update path
/// (´claim:channel:routing-labels-through-differently-rewarded-channels-teaches-the-core-the-same-thing´).
#[track_caller]
pub fn cycle_label_case_pair(
    left_world: &World,
    right_world: &World,
    channel: &'static str,
    case: LabelCycleCase<'_>,
    label: &str,
) -> (DerivedReckoning, DerivedReckoning) {
    cycle_request_pair(
        left_world,
        left_world.request(channel, case.entity),
        |assessment_id| case.label_spec(assessment_id),
        right_world,
        right_world.request(channel, case.entity),
        |assessment_id| case.label_spec(assessment_id),
        label,
    )
}

/// One matched label cycle for two worlds with intentionally different labels.
///
/// This is the variant-friendly companion to [`cycle_label_case_pair`]. It keeps
/// the public observation fixed by channel/entity while allowing the host label
/// payload to differ in one decision-layer field.
#[track_caller]
pub fn cycle_label_case_variant_pair(
    left_world: &World,
    right_world: &World,
    channel: &'static str,
    pair: LabelCyclePair<'_>,
    label: &str,
) -> (DerivedReckoning, DerivedReckoning) {
    cycle_request_pair(
        left_world,
        left_world.request(channel, pair.left.entity),
        |assessment_id| pair.left.label_spec(assessment_id),
        right_world,
        right_world.request(channel, pair.right.entity),
        |assessment_id| pair.right.label_spec(assessment_id),
        label,
    )
}

/// Drive a matched stream of variant [`LabelCyclePair`] values through two worlds.
#[track_caller]
pub fn cycle_label_case_variant_stream_pair<'a, I>(
    left_world: &World,
    right_world: &World,
    channel: &'static str,
    stream: I,
    label: &str,
) where
    I: IntoIterator<Item = LabelCyclePair<'a>>,
{
    // Both worlds settle their cold ramps before their histories diverge.
    // The ramp advances on the assessment clock and the steward applies it
    // asynchronously, so two worlds trained in parallel reach different
    // accepted counts depending only on how the threads were scheduled — and
    // a comparison taken afterwards would be reading that scheduling rather
    // than the thing under test (´inv:guarantee:evidence-authority´). Settled
    // first, both stand on the same empirical moments and the matched label
    // histories move them alike from there.
    left_world.settle_cold_ramp_with(&[left_world.request(channel, "alice")]);
    right_world.settle_cold_ramp_with(&[right_world.request(channel, "alice")]);

    for (cycle, pair) in stream.into_iter().enumerate() {
        cycle_label_case_variant_pair(left_world, right_world, channel, pair, &format!("{label} cycle {cycle}"));
    }
}

/// One label cycle for a reported Sentinel request.
///
/// Companion to [`World::cycle_on`] for the probes
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// whose core update path must include live Sentinel measurement features.
#[track_caller]
pub fn cycle_reported_label_case(
    world: &World,
    channel: &'static str,
    sentinel: &'static str,
    coordinate: u128,
    case: LabelCycleCase<'_>,
    label: &str,
) -> DerivedReckoning {
    let request = world.request_with_sentinel(channel, case.entity, sentinel, coordinate);
    let reckoning = world
        .derive_for_request(request)
        .unwrap_or_else(|error| panic!("{label}: reported assess failed: {error:?}"));
    world
        .label(case.label_spec(reckoning.assessment.id).build())
        .unwrap_or_else(|error| panic!("{label}: reported label failed: {error:?}"));
    world
        .flush_labels()
        .unwrap_or_else(|error| panic!("{label}: reported flush failed: {error:?}"));
    reckoning
}

/// Drive a stream of reported-Sentinel label cycles through one world.
#[track_caller]
pub fn cycle_reported_label_case_stream<'a, I>(
    world: &World,
    channel: &'static str,
    sentinel: &'static str,
    coordinate: u128,
    stream: I,
    label: &str,
) where
    I: IntoIterator<Item = LabelCycleCase<'a>>,
{
    for (cycle, case) in stream.into_iter().enumerate() {
        cycle_reported_label_case(world, channel, sentinel, coordinate, case, &format!("{label} cycle {cycle}"));
    }
}
