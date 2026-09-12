// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`tag_classification_vs_action`] | resonance | The seven tags fall into exactly two groups with nothing in either both or neither: Good, Suspicious and Malicious answer what the request *is*, while Allow, Challenge, Slow and Block answer what to *do* about it. The split is total, so code filtering the field by one predicate and code filtering by its negation between them see every tag exactly once. |
//! | [`cauchy_kernel_at_peak`] | resonance | Evaluated at its own location the kernel returns the magnitude exactly, with the bandwidth playing no part: the distance term vanishes and the denominator is one. Magnitude is therefore readable as the tag's peak height rather than as an abstract scaling coefficient, which is what makes the golden-value tables meaningful as a description of the field. |
//! | [`cauchy_kernel_guard_fires`] | resonance | Past the point where the squared distance term would overwhelm the arithmetic, the kernel stops dividing and hands back a fixed tiny constant instead — small enough to be negligible in any sum, but never zero. Normalising the field divides by the total of the kernels, so a tag that returned a true zero would make its own probability undefined rather than merely unlikely. |
//! | [`cauchy_kernel_symmetry`] | resonance | Equal steps either side of a tag's location give the same kernel value: only the squared distance enters, so the direction of the offset is invisible. A tag makes a claim about a point on the posture axis and how sharply it is held, and says nothing about which way an observation missed by. |
//! | [`kernel_at_logit_integrates`] | resonance | Asking a tag for its own kernel value gives exactly what calling the bare kernel with that tag's three fields would, the logit transform of the location included. The convenience method carries no extra behaviour of its own, so callers reasoning about the field in terms of the closed-form kernel are reasoning about what the method actually does. |

//! Resonance tag types and kernel functions.
//!
//! # Cross-References
//!
//! - (´dec:landscape:rendering-optional´) — the rendering layer these tag
//!   types and their display constants belong to
//! - (´eq:rendering:kernel´) — the resonance kernel and its normalisation
//! - (´ex:rendering:examples´) — the golden tag renderings of the four worked
//!   landscapes

// Items in this module are used by tests and Layer 5.

use crate::numerics::stable_logit;

// ═══════════════════════════════════════════════════════════════════════════════
// Tag Enum
// ═══════════════════════════════════════════════════════════════════════════════

/// Resonance tag variant (7 tags: 3 classification + 4 action).
///
/// Classification tags express the system's belief about the request's nature.
/// Action tags recommend operational responses at different posture levels.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Tag {
    // ── Classification ──
    /// Benign request.
    Good,
    /// Uncertain — possibly adverse.
    Suspicious,
    /// Likely adverse.
    Malicious,
    // ── Action ──
    /// Permit without friction.
    Allow,
    /// Interstitial verification.
    Challenge,
    /// Throttle / rate-limit.
    Slow,
    /// Deny outright.
    Block,
}

impl Tag {
    /// Returns `true` for classification tags (Good, Suspicious, Malicious).
    #[must_use]
    pub const fn is_classification(self) -> bool {
        matches!(self, Self::Good | Self::Suspicious | Self::Malicious)
    }

    /// Returns `true` for action tags (Allow, Challenge, Slow, Block).
    #[must_use]
    pub const fn is_action(self) -> bool {
        !self.is_classification()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TagResonance
// ═══════════════════════════════════════════════════════════════════════════════

/// A resonance tag with location, magnitude, and Q-bandwidth.
///
/// Each tag defines a Cauchy kernel on the logit-transformed posture axis.
/// The set of all tags forms a proper probability distribution at every
/// posture via normalisation (´prop:rendering:properness´).
///
/// Serialisable so a stored profile can carry its tags beside the
/// configuration echo that makes it self-describing
/// (´sig:rendering:contract´).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct TagResonance {
    /// Tag variant.
    pub tag: Tag,
    /// Location on posture axis, `μ_a ∈ (0, 1)`.
    pub location: f64,
    /// Peak magnitude, `A_a > 0`.
    pub magnitude: f64,
    /// Q-bandwidth (certainty of placement), `Q_a > 0`.
    pub q: f64,
    /// Whether this tag is dominated.
    pub dominated: bool,
}

impl TagResonance {
    /// Returns `true` for classification tags.
    #[must_use]
    pub const fn is_classification(&self) -> bool {
        self.tag.is_classification()
    }

