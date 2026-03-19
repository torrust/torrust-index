// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::fmt::Debug;

/// Position type for the spatial domain `[0, 2^N)`.
///
/// # Summary
///
/// The `C` parameter in [`GvGraph<C, V, N>`](crate::GvGraph).
/// Implementations provide dyadic midpoint arithmetic and domain
/// bounds.  The trait requires `Send + Sync` for thread-safe use
/// of `GvGraph` (ADR-M-007).
///
/// # Analogy
///
/// In the silver halide model ([ADR-M-032]), `Coordinate` defines
/// **where** on the film plane a grain sits — the spatial addressing
/// scheme that the G-Tree uses to route observations and subdivide
/// intervals.
///
/// # Contract
///
/// ## Seven structural properties (§IDEA M-3.1)
///
/// The method contracts encode seven properties required by the
/// spec: dyadic closure, width–depth coherence, total ordering,
/// domain spanning, midpoint determinism, non-negative domain,
/// and finality.
///
/// ## Integer vs floating-point domains (§IDEA M-3.2)
///
/// | Property | Integer | Floating-point |
/// |----------|---------|----------------|
/// | Domain maximum | `2^N` (or `MAX` at full bit-width) | `2^N` as a float |
/// | Midpoint | `a + (b − a) / 2` (integer division) | `a + (b − a) / 2` (exact for dyadic widths) |
/// | Finality | Natural: `width == 1` (unit cell) | Artificial: `depth >= N` |
/// | Successor | `self + 1` | **Panics** — not meaningful in the dyadic context (§IDEA M-3.2.6) |
/// | NaN | Impossible | Rejected at every entry point (§IDEA M-3.2.4) |
/// | Total ordering | Native | Via `total_cmp` (IEEE 754 totalOrder) |
///
/// # Implementations
///
/// Provided for `u8`, `u16`, `u32`, `u64`, `u128`, `f32`, `f64`.
/// Choose a concrete type — typically `u64` for discrete domains or
/// `f64` for continuous ones — and all capabilities are available
/// immediately.
///
/// # Examples
///
/// ```
/// use torrust_mudlark::Coordinate;
///
/// fn interval_width<C: Coordinate>(start: C, end: C) -> C {
///     C::width(start, end)
/// }
///
/// // u64 coordinate: domain [0, 256) with N = 8.
/// let w = interval_width(10u64, 42u64);
/// assert_eq!(w, 32);
///
/// // Midpoint arithmetic.
/// let mid = u64::midpoint(0, 256);
/// assert_eq!(mid, 128);
///
/// // Domain bounds.
/// assert_eq!(u64::zero(), 0);
/// assert_eq!(u64::domain_max(8), 256);
/// ```
///
/// In context — `Coordinate` is the spatial position type for
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
/// // u64 as coordinate — discrete domain [0, 2^8).
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// g.observe(42, 10u64);
/// let cell = g.get(42);
/// assert!(cell.start <= 42 && 42 < cell.end);
/// ```
///
/// # References
///
/// - [ADR-M-032] — Three-surface visibility model
///
/// [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md
pub trait Coordinate: Copy + PartialOrd + Debug + Default + Send + Sync + 'static {
    /// Bit-width of this coordinate type (e.g. 64 for `u64`).
    /// Used for compile-time validation: `N <= C::BITS`.
    const BITS: u32;

    /// The zero value (domain origin).
    fn zero() -> Self;

    /// The exclusive upper bound of the domain: `2^n` in coordinate
    /// space.
    ///
    /// For integer types where `n == Self::BITS`, returns `Self::MAX`.
    fn domain_max(n: u32) -> Self;

    /// Dyadic midpoint of the interval `[a, b)`.
    ///
    /// Uses the overflow-safe formula `a + (b − a) / 2` (§IDEA M-4.2.3).
    /// For dyadic power-of-two intervals within the IEEE
    /// 754 normal range, this is exact for floats (division by 2 is
    /// an exponent decrement, no rounding).
    fn midpoint(a: Self, b: Self) -> Self;

    /// Width of the interval `[start, end)`.
    fn width(start: Self, end: Self) -> Self;

    /// Whether the interval `[start, end)` is indivisible.
    ///
    /// For integers: `width == 1` (the unit cell — subdivision
    /// terminates naturally).
    /// For floats: `depth >= n` (subdivision terminates at the
    /// configured maximum resolution; see §IDEA M-3.2.1).
    fn is_final(start: Self, end: Self, depth: u32, n: u32) -> bool;

    /// Convert from a `u64` value.
    ///
    /// Used by plan builders and test harnesses to generate coordinate
    /// sequences from integer counters.  The conversion may be lossy
    /// for types narrower than `u64` (truncation) or for floats
    /// (rounding).
    fn from_u64(v: u64) -> Self;

    /// The next representable value after `self`.
    ///
    /// For integers: `self + 1` (panics on overflow).
    /// For floats: **panics** — there is no "next dyadic boundary"
    /// in continuous domains (§IDEA M-3.2.6).  Float range queries
    /// should use `Included`/`Excluded` bounds directly.
    ///
    /// Used by `range_sum` bound resolution to convert `Excluded`
    /// start bounds and `Included` end bounds to half-open form.
    #[must_use]
    fn next_value(self) -> Self;

    /// Convert to `f64` for width ratio arithmetic in range queries.
    ///
    /// Integers may lose precision for values > 2^53.
    fn to_f64(self) -> f64;

    /// Whether this value is NaN.
    ///
    /// Always `false` for integer types.  NaN coordinates are
    /// rejected at every entry point — observation, point query,
    /// range query — because NaN violates total ordering
    /// (§IDEA M-3.2.4).
    fn is_nan(self) -> bool;

    /// Total ordering for use as a `BTreeMap` key.
    ///
    /// Integers: delegates to `Ord::cmp` (zero overhead).
    /// Floats: uses IEEE 754 `totalOrder` via `f{32,64}::total_cmp`
    /// (stable since Rust 1.62).  NaN sorts deterministically
    /// (after +∞), though NaN values never appear as map keys in
    /// practice because they are rejected at entry points
    /// (§IDEA M-3.2.7).
    fn total_cmp(&self, other: &Self) -> std::cmp::Ordering;
}

