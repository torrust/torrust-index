// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`decay_factor_since_backward_clock`] | numerics | A pair of timestamps in the wrong order decays nothing at all, returning exactly unity. Clocks do step backwards — across restarts, across hosts, across corrections — and reading such a pair as an elapsed interval would either raise the factor above one, amplifying old evidence, or send it to a place the arithmetic was never meant to go. |
//! | [`decay_factor_since_enormous_gap`] | numerics | A gap of ten years leaves a factor that is tiny but still strictly positive, because the elapsed hours are clamped before they are used as an exponent. Evidence that old should count for almost nothing, yet reaching exactly zero would turn every value it multiplies into a hard zero and erase the distinction between very old and absent. |
//! | [`decay_factor_since_zero_gap`] | numerics | No time elapsed means no decay — exactly one, not approximately one. Decay is applied at every touch of a value, and repeated touches within the same instant are common, so a factor a hair below unity would erode values through nothing but bookkeeping. |
//! | [`decay_factor_elapsed_zero_duration`] | numerics | cites (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´) |
//! | [`decay_factor_range_property`] | numerics | Across a thousand sampled rates and intervals the factor never leaves the half-open unit interval: always strictly positive, never above one. This is the property every caller relies on without checking — a factor above one would let decay manufacture evidence, and a factor at or below zero would annihilate or invert whatever it scales. |
//! | [`decay_factor_composition_property`] | numerics | Decaying over two intervals in turn gives the same result as decaying once over their sum, to within rounding, across a thousand sampled rates and splits. Values are decayed lazily, whenever they happen next to be touched, so how often that happens must not change what they are worth — otherwise a frequently read value would age differently from a neglected one. |
//! | [`decay_time_monotonicity_seeded_sweep`] | numerics | cites (´claim:numerics:decay-never-increases-as-the-interval-lengthens´) |
//! | [`decay_rate_monotonicity_seeded_sweep`] | numerics | cites (´claim:numerics:a-higher-rate-retains-more-over-the-same-interval´) |
//! | [`decay_range_seeded_sweep`] | numerics | cites (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´) |
//! | [`decay_composition_seeded_sweep`] | numerics | cites (´claim:numerics:decay-composes-over-consecutive-intervals´) |
//! | [`sigmoid_logit_seeded_sweep`] | numerics | cites (´claim:numerics:the-sigmoid-inverts-the-logit-across-the-open-unit-interval´) |
//! | [`atanh_tanh_seeded_sweep`] | wellness | cites (´claim:wellness:the-stable-inverse-tanh-recovers-its-argument-across-the-whole-working-range´) |
//! | [`sigmoid_logit_roundtrip_1000_values`] | numerics | The sigmoid and the logit are genuine inverses across the open unit interval, right out to within a millionth of either end, recovering the original probability to within rounding. Probabilities are moved into log-odds to be combined additively and moved back afterwards, so any error in the pairing would be a systematic bias in every score that made the round trip. |
//! | [`stable_sigmoid_extreme_positive`] | numerics | At an argument far past where the naive exponential would overflow, the sigmoid returns a finite value pressed against one. Log-odds arriving from accumulated evidence can be arbitrarily large, and the saturating answer is the correct one — a non-finite result would poison every value downstream of it. |
//! | [`stable_sigmoid_extreme_negative`] | numerics | cites (´claim:numerics:the-sigmoid-saturates-without-overflowing-at-extreme-arguments´) |
//! | [`regime_transition_midpoint`] | numerics | The transition between regimes crosses the half-way mark exactly at its boundary parameter, giving neither regime the benefit at that point. The boundary is where the blend is meant to be balanced, so it is the one input whose output cannot be a matter of taste. |
//! | [`regime_transition_lower_bound`] | numerics | Deep in the lower regime the transition has effectively closed, sitting a hundredth away from zero or nearer. A blend that never quite commits would leave a permanent trace of the wrong regime in every value it weighs. |
//! | [`regime_transition_upper_bound`] | numerics | cites (´claim:numerics:the-regime-transition-saturates-at-both-ends-of-its-input-range´) |
//! | [`ln_gamma_known_values`] | numerics | The log-gamma implementation reproduces the values every downstream figure leans on: zero at one and at two, the half-log of π at one half, and the log of twenty-four at five, each to a comfortable margin inside double precision. The incomplete beta's prefactor is a ratio of three gammas, so an error here would scale every dominance probability and every credible quantile by a silent constant. |
//! | [`reg_inc_beta_uniform_is_identity`] | numerics | On the uniform distribution the regularised incomplete beta is the identity: I_x(1, 1) returns x itself across the unit interval, including the 0.375 the dominance theorem tabulates for Challenge at the cold-start prior (´thm:landscape:dominance´). The uniform case is the one a reader can check without any special function at all, which makes it the anchor for trusting the cases they cannot. |
//! | [`reg_inc_beta_symmetry_and_powers`] | numerics | The incomplete beta honours its reflection identity — I_x(a, b) and 1 − I of the complement with the shapes swapped agree across sampled shapes — and reduces to the closed-form power of x when the second shape is one. Both identities cross the continued fraction's internal series split, so their agreement is evidence the two branches meet without a seam. |
//! | [`beta_quantile_inverts_the_cdf`] | numerics | The Beta quantile is a genuine inverse: pushing a probability through the quantile and back through the distribution function recovers it to near double resolution, across uniform, skewed and bathtub shapes. Credible intervals evaluate crossover offsets exactly at these quantiles (´def:landscape:credible-intervals´), so any inversion slack would widen or narrow every reported interval by stealth. |
//! | [`normal_cdf_reference_points`] | numerics | The normal distribution function returns one half at zero, the textbook 0.975 at 1.96, mirrors itself around zero, and saturates cleanly in the far tails. Flip probabilities are sums of two of these evaluations (´def:fragility:definition´), so the reference points bound the error a host-facing fragility reading can inherit from this approximation. |

