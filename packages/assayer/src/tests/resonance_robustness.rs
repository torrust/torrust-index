// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`challenge_domination_low_qc`] | resonance | A challenge that almost never catches anyone is barely worth imposing: at q̂_c = 0.05 the adversary gives up nearly nothing by being challenged, so the Challenge regime narrows to nothing and the tag comes back either flagged dominated or carrying a magnitude too small to compete. The channel stops recommending friction it has no evidence will pay for. |
//! | [`extreme_p_hat`] | resonance | Pushing the calibrated probability to within 10⁻⁸ of either end — where the logit the whole placement is built on would otherwise diverge — still yields tags whose location, magnitude and Q are all finite, with every location strictly inside the open posture axis. The input floor is what buys this: a near-certain verdict is an ordinary case for the derivation rather than a singularity that leaks a NaN into every downstream kernel. |
//! | [`extreme_sigma_eff`] | resonance | cites (´claim:resonance:every-tag-stays-finite-and-well-formed-at-the-edges-of-the-input-envelope´) |
//! | [`regime_map_monotonicity`] | resonance | Sweeping the risk estimate across the whole unit interval moves the regime map in one direction only: the permissive action's magnitude never rises and the most restrictive action's never falls, ending with Allow reduced to almost nothing at p̂ = 0.99. Because magnitude is the width of the posture band an action owns, a host that grows more suspicious of an entity can never find the channel quietly recommending more permissiveness than before. |
//! | [`q_floor_binds_at_cold_start`] | resonance | Before any challenge has been observed the effectiveness estimate carries the full Beta(1,1) prior variance, and with blocking forfeiting its catch value the Slow→Block crossover's position becomes almost unknowable. The defensive Q floor is what the adjacent actions land on instead: at least one of Slow and Block comes back sitting at the floor or dominated outright. A tag whose placement is guesswork is given a deliberately blunt bandwidth rather than a sharp peak it has not earned. |
//! | [`q_floor_no_longer_binds_after_evidence`] | resonance | The cold-start bluntness is transient, not structural. Twenty observed challenge outcomes drive the effectiveness variance from one-twelfth down below 0.02, and with the same forfeited blocking rewards every live action then reports a Q strictly clear of the floor. Evidence about how well challenges actually work is what sharpens the whole channel's placement, so the floor is a backstop the system climbs off rather than a resting state. |
//! | [`challenge_dominated_below_375_threshold`] | resonance | A dominated action is not merely given a small regime — it is pinned to a fixed pair of values: magnitude exactly ε_mono and Q exactly Q_min. At q̂_c = 0.30 the Challenge tag reports precisely those. Keeping the tag in the output with a known tiny footprint, instead of dropping it, lets the field still normalise and lets a host see that the action was considered and ruled out rather than never offered. |
//! | [`challenge_alive_above_375_threshold`] | resonance | Ten percentage points of extra challenge effectiveness are enough to bring the action back: at q̂_c = 0.40 Challenge is no longer flagged dominated and carries a magnitude orders of magnitude above the dominated floor. Domination is a live reading of the reward arithmetic against the current effectiveness estimate, so an action written off while challenges were ineffective returns of its own accord once they start working. |
//! | [`interior_regime_widths_are_p_hat_independent`] | resonance | In logit space the crossover landscape slides rigidly as the risk estimate moves: the prior-odds factor is common to every crossover and cancels out of the differences between them, so the interior regimes keep identical widths — here 0.204 for Challenge and 0.928 for Slow — to within 10⁻¹² across probes spanning p̂ from 0.01 to 0.99. How wide an interior action's band is expresses the reward structure alone; only where that band sits depends on the entity. |
//! | [`classification_tag_placement_at_p_hat_one_per_thousand`] | resonance | The classification locations are placed from a *compressed* logit: at p̂ = 0.001 the Good tag sits at σ(½·ln 999) ≈ 0.969 rather than out at 0.999, with Malicious its exact reflection at ≈ 0.031 and Suspicious fixed at the midpoint. Compression is what keeps the outer classifications reachable — a Good peak jammed against the end of the axis would contribute almost nothing at the moderate postures where decisions are actually taken. |
//! | [`classification_tags_symmetric_at_p_hat_one_half`] | resonance | At exactly even odds the three classifications become indistinguishable: all three locations land on 0.5 and all three magnitudes coincide, because the compressed logit vanishes and the default Suspicious coefficient makes its shape factor equal the other two prefactors there. That is the honest output for an entity the model knows nothing about — no classification is preferred, and the field says so rather than manufacturing a lean. This is the "p̂ = 0.5 Maximally Ambiguous" scenario. At the symmetric midpoint `p̂ = 0.5`, the placement algorithm (´alg:rendering:placement´) forces `logit(p̂) = 0`, so the two outer classification tags collapse onto Suspicious: - `μ_Good      = σ( 0) = 0.5` - `μ_Suspicious = 0.5`  (always, by the same placement) - `μ_Malicious = σ( 0) = 0.5` Per the magnitude definition (´def:rendering:magnitudes´) the classification magnitudes, factored by the uncertainty attenuation α, become - `a_Good       = (1 − p̂)      · α = 0.5 α` - `a_Suspicious = c_susp · 4 p̂(1 − p̂) · α = c_susp · α` (the shape max) - `a_Malicious  = p̂           · α = 0.5 α` so at defaults (`c_susp = 0.5`) all three carry the same magnitude `0.5 α`. The scenario also pins the Suspicious-shape claim `c_susp × 4 × 0.25 = 0.5`, which is the attenuation-free magnitude (equivalently, the `α → 1` limit: `σ_eff → 0` drives `σ²_p̂ → 0`, which sends `α → 1` by construction). This is the crate-level companion to the public-surface test [`resonance_geometry::classification_magnitudes_symmetric_on_cold_world`](../../../tests/resonance_geometry.rs); the quantitative magnitude identities live here because they are direct assertions against the `derive_resonance` primitive and belong inside the crate. |
//! | [`derivation_answers_the_declared_action_set`] | resonance | The primary call derives against the declaration it was handed, not against a channel of its own: a well-formed two-action policy yields exactly three classification tags plus its own two action tags in declaration order, and no tag names an action the host never declared. The failure ruled out is the silent substitution the audit measured, where a policy the call disliked was replaced whole by the default four-action channel — rewards, exponents and action set all the package's — with no field of the result showing it. |
//! | [`derivation_fails_hard_on_malformed_action_set`] | resonance | A malformed action set reaching the primary call fails hard instead of being repaired: deriving under a single-action declaration panics. The well-formedness check is the host's at policy load, and a caller that skipped it has handed in its own defect — the call neither shortens the landscape nor swaps in the default channel on the caller's behalf. |

