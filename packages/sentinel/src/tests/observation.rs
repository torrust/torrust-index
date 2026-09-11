// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for [`crate::observation`] — the boundary where a coordinate value
//! becomes the vector the subspace engine works on.
//!
//! The conversion is fixed by two decisions. Bits are centred rather than
//! taken raw — a set bit is plus a half and a clear bit minus a half — so
//! that the data arrives at the tracker with zero mean per dimension, which
//! is what the subspace update assumes. And bits are stored most-significant
//! first, so that a cell's G-tree depth is a prefix length: the leading bits
//! routing has already resolved sit at the front, and the suffix a tracker
//! analyses is what remains behind them.
//!
//! Those two decisions together give the representation its one arithmetic
//! regularity. Every centred bit has magnitude one half whatever the value,
//! so a suffix of any width has squared norm equal to a quarter of that
//! width, at every depth and for every input. The tracker can therefore read
//! a residual as a departure from structure rather than as an artefact of
//! how large the observation happened to be.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`from_u128_zero_all_minus_half`] | bits | A bit is not carried into the model as zero or one but as minus or plus a half. A value with no bits set therefore becomes a vector of minus a half throughout, which is the centring the subspace tracker depends on: the representation has no mean to remove before the geometry means anything. |
//! | [`from_u128_max_all_plus_half`] | bits | cites (´claim:bits:a-clear-bit-becomes-minus-a-half-and-a-set-bit-plus-a-half´) |
//! | [`from_u128_one_only_lsb_set`] | bits | cites (´claim:bits:the-most-significant-bit-stands-at-index-zero´) |
//! | [`from_u128_msb_first_ordering`] | bits | Bits are stored most significant first: a value carrying only its top bit puts that bit at index zero and nothing else anywhere. This ordering is what lets a cell at depth `d` take its working observation by skipping the first `d` entries, because those are exactly the bits routing fixed. |
//! | [`u128_custom_width_populates_n_bits`] | bits | The vector is backed by a fixed hundred-and-twenty-eight-slot array, but the requested width is what counts as populated: asking for eight bits fills eight slots and leaves the remainder at zero, and the observation reports its length as eight rather than as the array's size. A domain narrower than the backing store is therefore not padded with fabricated structure. |
//! | [`u128_zero_width_gives_empty`] | bits | cites (´claim:bits:a-requested-width-populates-exactly-that-many-slots-and-leaves-the-rest-zero´) |
//! | [`u128_width_capped_at_128`] | bits | cites (´claim:bits:a-width-wider-than-the-coordinate-is-capped-at-the-coordinates-own-width´) |
//! | [`u64_max_all_plus_half`] | bits | The centring rule is a property of the conversion, not of the coordinate type: a sixty-four-bit value with every bit set produces sixty-four slots of plus a half, exactly as the wider coordinate does. A host working in a narrower domain gets the same representation, so the engine above the boundary need not know which width it was fed. |
//! | [`u64_width_capped_at_64`] | bits | Asking a sixty-four-bit coordinate for a wider observation does not invent bits: the width is capped at what the value actually holds. A sentinel configured for the wider domain can therefore be handed narrower coordinates without the shift going out of range or the tail of the vector filling with structure that was never observed. |
//! | [`from_coord_delegates_correctly`] | bits | The generic entry point the engine actually calls produces the same observation, bit for bit and length for length, as calling the conversion on the value directly. There is one encoding rather than two that happen to agree, so nothing can drift between the path tests exercise and the path production code takes. |
//! | [`suffix_zero_is_full_vector`] | bits | A cell at the root of the G-tree has had no bits resolved by routing, so its working observation is the whole vector — not a copy of it, but the same values. The root tracker analyses the full domain width, which is the base case the depth arithmetic has to agree with. |
//! | [`suffix_intermediate_depth`] | bits | At depth `d` the observation drops exactly its first `d` entries and keeps everything behind them. Those leading bits are constant across every value routed into the cell, so removing them leaves precisely the part that varies — the tracker's width is the domain width less its depth, and the bits it sees are the tail of the same vector rather than a re-derivation. |
//! | [`suffix_at_len_is_empty`] | bits | A cell as deep as its domain is wide has nothing left to analyse: routing has resolved every bit, and the suffix is empty rather than an error. This holds at the full width and at a narrower configured one alike, which is why such cells are excluded from the analysis set by width rather than caught as a failure when a tracker tries to run on them. |
//! | [`suffix_panics_beyond_len`] | bits | A depth past the observation's own width is treated as a programming error and not as a value to be tolerated. Empty is the answer at exactly the width; beyond it there is no honest answer, so the boundary between the degenerate case and the impossible one is drawn rather than blurred by silently clamping. |
//! | [`suffix_norm_squared_is_width_over_four`] | bits | Because every centred bit has magnitude one half, a suffix's squared norm is a quarter of its width and nothing else — the same for every value and at every depth, checked here across saturated, sparse and arbitrary inputs at all depths. Observation magnitude therefore carries no information: a residual the tracker measures is a departure from learned structure, never an artefact of which value arrived. |

