// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! V-Node: the tournament node of the Value Tree.
//!
//! A single `VNode<V>` type with a `VKind<V>` discriminant covers both
//! V-Entries (G-node membership tokens, always V-Tree leaves) and
//! V-Structural nodes (pure scaffolding with 2–3 children).
//!
//! Target size: 64 bytes for `VNode<u64>` (one cache line).

use std::sync::atomic::{AtomicU32, Ordering};

use crate::handle::{GNodeId, VNodeId};

/// Sentinel indicating cached depth is stale (needs recompute).
/// See ADR-M-029 for design rationale.
pub const DEPTH_STALE: u32 = u32::MAX;

/// A node in the V-Tree (tournament bracket).
///
/// - For entries: `intensity` = backing G-node's `own` value.
/// - For structural nodes: `intensity` = sum of children's intensities.
///
/// # Layout (64 bytes for `V = u64`)
///
/// The `cached_depth` field uses the 4-byte padding gap between `parent`
/// and `kind`, achieving zero memory overhead. See ADR-M-029.
#[derive(Debug)]
pub struct VNode<V> {
    /// Intensity (sum for structural, own for entry).
    pub intensity: V,
    /// Parent in the V-Tree.
    pub parent: Option<VNodeId>,
    /// Cached V-Tree depth. `DEPTH_STALE` indicates needs recompute.
    /// Uses `AtomicU32` for thread safety while allowing read-path caching (ADR-M-029).
    pub cached_depth: AtomicU32,
    /// Entry or structural discriminant.
    pub kind: VKind<V>,
}

// Manual Clone impl since AtomicU32 doesn't derive Clone
impl<V: Clone> Clone for VNode<V> {
    fn clone(&self) -> Self {
        Self {
            intensity: self.intensity.clone(),
            parent: self.parent,
            cached_depth: AtomicU32::new(self.cached_depth.load(Ordering::Relaxed)),
            kind: self.kind.clone(),
        }
    }
}

// Static assertion: VNode<u64> fits in one cache line (64 bytes).
// The cached_depth field occupies the 4-byte padding gap between
// parent (4 bytes) and kind (48 bytes, align 8). See ADR-M-029.
const _: () = assert!(std::mem::size_of::<VNode<u64>>() == 64);

/// Discriminant for V-node type. See §IDEA M-4.2 and §IDEA M-4.3.
#[derive(Debug, Clone)]
pub enum VKind<V> {
    /// A V-Tree leaf backed by a G-node (§IDEA M-4.2).
    Entry {
        /// The backing G-node.
        gnode: GNodeId,
        /// True iff the backing G-node is on the contour
        /// (has uncovered range). True for terminal AND semi-internal. §IDEA M-4.2.
        is_exposed: bool,
        /// Implementation cache: true iff the backing G-node has zero
        /// children and can be safely evicted. True for terminal only.
        /// Caches `¬has_dependents(gnode)` to avoid gnodes chase in
        /// propagation and eviction scan.
        is_evictable: bool,
    },
    /// Pure structural scaffolding with 2 or 3 children (§IDEA M-4.3).
    Structural {
        /// Packed child intensities and IDs (`SoA` layout).
        children: PackedChildren<V>,
        /// True iff any descendant entry backs a
        /// 0-child G-node (governs eviction scan pruning). §IDEA M-4.3.
        has_evictable: bool,
    },
}

// ── PackedChildren ──────────────────────────────────────────────────

/// `SoA` (struct-of-arrays) storage for 2 or 3 V-node children.
///
/// Intensities are packed contiguously for cache-friendly sampling:
/// the sampling decision reads 24 bytes of intensities in one scan,
/// then follows a single 4-byte child ID.
///
/// # Layout (40 bytes for `V = u64`)
///
/// ```text
/// intensities: [V; 3]           — 24 bytes (hot: sampling scan)
/// ids:         [Option<VNodeId>; 3] — 12 bytes (cold: follow after choice)
/// len:         u8                — 1 byte (2 or 3)
/// ```
#[derive(Debug, Clone)]
pub struct PackedChildren<V> {
    /// Cached child intensities (hot data for sampling).
    pub intensities: [V; 3],
    /// Child node IDs.
    pub ids: [Option<VNodeId>; 3],
    /// Number of children: 2 or 3.
    pub len: u8,
}

impl<V: Copy + Default + PartialOrd> PackedChildren<V> {
    /// Create a 2-child node.
    #[must_use]
    pub fn new_2(a: (VNodeId, V), b: (VNodeId, V)) -> Self {
        Self {
            intensities: [a.1, b.1, V::default()],
            ids: [Some(a.0), Some(b.0), None],
            len: 2,
        }
    }

