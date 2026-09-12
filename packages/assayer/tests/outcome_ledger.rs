// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`root_destroyed_on_deregister_then_fresh_on_reregister`] | ledger | Deregistering a Sentinel genuinely destroys its ledger state and re-registering the same name builds a fresh one: the name stops resolving the moment it is deregistered, and derivations after re-registration are as well-formed as those before, with engine health clean across the whole cycle. Nothing half-collapsed is left occupying the slot, so an operator can retire and reinstate a Sentinel without its old outcome history leaking into the new lifetime. |
//! | [`root_survives_many_assessments_within_lifetime`] | ledger | Within a Sentinel's lifetime the root is always there to route into: five hundred consecutive derivations for the same entity all succeed and all yield well-formed output, with health clean at the end. Nothing about sustained traffic — not cell-set churn, not decay, not collection — can erode the destination every assessment ultimately falls back to. |
//! | [`root_serves_many_entities_independently`] | ledger | One Sentinel's ledger serves every entity that passes through it, and the assessments stay individually addressable: twenty distinct entities each get a well-formed derivation, every assessment id handed out is unique, and each pending assessment survives until its own label arrives rather than being consumed by another entity's. Pending state is keyed on the assessment, not on the entity or the cell it routed to, so a busy Sentinel cannot cross-wire two entities' outcomes. |
//! | [`root_serves_multiple_channels_for_one_sentinel`] | ledger | A Sentinel's ledger is scoped to the Sentinel and not to a channel: one registration serves full assess-label cycles on three separate declared channels, each producing well-formed output. A host adding a channel therefore does not have to re-register its Sentinels, and outcome history accumulated on one channel is the same body of evidence the others read. |
//! | [`root_survives_repeated_reregister_cycles`] | ledger | cites (´claim:ledger:deregistering-a-sentinel-destroys-its-ledger-and-re-registering-builds-a-fresh-one´) |
//! | [`outcome_axis_registration_does_not_perturb_risk_basis`] | ledger | Registering an outcome axis is a structural change only: the same entity's core risk basis before and after the registration agrees to within the documented floating-point drift band. An axis with no reported values has nothing to say, so its mere existence must not move the scalars the decision is actually made from — otherwise declaring an axis would silently re-price every entity in the deployment. |
//! | [`outcome_axis_registration_does_not_perturb_tag_profile`] | ledger | The same registration boundary leaves the emitted tag profile unchanged within the drift band. The tags are the part of the output a host reads downstream, so this is the visible face of the same neutrality: adding an axis must not change what any consumer of the profile sees until the axis has values of its own to contribute. |
//! | [`outcome_predictions_do_not_perturb_reckoning_when_core_history_matches`] | ledger | Outcome-axis predictions stay outside the derivation even once they are live. Two worlds driven through the same twelve-cycle risk-label history differ only in that one also reports outcome values; that world emits a finite prediction with its uncertainty band while the control emits none, and yet both end with matching risk bases and matching tag profiles. Predictions are an output the host may consult, never an input that feeds back into the risk the engine assigns. |
//! | [`different_outcome_prediction_values_do_not_perturb_reckoning_when_core_history_matches`] | ledger | cites (´claim:ledger:live-outcome-predictions-stay-outside-the-derivation´) |

//! Integration tests for the outcome ledger
//! (´chap:spec:outcome-ledger´).
//!
//! The ledger scenarios
//! (´chap:spec:outcome-ledger´)
//! live deep inside the engine (the `OutcomeLedger`
//! is not part of the public surface), so crate-level tests in
//! `src/tests/ledger.rs` own the bulk of the coverage. The tests in
//! this file exercise the *public* lifecycle and assess/derive surface to
//! prove that the Sentinel-scoped Ledger state the spec describes is
//! observable from the outside — specifically, that deregistering a
//! Sentinel genuinely destroys its state (no leakage into the
//! re-registered slot).
//!
//! # Cross-References
//!
//! - (´dec:memory:root-permanence´) — the root entry persists within a lifetime and is destroyed on deregistration
//! - (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´) — outcome-axis predictions do not enter the resonance
//! - (´dec:construction:six-methods´) — registration and deregistration among the lifecycle methods

