// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`keep_indices_are_the_ascending_complement`] | bayes | The surviving coordinates are precisely those not named for removal, listed in ascending order. Every block gather in the marginalisation reads this list, so the order it comes back in is the order the surviving posterior will be laid out in — a permutation here would silently relabel features. |
//! | [`keep_indices_empty_remove`] | bayes | cites (´claim:bayes:the-kept-indices-are-the-ascending-complement-of-the-removed-ones´) |
//! | [`keep_indices_remove_all`] | bayes | cites (´claim:bayes:the-kept-indices-are-the-ascending-complement-of-the-removed-ones´) |
//! | [`scalar_case_r1`] | bayes | Removing one coordinate produces exactly the closed-form Schur complement: the surviving precision block less the outer product of its coupling to the departed coordinate, divided by that coordinate's own precision — with the pivot taken as the regularised value, not the bare one. The regularisation is visible in the arithmetic rather than hidden, and the correction is reported as applied rather than skipped. |
//! | [`information_transfer`] | bayes | When observations have built a real coupling between a coordinate and one about to be removed, the removal goes through the full correction rather than falling back to the bare submatrix, and what survives is a posterior still positive on both diagonals. A coordinate that is dropped was never isolated: whatever it explained about its neighbours has to be accounted for in what remains, not simply forgotten with it. |
//! | [`rank1_sigma_prime_is_covariance_submatrix`] | bayes | When a single coordinate is removed from a trained model, the surviving covariance is the plain submatrix of the covariance that was already there, bit for bit — no rank-one correction is applied to it, even though one is applied to the precision. The covariance and the precision marginalise by different rules, and treating the covariance submatrix as though it were the inverse of the precision submatrix is exactly the error this pins shut. |
//! | [`rank1_preserves_recompute_counter`] | bayes | Removing a single coordinate leaves the label counter exactly where it was. That path refactorises nothing, so it discharges none of the accumulated incremental drift, and resetting the counter would postpone the repair the model is already owed. |
//! | [`round_trip_extend_observe_marginalise`] | bayes | A model can be grown, trained across the enlarged space, and shrunk back to its original dimension, and come out the other side healthy: the full correction is applied rather than the fallback, the precision and covariance are still inverse to one another, and the mean carries the evidence gathered in between. This is the whole life-cycle a feature slot goes through, and it has to compose rather than merely working step by step. |
//! | [`condition_guard`] | bayes | When the block about to be removed is itself badly conditioned — diagonals spanning fifteen orders of magnitude here — the correction is declined and the surviving precision is the bare submatrix, with the skip named in the diagnostics. Inverting such a block would amplify rounding error into the correction, so an admittedly overconfident posterior is preferred to a numerically meaningless one, and the caller is told which it received. |
//! | [`bprime_verification_failure`] | bayes | The correction is checked before it is accepted: on a matrix built so the subtraction would drive a surviving precision diagonal negative, the result is rejected and the bare submatrix used instead. A negative precision is not a slightly wrong posterior but an impossible one, and it would break the next factorisation rather than the operation that produced it. |
//! | [`bprime_positive_diagonal_negative_eigenvalue`] | bayes | The verification refuses a B' that the diagonal test would have admitted. Removing the third coordinate of `[[1, 0, 0.8], [0, 1, 0.8], [0.8, 0.8, 1]]` leaves a correction that takes about 0.64 off both surviving diagonals and puts the same magnitude on the off-diagonal, so B' comes out with both diagonal entries near 0.36 — comfortably positive — and eigenvalues of about 1.0 and −0.28. This is the matrix the requirement describes (´req:gaussian:positive-definiteness´): strictly positive diagonal, negative eigenvalue carried through the off-diagonal structure. The test builds that B' independently and asserts both halves of the disagreement before checking that the call falls back, so the red and the green are visible in the same place rather than one of them living in a commit message. |
//! | [`rank1_verification_refuses_the_rounding_witness`] | bayes | A parent that is positive definite as an exact matrix of its represented values, marginalised by the implemented rank-one formula, produces a B' that the retired diagonal check adopts and the requirement refuses. Its minimum diagonal is about 10⁻⁷ while its least eigenvalue is about −4.1×10⁻¹⁹, and no guard ahead of the verification sees anything wrong: the scalar pivot is exactly one, so the condition estimate is one, and the regularised denominator is 1.00001. This is the case the exact-arithmetic theorem does not cover, because the path never performs the exact subtraction the theorem is about — the entrywise-rounded correction is full rank here, with determinant about 1.4×10⁻¹⁸ rather than zero, so rank-one interlacing no longer confines the risk to one eigenvalue (´req:gaussian:rank-one-verification´). Both halves are asserted before the call is made, so the red and the green stand in the same place: the witness must pass the diagonal test, or it witnesses nothing. |
//! | [`rank1_gershgorin_certifies_a_dominant_bprime`] | bayes | A strictly diagonally dominant B' is certified by the discs alone, and the diagnostics say so. This is the arm the arrangement exists for: the verdict is the same verdict the spectrum would return, reached without computing a spectrum, so the frequent path keeps the quadratic cost the corpus prices it at (´tab:gaussian:operation-costs´). The matrix here is chosen so that every disc clears the resolution floor by orders of magnitude rather than by a margin the outward rounding could eat, since a certificate that depended on the rounding direction to pass would be evidence that the rounding was directed the wrong way (´req:gaussian:rank-one-verification´). |
//! | [`rank1_diagonal_disproof_refuses_without_a_spectrum`] | bayes | A non-positive diagonal entry is refused without a spectrum being computed. The diagonal is still evidence in this one direction — a diagonal entry is the Rayleigh quotient at a coordinate vector, so a non-positive one puts λ_min at or below zero and the resolution floor is never negative — and factorising the block to confirm what a sign already settles would spend the cubic cost the whole arrangement exists to avoid (´req:gaussian:rank-one-verification´). What this must not become is the retired check read backwards: the accepting direction stays unavailable, which the witness above is what pins. |
//! | [`verification_refuses_inside_the_resolution_band_and_accepts_above_it`] | bayes | The band the resolution floor closes is refused, and what stands above the floor is accepted. A B' whose least eigenvalue lies strictly between zero and the floor does not pass the verification, and the same B' raised so that its least eigenvalue clears the floor by two orders does. What the boundary buys is stated at (´const:assayer:schur-verification-resolution´): at that distance from zero the arithmetic that decides the question carries an error of the same size, so a verdict read there would be noise rather than a reading of the requirement (´req:gaussian:positive-definiteness´). Both matrices are `[[1 + s, 1 − t], [1 − t, 1 + s]]`, whose eigenvalues are `s + t` and `2 + s − t`, with `s` and `t` powers of two so that the placement relative to the floor is constructed rather than observed; the floor is taken from the verification's own bound rather than recomputed here, so the test cannot drift away from the boundary it is about. The refusal names the pivot that failed and the floor it failed against, which is what the fallback logs. |
//! | [`marginalisation_ledger_reports_the_certificate_share`] | bayes | The ledger counts what share of verifications the cheap certificate settled, and leaves the events that put nothing to a certificate out of the denominator. That share is the measurement the arrangement was adopted on and the one no analysis supplies: whether the discs are enough depends on the precision matrices a deployment actually produces (´req:gaussian:rank-one-verification´). A rate reported over a denominator that included refusals-before-formation would read low for a reason that has nothing to do with the certificate, which is why the vacuous events are excluded rather than counted as misses. |
//! | [`schur_delta_scales_with_lambda_prior`] | bayes | The regularisation follows the prior it is derived from. Against the same precision `[[2, 1], [1, 1]]` and the same configuration, removing the second coordinate at a prior precision of one and at the default of 0.1 gives two different surviving precisions, and each names the δ it was computed with exactly: B' is 2 − 1/(1 + δ) here, so 1/(2 − B') − 1 recovers what the computation added. The recovered values are 10⁻⁴ and 10⁻⁵, the configured factor against each prior (´alg:gaussian:regularised-schur´). Held as a fixed constant instead, δ would be one value for both and the first recovery would fail — which is the defect this fixes, the constant having agreed with the derivation only at the default prior, so that a deployment moving its prior stopped following the specification without saying so. |
//! | [`bprime_semidefinite_boundary_refused`] | bayes | The boundary is closed on the wrong side for the correction: a B' that is positive *semi*-definite is refused, because the requirement asks for a strictly positive minimum eigenvalue and a zero one is not that (´req:gaussian:positive-definiteness´). Removing the third coordinate of `[[1, 0, 1], [0, 1, 1], [1, 1, 2]]` with the regularisation switched off gives B' = `[[0.5, −0.5], [−0.5, 0.5]]` exactly, whose eigenvalues are 1 and 0. Its diagonal is positive throughout, so this is a second matrix the retired test admitted; the singular precision it would have adopted has no inverse, and the covariance the marginalisation goes on to restore is exactly that inverse. |
//! | [`half_solve_equiv_full_solve`] | bayes | Two routes to the same correction agree element-wise: forming it as a Gram product of a half-solved factor, and forming it by solving the full system and multiplying back. The Gram route is chosen because it is positive semi-definite by construction and so cannot produce a correction that pushes precision the wrong way, and this fixes that the structural guarantee costs nothing in the answer — across differing block sizes, removal positions, and coupling strengths alike. |
//! | [`sigma_prime_from_fresh_cholesky`] | bayes | Removing more than one coordinate rebuilds the covariance by inverting the corrected precision outright, so a model carrying two hundred labels' worth of incremental drift comes out of the operation tightly synchronised again rather than inheriting that drift. The multi-coordinate path already pays for a factorisation, so it may as well discharge the accumulated error while it is there. |
//! | [`labels_since_recompute_reset`] | bayes | A model that has accumulated a real label count comes out of a multi-coordinate removal with that count back at zero. The fresh factorisation performed there has paid off the drift the counter was tracking, so the next scheduled recomputation is measured from here rather than being brought forward for work already done. |
//! | [`schur_vs_bkk_ordering`] | bayes | Against an identically trained twin marginalised by bare submatrix extraction, the corrected path never claims more precision on any surviving diagonal, and claims strictly less on at least one. Dropping a coordinate costs the model an information channel; the bare submatrix keeps the precision it had and so becomes overconfident, while the correction is what widens the posterior by the right amount. |
//! | [`empty_remove_noop`] | bayes | With nothing named for removal the precision comes back at its original dimension with every diagonal exactly as it was, and the result is reported as a full correction rather than a fallback. The degenerate case takes the ordinary path and produces the identity operation, so nothing upstream has to special-case an empty removal set. |
//! | [`marginalise_terminus_retains_and_flags`] | bayes | Marginalisation does not fail: driving the double failure through it — a negative-definite precision under a regularisation configuration whose cascade cannot repair a pivot — completes the removal anyway. The dimension comes down, the uncorrected kept block is adopted as the precision, the covariance is the maintained one's kept block value for value, and the outcome travels in the diagnostics as a flagged factorisation failure rather than in a Result. Before this repair the same construction returned an error the lifecycle path answered by abandoning the deregistration, so a caller had to handle a case the corpus promises does not exist. |
//! | [`marginalisation_error_is_the_measured_loss`] | bayes | The reported error is the loss actually committed, not the offset that caused it. Removing the second coordinate of `[[1, 0.001], [0.001, 0.001]]` leaves a removed block whose least eigenvalue is a thousandth, so at the default offset of 10⁻⁵ the correction loses δ/(a + δ), about one part in a hundred — three decimal orders more than the offset. Both figures are checked against the closed form and the share is checked to be far above δ, which is the reading this replaces: the audit showed that the bare offset bounds nothing, since the loss scales with the removed block's spectrum and the coupling, and a budget written as δ would understate this ordinary case by a factor of a thousand. |
//! | [`discarded_correction_reports_its_whole_size`] | bayes | Discarding the correction is reported as the error it is. The same matrix whose loss is a hundredth under the offset loses the whole correction when the host's posture refuses the block, and the report says so: the share is exactly one and the mass is the exact correction's own trace. This is the offset's endpoint — as the regularisation grows without bound the correction vanishes and the result approaches the kept block — so a fallback is the maximum of this same approximation rather than an exact path beside it. Reporting nothing here, as the code did before, made the most approximate outcome the one the host could not see. |
//! | [`two_guards_fire_distinctly`] | bayes | One block, two guards, and which fired is in the diagnostics. The removed block `diag(10⁸, 10⁻⁸)` has a raw condition number of 10¹⁶ and, at the default offset, a regularised one near 10¹³. Under the shipped posture it is the host's guard that refuses it, as a source too degraded to fold in. Under a posture raised past 10¹⁸ — a deployment declaring itself willing to take such a source — the same block is refused anyway, by the package's own solvability guard, because the matrix the correction would be solved against is beyond what the arithmetic supports. The second refusal is the one a host cannot configure away, which is what makes the two guards two and not one: the audit found a single threshold standing for both protections, and at the reference scale regularisation moves a raw 10⁸ to a regularised 10⁴, so the two questions have different answers on the same matrix. |
//! | [`posture_guard_reads_the_spectrum`] | bayes | The host's guard reads the spectrum, and a diagonal ratio would have let this block through. The audit's own counterexample is a removed block `[[1, 1 − 10⁻¹²], [1 − 10⁻¹², 1]]`: every diagonal entry is one, so the ratio the code used to test is exactly one and passes any ceiling, while the true condition number is about 2 × 10¹² and the block is nearly singular. The guard now refuses it, and the diagnostics say the figure is a measurement of the spectrum rather than the bound the ratio was. A lower bound that small certifies nothing, which is why the estimator could not be left standing under a guard whose whole purpose is refusal. |
//! | [`host_schur_configuration_reaches_the_marginalisation`] | bayes | A configuration the host sets reaches the computation. The same trained model is marginalised twice from identical copies, once at the shipped regularisation factor and once at a factor a hundred times larger, and the surviving precisions differ. They cannot differ unless the factor travelled from the configuration through the model owner's working copy and into the Schur complement, which is exactly the journey it did not make before: both marginalisation call sites built their own default configuration and discarded the exposed one, so a deployment could set the factor and change nothing. The larger factor attenuates more of the correction, so its surviving precision is the larger one — the direction the regularisation's one-sided bias predicts. |
//! | [`marginalisation_ledger_folds_events`] | bayes | The ledger folds events rather than reporting only the last one. Three events are recorded — a measured loss, a bound, and a removal with no finite figure — and each lands in its own count while the mass accumulates over the two that carried one and the worst share is the largest of them. The split between the counts is the point: a cumulative mass mixing measurements with bounds would be read as a level, and the counts are what say how much of it is which. The unbounded event moves no figure at all, because there is nothing honest to move it by, and is counted so that its silence is visible. |
//! | [`schur_complement_perf`] | bayes | At the dimensions the system actually runs at — a 638-coordinate model losing a 68-coordinate block — a marginalisation costs a few hundred milliseconds, so ten of them fit inside a three-second budget. Marginalising happens on a live model, and a cubic-cost operation at this size has to be known to be affordable rather than assumed to be. Kept off the default run because paying that cost on every test invocation is not. |