//! Crate-level tests for the numerics module.

use std::time::Duration;

use crate::numerics::{
    beta_quantile, decay_factor, decay_factor_elapsed, decay_factor_since, ln_gamma, normal_cdf, reg_inc_beta, regime_transition,
    stable_atanh, stable_logit, stable_sigmoid,
};
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_near, run_seeded_sweep};
use crate::types::PersistentTimestamp;

/// MAX_DECAY_HOURS clamp in `decay_factor_since`, the clamp the timestamp
/// interface embeds so no caller can bypass it (´dec:clock:embedded-clamp´).
const MAX_DECAY_HOURS: f64 = 8_760.0;

#[derive(Debug)]
struct DecayTimeCase {
    rate: f64,
    shorter_hours: f64,
    longer_hours: f64,
}

#[derive(Debug)]
struct DecayRateCase {
    lower_rate: f64,
    higher_rate: f64,
    hours: f64,
}

#[derive(Debug)]
struct DecayRangeCase {
    rate: f64,
    hours: f64,
    stored_value: f64,
}

#[derive(Debug)]
struct DecayCompositionCase {
    rate: f64,
    first_hours: f64,
    second_hours: f64,
}

#[derive(Debug)]
struct SigmoidCase {
    probability: f64,
}

#[derive(Debug)]
struct AtanhCase {
    unbounded_value: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Decay Function Properties
// ═══════════════════════════════════════════════════════════════════════════════

/// A pair of timestamps in the wrong order decays nothing at all, returning
/// exactly unity. Clocks do step backwards — across restarts, across hosts,
/// across corrections — and reading such a pair as an elapsed interval would
/// either raise the factor above one, amplifying old evidence, or send it to a
/// place the arithmetic was never meant to go.
///
/// ´claim:numerics:a-backward-clock-decays-nothing-at-all´
/// ´test:crate:decay-factor-since-backward-clock´
#[test]
fn decay_factor_since_backward_clock() {
    let t_new = PersistentTimestamp::new(1000, 0);
    let t_old = PersistentTimestamp::new(2000, 0); // "old" is actually later
    let f = decay_factor_since(0.999, &t_old, &t_new);
    assert_eq!(f.to_bits(), 1.0f64.to_bits(), "backward clock should yield 1.0 (no decay)");
}

/// A gap of ten years leaves a factor that is tiny but still strictly
/// positive, because the elapsed hours are clamped before they are used as an
/// exponent. Evidence that old should count for almost nothing, yet reaching
/// exactly zero would turn every value it multiplies into a hard zero and
/// erase the distinction between very old and absent.
///
/// ´claim:numerics:an-enormous-gap-is-clamped-and-still-leaves-a-strictly-positive-factor´
/// ´test:crate:decay-factor-since-enormous-gap´
#[test]
fn decay_factor_since_enormous_gap() {
    let t0 = PersistentTimestamp::new(0, 0);
    let ten_years_secs = 10 * 365 * 24 * 3600; // ~87,600 hours
    let t1 = PersistentTimestamp::new(ten_years_secs, 0);
    let f = decay_factor_since(0.999, &t0, &t1);
    // Clamped to MAX_DECAY_HOURS (8760), so f = 0.999^8760 ≈ 1.5e-4
    assert!(f > 0.0, "result must be positive");
    assert!(f < 0.01, "result must be very small, got {f}");
}

/// No time elapsed means no decay — exactly one, not approximately one. Decay
/// is applied at every touch of a value, and repeated touches within the same
/// instant are common, so a factor a hair below unity would erode values
/// through nothing but bookkeeping.
///
/// ´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´
/// ´test:crate:decay-factor-since-zero-gap´
#[test]
fn decay_factor_since_zero_gap() {
    let t = PersistentTimestamp::new(500_000, 0);
    let f = decay_factor_since(0.999, &t, &t);
    assert_eq!(f.to_bits(), 1.0f64.to_bits(), "zero gap should yield exactly 1.0");
}

/// The same holds when the interval arrives as a duration rather than as a
/// pair of timestamps: a zero duration decays by exactly unity. The several
/// entry points differ only in how they are told how much time passed.
///
/// (´claim:numerics:a-zero-elapsed-interval-decays-by-exactly-unity´)
/// ´test:crate:decay-factor-elapsed-zero-duration´
#[test]
fn decay_factor_elapsed_zero_duration() {
    let f = decay_factor_elapsed(0.999, Duration::ZERO);
    assert_eq!(f.to_bits(), 1.0f64.to_bits(), "zero duration should yield exactly 1.0");
}

/// Across a thousand sampled rates and intervals the factor never leaves the
/// half-open unit interval: always strictly positive, never above one. This is
/// the property every caller relies on without checking — a factor above one
/// would let decay manufacture evidence, and a factor at or below zero would
/// annihilate or invert whatever it scales.
///
/// ´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´
/// ´test:crate:decay-factor-range-property´
#[test]
fn decay_factor_range_property() {
    let mut rng = TestRng::new(0xDEAD_BEEF_CAFE_1234);

    for _ in 0..1_000 {
        let gamma = rng.next_f64().mul_add(0.0099, 0.99); // (0.99, 0.9999)
        let dt = rng.next_f64() * MAX_DECAY_HOURS; // [0, MAX_DECAY_HOURS]
        let f = decay_factor(gamma, dt);
        assert!(f > 0.0, "result must be positive for γ={gamma}, dt={dt}");
        assert!(f <= 1.0, "result must be ≤ 1.0 for γ={gamma}, dt={dt}");
    }
}

/// Decaying over two intervals in turn gives the same result as decaying once
/// over their sum, to within rounding, across a thousand sampled rates and
/// splits. Values are decayed lazily, whenever they happen next to be touched,
/// so how often that happens must not change what they are worth — otherwise a
/// frequently read value would age differently from a neglected one.
///
/// ´claim:numerics:decay-composes-over-consecutive-intervals´
/// ´test:crate:decay-factor-composition-property´
#[test]
fn decay_factor_composition_property() {
    let mut rng = TestRng::new(0xCAFE_BABE_0000_0042);

    for _ in 0..1_000 {
        let gamma = rng.next_f64().mul_add(0.0099, 0.99); // (0.99, 0.9999)
        let a = rng.next_f64() * (MAX_DECAY_HOURS / 2.0);
        let b = rng.next_f64() * (MAX_DECAY_HOURS / 2.0);
        let fa = decay_factor(gamma, a);
        let fb = decay_factor(gamma, b);
        let fab = decay_factor(gamma, a + b);
        assert_near(
            fa * fb,
            fab,
            1e-14,
            &format!("composition f(γ={gamma}, {a})·f(γ={gamma}, {b})"),
        );
    }
}

/// Waiting longer never retains more. The ordering makes pure decay a legible weakening operation rather than a curve that folds with age.
///
/// The sweep draws 459 independent pairs because that many draws give more than a ninety-nine per cent chance of entering any reversal region occupying at least one per cent of the declared joint rate-and-interval distribution. The figure is a starting bound over that distribution; deriving coverage for a thinner floating-point seam would require a partition of the implementation's exponentiation error rather than a larger number chosen by feel.
///
/// (´claim:numerics:decay-never-increases-as-the-interval-lengthens´)
/// ´test:crate:decay-time-monotonicity-seeded-sweep´
#[test]
fn decay_time_monotonicity_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xDEC4_710E_0000_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| {
            let hours_a = rng.next_f64() * MAX_DECAY_HOURS;
            let hours_b = rng.next_f64() * MAX_DECAY_HOURS;
            DecayTimeCase {
                rate: rng.next_f64().mul_add(0.5, 0.5),
                shorter_hours: hours_a.min(hours_b),
                longer_hours: hours_a.max(hours_b),
            }
        },
        |case| {
            let shorter = decay_factor(case.rate, case.shorter_hours);
            let longer = decay_factor(case.rate, case.longer_hours);
            if longer > shorter {
                return Err(format!(
                    "time ordering reversed: longer factor {longer} exceeds shorter factor {shorter}"
                ));
            }

            Ok(())
        },
    );
}

