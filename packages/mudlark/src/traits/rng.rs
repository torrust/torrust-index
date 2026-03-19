// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

/// Randomness source for proportional sampling.
///
/// # Summary
///
/// Required by [`GvGraph::sample`](crate::GvGraph::sample) (via the
/// [`WeightedSampler`](super::WeightedSampler) instrument trait).
/// Crate-local to avoid a hard dependency on any RNG crate.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), when film is dipped in
/// developer, molecules undergo Brownian motion through the gelatin.
/// The probability a molecule initiates reduction at a given grain is
/// proportional to that grain's latent image cluster size.  `Rng` is
/// that thermal diffusion — an external source of randomness driving
/// a weighted random encounter.
///
/// # Contract
///
/// ## `rand` feature integration
///
/// When the `rand` feature is enabled (default), a blanket impl
/// covers all `rand_core::Rng` types — so `rand::rngs::StdRng`,
/// `rand::rng()`, etc. work out of the box (ADR-M-009
/// DC-009-1, DC-009-2).
///
/// ## User extension
///
/// Only `Rng` and [`Observation`](super::Observation) are designed
/// for user extension.  A custom implementation is useful when
/// deterministic replay or a lightweight non-`rand` generator is
/// needed.
///
/// # Implementations
///
/// When the `rand` feature is enabled (default), all
/// `rand_core::Rng` types receive a blanket `Rng` impl.
///
/// # Examples
///
/// Implement a simple deterministic generator for testing:
///
/// ```
/// use torrust_mudlark::Rng;
///
/// struct FixedRng(f64);
///
/// impl Rng for FixedRng {
///     fn next_f64(&mut self) -> f64 {
///         self.0
///     }
/// }
///
/// let mut rng = FixedRng(0.42);
/// assert_eq!(rng.next_f64(), 0.42);
/// assert_eq!(rng.next_f64(), 0.42); // deterministic
/// ```
///
/// In context — `Rng` drives proportional sampling on a `GvGraph`:
///
/// ```
/// # use torrust_mudlark::{Config, GvGraph, Rng};
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
/// # struct SimpleRng(u64);
/// # impl Rng for SimpleRng {
/// #     fn next_f64(&mut self) -> f64 {
/// #         self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
/// #         (self.0 >> 11) as f64 / (1u64 << 53) as f64
/// #     }
/// # }
/// let mut rng = SimpleRng(12345);
/// let cell = g.sample(&mut rng).unwrap();
/// assert!(cell.intensity > 0);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Rng {
    /// Return a uniformly distributed `f64` in `[0, 1)`.
    fn next_f64(&mut self) -> f64;
}

#[cfg(feature = "rand")]
impl<T: rand_core::Rng> Rng for T {
    #[inline]
    #[allow(clippy::cast_precision_loss)] // intentional: top 53 bits of u64 → f64
    fn next_f64(&mut self) -> f64 {
        // Standard conversion: top 53 bits of u64 → f64 in [0, 1).
        (self.next_u64() >> 11) as f64 * (1.0 / (1u64 << 53) as f64)
    }
}
