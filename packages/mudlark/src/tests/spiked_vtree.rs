// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── SpikedVTree: raw V-tree builder for rebalance stress ────────

use std::sync::atomic::AtomicU32;

use crate::arena::Arena;
use crate::handle::{GNodeId, VNodeId};
use crate::rebalance::find_violated_nodes;
use crate::traits::{Accumulator, Inspectable};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};

/// Fluent builder for a balanced V-tree with spiked entries.
///
/// Constructs a V-node arena directly (bypassing `GvGraph`) to create
/// worst-case violation patterns for rebalance stress testing.
/// The full observation pipeline is not involved — this tests
/// `rebalance()` in isolation.
///
/// Generic over the accumulator type `V`, mirroring `Plan<C, V>`.
///
/// # Example
///
/// ```ignore
/// let (mut vnodes, _root, mut violations) = SpikedVTree::balanced(13)
///     .base(1u64)
///     .spike(10_000u64, 4, 2)
///     .build();
/// rebalance(&mut vnodes, &mut violations);
/// ```
pub struct SpikedVTree<V: Accumulator + Inspectable> {
    depth: u32,
    base_intensity: V,
    spike_intensity: V,
    /// Of every `group_size` consecutive leaves, the first
    /// `spike_count` get `spike_intensity`.
    group_size: usize,
    spike_count: usize,
}

impl<V: Accumulator + Inspectable> SpikedVTree<V> {
    /// Start with a balanced binary 2-node tree of given `depth`.
    ///
    /// Leaf count = `2^depth`, total nodes = `2^(depth+1) − 1`.
    ///
    /// Uses `V::from_f64` to initialise default intensities:
    /// `base = 1.0`, `spike = 10_000.0`.
    #[must_use]
    pub fn balanced(depth: u32) -> Self {
        Self {
            depth,
            base_intensity: V::from_f64(1.0),
            spike_intensity: V::from_f64(10_000.0),
            group_size: 4,
            spike_count: 2,
        }
    }

    /// Set the base (non-spiked) leaf intensity.
    #[must_use]
    pub const fn base(mut self, intensity: V) -> Self {
        self.base_intensity = intensity;
        self
    }

    /// Configure the spike pattern: the first `count` of every
    /// `every` consecutive leaves get `intensity`.
    ///
    /// For example, `.spike(V::from_f64(10_000.0), 4, 2)` spikes the
    /// first 2 of every group of 4 leaves.
    #[must_use]
    pub const fn spike(mut self, intensity: V, every: usize, count: usize) -> Self {
        self.spike_intensity = intensity;
        self.group_size = every;
        self.spike_count = count;
        self
    }

    /// Build the arena, apply the spike pattern, recompute structural
    /// intensities, and find all violated nodes.
    ///
    /// Returns `(arena, root_id, violations)` ready for
    /// `rebalance()`.
    #[must_use]
    pub fn build(self) -> (Arena<VNode<V>>, VNodeId, Vec<VNodeId>) {
        let mut vnodes = Arena::new();
        let root = build_balanced_2node_tree(&mut vnodes, self.depth, self.base_intensity);

        // Spike leaves according to the pattern.
        let leaves = collect_leaf_ids(&vnodes, root);
        for (i, &eid) in leaves.iter().enumerate() {
            if self.group_size > 0 && (i % self.group_size) < self.spike_count {
                vnodes.get_mut(eid.index()).intensity = self.spike_intensity;
            }
        }

        // Recompute structural sums bottom-up.
        crate::vtree::recompute_all_v_intensities(&mut vnodes, root);

        // Discover violations (deepest-first).
        let violations = find_violated_nodes(&vnodes);
        (vnodes, root, violations)
    }

    /// Number of leaves in the tree: `2^depth`.
    #[must_use]
    pub const fn leaf_count(&self) -> usize {
        1 << self.depth
    }
}

// ── Raw V-tree construction helpers ─────────────────────────────

/// Create a V-entry with the given intensity (dummy G-node).
fn make_entry<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, intensity: V) -> VNodeId {
    let e = VNode {
        intensity,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: GNodeId::from_index(0),
            is_exposed: true,
            is_evictable: true,
        },
    };
    VNodeId::from_index(vnodes.alloc(e))
}

/// Create a structural 2-node with children `a` and `b`.
fn make_structural_2<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, a: VNodeId, b: VNodeId) -> VNodeId {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let s = VNode {
        intensity: a_int.add(b_int),
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

/// Recursively build a balanced binary tree of 2-nodes.
fn build_balanced_2node_tree<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, depth: u32, leaf_intensity: V) -> VNodeId {
    if depth == 0 {
        return make_entry(vnodes, leaf_intensity);
    }
    let left = build_balanced_2node_tree(vnodes, depth - 1, leaf_intensity);
    let right = build_balanced_2node_tree(vnodes, depth - 1, leaf_intensity);
    make_structural_2(vnodes, left, right)
}

/// Collect all entry (leaf) node IDs via DFS.
fn collect_leaf_ids<V: Accumulator>(vnodes: &Arena<VNode<V>>, root: VNodeId) -> Vec<VNodeId> {
    let mut leaves = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        match &vnodes.get(id.index()).kind {
            VKind::Entry { .. } => leaves.push(id),
            VKind::Structural { children, .. } => {
                for i in 0..children.len() {
                    stack.push(children.get(i).0);
                }
            }
        }
    }
    leaves
}