//! Crate-level tests for `model::marginalise`, whose correction is a half-solve (´dec:posterior:half-solve´).
//!
//! # Cross-References
//!
//! - (´dec:posterior:half-solve´) — the Schur correction is taken with a half-solve rather than a full one
//! - (´thm:gaussian:marginalisation´) — the marginalisation the Schur complement computes

#![allow(
    clippy::cast_lossless,
    clippy::cast_precision_loss,
    clippy::similar_names,
    clippy::suboptimal_flops
)]

use crate::linalg::bridge::{cholesky, cholesky_full_solve, eigenvalue_min_max, schur_half_solve};
use crate::linalg::convert::vec_to_col;
use crate::linalg::symmetric::SymmetricMatrix;
use crate::model::bayesian::BayesianLinearModel;
use crate::model::marginalise::{
    ConditionMeasure, MarginalisationError, MarginalisationErrorLedger, SchurConfig, SchurCorrectionOutcome, SchurDiagnostics,
    VerificationCertificate, compute_keep_indices, compute_schur_complement, verification_floor, verify_positive_definite,
    verify_rank1_positive_definite,
};
use crate::model::update::compute_effective_weight;
use crate::testing::{DEFAULT_TOLERANCES, TestRng, assert_below, assert_near};
use crate::types::ModelId;

/// Seed for the deterministic [`TestRng`] used across this file.
const RNG_SEED: u64 = 42;

/// The prior precision the default configuration builds models at, and so
/// the prior these fixtures take their regularisation against: at the
/// default factor it puts δ at 10⁻⁵ (´alg:gaussian:regularised-schur´).
const FIXTURE_LAMBDA_PRIOR: f64 = 0.1;

/// Uniform `f64` in `[-0.5, 0.5)` — features for pseudo-random labels.
fn signed_half(rng: &mut TestRng) -> f64 {
    rng.next_f64() - 0.5
}

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

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

/// Check that all diagonal elements of a `SymmetricMatrix` are positive.
fn assert_positive_diagonal(m: &SymmetricMatrix, label: &str) {
    for i in 0..m.dim() {
        assert!(
            m.diagonal_element(i) > 0.0,
            "{label}: diagonal[{i}] = {} (expected positive)",
            m.diagonal_element(i),
        );
    }
}

/// Applies N SM updates to a model, returning it with non-trivial posterior.
fn train_model(model: &mut BayesianLinearModel, n_labels: usize, p: usize) {
    let mut rng = TestRng::new(RNG_SEED);
    for _ in 0..n_labels {
        let phi: Vec<f64> = (0..p).map(|_| signed_half(&mut rng)).collect();
        let phi_hat = vec_to_col(&phi);

        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        let r_rho = if rng.next_range(5) == 0 { 1.0 } else { -1.0 };
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.999, 1e-4);
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Keep-indices utility
// ═══════════════════════════════════════════════════════════════════════════════

/// The surviving coordinates are precisely those not named for removal, listed
/// in ascending order. Every block gather in the marginalisation reads this
/// list, so the order it comes back in is the order the surviving posterior
/// will be laid out in — a permutation here would silently relabel features.
///
/// ´claim:bayes:the-kept-indices-are-the-ascending-complement-of-the-removed-ones´
/// ´test:crate:keep-indices-are-the-ascending-complement´
#[test]
fn keep_indices_are_the_ascending_complement() {
    let keep = compute_keep_indices(6, &[2, 4]);
    assert_eq!(keep, vec![0, 1, 3, 5]);
}

/// Naming nothing for removal keeps everything, in order — the complement of
/// the empty set is the whole range rather than a degenerate answer.
///
/// (´claim:bayes:the-kept-indices-are-the-ascending-complement-of-the-removed-ones´)
/// ´test:crate:keep-indices-empty-remove´
#[test]
fn keep_indices_empty_remove() {
    let keep = compute_keep_indices(5, &[]);
    assert_eq!(keep, vec![0, 1, 2, 3, 4]);
}

/// Naming every coordinate for removal keeps none: the complement comes back
/// empty rather than wrapping round or leaving a stray survivor.
///
/// (´claim:bayes:the-kept-indices-are-the-ascending-complement-of-the-removed-ones´)
/// ´test:crate:keep-indices-remove-all´
#[test]
fn keep_indices_remove_all() {
    let keep = compute_keep_indices(3, &[0, 1, 2]);
    assert_eq!(keep, [] as [usize; 0]);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Schur complement algorithm
// ═══════════════════════════════════════════════════════════════════════════════

/// Removing one coordinate produces exactly the closed-form Schur complement:
/// the surviving precision block less the outer product of its coupling to the
/// departed coordinate, divided by that coordinate's own precision — with the
/// pivot taken as the regularised value, not the bare one. The regularisation
/// is visible in the arithmetic rather than hidden, and the correction is
/// reported as applied rather than skipped.
///
/// ´claim:bayes:removing-one-coordinate-gives-the-closed-form-schur-complement-about-the-regularised-pivot´
/// ´test:crate:scalar-case-r1´
#[test]
fn scalar_case_r1() {
    // 3×3 SPD matrix, remove index 2
    let data = faer::Mat::from_fn(3, 3, |i, j| {
        let vals = [[4.0, 1.0, 0.5], [1.0, 3.0, 0.3], [0.5, 0.3, 2.0]];
        vals[i][j]
    });
    let b = SymmetricMatrix::from_computation(data);

    let remove = [2_usize];
    let keep = compute_keep_indices(3, &remove);
    let config = SchurConfig::default();

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(!result.diagnostics.fell_back_to_bkk);
    assert_eq!(result.diagnostics.correction_outcome, SchurCorrectionOutcome::Applied);

    // Closed-form: B' = B_kk - B_k2 * (1/B_22) * B_2k
    // B_22 = 2.0
    // B_k2 = [0.5, 0.3]^T, B_2k = [0.5, 0.3]
    // Correction = [[0.5*0.5, 0.5*0.3], [0.3*0.5, 0.3*0.3]] / 2.0
    //            = [[0.125, 0.075], [0.075, 0.045]]
    // Adjusting for δ regularisation: B_rr + δI = 2.0 + 1e-5
    let b22_reg = 2.0 + config.delta(FIXTURE_LAMBDA_PRIOR);
    let expected_00 = 4.0 - 0.5 * 0.5 / b22_reg;
    let expected_01 = 1.0 - 0.5 * 0.3 / b22_reg;
    let expected_11 = 3.0 - 0.3 * 0.3 / b22_reg;

    let bp = &result.b_prime;
    let tol = 1e-10;
    assert_near(bp.as_inner()[(0, 0)], expected_00, tol, "B'[0,0]");
    assert_near(bp.as_inner()[(0, 1)], expected_01, tol, "B'[0,1]");
    assert_near(bp.as_inner()[(1, 1)], expected_11, tol, "B'[1,1]");
}

/// When observations have built a real coupling between a coordinate and one
/// about to be removed, the removal goes through the full correction rather
/// than falling back to the bare submatrix, and what survives is a posterior
/// still positive on both diagonals. A coordinate that is dropped was never
/// isolated: whatever it explained about its neighbours has to be accounted
/// for in what remains, not simply forgotten with it.
///
/// ´claim:bayes:removing-a-correlated-coordinate-accounts-for-its-cross-terms-instead-of-discarding-them´
/// ´test:crate:information-transfer´
#[test]
fn information_transfer() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);

    // Inject observations with features spanning dims 0 and 2
    // to build cross-correlations.
    for i in 0..50 {
        let mut phi = vec![0.0; p];
        phi[0] = 1.0; // Always use dim 0
        phi[2] = 0.8; // And dim 2 (to build cross-terms)
        phi[1] = 0.1 * (i as f64); // Varying dim 1
        let phi_hat = vec_to_col(&phi);
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, 1.0, 0.999, 1e-4);
    }

    // Read off-diagonal B[0,2] — should be non-trivial
    let b02_before = model.precision().as_inner()[(0, 2)];
    assert!(
        b02_before.abs() > 0.01,
        "B[0,2] should be non-trivial before marginalisation, got {b02_before}"
    );

    // Marginalise dim 2
    let diag = model.marginalise(&[2], &SchurConfig::default(), ModelId::Operational);

    // The correction term should have been applied (not B_kk fallback)
    assert!(!diag.fell_back_to_bkk, "Expected full Schur, got B_kk fallback");

    // After marginalisation, dim 0 should have higher precision
    // than the B_kk fallback would give. Check by comparing against
    // a B_kk-only version.
    // The Schur complement B' = B_kk - correction means
    // B'_00 should be slightly LESS than B_kk_00 (correction subtracted).
    // But the information is transferred: the posterior is tighter where
    // cross-terms contributed.
    assert_eq!(model.dim(), p - 1);
    assert_positive_diagonal(model.precision(), "B after Schur marginalise");
    assert_positive_diagonal(model.covariance(), "Σ after Schur marginalise");
}

