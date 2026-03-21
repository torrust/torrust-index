// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for the **V-Tree** arena operations in
//! [`crate::vtree`].
//!
//! These tests exercise the `pub(crate)` arena-level functions
//! directly (Surface 3), so they build V-node arenas by hand rather
//! than going through `GraphCreator` / `Plan`.  Every function under
//! test operates on raw `Arena<VNode<V>>` / `Arena<GNode<C, V>>`
//! pairs, which makes it straightforward to assert exact structural
//! outcomes — node counts, parent links, cached depths, intensity
//! sums, and evictable flags — without higher-level graph machinery
//! getting in the way.
//!
//! A test-only `vtree_insert` helper (mirroring the three insertion
//! cases from §IDEA M-9.1) is defined in this module so that removal,
//! depth, and propagation tests can start from well-understood tree
//! shapes.
//!
//! # Test index
//!
//! ## `vtree_insert` (test-only helper, §IDEA M-9.1)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`insert_case1_empty_tree`] | first entry becomes root with depth 0 |
//! | [`insert_case2_single_entry_root`] | second entry creates structural root with two depth-1 children |
//! | [`insert_case3_buddy_insert`] | third entry buddy-inserts at lightest leaf |
//! | [`insert_assigns_gnode_back_link`] | G-node `.entry` back-pointer set on every insert |
//!
//! ## `vtree_remove_leaf` (§IDEA M-9.2)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`remove_root_entry_yields_empty_tree`] | removing the sole entry yields `None` root and zero nodes |
//! | [`remove_collapses_two_node_parent`] | removing one of two siblings collapses the structural parent |
//! | [`remove_shrinks_three_node_parent`] | removing a child of a 2-node structural collapses it into its sibling |
//! | [`remove_clears_gnode_back_link`] | G-node `.entry` back-pointer cleared on removal |
//!
//! ## `v_depth` (ADR-M-029)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`v_depth_root_is_zero`] | single-entry root has depth 0 |
//! | [`v_depth_children_are_one`] | immediate children of root have depth 1 |
//! | [`v_depth_caching`] | repeated queries return the same value from cache |
//! | [`v_depth_recomputes_after_invalidation`] | depth recomputed correctly after `invalidate_depth_subtree` |
//!
//! ## `invalidate_depth_subtree`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`invalidate_marks_subtree_stale`] | all descendants marked `DEPTH_STALE` from root |
//! | [`invalidate_already_stale_is_noop`] | re-invalidation of an already-stale node is harmless |
//!
//! ## `propagate_v_sums` (§IDEA M-8.9.1)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`propagate_sums_updates_ancestors`] | single intensity change propagates to root |
//! | [`propagate_sums_deep_tree`] | varied intensities across 4 entries sum correctly at root |
//!
//! ## `propagate_evictable_flags` (§IDEA M-9.3)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`evictable_flag_propagates_upward`] | clearing all children clears the structural flag |
//! | [`evictable_flag_early_terminates`] | flag stays `true` when at least one child is evictable |
//! | [`evictable_flag_from_entry_walks_to_parent`] | propagation from an entry walks up to its structural parent |
//!
//! ## `update_parent_cached_intensity`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`update_parent_cached_intensity_syncs_parent`] | parent's cached child intensity updated correctly |
//! | [`update_parent_cached_intensity_noop_at_root`] | call on parentless root is a safe no-op |
//!
//! ## `recompute_structural_intensity`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`recompute_structural_intensity_sums_children`] | structural node intensity recomputed from cached child values |
//!
//! ## `replace_child_in_parent`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`replace_child_updates_parent`] | old child replaced and new child + intensity appear in parent |
//!
//! ## `recompute_all_v_intensities`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`recompute_all_fixes_stale_sums`] | full recompute fixes stale root intensity from leaf values |
//!
//! ## Round-trip insert/remove
//!
//! | Test | Focus |
//! |------|-------|
//! | [`insert_then_remove_all_yields_empty`] | inserting then removing all entries yields empty arena |
//! | [`insert_remove_interleaved`] | mixed insert/remove sequence maintains consistent structure |

use std::sync::atomic::{AtomicU32, Ordering};

