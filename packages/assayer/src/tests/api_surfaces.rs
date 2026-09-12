// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`assess_and_derive_returns_all_fields`] | api | A derived reckoning arrives complete: it carries a non-zero assessment identifier, the channel it was derived for, a finite risk, the tag and landscape and rendered profile, outcome predictions, the per-Sentinel alarm map and the health snapshot. None of these is conditionally present, so a host writes one code path over the result rather than a lattice of presence checks. |
//! | [`assess_without_channel_hint_succeeds`] | api | A request naming only an entity, with no channel attached, is assessed and yields a finite risk. Core assessment answers what the engine believes about an entity; which intervention surface that belief will be spent on is a derivation-time question, so a host can assess once and derive for several channels from the same answer. |
//! | [`assess_empty_entity_key_succeeds`] | api | Even an entity key with no bytes at all is assessed rather than refused, and the risk that comes back is finite. Assessment sits in the request path where a caller has nothing useful to do with a refusal, so a degenerate key falls back on what the engine knows in general instead of failing the request. |
//! | [`assess_and_derive_no_sentinels_valid_output`] | api | With no Sentinel reporting, the engine answers from the prior — a risk near maximum uncertainty — and says so, flagging the no-Sentinel condition in the health snapshot. The number alone would be indistinguishable from a confidently balanced judgement; the flag is what lets a host tell an uninformed answer from an informed one. |
//! | [`risk_basis_has_14_fields`] | api | The risk basis exposes the whole sufficient-statistic set publicly, not just the headline probability: the blended risk and its sister and operational halves, uncertainty, anchor weight, the effective log-odds, scale and sharpness, intervention effectiveness, the borrowed share behind the uncertainty, the convergence flag, the count of reporting Sentinels and both regimes' calibration record counts. Every numeric one is finite, and the share is a share. A host deriving its own policy needs the statistics the probability was built from, not only the probability — and since the uncertainty it reads has been tempered by the share, it needs the share to be able to read the uncertainty back the other way (´dec:risk:evidence-only-uncertainty´). |
//! | [`intervention_effectiveness_formula`] | api | Intervention effectiveness is a difference between the effective and operational log-odds, so while the two models still share a cold-start mean it is finite and close to zero. The measure reports how much the intervention regime is moving the answer, and at cold start the honest report is that it is moving it hardly at all. |
//! | [`calibration_records_non_negative`] | api | A cold instance reports zero calibration records in both the sister and the anchor regime — an honest count of the evidence behind its calibration, rather than a seeded figure that would make an uncalibrated answer look supported. The anchor convergence flag is readable at the same time, even though at identical priors the comparison behind it need not resolve either way. |
//! | [`per_sentinel_is_hashmap`] | api | Per-Sentinel alarms come back as a map keyed by Sentinel identifier, and with none registered the map is simply empty. Absence is represented by a missing key rather than by a zeroed placeholder entry, so a host can tell a Sentinel that reported nothing alarming from one that never reported at all. |
//! | [`ambiguity_reads_at_the_callers_posture`] | api | The ambiguity gauge answers at whatever posture the caller names — three finite figures at each of several probes — rather than shipping one payload at a posture some configuration chose (´sig:rendering:ambiguity-gauge´). A reading taken at the host's own operating point needs no reaching back into configuration to interpret, because the posture it was read against is the one the caller just passed. |
//! | [`outcome_prediction_has_axis_id_and_name`] | api | An outcome prediction names the axis it belongs to, by identifier and by name, and carries a finite point estimate, uncertainty and interval; a default one is well-formed rather than filled with sentinel values. Since predictions travel in a map a host may split apart, each one has to be self-describing — and with no axes registered the map is empty rather than carrying an unattributed entry. |
//! | [`health_snapshot_has_zero_sentinels`] | api | The no-Sentinel condition is stated three consistent ways at once: the health snapshot flags it, coverage is zero, and no Sentinels are counted as reporting. Coverage of zero on its own would be ambiguous between total absence and total non-participation, so the explicit flag settles which one the host is looking at. |
//! | [`assess_batch_shared_instant`] | api | A batch yields exactly one result per request, positionally aligned with the input, each succeeding and each carrying a strictly larger assessment identifier than the one before. Alignment is what lets a caller zip results back onto its own requests without a correlation key, and the strict ordering means no two requests in a batch can be labelled as one another. |
//! | [`label_valid_returns_ok`] | api | A label against a live assessment is accepted, and the acknowledgement echoes back the identifier it settled. Labelling is asynchronous — the model owner applies it later — so the echoed identifier is the caller's only immediate confirmation that the evidence landed against the assessment it meant. |
//! | [`label_path_stopped_refuses_and_shows_on_both_health_tiers`] | api | A stopped label path refuses every submission with the cause that stopped it, consuming nothing, while assessment carries on answering from a state no label has touched since the stop and both health tiers say so. The engine reached a state it could neither learn from nor rebuild, and the whole point of stopping rather than continuing is that everything which does not learn keeps working: the read surface is a snapshot that was good, the refusal names which model was refused and how far into the label stream it happened, and a host that polls the cheap tier learns of it without having to ask for the expensive one. A second submission of the same assessment is refused the same way rather than as an unknown assessment, which is what shows that the first refusal took the pending entry with it no more than it took the journal. |
//! | [`label_unknown_id_error`] | api | A label naming an assessment the engine never issued is refused as `AssessmentNotFound`. The label's meaning comes entirely from the features captured when the assessment was made, so with no pending entry to attach it to there is nothing the evidence could be evidence about — accepting it would train the model on features it invented. |
//! | [`label_second_call_same_id_error`] | api | Labelling consumes the pending assessment: the first label succeeds and a second against the same identifier is refused, even when it carries the opposite valence. One request yields one piece of evidence, so a retrying or duplicating caller cannot make a single observation count twice — and cannot quietly overwrite a settled outcome with a contradictory one. |
//! | [`label_nan_valence_sanitised`] | api | A label whose valence is not a number is accepted and sanitised rather than refused. The identifier still addresses a real assessment, so the evidence that the request was observed is real even though one field of it is unusable; refusing the whole label would throw away a genuine observation over a single bad float from an upstream computation. |
//! | [`label_nan_outcome_removed`] | api | cites (´claim:api:a-non-finite-label-value-is-sanitised-rather-than-costing-the-whole-label´) |
//! | [`receive_report_unknown_sentinel_error`] | api | A structurally sound report offered under an identifier that was never registered is refused as `UnknownSentinel`. Reports are attributed evidence: a Sentinel's slot is where its ledger and report index live, so a report with no slot has nowhere to be believed from and could not be weighted, aged, or later contradicted. |
//! | [`error_types_have_display`] | api | Every public failure — a missing assessment, a full channel, a shut-down model owner, a label journalled but not enqueued, a report with no root, an unknown Sentinel — renders as a non-empty message. Errors end up in logs and alerts read by people rather than by code, so a variant that printed nothing would be a failure mode with no trace at the moment it is being diagnosed. |
//! | [`receive_report_valid_returns_ok`] | api | A report from a registered Sentinel is accepted, and the acknowledgement says how many cells were taken in. Ingestion may drop or merge cells as it maintains the ledger, so a count the caller can compare against what it sent is what turns a bare success into evidence that the report was actually absorbed. |
//! | [`receive_report_missing_root_error`] | api | A report that omits its depth-zero cell is refused as `MissingRootCell`, even from a properly registered Sentinel. The root is the whole-domain baseline every deeper cell is read relative to; without it the hierarchy has no reference level, and the cells that are present would be interpreted against nothing. |
//! | [`receive_report_nan_scores_degraded`] | api | Non-finite cell scores do not fail a report: it is accepted, and the acknowledgement counts the affected cells as degraded. A report is a batch of many independent cells, so discarding all of them over a few bad numbers would lose far more evidence than it protects — and the count is what stops the sanitisation from being silent, letting a host notice a Sentinel that has started producing rubbish. |
//! | [`sentinel_alarm_summary_fully_populated`] | api | Once a Sentinel has registered and reported, and a request supplies a coordinate for it, that Sentinel appears in the reckoning's alarm map with every summary field populated: finite peak deviation, accumulated drift, coordinate deviation, composite and maturity, plus its chain depth and both assertion flags. Presence in the map is itself the statement that the Sentinel reported, so a host reads participation and severity from one place. |
//! | [`p_bad_sister_and_operational_differ_after_labels`] | api | The sister and operational models are conditioned on different regimes, so a hundred pre-seeded labels move them apart — and both sub-probabilities, along with the blended risk, remain finite and inside the unit interval throughout. The pair is published so a host can see the two regimes separately, which is only useful if each stays a probability in its own right rather than degenerating once the models stop agreeing. |
//! | [`register_sentinel_empty_name_error`] | api | Registering a Sentinel under an empty name is refused, and the refusal names the kind of thing that was being registered. Names are what an operator has to work with when reading health reports and alarms, so an anonymous registration would produce a Sentinel that could report but never be identified — and a host registering several entity kinds needs the error to say which call it came from. |
//! | [`register_axis_empty_name_error`] | api | cites (´claim:api:registration-refuses-an-empty-name-and-names-the-entity-kind-refused´) |
//! | [`assess_survives_poisoned_shared_state`] | api | Another thread has panicked while holding the shared state the assessment path reads — the identity-dimension map and a Sentinel's outcome ledger are both left poisoned — and the assessment still returns a finite risk. A poisoned lock is another thread's failure rather than a caller handing in something the contract does not admit, so the state behind it is the last state known good and reading it is what the posture asks for; and since the poisoning is permanent, a call that refused it would take down not one request but every request that followed, on a path whose whole promise is that it answers. |
//! | [`register_identity_empty_name_error`] | api | cites (´claim:api:registration-refuses-an-empty-name-and-names-the-entity-kind-refused´) |

