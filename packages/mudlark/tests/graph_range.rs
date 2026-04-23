// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **`GvGraph::range_sum()`**.
//!
//! `range_sum` answers "how much energy falls within this coordinate
//! range?" by walking the G-tree and pro-rating node energy where a
//! query range partially overlaps a node's interval.  These tests
//! exercise every `RangeBounds` variant (`..`, `a..b`, `a..=b`,
//! `a..`, `..b`), both integer and floating-point coordinate types,
//! single-root and multi-node trees, and the expected panic paths
//! (NaN bounds, unsupported `RangeInclusive` on f64).
//!
//! # Test index
//!
//! ## Empty / zero trees
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_sum_zero_tree`] | fresh graph returns 0 for any range variant |
//!
//! ## Single-root (no split) — u64
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_sum_full_domain_equals_total_sum`] | `..` and explicit bounds both equal `total_sum()` |
//! | [`range_sum_empty_range_is_zero`] | `start == end` → 0 |
//! | [`range_sum_inverted_range_is_zero`] | `start > end` → 0 |
//! | [`range_sum_single_root_prorate`] | half / quarter domain pro-rates correctly |
//! | [`range_sum_clamps_out_of_range`] | bounds beyond domain clamped to full range |
//! | [`range_sum_inclusive_end`] | `0..=127` equivalent to `0..128` |
//! | [`range_sum_unbounded_start`] | `..128` covers left half |
//! | [`range_sum_unbounded_end`] | `128..` covers right half |
//!
//! ## Multi-node tree — u64
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_sum_multi_node_full_domain`] | full range equals `total_sum()` |
//! | [`range_sum_multi_node_halves_sum`] | complementary halves ≈ total (integer rounding) |
//! | [`range_sum_multi_node_quarters_sum`] | four quarters ≈ total (integer rounding) |
//! | [`range_sum_multi_node_invariants`] | invariants hold after checked build |
//!
//! ## Floating-point — f64
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_sum_f64_prorate`] | pro-rate on single root (`N = 4`) |
//! | [`range_sum_f64_unbounded`] | `..` on f64 graph equals observed delta |
//! | [`range_sum_f64_range_from`] | `128.0..` returns correct half |
//! | [`range_sum_f64_range_inclusive_panics`] | `..=` panics on f64 (no `next_value`) |
//!
//! ## Panic / edge cases
//!
//! | Test | Focus |
//! |------|-------|
//! | [`range_sum_nan_start_panics`] | NaN start bound panics |
//! | [`range_sum_nan_end_panics`] | NaN end bound panics |

use torrust_mudlark::GvGraph;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    default_config, f64_no_split_config, no_split_config, plan_range_tree, range_tree_config, run, run_checked,
};

// ── Helpers ─────────────────────────────────────────────────────

/// Build a multi-node range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

/// Build a single-root u64 graph with one observation.
fn build_single_root(coord: u64, delta: u64) -> GvGraph<u64, u64, 8> {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(no_split_config());
    g.observe(coord, delta);
    g
}

// ── Empty / zero trees ──────────────────────────────────────────

#[test]
fn range_sum_zero_tree() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_eq!(g.range_sum(..), 0);
    assert_eq!(g.range_sum(0u64..128u64), 0);
    assert_eq!(g.range_sum(0u64..=255u64), 0);
}

// ── Single-root (no split) — u64 ───────────────────────────────

#[test]
fn range_sum_full_domain_equals_total_sum() {
    let g = build_single_root(100, 10);
    let total = g.total_sum();
    // Full domain via unbounded range.
    assert_eq!(g.range_sum(..), total);
    // Full domain via explicit bounds (N=8 → domain [0, 256)).
    assert_eq!(g.range_sum(0u64..256u64), total);
    // Full domain via inclusive end.
    assert_eq!(g.range_sum(0u64..=255u64), total);
}

#[test]
fn range_sum_empty_range_is_zero() {
    let g = build_single_root(100, 10);
    // Start == end → empty.
    assert_eq!(g.range_sum(100u64..100u64), 0);
}

#[test]
fn range_sum_inverted_range_is_zero() {
    let g = build_single_root(100, 10);
    // Start > end → clamped/empty.
    #[allow(clippy::reversed_empty_ranges)]
    let sum = g.range_sum(200u64..100u64);
    assert_eq!(sum, 0);
}

#[test]
fn range_sum_single_root_prorate() {
    let g = build_single_root(100, 10);
    // Half domain → pro-rated: 10 * 128/256 = 5.
    assert_eq!(g.range_sum(0u64..128u64), 5);
    // Quarter domain → 10 * 64/256 = 2 (truncated for integers).
    assert_eq!(g.range_sum(0u64..64u64), 2);
}