/// When a single coordinate is removed from a trained model, the surviving
/// covariance is the plain submatrix of the covariance that was already there,
/// bit for bit — no rank-one correction is applied to it, even though one is
/// applied to the precision. The covariance and the precision marginalise by
/// different rules, and treating the covariance submatrix as though it were
/// the inverse of the precision submatrix is exactly the error this pins shut.
///
/// ´claim:bayes:the-rank-one-path-takes-the-marginal-covariance-as-the-plain-submatrix´
/// ´test:crate:rank1-sigma-prime-is-covariance-submatrix´
#[test]
fn rank1_sigma_prime_is_covariance_submatrix() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);
    train_model(&mut model, 120, p);

    let remove = [3];
    let keep = compute_keep_indices(p, &remove);
    let expected_covariance = model.covariance().extract_symmetric_submatrix(&keep);

    let diag = model.marginalise(&remove, &SchurConfig::default(), ModelId::Operational);

    assert!(!diag.fell_back_to_bkk, "expected rank-1 Schur correction to apply");
    assert_eq!(model.dim(), p - 1);

    let tol = DEFAULT_TOLERANCES.bit_identical;
    for i in 0..model.dim() {
        for j in 0..model.dim() {
            assert_near(
                model.covariance().as_inner()[(i, j)],
                expected_covariance.as_inner()[(i, j)],
                tol,
                &format!("rank1 Σ_kk extraction at ({i},{j})"),
            );
        }
    }
}

/// Removing a single coordinate leaves the label counter exactly where it was.
/// That path refactorises nothing, so it discharges none of the accumulated
/// incremental drift, and resetting the counter would postpone the repair the
/// model is already owed.
///
/// ´claim:bayes:a-rank-one-marginalisation-refactorises-nothing-so-the-recompute-cadence-carries-on´
/// ´test:crate:rank1-preserves-recompute-counter´
#[test]
fn rank1_preserves_recompute_counter() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);
    train_model(&mut model, 50, p);
    let labels_before = model.labels_since_recompute();
    assert!(labels_before > 0);

    model.marginalise(&[2], &SchurConfig::default(), ModelId::Operational);

    assert_eq!(model.labels_since_recompute(), labels_before);
}

/// A model can be grown, trained across the enlarged space, and shrunk back to
/// its original dimension, and come out the other side healthy: the full
/// correction is applied rather than the fallback, the precision and
/// covariance are still inverse to one another, and the mean carries the
/// evidence gathered in between. This is the whole life-cycle a feature slot
/// goes through, and it has to compose rather than merely working step by step.
///
/// ´claim:bayes:a-full-extend-observe-marginalise-cycle-returns-a-healthy-model-carrying-what-it-learned´
/// ´test:crate:round-trip-extend-observe-marginalise´
#[test]
fn round_trip_extend_observe_marginalise() {
    let p_base = 10;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p_base, prior_precision, 1000);

    // Train the base model to move it from prior
    train_model(&mut model, 100, p_base);

    // Snapshot pre-extension state
    let mu_before: Vec<f64> = (0..p_base).map(|i| model.mu()[i]).collect();
    let _b_diag_before: Vec<f64> = (0..p_base).map(|i| model.precision().diagonal_element(i)).collect();

    // Extend by 5 dims
    let r = 5;
    model.extend(r, prior_precision);
    assert_eq!(model.dim(), p_base + r);

    // Observe 200 labels spanning all dims (builds cross-terms)
    train_model(&mut model, 200, p_base + r);

    // Marginalise the 5 new dims
    let remove_indices: Vec<usize> = (p_base..p_base + r).collect();
    let diag = model.marginalise(&remove_indices, &SchurConfig::default(), ModelId::Operational);

    assert_eq!(model.dim(), p_base);

    // The surviving dims should retain information from cross-terms
    // (via the Schur correction). Check dual tracking is healthy.
    let dev = frobenius_deviation_from_identity(model.precision(), model.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after Schur round-trip");

    // Posterior has evolved from the pre-extension state due to
    // 200 labels of forgetting + learning. The point is that
    // information from cross-terms is preserved (not discarded
    // as B_kk would).
    let mu_after: Vec<f64> = (0..p_base).map(|i| model.mu()[i]).collect();
    let mu_changed = mu_before.iter().zip(&mu_after).any(|(a, b)| (a - b).abs() > 1e-6);
    assert!(mu_changed, "μ should have changed after 200 labels + marginalisation");

    // If the Schur correction was applied, the diagnostics should show it
    assert!(!diag.fell_back_to_bkk, "Expected full Schur, not B_kk fallback");
}

/// When the block about to be removed is itself badly conditioned — diagonals
/// spanning fifteen orders of magnitude here — the correction is declined and
/// the surviving precision is the bare submatrix, with the skip named in the
/// diagnostics. Inverting such a block would amplify rounding error into the
/// correction, so an admittedly overconfident posterior is preferred to a
/// numerically meaningless one, and the caller is told which it received.
///
/// ´claim:bayes:an-ill-conditioned-removed-block-makes-the-correction-be-declined-in-favour-of-the-bare-submatrix´
/// ´test:crate:condition-guard´
#[test]
fn condition_guard() {
    // Build a precision matrix where the removed block has extreme conditioning.
    // We'll construct a 4×4 matrix where dims 2,3 have wildly different diagonals.
    let p = 4;
    let mut data = faer::Mat::zeros(p, p);
    // Well-conditioned kept block
    data[(0, 0)] = 1.0;
    data[(1, 1)] = 1.0;
    // Ill-conditioned removed block: ratio = 1e12 / 1e-3 = 1e15 >> 1e8
    data[(2, 2)] = 1e12;
    data[(3, 3)] = 1e-3;
    // Small cross-terms
    data[(0, 2)] = 0.01;
    data[(2, 0)] = 0.01;
    data[(1, 3)] = 0.01;
    data[(3, 1)] = 0.01;

    let b = SymmetricMatrix::from_computation(data);
    let remove = [2_usize, 3];
    let keep = compute_keep_indices(p, &remove);
    let config = SchurConfig::default();

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert_eq!(
        result.diagnostics.correction_outcome,
        SchurCorrectionOutcome::PostureGuardSkip,
    );
    assert!(result.diagnostics.fell_back_to_bkk);

    // B' should be B_kk (the 2×2 identity)
    assert_eq!(result.b_prime.dim(), 2);
    assert!((result.b_prime.as_inner()[(0, 0)] - 1.0).abs() < 1e-14);
    assert!((result.b_prime.as_inner()[(1, 1)] - 1.0).abs() < 1e-14);
}

/// The correction is checked before it is accepted: on a matrix built so the
/// subtraction would drive a surviving precision diagonal negative, the result
/// is rejected and the bare submatrix used instead. A negative precision is
/// not a slightly wrong posterior but an impossible one, and it would break
/// the next factorisation rather than the operation that produced it.
///
/// ´claim:bayes:a-correction-that-would-drive-a-precision-diagonal-negative-is-rejected-in-favour-of-the-bare-submatrix´
/// ´test:crate:bprime-verification-failure´
#[test]
fn bprime_verification_failure() {
    // We need B_kr (B_rr+δI)^{-1} B_rk to produce a correction
    // that exceeds B_kk on at least one diagonal. This happens when
    // B_kk is small but B_kr is large relative to B_rr.
    //
    // Strategy: B_kk very small, B_kr very large, B_rr moderate.
    let p = 3;
    let mut data = faer::Mat::zeros(p, p);
    // B = [[0.01, 0, 10], [0, 0.01, 0], [10, 0, 1.0]]
    // B_kk = [[0.01, 0], [0, 0.01]]
    // B_rr = [1.0], B_kr = [[10], [0]]
    // correction = B_kr * (1/(1+δ)) * B_rk
    //            = [[100], [0]] * (1/1.00001) ~= [[100, 0], [0, 0]]
    // B' = B_kk - correction = [[-99.99, 0], [0, 0.01]]
    // B'[0,0] < 0 → verification fails → fallback to B_kk
    data[(0, 0)] = 0.01;
    data[(1, 1)] = 0.01;
    data[(2, 2)] = 1.0;
    data[(0, 2)] = 10.0;
    data[(2, 0)] = 10.0;

    let b = SymmetricMatrix::from_computation(data);
    let remove = [2_usize];
    let keep = compute_keep_indices(p, &remove);
    let config = SchurConfig::default();

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    // The correction should over-subtract, failing verification
    assert!(!result.diagnostics.bprime_verification_passed);
    assert!(result.diagnostics.fell_back_to_bkk);

    // Fallback is B_kk
    assert_eq!(result.b_prime.dim(), 2);
    assert!((result.b_prime.as_inner()[(0, 0)] - 0.01).abs() < 1e-14);
}

