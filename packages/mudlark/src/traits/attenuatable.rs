// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::Accumulator;

/// Multiplicative scaling for temporal decay operations.
///
/// # Summary
///
/// Sub-trait of [`Accumulator`] that adds multiplicative modulation.
/// Required by [`GvGraph::decay`](crate::GvGraph::decay) (via the
/// [`TemporalDecay`](super::TemporalDecay) instrument trait).
/// Without this bound, calling `decay()` is a compile error —
/// types that cannot be meaningfully scaled (e.g. ordinal rankings)
/// simply omit this impl.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), different halide crystals
/// (`AgBr`, `AgCl`) respond differently to heat: some disperse their
/// latent image clusters faster than others.  `Attenuatable` encodes
/// that thermal sensitivity — it says "this accumulator type can be
/// multiplicatively modulated."
///
/// # Contract
///
/// ## Three regimes (§IDEA M-1.3.5, §IDEA M-14)
///
/// The spec names the `decay()` operation `modulate` and defines
/// three regimes depending on the scaling factor:
///
/// | Factor | Regime | Effect |
/// |--------|--------|--------|
/// | `0.0` | Annihilation | Hard reset — all competitive standing lost |
/// | `(0, 1)` | Attenuation | Regression — clusters disperse, recency wins |
/// | `1.0` | Identity | No change |
/// | `> 1.0` | Amplification | Intensification — fine-scale structure reinforced |
///
/// The method is named `attenuate` (matching the most common regime),
/// but the factor is unconstrained — amplification works correctly.
///
/// > **Note (spec naming):** §IDEA M-8.8.2 calls this capability
/// > "Temporal scaling" and the operation `modulate` — regime-neutral
/// > multiplicative scaling.  The trait name `Attenuatable` and
/// > method name `attenuate` reflect the dominant use case
/// > (regression) but do not preclude amplification or annihilation.
///
/// # Implementations
///
/// All built-in numeric types (`u8`–`u128`, `f32`, `f64`) implement
/// `Attenuatable`.  For integers, scaling uses `f64` intermediate
/// arithmetic with truncation toward zero.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Attenuatable;
///
/// // Attenuation (regression): factor < 1.
/// assert_eq!(100u64.attenuate(0.5), 50);
///
/// // Annihilation: factor = 0.
/// assert_eq!(100u64.attenuate(0.0), 0);
///
/// // Identity: factor = 1.
/// assert_eq!(100u64.attenuate(1.0), 100);
///
/// // Amplification (intensification): factor > 1.
/// assert_eq!(100u64.attenuate(2.0), 200);
/// ```
///
/// In context — `Attenuatable` enables temporal scaling on a
/// `GvGraph`:
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
/// g.observe(42, 100u64);
/// assert_eq!(g.total_sum(), 100);
///
/// // Thermal regression at 50%.
/// let root = g.g_root();
/// g.decay(root, 0.5, 0.0);
/// assert!(g.total_sum() < 100);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Attenuatable: Accumulator {
    /// Scale `self` by a multiplicative `factor`.
    ///
    /// The factor is unconstrained: values in `(0, 1)` attenuate,
    /// `0.0` annihilates (producing `zero()`), and values above
    /// `1.0` amplify.
    ///
    /// For built-in types: `(self as f64 * factor) as Self`.
    /// Custom types may snap to the nearest representable level.
    #[must_use]
    fn attenuate(self, factor: f64) -> Self;
}

// ── Unsigned integer implementations ────────────────────────────────

macro_rules! impl_attenuatable_uint {
    ($($ty:ty),+) => {$(
        impl Attenuatable for $ty {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn attenuate(self, factor: f64) -> Self {
                (self as f64 * factor) as Self
            }
        }
    )+};
}

impl_attenuatable_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Attenuatable for f32 {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn attenuate(self, factor: f64) -> Self {
        (f64::from(self) * factor) as Self
    }
}

impl Attenuatable for f64 {
    #[inline]
    fn attenuate(self, factor: f64) -> Self {
        self * factor
    }
}