/// Raising the hourly retention rate over one interval never retains less. The ordering makes the rate a real memory control rather than a parameter whose curve reverses.
///
/// The sweep draws 459 independent pairs because that many draws give more than a ninety-nine per cent chance of entering any reversal region occupying at least one per cent of the declared joint rate-and-interval distribution. The figure is a starting bound over that distribution; deriving coverage for a thinner floating-point seam would require a partition of the implementation's exponentiation error rather than a larger number chosen by feel.
///
/// (´claim:numerics:a-higher-rate-retains-more-over-the-same-interval´)
/// ´test:crate:decay-rate-monotonicity-seeded-sweep´
#[test]
fn decay_rate_monotonicity_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xDEC4_2A7E_0000_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| {
            let rate_a = rng.next_f64().mul_add(0.5, 0.5);
            let rate_b = rng.next_f64().mul_add(0.5, 0.5);
            DecayRateCase {
                lower_rate: rate_a.min(rate_b),
                higher_rate: rate_a.max(rate_b),
                hours: rng.next_f64() * MAX_DECAY_HOURS,
            }
        },
        |case| {
            let lower_rate = decay_factor(case.lower_rate, case.hours);
            let higher_rate = decay_factor(case.higher_rate, case.hours);
            if higher_rate < lower_rate {
                return Err(format!(
                    "rate ordering reversed: higher-rate factor {higher_rate} is below lower-rate factor {lower_rate}"
                ));
            }

            Ok(())
        },
    );
}

