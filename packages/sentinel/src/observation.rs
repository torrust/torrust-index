// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Observation boundary: converting raw coordinate values into the
//! mathematical representation the subspace engine needs.
//!
//! This module is the anti-corruption layer between the host's domain
//! (positionally structured coordinate values) and the linear algebra
//! world (`Mat<f64>`, centred bit vectors).
//!
//! The input values must have hierarchical positional structure —
//! leading bits define coarse groupings and successive bits refine
//! them (e.g. IPv6 addresses). The host is responsible for ensuring
//! this property before handing values to the sentinel.
//!
//! Nothing in this module touches `faer`. It produces plain `Vec<f64>`
//! data that the subspace module consumes.

use torrust_mudlark::Coordinate;

// ─── CentredBitSource trait ─────────────────────────────────

/// Bridge trait: convert a coordinate value into centred bit form.
///
/// Implemented in this crate for `u128` and `u64`, and open to a downstream
/// coordinate type that implements it as well: the trait is part of the
/// published surface, so nothing closes the set of implementations. An
/// implementor supplies `to_centred_bits` for its own width, and the two impls
/// here are what a wrapper around one of those widths delegates to. This is
/// the coordinate side of the generic parameters the sentinel is built on
/// (´rec:sentinel:generic-coordinate-accumulator-and-domain-width´).
///
/// The set of implementations is open; the width they can serve is not. Every
/// conversion returns a [`CentredBits`], which carries its values in a fixed
/// array of 128 slots, so a coordinate type wider than that has no vector to
/// return past the first 128 bits. An implementor for such a type is free to
/// exist — what it cannot do is drive a sentinel wider than the vector:
/// `SpectralSentinel::new` refuses a width above the ceiling rather than
/// building trackers whose extra dimensions would be fed a constant the
/// coordinate stream never produced.
pub trait CentredBitSource: Coordinate {
    /// Convert `self` into a centred bit vector of length `n`.
    ///
    /// `n` is a request, not a promise: the effective width is `n` capped at
    /// the width the implementing type actually holds, which is also the
    /// width of the vector that comes back. The implementations here cap at
    /// 128 and 64 respectively, and an implementation for another coordinate
    /// type caps at its own. The cap is not a courtesy — the returned vector
    /// is backed by a fixed hundred-and-twenty-eight-slot array, and a width
    /// beyond the type's own would either read bits that do not exist or
    /// index past that array — so an implementation applies it rather than
    /// trusting the caller, and no caller can provoke a panic by asking for
    /// more than the domain holds.
    fn to_centred_bits(&self, n: u32) -> CentredBits;
}

impl CentredBitSource for u128 {
    #[allow(clippy::cast_possible_truncation)] // i < 128, fits in u32
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        let mut bits = [0.0_f64; 128];
        let width = n.min(128);
        let len = width as usize;
        for (i, slot) in bits[..len].iter_mut().enumerate() {
            *slot = if (self >> (width - 1 - i as u32)) & 1 == 1 {
                0.5
            } else {
                -0.5
            };
        }
        CentredBits { bits, len }
    }
}

impl CentredBitSource for u64 {
    #[allow(clippy::cast_possible_truncation)] // i < 64, fits in u32
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        let mut bits = [0.0_f64; 128];
        let len = n.min(64) as usize;
        for (i, slot) in bits[..len].iter_mut().enumerate() {
            *slot = if (self >> (n.min(64) - 1 - i as u32)) & 1 == 1 {
                0.5
            } else {
                -0.5
            };
        }
        CentredBits { bits, len }
    }
}

// ─── CentredBits ────────────────────────────────────────────

/// A coordinate value converted to a centred bit vector.
///
/// Each of the `len` bits becomes:
/// - bit `1` → `+0.5`
/// - bit `0` → `−0.5`
///
/// This centring is critical: it ensures the data has zero mean
/// per dimension, which the subspace tracker requires.
///
/// The backing array is fixed at 128 slots, which is the sentinel's coordinate
/// width ceiling and not an implementation detail a wider coordinate type can
/// work around: a centred bit is `±0.5` and never zero, so the slots past
/// `len` are distinguishable from data and there is no honest way to present
/// them as observations. A sentinel is refused at construction above that
/// width for the same reason.
#[derive(Debug, Clone)]
pub struct CentredBits {
    /// The centred bit values, from MSB (index 0) to LSB (index `len - 1`).
    /// Only indices `[0, len)` are meaningful; the rest are zero-filled.
    pub bits: [f64; 128],

    /// Runtime length (= N for the sentinel's domain width).
    len: usize,
}

impl CentredBits {
    /// Build a vector from centred bit values already computed, with `len`
    /// of them meaningful.
    ///
    /// This is how an implementation of [`CentredBitSource`] outside this
    /// crate returns its conversion. The two implementations here work on
    /// coordinate types whose bits are already there to be shifted out, and a
    /// wrapper around one of those widths delegates to them; a coordinate
    /// type whose centred form has to be computed has nothing to delegate to,
    /// and this is the constructor it uses. Slots from `len` onward are the
    /// caller's to leave at zero — [`suffix`](Self::suffix) never reads them.
    ///
    /// # Panics
    ///
    /// Panics if `len` exceeds 128, the fixed size of the backing array. A
    /// length past the array is a mistake in the implementation rather than a
    /// value a host could supply, which is the same reading
    /// [`suffix`](Self::suffix) takes of a depth past the width: there is no
    /// honest vector to return, and clamping would hand back an observation
    /// narrower than the one the caller believes it built.
    #[must_use]
    pub const fn new(bits: [f64; 128], len: usize) -> Self {
        assert!(
            len <= crate::MAX_TRACKER_DIM,
            "centred bit length exceeds the 128-slot backing array"
        );
        Self { bits, len }
    }

    /// How many of the backing array's slots carry a centred bit.
    #[must_use]
    pub const fn len(&self) -> usize {
        self.len
    }

    /// Whether the vector carries no bits at all — the zero-width domain.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Convert a `u128` value to centred bits (128-bit, convenience wrapper).
    #[must_use]
    pub fn from_u128(value: u128) -> Self {
        value.to_centred_bits(128)
    }

    /// Construct from any coordinate type implementing `CentredBitSource`.
    #[must_use]
    pub(crate) fn from_coord<C: CentredBitSource>(value: &C, n: u32) -> Self {
        value.to_centred_bits(n)
    }

    /// Return a slice of the suffix bits from position `depth` to `len`.
    ///
    /// For a cell at G-tree depth `d`, the first `d` bits are resolved
    /// by routing (constant within the cell). The suffix `[d, len)` is
    /// the working observation — the bits that vary and carry
    /// statistical content (§ALGO S-3.2).
    ///
    /// Width: `len - depth`. At depth 0, the suffix is the entire
    /// bit vector. At depth `len`, the suffix is empty (zero-width
    /// cell — degenerate).
    ///
    /// # Panics
    ///
    /// Panics if `depth` exceeds `len`.
    #[must_use]
    pub fn suffix(&self, depth: u8) -> &[f64] {
        &self.bits[usize::from(depth)..self.len]
    }
}