use super::init_tracing;
use crate::GNodeId;
use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::VNodeId;
use crate::traits::{Accumulator, Coordinate};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};
use crate::vtree::{
    invalidate_depth_subtree, propagate_evictable_flags, propagate_v_sums, recompute_all_v_intensities,
    recompute_structural_intensity, replace_child_in_parent, update_parent_cached_intensity, v_depth, vtree_remove_leaf,
};

// ── Test-only V-Tree insertion (moved from vtree.rs) ────────────

/// Whether a V-entry's backing G-node is exposed (on the contour).
const fn entry_is_exposed<V: Accumulator>(node: &VNode<V>) -> bool {
    match &node.kind {
        VKind::Entry { is_exposed, .. } => *is_exposed,
        VKind::Structural { .. } => false,
    }
}

/// Descend from `start` to the lightest entry (V-leaf) by always
/// following the lightest child at each structural node.
fn descend_to_lightest<V: Accumulator>(vnodes: &Arena<VNode<V>>, start: VNodeId) -> VNodeId {
    let mut current = start;
    loop {
        let node = vnodes.get(current.index());
        match &node.kind {
            VKind::Entry { .. } => return current,
            VKind::Structural { children, .. } => {
                let idx = children.lightest_child_index();
                current = children.get(idx).0;
            }
        }
    }
}

/// Insert a new zero-intensity V-entry for the given G-node.
///
/// Handles three cases (§IDEA M-9.1):
/// 1. Empty V-Tree → entry becomes root.
/// 2. V-root is a single entry → create structural root.
/// 3. General → buddy-insert at the lightest entry.
///
/// Returns `(entry_id, new_v_root)`.
fn vtree_insert<C: Coordinate, V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    gnode_id: GNodeId,
    v_root: Option<VNodeId>,
) -> (VNodeId, Option<VNodeId>) {
    let entry = VNode {
        intensity: V::zero(),
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: gnode_id,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e_id = VNodeId::from_index(vnodes.alloc(entry));
    gnodes.get_mut(gnode_id.index()).entry = Some(e_id);

    // Case 1: empty V-Tree.
    let Some(root_id) = v_root else {
        vnodes.get(e_id.index()).cached_depth.store(0, Ordering::Relaxed);
        return (e_id, Some(e_id));
    };

    let root_node = vnodes.get(root_id.index());

    // Case 2: V-root is a single entry.
    if matches!(root_node.kind, VKind::Entry { .. }) && root_node.parent.is_none() {
        let root_int = root_node.intensity;
        let root_terminal = entry_is_exposed(root_node);
        let new_terminal = true;

        let structural = VNode {
            intensity: root_int,
            parent: None,
            cached_depth: AtomicU32::new(0),
            kind: VKind::Structural {
                children: PackedChildren::new_2((root_id, root_int), (e_id, V::zero())),
                has_evictable: root_terminal || new_terminal,
            },
        };
        let s_id = VNodeId::from_index(vnodes.alloc(structural));
        vnodes.get_mut(root_id.index()).parent = Some(s_id);
        vnodes.get_mut(e_id.index()).parent = Some(s_id);
        vnodes.get(root_id.index()).cached_depth.store(1, Ordering::Relaxed);
        vnodes.get(e_id.index()).cached_depth.store(1, Ordering::Relaxed);
        return (e_id, Some(s_id));
    }

    // Case 3: general — descend to lightest entry, buddy-insert.
    let buddy_id = descend_to_lightest(vnodes, root_id);
    let buddy = vnodes.get(buddy_id.index());
    let buddy_int = buddy.intensity;
    let buddy_depth = buddy.cached_depth.load(Ordering::Relaxed);
    let buddy_parent = buddy.parent.expect("buddy should have a parent in Case 3");

    let s_depth = buddy_depth;
    let child_depth = if buddy_depth == DEPTH_STALE {
        DEPTH_STALE
    } else {
        buddy_depth + 1
    };

    let s = VNode {
        intensity: buddy_int,
        parent: Some(buddy_parent),
        cached_depth: AtomicU32::new(s_depth),
        kind: VKind::Structural {
            children: PackedChildren::new_2((buddy_id, buddy_int), (e_id, V::zero())),
            has_evictable: true,
        },
    };
    let s_id = VNodeId::from_index(vnodes.alloc(s));
    vnodes.get_mut(buddy_id.index()).parent = Some(s_id);
    vnodes.get_mut(e_id.index()).parent = Some(s_id);
    vnodes
        .get(buddy_id.index())
        .cached_depth
        .store(child_depth, Ordering::Relaxed);
    vnodes.get(e_id.index()).cached_depth.store(child_depth, Ordering::Relaxed);

    replace_child_in_parent(vnodes, buddy_parent, buddy_id, s_id, buddy_int);
    propagate_evictable_flags(vnodes, buddy_parent);

    (e_id, Some(root_id))
}

// ── Test helpers ────────────────────────────────────────────────

/// Helper: create a minimal G-node and return its ID.
fn make_gnode(gnodes: &mut Arena<GNode<u64, u64>>) -> GNodeId {
    let g = GNode {
        lo: 0,
        hi: 8,
        ..GNode::default()
    };
    GNodeId::from_index(gnodes.alloc(g))
}

/// Helper: insert `n` entries and return `(entry_ids, v_root)`.
fn insert_n(
    vnodes: &mut Arena<VNode<u64>>,
    gnodes: &mut Arena<GNode<u64, u64>>,
    n: usize,
) -> (Vec<(GNodeId, VNodeId)>, Option<VNodeId>) {
    let mut root = None;
    let mut ids = Vec::with_capacity(n);
    for _ in 0..n {
        let g = make_gnode(gnodes);
        let (e, new_root) = vtree_insert(vnodes, gnodes, g, root);
        root = new_root;
        ids.push((g, e));
    }
    (ids, root)
}

/// Helper: build a 3-entry tree (structural root with one 2-node child).
///
/// Layout: `S_root(S_buddy(e1, e3), e2)` — root is structural with
/// one structural child and one entry child.
#[allow(clippy::type_complexity)]
fn build_three_entry_tree() -> (
    Arena<VNode<u64>>,
    Arena<GNode<u64, u64>>,
    Vec<(GNodeId, VNodeId)>,
    Option<VNodeId>,
) {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 3);
    (vnodes, gnodes, ids, root)
}

