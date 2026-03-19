// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::sync::atomic::{AtomicU32, Ordering};

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::rebalance::{find_violated_nodes, is_violated};
use crate::traits::{Accumulator, Coordinate};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};
use crate::vtree::{propagate_evictable_flags, propagate_v_sums, replace_child_in_parent, v_depth, vtree_remove_leaf};
use crate::{GNodeId, VNodeId};

// ── Test-only V-Tree insertion (moved from vtree.rs) ────────────────

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

// ── Test helpers ────────────────────────────────────────────────────

/// Helper: create a minimal G-node and return its ID.
fn make_gnode(gnodes: &mut Arena<GNode<u64, u64>>) -> GNodeId {
    let g = GNode {
        lo: 0,
        hi: 8,
        ..GNode::default()
    };
    GNodeId::from_index(gnodes.alloc(g))
}

#[test]
fn insert_case1_empty_tree() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    assert_eq!(root, Some(e));
    assert!(matches!(vnodes.get(e.index()).kind, VKind::Entry { .. }));
    assert_eq!(vnodes.get(e.index()).parent, None);
    assert_eq!(gnodes.get(g.index()).entry, Some(e));
}

#[test]
fn insert_case2_single_entry_root() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);

    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (_e2, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);

    let root_id = root.unwrap();
    assert_ne!(root_id, e1); // Root should be structural now.
    let root_node = vnodes.get(root_id.index());
    assert!(matches!(root_node.kind, VKind::Structural { .. }));

    if let VKind::Structural { children, .. } = &root_node.kind {
        assert_eq!(children.len(), 2);
    }
}

#[test]
fn insert_case3_buddy_insert() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);
    let g3 = make_gnode(&mut gnodes);

    let (_, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (_, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);
    let (_, root) = vtree_insert(&mut vnodes, &mut gnodes, g3, root);

    let root_id = root.unwrap();
    // Root should still be structural, now with a nested buddy.
    assert!(matches!(vnodes.get(root_id.index()).kind, VKind::Structural { .. }));
    // e1, e2, e3, root_structural, buddy_structural = 5
    assert_eq!(vnodes.count(), 5);
}

#[test]
fn remove_root_entry() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);

    let (e, root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e, root);
    assert_eq!(new_root, None);
    assert_eq!(vnodes.count(), 0);
    assert_eq!(gnodes.get(g.index()).entry, None);
}

#[test]
fn remove_from_3node_parent() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    // Build a 3-node parent manually: root_s with 3 entry children.
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);
    let g3 = make_gnode(&mut gnodes);

    let e1_node = VNode {
        intensity: 10u64,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: g1,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e1 = VNodeId::from_index(vnodes.alloc(e1_node));
    gnodes.get_mut(g1.index()).entry = Some(e1);

    let e2_node = VNode {
        intensity: 5u64,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: g2,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e2 = VNodeId::from_index(vnodes.alloc(e2_node));
    gnodes.get_mut(g2.index()).entry = Some(e2);

    let e3_node = VNode {
        intensity: 3u64,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: g3,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e3 = VNodeId::from_index(vnodes.alloc(e3_node));
    gnodes.get_mut(g3.index()).entry = Some(e3);

    let root_s = VNode {
        intensity: 18,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Structural {
            children: PackedChildren::new_3((e1, 10), (e2, 5), (e3, 3)),
            has_evictable: true,
        },
    };
    let root_id = VNodeId::from_index(vnodes.alloc(root_s));
    vnodes.get_mut(e1.index()).parent = Some(root_id);
    vnodes.get_mut(e2.index()).parent = Some(root_id);
    vnodes.get_mut(e3.index()).parent = Some(root_id);

    // Remove e3 (3-node → 2-node).
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e3, Some(root_id));
    assert_eq!(new_root, Some(root_id));

    let root_node = vnodes.get(root_id.index());
    if let VKind::Structural { children, .. } = &root_node.kind {
        assert_eq!(children.len(), 2);
    }
    assert_eq!(root_node.intensity, 15); // 10 + 5
}

#[test]
fn remove_from_2node_collapse() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);

    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (e2, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);

    // Root is structural with 2 children. Remove e1 → collapse.
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, e1, root);
    assert_eq!(new_root, Some(e2));
    assert_eq!(vnodes.get(e2.index()).parent, None);
    assert_eq!(vnodes.count(), 1); // Only e2 remains.
}

#[test]
fn propagate_evictable_flags_basic() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);

    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (_e2, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);

    let root_id = root.unwrap();
    // Both entries are evictable → root structural has_evictable = true.
    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(*has_evictable);
    }

    // Set e1 to non-evictable and propagate.
    if let VKind::Entry { is_evictable, .. } = &mut vnodes.get_mut(e1.index()).kind {
        *is_evictable = false;
    }
    propagate_evictable_flags(&mut vnodes, root_id);

    // Root should still be true because e2 is still evictable.
    if let VKind::Structural { has_evictable, .. } = &vnodes.get(root_id.index()).kind {
        assert!(*has_evictable);
    }
}

