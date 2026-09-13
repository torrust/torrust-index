// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`tag_parameters_are_well_formed_on_cold_world`] | resonance | Whatever the derivation does internally, what reaches a host through the public reckoning is always usable: at least one tag, each with finite fields, a location strictly inside the open posture axis, a non-negative magnitude, and a strictly positive Q even when the tag has been ruled out. Q is the denominator the Cauchy kernel divides by, so a zero would take the whole field down; dominated tags are held at a tiny magnitude rather than at zero Q. |
//! | [`tag_parameters_entity_independent_on_cold_world`] | resonance | On a world with no labels, no Sentinels and no registered identity dimensions, two different entities receive the same number of tags of the same kinds at the same locations, magnitudes and bandwidths, to within the documented floating-point drift budget. There is nothing yet to tell them apart, and the engine invents nothing: a name alone never moves the geometry. |
//! | [`kernel_strictly_positive_at_every_posture`] | resonance | cites (´claim:resonance:every-tags-kernel-stays-strictly-positive-out-into-the-tails´) |
//! | [`three_action_challenge_at_least_as_strong_as_four_action`] | resonance | cites (´claim:resonance:removing-an-action-from-the-channel-widens-the-regime-of-the-neighbour-it-crowded´) |
//! | [`classification_tags_all_present_and_live_on_cold_start`] | resonance | A cold start is the regime where every classification should still be worth hearing, and the reckoning delivers that: exactly three classification tags, one of each kind, none flagged dominated, each with a finite strictly positive magnitude. Nothing has crowded the others out, which is the structural precondition for the near-maximal classification entropy the scenario expects of an entity the system knows nothing about. |
//! | [`classification_magnitudes_symmetric_on_cold_world`] | resonance | A freshly-built world puts the risk estimate at the symmetric fixed point, and there the reflection between Good and Malicious becomes an equality: their locations sum to one and their magnitudes agree to within the same floating-point drift budget as the entity-independence check. Knowing nothing, the system leans neither way — the two readings weigh exactly the same rather than one of them starting out marginally favoured. |
//! | [`tag_profile_unchanged_across_posture_samples`] | resonance | Posture is a question asked of the field, not an input that reshapes it: evaluating every tag's kernel at seven postures spanning the logit axis leaves every location, magnitude, bandwidth and dominated flag bit-identical to the snapshot taken beforehand, while the kernel values themselves genuinely differ across those postures. The placement is fixed once at assessment time, so one reckoning can be re-read at many postures and still mean the same thing. |
//! | [`channel_policy_validation_reachable_from_host`] | resonance | The well-formedness check the derivation record assigns to the host at policy load is reachable from outside the crate: the crate root exports it beside the policy type, a default declaration passes, and a single-action or unordered declaration is refused by the name of its defect. Without the export no host-side registry could run the check the record requires, so a caller's defect would first surface downstream of the load instead of at it. |

//! Integration tests for the geometry of the derived tag field
//! (´alg:rendering:placement´).
//!
//! These scenarios probe invariants of the tag field produced by
//! `assess()` that are observable against the public `Reckoning`
//! surface: tag shape well-formedness, kernel tail positivity, and
//! the channel-set-dependent strength of the Challenge action tag.
//!
//! They are all clock-independent: every scenario issues assessments
//! against a freshly-built cold [`World`] and asserts geometric
//! properties of the returned `TagResonance` vector.
//!
//! # Cross-References
//!
//! - (´inv:guarantee:derivation-purity´) — the purity and correctness of the derivation
//! - (´prop:rendering:completeness´) — every rendered tag is present at every posture
//! - (´def:rendering:bandwidths´) — the kernel bandwidths and the display floor
//! - (´def:rendering:magnitudes´) — tag magnitudes, relevance and attenuation
//! - (´alg:rendering:placement´) — action-tag placement and regime widths
//! - (´dec:derivation:pure-transform´) — the derivation as a pure transform

use torrust_assayer::numerics_export::stable_logit;
use torrust_assayer::testing::{Scenario, assert_finite, assert_health_clean, scenario, scenario_with};
use torrust_assayer::types::Action;
use torrust_assayer::{ChannelPolicy, DerivedReckoning, Tag};

