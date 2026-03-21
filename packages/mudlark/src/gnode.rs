// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! G-Node: the spatial node of the Geometric Tree.
//!
//! Each G-node covers a dyadic interval `[lo, hi)` and stores both
//! an accumulated sum (for range queries) and an own value (for the
//! V-Tree tournament). Target size: 48 bytes for `GNode<u64, u64>`.
//!
//! # Surface boundary (ADR-M-032)
//!
//! This module straddles two surfaces:
//!
//! - **Surface 1 (Prints):** [`GState`] — lightweight `Copy` enum
//!   derived from child pointers. Re-exported from the crate root.
//! - **Surface 3 (Emulsion):** [`GNode`] — the full spatial node with
//!   `pub(crate)` fields. Internal machinery, not part of the public API.

use crate::handle::{GNodeId, VNodeId};

/// A spatial node in the G-Tree.
///
/// Covers dyadic interval `[lo, hi)` where `hi - lo` is a power of 2.
///
/// - `sum`: total value in this region (own + children's sums). G-Tree
///   ranking.
/// - `own`: direct accumulation at this node. V-Tree ranking.
///
/// See §IDEA M-4.1 for the full field description.
#[derive(Debug, Clone)]
pub struct GNode<C, V> {
    /// Lower bound of the dyadic range (inclusive).
    pub(crate) lo: C,
    /// Upper bound of the dyadic range (exclusive).
    pub(crate) hi: C,
    /// Total value: `own + children.sum`. Maintained by G-Tree propagation.
    pub(crate) sum: V,
    /// Direct accumulation at this node (observations received here).
    pub(crate) own: V,
    /// Left G-child, covering `[lo, mid)`.
    pub(crate) left: Option<GNodeId>,
    /// Right G-child, covering `[mid, hi)`.
    pub(crate) right: Option<GNodeId>,
    /// Parent in the G-Tree.
    pub(crate) parent: Option<GNodeId>,
    /// Cross-tree link: this node's membership token in the V-Tree.
    pub(crate) entry: Option<VNodeId>,
}

/// Categorical state of a spatial region in the G-Tree
///
/// Every node in the spatial index covers a dyadic interval
/// `[start, end)`. As observations accumulate above the split
/// threshold, regions subdivide to gain finer resolution. `GState`
/// tells you how far that subdivision has progressed:
///
/// | Variant | Children | Meaning |
/// |---------|----------|---------|
/// | [`Terminal`](Self::Terminal) | 0 | Leaf — receives observations directly |
/// | [`SemiInternal`](Self::SemiInternal) | 1 | One half subdivided, the other still accumulates locally |
/// | [`Internal`](Self::Internal) | 2 | Fully subdivided — both halves route to children |
///
/// You encounter `GState` as the [`Node::state`](crate::Node::state)
/// field on snapshots returned by
/// [`GvGraph::layers()`](crate::GvGraph::layers), and through
/// [`Node::to_cell()`](crate::Node::to_cell) (which succeeds only
/// for terminals).
///
/// Derived at query time from child pointers — zero storage overhead
/// (ADR-M-016).
///
/// *Silver halide analog (ADR-M-032):* crystal structure class —
/// isolated grain (terminal), partially-clustered aggregate
/// (semi-internal), or fully-subdivided aggregate (internal).
///
/// # Examples
///
/// Inspect the tree structure — terminals are leaves, the rest are
/// branching points with finer spatial detail underneath:
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, GState};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// // Enough energy to trigger subdivision.
/// g.observe(42, 10u64);
///
/// let mut terminals = 0u32;
/// let mut branching = 0u32;
/// for (_layer, node) in g.layers() {
///     match node.state {
///         GState::Terminal => terminals += 1,
///         GState::SemiInternal | GState::Internal => branching += 1,
///     }
/// }
///
/// // At least one terminal must exist (every tree has leaves).
/// assert!(terminals > 0);
/// // The observation exceeded the split threshold, so the tree
/// // has at least one branching point with finer structure.
/// assert!(branching > 0);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum GState {
    /// Zero children — leaf region that receives observations
    /// directly. Returned by [`get()`](crate::GvGraph::get) and
    /// [`sample()`](crate::GvGraph::sample) as a [`Cell`](crate::Cell).
    /// The contour cell's interval is the full G-node range.
    Terminal,
    /// One child — one half of the interval has been subdivided while
    /// the other half still accumulates locally on this node.
    /// [`get()`](crate::GvGraph::get) and
    /// [`sample()`](crate::GvGraph::sample) return the uncovered half
    /// as a [`Cell`](crate::Cell) with `intensity = g.own`.
    SemiInternal,
    /// Two children — both halves route to child nodes. All new
    /// observations pass through to finer-grained regions below.
    /// The node's V-entry retains its frozen `g.own` intensity, so
    /// [`sample()`](crate::GvGraph::sample) can still land here and
    /// return a [`Cell`](crate::Cell) covering the full `[lo, hi)`
    /// range (see ADR-M-019 and §IDEA M-6.5).
    Internal,
}

impl<C, V> GNode<C, V> {
    /// Derive the categorical state from child pointers (ADR-M-016).
    #[inline]
    #[must_use]
    pub const fn state(&self) -> GState {
        match (self.left, self.right) {
            (None, None) => GState::Terminal,
            (Some(_), Some(_)) => GState::Internal,
            _ => GState::SemiInternal,
        }
    }

    /// True iff this node has zero G-children.
    #[inline]
    #[must_use]
    pub const fn is_terminal(&self) -> bool {
        self.left.is_none() && self.right.is_none()
    }

    /// True iff this node has at least one G-child.
    /// Inverse of `is_terminal()`.
    #[inline]
    #[must_use]
    #[allow(dead_code)] // used in tests
    pub const fn has_dependents(&self) -> bool {
        self.left.is_some() || self.right.is_some()
    }

    /// True iff exactly one G-child is present.
    #[inline]
    #[must_use]
    pub const fn is_semi_internal(&self) -> bool {
        matches!(self.state(), GState::SemiInternal)
    }

    /// Returns the uncovered half-interval, or `None` for internal nodes.
    ///
    /// - Terminal: entire range `(lo, hi)`.
    /// - Semi-internal (left present): right half `(mid, hi)`.
    /// - Semi-internal (right present): left half `(lo, mid)`.
    /// - Internal: `None`.
    #[must_use]
    pub fn uncovered_range(&self) -> Option<(C, C)>
    where
        C: crate::traits::Coordinate,
    {
        match (self.left, self.right) {
            (None, None) => Some((self.lo, self.hi)),
            (Some(_), None) => {
                let mid = C::midpoint(self.lo, self.hi);
                Some((mid, self.hi))
            }
            (None, Some(_)) => {
                let mid = C::midpoint(self.lo, self.hi);
                Some((self.lo, mid))
            }
            (Some(_), Some(_)) => None,
        }
    }
}

impl<C: Default, V: Default> Default for GNode<C, V> {
    fn default() -> Self {
        Self {
            lo: C::default(),
            hi: C::default(),
            sum: V::default(),
            own: V::default(),
            left: None,
            right: None,
            parent: None,
            entry: None,
        }
    }
}

#[cfg(test)]
mod tests {}
