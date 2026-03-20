// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::fmt::Debug;

use super::Accumulator;

/// How an observation event combines with a stored accumulator.
///
/// # Summary
///
/// Bridge between observation input and storage.  A `GvGraph` with
/// `V = u16` can accept observations in `f64`, performing the math
/// at float precision and truncating back to `u16` for storage.  In
/// [`GvGraph::observe`](crate::GvGraph::observe), the `delta`
/// parameter is `O: Observation<V>` — the compiler selects the
/// correct accumulation rule based on the concrete types.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), different photon
/// wavelengths and energies interact with the emulsion differently.
/// `Observation<V>` is that spectral interaction rule: it says
/// "given the current state of a grain (`V`) and an incoming photon
/// (`Self`), what is the new state?"
///
/// # User extension
///
/// Only `Observation` and [`Rng`](super::Rng) are designed for user
/// extension — implement this trait when you need a custom
/// observation type (e.g. a weighted update or a log-domain delta).
///
/// If your observation type also supports multiplicative scaling
/// (e.g. cross-type `f64` → uint), implement
/// [`ScalableObservation`] as well.
///
/// # Implementations
///
/// - **Same-type:** every `V: Accumulator` is `Observation<V>` (via
///   [`Accumulator::add`]).  This is the common case.
/// - **Cross-type:** provided for `f32`/`f64` → integer types (with
///   truncating `as` casts).  See ADR-M-010 for the full cross-type
///   table and ADR-M-011 for overflow/narrowing semantics.
///
/// # Examples
///
/// Same-type accumulation (blanket impl):
///
/// ```
/// use torrust_mudlark::Observation;
///
/// let current = 10u64;
/// let updated = <u64 as Observation<u64>>::accumulate(current, 5u64);
/// assert_eq!(updated, 15);
/// ```
///
/// Cross-type accumulation (`f64` → `u16`):
///
/// ```
/// use torrust_mudlark::{Observation, ScalableObservation};
///
/// let current = 100u16;
/// let updated = <f64 as Observation<u16>>::accumulate(current, 2.7);
/// assert_eq!(updated, 102); // truncated from 102.7
///
/// let scaled = <f64 as ScalableObservation<u16>>::scale(100, 0.5);
/// assert_eq!(scaled, 50);
/// ```
///
/// In context — `Observation` is the delta type for
/// [`GvGraph::observe`](crate::GvGraph::observe):
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
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
///
/// // Same-type observation (u64 → u64, blanket impl).
/// g.observe(42, 10u64);
/// assert_eq!(g.total_sum(), 10);
///
/// // Cross-type observation (f64 → u64).
/// g.observe(100, 5.0f64);
/// assert_eq!(g.total_sum(), 15);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Observation<V: Accumulator>: Copy + Debug + Send + Sync {
    /// Additive update: `current + delta`, in `Self`'s precision,
    /// stored as `V`.
    fn accumulate(current: V, delta: Self) -> V;
}

/// Multiplicative scaling capability for cross-type observations.
///
/// Sub-trait of [`Observation`] — implement this when the observation
/// type supports `current × factor` (e.g. `f64` → uint).  Same-type
/// observations do not need this: `decay()` uses
/// [`Attenuatable::attenuate`](super::Attenuatable::attenuate)
/// directly (ADR-M-024).
///
/// This follows the crate's split-trait pattern: independent
/// capabilities are gated behind independent sub-traits rather than
/// bundled with panicking defaults.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::ScalableObservation;
///
/// // f64 → u16 cross-type scaling.
/// let scaled = <f64 as ScalableObservation<u16>>::scale(100, 0.5);
/// assert_eq!(scaled, 50);
/// ```
pub trait ScalableObservation<V: Accumulator>: Observation<V> {
    /// Multiplicative scaling: `current × factor`, in `Self`'s
    /// precision, stored as `V`.
    fn scale(current: V, factor: Self) -> V;
}

// ── Same-type blanket impl ──────────────────────────────────────────

/// Every `V: Accumulator` is `Observation<V>` (same-type).
///
/// Provides `accumulate` via [`Accumulator::add`].  Same-type `scale`
/// is not provided — `decay()` uses
/// [`Attenuatable::attenuate`](super::Attenuatable::attenuate)
/// directly (ADR-M-024).
impl<V: Accumulator> Observation<V> for V {
    #[inline]
    fn accumulate(current: V, delta: Self) -> V {
        V::add(current, delta)
    }
}

// ── Cross-type: f64 → unsigned integers ─────────────────────────────

macro_rules! impl_observation_f64_to_uint {
    ($($v:ty),+) => {$(
        impl Observation<$v> for f64 {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn accumulate(current: $v, delta: Self) -> $v {
                (current as f64 + delta) as $v
            }
        }

        impl ScalableObservation<$v> for f64 {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn scale(current: $v, factor: Self) -> $v {
                (current as f64 * factor) as $v
            }
        }
    )+};
}

impl_observation_f64_to_uint!(u8, u16, u32, u64, u128);

/// `f64` as observation on `f32` (narrowing).
impl Observation<f32> for f64 {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn accumulate(current: f32, delta: Self) -> f32 {
        (Self::from(current) + delta) as f32
    }
}

/// `f64` as scalable observation on `f32` (narrowing).
impl ScalableObservation<f32> for f64 {
    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn scale(current: f32, factor: Self) -> f32 {
        (Self::from(current) * factor) as f32
    }
}

// Note: f64 → f64 is handled by the blanket impl (same-type).

// ── Cross-type: f32 → unsigned integers ─────────────────────────────

macro_rules! impl_observation_f32_to_uint {
    ($($v:ty),+) => {$(
        impl Observation<$v> for f32 {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn accumulate(current: $v, delta: Self) -> $v {
                (current as f32 + delta) as $v
            }
        }

        impl ScalableObservation<$v> for f32 {
            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless, clippy::cast_precision_loss)]
            fn scale(current: $v, factor: Self) -> $v {
                (current as f32 * factor) as $v
            }
        }
    )+};
}

impl_observation_f32_to_uint!(u8, u16, u32);

// Note: f32 → f32 is handled by the blanket impl (same-type).
