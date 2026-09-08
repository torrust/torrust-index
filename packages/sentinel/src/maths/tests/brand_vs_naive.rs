// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`equivalence_representative_configs`] | svd | Across a spread of ambient widths, ranks, batch sizes and forgetting factors, the incremental update lands on the same singular values and the same axes as decomposing the whole composite matrix would. The cheap path is not an approximation that happens to be close enough; it is the same answer reached by exploiting structure the reference path ignores, which is why it can simply replace it. |
//! | [`equivalence_rank_one`] | svd | cites (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´) |
//! | [`equivalence_minimal_dimensions`] | svd | The incremental path declines to answer when the kernel it would build is not comfortably smaller than the ambient width: it needs spare dimensions for the residual's orthogonalisation to be stable, and without them the extra factorisation buys nothing and costs accuracy. At the narrowest widths it therefore returns nothing while the reference path still produces a model, and once there is headroom it runs and agrees again. Declining is how the boundary is expressed — never a degraded answer offered as a good one. |
//! | [`equivalence_single_sample`] | svd | cites (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´) |
//! | [`equivalence_large_batch`] | svd | cites (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´) |
//! | [`equivalence_saturated_rank`] | svd | cites (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´) |
//! | [`equivalence_cap_larger_than_k`] | svd | How many axes a step produces is the smallest of three limits: what the old rank plus the batch could span, the ambient width, and the ceiling the cell is allowed. With capacity to spare the step fills it, widening the model beyond the rank it started from — and both paths widen it identically, to the same count and with columns that are still orthonormal. Spare capacity is real room to grow rather than padding. |
//! | [`equivalence_identity_basis`] | svd | cites (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´) |
//! | [`equivalence_near_zero_residual`] | svd | A batch that already lies inside the model's own axes leaves almost nothing unexplained, and the incremental path has to orthogonalise that almost- nothing anyway — a factorisation of a matrix that is numerically close to rank-deficient. Directions recovered from vanishing energy are arbitrary, so the two paths part company in the last few digits, but they still land on the same model. Precision degrades where there is nothing left to measure; agreement does not. |
//! | [`equivalence_zero_initial_sigmas`] | svd | A model that has learned nothing yet carries no energy on any axis, so the remembered half of the step contributes exactly zero and the outcome is determined entirely by the arriving batch. The step is well defined there rather than degenerate: a cell's first real shape comes from its first real data, and both paths derive that shape identically. |
//! | [`equivalence_large_singular_values`] | svd | When a model carries enormous accumulated energy, an ordinary batch is a vanishing perturbation of it, and the difference between the two paths scales with that energy rather than staying absolute. Agreement is therefore stated relatively: the singular values match to a proportion of themselves, not to a fixed margin, so a long-lived cell whose values have grown large is no less trustworthy than a fresh one. |
//! | [`equivalence_equal_singular_values`] | svd | When every axis carries the same energy, nothing distinguishes one axis from another within the space they span: any rotation of them is an equally correct answer. The two paths still agree exactly on how much energy there is and on which space it occupies, while individual columns are compared only loosely, because insisting they coincide would be demanding an answer the mathematics does not define. |
//! | [`equivalence_no_forgetting`] | svd | Forgetting enters the step only as a scale factor on the remembered energy, so turning it off is the ordinary case with that factor at one, not a separate code path. A cell configured to weight all its history equally therefore evolves by the same arithmetic as one that discounts the past, and the two paths agree there as everywhere else. |
//! | [`output_basis_is_orthonormal`] | svd | Every step returns axes that are unit length and mutually perpendicular, from either path. Everything downstream assumes it: a batch's coordinates are obtained by projecting onto these axes, and the unexplained part is the batch minus that projection, which is only a decomposition if the axes are orthonormal. The incremental path has to work for this — its back-transform inherits drift from the basis it started with, so it re-orthogonalises before returning rather than trusting the construction. |
//! | [`singular_values_sorted_and_nonnegative`] | svd | Singular values come back non-negative and in descending order from both paths. Order is what makes position meaningful: rank adaptation walks the values accumulating energy until a threshold is met and takes that position as the rank, which is only a sensible rule if the strongest direction is first. The incremental path preserves the ordering through its back-transform and corrective step rather than inheriting it by luck. |
//! | [`deterministic_reproduction`] | svd | Given the same inputs, either path returns the same axes and the same values bit for bit — not close, identical. Nothing in the step draws on randomness, iteration order or timing, so two sentinels fed the same traffic hold the same model, and a difference between them is always evidence about the traffic rather than about the machine. |
//! | [`equivalence_multi_step`] | svd | Agreement between the two paths is a property of a whole streaming run, not of one step in isolation. Each path is fed the same sequence but carries its own state forward, so any difference compounds through every later step — and over a long run the difference stays within a margin that grows only in step with the number of steps taken. Divergence is linear rather than explosive, which is what makes a cell that has been running for hours as trustworthy as one that has just started. |
//! | [`reconstruction_error_equivalence`] | svd | The two paths do not merely agree on coordinates they were given; they explain data they have never seen equally well. Held-out rows projected onto either model leave the same amount unaccounted for, which is the property the sentinel actually depends on — novelty is measured from exactly that leftover, so equal reconstruction means equal scores whichever path produced the axes. |

