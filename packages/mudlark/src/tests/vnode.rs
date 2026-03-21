// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for [`VNode`](crate::vnode::VNode),
//! [`VKind`](crate::vnode::VKind), and
//! [`PackedChildren`](crate::vnode::PackedChildren).
//!
//! `VNode` is the virtual-tree node that mirrors each `GNode` in the
//! grid tree.  It carries an intensity, an optional parent link, a
//! lazily-cached depth, and a `VKind` discriminant that is either an
//! *entry* (leaf, pointing at a `GNodeId`) or a *structural* node
//! (holding a `PackedChildren` array of 2 or 3 children).
//!
//! `PackedChildren` is a fixed-capacity inline array (max 3 slots)
//! that avoids heap allocation for the common 2-node / 3-node fan-out.
//! Correctness of its add / remove / replace / query operations is
//! critical because every structural mutation in the virtual tree
//! passes through it.
//!
//! The layout test pins `VNode<u64>` at exactly 64 bytes — one cache
//! line — so that any field additions or padding changes are caught
//! immediately.
//!
//! # Test index
//!
//! ## Layout
//!
//! | Test | Focus |
//! |------|-------|
//! | [`vnode_size`] | `VNode<u64>` fits in one 64-byte cache line |
//!
//! ## `VNode` default & clone
//!
//! | Test | Focus |
//! |------|-------|
//! | [`vnode_default_fields`] | `Default` initialises entry kind, zero intensity, stale depth |
//! | [`vnode_clone_preserves_all_fields`] | manual `Clone` impl round-trips `AtomicU32` depth correctly |
//!
//! ## `PackedChildren` construction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`packed_children_new_2`] | 2-node: ids, intensities, lightest / heaviest |
//! | [`packed_children_new_3`] | 3-node: ids, intensities, lightest / heaviest |
//! | [`default_packed_children_is_empty`] | `Default` yields an empty container |
//!
//! ## `PackedChildren` mutation
//!
//! | Test | Focus |
//! |------|-------|
//! | [`packed_children_add_remove`] | add → 3-node, remove middle child (shift path) |
//! | [`remove_child_last_index_no_shift`] | remove last child — no shift needed |
//! | [`packed_children_replace`] | replace an existing child id + intensity |
//! | [`update_intensity_reflects_in_get`] | intensity update visible through `get` |
//!
//! ## `PackedChildren` queries
//!
//! | Test | Focus |
//! |------|-------|
//! | [`lightest_tiebreak_leftmost`] | equal intensities — leftmost index wins |
//! | [`find_index_returns_some_for_present_id`] | present id → `Some(index)` |
//! | [`find_index_returns_none_for_absent_id`] | absent id → `None` |
//! | [`iter_2node`] | iteration over 2-node yields both children |
//! | [`iter_3node`] | iteration over 3-node yields all three children |
//!
//! ## Panics (invalid usage)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`add_child_panics_on_3node`] | adding a 4th child panics |
//! | [`remove_child_panics_on_2node`] | removing from a 2-node panics |
//! | [`replace_child_panics_on_absent_id`] | replacing an absent id panics |
//! | [`get_panics_out_of_bounds`] | out-of-bounds index panics |

use std::mem::size_of;
use std::sync::atomic::Ordering;

use crate::handle::{GNodeId, VNodeId};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};

// ── Test-only PackedChildren methods ────────────────────────────────

impl<V: Copy + Default + PartialOrd> PackedChildren<V> {
    /// Index of the child with the lowest intensity.
    /// Ties broken by leftmost (index 0) wins.
    #[must_use]
    pub fn lightest_child_index(&self) -> usize {
        let mut min_idx = 0;
        for i in 1..self.len() {
            // Strict less-than: leftmost wins ties.
            if self.intensities[i] < self.intensities[min_idx] {
                min_idx = i;
            }
        }
        min_idx
    }
}

// ── Layout ──────────────────────────────────────────────────────

/// Compile-time structural check: `VKind` discriminant + payload.
/// Entry: `GNodeId`(4) + bool(1) = 5, padded.
/// Structural: `PackedChildren`(40) + bool(1) = 41, padded.
/// `VKind` discriminant picks the larger variant.
#[test]
fn vnode_size() {
    // VNode<u64>: intensity(8) + Option<VNodeId>(4) + VKind<u64>(~48 padded)
    // Exact size: 64 bytes — fits one cache line.
    let size = size_of::<VNode<u64>>();
    assert!(size <= 64, "VNode<u64> should fit in one cache line, got {size}");
    assert_eq!(size, 64, "VNode<u64> expected to be 64 bytes, got {size}");
}

// ── VNode default & clone ───────────────────────────────────────

#[test]
fn vnode_default_fields() {
    let node = VNode::<u64>::default();
    assert_eq!(node.intensity, 0);
    assert_eq!(node.parent, None);
    assert_eq!(node.cached_depth.load(Ordering::Relaxed), DEPTH_STALE);
    match &node.kind {
        VKind::Entry {
            gnode,
            is_exposed,
            is_evictable,
        } => {
            assert_eq!(*gnode, GNodeId::from_index(0));
            assert!(!is_exposed);
            assert!(!is_evictable);
        }
        VKind::Structural { .. } => panic!("default VNode should be Entry"),
    }
}

