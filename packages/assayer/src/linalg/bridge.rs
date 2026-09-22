// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Bridge between the Assayer and the `faer` linear algebra backend.
//!
//! All interaction with `faer` flows through this module. No other module
//! in the crate imports `faer` types directly (enforced by convention).
//!
//! # Cross-References
//!
//! - (´dec:substrate:backend-isolation´) — the isolation of the backend
//!   from every caller, which this module is
//! - (´dec:posterior:repair-cascade´) — the two-phase Cholesky
//!   recomputation cascade
//! - (´dec:posterior:half-solve´) — the Schur complement correction and
//!   its triangular solve

use std::fmt;

use faer::diag::Diag;
// Re-export the faer error type so callers do not need a direct faer import.
pub use faer::linalg::cholesky::llt::factor::LltError;
use faer::linalg::cholesky::llt::factor::{LltParams, LltRegularization};
use faer::linalg::evd::{ComputeEigenvectors, SelfAdjointEvdParams, self_adjoint_evd, self_adjoint_evd_scratch};
use faer::{Col, Mat, Par, Spec};

use super::symmetric::SymmetricMatrix;
use crate::types::ModelId;

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Error from the symmetric eigenvalue computation.
///
/// The backend's own error type is not re-exported here: unlike `LltError`,
/// which carries the offending pivot index and is therefore a diagnostic the
/// caller reads, this one carries nothing a caller could act on beyond the
/// fact of failure, so it crosses the boundary as the package's own type
/// (´dec:substrate:backend-isolation´).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EigenvalueError {
    /// The iterative eigensolver exhausted its iteration budget without
    /// converging, and no eigenvalue is available.
    NoConvergence,
}

impl fmt::Display for EigenvalueError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match *self {
            Self::NoConvergence => write!(f, "symmetric eigensolver did not converge"),
        }
    }
}

impl std::error::Error for EigenvalueError {}

/// Error from a plain (unregularised) Cholesky factorisation.
///
/// Wraps `faer`'s `LltError` with model context for diagnostics.
#[derive(Debug)]
pub struct CholeskyError {
    /// Index of the first non-positive pivot.
    pub pivot_index: usize,
    /// Which model triggered the failure.
    pub model: ModelId,
    /// Human-readable description of the call site.
    pub context: &'static str,
}

impl CholeskyError {
    /// Construct from `faer`'s `LltError` with model context.
    pub const fn from_llt_error(e: LltError, model: ModelId, context: &'static str) -> Self {
        let LltError::NonPositivePivot { index } = e;
        Self {
            pivot_index: index,
            model,
            context,
        }
    }
}

impl fmt::Display for CholeskyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Cholesky failed at pivot {}: {} (model {:?})",
            self.pivot_index, self.context, self.model
        )
    }
}

impl std::error::Error for CholeskyError {}

/// Call site for `cholesky_inverse`, used for diagnostic routing.
///
/// The utility is shared, so the caller says which site it is
/// (´dec:posterior:one-inversion-utility´).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CholeskyCallSite {
    /// Periodic $\Sigma \leftarrow B^{-1}$ recomputation.
    Periodic,
    /// Marginalisation: recompute $\Sigma' = (B')^{-1}$.
    Marginalisation,
    /// CP5/CP6 revert: reconstruct $B = \Sigma^{-1}$.
    Revert,
}

