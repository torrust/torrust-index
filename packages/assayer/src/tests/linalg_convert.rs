// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`col_roundtrip`] | linalg | Flattening a column vector to a plain slice and rebuilding it returns the same length and the same values. The flat form is what serialisation and snapshotting hand around, so a vector that changed shape or drifted in value on the way out would corrupt everything downstream of it. |
//! | [`col_empty_roundtrip`] | linalg | cites (´claim:linalg:a-vector-flattened-to-a-slice-rebuilds-with-its-length-and-values-intact´) |
//! | [`mat_roundtrip`] | linalg | A matrix flattens column-major and rebuilds with both of its dimensions and every entry back where it started — including a non-square one, where a row-major misreading would silently transpose the contents rather than fail. The rebuild is told the shape separately, so the flat form carries no shape of its own and the two must be kept consistent by the caller. |
//! | [`col_nan_preserved`] | linalg | Conversion moves bits, not numbers: NaN and both infinities survive the round trip with their exact bit patterns, compared through `to_bits()` because NaN is unequal even to itself. Nothing in the path canonicalises, clamps or otherwise repairs a non-finite value, so a diagnostic that a NaN has appeared in the model is not quietly erased by saving it. |

//! Crate-level tests for `linalg::convert`.
//!
//! Conversion sits underneath both the serde layer and the snapshot
//! projection, so it has to be a pure change of container: the same numbers,
//! the same shape, no arithmetic performed along the way.

#![allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]

use faer::{Col, Mat};

use crate::linalg::convert::{col_to_vec, mat_to_vec, vec_to_col, vec_to_mat};
use crate::testing::{DEFAULT_TOLERANCES, assert_finite, assert_near};

/// Flattening a column vector to a plain slice and rebuilding it returns the
/// same length and the same values. The flat form is what serialisation and
/// snapshotting hand around, so a vector that changed shape or drifted in
/// value on the way out would corrupt everything downstream of it.
///
/// ´claim:linalg:a-vector-flattened-to-a-slice-rebuilds-with-its-length-and-values-intact´
/// ´test:crate:col-roundtrip´
#[test]
fn col_roundtrip() {
    let col = Col::from_fn(5, |i| (i as f64) * 1.1 + 0.7);
    let v = col_to_vec(&col);
    assert_finite(&v, "col_to_vec output");
    let restored = vec_to_col(&v);
    assert_eq!(col.nrows(), restored.nrows());
    let tol = DEFAULT_TOLERANCES.bit_identical;
    for i in 0..5 {
        assert_near(restored[i], col[i], tol, &format!("col[{i}]"));
    }
}

/// A vector of no dimensions is a legitimate value rather than a degenerate
/// one: it flattens to an empty slice and comes back with zero rows, without
/// the conversion reaching for a first element it does not have. A model
/// whose active set has emptied still round-trips.
///
/// (´claim:linalg:a-vector-flattened-to-a-slice-rebuilds-with-its-length-and-values-intact´)
/// ´test:crate:col-empty-roundtrip´
#[test]
fn col_empty_roundtrip() {
    let col = Col::<f64>::zeros(0);
    let v = col_to_vec(&col);
    assert_eq!(v, [] as [f64; 0]);
    let restored = vec_to_col(&v);
    assert_eq!(restored.nrows(), 0);
}

/// A matrix flattens column-major and rebuilds with both of its dimensions
/// and every entry back where it started — including a non-square one, where
/// a row-major misreading would silently transpose the contents rather than
/// fail. The rebuild is told the shape separately, so the flat form carries
/// no shape of its own and the two must be kept consistent by the caller.
///
/// ´claim:linalg:a-matrix-flattens-column-major-and-rebuilds-at-the-shape-it-is-given´
/// ´test:crate:mat-roundtrip´
#[test]
fn mat_roundtrip() {
    let mat = Mat::from_fn(3, 4, |i, j| (i * 10 + j) as f64);
    let flat = mat_to_vec(&mat);
    assert_finite(&flat, "mat_to_vec output");
    let restored = vec_to_mat(&flat, 3, 4);
    assert_eq!(mat.nrows(), restored.nrows());
    assert_eq!(mat.ncols(), restored.ncols());
    let tol = DEFAULT_TOLERANCES.bit_identical;
    for j in 0..4 {
        for i in 0..3 {
            assert_near(restored[(i, j)], mat[(i, j)], tol, &format!("mat[({i},{j})]"));
        }
    }
}

/// Conversion moves bits, not numbers: NaN and both infinities survive the
/// round trip with their exact bit patterns, compared through `to_bits()`
/// because NaN is unequal even to itself. Nothing in the path canonicalises,
/// clamps or otherwise repairs a non-finite value, so a diagnostic that a
/// NaN has appeared in the model is not quietly erased by saving it.
///
/// ´claim:linalg:conversion-copies-bit-patterns-so-nan-and-the-infinities-pass-through-untouched´
/// ´test:crate:col-nan-preserved´
#[test]
fn col_nan_preserved() {
    // Bit-exact comparison via `to_bits()` — NaN ≠ NaN under `==`, so
    // the tolerance helpers can't express this invariant.
    let data = [1.0, f64::NAN, 3.0, f64::INFINITY, f64::NEG_INFINITY];
    let col = Col::from_fn(5, |i| data[i]);
    let v = col_to_vec(&col);
    let restored = vec_to_col(&v);
    for i in 0..5 {
        assert_eq!(col[i].to_bits(), restored[i].to_bits(), "mismatch at index {i}");
    }
}
