// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Metric sample type and mapper functions — the neutral intermediate form
//! values leave the package in (´dec:metrics:neutral-sample´).
//!
//! Provides the [`MetricSample`] intermediate representation and two
//! mapper functions that convert health data structures into metric
//! samples for host-side rendering.
//!
//! # Cross-References
//!
//! - (´cav:metrics:encoding-absorbed´) — where the rules for encoding an
//!   absent value into a sample went
//! - (´dec:health:tiered-queries´) — the two health surfaces these mappers
//!   read, and what each costs
//! - (´dec:metrics:passive-export´) — why the mappers stop at samples and
//!   render none of them

#![allow(clippy::cast_precision_loss)] // Justified: metric values are f64; u64→f64 precision loss is acceptable for monitoring

use super::catalog::MetricType;
use crate::health::{HealthSummary, PlattConvergenceState, SystemHealthReport};
use crate::types::ModelId;

// ═══════════════════════════════════════════════════════════════════════════════
// MetricSample
// ═══════════════════════════════════════════════════════════════════════════════

/// A single metric observation.
///
/// Intermediate representation between [`HealthSummary`] and a
/// wire format renderer. The host converts this to Prometheus text
/// format, OTLP, `StatsD`, or a custom format.
#[derive(Debug, Clone)]
pub struct MetricSample {
    /// Metric name from the catalog.
    pub name: &'static str,
    /// Metric type (counter or gauge).
    pub metric_type: MetricType,
    /// Label key-value pairs. Empty for scalar metrics.
    pub labels: Vec<(&'static str, String)>,
    /// The metric value.
    pub value: f64,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Encoding Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Encode an `Option<f64>` as a metric value, by the absent-value rule that
/// defines the sample's form (´cav:metrics:encoding-absorbed´).
///
/// `None` → `f64::NAN`. Prometheus stores NaN transparently;
/// Grafana renders it as a gap in the graph; alerting rules using
/// `absent()` fire correctly.
#[must_use]
pub const fn option_to_metric(val: Option<f64>) -> f64 {
    match val {
        Some(v) => v,
        None => f64::NAN,
    }
}

/// Convert a [`ModelId`] to a label value string, drawn from a declared set
/// so the series count stays bounded (´cor:metrics:bounded-cardinality´).
#[must_use]
pub fn model_label(id: &ModelId) -> String {
    match id {
        ModelId::Operational => "operational".to_string(),
        ModelId::Sister => "sister".to_string(),
        ModelId::Anchor => "anchor".to_string(),
        ModelId::OutcomeAxis(axis_id) => format!("axis_{}", axis_id.0),
    }
}

/// Encode a [`PlattConvergenceState`] as an integer gauge.
const fn platt_state_to_f64(state: PlattConvergenceState) -> f64 {
    match state {
        PlattConvergenceState::Initial => 0.0,
        PlattConvergenceState::FirstFit => 1.0,
        PlattConvergenceState::Converging => 2.0,
        PlattConvergenceState::Converged => 3.0,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// HealthSummary Mapper (scrape-path, ~500 ns)
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a [`HealthSummary`] to a vector of metric samples.
///
/// Pure function. No side effects. ~500 ns at ~30 scalar fields.
///
/// This function produces samples for all metrics that can be derived
/// from [`HealthSummary`] alone — the "scrape-path" metrics. Per-model,
/// per-sentinel, and per-channel metrics that require data from
/// [`SystemHealthReport`] are NOT included; use
/// [`full_report_to_samples()`] for those.
#[must_use]
pub fn health_summary_to_samples(summary: &HealthSummary) -> Vec<MetricSample> {
    let mut samples = Vec::with_capacity(40);
    push_summary_counters(&mut samples, summary);
    push_summary_gauges(&mut samples, summary);
    samples
}

/// Emit counter samples from a [`HealthSummary`].
fn push_summary_counters(samples: &mut Vec<MetricSample>, summary: &HealthSummary) {
    samples.push(MetricSample {
        name: "assayer_assessments_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.total_assessments as f64,
    });
    samples.push(MetricSample {
        name: "assayer_labels_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.total_labels as f64,
    });
    samples.push(MetricSample {
        name: "assayer_labels_eligible_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.eligible_labels as f64,
    });
    push_degradation_counters(samples, summary);
    samples.push(MetricSample {
        name: "assayer_cp5_reverts_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.cp5_reverts as f64,
    });
    samples.push(MetricSample {
        name: "assayer_cp6_reverts_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.cp6_reverts as f64,
    });
    samples.push(MetricSample {
        name: "assayer_platt_refits_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: f64::from(summary.platt_refits_completed),
    });
    samples.push(MetricSample {
        name: "assayer_health_events_dropped_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.health_events_dropped as f64,
    });
    samples.push(MetricSample {
        name: "assayer_drift_resets_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.drift_resets as f64,
    });
}

/// Emit assessment-degradation counter samples from a [`HealthSummary`].
fn push_degradation_counters(samples: &mut Vec<MetricSample>, summary: &HealthSummary) {
    samples.push(MetricSample {
        name: "assayer_degraded_assessments_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.total_degraded as f64,
    });
    samples.push(MetricSample {
        name: "assayer_signals_sanitised_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.signals_sanitised as f64,
    });
    samples.push(MetricSample {
        name: "assayer_signal_shape_mismatches_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.signals_shape_mismatched as f64,
    });
    samples.push(MetricSample {
        name: "assayer_unknown_signals_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.signals_unknown as f64,
    });
    samples.push(MetricSample {
        name: "assayer_sentinel_slots_zeroed_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.sentinel_slots_zeroed as f64,
    });
    samples.push(MetricSample {
        name: "assayer_features_sanitised_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.features_sanitised as f64,
    });
    samples.push(MetricSample {
        name: "assayer_model_fallbacks_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.model_fallbacks as f64,
    });
    samples.push(MetricSample {
        name: "assayer_batch_init_observations_skipped_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.batch_init_observations_skipped as f64,
    });
    samples.push(MetricSample {
        name: "assayer_zero_sentinel_assessments_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: summary.degradation.zero_sentinel_assessments as f64,
    });
}

/// Emit gauge samples from a [`HealthSummary`].
#[allow(clippy::too_many_lines)] // Justified: sequential metric mapping of HealthSummary fields
fn push_summary_gauges(samples: &mut Vec<MetricSample>, summary: &HealthSummary) {
    samples.push(MetricSample {
        name: "assayer_convergence_stage",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: f64::from(summary.convergence_stage as u8),
    });
    samples.push(MetricSample {
        name: "assayer_platt_state",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: platt_state_to_f64(summary.platt_state),
    });
    samples.push(MetricSample {
        name: "assayer_platt_delta_cal",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.last_delta_cal,
    });
    samples.push(MetricSample {
        name: "assayer_blend_weight_mean",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.mean,
    });
    samples.push(MetricSample {
        name: "assayer_blend_weight_p10",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.p10,
    });
    samples.push(MetricSample {
        name: "assayer_blend_weight_p50",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.p50,
    });
    samples.push(MetricSample {
        name: "assayer_blend_weight_p90",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.p90,
    });
    samples.push(MetricSample {
        name: "assayer_blend_weight_p99",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.p99,
    });
    samples.push(MetricSample {
        name: "assayer_blend_fraction_anchor_dominated",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.fraction_anchor_dominated,
    });
    samples.push(MetricSample {
        name: "assayer_w_infinity_estimate",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.w_infinity_estimate,
    });
    samples.push(MetricSample {
        name: "assayer_blend_steady_state",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: if summary.blend_statistics.steady_state { 1.0 } else { 0.0 },
    });
    samples.push(MetricSample {
        name: "assayer_blend_window_size",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.blend_statistics.window_size as f64,
    });
    samples.push(MetricSample {
        name: "assayer_feature_stable_outcome_drift",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: if summary.feature_stable_outcome_drift { 1.0 } else { 0.0 },
    });
    samples.push(MetricSample {
        name: "assayer_quantile_error_concentration",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: option_to_metric(summary.quantile_error_concentration),
    });
    samples.push(MetricSample {
        name: "assayer_auc_aggregate",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: option_to_metric(summary.auc_aggregate),
    });
    samples.push(MetricSample {
        name: "assayer_auc_recent",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: option_to_metric(summary.auc_recent),
    });
    samples.push(MetricSample {
        name: "assayer_max_kappa_estimate",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.max_diagonal_ratio,
    });
    samples.push(MetricSample {
        name: "assayer_max_sync_error",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.max_sync_error,
    });
    samples.push(MetricSample {
        name: "assayer_cascade_terminus_active",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: if summary.any_cascade_terminus { 1.0 } else { 0.0 },
    });
    samples.push(MetricSample {
        name: "assayer_platt_kappa_sister",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.platt_kappa_sister,
    });
    samples.push(MetricSample {
        name: "assayer_platt_kappa_anchor",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.platt_kappa_anchor,
    });
    samples.push(MetricSample {
        name: "assayer_positive_class_prior_global",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.p_positive_global,
    });
    samples.push(MetricSample {
        name: "assayer_positive_class_prior_eligible",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: summary.p_positive_eligible,
    });
}

// ═══════════════════════════════════════════════════════════════════════════════
// SystemHealthReport Mapper (full, ~20 ms source)
// ═══════════════════════════════════════════════════════════════════════════════

/// Convert a [`SystemHealthReport`] to metric samples for per-model,
/// per-sentinel, and per-dimension breakdowns.
///
/// Intended for low-frequency export (e.g., every 60 seconds) or
/// on-demand dashboards, NOT for 15-second Prometheus scrapes.
/// The [`SystemHealthReport`] itself costs ~20 ms to construct.
#[must_use]
pub fn full_report_to_samples(report: &SystemHealthReport) -> Vec<MetricSample> {
    let mut samples = Vec::with_capacity(200);
    push_report_precision(&mut samples, report);
    push_report_sentinels(&mut samples, report);
    push_report_identity(&mut samples, report);
    push_report_scalars(&mut samples, report);
    samples
}

/// Emit per-model precision samples from a [`SystemHealthReport`].
fn push_report_precision(samples: &mut Vec<MetricSample>, report: &SystemHealthReport) {
    for (model_id, precision) in &report.precision {
        let label = model_label(model_id);
        samples.push(MetricSample {
            name: "assayer_kappa_estimate",
            metric_type: MetricType::Gauge,
            labels: vec![("model", label.clone())],
            value: precision.diagonal_ratio,
        });
        samples.push(MetricSample {
            name: "assayer_precision_floored_rebuilds_total",
            metric_type: MetricType::Counter,
            labels: vec![("model", label.clone())],
            value: precision.floored_rebuilds as f64,
        });
        samples.push(MetricSample {
            name: "assayer_cholesky_recomputes_total",
            metric_type: MetricType::Counter,
            labels: vec![("model", label.clone())],
            value: precision.cholesky_recomputes as f64,
        });
        samples.push(MetricSample {
            name: "assayer_cholesky_cascade_terminus_total",
            metric_type: MetricType::Counter,
            labels: vec![("model", label)],
            value: precision.cascade_terminus_count as f64,
        });
    }
}

/// Emit per-Sentinel samples from a [`SystemHealthReport`].
fn push_report_sentinels(samples: &mut Vec<MetricSample>, report: &SystemHealthReport) {
    for (sentinel_id, sentinel) in &report.sentinels {
        let label = sentinel_id.0.to_string();
        samples.push(MetricSample {
            name: "assayer_sentinel_reporting",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label.clone())],
            value: if sentinel.has_report { 1.0 } else { 0.0 },
        });
        samples.push(MetricSample {
            name: "assayer_sentinel_report_age_seconds",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label.clone())],
            value: option_to_metric(sentinel.report_age_seconds),
        });
        samples.push(MetricSample {
            name: "assayer_sentinel_ledger_entries",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label.clone())],
            value: sentinel.ledger_entry_count as f64,
        });
        samples.push(MetricSample {
            name: "assayer_sentinel_informativeness",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label.clone())],
            value: option_to_metric(sentinel.informativeness),
        });
        samples.push(MetricSample {
            name: "assayer_sentinel_association",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label.clone())],
            value: option_to_metric(sentinel.association),
        });
        samples.push(MetricSample {
            name: "assayer_sentinel_contribution",
            metric_type: MetricType::Gauge,
            labels: vec![("sentinel", label)],
            value: option_to_metric(sentinel.contribution),
        });
    }
}

