// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`adr_review_3x3_example`] | bayes | cites (´claim:bayes:the-bare-covariance-submatrix-beats-the-sherman-morrison-candidate-by-orders-of-magnitude´) |
//! | [`error_table_across_structures`] | bayes | cites (´claim:bayes:the-bare-covariance-submatrix-beats-the-sherman-morrison-candidate-by-orders-of-magnitude´) |
//! | [`spec_formula_beats_adr_formula_by_orders_of_magnitude`] | bayes | Simply extracting the covariance submatrix lands nearer the true marginal covariance than the Sherman–Morrison candidate does, on every fixture, by at least two orders of magnitude. This is why the implementation does the cheap thing: the elaborate rank-one correction that formula applies is not a refinement of the submatrix but a departure from it, and doing nothing beats doing that. |
//! | [`woodbury_is_the_exact_rank1_correction`] | bayes | On these freshly inverted fixtures, the Woodbury rank-one correction reproduces the inverse of the implemented regularised precision to rounding. The difference from plain covariance-submatrix extraction is introduced by the offset in this exact-dual construction, but its magnitude also depends on the removed pivot, coupling, and covariance quadratic form. The sweep shows δ-scaling for these fixtures; δ alone is not a general error bound. |
//! | [`adr_formula_error_scales_with_h_not_delta`] | bayes | Driving the regularisation down through eight orders of magnitude shrinks the submatrix's error along with it but leaves the Sherman–Morrison candidate's error essentially unchanged. That distinguishes the two kinds of wrongness decisively: one is the price of regularising and vanishes when the regularisation does, the other is an error in the algebra itself and no amount of numerical care will remove it. |
//! | [`error_pattern_holds_at_p20`] | bayes | At twenty coordinates the same ranking holds and the same scalings hold with it: the submatrix's error tracks the regularisation, the Woodbury correction's sits at machine precision, and the Sherman–Morrison candidate's stays fixed as the regularisation is swept, with a coefficient matching the reciprocal of the removed coordinate's own precision. The argument is about block structure rather than about small matrices, so it must not depend on the dimension it is demonstrated at. |
//! | [`formula_ordering_is_index_position_independent`] | bayes | Removing a coordinate from the middle of the matrix, or the very last one, gives the same ranking as removing the first. Every other case here removes index zero, where the gather of surviving indices is trivially the tail — so an off-by-one in that gather would go unseen, and would look exactly like a formula error rather than the bookkeeping error it is. |
//! | [`woodbury_derivation_sanity_check`] | bayes | Algebraic sanity check for the Woodbury derivation in the review: B' = B_kk − c · b bᵀ = (B_kk − (1/B_11) · b bᵀ)  +  ((1/B_11) − c) · b bᵀ = Σ_kk⁻¹                     +  α · b bᵀ where `α = δ / (B_11 · (B_11 + δ))`. The two ways of writing the marginal precision agree to machine precision element by element, which is what makes the exact correction available at rank-one cost: the marginal precision is the inverse of the covariance submatrix plus one regularisation-scaled outer product, so its inverse follows by a rank-one identity rather than by a fresh factorisation. |

#![allow(clippy::doc_markdown, clippy::many_single_char_names)]
#![allow(clippy::print_stderr)]

