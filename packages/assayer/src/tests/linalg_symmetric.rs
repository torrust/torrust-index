// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`zeros_is_zero`] | linalg | A matrix asked for as zeros is exactly zero in every cell, off-diagonal and diagonal alike — compared with `==` rather than a tolerance, because there is no arithmetic involved for round-off to creep into. Anything a zero matrix is later accumulated into starts from nothing, with no residue of whatever the allocation happened to contain. |
//! | [`zeros_is_symmetric`] | linalg | The symmetry invariant holds from the moment of construction, not merely after the first operation restores it: a zero matrix already agrees with its own transpose bit for bit. Every later mutation is therefore re-establishing an invariant that was true on arrival, rather than gradually converging on one. |
//! | [`identity_scaled_values`] | linalg | A scaled identity puts the scale on every diagonal entry and exact zero everywhere off it. As a prior this says each dimension carries the same stated uncertainty and none of them are believed to covary — the starting position from which observation, not construction, is what introduces correlation between dimensions. |
//! | [`identity_scaled_symmetric`] | linalg | cites (´claim:linalg:a-freshly-constructed-matrix-is-already-bitwise-symmetric´) |
//! | [`from_computation_absorbs_rounding`] | linalg | Absorbing a computed matrix is forgiving by design: an entry perturbed by a single unit in the last place is not an error but the ordinary residue of floating-point arithmetic, and the lower triangle is simply mirrored over it, leaving the result bitwise symmetric. This is the entry point for results the crate produced itself — a Cholesky inverse, a Schur correction — where rejecting round-off would reject every real inverse. |
//! | [`from_storage_rejects_asymmetric`] | linalg | Loading from storage is the strict counterpart: a disagreement of 1e-10 between an entry and its transpose, far above round-off but far below anything a human would notice, is refused outright against a tolerance of 1e-12. Data that was written from a valid symmetric matrix cannot have drifted that far by arithmetic, so a gap of this size means the bytes are wrong, and the boundary is where that has to be caught. |
//! | [`from_storage_accepts_exact`] | linalg | Strictness at the storage boundary does not cost the honest case anything: an exactly symmetric matrix is admitted, and what emerges is bitwise symmetric rather than merely within tolerance. Passing the check and satisfying the invariant are the same event, so nothing downstream has to re-establish symmetry on a freshly loaded matrix. |
//! | [`rank1_update_changes_only_the_cells_the_vector_reaches`] | linalg | A rank-one update touches only where the vector is non-zero: adding the outer product of the first basis vector to an identity lifts the first diagonal entry to two and leaves the other four at one. Evidence about one dimension does not leak into dimensions the observation said nothing about. |
//! | [`rank1_update_matches_naive`] | linalg | Writing only the lower triangle and mirroring it costs nothing in accuracy: across a random ten-dimensional matrix the half-work update agrees cell for cell, to within 1e-14, with the textbook `M + w·vvᵀ` computed over the full square. The mirror tax is paid for with a copy, not with a different answer. |
//! | [`rank1_update_symmetry`] | linalg | Whatever an operation does to the contents, the result leaves the two triangles bit-identical — not close, identical — and free of NaN. Here a rank-one update on a random twenty-dimensional matrix, where the naive route would compute `w·v[i]·v[j]` and `w·v[j]·v[i]` separately and could land on different bit patterns. Because the audit is stated bitwise, an approximately symmetric result is a failure, and NaN cannot hide inside a matrix that passes. |
//! | [`rank1_update_spd`] | linalg | A positive-weight rank-one update cannot push a diagonal entry non-positive: adding `vvᵀ` to a positive definite matrix adds a non-negative `v[i]²` to each diagonal, so every one stays above zero. Precision only ever grows when evidence arrives, which is why the ordinary update path never needs regularising afterwards. |
//! | [`scale_and_rank1_positive_alpha`] | linalg | Fusing the scale into the rank-one update gives the same matrix as doing them one after the other: across fifteen dimensions every cell matches `s·M + α·vvᵀ` computed naively to within 1e-13. Forgetting and updating are one pass over the storage instead of two, and the saving is in traffic rather than in what the model ends up believing. |
//! | [`scale_and_rank1_negative_alpha_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`scale_and_rank1_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`scale_operation`] | linalg | Scaling multiplies the matrix through by the scalar: halving a twice-identity leaves ones on the diagonal. Forgetting is a uniform discount applied to everything the model believes, not a reweighting that favours some dimensions over others. |
//! | [`scale_zero`] | linalg | cites (´claim:linalg:scaling-multiplies-every-entry-through-by-the-scalar´) |
//! | [`clamp_diagonal_min_raises_floor`] | linalg | Clamping lifts every diagonal entry that sits below the floor up to exactly the floor. This is the replenishment floor: it stops a dimension's precision decaying towards zero under repeated forgetting, which would eventually make the matrix impossible to invert and the model impossible to update. |
//! | [`clamp_diagonal_min_no_change`] | linalg | A floor the diagonal already clears changes nothing at all — every cell is equal to what it was, not merely close to it. The floor is a one-sided safety net rather than a pull towards a preferred value, so a healthy matrix passing through it is untouched and repeated clamping is idempotent. |
//! | [`clamp_preserves_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`extend_dimensions`] | linalg | Extending grows the matrix without disturbing what it already held: the original occupies the leading block unchanged, the new dimensions get the requested value on their diagonal, and the blocks connecting old to new are zero in both directions. New dimensions therefore arrive believed independent of everything already known, and everything already known survives their arrival intact. |
//! | [`extend_zero_dimensions`] | linalg | Extending by nothing is a no-op that keeps the dimension where it was rather than an error or an off-by-one. A caller that discovers there are no new dimensions to add this round need not branch around the call. |
//! | [`extend_preserves_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`extract_symmetric_submatrix_correct`] | linalg | A symmetric submatrix gathers exactly the rows and columns named, keeping each cell's value and each dimension's position relative to the others: picking indices 0, 2 and 4 gives a three-dimensional matrix whose cells are the original's at those index pairs. Marginalisation is a restriction of what is known to a subset of dimensions, so a permutation or a misgathering here would silently reattribute one dimension's covariance to another. |
//! | [`extract_symmetric_submatrix_all`] | linalg | cites (´claim:linalg:a-symmetric-submatrix-gathers-exactly-the-named-rows-and-columns-preserving-their-values´) |
//! | [`extract_symmetric_submatrix_single`] | linalg | cites (´claim:linalg:a-symmetric-submatrix-gathers-exactly-the-named-rows-and-columns-preserving-their-values´) |
//! | [`extract_cross_block_correct`] | linalg | A cross-block gathers a rectangle — two chosen rows against three chosen columns — and comes back as a plain matrix of that shape rather than as a symmetric one. The coupling between a retained set of dimensions and a discarded set is genuinely not square, and pretending otherwise is what the separate return type prevents. |
//! | [`add_scaled_identity_correct`] | linalg | Adding a scaled identity lifts the diagonal by the given amount and leaves the off-diagonal alone, returning a new matrix rather than editing the original. This is the Schur regularisation nudge: it makes a block easier to invert by strengthening each dimension's own precision, without inventing correlations that the data never showed. |
//! | [`subtract_correct`] | linalg | Subtraction is element-wise and yields a new symmetric matrix: a twice-identity less an identity is an identity. Applying the Schur correction is a subtraction, so its result has to remain a matrix the rest of the layer can accept rather than a bare difference needing repair. |
//! | [`subtract_self_is_zero`] | linalg | cites (´claim:linalg:subtraction-is-element-wise-and-yields-another-symmetric-matrix´) |
//! | [`subtract_preserves_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`symmetric_matvec_identity`] | linalg | Multiplying by a scaled identity scales the vector and leaves its direction: against the identity itself, the vector comes back as it went in. The product reads both triangles rather than one, so a matrix that had somehow gone asymmetric would give a different answer here — the invariant is what makes this cheap general multiply the correct one to use. |
//! | [`symmetric_matvec_scaled`] | linalg | cites (´claim:linalg:multiplying-by-a-scaled-identity-scales-the-vector-and-keeps-its-direction´) |
//! | [`quadratic_form_identity`] | linalg | Against the identity the quadratic form is the squared norm: for [1, 2, 3] it is 14. The form is how the model turns a direction into a scalar amount of uncertainty, and this is the reference point that fixes its scale — no hidden factor of a half, no normalisation by dimension. |
//! | [`quadratic_form_from_slice_matches`] | linalg | The slice entry point exists to avoid allocating a column vector on a hot path, and it buys that saving without changing the answer: on a random ten-dimensional matrix the two agree to 1e-10 despite summing the same terms in a different order. The choice between them is a memory decision, not a numerical one. |
//! | [`quadratic_form_with_product_matches`] | linalg | The fused call returns both the product and the scalar, and each half equals what the separate calls would have produced — the vector bit-for-bit close to 1e-14, the scalar to 1e-10. Saving the second traversal is free: a caller that needs both is not trading accuracy for the single pass. |
//! | [`diagonal_element_correct`] | linalg | Asking for one diagonal entry returns that dimension's own stored value, in constant time and without touching the rest. A host wanting to know how certain the model is about a single dimension pays nothing for the other hundreds. |
//! | [`diagonal_iter_correct`] | linalg | Walking the diagonal yields one value per dimension and no more — the count matches the matrix's own dimension, and each value is the stored one. The iterator is a view over the diagonal rather than over the storage it lives in, so a caller reading it never has to skip the off-diagonal cells that lie between consecutive diagonal entries. |
//! | [`diagonal_min_max_correct`] | linalg | The diagonal extremes are found in a single pass and reported largest first, smallest second: across entries spanning five orders of magnitude the pair comes back as 100 and 0.001, that way round. Their ratio is the cheap standing estimate of how badly conditioned the matrix is, so the order they arrive in decides whether that estimate is read the right way up. |
//! | [`diagonal_min_max_identity`] | linalg | cites (´claim:linalg:the-diagonal-extremes-are-reported-largest-first-then-smallest´) |
//! | [`dim_correct`] | linalg | The reported dimension is the one the matrix was built with, whichever constructor built it. Since the matrix is square by construction there is a single number to report, and callers index against it rather than tracking a size of their own alongside the matrix. |
//! | [`as_inner_read_only`] | linalg | The inner matrix is reachable, at its full square shape and with its cells readable both on and off the diagonal — but only for reading. There is no mutable counterpart, which is what makes the symmetry invariant enforceable at all: every write goes through a named operation that mirrors and audits, and no caller can reach past that to poke a single cell. |
//! | [`debug_audit_passes_valid_mutations`] | linalg | The debug audit fires on no legitimate operation. Every mutating method, every derived constructor and both absorbing constructors are run in turn against a random matrix, and none of them trips the O(p²) symmetry and NaN check that follows each one. Since there is no mutable access to the inner matrix, corruption cannot be injected from outside — so the audit's silence here is the whole of what can be shown about it, and its converse is that any future panic points at a genuine defect in an operation rather than at a caller's misuse. |
//! | [`debug_audit_from_storage_rejects_corrupt`] | linalg | cites (´claim:linalg:loading-from-storage-refuses-triangles-that-disagree-beyond-tolerance´) |
//! | [`symmetric_operations_at_p638`] | linalg | The guarantees survive the dimension the system actually runs at, not just the handful of dimensions the other tests use: at p = 638 a rank-one update, a quadratic form, an extension to 706 and an extraction down to 551 all complete and all leave the matrix bitwise symmetric. Symmetry is maintained by an explicit mirroring loop, so it is precisely the kind of property that could hold at p = 5 and fail at scale through a blocked or parallel path that only engages on large matrices. |
//! | [`property_rank1_preserves_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`property_scale_preserves_symmetry`] | linalg | cites (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´) |
//! | [`property_extend_preserves_original`] | linalg | cites (´claim:linalg:extension-keeps-the-original-block-gives-new-dimensions-an-independent-prior-and-zeroes-the-cross-blocks´) |
//! | [`property_qf_non_negative_for_spd`] | linalg | The quadratic form never goes negative on a positive definite matrix, across a thousand random matrix-and-vector pairs. Downstream the form is read as an amount of uncertainty along a direction, and a negative reading would be meaningless there — worse, it would pass through unnoticed into whatever square root or division consumed it. |
//! | [`property_qf_equals_naive_vtmv`] | linalg | Computing the form as a matrix-vector product followed by a dot product gives the same number as summing `v[i]·M[i,j]·v[j]` over every pair — to within 1e-12 across a thousand random cases. The factored route exists for its cost, not for a different definition of the quantity. |
//! | [`property_qf_with_product_consistent`] | linalg | cites (´claim:linalg:the-fused-call-returns-the-same-product-and-scalar-as-the-two-separate-ones´) |

