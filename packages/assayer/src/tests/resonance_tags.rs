// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`dominated_action`] | resonance | Domination can be driven from the benign side as well as the adversarial one: making the friction of a challenge prohibitively expensive squeezes Challenge's regime between its two neighbours until it either vanishes or is flagged outright. An action is worth recommending only where it beats both the gentler and the harsher option, so pricing it out of that window removes it whatever it might do to an adversary. |
//! | [`two_action_channel`] | resonance | A channel offering only permit-or-deny still gets the full seven-slot shape minus the actions it lacks — three classification tags plus its two actions — and the pair is placed on opposite sides of the single crossover, Allow to the left of Block, each with a live positive magnitude. With no interior action to take a midpoint from, both tags are boundary tags, and the half-power offset is what keeps them apart instead of stacked. |
//! | [`catching_forfeiture`] | resonance | Blocking earns its place by catching adverse sources, not merely by refusing them: set that retained catching value to zero and escalating from Slow to Block buys nothing against an adversary while costing the benign class a great deal, so Block's regime closes to zero width. A channel that throws away the identifying value of its harshest action is left with an action it should never reach for. |
//! | [`boundary_existence_failure_dominated`] | resonance | The crossover-matching formula that places a boundary tag has no solution when the boundary's own peak is too weak to meet its neighbour at the crossover — the discriminant goes negative. Rather than returning an arbitrary or out-of-range location, the derivation treats the failure as domination and pins the tag to the standard dominated values. Placement is therefore either genuinely derived or honestly declared absent, never faked. |
//! | [`kernel_guard`] | resonance | cites (´claim:resonance:the-kernel-returns-a-tiny-positive-floor-instead-of-zero-in-the-far-tail´) |
//! | [`channel_derived_debug_assert`] | resonance | Deriving the channel constants is a pure function of the policy and the challenge estimate: run twice on the same inputs, the action list, the posture-sensitivity sum, and every reward differential come back identical to the last bit. Nothing accumulates between calls, which is what lets a host recompute the constants per request instead of caching them and worrying about staleness. |
//! | [`action_ordering_preserved`] | resonance | The action tags come back in the order the policy declared its actions, least to most restrictive, so a host can zip the tag vector against its own action list positionally. The derivation reads adjacent pairs of that list to build its crossovers, and returning the results in any other order would silently mismatch each tag with the wrong action at the call site. |
//! | [`tags_have_correct_structure`] | resonance | Good, Suspicious and Malicious are always emitted, whatever action set the channel declares: an ordinary mid-risk derivation under the default policy carries all three alongside the action tags, every one of them with a finite location inside the open axis, a non-negative magnitude and a positive Q. The classification half of the field describes what the system believes about the request and does not depend on which operational responses are on offer. This is the lighter-weight companion to the golden-value tests in [`resonance_golden`](super::resonance_golden), covering the integration seam between `derive_resonance`, policy defaults, and tag generation. |

//! Tag structure, ordering, and kernel-shape tests (´app:spec:resonance-rendering´).
//!
//! Acceptance tests that verify the *shape* of the resonance output —
//! which tags are present, how they are ordered, and whether dominated
//! tags are marked correctly. These checks are
//! independent of golden values (see [`resonance_golden`](super::resonance_golden)).

use super::helpers::{KernelScalars, derive_tags};
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters, compute_derived_constants};
use crate::resonance::derivation::ResonanceConfig;
use crate::resonance::tags::{Tag, cauchy_kernel};
use crate::testing::{assert_below, assert_finite, assert_near};

// ═══════════════════════════════════════════════════════════════════════════════
// Tag Properties
// ═══════════════════════════════════════════════════════════════════════════════

/// Domination can be driven from the benign side as well as the adversarial
/// one: making the friction of a challenge prohibitively expensive squeezes
/// Challenge's regime between its two neighbours until it either vanishes or is
/// flagged outright. An action is worth recommending only where it beats both
/// the gentler and the harsher option, so pricing it out of that window removes
/// it whatever it might do to an adversary.
///
/// ´claim:resonance:an-action-whose-cost-is-prohibitive-loses-its-regime-to-its-neighbours´
/// ´test:crate:dominated-action´
#[test]
fn dominated_action() {
    // When reward structure makes an action never optimal, it should be dominated.
    // Use extreme rewards where Challenge is always worse than both neighbours.
    let input = KernelScalars {
        p_hat: 0.5,
        sigma_eff: 1.0,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    // Make Challenge friction prohibitively expensive
    let reward = RewardParameters {
        friction: 1000.0,
        ..RewardParameters::default()
    };
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
        reward,
        ..ChannelPolicy::default()
    };
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    // Challenge should have very small magnitude or be dominated
    let challenge = tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    // The enormous friction cost will shrink Challenge's regime to near-zero
    assert!(
        challenge.magnitude < 0.05 || challenge.dominated,
        "Challenge should be dominated or near-zero, got magnitude {}",
        challenge.magnitude
    );
}

