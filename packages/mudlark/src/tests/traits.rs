// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for the **public trait contracts** defined in
//! the `traits` module (Surface 2 — Film).
//!
//! Each trait (`Coordinate`, `Accumulator`, `Inspectable`, `Proratable`,
//! `Attenuatable`, `Weighable`, `Rng`, `Observation`, `ScalableObservation`)
//! is exercised across every concrete type that implements it (`u8`,
//! `u16`, `u64`, `u128`, `f32`, `f64`).  The tests verify identity
//! elements, boundary conditions, round-trip consistency, expected
//! panics on overflow / unsupported operations, and cross-type
//! accumulation / scaling introduced by the `Observation` blanket
//! impls.
//!
//! # Test index
//!
//! ## Coordinate
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_zero`] | `zero()` returns 0 for `u64` and `u8` |
//! | [`u64_domain_max`] | `domain_max` for `u64` at N=32 and N=64 |
//! | [`u128_domain_max_full`] | `domain_max` for `u128` at N=128 and N=64 |
//! | [`u64_midpoint`] | integer midpoint, including width-1 edge case |
//! | [`f64_midpoint`] | floating-point midpoint |
//! | [`f32_midpoint`] | `f32` midpoint |
//! | [`u64_width`] | `width(lo, hi)` for unsigned intervals |
//! | [`f64_width`] | `width(lo, hi)` for `f64` |
//! | [`u64_is_final`] | final-depth detection for integer coordinates |
//! | [`f64_is_final`] | final-depth detection for `f64` (mantissa limit) |
//! | [`f32_is_final`] | final-depth detection for `f32` (mantissa limit) |
//! | [`u64_from_u64`] | `from_u64` identity for `u64` and narrowing for `u8` |
//! | [`f64_from_u64`] | `from_u64` conversion to `f64` |
//! | [`u64_next_value`] | `next_value` increments by 1 |
//! | [`f64_next_value_panics`] | `next_value` panics for `f64` (unsupported) |
//! | [`f32_next_value_panics`] | `next_value` panics for `f32` (unsupported) |
//! | [`u64_to_f64`] | `to_f64` exact conversion from `u64` |
//! | [`f64_to_f64`] | `to_f64` identity for `f64` |
//! | [`u64_is_nan`] | integers are never NaN |
//! | [`f64_is_nan`] | `f64::NAN` detected, normal values not |
//! | [`u64_total_cmp`] | total ordering for `u64` |
//! | [`f64_total_cmp`] | total ordering for `f64`, NaN sorts after +∞ |
//!
//! ## Accumulator
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_accumulator_zero_add`] | `zero()` identity and `add` for `u64` |
//! | [`f64_accumulator_zero_add`] | `add` and `zero` identity for `f64` |
//! | [`u64_sub`] | basic unsigned subtraction |
//! | [`u8_sub`] | `u8` subtraction |
//! | [`f64_sub`] | floating-point subtraction |
//! | [`f32_sub`] | `f32` subtraction |
//! | [`sub_mirrors_add`] | `sub(add(a, b), b) == a` round-trip |
//! | [`u64_sub_underflow_panics`] | underflow panics in debug builds |
//! | [`u64_add_overflow_panics`] | overflow panics in debug builds |
//!
//! ## Inspectable
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_to_f64_approx`] | approximate `f64` conversion for `u64` |
//! | [`f64_to_f64_approx`] | identity approximation for `f64` |
//! | [`f32_to_f64_approx`] | `f32` → `f64` widening |
//! | [`u64_from_f64_round_trip`] | `from_f64` with truncation semantics |
//! | [`f64_from_f64_identity`] | `from_f64` identity for `f64` |
//!
//! ## Proratable
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_prorate`] | proportional split including zero-portion and zero-total edges |
//! | [`f64_prorate`] | floating-point proportional split |
//! | [`u64_scale_by`] | `scale_by` with factors 0.0, 0.25, and 1.0 |
//! | [`f64_scale_by`] | `f64::scale_by` identity and quarter |
//!
//! ## Attenuatable
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_attenuate_four_regimes`] | annihilation / attenuation / identity / amplification |
//! | [`f64_attenuate_four_regimes`] | same four regimes for `f64` |
//! | [`f32_attenuate_zero_guard`] | `0 × ∞` and `v × 0` both return 0.0 (no NaN) |
//!
//! ## Weighable
//!
//! | Test | Focus |
//! |------|-------|
//! | [`u64_weight`] | `weight()` converts `u64` to `f64` |
//! | [`f64_weight`] | `weight()` identity for `f64` |
//! | [`f32_weight`] | `weight()` widens `f32` to `f64` |
//!
//! ## Rng
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fixed_rng_returns_value`] | `FixedRng` always returns the same value |
//! | [`seq_rng_cycles`] | `SeqRng` cycles through its sequence |
//! | [`test_lcg_rng_in_range`] | LCG output stays within `[0, 1)` over 100 draws |
//!
//! ## Observation / `ScalableObservation`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`same_type_accumulate`] | blanket `Observation<V> for V` accumulate |
//! | [`cross_type_f64_to_u16`] | `f64 → u16` accumulate (truncation) and scale |
//! | [`cross_type_f64_to_u64`] | `f64 → u64` accumulate and scale |
//! | [`cross_type_f32_to_u16`] | `f32 → u16` accumulate and scale |
//! | [`cross_type_f64_to_f32`] | `f64 → f32` accumulate and scale |
//! | [`observation_total_zero_edge`] | accumulate on zero, scale by zero |