//! Crate-level tests for `linalg::symmetric::SymmetricMatrix`.
//!
//! All tests access `SymmetricMatrix` exclusively through its public API
//! (`as_inner()`, `dim()`, `diagonal_element()`, etc.).
//!
//! The invariant under test throughout is *bitwise* symmetry, not symmetry
//! to a tolerance: the lower triangle is authoritative and is mirrored into
//! the upper after every operation, so the two halves agree bit for bit and
//! the debug audit can use `to_bits()` equality — which also catches NaN,
//! since a NaN never equals itself.

#![allow(clippy::cast_precision_loss, clippy::suboptimal_flops)]

use faer::{Col, Mat};

use crate::linalg::symmetric::SymmetricMatrix;
use crate::testing::{TestRng, assert_finite};

// ─────────────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────────────

/// Verify every element of a `SymmetricMatrix` is bitwise symmetric.
fn assert_bitwise_symmetric(m: &SymmetricMatrix) {
    let p = m.dim();
    for j in 0..p {
        for i in j..p {
            let a = m.as_inner()[(i, j)];
            let b = m.as_inner()[(j, i)];
            assert!(
                a.to_bits() == b.to_bits(),
                "not bitwise symmetric: ({i},{j})={a} vs ({j},{i})={b}"
            );
        }
    }
}

/// Verify no NaN or Inf in a `SymmetricMatrix`.
///
/// Flattens the matrix (column-major) and delegates to the shared
/// [`assert_finite`](crate::testing::assert_finite) helper so failure
/// messages match the rest of the test suite.
fn assert_no_nan(m: &SymmetricMatrix) {
    let p = m.dim();
    let flat: Vec<f64> = (0..p).flat_map(|j| (0..p).map(move |i| m.as_inner()[(i, j)])).collect();
    assert_finite(&flat, "SymmetricMatrix entries");
}