//! Tests for the two subspace-evolution algorithms — the reference dense
//! decomposition and the incremental one that runs in production.
//!
//! Both answer the same question. Given a cell's current axes and singular
//! values, a batch's coordinates within those axes, and the part of the batch
//! those axes failed to explain, what should the axes and values become? The
//! reference path assembles the whole composite matrix and decomposes it,
//! which is simple and costs a pass over every ambient dimension. The
//! incremental path orthogonalises only the unexplained part, decomposes a
//! small kernel whose size is the rank plus the batch rather than the ambient
//! width, and transforms the result back — then re-orthogonalises, because the
//! basis it starts from is itself the output of an earlier back-transform and
//! drifts if left uncorrected.
//!
//! The cheap path is therefore licensed only by agreement, and the engine
//! leans on that directly: whenever the oracle is active it runs both and
//! compares them, so a divergence is a panic rather than a silently different
//! model. These tests are where the agreement is established — across
//! dimensions, ranks, batch shapes, spectra and starting states, and over a
//! long run of steps where floating-point difference has time to accumulate.
//!
//! Where the two cannot be made to agree they decline to differ instead. The
//! incremental path returns nothing when the ambient width leaves no headroom
//! for a stable orthogonalisation of the residual, and the caller falls back
//! to the reference path rather than accepting a worse answer.

use faer::Mat;
use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};

use crate::maths::{SubspaceUpdate, brand_svd, naive_svd};

// ════════════════════════════════════════════════════════════
//  Helpers
// ════════════════════════════════════════════════════════════

/// Generate a random (rows × cols) matrix with values in [-1, 1].
fn random_matrix(rows: usize, cols: usize, rng: &mut SmallRng) -> Mat<f64> {
    let mut m = Mat::zeros(rows, cols);
    for i in 0..rows {
        for j in 0..cols {
            m[(i, j)] = rng.random_range(-1.0..1.0);
        }
    }
    m
}

/// Build a random identity-like basis (d × cap) with orthonormal columns.
fn random_basis(d: usize, cap: usize, rng: &mut SmallRng) -> Mat<f64> {
    // Start with random matrix, then QR to get orthonormal columns.
    let raw = random_matrix(d, cap, rng);
    let qr = raw.as_ref().qr();
    qr.compute_thin_Q()
}

/// Given X (b×d) and `U_k` (d×cap, using [:k]), compute z and residual.
fn phase1_projection(x: &Mat<f64>, basis: &Mat<f64>, k: usize) -> (Mat<f64>, Mat<f64>) {
    let u_k = basis.subcols(0, k);
    let z = x * u_k; // (b × k)
    let x_hat = &z * u_k.transpose(); // (b × d)
    let residual = x - &x_hat; // (b × d)
    (z, residual)
}