// ── Integer implementations ─────────────────────────────────────────

macro_rules! impl_coordinate_uint {
    ($($ty:ty),+) => {$(
        impl Coordinate for $ty {
            const BITS: u32 = <$ty>::BITS;

            #[inline]
            fn zero() -> Self { 0 }

            #[inline]
            fn domain_max(n: u32) -> Self {
                if n == Self::BITS {
                    <$ty>::MAX
                } else {
                    1 << n
                }
            }

            #[inline]
            fn midpoint(a: Self, b: Self) -> Self {
                a + (b - a) / 2
            }

            #[inline]
            fn width(start: Self, end: Self) -> Self {
                end - start
            }

            #[inline]
            fn is_final(start: Self, end: Self, _depth: u32, _n: u32) -> bool {
                end - start == 1
            }

            #[inline]
            #[allow(clippy::cast_possible_truncation, clippy::cast_lossless)]
            fn from_u64(v: u64) -> Self {
                v as Self
            }

            #[inline]
            fn next_value(self) -> Self {
                self + 1
            }

            #[inline]
            #[allow(clippy::cast_precision_loss, clippy::cast_lossless)]
            fn to_f64(self) -> f64 {
                self as f64
            }

            #[inline]
            fn is_nan(self) -> bool {
                false
            }

            #[inline]
            fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
                Ord::cmp(self, other)
            }
        }
    )+};
}

impl_coordinate_uint!(u8, u16, u32, u64, u128);

// ── Floating-point implementations ──────────────────────────────────

impl Coordinate for f32 {
    const BITS: u32 = 32;

    #[inline]
    fn zero() -> Self {
        0.0
    }

    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    fn domain_max(n: u32) -> Self {
        2.0_f32.powi(n as i32)
    }

    #[inline]
    fn midpoint(a: Self, b: Self) -> Self {
        a + (b - a) / 2.0
    }

    #[inline]
    fn width(start: Self, end: Self) -> Self {
        end - start
    }

    #[inline]
    fn is_final(_start: Self, _end: Self, depth: u32, n: u32) -> bool {
        depth >= n
    }

    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn from_u64(v: u64) -> Self {
        v as Self
    }

    #[inline]
    fn next_value(self) -> Self {
        panic!("next_value is not supported for f32 coordinates; use Excluded/Included bounds directly")
    }

    #[inline]
    fn to_f64(self) -> f64 {
        f64::from(self)
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    #[inline]
    fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        Self::total_cmp(self, other)
    }
}

impl Coordinate for f64 {
    const BITS: u32 = 64;

    #[inline]
    fn zero() -> Self {
        0.0
    }

    #[inline]
    #[allow(clippy::cast_possible_wrap)]
    fn domain_max(n: u32) -> Self {
        2.0_f64.powi(n as i32)
    }

    #[inline]
    fn midpoint(a: Self, b: Self) -> Self {
        a + (b - a) / 2.0
    }

    #[inline]
    fn width(start: Self, end: Self) -> Self {
        end - start
    }

    #[inline]
    fn is_final(_start: Self, _end: Self, depth: u32, n: u32) -> bool {
        depth >= n
    }

    #[inline]
    #[allow(clippy::cast_precision_loss)]
    fn from_u64(v: u64) -> Self {
        v as Self
    }

    #[inline]
    fn next_value(self) -> Self {
        panic!("next_value is not supported for f64 coordinates; use Excluded/Included bounds directly")
    }

    #[inline]
    fn to_f64(self) -> f64 {
        self
    }

    #[inline]
    fn is_nan(self) -> bool {
        self.is_nan()
    }

    #[inline]
    fn total_cmp(&self, other: &Self) -> std::cmp::Ordering {
        Self::total_cmp(self, other)
    }
}