#[test]
fn v_sum_propagation() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);

    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (_e2, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);

    // Simulate an observation: increase e1's intensity.
    vnodes.get_mut(e1.index()).intensity = 10;
    // Update the cached intensity in the parent.
    let root_id = root.unwrap();
    if let VKind::Structural { children, .. } = &mut vnodes.get_mut(root_id.index()).kind
        && let Some(idx) = children.find_index(e1)
    {
        children.update_intensity(idx, 10);
    }
    // Propagate sums.
    propagate_v_sums(&mut vnodes, e1);
    assert_eq!(vnodes.get(root_id.index()).intensity, 10);
}

// ── v_depth unit tests (Phase 2 Item 6) ──────────────────────────

#[test]
fn v_depth_root_entry_is_zero() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g = make_gnode(&mut gnodes);
    let (e, _root) = vtree_insert(&mut vnodes, &mut gnodes, g, None);
    assert_eq!(v_depth(&vnodes, e), 0);
}

#[test]
fn v_depth_one_under_structural_root() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);
    let (e1, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (e2, _root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);
    // Both entries are children of the structural root → depth 1.
    assert_eq!(v_depth(&vnodes, e1), 1);
    assert_eq!(v_depth(&vnodes, e2), 1);
}

#[test]
fn v_depth_two_in_three_level_tree() {
    let mut vnodes = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let g1 = make_gnode(&mut gnodes);
    let g2 = make_gnode(&mut gnodes);
    let g3 = make_gnode(&mut gnodes);
    let (_, root) = vtree_insert(&mut vnodes, &mut gnodes, g1, None);
    let (_, root) = vtree_insert(&mut vnodes, &mut gnodes, g2, root);
    let (e3, _root) = vtree_insert(&mut vnodes, &mut gnodes, g3, root);
    // e3 buddy-inserts next to the lightest entry.
    // Both lightest entries are at depth 1. After buddy-insert, e3
    // is wrapped in a structural at depth 1, so e3 is at depth 2.
    assert_eq!(v_depth(&vnodes, e3), 2);
}

#[test]
fn v_depth_after_bootstrap_split() {
    // Uses a full GvGraph to verify depths match the worked example.
    let mut g: crate::graph::GvGraph<u64, u64, 3> = crate::graph::GvGraph::new(crate::graph::Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    });
    g.observe(3u64, 10u64); // bootstrap split

    // Root entry at depth 1 (under structural root).
    let root_entry = g.gnodes().get(g.g_root().index()).entry.unwrap();
    assert_eq!(v_depth(g.vnodes(), root_entry), 1);

    // Child entries at depth 2 (under child structural).
    let left_id = g.gnodes().get(g.g_root().index()).left.unwrap();
    let left_entry = g.gnodes().get(left_id.index()).entry.unwrap();
    assert_eq!(v_depth(g.vnodes(), left_entry), 2);
}

// ── Leaf removal violation gap tests ─────────────────────────

