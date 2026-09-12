// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`ambiguity_non_negative`] | resonance | The gauge is an entropy and so is never negative, in any of its three forms and at any point in the envelope — probing risk from 0.05 to 0.90 against spreads from 0.30 to 1.50 leaves the total and both partitioned figures at or above zero. A host reading the gauge can treat the figure as a magnitude to compare, never a term that might flip sign. |
//! | [`ambiguity_nontrivial`] | resonance | The gauge is not a formality that reports near-zero everywhere: an entity at middling risk under real uncertainty renders a genuinely ambiguous tag field, and all three figures say so. What the number measures is the ambiguity of the rendered display (´sig:rendering:ambiguity-gauge´) — whether resolving it would change any action is fragility's question, asked of the landscape instead (´def:fragility:definition´). |
//! | [`entropy_decomposition_100_configs`] | resonance | Across a hundred randomly drawn assessments the joint entropy is exactly the entropy of the class-versus-action split plus each side's own entropy weighted by its share — to within 10⁻¹², an identity rather than an approximation. Note what this rules out: the total is *not* bounded by the sum of the two partitioned figures, because the family partition is itself informative and that term is what closes the books. |
//! | [`ambiguity_figures_are_sensible`] | resonance | cites (´claim:resonance:a-medium-risk-entity-renders-a-substantially-ambiguous-field´) |

//! Ambiguity-gauge and entropy-decomposition tests (´sig:rendering:ambiguity-gauge´).
//!
//! Property tests for `compute_profile_ambiguity` — non-negativity,
//! non-triviality at medium risk, and the exact chain-rule identity
//! that ties the joint entropy to the class/action partition.

use super::helpers::{KernelScalars, derive_tags};
use crate::numerics::stable_logit;
use crate::resonance::ambiguity::compute_profile_ambiguity;
use crate::resonance::channel::ChannelPolicy;
use crate::resonance::derivation::ResonanceConfig;
use crate::testing::{TestRng, assert_finite, assert_near};

/// The gauge is an entropy and so is never negative, in any of its three
/// forms and at any point in the envelope — probing risk from 0.05 to 0.90
/// against spreads from 0.30 to 1.50 leaves the total and both partitioned
/// figures at or above zero. A host reading the gauge can treat the figure
/// as a magnitude to compare, never a term that might flip sign.
///
/// ´claim:resonance:the-ambiguity-gauge-is-never-negative-in-any-of-its-three-decompositions´
/// ´test:crate:ambiguity-non-negative´
#[test]
fn ambiguity_non_negative() {
    // All three figures should be non-negative for various configurations.
    let configs = [
        (0.05, 0.42),
        (0.35, 0.88),
        (0.72, 0.74),
        (0.50, 0.50),
        (0.10, 1.50),
        (0.90, 0.30),
    ];

    for (p_hat, sigma_eff) in configs {
        let input = KernelScalars {
            p_hat,
            sigma_eff,
            kappa_eff: 1.0,
            sigma2_q_hat_c: 0.05,
        };
        let policy = ChannelPolicy::default();
        let config = ResonanceConfig::default();
        let tags = derive_tags(&input, 0.5, &policy, &config);
        let m = compute_profile_ambiguity(&tags, 0.5);

        assert!(
            m.total >= 0.0,
            "total should be non-negative at p̂={p_hat}, σ={sigma_eff}: {}",
            m.total
        );
        assert!(
            m.classification >= 0.0,
            "classification should be non-negative at p̂={p_hat}, σ={sigma_eff}: {}",
            m.classification
        );
        assert!(
            m.action >= 0.0,
            "action should be non-negative at p̂={p_hat}, σ={sigma_eff}: {}",
            m.action
        );
    }
}

/// The gauge is not a formality that reports near-zero everywhere: an entity
/// at middling risk under real uncertainty renders a genuinely ambiguous tag
/// field, and all three figures say so. What the number measures is the
/// ambiguity of the rendered display (´sig:rendering:ambiguity-gauge´) —
/// whether resolving it would change any action is fragility's question,
/// asked of the landscape instead (´def:fragility:definition´).
///
/// ´claim:resonance:a-medium-risk-entity-renders-a-substantially-ambiguous-field´
/// ´test:crate:ambiguity-nontrivial´
#[test]
fn ambiguity_nontrivial() {
    // A medium-risk configuration renders a genuinely ambiguous field.
    let input = KernelScalars {
        p_hat: 0.35,
        sigma_eff: 0.88,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.05,
    };
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let tags = derive_tags(&input, 0.5, &policy, &config);
    let m = compute_profile_ambiguity(&tags, 0.5);

    assert!(m.total > 0.3, "total = {} should be > 0.3", m.total);
    assert!(
        m.classification > 0.3,
        "classification = {} should be > 0.3",
        m.classification
    );
    assert!(m.action > 0.3, "action = {} should be > 0.3", m.action);
}

