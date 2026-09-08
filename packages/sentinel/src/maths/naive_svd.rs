// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Naïve dense thin SVD for subspace evolution.
//!
//! This is the original algorithm: build the composite matrix
//!
//!   M = \[√λ · `U_k` · diag(σ₁..ₖ)  |  X^T\]  ∈ ℝ^(d × (k+b))
//!
//! and compute a full dense thin SVD of M via `faer::Mat::thin_svd()`.
//!
//! Correct and simple, but O(d·(k+b)²) per call — the full
//! bidiagonalisation touches every element of the d-row matrix.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 2 — Subspace evolution
//! - ADR-S-016 §Context — Cost analysis

use faer::Mat;

use super::SubspaceUpdate;

/// Evolve the subspace via dense thin SVD of the full composite matrix.
///
/// Builds M = \[√λ · `U_k` · diag(σ) | X^T\] ∈ ℝ^(d × (k+b)) and
/// computes `thin_svd(M)`, retaining the top `n = min(k+b, d, cap)`
/// components.
///
/// # Arguments
///
/// * `current_basis` — `U_k`, shape `(d, cap)`. Only columns `[:k]` active.
/// * `sigmas` — singular values, length `cap`. Only `[:k]` meaningful.
/// * `z` — latent projection `X · U_k`, shape `(b, k)`.  (Unused by
///   naïve — present for API uniformity; the naïve path reconstructs
///   X^T from `residual + U_k · Z^T`.)
/// * `residual` — reconstruction residual `X − X̂`, shape `(b, d)`.
/// * `sqrt_lambda` — `√λ`.
/// * `k` — current active rank.
/// * `cap` — hard ceiling on rank.
// Linear algebra code — single-char names follow standard mathematical notation
// (d=dimension, b=batch, k=rank, m=composite matrix, s=scaled sigma, n=output rank).
#[allow(clippy::many_single_char_names)]
#[must_use]
pub fn evolve(
    current_basis: &Mat<f64>,
    sigmas: &[f64],
    z: &Mat<f64>,
    residual: &Mat<f64>,
    sqrt_lambda: f64,
    k: usize,
    cap: usize,
) -> Option<SubspaceUpdate> {
    let d = current_basis.nrows();
    let b = residual.nrows();
    let cols = k + b;

    // Build M: (d × (k + b))
    let mut m = Mat::zeros(d, cols);

    // Left block: √λ · U_k · diag(σ[:k])
    for j in 0..k {
        let s = sqrt_lambda * sigmas[j];
        for i in 0..d {
            m[(i, j)] = current_basis[(i, j)] * s;
        }
    }

    // Right block: X^T.
    // X = residual + Z · U_k^T  (reconstruct from Phase 1 outputs).
    // X^T[j, i] = X[i, j] = residual[i, j] + Σₗ z[i, l] · U_k[j, l]
    for i in 0..b {
        for j in 0..d {
            let mut val = residual[(i, j)];
            for l in 0..k {
                val = z[(i, l)].mul_add(current_basis[(j, l)], val);
            }
            m[(j, k + i)] = val;
        }
    }

    // Thin SVD of M.
    let svd = m.thin_svd().ok()?;

    let n = cols.min(d).min(cap);
    let u_new = svd.U();
    let s_new = svd.S().column_vector();

    let mut basis = Mat::zeros(d, n);
    let mut out_sigmas = Vec::with_capacity(n);

    for j in 0..n {
        for i in 0..d {
            basis[(i, j)] = u_new[(i, j)];
        }
        out_sigmas.push(s_new[j]);
    }

    Some(SubspaceUpdate {
        basis,
        sigmas: out_sigmas,
        n,
    })
}