use torrust_assayer::testing::{
    LabelSpec, PURITY_DRIFT, assert_core_risk_basis_near, assert_finite, assert_health_clean, assert_reckoning_well_formed,
    assert_risk_basis_near, assert_tag_profile_near, scenario, scenario_with,
};

const INSTANCE: &str = "outcome-ledger";
const SEED: u64 = 0x00DD_BA11;

/// Deregistering a Sentinel genuinely destroys its ledger state and
/// re-registering the same name builds a fresh one: the name stops resolving
/// the moment it is deregistered, and derivations after re-registration are as
/// well-formed as those before, with engine health clean across the whole
/// cycle. Nothing half-collapsed is left occupying the slot, so an operator
/// can retire and reinstate a Sentinel without its old outcome history leaking
/// into the new lifetime.
///
/// ´claim:ledger:deregistering-a-sentinel-destroys-its-ledger-and-re-registering-builds-a-fresh-one´
/// ´test:integration:root-destroyed-on-deregister-then-fresh-on-reregister´
#[test]
fn root_destroyed_on_deregister_then_fresh_on_reregister() {
    let mut s = scenario(INSTANCE, SEED);

    // Register → assess/derive → deregister → re-register → assess/derive. The
    // observable signal that the root was destroyed and re-created
    // is that every assess() succeeds, health stays clean across
    // the cycle, and the post-re-registration reckoning is as
    // well-formed as the pre-deregistration one (i.e. the engine
    // did not retain a half-collapsed state for the Sentinel's
    // slot).
    s.register_sentinel("S1").expect("initial register");

    let before = s.derive_default("alice").expect("reckoning before deregister");
    assert_reckoning_well_formed(&before, "before.p_bad");

    s.deregister_sentinel("S1").expect("deregister S1");

    // After deregistration the Sentinel is gone from the registry,
    // so the harness-side name lookup must fail.
    assert_eq!(s.sentinel("S1"), None);

    s.register_sentinel("S1").expect("re-register S1");

    let after = s.derive_default("alice").expect("reckoning after re-register");
    assert_reckoning_well_formed(&after, "after.p_bad");

    assert_health_clean(&s);
}

/// Within a Sentinel's lifetime the root is always there to route into: five
/// hundred consecutive derivations for the same entity all succeed and all
/// yield well-formed output, with health clean at the end. Nothing about
/// sustained traffic — not cell-set churn, not decay, not collection — can
/// erode the destination every assessment ultimately falls back to.
///
/// ´claim:ledger:the-root-remains-available-to-route-into-for-a-sentinels-whole-lifetime´
/// ´test:integration:root-survives-many-assessments-within-lifetime´
#[test]
fn root_survives_many_assessments_within_lifetime() {
    let mut s = scenario(INSTANCE, SEED);
    s.register_sentinel("S1").expect("register S1");

    // The root is permanent within a Sentinel's lifetime
    // (´dec:memory:root-permanence´).
    // 500 bare assessments should not trip any pipeline
    // state — the root is always there to route into.
    for i in 0..500 {
        let r = s
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("assessment/derivation #{i} failed: {e:?}"));
        assert_reckoning_well_formed(&r, &format!("#{i}.p_bad"));
    }

    assert_health_clean(&s);
}

