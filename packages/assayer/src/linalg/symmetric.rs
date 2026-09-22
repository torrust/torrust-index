// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Symmetric matrix wrapper with bitwise symmetry enforcement.
//!
//! `SymmetricMatrix` is a newtype over `faer::Mat<f64>` that guarantees
//! bitwise symmetry after every mutation.  All modification goes through
//! named operation methods — there is no `as_inner_mut()`.
//!
//! In debug builds, a full O(p²) symmetry + NaN audit runs after every
//! mutating method (stripped in release).
//!
//! # Cross-References
//!
//! - (´dec:substrate:symmetry-invariant´) — symmetric matrix storage and
//!   the invariant its layout maintains
//! - (´dec:substrate:single-backend´) — `faer` as the one linear algebra
//!   backend
//! - (´conv:substrate:mirror-direction´) — the mirroring whose tax the
//!   halved cost below is paying

use std::fmt;

use faer::{Col, Mat};

// ═══════════════════════════════════════════════════════════════════════════════
// Error Type
// ═══════════════════════════════════════════════════════════════════════════════

/// Error returned when a matrix loaded from storage is not symmetric.
#[derive(Debug)]
pub struct AsymmetryError {
    /// Row index of the first asymmetric element pair.
    pub row: usize,
    /// Column index of the first asymmetric element pair.
    pub col: usize,
    /// Value at `(row, col)`.
    pub val_ij: f64,
    /// Value at `(col, row)`.
    pub val_ji: f64,
}

impl fmt::Display for AsymmetryError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "asymmetric matrix: ({}, {}) = {} but ({}, {}) = {} (diff = {:e})",
            self.row,
            self.col,
            self.val_ij,
            self.col,
            self.row,
            self.val_ji,
            (self.val_ij - self.val_ji).abs(),
        )
    }
}

impl std::error::Error for AsymmetryError {}

// ═══════════════════════════════════════════════════════════════════════════════
// SymmetricMatrix
// ═══════════════════════════════════════════════════════════════════════════════

/// A symmetric matrix with enforced bitwise symmetry.
///
/// Wraps a `faer::Mat<f64>`. After every mutating operation, the
/// authoritative (lower) triangle is mirrored to the upper triangle.
/// In debug builds, a full $O(p^2)$ symmetry + NaN audit runs after
/// every mutation.
///
/// **No `as_inner_mut()`** — all mutation goes through named methods.
///
/// No raw mutable access is exposed (´dec:substrate:no-raw-access´).
#[derive(Clone)]
pub struct SymmetricMatrix {
    /// Inner dense matrix. Invariant: bitwise symmetric after every method.
    inner: Mat<f64>,
}

impl fmt::Debug for SymmetricMatrix {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("SymmetricMatrix").field("dim", &self.inner.nrows()).finish()
    }
}

impl SymmetricMatrix {
    // ─────────────────────────────────────────────────────────────────────
    // Constructors
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a zero matrix of dimension `p × p`.
    #[must_use]
    pub fn zeros(p: usize) -> Self {
        Self { inner: Mat::zeros(p, p) }
    }

    /// Creates a scaled identity matrix: `scale · I`.
    #[must_use]
    pub fn identity_scaled(p: usize, scale: f64) -> Self {
        let inner = Mat::from_fn(p, p, |i, j| if i == j { scale } else { 0.0 });
        Self { inner }
    }

    /// Absorbs a matrix produced by an internal computation.
    ///
    /// Rounding asymmetry is tolerated: the lower triangle is mirrored
    /// to the upper. Use this for Cholesky inverse output, Schur
    /// correction results, and similar computed matrices.
    #[must_use]
    pub fn from_computation(mat: Mat<f64>) -> Self {
        debug_assert_eq!(mat.nrows(), mat.ncols(), "from_computation: matrix must be square");
        let mut s = Self { inner: mat };
        s.mirror_lower_to_upper();
        s.debug_audit();
        s
    }