/// Uniform `f64` in `[-1, 1)` drawn from a seeded [`TestRng`].
fn signed_unit(rng: &mut TestRng) -> f64 {
    rng.next_f64() * 2.0 - 1.0
}

/// Generate a random SPD matrix of given dimension via `AᵀA` + εI.
fn random_spd(p: usize, seed: u64) -> SymmetricMatrix {
    let mut rng = TestRng::new(seed);
    let r = Mat::from_fn(p, p, |_, _| signed_unit(&mut rng));
    let mut a = r.transpose() * &r;
    for i in 0..p {
        a[(i, i)] += 0.1;
    }
    SymmetricMatrix::from_computation(a)
}

/// Generate a random column vector.
fn random_vec(p: usize, seed: u64) -> Col<f64> {
    let mut rng = TestRng::new(seed);
    Col::from_fn(p, |_| signed_unit(&mut rng))
}

// ─────────────────────────────────────────────────────────────────────────────
// Construction tests
// ─────────────────────────────────────────────────────────────────────────────

/// A matrix asked for as zeros is exactly zero in every cell, off-diagonal
/// and diagonal alike — compared with `==` rather than a tolerance, because
/// there is no arithmetic involved for round-off to creep into. Anything a
/// zero matrix is later accumulated into starts from nothing, with no
/// residue of whatever the allocation happened to contain.
///
/// ´claim:linalg:a-zeros-matrix-is-exactly-zero-in-every-cell´
/// ´test:crate:zeros-is-zero´
#[test]
fn zeros_is_zero() {
    let m = SymmetricMatrix::zeros(5);
    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(m.as_inner()[(i, j)], 0.0);
        }
    }
}

/// The symmetry invariant holds from the moment of construction, not merely
/// after the first operation restores it: a zero matrix already agrees with
/// its own transpose bit for bit. Every later mutation is therefore
/// re-establishing an invariant that was true on arrival, rather than
/// gradually converging on one.
///
/// ´claim:linalg:a-freshly-constructed-matrix-is-already-bitwise-symmetric´
/// ´test:crate:zeros-is-symmetric´
#[test]
fn zeros_is_symmetric() {
    assert_bitwise_symmetric(&SymmetricMatrix::zeros(5));
}

/// A scaled identity puts the scale on every diagonal entry and exact zero
/// everywhere off it. As a prior this says each dimension carries the same
/// stated uncertainty and none of them are believed to covary — the starting
/// position from which observation, not construction, is what introduces
/// correlation between dimensions.
///
/// ´claim:linalg:a-scaled-identity-carries-the-scale-on-the-diagonal-and-zero-off-it´
/// ´test:crate:identity-scaled-values´
#[test]
fn identity_scaled_values() {
    let m = SymmetricMatrix::identity_scaled(5, 0.1);
    for j in 0..5 {
        for i in 0..5 {
            let expected = if i == j { 0.1 } else { 0.0 };
            assert_eq!(m.as_inner()[(i, j)], expected);
        }
    }
}

/// The same holds for a scaled identity, and at a hundred dimensions rather
/// than a handful: constructing by a per-cell rule leaves the two triangles
/// bit-identical without any mirroring pass being needed to fix it up.
///
/// (´claim:linalg:a-freshly-constructed-matrix-is-already-bitwise-symmetric´)
/// ´test:crate:identity-scaled-symmetric´
#[test]
fn identity_scaled_symmetric() {
    assert_bitwise_symmetric(&SymmetricMatrix::identity_scaled(100, 0.1));
}

/// Absorbing a computed matrix is forgiving by design: an entry perturbed by
/// a single unit in the last place is not an error but the ordinary residue
/// of floating-point arithmetic, and the lower triangle is simply mirrored
/// over it, leaving the result bitwise symmetric. This is the entry point for
/// results the crate produced itself — a Cholesky inverse, a Schur
/// correction — where rejecting round-off would reject every real inverse.
///
/// ´claim:linalg:absorbing-a-computed-matrix-mirrors-away-its-rounding-asymmetry´
/// ´test:crate:from-computation-absorbs-rounding´
#[test]
fn from_computation_absorbs_rounding() {
    let mut mat = Mat::from_fn(5, 5, |i, j| if i == j { 1.0 } else { 0.5 });
    // Introduce tiny rounding asymmetry
    mat[(0, 1)] += 1e-16;
    let m = SymmetricMatrix::from_computation(mat);
    assert_bitwise_symmetric(&m);
}

/// Loading from storage is the strict counterpart: a disagreement of 1e-10
/// between an entry and its transpose, far above round-off but far below
/// anything a human would notice, is refused outright against a tolerance of
/// 1e-12. Data that was written from a valid symmetric matrix cannot have
/// drifted that far by arithmetic, so a gap of this size means the bytes are
/// wrong, and the boundary is where that has to be caught.
///
/// ´claim:linalg:loading-from-storage-refuses-triangles-that-disagree-beyond-tolerance´
/// ´test:crate:from-storage-rejects-asymmetric´
#[test]
fn from_storage_rejects_asymmetric() {
    let mut mat = Mat::from_fn(5, 5, |i, j| if i == j { 1.0 } else { 0.5 });
    mat[(0, 1)] = 0.5 + 1e-10; // Significant asymmetry
    let result = SymmetricMatrix::from_storage(mat, 1e-12);
    assert!(result.is_err());
}

/// Strictness at the storage boundary does not cost the honest case anything:
/// an exactly symmetric matrix is admitted, and what emerges is bitwise
/// symmetric rather than merely within tolerance. Passing the check and
/// satisfying the invariant are the same event, so nothing downstream has to
/// re-establish symmetry on a freshly loaded matrix.
///
/// ´claim:linalg:an-exactly-symmetric-matrix-loads-and-emerges-bitwise-symmetric´
/// ´test:crate:from-storage-accepts-exact´
#[test]
fn from_storage_accepts_exact() {
    let mat = Mat::from_fn(5, 5, |i, j| if i == j { 1.0 } else { 0.5 });
    let result = SymmetricMatrix::from_storage(mat, 1e-12);
    assert!(result.is_ok());
    assert_bitwise_symmetric(&result.unwrap());
}

// ─────────────────────────────────────────────────────────────────────────────
// Mutation tests
// ─────────────────────────────────────────────────────────────────────────────

/// A rank-one update touches only where the vector is non-zero: adding the
/// outer product of the first basis vector to an identity lifts the first
/// diagonal entry to two and leaves the other four at one. Evidence about one
/// dimension does not leak into dimensions the observation said nothing
/// about.
///
/// ´claim:linalg:a-rank-one-update-changes-only-the-cells-the-vector-reaches´
/// ´test:crate:rank1-update-changes-only-the-cells-the-vector-reaches´
#[test]
fn rank1_update_changes_only_the_cells_the_vector_reaches() {
    let mut m = SymmetricMatrix::identity_scaled(5, 1.0);
    let e0 = Col::from_fn(5, |i| if i == 0 { 1.0 } else { 0.0 });
    m.symmetric_rank1_update(1.0, &e0);
    assert!((m.diagonal_element(0) - 2.0).abs() < 1e-15);
    for i in 1..5 {
        assert!((m.diagonal_element(i) - 1.0).abs() < 1e-15);
    }
    assert_bitwise_symmetric(&m);
}