use crate::observation::*;

// ─── CentredBits::from_u128 ────────────────────────────────

/// A bit is not carried into the model as zero or one but as minus or plus a
/// half. A value with no bits set therefore becomes a vector of minus a half
/// throughout, which is the centring the subspace tracker depends on: the
/// representation has no mean to remove before the geometry means anything.
///
/// ´claim:bits:a-clear-bit-becomes-minus-a-half-and-a-set-bit-plus-a-half´
/// ´test:crate:from-u128-zero-all-minus-half´
#[test]
fn from_u128_zero_all_minus_half() {
    let cb = CentredBits::from_u128(0);
    for (i, &b) in cb.bits.iter().enumerate() {
        assert!((b - (-0.5)).abs() < f64::EPSILON, "bit {i}: expected -0.5, got {b}");
    }
}

/// The opposite extreme of the same rule: a value with every bit set becomes
/// a vector of plus a half throughout. The two saturated inputs bracket the
/// encoding, so no bit pattern can produce a magnitude other than a half.
///
/// (´claim:bits:a-clear-bit-becomes-minus-a-half-and-a-set-bit-plus-a-half´)
/// ´test:crate:from-u128-max-all-plus-half´
#[test]
fn from_u128_max_all_plus_half() {
    let cb = CentredBits::from_u128(u128::MAX);
    for (i, &b) in cb.bits.iter().enumerate() {
        assert!((b - 0.5).abs() < f64::EPSILON, "bit {i}: expected +0.5, got {b}");
    }
}

/// The value one has only its least significant bit set, and that bit shows
/// up at the very last index rather than the first. Which end of the vector a
/// bit lands on is what makes a G-tree depth readable as a prefix length.
///
/// (´claim:bits:the-most-significant-bit-stands-at-index-zero´)
/// ´test:crate:from-u128-one-only-lsb-set´
#[test]
fn from_u128_one_only_lsb_set() {
    let cb = CentredBits::from_u128(1);
    for &b in &cb.bits[..127] {
        assert!((b - (-0.5)).abs() < f64::EPSILON, "expected -0.5, got {b}");
    }
    assert!((cb.bits[127] - 0.5).abs() < f64::EPSILON);
}

/// Bits are stored most significant first: a value carrying only its top bit
/// puts that bit at index zero and nothing else anywhere. This ordering is
/// what lets a cell at depth `d` take its working observation by skipping the
/// first `d` entries, because those are exactly the bits routing fixed.
///
/// ´claim:bits:the-most-significant-bit-stands-at-index-zero´
/// ´test:crate:from-u128-msb-first-ordering´
#[test]
fn from_u128_msb_first_ordering() {
    // 0x80…0 has only the MSB set — index 0 should be +0.5.
    let cb = CentredBits::from_u128(1_u128 << 127);
    assert!((cb.bits[0] - 0.5).abs() < f64::EPSILON, "MSB should be at index 0");
    for &b in &cb.bits[1..] {
        assert!((b - (-0.5)).abs() < f64::EPSILON, "remaining bits should be -0.5");
    }
}

// ─── CentredBitSource for u128 — custom width ──────────────

/// The vector is backed by a fixed hundred-and-twenty-eight-slot array, but
/// the requested width is what counts as populated: asking for eight bits
/// fills eight slots and leaves the remainder at zero, and the observation
/// reports its length as eight rather than as the array's size. A domain
/// narrower than the backing store is therefore not padded with fabricated
/// structure.
///
/// ´claim:bits:a-requested-width-populates-exactly-that-many-slots-and-leaves-the-rest-zero´
/// ´test:crate:u128-custom-width-populates-n-bits´
#[test]
fn u128_custom_width_populates_n_bits() {
    // 0xFF = 8 one-bits; ask for n=8 → first 8 slots are +0.5, rest 0.0.
    let cb = 0xFF_u128.to_centred_bits(8);
    for (i, &b) in cb.bits[..8].iter().enumerate() {
        assert!((b - 0.5).abs() < f64::EPSILON, "bit {i}: expected +0.5, got {b}");
    }
    for (i, &b) in cb.bits[8..].iter().enumerate() {
        assert!(b.abs() < f64::EPSILON, "bit {}: expected 0.0, got {b}", i + 8);
    }
    // suffix(0) should return exactly 8 elements, not 128.
    assert_eq!(cb.suffix(0).len(), 8);
}