    /// Validates and wraps a matrix loaded from disk.
    ///
    /// Rejects matrices where any `|(i,j) - (j,i)| > tolerance`.
    /// The default tolerance of `1e-12` is appropriate for checkpoint
    /// data that was serialised from a valid `SymmetricMatrix`.
    ///
    /// # Errors
    ///
    /// Returns `AsymmetryError` if any element pair differs by more
    /// than `tolerance`.
    pub fn from_storage(mat: Mat<f64>, tolerance: f64) -> Result<Self, AsymmetryError> {
        let p = mat.nrows();
        assert_eq!(p, mat.ncols(), "from_storage: matrix must be square");

        for j in 0..p {
            for i in (j + 1)..p {
                let diff = (mat[(i, j)] - mat[(j, i)]).abs();
                if diff > tolerance {
                    return Err(AsymmetryError {
                        row: i,
                        col: j,
                        val_ij: mat[(i, j)],
                        val_ji: mat[(j, i)],
                    });
                }
            }
        }

        let mut s = Self { inner: mat };
        s.mirror_lower_to_upper();
        s.debug_audit();
        Ok(s)
    }

    // ─────────────────────────────────────────────────────────────────────
    // Mutation methods (each mirrors after mutation)
    // ─────────────────────────────────────────────────────────────────────

    /// Symmetric rank-1 update: `M ← M + w · v v^T`.
    ///
    /// Cost: $O(p^2/2)$ — lower triangle, then mirror
    /// (´conv:substrate:mirror-direction´).
    pub fn symmetric_rank1_update(&mut self, w: f64, v: &Col<f64>) {
        let p = self.inner.nrows();
        debug_assert_eq!(v.nrows(), p, "rank1_update: dimension mismatch");

        // Write lower triangle: M[i,j] += w * v[i] * v[j] for i >= j
        for j in 0..p {
            let vj = v[j];
            for i in j..p {
                let vi = v[i];
                self.inner[(i, j)] = (w * vi).mul_add(vj, self.inner[(i, j)]);
            }
        }
        self.mirror_lower_to_upper();
        self.debug_audit();
    }

    /// Combined scale and rank-1 update: `M ← s·M + α · v v^T`.
    ///
    /// The coefficient `α` may be negative (covariance downdate).
    /// Cost: $O(p^2/2)$ — lower triangle, then mirror
    /// (´conv:substrate:mirror-direction´).
    pub fn scale_and_symmetric_rank1_update(&mut self, s: f64, alpha: f64, v: &Col<f64>) {
        let p = self.inner.nrows();
        debug_assert_eq!(v.nrows(), p, "scale_and_rank1: dimension mismatch");

        // Write lower triangle: M[i,j] = s * M[i,j] + α * v[i] * v[j]
        for j in 0..p {
            let vj = v[j];
            for i in j..p {
                let vi = v[i];
                let old = self.inner[(i, j)];
                self.inner[(i, j)] = (alpha * vi).mul_add(vj, s * old);
            }
        }
        self.mirror_lower_to_upper();
        self.debug_audit();
    }

    /// Scalar scale: `M ← s · M`.
    ///
    /// Cost: $O(p^2/2)$ — scales only the lower triangle and mirrors.
    pub fn scale(&mut self, s: f64) {
        let p = self.inner.nrows();
        for j in 0..p {
            for i in j..p {
                self.inner[(i, j)] *= s;
            }
        }
        self.mirror_lower_to_upper();
        self.debug_audit();
    }

    /// Clamp diagonal elements to a minimum value.
    ///
    /// `M[i,i] ← max(M[i,i], floor)` for all `i`.
    /// Off-diagonal elements unchanged. Symmetry preserved.
    /// Cost: $O(p)$; the replenishment floor
    /// (´req:gaussian:prior-replenishment-floor´).
    pub fn clamp_diagonal_min(&mut self, floor: f64) {
        let p = self.inner.nrows();
        for i in 0..p {
            if self.inner[(i, i)] < floor {
                self.inner[(i, i)] = floor;
            }
        }
        // Diagonal-only change — symmetry preserved.
        self.debug_audit();
    }