    /// Evaluates the Cauchy kernel at logit-space position `logit_pi`.
    #[must_use]
    pub fn kernel_at_logit(&self, logit_pi: f64) -> f64 {
        let logit_mu = stable_logit(self.location);
        cauchy_kernel(self.magnitude, self.q, logit_pi, logit_mu)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Kernel Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Guard threshold: when $Q^2 d^2 > 10^{15}$, return $\varepsilon$ directly.
///
/// The kernel of (´eq:rendering:kernel´) is evaluated as a reciprocal, so
/// beyond this magnitude the denominator carries no information the quotient
/// can recover. The cutoff stands one decimal order below the point at
/// which the unit addend has no representation in the sum at all, which
/// keeps the guard on the side where the denominator is still exact
/// (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:kernel-saturation-cutoff´ (´alg:const:scalar´)
/// ´const:assayer:kernel-saturation-cutoff-scalar-1e15´
const KERNEL_GUARD_THRESHOLD: f64 = 1e15;

/// Minimum kernel return value (prevents exact zero).
///
/// Every rendered tag carries positive probability at every posture
/// (´prop:rendering:completeness´), which a kernel returning an exact zero
/// would break. The floor is normal rather than merely positive, standing
/// about seven and a half decimal orders above the smallest normal double,
/// which is the headroom a later multiplication or division needs before it
/// underflows to the zero this exists to avoid
/// (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:tag-presence-floor´ (´alg:const:scalar´)
/// ´const:assayer:tag-presence-floor-scalar-1en300´
const KERNEL_EPSILON: f64 = 1e-300;

/// Cauchy kernel evaluation.
///
/// $$K = \frac{A}{1 + Q^2 (l_\pi - l_\mu)^2}$$
///
/// Guard: $Q^2 d^2 > 10^{15} \Rightarrow$ return $\varepsilon$ (not 0).
///
/// # Arguments
///
/// * `a` — Magnitude ($A$)
/// * `q` — Q-bandwidth ($Q$)
/// * `logit_pi` — Logit of posture ($\ell(\pi)$)
/// * `logit_mu` — Logit of tag location ($\ell(\mu)$)
#[must_use]
pub fn cauchy_kernel(a: f64, q: f64, logit_pi: f64, logit_mu: f64) -> f64 {
    let d = logit_pi - logit_mu;
    let q2d2 = q * q * d * d;

    if q2d2 > KERNEL_GUARD_THRESHOLD {
        return KERNEL_EPSILON;
    }

    a / (1.0 + q2d2)
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The seven tags fall into exactly two groups with nothing in either both or
    /// neither: Good, Suspicious and Malicious answer what the request *is*,
    /// while Allow, Challenge, Slow and Block answer what to *do* about it. The
    /// split is total, so code filtering the field by one predicate and code
    /// filtering by its negation between them see every tag exactly once.
    ///
    /// ´claim:resonance:the-seven-tags-split-cleanly-into-classification-and-action-with-nothing-in-both´
    /// ´test:unit:tag-classification-vs-action´
    #[test]
    fn tag_classification_vs_action() {
        assert!(Tag::Good.is_classification());
        assert!(Tag::Suspicious.is_classification());
        assert!(Tag::Malicious.is_classification());
        assert!(Tag::Allow.is_action());
        assert!(Tag::Challenge.is_action());
        assert!(Tag::Slow.is_action());
        assert!(Tag::Block.is_action());
    }

    /// Evaluated at its own location the kernel returns the magnitude exactly,
    /// with the bandwidth playing no part: the distance term vanishes and the
    /// denominator is one. Magnitude is therefore readable as the tag's peak
    /// height rather than as an abstract scaling coefficient, which is what makes
    /// the golden-value tables meaningful as a description of the field.
    ///
    /// ´claim:resonance:the-kernel-attains-exactly-its-magnitude-at-the-tags-own-location´
    /// ´test:unit:cauchy-kernel-at-peak´
    #[test]
    fn cauchy_kernel_at_peak() {
        let a = 0.946;
        let q = 2.381;
        let logit = 1.472;
        // At the peak: K = A / (1 + 0) = A
        let k = cauchy_kernel(a, q, logit, logit);
        assert!((k - a).abs() < 1e-14, "kernel at peak should equal A, got {k}");
    }

    /// Past the point where the squared distance term would overwhelm the
    /// arithmetic, the kernel stops dividing and hands back a fixed tiny
    /// constant instead — small enough to be negligible in any sum, but never
    /// zero. Normalising the field divides by the total of the kernels, so a tag
    /// that returned a true zero would make its own probability undefined rather
    /// than merely unlikely.
    ///
    /// ´claim:resonance:the-kernel-returns-a-tiny-positive-floor-instead-of-zero-in-the-far-tail´
    /// ´test:unit:cauchy-kernel-guard-fires´
    #[test]
    fn cauchy_kernel_guard_fires() {
        let a = 1.0;
        let q = 1e8;
        let d = 1e4;
        let k = cauchy_kernel(a, q, d, 0.0);
        assert!(k < 1e-100, "guard should return ε for huge Q²d², got {k}");
    }

    /// Equal steps either side of a tag's location give the same kernel value:
    /// only the squared distance enters, so the direction of the offset is
    /// invisible. A tag makes a claim about a point on the posture axis and how
    /// sharply it is held, and says nothing about which way an observation
    /// missed by.
    ///
    /// ´claim:resonance:the-kernel-depends-only-on-the-distance-from-the-location-not-its-direction´
    /// ´test:unit:cauchy-kernel-symmetry´
    #[test]
    fn cauchy_kernel_symmetry() {
        let a = 0.5;
        let q = 2.0;
        let mu = 1.0;
        let k_pos = cauchy_kernel(a, q, mu + 0.5, mu);
        let k_neg = cauchy_kernel(a, q, mu - 0.5, mu);
        assert!(
            (k_pos - k_neg).abs() < 1e-14,
            "kernel should be symmetric: {k_pos} vs {k_neg}"
        );
    }

    /// Asking a tag for its own kernel value gives exactly what calling the bare
    /// kernel with that tag's three fields would, the logit transform of the
    /// location included. The convenience method carries no extra behaviour of
    /// its own, so callers reasoning about the field in terms of the closed-form
    /// kernel are reasoning about what the method actually does.
    ///
    /// ´claim:resonance:evaluating-a-tags-kernel-through-its-own-method-matches-calling-the-kernel-directly´
    /// ´test:unit:kernel-at-logit-integrates´
    #[test]
    fn kernel_at_logit_integrates() {
        let tr = TagResonance {
            tag: Tag::Good,
            location: 0.814,
            magnitude: 0.946,
            q: 2.381,
            dominated: false,
        };
        let logit_pi = 1.5;
        let logit_mu = stable_logit(0.814);
        let direct = cauchy_kernel(0.946, 2.381, logit_pi, logit_mu);
        let via_method = tr.kernel_at_logit(logit_pi);
        assert!(
            (direct - via_method).abs() < 1e-14,
            "method should match direct: {direct} vs {via_method}"
        );
    }
}
