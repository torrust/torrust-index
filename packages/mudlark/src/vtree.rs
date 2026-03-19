// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! V-Tree operations: insertion, removal, sum propagation, and flag
//! propagation.
//!
//! Implements §IDEA M-9 (entry management) and §IDEA M-8.9 (V-sum
//! propagation helpers).

use std::sync::atomic::Ordering;

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::VNodeId;
use crate::traits::{Accumulator, Coordinate};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};

/// Remove a V-entry (leaf) from the V-Tree.
///
/// Implements §IDEA M-9.2. Handles:
/// - Root entry → empty tree.
/// - 3-node parent → 2-node parent.
/// - 2-node parent → structural collapse.
///
/// Returns the new V-root (None if tree is now empty).
pub fn vtree_remove_leaf<C: Coordinate, V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    v_id: VNodeId,
    v_root: Option<VNodeId>,
) -> Option<VNodeId> {
    let span = tracing::debug_span!("vtree_remove_leaf", v_id = v_id.index(), case = tracing::field::Empty,).entered();

    // Clear the G-node's entry link.
    if let VKind::Entry { gnode, .. } = vnodes.get(v_id.index()).kind {
        gnodes.get_mut(gnode.index()).entry = None;
    }

    let parent = vnodes.get(v_id.index()).parent;

    // Case: root entry (no parent).
    let Some(p_id) = parent else {
        span.record("case", "root");
        vnodes.dealloc(v_id.index());
        return None;
    };

    let p = vnodes.get(p_id.index());
    let p_child_count = match &p.kind {
        VKind::Structural { children, .. } => children.len(),
        VKind::Entry { .. } => unreachable!("parent of entry should be structural"),
    };

    if p_child_count == 3 {
        // 3-node → 2-node: just remove the child.
        span.record("case", "shrink");
        remove_child_from_structural(vnodes, p_id, v_id);
        propagate_v_sums_from(vnodes, p_id);
        propagate_evictable_flags(vnodes, p_id);
        vnodes.dealloc(v_id.index());
        return v_root;
    }

    // 2-node parent → collapse: sole surviving child replaces parent.
    span.record("case", "collapse");
    let sole_id = sole_sibling(vnodes, p_id, v_id);
    let grandparent = vnodes.get(p_id.index()).parent;

    vnodes.get_mut(sole_id.index()).parent = grandparent;

    // Invalidate depth cache for sole subtree (it moved up one level)
    invalidate_depth_subtree(vnodes, sole_id);

    let new_root = grandparent.map_or(Some(sole_id), |g_id| {
        let sole_int = vnodes.get(sole_id.index()).intensity;
        replace_child_in_parent(vnodes, g_id, p_id, sole_id, sole_int);
        propagate_v_sums_from(vnodes, g_id);
        propagate_evictable_flags(vnodes, g_id);
        v_root
    });

    vnodes.dealloc(p_id.index());
    vnodes.dealloc(v_id.index());
    new_root
}

/// Propagate V-Tree sum intensities upward from `start`.
///
/// Implements §IDEA M-8.9.1 `propagate_v_sums`.
pub fn propagate_v_sums<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, start: VNodeId) {
    tracing::trace!(start = start.index(), "propagate_v_sums");
    let mut current = vnodes.get(start.index()).parent;
    while let Some(id) = current {
        recompute_structural_intensity(vnodes, id);
        let new_int = vnodes.get(id.index()).intensity;
        update_parent_cached_intensity(vnodes, id, new_int);
        current = vnodes.get(id.index()).parent;
    }
}

/// Recompute all V-structural intensities bottom-up from `v_root`.
///
/// Assumes V-entry intensities are already correct (synced from
/// `g.own`). Restores V-I1 for every structural node by post-order
/// traversal: each structural node's `PackedChildren::intensities`
/// and `intensity` are recomputed from its children's current values.
///
/// Cost: $O(V)$ — every V-node visited once.
///
/// Used by `decay()` after bulk-updating entry intensities.
pub fn recompute_all_v_intensities<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, v_root: VNodeId) {
    recompute_v_postorder(vnodes, v_root);
}