/// Across a hundred randomly drawn assessments the joint entropy is exactly
/// the entropy of the class-versus-action split plus each side's own entropy
/// weighted by its share — to within 10⁻¹², an identity rather than an
/// approximation. Note what this rules out: the total is *not* bounded by
/// the sum of the two partitioned figures, because the family partition is
/// itself informative and that term is what closes the books.
///
/// ´claim:resonance:total-information-value-is-the-chain-rule-sum-over-the-class-action-partition´
/// ´test:crate:entropy-decomposition-100-configs´
#[test]
fn entropy_decomposition_100_configs() {
    // The chain rule H_total = H(group) + p_class·H_class + p_action·H_action
    // always holds exactly. (The originally-planned "subadditivity" property
    // does NOT hold for independently renormalized subsets — total can
    // exceed classification + action when the class/action partition
    // is itself informative.)
    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();
    let mut rng = TestRng::new(0xA55A_A55A_A55A_A55A);

    for _ in 0..100_u32 {
        let p_hat = 0.98_f64.mul_add(rng.next_f64(), 0.01);
        let sigma_eff = 2.0_f64.mul_add(rng.next_f64(), 0.1);
        let kappa_eff = 2.0_f64.mul_add(rng.next_f64(), 0.5);

        let input = KernelScalars {
            p_hat,
            sigma_eff,
            kappa_eff,
            sigma2_q_hat_c: 0.05,
        };
        let tags = derive_tags(&input, 0.5, &policy, &config);
        let m = compute_profile_ambiguity(&tags, 0.5);

        // Evaluate kernels at reference posture
        let logit_ref = stable_logit(0.5);
        let kernels: Vec<f64> = tags.iter().map(|t| t.kernel_at_logit(logit_ref)).collect();
        assert_finite(&kernels, "kernels at reference posture");
        assert_finite(&[m.total, m.classification, m.action], "ambiguity figures");
        let total_k: f64 = kernels.iter().sum();
        if total_k <= 0.0 {
            continue;
        }

        let class_k: f64 = tags
            .iter()
            .zip(&kernels)
            .filter(|(t, _)| t.is_classification())
            .map(|(_, &k)| k)
            .sum();
        let p_class = class_k / total_k;
        let p_action = 1.0 - p_class;

        // Chain rule: H = H(group) + Σ p_g · H_g
        let h_group = if p_class > 0.0 && p_action > 0.0 {
            -p_class.mul_add(p_class.ln(), p_action * p_action.ln())
        } else {
            0.0
        };
        let h_chain = p_class.mul_add(m.classification, p_action.mul_add(m.action, h_group));

        assert_near(
            m.total,
            h_chain,
            1e-12,
            &format!("chain rule at p̂={p_hat:.3}, σ={sigma_eff:.3}"),
        );
    }
}

/// The same holds at a tighter spread than the property sweep uses: a
/// medium-risk entity assessed with only moderate uncertainty still renders
/// a substantially ambiguous field, with all three figures non-negative.
/// The figures describe the rendered display; the decision sensitivity a
/// host explores by lives on the landscape (´def:fragility:definition´).
///
/// (´claim:resonance:a-medium-risk-entity-renders-a-substantially-ambiguous-field´)
/// ´test:crate:ambiguity-figures-are-sensible´
#[test]
fn ambiguity_figures_are_sensible() {
    let input = KernelScalars {
        p_hat: 0.35,
        sigma_eff: 0.5,
        kappa_eff: 1.0,
        sigma2_q_hat_c: 0.04,
    };

    let policy = ChannelPolicy::default();
    let config = ResonanceConfig::default();

    let tags = derive_tags(&input, 0.5, &policy, &config);
    let metrics = compute_profile_ambiguity(&tags, 0.5);

    assert!(metrics.total >= 0.0, "total should be non-negative");
    assert!(metrics.classification >= 0.0, "classification should be non-negative");
    assert!(metrics.action >= 0.0, "action should be non-negative");

    assert!(
        metrics.total > 0.1,
        "a medium-risk field should be substantially ambiguous: got {}",
        metrics.total
    );
}
