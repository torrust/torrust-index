// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Opaque handles for G-tree and V-tree nodes.
//!
//! [`GNodeId`] and [`VNodeId`] are lightweight, `Copy` identity tokens
//! that let you refer to a specific node inside a [`GvGraph`] without
//! exposing how nodes are stored internally. Think of them as serial
//! numbers stamped onto a film grain: the number is enough to locate
//! the grain later, but reveals nothing about the crystal structure
//! underneath.
//!
//! | Handle    | Identifies                                     | Photography analogy               |
//! |-----------|------------------------------------------------|-----------------------------------|
//! | `GNodeId` | A spatial node in the G-tree (`[0, 2^N)`)      | Grain serial number               |
//! | `VNodeId` | A significance node in the V-tree (tournament) | Developing priority tag           |
//!
//! # When you need handles
//!
//! Most day-to-day operations — [`observe`], [`get`], [`extract`],
//! [`plateaus`], [`sample`] — work without handles. The one public
//! method that *requires* a handle is [`TemporalDecay::decay`], which
//! needs a [`GNodeId`] to know where to start the decay walk:
//!
//! ```
//! use torrust_mudlark::{Config, GvGraph, TemporalDecay};
//!
//! let cfg = Config {
//!     split_threshold: 5u64,
//!     depth_create: 3,
//!     depth_evict: 6,
//!     budget: None,
//!     alpha_relax: 0.75,
//!     bounded_eviction: true,
//! };
//! let mut g = GvGraph::<u64, u64, 8>::new(cfg);
//! g.observe(42, 100u64);
//!
//! // Obtain the root handle, then decay the whole tree by 50 %.
//! let root = g.g_root();
//! g.decay(root, 0.5, 0.0);
//!
//! assert!(g.total_sum() < 100);
//! ```
//!
//! You may also use [`GvGraph::v_root`] to inspect the V-tree root for
//! diagnostic or logging purposes.
//!
//! # Obtaining handles
//!
//! | Method              | Returns             |
//! |---------------------|---------------------|
//! | [`GvGraph::g_root`] | `GNodeId` (always)  |
//! | [`GvGraph::v_root`] | `Option<VNodeId>`   |
//!
//! Both root handles are assigned at construction and never change.
//!
//! # Representation
//!
//! Internally both types wrap [`NonZeroU32`], so `Option<GNodeId>` and
//! `Option<VNodeId>` are niche-optimized to 4 bytes with no
//! discriminant overhead. The zero-based arena index is accessible via
//! [`index()`](GNodeId::index) and can be round-tripped through
//! [`from_index()`](GNodeId::from_index) for serialization or logging.
//!
//! [`GvGraph`]: crate::GvGraph
//! [`observe`]: crate::SpatialWrite::observe
//! [`get`]: crate::SpatialRead::get
//! [`extract`]: crate::GvGraph::extract
//! [`plateaus`]: crate::SpatialRead::plateaus
//! [`sample`]: crate::WeightedSampler::sample
//! [`GvGraph::g_root`]: crate::GvGraph::g_root
//! [`GvGraph::v_root`]: crate::GvGraph::v_root
//! [`TemporalDecay::decay`]: crate::TemporalDecay::decay

use std::num::NonZeroU32;

/// Opaque handle to a node in the G-tree (spatial hierarchy).
///
/// A `GNodeId` is a lightweight identity token — a *grain serial
/// number* in the [photography analogy] — that refers to a single node
/// in the geometric tree partitioning `[0, 2^N)` into dyadic
/// intervals. The handle is cheap (`Copy`, 4 bytes) and carries no
/// spatial or value data: you pass it back to the [`GvGraph`] to
/// obtain that information.
///
/// Every graph begins with a root G-node covering the full domain;
/// splits create children as observations arrive. The root handle is
/// always index `0` and never changes.
///
/// # Where you use it
///
/// | Operation                    | Role of `GNodeId`                          |
/// |------------------------------|--------------------------------------------|
/// | [`TemporalDecay::decay`]     | Selects the subtree to attenuate           |
/// | [`Node::gnode_id`]           | Map key for per-cell tracker state         |
/// | [`GvGraph::gnode_info`]      | Read-only structural snapshot              |
/// | [`GvGraph::is_ancestor_of`]  | Ancestry check for coordination walks      |
/// | [`from_index`] / [`index`]   | Serialization round-trips and logging      |
///
/// Most common workflow: obtain the root via [`GvGraph::g_root`], then
/// pass it to [`decay()`](crate::TemporalDecay::decay).
///
/// `GNodeId` implements `Copy`, `Debug`, `PartialEq`, `Eq`, and
/// `Hash` — you can store it, compare it, and use it as a map key.
///
/// [photography analogy]: crate#three-surface-visibility-model-adr-032
/// [`GvGraph`]: crate::GvGraph
/// [`GvGraph::g_root`]: crate::GvGraph::g_root
/// [`GvGraph::gnode_info`]: crate::GvGraph::gnode_info
/// [`GvGraph::is_ancestor_of`]: crate::GvGraph::is_ancestor_of
/// [`Node::gnode_id`]: crate::Node::gnode_id
/// [`TemporalDecay::decay`]: crate::TemporalDecay::decay
/// [`from_index`]: Self::from_index
/// [`index`]: Self::index
///
/// # Examples
///
/// Retrieve the root handle and pass it to `decay()`:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, GNodeId, TemporalDecay};
///
/// let cfg = Config {
///     split_threshold: 5u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// g.observe(42, 100u64);
///
/// // The root handle is always index 0 and never changes.
/// let root: GNodeId = g.g_root();
/// assert_eq!(root.index(), 0);
///
/// // Use it to decay the entire tree by 50 %.
/// let before = g.total_sum();
/// g.decay(root, 0.5, 0.0);
/// assert!(g.total_sum() < before);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GNodeId(NonZeroU32);