/// Assert two `SubspaceUpdate`s are approximately equal.
///
/// - Singular values: relative tolerance `sigma_tol`.
/// - Basis columns: compared up to sign via |cos(angle)| > `basis_tol`.
fn assert_updates_close(a: &SubspaceUpdate, b: &SubspaceUpdate, sigma_tol: f64, basis_tol: f64, label: &str) {
    assert_eq!(a.n, b.n, "{label}: n mismatch ({} vs {})", a.n, b.n);
    let n = a.n;
    let d = a.basis.nrows();

    // Absolute floor below which singular values are considered zero.
    // Near-zero values carry arbitrary numerical residual, making
    // relative comparison meaningless.
    let sigma_abs_floor = 1e-6;

    // Singular values.
    for i in 0..n {
        let sa = a.sigmas[i];
        let sb = b.sigmas[i];
        if sa.abs() < sigma_abs_floor && sb.abs() < sigma_abs_floor {
            continue;
        }
        let denom = sa.abs().max(sb.abs()).max(1e-15);
        let rel = (sa - sb).abs() / denom;
        assert!(
            rel < sigma_tol,
            "{label}: σ[{i}] mismatch: naïve={sa:.12e}, brand={sb:.12e}, rel={rel:.2e}"
        );
    }

    // Basis columns (up to sign).
    // Skip columns whose singular value is near-zero (arbitrary direction).
    for j in 0..n {
        if a.sigmas[j].abs() < sigma_abs_floor && b.sigmas[j].abs() < sigma_abs_floor {
            continue;
        }
        let mut dot = 0.0;
        for i in 0..d {
            dot = a.basis[(i, j)].mul_add(b.basis[(i, j)], dot);
        }
        let cosine = dot.abs();
        assert!(cosine > basis_tol, "{label}: basis col {j} diverged: |cos| = {cosine:.8e}");
    }
}

/// Assert that the columns of a matrix are approximately orthonormal.
fn assert_orthonormal(basis: &Mat<f64>, n: usize, tol: f64, label: &str) {
    let d = basis.nrows();
    for j in 0..n {
        // Column norm ≈ 1.
        let mut norm_sq = 0.0;
        for i in 0..d {
            norm_sq = basis[(i, j)].mul_add(basis[(i, j)], norm_sq);
        }
        let norm = norm_sq.sqrt();
        assert!(
            (norm - 1.0).abs() < tol,
            "{label}: column {j} norm = {norm:.8e}, expected ≈ 1.0"
        );

        // Pairwise orthogonality.
        for l in (j + 1)..n {
            let mut dot = 0.0;
            for i in 0..d {
                dot = basis[(i, j)].mul_add(basis[(i, l)], dot);
            }
            assert!(
                dot.abs() < tol,
                "{label}: columns {j} and {l} not orthogonal: dot = {dot:.8e}"
            );
        }
    }
}

/// Run both algorithms and return (naïve, brand) results.
///
/// Brand returns `None` when `k + b >= d` (the kernel is not smaller
/// than the ambient dimension — see guard in `brand_svd::evolve`).
/// In that case the second element is `None`.
// Linear algebra test helpers — d, k, b, z, x follow standard mathematical
// notation for dimensionality, rank, batch size, latent projections, and input.
fn run_both(
    basis: &Mat<f64>,
    sigmas: &[f64],
    z: &Mat<f64>,
    residual: &Mat<f64>,
    sqrt_lambda: f64,
    k: usize,
    cap: usize,
) -> (SubspaceUpdate, Option<SubspaceUpdate>) {
    let naive = naive_svd::evolve(basis, sigmas, z, residual, sqrt_lambda, k, cap).expect("naïve SVD should not fail");
    let brand = brand_svd::evolve(basis, sigmas, z, residual, sqrt_lambda, k, cap);
    (naive, brand)
}

/// Pad a (d × n) basis to (d × cap) by appending zero columns.
fn pad_to_cap(basis: &Mat<f64>, d: usize, cap: usize) -> Mat<f64> {
    let n = basis.ncols();
    let mut out = Mat::zeros(d, cap);
    for j in 0..n.min(cap) {
        for i in 0..d {
            out[(i, j)] = basis[(i, j)];
        }
    }
    out
}

/// Pad sigmas to length `cap` with zeros.
fn pad_sigmas(sigmas: &[f64], cap: usize) -> Vec<f64> {
    let mut out = sigmas.to_vec();
    out.resize(cap, 0.0);
    out
}

/// Frobenius norm of reconstruction error: ‖X − U Uᵀ Xᵀ‖_F (row-major).
// Linear algebra helper — d, b, x, z, n follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn recon_error(x: &Mat<f64>, basis: &Mat<f64>, n: usize) -> f64 {
    let b = x.nrows();
    let d = x.ncols();
    let u_n = basis.subcols(0, n);
    let z = x * u_n; // (b × n)
    let x_hat = &z * u_n.transpose(); // (b × d)

    let mut err = 0.0;
    for i in 0..b {
        for j in 0..d {
            let diff = x[(i, j)] - x_hat[(i, j)];
            err = diff.mul_add(diff, err);
        }
    }
    err.sqrt()
}

// ════════════════════════════════════════════════════════════
// § 1 — Equivalence: basic configurations
// ════════════════════════════════════════════════════════════

