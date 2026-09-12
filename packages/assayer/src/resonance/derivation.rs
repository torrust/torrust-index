// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The resonance rendering: a layer over the decision landscape
//! (´dec:landscape:rendering-optional´).
//!
//! [`render_resonances()`] is a function of a landscape, a risk basis
//! and a display configuration, and of nothing else
//! (´sig:rendering:contract´). It carries no decision content the
//! landscape does not already hold: steps two through six of the fixed
//! computation order (´alg:landscape:computation-order´) — bandwidths,
//! magnitudes, interior and boundary placement, classification tags —
//! read the crossovers the landscape carries from step one.
//!
//! # Cross-References
//!
//! - (´app:spec:resonance-rendering´) — the rendering layer this module
//!   computes: kernel, placement, bandwidths and magnitudes
//! - (´sig:landscape:derivation-function´) — the derivation whose
//!   output this renders
//! - (´rule:derivation:closed-inputs´) — what the derivation layer
//!   receives and what may never be added to it

use std::collections::HashMap;

use super::ambiguity::{ProfileAmbiguity, compute_profile_ambiguity};
use super::channel::action_to_tag;
use super::landscape::{ActionCrossover, DecisionLandscape};
use super::tags::{Tag, TagResonance};
use crate::assessment::RiskBasis;
use crate::numerics::{stable_logit, stable_sigmoid};
pub use crate::types::ChallengeEstimate;

/// Input p̂ floor: prevents logit singularity at 0 and 1.
///
/// The clearance is a statement about evidence and not about arithmetic
/// — double precision would carry a probability far nearer the pole —
/// and what it fixes is that the logit handed on below never exceeds
/// about fourteen in magnitude (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:probability-pole-clearance´ (´alg:const:scalar´)
/// ´const:assayer:probability-pole-clearance-scalar-1en6´
pub const P_HAT_FLOOR: f64 = 1e-6;

/// Input `σ_eff` floor: prevents Q → ∞ and division by zero.
///
/// The classification bandwidth is a reciprocal of this quantity, so
/// the floor fixes the largest bandwidth the derivation will produce
/// rather than guarding a representation limit
/// (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:least-credible-uncertainty´ (´alg:const:scalar´)
/// ´const:assayer:least-credible-uncertainty-scalar-1en10´
pub const SIGMA_EFF_FLOOR: f64 = 1e-10;

/// The display threshold the reported domination probability is read
/// against: a tag whose action the landscape reports dominated with
/// probability above this is given the dominated display treatment
/// (´alg:rendering:dominated-treatment´). A named constant rather than
/// a ninth configuration field, because the rendering configuration is
/// specified as exactly eight constants (´tab:config:rendering´); the
/// discrepancy with the algorithm's "defaults to" phrasing is recorded
/// in wave ten's landing note.
///
/// ´const:assayer:domination-display-threshold´ (´alg:const:scalar´)
/// ´const:assayer:domination-display-threshold-scalar-0p5´
const DOMINATION_DISPLAY_THRESHOLD: f64 = 0.5;

/// Overflow-safe midpoint of two `f64` values.
fn midpoint(a: f64, b: f64) -> f64 {
    (b - a).mul_add(0.5, a)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration and Output Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for resonance rendering (8 presentation parameters).
///
/// The eight display constants the landscape deliberately excludes
/// (´tab:config:rendering´); they reach the rendering and nothing else
/// (´inv:landscape:presentation-free´).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ResonanceConfig {
    /// `c_Q`: Q scale factor for classification and action tags.
    pub c_q: f64,
    /// `c_A`: uncertainty attenuation coefficient (´def:rendering:magnitudes´).
    pub c_a: f64,
    /// `c_ℓ`: classification location compression (´def:rendering:magnitudes´).
    pub c_ell: f64,
    /// `c_susp`: suspicious tag magnitude coefficient (´def:rendering:magnitudes´).
    pub c_susp: f64,
    /// `c_{susp,Q}`: suspicious tag Q-bandwidth (´def:rendering:bandwidths´).
    pub c_susp_q: f64,
    /// `ε_mono`: dominated action magnitude (´alg:rendering:dominated-treatment´).
    pub epsilon_mono: f64,
    /// `Q_min`: dominated action Q-bandwidth (´alg:rendering:dominated-treatment´).
    pub q_min: f64,
    /// `Q_floor`: defensive Q floor (´def:rendering:bandwidths´).
    pub q_floor: f64,
}