/// Emit per-dimension identity samples from a [`SystemHealthReport`].
fn push_report_identity(samples: &mut Vec<MetricSample>, report: &SystemHealthReport) {
    for (dim_id, dim) in &report.identity_dimensions {
        let label = dim_id.0.to_string();
        samples.push(MetricSample {
            name: "assayer_identity_cell_count",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: dim.convergence.current_cell_count as f64,
        });
        samples.push(MetricSample {
            name: "assayer_identity_change_rate",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: dim.convergence.change_rate_ewma,
        });
        samples.push(MetricSample {
            name: "assayer_identity_coverage_fraction",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: option_to_metric(dim.competitive_coverage_fraction),
        });
        samples.push(MetricSample {
            name: "assayer_identity_observations_dropped_total",
            metric_type: MetricType::Counter,
            labels: vec![("dimension", label.clone())],
            value: dim.observations_dropped as f64,
        });
        samples.push(MetricSample {
            name: "assayer_identity_informativeness",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: option_to_metric(dim.informativeness),
        });
        samples.push(MetricSample {
            name: "assayer_identity_cell_weight_mass",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: option_to_metric(dim.cell_weight_mass),
        });
        samples.push(MetricSample {
            name: "assayer_identity_association",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label.clone())],
            value: option_to_metric(dim.association),
        });
        samples.push(MetricSample {
            name: "assayer_identity_contribution",
            metric_type: MetricType::Gauge,
            labels: vec![("dimension", label)],
            value: option_to_metric(dim.contribution),
        });
    }
}

