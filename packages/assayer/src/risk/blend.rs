// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`prior_level_blend_weight_near_zero`] | risk | The anchor earns weight only by being more certain than the sister is on the features they share. Two models sitting at the same isotropic prior are equally uncertain, so the anchor gets essentially none of it and the sister's own view stands. At cold start, borrowing from a model that knows no more than you do would add nothing but its bias. |
//! | [`time_correction_cancels_in_weight`] | risk | The blend weight is a ratio of the two models' uncertainties, and the staleness correction multiplies both of them, so it cancels exactly: doubling the correction leaves the weight bit-for-bit unchanged. Time passing makes both models less trustworthy, not one of them relatively more so, and the weight is computed from the raw forms to keep it that way. |
//! | [`time_correction_affects_variance`] | risk | Where the staleness correction does land is the uncertainty: raising it moves the blended variance. A model that has not been updated in a while is not wrong in a knowable direction, so what ages is the confidence attached to its answer rather than the answer itself. |
//! | [`projected_qf_matches_submatrix`] | risk | Reading the sister's uncertainty over just the anchor's features by scattered indexing gives the same number as physically cutting that submatrix out and working on it. The projection is a restriction of the full covariance, not an approximation of one, so the weight it feeds compares like with like. |
//! | [`mixture_variance_non_negative`] | risk | The three-term mixture stays non-negative across a wide sweep of weights, component variances and disagreeing means. The cross-term that accounts for the two models disagreeing only ever adds, so no combination of inputs can produce a negative variance for a caller to take a square root of. |
//! | [`qf_clamp_on_negative_eigenvalue`] | risk | A covariance that has drifted very slightly indefinite through accumulated rounding yields a negative quadratic form, and the blend clamps it to zero before treating it as a variance. The alternative is a negative uncertainty propagating into the weight ratio and the mixture, where it would be far harder to recognise as arithmetic noise. |
//! | [`kappa_eff_soft_transition`] | risk | The calibration constant follows the blend weight through a smooth transition rather than a switch: at no weight it is the sister's, at full weight the anchor's, and around the regime boundary it sits near the midpoint. Because the two regimes are calibrated separately, a hard switch would put a step discontinuity in the probability an entity is assigned as its weight crosses. |
//! | [`p_bad_uses_kappa_eff`] | risk | The risk probability is the sigmoid of the blended score divided by the blended calibration constant, and the division is not optional: at a constant of two the probability is visibly that of the halved score and demonstrably not that of the raw one. Skipping the division would return the model's uncalibrated confidence as though it were a probability. |
//! | [`no_overflow_at_large_rho`] | risk | An extreme score saturates the probability instead of breaking it: a blended score far outside the ordinary range still yields a finite value, pressed up against one. An entity the model is overwhelmingly sure about produces an actionable near-certainty rather than a non-finite value that would poison every downstream computation. |
//! | [`blended_estimate_is_convex`] | risk | The blended score is a genuine convex combination of the two models' own scores at the weight the blend chose — even when they disagree in sign. It can therefore never fall outside the interval the two models bracket, so borrowing from the anchor cannot produce a verdict neither model would endorse. |
//! | [`symmetry_optimised_qf_matches_naive`] | risk | Computing the quadratic form as a diagonal plus a doubled upper triangle reaches the same value as iterating every element of the matrix. Exploiting the covariance's symmetry halves the work and changes nothing about what the uncertainty means. |
//! | [`dot_product_correctness`] | risk | The linear predictor behind every score in this module is an ordinary elementwise-product sum over the feature vector and the model's means, with no scaling or reordering hidden inside it. |
//! | [`quadratic_form_identity_gives_norm_squared`] | risk | Against an identity covariance the quadratic form reduces to the squared norm of the feature vector — the known answer for the case where the model has no correlations and unit variance everywhere. This anchors the column-major indexing: a transposed or mis-strided read would not land on this value. |
//! | [`variance_diagnostics_populated`] | risk | The component variances reported alongside the blend are real quantities and not decoration: each is non-negative, they recombine into the blended variance through the same three-term formula the blend used, and the sister's variance restricted to the shared subspace never exceeds its variance over all features. A reader can therefore reconstruct why the weight came out where it did instead of taking the number on trust. |
//! | [`anchor_only_w_near_one`] | risk | The converse of an anchor that earns nothing: an anchor orders of magnitude more precise than the sister over the shared features takes nearly all the weight, and the blended score lands next to the anchor's own. A sister that has learned almost nothing yet defers to the model that has, which is what makes the population-level anchor useful during a cold start. |
//! | [`dimension_1_scalar_case`] | risk | In the degenerate single-feature case the machinery reduces to its closed form exactly: the weight is one minus the ratio of the two scalar variances, and the blended score is that weight applied to the two scalar means. The general matrix code is a generalisation of the scalar arithmetic and not a separate approximation that happens to agree in the usual dimensions. |
//! | [`intervention_effectiveness_at_call_site`] | risk | The blend publishes the sister's own score separately from the blended one so a caller can measure intervention effectiveness against it: the gap between what the inherited model expects of an entity and what the operational model expects after intervention. Only a score computed from unintervened history makes that difference meaningful, which is why the inherited score survives the blend rather than being folded away into it. |
//! | [`time_correction_does_not_affect_rho`] | risk | cites (´claim:risk:the-staleness-correction-widens-the-uncertainty-without-moving-the-point-estimate´) |
//! | [`golden_vector_known_output`] | risk | Run end to end on small fixed matrices, every published quantity — both quadratic forms, the weight, both linear predictors and the blend of them — matches values worked out by hand. This is the pin that catches a refactor which keeps all the internal identities consistent while quietly changing what the blend computes. |
//! | [`mixture_variance_identity`] | risk | The blended variance is exactly the variance of the two-component mixture the blend represents, recovered independently from the moment identity in a regime where the weight is strictly interior and the two models disagree. The blend is therefore not two models averaged with an uncertainty bolted on: the disagreement between them is itself part of the uncertainty reported, which is the only honest answer when the models are pulling in different directions. |
//! | [`a_model_whose_floors_have_never_acted_borrows_nothing`] | risk | A model whose floors have never acted has borrowed nothing, and the reading says so along every direction rather than along the ones it was asked about. A fresh model's precision is its construction prior alone: no rebuild has applied a spectral floor and no clamp has bound, so both tracked masses are zero and the numerator of the share is zero whatever the direction. This is the property that keeps the whole mechanism off the path of a healthy deployment: a widening that fired a little on every verdict would be a systematic inflation of every uncertainty the package reports, which is exactly what a calibration refit would then spend its evidence undoing. |
//! | [`the_borrowed_share_rises_as_the_direction_turns`] | risk | The share has an angle, and so does the widening it drives. A model that has clamped every coordinate and then learned about one of them has borrowed a small part of the precision along the direction it learned and a large part of the precision along the directions it did not, so turning the direction of interest from the first toward the second raises the share, and raises it monotonically rather than at some threshold. The variance the tempering reports rises with it over the same sweep, which is the end-to-end statement the ruling makes: two requests against one model, differing only in which direction they ask about, receive different uncertainties. Degradation is not equally applicable to every verdict, and which verdicts it applies to is decided by where the model's evidence actually is, not by an aggregate that has averaged that question away. The share stays strictly below one throughout, and that is a property of the engine rather than of this sweep. The prior precision a model is constructed at is not tracked as floor mass and so is not counted in the numerator, so a direction the floors hold up entirely is a limit the arithmetic must handle rather than a state a live model reaches. The saturation is tested against that limit where it can be reached, which is on the tempering itself. |
//! | [`the_borrowed_share_blends_by_the_weights_the_variances_take`] | risk | Two models that have borrowed differently blend their shares by the same weights the blend gave their variances, and the disagreement between them dilutes the result rather than inheriting a share of its own. The three-term mixture's last term is the extra variance from not knowing which model is right; that is uncertainty their evidence produced, and crediting a floor with it would widen a verdict twice for one cause. The identity this test checks is the definition read back off the result: borrowed variance over total variance, with the components weighted as the blend weighted them. |
//! | [`the_covariance_reading_is_the_definition_at_sigma_phi`] | risk | The share the assessment path reads from the published covariance is the share the model itself would report, because the two are one function evaluated along one direction. The definition divides the floors' contribution to the precision along a direction by the precision along it, and needs the precision matrix, which is not published. The identity `(Σφ)ᵀB(Σφ) = φᵀΣφ` supplies the denominator from the covariance alone, and `Σφ` is the direction whose precision holds that variance up — so the published reading is the definition evaluated at `Σφ`, and this test asserts exactly that by asking the model for its own reading there. The model whose two readings are compared has been rebuilt, because the identity is an identity only where the pair the model holds are actually inverses; between rebuilds they disagree by the clamp's own accumulated contribution, which the drift monitor reads and which no reading through the covariance can see. |

//! Blend weight (´def:risk:subspace-blend´) and uncertainty
//! (´prop:risk:blend-variance´) computation.
//!
//! This module computes the anchor-sister blend for risk estimation:
//! three quadratic forms, a blend weight from the shared subspace,
//! and a blended uncertainty with time correction.
//!
//! # Cross-References
//!
//! - (´dec:calibration:raw-weight-forms´) — why the weight is computed from
//!   raw rather than time-corrected forms
//! - (´def:risk:probability´) — the probability-space uncertainty this feeds
//! - (´def:platt:regimes´) — the `κ_eff` interpolation by blend weight
//! - (´dec:risk:evidence-only-uncertainty´) — the borrowed share the blend
//!   carries, and why it is blended in variance space