/// Writing only the lower triangle and mirroring it costs nothing in
/// accuracy: across a random ten-dimensional matrix the half-work update
/// agrees cell for cell, to within 1e-14, with the textbook `M + w·vvᵀ`
/// computed over the full square. The mirror tax is paid for with a copy, not
/// with a different answer.
///
/// ´claim:linalg:the-half-triangle-rank-one-update-agrees-with-the-full-square-formula´
/// ´test:crate:rank1-update-matches-naive´
#[test]
fn rank1_update_matches_naive() {
    let p = 10;
    let m = random_spd(p, 42);
    let v = random_vec(p, 99);
    let w = 0.5;

    let mut expected = m.as_inner().clone();
    for i in 0..p {
        for j in 0..p {
            expected[(i, j)] += w * v[i] * v[j];
        }
    }

    let mut actual = m;
    actual.symmetric_rank1_update(w, &v);

    for i in 0..p {
        for j in 0..p {
            let diff = (actual.as_inner()[(i, j)] - expected[(i, j)]).abs();
            assert!(diff < 1e-14, "rank1 mismatch at ({i},{j}): diff = {diff:e}");
        }
    }
}

/// Whatever an operation does to the contents, the result leaves the two
/// triangles bit-identical — not close, identical — and free of NaN. Here a
/// rank-one update on a random twenty-dimensional matrix, where the naive
/// route would compute `w·v[i]·v[j]` and `w·v[j]·v[i]` separately and could
/// land on different bit patterns. Because the audit is stated bitwise, an
/// approximately symmetric result is a failure, and NaN cannot hide inside a
/// matrix that passes.
///
/// ´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´
/// ´test:crate:rank1-update-symmetry´
#[test]
fn rank1_update_symmetry() {
    let m = &mut random_spd(20, 1);
    let v = random_vec(20, 2);
    m.symmetric_rank1_update(0.5, &v);
    assert_bitwise_symmetric(m);
    assert_no_nan(m);
}

/// A positive-weight rank-one update cannot push a diagonal entry
/// non-positive: adding `vvᵀ` to a positive definite matrix adds a
/// non-negative `v[i]²` to each diagonal, so every one stays above zero.
/// Precision only ever grows when evidence arrives, which is why the ordinary
/// update path never needs regularising afterwards.
///
/// ´claim:linalg:a-positive-weight-rank-one-update-cannot-drive-a-diagonal-non-positive´
/// ´test:crate:rank1-update-spd´
#[test]
fn rank1_update_spd() {
    let mut m = random_spd(10, 55);
    let v = random_vec(10, 56);
    m.symmetric_rank1_update(1.0, &v);
    for i in 0..10 {
        assert!(
            m.diagonal_element(i) > 0.0,
            "diagonal[{i}] = {} not positive",
            m.diagonal_element(i)
        );
    }
    assert_bitwise_symmetric(&m);
}

/// Fusing the scale into the rank-one update gives the same matrix as doing
/// them one after the other: across fifteen dimensions every cell matches
/// `s·M + α·vvᵀ` computed naively to within 1e-13. Forgetting and updating
/// are one pass over the storage instead of two, and the saving is in traffic
/// rather than in what the model ends up believing.
///
/// ´claim:linalg:the-fused-scale-and-rank-one-update-equals-scaling-then-adding-the-outer-product´
/// ´test:crate:scale-and-rank1-positive-alpha´
#[test]
fn scale_and_rank1_positive_alpha() {
    let p = 15;
    let m_orig = random_spd(p, 30);
    let v = random_vec(p, 31);
    let s = 2.0;
    let alpha = 0.5;

    let mut expected = m_orig.as_inner().clone();
    for j in 0..p {
        for i in 0..p {
            expected[(i, j)] = s * expected[(i, j)] + alpha * v[i] * v[j];
        }
    }

    let mut actual = m_orig;
    actual.scale_and_symmetric_rank1_update(s, alpha, &v);

    for j in 0..p {
        for i in 0..p {
            let diff = (actual.as_inner()[(i, j)] - expected[(i, j)]).abs();
            assert!(diff < 1e-13, "scale_and_rank1 mismatch at ({i},{j}): diff = {diff:e}");
        }
    }
    assert_bitwise_symmetric(&actual);
}

/// A downdate — a negative coefficient, subtracting evidence rather than
/// adding it — is no exception to the invariant. Removing an observation's
/// contribution is where a matrix is most likely to go numerically astray,
/// and it is exactly there that the result still comes back bit-identical
/// across its diagonal.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:scale-and-rank1-negative-alpha-symmetry´
#[test]
fn scale_and_rank1_negative_alpha_symmetry() {
    let mut m = random_spd(15, 10);
    let v = random_vec(15, 20);
    m.scale_and_symmetric_rank1_update(1.0, -0.3, &v);
    assert_bitwise_symmetric(&m);
}

/// The general fused case — a forgetting factor below one alongside a
/// positive update coefficient, the shape an ordinary decaying update
/// actually takes — likewise leaves the matrix bitwise symmetric and with no
/// NaN anywhere in it.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:scale-and-rank1-symmetry´
#[test]
fn scale_and_rank1_symmetry() {
    let mut m = random_spd(15, 77);
    let v = random_vec(15, 78);
    m.scale_and_symmetric_rank1_update(0.8, 1.3, &v);
    assert_bitwise_symmetric(&m);
    assert_no_nan(&m);
}

/// Scaling multiplies the matrix through by the scalar: halving a
/// twice-identity leaves ones on the diagonal. Forgetting is a uniform
/// discount applied to everything the model believes, not a reweighting that
/// favours some dimensions over others.
///
/// ´claim:linalg:scaling-multiplies-every-entry-through-by-the-scalar´
/// ´test:crate:scale-operation´
#[test]
fn scale_operation() {
    let mut m = SymmetricMatrix::identity_scaled(5, 2.0);
    m.scale(0.5);
    for i in 0..5 {
        assert!((m.diagonal_element(i) - 1.0).abs() < 1e-15);
    }
}

/// Scaling by zero reaches the off-diagonal cells too, not just the diagonal
/// the loop writes: a random dense matrix scaled by zero is exactly zero
/// everywhere. Since only the lower triangle is written and then mirrored,
/// this is where an upper triangle left stale by a partial pass would show
/// up, and it does not.
///
/// (´claim:linalg:scaling-multiplies-every-entry-through-by-the-scalar´)
/// ´test:crate:scale-zero´
#[test]
fn scale_zero() {
    let mut m = random_spd(5, 7);
    m.scale(0.0);
    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(m.as_inner()[(i, j)], 0.0);
        }
    }
}

