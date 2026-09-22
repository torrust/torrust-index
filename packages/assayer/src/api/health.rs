// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Health query methods on `Assayer`, tiered by cost
//! (´dec:health:tiered-queries´).
//!
//! Three methods:
//! - [`Assayer::health_summary()`] — ~100 ns, reads two `ArcSwap`s + atomics
//! - [`Assayer::full_health_report()`] — ~20 ms, reads everything
//! - [`Assayer::drain_health_events()`] — ~30 ns/event
//!
//! # Cross-References
//!
//! - Queries are tiered by cost (´dec:health:tiered-queries´)
//! - What the queries return is reported and never enforced
//!   (´dec:health:reports-never-gates´)

use std::collections::HashMap;
use std::sync::atomic::Ordering;

use indexmap::IndexMap;

use crate::Assayer;
use crate::feature::dimension_map::IndexRange;
use crate::feature::permutation::SLOT_AXIS_PAIR_OFFSET;
use crate::health::{
    AssessmentDegradationSummary, AxisHealth, BufferHealth, CalibrationHealth, ConvergenceHealth, CrossLayerHealth, DriftHealth,
    HealthEvent, HealthSummary, IdentityDimensionHealth, LabelIntegrityHealth, LabelQueueHealth, LedgerHealth,
    ObservabilityHealth, PrecisionHealthDetail, SentinelHealth, SentinelLedgerHealth, SentinelStructuralHealth,
    StandardisationHealth, SystemHealthReport,
};
use crate::types::{DimensionId, ModelId, OutcomeAxisId, SentinelId};

/// The whole-entry outcome-memory quantities that open the Ledger block.
///
/// The adverse rate, the compressed valence and the raw valence, in that order,
/// standing between the extraction's four fixed groups and the per-axis pairs
/// (´tab:extraction:ledger-features´). Counting back from the pair offset is
/// what keeps the adverse rate's position derived from the layout the map
/// publishes rather than transcribed as a second figure beside it.
///
/// ´const:assayer:outcome-memory-base-quantities´ (´alg:const:count´)
/// ´const:assayer:outcome-memory-base-quantities-count-3´
const OUTCOME_MEMORY_BASE_QUANTITIES: usize = 3;

/// The half-deviation multiplier of the materiality criterion.
///
/// One half of one standard deviation, the standardised departure the theorem
/// fixes as the boundary of materiality (´thm:ledger:materiality´) and the
/// coefficient the value-realisation definition writes it with
/// (´def:monitoring:ledger-value´).
///
/// ´const:assayer:materiality-standardised-departure´ (´alg:const:scalar´)
/// ´const:assayer:materiality-standardised-departure-scalar-0p5´
const MATERIALITY_STANDARDISED_DEPARTURE: f64 = 0.5;

/// One cell's materiality reading, and the traffic it carries.
///
/// The three states are exclusive by construction: a cell is realising its
/// value, or it is held short of realising it by thin evidence, or it is
/// neither. That exclusivity is what lets the two reported shares be read
/// against each other and keeps their sum inside the whole
/// (´def:monitoring:ledger-value´).
#[derive(Clone, Copy, Debug, Default)]
struct LedgerMaterialityScan {
    /// Assessed traffic across every scanned cell — the denominator of both
    /// reported shares (´def:monitoring:ledger-value´).
    traffic: u64,
    /// Traffic in cells whose remembered adverse rate clears the boundary.
    realised_traffic: u64,
    /// Traffic in cells held short of the boundary by thin evidence.
    attenuation_limited_traffic: u64,
    /// Cells below the eligible-label floor (´def:monitoring:immature-cells´).
    immature_cells: usize,
}

impl LedgerMaterialityScan {
    /// Adds another scan into this fleet-wide accumulator.
    const fn add(&mut self, other: Self) {
        self.traffic = self.traffic.saturating_add(other.traffic);
        self.realised_traffic = self.realised_traffic.saturating_add(other.realised_traffic);
        self.attenuation_limited_traffic = self
            .attenuation_limited_traffic
            .saturating_add(other.attenuation_limited_traffic);
        self.immature_cells = self.immature_cells.saturating_add(other.immature_cells);
    }

    /// Returns the realised traffic share, or zero for an untrafficked Ledger.
    fn value_realised(self) -> f64 {
        fraction(self.realised_traffic, self.traffic)
    }

    /// Returns the attenuation-limited traffic share, or zero when empty.
    fn attenuation_limited_fraction(self) -> f64 {
        fraction(self.attenuation_limited_traffic, self.traffic)
    }
}

/// Returns `numerator / denominator`, with an empty population reading zero.
#[allow(clippy::cast_precision_loss)] // Justified: traffic counts stay far below f64's exact integer range.
fn fraction(numerator: u64, denominator: u64) -> f64 {
    if denominator == 0 {
        0.0
    } else {
        numerator as f64 / denominator as f64
    }
}

