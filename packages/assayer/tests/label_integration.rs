// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`label_data_positive_spam`] | labelling | A label carries the valence and the ground-truth flag it was built with, out to the boundary a host actually uses. These two fields decide how hard the label pushes the model and whether it is admissible base-rate evidence at all, so a builder that quietly altered either would change the meaning of what the host reported. |
//! | [`label_data_benign`] | labelling | The defaults a host gets without asking are the harmless ones: the action is allow, and the outcome counts as inferred rather than established. Both omissions therefore claim less than the host might mean rather than more — an unstated action is not read as an intervention, and an unstated provenance is not read as confirmed truth. |
//! | [`companion_tracker_records_challenge_result`] | labelling | A failed challenge is evidence about the channel that issued it: recording one raises that channel's estimate of adverse traffic above the neutral prior and brings the channel into the tracker. The estimate is returned from the update itself, so a host learns the consequence of the evidence at the moment it supplies it rather than having to go back and ask. |
//! | [`world_routes_challenge_result_to_companion_tracker`] | labelling | Challenge evidence reaches later derivations only through an assessment the system actually issued: a known identifier routes the outcome to its channel and visibly changes the challenge tag the next derivation produces, while an unrecognised one is refused outright and changes nothing. Routing by identifier is what stops a host — or anything upstream of it — from feeding effectiveness evidence about traffic that never passed through the system. |
//! | [`label_data_ground_truth_flag`] | labelling | cites (´claim:labelling:a-label-carries-out-the-valence-and-ground-truth-it-was-built-with´) |
//! | [`label_data_multi_axis_outcomes`] | labelling | One label can report an outcome on several axes at once, each keyed to its own axis and keeping its own value and sign. A single request can be good on one dimension and bad on another, and collapsing that into one number — or letting one axis overwrite another — would lose exactly the distinction multiple axes exist to express. |
//! | [`label_data_action_slow`] | labelling | The action recorded is whichever one the host actually took, including the milder interventions: a request that was slowed rather than blocked is labelled as slowed. Eligibility turns on what the system did to the request, so an intervention flattened into allow or block would be judged against the wrong counterfactual. |
//! | [`label_data_is_clone`] | labelling | A label can be copied, and the copy carries the same valence and the same assessment identifier. Submission consumes the label, so a host that wants to keep a record of what it sent — for its own audit, or to retry after a full channel — needs to be able to take a copy first. |
//! | [`label_data_is_debug`] | labelling | A label renders under debug formatting with its type and its field names visible, not as an opaque handle. A rejected or surprising label is diagnosed by logging it, and a rendering that omitted the field names would leave an operator guessing which number was which. |
//! | [`label_data_is_send_sync`] | labelling | A label may cross thread boundaries and be shared across them — the compiler itself is the witness, since the assertion is a bound that would fail to build otherwise. Labels are built wherever the outcome became known and handed to the single owner thread through a channel, so a label that could not be sent could not be submitted at all. |
//! | [`label_data_serde_roundtrip`] | labelling | A label survives being written out and read back with its valence, provenance, assessment identifier and action all intact. This is the journal round trip in miniature: replay after a crash re-applies labels read from disk, so a field lost or altered in transit would make the recovered model differ from the one that crashed. |
//! | [`challenge_tracker_serde_roundtrip`] | labelling | The challenge tracker survives a round trip with its accumulated evidence, not merely its shape: a channel that had seen a failure still reports an elevated estimate after being restored. The tracker's whole value is memory of past challenges, so a restart that kept the channels but forgot what they had seen would return every channel to the neutral prior. |

//! Integration tests for label pipeline public API types.
//!
//! Tests the public surface of `LabelData`, `Action`, and the challenge
//! companion tracker — the types exposed to host applications that feed
//! ground-truth outcomes into the Assayer.
//!
//! # Cross-References
//!
//! - (´chap:spec:label-pipeline´) — label pipeline orchestration
//! - (´dec:surface:distinct-action-types´) — what the host did and what a derivation proposes are distinct types
//! - (´dec:construction:six-methods´) — registration and deregistration among the lifecycle methods

use torrust_assayer::testing::{LabelSpec, scenario};
use torrust_assayer::types::{Action, AssessmentId, ChallengeResult, ChannelId, OutcomeAxisId, PersistentTimestamp};
use torrust_assayer::{ChallengeEffectivenessTracker, DerivedReckoning, LabelData, Tag};

// ═══════════════════════════════════════════════════════════════════════════════
// LabelData Construction
// ═══════════════════════════════════════════════════════════════════════════════

/// Build a label on `AssessmentId(1)`, varying only the fields each case cares about.
fn make_label(valence: f64, ground_truth: bool) -> LabelData {
    let mut spec = LabelSpec::new(AssessmentId(1)).valence(valence);
    if ground_truth {
        spec = spec.ground_truth();
    }
    spec.build()
}

