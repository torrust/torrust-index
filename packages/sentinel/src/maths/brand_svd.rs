// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Brand's incremental SVD for subspace evolution (ADR-S-016).
//!
//! Instead of SVD-ing the full d × (k+b) composite matrix, this
//! projects new data onto the current basis, QR-orthogonalises
//! the residual, and SVDs a small (k+b) × (k+b) kernel matrix.
//!
//! The algorithm (Brand 2006) is:
//!
//! 1. P  = `U_k`ᵀ Xᵀ = Zᵀ              (k × b)    — reuse Phase 1
//! 2. Q  = Xᵀ − `U_k` P = residualᵀ    (d × b)    — reuse Phase 1
//! 3. Q⊥ R⊥ = `thin_qr(Q)`             (d × b), (b × b)
//! 4. K  = \[√λ·diag(σ), P; 0, R⊥\]    ((k+b) × (k+b))
//! 5. Û, σ̂, _ = `thin_svd(K)`          small SVD!
//! 6. `U_new` = \[`U_k` | Q⊥\] · Û\[:, :n\]  back-transform
//!
//! Cost: O((k+b)³ + d·b²) vs O(d·(k+b)²) for the naïve approach.
//! The win comes from the SVD kernel shrinking from d rows to (k+b).
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 2 — Subspace evolution
//! - ADR-S-016 — Brand's incremental SVD

use faer::Mat;

use super::SubspaceUpdate;