/// Valid decay factors remain positive and no greater than one, and discounting a positive stored value neither enlarges nor eliminates it.
///
/// The generator draws 459 independent rates from `[0.99, 0.9999)`, intervals across the declared decay horizon, and positive stored values from `(f64::EPSILON, 1]`. This covers the configured retention range and both small and horizon-scale discounts. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´inv:guarantee:ledger-floor´)
/// (´claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval´)
/// ´test:crate:decay-range-seeded-sweep´
#[test]
fn decay_range_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xDEC4_A16E_BA0D_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| DecayRangeCase {
            rate: rng.next_f64().mul_add(0.0099, 0.99),
            hours: rng.next_f64() * MAX_DECAY_HOURS,
            stored_value: rng.next_f64().mul_add(1.0 - f64::EPSILON, f64::EPSILON),
        },
        |case| {
            let factor = decay_factor(case.rate, case.hours);
            if !(factor > 0.0 && factor <= 1.0) {
                return Err(format!("decay factor left (0, 1]: factor={factor}"));
            }
            let discounted = case.stored_value * factor;
            if discounted <= 0.0 || discounted > case.stored_value {
                return Err(format!("discounted value {discounted} is outside (0, {}]", case.stored_value));
            }
            Ok(())
        },
    );
}

/// Sequential decay intervals compose to the summed interval within floating-point tolerance.
///
/// The generator draws 459 independent rates from `[0.99, 0.9999)` and two intervals whose sum stays inside the declared decay horizon, covering small and horizon-scale compositions. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´claim:numerics:decay-composes-over-consecutive-intervals´)
/// ´test:crate:decay-composition-seeded-sweep´
#[test]
fn decay_composition_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xDEC4_A16E_C0A5_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| DecayCompositionCase {
            rate: rng.next_f64().mul_add(0.0099, 0.99),
            first_hours: rng.next_f64() * (MAX_DECAY_HOURS / 2.0),
            second_hours: rng.next_f64() * (MAX_DECAY_HOURS / 2.0),
        },
        |case| {
            let first = decay_factor(case.rate, case.first_hours);
            let second = decay_factor(case.rate, case.second_hours);
            let combined = decay_factor(case.rate, case.first_hours + case.second_hours);
            let composition_error = first.mul_add(second, -combined).abs();
            if composition_error > 1e-14 {
                return Err(format!("sequential and summed decay differ by {composition_error}"));
            }
            Ok(())
        },
    );
}

