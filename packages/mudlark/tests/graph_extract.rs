// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `extract()`.
//!
//! Verify that `GvGraph::extract()` produces a correct PEWEI snapshot:
//! domain bounds, layer structure, field correctness for transitions
//! and terminals, energy conservation (G-I1), and agreement with
//! `total_energy()`. Also checks that `extract()` is read-only and
//! that invariants hold before and after extraction.
//!
//! # Test index
//!
//! ## Empty / single-node trees
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_empty_tree_single_terminal_layer`] | all-zero tree → 1 layer, 1 terminal |
//! | [`extract_single_root_with_intensity`] | single root with nonzero intensity |
//!
//! ## Domain bounds
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_domain_bounds_n8`] | domain `[0, 256)` for `N=8` |
//! | [`extract_domain_bounds_n64`] | domain `[0, u64::MAX)` for `N=64` |
//! | [`extract_f64_coordinates`] | domain `[0, 16)` for `f64, N=4` |
//!
//! ## Field correctness
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_transitions_have_correct_fields`] | `refinement = total - baseline`, valid regions |
//! | [`extract_terminals_have_correct_fields`] | valid regions, `v_depth` matches layer |
//! | [`extract_v_depth_matches_layer_index`] | `v_depth` correct for all nodes in all layers |
//! | [`extract_semi_internal_classified_as_transition`] | semi-internal G-nodes appear as transitions |
//!
//! ## Energy conservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_energy_conservation`] | sum of g.own values equals G-root sum (G-I1) |
//! | [`extract_total_energy_matches_manual_sum`] | `Pewei::total_energy()` agrees with manual sum |
//! | [`extract_total_energy_matches_graph_total_sum`] | `total_energy()` equals `GvGraph::total_sum()` |
//!
//! ## Domain tiling
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_reconstruct_partitions_domain`] | reconstruction tiles domain without gaps/overlaps |
//! | [`extract_reconstruct_energy_conserved`] | reconstruction span sum equals `total_energy()` |
//!
//! ## Invariant preservation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_invariants_hold_after`] | extraction is read-only; invariants before and after |
//!
//! ## Deeper / diverse topologies
//!
//! | Test | Focus |
//! |------|-------|
//! | [`extract_multi_observation_tree`] | larger tree with many observations |
//! | [`extract_node_count_matches_graph`] | `Pewei::node_count()` equals `GvGraph::node_count()` |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, default_config, f64_default_config, plan_range_tree, range_tree_config, run, run_checked};
use torrust_mudlark::{Accumulator, Config, GvGraph};

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

// ── Empty / single-node trees ───────────────────────────────────

#[test]
fn extract_empty_tree_single_terminal_layer() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let p = g.extract();
    assert_eq!(p.domain_start, 0);
    assert_eq!(p.domain_end, 256);
    assert_eq!(p.layer_count(), 1);
    assert_eq!(p.node_count(), 1);
    // Single terminal at root.
    assert_eq!(p.layers[0].transitions.len(), 0);
    assert_eq!(p.layers[0].terminals.len(), 1);
    let t = &p.layers[0].terminals[0];
    assert_eq!(t.start, 0);
    assert_eq!(t.end, 256);
    assert_eq!(t.intensity, 0);
    assert_eq!(t.depth, 0);
    assert_eq!(t.v_depth, 0);
}

#[test]
fn extract_single_root_with_intensity() {
    // High threshold prevents splitting; single root accumulates.
    let cfg = Config {
        split_threshold: 100,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(100u64, 42u64);
    let p = g.extract();
    assert_eq!(p.layer_count(), 1);
    assert_eq!(p.node_count(), 1);
    assert_eq!(p.layers[0].terminals.len(), 1);
    assert_eq!(p.layers[0].transitions.len(), 0);
    let t = &p.layers[0].terminals[0];
    assert_eq!(t.intensity, 42);
    assert_eq!(t.depth, 0);
}

// ── Domain bounds ───────────────────────────────────────────────

#[test]
fn extract_domain_bounds_n8() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let p = g.extract();
    assert_eq!(p.domain_start, 0);
    assert_eq!(p.domain_end, 256); // 2^8
}

