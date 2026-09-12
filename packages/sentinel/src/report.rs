// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Report types emitted by the sentinel.
//!
//! These types carry the raw statistical measurements from each
//! [`ingest`](crate::SpectralSentinel::ingest) call.
//! They contain numbers and facts — never opinions or recommended actions.
//!
//! The host reads these reports and applies its own policy to decide
//! what (if anything) to do about them.

use std::fmt::Debug;

use torrust_mudlark::GNodeId;

// ─── Batch-level ────────────────────────────────────────────

/// Complete statistical output from one
/// [`ingest`](crate::SpectralSentinel::ingest) call.
///
/// Matches the structure defined in algorithm.md Chapter 14.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct BatchReport<C: Copy + Debug> {
    /// Per-cell reports for competitively selected cells ($\mathcal{A}$).
    ///
    /// Only competitive cells that received observations in this batch
    /// are included. Ordered by `GNodeId` for deterministic output
    /// (ADR-S-005).
    pub cell_reports: Vec<CellReport<C>>,

    /// Per-cell reports for ancestor-only cells ($\mathcal{A}^* \setminus \mathcal{A}$).
    ///
    /// Ancestor cells provide multi-scale context. Only those that
    /// received observations in this batch are included.
    /// Ordered by `GNodeId`.
    pub ancestor_reports: Vec<CellReport<C>>,

    /// Hierarchical coordination reports from the G-tree walk (§ALGO S-9.4).
    ///
    /// One report per active coordination context. Ordered shallowest first,
    /// ties broken by ascending `GNodeId`.
    /// Empty when fewer than 2 competitive cells report scores.
    pub coordination_reports: Vec<CoordinationReport<C>>,

    /// Snapshot of the G-V Graph's spatial contour.
    pub contour: ContourSnapshot,

    /// Operational health snapshot of the sentinel.
    pub health: HealthReport,

    /// Summary of the current analysis set.
    pub analysis_set_summary: AnalysisSetSummary,

    /// How old the oldest observation in this batch was when this
    /// report was emitted, in microseconds.
    ///
    /// Measured entirely on the sentinel's own monotonic clock: the
    /// batch is stamped as it arrives at
    /// [`ingest`](crate::SpectralSentinel::ingest) and the figure is
    /// read off as this report is assembled. No wall-clock instant is
    /// recorded and no clock is compared with another machine's, so
    /// there is no skew for the number to carry.
    ///
    /// A batch arrives whole, at the call boundary, so its oldest
    /// observation is no older than the call and one figure bounds
    /// every observation in the batch: none is older than this, and the
    /// oldest is exactly this old.
    ///
    /// `None` says the report carries no age. Two situations reach it —
    /// a batch that held no observations, which therefore has no oldest
    /// one, and a payload written before this field existed — and they
    /// share a spelling because they tell a consumer the same thing:
    /// there is nothing here to read. A zero would say something else,
    /// and something untrue.
    ///
    /// What the figure does *not* include is how long the host held the
    /// observations before handing them over. That delay is real and is
    /// left as an explicitly unmeasured residual: measuring it would
    /// mean comparing two clocks, and this figure's whole value is that
    /// it compares none.
    ///
    /// Saturates at [`u64::MAX`] microseconds — some hundreds of
    /// thousands of years, and unreachable in practice.
    #[cfg_attr(feature = "serde", serde(default))]
    pub oldest_observation_age_micros: Option<u64>,
}

// ─── Cell-level ─────────────────────────────────────────────