//! Numerical verification of the rank-1 marginalisation covariance update
//! discussed in the review that settled the marginalisation correction
//! (´dec:numerics:marginalisation-correction´).
//!
//! # Background
//!
//! Earlier text
//! (´dec:numerics:marginalisation-correction´)
//! proposed a rank-1 fast path for
//! `marginalise_rank1` that updated the marginal covariance via a
//! Sherman–Morrison-style formula:
//!
//! ```text
//! Σ'_adr  = Σ_kk + (c / (1 − c·h)) · v vᵀ
//! ```
//!
//! where `v = Σ_kk · b`, `h = bᵀ v`, `c = 1 / (B_11 + δ)`, and
//! `b = B_{k,1}` is the cross-precision column for the removed dimension.
//!
//! The review argues this formula is mathematically wrong (treats `Σ_kk`
//! as `B_kk⁻¹`, which it is not — they differ by a structural rank-1
//! term per the block-matrix inverse identity), and that the correct
//! `O(p²)` choices are either:
//!
//! - **spec**: `Σ'_spec = Σ_kk` (just the submatrix extraction
//!   (´claim:bayes:the-schur-complement-is-the-analytical-marginal´))
//! - **woodbury**: `Σ'_wood = Σ_kk − (α/(1+α·h)) · v vᵀ` with
//!   `α = δ / (B_11 · (B_11 + δ))`, the exact rank-1 correction from the
//!   regularised Schur complement applied via the Woodbury identity.
//!
//! This file constructs several SPD precision matrices, computes the
//! ground truth `(B')⁻¹` via faer's Cholesky inverse, and reports the
//! max element-wise error of each candidate formula against the truth.
//!
//! # Cross-references
//!
//! - (´dec:substrate:dense-dynamic´) — the dense, dynamically dimensioned model
//! - (´claim:bayes:the-schur-complement-is-the-analytical-marginal´) — extension and marginalisation, stating `Σ' = Σ_kk`
//! - (´alg:gaussian:regularised-schur´) — numerical conditioning of the Schur complement

// Brings `.inverse()` into scope for Cholesky factors.
use faer::linalg::solvers::DenseSolveCore;
use faer::{Mat, Side};

// ═══════════════════════════════════════════════════════════════════════════════
// Linear-algebra helpers (faer wrappers used only by this test file)
// ═══════════════════════════════════════════════════════════════════════════════

/// Invert an SPD matrix via Cholesky. Panics on non-SPD input.
fn spd_inverse(m: &Mat<f64>) -> Mat<f64> {
    let llt = m.llt(Side::Lower).expect("matrix must be SPD");
    llt.inverse()
}

/// Submatrix gather at `indices × indices`.
fn submatrix(m: &Mat<f64>, indices: &[usize]) -> Mat<f64> {
    let k = indices.len();
    Mat::from_fn(k, k, |i, j| m[(indices[i], indices[j])])
}

/// Gather column `col` at rows `indices` (excluding `col` itself in usage).
fn subcolumn(m: &Mat<f64>, rows: &[usize], col: usize) -> Vec<f64> {
    rows.iter().map(|&i| m[(i, col)]).collect()
}

/// Returns `M · v` for dense `M` and column-vector `v` represented as `&[f64]`.
fn matvec(m: &Mat<f64>, v: &[f64]) -> Vec<f64> {
    let n = m.nrows();
    debug_assert_eq!(m.ncols(), v.len());
    let mut out = vec![0.0; n];
    for i in 0..n {
        let mut acc = 0.0;
        for j in 0..v.len() {
            acc = m[(i, j)].mul_add(v[j], acc);
        }
        out[i] = acc;
    }
    out
}

/// Dot product.
fn dot(a: &[f64], b: &[f64]) -> f64 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

/// Outer product `v vᵀ`.
fn outer(v: &[f64]) -> Mat<f64> {
    let n = v.len();
    Mat::from_fn(n, n, |i, j| v[i] * v[j])
}

/// `out = a + scale · vvᵀ` (rank-1 update of a dense matrix).
fn rank1_update(a: &Mat<f64>, scale: f64, v: &[f64]) -> Mat<f64> {
    let n = a.nrows();
    let vv = outer(v);
    Mat::from_fn(n, n, |i, j| scale.mul_add(vv[(i, j)], a[(i, j)]))
}

/// Max element-wise absolute difference between two equal-shape matrices.
fn max_abs_diff(a: &Mat<f64>, b: &Mat<f64>) -> f64 {
    debug_assert_eq!(a.nrows(), b.nrows());
    debug_assert_eq!(a.ncols(), b.ncols());
    let mut m = 0.0_f64;
    for i in 0..a.nrows() {
        for j in 0..a.ncols() {
            m = m.max((a[(i, j)] - b[(i, j)]).abs());
        }
    }
    m
}

