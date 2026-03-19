// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use crate::GNodeId;
use crate::arena::Arena;
use crate::gnode::GNode;
use crate::gtree::{gnode_depth_from_interval, recompute_g_sums, route_to_receiver};
use crate::traits::{Accumulator, Coordinate};

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

    // Wire up.
    arena.get_mut(root.index()).left = Some(left);
    arena.get_mut(root.index()).right = Some(right);
    arena.get_mut(left.index()).parent = Some(root);
    arena.get_mut(right.index()).parent = Some(root);

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

    // Only left child — semi-internal.
    arena.get_mut(root.index()).left = Some(left);
    arena.get_mut(left.index()).parent = Some(root);

    // Left half routes to child.
    assert_eq!(route_to_receiver(&arena, root, 2), left);
    // Right half falls through to root.
    assert_eq!(route_to_receiver(&arena, root, 6), root);
}

#[test]
fn propagate_sums_upward() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);

    arena.get_mut(root.index()).left = Some(left);
    arena.get_mut(root.index()).right = Some(right);
    arena.get_mut(left.index()).parent = Some(root);
    arena.get_mut(right.index()).parent = Some(root);

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
fn recompute_single_node() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    arena.get_mut(root.index()).own = 42;
    recompute_g_sums(&mut arena, root);
    assert_eq!(arena.get(root.index()).sum, 42);
}

#[test]
fn recompute_with_two_children() {
    let mut arena = Arena::new();
    let root = make_terminal(&mut arena, 0, 8);
    let left = make_terminal(&mut arena, 0, 4);
    let right = make_terminal(&mut arena, 4, 8);

    arena.get_mut(root.index()).left = Some(left);
    arena.get_mut(root.index()).right = Some(right);
    arena.get_mut(left.index()).parent = Some(root);
    arena.get_mut(right.index()).parent = Some(root);

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

    // Only left child — semi-internal.
    arena.get_mut(root.index()).left = Some(left);
    arena.get_mut(left.index()).parent = Some(root);

    arena.get_mut(left.index()).own = 7;
    arena.get_mut(left.index()).sum = 7;
    arena.get_mut(root.index()).own = 2;

    recompute_g_sums(&mut arena, left);
    // root.sum = own(2) + left.sum(7) + right.sum(0) = 9.
    assert_eq!(arena.get(root.index()).sum, 9);
}

// ── gnode_depth_from_interval (Phase 4) ─────────────────────

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
