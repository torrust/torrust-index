// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `range_sum()`.

use torrust_mudlark::testing::{plan_range_tree, range_tree_config, run};
use torrust_mudlark::{Config, GvGraph};

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

#[test]
fn range_sum_full_domain_equals_root_sum() {
    let g = build_range_tree();
    let root_sum = g.total_sum();
    // Full domain via ..
    assert_eq!(g.range_sum(..), root_sum);
    // Full domain via explicit bounds.
    assert_eq!(g.range_sum(0u64..256u64), root_sum);
}

#[test]
fn range_sum_empty_range_is_zero() {
    let g = build_range_tree();
    // Start == end → empty.
    assert_eq!(g.range_sum(100u64..100u64), 0);
}

#[test]
fn range_sum_zero_tree() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    assert_eq!(g.range_sum(..), 0);
    assert_eq!(g.range_sum(0u64..128u64), 0);
}

#[test]
fn range_sum_single_root_partial() {
    // Single root node with own=10, no children.
    let cfg = Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(100u64, 10u64); // all goes to root.own
    // Full domain → 10.
    assert_eq!(g.range_sum(..), 10);
    // Half domain → pro-rated: 10 * 128/256 = 5.
    assert_eq!(g.range_sum(0u64..128u64), 5);
    // Quarter domain → 10 * 64/256 = 2 (truncated for integers).
    assert_eq!(g.range_sum(0u64..64u64), 2);
}

#[test]
fn range_sum_clamps_out_of_range() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u64, 20u64);
    // Way beyond domain → clamped to full domain.
    assert_eq!(g.range_sum(0u64..1000u64), 20);
}

#[test]
fn range_sum_inclusive_end() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u64, 10u64);
    // 0..=127 → half-open [0, 128) → pro-rated 10*128/256 = 5.
    assert_eq!(g.range_sum(0u64..=127u64), 5);
}

#[test]
fn range_sum_unbounded_start() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u64, 10u64);
    // ..128 → [0, 128) → 5.
    assert_eq!(g.range_sum(..128u64), 5);
}

#[test]
fn range_sum_unbounded_end() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u64, 10u64);
    // 128.. → [128, 256) → 5.
    assert_eq!(g.range_sum(128u64..), 5);
}

#[test]
fn range_sum_multi_node_structure() {
    let g = build_range_tree();
    let total = g.total_sum();

    // The full range must equal root.sum.
    assert_eq!(g.range_sum(..), total);

    // Two complementary halves must sum to root.sum (with
    // possible integer truncation loss of at most 1 per
    // pro-rated node with odd own value).
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
#[should_panic(expected = "NaN")]
fn range_sum_nan_start_panics() {
    let g: GvGraph<f64, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    let _unused = g.range_sum(f64::NAN..10.0);
}

#[test]
#[should_panic(expected = "NaN")]
fn range_sum_nan_end_panics() {
    let g: GvGraph<f64, u64, 8> = GvGraph::new(Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    let _unused = g.range_sum(0.0..f64::NAN);
}

#[test]
fn range_sum_f64_prorate() {
    // Float coordinate: N=4, domain [0, 16).
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 100.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(5.0_f64, 16.0_f64); // single root, own=16
    // Full domain → 16.0.
    assert!((g.range_sum(..) - 16.0).abs() < f64::EPSILON);
    // Half → 8.0.
    assert!((g.range_sum(0.0..8.0) - 8.0).abs() < f64::EPSILON);
    // Quarter → 4.0.
    assert!((g.range_sum(0.0..4.0) - 4.0).abs() < f64::EPSILON);
}