/// Post-order traversal: recurse into children first, then recompute
/// this node's cached intensities and sum.
///
/// V-tree depth is O(log E), so recursion depth is bounded and safe.
fn recompute_v_postorder<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, id: VNodeId) {
    // Collect child IDs (if structural) before mutating.
    let child_ids: Option<Vec<VNodeId>> = {
        let node = vnodes.get(id.index());
        match &node.kind {
            VKind::Entry { .. } => None,
            VKind::Structural { children, .. } => {
                let ids: Vec<VNodeId> = (0..children.len()).map(|i| children.get(i).0).collect();
                Some(ids)
            }
        }
    };

    let Some(child_ids) = child_ids else {
        // Entry — leaf; intensity already correct.
        return;
    };

    // 1. Recurse into children (post-order).
    for &child in &child_ids {
        recompute_v_postorder(vnodes, child);
    }

    // 2. Refresh cached child intensities from children's current values.
    for (i, &child) in child_ids.iter().enumerate() {
        let child_int = vnodes.get(child.index()).intensity;
        let node = vnodes.get_mut(id.index());
        if let VKind::Structural { children, .. } = &mut node.kind {
            children.intensities[i] = child_int;
        }
    }

    // 3. Recompute own intensity = sum of children.
    let node = vnodes.get(id.index());
    if let VKind::Structural { children, .. } = &node.kind {
        let mut total = V::zero();
        for i in 0..children.len() {
            total = V::add(total, children.intensities[i]);
        }
        // Drop borrow before mutating.
        let _ = node;
        vnodes.get_mut(id.index()).intensity = total;
    }
}

/// Propagate evictable flags upward from `start`.
///
/// Implements §IDEA M-9.3. Early-terminates when a flag is unchanged.
pub fn propagate_evictable_flags<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, start: VNodeId) {
    let mut current = Some(start);
    while let Some(id) = current {
        let node = vnodes.get(id.index());
        match &node.kind {
            VKind::Entry { .. } => {
                current = node.parent;
            }
            VKind::Structural { children, has_evictable } => {
                let old = *has_evictable;
                let new_flag = compute_has_evictable(vnodes, children);
                if new_flag == old {
                    return; // No change — stop propagation.
                }
                // Must drop borrow before mutating.
                let parent = node.parent;
                set_has_evictable(vnodes, id, new_flag);
                current = parent;
            }
        }
    }
}

/// Compute the true V-Tree depth without caching (for debug assertions).
#[cfg(debug_assertions)]
fn v_depth_uncached<V: Accumulator>(vnodes: &Arena<VNode<V>>, id: VNodeId) -> u32 {
    let node = vnodes.get(id.index());
    node.parent.map_or(0, |p| v_depth_uncached(vnodes, p) + 1)
}

/// Compute the V-Tree depth (number of parent hops) of node `id`.
///
/// The V-root has depth 0. Used by the depth gate in `attempt_split`
/// and eviction re-verification.
///
/// Uses opportunistic caching with path warming (ADR-M-029):
/// - Fast path: return cached depth if valid
/// - Slow path: recurse to parent, cache all nodes on path
#[must_use]
pub fn v_depth<V: Accumulator>(vnodes: &Arena<VNode<V>>, id: VNodeId) -> u32 {
    let node = vnodes.get(id.index());
    let cached = node.cached_depth.load(Ordering::Relaxed);

    // Fast path: cached and valid
    if cached != DEPTH_STALE {
        #[cfg(debug_assertions)]
        assert_eq!(cached, v_depth_uncached(vnodes, id), "cached depth mismatch for VNode {id:?}");
        return cached;
    }

    // Slow path: recurse to parent, cache on return
    let depth = node.parent.map_or(0, |p| v_depth(vnodes, p) + 1);
    node.cached_depth.store(depth, Ordering::Relaxed);
    depth
}

/// Invalidate cached depth for a subtree rooted at `root`.
///
/// Uses early-exit: if a node is already stale, its entire subtree
/// must also be stale (ancestor invariant from ADR-M-029).
pub fn invalidate_depth_subtree<V: Accumulator>(vnodes: &Arena<VNode<V>>, root: VNodeId) {
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        let node = vnodes.get(id.index());

        // Already stale → subtree also stale (invariant)
        if node.cached_depth.load(Ordering::Relaxed) == DEPTH_STALE {
            continue;
        }

        node.cached_depth.store(DEPTH_STALE, Ordering::Relaxed);

        if let VKind::Structural { children, .. } = &node.kind {
            for i in 0..children.len() {
                stack.push(children.get(i).0);
            }
        }
    }
}

