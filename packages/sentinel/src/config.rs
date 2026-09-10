// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Configuration types for the Spectral Sentinel.
//!
//! [`SentinelConfig`] controls the measurement parameters of the sentinel.
//!
//! These types are deliberately free of policy concerns (thresholds, actions).
//! The sentinel measures; the host decides what the measurements mean.

use torrust_mudlark::{Accumulator, Inspectable};

// ─── Noise schedule (§ALGO S-18.2, ADR-S-015 §1) ────────────

/// Depth-tiered noise injection schedule.
///
/// Controls how many noise rounds each newly created tracker receives,
/// varying by G-tree depth. Deeper cells are narrower and need fewer
/// rounds to converge, so the schedule tapers with depth.
///
/// # Variants
///
/// - **`Geometric`**: computes rounds as `root × decay^depth`, floored
///   to `min`. Default: `Geometric { root: 450, decay: 0.5, min: 50 }`
///   → `[450, 225, 113, 56, 50, 50, …]`.
///
/// - **`Explicit`**: a hand-specified per-depth vector. Depths beyond
///   the vector length use the last entry. An empty vector disables
///   noise injection entirely.
///
/// # Examples
///
/// ```
/// use torrust_sentinel::NoiseSchedule;
///
/// let geo = NoiseSchedule::geometric(400, 0.5, 100);
/// assert_eq!(geo.rounds_for_depth(0), 400);
/// assert_eq!(geo.rounds_for_depth(1), 200);
/// assert_eq!(geo.rounds_for_depth(2), 100);
/// assert_eq!(geo.rounds_for_depth(10), 100);
///
/// let explicit = NoiseSchedule::Explicit(vec![50, 30, 10]);
/// assert_eq!(explicit.rounds_for_depth(0), 50);
/// assert_eq!(explicit.rounds_for_depth(2), 10);
/// assert_eq!(explicit.rounds_for_depth(99), 10); // clamps to last
/// ```
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum NoiseSchedule {
    /// Geometric decay: `root × decay^depth`, floored to `min`.
    Geometric {
        /// Rounds at depth 0.
        root: u32,
        /// Multiplicative decay per depth level (must be in `(0.0, 1.0]`).
        decay: f64,
        /// Floor — minimum rounds at any depth.
        min: u32,
    },

    /// Per-depth explicit schedule. Depths beyond the vector length
    /// use the last entry. An empty vector disables noise entirely.
    Explicit(Vec<u32>),
}

impl NoiseSchedule {
    /// Convenience constructor for the `Geometric` variant.
    #[must_use]
    pub const fn geometric(root: u32, decay: f64, min: u32) -> Self {
        Self::Geometric { root, decay, min }
    }

    /// Number of noise rounds for a tracker at the given G-tree depth.
    ///
    /// Returns `0` when noise is disabled (empty `Explicit` vector).
    #[must_use]
    pub fn rounds_for_depth(&self, depth: usize) -> u32 {
        match self {
            Self::Geometric { root, decay, min } => {
                // depth is bounded by G-tree depth (≤ 128), safe to truncate.
                #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
                let exp = depth as i32;
                let raw = f64::from(*root) * decay.powi(exp);
                // raw is non-negative (root ≥ 0, decay > 0), safe to truncate.
                #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
                let rounded = raw.round() as u32;
                rounded.max(*min)
            }
            Self::Explicit(v) => {
                if v.is_empty() {
                    return 0;
                }
                // Clamp to last entry for depths beyond the vector.
                v[depth.min(v.len() - 1)]
            }
        }
    }

    /// Whether this schedule produces zero rounds at every depth.
    ///
    /// True for `Explicit(vec![])`, for `Explicit(vec![0, 0, …])`, and for
    /// a `Geometric` schedule whose root and floor are both zero. A
    /// `Geometric` schedule with a positive root still produces rounds at
    /// the shallow depths even when its floor is zero, because the taper
    /// starts from the root and only decays towards the floor.
    #[must_use]
    pub fn is_disabled(&self) -> bool {
        match self {
            Self::Geometric { root, min, .. } => *root == 0 && *min == 0,
            Self::Explicit(v) => v.is_empty() || v.iter().all(|&r| r == 0),
        }
    }