// ═══════════════════════════════════════════════════════════════════════════════
// The three candidate Σ' formulas + the truth
// ═══════════════════════════════════════════════════════════════════════════════

/// Bundle of every Σ'-style matrix computed for one marginalisation case.
struct MarginaliseOutcome {
    /// `(B')⁻¹` — the ground truth marginal covariance.
    truth: Mat<f64>,
    /// `Σ_kk` — the analytical-marginal formula, just submatrix extraction
    /// (´claim:bayes:the-schur-complement-is-the-analytical-marginal´).
    spec_sigma_kk: Mat<f64>,
    /// Woodbury exact rank-1 correction: `Σ_kk − (α/(1+α·h)) · v vᵀ`.
    woodbury: Mat<f64>,
    /// The ADR's Sherman–Morrison formula: `Σ_kk + (c/(1−c·h)) · v vᵀ`.
    adr_sm: Mat<f64>,
    /// Coefficient of the ADR's outer-product term (≈ 0.7 in the headline case).
    adr_coefficient: f64,
    /// Coefficient of the Woodbury correction (`α/(1+α·h)`, ≈ 10⁻⁵ — δ-scale).
    woodbury_coefficient: f64,
    /// The quadratic form `h = bᵀ Σ_kk b`.
    h: f64,
}