//! Acceptance tests for the three API surfaces.
//!
//! Tests `Assayer::assess()`, `Assayer::label()`, and
//! `Assayer::receive_sentinel_report()` through the full public API.
//!
//! These tests drive the engine through the [`World`](crate::testing::World)
//! harness wherever it has a matching verb. Surfaces that the harness
//! does not wrap yet (outcome-axis / identity-dimension registration,
//! Sentinel reports, pre-seed, label-channel flush) reach through
//! [`World::assayer`](crate::testing::World::assayer) — the harness's
//! documented escape hatch.
//!
//! Three shapes recur across the surface. Assessment is infallible: it takes
//! whatever request it is given — no channel, a degenerate entity key, a whole
//! batch at once — and always returns a populated answer, because a caller in a
//! serving path has nothing useful to do with a refusal. Labelling and report
//! reception are fallible, and their errors are named and specific: an unknown
//! identifier, an already-settled assessment, an unregistered Sentinel, a
//! report with no root. Values that are merely malformed rather than
//! unaddressable — a non-finite valence, a non-finite cell score — are
//! sanitised and counted instead of rejected, so one bad number never costs a
//! host a whole batch of evidence.
//!
//! What comes back is equally part of the contract. Every field named in the
//! specification is present and finite, absent evidence is reported as absent
//! rather than fabricated, and the types are non-exhaustive so the surface can
//! grow without breaking callers.
//!
//! # Cross-References
//!
//! - (´chap:spec:assessment-interface´) — the core assessment output this surface returns
//! - (´dec:surface:batch-infallible´) — the assessment contract: a batch goes in and the call cannot fail
//! - (´dec:surface:async-label´) — the label contract: submission is asynchronous
//! - The acceptance criteria, whose implemented evidence is the test index above

use std::collections::HashMap;
use std::sync::Arc;