/// The degenerate end of the same rule: a width of zero populates nothing, so
/// the observation is empty and every slot stays at zero however large the
/// value handed in. A zero-width domain is admitted rather than rejected, and
/// it carries no bits of the value it came from.
///
/// (´claim:bits:a-requested-width-populates-exactly-that-many-slots-and-leaves-the-rest-zero´)
/// ´test:crate:u128-zero-width-gives-empty´
#[test]
fn u128_zero_width_gives_empty() {
    let cb = 42_u128.to_centred_bits(0);
    assert_eq!(cb.suffix(0).len(), 0);
    for &b in &cb.bits {
        assert!(b.abs() < f64::EPSILON, "all slots should be 0.0");
    }
}

/// The cap is a property of the conversion rather than of the narrower
/// coordinate: asking the wider type for more bits than it holds returns its
/// own width, exactly as the narrower type does. The vector is backed by an
/// array of that same width, so an uncapped request walks off the end of it —
/// a host computing its width from a configured domain would get a panic out
/// of the observation boundary instead of an observation.
///
/// (´claim:bits:a-width-wider-than-the-coordinate-is-capped-at-the-coordinates-own-width´)
/// ´test:crate:u128-width-capped-at-128´
#[test]
fn u128_width_capped_at_128() {
    // Requesting n=129 for a u128 should cap at 128.
    let cb = u128::MAX.to_centred_bits(129);
    assert_eq!(cb.suffix(0).len(), 128);
    for (i, &b) in cb.bits.iter().enumerate() {
        assert!((b - 0.5).abs() < f64::EPSILON, "bit {i}: expected +0.5, got {b}");
    }
}

// ─── CentredBitSource for u64 ──────────────────────────────

/// The centring rule is a property of the conversion, not of the coordinate
/// type: a sixty-four-bit value with every bit set produces sixty-four slots
/// of plus a half, exactly as the wider coordinate does. A host working in a
/// narrower domain gets the same representation, so the engine above the
/// boundary need not know which width it was fed.
///
/// ´claim:bits:a-narrower-coordinate-encodes-by-the-same-rule-as-a-wider-one´
/// ´test:crate:u64-max-all-plus-half´
#[test]
fn u64_max_all_plus_half() {
    let cb = u64::MAX.to_centred_bits(64);
    assert_eq!(cb.suffix(0).len(), 64);
    for (i, &b) in cb.bits[..64].iter().enumerate() {
        assert!((b - 0.5).abs() < f64::EPSILON, "bit {i}: expected +0.5, got {b}");
    }
}

/// Asking a sixty-four-bit coordinate for a wider observation does not
/// invent bits: the width is capped at what the value actually holds. A
/// sentinel configured for the wider domain can therefore be handed narrower
/// coordinates without the shift going out of range or the tail of the
/// vector filling with structure that was never observed.
///
/// ´claim:bits:a-width-wider-than-the-coordinate-is-capped-at-the-coordinates-own-width´
/// ´test:crate:u64-width-capped-at-64´
#[test]
fn u64_width_capped_at_64() {
    // Requesting n=128 for a u64 should cap at 64.
    let cb = u64::MAX.to_centred_bits(128);
    assert_eq!(cb.suffix(0).len(), 64);
}

// ─── CentredBits::from_coord ───────────────────────────────

/// The generic entry point the engine actually calls produces the same
/// observation, bit for bit and length for length, as calling the conversion
/// on the value directly. There is one encoding rather than two that happen
/// to agree, so nothing can drift between the path tests exercise and the
/// path production code takes.
///
/// ´claim:bits:the-generic-entry-point-produces-what-the-source-conversion-produces´
/// ´test:crate:from-coord-delegates-correctly´
#[test]
fn from_coord_delegates_correctly() {
    let value = 0xDEAD_BEEF_u128;
    let direct = value.to_centred_bits(128);
    let via_coord = CentredBits::from_coord(&value, 128);
    for (a, b) in direct.bits.iter().zip(&via_coord.bits) {
        assert!((a - b).abs() < f64::EPSILON);
    }
    assert_eq!(direct.suffix(0).len(), via_coord.suffix(0).len());
}

