// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Conversion helpers between `faer` types and flat `Vec<f64>`/slices.
//!
//! These functions bridge `faer::Col<f64>` and `faer::Mat<f64>` to standard
//! Rust containers. They are used by both the serde layer (`serde_support`)
//! and the model snapshot projection (`model::bayesian`).
//!
//! # Cross-References
//!
//! - (´dec:substrate:hand-serialisation´) — the hand-written serialisation
//!   strategy these conversion primitives serve

use faer::{Col, Mat};

// ═══════════════════════════════════════════════════════════════════════════════
// Col ↔ Vec
// ═══════════════════════════════════════════════════════════════════════════════

/// Extract a `faer::Col<f64>` as a `Vec<f64>`.
///
/// Uses contiguous slice access for a single `memcpy`; the cost of working at
/// this boundary is owned here (´rem:substrate:cost-ownership´).
pub fn col_to_vec(col: &Col<f64>) -> Vec<f64> {
    // Owned Col<f64> is always contiguous; unwrap is safe.
    col.try_as_col_major()
        .expect("owned Col<f64> is contiguous")
        .as_slice()
        .to_vec()
}

/// Reconstruct a `faer::Col<f64>` from a slice.
pub fn vec_to_col(data: &[f64]) -> Col<f64> {
    Col::from_fn(data.len(), |i| data[i])
}

// ═══════════════════════════════════════════════════════════════════════════════
// Mat ↔ Vec (column-major)
// ═══════════════════════════════════════════════════════════════════════════════

/// Extract a `faer::Mat<f64>` as a flat column-major `Vec<f64>`.
pub fn mat_to_vec(mat: &Mat<f64>) -> Vec<f64> {
    let (nrows, ncols) = (mat.nrows(), mat.ncols());
    let mut data = Vec::with_capacity(nrows * ncols);
    for j in 0..ncols {
        data.extend_from_slice(mat.col_as_slice(j));
    }
    data
}

/// Reconstruct a `faer::Mat<f64>` from a flat column-major slice.
///
/// # Panics
///
/// Panics if `data.len() != nrows * ncols`.
pub fn vec_to_mat(data: &[f64], nrows: usize, ncols: usize) -> Mat<f64> {
    assert_eq!(
        data.len(),
        nrows * ncols,
        "vec_to_mat: data.len() = {} but expected {} × {} = {}",
        data.len(),
        nrows,
        ncols,
        nrows * ncols,
    );
    Mat::from_fn(nrows, ncols, |i, j| data[j * nrows + i])
}
