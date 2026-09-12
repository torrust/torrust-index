// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{LabelSpec, World, assert_finite, assert_outcome_prediction_near_with_drift};
use torrust_assayer::types::OutcomeAxisId;
use torrust_assayer::{DerivedReckoning, RequestContext};

use super::labels::cycle_request_pair;

/// Pair of outcome axes trained with opposite raw values in matched worlds.
///
/// The public outcome-isolation witnesses
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// use this when the risk-label stream is identical but the emitted prediction
/// payloads must diverge. Keeping the two axis ids
/// together prevents matrix tests from accidentally mixing the positive and
/// negative world-local registrations.
#[derive(Clone, Copy, Debug)]
pub struct DivergentOutcomeAxisPair {
    /// Axis registered in the positive/high-output world.
    pub positive: OutcomeAxisId,
    /// Axis registered in the negative/low-output world.
    pub negative: OutcomeAxisId,
}

/// Canonical high raw value for the divergent-output witnesses
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´).
pub const STANDARD_DIVERGENT_OUTCOME_POSITIVE_VALUE: f64 = 4.0;

/// Canonical low raw value for the divergent-output witnesses
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´).
pub const STANDARD_DIVERGENT_OUTCOME_NEGATIVE_VALUE: f64 = -4.0;

/// Canonical training length for public divergent-output witnesses.
pub const STANDARD_DIVERGENT_OUTCOME_TRAINING_CYCLES: usize = 12;

/// Canonical raw value for public live-outcome composition witnesses.
pub const STANDARD_LIVE_OUTCOME_VALUE: f64 = 4.0;

/// Canonical training length for public live-outcome composition witnesses.
pub const STANDARD_LIVE_OUTCOME_TRAINING_CYCLES: usize = 12;

/// Train one outcome axis with the standard live-prediction fixture.
///
/// Most composition tests
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// only need an outcome prediction to be present and shared as public core
/// payload. This helper keeps the ordinary `4.0` / 12
/// cycle setup in one place while callers still choose the request shape:
/// plain, reported, signal-rich, or any combination of those public inputs.
#[track_caller]
pub fn train_standard_live_outcome_prediction_with_requests<F>(
    world: &World,
    axis: OutcomeAxisId,
    request_for_cycle: F,
    label: &str,
) where
    F: FnMut(usize, &World) -> RequestContext,
{
    train_outcome_prediction_with_requests(
        world,
        axis,
        STANDARD_LIVE_OUTCOME_VALUE,
        STANDARD_LIVE_OUTCOME_TRAINING_CYCLES,
        request_for_cycle,
        label,
    );
}

/// Train two matched worlds with the standard live-prediction fixture.
///
/// The two worlds advance cycle-by-cycle so persistent decay sees
/// matched histories that are as close together as the public API permits.
#[track_caller]
pub fn train_standard_live_outcome_prediction_pair_with_requests<F>(
    left_world: &World,
    left_axis: OutcomeAxisId,
    right_world: &World,
    right_axis: OutcomeAxisId,
    request_for_cycle: F,
    label: &str,
) where
    F: Copy + Fn(usize, &World) -> RequestContext,
{
    train_outcome_prediction_pair_with_matching_requests(
        left_world,
        left_axis,
        right_world,
        right_axis,
        STANDARD_LIVE_OUTCOME_VALUE,
        STANDARD_LIVE_OUTCOME_VALUE,
        STANDARD_LIVE_OUTCOME_TRAINING_CYCLES,
        request_for_cycle,
        label,
    );
}

/// Train one outcome axis until public assessments emit a live prediction.
///
/// The helper intentionally drives ordinary public labels instead of reaching
/// into model state. It is used by the composition tests
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// that need outcome predictions to exist before probing the resonance boundary.
#[track_caller]
pub fn train_outcome_prediction(
    world: &World,
    channel: &'static str,
    entity: &str,
    axis: OutcomeAxisId,
    value: f64,
    cycles: usize,
) {
    train_outcome_prediction_with_requests(
        world,
        axis,
        value,
        cycles,
        |_, world| world.request(channel, entity),
        "outcome training",
    );
}

/// Train one outcome axis with caller-built public requests.
///
/// This is the common outcome-training fixture
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// for richer request shapes:
/// request-scoped signals, live Sentinel coordinates, or both. The helper owns
/// the assess/label/flush loop while the caller supplies only the request for
/// each cycle.
#[track_caller]
pub fn train_outcome_prediction_with_requests<F>(
    world: &World,
    axis: OutcomeAxisId,
    value: f64,
    cycles: usize,
    mut request_for_cycle: F,
    label: &str,
) where
    F: FnMut(usize, &World) -> RequestContext,
{
    for cycle in 0..cycles {
        let request = request_for_cycle(cycle, world);
        let reckoning = world
            .derive_for_request(request)
            .unwrap_or_else(|error| panic!("{label} cycle {cycle}: assess failed: {error:?}"));
        world
            .label(LabelSpec::adverse(reckoning.assessment.id).outcome(axis, value).build())
            .unwrap_or_else(|error| panic!("{label} cycle {cycle}: label failed: {error:?}"));
        world
            .flush_labels()
            .unwrap_or_else(|error| panic!("{label} cycle {cycle}: flush failed: {error:?}"));
    }
}