/// Update the cached intensity for `child_id` in its parent's
/// `PackedChildren`.
///
/// After mutating a V-entry's intensity directly, this function
/// synchronizes the parent's cached copy so that subsequent
/// `propagate_v_sums` (which reads `PackedChildren::intensities`)
/// produces correct results.
///
/// No-op if `child_id` has no parent (i.e. it is the V-root).
pub fn update_parent_cached_intensity<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, child_id: VNodeId, new_intensity: V) {
    let parent = vnodes.get(child_id.index()).parent;
    let Some(p_id) = parent else { return };
    let p = vnodes.get_mut(p_id.index());
    if let VKind::Structural { children, .. } = &mut p.kind
        && let Some(idx) = children.find_index(child_id)
    {
        children.update_intensity(idx, new_intensity);
    }
}

// ── Internal helpers ─────────────────────────────────────────────────

/// Replace a child in a structural parent's `PackedChildren`.
pub fn replace_child_in_parent<V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    parent: VNodeId,
    old_child: VNodeId,
    new_child: VNodeId,
    new_intensity: V,
) {
    let p = vnodes.get_mut(parent.index());
    if let VKind::Structural { children, .. } = &mut p.kind {
        children.replace_child(old_child, new_child, new_intensity);
    }
}

/// Remove a child from a 3-node structural parent (→ 2-node).
fn remove_child_from_structural<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, parent: VNodeId, child: VNodeId) {
    let p = vnodes.get_mut(parent.index());
    if let VKind::Structural { children, .. } = &mut p.kind {
        children.remove_child(child);
    }
}

/// Get the sole sibling of `child` in a 2-node `parent`.
fn sole_sibling<V: Accumulator>(vnodes: &Arena<VNode<V>>, parent: VNodeId, child: VNodeId) -> VNodeId {
    let p = vnodes.get(parent.index());
    if let VKind::Structural { children, .. } = &p.kind {
        for i in 0..children.len() {
            let (id, _) = children.get(i);
            if id != child {
                return id;
            }
        }
    }
    unreachable!("sole_sibling: child not found in parent");
}

/// Recompute a structural node's intensity from its children.
pub fn recompute_structural_intensity<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, id: VNodeId) {
    let node = vnodes.get(id.index());
    if let VKind::Structural { children, .. } = &node.kind {
        let mut total = V::zero();
        for i in 0..children.len() {
            total = V::add(total, children.intensities[i]);
        }
        // Must drop borrow before mutating.
        let _ = node;
        vnodes.get_mut(id.index()).intensity = total;
    }
}

/// Propagate V-sums starting at `start` (inclusive) then upward.
fn propagate_v_sums_from<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, start: VNodeId) {
    recompute_structural_intensity(vnodes, start);
    let new_int = vnodes.get(start.index()).intensity;
    update_parent_cached_intensity(vnodes, start, new_int);
    propagate_v_sums(vnodes, start);
}

/// Compute the `has_evictable` flag from children.
fn compute_has_evictable<V: Accumulator>(vnodes: &Arena<VNode<V>>, children: &PackedChildren<V>) -> bool {
    for i in 0..children.len() {
        let (child_id, _) = children.get(i);
        let child = vnodes.get(child_id.index());
        let child_flag = match &child.kind {
            VKind::Entry { is_evictable, .. } => *is_evictable,
            VKind::Structural { has_evictable, .. } => *has_evictable,
        };
        if child_flag {
            return true;
        }
    }
    false
}

/// Set the `has_evictable` flag on a structural node.
fn set_has_evictable<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, id: VNodeId, flag: bool) {
    let node = vnodes.get_mut(id.index());
    if let VKind::Structural { has_evictable, .. } = &mut node.kind {
        *has_evictable = flag;
    }
}

#[cfg(test)]
mod tests {}