use crate::testing::{FixedRng, SeqRng, TestLcgRng};
use crate::traits::Coordinate;
use crate::{Accumulator, Attenuatable, Inspectable, Observation, Proratable, Rng, ScalableObservation, Weighable};

// ── Coordinate tests ────────────────────────────────────────

#[test]
fn u64_zero() {
    assert_eq!(<u64 as Coordinate>::zero(), 0);
    assert_eq!(<u8 as Coordinate>::zero(), 0);
}

#[test]
fn u64_domain_max() {
    assert_eq!(u64::domain_max(32), 1u64 << 32);
    assert_eq!(u64::domain_max(64), u64::MAX);
}

#[test]
fn u128_domain_max_full() {
    assert_eq!(u128::domain_max(128), u128::MAX);
    assert_eq!(u128::domain_max(64), 1u128 << 64);
}

#[test]
fn u64_midpoint() {
    assert_eq!(u64::midpoint(0, 8), 4);
    assert_eq!(u64::midpoint(4, 8), 6);
    assert_eq!(u64::midpoint(0, 1), 0);
}

#[test]
fn f64_midpoint() {
    assert!((f64::midpoint(0.0, 8.0) - 4.0).abs() < f64::EPSILON);
}

#[test]
fn f32_midpoint() {
    assert!((f32::midpoint(0.0, 8.0) - 4.0).abs() < f32::EPSILON);
}

#[test]
fn u64_width() {
    assert_eq!(u64::width(10, 42), 32);
    assert_eq!(u64::width(0, 256), 256);
}

#[test]
fn f64_width() {
    assert!((f64::width(1.0, 5.0) - 4.0).abs() < f64::EPSILON);
}

#[test]
fn u64_is_final() {
    assert!(u64::is_final(3, 4, 0, 0));
    assert!(!u64::is_final(0, 4, 0, 0));
}

#[test]
fn f64_is_final() {
    assert!(f64::is_final(0.0, 1.0, 52, 52));
    assert!(!f64::is_final(0.0, 1.0, 10, 52));
}

#[test]
fn f32_is_final() {
    assert!(f32::is_final(0.0, 1.0, 23, 23));
    assert!(!f32::is_final(0.0, 1.0, 5, 23));
}

#[test]
fn u64_from_u64() {
    assert_eq!(u64::from_u64(42), 42u64);
    assert_eq!(u8::from_u64(255), 255u8);
}

#[test]
fn f64_from_u64() {
    assert!((f64::from_u64(100) - 100.0).abs() < f64::EPSILON);
}

#[test]
fn u64_next_value() {
    assert_eq!(0u64.next_value(), 1);
    assert_eq!(41u64.next_value(), 42);
}

#[test]
#[should_panic(expected = "not supported for f64")]
fn f64_next_value_panics() {
    let _ = 1.0f64.next_value();
}

#[test]
#[should_panic(expected = "not supported for f32")]
fn f32_next_value_panics() {
    let _ = 1.0f32.next_value();
}

#[test]
fn u64_to_f64() {
    assert!((42u64.to_f64() - 42.0).abs() < f64::EPSILON);
}

