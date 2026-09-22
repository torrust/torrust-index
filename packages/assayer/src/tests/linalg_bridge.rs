// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cholesky_spd_succeeds`] | linalg | A genuinely positive definite matrix — here `AᵀA` with a positive shift on the diagonal — factors on the plain path, without the caller needing to reach for regularisation. The ordinary case stays ordinary: the expensive second phase exists for matrices that have actually gone wrong. |
//! | [`cholesky_non_spd_fails`] | linalg | When factorisation cannot proceed it names the offending pivot rather than merely refusing: an otherwise-identity matrix with a single negative diagonal fails at exactly that index. The index is the diagnostic — it tells the caller which dimension of the model went indefinite, which is what turns a failure into something actionable rather than a dead end. |
//! | [`cholesky_identity`] | linalg | The factor really is a lower-triangular factor of the input and not some other decomposition the backend might return: for the identity, every entry of L is the identity's own, zeros above the diagonal included. That is the anchor point which makes the solves below interpretable. |
//! | [`cholesky_inverse_clean`] | linalg | A `Clean` verdict is a claim about the numbers, not just about which code path ran: at p = 50 the returned inverse multiplied back against the original leaves a Frobenius residual from the identity below 1e-10. The caller can take an unqualified inverse at face value without measuring it again for itself. |
//! | [`a_barely_indefinite_matrix_is_refused`] | linalg | A matrix that is barely indefinite — one diagonal at −1e-8 among ones — is refused rather than absorbed. The retired second phase absorbed it: a shift of a thousandth swamps a pivot of minus a hundred-millionth, so the factorisation succeeded and the caller was handed the inverse of a matrix it had not supplied, under a variant it had to interrogate to notice. A refusal says the same thing without the interrogation, and the matrix that reaches this call is one the model has already floored, so a refusal here is a fault on the maintenance path rather than the routine event it used to be. |
//! | [`cholesky_inverse_failed`] | linalg | The refusal is honest and it names its pivot: an entirely negative diagonal produces a failure carrying the pivot index rather than an inverse of some kind. A matrix this far from positive definite is not a numerical wobble to be smoothed over, and the caller is told so instead of being handed a fabricated result. |
//! | [`eigenvalue_min_max_diagonal_spectrum`] | linalg | The spectrum of a diagonal matrix is its diagonal, and the pair is returned largest first: `diag(3, 1, 7)` answers `(7, 1)`. The ordering is the fact worth pinning, because it is the reverse of what the name reads as and the same order its diagonal counterpart uses — a caller that swapped the two would compare a matrix's largest eigenvalue against zero and find every matrix healthy. |
//! | [`eigenvalue_min_max_parts_from_the_diagonal`] | linalg | The spectral test and the diagonal test give different answers, and here is a matrix on which they disagree: `[[1, 2], [2, 1]]` has every diagonal entry strictly positive and eigenvalues 3 and −1. A positive minimum diagonal is necessary for positive definiteness and is not sufficient for it, so a check reading the diagonal admits this matrix and a check reading the spectrum refuses it (´req:gaussian:positive-definiteness´). Both halves are asserted here rather than only the second, because a witness that failed the diagonal test would witness nothing — the whole content of the requirement is that the cheap test says yes exactly where the real one says no. |
//! | [`extract_subvector_correct`] | linalg | Subvector extraction gathers exactly the entries named, in the order they are named, and nothing else: indices 1 and 3 of a four-element vector come back as a two-element vector holding those two values. Marginalising the mean means keeping a chosen subset of dimensions, so an off-by-one or a reordering here would quietly reassign each surviving dimension's value to its neighbour. |
//! | [`extract_subvector_empty`] | linalg | cites (´claim:linalg:subvector-extraction-takes-the-named-entries-in-the-order-named´) |
//! | [`schur_half_solve_identity`] | linalg | The half-solve is forward substitution against the lower factor alone, so a unit factor returns the right-hand side untouched — every column of it, since the solve handles a whole block at once rather than one vector at a time. That the operation is a no-op precisely when L is the identity is what distinguishes it from a full solve, which would also have to undo Lᵀ. |
//! | [`schur_half_solve_known`] | linalg | The half-solve returns the W that genuinely satisfies `L W = rhs`, checked against a system small enough to solve by hand: for A = [[4, 2], [2, 3]] the factor is L = [[2, 0], [1, √2]], and against [8, 7]ᵀ forward substitution gives w₀ = 4 and then w₁ = 3/√2. Both come back to within 1e-14. The Schur correction is formed as WᵀW, which is positive semi-definite by construction only if W really is this half-solve and not a full one. |
//! | [`cholesky_full_solve_known`] | linalg | The full solve goes the whole way and returns the x satisfying `A x = b`, not the intermediate the half-solve stops at: on the same hand-checkable system A = [[4, 2], [2, 3]] against [8, 7]ᵀ it yields [1.25, 1.5]ᵀ. Since this is the fallback method and the debug oracle both, it has to be an independent answer rather than a rearrangement of the half-solve's. |
//! | [`cholesky_inverse_contexts`] | linalg | The call-site tag is diagnostic routing and nothing more: the same positive definite matrix inverts cleanly whether it arrives from periodic recomputation, from marginalisation or from a revert. Numerical behaviour does not fork by caller, so a failure seen at one site is a property of the matrix rather than of the path it came in on. |
//! | [`release_oracle_is_absent`] | linalg | In a release build the Schur oracle is absent, and raising the log level does not bring it back: with a `DEBUG` subscriber installed, a correction wildly unlike the full-solve route passes through it without a panic. The oracle ends in an assertion, which is a panic in every build profile, and raising the log level is a supported operational action — so an oracle a subscriber could switch on would turn a numerical disagreement into a process abort on a deregistration, in a package whose posture is that numerical pathologies are retained and flagged rather than failed. The test exists only in the profile where the question can be asked; in a debug build the oracle is meant to fire. |
//! | [`parallelism_width_sweep`] | linalg | The width at which spreading a dense factorisation across worker threads starts to pay, measured rather than assumed: the factorisation and the inverse from its factor are timed at both settings across the widths this package's matrices reach, and the wall of each is printed beside the ratio. The work is cubic in the width while the dispatch is not, so there is a width below which the dispatch costs more than the work it spreads, and the threshold the bridge decides on is read off this table. Ignored by default because it is a measurement and not an assertion. |
//! | [`oracle_admits_two_ulp_at_the_tripping_magnitude`] | linalg | The oracle's tolerance is the arithmetic's own error at the magnitude compared, so it holds where a fixed distance cannot: a correction near 5.09e3 whose two forms stand two ulp apart passes. One ulp there is about 9.1e-13, so the two forms differ by more than the 1e-12 the oracle once asked for absolutely — a distance narrower than a single representable step at that magnitude, which no correct arithmetic can meet. The scan that found this drove the marginalisation path until its corrections reached that size, and the oracle aborted on agreement to the last bit the format carries. |
//! | [`oracle_refuses_a_relative_divergence`] | linalg | Widening the tolerance to the arithmetic's own error does not disarm the oracle: at the same magnitude, two forms differing by a billionth of it still abort. What a debug oracle exists to catch is an algorithmic divergence — a transposed block, a solve against the wrong factor, the wrong operand — and that moves an entry by a share of its own size rather than by its last bits, so a bound written in ulp keeps every divergence worth reporting while admitting the round-off that is not one. |
//! | [`oracle_admits_the_rounding_a_floored_factor_produces`] | linalg | The two correct routes disagree at the floored geometry by more than the retired bound allowed, and the derived bound admits them: a removed block whose near-collinear features leave every eigenvalue but the leading one at the replenishment floor, an eight-wide factor and two hundred kept dimensions. The witness is carried rather than assumed — the worst entry's disagreement is measured against the retired 2γ_r\|x\| + 1e-12 and asserted to exceed it — and the same entries are then measured against the derived bound and admitted, because that bound carries the conditioning of the substitution both routes perform and the retired form dropped. The margin is asserted to be a real one rather than a hair, so a bound that only just covered this geometry would fail here too. |
//! | [`oracle_refuses_a_solve_against_the_wrong_factor_at_the_floor`] | linalg | A bound wide enough for the floored factor's rounding is not wide enough for a solve against the wrong factor: at the same geometry, a half-solve taken against the block regularised a hundred times more heavily still aborts. The two blocks differ by a hundredth on the diagonal alone and every eigenvalue but the leading one is at the floor, so the correction they imply differs by a share of its own size — which is the scale of divergence the oracle exists to report, and it stands orders above the conditioning term that admits the rounding. |
//! | [`oracle_refuses_the_wrong_operand_at_the_floor`] | linalg | cites (´claim:linalg:the-derived-bound-still-refuses-a-faulted-route-at-the-floor´) |
//! | [`oracle_refuses_the_coupling_block_standing_for_the_half_solve`] | linalg | cites (´claim:linalg:the-derived-bound-still-refuses-a-faulted-route-at-the-floor´) |
//! | [`the_storage_boundary_admits_an_indefinite_matrix_and_the_factorisation_refuses_it`] | linalg | The witness the self-guarding study probed with is refused, and no device carries it. A matrix of ones on the diagonal and twos off it is symmetric to the bit and indefinite by a whole eigenvalue, and it crosses the storage boundary untouched, because that boundary's only verdict is symmetry. What the study found beyond the boundary was a success: the plain factorisation was refused, and the retired second phase answered anyway — not by its diagonal shift, since a shift of a thousandth cannot lift an eigenvalue of minus one, but by the per-pivot backstop beneath it, which replaced the offending pivot and returned an inverse of neither the matrix supplied nor the matrix shifted. With one phase there is nothing to carry it: the refusal is the answer, and it names the pivot it stopped at. The storage boundary still admits the matrix, which is why the entries that install a persisted matrix take a factorisation's verdict of their own. |