/// Opaque handle to a node in the V-tree (significance hierarchy).
///
/// A `VNodeId` is a *developing priority tag* — it identifies a node
/// in the value tree, the tournament bracket that orders entries by
/// intensity so high-value regions sit near the root for efficient
/// proportional sampling.
///
/// # When you need it
///
/// In most workflows you will **not** interact with `VNodeId`
/// directly. The V-tree drives [`sample()`] and [`extract()`]
/// internally; those methods return [`Cell`] and [`Pewei`] values
/// instead of raw handles.
///
/// `VNodeId` is surfaced for **diagnostic and logging** use cases —
/// for example, confirming the V-tree root exists or correlating
/// handles in [`dump_gtree`] output.
///
/// Obtain the root handle via [`GvGraph::v_root`], which returns
/// `Option<VNodeId>` — always `Some` for a graph created with
/// [`GvGraph::new`].
///
/// `VNodeId` implements `Copy`, `Debug`, `PartialEq`, `Eq`, and
/// `Hash` — same ergonomics as [`GNodeId`].
///
/// [`sample()`]: crate::WeightedSampler::sample
/// [`extract()`]: crate::GvGraph::extract
/// [`Cell`]: crate::Cell
/// [`Pewei`]: crate::Pewei
/// [`dump_gtree`]: crate::invariants::dump_gtree
/// [`GvGraph::v_root`]: crate::GvGraph::v_root
/// [`GvGraph::new`]: crate::GvGraph::new
///
/// # Examples
///
/// Verify the V-tree root exists and inspect its index:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, VNodeId};
///
/// let cfg = Config {
///     split_threshold: 5u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let g = GvGraph::<u64, u64, 8>::new(cfg);
///
/// // A freshly constructed graph always has a V-tree root.
/// let v_root: VNodeId = g.v_root().expect("V-root present after new()");
/// assert_eq!(v_root.index(), 0);
///
/// // Both trees start at index 0 — they share the initial node.
/// assert_eq!(g.g_root().index(), v_root.index());
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct VNodeId(NonZeroU32);

macro_rules! impl_handle {
    ($ty:ident) => {
        impl $ty {
            /// Create a handle from a 0-based arena index
            ///
            /// Converts a `usize` slot position into the corresponding
            /// handle. This is the inverse of [`Self::index`].
            ///
            /// Most users never need to call this directly — handles are
            /// returned by graph methods such as [`GvGraph::g_root`] and
            /// [`GvGraph::v_root`]. `from_index` exists for
            /// serialization round-trips, logging, and diagnostic tools
            /// that reconstitute a previously stored index.
            ///
            /// [`GvGraph::g_root`]: crate::GvGraph::g_root
            /// [`GvGraph::v_root`]: crate::GvGraph::v_root
            ///
            /// # Panics
            ///
            /// Panics if `index >= u32::MAX as usize` (exhausts the
            /// address space).
            ///
            /// # Examples
            ///
            /// Round-trip an index through a handle:
            ///
            /// ```
            /// use torrust_mudlark::{GNodeId, VNodeId};
            ///
            /// let g = GNodeId::from_index(42);
            /// assert_eq!(g.index(), 42);
            ///
            /// let v = VNodeId::from_index(0);
            /// assert_eq!(v.index(), 0);
            /// ```
            ///
            /// Reconstruct a handle from a previously stored index:
            ///
            /// ```
            /// # use torrust_mudlark::{Config, GvGraph, GNodeId};
            /// # let cfg = Config {
            /// #     split_threshold: 5u64,
            /// #     depth_create: 3,
            /// #     depth_evict: 6,
            /// #     budget: None,
            /// #     alpha_relax: 0.75,
            /// #     bounded_eviction: true,
            /// # };
            /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
            /// // Suppose we stored the root's index earlier.
            /// let saved_index = g.g_root().index();
            ///
            /// // Later, reconstitute the handle.
            /// let restored = GNodeId::from_index(saved_index);
            /// assert_eq!(restored, g.g_root());
            /// ```
            #[must_use]
            pub fn from_index(index: usize) -> Self {
                let raw = u32::try_from(index)
                    .ok()
                    .and_then(|i| i.checked_add(1))
                    .and_then(NonZeroU32::new)
                    .expect(concat!(stringify!($ty), ": index out of range"));
                Self(raw)
            }

            /// Return the 0-based arena index
            ///
            /// Converts this handle back into the `usize` slot position
            /// used internally. This is the inverse of
            /// [`Self::from_index`].
            ///
            /// Useful for serialization, logging, or using handles as
            /// indices into your own external data structures.
            ///
            /// # Examples
            ///
            /// ```
            /// use torrust_mudlark::GNodeId;
            ///
            /// let h = GNodeId::from_index(7);
            /// assert_eq!(h.index(), 7);
            /// ```
            ///
            /// Use the index to tag external per-node data:
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
            /// # let g = GvGraph::<u64, u64, 8>::new(cfg);
            /// let root = g.g_root();
            /// let mut labels: Vec<&str> = vec![""; 64];
            /// labels[root.index()] = "root";
            /// assert_eq!(labels[0], "root");
            /// ```
            #[must_use]
            #[inline]
            pub const fn index(self) -> usize {
                (self.0.get() - 1) as usize
            }
        }
    };
}

impl_handle!(GNodeId);
impl_handle!(VNodeId);

#[cfg(test)]
mod tests {}
