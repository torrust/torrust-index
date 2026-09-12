// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`cusum_accumulates_positive_residuals`] | wellness | cites (´claim:wellness:the-two-drift-accumulators-are-one-sided-so-over-and-under-prediction-cannot-cancel´) |
//! | [`cusum_accumulates_negative_residuals`] | wellness | cites (´claim:wellness:the-two-drift-accumulators-are-one-sided-so-over-and-under-prediction-cannot-cancel´) |
//! | [`cusum_decays_with_mixed_residuals`] | wellness | cites (´claim:wellness:residuals-inside-the-allowance-never-accumulate-however-long-they-continue´) |
//! | [`auto_reset_triggers`] | wellness | cites (´claim:wellness:an-accumulator-crossing-its-threshold-reports-the-reset-back-to-its-caller´) |
//! | [`ewma_updates`] | wellness | cites (´claim:wellness:the-smoothed-residual-magnitude-and-sign-converge-on-the-residuals-actually-seen´) |
//! | [`serde_round_trip`] | wellness | cites (´claim:wellness:drift-state-round-trips-with-both-accumulators-the-smoothed-measures-and-the-step-count´) |

//! Drift detection via CUSUM accumulators.
//!
//! This module implements per-model prediction drift detection using
//! one-sided CUSUM accumulators and supplementary EWMAs.
//!
//! # Algorithm
//!
//! For each model, we track:
//! - **S⁺**: One-sided CUSUM for underestimation (residual > 0)
//! - **S⁻**: One-sided CUSUM for overestimation (residual < 0)
//! - **MAR**: Mean absolute residual (EWMA)
//! - **Residual sign EWMA**: Detects systematic bias
//!
//! Auto-reset occurs when S⁺ > h or S⁻ > h (threshold), emitting
//! a health event and resetting both CUSUMs and the step counter.
//!
//! # Cross-References
//!
//! - (´alg:monitoring:drift-cusums´) — the detector these accumulators
//!   implement
//! - (´dec:calibration:drift-reset´) — why a large calibration shift resets
//!   every accumulator here

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════════
// Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for drift detection: the thresholds and rates the CUSUM
/// detector runs on (´alg:monitoring:drift-cusums´).
// TODO(2026-05-17) ´todo:code:reconcile-this-runtime-drift-config´: Reconcile this runtime drift config
// with the Core construction `DriftConfig` surface, which the parameter table
// declares (´tab:construction:parameters´), before adding it to
// `AssayerConfig`.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DriftConfig {
    /// CUSUM allowance parameter `κ_drift`.
    ///
    /// Larger values absorb more residual variance before accumulation.
    /// Default: 0.1 (´tab:config:monitoring´).
    pub kappa_drift: f64,

    /// EWMA decay factor `γ_drift`.
    ///
    /// Controls the smoothing of MAR and residual sign EWMA.
    /// Default: 0.99 (~100 effective samples).
    pub gamma_drift: f64,

    /// CUSUM auto-reset threshold h.
    ///
    /// When S⁺ > h or S⁻ > h, both CUSUMs reset and a health event
    /// is emitted. Default: 10.0.
    pub h_threshold: f64,
}