/// A label carries the valence and the ground-truth flag it was built with, out to
/// the boundary a host actually uses. These two fields decide how hard the label
/// pushes the model and whether it is admissible base-rate evidence at all, so a
/// builder that quietly altered either would change the meaning of what the host
/// reported.
///
/// ´claim:labelling:a-label-carries-out-the-valence-and-ground-truth-it-was-built-with´
/// ´test:integration:label-data-positive-spam´
#[test]
fn label_data_positive_spam() {
    let label = make_label(5.0, true);
    assert_eq!(label.valence.to_bits(), 5.0f64.to_bits());
    assert!(label.ground_truth);
}

/// The defaults a host gets without asking are the harmless ones: the action is
/// allow, and the outcome counts as inferred rather than established. Both
/// omissions therefore claim less than the host might mean rather than more — an
/// unstated action is not read as an intervention, and an unstated provenance is
/// not read as confirmed truth.
///
/// ´claim:labelling:an-unspecified-label-defaults-to-allow-and-to-inferred-provenance´
/// ´test:integration:label-data-benign´
#[test]
fn label_data_benign() {
    let label = make_label(0.0, false);
    assert_eq!(label.valence.to_bits(), 0.0f64.to_bits());
    assert!(!label.ground_truth);
    assert_eq!(label.action_taken, Action::Allow);
}

/// A failed challenge is evidence about the channel that issued it: recording one
/// raises that channel's estimate of adverse traffic above the neutral prior and
/// brings the channel into the tracker. The estimate is returned from the update
/// itself, so a host learns the consequence of the evidence at the moment it
/// supplies it rather than having to go back and ask.
///
/// ´claim:labelling:a-failed-challenge-raises-the-channel-estimate-of-adverse-traffic´
/// ´test:integration:companion-tracker-records-challenge-result´
#[test]
fn companion_tracker_records_challenge_result() {
    let mut tracker = ChallengeEffectivenessTracker::new();

    let estimate = tracker.update(ChannelId(1), ChallengeResult::Fail, &PersistentTimestamp::now());

    assert!(estimate.q_c() > 0.5);
    assert_eq!(tracker.len(), 1);
}

/// Challenge evidence reaches later derivations only through an assessment the
/// system actually issued: a known identifier routes the outcome to its channel and
/// visibly changes the challenge tag the next derivation produces, while an
/// unrecognised one is refused outright and changes nothing. Routing by identifier
/// is what stops a host — or anything upstream of it — from feeding effectiveness
/// evidence about traffic that never passed through the system.
///
/// ´claim:labelling:only-a-known-assessment-identifier-routes-challenge-evidence-into-later-derivations´
/// ´test:integration:world-routes-challenge-result-to-companion-tracker´
#[test]
fn world_routes_challenge_result_to_companion_tracker() {
    let world = scenario("label-companion-routing", 0x1ABE_1000);
    let assessment = world.assess(world.request("default", "alice"));

    let before = world.derive(&assessment, "default");
    let challenged = world.derive_default("bob").expect("host derives challenged request");

    assert!(
        world.record_challenge_result(challenged.assessment.id, ChallengeResult::Fail),
        "known assessment ID should route to a host derivation channel"
    );
    assert!(
        !world.record_challenge_result(AssessmentId(u64::MAX), ChallengeResult::Pass),
        "unknown assessment ID should not update companion state"
    );

    let after = world.derive(&assessment, "default");
    assert_ne!(
        challenge_tag_fingerprint(&before),
        challenge_tag_fingerprint(&after),
        "host-routed challenge evidence should feed the next explicit derivation estimate"
    );
}

fn challenge_tag_fingerprint(reckoning: &DerivedReckoning) -> (u64, u64, u64) {
    let tag = reckoning
        .profile
        .tags
        .iter()
        .find(|tag| tag.tag == Tag::Challenge)
        .expect("default channel emits a Challenge action tag");
    (tag.location.to_bits(), tag.magnitude.to_bits(), tag.q.to_bits())
}

/// Provenance is independent of severity: two labels reporting the same valence
/// remain distinguishable as established and inferred. A host that knows what
/// happened and one that guessed can report the same magnitude, and the pipeline
/// still tells their evidence apart.
///
/// (´claim:labelling:a-label-carries-out-the-valence-and-ground-truth-it-was-built-with´)
/// ´test:integration:label-data-ground-truth-flag´
#[test]
fn label_data_ground_truth_flag() {
    let gt = make_label(1.0, true);
    let inferred = make_label(1.0, false);
    assert!(gt.ground_truth);
    assert!(!inferred.ground_truth);
}

/// One label can report an outcome on several axes at once, each keyed to its own
/// axis and keeping its own value and sign. A single request can be good on one
/// dimension and bad on another, and collapsing that into one number — or letting
/// one axis overwrite another — would lose exactly the distinction multiple axes
/// exist to express.
///
/// ´claim:labelling:one-label-can-report-a-distinct-outcome-on-each-axis´
/// ´test:integration:label-data-multi-axis-outcomes´
#[test]
fn label_data_multi_axis_outcomes() {
    let label: LabelData = LabelSpec::new(AssessmentId(42))
        .action(Action::Challenge)
        .valence(2.0)
        .ground_truth()
        .outcome(OutcomeAxisId(1), 0.8)
        .outcome(OutcomeAxisId(2), -0.3)
        .outcome(OutcomeAxisId(3), 1.0)
        .into();

    assert_eq!(label.outcomes.len(), 3);
    assert_eq!(label.outcomes[&OutcomeAxisId(1)].to_bits(), 0.8f64.to_bits());
    assert_eq!(label.outcomes[&OutcomeAxisId(2)].to_bits(), (-0.3f64).to_bits());
    assert_eq!(label.action_taken, Action::Challenge);
}