/// Evolve the subspace via Brand's incremental SVD.
///
/// # Arguments
///
/// * `current_basis` — `U_k`, shape `(d, cap)`. Only columns `[:k]` active.
/// * `sigmas` — singular values, length `cap`. Only `[:k]` meaningful.
/// * `z` — latent projection `X · U_k`, shape `(b, k)`.
/// * `residual` — reconstruction residual `X − X̂`, shape `(b, d)`.
/// * `sqrt_lambda` — `√λ`.
/// * `k` — current active rank.
/// * `cap` — hard ceiling on rank.
// Linear algebra code — single-char names follow standard mathematical notation
// (d=dimension, b=batch, k=rank, c=kernel dim, n=output rank).
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
    let c = k + b; // kernel dimension

    // Guard: Brand's algorithm builds a (c × c) kernel and back-
    // transforms through [U_k | Q⊥].  This only provides a real
    // advantage when c is substantially smaller than d.
    //
    //  • c > d — The thin QR of residual^T (d × b) produces R⊥
    //    that is *not* square (ℝ^(d×b) rather than ℝ^(b×b)),
    //    causing index-out-of-bounds in the kernel construction.
    //
    //  • c = d — The kernel SVD is the same size as a full dense
    //    SVD but adds extra QR + back-transform roundoff.
    //
    //  • c + 1 = d — At tiny d (e.g. coordination tier d=4, c=3)
    //    the extra roundoff produces large basis errors (observed:
    //    54° divergence with well-separated σ after many steps).
    //
    // Require at least 2 spare dimensions (d − c ≥ 2) so the QR
    // residual has room for stable orthogonalisation.  Fall back
    // to the naïve path otherwise.
    if c + 2 > d {
        return None;
    }

    // ── Step 1–2: P = Z^T, Q = residual^T ──────────────
    // Both are already computed in Phase 1 of observe().
    // P = Z^T is (k × b), Q = residual^T is (d × b).

    // ── Step 3: Thin QR of Q = residual^T ───────────────
    // Q = residual^T ∈ ℝ^(d × b).
    // QR gives Q⊥ ∈ ℝ^(d × b) (orthonormal) and R⊥ ∈ ℝ^(b × b) (upper triangular).
    let mut q_t = Mat::zeros(d, b);
    for i in 0..b {
        for j in 0..d {
            q_t[(j, i)] = residual[(i, j)];
        }
    }

    let qr = q_t.as_ref().qr();
    let q_perp = qr.compute_thin_Q(); // (d × b)
    let r_perp = qr.thin_R(); // (b × b)  — upper triangular

    // ── Step 4: Build kernel K ∈ ℝ^(c × c) ─────────────
    //
    //     K = [ √λ · diag(σ₁..ₖ)   P  ]
    //         [       0             R⊥  ]
    //
    // where P = Z^T ∈ ℝ^(k × b).
    let mut kernel = Mat::zeros(c, c);

    // Top-left: √λ · diag(σ[:k])
    for j in 0..k {
        kernel[(j, j)] = sqrt_lambda * sigmas[j];
    }

    // Top-right: P = Z^T  (z is b×k, we need P = k×b)
    for i in 0..k {
        for j in 0..b {
            kernel[(i, k + j)] = z[(j, i)];
        }
    }

    // Bottom-right: R⊥
    for i in 0..b {
        for j in 0..b {
            kernel[(k + i, k + j)] = r_perp[(i, j)];
        }
    }

    // ── Step 5: Small SVD of kernel ─────────────────────
    let svd = kernel.thin_svd().ok()?;

    let n = c.min(d).min(cap);
    let u_hat = svd.U(); // (c × c), we use columns [:n]
    let s_hat = svd.S().column_vector();

    // ── Step 6: Back-transform U_new = [U_k | Q⊥] · Û[:, :n]
    //
    // [U_k | Q⊥] is (d × c), Û[:, :n] is (c × n).
    // Compute column-by-column to avoid materialising the (d × c) join.
    let mut basis = Mat::zeros(d, n);

    for col in 0..n {
        for row in 0..d {
            let mut val = 0.0;
            // U_k block: columns 0..k of [U_k | Q⊥], rows of Û: 0..k
            for j in 0..k {
                val = current_basis[(row, j)].mul_add(u_hat[(j, col)], val);
            }
            // Q⊥ block: columns k..c of [U_k | Q⊥], rows of Û: k..c
            for j in 0..b {
                val = q_perp[(row, j)].mul_add(u_hat[(k + j, col)], val);
            }
            basis[(row, col)] = val;
        }
    }

    // ── Step 7: Re-orthogonalise the output basis ───────
    //
    // Brand's incremental SVD accumulates orthogonality loss
    // because the input basis U_k is itself the output of a
    // previous back-transform step.  Over many iterations the
    // columns of [U_k | Q⊥] drift from exact orthonormality,
    // and the back-transform U_new = [U_k | Q⊥] · Û inherits
    // that error.  Without correction, the accumulated
    // perturbation eventually violates the Wedin bound for
    // moderate spectral gaps (empirically observed at ~15% gap
    // after hundreds of updates).
    //
    // Fix: QR-factorise the output basis and absorb the small
    // R factor into the singular values via a corrective SVD:
    //
    //   basis = Q · R            (thin QR, R is n × n, ≈ I)
    //   R · diag(σ̂) = Uc · Σc · Vc^T   (small SVD)
    //   corrected basis  = Q · Uc
    //   corrected sigmas = diag(Σc)
    //
    // Cost: O(d · n²) for the QR + O(n³) for the small SVD,
    // negligible compared to the existing O(d · b²) QR in step 3.
    let qr_correction = basis.as_ref().qr();
    let q_out = qr_correction.compute_thin_Q(); // (d × n)
    let r_out = qr_correction.thin_R(); // (n × n)

    // Form M = R · diag(σ̂), an n × n matrix.
    let mut m_corr = Mat::zeros(n, n);
    for i in 0..n {
        for j in 0..n {
            m_corr[(i, j)] = r_out[(i, j)] * s_hat[j];
        }
    }

    // SVD of the small corrective matrix.
    let Some(corr_svd) = m_corr.thin_svd().ok() else {
        // Fallback: skip re-orthogonalisation if the tiny SVD
        // fails (should never happen for well-conditioned n × n).
        let out_sigmas: Vec<f64> = (0..n).map(|i| s_hat[i]).collect();
        return Some(SubspaceUpdate {
            basis,
            sigmas: out_sigmas,
            n,
        });
    };
    let u_corr = corr_svd.U(); // (n × n)
    let s_corr = corr_svd.S().column_vector();

    // Final basis = Q_out · U_corr, final sigmas = diag(S_corr).
    let mut final_basis = Mat::zeros(d, n);
    for col in 0..n {
        for row in 0..d {
            let mut val = 0.0;
            for j in 0..n {
                val = q_out[(row, j)].mul_add(u_corr[(j, col)], val);
            }
            final_basis[(row, col)] = val;
        }
    }

    let out_sigmas: Vec<f64> = (0..n).map(|i| s_corr[i]).collect();

    Some(SubspaceUpdate {
        basis: final_basis,
        sigmas: out_sigmas,
        n,
    })
}
