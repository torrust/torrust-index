// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Plateau types — the contour map of the spatial index.
//!
//! The [`GvGraph`](crate::GvGraph) domain `[0, 2^N)` is tiled by
//! contiguous, non-overlapping **plateaus** — regions where every
//! leaf node sits at the same tree depth.  [`BasisEdge`] marks
//! where each plateau begins; [`Plateau`] captures the region,
//! depth, and accumulated energy.
//!
//! Both public types are re-exported flat from the crate root.
//!
//! | Type              | Visibility   | Purpose                                     |
//! |-------------------|--------------|---------------------------------------------|
//! | [`BasisEdge<C>`]  | `pub`        | `Ord`-providing newtype for `BTreeMap` keys  |
//! | [`Plateau<C, V>`] | `pub`        | Lightweight `Copy` view of one plateau       |
//! | `PlateauBasis<C>` | `pub(crate)` | Bidirectional basis ↔ edge bookkeeping      |
//! | `basis_edge_of`   | `pub(crate)` | Computes basis edge for a G-node (ADR-M-026)   |

use std::cmp::Ordering;
#[cfg(feature = "dynamic-contour-tracking")]
use std::collections::{BTreeMap, HashMap, HashSet};
#[cfg(feature = "dynamic-contour-tracking")]
use std::sync::OnceLock;

use crate::gnode::{GNode, GState};
#[cfg(feature = "dynamic-contour-tracking")]
use crate::handle::GNodeId;
use crate::traits::{Accumulator, Coordinate};
use crate::view::Span;

// ── BasisEdge ────────────────────────────────────────────────────────

/// The coordinate where a new plateau begins on the spatial contour.
///
/// When you call [`GvGraph::plateaus()`](crate::GvGraph::plateaus),
/// the returned `BTreeMap` is keyed by `BasisEdge<C>`.  Each key
/// marks the leftmost coordinate of a contiguous region (a
/// [`Plateau`]) where the index has uniform resolution depth.
///
/// `BasisEdge` is a zero-cost newtype that provides [`Ord`] for any
/// [`Coordinate`](crate::Coordinate) via
/// [`Coordinate::total_cmp`](crate::Coordinate::total_cmp) — the
/// same pattern as [`Reverse<T>`](std::cmp::Reverse) or
/// `OrderedFloat<T>`.  The inner coordinate is public (`self.0`),
/// so you can unwrap it when you need the raw value.
///
/// # Film analogy
///
/// An **isodensity contour boundary** — the coordinate on the film
/// plane where the emulsion's density transitions from one uniform
/// zone to the next.  See
/// [ADR-M-032](https://github.com/torrust/torrust-index/blob/main/packages/mudlark/adr/032-three-surface-model.md)
/// for the full photographic analogy.
///
/// # Examples
///
/// Ordering and equality delegate to the inner coordinate:
///
/// ```
/// use torrust_mudlark::BasisEdge;
///
/// let a = BasisEdge(10u64);
/// let b = BasisEdge(20u64);
/// assert!(a < b);
/// assert_eq!(a, BasisEdge(10u64));
/// assert_eq!(a.0, 10);   // inner coordinate is public
/// ```
///
/// **Floor-key lookup** — find which plateau contains a given point:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, BasisEdge};
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
/// g.observe(42, 10u64);
///
/// let plateaus = g.plateaus();
///
/// // The entry whose key is ≤ 42 owns the plateau containing it.
/// let (_edge, plateau) = plateaus
///     .range(..=BasisEdge(42u64))
///     .next_back()
///     .unwrap();
/// assert!(plateau.start <= 42 && 42 < plateau.end);
/// ```
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BasisEdge<C: Coordinate>(pub C);

impl<C: Coordinate> Ord for BasisEdge<C> {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}