//! Crate-level tests for `linalg::bridge`.
//!
//! The bridge is the crate's only doorway to the `faer` backend. What these
//! tests pin down is the behaviour the rest of the crate is allowed to rely
//! on: what a factorisation does when the matrix is not quite positive
//! definite, what the two-phase inverse cascade reports about the rescue it
//! performed, and that the solves return the vectors the equations define
//! rather than whatever the backend happens to hand back.

// The width sweep below is a measurement, and a measurement whose numbers
// nobody can read is not one.
#![allow(clippy::cast_precision_loss, clippy::suboptimal_flops, clippy::print_stdout)]

use faer::{Col, Mat};

use crate::linalg::bridge::{
    CholeskyCallSite, CholeskyInverseResult, cholesky, cholesky_full_solve, cholesky_inverse, eigenvalue_min_max,
    extract_subvector, schur_half_solve,
};
// The oracle and the quantities it is assembled from exist only where the
// oracle does, so the tests that measure it import them under the same gate.
#[cfg(debug_assertions)]
use crate::linalg::bridge::{CholeskyFactor, schur_oracle_sums, schur_oracle_tolerance};
use crate::linalg::symmetric::SymmetricMatrix;
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_near};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Generate a random SPD matrix using the shared [`TestRng`].
fn random_spd(p: usize, seed: u64) -> SymmetricMatrix {
    let mut rng = TestRng::new(seed);
    // Uniform in [-1, 1).
    let r = Mat::from_fn(p, p, |_, _| rng.next_f64().mul_add(2.0, -1.0));
    let mut a = r.transpose() * &r;
    for i in 0..p {
        a[(i, i)] += 0.1;
    }
    SymmetricMatrix::from_computation(a)
}

/// Frobenius norm of (M * N - I).
fn frobenius_deviation_from_identity(m: &SymmetricMatrix, n: &SymmetricMatrix) -> f64 {
    let p = m.dim();
    assert_eq!(p, n.dim());
    let product = m.as_inner() * n.as_inner();
    let mut sum_sq = 0.0;
    for j in 0..p {
        for i in 0..p {
            let expected = if i == j { 1.0 } else { 0.0 };
            let diff = product[(i, j)] - expected;
            sum_sq = diff.mul_add(diff, sum_sq);
        }
    }
    sum_sq.sqrt()
}

// ─────────────────────────────────────────────────────────────────────────────
// Tests
// ─────────────────────────────────────────────────────────────────────────────

/// A genuinely positive definite matrix — here `AᵀA` with a positive shift on
/// the diagonal — factors on the plain path, without the caller needing to
/// reach for regularisation. The ordinary case stays ordinary: the expensive
/// second phase exists for matrices that have actually gone wrong.
///
/// ´claim:linalg:a-positive-definite-matrix-factors-on-the-plain-path´
/// ´test:crate:cholesky-spd-succeeds´
#[test]
fn cholesky_spd_succeeds() {
    let m = random_spd(5, 0);
    assert!(cholesky(&m).is_ok());
}

/// When factorisation cannot proceed it names the offending pivot rather than
/// merely refusing: an otherwise-identity matrix with a single negative
/// diagonal fails at exactly that index. The index is the diagnostic — it
/// tells the caller which dimension of the model went indefinite, which is
/// what turns a failure into something actionable rather than a dead end.
///
/// ´claim:linalg:a-failed-factorisation-names-the-index-of-the-offending-pivot´
/// ´test:crate:cholesky-non-spd-fails´
#[test]
fn cholesky_non_spd_fails() {
    use faer::linalg::cholesky::llt::factor::LltError;

    // Matrix with a negative eigenvalue
    let mut mat = Mat::zeros(5, 5);
    for i in 0..5 {
        mat[(i, i)] = 1.0;
    }
    mat[(1, 1)] = -1.0;
    let m = SymmetricMatrix::from_computation(mat);
    match cholesky(&m) {
        Err(LltError::NonPositivePivot { index }) => {
            assert_eq!(index, 1);
        }
        Ok(_) => panic!("expected failure for non-SPD matrix"),
    }
}

