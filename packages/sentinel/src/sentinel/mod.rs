// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The sentinel engine — orchestration, tracking, and drift detection.
//!
//! This module contains the core [`SpectralSentinel`] orchestrator and
//! its internal machinery.  Only the orchestrator is part of the public
//! API; the subspace tracker and CUSUM accumulator are implementation
//! details.
//!
//! # Automatic noise injection (§ALGO S-11, §ALGO S-18.2)
//!
//! Every newly created tracker is automatically warmed with synthetic
//! noise before it receives any real observations. The root tracker
//! is warmed at construction; cells created during investment set
//! reconciliation (§ALGO S-8.5) are enqueued into a **staging area**
//! for deferred warm-up (§ALGO S-11.6). When `background_warming` is
//! enabled, a background thread drains the staging area asynchronously;
//! otherwise warm-up runs synchronously within `reconcile_analysis_set()`.
//! Warming cells hold **investment slots** in $\mathcal{I}$ but not
//! **production slots** in $\mathcal{A}$ (ADR-S-019).
//! Coordination contexts are warmed via Gamma-sampled synthetic
//! score vectors (§ALGO S-9.8). No manual injection API exists — the
//! sentinel owns the injection lifecycle entirely (ADR-S-007).
//!
//! # Quick start
//!
//! ```
//! use torrust_sentinel::SentinelConfig;
//! use torrust_sentinel::Sentinel128;
//!
//! let cfg = SentinelConfig::<u64> {
//!     analysis_k: 4,
//!     ..SentinelConfig::default()
//! };
//! // Root tracker is auto-warmed at construction.
//! let mut sentinel = Sentinel128::new(cfg).unwrap();
//!
//! let values: Vec<u128> = vec![
//!     0xF000_0000_0000_0000_0000_0000_0000_0001,
//!     0xF000_0000_0000_0000_0000_0000_0000_0002,
//!     0x1000_0000_0000_0000_0000_0000_0000_0003,
//! ];
//! let report = sentinel.ingest(&values);
//!
//! // Root cell always receives all observations (as an ancestor).
//! assert!(!report.ancestor_reports.is_empty());
//! ```

pub mod cusum;
pub mod staging;
pub mod tracker;
pub mod warming_thread;

use std::collections::{BTreeMap, BTreeSet};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use rand::rngs::SmallRng;
use rand::{RngExt, SeedableRng};
use rand_distr::{Distribution, Gamma};
use torrust_mudlark::{Config as GvConfig, Coordinate, GNodeId, GvGraph, Inspectable};

use self::tracker::SubspaceTracker;
use crate::{
    AnalysisSet, AxisBaselineSnapshots, BatchReport, CellInspection, CellReport, CentredBits, ClipPressureDistribution,
    ConfigError, ConfigErrors, ContourSnapshot, CoordinationHealth, CoordinationReport, GeometryDistribution, HealthReport,
    MaturityDistribution, MemberScore, RankDistribution, SentinelConfig,
};

// ─── Internal state types ───────────────────────────────────

/// Per-cell analysis state.
///
/// Each cell in the investment set $\mathcal{I}$ owns a single
/// `SubspaceTracker` operating on suffix bits at the cell's G-tree
/// depth. The root cell ($d = 0$) has `width = 128`; a cell at
/// depth $d$ has `width = 128 - d`.
///
/// Online cells (the producing sets $\mathcal{A}$/$\mathcal{A}^*$)
/// receive real observations; warming cells receive only synthetic
/// noise (ADR-S-019).
pub struct CellState<C: Coordinate> {
    /// The subspace tracker for this cell.
    pub tracker: SubspaceTracker,

    /// G-tree depth of this cell.
    pub depth: u32,

    /// Suffix width: `128 - depth`. Cached for convenience.
    pub width: usize,

    /// Lower bound of the dyadic interval (inclusive).
    pub start: C,

    /// Upper bound of the dyadic interval, exclusive everywhere except at the
    /// top of the domain: the cell whose bound is the domain maximum owns that
    /// maximum, because a coordinate width filling the coordinate type leaves
    /// no value above it to be excluded.
    pub end: C,

    /// Whether this cell is competitively selected (vs ancestor-only).
    pub is_competitive: bool,
}

/// Per-G-node coordination state (§ALGO S-9.1).
///
/// Active when both subtrees of this G-node contain competitive
/// cells that produced scores in at least one batch since
/// activation.
struct CoordContext {
    /// Subspace tracker at w = 4, using `cusum_coord_slow_decay`.
    tracker: SubspaceTracker,

    /// Running EWMA mean of the 4D cell-score input vectors (§ALGO S-9.3).
    /// Used for centring before feeding the coordination tracker.
    running_mean: [f64; 4],

    /// Whether the running mean has been initialised with at least
    /// one batch. Cold-start: first batch sets
    /// `running_mean = colmeans(O_g)` (§ALGO S-9.3).
    warm: bool,
}

// ─── SpectralSentinel ───────────────────────────────────────

/// Hierarchical online subspace anomaly detector.
///
/// The sentinel maintains one `SubspaceTracker` per analysis cell
/// in the full analysis set. Cells are selected by the analysis
/// selector (§ALGO S-4.1) from the G-V Graph's V-Tree.
///
/// A second tier of coordination trackers — one per active G-tree
/// internal node whose subtrees both contribute competitive cells —
/// analyses cross-cell score patterns for coordinated anomalies
/// (§ALGO S-9.1).
///
/// # Design principle
///
/// **The sentinel measures; the host decides.**
///
/// [`ingest`](Self::ingest) returns a [`BatchReport`] containing raw
/// statistical measurements. The host reads the report and applies
/// its own policy to decide what (if anything) to do.
///
/// **Feed-forward invariant (ADR-S-002).** The G-V Graph receives
/// only `observe(v, 1u64)` per raw input value during `ingest()`.
/// Anomaly scores and derived signals never flow back into the
/// graph's importance accounting. The host controls temporal policy
/// via `decay()`; the analysis tier has no influence on spatial
/// resolution.
pub struct SpectralSentinel<C, V, const N: u32>
where
    C: Coordinate + crate::CentredBitSource,
    V: Inspectable + torrust_mudlark::Attenuatable,
{
    config: SentinelConfig<V>,

    /// The G-V Graph spatial substrate (§ALGO S-2.4).
    ///
    /// Owns the adaptive spatial partition of the observation domain.
    graph: GvGraph<C, V, N>,

    /// Per-cell analysis state, keyed by `GNodeId`.
    ///
    /// `BTreeMap` for deterministic iteration order (ADR-S-005).
    /// Contains one entry per **online** cell in the producing full
    /// set $\mathcal{A}^*$. Warming cells are in the staging area
    /// (ADR-S-019).
    cells: BTreeMap<GNodeId, CellState<C>>,

    /// The current analysis set (competitive + ancestors).
    ///
    /// Recomputed after every observation pass (ADR-S-006).
    analysis_set: AnalysisSet<C, V>,

    /// The root tracker's `GNodeId`. Cached for fast access.
    /// Permanent — never destroyed (§ALGO S-4.7).
    root_gnode: GNodeId,

    /// Per-G-node coordination contexts (§ALGO S-9.1).
    ///
    /// Keyed by `GNodeId` of internal G-nodes whose subtrees
    /// contain competitive cells in both left and right branches.
    /// `BTreeMap` for deterministic iteration order (ADR-S-005).
    coordination: BTreeMap<GNodeId, CoordContext>,

    /// Monotonically increasing counter, incremented once per
    /// `ingest` call.
    batch_counter: u64,

    /// Total real (non-noise) observations across the sentinel's
    /// lifetime.
    lifetime_observations: u64,

    /// Number of G-tree nodes excluded from tracking because their
    /// suffix width was below `MIN_TRACKER_DIM` (ADR-S-011).
    degenerate_cells_skipped: usize,

    /// Persistent RNG for noise injection (§ALGO S-11.1).
    noise_rng: SmallRng,

    /// Staging area for cells undergoing deferred noise warm-up
    /// (§ALGO S-18.2).
    staging: Arc<Mutex<staging::StagingArea<C>>>,

    /// Handle to the background warming thread (Step 3).
    warming_thread: Option<warming_thread::WarmingThreadHandle<C>>,

    /// Snapshot of `graph.terminal_count()` at the end of the
    /// previous `ingest()`.  Used to derive structural mutation
    /// counts without requiring graph-internal event counters.
    prev_terminal_count: u32,

    /// Snapshot of `graph.node_count()` at the end of the
    /// previous `ingest()`.
    prev_node_count: u32,
}

