// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Assayer configuration types — the contracts and boundaries layer, whose
//! configuration is Core-only (´dec:construction:core-only-config´).
//!
//! This module defines the full configuration hierarchy for the Assayer:
//!
//! - [`AssayerConfig`] — Top-level configuration hierarchy
//! - [`ModelConfig`] — Core model hyperparameters (λ, γ, etc.)
//! - [`EligibilityPolicy`] — Core label eligibility policy
//! - [`TemporalConfig`] — Time-decay rates per entity class
//! - [`HibernationConfig`] — Hibernation archive lifetime
//! - [`LedgerConfig`] — Outcome ledger parameters
//! - [`StandardisationConfig`] — Feature standardisation parameters
//! - [`CholeskyConfig`] — Cholesky factorisation and recomputation
//! - [`ConcordanceConfig`] — Concordance tracking parameters
//! - [`ConvergenceThresholdConfig`] — Platt calibration convergence thresholds
//! - [`InfrastructureConfig`] — Buffer capacities and timeouts
//! - [`PersistenceConfig`] — Checkpoint and journal paths
//! - [`BlendStatisticsConfig`] — Blend tracking window configuration
//!
//! External configs (re-exported from their defining modules):
//! - [`PlattConfig`] — Platt calibration fitting parameters
//! - [`SchurConfig`] — Schur complement computation parameters
//!
//! # Cross-References
//!
//! - Configuration is Core-only (´dec:construction:core-only-config´)
//! - Parameter specifications (´chap:spec:configuration´)

use std::path::PathBuf;
use std::time::Duration;

// Re-export external configs for unified access
pub use crate::feature::standardisation::StandardisationConfig;
pub use crate::model::marginalise::SchurConfig;
pub use crate::risk::calibration::PlattConfig;

// ═══════════════════════════════════════════════════════════════════════════════
// AssayerConfig — Top-Level Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Top-level configuration for the Assayer.
///
/// All sub-configurations implement `Default` with spec-compliant values.
/// Derivation-layer and
/// rendering-layer parameters do not live here: configuration is
/// Core-only (´dec:construction:core-only-config´), the display
/// configuration is the rendering call's argument
/// (´sig:rendering:contract´), and the ambiguity gauge reads its
/// posture per call (´sig:rendering:ambiguity-gauge´).
///
/// # Example
///
/// ```ignore
/// use torrust_assayer::AssayerConfig;
///
/// let mut config = AssayerConfig {
///     instance_id: "prod-assayer-1".to_owned(),
///     ..Default::default()
/// };
/// // Host-set with no default: declare the command channel's capacity.
/// config.infrastructure.command_channel_capacity = Some(64);
/// ```
///
/// # Cross-References
///
/// - Configuration is Core-only (´dec:construction:core-only-config´)
/// - Parameter specifications (´chap:spec:configuration´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssayerConfig {
    /// Human-readable instance identifier (used in thread names and logs).
    pub instance_id: String,

    /// Core model hyperparameters (λ, γ, etc.).
    pub model: ModelConfig,

    /// Core label eligibility policy.
    #[cfg_attr(feature = "serde", serde(default))]
    pub eligibility: EligibilityPolicy,

    /// Time-decay rates per entity class.
    pub temporal: TemporalConfig,

    /// Hibernation archive lifetime.
    pub hibernation: HibernationConfig,

    /// Outcome ledger parameters.
    pub ledger: LedgerConfig,

    /// Feature standardisation parameters.
    pub standardisation: StandardisationConfig,

    /// Cholesky factorisation and recomputation parameters.
    pub cholesky: CholeskyConfig,

    /// Concordance tracking parameters.
    pub concordance: ConcordanceConfig,

    /// Health-monitoring reading thresholds.
    #[cfg_attr(feature = "serde", serde(default))]
    pub monitoring: MonitoringConfig,

    /// Identity convergence stage thresholds.
    pub convergence: ConvergenceThresholdConfig,

    /// Buffer capacities and timeouts.
    pub infrastructure: InfrastructureConfig,

    /// Persistence configuration. `None` disables persistence.
    ///
    /// Persisting needs the `serde` feature, which is what compiles the
    /// checkpoint and journal codecs: a configuration naming directories in a
    /// build without it is refused at construction.
    pub persistence: Option<PersistenceConfig>,

    /// Platt calibration fitting parameters.
    pub platt: PlattConfig,

    /// Schur complement computation parameters.
    pub schur: SchurConfig,

    /// Blend statistics tracking configuration.
    pub blend_statistics: BlendStatisticsConfig,
}

