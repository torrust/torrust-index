// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! **`SpikedVTree`**: raw V-tree builder and correctness tests for
//! rebalance stress testing.
//!
//! Constructs V-node arenas directly (bypassing `GvGraph`) to create
//! worst-case violation patterns.  The `testing::` module
//! (`GraphCreator`, `Plan`, `run`) works at the full observation
//! pipeline level; this builder operates one layer below, letting
//! tests exercise `rebalance()` in isolation.
//!
//! The builder itself is consumed by [`super::rebalance_stress`].
//! The tests here validate the builder's own structural guarantees —
//! node counts, parent-link consistency, spike-pattern placement, and
//! violation discovery — so that rebalance-stress tests can trust the
//! scaffolding.
//!
//! # Test index
//!
//! ## Tree structure
//!
//! | Test | Focus |
//! |------|-------|
//! | [`balanced_depth_0_is_single_entry`] | depth-0 tree is a single entry node |
//! | [`balanced_depth_1_has_two_leaves_and_one_structural`] | depth-1: 2 leaves + 1 structural |
//! | [`balanced_depth_n_node_counts`] | `2^depth` leaves, `2^(depth+1) − 1` total for depths 0–6 |
//! | [`leaf_count_matches_actual_leaves`] | builder's `leaf_count()` agrees with built tree |
//! | [`all_leaves_have_base_intensity_before_spike`] | zero-spike build → all leaves at base intensity |
//! | [`every_structural_node_has_two_children`] | every structural node is a 2-node |
//! | [`parent_links_are_consistent`] | parent ↔ child links round-trip correctly |
//!
//! ## Spike pattern
//!
//! | Test | Focus |
//! |------|-------|
//! | [`spike_pattern_applies_correct_fraction`] | `spike(V, 4, 2)` → exactly half the leaves spiked |
//! | [`no_spikes_when_spike_count_is_zero`] | `spike_count = 0` leaves tree uniform |
//! | [`all_spiked_when_count_equals_group_size`] | `count == group_size` → every leaf spiked |
//! | [`custom_base_and_spike_intensities`] | arbitrary base/spike values pass through correctly |
//!
//! ## Violations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`uniform_tree_has_no_violations`] | uniform intensities produce no violations |
//! | [`spiked_tree_produces_violations`] | large intensity contrast yields ≥ 1 violation |
//! | [`structural_intensities_recomputed_after_spike`] | structural sums match children after spike + recompute |

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

/// Collect all entry (leaf) node IDs in left-to-right order via DFS.
///
/// Children are pushed in reverse so left children are popped first.
fn collect_leaf_ids<V: Accumulator>(vnodes: &Arena<VNode<V>>, root: VNodeId) -> Vec<VNodeId> {
    let mut leaves = Vec::new();
    let mut stack = vec![root];
    while let Some(id) = stack.pop() {
        match &vnodes.get(id.index()).kind {
            VKind::Entry { .. } => leaves.push(id),
            VKind::Structural { children, .. } => {
                // Push in reverse so index 0 (leftmost) is popped first.
                for i in (0..children.len()).rev() {
                    stack.push(children.get(i).0);
                }
            }
        }
    }
    leaves
}

// ── Helper: count all occupied arena slots ──────────────────────

/// Total node count (entries + structural).
fn total_node_count<V: Accumulator>(vnodes: &Arena<VNode<V>>) -> usize {
    vnodes.iter_occupied().count()
}