/// The factor really is a lower-triangular factor of the input and not some
/// other decomposition the backend might return: for the identity, every
/// entry of L is the identity's own, zeros above the diagonal included. That
/// is the anchor point which makes the solves below interpretable.
///
/// ´claim:linalg:factoring-the-identity-yields-the-identity-as-its-lower-factor´
/// ´test:crate:cholesky-identity´
#[test]
fn cholesky_identity() {
    let m = SymmetricMatrix::identity_scaled(5, 1.0);
    let llt = cholesky(&m).unwrap();
    // L should be identity
    let l = llt.lower();
    for j in 0..5 {
        for i in 0..5 {
            let expected = if i == j { 1.0 } else { 0.0 };
            assert_near(l[(i, j)], expected, 1e-15, &format!("L[{i},{j}]"));
        }
    }
}

/// A `Clean` verdict is a claim about the numbers, not just about which code
/// path ran: at p = 50 the returned inverse multiplied back against the
/// original leaves a Frobenius residual from the identity below 1e-10. The
/// caller can take an unqualified inverse at face value without measuring it
/// again for itself.
///
/// ´claim:linalg:a-clean-inverse-multiplies-back-to-the-identity-within-round-off´
/// ´test:crate:cholesky-inverse-clean´
#[test]
fn cholesky_inverse_clean() {
    let m = random_spd(50, 100);
    match cholesky_inverse(&m, CholeskyCallSite::Periodic) {
        CholeskyInverseResult::Clean(inv) => {
            let dev = frobenius_deviation_from_identity(&m, &inv);
            assert!(dev < 1e-10, "||M * M^-1 - I||_F = {dev:e} (expected < 1e-10)");
        }
        CholeskyInverseResult::Failed { pivot } => panic!("expected Clean, got a refusal at pivot {pivot}"),
    }
}

/// A matrix that is barely indefinite — one diagonal at −1e-8 among ones — is refused rather than absorbed. The retired second phase absorbed it: a shift of a thousandth swamps a pivot of minus a hundred-millionth, so the factorisation succeeded and the caller was handed the inverse of a matrix it had not supplied, under a variant it had to interrogate to notice. A refusal says the same thing without the interrogation, and the matrix that reaches this call is one the model has already floored, so a refusal here is a fault on the maintenance path rather than the routine event it used to be.
///
/// ´claim:linalg:a-barely-indefinite-matrix-is-refused-rather-than-absorbed-by-a-shift´
/// ´test:crate:a-barely-indefinite-matrix-is-refused´
#[test]
fn a_barely_indefinite_matrix_is_refused() {
    let mut mat = Mat::zeros(10, 10);
    for i in 0..10 {
        mat[(i, i)] = 1.0;
    }
    // One diagonal entry small and negative: far too small for the retired
    // shift to have noticed, and negative all the same.
    mat[(5, 5)] = -1e-8;
    let m = SymmetricMatrix::from_computation(mat);
    match cholesky_inverse(&m, CholeskyCallSite::Marginalisation) {
        CholeskyInverseResult::Failed { pivot } => {
            assert_eq!(pivot, 5, "the refusal names the pivot that is negative");
        }
        CholeskyInverseResult::Clean(_) => {
            panic!("a negative pivot must be refused, not carried");
        }
    }
}

/// The refusal is honest and it names its pivot: an entirely negative diagonal produces a failure carrying the pivot index rather than an inverse of some kind. A matrix this far from positive definite is not a numerical wobble to be smoothed over, and the caller is told so instead of being handed a fabricated result.
///
/// ´claim:linalg:a-hopeless-matrix-fails-with-its-pivot-named´
/// ´test:crate:cholesky-inverse-failed´
#[test]
fn cholesky_inverse_failed() {
    let mut mat = Mat::zeros(5, 5);
    for i in 0..5 {
        mat[(i, i)] = -1.0;
    }
    let m = SymmetricMatrix::from_computation(mat);
    match cholesky_inverse(&m, CholeskyCallSite::Revert) {
        CholeskyInverseResult::Failed { pivot } => {
            assert_eq!(pivot, 0, "the first pivot is already negative");
        }
        CholeskyInverseResult::Clean(_) => {
            panic!("expected failure for all-negative diagonal");
        }
    }
}

/// The spectrum of a diagonal matrix is its diagonal, and the pair is returned
/// largest first: `diag(3, 1, 7)` answers `(7, 1)`. The ordering is the fact
/// worth pinning, because it is the reverse of what the name reads as and the
/// same order its diagonal counterpart uses — a caller that swapped the two
/// would compare a matrix's largest eigenvalue against zero and find every
/// matrix healthy.
///
/// ´claim:linalg:the-spectral-extremes-are-reported-largest-first-then-smallest´
/// ´test:crate:eigenvalue-min-max-diagonal-spectrum´
#[test]
fn eigenvalue_min_max_diagonal_spectrum() {
    let mut mat = Mat::zeros(3, 3);
    mat[(0, 0)] = 3.0;
    mat[(1, 1)] = 1.0;
    mat[(2, 2)] = 7.0;
    let m = SymmetricMatrix::from_computation(mat);

    let (lambda_max, lambda_min) = eigenvalue_min_max(&m).expect("a diagonal matrix has a spectrum");
    assert_near(lambda_max, 7.0, DEFAULT_TOLERANCES.default, "largest eigenvalue");
    assert_near(lambda_min, 1.0, DEFAULT_TOLERANCES.default, "smallest eigenvalue");
}