/// Across a spread of ambient widths, ranks, batch sizes and forgetting
/// factors, the incremental update lands on the same singular values and the
/// same axes as decomposing the whole composite matrix would. The cheap path
/// is not an approximation that happens to be close enough; it is the same
/// answer reached by exploiting structure the reference path ignores, which is
/// why it can simply replace it.
///
/// ´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´
/// ´test:unit:equivalence-representative-configs´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
fn equivalence_representative_configs() {
    let configs: &[(usize, usize, usize, f64)] = &[
        // (d, k, b, λ)
        (32, 2, 4, 0.95),    // small, test-like
        (64, 4, 8, 0.99),    // medium
        (128, 2, 4, 0.95),   // benchmark config
        (128, 2, 16, 0.99),  // realistic config
        (128, 16, 16, 0.99), // high rank + large batch
        (256, 8, 32, 0.99),  // wide dimension
    ];

    for &(d, k, b, lambda) in configs {
        let cap = k; // cap = k for simplicity
        let sqrt_lambda = lambda.sqrt();
        let mut rng = SmallRng::seed_from_u64(42);

        let basis = random_basis(d, cap, &mut rng);
        let mut sigmas = vec![0.0; cap];
        for s in &mut sigmas {
            *s = rng.random_range(0.1..10.0);
        }
        // Sort decreasing (as they would be in practice).
        sigmas.sort_by(|a, b| b.partial_cmp(a).unwrap());

        let x = random_matrix(b, d, &mut rng);
        let (z, residual) = phase1_projection(&x, &basis, k);

        let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
        let brand = brand.expect("Brand should succeed when c < d");

        let label = format!("d={d}, k={k}, b={b}, λ={lambda}");
        assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, &label);
    }
}

// ════════════════════════════════════════════════════════════
// § 2 — Equivalence: dimensional edge cases
// ════════════════════════════════════════════════════════════

/// A model carrying a single axis is the most constrained shape the update can
/// take: there is no spectrum to order and no neighbouring direction for the
/// new energy to be confused with. Agreement holds there too, so the two paths
/// do not depend on a rich spectrum to coincide.
///
/// (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´)
/// ´test:unit:equivalence-rank-one´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_rank_one() {
    let d = 64;
    let k = 1;
    let b = 4;
    let cap = 1;
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(123);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![5.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "rank-1");
}

/// The incremental path declines to answer when the kernel it would build is
/// not comfortably smaller than the ambient width: it needs spare dimensions
/// for the residual's orthogonalisation to be stable, and without them the
/// extra factorisation buys nothing and costs accuracy. At the narrowest
/// widths it therefore returns nothing while the reference path still produces
/// a model, and once there is headroom it runs and agrees again. Declining is
/// how the boundary is expressed — never a degraded answer offered as a good
/// one.
///
/// ´claim:svd:the-incremental-path-declines-when-the-ambient-width-leaves-no-headroom-so-the-reference-path-answers-instead´
/// ´test:unit:equivalence-minimal-dimensions´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_minimal_dimensions() {
    // d=2, k=1, b=1 ⇒ c = k + b = 2, c + 2 = 4 > d = 2.
    // Brand correctly returns None here (headroom guard),
    // so we verify the fallback: only Naive produces a result.
    let d = 2;
    let k = 1;
    let b = 1;
    let cap = 1;
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(101);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![1.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    assert!(brand.is_none(), "Brand should return None when c + 2 > d");
    assert_eq!(naive.n, 1);

    // d=3, c=2: c + 2 = 4 > 3 → still bails.
    let d = 3;
    let basis = random_basis(d, cap, &mut rng);
    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (_naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    assert!(brand.is_none(), "Brand should return None when c + 2 > d (d=3)");

    // d=8, c=2: c + 2 = 4 ≤ 8 → Brand runs (smallest OK case).
    let d = 8;
    let basis = random_basis(d, cap, &mut rng);
    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c + 2 <= d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "minimal-dims-d8");
}

/// The other minimal shape: a batch of one row against a model of several
/// axes. Here the kernel's new block is a single column and the residual's
/// factorisation is one-dimensional, which is the degenerate end of the
/// incremental construction rather than of the model — and agreement survives
/// it.
///
/// (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´)
/// ´test:unit:equivalence-single-sample´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_single_sample() {
    let d = 64;
    let k = 4;
    let b = 1;
    let cap = 4;
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(202);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![8.0, 4.0, 2.0, 1.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "single-sample");
}

