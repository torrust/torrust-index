// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::{Observation, SpatialRead};

/// Exposure instrument for recording observations.
///
/// # Summary
///
/// Extends [`SpatialRead`] — every writer can also read.
/// [`observe()`](Self::observe) routes a value delta to the correct
/// spatial leaf and triggers the structural adaptation that follows
/// — splits (§IDEA M-10), V-Tree rebalances (§IDEA M-11), and
/// budget-driven evictions (§IDEA M-12).  Temporal scaling lives in
/// the separate [`TemporalDecay`](super::TemporalDecay) trait,
/// which carries a stricter `V: Attenuatable` bound (ADR-M-009
/// Addendum 3).
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), photons strike grains to
/// form latent image centers.  `SpatialWrite` is that exposure
/// source — a controlled beam that deposits energy at a specific
/// position on the film plane.
///
/// # Contract
///
/// ## Object safety
///
/// `SpatialWrite` is **not** object-safe due to
/// `O: Observation<Self::Accum>` on `observe()`.  This is by
/// design — the trait is for static dispatch and capability
/// bounding, not for trait objects.
///
/// # Implementations
///
/// [`GvGraph<C, V, N>`](crate::GvGraph) is the primary implementor.
///
/// # Examples
///
/// Accept any writable spatial structure via a trait bound:
///
/// ```
/// use torrust_mudlark::{SpatialRead, SpatialWrite};
///
/// fn inject<S: SpatialWrite<Coord = u64, Accum = u64>>(
///     s: &mut S,
///     coord: u64,
///     value: u64,
/// ) {
///     s.observe(coord, value);
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
/// inject(&mut g, 42, 3);
/// assert_eq!(g.get(42).intensity, 3);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait SpatialWrite: SpatialRead {
    /// Record an observation of `delta` at `coord` — photon strike.
    ///
    /// Routes through the G-Tree to the leaf covering `coord`,
    /// accumulates `delta`, and triggers structural adaptation:
    /// splits (§IDEA M-10), V-Tree rebalances (§IDEA M-11),
    /// budget-driven evictions (§IDEA M-12).
    ///
    /// The observation type `O` is typically the same as `V`
    /// (same-type blanket impl), but cross-type deltas (e.g. `f32`
    /// into a `u32` accumulator) are also supported via
    /// [`Observation`](super::Observation).
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph};
    /// use torrust_mudlark::SpatialWrite;
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// SpatialWrite::observe(&mut g, 100, 3u64);
    /// assert_eq!(g.get(100).intensity, 3);
    /// ```
    fn observe<O: Observation<Self::Accum>>(&mut self, coord: Self::Coord, delta: O);
}