/// Emit scalar (non-per-entity) samples from a [`SystemHealthReport`].
///
/// Metrics that are also emitted from [`HealthSummary`] (notably
/// `assayer_cascade_terminus_active` and `assayer_health_events_dropped_total`)
/// are NOT repeated here — every name has exactly one emission site so hosts
/// calling both mappers never double-register series.
fn push_report_scalars(samples: &mut Vec<MetricSample>, report: &SystemHealthReport) {
    // Concordance calibrations.
    samples.push(MetricSample {
        name: "assayer_concordance_calibrations_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: report.concordance.calibrations_completed as f64,
    });

    // Standardisation snapshot version, phase, and the count that makes it
    // readable.
    //
    // The gauge distinguishes all three phases rather than collapsing the
    // transition into "not yet in service": a scraper that could not tell a
    // ramp part way along from one that had not started would be reading the
    // very quantity the transition was made visible to show. The count is
    // published beside it, so maturity is a division a dashboard can do. The
    // version is their join key when a scraper needs to compare this reading
    // with another surface
    // (´tab:monitoring:standardisation-transition´).
    samples.push(MetricSample {
        name: "assayer_standardisation_snapshot_version",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.standardisation.snapshot_version as f64,
    });
    samples.push(MetricSample {
        name: "assayer_standardisation_phase",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: match report.standardisation.phase {
            crate::feature::standardisation::StandardisationPhase::WaitingForInit => 0.0,
            crate::feature::standardisation::StandardisationPhase::Transitioning => 1.0,
            crate::feature::standardisation::StandardisationPhase::InService => 2.0,
        },
    });
    samples.push(MetricSample {
        name: "assayer_standardisation_observations",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.standardisation.accepted_observations as f64,
    });

    // Pending buffer utilisation.
    samples.push(MetricSample {
        name: "assayer_pending_buffer_utilisation",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: if report.buffer.capacity > 0 {
            report.buffer.utilisation as f64 / report.buffer.capacity as f64
        } else {
            0.0
        },
    });

    // Ledger cell count (aggregate across Sentinels).
    samples.push(MetricSample {
        name: "assayer_ledger_cells_tracked",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.ledger.total_cells_tracked as f64,
    });

    // Schur corrections skipped.
    samples.push(MetricSample {
        name: "assayer_schur_corrections_skipped_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: report.schur_corrections_skipped as f64,
    });
    samples.push(MetricSample {
        name: "assayer_schur_worst_correction_loss_fraction",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.marginalisation.worst_correction_loss_fraction,
    });

    // Signal cache counters and occupancy.
    samples.push(MetricSample {
        name: "assayer_signal_cache_evictions_total",
        metric_type: MetricType::Counter,
        labels: vec![],
        value: report.signal_cache.evictions as f64,
    });
    samples.push(MetricSample {
        name: "assayer_signal_cache_hit_rate",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.signal_cache.hit_rate(),
    });
    samples.push(MetricSample {
        name: "assayer_signal_cache_utilisation",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.signal_cache.utilisation(),
    });
    samples.push(MetricSample {
        name: "assayer_sentinel_coverage",
        metric_type: MetricType::Gauge,
        labels: vec![],
        value: report.sentinel_coverage,
    });
}