// ── Tests ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::tests::init_tracing;

    // ── Tree structure ──────────────────────────────────────

    #[test]
    fn balanced_depth_0_is_single_entry() {
        let _t = init_tracing();
        let (vnodes, root, violations) = SpikedVTree::<u64>::balanced(0).build();

        assert!(matches!(vnodes.get(root.index()).kind, VKind::Entry { .. }));
        assert_eq!(total_node_count(&vnodes), 1);
        assert!(violations.is_empty(), "single entry cannot be violated");
    }

    #[test]
    fn balanced_depth_1_has_two_leaves_and_one_structural() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::<u64>::balanced(1).build();

        assert!(matches!(vnodes.get(root.index()).kind, VKind::Structural { .. }));
        let leaves = collect_leaf_ids(&vnodes, root);
        assert_eq!(leaves.len(), 2);
        assert_eq!(total_node_count(&vnodes), 3); // 2 leaves + 1 structural
    }

    /// `2^depth` leaves, `2^(depth+1) − 1` total nodes for depths 0–6.
    #[test]
    fn balanced_depth_n_node_counts() {
        let _t = init_tracing();
        for depth in 0..=6 {
            let (vnodes, root, _) = SpikedVTree::<u64>::balanced(depth).build();
            let leaves = collect_leaf_ids(&vnodes, root);
            let expected_leaves = 1usize << depth;
            let expected_total = (1usize << (depth + 1)) - 1;
            assert_eq!(
                leaves.len(),
                expected_leaves,
                "depth {depth}: expected {expected_leaves} leaves, got {}",
                leaves.len(),
            );
            assert_eq!(
                total_node_count(&vnodes),
                expected_total,
                "depth {depth}: expected {expected_total} total nodes, got {}",
                total_node_count(&vnodes),
            );
        }
    }

    /// `leaf_count()` on the builder agrees with the actual built tree.
    #[test]
    fn leaf_count_matches_actual_leaves() {
        let _t = init_tracing();
        for depth in 0..=5 {
            let builder = SpikedVTree::<u64>::balanced(depth);
            let expected = builder.leaf_count();
            let (vnodes, root, _) = builder.build();
            let actual = collect_leaf_ids(&vnodes, root).len();
            assert_eq!(expected, actual, "depth {depth}");
        }
    }

    /// Before spiking, every leaf has `base_intensity`.
    #[test]
    fn all_leaves_have_base_intensity_before_spike() {
        let _t = init_tracing();
        // Use spike_count=0 → no spiking occurs.
        let (vnodes, root, _) = SpikedVTree::<u64>::balanced(3).base(42u64).spike(999u64, 4, 0).build();

        for &leaf in &collect_leaf_ids(&vnodes, root) {
            assert_eq!(vnodes.get(leaf.index()).intensity, 42);
        }
    }

    /// Every structural node in the balanced tree is a 2-node.
    #[test]
    fn every_structural_node_has_two_children() {
        let _t = init_tracing();
        let (vnodes, _root, _) = SpikedVTree::<u64>::balanced(4).build();

        for (idx, _) in vnodes.iter_occupied() {
            if let VKind::Structural { children, .. } = &vnodes.get(idx).kind {
                assert_eq!(children.len(), 2, "node {idx} has {} children", children.len());
            }
        }
    }

    /// Every non-root node's parent link points to a node that lists
    /// it as a child.
    #[test]
    fn parent_links_are_consistent() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::<u64>::balanced(4).build();

        for (idx, _) in vnodes.iter_occupied() {
            let id = VNodeId::from_index(idx);
            let node = vnodes.get(idx);
            if id == root {
                assert!(node.parent.is_none(), "root should have no parent");
            } else {
                let parent_id = node.parent.expect("non-root should have parent");
                let parent = vnodes.get(parent_id.index());
                match &parent.kind {
                    VKind::Structural { children, .. } => {
                        let found = (0..children.len()).any(|i| children.get(i).0 == id);
                        assert!(found, "node {idx} not found among parent's children");
                    }
                    VKind::Entry { .. } => panic!("parent of node {idx} is an entry"),
                }
            }
        }
    }

    // ── Spike pattern ───────────────────────────────────────

    /// With `spike(V, 4, 2)`, exactly half the leaves are spiked.
    #[test]
    fn spike_pattern_applies_correct_fraction() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::balanced(4).base(1u64).spike(9999u64, 4, 2).build();

        let leaves = collect_leaf_ids(&vnodes, root);
        let spiked = leaves.iter().filter(|&&id| vnodes.get(id.index()).intensity == 9999).count();
        let base = leaves.len() - spiked;

        // 16 leaves, groups of 4 → 4 groups × 2 spiked = 8 spiked, 8 base
        assert_eq!(spiked, 8, "expected 8 spiked leaves, got {spiked}");
        assert_eq!(base, 8, "expected 8 base leaves, got {base}");
    }

    #[test]
    fn no_spikes_when_spike_count_is_zero() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::balanced(3).base(5u64).spike(9999u64, 4, 0).build();

        for &leaf in &collect_leaf_ids(&vnodes, root) {
            assert_eq!(vnodes.get(leaf.index()).intensity, 5, "no leaf should be spiked");
        }
    }

    #[test]
    fn all_spiked_when_count_equals_group_size() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::balanced(3).base(1u64).spike(500u64, 4, 4).build();

        for &leaf in &collect_leaf_ids(&vnodes, root) {
            assert_eq!(vnodes.get(leaf.index()).intensity, 500, "every leaf should be spiked");
        }
    }

    /// Verify `.base()` and `.spike()` accept arbitrary intensities.
    #[test]
    fn custom_base_and_spike_intensities() {
        let _t = init_tracing();
        let (vnodes, root, _) = SpikedVTree::balanced(2).base(7u64).spike(42u64, 2, 1).build();

        let leaves = collect_leaf_ids(&vnodes, root);
        let intensities: Vec<u64> = leaves.iter().map(|&id| vnodes.get(id.index()).intensity).collect();

        // 4 leaves, groups of 2, 1 spiked per group → [42, 7, 42, 7]
        assert_eq!(intensities, vec![42, 7, 42, 7]);
    }

    // ── Violations ──────────────────────────────────────────

    /// A uniform tree (no spikes) produces no violated nodes.
    #[test]
    fn uniform_tree_has_no_violations() {
        let _t = init_tracing();
        let (_, _, violations) = SpikedVTree::balanced(4)
            .base(10u64)
            .spike(10u64, 4, 0) // no spiking
            .build();

        assert!(violations.is_empty(), "uniform tree should have no violations");
    }

    /// A spiked tree with large intensity contrast produces violations.
    #[test]
    fn spiked_tree_produces_violations() {
        let _t = init_tracing();
        let (_, _, violations) = SpikedVTree::balanced(4).base(1u64).spike(10_000u64, 4, 2).build();

        assert!(!violations.is_empty(), "spiked tree should have at least one violation");
    }

    /// After `build()`, structural node intensities equal the sum of
    /// their children's intensities (bottom-up recomputation).
    #[test]
    fn structural_intensities_recomputed_after_spike() {
        let _t = init_tracing();
        let (vnodes, _root, _) = SpikedVTree::balanced(3).base(1u64).spike(100u64, 4, 2).build();

        for (idx, _) in vnodes.iter_occupied() {
            if let VKind::Structural { children, .. } = &vnodes.get(idx).kind {
                let child_sum: u64 = (0..children.len())
                    .map(|i| vnodes.get(children.get(i).0.index()).intensity)
                    .sum();
                assert_eq!(
                    vnodes.get(idx).intensity,
                    child_sum,
                    "structural node {idx}: intensity {} != child sum {child_sum}",
                    vnodes.get(idx).intensity,
                );
            }
        }
    }
}