// ── vtree_insert (§IDEA M-9.1) ─────────────────────────────────

#[test]
fn insert_case1_empty_tree() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    assert_eq!(root, Some(e));
    assert!(matches!(vnodes.get(e.index()).kind, VKind::Entry { .. }));
    assert_eq!(vnodes.get(e.index()).parent, None);
    assert_eq!(v_depth(&vnodes, e), 0);
}

#[test]
fn insert_case2_single_entry_root() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    let (_, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();
    let root_node = vnodes.get(root_id.index());
    assert!(matches!(root_node.kind, VKind::Structural { .. }));
    assert_eq!(root_node.parent, None);

    if let VKind::Structural { children, .. } = &root_node.kind {
        assert_eq!(children.len(), 2);
        // Both children should be entries at depth 1.
        for i in 0..children.len() {
            let (child_id, _) = children.get(i);
            assert!(matches!(vnodes.get(child_id.index()).kind, VKind::Entry { .. }));
            assert_eq!(v_depth(&vnodes, child_id), 1);
        }
    }
}

#[test]
fn insert_case3_buddy_insert() {
    let _t = init_tracing();
    let (vnodes, _, _, root) = build_three_entry_tree();

    let root_id = root.unwrap();
    assert!(matches!(vnodes.get(root_id.index()).kind, VKind::Structural { .. }));
    // 3 entries + 1 root structural + 1 buddy structural = 5
    assert_eq!(vnodes.count(), 5);
}

#[test]
fn insert_assigns_gnode_back_link() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    let (ids, _) = insert_n(&mut vnodes, &mut gnodes, 3);
    for (g_id, e_id) in &ids {
        assert_eq!(gnodes.get(g_id.index()).entry, Some(*e_id));
    }
}

// ── vtree_remove_leaf (§IDEA M-9.2) ────────────────────────────

#[test]
fn remove_root_entry_yields_empty_tree() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e, root);
    assert_eq!(new_root, None);
    assert_eq!(vnodes.count(), 0);
}

