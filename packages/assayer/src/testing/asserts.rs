// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Domain-aware assertion helpers.
//!
//! # Stage 1
//!
//! Only the cheap, always-applicable checks are wired up. The richer
//! builder-style assertions (`assert_ledger_ewma(…).near(…)`,
//! `assert_auc(…).in_range(…)`, `assert_health(…).flag(…).asserted()`)
//! land in Stage 3 together with the snapshot and tape infrastructure
//! they depend on.

use super::drift::PURITY_DRIFT;
use super::world::World;
use crate::assessment::{DerivedReckoning, RiskBasis};
use crate::error::check_vec_nan;
use crate::resonance::tags::{Tag, TagResonance};
use crate::types::OutcomeAxisId;

/// Assert that a floating-point value is within `tol` of `expected`.
///
/// Uses the default tolerance if `tol` is `None`. Failure message
/// includes both absolute and relative error.
#[track_caller]
pub fn assert_near(actual: f64, expected: f64, tol: f64, label: &str) {
    let diff = (actual - expected).abs();
    assert!(
        diff <= tol,
        "{label}: expected {expected} ± {tol}, got {actual} (|Δ| = {diff})",
    );
}

/// Assert that `actual <= limit` with a clear failure message.
#[track_caller]
pub fn assert_below(actual: f64, limit: f64, label: &str) {
    assert!(actual <= limit, "{label}: expected ≤ {limit}, got {actual}");
}

/// Assert that `actual > 0` with a clear failure message.
#[track_caller]
pub fn assert_positive(actual: f64, label: &str) {
    assert!(actual > 0.0, "{label}: expected > 0, got {actual}");
}

/// Assert that `value` lies strictly within the open unit interval `(0, 1)`.
#[track_caller]
pub fn assert_in_unit_interval(value: f64, label: &str) {
    assert!(value > 0.0 && value < 1.0, "{label}: expected value in (0, 1), got {value}");
}

/// Assert that `values` contains no NaN or infinity.
#[track_caller]
pub fn assert_finite(values: &[f64], label: &str) {
    let bad = check_vec_nan(values);
    assert!(bad.is_empty(), "{label}: non-finite values at indices {bad:?}");
}

/// Assert that a [`DerivedReckoning`]'s top-line assessment risk fields are well-formed.
///
/// Combines the two checks every integration test repeats verbatim:
///
/// 1. `assessment.risk.p_bad` is finite (no NaN / no infinity).
/// 2. `assessment.risk.p_bad` lies strictly inside the open unit
///    interval `(0, 1)`.
///
/// `label` is prepended to the failure message so a stack of
/// assessments (e.g. inside a churn loop) can be told apart at a
/// glance.
#[track_caller]
pub fn assert_reckoning_well_formed(reckoning: &DerivedReckoning, label: &str) {
    assert_finite(&[reckoning.assessment.risk.p_bad], label);
    assert_in_unit_interval(reckoning.assessment.risk.p_bad, label);
}