impl Default for DriftConfig {
    fn default() -> Self {
        Self {
            kappa_drift: 0.1,
            gamma_drift: 0.99,
            h_threshold: 10.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DriftState
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-model drift accumulator state.
///
/// Updated at label step 16 with the residual `r - ρ̂_eff` where:
/// - `r` is the observed risk target (from the label's valence)
/// - `ρ̂_eff` is the assessment-time blended prediction
///
/// # CUSUM Formulas
///
/// ```text
/// S⁺ ← max(0, S⁺ + residual − κ_drift)
/// S⁻ ← max(0, S⁻ − residual − κ_drift)
/// ```
///
/// # Cross-References
///
/// - (´alg:monitoring:drift-cusums´) — the drift detection algorithm these
///   formulas are
#[derive(Clone, Debug, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DriftState {
    /// One-sided CUSUM for underestimation (positive residuals).
    pub s_plus: f64,

    /// One-sided CUSUM for overestimation (negative residuals).
    pub s_minus: f64,

    /// Exponentially weighted mean absolute residual.
    pub mean_abs_residual: f64,

    /// Exponentially weighted mean of sign(residual).
    ///
    /// Values near +1 indicate systematic underestimation;
    /// values near −1 indicate systematic overestimation.
    pub residual_sign_ewma: f64,

    /// Number of label steps since the last CUSUM auto-reset.
    pub steps_since_reset: u64,
}

impl DriftState {
    /// Creates a fresh drift state (all accumulators at zero).
    #[must_use]
    #[allow(dead_code)] // Justified: test-only constructor kept beside the type; production restores or defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Updates the drift accumulators with a new residual observation.
    ///
    /// # Arguments
    ///
    /// * `residual` — The prediction error: `r_rho - rho_predicted`
    /// * `config` — Drift configuration (`κ_drift`, `γ_drift`)
    ///
    /// # Returns
    ///
    /// `true` if auto-reset was triggered (S⁺ > h or S⁻ > h), `false` otherwise.
    pub fn update(&mut self, residual: f64, config: &DriftConfig) -> bool {
        // CUSUM update (´alg:monitoring:drift-cusums´)
        self.s_plus = (self.s_plus + residual - config.kappa_drift).max(0.0);
        self.s_minus = (self.s_minus - residual - config.kappa_drift).max(0.0);

        // EWMA updates (´alg:monitoring:drift-cusums´)
        let gamma = config.gamma_drift;
        let one_minus_gamma = 1.0 - gamma;
        self.mean_abs_residual = one_minus_gamma.mul_add(residual.abs(), gamma * self.mean_abs_residual);
        self.residual_sign_ewma = one_minus_gamma.mul_add(residual.signum(), gamma * self.residual_sign_ewma);

        // Increment step counter
        self.steps_since_reset += 1;

        // Check for auto-reset
        let should_reset = self.s_plus > config.h_threshold || self.s_minus > config.h_threshold;
        if should_reset {
            self.reset_cusums();
        }

        should_reset
    }

    /// Resets the CUSUM accumulators and step counter.
    ///
    /// Called on auto-reset (when CUSUMs exceed threshold) or when
    /// Platt calibration `δ_cal` exceeds its threshold.
    ///
    /// Note: The EWMAs (MAR and residual sign) are NOT reset.
    pub const fn reset_cusums(&mut self) {
        self.s_plus = 0.0;
        self.s_minus = 0.0;
        self.steps_since_reset = 0;
    }

    /// Fully resets all drift state to initial values.
    ///
    /// The model-replacement operation: residual statistics describing
    /// a predecessor's predictions would mislead applied to its
    /// successor. No reset row calls for it today — the calibration
    /// trigger narrowed to `reset_cusums` on the affected regime
    /// (´tab:monitoring:drift-resets´) — so it waits on a model
    /// replacement surface.
    #[allow(dead_code)] // Justified: the model-replacement semantics, pinned by its unit test; no production replacement surface exists yet
    pub fn reset_all(&mut self) {
        *self = Self::default();
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Unit Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Fifty rounds of residuals twice the allowance drive the upper
    /// accumulator alone: the lower one stays at zero to within floating-point
    /// exactness, not merely small. Underestimation leaves no trace whatever in
    /// the overestimation accumulator, so the two readings can be interpreted
    /// independently.
    ///
    /// (´claim:wellness:the-two-drift-accumulators-are-one-sided-so-over-and-under-prediction-cannot-cancel´)
    /// ´test:unit:cusum-accumulates-positive-residuals´
    #[test]
    fn cusum_accumulates_positive_residuals() {
        let config = DriftConfig::default();
        let mut drift = DriftState::new();

        // 50 labels with sustained positive residual well past the
        // allowance: each step banks 1.0 − κ_drift = 0.9.
        for _ in 0..50 {
            drift.update(1.0, &config);
        }

        // S⁺ accumulates 0.9 per step and auto-resets past the threshold
        // of 10; after resets it continues accumulating.
        assert!(drift.s_plus >= 0.0, "S⁺ should be non-negative");
        assert!(drift.s_minus < f64::EPSILON, "S⁻ should be ~0 for positive residuals");
    }

    /// The mirror holds exactly: sustained negative residuals build the lower
    /// accumulator and leave the upper one at zero. Neither direction is
    /// privileged — a model that systematically over-predicts is detected by
    /// the same machinery, on the same terms, as one that under-predicts.
    ///
    /// (´claim:wellness:the-two-drift-accumulators-are-one-sided-so-over-and-under-prediction-cannot-cancel´)
    /// ´test:unit:cusum-accumulates-negative-residuals´
    #[test]
    fn cusum_accumulates_negative_residuals() {
        let config = DriftConfig::default();
        let mut drift = DriftState::new();

        // 20 labels with sustained negative residual -1.0
        for _ in 0..20 {
            drift.update(-1.0, &config);
        }

        // S⁻ should accumulate
        assert!(drift.s_minus >= 0.0, "S⁻ should be non-negative");
        assert!(drift.s_plus < f64::EPSILON, "S⁺ should be ~0 for negative residuals");
    }

    /// Residuals alternating either side of zero, each below the allowance,
    /// leave both accumulators small after a hundred rounds. The allowance is
    /// subtracted before anything is banked, so error that never exceeds it
    /// contributes nothing no matter how many times it recurs.
    ///
    /// (´claim:wellness:residuals-inside-the-allowance-never-accumulate-however-long-they-continue´)
    /// ´test:unit:cusum-decays-with-mixed-residuals´
    #[test]
    fn cusum_decays_with_mixed_residuals() {
        let config = DriftConfig::default();
        let mut drift = DriftState::new();

        // 100 labels alternating ±0.05, inside the κ_drift = 0.1 allowance
        for i in 0..100 {
            let residual = if i % 2 == 0 { 0.05 } else { -0.05 };
            drift.update(residual, &config);
        }

        // With |residual| < κ_drift, CUSUMs should stay near zero
        assert!(drift.s_plus < 1.0, "S⁺ should be small with low residuals");
        assert!(drift.s_minus < 1.0, "S⁻ should be small with low residuals");
    }

    /// A residual equal to the configured allowance banks nothing, an
    /// accumulator equal to the configured threshold does not reset, and the
    /// smallest represented crossing reports and performs the reset.
    ///
    /// (´claim:wellness:an-accumulator-crossing-its-threshold-reports-the-reset-back-to-its-caller´)
    /// ´test:unit:auto-reset-triggers´
    #[test]
    fn auto_reset_triggers() {
        let config = DriftConfig {
            kappa_drift: 0.25,
            gamma_drift: 0.99,
            h_threshold: 0.75,
        };
        let mut drift = DriftState::new();

        assert!(!drift.update(0.25, &config));
        assert_eq!(
            drift.s_plus.to_bits(),
            0.0f64.to_bits(),
            "the configured allowance is inclusive"
        );
        assert!(!drift.update(1.0, &config));
        assert_eq!(
            drift.s_plus.to_bits(),
            0.75f64.to_bits(),
            "equality with the threshold stays observable"
        );
        assert!(drift.update(0.25 + f64::EPSILON, &config));
        assert_eq!(
            drift.s_plus.to_bits(),
            0.0f64.to_bits(),
            "crossing the configured threshold resets"
        );
    }

    /// The smoothed measures track the residuals through the same updates that
    /// drive the accumulators, and keep doing so across the resets those
    /// accumulators take: after two hundred rounds of a constant residual, the
    /// mean absolute residual has closed on its magnitude and the sign measure
    /// on its direction. The bias description survives the very detections it
    /// helps explain.
    ///
    /// (´claim:wellness:the-smoothed-residual-magnitude-and-sign-converge-on-the-residuals-actually-seen´)
    /// ´test:unit:ewma-updates´
    #[test]
    fn ewma_updates() {
        let config = DriftConfig::default();
        let mut drift = DriftState::new();

        // Feed constant positive residuals
        // With gamma=0.99, EWMA after n steps ≈ target * (1 - gamma^n)
        // Need ~200 steps for gamma^n ≈ 0.13, giving EWMA ≈ 0.87 * target
        for _ in 0..200 {
            drift.update(0.4, &config);
        }

        // MAR should approach 0.4 (with gamma=0.99, after 200 steps: ~0.35)
        assert!(
            (drift.mean_abs_residual - 0.4).abs() < 0.1,
            "MAR should converge to ~0.4, got {}",
            drift.mean_abs_residual
        );

        // Residual sign EWMA should approach +1.0
        assert!(
            drift.residual_sign_ewma > 0.8,
            "Sign EWMA should be near +1.0 for all-positive residuals, got {}",
            drift.residual_sign_ewma
        );
    }

    /// Every field of a drift state survives serialisation to within
    /// floating-point exactness, the negative sign measure included. A
    /// round-trip that flattened the sign would preserve the fact of a bias
    /// while losing which way it leaned — the half that says what to do about
    /// it.
    ///
    /// (´claim:wellness:drift-state-round-trips-with-both-accumulators-the-smoothed-measures-and-the-step-count´)
    /// ´test:unit:serde-round-trip´
    #[cfg(feature = "serde")]
    #[test]
    fn serde_round_trip() {
        let original = DriftState {
            s_plus: 1.5,
            s_minus: 2.3,
            mean_abs_residual: 0.42,
            residual_sign_ewma: -0.3,
            steps_since_reset: 123,
        };

        let json = serde_json::to_string(&original).expect("serialisation failed");
        let restored: DriftState = serde_json::from_str(&json).expect("deserialisation failed");

        assert!((restored.s_plus - original.s_plus).abs() < f64::EPSILON);
        assert!((restored.s_minus - original.s_minus).abs() < f64::EPSILON);
        assert!((restored.mean_abs_residual - original.mean_abs_residual).abs() < f64::EPSILON);
        assert!((restored.residual_sign_ewma - original.residual_sign_ewma).abs() < f64::EPSILON);
        assert_eq!(restored.steps_since_reset, original.steps_since_reset);
    }
}