/// One Sentinel's ledger serves every entity that passes through it, and the
/// assessments stay individually addressable: twenty distinct entities each
/// get a well-formed derivation, every assessment id handed out is unique, and
/// each pending assessment survives until its own label arrives rather than
/// being consumed by another entity's. Pending state is keyed on the
/// assessment, not on the entity or the cell it routed to, so a busy Sentinel
/// cannot cross-wire two entities' outcomes.
///
/// ´claim:ledger:one-sentinels-ledger-serves-every-entity-with-assessments-tracked-per-id´
/// ´test:integration:root-serves-many-entities-independently´
#[test]
fn root_serves_many_entities_independently() {
    let mut s = scenario(INSTANCE, SEED);
    s.register_sentinel("S1").expect("register S1");

    // A single Sentinel's root
    // (´dec:memory:root-permanence´)
    // is the routing
    // destination for every assessment in the Sentinel's lifetime,
    // *regardless* of entity. Issue 20 assessments against 20
    // distinct entity names and label each one — every assess()
    // and label() round-trip must succeed (proving the pending
    // buffer is keyed on the monotonic `AssessmentId`, not on the
    // entity or the root), and the root is demonstrably there to
    // route each one into.
    let names = [
        "alice", "bob", "carol", "dave", "eve", "frank", "grace", "heidi", "ivan", "judy", "kelly", "lars", "mary", "nate",
        "olive", "peggy", "quinn", "randy", "sue", "trent",
    ];
    let mut ids = Vec::with_capacity(names.len());

    for name in names {
        let r = s
            .derive_default(name)
            .unwrap_or_else(|e| panic!("assess/derive for {name} failed: {e:?}"));
        assert_reckoning_well_formed(&r, name);
        ids.push(r.assessment.id);
    }

    // Every id the engine handed out is unique — the monotonic
    // generator hasn't recycled an id across entities.
    let mut sorted = ids.clone();
    sorted.sort();
    sorted.dedup();
    assert_eq!(sorted.len(), ids.len(), "every assessment ID is unique");

    // Each pending entry survives until *its own* label arrives;
    // one entity's label does not consume another's slot.
    for id in ids {
        s.label(LabelSpec::benign(id).build())
            .unwrap_or_else(|e| panic!("label for id {id:?} failed: {e:?}"));
    }

    assert_health_clean(&s);
}

/// A Sentinel's ledger is scoped to the Sentinel and not to a channel: one
/// registration serves full assess-label cycles on three separate declared
/// channels, each producing well-formed output. A host adding a channel
/// therefore does not have to re-register its Sentinels, and outcome history
/// accumulated on one channel is the same body of evidence the others read.
///
/// ´claim:ledger:a-sentinels-ledger-is-scoped-to-the-sentinel-not-to-a-channel´
/// ´test:integration:root-serves-multiple-channels-for-one-sentinel´
#[test]
fn root_serves_multiple_channels_for_one_sentinel() {
    // Channel orthogonality: a Sentinel's root
    // (´dec:memory:root-permanence´)
    // is a Sentinel-scoped routing destination — it is not
    // channel-specific. Registering a Sentinel once should allow
    // assessments on *any* declared channel to route through the
    // same root without error and without contaminating each
    // other. This exercises the "root serves many channels" half
    // of the invariant that complements
    // [`root_serves_many_entities_independently`].
    let mut world = scenario_with("outcome-ledger-channels", 0x0BAD_CAFE, |builder| {
        builder
            .channel("login", torrust_assayer::ChannelPolicy::default())
            .channel("api", torrust_assayer::ChannelPolicy::default())
            .channel("transaction", torrust_assayer::ChannelPolicy::default())
    })
    .expect("world should build");

    world.register_sentinel("S1").expect("register S1");

    for channel in ["login", "api", "transaction"] {
        let r = world.cycle_on(channel, "alice", LabelSpec::benign);
        assert_reckoning_well_formed(&r, channel);
    }

    assert_health_clean(&world);
}

/// The destruction holds under repetition, not just once: sixteen successive
/// register, assess, label, deregister cycles on the same Sentinel name each
/// succeed on their own terms, with the name unresolvable after every
/// deregistration. Residue that accumulated only slowly across lifetimes would
/// escape a single-cycle check while still poisoning a long-lived deployment.
///
/// (´claim:ledger:deregistering-a-sentinel-destroys-its-ledger-and-re-registering-builds-a-fresh-one´)
/// ´test:integration:root-survives-repeated-reregister-cycles´
#[test]
fn root_survives_repeated_reregister_cycles() {
    let mut s = scenario(INSTANCE, SEED);

    // The multi-cycle case: the wider property
    // (´dec:memory:root-permanence´)
    // asserts that the root is destroyed on deregistration *and* that a
    // fresh root is created on re-registration. The single-cycle
    // variant [`root_destroyed_on_deregister_then_fresh_on_reregister`]
    // pins the qualitative behaviour once; this test runs the
    // cycle 16 times on the same harness-side name to confirm no
    // state leaks forward across cycles — every re-registration
    // yields a root that accepts assessments and labels
    // independently of every prior lifetime.
    for cycle in 0..16 {
        s.register_sentinel("S1")
            .unwrap_or_else(|e| panic!("cycle {cycle}: register: {e:?}"));

        let r = s.cycle_benign("alice");
        assert_reckoning_well_formed(&r, &format!("cycle {cycle} p_bad"));

        s.deregister_sentinel("S1")
            .unwrap_or_else(|e| panic!("cycle {cycle}: deregister: {e:?}"));

        // After deregistration, the name lookup must return None —
        // the prior lifetime's slot is genuinely unreachable.
        assert_eq!(s.sentinel("S1"), None, "cycle {cycle}: name still resolves post-deregister");
    }

    assert_health_clean(&s);
}

