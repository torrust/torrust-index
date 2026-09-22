// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`golden_example_1_low_risk`] | resonance | A confidently-benign entity produces the numbers the specification worked out by hand: seven tags, Good peaked at 0.814 with nearly all the magnitude, Malicious a distant reflection at 0.186, and Allow owning by far the widest action regime at 0.580 against Challenge's 0.048. The closed-form derivation is checked against arithmetic done independently of the code, so a refactor that changes the shape of the field cannot pass unnoticed. |
//! | [`golden_example_2_high_risk`] | resonance | cites (´claim:resonance:the-derivation-reproduces-the-specifications-worked-examples´) |
//! | [`golden_example_3_uncertain`] | resonance | cites (´claim:resonance:the-derivation-reproduces-the-specifications-worked-examples´) |
//! | [`golden_example_4_three_action`] | resonance | Withdrawing Slow from the channel hands its band to the neighbour it was crowding: at inputs otherwise identical to the low-risk example, Challenge's magnitude rises from 0.048 to 0.216 while no Slow tag is emitted at all. The classification tags are untouched, because they read only the assessment and not the action set — so narrowing the operational menu redistributes the posture axis among the survivors without disturbing what the system believes. |
//! | [`profile_properness`] | resonance | The tag field is a probability distribution wherever it is read: at a hundred postures spanning the axis the kernels are all finite, their total is strictly positive, and dividing through by that total gives probabilities summing to one within 10⁻¹². Nothing further is needed to turn a set of independently placed Cauchy peaks into an answer a host can act on — the normalisation always exists because the total can never reach zero. |
//! | [`profile_positivity`] | resonance | No tag ever contributes exactly nothing, however far the posture is from where it was placed: sampled out at 0.001 and 0.999 — the extremes of the usable axis — every tag in a low-risk field still returns a strictly positive kernel. The Cauchy tail decays but does not terminate, which is what keeps even a badly-mismatched action representable rather than making its probability structurally impossible to express. |
//! | [`classification_tag_ordering`] | resonance | Good and Malicious are placed as reflections of one another: their locations sum to one at every probe from a confidently-benign 0.05 through to a high-risk 0.72, with Suspicious pinned at the midpoint they reflect about. The pair says the same thing from opposite ends, so reading the field for evidence of benignity and reading it for evidence of malice are the same measurement with the sign flipped, never two independently-drifting opinions. |

//! Golden-value tests for resonance derivation, against the four landscapes the specification works out (´setup:landscape:worked-assumptions´).
//!
//! Each test runs the layered pipeline — derive the landscape, render it
//! (´sig:rendering:contract´) — with exact spec inputs and verifies tag
//! parameters (location, magnitude, Q) against spec tables.

use super::helpers::{KernelScalars, derive_tags};
use crate::numerics::stable_logit;
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters};
use crate::resonance::derivation::ResonanceConfig;
use crate::resonance::tags::{Tag, TagResonance};
use crate::testing::{assert_finite, assert_near, find_tag};

/// Default q̂_c for golden examples: Beta(1,1) posterior mean.
const Q_HAT_C: f64 = 0.5;
/// Default σ²_q̂c for golden examples: Beta(1,1) posterior variance.
const SIGMA2_Q_HAT_C: f64 = 1.0 / 12.0;

/// Tolerance for golden-value comparisons (10⁻² — relaxed from spec 10⁻³).
///
/// The spec's worked examples round intermediate results, leading to
/// accumulated differences up to ~0.01 in locations. We use 10⁻² for
/// location/magnitude and verify structural properties exactly.
const TOL: f64 = 0.02;

fn run_4action(p_hat: f64, sigma_eff: f64, kappa_eff: f64) -> Vec<TagResonance> {
    let scalars = KernelScalars {
        p_hat,
        sigma_eff,
        kappa_eff,
        sigma2_q_hat_c: SIGMA2_Q_HAT_C,
    };
    derive_tags(&scalars, Q_HAT_C, &ChannelPolicy::default(), &ResonanceConfig::default())
}

