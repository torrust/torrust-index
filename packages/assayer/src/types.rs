// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Shared type definitions for the Assayer crate.
//!
//! This module provides foundational types used throughout the crate:
//!
//! - **Scoring axes** — the arity of the axis quartet every batch report carries,
//!   which every per-axis width downstream is derived from.
//! - **Newtype IDs** — Sentinel, dimension, outcome axis, channel, model, and assessment
//!   identifiers with appropriate trait bounds.
//! - **Action types** — Decision outcomes and challenge results.
//! - **Timestamps** — Persistent timestamps for serialisation and decay computation.
//! - **Spatial utilities** — Dyadic interval types for hierarchical routing.
//!
//! # Cross-References
//!
//! - Time-indexed Ledger decay (´def:ledger:time-decay´)
//! - The decay inventory the half-lives come from (´tab:temporal:decay-inventory´)
//! - Two timestamp domains, separated by type (´dec:clock:two-domains´)
//! - The transform is generic over an ordered action set (´dec:derivation:ordered-actions´)
//! - Companion state is host-owned (´dec:challenge:host-ownership´)
//! - The two spatial key types stay distinct (´dec:memory:key-types-distinct´)

use std::hash::Hash;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

/// Number of scoring axes (Novelty, Displacement, Surprise, Coherence).
///
/// The four scoring axes every well-formed batch report carries, an assumed
/// property of the Sentinel interface (´tab:architecture:sentinel-properties´).
/// Every per-axis width downstream is this one number: the six chain z-score
/// views and the three chain CUSUM views take four axes to a view, the
/// coordination block's two per-axis maxima are four wide apiece, and the
/// aggregate block's per-axis maxima and concordances run over it — the
/// extraction chapter derives each of those from a report carrying scores
/// across these axes rather than fixing them independently
/// (´setup:extraction:from-batch-report´).
///
/// This is not the outcome axis count. Outcome axes are registered by the host
/// at runtime, unbounded in number, and enter the extraction width as the
/// term `2 m_s` (´const:assayer:extraction-spatial-stride´); they are
/// identified by `OutcomeAxisId` below and never by this constant.
///
/// ´const:assayer:scoring-axis-arity´ (´alg:const:count´)
/// ´const:assayer:scoring-axis-arity-count-4´
pub const SCORING_AXIS_COUNT: usize = 4;

// ═══════════════════════════════════════════════════════════════════════════════
// Newtype IDs
// ═══════════════════════════════════════════════════════════════════════════════

/// Identifier for a registered Sentinel.
///
/// Each Sentinel provides spatial anomaly scores aggregated from its G-V graph.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SentinelId(pub u32);

/// Identifier for an identity dimension (competitive cell partitioning).
///
/// Dimensions enable entity-specific baseline tracking via competitive cells.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct DimensionId(pub u32);

/// Identifier for an outcome axis (per-axis Bayesian models).
///
/// Outcome axes capture different signal interpretations (e.g., spam vs. fraud).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct OutcomeAxisId(pub u32);

/// Identifier for a decision channel (policy pathway).
///
/// Channels define decision thresholds and action policies per traffic category.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChannelId(pub u32);

/// Identifier for a Bayesian model within the system.
///
/// Multiple model types exist: operational, sister, anchor, and per-axis outcome.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ModelId {
    /// The primary operational model (V).
    Operational,
    /// The sister model for drift detection.
    Sister,
    /// The anchor model with fixed low dimensionality.
    Anchor,
    /// A per-outcome-axis model.
    OutcomeAxis(OutcomeAxisId),
}

/// A per-process monotonic identifier for a risk assessment.
///
/// Assigned during `assess()` via `AtomicU64::fetch_add`. Unique within a
/// single process instance. Not globally unique — for distributed
/// tracing, compose with a process identifier externally.
///
/// `u64` overflows after 2^64 assessments. At 10,000 req/s: 58 million
/// years. No wraparound handling.
///
/// The identifier keys the map an assessment waits in
/// (´dec:retention:pending-map´).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AssessmentId(pub u64);

impl std::fmt::Display for AssessmentId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}

impl From<u64> for AssessmentId {
    fn from(v: u64) -> Self {
        Self(v)
    }
}

impl From<AssessmentId> for u64 {
    fn from(id: AssessmentId) -> Self {
        id.0
    }
}