/// Registering an outcome axis is a structural change only: the same entity's
/// core risk basis before and after the registration agrees to within the
/// documented floating-point drift band. An axis with no reported values has
/// nothing to say, so its mere existence must not move the scalars the
/// decision is actually made from — otherwise declaring an axis would silently
/// re-price every entity in the deployment.
///
/// ´claim:ledger:registering-an-outcome-axis-leaves-the-core-risk-basis-untouched´
/// ´test:integration:outcome-axis-registration-does-not-perturb-risk-basis´
#[test]
fn outcome_axis_registration_does_not_perturb_risk_basis() {
    // The resonance derivation
    // (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´)
    // is a pure function
    // of `RiskBasis = (p_bad, σ_eff, anchor_weight, …)` — outcome
    // axis *predictions* (ô_a, σ_o,a) are fed only into the
    // resonance step, never into the core assessment. Until a
    // host actually reports values for an axis, registering it is
    // structurally observable (dimension map grows; per-Sentinel
    // tag count is unaffected) but must leave the core scalars on
    // the same entity within the documented floating-point drift
    // band.
    //
    // The full scenario also asserts tag *parameters* are
    // unchanged (since no axis predictions exist yet); this slice
    // pins the load-bearing half — risk-basis invariance — which
    // catches a regression where axis registration leaks into the
    // model layer through e.g. a stale dimension-map index.
    //
    // The harness clock is fixed. This budget covers only floating-point
    // summation residue across the lifecycle rebuild and assessment paths.
    const DRIFT: f64 = 1e-9;

    let mut s = scenario(INSTANCE, SEED);
    s.register_sentinel("S1").expect("register S1");

    // Settle the cold ramp before the pair is taken. The claim is that
    // registering an axis leaves the risk basis where it was, and a ramp still
    // transitioning would move it between the two reads on its own account —
    // which would be the ramp working, not the registration leaking
    // (´inv:guarantee:evidence-authority´). Settled first, the registration is
    // the only thing that happens between them; the lifecycle rebase takes the
    // moments it publishes as the ramp's new base and rolls nothing back
    // (´req:standardisation:lifecycle-entries´).
    s.settle_cold_ramp_with(&[s.request("default", "alice")]);

    let before = s
        .derive_default("alice")
        .expect("baseline assess/derive (no axis registered)");

    s.register_axis("magnitude", false).expect("register axis");

    let after = s.derive_default("alice").expect("assess/derive after axis registration");

    assert_risk_basis_near(
        &before.assessment.risk,
        &after.assessment.risk,
        DRIFT,
        "risk basis across axis registration",
    );

    assert_health_clean(&s);
}

/// The same registration boundary leaves the emitted tag profile unchanged
/// within the drift band. The tags are the part of the output a host reads
/// downstream, so this is the visible face of the same neutrality: adding an
/// axis must not change what any consumer of the profile sees until the axis
/// has values of its own to contribute.
///
/// ´claim:ledger:registering-an-outcome-axis-leaves-the-emitted-tag-profile-untouched´
/// ´test:integration:outcome-axis-registration-does-not-perturb-tag-profile´
#[test]
fn outcome_axis_registration_does_not_perturb_tag_profile() {
    const DRIFT: f64 = 1e-9;

    let mut s = scenario(INSTANCE, SEED);
    s.register_sentinel("S1").expect("register S1");

    // Settle the cold ramp before the pair is taken. The claim is that
    // registering an axis leaves the risk basis where it was, and a ramp still
    // transitioning would move it between the two reads on its own account —
    // which would be the ramp working, not the registration leaking
    // (´inv:guarantee:evidence-authority´). Settled first, the registration is
    // the only thing that happens between them; the lifecycle rebase takes the
    // moments it publishes as the ramp's new base and rolls nothing back
    // (´req:standardisation:lifecycle-entries´).
    s.settle_cold_ramp_with(&[s.request("default", "alice")]);

    let before = s
        .derive_default("alice")
        .expect("baseline assess/derive (no axis registered)");

    s.register_axis("magnitude", false).expect("register axis");

    let after = s.derive_default("alice").expect("assess/derive after axis registration");

    assert_tag_profile_near(
        &after.profile.tags,
        &before.profile.tags,
        DRIFT,
        "tag profile across axis registration",
    );

    assert_health_clean(&s);
}

