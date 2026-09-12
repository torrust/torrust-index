// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`assess_and_derive_with_no_sentinels_returns_valid_output`] | scenario | Sentinels are an enrichment, not a prerequisite: with none registered at all, an unseen entity still assesses and derives to a well-formed reckoning and the health snapshot stays clean. A host can therefore stand the engine up and get usable answers before it has wired any sentinel in, rather than having to complete the whole installation before the first request. |
//! | [`duplicate_label_rejected_without_double_update`] | scenario | An assessment can be labelled exactly once. The first label consumes its entry in the pending buffer, and a second label naming the same assessment is refused as not-found rather than applied again. Retries and duplicate deliveries on the host's side therefore cannot teach the models the same outcome twice and quietly inflate its weight. |
//! | [`label_with_unknown_assessment_id_is_rejected`] | scenario | An assessment id the engine never issued is refused with the same not-found error an evicted one gets, and the refusal is inert: the very next assess-derive-label cycle on a different entity succeeds and is well-formed. A rejected label is a rejected message, not damage — a host replaying a stale queue cannot brick the engine by getting an id wrong. |
//! | [`inverted_valence_signs_accepted_without_corruption`] | scenario | Valence is the host's convention and the engine does not second-guess it: labels carrying either sign on the same entity are accepted without any consistency check against prior belief, and the engine stays numerically sound and clean afterwards. A host that inverts the convention gets a wrong model rather than a broken one — the fault stays diagnosable as the host's, which it could not be if the engine silently corrected or rejected signs. |
//! | [`repeated_identical_benign_cycle_stays_finite`] | scenario | Two hundred rounds of assessing one unchanging entity and labelling it benign leave every reckoning well-formed and the health snapshot clean. The negative-valence update path is bounded, so an entity a host sees over and over cannot drive the posterior to collapse against its own boundary or out of the unit interval no matter how one-sided the evidence gets. |
//! | [`ground_truth_flag_overrides_block_eligibility`] | scenario | A blocked observation normally teaches nothing, because the outcome was never allowed to happen — but a label marked as ground truth is accepted alongside a block action rather than filtered out, and the engine stays clean and well-formed on the next assessment of that entity. The eligibility bypass is reachable from the public surface, which is what lets a host feed back what it independently knows about a request it stopped. |
//! | [`repeated_identical_adverse_cycle_stays_finite`] | scenario | cites (´claim:scenario:repeated-identical-labelled-cycles-do-not-diverge-or-collapse´) |
//! | [`cold_start_assessment_near_half`] | scenario | A world that has learned nothing says so. Its very first assessment sits near a half with strictly positive uncertainty, reports no sentinels reporting, and raises the zero-sentinel flag in the health snapshot without that counting as degradation. Ignorance is expressed as a confessed coin-flip rather than as a confident number or an error, so a host can tell "nothing is known yet" apart from "this is genuinely borderline". |

//! Integration tests for the corners where the guarantees are thinnest
//! (´chap:spec:guarantees-and-limitations´).
//!
//! Clock-independent scenarios: each one probes a corner of the
//! assessment, derivation, or label pipelines that is reachable through today's
//! public API without engine-side time travel.
//!
//! # Cross-References
//!
//! - (´claim:scenario:the-engine-assesses-and-derives-with-no-sentinels-registered´) — the engine holds up on a world with nothing registered
//! - (´dec:surface:async-label´) — the label-submission contract

use torrust_assayer::error::LabelError;
use torrust_assayer::testing::{
    LabelSpec, assert_finite, assert_health_clean, assert_in_unit_interval, assert_reckoning_well_formed, scenario,
};
use torrust_assayer::types::{Action, AssessmentId};

/// Stable instance id + seed pair shared by every scenario in this
/// file. Centralised here so a future seed regeneration touches one
/// site instead of nine.
const INSTANCE: &str = "edge-cases";
const SEED: u64 = 0x0042;