/// The spectral test and the diagonal test give different answers, and here is
/// a matrix on which they disagree: `[[1, 2], [2, 1]]` has every diagonal entry
/// strictly positive and eigenvalues 3 and −1. A positive minimum diagonal is
/// necessary for positive definiteness and is not sufficient for it, so a check
/// reading the diagonal admits this matrix and a check reading the spectrum
/// refuses it (´req:gaussian:positive-definiteness´). Both halves are asserted
/// here rather than only the second, because a witness that failed the diagonal
/// test would witness nothing — the whole content of the requirement is that
/// the cheap test says yes exactly where the real one says no.
///
/// ´claim:linalg:a-strictly-positive-diagonal-does-not-make-the-spectrum-positive´
/// ´test:crate:eigenvalue-min-max-parts-from-the-diagonal´
#[test]
fn eigenvalue_min_max_parts_from_the_diagonal() {
    let mut mat = Mat::zeros(2, 2);
    mat[(0, 0)] = 1.0;
    mat[(1, 1)] = 1.0;
    mat[(0, 1)] = 2.0;
    mat[(1, 0)] = 2.0;
    let m = SymmetricMatrix::from_computation(mat);

    // The retired test admits it.
    let (_, min_diag) = m.diagonal_min_max();
    assert!(
        min_diag > 0.0,
        "the witness must pass the diagonal test, or it witnesses nothing (got {min_diag})"
    );

    // The requirement's test refuses it.
    let (lambda_max, lambda_min) = eigenvalue_min_max(&m).expect("a symmetric matrix has a spectrum");
    assert_near(lambda_max, 3.0, DEFAULT_TOLERANCES.default, "largest eigenvalue");
    assert_near(lambda_min, -1.0, DEFAULT_TOLERANCES.default, "smallest eigenvalue");
    assert!(lambda_min < 0.0, "the spectral test must refuse the witness");
}

/// Subvector extraction gathers exactly the entries named, in the order they
/// are named, and nothing else: indices 1 and 3 of a four-element vector come
/// back as a two-element vector holding those two values. Marginalising the
/// mean means keeping a chosen subset of dimensions, so an off-by-one or a
/// reordering here would quietly reassign each surviving dimension's value to
/// its neighbour.
///
/// ´claim:linalg:subvector-extraction-takes-the-named-entries-in-the-order-named´
/// ´test:crate:extract-subvector-correct´
#[test]
fn extract_subvector_correct() {
    let v = Col::from_fn(4, |i| (i + 1) as f64 * 10.0);
    let sub = extract_subvector(&v, &[1, 3]);
    assert_eq!(sub.nrows(), 2);
    assert_near(sub[0], 20.0, 1e-15, "sub[0]");
    assert_near(sub[1], 40.0, 1e-15, "sub[1]");
}

/// Naming no indices at all yields an empty vector rather than a panic or the
/// whole thing: keeping nothing is a legitimate request, so a marginalisation
/// that retains no dimensions needs no special-casing at the call site.
///
/// (´claim:linalg:subvector-extraction-takes-the-named-entries-in-the-order-named´)
/// ´test:crate:extract-subvector-empty´
#[test]
fn extract_subvector_empty() {
    let v = Col::from_fn(4, |i| i as f64);
    let sub = extract_subvector(&v, &[]);
    assert_eq!(sub.nrows(), 0);
}

/// The half-solve is forward substitution against the lower factor alone, so
/// a unit factor returns the right-hand side untouched — every column of it,
/// since the solve handles a whole block at once rather than one vector at a
/// time. That the operation is a no-op precisely when L is the identity is
/// what distinguishes it from a full solve, which would also have to undo Lᵀ.
///
/// ´claim:linalg:the-half-solve-against-a-unit-factor-returns-the-right-hand-side-unchanged´
/// ´test:crate:schur-half-solve-identity´
#[test]
fn schur_half_solve_identity() {
    let m = SymmetricMatrix::identity_scaled(3, 1.0);
    let llt = cholesky(&m).unwrap();
    let rhs = Mat::from_fn(3, 2, |i, j| ((i + 1) * (j + 1)) as f64);
    let result = schur_half_solve(&llt, &rhs);
    for j in 0..2 {
        for i in 0..3 {
            assert_near(result[(i, j)], rhs[(i, j)], 1e-14, &format!("solve[{i},{j}]"));
        }
    }
}

/// The half-solve returns the W that genuinely satisfies `L W = rhs`, checked
/// against a system small enough to solve by hand: for A = [[4, 2], [2, 3]]
/// the factor is L = [[2, 0], [1, √2]], and against [8, 7]ᵀ forward
/// substitution gives w₀ = 4 and then w₁ = 3/√2. Both come back to within
/// 1e-14. The Schur correction is formed as WᵀW, which is positive
/// semi-definite by construction only if W really is this half-solve and not
/// a full one.
///
/// ´claim:linalg:the-half-solve-returns-the-w-satisfying-l-w-equals-the-right-hand-side´
/// ´test:crate:schur-half-solve-known´
#[test]
fn schur_half_solve_known() {
    let mut mat = Mat::zeros(2, 2);
    mat[(0, 0)] = 4.0;
    mat[(0, 1)] = 2.0;
    mat[(1, 0)] = 2.0;
    mat[(1, 1)] = 3.0;
    let m = SymmetricMatrix::from_computation(mat);
    let llt = cholesky(&m).unwrap();

    let rhs = Mat::from_fn(2, 1, |i, _| if i == 0 { 8.0 } else { 7.0 });
    let w = schur_half_solve(&llt, &rhs);
    // L = [[2, 0], [1, √2]], so w₀ = 4.0, w₁ = (7-4)/√2 = 3/√2
    assert_near(w[(0, 0)], 4.0, 1e-14, "w[0]");
    let expected_w1 = 3.0 / 2.0_f64.sqrt();
    assert_near(w[(1, 0)], expected_w1, 1e-14, "w[1]");
}

/// The full solve goes the whole way and returns the x satisfying `A x = b`,
/// not the intermediate the half-solve stops at: on the same hand-checkable
/// system A = [[4, 2], [2, 3]] against [8, 7]ᵀ it yields [1.25, 1.5]ᵀ. Since
/// this is the fallback method and the debug oracle both, it has to be an
/// independent answer rather than a rearrangement of the half-solve's.
///
/// ´claim:linalg:the-full-solve-returns-the-x-satisfying-a-x-equals-the-right-hand-side´
/// ´test:crate:cholesky-full-solve-known´
#[test]
fn cholesky_full_solve_known() {
    let mut mat = Mat::zeros(2, 2);
    mat[(0, 0)] = 4.0;
    mat[(0, 1)] = 2.0;
    mat[(1, 0)] = 2.0;
    mat[(1, 1)] = 3.0;
    let m = SymmetricMatrix::from_computation(mat);
    let llt = cholesky(&m).unwrap();

    let rhs = Mat::from_fn(2, 1, |i, _| if i == 0 { 8.0 } else { 7.0 });
    let result = cholesky_full_solve(&llt, &rhs);
    assert_near(result[(0, 0)], 1.25, 1e-14, "x[0]");
    assert_near(result[(1, 0)], 1.5, 1e-14, "x[1]");
}