/// Outcome-axis predictions stay outside the derivation even once they are
/// live. Two worlds driven through the same twelve-cycle risk-label history
/// differ only in that one also reports outcome values; that world emits a
/// finite prediction with its uncertainty band while the control emits none,
/// and yet both end with matching risk bases and matching tag profiles.
/// Predictions are an output the host may consult, never an input that feeds
/// back into the risk the engine assigns.
///
/// ´claim:ledger:live-outcome-predictions-stay-outside-the-derivation´
/// ´test:integration:outcome-predictions-do-not-perturb-reckoning-when-core-history-matches´
#[test]
fn outcome_predictions_do_not_perturb_reckoning_when_core_history_matches() {
    // The live-prediction case
    // (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´):
    // outcome-axis predictions are
    // produced by the core assessment surface, but resonance derivation
    // must not read those prediction outputs. Drive two matched worlds
    // through the same core label history; one world also reports an
    // outcome value so its axis model learns and emits a prediction.
    // The final `RiskBasis` and tag profile must still match the
    // no-axis control within the usual floating-point drift band.
    let mut with_axis = scenario("outcome-prediction-neutrality-axis", 0x0A51_5074);
    let without_axis = scenario("outcome-prediction-neutrality-control", 0x0A51_5074);
    let axis = with_axis.register_axis("magnitude", false).expect("register axis");

    for cycle in 0..12 {
        let r_axis = with_axis
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("axis world cycle {cycle}: assess failed: {e:?}"));
        let r_control = without_axis
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("control world cycle {cycle}: assess failed: {e:?}"));

        with_axis
            .label(LabelSpec::adverse(r_axis.assessment.id).outcome(axis, 4.0).build())
            .unwrap_or_else(|e| panic!("axis world cycle {cycle}: label failed: {e:?}"));
        without_axis
            .label(LabelSpec::adverse(r_control.assessment.id).build())
            .unwrap_or_else(|e| panic!("control world cycle {cycle}: label failed: {e:?}"));

        with_axis
            .flush_labels()
            .unwrap_or_else(|e| panic!("axis world cycle {cycle}: flush failed: {e:?}"));
        without_axis
            .flush_labels()
            .unwrap_or_else(|e| panic!("control world cycle {cycle}: flush failed: {e:?}"));
    }

    let with_prediction = with_axis
        .derive_default("alice")
        .expect("axis world final reckoning succeeds");
    let without_prediction = without_axis
        .derive_default("alice")
        .expect("control world final reckoning succeeds");

    assert_reckoning_well_formed(&with_prediction, "axis world final reckoning");
    assert_reckoning_well_formed(&without_prediction, "control world final reckoning");

    let prediction = with_prediction
        .assessment
        .outcome_predictions
        .get(&axis)
        .expect("registered axis should emit a prediction");
    assert_finite(
        &[
            prediction.predicted_raw,
            prediction.uncertainty,
            prediction.prediction_interval.0,
            prediction.prediction_interval.1,
        ],
        "live outcome prediction",
    );
    assert!(
        without_prediction.assessment.outcome_predictions.is_empty(),
        "control world should have no outcome predictions",
    );

    assert_core_risk_basis_near(
        &with_prediction,
        &without_prediction,
        "RiskBasis with live outcome prediction vs no-axis control",
    );
    assert_tag_profile_near(
        &with_prediction.profile.tags,
        &without_prediction.profile.tags,
        PURITY_DRIFT,
        "tag profile with live outcome prediction vs no-axis control",
    );

    assert_health_clean(&with_axis);
    assert_health_clean(&without_axis);
}

