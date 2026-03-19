// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::{Accumulator, Coordinate};

/// Read-only spatial interface for contour projection and point queries.
///
/// # Summary
///
/// The read half of the spatial capability split ([§API M-5.4]).
/// [`plateaus()`](Self::plateaus) maps the isodensity contour zones
/// across the full domain, and [`get()`](Self::get) takes a spot
/// reading at a single coordinate.  Use it as a **capability
/// bound** when a function only needs to read from the index —
/// accepting `&impl SpatialRead` documents that intent and enables
/// mock-based testing.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), a **densitometer**
/// measures the latent image's density profile without altering the
/// negative.  `SpatialRead` is that non-destructive measurement.
///
/// # Contract
///
/// ## Object safety
///
/// `SpatialRead` is object-safe:
/// `&dyn SpatialRead<Coord = u64, Accum = u64>` compiles.  This
/// enables trait-object mocking in test harnesses.
///
/// # Implementations
///
/// [`GvGraph<C, V, N>`](crate::GvGraph) is the primary implementor.
///
/// # Examples
///
/// Accept any readable spatial structure via a trait bound:
///
/// ```
/// use torrust_mudlark::{SpatialRead, Cell};
///
/// fn lookup<S: SpatialRead<Coord = u64, Accum = u64>>(
///     s: &S,
///     coord: u64,
/// ) -> Cell<u64, u64> {
///     s.get(coord)
/// }
///
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
/// g.observe(42, 3u64);
/// let cell = lookup(&g, 42);
/// assert_eq!(cell.intensity, 3);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
/// - [§API M-5.4] — Instrument capability split
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
/// [§API M-5.4]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/docs/api.md#54-instruments
pub trait SpatialRead {
    /// Coordinate type.
    type Coord: Coordinate;
    /// Accumulator / intensity type.
    type Accum: Accumulator;

    /// Return the contour projection as a sorted map of plateaus.
    ///
    /// Each entry in the returned `BTreeMap` represents one
    /// contiguous region of uniform spatial resolution.  Keys are
    /// [`BasisEdge<C>`](crate::BasisEdge) (left-edge coordinate of
    /// each plateau) and together they partition the entire domain
    /// — no gaps, no overlaps.
    ///
    /// With the `dynamic-contour-tracking` feature (on by default):
    /// `Cow::Borrowed` — $O(1)$.
    /// Without: `Cow::Owned` — $O(G)$ tree walk to rebuild.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// use torrust_mudlark::SpatialRead;
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(10, 5u64);
    /// let plateaus = g.plateaus();
    /// assert!(!plateaus.is_empty());
    /// ```
    #[allow(clippy::type_complexity)]
    fn plateaus(
        &self,
    ) -> std::borrow::Cow<
        '_,
        std::collections::BTreeMap<crate::plateau::BasisEdge<Self::Coord>, crate::plateau::Plateau<Self::Coord, Self::Accum>>,
    >;

    /// Infallible point query — spot densitometer reading.
    ///
    /// Returns the leaf-level [`Cell`](crate::Cell) that contains
    /// `coord`, with the half-open interval `[start, end)` and
    /// accumulated intensity.  Coordinates outside the domain
    /// `[0, 2^N)` are clamped to the nearest boundary
    /// (§IDEA M-3.2.5).
    ///
    /// Cost: $O(\text{depth})$.
    ///
    /// # Panics
    ///
    /// Panics if `coord` is NaN (float coordinate types only;
    /// §IDEA M-3.2.4).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// use torrust_mudlark::SpatialRead;
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// g.observe(42, 3u64);
    /// let cell = SpatialRead::get(&g, 42);
    /// assert!(cell.start <= 42 && 42 < cell.end);
    /// assert_eq!(cell.intensity, 3);
    /// ```
    fn get(&self, coord: Self::Coord) -> crate::view::Cell<Self::Coord, Self::Accum>;
}