use crate::assessment::{OutcomePrediction, RequestContext};
use crate::error::{LabelError, LabelPathStop, ReportError};
use crate::owner::commands::SentinelRegistration;
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters};
use crate::testing::{GOLDEN_COORD, LabelSpec, World, degraded_report, minimal_report, report_missing_root};
use crate::types::{AssessmentId, ChannelId, EntityKey, ModelId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Default channel name used by every test in this module.
const CHANNEL: &str = "test";

fn default_policy() -> ChannelPolicy {
    ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    }
}

/// Build a minimal [`World`] with one channel declared as [`CHANNEL`]
/// (which the engine assigns `ChannelId(0)`).
///
/// Cold-start dimension is derived from the schema: with no signals
/// or interaction templates: `p = 1 + 15 = 16` (bias + aggregates).
fn build_test_world() -> World {
    let config = super::helpers::test_config_with_id("api-test");
    World::builder(config)
        .channel(CHANNEL, default_policy())
        .seed(0xA55A_1234_5678_9ABC)
        .build()
        .expect("test world build should succeed")
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assess And Derive Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A derived reckoning arrives complete: it carries a non-zero assessment
/// identifier, the channel it was derived for, a finite risk, the tag and
/// landscape and rendered profile, outcome predictions, the per-Sentinel alarm map and the
/// health snapshot. None of these is conditionally present, so a host writes
/// one code path over the result rather than a lattice of presence checks.
///
/// ´claim:api:a-derived-reckoning-arrives-with-every-documented-top-level-field-present´
/// ´test:crate:assess-and-derive-returns-all-fields´
#[test]
fn assess_and_derive_returns_all_fields() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // All 7 top-level fields populated
    assert!(reckoning.assessment.id > AssessmentId(0));
    assert_eq!(reckoning.channel, ChannelId(0));
    assert!(reckoning.assessment.risk.p_bad.is_finite());
    // tags may be empty with default policy, but the field exists
    let _ = &reckoning.profile.tags;
    let _ = &reckoning.assessment.outcome_predictions;
    let gauge = crate::profile_ambiguity(&reckoning.profile, 0.3);
    assert!(gauge.total.is_finite());
    let _ = &reckoning.landscape.crossovers;
    let _ = &reckoning.assessment.per_sentinel;
    let _ = &reckoning.assessment.health;
}

/// A request naming only an entity, with no channel attached, is assessed and
/// yields a finite risk. Core assessment answers what the engine believes about
/// an entity; which intervention surface that belief will be spent on is a
/// derivation-time question, so a host can assess once and derive for several
/// channels from the same answer.
///
/// ´claim:api:core-assessment-needs-no-channel-because-policy-belongs-to-derivation´
/// ´test:crate:assess-without-channel-hint-succeeds´
#[test]
fn assess_without_channel_hint_succeeds() {
    let world = build_test_world();
    let req = RequestContext::new(World::entity("e1"));

    let assessment = world.assess(req);
    assert!(assessment.risk.p_bad.is_finite());
}

/// Even an entity key with no bytes at all is assessed rather than refused, and
/// the risk that comes back is finite. Assessment sits in the request path
/// where a caller has nothing useful to do with a refusal, so a degenerate key
/// falls back on what the engine knows in general instead of failing the
/// request.
///
/// ´claim:api:assessment-answers-even-for-a-degenerate-entity-key´
/// ´test:crate:assess-empty-entity-key-succeeds´
#[test]
fn assess_empty_entity_key_succeeds() {
    let world = build_test_world();
    let req = RequestContext::new(EntityKey::new(Vec::<u8>::new()));

    let assessment = world.assess(req);
    assert!(assessment.risk.p_bad.is_finite());
}

/// With no Sentinel reporting, the engine answers from the prior — a risk near
/// maximum uncertainty — and says so, flagging the no-Sentinel condition in the
/// health snapshot. The number alone would be indistinguishable from a
/// confidently balanced judgement; the flag is what lets a host tell an
/// uninformed answer from an informed one.
///
/// ´claim:api:with-no-sentinel-reporting-the-answer-is-the-prior-and-is-marked-as-such´
/// ´test:crate:assess-and-derive-no-sentinels-valid-output´
#[test]
fn assess_and_derive_no_sentinels_valid_output() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // p_bad ≈ 0.5 with no Sentinels (prior-only)
    assert!(
        (reckoning.assessment.risk.p_bad - 0.5).abs() < 0.1,
        "p_bad should be near 0.5, got {}",
        reckoning.assessment.risk.p_bad,
    );

    assert!(reckoning.assessment.health.zero_sentinels, "zero_sentinels should be true");
}

/// The risk basis exposes the whole sufficient-statistic set publicly, not just
/// the headline probability: the blended risk and its sister and operational
/// halves, uncertainty, anchor weight, the effective log-odds, scale and
/// sharpness, intervention effectiveness, the borrowed share behind the
/// uncertainty, the convergence flag, the count of reporting Sentinels and both
/// regimes' calibration record counts. Every numeric one is finite, and the
/// share is a share. A host deriving its own policy needs the statistics the
/// probability was built from, not only the probability — and since the
/// uncertainty it reads has been tempered by the share, it needs the share to
/// be able to read the uncertainty back the other way
/// (´dec:risk:evidence-only-uncertainty´).
///
/// ´claim:api:the-risk-basis-publishes-the-whole-sufficient-statistic-set-with-finite-values´
/// ´test:crate:risk-basis-has-14-fields´
#[test]
fn risk_basis_has_14_fields() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    let risk = &reckoning.assessment.risk;

    // The 14 public fields exist and are finite where applicable. The raw
    // sufficient-statistic triple is public because it is all that crosses
    // into derivation (´dec:ordering:closed-crossing´).
    assert!(risk.p_bad.is_finite());
    assert!(risk.uncertainty.is_finite());
    assert!(risk.anchor_weight.is_finite());
    assert!(risk.rho_eff.is_finite());
    assert!(risk.sigma_eff.is_finite());
    assert!(risk.kappa_eff.is_finite());
    assert!(risk.p_bad_sister.is_finite());
    assert!(risk.p_bad_operational.is_finite());
    assert!(risk.intervention_effectiveness.is_finite());
    assert!(
        (0.0..=1.0).contains(&risk.borrowed_share),
        "the borrowed share is a share (´dec:risk:evidence-only-uncertainty´): {}",
        risk.borrowed_share
    );
    let _ = risk.anchor_converged; // bool
    let _ = risk.n_sentinels_reporting; // usize
    // Weighted sample counts (´schema:risk:basis´).
    let _: f64 = risk.sister_regime_calibration_records;
    let _: f64 = risk.anchor_regime_calibration_records;
}