impl<C: Coordinate> PartialOrd for BasisEdge<C> {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl<C: Coordinate> PartialEq for BasisEdge<C> {
    fn eq(&self, other: &Self) -> bool {
        self.cmp(other) == Ordering::Equal
    }
}

impl<C: Coordinate> Eq for BasisEdge<C> {}

// ── Plateau ──────────────────────────────────────────────────────────

/// One contiguous region of uniform resolution depth on the spatial
/// contour.
///
/// The [`GvGraph`](crate::GvGraph) domain `[0, 2^N)` is partitioned
/// into non-overlapping plateaus — zones where every leaf node sits
/// at the same tree depth.  High-activity areas are finely
/// subdivided (deeper plateaus); quiet areas remain coarse
/// (shallower).  Call
/// [`GvGraph::plateaus()`](crate::GvGraph::plateaus) to obtain the
/// full partition as a
/// <code>BTreeMap&lt;[BasisEdge]&lt;C&gt;, Plateau&lt;C, V&gt;&gt;</code>.
///
/// `Plateau` is a lightweight [`Copy`] view type, consistent with
/// [`Cell`](crate::Cell), [`Node`](crate::Node), and
/// [`Span`](crate::Span) — it carries no references back into the
/// graph.
///
/// # Film analogy
///
/// An **isodensity zone** — a region of the negative where all
/// grains sit at the same crystallographic depth: uniform
/// resolution, one characteristic density.  See
/// [ADR-M-032](https://github.com/torrust/torrust-index/blob/main/packages/mudlark/adr/032-three-surface-model.md)
/// for the full photographic analogy.
///
/// # Fields
///
/// | Field        | Meaning                                                  |
/// |--------------|----------------------------------------------------------|
/// | `basis_edge` | [`BasisEdge`] key — plateau start on the contour map     |
/// | `start`      | Spatial range start (inclusive)                           |
/// | `end`        | Spatial range end (exclusive)                             |
/// | `depth`      | Tree depth — higher means finer resolution               |
/// | `sum`        | Total accumulated energy across this plateau's leaf nodes |
///
/// # Examples
///
/// Inspect how the contour evolves as observations arrive:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, BasisEdge};
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
///
/// // A fresh graph has a single plateau covering the entire domain.
/// let p = g.plateaus();
/// assert_eq!(p.len(), 1);
/// let only = p.values().next().unwrap();
/// assert_eq!(only.start, 0);
/// assert_eq!(only.end, 256);   // 2^8
///
/// // Observations that exceed the split threshold refine the contour.
/// for i in 0..10 {
///     g.observe(i * 25, 10u64);
/// }
/// let p = g.plateaus();
/// assert!(p.len() >= 2, "contour refined after observations");
///
/// // Every plateau covers a non-empty range and is self-describing.
/// for (&edge, plateau) in p.iter() {
///     assert!(plateau.start < plateau.end);
///     assert_eq!(edge, plateau.basis_edge);
/// }
/// ```
///
/// Convert a plateau to a [`Span`](crate::Span) for interop with
/// reconstruction or rendering code:
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph};
/// # let cfg = Config {
/// #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
/// #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
/// # };
/// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// # g.observe(42, 10u64);
/// let plateau = *g.plateaus().values().next().unwrap();
/// let span = plateau.to_span();
/// assert_eq!(span.start, plateau.start);
/// assert_eq!(span.intensity, plateau.sum);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct Plateau<C: Coordinate, V: Accumulator> {
    /// [`BasisEdge`] key identifying this plateau in the contour map.
    pub basis_edge: BasisEdge<C>,
    /// Spatial range start (inclusive).
    pub start: C,
    /// Spatial range end (exclusive).
    pub end: C,
    /// G-Tree depth of every leaf node in this plateau.
    /// Higher depth means finer spatial resolution.
    pub depth: u32,
    /// Total accumulated energy across this plateau's leaf nodes.
    pub sum: V,
}