/// Clamping lifts every diagonal entry that sits below the floor up to
/// exactly the floor. This is the replenishment floor: it stops a dimension's
/// precision decaying towards zero under repeated forgetting, which would
/// eventually make the matrix impossible to invert and the model impossible
/// to update.
///
/// ´claim:linalg:clamping-lifts-every-diagonal-below-the-floor-up-to-it´
/// ´test:crate:clamp-diagonal-min-raises-floor´
#[test]
fn clamp_diagonal_min_raises_floor() {
    let mut m = SymmetricMatrix::identity_scaled(5, 0.1);
    m.clamp_diagonal_min(0.5);
    for i in 0..5 {
        assert!((m.diagonal_element(i) - 0.5).abs() < 1e-15);
    }
}

/// A floor the diagonal already clears changes nothing at all — every cell is
/// equal to what it was, not merely close to it. The floor is a one-sided
/// safety net rather than a pull towards a preferred value, so a healthy
/// matrix passing through it is untouched and repeated clamping is idempotent.
///
/// ´claim:linalg:a-floor-the-diagonal-already-clears-leaves-every-cell-untouched´
/// ´test:crate:clamp-diagonal-min-no-change´
#[test]
fn clamp_diagonal_min_no_change() {
    let mut m = SymmetricMatrix::identity_scaled(5, 0.1);
    let original = m.clone();
    m.clamp_diagonal_min(0.01);
    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(m.as_inner()[(i, j)], original.as_inner()[(i, j)]);
        }
    }
}

/// Clamping is the one mutation that skips the mirroring pass, on the grounds
/// that it writes only the diagonal — where a cell is its own transpose. A
/// floor high enough to raise every diagonal of a random matrix confirms the
/// shortcut is sound rather than merely cheap.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:clamp-preserves-symmetry´
#[test]
fn clamp_preserves_symmetry() {
    let mut m = random_spd(10, 5);
    m.clamp_diagonal_min(100.0);
    assert_bitwise_symmetric(&m);
}

// ─────────────────────────────────────────────────────────────────────────────
// Non-mutating construction tests
// ─────────────────────────────────────────────────────────────────────────────

/// Extending grows the matrix without disturbing what it already held: the
/// original occupies the leading block unchanged, the new dimensions get the
/// requested value on their diagonal, and the blocks connecting old to new
/// are zero in both directions. New dimensions therefore arrive believed
/// independent of everything already known, and everything already known
/// survives their arrival intact.
///
/// ´claim:linalg:extension-keeps-the-original-block-gives-new-dimensions-an-independent-prior-and-zeroes-the-cross-blocks´
/// ´test:crate:extend-dimensions´
#[test]
fn extend_dimensions() {
    let m = SymmetricMatrix::identity_scaled(5, 2.0);
    let ext = m.extend(3, 0.5);
    assert_eq!(ext.dim(), 8);

    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(ext.as_inner()[(i, j)], m.as_inner()[(i, j)]);
        }
    }
    for i in 5..8 {
        for j in 5..8 {
            let expected = if i == j { 0.5 } else { 0.0 };
            assert_eq!(ext.as_inner()[(i, j)], expected);
        }
    }
    for i in 0..5 {
        for j in 5..8 {
            assert_eq!(ext.as_inner()[(i, j)], 0.0);
            assert_eq!(ext.as_inner()[(j, i)], 0.0);
        }
    }
}

/// Extending by nothing is a no-op that keeps the dimension where it was
/// rather than an error or an off-by-one. A caller that discovers there are
/// no new dimensions to add this round need not branch around the call.
///
/// ´claim:linalg:extending-by-zero-dimensions-leaves-the-dimension-unchanged´
/// ´test:crate:extend-zero-dimensions´
#[test]
fn extend_zero_dimensions() {
    let m = SymmetricMatrix::identity_scaled(5, 1.0);
    let ext = m.extend(0, 0.5);
    assert_eq!(ext.dim(), 5);
}

/// A grown matrix is bitwise symmetric across the whole of its new extent,
/// not merely within the block it inherited. The extension is built cell by
/// cell from a rule rather than mirrored afterwards, so the rule itself has
/// to be symmetric in its two indices — and on a random matrix grown by half
/// again its size, it is.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:extend-preserves-symmetry´
#[test]
fn extend_preserves_symmetry() {
    let m = random_spd(10, 3);
    let ext = m.extend(5, 0.1);
    assert_bitwise_symmetric(&ext);
}

/// A symmetric submatrix gathers exactly the rows and columns named, keeping
/// each cell's value and each dimension's position relative to the others:
/// picking indices 0, 2 and 4 gives a three-dimensional matrix whose cells
/// are the original's at those index pairs. Marginalisation is a restriction
/// of what is known to a subset of dimensions, so a permutation or a
/// misgathering here would silently reattribute one dimension's covariance to
/// another.
///
/// ´claim:linalg:a-symmetric-submatrix-gathers-exactly-the-named-rows-and-columns-preserving-their-values´
/// ´test:crate:extract-symmetric-submatrix-correct´
#[test]
fn extract_symmetric_submatrix_correct() {
    let m = random_spd(5, 1);
    let indices = [0, 2, 4];
    let sub = m.extract_symmetric_submatrix(&indices);
    assert_eq!(sub.dim(), 3);
    for (si, &ri) in indices.iter().enumerate() {
        for (sj, &rj) in indices.iter().enumerate() {
            assert_eq!(sub.as_inner()[(si, sj)], m.as_inner()[(ri, rj)]);
        }
    }
    assert_bitwise_symmetric(&sub);
}

/// Naming every index in order reproduces the matrix exactly — extraction has
/// no edge behaviour that only shows up when nothing is being dropped, so
/// keeping everything costs a copy and nothing else.
///
/// (´claim:linalg:a-symmetric-submatrix-gathers-exactly-the-named-rows-and-columns-preserving-their-values´)
/// ´test:crate:extract-symmetric-submatrix-all´
#[test]
fn extract_symmetric_submatrix_all() {
    let m = random_spd(5, 2);
    let indices: Vec<usize> = (0..5).collect();
    let sub = m.extract_symmetric_submatrix(&indices);
    assert_eq!(sub.dim(), 5);
    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(sub.as_inner()[(i, j)], m.as_inner()[(i, j)]);
        }
    }
}

/// At the other extreme, keeping a single dimension gives a one-by-one matrix
/// holding that dimension's own diagonal entry. A model narrowed to one
/// surviving dimension is still a matrix, so callers need no special path for
/// the degenerate case.
///
/// (´claim:linalg:a-symmetric-submatrix-gathers-exactly-the-named-rows-and-columns-preserving-their-values´)
/// ´test:crate:extract-symmetric-submatrix-single´
#[test]
fn extract_symmetric_submatrix_single() {
    let m = random_spd(5, 3);
    let sub = m.extract_symmetric_submatrix(&[2]);
    assert_eq!(sub.dim(), 1);
    assert_eq!(sub.as_inner()[(0, 0)], m.as_inner()[(2, 2)]);
}

/// A cross-block gathers a rectangle — two chosen rows against three chosen
/// columns — and comes back as a plain matrix of that shape rather than as a
/// symmetric one. The coupling between a retained set of dimensions and a
/// discarded set is genuinely not square, and pretending otherwise is what
/// the separate return type prevents.
///
/// ´claim:linalg:a-cross-block-gathers-a-rectangle-of-chosen-rows-against-chosen-columns´
/// ´test:crate:extract-cross-block-correct´
#[test]
fn extract_cross_block_correct() {
    let m = random_spd(5, 3);
    let rows = [0, 1];
    let cols = [2, 3, 4];
    let block = m.extract_cross_block(&rows, &cols);
    assert_eq!(block.nrows(), 2);
    assert_eq!(block.ncols(), 3);
    for (bi, &ri) in rows.iter().enumerate() {
        for (bj, &cj) in cols.iter().enumerate() {
            assert_eq!(block[(bi, bj)], m.as_inner()[(ri, cj)]);
        }
    }
}