/// Opaque entity key for the signal cache.
///
/// Typically contains a hashed or serialised entity identifier; it keys the
/// pre-encoded entity cache (´dec:retention:cache-preencoded´).
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct EntityKey(pub Arc<[u8]>);

impl EntityKey {
    /// Creates an `EntityKey` from raw bytes.
    #[must_use]
    pub fn new(bytes: impl Into<Arc<[u8]>>) -> Self {
        Self(bytes.into())
    }

    /// Returns the raw bytes of this key.
    #[must_use]
    pub fn as_bytes(&self) -> &[u8] {
        &self.0
    }
}

impl From<Vec<u8>> for EntityKey {
    fn from(v: Vec<u8>) -> Self {
        Self::new(v)
    }
}

impl From<&[u8]> for EntityKey {
    fn from(s: &[u8]) -> Self {
        Self::new(s.to_vec())
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Action and Challenge Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Decision action taken in response to a derivation-layer reckoning.
///
/// Actions are ordered by severity: `Allow` < `Challenge` < `Slow` < `Block`.
/// The transform is generic over that ordered set
/// (´dec:derivation:ordered-actions´), and what the host did stays a
/// distinct type from what a derivation proposes
/// (´dec:surface:distinct-action-types´).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum Action {
    /// Permit the request without intervention.
    Allow,
    /// Issue a challenge (e.g., CAPTCHA) before proceeding.
    Challenge,
    /// Rate-limit or delay the request.
    Slow,
    /// Deny the request outright.
    Block,
}

/// Outcome of a challenge presented to the user.
///
/// The companion state this feeds is host-owned
/// (´dec:challenge:host-ownership´), and it stays a distinct type from
/// what a derivation proposes (´dec:surface:distinct-action-types´).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[non_exhaustive]
pub enum ChallengeResult {
    /// The challenge was failed (likely non-human).
    Fail,
    /// The challenge was passed (likely human).
    Pass,
}

/// Challenge-effectiveness estimate supplied by the Companion Tracker.
///
/// This is the only Companion evidence read by the Derivation Function. The
/// Core does not store it; hosts pass it explicitly to `derive_reckoning()`,
/// which is what the crossing admits (´rule:derivation:closed-inputs´).
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ChallengeEstimate {
    /// Posterior mean challenge catch rate `q̂_c`.
    pub(crate) q_c: f64,
    /// Posterior variance `σ²(q̂_c)`.
    pub(crate) variance: f64,
}

/// A challenge estimate whose pair is outside the posterior's domain.
///
/// Returned by [`ChallengeEstimate::new`]: the constructor is where a
/// malformed pair is refused, so the derivation can take the estimate
/// as given instead of repairing it per call
/// (´dec:degradation:error-partition´).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidChallengeEstimate {
    /// The refused posterior mean.
    pub q_c: f64,
    /// The refused posterior variance.
    pub variance: f64,
}

impl std::fmt::Display for InvalidChallengeEstimate {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid challenge estimate: q_c = {} must be finite in [0, 1] and variance = {} must be finite and non-negative",
            self.q_c, self.variance
        )
    }
}

impl std::error::Error for InvalidChallengeEstimate {}

impl ChallengeEstimate {
    /// Creates a challenge estimate from its posterior mean and variance.
    ///
    /// # Errors
    ///
    /// Refuses a malformed pair — a mean that is not finite or lies
    /// outside `[0, 1]`, or a variance that is not finite or is
    /// negative. A malformed posterior is the caller's defect and is
    /// refused where the caller can fix it, rather than silently
    /// replaced downstream (´dec:degradation:error-partition´).
    pub fn new(q_c: f64, variance: f64) -> Result<Self, InvalidChallengeEstimate> {
        if !q_c.is_finite() || !(0.0..=1.0).contains(&q_c) || !variance.is_finite() || variance < 0.0 {
            return Err(InvalidChallengeEstimate { q_c, variance });
        }
        Ok(Self { q_c, variance })
    }

    /// Creates an estimate from a pair already established to be in
    /// domain — for crate-internal values computed by construction
    /// (a Beta posterior's moments, a sanitised override).
    #[must_use]
    pub(crate) const fn new_unchecked(q_c: f64, variance: f64) -> Self {
        Self { q_c, variance }
    }

    /// The posterior mean challenge catch rate `q̂_c`.
    #[must_use]
    pub const fn q_c(&self) -> f64 {
        self.q_c
    }