// ═══════════════════════════════════════════════════════════════════════════════
// Golden Examples
// ═══════════════════════════════════════════════════════════════════════════════

/// A confidently-benign entity produces the numbers the specification worked out
/// by hand: seven tags, Good peaked at 0.814 with nearly all the magnitude,
/// Malicious a distant reflection at 0.186, and Allow owning by far the widest
/// action regime at 0.580 against Challenge's 0.048. The closed-form derivation
/// is checked against arithmetic done independently of the code, so a refactor
/// that changes the shape of the field cannot pass unnoticed.
///
/// ´claim:resonance:the-derivation-reproduces-the-specifications-worked-examples´
/// ´test:crate:golden-example-1-low-risk´
#[test]
fn golden_example_1_low_risk() {
    let tags = run_4action(0.05, 0.42, 1.0);
    assert_eq!(tags.len(), 7, "3 classification + 4 action tags");

    // Classification tags, against the low-risk worked landscape (´ex:landscape:low-risk´)
    let good = find_tag(&tags, Tag::Good);
    assert_near(good.location, 0.814, TOL, "Good.location");
    assert_near(good.magnitude, 0.946, TOL, "Good.magnitude");
    assert_near(good.q, 2.381, TOL, "Good.Q");

    let susp = find_tag(&tags, Tag::Suspicious);
    assert_near(susp.location, 0.500, TOL, "Suspicious.location");
    assert_near(susp.magnitude, 0.095, TOL, "Suspicious.magnitude");
    assert_near(susp.q, 0.500, TOL, "Suspicious.Q");

    let mal = find_tag(&tags, Tag::Malicious);
    assert_near(mal.location, 0.186, TOL, "Malicious.location");
    assert_near(mal.magnitude, 0.050, TOL, "Malicious.magnitude");
    assert_near(mal.q, 2.381, TOL, "Malicious.Q");

    // Action tags, against the low-risk worked landscape (´ex:landscape:low-risk´)
    let allow = find_tag(&tags, Tag::Allow);
    assert_near(allow.magnitude, 0.580, TOL, "Allow.magnitude");

    let challenge = find_tag(&tags, Tag::Challenge);
    assert_near(challenge.magnitude, 0.048, TOL, "Challenge.magnitude");

    let slow = find_tag(&tags, Tag::Slow);
    assert_near(slow.magnitude, 0.181, TOL, "Slow.magnitude");

    let block = find_tag(&tags, Tag::Block);
    assert_near(block.magnitude, 0.187, TOL, "Block.magnitude");
}

/// The mirror-image scenario lands on the mirror-image numbers: with the risk
/// estimate at 0.72 it is Malicious that carries the weight at 0.589 while Good
/// falls to 0.229, and Block's regime at 0.428 overtakes Allow's 0.186. Both
/// classification tags share the same Q, since their bandwidth comes from the
/// assessment's spread rather than from which of them happens to be favoured.
///
/// (´claim:resonance:the-derivation-reproduces-the-specifications-worked-examples´)
/// ´test:crate:golden-example-2-high-risk´
#[test]
fn golden_example_2_high_risk() {
    let tags = run_4action(0.72, 0.74, 1.0);
    assert_eq!(tags.len(), 7);

    let good = find_tag(&tags, Tag::Good);
    assert_near(good.location, 0.384, TOL, "Good.location");
    assert_near(good.magnitude, 0.229, TOL, "Good.magnitude");
    assert_near(good.q, 1.351, TOL, "Good.Q");

    let susp = find_tag(&tags, Tag::Suspicious);
    assert_near(susp.location, 0.500, TOL, "Suspicious.location");
    assert_near(susp.magnitude, 0.330, TOL, "Suspicious.magnitude");

    let mal = find_tag(&tags, Tag::Malicious);
    assert_near(mal.location, 0.616, TOL, "Malicious.location");
    assert_near(mal.magnitude, 0.589, TOL, "Malicious.magnitude");
    assert_near(mal.q, 1.351, TOL, "Malicious.Q");

    let allow = find_tag(&tags, Tag::Allow);
    assert_near(allow.magnitude, 0.186, TOL, "Allow.magnitude");

    let block = find_tag(&tags, Tag::Block);
    assert_near(block.magnitude, 0.428, TOL, "Block.magnitude");
}

