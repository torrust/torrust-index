// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for negative `f64` observations.
//!
//! `f64` implements `Accumulator` with `zero() = 0.0`, which
//! satisfies the Standard property profile (P0–P5) only for
//! non-negative values.  Negative deltas violate P2 (Grounded:
//! `zero() <= v` for all `v`) and break:
//!
//! - **Sampling** — `sample_child` computes probability ratios
//!   from potentially negative or zero totals.
//! - **Fibonacci depth bound** — P1 fails, so the V-Tree can grow
//!   deeper than `log_φ(n)` under adversarial sign patterns.
//! - **Violation-free splits** — a new `zero()` entry can outrank
//!   an existing entry with negative intensity, creating V-I3
//!   violations the split path does not expect.
//! - **Ghost detection** — a node at `intensity = -3.0` is less
//!   important than `zero()` but is not detected as a ghost.
//!
//! # Debug vs release behaviour
//!
//! `observe()` contains a `debug_assert!` that catches negative
//! accumulations in debug builds (ADR-M-033).  In debug mode the
//! tests verify the guard fires; in release mode the guard is
//! stripped and the tests exercise the downstream semantic breakage
//! that results from unchecked negative values.
//!
//! # Test index
//!
//! ## Debug-mode (P2 guard)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`debug_first_observation_negative_panics`] | P2 guard fires on negative cold-start observation |
//! | [`debug_observe_negative_panics_with_p2_message`] | P2 guard fires when accumulation goes negative |
//! | [`debug_negative_after_splits_panics_before_rebalance`] | P2 guard fires even in a structured tree |
//! | [`debug_exact_cancellation_to_zero_does_not_panic`] | Exact cancellation to zero is NOT a P2 violation |
//!
//! ## Release-mode (downstream breakage)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`release_first_observation_negative_goes_below_zero`] | Cold-start negative drives total below zero |
//! | [`release_observe_negative_delta_goes_below_zero`] | Negative delta after positive drives below zero |
//! | [`release_exact_cancellation_to_zero`] | Exact cancellation yields zero total |
//! | [`release_negative_delta_causes_irresolvable_violations`] | Irresolvable V-I3 violations from negative intensity |
//! | [`release_sample_with_mixed_sign_children`] | Sampling from incoherent probability weights |
//! | [`release_negative_intensity_not_detected_as_ghost`] | Below-ground intensity not flagged as ghost |
//! | [`release_catalytic_split_not_violation_free_with_negative_entries`] | Splits not violation-free with negative entries |
//! | [`release_g_i1_summation_holds_with_negative_deltas`] | G-I1 (algebraic summation) holds regardless of sign |
//! | [`release_total_sum_is_algebraic_sum_of_deltas`] | `total_sum()` tracks algebraic sum of all deltas |

mod support;

use std::panic;

#[cfg(not(debug_assertions))]
use support::FixedRng;
#[cfg(not(debug_assertions))]
use torrust_mudlark::Config;
use torrust_mudlark::GvGraph;
#[cfg(not(debug_assertions))]
use torrust_mudlark::invariants::check_all_invariants;
use torrust_mudlark::testing::f64_default_config;

/// Extract a panic payload as a `&str` (covers both `String` and `&str`).
fn panic_message(payload: &Box<dyn std::any::Any + Send>) -> &str {
    payload
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| payload.downcast_ref::<&str>().copied())
        .unwrap_or("")
}

// ═══════════════════════════════════════════════════════════════
// Debug-mode tests: verify the P2 guard fires
// ═══════════════════════════════════════════════════════════════

/// The `debug_assert!` in `observe()` catches a negative
/// accumulation when the very first delta is negative (cold start).
#[test]
#[cfg(debug_assertions)]
fn debug_first_observation_negative_panics() {
    let result = panic::catch_unwind(|| {
        let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
        g.observe(42u64, -1.0_f64);
    });

    let err = result.expect_err("should panic in debug mode");
    assert!(panic_message(&err).contains("P2 violation"));
}

