// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The decision landscape: the presentation-free output of the
//! Derivation Function (´sig:landscape:output´).
//!
//! The derivation is a function of exactly three arguments — a risk
//! assessment, a channel policy and a challenge posterior — returning
//! one decision landscape (´sig:landscape:derivation-function´). Every
//! field is a function of those three inputs alone; no display constant
//! reaches any of them (´inv:landscape:presentation-free´). The
//! resonance rendering is a separate, optional layer over this object
//! (´dec:landscape:rendering-optional´).
//!
//! # Cross-References
//!
//! - (´chap:spec:derivation-interface´) — the interface this module is
//! - (´chap:spec:crossover-landscape´) — the closed-form derivation of
//!   every quantity carried here
//! - (´eq:landscape:rigid-decomposition´) — the shared term and the
//!   per-transition offsets
//! - (´def:landscape:credible-intervals´) — the quantile push-through
//!   the intervals carry
//!
//! Two fields exist so the landscape utilities can be computed from a
//! landscape and a posture alone (´sig:landscape:utilities´): the
//! exponent sum on the landscape and the two reward differentials on
//! each crossover. The optimal-action envelope
//! (´alg:landscape:optimal-action´) is not recoverable from the offsets
//! — they carry only differential ratios, and the envelope's skipped-line
//! breakpoints need the magnitudes — so the differentials travel with
//! the record. Both are functions of the declared policy and the
//! posterior, so the presentation-free invariant is untouched.

use super::channel::{ChannelPolicy, compute_derived_constants};
use super::derivation::{P_HAT_FLOOR, SIGMA_EFF_FLOOR};
use crate::assessment::{RiskAssessment, RiskBasis};
use crate::numerics::{beta_quantile, reg_inc_beta, stable_logit, stable_sigmoid};
use crate::types::{Action, ChallengePosteriorInput};

/// `z_{0.975}`: the two-sided ninety-five per cent normal quantile the
/// credible intervals compose with (´def:landscape:credible-intervals´).
///
/// ´const:assayer:landscape-interval-normal-quantile´ (´alg:const:scalar´)
/// ´const:assayer:landscape-interval-normal-quantile-scalar-1p96´
const Z_975: f64 = 1.96;

/// Sensitivity guard: below this differential the challenge-sensitivity
/// quotient is not evaluated, matching the kernel's variance guard for
/// non-positive differentials (´thm:landscape:catching-forfeiture´).
///
/// ´const:assayer:landscape-differential-floor´ (´alg:const:scalar´)
/// ´const:assayer:landscape-differential-floor-scalar-1en20´
const DELTA_BAD_FLOOR: f64 = 1e-20;

// ═══════════════════════════════════════════════════════════════════════════════
// Output Types
// ═══════════════════════════════════════════════════════════════════════════════

/// One crossover record per adjacent pair of declared actions
/// (´def:landscape:action-crossover´).
///
/// Location is reported on both scales (´eq:landscape:crossover´): the
/// logit scale is where the arithmetic is linear and the uncertainty is
/// Gaussian, the posture scale is where the host's cursor lives. Under
/// dominance a crossover may be inverted relative to its neighbours and
/// the record is reported unchanged.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ActionCrossover {
    /// The less restrictive action of the adjacent pair.
    pub from: Action,
    /// The more restrictive action of the adjacent pair.
    pub to: Action,
    /// The crossover's logit: the shared term plus this transition's
    /// offset. Infinite where the transition never pays
    /// (´thm:landscape:catching-forfeiture´).
    pub logit: f64,
    /// The logistic image of the logit, in the closed unit interval.
    pub posture: f64,
    /// This transition's offset at the point estimate
    /// (´eq:landscape:rigid-decomposition´).
    pub offset: f64,
    /// `∂ℓ*/∂q̂_c`: the signed derivative of the location in the
    /// challenge estimate.
    pub sensitivity: f64,
    /// `Δ_good`: the transition's benign-class differential
    /// (´def:landscape:reward-differentials´). Carried so the envelope
    /// is computable from the landscape (´alg:landscape:optimal-action´).
    pub delta_good: f64,
    /// `Δ_bad` at the point estimate: the transition's adverse-class
    /// differential (´def:landscape:reward-differentials´).
    pub delta_bad: f64,
    /// The two-source variance at this crossover
    /// (´thm:landscape:crossover-covariance´).
    pub variance: f64,
    /// The ninety-five per cent credible interval on the logit scale
    /// (´def:landscape:credible-intervals´).
    pub interval_logit: (f64, f64),
    /// The same interval mapped through the logistic — exact, because
    /// an interval transformed after computation is exact
    /// (´eq:landscape:crossover´).
    pub interval_posture: (f64, f64),
}

