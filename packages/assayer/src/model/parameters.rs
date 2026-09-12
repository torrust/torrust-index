// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Serialisable model parameter snapshot.
//!
//! `ModelParameters` captures the mean vector μ and covariance matrix Σ
//! of a `BayesianLinearModel` for snapshot publication and persistence.
//! The precision matrix B is excluded — it is reconstructed via
//! `cholesky_inverse` on restoration (´dec:retention:precision-excluded´).
//!
//! `FloorMass` captures the two amounts the model's floors have added to its
//! precision matrix, which is what lets a reader of the published parameters
//! tell prior from evidence along a direction (´dec:posterior:spectral-floor´).
//!
//! # Cross-References
//!
//! - (´dec:substrate:dense-dynamic´) — the dynamic-dimension model type
//! - (´dec:retention:precision-excluded´) — snapshot composition and the
//!   B-exclusion decision
//! - (´dec:posterior:spectral-floor´) — the two masses and the share they
//!   resolve to along a direction

/// Lightweight snapshot of a Bayesian linear model's learned state.
///
/// Contains μ (mean vector) and the column-major flat representation
/// of Σ (covariance matrix). The precision matrix B is **not** included —
/// it is reconstructed via `cholesky_inverse(Σ)` on restoration.
///
/// This is the unit of data published through `ArcSwap` snapshots
/// and persisted to checkpoint files.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelParameters {
    /// Mean vector μ — element-wise.
    pub mu: Vec<f64>,
    /// Covariance matrix Σ — column-major flat layout.
    pub covariance_data: Vec<f64>,
    /// Dimension p of the model (length of μ, Σ is p×p).
    pub p: usize,
}

impl ModelParameters {
    /// Creates an empty (zero-dimension) parameter set.
    #[must_use]
    pub const fn empty() -> Self {
        Self {
            mu: Vec::new(),
            covariance_data: Vec::new(),
            p: 0,
        }
    }
}

/// The two amounts a model's floors have added to its precision matrix
/// (´dec:posterior:spectral-floor´).
///
/// The spectral floor's additions are multiples of the identity, so one
/// scalar carries them; the replenishment clamp's additions are
/// coordinatewise, so a vector of the model's width carries them
/// (´req:gaussian:prior-replenishment-floor´). Both are parts of `B` and are
/// scaled whenever it is scaled, so they arrive here already decayed to the
/// matrix they describe.
///
/// The pair travels with the parameters because the covariance alone cannot
/// answer what the floors are holding up: the amounts are recoverable from
/// nothing the published matrices carry, and a floor whose size is known and
/// whose contribution is not is only half a declaration.
#[derive(Clone, Debug, Default, PartialEq)]
pub struct FloorMass {
    /// The identity-shaped mass `f` the spectral floor has added.
    pub spectral: f64,
    /// The coordinate-shaped masses `c_j` the replenishment clamp has added,
    /// one entry per coordinate.
    pub clamp: Vec<f64>,
}

impl FloorMass {
    /// Creates a floor-mass pair from its two tracked quantities.
    #[must_use]
    pub const fn new(spectral: f64, clamp: Vec<f64>) -> Self {
        Self { spectral, clamp }
    }

    /// Returns the floors' own quadratic form along `v`:
    /// `f·‖v‖² + Σ_j c_j·v_j²`.
    ///
    /// This is the numerator of the share (´dec:posterior:spectral-floor´).
    /// It is exact rather than estimated: the two masses are the amounts the
    /// floors added to `B` in those two shapes, so the floors' contribution
    /// to `vᵀBv` is this sum and nothing else.
    ///
    /// Coordinates beyond the clamp vector's length carry the identity-shaped
    /// mass alone, which is the reading for a coordinate no clamp has been
    /// applied to.
    #[must_use]
    pub fn quadratic_form(&self, v: &[f64]) -> f64 {
        floor_mass_quadratic_form(self.spectral, &self.clamp, v)
    }

    /// Returns the share of `precision_quadratic_form` the floors are holding
    /// up along the direction that quadratic form was taken along.
    ///
    /// The caller supplies `vᵀBv` because the two callers reach it by
    /// different routes: the model holds `B` and forms it directly, while a
    /// reader of the published covariance reaches it through the identity
    /// `(Σφ)ᵀB(Σφ) = φᵀΣφ`.
    ///
    /// A direction the model holds no precision along — a zero vector, or a
    /// quadratic form rounding has driven to zero or below — has no share to
    /// report and reads zero. Zero is the reading that leaves an uncertainty
    /// as it stands, which is the right answer to arithmetic noise: the
    /// alternative saturates a verdict on a denominator that has failed.
    #[must_use]
    pub fn share_along(&self, v: &[f64], precision_quadratic_form: f64) -> f64 {
        borrowed_share(self.quadratic_form(v), precision_quadratic_form)
    }
}

/// The floors' own quadratic form along `v`, from the two masses borrowed
/// rather than owned: `f·‖v‖² + Σ_j c_j·v_j²` (´dec:posterior:spectral-floor´).
///
/// The borrowed form exists because the live model holds its coordinate-shaped
/// masses in place and has no reason to copy them to read one direction.
#[must_use]
pub fn floor_mass_quadratic_form(spectral: f64, clamp: &[f64], v: &[f64]) -> f64 {
    let mut total = 0.0_f64;
    for (index, &value) in v.iter().enumerate() {
        let coefficient = spectral + clamp.get(index).copied().unwrap_or(0.0);
        total = (coefficient * value).mul_add(value, total);
    }
    total
}

/// The share of a precision quadratic form that a floor mass along the same
/// direction accounts for, in `[0, 1]` — the precision the model borrowed from
/// its floors rather than earned from evidence (´dec:posterior:spectral-floor´).
///
/// Distinct from the spectral reading of the same name
/// ([`floor_share`](crate::model::recompute::floor_share)), which asks what
/// fraction of a model's *least eigenvalue* the spectral floor is carrying.
/// That is one number per model and says where the model is weakest; this is
/// one number per direction and says whether the weakness is in the way.
///
/// A quadratic form that is not positive and finite reports no share, and a
/// mass that is not finite reports none either: both are readings the
/// arithmetic has failed to produce, and a share of zero leaves whatever
/// depends on it exactly as it stood.
#[must_use]
pub fn borrowed_share(mass: f64, precision_quadratic_form: f64) -> f64 {
    if precision_quadratic_form <= 0.0 || !precision_quadratic_form.is_finite() || !mass.is_finite() {
        return 0.0;
    }
    (mass / precision_quadratic_form).clamp(0.0, 1.0)
}