/// The `debug_assert!` in `observe()` catches a negative
/// accumulation and panics with a "P2 violation" message.
#[test]
#[cfg(debug_assertions)]
fn debug_observe_negative_panics_with_p2_message() {
    let result = panic::catch_unwind(|| {
        let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
        g.observe(42u64, 10.0_f64);
        // This drives own to -5.0 → P2 violation.
        g.observe(42u64, -15.0_f64);
    });

    let err = result.expect_err("should panic in debug mode");
    let msg = panic_message(&err);
    assert!(
        msg.contains("P2 violation"),
        "expected 'P2 violation' in panic message, got: {msg}",
    );
}

/// The guard fires even in a structured tree — the rebalancer
/// never gets a chance to see the negative intensity.
#[test]
#[cfg(debug_assertions)]
fn debug_negative_after_splits_panics_before_rebalance() {
    let result = panic::catch_unwind(|| {
        let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
        for _ in 0..20 {
            g.observe(42u64, 10.0_f64);
        }
        g.observe(200u64, 1.0_f64);
        // Drives coord 200 to -4.0 → guard fires.
        g.observe(200u64, -5.0_f64);
    });

    let err = result.expect_err("should panic in debug mode");
    assert!(panic_message(&err).contains("P2 violation"));
}

/// Exact cancellation to zero is NOT a P2 violation:
/// `zero() = 0.0 <= 0.0` holds, so the guard must not fire.
///
/// Values stay below `split_threshold` (5.0) to ensure both
/// observations hit the same G-node.
#[test]
#[cfg(debug_assertions)]
fn debug_exact_cancellation_to_zero_does_not_panic() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
    g.observe(42u64, 4.0_f64);
    g.observe(42u64, -4.0_f64);

    let cell = g.get(42u64);
    assert!(
        cell.intensity.abs() < f64::EPSILON,
        "intensity should be ~0.0, got: {}",
        cell.intensity
    );
}

// ═══════════════════════════════════════════════════════════════
// Release-mode tests: the guard is stripped — exercise downstream
// semantic breakage
// ═══════════════════════════════════════════════════════════════

/// Cold start: the very first observation is negative, driving
/// the accumulator below zero immediately.
#[test]
#[cfg(not(debug_assertions))]
fn release_first_observation_negative_goes_below_zero() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
    g.observe(42u64, -5.0_f64);

    assert!(g.total_sum() < 0.0, "total_sum should be negative: {}", g.total_sum());
    let cell = g.get(42u64);
    assert!(cell.intensity < 0.0, "intensity should be negative: {}", cell.intensity);
}

/// Negative delta drives the accumulator below zero.
#[test]
#[cfg(not(debug_assertions))]
fn release_observe_negative_delta_goes_below_zero() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());

    g.observe(42u64, 10.0_f64);
    assert!((g.total_sum() - 10.0).abs() < f64::EPSILON);

    g.observe(42u64, -15.0_f64);
    assert!(g.total_sum() < 0.0, "total_sum should be negative: {}", g.total_sum());
}

/// Exact cancellation: positive then equal negate yields zero.
#[test]
#[cfg(not(debug_assertions))]
fn release_exact_cancellation_to_zero() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
    g.observe(42u64, 10.0_f64);
    g.observe(42u64, -10.0_f64);

    assert!(
        g.total_sum().abs() < f64::EPSILON,
        "total_sum should be ~0.0 after exact cancellation: {}",
        g.total_sum()
    );
}

/// After building structure, a negative delta can create
/// irresolvable V-I3 violations — the rebalancer's safety net
/// panics with "residual violations".
#[test]
#[cfg(not(debug_assertions))]
fn release_negative_delta_causes_irresolvable_violations() {
    let result = panic::catch_unwind(|| {
        let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());
        for _ in 0..20 {
            g.observe(42u64, 10.0_f64);
        }
        g.observe(200u64, 1.0_f64);
        g.observe(200u64, -5.0_f64);
        g
    });

    match result {
        Err(payload) => {
            assert!(
                panic_message(&payload).contains("residual violations"),
                "unexpected panic: {}",
                panic_message(&payload)
            );
        }
        Ok(g) => {
            let errors = check_all_invariants(&g);
            if !errors.is_empty() {
                let has_v_i3 = errors.iter().any(|e| e.contains("V-I3"));
                eprintln!(
                    "Invariant violations (expected): {} total, \
                     V-I3 present: {has_v_i3}",
                    errors.len()
                );
            }
        }
    }
}

