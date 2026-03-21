// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Snapshot view types returned by [`GvGraph`](crate::GvGraph) queries.
//!
//! These are the primary result types when reading from the index.
//! All three are lightweight `Copy` structs (~24–40 bytes for
//! `<u64, u64>`) that snapshot field values at construction, so they
//! never borrow the graph.
//!
//! | Type     | Returned by   | Use case |
//! |----------|---------------|----------|
//! | [`Cell`] | [`get()`](crate::GvGraph::get), [`sample()`](crate::GvGraph::sample) | Contour cell snapshot (terminal or uncovered semi-internal half) |
//! | [`Node`] | [`layers()`](crate::GvGraph::layers) | Any G-node snapshot (`own`, `sum`, `state`) |
//! | [`Span`] | [`Cell::to_span`], [`Node::to_span`], [`Pewei::reconstruct`](crate::Pewei::reconstruct) | Owned dyadic interval + intensity |
//!
//! Most users will work primarily with [`Cell`] from point queries
//! and sampling. [`Node`] appears when iterating the full tree in
//! significance order via [`layers()`](crate::GvGraph::layers), and
//! [`Span`] serves as the common denominator when you need only
//! geometry + intensity.

use crate::gnode::GState;
use crate::handle::GNodeId;
use crate::traits::{Accumulator, Coordinate};

// ── Span ─────────────────────────────────────────────────────────────

/// An owned dyadic interval with a single intensity value
///
/// `Span` is the common denominator of [`Cell`] and [`Node`] — it
/// carries the spatial range (`start..end`), an `intensity`, and the
/// tree `depth`, with no node-identity or tree-state information.
///
/// You obtain a `Span` by:
/// - converting a [`Cell`] or [`Node`] via [`.to_span()`](Cell::to_span), or
/// - reconstructing a signal from a [`Pewei`](crate::Pewei) via
///   [`reconstruct()`](crate::Pewei::reconstruct).
///
/// # Examples
///
/// Obtain a `Span` from a point-query result:
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Cell};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// # g.observe(42, 10u64);
/// let cell = g.get(42);
/// let span = cell.to_span();
///
/// // All fields transfer unchanged.
/// assert_eq!(span.start, cell.start);
/// assert_eq!(span.end, cell.end);
/// assert_eq!(span.intensity, cell.intensity);
/// assert_eq!(span.depth, cell.depth);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Span<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Intensity value for this interval.
    pub intensity: V,
    /// G-Tree depth at which this interval lives.
    pub depth: u32,
}

impl<C: Coordinate, V: Accumulator> Span<C, V> {
    /// Width of this interval: `end - start`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let span = g.get(42).to_span();
    /// assert_eq!(span.width(), span.end - span.start);
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }
}

// ── Cell ─────────────────────────────────────────────────────────────

/// Snapshot of a single G-node's directly-accumulated energy.
///
/// `Cell` is the most common query result — returned by
/// [`get()`](crate::GvGraph::get) (infallible point query) and
/// [`sample()`](crate::GvGraph::sample) (proportional sampling).
///
/// The typical case is a **contour cell** — a terminal G-node or
/// the uncovered half of a semi-internal G-node. However,
/// `sample()` can also land on an **internal** G-node whose
/// V-entry still carries frozen pre-split intensity (see
/// [ADR-M-019] and §IDEA M-6.5).
///
/// | G-node state  | Interval              | `intensity`          |
/// |---------------|-----------------------|----------------------|
/// | Terminal      | full `[lo, hi)`       | `g.own` (= `g.sum`) |
/// | Semi-internal | uncovered half only   | `g.own`              |
/// | Internal      | full `[lo, hi)`       | `g.own` (frozen)     |
///
/// For the uncovered half of a semi-internal node, the interval is
/// narrowed to the vacated half and `intensity` is the node's
/// `g.own` — direct accumulation only (pre-split + absorbed +
/// post-eviction observations routed to that half; §IDEA M-5.5.1).
/// For internal nodes both halves have children, so no narrowing
/// occurs; `intensity` is the frozen baseline from before the split.
///
/// `get()` routes through the G-Tree and always reaches a terminal
/// or semi-internal node. `sample()` walks the V-Tree, where an
/// internal G-node's V-entry persists with its frozen `g.own`
/// weight.
///
/// [ADR-M-019]: https://github.com/torrust/torrust-index/blob/main/packages/mudlark/adr/019-sampling-semantics.md
///
/// # Examples
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph};
/// # let cfg = Config {
/// #     split_threshold: 5u64,
/// #     depth_create: 3,
/// #     depth_evict: 6,
/// #     budget: None,
/// #     alpha_relax: 0.75,
/// #     bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// g.observe(42, 10u64);
///
/// // `get` always returns the contour cell containing the coordinate.
/// let cell = g.get(42);
/// assert!(cell.start <= 42 && 42 < cell.end);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Cell<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Intensity: `g.own`. Equals `g.sum` for terminals; for
    /// semi-internal uncovered halves this is the node's direct
    /// accumulation only.
    pub intensity: V,
    /// G-Tree depth of this cell.
    pub depth: u32,
}

impl<C: Coordinate, V: Accumulator> Cell<C, V> {
    /// Convert to a [`Span`] (lossless — same fields).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, Span};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let cell = g.get(42);
    /// let span = cell.to_span();
    /// assert_eq!(span.start, cell.start);
    /// assert_eq!(span.intensity, cell.intensity);
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_span(self) -> Span<C, V> {
        Span {
            start: self.start,
            end: self.end,
            intensity: self.intensity,
            depth: self.depth,
        }
    }