use crate::model::parameters::{FloorMass, ModelParameters};
use crate::numerics::regime_transition;

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Guard against division by zero in blend weight computation.
///
/// The guard is added to the variance the weight then divides by, so its
/// whole influence is its ratio to that variance. At prior-level
/// uncertainty (`σ² ~ 1/λ_prior = 10`) the ratio is `1e-13`, and the
/// variance would have to fall to `1e-9` before the ratio reached even
/// `1e-3`; no state between those moves `w` observably
/// (´tab:degradation:guard-magnitudes´).
///
/// The guard's site is here rather than in the record
/// (´dec:calibration:clamped-forms´).
///
/// ´const:assayer:blend-weight-guard´ (´alg:const:scalar´)
/// ´const:assayer:blend-weight-guard-scalar-1en12´
const EPSILON_BLEND: f64 = 1e-12;

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Result of the blend computation between sister and anchor models.
///
/// Contains all quantities needed by:
/// - The resonance derivation (´rule:derivation:closed-inputs´):
///   `rho_effective`, `variance_effective`, `kappa_effective`
/// - The `RiskBasis` in the `RiskAssessment` (´schema:risk:basis´): all fields
/// - The convergence tracker (´dec:health:concrete-trackers´): `anchor_weight`
///   for diagnostics
/// - The intervention effectiveness (´def:risk:intervention-effectiveness´):
///   `rho_inherited` - `ρ̂_opr` (`ρ̂_opr` is computed separately at the call site)
///
/// The construction these fields come from is the subspace-restricted blend
/// (´def:risk:subspace-blend´).
#[derive(Clone, Debug)]
pub struct BlendResult {
    /// Anchor weight `w ∈ [0, 1]` (0 = sister only, 1 = anchor only).
    pub anchor_weight: f64,
    /// Sister linear predictor `ρ̂_inh = φ̂ᵀμ_sister`.
    ///
    /// Used for intervention effectiveness: `Δρ̂ = ρ̂_inh - ρ̂_opr`
    /// (´def:risk:intervention-effectiveness´).
    pub rho_inherited: f64,
    /// Anchor linear predictor `ρ̂_anc = φ̃ᵀμ_anchor`.
    ///
    /// Used for diagnostics and calibration analysis.
    #[allow(dead_code)] // calibration diagnostics (´def:platt:regimes´)
    pub rho_anchor: f64,
    /// Blended linear predictor `ρ̂_eff = (1-w)ρ̂_inh + w·ρ̂_anc`.
    pub rho_effective: f64,
    /// Blended variance `σ²_eff` (time-corrected, three-term mixture).
    pub variance_effective: f64,
    /// Full sister variance `σ²_inh(φ)`, time-corrected.
    ///
    /// Used downstream for Q-bandwidth in resonance derivation
    /// (´def:rendering:bandwidths´).
    #[allow(dead_code)] // resonance derivation (´rule:derivation:closed-inputs´)
    pub variance_inherited: f64,
    /// Anchor variance `σ²_anc(φ̃)`, time-corrected.
    pub variance_anchor: f64,
    /// Projected sister variance `σ²_{inh,S}(φ)` in the shared anchor subspace,
    /// time-corrected.
    ///
    /// Diagnostic: reveals why `anchor_weight` has its current value.
    pub variance_inherited_shared: f64,
    /// Blended condition number `κ_eff`.
    pub kappa_effective: f64,
    /// The share of `variance_effective` the two models' floors are holding up
    /// rather than their evidence (´dec:risk:evidence-only-uncertainty´).
    ///
    /// The same weights the blend applies to the component variances, applied
    /// to the components' own borrowed shares. The disagreement term carries
    /// no borrowed share and therefore dilutes this one, which is the honest
    /// reading: two models disagreeing is uncertainty their evidence produced,
    /// not confidence their floors lent.
    pub borrowed_share_effective: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Core Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the anchor-sister blend for risk estimation.
///
/// # Arguments
///
/// * `sister` — Sister model parameters (μ, Σ at full dimension p)
/// * `anchor` — Anchor model parameters (μ, Σ at dimension `p_a` = 15)
/// * `sister_mass` — What the sister's two floors have added to its precision
/// * `anchor_mass` — What the anchor's two floors have added to its precision
/// * `phi_hat` — Standardised feature vector (dimension p)
/// * `phi_tilde` — Anchor subvector (dimension `p_a` = 15)
/// * `anchor_indices` — Maps anchor features to full φ̂ positions (`None` for absent)
/// * `time_correction` — `1 / γ_t^Δt` (applied to variance only)
/// * `kappa_sister` — Platt calibration κ for sister regime
/// * `kappa_anchor` — Platt calibration κ for anchor regime
///
/// # Algorithm
///
/// 1. Compute three quadratic forms:
///    - `σ²_anc = φ̃ᵀ Σ_anchor φ̃` (anchor uncertainty)
///    - `σ²_{inh,S} = proj_QF(Σ_sister, φ̃, indices)` (sister in shared subspace)
///    - `σ²_inh = φ̂ᵀ Σ_sister φ̂` (sister full uncertainty)
///
/// 2. Blend weight from **raw** QF ratio (time correction cancels):
///    `w = clamp((1 - σ²_anc / (σ²_{inh,S} + ε)), 0, 1)`
///
/// 3. Linear predictors:
///    `ρ̂_inh = φ̂ᵀ μ_sister`, `ρ̂_anc = φ̃ᵀ μ_anchor`
///    `ρ̂_eff = (1-w) ρ̂_inh + w ρ̂_anc`
///
/// 4. Blended variance (time-corrected):
///    Apply `time_correction` to each QF, then three-term mixture.
///
/// 5. Soft-blend κ via `regime_transition(w)`.
///
/// 6. Borrowed share, blended in variance space by the same weights
///    (´dec:risk:evidence-only-uncertainty´). Each model's share is read from
///    its **raw** covariance, because the staleness correction inflates a
///    variance without moving any mass into or out of the precision matrix.
#[must_use]
#[allow(clippy::too_many_arguments)] // Justified: pure function taking two model snapshots + feature vectors + scalar parameters; grouping into structs would obscure the data flow
pub fn compute_blend(
    sister: &ModelParameters,
    anchor: &ModelParameters,
    sister_mass: &FloorMass,
    anchor_mass: &FloorMass,
    phi_hat: &[f64],
    phi_tilde: &[f64],
    anchor_indices: &[Option<usize>],
    time_correction: f64,
    kappa_sister: f64,
    kappa_anchor: f64,
) -> BlendResult {
    debug_assert_eq!(phi_hat.len(), sister.p, "φ̂ dimension mismatch with sister");
    debug_assert_eq!(phi_tilde.len(), anchor.p, "φ̃ dimension mismatch with anchor");
    debug_assert_eq!(anchor_indices.len(), anchor.p, "anchor_indices length mismatch");
    debug_assert!(time_correction >= 1.0, "time_correction must be ≥ 1.0; got {time_correction}");

    // ─── Step 1: Compute three quadratic forms ───
    // σ²_anc = φ̃ᵀ Σ_anchor φ̃
    let sigma2_anc = quadratic_form_from_flat(&anchor.covariance_data, phi_tilde, anchor.p).max(0.0);

    // σ²_{inh,S} = projected QF from sister in shared anchor subspace
    let sigma2_inh_shared = projected_quadratic_form(&sister.covariance_data, sister.p, phi_tilde, anchor_indices).max(0.0);

    // σ²_inh = φ̂ᵀ Σ_sister φ̂
    let sigma2_inh = quadratic_form_from_flat(&sister.covariance_data, phi_hat, sister.p).max(0.0);

    // ─── Step 2: Blend weight from raw QF ratio ───
    // Time correction cancels in the ratio, so use raw values.
    let w = (1.0 - sigma2_anc / (sigma2_inh_shared + EPSILON_BLEND)).clamp(0.0, 1.0);

    // ─── Step 3: Linear predictors ───
    let rho_inh = dot_product(&sister.mu, phi_hat);
    let rho_anc = dot_product(&anchor.mu, phi_tilde);
    let rho_eff = (1.0 - w).mul_add(rho_inh, w * rho_anc);

    // ─── Step 4: Blended variance with time correction ───
    // Apply time correction to each variance component.
    // All uncertainty computations read the time-corrected covariance
    // (´def:runtime:time-correction´). Point estimates are unaffected.
    let tc_sigma2_anc = sigma2_anc * time_correction;
    let tc_sigma2_inh = sigma2_inh * time_correction;
    let tc_sigma2_inh_shared = sigma2_inh_shared * time_correction;

    // Three-term mixture variance (´prop:risk:blend-variance´):
    // σ²_eff = (1-w)σ²_inh + w·σ²_anc + w(1-w)(ρ̂_inh - ρ̂_anc)²
    let rho_diff = rho_inh - rho_anc;
    let base_var = (1.0 - w).mul_add(tc_sigma2_inh, w * tc_sigma2_anc);
    let sigma2_eff = (w * (1.0 - w) * rho_diff).mul_add(rho_diff, base_var);
    debug_assert!(sigma2_eff >= 0.0, "blended variance must be non-negative; got {sigma2_eff}");

    // ─── Step 5: Soft-blend κ_eff via regime_transition ───
    let t = regime_transition(w);
    let kappa_eff = (1.0 - t).mul_add(kappa_sister, t * kappa_anchor);

    // ─── Step 6: The borrowed share, blended in variance space ───
    // Each model's share is read from its raw covariance and its own masses,
    // then the two are combined with the weights the blend applied to their
    // variances (´dec:risk:evidence-only-uncertainty´).
    let share_inh = covariance_borrowed_share(sister, sister_mass, phi_hat, sigma2_inh);
    let share_anc = covariance_borrowed_share(anchor, anchor_mass, phi_tilde, sigma2_anc);
    let borrowed_effective = blend_borrowed_share(w, share_inh, tc_sigma2_inh, share_anc, tc_sigma2_anc, sigma2_eff);

    BlendResult {
        anchor_weight: w,
        rho_inherited: rho_inh,
        rho_anchor: rho_anc,
        rho_effective: rho_eff,
        variance_effective: sigma2_eff,
        variance_inherited: tc_sigma2_inh,
        variance_anchor: tc_sigma2_anc,
        variance_inherited_shared: tc_sigma2_inh_shared,
        kappa_effective: kappa_eff,
        borrowed_share_effective: borrowed_effective,
    }
}

/// The share of one model's uncertainty along `phi` that its floors are
/// holding up, read from the published covariance
/// (´dec:risk:evidence-only-uncertainty´).
///
/// The share is defined along a direction as the floors' own contribution to
/// the precision there, over the precision there. The precision matrix is not
/// published, so this reads it through an identity instead: with `ψ = Σφ`,
/// `ψᵀBψ = φᵀΣBΣφ = φᵀΣφ`, which is the variance the caller has already
/// computed. The share is therefore taken along `ψ` — and `ψ` is not an
/// arbitrary substitute for `φ` but the direction whose precision holds that
/// variance up, which is the direction the tempering is about. The two
/// coincide wherever `φ` is an eigenvector of `B`, because `Σφ` is then
/// parallel to `φ` and the share is scale-free.
///
/// The variance is passed in rather than recomputed so that the quadratic form
/// the blend already published stays the denominator: a second computation of
/// the same quantity by a different route would agree only to rounding, and a
/// share whose denominator is not the variance being tempered is a share of
/// something else.
fn covariance_borrowed_share(params: &ModelParameters, mass: &FloorMass, phi: &[f64], variance: f64) -> f64 {
    if params.p == 0 || phi.len() != params.p || variance <= 0.0 || !variance.is_finite() {
        return 0.0;
    }
    let psi = covariance_matvec(&params.covariance_data, phi, params.p);
    mass.share_along(&psi, variance)
}

/// Computes `Σφ` from a column-major flattened covariance matrix.
///
/// One pass over the columns, accumulating each scaled by its own component of
/// `φ`, at `O(p²)` and one allocation of the model's width. The symmetry the
/// quadratic forms exploit buys nothing here, because every entry of the
/// product depends on a full row.
fn covariance_matvec(sigma_flat: &[f64], phi: &[f64], p: usize) -> Vec<f64> {
    let mut product = vec![0.0_f64; p];
    for (column, &weight) in sigma_flat.chunks_exact(p).zip(phi.iter()) {
        for (slot, &entry) in product.iter_mut().zip(column.iter()) {
            *slot = entry.mul_add(weight, *slot);
        }
    }
    product
}

/// Blends two borrowed shares by the weights the blend gave their variances
/// (´dec:risk:evidence-only-uncertainty´).
///
/// The numerator is the borrowed part of each component variance, weighted as
/// the variances themselves were weighted; the denominator is the whole blended
/// variance, disagreement term included. The disagreement term therefore
/// dilutes the share rather than inheriting one, which is the reading that
/// follows from what that term is: the extra variance from not knowing which
/// of two models is right is uncertainty their evidence produced, and no part
/// of it was lent by a floor.
fn blend_borrowed_share(
    w: f64,
    share_inherited: f64,
    variance_inherited: f64,
    share_anchor: f64,
    variance_anchor: f64,
    variance_effective: f64,
) -> f64 {
    if variance_effective <= 0.0 || !variance_effective.is_finite() {
        return 0.0;
    }
    let borrowed = (1.0 - w).mul_add(share_inherited * variance_inherited, w * share_anchor * variance_anchor);
    crate::model::parameters::borrowed_share(borrowed, variance_effective)
}

/// Projected quadratic form: `φ̃ᵀ Σ[indices, indices] φ̃`.
///
/// Zero-allocation indexed QF using scattered reads from the full p×p
/// covariance matrix. Exploits symmetry: computes diagonal separately,
/// then off-diagonal with 2× multiplier.
///
/// # Arguments
///
/// * `sigma_flat` — Column-major flattened covariance matrix (p × p)
/// * `p` — Dimension of the full covariance matrix
/// * `phi_tilde` — Anchor subvector (dimension `p_a`)
/// * `indices` — Maps each anchor index to full matrix index; `None` skips
///
/// # Returns
///
/// The projected quadratic form value. May be slightly negative due to
/// floating-point error; caller should clamp.
#[must_use]
fn projected_quadratic_form(sigma_flat: &[f64], p: usize, phi_tilde: &[f64], indices: &[Option<usize>]) -> f64 {
    debug_assert_eq!(phi_tilde.len(), indices.len(), "φ̃ and indices length mismatch");
    debug_assert_eq!(sigma_flat.len(), p * p, "σ_flat length mismatch");

    let p_a = phi_tilde.len();
    let mut result = 0.0;

    // Diagonal terms: Σ[i,i] * φ̃[i]²
    for i in 0..p_a {
        if let Some(idx_i) = indices[i] {
            // Column-major: element (idx_i, idx_i) is at idx_i * p + idx_i
            let sigma_ii = sigma_flat[idx_i * p + idx_i];
            result = (sigma_ii * phi_tilde[i]).mul_add(phi_tilde[i], result);
        }
    }

    // Off-diagonal terms: 2 * Σ[i,j] * φ̃[i] * φ̃[j] for i < j
    for i in 0..p_a {
        if let Some(idx_i) = indices[i] {
            for j in (i + 1)..p_a {
                if let Some(idx_j) = indices[j] {
                    // Column-major: element (idx_i, idx_j) is at idx_j * p + idx_i
                    let sigma_ij = sigma_flat[idx_j * p + idx_i];
                    result = (2.0 * sigma_ij * phi_tilde[i]).mul_add(phi_tilde[j], result);
                }
            }
        }
    }

    result
}

/// Quadratic form from column-major flat storage: `vᵀ Σ v`.
///
/// Exploits symmetry: diagonal terms + 2× upper-triangle off-diagonal.
///
/// # Arguments
///
/// * `sigma_flat` — Column-major flattened matrix (p × p)
/// * `v` — Vector (dimension p)
/// * `p` — Matrix dimension
#[must_use]
#[inline]
fn quadratic_form_from_flat(sigma_flat: &[f64], v: &[f64], p: usize) -> f64 {
    debug_assert_eq!(v.len(), p, "vector dimension mismatch");
    debug_assert_eq!(sigma_flat.len(), p * p, "matrix size mismatch");

    let mut result = 0.0;

    // Diagonal: Σ[i,i] * v[i]²
    for i in 0..p {
        let sigma_ii = sigma_flat[i * p + i];
        result = (sigma_ii * v[i]).mul_add(v[i], result);
    }

    // Off-diagonal (upper triangle): 2 * Σ[i,j] * v[i] * v[j] for i < j
    for i in 0..p {
        for j in (i + 1)..p {
            // Column-major: element (i, j) is at j * p + i
            let sigma_ij = sigma_flat[j * p + i];
            result = (2.0 * sigma_ij * v[i]).mul_add(v[j], result);
        }
    }

    result
}

/// Dot product of two vectors.
#[must_use]
#[inline]
fn dot_product(a: &[f64], b: &[f64]) -> f64 {
    debug_assert_eq!(a.len(), b.len(), "dot product dimension mismatch");
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
#[allow(clippy::cast_precision_loss)]
mod tests {
    use super::*;
    use crate::numerics::stable_sigmoid;

    /// Creates identity covariance (column-major flat) of dimension p.
    fn identity_covariance(p: usize) -> Vec<f64> {
        let mut cov = vec![0.0; p * p];
        for i in 0..p {
            cov[i * p + i] = 1.0;
        }
        cov
    }

    /// Creates scaled identity covariance.
    fn scaled_identity_covariance(p: usize, scale: f64) -> Vec<f64> {
        let mut cov = vec![0.0; p * p];
        for i in 0..p {
            cov[i * p + i] = scale;
        }
        cov
    }

    /// Creates a simple non-identity covariance for testing.
    fn non_identity_covariance(p: usize) -> Vec<f64> {
        let mut cov = vec![0.0; p * p];
        for i in 0..p {
            cov[i * p + i] = 0.1f64.mul_add(i as f64, 1.0);
            // Small off-diagonal correlation
            if i + 1 < p {
                cov[(i + 1) * p + i] = 0.05;
                cov[i * p + (i + 1)] = 0.05;
            }
        }
        cov
    }

    /// Helper: build φ vectors from index range scaled by 0.1.
    fn scaled_phi(n: usize) -> Vec<f64> {
        (0..n).map(|i| 0.1 * i as f64).collect()
    }

    /// Helper: the blend of two models whose floors have never acted.
    ///
    /// Every test of the blend's own arithmetic uses it, because a model with
    /// no floor mass borrows nothing and the borrowed share is zero along
    /// every direction — which leaves the weight, the predictors and the
    /// three-term mixture exactly the quantities these tests are about. The
    /// tempering has its own tests and they supply their own masses.
    #[allow(clippy::too_many_arguments)] // Justified: forwards `compute_blend`'s own calling convention unchanged
    fn compute_blend_unfloored(
        sister: &ModelParameters,
        anchor: &ModelParameters,
        phi_hat: &[f64],
        phi_tilde: &[f64],
        anchor_indices: &[Option<usize>],
        time_correction: f64,
        kappa_sister: f64,
        kappa_anchor: f64,
    ) -> BlendResult {
        compute_blend(
            sister,
            anchor,
            &FloorMass::default(),
            &FloorMass::default(),
            phi_hat,
            phi_tilde,
            anchor_indices,
            time_correction,
            kappa_sister,
            kappa_anchor,
        )
    }

    /// The anchor earns weight only by being more certain than the sister is on the
    /// features they share. Two models sitting at the same isotropic prior are
    /// equally uncertain, so the anchor gets essentially none of it and the sister's
    /// own view stands. At cold start, borrowing from a model that knows no more
    /// than you do would add nothing but its bias.
    ///
    /// ´claim:risk:an-anchor-no-more-certain-than-the-sister-earns-no-weight´
    /// ´test:unit:prior-level-blend-weight-near-zero´
    #[test]
    fn prior_level_blend_weight_near_zero() {
        // Both models at isotropic prior → w ≈ 0 (both equally uncertain)
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.0; p],
            covariance_data: identity_covariance(p),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![0.0; p_a],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };

        let phi_hat = scaled_phi(p);
        let phi_tilde = scaled_phi(p_a);
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);

