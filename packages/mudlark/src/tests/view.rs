// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::gnode::GState;
use crate::handle::GNodeId;
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