    /// Writes a symmetric block onto the diagonal, starting at `start`.
    ///
    /// The `k × k` block replaces the positions `[start, start + k)` in both
    /// index directions and touches nothing else: the rows and columns that
    /// couple those positions to the rest of the matrix keep whatever they
    /// hold. Symmetry survives because a symmetric block written at a
    /// diagonal position is symmetric about the same diagonal.
    ///
    /// This is what restores an archived self-structure block into a model
    /// the extension has just widened (´alg:registry:hibernation´): the
    /// extension left the block at the prior with zero couplings, and the
    /// couplings stay zero because cross-feature structure is not preserved
    /// (´cav:limitation:hibernation´).
    ///
    /// # Panics
    ///
    /// Debug-asserts that the block fits inside this matrix.
    pub fn write_diagonal_block(&mut self, start: usize, block: &Self) {
        let k = block.inner.nrows();
        debug_assert!(
            start.checked_add(k).is_some_and(|end| end <= self.inner.nrows()),
            "write_diagonal_block: block of {k} at {start} does not fit in {}",
            self.inner.nrows()
        );
        for j in 0..k {
            for i in 0..k {
                self.inner[(start + i, start + j)] = block.inner[(i, j)];
            }
        }
        self.debug_audit();
    }

    // ─────────────────────────────────────────────────────────────────────
    // Non-mutating constructors (return new SymmetricMatrix)
    // ─────────────────────────────────────────────────────────────────────

    /// Extend the matrix by `r` dimensions with independent prior.
    ///
    /// Returns a `(p+r) × (p+r)` matrix with the original in the top-left
    /// block and `diag_val · I` in the bottom-right block. Cross-blocks
    /// are zero (´thm:gaussian:extension´).
    #[must_use]
    pub fn extend(&self, r: usize, diag_val: f64) -> Self {
        let p = self.inner.nrows();
        let new_p = p.checked_add(r).expect("extend: dimension overflow");
        let inner = Mat::from_fn(new_p, new_p, |i, j| {
            if i < p && j < p {
                self.inner[(i, j)]
            } else if i >= p && j >= p && i == j {
                diag_val
            } else {
                0.0
            }
        });
        let result = Self { inner };
        result.debug_audit();
        result
    }

    /// Extract a symmetric submatrix at the given indices.
    ///
    /// Returns a `k × k` matrix where `k = indices.len()`,
    /// gathering elements from the row/column indices given.
    /// Extracts `B_kk` and `B_rr` from the gathered partition
    /// (´def:gaussian:gathered-partition´).
    #[must_use]
    pub fn extract_symmetric_submatrix(&self, indices: &[usize]) -> Self {
        let k = indices.len();
        let inner = Mat::from_fn(k, k, |i, j| self.inner[(indices[i], indices[j])]);
        let result = Self { inner };
        result.debug_audit();
        result
    }

    /// Extract a cross-block (non-symmetric) submatrix.
    ///
    /// Returns an `r × c` matrix gathering from the specified
    /// row and column indices, extracting `B_kr` from the gathered partition
    /// (´def:gaussian:gathered-partition´).
    #[must_use]
    pub fn extract_cross_block(&self, row_indices: &[usize], col_indices: &[usize]) -> Mat<f64> {
        let r = row_indices.len();
        let c = col_indices.len();
        Mat::from_fn(r, c, |i, j| self.inner[(row_indices[i], col_indices[j])])
    }