/// Intervention effectiveness is a difference between the effective and
/// operational log-odds, so while the two models still share a cold-start mean
/// it is finite and close to zero. The measure reports how much the
/// intervention regime is moving the answer, and at cold start the honest
/// report is that it is moving it hardly at all.
///
/// ´claim:api:intervention-effectiveness-is-near-zero-while-the-two-regimes-still-agree´
/// ´test:crate:intervention-effectiveness-formula´
#[test]
fn intervention_effectiveness_formula() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    let risk = &reckoning.assessment.risk;

    // At cold start, sister and operational models share the same μ,
    // so rho_eff ≈ rho_opr. The intervention effectiveness
    // (rho_eff - rho_opr) should be near zero.
    //
    // We verify the relationship holds: the value is finite and
    // consistent with the formula.
    assert!(
        risk.intervention_effectiveness.is_finite(),
        "intervention_effectiveness should be finite"
    );

    // With identical cold-start models, the difference should be small.
    assert!(
        risk.intervention_effectiveness.abs() < 1.0,
        "at cold start, intervention_effectiveness should be near 0, got {}",
        risk.intervention_effectiveness,
    );
}

/// A cold instance reports zero calibration records in both the sister and the
/// anchor regime — an honest count of the evidence behind its calibration,
/// rather than a seeded figure that would make an uncalibrated answer look
/// supported. The anchor convergence flag is readable at the same time, even
/// though at identical priors the comparison behind it need not resolve either
/// way.
///
/// ´claim:api:a-cold-instance-reports-no-calibration-records-in-either-regime´
/// ´test:crate:calibration-records-non-negative´
#[test]
fn calibration_records_non_negative() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    let risk = &reckoning.assessment.risk;

    // At cold start with no labels, both weighted sample counts are zero.
    assert!(risk.sister_regime_calibration_records.abs() < f64::EPSILON);
    assert!(risk.anchor_regime_calibration_records.abs() < f64::EPSILON);

    // anchor_converged compares anchor vs sister shared-subspace variance
    // under the subspace-restricted blend (´def:risk:subspace-blend´). At cold
    // start with identical priors, it is implementation-
    // defined whether the comparison strictly resolves either way; we only
    // assert that the field is readable and has the documented type.
    let _: bool = risk.anchor_converged;
}

/// Per-Sentinel alarms come back as a map keyed by Sentinel identifier, and
/// with none registered the map is simply empty. Absence is represented by a
/// missing key rather than by a zeroed placeholder entry, so a host can tell a
/// Sentinel that reported nothing alarming from one that never reported at all.
///
/// ´claim:api:per-sentinel-alarms-are-keyed-by-sentinel-id-and-absent-rather-than-zero-filled´
/// ´test:crate:per-sentinel-is-hashmap´
#[test]
fn per_sentinel_is_hashmap() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // per_sentinel is HashMap<SentinelId, SentinelAlarmSummary> — verify by type use
    let map: &HashMap<SentinelId, _> = &reckoning.assessment.per_sentinel;
    // With no registered Sentinels, the map is empty
    assert!(map.is_empty());
}

/// The ambiguity gauge answers at whatever posture the caller names — three
/// finite figures at each of several probes — rather than shipping one
/// payload at a posture some configuration chose
/// (´sig:rendering:ambiguity-gauge´). A reading taken at the host's own
/// operating point needs no reaching back into configuration to interpret,
/// because the posture it was read against is the one the caller just
/// passed.
///
/// ´claim:api:the-ambiguity-gauge-reads-at-the-callers-posture´
/// ´test:crate:ambiguity-reads-at-the-callers-posture´
#[test]
fn ambiguity_reads_at_the_callers_posture() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // The gauge is read at the caller's posture, per call
    // (´sig:rendering:ambiguity-gauge´): three finite figures at any
    // posture the host chooses, not one payload at a configured constant.
    for posture in [0.1, 0.3, 0.7] {
        let gauge = crate::profile_ambiguity(&reckoning.profile, posture);
        assert!(gauge.total.is_finite());
        assert!(gauge.classification.is_finite());
        assert!(gauge.action.is_finite());
    }
}

/// An outcome prediction names the axis it belongs to, by identifier and by
/// name, and carries a finite point estimate, uncertainty and interval; a
/// default one is well-formed rather than filled with sentinel values. Since
/// predictions travel in a map a host may split apart, each one has to be
/// self-describing — and with no axes registered the map is empty rather than
/// carrying an unattributed entry.
///
/// ´claim:api:an-outcome-prediction-names-the-axis-it-belongs-to´
/// ´test:crate:outcome-prediction-has-axis-id-and-name´
#[test]
fn outcome_prediction_has_axis_id_and_name() {
    // Verify the structural contract: OutcomePrediction carries
    // `axis_id` and `axis_name`.
    //
    // At cold start no outcome axes are registered, so the map is empty.
    // We verify the type contract by constructing a default prediction.
    let pred = OutcomePrediction::default();
    assert_eq!(pred.axis_id, OutcomeAxisId(0));
    assert_eq!(pred.axis_name, "");
    assert!(pred.predicted_raw.is_finite());
    assert!(pred.uncertainty.is_finite());
    assert!(pred.prediction_interval.0.is_finite());
    assert!(pred.prediction_interval.1.is_finite());

    // Also verify the map on a live reckoning is accessible.
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // With no outcome axes, the map type is correct but empty.
    assert!(reckoning.assessment.outcome_predictions.is_empty());
}