/// The mean absolute published operational weight over one block of the vector.
///
/// One functional, read at whichever block the caller names: a Sentinel's own
/// slot gives the per-Sentinel reading (´def:monitoring:encoding-effectiveness´)
/// and an identity dimension's own block gives the per-dimension one
/// (´def:monitoring:dimension-informativeness´). The two callers share this
/// body rather than each folding the weights for itself, because what makes
/// the readings comparable is that they are one recipe over two disjoint
/// position sets.
///
/// `None` where the published model does not carry the block, and `None` again
/// where the block names no positions: an unmeasured entity stays
/// distinguishable from an uninformative one, and an empty block would
/// otherwise divide by zero.
#[allow(clippy::cast_precision_loss)] // Justified: a block's width is a small count, far below f64's exact integer range.
fn mean_absolute_block_weight(mu: &[f64], block: &IndexRange) -> Option<f64> {
    let weights = mu.get(block.to_range())?;
    if weights.is_empty() {
        return None;
    }
    let sum: f64 = weights.iter().map(|weight| weight.abs()).sum();
    Some(sum / weights.len() as f64)
}

/// The same functional at a set of positions that need not be contiguous.
///
/// The block form above is the case where the layout gives an entity one span.
/// An identity dimension's competitive indicators are the case where it does
/// not: the layout assigns one position per competitive cell, and a
/// dimension's cells are addressed through a per-cell map rather than through
/// a range (´def:dimension:competitive-range´). Both callers fold one recipe,
/// so what the readings share is the recipe and what differs is the population
/// of positions.
///
/// `None` where the named set is empty, and `None` again where the published
/// model does not carry one of its positions — the same two absences the block
/// form gets from its slice lookup, so an entity the layout has not placed
/// stays distinguishable from one carrying no weight.
#[allow(clippy::cast_precision_loss)] // Justified: a position count is a small count, far below f64's exact integer range.
fn mean_absolute_position_weight(mu: &[f64], positions: impl IntoIterator<Item = usize>) -> Option<f64> {
    let mut sum = 0.0_f64;
    let mut counted = 0_usize;
    for position in positions {
        sum += mu.get(position)?.abs();
        counted += 1;
    }
    if counted == 0 {
        return None;
    }
    Some(sum / counted as f64)
}

/// The rates and thresholds one materiality scan is taken against.
#[derive(Clone, Copy, Debug)]
struct MaterialityCriteria {
    /// Hourly Ledger decay `γ_t,ledger` (´def:ledger:time-decay´).
    gamma_t_ledger: f64,
    /// Label-indexed Ledger averaging rate `λ_L` (´tab:ledger:entry-state´).
    lambda_l: f64,
    /// Eligible-label floor below which a cell is immature
    /// (´def:monitoring:immature-cells´).
    materiality_threshold: u64,
    /// The standardised-departure boundary a rate must clear to be material.
    ///
    /// The theorem's own criterion: a departure of more than half a standard
    /// deviation from the standardisation mean (´thm:ledger:materiality´),
    /// which the value-realisation definition writes as
    /// `b̄ > μ̄_b + 0.5·√v̄_b` (´def:monitoring:ledger-value´).
    boundary: f64,
}

impl MaterialityCriteria {
    /// Builds the criteria from the configured rates and this feature's own
    /// standardisation moments.
    fn new(gamma_t_ledger: f64, lambda_l: f64, materiality_threshold: u64, mean: f64, variance: f64) -> Self {
        Self {
            gamma_t_ledger,
            lambda_l,
            materiality_threshold,
            boundary: MATERIALITY_STANDARDISED_DEPARTURE.mul_add(variance.max(0.0).sqrt(), mean),
        }
    }
}

