// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Unit tests for **G-tree spatial routing**, **sum propagation**, and
//! **depth computation** (§IDEA M-5.2, §IDEA M-8 Step 4).
//!
//! The routing tests (`route_to_receiver`) exercise the binary descent
//! that maps an observation coordinate to the correct receiver node —
//! covering terminal (leaf), internal (two children), semi-internal
//! (one child), and multi-level topologies.
//!
//! The propagation tests verify two complementary paths for maintaining
//! the summation invariant (G-I1): a test-only bottom-up delta walker
//! (`propagate_g_sums`) and the production `recompute_g_sums` /
//! `recompute_g_sums_subtree` functions that rebuild sums from `own`
//! values — including repair of arbitrarily stale sums.
//!
//! The depth tests confirm that `gnode_depth_from_interval` returns
//! the correct tree level for both `u64` and `f64` interval widths,
//! including sub-unit f64 widths that exercise the signed-cast path.
//!
//! # Test index
//!
//! ## `route_to_receiver`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`route_single_node`] | any coordinate routes to a lone root |
//! | [`route_with_children`] | left/right binary descent with two children |
//! | [`route_semi_internal`] | left child present, right half falls through to root |
//! | [`route_semi_internal_right_only`] | right child present, left half falls through to root |
//! | [`route_three_levels`] | three-level descent (root → left → ll/lr, root → right) |
//!
//! ## `propagate_g_sums` (test-only delta propagation)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`propagate_sums_upward`] | delta from left child bubbles to root, sibling untouched |
//! | [`propagate_sums_three_levels`] | delta from grandchild propagates through two ancestors |
//!
//! ## `recompute_g_sums`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`recompute_single_node`] | leaf: `sum = own` |
//! | [`recompute_from_root`] | recompute starting at root itself (no parent walk) |
//! | [`recompute_with_two_children`] | `root.sum = own + left.sum + right.sum` |
//! | [`recompute_semi_internal`] | missing right child contributes zero |
//!
//! ## `recompute_g_sums_subtree`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`subtree_recompute_single_node`] | single-node subtree |
//! | [`subtree_recompute_three_nodes`] | post-order rebuild over root + two children |
//! | [`subtree_recompute_three_levels`] | three-level pre-order traversal produces correct sums |
//! | [`subtree_recompute_dirty_sums`] | repairs arbitrarily wrong sums (simulates post-decay) |
//!
//! ## `gnode_depth_from_interval` — u64
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_root_n8`] | full-domain width → depth 0 |
//! | [`depth_half_n8`] | half-domain width → depth 1 |
//! | [`depth_quarter_n8`] | quarter-domain width → depth 2 |
//! | [`depth_leaf_n8`] | unit width → depth *N* |
//! | [`depth_n32_root`] | N=32 root sanity check |
//!
//! ## `gnode_depth_from_interval` — f64
//!
//! | Test | Focus |
//! |------|-------|
//! | [`depth_f64_root_n4`] | f64 full domain → depth 0 |
//! | [`depth_f64_half_n4`] | f64 half domain → depth 1 |
//! | [`depth_f64_quarter_n4`] | f64 quarter domain → depth 2 |
//! | [`depth_f64_sub_unit_n4`] | sub-unit widths via signed-cast path (depth > *N*) |

use crate::GNodeId;
use crate::arena::Arena;
use crate::gnode::GNode;
use crate::gtree::{gnode_depth_from_interval, recompute_g_sums, recompute_g_sums_subtree, route_to_receiver};
use crate::traits::{Accumulator, Coordinate};

// ── Helpers ─────────────────────────────────────────────────

/// Propagate a value delta upward from `start` to the G-Tree root,
/// maintaining G-I1 (summation invariant).
fn propagate_g_sums<C: Coordinate, V: Accumulator>(gnodes: &mut Arena<GNode<C, V>>, start: GNodeId, delta: V) {
    let mut current = Some(start);
    while let Some(id) = current {
        let g = gnodes.get_mut(id.index());
        g.sum = V::add(g.sum, delta);
        current = g.parent;
    }
}

fn make_terminal(arena: &mut Arena<GNode<u64, u64>>, lo: u64, hi: u64) -> GNodeId {
    let g = GNode {
        lo,
        hi,
        ..GNode::default()
    };
    GNodeId::from_index(arena.alloc(g))
}

/// Wire `left` and/or `right` as children of `parent`.
fn wire_children(arena: &mut Arena<GNode<u64, u64>>, parent: GNodeId, left: Option<GNodeId>, right: Option<GNodeId>) {
    arena.get_mut(parent.index()).left = left;
    arena.get_mut(parent.index()).right = right;
    if let Some(l) = left {
        arena.get_mut(l.index()).parent = Some(parent);
    }
    if let Some(r) = right {
        arena.get_mut(r.index()).parent = Some(parent);
    }
}

// ── route_to_receiver ───────────────────────────────────────

#[test]
fn route_single_node() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    // Any coordinate routes to the root.
    assert_eq!(route_to_receiver(&arena, root, 0), root);
    assert_eq!(route_to_receiver(&arena, root, 5), root);
    assert_eq!(route_to_receiver(&arena, root, 7), root);
}

#[test]
fn route_with_children() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    assert_eq!(route_to_receiver(&arena, root, 0), left);
    assert_eq!(route_to_receiver(&arena, root, 3), left);
    assert_eq!(route_to_receiver(&arena, root, 4), right);
    assert_eq!(route_to_receiver(&arena, root, 7), right);
}

#[test]
fn route_semi_internal() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    wire_children(&mut arena, root, Some(left), None);

    // Left half routes to child.
    assert_eq!(route_to_receiver(&arena, root, 2), left);
    // Right half falls through to root.
    assert_eq!(route_to_receiver(&arena, root, 6), root);
}

#[test]
fn route_semi_internal_right_only() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, None, Some(right));

    // Right half routes to child.
    assert_eq!(route_to_receiver(&arena, root, 4), right);
    assert_eq!(route_to_receiver(&arena, root, 7), right);
    // Left half falls through to root.
    assert_eq!(route_to_receiver(&arena, root, 0), root);
    assert_eq!(route_to_receiver(&arena, root, 3), root);
}

#[test]
fn route_three_levels() {
    // root [0,8) → left [0,4) → ll [0,2), lr [2,4)
    //             → right [4,8) (terminal)
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    let ll = make_terminal(&mut arena, 0, 2);
    let lr = make_terminal(&mut arena, 2, 4);

    wire_children(&mut arena, root, Some(left), Some(right));
    wire_children(&mut arena, left, Some(ll), Some(lr));

    assert_eq!(route_to_receiver(&arena, root, 0), ll);
    assert_eq!(route_to_receiver(&arena, root, 1), ll);
    assert_eq!(route_to_receiver(&arena, root, 2), lr);
    assert_eq!(route_to_receiver(&arena, root, 3), lr);
    assert_eq!(route_to_receiver(&arena, root, 4), right);
    assert_eq!(route_to_receiver(&arena, root, 7), right);
}

// ── propagate_g_sums (test-only delta propagation) ──────────

#[test]
fn propagate_sums_upward() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    // Propagate from left.
    propagate_g_sums(&mut arena, left, 10);
    assert_eq!(arena.get(left.index()).sum, 10);
    assert_eq!(arena.get(root.index()).sum, 10);
    assert_eq!(arena.get(right.index()).sum, 0);

    // Propagate from right.
    propagate_g_sums(&mut arena, right, 5);
    assert_eq!(arena.get(right.index()).sum, 5);
    assert_eq!(arena.get(root.index()).sum, 15);
}

#[test]
fn propagate_sums_three_levels() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    let ll = make_terminal(&mut arena, 0, 2);
    let lr = make_terminal(&mut arena, 2, 4);
    wire_children(&mut arena, root, Some(left), Some(right));
    wire_children(&mut arena, left, Some(ll), Some(lr));

    propagate_g_sums(&mut arena, ll, 7);
    assert_eq!(arena.get(ll.index()).sum, 7);
    assert_eq!(arena.get(left.index()).sum, 7);
    assert_eq!(arena.get(root.index()).sum, 7);
    assert_eq!(arena.get(lr.index()).sum, 0);
    assert_eq!(arena.get(right.index()).sum, 0);
}

// ── recompute_g_sums ────────────────────────────────────────

#[test]
fn recompute_single_node() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    arena.get_mut(root.index()).own = 42;
    recompute_g_sums(&mut arena, root);
    assert_eq!(arena.get(root.index()).sum, 42);
}

#[test]
fn recompute_from_root() {
    // Recompute starting at the root itself (no parent walk).
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    arena.get_mut(left.index()).own = 10;
    arena.get_mut(left.index()).sum = 10;
    arena.get_mut(right.index()).own = 5;
    arena.get_mut(right.index()).sum = 5;
    arena.get_mut(root.index()).own = 1;

    recompute_g_sums(&mut arena, root);
    // root.sum = own(1) + left.sum(10) + right.sum(5) = 16.
    assert_eq!(arena.get(root.index()).sum, 16);
}

#[test]
fn recompute_with_two_children() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    // Set child sums and parent own directly.
    arena.get_mut(left.index()).own = 10;
    arena.get_mut(left.index()).sum = 10;
    arena.get_mut(right.index()).own = 5;
    arena.get_mut(right.index()).sum = 5;
    arena.get_mut(root.index()).own = 3;

    // Recompute from left — root.sum = root.own + left.sum + right.sum = 3+10+5.
    recompute_g_sums(&mut arena, left);
    assert_eq!(arena.get(left.index()).sum, 10); // leaf: sum = own
    assert_eq!(arena.get(root.index()).sum, 18);
}

#[test]
fn recompute_semi_internal() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    wire_children(&mut arena, root, Some(left), None);

    arena.get_mut(left.index()).own = 7;
    arena.get_mut(left.index()).sum = 7;
    arena.get_mut(root.index()).own = 2;

    recompute_g_sums(&mut arena, left);
    // root.sum = own(2) + left.sum(7) + right.sum(0) = 9.
    assert_eq!(arena.get(root.index()).sum, 9);
}

// ── recompute_g_sums_subtree ────────────────────────────────

#[test]
fn subtree_recompute_single_node() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    arena.get_mut(root.index()).own = 99;
    arena.get_mut(root.index()).sum = 0; // stale

    recompute_g_sums_subtree(&mut arena, &[root]);
    assert_eq!(arena.get(root.index()).sum, 99);
}

#[test]
fn subtree_recompute_three_nodes() {
    // Pre-order: [root, left, right]. Reverse iteration = post-order.
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    arena.get_mut(left.index()).own = 10;
    arena.get_mut(right.index()).own = 5;
    arena.get_mut(root.index()).own = 3;
    // All sums deliberately wrong.
    arena.get_mut(left.index()).sum = 0;
    arena.get_mut(right.index()).sum = 0;
    arena.get_mut(root.index()).sum = 0;

    recompute_g_sums_subtree(&mut arena, &[root, left, right]);
    assert_eq!(arena.get(left.index()).sum, 10);
    assert_eq!(arena.get(right.index()).sum, 5);
    assert_eq!(arena.get(root.index()).sum, 18); // 3 + 10 + 5
}

#[test]
fn subtree_recompute_three_levels() {
    // root [0,8) → left [0,4) → ll [0,2), lr [2,4)
    //             → right [4,8)
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    let ll = make_terminal(&mut arena, 0, 2);
    let lr = make_terminal(&mut arena, 2, 4);
    wire_children(&mut arena, root, Some(left), Some(right));
    wire_children(&mut arena, left, Some(ll), Some(lr));

    arena.get_mut(ll.index()).own = 1;
    arena.get_mut(lr.index()).own = 2;
    arena.get_mut(left.index()).own = 3;
    arena.get_mut(right.index()).own = 4;
    arena.get_mut(root.index()).own = 5;

    // Pre-order traversal.
    let preorder = [root, left, ll, lr, right];
    recompute_g_sums_subtree(&mut arena, &preorder);

    assert_eq!(arena.get(ll.index()).sum, 1);
    assert_eq!(arena.get(lr.index()).sum, 2);
    assert_eq!(arena.get(left.index()).sum, 6); // 3 + 1 + 2
    assert_eq!(arena.get(right.index()).sum, 4);
    assert_eq!(arena.get(root.index()).sum, 15); // 5 + 6 + 4
}

#[test]
fn subtree_recompute_dirty_sums() {
    // Verify it fixes arbitrarily wrong sums (simulating post-decay).
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);
    wire_children(&mut arena, root, Some(left), Some(right));

    arena.get_mut(left.index()).own = 10;
    arena.get_mut(left.index()).sum = 999; // wildly wrong
    arena.get_mut(right.index()).own = 5;
    arena.get_mut(right.index()).sum = 888; // wildly wrong
    arena.get_mut(root.index()).own = 1;
    arena.get_mut(root.index()).sum = 777; // wildly wrong

    recompute_g_sums_subtree(&mut arena, &[root, left, right]);
    assert_eq!(arena.get(left.index()).sum, 10);
    assert_eq!(arena.get(right.index()).sum, 5);
    assert_eq!(arena.get(root.index()).sum, 16); // 1 + 10 + 5
}

// ── gnode_depth_from_interval — u64 ─────────────────────────

#[test]
fn depth_root_n8() {
    // N=8, root covers [0, 256) → width=256=2^8 → depth=0.
    assert_eq!(gnode_depth_from_interval(0u64, 256u64, 8), 0);
}

#[test]
fn depth_half_n8() {
    // [0, 128) → width=128=2^7 → depth=1.
    assert_eq!(gnode_depth_from_interval(0u64, 128u64, 8), 1);
    // [128, 256) → same width → depth=1.
    assert_eq!(gnode_depth_from_interval(128u64, 256u64, 8), 1);
}

#[test]
fn depth_quarter_n8() {
    assert_eq!(gnode_depth_from_interval(0u64, 64u64, 8), 2);
    assert_eq!(gnode_depth_from_interval(64u64, 128u64, 8), 2);
    assert_eq!(gnode_depth_from_interval(192u64, 256u64, 8), 2);
}

#[test]
fn depth_leaf_n8() {
    // Width=1=2^0 → depth=8.
    assert_eq!(gnode_depth_from_interval(42u64, 43u64, 8), 8);
}

#[test]
fn depth_n32_root() {
    assert_eq!(gnode_depth_from_interval(0u64, 1u64 << 32, 32), 0);
}

// ── gnode_depth_from_interval — f64 ─────────────────────────

#[test]
fn depth_f64_root_n4() {
    // Float: N=4, domain [0, 16) → width=16=2^4 → depth=0.
    assert_eq!(gnode_depth_from_interval(0.0_f64, 16.0_f64, 4), 0);
}

#[test]
fn depth_f64_half_n4() {
    assert_eq!(gnode_depth_from_interval(0.0_f64, 8.0_f64, 4), 1);
    assert_eq!(gnode_depth_from_interval(8.0_f64, 16.0_f64, 4), 1);
}

#[test]
fn depth_f64_quarter_n4() {
    assert_eq!(gnode_depth_from_interval(0.0_f64, 4.0_f64, 4), 2);
}

#[test]
fn depth_f64_sub_unit_n4() {
    // width=0.5 → log2=-1 → depth = 4 - (-1) = 5.
    assert_eq!(gnode_depth_from_interval(2.0_f64, 2.5_f64, 4), 5);
    // width=0.25 → log2=-2 → depth = 4 - (-2) = 6.
    assert_eq!(gnode_depth_from_interval(2.0_f64, 2.25_f64, 4), 6);
}