#[test]
fn vnode_clone_preserves_all_fields() {
    let node = VNode {
        intensity: 42u64,
        parent: Some(VNodeId::from_index(7)),
        cached_depth: std::sync::atomic::AtomicU32::new(3),
        kind: VKind::Entry {
            gnode: GNodeId::from_index(5),
            is_exposed: true,
            is_evictable: false,
        },
    };
    #[allow(clippy::redundant_clone)]
    let cloned = node.clone();
    assert_eq!(cloned.intensity, 42);
    assert_eq!(cloned.parent, Some(VNodeId::from_index(7)));
    // AtomicU32 round-trips through the manual Clone impl.
    assert_eq!(cloned.cached_depth.load(Ordering::Relaxed), 3);
    match &cloned.kind {
        VKind::Entry {
            gnode,
            is_exposed,
            is_evictable,
        } => {
            assert_eq!(*gnode, GNodeId::from_index(5));
            assert!(*is_exposed);
            assert!(!is_evictable);
        }
        VKind::Structural { .. } => panic!("cloned VNode should be Entry"),
    }
}

// ── PackedChildren construction ─────────────────────────────────

#[test]
fn packed_children_new_2() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let pc = PackedChildren::new_2((a, 10u64), (b, 20));
    assert_eq!(pc.len(), 2);
    assert_eq!(pc.get(0), (a, 10));
    assert_eq!(pc.get(1), (b, 20));
    assert_eq!(pc.lightest_child_index(), 0);
    assert_eq!(pc.heaviest_child_index(), 1);
}

#[test]
fn packed_children_new_3() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let pc = PackedChildren::new_3((a, 5u64), (b, 15), (c, 10));
    assert_eq!(pc.len(), 3);
    assert_eq!(pc.lightest_child_index(), 0);
    assert_eq!(pc.heaviest_child_index(), 1);
}

#[test]
fn default_packed_children_is_empty() {
    let pc = PackedChildren::<u64>::default();
    assert!(pc.is_empty());
    assert_eq!(pc.len(), 0);
}

// ── PackedChildren mutation ─────────────────────────────────────

#[test]
fn packed_children_add_remove() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let mut pc = PackedChildren::new_2((a, 10u64), (b, 20));

    pc.add_child(c, 5);
    assert_eq!(pc.len(), 3);
    assert_eq!(pc.get(2), (c, 5));

    // Removes middle child (idx 1) — exercises the shift path.
    let removed = pc.remove_child(b);
    assert_eq!(removed, (b, 20));
    assert_eq!(pc.len(), 2);
}

#[test]
fn remove_child_last_index_no_shift() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let mut pc = PackedChildren::new_3((a, 10u64), (b, 20), (c, 30));

    // Removes the last child (idx 2) — no shift needed.
    let removed = pc.remove_child(c);
    assert_eq!(removed, (c, 30));
    assert_eq!(pc.len(), 2);
    assert_eq!(pc.get(0), (a, 10));
    assert_eq!(pc.get(1), (b, 20));
}

#[test]
fn packed_children_replace() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let mut pc = PackedChildren::new_2((a, 10u64), (b, 20));

    pc.replace_child(a, c, 99);
    assert_eq!(pc.get(0), (c, 99));
    assert_eq!(pc.get(1), (b, 20));
}

#[test]
fn update_intensity_reflects_in_get() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let mut pc = PackedChildren::new_2((a, 10u64), (b, 20));
    pc.update_intensity(0, 99);
    assert_eq!(pc.get(0), (a, 99));
    assert_eq!(pc.get(1), (b, 20)); // unchanged
}

// ── PackedChildren queries ──────────────────────────────────────

#[test]
fn lightest_tiebreak_leftmost() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let pc = PackedChildren::new_3((a, 5u64), (b, 5), (c, 5));
    // All equal — leftmost wins.
    assert_eq!(pc.lightest_child_index(), 0);
    assert_eq!(pc.heaviest_child_index(), 0);
}

#[test]
fn find_index_returns_some_for_present_id() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let pc = PackedChildren::new_3((a, 10u64), (b, 20), (c, 30));
    assert_eq!(pc.find_index(a), Some(0));
    assert_eq!(pc.find_index(b), Some(1));
    assert_eq!(pc.find_index(c), Some(2));
}

#[test]
fn find_index_returns_none_for_absent_id() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let absent = VNodeId::from_index(99);
    let pc = PackedChildren::new_2((a, 10u64), (b, 20));
    assert_eq!(pc.find_index(absent), None);
}

#[test]
fn iter_2node() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let pc = PackedChildren::new_2((a, 10u64), (b, 20));
    let items: Vec<_> = pc.iter().collect();
    assert_eq!(items.len(), 2);
    assert_eq!(items[0], (a, 10));
    assert_eq!(items[1], (b, 20));
}

#[test]
fn iter_3node() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let pc = PackedChildren::new_3((a, 5u64), (b, 15), (c, 10));
    let items: Vec<_> = pc.iter().collect();
    assert_eq!(items.len(), 3);
    assert_eq!(items[0], (a, 5));
    assert_eq!(items[1], (b, 15));
    assert_eq!(items[2], (c, 10));
}

// ── Panics (invalid usage) ─────────────────────────────────────

#[test]
#[should_panic(expected = "already a 3-node")]
fn add_child_panics_on_3node() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let d = VNodeId::from_index(3);
    let mut pc = PackedChildren::new_3((a, 1u64), (b, 2), (c, 3));
    pc.add_child(d, 4);
}

#[test]
#[should_panic(expected = "not a 3-node")]
fn remove_child_panics_on_2node() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let mut pc = PackedChildren::new_2((a, 1u64), (b, 2));
    pc.remove_child(a);
}

#[test]
#[should_panic(expected = "old id not found")]
fn replace_child_panics_on_absent_id() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let absent = VNodeId::from_index(99);
    let mut pc = PackedChildren::new_2((a, 1u64), (b, 2));
    pc.replace_child(absent, VNodeId::from_index(3), 42);
}

#[test]
#[should_panic(expected = "out of bounds")]
fn get_panics_out_of_bounds() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let pc = PackedChildren::new_2((a, 1u64), (b, 2));
    let _ = pc.get(2);
}
