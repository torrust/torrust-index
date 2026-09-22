// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Numerical utilities for the Assayer crate.
//!
//! This module centralises decay computations to ensure consistent behaviour
//! across the codebase. All decay operations use these functions; direct `powf`
//! calls with decay rates are prohibited elsewhere (enforced via CI lint).
//!
//! # Decay Model
//!
//! The Assayer uses exponential decay with per-hour decay factors:
//!
//! $$\text{weight}(t) = \gamma^{\Delta t}$$
//!
//! where $\gamma \in (0, 1)$ is the hourly retention rate and $\Delta t$ is
//! the elapsed time in hours.
//!
//! # Half-Life Reference (from specification)
//!
//! | Decay rate | Half-life |
//! |------------|-----------|
//! | γ = 0.999  | ~29 days  |
//! | γ = 0.9999 | ~290 days |
//!
//! # Cross-References
//!
//! - (´dec:clock:shared-functions´) — all decay flows through the shared
//!   functions this module holds
//! - (´tab:temporal:component-rates´) — the operational and sister model
//!   rates and the half-lives they imply

use std::time::Duration;

use crate::types::{MAX_DECAY_HOURS, PersistentTimestamp, duration_to_hours};

// ═══════════════════════════════════════════════════════════════════════════════
// Core Decay Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the decay factor for a given hourly rate and elapsed time.
///
/// Returns $\gamma^{\Delta t}$ where $\Delta t$ is in hours.
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate, should be in $(0, 1)$
/// * `dt_hours` — Elapsed time in hours (pre-clamped by caller)
///
/// # Returns
///
/// The decay factor, always in $(0, 1]$.
///
/// # Panics
///
/// Debug-asserts that `dt_hours` is non-negative and within `MAX_DECAY_HOURS`.
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::decay_factor;
///
/// // After 1 hour with γ = 0.999
/// let factor = decay_factor(0.999, 1.0);
/// assert!((factor - 0.999).abs() < 1e-10);
///
/// // After 0 hours, no decay
/// let factor = decay_factor(0.999, 0.0);
/// assert!((factor - 1.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn decay_factor(gamma: f64, dt_hours: f64) -> f64 {
    debug_assert!(dt_hours >= 0.0, "dt_hours must be non-negative, got {dt_hours}");
    debug_assert!(
        dt_hours <= MAX_DECAY_HOURS,
        "dt_hours exceeds MAX_DECAY_HOURS: {dt_hours} > {MAX_DECAY_HOURS}"
    );

    // Handle edge cases
    if dt_hours <= 0.0 {
        return 1.0;
    }

    gamma.powf(dt_hours)
}

/// Computes the decay factor between two persistent timestamps.
///
/// This is the primary decay function for lazy read/write decay patterns.
/// Handles backward clock jumps (returns 1.0) and enormous gaps (clamped).
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate
/// * `old` — Earlier timestamp
/// * `new` — Later timestamp
///
/// # Returns
///
/// The decay factor, always in $(0, 1]$.
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::decay_factor_since;
/// use torrust_assayer::types::PersistentTimestamp;
///
/// let t0 = PersistentTimestamp::new(0, 0);
/// let t1 = PersistentTimestamp::new(3600, 0); // 1 hour later
///
/// let factor = decay_factor_since(0.999, &t0, &t1);
/// assert!((factor - 0.999).abs() < 1e-10);
/// ```
#[must_use]
pub fn decay_factor_since(gamma: f64, old: &PersistentTimestamp, new: &PersistentTimestamp) -> f64 {
    let dt_hours = new.hours_since(old);
    decay_factor(gamma, dt_hours)
}