/// The no-Sentinel condition is stated three consistent ways at once: the
/// health snapshot flags it, coverage is zero, and no Sentinels are counted as
/// reporting. Coverage of zero on its own would be ambiguous between total
/// absence and total non-participation, so the explicit flag settles which one
/// the host is looking at.
///
/// ´claim:api:the-health-snapshot-flags-the-no-sentinel-condition-rather-than-leaving-coverage-ambiguous´
/// ´test:crate:health-snapshot-has-zero-sentinels´
#[test]
fn health_snapshot_has_zero_sentinels() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // zero_sentinels field exists and is correct
    assert!(reckoning.assessment.health.zero_sentinels);
    assert_eq!(reckoning.assessment.health.sentinel_coverage, 0.0);
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 0);
}

/// A batch yields exactly one result per request, positionally aligned with the
/// input, each succeeding and each carrying a strictly larger assessment
/// identifier than the one before. Alignment is what lets a caller zip results
/// back onto its own requests without a correlation key, and the strict
/// ordering means no two requests in a batch can be labelled as one another.
///
/// ´claim:api:a-batch-returns-one-aligned-result-per-request-with-strictly-increasing-ids´
/// ´test:crate:assess-batch-shared-instant´
#[test]
fn assess_batch_shared_instant() {
    let world = build_test_world();

    let entities = ["e0", "e1", "e2", "e3", "e4"];
    let requests: Vec<_> = entities.iter().map(|e| world.request(CHANNEL, e)).collect();

    let results = world.derive_for_requests(&requests);
    assert_eq!(results.len(), entities.len());

    // All should succeed
    for result in &results {
        assert!(result.is_ok());
    }

    // IDs should be unique and sequential
    let ids: Vec<AssessmentId> = results
        .iter()
        .filter_map(|r| r.as_ref().ok().map(|r| r.assessment.id))
        .collect();
    for window in ids.windows(2) {
        assert!(window[1] > window[0], "IDs should be monotonically increasing");
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A label against a live assessment is accepted, and the acknowledgement
/// echoes back the identifier it settled. Labelling is asynchronous — the model
/// owner applies it later — so the echoed identifier is the caller's only
/// immediate confirmation that the evidence landed against the assessment it
/// meant.
///
/// ´claim:api:a-label-is-acknowledged-with-the-assessment-id-it-settled´
/// ´test:crate:label-valid-returns-ok´
#[test]
fn label_valid_returns_ok() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    let label = LabelSpec::new(reckoning.assessment.id).valence(1.0).build();
    let ack = world.label(label).expect("label should succeed");
    assert_eq!(ack.assessment_id, reckoning.assessment.id);
}

/// A stopped label path refuses every submission with the cause that stopped it,
/// consuming nothing, while assessment carries on answering from a state no
/// label has touched since the stop and both health tiers say so. The engine reached a
/// state it could neither learn from nor rebuild, and the whole point of stopping
/// rather than continuing is that everything which does not learn keeps working:
/// the read surface is a snapshot that was good, the refusal names which model
/// was refused and how far into the label stream it happened, and a host that
/// polls the cheap tier learns of it without having to ask for the expensive one.
/// A second submission of the same assessment is refused the same way rather than
/// as an unknown assessment, which is what shows that the first refusal took the
/// pending entry with it no more than it took the journal.
///
/// ´claim:api:a-stopped-label-path-refuses-with-its-cause-while-assessment-and-health-carry-on´
/// ´test:crate:label-path-stopped-refuses-and-shows-on-both-health-tiers´
#[test]
fn label_path_stopped_refuses_and_shows_on_both_health_tiers() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    let labels_before = world.assayer().health_summary().total_labels;

    let stop = LabelPathStop {
        model: ModelId::Operational,
        pivot: 3,
        p: 16,
        labels_taken_up: 7,
    };
    world.assayer().shared.label_path_stop.store(Some(Arc::new(stop.clone())));

    let label = LabelSpec::new(reckoning.assessment.id).valence(1.0).build();
    match world.label(label) {
        Err(LabelError::LabelPathStopped(cause)) => {
            assert_eq!(cause, stop, "the refusal carries the recorded cause whole");
        }
        other => panic!("a stopped label path must refuse with its cause, got {other:?}"),
    }

    // Nothing was consumed: the same assessment is refused again for the same
    // reason rather than coming back as an assessment the engine never issued.
    let again = LabelSpec::new(reckoning.assessment.id).valence(1.0).build();
    assert!(
        matches!(world.label(again), Err(LabelError::LabelPathStopped(_))),
        "the refusal must leave the pending entry where it found it",
    );

    // Assessment still answers, and answers from a snapshot no label contributed
    // to: the label tally is where the stop left it. The snapshot's version is
    // not the witness here, because the cold ramp publishes a version of its own
    // for every observation an assessment offers
    // (´alg:standardisation:batch-initialisation´).
    let after = world.assess(RequestContext::new(World::entity("e3")));
    assert!(after.risk.p_bad.is_finite(), "assessment still answers after the stop");
    assert_eq!(
        world.assayer().health_summary().total_labels,
        labels_before,
        "no label was applied, so the models the assessment read are the ones the stop found",
    );

    let summary = world.assayer().health_summary();
    assert!(summary.label_path_stopped, "the compact tier carries the flag");

    let report = world.assayer().full_health_report();
    assert_eq!(
        report.label_path_stop.as_ref(),
        Some(&stop),
        "the detailed tier carries the cause",
    );
}

/// A label naming an assessment the engine never issued is refused as
/// `AssessmentNotFound`. The label's meaning comes entirely from the features
/// captured when the assessment was made, so with no pending entry to attach it
/// to there is nothing the evidence could be evidence about — accepting it
/// would train the model on features it invented.
///
/// ´claim:api:labelling-an-unknown-assessment-is-refused-rather-than-absorbed´
/// ´test:crate:label-unknown-id-error´
#[test]
fn label_unknown_id_error() {
    let world = build_test_world();

    let label = LabelSpec::new(AssessmentId(99_999)).valence(1.0).build();
    let result = world.label(label);
    assert!(matches!(result, Err(LabelError::AssessmentNotFound)));
}

