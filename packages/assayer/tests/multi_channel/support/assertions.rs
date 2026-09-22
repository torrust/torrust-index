// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_assayer::testing::{
    PURITY_DRIFT, assert_core_assessment_near, assert_resonance_profile_near_with_drift, assert_risk_basis_near,
    assert_signal_degradation_counts, assert_tag_profile_diverges_somewhere, assert_tag_profile_near,
};
use torrust_assayer::types::{OutcomeAxisId, SentinelId};
use torrust_assayer::{DerivedReckoning, Tag, TagResonance};

/// Expected signal-degradation footprint for a core assessment payload.
#[derive(Clone, Copy, Debug)]
pub struct SignalDegradationExpectation {
    /// Expected sanitised-signal count.
    pub signals_sanitised: u32,
    /// Expected shape-mismatched-signal count.
    pub signals_shape_mismatched: u32,
}

/// Expected public core payload carried by a reckoning.
///
/// The integration tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// use this to keep the liveness witness for rich core paths close to the
/// shared fixed-core assertions: live Sentinel reports,
/// outcome predictions, and inline signal-degradation diagnostics all belong to
/// the core assessment, not to per-channel resonance derivation.
#[derive(Clone, Copy, Debug, Default)]
pub struct CorePayloadExpectation {
    /// Sentinel whose live report should appear in the core payload.
    pub sentinel: Option<SentinelId>,
    /// Outcome axis whose live prediction should appear in the core payload.
    pub outcome_axis: Option<OutcomeAxisId>,
    /// Expected signal degradation footprint, when the request is intentionally degraded.
    pub signal_degradation: Option<SignalDegradationExpectation>,
}

impl CorePayloadExpectation {
    /// Expect a live Sentinel alarm summary in the core payload.
    #[must_use]
    pub const fn live_sentinel(sentinel: SentinelId) -> Self {
        Self {
            sentinel: Some(sentinel),
            outcome_axis: None,
            signal_degradation: None,
        }
    }

    /// Expect a live outcome prediction in the core payload.
    #[must_use]
    pub const fn outcome(axis: OutcomeAxisId) -> Self {
        Self {
            sentinel: None,
            outcome_axis: Some(axis),
            signal_degradation: None,
        }
    }

    /// Expect both a live Sentinel alarm summary and a live outcome prediction.
    #[must_use]
    pub const fn reported_outcome(sentinel: SentinelId, axis: OutcomeAxisId) -> Self {
        Self {
            sentinel: Some(sentinel),
            outcome_axis: Some(axis),
            signal_degradation: None,
        }
    }

    /// Expect only a signal-degradation footprint in the core payload.
    #[must_use]
    pub const fn signal_degradation(signals_sanitised: u32, signals_shape_mismatched: u32) -> Self {
        Self {
            sentinel: None,
            outcome_axis: None,
            signal_degradation: Some(SignalDegradationExpectation {
                signals_sanitised,
                signals_shape_mismatched,
            }),
        }
    }

    /// Add an expected signal-degradation footprint.
    #[must_use]
    pub const fn with_signal_degradation(mut self, signals_sanitised: u32, signals_shape_mismatched: u32) -> Self {
        self.signal_degradation = Some(SignalDegradationExpectation {
            signals_sanitised,
            signals_shape_mismatched,
        });
        self
    }
}

/// Number of action tags emitted by a reckoning.
///
/// Equal to the size of the channel policy's action set whenever
/// the engine derived the reckoning correctly
/// (´claim:channel:the-action-tags-of-a-reckoning-are-exactly-the-channels-declared-actions-in-order´).
#[must_use]
pub fn action_tag_count(r: &DerivedReckoning) -> usize {
    r.profile.tags.iter().filter(|t| t.tag.is_action()).count()
}

/// Number of classification tags emitted by a reckoning.
///
/// Always exactly 3 (Good, Suspicious, Malicious) — channel-
/// independent by construction.
#[must_use]
pub fn classification_tag_count(r: &DerivedReckoning) -> usize {
    r.profile.tags.iter().filter(|t| t.tag.is_classification()).count()
}

/// Action tags emitted by a reckoning, preserving engine order.
#[must_use]
pub fn action_tags(reckoning: &DerivedReckoning) -> Vec<TagResonance> {
    reckoning
        .profile
        .tags
        .iter()
        .copied()
        .filter(|tag| tag.tag.is_action())
        .collect()
}

/// Classification tags emitted by a reckoning, preserving engine order.
#[must_use]
pub fn classification_tags(reckoning: &DerivedReckoning) -> Vec<TagResonance> {
    reckoning
        .profile
        .tags
        .iter()
        .copied()
        .filter(|tag| tag.tag.is_classification())
        .collect()
}

/// Classification tag kinds emitted by a reckoning, sorted for set comparison.
#[must_use]
pub fn classification_tag_kinds(reckoning: &DerivedReckoning) -> Vec<Tag> {
    let mut kinds: Vec<Tag> = classification_tags(reckoning).into_iter().map(|tag| tag.tag).collect();
    kinds.sort_by_key(|tag| format!("{tag:?}"));
    kinds
}