/// One regime record per declared action (´sig:landscape:output´).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct ActionRegime {
    /// The declared action this regime belongs to.
    pub action: Action,
    /// The regime's logit width where the action is interior — the
    /// difference of its two bounding offsets, negative under
    /// inversion — and `None` for the two boundary actions, whose
    /// regimes are half-lines (´thm:landscape:rigid-translation´).
    pub width: Option<f64>,
    /// The width's variance (´prop:landscape:width-variance´): the
    /// shared term cancels identically, so this carries challenge
    /// variance alone. `None` where the width is.
    pub width_variance: Option<f64>,
    /// The probability that this action is dominated, under the
    /// challenge posterior (´thm:landscape:dominance´): every
    /// domination condition reduces to a threshold on the challenge
    /// effectiveness alone, and the probability is the posterior mass
    /// on the dominated side — exact for the conjugate variant,
    /// evaluated on the moment-matched Beta otherwise. Reported and
    /// never structurally enforced.
    pub domination_probability: f64,
}

/// The decision landscape: the complete decision content of one
/// assessment on one channel (´sig:landscape:output´).
///
/// The crossover vector has fixed shape — exactly one entry fewer than
/// the declared actions, whatever the evidence says about dominance —
/// which is what makes a landscape safe to store, to replay, and to
/// difference across a policy comparison.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct DecisionLandscape {
    /// Shared term `u`: the risk-driven crossover location, common to
    /// every transition (´eq:landscape:rigid-decomposition´).
    pub u: f64,
    /// Shared uncertainty `σ_u`: the single scalar carrying all
    /// risk-driven uncertainty (´thm:landscape:rigid-translation´).
    pub sigma_u: f64,
    /// `β_b + β_g`: the declared exponent sum. Carried so a consumer
    /// can reconstruct the cost-curve envelope from the landscape alone
    /// (´alg:landscape:optimal-action´).
    pub beta_sum: f64,
    /// The challenge posterior as it crossed the boundary
    /// (´sig:companion:posterior´): conjugate where the Companion's
    /// structure survived the crossing, moment-matched where the
    /// estimate came from somewhere else.
    pub posterior: ChallengePosteriorInput,
    /// One crossover per adjacent pair of declared actions
    /// (´def:landscape:action-crossover´).
    pub crossovers: Vec<ActionCrossover>,
    /// One regime per declared action.
    pub regimes: Vec<ActionRegime>,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derivation
// ═══════════════════════════════════════════════════════════════════════════════

/// Derives the decision landscape from an assessment, a channel policy
/// and a challenge posterior (´sig:landscape:derivation-function´).
///
/// Reads the risk basis and ignores the rest of the assessment; the
/// remaining fields travel in the argument because a host holds one
/// assessment and derives from it, not because the transform consumes
/// them. Pure, infallible, deterministic: no state read or written, and
/// the same three inputs produce the same landscape on every machine
/// and at every time (´pf:landscape:purity´).
///
/// The declaration is assumed well formed and is not checked here: the
/// check is the host's, at policy load, through
/// [`validate_channel_policy`](crate::validate_channel_policy)
/// (´dec:derivation:ordered-actions´).
///
/// The posterior crosses in either of its two variants
/// (´sig:companion:posterior´): the Companion's conjugate pseudo-count
/// pair, whose structure survives into the landscape's quantile and
/// dominance arithmetic exactly, or a bare moment pair from elsewhere,
/// admitted at moment-matched fidelity — a
/// [`ChallengeEstimate`](crate::types::ChallengeEstimate) converts
/// into the moment-matched variant.
#[must_use]
pub fn derive_landscape(
    assessment: &RiskAssessment,
    channel_policy: &ChannelPolicy,
    challenge_posterior: impl Into<ChallengePosteriorInput>,
) -> DecisionLandscape {
    derive_landscape_from_basis(&assessment.risk, channel_policy, challenge_posterior)
}