// ─── CentredBits::suffix ───────────────────────────────────

/// A cell at the root of the G-tree has had no bits resolved by routing, so
/// its working observation is the whole vector — not a copy of it, but the
/// same values. The root tracker analyses the full domain width, which is the
/// base case the depth arithmetic has to agree with.
///
/// ´claim:bits:the-suffix-at-depth-zero-is-the-whole-observation´
/// ´test:crate:suffix-zero-is-full-vector´
#[test]
fn suffix_zero_is_full_vector() {
    let cb = CentredBits::from_u128(0xDEAD_BEEF);
    let s = cb.suffix(0);
    assert_eq!(s.len(), 128);
    assert_eq!(s, &cb.bits[..]);
}

/// At depth `d` the observation drops exactly its first `d` entries and keeps
/// everything behind them. Those leading bits are constant across every value
/// routed into the cell, so removing them leaves precisely the part that
/// varies — the tracker's width is the domain width less its depth, and the
/// bits it sees are the tail of the same vector rather than a re-derivation.
///
/// ´claim:bits:a-suffix-drops-exactly-the-bits-routing-already-resolved´
/// ´test:crate:suffix-intermediate-depth´
#[test]
fn suffix_intermediate_depth() {
    let cb = CentredBits::from_u128(1);
    let s = cb.suffix(8);
    assert_eq!(s.len(), 120);
    assert_eq!(s, &cb.bits[8..128]);
}

/// A cell as deep as its domain is wide has nothing left to analyse: routing
/// has resolved every bit, and the suffix is empty rather than an error. This
/// holds at the full width and at a narrower configured one alike, which is
/// why such cells are excluded from the analysis set by width rather than
/// caught as a failure when a tracker tries to run on them.
///
/// ´claim:bits:a-cell-as-deep-as-its-domain-is-wide-has-an-empty-suffix´
/// ´test:crate:suffix-at-len-is-empty´
#[test]
fn suffix_at_len_is_empty() {
    let cb = CentredBits::from_u128(42);
    assert_eq!(cb.suffix(128).len(), 0);

    // Also verify for a smaller width.
    let cb_small = 0xFF_u128.to_centred_bits(16);
    assert_eq!(cb_small.suffix(16).len(), 0);
}

/// A depth past the observation's own width is treated as a programming
/// error and not as a value to be tolerated. Empty is the answer at exactly
/// the width; beyond it there is no honest answer, so the boundary between
/// the degenerate case and the impossible one is drawn rather than blurred by
/// silently clamping.
///
/// ´claim:bits:a-depth-past-the-observations-width-is-a-fault-not-a-value´
/// ´test:crate:suffix-panics-beyond-len´
#[test]
#[should_panic(expected = "slice index starts at 17 but ends at 16")]
fn suffix_panics_beyond_len() {
    let cb = 0xFF_u128.to_centred_bits(16);
    let _ = cb.suffix(17); // depth > len → panic
}

/// Because every centred bit has magnitude one half, a suffix's squared norm
/// is a quarter of its width and nothing else — the same for every value and
/// at every depth, checked here across saturated, sparse and arbitrary inputs
/// at all depths. Observation magnitude therefore carries no information: a
/// residual the tracker measures is a departure from learned structure, never
/// an artefact of which value arrived.
///
/// ´claim:bits:every-suffix-has-squared-norm-equal-to-a-quarter-of-its-width´
/// ´test:crate:suffix-norm-squared-is-width-over-four´
#[test]
fn suffix_norm_squared_is_width_over_four() {
    let values: [u128; 5] = [0, 1, u128::MAX, 0xDEAD_BEEF, 0x1234_5678_9ABC_DEF0];

    for &v in &values {
        let cb = CentredBits::from_u128(v);
        for d in 0..128_u8 {
            let s = cb.suffix(d);
            let w = s.len();
            let norm_sq: f64 = s.iter().map(|x| x * x).sum();
            #[allow(clippy::cast_precision_loss)] // width ≤ 128, well within f64 precision
            let expected = w as f64 / 4.0;
            assert!(
                (norm_sq - expected).abs() < 1e-10,
                "v={v:#x}, depth={d}: suffix norm²={norm_sq}, expected w/4={expected}"
            );
        }
    }
}