/// Assert that two [`RiskBasis`] values match within `tol`.
///
/// Floating-point fields are compared with [`assert_near`]; discrete
/// fields (`bool`, counts) are compared exactly.
#[track_caller]
pub fn assert_risk_basis_near(actual: &RiskBasis, expected: &RiskBasis, tol: f64, label: &str) {
    assert_finite(
        &[
            actual.p_bad,
            actual.uncertainty,
            actual.anchor_weight,
            actual.rho_eff,
            actual.sigma_eff,
            actual.kappa_eff,
            actual.p_bad_sister,
            actual.p_bad_operational,
            actual.intervention_effectiveness,
            actual.borrowed_share,
            expected.p_bad,
            expected.uncertainty,
            expected.anchor_weight,
            expected.rho_eff,
            expected.sigma_eff,
            expected.kappa_eff,
            expected.p_bad_sister,
            expected.p_bad_operational,
            expected.intervention_effectiveness,
            expected.borrowed_share,
        ],
        label,
    );

    assert_near(actual.p_bad, expected.p_bad, tol, &format!("{label}: p_bad"));
    assert_near(
        actual.uncertainty,
        expected.uncertainty,
        tol,
        &format!("{label}: uncertainty"),
    );
    assert_near(
        actual.anchor_weight,
        expected.anchor_weight,
        tol,
        &format!("{label}: anchor_weight"),
    );
    // The borrowed share is a property of the models and of the request's own
    // features, so two bases that agree on everything else must agree on it
    // exactly rather than nearly: a channel is not a feature
    // (´dec:risk:evidence-only-uncertainty´).
    assert_near(
        actual.borrowed_share,
        expected.borrowed_share,
        tol,
        &format!("{label}: borrowed_share"),
    );
    assert_near(actual.rho_eff, expected.rho_eff, tol, &format!("{label}: rho_eff"));
    assert_near(actual.sigma_eff, expected.sigma_eff, tol, &format!("{label}: sigma_eff"));
    assert_near(actual.kappa_eff, expected.kappa_eff, tol, &format!("{label}: kappa_eff"));
    assert_near(
        actual.p_bad_sister,
        expected.p_bad_sister,
        tol,
        &format!("{label}: p_bad_sister"),
    );
    assert_near(
        actual.p_bad_operational,
        expected.p_bad_operational,
        tol,
        &format!("{label}: p_bad_operational"),
    );
    assert_near(
        actual.intervention_effectiveness,
        expected.intervention_effectiveness,
        tol,
        &format!("{label}: intervention_effectiveness"),
    );

    assert_eq!(
        actual.anchor_converged, expected.anchor_converged,
        "{label}: anchor_converged mismatch: {} vs {}",
        actual.anchor_converged, expected.anchor_converged,
    );
    assert_eq!(
        actual.n_sentinels_reporting, expected.n_sentinels_reporting,
        "{label}: n_sentinels_reporting mismatch: {} vs {}",
        actual.n_sentinels_reporting, expected.n_sentinels_reporting,
    );
    assert!(
        (actual.sister_regime_calibration_records - expected.sister_regime_calibration_records).abs() <= tol,
        "{label}: sister_regime_calibration_records mismatch: {} vs {}",
        actual.sister_regime_calibration_records,
        expected.sister_regime_calibration_records,
    );
    assert!(
        (actual.anchor_regime_calibration_records - expected.anchor_regime_calibration_records).abs() <= tol,
        "{label}: anchor_regime_calibration_records mismatch: {} vs {}",
        actual.anchor_regime_calibration_records,
        expected.anchor_regime_calibration_records,
    );
}

/// The emitted tag of one kind, taken out of a profile's tag slice.
///
/// A tag profile is a short flat slice in engine order, so a test that wants one kind by name searches it linearly; doing that once here keeps the search, and the message a missing kind fails with, out of every assertion that needs a single tag's payload. The tag comes back by value because [`TagResonance`] is `Copy` and callers only read the numbers on it, which leaves the result usable in exactly the places a borrow of the slice would have been.
///
/// # Panics
///
/// Panics if the profile emitted no tag of the requested kind.
#[must_use]
#[track_caller]
pub fn find_tag(tags: &[TagResonance], kind: Tag) -> TagResonance {
    tags.iter()
        .copied()
        .find(|tag| tag.tag == kind)
        .unwrap_or_else(|| panic!("tag {kind:?} not found"))
}