    /// Maximum rounds this schedule can produce (useful for capacity hints).
    #[must_use]
    pub fn max_rounds(&self) -> u32 {
        match self {
            Self::Geometric { root, .. } => *root,
            Self::Explicit(v) => v.iter().copied().max().unwrap_or(0),
        }
    }
}

impl Default for NoiseSchedule {
    /// Default: `Geometric { root: 450, decay: 0.5, min: 50 }`.
    ///
    /// Calibrated for the default forgetting factor λ = 0.99 (§ALGO S-13.1).
    /// At λ = 0.99, b = 16, the worst-case baseline convergence is ~398
    /// rounds (surprise axis), so root = 450 provides ~13% margin.
    /// The floor of 50 covers deep cells where convergence times scale
    /// down with analysis width but remain ~50 rounds at λ = 0.99.
    /// See §ALGO S-A.7 for the empirical derivation.
    fn default() -> Self {
        Self::Geometric {
            root: 450,
            decay: 0.5,
            min: 50,
        }
    }
}

/// Measurement parameters for the sentinel.
///
/// Every field controls *how* the sentinel observes and learns,
/// never *what it thinks* about what it sees.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "V: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct SentinelConfig<V: Accumulator> {
    /// Maximum rank (number of basis vectors) any subspace tracker can use.
    ///
    /// Higher = more expressive model of "normal", but more memory and
    /// SVD cost per observation. The actual rank adapts automatically
    /// and will never exceed `min(suffix_width, max_rank)`.
    ///
    /// Default: `16`
    pub max_rank: usize,

    /// Exponential forgetting factor (λ).
    ///
    /// Controls how fast old observations fade from memory.
    /// - `0.99` = long memory (~69 observations half-life)
    /// - `0.95` = short memory (~14 observations half-life)
    ///
    /// Must be in `(0.0, 1.0)`.
    ///
    /// Default: `0.99`
    pub forgetting_factor: f64,

    /// How often (in observation steps) to reassess the rank of each tracker.
    ///
    /// Rank changes by at most ±1 per evaluation to avoid instability.
    ///
    /// Default: `100`
    pub rank_update_interval: u64,

    /// Cumulative energy threshold for automatic rank adaptation.
    ///
    /// The rank adapts to capture at least this fraction of the total
    /// variance (sum of squared singular values). Lower = fewer dimensions
    /// retained, higher = more faithful representation.
    ///
    /// Must be in `(0.0, 1.0)`.
    ///
    /// Default: `0.90`
    pub energy_threshold: f64,

    /// Numerical stability constant.
    ///
    /// Added to denominators to prevent division by zero.
    ///
    /// Default: `1e-6`
    pub eps: f64,

    /// Slow EWMA decay factor for per-tracker CUSUM reference baselines.
    ///
    /// The CUSUM accumulator detects gradual drift that the fast EWMA
    /// (controlled by `forgetting_factor`) absorbs. The slow EWMA
    /// provides the reference: CUSUM accumulates the gap between the
    /// batch mean score and the slow baseline.
    ///
    /// Must be in `(0.0, 1.0)` and strictly greater than
    /// `forgetting_factor` — a slower memory than the fast baseline.
    ///
    /// Half-life ≈ `ln(2) / ln(1/λ_s)`:
    /// - `0.999` = ~693 steps (default)
    /// - `0.995` = ~139 steps
    ///
    /// Default: `0.999`
    pub cusum_slow_decay: f64,

    /// Slow EWMA decay factor for coordination-tier CUSUM (§ALGO S-13.1).
    ///
    /// Controls the CUSUM reference baseline at the cross-cell
    /// coordination tier. Separated from `cusum_slow_decay` because
    /// the coordination tier may see different batch cadences and the
    /// host may want different drift sensitivity at each tier.
    ///
    /// Must be in `(0.0, 1.0)` and strictly greater than
    /// `forgetting_factor`.
    ///
    /// Default: `0.999`
    pub cusum_coord_slow_decay: f64,

    /// CUSUM noise allowance in slow-baseline σ units.
    ///
    /// Each CUSUM step subtracts `κ_σ · √(slow_variance)` before
    /// accumulating. This absorbs normal noise fluctuations so the
    /// accumulator only grows under sustained elevation.
    ///
    /// - `0.5` = tolerate up to half a slow-σ per step (default)
    /// - `0.0` = no allowance — any positive gap accumulates
    /// - `1.0` = generous allowance — only strong drift accumulates
    ///
    /// Must be ≥ 0.
    ///
    /// Default: `0.5`
    pub cusum_allowance_sigmas: f64,

    /// Outlier clip width in σ units for EWMA baseline updates.
    ///
    /// During each EWMA update, observations beyond
    /// `mean + clip_sigmas · √variance` are rejected to prevent
    /// baseline poisoning.  Higher values accept more of the upper
    /// tail; lower values clip more aggressively.
    ///
    /// Must be > 0.
    ///
    /// Default: `3.0`
    pub clip_sigmas: f64,

    /// Clip-pressure EWMA decay factor (`λ_ρ`).
    ///
    /// Controls how quickly the per-axis clip-pressure estimate adapts
    /// to changing contamination levels.  Higher values = longer memory.
    ///
    /// Half-life ≈ ln(2) / `ln(1/λ_ρ)`:
    /// - `0.95` = ~14 batches (default, §ALGO S-13.1)
    /// - `0.99` = ~69 batches
    ///
    /// Must be in `(0.0, 1.0)`.
    ///
    /// Default: `0.95`
    pub clip_pressure_decay: f64,

    /// Whether to include per-sample scores in reports.
    ///
    /// When `true`, each [`CellReport`](crate::CellReport)
    /// includes a `Vec<SampleScore>` with individual scores for every
    /// observation in the batch. Useful for forensics, expensive for
    /// large batches.
    ///
    /// Default: `false`
    pub per_sample_scores: bool,

    /// Maximum number of competitive analysis cells (§ALGO S-4.1).
    ///
    /// Controls the resource ceiling for the analysis tier. The total
    /// tracker count is bounded by `2 × analysis_k` (Steiner tree
    /// bound, §ALGO S-4.8) — `analysis_k` competitive cells plus at
    /// most `analysis_k` ancestor cells.
    ///
    /// Must be ≥ 1.
    ///
    /// Default: `1024`  (§ALGO S-13.2: `analysis_K`)
    pub analysis_k: usize,

    /// Maximum V-Tree depth at which entries are considered for
    /// competitive selection (§ALGO S-4.1).
    ///
    /// Only V-entries with `v_depth ≤ analysis_depth_cutoff` are
    /// eligible. Deeper entries have not yet proven sufficient
    /// significance. This prevents noise from promoting ephemeral
    /// cells into the analysis set.
    ///
    /// Must be ≥ 0. A value of 0 means only the V-Tree root
    /// (if it is an entry) is eligible — effectively disabling
    /// adaptive selection.
    ///
    /// Default: `6`  (§ALGO S-13.2: `analysis_depth_cutoff`)
    pub analysis_depth_cutoff: usize,

    /// G-V Graph: minimum accumulated intensity before a cell subdivides.
    ///
    /// For count-based observation (Δ=1 per value), this is the number of
    /// observations a cell must receive before subdividing.
    ///
    /// Maps to `torrust_mudlark::Config::split_threshold`.
    ///
    /// Must be > 0.
    ///
    /// Default: `100`  (§ALGO S-13.3: `split_threshold`)
    pub split_threshold: V,

    /// G-V Graph: maximum V-Tree depth at which new splits are allowed.
    ///
    /// Controls how deep the tree can grow. Deeper cells need more
    /// sustained traffic to compete for analysis slots.
    ///
    /// Maps to `torrust_mudlark::Config::depth_create`.
    ///
    /// Must be ≥ 1.
    ///
    /// Default: `3`  (§ALGO S-13.3: `D_create`)
    pub d_create: u32,

    /// G-V Graph: minimum V-Tree depth at which entries become eviction-eligible.
    ///
    /// Must be strictly greater than `d_create`. The gap (`d_evict − d_create`)
    /// is the buffer zone — entries too deep to create children but not yet
    /// deep enough to be evicted.
    ///
    /// Maps to `torrust_mudlark::Config::depth_evict`.
    ///
    /// Must be > `d_create`.
    ///
    /// Default: `6`  (§ALGO S-13.3: `D_evict`)
    pub d_evict: u32,

    /// G-V Graph: hard ceiling on live G-node count.
    ///
    /// Enables dynamic depth control — the graph adjusts depth gates
    /// at runtime to keep the live node count under this ceiling.
    /// The sentinel always operates in budgeted mode.
    ///
    /// Maps to `torrust_mudlark::Config::budget` (wrapped in `Some`).
    ///
    /// Must be > 0 and large enough to satisfy mudlark's headroom
    /// requirement: `budget > max(3^(d_evict − d_create + 1), 2*(d_create − 1))`.
    ///
    /// Default: `100_000`  (§ALGO S-13.3: `budget` / `G_max`)
    pub budget: usize,

    /// Depth-tiered noise injection schedule (§ALGO S-18.2, ADR-S-015 §1).
    ///
    /// Controls how many synthetic noise batches each newly created
    /// tracker receives, varying by G-tree depth. Deeper cells are
    /// narrower and need fewer rounds to converge.
    ///
    /// Default: `Geometric { root: 450, decay: 0.5, min: 50 }`
    pub noise_schedule: NoiseSchedule,

    /// Number of synthetic samples per noise batch (§ALGO S-13.5).
    ///
    /// Each round feeds this many random ±0.5 centred vectors through
    /// the tracker's `observe()` path with `is_noise = true`.
    ///
    /// Must be ≥ 1 when noise is enabled.
    ///
    /// Default: `16`
    pub noise_batch_size: usize,

    /// RNG seed for deterministic noise generation (§ALGO S-13.5).
    ///
    /// `Some(seed)` → reproducible noise across restarts.
    /// `None` → seeded from system entropy.
    ///
    /// Default: `Some(42)`
    pub noise_seed: Option<u64>,

    /// Whether to perform cell warm-up on a background thread (§ALGO S-18.2, Step 3).
    ///
    /// When `true`, newly created analysis cells are warmed by a
    /// dedicated background thread instead of being warmed inline
    /// during `ingest()`. This eliminates the latency spike that
    /// accompanies cell creation at the cost of a short delay before
    /// new cells participate in scoring.
    ///
    /// When `false`, warm-up runs synchronously within
    /// `reconcile_analysis_set()` — identical to the Step 2 behaviour.
    /// This mode is deterministic (given a fixed `noise_seed`) and is
    /// used by the test suite.
    ///
    /// Default: `false`  (opt-in; production deployments should enable)
    pub background_warming: bool,

    /// Which SVD algorithm to use for subspace evolution (§ALGO S-4.2 Phase 2).
    ///
    /// - `Naive`: dense thin SVD of the full composite matrix — simple,
    ///   correct, O(d·(k+b)²) per call.
    /// - `Brand`: incremental SVD (Brand 2006) — projects onto the current
    ///   basis and SVDs a small (k+b)×(k+b) kernel instead.  Much faster.
    ///
    /// In debug builds, **both** algorithms run regardless of this setting
    /// and their outputs are compared as a continuous oracle test.
    ///
    /// Default: `Brand`
    pub svd_strategy: crate::SvdStrategy,
}

