// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for the **`GvGraph::layers()` iterator**.
//!
//! `layers()` is the primary BFS-order traversal of the g-tree,
//! yielding `(layer_index, Node)` pairs.  These tests verify that the
//! iterator faithfully mirrors the internal structure exposed by
//! `extract()`, that every `Node` accessor (`width()`, `refinement()`,
//! `to_cell()`, `to_span()`, …) behaves correctly, and that the
//! iterator itself satisfies the standard `Iterator` contract across
//! various tree shapes, type combinations, and configurations.
//!
//! # Test index
//!
//! ## Empty / minimal trees
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_empty_tree`] | fresh graph yields single terminal root |
//! | [`layers_single_root_with_intensity`] | root with `own > 0` before split |
//!
//! ## Splitting behaviour
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_after_bootstrap_split`] | first split produces ≥2 nodes |
//! | [`layers_after_multiple_splits`] | range-tree preset, ≥3 nodes, BFS order |
//! | [`layers_all_states_present`] | dense observations yield all `GState` variants |
//!
//! ## Equivalence with `extract()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_count_matches_extract`] | node count parity across scenarios |
//! | [`layers_matches_extract_entries`] | field-level match (terminal/transition) |
//! | [`layers_order_matches_extract`] | same layer indices and (start, end) sets |
//!
//! ## Node field / method coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_start_lt_end`] | `start < end` for every node |
//! | [`layers_sum_ge_own`] | `sum >= own` for every node |
//! | [`layers_terminal_to_cell`] | `to_cell()` succeeds on terminals |
//! | [`layers_non_terminal_to_cell_is_none`] | `to_cell()` returns `None` on non-terminals |
//! | [`layers_to_span`] | `to_span()` field transfer |
//! | [`layers_width`] | `width()` equals `end - start` |
//! | [`layers_refinement`] | `refinement()` equals `sum - own` |
//! | [`layers_is_terminal_matches_state`] | `is_terminal()` agrees with `state` |
//! | [`layers_is_root_matches_parent`] | `is_root()` agrees with `parent` |
//! | [`layers_gnode_id_unique`] | all `gnode_id` values are distinct |
//! | [`layers_exactly_one_root`] | exactly one node with `parent.is_none()` |
//!
//! ## Iterator protocol
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_iterator_adapters`] | `count`, `last`, `filter`, `take`, `map` |
//!
//! ## Configurations / type combinations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`layers_budget_constrained`] | budgeted tree with evictions |
//! | [`layers_after_decay`] | f64 tree after `decay()` |
//! | [`layers_u32_u32`] | `<u32, u32, 16>` type combo |
//! | [`layers_f64_f64`] | `<f64, f64, 8>` type combo |
//! | [`layers_u128_f64`] | `<u128, f64, 64>` type combo |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{aggressive_config, default_config, f64_default_config, plan_range_tree, range_tree_config, run};
use torrust_mudlark::{Accumulator, Config, Coordinate, GState, GvGraph, Inspectable, Proratable};

/// Build a small range-tree via the shared preset.
fn build_range_tree() -> GvGraph<u64, u64, 8> {
    run::<u64, u64, 8>(range_tree_config(), &plan_range_tree())
}

/// Verify field-level equivalence between `layers()` and `extract()`.
#[allow(clippy::suspicious_operation_groupings)]
fn assert_layers_matches_extract<C, V, const N: u32>(graph: &GvGraph<C, V, N>)
where
    C: Coordinate + std::fmt::Debug,
    V: Accumulator + Inspectable + Proratable + std::fmt::Debug,
{
    let pewei = graph.extract();
    let from_layers: Vec<(usize, torrust_mudlark::Node<C, V>)> = graph.layers().collect();

    assert_eq!(pewei.node_count(), from_layers.len(), "node count mismatch");

    for (layer_idx, node) in &from_layers {
        assert!(
            *layer_idx < pewei.layers.len(),
            "layer index {layer_idx} out of bounds (only {} layers)",
            pewei.layers.len()
        );
        let layer = &pewei.layers[*layer_idx];
        #[allow(clippy::cast_possible_truncation)]
        let v_depth = *layer_idx as u32;
        match node.state {
            GState::Terminal => {
                let found = layer.terminals.iter().any(|t| {
                    t.start == node.start
                        && t.end == node.end
                        && t.intensity == node.own
                        && t.depth == node.depth
                        && t.v_depth == v_depth
                });
                assert!(found, "terminal not found in extract layer {layer_idx}: {node:?}");
            }
            GState::SemiInternal | GState::Internal => {
                let found = layer.transitions.iter().any(|t| {
                    t.start == node.start
                        && t.end == node.end
                        && t.baseline == node.own
                        && t.total == node.sum
                        && t.depth == node.depth
                        && t.v_depth == v_depth
                });
                assert!(found, "transition not found in extract layer {layer_idx}: {node:?}");
            }
        }
    }
}

// ── Empty / minimal trees ───────────────────────────────────────

#[test]
fn layers_empty_tree() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let entries: Vec<_> = g.layers().collect();
    assert_eq!(entries.len(), 1);
    let (layer, node) = &entries[0];
    assert_eq!(*layer, 0);
    assert_eq!(node.state, GState::Terminal);
    assert_eq!(node.own, 0);
    assert_eq!(node.sum, 0);
    assert_eq!(node.start, 0);
    assert_eq!(node.end, 256); // 2^8
}

#[test]
fn layers_single_root_with_intensity() {
    // θ=100 prevents splitting — root accumulates without children.
    let cfg = Config {
        split_threshold: 100u64,
        ..default_config()
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(42u64, 10u64);
    assert_invariants(&g);

    let entries: Vec<_> = g.layers().collect();
    assert_eq!(entries.len(), 1);
    let (layer, node) = &entries[0];
    assert_eq!(*layer, 0);
    assert_eq!(node.state, GState::Terminal);
    assert_eq!(node.own, 10);
    assert_eq!(node.sum, 10);
    assert!(node.is_root());
}

// ── Splitting behaviour ─────────────────────────────────────────

#[test]
fn layers_after_bootstrap_split() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(range_tree_config());
    // Multiple observations to guarantee at least one split.
    g.observe(32u64, 5u64);
    g.observe(32u64, 5u64);
    assert_invariants(&g);

    let count = g.layers().count();
    assert!(count >= 2, "bootstrap split should yield >= 2 nodes, got {count}");
    assert!(g.layers().next().is_some());
}

#[test]
fn layers_after_multiple_splits() {
    let g = build_range_tree();
    assert_invariants(&g);

    let entries: Vec<_> = g.layers().collect();
    assert!(entries.len() >= 3, "multiple splits should yield >= 3 nodes");

    // Layer indices must be non-decreasing in BFS order.
    for window in entries.windows(2) {
        assert!(
            window[0].0 <= window[1].0,
            "layer indices must be non-decreasing: {} > {}",
            window[0].0,
            window[1].0
        );
    }
}

#[test]
fn layers_all_states_present() {
    // θ=1 forces immediate splitting; many coords → all three states.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(range_tree_config());
    for &c in &[32u64, 96, 64, 128, 192, 16, 48, 80, 112, 200, 220, 240] {
        g.observe(c, 5u64);
    }
    assert_invariants(&g);

    let states: Vec<GState> = g.layers().map(|(_, n)| n.state).collect();
    assert!(states.contains(&GState::Terminal), "terminal state should be present");
    assert!(
        states.contains(&GState::Internal) || states.contains(&GState::SemiInternal),
        "at least one non-terminal state should be present with enough splits"
    );
}

// ── Equivalence with extract() ──────────────────────────────────

#[test]
fn layers_count_matches_extract() {
    // Fresh tree.
    let g0: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_eq!(g0.layers().count(), g0.extract().node_count());

    // After splits.
    let g1 = build_range_tree();
    assert_eq!(g1.layers().count(), g1.extract().node_count());

    // Budget-constrained (small domain forces evictions).
    let cfg = Config {
        split_threshold: 1u64,
        depth_create: 2,
        depth_evict: 4,
        budget: Some(30),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g2: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    for i in 0..30u64 {
        g2.observe(i % 16, 3u64);
    }
    assert_invariants(&g2);
    assert_eq!(g2.layers().count(), g2.extract().node_count());
}

#[test]
fn layers_matches_extract_entries() {
    // Fresh tree.
    let g0: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_layers_matches_extract(&g0);

    // After splits.
    let g1 = build_range_tree();
    assert_layers_matches_extract(&g1);

    // Aggressive config: θ=1, deeper tree.
    let mut g2: GvGraph<u64, u64, 8> = GvGraph::new(aggressive_config());
    for i in 0..20u64 {
        g2.observe(i * 12, 5u64);
    }
    assert_invariants(&g2);
    assert_layers_matches_extract(&g2);
}

#[test]
fn layers_order_matches_extract() {
    let g = build_range_tree();
    let from_layers: Vec<(usize, torrust_mudlark::Node<u64, u64>)> = g.layers().collect();
    let pewei = g.extract();

    // layers() yields entries layer-by-layer.  Group by layer and
    // compare the set of (start, end) per layer.
    let mut layers_by_idx: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = std::collections::BTreeMap::new();
    for (idx, node) in &from_layers {
        layers_by_idx.entry(*idx).or_default().push((node.start, node.end));
    }

    let mut extract_by_idx: std::collections::BTreeMap<usize, Vec<(u64, u64)>> = std::collections::BTreeMap::new();
    for (layer_idx, layer) in pewei.layers.iter().enumerate() {
        let mut entries = Vec::new();
        for tr in &layer.transitions {
            entries.push((tr.start, tr.end));
        }
        for term in &layer.terminals {
            entries.push((term.start, term.end));
        }
        if !entries.is_empty() {
            extract_by_idx.insert(layer_idx, entries);
        }
    }

    // Same layer indices.
    assert_eq!(
        layers_by_idx.keys().collect::<Vec<_>>(),
        extract_by_idx.keys().collect::<Vec<_>>(),
        "layer indices should match"
    );

    // Same (start, end) sets per layer (order within a layer
    // may differ between BFS orderings of Entry vs Structural,
    // so compare as sorted sets).
    for (idx, mut from_l) in layers_by_idx {
        let mut from_e = extract_by_idx.remove(&idx).unwrap();
        from_l.sort_unstable();
        from_e.sort_unstable();
        assert_eq!(from_l, from_e, "entries at layer {idx} should match");
    }
}

// ── Node field / method coverage ────────────────────────────────

#[test]
fn layers_start_lt_end() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        assert!(
            node.start < node.end,
            "start ({:?}) must be < end ({:?})",
            node.start,
            node.end
        );
    }
}

#[test]
fn layers_sum_ge_own() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        assert!(node.sum >= node.own, "sum ({:?}) must be >= own ({:?})", node.sum, node.own);
    }
}

#[test]
fn layers_terminal_to_cell() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        if node.state == GState::Terminal {
            let cell = node.to_cell();
            assert!(cell.is_some(), "terminal node should convert to Cell");
            let cell = cell.unwrap();
            assert_eq!(cell.intensity, node.own);
            assert_eq!(cell.start, node.start);
            assert_eq!(cell.end, node.end);
            assert_eq!(cell.depth, node.depth);
        }
    }
}

#[test]
fn layers_non_terminal_to_cell_is_none() {
    // Aggressive splitting to ensure non-terminals exist.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(range_tree_config());
    for &c in &[32u64, 96, 64, 128, 192, 16, 48] {
        g.observe(c, 5u64);
    }
    assert_invariants(&g);

    for (_layer, node) in g.layers() {
        if node.state != GState::Terminal {
            assert!(
                node.to_cell().is_none(),
                "non-terminal node should not convert to Cell: {node:?}"
            );
        }
    }
}

#[test]
fn layers_to_span() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        let span = node.to_span();
        assert_eq!(span.start, node.start);
        assert_eq!(span.end, node.end);
        assert_eq!(span.intensity, node.sum, "to_span uses sum as intensity");
        assert_eq!(span.depth, node.depth);
    }
}

#[test]
fn layers_width() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        assert_eq!(node.width(), node.end - node.start);
    }
}

#[test]
fn layers_refinement() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        let refinement = node.refinement();
        assert_eq!(
            refinement,
            Accumulator::sub(node.sum, node.own),
            "refinement should equal sum - own"
        );
        // refinement + own == sum
        assert_eq!(refinement + node.own, node.sum);

        if node.state == GState::Terminal {
            assert_eq!(refinement, 0, "terminal refinement must be zero");
        }
    }
}

#[test]
fn layers_is_terminal_matches_state() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        assert_eq!(
            node.is_terminal(),
            node.state == GState::Terminal,
            "is_terminal() must agree with state"
        );
    }
}

#[test]
fn layers_is_root_matches_parent() {
    let g = build_range_tree();
    for (_layer, node) in g.layers() {
        assert_eq!(
            node.is_root(),
            node.parent.is_none(),
            "is_root() must agree with parent.is_none()"
        );
    }
}

#[test]
fn layers_gnode_id_unique() {
    let g = build_range_tree();
    let ids: Vec<_> = g.layers().map(|(_, n)| n.gnode_id).collect();
    let unique: std::collections::HashSet<_> = ids.iter().collect();
    assert_eq!(ids.len(), unique.len(), "all gnode_id values must be distinct");
}

#[test]
fn layers_exactly_one_root() {
    let g = build_range_tree();
    let root_count = g.layers().filter(|(_, n)| n.is_root()).count();
    assert_eq!(root_count, 1, "exactly one node should have parent == None");
}

// ── Iterator protocol ───────────────────────────────────────────

#[test]
fn layers_iterator_adapters() {
    let g = build_range_tree();

    // count() works.
    let count = g.layers().count();
    assert!(count >= 1);

    // last() works.
    let last = g.layers().last();
    assert!(last.is_some());

    // filter() works.
    let terminal_count = g.layers().filter(|(_, n)| n.state == GState::Terminal).count();
    assert!(terminal_count >= 1);

    // take(1) is lazy — only visits enough to yield one node.
    assert_eq!(g.layers().take(1).count(), 1);

    // map() + for_each() work.
    let mut mapped_count = 0usize;
    g.layers().map(|(_, n)| n.start).for_each(|_| mapped_count += 1);
    assert_eq!(mapped_count, count);
}

// ── Configurations / type combinations ──────────────────────────

#[test]
fn layers_budget_constrained() {
    let cfg = Config {
        split_threshold: 1u64,
        depth_create: 2,
        depth_evict: 4,
        budget: Some(30),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    for i in 0..50u64 {
        g.observe(i % 16, 5u64);
        assert_invariants(&g);
    }
    assert_eq!(g.layers().count(), g.extract().node_count());
    assert_layers_matches_extract(&g);
}

#[test]
fn layers_after_decay() {
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(f64_default_config());
    g.observe(2.0_f64, 10.0_f64);
    g.observe(12.0_f64, 5.0_f64);
    assert_invariants(&g);

    let root = g.g_root();
    g.decay(root, 0.5, 0.0);
    assert_invariants(&g);

    // After decay, layers() should still work and match extract().
    assert_eq!(g.layers().count(), g.extract().node_count());
}

#[test]
fn layers_u32_u32() {
    let cfg = Config {
        split_threshold: 5u32,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u32, u32, 16> = GvGraph::new(cfg);
    g.observe(42u32, 10u32);
    assert_invariants(&g);
    assert!(g.layers().next().is_some());
    assert_eq!(g.layers().count(), g.extract().node_count());
}

#[test]
fn layers_f64_f64() {
    let cfg = Config {
        split_threshold: 1.0_f64,
        ..f64_default_config()
    };
    let mut g: GvGraph<f64, f64, 8> = GvGraph::new(cfg);
    g.observe(2.0_f64, 10.0_f64);
    g.observe(200.0_f64, 5.0_f64);
    assert_invariants(&g);
    assert!(g.layers().next().is_some());
    assert_eq!(g.layers().count(), g.extract().node_count());
}

#[test]
fn layers_u128_f64() {
    let cfg = Config {
        split_threshold: 1.0_f64,
        ..f64_default_config()
    };
    let mut g: GvGraph<u128, f64, 64> = GvGraph::new(cfg);
    g.observe(1000u128, 2.72_f64);
    assert_invariants(&g);
    assert!(g.layers().next().is_some());
    assert_eq!(g.layers().count(), g.extract().node_count());
}