/// Computes the label-indexed decay factor over a count of labels.
///
/// Returns $\gamma^{n}$ for the per-label retention rate $\gamma$ and the
/// number of labels $n$ processed across the interval. The label clock is the
/// system's second clock, and a quantity aged across an interval is aged on
/// both (´tab:risk:forgetting-rates´); this is the label half, and it lives
/// here so that no caller reaches for `powf` on its own
/// (´dec:clock:shared-functions´).
///
/// Underflow to zero is a value rather than an error: a rate below one raised
/// to a large enough count is zero in binary64, and a caller that cannot use a
/// zero factor decides that for itself rather than being handed a clamped
/// figure it did not ask for.
///
/// # Arguments
///
/// * `gamma` — Per-label retention rate, in $(0, 1]$
/// * `labels` — Labels processed across the interval
///
/// # Returns
///
/// The decay factor, in $[0, 1]$.
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::label_decay_factor;
///
/// // No labels processed: nothing is forgotten.
/// assert!((label_decay_factor(0.9995, 0) - 1.0).abs() < 1e-12);
///
/// // One label at the operational rate.
/// assert!((label_decay_factor(0.9995, 1) - 0.9995).abs() < 1e-12);
/// ```
#[must_use]
#[allow(clippy::cast_precision_loss)] // Justified: label counts are far below f64's exact integer range in every deployment the rate is meaningful for.
pub fn label_decay_factor(gamma: f64, labels: u64) -> f64 {
    debug_assert!(gamma > 0.0 && gamma <= 1.0, "gamma must be in (0, 1], got {gamma}");
    if labels == 0 {
        return 1.0;
    }
    gamma.powf(labels as f64)
}

/// Computes the decay factor for an elapsed `Duration`.
///
/// This is used for model decay from `Instant` measurements during runtime.
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate
/// * `dur` — Elapsed duration
///
/// # Returns
///
/// The decay factor, always in $(0, 1]$.
///
/// # Examples
///
/// ```
/// use std::time::Duration;
/// use torrust_assayer::numerics_export::decay_factor_elapsed;
///
/// // 1 hour elapsed
/// let factor = decay_factor_elapsed(0.999, Duration::from_secs(3600));
/// assert!((factor - 0.999).abs() < 1e-10);
///
/// // Zero duration = no decay
/// let factor = decay_factor_elapsed(0.999, Duration::ZERO);
/// assert!((factor - 1.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn decay_factor_elapsed(gamma: f64, dur: Duration) -> f64 {
    let dt_hours = duration_to_hours(dur);
    decay_factor(gamma, dt_hours)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Derived Decay Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the effective sample count after decay.
///
/// Given a current effective count $n$ and decay factor $f$, returns $n \cdot f$.
///
/// # Arguments
///
/// * `count` — Current effective sample count
/// * `factor` — Decay factor from one of the decay functions
///
/// # Returns
///
/// The decayed sample count.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "test-support"), allow(dead_code))] // Justified: part of the numerics_export test surface
pub fn decay_count(count: f64, factor: f64) -> f64 {
    count * factor
}

/// Computes hours needed for the decay factor to reach a target value.
///
/// Solves $\gamma^t = \text{target}$ for $t$.
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate, must be in $(0, 1)$
/// * `target_factor` — Desired decay factor, must be in $(0, 1]$
///
/// # Returns
///
/// Hours until the target factor is reached.
///
/// # Panics
///
/// Debug-asserts valid input ranges.
#[must_use]
#[cfg_attr(not(feature = "test-support"), allow(dead_code))] // Justified: part of the numerics_export test surface
pub fn hours_to_decay_target(gamma: f64, target_factor: f64) -> f64 {
    debug_assert!(gamma > 0.0 && gamma < 1.0, "gamma must be in (0, 1), got {gamma}");
    debug_assert!(
        target_factor > 0.0 && target_factor <= 1.0,
        "target_factor must be in (0, 1], got {target_factor}"
    );

    // γ^t = target => t = log_γ(target)
    target_factor.log(gamma)
}

/// Computes the half-life in hours for a given decay rate.
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate, must be in $(0, 1)$
///
/// # Returns
///
/// Hours until the weight is halved.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "test-support"), allow(dead_code))] // Justified: part of the numerics_export test surface
pub fn half_life_hours(gamma: f64) -> f64 {
    hours_to_decay_target(gamma, 0.5)
}

