// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Dynamic-dimension Bayesian linear model.
//!
//! `BayesianLinearModel` is the core inference type. It maintains the
//! conjugate normal posterior in precision form (B) and covariance form
//! (Σ) simultaneously, enabling O(p²) rank-1 updates on both.
//!
//! # Cross-References
//!
//! - (´dec:substrate:dense-dynamic´) — the dynamic-dimension model type
//! - (´dec:retention:precision-excluded´) — snapshot composition and the
//!   B-exclusion
//! - (´dec:posterior:recomputation-trigger´) — what the Cholesky
//!   recomputation tracking counts towards
//! - (´dec:posterior:half-solve´) — the Schur complement marginalisation

use std::fmt;

use faer::{Col, Mat};

use super::marginalise::{
    ConditionMeasure, SchurConfig, SchurCorrectionOutcome, SchurDiagnostics, VerificationCertificate, compute_keep_indices,
    compute_schur_complement, discarded_correction_error, rank1_marginalisation_error, verify_rank1_positive_definite,
};
use super::parameters::ModelParameters;
use crate::linalg::bridge::{CholeskyCallSite, CholeskyError, CholeskyInverseResult, cholesky_inverse, extract_subvector};
use crate::linalg::convert::{col_to_vec, mat_to_vec, vec_to_col, vec_to_mat};
use crate::linalg::symmetric::SymmetricMatrix;
use crate::types::ModelId;

// ═══════════════════════════════════════════════════════════════════════════════
// Error Types
// ═══════════════════════════════════════════════════════════════════════════════

// Marginalisation carries no error type. A factorisation failure inside
// it retains and flags (´dec:posterior:cascade-never-fails´),
// (´dec:degradation:retain-and-flag´), and the outcome travels in
// [`SchurDiagnostics`] rather than in a `Result`.

/// What one label's leverage-bounded update did to a model.
///
/// The refusal is not an error and does not travel in a `Result`: the
/// package's numerical posture is that a pathology is retained and flagged
/// rather than failed (´dec:degradation:retain-and-flag´), and a refused
/// update has left the model exactly as it found it. What the caller owes the
/// outcome is a report on the health surface, not a propagated failure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum LabelUpdateOutcome {
    /// The rank-one update was applied.
    Applied,
    /// The update was refused because the leverage read against the model's
    /// covariance was negative or not a number, which says the covariance is
    /// no longer positive semi-definite
    /// (´req:gaussian:positive-definiteness´). Nothing was mutated.
    RefusedIndefiniteCovariance,
}

/// Error from parameter restoration (snapshot → model reconstruction).
///
/// Wraps `CholeskyError` from the B = Σ⁻¹ reconstruction step, and carries
/// the pivot count that separates the two ways a restore is refused: a
/// factorisation that produced nothing, and one that produced an answer only
/// by replacing pivots the arithmetic would not accept. Both are refusals on
/// this path, and the count is what tells a reader which of the two it read
/// (´dec:posterior:cascade-never-fails´).
#[derive(Debug)]
pub struct RevertError {
    /// The underlying Cholesky failure.
    pub source: CholeskyError,
    /// Dimension of the model being reconstructed.
    pub p: usize,
}

impl fmt::Display for RevertError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "parameter restoration refused (p={}): {}", self.p, self.source)
    }
}

impl std::error::Error for RevertError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.source)
    }
}

/// Computes `B⁻¹` for a marginal precision matrix.
///
/// Returns the inverse, or the pivot at which the factorisation was refused —
/// the caller retains and flags rather than erroring
/// (´dec:posterior:infallible-correction´).
fn cholesky_inverse_for_marginalisation(precision: &SymmetricMatrix) -> Result<SymmetricMatrix, usize> {
    match cholesky_inverse(precision, CholeskyCallSite::Marginalisation) {
        CholeskyInverseResult::Clean(inv) => Ok(inv),
        CholeskyInverseResult::Failed { pivot } => Err(pivot),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BayesianLinearModel
// ═══════════════════════════════════════════════════════════════════════════════

/// Dynamic-dimension Bayesian linear model.
///
/// Maintains the conjugate normal posterior:
/// - **μ** — mean vector (posterior weight estimate)
/// - **B** — precision matrix (inverse covariance)
/// - **Σ** — covariance matrix (B⁻¹, tracked for O(p²) updates)
///
/// Both B and Σ are updated on every label via rank-1 updates.
/// Periodic Cholesky recomputation of Σ from B corrects accumulated
/// drift (´dec:posterior:recomputation-trigger´). The four tracking
/// fields the cadence reads are present and functional
/// (´dec:posterior:adaptive-cadence´).
///
/// The model is dense and dynamically dimensioned
/// (´dec:substrate:dense-dynamic´), and both matrices are tracked for the
/// reason the specification records (´rem:gaussian:dual-tracking´).
pub struct BayesianLinearModel {
    /// Posterior mean vector.
    mu: Col<f64>,
    /// Precision matrix B.
    precision: SymmetricMatrix,
    /// Covariance matrix Σ = B⁻¹.
    covariance: SymmetricMatrix,
    /// Current dimensionality.
    p: usize,
    /// The prior precision `λ_prior` this model was built at. It is held
    /// because the Schur regularisation is derived from it rather than
    /// fixed beside it: δ = `λ_prior` × `ε_Schur`
    /// (´alg:gaussian:regularised-schur´). The prior is the scale B starts
    /// at, so it is the scale the regularisation is taken against.
    lambda_prior: f64,
    /// Labels processed since last Cholesky recomputation.
    labels_since_recompute: u32,

    // ─── Cadence tracking fields (´dec:posterior:adaptive-cadence´) ──────
    /// Effective recompute interval, halved on non-clean outcomes and clean
    /// recomputations whose synchronisation error exceeds its model-width
    /// threshold (´def:monitoring:synchronisation-error´).
    n_recompute_effective: u32,
    /// Consecutive recomputes that were clean and adopted.
    consecutive_clean_recomputes: u32,
    /// The cheap diagonal ratio last read, a lower bound on the true
    /// condition number (´def:monitoring:condition-number´).
    last_diagonal_ratio: f64,
    /// Last synchronisation error (´def:monitoring:synchronisation-error´).
    last_sync_error: f64,
    /// Rebuilds that were needed, were adopted, and shortened the effective
    /// interval (´dec:posterior:adaptive-cadence´).
    sync_error_shortenings: u64,

    // ─── Aggregate counters for health reporting ──────────────────────────
    // TODO(2026-05-23) ´todo:code:move-these-aggregate-counters-out-of´: Move these aggregate counters out of
    // `BayesianLinearModel` and into working-copy `ModelPrecisionHealth`,
    // alongside dimensions-at-floor and sync-error shortening counts, so each
    // process carries its own concrete tracker
    // (´dec:health:concrete-trackers´).
    /// Cholesky recomputes this model has paid for.
    ///
    /// Rebuilds and nothing else. A visit that measured and found nothing to
    /// do factored no matrix and is counted beside this, not in it
    /// (´dec:posterior:recomputation-trigger´).
    total_recomputes: u64,
    /// Visits that measured and rebuilt nothing.
    ///
    /// The counter the restructure adds, and the one that says what the
    /// cadence is actually doing: a healthy model accumulates measurements and
    /// leaves the two counters beside it at zero.
    measurements: u64,
    /// Of which repaired the spectrum: rebuilds at which the spectral floor's
    /// deficit was positive and applied (´dec:posterior:spectral-floor´).
    ///
    /// The counter's meaning is unchanged and its subject has moved. It has
    /// always answered "how often has this model's precision needed help",
    /// and the help it counted was the retired factorisation shift; the help
    /// it counts now is the floor written into the matrix the model installs.
    /// The move is what keeps it a measurement: a counter left on the retired
    /// device could hold only zero, which is a constant reported where a
    /// measurement is promised.
    floored_rebuilds: u64,
    /// Of which hit the cascade terminus (severe).
    cascade_terminus_events: u64,
    /// Dimensions at the replenishment floor
    /// (´req:monitoring:replenishment-floor´).
    last_dimensions_at_floor: usize,
    /// Rebuilds that were needed and left the model no better, or that the
    /// factorisation refused (´dec:posterior:measured-adoption´).
    ///
    /// The counter is the same counter as before and its meaning has changed
    /// with the flow around it. It used to count declines, which a healthy
    /// model produced nine times in ten because a rebuild ran whether or not
    /// one was called for (´rep:assayer:declined-rebuild-cadence´). A rebuild
    /// now runs only where a measurement found drift over the threshold and
    /// above the resolution of its own reading, so a decline is a model whose
    /// drift is real and whose fresh inverse is worse than the pair it holds.
    /// That is an alarm, and a non-zero reading here is a condition to look
    /// at rather than a rate to expect.
    alarms: u64,

    // ─── The prior's share of the posterior precision ─────────────────────
    /// The identity-shaped share of `B` the spectral floor has put there
    /// (´dec:posterior:spectral-floor´).
    ///
    /// The accumulated per-label increments, decayed by the same factor `B`
    /// is scaled by, because they are part of `B` and decay with it. It is a
    /// scalar because every increment is a multiple of the identity.
    spectral_floor_mass: f64,

    /// The coordinate-shaped share of `B` the replenishment clamp has put
    /// there (´req:gaussian:prior-replenishment-floor´), one entry per
    /// coordinate.
    ///
    /// The accumulated amount the clamp lifted each diagonal entry by,
    /// decayed the same way. It rides alongside the precision matrix through
    /// every structural change: extended with zeros, gathered under a
    /// permutation, and restricted to the coordinates a marginalisation
    /// keeps.
    clamp_mass: Vec<f64>,

    /// The part of `clamp_mass` the maintained covariance has not been
    /// rebuilt against yet — what the clamp has added since the last adopted
    /// rebuild, decayed the same way.
    ///
    /// The two are different quantities and both are needed. The lifetime
    /// figure above says what share of the precision matrix this model
    /// borrowed rather than earned, which is a fact about the model and does
    /// not reset. This one says what the tracked pair currently disagrees by,
    /// which is a fact about the pair and stops being true the moment a
    /// rebuild takes the covariance from the precision matrix the clamp has
    /// already raised (´def:monitoring:synchronisation-error´). A monitor
    /// reading the lifetime figure would overstate the disagreement by
    /// everything earlier rebuilds already absorbed.
    ///
    /// It is not persisted. A restored model starts at zero and reports no
    /// prior-induced component until its first rebuild, which errs toward the
    /// cadence reacting to drift it cannot attribute rather than away from it.
    clamp_mass_since_rebuild: Vec<f64>,

    /// The record the last rebuild wrote, absent until the first one runs
    /// (´dec:posterior:recomputation-trigger´).
    ///
    /// Everything the per-label check tests is relative to this record, and
    /// everything the spectral floor holds is derived from it. A model that
    /// has not rebuilt yet holds neither, which is the honest state rather
    /// than a sentinel: the check falls back to its counter and the update
    /// applies no increment until a rebuild has measured a spectrum.
    baseline: Option<super::recompute::RecomputeBaseline>,
}

impl fmt::Debug for BayesianLinearModel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("BayesianLinearModel")
            .field("p", &self.p)
            .field("labels_since_recompute", &self.labels_since_recompute)
            .field("n_recompute_effective", &self.n_recompute_effective)
            .finish_non_exhaustive()
    }
}