/// Assert that two tag profiles match within `tol`.
///
/// This checks the structural parts of the profile exactly (count,
/// order, tag kind, domination flags) and the floating-point payloads
/// (`location`, `magnitude`, `q`) within the supplied tolerance.
#[track_caller]
pub fn assert_tag_profile_near(actual: &[TagResonance], expected: &[TagResonance], tol: f64, label: &str) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{label}: tag count mismatch: {} vs {}",
        actual.len(),
        expected.len(),
    );

    for (index, (lhs, rhs)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            lhs.tag, rhs.tag,
            "{label}: tag #{index} kind mismatch: {:?} vs {:?}",
            lhs.tag, rhs.tag,
        );
        assert_eq!(
            lhs.dominated, rhs.dominated,
            "{label}: tag #{index} {:?} dominated mismatch",
            lhs.tag,
        );
        assert_near(
            lhs.location,
            rhs.location,
            tol,
            &format!("{label}: tag #{index} {:?} location", lhs.tag),
        );
        assert_near(
            lhs.magnitude,
            rhs.magnitude,
            tol,
            &format!("{label}: tag #{index} {:?} magnitude", lhs.tag),
        );
        assert_near(lhs.q, rhs.q, tol, &format!("{label}: tag #{index} {:?} q", lhs.tag));
    }
}

/// Assert that two equally-shaped tag profiles diverge in at least
/// one `(location, magnitude, q)` triple beyond `eps`.
///
/// This is the *liveness* counterpart of [`assert_tag_profile_near`]:
/// it pins the negative-half assertion that "the perturbation under
/// test actually moved a tag somewhere in the resonance," so a
/// regression that silently neutralised the perturbation cannot
/// pass. Tag *kinds* and the per-tag domination flag are **not**
/// required to differ — only one of the three floating-point
/// payloads needs to shift past `eps` on at least one tag.
///
/// # Panics
///
/// - The two profiles disagree on length (the action set must match).
/// - The tag kinds disagree position-by-position (the resonance is
///   ordered, and a kind reorder is itself a regression worth
///   flagging eagerly).
/// - Every position-aligned `(location, magnitude, q)` triple lies
///   within `eps` — i.e. the perturbation is invisible at the
///   resolution that matters.
#[track_caller]
pub fn assert_tag_profile_diverges_somewhere(actual: &[TagResonance], expected: &[TagResonance], eps: f64, label: &str) {
    assert_eq!(
        actual.len(),
        expected.len(),
        "{label}: tag count mismatch: {} vs {}",
        actual.len(),
        expected.len(),
    );

    let mut max_delta = 0.0_f64;
    let mut any_shift = false;
    for (index, (lhs, rhs)) in actual.iter().zip(expected.iter()).enumerate() {
        assert_eq!(
            lhs.tag, rhs.tag,
            "{label}: tag #{index} kind mismatch: {:?} vs {:?}",
            lhs.tag, rhs.tag,
        );
        let dl = (lhs.location - rhs.location).abs();
        let dm = (lhs.magnitude - rhs.magnitude).abs();
        let dq = (lhs.q - rhs.q).abs();
        max_delta = max_delta.max(dl).max(dm).max(dq);
        if dl > eps || dm > eps || dq > eps {
            any_shift = true;
        }
    }

    assert!(
        any_shift,
        "{label}: expected at least one tag's (location, magnitude, q) to shift past {eps}, but the largest observed delta was {max_delta} — the probe is degenerate",
    );
}