/// Adding a scaled identity lifts the diagonal by the given amount and leaves
/// the off-diagonal alone, returning a new matrix rather than editing the
/// original. This is the Schur regularisation nudge: it makes a block easier
/// to invert by strengthening each dimension's own precision, without
/// inventing correlations that the data never showed.
///
/// ´claim:linalg:adding-a-scaled-identity-lifts-only-the-diagonal´
/// ´test:crate:add-scaled-identity-correct´
#[test]
fn add_scaled_identity_correct() {
    let m = SymmetricMatrix::zeros(5);
    let result = m.add_scaled_identity(0.1);
    for i in 0..5 {
        assert!((result.diagonal_element(i) - 0.1).abs() < 1e-15);
    }
    assert_bitwise_symmetric(&result);
}

/// Subtraction is element-wise and yields a new symmetric matrix: a
/// twice-identity less an identity is an identity. Applying the Schur
/// correction is a subtraction, so its result has to remain a matrix the rest
/// of the layer can accept rather than a bare difference needing repair.
///
/// ´claim:linalg:subtraction-is-element-wise-and-yields-another-symmetric-matrix´
/// ´test:crate:subtract-correct´
#[test]
fn subtract_correct() {
    let m = SymmetricMatrix::identity_scaled(5, 2.0);
    let n = SymmetricMatrix::identity_scaled(5, 1.0);
    let diff = m.subtract(&n);
    for i in 0..5 {
        assert!((diff.diagonal_element(i) - 1.0).abs() < 1e-15);
    }
    assert_bitwise_symmetric(&diff);
}

/// A random matrix subtracted from itself is exactly zero in every cell, with
/// no residue anywhere — the difference of two identical floating-point
/// values is exact, and the operation adds no arithmetic of its own on top of
/// it. A correction that happens to cancel completely leaves nothing behind.
///
/// (´claim:linalg:subtraction-is-element-wise-and-yields-another-symmetric-matrix´)
/// ´test:crate:subtract-self-is-zero´
#[test]
fn subtract_self_is_zero() {
    let m = random_spd(5, 4);
    let zero = m.subtract(&m);
    for j in 0..5 {
        for i in 0..5 {
            assert_eq!(zero.as_inner()[(i, j)], 0.0);
        }
    }
}

/// Differencing two independently generated random matrices still leaves the
/// triangles bit-identical. Subtraction runs over the whole square and is
/// never mirrored afterwards, so it relies on both operands already
/// satisfying the invariant — which, coming out of the constructors, they do.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:subtract-preserves-symmetry´
#[test]
fn subtract_preserves_symmetry() {
    let a = random_spd(10, 80);
    let b = random_spd(10, 81);
    let diff = a.subtract(&b);
    assert_bitwise_symmetric(&diff);
}

// ─────────────────────────────────────────────────────────────────────────────
// Read method tests
// ─────────────────────────────────────────────────────────────────────────────

/// Multiplying by a scaled identity scales the vector and leaves its
/// direction: against the identity itself, the vector comes back as it went
/// in. The product reads both triangles rather than one, so a matrix that had
/// somehow gone asymmetric would give a different answer here — the invariant
/// is what makes this cheap general multiply the correct one to use.
///
/// ´claim:linalg:multiplying-by-a-scaled-identity-scales-the-vector-and-keeps-its-direction´
/// ´test:crate:symmetric-matvec-identity´
#[test]
fn symmetric_matvec_identity() {
    let m = SymmetricMatrix::identity_scaled(3, 1.0);
    let v = Col::from_fn(3, |i| (i + 1) as f64);
    let result = m.symmetric_matvec(&v);
    for i in 0..3 {
        assert!((result[i] - v[i]).abs() < 1e-15);
    }
}

/// With a scale of two the product is the vector doubled, entry for entry —
/// the scale passes through the multiplication rather than being applied once
/// somewhere along the way or absorbed into the diagonal traversal.
///
/// (´claim:linalg:multiplying-by-a-scaled-identity-scales-the-vector-and-keeps-its-direction´)
/// ´test:crate:symmetric-matvec-scaled´
#[test]
fn symmetric_matvec_scaled() {
    let m = SymmetricMatrix::identity_scaled(3, 2.0);
    let v = Col::from_fn(3, |i| (i + 1) as f64);
    let result = m.symmetric_matvec(&v);
    for i in 0..3 {
        assert!((result[i] - 2.0 * v[i]).abs() < 1e-15);
    }
}

/// Against the identity the quadratic form is the squared norm: for
/// [1, 2, 3] it is 14. The form is how the model turns a direction into a
/// scalar amount of uncertainty, and this is the reference point that fixes
/// its scale — no hidden factor of a half, no normalisation by dimension.
///
/// ´claim:linalg:the-quadratic-form-against-the-identity-is-the-squared-norm´
/// ´test:crate:quadratic-form-identity´
#[test]
fn quadratic_form_identity() {
    let m = SymmetricMatrix::identity_scaled(3, 1.0);
    let v = Col::from_fn(3, |i| (i + 1) as f64);
    let qf = m.quadratic_form(&v);
    assert!((qf - 14.0).abs() < 1e-14);
}

/// The slice entry point exists to avoid allocating a column vector on a hot
/// path, and it buys that saving without changing the answer: on a random
/// ten-dimensional matrix the two agree to 1e-10 despite summing the same
/// terms in a different order. The choice between them is a memory decision,
/// not a numerical one.
///
/// ´claim:linalg:the-allocation-free-slice-quadratic-form-agrees-with-the-column-vector-one´
/// ´test:crate:quadratic-form-from-slice-matches´
#[test]
fn quadratic_form_from_slice_matches() {
    let m = random_spd(10, 5);
    let v = random_vec(10, 6);
    let qf = m.quadratic_form(&v);
    let slice: Vec<f64> = (0..10).map(|i| v[i]).collect();
    let qf_slice = m.quadratic_form_from_slice(&slice);
    assert!((qf - qf_slice).abs() < 1e-10, "qf={qf} qf_slice={qf_slice}");
}

/// The fused call returns both the product and the scalar, and each half
/// equals what the separate calls would have produced — the vector
/// bit-for-bit close to 1e-14, the scalar to 1e-10. Saving the second
/// traversal is free: a caller that needs both is not trading accuracy for
/// the single pass.
///
/// ´claim:linalg:the-fused-call-returns-the-same-product-and-scalar-as-the-two-separate-ones´
/// ´test:crate:quadratic-form-with-product-matches´
#[test]
fn quadratic_form_with_product_matches() {
    let m = random_spd(10, 7);
    let v = random_vec(10, 8);
    let (mv, h) = m.quadratic_form_with_product(&v);
    let h2 = m.quadratic_form(&v);
    let mv2 = m.symmetric_matvec(&v);
    assert!((h - h2).abs() < 1e-10, "h mismatch: {h} vs {h2}");
    for i in 0..10 {
        assert!((mv[i] - mv2[i]).abs() < 1e-14, "matvec mismatch at {i}");
    }
}