/// Statistics for a single analysis cell after processing one batch.
///
/// Each cell is at a specific G-tree depth and operates on suffix bits
/// `[d, N)` at width `w = N - d`. Competitive cells are selected
/// by the analysis selector (§ALGO S-4.1); ancestor cells provide
/// multi-scale context (§ALGO S-4.2).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct CellReport<C: Copy + Debug> {
    /// Arena handle of the backing G-node.
    pub gnode_id: GNodeId,

    /// Lower bound of the dyadic interval (inclusive).
    pub start: C,

    /// Upper bound of the dyadic interval, exclusive everywhere except at the
    /// top of the domain: the cell whose bound is the domain maximum owns that
    /// maximum, because a coordinate width filling the coordinate type leaves
    /// no value above it to be excluded.
    pub end: C,

    /// G-tree depth of this cell.
    pub depth: u32,

    /// Suffix width: `N - depth`.
    pub analysis_width: usize,

    /// Whether this cell is competitively selected (vs ancestor-only).
    pub is_competitive: bool,

    /// How many observations in this batch routed to this cell.
    pub sample_count: usize,

    /// Current rank of the learned subspace.
    pub rank: usize,

    /// Fraction of total variance captured by the current rank.
    pub energy_ratio: f64,

    /// Largest singular value of the learned subspace.
    pub top_singular_value: f64,

    /// Anomaly scores along all four measurement axes.
    pub scores: AnomalyScores,

    /// How mature is this tracker's learned model?
    pub maturity: TrackerMaturity,

    /// Geometric properties of this tracker's current state.
    pub geometry: ScoringGeometry,

    /// Per-sample scores, if enabled.
    pub per_sample: Option<Vec<SampleScore>>,
}

// ─── Coordination-level ───────────────────────────────────────

/// Coordination analysis at a single G-tree internal node (§ALGO S-9.1).
///
/// Produced by a coordination tracker (`SubspaceTracker` at $w = 4$)
/// that consumes running-mean-centred cell-score matrices as
/// observations. The group consists of all competitive cells in
/// this node's subtree that reported scores in this batch.
///
/// The four anomaly axes have second-order meaning at this level:
///
/// | Meta-axis    | Detects                                              |
/// |-------------|------------------------------------------------------|
/// | Novelty     | A cell-score pattern the model has never seen         |
/// | Displacement| The overall score landscape has shifted               |
/// | Surprise    | A specific scoring axis is system-wide anomalous      |
/// | Coherence   | An unusual combination of axis elevations             |
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct CoordinationReport<C: Copy + Debug> {
    /// Arena handle of the coordination context's G-node.
    pub gnode_id: GNodeId,

    /// Lower bound of the coordination context's dyadic interval (inclusive).
    pub start: C,

    /// Upper bound of the coordination context's dyadic interval, exclusive
    /// everywhere except at the top of the domain: a context whose bound is the
    /// domain maximum owns that maximum, because a coordinate width filling the
    /// coordinate type leaves no value above it to be excluded. The root
    /// context covers the full-width interval, so this is the ordinary case
    /// rather than a corner of it.
    pub end: C,

    /// G-tree depth of the coordination context.
    pub depth: u32,

    /// How many competitive cells in this subtree contributed
    /// score vectors this batch.
    pub cells_reporting: usize,

    /// Current rank of the coordination tracker's learned subspace.
    pub rank: usize,

    /// Fraction of total variance captured by the rank.
    pub energy_ratio: f64,

    /// Largest singular value.
    pub top_singular_value: f64,

    /// Anomaly scores at the coordination level.
    pub scores: AnomalyScores,

    /// How mature is this coordination tracker's model?
    pub maturity: TrackerMaturity,

    /// Geometric properties of the coordination tracker.
    pub geometry: ScoringGeometry,

    /// Per-member scores, if
    /// [`SentinelConfig::per_sample_scores`](crate::SentinelConfig::per_sample_scores)
    /// is enabled.
    ///
    /// Each entry corresponds to one competitive cell in the coordination
    /// group. The `cell_start`/`cell_end`/`cell_depth` fields identify
    /// which cell produced this score vector. When present, indices
    /// correspond to cells in subtree order (left-to-right).
    pub per_member: Option<Vec<MemberScore<C>>>,
}

// ─── Per-depth-level (internal) ─────────────────────────────