/// Build a cold [`Scenario`] with a single 4-action `"default"` channel.
fn build_world() -> Scenario {
    scenario("resonance-geometry", 0x600D_F00D)
}

/// Floating-point drift budget for fingerprint equality on a cold world.
/// The harness clock is fixed; the allowance covers only summation residue
/// along the assessment and rendering paths.
const GEOMETRY_DRIFT: f64 = 1e-9;

/// Whatever the derivation does internally, what reaches a host through the
/// public reckoning is always usable: at least one tag, each with finite fields,
/// a location strictly inside the open posture axis, a non-negative magnitude,
/// and a strictly positive Q even when the tag has been ruled out. Q is the
/// denominator the Cauchy kernel divides by, so a zero would take the whole
/// field down; dominated tags are held at a tiny magnitude rather than at zero Q.
///
/// ´claim:resonance:every-tag-reaching-the-host-has-a-location-inside-the-axis-a-non-negative-magnitude-and-a-positive-q´
/// ´test:integration:tag-parameters-are-well-formed-on-cold-world´
#[test]
fn tag_parameters_are_well_formed_on_cold_world() {
    let world = build_world();
    let r = world
        .derive_for_request(world.request("default", "alice"))
        .expect("cold-start assess should succeed");

    assert!(!r.profile.tags.is_empty(), "reckoning should carry at least one tag");

    for tag in &r.profile.tags {
        let label = format!("{:?}", tag.tag);
        assert_finite(&[tag.location, tag.magnitude, tag.q], &label);

        // Location lives on the open posture axis
        // (´alg:rendering:placement´).
        assert!(
            tag.location > 0.0 && tag.location < 1.0,
            "{label}: location {} must lie in (0, 1)",
            tag.location,
        );

        // Magnitude is non-negative; exact zero is reserved for the
        // dominated-action monotonicity override
        // (´alg:rendering:dominated-treatment´)
        // (`ε_mono = 1e-6`), but the shipped implementation returns a
        // tiny positive floor rather than exactly 0.
        assert!(
            tag.magnitude >= 0.0,
            "{label}: magnitude {} must be non-negative",
            tag.magnitude,
        );

        // Q-bandwidth is strictly positive for live tags; dominated
        // tags may report `Q_min` but never zero (avoids divide-by-
        // zero in the Cauchy kernel).
        assert!(tag.q > 0.0, "{label}: Q {} must be strictly positive", tag.q);

        if tag.dominated {
            // A dominated tag has magnitude pinned at `ε_mono`; assert
            // it is numerically tiny but never negative or NaN.
            assert!(
                tag.magnitude <= 1e-3,
                "{label}: dominated tag should have ε_mono-scale magnitude, got {}",
                tag.magnitude,
            );
        }
    }

    assert_health_clean(&world);
}

/// On a world with no labels, no Sentinels and no registered identity
/// dimensions, two different entities receive the same number of tags of the
/// same kinds at the same locations, magnitudes and bandwidths, to within the
/// documented floating-point drift budget. There is nothing yet to tell them
/// apart, and the engine invents nothing: a name alone never moves the geometry.
///
/// ´claim:resonance:on-a-cold-world-tag-geometry-is-the-same-for-every-entity´
/// ´test:integration:tag-parameters-entity-independent-on-cold-world´
#[test]
fn tag_parameters_entity_independent_on_cold_world() {
    let world = build_world();

    // The two assessments must be taken in one coordinate state to be
    // comparable at all: on a cold world the standardisation ramp is running,
    // and the second request would otherwise be answered one accepted
    // observation later than the first (´inv:guarantee:evidence-authority´).
    // Settling makes the coordinate system the constant this scenario needs it
    // to be, leaving the entity as the only thing that differs.
    world.settle_cold_ramp_with(&[world.request("default", "alice")]);

    let alice = world
        .derive_for_request(world.request("default", "alice"))
        .expect("alice assess");
    let bob = world.derive_for_request(world.request("default", "bob")).expect("bob assess");

    // Cold world: no labels have shaped the model, no Sentinels
    // are reporting, no identity dimension is registered. The
    // feature vector φ for both entities is dominated by the bias
    // and the zero-filled aggregates — tag parameters (a deterministic
    // function of the `RiskAssessment` scalars) must therefore coincide
    // across the two entities within the documented purity drift
    // budget.
    assert_eq!(
        alice.profile.tags.len(),
        bob.profile.tags.len(),
        "tag count must be entity-independent on a cold world",
    );

    for (a, b) in alice.profile.tags.iter().zip(bob.profile.tags.iter()) {
        assert_eq!(a.tag, b.tag, "tag kind sequence must match");
        let diff_loc = (a.location - b.location).abs();
        let diff_mag = (a.magnitude - b.magnitude).abs();
        let diff_q = (a.q - b.q).abs();
        assert!(
            diff_loc <= GEOMETRY_DRIFT && diff_mag <= GEOMETRY_DRIFT && diff_q <= GEOMETRY_DRIFT,
            "{:?}: (Δloc, Δmag, Δq) = ({diff_loc}, {diff_mag}, {diff_q}) exceeds {GEOMETRY_DRIFT}",
            a.tag,
        );
    }
}