/// Sentinels are an enrichment, not a prerequisite: with none registered at
/// all, an unseen entity still assesses and derives to a well-formed reckoning
/// and the health snapshot stays clean. A host can therefore stand the engine
/// up and get usable answers before it has wired any sentinel in, rather than
/// having to complete the whole installation before the first request.
///
/// ´claim:scenario:the-engine-assesses-and-derives-with-no-sentinels-registered´
/// ´test:integration:assess-and-derive-with-no-sentinels-returns-valid-output´
#[test]
fn assess_and_derive_with_no_sentinels_returns_valid_output() {
    let s = scenario(INSTANCE, SEED);

    let reckoning = s.derive_default("alice").expect("assess should succeed without sentinels");

    // Fresh engine, unseen entity, no sentinels — blend should not
    // produce anything wild. This test is primarily a smoke check
    // that the zero-Sentinel path does not trip the pipeline.
    assert_reckoning_well_formed(&reckoning, "p_bad");
    assert_health_clean(&s);
}

/// An assessment can be labelled exactly once. The first label consumes its
/// entry in the pending buffer, and a second label naming the same assessment
/// is refused as not-found rather than applied again. Retries and duplicate
/// deliveries on the host's side therefore cannot teach the models the same
/// outcome twice and quietly inflate its weight.
///
/// ´claim:scenario:a-label-consumes-its-pending-entry-so-a-second-label-cannot-update-twice´
/// ´test:integration:duplicate-label-rejected-without-double-update´
#[test]
fn duplicate_label_rejected_without_double_update() {
    let s = scenario(INSTANCE, SEED);
    let reckoning = s.derive_default("bob").expect("first reckoning");

    // First label consumes the pending-buffer entry.
    s.label(LabelSpec::benign(reckoning.assessment.id).build())
        .expect("first label should be accepted");

    // Second label for the same assessment — the pending entry is
    // gone, so the engine returns `AssessmentNotFound`.
    let err = s
        .label(LabelSpec::benign(reckoning.assessment.id).build())
        .expect_err("second label should be rejected");
    assert!(
        matches!(err, LabelError::AssessmentNotFound),
        "expected AssessmentNotFound, got {err:?}",
    );
}

/// An assessment id the engine never issued is refused with the same
/// not-found error an evicted one gets, and the refusal is inert: the very
/// next assess-derive-label cycle on a different entity succeeds and is
/// well-formed. A rejected label is a rejected message, not damage — a host
/// replaying a stale queue cannot brick the engine by getting an id wrong.
///
/// ´claim:scenario:a-rejected-label-leaves-the-engine-fully-operational´
/// ´test:integration:label-with-unknown-assessment-id-is-rejected´
#[test]
fn label_with_unknown_assessment_id_is_rejected() {
    // The expiry case
    // (´claim:pending:entries-past-the-expiry-horizon-are-dropped-even-well-under-capacity´)
    // is about an *expired* pending entry — clock
    // injection (Stage 2) is required to drive the TTL path. The
    // error surface the host must handle is identical when the
    // assessment ID was simply never issued by this engine: an
    // unknown id is the generalisation of an evicted one.
    //
    // `AssessmentId(u64::MAX)` is vanishingly unlikely to ever be
    // allocated by the monotonic id generator, so it exercises the
    // "not in the pending buffer" path without relying on timing.
    let s = scenario(INSTANCE, SEED);
    let err = s
        .label(LabelSpec::benign(AssessmentId(u64::MAX)).build())
        .expect_err("labeling an unknown id must fail");
    assert!(
        matches!(err, LabelError::AssessmentNotFound),
        "expected AssessmentNotFound, got {err:?}",
    );

    // The engine must not have been poisoned by the error —
    // subsequent assess/derive/label cycles still succeed.
    let reckoning = s.cycle_benign("carol");
    assert_reckoning_well_formed(&reckoning, "post-rejection cycle p_bad");
}