/// Helper: create a V-entry with a given intensity and a backing G-node.
fn make_entry(vnodes: &mut Arena<VNode<u64>>, gnodes: &mut Arena<GNode<u64, u64>>, intensity: u64) -> VNodeId {
    let g = make_gnode(gnodes);
    let e = VNode {
        intensity,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: g,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e_id = VNodeId::from_index(vnodes.alloc(e));
    gnodes.get_mut(g.index()).entry = Some(e_id);
    e_id
}

/// Helper: create a structural 2-node and parent both children.
fn make_s2(vnodes: &mut Arena<VNode<u64>>, a: VNodeId, b: VNodeId) -> VNodeId {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let s = VNode {
        intensity: a_int + b_int,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Structural {
            children: PackedChildren::new_2((a, a_int), (b, b_int)),
            has_evictable: true,
        },
    };
    let s_id = VNodeId::from_index(vnodes.alloc(s));
    vnodes.get_mut(a.index()).parent = Some(s_id);
    vnodes.get_mut(b.index()).parent = Some(s_id);
    s_id
}

/// Helper: create a structural 3-node and parent all children.
fn make_s3(vnodes: &mut Arena<VNode<u64>>, a: VNodeId, b: VNodeId, c: VNodeId) -> VNodeId {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let c_int = vnodes.get(c.index()).intensity;
    let s = VNode {
        intensity: a_int + b_int + c_int,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Structural {
            children: PackedChildren::new_3((a, a_int), (b, b_int), (c, c_int)),
            has_evictable: true,
        },
    };
    let s_id = VNodeId::from_index(vnodes.alloc(s));
    vnodes.get_mut(a.index()).parent = Some(s_id);
    vnodes.get_mut(b.index()).parent = Some(s_id);
    vnodes.get_mut(c.index()).parent = Some(s_id);
    s_id
}

/// Removing a child from a 3-node grandparent can weaken uncle
/// coverage for grandchildren, creating a V-I3 violation that
/// `vtree_remove_leaf` does not enqueue.
///
/// Tree before removal:
/// ```text
///     g (S3: [p1(10), p2(20), victim(15)])
///      └── p2 (S2: [c1(12), c2(8)])
/// ```
///
/// c1's uncles = max(p1=10, victim=15) = 15. c1=12 ≤ 15 → OK.
///
/// After removing `victim` (3-node → 2-node):
/// g becomes S2:[p1(10), p2(20)].
/// c1's uncles = max(p1=10) = 10. c1=12 > 10 → **VIOLATED**.
#[test]
fn remove_from_3node_creates_violation_for_grandchild() {
    let mut vnodes: Arena<VNode<u64>> = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    // Leaf entries.
    let p1 = make_entry(&mut vnodes, &mut gnodes, 10);
    let c1 = make_entry(&mut vnodes, &mut gnodes, 12);
    let c2 = make_entry(&mut vnodes, &mut gnodes, 8);
    let victim = make_entry(&mut vnodes, &mut gnodes, 15);

    // p2 wraps c1 and c2.
    let p2 = make_s2(&mut vnodes, c1, c2);

    // g is a 3-node: [p1, p2, victim].
    let g = make_s3(&mut vnodes, p1, p2, victim);

    // Pre-condition: no violations.
    assert!(!is_violated(&vnodes, c1), "c1 should NOT be violated before removal");
    assert!(!is_violated(&vnodes, c2), "c2 should NOT be violated before removal");
    assert!(find_violated_nodes(&vnodes).is_empty(), "tree should be clean before removal");

    // Remove victim from the 3-node g.
    let _new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, victim, Some(g));

    // Post-condition: c1 is now violated (12 > max uncle 10).
    let violated = find_violated_nodes(&vnodes);
    assert!(
        !violated.is_empty(),
        "removing a child from a 3-node should create a violation \
             (c1 intensity 12 > new max uncle 10) — this is the gap"
    );
    assert!(
        is_violated(&vnodes, c1),
        "c1 (intensity=12) should be violated against uncle p1 (intensity=10)"
    );
}

/// Collapsing a 2-node parent replaces it with the surviving
/// child. This weakens the uncle seen by the **sibling subtree's**
/// children: they formerly had the full parent intensity as their
/// uncle; now they have only the survivor's (smaller) intensity.
///
/// Tree before removal:
/// ```text
///     g (S2: [p(24), g_sib(23)])        ← root
///      ├── p   (S2: [sole(8), victim(16)])
///      └── g_sib (S2: [nephew1(20), nephew2(3)])
/// ```
///
/// `nephew1`: `parent=g_sib`, `gp=g`. uncle=\\[`p(24)`\\]. 20 ≤ 24 → OK.
///
/// After removing `victim`: p collapses, `sole(8)` replaces p.
/// g becomes `S2:[sole(8), g_sib(23)]`.
/// `nephew1`: uncle=\\[`sole(8)`\\]. 20 > 8 → **VIOLATED**.
#[test]
fn collapse_2node_creates_violation_for_sibling_subtree() {
    let mut vnodes: Arena<VNode<u64>> = Arena::new();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();

    // Leaf entries.
    let sole = make_entry(&mut vnodes, &mut gnodes, 8);
    let victim = make_entry(&mut vnodes, &mut gnodes, 16);
    let nephew1 = make_entry(&mut vnodes, &mut gnodes, 20);
    let nephew2 = make_entry(&mut vnodes, &mut gnodes, 3);

    // Build tree bottom-up.
    let p = make_s2(&mut vnodes, sole, victim);
    let g_sib = make_s2(&mut vnodes, nephew1, nephew2);
    let g = make_s2(&mut vnodes, p, g_sib);

    // Pre-condition: no violations.
    assert!(
        !is_violated(&vnodes, nephew1),
        "nephew1 should NOT be violated before removal"
    );
    assert!(
        !is_violated(&vnodes, nephew2),
        "nephew2 should NOT be violated before removal"
    );
    assert!(!is_violated(&vnodes, sole), "sole should NOT be violated before removal");
    assert!(find_violated_nodes(&vnodes).is_empty(), "tree should be clean before removal");

    // Remove victim — p collapses, sole(8) replaces p(24) under g.
    let _new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, victim, Some(g));

    // Post-condition: nephew1 is now violated (20 > new uncle sole=8).
    let violated = find_violated_nodes(&vnodes);
    assert!(
        !violated.is_empty(),
        "collapsing a 2-node should create a violation for the sibling \
             subtree (nephew1 intensity 20 > new uncle sole=8) — this is the gap"
    );
    assert!(
        is_violated(&vnodes, nephew1),
        "nephew1 (intensity=20) should be violated against uncle sole (intensity=8)"
    );
}