/// The action recorded is whichever one the host actually took, including the
/// milder interventions: a request that was slowed rather than blocked is labelled
/// as slowed. Eligibility turns on what the system did to the request, so an
/// intervention flattened into allow or block would be judged against the wrong
/// counterfactual.
///
/// ´claim:labelling:the-action-recorded-is-whichever-intervention-the-host-actually-took´
/// ´test:integration:label-data-action-slow´
#[test]
fn label_data_action_slow() {
    let label = LabelSpec::new(AssessmentId(1))
        .valence(1.5)
        .ground_truth()
        .action(Action::Slow)
        .build();
    assert_eq!(label.action_taken, Action::Slow);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Trait Bounds
// ═══════════════════════════════════════════════════════════════════════════════

/// A label can be copied, and the copy carries the same valence and the same
/// assessment identifier. Submission consumes the label, so a host that wants to
/// keep a record of what it sent — for its own audit, or to retry after a full
/// channel — needs to be able to take a copy first.
///
/// ´claim:labelling:a-label-can-be-copied-so-a-host-may-keep-what-it-submitted´
/// ´test:integration:label-data-is-clone´
#[test]
fn label_data_is_clone() {
    let label = make_label(1.0, true);
    let cloned = label.clone();
    assert_eq!(cloned.valence.to_bits(), label.valence.to_bits());
    assert_eq!(cloned.assessment_id, label.assessment_id);
}

/// A label renders under debug formatting with its type and its field names
/// visible, not as an opaque handle. A rejected or surprising label is diagnosed by
/// logging it, and a rendering that omitted the field names would leave an
/// operator guessing which number was which.
///
/// ´claim:labelling:a-label-renders-its-type-and-field-names-under-debug-formatting´
/// ´test:integration:label-data-is-debug´
#[test]
fn label_data_is_debug() {
    let label = make_label(1.0, false);
    let debug = format!("{label:?}");
    assert!(debug.contains("LabelData"), "Debug output should contain type name");
    assert!(debug.contains("valence"), "Debug output should contain field name");
}

/// A label may cross thread boundaries and be shared across them — the compiler
/// itself is the witness, since the assertion is a bound that would fail to build
/// otherwise. Labels are built wherever the outcome became known and handed to the
/// single owner thread through a channel, so a label that could not be sent could
/// not be submitted at all.
///
/// ´claim:labelling:a-label-may-cross-and-be-shared-across-thread-boundaries´
/// ´test:integration:label-data-is-send-sync´
#[test]
fn label_data_is_send_sync() {
    fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<LabelData>();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serde Round-Trip
// ═══════════════════════════════════════════════════════════════════════════════

/// A label survives being written out and read back with its valence, provenance,
/// assessment identifier and action all intact. This is the journal round trip in
/// miniature: replay after a crash re-applies labels read from disk, so a field
/// lost or altered in transit would make the recovered model differ from the one
/// that crashed.
///
/// ´claim:labelling:a-label-survives-a-serialisation-round-trip-with-every-field-intact´
/// ´test:integration:label-data-serde-roundtrip´
#[cfg(feature = "serde")]
#[test]
fn label_data_serde_roundtrip() {
    let label = make_label(3.5, true);
    let json = serde_json::to_string(&label).expect("serialise");
    let restored: LabelData = serde_json::from_str(&json).expect("deserialise");

    assert_eq!(restored.valence.to_bits(), label.valence.to_bits());
    assert_eq!(restored.ground_truth, label.ground_truth);
    assert_eq!(restored.assessment_id, label.assessment_id);
    assert_eq!(restored.action_taken, label.action_taken);
}

/// The challenge tracker survives a round trip with its accumulated evidence, not
/// merely its shape: a channel that had seen a failure still reports an elevated
/// estimate after being restored. The tracker's whole value is memory of past
/// challenges, so a restart that kept the channels but forgot what they had seen
/// would return every channel to the neutral prior.
///
/// ´claim:labelling:the-challenge-tracker-survives-a-round-trip-with-its-evidence-intact´
/// ´test:integration:challenge-tracker-serde-roundtrip´
#[cfg(feature = "serde")]
#[test]
fn challenge_tracker_serde_roundtrip() {
    let mut tracker = ChallengeEffectivenessTracker::new();
    let now = PersistentTimestamp::now();
    tracker.update(ChannelId(1), ChallengeResult::Fail, &now);

    let json = serde_json::to_string(&tracker).expect("serialise");
    let restored: ChallengeEffectivenessTracker = serde_json::from_str(&json).expect("deserialise");

    assert!(restored.estimate(ChannelId(1), &now).q_c() > 0.5);
}

// Note: `LifecycleError` Display and Send + Sync bounds are covered
// exhaustively in `tests/error.rs`.
