// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::Accumulator;

/// Fractional subdivision for range queries and wavelet reconstruction.
///
/// # Summary
///
/// Sub-trait of [`Accumulator`] that adds proportional scaling.
/// Required by [`GvGraph::range_sum`](crate::GvGraph::range_sum) for
/// partial-overlap pro-rating (§IDEA M-5.5.2) and by
/// [`Pewei::reconstruct`](crate::Pewei::reconstruct) for
/// energy-conserving wavelet reconstruction.  Without this bound,
/// those methods are compile errors.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), when a densitometer
/// aperture partially overlaps a grain, the measured density is
/// pro-rated by the overlap fraction.  `Proratable` encodes this
/// capability: given a total value and a fractional portion, compute
/// the proportional contribution.
///
/// # Contract
///
/// [`prorate`](Self::prorate) returns [`zero()`](Accumulator::zero)
/// when `total == 0`.  Integer impls use `u128` intermediate
/// arithmetic and truncate toward zero.  Float impls use native
/// arithmetic.
///
/// # Implementations
///
/// All built-in types (`u8`–`u128`, `f32`, `f64`) implement
/// `Proratable`.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Proratable;
///
/// // Pro-rate: 100 × (3 / 4) = 75 for integers (truncating).
/// assert_eq!(100u64.prorate(3, 4), 75);
///
/// // Full portion returns the original value.
/// assert_eq!(100u64.prorate(4, 4), 100);
///
/// // Zero portion returns zero.
/// assert_eq!(100u64.prorate(0, 4), 0);
///
/// // scale_by with a floating-point ratio.
/// assert_eq!(100u64.scale_by(0.25), 25);
/// ```
///
/// In context — `Proratable` enables range queries on a `GvGraph`:
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
/// g.observe(50, 10u64);
/// g.observe(150, 20u64);
///
/// let total = g.range_sum(..);
/// assert_eq!(total, 30);
///
/// // Sub-range containing only the first observation's region.
/// let partial = g.range_sum(0..128);
/// assert!(partial > 0);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Proratable: Accumulator {
    /// Pro-rate: `self × (portion / total)`.
    ///
    /// Integer impls use `u128` intermediate arithmetic and truncate
    /// toward zero.  Float impls use native arithmetic.
    /// Returns `zero()` when `total == 0`.
    #[must_use]
    fn prorate(self, portion: u64, total: u64) -> Self;

    /// Scale by a floating-point ratio.
    ///
    /// Used by range-query overlap calculation where the overlap
    /// fraction is already computed as `f64`.
    #[must_use]
    fn scale_by(self, ratio: f64) -> Self;
}

// ── Unsigned integer implementations ────────────────────────────────

macro_rules! impl_proratable_uint {
    ($($ty:ty),+) => {$(
        impl Proratable for $ty {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
            fn prorate(self, portion: u64, total: u64) -> Self {
                if total == 0 { return <Self as Accumulator>::zero(); }
                ((self as u128) * (portion as u128) / (total as u128)) as Self
            }

            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn scale_by(self, ratio: f64) -> Self {
                (self as f64 * ratio) as Self
            }
        }
    )+};
}

impl_proratable_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Proratable for f32 {
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn prorate(self, portion: u64, total: u64) -> Self {
        if total == 0 {
            return <Self as Accumulator>::zero();
        }
        self * (portion as Self / total as Self)
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn scale_by(self, ratio: f64) -> Self {
        (f64::from(self) * ratio) as Self
    }
}

impl Proratable for f64 {
    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn prorate(self, portion: u64, total: u64) -> Self {
        if total == 0 {
            return <Self as Accumulator>::zero();
        }
        self * (portion as Self / total as Self)
    }

    #[inline]
    fn scale_by(self, ratio: f64) -> Self {
        self * ratio
    }
}