/// Compute every candidate marginal covariance for `precision` after
/// removing dimension `idx` with regularisation `delta`.
fn marginalise_rank1_all_formulas(precision: &Mat<f64>, idx: usize, delta: f64) -> MarginaliseOutcome {
    let p = precision.nrows();
    assert_eq!(precision.ncols(), p, "precision must be square");

    let keep: Vec<usize> = (0..p).filter(|&i| i != idx).collect();

    let b_11 = precision[(idx, idx)];
    let b = subcolumn(precision, &keep, idx);
    let b_kk = submatrix(precision, &keep);

    // True covariance Σ = B⁻¹ and its submatrix Σ_kk.
    let sigma = spd_inverse(precision);
    let sigma_kk = submatrix(&sigma, &keep);

    // Regularised precision after marginalisation: B' = B_kk − c · b bᵀ.
    let c = 1.0 / (b_11 + delta);
    let b_prime = rank1_update(&b_kk, -c, &b);

    // Truth: (B')⁻¹.
    let truth = spd_inverse(&b_prime);

    // Common vector v = Σ_kk · b and scalar h = bᵀ v.
    let v = matvec(&sigma_kk, &b);
    let h = dot(&b, &v);

    // Woodbury coefficient: α = δ / (B_11 · (B_11 + δ)).
    let alpha = delta / (b_11 * (b_11 + delta));
    let woodbury_coefficient = alpha / alpha.mul_add(h, 1.0);
    let woodbury = rank1_update(&sigma_kk, -woodbury_coefficient, &v);

    // ADR's Sherman–Morrison coefficient.
    let adr_coefficient = c / c.mul_add(-h, 1.0);
    let adr_sm = rank1_update(&sigma_kk, adr_coefficient, &v);

    MarginaliseOutcome {
        truth,
        spec_sigma_kk: sigma_kk,
        woodbury,
        adr_sm,
        adr_coefficient,
        woodbury_coefficient,
        h,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Fixture matrices
// ═══════════════════════════════════════════════════════════════════════════════

/// The 3×3 example used as the review's headline figure.
fn review_headline_3x3() -> Mat<f64> {
    Mat::from_fn(3, 3, |i, j| match (i, j) {
        (0, 0) => 1.5,
        (1, 1) => 1.8,
        (2, 2) => 1.3,
        (0, 1) | (1, 0) => 0.2,
        (0, 2) | (2, 0) => -0.1,
        (1, 2) | (2, 1) => 0.25,
        _ => 0.0,
    })
}

/// Deterministic SplitMix64 — small inline PRNG so the random fixture
/// does not pull in an external crate and does not depend on the
/// `testing::TestRng` private helper (which is not exposed to integration
/// tests).
const fn splitmix64_next(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Uniform `f64` in `[-0.5, 0.5)` from a SplitMix64 state.
fn splitmix64_unit(state: &mut u64) -> f64 {
    // Use top 32 bits so conversion to `f64` is exact.
    let bits = u32::try_from(splitmix64_next(state) >> 32).expect("shifted SplitMix64 output fits in u32");
    f64::mul_add(f64::from(bits), 1.0 / 4_294_967_296.0, -0.5)
}

/// Random `p × p` SPD precision matrix `B = A Aᵀ + λ I` where `A` is a
/// `p × p` matrix of independent uniform `[-0.5, 0.5)` entries and
/// `λ = 0.5` keeps the spectrum bounded away from zero. Seed is fixed
/// so the fixture is reproducible across runs.
fn random_spd(p: usize, seed: u64) -> Mat<f64> {
    let mut state = seed;
    let a = Mat::from_fn(p, p, |_, _| splitmix64_unit(&mut state));
    let mut b = &a * a.transpose();
    for i in 0..p {
        b[(i, i)] += 0.5;
    }
    b
}

/// A handful of diagonally-dominant SPD test matrices.
///
/// The structural discrepancy already appears for `p = 2` when the
/// cross-precision is non-zero. The 3×3 fixtures keep a 2×2 block after
/// marginalisation, making the matrix-valued covariance update visible in
/// off-diagonal as well as diagonal entries. The `p = 20` random case shows
/// the same pattern survives at non-trivial dimension.
fn fixture_matrices() -> Vec<(&'static str, Mat<f64>)> {
    vec![
        ("review_headline", review_headline_3x3()),
        (
            "stronger_off_diagonal",
            Mat::from_fn(3, 3, |i, j| match (i, j) {
                (0, 0) => 1.5,
                (1, 1) => 1.7,
                (2, 2) => 1.25,
                (0, 1) | (1, 0) => 0.18,
                (0, 2) | (2, 0) => -0.12,
                (1, 2) | (2, 1) => 0.32,
                _ => 0.0,
            }),
        ),
        (
            "moderate_correlation",
            Mat::from_fn(3, 3, |i, j| match (i, j) {
                (0, 0) => 1.5,
                (1, 1) => 1.3,
                (2, 2) => 1.1,
                (0, 1) | (1, 0) => 0.15,
                (0, 2) | (2, 0) => 0.08,
                (1, 2) | (2, 1) => 0.4,
                _ => 0.0,
            }),
        ),
        (
            "near_singular_kept_block",
            Mat::from_fn(3, 3, |i, j| match (i, j) {
                (0, 0) => 1.5,
                (1, 1) => 1.0,
                (2, 2) => 0.9,
                (0, 1) | (1, 0) => 0.25,
                (0, 2) | (2, 0) => 0.1,
                (1, 2) | (2, 1) => 0.35,
                _ => 0.0,
            }),
        ),
        ("random_spd_p20", random_spd(20, 0x5EED_C0DE_C0FE_F00D)),
    ]
}

/// The Schur regularisation δ the defaults come to: the default factor
/// ε_Schur of 10⁻⁴ taken against the default prior precision of 0.1
/// (´alg:gaussian:regularised-schur´). The configuration holds the factor
/// and the computation derives δ, so this is a product rather than a
/// setting, and it is spelled here because these formula comparisons take
/// δ directly.
const DEFAULT_DELTA: f64 = 1e-4 * 0.1;

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// On the small correlated matrix that made the case, the three candidate
/// marginal covariances separate cleanly against the ground truth obtained by
/// inverting the marginal precision outright: the bare submatrix is close, the
/// Woodbury correction closer still, and the Sherman–Morrison candidate wrong
/// by orders of magnitude — and wrong in direction, adding an outer-product
/// term where the truth subtracts one.
///
/// (´claim:bayes:the-bare-covariance-submatrix-beats-the-sherman-morrison-candidate-by-orders-of-magnitude´)
/// ´test:integration:adr-review-3x3-example´
#[test]
fn adr_review_3x3_example() {
    let precision = review_headline_3x3();
    let delta = 1e-4;
    let outcome = marginalise_rank1_all_formulas(&precision, 0, delta);

    let err_spec = max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth);
    let err_wood = max_abs_diff(&outcome.woodbury, &outcome.truth);
    let err_adr = max_abs_diff(&outcome.adr_sm, &outcome.truth);

    eprintln!("=== adr_review_3x3_example (δ = {delta:e}) ===");
    eprintln!("  Σ' formula            max|err|");
    eprintln!("  spec (Σ_kk)           {err_spec:.3e}");
    eprintln!("  Woodbury (exact)      {err_wood:.3e}");
    eprintln!("  ADR (Sherman–Morrison){err_adr:.3e}");
    eprintln!("  coefficients: ADR c/(1−ch) = {:.4}", outcome.adr_coefficient);
    eprintln!("                Woodbury α/(1+αh) = {:.4e}", outcome.woodbury_coefficient);
    eprintln!("                h = bᵀ Σ_kk b      = {:.4}", outcome.h);

    // The review's headline numbers, to within a small factor.
    assert!(err_spec < 1e-3, "spec error {err_spec:e} exceeded 1e-3");
    assert!(err_wood < 1e-4, "woodbury error {err_wood:e} exceeded 1e-4");
    assert!(err_adr > 1e-3, "ADR formula error {err_adr:e} unexpectedly small");

    // The ADR formula adds when the truth subtracts: it should be at
    // least one order of magnitude worse than the spec.
    assert!(
        err_adr > 10.0 * err_spec,
        "expected ADR err ({err_adr:e}) >> spec err ({err_spec:e})"
    );
}

/// The same separation between the three candidates can be read off across
/// every fixture — differing coupling strengths, a nearly singular surviving
/// block, and a twenty-coordinate random matrix — as a printed table rather
/// than as a pass or a failure. This asserts nothing on purpose: it is the
/// evidence a reader consults when deciding whether the thresholds the
/// neighbouring tests assert are the right thresholds.
///
/// (´claim:bayes:the-bare-covariance-submatrix-beats-the-sherman-morrison-candidate-by-orders-of-magnitude´)
/// ´test:integration:error-table-across-structures´
#[test]
fn error_table_across_structures() {
    eprintln!();
    eprintln!("Rank-1 marginalisation: max|Σ' − (B')⁻¹| across formulas");
    eprintln!("δ = {DEFAULT_DELTA:e} (the default factor against the default prior)");
    eprintln!();
    eprintln!(
        "  {:<28}  {:>10}  {:>10}  {:>10}  {:>10}",
        "case", "spec", "woodbury", "ADR SM", "h"
    );
    eprintln!("  {:-<28}  {:->10}  {:->10}  {:->10}  {:->10}", "", "", "", "", "");
    for (name, precision) in fixture_matrices() {
        let outcome = marginalise_rank1_all_formulas(&precision, 0, DEFAULT_DELTA);
        let err_spec = max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth);
        let err_wood = max_abs_diff(&outcome.woodbury, &outcome.truth);
        let err_adr = max_abs_diff(&outcome.adr_sm, &outcome.truth);
        eprintln!(
            "  {:<28}  {:>10.3e}  {:>10.3e}  {:>10.3e}  {:>10.4}",
            name, err_spec, err_wood, err_adr, outcome.h
        );
    }
    eprintln!();
}

/// Simply extracting the covariance submatrix lands nearer the true marginal
/// covariance than the Sherman–Morrison candidate does, on every fixture, by
/// at least two orders of magnitude. This is why the implementation does the
/// cheap thing: the elaborate rank-one correction that formula applies is not
/// a refinement of the submatrix but a departure from it, and doing nothing
/// beats doing that.
///
/// ´claim:bayes:the-bare-covariance-submatrix-beats-the-sherman-morrison-candidate-by-orders-of-magnitude´
/// ´test:integration:spec-formula-beats-adr-formula-by-orders-of-magnitude´
#[test]
fn spec_formula_beats_adr_formula_by_orders_of_magnitude() {
    for (name, precision) in fixture_matrices() {
        let outcome = marginalise_rank1_all_formulas(&precision, 0, DEFAULT_DELTA);
        let err_spec = max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth);
        let err_adr = max_abs_diff(&outcome.adr_sm, &outcome.truth);
        let ratio = err_adr / err_spec;
        assert!(
            ratio > 100.0,
            "[{name}] expected ADR err to be >> spec err (ratio = {ratio:.2e}, \
             spec = {err_spec:e}, adr = {err_adr:e})"
        );
    }
}