impl<V: Inspectable> Default for SentinelConfig<V> {
    fn default() -> Self {
        Self {
            max_rank: 16,
            forgetting_factor: 0.99,
            rank_update_interval: 100,
            energy_threshold: 0.90,
            eps: 1e-6,
            cusum_slow_decay: 0.999,
            cusum_coord_slow_decay: 0.999,
            cusum_allowance_sigmas: 0.5,
            clip_sigmas: 3.0,
            clip_pressure_decay: 0.95,
            per_sample_scores: false,
            analysis_k: 1024,
            analysis_depth_cutoff: 6,
            split_threshold: V::from_f64(100.0),
            d_create: 3,
            d_evict: 6,
            budget: 100_000,
            noise_schedule: NoiseSchedule::default(),
            noise_batch_size: 16,
            noise_seed: Some(42),
            background_warming: false,
            svd_strategy: crate::SvdStrategy::default(),
        }
    }
}

/// Validation errors for [`SentinelConfig`].
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConfigError {
    /// `max_rank` must be at least 1.
    MaxRankZero,
    /// `forgetting_factor` must be in `(0.0, 1.0)`.
    ForgettingFactorOutOfRange(f64),
    /// `rank_update_interval` must be at least 1.
    RankUpdateIntervalZero,
    /// `analysis_k` must be at least 1.
    AnalysisKZero,
    /// `energy_threshold` must be in `(0.0, 1.0)`.
    EnergyThresholdOutOfRange(f64),
    /// `eps` must be positive.
    EpsNotPositive(f64),
    /// `cusum_slow_decay` must be in `(0.0, 1.0)`.
    CusumSlowDecayOutOfRange(f64),
    /// `cusum_slow_decay` must be strictly greater than `forgetting_factor`.
    CusumSlowDecayTooLow { slow: f64, fast: f64 },
    /// `cusum_coord_slow_decay` must be in `(0.0, 1.0)`.
    CusumCoordSlowDecayOutOfRange(f64),
    /// `cusum_coord_slow_decay` must be strictly greater than `forgetting_factor`.
    CusumCoordSlowDecayTooLow { slow: f64, fast: f64 },
    /// `cusum_allowance_sigmas` must be non-negative.
    CusumAllowanceNegative(f64),
    /// `clip_sigmas` must be positive.
    ClipSigmasNotPositive(f64),
    /// `clip_pressure_decay` must be in `(0.0, 1.0)`.
    ClipPressureDecayOutOfRange(f64),
    /// `split_threshold` must be positive.
    SplitThresholdNotPositive(f64),
    /// `d_create` must be at least 1.
    DCreateZero,
    /// `d_evict` must be strictly greater than `d_create`.
    DEvictNotGreaterThanDCreate { d_create: u32, d_evict: u32 },
    /// `budget` must be positive.
    BudgetZero,
    /// `budget` must exceed mudlark's headroom requirement.
    BudgetTooSmall { budget: usize, required_minimum: usize },
    /// `noise_batch_size` must be ≥ 1 when noise is enabled.
    NoiseBatchSizeZero,
    /// `NoiseSchedule::Geometric::decay` must be in `(0.0, 1.0]`.
    NoiseScheduleDecayOutOfRange(f64),
    /// `NoiseSchedule::Geometric::root` must be > 0 when `min` > 0.
    NoiseScheduleRootZero,
    /// The gap between `d_create` and `d_evict` implies a headroom
    /// requirement too large to represent, so no budget can satisfy it.
    DepthBufferTooLarge { d_create: u32, d_evict: u32 },
    /// The coordinate width `N` is below the smallest width a subspace
    /// tracker can model.
    TrackerDimensionTooSmall { width: u32, minimum: usize },
}

