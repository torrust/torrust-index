// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Subject-level fixtures for crate tests whose values are not owned by the shared scenario.

// ═══════════════════════════════════════════════════════════════════════════════
// Derivation-Layer Fixtures
// ═══════════════════════════════════════════════════════════════════════════════

/// A risk basis carrying exactly the sufficient statistic the derivation
/// reads (´schema:risk:basis´); every other field is inert there by
/// outcome-prediction neutrality (´def:landscape:outcome-neutrality´).
#[must_use]
pub fn risk_basis(p_bad: f64, sigma_eff: f64, kappa_eff: f64) -> crate::RiskBasis {
    crate::RiskBasis {
        p_bad,
        uncertainty: p_bad * (1.0 - p_bad) * sigma_eff / kappa_eff,
        // A basis built to a stated uncertainty has not been tempered, so it
        // reports nothing borrowed and the two figures stay consistent.
        borrowed_share: 0.0,
        anchor_weight: 0.0,
        rho_eff: crate::numerics::stable_logit(p_bad.clamp(1e-9, 1.0 - 1e-9)) * kappa_eff,
        sigma_eff,
        kappa_eff,
        p_bad_sister: p_bad,
        p_bad_operational: p_bad,
        intervention_effectiveness: 0.0,
        anchor_converged: false,
        n_sentinels_reporting: 0,
        sister_regime_calibration_records: 0.0,
        anchor_regime_calibration_records: 0.0,
    }
}

/// The four kernel scalars the retired input bundle carried, kept as a
/// terse fixture shape for the rendering tests.
#[derive(Clone, Debug)]
pub struct KernelScalars {
    /// Calibrated probability of adverse outcome.
    pub p_hat: f64,
    /// Logit-space effective standard deviation.
    pub sigma_eff: f64,
    /// Effective calibration condition number.
    pub kappa_eff: f64,
    /// Variance of the challenge-effectiveness estimate.
    pub sigma2_q_hat_c: f64,
}

/// Runs the layered pipeline — derive the landscape, render it — and
/// returns the rendered tags, the shape the retired single-call kernel
/// returned (´dec:landscape:rendering-optional´).
#[must_use]
pub fn derive_tags(
    scalars: &KernelScalars,
    q_hat_c: f64,
    policy: &crate::ChannelPolicy,
    config: &crate::ResonanceConfig,
) -> Vec<crate::TagResonance> {
    let basis = risk_basis(scalars.p_hat, scalars.sigma_eff, scalars.kappa_eff);
    let estimate = crate::ChallengeEstimate::new(q_hat_c, scalars.sigma2_q_hat_c).expect("test estimate pair in domain");
    let landscape = crate::resonance::landscape::derive_landscape_from_basis(&basis, policy, estimate);
    crate::render_resonances(&landscape, &basis, config).tags
}