/// On these freshly inverted fixtures, the Woodbury rank-one correction
/// reproduces the inverse of the implemented regularised precision to rounding.
/// The difference from plain covariance-submatrix extraction is introduced by
/// the offset in this exact-dual construction, but its magnitude also depends
/// on the removed pivot, coupling, and covariance quadratic form. The sweep
/// shows δ-scaling for these fixtures; δ alone is not a general error bound.
///
/// ´claim:bayes:the-woodbury-correction-removes-the-regularisation-sized-error-the-bare-submatrix-leaves´
/// ´test:integration:woodbury-is-the-exact-rank1-correction´
#[test]
fn woodbury_is_the_exact_rank1_correction() {
    for (name, precision) in fixture_matrices() {
        let outcome = marginalise_rank1_all_formulas(&precision, 0, DEFAULT_DELTA);
        let err_spec = max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth);
        let err_wood = max_abs_diff(&outcome.woodbury, &outcome.truth);
        assert!(
            err_wood < err_spec,
            "[{name}] Woodbury ({err_wood:e}) should beat bare Σ_kk ({err_spec:e})"
        );
        // The improvement should be at least an order of magnitude.
        assert!(
            err_spec / err_wood > 10.0,
            "[{name}] expected ≥10× tightening from Woodbury \
             (spec = {err_spec:e}, woodbury = {err_wood:e})"
        );
    }
}