/// Scans one Ledger against its bad-rate standardisation boundary.
///
/// The predicate is the theorem's two conditions, taken separately because the
/// theorem says neither suffices alone (´thm:ledger:materiality´).
///
/// A cell is **realising** when its remembered adverse rate — decayed to now,
/// which is the value any reader of the Ledger sees — stands above the
/// boundary. That is the definition's own condition
/// (´def:monitoring:ledger-value´).
///
/// A cell is **attenuation-limited** when it is not realising, its true adverse
/// rate does clear the boundary, and the attenuation its own eligible arrival
/// rate implies leaves the attenuated rate at or below it. The first condition
/// of the theorem is read off the raw undecayed pair and the second off the
/// arrival window, so the two are answered by separate evidence rather than by
/// one average standing in for both. A cell whose true rate is ordinary is not
/// attenuation-limited however sparse its labels, and a cell whose arrivals are
/// dense enough to preserve its excess is not attenuation-limited however high
/// its true rate — which is the theorem's independence made mechanical.
fn scan_ledger_materiality(
    ledger: &crate::ledger::SentinelLedger,
    criteria: MaterialityCriteria,
    now: &crate::types::PersistentTimestamp,
) -> LedgerMaterialityScan {
    let mut scan = LedgerMaterialityScan::default();

    for entry in ledger.entries().values() {
        let traffic = entry.total_assessments;
        scan.traffic = scan.traffic.saturating_add(traffic);
        scan.immature_cells += usize::from(entry.recent_eligible_count < criteria.materiality_threshold);

        let realised = entry.read_decayed(criteria.gamma_t_ledger, now).bad_rate > criteria.boundary;
        if realised {
            scan.realised_traffic = scan.realised_traffic.saturating_add(traffic);
            continue;
        }

        // Both halves of the theorem, or the cell is not attenuation-limited.
        let Some(true_rate) = entry.raw_adverse_rate() else {
            continue;
        };
        if true_rate <= criteria.boundary {
            continue;
        }
        let Some(attenuation) = entry.steady_state_attenuation(criteria.gamma_t_ledger, now, criteria.lambda_l) else {
            continue;
        };
        if attenuation * true_rate <= criteria.boundary {
            scan.attenuation_limited_traffic = scan.attenuation_limited_traffic.saturating_add(traffic);
        }
    }

    scan
}

impl Assayer {
    /// Returns a lightweight health summary (~100 ns).
    ///
    /// Reads two `ArcSwap`s and atomic counters. No locks acquired.
    ///
    /// # Cross-References
    ///
    /// - Health query tier 1 (´dec:health:tiered-queries´)
    #[must_use]
    pub fn health_summary(&self) -> HealthSummary {
        let health = self.shared.health.load();
        let snapshot = self.shared.published.load();
        let blend = self.blend_stats.current();
        let degradation = self.degradation_counters.snapshot();
        let total_assessments = self.total_assessments.load(Ordering::Relaxed);
        let health_events_dropped = self.health_events_dropped.load(Ordering::Relaxed);
        let drift_resets = self.drift_reset_counter.value();

        // Derive per-model precision aggregates from health ArcSwap.
        let max_diagonal_ratio = health
            .precision_health
            .values()
            .map(|h| h.diagonal_ratio)
            .fold(0.0_f64, f64::max);
        // A model with no rebuild behind it carries no reading and contributes
        // nothing to the maximum, which is the identity for a maximum rather
        // than a claim that it is undegraded (´dec:health:tiered-queries´).
        let max_floor_share = health
            .precision_health
            .values()
            .filter_map(|h| h.floor_share)
            .fold(0.0_f64, f64::max);
        let max_sync_error = health
            .precision_health
            .values()
            .map(|h| h.last_sync_error)
            .fold(0.0_f64, f64::max);
        // The two components of that reading, each maximised over the models
        // in its own right. The maxima are taken separately and not derived
        // from one another, because the model leaning hardest on its prior
        // and the model carrying the most rounding need not be the same model
        // (´def:monitoring:synchronisation-error´).
        let max_prior_induced_sync_error = health
            .precision_health
            .values()
            .map(|h| h.last_prior_induced_sync_error)
            .fold(0.0_f64, f64::max);
        let max_sync_error_residual = health
            .precision_health
            .values()
            .map(|h| h.last_sync_error_residual)
            .fold(0.0_f64, f64::max);
        let any_cascade_terminus = health.precision_health.values().any(|h| h.cascade_terminus_events > 0);
        // Read from the owner's own swap rather than from the published health
        // summary: the summary is republished on a label, and the path that
        // takes labels is the one that stopped (´dec:surface:async-label´).
        let label_path_stopped = self.shared.label_path_stop.load().is_some();

        HealthSummary {
            convergence_stage: health.convergence_stage,
            platt_state: health.platt_state,
            last_delta_cal: health.last_delta_cal,
            platt_refits_completed: health.platt_tracker.refits_completed,
            total_labels: health.total_labels,
            eligible_labels: health.eligible_labels,
            total_assessments,
            degradation,
            blend_statistics: (*blend).clone(),
            feature_stable_outcome_drift: health.feature_stable_outcome_drift,
            quantile_error_concentration: health.quantile_error_concentration,
            cp5_reverts: health.cp5_reverts,
            cp6_reverts: health.cp6_reverts,
            auc_aggregate: health.discrimination.as_ref().and_then(|d| d.auc_aggregate),
            auc_recent: health.discrimination.as_ref().and_then(|d| d.auc_recent),
            max_diagonal_ratio,
            max_floor_share,
            max_sync_error,
            max_prior_induced_sync_error,
            max_sync_error_residual,
            any_cascade_terminus,
            label_path_stopped,
            platt_kappa_sister: snapshot.kappa_sister,
            platt_kappa_anchor: snapshot.kappa_anchor,
            p_positive_global: snapshot.p_positive_global,
            p_positive_eligible: snapshot.p_positive_eligible,
            health_events_dropped,
            drift_resets,
        }
    }