/// Statistics for a single tracker after processing one batch.
///
/// This is the internal report type returned by `SubspaceTracker::observe()`.
/// Used internally; callers see [`CellReport`] at the public API.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackerReport {
    /// The suffix depth `d`. The tracker analyses suffix bits `[d, N)`
    /// at width `w = N − d` (§ALGO S-3.2).
    pub depth: u8,

    /// Current rank of the learned subspace (number of active basis vectors).
    pub rank: usize,

    /// Fraction of total variance captured by the current rank.
    pub energy_ratio: f64,

    /// Largest singular value of the learned subspace.
    pub top_singular_value: f64,

    /// Anomaly scores along all four measurement axes.
    pub scores: AnomalyScores,

    /// How mature is this tracker's learned model?
    pub maturity: TrackerMaturity,

    /// Geometric properties that determine which scoring axes are
    /// structurally meaningful at this tracker's current state.
    pub geometry: ScoringGeometry,

    /// Per-sample scores, if enabled.
    pub per_sample: Option<Vec<SampleScore>>,
}

// ─── Anomaly scores ─────────────────────────────────────

/// The four anomaly-score axes for a batch of observations.
///
/// The sentinel scores each observation along four independent axes
/// organised into two conceptual groups:
///
/// **Subspace axis** — how well the learned model explains the observation:
///
/// | Score | Metric | Intuition |
/// |-------|--------|-----------|
/// | *Novelty* | Residual energy / DOF: `‖X − X̂‖² / (dim − k)` | "How much of this is foreign?" |
///
/// **Cell axis** — how typical the observation is for *this* cell:
///
/// | Score | Metric | Intuition |
/// |-------|--------|-----------|
/// | *Displacement* | `‖z‖² / (k + ‖z‖²)`, bounded in `[0, 1)` | "How far is this from the centroid?" |
/// | *Surprise* | Mahalanobis / rank: `Σⱼ ((zⱼ − μⱼ)/σⱼ)² / k` | "The shape is familiar, but the magnitude is wild" |
/// | *Coherence* | Cross-correlation deviation: `Σⱼ<ₗ (zⱼzₗ − Cⱼₗ)²` | "Normal individually, but this combination is new" |
///
/// Displacement, surprise, and coherence decompose the latent
/// activation pattern along orthogonal statistical concerns:
/// displacement measures total energy, surprise measures
/// per-dimension magnitude (diagonal covariance), and coherence
/// measures pairwise interaction (off-diagonal covariance).
///
/// All four axes share the same polarity: **higher values indicate
/// greater anomalous departure**. This ensures uniform z-score
/// interpretation and EWMA outlier-filter robustness (see
/// `docs/algorithm.md`, Appendix A).
///
/// **Why not projection energy / "normality"?** Under the sentinel's
/// centred binary encoding, every observation has the same L2 norm
/// (`d / 4`). Projection energy is therefore a perfect affine function
/// of residual energy — it carries zero independent information.
/// See `docs/algorithm.md`, Appendix B for the full proof.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnomalyScores {
    /// Residual energy per residual DOF: `‖X − X̂‖² / (dim − k)`.
    pub novelty: ScoreDistribution,

    /// Cell displacement: `‖z‖² / (k + ‖z‖²)`, bounded in `[0, 1)`.
    pub displacement: ScoreDistribution,

    /// Latent surprise: Mahalanobis distance per rank,
    /// `Σⱼ ((zⱼ − μⱼ) / σⱼ)² / k`.
    pub surprise: ScoreDistribution,

    /// Latent coherence: cross-correlation deviation,
    /// `2 / (k(k−1)) · Σⱼ<ₗ (zⱼzₗ − Cⱼₗ)²`.
    pub coherence: ScoreDistribution,
}

// ─── Score distribution ─────────────────────────────────────

/// Summary statistics for a vector of anomaly scores.
///
/// Contains both the raw score distribution and its relationship
/// to the learned baseline (via z-scores).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoreDistribution {
    /// Minimum score in the batch.
    pub min: f64,

    /// Maximum score in the batch.
    pub max: f64,

    /// Arithmetic mean of scores in the batch.
    pub mean: f64,

    /// Z-score of the *maximum* score against the EWMA baseline.
    pub max_z_score: f64,

    /// Z-score of the *mean* score against the EWMA baseline.
    pub mean_z_score: f64,

    /// Snapshot of the fast EWMA baseline this distribution was scored against.
    pub baseline: BaselineSnapshot,

    /// CUSUM drift accumulator for this scoring axis.
    pub cusum: CusumSnapshot,

    /// Current clip-pressure EWMA for this axis: ρ̄ ∈ [0, 1]  (§ALGO S-14.4).
    pub clip_pressure: f64,
}