    /// Width of this cell: `end - start`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let cell = g.get(42);
    /// assert_eq!(cell.width(), cell.end - cell.start);
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }

    /// Whether this cell is at the finest possible resolution for
    /// an `N`-bit domain
    ///
    /// A "final" cell covers a single unit interval and cannot be
    /// subdivided further. For integer coordinates in an 8-bit domain
    /// (`N = 8`) that means `width() == 1` — a single point in
    /// `[0, 256)`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 10u64);
    ///
    /// // After a single observation with depth_create=3, the cell
    /// // is still much wider than a unit interval — not final.
    /// let cell = g.get(42);
    /// assert!(!cell.is_final(8));
    /// ```
    #[inline]
    #[must_use]
    pub fn is_final(&self, n: u32) -> bool {
        C::is_final(self.start, self.end, self.depth, n)
    }
}

// ── Node ─────────────────────────────────────────────────────────────

/// Snapshot of any G-node (terminal, semi-internal, or internal)
///
/// Unlike [`Cell`] (which captures only leaf nodes), `Node` exposes
/// both `own` (direct accumulation) and `sum` (own + children's
/// sums), together with the node's categorical [`GState`]. Nodes
/// are yielded by [`GvGraph::layers()`](crate::GvGraph::layers),
/// which walks the V-Tree in significance order — highest-energy
/// nodes first.
///
/// # Examples
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
/// # g.observe(42, 10u64);
/// // `layers()` yields nodes in V-Tree order (highest energy first).
/// let (_layer_index, node) = g.layers().next().unwrap();
///
/// // Every node covers a non-empty dyadic interval.
/// assert!(node.start < node.end);
/// // `sum` is always >= `own` (sum includes children's contributions).
/// assert!(node.sum >= node.own);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Node<C: Coordinate, V: Accumulator> {
    /// Lower bound of the dyadic range (inclusive).
    pub start: C,
    /// Upper bound of the dyadic range (exclusive).
    pub end: C,
    /// Direct accumulation at this node.
    pub own: V,
    /// Total value: `own + children.sum`.
    pub sum: V,
    /// G-Tree depth of this node.
    pub depth: u32,
    /// Categorical state at snapshot time.
    pub state: GState,
    /// Arena handle of the backing G-node (ADR-M-036 D1).
    ///
    /// Useful as a map key (e.g. `BTreeMap<GNodeId, CellState>`) and
    /// for passing to [`GvGraph::gnode_info()`](crate::GvGraph::gnode_info)
    /// or [`GvGraph::is_ancestor_of()`](crate::GvGraph::is_ancestor_of).
    pub gnode_id: GNodeId,
    /// Stable provenance: the G-node that was split to produce this
    /// one (ADR-M-032). `None` at the root. Determined at creation;
    /// immutable for the node's lifetime.
    pub parent: Option<GNodeId>,
}

impl<C: Coordinate, V: Accumulator> Node<C, V> {
    /// Convert to a [`Span`] using `sum` as the intensity.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let (_layer, node) = g.layers().next().unwrap();
    /// let span = node.to_span();
    /// assert_eq!(span.start, node.start);
    /// assert_eq!(span.intensity, node.sum);
    /// ```
    #[inline]
    #[must_use]
    pub const fn to_span(self) -> Span<C, V> {
        Span {
            start: self.start,
            end: self.end,
            intensity: self.sum,
            depth: self.depth,
        }
    }

    /// Try to convert to a [`Cell`]. Returns `Some` only for terminal
    /// nodes (where `own == sum`).
    ///
    /// # Examples
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
    /// # g.observe(42, 10u64);
    /// let (_layer, node) = g.layers().next().unwrap();
    /// if node.state == GState::Terminal {
    ///     let cell = node.to_cell().unwrap();
    ///     assert_eq!(cell.intensity, node.own);
    /// } else {
    ///     assert!(node.to_cell().is_none());
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn to_cell(self) -> Option<Cell<C, V>> {
        if self.state == GState::Terminal {
            Some(Cell {
                start: self.start,
                end: self.end,
                intensity: self.own,
                depth: self.depth,
            })
        } else {
            None
        }
    }

    /// Whether this node is a terminal (leaf) in the G-Tree.
    ///
    /// # Examples
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
    /// # g.observe(42, 10u64);
    /// let (_layer, node) = g.layers().next().unwrap();
    /// assert_eq!(node.is_terminal(), node.state == GState::Terminal);
    /// ```
    #[inline]
    #[must_use]
    pub fn is_terminal(&self) -> bool {
        self.state == GState::Terminal
    }

    /// Refinement energy: `sum - own`
    ///
    /// Quantifies how much energy in this node's region comes from
    /// child nodes rather than direct accumulation. For terminal
    /// nodes (no children) this is always zero; for internal or
    /// semi-internal nodes it reveals how much spatial detail exists
    /// beneath this level of the tree.
    ///
    /// # Examples
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
    /// # g.observe(42, 10u64);
    /// for (_layer, node) in g.layers() {
    ///     if node.is_terminal() {
    ///         // Leaves have no children, so refinement is zero.
    ///         assert_eq!(node.refinement(), 0);
    ///     }
    ///     // For any node: refinement + own == sum.
    ///     assert_eq!(node.refinement() + node.own, node.sum);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn refinement(&self) -> V {
        V::sub(self.sum, self.own)
    }

    /// Whether this node is the root of the G-Tree.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let (_layer, node) = g.layers().next().unwrap();
    /// assert_eq!(node.is_root(), node.parent.is_none());
    /// ```
    #[inline]
    #[must_use]
    pub const fn is_root(&self) -> bool {
        self.parent.is_none()
    }

    /// Width of this node's interval: `end - start`.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let (_layer, node) = g.layers().next().unwrap();
    /// assert_eq!(node.width(), node.end - node.start);
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }
}

#[cfg(test)]
mod tests {}