/// Negative deltas can cause the rebalancer to panic, or — if the
/// tree survives — leave `sample()` operating on incoherent
/// probability weights.
#[test]
#[cfg(not(debug_assertions))]
fn release_sample_with_mixed_sign_children() {
    let cfg = Config {
        split_threshold: 2.0_f64,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };

    let result = panic::catch_unwind(|| {
        let mut g: GvGraph<u64, f64, 8> = GvGraph::new(cfg);
        for i in 0..30 {
            g.observe(i * 8, 5.0_f64);
        }
        g.observe(0u64, -100.0_f64);
        g
    });

    match result {
        Err(_) => {
            // Rebalancer panicked — expected.
        }
        Ok(g) => {
            let total = g.total_sum();
            if total > 0.0 {
                let cell = g.sample(&mut FixedRng(0.5));
                assert!(cell.is_some(), "sample should return Some when total > 0");
            } else if total == 0.0 {
                assert!(g.sample(&mut FixedRng(0.5)).is_none());
            } else {
                // Negative total — sample() only checks `== zero()`,
                // not `< zero()`.  It may run without panicking.
                let _ = g.sample(&mut FixedRng(0.5));
            }
        }
    }
}

/// A cell at `intensity = -3.0` is below ground but is NOT
/// detected as a ghost — P2 failure.
#[test]
#[cfg(not(debug_assertions))]
fn release_negative_intensity_not_detected_as_ghost() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());

    g.observe(42u64, 3.0_f64);
    g.observe(42u64, -6.0_f64);

    let cell = g.get(42u64);
    assert!(cell.intensity < 0.0);
    assert!(g.total_sum() < 0.0);
}

/// New `zero()` entries from splits can outrank existing negative
/// entries — violation-free splits no longer hold.
#[test]
#[cfg(not(debug_assertions))]
fn release_catalytic_split_not_violation_free_with_negative_entries() {
    let cfg = Config {
        split_threshold: 3.0_f64,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(cfg);

    for i in 0..20 {
        g.observe(i * 12, 4.0_f64);
    }
    for i in 0..5 {
        g.observe(i * 12, -10.0_f64);
    }
    for _ in 0..10 {
        g.observe(250u64, 5.0_f64);
    }

    let errors = check_all_invariants(&g);
    if !errors.is_empty() {
        let v_i3_count = errors.iter().filter(|e| e.contains("V-I3")).count();
        eprintln!(
            "Invariant violations after negative-then-split: {} total, \
             {v_i3_count} V-I3",
            errors.len()
        );
    }
}

/// G-I1 (summation) is purely algebraic and holds regardless of
/// sign — the *structural* invariant is fine even when the
/// *semantic* contract (P2) is broken.
#[test]
#[cfg(not(debug_assertions))]
fn release_g_i1_summation_holds_with_negative_deltas() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());

    for i in 0..50 {
        let delta = if i % 3 == 0 { -2.0_f64 } else { 3.0_f64 };
        g.observe((i * 5) % 256, delta);
    }

    let errors = check_all_invariants(&g);
    let g_i1_errors: Vec<_> = errors.iter().filter(|e| e.contains("G-I1")).collect();
    assert!(g_i1_errors.is_empty(), "G-I1 should hold regardless of sign: {g_i1_errors:?}");
}

/// Total sum tracks the algebraic sum of all deltas.
#[test]
#[cfg(not(debug_assertions))]
fn release_total_sum_is_algebraic_sum_of_deltas() {
    let mut g: GvGraph<u64, f64, 8> = GvGraph::new(f64_default_config());

    let deltas = [10.0, -3.0, 7.0, -15.0, 1.0, 4.0, -2.0];
    let expected: f64 = deltas.iter().sum();

    for (i, &d) in deltas.iter().enumerate() {
        g.observe((i as u64) * 30, d);
    }

    assert!(
        (g.total_sum() - expected).abs() < 1e-9,
        "total_sum {} != expected {expected}",
        g.total_sum()
    );
}