    /// The posterior variance `σ²(q̂_c)`.
    #[must_use]
    pub const fn variance(&self) -> f64 {
        self.variance
    }
}

impl Default for ChallengeEstimate {
    fn default() -> Self {
        Self {
            q_c: 0.5,
            variance: 1.0 / 12.0,
        }
    }
}

/// The challenge posterior as it crosses the derivation boundary, in
/// one of the two specified variants (´sig:companion:posterior´).
///
/// The variant distinction is not a convenience: it is the boundary at
/// which the Companion's conjugate structure either survives into the
/// derivation or does not. The conjugate variant carries the Beta
/// pseudo-count pair and supports exact quantiles and exact dominance
/// probabilities; the moment-matched variant exists for hosts whose
/// challenge estimate comes from somewhere else, and degrades those
/// consumers to a moment-matched surrogate rather than removing them.
#[derive(Clone, Copy, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum ChallengePosteriorInput {
    /// The Beta pseudo-count pair `(α, β)`, as the Companion maintains
    /// it (´alg:companion:inference´).
    Conjugate {
        /// Failure pseudo-count α (adverse actors correctly caught).
        alpha: f64,
        /// Pass pseudo-count β (benign actors correctly released).
        beta: f64,
    },
    /// A point estimate and variance from an external source; quantiles
    /// only through a moment-matched Beta (´sig:companion:posterior´).
    MomentMatched(ChallengeEstimate),
}

/// A conjugate pseudo-count pair outside the posterior's domain.
///
/// Returned by [`ChallengePosteriorInput::conjugate`]: the constructor
/// is where a malformed pair is refused, so the derivation can take the
/// posterior as given (´dec:degradation:error-partition´).
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct InvalidChallengePosterior {
    /// The refused failure pseudo-count.
    pub alpha: f64,
    /// The refused pass pseudo-count.
    pub beta: f64,
}

impl std::fmt::Display for InvalidChallengePosterior {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "invalid challenge posterior: pseudo-counts alpha = {} and beta = {} must be finite and positive",
            self.alpha, self.beta
        )
    }
}

impl std::error::Error for InvalidChallengePosterior {}

impl ChallengePosteriorInput {
    /// Creates the conjugate variant from its two pseudo-counts.
    ///
    /// # Errors
    ///
    /// Refuses a malformed pair — a count that is not finite or is not
    /// strictly positive. A malformed posterior is the caller's defect
    /// and is refused where the caller can fix it
    /// (´dec:degradation:error-partition´).
    pub fn conjugate(alpha: f64, beta: f64) -> Result<Self, InvalidChallengePosterior> {
        if !alpha.is_finite() || alpha <= 0.0 || !beta.is_finite() || beta <= 0.0 {
            return Err(InvalidChallengePosterior { alpha, beta });
        }
        Ok(Self::Conjugate { alpha, beta })
    }

    /// The posterior mean `q̂_c`: `α / (α + β)` for the conjugate
    /// variant, the carried estimate otherwise.
    #[must_use]
    pub fn q_c(&self) -> f64 {
        match self {
            Self::Conjugate { alpha, beta } => alpha / (alpha + beta),
            Self::MomentMatched(estimate) => estimate.q_c(),
        }
    }

    /// The posterior variance `σ²(q̂_c)`.
    #[must_use]
    // Justified: σ² = αβ / ((α+β)²(α+β+1)) — the squared sum is the
    // formula, not a mistyped pairing.
    #[allow(clippy::suspicious_operation_groupings)]
    pub fn variance(&self) -> f64 {
        match self {
            Self::Conjugate { alpha, beta } => {
                let sum = alpha + beta;
                (alpha * beta) / (sum * sum * (sum + 1.0))
            }
            Self::MomentMatched(estimate) => estimate.variance(),
        }
    }

    /// The Beta shape the posterior's quantiles and dominance
    /// probabilities are evaluated on: the carried pair itself for the
    /// conjugate variant — exact — and a moment-matched Beta for the
    /// other (´sig:companion:posterior´). A moment pair no Beta carries
    /// falls back to the uniform prior (´def:companion:prior´) rather
    /// than to an invented shape.
    #[must_use]
    pub fn beta_shape(&self) -> (f64, f64) {
        match self {
            Self::Conjugate { alpha, beta } => (*alpha, *beta),
            Self::MomentMatched(estimate) => {
                let m = estimate.q_c();
                let v = estimate.variance();
                if m <= 0.0 || m >= 1.0 || v <= 0.0 {
                    return (1.0, 1.0);
                }
                let nu = m * (1.0 - m) / v - 1.0;
                if nu <= 0.0 || !nu.is_finite() {
                    return (1.0, 1.0);
                }
                (m * nu, (1.0 - m) * nu)
            }
        }
    }
}