/// The stable sigmoid inverts the logit throughout the working interior of the unit interval.
///
/// The generator draws 459 probabilities from `[10⁻⁶, 1 − 10⁻⁶]`, including values arbitrarily near both declared endpoints under the deterministic stream. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´claim:numerics:the-sigmoid-inverts-the-logit-across-the-open-unit-interval´)
/// ´test:crate:sigmoid-logit-seeded-sweep´
#[test]
fn sigmoid_logit_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0x5160_01D1_10A1_0459;
    const EDGE: f64 = 1e-6;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| SigmoidCase {
            probability: rng.next_f64().mul_add(2.0_f64.mul_add(-EDGE, 1.0), EDGE),
        },
        |case| {
            let probability_error = (stable_sigmoid(stable_logit(case.probability)) - case.probability).abs();
            if probability_error > 1e-14 {
                return Err(format!("sigmoid-logit round-trip error {probability_error}"));
            }
            Ok(())
        },
    );
}

/// The stable inverse hyperbolic tangent recovers every drawn working-range input after a forward transform.
///
/// The generator draws 459 unbounded values from `[−5, 5)`, spanning the full working range and its most saturated images. The count gives more than a ninety-nine per cent chance of entering any defect region occupying at least one per cent of this distribution.
///
/// (´claim:wellness:the-stable-inverse-tanh-recovers-its-argument-across-the-whole-working-range´)
/// ´test:crate:atanh-tanh-seeded-sweep´
#[test]
fn atanh_tanh_seeded_sweep() {
    const CASES: usize = 459;
    const SEED: u64 = 0xA7A4_7A4A_10A1_0459;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| AtanhCase {
            unbounded_value: rng.next_f64().mul_add(10.0, -5.0),
        },
        |case| {
            let hyperbolic_error = (stable_atanh(case.unbounded_value.tanh()) - case.unbounded_value).abs();
            if hyperbolic_error > 1e-10 {
                return Err(format!("atanh-tanh round-trip error {hyperbolic_error}"));
            }
            Ok(())
        },
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// L3 Shared Algorithm Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// The sigmoid and the logit are genuine inverses across the open unit
/// interval, right out to within a millionth of either end, recovering the
/// original probability to within rounding. Probabilities are moved into
/// log-odds to be combined additively and moved back afterwards, so any error
/// in the pairing would be a systematic bias in every score that made the
/// round trip.
///
/// ´claim:numerics:the-sigmoid-inverts-the-logit-across-the-open-unit-interval´
/// ´test:crate:sigmoid-logit-roundtrip-1000-values´
#[test]
fn sigmoid_logit_roundtrip_1000_values() {
    let mut rng = TestRng::new(0xDEAD_BEEF_0000_0001);

    let lo = 1e-6;
    let hi = 1.0 - 1e-6;
    for _ in 0..1_000 {
        let x = rng.next_f64().mul_add(hi - lo, lo);
        let roundtrip = stable_sigmoid(stable_logit(x));
        assert_near(roundtrip, x, 1e-14, "sigmoid∘logit roundtrip");
    }
}

/// At an argument far past where the naive exponential would overflow, the
/// sigmoid returns a finite value pressed against one. Log-odds arriving from
/// accumulated evidence can be arbitrarily large, and the saturating answer is
/// the correct one — a non-finite result would poison every value downstream
/// of it.
///
/// ´claim:numerics:the-sigmoid-saturates-without-overflowing-at-extreme-arguments´
/// ´test:crate:stable-sigmoid-extreme-positive´
#[test]
fn stable_sigmoid_extreme_positive() {
    let result = stable_sigmoid(700.0);
    assert!(result.is_finite(), "must not overflow");
    assert_near(result, 1.0, 1e-10, "stable_sigmoid(+700)");
}

/// The negative extreme is handled as gracefully as the positive one, settling
/// finitely against zero. The two tails need different arrangements of the
/// same algebra to avoid overflow, so neither can be assumed safe from the
/// other being safe.
///
/// (´claim:numerics:the-sigmoid-saturates-without-overflowing-at-extreme-arguments´)
/// ´test:crate:stable-sigmoid-extreme-negative´
#[test]
fn stable_sigmoid_extreme_negative() {
    let result = stable_sigmoid(-700.0);
    assert!(result.is_finite(), "must not overflow");
    assert_near(result, 0.0, 1e-10, "stable_sigmoid(-700)");
}

/// The transition between regimes crosses the half-way mark exactly at its
/// boundary parameter, giving neither regime the benefit at that point. The
/// boundary is where the blend is meant to be balanced, so it is the one input
/// whose output cannot be a matter of taste.
///
/// ´claim:numerics:the-regime-transition-crosses-half-way-at-its-boundary-parameter´
/// ´test:crate:regime-transition-midpoint´
#[test]
fn regime_transition_midpoint() {
    let val = regime_transition(0.3);
    assert_near(val, 0.5, DEFAULT_TOLERANCES.default, "regime_transition(0.3)");
}

/// Deep in the lower regime the transition has effectively closed, sitting a
/// hundredth away from zero or nearer. A blend that never quite commits would
/// leave a permanent trace of the wrong regime in every value it weighs.
///
/// ´claim:numerics:the-regime-transition-saturates-at-both-ends-of-its-input-range´
/// ´test:crate:regime-transition-lower-bound´
#[test]
fn regime_transition_lower_bound() {
    let val = regime_transition(0.0);
    assert!(val < 0.01, "expected < 0.01 at w=0.0, got {val}");
}

/// At the top of the range the transition has opened just as fully, within a
/// thousandth of one. The boundary parameter sits well below the midpoint of
/// the range, so the two ends are at unequal distances from it and neither
/// saturation follows from the other.
///
/// (´claim:numerics:the-regime-transition-saturates-at-both-ends-of-its-input-range´)
/// ´test:crate:regime-transition-upper-bound´
#[test]
fn regime_transition_upper_bound() {
    let val = regime_transition(1.0);
    assert!(val > 1.0 - 1e-3, "expected > 1−10⁻³ at w=1.0, got {val}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Special Functions for the Decision Landscape
// ═══════════════════════════════════════════════════════════════════════════════

/// The log-gamma implementation reproduces the values every downstream figure
/// leans on: zero at one and at two, the half-log of π at one half, and the
/// log of twenty-four at five, each to a comfortable margin inside double
/// precision. The incomplete beta's prefactor is a ratio of three gammas, so
/// an error here would scale every dominance probability and every credible
/// quantile by a silent constant.
///
/// ´claim:numerics:log-gamma-reproduces-the-textbook-anchor-values´
/// ´test:crate:ln-gamma-known-values´
#[test]
fn ln_gamma_known_values() {
    assert!(ln_gamma(1.0).abs() < 1e-13, "ln Γ(1) = 0, got {}", ln_gamma(1.0));
    assert!(ln_gamma(2.0).abs() < 1e-13, "ln Γ(2) = 0, got {}", ln_gamma(2.0));
    let half = 0.5 * std::f64::consts::PI.ln();
    assert!((ln_gamma(0.5) - half).abs() < 1e-12, "ln Γ(1/2) = ln √π");
    assert!((ln_gamma(5.0) - 24.0_f64.ln()).abs() < 1e-12, "ln Γ(5) = ln 24");
}

/// On the uniform distribution the regularised incomplete beta is the
/// identity: I_x(1, 1) returns x itself across the unit interval, including
/// the 0.375 the dominance theorem tabulates for Challenge at the cold-start
/// prior (´thm:landscape:dominance´). The uniform case is the one a reader
/// can check without any special function at all, which makes it the anchor
/// for trusting the cases they cannot.
///
/// ´claim:numerics:the-incomplete-beta-is-the-identity-on-the-uniform-distribution´
/// ´test:crate:reg-inc-beta-uniform-is-identity´
#[test]
fn reg_inc_beta_uniform_is_identity() {
    for &x in &[0.025, 0.1, 0.375, 0.5, 0.706, 0.975] {
        let i = reg_inc_beta(1.0, 1.0, x);
        assert!((i - x).abs() < 1e-12, "I_{x}(1,1) should be {x}, got {i}");
    }
}

/// The incomplete beta honours its reflection identity — I_x(a, b) and
/// 1 − I of the complement with the shapes swapped agree across sampled
/// shapes — and reduces to the closed-form power of x when the second shape
/// is one. Both identities cross the continued fraction's internal series
/// split, so their agreement is evidence the two branches meet without a
/// seam.
///
/// ´claim:numerics:the-incomplete-beta-honours-reflection-and-the-power-law-reduction´
/// ´test:crate:reg-inc-beta-symmetry-and-powers´
#[test]
fn reg_inc_beta_symmetry_and_powers() {
    for &(a, b) in &[(2.0, 5.0), (0.5, 0.5), (7.0, 1.5), (1.0, 1.6)] {
        for &x in &[0.05, 0.3, 0.5, 0.8, 0.95] {
            let forward = reg_inc_beta(a, b, x);
            let reflected = 1.0 - reg_inc_beta(b, a, 1.0 - x);
            assert!(
                (forward - reflected).abs() < 1e-12,
                "reflection at a={a} b={b} x={x}: {forward} vs {reflected}"
            );
        }
    }
    // b = 1 reduces to x^a: I_x(a, 1) = x^a.
    for &(a, x) in &[(2.0, 0.7), (3.0, 0.4), (1.6, 0.625)] {
        let i = reg_inc_beta(a, 1.0, x);
        let power = x.powf(a);
        assert!((i - power).abs() < 1e-12, "I_{x}({a},1) = {power}, got {i}");
    }
}

/// The Beta quantile is a genuine inverse: pushing a probability through the
/// quantile and back through the distribution function recovers it to near
/// double resolution, across uniform, skewed and bathtub shapes. Credible
/// intervals evaluate crossover offsets exactly at these quantiles
/// (´def:landscape:credible-intervals´), so any inversion slack would widen
/// or narrow every reported interval by stealth.
///
/// ´claim:numerics:the-beta-quantile-inverts-the-distribution-function-to-double-resolution´
/// ´test:crate:beta-quantile-inverts-the-cdf´
#[test]
fn beta_quantile_inverts_the_cdf() {
    for &(a, b) in &[(1.0, 1.0), (2.0, 5.0), (0.5, 0.5), (30.0, 10.0)] {
        for &p in &[0.025, 0.25, 0.5, 0.75, 0.975] {
            let q = beta_quantile(a, b, p);
            let back = reg_inc_beta(a, b, q);
            assert!(
                (back - p).abs() < 1e-12,
                "quantile roundtrip at a={a} b={b} p={p}: q={q}, back={back}"
            );
        }
    }
    // Uniform quantile is the probability itself.
    assert!((beta_quantile(1.0, 1.0, 0.025) - 0.025).abs() < 1e-12);
    assert!((beta_quantile(1.0, 1.0, 0.975) - 0.975).abs() < 1e-12);
}

/// The normal distribution function returns one half at zero, the textbook
/// 0.975 at 1.96, mirrors itself around zero, and saturates cleanly in the
/// far tails. Flip probabilities are sums of two of these evaluations
/// (´def:fragility:definition´), so the reference points bound the error a
/// host-facing fragility reading can inherit from this approximation.
///
/// ´claim:numerics:the-normal-distribution-function-hits-its-reference-points´
/// ´test:crate:normal-cdf-reference-points´
#[test]
fn normal_cdf_reference_points() {
    assert!((normal_cdf(0.0) - 0.5).abs() < 1e-7, "Φ(0) = 1/2");
    assert!((normal_cdf(1.96) - 0.975_002).abs() < 1e-4, "Φ(1.96) ≈ 0.975");
    for &z in &[0.3, 1.0, 1.96, 2.5] {
        let mirror = normal_cdf(z) + normal_cdf(-z);
        assert!((mirror - 1.0).abs() < 2e-7, "Φ(z) + Φ(−z) = 1 at z={z}");
    }
    assert!(normal_cdf(8.0) > 1.0 - 1e-14, "far upper tail saturates to one");
    assert!(normal_cdf(-8.0) < 1e-14, "far lower tail saturates to zero");
}