impl Default for ResonanceConfig {
    fn default() -> Self {
        Self {
            c_q: 1.0,
            c_a: 10.0,
            c_ell: 0.5,
            c_susp: 0.5,
            c_susp_q: 0.5,
            epsilon_mono: 1e-6,
            q_min: 0.01,
            q_floor: 0.3,
        }
    }
}

/// A rendered resonance profile (´sig:rendering:contract´).
///
/// The profile is self-describing: it carries the posture domain its
/// locations live on and the configuration it was rendered under, so
/// rendering replay needs no external state.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ResonanceProfile {
    /// Per-tag resonances on the posture axis: three classification
    /// tags followed by the action tags, competing on one shared axis
    /// (´tab:rendering:spectrum´).
    pub tags: Vec<TagResonance>,
    /// The open posture interval the locations live on.
    pub posture_domain: (f64, f64),
    /// The display constants this profile was rendered under
    /// (´sig:rendering:contract´).
    pub config_echo: ResonanceConfig,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Public API
// ═══════════════════════════════════════════════════════════════════════════════

/// Renders a resonance profile from a landscape, a risk basis and a
/// display configuration (´sig:rendering:contract´).
///
/// Optional, and the landscape is complete without it
/// (´dec:landscape:rendering-optional´): the rendered tag probabilities
/// are normalised kernel evaluations under display constants, not
/// posterior probabilities of any event, so all decision semantics stay
/// on the landscape. Pure infallible function: no state, no side
/// effects, no `Result`.
///
/// The risk basis supplies the classification family — placement,
/// magnitude and bandwidth read the probability and its uncertainty
/// (´def:rendering:magnitudes´) — and must be the basis the landscape
/// was derived from for the two families to describe one assessment.
#[must_use]
pub fn render_resonances(landscape: &DecisionLandscape, risk: &RiskBasis, config: &ResonanceConfig) -> ResonanceProfile {
    // Defensive clamps shared with the derivation's own reading of the
    // basis (´const:assayer:probability-pole-clearance´): the rendering
    // has no error channel, so every edge its arithmetic can reach must
    // carry a defined value (´dec:derivation:pure-transform´).
    let p_hat = risk.p_bad.clamp(P_HAT_FLOOR, 1.0 - P_HAT_FLOOR);
    let sigma_eff = risk.sigma_eff.max(SIGMA_EFF_FLOOR);
    let kappa_eff = risk.kappa_eff;

    let crossovers = &landscape.crossovers;
    let n = landscape.regimes.len();
    debug_assert!(n >= 2, "channel must have at least 2 actions");

    // ── Step 2: Q-bandwidths (´def:rendering:bandwidths´) ──
    let q_class = config.c_q * kappa_eff / sigma_eff;
    let q_actions = compute_action_q(crossovers, config, n);

    // ── Step 3: Magnitudes (´def:rendering:magnitudes´) ──
    let alpha = compute_attenuation(p_hat, sigma_eff, kappa_eff, config);
    let action_magnitudes = compute_action_magnitudes(crossovers, alpha, n);

    // ── Steps 4–5: Action tag locations (´alg:rendering:placement´) ──
    let action_tags = compute_action_tags(landscape, &q_actions, &action_magnitudes, config, n);

    // ── Step 6: Classification tags (´def:rendering:magnitudes´) ──
    let class_tags = compute_classification_tags(p_hat, alpha, q_class, config);

    // Assemble: classification first, then actions.
    let mut tags = Vec::with_capacity(3 + n);
    tags.extend(class_tags);
    tags.extend(action_tags);

    ResonanceProfile {
        tags,
        posture_domain: (0.0, 1.0),
        config_echo: config.clone(),
    }
}