impl From<ChallengeEstimate> for ChallengePosteriorInput {
    fn from(estimate: ChallengeEstimate) -> Self {
        Self::MomentMatched(estimate)
    }
}

impl Default for ChallengePosteriorInput {
    /// The uniform prior Beta(1, 1) (´def:companion:prior´), in its
    /// conjugate form: what a host with no Companion supplies.
    fn default() -> Self {
        Self::Conjugate { alpha: 1.0, beta: 1.0 }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Registration Policy Types
// ═══════════════════════════════════════════════════════════════════════════════

/// Controls which labels an outcome axis receives.
///
/// Some axes only want labels where the entity was eligible for the outcome
/// (e.g., a fraud axis only cares about completed transactions).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum OutcomeEligibility {
    /// Only labels where the entity was eligible for this outcome.
    #[default]
    EligibleOnly,
    /// All labels, regardless of eligibility.
    AllLabels,
}

/// Controls whether an outcome axis produces spatial features.
///
/// Spatial features are expensive but valuable for location-specific patterns.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SpatialFeaturePolicy {
    /// Spatial features are disabled for this axis.
    #[default]
    Disabled,
    /// Spatial features are enabled for this axis.
    Enabled,
}

/// Resource budget for an identity dimension.
///
/// Controls how much memory/computation the dimension is allowed to use.
#[derive(Clone, Debug, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentityBudget {
    /// Maximum number of competitive cells to track.
    pub max_cells: usize,
    /// Maximum number of observations to buffer.
    pub observation_capacity: usize,
    /// Minimum accumulated importance at which a graph cell splits.
    pub split_threshold: u64,
    /// Initial deepest graph depth at which a cell may be created.
    pub depth_create: u32,
    /// Initial shallowest graph depth at which a cell may be evicted.
    pub depth_evict: u32,
    /// Hourly spatial decay rate applied to graph importance.
    pub spatial_decay_rate: f64,
}

impl IdentityBudget {
    /// Returns the shipped budget for a competitive depth cutoff.
    ///
    /// The create gate matches the cutoff and the eviction gate keeps the
    /// historical two-level buffer. All returned fields remain public so the
    /// host can override them before registration
    /// (´schema:keyspace:dimension-record´).
    #[must_use]
    pub const fn for_depth_cutoff(depth_cutoff: u8) -> Self {
        Self {
            max_cells: 10_000,
            observation_capacity: 100_000,
            split_threshold: 10,
            depth_create: depth_cutoff as u32,
            depth_evict: depth_cutoff as u32 + 2,
            spatial_decay_rate: 0.998,
        }
    }
}

