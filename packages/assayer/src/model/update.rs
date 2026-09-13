// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sherman–Morrison leverage-bounded model update
//! (´alg:update:sherman-morrison´).
//!
//! Implements the 7-substep rank-1 update of a `BayesianLinearModel`:
//! time decay, precision floor, precision rank-1, covariance SM,
//! and mean update.
//!
//! # Cross-References
//!
//! - (´dec:posterior:leverage-before´) — leverage is bounded before the
//!   update and never repaired after it
//! - (´req:gaussian:prior-replenishment-floor´) — the replenishment floor
//!   `λ_floor`
//! - (´alg:update:sherman-morrison´) — the operational model update this
//!   weighting feeds

// ═══════════════════════════════════════════════════════════════════════════════
// Effective Weight
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes the effective importance weight after leverage bounding.
///
/// Returns `min(w_target, w_ceiling, c / (h + epsilon))`.
///
/// The leverage bound `c / (h + ε)` ensures that high-leverage observations
/// (where `h = φ̂ᵀΣφ̂` is large) receive reduced weight, preventing a single
/// observation from dominating the model update.
///
/// # Arguments
///
/// * `w_target` — Desired importance weight
/// * `w_ceiling` — Configuration ceiling (default 100)
/// * `c` — Leverage safety factor (default 5)
/// * `h` — Leverage: `φ̂ᵀΣφ̂`
/// * `epsilon` — Leverage guard (default 10⁻⁸)
///
/// # Cross-References
///
/// - (´dec:posterior:leverage-before´) — the leverage bounding this weight
///   applies
#[must_use]
pub fn compute_effective_weight(w_target: f64, w_ceiling: f64, c: f64, h: f64, epsilon: f64) -> f64 {
    let leverage_bound = c / (h + epsilon);
    w_target.min(w_ceiling).min(leverage_bound)
}

/// Whether a leverage reading can carry an update at all.
///
/// The leverage `h = φ̂ᵀΣφ̂` is a quadratic form in the covariance, so it is
/// non-negative for every feature vector exactly while `Σ` is positive
/// semi-definite. A negative reading is therefore neither a large leverage nor
/// a small one: it is the report that what the model is carrying is no longer
/// a covariance (´req:gaussian:positive-definiteness´).
///
/// It has to be refused rather than bounded because of what the weight above
/// does with it. The leverage bound `c / (h + ε)` is a division, and a
/// negative denominator changes the quotient's sign rather than its size, so a
/// negative leverage yields a negative effective weight. A negative weight in
/// the rank-one update subtracts information the model never received, which
/// carries the precision matrix further from positive definiteness on every
/// label that follows — the same loss, fed back through the update that reads
/// it. There is no weight at which the update would be right, because the
/// state it would be applied to is already wrong.
///
/// A non-finite reading is refused on the same footing: nothing downstream of
/// a NaN leverage is a number.
///
/// # Cross-References
///
/// - (´req:gaussian:positive-definiteness´) — the property whose loss this
///   reading reports
/// - (´dec:posterior:leverage-before´) — the bounding this predicate stands in
///   front of
#[must_use]
pub fn leverage_admits_update(h: f64) -> bool {
    h.is_finite() && h >= 0.0
}

/// Whether the leverage bound was the binding constraint.
///
/// Returns `true` when `c / (h + ε) < min(w_target, w_ceiling)`,
/// meaning the observation's leverage forced a weight reduction.
///
/// # Cross-References
///
/// - (´prop:update:leverage-bound´) — the bound this predicate reports on
#[must_use]
pub fn leverage_bound_fired(w_target: f64, w_ceiling: f64, c: f64, h: f64, epsilon: f64) -> bool {
    c / (h + epsilon) < w_target.min(w_ceiling)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Leverage-Bounded Update Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Default leverage safety factor `c`.
///
/// Prevents any single observation from dominating: at this factor no
/// observation sharpens the precision along its own direction by more
/// than a factor of six (´prop:update:leverage-bound´).
///
/// ´const:assayer:observation-sharpening-bound´ (´alg:const:scalar´)
/// ´const:assayer:observation-sharpening-bound-scalar-5p0´
pub const DEFAULT_LEVERAGE_SAFETY_FACTOR: f64 = 5.0;

/// Default importance weight ceiling.
///
/// Hard cap on importance weight, the ceiling the balancing weights are
/// taken against (´def:weighting:balancing-weights´).
///
/// ´const:assayer:balancing-weight-ceiling´ (´alg:const:scalar´)
/// ´const:assayer:balancing-weight-ceiling-scalar-100p0´
pub const DEFAULT_IMPORTANCE_CEILING: f64 = 100.0;

/// Default leverage guard `ε` to prevent division by zero.
///
/// A shipped operating point rather than a derived magnitude: it must
/// sit below every leverage carrying information and above the point
/// the quotient stops being finite, and it is revisable anywhere in
/// that very wide band (´tab:degradation:guard-magnitudes´).
///
/// ´const:assayer:leverage-division-guard´ (´alg:const:scalar´)
/// ´const:assayer:leverage-division-guard-scalar-1en8´
pub const EPSILON_LEVERAGE: f64 = 1e-8;

/// Default replenishment floor `λ_floor` for precision diagonal.
///
/// A hundredth of the prior precision, clamping each diagonal entry after
/// forgetting (´req:gaussian:prior-replenishment-floor´).
///
/// ´const:assayer:precision-replenishment-floor´ (´alg:const:scalar´)
/// ´const:assayer:precision-replenishment-floor-scalar-1en3´
pub const DEFAULT_LAMBDA_FLOOR: f64 = 1e-3;

// ═══════════════════════════════════════════════════════════════════════════════
// Per-Label Decay Rates
// ═══════════════════════════════════════════════════════════════════════════════
// Time-indexed decay rates (per hour, uniform across models) live in
// `TemporalConfig`, and all decay flows through the shared functions
// (´dec:clock:shared-functions´).  Only per-label rates are constants here.

/// Default per-label decay rate for the operational model (`γ_label,op`).
///
/// The fastest of the three: operational risk carries the host's
/// intervention posture, which changes overnight (´tab:risk:forgetting-rates´).
///
/// ´const:assayer:operational-forgetting-rate´ (´alg:const:scalar´)
/// ´const:assayer:operational-forgetting-rate-scalar-0p9995´
pub const GAMMA_LABEL_OPERATIONAL: f64 = 0.9995;

/// Default per-label decay rate for the sister model (`γ_label,sis`).
///
/// Inherent risk reflects the underlying environment, which changes on its
/// own slower schedule (´tab:risk:forgetting-rates´).
///
/// ´const:assayer:sister-forgetting-rate´ (´alg:const:scalar´)
/// ´const:assayer:sister-forgetting-rate-scalar-0p9998´
pub const GAMMA_LABEL_SISTER: f64 = 0.9998;

/// Default per-label decay rate for the anchor model (`γ_label,anc`).
///
/// Coarse risk forgets at the inherited rate rather than the operational one
/// (´tab:risk:forgetting-rates´).
///
/// ´const:assayer:anchor-forgetting-rate´ (´alg:const:scalar´)
/// ´const:assayer:anchor-forgetting-rate-scalar-0p9998´
pub const GAMMA_LABEL_ANCHOR: f64 = 0.9998;

// Note: The actual `apply_leverage_bounded_update` method is implemented
// on `BayesianLinearModel` in `model/bayesian.rs`.
