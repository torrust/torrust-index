// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::mem::size_of;

use crate::GNodeId;
use crate::gnode::{GNode, GState};

#[test]
fn gnode_size() {
    // 2×u64 (lo, hi) = 16
    // 2×u64 (sum, own) = 16
    // 4×Option<handle> = 16
    // Total = 48 — no stored state field (ADR-M-016).
    assert_eq!(size_of::<GNode<u64, u64>>(), 48);
}

#[test]
fn state_terminal() {
    let g: GNode<u64, u64> = GNode::default();
    assert_eq!(g.state(), GState::Terminal);
    assert!(g.is_terminal());
}

#[test]
fn state_semi_internal_left() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert_eq!(g.state(), GState::SemiInternal);
    assert!(!g.is_terminal());
}

#[test]
fn state_semi_internal_right() {
    let g: GNode<u64, u64> = GNode {
        right: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert_eq!(g.state(), GState::SemiInternal);
    assert!(!g.is_terminal());
}

#[test]
fn state_internal() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert_eq!(g.state(), GState::Internal);
    assert!(!g.is_terminal());
}

// ── has_dependents ──────────────────────────────────────────

#[test]
fn has_dependents_matches_state() {
    // Terminal: no dependents.
    let terminal: GNode<u64, u64> = GNode::default();
    assert!(!terminal.has_dependents());

    // Semi-internal (left): has dependents.
    let semi_left: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(semi_left.has_dependents());

    // Internal: has dependents.
    let internal: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert!(internal.has_dependents());
}

// ── is_semi_internal ────────────────────────────────────────

#[test]
fn is_semi_internal_matches_state() {
    let terminal: GNode<u64, u64> = GNode::default();
    assert!(!terminal.is_semi_internal());

    let semi: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(semi.is_semi_internal());

    let internal: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert!(!internal.is_semi_internal());
}

// ── uncovered_range ─────────────────────────────────────────

#[test]
fn uncovered_range_terminal() {
    let g: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 8,
        ..GNode::default()
    };
    assert_eq!(g.uncovered_range(), Some((0, 8)));
}

#[test]
fn uncovered_range_semi_left() {
    // Left present → uncovered is right half (mid, hi).
    let g: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 8,
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert_eq!(g.uncovered_range(), Some((4, 8)));
}

#[test]
fn uncovered_range_semi_right() {
    // Right present → uncovered is left half (lo, mid).
    let g: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 8,
        right: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert_eq!(g.uncovered_range(), Some((0, 4)));
}

#[test]
fn uncovered_range_internal() {
    let g: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 8,
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert_eq!(g.uncovered_range(), None);
}
