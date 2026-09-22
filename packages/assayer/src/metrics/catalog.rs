// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Metric catalog: compile-time table of all Assayer metrics
//! (´dec:metrics:compile-time-catalogue´).
//!
//! The host's integration layer iterates [`METRICS_CATALOG`] to register
//! metrics at startup and to map [`crate::health::HealthSummary`] fields at scrape time.
//!
//! # Alert Threshold Guidance
//!
//! These are **guidance, not enforced rules** — the monitoring surface
//! reports and never acts (´inv:monitoring:report-only´). The Assayer does
//! not evaluate alert conditions internally. The host implements alerting
//! based on the exported metrics.
//!
//! | Alert                         | Metric condition                                                          | Severity |
//! |-------------------------------|---------------------------------------------------------------------------|----------|
//! | No Sentinels reporting        | all `assayer_sentinel_reporting` gauges == 0 (host-derived)               | Critical |
//! | AUC discrimination collapse   | `assayer_auc_recent < 0.55 AND assayer_auc_aggregate >= 0.65`             | Critical |
//! | Cascade terminus              | `assayer_cascade_terminus_active == 1`                                    | Critical |
//! | CP5/CP6 reverts occurring     | `rate(assayer_cp5_reverts_total) > 0` or `rate(assayer_cp6_reverts_total) > 0` | Critical |
//! | Low Sentinel coverage         | fraction of `assayer_sentinel_reporting` == 1 < 0.5 (host-derived)        | Warning  |
//! | Near-chance AUC               | `assayer_auc_aggregate < 0.55`                                            | Warning  |
//! | High condition number         | `max(assayer_kappa_estimate) > 1e6` across models                         | Warning  |
//! | Platt instability             | `assayer_platt_delta_cal > 0.2`                                           | Warning  |
//! | Calibration immature          | `assayer_convergence_stage < 3 AND assayer_labels_eligible_total > 500`   | Info     |
//! | Anchor-dominated steady state | `assayer_blend_fraction_anchor_dominated > 0.5 AND assayer_labels_eligible_total > 2000` | Warning |
//! | Signal cache thrashing        | `rate(assayer_signal_cache_evictions_total) / assessment rate > 0.5` (host-derived) | Warning  |
//! | Health event backpressure     | `rate(assayer_health_events_dropped_total[5m]) > 0`                       | Warning  |
//!
//! # Cross-References
//!
//! - (´dec:metrics:catalogue-membership´) — what belongs in this table, and
//!   the five things each entry declares
//! - (´cor:metrics:bounded-cardinality´) — what a table fixed at compile time
//!   rules out

// ═══════════════════════════════════════════════════════════════════════════════
// Types
// ═══════════════════════════════════════════════════════════════════════════════

/// A single metric definition in the catalog.
///
/// Describes the metric's name, type, help text, label dimensions,
/// and the health surface it is sourced from.  The host uses this to
/// register metrics at startup in whatever monitoring system it uses
/// (Prometheus, OTLP, `StatsD`, etc.).
#[derive(Debug, Clone)]
pub struct MetricDefinition {
    /// The metric name. Follows Prometheus naming conventions:
    /// `<namespace>_<subsystem>_<name>_<unit>`.
    ///
    /// All names start with `assayer_`.
    pub name: &'static str,

    /// The metric type (counter or gauge).
    pub metric_type: MetricType,

    /// Human-readable description. Suitable for Prometheus HELP text.
    pub help: &'static str,

    /// Label dimensions. Empty for scalar metrics.
    /// Non-empty for per-model, per-sentinel, per-channel, or
    /// per-dimension metrics.
    pub labels: &'static [&'static str],

    /// Which health surface produces this metric.
    ///
    /// Hosts can filter the catalog by surface to decide scrape cadence
    /// (cheap [`MetricSurface::Summary`] at 15 s; expensive
    /// [`MetricSurface::FullReport`] at 60 s; [`MetricSurface::Host`]
    /// entries are declared here for naming consistency only — no mapper
    /// emits them).
    pub surface: MetricSurface,
}

/// Metric type classification — one of the five things a catalogue entry
/// declares (´dec:metrics:catalogue-membership´).
///
/// # Encoding Conventions
///
/// - **Counter:** Monotonically increasing `u64` value. Reset only at
///   process restart. Mapped to Prometheus `counter`.
/// - **Gauge:** Point-in-time `f64` value. Can go up or down. Mapped
///   to Prometheus `gauge`.
///
/// Enum fields are encoded as integer gauges, by the encoding rules that
/// define the sample's form rather than choosing it
/// (´cav:metrics:encoding-absorbed´):
/// - `CompositeConvergenceStage` → 0–5
/// - `StandardisationPhase` → 0=`WaitingForInit`, 1=`Transitioning`, 2=`InService`
/// - `PlattConvergenceState` → 0=`Initial`, 1=`FirstFit`, 2=`Converging`, 3=`Converged`
///
/// Boolean fields → 0.0/1.0. `Option<f64>::None` → `f64::NAN`.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricType {
    /// Monotonically increasing counter.
    Counter,
    /// Point-in-time gauge.
    Gauge,
}