/// Assert that the world's health report is free of degradation signals.
///
/// Queries [`Assayer::health_summary`](crate::Assayer::health_summary)
/// — a lock-free ~100 ns read of the shared-state `ArcSwap`s — and
/// fails if any of the **per-assessment degradation counters** is
/// non-zero or if any of the **hard-failure flags** is set:
///
/// - `degradation.total_degraded` beyond what the reported batch-init
///   load conditions account for
/// - `degradation.signals_sanitised` (NaN/Inf signals replaced)
/// - `degradation.signals_shape_mismatched` (signal values zero-filled for schema mismatch)
/// - `degradation.signals_unknown` (undeclared signal names skipped)
/// - `degradation.sentinel_slots_zeroed` (Sentinel slot zeroed for NaN)
/// - `degradation.features_sanitised` (feature vector NaN guard fired)
/// - `degradation.model_fallbacks` (model fell back to prior)
/// - `any_cascade_terminus` — any model hit the regularisation cascade
///   terminus, the hard numerical failure mode the cascade flags
///   rather than raises (´dec:posterior:cascade-never-fails´)
/// - `health_events_dropped` (health-event channel overflowed)
/// - `label_path_stopped` — a numeric checkpoint could not rebuild the
///   working copy from the last published snapshot, so the engine has
///   stopped applying labels altogether (´dec:surface:async-label´)
///
/// **Intentionally excluded** from this check:
///
/// - `feature_stable_outcome_drift` / `quantile_error_concentration`
///   / `cp5_reverts` / `cp6_reverts` / `auc_*` / Platt state — these
///   are *detection and observability* signals, which report and
///   never act (´inv:monitoring:report-only´), not hard
///   failures; short scenarios can legitimately set them and
///   scenarios that care assert on them explicitly.
/// - Convergence-stage / Platt-state fields — a freshly built
///   [`World`] legitimately sits in `ColdStart` / `Initial`.
/// - `batch_init_observations_skipped` — the cold ramp's load
///   condition, reported in band (´cor:degradation:in-band-travel´):
///   a concurrent scenario can legitimately fill the command channel,
///   and the honest trace of that refusal is not a hard failure.
///   `total_degraded` must still be accounted for by exactly that
///   trace once every other counter is zero.
#[track_caller]
pub fn assert_health_clean(world: &World) {
    let h = world.assayer().health_summary();
    let d = &h.degradation;

    assert!(
        d.total_degraded <= d.batch_init_observations_skipped,
        "health: {} degraded assessments exceed the {} refused cold-ramp observations",
        d.total_degraded,
        d.batch_init_observations_skipped,
    );
    assert_eq!(d.signals_sanitised, 0, "health: signals_sanitised = {}", d.signals_sanitised);
    assert_eq!(
        d.signals_shape_mismatched, 0,
        "health: signals_shape_mismatched = {}",
        d.signals_shape_mismatched,
    );
    assert_eq!(d.signals_unknown, 0, "health: signals_unknown = {}", d.signals_unknown);
    assert_eq!(
        d.sentinel_slots_zeroed, 0,
        "health: sentinel_slots_zeroed = {}",
        d.sentinel_slots_zeroed,
    );
    assert_eq!(
        d.features_sanitised, 0,
        "health: features_sanitised = {}",
        d.features_sanitised,
    );
    assert_eq!(d.model_fallbacks, 0, "health: model_fallbacks = {}", d.model_fallbacks);
    assert!(!h.any_cascade_terminus, "health: any_cascade_terminus set");
    assert!(!h.label_path_stopped, "health: label_path_stopped set");
    assert_eq!(
        h.health_events_dropped, 0,
        "health: health_events_dropped = {}",
        h.health_events_dropped,
    );
}

/// Assert that two assessments share the same core risk basis within
/// the integration-suite purity drift budget.
#[track_caller]
pub fn assert_core_risk_basis_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_risk_basis_near(&actual.assessment.risk, &expected.assessment.risk, PURITY_DRIFT, label);
}

/// Assert that two outcome-prediction maps match within the purity drift budget.
///
/// Outcome predictions are part of the core assessment payload
/// (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´). They
/// may be present while resonance tags are derived, but channel policy must not
/// perturb their keys or numerical payload for the same observation.
#[track_caller]
pub fn assert_outcome_prediction_maps_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    let actual_predictions = &actual.assessment.outcome_predictions;
    let expected_predictions = &expected.assessment.outcome_predictions;

    assert_eq!(
        actual_predictions.len(),
        expected_predictions.len(),
        "{label}: outcome-prediction count mismatch",
    );

    for axis in actual_predictions.keys() {
        assert!(
            expected_predictions.contains_key(axis),
            "{label}: expected reckoning is missing outcome prediction for {axis:?}",
        );
        assert_outcome_prediction_near(actual, expected, *axis, &format!("{label}: outcome prediction {axis:?}"));
    }
}