#[test]
#[allow(clippy::approx_constant)]
fn f64_to_f64() {
    assert!((3.14f64.to_f64() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn u64_is_nan() {
    assert!(!0u64.is_nan());
    assert!(!u64::MAX.is_nan());
}

#[test]
fn f64_is_nan() {
    assert!(!0.0f64.is_nan());
    assert!(f64::NAN.is_nan());
}

#[test]
fn u64_total_cmp() {
    assert_eq!(5u64.total_cmp(&10), std::cmp::Ordering::Less);
    assert_eq!(10u64.total_cmp(&10), std::cmp::Ordering::Equal);
    assert_eq!(10u64.total_cmp(&5), std::cmp::Ordering::Greater);
}

#[test]
fn f64_total_cmp() {
    assert_eq!(1.0f64.total_cmp(&2.0), std::cmp::Ordering::Less);
    assert_eq!(1.0f64.total_cmp(&1.0), std::cmp::Ordering::Equal);
    // NaN sorts after +∞.
    assert_eq!(f64::NAN.total_cmp(&f64::INFINITY), std::cmp::Ordering::Greater);
}

// ── Accumulator tests ───────────────────────────────────────

#[test]
fn u64_accumulator_zero_add() {
    assert_eq!(<u64 as Accumulator>::zero(), 0);
    assert_eq!(u64::add(3, 5), 8);
    // Identity: add(a, zero) == a.
    assert_eq!(u64::add(42, 0), 42);
}

#[test]
fn f64_accumulator_zero_add() {
    assert!((f64::add(1.5, 2.5) - 4.0).abs() < f64::EPSILON);
    assert!((f64::add(42.0, 0.0) - 42.0).abs() < f64::EPSILON);
}

#[test]
fn u64_sub() {
    assert_eq!(u64::sub(10, 3), 7);
    assert_eq!(u64::sub(5, 5), 0);
}

#[test]
fn u8_sub() {
    assert_eq!(u8::sub(200, 55), 145);
}

#[test]
fn f64_sub() {
    assert!((f64::sub(4.5, 1.5) - 3.0).abs() < f64::EPSILON);
    assert!((f64::sub(1.0, 1.0)).abs() < f64::EPSILON);
}

#[test]
fn f32_sub() {
    assert!((f32::sub(10.0, 3.5) - 6.5).abs() < f32::EPSILON);
}

#[test]
fn sub_mirrors_add() {
    // For all types: add then sub recovers original.
    assert_eq!(u64::sub(u64::add(42, 8), 8), 42);
    assert!((f64::sub(f64::add(3.125, 2.0), 2.0) - 3.125).abs() < f64::EPSILON);
}

#[test]
#[should_panic(expected = "attempt to subtract with overflow")]
fn u64_sub_underflow_panics() {
    let _ = u64::sub(3, 10);
}

#[test]
#[should_panic(expected = "attempt to add with overflow")]
fn u64_add_overflow_panics() {
    let _ = u64::add(u64::MAX, 1);
}

// ── Inspectable tests ───────────────────────────────────────

#[test]
fn u64_to_f64_approx() {
    assert!((42u64.to_f64_approx() - 42.0).abs() < f64::EPSILON);
    assert!((0u64.to_f64_approx()).abs() < f64::EPSILON);
}

#[test]
#[allow(clippy::approx_constant)]
fn f64_to_f64_approx() {
    assert!((3.14f64.to_f64_approx() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn f32_to_f64_approx() {
    assert!((1.5f32.to_f64_approx() - 1.5).abs() < f64::EPSILON);
}

#[test]
fn u64_from_f64_round_trip() {
    assert_eq!(u64::from_f64(42.0), 42);
    // Truncation: fractional part dropped.
    assert_eq!(u64::from_f64(99.9), 99);
}

#[test]
#[allow(clippy::approx_constant)]
fn f64_from_f64_identity() {
    assert!((f64::from_f64(3.14) - 3.14).abs() < f64::EPSILON);
}

// ── Proratable tests ────────────────────────────────────────

#[test]
fn u64_prorate() {
    // 100 × (3/4) = 75.
    assert_eq!(u64::prorate(100, 3, 4), 75);
    // 100 × (1/3) = 33 (truncated).
    assert_eq!(u64::prorate(100, 1, 3), 33);
    // Full portion returns original.
    assert_eq!(u64::prorate(100, 4, 4), 100);
    // Zero portion returns zero.
    assert_eq!(u64::prorate(100, 0, 4), 0);
    // Edge: total = 0.
    assert_eq!(u64::prorate(100, 5, 0), 0);
}

#[test]
fn f64_prorate() {
    let p = f64::prorate(100.0, 3, 4);
    assert!((p - 75.0).abs() < 1e-10);
    assert!((f64::prorate(100.0, 0, 4)).abs() < f64::EPSILON);
    assert!((f64::prorate(100.0, 5, 0)).abs() < f64::EPSILON);
}

#[test]
fn u64_scale_by() {
    assert_eq!(100u64.scale_by(0.25), 25);
    assert_eq!(100u64.scale_by(1.0), 100);
    assert_eq!(100u64.scale_by(0.0), 0);
}

#[test]
fn f64_scale_by() {
    assert!((100.0f64.scale_by(0.25) - 25.0).abs() < f64::EPSILON);
    assert!((100.0f64.scale_by(1.0) - 100.0).abs() < f64::EPSILON);
}

// ── Attenuatable tests ──────────────────────────────────────

#[test]
fn u64_attenuate_four_regimes() {
    // Annihilation: factor = 0.
    assert_eq!(100u64.attenuate(0.0), 0);
    // Attenuation: factor in (0, 1).
    assert_eq!(100u64.attenuate(0.5), 50);
    // Identity: factor = 1.
    assert_eq!(100u64.attenuate(1.0), 100);
    // Amplification: factor > 1.
    assert_eq!(100u64.attenuate(2.0), 200);
}

#[test]
fn f64_attenuate_four_regimes() {
    assert!((100.0f64.attenuate(0.0)).abs() < f64::EPSILON);
    assert!((100.0f64.attenuate(0.5) - 50.0).abs() < f64::EPSILON);
    assert!((100.0f64.attenuate(1.0) - 100.0).abs() < f64::EPSILON);
    assert!((100.0f64.attenuate(2.0) - 200.0).abs() < f64::EPSILON);
}

#[test]
fn f32_attenuate_zero_guard() {
    // Both the zero-self and zero-factor guards return 0.0 (not NaN).
    assert!((0.0f32.attenuate(f64::INFINITY)).abs() < f32::EPSILON);
    assert!((100.0f32.attenuate(0.0)).abs() < f32::EPSILON);
}

// ── Weighable tests ─────────────────────────────────────────

#[test]
fn u64_weight() {
    assert!((42u64.weight() - 42.0).abs() < f64::EPSILON);
    assert!((0u64.weight()).abs() < f64::EPSILON);
}

#[test]
#[allow(clippy::approx_constant)]
fn f64_weight() {
    assert!((3.14f64.weight() - 3.14).abs() < f64::EPSILON);
}

#[test]
fn f32_weight() {
    assert!((1.5f32.weight() - 1.5).abs() < f64::EPSILON);
}

// ── Rng trait tests ─────────────────────────────────────────

#[test]
fn fixed_rng_returns_value() {
    let mut rng = FixedRng(0.42);
    assert!((rng.next_f64() - 0.42).abs() < f64::EPSILON);
    // Deterministic: same value again.
    assert!((rng.next_f64() - 0.42).abs() < f64::EPSILON);
}

#[test]
fn seq_rng_cycles() {
    let mut rng = SeqRng::new(vec![0.1, 0.9]);
    assert!((rng.next_f64() - 0.1).abs() < f64::EPSILON);
    assert!((rng.next_f64() - 0.9).abs() < f64::EPSILON);
    // Wraps around.
    assert!((rng.next_f64() - 0.1).abs() < f64::EPSILON);
}

#[test]
fn test_lcg_rng_in_range() {
    let mut rng = TestLcgRng(12345);
    for _ in 0..100 {
        let v = rng.next_f64();
        assert!((0.0..1.0).contains(&v), "LCG produced {v} outside [0, 1)");
    }
}

// ── Observation tests ───────────────────────────────────────

#[test]
fn same_type_accumulate() {
    // Blanket impl: V is Observation<V>.
    assert_eq!(<u64 as Observation<u64>>::accumulate(10, 5), 15);
    let r = <f64 as Observation<f64>>::accumulate(1.5, 2.5);
    assert!((r - 4.0).abs() < f64::EPSILON);
}

// Same-type `scale` no longer exists on `Observation` — factored
// into `ScalableObservation` (ADR-M-032 surface assignment test).
// The blanket `Observation<V> for V` does not impl
// `ScalableObservation` because `decay()` uses
// `Attenuatable::attenuate` directly (ADR-M-024).

#[test]
fn cross_type_f64_to_u16() {
    // 100 + 3.7 = 103.7 → truncates to 103.
    assert_eq!(<f64 as Observation<u16>>::accumulate(100, 3.7), 103);
    // 100 × 0.5 = 50.0 → 50.
    assert_eq!(<f64 as ScalableObservation<u16>>::scale(100, 0.5), 50);
}

#[test]
fn cross_type_f64_to_u64() {
    assert_eq!(<f64 as Observation<u64>>::accumulate(10, 5.9), 15);
    assert_eq!(<f64 as ScalableObservation<u64>>::scale(100, 0.75), 75);
}

#[test]
fn cross_type_f32_to_u16() {
    assert_eq!(<f32 as Observation<u16>>::accumulate(10, 3.2_f32), 13);
    assert_eq!(<f32 as ScalableObservation<u16>>::scale(100, 0.5_f32), 50);
}

#[test]
fn cross_type_f64_to_f32() {
    let result = <f64 as Observation<f32>>::accumulate(1.0_f32, 0.5_f64);
    assert!((result - 1.5_f32).abs() < f32::EPSILON);
    let scaled = <f64 as ScalableObservation<f32>>::scale(10.0_f32, 0.25_f64);
    assert!((scaled - 2.5_f32).abs() < f32::EPSILON);
}

#[test]
fn observation_total_zero_edge() {
    // f64 accumulate on zero.
    assert_eq!(<f64 as Observation<u64>>::accumulate(0, 5.0), 5);
    // Scale by zero.
    assert_eq!(<f64 as ScalableObservation<u64>>::scale(100, 0.0), 0);
}