/// Normalised tag probabilities at a posture
/// (´sig:rendering:ambiguity-gauge´).
///
/// Applies the kernel and normalises (´eq:rendering:kernel´). These are
/// display quantities: a consumer wanting probabilities within one
/// family renormalises within that family.
#[must_use]
pub fn tag_probabilities(profile: &ResonanceProfile, posture: f64) -> HashMap<Tag, f64> {
    let posture = posture.clamp(0.001, 0.999);
    let logit_pi = stable_logit(posture);
    let kernels: Vec<f64> = profile.tags.iter().map(|t| t.kernel_at_logit(logit_pi)).collect();
    let total: f64 = kernels.iter().sum();
    profile
        .tags
        .iter()
        .zip(kernels.iter())
        .map(|(t, &k)| (t.tag, if total > 0.0 { k / total } else { 0.0 }))
        .collect()
}

/// The ambiguity gauge of a rendered profile, read at a caller-chosen
/// posture (´sig:rendering:ambiguity-gauge´).
///
/// A display gauge over the rendered tag distribution, not a value of
/// information; decision-aware exploration reads fragility on the
/// landscape instead (´def:fragility:definition´).
#[must_use]
pub fn profile_ambiguity(profile: &ResonanceProfile, posture: f64) -> ProfileAmbiguity {
    compute_profile_ambiguity(&profile.tags, posture)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Step 2: Q-Bandwidths (´def:rendering:bandwidths´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Action Q from the landscape's crossover variances
/// (´def:rendering:bandwidths´).
fn compute_action_q(crossovers: &[ActionCrossover], config: &ResonanceConfig, n: usize) -> Vec<f64> {
    (0..n)
        .map(|j| {
            let raw = if j == 0 {
                // First (boundary): Q = c_Q / sqrt(2·σ²₁)
                config.c_q / (2.0 * crossovers[0].variance).sqrt()
            } else if j == n - 1 {
                // Last (boundary): Q = c_Q / sqrt(2·σ²_{n-1})
                config.c_q / (2.0 * crossovers[n - 2].variance).sqrt()
            } else {
                // Interior: Q = c_Q / sqrt(σ²_{j-1} + σ²_j)
                let var_sum = crossovers[j - 1].variance + crossovers[j].variance;
                config.c_q / var_sum.sqrt()
            };
            // Q floor (´def:rendering:bandwidths´)
            raw.max(config.q_floor)
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Step 3: Magnitudes (´def:rendering:magnitudes´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Uncertainty attenuation: `α = 1 / (1 + c_A · σ²_p̂)`.
fn compute_attenuation(p_hat: f64, sigma_eff: f64, kappa_eff: f64, config: &ResonanceConfig) -> f64 {
    let sigma_p_hat = p_hat * (1.0 - p_hat) * sigma_eff / kappa_eff;
    1.0 / (sigma_p_hat * sigma_p_hat).mul_add(config.c_a, 1.0)
}

/// Action magnitudes from regime widths and attenuation.
fn compute_action_magnitudes(crossovers: &[ActionCrossover], alpha: f64, n: usize) -> Vec<f64> {
    (0..n)
        .map(|j| {
            let lo = if j == 0 { 0.0 } else { crossovers[j - 1].posture };
            let hi = if j == n - 1 { 1.0 } else { crossovers[j].posture };
            (hi - lo).max(0.0) * alpha
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Steps 4–5: Action Tag Locations (´alg:rendering:placement´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes action tag locations (´alg:rendering:placement´).
fn compute_action_tags(
    landscape: &DecisionLandscape,
    q_actions: &[f64],
    magnitudes: &[f64],
    config: &ResonanceConfig,
    n: usize,
) -> Vec<TagResonance> {
    let crossovers = &landscape.crossovers;

    // Special case: 2-action channel — both tags are boundary tags sharing one
    // crossover (´alg:rendering:placement´)
    if n == 2 {
        return compute_two_action_tags(landscape, q_actions, magnitudes, config);
    }

    let mut tags: Vec<TagResonance> = (0..n)
        .map(|j| {
            let tag = action_to_tag(landscape.regimes[j].action);
            // Trigger one of the dominated treatment: the landscape's
            // reported domination probability, read against the display
            // threshold (´alg:rendering:dominated-treatment´). Triggers
            // two and three — inversion and existence failure — arrive
            // through the magnitude and the placement below.
            let reported = landscape.regimes[j].domination_probability > DOMINATION_DISPLAY_THRESHOLD;
            let (location, dominated) = if j == 0 {
                let loc = boundary_location(j, 1, crossovers, q_actions, magnitudes, n, true);
                (loc.0.clamp(0.01, 0.49), loc.1)
            } else if j == n - 1 {
                let loc = boundary_location(j, n - 2, crossovers, q_actions, magnitudes, n, false);
                (loc.0.clamp(0.51, 0.99), loc.1)
            } else {
                // Interior: midpoint of adjacent crossovers (´alg:rendering:placement´)
                let logit = midpoint(crossovers[j - 1].logit, crossovers[j].logit);
                let loc = if logit.is_finite() { stable_sigmoid(logit) } else { 0.5 };
                (loc, false)
            };

            TagResonance {
                tag,
                location,
                magnitude: magnitudes[j],
                q: q_actions[j],
                dominated: dominated || reported,
            }
        })
        .collect();

    apply_monotonicity(&mut tags, config);
    tags
}

/// Two-action channel: Q-matched half-power placement (´alg:rendering:placement´).
fn compute_two_action_tags(
    landscape: &DecisionLandscape,
    q_actions: &[f64],
    magnitudes: &[f64],
    config: &ResonanceConfig,
) -> Vec<TagResonance> {
    let c = &landscape.crossovers[0];
    let logit_first = c.logit - 1.0 / q_actions[0];
    let logit_last = c.logit + 1.0 / q_actions[1];

    // Trigger one applies to the two boundary tags exactly as it does
    // in the general case (´alg:rendering:dominated-treatment´).
    let mut tags = vec![
        TagResonance {
            tag: action_to_tag(landscape.regimes[0].action),
            location: stable_sigmoid(logit_first).clamp(0.01, 0.49),
            magnitude: magnitudes[0],
            q: q_actions[0],
            dominated: landscape.regimes[0].domination_probability > DOMINATION_DISPLAY_THRESHOLD,
        },
        TagResonance {
            tag: action_to_tag(landscape.regimes[1].action),
            location: stable_sigmoid(logit_last).clamp(0.51, 0.99),
            magnitude: magnitudes[1],
            q: q_actions[1],
            dominated: landscape.regimes[1].domination_probability > DOMINATION_DISPLAY_THRESHOLD,
        },
    ];

    apply_monotonicity(&mut tags, config);
    tags
}

/// Crossover-matching formula for boundary actions (´alg:rendering:placement´).
///
/// Returns `(location, dominated)`.
fn boundary_location(
    boundary_idx: usize,
    interior_idx: usize,
    crossovers: &[ActionCrossover],
    q_actions: &[f64],
    magnitudes: &[f64],
    n: usize,
    is_first: bool,
) -> (f64, bool) {
    // Crossover between boundary and adjacent interior
    let c_idx = if is_first { 0 } else { n - 2 };
    let c = &crossovers[c_idx];

    if !c.logit.is_finite() {
        return if is_first { (0.25, true) } else { (0.75, true) };
    }

    let a_boundary = magnitudes[boundary_idx];
    let a_interior = magnitudes[interior_idx];
    let q_boundary = q_actions[boundary_idx];
    let q_interior = q_actions[interior_idx];

    // Interior half-width: distance from this crossover to interior midpoint
    let w_half = if interior_idx > 0 && interior_idx < n - 1 {
        let lo_cross = &crossovers[interior_idx - 1];
        let hi_cross = &crossovers[interior_idx];
        let logit_mid = midpoint(lo_cross.logit, hi_cross.logit);
        (logit_mid - c.logit).abs()
    } else {
        0.5 // Fallback
    };

    if a_interior <= 1e-20 {
        return (stable_sigmoid(c.logit), true);
    }

    // ρ = (A_a/A_b)·(1 + Q_b²·w²) - 1
    let qw = q_interior * w_half;
    let rho = (a_boundary / a_interior).mul_add(qw.mul_add(qw, 1.0), -1.0);

    if rho < 0.0 {
        // Existence failure: no offset satisfies the matching condition
        // (´alg:rendering:placement´), so the tag is treated as dominated
        // (´alg:rendering:dominated-treatment´).
        return (stable_sigmoid(c.logit), true);
    }

    // d = sqrt(ρ) / Q_a
    let d = rho.sqrt() / q_boundary;

    let logit_mu = if is_first { c.logit - d } else { c.logit + d };

    (stable_sigmoid(logit_mu), false)
}

/// Monotonicity enforcement (´alg:rendering:dominated-treatment´). Max 2 iterations.
fn apply_monotonicity(tags: &mut [TagResonance], config: &ResonanceConfig) {
    for _iter in 0..2 {
        let mut changed = false;

        // Zero-magnitude and boundary existence failure → dominated
        // (´alg:rendering:dominated-treatment´).
        for tag in tags.iter_mut() {
            if tag.magnitude <= 0.0 || tag.dominated {
                let changed_this_tag = !tag.dominated
                    || tag.location.to_bits() != 0.5_f64.to_bits()
                    || tag.magnitude.to_bits() != config.epsilon_mono.to_bits()
                    || tag.q.to_bits() != config.q_min.to_bits();
                tag.location = 0.5;
                tag.magnitude = config.epsilon_mono;
                tag.q = config.q_min;
                tag.dominated = true;
                changed |= changed_this_tag;
            }
        }

        if !changed {
            break;
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Step 6: Classification Tags (´def:rendering:magnitudes´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Classification tags from `p̂` and `σ_eff` (´def:rendering:magnitudes´).
fn compute_classification_tags(p_hat: f64, alpha: f64, q_gm: f64, config: &ResonanceConfig) -> [TagResonance; 3] {
    let p = p_hat.clamp(1e-10, 1.0 - 1e-10);
    let logit_p = stable_logit(p);

    // Locations (´def:rendering:magnitudes´)
    let mu_good = stable_sigmoid(-config.c_ell * logit_p);
    let mu_malicious = stable_sigmoid(config.c_ell * logit_p);

    // Magnitudes (´def:rendering:magnitudes´)
    let a_good = (1.0 - p_hat) * alpha;
    let a_suspicious = config.c_susp * 4.0 * p_hat * (1.0 - p_hat) * alpha;
    let a_malicious = p_hat * alpha;

    [
        TagResonance {
            tag: Tag::Good,
            location: mu_good,
            magnitude: a_good,
            q: q_gm,
            dominated: false,
        },
        TagResonance {
            tag: Tag::Suspicious,
            location: 0.5,
            magnitude: a_suspicious,
            q: config.c_susp_q,
            dominated: false,
        },
        TagResonance {
            tag: Tag::Malicious,
            location: mu_malicious,
            magnitude: a_malicious,
            q: q_gm,
            dominated: false,
        },
    ]
}
