// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::Accumulator;

/// Scalar weight projection for proportional sampling.
///
/// # Summary
///
/// Sub-trait of [`Accumulator`] that projects values to `f64` for
/// use as probability weights.  Required by
/// [`GvGraph::sample`](crate::GvGraph::sample) (via
/// [`WeightedSampler`](super::WeightedSampler)) and by
/// [`Transition::snr`](crate::Transition::snr) for signal-to-noise
/// computation.  Without this bound, calling `sample()` or `snr()`
/// is a compile error.
///
/// Semantically distinct from
/// [`Inspectable::to_f64_approx`](super::Inspectable::to_f64_approx)
/// — `weight()` is a *correctness-critical* projection used for
/// probability computation, while `to_f64_approx` is a *diagnostic*
/// projection used for display and debug assertions.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), a grain's
/// **developability** is how likely it is to be reduced by developer
/// — proportional to its latent image cluster size.
/// [`WeightedSampler::sample`](super::WeightedSampler) draws a
/// random grain with probability proportional to this weight, the
/// same way developer molecules encounter grains with probability
/// proportional to their cluster size during Brownian diffusion.
///
/// # Contract
///
/// The returned value must be non-negative (P1 guarantees this for
/// the Standard configuration).
///
/// # Implementations
///
/// All built-in types (`u8`–`u128`, `f32`, `f64`) implement
/// `Weighable`.  Integers project to `f64` via `as` cast (lossless
/// up to $2^{53}$); floats are identity.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Weighable;
///
/// // Integer weights — lossless within f64 precision.
/// assert_eq!(42u64.weight(), 42.0);
/// assert_eq!(0u64.weight(), 0.0);
///
/// // Float weights — identity projection.
/// assert_eq!(3.14f64.weight(), 3.14);
/// ```
///
/// In context — `Weighable` enables proportional sampling on a
/// `GvGraph`:
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
/// g.observe(200, 5u64);
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
pub trait Weighable: Accumulator {
    /// Project to `f64` for use as a proportional weight.
    ///
    /// The returned value must be non-negative (P1 guarantees this
    /// for the Standard configuration).
    fn weight(self) -> f64;
}

// ── Unsigned integer implementations ────────────────────────────────

macro_rules! impl_weighable_uint {
    ($($ty:ty),+) => {$(
        impl Weighable for $ty {
            #[inline]
            #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
            fn weight(self) -> f64 {
                self as f64
            }
        }
    )+};
}

impl_weighable_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Weighable for f32 {
    #[inline]
    fn weight(self) -> f64 {
        f64::from(self)
    }
}

impl Weighable for f64 {
    #[inline]
    fn weight(self) -> f64 {
        self
    }
}