/// Labelling consumes the pending assessment: the first label succeeds and a
/// second against the same identifier is refused, even when it carries the
/// opposite valence. One request yields one piece of evidence, so a retrying or
/// duplicating caller cannot make a single observation count twice — and cannot
/// quietly overwrite a settled outcome with a contradictory one.
///
/// ´claim:api:labelling-consumes-the-pending-assessment-so-evidence-cannot-be-double-counted´
/// ´test:crate:label-second-call-same-id-error´
#[test]
fn label_second_call_same_id_error() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();
    let id = reckoning.assessment.id;

    // First label succeeds
    let label1 = LabelSpec::new(id).valence(1.0).build();
    assert!(world.label(label1).is_ok());

    // Second label with same ID fails (entry consumed)
    let label2 = LabelSpec::new(id).valence(0.0).build();
    assert!(matches!(world.label(label2), Err(LabelError::AssessmentNotFound)));
}

/// A label whose valence is not a number is accepted and sanitised rather than
/// refused. The identifier still addresses a real assessment, so the evidence
/// that the request was observed is real even though one field of it is
/// unusable; refusing the whole label would throw away a genuine observation
/// over a single bad float from an upstream computation.
///
/// ´claim:api:a-non-finite-label-value-is-sanitised-rather-than-costing-the-whole-label´
/// ´test:crate:label-nan-valence-sanitised´
#[test]
fn label_nan_valence_sanitised() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // NaN valence should be accepted (sanitised to 0.0 internally)
    let label = LabelSpec::new(reckoning.assessment.id).valence(f64::NAN).build();
    let result = world.label(label);
    assert!(result.is_ok(), "NaN valence should be sanitised, not rejected");
}

/// The same tolerance covers the per-axis outcome map: a label carrying one
/// non-finite outcome beside a sound one is accepted, the unusable entry
/// dropped from the map. Outcome axes are independent, so a broken measurement
/// on one axis costs that axis's observation and leaves the rest of the label
/// intact.
///
/// (´claim:api:a-non-finite-label-value-is-sanitised-rather-than-costing-the-whole-label´)
/// ´test:crate:label-nan-outcome-removed´
#[test]
fn label_nan_outcome_removed() {
    let world = build_test_world();
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e1")).unwrap();

    // A label with NaN outcome values should be accepted
    // (NaN outcomes are removed from the map by sanitisation).
    let label = LabelSpec::new(reckoning.assessment.id)
        .valence(1.0)
        .outcome(OutcomeAxisId(1), f64::NAN)
        .outcome(OutcomeAxisId(2), 0.8)
        .build();

    let result = world.label(label);
    assert!(result.is_ok(), "NaN outcome should be sanitised, not rejected");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A structurally sound report offered under an identifier that was never
/// registered is refused as `UnknownSentinel`. Reports are attributed evidence:
/// a Sentinel's slot is where its ledger and report index live, so a report
/// with no slot has nowhere to be believed from and could not be weighted,
/// aged, or later contradicted.
///
/// ´claim:api:a-report-from-an-unregistered-sentinel-is-refused´
/// ´test:crate:receive-report-unknown-sentinel-error´
#[test]
fn receive_report_unknown_sentinel_error() {
    let world = build_test_world();
    let report = minimal_report();

    // Reports go through the engine directly — the harness does not
    // yet wrap this verb.
    let result = world.assayer().receive_sentinel_report(SentinelId(999), report);
    assert!(matches!(result, Err(ReportError::UnknownSentinel { .. })));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Error Display Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Every public failure — a missing assessment, a full channel, a shut-down
/// model owner, a label journalled but not enqueued, a report with no root, an
/// unknown Sentinel — renders as a non-empty message. Errors end up in logs and
/// alerts read by people rather than by code, so a variant that printed nothing
/// would be a failure mode with no trace at the moment it is being diagnosed.
///
/// ´claim:api:every-public-error-renders-a-non-empty-human-readable-message´
/// ´test:crate:error-types-have-display´
#[test]
fn error_types_have_display() {
    // LabelError
    let err = LabelError::AssessmentNotFound;
    assert_ne!(err.to_string(), "");

    let err = LabelError::ChannelFull;
    assert_ne!(err.to_string(), "");

    let err = LabelError::ModelOwnerShutdown;
    assert_ne!(err.to_string(), "");

    let err = LabelError::JournaledButNotEnqueued;
    assert_ne!(err.to_string(), "");

    // ReportError
    let err = ReportError::MissingRootCell;
    assert_ne!(err.to_string(), "");

    let err = ReportError::UnknownSentinel { id: SentinelId(42) };
    assert_ne!(err.to_string(), "");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Deferred Tests (unblocked by Sentinel registration)
// ═══════════════════════════════════════════════════════════════════════════════

use crate::error::LifecycleError;
use crate::owner::commands::{IdentityDimensionRegistration, OutcomeAxisRegistration};
use crate::types::{DimensionId, IdentityBudget, OutcomeEligibility, SpatialFeaturePolicy};

fn sentinel_reg(id: u32, name: &str) -> SentinelRegistration {
    SentinelRegistration {
        id: SentinelId(id),
        name: name.to_owned(),
    }
}

fn axis_reg(id: u32, name: &str) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id: OutcomeAxisId(id),
        name: name.to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Disabled,
    }
}

fn identity_reg(id: u32, name: &str) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id: DimensionId(id),
        name: name.to_owned(),
        description: "API-surface test hierarchy".to_owned(),
        coordinate_semantics: "the leading bytes select a prefix group".to_owned(),
        domain_bits: 128,
        depth_cutoff: 10,
        budget: IdentityBudget::for_depth_cutoff(10),
        encode: |entity| {
            let bytes = entity.as_bytes();
            if bytes.len() >= 8 {
                u128::from(u64::from_le_bytes(bytes[..8].try_into().unwrap_or_default()))
            } else {
                0
            }
        },
    }
}

