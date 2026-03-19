// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `extract()`.

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{default_config, plan_range_tree, range_tree_config, run};
use torrust_mudlark::{Accumulator, Config, GvGraph};

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

#[test]
fn extract_empty_tree_single_terminal_layer() {
    // DC-021-4: all-zero tree → 1 layer, 1 terminal.
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
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(Config {
        split_threshold: 100,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
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

#[test]
fn extract_transitions_have_correct_fields() {
    let g = build_range_tree();
    let p = g.extract();

    for layer in &p.layers {
        for tr in &layer.transitions {
            // refinement = total - baseline.
            assert_eq!(tr.refinement, Accumulator::sub(tr.total, tr.baseline));
            // Non-terminal must have total >= baseline (children add energy).
            assert!(tr.total >= tr.baseline);
            // Region is valid.
            assert!(tr.start < tr.end);
        }
    }
}

#[test]
fn extract_terminals_have_correct_fields() {
    let g = build_range_tree();
    let p = g.extract();

    for layer in &p.layers {
        for term in &layer.terminals {
            // Terminal: intensity = g.own = g.sum.
            // Verify region is valid.
            assert!(term.start < term.end);
            // v_depth matches the layer index.
            let layer_idx = p.layers.iter().position(|l| l.terminals.contains(term)).unwrap();
            #[allow(clippy::cast_possible_truncation)]
            let expected_depth = layer_idx as u32;
            assert_eq!(term.v_depth, expected_depth);
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
fn extract_energy_conservation() {
    // G-I1: g.sum = g.own + children.sum (recursive).
    // Therefore root.sum = sum of all g.own across all G-nodes.
    // Extraction emits baseline (= g.own) for transitions and
    // intensity (= g.own) for terminals. Their sum must equal
    // root.sum.
    let g = build_range_tree();
    let root_sum = g.total_sum();
    let p = g.extract();

    // All extracted nodes correspond to exactly the live G-nodes.
    assert_eq!(p.node_count(), g.node_count() as usize);

    // Sum of all g.own values (baseline for transitions,
    // intensity for terminals) must equal G-root sum.
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
fn extract_domain_bounds() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let p = g.extract();
    assert_eq!(p.domain_start, 0);
    assert_eq!(p.domain_end, 256); // 2^8

    let g64: GvGraph<u64, u64, 64> = GvGraph::new(default_config());
    let p64 = g64.extract();
    assert_eq!(p64.domain_start, 0);
    assert_eq!(p64.domain_end, u64::MAX); // domain_max(64)
}

#[test]
fn extract_f64_coordinates() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(Config {
        split_threshold: 1.0,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(2.0_f64, 10.0_f64);
    g.observe(12.0_f64, 5.0_f64);

    let p = g.extract();
    assert!(p.layer_count() >= 1);
    assert!(p.node_count() >= 2);
    assert!((p.domain_start - 0.0).abs() < f64::EPSILON);
    assert!((p.domain_end - 16.0).abs() < f64::EPSILON); // 2^4
}

#[test]
fn extract_invariants_hold_after() {
    // Extraction is read-only; invariants must hold before and after.
    let g = build_range_tree();
    assert_invariants(&g);
    let _p = g.extract();
    assert_invariants(&g);
}