/// Train two matched worlds with the standard opposite outcome values.
///
/// Use this when both worlds should receive the same request shape for each
/// cycle. The helper keeps the common `4.0` / `-4.0` / `12 cycles` fixture in
/// one place while callers still choose whether the request is plain, reported,
/// signal-rich, or deliberately degraded.
#[track_caller]
pub fn train_standard_divergent_outcome_prediction_pair_with_requests<F>(
    positive_world: &World,
    positive_axis: OutcomeAxisId,
    negative_world: &World,
    negative_axis: OutcomeAxisId,
    request_for_cycle: F,
    label: &str,
) where
    F: Copy + Fn(usize, &World) -> RequestContext,
{
    train_divergent_outcome_prediction_pair_with_matching_requests(
        positive_world,
        positive_axis,
        negative_world,
        negative_axis,
        STANDARD_DIVERGENT_OUTCOME_POSITIVE_VALUE,
        STANDARD_DIVERGENT_OUTCOME_NEGATIVE_VALUE,
        STANDARD_DIVERGENT_OUTCOME_TRAINING_CYCLES,
        request_for_cycle,
        label,
    );
}

/// Train two matched worlds with caller-supplied opposite outcome values and
/// the same request shape on both sides.
#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: paired-world fixture keeps axis ids, values, cycle count, and request builder explicit.
pub fn train_divergent_outcome_prediction_pair_with_matching_requests<F>(
    positive_world: &World,
    positive_axis: OutcomeAxisId,
    negative_world: &World,
    negative_axis: OutcomeAxisId,
    positive_value: f64,
    negative_value: f64,
    cycles: usize,
    request_for_cycle: F,
    label: &str,
) where
    F: Copy + Fn(usize, &World) -> RequestContext,
{
    train_outcome_prediction_pair_with_matching_requests(
        positive_world,
        positive_axis,
        negative_world,
        negative_axis,
        positive_value,
        negative_value,
        cycles,
        request_for_cycle,
        label,
    );
}

#[track_caller]
#[allow(clippy::too_many_arguments)] // Justified: paired-world fixture keeps axis ids, values, cycle count, and request builder explicit.
fn train_outcome_prediction_pair_with_matching_requests<F>(
    positive_world: &World,
    positive_axis: OutcomeAxisId,
    negative_world: &World,
    negative_axis: OutcomeAxisId,
    positive_value: f64,
    negative_value: f64,
    cycles: usize,
    request_for_cycle: F,
    label: &str,
) where
    F: Copy + Fn(usize, &World) -> RequestContext,
{
    assert!(cycles > 0, "{label}: at least one outcome-training cycle is required");

    for cycle in 0..cycles {
        cycle_request_pair(
            positive_world,
            request_for_cycle(cycle, positive_world),
            |assessment_id| LabelSpec::adverse(assessment_id).outcome(positive_axis, positive_value),
            negative_world,
            request_for_cycle(cycle, negative_world),
            |assessment_id| LabelSpec::adverse(assessment_id).outcome(negative_axis, negative_value),
            &format!("{label} cycle {cycle}"),
        );
    }
}

/// Assert every reckoning carries the same prediction within a caller-supplied drift budget.
#[track_caller]
pub fn assert_outcome_prediction_shared_by_all_with_drift(
    assessments: &[DerivedReckoning],
    axis: OutcomeAxisId,
    drift: f64,
    label: &str,
) {
    assert!(!assessments.is_empty(), "{label}: at least one reckoning is required");

    let first = &assessments[0];
    assert!(
        first.assessment.outcome_predictions.contains_key(&axis),
        "{label}: first reckoning is missing prediction for {axis:?}",
    );
    for (index, reckoning) in assessments.iter().enumerate().skip(1) {
        assert_outcome_prediction_near_with_drift(
            first,
            reckoning,
            axis,
            drift,
            &format!("{label}: first vs reckoning #{index}"),
        );
    }
}

/// Assert that two live outcome predictions are ordered by raw prediction value.
///
/// The public outcome-isolation witnesses
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
/// often train two same-risk worlds with opposite outcome values, then compare resonance from the same `RiskBasis`. This
/// helper keeps the prediction-output liveness check beside the other outcome
/// helpers so those tests can focus on the derivation invariant.
#[track_caller]
pub fn assert_outcome_prediction_raw_ordered(
    higher: &DerivedReckoning,
    higher_axis: OutcomeAxisId,
    lower: &DerivedReckoning,
    lower_axis: OutcomeAxisId,
    min_gap: f64,
    label: &str,
) {
    assert!(min_gap >= 0.0, "{label}: min_gap must be non-negative");

    let higher_prediction = higher
        .assessment
        .outcome_predictions
        .get(&higher_axis)
        .unwrap_or_else(|| panic!("{label}: higher reckoning is missing prediction for {higher_axis:?}"));
    let lower_prediction = lower
        .assessment
        .outcome_predictions
        .get(&lower_axis)
        .unwrap_or_else(|| panic!("{label}: lower reckoning is missing prediction for {lower_axis:?}"));

    assert_finite(
        &[
            higher_prediction.predicted_raw,
            higher_prediction.uncertainty,
            higher_prediction.prediction_interval.0,
            higher_prediction.prediction_interval.1,
            lower_prediction.predicted_raw,
            lower_prediction.uncertainty,
            lower_prediction.prediction_interval.0,
            lower_prediction.prediction_interval.1,
        ],
        label,
    );
    assert!(
        higher_prediction.predicted_raw > lower_prediction.predicted_raw + min_gap,
        "{label}: expected {higher_axis:?} predicted_raw ({}) to exceed {lower_axis:?} predicted_raw ({}) by more than {min_gap}",
        higher_prediction.predicted_raw,
        lower_prediction.predicted_raw,
    );
}