/// The call-site tag is diagnostic routing and nothing more: the same
/// positive definite matrix inverts cleanly whether it arrives from periodic
/// recomputation, from marginalisation or from a revert. Numerical behaviour
/// does not fork by caller, so a failure seen at one site is a property of
/// the matrix rather than of the path it came in on.
///
/// ´claim:linalg:the-call-site-tag-routes-diagnostics-without-changing-the-result´
/// ´test:crate:cholesky-inverse-contexts´
#[test]
fn cholesky_inverse_contexts() {
    let m = random_spd(5, 0);
    // All three call sites should work
    for site in [
        CholeskyCallSite::Periodic,
        CholeskyCallSite::Marginalisation,
        CholeskyCallSite::Revert,
    ] {
        match cholesky_inverse(&m, site) {
            CholeskyInverseResult::Clean(_) => {}
            CholeskyInverseResult::Failed { pivot } => panic!("expected Clean for SPD matrix, got a refusal at pivot {pivot}"),
        }
    }
}

/// In a release build the Schur oracle is absent, and raising the log level
/// does not bring it back: with a `DEBUG` subscriber installed, a correction
/// wildly unlike the full-solve route passes through it without a panic. The
/// oracle ends in an assertion, which is a panic in every build profile, and
/// raising the log level is a supported operational action — so an oracle a
/// subscriber could switch on would turn a numerical disagreement into a
/// process abort on a deregistration, in a package whose posture is that
/// numerical pathologies are retained and flagged rather than failed. The test
/// exists only in the profile where the question can be asked; in a debug
/// build the oracle is meant to fire.
///
/// ´claim:linalg:the-release-profile-has-no-oracle-for-a-subscriber-to-switch-on´
/// ´test:crate:release-oracle-is-absent´
#[cfg(not(debug_assertions))]
#[test]
fn release_oracle_is_absent() {
    use tracing_subscriber::util::SubscriberInitExt as _;

    let _guard = tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .finish()
        .set_default();
    assert!(
        tracing::enabled!(tracing::Level::DEBUG),
        "the subscriber should have made DEBUG enabled"
    );

    let m = SymmetricMatrix::identity_scaled(2, 1.0);
    let llt = cholesky(&m).unwrap();
    let b_rk = Mat::from_fn(2, 1, |i, _| if i == 0 { 1.0 } else { 2.0 });
    let b_kr = Mat::from_fn(1, 2, |_, j| if j == 0 { 1.0 } else { 2.0 });
    // Deliberately nothing like B_kr · Y: the oracle, were it here, would abort.
    let wrong = Mat::from_fn(1, 1, |_, _| 1.0e8);

    crate::linalg::bridge::schur_correction_oracle(&wrong, &llt, &b_rk, &b_kr);
}

/// The width at which spreading a dense factorisation across worker threads starts to pay, measured rather than assumed: the factorisation and the inverse from its factor are timed at both settings across the widths this package's matrices reach, and the wall of each is printed beside the ratio. The work is cubic in the width while the dispatch is not, so there is a width below which the dispatch costs more than the work it spreads, and the threshold the bridge decides on is read off this table. Ignored by default because it is a measurement and not an assertion.
///
/// ´claim:linalg:the-parallelism-threshold-is-read-off-a-width-sweep-of-the-two-operations´
/// ´test:crate:parallelism-width-sweep´
#[test]
#[ignore = "measurement: the width sweep the parallelism threshold is read off"]
fn parallelism_width_sweep() {
    use std::time::Instant;

    use faer::Par;

    /// Repeats at each width and setting; the least of them is reported,
    /// because a sweep on a shared box measures interference otherwise.
    const REPEATS: usize = 5;

    /// The plain factorisation at a stated parallelism, timed.
    fn factor_once(mat: &SymmetricMatrix, par: Par) -> f64 {
        use faer::Spec;
        use faer::linalg::cholesky::llt::factor::{LltParams, LltRegularization, cholesky_in_place, cholesky_in_place_scratch};

        let p = mat.dim();
        let scratch = cholesky_in_place_scratch::<f64>(p, par, Spec::<LltParams, f64>::default());
        let mut mem = faer::dyn_stack::MemBuffer::new(scratch);
        let stack = faer::dyn_stack::MemStack::new(&mut mem);
        let mut work = mat.as_inner().clone();
        let regularization = LltRegularization {
            dynamic_regularization_delta: 0.0,
            dynamic_regularization_epsilon: 0.0,
        };
        let started = Instant::now();
        let outcome = cholesky_in_place(work.as_mut(), regularization, par, stack, Spec::<LltParams, f64>::default());
        let elapsed = started.elapsed().as_secs_f64();
        assert!(outcome.is_ok(), "the sweep's matrices are positive definite");
        elapsed
    }

    /// The inverse from a lower factor at a stated parallelism, timed.
    fn invert_once(l_factor: &Mat<f64>, par: Par) -> f64 {
        use faer::linalg::cholesky::llt::inverse::{inverse, inverse_scratch};

        let p = l_factor.nrows();
        let scratch = inverse_scratch::<f64>(p, par);
        let mut mem = faer::dyn_stack::MemBuffer::new(scratch);
        let stack = faer::dyn_stack::MemStack::new(&mut mem);
        let mut inv = Mat::zeros(p, p);
        let started = Instant::now();
        inverse(inv.as_mut(), l_factor.as_ref(), par, stack);
        started.elapsed().as_secs_f64()
    }

    println!("width  factor-seq  factor-par  ratio | inverse-seq  inverse-par  ratio");
    for p in [8_usize, 16, 32, 48, 64, 96, 128, 192, 256, 384, 512, 768, 1024] {
        let mat = random_spd(p, 20_260_902);
        let llt = cholesky(&mat).expect("the sweep's matrices are positive definite");
        let l_factor = llt.lower().to_owned();

        let best = |mut run: Box<dyn FnMut(Par) -> f64>, par: Par| -> f64 {
            (0..REPEATS).map(|_| run(par)).fold(f64::INFINITY, f64::min)
        };
        let factor_seq = best(Box::new(|par| factor_once(&mat, par)), Par::Seq);
        let factor_par = best(Box::new(|par| factor_once(&mat, par)), Par::rayon(0));
        let inverse_seq = best(Box::new(|par| invert_once(&l_factor, par)), Par::Seq);
        let inverse_par = best(Box::new(|par| invert_once(&l_factor, par)), Par::rayon(0));

        println!(
            "{p:5}  {factor_seq:10.6}  {factor_par:10.6}  {:5.2} | {inverse_seq:11.6}  {inverse_par:11.6}  {:5.2}",
            factor_seq / factor_par,
            inverse_seq / inverse_par,
        );
    }
}