/// Asking for one diagonal entry returns that dimension's own stored value,
/// in constant time and without touching the rest. A host wanting to know how
/// certain the model is about a single dimension pays nothing for the other
/// hundreds.
///
/// ´claim:linalg:the-diagonal-accessor-returns-that-dimensions-own-stored-value´
/// ´test:crate:diagonal-element-correct´
#[test]
fn diagonal_element_correct() {
    let m = SymmetricMatrix::identity_scaled(5, 3.0);
    for i in 0..5 {
        assert_eq!(m.diagonal_element(i), 3.0);
    }
}

/// Walking the diagonal yields one value per dimension and no more — the
/// count matches the matrix's own dimension, and each value is the stored
/// one. The iterator is a view over the diagonal rather than over the storage
/// it lives in, so a caller reading it never has to skip the off-diagonal
/// cells that lie between consecutive diagonal entries.
///
/// ´claim:linalg:the-diagonal-iterator-yields-one-stored-value-per-dimension´
/// ´test:crate:diagonal-iter-correct´
#[test]
fn diagonal_iter_correct() {
    let m = SymmetricMatrix::identity_scaled(5, 7.0);
    let diag: Vec<f64> = m.diagonal_iter().collect();
    assert_eq!(diag.len(), 5);
    for &d in &diag {
        assert_eq!(d, 7.0);
    }
}

/// The diagonal extremes are found in a single pass and reported largest
/// first, smallest second: across entries spanning five orders of magnitude
/// the pair comes back as 100 and 0.001, that way round. Their ratio is the
/// cheap standing estimate of how badly conditioned the matrix is, so the
/// order they arrive in decides whether that estimate is read the right way
/// up.
///
/// ´claim:linalg:the-diagonal-extremes-are-reported-largest-first-then-smallest´
/// ´test:crate:diagonal-min-max-correct´
#[test]
fn diagonal_min_max_correct() {
    let mut mat = Mat::zeros(3, 3);
    mat[(0, 0)] = 1.0;
    mat[(1, 1)] = 0.001;
    mat[(2, 2)] = 100.0;
    let m = SymmetricMatrix::from_computation(mat);
    let (max_val, min_val) = m.diagonal_min_max();
    assert!((max_val - 100.0).abs() < 1e-15);
    assert!((min_val - 0.001).abs() < 1e-15);
}

/// On a uniform diagonal the two extremes coincide, both landing on the
/// shared value rather than one of them being left at whatever the scan
/// started from. A perfectly conditioned matrix reads back as a ratio of one.
///
/// (´claim:linalg:the-diagonal-extremes-are-reported-largest-first-then-smallest´)
/// ´test:crate:diagonal-min-max-identity´
#[test]
fn diagonal_min_max_identity() {
    let m = SymmetricMatrix::identity_scaled(5, 1.0);
    let (max_val, min_val) = m.diagonal_min_max();
    assert!((max_val - 1.0).abs() < 1e-15);
    assert!((min_val - 1.0).abs() < 1e-15);
}

/// The reported dimension is the one the matrix was built with, whichever
/// constructor built it. Since the matrix is square by construction there is
/// a single number to report, and callers index against it rather than
/// tracking a size of their own alongside the matrix.
///
/// ´claim:linalg:the-reported-dimension-is-the-one-the-matrix-was-constructed-with´
/// ´test:crate:dim-correct´
#[test]
fn dim_correct() {
    assert_eq!(SymmetricMatrix::zeros(7).dim(), 7);
    assert_eq!(SymmetricMatrix::identity_scaled(13, 1.0).dim(), 13);
}

/// The inner matrix is reachable, at its full square shape and with its cells
/// readable both on and off the diagonal — but only for reading. There is no
/// mutable counterpart, which is what makes the symmetry invariant
/// enforceable at all: every write goes through a named operation that
/// mirrors and audits, and no caller can reach past that to poke a single
/// cell.
///
/// ´claim:linalg:the-only-route-to-the-inner-matrix-is-read-only´
/// ´test:crate:as-inner-read-only´
#[test]
fn as_inner_read_only() {
    let m = SymmetricMatrix::identity_scaled(3, 5.0);
    let inner = m.as_inner();
    assert_eq!(inner[(0, 0)], 5.0);
    assert_eq!(inner[(0, 1)], 0.0);
    assert_eq!(inner.nrows(), 3);
    assert_eq!(inner.ncols(), 3);
}

// ─────────────────────────────────────────────────────────────────────────────
// Debug audit tests
// ─────────────────────────────────────────────────────────────────────────────

/// The debug audit fires on no legitimate operation. Every mutating method,
/// every derived constructor and both absorbing constructors are run in turn
/// against a random matrix, and none of them trips the O(p²) symmetry and
/// NaN check that follows each one. Since there is no mutable access to the
/// inner matrix, corruption cannot be injected from outside — so the audit's
/// silence here is the whole of what can be shown about it, and its converse
/// is that any future panic points at a genuine defect in an operation rather
/// than at a caller's misuse.
///
/// ´claim:linalg:no-legitimate-operation-trips-the-debug-audit´
/// ´test:crate:debug-audit-passes-valid-mutations´
#[test]
fn debug_audit_passes_valid_mutations() {
    let mut m = random_spd(10, 90);
    let v = random_vec(10, 91);

    // Each of these invokes debug_audit internally.
    m.symmetric_rank1_update(1.0, &v);
    m.scale_and_symmetric_rank1_update(0.9, 0.5, &v);
    m.scale(0.7);
    m.clamp_diagonal_min(0.001);

    // Non-mutating constructors also invoke debug_audit.
    let _a = m.extend(3, 0.1);
    let _b = m.extract_symmetric_submatrix(&[0, 2, 4]);
    let _c = m.add_scaled_identity(0.01);
    let _d = m.subtract(&random_spd(10, 92));

    // from_computation and from_storage also audit.
    let _e = SymmetricMatrix::from_computation(Mat::zeros(3, 3));
    let _f = SymmetricMatrix::from_storage(Mat::zeros(3, 3), 1e-12);
}

/// The one place corruption can enter from outside is data read back from
/// disk, and that is the one place a runtime check guards rather than a
/// debug-only audit: an asymmetry of 1e-8 is rejected against a tolerance of
/// 1e-12. The rejection is a returned error in every build, so a corrupt
/// checkpoint does not become a release-mode matrix that silently violates
/// the invariant everything else assumes.
///
/// (´claim:linalg:loading-from-storage-refuses-triangles-that-disagree-beyond-tolerance´)
/// ´test:crate:debug-audit-from-storage-rejects-corrupt´
#[test]
fn debug_audit_from_storage_rejects_corrupt() {
    let mut mat = Mat::from_fn(5, 5, |i, j| if i == j { 1.0 } else { 0.5 });
    // Introduce asymmetry that exceeds tolerance
    mat[(0, 1)] = 0.5 + 1e-8;
    assert!(SymmetricMatrix::from_storage(mat, 1e-12).is_err());
}