/// Tail positivity survives the round trip through the public surface: sweeping
/// nine postures from 0.001 to 0.999 across both wings of the logit axis, every
/// tag of a real cold-start reckoning returns a finite, strictly positive
/// kernel. A host is free to evaluate the field at any posture it likes without
/// first checking whether that posture happens to annihilate one of the tags.
///
/// (´claim:resonance:every-tags-kernel-stays-strictly-positive-out-into-the-tails´)
/// ´test:integration:kernel-strictly-positive-at-every-posture´
#[test]
fn kernel_strictly_positive_at_every_posture() {
    let world = build_world();
    let r = world
        .derive_for_request(world.request("default", "charlie"))
        .expect("assess should succeed");

    // The Cauchy tail is guaranteed strictly positive
    // (´prop:rendering:completeness´);
    // when `Q²d² > 10¹⁵` the engine returns `ε = 1e-300` rather than
    // zero to preserve normalisability even for dominated tags. Sweep
    // a representative set of postures covering both wings of the
    // logit axis and assert the kernel stays strictly above zero for
    // every tag at every posture.
    let postures = [0.001_f64, 0.01, 0.1, 0.3, 0.5, 0.7, 0.9, 0.99, 0.999];

    for &pi in &postures {
        let logit_pi = stable_logit(pi);
        for tag in &r.profile.tags {
            let k = tag.kernel_at_logit(logit_pi);
            assert!(k.is_finite(), "{:?} at π={pi}: kernel {k} is not finite", tag.tag);
            assert!(k > 0.0, "{:?} at π={pi}: kernel {k} violated tail-positivity", tag.tag);
        }
    }
}