    /// Returns a comprehensive system health report (~20 ms).
    ///
    /// Reads two `ArcSwap`s, iterates `DashMap`, acquires `RwLock`s for
    /// identity dimensions and ledger entry counts.
    ///
    /// # Panics
    ///
    /// Panics if a ledger `RwLock` is poisoned.
    ///
    /// # Cross-References
    ///
    /// - Health query tier 2 (´dec:health:tiered-queries´)
    #[must_use]
    #[allow(clippy::too_many_lines)] // Justified: sequential section assembly of a composite report
    #[allow(clippy::cast_precision_loss)]
    pub fn full_health_report(&self) -> SystemHealthReport {
        let health = self.shared.health.load();
        let snapshot = self.shared.published.load();
        let blend = self.blend_stats.current();
        let degradation_snap = self.degradation_counters.snapshot();
        let total_assessments = self.total_assessments.load(Ordering::Relaxed);
        let now = self.clock.now_monotonic();
        let persistent_now = self.clock.now();

        // Convergence
        let identity_stages: HashMap<DimensionId, _> = health
            .identity_trackers
            .iter()
            .map(|(&dim_id, tracker)| (dim_id, tracker.convergence_stage(&persistent_now)))
            .collect();
        let convergence = ConvergenceHealth {
            composite_stage: health.convergence_stage,
            platt_state: health.platt_state,
            identity_stages,
        };

        // Drift per model
        let drift: HashMap<ModelId, DriftHealth> = health
            .drift_state
            .iter()
            .map(|(model_id, ds)| {
                (
                    model_id.clone(),
                    DriftHealth {
                        s_plus: ds.s_plus,
                        s_minus: ds.s_minus,
                        mean_abs_residual: ds.mean_abs_residual,
                        sign_ewma: ds.residual_sign_ewma,
                        steps_since_reset: ds.steps_since_reset,
                    },
                )
            })
            .collect();

        // Precision per model
        let mut cascade_terminus_active = false;
        let precision: HashMap<ModelId, PrecisionHealthDetail> = health
            .precision_health
            .iter()
            .map(|(model_id, ph)| {
                let detail = PrecisionHealthDetail {
                    kappa: ph.kappa,
                    diagonal_ratio: ph.diagonal_ratio,
                    lambda_min: ph.lambda_min,
                    spectral_floor: ph.spectral_floor,
                    floor_share: ph.floor_share,
                    alarms: ph.alarms,
                    measurements: ph.measurements,
                    last_measurement_labels: ph.last_measurement_labels,
                    floored_rebuilds: ph.floored_rebuilds,
                    cholesky_recomputes: ph.total_recomputes,
                    cascade_terminus_count: ph.cascade_terminus_events,
                    last_sync_error: ph.last_sync_error,
                    last_prior_induced_sync_error: ph.last_prior_induced_sync_error,
                    last_sync_error_residual: ph.last_sync_error_residual,
                    last_sync_error_resolution: ph.last_sync_error_resolution,
                    last_measurement_at_resolution: ph.last_measurement_at_resolution,
                    // The readings the last rebuild took of itself, kept
                    // apart from the readings above because those are retaken
                    // at every visit and these are not. They reach this tier
                    // and no other: the compact tier's aggregates are maxima
                    // across models, and an after-drift is read against the
                    // before-drift and threshold of its own model
                    // (´dec:health:tiered-queries´),
                    // (´dec:posterior:measured-adoption´).
                    last_sync_error_after: ph.last_sync_error_after,
                    last_rebuild_verdict: ph.last_rebuild_verdict,
                    last_rebuild_adopted: ph.last_rebuild_adopted,
                    n_recompute_effective: ph.n_recompute_effective,
                    sync_error_shortenings: ph.sync_error_shortenings,
                    dimensions_at_floor: ph.dimensions_at_floor,
                };
                if detail.cascade_terminus_count > 0 {
                    cascade_terminus_active = true;
                }
                (model_id.clone(), detail)
            })
            .collect();

        // Calibration
        let platt = &health.platt_tracker;
        let anchor_records = snapshot.calibration_buffer.anchor_regime_records;
        let labels_since_last_anchor_fit = health.total_labels.saturating_sub(platt.last_anchor_refit_label_index);
        let calibration = CalibrationHealth {
            kappa_sister: snapshot.kappa_sister,
            kappa_anchor: snapshot.kappa_anchor,
            delta_cal: health.last_delta_cal,
            refits_completed: platt.refits_completed,
            labels_since_refit: platt.labels_since_refit,
            buffer_total: snapshot.calibration_buffer.total_entries,
            sister_regime_records: snapshot.calibration_buffer.sister_regime_records,
            anchor_regime_records: anchor_records,
            // Frozen when the weighted sample count (´constr:platt:buffer´)
            // sits below the minimum sample floor of thirty
            // (´req:platt:minimum-samples´).
            anchor_regime_frozen: platt.refits_completed > 0 && anchor_records < f64::from(self.config.platt.n_cal_min),
            labels_since_last_anchor_fit,
            convergence_state: health.platt_state,
        };

        // Per-Sentinel and Ledger health. Structural, maturity and materiality
        // fields are query-time scans; report deltas are lifetime counters
        // because prior acknowledgements cannot be reconstructed from current
        // state.
        let mut ledger_per_sentinel = IndexMap::new();
        let mut total_cells_tracked: usize = 0;
        let mut fleet_materiality = LedgerMaterialityScan::default();
        let mut adequate_cells: usize = 0;
        let mut mature_measurement_traffic: usize = 0;
        let mut total_measurement_traffic: usize = 0;
        let mut max_report_stage_lower_bound_seconds: Option<f64> = None;
        let sentinels: IndexMap<SentinelId, SentinelHealth> = self
            .sentinel_slots
            .iter()
            .map(|entry| {
                let sentinel_id = *entry.key();
                let report_guard = entry.value().report_index.load();
                let has_report = report_guard.has_report();
                let report_age_seconds = report_guard.received_at().map(|t| now.duration_since(t).as_secs_f64());
                // The producing Sentinel's own measure of how long its oldest
                // observation had waited when it emitted the report — a lower
                // bound on the first latency stage, on the far side of the
                // arrival `report_age_seconds` is measured from
                // (´def:monitoring:feedback-latency´).
                let report_stage_lower_bound_seconds = report_guard
                    .oldest_observation_age_micros()
                    .map(|micros| std::time::Duration::from_micros(micros).as_secs_f64());
                max_report_stage_lower_bound_seconds =
                    match (max_report_stage_lower_bound_seconds, report_stage_lower_bound_seconds) {
                        (Some(held), Some(candidate)) => Some(held.max(candidate)),
                        (held, candidate) => held.or(candidate),
                    };
                for cell in report_guard.cells().values() {
                    total_measurement_traffic = total_measurement_traffic.saturating_add(cell.sample_count);
                    if cell.noise_influence < self.config.monitoring.maturity_threshold {
                        mature_measurement_traffic = mature_measurement_traffic.saturating_add(cell.sample_count);
                    }
                }
                let structural = has_report.then(|| {
                    let level = report_guard.level_data();
                    SentinelStructuralHealth {
                        competitive_cell_count: level.competitive_cell_count,
                        full_set_size: level.full_set_size,
                        depth_range: level.depth_range,
                        plateau_count: level.plateau_count,
                        total_importance: level.total_importance,
                        contour_cell_count: level.contour_cell_count,
                        splits_since_last_report: level.splits_since_last_report,
                        net_removals_since_last_report: level.net_removals_since_last_report,
                        importance_range: level.importance_range,
                        v_depth_range: level.v_depth_range,
                        degenerate_cells_skipped: level.degenerate_cells_skipped,
                    }
                });
                let (cells_created, cells_deleted) = entry.value().ledger_cell_delta_totals();
                let (ledger_entry_count, sentinel_ledger_health) = self.outcome_ledger.get_arc(sentinel_id).map_or_else(
                    || (0, SentinelLedgerHealth::default()),
                    |arc| {
                        let guard = arc.read().expect("ledger lock poisoned");
                        let root = guard.root();
                        let mut depth_histogram = std::collections::BTreeMap::new();
                        for key in guard.entries().keys() {
                            *depth_histogram.entry(key.depth).or_insert(0) += 1;
                        }
                        // Each Sentinel's bad-rate feature is a distinct
                        // standardisation position, so the moments the
                        // materiality boundary is taken against resolve
                        // through this Sentinel's own published slot
                        // (´def:monitoring:ledger-value´).
                        let bad_rate_position = snapshot
                            .dimension_map
                            .sentinel_slots
                            .get(&sentinel_id)
                            .map(|slot| slot.start + SLOT_AXIS_PAIR_OFFSET - OUTCOME_MEMORY_BASE_QUANTITIES);
                        let mean = bad_rate_position
                            .and_then(|position| snapshot.feature_means.get(position))
                            .copied()
                            .unwrap_or(0.0);
                        let variance = bad_rate_position
                            .and_then(|position| snapshot.feature_variances.get(position))
                            .copied()
                            .unwrap_or(0.0);
                        let materiality = scan_ledger_materiality(
                            &guard,
                            MaterialityCriteria::new(
                                self.config.temporal.gamma_t_ledger,
                                self.config.ledger.lambda_l,
                                self.config.monitoring.ledger_materiality_threshold,
                                mean,
                                variance,
                            ),
                            &persistent_now,
                        );
                        adequate_cells = adequate_cells.saturating_add(
                            guard
                                .entries()
                                .values()
                                .filter(|entry| {
                                    entry.recent_eligible_count >= self.config.monitoring.resolution_adequacy_threshold
                                })
                                .count(),
                        );
                        let slh = SentinelLedgerHealth {
                            entry_count: guard.entry_count(),
                            max_depth: guard.max_depth(),
                            depth_histogram,
                            cells_created,
                            cells_deleted,
                            immature_cell_count: materiality.immature_cells,
                            value_realised: materiality.value_realised(),
                            attenuation_limited_fraction: materiality.attenuation_limited_fraction(),
                            root_adverse_rate: root.ewma_bad_rate,
                            root_compressed_valence: root.compressed_valence_ewma,
                            root_raw_valence: root.raw_valence_ewma,
                        };
                        fleet_materiality.add(materiality);
                        (guard.entry_count(), slh)
                    },
                );
                total_cells_tracked += ledger_entry_count;
                ledger_per_sentinel.insert(sentinel_id, sentinel_ledger_health);
                // Mean absolute operational weight over the Sentinel's own
                // slot (´def:monitoring:encoding-effectiveness´); `None`
                // when the published map does not yet carry the slot.
                let informativeness = snapshot
                    .dimension_map
                    .sentinel_slots
                    .get(&sentinel_id)
                    .and_then(|slot| mean_absolute_block_weight(&snapshot.operational.mu, slot));
                // The two legibility readings, reduced on the model owner's
                // thread from evidence accumulated at label time
                // (´def:monitoring:slot-association´),
                // (´def:monitoring:slot-contribution´). Absent for a Sentinel
                // whose slot has not yet carried a window of labels, which is
                // the absence discipline the weight-mass reading keeps for a
                // slot the layout does not carry.
                let legibility = health.sentinel_legibility.get(&sentinel_id).copied().unwrap_or_default();
                (
                    sentinel_id,
                    SentinelHealth {
                        has_report,
                        structural,
                        alarm_outcome: health.alarm_outcome.get(&sentinel_id).copied().unwrap_or_default(),
                        ledger_entry_count,
                        report_age_seconds,
                        informativeness,
                        association: legibility.association,
                        contribution: legibility.contribution,
                        report_stage_lower_bound_seconds,
                    },
                )
            })
            .collect();
        let sentinel_coverage = if sentinels.is_empty() {
            0.0
        } else {
            sentinels.values().filter(|sentinel| sentinel.has_report).count() as f64 / sentinels.len() as f64
        };
        let ledger = LedgerHealth {
            per_sentinel: ledger_per_sentinel,
            total_cells_tracked,
            total_immature_cells: fleet_materiality.immature_cells,
            value_realised: fleet_materiality.value_realised(),
            attenuation_limited_fraction: fleet_materiality.attenuation_limited_fraction(),
        };
        // End-to-end feedback latency (´def:monitoring:feedback-latency´). The
        // three local stages are smoothed on the model owner's thread, one
        // observation per published label, and ride the published summary; the
        // fleet-wide worst report stage comes from the scan above, because it
        // is a statement about the evidence in hand rather than about the
        // labels being published.
        let stages = health.feedback_latency;
        let cross_layer = CrossLayerHealth {
            stages,
            total_seconds_lower_bound: stages.total_seconds(),
            max_report_stage_lower_bound_seconds,
        };

        let observability = ObservabilityHealth {
            maturity_coverage: (total_measurement_traffic > 0)
                .then(|| mature_measurement_traffic as f64 / total_measurement_traffic as f64),
            resolution_utilisation: (total_cells_tracked > 0).then(|| adequate_cells as f64 / total_cells_tracked as f64),
        };

        // Per-axis
        let axes: IndexMap<OutcomeAxisId, AxisHealth> = snapshot
            .outcome_models
            .iter()
            .map(|(&axis_id, axis_state)| {
                (
                    axis_id,
                    AxisHealth {
                        name: axis_state.name.clone(),
                        kappa_a: axis_state.kappa_a,
                        gamma_a: axis_state.gamma_a,
                    },
                )
            })
            .collect();

        // Per-identity-dimension. The encoding reading is the same fold the
        // per-Sentinel one is, restricted to the dimension's own fixed-width
        // block rather than to a Sentinel's slot
        // (´def:monitoring:dimension-informativeness´). The block's extent
        // comes from the published layout, so a rebuild that moved the block
        // moves the reading with it.
        let dimension_informativeness = |dim_id: DimensionId| {
            snapshot
                .dimension_map
                .id_dim_ranges
                .get(&dim_id)
                .and_then(|block| mean_absolute_block_weight(&snapshot.operational.mu, block))
        };
        // The dimension's cell weights: one recipe again, at the positions the
        // layout gives the dimension's competitive indicators rather than at
        // its fixed-width block (´def:dimension:competitive-range´). The two
        // populations are disjoint, so this is a second reading beside the
        // block one and not a refinement of it — a dimension can carry weight
        // on its block and none on its cells, which is the case the host duty
        // reads them separately to catch (´req:keyspace:host-duties´).
        let dimension_cell_weight_mass = |dim_id: DimensionId| {
            snapshot
                .dimension_map
                .competitive_indices
                .get(&dim_id)
                .and_then(|cells| mean_absolute_position_weight(&snapshot.operational.mu, cells.values().copied()))
        };
        // The dimension's own pair of legibility readings, the siblings of the
        // per-Sentinel pair over a disjoint block
        // (´def:monitoring:slot-association´),
        // (´def:monitoring:slot-contribution´).
        let dimension_legibility = |dim_id: DimensionId| health.dimension_legibility.get(&dim_id).copied().unwrap_or_default();
        let mut identity_dimensions: IndexMap<DimensionId, IdentityDimensionHealth> = self
            .identity_dimensions
            .read()
            .expect("identity_dimensions lock poisoned")
            .iter()
            .map(|(&dim_id, infra)| {
                let convergence = health
                    .identity_trackers
                    .get(&dim_id)
                    .map_or_else(Default::default, |tracker| tracker.health(&persistent_now));
                let mut competitive_cell_depth_distribution = std::collections::BTreeMap::new();
                for cell in infra.load_competitive_set().cell_ids() {
                    *competitive_cell_depth_distribution.entry(cell.depth).or_insert(0) += 1;
                }
                // Coverage is folded from the very map this report carries,
                // not from a second read of the tally: the two rows are then
                // arithmetically consistent because there is one observation
                // of one counter behind both (´tab:monitoring:dimension-health´).
                let active_indicator_count_distribution = infra.active_indicator_count_distribution();
                let competitive_coverage_fraction =
                    crate::identity::competitive_coverage_fraction(&active_indicator_count_distribution);
                (
                    dim_id,
                    IdentityDimensionHealth {
                        name: infra.dimension.name.clone(),
                        description: infra.dimension.description.clone(),
                        coordinate_semantics: infra.dimension.coordinate_semantics.clone(),
                        convergence,
                        competitive_cell_depth_distribution,
                        active_indicator_count_distribution,
                        competitive_coverage_fraction,
                        observations_dropped: infra.observations_dropped(),
                        informativeness: dimension_informativeness(dim_id),
                        cell_weight_mass: dimension_cell_weight_mass(dim_id),
                        association: dimension_legibility(dim_id).association,
                        contribution: dimension_legibility(dim_id).contribution,
                    },
                )
            })
            .collect();
        for (&dim_id, tracker) in &health.identity_trackers {
            identity_dimensions.entry(dim_id).or_insert_with(|| IdentityDimensionHealth {
                name: String::new(),
                description: String::new(),
                coordinate_semantics: String::new(),
                convergence: tracker.health(&persistent_now),
                competitive_cell_depth_distribution: std::collections::BTreeMap::new(),
                active_indicator_count_distribution: std::collections::BTreeMap::new(),
                // A dimension known only to the tracker has no infrastructure
                // to have tallied traffic against, so coverage is absent
                // rather than zero: nothing has been assessed here.
                competitive_coverage_fraction: None,
                observations_dropped: 0,
                informativeness: dimension_informativeness(dim_id),
                cell_weight_mass: dimension_cell_weight_mass(dim_id),
                association: dimension_legibility(dim_id).association,
                contribution: dimension_legibility(dim_id).contribution,
            });
        }

        // Buffer
        let (pending_evictions, pending_insertions) = self.pending_buffer.eviction_totals();
        let eviction_rate = if pending_insertions > 0 {
            pending_evictions as f64 / pending_insertions as f64
        } else {
            0.0
        };
        let buffer = BufferHealth {
            capacity: self.pending_buffer.capacity(),
            utilisation: self.pending_buffer.len(),
            insertions: pending_insertions,
            evictions: pending_evictions,
            eviction_rate,
            oldest_pending_age_seconds: self.pending_buffer.oldest_pending_age(now).map(|age| age.as_secs_f64()),
        };
        let label_queue = LabelQueueHealth {
            capacity: self.label_tx.capacity().unwrap_or(0),
            depth: self.label_tx.len(),
            drops: self.label_queue_drops.load(Ordering::Relaxed),
        };

        // Standardisation.
        //
        // The phase and count come from the published snapshot rather than
        // from the last label-time summary, because the full report's pair is
        // the current one (´tab:monitoring:standardisation-transition´) and the
        // summary is only refreshed when a label is processed. Reading them
        // from the summary would leave a deployment that has not yet labelled
        // anything — which is every deployment for the whole of its cold start,
        // exactly when this pair is worth reading — reporting `WaitingForInit`
        // and a count of zero while the ramp ran to its horizon underneath it.
        //
        // Features at the floor stays with the summary: it is a per-position
        // count over the working copy, and the snapshot does not carry the
        // classes needed to exclude the unstandardised bias.
        let standardisation = StandardisationHealth {
            snapshot_version: snapshot.version,
            phase: snapshot.standardisation_phase,
            accepted_observations: snapshot.standardisation_observations,
            features_at_floor: health.features_at_floor,
        };

        // Label integrity
        let total_labels = health.total_labels;
        let eligible_labels = health.eligible_labels;
        let eligibility_rate = if total_labels > 0 {
            eligible_labels as f64 / total_labels as f64
        } else {
            0.0
        };
        let label_integrity = LabelIntegrityHealth {
            total_labels,
            eligible_labels,
            eligibility_rate,
            per_action: health.labels_by_action.clone(),
            valences_sanitised: self.label_valences_sanitised.load(Ordering::Relaxed),
            outcomes_dropped: self.label_outcomes_dropped.load(Ordering::Relaxed),
        };

        // Assessment degradation
        let degraded_fraction = if total_assessments > 0 {
            degradation_snap.total_degraded as f64 / total_assessments as f64
        } else {
            0.0
        };
        let assessment_degradation = AssessmentDegradationSummary {
            total_assessments,
            degraded_fraction,
            counters: degradation_snap,
        };

        // Concordance
        let mut concordance = self.concordance.health();
        concordance.per_entity = health.per_entity_concordance.health_with_thresholds(
            health.discrimination.as_ref().and_then(|metrics| metrics.auc_aggregate),
            self.config.monitoring.min_concordance_labels,
            self.config.monitoring.concordance_deficit_threshold,
        );
        concordance.per_entity_capacity = health.per_entity_concordance.capacity();
        concordance.per_entity_evictions = health.per_entity_concordance.evictions();

        // Global counters
        let health_events_dropped = self.health_events_dropped.load(Ordering::Relaxed);
        let signal_cache = self.signal_cache.health();

        SystemHealthReport {
            convergence,
            drift,
            precision,
            calibration,
            sentinels,
            sentinel_coverage,
            axes,
            identity_dimensions,
            buffer,
            label_queue,
            standardisation,
            label_integrity,
            assessment_degradation,
            discrimination: health.discrimination.clone(),
            blend_statistics: (*blend).clone(),
            concordance,
            ledger,
            observability,
            cross_layer,
            importance_ceiling: health.importance_ceiling.clone(),
            marginalisation: health.marginalisation.clone(),
            cascade_terminus_active,
            label_path_stop: self.shared.label_path_stop.load_full().map(|stop| (*stop).clone()),
            health_events_dropped,
            schur_corrections_skipped: health.marginalisation.corrections_skipped,
            signal_cache,
        }
    }