impl std::fmt::Display for ConfigError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MaxRankZero => write!(f, "max_rank must be at least 1"),
            Self::ForgettingFactorOutOfRange(v) => {
                write!(f, "forgetting_factor must be in (0.0, 1.0), got {v}")
            }
            Self::RankUpdateIntervalZero => write!(f, "rank_update_interval must be at least 1"),
            Self::AnalysisKZero => write!(f, "analysis_k must be at least 1"),
            Self::EnergyThresholdOutOfRange(v) => {
                write!(f, "energy_threshold must be in (0.0, 1.0), got {v}")
            }
            Self::EpsNotPositive(v) => write!(f, "eps must be positive, got {v}"),
            Self::CusumSlowDecayOutOfRange(v) => {
                write!(f, "cusum_slow_decay must be in (0.0, 1.0), got {v}")
            }
            Self::CusumSlowDecayTooLow { slow, fast } => {
                write!(f, "cusum_slow_decay ({slow}) must be > forgetting_factor ({fast})")
            }
            Self::CusumCoordSlowDecayOutOfRange(v) => {
                write!(f, "cusum_coord_slow_decay must be in (0.0, 1.0), got {v}")
            }
            Self::CusumCoordSlowDecayTooLow { slow, fast } => {
                write!(f, "cusum_coord_slow_decay ({slow}) must be > forgetting_factor ({fast})")
            }
            Self::CusumAllowanceNegative(v) => {
                write!(f, "cusum_allowance_sigmas must be >= 0, got {v}")
            }
            Self::ClipSigmasNotPositive(v) => write!(f, "clip_sigmas must be > 0, got {v}"),
            Self::ClipPressureDecayOutOfRange(v) => {
                write!(f, "clip_pressure_decay must be in (0.0, 1.0), got {v}")
            }
            Self::SplitThresholdNotPositive(v) => {
                write!(f, "split_threshold must be > 0, got {v}")
            }
            Self::DCreateZero => write!(f, "d_create must be >= 1"),
            Self::DEvictNotGreaterThanDCreate { d_create, d_evict } => {
                write!(f, "d_evict ({d_evict}) must be > d_create ({d_create})")
            }
            Self::BudgetZero => write!(f, "budget must be > 0"),
            Self::BudgetTooSmall {
                budget,
                required_minimum,
            } => {
                write!(
                    f,
                    "budget ({budget}) must be > {required_minimum} (mudlark headroom requirement)"
                )
            }
            Self::NoiseBatchSizeZero => {
                write!(f, "noise_batch_size must be >= 1 when noise is enabled")
            }
            Self::NoiseScheduleDecayOutOfRange(v) => {
                write!(f, "noise_schedule geometric decay must be in (0.0, 1.0], got {v}")
            }
            Self::NoiseScheduleRootZero => {
                write!(f, "noise_schedule geometric root must be > 0 when min > 0")
            }
            Self::DepthBufferTooLarge { d_create, d_evict } => {
                write!(
                    f,
                    "the depth buffer d_evict ({d_evict}) - d_create ({d_create}) implies a \
                     headroom of 3^(buffer+1) that exceeds the addressable range, so no budget \
                     can satisfy it"
                )
            }
            Self::TrackerDimensionTooSmall { width, minimum } => {
                write!(
                    f,
                    "coordinate width N ({width}) is below the minimum tracker dimension \
                     ({minimum}); a narrower root spans its own space and can model nothing"
                )
            }
        }
    }
}

