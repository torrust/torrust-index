// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::traits::Coordinate;
use crate::{Accumulator, Inspectable, Observation, Proratable, Rng, ScalableObservation};

// ── Coordinate tests ────────────────────────────────────────

#[test]
fn u64_domain_max() {
    assert_eq!(u64::domain_max(32), 1u64 << 32);
    assert_eq!(u64::domain_max(64), u64::MAX);
}

#[test]
fn u64_midpoint() {
    assert_eq!(u64::midpoint(0, 8), 4);
    assert_eq!(u64::midpoint(4, 8), 6);
    assert_eq!(u64::midpoint(0, 1), 0);
}

#[test]
fn u64_is_final() {
    assert!(u64::is_final(3, 4, 0, 0));
    assert!(!u64::is_final(0, 4, 0, 0));
}

#[test]
fn u128_domain_max_full() {
    assert_eq!(u128::domain_max(128), u128::MAX);
    assert_eq!(u128::domain_max(64), 1u128 << 64);
}

#[test]
fn f64_midpoint() {
    assert!((f64::midpoint(0.0, 8.0) - 4.0).abs() < f64::EPSILON);
}

#[test]
fn f64_is_final() {
    assert!(f64::is_final(0.0, 1.0, 52, 52));
    assert!(!f64::is_final(0.0, 1.0, 10, 52));
}

// ── Accumulator tests ───────────────────────────────────────

#[test]
fn u64_accumulator() {
    assert_eq!(<u64 as Accumulator>::zero(), 0);
    assert_eq!(u64::add(3, 5), 8);
    assert!((<u64 as Inspectable>::to_f64_approx(42) - 42.0).abs() < f64::EPSILON);
}

#[test]
fn u64_prorate() {
    // 100 × (3/4) = 75
    assert_eq!(u64::prorate(100, 3, 4), 75);
    // 100 × (1/3) = 33 (truncated)
    assert_eq!(u64::prorate(100, 1, 3), 33);
    // Edge: total = 0
    assert_eq!(u64::prorate(100, 5, 0), 0);
}

#[test]
fn f64_accumulator() {
    assert!((f64::add(1.5, 2.5) - 4.0).abs() < f64::EPSILON);
    let p = f64::prorate(100.0, 3, 4);
    assert!((p - 75.0).abs() < 1e-10);
}

// ── Accumulator::sub tests ──────────────────────────────────

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
#[should_panic(expected = "attempt to subtract with overflow")]
fn u64_sub_underflow_panics() {
    let _ = u64::sub(3, 10);
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

// ── Rng trait tests ─────────────────────────────────────────

/// Deterministic Rng for testing.
struct FixedRng(f64);

impl Rng for FixedRng {
    fn next_f64(&mut self) -> f64 {
        self.0
    }
}

#[test]
fn fixed_rng_returns_value() {
    let mut rng = FixedRng(0.42);
    assert!((rng.next_f64() - 0.42).abs() < f64::EPSILON);
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