impl<C, V, const N: u32> SpectralSentinel<C, V, N>
where
    C: Coordinate + crate::CentredBitSource,
    V: Inspectable + torrust_mudlark::Attenuatable,
{
    /// Create a new sentinel with the given configuration.
    ///
    /// Validates the configuration and creates the root tracker.
    /// No other cells are created until the first
    /// [`ingest`](Self::ingest) call triggers analysis set computation.
    ///
    /// # Errors
    ///
    /// Returns [`ConfigErrors`] if the
    /// configuration violates any invariant (see
    /// [`SentinelConfig::validate`]), or if the coordinate width `N` is
    /// narrower than the smallest dimension a subspace tracker can model.
    /// Both faults are collected in one pass.
    pub fn new(config: SentinelConfig<V>) -> Result<Self, ConfigErrors> {
        // The root tracker spans the whole coordinate width, so a width the
        // tracker cannot model is refused here rather than left to build a
        // root whose lone basis vector spans its own space and therefore
        // reports no novelty at all. This is the only place the width can be
        // judged: it is a parameter of the type, not a field of the
        // configuration, so validation of the configuration alone can never
        // see it.
        let mut errors = Vec::new();
        if (N as usize) < crate::MIN_TRACKER_DIM {
            errors.push(ConfigError::TrackerDimensionTooSmall {
                width: N,
                minimum: crate::MIN_TRACKER_DIM,
            });
        }
        if let Err(ConfigErrors(config_errors)) = config.validate() {
            errors.extend(config_errors);
        }
        if !errors.is_empty() {
            return Err(ConfigErrors(errors));
        }

        // ── G-V Graph construction ──────────────────────
        let gv_config = GvConfig {
            split_threshold: config.split_threshold,
            depth_create: config.d_create,
            depth_evict: config.d_evict,
            budget: Some(config.budget),
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        let graph: GvGraph<C, V, N> = GvGraph::new(gv_config);

        // ── Persistent RNG (§ALGO S-11.1) ─────────────────
        let mut noise_rng = config
            .noise_seed
            .map_or_else(|| SmallRng::from_rng(&mut rand::rng()), SmallRng::seed_from_u64);

        // ── Root tracker (permanent, §ALGO S-4.7) ─────────
        let root_gnode = graph.g_root();
        let mut root_cell = CellState {
            tracker: SubspaceTracker::new(N as usize, &config, config.cusum_slow_decay),
            depth: 0,
            width: N as usize,
            start: C::zero(),
            end: C::domain_max(N),
            is_competitive: false,
        };

        // Auto noise injection on root tracker (§ALGO S-11.2).
        let root_rounds = config.noise_schedule.rounds_for_depth(0);
        if root_rounds > 0 {
            inject_noise_into_cell(&mut root_cell, root_rounds as usize, config.noise_batch_size, &mut noise_rng);
        }

        let mut cells = BTreeMap::new();
        cells.insert(root_gnode, root_cell);

        // ── Initial analysis set (just root) ────────────
        let analysis_set = AnalysisSet::recompute::<N>(&graph, config.analysis_k, config.analysis_depth_cutoff);

        let staging = Arc::new(Mutex::new(staging::StagingArea::<C>::new()));

        // ── Background warming thread (Step 3) ─────────
        let warming_thread = if config.background_warming {
            Some(warming_thread::WarmingThreadHandle::<C>::spawn(
                &staging,
                config.noise_batch_size,
                config.noise_seed,
            ))
        } else {
            None
        };

        let init_terminal = graph.terminal_count();
        let init_node = graph.node_count();

        Ok(Self {
            config,
            graph,
            cells,
            analysis_set,
            root_gnode,
            coordination: BTreeMap::new(),
            batch_counter: 0,
            lifetime_observations: 0,
            degenerate_cells_skipped: 0,
            noise_rng,
            staging,
            warming_thread,
            prev_terminal_count: init_terminal,
            prev_node_count: init_node,
        })
    }

    /// Process a batch of raw `u128` observations and return a full
    /// statistical report.
    ///
    /// Each value is fed to the G-V Graph, then routed to every
    /// analysis cell whose interval contains it. Multi-scale delivery
    /// ensures ancestor cells also receive the observation (§ALGO S-4.4).
    ///
    /// An empty input slice produces an empty report.
    ///
    /// # Panics
    ///
    /// Panics if the internal staging mutex is poisoned.
    pub fn ingest(&mut self, values: &[C]) -> BatchReport<C> {
        // ── Early return on empty input ─────────────────
        if values.is_empty() {
            return self.empty_report();
        }

        // ── Arrival stamp ─────────────────────────────────
        // (´sec:sentinel:algorithm-output-report-overview´)
        // The batch arrives whole at this call boundary, so this one
        // instant is the arrival of every observation in it — the
        // oldest included. It is read back at emission and reported as
        // the batch's age. The clock is the sentinel's own monotonic
        // one and is never compared with anybody else's, which is what
        // keeps the figure free of skew.
        let batch_arrival = Instant::now();

        // ── Observation algorithm ──────────────────────
        // Steps 0–6 correspond to §ALGO S-9.1.
        // The implementation reorders Step 1 (encoding) to just
        // before Step 4 (scoring), since the spatial layer routes
        // on raw coordinates and does not need centred bits.

        // ── Step 0: Promote ready cells (§ALGO S-18.2) ────
        // Cells that completed background warm-up since the last
        // ingest are moved into the live cells map. In the
        // synchronous transitional version (Step 2) this promotes
        // cells from the drain at the end of reconcile_analysis_set();
        // once the background thread is introduced (Step 3) it will
        // catch cells that finished between ingest calls.
        self.promote_ready_cells();

        // ── Step 2: G-V Graph observation (§ALGO S-8.1) ───
        // [Step 1 (encoding) deferred to just before scoring.]
        let unit_delta = V::from_f64(1.0);

        #[cfg(debug_assertions)]
        let pre_observe_sum = self.graph.total_sum().to_f64_approx();

        for &value in values {
            self.graph.observe(value, unit_delta);
        }

        #[cfg(debug_assertions)]
        {
            #[allow(clippy::cast_precision_loss)]
            let expected_sum = pre_observe_sum + values.len() as f64;
            let actual_sum = self.graph.total_sum().to_f64_approx();
            debug_assert!(
                (actual_sum - expected_sum).abs() < 1.0,
                "feed-forward invariant violated: total_sum should increase by exactly n"
            );
        }

        // ── Step 3: Analysis set reconciliation ─────────
        self.reconcile_analysis_set();

        // ── Steps 1+4: Encode and route/score ───────────
        let centred: Vec<CentredBits> = values.iter().map(|v| CentredBits::from_coord(v, N)).collect();
        let cell_reports = self.route_and_score(values, &centred);

        // ── Step 5: Coordination tier (§ALGO S-9.4) ───────
        let cell_scores = Self::assemble_cell_scores(&cell_reports);
        let coordination_reports = self.propagate_coordination_from_root(&cell_scores);

        // ── Step 6: Assemble report ─────────────────────
        self.batch_counter += 1;

        let count = values.len() as u64;
        self.lifetime_observations += count;

        // ── (Step 6 continued: report assembly) ──────────
        let (competitive, ancestors): (Vec<_>, Vec<_>) = cell_reports.into_iter().partition(|r| r.is_competitive);

        let (splits, net_removals, terminal_count) = self.take_structural_mutation_counts();

        let contour = ContourSnapshot {
            plateau_count: self.graph.plateaus().len(),
            cell_count: terminal_count as usize,
            total_importance: self.graph.total_sum().to_f64_approx(),
            splits_since_last_report: splits,
            net_removals_since_last_report: net_removals,
        };

        let mut analysis_set_summary = self.analysis_set.summary();
        analysis_set_summary.degenerate_cells_skipped = self.degenerate_cells_skipped;
        // Investment set = online cells + warming cells (ADR-S-019).
        let warming_count = self.staging.lock().expect("staging mutex poisoned").total_count();
        analysis_set_summary.investment_set_size = self.cells.len() + warming_count;

        BatchReport {
            cell_reports: competitive,
            ancestor_reports: ancestors,
            coordination_reports,
            contour,
            health: self.health(),
            analysis_set_summary,
            // Read last, so the age covers every part of the work this
            // call did on the batch, the health snapshot included.
            // Saturates rather than wrapping; the ceiling is hundreds
            // of thousands of years away.
            oldest_observation_age_micros: Some(u64::try_from(batch_arrival.elapsed().as_micros()).unwrap_or(u64::MAX)),
        }
    }

    /// Produce an operational health snapshot of the sentinel.
    ///
    /// Summarises rank distribution and maturity across all active
    /// trackers. Useful for dashboards.
    ///
    /// # Panics
    ///
    /// Panics if the internal staging mutex is poisoned.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn health(&self) -> HealthReport {
        let competitive_count = self.analysis_set.competitive_count();
        let total_cells = self.analysis_set.total_count();
        let active_trackers = self.cells.len();
        let coord_health = self.coordination_health();

        // Query staging area for investment/warming counts (ADR-S-019).
        let (warming_total, warming_competitive) = {
            let staging = self.staging.lock().expect("staging mutex poisoned");
            (staging.total_count(), staging.warming_competitive_count())
        };
        let investment_set_size = active_trackers + warming_total;

        if active_trackers == 0 {
            return HealthReport {
                total_g_nodes: self.graph.node_count() as usize,
                // TODO ´todo:code:empty-tracker-health-branch-hardcodes-semi´: empty-tracker health branch hardcodes semi-internal count.
                semi_internal_count: 0,
                active_trackers: 0,
                active_competitive_trackers: 0,
                active_ancestor_trackers: 0,
                active_coordination_contexts: 0,
                investment_set_size,
                warming_trackers: warming_total,
                warming_competitive_targets: warming_competitive,
                lifetime_observations: self.lifetime_observations,
                cells_tracked: 0,
                rank_distribution: RankDistribution {
                    min: 0,
                    max: 0,
                    mean: 0.0,
                },
                maturity_distribution: MaturityDistribution {
                    max_noise_influence: 0.0,
                    min_noise_influence: 0.0,
                    mean_noise_influence: 0.0,
                    cold_trackers: 0,
                },
                geometry_distribution: GeometryDistribution {
                    novelty_saturated: 0,
                    novelty_saturable: 0,
                    coherence_inactive: 0,
                },
                coordination_health: coord_health,
                clip_pressure_distribution: ClipPressureDistribution {
                    min: 0.0,
                    max: 0.0,
                    mean: 0.0,
                },
            };
        }

        let mut rank_min = usize::MAX;
        let mut rank_max = 0_usize;
        let mut rank_sum = 0_u64;

        let mut ni_min = f64::INFINITY;
        let mut ni_max = f64::NEG_INFINITY;
        let mut ni_sum = 0.0_f64;
        let mut cold = 0_usize;

        let mut geo_saturated = 0_usize;
        let mut geo_saturable = 0_usize;
        let mut geo_coh_inactive = 0_usize;

        let mut cp_min = f64::INFINITY;
        let mut cp_max = f64::NEG_INFINITY;
        let mut cp_sum = 0.0_f64;
        let mut cp_count = 0_u64;

        for cell in self.cells.values() {
            let tracker = &cell.tracker;
            let r = tracker.rank();
            rank_min = rank_min.min(r);
            rank_max = rank_max.max(r);
            rank_sum += r as u64;

            let m = tracker.maturity();
            ni_min = ni_min.min(m.noise_influence);
            ni_max = ni_max.max(m.noise_influence);
            ni_sum += m.noise_influence;
            if m.real_observations == 0 {
                cold += 1;
            }

            let g = tracker.scoring_geometry();
            if g.is_novelty_saturated() {
                geo_saturated += 1;
            }
            if g.is_novelty_saturable() {
                geo_saturable += 1;
            }
            if r < 2 {
                geo_coh_inactive += 1;
            }

            for cp in cell.tracker.clip_pressures() {
                cp_min = cp_min.min(cp);
                cp_max = cp_max.max(cp);
                cp_sum += cp;
                cp_count += 1;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let n = active_trackers as f64;

        #[allow(clippy::cast_precision_loss)]
        let rank_mean = rank_sum as f64 / n;

        HealthReport {
            total_g_nodes: self.graph.node_count() as usize,
            // TODO ´todo:code:active-tracker-health-branch-hardcodes-semi´: active-tracker health branch hardcodes semi-internal count.
            semi_internal_count: 0,
            active_trackers,
            active_competitive_trackers: competitive_count,
            active_ancestor_trackers: total_cells.saturating_sub(competitive_count + 1),
            active_coordination_contexts: self.coordination.len(),
            investment_set_size,
            warming_trackers: warming_total,
            warming_competitive_targets: warming_competitive,
            lifetime_observations: self.lifetime_observations,
            cells_tracked: self.cells.len(),
            rank_distribution: RankDistribution {
                min: rank_min,
                max: rank_max,
                mean: rank_mean,
            },
            maturity_distribution: MaturityDistribution {
                max_noise_influence: ni_max,
                min_noise_influence: ni_min,
                mean_noise_influence: ni_sum / n,
                cold_trackers: cold,
            },
            geometry_distribution: GeometryDistribution {
                novelty_saturated: geo_saturated,
                novelty_saturable: geo_saturable,
                coherence_inactive: geo_coh_inactive,
            },
            coordination_health: coord_health,
            clip_pressure_distribution: ClipPressureDistribution {
                min: if cp_count > 0 { cp_min } else { 0.0 },
                max: if cp_count > 0 { cp_max } else { 0.0 },
                #[allow(clippy::cast_precision_loss)]
                mean: if cp_count > 0 { cp_sum / cp_count as f64 } else { 0.0 },
            },
        }
    }

    /// Number of cells in the full analysis set.
    #[must_use]
    pub fn cells_tracked(&self) -> usize {
        self.cells.len()
    }

    /// Total real observations processed across the sentinel's lifetime.
    #[must_use]
    pub const fn lifetime_observations(&self) -> u64 {
        self.lifetime_observations
    }

    /// Number of G-tree nodes excluded from tracking because their
    /// suffix width was below `MIN_TRACKER_DIM` (ADR-S-011).
    #[must_use]
    pub const fn degenerate_cells_skipped(&self) -> usize {
        self.degenerate_cells_skipped
    }

    /// Read-only access to the configuration.
    #[must_use]
    pub const fn config(&self) -> &SentinelConfig<V> {
        &self.config
    }

    /// Read-only access to the G-V Graph.
    #[must_use]
    pub const fn graph(&self) -> &GvGraph<C, V, N> {
        &self.graph
    }

    /// Read-only access to the current analysis set.
    #[must_use]
    pub const fn analysis_set(&self) -> &AnalysisSet<C, V> {
        &self.analysis_set
    }

    /// Reset the sentinel to its freshly-constructed state.
    ///
    /// Drops all cell trackers and their learned subspaces,
    /// re-initialises the G-V Graph, and zeroes all counters.
    /// The configuration is preserved.
    ///
    /// # Panics
    ///
    /// Panics if the staging area mutex is poisoned.
    pub fn reset(&mut self) {
        // Shut down background thread before clearing state (Step 3.4).
        if let Some(ref wt) = self.warming_thread {
            wt.shutdown();
        }

        self.cells.clear();
        self.coordination.clear();
        self.staging.lock().expect("staging mutex poisoned").clear();
        self.batch_counter = 0;
        self.lifetime_observations = 0;
        self.degenerate_cells_skipped = 0;

        // Reset RNG (§ALGO S-11.1).
        self.noise_rng = self
            .config
            .noise_seed
            .map_or_else(|| SmallRng::from_rng(&mut rand::rng()), SmallRng::seed_from_u64);

        // Reset the G-V Graph to a single root node.
        let gv_config = GvConfig {
            split_threshold: self.config.split_threshold,
            depth_create: self.config.d_create,
            depth_evict: self.config.d_evict,
            budget: Some(self.config.budget),
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        self.graph = GvGraph::new(gv_config);
        self.root_gnode = self.graph.g_root();

        // Re-baseline the structural mutation snapshots so the next
        // report does not attribute the reset's teardown to splits
        // or evictions.
        self.prev_terminal_count = self.graph.terminal_count();
        self.prev_node_count = self.graph.node_count();

        // Recreate the root tracker with auto noise injection. The width
        // needs no second judgement here: it is fixed by the type, the sole
        // constructor refuses a width below the tracker's minimum, and a
        // reset can only be reached through an instance that constructor
        // returned. A width this method could reject could never have got
        // this far.
        let mut root_cell = CellState {
            tracker: SubspaceTracker::new(N as usize, &self.config, self.config.cusum_slow_decay),
            depth: 0,
            width: N as usize,
            start: C::zero(),
            end: C::domain_max(N),
            is_competitive: false,
        };
        if self.config.noise_schedule.rounds_for_depth(0) > 0 {
            inject_noise_into_cell(
                &mut root_cell,
                self.config.noise_schedule.rounds_for_depth(0) as usize,
                self.config.noise_batch_size,
                &mut self.noise_rng,
            );
        }
        self.cells.insert(self.root_gnode, root_cell);

        // Recompute empty analysis set.
        self.analysis_set = AnalysisSet::recompute::<N>(&self.graph, self.config.analysis_k, self.config.analysis_depth_cutoff);

        // Restart background thread if configured (Step 3.4).
        if self.config.background_warming {
            self.warming_thread = Some(warming_thread::WarmingThreadHandle::<C>::spawn(
                &self.staging,
                self.config.noise_batch_size,
                self.config.noise_seed,
            ));
        }
    }

    /// Apply spatial decay to the entire G-V Graph.
    ///
    /// Delegates to the G-V Graph's temporal filter (§ALGO S-10.1,
    /// §IDEA M-14) at the graph root, scaling accumulated importance
    /// across all cells.
    ///
    /// # Parameters
    ///
    /// - `attenuation` — base decay factor at the midpoint depth.
    ///   - `(0, 1)`: **Attenuation** — cold cells lose standing.
    ///   - `> 1.0`: **Amplification** — hot cells reinforced.
    ///   - `1.0`: no-op (identity).
    /// - `q` — depth selectivity in `[0.0, 1.0]`.
    ///   - `0.0`: uniform — all depths decay at the same rate.
    ///   - `> 0.0`: selective — coarse structure persists, fine
    ///     structure fades (or is amplified) faster.
    ///   - `1.0`: maximum selectivity.
    ///
    /// # Design
    ///
    /// The sentinel **never** calls this automatically. The host
    /// controls temporal policy: when to decay, how aggressively,
    /// and with what selectivity (§ALGO S-13.4).
    ///
    /// Decay does not trigger analysis set recomputation (the V-Tree
    /// rankings change, but no scoring happens until the next
    /// `ingest()`). The analysis set is recomputed at the start of
    /// the next `ingest()` — no eager invalidation needed.
    ///
    /// The feed-forward invariant (ADR-S-002) is maintained: decay
    /// modifies *importance* (accumulated observation counts), never
    /// tracker state (subspace, baselines, CUSUM).
    ///
    /// # Panics
    ///
    /// - `attenuation < 0.0` or `attenuation.is_nan()`.
    /// - `q < 0.0`, `q > 1.0`, or `q.is_nan()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_sentinel::SentinelConfig;
    /// use torrust_sentinel::Sentinel128;
    ///
    /// let mut s = Sentinel128::new(SentinelConfig::default()).unwrap();
    /// s.ingest(&[42_u128, 43, 44]);
    ///
    /// // Uniform 50% attenuation.
    /// s.decay(0.5, 0.0);
    /// assert!(s.graph().total_sum() < 3);
    /// ```
    pub fn decay(&mut self, attenuation: f64, q: f64) {
        let root = self.graph.g_root();
        self.graph.decay(root, attenuation, q);
    }

    /// Apply spatial decay to a subtree of the G-V Graph.
    ///
    /// Like [`decay()`](Self::decay), but targets only the G-subtree
    /// rooted at `root`. Cells outside this subtree are unaffected.
    ///
    /// # Use cases (§ALGO S-10.3)
    ///
    /// - **Regime change in a region.** Attenuate a subtree that
    ///   experienced a traffic regime shift, allowing it to re-form
    ///   under new observations without affecting global structure.
    /// - **Suspected poisoning.** `decay_subtree(root, att=0.0₊, q=1.0)`
    ///   is a detail flush: the subtree root's own count is preserved,
    ///   descendants are zeroed.
    /// - **Hot reinforcement.** `decay_subtree(root, att=1.5, q=0.0)`
    ///   amplifies a known-active subtree to boost its competitive
    ///   standing.
    ///
    /// # Parameters
    ///
    /// - `root` — the G-node whose subtree receives decay. Obtain
    ///   via `sentinel.graph().g_root()` for the global root, or
    ///   from a G-node inspection method.
    /// - `attenuation` — see [`decay()`](Self::decay).
    /// - `q` — see [`decay()`](Self::decay).
    ///
    /// # Panics
    ///
    /// - `root` does not refer to a live G-node (the graph has been
    ///   restructured since the handle was obtained).
    /// - `attenuation < 0.0` or `attenuation.is_nan()`.
    /// - `q < 0.0`, `q > 1.0`, or `q.is_nan()`.
    ///
    /// # Examples
    ///
    /// ```
    /// use torrust_sentinel::SentinelConfig;
    /// use torrust_sentinel::Sentinel128;
    ///
    /// let mut s = Sentinel128::new(SentinelConfig::default()).unwrap();
    /// s.ingest(&[42_u128, 43, 44]);
    ///
    /// let root = s.graph().g_root();
    /// s.decay_subtree(root, 0.5, 0.0);
    /// assert!(s.graph().total_sum() < 3);
    /// ```
    pub fn decay_subtree(&mut self, root: GNodeId, attenuation: f64, q: f64) {
        self.graph.decay(root, attenuation, q);
    }

    /// List all cell `GNodeId`s in the full analysis set.
    ///
    /// Returns IDs in ascending order (`BTreeMap` iteration order).
    #[must_use]
    pub fn cell_gnodes(&self) -> Vec<GNodeId> {
        self.cells.keys().copied().collect()
    }

    /// Inspect a specific cell's tracker state.
    ///
    /// Returns `None` if the cell is not in the analysis set.
    #[must_use]
    pub fn inspect_cell(&self, gnode: GNodeId) -> Option<CellInspection<C>> {
        let cell = self.cells.get(&gnode)?;
        let bl = cell.tracker.axis_baselines();
        Some(CellInspection {
            gnode_id: gnode,
            start: cell.start,
            end: cell.end,
            depth: cell.depth,
            analysis_width: cell.width,
            is_competitive: cell.is_competitive,
            rank: cell.tracker.rank(),
            energy_ratio: cell.tracker.energy_ratio(),
            top_singular_value: cell.tracker.top_singular_value(),
            maturity: cell.tracker.maturity(),
            geometry: cell.tracker.scoring_geometry(),
            baselines: AxisBaselineSnapshots {
                novelty: crate::BaselineSnapshot {
                    mean: bl.novelty_mean,
                    variance: bl.novelty_var,
                },
                displacement: crate::BaselineSnapshot {
                    mean: bl.displacement_mean,
                    variance: bl.displacement_var,
                },
                surprise: crate::BaselineSnapshot {
                    mean: bl.surprise_mean,
                    variance: bl.surprise_var,
                },
                coherence: crate::BaselineSnapshot {
                    mean: bl.coherence_mean,
                    variance: bl.coherence_var,
                },
            },
        })
    }

    // ════════════════════════════════════════════════════════
    //  Private implementation
    // ════════════════════════════════════════════════════════

    /// Reconcile the cells map with the current analysis set.
    ///
    /// Recomputes the analysis set from the V-Tree, creates trackers
    /// for new cells, and destroys trackers for exited cells. The
    /// root tracker is never destroyed (§ALGO S-4.7).
    ///
    /// New cells are enqueued into the staging area rather than being
    /// warmed inline (Step 2, §ALGO S-18.2). A synchronous drain loop
    /// warms all queued cells to completion, then promotes them into
    /// the live cells map — identical external behaviour to the old
    /// inline path. The background-thread version (Step 3) will
    /// remove the drain loop.
    fn reconcile_analysis_set(&mut self) {
        let new_set = AnalysisSet::recompute::<N>(&self.graph, self.config.analysis_k, self.config.analysis_depth_cutoff);

        // ── Identify entries and exits ──────────────────────
        let old_gnodes: BTreeSet<GNodeId> = self.cells.keys().copied().collect();
        let new_gnodes: BTreeSet<GNodeId> = new_set.full().iter().map(|e| e.gnode).collect();

        // Destroy exited cells (except root — §ALGO S-4.7).
        for &gone in old_gnodes.difference(&new_gnodes) {
            if gone != self.root_gnode {
                self.cells.remove(&gone);
            }
        }

        // Evict staging cells that exited the analysis set (Step 2.5).
        // A warming cell may have been evicted by graph rebalance.
        // Also update cached volumes and enqueue new cells — all under
        // a single lock acquisition to avoid repeated locking.
        {
            let mut staging = self.staging.lock().expect("staging mutex poisoned");

            staging.retain_in_set(&new_gnodes);

            // Update cached volumes from the graph so the background
            // thread can prioritise correctly (Step 3.2c).
            staging.update_volumes(&self.graph);

            // Enqueue entered cells into the staging area (Step 2.1).
            for entry in new_set.full() {
                if self.cells.contains_key(&entry.gnode) || staging.contains(entry.gnode) {
                    continue;
                }

                let width = (N as usize).saturating_sub(entry.depth as usize);

                // ADR-S-011: skip degenerate cells whose suffix width
                // is too narrow for a meaningful subspace model.
                if width < crate::MIN_TRACKER_DIM {
                    tracing::warn!(
                        gnode = ?entry.gnode,
                        depth = entry.depth,
                        width,
                        "skipping degenerate cell (width < MIN_TRACKER_DIM)"
                    );
                    self.degenerate_cells_skipped += 1;
                    continue;
                }

                let cell = CellState {
                    tracker: SubspaceTracker::new(width, &self.config, self.config.cusum_slow_decay),
                    depth: entry.depth,
                    width,
                    start: entry.start,
                    end: entry.end,
                    is_competitive: entry.is_competitive,
                };

                // Enqueue for deferred noise warm-up (Step 2.1).
                let rounds = self.config.noise_schedule.rounds_for_depth(entry.depth as usize);
                staging.enqueue(entry.gnode, cell, rounds);
            }

            // ── Warm-up dispatch ────────────────────────────
            if self.warming_thread.is_none() {
                // Synchronous mode: drain all warming cells in-line
                // (deterministic, matching the old inline path).
                staging.drain_all_synchronous(self.config.noise_batch_size, &mut self.noise_rng);
            }
        } // staging lock released

        // Notify background thread if running (Step 3).
        if let Some(ref wt) = self.warming_thread {
            wt.notify();
        }

        // Promote newly ready cells so they participate in routing
        // within *this* ingest call (identical to old inline path
        // in synchronous mode; in background mode, promotes cells
        // that finished since the last ingest).
        self.promote_ready_cells();

        // Update competitive flags on retained cells.
        for entry in new_set.full() {
            if let Some(cell) = self.cells.get_mut(&entry.gnode) {
                cell.is_competitive = entry.is_competitive;
            }
        }

        self.analysis_set = new_set;
    }

    /// Promote all ready cells from the staging area into the live
    /// cells map (Step 2.3, §ALGO S-18.2 Step 0).
    ///
    /// Ready cells have completed their full noise warm-up schedule.
    /// Coordination warm-up for these cells fires naturally when
    /// `propagate_coordination_from_root()` creates their coordination
    /// context on the next scoring pass.
    fn promote_ready_cells(&mut self) {
        let ready = self.staging.lock().expect("staging mutex poisoned").take_ready();
        for (gnode, cell) in ready {
            self.cells.insert(gnode, cell);
        }
    }

    /// Route observations to cells and score each cell.
    #[allow(clippy::cast_possible_truncation)] // depth ≤ 128, always fits in u8
    fn route_and_score(&mut self, values: &[C], centred: &[CentredBits]) -> Vec<CellReport<C>> {
        // ── Build per-cell observation buffers ───────────────
        // Key: GNodeId → Vec of observation indices.
        //
        // Route each observation to every active cell whose interval
        // contains it. `self.cells` holds exactly the online
        // producing set — staging cells are excluded by construction.
        let mut cell_obs: BTreeMap<GNodeId, Vec<usize>> = BTreeMap::new();

        // Cell intervals are half-open, which needs one exception at the very
        // top of the domain. When the coordinate width fills the coordinate
        // type there is no value above the maximum to serve as an exclusive
        // bound, so a half-open reading of the topmost interval excludes a
        // coordinate that is genuinely inside the domain. That observation is
        // still counted — it moves the spatial layer and it raises the
        // lifetime total — so leaving it unrouted drops a real observation
        // from every tracker while the totals go on including it. The cell
        // ending at the top of the domain therefore owns its upper bound.
        // Every other boundary stays half-open, so no coordinate can fall in
        // two sibling cells, and where the width is narrower than the type the
        // bound is a representable value outside the domain and stays
        // exclusive.
        let domain_top = C::domain_max(N);
        let width_fills_type = N == C::BITS;

        for (i, &value) in values.iter().enumerate() {
            for (&gnode, cell) in &self.cells {
                let owns_domain_top = width_fills_type && cell.end == domain_top && value == domain_top;
                if value >= cell.start && (value < cell.end || owns_domain_top) {
                    cell_obs.entry(gnode).or_default().push(i);
                }
            }
        }

        // ── Score each cell ─────────────────────────────────
        let mut reports = Vec::with_capacity(self.cells.len());

        for (&gnode, cell) in &mut self.cells {
            let obs_indices = cell_obs.get(&gnode);

            if let Some(indices) = obs_indices
                && !indices.is_empty()
            {
                let slices: Vec<&[f64]> = indices.iter().map(|&i| centred[i].suffix(cell.depth as u8)).collect();

                #[cfg(debug_assertions)]
                {
                    #[allow(clippy::cast_precision_loss)]
                    let expected_norm_sq = cell.width as f64 / 4.0;
                    for slice in &slices {
                        let norm_sq: f64 = slice.iter().map(|x| x * x).sum();
                        debug_assert!(
                            (norm_sq - expected_norm_sq).abs() < 1e-10,
                            "suffix norm² = {norm_sq}, expected w/4 = {expected_norm_sq}"
                        );
                    }
                }

                let prefix_report = cell.tracker.observe(&slices, cell.depth as u8, false);

                reports.push(CellReport {
                    gnode_id: gnode,
                    start: cell.start,
                    end: cell.end,
                    depth: cell.depth,
                    analysis_width: cell.width,
                    is_competitive: cell.is_competitive,
                    sample_count: indices.len(),
                    scores: prefix_report.scores,
                    rank: prefix_report.rank,
                    energy_ratio: prefix_report.energy_ratio,
                    top_singular_value: prefix_report.top_singular_value,
                    maturity: prefix_report.maturity,
                    geometry: prefix_report.geometry,
                    per_sample: prefix_report.per_sample,
                });
            }
            // Cells with no observations in this batch are omitted
            // from the report (§ALGO S-4.5).
        }

        reports
    }

    // ════════════════════════════════════════════════════════
    //  Hierarchical coordination (§ALGO S-9)
    // ════════════════════════════════════════════════════════

    /// Extract the 4D mean score vector from competitive cell reports.
    ///
    /// Only competitive cells with `sample_count > 0` contribute.
    fn assemble_cell_scores(cell_reports: &[CellReport<C>]) -> BTreeMap<GNodeId, [f64; 4]> {
        let mut scores = BTreeMap::new();
        for report in cell_reports {
            if report.is_competitive && report.sample_count > 0 {
                scores.insert(
                    report.gnode_id,
                    [
                        report.scores.novelty.mean,
                        report.scores.displacement.mean,
                        report.scores.surprise.mean,
                        report.scores.coherence.mean,
                    ],
                );
            }
        }
        scores
    }

    /// Run hierarchical coordination from the G-tree root.
    ///
    /// Builds the pruned coordination topology, walks bottom-up, fires
    /// coordination at internal nodes where both subtrees contribute
    /// competitive cell scores, and returns coordination reports for
    /// all active contexts (§ALGO S-9.4).
    fn propagate_coordination_from_root(&mut self, cell_scores: &BTreeMap<GNodeId, [f64; 4]>) -> Vec<CoordinationReport<C>> {
        let competitive_gnodes: BTreeSet<GNodeId> = cell_scores.keys().copied().collect();

        // Build the pruned coordination tree (§5.3).
        let Some(tree) = Self::build_coordination_tree(&self.graph, &competitive_gnodes, self.root_gnode) else {
            // No competitive cells with scores ⇒ no coordination.
            return Vec::new();
        };

        // Walk bottom-up and fire coordination at active contexts.
        let (reports, _cells) = self.walk_coordination(&tree, cell_scores);

        // Prune stale coordination contexts (§5.5).
        let active_gnodes = Self::collect_internal_gnodes(&tree);
        self.coordination.retain(|gnode, _| active_gnodes.contains(gnode));

        reports
    }

    /// Build the coordination tree topology from the G-tree.
    ///
    /// Only includes nodes reachable from the root through subtrees
    /// that contain at least one competitive cell. This prunes
    /// branches that contain no competitive cells, reducing the
    /// coordination walk from $O(|G|)$ to $O(|\mathcal{A}| \cdot d_{\max})$.
    fn build_coordination_tree(
        graph: &GvGraph<C, V, N>,
        competitive_gnodes: &BTreeSet<GNodeId>,
        root: GNodeId,
    ) -> Option<CoordNode<C>> {
        let info = graph.gnode_info(root)?;
        let children = graph.gnode_children(root)?;

        let has_left = children.left.is_some();
        let has_right = children.right.is_some();
        let is_competitive = competitive_gnodes.contains(&root);

        // Terminal node (no children).
        if !has_left && !has_right {
            return if is_competitive {
                Some(CoordNode::Terminal { gnode: root })
            } else {
                None // Prune: no competitive cell here.
            };
        }

        let left_tree = children
            .left
            .and_then(|l| Self::build_coordination_tree(graph, competitive_gnodes, l));
        let right_tree = children
            .right
            .and_then(|r| Self::build_coordination_tree(graph, competitive_gnodes, r));

        match (left_tree, right_tree) {
            (Some(left), Some(right)) => Some(CoordNode::Internal {
                gnode: root,
                depth: info.depth,
                start: info.start,
                end: info.end,
                is_competitive,
                left: Box::new(left),
                right: Box::new(right),
            }),
            (Some(child), None) | (None, Some(child)) => Some(CoordNode::SemiInternal {
                gnode: root,
                is_competitive,
                child: Box::new(child),
            }),
            (None, None) => {
                // Both subtrees pruned, but this node itself might be competitive.
                if is_competitive {
                    Some(CoordNode::Terminal { gnode: root })
                } else {
                    None
                }
            }
        }
    }

    /// Walk the coordination tree bottom-up, firing coordination at
    /// internal nodes where both subtrees contribute competitive cell
    /// scores (§ALGO S-9.4).
    ///
    /// Returns `(reports, cells_in_subtree)`.
    #[allow(clippy::type_complexity)]
    fn walk_coordination(
        &mut self,
        node: &CoordNode<C>,
        cell_scores: &BTreeMap<GNodeId, [f64; 4]>,
    ) -> (Vec<CoordinationReport<C>>, Vec<(GNodeId, [f64; 4])>) {
        match node {
            CoordNode::Terminal { gnode } => {
                if let Some(&scores) = cell_scores.get(gnode) {
                    (Vec::new(), vec![(*gnode, scores)])
                } else {
                    (Vec::new(), Vec::new())
                }
            }
            CoordNode::SemiInternal {
                gnode,
                is_competitive,
                child,
            } => {
                let (reports, mut cells) = self.walk_coordination(child, cell_scores);
                // If this node itself is competitive and has scores, add it.
                if *is_competitive && let Some(&scores) = cell_scores.get(gnode) {
                    cells.push((*gnode, scores));
                }
                (reports, cells)
            }
            CoordNode::Internal {
                gnode,
                depth,
                start,
                end,
                is_competitive,
                left,
                right,
            } => {
                let (left_reports, left_cells) = self.walk_coordination(left, cell_scores);
                let (right_reports, right_cells) = self.walk_coordination(right, cell_scores);

                let mut reports = left_reports;
                reports.extend(right_reports);

                let mut my_cells: Vec<(GNodeId, [f64; 4])> = Vec::with_capacity(left_cells.len() + right_cells.len() + 1);
                my_cells.extend_from_slice(&left_cells);
                my_cells.extend_from_slice(&right_cells);

                // If this node itself is competitive and has scores, add it.
                if *is_competitive && let Some(&scores) = cell_scores.get(gnode) {
                    my_cells.push((*gnode, scores));
                }

                // Fire coordination if both subtrees contribute and ≥ 2 cells total.
                if !left_cells.is_empty() && !right_cells.is_empty() && my_cells.len() >= 2 {
                    let report = self.fire_coordination(*gnode, *depth, *start, *end, &my_cells, false);
                    reports.push(report);
                }

                (reports, my_cells)
            }
        }
    }

    /// Fire a coordination context at a G-node: centre, observe, update
    /// running mean, and produce a `CoordinationReport`.
    #[allow(clippy::cast_possible_truncation)] // depth ≤ 128, always fits in u8
    fn fire_coordination(
        &mut self,
        gnode: GNodeId,
        depth: u32,
        start: C,
        end: C,
        my_cells: &[(GNodeId, [f64; 4])],
        is_noise: bool,
    ) -> CoordinationReport<C> {
        let lam = self.config.forgetting_factor;
        let alpha = 1.0 - lam;

        // §ALGO S-9.8: Collect baselines before borrowing self.coordination
        // to avoid simultaneous &mut borrows.
        let cell_baselines: Vec<(GNodeId, AxisBaselines)> = if self.coordination.contains_key(&gnode) {
            Vec::new()
        } else {
            my_cells
                .iter()
                .filter_map(|(cell_gnode, _)| self.cells.get(cell_gnode).map(|c| (*cell_gnode, c.tracker.axis_baselines())))
                .collect()
        };

        let ctx = self.coordination.entry(gnode).or_insert_with(|| CoordContext {
            tracker: SubspaceTracker::new(4, &self.config, self.config.cusum_coord_slow_decay),
            running_mean: [0.0; 4],
            warm: false,
        });

        // §ALGO S-9.8: Warm new contexts when not in noise phase.
        let coord_rounds = self.config.noise_schedule.rounds_for_depth(depth as usize);
        if !cell_baselines.is_empty() && !is_noise && coord_rounds > 0 {
            warm_coordination_context(
                ctx,
                &cell_baselines,
                coord_rounds as usize,
                &mut self.noise_rng,
                self.config.forgetting_factor,
                depth as u8,
            );
        }

        // Centre against running mean (§ALGO S-9.3).
        let centred: Vec<Vec<f64>> = my_cells
            .iter()
            .map(|(_, scores)| scores.iter().enumerate().map(|(j, &v)| v - ctx.running_mean[j]).collect())
            .collect();

        let slices: Vec<&[f64]> = centred.iter().map(Vec::as_slice).collect();
        let prefix_report = ctx.tracker.observe(&slices, depth as u8, is_noise);

        // Compute column means.
        let col_means = compute_col_means(my_cells);

        // Update running mean (§ALGO S-9.3).
        if ctx.warm {
            for (j, m) in ctx.running_mean.iter_mut().enumerate() {
                *m = lam.mul_add(*m, alpha * col_means[j]);
            }
        } else {
            ctx.running_mean = col_means;
            ctx.warm = true;
        }

        CoordinationReport {
            gnode_id: gnode,
            start,
            end,
            depth,
            cells_reporting: my_cells.len(),
            rank: prefix_report.rank,
            energy_ratio: prefix_report.energy_ratio,
            top_singular_value: prefix_report.top_singular_value,
            scores: prefix_report.scores,
            maturity: prefix_report.maturity,
            geometry: prefix_report.geometry,
            per_member: prefix_report.per_sample.map(|samples| {
                samples
                    .into_iter()
                    .zip(my_cells.iter())
                    .map(|(s, (cell_gnode, _))| {
                        let (cell_start, cell_end, cell_depth) = self
                            .cells
                            .get(cell_gnode)
                            .map_or_else(|| (C::zero(), C::zero(), 0), |c| (c.start, c.end, c.depth));
                        MemberScore {
                            cell_start,
                            cell_end,
                            cell_depth,
                            novelty: s.novelty,
                            displacement: s.displacement,
                            surprise: s.surprise,
                            coherence: s.coherence,
                            novelty_z: s.novelty_z,
                            displacement_z: s.displacement_z,
                            surprise_z: s.surprise_z,
                            coherence_z: s.coherence_z,
                        }
                    })
                    .collect()
            }),
        }
    }

    /// Collect the `GNodeId`s of all `Internal` nodes in a coordination tree.
    ///
    /// These are the nodes where coordination fires. Used for
    /// pruning stale contexts (§5.5).
    fn collect_internal_gnodes(node: &CoordNode<C>) -> BTreeSet<GNodeId> {
        let mut result = BTreeSet::new();
        Self::collect_internal_gnodes_inner(node, &mut result);
        result
    }

    fn collect_internal_gnodes_inner(node: &CoordNode<C>, result: &mut BTreeSet<GNodeId>) {
        match node {
            CoordNode::Terminal { .. } => {}
            CoordNode::SemiInternal { child, .. } => {
                Self::collect_internal_gnodes_inner(child, result);
            }
            CoordNode::Internal { gnode, left, right, .. } => {
                result.insert(*gnode);
                Self::collect_internal_gnodes_inner(left, result);
                Self::collect_internal_gnodes_inner(right, result);
            }
        }
    }

    /// Compute coordination health summary.
    fn coordination_health(&self) -> CoordinationHealth {
        let count = self.coordination.len();

        if count == 0 {
            return CoordinationHealth {
                active_contexts: 0,
                capacity: 0,
                rank_distribution: RankDistribution {
                    min: 0,
                    max: 0,
                    mean: 0.0,
                },
                maturity_distribution: MaturityDistribution {
                    max_noise_influence: 0.0,
                    min_noise_influence: 0.0,
                    mean_noise_influence: 0.0,
                    cold_trackers: 0,
                },
                dim: 4,
                geometry_distribution: GeometryDistribution {
                    novelty_saturated: 0,
                    novelty_saturable: 0,
                    coherence_inactive: 0,
                },
            };
        }

        let mut rank_min = usize::MAX;
        let mut rank_max = 0_usize;
        let mut rank_sum = 0_u64;

        let mut ni_min = f64::INFINITY;
        let mut ni_max = f64::NEG_INFINITY;
        let mut ni_sum = 0.0_f64;
        let mut cold = 0_usize;

        let mut geo_saturated = 0_usize;
        let mut geo_saturable = 0_usize;
        let mut geo_coh_inactive = 0_usize;

        for ctx in self.coordination.values() {
            let r = ctx.tracker.rank();
            rank_min = rank_min.min(r);
            rank_max = rank_max.max(r);
            rank_sum += r as u64;

            let m = ctx.tracker.maturity();
            ni_min = ni_min.min(m.noise_influence);
            ni_max = ni_max.max(m.noise_influence);
            ni_sum += m.noise_influence;
            if m.real_observations == 0 {
                cold += 1;
            }

            let g = ctx.tracker.scoring_geometry();
            if g.is_novelty_saturated() {
                geo_saturated += 1;
            }
            if g.is_novelty_saturable() {
                geo_saturable += 1;
            }
            if r < 2 {
                geo_coh_inactive += 1;
            }
        }

        #[allow(clippy::cast_precision_loss)]
        let n = count as f64;

        #[allow(clippy::cast_precision_loss)]
        let rank_mean = rank_sum as f64 / n;

        // Read capacity and dim from the first coordination tracker.
        let first = self.coordination.values().next().unwrap();
        let capacity = first.tracker.cap();
        let dim = first.tracker.dim();

        CoordinationHealth {
            active_contexts: count,
            capacity,
            rank_distribution: RankDistribution {
                min: rank_min,
                max: rank_max,
                mean: rank_mean,
            },
            maturity_distribution: MaturityDistribution {
                max_noise_influence: ni_max,
                min_noise_influence: ni_min,
                mean_noise_influence: ni_sum / n,
                cold_trackers: cold,
            },
            dim,
            geometry_distribution: GeometryDistribution {
                novelty_saturated: geo_saturated,
                novelty_saturable: geo_saturable,
                coherence_inactive: geo_coh_inactive,
            },
        }
    }

    /// Produce an empty `BatchReport` (for empty input slices).
    ///
    /// Takes `&mut self` so that structural-mutation counters are
    /// drained (and `prev_*` snapshots advanced) through this path
    /// too — e.g. if the caller interleaves `decay()` or
    /// `decay_subtree()` with empty `ingest(&[])` calls, the next
    /// non-empty report still sees the correct "since last report"
    /// delta rather than a double-count.
    fn empty_report(&mut self) -> BatchReport<C> {
        let health = self.health();
        let mut summary = self.analysis_set.summary();
        let warming_count = self.staging.lock().expect("staging mutex poisoned").total_count();
        summary.investment_set_size = self.cells.len() + warming_count;
        summary.degenerate_cells_skipped = self.degenerate_cells_skipped;

        let (splits, net_removals, terminal_count) = self.take_structural_mutation_counts();

        BatchReport {
            cell_reports: Vec::new(),
            ancestor_reports: Vec::new(),
            coordination_reports: Vec::new(),
            contour: ContourSnapshot {
                plateau_count: self.graph.plateaus().len(),
                cell_count: terminal_count as usize,
                total_importance: self.graph.total_sum().to_f64_approx(),
                splits_since_last_report: splits,
                net_removals_since_last_report: net_removals,
            },
            health,
            analysis_set_summary: summary,
            // No observations, so no oldest one to be any age at all.
            oldest_observation_age_micros: None,
        }
    }

    /// Compute `(splits, net_removals, terminal_count)` since the
    /// previous report and advance the `prev_*` snapshots.
    ///
    /// Derived exactly from `terminal_count()` / `node_count()`
    /// deltas via the identities (§ALGO S-14.10):
    ///
    /// - Each split:       `ΔN = +2`, `ΔT = +1`
    /// - Each eviction:    `ΔN = −1`, `ΔT = −1`
    /// - Each restoration: `ΔN = +1`, `ΔT = +1`
    ///
    /// ⟹ `splits = ΔN − ΔT` and
    ///   `net_removals = splits − ΔT = evictions − restorations`.
    ///
    /// Both outputs are non-negative by construction; debug builds
    /// assert this to catch identity violations (e.g. if a future
    /// graph mutation path were to break the accounting).
    /// Release builds saturate at `u32::MAX` on the astronomically
    /// unlikely overflow path.
    fn take_structural_mutation_counts(&mut self) -> (u32, u32, u32) {
        let terminal_count = self.graph.terminal_count();
        let node_count = self.graph.node_count();
        let delta_terminal = i64::from(terminal_count) - i64::from(self.prev_terminal_count);
        let delta_node = i64::from(node_count) - i64::from(self.prev_node_count);

        let raw_splits = delta_node - delta_terminal;
        debug_assert!(
            raw_splits >= 0,
            "split identity violated: ΔN ({delta_node}) < ΔT ({delta_terminal})",
        );
        let raw_net_removals = raw_splits - delta_terminal;
        debug_assert!(
            raw_net_removals >= 0,
            "net-removal identity violated: splits ({raw_splits}) < ΔT ({delta_terminal})",
        );

        let splits = u32::try_from(raw_splits.max(0)).unwrap_or(u32::MAX);
        let net_removals = u32::try_from(raw_net_removals.max(0)).unwrap_or(u32::MAX);

        self.prev_terminal_count = terminal_count;
        self.prev_node_count = node_count;

        (splits, net_removals, terminal_count)
    }
}

// ─── Automatic noise injection (§ALGO S-11) ───────────────────

/// Inject noise into a single cell tracker and reset its CUSUM.
///
/// Feeds `rounds` batches of `batch_size` random ±0.5 centred vectors
/// through the tracker with `is_noise = true`, then resets all four
/// CUSUM accumulators (§ALGO S-7.4, §ALGO S-11.1).
///
/// Generate a batch of random centred vectors (each entry ±0.5).
fn generate_noise_batch(dim: usize, batch_size: usize, rng: &mut SmallRng) -> Vec<Vec<f64>> {
    (0..batch_size)
        .map(|_| (0..dim).map(|_| if rng.random_bool(0.5) { 0.5 } else { -0.5 }).collect())
        .collect()
}

/// Compute column means of the 4D cell score vectors.
fn compute_col_means(cells: &[(GNodeId, [f64; 4])]) -> [f64; 4] {
    let mut col_means = [0.0_f64; 4];
    for (_, scores) in cells {
        for (j, &v) in scores.iter().enumerate() {
            col_means[j] += v;
        }
    }
    #[allow(clippy::cast_precision_loss)]
    let n_f = cells.len() as f64;
    for m in &mut col_means {
        *m /= n_f;
    }
    col_means
}

/// Returns the per-round mean score vectors (4D), needed for chained
/// coordination warming when the cell is competitive.
#[allow(clippy::cast_possible_truncation)] // depth ≤ 128
fn inject_noise_into_cell<C: Coordinate>(
    cell: &mut CellState<C>,
    rounds: usize,
    batch_size: usize,
    rng: &mut SmallRng,
) -> Vec<[f64; 4]> {
    let mut round_scores = Vec::with_capacity(rounds);

    for _ in 0..rounds {
        let noise = generate_noise_batch(cell.width, batch_size, rng);
        let slices: Vec<&[f64]> = noise.iter().map(Vec::as_slice).collect();
        let report = cell.tracker.observe(&slices, cell.depth as u8, true);

        round_scores.push([
            report.scores.novelty.mean,
            report.scores.displacement.mean,
            report.scores.surprise.mean,
            report.scores.coherence.mean,
        ]);
    }

    cell.tracker.seed_cusum_slow_from_baselines();
    cell.tracker.reset_cusum();
    cell.tracker.reset_clip_pressure();
    round_scores
}

// ─── Coordination warming (§ALGO S-9.8) ──────────────────────

/// Snapshot of per-axis baseline statistics for synthetic score
/// generation (§ALGO S-9.8).
pub struct AxisBaselines {
    pub novelty_mean: f64,
    pub novelty_var: f64,
    pub displacement_mean: f64,
    pub displacement_var: f64,
    pub surprise_mean: f64,
    pub surprise_var: f64,
    pub coherence_mean: f64,
    pub coherence_var: f64,
}

/// Sample a synthetic 4-axis score vector from axis baselines.
///
/// Uses Gamma(α, β) where α = mean²/variance, β = mean/variance.
/// Falls back to axis-specific defaults when baselines are cold
/// (§ALGO S-9.8).
fn sample_synthetic_score(baselines: &AxisBaselines, rng: &mut SmallRng) -> [f64; 4] {
    [
        gamma_sample(baselines.novelty_mean, baselines.novelty_var, 0.25, 0.01, rng),
        gamma_sample(baselines.displacement_mean, baselines.displacement_var, 0.1, 0.01, rng),
        gamma_sample(baselines.surprise_mean, baselines.surprise_var, 0.25, 0.01, rng),
        gamma_sample(baselines.coherence_mean, baselines.coherence_var, 0.1, 0.01, rng),
    ]
}

/// Sample from Gamma(α, β) with α = mean²/var, β = mean/var.
/// Falls back to `default_mean`/`default_var` when `mean` or `var`
/// are invalid (zero, negative, NaN).
fn gamma_sample(mean: f64, var: f64, default_mean: f64, default_var: f64, rng: &mut SmallRng) -> f64 {
    let (m, v) = if mean > 0.0 && var > 0.0 && mean.is_finite() && var.is_finite() {
        (mean, var)
    } else {
        (default_mean, default_var)
    };
    let alpha = m * m / v;
    let beta = m / v;
    let gamma = Gamma::new(alpha, 1.0 / beta).unwrap_or_else(|_| {
        let a = default_mean * default_mean / default_var;
        let b = default_mean / default_var;
        Gamma::new(a, 1.0 / b).expect("default Gamma params must be valid")
    });
    gamma.sample(rng)
}

/// Warm a newly activated coordination context with synthetic
/// score vectors sampled from cell baselines (§ALGO S-9.8).
///
/// Uses Gamma sampling to match cell baseline moments while
/// respecting axis non-negativity.
fn warm_coordination_context(
    ctx: &mut CoordContext,
    contributing_cells: &[(GNodeId, AxisBaselines)],
    rounds: usize,
    rng: &mut SmallRng,
    forgetting_factor: f64,
    depth: u8,
) {
    for _ in 0..rounds {
        let synth_with_gnodes: Vec<(GNodeId, [f64; 4])> = contributing_cells
            .iter()
            .map(|(gnode, baselines)| (*gnode, sample_synthetic_score(baselines, rng)))
            .collect();

        // Centre against running mean.
        let centred: Vec<Vec<f64>> = synth_with_gnodes
            .iter()
            .map(|(_, scores)| scores.iter().enumerate().map(|(j, &v)| v - ctx.running_mean[j]).collect())
            .collect();

        let slices: Vec<&[f64]> = centred.iter().map(Vec::as_slice).collect();
        ctx.tracker.observe(&slices, depth, true);

        // Update running mean.
        let col_means = compute_col_means(&synth_with_gnodes);
        let lam = forgetting_factor;
        let alpha = 1.0 - lam;
        if ctx.warm {
            for (j, m) in ctx.running_mean.iter_mut().enumerate() {
                *m = lam.mul_add(*m, alpha * col_means[j]);
            }
        } else {
            ctx.running_mean = col_means;
            ctx.warm = true;
        }
    }

    ctx.tracker.reset_cusum();
    ctx.tracker.reset_clip_pressure();
}

// ─── Coordination tree topology ─────────────────────────────

/// Lightweight mirror of the G-tree topology, pre-collected for
/// coordination traversal. Avoids borrowing `self.graph` during
/// `self.coordination` mutation.
enum CoordNode<C: Coordinate> {
    /// Terminal node: a competitive cell with no relevant children.
    Terminal { gnode: GNodeId },
    /// Internal node with only one relevant child (no coordination fires here).
    SemiInternal {
        gnode: GNodeId,
        is_competitive: bool,
        child: Box<Self>,
    },
    /// Internal node with both relevant children: coordination fires here.
    Internal {
        gnode: GNodeId,
        depth: u32,
        start: C,
        end: C,
        is_competitive: bool,
        left: Box<Self>,
        right: Box<Self>,
    },
}

// ─── Compile-time safety ────────────────────────────────────

/// Static assertion that `SpectralSentinel` is `Send + Sync`.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<SpectralSentinel<u128, u64, 128>>();
};