// ─── Baseline snapshot ──────────────────────────────────────

/// Frozen snapshot of an EWMA baseline at the time of scoring.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BaselineSnapshot {
    /// Current EWMA mean of "normal" scores.
    pub mean: f64,

    /// Current EWMA variance of "normal" scores.
    pub variance: f64,
}

// ─── CUSUM snapshot ─────────────────────────────────────────

/// Frozen snapshot of a CUSUM accumulator at the time of scoring.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CusumSnapshot {
    /// Current CUSUM accumulator value.
    pub accumulator: f64,

    /// Snapshot of the slow EWMA baseline used as the CUSUM reference.
    pub slow_baseline: BaselineSnapshot,

    /// Number of batches since the CUSUM was last reset.
    pub steps_since_reset: u64,
}

// ─── Maturity ───────────────────────────────────────────────

/// How much experience a tracker has, and how much of that is noise.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct TrackerMaturity {
    /// Number of real (non-noise) observations this tracker has processed.
    pub real_observations: u64,

    /// Number of noise observations injected into this tracker.
    pub noise_observations: u64,

    /// Estimated fraction of the baseline **not yet established by
    /// real data**.
    pub noise_influence: f64,
}

impl TrackerMaturity {
    /// A tracker with no observations of any kind.
    #[must_use]
    pub const fn cold() -> Self {
        Self {
            real_observations: 0,
            noise_observations: 0,
            noise_influence: 1.0,
        }
    }

    /// Total observations (real + noise).
    #[must_use]
    pub const fn total_observations(&self) -> u64 {
        self.real_observations + self.noise_observations
    }
}

// ─── Scoring geometry ───────────────────────────────────────

/// Geometric properties of the tracker's scoring state.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ScoringGeometry {
    /// Working dimensionality of the tracker's input space.
    pub dim: usize,

    /// Maximum rank this tracker can reach: `min(dim, max_rank)`.
    pub cap: usize,

    /// Residual degrees of freedom: `dim - rank`.
    pub residual_dof: usize,
}

impl ScoringGeometry {
    /// Whether the novelty axis is structurally degenerate
    /// (`residual_dof == 0`).
    #[must_use]
    pub const fn is_novelty_saturated(&self) -> bool {
        self.residual_dof == 0
    }

    /// Whether the novelty axis *can* become degenerate as rank
    /// adapts (`cap >= dim`).
    #[must_use]
    pub const fn is_novelty_saturable(&self) -> bool {
        self.cap >= self.dim
    }
}

// ─── Per-sample scores ──────────────────────────────────────

/// Raw anomaly scores for a single observation.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct SampleScore {
    /// Residual energy per residual DOF (novelty axis).
    pub novelty: f64,

    /// Cell displacement score, bounded in `[0, 1)`.
    pub displacement: f64,

    /// Mahalanobis distance per rank (surprise axis).
    pub surprise: f64,

    /// Cross-correlation deviation (coherence axis).
    pub coherence: f64,

    /// Z-score of `novelty` against its EWMA baseline.
    pub novelty_z: f64,

    /// Z-score of `displacement` against its EWMA baseline.
    pub displacement_z: f64,

    /// Z-score of `surprise` against its EWMA baseline.
    pub surprise_z: f64,

    /// Z-score of `coherence` against its EWMA baseline.
    pub coherence_z: f64,
}

// ─── Health ─────────────────────────────────────────────────