/// A channel offering only permit-or-deny still gets the full seven-slot shape
/// minus the actions it lacks — three classification tags plus its two actions —
/// and the pair is placed on opposite sides of the single crossover, Allow to
/// the left of Block, each with a live positive magnitude. With no interior
/// action to take a midpoint from, both tags are boundary tags, and the
/// half-power offset is what keeps them apart instead of stacked.
///
/// ´claim:resonance:a-two-action-channel-places-its-pair-either-side-of-the-crossover´
/// ´test:crate:two-action-channel´
#[test]
fn two_action_channel() {
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    assert_eq!(tags.len(), 5, "3 classification + 2 action tags");

    let allow = tags.iter().find(|t| t.tag == Tag::Allow).unwrap();
    let block = tags.iter().find(|t| t.tag == Tag::Block).unwrap();

    // Allow should be on the left, Block on the right
    assert!(
        allow.location < block.location,
        "Allow ({}) should be left of Block ({})",
        allow.location,
        block.location
    );

    // Both should have positive magnitude
    assert!(allow.magnitude > 0.0, "Allow magnitude should be positive");
    assert!(block.magnitude > 0.0, "Block magnitude should be positive");
}

/// Blocking earns its place by catching adverse sources, not merely by refusing
/// them: set that retained catching value to zero and escalating from Slow to
/// Block buys nothing against an adversary while costing the benign class a
/// great deal, so Block's regime closes to zero width. A channel that throws
/// away the identifying value of its harshest action is left with an action it
/// should never reach for.
///
/// ´claim:resonance:forfeiting-the-catch-value-of-blocking-leaves-blocking-with-no-regime´
/// ´test:crate:catching-forfeiture´
#[test]
fn catching_forfeiture() {
    // β_c = 0 means Block offers no catching value → Block should be
    // dominated (Δ_bad for S→B ≤ 0 when q̂_c = 0.6).
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };

    // Forfeited: β_c = 0
    let reward = RewardParameters {
        block_catches: 0.0,
        ..RewardParameters::default()
    };
    let forfeited_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
        reward,
        ..ChannelPolicy::default()
    };
    let config = ResonanceConfig::default();
    let forfeited_tags = derive_tags(&input, 0.6, &forfeited_policy, &config);
    let forfeited_block = forfeited_tags.iter().find(|t| t.tag == Tag::Block).unwrap();

    // With β_c = 0, Δ_bad(S→B) < 0 so Block's regime is zero-width.
    // Block should be dominated or have near-zero magnitude.
    if !forfeited_block.dominated {
        assert_below(
            forfeited_block.magnitude,
            1e-4,
            "Block magnitude when β_c=0, q̂_c=0.6 (not dominated)",
        );
    }
}