/// Assert that two outcome-prediction maps match within a caller-supplied drift budget.
#[track_caller]
pub fn assert_outcome_prediction_maps_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    drift: f64,
    label: &str,
) {
    let actual_predictions = &actual.assessment.outcome_predictions;
    let expected_predictions = &expected.assessment.outcome_predictions;

    assert_eq!(
        actual_predictions.len(),
        expected_predictions.len(),
        "{label}: outcome-prediction count mismatch",
    );

    for (axis, actual_prediction) in actual_predictions {
        let expected_prediction = expected_predictions
            .get(axis)
            .unwrap_or_else(|| panic!("{label}: expected reckoning is missing outcome prediction for {axis:?}"));

        assert_eq!(
            actual_prediction.axis_id, expected_prediction.axis_id,
            "{label}: outcome prediction {axis:?}: axis id mismatch",
        );
        assert_eq!(
            actual_prediction.axis_name, expected_prediction.axis_name,
            "{label}: outcome prediction {axis:?}: axis name mismatch",
        );
        assert_near(
            actual_prediction.predicted_raw,
            expected_prediction.predicted_raw,
            drift,
            &format!("{label}: outcome prediction {axis:?}: predicted_raw"),
        );
        assert_near(
            actual_prediction.uncertainty,
            expected_prediction.uncertainty,
            drift,
            &format!("{label}: outcome prediction {axis:?}: uncertainty"),
        );
        assert_near(
            actual_prediction.prediction_interval.0,
            expected_prediction.prediction_interval.0,
            drift,
            &format!("{label}: outcome prediction {axis:?}: prediction interval lower"),
        );
        assert_near(
            actual_prediction.prediction_interval.1,
            expected_prediction.prediction_interval.1,
            drift,
            &format!("{label}: outcome prediction {axis:?}: prediction interval upper"),
        );
    }
}

/// Assert that per-Sentinel alarm summaries match within the purity drift budget.
///
/// The alarm summaries are emitted from the core extraction/aggregation path.
/// They should therefore stay fixed when only the channel policy or companion
/// decision-layer state changes for the same observation.
#[track_caller]
pub fn assert_per_sentinel_alarms_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_per_sentinel_alarms_near_with_drift(actual, expected, PURITY_DRIFT, label);
}

/// Assert that per-Sentinel alarm summaries match within a caller-supplied drift budget.
#[track_caller]
pub fn assert_per_sentinel_alarms_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    drift: f64,
    label: &str,
) {
    let actual_alarms = &actual.assessment.per_sentinel;
    let expected_alarms = &expected.assessment.per_sentinel;

    assert_eq!(
        actual_alarms.len(),
        expected_alarms.len(),
        "{label}: per-Sentinel alarm count mismatch",
    );

    for (sentinel, actual_alarm) in actual_alarms {
        let expected_alarm = expected_alarms
            .get(sentinel)
            .unwrap_or_else(|| panic!("{label}: expected reckoning is missing alarm summary for {sentinel:?}"));

        assert_near(
            actual_alarm.peak_z,
            expected_alarm.peak_z,
            drift,
            &format!("{label}: {sentinel:?} peak_z"),
        );
        assert_near(
            actual_alarm.peak_cusum,
            expected_alarm.peak_cusum,
            drift,
            &format!("{label}: {sentinel:?} peak_cusum"),
        );
        assert_near(
            actual_alarm.peak_coord_z,
            expected_alarm.peak_coord_z,
            drift,
            &format!("{label}: {sentinel:?} peak_coord_z"),
        );
        assert_near(
            actual_alarm.composite,
            expected_alarm.composite,
            drift,
            &format!("{label}: {sentinel:?} composite"),
        );
        assert_eq!(
            actual_alarm.chain_depth, expected_alarm.chain_depth,
            "{label}: {sentinel:?} chain_depth mismatch",
        );
        assert_near(
            actual_alarm.maturity,
            expected_alarm.maturity,
            drift,
            &format!("{label}: {sentinel:?} maturity"),
        );
        assert_eq!(
            actual_alarm.hierarchical_asserted, expected_alarm.hierarchical_asserted,
            "{label}: {sentinel:?} hierarchical_asserted mismatch",
        );
        assert_eq!(
            actual_alarm.ledger_immature, expected_alarm.ledger_immature,
            "{label}: {sentinel:?} ledger_immature mismatch",
        );
    }
}