/// The opposite imbalance: a batch far wider than the model's rank, so most of
/// the kernel is the freshly orthogonalised residual and only a sliver of it
/// is remembered energy. This is the regime where the incremental path does
/// the most work outside the old basis, and it still coincides with the
/// reference.
///
/// (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´)
/// ´test:unit:equivalence-large-batch´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_large_batch() {
    let d = 64;
    let k = 2;
    let b = 32;
    let cap = 2;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(456);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![10.0, 3.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "large-batch");
}

/// A model already at its rank ceiling has no spare capacity: everything the
/// batch contributes must be resolved within the axes it is allowed to keep,
/// and the trailing singular values are the ones least separated from their
/// neighbours. Agreement holds, at a tolerance loosened to match how much
/// closer those neighbours sit — the weakest directions are where a difference
/// would first show.
///
/// (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´)
/// ´test:unit:equivalence-saturated-rank´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
// usize→f64 cast is for the decreasing-sigma formula: 20.0/(i+1.0) where i ≤ 16.
#[allow(clippy::many_single_char_names, clippy::cast_precision_loss)]
fn equivalence_saturated_rank() {
    let d = 32;
    let k = 16;
    let b = 8;
    let cap = 16;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(789);

    let basis = random_basis(d, cap, &mut rng);
    let mut sigmas = vec![0.0; cap];
    for (i, s) in sigmas.iter_mut().enumerate() {
        *s = 20.0 / (i as f64 + 1.0); // decreasing: 20, 10, 6.67, ..
    }

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-7, 1.0 - 1e-5, "saturated-rank");
}

/// How many axes a step produces is the smallest of three limits: what the old
/// rank plus the batch could span, the ambient width, and the ceiling the cell
/// is allowed. With capacity to spare the step fills it, widening the model
/// beyond the rank it started from — and both paths widen it identically, to
/// the same count and with columns that are still orthonormal. Spare capacity
/// is real room to grow rather than padding.
///
/// ´claim:svd:the-width-of-a-step-is-the-least-of-what-the-batch-can-span-the-ambient-width-and-the-ceiling´
/// ´test:unit:equivalence-cap-larger-than-k´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_cap_larger_than_k() {
    let d = 64;
    let k = 2;
    let b = 4;
    let cap = 8; // Much larger than k
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(555);

    let mut basis = Mat::zeros(d, cap);
    // Only initialise the first k columns as proper orthonormal vectors.
    let partial = random_basis(d, k, &mut rng);
    for j in 0..k {
        for i in 0..d {
            basis[(i, j)] = partial[(i, j)];
        }
    }
    let mut sigmas = vec![0.0; cap];
    sigmas[0] = 5.0;
    sigmas[1] = 2.0;

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");

    // n should be min(k+b, d, cap) = min(6, 64, 8) = 6.
    assert_eq!(naive.n, 6);
    assert_eq!(brand.n, 6);
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "cap>k");
    assert_orthonormal(&naive.basis, naive.n, 1e-10, "naïve cap>k");
    assert_orthonormal(&brand.basis, brand.n, 1e-10, "brand cap>k");
}

// ════════════════════════════════════════════════════════════
// § 3 — Equivalence: data / state edge cases
// ════════════════════════════════════════════════════════════

/// The state a cell actually begins in is not a random orthonormal basis but
/// an axis-aligned one, each column a single coordinate direction, with tiny
/// singular values standing in for knowledge not yet acquired. That start is
/// perfectly orthonormal even though nothing rotated it there, and both paths
/// evolve it alike — so the very first steps a live cell takes are covered by
/// the same agreement as its later ones.
///
/// (´claim:svd:the-incremental-update-reaches-the-same-answer-as-the-reference-decomposition´)
/// ´test:unit:equivalence-identity-basis´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_identity_basis() {
    let d = 64;
    let k = 2;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(314);

    // Identity-like basis (same as SubspaceTracker::new).
    let mut basis = Mat::zeros(d, cap);
    for j in 0..cap.min(d) {
        basis[(j, j)] = 1.0;
    }
    let sigmas = vec![0.01; cap];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "identity-basis");
}