/// Action tag kinds emitted by a reckoning, preserving engine order.
#[must_use]
pub fn action_tag_kinds(reckoning: &DerivedReckoning) -> Vec<Tag> {
    action_tags(reckoning).into_iter().map(|tag| tag.tag).collect()
}

/// Sum of the action-tag magnitudes.
///
/// In the resonance derivation this is the attenuated total policy
/// mass allocated to action regimes. When the core risk statistic is
/// unchanged and only the channel action set varies, this total stays
/// fixed while individual action regimes redistribute it.
#[must_use]
pub fn action_magnitude_sum(reckoning: &DerivedReckoning) -> f64 {
    action_tags(reckoning).into_iter().map(|tag| tag.magnitude).sum()
}

/// Assert that action tags appear in the exact expected order.
#[track_caller]
pub fn assert_action_tag_sequence(reckoning: &DerivedReckoning, expected: &[Tag], label: &str) {
    assert_eq!(action_tag_kinds(reckoning), expected, "{label}: action-tag sequence mismatch");
}

/// Assert that a reckoning's core assessment includes exactly one live Sentinel
/// report and carries that Sentinel's alarm summary.
///
/// The independence tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// use this after [`assert_core_assessment_near`] has already proven the
/// payload is shared across channels. This helper keeps the liveness witness
/// for the reported-measurement path in one place.
#[track_caller]
pub fn assert_live_sentinel_core_payload(reckoning: &DerivedReckoning, sentinel: SentinelId, label: &str) {
    assert_eq!(
        reckoning.assessment.risk.n_sentinels_reporting, 1,
        "{label}: exactly one live Sentinel report should feed the core",
    );
    assert!(
        reckoning.assessment.per_sentinel.contains_key(&sentinel),
        "{label}: live Sentinel alarm summary {sentinel:?} should be present",
    );
}

/// Assert that a reckoning's core assessment carries a live outcome prediction.
///
/// The prediction payload is core-side assessment data
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´). Tests pair this
/// presence check with shared-map comparisons to prove the prediction path was
/// live without letting it explain resonance movement.
#[track_caller]
pub fn assert_outcome_prediction_present(reckoning: &DerivedReckoning, axis: OutcomeAxisId, label: &str) {
    assert!(
        reckoning.assessment.outcome_predictions.contains_key(&axis),
        "{label}: live outcome-axis prediction {axis:?} should be present",
    );
}

/// Assert that one reckoning carries the expected public core payload.
#[track_caller]
pub fn assert_core_payload_expectation(reckoning: &DerivedReckoning, expectation: CorePayloadExpectation, label: &str) {
    if let Some(sentinel) = expectation.sentinel {
        assert_live_sentinel_core_payload(reckoning, sentinel, label);
    }

    if let Some(axis) = expectation.outcome_axis {
        assert_outcome_prediction_present(reckoning, axis, label);
    }

    if let Some(degradation) = expectation.signal_degradation {
        assert_signal_degradation_counts(
            reckoning,
            degradation.signals_sanitised,
            degradation.signals_shape_mismatched,
            label,
        );
    }
}

/// Assert that classification tags are unchanged across two assessments.
///
/// Classification tags are part of the channel-independent derivation
/// fingerprint
/// (´claim:channel:classification-tags-are-core-derived-and-therefore-identical-across-channels´);
/// action tags may move when the
/// decision layer changes, but classification tags should remain fixed
/// whenever the core statistic is fixed.
#[track_caller]
pub fn assert_classification_tags_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_classification_tags_near_with_drift(actual, expected, PURITY_DRIFT, label);
}

/// Assert that classification tags are unchanged within a caller-supplied drift budget.
#[track_caller]
pub fn assert_classification_tags_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    drift: f64,
    label: &str,
) {
    let actual_tags = classification_tags(actual);
    let expected_tags = classification_tags(expected);
    assert_tag_profile_near(&actual_tags, &expected_tags, drift, label);
}

/// Assert that two assessments share the sufficient risk statistic and resonance
/// profile within a caller-supplied drift budget.
#[track_caller]
pub fn assert_risk_basis_and_resonance_profile_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    drift: f64,
    label: &str,
) {
    assert_risk_basis_near(
        &actual.assessment.risk,
        &expected.assessment.risk,
        drift,
        &format!("{label}: RiskBasis"),
    );
    assert_resonance_profile_near_with_drift(actual, expected, drift, &format!("{label}: resonance profile"));
}

/// Assert the independence shape
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´):
/// the core is fixed, but the action side of the decision-layer profile moves.
///
/// The two assessments must have the same tag shape. Use this for
/// reward-parameter and challenge-tracker probes where action sets are
/// intentionally identical. Classification tags are asserted stable
/// separately so the liveness check lands on the action tags rather
/// than being satisfied by an unintended core-side movement.
#[track_caller]
pub fn assert_action_layer_moves_without_core(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_core_assessment_near(actual, expected, &format!("{label}: core assessment"));
    assert_classification_tags_near(actual, expected, &format!("{label}: classification tags"));

    let actual_actions = action_tags(actual);
    let expected_actions = action_tags(expected);
    assert_tag_profile_diverges_somewhere(
        &actual_actions,
        &expected_actions,
        PURITY_DRIFT,
        &format!("{label}: action tags"),
    );
}