//! Robustness tests for extreme inputs and regime dynamics, where the transform must stay total (´dec:derivation:pure-transform´).
//!
//! Sanity checks that the resonance derivation remains well-behaved
//! (no NaN/Inf, bounded Q, monotonic regime migration) across the full
//! parameter envelope — including cold-start priors and near-boundary
//! posteriors.

use super::helpers::{KernelScalars, derive_tags};
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters, compute_derived_constants};
use crate::resonance::derivation::ResonanceConfig;
use crate::resonance::tags::{Tag, TagResonance};
use crate::risk::challenge::ChallengeEffectivenessState;
use crate::testing::{DEFAULT_TOLERANCES, assert_below, assert_finite, assert_in_unit_interval, assert_near, assert_positive};
use crate::types::ChallengeResult;

// ═══════════════════════════════════════════════════════════════════════════════
// Challenge Domination (´alg:rendering:dominated-treatment´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A challenge that almost never catches anyone is barely worth imposing: at
/// q̂_c = 0.05 the adversary gives up nearly nothing by being challenged, so the
/// Challenge regime narrows to nothing and the tag comes back either flagged
/// dominated or carrying a magnitude too small to compete. The channel stops
/// recommending friction it has no evidence will pay for.
///
/// ´claim:resonance:a-challenge-that-catches-almost-nobody-collapses-to-a-vanishing-regime´
/// ´test:crate:challenge-domination-low-qc´
#[test]
fn challenge_domination_low_qc() {
    // With very low q̂_c, Challenge catches almost nobody.
    // Δ_bad(A→C) approaches 0 (adversary gains nothing from the challenge).
    // Challenge should be dominated or have near-zero magnitude.
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.05, &policy, &config); // Very low q̂_c

    let challenge = tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    // At q̂_c = 0.05 the Challenge regime is very narrow.
    assert!(
        challenge.dominated || challenge.magnitude < 0.02,
        "Challenge should be dominated or near-zero at q̂_c=0.05 \
         (dominated={}, magnitude={})",
        challenge.dominated,
        challenge.magnitude
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Extreme Inputs (´dec:derivation:pure-transform´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Pushing the calibrated probability to within 10⁻⁸ of either end — where the
/// logit the whole placement is built on would otherwise diverge — still yields
/// tags whose location, magnitude and Q are all finite, with every location
/// strictly inside the open posture axis. The input floor is what buys this: a
/// near-certain verdict is an ordinary case for the derivation rather than a
/// singularity that leaks a NaN into every downstream kernel.
///
/// ´claim:resonance:every-tag-stays-finite-and-well-formed-at-the-edges-of-the-input-envelope´
/// ´test:crate:extreme-p-hat´
#[test]
fn extreme_p_hat() {
    // p̂ near 0 and near 1: no NaN/Inf; all locations in (0, 1).
    let config = ResonanceConfig::default();
    let policy = ChannelPolicy::default();

    for &p_hat in &[1e-8, 1e-6, 0.999_999, 1.0 - 1e-8] {
        let input = KernelScalars {
            p_hat,
            sigma_eff: 0.5,
            kappa_eff: 1.0,
            sigma2_q_hat_c: 0.05,
        };
        let tags = derive_tags(&input, 0.5, &policy, &config);

        let locations: Vec<f64> = tags.iter().map(|t| t.location).collect();
        let magnitudes: Vec<f64> = tags.iter().map(|t| t.magnitude).collect();
        let qs: Vec<f64> = tags.iter().map(|t| t.q).collect();
        assert_finite(&locations, &format!("locations at p̂={p_hat}"));
        assert_finite(&magnitudes, &format!("magnitudes at p̂={p_hat}"));
        assert_finite(&qs, &format!("Q values at p̂={p_hat}"));

        for t in &tags {
            assert_in_unit_interval(t.location, &format!("{:?} location at p̂={p_hat}", t.tag));
            assert_positive(t.magnitude, &format!("{:?} magnitude at p̂={p_hat}", t.tag));
            assert_positive(t.q, &format!("{:?} Q at p̂={p_hat}", t.tag));
        }
    }
}

/// The other extreme axis behaves the same way: uncertainty spanning fifteen
/// orders of magnitude from 10⁻¹⁵ to 10⁶ leaves every Q positive and finite
/// rather than exploding as σ approaches zero or collapsing as it grows. A
/// perfectly-confident assessment and a hopelessly vague one are both merely
/// ends of the range the placement formula is defined on.
///
/// (´claim:resonance:every-tag-stays-finite-and-well-formed-at-the-edges-of-the-input-envelope´)
/// ´test:crate:extreme-sigma-eff´
#[test]
fn extreme_sigma_eff() {
    // σ_eff near 0 and very large: Q bounded; magnitudes positive.
    let config = ResonanceConfig::default();
    let policy = ChannelPolicy::default();

    for &sigma_eff in &[1e-15, 1e-10, 100.0, 1e6] {
        let input = KernelScalars {
            p_hat: 0.3,
            sigma_eff,
            kappa_eff: 1.0,
            sigma2_q_hat_c: 0.05,
        };
        let tags = derive_tags(&input, 0.5, &policy, &config);

        let locations: Vec<f64> = tags.iter().map(|t| t.location).collect();
        let magnitudes: Vec<f64> = tags.iter().map(|t| t.magnitude).collect();
        let qs: Vec<f64> = tags.iter().map(|t| t.q).collect();
        assert_finite(&locations, &format!("locations at σ={sigma_eff}"));
        assert_finite(&magnitudes, &format!("magnitudes at σ={sigma_eff}"));
        assert_finite(&qs, &format!("Q values at σ={sigma_eff}"));

        for t in &tags {
            assert_positive(t.q, &format!("{:?} Q at σ={sigma_eff}", t.tag));
            assert_positive(t.magnitude, &format!("{:?} magnitude at σ={sigma_eff}", t.tag));
            assert_in_unit_interval(t.location, &format!("{:?} location at σ={sigma_eff}", t.tag));
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Regime Map Monotonicity (´cav:derivation:guard-absorbed´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Sweeping the risk estimate across the whole unit interval moves the regime
/// map in one direction only: the permissive action's magnitude never rises and
/// the most restrictive action's never falls, ending with Allow reduced to
/// almost nothing at p̂ = 0.99. Because magnitude is the width of the posture
/// band an action owns, a host that grows more suspicious of an entity can
/// never find the channel quietly recommending more permissiveness than before.
///
/// ´claim:resonance:rising-risk-shrinks-the-permissive-regime-and-grows-the-restrictive-one´
/// ´test:crate:regime-map-monotonicity´
#[test]
fn regime_map_monotonicity() {
    // As p̂ increases from 0.01 to 0.99, the Allow magnitude should
    // monotonically decrease and the Block magnitude should monotonically
    // increase (regime widths shift toward more restrictive actions).
    let config = ResonanceConfig::default();
    let policy = ChannelPolicy::default();

    let slack = DEFAULT_TOLERANCES.default;
    let mut prev_allow = f64::MAX;
    let mut prev_block = 0.0_f64;

    for i in 1..100 {
        let p_hat = i as f64 / 100.0;
        let input = KernelScalars {
            p_hat,
            sigma_eff: 0.5,
            kappa_eff: 1.0,
            sigma2_q_hat_c: 0.05,
        };
        let tags = derive_tags(&input, 0.5, &policy, &config);

        let allow = tags.iter().find(|t| t.tag == Tag::Allow).unwrap();
        let block = tags.iter().find(|t| t.tag == Tag::Block).unwrap();

        // Skip dominated tags — they have ε_mono magnitude.
        if !allow.dominated {
            assert_below(
                allow.magnitude,
                prev_allow + slack,
                &format!("Allow magnitude should be non-increasing at p̂={p_hat}"),
            );
            prev_allow = allow.magnitude;
        }

        if !block.dominated {
            assert_below(
                prev_block,
                block.magnitude + slack,
                &format!("Block magnitude should be non-decreasing at p̂={p_hat}"),
            );
            prev_block = block.magnitude;
        }
    }

    // Sanity: Allow at p̂ = 0.01 should be much larger than at p̂ = 0.99.
    assert_below(prev_allow, 0.1, "Allow should be small at high p̂");
}

// ═══════════════════════════════════════════════════════════════════════════════
// q̂_c Sensitivity — the Q floor at cold start (´def:rendering:bandwidths´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Before any challenge has been observed the effectiveness estimate carries the
/// full Beta(1,1) prior variance, and with blocking forfeiting its catch value
/// the Slow→Block crossover's position becomes almost unknowable. The defensive
/// Q floor is what the adjacent actions land on instead: at least one of Slow
/// and Block comes back sitting at the floor or dominated outright. A tag whose
/// placement is guesswork is given a deliberately blunt bandwidth rather than a
/// sharp peak it has not earned.
///
/// ´claim:resonance:the-q-floor-catches-the-crossover-whose-uncertainty-has-become-enormous´
/// ´test:crate:q-floor-binds-at-cold-start´
#[test]
fn q_floor_binds_at_cold_start() {
    // At β_c = 0, the S→B crossover uncertainty is enormous at cold start
    // (σ²_q̂c = 1/12 from Beta(1,1) prior). The Q floor should bind for
    // the actions adjacent to the S→B crossover.
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 1.0 / 12.0, // Beta(1,1) prior variance
    };
    let reward = RewardParameters {
        block_catches: 0.0, // β_c = 0
        ..RewardParameters::default()
    };
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
        reward,
        ..ChannelPolicy::default()
    };
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    // At β_c = 0 with σ²_q̂c = 1/12: the S→B crossover q̂_c-contribution
    // is very large, ~6.48 by the crossover covariance
    // (´thm:landscape:crossover-covariance´), so the Q floor (0.3)
    // should bind for Slow and/or Block.
    let slow = tags.iter().find(|t| t.tag == Tag::Slow).unwrap();
    let block = tags.iter().find(|t| t.tag == Tag::Block).unwrap();

    // TODO ´todo:test:tighten-this-once-the-derivation-path´: Tighten this once the derivation path propagates
    // non-positive Δ_bad through the ADR's extreme-uncertainty/Q-floor backstop.
    // The current assertion accepts the existing dominated fallback; the ADR
    // text calls for a stricter adjacent-Q-floor witness.
    // At least one of Slow / Block should have Q at or very near the floor.
    let near_floor = |t: &TagResonance| t.dominated || (t.q - config.q_floor).abs() < 0.05;
    assert!(
        near_floor(slow) || near_floor(block),
        "Slow Q={} or Block Q={} should be near Q_floor={} at β_c=0 cold start",
        slow.q,
        block.q,
        config.q_floor
    );
}

/// The cold-start bluntness is transient, not structural. Twenty observed
/// challenge outcomes drive the effectiveness variance from one-twelfth down
/// below 0.02, and with the same forfeited blocking rewards every live action
/// then reports a Q strictly clear of the floor. Evidence about how well
/// challenges actually work is what sharpens the whole channel's placement, so
/// the floor is a backstop the system climbs off rather than a resting state.
///
/// ´claim:resonance:the-floor-releases-once-challenge-evidence-shrinks-the-crossover-variance´
/// ´test:crate:q-floor-no-longer-binds-after-evidence´
#[test]
fn q_floor_no_longer_binds_after_evidence() {
    // The Q floor (´def:rendering:bandwidths´) — companion to `q_floor_binds_at_cold_start`.
    //
    // At β_c = 0 with a Beta(1, 1) prior the S→B crossover variance is
    // dominated by σ²(q̂_c) = 1/12 and pushes the floor. After 20
    // challenge outcomes (10 Fail + 10 Pass) the tracker variance drops
    // to ~0.011, the floor no longer binds, and every live non-dominated
    // action regains a strictly-above-floor Q.
    let mut state = ChallengeEffectivenessState::new(1.0, 1.0, ChallengeEffectivenessState::DEFAULT_GAMMA_QT);
    let t0 = state.t_last;
    for _ in 0..10 {
        state.update(ChallengeResult::Fail, &t0);
    }
    for _ in 0..10 {
        state.update(ChallengeResult::Pass, &t0);
    }
    let (q_hat, sigma2_q_hat_c) = state.estimate(&t0);
    assert_below(sigma2_q_hat_c, 0.02, "σ²(q̂_c) should drop below 0.02 after 20 outcomes");

    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c,
    };
    let reward = RewardParameters {
        block_catches: 0.0, // β_c = 0 (unchanged from cold-start scenario)
        ..RewardParameters::default()
    };
    let policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Slow, Action::Block],
        reward,
        ..ChannelPolicy::default()
    };
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, q_hat, &policy, &config);

    // Every live (non-dominated) action tag should now sit strictly above
    // the floor — the transient pathology has resolved with evidence.
    for t in tags.iter().filter(|t| !t.is_classification() && !t.dominated) {
        assert!(
            t.q > config.q_floor + 0.05,
            "{:?} Q={} is still near the floor ({}) after 20 outcomes",
            t.tag,
            t.q,
            config.q_floor
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Challenge Domination Threshold (´thm:landscape:catching-forfeiture´)
// ═══════════════════════════════════════════════════════════════════════════════
//
// For the 4-action [Allow, Challenge, Slow, Block] channel with default
// rewards, Challenge becomes dominated when q̂_c ≤ 0.375. Derivation:
//
//   δ_bad(A→C) = q · (missed + caught) = 5q
//   δ_bad(C→S) = (1 − q) · missed · slow_severity = 0.6 · (1 − q)
//   δ_good(A→C) = friction = 0.30
//   δ_good(C→S) = friction · slow_severity = 0.06
//
// Domination condition (crossover posture A→C ≥ C→S):
//   0.30 / 5q ≥ 0.06 / (0.6 · (1 − q))
//   ⇔ q ≤ 0.375.

/// A dominated action is not merely given a small regime — it is pinned to a
/// fixed pair of values: magnitude exactly ε_mono and Q exactly Q_min. At
/// q̂_c = 0.30 the Challenge tag reports precisely those. Keeping the tag in the
/// output with a known tiny footprint, instead of dropping it, lets the field
/// still normalise and lets a host see that the action was considered and ruled
/// out rather than never offered.
///
/// ´claim:resonance:a-dominated-action-is-pinned-to-epsilon-mono-magnitude-and-q-min´
/// ´test:crate:challenge-dominated-below-375-threshold´
#[test]
fn challenge_dominated_below_375_threshold() {
    // At q̂_c = 0.30 (below the 0.375 threshold), the Challenge tag
    // should collapse to the dominated-action floor: magnitude = ε_mono
    // and Q = Q_min, the dominated-tag floor (´alg:rendering:dominated-treatment´).
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.30, &policy, &config);

    let challenge = tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    assert!(
        challenge.dominated,
        "Challenge should be dominated at q̂_c = 0.30 (below 0.375 threshold)"
    );
    assert_near(
        challenge.magnitude,
        config.epsilon_mono,
        DEFAULT_TOLERANCES.default,
        "dominated Challenge magnitude should equal ε_mono",
    );
    assert_near(
        challenge.q,
        config.q_min,
        DEFAULT_TOLERANCES.default,
        "dominated Challenge Q should equal Q_min",
    );
}

/// Ten percentage points of extra challenge effectiveness are enough to bring
/// the action back: at q̂_c = 0.40 Challenge is no longer flagged dominated and
/// carries a magnitude orders of magnitude above the dominated floor. Domination
/// is a live reading of the reward arithmetic against the current effectiveness
/// estimate, so an action written off while challenges were ineffective returns
/// of its own accord once they start working.
///
/// ´claim:resonance:domination-is-a-threshold-in-challenge-effectiveness-not-a-permanent-verdict´
/// ´test:crate:challenge-alive-above-375-threshold´
#[test]
fn challenge_alive_above_375_threshold() {
    // At q̂_c = 0.40 (above the 0.375 threshold), Challenge must be live
    // again — not flagged as dominated and carrying a meaningful magnitude.
    let input = KernelScalars {
        p_hat: 0.3,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.40, &policy, &config);

    let challenge = tags.iter().find(|t| t.tag == Tag::Challenge).unwrap();
    assert!(
        !challenge.dominated,
        "Challenge should be alive at q̂_c = 0.40 (above 0.375 threshold)"
    );
    assert!(
        challenge.magnitude > 10.0 * config.epsilon_mono,
        "Challenge magnitude ({}) should sit well above ε_mono ({}) at q̂_c = 0.40",
        challenge.magnitude,
        config.epsilon_mono
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interior Regime Widths — p̂-Independence (´thm:landscape:rigid-translation´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Returns the logit-space width of each interior regime under the given
/// derived constants at probability `p_hat`.
///
/// The spec-level formula for the j-th crossover logit is
///   ℓⱼ = (1/β_sum) · ln( ((1−p)/p) · (δ_good_j / δ_bad_j) )
/// so the width of interior regime j (between crossovers j−1 and j) is
///   wⱼ = ℓⱼ − ℓ_{j−1} = (1/β_sum) · ln(
///       (δ_good_j · δ_bad_{j−1}) / (δ_bad_j · δ_good_{j−1})
///   )
/// — the (1−p)/p factor cancels, so widths are p̂-independent.
fn interior_regime_logit_widths(derived: &crate::resonance::channel::ChannelDerivedConstants, p_hat: f64) -> Vec<f64> {
    let beta_sum = derived.beta_sum;
    let logits: Vec<f64> = derived
        .differentials
        .iter()
        .map(|d| (1.0 / beta_sum) * ((1.0 - p_hat) * d.delta_good / (p_hat * d.delta_bad)).ln())
        .collect();
    logits.windows(2).map(|w| w[1] - w[0]).collect()
}

/// In logit space the crossover landscape slides rigidly as the risk estimate
/// moves: the prior-odds factor is common to every crossover and cancels out of
/// the differences between them, so the interior regimes keep identical widths —
/// here 0.204 for Challenge and 0.928 for Slow — to within 10⁻¹² across probes
/// spanning p̂ from 0.01 to 0.99. How wide an interior action's band is expresses
/// the reward structure alone; only where that band sits depends on the entity.
///
/// ´claim:resonance:interior-regime-widths-are-fixed-by-the-reward-structure-alone´
/// ´test:crate:interior-regime-widths-are-p-hat-independent´
#[test]
fn interior_regime_widths_are_p_hat_independent() {
    // Per the rigid-translation theorem (´thm:landscape:rigid-translation´): the entire crossover landscape translates rigidly
    // with p̂ in logit space. Interior widths (w_C for Challenge, w_S for
    // Slow) are identical at every p̂.
    let policy = ChannelPolicy::default();
    let derived = compute_derived_constants(&policy, 0.5);
    let probes = [0.01, 0.1, 0.3, 0.5, 0.7, 0.9, 0.99];

    let reference = interior_regime_logit_widths(&derived, probes[0]);
    assert_eq!(reference.len(), 2, "4-action channel → 2 interior regimes (C, S)");

    // Spec-level values at defaults (β_sum = 2.5):
    //   w_C = (1/2.5) · ln((0.06 · 2.50) / (0.30 · 0.30)) ≈ 0.204
    //   w_S = (1/2.5) · ln((2.44 · 0.30) / (1.20 · 0.06)) ≈ 0.928
    assert_near(reference[0], 0.204, 0.001, "w_C in logit space ≈ 0.204 at defaults");
    assert_near(reference[1], 0.928, 0.001, "w_S in logit space ≈ 0.928 at defaults");

    for &p in &probes[1..] {
        let widths = interior_regime_logit_widths(&derived, p);
        assert_near(widths[0], reference[0], 1e-12, &format!("w_C invariant at p̂={p}"));
        assert_near(widths[1], reference[1], 1e-12, &format!("w_S invariant at p̂={p}"));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Classification Tag Placement (´alg:rendering:placement´)
// ═══════════════════════════════════════════════════════════════════════════════

/// The classification locations are placed from a *compressed* logit: at
/// p̂ = 0.001 the Good tag sits at σ(½·ln 999) ≈ 0.969 rather than out at 0.999,
/// with Malicious its exact reflection at ≈ 0.031 and Suspicious fixed at the
/// midpoint. Compression is what keeps the outer classifications reachable —
/// a Good peak jammed against the end of the axis would contribute almost
/// nothing at the moderate postures where decisions are actually taken.
///
/// ´claim:resonance:location-compression-keeps-the-outer-classifications-reachable-from-moderate-postures´
/// ´test:crate:classification-tag-placement-at-p-hat-one-per-thousand´
#[test]
fn classification_tag_placement_at_p_hat_one_per_thousand() {
    // Placement (´alg:rendering:placement´): with the default c_ℓ = 0.5 location compression,
    //   μ_Good      = σ(−c_ℓ · logit(p̂))
    //   μ_Malicious = σ( c_ℓ · logit(p̂))
    // At p̂ = 0.001: logit(p̂) = ln(0.001 / 0.999) = −ln(999),
    //   μ_Good      = σ( 0.5 · ln(999)) ≈ 0.969
    //   μ_Malicious = σ(−0.5 · ln(999)) ≈ 0.031
    let input = KernelScalars {
        p_hat: 0.001,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);

    let good = tags.iter().find(|t| t.tag == Tag::Good).unwrap();
    let suspicious = tags.iter().find(|t| t.tag == Tag::Suspicious).unwrap();
    let malicious = tags.iter().find(|t| t.tag == Tag::Malicious).unwrap();

    let sigmoid_half_ln999 = 1.0 / (1.0 + (-0.5_f64 * 999.0_f64.ln()).exp());
    // sigmoid(0.5 · ln 999) ≈ 0.969389
    assert_near(good.location, sigmoid_half_ln999, 1e-9, "μ_Good at p̂ = 0.001");
    assert_near(malicious.location, 1.0 - sigmoid_half_ln999, 1e-9, "μ_Malicious at p̂ = 0.001");
    assert_near(suspicious.location, 0.5, 1e-12, "μ_Suspicious is always 0.5");

    // The c_ℓ = 0.5 compression keeps Good away from π = 0.999 — it
    // should sit comfortably below 0.99 so it remains accessible from
    // moderate postures.
    assert_below(good.location, 0.99, "Good stays below 0.99 under c_ℓ compression");
    assert!(
        good.location > 0.96,
        "Good should sit around 0.969 at p̂ = 0.001 under c_ℓ = 0.5"
    );
    assert_in_unit_interval(malicious.location, "μ_Malicious");
}

/// At exactly even odds the three classifications become indistinguishable:
/// all three locations land on 0.5 and all three magnitudes coincide, because
/// the compressed logit vanishes and the default Suspicious coefficient makes
/// its shape factor equal the other two prefactors there. That is the honest
/// output for an entity the model knows nothing about — no classification is
/// preferred, and the field says so rather than manufacturing a lean.
///
/// This is the "p̂ = 0.5 Maximally Ambiguous" scenario.
///
/// At the symmetric midpoint `p̂ = 0.5`, the placement algorithm (´alg:rendering:placement´) forces
/// `logit(p̂) = 0`, so the two outer classification tags collapse onto
/// Suspicious:
///
/// - `μ_Good      = σ( 0) = 0.5`
/// - `μ_Suspicious = 0.5`  (always, by the same placement)
/// - `μ_Malicious = σ( 0) = 0.5`
///
/// Per the magnitude definition (´def:rendering:magnitudes´) the classification magnitudes, factored by the
/// uncertainty attenuation α, become
///
/// - `a_Good       = (1 − p̂)      · α = 0.5 α`
/// - `a_Suspicious = c_susp · 4 p̂(1 − p̂) · α = c_susp · α` (the shape max)
/// - `a_Malicious  = p̂           · α = 0.5 α`
///
/// so at defaults (`c_susp = 0.5`) all three carry the same magnitude
/// `0.5 α`. The scenario also pins the Suspicious-shape claim
/// `c_susp × 4 × 0.25 = 0.5`, which is the attenuation-free magnitude
/// (equivalently, the `α → 1` limit: `σ_eff → 0` drives `σ²_p̂ → 0`,
/// which sends `α → 1` by construction).
///
/// This is the crate-level companion to the public-surface test
/// [`resonance_geometry::classification_magnitudes_symmetric_on_cold_world`](../../../tests/resonance_geometry.rs);
/// the quantitative magnitude identities live here because they are
/// direct assertions against the `derive_resonance` primitive and
/// belong inside the crate.
///
/// ´claim:resonance:at-maximum-ambiguity-the-three-classifications-collapse-to-one-location-and-one-magnitude´
/// ´test:crate:classification-tags-symmetric-at-p-hat-one-half´
#[test]
fn classification_tags_symmetric_at_p_hat_one_half() {
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();

    // (a) Symmetry at cold-start σ_eff (high uncertainty — α ≠ 1 in
    //     general, but the three-way magnitude equality must still hold).
    let input = KernelScalars {
        p_hat: 0.5,
        sigma_eff: 0.42,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let tags = derive_tags(&input, 0.5, &policy, &config);
    let good = tags.iter().find(|t| t.tag == Tag::Good).unwrap();
    let suspicious = tags.iter().find(|t| t.tag == Tag::Suspicious).unwrap();
    let malicious = tags.iter().find(|t| t.tag == Tag::Malicious).unwrap();

    // Locations collapse to 0.5.
    let tol = DEFAULT_TOLERANCES.default;
    assert_near(good.location, 0.5, tol, "μ_Good at p̂ = 0.5");
    assert_near(suspicious.location, 0.5, 1e-12, "μ_Suspicious is always 0.5");
    assert_near(malicious.location, 0.5, tol, "μ_Malicious at p̂ = 0.5");

    // Magnitudes equal (defaults: c_susp = 0.5 makes the prefactors
    // (1 − p̂) = c_susp · 4 p̂(1 − p̂) = p̂ all collapse to 0.5 at p̂ = 0.5,
    // and all three share the same attenuation α).
    assert_near(
        good.magnitude,
        suspicious.magnitude,
        tol,
        "mag(Good) = mag(Suspicious) at p̂ = 0.5 under defaults",
    );
    assert_near(
        suspicious.magnitude,
        malicious.magnitude,
        tol,
        "mag(Suspicious) = mag(Malicious) at p̂ = 0.5 under defaults",
    );
    assert_positive(suspicious.magnitude, "Suspicious magnitude is live");
    assert_finite(
        &[good.magnitude, suspicious.magnitude, malicious.magnitude],
        "classification magnitudes at p̂ = 0.5",
    );
    assert!(!good.dominated, "Good live at p̂ = 0.5");
    assert!(!suspicious.dominated, "Suspicious live at p̂ = 0.5");
    assert!(!malicious.dominated, "Malicious live at p̂ = 0.5");

    // (b) Shape-maximum identity: in the α → 1 limit (no uncertainty
    //     attenuation), Suspicious's magnitude equals
    //     `c_susp · 4 p̂(1 − p̂) = 0.5`. This is exactly the scenario's
    //     closed-form claim, independent of σ_eff-driven attenuation.
    let input_no_attenuation = KernelScalars {
        p_hat: 0.5,
        sigma_eff: 0.0, // drives σ²_p̂ → 0, so α → 1
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let tags_na = derive_tags(&input_no_attenuation, 0.5, &policy, &config);
    let susp_na = tags_na.iter().find(|t| t.tag == Tag::Suspicious).unwrap();
    assert_near(
        susp_na.magnitude,
        0.5,
        1e-12,
        "mag(Suspicious) at p̂ = 0.5 with α = 1 is exactly c_susp · 4 · 0.25 = 0.5",
    );

    // All three magnitudes equal 0.5 in the α = 1 limit.
    let good_na = tags_na.iter().find(|t| t.tag == Tag::Good).unwrap();
    let mal_na = tags_na.iter().find(|t| t.tag == Tag::Malicious).unwrap();
    assert_near(good_na.magnitude, 0.5, 1e-12, "mag(Good) = 0.5 at α = 1, p̂ = 0.5");
    assert_near(mal_na.magnitude, 0.5, 1e-12, "mag(Malicious) = 0.5 at α = 1, p̂ = 0.5");
}

// ═══════════════════════════════════════════════════════════════════════════════
// The primary call takes the declaration as given
// ═══════════════════════════════════════════════════════════════════════════════

/// A held assessment for exercising the primary derivation directly: moderate
/// risk, finite spread, no Sentinels, no axes.
fn held_assessment() -> crate::assessment::RiskAssessment {
    crate::assessment::RiskAssessment {
        id: crate::types::AssessmentId(1),
        risk: crate::assessment::RiskBasis {
            p_bad: 0.3,
            uncertainty: 0.2,
            borrowed_share: 0.0,
            anchor_weight: 0.5,
            rho_eff: -0.85,
            sigma_eff: 0.4,
            kappa_eff: 1.0,
            p_bad_sister: 0.3,
            p_bad_operational: 0.3,
            intervention_effectiveness: 0.0,
            anchor_converged: false,
            n_sentinels_reporting: 0,
            sister_regime_calibration_records: 0.0,
            anchor_regime_calibration_records: 0.0,
        },
        outcome_predictions: std::collections::HashMap::new(),
        per_sentinel: std::collections::HashMap::new(),
        health: crate::assessment::HealthSnapshot {
            degradation: crate::health::DegradationContext::default(),
            snapshot_version: 0,
            snapshot_age_seconds: 0.0,
            convergence_stage: crate::health::CompositeConvergenceStage::ColdStart,
            sentinel_coverage: 0.0,
            calibration_mature: false,
            anchor_regime_frozen: false,
            drift_flag: false,
            uncertainty_inflation: None,
            zero_sentinels: true,
            standardisation_phase: crate::feature::standardisation::StandardisationPhase::WaitingForInit,
            standardisation_observations: 0,
            pending_buffer_evictions: 0,
        },
    }
}

/// The primary call derives against the declaration it was handed, not
/// against a channel of its own: a well-formed two-action policy yields
/// exactly three classification tags plus its own two action tags in
/// declaration order, and no tag names an action the host never declared.
/// The failure ruled out is the silent substitution the audit measured,
/// where a policy the call disliked was replaced whole by the default
/// four-action channel — rewards, exponents and action set all the
/// package's — with no field of the result showing it.
///
/// ´claim:resonance:the-primary-call-derives-against-the-declared-action-set-and-never-substitutes-a-channel´
/// ´test:crate:derivation-answers-the-declared-action-set´
#[test]
fn derivation_answers_the_declared_action_set() {
    use crate::resonance::derivation::{ChallengeEstimate, render_resonances};
    use crate::resonance::landscape::derive_landscape;

    let assessment = held_assessment();
    let two_action = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        ..ChannelPolicy::default()
    };
    crate::resonance::channel::validate_channel_policy(&two_action).expect("the declaration is well formed");

    let narrow = render_resonances(
        &derive_landscape(&assessment, &two_action, ChallengeEstimate::default()),
        &assessment.risk,
        &ResonanceConfig::default(),
    );
    let wide = render_resonances(
        &derive_landscape(&assessment, &ChannelPolicy::default(), ChallengeEstimate::default()),
        &assessment.risk,
        &ResonanceConfig::default(),
    );

    // 3 classification tags + the 2 declared action tags, in order.
    assert_eq!(narrow.tags.len(), 5, "3 classification + 2 declared action tags");
    let action_kinds: Vec<Tag> = narrow.tags[3..].iter().map(|t| t.tag).collect();
    assert_eq!(action_kinds, vec![Tag::Allow, Tag::Block]);

    // The default channel's derivation has a different shape entirely, so
    // the two cannot agree tag for tag — the audit's confirming experiment
    // (two derivations agreeing across different declarations) now fails.
    assert_eq!(wide.tags.len(), 7, "the default four-action channel derives 7 tags");
}

/// A malformed action set reaching the primary call fails hard instead of
/// being repaired: deriving under a single-action declaration panics. The
/// well-formedness check is the host's at policy load, and a caller that
/// skipped it has handed in its own defect — the call neither shortens the
/// landscape nor swaps in the default channel on the caller's behalf.
///
/// ´claim:resonance:a-malformed-declaration-fails-hard-at-the-call-instead-of-being-repaired´
/// ´test:crate:derivation-fails-hard-on-malformed-action-set´
#[test]
#[should_panic(expected = "action set must have at least 2 actions")]
fn derivation_fails_hard_on_malformed_action_set() {
    use crate::resonance::derivation::ChallengeEstimate;
    use crate::resonance::landscape::derive_landscape;

    let assessment = held_assessment();
    let malformed = ChannelPolicy {
        actions: vec![Action::Allow],
        ..ChannelPolicy::default()
    };
    drop(derive_landscape(&assessment, &malformed, ChallengeEstimate::default()));
}