/// The verification refuses a B' that the diagonal test would have admitted.
/// Removing the third coordinate of `[[1, 0, 0.8], [0, 1, 0.8], [0.8, 0.8, 1]]`
/// leaves a correction that takes about 0.64 off both surviving diagonals and
/// puts the same magnitude on the off-diagonal, so B' comes out with both
/// diagonal entries near 0.36 — comfortably positive — and eigenvalues of about
/// 1.0 and −0.28. This is the matrix the requirement describes
/// (´req:gaussian:positive-definiteness´): strictly positive diagonal, negative
/// eigenvalue carried through the off-diagonal structure. The test builds that
/// B' independently and asserts both halves of the disagreement before checking
/// that the call falls back, so the red and the green are visible in the same
/// place rather than one of them living in a commit message.
///
/// ´claim:bayes:a-b-prime-with-positive-diagonal-and-negative-eigenvalue-is-refused´
/// ´test:crate:bprime-positive-diagonal-negative-eigenvalue´
#[test]
fn bprime_positive_diagonal_negative_eigenvalue() {
    let p = 3;
    let coupling = 0.8;
    let mut data = faer::Mat::zeros(p, p);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = 1.0;
    data[(2, 2)] = 1.0;
    data[(0, 2)] = coupling;
    data[(2, 0)] = coupling;
    data[(1, 2)] = coupling;
    data[(2, 1)] = coupling;

    let b = SymmetricMatrix::from_computation(data);
    let remove = [2_usize];
    let keep = compute_keep_indices(p, &remove);
    let config = SchurConfig::default();

    // Rebuild the B' the correction produces, independently of the code under
    // test: correction = B_kr (B_rr + δ)^{-1} B_rk with B_rr = [1].
    let scale = coupling * coupling / (1.0 + config.delta(FIXTURE_LAMBDA_PRIOR));
    let mut expected = faer::Mat::zeros(2, 2);
    expected[(0, 0)] = 1.0 - scale;
    expected[(1, 1)] = 1.0 - scale;
    expected[(0, 1)] = -scale;
    expected[(1, 0)] = -scale;
    let b_prime = SymmetricMatrix::from_computation(expected);

    // Red: the retired test admits this B'.
    let (_, min_diag) = b_prime.diagonal_min_max();
    assert!(
        min_diag > 0.0,
        "the witness must pass the diagonal test, or it witnesses nothing (got {min_diag})"
    );

    // Green: the requirement's test refuses it.
    let (_, lambda_min) = eigenvalue_min_max(&b_prime).expect("a symmetric matrix has a spectrum");
    assert!(
        lambda_min < 0.0,
        "the witness must carry a negative eigenvalue (got {lambda_min})"
    );

    // And the call site now follows the requirement rather than the diagonal.
    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(
        !result.diagnostics.bprime_verification_passed,
        "B' has a negative eigenvalue and must not be adopted"
    );
    assert!(result.diagnostics.fell_back_to_bkk);
    assert_eq!(result.diagnostics.correction_outcome, SchurCorrectionOutcome::Applied);

    // The fallback is B_kk, the untouched kept block.
    assert_eq!(result.b_prime.dim(), 2);
    assert_near(
        result.b_prime.as_inner()[(0, 0)],
        1.0,
        DEFAULT_TOLERANCES.default,
        "fallback keeps B_kk",
    );
    assert_near(
        result.b_prime.as_inner()[(0, 1)],
        0.0,
        DEFAULT_TOLERANCES.default,
        "fallback keeps B_kk off-diagonal",
    );
}

/// The rank-one study's parent, as six exact binary64 values.
///
/// Rust has no hexadecimal float literal, so the bit patterns stand in and the
/// hexadecimal float each one denotes is named beside it. Nothing here may be
/// rewritten as a decimal literal: three of the six sit one to three units in
/// the last place away from the decimal they print near, and that displacement
/// is the entire construction.
#[cfg(feature = "serde")]
struct RoundingWitness {
    /// The removed scalar `a`, `0x1.0000000000000p+0`.
    a: f64,
    /// Coupling `u_0`, `0x1.0000000000000p+0`.
    u0: f64,
    /// Coupling `u_1`, `0x1.999999999999ap-4`.
    u1: f64,
    /// `A_00`, `0x1.0000000000001p+0`.
    a00: f64,
    /// `A_11`, `0x1.47ae147ae147ep-7`.
    a11: f64,
    /// `A_10`, `0x1.999999999999cp-4`.
    a10: f64,
}

#[cfg(feature = "serde")]
impl RoundingWitness {
    fn new() -> Self {
        Self {
            a: f64::from_bits(0x3FF0_0000_0000_0000),
            u0: f64::from_bits(0x3FF0_0000_0000_0000),
            u1: f64::from_bits(0x3FB9_9999_9999_999A),
            a00: f64::from_bits(0x3FF0_0000_0000_0001),
            a11: f64::from_bits(0x3F84_7AE1_47AE_147E),
            a10: f64::from_bits(0x3FB9_9999_9999_999C),
        }
    }

    /// The parent B, with the removed dimension last.
    fn parent(&self) -> SymmetricMatrix {
        let mut data = faer::Mat::zeros(3, 3);
        data[(0, 0)] = self.a00;
        data[(1, 1)] = self.a11;
        data[(2, 2)] = self.a;
        data[(1, 0)] = self.a10;
        data[(2, 0)] = self.u0;
        data[(2, 1)] = self.u1;
        SymmetricMatrix::from_computation(data)
    }

    /// The B' the rank-one path forms, rebuilt here in the source's own
    /// operation order and parenthesisation so that the witness is a witness
    /// about the implemented computation and not about an idealisation of it.
    fn b_prime(&self, delta: f64) -> SymmetricMatrix {
        let c = 1.0 / (self.a + delta);
        let mut data = faer::Mat::zeros(2, 2);
        data[(0, 0)] = self.a00 - (c * self.u0) * self.u0;
        data[(1, 1)] = self.a11 - (c * self.u1) * self.u1;
        data[(1, 0)] = self.a10 - (c * self.u1) * self.u0;
        SymmetricMatrix::from_computation(data)
    }
}

/// A parent that is positive definite as an exact matrix of its represented
/// values, marginalised by the implemented rank-one formula, produces a B'
/// that the retired diagonal check adopts and the requirement refuses. Its
/// minimum diagonal is about 10⁻⁷ while its least eigenvalue is about
/// −4.1×10⁻¹⁹, and no guard ahead of the verification sees anything wrong: the
/// scalar pivot is exactly one, so the condition estimate is one, and the
/// regularised denominator is 1.00001. This is the case the exact-arithmetic
/// theorem does not cover, because the path never performs the exact
/// subtraction the theorem is about — the entrywise-rounded correction is full
/// rank here, with determinant about 1.4×10⁻¹⁸ rather than zero, so rank-one
/// interlacing no longer confines the risk to one eigenvalue
/// (´req:gaussian:rank-one-verification´). Both halves are asserted before the
/// call is made, so the red and the green stand in the same place: the witness
/// must pass the diagonal test, or it witnesses nothing.
///
/// ´claim:bayes:a-represented-spd-parent-can-round-into-a-b-prime-the-diagonal-adopts-and-the-spectrum-refuses´
/// ´test:crate:rank1-verification-refuses-the-rounding-witness´
#[cfg(feature = "serde")]
#[test]
fn rank1_verification_refuses_the_rounding_witness() {
    let witness = RoundingWitness::new();
    let config = SchurConfig::default();
    let delta = config.delta(FIXTURE_LAMBDA_PRIOR);
    let b_prime = witness.b_prime(delta);

    // Red: the retired check adopts this B'.
    let (_, min_diag) = b_prime.diagonal_min_max();
    assert!(
        min_diag > 0.0,
        "the witness must pass the diagonal test, or it witnesses nothing (got {min_diag})"
    );

    // Green: the spectrum refuses it.
    let (_, lambda_min) = eigenvalue_min_max(&b_prime).expect("a symmetric matrix has a spectrum");
    assert!(
        lambda_min < 0.0,
        "the witness must carry a negative eigenvalue (got {lambda_min})"
    );

    // The cheap certificate declines to certify rather than certifying
    // wrongly: the second row's off-diagonal magnitude exceeds its diagonal,
    // so its disc reaches across zero and the spectrum is consulted.
    let (passed, certificate) = verify_rank1_positive_definite(&b_prime);
    assert!(!passed, "the rank-one verification must refuse this B'");
    assert_eq!(
        certificate,
        VerificationCertificate::Factorisation,
        "Gershgorin is inconclusive here, so the factorisation must be what decided"
    );

    // And the call site follows the certificate rather than the diagonal. The
    // parent must reach the path without an SPD factorisation standing behind
    // it, which is what this case is about, so it is installed through the
    // unverified route rather than through a restore that would take its own
    // verdict first and change what is being tested.
    let mut model = BayesianLinearModel::from_stored_state_without_verdict(
        faer::Col::zeros(3),
        witness.parent(),
        SymmetricMatrix::identity_scaled(3, 1.0),
        FIXTURE_LAMBDA_PRIOR,
        0,
        1000,
        0,
        1.0,
        0.0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        None,
        0.0,
        Vec::new(),
    );
    let diagnostics = model.marginalise(&[2], &config, ModelId::Operational);

    assert!(
        !diagnostics.bprime_verification_passed,
        "B' carries a negative eigenvalue and must not be adopted"
    );
    assert!(diagnostics.fell_back_to_bkk, "the refusal falls back to B_kk");
    assert_eq!(
        diagnostics.bprime_verification_certificate,
        VerificationCertificate::Factorisation
    );
    assert_eq!(
        diagnostics.correction_outcome,
        SchurCorrectionOutcome::Applied,
        "no guard ahead of the verification fired: the pivot and the denominator are both healthy"
    );
    assert_eq!(model.dim(), 2, "the removal completes on the fallback");
}

/// A strictly diagonally dominant B' is certified by the discs alone, and the
/// diagnostics say so. This is the arm the arrangement exists for: the verdict
/// is the same verdict the spectrum would return, reached without computing a
/// spectrum, so the frequent path keeps the quadratic cost the corpus prices
/// it at (´tab:gaussian:operation-costs´). The matrix here is chosen so that
/// every disc clears the resolution floor by orders of magnitude rather than
/// by a margin the outward rounding could eat, since a certificate that
/// depended on the rounding direction to pass would be evidence that the
/// rounding was directed the wrong way (´req:gaussian:rank-one-verification´).
///
/// ´claim:bayes:a-diagonally-dominant-b-prime-is-certified-without-a-spectrum´
/// ´test:crate:rank1-gershgorin-certifies-a-dominant-bprime´
#[test]
fn rank1_gershgorin_certifies_a_dominant_bprime() {
    let mut data = faer::Mat::zeros(3, 3);
    for i in 0..3 {
        data[(i, i)] = 10.0;
    }
    data[(1, 0)] = 1.0;
    data[(2, 0)] = 0.5;
    data[(2, 1)] = 0.25;
    let b_prime = SymmetricMatrix::from_computation(data);

    let (passed, certificate) = verify_rank1_positive_definite(&b_prime);
    assert!(passed, "a strictly diagonally dominant B' is positive definite");
    assert_eq!(
        certificate,
        VerificationCertificate::Gershgorin,
        "the discs settle this one, so no spectrum should have been computed"
    );

    // The certificate agrees with the spectrum it stood in for. It has to:
    // the whole soundness argument is that a Gershgorin accept implies the
    // spectral accept, so a disagreement here is the arrangement failing.
    let (lambda_max, lambda_min) = eigenvalue_min_max(&b_prime).expect("a symmetric matrix has a spectrum");
    let floor = 8.0 * 3.0 * f64::EPSILON * lambda_max.max(0.0);
    assert!(
        lambda_min > floor,
        "the spectral rule must accept what the discs certified (λ_min = {lambda_min}, floor = {floor})"
    );
}