/// The oracle's tolerance is the arithmetic's own error at the magnitude compared, so it holds where a fixed distance cannot: a correction near 5.09e3 whose two forms stand two ulp apart passes. One ulp there is about 9.1e-13, so the two forms differ by more than the 1e-12 the oracle once asked for absolutely — a distance narrower than a single representable step at that magnitude, which no correct arithmetic can meet. The scan that found this drove the marginalisation path until its corrections reached that size, and the oracle aborted on agreement to the last bit the format carries.
///
/// ´claim:linalg:the-oracle-holds-where-a-fixed-distance-falls-under-one-ulp´
/// ´test:crate:oracle-admits-two-ulp-at-the-tripping-magnitude´
#[cfg(debug_assertions)]
#[test]
fn oracle_admits_two_ulp_at_the_tripping_magnitude() {
    /// The retained order both forms sum over.
    const TERMS: usize = 8;
    /// The magnitude the oracle tripped at.
    const TRIPPED_AT: f64 = 5.09e3;

    // A column of this against its own transpose sums to the tripping
    // magnitude over the identity's trivial factor.
    let entry = (TRIPPED_AT / TERMS as f64).sqrt();
    let m = SymmetricMatrix::identity_scaled(TERMS, 1.0);
    let llt = cholesky(&m).unwrap();
    let b_rk = Mat::from_fn(TERMS, 1, |_, _| entry);
    let b_kr = Mat::from_fn(1, TERMS, |_, _| entry);

    let full = (&b_kr * &cholesky_full_solve(&llt, &b_rk))[(0, 0)];
    assert!(
        (full - TRIPPED_AT).abs() < 1.0,
        "the correction lands at the tripping magnitude: {full}"
    );

    // Two ulp above the full-solve's own answer: the disagreement the scan
    // reported, exhibited exactly rather than hoped for from the backend.
    let half_value = f64::from_bits(full.to_bits() + 2);
    let diff = half_value - full;
    assert!(
        diff > 1e-12,
        "two ulp at this magnitude is wider than the retired fixed bound: {diff:.3e}"
    );

    let half = Mat::from_fn(1, 1, |_, _| half_value);
    crate::linalg::bridge::schur_correction_oracle(&half, &llt, &b_rk, &b_kr);
}

/// Widening the tolerance to the arithmetic's own error does not disarm the oracle: at the same magnitude, two forms differing by a billionth of it still abort. What a debug oracle exists to catch is an algorithmic divergence — a transposed block, a solve against the wrong factor, the wrong operand — and that moves an entry by a share of its own size rather than by its last bits, so a bound written in ulp keeps every divergence worth reporting while admitting the round-off that is not one.
///
/// ´claim:linalg:a-divergence-of-a-share-of-the-magnitude-still-aborts´
/// ´test:crate:oracle-refuses-a-relative-divergence´
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Schur oracle")]
fn oracle_refuses_a_relative_divergence() {
    /// The retained order both forms sum over.
    const TERMS: usize = 8;
    /// The magnitude the oracle tripped at.
    const TRIPPED_AT: f64 = 5.09e3;

    let entry = (TRIPPED_AT / TERMS as f64).sqrt();
    let m = SymmetricMatrix::identity_scaled(TERMS, 1.0);
    let llt = cholesky(&m).unwrap();
    let b_rk = Mat::from_fn(TERMS, 1, |_, _| entry);
    let b_kr = Mat::from_fn(1, TERMS, |_, _| entry);

    let full = (&b_kr * &cholesky_full_solve(&llt, &b_rk))[(0, 0)];
    let half = Mat::from_fn(1, 1, |_, _| full.mul_add(1e-9, full));

    crate::linalg::bridge::schur_correction_oracle(&half, &llt, &b_rk, &b_kr);
}

// ─────────────────────────────────────────────────────────────────────────────
// The floored geometry the oracle is audited at
// ─────────────────────────────────────────────────────────────────────────────

/// The removed block, the coupling and the factor of the geometry the oracle
/// was found failing in: a block whose features are near-collinear, so the
/// replenishment floor is what holds every eigenvalue but the leading one up,
/// and a coupling into two hundred kept dimensions.
///
/// The factor's order is the small one — eight — because the factor the oracle
/// solves against is the removed block's and not the model's width, which is
/// what the recorded failures were taken at.
#[cfg(debug_assertions)]
fn floored_schur_geometry() -> (CholeskyFactor, Mat<f64>, Mat<f64>) {
    /// The removed block's order, which is the factor's order and the number
    /// of terms both routes sum over.
    const R: usize = 8;
    /// The retained width, which is the correction's own.
    const K: usize = 200;
    /// The replenishment floor, where the block's smallest eigenvalues rest.
    const FLOOR: f64 = 1e-3;
    /// Rows of the feature matrix whose Gram the block is.
    const OBSERVATIONS: usize = 24;

    // One shared direction, each column nudged off it by a thousandth: the
    // Gram is rank one to within the nudge, whose own scale is well below the
    // floor, so the floor is the smallest eigenvalue and the smallest pivot.
    let features = Mat::from_fn(OBSERVATIONS, R, |i, j| {
        let shared = (i as f64).mul_add(0.125, 3.5);
        ((i + j) % 3) as f64 * 1e-3 + shared
    });
    let block = SymmetricMatrix::from_computation(features.transpose() * &features).add_scaled_identity(FLOOR);
    let llt = cholesky(&block).expect("the floored block factors");

    let mut rng = TestRng::new(0x5c47_b0d1);
    let b_rk = Mat::from_fn(R, K, |_, _| rng.next_f64().mul_add(20.0, -10.0));
    let b_kr = b_rk.transpose().to_owned();
    (llt, b_rk, b_kr)
}

/// The tolerance the retired form asked for, $2\gamma_r|x| + 10^{-12}$, kept
/// here as the witness's yardstick and nowhere else: what makes the geometry
/// below a witness is that the two correct routes stand further apart than
/// this.
#[cfg(debug_assertions)]
fn retired_oracle_tolerance(magnitude: f64, terms: usize) -> f64 {
    let unit_roundoff = f64::EPSILON / 2.0;
    let scaled = terms as f64 * unit_roundoff;
    let gamma = scaled / (1.0 - scaled);
    2.0_f64.mul_add(gamma * magnitude, 1e-12)
}

/// The entry of a comparison that came closest to the bound: what the larger
/// of the two routes said there, how far apart the two stood, and what the
/// bound allowed.
#[cfg(debug_assertions)]
struct WorstEntry {
    /// The larger of the two routes' magnitudes at that entry.
    magnitude: f64,
    /// How far apart the two routes stood there.
    diff: f64,
    /// What the bound allowed there.
    tolerance: f64,
}

#[cfg(debug_assertions)]
impl WorstEntry {
    /// The share of the bound the disagreement took up, one being the oracle's
    /// own verdict boundary.
    fn ratio(&self) -> f64 {
        self.diff / self.tolerance
    }

    /// The disagreement as a share of the entry's own magnitude, which is the
    /// scale a divergence has to be stated in to be compared across entries.
    fn relative_diff(&self) -> f64 {
        self.diff / self.magnitude
    }