impl std::error::Error for ConfigError {}

/// One or more configuration validation errors.
///
/// Returned by [`SentinelConfig::validate`] when at least one field
/// violates its constraints.  Contains every violation found, not
/// just the first.
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ConfigErrors(pub Vec<ConfigError>);

impl std::fmt::Display for ConfigErrors {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let n = self.0.len();
        for (i, e) in self.0.iter().enumerate() {
            write!(f, "{e}")?;
            if i + 1 < n {
                write!(f, "; ")?;
            }
        }
        Ok(())
    }
}

impl std::error::Error for ConfigErrors {}

/// Non-fatal diagnostic for parameter combinations that are technically
/// valid but likely to produce poor results (§ALGO S-A.7).
#[derive(Debug, Clone, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ConfigWarning {
    /// The noise schedule root is below the empirically derived minimum
    /// for the configured forgetting factor, meaning baselines may not
    /// converge before real observations arrive.
    NoiseScheduleInsufficient {
        /// Noise schedule root (depth-0) round count.
        root: u32,
        /// Minimum recommended rounds for the configured λ.
        recommended_root: u32,
        /// The configured forgetting factor.
        lambda: f64,
    },
}

impl std::fmt::Display for ConfigWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::NoiseScheduleInsufficient {
                root,
                recommended_root,
                lambda,
            } => {
                write!(
                    f,
                    "noise_schedule root ({root}) is below the recommended minimum \
                     ({recommended_root}) for forgetting_factor = {lambda}; \
                     baselines may not converge before real observations arrive \
                     (see §ALGO S-A.7)"
                )
            }
        }
    }
}

