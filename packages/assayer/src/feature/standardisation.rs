// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(dead_code)]

//! Feature standardisation for the Bayesian model.
//!
//! Provides online standardisation of feature vectors using Welford's algorithm
//! for running mean and variance. Features are standardised before being fed
//! to the model to improve numerical stability and convergence.
//!
//! # Standardisation Pipeline
//!
//! 1. Features are assembled (raw values)
//! 2. Interactions are computed from raw values
//! 3. Full vector is standardised: `φ̂_j = (φ_j - μ̄_j) / (√v̄_j + ε)`
//!
//! # Tests
//!
//! Crate-level tests: `src/tests/standardisation.rs`
//!
//! # Cross-References
//!
//! - (´sec:standardisation:online´) — why the features are standardised, where
//!   the statistics live, and what feeds them
//! - (´dec:vector:statistic-timing´) — that the statistics are observed at
//!   assessment time and their continuing update applied at label time

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::feature::dimension_map::DimensionMap;
use crate::feature::permutation::PositionKey;

// ═══════════════════════════════════════════════════════════════════════════════
// Standardisation Phase
// ═══════════════════════════════════════════════════════════════════════════════

/// Where the cold full-vector standardisation ramp stands.
///
/// The three values are readings of one quantity, the accepted-observation
/// count, rather than a second piece of state that could disagree with it
/// (´dec:health:standardisation-phase-reported´). `WaitingForInit` is a count
/// of zero and the class priors exactly; `Transitioning` is a positive count
/// below the horizon and a mixture of the priors and what has been observed;
/// `InService` is the horizon reached, with no prior mass left
/// (´alg:standardisation:batch-initialisation´).
///
/// The phase is stored and reported rather than inferred from the moments it
/// describes. Counting how many variances sit at the floor estimates it and
/// gets it wrong in both directions: a position whose observed variance is
/// genuinely small is indistinguishable from one the ramp has not yet moved,
/// and the count says nothing at all about how far along the ramp has come.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum StandardisationPhase {
    /// No accepted observation yet. Standardising against the class priors.
    #[default]
    WaitingForInit,
    /// Between the first accepted observation and the horizon. Standardising
    /// against a mixture of the priors and the accepted sample.
    Transitioning,
    /// The horizon is reached. Standardising against the empirical moments of
    /// the accepted sample, with no prior mass remaining.
    InService,
}

impl StandardisationPhase {
    /// Returns `true` if standardisation is using empirical statistics alone.
    #[must_use]
    pub const fn is_in_service(&self) -> bool {
        matches!(self, Self::InService)
    }

    /// Returns `true` while the ramp is between its first accepted observation
    /// and its horizon.
    #[must_use]
    pub const fn is_transitioning(&self) -> bool {
        matches!(self, Self::Transitioning)
    }