impl Default for IdentityBudget {
    fn default() -> Self {
        Self::for_depth_cutoff(10)
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Persistent Timestamp
// ═══════════════════════════════════════════════════════════════════════════════

/// Maximum decay hours for clamping (1 year).
///
/// Prevents numerical overflow on very large time gaps. The one shared
/// constant both timestamp accessors clamp against, so that no caller can
/// obtain an unclamped elapsed reading (´dec:clock:embedded-clamp´).
///
/// ´const:assayer:elapsed-time-ceiling´ (´alg:const:scalar´)
/// ´const:assayer:elapsed-time-ceiling-scalar-8760p0´
pub const MAX_DECAY_HOURS: f64 = 8_760.0;

/// A timestamp that can be serialised and used for decay calculations.
///
/// Unlike `Instant`, this type survives across process restarts and can be
/// serialised to checkpoints: it is the persistent one of the two
/// timestamp domains (´dec:clock:two-domains´).
/// The fields are sealed: a caller reads elapsed time through
/// [`hours_since`](Self::hours_since), whose double clamp is what makes
/// the unbounded interval unobtainable (´dec:clock:embedded-clamp´),
/// (´cor:clock:clamp-unbypassable´). The derived serialisation reaches
/// the private fields, which is the permitted route across a checkpoint.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct PersistentTimestamp {
    /// Seconds since the Unix epoch (may be negative for pre-1970 times).
    pub(crate) seconds: i64,
    /// Nanoseconds within the second. Invariant: `< 1_000_000_000`.
    pub(crate) nanos: u32,
}

impl PersistentTimestamp {
    /// Creates a new `PersistentTimestamp`.
    ///
    /// # Panics
    ///
    /// Debug-panics if `nanos >= 1_000_000_000`. In release builds,
    /// overflow nanoseconds are carried into seconds so the invariant
    /// `nanos < 1_000_000_000` is restored, keeping the persistent domain's
    /// representation well formed (´dec:clock:two-domains´).
    ///
    /// The carry is checked, and a carry with nowhere to go saturates at the
    /// greatest representable instant — `i64::MAX` seconds and 999,999,999
    /// nanoseconds — rather than wrapping. The workspace builds release
    /// without overflow checks, so an unchecked carry past `i64::MAX` turned
    /// the latest instant the type can hold into the earliest, and this type
    /// derives its ordering from the second count first: every comparison
    /// against the wrapped value inverted, which is the one failure the
    /// timestamp domain cannot absorb, since the ordering is what decay and
    /// staleness are read from. Saturating at the maximum keeps the
    /// constructor monotone — an input at or beyond the representable range
    /// maps to the greatest value and to nothing below it — and keeps it
    /// infallible, so no caller has to decide what to do with a time it
    /// cannot express. Saturating the second count alone would not: that
    /// would send `new(i64::MAX, 1_000_000_000)` to `i64::MAX` seconds and
    /// zero nanoseconds, which orders *below* the strictly earlier
    /// `new(i64::MAX, 999_999_999)`.
    #[must_use]
    pub fn new(seconds: i64, nanos: u32) -> Self {
        // TODO ´todo:code:make-this-a-hard-release-invariant´: make this a hard release invariant instead of
        // normalising malformed nanoseconds once checkpoint deserialisation has
        // a fallible validation path for corrupted persisted timestamps
        // (´dec:clock:two-domains´).
        debug_assert!(nanos < 1_000_000_000, "nanos must be < 1_000_000_000, got {nanos}");
        let extra_secs = nanos / 1_000_000_000;
        let Some(carried) = seconds.checked_add(i64::from(extra_secs)) else {
            // The carry has nowhere to go: hold the whole instant at the
            // greatest representable one, which is the only value that keeps
            // the ordering monotone here.
            return Self {
                seconds: i64::MAX,
                nanos: 999_999_999,
            };
        };
        Self {
            seconds: carried,
            nanos: nanos % 1_000_000_000,
        }
    }

    /// Returns the current time as a `PersistentTimestamp`.
    #[must_use]
    pub fn now() -> Self {
        Self::from_system_time(SystemTime::now())
    }

    /// Converts a `SystemTime` to a `PersistentTimestamp`.
    ///
    /// Handles pre-epoch times via the borrow-a-second pattern so that
    /// the `nanos` field stays in `[0, 10^9)` and the round-trip
    /// through `to_system_time` is exact.
    #[must_use]
    pub fn from_system_time(time: SystemTime) -> Self {
        match time.duration_since(UNIX_EPOCH) {
            Ok(dur) => Self::new(dur.as_secs().try_into().unwrap_or(i64::MAX), dur.subsec_nanos()),
            Err(e) => {
                // Pre-epoch: t < UNIX_EPOCH.
                let dur = e.duration();
                let raw_secs: i64 = dur.as_secs().try_into().unwrap_or(i64::MAX);
                let sub_nanos = dur.subsec_nanos();
                if sub_nanos == 0 {
                    Self::new(-raw_secs, 0)
                } else {
                    // Borrow one second so nanos stays positive.
                    Self::new(-raw_secs - 1, 1_000_000_000 - sub_nanos)
                }
            }
        }
    }

    /// Converts this timestamp to a `SystemTime`.
    ///
    /// Returns `None` if the timestamp cannot be represented as a
    /// `SystemTime`. Sealed with the fields: standard-library arithmetic
    /// over two converted readings is the second route to the unbounded
    /// interval the clamp exists to withhold (´cor:clock:clamp-unbypassable´).
    #[must_use]
    #[allow(dead_code)] // Justified: the crate suite's round-trip oracle; no production path converts back
    pub(crate) fn to_system_time(self) -> Option<SystemTime> {
        if self.seconds >= 0 {
            let secs: u64 = self.seconds.try_into().ok()?;
            let dur = Duration::new(secs, self.nanos);
            UNIX_EPOCH.checked_add(dur)
        } else {
            // Reverse the borrow-a-second encoding.
            let abs_secs: u64 = (-(self.seconds + 1)).try_into().ok()?;
            let complement_nanos = 1_000_000_000 - self.nanos;
            UNIX_EPOCH.checked_sub(Duration::new(abs_secs, complement_nanos))
        }
    }