/// A non-positive diagonal entry is refused without a spectrum being computed.
/// The diagonal is still evidence in this one direction — a diagonal entry is
/// the Rayleigh quotient at a coordinate vector, so a non-positive one puts
/// λ_min at or below zero and the resolution floor is never negative — and
/// factorising the block to confirm what a sign already settles would spend
/// the cubic cost the whole arrangement exists to avoid
/// (´req:gaussian:rank-one-verification´). What this must not become is the
/// retired check read backwards: the accepting direction stays unavailable,
/// which the witness above is what pins.
///
/// ´claim:bayes:a-non-positive-diagonal-refuses-the-rank-one-correction-without-a-spectrum´
/// ´test:crate:rank1-diagonal-disproof-refuses-without-a-spectrum´
#[test]
fn rank1_diagonal_disproof_refuses_without_a_spectrum() {
    let mut data = faer::Mat::zeros(2, 2);
    data[(0, 0)] = 4.0;
    data[(1, 1)] = -1.0;
    let b_prime = SymmetricMatrix::from_computation(data);

    let (passed, certificate) = verify_rank1_positive_definite(&b_prime);
    assert!(!passed, "a negative diagonal entry is not positive definite");
    assert_eq!(certificate, VerificationCertificate::DiagonalDisproof);
}

/// The band the resolution floor closes is refused, and what stands above the
/// floor is accepted. A B' whose least eigenvalue lies strictly between zero
/// and the floor does not pass the verification, and the same B' raised so
/// that its least eigenvalue clears the floor by two orders does. What the
/// boundary buys is stated at (´const:assayer:schur-verification-resolution´):
/// at that distance from zero the arithmetic that decides the question carries
/// an error of the same size, so a verdict read there would be noise rather
/// than a reading of the requirement (´req:gaussian:positive-definiteness´).
/// Both matrices are `[[1 + s, 1 − t], [1 − t, 1 + s]]`, whose eigenvalues are
/// `s + t` and `2 + s − t`, with `s` and `t` powers of two so that the
/// placement relative to the floor is constructed rather than observed; the
/// floor is taken from the verification's own bound rather than recomputed
/// here, so the test cannot drift away from the boundary it is about. The
/// refusal names the pivot that failed and the floor it failed against, which
/// is what the fallback logs.
///
/// ´claim:bayes:the-verification-refuses-inside-the-resolution-band-and-accepts-above-it´
/// ´test:crate:verification-refuses-inside-the-resolution-band-and-accepts-above-it´
#[test]
fn verification_refuses_inside_the_resolution_band_and_accepts_above_it() {
    let witness = |shift: f64, gap: f64| {
        let mut data = faer::Mat::zeros(2, 2);
        data[(0, 0)] = 1.0 + shift;
        data[(1, 1)] = 1.0 + shift;
        data[(1, 0)] = 1.0 - gap;
        SymmetricMatrix::from_computation(data)
    };

    // A least eigenvalue of 2⁻⁴⁸ ≈ 3.6 × 10⁻¹⁵, against a floor of 16 ε λ_max
    // ≈ 7.1 × 10⁻¹⁵: inside the band by a factor of two.
    let gap = 2.0_f64.powi(-48);
    let inside = witness(0.0, gap);
    let floor = verification_floor(&inside).expect("a finite B' bounds its own spectrum");

    let (_, lambda_min) = eigenvalue_min_max(&inside).expect("a symmetric matrix has a spectrum");
    assert!(
        lambda_min > 0.0 && lambda_min < floor,
        "the witness must sit strictly inside the band (λ_min = {lambda_min}, floor = {floor})"
    );

    let (passed, certificate, refusal) = verify_positive_definite(&inside);
    assert!(
        !passed,
        "a least eigenvalue inside the band is not a verdict of positive definiteness"
    );
    assert_eq!(certificate, VerificationCertificate::Factorisation);
    let refusal = refusal.expect("a refused factorisation reports what refused it");
    assert_eq!(
        refusal.pivot_index, 1,
        "the leading 1 × 1 block is positive, so the second pivot is where the factorisation fails"
    );
    assert_near(
        refusal.floor,
        floor,
        DEFAULT_TOLERANCES.bit_identical,
        "the refusal names the floor the block was lowered by",
    );

    // The same matrix raised by 2⁻⁴⁰ ≈ 9.1 × 10⁻¹³, which is over a hundred
    // times the floor.
    let shift = 2.0_f64.powi(-40);
    let above = witness(shift, gap);
    let raised_floor = verification_floor(&above).expect("a finite B' bounds its own spectrum");
    let (_, raised_min) = eigenvalue_min_max(&above).expect("a symmetric matrix has a spectrum");
    assert!(
        raised_min > 64.0 * raised_floor,
        "the accepted witness must clear the floor by a comfortable multiple (λ_min = {raised_min}, floor = {raised_floor})"
    );

    let (raised_passed, raised_certificate, raised_refusal) = verify_positive_definite(&above);
    assert!(raised_passed, "a least eigenvalue well above the floor is positive definite");
    assert_eq!(raised_certificate, VerificationCertificate::Factorisation);
    assert!(
        raised_refusal.is_none(),
        "an accepted B' leaves nothing to report as a refusal"
    );
}

/// The ledger counts what share of verifications the cheap certificate
/// settled, and leaves the events that put nothing to a certificate out of the
/// denominator. That share is the measurement the arrangement was adopted on
/// and the one no analysis supplies: whether the discs are enough depends on
/// the precision matrices a deployment actually produces
/// (´req:gaussian:rank-one-verification´). A rate reported over a denominator
/// that included refusals-before-formation would read low for a reason that
/// has nothing to do with the certificate, which is why the vacuous events are
/// excluded rather than counted as misses.
///
/// ´claim:bayes:the-ledger-reports-the-cheap-certificates-acceptance-rate-over-the-events-that-asked-it´
/// ´test:crate:marginalisation-ledger-reports-the-certificate-share´
#[test]
fn marginalisation_ledger_reports_the_certificate_share() {
    let event = |certificate: VerificationCertificate| SchurDiagnostics {
        brr_cond_estimate: 1.0,
        brr_cond_measure: ConditionMeasure::Spectral,
        brr_regularised_cond: Some(1.0),
        correction_outcome: SchurCorrectionOutcome::Applied,
        bprime_verification_passed: true,
        bprime_verification_certificate: certificate,
        fell_back_to_bkk: false,
        error: MarginalisationError::NONE,
    };

    let mut ledger = MarginalisationErrorLedger::default();
    assert_eq!(
        ledger.gershgorin_acceptance_rate(),
        None,
        "no verification has happened, so there is no share to report"
    );

    ledger.record(&event(VerificationCertificate::Vacuous));
    assert_eq!(
        ledger.gershgorin_acceptance_rate(),
        None,
        "an event that put nothing to a certificate is outside the denominator"
    );

    ledger.record(&event(VerificationCertificate::Gershgorin));
    ledger.record(&event(VerificationCertificate::Gershgorin));
    ledger.record(&event(VerificationCertificate::Gershgorin));
    ledger.record(&event(VerificationCertificate::Factorisation));

    assert_eq!(ledger.verified_events, 4);
    assert_eq!(ledger.gershgorin_certified_events, 3);
    assert_eq!(ledger.events, 5, "every event is still counted as an event");
    assert_near(
        ledger.gershgorin_acceptance_rate().expect("four events were verified"),
        0.75,
        DEFAULT_TOLERANCES.bit_identical,
        "three of the four verifications stayed quadratic",
    );
}

/// The regularisation follows the prior it is derived from. Against the same
/// precision `[[2, 1], [1, 1]]` and the same configuration, removing the
/// second coordinate at a prior precision of one and at the default of 0.1
/// gives two different surviving precisions, and each names the δ it was
/// computed with exactly: B' is 2 − 1/(1 + δ) here, so 1/(2 − B') − 1
/// recovers what the computation added. The recovered values are 10⁻⁴ and
/// 10⁻⁵, the configured factor against each prior
/// (´alg:gaussian:regularised-schur´). Held as a fixed constant instead, δ
/// would be one value for both and the first recovery would fail — which is
/// the defect this fixes, the constant having agreed with the derivation
/// only at the default prior, so that a deployment moving its prior stopped
/// following the specification without saying so.
///
/// ´claim:bayes:the-schur-regularisation-scales-with-the-prior-it-is-derived-from´
/// ´test:crate:schur-delta-scales-with-lambda-prior´
#[test]
fn schur_delta_scales_with_lambda_prior() {
    let mut data = faer::Mat::zeros(2, 2);
    data[(0, 0)] = 2.0;
    data[(1, 1)] = 1.0;
    data[(0, 1)] = 1.0;
    data[(1, 0)] = 1.0;
    let b = SymmetricMatrix::from_computation(data);

    let remove = [1_usize];
    let keep = compute_keep_indices(2, &remove);
    let config = SchurConfig::default();

    // B' = B_kk − B_kr (B_rr + δ)⁻¹ B_rk = 2 − 1/(1 + δ) for this matrix, so
    // the δ the computation actually used is recoverable from its answer.
    let recovered_delta = |lambda_prior: f64| {
        let result = compute_schur_complement(&b, &remove, &keep, &config, lambda_prior);
        assert_eq!(result.diagnostics.correction_outcome, SchurCorrectionOutcome::Applied);
        1.0 / (2.0 - result.b_prime.as_inner()[(0, 0)]) - 1.0
    };

    assert_near(recovered_delta(1.0), 1e-4, 1e-12, "δ at a prior precision of one");
    assert_near(recovered_delta(FIXTURE_LAMBDA_PRIOR), 1e-5, 1e-12, "δ at the default prior");
    assert_near(config.delta(1.0), 1e-4, 1e-15, "the configuration derives the same δ");
}