    /// Reads the phase off an accepted count and the ramp's horizon.
    ///
    /// This is the whole of the derivation, and it is written once so that no
    /// caller has its own copy to disagree with: a count at or past the
    /// horizon is in service, a count of zero is waiting, and everything
    /// between is a transition (´alg:standardisation:batch-initialisation´).
    /// A horizon of zero is in service at once, because a ramp with no
    /// observations to take has no prior mass left to retire.
    #[must_use]
    pub const fn from_progress(accepted: usize, horizon: usize) -> Self {
        if accepted >= horizon {
            Self::InService
        } else if accepted == 0 {
            Self::WaitingForInit
        } else {
            Self::Transitioning
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Feature Class
// ═══════════════════════════════════════════════════════════════════════════════

/// Classification of feature types for standardisation priors.
///
/// Each variant defines the prior (μ, σ²) a position of that class starts at.
/// Those moments are the cold ramp's base — what the accepted sample is mixed
/// against until the horizon retires the last of them
/// (´alg:standardisation:batch-initialisation´) — and are also what a
/// lifecycle event gives each position it adds.
///
/// # Cross-References
///
/// - (´tab:standardisation:class-priors´) — the prior moments each class
///   starts a position at
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FeatureClass {
    /// Constant bias term (always 1.0). Not standardised.
    Bias,
    /// Z-score features (approximately N(0,1)).
    ZScore,
    /// CUSUM features (non-negative, heavy-tailed).
    Cusum,
    /// Rate or fraction features (in [0,1]).
    RateOrFraction,
    /// Log-scaled features (e.g., log(1+x)).
    LogScaled,
    /// Binary indicators (0 or 1).
    Binary,
    /// Hashed categorical features (sparse, bounded variance).
    HashedCategorical,
    /// Cyclic features (in [0,1] with wrapped boundary).
    Cyclic,
    /// Interaction product features.
    InteractionProduct,
    /// Sentinel occupancy indicator (0 or 1).
    OccupancyIndicator,
    /// Competitive cell indicator (0 or 1).
    CompetitiveCellIndicator,
    /// Compressed outcome axis EWMA.
    OutcomeAxisCompressed,
    /// Raw outcome axis EWMA.
    OutcomeAxisRaw,
    /// Outcome axis stability indicator.
    OutcomeAxisStability,
    /// Identity measurement alarm feature.
    IdentityMeasurementAlarm,
    /// Identity measurement suspicion feature.
    IdentityMeasurementSuspicion,
    /// Identity measurement volatility feature.
    IdentityMeasurementVolatility,
    /// Cross-dimension max aggregate feature.
    CrossDimensionMaxAggregate,
    /// Cross-dimension binary feature.
    CrossDimensionBinary,
}

impl FeatureClass {
    /// Returns the prior (mean, variance) for this feature class.
    ///
    /// The cold ramp's base moments and the moments a lifecycle extension
    /// gives a new position (´tab:standardisation:class-priors´).
    ///
    /// # Cross-References
    ///
    /// - (´tab:standardisation:class-priors´) — the prior values per class
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)] // Justified: idiomatic &self on enum
    #[allow(clippy::match_same_arms)] // Justified: each variant has distinct semantics; shared values are coincidental
    pub const fn prior(&self) -> (f64, f64) {
        // Values from (´tab:standardisation:class-priors´).
        match self {
            Self::Bias => (1.0, 0.0),                      // Constant, never standardised
            Self::ZScore => (0.0, 1.0),                    // Approximately N(0,1)
            Self::Cusum => (2.0, 25.0),                    // Non-negative, heavy-tailed
            Self::RateOrFraction => (0.3, 0.05),           // Rates in [0,1]
            Self::LogScaled => (3.0, 4.0),                 // log(1+x), typical range ~1–6
            Self::Binary => (0.5, 0.25),                   // Bernoulli(0.5)
            Self::HashedCategorical => (0.0, 0.25),        // Sparse, bounded variance
            Self::Cyclic => (0.0, 0.5),                    // Wrapped boundary
            Self::InteractionProduct => (0.0, 1.0),        // Product of standardised
            Self::OccupancyIndicator => (0.5, 0.25),       // Bernoulli(0.5)
            Self::CompetitiveCellIndicator => (0.5, 0.25), // Bernoulli(0.5)
            Self::OutcomeAxisCompressed => (0.0, 0.25),    // Compressed EWMA
            Self::OutcomeAxisRaw => (0.0, 1.0),            // Raw EWMA
            Self::OutcomeAxisStability => (0.1, 0.05),     // Stability in [0,1]
            Self::IdentityMeasurementAlarm => (0.0, 1.0),
            Self::IdentityMeasurementSuspicion => (0.1, 0.05),
            Self::IdentityMeasurementVolatility => (0.1, 0.05),
            Self::CrossDimensionMaxAggregate => (0.0, 1.0),
            Self::CrossDimensionBinary => (0.5, 0.25),
        }
    }

    /// Returns the prior mean for this feature class.
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)] // Justified: idiomatic &self on enum
    pub const fn prior_mean(&self) -> f64 {
        self.prior().0
    }

    /// Returns the prior variance for this feature class.
    #[must_use]
    #[allow(clippy::trivially_copy_pass_by_ref)] // Justified: idiomatic &self on enum
    pub const fn prior_variance(&self) -> f64 {
        self.prior().1
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Standardisation Configuration
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration for standardisation operations.
///
/// # Cross-References
///
/// - (´alg:standardisation:label-time-procedure´) — the update procedure these
///   parameters are consumed by
/// - (´tab:config:standardisation´) — the defaults and constraints tabulated
///   for each of them
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct StandardisationConfig {
    /// Decay factor for running statistics (`γ_std`). Default: 0.9998
    /// (´tab:config:standardisation´).
    pub gamma_std: f64,
    /// Epsilon for numerical stability in division. Default: 1e-8.
    pub epsilon: f64,
    /// Clip threshold for extreme values (`n_clip` × σ). Default: 10.0
    /// (´tab:config:standardisation´).
    pub n_clip: f64,
    /// Minimum variance floor to prevent division issues. Default: 1e-4
    /// (´tab:config:standardisation´).
    pub v_floor: f64,
    /// Cold prior-mass ramp horizon `N_init`, in accepted raw assessment
    /// vectors. Default: 100 (´tab:config:standardisation´).
    ///
    /// This is the horizon over which prior mass falls to zero — each accepted
    /// observation retires exactly one `N_init`-th of it — and not a gate the
    /// ramp waits behind before publishing anything
    /// (´alg:standardisation:batch-initialisation´).
    pub n_init: u32,
    /// Per-Sentinel bootstrap sample count `N_boot`. Default: 100
    /// (´alg:standardisation:sentinel-bootstrap´).
    pub n_boot: u32,
    /// Bootstrap blend factor `α_boot`, in (0, 1]. Default: 0.8
    /// (´alg:standardisation:sentinel-bootstrap´).
    pub alpha_boot: f64,
}

impl Default for StandardisationConfig {
    fn default() -> Self {
        Self {
            gamma_std: 0.9998,
            epsilon: 1e-8,
            n_clip: 10.0,
            v_floor: 1e-4,
            n_init: 100,
            n_boot: 100,
            alpha_boot: 0.8,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Standardisation Functions
// ═══════════════════════════════════════════════════════════════════════════════

/// Standardises a feature vector in place.
///
/// Applies the transformation: `φ̂_j = (φ_j - μ̄_j) / (√v̄_j + ε)`
///
/// The bias term (index 0) is excluded from standardisation.
///
/// # Arguments
///
/// * `phi` — The feature vector to standardise (modified in place)
/// * `means` — Running mean estimates per feature
/// * `variances` — Running variance estimates per feature
/// * `epsilon` — Small constant for numerical stability
///
/// # Panics
///
/// Debug panics if `phi.len() != means.len()` or `phi.len() != variances.len()`.
pub fn standardise_in_place(phi: &mut [f64], means: &[f64], variances: &[f64], epsilon: f64) {
    debug_assert_eq!(phi.len(), means.len(), "phi and means length mismatch");
    debug_assert_eq!(phi.len(), variances.len(), "phi and variances length mismatch");

    // Skip index 0 (bias term)
    for j in 1..phi.len() {
        let mu = means[j];
        // Defence-in-depth: .max(0.0) prevents sqrt of negative if EWMA
        // drift pushes variance below zero, which the floor applied at the end
        // of the update should already have prevented
        // (´alg:standardisation:label-time-procedure´).
        let sigma = variances[j].max(0.0).sqrt() + epsilon;
        phi[j] = (phi[j] - mu) / sigma;
    }
}

/// Updates running standardisation statistics from a single observation.
///
/// Implements the EWMA update of (´alg:standardisation:label-time-procedure´):
/// 1. Clip extreme values: `|x - μ̄| > n_clip · √v̄` → clamp (update only)
/// 2. Update mean: `μ̄ ← γ · μ̄ + (1-γ) · x_clipped`
/// 3. Update variance: `v̄ ← γ · v̄ + (1-γ) · (x_clipped - μ̄_old)²`
/// 4. Apply variance floor: `v̄ ← max(v̄, v_floor)`
///
/// The bias term (index 0) is excluded from updates.
///
/// # Arguments
///
/// * `phi_raw` — The raw (unstandardised) feature vector
/// * `means` — Running mean estimates (modified in place)
/// * `variances` — Running variance estimates (modified in place)
/// * `config` — Standardisation configuration (`gamma_std`, `n_clip`, `v_floor`)
///
/// # Cross-References
///
/// - (´alg:standardisation:label-time-procedure´) — the clip, the mean and
///   variance update, and the variance floor the `.max(0.0)` guard backs up
#[allow(clippy::suboptimal_flops)] // Justified: the formula mirrors the procedure's equations (´alg:standardisation:label-time-procedure´) for auditability
pub fn update_standardisation(phi_raw: &[f64], means: &mut [f64], variances: &mut [f64], config: &StandardisationConfig) {
    debug_assert_eq!(phi_raw.len(), means.len(), "phi_raw and means length mismatch");
    debug_assert_eq!(phi_raw.len(), variances.len(), "phi_raw and variances length mismatch");

    let gamma = config.gamma_std;
    let n_clip = config.n_clip;
    let v_floor = config.v_floor;

    // Skip index 0 (bias term)
    for j in 1..phi_raw.len() {
        let x = phi_raw[j];
        let mu = means[j];
        // Defence-in-depth: .max(0.0) prevents sqrt of negative if EWMA
        // drift pushes variance below zero, behind the floor the procedure
        // applies (´alg:standardisation:label-time-procedure´).
        let sigma = variances[j].max(0.0).sqrt();

        // Step 2 of (´alg:standardisation:label-time-procedure´): clip for update only.
        let x_clipped = x.clamp(mu - n_clip * sigma, mu + n_clip * sigma);

        // Step 5a: update mean.
        let new_mu = gamma * mu + (1.0 - gamma) * x_clipped;

        // Step 5b: update variance using (x_clipped - μ̄_old)² per spec.
        let delta = x_clipped - mu;
        let new_var = gamma * variances[j] + (1.0 - gamma) * delta * delta;

        means[j] = new_mu;
        // Step 6: variance floor.
        variances[j] = new_var.max(v_floor);
    }
}

/// Extends standardisation statistics for new feature positions.
///
/// Appends prior values for each new feature class to the means and variances vectors.
///
/// # Arguments
///
/// * `means` — Running mean estimates (extended in place)
/// * `variances` — Running variance estimates (extended in place)
/// * `classes` — Current feature classes (extended in place)
/// * `new_classes` — Feature classes for new positions
/// * `model_dimension` — The model dimension the vectors extend alongside;
///   the post-condition that every position carries a class
///   (´req:standardisation:class-assignment´) is asserted against it
pub fn extend_standardisation(
    means: &mut Vec<f64>,
    variances: &mut Vec<f64>,
    classes: &mut Vec<FeatureClass>,
    new_classes: &[FeatureClass],
    model_dimension: usize,
) {
    for &class in new_classes {
        let (mu, var) = class.prior();
        means.push(mu);
        variances.push(var);
        classes.push(class);
    }
    assert_eq!(
        means.len(),
        model_dimension,
        "standardisation extension desynchronised from the model dimension"
    );
}

/// Compacts standardisation statistics to retain only specified indices.
///
/// Creates new vectors containing only the positions in `keep_indices`.
///
/// # Arguments
///
/// * `means` — Running mean estimates (replaced in place)
/// * `variances` — Running variance estimates (replaced in place)
/// * `classes` — Feature classes (replaced in place)
/// * `keep_indices` — Indices to retain (must be sorted and within bounds)
pub fn compact_standardisation(
    means: &mut Vec<f64>,
    variances: &mut Vec<f64>,
    classes: &mut Vec<FeatureClass>,
    keep_indices: &[usize],
) {
    let new_means: Vec<f64> = keep_indices.iter().map(|&i| means[i]).collect();
    let new_variances: Vec<f64> = keep_indices.iter().map(|&i| variances[i]).collect();
    let new_classes: Vec<FeatureClass> = keep_indices.iter().map(|&i| classes[i]).collect();

    *means = new_means;
    *variances = new_variances;
    *classes = new_classes;
}

/// Initialises standardisation statistics from class priors.
///
/// Creates mean and variance vectors with prior values for each feature class.
/// This is what cold construction publishes and what the cold ramp takes as
/// its base: step 1 of (´alg:standardisation:batch-initialisation´) publishes
/// these moments, the phase `WaitingForInit` and a count of zero.
///
/// # Arguments
///
/// * `classes` — Feature classes for each position
///
/// # Returns
///
/// A tuple of (means, variances) vectors.
#[must_use]
pub fn init_from_priors(classes: &[FeatureClass]) -> (Vec<f64>, Vec<f64>) {
    let mut means = Vec::with_capacity(classes.len());
    let mut variances = Vec::with_capacity(classes.len());

    for &class in classes {
        let (mu, var) = class.prior();
        means.push(mu);
        variances.push(var);
    }

    (means, variances)
}

// TODO ´todo:code:add-expand-signal-classes-and´: add `expand_signal_classes()` and
// `assign_feature_classes(dim_map, signal_classes)` so cold-start and
// lifecycle priors come from the DimensionMap, the slot offset convention
// (´conv:extraction:offsets´), and declared SignalShape values instead of
// neutral ZScore defaults — the class being derived from the map is what the
// decision requires (´dec:vector:derived-class-priors´).

// ═══════════════════════════════════════════════════════════════════════════════

// ═══════════════════════════════════════════════════════════════════════════════
// Class assignment
// ═══════════════════════════════════════════════════════════════════════════════

/// The eight base identity features' classes, in block order.
const IDENTITY_BASE_CLASSES: [FeatureClass; 8] = [
    FeatureClass::RateOrFraction,
    FeatureClass::RateOrFraction,
    FeatureClass::LogScaled,
    FeatureClass::IdentityMeasurementAlarm,
    FeatureClass::IdentityMeasurementAlarm,
    FeatureClass::IdentityMeasurementSuspicion,
    FeatureClass::IdentityMeasurementVolatility,
    FeatureClass::Binary,
];

/// The eight cross-dimension aggregates' classes, in block order.
///
/// Offsets three and seven are `RateOrFraction` in the reference table and are
/// carried here as the lifecycle handler already pushes them, under the
/// standing notice at that handler rather than corrected in passing.
const IDENTITY_CROSS_CLASSES: [FeatureClass; 8] = [
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionMaxAggregate,
    FeatureClass::CrossDimensionBinary,
    FeatureClass::CrossDimensionMaxAggregate,
];

/// The per-axis identity triple's classes, in block order.
const IDENTITY_AXIS_CLASSES: [FeatureClass; 3] = [
    FeatureClass::OutcomeAxisCompressed,
    FeatureClass::OutcomeAxisRaw,
    FeatureClass::OutcomeAxisStability,
];

/// Returns the class of one aggregate-block position.
///
/// The block is fifteen summaries and most of them are z-scores, but six are
/// not: the four concordance features are fractions above a threshold and
/// coverage is a fraction of the registered fleet, while the maximum
/// cumulative sum is a cumulative sum (´tab:standardisation:class-priors´).
/// Classing all fifteen as z-scores puts the maximum cumulative sum five times
/// too loud against an isotropic model prior.
const fn aggregate_class(offset: usize) -> FeatureClass {
    match offset {
        7 | 8 | 9 | 10 | 13 => FeatureClass::RateOrFraction,
        14 => FeatureClass::Cusum,
        _ => FeatureClass::ZScore,
    }
}

/// Derives every position's standardisation class from the map and the schema.
///
/// The requirement names three authorities and this is all of them
/// (´req:standardisation:class-assignment´): the map for the block and the
/// slot, the fixed extraction layout for the class within a slot, and the
/// signal schema for the signal block, which the caller has already expanded
/// with `expand_signal_classes`. The map's own position enumeration is what
/// supplies the first, so the derivation follows the canonical layout by
/// construction rather than by a second copy of it.
///
/// The Sentinel extraction's own sub-groups are not classified here. Their
/// classification is the standing notice at the registration handler, and
/// treating the extraction block as z-scores is what this function preserves
/// rather than what it decides. The per-axis outcome-memory pairs beside them
/// do carry their own classes, because the lifecycle already assigns those.
#[must_use]
pub fn assign_feature_classes(dim_map: &DimensionMap, signal_classes: &[FeatureClass]) -> Vec<FeatureClass> {
    dim_map
        .position_keys()
        .into_iter()
        .map(|key| match key {
            PositionKey::Bias => FeatureClass::Bias,
            PositionKey::Aggregate(offset) => aggregate_class(offset),
            PositionKey::IdentityBase(_, offset) => IDENTITY_BASE_CLASSES.get(offset).copied().unwrap_or(FeatureClass::ZScore),
            PositionKey::IdentityAxis(_, _, within) => IDENTITY_AXIS_CLASSES.get(within).copied().unwrap_or(FeatureClass::ZScore),
            PositionKey::IdentityCross(offset) => IDENTITY_CROSS_CLASSES.get(offset).copied().unwrap_or(FeatureClass::ZScore),
            PositionKey::Signal(offset) => signal_classes.get(offset).copied().unwrap_or(FeatureClass::ZScore),
            PositionKey::SentinelOccupancy(_) => FeatureClass::OccupancyIndicator,
            PositionKey::SentinelFixed(_, _) | PositionKey::SentinelBatchContext(_) => FeatureClass::ZScore,
            PositionKey::SentinelAxisPair(_, _, half) => {
                if half == 0 {
                    FeatureClass::OutcomeAxisCompressed
                } else {
                    FeatureClass::OutcomeAxisRaw
                }
            }
            PositionKey::Interaction(_) => FeatureClass::InteractionProduct,
            PositionKey::Competitive(_, _) => FeatureClass::CompetitiveCellIndicator,
        })
        .collect()
}