/// What one factorisation answered (´dec:posterior:repair-cascade´). The type
/// is the package's own rather than the backend's
/// (´dec:substrate:backend-isolation´).
///
/// Two variants and not three, because there is one attempt and not two. The
/// retired second phase shifted the matrix by `δI`, factored the shifted
/// matrix, and offered that matrix's inverse under a third variant the caller
/// had to interrogate to tell apart from an exact answer; beneath the shift
/// the backend's per-pivot backstop rewrote whatever pivot the shift did not
/// reach, so the offered inverse was of a third matrix again. A refusal is
/// unambiguous where such a success is not.
#[derive(Debug)]
pub enum CholeskyInverseResult {
    /// The factorisation succeeded, and the inverse is that matrix's own.
    Clean(SymmetricMatrix),
    /// The factorisation was refused, and nothing is offered in its place.
    Failed {
        /// The pivot index at which the factorisation was refused.
        pivot: usize,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Bridge Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// The width at or above which a dense factorisation or inverse is spread
/// across worker threads.
///
/// A measurement rather than a derivation. The sweep that produced it ships
/// beside it and can be run again: the factorisation and the inverse from its
/// factor, timed at both settings across the widths this package's matrices
/// reach, least of five repeats. The two operations do not cross over at the
/// same width — the inverse pays from about three hundred and fifty, the
/// factorisation not until about six hundred — and they run one after the
/// other on the recomputation path, so what decides is the pair. At three
/// hundred and eighty-four the pair was still faster sequential; at this width
/// it was faster spread, by about a quarter, and by a factor of two and a half
/// at a thousand.
///
/// It is the least measured width at which spreading pays, which is the
/// conservative end of the answer, and deliberately so: the sweep is taken on
/// an idle machine, and every reading it takes at the parallel setting gets
/// worse under contention while the sequential ones do not. The measurement is
/// what warrants the value
/// (´claim:linalg:the-parallelism-threshold-is-read-off-a-width-sweep-of-the-two-operations´).
///
/// ´const:assayer:linalg-parallel-width-threshold´ (´alg:const:count´)
/// ´const:assayer:linalg-parallel-width-threshold-count-512´
pub const PARALLEL_WIDTH_THRESHOLD: usize = 512;

/// The parallelism one operation on a `p`-wide matrix is given.
///
/// The backend's own default is a global read at every call, and what that
/// global holds is a worker pool sized to the whole machine. It is the wrong
/// quantity twice over. It is per process, so a deployment running several
/// instances gives each of them a pool sized as though it owned the host, and
/// the machine then spends its time moving work between workers rather than
/// doing it. And it is per machine rather than per matrix, so a factorisation
/// of a few tens of positions — which is what most of this package's matrices
/// are — pays a dispatch for work that a single thread finishes in less time
/// than the dispatch takes.
///
/// The width is what actually decides whether spreading pays, because the
/// work is cubic in it while the dispatch is not, so the decision is taken
/// here from the width and the global is never read. The threshold is a
/// measurement and is stated at
/// (´const:assayer:linalg-parallel-width-threshold´).
#[must_use]
pub fn parallelism_for(p: usize) -> Par {
    if p >= PARALLEL_WIDTH_THRESHOLD {
        Par::rayon(0)
    } else {
        Par::Seq
    }
}

/// The inverse of a matrix from its lower Cholesky factor.
///
/// Shared by both arms of the cascade so that the parallelism decision is
/// taken once and in one place, and so that neither arm reaches for the
/// backend's high-level wrapper, which would read the global.
fn inverse_from_lower_factor(l_factor: faer::MatRef<'_, f64>, p: usize) -> Mat<f64> {
    let par = parallelism_for(p);

    let scratch_req = faer::linalg::cholesky::llt::inverse::inverse_scratch::<f64>(p, par);
    let mut mem = faer::dyn_stack::MemBuffer::new(scratch_req);
    let stack = faer::dyn_stack::MemStack::new(&mut mem);

    let mut inv = Mat::zeros(p, p);
    faer::linalg::cholesky::llt::inverse::inverse(inv.as_mut(), l_factor, par, stack);
    // Make the result self-adjoint (copy lower to upper).
    make_self_adjoint(&mut inv);
    inv
}

/// The lower Cholesky factor $L$ of $A = LL^\top$.
///
/// The package's own type rather than the backend's solver wrapper
/// (´dec:substrate:backend-isolation´), and it is the package's own for a
/// reason beyond the convention: the wrapper reads the backend's global
/// parallelism when it is built and again at every solve, and reading a global
/// is precisely the decision this module now takes for itself
/// ([`parallelism_for`]). The wrapper's factor field is private and it offers
/// no constructor from a factor already computed, so a caller that wants to
/// say how the factorisation is spread cannot use it at all.
///
/// The entries above the diagonal are zero, as the wrapper's are, so a
/// consumer reading the whole matrix reads a lower triangular one.
#[derive(Clone, Debug)]
pub struct CholeskyFactor {
    /// The factor itself, lower triangular.
    l: Mat<f64>,
}

impl CholeskyFactor {
    /// The factor, for a caller solving against it.
    #[must_use]
    pub fn lower(&self) -> faer::MatRef<'_, f64> {
        self.l.as_ref()
    }

    /// The order of the matrix that was factored.
    #[must_use]
    pub fn dim(&self) -> usize {
        self.l.nrows()
    }
}

/// Plain Cholesky factorisation of a symmetric positive definite matrix.
///
/// Returns the factorisation or an error if a non-positive pivot is
/// encountered.
pub fn cholesky(mat: &SymmetricMatrix) -> Result<CholeskyFactor, LltError> {
    let p = mat.dim();
    let par = parallelism_for(p);

    let scratch_req =
        faer::linalg::cholesky::llt::factor::cholesky_in_place_scratch::<f64>(p, par, Spec::<LltParams, f64>::default());
    let mut mem = faer::dyn_stack::MemBuffer::new(scratch_req);
    let stack = faer::dyn_stack::MemStack::new(&mut mem);

    let mut work = mat.as_inner().clone();
    // No regularisation: a zero delta and a zero threshold turn the backend's
    // per-pivot device off, which is what makes this the plain phase.
    let regularization = LltRegularization {
        dynamic_regularization_delta: 0.0,
        dynamic_regularization_epsilon: 0.0,
    };
    faer::linalg::cholesky::llt::factor::cholesky_in_place(
        work.as_mut(),
        regularization,
        par,
        stack,
        Spec::<LltParams, f64>::default(),
    )?;

    // The in-place factorisation writes the lower triangle and leaves the
    // input's own entries above the diagonal, so they are cleared here: a
    // factor is lower triangular, and a consumer reading the whole matrix
    // should read one.
    for j in 0..p {
        for i in 0..j {
            work[(i, j)] = 0.0;
        }
    }

    Ok(CholeskyFactor { l: work })
}

/// Cholesky factorisation with model context for error reporting.
///
/// Wraps the raw Cholesky result in a `CholeskyError` that includes
/// the model identifier and operation context.
pub fn cholesky_with_context(
    mat: &SymmetricMatrix,
    model: ModelId,
    context: &'static str,
) -> Result<CholeskyFactor, CholeskyError> {
    cholesky(mat).map_err(|e| CholeskyError::from_llt_error(e, model, context))
}

/// What a matrix looked like when a factorisation refused it.
///
/// A failure that names only the pivot index cannot say whether the matrix
/// arrived corrupt or was made so by the factorisation, and those want
/// different repairs: an input carrying a non-finite entry is a fault upstream
/// of the numerics, while a finite well-scaled input that fails is a fault in
/// the factorisation's own arithmetic. The scan is $O(p^2)$ against the
/// $O(p^3)$ the refused factorisation has already spent, so it is taken on the
/// failing path only and costs nothing on any other.
#[derive(Clone, Copy, Debug)]
struct MatrixCondition {
    /// Smallest diagonal entry.
    diag_min: f64,
    /// Largest diagonal entry.
    diag_max: f64,
    /// Largest entry in absolute value, over the whole matrix.
    max_abs: f64,
    /// Entries that are not finite.
    non_finite: usize,
}

/// Reads a matrix's extremes for a failure diagnostic.
fn matrix_condition(mat: &SymmetricMatrix) -> MatrixCondition {
    let (diag_max, diag_min) = mat.diagonal_min_max();
    let inner = mat.as_inner();
    let p = mat.dim();
    let mut max_abs = 0.0_f64;
    let mut non_finite = 0;
    for j in 0..p {
        for i in j..p {
            let v = inner[(i, j)];
            if v.is_finite() {
                max_abs = max_abs.max(v.abs());
            } else {
                non_finite += 1;
            }
        }
    }
    MatrixCondition {
        diag_min,
        diag_max,
        max_abs,
        non_finite,
    }
}

/// One factorisation, and the inverse it produces or the refusal it reports.
///
/// Plain Cholesky. On success the inverse is built from that factor and
/// returned as `Clean`; on a non-positive pivot the refusal is returned as
/// `Failed`, and nothing is offered in its place
/// (´dec:posterior:repair-cascade´).
///
/// There is no second attempt. A shifted retry answers with the inverse of a
/// matrix the caller did not supply, which is a claim about which matrix was
/// inverted rather than a repair of the one that was; what bounds the least
/// eigenvalue is a declared floor written into the model at the rebuild
/// (´dec:posterior:spectral-floor´), where its effect on the posterior is
/// stated rather than attached to a result as a flag.
///
/// The refusal carries a diagnostic reading of the matrix, because a failure
/// naming only the pivot index cannot say whether the matrix arrived corrupt
/// or was made so by the arithmetic, and those want different repairs.
///
/// The `call_site` parameter is used for structured logging only; the
/// disposition of a refusal is the caller's, because it depends on where the
/// matrix came from and this utility cannot know that
/// (´dec:posterior:one-inversion-utility´). The types crossing this boundary
/// are the package's own (´dec:substrate:backend-isolation´).
pub fn cholesky_inverse(mat: &SymmetricMatrix, call_site: CholeskyCallSite) -> CholeskyInverseResult {
    match cholesky(mat) {
        Ok(factor) => {
            let inv = inverse_from_lower_factor(factor.lower(), mat.dim());
            CholeskyInverseResult::Clean(SymmetricMatrix::from_computation(inv))
        }
        Err(LltError::NonPositivePivot { index: pivot }) => {
            let condition = matrix_condition(mat);
            tracing::warn!(
                pivot,
                p = mat.dim(),
                diag_min = condition.diag_min,
                diag_max = condition.diag_max,
                max_abs = condition.max_abs,
                non_finite = condition.non_finite,
                ?call_site,
                "Cholesky refused the matrix; no inverse is offered",
            );
            CholeskyInverseResult::Failed { pivot }
        }
    }
}

/// The extreme eigenvalues of a symmetric matrix, as `(λ_max, λ_min)`.
///
/// The pair is ordered largest first, mirroring
/// [`SymmetricMatrix::diagonal_min_max`], so that a call site holding both
/// destructures them the same way. The ordering is the only thing the two
/// share and they are not interchangeable: the diagonal pair bounds the
/// spectrum without determining it, and only this one answers whether the
/// matrix is positive definite (´req:gaussian:positive-definiteness´).
///
/// The eigenvalues are real because the matrix is symmetric, which is the
/// invariant [`SymmetricMatrix`] carries; the backend is told to read the
/// lower triangle, the same half every other routine here reads.
///
/// The decomposition is dispatched through [`parallelism_for`] on the width,
/// like every other operation in this module. The backend's convenience
/// wrapper would have read its global instead, which is a pool sized to the
/// whole machine and therefore the wrong quantity here for the reasons that
/// function states; the low-level entry point takes the decision as an
/// argument, so the package's rule holds for the spectrum as it holds for the
/// factorisations. Eigenvectors are not requested: the callers read the two
/// ends of the spectrum and nothing else, and the reduction that produces the
/// eigenvalues is the whole of what they need paid for.
///
/// Cost: $O(p^3)$, the tridiagonal reduction dominating. This is a spectral
/// decomposition and not the $O(p)$ diagonal scan — a caller wanting a cheap
/// conditioning estimate rather than a verdict wants
/// [`SymmetricMatrix::diagonal_min_max`] instead
/// (´def:monitoring:condition-number´).
///
/// # Errors
///
/// Returns [`EigenvalueError::NoConvergence`] if the backend's iterative
/// eigensolver exhausts its iteration budget.
///
/// # Panics
///
/// Panics if the matrix is empty (p = 0), which has no spectrum to report.
pub fn eigenvalue_min_max(mat: &SymmetricMatrix) -> Result<(f64, f64), EigenvalueError> {
    assert!(mat.dim() > 0, "eigenvalue_min_max: matrix is empty");

    let p = mat.dim();
    let par = parallelism_for(p);

    let scratch_req =
        self_adjoint_evd_scratch::<f64>(p, ComputeEigenvectors::No, par, Spec::<SelfAdjointEvdParams, f64>::default());
    let mut mem = faer::dyn_stack::MemBuffer::new(scratch_req);
    let stack = faer::dyn_stack::MemStack::new(&mut mem);

    let mut spectrum = Diag::<f64>::zeros(p);
    self_adjoint_evd(
        mat.as_inner().as_ref(),
        spectrum.as_mut(),
        None,
        par,
        stack,
        Spec::<SelfAdjointEvdParams, f64>::default(),
    )
    .map_err(|_| EigenvalueError::NoConvergence)?;

    // The backend documents its eigenvalues as sorted nondecreasing. Folding
    // rather than reading the ends keeps the answer right without depending on
    // that promise, at $O(p)$ against the $O(p^3)$ already spent
    // (´dec:substrate:backend-isolation´).
    let mut lambda_max = f64::NEG_INFINITY;
    let mut lambda_min = f64::INFINITY;
    for &lambda in spectrum.column_vector().iter() {
        if lambda > lambda_max {
            lambda_max = lambda;
        }
        if lambda < lambda_min {
            lambda_min = lambda;
        }
    }

    Ok((lambda_max, lambda_min))
}

/// Extract a subvector at the given indices.
///
/// The marginal mean is the subvector (´thm:gaussian:marginalisation´).
pub fn extract_subvector(v: &Col<f64>, indices: &[usize]) -> Col<f64> {
    Col::from_fn(indices.len(), |i| v[indices[i]])
}

/// Half-solve: given the Cholesky factor L from `LL^T = A`, solve `L W = B`
/// via forward substitution only.
///
/// This is the primary path for the Schur complement correction, Option C
/// (´dec:posterior:half-solve´). The result W satisfies `L W = rhs`, and
/// `W^T W = rhs^T (LL^T)^{-1} rhs = rhs^T A^{-1} rhs`.
///
/// The Gram matrix `W^T W` is PSD by construction — structurally
/// superior to the full-solve path's `B_kr · Y`.
pub fn schur_half_solve(factor: &CholeskyFactor, rhs: &Mat<f64>) -> Mat<f64> {
    let l = factor.lower();
    let r = l.nrows();
    let k = rhs.ncols();
    let mut w = rhs.clone();
    faer::linalg::triangular_solve::solve_lower_triangular_in_place(l, w.as_mut(), parallelism_for(r));
    debug_assert_eq!(w.nrows(), r);
    debug_assert_eq!(w.ncols(), k);
    w
}

/// Full-solve: given the Cholesky factor `LL^T = A`, solve `A Y = B`
/// (i.e. `LL^T Y = B`).
///
/// This is Method B, retained as a debug-only oracle
/// (´dec:posterior:debug-oracle´). The correction is formed as
/// `B_kr · Y`, which is only approximately symmetric in floating point.
///
/// Used by the debug-mode oracle in `compute_schur_complement` to
/// cross-check the half-solve result, and by the equivalence test.
pub fn cholesky_full_solve(factor: &CholeskyFactor, rhs: &Mat<f64>) -> Mat<f64> {
    let l = factor.lower();
    let par = parallelism_for(l.nrows());
    let mut x = rhs.clone();
    // $LL^\top x = b$ in its two halves: forward substitution against $L$,
    // then back substitution against $L^\top$.
    faer::linalg::triangular_solve::solve_lower_triangular_in_place(l, x.as_mut(), par);
    faer::linalg::triangular_solve::solve_upper_triangular_in_place(l.transpose(), x.as_mut(), par);
    x
}

// ═══════════════════════════════════════════════════════════════════════════════
// Internal Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// The Wilkinson constant $\gamma_{n} = nu/(1-nu)$ for a unit roundoff $u =
/// \varepsilon/2$, which is the standard model's coefficient for $n$
/// successive rounded operations (Higham, *Accuracy and Stability of Numerical
/// Algorithms*, 2nd ed., chapter 3).
#[cfg(debug_assertions)]
#[allow(clippy::cast_precision_loss)] // Model dimensions are far below f64's exact integer range.
fn wilkinson_gamma(terms: usize) -> f64 {
    let unit_roundoff = f64::EPSILON / 2.0;
    let scaled = terms as f64 * unit_roundoff;
    debug_assert!(scaled < 1.0, "wilkinson_gamma: {terms} terms exhaust the format");
    scaled / (1.0 - scaled)
}

/// The distance below which two evaluations of one correction entry agree
/// (´dec:posterior:debug-oracle´).
///
/// # What the two routes compute, and what separates them
///
/// Both routes are built from one Cholesky factor $L$ of the regularised
/// removed block, and both approximate the same quantity $C = B_{kr}
/// (LL^\top)^{-1} B_{rk}$. The half-solve route solves $LW = B_{rk}$ by
/// forward substitution and forms the Gram matrix $W^\top W$; the full-solve
/// route solves $LL^\top Y = B_{rk}$ by two substitutions and forms $B_{kr}
/// Y$. Because the factor is shared, the factorisation's own error is common
/// to both and cancels out of their difference: the reference both are
/// measured against is the exact solve against the *computed* $L$, not against
/// the matrix $L$ was factored from. What separates the two evaluations is
/// therefore the error of the substitutions and of the products and nothing
/// else, which is what makes a bound on their difference derivable rather than
/// estimated.
///
/// # The substitution term, which is the conditioning
///
/// Substitution is componentwise backward stable: the computed $\hat{W}$
/// satisfies $(L + \Delta L)\hat{W} = B_{rk}$ with $|\Delta L| \le \gamma_{r}
/// |L|$, for $r$ the factor's order (Higham, Theorem 8.5). Rearranged,
/// $\hat{W} - W = -L^{-1} \Delta L \hat{W}$, so $|\hat{W} - W| \le \gamma_{r}
/// |L^{-1}||L||\hat{W}| =: S$ — an exact inequality rather than a first-order
/// one.
///
/// $|L^{-1}||L|$ is the factor's componentwise condition, and it is the term
/// the retired form omitted. It is also the quantity the replenishment floor
/// creates: the floor is where the smallest pivot of $L$ comes from, so a
/// bound that drops this factor drops exactly the geometry the engine spends
/// its time in.
///
/// No inverse is formed to evaluate it. For a triangular $T$, $|T^{-1}| \le
/// M(T)^{-1}$ entrywise, where $M(T)$ is the comparison matrix — $|t_{ii}|$ on
/// the diagonal and $-|t_{ij}|$ off it (Higham, chapter 8). Applying $M(L)^{-1}$ to
/// a non-negative matrix is one forward substitution whose every operation is
/// an addition or a division by a positive pivot, so the auxiliary solve
/// carries no cancellation of its own, and it costs $O(r^2 k)$ against the
/// $O(rk^2)$ the oracle's own product already pays.
///
/// # The product term
///
/// An $r$-term inner product carries a forward error of at most $\gamma_{r}$
/// times the sum of its terms' magnitudes, whatever the summation order
/// (Higham, chapter 3). Writing $H = |\hat{W}| + S$, the half-solve route's total
/// error at entry $(i,j)$ is at most
///
/// $\gamma_{r} (|\hat{W}|^\top|\hat{W}|)_{ij} + (H^\top H - |\hat{W}|^\top|\hat{W}|)_{ij}$,
///
/// the second term collecting $S^\top|\hat{W}| + |\hat{W}|^\top S + S^\top S$,
/// which is what bounds $|\hat{W}^\top\hat{W} - W^\top W|$. The full-solve
/// route is the same argument with two substitutions, giving $|\hat{Y} - Y|
/// \le (2\gamma_{r} + \gamma_{r}^2) |L^{-\top}||L^{-1}||L||L^\top||\hat{Y}| =: T$
/// with $|L^{-\top}||L^{-1}| \le M(L)^{-\top}M(L)^{-1}$, and the product
/// $B_{kr}\hat{Y}$ in place of the Gram:
///
/// $\gamma_{r} (|B_{kr}||\hat{Y}|)_{ij} + (|B_{kr}|(|\hat{Y}| + T) - |B_{kr}||\hat{Y}|)_{ij}$.
///
/// The tolerance is the sum of the two routes' bounds, which is what this
/// function assembles from the two per-entry sums its caller passes:
/// `magnitudes` is $(|\hat{W}|^\top|\hat{W}| + |B_{kr}||\hat{Y}|)_{ij}$, the
/// sums of term magnitudes the two products actually round, and `inflated` is
/// those same sums re-formed from operands raised by their own solve's error,
/// $(H^\top H + |B_{kr}|(|\hat{Y}| + T))_{ij}$.
///
/// # The accounting at the recorded failures
///
/// One unit below is $u|x|$, the unit of the standard model's relative error: for $|x|$ in the binade $[2^e, 2^{e+1})$ the unit in the last place is $2^{e-52}$ and $u|x|$ is $|x|/2^{e+1}$ of it, which is half an ulp exactly at a power of two and rises across the binade towards, without reaching, a whole one — $0.73$ ulp at the first row's magnitude. In those units the retired relative term was worth $2r$ and this bound is worth at least $6r$ — three times as much, attained only where the factor is perfectly conditioned and no term of either product cancels. The five failures recorded against the retired form stood at
///
/// | entry magnitude | $r$ | retired relative term | this bound at its weakest | observed difference |
/// | --- | --- | --- | --- | --- |
/// | 2.998e3 | 4 | 8 | 24 | 15.0 |
/// | 2.097e3 | 10 | 20 | 60 | 27.4 |
/// | 1.825e3 | 10 | 20 | 60 | 25.8 |
/// | 2.329e2 | 10 | 20 | 60 | 59.2 |
/// | 1.757e3 | 8 | 16 | 48 | 33.8 |
///
/// so every one is admitted even under the assumption that flatters the
/// retired form most, and the geometry that produced them — a factor whose
/// smallest pivot is the replenishment floor — is not that assumption. The
/// margin is 1.4 per cent on the fourth row under it and far wider at the
/// conditioning actually present.
///
/// # What the bound is worth at the floored geometry
///
/// A worst-case componentwise bound is not a tight one, and the falsifier measures how untight. At the geometry those tests build — a removed block whose near-collinear features leave every eigenvalue but the leading one at the replenishment floor, an eight-wide factor and two hundred kept dimensions — the two correct routes disagree at their worst entry by $7.84 \times 10^{-16}$ of that entry's own magnitude, which is three and a half ulp of it where that magnitude sits at the foot of its binade and about seven where it sits at the top, since one ulp is $2^{-52}$ of a magnitude at the foot and half that at the top, while this bound allows $1.733 \times 10^{-7}$ of it — a factor of $2.2 \times 10^8$ between what the arithmetic does and what the bound permits, and $9.8 \times 10^7$ times the retired form's relative term $2\gamma_{8}$. That is the comparison matrix being the worst case over every factor with those magnitudes rather than the realisation, which is a known pessimism of the bound on an ill-conditioned triangular factor and not an error in it. Its consequence is worth stating plainly: at the floor this oracle audits nothing finer than one part in ten million.
///
/// What that price leaves is still an oracle. The smallest relative divergence
/// refused at that entry is $1.73 \times 10^{-7}$, and the three faults the
/// falsifier drives — a solve against a block regularised a hundred times more
/// heavily, a coupling gathered one column out, a correction assembled as
/// $B_{kr}W$ rather than $W^\top W$ — move their worst entry by 0.81 to 0.99 of
/// its own magnitude, which is $2.3 \times 10^7$ to $4.9 \times 10^7$ times
/// this bound. What the floor's conditioning costs is sensitivity to
/// divergences below a part in ten million; what an oracle exists to report is
/// parts in one.
///
/// # The two retired forms
///
/// The first was a fixed $10^{-12}$, which falls below a single representable
/// step once $|x| \gtrsim 4.5 \times 10^3$ and is therefore a bound no correct
/// arithmetic can meet there; a correction near $5.09 \times 10^3$ whose two
/// forms stood two ulp apart tripped it. The second, $2\gamma_{r}|x| +
/// 10^{-12}$, was short in two independent ways: it dropped the substitution's
/// conditioning entirely, and it put the computed answer $|x|$ where the sum
/// of the terms' magnitudes belongs, which understates that sum by however
/// much the entry cancelled. Stating the bound against the operands repairs
/// both at once, and retires the fixed term with them — an absolute floor was
/// needed only because a bound written against the answer shrinks to nothing
/// when the answer does, and a bound written against the operands does not.
///
/// # The bound is deliberately looser than a last-bit disagreement
///
/// An oracle tighter than the error of the arithmetic it audits reports the
/// arithmetic rather than the algorithm, and the algorithmic divergence it
/// exists to catch — a transposed block, a solve against the wrong factor, the
/// wrong operand — moves an entry by a share of its own magnitude rather than
/// by its last bits. The conditioning term widens the budget by the factor the
/// floor puts into $L$, which stays orders below the share of its own
/// magnitude a faulted route moves an entry by.
#[cfg(debug_assertions)]
#[must_use]
pub fn schur_oracle_tolerance(magnitudes: f64, inflated: f64, terms: usize) -> f64 {
    // The two sums are separate reductions of non-negative quantities and
    // `inflated` dominates `magnitudes` term by term, so their difference is
    // non-negative in exact arithmetic and can only round below zero where the
    // solve contributed nothing at all.
    let inflation = (inflated - magnitudes).max(0.0);
    wilkinson_gamma(terms).mul_add(magnitudes, inflation)
}

/// The entrywise absolute value of a matrix.
#[cfg(debug_assertions)]
fn abs_entries(matrix: &Mat<f64>) -> Mat<f64> {
    Mat::from_fn(matrix.nrows(), matrix.ncols(), |row, col| matrix[(row, col)].abs())
}

/// $|L| X$ for a lower-triangular $L$ and a non-negative $X$.
#[cfg(debug_assertions)]
fn abs_lower_times(lower: faer::MatRef<'_, f64>, rhs: &Mat<f64>) -> Mat<f64> {
    Mat::from_fn(lower.nrows(), rhs.ncols(), |row, col| {
        let mut acc = 0.0;
        for term in 0..=row {
            acc = lower[(row, term)].abs().mul_add(rhs[(term, col)], acc);
        }
        acc
    })
}

/// $|L^\top| X$ for a lower-triangular $L$ and a non-negative $X$.
#[cfg(debug_assertions)]
fn abs_upper_times(lower: faer::MatRef<'_, f64>, rhs: &Mat<f64>) -> Mat<f64> {
    let order = lower.nrows();
    Mat::from_fn(order, rhs.ncols(), |row, col| {
        let mut acc = 0.0;
        for term in row..order {
            acc = lower[(term, row)].abs().mul_add(rhs[(term, col)], acc);
        }
        acc
    })
}

/// $M(L)^{-1} V$ for the comparison matrix of a lower-triangular $L$ and a
/// non-negative $V$, by forward substitution.
///
/// Every operation is an addition or a division by a positive pivot, so the
/// result is non-negative and the auxiliary solve carries no cancellation of
/// its own.
#[cfg(debug_assertions)]
fn comparison_forward_solve(lower: faer::MatRef<'_, f64>, rhs: &Mat<f64>) -> Mat<f64> {
    let order = lower.nrows();
    let columns = rhs.ncols();
    let mut solution: Mat<f64> = Mat::zeros(order, columns);
    for col in 0..columns {
        for row in 0..order {
            let mut acc = rhs[(row, col)];
            for term in 0..row {
                acc = lower[(row, term)].abs().mul_add(solution[(term, col)], acc);
            }
            solution[(row, col)] = acc / lower[(row, row)].abs();
        }
    }
    solution
}

/// $M(L)^{-\top} V$ for the comparison matrix of a lower-triangular $L$ and a
/// non-negative $V$, by back substitution.
#[cfg(debug_assertions)]
fn comparison_back_solve(lower: faer::MatRef<'_, f64>, rhs: &Mat<f64>) -> Mat<f64> {
    let order = lower.nrows();
    let columns = rhs.ncols();
    let mut solution: Mat<f64> = Mat::zeros(order, columns);
    for col in 0..columns {
        for row in (0..order).rev() {
            let mut acc = rhs[(row, col)];
            for term in row + 1..order {
                acc = lower[(term, row)].abs().mul_add(solution[(term, col)], acc);
            }
            solution[(row, col)] = acc / lower[(row, row)].abs();
        }
    }
    solution
}

/// The two per-entry sums [`schur_oracle_tolerance`] is assembled from: the
/// term magnitudes both products round, and those same sums re-formed from
/// operands raised by their own solve's componentwise forward error.
///
/// Both are $k \times k$ and non-negative, and the second dominates the first
/// entry by entry. The solves against the comparison matrix cost $O(r^2 k)$
/// and the four products $O(rk^2)$, which is the order the oracle's own
/// full-solve product already pays.
///
/// The added order is therefore not what the added cost is. Measured at the
/// reproduction's own dimensions, $r = 4$ and $k = 100$, the oracle went from
/// 119 to 1374 microseconds a call, a factor of 11.5, and the factor is
/// constants rather than orders: the four products are four more of the shape
/// the oracle already formed once, so the product work is five times what it
/// was ($1.6 \times 10^5$ multiply-adds against $4 \times 10^4$); two $k
/// \times k$ accumulators are allocated where none were, which is $2 \times
/// 10^4$ entries written and read again; six $r \times k$ and $k \times r$
/// temporaries hold the absolute values and the inflated operands; and the
/// assertion loop reads two matrices per entry where it read none. The
/// comparison-matrix solves themselves are $1.6 \times 10^3$ operations each
/// at these dimensions, which is the part that does not show.
///
/// What that costs a run is small and bounded. The reproductions drive 3400
/// labels in 402 to 413 seconds, which is 118 milliseconds a label against
/// 1.4 milliseconds a call, and the oracle fires once per marginalisation
/// rather than once per label — the dense schedule performs none at all and
/// pays nothing. And it is absent from a release binary, which is what
/// (´dec:posterior:debug-oracle´) says it costs there.
#[cfg(debug_assertions)]
#[must_use]
pub fn schur_oracle_sums(
    factor: &CholeskyFactor,
    b_rk: &Mat<f64>,
    b_kr: &Mat<f64>,
    full_solve_y: &Mat<f64>,
) -> (Mat<f64>, Mat<f64>) {
    let lower = factor.lower();
    let kept = b_kr.nrows();
    let gamma = wilkinson_gamma(factor.dim());

    // The half-solve route. Recomputing W costs one more substitution and
    // keeps the oracle's signature the caller's, rather than asking the call
    // site to hand over an intermediate it has no other reason to keep.
    let half = abs_entries(&schur_half_solve(factor, b_rk));
    let half_error = comparison_forward_solve(lower, &abs_lower_times(lower, &half));
    let half_inflated = Mat::from_fn(half.nrows(), half.ncols(), |row, col| {
        gamma.mul_add(half_error[(row, col)], half[(row, col)])
    });

    // The full-solve route: two substitutions, so the backward perturbation is
    // $(2\gamma_{r} + \gamma_{r}^2)|L||L^\top|$ and the inverse it passes through
    // is bounded by the comparison matrix twice.
    let full = abs_entries(full_solve_y);
    let backward = abs_lower_times(lower, &abs_upper_times(lower, &full));
    let full_error = comparison_back_solve(lower, &comparison_forward_solve(lower, &backward));
    let two_solves = gamma * (2.0 + gamma);
    let full_inflated = Mat::from_fn(full.nrows(), full.ncols(), |row, col| {
        two_solves.mul_add(full_error[(row, col)], full[(row, col)])
    });

    let coupling = abs_entries(b_kr);
    let par = parallelism_for(kept);
    let mut magnitudes = Mat::zeros(kept, kept);
    faer::linalg::matmul::matmul(
        magnitudes.as_mut(),
        faer::Accum::Replace,
        half.transpose(),
        half.as_ref(),
        1.0,
        par,
    );
    faer::linalg::matmul::matmul(
        magnitudes.as_mut(),
        faer::Accum::Add,
        coupling.as_ref(),
        full.as_ref(),
        1.0,
        par,
    );

    let mut inflated = Mat::zeros(kept, kept);
    faer::linalg::matmul::matmul(
        inflated.as_mut(),
        faer::Accum::Replace,
        half_inflated.transpose(),
        half_inflated.as_ref(),
        1.0,
        par,
    );
    faer::linalg::matmul::matmul(
        inflated.as_mut(),
        faer::Accum::Add,
        coupling.as_ref(),
        full_inflated.as_ref(),
        1.0,
        par,
    );

    (magnitudes, inflated)
}

/// Cross-check the half-solve correction ($W^\top W$) against the full-solve
/// correction ($B_{kr} \cdot Y$) in debug builds.
///
/// Computes the Method B correction and asserts per-element agreement within
/// [`schur_oracle_tolerance`], whose two operand sums come from
/// [`schur_oracle_sums`]. Mirrors the SVD oracle in sentinel's maths module.
///
/// Gated by the `debug_assertions` attribute rather than the `cfg!`
/// expression it used, so that the record's cost claim is true of the
/// code: the body, the full solve it calls and the assertion it ends in
/// are absent from a release binary (´dec:posterior:debug-oracle´). The
/// tracing disjunction went with the expression — the oracle ends in a
/// panic, and a `DEBUG`-level subscriber is a supported operational
/// action, so keeping it would have made raising the log level a way to
/// abort the process on a deregistration
/// (´dec:degradation:retain-and-flag´).
///
/// # Panics
///
/// Panics (via `assert!`) if the two methods diverge beyond tolerance.
#[cfg(debug_assertions)]
pub fn schur_correction_oracle(half_solve_correction: &Mat<f64>, factor: &CholeskyFactor, b_rk: &Mat<f64>, b_kr: &Mat<f64>) {
    let full_solve_y = cholesky_full_solve(factor, b_rk);
    let mut full_solve_correction = Mat::zeros(b_kr.nrows(), full_solve_y.ncols());
    faer::linalg::matmul::matmul(
        full_solve_correction.as_mut(),
        faer::Accum::Replace,
        b_kr.as_ref(),
        full_solve_y.as_ref(),
        1.0,
        parallelism_for(factor.dim()),
    );

    let (magnitudes, inflated) = schur_oracle_sums(factor, b_rk, b_kr, &full_solve_y);

    // Both forms sum one product per retained position, which is the factor's
    // own order, and that order is also the substitutions' own.
    let terms = factor.dim();
    let rows = half_solve_correction.nrows();
    let cols = half_solve_correction.ncols();
    for row in 0..rows {
        for col in 0..cols {
            let half = half_solve_correction[(row, col)];
            let full = full_solve_correction[(row, col)];
            let diff = (half - full).abs();
            let tolerance = schur_oracle_tolerance(magnitudes[(row, col)], inflated[(row, col)], terms);
            assert!(
                diff <= tolerance,
                "Schur oracle: correction[{row},{col}] half-solve={half:.15e} \
                 full-solve={full:.15e} diff={diff:.2e} tolerance={tolerance:.2e}"
            );
        }
    }
}

/// The release build's oracle: nothing, which is what
/// (´dec:posterior:debug-oracle´) says it costs there.
#[cfg(not(debug_assertions))]
pub const fn schur_correction_oracle(_correction: &Mat<f64>, _factor: &CholeskyFactor, _b_rk: &Mat<f64>, _b_kr: &Mat<f64>) {}

/// Copy lower triangle to upper to make a matrix self-adjoint.
fn make_self_adjoint(mat: &mut Mat<f64>) {
    let n = mat.nrows();
    for j in 0..n {
        for i in 0..j {
            let val = mat[(j, i)];
            mat[(i, j)] = val;
        }
    }
}