#[test]
fn extract_domain_bounds_n64() {
    let g: GvGraph<u64, u64, 64> = GvGraph::new(default_config());
    let p = g.extract();
    assert_eq!(p.domain_start, 0);
    assert_eq!(p.domain_end, u64::MAX); // domain_max(64)
}

#[test]
fn extract_f64_coordinates() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(f64_default_config());
    g.observe(2.0_f64, 10.0_f64);
    g.observe(12.0_f64, 5.0_f64);

    let p = g.extract();
    assert!(p.layer_count() >= 1);
    assert!(p.node_count() >= 1);
    assert!((p.domain_start - 0.0).abs() < f64::EPSILON);
    assert!((p.domain_end - 16.0).abs() < f64::EPSILON); // 2^4
}

// ── Field correctness ───────────────────────────────────────────

#[test]
fn extract_transitions_have_correct_fields() {
    let g = build_range_tree();
    let p = g.extract();

    for layer in &p.layers {
        for tr in &layer.transitions {
            assert_eq!(tr.refinement, Accumulator::sub(tr.total, tr.baseline));
            assert!(tr.total >= tr.baseline);
            assert!(tr.start < tr.end);
        }
    }
}

#[test]
fn extract_terminals_have_correct_fields() {
    let g = build_range_tree();
    let p = g.extract();

    for (layer_idx, layer) in p.layers.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let expected_v_depth = layer_idx as u32;
        for term in &layer.terminals {
            assert!(term.start < term.end);
            assert_eq!(term.v_depth, expected_v_depth);
        }
    }
}

#[test]
fn extract_v_depth_matches_layer_index() {
    let g = build_range_tree();
    let p = g.extract();

    for (layer_idx, layer) in p.layers.iter().enumerate() {
        #[allow(clippy::cast_possible_truncation)]
        let expected_depth = layer_idx as u32;
        for tr in &layer.transitions {
            assert_eq!(tr.v_depth, expected_depth, "transition v_depth mismatch at layer {layer_idx}");
        }
        for term in &layer.terminals {
            assert_eq!(term.v_depth, expected_depth, "terminal v_depth mismatch at layer {layer_idx}");
        }
    }
}

#[test]
fn extract_semi_internal_classified_as_transition() {
    // Build a tree where at least one G-node is semi-internal:
    // only one child has split further. With θ=1 and specific
    // observations, the left child splits but right doesn't.
    let plan = Plan::new().observe(0u64, 3u64).observe(64u64, 3u64).observe(0u64, 10u64);
    let g = run_checked::<u64, u64, 8>(range_tree_config(), &plan, 1);

    let p = g.extract();
    // Collect all transitions — at least one should have
    // total > baseline (i.e. has children contributing energy).
    let transitions: Vec<_> = p.layers.iter().flat_map(|l| &l.transitions).collect();
    assert!(!transitions.is_empty(), "tree with splits must have at least one transition");
    // Every transition must have valid refinement.
    for tr in &transitions {
        assert_eq!(tr.refinement, Accumulator::sub(tr.total, tr.baseline));
    }
}

// ── Energy conservation ─────────────────────────────────────────

#[test]
fn extract_energy_conservation() {
    // G-I1: g.sum = g.own + children.sum (recursive).
    // Therefore root.sum = sum of all g.own across all G-nodes.
    let g = build_range_tree();
    let root_sum = g.total_sum();
    let p = g.extract();

    assert_eq!(p.node_count(), g.node_count() as usize);

    let own_sum: u64 = p
        .layers
        .iter()
        .flat_map(|l| {
            l.transitions
                .iter()
                .map(|t| t.baseline)
                .chain(l.terminals.iter().map(|t| t.intensity))
        })
        .sum();
    assert_eq!(own_sum, root_sum, "sum of all g.own values must equal G-root sum (G-I1)");
}