/// Assert that two per-reckoning health snapshots match.
///
/// Health is carried on the core assessment surface. Comparing it here catches
/// regressions where a derivation-layer input accidentally mutates per-request
/// diagnostics while leaving the numeric risk basis unchanged.
#[track_caller]
pub fn assert_health_snapshot_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_health_snapshot_near_with_drift(actual, expected, PURITY_DRIFT, label);
}

/// Assert that two per-reckoning health snapshots match within a caller-supplied drift budget.
#[track_caller]
pub fn assert_health_snapshot_near_with_drift(actual: &DerivedReckoning, expected: &DerivedReckoning, drift: f64, label: &str) {
    let actual_health = &actual.assessment.health;
    let expected_health = &expected.assessment.health;
    let actual_degradation = &actual_health.degradation;
    let expected_degradation = &expected_health.degradation;

    assert_eq!(
        actual_degradation.signals_sanitised, expected_degradation.signals_sanitised,
        "{label}: signals_sanitised mismatch",
    );
    assert_eq!(
        actual_degradation.signals_shape_mismatched, expected_degradation.signals_shape_mismatched,
        "{label}: signals_shape_mismatched mismatch",
    );
    assert_eq!(
        actual_degradation.nan_sentinels, expected_degradation.nan_sentinels,
        "{label}: nan_sentinels mismatch",
    );
    assert_eq!(
        actual_degradation.degraded_report_sentinels, expected_degradation.degraded_report_sentinels,
        "{label}: degraded_report_sentinels mismatch",
    );
    assert_eq!(
        actual_degradation.features_sanitised, expected_degradation.features_sanitised,
        "{label}: features_sanitised mismatch",
    );
    assert_eq!(
        actual_degradation.degraded_models, expected_degradation.degraded_models,
        "{label}: degraded_models mismatch",
    );
    assert_eq!(
        actual_health.convergence_stage, expected_health.convergence_stage,
        "{label}: convergence_stage mismatch",
    );
    assert_near(
        actual_health.sentinel_coverage,
        expected_health.sentinel_coverage,
        drift,
        &format!("{label}: sentinel_coverage"),
    );
    assert_eq!(
        actual_health.calibration_mature, expected_health.calibration_mature,
        "{label}: calibration_mature mismatch",
    );
    assert_eq!(
        actual_health.zero_sentinels, expected_health.zero_sentinels,
        "{label}: zero_sentinels mismatch",
    );
}

/// Assert the exact signal-degradation footprint for one reckoning.
///
/// This keeps expected CP1/shape-mismatch degradation out of
/// [`assert_health_clean`], while still proving no unrelated degradation path
/// fired during a scenario.
#[track_caller]
pub fn assert_signal_degradation_counts(
    reckoning: &DerivedReckoning,
    signals_sanitised: u32,
    signals_shape_mismatched: u32,
    label: &str,
) {
    let degradation = &reckoning.assessment.health.degradation;

    assert_eq!(
        degradation.signals_sanitised, signals_sanitised,
        "{label}: signals_sanitised mismatch",
    );
    assert_eq!(
        degradation.signals_shape_mismatched, signals_shape_mismatched,
        "{label}: signals_shape_mismatched mismatch",
    );
    assert!(
        degradation.nan_sentinels.is_empty(),
        "{label}: unexpected nan_sentinels {:?}",
        degradation.nan_sentinels,
    );
    assert!(
        degradation.degraded_report_sentinels.is_empty(),
        "{label}: unexpected degraded_report_sentinels {:?}",
        degradation.degraded_report_sentinels,
    );
    assert_eq!(
        degradation.features_sanitised, 0,
        "{label}: unexpected features_sanitised count",
    );
    assert!(
        degradation.degraded_models.is_empty(),
        "{label}: unexpected degraded_models {:?}",
        degradation.degraded_models,
    );
}

