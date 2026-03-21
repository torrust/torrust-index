// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for [`GNode`](crate::gnode::GNode) and
//! [`GState`](crate::gnode::GState).
//!
//! `GNode` is the fundamental tree node: it carries an interval
//! `[lo, hi)`, aggregate energy (`sum`), own energy (`own`), and up
//! to four handles (left, right, parent, entry).  `GState` is a
//! derived three-variant enum — `Terminal`, `SemiInternal`, or
//! `Internal` — computed from which child handles are present.
//!
//! These tests exercise the node struct at the lowest level: memory
//! layout, default construction, state derivation, predicate helpers,
//! uncovered-range logic, and trait impls for `GState`.
//!
//! # Test index
//!
//! ## Layout
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnode_size`] | `GNode<u64,u64>` is exactly 48 bytes (no stored state field) |
//!
//! ## Default impl
//!
//! | Test | Focus |
//! |------|-------|
//! | [`default_fields_are_zeroed`] | all numeric fields zero, all handles `None` |
//!
//! ## `state()` derivation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`state_terminal`] | no children → `Terminal` |
//! | [`state_semi_internal_left`] | left-only → `SemiInternal` |
//! | [`state_semi_internal_right`] | right-only → `SemiInternal` |
//! | [`state_internal`] | both children → `Internal` |
//!
//! ## `has_dependents()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`has_dependents_terminal`] | no children → `false` |
//! | [`has_dependents_semi_left`] | left-only → `true` |
//! | [`has_dependents_semi_right`] | right-only → `true` |
//! | [`has_dependents_internal`] | both children → `true` |
//!
//! ## `is_semi_internal()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`is_semi_internal_terminal`] | no children → `false` |
//! | [`is_semi_internal_left`] | left-only → `true` |
//! | [`is_semi_internal_right`] | right-only → `true` |
//! | [`is_semi_internal_internal`] | both children → `false` |
//!
//! ## `uncovered_range()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`uncovered_range_terminal`] | terminal returns full `[lo, hi)` |
//! | [`uncovered_range_semi_left`] | left present → right half `[mid, hi)` |
//! | [`uncovered_range_semi_right`] | right present → left half `[lo, mid)` |
//! | [`uncovered_range_internal`] | both children → `None` |
//! | [`uncovered_range_minimal_interval`] | `hi - lo = 1` edge case |
//!
//! ## `GState` traits
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gstate_clone_and_copy`] | `Copy` + `Clone` + `PartialEq` derive |

use std::mem::size_of;

use crate::GNodeId;
use crate::gnode::{GNode, GState};

// ── Layout ──────────────────────────────────────────────────────

#[test]
fn gnode_size() {
    // 2×u64 (lo, hi) = 16
    // 2×u64 (sum, own) = 16
    // 4×Option<handle> = 16
    // Total = 48 — no stored state field (ADR-M-016).
    assert_eq!(size_of::<GNode<u64, u64>>(), 48);
}

// ── Default impl ────────────────────────────────────────────────

#[test]
fn default_fields_are_zeroed() {
    let g: GNode<u64, u64> = GNode::default();
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, 0);
    assert_eq!(g.sum, 0);
    assert_eq!(g.own, 0);
    assert!(g.left.is_none());
    assert!(g.right.is_none());
    assert!(g.parent.is_none());
    assert!(g.entry.is_none());
}

// ── state() derivation ──────────────────────────────────────────

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

// ── has_dependents ──────────────────────────────────────────────

#[test]
fn has_dependents_terminal() {
    let g: GNode<u64, u64> = GNode::default();
    assert!(!g.has_dependents());
}

#[test]
fn has_dependents_semi_left() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(g.has_dependents());
}

#[test]
fn has_dependents_semi_right() {
    let g: GNode<u64, u64> = GNode {
        right: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(g.has_dependents());
}

#[test]
fn has_dependents_internal() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert!(g.has_dependents());
}

// ── is_semi_internal ────────────────────────────────────────────

#[test]
fn is_semi_internal_terminal() {
    let g: GNode<u64, u64> = GNode::default();
    assert!(!g.is_semi_internal());
}

#[test]
fn is_semi_internal_left() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(g.is_semi_internal());
}

#[test]
fn is_semi_internal_right() {
    let g: GNode<u64, u64> = GNode {
        right: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert!(g.is_semi_internal());
}

#[test]
fn is_semi_internal_internal() {
    let g: GNode<u64, u64> = GNode {
        left: Some(GNodeId::from_index(0)),
        right: Some(GNodeId::from_index(1)),
        ..GNode::default()
    };
    assert!(!g.is_semi_internal());
}

// ── uncovered_range ─────────────────────────────────────────────

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

/// Smallest dyadic interval: `hi - lo = 1`. Midpoint equals `lo`,
/// so the "left half" is empty `[0,0)` and "right half" is `[0,1)`.
#[test]
fn uncovered_range_minimal_interval() {
    let g: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 1,
        ..GNode::default()
    };
    // Terminal — full range returned.
    assert_eq!(g.uncovered_range(), Some((0, 1)));

    // Semi-internal (left present) — right half is [mid=0, hi=1).
    let semi: GNode<u64, u64> = GNode {
        lo: 0,
        hi: 1,
        left: Some(GNodeId::from_index(0)),
        ..GNode::default()
    };
    assert_eq!(semi.uncovered_range(), Some((0, 1)));
}

// ── GState traits ───────────────────────────────────────────────

#[test]
fn gstate_clone_and_copy() {
    let s = GState::SemiInternal;
    let cloned = s; // Copy (same as clone for Copy types)
    let copied = s; // Copy
    assert_eq!(s, cloned);
    assert_eq!(s, copied);
}