/// Computes the half-life in days for a given decay rate.
///
/// # Arguments
///
/// * `gamma` — Hourly retention rate, must be in $(0, 1)$
///
/// # Returns
///
/// Days until the weight is halved.
#[must_use]
#[inline]
#[cfg_attr(not(feature = "test-support"), allow(dead_code))] // Justified: part of the numerics_export test surface
pub fn half_life_days(gamma: f64) -> f64 {
    half_life_hours(gamma) / 24.0
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shared Algorithm Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Numerically stable sigmoid: $\sigma(x) = 1 / (1 + e^{-x})$.
///
/// Two-branch implementation to avoid overflow:
/// - $x \ge 0$: $1 / (1 + e^{-x})$
/// - $x < 0$: $e^x / (1 + e^x)$
///
/// Shared by the resonance derivation, the Platt calibration fit, and the
/// blend weight. One definition, multiple imports.
///
/// # Cross-References
///
/// - (´dec:derivation:pure-transform´) — the derivation this total function
///   keeps free of an error channel
/// - (´dec:calibration:pure-refit´) — the Platt calibration fit
/// - (´dec:calibration:raw-weight-forms´) — the blend weight computation
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::stable_sigmoid;
///
/// // σ(0) = 0.5 exactly
/// assert!((stable_sigmoid(0.0) - 0.5).abs() < 1e-15);
///
/// // Large positive → near 1.0, no overflow
/// assert!((stable_sigmoid(700.0) - 1.0).abs() < 1e-10);
///
/// // Large negative → near 0.0, no overflow
/// assert!(stable_sigmoid(-700.0) < 1e-10);
/// ```
#[must_use]
#[inline]
pub fn stable_sigmoid(x: f64) -> f64 {
    if x >= 0.0 {
        1.0 / (1.0 + (-x).exp())
    } else {
        let ex = x.exp();
        ex / (1.0 + ex)
    }
}

/// Numerically stable logit (inverse sigmoid): $\text{logit}(p) = \ln(p / (1-p))$.
///
/// Two-branch implementation to avoid precision loss:
/// - $p \ge 0.5$: $\ln(p / (1-p))$
/// - $p < 0.5$: $-\ln((1-p) / p)$
///
/// # Panics
///
/// Debug-asserts that $p \in (0, 1)$.
///
/// # Cross-References
///
/// - (´dec:calibration:pure-refit´) — the Platt calibration fit
/// - (´dec:calibration:raw-weight-forms´) — the blend weight computation
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::{stable_logit, stable_sigmoid};
///
/// // logit(0.5) = 0.0
/// assert!(stable_logit(0.5).abs() < 1e-15);
///
/// // logit(σ(x)) ≈ x
/// let x = 2.3;
/// let p = stable_sigmoid(x);
/// assert!((stable_logit(p) - x).abs() < 1e-14);
/// ```
#[must_use]
#[inline]
pub fn stable_logit(p: f64) -> f64 {
    debug_assert!(p > 0.0 && p < 1.0, "logit requires p in (0, 1), got {p}");
    if p >= 0.5 {
        (p / (1.0 - p)).ln()
    } else {
        -((1.0 - p) / p).ln()
    }
}

/// Regime transition function: $\sigma(s_\kappa \cdot (w - b))$.
///
/// Computes `stable_sigmoid(S_KAPPA * (w - BOUNDARY))` where:
/// - $s_\kappa = 20$ (steepness)
/// - $b = 0.3$ (transition boundary)
///
/// Used for soft blending of $\kappa_\text{eff}$ between sister and
/// anchor calibration regimes.
///
/// # Cross-References
///
/// - (´dec:calibration:shared-transition´) — the shared regime transition
///   for `κ_eff` blending
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::regime_transition;
///
/// // At the boundary w = 0.3, output ≈ 0.5
/// assert!((regime_transition(0.3) - 0.5).abs() < 1e-10);
///
/// // Well below boundary → near 0
/// assert!(regime_transition(0.0) < 0.01);
///
/// // Well above boundary → near 1
/// assert!(regime_transition(1.0) > 1.0 - 1e-3);
/// ```
#[must_use]
#[inline]
pub fn regime_transition(w: f64) -> f64 {
    /// Steepness of the regime transition sigmoid.
    const S_KAPPA: f64 = 20.0;
    /// Boundary point where transition is 0.5.
    const BOUNDARY: f64 = 0.3;

    stable_sigmoid(S_KAPPA * (w - BOUNDARY))
}

/// Numerically stable inverse hyperbolic tangent: $\text{atanh}(x) = \frac{1}{2}\ln\frac{1+x}{1-x}$.
///
/// Clamps input to $(-1 + \epsilon, 1 - \epsilon)$ to avoid infinities.
/// Used for outcome axis evaluation to transform predictions back to
/// the unbounded scale.
///
/// # Arguments
///
/// * `x` — Value in $(-1, 1)$
///
/// # Panics
///
/// Debug-asserts that `|x| < 1`.
///
/// # Cross-References
///
/// - (´tab:axis:inference-outputs´) — the outcome axis evaluation this
///   inverts the compression for
///
/// # Examples
///
/// ```
/// use torrust_assayer::numerics_export::stable_atanh;
///
/// // atanh(0) = 0
/// assert!(stable_atanh(0.0).abs() < 1e-15);
///
/// // atanh(tanh(x)) ≈ x
/// let x: f64 = 1.5;
/// let th = x.tanh();
/// assert!((stable_atanh(th) - x).abs() < 1e-14);
/// ```
#[must_use]
#[inline]
pub fn stable_atanh(x: f64) -> f64 {
    const EPSILON: f64 = 1e-15;
    debug_assert!(x.abs() < 1.0, "atanh requires |x| < 1, got {x}");
    // Clamp to avoid infinity at boundaries
    let clamped = x.clamp(-1.0 + EPSILON, 1.0 - EPSILON);
    0.5 * ((1.0 + clamped) / (1.0 - clamped)).ln()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Special Functions for the Decision Landscape
// ═══════════════════════════════════════════════════════════════════════════════

/// Natural logarithm of the gamma function.
///
/// Lanczos approximation with g = 7 and nine coefficients, accurate to
/// about fifteen significant digits over the positive reals, with the
/// reflection formula covering arguments below one half. Serves the
/// regularised incomplete beta function below; the derivation layer
/// reaches it for dominance probabilities (´thm:landscape:dominance´)
/// and Beta quantiles (´def:landscape:credible-intervals´).
#[must_use]
#[allow(clippy::excessive_precision)] // Justified: the published Lanczos coefficients are kept verbatim
pub fn ln_gamma(x: f64) -> f64 {
    /// Lanczos coefficients for g = 7, n = 9.
    const LANCZOS: [f64; 9] = [
        0.999_999_999_999_809_93,
        676.520_368_121_885_1,
        -1_259.139_216_722_402_8,
        771.323_428_777_653_13,
        -176.615_029_162_140_59,
        12.507_343_278_686_905,
        -0.138_571_095_265_720_12,
        9.984_369_578_019_571_6e-6,
        1.505_632_735_149_311_6e-7,
    ];

    if x < 0.5 {
        // Reflection: Γ(x)Γ(1−x) = π / sin(πx).
        let sin_pi_x = (std::f64::consts::PI * x).sin();
        return std::f64::consts::PI.ln() - sin_pi_x.abs().ln() - ln_gamma(1.0 - x);
    }

    let x = x - 1.0;
    let mut acc = LANCZOS[0];
    for (i, &c) in LANCZOS.iter().enumerate().skip(1) {
        #[allow(clippy::cast_precision_loss)] // Justified: i is at most eight
        let denominator = x + i as f64;
        acc += c / denominator;
    }
    let t = x + 7.5;
    (x + 0.5).mul_add(t.ln(), 0.5 * (2.0 * std::f64::consts::PI).ln()) - t + acc.ln()
}

/// Continued-fraction kernel of the incomplete beta function.
///
/// Modified Lentz evaluation with a fixed iteration ceiling; the
/// fraction converges in a handful of terms for the argument range the
/// series split in [`reg_inc_beta`] admits.
#[allow(clippy::many_single_char_names)] // Justified: the Lentz recurrence's standard notation
fn beta_continued_fraction(a: f64, b: f64, x: f64) -> f64 {
    /// Iteration ceiling: the fraction converges long before this.
    const MAX_ITER: usize = 200;
    /// Relative-change convergence floor, a few ulps above machine ε.
    const EPS: f64 = 3.0e-16;
    /// Underflow guard for Lentz denominators.
    const FP_MIN: f64 = 1.0e-300;

    let qab = a + b;
    let qap = a + 1.0;
    let qam = a - 1.0;
    let mut c = 1.0;
    let mut d = 1.0 - qab * x / qap;
    if d.abs() < FP_MIN {
        d = FP_MIN;
    }
    d = 1.0 / d;
    let mut h = d;

    for m in 1..=MAX_ITER {
        #[allow(clippy::cast_precision_loss)] // Justified: the iteration ceiling is two hundred
        let m = m as f64;
        let m2 = 2.0 * m;

        // Even step.
        let aa = m * (b - m) * x / ((qam + m2) * (a + m2));
        d = aa.mul_add(d, 1.0);
        if d.abs() < FP_MIN {
            d = FP_MIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FP_MIN {
            c = FP_MIN;
        }
        d = 1.0 / d;
        h *= d * c;

        // Odd step.
        let aa = -(a + m) * (qab + m) * x / ((a + m2) * (qap + m2));
        d = aa.mul_add(d, 1.0);
        if d.abs() < FP_MIN {
            d = FP_MIN;
        }
        c = 1.0 + aa / c;
        if c.abs() < FP_MIN {
            c = FP_MIN;
        }
        d = 1.0 / d;
        let delta = d * c;
        h *= delta;

        if (delta - 1.0).abs() < EPS {
            break;
        }
    }

    h
}

/// Regularised incomplete beta function `I_x(a, b)`.
///
/// The cumulative distribution function of the Beta distribution: the
/// dominance theorem reports each action's domination probability
/// through it (´thm:landscape:dominance´). Arguments `a` and `b` must
/// be positive; `x` is clamped to the unit interval.
#[must_use]
pub fn reg_inc_beta(a: f64, b: f64, x: f64) -> f64 {
    debug_assert!(a > 0.0 && b > 0.0, "beta shape parameters must be positive");
    if x <= 0.0 {
        return 0.0;
    }
    if x >= 1.0 {
        return 1.0;
    }

    // ln of x^a (1−x)^b / (a·B(a, b)), the prefactor of both branches.
    let ln_front = b.mul_add((1.0 - x).ln(), a.mul_add(x.ln(), ln_gamma(a + b) - ln_gamma(a) - ln_gamma(b)));

    // Series split at the symmetry point for fast convergence.
    if x < (a + 1.0) / (a + b + 2.0) {
        ln_front.exp() * beta_continued_fraction(a, b, x) / a
    } else {
        1.0 - ln_front.exp() * beta_continued_fraction(b, a, 1.0 - x) / b
    }
}

/// Quantile of the Beta(`a`, `b`) distribution.
///
/// Inverse of [`reg_inc_beta`] in its `x` argument, by a fixed count of
/// bisection halvings — deterministic, and past double resolution on
/// the unit interval well before the count runs out. Credible intervals
/// push the posterior's quantiles through the crossover offsets
/// (´def:landscape:credible-intervals´).
#[must_use]
pub fn beta_quantile(a: f64, b: f64, p: f64) -> f64 {
    /// Bisection halvings: 2⁻¹⁰⁰ is far below one ulp on `(0, 1)`.
    const HALVINGS: usize = 100;

    if p <= 0.0 {
        return 0.0;
    }
    if p >= 1.0 {
        return 1.0;
    }

    let mut lo = 0.0_f64;
    let mut hi = 1.0_f64;
    for _ in 0..HALVINGS {
        let mid = f64::midpoint(lo, hi);
        if reg_inc_beta(a, b, mid) < p {
            lo = mid;
        } else {
            hi = mid;
        }
    }
    f64::midpoint(lo, hi)
}

/// Standard normal cumulative distribution function `Φ(z)`.
///
/// Abramowitz–Stegun 26.2.17 polynomial, absolute error below
/// `7.5e-8` — ample for the flip probabilities fragility reports
/// (´def:fragility:definition´).
#[must_use]
pub fn normal_cdf(z: f64) -> f64 {
    /// A&S 26.2.17 rational-tail coefficients.
    const B: [f64; 5] = [0.319_381_530, -0.356_563_782, 1.781_477_937, -1.821_255_978, 1.330_274_429];
    /// A&S 26.2.17 substitution constant.
    const P: f64 = 0.231_641_9;

    let x = z.abs();
    let t = 1.0 / P.mul_add(x, 1.0);
    let poly = t * B[4].mul_add(t, B[3]).mul_add(t, B[2]).mul_add(t, B[1]).mul_add(t, B[0]);
    let pdf = (-0.5 * x * x).exp() / (2.0 * std::f64::consts::PI).sqrt();
    let upper = pdf * poly;
    if z >= 0.0 { 1.0 - upper } else { upper }
}