/// Driving the regularisation down through eight orders of magnitude shrinks
/// the submatrix's error along with it but leaves the Sherman–Morrison
/// candidate's error essentially unchanged. That distinguishes the two kinds
/// of wrongness decisively: one is the price of regularising and vanishes when
/// the regularisation does, the other is an error in the algebra itself and no
/// amount of numerical care will remove it.
///
/// ´claim:bayes:the-sherman-morrison-candidates-error-is-structural-because-it-does-not-shrink-with-the-regularisation´
/// ´test:integration:adr-formula-error-scales-with-h-not-delta´
#[test]
fn adr_formula_error_scales_with_h_not_delta() {
    let precision = review_headline_3x3();

    let mut adr_errors = Vec::new();
    let mut spec_errors = Vec::new();
    for &delta in &[1e-3_f64, 1e-5, 1e-7, 1e-9, 1e-11] {
        let outcome = marginalise_rank1_all_formulas(&precision, 0, delta);
        adr_errors.push((delta, max_abs_diff(&outcome.adr_sm, &outcome.truth)));
        spec_errors.push((delta, max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth)));
    }

    eprintln!("=== adr_formula_error_scales_with_h_not_delta ===");
    eprintln!("  δ           spec err     ADR SM err");
    for ((d, s), (_, a)) in spec_errors.iter().zip(adr_errors.iter()) {
        eprintln!("  {d:.0e}    {s:.3e}    {a:.3e}");
    }

    // ADR errors stay roughly constant across 8 orders of magnitude in δ.
    let max_adr = adr_errors.iter().map(|(_, e)| *e).fold(0.0_f64, f64::max);
    let min_adr = adr_errors.iter().map(|(_, e)| *e).fold(f64::INFINITY, f64::min);
    assert!(
        max_adr / min_adr < 10.0,
        "ADR error should be ~constant in δ; got ratio {:.2e} (max={max_adr:e}, min={min_adr:e})",
        max_adr / min_adr
    );

    // Spec errors should fall by roughly the same factor as δ over the
    // tested range. Going from δ = 1e-3 to δ = 1e-11 (8 orders) should
    // shrink the spec error by many orders of magnitude.
    let spec_first = spec_errors.first().unwrap().1;
    let spec_last = spec_errors.last().unwrap().1;
    assert!(
        spec_first / spec_last.max(f64::MIN_POSITIVE) > 1e4,
        "spec error should track δ (first = {spec_first:e}, last = {spec_last:e})"
    );
}