#[test]
fn range_sum_clamps_out_of_range() {
    let g = build_single_root(100, 20);
    // Way beyond domain → clamped to full domain.
    assert_eq!(g.range_sum(0u64..1000u64), 20);
}

#[test]
fn range_sum_inclusive_end() {
    let g = build_single_root(100, 10);
    // 0..=127 → half-open [0, 128) → pro-rated 10*128/256 = 5.
    assert_eq!(g.range_sum(0u64..=127u64), 5);
}

#[test]
fn range_sum_unbounded_start() {
    let g = build_single_root(100, 10);
    // ..128 → [0, 128) → 5.
    assert_eq!(g.range_sum(..128u64), 5);
}

#[test]
fn range_sum_unbounded_end() {
    let g = build_single_root(100, 10);
    // 128.. → [128, 256) → 5.
    assert_eq!(g.range_sum(128u64..), 5);
}

// ── Multi-node tree — u64 ───────────────────────────────────────

#[test]
fn range_sum_multi_node_full_domain() {
    let g = build_range_tree();
    assert_eq!(g.range_sum(..), g.total_sum());
}

#[test]
fn range_sum_multi_node_halves_sum() {
    let g = build_range_tree();
    let total = g.total_sum();

    let left_half = g.range_sum(0u64..128u64);
    let right_half = g.range_sum(128u64..256u64);
    let sum_of_halves = left_half + right_half;

    // Allow rounding loss of up to 1 per internal node with own > 0.
    assert!(
        sum_of_halves <= total && total - sum_of_halves <= u64::from(g.node_count()),
        "halves={sum_of_halves}, total={total}, node_count={}",
        g.node_count()
    );
}

#[test]
fn range_sum_multi_node_quarters_sum() {
    let g = build_range_tree();
    let total = g.total_sum();

    let q1 = g.range_sum(0u64..64u64);
    let q2 = g.range_sum(64u64..128u64);
    let q3 = g.range_sum(128u64..192u64);
    let q4 = g.range_sum(192u64..256u64);
    let sum_of_quarters = q1 + q2 + q3 + q4;

    assert!(
        sum_of_quarters <= total && total - sum_of_quarters <= u64::from(g.node_count()),
        "quarters={sum_of_quarters}, total={total}, node_count={}",
        g.node_count()
    );
}

#[test]
fn range_sum_multi_node_invariants() {
    // Build with invariant checking at every observation.
    let g = run_checked::<u64, u64, 8>(range_tree_config(), &plan_range_tree(), 1);
    assert_invariants(&g);
    assert_eq!(g.range_sum(..), g.total_sum());
}

// ── Floating-point — f64 ────────────────────────────────────────

#[test]
fn range_sum_f64_prorate() {
    // Float coordinate: N=4, domain [0, 16).
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(f64_no_split_config());
    g.observe(5.0_f64, 16.0_f64); // single root, own=16
    // Full domain → 16.0.
    assert!((g.range_sum(..) - 16.0).abs() < f64::EPSILON);
    // Half → 8.0.
    assert!((g.range_sum(0.0..8.0) - 8.0).abs() < f64::EPSILON);
    // Quarter → 4.0.
    assert!((g.range_sum(0.0..4.0) - 4.0).abs() < f64::EPSILON);
}

#[test]
fn range_sum_f64_unbounded() {
    let mut g: GvGraph<f64, f64, 8> = GvGraph::new(f64_no_split_config());
    g.observe(100.0, 20.0);
    assert!((g.range_sum(..) - 20.0).abs() < f64::EPSILON);
}

#[test]
fn range_sum_f64_range_from() {
    // N=8, domain [0, 256). 128.. → [128, 256) → half.
    let mut g: GvGraph<f64, f64, 8> = GvGraph::new(f64_no_split_config());
    g.observe(100.0, 10.0);
    assert!((g.range_sum(128.0_f64..) - 5.0).abs() < f64::EPSILON);
}

#[test]
#[should_panic(expected = "next_value is not supported for f64")]
fn range_sum_f64_range_inclusive_panics() {
    // f64 coordinates do not support `RangeInclusive` because
    // `next_value()` is not defined for floats.
    let mut g: GvGraph<f64, f64, 8> = GvGraph::new(f64_no_split_config());
    g.observe(100.0, 256.0);
    let _unused = g.range_sum(0.0..=127.0);
}

// ── Panic / edge cases ──────────────────────────────────────────

#[test]
#[should_panic(expected = "NaN")]
fn range_sum_nan_start_panics() {
    let g: GvGraph<f64, u64, 8> = GvGraph::new(default_config());
    let _unused = g.range_sum(f64::NAN..10.0);
}

#[test]
#[should_panic(expected = "NaN")]
fn range_sum_nan_end_panics() {
    let g: GvGraph<f64, u64, 8> = GvGraph::new(default_config());
    let _unused = g.range_sum(0.0..f64::NAN);
}