/// Operational health snapshot of the entire sentinel.
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct HealthReport {
    /// Total live G-nodes in the G-V Graph.
    pub total_g_nodes: usize,

    /// Number of semi-internal G-nodes — those with one subdivided half and
    /// one that still accumulates locally. They sit on the contour alongside
    /// the terminals.
    pub semi_internal_count: usize,

    /// Number of active cell trackers (total).
    pub active_trackers: usize,

    /// Number of active competitive trackers: $|\mathcal{A}|$.
    pub active_competitive_trackers: usize,

    /// Number of active ancestor-only trackers: $|\mathcal{A}^*| - |\mathcal{A}| - 1$.
    ///
    /// The −1 accounts for the permanent root tracker, which is
    /// neither competitive nor a normal ancestor.
    pub active_ancestor_trackers: usize,

    /// Number of active coordination contexts.
    pub active_coordination_contexts: usize,

    /// Total cells with allocated trackers (online + warming):
    /// $|\mathcal{I}|$ (the investment set, §ALGO S-8.2, ADR-S-019).
    pub investment_set_size: usize,

    /// Members of $\mathcal{I}$ currently in the warm-up pipeline
    /// (§ALGO S-11.6, ADR-S-019).
    pub warming_trackers: usize,

    /// Competitive targets ($\mathcal{T}$) not yet promoted to
    /// $\mathcal{A}$ — i.e. warming cells that are competitive
    /// targets, not ancestors (§ALGO S-14.11, ADR-S-019).
    pub warming_competitive_targets: usize,

    /// Total real observations across the sentinel's lifetime.
    pub lifetime_observations: u64,

    /// Number of cells with a live tracker — the same figure as
    /// `active_trackers`, kept because it is part of the published shape of
    /// this report. It is not the size of the analysis set, which also
    /// names cells still warming and is reported as `investment_set_size`.
    pub cells_tracked: usize,

    /// Distribution of ranks across all active trackers.
    pub rank_distribution: RankDistribution,

    /// Distribution of maturity across all active trackers.
    pub maturity_distribution: MaturityDistribution,

    /// Distribution of geometric scoring reliability across all
    /// active per-cell trackers.
    pub geometry_distribution: GeometryDistribution,

    /// Health of the coordination tier.
    pub coordination_health: CoordinationHealth,

    /// Clip-pressure distribution across active trackers (§ALGO S-14.11).
    pub clip_pressure_distribution: ClipPressureDistribution,
}

/// Summary of rank values across all active trackers.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct RankDistribution {
    /// Lowest rank among all trackers.
    pub min: usize,

    /// Highest rank among all trackers.
    pub max: usize,

    /// Mean rank across all trackers.
    pub mean: f64,
}

/// Summary of maturity across all active trackers.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct MaturityDistribution {
    /// Highest noise influence among all trackers (least mature).
    pub max_noise_influence: f64,

    /// Lowest noise influence among all trackers (most mature).
    pub min_noise_influence: f64,

    /// Mean noise influence across all trackers.
    pub mean_noise_influence: f64,

    /// Number of trackers with zero real observations.
    pub cold_trackers: usize,
}

// ─── Geometry distribution ──────────────────────────────────

/// Summary of geometric scoring reliability across a set of trackers.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct GeometryDistribution {
    /// Trackers where `rank == dim` (novelty axis degenerate).
    pub novelty_saturated: usize,

    /// Trackers where `cap >= dim` (novelty *can* become degenerate).
    pub novelty_saturable: usize,

    /// Trackers where `rank < 2` (coherence axis does not exist).
    pub coherence_inactive: usize,
}

// ─── Clip-pressure distribution ─────────────────────────────

/// Summary of clip-pressure EWMA values across active trackers (§ALGO S-14.11).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ClipPressureDistribution {
    /// Minimum clip-pressure EWMA across active tracker axes.
    pub min: f64,

    /// Maximum clip-pressure EWMA across active tracker axes.
    pub max: f64,

    /// Mean clip-pressure EWMA across active tracker axes.
    pub mean: f64,
}

// ─── Coordination health ────────────────────────────────────

/// Health snapshot of the hierarchical coordination tier.
///
/// Summarises the active coordination contexts — subspace trackers
/// operating at $w = 4$ on cross-cell score patterns.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CoordinationHealth {
    /// Number of active coordination contexts.
    pub active_contexts: usize,

    /// Capacity (max rank) of the coordination trackers.
    /// Always min(4, `max_rank`) since $w = 4$.
    pub capacity: usize,

    /// Rank distribution across active contexts.
    pub rank_distribution: RankDistribution,

    /// Maturity distribution across active contexts.
    pub maturity_distribution: MaturityDistribution,

    /// Working dimensionality (always 4).
    pub dim: usize,

    /// Geometry distribution across active contexts.
    pub geometry_distribution: GeometryDistribution,
}