impl<C: Coordinate, V: Accumulator> Plateau<C, V> {
    /// Width of this plateau's spatial range: `end - start`.
    ///
    /// Wider plateaus cover more of the domain at uniform resolution.
    /// A narrow plateau indicates a region where observations have
    /// concentrated, causing the contour to refine locally.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
    /// #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// for plateau in g.plateaus().values() {
    ///     assert_eq!(plateau.width(), plateau.end - plateau.start);
    /// }
    /// ```
    #[inline]
    #[must_use]
    pub fn width(&self) -> C {
        C::width(self.start, self.end)
    }

    /// Convert to a [`Span`] using `sum` as the intensity.
    ///
    /// Useful when you need a uniform `Span` sequence — for example
    /// building a heat-map or feeding plateau data into code that
    /// operates on [`Span`]s rather than `Plateau`s.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// # let cfg = Config {
    /// #     split_threshold: 5u64, depth_create: 3, depth_evict: 6,
    /// #     budget: None, alpha_relax: 0.75, bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # g.observe(42, 10u64);
    /// let plateau = *g.plateaus().values().next().unwrap();
    /// let span = plateau.to_span();
    ///
    /// // All fields transfer: start, end, sum→intensity, depth.
    /// assert_eq!(span.start, plateau.start);
    /// assert_eq!(span.end, plateau.end);
    /// assert_eq!(span.intensity, plateau.sum);
    /// assert_eq!(span.depth, plateau.depth);
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
}

// ── PlateauBasis ─────────────────────────────────────────────────────

/// Internal bookkeeping for plateau ↔ basis element associations.
///
/// Maintains the bidirectional mapping that satisfies P-I1 through
/// P-I5. Updated atomically by `split`/`evict`/`decay` code paths
/// (Step 2).
///
/// Not part of the public API. Keyed by `BasisEdge<C>`, identically
/// to the public `plateaus` `BTreeMap` on `GvGraph`.
///
/// No `C: Ord` required — `BasisEdge<C>` provides `Ord` via
/// `Coordinate::total_cmp`.
#[cfg(feature = "dynamic-contour-tracking")]
#[derive(Debug, Clone)]
pub struct PlateauBasis<C: Coordinate> {
    /// Forward: basis edge → set of basis element `GNodeId`s.
    ///
    /// `HashSet` gives O(1) amortised removal (vs O(n) `Vec::retain`).
    /// Most plateaus have 1–3 basis elements, so the per-element
    /// overhead is negligible.
    forward: BTreeMap<BasisEdge<C>, HashSet<GNodeId>>,
    /// Back: basis element `GNodeId` → its basis edge key.
    back: HashMap<GNodeId, BasisEdge<C>>,
}

#[cfg(feature = "dynamic-contour-tracking")]
impl<C: Coordinate> PlateauBasis<C> {
    /// Create an empty basis tracker.
    pub(crate) fn new() -> Self {
        Self {
            forward: BTreeMap::new(),
            back: HashMap::new(),
        }
    }

    /// Register `gnode` as a basis element of the plateau at `key`.
    ///
    /// # Panics (debug)
    ///
    /// Panics if `gnode` is already registered as a basis element
    /// of any plateau (would violate basis uniqueness).
    pub(crate) fn insert(&mut self, key: BasisEdge<C>, gnode: GNodeId) {
        debug_assert!(
            !self.back.contains_key(&gnode),
            "PlateauBasis::insert: gnode {gnode:?} already in \
             basis of plateau {:?}",
            self.back.get(&gnode)
        );
        self.forward.entry(key).or_default().insert(gnode);
        self.back.insert(gnode, key);
    }

    /// Remove `gnode` from whatever plateau's basis it belongs to.
    ///
    /// Returns the basis edge key it was removed from, or `None`
    /// if the gnode was not a basis element.
    ///
    /// If this was the last basis element for that plateau,
    /// removes the forward entry entirely.
    pub(crate) fn remove(&mut self, gnode: GNodeId) -> Option<BasisEdge<C>> {
        let key = self.back.remove(&gnode)?;
        if let Some(set) = self.forward.get_mut(&key) {
            set.remove(&gnode);
            if set.is_empty() {
                self.forward.remove(&key);
            }
        }
        Some(key)
    }

