// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Linear algebra building blocks for the subspace tracker.
//!
//! This module isolates the SVD-based subspace evolution from the
//! rest of the tracker so that:
//!
//! 1. The algorithm can be swapped at runtime between the naïve
//!    dense thin SVD and Brand's incremental SVD (ADR-S-016).
//! 2. In debug builds (or with a `DEBUG`-level tracing subscriber),
//!    **both** algorithms run and their outputs are compared via
//!    `assert!` — a continuous oracle test.
//! 3. The maths is independently unit-testable against known
//!    matrix identities.
//!
//! # §-references
//!
//! - §ALGO S-4.2 Phase 2 — Subspace evolution
//! - ADR-S-016 — Brand's incremental SVD

pub mod brand_svd;
pub mod naive_svd;

#[cfg(test)]
mod tests;

use std::cell::Cell;

use faer::Mat;

// Re-export so benchmarks can reference the timing helper.
#[cfg(test)]
pub mod bench_tracing;

// ════════════════════════════════════════════════════════════
//  Strategy enum
// ════════════════════════════════════════════════════════════

/// Which SVD algorithm to use for subspace evolution (§ALGO S-4.2 Phase 2).
///
/// Selectable at runtime via [`SentinelConfig::svd_strategy`](crate::SentinelConfig).
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SvdStrategy {
    /// Dense thin SVD of the full composite matrix M ∈ ℝ^(d × (k+b)).
    ///
    /// Correct and simple but O(d·(k+b)²) per call. This is the
    /// original implementation.
    Naive,

    /// Brand's incremental SVD (Brand 2006).
    ///
    /// Projects onto the current basis, QR-orthogonalises the residual,
    /// and SVDs a small (k+b)×(k+b) kernel. O((k+b)³ + d·b²) per call.
    #[default]
    Brand,
}

// ════════════════════════════════════════════════════════════
//  Result type
// ════════════════════════════════════════════════════════════

/// Output of a subspace evolution step.
///
/// Contains the updated basis vectors and singular values, ready
/// to be written back into the tracker's state.
#[derive(Debug, Clone)]
pub struct SubspaceUpdate {
    /// Updated orthonormal basis, shape `(dim, n)` where
    /// `n = min(k+b, d, cap)`.
    pub basis: Mat<f64>,

    /// Updated singular values, length `n`.
    pub sigmas: Vec<f64>,

    /// How many components are meaningful (`n`).
    pub n: usize,
}

// ════════════════════════════════════════════════════════════
//  Dispatch
// ════════════════════════════════════════════════════════════

thread_local! {
    /// Alternates execution order in the oracle so that neither
    /// strategy consistently benefits from warmed caches or branch
    /// predictors.  Toggled on every oracle-active call.
    static ORACLE_FLIP: Cell<bool> = const { Cell::new(false) };
}

/// Run one SVD strategy under a named tracing span.
#[allow(clippy::too_many_arguments)]
fn run_svd(
    which: SvdStrategy,
    current_basis: &Mat<f64>,
    sigmas: &[f64],
    z: &Mat<f64>,
    residual: &Mat<f64>,
    sqrt_lambda: f64,
    k: usize,
    cap: usize,
) -> Option<SubspaceUpdate> {
    match which {
        SvdStrategy::Naive => {
            let _span = tracing::info_span!("svd_naive").entered();
            naive_svd::evolve(current_basis, sigmas, z, residual, sqrt_lambda, k, cap)
        }
        SvdStrategy::Brand => {
            let _span = tracing::info_span!("svd_brand").entered();
            brand_svd::evolve(current_basis, sigmas, z, residual, sqrt_lambda, k, cap)
        }
    }
}