/// The crossover-matching formula that places a boundary tag has no solution
/// when the boundary's own peak is too weak to meet its neighbour at the
/// crossover — the discriminant goes negative. Rather than returning an
/// arbitrary or out-of-range location, the derivation treats the failure as
/// domination and pins the tag to the standard dominated values. Placement is
/// therefore either genuinely derived or honestly declared absent, never faked.
///
/// ´claim:resonance:a-boundary-placement-that-cannot-exist-is-reported-as-domination-rather-than-a-fabricated-location´
/// ´test:crate:boundary-existence-failure-dominated´
#[test]
fn boundary_existence_failure_dominated() {
    // Force ρ < 0 in the boundary_location formula. With very large σ_eff,
    // Q-bandwidths hit the Q_floor, making Q²·w² small. Combined with
    // low p̂ pushing the Block boundary regime narrower than Slow's interior
    // regime, ρ = (A_block/A_slow)·(1 + Q²·w²) - 1 goes negative.
    let input = KernelScalars {
        p_hat: 0.01,
        sigma_eff: 100.0,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    let block = tags.iter().find(|t| t.tag == Tag::Block).unwrap();
    assert!(block.dominated, "Block should be dominated when ρ < 0 (p̂=0.01, σ=100)");
    assert_near(block.magnitude, config.epsilon_mono, 1e-12, "ρ < 0 dominated magnitude");
    assert_near(block.q, config.q_min, 1e-12, "ρ < 0 dominated Q");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Kernel Numerical Guard
// ═══════════════════════════════════════════════════════════════════════════════

/// The numerical guard is reachable from the tag layer with realistic-looking
/// arguments, not merely a defensive branch: a sharp bandwidth evaluated far
/// from its location returns a value below 10⁻¹⁰⁰ instead of underflowing to
/// nothing. A dominated tag sampled at the far end of the axis therefore still
/// contributes something a normalisation can divide by.
///
/// (´claim:resonance:the-kernel-returns-a-tiny-positive-floor-instead-of-zero-in-the-far-tail´)
/// ´test:crate:kernel-guard´
#[test]
fn kernel_guard() {
    let q = 1e8;
    let d = 1e4;
    let k = cauchy_kernel(1.0, q, d, 0.0);
    assert_below(k, 1e-100, "kernel guard: Q²d² > 10¹⁵ should return ε");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Consistency
// ═══════════════════════════════════════════════════════════════════════════════

/// Deriving the channel constants is a pure function of the policy and the
/// challenge estimate: run twice on the same inputs, the action list, the
/// posture-sensitivity sum, and every reward differential come back identical to
/// the last bit. Nothing accumulates between calls, which is what lets a host
/// recompute the constants per request instead of caching them and worrying
/// about staleness.
///
/// ´claim:resonance:deriving-the-channel-constants-twice-from-one-policy-gives-the-same-numbers´
/// ´test:crate:channel-derived-debug-assert´
#[test]
fn channel_derived_debug_assert() {
    // Re-computation from policy matches stored constants.
    let policy = ChannelPolicy::default();
    let d1 = compute_derived_constants(&policy, 0.5);
    let d2 = compute_derived_constants(&policy, 0.5);

    assert_eq!(d1.actions, d2.actions);
    assert_near(d1.beta_sum, d2.beta_sum, f64::EPSILON, "beta_sum recomputation");
    for (a, b) in d1.differentials.iter().zip(d2.differentials.iter()) {
        assert_near(a.delta_good, b.delta_good, f64::EPSILON, "delta_good recomputation");
        assert_near(a.delta_bad, b.delta_bad, f64::EPSILON, "delta_bad recomputation");
    }
}

/// The action tags come back in the order the policy declared its actions,
/// least to most restrictive, so a host can zip the tag vector against its own
/// action list positionally. The derivation reads adjacent pairs of that list to
/// build its crossovers, and returning the results in any other order would
/// silently mismatch each tag with the wrong action at the call site.
///
/// ´claim:resonance:action-tags-are-emitted-in-the-policys-own-order-of-severity´
/// ´test:crate:action-ordering-preserved´
#[test]
fn action_ordering_preserved() {
    // Action tags should appear in the same order as the policy's action set.
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    let action_tags: Vec<Tag> = tags.iter().filter(|t| !t.is_classification()).map(|t| t.tag).collect();
    assert_eq!(
        action_tags,
        vec![Tag::Allow, Tag::Challenge, Tag::Slow, Tag::Block],
        "action tags should preserve policy order"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Structural integration — default policy end-to-end
// ═══════════════════════════════════════════════════════════════════════════════

/// Good, Suspicious and Malicious are always emitted, whatever action set the
/// channel declares: an ordinary mid-risk derivation under the default policy
/// carries all three alongside the action tags, every one of them with a finite
/// location inside the open axis, a non-negative magnitude and a positive Q. The
/// classification half of the field describes what the system believes about the
/// request and does not depend on which operational responses are on offer.
///
/// This is the lighter-weight companion to the golden-value tests in
/// [`resonance_golden`](super::resonance_golden), covering the integration seam
/// between `derive_resonance`, policy defaults, and tag generation.
///
/// ´claim:resonance:every-derivation-emits-the-three-classification-tags-alongside-the-action-tags´
/// ´test:crate:tags-have-correct-structure´
#[test]
fn tags_have_correct_structure() {
    let input = KernelScalars {
        p_hat: 0.35,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.04,
    };

    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();

    let tags = derive_tags(&input, 0.5, &policy, &config);

    assert!(!tags.is_empty(), "Should generate resonance tags");

    // All numerical fields should be finite (no NaN/inf leak).
    let locations: Vec<f64> = tags.iter().map(|t| t.location).collect();
    let magnitudes: Vec<f64> = tags.iter().map(|t| t.magnitude).collect();
    let qs: Vec<f64> = tags.iter().map(|t| t.q).collect();
    assert_finite(&locations, "tag locations");
    assert_finite(&magnitudes, "tag magnitudes");
    assert_finite(&qs, "tag Q values");

    // Classification tags should be present
    assert!(
        tags.iter().any(|t| matches!(t.tag, Tag::Good)),
        "Should have Good classification tag"
    );
    assert!(
        tags.iter().any(|t| matches!(t.tag, Tag::Suspicious)),
        "Should have Suspicious classification tag"
    );
    assert!(
        tags.iter().any(|t| matches!(t.tag, Tag::Malicious)),
        "Should have Malicious classification tag"
    );

    // All tags should have valid locations in (0, 1)
    for tag in &tags {
        assert!(
            tag.location > 0.0 && tag.location < 1.0,
            "Tag {:?} has invalid location: {}",
            tag.tag,
            tag.location
        );
        assert!(
            tag.magnitude >= 0.0,
            "Tag {:?} has negative magnitude: {}",
            tag.tag,
            tag.magnitude
        );
        assert!(tag.q > 0.0, "Tag {:?} has non-positive Q: {}", tag.tag, tag.q);
    }
}