/// Valence is the host's convention and the engine does not second-guess it:
/// labels carrying either sign on the same entity are accepted without any
/// consistency check against prior belief, and the engine stays numerically
/// sound and clean afterwards. A host that inverts the convention gets a wrong
/// model rather than a broken one — the fault stays diagnosable as the host's,
/// which it could not be if the engine silently corrected or rejected signs.
///
/// ´claim:scenario:the-engine-trusts-the-host-valence-convention-without-an-integrity-check´
/// ´test:integration:inverted-valence-signs-accepted-without-corruption´
#[test]
fn inverted_valence_signs_accepted_without_corruption() {
    // The engine honours the host's valence convention — positive valence is
    // adverse (´chap:spec:valence-asymmetry´) — and
    // performs no sanity check against prior beliefs. A host that
    // labels with flipped signs (benign outcomes tagged positive,
    // adverse outcomes tagged negative) gets what it asked for;
    // the fault is the host's.
    //
    // This slice asserts only the *accept* half of the scenario
    // (no streaming + model-drift observation) — the engine does
    // not reject the label, does not flip any health flag, and
    // remains operational for subsequent assess/derive/label cycles.
    let s = scenario(INSTANCE, SEED);
    let first = s.derive_default("alice").expect("first reckoning");

    // Inverted convention: host treats this as adverse but labels
    // it with negative valence (the "benign" shorthand).
    s.label(LabelSpec::new(first.assessment.id).valence(-1.0).build())
        .expect("engine accepts inverted-valence label");

    // A second, "positive-valence-means-benign" label on the next
    // derived output should likewise round-trip without complaint.
    let second = s.derive_default("alice").expect("second reckoning");
    s.label(LabelSpec::new(second.assessment.id).valence(1.0).build())
        .expect("engine accepts flipped positive-valence label");

    // Subsequent derived outputs remain finite and in (0, 1) — the
    // inverted convention did not corrupt numeric state.
    let third = s.derive_default("alice").expect("third reckoning");
    assert_reckoning_well_formed(&third, "post-inverted p_bad");
    assert_health_clean(&s);
}

/// Two hundred rounds of assessing one unchanging entity and labelling it
/// benign leave every reckoning well-formed and the health snapshot clean.
/// The negative-valence update path is bounded, so an entity a host sees over
/// and over cannot drive the posterior to collapse against its own boundary or
/// out of the unit interval no matter how one-sided the evidence gets.
///
/// ´claim:scenario:repeated-identical-labelled-cycles-do-not-diverge-or-collapse´
/// ´test:integration:repeated-identical-benign-cycle-stays-finite´
#[test]
fn repeated_identical_benign_cycle_stays_finite() {
    // 200 rounds of `assess → label(benign)` with identical inputs. The
    // wider property is leverage bounding and EWMA convergence
    // (´claim:bayes:the-leverage-bound-fires-at-cold-start-and-rarely-after´);
    // this slice only pins the
    // "no posterior collapse / no NaN / no error" floor —
    // sufficient to catch regressions that cause the pipeline
    // to diverge under repeat-exposure.
    let s = scenario(INSTANCE, SEED);

    for i in 0..200 {
        let r = s.cycle_benign("alice");
        assert_reckoning_well_formed(&r, &format!("round {i} p_bad"));
    }

    assert_health_clean(&s);
}

/// A blocked observation normally teaches nothing, because the outcome was
/// never allowed to happen — but a label marked as ground truth is accepted
/// alongside a block action rather than filtered out, and the engine stays
/// clean and well-formed on the next assessment of that entity. The
/// eligibility bypass is reachable from the public surface, which is what lets
/// a host feed back what it independently knows about a request it stopped.
///
/// ´claim:scenario:a-ground-truth-label-is-accepted-despite-a-block-action´
/// ´test:integration:ground-truth-flag-overrides-block-eligibility´
#[test]
fn ground_truth_flag_overrides_block_eligibility() {
    // The ground-truth flag overrides eligibility for blocked observations
    // (´conv:eligibility:ground-truth´):
    // that `ground_truth: true` + `Action::Block` updates
    // every model (sister, anchor, ledger, operational) by
    // overriding the Block-eligibility filter. This slice only
    // pins the *public-surface* invariant that the ground-truth
    // path does not reject the label — the full "did every model
    // update?" check requires crate-level model-state inspection
    // and lives in `label_pipeline::steps4_8_ground_truth_overrides`.
    //
    // Assertion: submitting a Block + ground_truth label succeeds,
    // health stays clean, and a subsequent assess/derive on the same
    // entity is well-formed (proving the engine was not poisoned
    // by the eligibility-bypass branch).
    let s = scenario(INSTANCE, SEED);
    let r = s.derive_default("alice").expect("assess");

    s.label(
        LabelSpec::new(r.assessment.id)
            .action(Action::Block)
            .valence(1.5)
            .ground_truth()
            .build(),
    )
    .expect("ground-truth + Block label must be accepted");

    let after = s.derive_default("alice").expect("post-label assess");
    assert_reckoning_well_formed(&after, "post-ground-truth p_bad");
    assert_health_clean(&s);
}