    /// Create a new matrix by adding `delta · I` to self.
    ///
    /// This is the Schur regularisation
    /// (´alg:gaussian:regularised-schur´).
    #[must_use]
    pub fn add_scaled_identity(&self, delta: f64) -> Self {
        let p = self.inner.nrows();
        let inner = Mat::from_fn(p, p, |i, j| self.inner[(i, j)] + if i == j { delta } else { 0.0 });
        let result = Self { inner };
        result.debug_audit();
        result
    }

    /// The ordinary matrix product `self · other`, which is not symmetric and
    /// so is not one of these.
    ///
    /// Written as an explicit call rather than with the backend's operator
    /// because the operator reads the backend's global parallelism, and the
    /// width of the work is a decision this package takes from the matrix
    /// (´dec:substrate:backend-isolation´). Cost: $O(p^3)$.
    #[must_use]
    pub fn multiply(&self, other: &Self) -> Mat<f64> {
        let p = self.inner.nrows();
        debug_assert_eq!(p, other.inner.nrows(), "multiply: dimension mismatch");
        let mut product = Mat::zeros(p, p);
        faer::linalg::matmul::matmul(
            product.as_mut(),
            faer::Accum::Replace,
            self.inner.as_ref(),
            other.inner.as_ref(),
            1.0,
            super::bridge::parallelism_for(p),
        );
        product
    }

    /// Subtract another symmetric matrix element-wise.
    ///
    /// This is the Schur correction, `B' = B_kk - correction`
    /// (´alg:gaussian:regularised-schur´).
    #[must_use]
    pub fn subtract(&self, other: &Self) -> Self {
        let p = self.inner.nrows();
        debug_assert_eq!(p, other.inner.nrows(), "subtract: dimension mismatch");
        let result = Self {
            inner: &self.inner - &other.inner,
        };
        result.debug_audit();
        result
    }

    // ─────────────────────────────────────────────────────────────────────
    // Read methods
    // ─────────────────────────────────────────────────────────────────────

    /// Symmetric matrix-vector product: returns `M · v`.
    ///
    /// Cost: $O(p^2)$.
    #[must_use]
    pub fn symmetric_matvec(&self, v: &Col<f64>) -> Col<f64> {
        debug_assert_eq!(v.nrows(), self.inner.nrows(), "matvec: dimension mismatch");
        // Use faer's general matmul — reads both triangles (correct under
        // mirroring invariant). ~2× theoretical minimum traffic but negligible
        // at p ≤ 622.
        &self.inner * v
    }

    /// Quadratic form: returns `v^T M v`.
    ///
    /// Cost: $O(p^2)$ (computed as `v · (M · v)`).
    #[must_use]
    pub fn quadratic_form(&self, v: &Col<f64>) -> f64 {
        let mv = self.symmetric_matvec(v);
        dot(v, &mv)
    }

    /// Combined quadratic form and product.
    ///
    /// Returns `(Mv, v^T M v)` in a single pass, saving one matvec.
    /// The update's first step returns $(v = \Sigma\hat\phi, h = \hat\phi^\top v)$
    /// (´alg:update:sherman-morrison´).
    #[must_use]
    pub fn quadratic_form_with_product(&self, v: &Col<f64>) -> (Col<f64>, f64) {
        let mv = self.symmetric_matvec(v);
        let h = dot(v, &mv);
        (mv, h)
    }

    /// Quadratic form computed from a slice, avoiding allocation.
    ///
    /// The blended uncertainty (´prop:risk:blend-variance´) is computed this
    /// way to avoid a ~5 KB allocation.
    #[must_use]
    pub fn quadratic_form_from_slice(&self, s: &[f64]) -> f64 {
        let p = self.inner.nrows();
        debug_assert_eq!(s.len(), p, "quadratic_form_from_slice: dimension mismatch");
        let mut result = 0.0;
        for i in 0..p {
            for j in 0..p {
                result = (s[i] * self.inner[(i, j)]).mul_add(s[j], result);
            }
        }
        result
    }