/// A batch that already lies inside the model's own axes leaves almost nothing
/// unexplained, and the incremental path has to orthogonalise that almost-
/// nothing anyway — a factorisation of a matrix that is numerically close to
/// rank-deficient. Directions recovered from vanishing energy are arbitrary,
/// so the two paths part company in the last few digits, but they still land
/// on the same model. Precision degrades where there is nothing left to
/// measure; agreement does not.
///
/// ´claim:svd:a-batch-with-almost-nothing-left-unexplained-costs-precision-without-costing-agreement´
/// ´test:unit:equivalence-near-zero-residual´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_near_zero_residual() {
    let d = 64;
    let k = 4;
    let b = 4;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(999);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![10.0, 5.0, 2.0, 1.0];

    // Build X as a linear combination of basis vectors + tiny noise.
    let u_k = basis.subcols(0, k);
    let coeffs = random_matrix(b, k, &mut rng);
    let noise = {
        let mut n = random_matrix(b, d, &mut rng);
        for i in 0..b {
            for j in 0..d {
                n[(i, j)] *= 1e-10; // tiny noise
            }
        }
        n
    };
    let x = &coeffs * u_k.transpose() + &noise; // (b × d), mostly in subspace

    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");

    // Looser tolerance due to near-singular QR.
    assert_updates_close(&naive, &brand, 1e-4, 1.0 - 1e-3, "near-zero-residual");
}

/// A model that has learned nothing yet carries no energy on any axis, so the
/// remembered half of the step contributes exactly zero and the outcome is
/// determined entirely by the arriving batch. The step is well defined there
/// rather than degenerate: a cell's first real shape comes from its first real
/// data, and both paths derive that shape identically.
///
/// ´claim:svd:a-model-carrying-no-energy-yet-takes-its-whole-shape-from-the-arriving-batch´
/// ´test:unit:equivalence-zero-initial-sigmas´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_zero_initial_sigmas() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(303);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![0.0; cap]; // Cold start — no prior information.

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "zero-initial-sigmas");
}

/// When a model carries enormous accumulated energy, an ordinary batch is a
/// vanishing perturbation of it, and the difference between the two paths
/// scales with that energy rather than staying absolute. Agreement is
/// therefore stated relatively: the singular values match to a proportion of
/// themselves, not to a fixed margin, so a long-lived cell whose values have
/// grown large is no less trustworthy than a fresh one.
///
/// ´claim:svd:agreement-is-relative-so-a-model-carrying-large-energy-is-held-to-a-proportional-margin´
/// ´test:unit:equivalence-large-singular-values´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_large_singular_values() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(666);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![1e6, 5e5, 1e5, 1e4];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-6, 1.0 - 1e-4, "large-sigmas");
}

/// When every axis carries the same energy, nothing distinguishes one axis
/// from another within the space they span: any rotation of them is an equally
/// correct answer. The two paths still agree exactly on how much energy there
/// is and on which space it occupies, while individual columns are compared
/// only loosely, because insisting they coincide would be demanding an answer
/// the mathematics does not define.
///
/// ´claim:svd:an-equal-valued-spectrum-fixes-the-space-but-not-the-axes-chosen-within-it´
/// ´test:unit:equivalence-equal-singular-values´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_equal_singular_values() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(404);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![5.0; cap]; // Degenerate spectrum — all σ identical.

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    // Looser basis tolerance: degenerate spectrum makes basis column
    // orientation ambiguous within the equal-σ subspace.
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-4, "equal-sigmas");
}

/// Forgetting enters the step only as a scale factor on the remembered
/// energy, so turning it off is the ordinary case with that factor at one, not
/// a separate code path. A cell configured to weight all its history equally
/// therefore evolves by the same arithmetic as one that discounts the past,
/// and the two paths agree there as everywhere else.
///
/// ´claim:svd:forgetting-is-only-a-scale-factor-so-switching-it-off-is-the-ordinary-step-with-that-factor-at-one´
/// ´test:unit:equivalence-no-forgetting´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn equivalence_no_forgetting() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 1.0; // λ = 1.0 → no exponential forgetting.
    let mut rng = SmallRng::seed_from_u64(505);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![8.0, 4.0, 2.0, 1.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let (naive, brand) = run_both(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap);
    let brand = brand.expect("Brand should succeed when c < d");
    assert_updates_close(&naive, &brand, 1e-8, 1.0 - 1e-6, "no-forgetting");
}

// ════════════════════════════════════════════════════════════
// § 4 — Properties (invariants)
// ════════════════════════════════════════════════════════════

