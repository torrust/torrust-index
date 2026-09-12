// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`uniform_distribution_max_entropy`] | resonance | When four tags sit at the same location with the same magnitude and bandwidth, nothing distinguishes them at any posture and the gauge reaches the natural logarithm of four exactly — the ceiling for a field of that size. A field that has learned nothing scores the most whether or not any observation would change an action, which is exactly why the quantity is an ambiguity gauge and not a measure of what a label would buy (´sig:rendering:ambiguity-gauge´). |
//! | [`single_tag_zero_entropy`] | resonance | At the other end, a field holding one tag scores zero: after normalisation that tag takes the whole distribution, and there is no rendered ambiguity left to report. The gauge says so rather than reporting some floor. |
//! | [`chain_rule_ties_the_three_figures`] | resonance | On a full seven-tag field of varied locations and magnitudes, the joint entropy equals the entropy of the class-versus-action split plus each family's share-weighted own entropy, to within rounding — the identity the hundred-configuration sweep pins across random fields, held here on one hand-built fixture. The retired subadditivity reading is refuted in this same module: the maximum-entropy fixture's total of ln 4 exceeds its families' ln 3 plus zero. |

//! The ambiguity gauge: Shannon entropies of the rendered tag
//! distribution (´sig:rendering:ambiguity-gauge´), computed in the
//! derivation layer and nowhere else (´dec:derivation:exploration-layer´).
//!
//! Three figures at a caller-chosen posture: the joint entropy over all
//! rendered tags, and the two within-family entropies, each family
//! renormalised on its own.
//!
//! **The gauge is not a value of information, and this disclaimer is
//! part of the specification rather than a note about it**
//! (´sig:rendering:ambiguity-gauge´). Entropy of the rendered
//! distribution weights uncertainty by nothing; a value of information
//! weights it by decision sensitivity. The two agree only by
//! coincidence, and they disagree exactly where a host would act on the
//! difference — at a posture where the distribution is broad but every
//! tag leads to the same action, the entropy is high and the
//! information value is nil. Decision-aware exploration uses fragility,
//! which is defined on the landscape and not on this rendering
//! (´def:fragility:definition´); the type is named for ambiguity
//! precisely to sever the association.
//!
//! The three figures are tied by the chain rule, exactly: the joint
//! entropy is the entropy of the class-versus-action split plus each
//! family's own entropy weighted by its share of the rendered mass.
//! No subadditivity holds between them — the total can exceed the sum
//! of the two renormalised family figures whenever the family
//! partition is itself informative, and the module's own maximum-
//! entropy fixture is a counterexample to the inequality once stated
//! here (four indistinguishable tags: total ln 4, classification
//! ln 3, action 0).
//!
//! # Cross-References
//!
//! - (´sig:rendering:ambiguity-gauge´) — the three entropies this module
//!   returns, and the disclaimer above
//! - (´def:fragility:definition´) — the decision-sensitivity surface a
//!   host explores with instead
//! - (´dec:derivation:exploration-layer´) — why the quantity is computed
//!   here and nowhere else

