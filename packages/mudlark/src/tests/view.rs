// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Unit and integration tests for the **snapshot view types**:
//! [`Span`], [`Cell`], and [`Node`].
//!
//! These are the read-only projections that callers receive when
//! querying a `GvGraph`.  The unit tests construct instances directly
//! and verify field accessors, conversions, copy semantics, and
//! finality predicates for both `u64` and `f64` coordinate types.
//! The integration tests build a graph through [`GraphCreator`] and
//! exercise `get()` and `layers()`, confirming round-trip fidelity
//! and structural invariants such as `refinement() + own == sum`.
//!
//! # Test index
//!
//! ## Span (unit)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`span_width_u64`] | width arithmetic for integers |
//! | [`span_width_f64`] | width arithmetic for floats |
//! | [`span_copy_semantics`] | `Copy` derive |
//!
//! ## Cell (unit)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`cell_to_span`] | lossless conversion to `Span` |
//! | [`cell_to_span_f64`] | lossless conversion to `Span` (f64) |
//! | [`cell_width`] | width arithmetic |
//! | [`cell_is_final_true`] | unit-interval cell → final |
//! | [`cell_is_final_false`] | wide cell → not final |
//! | [`cell_is_final_f64`] | depth-based finality for floats |
//! | [`cell_copy_semantics`] | `Copy` derive |
//!
//! ## Node (unit)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`node_terminal_to_cell`] | Terminal → `Some(Cell)` |
//! | [`node_internal_to_cell_none`] | Internal → `None` |
//! | [`node_semi_internal_to_cell_none`] | `SemiInternal` → `None` |
//! | [`node_is_terminal`] | `is_terminal()` for all states |
//! | [`node_is_root_true`] | `is_root()` when `parent` is `None` |
//! | [`node_is_root_false`] | `is_root()` when `parent` is `Some` |
//! | [`node_to_span_uses_sum`] | `to_span()` maps `sum` to `intensity` |
//! | [`node_width`] | width arithmetic |
//! | [`node_copy_semantics`] | `Copy` derive |
//! | [`node_f64_types`] | all accessors with `<f64, f64>` |
//!
//! ## `Node::refinement` (unit)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`node_refinement_internal`] | `sum - own` for internal node |
//! | [`node_refinement_terminal_is_zero`] | 0 for terminal node |
//! | [`node_refinement_f64`] | float subtraction |
//!
//! ## Graph integration (via [`GraphCreator`])
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_returns_valid_cell`] | `get()` cell covers queried coord |
//! | [`layers_root_is_root`] | first `layers()` node has `is_root` |
//! | [`layers_terminals_have_zero_refinement`] | terminal refinement == 0 |
//! | [`layers_to_cell_round_trip`] | terminal node → cell → span preserves fields |

use crate::gnode::GState;
use crate::handle::GNodeId;
use crate::testing::GraphCreator;
use crate::{Cell, Node, Span};

// ── Span tests ──────────────────────────────────────────────

#[test]
fn span_width_u64() {
    let s = Span::<u64, u64> {
        start: 4,
        end: 8,
        intensity: 100,
        depth: 2,
    };
    assert_eq!(s.width(), 4);
}

#[test]
fn span_width_f64() {
    let s = Span::<f64, f64> {
        start: 0.0,
        end: 16.0,
        intensity: 3.5,
        depth: 0,
    };
    assert!((s.width() - 16.0).abs() < f64::EPSILON);
}

#[test]
fn span_copy_semantics() {
    let a = Span::<u64, u64> {
        start: 0,
        end: 4,
        intensity: 10,
        depth: 1,
    };
    let b = a; // Copy
    assert_eq!(a, b);
}

// ── Cell tests ──────────────────────────────────────────────

#[test]
fn cell_to_span() {
    let cell = Cell::<u64, u64> {
        start: 2,
        end: 4,
        intensity: 50,
        depth: 3,
    };
    let span = cell.to_span();
    assert_eq!(span.start, 2);
    assert_eq!(span.end, 4);
    assert_eq!(span.intensity, 50);
    assert_eq!(span.depth, 3);
}

#[test]
fn cell_to_span_f64() {
    let cell = Cell::<f64, f64> {
        start: 1.5,
        end: 3.0,
        intensity: 7.25,
        depth: 2,
    };
    let span = cell.to_span();
    assert!((span.start - 1.5).abs() < f64::EPSILON);
    assert!((span.end - 3.0).abs() < f64::EPSILON);
    assert!((span.intensity - 7.25).abs() < f64::EPSILON);
    assert_eq!(span.depth, 2);
}

#[test]
fn cell_width() {
    let cell = Cell::<u64, u64> {
        start: 0,
        end: 8,
        intensity: 1,
        depth: 1,
    };
    assert_eq!(cell.width(), 8);
}

#[test]
fn cell_is_final_true() {
    // Width-1 integer cell at any depth → final.
    let cell = Cell::<u64, u64> {
        start: 5,
        end: 6,
        intensity: 10,
        depth: 4,
    };
    assert!(cell.is_final(4));
}

#[test]
fn cell_is_final_false() {
    let cell = Cell::<u64, u64> {
        start: 4,
        end: 8,
        intensity: 10,
        depth: 2,
    };
    assert!(!cell.is_final(4));
}

#[test]
fn cell_is_final_f64() {
    let cell = Cell::<f64, f64> {
        start: 0.0,
        end: 1.0,
        intensity: 1.0,
        depth: 8,
    };
    // depth == n → final for floats.
    assert!(cell.is_final(8));
    // depth < n → not final.
    assert!(!cell.is_final(16));
}

#[test]
fn cell_copy_semantics() {
    let a = Cell::<u64, u64> {
        start: 0,
        end: 1,
        intensity: 42,
        depth: 5,
    };
    let b = a;
    assert_eq!(a, b);
}

// ── Node tests ──────────────────────────────────────────────

#[test]
fn node_terminal_to_cell() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 4,
        own: 10,
        sum: 10,
        depth: 2,
        state: GState::Terminal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    let cell = node.to_cell().expect("terminal → Some(Cell)");
    assert_eq!(cell.start, 0);
    assert_eq!(cell.end, 4);
    assert_eq!(cell.intensity, 10);
    assert_eq!(cell.depth, 2);
}

#[test]
fn node_internal_to_cell_none() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 8,
        own: 5,
        sum: 25,
        depth: 1,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!(node.to_cell().is_none());
}

#[test]
fn node_semi_internal_to_cell_none() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 8,
        own: 5,
        sum: 15,
        depth: 1,
        state: GState::SemiInternal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!(node.to_cell().is_none());
}

#[test]
fn node_is_terminal() {
    let terminal = Node::<u64, u64> {
        start: 0,
        end: 1,
        own: 1,
        sum: 1,
        depth: 3,
        state: GState::Terminal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!(terminal.is_terminal());

    let internal = Node::<u64, u64> {
        start: 0,
        end: 4,
        own: 1,
        sum: 10,
        depth: 1,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!(!internal.is_terminal());
}

#[test]
fn node_is_root_true() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 256,
        own: 10,
        sum: 100,
        depth: 0,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!(node.is_root());
}

#[test]
fn node_is_root_false() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 128,
        own: 5,
        sum: 50,
        depth: 1,
        state: GState::Terminal,
        gnode_id: GNodeId::from_index(1),
        parent: Some(GNodeId::from_index(0)),
    };
    assert!(!node.is_root());
}

#[test]
fn node_to_span_uses_sum() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 16,
        own: 3,
        sum: 42,
        depth: 0,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    let span = node.to_span();
    assert_eq!(span.intensity, 42); // sum, not own
}

#[test]
fn node_width() {
    let node = Node::<u64, u64> {
        start: 8,
        end: 16,
        own: 0,
        sum: 0,
        depth: 1,
        state: GState::Terminal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert_eq!(node.width(), 8);
}

#[test]
fn node_copy_semantics() {
    let a = Node::<u64, u64> {
        start: 0,
        end: 8,
        own: 5,
        sum: 20,
        depth: 1,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    let b = a;
    assert_eq!(a, b);
}

#[test]
fn node_f64_types() {
    let node = Node::<f64, f64> {
        start: 0.0,
        end: 8.0,
        own: 1.5,
        sum: 4.5,
        depth: 1,
        state: GState::SemiInternal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!((node.width() - 8.0).abs() < f64::EPSILON);
    assert!(node.to_cell().is_none());
    let span = node.to_span();
    assert!((span.intensity - 4.5).abs() < f64::EPSILON);
}

// ── Node::refinement tests ──────────────────────────────────

#[test]
fn node_refinement_internal() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 8,
        own: 5,
        sum: 25,
        depth: 1,
        state: GState::Internal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert_eq!(node.refinement(), 20); // 25 - 5
}

#[test]
fn node_refinement_terminal_is_zero() {
    let node = Node::<u64, u64> {
        start: 0,
        end: 4,
        own: 10,
        sum: 10,
        depth: 2,
        state: GState::Terminal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert_eq!(node.refinement(), 0);
}

#[test]
fn node_refinement_f64() {
    let node = Node::<f64, f64> {
        start: 0.0,
        end: 8.0,
        own: 1.5,
        sum: 4.5,
        depth: 1,
        state: GState::SemiInternal,
        gnode_id: GNodeId::from_index(0),
        parent: None,
    };
    assert!((node.refinement() - 3.0).abs() < f64::EPSILON);
}

// ── Graph integration (via GraphCreator) ────────────────────

#[test]
fn get_returns_valid_cell() {
    let g = GraphCreator::default_u64().hotspot(42, 10, 6).spread(256, 1, 20).build::<8>();

    let cell = g.get(42);
    assert!(cell.start <= 42 && 42 < cell.end);
    assert!(cell.intensity > 0);
    assert!(cell.width() > 0);

    // to_span preserves all fields.
    let span = cell.to_span();
    assert_eq!(span.start, cell.start);
    assert_eq!(span.end, cell.end);
    assert_eq!(span.intensity, cell.intensity);
    assert_eq!(span.depth, cell.depth);
}

#[test]
fn layers_root_is_root() {
    let g = GraphCreator::default_u64().hotspot(100, 5, 10).build::<8>();

    // The first node in layers() ordering is the root.
    let (_, root) = g.layers().next().expect("non-empty graph has layers");
    assert!(root.is_root(), "first layers() node should be the root");
    assert_eq!(root.depth, 0);
    assert!(!root.is_terminal(), "root with children is not terminal");
}

#[test]
fn layers_terminals_have_zero_refinement() {
    let g = GraphCreator::default_u64().spread(256, 2, 30).hotspot(10, 5, 8).build::<8>();

    for (_, node) in g.layers() {
        if node.is_terminal() {
            assert_eq!(
                node.refinement(),
                0,
                "terminal at [{}, {}) depth {} should have zero refinement",
                node.start,
                node.end,
                node.depth,
            );
            assert_eq!(node.own, node.sum);
        }
        // Invariant: refinement + own == sum.
        assert_eq!(node.refinement() + node.own, node.sum);
    }
}

#[test]
fn layers_to_cell_round_trip() {
    let g = GraphCreator::default_u64().hotspot(50, 3, 12).build::<8>();

    for (_, node) in g.layers() {
        if node.is_terminal() {
            let cell = node.to_cell().expect("terminal → Some(Cell)");
            assert_eq!(cell.start, node.start);
            assert_eq!(cell.end, node.end);
            assert_eq!(cell.intensity, node.own);
            assert_eq!(cell.depth, node.depth);

            // Cell → Span should match Node → Span only for terminals
            // (where own == sum).
            let cell_span = cell.to_span();
            let node_span = node.to_span();
            assert_eq!(cell_span.start, node_span.start);
            assert_eq!(cell_span.end, node_span.end);
            assert_eq!(cell_span.depth, node_span.depth);
            assert_eq!(cell_span.intensity, node_span.intensity);
        } else {
            assert!(node.to_cell().is_none());
        }
    }
}