    /// Read a single diagonal element.
    ///
    /// Cost: $O(1)$.
    #[must_use]
    #[inline]
    pub fn diagonal_element(&self, i: usize) -> f64 {
        self.inner[(i, i)]
    }

    /// Iterate over diagonal elements.
    ///
    /// Cost: $O(p)$.
    pub fn diagonal_iter(&self) -> impl Iterator<Item = f64> + '_ {
        (0..self.inner.nrows()).map(move |i| self.inner[(i, i)])
    }

    /// Returns `(max_diagonal, min_diagonal)`.
    ///
    /// The two ends of the $O(p)$ condition estimate
    /// (´def:monitoring:condition-number´).
    ///
    /// # Panics
    ///
    /// Panics if the matrix is empty (p = 0).
    #[must_use]
    pub fn diagonal_min_max(&self) -> (f64, f64) {
        let p = self.inner.nrows();
        assert!(p > 0, "diagonal_min_max: matrix is empty");
        let mut max_val = self.inner[(0, 0)];
        let mut min_val = max_val;
        for i in 1..p {
            let d = self.inner[(i, i)];
            if d > max_val {
                max_val = d;
            }
            if d < min_val {
                min_val = d;
            }
        }
        (max_val, min_val)
    }

    /// Counts diagonal elements at or below a threshold.
    ///
    /// Reports the number of dimensions at the precision floor,
    /// $B_{jj} = \lambda_\text{floor}$ (´req:monitoring:replenishment-floor´).
    ///
    /// Uses a relative tolerance: `d <= threshold * (1 + 1e-12)` to
    /// account for floating-point representation of the floor value.
    ///
    /// Cost: $O(p)$.
    #[must_use]
    pub fn count_diagonal_at_or_below(&self, threshold: f64) -> usize {
        let tol = threshold * (1.0 + 1e-12);
        let mut count: usize = 0;
        for i in 0..self.inner.nrows() {
            if self.inner[(i, i)] <= tol {
                count += 1;
            }
        }
        count
    }

    /// Read-only access to the inner `faer::Mat<f64>`.
    ///
    /// Used by Cholesky factorisation and bridge functions.
    #[must_use]
    #[inline]
    pub const fn as_inner(&self) -> &Mat<f64> {
        &self.inner
    }

    /// Returns the dimension `p` of this `p × p` matrix.
    #[must_use]
    #[inline]
    pub fn dim(&self) -> usize {
        self.inner.nrows()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Internal helpers
    // ─────────────────────────────────────────────────────────────────────

    /// Mirror the lower triangle to the upper triangle.
    fn mirror_lower_to_upper(&mut self) {
        let p = self.inner.nrows();
        for j in 0..p {
            for i in (j + 1)..p {
                let val = self.inner[(i, j)];
                self.inner[(j, i)] = val;
            }
        }
    }

    /// Debug-only audit: check bitwise symmetry and NaN absence.
    ///
    /// Runs in $O(p^2)$. Stripped in release builds.
    #[inline]
    fn debug_audit(&self) {
        #[cfg(debug_assertions)]
        {
            let p = self.inner.nrows();
            for j in 0..p {
                for i in j..p {
                    let a = self.inner[(i, j)];
                    let b = self.inner[(j, i)];
                    // Bitwise equality check — catches NaN since NaN ≠ NaN.
                    assert!(
                        a.to_bits() == b.to_bits(),
                        "symmetry audit failed: ({i}, {j}) = {a} but ({j}, {i}) = {b}"
                    );
                }
            }
        }
    }
}

/// Dot product of two column vectors.
fn dot(a: &Col<f64>, b: &Col<f64>) -> f64 {
    debug_assert_eq!(a.nrows(), b.nrows(), "dot: dimension mismatch");
    let n = a.nrows();
    let mut sum = 0.0;
    for i in 0..n {
        sum = a[i].mul_add(b[i], sum);
    }
    sum
}