    /// Computes the hours elapsed from `earlier` to `self`.
    ///
    /// # Clamping Behaviour
    ///
    /// - Backward clock jumps (`self` < `earlier`) → `0.0`
    /// - Enormous gaps (> 1 year) → [`MAX_DECAY_HOURS`]
    ///
    /// The clamp is embedded in the timestamp interface
    /// (´dec:clock:embedded-clamp´).
    #[must_use]
    pub fn hours_since(&self, earlier: &Self) -> f64 {
        let self_nanos = i128::from(self.seconds) * 1_000_000_000 + i128::from(self.nanos);
        let earlier_nanos = i128::from(earlier.seconds) * 1_000_000_000 + i128::from(earlier.nanos);

        let delta_nanos = self_nanos.saturating_sub(earlier_nanos);

        if delta_nanos <= 0 {
            return 0.0;
        }

        // Convert nanoseconds to hours: nanos / (3600 * 1e9)
        // Precision loss is acceptable: we only need ~4 decimal places for hours
        #[allow(clippy::cast_precision_loss)]
        let hours = delta_nanos as f64 / 3_600_000_000_000.0;

        hours.min(MAX_DECAY_HOURS)
    }
}

// Deliberately no `Default`: a default that read the wall clock was an
// unbounded time read with no call site to inspect, reachable from every
// derived default of every containing structure — the shape that defeats
// clock injection (´dec:clock:two-domains´). Obtaining the present
// requires saying so, through [`PersistentTimestamp::now`] or an
// injected clock.

// ═══════════════════════════════════════════════════════════════════════════════
// Spatial Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Key for addressing cells in the Outcome Ledger.
///
/// A `LedgerKey` identifies a dyadic interval by its lower bound (`lo`)
/// and depth. Used for hierarchical routing in Ledger, identity, and
/// report index structures. It is one of the two spatial key types, which
/// stay distinct (´dec:memory:key-types-distinct´).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct LedgerKey {
    /// Lower bound of the dyadic interval.
    pub lo: u128,
    /// Depth in the dyadic hierarchy (0 = full space, 128 = single point).
    pub depth: u8,
}

impl LedgerKey {
    /// Creates a new `LedgerKey`.
    #[must_use]
    pub const fn new(lo: u128, depth: u8) -> Self {
        Self { lo, depth }
    }

    /// Creates a `LedgerKey` from a coordinate at the given depth.
    ///
    /// Uses [`dyadic_ancestor_lo`] to compute the interval lower bound.
    #[must_use]
    pub const fn from_coordinate(coord: u128, depth: u8) -> Self {
        Self {
            lo: dyadic_ancestor_lo(coord, depth),
            depth,
        }
    }

    /// Returns the upper bound of this key's dyadic interval.
    #[must_use]
    pub const fn hi(&self) -> u128 {
        dyadic_ancestor_hi(self.lo, self.depth)
    }

    /// Returns `true` if `coord` is contained in this key's dyadic interval.
    #[must_use]
    pub const fn contains(&self, coord: u128) -> bool {
        let hi = self.hi();
        coord >= self.lo && coord <= hi
    }
}

/// A dyadic interval in a 128-bit coordinate space.
///
/// Used for hierarchical cell routing in the Ledger and identity dimensions.
/// It is the other of the two spatial key types
/// (´dec:memory:key-types-distinct´).
///
/// # Inclusive Upper Bound
///
/// `hi` is inclusive. This intentionally differs from the canonical spec's
/// half-open interval notation; callers should test containment as
/// `coord >= lo && coord <= hi`.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CellInterval {
    /// Inclusive lower bound of the interval.
    pub lo: u128,
    /// Inclusive upper bound of the interval.
    pub hi: u128,
    /// Depth of the interval in the dyadic hierarchy (0 = full space).
    pub depth: u8,
}

impl CellInterval {
    /// Creates a new `CellInterval` with an inclusive upper bound.
    ///
    /// The `hi` argument is included in the interval. This convention matches
    /// [`Self::width`], which computes `hi - lo + 1`.
    #[must_use]
    pub const fn new(lo: u128, hi: u128, depth: u8) -> Self {
        Self { lo, hi, depth }
    }