// ─── Axis baseline snapshots ────────────────────────────────

/// Per-axis EWMA baseline snapshots for all four scoring axes.
///
/// Provides read-only access to the learned baseline statistics
/// without requiring direct access to the tracker internals.
/// Used by convergence tests to verify EWMA settling behaviour
/// (ADR-S-014).
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AxisBaselineSnapshots {
    /// Novelty axis baseline.
    pub novelty: BaselineSnapshot,

    /// Displacement axis baseline.
    pub displacement: BaselineSnapshot,

    /// Surprise axis baseline.
    pub surprise: BaselineSnapshot,

    /// Coherence axis baseline.
    pub coherence: BaselineSnapshot,
}

// ─── Cell inspection ────────────────────────────────────────

/// Snapshot of a cell's tracker state.
///
/// Returned by [`SpectralSentinel::inspect_cell`](crate::SpectralSentinel::inspect_cell).
#[derive(Debug, Clone)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct CellInspection<C: Copy + Debug> {
    /// Arena handle of the backing G-node.
    pub gnode_id: GNodeId,

    /// Lower bound of the dyadic interval (inclusive).
    pub start: C,

    /// Upper bound of the dyadic interval, exclusive everywhere except at the
    /// top of the domain: the cell whose bound is the domain maximum owns that
    /// maximum, because a coordinate width filling the coordinate type leaves
    /// no value above it to be excluded.
    pub end: C,

    /// G-tree depth of this cell.
    pub depth: u32,

    /// Suffix width: `N - depth`.
    pub analysis_width: usize,

    /// Whether this cell is competitively selected (vs ancestor-only).
    pub is_competitive: bool,

    /// Current rank (number of active basis vectors).
    pub rank: usize,

    /// Fraction of total variance captured by the current rank.
    pub energy_ratio: f64,

    /// Largest singular value of the learned subspace.
    pub top_singular_value: f64,

    /// Maturity state.
    pub maturity: TrackerMaturity,

    /// Geometric scoring properties.
    pub geometry: ScoringGeometry,

    /// Per-axis EWMA baseline snapshots (ADR-S-014).
    pub baselines: AxisBaselineSnapshots,
}

// ─── Contour snapshot ───────────────────────────────────────

/// Snapshot of the G-V Graph's spatial contour at report time.
///
/// The contour is the observable surface of the spatial structure —
/// how many distinct regions exist, how many leaf cells, and how
/// much total traffic volume the graph has accumulated.
///
/// Populated from `GvGraph::plateaus()`, `GvGraph::terminal_count()`,
/// and `GvGraph::total_sum()`.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContourSnapshot {
    /// Number of plateaus in the G-V Graph's spatial structure.
    ///
    /// A plateau is a contiguous range of cells at the same
    /// depth. Fewer plateaus → more uniform spatial resolution.
    pub plateau_count: usize,

    /// Number of cells on the contour: the terminal nodes together with the
    /// semi-internal ones, whose unsubdivided half still accumulates locally
    /// and is a cell in its own right.
    ///
    /// This is the spatial resolution: how many non-overlapping
    /// regions the domain is partitioned into. It may differ from the number
    /// of cells in the batch report, which also carries the ancestors above
    /// the contour.
    pub cell_count: usize,

    /// Total accumulated importance across the entire G-V Graph.
    ///
    /// Erased to `f64` via `V::to_f64_approx()` — the concrete
    /// accumulator type is hidden from report consumers.
    /// Values above 2^53 may lose LSBs (acceptable for diagnostics).
    pub total_importance: f64,

    /// Cells created by catalytic or bootstrap bisection since the
    /// previous report.
    ///
    /// Computed exactly from graph state deltas:
    /// `splits = Δnode_count − Δterminal_count`.
    ///
    /// Saturates at [`u32::MAX`] in the (practically unreachable)
    /// event of more than ~4 × 10⁹ splits between consecutive reports.
    pub splits_since_last_report: u32,

    /// Net structural removals since the previous report:
    /// evictions minus restorations (legacy promotions).
    ///
    /// Restorations are rare (they occur only when an eviction
    /// leaves a semi-internal node); this value typically equals
    /// the raw eviction count.
    ///
    /// Computed exactly from graph state deltas:
    /// `net_removals = splits − Δterminal_count`.
    ///
    /// Saturates at [`u32::MAX`] under the same caveat as
    /// [`Self::splits_since_last_report`].
    pub net_removals_since_last_report: u32,
}

