// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use super::Accumulator;

/// Diagnostic `f64` projection for display, logging, and invariant checks.
///
/// # Summary
///
/// Sub-trait of [`Accumulator`] that projects values to `f64` for
/// human consumption.  Required by most core
/// [`GvGraph`](crate::GvGraph) operations:
/// [`observe`](crate::GvGraph::observe),
/// [`extract`](crate::GvGraph::extract),
/// [`layers`](crate::GvGraph::layers), and
/// [`check_evictions`](crate::GvGraph::check_evictions).  In
/// practice, any useful `V` type should implement this — it is the
/// most commonly needed sub-trait.
///
/// Semantically distinct from
/// [`Weighable::weight`](super::Weighable::weight) — this is for
/// human-readable output and debug assertions, not
/// correctness-critical sampling weights.
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), a lab densitometer
/// converts crystal measurements to a standard numeric scale for
/// quality-control checks and logging.  `Inspectable` is that
/// calibration step — it says "this accumulator type can be
/// projected to an approximate `f64` for human consumption."
///
/// # Contract
///
/// > **Note (spec divergence — `observe()` requires
/// > `Inspectable`):** §IDEA M-8.8.1 defines observation as a
/// > core operation needing only `zero` + `add`.  This
/// > implementation requires `Inspectable` on `observe()` because
/// > the internal split and eviction machinery triggered during
/// > observation uses diagnostic projections (threshold comparisons
/// > against `to_f64_approx`, invariant assertions).  This is an
/// > implementation-level restriction, not a spec-level one — a
/// > future refactoring could decouple them.
///
/// # Implementations
///
/// All built-in accumulator types (`u8`–`u128`, `f32`, `f64`)
/// implement `Inspectable`.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Inspectable;
///
/// // Round-trip through f64 for diagnostic display.
/// let v = 42u64;
/// let approx = v.to_f64_approx();
/// assert_eq!(approx, 42.0);
///
/// // Reconstruct from f64.
/// let restored = u64::from_f64(approx);
/// assert_eq!(restored, 42);
///
/// // Floats are identity.
/// assert_eq!(3.14f64.to_f64_approx(), 3.14);
/// assert_eq!(f64::from_f64(3.14), 3.14);
/// ```
///
/// In context — `Inspectable` gates
/// [`observe()`](crate::GvGraph::observe),
/// [`extract()`](crate::GvGraph::extract), and
/// [`layers()`](crate::GvGraph::layers):
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
/// g.observe(42, 10u64);
/// g.observe(100, 20u64);
///
/// // Contact print from the negative.
/// let pewei = g.extract();
/// assert!(pewei.layer_count() > 0);
///
/// // Stream V-Tree layers (BFS significance order).
/// let nodes: Vec<_> = g.layers().collect();
/// assert!(!nodes.is_empty());
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Inspectable: Accumulator {
    /// Return an approximate `f64` representation for display purposes.
    ///
    /// May be lossy (e.g. `u128` values > 2^53 lose precision).
    fn to_f64_approx(self) -> f64;

    /// Construct a value from an `f64` (inverse of [`to_f64_approx`](Self::to_f64_approx)).
    ///
    /// Used by test harnesses and diagnostic code that needs to
    /// synthesize accumulator values from floating-point quantities.
    fn from_f64(v: f64) -> Self;
}

// ── Unsigned integer implementations ────────────────────────────────

macro_rules! impl_inspectable_uint {
    ($($ty:ty),+) => {$(
        impl Inspectable for $ty {
            #[inline]
            #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
            fn to_f64_approx(self) -> f64 {
                self as f64
            }

            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss, clippy::cast_lossless)]
            fn from_f64(v: f64) -> Self {
                v as Self
            }
        }
    )+};
}

impl_inspectable_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Inspectable for f32 {
    #[inline]
    fn to_f64_approx(self) -> f64 {
        f64::from(self)
    }

    #[inline]
    #[allow(clippy::cast_possible_truncation)]
    fn from_f64(v: f64) -> Self {
        v as Self
    }
}

impl Inspectable for f64 {
    #[inline]
    fn to_f64_approx(self) -> f64 {
        self
    }

    #[inline]
    fn from_f64(v: f64) -> Self {
        v
    }
}