    /// Returns the width of this inclusive interval.
    ///
    /// Because [`Self::hi`] is inclusive, the width is `hi - lo + 1`, not
    /// `hi - lo`.
    #[must_use]
    pub const fn width(&self) -> u128 {
        self.hi.saturating_sub(self.lo).saturating_add(1)
    }
}

/// Computes the dyadic ancestor lower bound at a given depth.
///
/// Given a 128-bit coordinate and a depth, returns the lower bound of the
/// dyadic interval containing that coordinate at that depth.
///
/// # Arguments
///
/// * `coord` — A 128-bit coordinate
/// * `depth` — Depth in bits (0 = full space, 128 = single point)
///
/// # Returns
///
/// The lower bound of the dyadic interval at `depth` containing `coord`.
///
/// # Examples
///
/// ```
/// use torrust_assayer::types::dyadic_ancestor_lo;
///
/// // At depth 8, keeps only the top 8 bits (zeros out bottom 120 bits)
/// let coord = 0xABCD_EF01_2345_6789_ABCD_EF01_2345_6789_u128;
/// let lo = dyadic_ancestor_lo(coord, 8);
/// // Top 8 bits are 0xAB, so lo = 0xAB << 120
/// assert_eq!(lo, 0xAB00_0000_0000_0000_0000_0000_0000_0000_u128);
///
/// // At depth 0, full space starts at 0
/// assert_eq!(dyadic_ancestor_lo(coord, 0), 0);
///
/// // At depth 128, single point = coord itself
/// assert_eq!(dyadic_ancestor_lo(coord, 128), coord);
/// ```
#[must_use]
pub const fn dyadic_ancestor_lo(coord: u128, depth: u8) -> u128 {
    if depth == 0 {
        return 0;
    }
    if depth >= 128 {
        return coord;
    }

    // Zero out the bottom (128 - depth) bits
    let shift = 128 - depth;
    (coord >> shift) << shift
}

/// Computes the dyadic ancestor upper bound at a given depth.
///
/// Given a 128-bit coordinate and a depth, returns the upper bound of the
/// dyadic interval containing that coordinate at that depth.
#[must_use]
pub const fn dyadic_ancestor_hi(coord: u128, depth: u8) -> u128 {
    if depth == 0 {
        return u128::MAX;
    }
    if depth >= 128 {
        return coord;
    }

    let lo = dyadic_ancestor_lo(coord, depth);
    let width = 1_u128 << (128 - depth);
    lo.saturating_add(width.saturating_sub(1))
}

/// Checks whether an interval is dyadic at the given depth.
///
/// A dyadic interval at depth `d` has:
/// - `lo` aligned to a multiple of `2^(128-d)`
/// - `hi = lo + 2^(128-d) - 1`
///
/// # Arguments
///
/// * `lo` — Lower bound of the interval
/// * `hi` — Upper bound of the interval
/// * `depth` — Claimed depth
///
/// # Returns
///
/// `true` if the interval is a valid dyadic interval at the given depth.
#[must_use]
pub const fn is_dyadic(lo: u128, hi: u128, depth: u8) -> bool {
    if depth == 0 {
        return lo == 0 && hi == u128::MAX;
    }
    if depth >= 128 {
        return lo == hi;
    }

    // Width must be 2^(128 - depth)
    let expected_width = 1_u128 << (128 - depth);

    // Check alignment: lo must be a multiple of expected_width
    if !lo.is_multiple_of(expected_width) {
        return false;
    }

    // Check hi = lo + expected_width - 1
    let expected_hi = lo.saturating_add(expected_width.saturating_sub(1));
    hi == expected_hi
}

// ═══════════════════════════════════════════════════════════════════════════════
// Time Utilities
// ═══════════════════════════════════════════════════════════════════════════════

/// Converts a `Duration` to hours as a floating-point value.
///
/// Clamped to [`MAX_DECAY_HOURS`].
#[must_use]
pub fn duration_to_hours(dur: Duration) -> f64 {
    let hours = dur.as_secs_f64() / 3600.0;
    hours.min(MAX_DECAY_HOURS)
}

/// Computes hours elapsed since a given `Instant`.
///
/// Clamped to [`MAX_DECAY_HOURS`].
#[must_use]
pub fn hours_since_instant(earlier: Instant) -> f64 {
    duration_to_hours(earlier.elapsed())
}
