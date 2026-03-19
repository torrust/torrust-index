// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::{Rng, SpatialRead};

/// Intensity-weighted proportional sampling.
///
/// # Summary
///
/// Extends [`SpatialRead`] — read-only access to the contour and
/// point queries.  [`sample()`](Self::sample) draws a random leaf
/// cell with probability proportional to its accumulated intensity.
/// Expected cost is $O(1.44\,H + 1.67)$ where $H$ is the Shannon
/// entropy of the intensity distribution
/// ([ADR-M-019]).
///
/// Use this trait as a capability bound when a function only needs
/// to read and sample — it documents that no mutation occurs.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), when film is dipped in
/// developer, molecules undergo Brownian motion through the
/// gelatin.  The probability a molecule initiates reduction at a
/// given grain is proportional to that grain's latent image cluster
/// size.  [`sample()`](Self::sample) is that weighted random
/// encounter.
///
/// # Contract
///
/// ## RNG integration
///
/// The [`Rng`] parameter is satisfied by any `rand_core::Rng`
/// type when the `rand` feature is enabled (on by default), so
/// `rand::rng()` and friends work out of the box.
///
/// ## Object safety
///
/// `WeightedSampler` is **not** object-safe due to `impl Rng` on
/// `sample()`.  Same reasoning as
/// [`SpatialWrite`](super::SpatialWrite) — the trait is for static
/// dispatch and capability bounding, not trait objects.
///
/// # Implementations
///
/// [`GvGraph<C, V, N>`](crate::GvGraph) is the primary implementor.
///
/// # Examples
///
/// Accept any samplable structure via a trait bound:
///
/// ```
/// use torrust_mudlark::{WeightedSampler, Rng, Cell};
///
/// fn draw<S: WeightedSampler<Coord = u64, Accum = u64>>(
///     s: &S,
///     rng: &mut impl Rng,
/// ) -> Option<Cell<u64, u64>> {
///     s.sample(rng)
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
/// g.observe(42, 10u64);
/// # struct SimpleRng(u64);
/// # impl Rng for SimpleRng {
/// #     fn next_f64(&mut self) -> f64 {
/// #         self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
/// #         (self.0 >> 11) as f64 / (1u64 << 53) as f64
/// #     }
/// # }
/// let mut rng = SimpleRng(12345);
/// let cell = draw(&g, &mut rng).unwrap();
/// assert!(cell.start <= 42 && 42 < cell.end);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
/// - [ADR-M-019] — Sampling semantics and expected cost
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
/// [ADR-M-019]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/019-sampling-semantics.md
pub trait WeightedSampler: SpatialRead {
    /// Draw a random leaf cell weighted by intensity.
    ///
    /// Returns `None` when the index has zero total intensity
    /// (no observations recorded, or all values scaled to zero).
    /// When entries exist, each cell's selection probability is
    /// proportional to its accumulated intensity — higher-intensity
    /// cells are drawn more often.
    ///
    /// # Examples
    ///
    /// ```
    /// # use torrust_mudlark::{Config, GvGraph, Rng};
    /// use torrust_mudlark::WeightedSampler;
    /// # let cfg = Config {
    /// #     split_threshold: 5u64,
    /// #     depth_create: 3,
    /// #     depth_evict: 6,
    /// #     budget: None,
    /// #     alpha_relax: 0.75,
    /// #     bounded_eviction: true,
    /// # };
    /// # let mut g = GvGraph::<u64, u64, 8>::new(cfg);
    /// # struct SimpleRng(u64);
    /// # impl Rng for SimpleRng {
    /// #     fn next_f64(&mut self) -> f64 {
    /// #         self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
    /// #         (self.0 >> 11) as f64 / (1u64 << 53) as f64
    /// #     }
    /// # }
    /// // Empty graph — no intensity to sample from.
    /// let mut rng = SimpleRng(12345);
    /// assert!(WeightedSampler::sample(&g, &mut rng).is_none());
    ///
    /// g.observe(100, 20u64);
    /// let cell = WeightedSampler::sample(&g, &mut rng).unwrap();
    /// assert!(cell.start <= 100 && 100 < cell.end);
    /// ```
    fn sample(&self, rng: &mut impl Rng) -> Option<crate::view::Cell<Self::Coord, Self::Accum>>;
}