#[test]
fn remove_collapses_two_node_parent() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let (_, e1) = ids[0];
    let (_, e2) = ids[1];

    // Remove first entry — structural root should collapse, leaving e2 as root.
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e1, root);
    let new_root_id = new_root.unwrap();
    assert_eq!(new_root_id, e2);
    assert_eq!(vnodes.get(e2.index()).parent, None);
    assert_eq!(vnodes.count(), 1); // Only e2 remains.
}

#[test]
fn remove_shrinks_three_node_parent() {
    let _t = init_tracing();
    let (mut vnodes, mut gnodes, ids, root) = build_three_entry_tree();

    // The tree has 5 nodes: 3 entries + 2 structural.
    assert_eq!(vnodes.count(), 5);

    // Remove one entry — expect the parent to shrink or collapse.
    let (_, e3) = ids[2];
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e3, root);
    assert!(new_root.is_some());

    // Removing e3 collapses its 2-node parent (buddy structural),
    // leaving: S_root(e1, e2) = 2 entries + 1 structural = 3 nodes.
    assert_eq!(vnodes.count(), 3);
}

#[test]
fn remove_clears_gnode_back_link() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    assert_eq!(gnodes.get(g.index()).entry, Some(e));

    vtree_remove_leaf(&mut vnodes, &mut gnodes, e, root);
    assert_eq!(gnodes.get(g.index()).entry, None);
}

// ── v_depth (ADR-M-029) ────────────────────────────────────────

#[test]
fn v_depth_root_is_zero() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, _) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    assert_eq!(v_depth(&vnodes, e), 0);
}

#[test]
fn v_depth_children_are_one() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();
    assert_eq!(v_depth(&vnodes, root_id), 0);

    for (_, e_id) in &ids {
        assert_eq!(v_depth(&vnodes, *e_id), 1);
    }
}

#[test]
fn v_depth_caching() {
    let _t = init_tracing();
    let (vnodes, _, ids, _) = build_three_entry_tree();

    // First call computes; second should hit cache.
    let (_, e1) = ids[0];
    let d1 = v_depth(&vnodes, e1);
    let d2 = v_depth(&vnodes, e1);
    assert_eq!(d1, d2);
    // Depth should be 2 (root → buddy_structural → e1).
    assert_eq!(d1, 2);
}