/// Every step returns axes that are unit length and mutually perpendicular,
/// from either path. Everything downstream assumes it: a batch's coordinates
/// are obtained by projecting onto these axes, and the unexplained part is the
/// batch minus that projection, which is only a decomposition if the axes are
/// orthonormal. The incremental path has to work for this — its back-transform
/// inherits drift from the basis it started with, so it re-orthogonalises
/// before returning rather than trusting the construction.
///
/// ´claim:svd:every-step-returns-orthonormal-axes-so-a-projection-remains-a-decomposition´
/// ´test:unit:output-basis-is-orthonormal´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn output_basis_is_orthonormal() {
    let d = 128;
    let k = 4;
    let b = 16;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(2024);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![8.0, 4.0, 2.0, 1.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let naive = naive_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).expect("naïve should not fail");
    let brand = brand_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).expect("brand should not fail");

    assert_orthonormal(&naive.basis, naive.n, 1e-10, "naïve orthonormality");
    assert_orthonormal(&brand.basis, brand.n, 1e-10, "brand orthonormality");
}

/// Singular values come back non-negative and in descending order from both
/// paths. Order is what makes position meaningful: rank adaptation walks the
/// values accumulating energy until a threshold is met and takes that position
/// as the rank, which is only a sensible rule if the strongest direction is
/// first. The incremental path preserves the ordering through its
/// back-transform and corrective step rather than inheriting it by luck.
///
/// ´claim:svd:singular-values-come-back-non-negative-and-in-descending-order-so-position-means-strength´
/// ´test:unit:singular-values-sorted-and-nonnegative´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn singular_values_sorted_and_nonnegative() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let sqrt_lambda = 0.95_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(7777);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![12.0, 6.0, 3.0, 1.5];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    for (label, result) in [
        (
            "naïve",
            naive_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap(),
        ),
        (
            "brand",
            brand_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap(),
        ),
    ] {
        for (i, &s) in result.sigmas.iter().enumerate() {
            assert!(s >= 0.0, "{label}: σ[{i}] = {s} is negative");
        }
        for i in 1..result.n {
            assert!(
                result.sigmas[i - 1] >= result.sigmas[i] - 1e-12,
                "{label}: σ not decreasing: σ[{}]={}, σ[{i}]={}",
                i - 1,
                result.sigmas[i - 1],
                result.sigmas[i]
            );
        }
    }
}

/// Given the same inputs, either path returns the same axes and the same
/// values bit for bit — not close, identical. Nothing in the step draws on
/// randomness, iteration order or timing, so two sentinels fed the same
/// traffic hold the same model, and a difference between them is always
/// evidence about the traffic rather than about the machine.
///
/// ´claim:svd:repeating-a-step-on-the-same-input-reproduces-it-bit-for-bit´
/// ´test:unit:deterministic-reproduction´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn deterministic_reproduction() {
    let d = 128;
    let k = 2;
    let b = 16;
    let cap = 2;
    let sqrt_lambda = 0.99_f64.sqrt();

    // Run twice with same seed.
    for strategy in ["naive", "brand"] {
        let mut results = Vec::new();
        for _ in 0..2 {
            let mut rng = SmallRng::seed_from_u64(42);
            let basis = random_basis(d, cap, &mut rng);
            let sigmas = vec![5.0, 2.0];
            let x = random_matrix(b, d, &mut rng);
            let (z, residual) = phase1_projection(&x, &basis, k);

            let result = match strategy {
                "naive" => naive_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap(),
                "brand" => brand_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap(),
                _ => unreachable!(),
            };
            results.push(result);
        }

        // Exact bit-for-bit equality — deterministic SVD produces identical
        // floating-point values, so bitwise comparison is intentional.
        {
            let a = &results[0];
            let b_r = &results[1];
            assert_eq!(a.n, b_r.n, "{strategy}: n mismatch");
            for i in 0..a.n {
                assert_eq!(a.sigmas[i], b_r.sigmas[i], "{strategy}: σ[{i}] not bitwise equal");
                for j in 0..d {
                    assert_eq!(
                        a.basis[(j, i)],
                        b_r.basis[(j, i)],
                        "{strategy}: basis[{j},{i}] not bitwise equal"
                    );
                }
            }
        } // #[allow(clippy::float_cmp)]
    }
}

// ════════════════════════════════════════════════════════════
// § 5 — Functional / integration
// ════════════════════════════════════════════════════════════