    /// Create a 3-child node.
    #[must_use]
    pub const fn new_3(a: (VNodeId, V), b: (VNodeId, V), c: (VNodeId, V)) -> Self {
        Self {
            intensities: [a.1, b.1, c.1],
            ids: [Some(a.0), Some(b.0), Some(c.0)],
            len: 3,
        }
    }

    /// Number of children (2 or 3).
    #[must_use]
    #[inline]
    pub const fn len(&self) -> usize {
        self.len as usize
    }

    /// Returns `true` if there are no children. (Always false for a valid node.)
    #[must_use]
    #[inline]
    #[allow(dead_code)]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Get the `(id, intensity)` pair at `index`.
    ///
    /// # Panics
    ///
    /// Panics if `index >= self.len()`.
    #[must_use]
    #[inline]
    pub fn get(&self, index: usize) -> (VNodeId, V) {
        assert!(index < self.len(), "PackedChildren::get out of bounds");
        (
            self.ids[index].expect("child ID should be Some within len"),
            self.intensities[index],
        )
    }

    /// Iterate over `(id, intensity)` pairs.
    pub fn iter(&self) -> impl Iterator<Item = (VNodeId, V)> + '_ {
        (0..self.len()).map(|i| self.get(i))
    }

    /// Index of the child with the highest intensity.
    /// Ties broken by leftmost (index 0) wins.
    #[must_use]
    pub fn heaviest_child_index(&self) -> usize {
        let mut max_idx = 0;
        for i in 1..self.len() {
            // Strict greater-than: leftmost wins ties.
            if self.intensities[i] > self.intensities[max_idx] {
                max_idx = i;
            }
        }
        max_idx
    }

    /// Find the position of `id` in this node's children.
    #[must_use]
    pub fn find_index(&self, id: VNodeId) -> Option<usize> {
        (0..self.len()).find(|&i| self.ids[i] == Some(id))
    }

    /// Replace one child with another.
    ///
    /// # Panics
    ///
    /// Panics if `old` is not found among the children.
    pub fn replace_child(&mut self, old: VNodeId, new: VNodeId, new_intensity: V) {
        let idx = self.find_index(old).expect("replace_child: old id not found");
        self.ids[idx] = Some(new);
        self.intensities[idx] = new_intensity;
    }

    /// Add a child (2-node → 3-node).
    ///
    /// # Panics
    ///
    /// Panics if already a 3-node.
    pub fn add_child(&mut self, id: VNodeId, intensity: V) {
        assert!(self.len == 2, "add_child: already a 3-node");
        self.ids[2] = Some(id);
        self.intensities[2] = intensity;
        self.len = 3;
    }

    /// Remove a child by ID (3-node → 2-node), returning the removed
    /// `(id, intensity)`.
    ///
    /// # Panics
    ///
    /// Panics if not a 3-node or if `id` is not found.
    pub fn remove_child(&mut self, id: VNodeId) -> (VNodeId, V) {
        assert!(self.len == 3, "remove_child: not a 3-node");
        let idx = self.find_index(id).expect("remove_child: id not found");
        let removed = self.get(idx);

        // Shift the last child into the removed slot if needed.
        if idx < 2 {
            self.ids[idx] = self.ids[2];
            self.intensities[idx] = self.intensities[2];
        }
        self.ids[2] = None;
        self.intensities[2] = V::default();
        self.len = 2;
        removed
    }

    /// Update the cached intensity at `index`.
    #[inline]
    pub fn update_intensity(&mut self, index: usize, new: V) {
        debug_assert!(index < self.len());
        self.intensities[index] = new;
    }
}

// ── Default impls ────────────────────────────────────────────────────

impl<V: Default> Default for VNode<V> {
    fn default() -> Self {
        Self {
            intensity: V::default(),
            parent: None,
            cached_depth: AtomicU32::new(DEPTH_STALE),
            kind: VKind::Entry {
                gnode: GNodeId::from_index(0), // dead sentinel
                is_exposed: false,
                is_evictable: false,
            },
        }
    }
}

impl<V: Default + Copy> Default for PackedChildren<V> {
    fn default() -> Self {
        Self {
            intensities: [V::default(); 3],
            ids: [None; 3],
            len: 0,
        }
    }
}

// ── Size check ───────────────────────────────────────────────────────

#[cfg(test)]
mod tests {}