/// The boundary is closed on the wrong side for the correction: a B' that is
/// positive *semi*-definite is refused, because the requirement asks for a
/// strictly positive minimum eigenvalue and a zero one is not that
/// (´req:gaussian:positive-definiteness´). Removing the third coordinate of
/// `[[1, 0, 1], [0, 1, 1], [1, 1, 2]]` with the regularisation switched off
/// gives B' = `[[0.5, −0.5], [−0.5, 0.5]]` exactly, whose eigenvalues are 1 and
/// 0. Its diagonal is positive throughout, so this is a second matrix the
/// retired test admitted; the singular precision it would have adopted has no
/// inverse, and the covariance the marginalisation goes on to restore is
/// exactly that inverse.
///
/// ´claim:bayes:a-semidefinite-b-prime-is-refused-at-the-boundary´
/// ´test:crate:bprime-semidefinite-boundary-refused´
#[test]
fn bprime_semidefinite_boundary_refused() {
    let p = 3;
    let mut data = faer::Mat::zeros(p, p);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = 1.0;
    data[(2, 2)] = 2.0;
    data[(0, 2)] = 1.0;
    data[(2, 0)] = 1.0;
    data[(1, 2)] = 1.0;
    data[(2, 1)] = 1.0;

    let b = SymmetricMatrix::from_computation(data);
    let remove = [2_usize];
    let keep = compute_keep_indices(p, &remove);
    // Regularisation off, so the boundary lands exactly on zero rather than
    // just inside it.
    let config = SchurConfig {
        epsilon_schur: 0.0,
        posture_condition_ceiling: 1e8,
    };

    // The B' the correction produces: B_kk − (1/2)·[[1, 1], [1, 1]].
    let mut expected = faer::Mat::zeros(2, 2);
    expected[(0, 0)] = 0.5;
    expected[(1, 1)] = 0.5;
    expected[(0, 1)] = -0.5;
    expected[(1, 0)] = -0.5;
    let b_prime = SymmetricMatrix::from_computation(expected);

    // Red: the retired test admits it — every diagonal entry is 0.5.
    let (_, min_diag) = b_prime.diagonal_min_max();
    assert!(
        min_diag > 0.0,
        "the witness must pass the diagonal test, or it witnesses nothing (got {min_diag})"
    );

    // The spectrum sits on the boundary rather than inside it.
    let (lambda_max, lambda_min) = eigenvalue_min_max(&b_prime).expect("a symmetric matrix has a spectrum");
    assert_near(lambda_max, 1.0, DEFAULT_TOLERANCES.default, "largest eigenvalue");
    assert_below(lambda_min.abs(), 1e-12, "least eigenvalue sits on zero");

    // Green: strictly positive is asked for, so the boundary is refused.
    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(
        !result.diagnostics.bprime_verification_passed,
        "a singular B' is not strictly positive definite and must not be adopted"
    );
    assert!(result.diagnostics.fell_back_to_bkk);
}

/// Two routes to the same correction agree element-wise: forming it as a Gram
/// product of a half-solved factor, and forming it by solving the full system
/// and multiplying back. The Gram route is chosen because it is positive
/// semi-definite by construction and so cannot produce a correction that
/// pushes precision the wrong way, and this fixes that the structural
/// guarantee costs nothing in the answer — across differing block sizes,
/// removal positions, and coupling strengths alike.
///
/// ´claim:bayes:the-gram-form-half-solve-and-the-full-solve-produce-the-same-correction´
/// ´test:crate:half-solve-equiv-full-solve´
#[test]
fn half_solve_equiv_full_solve() {
    // Test with multiple matrices of varying size and conditioning.
    let test_cases: &[(&[[f64; 4]; 4], &[usize])] = &[
        // Case 1: well-conditioned 4×4, remove last 2
        (
            &[
                [4.0, 1.0, 0.5, 0.2],
                [1.0, 3.0, 0.3, 0.1],
                [0.5, 0.3, 2.0, 0.4],
                [0.2, 0.1, 0.4, 2.5],
            ],
            &[2, 3],
        ),
        // Case 2: same matrix, remove dim 0 only (scalar r=1)
        (
            &[
                [4.0, 1.0, 0.5, 0.2],
                [1.0, 3.0, 0.3, 0.1],
                [0.5, 0.3, 2.0, 0.4],
                [0.2, 0.1, 0.4, 2.5],
            ],
            &[0],
        ),
        // Case 3: stronger cross-terms
        (
            &[
                [5.0, 2.0, 1.5, 0.8],
                [2.0, 4.0, 1.2, 0.6],
                [1.5, 1.2, 3.0, 0.9],
                [0.8, 0.6, 0.9, 3.5],
            ],
            &[2, 3],
        ),
    ];

    for (case_idx, (vals, remove)) in test_cases.iter().enumerate() {
        let data = faer::Mat::from_fn(4, 4, |i, j| vals[i][j]);
        let b = SymmetricMatrix::from_computation(data);
        let keep = compute_keep_indices(4, remove);
        let config = SchurConfig::default();

        // Extract blocks
        let b_rr = b.extract_symmetric_submatrix(remove);
        let b_kr = b.extract_cross_block(&keep, remove);
        let b_rk = b_kr.transpose();
        let b_rr_reg = b_rr.add_scaled_identity(config.delta(FIXTURE_LAMBDA_PRIOR));
        let llt = cholesky(&b_rr_reg).unwrap();

        // Option C: half-solve → W^T W (Gram matrix)
        let w = schur_half_solve(&llt, &b_rk.to_owned());
        let correction_c = w.transpose() * &w;

        // Method B: full-solve → B_kr · Y
        let y = cholesky_full_solve(&llt, &b_rk.to_owned());
        let correction_b = &b_kr * &y;

        // Compare per-element within 10⁻¹²
        let k = keep.len();
        let mut max_diff = 0.0_f64;
        for i in 0..k {
            for j in 0..k {
                let diff = (correction_c[(i, j)] - correction_b[(i, j)]).abs();
                max_diff = max_diff.max(diff);
                assert!(
                    diff < 1e-12,
                    "case {case_idx}: correction[{i},{j}] half-solve={} full-solve={} diff={diff:e}",
                    correction_c[(i, j)],
                    correction_b[(i, j)],
                );
            }
        }

        // Also verify the Schur complement result matches analytical expectation
        let result = compute_schur_complement(&b, remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
        assert!(!result.diagnostics.fell_back_to_bkk, "case {case_idx}: expected full Schur");
        assert_eq!(
            result.diagnostics.correction_outcome,
            SchurCorrectionOutcome::Applied,
            "case {case_idx}: expected Applied",
        );
    }
}

/// Removing more than one coordinate rebuilds the covariance by inverting the
/// corrected precision outright, so a model carrying two hundred labels' worth
/// of incremental drift comes out of the operation tightly synchronised again
/// rather than inheriting that drift. The multi-coordinate path already pays
/// for a factorisation, so it may as well discharge the accumulated error
/// while it is there.
///
/// ´claim:bayes:a-multi-coordinate-marginalisation-rebuilds-the-covariance-from-scratch-clearing-accumulated-drift´
/// ´test:crate:sigma-prime-from-fresh-cholesky´
#[test]
fn sigma_prime_from_fresh_cholesky() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);

    // Train to build cross-terms and accumulate SM drift
    train_model(&mut model, 200, p);

    // Marginalise last 2 dims
    model.marginalise(&[8, 9], &SchurConfig::default(), ModelId::Operational);

    // After marginalisation, BΣ ≈ I should be very tight
    // (fresh Cholesky, not inherited from drifted Σ)
    let dev = frobenius_deviation_from_identity(model.precision(), model.covariance());
    assert_below(dev, 1e-10, "||BΣ - I||_F after fresh Cholesky");
}

/// A model that has accumulated a real label count comes out of a
/// multi-coordinate removal with that count back at zero. The fresh
/// factorisation performed there has paid off the drift the counter was
/// tracking, so the next scheduled recomputation is measured from here rather
/// than being brought forward for work already done.
///
/// ´claim:bayes:a-multi-dimension-marginalisation-restarts-the-recompute-cadence´
/// ´test:crate:labels-since-recompute-reset´
#[test]
fn labels_since_recompute_reset() {
    let p = 10;
    let mut model = BayesianLinearModel::new(p, 0.1, 1000);
    train_model(&mut model, 50, p);
    assert!(model.labels_since_recompute() > 0);

    model.marginalise(&[8, 9], &SchurConfig::default(), ModelId::Operational);
    assert_eq!(model.labels_since_recompute(), 0);
}

/// Against an identically trained twin marginalised by bare submatrix
/// extraction, the corrected path never claims more precision on any surviving
/// diagonal, and claims strictly less on at least one. Dropping a coordinate
/// costs the model an information channel; the bare submatrix keeps the
/// precision it had and so becomes overconfident, while the correction is what
/// widens the posterior by the right amount.
///
/// ´claim:bayes:the-corrected-marginal-never-claims-more-precision-than-the-bare-submatrix-and-usually-claims-less´
/// ´test:crate:schur-vs-bkk-ordering´
#[test]
fn schur_vs_bkk_ordering() {
    let p = 15;
    let prior_precision = 0.1;
    let mut model_schur = BayesianLinearModel::new(p, prior_precision, 1000);
    let mut model_bkk = BayesianLinearModel::new(p, prior_precision, 1000);

    // Train both identically
    let n_labels = 200;
    let mut rng = TestRng::new(RNG_SEED);
    for _ in 0..n_labels {
        let phi: Vec<f64> = (0..p).map(|_| signed_half(&mut rng)).collect();
        let phi_hat = vec_to_col(&phi);
        let r_rho = if rng.next_range(5) == 0 { 1.0 } else { -1.0 };

        // Update model_schur
        let (v, h) = model_schur.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff = compute_effective_weight(1.0, 100.0, 5.0, h, 1e-8);
        model_schur.apply_leverage_bounded_update(&phi_hat, &v, h, w_eff, r_rho, 0.999, 1e-4);

        // Update model_bkk identically
        let (v2, h2) = model_bkk.covariance().quadratic_form_with_product(&phi_hat);
        let w_eff2 = compute_effective_weight(1.0, 100.0, 5.0, h2, 1e-8);
        model_bkk.apply_leverage_bounded_update(&phi_hat, &v2, h2, w_eff2, r_rho, 0.999, 1e-4);
    }

    // Marginalise dims 10..15 via Schur (model_schur) and B_kk (model_bkk)
    let remove_indices: Vec<usize> = (10..15).collect();
    let keep_indices = compute_keep_indices(p, &remove_indices);

    // Schur path
    let schur_diag = model_schur.marginalise(&remove_indices, &SchurConfig::default(), ModelId::Operational);

    // B_kk path: extract B_kk manually and use cholesky_inverse
    let b_kk = model_bkk.precision().extract_symmetric_submatrix(&keep_indices);

    // Compare: Schur B' should have larger or equal diagonal entries compared
    // to B_kk on the surviving dimensions (correction subtracts a PSD matrix,
    // so B' ≤ B_kk element-wise... actually B' = B_kk - C where C ≥ 0).
    //
    // Wait — B' = B_kk - correction means B' has LESS precision (smaller
    // diagonals). But the Schur complement IS the correct marginal, while
    // B_kk over-estimates precision by ignoring cross-terms.
    //
    // So B_kk ≥ B' element-wise on diagonals (B_kk is MORE precise, which
    // is WRONG — it's overconfident). The Schur path correctly reduces
    // precision to account for the lost information channel.
    //
    // For the covariance: Σ'_schur ≥ Σ'_bkk (wider, more uncertain = correct).
    //
    // Verify: at least one diagonal of B' < B_kk (correction was non-zero).
    if !schur_diag.fell_back_to_bkk {
        let mut any_strictly_less = false;
        for i in 0..keep_indices.len() {
            let schur_diag_val = model_schur.precision().diagonal_element(i);
            let bkk_diag_val = b_kk.diagonal_element(i);
            assert!(
                schur_diag_val <= bkk_diag_val + 1e-10,
                "B'[{i},{i}] = {} > B_kk[{i},{i}] = {} (Schur should not increase precision)",
                schur_diag_val,
                bkk_diag_val,
            );
            if bkk_diag_val - schur_diag_val > 1e-10 {
                any_strictly_less = true;
            }
        }
        assert!(
            any_strictly_less,
            "Schur correction should reduce at least one diagonal (cross-terms present)"
        );
    }
}