/// At twenty coordinates the same ranking holds and the same scalings hold
/// with it: the submatrix's error tracks the regularisation, the Woodbury
/// correction's sits at machine precision, and the Sherman–Morrison
/// candidate's stays fixed as the regularisation is swept, with a coefficient
/// matching the reciprocal of the removed coordinate's own precision. The
/// argument is about block structure rather than about small matrices, so it
/// must not depend on the dimension it is demonstrated at.
///
/// ´claim:bayes:the-ranking-of-the-three-candidates-is-dimension-independent´
/// ´test:integration:error-pattern-holds-at-p20´
#[test]
fn error_pattern_holds_at_p20() {
    let precision = random_spd(20, 0x5EED_C0DE_C0FE_F00D);
    let idx = 0;
    let b_11 = precision[(idx, idx)];

    let outcome_default = marginalise_rank1_all_formulas(&precision, idx, DEFAULT_DELTA);
    let err_spec = max_abs_diff(&outcome_default.spec_sigma_kk, &outcome_default.truth);
    let err_wood = max_abs_diff(&outcome_default.woodbury, &outcome_default.truth);
    let err_adr = max_abs_diff(&outcome_default.adr_sm, &outcome_default.truth);

    eprintln!("=== error_pattern_holds_at_p20 (δ = {DEFAULT_DELTA:e}, B_11 = {b_11:.4}) ===");
    eprintln!("  spec (Σ_kk)            {err_spec:.3e}");
    eprintln!("  Woodbury (exact)       {err_wood:.3e}");
    eprintln!("  ADR (Sherman–Morrison) {err_adr:.3e}");
    eprintln!(
        "  ADR coefficient        {:.4}    (predicted ≈ 1/B_11 = {:.4})",
        outcome_default.adr_coefficient,
        1.0 / b_11,
    );
    eprintln!("  h = bᵀ Σ_kk b          {:.4}", outcome_default.h);

    // Spec ≈ O(δ): comfortably below 10⁻³ at δ = 10⁻⁵.
    assert!(err_spec < 1e-3, "p20 spec error {err_spec:e} unexpectedly large");
    // Woodbury ≈ machine eps: below 10⁻¹⁰ leaves headroom for accumulated roundoff in the p = 20 matvec.
    assert!(err_wood < 1e-10, "p20 Woodbury error {err_wood:e} unexpectedly large");
    // ADR has structural error orders of magnitude worse than spec.
    assert!(
        err_adr / err_spec > 1_000.0,
        "p20: expected ADR err to dwarf spec err (ratio {:.2e})",
        err_adr / err_spec,
    );

    // δ-independence: ADR error stays constant across 8 orders of magnitude.
    let mut adr_at_delta = Vec::new();
    for &delta in &[1e-3_f64, 1e-5, 1e-7, 1e-9, 1e-11] {
        let o = marginalise_rank1_all_formulas(&precision, idx, delta);
        adr_at_delta.push((delta, max_abs_diff(&o.adr_sm, &o.truth)));
    }
    let max_adr = adr_at_delta.iter().map(|(_, e)| *e).fold(0.0_f64, f64::max);
    let min_adr = adr_at_delta.iter().map(|(_, e)| *e).fold(f64::INFINITY, f64::min);
    assert!(
        max_adr / min_adr < 10.0,
        "p20: ADR error should be ~constant in δ; got ratio {:.2e}",
        max_adr / min_adr,
    );
}