    /// The bound as a share of the entry's own magnitude, which is the
    /// smallest relative divergence the oracle still refuses there.
    fn relative_tolerance(&self) -> f64 {
        self.tolerance / self.magnitude
    }
}

/// The entry at which a comparison came closest to its bound, over the whole
/// correction.
#[cfg(debug_assertions)]
fn worst_entry(half: &Mat<f64>, full: &Mat<f64>, bound: impl Fn(usize, usize, f64) -> f64) -> WorstEntry {
    let mut worst = WorstEntry {
        magnitude: 0.0,
        diff: 0.0,
        tolerance: 1.0,
    };
    for row in 0..half.nrows() {
        for col in 0..half.ncols() {
            let (here, there) = (half[(row, col)], full[(row, col)]);
            let magnitude = here.abs().max(there.abs());
            let candidate = WorstEntry {
                magnitude,
                diff: (here - there).abs(),
                tolerance: bound(row, col, magnitude),
            };
            if candidate.ratio() > worst.ratio() {
                worst = candidate;
            }
        }
    }
    worst
}

/// The margin by which the oracle catches a faulted correction at the floored
/// geometry: the worst entry's disagreement with the correct full-solve route,
/// measured against the derived bound.
///
/// A fault the oracle merely catches and a fault it catches with orders to
/// spare are different findings, and only the second says the bound has room
/// to have been widened for the rounding without losing the faults.
#[cfg(debug_assertions)]
fn margin_against_derived(faulted: &Mat<f64>, llt: &CholeskyFactor, b_rk: &Mat<f64>, b_kr: &Mat<f64>) -> WorstEntry {
    let y = cholesky_full_solve(llt, b_rk);
    let full = b_kr * &y;
    let (magnitudes, inflated) = schur_oracle_sums(llt, b_rk, b_kr, &y);
    let terms = llt.dim();
    worst_entry(faulted, &full, |row, col, _| {
        schur_oracle_tolerance(magnitudes[(row, col)], inflated[(row, col)], terms)
    })
}

/// The two correct routes disagree at the floored geometry by more than the retired bound allowed, and the derived bound admits them: a removed block whose near-collinear features leave every eigenvalue but the leading one at the replenishment floor, an eight-wide factor and two hundred kept dimensions. The witness is carried rather than assumed — the worst entry's disagreement is measured against the retired 2γ_r|x| + 1e-12 and asserted to exceed it — and the same entries are then measured against the derived bound and admitted, because that bound carries the conditioning of the substitution both routes perform and the retired form dropped. The margin is asserted to be a real one rather than a hair, so a bound that only just covered this geometry would fail here too.
///
/// ´claim:linalg:the-derived-bound-admits-the-rounding-a-floored-factor-produces´
/// ´test:crate:oracle-admits-the-rounding-a-floored-factor-produces´
#[cfg(debug_assertions)]
#[test]
fn oracle_admits_the_rounding_a_floored_factor_produces() {
    let (llt, b_rk, b_kr) = floored_schur_geometry();
    let terms = llt.dim();

    let half = {
        let w = schur_half_solve(&llt, &b_rk);
        w.transpose() * &w
    };
    let y = cholesky_full_solve(&llt, &b_rk);
    let full = &b_kr * &y;

    let against_retired = worst_entry(&half, &full, |_, _, magnitude| retired_oracle_tolerance(magnitude, terms));
    assert!(
        against_retired.ratio() > 1.0,
        "the geometry is one the retired bound refuses: its worst entry stood at {:.3} of it",
        against_retired.ratio()
    );

    let (magnitudes, inflated) = schur_oracle_sums(&llt, &b_rk, &b_kr, &y);
    let against_derived = worst_entry(&half, &full, |row, col, _| {
        schur_oracle_tolerance(magnitudes[(row, col)], inflated[(row, col)], terms)
    });
    println!(
        "floored geometry: worst entry stood at {:.1} of the retired bound and {:.3e} of the derived one; \
         at that entry the routes disagree by {:.3e} of its own magnitude and the derived bound allows {:.3e} of it, \
         which is the smallest relative divergence still refused there",
        against_retired.ratio(),
        against_derived.ratio(),
        against_derived.relative_diff(),
        against_derived.relative_tolerance()
    );
    assert!(
        against_derived.ratio() < 0.5,
        "the derived bound admits the geometry with a margin rather than by a hair: {:.3e}",
        against_derived.ratio()
    );

    // And the oracle itself, which is the verdict the engine takes.
    crate::linalg::bridge::schur_correction_oracle(&half, &llt, &b_rk, &b_kr);
}

/// A bound wide enough for the floored factor's rounding is not wide enough for a solve against the wrong factor: at the same geometry, a half-solve taken against the block regularised a hundred times more heavily still aborts. The two blocks differ by a hundredth on the diagonal alone and every eigenvalue but the leading one is at the floor, so the correction they imply differs by a share of its own size — which is the scale of divergence the oracle exists to report, and it stands orders above the conditioning term that admits the rounding.
///
/// ´claim:linalg:the-derived-bound-still-refuses-a-faulted-route-at-the-floor´
/// ´test:crate:oracle-refuses-a-solve-against-the-wrong-factor-at-the-floor´
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Schur oracle")]
fn oracle_refuses_a_solve_against_the_wrong_factor_at_the_floor() {
    /// The floor the faulted route regularises by, a hundred times the one the
    /// geometry was built at.
    const WRONG_FLOOR: f64 = 1e-1;
    /// Rows of the feature matrix whose Gram the block is.
    const OBSERVATIONS: usize = 24;

    let (llt, b_rk, b_kr) = floored_schur_geometry();
    let r = llt.dim();

    let features = Mat::from_fn(OBSERVATIONS, r, |i, j| {
        let shared = (i as f64).mul_add(0.125, 3.5);
        ((i + j) % 3) as f64 * 1e-3 + shared
    });
    let wrong_block = SymmetricMatrix::from_computation(features.transpose() * &features).add_scaled_identity(WRONG_FLOOR);
    let wrong_llt = cholesky(&wrong_block).expect("the more heavily regularised block factors");

    let w = schur_half_solve(&wrong_llt, &b_rk);
    let faulted = w.transpose() * &w;

    let margin = margin_against_derived(&faulted, &llt, &b_rk, &b_kr);
    println!(
        "a solve against the wrong factor at the floor: worst entry stood at {:.3e} of the derived bound, moving it by {:.3e} of its own magnitude against the {:.3e} allowed",
        margin.ratio(),
        margin.relative_diff(),
        margin.relative_tolerance()
    );
    assert!(
        margin.ratio() > 1e3,
        "the fault is caught with margin rather than by a hair: {:.3e}",
        margin.ratio()
    );

    crate::linalg::bridge::schur_correction_oracle(&faulted, &llt, &b_rk, &b_kr);
}