impl Default for AssayerConfig {
    fn default() -> Self {
        Self {
            instance_id: "assayer".to_owned(),
            model: ModelConfig::default(),
            eligibility: EligibilityPolicy::default(),
            temporal: TemporalConfig::default(),
            hibernation: HibernationConfig::default(),
            ledger: LedgerConfig::default(),
            standardisation: StandardisationConfig::default(),
            cholesky: CholeskyConfig::default(),
            concordance: ConcordanceConfig::default(),
            monitoring: MonitoringConfig::default(),
            convergence: ConvergenceThresholdConfig::default(),
            infrastructure: InfrastructureConfig::default(),
            persistence: None,
            platt: PlattConfig::default(),
            schur: SchurConfig::default(),
            blend_statistics: BlendStatisticsConfig::default(),
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// MonitoringConfig — Health Reading Thresholds
// ═══════════════════════════════════════════════════════════════════════════════

/// Health-monitoring parameters gathered by the monitoring configuration table
/// (´tab:config:monitoring´).
///
/// These values classify and aggregate diagnostic readings. They do not alter
/// assessment, learning, alarm production, queueing, or maturity state. The
/// synchronisation coefficient is the one operational exception in the table:
/// multiplying it by model dimension gives the reported-error threshold used by
/// the already-specified adaptive recomputation cadence.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MonitoringConfig {
    /// Drift CUSUM noise allowance `κ_drift`.
    pub kappa_drift: f64,
    /// Drift CUSUM accumulator threshold `h`.
    pub h_threshold: f64,
    /// Minimum positive and negative class count for AUC.
    pub min_auc_class_count: usize,
    /// Number of newest labels in the recent discrimination reading.
    pub recent_discrimination_window: usize,
    /// Synchronisation-error threshold coefficient, multiplied by model dimension.
    pub sync_error_threshold_per_dimension: f64,
    /// Minimum labels before per-entity concordance may be flagged.
    pub min_concordance_labels: u64,
    /// Deficit below aggregate AUC that flags per-entity concordance.
    pub concordance_deficit_threshold: f64,
    /// Alarm value at or above which an eventual benign outcome disagrees.
    pub strong_alarm_threshold: f64,
    /// Alarm value at or below which an eventual adverse outcome disagrees.
    pub quiet_alarm_threshold: f64,
    /// Allowance subtracted from every alarm-outcome Page update.
    pub alarm_outcome_noise_allowance: f64,
    /// Minimum feature variance accepted by the feature-stability reading.
    pub feature_stability_threshold: f64,
    /// Maximum noise influence at which a measurement cell is mature.
    pub maturity_threshold: f64,
    /// Minimum recent eligible labels at which a Ledger cell is adequate.
    pub resolution_adequacy_threshold: u64,
    /// EWMA rate reserved for cross-layer feedback latency.
    pub feedback_latency_ewma_rate: f64,
    /// Recent eligible-label floor for a material Ledger cell.
    pub ledger_materiality_threshold: u64,
    /// Attenuation floor reserved for the per-assessment materiality predicate.
    pub attenuation_materiality_floor: f64,
}

impl MonitoringConfig {
    /// Synchronisation threshold for a model of `dimension` coordinates.
    #[must_use]
    #[allow(clippy::cast_precision_loss)] // Justified: model dimensions are far below f64's exact integer range.
    pub fn synchronisation_error_threshold(&self, dimension: usize) -> f64 {
        self.sync_error_threshold_per_dimension * dimension as f64
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            kappa_drift: 0.1,
            h_threshold: 10.0,
            min_auc_class_count: 20,
            recent_discrimination_window: 500,
            sync_error_threshold_per_dimension: 1e-6,
            min_concordance_labels: 5,
            concordance_deficit_threshold: 0.25,
            strong_alarm_threshold: 3.0,
            quiet_alarm_threshold: 0.5,
            alarm_outcome_noise_allowance: 0.02,
            feature_stability_threshold: 0.001,
            maturity_threshold: 0.1,
            resolution_adequacy_threshold: 20,
            feedback_latency_ewma_rate: 0.99,
            ledger_materiality_threshold: 100,
            attenuation_materiality_floor: 0.25,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ModelConfig — Core Model Hyperparameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Fixed anchor model dimensionality `p_a = 15`.
///
/// The anchor model always uses exactly 15 features (the anchor
/// projection). This is not configurable — the anchor operates on a fixed
/// fifteen-dimensional vector, is never extended and never marginalised,
/// and the features are individually specified (´def:risk:anchor-model´).
///
/// ´const:assayer:anchor-model-arity´ (´alg:const:count´)
/// ´const:assayer:anchor-model-arity-count-15´
pub const P_A: usize = 15;

/// Core model hyperparameters.
///
/// These control the Bayesian linear model behaviour: prior precision,
/// decay rates, leverage ceiling, and initial posture estimates.
///
/// The anchor model uses the same `lambda_prior` as operational and
/// sister models (´def:risk:model-triple´). Its dimension is fixed at
/// [`P_A`] (´def:risk:anchor-model´). Drift detection parameters live in
/// [`DriftConfig`](crate::health::DriftConfig) (´tab:config:monitoring´).
///
/// # Cross-References
///
/// - Core risk model parameters (´tab:config:risk-model´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ModelConfig {
    /// Prior precision λ for all models (operational, sister, anchor).
    /// Default: 0.1.
    pub lambda_prior: f64,

    /// Minimum precision floor `λ_floor`. Default: 0.001.
    pub lambda_floor: f64,

    /// Leverage ceiling `c_lev`. Default: 5.0.
    pub c_leverage: f64,

    /// Maximum observation weight `w_ceil`. Default: 100.0.
    pub w_ceiling: f64,

    /// Operational model decay rate `γ_opr` (per label). Default: 0.9995.
    pub gamma_opr: f64,

    /// Sister (inheritance) model decay rate `γ_inh` (per label). Default: 0.9998.
    pub gamma_inh: f64,

    /// Global valence scaling decay `γ_κ` (per label). Default: 0.9998 (´tab:config:risk-model´).
    pub gamma_kappa: f64,

    /// Initial positive-class probability `p⁺_init`. Default: 0.5.
    pub p_plus_init: f64,
}

// Note: kappa_drift and h_drift belong in the drift configuration
// (´tab:config:monitoring´), carried by the health::drift module rather
// than by ModelConfig. The anchor dimension p_a = 15 is fixed by the
// anchor model (´def:risk:anchor-model´) — see the P_A constant. The
// anchor prior precision is the same lambda_prior used by all models
// (´def:risk:model-triple´).

impl Default for ModelConfig {
    fn default() -> Self {
        Self {
            lambda_prior: 0.1,
            lambda_floor: 0.001,
            c_leverage: 5.0,
            w_ceiling: 100.0,
            gamma_opr: 0.9995,
            gamma_inh: 0.9998,
            gamma_kappa: 0.9998,
            p_plus_init: 0.5,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// EligibilityPolicy — Core Label Eligibility
// ═══════════════════════════════════════════════════════════════════════════════

/// Core policy for label eligibility decisions.
///
/// This construction-time policy belongs to the Core, not to per-channel
/// derivation policy. It controls the one configurable row in the label
/// eligibility table: whether a failed challenge may be treated as
/// unconfounded evidence about the entity.
///
/// # Cross-References
///
/// - The eligibility policy (´tab:config:eligibility´)
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EligibilityPolicy {
    /// Whether non-ground-truth Challenge labels are unconfounded evidence for Core training.
    ///
    /// Default: `true` (´tab:config:eligibility´).
    pub challenge_fail_is_unconfounded: bool,
}

impl Default for EligibilityPolicy {
    fn default() -> Self {
        Self {
            challenge_fail_is_unconfounded: true,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// TemporalConfig — Time-Decay Rates
// ═══════════════════════════════════════════════════════════════════════════════

/// Time-decay rates per entity class (hourly γ values).
///
/// These control how quickly different parts of the model forget
/// old information when no new data arrives.
///
/// # Cross-References
///
/// - Temporal parameters (´tab:config:temporal´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
// Justified: the field names match the tabulated notation (´tab:config:temporal´).
#[allow(clippy::struct_field_names)]
pub struct TemporalConfig {
    /// Core model time-decay rate `γ_t` (hourly). Default: 0.9999.
    pub gamma_t_core: f64,

    /// Ledger time-decay rate `γ_t_ledger` (hourly). Default: 0.999.
    pub gamma_t_ledger: f64,

    /// Identity dimension time-decay rate `γ_t_identity` (hourly). Default: 0.998.
    pub gamma_t_identity: f64,
}

// Note: gamma_{q,t} (challenge-effectiveness time-decay) belongs to the
// host-owned Companion Tracker (´tab:config:companion´), not Core temporal config.
// Host-composition helpers seed tracker state from `RewardParameters::gamma_q_t`.

impl Default for TemporalConfig {
    fn default() -> Self {
        Self {
            gamma_t_core: 0.9999,
            gamma_t_ledger: 0.999,
            gamma_t_identity: 0.998,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HibernationConfig — Hibernation Archive Lifetime
// ═══════════════════════════════════════════════════════════════════════════════

/// How long a hibernating entity's archived parameter block stays usable.
///
/// Hibernation stores a deregistered Sentinel's or outcome axis's own
/// parameter block against its identifier, and a re-registration before the
/// store expires re-extends the models with the aged block rather than with
/// the prior (´alg:registry:hibernation´). The algorithm fixes that there is
/// an expiry and leaves its length to the deployment, which is why the figure
/// is a configured one rather than a constant: how long a retired measurement
/// surface's structure is still worth having back is a question about the
/// deployment's own traffic, not about the mathematics.
///
/// The lifetime is wall-clock, and it is read against the persistent clock
/// rather than the label clock, because an archive's storage cost accrues in
/// time whether or not the instance is processing anything
/// (´dec:clock:two-domains´).
///
/// # Cross-References
///
/// - (´alg:registry:hibernation´) — the store the expiry bounds
/// - (´tab:construction:parameters´) — the parameter groups a host may move
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HibernationConfig {
    /// Wall-clock lifetime of an archived parameter block, in days.
    /// Default: 90.
    ///
    /// The ceiling is a year, and it is the elapsed-time clamp's rather than
    /// a preference: every elapsed reading is clamped at
    /// [`MAX_DECAY_HOURS`](crate::types::MAX_DECAY_HOURS)
    /// (´dec:clock:embedded-clamp´), so a lifetime longer than that clamp
    /// could never be reached and would read as an archive that never
    /// expires. The builder refuses it rather than shipping a figure the
    /// clock cannot express.
    pub expiry_days: u32,
}

impl HibernationConfig {
    /// The archive lifetime in hours, which is the unit every elapsed reading
    /// is taken in.
    #[must_use]
    pub fn expiry_hours(&self) -> f64 {
        f64::from(self.expiry_days) * 24.0
    }
}

impl Default for HibernationConfig {
    fn default() -> Self {
        Self { expiry_days: 90 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// LedgerConfig — Outcome Ledger Parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Outcome ledger parameters.
///
/// Controls ledger decay, absence tracking, and garbage collection.
///
/// # Cross-References
///
/// - Ledger parameters (´tab:config:ledger´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LedgerConfig {
    /// Ledger decay factor `λ_l` (per observation). Default: 0.999.
    pub lambda_l: f64,

    /// Number of consecutive absences before the entry is deleted
    /// (´alg:ledger:entry-deletion´). Accepted range: 1 to 255, the width of
    /// the counter the deletion predicate reads. Default: 3.
    pub n_absent: u32,

    /// Garbage collection horizon (days). Default: 60.
    pub gc_horizon_days: u32,

    /// Garbage collection floor (minimum weight). Default: 1e-6.
    pub gc_floor: f64,
}

impl Default for LedgerConfig {
    fn default() -> Self {
        Self {
            lambda_l: 0.999,
            n_absent: 3,
            gc_horizon_days: 60,
            gc_floor: 1e-6,
        }
    }
}

/// Why an absence threshold outside 1 to 255 is refused, stated once so that
/// the builder's range check and the construction-time conversion cannot come
/// to state different ranges (´alg:ledger:entry-deletion´).
pub const N_ABSENT_RANGE_REASON: &str = "must be between 1 and 255: the absence counter is eight bits wide";

// ═══════════════════════════════════════════════════════════════════════════════
// CholeskyConfig — Cholesky Factorisation Parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Cholesky factorisation and recomputation parameters.
///
/// Controls the cadence of the periodic Cholesky recomputation. It carries no
/// regularisation parameters: the factorisation is attempted once and its
/// refusal is a verdict (´dec:posterior:repair-cascade´), and what bounds the
/// least eigenvalue is the spectral floor, whose magnitude is derived per
/// model from the decay rate and the replenishment floor rather than
/// configured here (´dec:posterior:spectral-floor´).
///
/// # Cross-References
///
/// - Core risk model parameters (´tab:config:risk-model´)
/// - Recomputation fires on a counter or on conditioning
///   (´dec:posterior:recomputation-trigger´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CholeskyConfig {
    /// Recomputation cadence (labels). Default: 1000.
    pub n_recompute: u32,

    /// Growth factor `κ_growth` the cheap diagonal ratio must clear, relative
    /// to the ratio the last rebuild recorded, to call for another one.
    /// Default: 2.0.
    ///
    /// This is a multiple rather than a threshold, and the change of kind is
    /// the point. A fixed threshold on the ratio is met permanently once any
    /// dimension rests at the replenishment floor, because the clamp puts the
    /// least diagonal entry there on every label; a multiple of what the last
    /// rebuild measured is reset by every rebuild, so the arm fires at most
    /// once per doubling of the ratio (´dec:posterior:recomputation-trigger´).
    pub kappa_growth_factor: f64,
    // TODO(2026-05-23) ´todo:code:add-configurable´: Add configurable
    // `sync_error_threshold` with default 1e-4 for proactive interval
    // shortening on clean high-sync-error recomputes
    // (´def:config:synchronisation-threshold´).
}

impl Default for CholeskyConfig {
    fn default() -> Self {
        Self {
            n_recompute: 1000,
            kappa_growth_factor: 2.0,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConcordanceConfig — Concordance Tracking Parameters
// ═══════════════════════════════════════════════════════════════════════════════

/// Concordance tracking parameters.
///
/// Controls the adaptive concordance threshold mechanism for cross-Sentinel
/// agreement detection.
///
/// # Cross-References
///
/// - Concordance window sizes and thresholds (´tab:construction:parameters´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConcordanceConfig {
    /// Rolling window capacity. Default: 5000.
    /// TODO ´todo:code:change-the-default-to-10-000´: change the default to 10,000 per-Sentinel
    /// sub-score entries once `ConcordanceTracker::observe` records
    /// one entry per reporting Sentinel instead of one aggregate entry
    /// per assessment.
    pub window_capacity: usize,

    /// Recalibration interval (assessments). Default: 1000.
    pub recalibration_interval: usize,

    /// Target percentile for threshold. Default: 0.80.
    pub percentile: f64,
}

impl Default for ConcordanceConfig {
    fn default() -> Self {
        Self {
            // TODO ´todo:code:update-to-10-000-with-the´: update to 10_000 with the per-Sentinel
            // concordance observation refactor.
            window_capacity: 5000,
            recalibration_interval: 1000,
            percentile: 0.80,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// ConvergenceThresholdConfig — Convergence Thresholds
// ═══════════════════════════════════════════════════════════════════════════════

/// Convergence diagnostic thresholds.
///
/// Defines the thresholds at which the composite reports Platt calibration
/// converged. All thresholds are diagnostic only — they do not affect the
/// system's mathematical behaviour.
///
/// The eight generic Warming, Converging and Converged stage figures this
/// structure used to carry were deleted, and the grounds are the whole of
/// the reason: no path read them, the specification named no stage they
/// belonged to, and a mapping onto either ladder the package actually
/// reports would have been a design decision rather than a repair. The
/// identity ladder's own cuts stay where they are read
/// (´tab:health:identity-stability-cuts´).
///
/// The two Companion challenge figures went in the same change. The
/// sufficiency floor is the host's and now stands beside the tracker that
/// reads it (´dec:challenge:sufficiency-floor-owned-here´); the
/// minimum-samples count named no operation and was deleted rather than
/// moved.
///
/// # Cross-References
///
/// - Convergence is reported and never enforced (´dec:health:reports-never-gates´)
/// - Configuration is Core-only (´dec:construction:core-only-config´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConvergenceThresholdConfig {
    // TODO ´todo:code:add-cold-start-labels-default-30´: add `cold_start_labels` (default 30 eligible
    // labels) and `anchor_emergence_labels` (default 80 eligible
    // labels). `compute_composite_stage` should consume those fields
    // instead of hardcoded total-label thresholds.
    /// Platt `δ_cal` threshold for `Converged` state. Default: 0.01.
    ///
    /// When `last_delta_cal <= platt_converged_delta_cal` and
    /// `refits_completed >= platt_converged_refit_count`, the Platt
    /// calibration is reported as converged. Distinct from
    /// `PlattConfig::delta_cal_threshold` (default 0.1), which governs
    /// drift accumulator reset.
    pub platt_converged_delta_cal: f64,

    /// Minimum Platt refits for `Converged` state. Default: 5.
    pub platt_converged_refit_count: u32,
}

impl Default for ConvergenceThresholdConfig {
    fn default() -> Self {
        Self {
            platt_converged_delta_cal: 0.01,
            platt_converged_refit_count: 5,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// InfrastructureConfig — Buffer Capacities and Timeouts
// ═══════════════════════════════════════════════════════════════════════════════

/// Buffer capacities and timeouts.
///
/// Controls the sizes of internal buffers and various timeout durations.
///
/// # Cross-References
///
/// - Concurrency parameters (´tab:config:concurrency´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct InfrastructureConfig {
    /// Expected peak request rate, per second — the R of the pending-buffer
    /// capacity computation (´tab:config:pending-buffer´). Default: 100
    /// (´def:config:reference-peak-request-rate-100-per-second´).
    pub expected_peak_request_rate: f64,

    /// Expected median labelling latency, in seconds — the L of the
    /// pending-buffer capacity computation. Default: 3,600 (one hour)
    /// (´def:config:reference-label-latency-3600-seconds´).
    pub expected_label_latency_secs: u64,

    /// Pending entry expiry horizon (seconds). Default: 86,400 —
    /// twenty-four hours (´tab:config:pending-buffer´).
    pub expiry_horizon_secs: u64,

    /// Precision at which pending entries store feature values.
    /// Default: single, constrained to single or double by the type
    /// (´tab:config:pending-buffer´), (´def:runtime:storage-precision´).
    pub feature_storage_precision: crate::pending::StoragePrecision,

    /// Signal cache capacity. Default: 100,000.
    pub signal_cache_capacity: usize,

    /// Label channel capacity. Default: 10,000.
    pub label_channel_capacity: usize,

    /// Command channel capacity. **Host-set, with no default**: `None`
    /// refuses to build, so the capacity is a declared decision at every
    /// deployment rather than a constant nobody chose. The channel
    /// carries every lifecycle registration, every scheduled checkpoint
    /// and the batch-initialisation completion, and a full channel
    /// surfaces as `LifecycleError::CommandChannelFull`
    /// (´tab:construction:parameters´).
    pub command_channel_capacity: Option<usize>,

    /// Health event channel capacity. Default: 1,000.
    pub health_event_capacity: usize,

    /// Identity observation capacity. Default: 100,000.
    pub identity_observation_capacity: usize,

    /// Identity registration timeout (seconds). Default: 10.
    pub identity_registration_timeout_secs: u64,
}

impl InfrastructureConfig {
    /// Pending buffer capacity: twice the expected peak request rate times
    /// the expected median labelling latency, at least one
    /// (´req:runtime:buffer-capacity´).
    #[must_use]
    pub fn pending_buffer_capacity(&self) -> usize {
        #[allow(clippy::cast_precision_loss)]
        let raw = 2.0 * self.expected_peak_request_rate * self.expected_label_latency_secs as f64;
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let capacity = raw.ceil().max(1.0) as usize;
        capacity
    }
}

/// Default bound on the identity registration acknowledgement wait.
///
/// A liveness bound, not a latency budget: exceeding it returns the error that
/// says the maintenance thread is gone, so it must clear every acknowledgement a
/// live owner would ever send rather than name a target one should meet. Nothing
/// bounds the pass it waits on, so the figure is chosen wide rather than derived
/// (´rem:construction:registration-timeout´). It is the default only — the wait
/// is held to whatever the host configures here.
///
/// ´const:assayer:registration-acknowledgement-timeout´ (´alg:const:seconds´)
/// ´const:assayer:registration-acknowledgement-timeout-seconds-10´
const DEFAULT_IDENTITY_REGISTRATION_TIMEOUT_SECS: u64 = 10;

impl Default for InfrastructureConfig {
    fn default() -> Self {
        Self {
            expected_peak_request_rate: 100.0,
            expected_label_latency_secs: 3_600,
            expiry_horizon_secs: 86_400,
            feature_storage_precision: crate::pending::StoragePrecision::Single,
            signal_cache_capacity: 100_000,
            label_channel_capacity: 10_000,
            // Deliberately unset: the command channel's capacity is
            // host-set with no default.
            command_channel_capacity: None,
            health_event_capacity: 1_000,
            identity_observation_capacity: 100_000,
            identity_registration_timeout_secs: DEFAULT_IDENTITY_REGISTRATION_TIMEOUT_SECS,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// PersistenceConfig — Checkpoint and Journal Paths
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for state persistence.
///
/// Enables checkpoint writing, journal logging, and crash recovery.
/// When `persistence` is `None` on `AssayerConfig`, the system runs
/// without persistence (no journal, no checkpoint scheduler thread).
// TODO ´todo:code:reconcile-the-docs-optional-checkpoint-journal´: Reconcile the docs' optional checkpoint/journal
// directory model with the current `AssayerConfig::persistence =
// Option<PersistenceConfig>` wrapper plus concrete directories here. The
// pair is a periodic checkpoint and a self-contained journal
// (´dec:durability:checkpoint-journal´), and configuration is Core-only
// (´dec:construction:core-only-config´).
///
/// # Cross-References
///
/// - A periodic checkpoint and a self-contained journal (´dec:durability:checkpoint-journal´)
/// - The persistence parameter group (´tab:construction:parameters´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersistenceConfig {
    /// Directory for checkpoint files.
    pub checkpoint_dir: PathBuf,

    /// Directory for journal files.
    pub journal_dir: PathBuf,

    /// Interval between automatic checkpoints. Default: 1 hour.
    pub checkpoint_interval: Duration,
}

impl PersistenceConfig {
    /// Creates a new persistence configuration with the given directories.
    #[must_use]
    pub const fn new(checkpoint_dir: PathBuf, journal_dir: PathBuf) -> Self {
        Self {
            checkpoint_dir,
            journal_dir,
            checkpoint_interval: Duration::from_secs(3600),
        }
    }

    /// Concrete checkpoint file path, derived from `checkpoint_dir`.
    #[must_use]
    pub fn checkpoint_path(&self) -> PathBuf {
        self.checkpoint_dir.join("checkpoint.bin")
    }

    /// Concrete journal file path, derived from `journal_dir`.
    #[must_use]
    pub fn journal_path(&self) -> PathBuf {
        self.journal_dir.join("journal.bin")
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// BlendStatisticsConfig — Blend Tracking Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Blend statistics tracking configuration.
///
/// Controls the rolling window for tracking blend computation statistics
/// used in health monitoring.
///
/// # Cross-References
///
/// - Queries are tiered by cost (´dec:health:tiered-queries´)
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BlendStatisticsConfig {
    /// Rolling window capacity. Default: 5000.
    pub window_capacity: usize,

    /// Publish interval (assessments). Default: 1000.
    pub publish_interval: usize,
}

impl Default for BlendStatisticsConfig {
    fn default() -> Self {
        Self {
            window_capacity: 5000,
            publish_interval: 1000,
        }
    }
}