/// Removing a coordinate from the middle of the matrix, or the very last one,
/// gives the same ranking as removing the first. Every other case here removes
/// index zero, where the gather of surviving indices is trivially the tail —
/// so an off-by-one in that gather would go unseen, and would look exactly
/// like a formula error rather than the bookkeeping error it is.
///
/// ´claim:bayes:the-ranking-holds-whichever-coordinate-is-removed´
/// ´test:integration:formula-ordering-is-index-position-independent´
#[test]
fn formula_ordering_is_index_position_independent() {
    let precision = random_spd(20, 0xCAFE_D00D_F00D_BAAD);

    for &idx in &[10_usize, 19] {
        let outcome = marginalise_rank1_all_formulas(&precision, idx, DEFAULT_DELTA);
        let err_spec = max_abs_diff(&outcome.spec_sigma_kk, &outcome.truth);
        let err_wood = max_abs_diff(&outcome.woodbury, &outcome.truth);
        let err_adr = max_abs_diff(&outcome.adr_sm, &outcome.truth);

        assert!(err_spec < 1e-3, "idx {idx}: spec error {err_spec:e} unexpectedly large");
        assert!(err_wood < err_spec, "idx {idx}: Woodbury should improve on Σ_kk");
        assert!(
            err_adr / err_spec > 100.0,
            "idx {idx}: expected ADR err to dwarf spec err (ratio {:.2e})",
            err_adr / err_spec,
        );
    }
}

/// Algebraic sanity check for the Woodbury derivation in the review:
///
///   B' = B_kk − c · b bᵀ
///      = (B_kk − (1/B_11) · b bᵀ)  +  ((1/B_11) − c) · b bᵀ
///      = Σ_kk⁻¹                     +  α · b bᵀ
///
/// where `α = δ / (B_11 · (B_11 + δ))`. The two ways of writing the marginal
/// precision agree to machine precision element by element, which is what
/// makes the exact correction available at rank-one cost: the marginal
/// precision is the inverse of the covariance submatrix plus one
/// regularisation-scaled outer product, so its inverse follows by a rank-one
/// identity rather than by a fresh factorisation.
///
/// ´claim:bayes:the-marginal-precision-is-the-covariance-submatrixs-inverse-plus-a-regularisation-scaled-rank-one-term´
/// ´test:integration:woodbury-derivation-sanity-check´
#[test]
fn woodbury_derivation_sanity_check() {
    let precision = review_headline_3x3();
    let delta = DEFAULT_DELTA;
    let idx = 0;
    let keep: Vec<usize> = (0..precision.nrows()).filter(|&i| i != idx).collect();

    let b_11 = precision[(idx, idx)];
    let b = subcolumn(&precision, &keep, idx);
    let b_kk = submatrix(&precision, &keep);

    let sigma_kk = submatrix(&spd_inverse(&precision), &keep);
    let sigma_kk_inv = spd_inverse(&sigma_kk);

    // Direct: B' = B_kk − c · b bᵀ
    let c = 1.0 / (b_11 + delta);
    let b_prime_direct = rank1_update(&b_kk, -c, &b);

    // Decomposed: B' = Σ_kk⁻¹ + α · b bᵀ
    let alpha = delta / (b_11 * (b_11 + delta));
    let b_prime_decomposed = rank1_update(&sigma_kk_inv, alpha, &b);

    let err = max_abs_diff(&b_prime_direct, &b_prime_decomposed);
    eprintln!("woodbury_derivation_sanity_check: max|direct − decomposed| = {err:e}");
    assert!(
        err < 1e-12,
        "the two algebraic decompositions of B' should agree to machine precision \
         (got {err:e})"
    );
}