/// The landscape from the risk basis alone — the part of the assessment
/// the derivation reads (´schema:risk:basis´).
pub fn derive_landscape_from_basis(
    risk: &RiskBasis,
    channel_policy: &ChannelPolicy,
    challenge_posterior: impl Into<ChallengePosteriorInput>,
) -> DecisionLandscape {
    let challenge_posterior = challenge_posterior.into();
    let q_c = challenge_posterior.q_c();
    let sigma2_q = challenge_posterior.variance();

    let derived = compute_derived_constants(channel_policy, q_c);
    let beta_sum = derived.beta_sum;
    let n = derived.actions.len();

    // Evidence group (´eq:landscape:rigid-decomposition´): the shared
    // term and its one scalar of risk-driven uncertainty. The pole
    // clearance is the kernel's own (´const:assayer:probability-pole-clearance´).
    let p = risk.p_bad.clamp(P_HAT_FLOOR, 1.0 - P_HAT_FLOOR);
    let u = -stable_logit(p) / beta_sum;
    let sigma_u = risk.sigma_eff.max(SIGMA_EFF_FLOOR) / (beta_sum * risk.kappa_eff);

    // The posterior's ninety-five per cent quantile pair, for the
    // interval push-through (´def:landscape:credible-intervals´): exact
    // on the conjugate variant's own shape, moment-matched on the other
    // (´sig:companion:posterior´).
    let (alpha, beta) = challenge_posterior.beta_shape();
    let q_lo = beta_quantile(alpha, beta, 0.025);
    let q_hi = beta_quantile(alpha, beta, 0.975);

    let crossovers: Vec<ActionCrossover> = derived
        .differentials
        .iter()
        .enumerate()
        .map(|(j, diff)| {
            let offset = offset_at(diff.delta_good, diff.delta_bad, beta_sum);
            let logit = u + offset;
            let posture = saturating_sigmoid(logit);

            // Signed sensitivity `s = −Δ_bad′ / (β_sum · Δ_bad)`
            // (´eq:landscape:rigid-decomposition´), guarded like the
            // kernel's variance quotient where the differential has
            // vanished (´thm:landscape:catching-forfeiture´).
            let sensitivity = if diff.delta_bad > DELTA_BAD_FLOOR {
                -diff.d_delta_bad_d_q / (beta_sum * diff.delta_bad)
            } else {
                0.0
            };

            // Two-source variance (´thm:landscape:crossover-covariance´).
            let variance = (sensitivity * sensitivity).mul_add(sigma2_q, sigma_u * sigma_u);

            // Quantile push-through (´def:landscape:credible-intervals´):
            // the offset is evaluated exactly at the posterior's two
            // quantiles — the differential is affine in the estimate —
            // and the risk term is unioned on afterward.
            let offset_lo = offset_at_estimate(diff.delta_good, diff.delta_bad, diff.d_delta_bad_d_q, q_c, q_lo, beta_sum);
            let offset_hi = offset_at_estimate(diff.delta_good, diff.delta_bad, diff.d_delta_bad_d_q, q_c, q_hi, beta_sum);
            let (b_min, b_max) = if offset_lo <= offset_hi {
                (offset_lo, offset_hi)
            } else {
                (offset_hi, offset_lo)
            };
            let interval_logit = (Z_975.mul_add(-sigma_u, u + b_min), Z_975.mul_add(sigma_u, u + b_max));
            let interval_posture = (saturating_sigmoid(interval_logit.0), saturating_sigmoid(interval_logit.1));

            ActionCrossover {
                from: derived.actions[j],
                to: derived.actions[j + 1],
                logit,
                posture,
                offset,
                sensitivity,
                delta_good: diff.delta_good,
                delta_bad: diff.delta_bad,
                variance,
                interval_logit,
                interval_posture,
            }
        })
        .collect();

    let regimes: Vec<ActionRegime> = (0..n)
        .map(|j| {
            let (width, width_variance) = if j == 0 || j == n - 1 {
                (None, None)
            } else {
                // Interior width: the difference of adjacent offsets;
                // the shared term cancels identically
                // (´prop:landscape:width-variance´).
                let width = crossovers[j].offset - crossovers[j - 1].offset;
                let ds = crossovers[j].sensitivity - crossovers[j - 1].sensitivity;
                (Some(width), Some(ds * ds * sigma2_q))
            };
            ActionRegime {
                action: derived.actions[j],
                width,
                width_variance,
                domination_probability: domination_probability(&derived.differentials, j, n, q_c, alpha, beta),
            }
        })
        .collect();

    DecisionLandscape {
        u,
        sigma_u,
        beta_sum,
        posterior: challenge_posterior,
        crossovers,
        regimes,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// The probability that action `j` is dominated, as the posterior mass
/// on the dominated side of its challenge-effectiveness threshold
/// (´thm:landscape:dominance´).
///
/// Every domination condition is a threshold on the effectiveness
/// alone, because regime widths are risk-independent
/// (´thm:landscape:rigid-translation´). Each adverse differential is
/// affine in the estimate, so the conditions reduce to one affine
/// inequality per action: an interior action is dominated where its
/// width goes non-positive — the cross-multiplied form
/// `Δg_{j−1}·Δb_j(q) ≤ Δg_j·Δb_{j−1}(q)`, which also covers an entry
/// crossover sent to infinity — the first action where its single
/// bounding transition stops paying for benign traffic, and the last
/// where its adverse differential forfeits
/// (´thm:landscape:catching-forfeiture´). The mass is the regularised
/// incomplete beta at the threshold, on the posterior's own shape.
fn domination_probability(
    differentials: &[super::channel::RewardDifferential],
    j: usize,
    n: usize,
    q_hat: f64,
    alpha: f64,
    beta: f64,
) -> f64 {
    // The posterior mass at or below a threshold on the effectiveness.
    let mass_below = |theta: f64| reg_inc_beta(alpha, beta, theta);

    // An affine condition `h0 + h1·q ≥ 0` marks dominance; the mass on
    // the satisfying side is the probability.
    let affine_mass = |h0: f64, h1: f64| -> f64 {
        if h1 == 0.0 {
            return if h0 >= 0.0 { 1.0 } else { 0.0 };
        }
        let theta = -h0 / h1;
        if h1 > 0.0 {
            1.0 - mass_below(theta)
        } else {
            mass_below(theta)
        }
    };

    if j == 0 {
        // The least restrictive action is dominated only where its one
        // bounding transition costs benign traffic nothing or less, a
        // condition the estimate does not enter.
        let d = &differentials[0];
        return if d.delta_good <= 0.0 { 1.0 } else { 0.0 };
    }
    if j == n - 1 {
        // The most restrictive action is dominated where its adverse
        // differential is non-positive: `Δb(q) = Δb(q̂) + d·(q − q̂) ≤ 0`.
        let d = &differentials[n - 2];
        return affine_mass(-d.d_delta_bad_d_q.mul_add(-q_hat, d.delta_bad), -d.d_delta_bad_d_q);
    }

    // Interior: dominated where the width is non-positive. With
    // `Δb_i(0) = Δb_i(q̂) − d_i·q̂`, the cross-multiplied condition is
    // affine with the coefficients below.
    let lo = &differentials[j - 1];
    let hi = &differentials[j];
    let hi_at_zero = hi.d_delta_bad_d_q.mul_add(-q_hat, hi.delta_bad);
    let lo_at_zero = lo.d_delta_bad_d_q.mul_add(-q_hat, lo.delta_bad);
    let h0 = lo.delta_good.mul_add(hi_at_zero, -(hi.delta_good * lo_at_zero));
    let h1 = lo
        .delta_good
        .mul_add(hi.d_delta_bad_d_q, -(hi.delta_good * lo.d_delta_bad_d_q));
    affine_mass(h0, h1)
}

/// The transition offset `b = ln(Δ_good / Δ_bad) / β_sum`, with the
/// kernel's sign conventions for vanished differentials: a non-positive
/// adverse differential sends the crossover to `+∞`
/// (´thm:landscape:catching-forfeiture´), a non-positive benign
/// differential to `−∞`.
fn offset_at(delta_good: f64, delta_bad: f64, beta_sum: f64) -> f64 {
    if delta_good > 0.0 && delta_bad > 0.0 {
        (delta_good / delta_bad).ln() / beta_sum
    } else if delta_bad <= 0.0 {
        f64::INFINITY
    } else {
        f64::NEG_INFINITY
    }
}

/// The offset re-evaluated at another challenge estimate: the adverse
/// differential is affine in the estimate with the carried derivative
/// as its slope, so the evaluation is exact rather than linearised
/// (´def:landscape:credible-intervals´).
fn offset_at_estimate(delta_good: f64, delta_bad: f64, d_delta_bad_d_q: f64, q_at: f64, q_to: f64, beta_sum: f64) -> f64 {
    let delta_bad_to = d_delta_bad_d_q.mul_add(q_to - q_at, delta_bad);
    offset_at(delta_good, delta_bad_to, beta_sum)
}

/// The logistic map extended to the saturated crossovers: an infinite
/// logit answers with the axis end it saturated toward.
fn saturating_sigmoid(logit: f64) -> f64 {
    if logit.is_finite() {
        stable_sigmoid(logit)
    } else if logit > 0.0 {
        1.0
    } else {
        0.0
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The Landscape Utilities (´sig:landscape:utilities´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A fragility reading of a landscape at a posture
/// (´def:fragility:definition´).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub struct DecisionFragility {
    /// The posture the reading was taken at.
    pub posture: f64,
    /// The action optimal at the point-estimate crossovers, recovered
    /// from the cost-curve envelope (´alg:landscape:optimal-action´).
    pub modal_action: Action,
    /// The probability that the true optimal action differs from the
    /// modal one.
    pub flip_probability: f64,
    /// The fraction of the bounding-crossover variance from the shared
    /// risk term.
    pub risk_share: f64,
    /// The fraction from the challenge posterior.
    pub challenge_share: f64,
}

/// The cumulative differential sums that make every action's expected
/// cost an affine function of `x = exp(β_sum · (ℓ − u))`: relative to
/// the least restrictive action, action `j` costs
/// `Σ_{i<j} Δ_good_i − x · Σ_{i<j} Δ_bad_i` up to one positive factor
/// shared by all actions (´thm:landscape:rigid-translation´).
fn cumulative_differentials(landscape: &DecisionLandscape) -> (Vec<f64>, Vec<f64>) {
    let mut a = vec![0.0];
    let mut b = vec![0.0];
    for c in &landscape.crossovers {
        a.push(a.last().unwrap() + c.delta_good);
        b.push(b.last().unwrap() + c.delta_bad);
    }
    (a, b)
}

/// The envelope's minimiser at `x`, ties broken toward the more
/// permissive action so the answer is left-continuous in the posture
/// (´alg:landscape:optimal-action´).
fn envelope_argmin(a: &[f64], b: &[f64], x: f64) -> usize {
    let mut best = 0;
    let mut best_cost = 0.0_f64;
    for j in 1..a.len() {
        let cost = b[j].mul_add(-x, a[j]);
        if cost < best_cost {
            best = j;
            best_cost = cost;
        }
    }
    best
}

/// The optimal action of a landscape at a posture
/// (´alg:landscape:optimal-action´).
///
/// Computed from the upper envelope of the per-action cost curves, not
/// by locating the bracketing crossover entries: under dominance the
/// bracketing shortcut returns a dominated action for an interval where
/// it is never optimal, and the envelope has no such failure mode. The
/// utility is total over the open unit interval and reads no model
/// state (´sig:landscape:utilities´).
#[must_use]
pub fn optimal_action(landscape: &DecisionLandscape, posture: f64) -> Action {
    let posture = posture.clamp(P_HAT_FLOOR, 1.0 - P_HAT_FLOOR);
    let x = (landscape.beta_sum * (stable_logit(posture) - landscape.u)).exp();
    let (a, b) = cumulative_differentials(landscape);
    landscape.regimes[envelope_argmin(&a, &b, x)].action
}

/// One boundary of the modal action's optimal regime, as the envelope
/// sees it: its logit, and the challenge sensitivity of the compound
/// crossing it stands at.
struct EnvelopeBoundary {
    /// The boundary's location on the logit scale.
    logit: f64,
    /// `∂ℓ*/∂q̂_c` of the compound crossing.
    sensitivity: f64,
}

/// The compound crossing between the modal action `m` and another
/// action `j`: the boundary the envelope hands over at, which for
/// non-adjacent pairs is a crossing no single crossover record carries.
/// Its location and sensitivity follow from the spanned differential
/// sums exactly as an adjacent crossover's do from its own.
fn compound_boundary(landscape: &DecisionLandscape, lo: usize, hi: usize) -> EnvelopeBoundary {
    let span = &landscape.crossovers[lo..hi];
    let sum_good: f64 = span.iter().map(|c| c.delta_good).sum();
    let sum_bad: f64 = span.iter().map(|c| c.delta_bad).sum();
    let logit = landscape.u + (sum_good / sum_bad).ln() / landscape.beta_sum;
    // The spanned sensitivity is the Δ_bad-weighted mean of the members'
    // (recovering each `dΔ_bad/dq` from `s = −d/(β_sum·Δ_bad)`).
    let weighted: f64 = span.iter().map(|c| c.sensitivity * c.delta_bad).sum();
    let sensitivity = weighted / sum_bad;
    EnvelopeBoundary { logit, sensitivity }
}

/// The decision fragility of a landscape at a posture
/// (´def:fragility:definition´).
///
/// The modal action comes from the cost-curve envelope, so a posture
/// inside a dominated action's inverted region is handled correctly,
/// and the boundaries used are the crossings bounding the *optimal*
/// regime from the envelope rather than the raw adjacent records. The
/// flip probability is the mass of the bounding crossing distributions
/// on the far side of the posture's logit, by the Gaussian
/// approximation on the marginals the definition prescribes as the
/// default, the two exceedances summed — they are disjoint up to that
/// approximation, since the boundaries bracket the posture — and capped
/// at one. The two shares split the bounding variance into its
/// risk-driven and challenge-driven parts.
#[must_use]
pub fn fragility(landscape: &DecisionLandscape, posture: f64) -> DecisionFragility {
    let clamped = posture.clamp(P_HAT_FLOOR, 1.0 - P_HAT_FLOOR);
    let logit_pi = stable_logit(clamped);
    let x = (landscape.beta_sum * (logit_pi - landscape.u)).exp();
    let (a, b) = cumulative_differentials(landscape);
    let m = envelope_argmin(&a, &b, x);

    // The envelope's bounding crossings: for each other action, the
    // affine difference `(A_j − A_m) − (B_j − B_m)·x` is non-negative at
    // `x` and crosses zero where that action takes over. A crossing
    // above `x` bounds the regime from above, one below from below; the
    // nearest of each kind is the regime boundary.
    let mut upper: Option<(f64, usize)> = None;
    let mut lower: Option<(f64, usize)> = None;
    for j in 0..a.len() {
        if j == m {
            continue;
        }
        let da = a[j] - a[m];
        let db = b[j] - b[m];
        if db == 0.0 {
            continue; // parallel: never crosses
        }
        let x_cross = da / db;
        if x_cross <= 0.0 {
            continue; // no crossing on the axis
        }
        if db > 0.0 {
            // Difference decreasing in x: `j` takes over above.
            if x_cross >= x && upper.is_none_or(|(best, _)| x_cross < best) {
                upper = Some((x_cross, j));
            }
        } else if x_cross <= x && lower.is_none_or(|(best, _)| x_cross > best) {
            // Difference increasing in x: `j` takes over below.
            lower = Some((x_cross, j));
        }
    }

    let bound = |cross: Option<(f64, usize)>| -> Option<EnvelopeBoundary> {
        cross.map(|(_, j)| {
            let (lo, hi) = if j > m { (m, j) } else { (j, m) };
            compound_boundary(landscape, lo, hi)
        })
    };
    let lower = bound(lower);
    let upper = bound(upper);

    let sigma2_u = landscape.sigma_u * landscape.sigma_u;
    let sigma2_q = landscape.posterior.variance();
    let boundary_variance =
        |boundary: &EnvelopeBoundary| (boundary.sensitivity * boundary.sensitivity).mul_add(sigma2_q, sigma2_u);

    let mut flip = 0.0;
    let mut total_variance = 0.0;
    let mut risk_variance = 0.0;
    for (boundary, sign) in [(&lower, 1.0), (&upper, -1.0)] {
        if let Some(boundary) = boundary {
            let variance = boundary_variance(boundary);
            // Lower boundary above the posture, or upper below it, flips
            // the action; the signed z-score reads the right tail.
            flip += crate::numerics::normal_cdf(sign * (boundary.logit - logit_pi) / variance.sqrt());
            total_variance += variance;
            risk_variance += sigma2_u;
        }
    }
    let (risk_share, challenge_share) = if total_variance > 0.0 {
        let risk = risk_variance / total_variance;
        (risk, 1.0 - risk)
    } else {
        (0.0, 0.0)
    };

    DecisionFragility {
        posture,
        modal_action: landscape.regimes[m].action,
        flip_probability: flip.min(1.0),
        risk_share,
        challenge_share,
    }
}