/// Which health surface the metric is sourced from — the two the queries
/// tier by cost, plus the host's own (´dec:health:tiered-queries´).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MetricSurface {
    /// From [`crate::health::HealthSummary`]. Cheap (~100 ns). Suitable for
    /// 15-second scrape intervals.
    Summary,
    /// From [`crate::health::SystemHealthReport`]. Expensive (~20 ms to source).
    /// Suitable for low-frequency export or on-demand dashboards.
    FullReport,
    /// Host-maintained. Documented here for naming consistency but
    /// sourced outside the Assayer (e.g., `assess()` latency histogram
    /// kept by the host around its call site).
    Host,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Catalog
// ═══════════════════════════════════════════════════════════════════════════════

/// The complete metric catalog.
///
/// Compile-time constant. One entry per metric. The host's integration
/// layer iterates this table to register metrics at startup and to
/// verify mapping completeness.
///
/// Metrics are grouped by surface:
/// - **Summary** scalars from [`crate::health::HealthSummary`] — cheap scrape path.
/// - **`FullReport`** per-entity and deep scalars from
///   [`crate::health::SystemHealthReport`] — low-frequency export.
/// - **Host** — names reserved for host-maintained metrics
///   (e.g., `assess()` latency). No mapper emits these.
///
/// What belongs here is decided rather than collected: an entry exactly where a
/// mapper emits it, plus the reserved host-maintained names no mapper emits,
/// each declaring its name, type, help, labels and surface
/// (´dec:metrics:catalogue-membership´). The pin below is derived from this
/// table's own text and is expected to move with it — which is what makes a
/// citation of it fail when the table changes (´conv:metrics:pin-churn´).
///
/// ´const:assayer:metric-catalogue´ (´alg:const:form´)
/// ´const:assayer:metric-catalogue-form-x86d95ced´
pub static METRICS_CATALOG: &[MetricDefinition] = &[
    // ─── Summary counters ────────────────────────────────────────────
    MetricDefinition {
        name: "assayer_assessments_total",
        metric_type: MetricType::Counter,
        help: "Total Core assessment requests processed",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_labels_total",
        metric_type: MetricType::Counter,
        help: "Total labels processed by the model owner",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_labels_eligible_total",
        metric_type: MetricType::Counter,
        help: "Labels eligible for model updates; convergence clock for early warm-up stages",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_degraded_assessments_total",
        metric_type: MetricType::Counter,
        help: "Assessments that suffered any degradation (NaN checkpoints or reported load conditions)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_signals_sanitised_total",
        metric_type: MetricType::Counter,
        help: "Host signal values replaced (CP1: NaN/Inf to 0.0)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_signal_shape_mismatches_total",
        metric_type: MetricType::Counter,
        help: "Host signal values whose shape did not match the declared schema (CP1)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_unknown_signals_total",
        metric_type: MetricType::Counter,
        help: "Host signal names skipped because they were not declared in the schema (CP1)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_sentinel_slots_zeroed_total",
        metric_type: MetricType::Counter,
        help: "Per-Sentinel extraction slots zeroed due to NaN (CP2)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_features_sanitised_total",
        metric_type: MetricType::Counter,
        help: "Post-standardisation features replaced (CP3)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_model_fallbacks_total",
        metric_type: MetricType::Counter,
        help: "Model evaluations that fell back to prior (CP4)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_batch_init_observations_skipped_total",
        metric_type: MetricType::Counter,
        help: "Batch-init observations skipped on a contended lock",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_zero_sentinel_assessments_total",
        metric_type: MetricType::Counter,
        help: "Assessments completed without a reporting Sentinel",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_cp5_reverts_total",
        metric_type: MetricType::Counter,
        help: "Working copy reverts from post-label NaN detection",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_cp6_reverts_total",
        metric_type: MetricType::Counter,
        help: "Pre-publish snapshot reverts from NaN detection",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_platt_refits_total",
        metric_type: MetricType::Counter,
        help: "Total Platt calibration refits completed",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_health_events_dropped_total",
        metric_type: MetricType::Counter,
        help: "Push health events lost due to channel capacity",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_drift_resets_total",
        metric_type: MetricType::Counter,
        help: "Drift-reset events emitted by automatic and calibration triggers",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    // ─── Summary gauges ──────────────────────────────────────────────
    MetricDefinition {
        name: "assayer_convergence_stage",
        metric_type: MetricType::Gauge,
        // TODO ´todo:code:once-composite-computation-is-aligned-this´: once composite computation is aligned, this gauge
        // may regress after lifecycle events (´dec:health:on-demand-composite´)
        // and early-stage boundaries should be driven by
        // `assayer_labels_eligible_total`.
        help: "System convergence stage (0=ColdStart .. 5=SteadyState)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_platt_state",
        metric_type: MetricType::Gauge,
        help: "Platt convergence state (0=Initial .. 3=Converged)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_platt_delta_cal",
        metric_type: MetricType::Gauge,
        help: "Most recent Platt calibration quality metric (log-kappa change)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_platt_kappa_sister",
        metric_type: MetricType::Gauge,
        help: "Sister-model Platt calibration kappa",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_platt_kappa_anchor",
        metric_type: MetricType::Gauge,
        help: "Anchor-model Platt calibration kappa",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_weight_mean",
        metric_type: MetricType::Gauge,
        help: "Mean blend weight (anchor contribution) over rolling window",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_weight_p10",
        metric_type: MetricType::Gauge,
        help: "10th percentile blend weight",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_weight_p50",
        metric_type: MetricType::Gauge,
        help: "Median blend weight",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_weight_p90",
        metric_type: MetricType::Gauge,
        help: "90th percentile blend weight",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_weight_p99",
        metric_type: MetricType::Gauge,
        help: "99th percentile blend weight",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_fraction_anchor_dominated",
        metric_type: MetricType::Gauge,
        help: "Fraction of assessments in anchor-dominated regime (w > 0.3)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_w_infinity_estimate",
        metric_type: MetricType::Gauge,
        help: "Estimated steady-state residual blend weight",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_steady_state",
        metric_type: MetricType::Gauge,
        help: "1.0 if blend window indicates steady state (w_infinity < 0.15); else 0.0",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_blend_window_size",
        metric_type: MetricType::Gauge,
        help: "Current blend observation window size (<= capacity during warm-up)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_auc_aggregate",
        metric_type: MetricType::Gauge,
        help: "Aggregate AUC from calibration buffer (NaN if insufficient data)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_auc_recent",
        metric_type: MetricType::Gauge,
        help: "Recent AUC (NaN if insufficient data)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_max_kappa_estimate",
        metric_type: MetricType::Gauge,
        help: "Maximum condition number estimate across all models",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_max_sync_error",
        metric_type: MetricType::Gauge,
        help: "Maximum synchronisation error across all models",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_cascade_terminus_active",
        metric_type: MetricType::Gauge,
        help: "1.0 if any model has experienced a cascade terminus; 0.0 otherwise",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_feature_stable_outcome_drift",
        metric_type: MetricType::Gauge,
        help: "Feature-stable outcome drift flag (0.0=false, 1.0=true)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_quantile_error_concentration",
        metric_type: MetricType::Gauge,
        help: "Quantile error concentration from last Platt refit (NaN if insufficient data)",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_positive_class_prior_global",
        metric_type: MetricType::Gauge,
        help: "Global positive-class prior used by the operational model",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    MetricDefinition {
        name: "assayer_positive_class_prior_eligible",
        metric_type: MetricType::Gauge,
        help: "Eligible positive-class prior used by the sister model",
        labels: &[],
        surface: MetricSurface::Summary,
    },
    // ─── Full-report per-entity metrics ──────────────────────────────
    MetricDefinition {
        name: "assayer_kappa_estimate",
        metric_type: MetricType::Gauge,
        help: "Lower bound on the condition number of precision matrix B, from its diagonal",
        labels: &["model"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_cholesky_recomputes_total",
        metric_type: MetricType::Counter,
        help: "Cholesky recomputations per model",
        labels: &["model"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_precision_floored_rebuilds_total",
        metric_type: MetricType::Counter,
        help: "Rebuilds at which the spectral floor repaired the precision matrix",
        labels: &["model"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_cholesky_cascade_terminus_total",
        metric_type: MetricType::Counter,
        help: "Cholesky cascade terminations (severe)",
        labels: &["model"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_reporting",
        metric_type: MetricType::Gauge,
        help: "1.0 if Sentinel has a cached report; 0.0 otherwise",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_report_age_seconds",
        metric_type: MetricType::Gauge,
        help: "Seconds since last report received (NaN if never)",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_ledger_entries",
        metric_type: MetricType::Gauge,
        help: "Number of Ledger entries for this Sentinel",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_informativeness",
        metric_type: MetricType::Gauge,
        help: "Mean absolute operational weight over the Sentinel's slot; NaN when unmeasured",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_association",
        metric_type: MetricType::Gauge,
        help: "Slot-outcome association as a multiple of the null floor; NaN when unmeasured",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_contribution",
        metric_type: MetricType::Gauge,
        help: "Loss cost of removing the slot as a fraction of the score's loss; NaN when unmeasured",
        labels: &["sentinel"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_cell_count",
        metric_type: MetricType::Gauge,
        help: "Number of competitive cells in this identity dimension",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_change_rate",
        metric_type: MetricType::Gauge,
        help: "Competitive set change rate EWMA",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_coverage_fraction",
        metric_type: MetricType::Gauge,
        help: "Share of assessed traffic that fell in a competitive cell of this dimension; NaN before any traffic",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_observations_dropped_total",
        metric_type: MetricType::Counter,
        help: "Identity observation queue overflows",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_informativeness",
        metric_type: MetricType::Gauge,
        help: "Mean absolute operational weight over the dimension's block; NaN when unmeasured",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_cell_weight_mass",
        metric_type: MetricType::Gauge,
        help: "Mean absolute operational weight over the dimension's competitive indicators; NaN when unmeasured",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_association",
        metric_type: MetricType::Gauge,
        help: "Block-outcome association as a multiple of the null floor; NaN when unmeasured",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_identity_contribution",
        metric_type: MetricType::Gauge,
        help: "Loss cost of removing the block as a fraction of the score's loss; NaN when unmeasured",
        labels: &["dimension"],
        surface: MetricSurface::FullReport,
    },
    // ─── Companion challenge gauges (host-emitted) ───────────────────
    // Reserved for host emission: these two have no package mapper by
    // design, because Companion state is the host's and reaches no Core
    // structure (´dec:contracts:companion-boundary´). The values behind
    // the names are the tracker's own health report
    // (´tab:companion:health´).
    MetricDefinition {
        name: "assayer_challenge_q_hat",
        metric_type: MetricType::Gauge,
        help: "Companion challenge effectiveness point estimate",
        labels: &["channel"],
        surface: MetricSurface::Host,
    },
    MetricDefinition {
        name: "assayer_challenge_sufficient_evidence",
        metric_type: MetricType::Gauge,
        help: "Companion challenge tracker sufficient-evidence flag",
        labels: &["channel"],
        surface: MetricSurface::Host,
    },
    // ─── Full-report deep scalars ────────────────────────────────────
    MetricDefinition {
        name: "assayer_concordance_calibrations_total",
        metric_type: MetricType::Counter,
        help: "Concordance threshold recalibrations completed",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_schur_corrections_skipped_total",
        metric_type: MetricType::Counter,
        help: "Schur corrections skipped (condition guard or factor failure)",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_schur_worst_correction_loss_fraction",
        metric_type: MetricType::Gauge,
        help: "Largest fraction of a Schur correction lost by any marginalisation",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_signal_cache_evictions_total",
        metric_type: MetricType::Counter,
        help: "Signal cache LRU evictions",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_signal_cache_hit_rate",
        metric_type: MetricType::Gauge,
        help: "Signal cache hits as a fraction of all cache lookups",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_signal_cache_utilisation",
        metric_type: MetricType::Gauge,
        help: "Signal cache occupancy as a fraction of capacity",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_sentinel_coverage",
        metric_type: MetricType::Gauge,
        help: "Fraction of registered Sentinels with a cached report",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_standardisation_snapshot_version",
        metric_type: MetricType::Gauge,
        help: "Published model snapshot version joining the standardisation phase and count",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_standardisation_phase",
        metric_type: MetricType::Gauge,
        help: "Cold standardisation ramp phase (0=WaitingForInit, 1=Transitioning, 2=InService)",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_standardisation_observations",
        metric_type: MetricType::Gauge,
        help: "Accepted cold-ramp observations, from zero through N_init",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_pending_buffer_utilisation",
        metric_type: MetricType::Gauge,
        help: "Pending buffer utilisation (0.0-1.0)",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    MetricDefinition {
        name: "assayer_ledger_cells_tracked",
        metric_type: MetricType::Gauge,
        help: "Total ledger cells across all Sentinels",
        labels: &[],
        surface: MetricSurface::FullReport,
    },
    // ─── Host-maintained (reserved names; no mapper emits these) ─────
    MetricDefinition {
        name: "assayer_reckoning_duration_seconds",
        metric_type: MetricType::Gauge,
        help: "Host-side histogram of assess() call latency (host-maintained; see dec:metrics:catalogue-membership)",
        labels: &[],
        surface: MetricSurface::Host,
    },
];