/// Agreement between the two paths is a property of a whole streaming run, not
/// of one step in isolation. Each path is fed the same sequence but carries its
/// own state forward, so any difference compounds through every later step —
/// and over a long run the difference stays within a margin that grows only in
/// step with the number of steps taken. Divergence is linear rather than
/// explosive, which is what makes a cell that has been running for hours as
/// trustworthy as one that has just started.
///
/// ´claim:svd:agreement-survives-a-long-streaming-run-with-divergence-growing-no-faster-than-the-step-count´
/// ´test:unit:equivalence-multi-step´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
// i32→f64 cast is for the growing FP-error tolerance: 1e-6 × (step + 1.0).
fn equivalence_multi_step() {
    let d = 64;
    let k = 4;
    let b = 8;
    let cap = 4;
    let lambda: f64 = 0.99;
    let sqrt_lambda = lambda.sqrt();
    let steps = 50;
    let mut rng = SmallRng::seed_from_u64(1337);

    // Shared initial state.
    let mut naive_basis = random_basis(d, cap, &mut rng);
    let mut naive_sigmas = vec![5.0, 3.0, 1.0, 0.5];
    let mut brand_basis = naive_basis.clone();
    let mut brand_sigmas = naive_sigmas.clone();

    for step in 0..steps {
        let x = random_matrix(b, d, &mut rng);

        // Project against naïve state (both should be identical).
        let (z_n, res_n) = phase1_projection(&x, &naive_basis, k);
        let (z_b, res_b) = phase1_projection(&x, &brand_basis, k);

        let naive_result =
            naive_svd::evolve(&naive_basis, &naive_sigmas, &z_n, &res_n, sqrt_lambda, k, cap).expect("naïve should not fail");
        let brand_result =
            brand_svd::evolve(&brand_basis, &brand_sigmas, &z_b, &res_b, sqrt_lambda, k, cap).expect("brand should not fail");

        // Update state for next step.
        naive_basis = pad_to_cap(&naive_result.basis, d, cap);
        naive_sigmas = pad_sigmas(&naive_result.sigmas, cap);
        brand_basis = pad_to_cap(&brand_result.basis, d, cap);
        brand_sigmas = pad_sigmas(&brand_result.sigmas, cap);

        // Allow growing tolerance as FP differences accumulate.
        let sigma_tol = 1e-6 * (f64::from(step) + 1.0);
        // Start from (1.0 − 1e-4) so the check is satisfiable at step 0
        // (|cos| ≤ 1.0 exactly for unit vectors, so "> 1.0" always fails).
        let basis_tol = f64::from(step + 1).mul_add(-1e-4, 1.0);
        let label = format!("multi-step {step}");

        assert_updates_close(&naive_result, &brand_result, sigma_tol, basis_tol, &label);
    }
}

/// The two paths do not merely agree on coordinates they were given; they
/// explain data they have never seen equally well. Held-out rows projected
/// onto either model leave the same amount unaccounted for, which is the
/// property the sentinel actually depends on — novelty is measured from
/// exactly that leftover, so equal reconstruction means equal scores whichever
/// path produced the axes.
///
/// ´claim:svd:the-two-paths-explain-unseen-data-equally-well-not-merely-agree-on-the-numbers-they-were-handed´
/// ´test:unit:reconstruction-error-equivalence´
#[test]
// Linear algebra test — d, k, b, z, x follow standard mathematical notation.
#[allow(clippy::many_single_char_names)]
fn reconstruction_error_equivalence() {
    let d = 128;
    let k = 4;
    let b = 16;
    let cap = 4;
    let sqrt_lambda = 0.99_f64.sqrt();
    let mut rng = SmallRng::seed_from_u64(2025);

    let basis = random_basis(d, cap, &mut rng);
    let sigmas = vec![8.0, 4.0, 2.0, 1.0];

    let x = random_matrix(b, d, &mut rng);
    let (z, residual) = phase1_projection(&x, &basis, k);

    let naive = naive_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap();
    let brand = brand_svd::evolve(&basis, &sigmas, &z, &residual, sqrt_lambda, k, cap).unwrap();

    // Project test data onto both new bases and measure reconstruction error.
    let test_x = random_matrix(b, d, &mut rng);

    let naive_err = recon_error(&test_x, &naive.basis, naive.n);
    let brand_err = recon_error(&test_x, &brand.basis, brand.n);

    let rel = (naive_err - brand_err).abs() / naive_err.max(1e-15);
    assert!(
        rel < 1e-6,
        "reconstruction error diverged: naïve={naive_err:.8e}, brand={brand_err:.8e}, rel={rel:.2e}"
    );
}