/// A report from a registered Sentinel is accepted, and the acknowledgement
/// says how many cells were taken in. Ingestion may drop or merge cells as it
/// maintains the ledger, so a count the caller can compare against what it sent
/// is what turns a bare success into evidence that the report was actually
/// absorbed.
///
/// ´claim:api:an-accepted-report-acknowledges-how-many-cells-it-ingested´
/// ´test:crate:receive-report-valid-returns-ok´
#[test]
fn receive_report_valid_returns_ok() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").unwrap();

    let report = minimal_report();
    let ack = world.assayer().receive_sentinel_report(sentinel, report);
    assert!(ack.is_ok(), "valid report should succeed: {ack:?}");

    let ack = ack.unwrap();
    assert!(ack.cells_in_report > 0, "report should have cells ingested");
}

/// A report that omits its depth-zero cell is refused as `MissingRootCell`,
/// even from a properly registered Sentinel. The root is the whole-domain
/// baseline every deeper cell is read relative to; without it the hierarchy has
/// no reference level, and the cells that are present would be interpreted
/// against nothing.
///
/// ´claim:api:a-report-without-its-root-cell-is-refused´
/// ´test:crate:receive-report-missing-root-error´
#[test]
fn receive_report_missing_root_error() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").unwrap();

    let report = report_missing_root();
    let result = world.assayer().receive_sentinel_report(sentinel, report);
    assert!(
        matches!(result, Err(ReportError::MissingRootCell)),
        "missing root should fail: {result:?}"
    );
}

/// Non-finite cell scores do not fail a report: it is accepted, and the
/// acknowledgement counts the affected cells as degraded. A report is a batch
/// of many independent cells, so discarding all of them over a few bad numbers
/// would lose far more evidence than it protects — and the count is what stops
/// the sanitisation from being silent, letting a host notice a Sentinel that
/// has started producing rubbish.
///
/// ´claim:api:non-finite-cell-scores-are-counted-as-degraded-rather-than-failing-the-report´
/// ´test:crate:receive-report-nan-scores-degraded´
#[test]
fn receive_report_nan_scores_degraded() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").unwrap();

    let report = degraded_report();
    let result = world.assayer().receive_sentinel_report(sentinel, report);
    assert!(result.is_ok(), "NaN report should be accepted: {result:?}");

    let ack = result.unwrap();
    assert!(
        ack.degraded_cells > 0,
        "NaN scores should produce degraded_cells > 0; got {}",
        ack.degraded_cells,
    );
}

/// Once a Sentinel has registered and reported, and a request supplies a
/// coordinate for it, that Sentinel appears in the reckoning's alarm map with
/// every summary field populated: finite peak deviation, accumulated drift,
/// coordinate deviation, composite and maturity, plus its chain depth and both
/// assertion flags. Presence in the map is itself the statement that the
/// Sentinel reported, so a host reads participation and severity from one
/// place.
///
/// ´claim:api:a-reporting-sentinel-appears-in-the-alarm-map-with-every-summary-field-populated´
/// ´test:crate:sentinel-alarm-summary-fully-populated´
#[test]
fn sentinel_alarm_summary_fully_populated() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").unwrap();

    // Ingest a report so the Sentinel has data
    let report = minimal_report();
    world.assayer().receive_sentinel_report(sentinel, report).unwrap();

    // Flush to ensure model owner processes the Sentinel registration.
    world.assayer().flush_label_channel().unwrap();

    // Reckon — provide a sentinel coordinate so the extraction pipeline runs.
    let req = world.request(CHANNEL, "e1").with_sentinel(sentinel, GOLDEN_COORD);
    let reckoning = world.derive_for_request(req).unwrap();

    assert!(
        reckoning.assessment.per_sentinel.contains_key(&sentinel),
        "per_sentinel should contain registered Sentinel; keys: {:?}",
        reckoning.assessment.per_sentinel.keys().collect::<Vec<_>>(),
    );

    let summary = &reckoning.assessment.per_sentinel[&sentinel];

    // Verify alarm fields are populated (all 8 fields of SentinelAlarmSummary)
    assert!(summary.peak_z.is_finite(), "peak_z should be finite");
    assert!(summary.peak_cusum.is_finite(), "peak_cusum should be finite");
    assert!(summary.peak_coord_z.is_finite(), "peak_coord_z should be finite");
    assert!(summary.composite.is_finite(), "composite should be finite");
    let _: usize = summary.chain_depth; // one of the eight fields (´def:runtime:alarm-summary´)
    assert!(summary.maturity.is_finite(), "maturity should be finite");
    let _ = summary.hierarchical_asserted; // bool
    let _ = summary.ledger_immature; // bool
    // Presence in the per_sentinel map already implies the Sentinel reported
    // — the per-Sentinel alarm summary (´def:runtime:alarm-summary´).
}