/// Assert that two assessments share the same public core assessment payload.
///
/// This compares every stable field in [`RiskAssessment`](crate::assessment::RiskAssessment):
/// the risk basis, outcome prediction map, per-Sentinel alarm summaries, and
/// inline health snapshot. The monotonic assessment ID is intentionally ignored.
#[track_caller]
pub fn assert_core_assessment_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_core_risk_basis_near(actual, expected, &format!("{label}: RiskBasis"));
    assert_outcome_prediction_maps_near(actual, expected, &format!("{label}: outcome predictions"));
    assert_per_sentinel_alarms_near(actual, expected, &format!("{label}: per-Sentinel alarms"));
    assert_health_snapshot_near(actual, expected, &format!("{label}: health"));
}

/// Assert that two assessments share the same public core assessment payload
/// within a caller-supplied drift budget.
#[track_caller]
pub fn assert_core_assessment_near_with_drift(actual: &DerivedReckoning, expected: &DerivedReckoning, drift: f64, label: &str) {
    assert_risk_basis_near(
        &actual.assessment.risk,
        &expected.assessment.risk,
        drift,
        &format!("{label}: RiskBasis"),
    );
    assert_outcome_prediction_maps_near_with_drift(actual, expected, drift, &format!("{label}: outcome predictions"));
    assert_per_sentinel_alarms_near_with_drift(actual, expected, drift, &format!("{label}: per-Sentinel alarms"));
    assert_health_snapshot_near_with_drift(actual, expected, drift, &format!("{label}: health"));
}

/// Assert that two assessments have the same resonance profile within
/// the integration-suite purity drift budget.
///
/// This deliberately compares only the derivation-layer payload
/// (`profile`), leaving the core [`RiskBasis`] and monotonic reckoning
/// ids to callers. Use [`assert_core_and_resonance_profile_near`] when
/// a scenario wants both halves of the public split.
#[track_caller]
pub fn assert_resonance_profile_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_resonance_profile_near_with_drift(actual, expected, PURITY_DRIFT, label);
}

/// Assert that two assessments have the same resonance profile within a caller-
/// supplied drift budget.
///
/// Most tests should use [`assert_resonance_profile_near`]. This variant exists
/// for cross-world public API probes whose sequential `now()` reads can widen
/// very slightly under parallel test load while still exercising the same
/// decision-layer invariant.
#[track_caller]
pub fn assert_resonance_profile_near_with_drift(actual: &DerivedReckoning, expected: &DerivedReckoning, drift: f64, label: &str) {
    assert_tag_profile_near(&actual.profile.tags, &expected.profile.tags, drift, &format!("{label}: tags"));
    // The gauge is a per-call read over the profile
    // (´sig:rendering:ambiguity-gauge´); comparing it at one posture pins
    // the whole rendered distribution the two profiles imply there.
    let actual_gauge = crate::profile_ambiguity(&actual.profile, 0.3);
    let expected_gauge = crate::profile_ambiguity(&expected.profile, 0.3);
    assert_near(
        actual_gauge.total,
        expected_gauge.total,
        drift,
        &format!("{label}: gauge.total"),
    );
    assert_near(
        actual_gauge.classification,
        expected_gauge.classification,
        drift,
        &format!("{label}: gauge.classification"),
    );
    assert_near(
        actual_gauge.action,
        expected_gauge.action,
        drift,
        &format!("{label}: gauge.action"),
    );
}