    /// Look up which basis edge key a basis element belongs to.
    ///
    /// Returns `None` if `gnode` is not a basis element of any
    /// plateau.
    #[inline]
    pub(crate) fn plateau_key(&self, gnode: GNodeId) -> Option<BasisEdge<C>> {
        self.back.get(&gnode).copied()
    }

    /// The basis elements for the plateau at `key`.
    ///
    /// Returns an empty set reference if the key has no entry
    /// (should not happen in a well-maintained graph; defensive).
    pub(crate) fn basis_elements(&self, key: &BasisEdge<C>) -> &HashSet<GNodeId> {
        static EMPTY: OnceLock<HashSet<GNodeId>> = OnceLock::new();
        self.forward.get(key).unwrap_or_else(|| EMPTY.get_or_init(HashSet::new))
    }

    /// Whether `gnode` is currently a basis element of any plateau.
    #[inline]
    #[allow(unused)] // Available for future use; not called yet.
    pub(crate) fn contains(&self, gnode: GNodeId) -> bool {
        self.back.contains_key(&gnode)
    }

    /// Number of tracked plateaus (forward map entries).
    #[inline]
    pub(crate) fn plateau_count(&self) -> usize {
        self.forward.len()
    }

    /// Number of tracked basis elements (back map entries).
    #[inline]
    pub(crate) fn basis_count(&self) -> usize {
        self.back.len()
    }

    /// Iterator over all (key, basis-elements) pairs, in key order.
    ///
    /// Used by `assert_invariants` for the full O(n) sweep.
    #[allow(unused)] // Available for future use; not called yet.
    pub(crate) fn iter(&self) -> impl Iterator<Item = (&BasisEdge<C>, &HashSet<GNodeId>)> {
        self.forward.iter()
    }

    /// Reference to the back-pointer map.
    ///
    /// Used by `assert_invariants` for cross-checking.
    pub(crate) const fn back_map(&self) -> &HashMap<GNodeId, BasisEdge<C>> {
        &self.back
    }

    /// Rebuild both forward and back maps from an explicit list of
    /// `(plateau_key, gnode)` assignments.
    ///
    /// Called by `normalize_plateaus` to re-key basis elements after
    /// the plateau `BTreeMap` is rebuilt with the correct merge logic.
    pub(crate) fn rebuild(&mut self, assignments: impl IntoIterator<Item = (BasisEdge<C>, GNodeId)>) {
        self.forward.clear();
        self.back.clear();
        for (key, gid) in assignments {
            self.forward.entry(key).or_default().insert(gid);
            self.back.insert(gid, key);
        }
    }
}

// ── basis_edge_of ────────────────────────────────────────────────────

/// Compute the basis edge for a G-node acting as a basis element.
///
/// Per ADR-M-026 P-I1:
///
/// | Basis element type                  | `edge(R)`       | Rationale                  |
/// |-------------------------------------|-----------------|----------------------------|
/// | Terminal `[l, r)`                   | `l`             | Contour starts at left     |
/// | Balanced internal `[l, r)`          | `l`             | All subtree same-depth     |
/// | Semi-internal `[l, r)`, child left  | `m` (midpoint)  | Uncovered half is `[m, r)` |
/// | Semi-internal `[l, r)`, child right | `l`             | Uncovered half is `[l, m)` |
///
/// This helper is used by every mutation path (split, evict) when
/// creating or updating basis elements.
pub fn basis_edge_of<C, V>(g: &GNode<C, V>) -> BasisEdge<C>
where
    C: Coordinate,
    V: Accumulator,
{
    match g.state() {
        GState::Terminal | GState::Internal => BasisEdge(g.lo),
        GState::SemiInternal => {
            if g.left.is_some() {
                // Child on left [l, m) → uncovered half is [m, r)
                BasisEdge(C::midpoint(g.lo, g.hi))
            } else {
                // Child on right [m, r) → uncovered half is [l, m)
                BasisEdge(g.lo)
            }
        }
    }
}

#[cfg(test)]
mod tests {}