#[test]
fn v_depth_recomputes_after_invalidation() {
    let _t = init_tracing();
    let (vnodes, _, ids, root) = build_three_entry_tree();

    let root_id = root.unwrap();
    let (_, e1) = ids[0];

    // Warm the cache.
    let original = v_depth(&vnodes, e1);
    assert_eq!(original, 2);

    // Invalidate the whole subtree.
    invalidate_depth_subtree(&vnodes, root_id);
    assert_eq!(vnodes.get(e1.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);

    // Re-query — should recompute and return the same depth.
    let recomputed = v_depth(&vnodes, e1);
    assert_eq!(recomputed, original);
    // Cache should be warm again.
    assert_ne!(vnodes.get(e1.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);
}

// ── invalidate_depth_subtree ────────────────────────────────────

#[test]
fn invalidate_marks_subtree_stale() {
    let _t = init_tracing();
    let (vnodes, _, ids, root) = build_three_entry_tree();

    let root_id = root.unwrap();
    // Warm the cache for all nodes.
    for (_, e_id) in &ids {
        let _ = v_depth(&vnodes, *e_id);
    }
    // All cached depths should be valid.
    assert_ne!(vnodes.get(root_id.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);

    // Invalidate from root — everything should become stale.
    invalidate_depth_subtree(&vnodes, root_id);

    assert_eq!(vnodes.get(root_id.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);
    for (_, e_id) in &ids {
        assert_eq!(vnodes.get(e_id.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);
    }
}

#[test]
fn invalidate_already_stale_is_noop() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    // Insert but don't warm the cache — insert sets depth for case 1.
    let (e, _) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    // Manually mark as stale.
    vnodes.get(e.index()).cached_depth.store(DEPTH_STALE, Ordering::Relaxed);

    // Invalidate should be a no-op (already stale).
    invalidate_depth_subtree(&vnodes, e);
    assert_eq!(vnodes.get(e.index()).cached_depth.load(Ordering::Relaxed), DEPTH_STALE);
}

// ── propagate_v_sums (§IDEA M-8.9.1) ───────────────────────────

#[test]
fn propagate_sums_updates_ancestors() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();
    let (_, e1) = ids[0];

    // Root intensity should be 0 (both entries have zero intensity).
    assert_eq!(vnodes.get(root_id.index()).intensity, 0);

    // Set e1's intensity and propagate.
    vnodes.get_mut(e1.index()).intensity = 42;
    // Update the parent's cached child intensity first.
    update_parent_cached_intensity(&mut vnodes, e1, 42);
    propagate_v_sums(&mut vnodes, e1);

    assert_eq!(vnodes.get(root_id.index()).intensity, 42);
}

#[test]
fn propagate_sums_deep_tree() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    // 4 entries → depth-2 tree with multiple structural levels.
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 4);
    let root_id = root.unwrap();

    // Set varied intensities on all entries.
    for (i, (_, e_id)) in ids.iter().enumerate() {
        let val = (i as u64 + 1) * 10; // 10, 20, 30, 40
        vnodes.get_mut(e_id.index()).intensity = val;
        update_parent_cached_intensity(&mut vnodes, *e_id, val);
        propagate_v_sums(&mut vnodes, *e_id);
    }

    // Root intensity should equal the sum of all entry intensities.
    assert_eq!(vnodes.get(root_id.index()).intensity, 100);
}

// ── propagate_evictable_flags (§IDEA M-9.3) ────────────────────

#[test]
fn evictable_flag_propagates_upward() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();

    // Both entries are evictable by default, so root should have has_evictable.
    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(*has_evictable);
    }

    // Mark both entries as non-evictable.
    for (_, e_id) in &ids {
        if let VKind::Entry { is_evictable, .. } = &mut vnodes.get_mut(e_id.index()).kind {
            *is_evictable = false;
        }
    }

    propagate_evictable_flags(&mut vnodes, root_id);

    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(!*has_evictable, "root should be non-evictable after clearing children");
    }
}

#[test]
fn evictable_flag_early_terminates() {
    let _t = init_tracing();
    let (mut vnodes, _, ids, root) = build_three_entry_tree();

    let root_id = root.unwrap();

    // Mark just one entry as non-evictable — flag should still be true
    // at root because other entries remain evictable.
    let (_, e1) = ids[0];
    if let VKind::Entry { is_evictable, .. } = &mut vnodes.get_mut(e1.index()).kind {
        *is_evictable = false;
    }

    propagate_evictable_flags(&mut vnodes, root_id);

    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(
            *has_evictable,
            "root should remain evictable when some children are evictable"
        );
    }
}

#[test]
fn evictable_flag_from_entry_walks_to_parent() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();

    // Mark both entries as non-evictable.
    for (_, e_id) in &ids {
        if let VKind::Entry { is_evictable, .. } = &mut vnodes.get_mut(e_id.index()).kind {
            *is_evictable = false;
        }
    }

    // Start propagation from an *entry* node — the function should
    // walk up to the structural parent and clear its flag.
    propagate_evictable_flags(&mut vnodes, ids[0].1);

    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(!*has_evictable, "flag should be false after propagating from entry");
    }
}

// ── update_parent_cached_intensity ──────────────────────────────

#[test]
fn update_parent_cached_intensity_syncs_parent() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();
    let (_, e1) = ids[0];

    // Before: parent's cached intensity for e1 should be 0.
    if let VKind::Structural { children, .. } = &vnodes.get(root_id.index()).kind {
        let idx = children.find_index(e1).unwrap();
        assert_eq!(children.intensities[idx], 0);
    }

    // Update the cached intensity.
    update_parent_cached_intensity(&mut vnodes, e1, 77);

    // After: parent's cached intensity should reflect the new value.
    if let VKind::Structural { children, .. } = &vnodes.get(root_id.index()).kind {
        let idx = children.find_index(e1).unwrap();
        assert_eq!(children.intensities[idx], 77);
    }
}