/// The same entity assessed through two channels on one engine, differing only
/// in whether Slow is on the menu, gets a Challenge tag at least as strong on
/// the shorter channel — both live, neither dominated. Registering an extra
/// intermediate action can only take posture band away from the actions
/// adjacent to it, so a host adding Slow to a channel should expect Challenge to
/// be recommended less often, never more.
///
/// (´claim:resonance:removing-an-action-from-the-channel-widens-the-regime-of-the-neighbour-it-crowded´)
/// ´test:integration:three-action-challenge-at-least-as-strong-as-four-action´
#[test]
fn three_action_challenge_at_least_as_strong_as_four_action() {
    // Two channels on the same engine, same reward parameters —
    // only the action set differs. Dropping `Slow` widens the Challenge regime
    // (´prop:channel:challenge-width´): there is no adjacent action
    // crowding Challenge from above, so its Cauchy peak is free to
    // grow. At the magnitude level, that translates to the 3-action
    // Challenge having a peak magnitude at least as large as the
    // 4-action Challenge.
    let default_reward = ChannelPolicy::default().reward;
    let three = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: default_reward.clone(),
        ..ChannelPolicy::default()
    };
    let four = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
        reward: default_reward,
        ..ChannelPolicy::default()
    };

    let world = scenario_with("resonance-geometry-3v4", 0xBEEF_CAFE, |builder| {
        builder.channel("three", three).channel("four", four)
    })
    .expect("world should build");

    let r3 = world
        .derive_for_request(world.request("three", "alice"))
        .expect("three-action assess");
    let r4 = world
        .derive_for_request(world.request("four", "alice"))
        .expect("four-action assess");

    let challenge = |r: &DerivedReckoning| {
        r.profile
            .tags
            .iter()
            .find(|t| t.tag == Tag::Challenge)
            .copied()
            .expect("every channel carrying Challenge in its action set emits a Challenge tag")
    };

    let c3 = challenge(&r3);
    let c4 = challenge(&r4);

    assert_finite(&[c3.magnitude, c4.magnitude], "challenge magnitudes");

    // Both tags must be live (neither dominated) — at default
    // rewards and cold-start $\hat{q}_c$, Challenge is well inside
    // its live regime on both channels.
    assert!(!c3.dominated, "3-action Challenge should be live on cold start");
    assert!(!c4.dominated, "4-action Challenge should be live on cold start");

    assert!(
        c3.magnitude >= c4.magnitude,
        "3-action Challenge magnitude ({}) should be ≥ 4-action ({}) — dropping Slow cannot weaken Challenge",
        c3.magnitude,
        c4.magnitude,
    );

    assert_health_clean(&world);
}

/// A cold start is the regime where every classification should still be worth
/// hearing, and the reckoning delivers that: exactly three classification tags,
/// one of each kind, none flagged dominated, each with a finite strictly
/// positive magnitude. Nothing has crowded the others out, which is the
/// structural precondition for the near-maximal classification entropy the
/// scenario expects of an entity the system knows nothing about.
///
/// ´claim:resonance:the-three-classifications-arrive-live-and-informative-when-the-world-is-maximally-ambiguous´
/// ´test:integration:classification-tags-all-present-and-live-on-cold-start´
#[test]
fn classification_tags_all_present_and_live_on_cold_start() {
    // On a cold world $\hat{p}$ sits near 0.5
    // with maximal uncertainty, which is the regime where every
    // classification tag should be informative — not one dominating
    // and crowding the others out
    // (´prop:rendering:completeness´). The wider property asserts
    // quantitative magnitude symmetry and a classification entropy
    // approaching $\ln 3 \approx 1.1$ nats; this slice pins the
    // *structural* precondition that entropy assertion rests on:
    // all three classification kinds are emitted, none is flagged
    // dominated, and each has a strictly-positive finite magnitude.
    let world = build_world();
    let r = world
        .derive_for_request(world.request("default", "charlie"))
        .expect("cold-start assess should succeed");

    // The three classification tag kinds are emitted exactly once each,
    // and the set-equality check is stronger than a simple count.
    let classification: Vec<_> = r.profile.tags.iter().filter(|t| t.tag.is_classification()).collect();
    assert_eq!(
        classification.len(),
        3,
        "expected exactly 3 classification tags, got {} (kinds: {:?})",
        classification.len(),
        classification.iter().map(|t| t.tag).collect::<Vec<_>>(),
    );

    for kind in [Tag::Good, Tag::Suspicious, Tag::Malicious] {
        let t = classification
            .iter()
            .find(|t| t.tag == kind)
            .unwrap_or_else(|| panic!("classification tag {kind:?} missing from cold-start reckoning"));
        let label = format!("{kind:?}");
        assert_finite(&[t.magnitude, t.q, t.location], &label);
        assert!(
            !t.dominated,
            "{label}: classification tags should be live on a cold, maximally-ambiguous world",
        );
        assert!(
            t.magnitude > 0.0,
            "{label}: classification magnitude must be strictly positive, got {}",
            t.magnitude,
        );
    }

    assert_health_clean(&world);
}