impl<V: Inspectable> SentinelConfig<V> {
    /// Validate all invariants.
    ///
    /// Checks every field and collects all violations so the caller
    /// can fix them in one pass rather than iterating one-at-a-time.
    ///
    /// # Errors
    ///
    /// Returns a [`ConfigErrors`] containing every [`ConfigError`]
    /// found, if any.
    pub fn validate(&self) -> Result<(), ConfigErrors> {
        let mut errors = Vec::new();

        if self.max_rank == 0 {
            errors.push(ConfigError::MaxRankZero);
        }
        if self.forgetting_factor.is_nan() || self.forgetting_factor <= 0.0 || self.forgetting_factor >= 1.0 {
            errors.push(ConfigError::ForgettingFactorOutOfRange(self.forgetting_factor));
        }
        if self.rank_update_interval == 0 {
            errors.push(ConfigError::RankUpdateIntervalZero);
        }
        if self.analysis_k == 0 {
            errors.push(ConfigError::AnalysisKZero);
        }
        if self.energy_threshold.is_nan() || self.energy_threshold <= 0.0 || self.energy_threshold >= 1.0 {
            errors.push(ConfigError::EnergyThresholdOutOfRange(self.energy_threshold));
        }
        if self.eps.is_nan() || self.eps <= 0.0 {
            errors.push(ConfigError::EpsNotPositive(self.eps));
        }
        if self.cusum_slow_decay.is_nan() || self.cusum_slow_decay <= 0.0 || self.cusum_slow_decay >= 1.0 {
            errors.push(ConfigError::CusumSlowDecayOutOfRange(self.cusum_slow_decay));
        }
        if self.cusum_slow_decay <= self.forgetting_factor {
            errors.push(ConfigError::CusumSlowDecayTooLow {
                slow: self.cusum_slow_decay,
                fast: self.forgetting_factor,
            });
        }
        if self.cusum_coord_slow_decay.is_nan() || self.cusum_coord_slow_decay <= 0.0 || self.cusum_coord_slow_decay >= 1.0 {
            errors.push(ConfigError::CusumCoordSlowDecayOutOfRange(self.cusum_coord_slow_decay));
        }
        if self.cusum_coord_slow_decay <= self.forgetting_factor {
            errors.push(ConfigError::CusumCoordSlowDecayTooLow {
                slow: self.cusum_coord_slow_decay,
                fast: self.forgetting_factor,
            });
        }
        if self.cusum_allowance_sigmas.is_nan() || self.cusum_allowance_sigmas < 0.0 {
            errors.push(ConfigError::CusumAllowanceNegative(self.cusum_allowance_sigmas));
        }
        if self.clip_sigmas.is_nan() || self.clip_sigmas <= 0.0 {
            errors.push(ConfigError::ClipSigmasNotPositive(self.clip_sigmas));
        }
        if self.clip_pressure_decay.is_nan() || self.clip_pressure_decay <= 0.0 || self.clip_pressure_decay >= 1.0 {
            errors.push(ConfigError::ClipPressureDecayOutOfRange(self.clip_pressure_decay));
        }

        // ── G-V Graph fields (§ALGO S-13.3) ────────────────────
        if self.split_threshold.to_f64_approx().is_nan() || self.split_threshold.to_f64_approx() <= 0.0 {
            errors.push(ConfigError::SplitThresholdNotPositive(self.split_threshold.to_f64_approx()));
        }
        if self.d_create < 1 {
            errors.push(ConfigError::DCreateZero);
        }
        if self.d_evict <= self.d_create {
            errors.push(ConfigError::DEvictNotGreaterThanDCreate {
                d_create: self.d_create,
                d_evict: self.d_evict,
            });
        }
        if self.budget == 0 {
            errors.push(ConfigError::BudgetZero);
        }
        // Mudlark's headroom requirement: budget must exceed
        // max(3^(buffer+1), 2*(d_create-1)) where buffer = d_evict - d_create.
        // Only check when d_evict > d_create (otherwise the earlier check fails).
        if self.budget > 0 && self.d_evict > self.d_create {
            let buffer = self.d_evict - self.d_create;
            // A buffer wide enough to overflow the exponent leaves no
            // representable budget that could clear the requirement, so the
            // depth pair is refused on its own terms rather than measured
            // against a wrapped figure.
            if let Some(headroom) = 3usize.checked_pow(buffer + 1) {
                let convergence = 2 * (self.d_create as usize).saturating_sub(1);
                let required = headroom.max(convergence);
                if self.budget <= required {
                    errors.push(ConfigError::BudgetTooSmall {
                        budget: self.budget,
                        required_minimum: required,
                    });
                }
            } else {
                errors.push(ConfigError::DepthBufferTooLarge {
                    d_create: self.d_create,
                    d_evict: self.d_evict,
                });
            }
        }

        // ── Noise injection fields (§ALGO S-13.5, §ALGO S-18.2) ──
        if !self.noise_schedule.is_disabled() && self.noise_batch_size == 0 {
            errors.push(ConfigError::NoiseBatchSizeZero);
        }
        match &self.noise_schedule {
            NoiseSchedule::Geometric { root, decay, min } => {
                if *decay <= 0.0 || *decay > 1.0 || decay.is_nan() {
                    errors.push(ConfigError::NoiseScheduleDecayOutOfRange(*decay));
                }
                if *root == 0 && *min > 0 {
                    errors.push(ConfigError::NoiseScheduleRootZero);
                }
            }
            NoiseSchedule::Explicit(_) => { /* any values are valid */ }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(ConfigErrors(errors))
        }
    }

    /// Return non-fatal diagnostics for parameter combinations that are
    /// technically valid but empirically known to produce poor results.
    ///
    /// Call after [`validate`](Self::validate) succeeds. The returned
    /// warnings are advisory — the sentinel will still function, but
    /// warm-up may be insufficient and early scores unreliable.
    #[must_use]
    pub fn warnings(&self) -> Vec<ConfigWarning> {
        let mut warnings = Vec::new();

        // §ALGO S-A.7: cross-check noise schedule root vs forgetting factor.
        // Thresholds derived from empirical convergence data (Appendix A).
        let recommended_root = if self.forgetting_factor >= 0.99 {
            450
        } else if self.forgetting_factor >= 0.95 {
            if self.noise_batch_size <= 4 { 200 } else { 50 }
        } else {
            0 // no recommendation for very low λ
        };

        if recommended_root > 0 {
            let actual_root = self.noise_schedule.rounds_for_depth(0);
            if actual_root < recommended_root {
                warnings.push(ConfigWarning::NoiseScheduleInsufficient {
                    root: actual_root,
                    recommended_root,
                    lambda: self.forgetting_factor,
                });
            }
        }

        warnings
    }
}
