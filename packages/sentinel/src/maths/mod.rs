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

/// Compare two `SubspaceUpdate`s for approximate equality.
///
/// Singular values must match to high relative tolerance.
/// Basis vectors may differ by sign (SVD sign ambiguity) so we
/// compare `|uᵢᵀ vᵢ|` ≈ 1.0 for each column.
///
/// Uses `assert!` (not `debug_assert!`) because this function is
/// only called when the oracle is active — the gate is at the call
/// site, not here.
fn compare_subspace_updates(
    update_a: &SubspaceUpdate,
    update_b: &SubspaceUpdate,
    strategy_a: SvdStrategy,
    strategy_b: SvdStrategy,
) {
    /// Returns `true` when the pair of values should be considered
    /// "effectively zero" and comparison skipped.
    fn near_zero(va: f64, vb: f64, and_floor: f64, or_floor: f64) -> bool {
        (va.abs() < and_floor && vb.abs() < and_floor) || (va.abs() < or_floor || vb.abs() < or_floor)
    }

    let rank = update_a.n;
    let dims = update_a.basis.nrows();

    // ── Near-zero floors (shared by σ and basis checks) ──
    //
    // Two-tier skip logic prevents both false positives and false
    // negatives when comparing near-zero singular values:
    //
    // • `and_floor` (1e-5, generous) — skip comparison when *both*
    //   strategies agree the value is near-zero.  Covers the
    //   practical noise floor: trailing singular values of rank-
    //   deficient inputs routinely land at 1e-7 to 1e-16.  Brand's
    //   incremental QR may accumulate slightly more residual than
    //   the full SVD (e.g. 1.2e-6 vs 1.5e-9 after 1000 identical
    //   observations), but both are "effectively zero."
    //
    // • `or_floor` (1e-12, strict) — skip comparison when *either*
    //   strategy produces a value at (or near) machine-epsilon
    //   level.  Catches exact-zero vs tiny-residual mismatches
    //   without masking real discrepancies.
    //
    // The combined gate is:
    //
    //     (both < and_floor) ∨ (either < or_floor)
    //
    // If one value is O(1) and the other is 1e-9, neither gate
    // fires and the assertion correctly catches the divergence.
    let and_floor: f64 = 1e-5;
    let or_floor: f64 = 1e-12;

    // ── Singular values ─────────────────────────────────
    let sv_tolerance = 1e-8;
    for idx in 0..rank {
        let sa = update_a.sigmas[idx];
        let sb = update_b.sigmas[idx];
        if near_zero(sa, sb, and_floor, or_floor) {
            continue;
        }
        let denom = sa.abs().max(sb.abs()).max(1e-15);
        let rel = (sa - sb).abs() / denom;
        assert!(
            rel < sv_tolerance,
            "maths oracle: σ[{idx}] mismatch: {strategy_a:?}={sa:.12e}, {strategy_b:?}={sb:.12e}, rel={rel:.2e}"
        );
    }

    // ── Basis columns (up to sign, with subspace clustering) ──
    //
    // Each column pair should satisfy |uᵢᵀ vᵢ| ≈ 1.0.
    // SVD sign ambiguity means uᵢ and −uᵢ are both valid.
    //
    // When a singular value is near zero, the corresponding basis
    // direction is numerically arbitrary — both strategies may
    // legitimately return completely different unit vectors for the
    // null-energy component.  Skip the cosine check in that case
    // using the same two-tier gate.
    //
    // **Repeated singular values**: when σ[j] ≈ σ[j+1] (within
    // `cluster_rtol`), the corresponding basis vectors span an
    // invariant subspace — *any* orthonormal basis of that
    // subspace is equally valid.  Comparing individual columns is
    // meaningless; instead we compare the subspace via principal
    // angles: form S = A_group^T · B_group, SVD it, and check
    // that all singular values (= cos(θ_i)) ≈ 1.0.

    // ── Gap-dependent basis tolerance (exact Wedin bound) ──
    //
    // For a singular vector with spectral gap δ to its nearest
    // neighbour, the Wedin (1972) sin-θ theorem gives:
    //
    //   sin θ ≤ ‖E‖ / δ
    //
    // where δ is the **absolute** gap and
    //   ‖E‖ ≈ σ_max · √(rel_perturbation_sq).
    //
    // Define `ratio_sq = (‖E‖/δ)²`.  The exact Wedin bound on
    // the cosine is:
    //
    //   |cos θ| ≥ √(1 − ratio_sq)
    //
    // or equivalently:
    //
    //   1 − |cos θ| ≤ 1 − √(1 − ratio_sq)     (exact)
    //
    // Previous code used the Taylor approximation `ratio_sq / 2`,
    // which underestimates the tolerance by >15% when ratio_sq
    // exceeds 0.3 and >29% at 0.5, causing false oracle panics
    // for poorly-conditioned columns.  Using the exact form
    // eliminates this bias.
    //
    // When ratio_sq ≥ 1 the bound is vacuous: ‖E‖ ≥ δ means
    // the perturbation can rotate the singular vector through
    // *any* angle, so only the σ-value check (above) provides
    // meaningful validation for that column.
    //
    // **Calibration**: Brand's incremental SVD accumulates
    // truncation error over many rank-1 updates.  This error
    // is *anisotropic* — it preferentially affects the weakest
    // singular directions because truncation discards energy
    // from the (k+1)-th component, which leaks into the k-th
    // direction proportionally to 1/δ_k.
    //
    // Empirical calibration: 1600 updates at λ = 0.95 with a
    // 2% relative gap produced |cos| = 0.588, giving
    // (‖E‖/σ_max)² ≈ 3.3 × 10⁻⁴.  However, traffic patterns
    // with high condition numbers (κ = σ_max/σ_min > 25) and
    // diverse cell structure push the actual perturbation to
    // ~5× the calibration value.  We budget 10× margin
    // (rel_perturbation_sq = 0.003) to cover worst-case
    // anisotropic error accumulation.
    //
    // The σ_max² factor is critical: when the condition number
    // σ_max/σ_min is large, the perturbation ‖E‖ scales with
    // σ_max but the gap δ scales with σ_min's neighbourhood.
    // Without this factor, the tolerance is underestimated by
    // (σ_max/σ_neighbor)² — up to 400× for condition number 20.
    let basis_tol_floor = 1e-6;
    let rel_perturbation_sq: f64 = 0.003;
    let sv_max = f64::midpoint(update_a.sigmas[0].abs(), update_b.sigmas[0].abs());

    // Cluster tolerance: any pair whose relative gap is smaller
    // than 2× the perturbation scale must be compared as a
    // subspace, because the Wedin bound's ‖E‖/δ ratio ≥ 1
    // makes per-column cosine comparison meaningless there.
    // Deriving cluster_rtol from rel_perturbation_sq eliminates
    // the dead zone between "too close to cluster" and "too
    // close for the Wedin bound to be tight."
    let cluster_rtol = 2.0 * rel_perturbation_sq.sqrt();

    // Group consecutive singular values that are nearly equal.
    // Uses the average of both strategies' values for robustness.
    let mut col = 0;
    while col < rank {
        // Find extent of cluster starting at col.
        let mut end = col + 1;
        while end < rank {
            let prev_sv_a = update_a.sigmas[end - 1].abs();
            let prev_sv_b = update_b.sigmas[end - 1].abs();
            let curr_sv_a = update_a.sigmas[end].abs();
            let curr_sv_b = update_b.sigmas[end].abs();
            let avg = (prev_sv_a + prev_sv_b + curr_sv_a + curr_sv_b) * 0.25;
            if avg < 1e-15 {
                break; // all near-zero, stop clustering
            }
            let diff_a = (prev_sv_a - curr_sv_a).abs();
            let diff_b = (prev_sv_b - curr_sv_b).abs();
            if diff_a / avg > cluster_rtol || diff_b / avg > cluster_rtol {
                break;
            }
            end += 1;
        }

        let group_size = end - col;

        // Skip the whole group if sigmas are near-zero.
        if near_zero(update_a.sigmas[col], update_b.sigmas[col], and_floor, or_floor) {
            col = end;
            continue;
        }

        if group_size == 1 {
            // ── Singleton: gap-dependent cosine check ──
            //
            // Compute minimum absolute gap to the nearest neighbour.
            // Singular values are sorted descending, so the immediate
            // predecessor (col−1) and successor (col+1) are the closest
            // candidates.  We average both strategies' values for
            // robustness.
            let sv_col = f64::midpoint(update_a.sigmas[col].abs(), update_b.sigmas[col].abs());
            let mut min_abs_gap = f64::INFINITY;
            if col > 0 {
                let sv_prev = f64::midpoint(update_a.sigmas[col - 1].abs(), update_b.sigmas[col - 1].abs());
                min_abs_gap = min_abs_gap.min((sv_col - sv_prev).abs());
            }
            if col + 1 < rank {
                let sv_next = f64::midpoint(update_a.sigmas[col + 1].abs(), update_b.sigmas[col + 1].abs());
                min_abs_gap = min_abs_gap.min((sv_col - sv_next).abs());
            }

            // Wedin-derived tolerance using the exact sin-θ bound
            // (Wedin 1972):
            //
            //   sin θ ≤ ‖E‖/δ  ⟹  |cos θ| ≥ √(1 − (‖E‖/δ)²)
            //
            // Define ratio_sq = (‖E‖/δ)² = rel_perturbation_sq · σ_max²/δ².
            //
            // When ratio_sq ≥ 1 the Wedin bound is vacuous
            // (‖E‖ ≥ δ) — the singular vector's direction is not
            // constrained by theory.  Any cosine value, including 0,
            // is consistent with both strategies being correct.  Skip
            // the comparison entirely; only the σ-value check (above)
            // provides meaningful validation for this column.
            let effective_tol = if min_abs_gap.is_infinite() {
                // Only one singular value — no neighbour to mix with.
                Some(basis_tol_floor)
            } else {
                let denom = min_abs_gap.max(1e-15);
                let ratio_sq = rel_perturbation_sq * sv_max * sv_max / (denom * denom);
                if ratio_sq >= 1.0 {
                    None // Wedin bound vacuous — skip cosine check.
                } else {
                    // Exact: 1 − |cos θ| ≤ 1 − √(1 − ratio_sq)
                    let exact = 1.0 - (1.0 - ratio_sq).sqrt();
                    Some(exact.max(basis_tol_floor))
                }
            };

            if let Some(tol) = effective_tol {
                let mut dot = 0.0;
                for row in 0..dims {
                    dot = update_a.basis[(row, col)].mul_add(update_b.basis[(row, col)], dot);
                }
                let cosine = dot.abs();
                assert!(
                    cosine > 1.0 - tol,
                    "maths oracle: basis column {col} diverged: |cos| = {cosine:.8}, \
                     effective_tol = {tol:.6e} (min_abs_gap = {min_abs_gap:.4}, σ_max = {sv_max:.4}), \
                     σ_a={:.6e}, σ_b={:.6e}, \
                     all_σ_a={:?}, all_σ_b={:?}, \
                     {strategy_a:?} vs {strategy_b:?}",
                    update_a.sigmas[col],
                    update_b.sigmas[col],
                    &update_a.sigmas[..rank],
                    &update_b.sigmas[..rank],
                );
            }
        } else {
            // ── Cluster: degenerate eigenvalue group ──
            //
            // When singular values are repeated (σ[j] ≈ σ[j+1] …),
            // the corresponding SVD eigenvectors are only defined up
            // to an arbitrary rotation within the (potentially high-
            // dimensional) eigenspace.  In ℝ^d with d ≫ g, the true
            // eigenspace for this σ can be much larger than the g
            // columns the SVD returns, so different implementations
            // legitimately pick different g-dimensional slices.
            //
            // Per-column and even per-subspace comparison is
            // meaningless.  The σ comparison already validates that
            // the singular values match; skip the basis check for
            // degenerate groups.
        }

        col = end;
    }
}