/// A freshly-built world puts the risk estimate at the symmetric fixed point,
/// and there the reflection between Good and Malicious becomes an equality:
/// their locations sum to one and their magnitudes agree to within the same
/// floating-point drift budget as the entity-independence check. Knowing
/// nothing, the system leans neither way — the two readings weigh exactly the
/// same rather than one of them starting out marginally favoured.
///
/// ´claim:resonance:a-cold-world-sits-at-the-symmetric-fixed-point-where-good-and-malicious-carry-equal-weight´
/// ´test:integration:classification-magnitudes-symmetric-on-cold-world´
#[test]
fn classification_magnitudes_symmetric_on_cold_world() {
    // The symmetry half
    // (´def:rendering:magnitudes´):
    // at $\hat{p} = 0.5$ the
    // `Good` and `Malicious` classification tags should be mirror
    // images around the posture axis — equal magnitude and
    // locations symmetric about $\pi = 0.5$ — because the
    // underlying log-odds is symmetric under `p ↔ 1 - p` at the
    // fixed point. The full scenario also asserts Suspicious sits
    // at its peak magnitude; here we pin only the symmetry
    // invariant that rests on the cold-start $\hat{p} \approx 0.5$
    // and is reachable through the public `TagResonance` surface.
    let world = build_world();
    let r = world
        .derive_for_request(world.request("default", "delta"))
        .expect("cold-start assess should succeed");

    let pick = |kind: Tag| {
        r.profile
            .tags
            .iter()
            .find(|t| t.tag == kind)
            .copied()
            .unwrap_or_else(|| panic!("classification tag {kind:?} must be emitted on a cold world"))
    };

    let good = pick(Tag::Good);
    let malicious = pick(Tag::Malicious);

    assert_finite(
        &[good.magnitude, good.location, malicious.magnitude, malicious.location],
        "Good / Malicious fields",
    );

    // Magnitudes coincide within the same drift budget used by the
    // entity-independence test — the two kernels are reflections
    // of each other around the symmetric fixed point.
    let dmag = (good.magnitude - malicious.magnitude).abs();
    assert!(
        dmag <= GEOMETRY_DRIFT,
        "cold-start |Δmagnitude| Good vs Malicious = {dmag} exceeds {GEOMETRY_DRIFT}",
    );

    // Location symmetry: `good.location + malicious.location ≈ 1.0`.
    let sum = good.location + malicious.location;
    assert!(
        (sum - 1.0).abs() <= GEOMETRY_DRIFT,
        "cold-start Good.location + Malicious.location = {sum}, expected ≈ 1.0 (Δ > {GEOMETRY_DRIFT})",
    );

    assert_health_clean(&world);
}

