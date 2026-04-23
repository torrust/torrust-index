// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Opaque handles for G-tree and V-tree nodes.
//!
//! [`GNodeId`] is the public, generational handle that lets you refer
//! to a specific G-node inside a [`GvGraph`] without exposing how nodes
//! are stored internally. Think of it as a serial number stamped onto
//! a film grain: the number is enough to locate the grain later, but
//! reveals nothing about the crystal structure underneath. The
//! generation counter (ADR-M-040) detects stale handles after slot
//! reuse — a mismatch panics with a diagnostic message.
//!
//! `GSlotPointer` is the lightweight, generation-free `pub(crate)`
//! handle used internally for intra-arena tree walks where no
//! eviction can interleave and the generation check is unnecessary
//! overhead. `VSlotPointer` is the corresponding internal handle for
//! the V-tree (significance hierarchy); it also has no public
//! consuming method (ADR-M-032 handle test).
//!
//! | Handle           | Visibility   | Identifies                                     | Photography analogy               |
//! |------------------|--------------|-------------------------------------------------|-----------------------------------|
//! | `GNodeId`        | `pub`        | A spatial node in the G-tree (`[0, 2^N)`)       | Grain serial number (dated)       |
//! | `GSlotPointer`   | `pub(crate)` | Same, without generation — internal fast path   | Grain serial number (undated)     |
//! | `VSlotPointer`   | `pub(crate)` | A significance node in the V-tree (tournament)  | Developing priority tag           |
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
//! # Obtaining handles
//!
//! | Method              | Returns                |
//! |---------------------|------------------------|
//! | [`GvGraph::g_root`] | `GNodeId` (always)     |
//!
//! The root handle is assigned at construction and never changes.
//! (`v_root()` is `pub(crate)` — available inside the crate only.)
//!
//! # Representation
//!
//! `GNodeId` is 8 bytes (slot index + generation counter).
//! `Option<GNodeId>` is 8 bytes (niche-optimised via `NonZeroU32`).
//!
//! Internally, `GSlotPointer` and `VSlotPointer` wrap [`NonZeroU32`],
//! so `Option<GSlotPointer>` and `Option<VSlotPointer>` are
//! niche-optimized to 4 bytes with no discriminant overhead. The hot
//! internal paths use these 4-byte handles; the 8-byte `GNodeId`
//! exists only at the public API boundary.
//!
//! [`GvGraph`]: crate::GvGraph
//! [`observe`]: crate::SpatialWrite::observe
//! [`get`]: crate::SpatialRead::get
//! [`extract`]: crate::GvGraph::extract
//! [`plateaus`]: crate::SpatialRead::plateaus
//! [`sample`]: crate::WeightedSampler::sample
//! [`GvGraph::g_root`]: crate::GvGraph::g_root
//! [`TemporalDecay::decay`]: crate::TemporalDecay::decay

use std::num::NonZeroU32;

/// Internal, generation-free handle to a G-tree node.
///
/// `GSlotPointer` is a lightweight `Copy` token (4 bytes) used
/// internally for tree walks where no eviction can interleave and
/// the generational check would be unnecessary overhead.
///
/// The public API uses [`GNodeId`] (8 bytes, generational) instead.
/// `GSlotPointer` is `pub(crate)` — never exposed to downstream
/// crates (ADR-M-040 D3, D6).
///
/// `GSlotPointer` implements `Copy`, `Debug`, `PartialEq`, `Eq`,
/// `PartialOrd`, `Ord`, and `Hash`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GSlotPointer(NonZeroU32);

/// Opaque identity token for a G-node in the spatial hierarchy.
///
/// A `GNodeId` is a generational handle — it carries the slot index
/// and a generation counter, allowing detection of use-after-free
/// when a slot is reused (ADR-M-040).
///
/// Both consuming methods (`decay`, `gnode_info`, `is_ancestor_of`)
/// and snapshot types (`Node`, `BasisElement`) use `GNodeId`.
/// The generation check is always performed; a mismatch panics with
/// a diagnostic message.
///
/// `GNodeId` implements `Copy`, `Debug`, `PartialEq`, `Eq`,
/// `PartialOrd`, `Ord`, and `Hash`.
///
/// # Examples
///
/// Obtain the root handle and use it for decay:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, TemporalDecay};
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
/// let root = g.g_root();
/// let before = g.total_sum();
/// g.decay(root, 0.5, 0.0);
/// assert!(g.total_sum() < before);
/// ```
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GNodeId {
    /// 1-indexed arena slot (encoded as `NonZeroU32`).
    index: NonZeroU32,
    /// Generation counter at handle creation time.
    generation: u32,
}