/// With nothing named for removal the precision comes back at its original
/// dimension with every diagonal exactly as it was, and the result is reported
/// as a full correction rather than a fallback. The degenerate case takes the
/// ordinary path and produces the identity operation, so nothing upstream has
/// to special-case an empty removal set.
///
/// ´claim:bayes:removing-no-dimensions-leaves-the-precision-exactly-as-it-was´
/// ´test:crate:empty-remove-noop´
#[test]
fn empty_remove_noop() {
    let p = 5;
    let mut data = faer::Mat::zeros(p, p);
    for i in 0..p {
        data[(i, i)] = 1.0 + i as f64;
    }
    let b = SymmetricMatrix::from_computation(data);

    let remove: [usize; 0] = [];
    let keep = compute_keep_indices(p, &remove);
    let config = SchurConfig::default();

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(!result.diagnostics.fell_back_to_bkk);
    assert_eq!(result.b_prime.dim(), p);

    let tol = DEFAULT_TOLERANCES.bit_identical;
    for i in 0..p {
        assert_near(
            result.b_prime.diagonal_element(i),
            b.diagonal_element(i),
            tol,
            &format!("noop: diagonal[{i}]"),
        );
    }
}

/// Marginalisation does not fail: driving the double failure through it —
/// a negative-definite precision under a regularisation configuration
/// whose cascade cannot repair a pivot — completes the removal anyway.
/// The dimension comes down, the uncorrected kept block is adopted as the
/// precision, the covariance is the maintained one's kept block value for
/// value, and the outcome travels in the diagnostics as a flagged
/// factorisation failure rather than in a Result. Before this repair the
/// same construction returned an error the lifecycle path answered by
/// abandoning the deregistration, so a caller had to handle a case the
/// corpus promises does not exist.
///
/// ´claim:bayes:a-factorisation-failure-inside-marginalisation-retains-and-flags-instead-of-erroring´
/// ´test:crate:marginalise-terminus-retains-and-flags´
#[cfg(feature = "serde")]
#[test]
fn marginalise_terminus_retains_and_flags() {
    let p = 5;
    // A negative-definite precision: every pivot fails, and with both
    // regularisation parameters zero the cascade cannot repair one.
    let mut neg = faer::Mat::zeros(p, p);
    for i in 0..p {
        neg[(i, i)] = -1.0;
    }
    let precision = SymmetricMatrix::from_computation(neg);
    // The maintained covariance: a recognisable diagonal, so retention
    // is checkable value for value.
    let mut cov = faer::Mat::zeros(p, p);
    for i in 0..p {
        #[allow(clippy::cast_precision_loss)] // Justified: tiny test index
        let v = 2.0 + i as f64;
        cov[(i, i)] = v;
    }
    let covariance = SymmetricMatrix::from_computation(cov);
    let mu = faer::Col::from_fn(p, |i| i as f64);

    // A negative-definite precision is the premise, so no restore installs
    // it: the unverified route is what puts the model in the state whose
    // marginalisation this test is about.
    let mut model = BayesianLinearModel::from_stored_state_without_verdict(
        mu,
        precision,
        covariance,
        FIXTURE_LAMBDA_PRIOR,
        0,
        1000,
        0,
        1.0,
        0.0,
        0,
        0,
        0,
        0,
        0,
        0,
        0,
        None,
        0.0,
        Vec::new(),
    );

    let diag = model.marginalise(&[3, 4], &SchurConfig::default(), ModelId::Operational);

    // The removal completed: no error channel exists and the dimension
    // came down.
    assert_eq!(model.dim(), 3, "the removal completes despite the terminus");
    assert!(diag.fell_back_to_bkk, "the fallback is flagged");
    assert!(
        matches!(diag.correction_outcome, SchurCorrectionOutcome::FactorisationFailed { .. }),
        "the terminus travels in the diagnostics: {:?}",
        diag.correction_outcome
    );

    // The covariance is the maintained one's kept block, value for value.
    for i in 0..3 {
        #[allow(clippy::cast_precision_loss)] // Justified: tiny test index
        let expected = 2.0 + i as f64;
        assert!(
            (model.covariance().as_inner()[(i, i)] - expected).abs() < 1e-12,
            "kept covariance diagonal {i} retained"
        );
    }
    // The precision is the uncorrected kept block of the original.
    for i in 0..3 {
        assert!(
            (model.precision().as_inner()[(i, i)] - (-1.0)).abs() < 1e-12,
            "kept precision diagonal {i} is the uncorrected block"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The error result and the two guards
// ═══════════════════════════════════════════════════════════════════════════════

/// The reported error is the loss actually committed, not the offset that
/// caused it. Removing the second coordinate of `[[1, 0.001], [0.001, 0.001]]`
/// leaves a removed block whose least eigenvalue is a thousandth, so at the
/// default offset of 10⁻⁵ the correction loses δ/(a + δ), about one part in a
/// hundred — three decimal orders more than the offset. Both figures are
/// checked against the closed form and the share is checked to be far above δ,
/// which is the reading this replaces: the audit showed that the bare offset
/// bounds nothing, since the loss scales with the removed block's spectrum and
/// the coupling, and a budget written as δ would understate this ordinary case
/// by a factor of a thousand.
///
/// ´claim:bayes:the-marginalisation-error-is-the-computed-loss-and-not-the-offset´
/// ´test:crate:marginalisation-error-is-the-measured-loss´
#[test]
fn marginalisation_error_is_the_measured_loss() {
    let pivot = 0.001;
    let coupling = 0.001;
    let mut data = faer::Mat::zeros(2, 2);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = pivot;
    data[(0, 1)] = coupling;
    data[(1, 0)] = coupling;
    let b = SymmetricMatrix::from_computation(data);

    let remove = [1_usize];
    let keep = compute_keep_indices(2, &remove);
    let config = SchurConfig::default();
    let delta = config.delta(FIXTURE_LAMBDA_PRIOR);

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(!result.diagnostics.fell_back_to_bkk, "the block is well conditioned");

    // The exact correction's trace is c²/a and the applied one's is c²/(a + δ),
    // so the loss is that difference and the share of it is δ/(a + δ).
    let expected_share = delta / (pivot + delta);
    let coupling_mass = coupling * coupling;
    let expected_mass = coupling_mass / pivot - coupling_mass / (pivot + delta);

    let MarginalisationError::Measured {
        discarded_precision_mass,
        correction_loss_fraction,
    } = result.diagnostics.error
    else {
        panic!(
            "a removed block of condition number one puts the exact error in reach, got {:?}",
            result.diagnostics.error
        );
    };
    assert_near(discarded_precision_mass, expected_mass, 1e-16, "discarded precision mass");
    assert_near(correction_loss_fraction, expected_share, 1e-14, "correction loss fraction");
    assert!(
        correction_loss_fraction > 100.0 * delta,
        "the share lost is {correction_loss_fraction}, which the offset {delta} does not bound"
    );
}

/// Discarding the correction is reported as the error it is. The same matrix
/// whose loss is a hundredth under the offset loses the whole correction when
/// the host's posture refuses the block, and the report says so: the share is
/// exactly one and the mass is the exact correction's own trace. This is the
/// offset's endpoint — as the regularisation grows without bound the
/// correction vanishes and the result approaches the kept block — so a fallback
/// is the maximum of this same approximation rather than an exact path beside
/// it. Reporting nothing here, as the code did before, made the most
/// approximate outcome the one the host could not see.
///
/// ´claim:bayes:a-discarded-correction-is-reported-as-an-error-of-its-own-whole-size´
/// ´test:crate:discarded-correction-reports-its-whole-size´
#[test]
fn discarded_correction_reports_its_whole_size() {
    let pivot = 0.001;
    let coupling = 0.001;
    let mut data = faer::Mat::zeros(2, 2);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = pivot;
    data[(0, 1)] = coupling;
    data[(1, 0)] = coupling;
    let b = SymmetricMatrix::from_computation(data);

    let remove = [1_usize];
    let keep = compute_keep_indices(2, &remove);
    // A posture that refuses everything, so the same block is declined.
    let config = SchurConfig {
        epsilon_schur: 1e-4,
        posture_condition_ceiling: 0.5,
    };

    let result = compute_schur_complement(&b, &remove, &keep, &config, FIXTURE_LAMBDA_PRIOR);
    assert!(result.diagnostics.fell_back_to_bkk);
    assert_eq!(
        result.diagnostics.correction_outcome,
        SchurCorrectionOutcome::PostureGuardSkip
    );

    let MarginalisationError::Measured {
        discarded_precision_mass,
        correction_loss_fraction,
    } = result.diagnostics.error
    else {
        panic!("the exact correction is reachable here, got {:?}", result.diagnostics.error);
    };
    assert_near(
        discarded_precision_mass,
        coupling * coupling / pivot,
        1e-16,
        "the whole correction's trace",
    );
    assert_near(correction_loss_fraction, 1.0, 1e-14, "the whole correction was lost");
}

/// One block, two guards, and which fired is in the diagnostics. The removed
/// block `diag(10⁸, 10⁻⁸)` has a raw condition number of 10¹⁶ and, at the
/// default offset, a regularised one near 10¹³. Under the shipped posture it is
/// the host's guard that refuses it, as a source too degraded to fold in. Under
/// a posture raised past 10¹⁸ — a deployment declaring itself willing to take
/// such a source — the same block is refused anyway, by the package's own
/// solvability guard, because the matrix the correction would be solved against
/// is beyond what the arithmetic supports. The second refusal is the one a host
/// cannot configure away, which is what makes the two guards two and not one:
/// the audit found a single threshold standing for both protections, and at the
/// reference scale regularisation moves a raw 10⁸ to a regularised 10⁴, so the
/// two questions have different answers on the same matrix.
///
/// ´claim:bayes:the-two-guards-refuse-separately-and-the-diagnostics-name-which-refused´
/// ´test:crate:two-guards-fire-distinctly´
#[test]
fn two_guards_fire_distinctly() {
    let p = 4;
    let mut data = faer::Mat::zeros(p, p);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = 1.0;
    data[(2, 2)] = 1e8;
    data[(3, 3)] = 1e-8;
    let b = SymmetricMatrix::from_computation(data);

    let remove = [2_usize, 3];
    let keep = compute_keep_indices(p, &remove);

    let shipped = SchurConfig::default();
    let refused_by_host = compute_schur_complement(&b, &remove, &keep, &shipped, FIXTURE_LAMBDA_PRIOR);
    assert_eq!(
        refused_by_host.diagnostics.correction_outcome,
        SchurCorrectionOutcome::PostureGuardSkip,
        "at the shipped posture the host's guard is the binding one"
    );
    assert!(refused_by_host.diagnostics.fell_back_to_bkk);

    let permissive = SchurConfig {
        epsilon_schur: 1e-4,
        posture_condition_ceiling: 1e18,
    };
    let refused_by_us = compute_schur_complement(&b, &remove, &keep, &permissive, FIXTURE_LAMBDA_PRIOR);
    assert_eq!(
        refused_by_us.diagnostics.correction_outcome,
        SchurCorrectionOutcome::SolvabilityGuardSkip,
        "a raised posture reaches our guard, which it cannot move"
    );
    assert!(refused_by_us.diagnostics.fell_back_to_bkk);

    // The regularised figure the second guard read is the raw spectrum shifted
    // by δ, and it is the smaller of the two: regularisation is what the
    // package's guard is protecting, so it reads the matrix after the lift.
    let raw = refused_by_us.diagnostics.brr_cond_estimate;
    let regularised = refused_by_us
        .diagnostics
        .brr_regularised_cond
        .expect("the spectrum was available, so the solvability guard had a figure to read");
    assert!(
        regularised < raw,
        "the regularised condition {regularised} must be below the raw {raw}"
    );
}

/// The host's guard reads the spectrum, and a diagonal ratio would have let
/// this block through. The audit's own counterexample is a removed block
/// `[[1, 1 − 10⁻¹²], [1 − 10⁻¹², 1]]`: every diagonal entry is one, so the
/// ratio the code used to test is exactly one and passes any ceiling, while the
/// true condition number is about 2 × 10¹² and the block is nearly singular.
/// The guard now refuses it, and the diagnostics say the figure is a
/// measurement of the spectrum rather than the bound the ratio was. A lower
/// bound that small certifies nothing, which is why the estimator could not be
/// left standing under a guard whose whole purpose is refusal.
///
/// ´claim:bayes:the-posture-guard-reads-the-spectrum-and-refuses-what-a-diagonal-ratio-admits´
/// ´test:crate:posture-guard-reads-the-spectrum´
#[test]
fn posture_guard_reads_the_spectrum() {
    let p = 4;
    let off = 1.0 - 1e-12;
    let mut data = faer::Mat::zeros(p, p);
    data[(0, 0)] = 1.0;
    data[(1, 1)] = 1.0;
    data[(2, 2)] = 1.0;
    data[(3, 3)] = 1.0;
    data[(2, 3)] = off;
    data[(3, 2)] = off;
    let b = SymmetricMatrix::from_computation(data);

    let remove = [2_usize, 3];
    let keep = compute_keep_indices(p, &remove);

    // The retired estimator's verdict, computed here so the disagreement is
    // visible in the test rather than only in a commit message.
    let b_rr = b.extract_symmetric_submatrix(&remove);
    let (max_diagonal, min_diagonal) = b_rr.diagonal_min_max();
    assert_near(max_diagonal / min_diagonal, 1.0, 1e-15, "the diagonal ratio is one");

    let result = compute_schur_complement(&b, &remove, &keep, &SchurConfig::default(), FIXTURE_LAMBDA_PRIOR);
    assert_eq!(
        result.diagnostics.brr_cond_measure,
        ConditionMeasure::Spectral,
        "the figure the guard read is the spectrum"
    );
    assert!(
        result.diagnostics.brr_cond_estimate > 1e11,
        "the true condition number is about 2e12, got {}",
        result.diagnostics.brr_cond_estimate
    );
    assert_eq!(
        result.diagnostics.correction_outcome,
        SchurCorrectionOutcome::PostureGuardSkip,
        "the block a diagonal ratio would have admitted is refused"
    );
}

/// A configuration the host sets reaches the computation. The same trained
/// model is marginalised twice from identical copies, once at the shipped
/// regularisation factor and once at a factor a hundred times larger, and the
/// surviving precisions differ. They cannot differ unless the factor travelled
/// from the configuration through the model owner's working copy and into the
/// Schur complement, which is exactly the journey it did not make before: both
/// marginalisation call sites built their own default configuration and
/// discarded the exposed one, so a deployment could set the factor and change
/// nothing. The larger factor attenuates more of the correction, so its
/// surviving precision is the larger one — the direction the regularisation's
/// one-sided bias predicts.
///
/// ´claim:bayes:the-configured-schur-factor-reaches-the-marginalisation´
/// ´test:crate:host-schur-configuration-reaches-the-marginalisation´
#[test]
fn host_schur_configuration_reaches_the_marginalisation() {
    let p = 8;
    let remove = [7_usize];

    let mut shipped_model = BayesianLinearModel::new(p, FIXTURE_LAMBDA_PRIOR, 1000);
    train_model(&mut shipped_model, 200, p);
    let mut raised_model = BayesianLinearModel::new(p, FIXTURE_LAMBDA_PRIOR, 1000);
    train_model(&mut raised_model, 200, p);
    assert_near(
        shipped_model.precision().as_inner()[(0, 0)],
        raised_model.precision().as_inner()[(0, 0)],
        0.0,
        "the two copies start identical",
    );

    let shipped = SchurConfig::default();
    let raised = SchurConfig {
        epsilon_schur: 1e-2,
        posture_condition_ceiling: 1e8,
    };

    let shipped_diagnostics = shipped_model.marginalise(&remove, &shipped, ModelId::Operational);
    let raised_diagnostics = raised_model.marginalise(&remove, &raised, ModelId::Operational);
    assert!(!shipped_diagnostics.fell_back_to_bkk);
    assert!(!raised_diagnostics.fell_back_to_bkk);

    let shipped_survivor = shipped_model.precision().as_inner()[(0, 0)];
    let raised_survivor = raised_model.precision().as_inner()[(0, 0)];
    assert!(
        raised_survivor > shipped_survivor,
        "the larger factor must attenuate more of the correction: {raised_survivor} against {shipped_survivor}"
    );

    // And the error each reports moves with the factor it was computed at.
    let shipped_loss = shipped_diagnostics
        .error
        .correction_loss_fraction()
        .expect("a healthy block reports a figure");
    let raised_loss = raised_diagnostics
        .error
        .correction_loss_fraction()
        .expect("a healthy block reports a figure");
    assert!(
        raised_loss > shipped_loss,
        "a hundredfold factor loses more of the correction: {raised_loss} against {shipped_loss}"
    );
}

/// The ledger folds events rather than reporting only the last one. Three
/// events are recorded — a measured loss, a bound, and a removal with no finite
/// figure — and each lands in its own count while the mass accumulates over the
/// two that carried one and the worst share is the largest of them. The split
/// between the counts is the point: a cumulative mass mixing measurements with
/// bounds would be read as a level, and the counts are what say how much of it
/// is which. The unbounded event moves no figure at all, because there is
/// nothing honest to move it by, and is counted so that its silence is visible.
///
/// ´claim:bayes:the-marginalisation-ledger-folds-events-and-keeps-measurements-apart-from-bounds´
/// ´test:crate:marginalisation-ledger-folds-events´
#[test]
fn marginalisation_ledger_folds_events() {
    let event = |error: MarginalisationError, fell_back: bool| SchurDiagnostics {
        brr_cond_estimate: 1.0,
        brr_cond_measure: ConditionMeasure::Spectral,
        brr_regularised_cond: Some(1.0),
        correction_outcome: SchurCorrectionOutcome::Applied,
        bprime_verification_passed: true,
        bprime_verification_certificate: VerificationCertificate::Factorisation,
        fell_back_to_bkk: fell_back,
        error,
    };

    let mut ledger = MarginalisationErrorLedger::default();
    ledger.record(&event(
        MarginalisationError::Measured {
            discarded_precision_mass: 0.25,
            correction_loss_fraction: 0.1,
        },
        false,
    ));
    ledger.record(&event(
        MarginalisationError::Bounded {
            discarded_precision_mass: 0.5,
            correction_loss_fraction: 0.4,
        },
        true,
    ));
    ledger.record(&event(MarginalisationError::Unbounded, true));

    assert_eq!(ledger.events, 3);
    assert_eq!(ledger.measured_events, 1);
    assert_eq!(ledger.bounded_events, 1);
    assert_eq!(ledger.unbounded_events, 1);
    assert_eq!(ledger.fallback_events, 2);
    assert_eq!(ledger.corrections_skipped(), 2);
    assert_near(
        ledger.cumulative_discarded_precision_mass,
        0.75,
        1e-15,
        "the two events carrying a figure",
    );
    assert_near(ledger.worst_correction_loss_fraction, 0.4, 1e-15, "the largest share lost");
}

/// At the dimensions the system actually runs at — a 638-coordinate model
/// losing a 68-coordinate block — a marginalisation costs a few hundred
/// milliseconds, so ten of them fit inside a three-second budget. Marginalising
/// happens on a live model, and a cubic-cost operation at this size has to be
/// known to be affordable rather than assumed to be. Kept off the default run
/// because paying that cost on every test invocation is not.
///
/// ´claim:bayes:marginalising-a-large-block-at-the-reference-dimension-stays-within-its-time-budget´
/// ´test:crate:schur-complement-perf´
#[test]
#[ignore = "expensive: 638-dim matrix, run with --ignored"]
fn schur_complement_perf() {
    // r=68 (one Sentinel slot plus interactions at the reference config), p=638 -> k=570
    let p = 638;
    let r = 68;
    let prior_precision = 0.1;
    let mut model = BayesianLinearModel::new(p, prior_precision, 1000);

    // Quick train to make matrix non-trivial
    train_model(&mut model, 50, p);

    let remove_indices: Vec<usize> = (p - r..p).collect();

    let start = std::time::Instant::now();
    for _ in 0..10 {
        let mut m = BayesianLinearModel::new(p, prior_precision, 1000);
        train_model(&mut m, 10, p);
        m.marginalise(&remove_indices, &SchurConfig::default(), ModelId::Operational);
    }
    let elapsed = start.elapsed();
    // Budget: < 3.0s for 10 marginalisations (~247 ms each)
    assert!(
        elapsed.as_secs_f64() < 3.0,
        "10 marginalisations took {elapsed:?} (budget: 3.0s)"
    );
}