/// Posture is a question asked of the field, not an input that reshapes it:
/// evaluating every tag's kernel at seven postures spanning the logit axis
/// leaves every location, magnitude, bandwidth and dominated flag bit-identical
/// to the snapshot taken beforehand, while the kernel values themselves genuinely
/// differ across those postures. The placement is fixed once at assessment time,
/// so one reckoning can be re-read at many postures and still mean the same thing.
///
/// ´claim:resonance:posture-selects-a-probability-from-the-tag-field-without-altering-the-field´
/// ´test:integration:tag-profile-unchanged-across-posture-samples´
#[test]
fn tag_profile_unchanged_across_posture_samples() {
    // The posture does not touch the profile
    // (´inv:guarantee:posture-independence´) — the
    // claim is that posture (the observation's position on the risk
    // axis at decision time) is an *input* to the action-probability
    // computation, but not to tag placement: the `(location,
    // magnitude, q)` triple on every `TagResonance` is a function of
    // the assessment-time `RiskAssessment` only. Only the probabilities
    // `P(a | π)` — obtained by evaluating the Cauchy kernel at π —
    // depend on the posture.
    //
    // The scenario is structurally guaranteed by `TagResonance`
    // being a plain value type whose fields are populated once at
    // `assess()` time and are never mutated thereafter: every
    // `kernel_at_logit` call takes `&self`. This test pins the
    // invariant in two ways:
    //
    // 1. Snapshot every tag's `(location, magnitude, q, dominated)`
    //    before any kernel evaluation. Evaluate the kernel at 7
    //    postures covering the full logit axis. Re-read the fields
    //    afterward and assert bit-identical equality — posture
    //    sampling cannot write through an `&self` receiver.
    // 2. Confirm the kernel outputs actually differ across the 7
    //    postures for at least one tag (so the test is a live probe
    //    rather than a vacuous no-op). Cauchy kernels concentrated
    //    at different locations *must* produce different values when
    //    sampled at the same π.
    let world = build_world();
    let r = world
        .derive_for_request(world.request("default", "posture-probe"))
        .expect("cold-start assess should succeed");

    assert!(!r.profile.tags.is_empty(), "at least one tag must be emitted");

    // Snapshot the profile of every tag before any kernel evaluation.
    let before: Vec<(Tag, f64, f64, f64, bool)> = r
        .profile
        .tags
        .iter()
        .map(|t| (t.tag, t.location, t.magnitude, t.q, t.dominated))
        .collect();

    let postures = [0.01_f64, 0.1, 0.3, 0.5, 0.7, 0.9, 0.99];
    let mut kernel_samples: Vec<Vec<f64>> = vec![Vec::with_capacity(postures.len()); r.profile.tags.len()];

    for &pi in &postures {
        let logit_pi = stable_logit(pi);
        for (i, tag) in r.profile.tags.iter().enumerate() {
            let k = tag.kernel_at_logit(logit_pi);
            assert!(
                k.is_finite() && k > 0.0,
                "{:?} at π={pi}: kernel must be finite and positive",
                tag.tag
            );
            kernel_samples[i].push(k);
        }
    }

    // Re-read the profile after every posture sample. Bit-identical
    // equality is the correct bar — `kernel_at_logit` takes `&self`
    // and cannot mutate.
    for (snapshot, tag) in before.iter().zip(r.profile.tags.iter()) {
        let (kind, loc, mag, q, dom) = *snapshot;
        assert_eq!(tag.tag, kind);
        assert_eq!(
            tag.location.to_bits(),
            loc.to_bits(),
            "{kind:?}: location drifted under posture sampling"
        );
        assert_eq!(
            tag.magnitude.to_bits(),
            mag.to_bits(),
            "{kind:?}: magnitude drifted under posture sampling"
        );
        assert_eq!(tag.q.to_bits(), q.to_bits(), "{kind:?}: q drifted under posture sampling");
        assert_eq!(tag.dominated, dom, "{kind:?}: dominated flag drifted under posture sampling");
    }

    // Liveness: at least one tag's kernel values must *actually
    // differ* across the posture sweep — otherwise the profile-
    // invariance assertion is vacuously true.
    let any_tag_varies = kernel_samples.iter().any(|samples| {
        let first = samples[0];
        samples.iter().any(|&v| (v - first).abs() > 1e-12)
    });
    assert!(
        any_tag_varies,
        "no tag's kernel changed across 7 postures — the posture-dependence probe is degenerate",
    );

    assert_health_clean(&world);
}

/// The well-formedness check the derivation record assigns to the host at
/// policy load is reachable from outside the crate: the crate root exports
/// it beside the policy type, a default declaration passes, and a
/// single-action or unordered declaration is refused by the name of its
/// defect. Without the export no host-side registry could run the check the
/// record requires, so a caller's defect would first surface downstream of
/// the load instead of at it.
///
/// ´claim:resonance:the-load-time-well-formedness-check-is-reachable-by-a-host´
/// ´test:integration:channel-policy-validation-reachable-from-host´
#[test]
fn channel_policy_validation_reachable_from_host() {
    use torrust_assayer::error::ChannelError;
    use torrust_assayer::validate_channel_policy;

    assert!(validate_channel_policy(&ChannelPolicy::default()).is_ok());

    let single_action = ChannelPolicy {
        actions: vec![Action::Allow],
        ..ChannelPolicy::default()
    };
    assert!(matches!(
        validate_channel_policy(&single_action),
        Err(ChannelError::TooFewActions)
    ));

    let unordered = ChannelPolicy {
        actions: vec![Action::Block, Action::Allow],
        ..ChannelPolicy::default()
    };
    assert!(matches!(
        validate_channel_policy(&unordered),
        Err(ChannelError::ActionsNotOrdered)
    ));
}