impl GNodeId {
    /// Create a `GNodeId` from a slot index and generation.
    ///
    /// This is the inverse of [`Self::index`] + [`Self::generation`].
    /// Primarily used for serialization and diagnostics (`#[doc(hidden)]`).
    ///
    /// # Panics
    ///
    /// Panics if `index >= u32::MAX as usize` (exhausts the address space).
    #[doc(hidden)]
    #[must_use]
    pub fn from_parts(index: usize, generation: u32) -> Self {
        let raw = u32::try_from(index)
            .ok()
            .and_then(|i| i.checked_add(1))
            .and_then(NonZeroU32::new)
            .expect("GNodeId::from_parts: index out of range");
        Self { index: raw, generation }
    }

    /// Return the 0-based arena index.
    ///
    /// The inverse of [`Self::from_parts`]. Primarily used for
    /// serialization and diagnostics (`#[doc(hidden)]`).
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn index(self) -> usize {
        (self.index.get() - 1) as usize
    }

    /// Return the generation counter at handle creation time.
    ///
    /// Used for serialization and diagnostics (`#[doc(hidden)]`).
    #[doc(hidden)]
    #[must_use]
    #[inline]
    pub const fn generation(self) -> u32 {
        self.generation
    }

    /// Internal: extract the raw slot pointer (no generation check).
    ///
    /// Used internally where we elide the generation check (e.g.,
    /// within a single alloc/observe cycle where no interleaving
    /// eviction can occur).
    pub(crate) const fn slot(self) -> GSlotPointer {
        GSlotPointer(self.index)
    }
}

/// Opaque handle to a node in the V-tree (significance hierarchy).
///
/// A `VSlotPointer` is a *developing priority tag* — it identifies a node
/// in the value tree, the tournament bracket that orders entries by
/// intensity so high-value regions sit near the root for efficient
/// proportional sampling.
///
/// # Visibility
///
/// `VSlotPointer` is `pub(crate)` (ADR-M-032 handle test: no public
/// consuming method). The V-tree drives [`sample()`] and [`extract()`]
/// internally; those methods return [`Cell`] and [`Pewei`] values
/// instead of raw handles.
///
/// Within the crate, obtain the root handle via `GvGraph::v_root()`,
/// which returns `Option<VSlotPointer>` — always `Some` for a graph
/// created with [`GvGraph::new`].
///
/// `VSlotPointer` implements `Copy`, `Debug`, `PartialEq`, `Eq`, and
/// `Hash` — same ergonomics as [`GNodeId`].
///
/// [`sample()`]: crate::WeightedSampler::sample
/// [`extract()`]: crate::GvGraph::extract
/// [`Cell`]: crate::Cell
/// [`Pewei`]: crate::Pewei
/// [`GvGraph::new`]: crate::GvGraph::new
///
/// # Examples
///
/// `VSlotPointer` is `pub(crate)` — see crate-level tests in
/// `src/tests/worked_example.rs` for usage.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[allow(clippy::redundant_pub_crate)]
pub(crate) struct VSlotPointer(NonZeroU32);

macro_rules! impl_slot_handle {
    ($ty:ident) => {
        impl $ty {
            /// Create a handle from a 0-based arena index
            ///
            /// Converts a `usize` slot position into the corresponding
            /// handle. This is the inverse of [`Self::index`].
            ///
            /// Most users never need to call this directly — handles are
            /// returned by graph methods such as [`GvGraph::g_root`].
            /// `from_index` exists for
            /// serialization round-trips, logging, and diagnostic tools
            /// that reconstitute a previously stored index.
            ///
            /// [`GvGraph::g_root`]: crate::GvGraph::g_root
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
            /// ```ignore
            /// // GSlotPointer/VSlotPointer are pub(crate); this
            /// // example illustrates the API but cannot compile
            /// // as a standalone doc test.
            /// let g = GSlotPointer::from_index(42);
            /// assert_eq!(g.index(), 42);
            /// ```
            #[doc(hidden)]
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
            /// ```ignore
            /// // GSlotPointer/VSlotPointer are pub(crate); this
            /// // example illustrates the API but cannot compile
            /// // as a standalone doc test.
            /// let h = GSlotPointer::from_index(7);
            /// assert_eq!(h.index(), 7);
            /// ```
            #[doc(hidden)]
            #[must_use]
            #[inline]
            pub const fn index(self) -> usize {
                (self.0.get() - 1) as usize
            }
        }
    };
}

impl_slot_handle!(GSlotPointer);
impl_slot_handle!(VSlotPointer);

#[cfg(test)]
mod tests {}