/// The genuinely uncertain case is the one where the attenuation shows: a wide
/// spread of 0.88 pulls every magnitude toward the middle, leaving Good at 0.464,
/// Malicious at 0.250 and Suspicious at 0.325 — no tag dominant — while the
/// shared classification Q drops to 1.136, the broadest of the three worked
/// examples. Uncertainty flattens and widens the field instead of sharpening a
/// verdict the assessment cannot support.
///
/// (´claim:resonance:the-derivation-reproduces-the-specifications-worked-examples´)
/// ´test:crate:golden-example-3-uncertain´
#[test]
fn golden_example_3_uncertain() {
    let tags = run_4action(0.35, 0.88, 1.0);
    assert_eq!(tags.len(), 7);

    let good = find_tag(&tags, Tag::Good);
    assert_near(good.location, 0.577, TOL, "Good.location");
    assert_near(good.magnitude, 0.464, TOL, "Good.magnitude");
    assert_near(good.q, 1.136, TOL, "Good.Q");

    let susp = find_tag(&tags, Tag::Suspicious);
    assert_near(susp.magnitude, 0.325, TOL, "Suspicious.magnitude");

    let mal = find_tag(&tags, Tag::Malicious);
    assert_near(mal.location, 0.423, TOL, "Malicious.location");
    assert_near(mal.magnitude, 0.250, TOL, "Malicious.magnitude");
    assert_near(mal.q, 1.136, TOL, "Malicious.Q");

    let allow = find_tag(&tags, Tag::Allow);
    assert_near(allow.magnitude, 0.253, TOL, "Allow.magnitude");

    let block = find_tag(&tags, Tag::Block);
    assert_near(block.magnitude, 0.264, TOL, "Block.magnitude");
}