/// Run subspace evolution using the selected strategy (§ALGO S-4.2 Phase 2).
///
/// In **debug builds** (or when a `DEBUG`-level tracing subscriber is
/// attached), both strategies are executed and compared element-wise —
/// a continuous oracle test.  Execution order alternates on each call
/// via a thread-local toggle so neither strategy consistently benefits
/// from warmed caches.
///
/// # Arguments
///
/// * `strategy` — which algorithm to use for the returned result.
/// * `current_basis` — current `U_k`, shape `(d, cap)`.  Only `[:, :k]` is active.
/// * `sigmas` — current singular values, length `cap`.  Only `[:k]` meaningful.
/// * `z` — latent projection from Phase 1: `X · U_k`, shape `(b, k)`.
/// * `residual` — reconstruction residual from Phase 1: `X − X̂`, shape `(b, d)`.
/// * `sqrt_lambda` — `√λ` (square root of the forgetting factor).
/// * `k` — current active rank.
/// * `cap` — hard ceiling on rank.
///
/// # Returns
///
/// `Some(SubspaceUpdate)` on success, `None` if SVD failed to converge.
///
/// # Panics
///
/// Panics (via `assert!`) if the oracle is active and the two SVD
/// strategies produce results that differ beyond tolerance.
#[allow(clippy::too_many_arguments)]
pub fn evolve(
    strategy: SvdStrategy,
    current_basis: &Mat<f64>,
    sigmas: &[f64],
    z: &Mat<f64>,
    residual: &Mat<f64>,
    sqrt_lambda: f64,
    k: usize,
    cap: usize,
) -> Option<SubspaceUpdate> {
    let oracle_active = cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG);

    if !oracle_active {
        let result = run_svd(strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
        if result.is_some() {
            return result;
        }
        // Fallback: the primary strategy could not handle these
        // dimensions (e.g. Brand with b+k > d).  Try the other.
        let other_strategy = match strategy {
            SvdStrategy::Naive => SvdStrategy::Brand,
            SvdStrategy::Brand => SvdStrategy::Naive,
        };
        return run_svd(other_strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
    }

    // Oracle: run both strategies and compare results.
    //
    // Active in debug builds unconditionally, or in release builds
    // when a DEBUG-level tracing subscriber is attached.  Uses the
    // same span names (`svd_brand` / `svd_naive`) so the
    // SpanTimingLayer captures both strategies' cost from a single
    // run — no need for the benchmarks to loop over strategies.
    //
    // A thread-local bool alternates execution order so that
    // neither strategy consistently benefits from warmed caches
    // or branch predictors.
    let other_strategy = match strategy {
        SvdStrategy::Naive => SvdStrategy::Brand,
        SvdStrategy::Brand => SvdStrategy::Naive,
    };

    let flip = ORACLE_FLIP.with(|f| {
        let v = f.get();
        f.set(!v);
        v
    });

    let (result, other) = if flip {
        // Reversed: run other strategy first.
        let o = run_svd(other_strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
        let r = run_svd(strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
        (r, o)
    } else {
        // Normal: run selected strategy first.
        let r = run_svd(strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
        let o = run_svd(other_strategy, current_basis, sigmas, z, residual, sqrt_lambda, k, cap);
        (r, o)
    };

    if let (Some(a), Some(b)) = (&result, &other) {
        let (a_n, b_n) = (a.n, b.n);
        assert_eq!(a_n, b_n, "maths oracle: n mismatch ({a_n} vs {b_n})");
        compare_subspace_updates(a, b, strategy, other_strategy);
    }

    // Use primary if it succeeded; otherwise fall back to the other
    // strategy (e.g. Brand cannot handle b+k > d, naïve can).
    if result.is_some() { result } else { other }
}

/// Compare singular values and spectral projectors of two models.
///
/// # Panics
///
/// Panics when the models disagree beyond the precision budget. The caller
/// enables this assertion only while the numerical oracle is active.
fn compare_subspace_updates(
    update_a: &SubspaceUpdate,
    update_b: &SubspaceUpdate,
    strategy_a: SvdStrategy,
    strategy_b: SvdStrategy,
) {
    let rank = update_a.n;
    assert_eq!(rank, update_b.n, "maths oracle: rank mismatch");
    let dims = update_a.basis.nrows();
    assert_eq!(dims, update_b.basis.nrows(), "maths oracle: dimension mismatch");
    if rank == 0 {
        return;
    }

    // Require half-significand agreement with one guard bit: sqrt(epsilon)/2
    // is 2^-27 for f64, tighter than the former 1e-8 relative sigma check.
    // This is the oracle's accuracy requirement, not an error estimate fitted
    // to an update trajectory. Both strategies receive the same prior state.
    let relative = f64::EPSILON.sqrt() / 2.0;
    let largest = update_a.sigmas[0].abs().max(update_b.sigmas[0].abs());
    // A d-term dot product has relative rounding bound gamma_d=d*eps/(1-d*eps)
    // without overflow or significant underflow (zeros are exact). Covariance
    // noise at that scale corresponds to
    // singular values sqrt(gamma_d)*sigma_max; only two values below this
    // common absolute floor may be treated as unresolved.
    let dimension = f64::from(u32::try_from(dims).expect("tracker dimension fits u32"));
    let gamma = dimension * f64::EPSILON / dimension.mul_add(-f64::EPSILON, 1.0);
    let zero_floor = gamma.sqrt() * largest;
    for idx in 0..rank {
        let left = update_a.sigmas[idx];
        let right = update_b.sigmas[idx];
        assert!(left.is_finite() && right.is_finite() && left >= 0.0 && right >= 0.0);
        let difference = (left - right).abs();
        let unresolved = left.max(right) <= zero_floor && difference <= zero_floor;
        assert!(
            unresolved || difference <= relative * left.max(right),
            "maths oracle: sigma[{idx}] mismatch: {strategy_a:?}={left:.12e}, {strategy_b:?}={right:.12e}, zero_floor={zero_floor:.12e}"
        );
    }

    // The allowed normwise perturbation is eta=relative*sigma_max. Weyl's
    // singular-value intervals of radius eta overlap at gaps <=2*eta, so
    // compare those columns as a block. This budget must not be inferred
    // merely from matching sigmas: the projector check tests its consequence.
    let perturbation = relative * largest;
    let mut start = 0;
    while start < rank {
        if update_a.sigmas[start].max(update_b.sigmas[start]) <= zero_floor {
            break;
        }
        let mut end = start + 1;
        while end < rank
            && (update_a.sigmas[end - 1] - update_a.sigmas[end]).max(update_b.sigmas[end - 1] - update_b.sigmas[end])
                <= 2.0 * perturbation
        {
            end += 1;
        }
        compare_projectors(update_a, update_b, start, end, perturbation, gamma);
        start = end;
    }
}

fn compare_projectors(left: &SubspaceUpdate, right: &SubspaceUpdate, start: usize, end: usize, perturbation: f64, gamma: f64) {
    let rank = end - start;
    let block_size = f64::from(u32::try_from(rank).expect("tracker rank fits u32"));
    let mut gap = left.sigmas[end - 1].min(right.sigmas[end - 1]);
    if start > 0 {
        gap = gap.min((left.sigmas[start - 1] - left.sigmas[start]).min(right.sigmas[start - 1] - right.sigmas[start]));
    }
    if end < left.n {
        gap = gap.min((left.sigmas[end - 1] - left.sigmas[end]).min(right.sigmas[end - 1] - right.sigmas[end]));
    }

    // For orthonormal bases, ||P-Q||_F^2=2*(r-||A^T B||_F^2).
    // Each overlap dot has error <=gamma_d; squaring adds at most
    // 2*gamma_d+gamma_d^2, and summing r^2 terms adds gamma_(r^2).
    // Include the final subtraction and factor two in the squared floor.
    let sum_terms = block_size * block_size;
    let sum_gamma = sum_terms * f64::EPSILON / sum_terms.mul_add(-f64::EPSILON, 1.0);
    let rounding = 2.0
        * f64::EPSILON.mul_add(
            block_size,
            sum_gamma.mul_add(block_size, sum_terms * gamma.mul_add(gamma, 2.0 * gamma)),
        );
    let mut overlap = 0.0;
    for a_col in start..end {
        for b_col in start..end {
            let mut dot = 0.0;
            for row in 0..left.basis.nrows() {
                dot = left.basis[(row, a_col)].mul_add(right.basis[(row, b_col)], dot);
            }
            overlap = dot.mul_add(dot, overlap);
        }
    }
    let distance_sq = (2.0 * (block_size - overlap)).max(0.0);
    // Wedin's combined left/right Frobenius sin-theta bound, with each
    // residual <=sqrt(r)*eta, gives ||P-Q||_F <=2*sqrt(r)*eta/delta.
    // Weyl reduces the available separation to delta=gap-eta. Include
    // the omitted zero spectrum, so even a rank-one model checks its axis.
    let separation = gap - perturbation;
    assert!(separation > 0.0, "maths oracle: unresolved nonzero projector separation");
    let tolerance_sq = (4.0 * block_size).mul_add((perturbation / separation).powi(2), rounding);
    assert!(
        distance_sq <= tolerance_sq,
        "maths oracle: projector [{start}..{end}] mismatch: distance_sq={distance_sq:.12e}, tolerance_sq={tolerance_sq:.12e}, gap={gap:.12e}"
    );
}

// Resolve a repeated singular space before applying the rank cap. Project
// coordinate axes in order and re-orthogonalise them twice, so a truncated
// repeated block retains the same plane regardless of the SVD's basis choice.
// Singletons and unresolved zero-energy columns retain their original basis.
fn truncate_update(mut basis: Mat<f64>, sigmas: Vec<f64>, cap: usize) -> SubspaceUpdate {
    let dims = basis.nrows();
    let dimension = f64::from(u32::try_from(dims).expect("tracker dimension fits u32"));
    let gamma = dimension * f64::EPSILON / dimension.mul_add(-f64::EPSILON, 1.0);
    let largest = sigmas.first().copied().unwrap_or(0.0);
    // Use the same intervals as the oracle: eta=sqrt(epsilon)*sigma_max/2,
    // so numerical ties have overlapping intervals at gaps <=2*eta. The
    // complete block must choose its basis before truncation, including a
    // neighbouring value just beyond the cap. This is a precision policy,
    // not a measured error bound on the SVD backend.
    let tie_gap = f64::EPSILON.sqrt() * largest;
    let mut start = 0;
    while start < sigmas.len() && sigmas[start] > gamma.sqrt() * largest {
        let mut end = start + 1;
        while end < sigmas.len() && sigmas[end - 1] - sigmas[end] <= tie_gap {
            end += 1;
        }
        if end - start > 1 {
            canonicalize_cluster(&mut basis, start, end);
        }
        start = end;
    }
    let n = cap.min(sigmas.len());
    SubspaceUpdate {
        basis: basis.subcols(0, n).to_owned(),
        sigmas: sigmas.into_iter().take(n).collect(),
        n,
    }
}

fn canonicalize_cluster(basis: &mut Mat<f64>, start: usize, end: usize) {
    let dims = basis.nrows();
    let width = end - start;
    let mut canonical = Mat::<f64>::zeros(dims, width);
    let mut chosen = 0;
    // For unit input columns, candidate construction uses d*r FMAs; two
    // Gram-Schmidt passes use at most 2*r*(d FMAs + 2*d scalar updates),
    // and the squared norm uses d FMAs: at most q=d*(7*r+1) roundings.
    // Fusing the vector updates below only reduces this conservative count.
    // Under finite arithmetic without significant underflow, gamma_q is
    // the squared-energy resolution we require of a usable pivot. Tiny
    // cancellation residues below that floor do not choose an arbitrary axis.
    let terms = f64::from(u32::try_from(dims * (7 * width + 1)).expect("tracker operation count fits u32"));
    let pivot_floor_sq = terms * f64::EPSILON / terms.mul_add(-f64::EPSILON, 1.0);
    for axis in 0..dims {
        let mut candidate: Vec<f64> = (0..dims)
            .map(|row| (start..end).fold(0.0, |sum, col| basis[(row, col)].mul_add(basis[(axis, col)], sum)))
            .collect();
        for _ in 0..2 {
            for col in 0..chosen {
                let dot = candidate
                    .iter()
                    .enumerate()
                    .fold(0.0, |sum, (row, value)| canonical[(row, col)].mul_add(*value, sum));
                for (row, value) in candidate.iter_mut().enumerate() {
                    *value = dot.mul_add(-canonical[(row, col)], *value);
                }
            }
        }
        let norm_sq = candidate.iter().fold(0.0, |sum, value| value.mul_add(*value, sum));
        if norm_sq > pivot_floor_sq {
            let norm = norm_sq.sqrt();
            for (row, value) in candidate.into_iter().enumerate() {
                canonical[(row, chosen)] = value / norm;
            }
            chosen += 1;
            if chosen == width {
                break;
            }
        }
    }
    // A numerically unresolved basis is left intact, so this normalization
    // cannot hide a disagreement from the oracle or turn it into a fallback.
    if chosen == width {
        basis.subcols_mut(start, width).copy_from(&canonical);
    }
}
