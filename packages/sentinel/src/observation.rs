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

// ─── CentredBitSource trait (§ADR-S-018 Part A) ──────────────

/// Bridge trait: convert a coordinate value into centred bit form.
///
/// Implemented in this crate for `u128` and `u64`, and open to a downstream
/// coordinate type that implements it as well: the trait is part of the
/// published surface, so nothing closes the set of implementations. An
/// implementor supplies `to_centred_bits` for its own width, and the two impls
/// here are what a wrapper around one of those widths delegates to.
pub trait CentredBitSource: Coordinate {
    /// Convert `self` into a centred bit vector of length `n`.
    fn to_centred_bits(&self, n: u32) -> CentredBits;
}

impl CentredBitSource for u128 {
    #[allow(clippy::cast_possible_truncation)] // i < 128, fits in u32
    fn to_centred_bits(&self, n: u32) -> CentredBits {
        let mut bits = [0.0_f64; 128];
        let len = n as usize;
        for (i, slot) in bits[..len].iter_mut().enumerate() {
            *slot = if (self >> (n - 1 - i as u32)) & 1 == 1 { 0.5 } else { -0.5 };
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
#[derive(Debug, Clone)]
pub struct CentredBits {
    /// The centred bit values, from MSB (index 0) to LSB (index `len - 1`).
    /// Only indices `[0, len)` are meaningful; the rest are zero-filled.
    pub bits: [f64; 128],

    /// Runtime length (= N for the sentinel's domain width).
    len: usize,
}

impl CentredBits {
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