/// The adverse branch holds up under the same punishment as the benign one:
/// two hundred rounds of labelling one entity adverse keep the posterior
/// finite and inside the unit interval and the uncertainty non-negative
/// throughout. This is the branch carrying importance weighting and the
/// leverage bound, where an unbounded update would show first, so it is the
/// side worth driving to saturation.
///
/// (´claim:scenario:repeated-identical-labelled-cycles-do-not-diverge-or-collapse´)
/// ´test:integration:repeated-identical-adverse-cycle-stays-finite´
#[test]
fn repeated_identical_adverse_cycle_stays_finite() {
    // The adverse branch: complement to
    // [`repeated_identical_benign_cycle_stays_finite`]. The benign
    // variant exercises the $v < 0$ update path; this one drives
    // 200 rounds of `assess → adverse-label` to confirm the
    // positive-valence branch of the update (where importance
    // weighting and the leverage bound carry most of the runtime
    // complexity) also survives repeat-exposure without posterior
    // collapse, NaN, or non-finite reports.
    let s = scenario(INSTANCE, SEED);

    for i in 0..200 {
        let r = s.cycle_adverse("alice");
        assert_finite(
            &[r.assessment.risk.p_bad, r.assessment.risk.uncertainty],
            &format!("round {i}"),
        );
        assert_in_unit_interval(r.assessment.risk.p_bad, &format!("round {i} p_bad"));
        assert!(
            r.assessment.risk.uncertainty >= 0.0,
            "round {i}: uncertainty {} must be non-negative",
            r.assessment.risk.uncertainty,
        );
    }

    assert_health_clean(&s);
}

/// A world that has learned nothing says so. Its very first assessment sits
/// near a half with strictly positive uncertainty, reports no sentinels
/// reporting, and raises the zero-sentinel flag in the health snapshot without
/// that counting as degradation. Ignorance is expressed as a confessed
/// coin-flip rather than as a confident number or an error, so a host can tell
/// "nothing is known yet" apart from "this is genuinely borderline".
///
/// ´claim:scenario:a-cold-world-answers-near-a-half-and-declares-its-ignorance´
/// ´test:integration:cold-start-assessment-near-half´
#[test]
fn cold_start_assessment_near_half() {
    // The first step of the cold-start property
    // (´inv:guarantee:honest-uncertainty´):
    // start with 0 labels and verify
    // $\hat{p} \approx 0.5$ for everything (maximum uncertainty)."
    //
    // A freshly-built `World` has no Sentinels registered, no
    // labels processed, and no identity dimensions — the feature
    // vector collapses to bias + zero-filled aggregates, and the
    // prior-only sigmoid should sit very close to 0.5. The exact
    // value drifts slightly off 0.5 because the bias is
    // standardised and the Platt calibration starts at the
    // `ColdStart` composite stage; crate-level api-surface tests
    // already use a ±0.1 tolerance for the same invariant, which
    // is what we adopt here.
    let s = scenario(INSTANCE, SEED);
    let r = s.derive_default("alice").expect("cold assess");

    assert_finite(
        &[
            r.assessment.risk.p_bad,
            r.assessment.risk.uncertainty,
            r.assessment.risk.anchor_weight,
        ],
        "cold assessment",
    );
    assert_in_unit_interval(r.assessment.risk.p_bad, "cold p_bad");
    assert!(
        (r.assessment.risk.p_bad - 0.5).abs() < 0.1,
        "cold-start p_bad should be near 0.5, got {}",
        r.assessment.risk.p_bad,
    );

    // The prior is maximally uncertain — the reported uncertainty
    // on a cold world must be strictly positive. (The wider property
    // (´inv:guarantee:honest-uncertainty´)
    // puts it *at its ceiling*; the public
    // `RiskBasis.uncertainty` field is a probability-space
    // quantity whose ceiling depends on $\sigma_\text{eff}$, so
    // we pin only the weaker sign/finite invariant here.)
    assert!(
        r.assessment.risk.uncertainty > 0.0,
        "cold-start uncertainty should be strictly positive, got {}",
        r.assessment.risk.uncertainty,
    );

    // No Sentinels are registered, so the health snapshot should
    // flag the zero-Sentinel prior-only operating mode.
    assert!(
        r.assessment.health.zero_sentinels,
        "cold world has no Sentinels: zero_sentinels should be true"
    );
    assert_eq!(
        r.assessment.risk.n_sentinels_reporting, 0,
        "no Sentinels reporting on a cold world"
    );

    assert_health_clean(&s);
}