#[test]
fn extract_total_energy_matches_manual_sum() {
    let g = build_range_tree();
    let p = g.extract();

    let manual_sum: u64 = p
        .layers
        .iter()
        .flat_map(|l| {
            l.transitions
                .iter()
                .map(|t| t.baseline)
                .chain(l.terminals.iter().map(|t| t.intensity))
        })
        .sum();
    assert_eq!(p.total_energy(), manual_sum, "total_energy() must agree with manual sum");
}

#[test]
fn extract_total_energy_matches_graph_total_sum() {
    let g = build_range_tree();
    let p = g.extract();
    assert_eq!(
        p.total_energy(),
        g.total_sum(),
        "Pewei::total_energy() must equal GvGraph::total_sum()"
    );
}

// ── Domain tiling via reconstruct ───────────────────────────────

#[test]
fn extract_reconstruct_partitions_domain() {
    let g = build_range_tree();
    let p = g.extract();
    let spans = p.reconstruct(p.layer_count().saturating_sub(1));

    // Spans must tile [0, 256) with no gaps and no overlaps.
    assert_eq!(spans.first().unwrap().start, 0u64);
    assert_eq!(spans.last().unwrap().end, 256u64);
    for w in spans.windows(2) {
        assert_eq!(w[0].end, w[1].start, "gap or overlap between spans");
    }
}

#[test]
fn extract_reconstruct_energy_conserved() {
    let g = build_range_tree();
    let p = g.extract();
    let spans = p.reconstruct(p.layer_count().saturating_sub(1));

    let span_sum: u64 = spans.iter().map(|s| s.intensity).sum();
    // Integer prorate truncation may lose at most N units.
    assert!(
        p.total_energy().abs_diff(span_sum) <= 8,
        "reconstruction energy {span_sum} differs from total_energy {} by more than N=8",
        p.total_energy()
    );
}

// ── Invariant preservation ──────────────────────────────────────

#[test]
fn extract_invariants_hold_after() {
    // Extraction is read-only; invariants must hold before and after.
    let g = build_range_tree();
    assert_invariants(&g);
    let _p = g.extract();
    assert_invariants(&g);
}

// ── Deeper / diverse topologies ─────────────────────────────────

#[test]
fn extract_multi_observation_tree() {
    // A deeper tree built with many spread observations.
    let plan = Plan::new().spread(256, 6u64, 50);
    let g = run_checked::<u64, u64, 8>(range_tree_config(), &plan, 10);

    let p = g.extract();
    assert!(
        p.layer_count() >= 2,
        "50 spread observations under θ=1 should produce multiple layers"
    );
    assert!(p.node_count() >= 5, "50 spread observations should produce many nodes");

    // Energy conservation still holds.
    assert_eq!(p.total_energy(), g.total_sum());

    // All nodes have valid regions.
    for layer in &p.layers {
        for tr in &layer.transitions {
            assert!(tr.start < tr.end);
            assert!(tr.total >= tr.baseline);
        }
        for term in &layer.terminals {
            assert!(term.start < term.end);
        }
    }
}

#[test]
fn extract_node_count_matches_graph() {
    // Verify across different tree shapes.
    let configs_and_plans: Vec<(Config<u64>, Plan<u64, u64>)> = vec![
        // Empty tree.
        (default_config(), Plan::new()),
        // Range tree preset.
        (range_tree_config(), plan_range_tree()),
        // Spread observations.
        (range_tree_config(), Plan::new().spread(256, 6u64, 30)),
    ];

    for (cfg, plan) in &configs_and_plans {
        let g = run::<u64, u64, 8>(cfg.clone(), plan);
        let p = g.extract();
        assert_eq!(
            p.node_count(),
            g.node_count() as usize,
            "Pewei::node_count() must equal GvGraph::node_count()"
        );
    }
}