use super::tags::TagResonance;
use crate::numerics::stable_logit;

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// The ambiguity of a rendered tag field at one posture
/// (´sig:rendering:ambiguity-gauge´): display entropies, not values of
/// information — see the module documentation's disclaimer.
#[derive(Clone, Debug, Default)]
pub struct ProfileAmbiguity {
    /// Entropy of the joint rendered distribution at the posture.
    pub total: f64,
    /// Entropy within the classification family (renormalised).
    pub classification: f64,
    /// Entropy within the action family (renormalised).
    pub action: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Algorithm
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the ambiguity gauge from tag resonances at a posture.
///
/// Three independent Shannon entropy computations
/// (´sig:rendering:ambiguity-gauge´):
/// - `total`: H(all tags at the posture)
/// - `classification`: H({Good, Suspicious, Malicious} renormalised)
/// - `action`: H(action tags renormalised)
///
/// # Arguments
///
/// * `tags` — Full set of rendered resonance tags
/// * `posture` — Posture point for evaluation
#[must_use]
pub fn compute_profile_ambiguity(tags: &[TagResonance], posture: f64) -> ProfileAmbiguity {
    if tags.is_empty() {
        return ProfileAmbiguity::default();
    }

    let posture_clamped = posture.clamp(0.001, 0.999);
    let logit_pi = stable_logit(posture_clamped);

    // Evaluate all kernels at the reference posture
    let kernels: Vec<f64> = tags.iter().map(|t| t.kernel_at_logit(logit_pi)).collect();

    // Total entropy over all tags
    let total = shannon_entropy(&kernels);

    // Classification tags: Good, Suspicious, Malicious
    let class_kernels: Vec<f64> = tags
        .iter()
        .zip(kernels.iter())
        .filter(|(t, _)| t.is_classification())
        .map(|(_, &k)| k)
        .collect();
    let classification = shannon_entropy(&class_kernels);

    // Action tags: Allow, Challenge, Slow, Block
    let action_kernels: Vec<f64> = tags
        .iter()
        .zip(kernels.iter())
        .filter(|(t, _)| !t.is_classification())
        .map(|(_, &k)| k)
        .collect();
    let action = shannon_entropy(&action_kernels);

    ProfileAmbiguity {
        total,
        classification,
        action,
    }
}

/// Shannon entropy of unnormalised kernel values.
fn shannon_entropy(kernels: &[f64]) -> f64 {
    let total: f64 = kernels.iter().sum();
    if total <= 0.0 {
        return 0.0;
    }

    let mut h = 0.0;
    for &k in kernels {
        if k > 0.0 {
            let p = k / total;
            h = p.mul_add(-p.ln(), h);
        }
    }

    h
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::resonance::tags::Tag;

    fn make_tag(tag: Tag, location: f64, magnitude: f64, q: f64) -> TagResonance {
        TagResonance {
            tag,
            location,
            magnitude,
            q,
            dominated: false,
        }
    }

    /// When four tags sit at the same location with the same magnitude and
    /// bandwidth, nothing distinguishes them at any posture and the gauge
    /// reaches the natural logarithm of four exactly — the ceiling for a
    /// field of that size. A field that has learned nothing scores the most
    /// whether or not any observation would change an action, which is
    /// exactly why the quantity is an ambiguity gauge and not a measure of
    /// what a label would buy (´sig:rendering:ambiguity-gauge´).
    ///
    /// ´claim:resonance:a-tag-field-with-nothing-to-distinguish-carries-the-maximum-entropy-of-its-size´
    /// ´test:unit:uniform-distribution-max-entropy´
    #[test]
    fn uniform_distribution_max_entropy() {
        // All tags at the same location with same magnitude → uniform → H = ln(n)
        let tags = vec![
            make_tag(Tag::Good, 0.5, 1.0, 1.0),
            make_tag(Tag::Suspicious, 0.5, 1.0, 1.0),
            make_tag(Tag::Malicious, 0.5, 1.0, 1.0),
            make_tag(Tag::Allow, 0.5, 1.0, 1.0),
        ];
        let m = compute_profile_ambiguity(&tags, 0.5);

        let expected = (4.0_f64).ln();
        assert!(
            (m.total - expected).abs() < 1e-10,
            "expected ln(4) = {expected}, got {}",
            m.total
        );
    }

    /// At the other end, a field holding one tag scores zero: after
    /// normalisation that tag takes the whole distribution, and there is no
    /// rendered ambiguity left to report. The gauge says so rather than
    /// reporting some floor.
    ///
    /// ´claim:resonance:a-single-tag-field-carries-zero-ambiguity´
    /// ´test:unit:single-tag-zero-entropy´
    #[test]
    fn single_tag_zero_entropy() {
        let tags = vec![make_tag(Tag::Good, 0.5, 1.0, 1.0)];
        let m = compute_profile_ambiguity(&tags, 0.5);
        assert!(m.total.abs() < 1e-14);
    }

    /// On a full seven-tag field of varied locations and magnitudes, the
    /// joint entropy equals the entropy of the class-versus-action split
    /// plus each family's share-weighted own entropy, to within rounding —
    /// the identity the hundred-configuration sweep pins across random
    /// fields, held here on one hand-built fixture. The retired subadditivity
    /// reading is refuted in this same module: the maximum-entropy
    /// fixture's total of ln 4 exceeds its families' ln 3 plus zero.
    ///
    /// ´claim:resonance:the-chain-rule-ties-the-joint-entropy-to-the-family-partition´
    /// ´test:unit:chain-rule-ties-the-three-figures´
    #[test]
    fn chain_rule_ties_the_three_figures() {
        let tags = vec![
            make_tag(Tag::Good, 0.8, 0.9, 2.0),
            make_tag(Tag::Suspicious, 0.5, 0.1, 0.5),
            make_tag(Tag::Malicious, 0.2, 0.1, 2.0),
            make_tag(Tag::Allow, 0.2, 0.5, 2.0),
            make_tag(Tag::Challenge, 0.6, 0.05, 2.0),
            make_tag(Tag::Slow, 0.7, 0.2, 2.0),
            make_tag(Tag::Block, 0.9, 0.2, 2.0),
        ];
        let m = compute_profile_ambiguity(&tags, 0.5);

        // The family shares of the rendered mass at the read posture.
        let logit_ref = crate::numerics::stable_logit(0.5);
        let kernels: Vec<f64> = tags.iter().map(|t| t.kernel_at_logit(logit_ref)).collect();
        let total_k: f64 = kernels.iter().sum();
        let class_k: f64 = tags
            .iter()
            .zip(&kernels)
            .filter(|(t, _)| t.is_classification())
            .map(|(_, &k)| k)
            .sum();
        let p_class = class_k / total_k;
        let p_action = 1.0 - p_class;

        let h_group = -p_class.mul_add(p_class.ln(), p_action * p_action.ln());
        let h_chain = p_class.mul_add(m.classification, p_action.mul_add(m.action, h_group));
        assert!(
            (m.total - h_chain).abs() < 1e-12,
            "chain rule: total={} vs split-plus-weighted={}",
            m.total,
            h_chain
        );
    }
}
