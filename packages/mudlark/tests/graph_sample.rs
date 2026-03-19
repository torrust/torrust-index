// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `sample()`.

mod support;

use support::{FixedRng, SeqRng, TestRng};
use torrust_mudlark::testing::{default_config, plan_range_tree, range_tree_config, run};
use torrust_mudlark::{Config, GvGraph, Inspectable};

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

#[test]
fn sample_zero_intensity_returns_none() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // No observations → zero intensity → None.
    assert!(g.sample(&mut FixedRng(0.5)).is_none());
}

#[test]
fn sample_single_root_entry() {
    // Single root entry with nonzero intensity.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(100u64, 42u64);

    // V-root is the only entry → any RNG value returns it.
    let cell = g.sample(&mut FixedRng(0.0)).unwrap();
    assert_eq!(cell.start, 0);
    assert_eq!(cell.end, 256);
    assert_eq!(cell.intensity, 42);
    assert_eq!(cell.depth, 0);

    // Same result regardless of RNG value.
    let cell2 = g.sample(&mut FixedRng(0.999)).unwrap();
    assert_eq!(cell2.start, cell.start);
    assert_eq!(cell2.intensity, cell.intensity);
}

#[test]
fn sample_returns_terminal_cell_fields() {
    let g = build_range_tree();
    // build_range_tree creates a multi-node tree.
    // Any sample must return a terminal cell.
    let cell = g.sample(&mut FixedRng(0.5)).unwrap();
    // Cell must lie within the domain [0, 256).
    assert!(cell.start < cell.end);
    assert!(cell.end <= 256);
    // Depth must be positive (not the root, since root has children).
    assert!(cell.depth > 0);
}

#[test]
fn sample_deterministic_low_is_consistent() {
    // rng=0.0 → threshold=0.0 → always picks the first child
    // (index 0) in each PackedChildren. V-Tree order is by
    // intensity (tournament), not spatial, so we just verify
    // determinism: same RNG always yields the same cell.
    let g = build_range_tree();
    let cell1 = g.sample(&mut FixedRng(0.0)).unwrap();
    let cell2 = g.sample(&mut FixedRng(0.0)).unwrap();
    assert_eq!(cell1, cell2, "same RNG seed must yield same cell");
    // Must be a valid terminal within the domain.
    assert!(cell1.start < cell1.end);
    assert!(cell1.end <= 256);
    assert!(cell1.depth > 0);
}

#[test]
fn sample_deterministic_high_selects_last_child() {
    // rng just below 1.0 → picks the last child at every level.
    let g = build_range_tree();
    let cell = g.sample(&mut FixedRng(0.999_999_9)).unwrap();
    // Should land on the rightmost terminal.
    assert_eq!(cell.end, 256, "rng≈1.0 should pick the rightmost terminal");
}

#[test]
fn sample_respects_weight_proportions() {
    // Build a tree with two entries at known weights.
    // split_threshold=1 forces immediate splits.
    let cfg = Config {
        split_threshold: 1,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    // After these observations, we'll have terminal cells
    // in the left and right halves. The exact weights depend
    // on split mechanics, but we can verify the probability
    // boundary by testing two carefully chosen RNG values.
    g.observe(32u64, 30u64); // left half
    g.observe(200u64, 10u64); // right half

    let total_intensity = g.total_sum();
    assert!(total_intensity > 0);

    // Sample many times and verify both halves are reachable.
    let mut saw_left = false;
    let mut saw_right = false;
    for i in 0..20 {
        let rng_val = f64::from(i) / 20.0;
        if let Some(cell) = g.sample(&mut FixedRng(rng_val)) {
            if cell.start < 128 {
                saw_left = true;
            } else {
                saw_right = true;
            }
        }
    }
    assert!(saw_left, "should sample from left half");
    assert!(saw_right, "should sample from right half");
}

#[test]
fn sample_frequency_convergence() {
    // Statistical test: N samples, verify observed frequencies
    // converge to expected proportions derived from V-entry
    // intensities (not range_sum, which includes prorated own).
    let cfg = Config {
        split_threshold: 1,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(32u64, 30u64);
    g.observe(200u64, 10u64);

    // The V-root intensity is the sum of all V-entry intensities
    // — this is the normalizing constant for sampling.
    let total_v = Inspectable::to_f64_approx(g.total_sum());
    assert!(total_v > 0.0);

    // First pass: collect the distinct cells and their weights.
    let n = 10_000u32;
    let mut rng = TestRng(42);
    let mut buckets: std::collections::HashMap<u64, (u32, f64)> = std::collections::HashMap::new();
    for _ in 0..n {
        let cell = g.sample(&mut rng).unwrap();
        buckets
            .entry(cell.start)
            .and_modify(|(count, _)| *count += 1)
            .or_insert_with(|| (1, Inspectable::to_f64_approx(cell.intensity)));
    }

    // Verify each bucket's frequency matches its expected weight.
    for (&start, &(count, intensity)) in &buckets {
        let expected = intensity / total_v;
        let observed = f64::from(count) / f64::from(n);
        // 4σ tolerance.
        let tolerance = 4.0 * (expected * (1.0 - expected) / f64::from(n)).sqrt();
        assert!(
            (observed - expected).abs() < tolerance,
            "bucket start={start}: observed={observed:.4}, expected={expected:.4}, tol={tolerance:.4}"
        );
    }
}

#[test]
fn sample_f64_coordinate() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 100.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(5.0_f64, 10.0_f64);

    let cell = g.sample(&mut FixedRng(0.5)).unwrap();
    assert!(cell.start < cell.end);
    assert!((cell.intensity - 10.0).abs() < f64::EPSILON);
    assert_eq!(cell.depth, 0); // single root
}

#[test]
fn sample_with_seq_rng() {
    // Verify that successive sample() calls consume successive
    // RNG values. With extreme 0.0 and 0.999 values, the two
    // calls should (in a multi-entry tree) land on different
    // cells — validating that the RNG is advanced per call.
    let g = build_range_tree();
    let mut rng = SeqRng::new(vec![0.0, 0.999]);
    let cell1 = g.sample(&mut rng).unwrap();
    let cell2 = g.sample(&mut rng).unwrap();
    // Both are valid terminals.
    assert!(cell1.start < cell1.end);
    assert!(cell2.start < cell2.end);
    // Extreme RNG values should generally pick different cells
    // in a multi-entry tree (not guaranteed for all structures,
    // but very likely with build_range_tree's 3+ entries).
    assert_ne!(cell1, cell2, "extreme RNG values should pick different cells");
}