/// The same bound refuses the wrong operand at the same geometry: a half-solve taken against the coupling block with its columns gathered one position out — the off-by-one a marginalisation makes when the retained index list and the coupling disagree — aborts, because it reassigns every kept dimension's coupling to its neighbour and moves each correction entry by its own size rather than by its last bits.
///
/// (´claim:linalg:the-derived-bound-still-refuses-a-faulted-route-at-the-floor´)
/// ´test:crate:oracle-refuses-the-wrong-operand-at-the-floor´
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Schur oracle")]
fn oracle_refuses_the_wrong_operand_at_the_floor() {
    let (llt, b_rk, b_kr) = floored_schur_geometry();

    let columns = b_rk.ncols();
    let shifted = Mat::from_fn(b_rk.nrows(), columns, |i, j| b_rk[(i, (j + 1) % columns)]);
    let w = schur_half_solve(&llt, &shifted);
    let faulted = w.transpose() * &w;

    let margin = margin_against_derived(&faulted, &llt, &b_rk, &b_kr);
    println!(
        "the wrong operand at the floor: worst entry stood at {:.3e} of the derived bound, moving it by {:.3e} of its own magnitude against the {:.3e} allowed",
        margin.ratio(),
        margin.relative_diff(),
        margin.relative_tolerance()
    );
    assert!(
        margin.ratio() > 1e3,
        "the fault is caught with margin rather than by a hair: {:.3e}",
        margin.ratio()
    );

    crate::linalg::bridge::schur_correction_oracle(&faulted, &llt, &b_rk, &b_kr);
}

/// The same bound refuses a correction assembled from the wrong block at the same geometry: forming it as `B_kr · W` rather than `Wᵀ W` — the coupling standing where the half-solve's own transpose belongs, which is the shape a transposition slip leaves behind, since the two have the same dimensions and only one of them is the Gram matrix the correction is defined as — aborts. That the shapes agree is exactly why the oracle has to be the thing that catches it: nothing about the types refuses this product.
///
/// (´claim:linalg:the-derived-bound-still-refuses-a-faulted-route-at-the-floor´)
/// ´test:crate:oracle-refuses-the-coupling-block-standing-for-the-half-solve´
#[cfg(debug_assertions)]
#[test]
#[should_panic(expected = "Schur oracle")]
fn oracle_refuses_the_coupling_block_standing_for_the_half_solve() {
    let (llt, b_rk, b_kr) = floored_schur_geometry();

    let w = schur_half_solve(&llt, &b_rk);
    let faulted = &b_kr * &w;

    let margin = margin_against_derived(&faulted, &llt, &b_rk, &b_kr);
    println!(
        "the coupling block for the half-solve at the floor: worst entry stood at {:.3e} of the derived bound, moving it by {:.3e} of its own magnitude against the {:.3e} allowed",
        margin.ratio(),
        margin.relative_diff(),
        margin.relative_tolerance()
    );
    assert!(
        margin.ratio() > 1e3,
        "the fault is caught with margin rather than by a hair: {:.3e}",
        margin.ratio()
    );

    crate::linalg::bridge::schur_correction_oracle(&faulted, &llt, &b_rk, &b_kr);
}

/// The witness the self-guarding study probed with is refused, and no device carries it. A matrix of ones on the diagonal and twos off it is symmetric to the bit and indefinite by a whole eigenvalue, and it crosses the storage boundary untouched, because that boundary's only verdict is symmetry. What the study found beyond the boundary was a success: the plain factorisation was refused, and the retired second phase answered anyway — not by its diagonal shift, since a shift of a thousandth cannot lift an eigenvalue of minus one, but by the per-pivot backstop beneath it, which replaced the offending pivot and returned an inverse of neither the matrix supplied nor the matrix shifted. With one phase there is nothing to carry it: the refusal is the answer, and it names the pivot it stopped at. The storage boundary still admits the matrix, which is why the entries that install a persisted matrix take a factorisation's verdict of their own.
///
/// ´claim:linalg:the-indefinite-witness-crosses-the-storage-boundary-and-the-factorisation-refuses-it´
/// ´test:crate:the-storage-boundary-admits-an-indefinite-matrix-and-the-factorisation-refuses-it´
#[test]
fn the_storage_boundary_admits_an_indefinite_matrix_and_the_factorisation_refuses_it() {
    // Ones on the diagonal, twos off it. Every diagonal entry is strictly
    // positive and the spectrum is {3, -1}, which is the witness the suite
    // already carries for the two tests disagreeing
    // (´req:gaussian:positive-definiteness´).
    let mut mat = Mat::zeros(2, 2);
    mat[(0, 0)] = 1.0;
    mat[(1, 1)] = 1.0;
    mat[(0, 1)] = 2.0;
    mat[(1, 0)] = 2.0;

    let witness = SymmetricMatrix::from_computation(mat.clone());
    let (lambda_max, lambda_min) = eigenvalue_min_max(&witness).expect("the two-by-two spectrum converges");
    assert!(
        lambda_min < -0.5,
        "the witness must be indefinite by more than round-off: {lambda_min:.3e}"
    );
    assert!(
        lambda_max > 2.5,
        "the witness's leading eigenvalue is the other half of the {{3, -1}} spectrum: {lambda_max:.3e}"
    );
    let (_diag_max, diag_min) = witness.diagonal_min_max();
    assert!(
        diag_min > 0.0,
        "the witness must pass the diagonal test it is here to defeat: {diag_min:.3e}"
    );

    // The storage boundary's only verdict is symmetry, and this matrix passes
    // it: what comes back is a live precision matrix carrying a negative
    // eigenvalue.
    let restored = SymmetricMatrix::from_storage(mat, 1e-12).expect("the storage boundary admits an indefinite matrix");

    assert!(
        cholesky(&restored).is_err(),
        "the plain factorisation must refuse a matrix with a negative eigenvalue"
    );

    // And the utility the model calls returns that refusal rather than an
    // answer from some other matrix. Every call site is asked, because the
    // tag is diagnostic and must not fork the arithmetic.
    for site in [
        CholeskyCallSite::Periodic,
        CholeskyCallSite::Marginalisation,
        CholeskyCallSite::Revert,
    ] {
        match cholesky_inverse(&restored, site) {
            CholeskyInverseResult::Failed { pivot } => {
                assert_eq!(
                    pivot, 1,
                    "the refusal names the pivot the factorisation stopped at, at {site:?}"
                );
            }
            CholeskyInverseResult::Clean(_) => {
                panic!("the indefinite witness was inverted at {site:?}; no device may carry it");
            }
        }
    }
}
