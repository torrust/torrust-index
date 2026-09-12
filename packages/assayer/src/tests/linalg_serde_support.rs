// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`col_serde_roundtrip`] | linalg | A column vector written to bytes and read back is bit-identical, compared through `to_bits()` rather than through a tolerance. Resuming from a checkpoint therefore continues the same computation rather than a numerically similar one, which matters because the model's own invariants are stated bitwise and a rounded reload would violate them on arrival. |
//! | [`mat_serde_roundtrip`] | linalg | cites (´claim:linalg:a-serialisation-round-trip-restores-shape-and-every-value-bit-identically´) |
//! | [`symmetric_roundtrip`] | linalg | cites (´claim:linalg:a-serialisation-round-trip-restores-shape-and-every-value-bit-identically´) |
//! | [`symmetric_rejects_corrupt`] | linalg | Deserialisation is a validating boundary, not a cast: a payload of the right shape whose entries no longer mirror each other — here one off-diagonal overwritten so it disagrees with its transpose — comes back as an error rather than as a matrix. Damaged storage is caught where it enters, instead of surfacing much later as a symmetry audit failure deep inside an update with no trace of where the corruption came from. |

//! Crate-level tests for `linalg::serde_support`.
//!
//! A checkpoint is only useful if resuming from it lands the model in
//! exactly the state it was saved from, so the bar here is bit-identity
//! rather than agreement to a tolerance. The other half of the story is what
//! happens when the bytes are *not* trustworthy: a symmetric matrix
//! reconstructs through `from_storage`, so damage that breaks symmetry is
//! caught at the boundary instead of entering the model.

#![allow(clippy::float_cmp, clippy::cast_precision_loss, clippy::suboptimal_flops)]

use faer::{Col, Mat};

use crate::linalg::convert::{col_to_vec, mat_to_vec, vec_to_col, vec_to_mat};
use crate::linalg::symmetric::SymmetricMatrix;
use crate::testing::TestRng;

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a random SPD matrix via `AᵀA` + εI using the shared
/// deterministic [`TestRng`] from the integration-test harness.
fn random_spd(p: usize, seed: u64) -> SymmetricMatrix {
    let mut rng = TestRng::new(seed);
    let r = Mat::from_fn(p, p, |_, _| rng.next_f64() * 2.0 - 1.0);
    let mut a = r.transpose() * &r;
    for i in 0..p {
        a[(i, i)] += 0.1;
    }
    SymmetricMatrix::from_computation(a)
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// A column vector written to bytes and read back is bit-identical, compared
/// through `to_bits()` rather than through a tolerance. Resuming from a
/// checkpoint therefore continues the same computation rather than a
/// numerically similar one, which matters because the model's own invariants
/// are stated bitwise and a rounded reload would violate them on arrival.
///
/// ´claim:linalg:a-serialisation-round-trip-restores-shape-and-every-value-bit-identically´
/// ´test:crate:col-serde-roundtrip´
#[test]
fn col_serde_roundtrip() {
    let col = Col::from_fn(5, |i| (i as f64) * 1.1 + 0.7);
    let bytes = crate::serde_codec::serialize(&col_to_vec(&col)).unwrap();
    let data: Vec<f64> = crate::serde_codec::deserialize(&bytes).unwrap();
    let restored = vec_to_col(&data);
    assert_eq!(col.nrows(), restored.nrows());
    for i in 0..5 {
        assert_eq!(col[i].to_bits(), restored[i].to_bits());
    }
}

/// A rectangular matrix survives the same journey: both dimensions and every
/// entry return exactly, so the column-major ordering agreed on by the writer
/// is the one the reader assumes. A disagreement there would transpose a
/// non-square matrix rather than announce itself.
///
/// (´claim:linalg:a-serialisation-round-trip-restores-shape-and-every-value-bit-identically´)
/// ´test:crate:mat-serde-roundtrip´
#[test]
fn mat_serde_roundtrip() {
    let mat = Mat::from_fn(3, 4, |i, j| (i * 10 + j) as f64);
    let bytes = crate::serde_codec::serialize(&mat_to_vec(&mat)).unwrap();
    let data: Vec<f64> = crate::serde_codec::deserialize(&bytes).unwrap();
    let restored = vec_to_mat(&data, 3, 4);
    assert_eq!(mat.nrows(), restored.nrows());
    assert_eq!(mat.ncols(), restored.ncols());
    for j in 0..4 {
        for i in 0..3 {
            assert_eq!(mat[(i, j)].to_bits(), restored[(i, j)].to_bits());
        }
    }
}

/// A symmetric matrix reloads bit-identically too, dimension and all entries
/// alike. This is the one that has to hold for a checkpoint to be worth
/// taking: the reload path runs the reconstructed matrix back through the
/// symmetry validation, so a round trip that merely came close would be
/// rejected at its own boundary rather than accepted with drift.
///
/// (´claim:linalg:a-serialisation-round-trip-restores-shape-and-every-value-bit-identically´)
/// ´test:crate:symmetric-roundtrip´
#[test]
fn symmetric_roundtrip() {
    let m = random_spd(10, 42);
    let bytes = crate::serde_codec::serialize(&m).unwrap();
    let restored: SymmetricMatrix = crate::serde_codec::deserialize(&bytes).unwrap();
    assert_eq!(m.dim(), restored.dim());
    for j in 0..10 {
        for i in 0..10 {
            assert_eq!(
                m.as_inner()[(i, j)].to_bits(),
                restored.as_inner()[(i, j)].to_bits(),
                "mismatch at ({i},{j})"
            );
        }
    }
}

/// Deserialisation is a validating boundary, not a cast: a payload of the
/// right shape whose entries no longer mirror each other — here one
/// off-diagonal overwritten so it disagrees with its transpose — comes back
/// as an error rather than as a matrix. Damaged storage is caught where it
/// enters, instead of surfacing much later as a symmetry audit failure deep
/// inside an update with no trace of where the corruption came from.
///
/// ´claim:linalg:bytes-whose-triangles-disagree-are-rejected-at-deserialisation-not-loaded´
/// ´test:crate:symmetric-rejects-corrupt´
#[test]
fn symmetric_rejects_corrupt() {
    let m = random_spd(5, 0);
    let mut bytes = crate::serde_codec::serialize(&m).unwrap();
    // Corrupt a byte in the data section (not the length prefix).
    if bytes.len() > 20 {
        bytes[20] ^= 0xFF;
    }
    let _result: Result<SymmetricMatrix, _> = crate::serde_codec::deserialize(&bytes);

    let m2 = random_spd(5, 0);

    // Create asymmetric data manually by corrupting a valid matrix.
    let mut data = mat_to_vec(m2.as_inner());
    // Break symmetry: set (0,1) ≠ (1,0).
    data[5] = 999.0; // (0,1) in column-major
    // Wrap in the same format the serde codec expects: { p: usize, data: Vec<f64> }.
    let corrupt_payload = (5_usize, data);
    let bad_bytes = crate::serde_codec::serialize(&corrupt_payload).unwrap();
    let result2: Result<SymmetricMatrix, _> = crate::serde_codec::deserialize(&bad_bytes);
    assert!(result2.is_err(), "should reject asymmetric data");
}
