// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::mem::size_of;

use crate::VNodeId;
use crate::vnode::{PackedChildren, VNode};

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

/// Compile-time structural check: `VKind` discriminant + payload.
/// Entry: GNodeId(4) + bool(1) = 5, padded.
/// Structural: PackedChildren(40) + bool(1) = 41, padded.
/// `VKind` discriminant picks the larger variant.
#[test]
fn vnode_size() {
    // VNode<u64>: intensity(8) + Option<VNodeId>(4) + VKind<u64>(~48 padded)
    // Exact size: 64 bytes — fits one cache line.
    let size = size_of::<VNode<u64>>();
    assert!(size <= 64, "VNode<u64> should fit in one cache line, got {size}");
    assert_eq!(size, 64, "VNode<u64> expected to be 64 bytes, got {size}");
}

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
fn packed_children_add_remove() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let mut pc = PackedChildren::new_2((a, 10u64), (b, 20));

    pc.add_child(c, 5);
    assert_eq!(pc.len(), 3);
    assert_eq!(pc.get(2), (c, 5));

    let removed = pc.remove_child(b);
    assert_eq!(removed, (b, 20));
    assert_eq!(pc.len(), 2);
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
fn lightest_tiebreak_leftmost() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let c = VNodeId::from_index(2);
    let pc = PackedChildren::new_3((a, 5u64), (b, 5), (c, 5));
    // All equal — leftmost wins.
    assert_eq!(pc.lightest_child_index(), 0);
    assert_eq!(pc.heaviest_child_index(), 0);
}

// ── PackedChildren edge coverage (Phase 2 Item 5) ────────────────

#[test]
fn find_index_returns_none_for_absent_id() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let absent = VNodeId::from_index(99);
    let pc = PackedChildren::new_2((a, 10u64), (b, 20));
    assert_eq!(pc.find_index(absent), None);
}

#[test]
fn default_packed_children_is_empty() {
    let pc = PackedChildren::<u64>::default();
    assert!(pc.is_empty());
    assert_eq!(pc.len(), 0);
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

#[test]
fn update_intensity_reflects_in_get() {
    let a = VNodeId::from_index(0);
    let b = VNodeId::from_index(1);
    let mut pc = PackedChildren::new_2((a, 10u64), (b, 20));
    pc.update_intensity(0, 99);
    assert_eq!(pc.get(0), (a, 99));
    assert_eq!(pc.get(1), (b, 20)); // unchanged
}