    /// Requests a host-initiated drift accumulator reset
    /// (´tab:monitoring:drift-resets´).
    ///
    /// The model owner resets every model's accumulators and step
    /// counters; the smoothed diagnostics survive, as they survive the
    /// automatic trigger. Asynchronous: the reset is applied when the
    /// owner thread drains its command channel, before the next label.
    ///
    /// # Errors
    ///
    /// - [`crate::error::LifecycleError::CommandChannelFull`] — the
    ///   command channel is at capacity; retry later
    /// - [`crate::error::LifecycleError::ModelOwnerShutdown`] — the
    ///   model owner has shut down
    pub fn reset_drift_accumulators(&self) -> Result<(), crate::error::LifecycleError> {
        use crossbeam_channel::TrySendError;
        self.command_tx()
            .try_send(crate::owner::commands::ModelOwnerCommand::ResetDriftAccumulators)
            .map_err(|e| match e {
                TrySendError::Full(_) => crate::error::LifecycleError::CommandChannelFull,
                TrySendError::Disconnected(_) => crate::error::LifecycleError::ModelOwnerShutdown,
            })
    }

    /// Drains all pending health events (~30 ns/event).
    ///
    /// Returns events in emission order. Empty after drain.
    ///
    /// # Cross-References
    ///
    /// - Health query tier 3 (´dec:health:tiered-queries´)
    #[must_use]
    pub fn drain_health_events(&self) -> Vec<HealthEvent> {
        let mut events = Vec::new();
        while let Ok(event) = self.health_event_rx.try_recv() {
            events.push(event);
        }
        events
    }
}