// ─────────────────────────────────────────────────────────────────────────────
// Scaling test at reference configuration
// ─────────────────────────────────────────────────────────────────────────────

/// The guarantees survive the dimension the system actually runs at, not just
/// the handful of dimensions the other tests use: at p = 638 a rank-one
/// update, a quadratic form, an extension to 706 and an extraction down to
/// 551 all complete and all leave the matrix bitwise symmetric. Symmetry is
/// maintained by an explicit mirroring loop, so it is precisely the kind of
/// property that could hold at p = 5 and fail at scale through a blocked or
/// parallel path that only engages on large matrices.
///
/// ´claim:linalg:the-symmetry-guarantee-holds-at-the-reference-dimension-not-only-at-toy-sizes´
/// ´test:crate:symmetric-operations-at-p638´
#[test]
fn symmetric_operations_at_p638() {
    let p = 638;
    let m = SymmetricMatrix::identity_scaled(p, 1.0);
    assert_bitwise_symmetric(&m);

    let v = Col::from_fn(p, |i| (i as f64) / (p as f64));

    let mut m2 = m.clone();
    m2.symmetric_rank1_update(1.0, &v);
    assert_bitwise_symmetric(&m2);

    let _qf = m2.quadratic_form(&v);

    let m3 = m.extend(68, 0.1);
    assert_eq!(m3.dim(), p + 68);
    assert_bitwise_symmetric(&m3);

    let indices: Vec<usize> = (0..551).collect();
    let sub = m2.extract_symmetric_submatrix(&indices);
    assert_eq!(sub.dim(), 551);
    assert_bitwise_symmetric(&sub);
}

// ─────────────────────────────────────────────────────────────────────────────
// Property-based tests (1,000 random samples each, seeded PRNG)
// ─────────────────────────────────────────────────────────────────────────────

/// Symmetry after a rank-one update is not an accident of a chosen example: a
/// thousand seeded matrices updated with weights swept from −2 to +2, upweight
/// and downweight alike, all come back bitwise symmetric. The sign of the
/// weight is the interesting axis here, since a downdate is where cancellation
/// could produce two different bit patterns for a cell and its transpose.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:property-rank1-preserves-symmetry´
#[test]
fn property_rank1_preserves_symmetry() {
    for seed in 0..1_000u64 {
        let p = 8;
        let mut m = random_spd(p, seed);
        let v = random_vec(p, seed.wrapping_add(10_000));
        let w = ((seed % 200) as f64 - 100.0) / 50.0;
        m.symmetric_rank1_update(w, &v);
        assert_bitwise_symmetric(&m);
    }
}

/// Scaling likewise holds across a thousand random matrices and scales
/// sweeping from zero to two — through the annihilating case at one end and
/// growth at the other. Because only the lower triangle is multiplied and
/// then copied over, the copy is what has to be exact, and it is, at every
/// factor tried.
///
/// (´claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance´)
/// ´test:crate:property-scale-preserves-symmetry´
#[test]
fn property_scale_preserves_symmetry() {
    for seed in 0..1_000u64 {
        let p = 8;
        let mut m = random_spd(p, seed);
        let s = (seed % 1000) as f64 / 500.0;
        m.scale(s);
        assert_bitwise_symmetric(&m);
    }
}

/// What the original block keeps is its exact bits, across a thousand
/// matrices grown by between one and five dimensions with varying priors on
/// the new diagonal: every inherited cell compares equal under `to_bits()`,
/// not merely within a tolerance. Learning about a new dimension therefore
/// perturbs nothing already learned — the extension copies rather than
/// recomputes.
///
/// (´claim:linalg:extension-keeps-the-original-block-gives-new-dimensions-an-independent-prior-and-zeroes-the-cross-blocks´)
/// ´test:crate:property-extend-preserves-original´
#[test]
fn property_extend_preserves_original() {
    for seed in 0..1_000u64 {
        let p = 6;
        let r = (seed % 5) as usize + 1;
        let diag = (seed % 100) as f64 / 10.0;
        let m = random_spd(p, seed);
        let ext = m.extend(r, diag);
        for j in 0..p {
            for i in 0..p {
                assert!(
                    ext.as_inner()[(i, j)].to_bits() == m.as_inner()[(i, j)].to_bits(),
                    "extend changed ({i},{j}) at seed {seed}"
                );
            }
        }
    }
}

/// The quadratic form never goes negative on a positive definite matrix,
/// across a thousand random matrix-and-vector pairs. Downstream the form is
/// read as an amount of uncertainty along a direction, and a negative reading
/// would be meaningless there — worse, it would pass through unnoticed into
/// whatever square root or division consumed it.
///
/// ´claim:linalg:the-quadratic-form-stays-non-negative-on-a-positive-definite-matrix´
/// ´test:crate:property-qf-non-negative-for-spd´
#[test]
fn property_qf_non_negative_for_spd() {
    for seed in 0..1_000u64 {
        let p = 8;
        let m = random_spd(p, seed);
        let v = random_vec(p, seed.wrapping_add(5_000));
        let qf = m.quadratic_form(&v);
        assert!(qf >= 0.0, "QF negative ({qf}) at seed {seed}");
    }
}

/// Computing the form as a matrix-vector product followed by a dot product
/// gives the same number as summing `v[i]·M[i,j]·v[j]` over every pair — to
/// within 1e-12 across a thousand random cases. The factored route exists for
/// its cost, not for a different definition of the quantity.
///
/// ´claim:linalg:the-quadratic-form-equals-the-naive-double-sum-over-every-entry´
/// ´test:crate:property-qf-equals-naive-vtmv´
#[test]
fn property_qf_equals_naive_vtmv() {
    for seed in 0..1_000u64 {
        let p = 8;
        let m = random_spd(p, seed);
        let v = random_vec(p, seed.wrapping_add(3_000));
        let qf = m.quadratic_form(&v);
        let mut naive = 0.0;
        for i in 0..p {
            for j in 0..p {
                naive += v[i] * m.as_inner()[(i, j)] * v[j];
            }
        }
        let diff = (qf - naive).abs();
        assert!(diff < 1e-12, "QF mismatch: diff={diff:e} at seed {seed}");
    }
}

/// The agreement between the fused call and the separate ones holds across a
/// thousand random cases rather than on a single example, and at the same
/// tolerances: 1e-10 on the scalar, 1e-14 on each entry of the product. The
/// saving of one traversal is safe to take everywhere, not only where it
/// happens to have been measured.
///
/// (´claim:linalg:the-fused-call-returns-the-same-product-and-scalar-as-the-two-separate-ones´)
/// ´test:crate:property-qf-with-product-consistent´
#[test]
fn property_qf_with_product_consistent() {
    for seed in 0..1_000u64 {
        let p = 8;
        let m = random_spd(p, seed);
        let v = random_vec(p, seed.wrapping_add(7_000));
        let (mv, h) = m.quadratic_form_with_product(&v);
        let h2 = m.quadratic_form(&v);
        let mv2 = m.symmetric_matvec(&v);
        assert!((h - h2).abs() < 1e-10, "QF mismatch at seed {seed}: {h} vs {h2}");
        for i in 0..p {
            assert!((mv[i] - mv2[i]).abs() < 1e-14, "matvec mismatch at {i}, seed {seed}");
        }
    }
}