        // Both at identity → σ²_anc ≈ σ²_{inh,S} → w ≈ 0
        assert!(result.anchor_weight < 1e-6, "w = {} should be near 0", result.anchor_weight);
    }

    /// The blend weight is a ratio of the two models' uncertainties, and the
    /// staleness correction multiplies both of them, so it cancels exactly: doubling
    /// the correction leaves the weight bit-for-bit unchanged. Time passing makes
    /// both models less trustworthy, not one of them relatively more so, and the
    /// weight is computed from the raw forms to keep it that way.
    ///
    /// ´claim:risk:the-blend-weight-is-a-ratio-of-uncertainties-so-the-staleness-correction-cancels-out-of-it´
    /// ´test:unit:time-correction-cancels-in-weight´
    #[test]
    fn time_correction_cancels_in_weight() {
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.1; p],
            covariance_data: scaled_identity_covariance(p, 2.0),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![0.2; p_a],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };

        let phi_hat = scaled_phi(p);
        let phi_tilde = scaled_phi(p_a);
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let r1 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);
        let r2 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 2.0, 1.0, 1.5);

        // w should be identical regardless of time_correction
        assert!(
            (r1.anchor_weight - r2.anchor_weight).abs() < 1e-15,
            "w differs: {} vs {}",
            r1.anchor_weight,
            r2.anchor_weight
        );
    }

    /// Where the staleness correction does land is the uncertainty: raising it moves
    /// the blended variance. A model that has not been updated in a while is not
    /// wrong in a knowable direction, so what ages is the confidence attached to its
    /// answer rather than the answer itself.
    ///
    /// ´claim:risk:the-staleness-correction-widens-the-uncertainty-without-moving-the-point-estimate´
    /// ´test:unit:time-correction-affects-variance´
    #[test]
    fn time_correction_affects_variance() {
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.1; p],
            covariance_data: identity_covariance(p),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![0.2; p_a],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };

        let phi_hat = scaled_phi(p);
        let phi_tilde = scaled_phi(p_a);
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let r1 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);
        let r2 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 2.0, 1.0, 1.5);

        // σ²_eff should differ
        assert!(
            (r1.variance_effective - r2.variance_effective).abs() > 1e-10,
            "σ²_eff should differ: {} vs {}",
            r1.variance_effective,
            r2.variance_effective
        );
    }

    /// Reading the sister's uncertainty over just the anchor's features by scattered
    /// indexing gives the same number as physically cutting that submatrix out and
    /// working on it. The projection is a restriction of the full covariance, not an
    /// approximation of one, so the weight it feeds compares like with like.
    ///
    /// ´claim:risk:the-indexed-projection-equals-the-form-computed-from-a-physically-extracted-submatrix´
    /// ´test:unit:projected-qf-matches-submatrix´
    #[test]
    fn projected_qf_matches_submatrix() {
        let p = 20;
        let p_a = 5;

        // Create a non-identity covariance
        let sigma_flat = non_identity_covariance(p);

        let phi_tilde: Vec<f64> = vec![0.1, 0.2, 0.3, 0.4, 0.5];
        let indices: Vec<Option<usize>> = vec![Some(2), Some(5), Some(8), Some(11), Some(14)];

        // Compute via projected_quadratic_form
        let qf_proj = projected_quadratic_form(&sigma_flat, p, &phi_tilde, &indices);

        // Extract 5×5 submatrix manually and compute QF
        let mut submatrix = vec![0.0; p_a * p_a];
        for i in 0..p_a {
            for j in 0..p_a {
                if let (Some(idx_i), Some(idx_j)) = (indices[i], indices[j]) {
                    submatrix[j * p_a + i] = sigma_flat[idx_j * p + idx_i];
                }
            }
        }
        let qf_manual = quadratic_form_from_flat(&submatrix, &phi_tilde, p_a);

        assert!((qf_proj - qf_manual).abs() < 1e-14, "QF mismatch: {qf_proj} vs {qf_manual}");
    }

    /// The three-term mixture stays non-negative across a wide sweep of weights,
    /// component variances and disagreeing means. The cross-term that accounts for
    /// the two models disagreeing only ever adds, so no combination of inputs can
    /// produce a negative variance for a caller to take a square root of.
    ///
    /// ´claim:risk:the-three-term-mixture-variance-is-non-negative-across-the-whole-input-range´
    /// ´test:unit:mixture-variance-non-negative´
    #[test]
    fn mixture_variance_non_negative() {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        for seed in 0..100u64 {
            // Deterministic pseudo-random generation
            let mut hasher = DefaultHasher::new();
            seed.hash(&mut hasher);
            let h = hasher.finish();

            let w = (h % 1000) as f64 / 1000.0;
            let sigma2_inh = ((h >> 10) % 1000) as f64 / 100.0 + 0.01;
            let sigma2_anc = ((h >> 20) % 1000) as f64 / 100.0 + 0.01;
            let rho_inh = ((h >> 30) % 200) as f64 / 100.0 - 1.0;
            let rho_anc = ((h >> 40) % 200) as f64 / 100.0 - 1.0;

            let rho_diff = rho_inh - rho_anc;
            let base_var = (1.0 - w).mul_add(sigma2_inh, w * sigma2_anc);
            let sigma2_eff = (w * (1.0 - w) * rho_diff).mul_add(rho_diff, base_var);

            assert!(sigma2_eff >= 0.0, "σ²_eff < 0 at seed {seed}: {sigma2_eff}");
        }
    }

    /// A covariance that has drifted very slightly indefinite through accumulated
    /// rounding yields a negative quadratic form, and the blend clamps it to zero
    /// before treating it as a variance. The alternative is a negative uncertainty
    /// propagating into the weight ratio and the mixture, where it would be far
    /// harder to recognise as arithmetic noise.
    ///
    /// ´claim:risk:a-quadratic-form-driven-negative-by-rounding-is-clamped-to-zero-before-it-becomes-a-variance´
    /// ´test:unit:qf-clamp-on-negative-eigenvalue´
    #[test]
    fn qf_clamp_on_negative_eigenvalue() {
        let p = 5;

        // Create covariance with slightly negative eigenvalue via perturbation
        let mut sigma_flat = identity_covariance(p);
        // Make (0,0) slightly negative (invalid but possible from numerical error)
        sigma_flat[0] = -1e-10;

        let phi: Vec<f64> = vec![1.0, 0.0, 0.0, 0.0, 0.0];

        // Raw QF would be negative
        let raw_qf = quadratic_form_from_flat(&sigma_flat, &phi, p);
        assert!(raw_qf < 0.0, "raw QF should be negative");

        // In compute_blend, we clamp to 0
        let clamped = raw_qf.max(0.0);
        assert!((clamped - 0.0).abs() < 1e-15, "clamped should be 0");
    }

    /// The calibration constant follows the blend weight through a smooth transition
    /// rather than a switch: at no weight it is the sister's, at full weight the
    /// anchor's, and around the regime boundary it sits near the midpoint. Because
    /// the two regimes are calibrated separately, a hard switch would put a step
    /// discontinuity in the probability an entity is assigned as its weight crosses.
    ///
    /// ´claim:risk:the-blended-calibration-constant-moves-smoothly-from-the-sister-value-to-the-anchor-value´
    /// ´test:unit:kappa-eff-soft-transition´
    #[test]
    fn kappa_eff_soft_transition() {
        let kappa_sister = 1.0;
        let kappa_anchor = 2.0;

        // w = 0 → κ_eff ≈ κ_sister (regime_transition(0) ≈ 0.002)
        let t0 = regime_transition(0.0);
        let kappa_w0 = (1.0 - t0).mul_add(kappa_sister, t0 * kappa_anchor);
        assert!(
            (kappa_w0 - kappa_sister).abs() < 0.01,
            "κ_eff at w=0: {kappa_w0} should be near {kappa_sister}",
        );

        // w = 1 → κ_eff ≈ κ_anchor (regime_transition(1) ≈ 0.9999)
        let t1 = regime_transition(1.0);
        let kappa_w1 = (1.0 - t1).mul_add(kappa_sister, t1 * kappa_anchor);
        assert!(
            (kappa_w1 - kappa_anchor).abs() < 0.01,
            "κ_eff at w=1: {kappa_w1} should be near {kappa_anchor}",
        );

        // w = 0.3 → κ_eff ≈ mean (regime_transition(0.3) ≈ 0.5)
        let t03 = regime_transition(0.3);
        let kappa_w03 = (1.0 - t03).mul_add(kappa_sister, t03 * kappa_anchor);
        let mean = f64::midpoint(kappa_sister, kappa_anchor);
        assert!(
            (kappa_w03 - mean).abs() < 0.01,
            "κ_eff at w=0.3: {kappa_w03} should be near {mean}",
        );
    }

    /// The risk probability is the sigmoid of the blended score divided by the
    /// blended calibration constant, and the division is not optional: at a constant
    /// of two the probability is visibly that of the halved score and demonstrably
    /// not that of the raw one. Skipping the division would return the model's
    /// uncalibrated confidence as though it were a probability.
    ///
    /// ´claim:risk:the-risk-probability-divides-the-blended-score-by-the-blended-calibration-constant-before-the-sigmoid´
    /// ´test:unit:p-bad-uses-kappa-eff´
    #[test]
    fn p_bad_uses_kappa_eff() {
        // With κ_eff=2.0 and ρ̂_eff=1.0, p̂ = σ(0.5) ≈ 0.622, NOT σ(1.0) ≈ 0.731.
        let blend = BlendResult {
            anchor_weight: 0.5,
            rho_inherited: 1.0,
            rho_anchor: 1.0,
            rho_effective: 1.0,
            variance_effective: 0.5,
            variance_inherited: 0.5,
            variance_anchor: 0.5,
            variance_inherited_shared: 0.5,
            kappa_effective: 2.0,
            borrowed_share_effective: 0.0,
        };

        let calibrated = blend.rho_effective / blend.kappa_effective;
        let p_bad = stable_sigmoid(calibrated);

        // σ(0.5) ≈ 0.6225
        assert!(
            (p_bad - stable_sigmoid(0.5)).abs() < 1e-14,
            "p̂ should use κ_eff division: got {p_bad}",
        );
        // NOT σ(1.0) ≈ 0.7311
        assert!(
            (p_bad - stable_sigmoid(1.0)).abs() > 0.1,
            "p̂ should NOT be σ(ρ̂_eff) without κ division",
        );
    }

    /// An extreme score saturates the probability instead of breaking it: a blended
    /// score far outside the ordinary range still yields a finite value, pressed up
    /// against one. An entity the model is overwhelmingly sure about produces an
    /// actionable near-certainty rather than a non-finite value that would poison
    /// every downstream computation.
    ///
    /// ´claim:risk:an-extreme-blended-score-saturates-the-probability-instead-of-overflowing´
    /// ´test:unit:no-overflow-at-large-rho´
    #[test]
    fn no_overflow_at_large_rho() {
        let blend = BlendResult {
            anchor_weight: 0.5,
            rho_inherited: 50.0,
            rho_anchor: 50.0,
            rho_effective: 50.0,
            variance_effective: 0.5,
            variance_inherited: 0.5,
            variance_anchor: 0.5,
            variance_inherited_shared: 0.5,
            kappa_effective: 1.0,
            borrowed_share_effective: 0.0,
        };

        let p_bad = stable_sigmoid(blend.rho_effective / blend.kappa_effective);
        assert!(p_bad.is_finite(), "should not overflow at ρ̂_eff = 50");
        assert!(p_bad > 0.999, "σ(50) should be near 1.0");
    }

    /// The blended score is a genuine convex combination of the two models' own
    /// scores at the weight the blend chose — even when they disagree in sign. It can
    /// therefore never fall outside the interval the two models bracket, so borrowing
    /// from the anchor cannot produce a verdict neither model would endorse.
    ///
    /// ´claim:risk:the-blended-score-is-a-convex-combination-of-the-two-models-own-scores´
    /// ´test:unit:blended-estimate-is-convex´
    #[test]
    fn blended_estimate_is_convex() {
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.5; p],
            covariance_data: identity_covariance(p),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![-0.3; p_a],
            covariance_data: scaled_identity_covariance(p_a, 0.5),
            p: p_a,
        };

        let phi_hat: Vec<f64> = vec![1.0; p];
        let phi_tilde: Vec<f64> = vec![1.0; p_a];
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);

        let w = result.anchor_weight;
        let expected_rho = (1.0 - w).mul_add(result.rho_inherited, w * result.rho_anchor);

        assert!(
            (result.rho_effective - expected_rho).abs() < 1e-14,
            "ρ̂_eff = {} should equal (1-w)ρ̂_inh + w·ρ̂_anc = {expected_rho}",
            result.rho_effective,
        );
    }

    /// Computing the quadratic form as a diagonal plus a doubled upper triangle
    /// reaches the same value as iterating every element of the matrix. Exploiting
    /// the covariance's symmetry halves the work and changes nothing about what the
    /// uncertainty means.
    ///
    /// ´claim:risk:exploiting-symmetry-in-the-quadratic-form-changes-the-cost-and-not-the-value´
    /// ´test:unit:symmetry-optimised-qf-matches-naive´
    #[test]
    fn symmetry_optimised_qf_matches_naive() {
        // Verify the symmetry-exploiting QF gives the same result as naive O(p²).
        let p = 10;
        let sigma_flat = non_identity_covariance(p);
        let v: Vec<f64> = (0..p).map(|i| 0.3f64.mul_add(i as f64, -1.0)).collect();

        let qf_sym = quadratic_form_from_flat(&sigma_flat, &v, p);

        // Naive: iterate all p×p elements
        let mut qf_naive = 0.0;
        for i in 0..p {
            for j in 0..p {
                qf_naive = (v[i] * sigma_flat[j * p + i]).mul_add(v[j], qf_naive);
            }
        }

        assert!(
            (qf_sym - qf_naive).abs() < 1e-12,
            "symmetry QF = {qf_sym}, naive QF = {qf_naive}",
        );
    }

    /// The linear predictor behind every score in this module is an ordinary
    /// elementwise-product sum over the feature vector and the model's means, with no
    /// scaling or reordering hidden inside it.
    ///
    /// ´claim:risk:the-linear-predictor-is-an-ordinary-dot-product-of-features-and-means´
    /// ´test:unit:dot-product-correctness´
    #[test]
    fn dot_product_correctness() {
        let a = vec![1.0, 2.0, 3.0];
        let b = vec![4.0, 5.0, 6.0];
        assert!((dot_product(&a, &b) - 32.0).abs() < 1e-15);
    }

    /// Against an identity covariance the quadratic form reduces to the squared norm
    /// of the feature vector — the known answer for the case where the model has no
    /// correlations and unit variance everywhere. This anchors the column-major
    /// indexing: a transposed or mis-strided read would not land on this value.
    ///
    /// ´claim:risk:a-quadratic-form-against-the-identity-covariance-is-the-squared-norm-of-the-vector´
    /// ´test:unit:quadratic-form-identity-gives-norm-squared´
    #[test]
    fn quadratic_form_identity_gives_norm_squared() {
        let p = 5;
        let sigma_flat = identity_covariance(p);
        let v: Vec<f64> = vec![1.0, 2.0, 3.0, 4.0, 5.0];

        let qf = quadratic_form_from_flat(&sigma_flat, &v, p);
        let norm_sq: f64 = v.iter().map(|x| x * x).sum();

        assert!((qf - norm_sq).abs() < 1e-14, "QF on I should equal ‖v‖²");
    }

    /// The component variances reported alongside the blend are real quantities and
    /// not decoration: each is non-negative, they recombine into the blended variance
    /// through the same three-term formula the blend used, and the sister's variance
    /// restricted to the shared subspace never exceeds its variance over all features.
    /// A reader can therefore reconstruct why the weight came out where it did instead
    /// of taking the number on trust.
    ///
    /// ´claim:risk:the-reported-component-variances-recompose-into-the-blended-variance-and-respect-the-subspace-ordering´
    /// ´test:unit:variance-diagnostics-populated´
    #[test]
    fn variance_diagnostics_populated() {
        // Verify that the three diagnostic variance fields are correctly
        // populated and consistent with the blended variance.
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.3; p],
            covariance_data: scaled_identity_covariance(p, 2.0),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![-0.1; p_a],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };

        let phi_hat = scaled_phi(p);
        let phi_tilde = scaled_phi(p_a);
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.5, 1.0, 1.5);

        // All diagnostic variances must be non-negative
        assert!(result.variance_inherited >= 0.0, "σ²_inh must be non-negative");
        assert!(result.variance_anchor >= 0.0, "σ²_anc must be non-negative");
        assert!(
            result.variance_inherited_shared >= 0.0,
            "projected sister variance must be non-negative"
        );

        // With time_correction > 1, diagnostic variances must be scaled up from raw
        assert!(result.variance_inherited > 0.0, "σ²_inh must be positive with non-zero φ̂");

        // The three-term mixture should satisfy:
        // σ²_eff = (1-w)σ²_inh + w·σ²_anc + w(1-w)(ρ̂_inh − ρ̂_anc)²
        let w = result.anchor_weight;
        let rho_diff = result.rho_inherited - result.rho_anchor;
        let expected_eff = (w * (1.0 - w) * rho_diff).mul_add(
            rho_diff,
            (1.0 - w).mul_add(result.variance_inherited, w * result.variance_anchor),
        );
        assert!(
            (result.variance_effective - expected_eff).abs() < 1e-12,
            "σ²_eff = {} should match three-term formula = {expected_eff}",
            result.variance_effective
        );

        // Projected sister variance ≤ full sister variance (subspace restriction)
        assert!(
            result.variance_inherited_shared <= result.variance_inherited + 1e-14,
            "σ²_{{inh,S}} = {} should not exceed σ²_inh = {}",
            result.variance_inherited_shared,
            result.variance_inherited
        );
    }

    /// The converse of an anchor that earns nothing: an anchor orders of magnitude
    /// more precise than the sister over the shared features takes nearly all the
    /// weight, and the blended score lands next to the anchor's own. A sister that
    /// has learned almost nothing yet defers to the model that has, which is what
    /// makes the population-level anchor useful during a cold start.
    ///
    /// ´claim:risk:a-far-more-precise-anchor-takes-nearly-all-the-weight-and-the-blended-score-follows-it´
    /// ´test:unit:anchor-only-w-near-one´
    #[test]
    fn anchor_only_w_near_one() {
        // Sister has very high uncertainty in shared subspace, anchor has low → w ≈ 1
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.5; p],
            covariance_data: scaled_identity_covariance(p, 100.0),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![-0.3; p_a],
            covariance_data: scaled_identity_covariance(p_a, 0.01),
            p: p_a,
        };

        let phi_hat: Vec<f64> = vec![1.0; p];
        let phi_tilde: Vec<f64> = vec![1.0; p_a];
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);

        // σ²_anc = 0.01 * 15 = 0.15, σ²_{inh,S} = 100 * 15 = 1500
        // w = (1 - 0.15/1500.0) ≈ 0.9999
        assert!(
            result.anchor_weight > 0.99,
            "w = {} should be near 1 when anchor is much more precise",
            result.anchor_weight,
        );

        // ρ̂_eff should be close to ρ̂_anc
        assert!(
            (result.rho_effective - result.rho_anchor).abs() < 0.02,
            "ρ̂_eff = {} should be near ρ̂_anc = {} at w ≈ 1",
            result.rho_effective,
            result.rho_anchor,
        );
    }

    /// In the degenerate single-feature case the machinery reduces to its closed
    /// form exactly: the weight is one minus the ratio of the two scalar variances,
    /// and the blended score is that weight applied to the two scalar means. The
    /// general matrix code is a generalisation of the scalar arithmetic and not a
    /// separate approximation that happens to agree in the usual dimensions.
    ///
    /// ´claim:risk:the-blend-reduces-exactly-to-its-scalar-closed-form-in-one-dimension´
    /// ´test:unit:dimension-1-scalar-case´
    #[test]
    fn dimension_1_scalar_case() {
        // Degenerate: p = 1, p_a = 1 → closed-form w = max(0, 1 - Σ_anc/(Σ_inh + ε))
        let sister = ModelParameters {
            mu: vec![2.0],
            covariance_data: vec![4.0], // Σ_inh = 4
            p: 1,
        };
        let anchor = ModelParameters {
            mu: vec![1.0],
            covariance_data: vec![1.0], // Σ_anc = 1
            p: 1,
        };

        let phi_hat = vec![1.0];
        let phi_tilde = vec![1.0];
        let indices = vec![Some(0)];

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.0);

        // w = clamp(1 - 1.0 / (4.0 + 1e-12), 0, 1) = 0.75
        let expected_w = (1.0 - 1.0 / (4.0 + EPSILON_BLEND)).clamp(0.0, 1.0);
        assert!(
            (result.anchor_weight - expected_w).abs() < 1e-14,
            "w = {} should match closed-form {expected_w}",
            result.anchor_weight,
        );

        // ρ̂_eff = (1-0.75)*2.0 + 0.75*1.0 = 0.5 + 0.75 = 1.25
        let expected_rho = (1.0 - expected_w).mul_add(2.0, expected_w * 1.0);
        assert!(
            (result.rho_effective - expected_rho).abs() < 1e-14,
            "ρ̂_eff = {} should be {expected_rho}",
            result.rho_effective,
        );
    }

    /// The blend publishes the sister's own score separately from the blended one so
    /// a caller can measure intervention effectiveness against it: the gap between
    /// what the inherited model expects of an entity and what the operational model
    /// expects after intervention. Only a score computed from unintervened history
    /// makes that difference meaningful, which is why the inherited score survives
    /// the blend rather than being folded away into it.
    ///
    /// ´claim:risk:intervention-effectiveness-is-the-gap-between-the-inherited-score-and-the-operational-one´
    /// ´test:unit:intervention-effectiveness-at-call-site´
    #[test]
    fn intervention_effectiveness_at_call_site() {
        // Verify Δ = ρ̂_inh − ρ̂_opr as done at the assessment call site
        let p = 5;
        let p_a = 3;

        let sister = ModelParameters {
            mu: vec![0.5, 0.3, -0.2, 0.1, 0.4],
            covariance_data: identity_covariance(p),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![0.1, 0.2, -0.1],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };
        let operational_mu = [0.3, 0.1, -0.1, 0.05, 0.2];

        let phi_hat = vec![1.0, 1.0, 1.0, 1.0, 1.0];
        let phi_tilde = vec![1.0, 1.0, 1.0];
        let indices: Vec<Option<usize>> = vec![Some(0), Some(1), Some(2)];

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.0);

        // Operational dot product (as computed at call site)
        let rho_opr: f64 = phi_hat.iter().zip(operational_mu.iter()).map(|(a, b)| a * b).sum();
        let intervention_effectiveness = result.rho_inherited - rho_opr;

        // Manual: ρ̂_inh = 0.5+0.3-0.2+0.1+0.4 = 1.1, ρ̂_opr = 0.3+0.1-0.1+0.05+0.2 = 0.55
        let expected_rho_inh: f64 = sister.mu.iter().zip(phi_hat.iter()).map(|(a, b)| a * b).sum();
        assert!((result.rho_inherited - expected_rho_inh).abs() < 1e-14, "ρ̂_inh mismatch");
        assert!(
            (intervention_effectiveness - (expected_rho_inh - rho_opr)).abs() < 1e-14,
            "Δρ̂ = {} should be ρ̂_inh - ρ̂_opr = {}",
            intervention_effectiveness,
            expected_rho_inh - rho_opr,
        );
    }

    /// The other half of the same truth: both the sister's score and the blended one
    /// come out identical under different staleness corrections. Point estimates are
    /// read off the means, which the correction never touches, so ageing can never
    /// silently push an entity across a decision boundary.
    ///
    /// (´claim:risk:the-staleness-correction-widens-the-uncertainty-without-moving-the-point-estimate´)
    /// ´test:unit:time-correction-does-not-affect-rho´
    #[test]
    fn time_correction_does_not_affect_rho() {
        let p = 20;
        let p_a = 15;

        let sister = ModelParameters {
            mu: vec![0.1; p],
            covariance_data: scaled_identity_covariance(p, 2.0),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![0.2; p_a],
            covariance_data: identity_covariance(p_a),
            p: p_a,
        };

        let phi_hat = scaled_phi(p);
        let phi_tilde = scaled_phi(p_a);
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let r1 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);
        let r2 = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 2.0, 1.0, 1.5);

        // ρ̂_eff should be identical (point estimates are not time-corrected)
        assert!(
            (r1.rho_effective - r2.rho_effective).abs() < 1e-15,
            "ρ̂_eff differs: {} vs {}",
            r1.rho_effective,
            r2.rho_effective,
        );
        assert!(
            (r1.rho_inherited - r2.rho_inherited).abs() < 1e-15,
            "ρ̂_inh differs: {} vs {}",
            r1.rho_inherited,
            r2.rho_inherited,
        );
    }

    /// Run end to end on small fixed matrices, every published quantity — both
    /// quadratic forms, the weight, both linear predictors and the blend of them —
    /// matches values worked out by hand. This is the pin that catches a refactor
    /// which keeps all the internal identities consistent while quietly changing what
    /// the blend computes.
    ///
    /// ´claim:risk:the-whole-blend-reproduces-hand-computed-values-from-fixed-inputs´
    /// ´test:unit:golden-vector-known-output´
    #[test]
    fn golden_vector_known_output() {
        // Golden test: p=3, p_a=2, known matrices, pre-computed expected values.
        let sister = ModelParameters {
            mu: vec![1.0, -0.5, 0.3],
            covariance_data: vec![
                // Column-major 3×3:
                // [[2.0, 0.1, 0.0],
                //  [0.1, 3.0, 0.2],
                //  [0.0, 0.2, 1.5]]
                2.0, 0.1, 0.0, // col 0
                0.1, 3.0, 0.2, // col 1
                0.0, 0.2, 1.5, // col 2
            ],
            p: 3,
        };
        let anchor = ModelParameters {
            mu: vec![0.5, -0.2],
            covariance_data: vec![
                // Column-major 2×2:
                // [[0.5, 0.05],
                //  [0.05, 0.8]]
                0.5, 0.05, // col 0
                0.05, 0.8, // col 1
            ],
            p: 2,
        };

        let phi_hat = vec![1.0, 0.5, -0.3];
        let phi_tilde = vec![1.0, 0.5];
        let indices = vec![Some(0), Some(1)];

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.2, 1.8);

        // Manual computation:
        // σ²_anc = [1, 0.5] [[0.5,0.05],[0.05,0.8]] [1, 0.5]ᵀ
        //        = 0.5*1 + 2*0.05*1*0.5 + 0.8*0.25 = 0.5 + 0.05 + 0.2 = 0.75
        let expected_sigma2_anc = 0.75;

        // σ²_{inh,S} = submatrix of sister at indices [0,1] = [[2.0,0.1],[0.1,3.0]]
        // [1, 0.5] [[2,0.1],[0.1,3]] [1,0.5]ᵀ = 2.0 + 2*0.1*0.5 + 3.0*0.25
        //   = 2.0 + 0.1 + 0.75 = 2.85
        let expected_sigma2_inh_s = 2.85;

        // w = clamp(1 - 0.75/(2.85 + ε), 0, 1)
        let expected_w = (1.0 - expected_sigma2_anc / (expected_sigma2_inh_s + EPSILON_BLEND)).clamp(0.0, 1.0);

        // ρ̂_inh = [1, 0.5, -0.3]·[1, -0.5, 0.3] = 1.0 - 0.25 - 0.09 = 0.66
        // ρ̂_anc = [1, 0.5]·[0.5, -0.2] = 0.5 - 0.1 = 0.4
        let expected_rho_inh = (-0.3f64).mul_add(0.3, 0.5f64.mul_add(-0.5, 1.0 * 1.0));
        let expected_rho_anc = 0.5f64.mul_add(-0.2, 1.0 * 0.5);
        let expected_rho_eff = (1.0 - expected_w).mul_add(expected_rho_inh, expected_w * expected_rho_anc);

        assert!(
            (result.anchor_weight - expected_w).abs() < 1e-14,
            "w: {} vs expected {expected_w}",
            result.anchor_weight,
        );
        assert!(
            (result.rho_inherited - expected_rho_inh).abs() < 1e-14,
            "ρ̂_inh: {} vs expected {expected_rho_inh}",
            result.rho_inherited,
        );
        assert!(
            (result.rho_anchor - expected_rho_anc).abs() < 1e-14,
            "ρ̂_anc: {} vs expected {expected_rho_anc}",
            result.rho_anchor,
        );
        assert!(
            (result.rho_effective - expected_rho_eff).abs() < 1e-14,
            "ρ̂_eff: {} vs expected {expected_rho_eff}",
            result.rho_effective,
        );
        assert!(
            (result.variance_anchor - expected_sigma2_anc).abs() < 1e-14,
            "σ²_anc: {} vs expected {expected_sigma2_anc}",
            result.variance_anchor,
        );
        assert!(
            (result.variance_inherited_shared - expected_sigma2_inh_s).abs() < 1e-14,
            "σ²_{{inh,S}}: {} vs expected {expected_sigma2_inh_s}",
            result.variance_inherited_shared,
        );
    }

    /// The blended variance is exactly the variance of the two-component mixture the
    /// blend represents, recovered independently from the moment identity in a regime
    /// where the weight is strictly interior and the two models disagree. The blend
    /// is therefore not two models averaged with an uncertainty bolted on: the
    /// disagreement between them is itself part of the uncertainty reported, which is
    /// the only honest answer when the models are pulling in different directions.
    ///
    /// ´claim:risk:the-blended-variance-is-exactly-the-variance-of-the-two-component-mixture´
    /// ´test:unit:mixture-variance-identity´
    #[test]
    fn mixture_variance_identity() {
        // The law of total variance for the two-component blend.
        //
        // The three-term formula in `compute_blend` is:
        //   σ²_eff = (1−w)·σ²_inh + w·σ²_anc + w(1−w)·(ρ̂_inh − ρ̂_anc)²
        //
        // Classical mixture-variance identity:
        //   For a two-component mixture Z with mixing weights (1−w, w)
        //   drawing from N(ρ̂_inh, σ²_inh) and N(ρ̂_anc, σ²_anc):
        //     E[Z]   = (1−w)·ρ̂_inh + w·ρ̂_anc
        //     E[Z²]  = (1−w)·(σ²_inh + ρ̂_inh²) + w·(σ²_anc + ρ̂_anc²)
        //     Var[Z] = E[Z²] − E[Z]²
        //
        // We verify σ²_eff from `compute_blend` matches Var[Z] exactly.
        //
        // Construction is chosen so the blend weight `w` is strictly interior
        // to (0, 1) and the two means differ — the only regime where the
        // third cross-term is non-trivial.
        let p = 4;
        let p_a = 3;

        let sister = ModelParameters {
            mu: vec![0.7, -0.2, 0.1, 0.4],
            covariance_data: scaled_identity_covariance(p, 2.0),
            p,
        };
        let anchor = ModelParameters {
            mu: vec![-0.3, 0.5, 0.2],
            covariance_data: scaled_identity_covariance(p_a, 0.5),
            p: p_a,
        };

        let phi_hat = vec![0.5, 0.3, -0.2, 0.1];
        let phi_tilde = vec![0.5, 0.3, -0.2];
        let indices: Vec<Option<usize>> = (0..p_a).map(Some).collect();

        let result = compute_blend_unfloored(&sister, &anchor, &phi_hat, &phi_tilde, &indices, 1.0, 1.0, 1.5);

        // Sanity: we want a genuine mixture (w strictly in (0,1)) with
        // disagreeing means so the w(1−w)·Δρ² cross-term is non-zero.
        assert!(result.anchor_weight > 1e-3 && result.anchor_weight < 1.0 - 1e-3);
        let rho_diff = (result.rho_inherited - result.rho_anchor).abs();
        assert!(rho_diff > 0.1, "need disagreeing means for a meaningful identity");

        // Classical mixture variance from the moment identity
        //   Var[Z] = E[Z²] − E[Z]².
        let w = result.anchor_weight;
        let e_z = (1.0 - w).mul_add(result.rho_inherited, w * result.rho_anchor);
        let e_z2 = (1.0 - w).mul_add(
            result.rho_inherited.mul_add(result.rho_inherited, result.variance_inherited),
            w * result.rho_anchor.mul_add(result.rho_anchor, result.variance_anchor),
        );
        let var_z = e_z.mul_add(-e_z, e_z2);

        assert!(
            (var_z - result.variance_effective).abs() < 1e-12,
            "σ²_eff={} must equal mixture Var[Z]={} (law of total variance)",
            result.variance_effective,
            var_z,
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // The borrowed share (´dec:risk:evidence-only-uncertainty´)
    // ─────────────────────────────────────────────────────────────────────

    /// Helper: a model whose replenishment clamp has bound on every coordinate,
    /// whose evidence has since sharpened one of them, and whose covariance has
    /// been rebuilt from the precision matrix it holds.
    ///
    /// The rebuild is what makes the pair inverses of one another again, which
    /// is the state every reading through the covariance assumes; the clamp is
    /// what gives the model something to have borrowed.
    fn clamped_and_rebuilt(
        p: usize,
        prior: f64,
        binding_floor: f64,
        sharpening: f64,
    ) -> crate::model::bayesian::BayesianLinearModel {
        use crate::linalg::convert::vec_to_col;
        use crate::model::bayesian::BayesianLinearModel;
        use crate::model::recompute::{
            RecomputeBaseline, VisitInputs, measure_synchronisation, rebuild_covariance, synchronisation_error_threshold,
        };

        let mut model = BayesianLinearModel::new(p, prior, 1_000_000);

        // One label at unit decay: the clamp lifts every coordinate to the
        // floor, then the rank-one term sharpens the first coordinate alone.
        let mut phi = vec![0.0; p];
        phi[0] = 1.0;
        let phi_hat = vec_to_col(&phi);
        let (v, h) = model.covariance().quadratic_form_with_product(&phi_hat);
        model.apply_leverage_bounded_update(&phi_hat, &v, h, sharpening, 0.0, 1.0, binding_floor);

        // Rebuild, so that the covariance is the precision matrix's inverse.
        // The rebuild is asked for directly rather than through a visit: what
        // this helper needs is the pair, and a visit would measure first and
        // decline to rebuild a model whose drift is already under its
        // threshold (´dec:posterior:recomputation-trigger´).
        let inputs = VisitInputs {
            labels_since_recompute: 1,
            diagonal_ratio: 1.0,
            dimensions_at_floor: 0,
            sync_error_threshold: synchronisation_error_threshold(p),
            spectral_floor_mass: model.spectral_floor_mass(),
            clamp_mass: model.clamp_mass_since_rebuild(),
        };
        let measurement = measure_synchronisation(model.precision(), model.covariance(), &inputs);
        let (outcome, rebuilt, record) = rebuild_covariance(model.precision(), &measurement, &inputs);
        let baseline = RecomputeBaseline::from_visit(&measurement, &inputs, record.wall_nanos, Some(record));
        model.apply_recompute_outcome(&outcome, rebuilt, Some(baseline), 1_000_000);
        model
    }

    /// Helper: the published projection of a model, parameters and masses.
    fn published(model: &crate::model::bayesian::BayesianLinearModel) -> (ModelParameters, FloorMass) {
        (
            model.to_parameters(),
            FloorMass::new(model.spectral_floor_mass(), model.clamp_mass().to_vec()),
        )
    }

    /// A model whose floors have never acted has borrowed nothing, and the
    /// reading says so along every direction rather than along the ones it was
    /// asked about. A fresh model's precision is its construction prior alone:
    /// no rebuild has applied a spectral floor and no clamp has bound, so both
    /// tracked masses are zero and the numerator of the share is zero whatever
    /// the direction. This is the property that keeps the whole mechanism off
    /// the path of a healthy deployment: a widening that fired a little on
    /// every verdict would be a systematic inflation of every uncertainty the
    /// package reports, which is exactly what a calibration refit would then
    /// spend its evidence undoing.
    ///
    /// ´claim:risk:a-model-whose-floors-have-never-acted-reports-a-borrowed-share-of-zero-along-every-direction´
    /// ´test:unit:a-model-whose-floors-have-never-acted-borrows-nothing´
    #[test]
    fn a_model_whose_floors_have_never_acted_borrows_nothing() {
        use crate::model::bayesian::BayesianLinearModel;

        let p = 6;
        let model = BayesianLinearModel::new(p, 0.1, 1_000_000);
        let (params, mass) = published(&model);
        assert!(mass.spectral.abs() < f64::EPSILON, "no rebuild has applied a floor");
        assert!(mass.clamp.iter().all(|&c| c.abs() < f64::EPSILON), "no clamp has bound");

        for index in 0..p {
            let mut axis = vec![0.0; p];
            axis[index] = 1.0;
            let variance = quadratic_form_from_flat(&params.covariance_data, &axis, p);
            assert!(
                covariance_borrowed_share(&params, &mass, &axis, variance).abs() < f64::EPSILON,
                "coordinate {index} borrowed nothing"
            );
        }

        let oblique = vec![0.4, -1.3, 0.0, 2.2, 0.7, -0.1];
        let variance = quadratic_form_from_flat(&params.covariance_data, &oblique, p);
        assert!(
            covariance_borrowed_share(&params, &mass, &oblique, variance).abs() < f64::EPSILON,
            "and nothing along an oblique direction either"
        );

        // The blend of two such models borrows nothing, and its variance is
        // therefore the one the mixture computed and not a widened copy of it.
        let anchor = BayesianLinearModel::new(p, 0.1, 1_000_000);
        let (anchor_params, anchor_mass) = published(&anchor);
        let indices: Vec<Option<usize>> = (0..p).map(Some).collect();
        let result = compute_blend(
            &params,
            &anchor_params,
            &mass,
            &anchor_mass,
            &oblique,
            &oblique,
            &indices,
            1.0,
            1.0,
            1.0,
        );
        assert!(
            result.borrowed_share_effective.abs() < f64::EPSILON,
            "the blend of two unfloored models borrows nothing: {}",
            result.borrowed_share_effective
        );
    }

    /// The share has an angle, and so does the widening it drives. A model
    /// that has clamped every coordinate and then learned about one of them
    /// has borrowed a small part of the precision along the direction it
    /// learned and a large part of the precision along the directions it did
    /// not, so turning the direction of interest from the first toward the
    /// second raises the share, and raises it monotonically rather than at
    /// some threshold. The variance the tempering reports rises with it over
    /// the same sweep, which is the end-to-end statement the ruling makes:
    /// two requests against one model, differing only in which direction they
    /// ask about, receive different uncertainties. Degradation is not equally
    /// applicable to every verdict, and which verdicts it applies to is
    /// decided by where the model's evidence actually is, not by an aggregate
    /// that has averaged that question away.
    ///
    /// The share stays strictly below one throughout, and that is a property
    /// of the engine rather than of this sweep. The prior precision a model is
    /// constructed at is not tracked as floor mass and so is not counted in
    /// the numerator, so a direction the floors hold up entirely is a limit
    /// the arithmetic must handle rather than a state a live model reaches.
    /// The saturation is tested against that limit where it can be reached,
    /// which is on the tempering itself.
    ///
    /// ´claim:risk:the-borrowed-share-rises-monotonically-as-the-direction-turns-from-a-well-evidenced-coordinate-toward-a-clamped-one´
    /// ´test:unit:the-borrowed-share-rises-as-the-direction-turns´
    #[test]
    fn the_borrowed_share_rises_as_the_direction_turns() {
        let p = 4;
        let model = clamped_and_rebuilt(p, 1.0, 2.0, 6.0);
        let (params, mass) = published(&model);
        assert!(mass.clamp.iter().all(|&c| c > 0.0), "the clamp bound on every coordinate");

        let steps = 24;
        let mut previous = f64::NEG_INFINITY;
        let mut previous_tempered = f64::NEG_INFINITY;
        let mut shares = Vec::with_capacity(steps + 1);
        for step in 0..=steps {
            let theta = std::f64::consts::FRAC_PI_2 * (step as f64) / (steps as f64);
            let mut direction = vec![0.0; p];
            direction[0] = theta.cos();
            direction[1] = theta.sin();
            let variance = quadratic_form_from_flat(&params.covariance_data, &direction, p);
            let share = covariance_borrowed_share(&params, &mass, &direction, variance);
            assert!(share > previous, "the share rose at step {step}: {share} against {previous}");
            assert!(
                share < 1.0,
                "and stayed below one, because the construction prior is not tracked as floor mass: {share}"
            );

            // The end-to-end reading: the variance the tempering reports rises
            // over the same sweep, so the widening has the angle the share has
            // (´dec:risk:evidence-only-uncertainty´).
            let tempered = variance / (1.0 - share);
            assert!(
                tempered > previous_tempered,
                "the tempered variance rose at step {step}: {tempered} against {previous_tempered}"
            );
            assert!(
                tempered > variance,
                "and it widens the posterior variance rather than replacing it: {tempered} against {variance}"
            );
            previous = share;
            previous_tempered = tempered;
            shares.push(share);
        }

        let first = shares[0];
        let last = shares[steps];
        assert!(
            last > first * 2.0,
            "and the two ends are far apart rather than nearly equal: {last} against {first}"
        );
    }

    /// Two models that have borrowed differently blend their shares by the same
    /// weights the blend gave their variances, and the disagreement between
    /// them dilutes the result rather than inheriting a share of its own. The
    /// three-term mixture's last term is the extra variance from not knowing
    /// which model is right; that is uncertainty their evidence produced, and
    /// crediting a floor with it would widen a verdict twice for one cause.
    /// The identity this test checks is the definition read back off the
    /// result: borrowed variance over total variance, with the components
    /// weighted as the blend weighted them.
    ///
    /// ´claim:risk:the-blended-borrowed-share-is-the-weighted-borrowed-part-of-each-component-variance-over-the-whole-blended-variance´
    /// ´test:unit:the-borrowed-share-blends-by-the-weights-the-variances-take´
    #[test]
    fn the_borrowed_share_blends_by_the_weights_the_variances_take() {
        let p = 4;
        let sister_model = clamped_and_rebuilt(p, 1.0, 2.0, 6.0);
        let anchor_model = clamped_and_rebuilt(p, 0.5, 3.0, 40.0);
        let (sister, sister_mass) = published(&sister_model);
        let (anchor, anchor_mass) = published(&anchor_model);

        // A direction that leans on the coordinate the two models learned
        // differently about, so their shares along it genuinely differ.
        let phi = vec![0.3, 1.0, 0.2, -0.4];
        let indices: Vec<Option<usize>> = (0..p).map(Some).collect();
        let time_correction = 1.7;
        let result = compute_blend(
            &sister,
            &anchor,
            &sister_mass,
            &anchor_mass,
            &phi,
            &phi,
            &indices,
            time_correction,
            1.0,
            1.0,
        );

        let raw_inherited = quadratic_form_from_flat(&sister.covariance_data, &phi, p);
        let raw_anchor = quadratic_form_from_flat(&anchor.covariance_data, &phi, p);
        let share_inherited = covariance_borrowed_share(&sister, &sister_mass, &phi, raw_inherited);
        let share_anchor = covariance_borrowed_share(&anchor, &anchor_mass, &phi, raw_anchor);
        assert!(
            (share_inherited - share_anchor).abs() > 1e-6,
            "the two models must disagree in share for the blend to be testable: {share_inherited} against {share_anchor}"
        );

        let w = result.anchor_weight;
        let borrowed = (1.0 - w).mul_add(
            share_inherited * result.variance_inherited,
            w * share_anchor * result.variance_anchor,
        );
        let expected = borrowed / result.variance_effective;
        assert!(
            (result.borrowed_share_effective - expected).abs() < 1e-14,
            "the blended share is the weighted borrowed variance over the whole: {} against {expected}",
            result.borrowed_share_effective
        );

        // The disagreement term dilutes rather than inherits: the blended share
        // is strictly below the weighted mean of the two shares wherever the
        // two models disagree in their point estimates.
        let weighted_mean_of_shares = (1.0 - w).mul_add(share_inherited, w * share_anchor);
        if (result.rho_inherited - result.rho_anchor).abs() > 1e-9 && w > 1e-9 && w < 1.0 - 1e-9 {
            assert!(
                result.borrowed_share_effective < weighted_mean_of_shares,
                "the disagreement term carries no borrowed share: {} against {weighted_mean_of_shares}",
                result.borrowed_share_effective
            );
        }
    }

    /// The share the assessment path reads from the published covariance is the
    /// share the model itself would report, because the two are one function
    /// evaluated along one direction. The definition divides the floors'
    /// contribution to the precision along a direction by the precision along
    /// it, and needs the precision matrix, which is not published. The identity
    /// `(Σφ)ᵀB(Σφ) = φᵀΣφ` supplies the denominator from the covariance alone,
    /// and `Σφ` is the direction whose precision holds that variance up — so
    /// the published reading is the definition evaluated at `Σφ`, and this test
    /// asserts exactly that by asking the model for its own reading there.
    ///
    /// The model whose two readings are compared has been rebuilt, because the
    /// identity is an identity only where the pair the model holds are actually
    /// inverses; between rebuilds they disagree by the clamp's own accumulated
    /// contribution, which the drift monitor reads and which no reading through
    /// the covariance can see.
    ///
    /// ´claim:risk:the-share-read-through-the-covariance-is-the-definition-evaluated-along-the-direction-the-covariance-maps-to´
    /// ´test:unit:the-covariance-reading-is-the-definition-at-sigma-phi´
    #[test]
    fn the_covariance_reading_is_the_definition_at_sigma_phi() {
        let p = 4;
        let model = clamped_and_rebuilt(p, 1.0, 2.0, 6.0);
        let (params, mass) = published(&model);
        assert!(mass.clamp.iter().any(|&c| c > 0.0), "the model has borrowed something");

        for phi in [
            vec![1.0, 0.0, 0.0, 0.0],
            vec![0.0, 1.0, 0.0, 0.0],
            vec![0.6, 0.8, 0.0, 0.0],
            vec![0.3, -1.1, 2.0, 0.5],
        ] {
            let variance = quadratic_form_from_flat(&params.covariance_data, &phi, p);
            let through_covariance = covariance_borrowed_share(&params, &mass, &phi, variance);
            let psi = covariance_matvec(&params.covariance_data, &phi, p);
            let from_the_model = model.borrowed_share(&psi);
            assert!(
                (through_covariance - from_the_model).abs() < 1e-9,
                "the two readings agree along {phi:?}: {through_covariance} against {from_the_model}"
            );
            assert!(through_covariance > 0.0, "and the reading is not trivially zero");
        }
    }
}