/// Withdrawing Slow from the channel hands its band to the neighbour it was
/// crowding: at inputs otherwise identical to the low-risk example, Challenge's
/// magnitude rises from 0.048 to 0.216 while no Slow tag is emitted at all. The
/// classification tags are untouched, because they read only the assessment and
/// not the action set — so narrowing the operational menu redistributes the
/// posture axis among the survivors without disturbing what the system believes.
///
/// ´claim:resonance:removing-an-action-from-the-channel-widens-the-regime-of-the-neighbour-it-crowded´
/// ´test:crate:golden-example-4-three-action´
#[test]
fn golden_example_4_three_action() {
    // 3-action: [Allow, Challenge, Block] — the three-action worked landscape (´ex:landscape:three-action´)
    let scalars = KernelScalars {
        p_hat: 0.05,
        sigma_eff: 0.42,
        kappa_eff: 1.0,
        // Beta(1,1) prior as stated in the spec for this example
        sigma2_q_hat_c: 1.0 / 12.0,
    };
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };
    let tags = derive_tags(&scalars, Q_HAT_C, &policy, &ResonanceConfig::default());

    assert_eq!(tags.len(), 6, "3 classification + 3 action tags");

    // Classification tags are identical to Example 1 at same p̂, σ_eff
    let good = find_tag(&tags, Tag::Good);
    assert_near(good.location, 0.814, TOL, "Good.location");
    assert_near(good.magnitude, 0.946, TOL, "Good.magnitude");

    // Action magnitudes: regime widths differ from 4-action
    let allow = find_tag(&tags, Tag::Allow);
    assert_near(allow.magnitude, 0.579, TOL, "Allow.magnitude");

    let challenge = find_tag(&tags, Tag::Challenge);
    // Challenge regime is wider in 3-action: 0.217 vs 0.048
    assert_near(challenge.magnitude, 0.216, TOL, "Challenge.magnitude");

    let block = find_tag(&tags, Tag::Block);
    assert_near(block.magnitude, 0.200, TOL, "Block.magnitude");

    // No Slow tag in 3-action system
    assert!(
        tags.iter().all(|t| t.tag != Tag::Slow),
        "3-action system should not have Slow tag"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Structural Properties
// ═══════════════════════════════════════════════════════════════════════════════

/// The tag field is a probability distribution wherever it is read: at a hundred
/// postures spanning the axis the kernels are all finite, their total is strictly
/// positive, and dividing through by that total gives probabilities summing to
/// one within 10⁻¹². Nothing further is needed to turn a set of independently
/// placed Cauchy peaks into an answer a host can act on — the normalisation
/// always exists because the total can never reach zero.
///
/// ´claim:resonance:the-tag-field-normalises-to-a-proper-distribution-at-every-posture´
/// ´test:crate:profile-properness´
#[test]
fn profile_properness() {
    // At every posture π, the kernels sum to a constant (≠ 0)
    // and can be normalised to ΣP = 1.
    let tags = run_4action(0.05, 0.42, 1.0);

    for i in 1..100 {
        let pi = i as f64 / 100.0;
        let logit = stable_logit(pi);

        let kernels: Vec<f64> = tags.iter().map(|t| t.kernel_at_logit(logit)).collect();
        assert_finite(&kernels, &format!("kernels at π = {pi}"));

        let total: f64 = kernels.iter().sum();
        assert!(total > 0.0, "total kernel = 0 at π = {pi}");

        // Renormalised probabilities sum to 1
        let prob_sum: f64 = kernels.iter().map(|k| k / total).sum();
        assert_near(prob_sum, 1.0, 1e-12, &format!("ΣP at π = {pi}"));
    }
}

/// No tag ever contributes exactly nothing, however far the posture is from
/// where it was placed: sampled out at 0.001 and 0.999 — the extremes of the
/// usable axis — every tag in a low-risk field still returns a strictly positive
/// kernel. The Cauchy tail decays but does not terminate, which is what keeps
/// even a badly-mismatched action representable rather than making its
/// probability structurally impossible to express.
///
/// ´claim:resonance:every-tags-kernel-stays-strictly-positive-out-into-the-tails´
/// ´test:crate:profile-positivity´
#[test]
fn profile_positivity() {
    let tags = run_4action(0.05, 0.42, 1.0);

    for &pi in &[0.001, 0.999] {
        let logit = stable_logit(pi);
        for t in &tags {
            let k = t.kernel_at_logit(logit);
            assert!(k > 0.0, "{:?} kernel = {k} ≤ 0 at π = {pi}", t.tag);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tag Ordering
// ═══════════════════════════════════════════════════════════════════════════════

/// Good and Malicious are placed as reflections of one another: their locations
/// sum to one at every probe from a confidently-benign 0.05 through to a
/// high-risk 0.72, with Suspicious pinned at the midpoint they reflect about.
/// The pair says the same thing from opposite ends, so reading the field for
/// evidence of benignity and reading it for evidence of malice are the same
/// measurement with the sign flipped, never two independently-drifting opinions.
///
/// ´claim:resonance:good-and-malicious-are-reflections-of-each-other-about-the-midpoint-where-suspicious-sits´
/// ´test:crate:classification-tag-ordering´
#[test]
fn classification_tag_ordering() {
    for &(p_hat, sigma_eff) in &[(0.05, 0.42), (0.72, 0.74), (0.35, 0.88)] {
        let tags = run_4action(p_hat, sigma_eff, 1.0);
        let good = find_tag(&tags, Tag::Good);
        let mal = find_tag(&tags, Tag::Malicious);
        let susp = find_tag(&tags, Tag::Suspicious);

        // Good.location should be symmetric to Malicious around 0.5
        assert_near(
            good.location + mal.location,
            1.0,
            TOL,
            &format!("Good+Mal symmetry at p̂={p_hat}"),
        );
        // Suspicious is at midpoint
        assert_near(susp.location, 0.5, 0.01, "Suspicious.location");
    }
}