// ─── Analysis set summary ───────────────────────────────────

/// Summary of the analysis set at report time.
///
/// Describes the investment set and producing sets without
/// enumerating every cell. For the complete set, use
/// [`SpectralSentinel::analysis_set()`](crate::SpectralSentinel::analysis_set).
///
/// See §ALGO S-14.12, ADR-S-019.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct AnalysisSetSummary {
    /// Number of online competitive targets: $|\mathcal{A}|$
    /// (the producing competitive set, §ALGO S-8.3).
    ///
    /// Always ≤ `analysis_k` from the configuration.
    pub competitive_size: usize,

    /// Total online cells in the producing full set: $|\mathcal{A}^*|$
    /// (§ALGO S-8.3).
    ///
    /// Includes online competitive targets, their online G-tree
    /// ancestors, and the permanent root tracker.
    pub full_size: usize,

    /// Total cells with allocated trackers (online + warming):
    /// $|\mathcal{I}|$ (the investment set, §ALGO S-8.2).
    ///
    /// `investment_set_size = full_size + warming trackers`.
    pub investment_set_size: usize,

    /// (min, max) G-tree depth across the full analysis set.
    ///
    /// Depth 0 = root (always present). Max depth reflects
    /// the finest spatial resolution currently being analysed.
    pub depth_range: (u32, u32),

    /// (min, max) importance across the competitive set.
    ///
    /// Erased to `f64` via `V::to_f64_approx()` — the concrete
    /// accumulator type is hidden from report consumers.
    /// `(0.0, 0.0)` when `competitive_size == 0`.
    pub importance_range: (f64, f64),

    /// (min, max) V-Tree depth across the competitive set.
    ///
    /// V-Tree depth reflects competitive standing — lower = more
    /// significant. `(0, 0)` when `competitive_size == 0`.
    pub v_depth_range: (usize, usize),

    /// Number of G-tree nodes excluded while producing the current analysis-set snapshot because their suffix width was below `MIN_TRACKER_DIM`.
    ///
    /// Recomputed with selection rather than accumulated over the sentinel's lifetime. A persistently non-zero count may indicate the `split_threshold` is too low for the traffic mix.
    /// See ADR-S-011.
    pub degenerate_cells_skipped: usize,
}

// ─── Member score ───────────────────────────────────────────

/// Per-cell scores from a coordination context's model.
///
/// Each entry corresponds to one competitive cell in the coordination
/// group. The `cell_start`/`cell_end`/`cell_depth` fields identify
/// which cell produced this score vector.
#[derive(Debug, Clone, Copy)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[cfg_attr(feature = "serde", serde(bound = "C: serde::Serialize + serde::de::DeserializeOwned"))]
pub struct MemberScore<C: Copy + Debug> {
    /// Lower bound of the scored cell's dyadic interval (inclusive).
    pub cell_start: C,

    /// Upper bound of the scored cell's dyadic interval (exclusive).
    pub cell_end: C,

    /// G-tree depth of the scored cell.
    pub cell_depth: u32,

    /// Novelty score for this cell's contribution.
    pub novelty: f64,

    /// Displacement score for this cell's contribution.
    pub displacement: f64,

    /// Surprise score for this cell's contribution.
    pub surprise: f64,

    /// Coherence score for this cell's contribution.
    pub coherence: f64,

    /// Z-score of `novelty` against the coordination baseline.
    pub novelty_z: f64,

    /// Z-score of `displacement` against the coordination baseline.
    pub displacement_z: f64,

    /// Z-score of `surprise` against the coordination baseline.
    pub surprise_z: f64,

    /// Z-score of `coherence` against the coordination baseline.
    pub coherence_z: f64,
}