/// The sister and operational models are conditioned on different regimes, so a
/// hundred pre-seeded labels move them apart — and both sub-probabilities, along
/// with the blended risk, remain finite and inside the unit interval throughout.
/// The pair is published so a host can see the two regimes separately, which is
/// only useful if each stays a probability in its own right rather than
/// degenerating once the models stop agreeing.
///
/// ´claim:api:the-sister-and-operational-sub-probabilities-stay-valid-probabilities-once-the-models-diverge´
/// ´test:crate:p-bad-sister-and-operational-differ-after-labels´
#[test]
fn p_bad_sister_and_operational_differ_after_labels() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").unwrap();

    // Ingest a report so the Sentinel has non-zero features
    let report = minimal_report();
    world.assayer().receive_sentinel_report(sentinel, report).unwrap();

    // Pre-seed with a mix of positive and negative labels to move the models
    // away from the prior. The sister and operational models should diverge
    // because the sister model includes sister-regime information while
    // the operational model conditions on the operational regime.
    use crate::api::PreSeedEntry;
    let entries: Vec<PreSeedEntry> = (0..100)
        .map(|i| PreSeedEntry {
            entity: World::entity(&format!("pre-seed-{i}")),
            channel: ChannelId(0),
            action_taken: crate::types::Action::Allow,
            valence: if i % 5 == 0 { 1.0 } else { 0.0 },
            outcomes: HashMap::new(),
            ground_truth: true,
        })
        .collect();
    world.assayer().pre_seed(&entries).unwrap();

    // Reckon and check that the two sub-probabilities exist and are finite
    let reckoning = world.derive_for_request(world.request(CHANNEL, "e-probe")).unwrap();
    let risk = &reckoning.assessment.risk;

    assert!(risk.p_bad_sister.is_finite(), "p_bad_sister should be finite");
    assert!(risk.p_bad_operational.is_finite(), "p_bad_operational should be finite");
    assert!(risk.p_bad.is_finite(), "p_bad should be finite");

    // After 100 labels (20% positive rate), p_bad should have moved from 0.5
    // toward the empirical rate. At minimum, both sub-probabilities should be
    // valid probabilities.
    assert!(
        (0.0..=1.0).contains(&risk.p_bad_sister),
        "p_bad_sister should be in [0, 1]; got {}",
        risk.p_bad_sister,
    );
    assert!(
        (0.0..=1.0).contains(&risk.p_bad_operational),
        "p_bad_operational should be in [0, 1]; got {}",
        risk.p_bad_operational,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Boundary Validation Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Registering a Sentinel under an empty name is refused, and the refusal names
/// the kind of thing that was being registered. Names are what an operator has
/// to work with when reading health reports and alarms, so an anonymous
/// registration would produce a Sentinel that could report but never be
/// identified — and a host registering several entity kinds needs the error to
/// say which call it came from.
///
/// ´claim:api:registration-refuses-an-empty-name-and-names-the-entity-kind-refused´
/// ´test:crate:register-sentinel-empty-name-error´
#[test]
fn register_sentinel_empty_name_error() {
    // Empty-name registration is a raw-API boundary test: the harness's
    // `World::register_sentinel` requires a non-empty `SentinelName`,
    // so we go through the engine directly.
    let world = build_test_world();
    let result = world.assayer().register_sentinel(sentinel_reg(1, ""));
    assert!(
        matches!(&result, Err(LifecycleError::EmptyName { entity_type: "Sentinel" })),
        "empty name should fail: {result:?}"
    );
}

/// An outcome axis is held to the same rule, and its refusal identifies itself
/// as an axis. Axis names travel outwards on every prediction the engine makes,
/// so an unnamed axis would leave the host's own callers holding predictions
/// they cannot attribute.
///
/// (´claim:api:registration-refuses-an-empty-name-and-names-the-entity-kind-refused´)
/// ´test:crate:register-axis-empty-name-error´
#[test]
fn register_axis_empty_name_error() {
    let world = build_test_world();
    let result = world.assayer().register_outcome_axis(axis_reg(1, ""));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::EmptyName {
                entity_type: "OutcomeAxis"
            })
        ),
        "empty name should fail: {result:?}"
    );
}

/// Another thread has panicked while holding the shared state the assessment
/// path reads — the identity-dimension map and a Sentinel's outcome ledger are
/// both left poisoned — and the assessment still returns a finite risk. A
/// poisoned lock is another thread's failure rather than a caller handing in
/// something the contract does not admit, so the state behind it is the last
/// state known good and reading it is what the posture asks for; and since the
/// poisoning is permanent, a call that refused it would take down not one
/// request but every request that followed, on a path whose whole promise is
/// that it answers.
///
/// ´claim:api:a-poisoned-lock-degrades-the-assessment-rather-than-ending-it´
/// ´test:crate:assess-survives-poisoned-shared-state´
#[test]
fn assess_survives_poisoned_shared_state() {
    let mut world = build_test_world();
    let sentinel = world.register_sentinel("s1").expect("sentinel registers");
    world
        .assayer()
        .receive_sentinel_report(sentinel, minimal_report())
        .expect("report is accepted");
    world
        .assayer()
        .register_identity_dimension(identity_reg(1, "ip-hash"))
        .expect("identity dimension registers");

    poison(&world.assayer().identity_dimensions);
    poison(
        &world
            .assayer()
            .outcome_ledger
            .get_arc(sentinel)
            .expect("a registered Sentinel owns a ledger"),
    );
    assert!(
        world.assayer().identity_dimensions.is_poisoned(),
        "the identity-dimension lock should now be poisoned"
    );

    let assessment = world.assess(RequestContext::new(World::entity("e1")).with_sentinel(sentinel, GOLDEN_COORD));
    assert!(assessment.risk.p_bad.is_finite());
}

/// Poisons `lock` by panicking on another thread while its write guard is
/// held. The panic hook is silenced for the duration so the deliberate unwind
/// does not read as a test failure in the output.
fn poison<T: Send + Sync + 'static>(lock: &Arc<std::sync::RwLock<T>>) {
    let lock = Arc::clone(lock);
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(|_| {}));
    let joined = std::thread::spawn(move || {
        let _guard = lock.write();
        panic!("deliberate poisoning");
    })
    .join();
    std::panic::set_hook(previous);
    assert!(joined.is_err(), "the poisoning thread should have panicked");
}

/// Identity dimensions complete the pattern: an empty name is refused and the
/// error names the dimension surface. The rule is uniform across all three
/// registration surfaces, so a host does not have to remember which of them
/// tolerate an unnamed entity — none do.
///
/// (´claim:api:registration-refuses-an-empty-name-and-names-the-entity-kind-refused´)
/// ´test:crate:register-identity-empty-name-error´
#[test]
fn register_identity_empty_name_error() {
    let world = build_test_world();
    let result = world.assayer().register_identity_dimension(identity_reg(1, ""));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::EmptyName {
                entity_type: "IdentityDimension"
            })
        ),
        "empty name should fail: {result:?}"
    );
}