/// Assert that two assessments agree on both public halves that are allowed to
/// be stable across repeated derivations.
///
/// The two halves are the core assessment payload and the derivation-layer
/// [`ResonanceProfile`](crate::ResonanceProfile).
///
/// This intentionally ignores `assessment.id`, which is a monotonic
/// request identifier and must change across repeated public calls.
#[track_caller]
pub fn assert_core_and_resonance_profile_near(actual: &DerivedReckoning, expected: &DerivedReckoning, label: &str) {
    assert_core_assessment_near(actual, expected, &format!("{label}: core assessment"));
    assert_resonance_profile_near(actual, expected, &format!("{label}: resonance profile"));
}

/// Assert that the public core assessment and resonance profile match within a
/// caller-supplied drift budget.
#[track_caller]
pub fn assert_core_and_resonance_profile_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    drift: f64,
    label: &str,
) {
    assert_core_assessment_near_with_drift(actual, expected, drift, &format!("{label}: core assessment"));
    assert_resonance_profile_near_with_drift(actual, expected, drift, &format!("{label}: resonance profile"));
}

/// Assert that two assessments carry the same prediction for one outcome axis.
///
/// This is the outcome-prediction analogue of [`assert_core_risk_basis_near`]:
/// useful when a scenario wants to prove prediction outputs are present but do
/// not explain any movement in the resonance profile.
#[track_caller]
pub fn assert_outcome_prediction_near(actual: &DerivedReckoning, expected: &DerivedReckoning, axis: OutcomeAxisId, label: &str) {
    assert_outcome_prediction_near_with_drift(actual, expected, axis, PURITY_DRIFT, label);
}

/// Assert that two assessments carry the same prediction within a caller-supplied drift budget.
#[track_caller]
pub fn assert_outcome_prediction_near_with_drift(
    actual: &DerivedReckoning,
    expected: &DerivedReckoning,
    axis: OutcomeAxisId,
    drift: f64,
    label: &str,
) {
    let actual_prediction = actual
        .assessment
        .outcome_predictions
        .get(&axis)
        .unwrap_or_else(|| panic!("{label}: actual reckoning is missing prediction for {axis:?}"));
    let expected_prediction = expected
        .assessment
        .outcome_predictions
        .get(&axis)
        .unwrap_or_else(|| panic!("{label}: expected reckoning is missing prediction for {axis:?}"));

    assert_eq!(
        actual_prediction.axis_id, expected_prediction.axis_id,
        "{label}: axis id mismatch",
    );
    assert_eq!(
        actual_prediction.axis_name, expected_prediction.axis_name,
        "{label}: axis name mismatch",
    );
    assert_finite(
        &[
            actual_prediction.predicted_raw,
            actual_prediction.uncertainty,
            actual_prediction.prediction_interval.0,
            actual_prediction.prediction_interval.1,
            expected_prediction.predicted_raw,
            expected_prediction.uncertainty,
            expected_prediction.prediction_interval.0,
            expected_prediction.prediction_interval.1,
        ],
        label,
    );
    assert_near(
        actual_prediction.predicted_raw,
        expected_prediction.predicted_raw,
        drift,
        &format!("{label}: predicted_raw"),
    );
    assert_near(
        actual_prediction.uncertainty,
        expected_prediction.uncertainty,
        drift,
        &format!("{label}: uncertainty"),
    );
    assert_near(
        actual_prediction.prediction_interval.0,
        expected_prediction.prediction_interval.0,
        drift,
        &format!("{label}: prediction interval lower"),
    );
    assert_near(
        actual_prediction.prediction_interval.1,
        expected_prediction.prediction_interval.1,
        drift,
        &format!("{label}: prediction interval upper"),
    );
}