impl BayesianLinearModel {
    // ─────────────────────────────────────────────────────────────────────
    // Construction
    // ─────────────────────────────────────────────────────────────────────

    /// Creates a new model with isotropic prior.
    ///
    /// - μ = 0 (length p)
    /// - B = `prior_precision` · I  (p × p)
    /// - Σ = (1 / `prior_precision`) · I  (p × p)
    /// - `n_recompute` — Cholesky recomputation cadence
    ///   (´alg:gaussian:condition-adaptive-recompute´), default 1000,
    ///   constraint ≥ 100. Threaded from
    ///   `CholeskyConfig::n_recompute`.
    ///
    /// # Panics
    ///
    /// Panics if `prior_precision` is not positive and finite.
    #[must_use]
    pub fn new(p: usize, prior_precision: f64, n_recompute: u32) -> Self {
        assert!(
            prior_precision > 0.0 && prior_precision.is_finite(),
            "prior_precision must be positive and finite, got {prior_precision}"
        );

        Self {
            mu: Col::zeros(p),
            precision: SymmetricMatrix::identity_scaled(p, prior_precision),
            covariance: SymmetricMatrix::identity_scaled(p, 1.0 / prior_precision),
            p,
            lambda_prior: prior_precision,
            labels_since_recompute: 0,
            n_recompute_effective: n_recompute,
            consecutive_clean_recomputes: 0,
            last_diagonal_ratio: 0.0,
            last_sync_error: 0.0,
            sync_error_shortenings: 0,
            total_recomputes: 0,
            measurements: 0,
            floored_rebuilds: 0,
            cascade_terminus_events: 0,
            last_dimensions_at_floor: 0,
            alarms: 0,
            spectral_floor_mass: 0.0,
            clamp_mass: vec![0.0; p],
            clamp_mass_since_rebuild: vec![0.0; p],
            baseline: None,
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Time decay
    // ─────────────────────────────────────────────────────────────────────

    /// Applies bulk time-decay to the precision matrix.
    ///
    /// Scales B by `decay_factor` (B ← `decay_factor` · B). Covariance Σ
    /// is scaled by `1 / decay_factor` to maintain the B = Σ⁻¹
    /// relationship approximately.
    ///
    /// Used by recovery to account for time elapsed since the checkpoint,
    /// which re-applies the elapsed decay exactly once
    /// (´dec:durability:decay-once´); all decay flows through the shared
    /// functions (´dec:clock:shared-functions´).
    pub fn apply_time_decay(&mut self, decay_factor: f64) {
        debug_assert!(
            decay_factor > 0.0 && decay_factor <= 1.0,
            "decay_factor must be in (0, 1], got {decay_factor}"
        );
        if (decay_factor - 1.0).abs() < f64::EPSILON {
            return;
        }
        self.precision.scale(decay_factor);
        self.covariance.scale(1.0 / decay_factor);
        // The two masses are shares of `B`, so a scaling of `B` is a scaling
        // of them (´dec:posterior:spectral-floor´).
        self.scale_prior_masses(decay_factor);
    }

    /// Scales the identity-shaped and coordinate-shaped prior masses by the
    /// factor the precision matrix was scaled by.
    fn scale_prior_masses(&mut self, factor: f64) {
        self.spectral_floor_mass *= factor;
        for mass in &mut self.clamp_mass {
            *mass *= factor;
        }
        for mass in &mut self.clamp_mass_since_rebuild {
            *mass *= factor;
        }
    }

    /// Restricts the coordinate-shaped mass to the coordinates a structural
    /// change keeps, in the order it keeps them.
    ///
    /// The same gather the precision matrix takes, so the entry standing at a
    /// coordinate after the change is the mass that coordinate accumulated
    /// before it. A coordinate that is not kept takes its mass with it.
    fn gather_clamp_mass(&mut self, indices: &[usize]) {
        self.clamp_mass = indices
            .iter()
            .map(|&i| self.clamp_mass.get(i).copied().unwrap_or(0.0))
            .collect();
        self.clamp_mass_since_rebuild = indices
            .iter()
            .map(|&i| self.clamp_mass_since_rebuild.get(i).copied().unwrap_or(0.0))
            .collect();
    }

    // ─────────────────────────────────────────────────────────────────────
    // Lifecycle: extend
    // ─────────────────────────────────────────────────────────────────────

    /// Extends the model by `r` dimensions with independent prior.
    ///
    /// The existing posterior is the exact marginal of the extended
    /// posterior. New dimensions receive isotropic prior:
    /// - `μ_new` = 0
    /// - `B_new` = `prior_precision` · I  (r × r, zero cross-blocks)
    /// - `Σ_new` = (1 / `prior_precision`) · I  (r × r, zero cross-blocks)
    ///
    /// This is the extension result (´thm:gaussian:extension´).
    ///
    /// # Panics
    ///
    /// Panics if `prior_precision` is not positive and finite.
    pub fn extend(&mut self, r: usize, prior_precision: f64) {
        if r == 0 {
            return;
        }
        assert!(
            prior_precision > 0.0 && prior_precision.is_finite(),
            "prior_precision must be positive and finite, got {prior_precision}"
        );

        let new_p = self.p.checked_add(r).expect("extend: dimension overflow");

        // Extend μ with zeros.
        let mut new_mu = Col::zeros(new_p);
        for i in 0..self.p {
            new_mu[i] = self.mu[i];
        }
        self.mu = new_mu;

        // Extend B and Σ using SymmetricMatrix::extend.
        self.precision = self.precision.extend(r, prior_precision);
        self.covariance = self.covariance.extend(r, 1.0 / prior_precision);

        // A coordinate that has just arrived has had no clamp applied to it.
        self.clamp_mass.resize(new_p, 0.0);
        self.clamp_mass_since_rebuild.resize(new_p, 0.0);

        self.p = new_p;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Lifecycle: hibernate and restore (´alg:registry:hibernation´)
    // ─────────────────────────────────────────────────────────────────────

    /// Copies out the self-structure block the named positions form.
    ///
    /// The mean at those positions, and the precision and covariance
    /// restricted to them in both index directions. Nothing coupling them to
    /// any other position is taken, because nothing coupling them to another
    /// position could be given back: extension writes zero off-diagonal
    /// blocks (´cav:limitation:hibernation´).
    ///
    /// The positions are taken in the order supplied rather than sorted, so a
    /// block whose positions are scattered — an outcome axis's features, one
    /// triple per identity dimension and one pair per Sentinel — comes back
    /// in the order a later registration will lay them down in.
    ///
    /// # Panics
    ///
    /// Debug-asserts that every position is in range.
    #[must_use]
    pub fn self_structure_block(&self, positions: &[usize]) -> super::hibernation::HibernatedBlock {
        debug_assert!(
            positions.iter().all(|&i| i < self.p),
            "self_structure_block: position out of range (p={})",
            self.p
        );
        super::hibernation::HibernatedBlock {
            mean: positions.iter().map(|&i| self.mu[i]).collect(),
            precision: self.precision.extract_symmetric_submatrix(positions),
            covariance: self.covariance.extract_symmetric_submatrix(positions),
        }
    }

    /// Writes an aged self-structure block over the positions an extension
    /// has just created.
    ///
    /// The block arrives at the positions `[start, start + k)`, which is
    /// where [`Self::extend`] put the new coordinates. The precision block is
    /// scaled by the ageing factor and the covariance block by its
    /// reciprocal, which is the same pairing bulk time-decay uses and is what
    /// keeps `Σ = B⁻¹` exactly across the ageing
    /// (´dec:clock:shared-functions´). The mean is carried across unscaled:
    /// what a decay of the posterior expresses is a loss of confidence in an
    /// estimate, not a movement of the estimate.
    ///
    /// The couplings the extension zeroed stay zero. That is the whole of
    /// what hibernation does not preserve, stated as an operation rather than
    /// as a caveat (´cav:limitation:hibernation´).
    ///
    /// # Panics
    ///
    /// Debug-asserts that the block fits, that its pieces agree on one width,
    /// and that the ageing factor is a usable positive number.
    pub fn restore_self_structure_block(&mut self, start: usize, block: &super::hibernation::HibernatedBlock, decay: f64) {
        debug_assert!(block.is_well_formed(), "restore_self_structure_block: malformed block");
        debug_assert!(
            decay > 0.0 && decay.is_finite() && (1.0 / decay).is_finite(),
            "restore_self_structure_block: decay must be a usable positive factor, got {decay}"
        );
        let k = block.width();
        debug_assert!(
            start.checked_add(k).is_some_and(|end| end <= self.p),
            "restore_self_structure_block: block of {k} at {start} does not fit in {}",
            self.p
        );

        let mut precision = block.precision.clone();
        precision.scale(decay);
        let mut covariance = block.covariance.clone();
        covariance.scale(1.0 / decay);

        for (offset, &value) in block.mean.iter().enumerate() {
            self.mu[start + offset] = value;
        }
        self.precision.write_diagonal_block(start, &precision);
        self.covariance.write_diagonal_block(start, &covariance);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Lifecycle: permute
    // ─────────────────────────────────────────────────────────────────────

    /// Rearranges the parameters into a new index order.
    ///
    /// `source[j]` names the current index whose value belongs at index `j`.
    /// The mean is gathered and both matrices take the symmetric permutation
    /// `P A Pᵀ`, so the posterior is the same distribution read under a new
    /// index semantics rather than a different one. Nothing is added or
    /// removed and no factorisation is involved.
    ///
    /// This is the physical rearrangement compaction already performs after a
    /// marginalisation (´alg:dimension:compaction´), under a different index
    /// set: the rebuild's canonical block order rather than the gap-closing
    /// shift down.
    ///
    /// # Panics
    ///
    /// Debug-asserts that `source` is a permutation of `0..p`.
    pub fn permute(&mut self, source: &[usize]) {
        debug_assert_eq!(source.len(), self.p, "permutation length must match p");
        #[cfg(debug_assertions)]
        {
            let mut seen = vec![false; self.p];
            let is_permutation = source.iter().all(|&i| i < self.p && !std::mem::replace(&mut seen[i], true));
            debug_assert!(is_permutation, "source must be a permutation of 0..p");
        }
        if source.len() != self.p || source.iter().enumerate().all(|(j, &i)| j == i) {
            return;
        }

        let mut new_mu = Col::zeros(self.p);
        for (j, &i) in source.iter().enumerate() {
            new_mu[j] = self.mu[i];
        }
        self.mu = new_mu;
        self.precision = self.precision.extract_symmetric_submatrix(source);
        self.covariance = self.covariance.extract_symmetric_submatrix(source);
        self.gather_clamp_mass(source);
    }

    // ─────────────────────────────────────────────────────────────────────
    // Lifecycle: marginalise — the Schur complement correction
    // (´dec:posterior:half-solve´)
    // ─────────────────────────────────────────────────────────────────────

    /// Removes the specified dimensions via the Schur complement.
    ///
    /// Computes `B' = B_kk − B_kr (B_rr + δI)⁻¹ B_rk` via the four-stage
    /// algorithm (´alg:gaussian:regularised-schur´). Falls back to `B' = B_kk` if the
    /// condition guard fires, factorisation fails, or `B'` verification
    /// fails.
    ///
    /// For general marginalisation, after the Schur step, `Σ' = (B')⁻¹`
    /// is recomputed via `cholesky_inverse`, because the covariance is always
    /// refactored (´dec:posterior:always-refactor´), and
    /// `labels_since_recompute` is reset.
    ///
    /// For rank-1 marginalisation (`r = 1`), `B'` is computed by scalar
    /// Schur complement and `Σ' = Σ_kk` is extracted directly, preserving
    /// the existing recomputation cadence at `O(p²)` cost.
    ///
    /// Marginalisation does not fail (´dec:posterior:cascade-never-fails´),
    /// (´dec:posterior:infallible-correction´): when the covariance
    /// recomputation reaches the cascade's terminus, this level retains
    /// and flags as the level below does — the uncorrected block is
    /// adopted, the covariance is the maintained one's kept block, and
    /// the outcome travels in the returned diagnostics.
    ///
    /// TODO(2026-05-23) ´todo:code:add-a-small-r´: Add a small-r
    /// competitive-cell exit path that extracts `Σ_kk` for the whole
    /// `1 + T5` removal block before falling back to full Cholesky — the
    /// low-rank marginalisation the cost table already prices
    /// (´tab:gaussian:operation-costs´).
    ///
    /// # Panics
    ///
    /// - Debug-asserts that `remove_indices` is sorted ascending.
    /// - Debug-asserts that all indices are in range `[0, p)`.
    pub fn marginalise(&mut self, remove_indices: &[usize], schur_config: &SchurConfig, model_id: ModelId) -> SchurDiagnostics {
        // Debug-assert: sorted ascending.
        debug_assert!(
            remove_indices.windows(2).all(|w| w[0] < w[1]),
            "remove_indices must be sorted ascending: {remove_indices:?}"
        );
        // Debug-assert: all in range.
        debug_assert!(
            remove_indices.iter().all(|&i| i < self.p),
            "remove_indices out of range: {remove_indices:?} (p={})",
            self.p
        );

        if remove_indices.is_empty() {
            return SchurDiagnostics::noop();
        }

        // TODO(2026-05-23) ´todo:code:generalise-this´: Generalise this
        // extraction path from rank-1 to competitive-cell small-r blocks,
        // which the cost table prices as low-rank marginalisation
        // (´tab:gaussian:operation-costs´).
        if let [idx] = remove_indices {
            return self.marginalise_rank1(*idx, schur_config, model_id);
        }

        // Compute kept indices (complement of remove_indices).
        let keep_indices = compute_keep_indices(self.p, remove_indices);

        // Step 1: Schur complement (´dec:posterior:half-solve´).
        // Always succeeds. Returns B' (with correction) or B_kk (fallback).
        let schur = compute_schur_complement(
            &self.precision,
            remove_indices,
            &keep_indices,
            schur_config,
            self.lambda_prior,
        );

        // Step 2: μ' = μ_k.
        let new_mu = extract_subvector(&self.mu, &keep_indices);
        let new_p = keep_indices.len();

        // Step 3: Σ' = (B')⁻¹ via cholesky_inverse
        // (´dec:posterior:always-refactor´).
        // Not extracted from old Σ — fresh Cholesky from B' is exact.
        // On the cascade's terminus, retain and flag: adopt the
        // uncorrected block and the maintained covariance's kept block
        // (´dec:degradation:retain-and-flag´).
        let (new_precision, new_covariance, diagnostics) = match cholesky_inverse_for_marginalisation(&schur.b_prime) {
            Ok(inv) => (schur.b_prime, inv, schur.diagnostics),
            Err(pivot_index) => {
                tracing::warn!(
                    ?model_id,
                    pivot_index,
                    "marginalisation covariance recomputation hit the cascade terminus; retaining B_kk and the maintained covariance"
                );
                let b_kk = self.precision.extract_symmetric_submatrix(&keep_indices);
                let cov_kk = self.covariance.extract_symmetric_submatrix(&keep_indices);
                // The correction was computed and is now being dropped, so
                // the error this event committed is the whole of it rather
                // than the offset's share of it. The recomputation is the
                // rare escape hatch, so recomputing the figure here is
                // cheaper than carrying the blocks through the common path.
                let diagnostics = SchurDiagnostics {
                    correction_outcome: SchurCorrectionOutcome::FactorisationFailed { pivot_index },
                    fell_back_to_bkk: true,
                    error: discarded_correction_error(
                        &self.precision,
                        remove_indices,
                        &keep_indices,
                        schur_config,
                        self.lambda_prior,
                    ),
                    ..schur.diagnostics
                };
                (b_kk, cov_kk, diagnostics)
            }
        };

        self.mu = new_mu;
        self.precision = new_precision;
        self.covariance = new_covariance;
        self.gather_clamp_mass(&keep_indices);
        self.p = new_p;
        self.labels_since_recompute = 0;
        self.consecutive_clean_recomputes = 0;

        diagnostics
    }

    /// Removes a single dimension using the `O(p²)` rank-1 Schur path.
    ///
    /// If scalar validation fails, falls back to `B_kk` and recomputes
    /// covariance via Cholesky. That rare escape hatch is `O(p³)`; the
    /// normal competitive-cell exit path remains `O(p²)`. A terminus
    /// inside the escape hatch retains the maintained covariance's kept
    /// block and flags, as [`Self::marginalise`] documents.
    ///
    /// The formed `B'` is verified against the general path's contract before
    /// it is adopted (´req:gaussian:rank-one-verification´). The cheap
    /// certificate is tried first and keeps the exit quadratic when it
    /// carries; where it is inconclusive the factorisation decides and the
    /// exit is cubic for that event. Which of the two decided travels in the returned
    /// diagnostics, so how often the quadratic case actually obtains is a
    /// figure a deployment reads rather than one this comment asserts.
    // Justified: mirrors `marginalise`'s signature; the id is diagnostic only.
    #[allow(clippy::needless_pass_by_value)]
    fn marginalise_rank1(&mut self, idx: usize, schur_config: &SchurConfig, model_id: ModelId) -> SchurDiagnostics {
        let keep_indices = compute_keep_indices(self.p, &[idx]);
        let new_p = keep_indices.len();

        let b_11 = self.precision.as_inner()[(idx, idx)];
        let b_kk = self.precision.extract_symmetric_submatrix(&keep_indices);
        let new_mu = extract_subvector(&self.mu, &keep_indices);
        let coupling_column = Col::from_fn(new_p, |i| self.precision.as_inner()[(keep_indices[i], idx)]);

        // A one-by-one block is its own spectrum, so this figure is the
        // condition number and not a lower bound on it: exactly one wherever
        // the entry is a usable precision, infinite where it is not
        // (´alg:gaussian:regularised-schur´). Adding δ to a positive scalar
        // leaves it at one, so the solvability guard has nothing to refuse
        // here that the denominator test below does not already refuse.
        let scalar_is_usable = b_11 > 0.0 && b_11.is_finite();
        let brr_cond_estimate = if scalar_is_usable { 1.0 } else { f64::INFINITY };
        let brr_regularised_cond = Some(brr_cond_estimate);
        let brr_cond_measure = ConditionMeasure::Spectral;

        // The exact correction's trace, which the closed-form error is taken
        // against (´alg:gaussian:regularised-schur´).
        let delta = schur_config.delta(self.lambda_prior);
        let coupling_mass: f64 = (0..new_p).map(|i| coupling_column[i] * coupling_column[i]).sum();
        let exact_correction_trace = (scalar_is_usable && (coupling_mass / b_11).is_finite()).then(|| coupling_mass / b_11);
        let discarded_error = rank1_marginalisation_error(exact_correction_trace, delta, b_11, false);

        // Recomputes Σ' = (B_kk)⁻¹ for the escape hatches, retaining the
        // maintained covariance's kept block on a terminus
        // (´dec:degradation:retain-and-flag´). Returns the covariance and
        // whether the terminus was reached.
        let covariance_or_retained = |model: &Self| -> (SymmetricMatrix, bool) {
            if new_p == 0 {
                return (b_kk.clone(), false);
            }
            match cholesky_inverse_for_marginalisation(&b_kk) {
                Ok(inv) => (inv, false),
                Err(pivot_index) => {
                    tracing::warn!(
                        ?model_id,
                        pivot_index,
                        "rank-1 marginalisation escape hatch hit the cascade terminus; retaining the maintained covariance"
                    );
                    (model.covariance.extract_symmetric_submatrix(&keep_indices), true)
                }
            }
        };

        // Two refusals, and they are not the same refusal. A removed scalar
        // that is not a usable precision says nothing about conditioning — a
        // one-by-one block's condition number is one whenever it has one — so
        // it reports as the degenerate block it is rather than under a guard's
        // name. Reporting it as a conditioning skip is the confusion the
        // conformance audit named as a hazard of counting the guard's variant:
        // an aggregate built on it would report a deployment conditioning
        // refusals it never had. The host's posture guard follows, and at r = 1
        // it can only refuse a block whose condition number is one, so it fires
        // only where a deployment has declared a ceiling below that.
        let refusal = if scalar_is_usable {
            (brr_cond_estimate > schur_config.posture_condition_ceiling).then_some(SchurCorrectionOutcome::PostureGuardSkip)
        } else {
            Some(SchurCorrectionOutcome::FactorisationFailed { pivot_index: 0 })
        };
        if let Some(correction_outcome) = refusal {
            let (new_covariance, _terminus) = covariance_or_retained(self);
            self.adopt_kept_block(new_mu, b_kk, new_covariance, &keep_indices, new_p);

            return SchurDiagnostics {
                brr_cond_estimate,
                brr_cond_measure,
                brr_regularised_cond,
                correction_outcome,
                // Refused before B' was formed, so nothing was put to a
                // certificate and this event is outside the acceptance rate.
                bprime_verification_passed: true,
                bprime_verification_certificate: VerificationCertificate::Vacuous,
                fell_back_to_bkk: true,
                error: discarded_error,
            };
        }

        // The scalar form of the solvability question: whether the regularised
        // denominator can be divided by at all. The general path asks it of a
        // condition number and then of a factorisation; here the two are one
        // test, and it reports as the factorisation it stands in for.
        let denom = b_11 + delta;
        if denom <= 0.0 || !denom.is_finite() {
            let (new_covariance, _terminus) = covariance_or_retained(self);
            self.adopt_kept_block(new_mu, b_kk, new_covariance, &keep_indices, new_p);

            return SchurDiagnostics {
                brr_cond_estimate,
                brr_cond_measure,
                brr_regularised_cond,
                correction_outcome: SchurCorrectionOutcome::FactorisationFailed { pivot_index: 0 },
                bprime_verification_passed: true,
                bprime_verification_certificate: VerificationCertificate::Vacuous,
                fell_back_to_bkk: true,
                error: discarded_error,
            };
        }

        let c = 1.0 / denom;
        let correction =
            SymmetricMatrix::from_computation(Mat::from_fn(new_p, new_p, |i, j| c * coupling_column[i] * coupling_column[j]));
        let b_prime = b_kk.subtract(&correction);
        // The cheap certificate first, the factorisation where it is inconclusive
        // (´req:gaussian:rank-one-verification´). What stood here before was a
        // diagonal scan, resting on the exact-arithmetic theorem that an SPD
        // parent and a positive denominator give an SPD regularised Schur
        // complement. The theorem is sound and the path does not compute it:
        // the correction is formed and subtracted entrywise in binary64, and
        // the rounded correction is neither rank one nor positive
        // semi-definite, so interlacing no longer protects the one exposed
        // eigenvalue. A represented parent that is exactly SPD can therefore
        // reach here as a B' with every diagonal entry near 10⁻⁷ and a least
        // eigenvalue near −4×10⁻¹⁹, which the diagonal scan adopted and the
        // general path's rule refuses.
        let (bprime_verification_passed, bprime_verification_certificate) = verify_rank1_positive_definite(&b_prime);

        let (new_precision, new_covariance, fell_back_to_bkk, used_cholesky) = if bprime_verification_passed {
            let covariance = self.covariance.extract_symmetric_submatrix(&keep_indices);
            (b_prime, covariance, false, false)
        } else {
            let (covariance, terminus) = covariance_or_retained(self);
            // A terminus inside the escape hatch retained the maintained
            // covariance, so the recomputation cadence is not reset.
            (b_kk, covariance, true, !terminus)
        };

        self.mu = new_mu;
        self.precision = new_precision;
        self.covariance = new_covariance;
        self.gather_clamp_mass(&keep_indices);
        self.p = new_p;
        if used_cholesky {
            self.labels_since_recompute = 0;
            self.consecutive_clean_recomputes = 0;
        }

        SchurDiagnostics {
            brr_cond_estimate,
            brr_cond_measure,
            brr_regularised_cond,
            correction_outcome: SchurCorrectionOutcome::Applied,
            bprime_verification_passed,
            bprime_verification_certificate,
            fell_back_to_bkk,
            // A refused B' discards the correction whole; an adopted one loses
            // only the share the offset attenuated.
            error: if fell_back_to_bkk {
                discarded_error
            } else {
                rank1_marginalisation_error(exact_correction_trace, delta, denom, true)
            },
        }
    }

    /// Adopts the kept block uncorrected, the state every rank-1 refusal
    /// leaves the model in (´dec:degradation:retain-and-flag´).
    fn adopt_kept_block(
        &mut self,
        new_mu: Col<f64>,
        b_kk: SymmetricMatrix,
        new_covariance: SymmetricMatrix,
        keep_indices: &[usize],
        new_p: usize,
    ) {
        self.mu = new_mu;
        self.precision = b_kk;
        self.covariance = new_covariance;
        self.gather_clamp_mass(keep_indices);
        self.p = new_p;
        self.labels_since_recompute = 0;
        self.consecutive_clean_recomputes = 0;
    }

    // ─────────────────────────────────────────────────────────────────────
    // Snapshot projection
    // ─────────────────────────────────────────────────────────────────────

    /// Projects the model into a serialisable `ModelParameters` snapshot.
    ///
    /// Clones μ and Σ only — B is excluded and reconstructed on demand
    /// (´dec:retention:precision-excluded´).
    #[must_use]
    pub fn to_parameters(&self) -> ModelParameters {
        ModelParameters {
            mu: col_to_vec(&self.mu),
            covariance_data: mat_to_vec(self.covariance.as_inner()),
            p: self.p,
        }
    }

    /// Projects the model into a full checkpoint state including B.
    ///
    /// Unlike `to_parameters`, this includes the precision matrix B
    /// and all cadence tracking fields, so the model can be restored
    /// without `cholesky_inverse`; this is the checkpoint
    /// (´dec:durability:checkpoint-journal´).
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn to_checkpoint_state(&self) -> crate::persistence::checkpoint::CheckpointModelState {
        crate::persistence::checkpoint::CheckpointModelState {
            parameters: self.to_parameters(),
            precision: self.precision.clone(),
            labels_since_recompute: self.labels_since_recompute,
            n_recompute_effective: self.n_recompute_effective,
            consecutive_clean_recomputes: self.consecutive_clean_recomputes,
            last_diagonal_ratio: self.last_diagonal_ratio,
            last_sync_error: self.last_sync_error,
            sync_error_shortenings: self.sync_error_shortenings,
            total_recomputes: self.total_recomputes,
            measurements: self.measurements,
            floored_rebuilds: self.floored_rebuilds,
            cascade_terminus_events: self.cascade_terminus_events,
            last_dimensions_at_floor: self.last_dimensions_at_floor,
            alarms: self.alarms,
            baseline: self.baseline,
            spectral_floor_mass: self.spectral_floor_mass,
            clamp_mass: self.clamp_mass.clone(),
        }
    }

    /// Reconstructs a model from a `ModelParameters` snapshot.
    ///
    /// B is recomputed as Σ⁻¹ via `cholesky_inverse`. The four cadence
    /// tracking fields are reset to defaults, and
    /// `labels_since_recompute` is set to 0 (fresh recompute).
    ///
    /// `floor_mass` is the pair the snapshot published beside these
    /// parameters, and it is restored rather than reset
    /// (´dec:posterior:spectral-floor´). The two masses are not derivable
    /// from Σ: they record how much of the precision behind it was borrowed
    /// from the floors rather than earned from labels, and a model that came
    /// back reading zero would report the covariance as entirely earned. That
    /// is the direction that under-reports uncertainty, which is the one
    /// direction a restore must not err in. The checkpoint path restores them
    /// for the same reason, and this is the same restore reached by the other
    /// route.
    ///
    /// A coordinate-shaped mass whose length is not this model's width is
    /// widened to zeros, on the reading the checkpoint path already takes: a
    /// vector that does not describe these coordinates says nothing about
    /// them, and zero is what nothing accumulated looks like.
    ///
    /// # Errors
    ///
    /// Returns `RevertError` where the Cholesky inverse of Σ is refused —
    /// either because no factorisation succeeded, or because one succeeded
    /// only under the diagonal shift, which on a restore is a refusal too
    /// (´dec:posterior:cascade-never-fails´). Both say the snapshot's
    /// covariance is not positive definite in the working arithmetic.
    pub fn from_parameters(
        params: &ModelParameters,
        floor_mass: &super::parameters::FloorMass,
        prior_precision: f64,
        model_id: ModelId,
        n_recompute_configured: u32,
    ) -> Result<Self, RevertError> {
        let p = params.p;

        if p == 0 {
            return Ok(Self::new(0, prior_precision, n_recompute_configured));
        }

        let mu = vec_to_col(&params.mu);
        let covariance_mat = vec_to_mat(&params.covariance_data, p, p);
        let covariance = SymmetricMatrix::from_computation(covariance_mat);

        // Reconstruct B = Σ⁻¹. The verdict is the factorisation's, and there
        // is one factorisation to take it from: a refusal is refused rather
        // than retried under a shift, because a shifted answer inverts a
        // matrix the snapshot does not carry and the model would hold a B
        // that is not this Σ's inverse permanently. A restore is the one place
        // where the matrix's provenance makes the disposition unambiguous:
        // the covariance came from a persisted artefact, and a non-definite
        // artefact is a defect of whoever wrote it and one that whoever wrote
        // it can act on
        // (´dec:posterior:cascade-never-fails´) (´dec:degradation:error-partition´).
        let precision = match cholesky_inverse(&covariance, CholeskyCallSite::Revert) {
            CholeskyInverseResult::Clean(inv) => inv,
            CholeskyInverseResult::Failed { pivot } => {
                return Err(RevertError {
                    source: CholeskyError {
                        pivot_index: pivot,
                        model: model_id,
                        context: "from_parameters: cholesky_inverse(Σ)",
                    },
                    p,
                });
            }
        };

        Ok(Self {
            mu,
            precision,
            covariance,
            p,
            lambda_prior: prior_precision,
            labels_since_recompute: 0,
            n_recompute_effective: n_recompute_configured,
            consecutive_clean_recomputes: 0,
            last_diagonal_ratio: 0.0,
            last_sync_error: 0.0,
            sync_error_shortenings: 0,
            total_recomputes: 0,
            measurements: 0,
            floored_rebuilds: 0,
            cascade_terminus_events: 0,
            last_dimensions_at_floor: 0,
            alarms: 0,
            spectral_floor_mass: floor_mass.spectral,
            clamp_mass: if floor_mass.clamp.len() == p {
                floor_mass.clamp.clone()
            } else {
                vec![0.0; p]
            },
            // Not carried by a snapshot: the restored pair is the one the
            // snapshot recorded, and what the clamp added before it is the
            // snapshot's business rather than this model's.
            clamp_mass_since_rebuild: vec![0.0; p],
            baseline: None,
        })
    }

    // ─────────────────────────────────────────────────────────────────────
    // Checkpoint restoration
    // ─────────────────────────────────────────────────────────────────────

    /// Restores a model from a full checkpoint state (μ, B, Σ, and
    /// all cadence tracking fields).
    ///
    /// Unlike `from_parameters`, this does **not** recompute B through a
    /// `cholesky_inverse`, because the precision matrix is included in the
    /// checkpoint (´dec:durability:checkpoint-journal´). It does factor the
    /// matrix it was handed, and that is a different thing: the inverse is
    /// not needed, but a verdict on the matrix is.
    ///
    /// `prior_precision` is not among the restored state and is not read
    /// from the payload. It comes from the configuration the instance is
    /// being rebuilt at, exactly as it does at construction, so a restore
    /// takes the regularisation the running deployment asks for rather
    /// than the one the writer happened to hold.
    ///
    /// # The verdict, and why it is taken here
    ///
    /// B arrives from a persisted artefact and is installed as the model's
    /// live precision matrix. Nothing between the file and this call reads
    /// its spectrum: the matrix type's deserialiser validates symmetry
    /// against a tolerance and returns on a triangle disagreement, which is
    /// the storage boundary's only verdict, and a matrix that is symmetric
    /// to the bit can still be indefinite by a whole eigenvalue. A truncated
    /// checkpoint, one hand-edited, one written by a build with a different
    /// convention, or one restored from the wrong deployment therefore
    /// becomes the posterior's precision without anything having disagreed.
    ///
    /// So the restore takes a verdict of its own, once, on the plain
    /// factorisation. A factorisation is the cheapest available witness of
    /// definiteness and this call has no factorisation of its own to reuse,
    /// which is the argument the marginalisation path already made for
    /// reading a spectrum rather than a diagonal — a minimum diagonal that is
    /// strictly positive is necessary for definiteness and is not sufficient
    /// for it (´req:gaussian:positive-definiteness´). The cost is one
    /// factorisation per model per restore, on an event that happens at
    /// start-up rather than per label.
    ///
    /// The covariance is not given a verdict of its own. The invariant the
    /// model promises is carried by B, which is what every lifecycle event
    /// and every leverage reading is taken against; a covariance that is
    /// inconsistent with a definite B is caught where inconsistency is
    /// measurable, at the next rebuild's adoption test, rather than guessed
    /// at here.
    ///
    /// # Errors
    ///
    /// Returns [`BuildError::CheckpointNotPositiveDefinite`] carrying the
    /// refused pivot where B does not factor. A restore happens while the
    /// instance is being constructed, so the builder's enum is the one that
    /// reaches a host at that moment (´dec:construction:eager-validation´).
    /// What a host does with it is act on the artefact: restore an earlier
    /// checkpoint, rebuild the state from the journal, or start clean and
    /// relearn. Construction fails rather than falling back to a cold start
    /// on its own, because a cold start silently discards every learned
    /// parameter and the choice to pay that is the host's to make.
    ///
    /// [`BuildError::CheckpointNotPositiveDefinite`]: crate::error::BuildError::CheckpointNotPositiveDefinite
    #[cfg(feature = "serde")]
    #[allow(clippy::too_many_arguments)] // Justified: 1:1 with checkpoint fields, plus the identity the refusal names; alternative is a builder struct
    pub fn from_checkpoint(
        mu: Col<f64>,
        precision: SymmetricMatrix,
        covariance: SymmetricMatrix,
        prior_precision: f64,
        model: ModelId,
        labels_since_recompute: u32,
        n_recompute_effective: u32,
        consecutive_clean_recomputes: u32,
        last_diagonal_ratio: f64,
        last_sync_error: f64,
        sync_error_shortenings: u64,
        total_recomputes: u64,
        measurements: u64,
        floored_rebuilds: u64,
        cascade_terminus_events: u64,
        last_dimensions_at_floor: usize,
        alarms: u64,
        baseline: Option<super::recompute::RecomputeBaseline>,
        spectral_floor_mass: f64,
        clamp_mass: Vec<f64>,
    ) -> Result<Self, crate::error::BuildError> {
        let p = mu.nrows();
        debug_assert_eq!(precision.dim(), p, "precision dim mismatch");
        debug_assert_eq!(covariance.dim(), p, "covariance dim mismatch");

        // A zero-width model has no matrix to factor and no spectrum to read;
        // the verdict is vacuous rather than favourable.
        if p > 0
            && let Err(refusal) = crate::linalg::bridge::cholesky_with_context(&precision, model, "from_checkpoint: cholesky(B)")
        {
            // The identity travels into the factorisation and back out on its
            // refusal, so the restore holds it once rather than twice.
            return Err(crate::error::BuildError::CheckpointNotPositiveDefinite {
                model: refusal.model,
                pivot: refusal.pivot_index,
            });
        }

        Ok(Self::install_stored_state(
            mu,
            precision,
            covariance,
            prior_precision,
            labels_since_recompute,
            n_recompute_effective,
            consecutive_clean_recomputes,
            last_diagonal_ratio,
            last_sync_error,
            sync_error_shortenings,
            total_recomputes,
            measurements,
            floored_rebuilds,
            cascade_terminus_events,
            last_dimensions_at_floor,
            alarms,
            baseline,
            spectral_floor_mass,
            clamp_mass,
        ))
    }

    /// Installs restored state into a model, with no verdict taken on it.
    ///
    /// This is [`Self::from_checkpoint`]'s second half, factored out so that
    /// the verdict is the only difference between the restore and the
    /// test-only route beside it. It is private, and it is the only thing in
    /// the package that writes a precision matrix nothing has certified.
    #[cfg(any(feature = "serde", test, feature = "test-support"))]
    #[allow(clippy::too_many_arguments)] // Justified: 1:1 with checkpoint fields; alternative is a builder struct
    fn install_stored_state(
        mu: Col<f64>,
        precision: SymmetricMatrix,
        covariance: SymmetricMatrix,
        prior_precision: f64,
        labels_since_recompute: u32,
        n_recompute_effective: u32,
        consecutive_clean_recomputes: u32,
        last_diagonal_ratio: f64,
        last_sync_error: f64,
        sync_error_shortenings: u64,
        total_recomputes: u64,
        measurements: u64,
        floored_rebuilds: u64,
        cascade_terminus_events: u64,
        last_dimensions_at_floor: usize,
        alarms: u64,
        baseline: Option<super::recompute::RecomputeBaseline>,
        spectral_floor_mass: f64,
        clamp_mass: Vec<f64>,
    ) -> Self {
        let p = mu.nrows();
        Self {
            mu,
            precision,
            covariance,
            p,
            lambda_prior: prior_precision,
            labels_since_recompute,
            n_recompute_effective,
            consecutive_clean_recomputes,
            last_diagonal_ratio,
            last_sync_error,
            sync_error_shortenings,
            total_recomputes,
            measurements,
            floored_rebuilds,
            cascade_terminus_events,
            last_dimensions_at_floor,
            alarms,
            spectral_floor_mass,
            // A checkpoint written before the coordinate-shaped mass existed
            // carries none, and an empty one widens to the model's own width
            // rather than being rejected: nothing was accumulated, and zero is
            // what nothing accumulated looks like.
            clamp_mass: if clamp_mass.len() == p { clamp_mass } else { vec![0.0; p] },
            // Not persisted, so every restore starts it at zero: a model that
            // has just been restored has no disagreement it can attribute to
            // the clamp until it rebuilds.
            clamp_mass_since_rebuild: vec![0.0; p],
            // A checkpoint written before the baseline existed carries none,
            // and the first rebuild after the restore establishes one.
            baseline,
        }
    }

    /// Installs stored state without the definiteness verdict, for tests that
    /// need a model holding a matrix the invariant forbids.
    ///
    /// Once the restore takes a verdict there is no production route left
    /// that installs an uncertified precision matrix, and that is the point
    /// of the verdict. But some questions can only be asked of a model that
    /// already holds such a matrix — what the marginalisation path does when
    /// its parent is barely indefinite, what a configured replacement delta
    /// changes about the covariance it produces — and those are questions
    /// about the maintenance path rather than about what a restore admits.
    /// Answering them needs a way in, and a way in that only the test builds
    /// have is better than a fallible restore whose failure the suite has
    /// learned to step around.
    ///
    /// A caller of this is asserting that the state it installs is a
    /// deliberate premise of the test and not an artefact it expects to be
    /// treated as valid.
    #[cfg(any(test, feature = "test-support"))]
    #[allow(clippy::too_many_arguments)] // Justified: 1:1 with the restore it mirrors; alternative is a builder struct
    pub fn from_stored_state_without_verdict(
        mu: Col<f64>,
        precision: SymmetricMatrix,
        covariance: SymmetricMatrix,
        prior_precision: f64,
        labels_since_recompute: u32,
        n_recompute_effective: u32,
        consecutive_clean_recomputes: u32,
        last_diagonal_ratio: f64,
        last_sync_error: f64,
        sync_error_shortenings: u64,
        total_recomputes: u64,
        measurements: u64,
        floored_rebuilds: u64,
        cascade_terminus_events: u64,
        last_dimensions_at_floor: usize,
        alarms: u64,
        baseline: Option<super::recompute::RecomputeBaseline>,
        spectral_floor_mass: f64,
        clamp_mass: Vec<f64>,
    ) -> Self {
        Self::install_stored_state(
            mu,
            precision,
            covariance,
            prior_precision,
            labels_since_recompute,
            n_recompute_effective,
            consecutive_clean_recomputes,
            last_diagonal_ratio,
            last_sync_error,
            sync_error_shortenings,
            total_recomputes,
            measurements,
            floored_rebuilds,
            cascade_terminus_events,
            last_dimensions_at_floor,
            alarms,
            baseline,
            spectral_floor_mass,
            clamp_mass,
        )
    }

    // ─────────────────────────────────────────────────────────────────────
    // Accessors
    // ─────────────────────────────────────────────────────────────────────

    /// Returns the current dimensionality.
    #[must_use]
    pub const fn dim(&self) -> usize {
        self.p
    }

    /// Returns a reference to the mean vector μ.
    #[must_use]
    pub const fn mu(&self) -> &Col<f64> {
        &self.mu
    }

    /// Returns a reference to the precision matrix B.
    #[must_use]
    pub const fn precision(&self) -> &SymmetricMatrix {
        &self.precision
    }

    /// Returns a reference to the covariance matrix Σ.
    #[must_use]
    pub const fn covariance(&self) -> &SymmetricMatrix {
        &self.covariance
    }

    /// Returns the number of labels processed since the last Cholesky
    /// recomputation.
    #[must_use]
    pub const fn labels_since_recompute(&self) -> u32 {
        self.labels_since_recompute
    }

    /// Returns the effective recompute interval (´dec:posterior:adaptive-cadence´).
    #[must_use]
    pub const fn n_recompute_effective(&self) -> u32 {
        self.n_recompute_effective
    }

    /// Returns the count of consecutive clean recomputes
    /// (´dec:posterior:adaptive-cadence´).
    #[must_use]
    pub const fn consecutive_clean_recomputes(&self) -> u32 {
        self.consecutive_clean_recomputes
    }

    /// Returns the last condition estimate (´def:monitoring:condition-number´).
    #[must_use]
    pub const fn last_diagonal_ratio(&self) -> f64 {
        self.last_diagonal_ratio
    }

    /// Returns the last synchronisation error
    /// (´def:monitoring:synchronisation-error´).
    #[must_use]
    pub const fn last_sync_error(&self) -> f64 {
        self.last_sync_error
    }

    /// Returns the number of needed rebuilds that shortened this model's
    /// effective interval (´dec:posterior:adaptive-cadence´).
    #[must_use]
    pub const fn sync_error_shortenings(&self) -> u64 {
        self.sync_error_shortenings
    }

    /// Returns the number of Cholesky recomputes this model has paid for, one
    /// of this process's own tracked counters (´dec:health:concrete-trackers´).
    #[must_use]
    pub const fn total_recomputes(&self) -> u64 {
        self.total_recomputes
    }

    /// Returns the number of visits that measured and rebuilt nothing
    /// (´dec:posterior:recomputation-trigger´).
    #[must_use]
    pub const fn measurements(&self) -> u64 {
        self.measurements
    }

    /// Returns the number of regularised Cholesky recomputes
    /// (´dec:health:concrete-trackers´).
    #[must_use]
    pub const fn floored_rebuilds(&self) -> u64 {
        self.floored_rebuilds
    }

    /// Returns the number of cascade terminus events
    /// (´dec:health:concrete-trackers´).
    #[must_use]
    pub const fn cascade_terminus_events(&self) -> u64 {
        self.cascade_terminus_events
    }

    /// Returns the last dimensions-at-floor count
    /// (´req:monitoring:replenishment-floor´).
    #[must_use]
    pub const fn last_dimensions_at_floor(&self) -> usize {
        self.last_dimensions_at_floor
    }

    /// Returns the count of rebuilds that were needed and left this model no
    /// better (´dec:posterior:measured-adoption´).
    #[must_use]
    pub const fn alarms(&self) -> u64 {
        self.alarms
    }

    /// Returns the identity-shaped share of the precision matrix the spectral
    /// floor has put there (´dec:posterior:spectral-floor´).
    #[must_use]
    pub const fn spectral_floor_mass(&self) -> f64 {
        self.spectral_floor_mass
    }

    /// Returns the coordinate-shaped share of the precision matrix the
    /// replenishment clamp has put there, one entry per coordinate
    /// (´req:gaussian:prior-replenishment-floor´).
    #[must_use]
    pub fn clamp_mass(&self) -> &[f64] {
        &self.clamp_mass
    }

    /// Returns the part of that mass the maintained covariance has not been
    /// rebuilt against — what the clamp has added since the last adopted
    /// rebuild, which is the component the prior contributes to the
    /// synchronisation reading (´def:monitoring:synchronisation-error´).
    #[must_use]
    pub fn clamp_mass_since_rebuild(&self) -> &[f64] {
        &self.clamp_mass_since_rebuild
    }

    /// The part of the last reading the prior put there rather than the
    /// arithmetic, from the record the last rebuild wrote
    /// (´def:monitoring:synchronisation-error´).
    ///
    /// Zero before the first rebuild and zero for a model restored from a
    /// checkpoint written before the components were separated, which reads
    /// as a model attributing none of its drift to the prior.
    #[must_use]
    pub fn last_prior_induced_sync_error(&self) -> f64 {
        self.baseline.map_or(0.0, |record| record.sync_error_prior_induced)
    }

    /// What was left of the last reading once the prior's part was taken out
    /// — the quantity the cadence acted on
    /// (´dec:posterior:adaptive-cadence´).
    #[must_use]
    pub fn last_sync_error_residual(&self) -> f64 {
        self.baseline.map_or(0.0, |record| record.sync_error_residual)
    }

    /// Returns the share of the posterior precision along `phi` that the two
    /// floors are holding up rather than the evidence — the precision this
    /// model borrowed rather than earned, resolved to one direction
    /// (´dec:posterior:spectral-floor´).
    ///
    /// Not to be confused with the spectral floor's share of the least
    /// eigenvalue ([`floor_share`](super::recompute::floor_share)), which is
    /// one reading per model taken along whichever direction the model is
    /// weakest in. This is one reading per direction, taken along whichever
    /// direction was asked about, and the two agree only where the direction
    /// asked about is the weakest one.
    ///
    /// The reading is `(f·‖φ‖² + Σ_j c_j·φ_j²) / (φᵀBφ)`, in `[0, 1]`, at
    /// `O(p²)` for the quadratic form and `O(p)` for the masses. This is the
    /// definition of the share. The assessment path evaluates the same
    /// function from the published covariance instead, along the direction
    /// that holds up the variance it is about to temper, because the
    /// precision matrix is not published (´dec:retention:precision-excluded´).
    ///
    /// **What it is exact about.** The split of `φᵀBφ` into a prior part and
    /// an evidence part, along `φ`. The two masses are the amounts the floors
    /// added to `B` in identity and coordinate shape, and both have been
    /// scaled by every factor `B` has been scaled by since, so the numerator
    /// is the floors' own contribution to this quadratic form rather than an
    /// estimate of it. Nothing here is fitted.
    ///
    /// **What it approximates.** The full posterior. A Rayleigh quotient
    /// reads one direction, so a share of one half along `φ` says that half
    /// the precision holding up `φ` came from the floors and says nothing
    /// about the subspace `φ` lies in. One overstatement is inherited from
    /// the masses themselves: the identity-shaped one is uniform across
    /// coordinates and so credits a coordinate that arrived after some of it
    /// was added with mass never applied to it. That is bounded by the
    /// floor's own magnitude, orders below the prior precision a new
    /// coordinate starts at, and it errs toward reporting more prior than
    /// there is — the safe direction for a reading a consumer treats as a
    /// source of uncertainty.
    ///
    /// A `phi` of the wrong width is not a direction in this model's space
    /// and reads zero, rather than a share computed over a truncation.
    #[must_use]
    pub fn borrowed_share(&self, phi: &[f64]) -> f64 {
        if phi.len() != self.p {
            return 0.0;
        }
        let mass = super::parameters::floor_mass_quadratic_form(self.spectral_floor_mass, &self.clamp_mass, phi);
        super::parameters::borrowed_share(mass, self.precision.quadratic_form_from_slice(phi))
    }

    /// Returns the record the last rebuild wrote, absent until one has run.
    #[must_use]
    pub const fn baseline(&self) -> Option<&super::recompute::RecomputeBaseline> {
        self.baseline.as_ref()
    }

    // ─────────────────────────────────────────────────────────────────────
    // Sherman–Morrison update (´alg:update:sherman-morrison´)
    // ─────────────────────────────────────────────────────────────────────

    /// Applies a leverage-bounded Sherman–Morrison rank-1 update.
    ///
    /// Implements the 7-substep algorithm (´alg:update:sherman-morrison´):
    ///
    /// | Step | Operation |
    /// |------|-----------|
    /// | 3 | `B ← γ_eff · B` |
    /// | 4 | `B_{jj} ← max(B_{jj}, λ_floor)` |
    /// | 5 | `B ← B + w_eff · φ̂φ̂ᵀ` |
    /// | 6 | `Σ ← (1/γ)(Σ − w·vvᵀ/(γ+wh))` |
    /// | 7 | `μ ← μ + [w(r−φ̂ᵀμ)/(γ+wh)] · v` |
    ///
    /// Steps 1–2 (computing `v = Σφ̂`, `h = φ̂ᵀv`, and `w_eff`) are performed
    /// by the caller and passed in.
    ///
    /// The update is refused, in every build profile, when the leverage does
    /// not admit one
    /// ([`leverage_admits_update`](super::update::leverage_admits_update)) or
    /// when the weight it produced is not a non-negative number. Refusing
    /// leaves every matrix exactly as it stood: no scaling, no floor, no
    /// rank-one term. That is the only available answer, because a negative
    /// weight is not a large or a small step in a good direction but a
    /// subtraction of information the model never received, and the state it
    /// would be subtracted from has already lost the property the reading was
    /// taken against.
    ///
    /// # Arguments
    ///
    /// * `phi_hat` — Standardised feature vector φ̂
    /// * `v` — Covariance product `Σφ̂` (computed by caller)
    /// * `h` — Leverage `φ̂ᵀΣφ̂` (computed by caller)
    /// * `w_eff` — Effective importance weight after leverage bounding
    /// * `r_rho` — Risk target `r_ρ ∈ {-1, 0, +1}`
    /// * `gamma_eff` — Combined decay factor `γ_label · γ_t^Δt`
    /// * `lambda_floor` — Replenishment floor for `B_{jj}`
    ///
    /// # Cross-References
    ///
    /// - (´dec:posterior:three-step-update´) — read, decide, mutate, in
    ///   three separated steps
    /// - (´req:gaussian:prior-replenishment-floor´) — the replenishment
    ///   floor
    /// - (´req:gaussian:positive-definiteness´) — the property a refused
    ///   reading reports the loss of
    #[allow(clippy::too_many_arguments)] // Justified: matches the 7-parameter SM calling convention
    pub fn apply_leverage_bounded_update(
        &mut self,
        phi_hat: &Col<f64>,
        v: &Col<f64>,
        h: f64,
        w_eff: f64,
        r_rho: f64,
        gamma_eff: f64,
        lambda_floor: f64,
    ) -> LabelUpdateOutcome {
        debug_assert_eq!(phi_hat.nrows(), self.p, "phi_hat dimension mismatch");
        debug_assert_eq!(v.nrows(), self.p, "v dimension mismatch");
        // Allows gamma_eff = 1.0 for testing (no-decay mode).
        // Production values are always strictly < 1
        // (´tab:risk:forgetting-rates´).
        debug_assert!(gamma_eff > 0.0 && gamma_eff <= 1.0, "gamma_eff must be in (0, 1]");
        debug_assert!(lambda_floor > 0.0, "lambda_floor must be positive");

        // The refusal, in every profile. A debug assertion here would have
        // made the release build the one that carried the negative weight
        // through, which is the build where nobody is watching.
        if !super::update::leverage_admits_update(h) || !w_eff.is_finite() || w_eff < 0.0 {
            return LabelUpdateOutcome::RefusedIndefiniteCovariance;
        }

        // Step 7 residual computation BEFORE μ mutation
        let phi_dot_mu: f64 = (0..self.p).map(|i| phi_hat[i] * self.mu[i]).sum();
        let residual = r_rho - phi_dot_mu;

        // Step 3: B ← γ_eff · B
        self.precision.scale(gamma_eff);
        // The prior's two shares of B are scaled with it, because they are
        // part of it (´dec:posterior:spectral-floor´).
        self.scale_prior_masses(gamma_eff);

        // Step 4: B_{jj} ← max(B_{jj}, λ_floor), with what the clamp lifts
        // each coordinate by accumulated into the coordinate-shaped mass. The
        // reading is taken before the mutation, which is the only order in
        // which the amount added is available at all.
        for j in 0..self.p {
            let before = self.precision.as_inner()[(j, j)];
            if before < lambda_floor {
                self.clamp_mass[j] += lambda_floor - before;
                self.clamp_mass_since_rebuild[j] += lambda_floor - before;
            }
        }
        self.precision.clamp_diagonal_min(lambda_floor);

        // Step 5: B ← B + w_eff · φ̂φ̂ᵀ
        self.precision.symmetric_rank1_update(w_eff, phi_hat);

        // Step 6: Σ ← (1/γ)(Σ − w·vvᵀ/(γ+wh))
        // Rewritten as: Σ ← (1/γ) · Σ + α · vvᵀ where α = -w_eff / (γ(γ+wh))
        // Using scale_and_symmetric_rank1_update(s, α, v) which does Σ ← s·Σ + α·vvᵀ
        let denom = w_eff.mul_add(h, gamma_eff);
        let scale = 1.0 / gamma_eff;
        let alpha = -w_eff / (gamma_eff * denom);
        self.covariance.scale_and_symmetric_rank1_update(scale, alpha, v);

        // Step 7: μ ← μ + [w(r−φ̂ᵀμ)/(γ+wh)] · v
        let coeff = (w_eff * residual) / denom;
        for i in 0..self.p {
            self.mu[i] = coeff.mul_add(v[i], self.mu[i]);
        }

        // Increment label counter
        self.labels_since_recompute += 1;

        LabelUpdateOutcome::Applied
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cholesky recomputation (´dec:posterior:recomputation-trigger´)
    // ─────────────────────────────────────────────────────────────────────

    /// Applies the outcome of one visit to this model.
    ///
    /// Installs the pair where a rebuild produced one the measurement
    /// adopted, stores the baseline, resets the label counter, moves the
    /// counters, and applies the cadence's shortening or recovery.
    ///
    /// A pair arrives only where a rebuild's own measurement adopted it, so
    /// `rebuilt.is_some()` and a [`Rebuilt`] outcome say the same thing
    /// (´dec:posterior:measured-adoption´). A synthesised outcome carrying
    /// neither pair nor baseline — the shortening a failed revert asks for —
    /// is an alarm, which is what it is: something needed doing and the model
    /// is no better for it.
    ///
    /// The pair is installed whole. The rebuild may have added the spectral
    /// floor's deficit to the precision matrix, and installing the covariance
    /// without the precision matrix it was taken from would recreate the
    /// inconsistency the floor's placement exists to avoid
    /// (´dec:posterior:spectral-floor´).
    ///
    /// The threshold is no longer a parameter. It enters at the measurement,
    /// which is where the residual is judged, and at the adoption test; by the
    /// time an outcome reaches the cadence the comparisons have been made and
    /// the outcome names their answer.
    ///
    /// # Arguments
    ///
    /// * `outcome` — what the visit concluded
    /// * `rebuilt` — the pair a rebuild produced and its measurement adopted
    /// * `baseline` — the record the visit wrote
    /// * `n_recompute_configured` — the configured recomputation interval
    ///
    /// # Sync Warning
    ///
    /// The shortening and recovery logic delegates to
    /// [`compute_new_interval`](super::recompute::compute_new_interval),
    /// which is the single source of truth shared with
    /// [`ModelPrecisionHealth::update_interval`](super::recompute::ModelPrecisionHealth::update_interval).
    ///
    /// # Cross-References
    ///
    /// - (´dec:posterior:adaptive-cadence´) — the shortening and the slow
    ///   recovery this applies
    ///
    /// [`Rebuilt`]: super::recompute::RecomputeOutcome::Rebuilt
    pub fn apply_recompute_outcome(
        &mut self,
        outcome: &super::recompute::RecomputeOutcome,
        rebuilt: Option<super::recompute::RebuiltPair>,
        baseline: Option<super::recompute::RecomputeBaseline>,
        n_recompute_configured: u32,
    ) {
        use super::recompute::{RecomputeOutcome, compute_new_interval};

        let old_interval = self.n_recompute_effective;
        // The whole reading is carried by the baseline, because the outcome
        // carries only the part of it the cadence acts on
        // (´def:monitoring:synchronisation-error´).
        let measured_before = baseline.map(|record| record.sync_error_before);

        // Install the pair the rebuild's measurement adopted, both halves or
        // neither.
        if let Some(pair) = rebuilt {
            self.precision = pair.precision;
            self.covariance = pair.covariance;
            // The rebuilt covariance was taken from the precision matrix the
            // clamp has already raised, so everything the clamp added up to
            // here is absorbed into the pair and is no longer a disagreement
            // the monitor can read (´def:monitoring:synchronisation-error´).
            // A visit that rebuilt nothing, and a rebuild the model declined,
            // both leave that disagreement standing — which is why this rides
            // with the pair rather than with the baseline.
            self.clamp_mass_since_rebuild.iter_mut().for_each(|mass| *mass = 0.0);
        }

        // The baseline the next label's check reads comes from here, and so
        // does the deficit the floor just added to the identity-shaped share
        // the prior holds in the precision matrix.
        //
        // **The condition is the outcome and not the record's own adoption
        // flag.** A measurement carries the last rebuild's record forward
        // unchanged, so that flag is still set on a visit that applied no
        // floor; reading it here would add the same deficit again at every
        // measurement that followed an adopted rebuild
        // (´dec:posterior:spectral-floor´).
        let rebuilt_and_adopted = matches!(outcome, RecomputeOutcome::Rebuilt { .. });
        if let Some(record) = baseline {
            if rebuilt_and_adopted {
                self.spectral_floor_mass += record.spectrum().map_or(0.0, |reading| reading.deficit);
                // The repaired count is taken here and under the same
                // condition as the mass, so the two never disagree: a rebuild
                // the measurement declined leaves the model holding neither
                // the floored matrix nor a claim to have repaired it
                // (´dec:posterior:spectral-floor´),
                // (´dec:posterior:measured-adoption´).
                self.floored_rebuilds += u64::from(record.floor_repaired());
            }
            self.baseline = Some(record);
        }

        // Every visit resets the counter, because every visit measured the
        // pair the model holds (´dec:posterior:recomputation-trigger´).
        self.labels_since_recompute = 0;

        // The three counters are disjoint and their sum is the number of
        // visits. A measurement factored nothing and is not a recompute; a
        // rebuild is, whatever its measurement then did with the answer; and
        // an alarm is a rebuild that is also a condition to publish.
        match outcome {
            RecomputeOutcome::Measured { .. } => {
                self.measurements += 1;
                self.consecutive_clean_recomputes += 1;
            }
            RecomputeOutcome::Rebuilt { drift_was_real, .. } => {
                self.total_recomputes += 1;
                // A rebuild that ran to absorb the replenishment clamp's own
                // contribution found no drift, so it does not break the run of
                // visits that found none (´dec:posterior:adaptive-cadence´).
                if *drift_was_real {
                    self.consecutive_clean_recomputes = 0;
                } else {
                    self.consecutive_clean_recomputes += 1;
                }
            }
            RecomputeOutcome::Alarm { refused_pivot, .. } => {
                self.total_recomputes += 1;
                self.alarms += 1;
                self.consecutive_clean_recomputes = 0;
                if refused_pivot.is_some() {
                    self.cascade_terminus_events += 1;
                }
            }
        }

        // The reading published per model is the measured one, both
        // components in it, as the specification defines the quantity. A
        // synthesised outcome carrying no baseline has attributed nothing to
        // the prior, so its residual is the whole of what it measured.
        self.last_sync_error = measured_before.unwrap_or_else(|| outcome.sync_error_residual());

        // Single source of truth for the shortening and recovery policy.
        self.n_recompute_effective = compute_new_interval(
            self.n_recompute_effective,
            n_recompute_configured,
            self.consecutive_clean_recomputes,
            outcome,
        );
        if matches!(
            outcome,
            RecomputeOutcome::Rebuilt {
                drift_was_real: true,
                ..
            }
        ) && self.n_recompute_effective < old_interval
        {
            self.sync_error_shortenings += 1;
        }
    }

    /// Updates the per-label health diagnostics from a recomputation trigger check.
    ///
    /// Called after every `should_recompute` regardless of whether
    /// recomputation was actually triggered.
    pub const fn update_trigger_diagnostics(&mut self, trigger: &super::recompute::RecomputeTrigger) {
        self.last_diagonal_ratio = trigger.diagonal_ratio();
        self.last_dimensions_at_floor = trigger.dimensions_at_floor();
    }
}