/// The separation holds even when the predictions actively disagree: two
/// worlds fed identical adverse risk-label histories but opposite outcome
/// values learn predictions that are strictly ordered against each other,
/// while their risk bases and tag profiles remain matched. This is the
/// liveness half of the neutrality argument — the axis models demonstrably
/// moved, so the unchanged core cannot be explained away as the predictions
/// having been inert.
///
/// (´claim:ledger:live-outcome-predictions-stay-outside-the-derivation´)
/// ´test:integration:different-outcome-prediction-values-do-not-perturb-reckoning-when-core-history-matches´
#[test]
fn different_outcome_prediction_values_do_not_perturb_reckoning_when_core_history_matches() {
    // The liveness companion
    // (´claim:channel:outcome-predictions-ride-along-as-payload-and-never-enter-the-derivation´):
    // the no-axis control
    // proves outcome predictions can exist without moving resonance.
    // This companion makes the prediction outputs themselves diverge:
    // both worlds see the same entity and the same adverse risk-label
    // history, but one outcome axis learns +4.0 values while the other
    // learns -4.0 values. The core risk path is identical; only the
    // outcome-axis model output may differ.
    let mut positive_axis = scenario("outcome-prediction-positive-values", 0x0A51_7474);
    let mut negative_axis = scenario("outcome-prediction-negative-values", 0x0A51_7474);
    let positive_id = positive_axis
        .register_axis("magnitude", false)
        .expect("register positive axis");
    let negative_id = negative_axis
        .register_axis("magnitude", false)
        .expect("register negative axis");

    for cycle in 0..12 {
        let positive_r = positive_axis
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("positive axis cycle {cycle}: assess failed: {e:?}"));
        let negative_r = negative_axis
            .derive_default("alice")
            .unwrap_or_else(|e| panic!("negative axis cycle {cycle}: assess failed: {e:?}"));

        positive_axis
            .label(LabelSpec::adverse(positive_r.assessment.id).outcome(positive_id, 4.0).build())
            .unwrap_or_else(|e| panic!("positive axis cycle {cycle}: label failed: {e:?}"));
        negative_axis
            .label(
                LabelSpec::adverse(negative_r.assessment.id)
                    .outcome(negative_id, -4.0)
                    .build(),
            )
            .unwrap_or_else(|e| panic!("negative axis cycle {cycle}: label failed: {e:?}"));

        positive_axis
            .flush_labels()
            .unwrap_or_else(|e| panic!("positive axis cycle {cycle}: flush failed: {e:?}"));
        negative_axis
            .flush_labels()
            .unwrap_or_else(|e| panic!("negative axis cycle {cycle}: flush failed: {e:?}"));
    }

    let positive = positive_axis
        .derive_default("alice")
        .expect("positive-axis final reckoning succeeds");
    let negative = negative_axis
        .derive_default("alice")
        .expect("negative-axis final reckoning succeeds");

    assert_reckoning_well_formed(&positive, "positive-axis final reckoning");
    assert_reckoning_well_formed(&negative, "negative-axis final reckoning");

    let positive_prediction = positive
        .assessment
        .outcome_predictions
        .get(&positive_id)
        .expect("positive axis should emit prediction");
    let negative_prediction = negative
        .assessment
        .outcome_predictions
        .get(&negative_id)
        .expect("negative axis should emit prediction");

    assert_finite(
        &[
            positive_prediction.predicted_raw,
            positive_prediction.uncertainty,
            negative_prediction.predicted_raw,
            negative_prediction.uncertainty,
        ],
        "divergent outcome predictions",
    );
    assert!(
        positive_prediction.predicted_raw > negative_prediction.predicted_raw + 1e-6,
        "opposite outcome histories should produce ordered predictions: +={} vs -={}",
        positive_prediction.predicted_raw,
        negative_prediction.predicted_raw,
    );

    assert_core_risk_basis_near(&positive, &negative, "RiskBasis across divergent outcome predictions");
    assert_tag_profile_near(
        &positive.profile.tags,
        &negative.profile.tags,
        PURITY_DRIFT,
        "tag profile across divergent outcome predictions",
    );

    assert_health_clean(&positive_axis);
    assert_health_clean(&negative_axis);
}