#[test]
fn update_parent_cached_intensity_noop_at_root() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, _) = vtree_insert(&mut vnodes, &mut gnodes, g, None);

    // e is the root (no parent) — should be a no-op.
    update_parent_cached_intensity(&mut vnodes, e, 999);
    // No panic, no structural change.
    assert_eq!(vnodes.count(), 1);
}

// ── recompute_structural_intensity ──────────────────────────────

#[test]
fn recompute_structural_intensity_sums_children() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();

    // Set child intensities and update the parent's cached copies.
    vnodes.get_mut(ids[0].1.index()).intensity = 15;
    vnodes.get_mut(ids[1].1.index()).intensity = 25;
    update_parent_cached_intensity(&mut vnodes, ids[0].1, 15);
    update_parent_cached_intensity(&mut vnodes, ids[1].1, 25);

    // Root intensity is still 0 (stale).
    assert_eq!(vnodes.get(root_id.index()).intensity, 0);

    recompute_structural_intensity(&mut vnodes, root_id);
    assert_eq!(vnodes.get(root_id.index()).intensity, 40);
}

// ── replace_child_in_parent ─────────────────────────────────────

#[test]
fn replace_child_updates_parent() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();
    let (_, e1) = ids[0];

    // Create a new entry to replace e1.
    let new_entry = VNode {
        intensity: 99,
        parent: Some(root_id),
        cached_depth: AtomicU32::new(1),
        kind: VKind::Entry {
            gnode: ids[0].0,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let new_id = VNodeId::from_index(vnodes.alloc(new_entry));

    replace_child_in_parent(&mut vnodes, root_id, e1, new_id, 99);

    // Verify parent now references the new child.
    if let VKind::Structural { children, .. } = &vnodes.get(root_id.index()).kind {
        let mut found = false;
        for i in 0..children.len() {
            let (id, int) = children.get(i);
            if id == new_id {
                assert_eq!(int, 99);
                found = true;
            }
            assert_ne!(id, e1, "old child should no longer appear in parent");
        }
        assert!(found, "new child should appear in parent");
    }
}

// ── recompute_all_v_intensities ─────────────────────────────────

#[test]
fn recompute_all_fixes_stale_sums() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, root) = insert_n(&mut vnodes, &mut gnodes, 2);

    let root_id = root.unwrap();

    // Set entry intensities directly (simulating stale sums).
    vnodes.get_mut(ids[0].1.index()).intensity = 10;
    vnodes.get_mut(ids[1].1.index()).intensity = 20;

    // Root intensity is stale (still 0 from insertion).
    assert_eq!(vnodes.get(root_id.index()).intensity, 0);

    recompute_all_v_intensities(&mut vnodes, root_id);

    // Root should now reflect the sum of its children.
    assert_eq!(vnodes.get(root_id.index()).intensity, 30);
}

// ── Round-trip insert/remove ────────────────────────────────────

#[test]
fn insert_then_remove_all_yields_empty() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let (ids, mut root) = insert_n(&mut vnodes, &mut gnodes, 4);

    // Remove all entries in reverse order.
    for (_, e_id) in ids.iter().rev() {
        root = vtree_remove_leaf(&mut vnodes, &mut gnodes, *e_id, root);
    }

    assert_eq!(root, None);
    assert_eq!(vnodes.count(), 0);
}

#[test]
fn insert_remove_interleaved() {
    let _t = init_tracing();
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    // Insert 3.
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);
    let g3 = make_gnode(&mut gnodes);

    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (e2, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);
    let (_e3, root) = vtree_insert(&mut vnodes, &mut gnodes, g3, root);

    // Remove middle.
    let root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e2, root);
    assert!(root.is_some());

    // Insert another.
    let g4 = make_gnode(&mut gnodes);
    let (_e4, root) = vtree_insert(&mut vnodes, &mut gnodes, g4, root);
    assert!(root.is_some());

    // Remove first.
    let root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e1, root);
    assert!(root.is_some());

    // Should have 2 entries remaining with valid structure.
    let entry_count = vnodes
        .iter_occupied()
        .filter(|(_, v)| matches!(v.kind, VKind::Entry { .. }))
        .count();
    assert_eq!(entry_count, 2);
}
