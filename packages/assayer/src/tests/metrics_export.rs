// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`catalog_size_stays_inside_its_band`] | metrics | The exported surface is a deliberately sized catalogue, held within a band rather than growing to whatever the mappers happen to emit. A metrics surface is an interface an operator builds dashboards and alerts against; letting it drift by dozens either way would mean it was never designed, and the band is what makes an unnoticed sprawl a failure rather than a diff nobody reads. |
//! | [`all_metric_names_start_with_assayer`] | metrics | Every declared metric name is namespaced to the crate. Metrics from many subsystems land in one flat namespace in the collector a host runs, so an unprefixed name would sooner or later collide with something else's, and the two series would silently merge into one meaningless one. |
//! | [`all_counter_names_end_with_total`] | metrics | A metric's name announces its kind: everything declared as a counter ends in the conventional suffix, and nothing else has to. Counters and gauges are queried quite differently — one is differentiated over time, the other read as it stands — so the naming convention is what lets an operator write the right query from the name alone, without consulting the catalogue. |
//! | [`all_metric_names_are_unique`] | metrics | No two catalogue entries share a name. A name is the only handle an operator has on a measurement, so a duplicate would leave two different quantities indistinguishable at the point of query — and worse, would make the catalogue's own claim to be a complete description of the surface false. |
//! | [`catalog_counters_and_gauges_counts`] | metrics | Every catalogue entry is a counter or a gauge and nothing else — the two kinds account for the whole catalogue exactly — and each kind is substantially represented. The surface therefore describes both what has accumulated over the system's life and what is true of it right now; a surface of only counters could not report a current state, and one of only gauges could not report a rate. |
//! | [`option_none_maps_to_nan`] | metrics | A measurement the system does not have is exported as NaN, never as zero. The difference matters enormously for a quantity like discrimination: zero would read as catastrophically bad performance and page someone, whereas NaN reads as "not yet measured" and is skipped by aggregation rather than dragging an average down. |
//! | [`option_some_maps_to_value`] | metrics | A measurement the system does have is exported as exactly that value. Nothing is rounded, rescaled, or clipped on the way out, so the number on an operator's dashboard is the number the model was working with. |
//! | [`boolean_maps_to_zero_or_one`] | metrics | A condition that is either true or false is exported as one or zero, and the exported value tracks the condition as it changes. Metrics collectors carry numbers and not flags, so an alert on a boolean condition has to be expressible as a numeric comparison — which it can be, precisely because the two states occupy two definite values. |
//! | [`convergence_stage_maps_to_integer`] | metrics | A lifecycle stage exports as its position in the sequence, with the settled end of the composite progression carrying the highest number. Because the encoding preserves the order, an operator can alert on the system having fallen below a stage — a comparison that would be meaningless against arbitrary codes. |
//! | [`platt_state_maps_to_integer`] | metrics | cites (´claim:metrics:an-ordered-lifecycle-stage-exports-as-its-ordinal-so-progress-is-numerically-comparable´) |
//! | [`model_label_encoding`] | metrics | Each of the three named models labels itself with its own name, while an outcome axis labels itself by its identifier. There are exactly three named models but arbitrarily many axes, so the axis label has to be generated from the identity rather than enumerated — and generating it keeps every axis's series distinct without the catalogue having to grow an entry per axis. |
//! | [`health_summary_to_samples_has_every_field`] | metrics | Every field of the health summary reaches the export under its own name and carrying its own value — the assessment and label tallies, each separate degradation counter, the whole blend-weight distribution, and the optional discrimination figures alike. A field held internally but never exported is invisible to the only people who could act on it, so the mapping has to be exhaustive rather than a selection of what seemed interesting. |
//! | [`health_summary_sample_count_matches`] | metrics | The summary export emits a pinned number of samples, so the surface cannot change size unnoticed. Adding a field to the summary without mapping it would otherwise be an invisible omission — the code compiles, the tests pass, and the new measurement simply never appears; pinning the count turns that silence into a failure that names what to do about it. |
//! | [`health_summary_samples_all_finite_or_nan`] | metrics | The two kinds of metric are held to different standards, and each sample meets the one for its own kind: a counter is always finite and never negative, while a gauge may additionally be NaN. Only a gauge can stand for a measurement that does not exist yet; a counter always does, because a thing that has happened zero times has happened zero times, and a negative or infinite counter would break the rate arithmetic built on it. |
//! | [`counters_are_non_negative`] | metrics | cites (´claim:metrics:counters-are-always-finite-and-non-negative-while-only-gauges-may-be-nan´) |
//! | [`full_report_standardisation_snapshot_version`] | metrics | The full report's standardisation snapshot version reaches the metrics surface unchanged, so a scraped phase and accepted count retain the join key that identifies their published coordinate state. |
//! | [`full_report_produces_per_entity_samples`] | metrics | Each kind of entity contributes a fixed number of labelled samples — so many per model, per Sentinel, per identity dimension — and no more. Export volume is therefore a linear function of how many entities are registered, which is what makes the cost of running a large fleet predictable in advance rather than discovered when the collector falls over. |
//! | [`full_report_per_sentinel_labelled`] | metrics | An entity-scoped measurement carries exactly one label, naming the entity it belongs to and nothing besides — and every registered Sentinel gets its own sample of each such metric, whether it is currently reporting or not. One label per series keeps cardinality equal to the entity count, and emitting even for the silent ones is what lets an operator see a Sentinel that has gone quiet rather than merely find its series missing. |
//! | [`informativeness_export_carries_measurement_and_absence`] | metrics | Weight mass is exported as the measurement it is: a Sentinel whose mean absolute operational slot weight was measured arrives with that value untouched, and an unmeasured Sentinel arrives as NaN rather than as a reassuring zero. Zero is a reading — the model carrying no weight there — and it is a different fact from having taken no reading at all, so a gauge that flattened the two would put a figure on the surface for a Sentinel nothing had been measured on (´def:monitoring:encoding-effectiveness´). |
//! | [`dimension_informativeness_export_carries_measurement_and_absence`] | metrics | The per-dimension reading is exported under the same discipline as the per-Sentinel one: a dimension whose mean absolute operational block weight was measured arrives with that value untouched, and an unmeasured dimension arrives as NaN. Zero is a reading here too, and absence is not it, so a gauge that reported absence as zero would put a figure on the surface for a dimension nothing had been measured on (´def:monitoring:dimension-informativeness´). |
//! | [`dimension_coverage_and_cell_weight_export_carry_measurement_and_absence`] | metrics | The identity-health table's two remaining rows export under the discipline the block reading already keeps: a measured figure arrives untouched and an unmeasured one arrives as NaN. Zero is a real reading for both — cells that carry no weight, and traffic that missed the competitive set entirely — so a gauge that reported absence as zero would put a figure on the surface for a dimension nothing had been measured on, and would put it there in exactly the range an operator is watching for trouble (´tab:monitoring:dimension-health´). |
//! | [`full_report_per_dimension_labelled`] | metrics | cites (´claim:metrics:an-entity-scoped-sample-carries-exactly-the-one-label-naming-its-entity´) |
//! | [`challenge_metrics_are_host_surface`] | metrics | Challenge metrics are declared in the catalogue but marked as the host's to emit, and no core mapper produces one. The core has no view of what happened after a decision was returned, so it could only invent these values; declaring them anyway means the host that does know publishes them under names that match the rest of the surface instead of inventing its own. |
//! | [`full_report_buffer_utilisation`] | metrics | Pending-buffer occupancy is exported as a fraction of capacity, computed at export time from the two raw figures. A fraction is comparable across deployments and directly alertable — the same threshold means the same thing whatever the configured capacity — whereas a raw occupancy count only becomes meaningful once the reader also knows how large the buffer is. |
//! | [`positive_class_priors_reach_summary_samples`] | metrics | The two distinct priors carried by the published snapshot remain distinct on the cheap metric surface: neither is substituted for the other, rounded, or replaced by the cold-start default. |
//! | [`drift_reset_counter_reaches_summary_samples`] | metrics | The monotone count of emitted drift-reset events reaches the summary sample unchanged under its counter name. |
//! | [`full_report_signal_cache_metrics_carry_live_statistics`] | metrics | Signal-cache hits, misses, size, capacity and evictions survive report assembly as raw statistics, and the mapper derives hit rate and utilisation from those live values. |
//! | [`full_report_sentinel_coverage_carries_reporting_fraction`] | metrics | The fleet-wide coverage sample carries the fraction assembled from registered Sentinels with live reports, including the defined zero for an empty registry. |
//! | [`full_report_concordance_calibrations`] | metrics | The number of concordance calibrations the tracker has completed reaches the export as the tracker recorded it. Concordance thresholds are recomputed periodically from the observed distribution, and this count is how an operator confirms that recalibration is actually happening rather than silently stalled behind a window that never fills. |
//! | [`full_report_marginalisation_loss_fraction`] | metrics | The largest correction-loss fraction accumulated by marginalisation reaches the metrics surface under the Schur namespace and with its value unchanged. It is the dimensionless approximation figure a deployment can compare across the model's life; leaving it only in the health report would make a scraper blind to the trend it exists to expose. |
//! | [`cascade_terminus_emitted_from_summary_only`] | metrics | A metric name has exactly one emission site even when both mappers have access to the underlying state: the cascade-terminus flag comes from the summary mapper, and the full-report mapper stays silent about it although its own report carries the same field. Two mappers emitting one name would put two samples of the same series in a single scrape, and which one won would depend on the collector rather than on the system. |
//! | [`health_events_dropped_emitted_from_summary_only`] | metrics | cites (´claim:metrics:each-metric-name-has-a-single-emission-site-even-when-both-mappers-can-see-the-state´) |
//! | [`full_report_cardinality_bounded`] | metrics | A substantial deployment — twenty Sentinels across five identity dimensions — still exports well under a couple of thousand samples in one scrape. Time-series cardinality is the resource a metrics backend actually runs out of, so the export staying inside a stated budget at realistic scale is what makes it safe to enable in production rather than only in a demo. |
//! | [`host_surface_entries_not_emitted`] | metrics | The surface declaration is binding on the mappers: nothing marked as the host's is emitted by either of them, and there is genuinely at least one such entry for the rule to bite on. Were the core to emit a host-surface name as well, the host's own value and the core's would collide in the same series, and the field would stop meaning what the catalogue says it means. |
//! | [`every_catalog_entry_is_emitted`] | metrics | The catalogue cannot promise more than the code delivers: given a summary and a report populated richly enough to reach every branch, every entry not reserved for the host is produced by one mapper or the other. Together with the reverse direction this closes the contract in both ways — a name in the catalogue is a name an operator can build a dashboard on, and a declared metric that nothing emits is a broken promise the build refuses to ship. |
//! | [`every_emitted_sample_is_catalogued`] | metrics | The reverse direction closes the contract: every sample either mapper emits is catalogued, under its declared type and its declared label keys. The catalogue decides it is a compile-time constant precisely so it cannot drift from what the code emits, but nothing mechanical held the emitting half of that promise — a mapper could emit a name no host registered, and the sample would arrive unclassifiable. Walking every emitted sample back into the catalogue is the half of the correspondence the forward test cannot see (´dec:metrics:compile-time-catalogue´). |
//! | [`full_report_exports_the_repaired_count`] | metrics | The repaired count reaches the export under the name that says what it now counts, carrying each model's own reading. The metric was `assayer_cholesky_regularised_total` and counted the retired factorisation shift; with the shift gone it would have exported a constant zero for every model for ever, which is a measurement promised and not delivered. Renaming it with the quantity is what keeps the promise, and this reads the two models' distinct values through the mapper to show the rename carried the wiring rather than only the string. |

//! Metrics export acceptance tests, passive throughout (´dec:metrics:passive-export´).
//!
//! The catalogue is the contract: a declared set of metric names, each with a
//! kind and a surface saying who emits it. Two mappers project internal state
//! onto those names — one from the lightweight health summary, one from the
//! full system report — and between them the catalogue and the emission must
//! agree exactly in both directions, so an operator's dashboard never queries
//! a name nothing produces and no metric appears that was never declared.

use std::collections::{BTreeMap, HashMap, HashSet};

use indexmap::IndexMap;

use crate::health::{
    AlarmOutcomeHealth, AssessmentDegradationSnapshot, AssessmentDegradationSummary, BlendStatistics, BufferHealth,
    CalibrationHealth, CompositeConvergenceStage, ConcordanceHealth, ConvergenceHealth, HealthSummary, IdentityConvergenceHealth,
    IdentityConvergenceStage, IdentityDimensionHealth, ImportanceCeilingHealth, LabelIntegrityHealth, LabelQueueHealth,
    LedgerHealth, ObservabilityHealth, PlattConvergenceState, PrecisionHealthDetail, PublishedRebuildVerdict, SentinelHealth,
    StandardisationHealth, SystemHealthReport,
};
use crate::metrics::{
    METRICS_CATALOG, MetricSurface, MetricType, full_report_to_samples, health_summary_to_samples, model_label, option_to_metric,
};
use crate::signal::SignalCacheHealth;
use crate::testing::{DEFAULT_TOLERANCES, assert_near};
use crate::types::{DimensionId, ModelId, OutcomeAxisId, SentinelId};

/// Constructs a default `HealthSummary` for testing.
fn default_health_summary() -> HealthSummary {
    HealthSummary {
        convergence_stage: CompositeConvergenceStage::default(),
        platt_state: PlattConvergenceState::default(),
        last_delta_cal: 0.0,
        platt_refits_completed: 0,
        total_labels: 0,
        eligible_labels: 0,
        total_assessments: 0,
        degradation: AssessmentDegradationSnapshot::default(),
        blend_statistics: BlendStatistics::default(),
        feature_stable_outcome_drift: false,
        quantile_error_concentration: None,
        cp5_reverts: 0,
        cp6_reverts: 0,
        auc_aggregate: None,
        auc_recent: None,
        max_diagonal_ratio: 0.0,
        max_floor_share: 0.0,
        max_sync_error: 0.0,
        max_prior_induced_sync_error: 0.0,
        max_sync_error_residual: 0.0,
        any_cascade_terminus: false,
        label_path_stopped: false,
        platt_kappa_sister: 1.0,
        platt_kappa_anchor: 1.0,
        p_positive_global: 0.5,
        p_positive_eligible: 0.5,
        health_events_dropped: 0,
        drift_resets: 0,
    }
}

/// Constructs a minimal `SystemHealthReport` with configurable entity counts.
fn test_system_report(n_sentinels: usize, n_dimensions: usize) -> SystemHealthReport {
    test_system_report_full(n_sentinels, n_dimensions, 0)
}

/// Constructs a `SystemHealthReport` with configurable entity counts.
fn test_system_report_full(n_sentinels: usize, n_dimensions: usize, _n_channels: usize) -> SystemHealthReport {
    let mut sentinels = IndexMap::new();
    for i in 0..n_sentinels {
        sentinels.insert(
            SentinelId(i as u32),
            SentinelHealth {
                has_report: i % 2 == 0,
                structural: None,
                alarm_outcome: AlarmOutcomeHealth::default(),
                ledger_entry_count: i * 10,
                report_age_seconds: if i % 2 == 0 { Some(i as f64 * 1.5) } else { None },
                // A measured value on the even Sentinels, unmeasured on
                // the odd — the export must keep the two distinguishable.
                informativeness: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.01, 0.25))
                } else {
                    None
                },
                // The two legibility readings follow the same discipline: a
                // figure on the even Sentinels and absence on the odd, so the
                // export is exercised on both for every reading it carries.
                association: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.5, 19.7))
                } else {
                    None
                },
                contribution: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.01, 0.42))
                } else {
                    None
                },
                report_stage_lower_bound_seconds: None,
            },
        );
    }
    let sentinel_coverage = if sentinels.is_empty() {
        0.0
    } else {
        sentinels.values().filter(|sentinel| sentinel.has_report).count() as f64 / sentinels.len() as f64
    };

    let mut identity_dimensions = IndexMap::new();
    for i in 0..n_dimensions {
        identity_dimensions.insert(
            DimensionId(i as u32),
            IdentityDimensionHealth {
                name: format!("identity-{i}"),
                description: "metrics fixture hierarchy".to_owned(),
                coordinate_semantics: "prefixes identify fixture groups".to_owned(),
                convergence: IdentityConvergenceHealth {
                    change_rate_ewma: 0.01 * (i as f64),
                    jaccard_ewma: 0.95,
                    current_cell_count: 100 + i,
                    total_entries: 500,
                    total_exits: 50,
                    convergence_stage: IdentityConvergenceStage::Stabilising,
                },
                competitive_cell_depth_distribution: BTreeMap::default(),
                active_indicator_count_distribution: BTreeMap::default(),
                // Traffic on the even dimensions and none yet on the odd, so
                // the fixture carries a measured share beside the absence that
                // precedes any traffic at all.
                competitive_coverage_fraction: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.05, 0.25))
                } else {
                    None
                },
                observations_dropped: 0,
                // Measured on the even dimensions, unmeasured on the odd —
                // the same discipline the Sentinel reading is fixtured under,
                // because it is the same reading over a different block.
                informativeness: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.01, 0.5))
                } else {
                    None
                },
                cell_weight_mass: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.01, 0.75))
                } else {
                    None
                },
                association: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.5, 12.3))
                } else {
                    None
                },
                contribution: if i % 2 == 0 {
                    Some((i as f64).mul_add(0.01, 0.31))
                } else {
                    None
                },
            },
        );
    }

    let mut precision = HashMap::new();
    precision.insert(
        ModelId::Operational,
        PrecisionHealthDetail {
            kappa: Some(42.0),
            diagonal_ratio: 42.0,
            lambda_min: Some(1.0e-6),
            spectral_floor: Some(2.0e-8),
            floor_share: Some(0.02),
            alarms: 0,
            measurements: 4,
            last_measurement_labels: 1_000,
            last_sync_error: 2.5e-7,
            last_prior_induced_sync_error: 0.0,
            last_sync_error_residual: 2.5e-7,
            last_sync_error_resolution: 1.0e-12,
            last_measurement_at_resolution: false,
            last_sync_error_after: Some(1.0e-7),
            last_rebuild_verdict: Some(PublishedRebuildVerdict::Clean),
            last_rebuild_adopted: Some(true),
            n_recompute_effective: 500,
            sync_error_shortenings: 2,
            floored_rebuilds: 3,
            cholesky_recomputes: 10,
            cascade_terminus_count: 0,
            dimensions_at_floor: 0,
        },
    );
    precision.insert(
        ModelId::Sister,
        PrecisionHealthDetail {
            kappa: Some(55.0),
            diagonal_ratio: 55.0,
            lambda_min: Some(2.0e-6),
            spectral_floor: Some(2.0e-8),
            floor_share: Some(0.01),
            alarms: 0,
            measurements: 6,
            last_measurement_labels: 1_000,
            last_sync_error: 4.0e-7,
            last_prior_induced_sync_error: 0.0,
            last_sync_error_residual: 4.0e-7,
            last_sync_error_resolution: 1.0e-12,
            last_measurement_at_resolution: false,
            last_sync_error_after: Some(1.5e-7),
            last_rebuild_verdict: Some(PublishedRebuildVerdict::Clean),
            last_rebuild_adopted: Some(true),
            n_recompute_effective: 1_000,
            sync_error_shortenings: 0,
            floored_rebuilds: 1,
            cholesky_recomputes: 8,
            cascade_terminus_count: 0,
            dimensions_at_floor: 0,
        },
    );

    SystemHealthReport {
        convergence: ConvergenceHealth {
            composite_stage: CompositeConvergenceStage::default(),
            platt_state: PlattConvergenceState::default(),
            identity_stages: HashMap::new(),
        },
        drift: HashMap::new(),
        precision,
        calibration: CalibrationHealth {
            kappa_sister: 1.0,
            kappa_anchor: 1.0,
            delta_cal: 0.0,
            refits_completed: 0,
            labels_since_refit: 0,
            buffer_total: 0,
            sister_regime_records: 0.0,
            anchor_regime_records: 0.0,
            anchor_regime_frozen: false,
            labels_since_last_anchor_fit: 0,
            convergence_state: PlattConvergenceState::default(),
        },
        sentinels,
        sentinel_coverage,
        axes: IndexMap::new(),
        identity_dimensions,
        buffer: BufferHealth {
            capacity: 1_000_000,
            utilisation: 500,
            insertions: 500,
            evictions: 0,
            eviction_rate: 0.0,
            oldest_pending_age_seconds: None,
        },
        label_queue: LabelQueueHealth {
            capacity: 10_000,
            depth: 0,
            drops: 0,
        },
        standardisation: StandardisationHealth {
            snapshot_version: 47,
            phase: crate::feature::standardisation::StandardisationPhase::default(),
            accepted_observations: 0,
            features_at_floor: 0,
        },
        label_integrity: LabelIntegrityHealth {
            total_labels: 0,
            eligible_labels: 0,
            eligibility_rate: 0.0,
            per_action: HashMap::new(),
            valences_sanitised: 0,
            outcomes_dropped: 0,
        },
        assessment_degradation: AssessmentDegradationSummary {
            total_assessments: 100,
            degraded_fraction: 0.02,
            counters: AssessmentDegradationSnapshot {
                total_degraded: 2,
                signals_sanitised: 1,
                signals_shape_mismatched: 0,
                signals_unknown: 0,
                sentinel_slots_zeroed: 0,
                features_sanitised: 1,
                model_fallbacks: 0,
                batch_init_observations_skipped: 0,
                zero_sentinel_assessments: 0,
            },
        },
        discrimination: None,
        blend_statistics: BlendStatistics::default(),
        concordance: ConcordanceHealth {
            window_size: 0,
            window_capacity: 5000,
            calibrations_completed: 7,
            current_thresholds: [1.0; 4],
            total_observations: 0,
            per_entity: HashMap::new(),
            per_entity_capacity: 10_000,
            per_entity_evictions: 0,
        },
        ledger: LedgerHealth::default(),
        observability: ObservabilityHealth::default(),
        cross_layer: crate::health::CrossLayerHealth::default(),
        importance_ceiling: ImportanceCeilingHealth::default(),
        marginalisation: crate::health::MarginalisationHealth::default(),
        cascade_terminus_active: false,
        label_path_stop: None,
        health_events_dropped: 0,
        schur_corrections_skipped: 0,
        signal_cache: SignalCacheHealth {
            hits: 0,
            misses: 0,
            evictions: 0,
            capacity: 1_000,
            size: 0,
        },
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Catalog Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The exported surface is a deliberately sized catalogue, held within a band
/// rather than growing to whatever the mappers happen to emit. A metrics
/// surface is an interface an operator builds dashboards and alerts against;
/// letting it drift by dozens either way would mean it was never designed, and
/// the band is what makes an unnoticed sprawl a failure rather than a
/// diff nobody reads.
///
/// ´claim:metrics:the-exported-surface-is-a-deliberately-sized-catalogue-not-whatever-the-mappers-happen-to-emit´
/// ´test:crate:catalog-size-stays-inside-its-band´
#[test]
fn catalog_size_stays_inside_its_band() {
    // The catalogue's size is stated nowhere outside the code
    // (´conv:metrics:size-unstated´): ~66 metrics, grown deliberately
    // since by the weighted sample-count pair, the wave-nine batch-init
    // load-condition counters, the observability-deferral activations, the
    // two legibility readings at each of the two entity scopes, and the
    // per-dimension coverage share and cell weight mass the identity-health
    // table names.
    // The band moves when the surface is deliberately resized, never to
    // absorb an unnoticed sprawl.
    assert!(
        METRICS_CATALOG.len() >= 60,
        "catalog has {} entries, expected >= 60",
        METRICS_CATALOG.len()
    );
    assert!(
        METRICS_CATALOG.len() <= 75,
        "catalog has {} entries, expected <= 75",
        METRICS_CATALOG.len()
    );
}

/// Every declared metric name is namespaced to the crate. Metrics from many
/// subsystems land in one flat namespace in the collector a host runs, so an
/// unprefixed name would sooner or later collide with something else's, and
/// the two series would silently merge into one meaningless one.
///
/// ´claim:metrics:every-declared-metric-name-is-namespaced-to-the-crate´
/// ´test:crate:all-metric-names-start-with-assayer´
#[test]
fn all_metric_names_start_with_assayer() {
    for def in METRICS_CATALOG {
        assert!(
            def.name.starts_with("assayer_"),
            "metric {:?} does not start with 'assayer_'",
            def.name
        );
    }
}

/// A metric's name announces its kind: everything declared as a counter ends
/// in the conventional suffix, and nothing else has to. Counters and gauges are
/// queried quite differently — one is differentiated over time, the other read
/// as it stands — so the naming convention is what lets an operator write the
/// right query from the name alone, without consulting the catalogue.
///
/// ´claim:metrics:a-metrics-name-announces-its-kind-with-counters-carrying-the-conventional-suffix´
/// ´test:crate:all-counter-names-end-with-total´
#[test]
fn all_counter_names_end_with_total() {
    // What each catalogue entry declares (´dec:metrics:catalogue-membership´):
    // counters end with `_total`.
    for def in METRICS_CATALOG {
        if def.metric_type == MetricType::Counter {
            assert!(
                def.name.ends_with("_total"),
                "counter metric {:?} does not end with '_total'",
                def.name
            );
        }
    }
}

/// No two catalogue entries share a name. A name is the only handle an
/// operator has on a measurement, so a duplicate would leave two different
/// quantities indistinguishable at the point of query — and worse, would make
/// the catalogue's own claim to be a complete description of the surface
/// false.
///
/// ´claim:metrics:no-two-catalogue-entries-share-a-name´
/// ´test:crate:all-metric-names-are-unique´
#[test]
fn all_metric_names_are_unique() {
    let mut seen = HashSet::new();
    for def in METRICS_CATALOG {
        assert!(seen.insert(def.name), "duplicate metric name: {:?}", def.name);
    }
}

/// Every catalogue entry is a counter or a gauge and nothing else — the two
/// kinds account for the whole catalogue exactly — and each kind is
/// substantially represented. The surface therefore describes both what has
/// accumulated over the system's life and what is true of it right now;
/// a surface of only counters could not report a current state, and one of
/// only gauges could not report a rate.
///
/// ´claim:metrics:every-catalogue-entry-is-a-counter-or-a-gauge-and-both-kinds-are-substantially-represented´
/// ´test:crate:catalog-counters-and-gauges-counts´
#[test]
fn catalog_counters_and_gauges_counts() {
    let counters = METRICS_CATALOG
        .iter()
        .filter(|d| d.metric_type == MetricType::Counter)
        .count();
    let gauges = METRICS_CATALOG.iter().filter(|d| d.metric_type == MetricType::Gauge).count();

    assert!(counters >= 15, "expected >= 15 counters, got {counters}");
    assert!(gauges >= 20, "expected >= 20 gauges, got {gauges}");
    assert_eq!(counters + gauges, METRICS_CATALOG.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Encoding Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A measurement the system does not have is exported as NaN, never as zero.
/// The difference matters enormously for a quantity like discrimination: zero
/// would read as catastrophically bad performance and page someone, whereas
/// NaN reads as "not yet measured" and is skipped by aggregation rather than
/// dragging an average down.
///
/// ´claim:metrics:a-measurement-the-system-does-not-have-is-exported-as-nan-never-as-zero´
/// ´test:crate:option-none-maps-to-nan´
#[test]
fn option_none_maps_to_nan() {
    assert!(option_to_metric(None).is_nan());
}

/// A measurement the system does have is exported as exactly that value.
/// Nothing is rounded, rescaled, or clipped on the way out, so the number on
/// an operator's dashboard is the number the model was working with.
///
/// ´claim:metrics:a-measurement-that-exists-is-exported-as-exactly-its-own-value´
/// ´test:crate:option-some-maps-to-value´
#[test]
fn option_some_maps_to_value() {
    assert_eq!(option_to_metric(Some(42.5)), 42.5);
}

/// A condition that is either true or false is exported as one or zero, and
/// the exported value tracks the condition as it changes. Metrics collectors
/// carry numbers and not flags, so an alert on a boolean condition has to be
/// expressible as a numeric comparison — which it can be, precisely because the
/// two states occupy two definite values.
///
/// ´claim:metrics:a-boolean-condition-exports-as-one-or-zero-so-it-can-be-alerted-on-numerically´
/// ´test:crate:boolean-maps-to-zero-or-one´
#[test]
fn boolean_maps_to_zero_or_one() {
    let mut summary = default_health_summary();

    summary.feature_stable_outcome_drift = false;
    let samples = health_summary_to_samples(&summary);
    let drift = samples
        .iter()
        .find(|s| s.name == "assayer_feature_stable_outcome_drift")
        .unwrap();
    assert_eq!(drift.value, 0.0);

    summary.feature_stable_outcome_drift = true;
    let samples = health_summary_to_samples(&summary);
    let drift = samples
        .iter()
        .find(|s| s.name == "assayer_feature_stable_outcome_drift")
        .unwrap();
    assert_eq!(drift.value, 1.0);
}

/// A lifecycle stage exports as its position in the sequence, with the settled
/// end of the composite progression carrying the highest number. Because the
/// encoding preserves the order, an operator can alert on the system having
/// fallen below a stage — a comparison that would be meaningless against
/// arbitrary codes.
///
/// ´claim:metrics:an-ordered-lifecycle-stage-exports-as-its-ordinal-so-progress-is-numerically-comparable´
/// ´test:crate:convergence-stage-maps-to-integer´
#[test]
fn convergence_stage_maps_to_integer() {
    let mut summary = default_health_summary();
    summary.convergence_stage = CompositeConvergenceStage::SteadyState;
    let samples = health_summary_to_samples(&summary);
    let stage = samples.iter().find(|s| s.name == "assayer_convergence_stage").unwrap();
    assert_eq!(stage.value, 5.0);
}

/// The calibration lifecycle encodes the same way, running from its initial
/// state at zero up through the first fit and convergence to converged at
/// three, with each state pinned to its own number. The values are contiguous
/// and gapless from the bottom, so a dashboard can show the calibrator's
/// progress on a single ordered scale.
///
/// (´claim:metrics:an-ordered-lifecycle-stage-exports-as-its-ordinal-so-progress-is-numerically-comparable´)
/// ´test:crate:platt-state-maps-to-integer´
#[test]
fn platt_state_maps_to_integer() {
    let mut summary = default_health_summary();

    for (state, expected) in [
        (PlattConvergenceState::Initial, 0.0),
        (PlattConvergenceState::FirstFit, 1.0),
        (PlattConvergenceState::Converging, 2.0),
        (PlattConvergenceState::Converged, 3.0),
    ] {
        summary.platt_state = state;
        let samples = health_summary_to_samples(&summary);
        let s = samples.iter().find(|s| s.name == "assayer_platt_state").unwrap();
        assert_eq!(s.value, expected, "PlattConvergenceState::{state:?}");
    }
}

/// Each of the three named models labels itself with its own name, while an
/// outcome axis labels itself by its identifier. There are exactly three named
/// models but arbitrarily many axes, so the axis label has to be generated
/// from the identity rather than enumerated — and generating it keeps every
/// axis's series distinct without the catalogue having to grow an entry per
/// axis.
///
/// ´claim:metrics:each-model-labels-itself-by-name-while-outcome-axes-label-themselves-by-identifier´
/// ´test:crate:model-label-encoding´
#[test]
fn model_label_encoding() {
    assert_eq!(model_label(&ModelId::Operational), "operational");
    assert_eq!(model_label(&ModelId::Sister), "sister");
    assert_eq!(model_label(&ModelId::Anchor), "anchor");
    assert_eq!(model_label(&ModelId::OutcomeAxis(OutcomeAxisId(7))), "axis_7");
}

// ═══════════════════════════════════════════════════════════════════════════════
// HealthSummary Mapper Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Expected number of samples from `health_summary_to_samples()`.
///
/// Bump this constant when adding a new `HealthSummary` field + mapper entry.
const EXPECTED_SUMMARY_SAMPLE_COUNT: usize = 40;

/// Every field of the health summary reaches the export under its own name and
/// carrying its own value — the assessment and label tallies, each separate
/// degradation counter, the whole blend-weight distribution, and the optional
/// discrimination figures alike. A field held internally but never exported is
/// invisible to the only people who could act on it, so the mapping has to be
/// exhaustive rather than a selection of what seemed interesting.
///
/// ´claim:metrics:every-field-of-the-health-summary-reaches-the-export-carrying-its-own-value´
/// ´test:crate:health-summary-to-samples-has-every-field´
#[test]
fn health_summary_to_samples_has_every_field() {
    let summary = HealthSummary {
        convergence_stage: CompositeConvergenceStage::SisterConverging,
        platt_state: PlattConvergenceState::Converging,
        last_delta_cal: 0.05,
        platt_refits_completed: 12,
        total_labels: 500,
        eligible_labels: 450,
        total_assessments: 10_000,
        degradation: AssessmentDegradationSnapshot {
            total_degraded: 20,
            signals_sanitised: 5,
            signals_shape_mismatched: 4,
            signals_unknown: 6,
            sentinel_slots_zeroed: 2,
            features_sanitised: 3,
            model_fallbacks: 1,
            batch_init_observations_skipped: 0,
            zero_sentinel_assessments: 7,
        },
        blend_statistics: BlendStatistics {
            mean: 0.15,
            p10: 0.02,
            p50: 0.12,
            p90: 0.30,
            p99: 0.45,
            fraction_anchor_dominated: 0.08,
            w_infinity_estimate: 0.05,
            steady_state: true,
            window_size: 500,
        },
        feature_stable_outcome_drift: true,
        quantile_error_concentration: Some(1.8),
        cp5_reverts: 1,
        cp6_reverts: 0,
        auc_aggregate: Some(0.82),
        auc_recent: Some(0.79),
        max_diagonal_ratio: 42.0,
        max_floor_share: 0.25,
        max_sync_error: 0.001,
        max_prior_induced_sync_error: 0.0,
        max_sync_error_residual: 0.001,
        any_cascade_terminus: false,
        label_path_stopped: false,
        platt_kappa_sister: 1.5,
        platt_kappa_anchor: 1.2,
        p_positive_global: 0.37,
        p_positive_eligible: 0.61,
        health_events_dropped: 0,
        drift_resets: 4,
    };

    let samples = health_summary_to_samples(&summary);

    // Every HealthSummary field must have a corresponding sample.
    let names: HashSet<&str> = samples.iter().map(|s| s.name).collect();

    // Counters
    assert!(names.contains("assayer_assessments_total"));
    assert!(names.contains("assayer_labels_total"));
    assert!(names.contains("assayer_labels_eligible_total"));
    assert!(names.contains("assayer_degraded_assessments_total"));
    assert!(names.contains("assayer_signals_sanitised_total"));
    assert!(names.contains("assayer_signal_shape_mismatches_total"));
    assert!(names.contains("assayer_unknown_signals_total"));
    assert!(names.contains("assayer_sentinel_slots_zeroed_total"));
    assert!(names.contains("assayer_features_sanitised_total"));
    assert!(names.contains("assayer_model_fallbacks_total"));
    assert!(names.contains("assayer_zero_sentinel_assessments_total"));
    assert!(names.contains("assayer_cp5_reverts_total"));
    assert!(names.contains("assayer_cp6_reverts_total"));
    assert!(names.contains("assayer_platt_refits_total"));
    assert!(names.contains("assayer_drift_resets_total"));

    // Gauges
    assert!(names.contains("assayer_convergence_stage"));
    assert!(names.contains("assayer_platt_state"));
    assert!(names.contains("assayer_platt_delta_cal"));
    assert!(names.contains("assayer_blend_weight_mean"));
    assert!(names.contains("assayer_blend_weight_p10"));
    assert!(names.contains("assayer_blend_weight_p50"));
    assert!(names.contains("assayer_blend_weight_p90"));
    assert!(names.contains("assayer_blend_weight_p99"));
    assert!(names.contains("assayer_blend_fraction_anchor_dominated"));
    assert!(names.contains("assayer_w_infinity_estimate"));
    assert!(names.contains("assayer_blend_steady_state"));
    assert!(names.contains("assayer_blend_window_size"));
    assert!(names.contains("assayer_feature_stable_outcome_drift"));
    assert!(names.contains("assayer_quantile_error_concentration"));
    assert!(names.contains("assayer_auc_aggregate"));
    assert!(names.contains("assayer_positive_class_prior_global"));
    assert!(names.contains("assayer_positive_class_prior_eligible"));

    // Verify specific values.
    let find = |name: &str| samples.iter().find(|s| s.name == name).unwrap();
    assert_eq!(find("assayer_assessments_total").value, 10_000.0);
    assert_eq!(find("assayer_labels_total").value, 500.0);
    assert_eq!(find("assayer_labels_eligible_total").value, 450.0);
    assert_eq!(find("assayer_signal_shape_mismatches_total").value, 4.0);
    assert_eq!(find("assayer_unknown_signals_total").value, 6.0);
    assert_eq!(find("assayer_zero_sentinel_assessments_total").value, 7.0);
    assert_eq!(find("assayer_drift_resets_total").value, 4.0);
    assert_eq!(find("assayer_convergence_stage").value, 3.0);
    assert_eq!(find("assayer_platt_delta_cal").value, 0.05);
    assert_eq!(find("assayer_blend_weight_mean").value, 0.15);
    assert_eq!(find("assayer_feature_stable_outcome_drift").value, 1.0);
    assert_eq!(find("assayer_quantile_error_concentration").value, 1.8);
    assert_eq!(find("assayer_auc_aggregate").value, 0.82);
    assert_eq!(find("assayer_positive_class_prior_global").value, 0.37);
    assert_eq!(find("assayer_positive_class_prior_eligible").value, 0.61);
}

/// The summary export emits a pinned number of samples, so the surface cannot
/// change size unnoticed. Adding a field to the summary without mapping it
/// would otherwise be an invisible omission — the code compiles, the tests
/// pass, and the new measurement simply never appears; pinning the count turns
/// that silence into a failure that names what to do about it.
///
/// ´claim:metrics:the-summary-export-has-a-pinned-sample-count-so-an-unmapped-new-field-cannot-pass-unnoticed´
/// ´test:crate:health-summary-sample-count-matches´
#[test]
fn health_summary_sample_count_matches() {
    let summary = default_health_summary();
    let samples = health_summary_to_samples(&summary);
    assert_eq!(
        samples.len(),
        EXPECTED_SUMMARY_SAMPLE_COUNT,
        "sample count drift: got {}, expected {EXPECTED_SUMMARY_SAMPLE_COUNT}. \
         Update EXPECTED_SUMMARY_SAMPLE_COUNT when adding new HealthSummary fields.",
        samples.len()
    );
}

/// The two kinds of metric are held to different standards, and each sample
/// meets the one for its own kind: a counter is always finite and never
/// negative, while a gauge may additionally be NaN. Only a gauge can stand for
/// a measurement that does not exist yet; a counter always does, because a
/// thing that has happened zero times has happened zero times, and a negative
/// or infinite counter would break the rate arithmetic built on it.
///
/// ´claim:metrics:counters-are-always-finite-and-non-negative-while-only-gauges-may-be-nan´
/// ´test:crate:health-summary-samples-all-finite-or-nan´
#[test]
fn health_summary_samples_all_finite_or_nan() {
    let summary = default_health_summary();
    let samples = health_summary_to_samples(&summary);

    for sample in &samples {
        match sample.metric_type {
            MetricType::Counter => {
                assert!(
                    sample.value.is_finite() && sample.value >= 0.0,
                    "counter {:?} has invalid value {}",
                    sample.name,
                    sample.value
                );
            }
            MetricType::Gauge => {
                // Gauges can be NaN (from Option::None) or finite.
                assert!(
                    sample.value.is_finite() || sample.value.is_nan(),
                    "gauge {:?} has invalid value {}",
                    sample.name,
                    sample.value
                );
            }
        }
    }
}

/// The non-negativity of counters is not an artefact of everything being zero:
/// it still holds once assessments, labels and reverts have actually
/// accumulated. Counters are read by differencing consecutive scrapes, so a
/// value that ever went down would be read as a restart and the interval's
/// worth of activity attributed to a counter reset.
///
/// (´claim:metrics:counters-are-always-finite-and-non-negative-while-only-gauges-may-be-nan´)
/// ´test:crate:counters-are-non-negative´
#[test]
fn counters_are_non_negative() {
    let mut summary = default_health_summary();
    summary.total_assessments = 42;
    summary.total_labels = 10;
    summary.cp5_reverts = 1;

    let samples = health_summary_to_samples(&summary);
    for sample in &samples {
        if sample.metric_type == MetricType::Counter {
            assert!(sample.value >= 0.0, "counter {:?} is negative: {}", sample.name, sample.value);
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// SystemHealthReport Mapper Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The full report's standardisation snapshot version reaches the metrics
/// surface unchanged, so a scraped phase and accepted count retain the join
/// key that identifies their published coordinate state.
///
/// ´claim:metrics:the-standardisation-snapshot-version-reaches-the-export-as-the-coordinate-join-key´
/// ´test:crate:full-report-standardisation-snapshot-version´
#[test]
fn full_report_standardisation_snapshot_version() {
    let report = test_system_report(0, 0);
    let samples = full_report_to_samples(&report);
    let version = samples
        .iter()
        .find(|s| s.name == "assayer_standardisation_snapshot_version")
        .expect("standardisation snapshot version emitted");

    assert_eq!(version.value, 47.0);
}

/// Each kind of entity contributes a fixed number of labelled samples — so
/// many per model, per Sentinel, per identity dimension — and no more. Export
/// volume is therefore a linear function of how many entities are registered,
/// which is what makes the cost of running a large fleet predictable in
/// advance rather than discovered when the collector falls over.
///
/// ´claim:metrics:each-entity-contributes-a-fixed-number-of-samples-so-export-volume-is-linear-in-the-fleet´
/// ´test:crate:full-report-produces-per-entity-samples´
#[test]
fn full_report_produces_per_entity_samples() {
    let report = test_system_report_full(3, 2, 1);
    let samples = full_report_to_samples(&report);
    assert!(!samples.is_empty());

    // Per-model samples (2 models × 4 metrics = 8).
    let model_count = samples.iter().filter(|s| s.labels.iter().any(|(k, _)| *k == "model")).count();
    assert_eq!(model_count, 8, "expected 8 per-model samples, got {model_count}");

    // Per-sentinel samples (3 sentinels × 6 metrics = 18).
    let sentinel_count = samples
        .iter()
        .filter(|s| s.labels.iter().any(|(k, _)| *k == "sentinel"))
        .count();
    assert_eq!(sentinel_count, 18, "expected 18 per-sentinel samples, got {sentinel_count}");

    // Per-dimension samples (2 dimensions × 8 metrics = 16).
    let dim_count = samples
        .iter()
        .filter(|s| s.labels.iter().any(|(k, _)| *k == "dimension"))
        .count();
    assert_eq!(dim_count, 16, "expected 16 per-dimension samples, got {dim_count}");

    // Challenge metrics are host-owned and are not emitted from core full reports.
    let channel_count = samples
        .iter()
        .filter(|s| s.labels.iter().any(|(k, _)| *k == "channel"))
        .count();
    assert_eq!(channel_count, 0, "expected no core per-channel samples, got {channel_count}");
}

/// An entity-scoped measurement carries exactly one label, naming the entity
/// it belongs to and nothing besides — and every registered Sentinel gets its
/// own sample of each such metric, whether it is currently reporting or not.
/// One label per series keeps cardinality equal to the entity count, and
/// emitting even for the silent ones is what lets an operator see a Sentinel
/// that has gone quiet rather than merely find its series missing.
///
/// ´claim:metrics:an-entity-scoped-sample-carries-exactly-the-one-label-naming-its-entity´
/// ´test:crate:full-report-per-sentinel-labelled´
#[test]
fn full_report_per_sentinel_labelled() {
    let report = test_system_report(2, 0);
    let samples = full_report_to_samples(&report);

    let sentinel_reporting: Vec<_> = samples.iter().filter(|s| s.name == "assayer_sentinel_reporting").collect();
    assert_eq!(sentinel_reporting.len(), 2);

    for s in &sentinel_reporting {
        assert_eq!(s.labels.len(), 1);
        assert_eq!(s.labels[0].0, "sentinel");
    }

    // Report age is mapped.
    assert_eq!(
        samples
            .iter()
            .filter(|s| s.name == "assayer_sentinel_report_age_seconds")
            .count(),
        2
    );

    // Informativeness is mapped.
    assert_eq!(
        samples
            .iter()
            .filter(|s| s.name == "assayer_sentinel_informativeness")
            .count(),
        2
    );
}

/// Weight mass is exported as the measurement it is: a Sentinel whose mean
/// absolute operational slot weight was measured arrives with that value
/// untouched, and an unmeasured Sentinel arrives as NaN rather than as a
/// reassuring zero. Zero is a reading — the model carrying no weight there —
/// and it is a different fact from having taken no reading at all, so a gauge
/// that flattened the two would put a figure on the surface for a Sentinel
/// nothing had been measured on (´def:monitoring:encoding-effectiveness´).
///
/// ´claim:metrics:informativeness-exports-the-measured-value-and-absence-as-nan-never-a-constant´
/// ´test:crate:informativeness-export-carries-measurement-and-absence´
#[test]
fn informativeness_export_carries_measurement_and_absence() {
    let report = test_system_report(2, 0);
    let samples = full_report_to_samples(&report);

    let informativeness: Vec<_> = samples
        .iter()
        .filter(|s| s.name == "assayer_sentinel_informativeness")
        .collect();
    assert_eq!(informativeness.len(), 2);
    let measured = informativeness
        .iter()
        .find(|s| s.labels[0].1 == "0")
        .expect("sentinel 0 emitted");
    assert!(
        (measured.value - 0.25).abs() < 1e-12,
        "measured value arrives, got {}",
        measured.value
    );
    let unmeasured = informativeness
        .iter()
        .find(|s| s.labels[0].1 == "1")
        .expect("sentinel 1 emitted");
    assert!(
        unmeasured.value.is_nan(),
        "unmeasured arrives as NaN, got {}",
        unmeasured.value
    );
}

/// The per-dimension reading is exported under the same discipline as the
/// per-Sentinel one: a dimension whose mean absolute operational block weight
/// was measured arrives with that value untouched, and an unmeasured dimension
/// arrives as NaN. Zero is a reading here too, and absence is not it, so a
/// gauge that reported absence as zero would put a figure on the surface for a
/// dimension nothing had been measured on
/// (´def:monitoring:dimension-informativeness´).
///
/// ´claim:metrics:dimension-informativeness-exports-the-measured-value-and-absence-as-nan´
/// ´test:crate:dimension-informativeness-export-carries-measurement-and-absence´
#[test]
fn dimension_informativeness_export_carries_measurement_and_absence() {
    let report = test_system_report(0, 2);
    let samples = full_report_to_samples(&report);

    let informativeness: Vec<_> = samples
        .iter()
        .filter(|s| s.name == "assayer_identity_informativeness")
        .collect();
    assert_eq!(informativeness.len(), 2);
    let measured = informativeness
        .iter()
        .find(|s| s.labels[0].1 == "0")
        .expect("dimension 0 emitted");
    assert!(
        (measured.value - 0.5).abs() < 1e-12,
        "measured value arrives, got {}",
        measured.value
    );
    let unmeasured = informativeness
        .iter()
        .find(|s| s.labels[0].1 == "1")
        .expect("dimension 1 emitted");
    assert!(
        unmeasured.value.is_nan(),
        "unmeasured arrives as NaN, got {}",
        unmeasured.value
    );
}

/// The identity-health table's two remaining rows export under the discipline
/// the block reading already keeps: a measured figure arrives untouched and an
/// unmeasured one arrives as NaN. Zero is a real reading for both — cells that
/// carry no weight, and traffic that missed the competitive set entirely — so
/// a gauge that reported absence as zero would put a figure on the surface for
/// a dimension nothing had been measured on, and would put it there in exactly
/// the range an operator is watching for trouble
/// (´tab:monitoring:dimension-health´).
///
/// ´claim:metrics:the-per-dimension-coverage-share-and-cell-weight-mass-export-their-measured-values-and-absence-as-nan´
/// ´test:crate:dimension-coverage-and-cell-weight-export-carry-measurement-and-absence´
#[test]
fn dimension_coverage_and_cell_weight_export_carry_measurement_and_absence() {
    let report = test_system_report(0, 2);
    let samples = full_report_to_samples(&report);

    for (name, measured) in [
        ("assayer_identity_coverage_fraction", 0.25_f64),
        ("assayer_identity_cell_weight_mass", 0.75_f64),
    ] {
        let emitted: Vec<_> = samples.iter().filter(|s| s.name == name).collect();
        assert_eq!(emitted.len(), 2, "{name} is emitted once per registered dimension");
        let present = emitted
            .iter()
            .find(|s| s.labels[0].1 == "0")
            .unwrap_or_else(|| panic!("dimension 0 emits {name}"));
        assert!(
            (present.value - measured).abs() < 1e-12,
            "{name} measured value arrives, got {}",
            present.value
        );
        let absent = emitted
            .iter()
            .find(|s| s.labels[0].1 == "1")
            .unwrap_or_else(|| panic!("dimension 1 emits {name}"));
        assert!(absent.value.is_nan(), "{name} absence arrives as NaN, got {}", absent.value);
    }
}

/// Identity dimensions follow the same discipline as Sentinels: each
/// dimension's cell count and dropped-observation tally arrive under a single
/// label naming that dimension. Because the labelling scheme is uniform across
/// entity kinds, a dashboard written for one kind of entity transfers to
/// another by changing the label name.
///
/// (´claim:metrics:an-entity-scoped-sample-carries-exactly-the-one-label-naming-its-entity´)
/// ´test:crate:full-report-per-dimension-labelled´
#[test]
fn full_report_per_dimension_labelled() {
    let report = test_system_report(0, 3);
    let samples = full_report_to_samples(&report);

    let cell_count: Vec<_> = samples.iter().filter(|s| s.name == "assayer_identity_cell_count").collect();
    assert_eq!(cell_count.len(), 3);

    for s in &cell_count {
        assert_eq!(s.labels.len(), 1);
        assert_eq!(s.labels[0].0, "dimension");
    }

    // Observations dropped is mapped.
    assert_eq!(
        samples
            .iter()
            .filter(|s| s.name == "assayer_identity_observations_dropped_total")
            .count(),
        3
    );
}

/// Challenge metrics are declared in the catalogue but marked as the host's to
/// emit, and no core mapper produces one. The core has no view of what
/// happened after a decision was returned, so it could only invent these
/// values; declaring them anyway means the host that does know publishes them
/// under names that match the rest of the surface instead of inventing its own.
///
/// ´claim:metrics:quantities-only-the-host-can-observe-are-declared-in-the-catalogue-but-emitted-by-the-host´
/// ´test:crate:challenge-metrics-are-host-surface´
#[test]
fn challenge_metrics_are_host_surface() {
    let report = test_system_report_full(0, 0, 2);
    let samples = full_report_to_samples(&report);

    assert!(!samples.iter().any(|s| s.name == "assayer_challenge_q_hat"));
    assert!(!samples.iter().any(|s| s.name == "assayer_challenge_sufficient_evidence"));
    assert!(
        METRICS_CATALOG
            .iter()
            .filter(|definition| definition.name.starts_with("assayer_challenge_"))
            .all(|definition| definition.surface == MetricSurface::Host)
    );
}

/// Pending-buffer occupancy is exported as a fraction of capacity, computed at
/// export time from the two raw figures. A fraction is comparable across
/// deployments and directly alertable — the same threshold means the same
/// thing whatever the configured capacity — whereas a raw occupancy count only
/// becomes meaningful once the reader also knows how large the buffer is.
///
/// ´claim:metrics:buffer-occupancy-is-exported-as-a-fraction-of-capacity-so-one-threshold-serves-every-deployment´
/// ´test:crate:full-report-buffer-utilisation´
#[test]
fn full_report_buffer_utilisation() {
    let report = test_system_report(0, 0);
    let samples = full_report_to_samples(&report);

    let buf = samples
        .iter()
        .find(|s| s.name == "assayer_pending_buffer_utilisation")
        .unwrap();
    // 500 / 1_000_000 = 0.0005
    assert_near(buf.value, 0.0005, DEFAULT_TOLERANCES.default, "buffer utilisation");
}

/// The two distinct priors carried by the published snapshot remain distinct
/// on the cheap metric surface: neither is substituted for the other, rounded,
/// or replaced by the cold-start default.
///
/// ´claim:metrics:the-global-and-eligible-positive-class-priors-reach-distinct-summary-samples´
/// ´test:crate:positive-class-priors-reach-summary-samples´
#[test]
fn positive_class_priors_reach_summary_samples() {
    let mut summary = default_health_summary();
    summary.p_positive_global = 0.29;
    summary.p_positive_eligible = 0.73;

    let samples = health_summary_to_samples(&summary);
    let global = samples
        .iter()
        .find(|sample| sample.name == "assayer_positive_class_prior_global")
        .expect("global prior emitted");
    let eligible = samples
        .iter()
        .find(|sample| sample.name == "assayer_positive_class_prior_eligible")
        .expect("eligible prior emitted");

    assert_eq!(global.value, 0.29);
    assert_eq!(eligible.value, 0.73);
}

/// The monotone count of emitted drift-reset events reaches the summary sample
/// unchanged under its counter name.
///
/// ´claim:metrics:the-drift-reset-event-count-reaches-the-summary-counter-unchanged´
/// ´test:crate:drift-reset-counter-reaches-summary-samples´
#[test]
fn drift_reset_counter_reaches_summary_samples() {
    let mut summary = default_health_summary();
    summary.drift_resets = 6;

    let samples = health_summary_to_samples(&summary);
    let resets = samples
        .iter()
        .find(|sample| sample.name == "assayer_drift_resets_total")
        .expect("drift-reset counter emitted");
    assert_eq!(resets.value, 6.0);
}

/// Signal-cache hits, misses, size, capacity and evictions survive report
/// assembly as raw statistics, and the mapper derives hit rate and utilisation
/// from those live values.
///
/// ´claim:metrics:signal-cache-derived-metrics-read-the-raw-statistics-carried-by-the-full-report´
/// ´test:crate:full-report-signal-cache-metrics-carry-live-statistics´
#[test]
fn full_report_signal_cache_metrics_carry_live_statistics() {
    let mut report = test_system_report(0, 0);
    report.signal_cache = SignalCacheHealth {
        hits: 9,
        misses: 3,
        evictions: 4,
        capacity: 20,
        size: 5,
    };

    let samples = full_report_to_samples(&report);
    let find = |name: &str| {
        samples
            .iter()
            .find(|sample| sample.name == name)
            .expect("signal-cache metric emitted")
    };

    assert_eq!(find("assayer_signal_cache_evictions_total").value, 4.0);
    assert_eq!(find("assayer_signal_cache_hit_rate").value, 0.75);
    assert_eq!(find("assayer_signal_cache_utilisation").value, 0.25);
}

/// The fleet-wide coverage sample carries the fraction assembled from
/// registered Sentinels with live reports, including the defined zero for an
/// empty registry.
///
/// ´claim:metrics:sentinel-coverage-exports-the-reporting-share-of-the-registered-fleet´
/// ´test:crate:full-report-sentinel-coverage-carries-reporting-fraction´
#[test]
fn full_report_sentinel_coverage_carries_reporting_fraction() {
    let populated = full_report_to_samples(&test_system_report(3, 0));
    let populated_coverage = populated
        .iter()
        .find(|sample| sample.name == "assayer_sentinel_coverage")
        .expect("coverage emitted");
    assert_near(
        populated_coverage.value,
        2.0 / 3.0,
        DEFAULT_TOLERANCES.default,
        "two of three Sentinels reporting",
    );

    let empty = full_report_to_samples(&test_system_report(0, 0));
    let empty_coverage = empty
        .iter()
        .find(|sample| sample.name == "assayer_sentinel_coverage")
        .expect("empty-fleet coverage emitted");
    assert_eq!(empty_coverage.value, 0.0);
}

/// The number of concordance calibrations the tracker has completed reaches
/// the export as the tracker recorded it. Concordance thresholds are recomputed
/// periodically from the observed distribution, and this count is how an
/// operator confirms that recalibration is actually happening rather than
/// silently stalled behind a window that never fills.
///
/// ´claim:metrics:the-concordance-calibration-count-reaches-the-export-as-the-tracker-recorded-it´
/// ´test:crate:full-report-concordance-calibrations´
#[test]
fn full_report_concordance_calibrations() {
    let report = test_system_report(0, 0);
    let samples = full_report_to_samples(&report);

    let conc = samples
        .iter()
        .find(|s| s.name == "assayer_concordance_calibrations_total")
        .unwrap();
    assert_eq!(conc.value, 7.0);
}

/// The largest correction-loss fraction accumulated by marginalisation reaches
/// the metrics surface under the Schur namespace and with its value unchanged.
/// It is the dimensionless approximation figure a deployment can compare
/// across the model's life; leaving it only in the health report would make a
/// scraper blind to the trend it exists to expose.
///
/// ´claim:metrics:the-marginalisation-loss-fraction-is-exported-under-the-schur-metric-surface´
/// ´test:crate:full-report-marginalisation-loss-fraction´
#[test]
fn full_report_marginalisation_loss_fraction() {
    let mut report = test_system_report(0, 0);
    report.marginalisation.worst_correction_loss_fraction = 0.375;
    let samples = full_report_to_samples(&report);

    let loss = samples
        .iter()
        .find(|sample| sample.name == "assayer_schur_worst_correction_loss_fraction")
        .expect("marginalisation loss fraction emitted");
    assert_near(loss.value, 0.375, DEFAULT_TOLERANCES.default, "correction loss fraction");
}

/// A metric name has exactly one emission site even when both mappers have
/// access to the underlying state: the cascade-terminus flag comes from the
/// summary mapper, and the full-report mapper stays silent about it although
/// its own report carries the same field. Two mappers emitting one name would
/// put two samples of the same series in a single scrape, and which one won
/// would depend on the collector rather than on the system.
///
/// ´claim:metrics:each-metric-name-has-a-single-emission-site-even-when-both-mappers-can-see-the-state´
/// ´test:crate:cascade-terminus-emitted-from-summary-only´
#[test]
fn cascade_terminus_emitted_from_summary_only() {
    // A deviation fix: `assayer_cascade_terminus_active` has a
    // single emission site (summary mapper), sourced from
    // `HealthSummary.any_cascade_terminus`.  The full-report mapper
    // does NOT repeat it.
    let mut summary = default_health_summary();
    summary.any_cascade_terminus = true;
    let summary_samples = health_summary_to_samples(&summary);
    let ct = summary_samples
        .iter()
        .find(|s| s.name == "assayer_cascade_terminus_active")
        .unwrap();
    assert_eq!(ct.value, 1.0);

    let mut report = test_system_report(0, 0);
    report.cascade_terminus_active = true;
    let report_samples = full_report_to_samples(&report);
    assert!(
        !report_samples.iter().any(|s| s.name == "assayer_cascade_terminus_active"),
        "full-report mapper must not duplicate cascade_terminus_active"
    );
}

/// The dropped-health-event tally is likewise the summary mapper's alone,
/// with the full-report mapper declining to repeat it despite holding the same
/// number. This one is a counter, so duplication would be worse than an
/// ambiguous gauge: a collector that summed both sites would report backpressure
/// at twice its true rate.
///
/// (´claim:metrics:each-metric-name-has-a-single-emission-site-even-when-both-mappers-can-see-the-state´)
/// ´test:crate:health-events-dropped-emitted-from-summary-only´
#[test]
fn health_events_dropped_emitted_from_summary_only() {
    // A deviation fix: single emission site for
    // `assayer_health_events_dropped_total` (summary mapper).
    let mut summary = default_health_summary();
    summary.health_events_dropped = 42;
    let summary_samples = health_summary_to_samples(&summary);
    let dropped = summary_samples
        .iter()
        .find(|s| s.name == "assayer_health_events_dropped_total")
        .unwrap();
    assert_eq!(dropped.value, 42.0);

    let mut report = test_system_report(0, 0);
    report.health_events_dropped = 42;
    let report_samples = full_report_to_samples(&report);
    assert!(
        !report_samples.iter().any(|s| s.name == "assayer_health_events_dropped_total"),
        "full-report mapper must not duplicate health_events_dropped_total"
    );
}

/// A substantial deployment — twenty Sentinels across five identity dimensions
/// — still exports well under a couple of thousand samples in one scrape.
/// Time-series cardinality is the resource a metrics backend actually runs out
/// of, so the export staying inside a stated budget at realistic scale is what
/// makes it safe to enable in production rather than only in a demo.
///
/// ´claim:metrics:a-realistically-sized-deployment-exports-well-inside-the-stated-cardinality-budget´
/// ´test:crate:full-report-cardinality-bounded´
#[test]
fn full_report_cardinality_bounded() {
    // At 20 Sentinels × 5 dimensions × 3 channels, total samples must be < 2,500.
    let report = test_system_report_full(20, 5, 3);
    let samples = full_report_to_samples(&report);
    assert!(samples.len() < 2_500, "cardinality too high: {} samples", samples.len());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Completeness Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The surface declaration is binding on the mappers: nothing marked as the
/// host's is emitted by either of them, and there is genuinely at least one
/// such entry for the rule to bite on. Were the core to emit a host-surface
/// name as well, the host's own value and the core's would collide in the same
/// series, and the field would stop meaning what the catalogue says it means.
///
/// ´claim:metrics:the-surface-declaration-is-binding-so-no-mapper-emits-a-name-reserved-for-the-host´
/// ´test:crate:host-surface-entries-not-emitted´
#[test]
fn host_surface_entries_not_emitted() {
    // Host-surface catalog entries (e.g., `assayer_reckoning_duration_seconds`)
    // are declared for naming consistency only — the host provides the values.
    // Neither mapper emits them.
    let summary = default_health_summary();
    let report = test_system_report(0, 0);
    let all_emitted: HashSet<&str> = health_summary_to_samples(&summary)
        .iter()
        .chain(full_report_to_samples(&report).iter())
        .map(|s| s.name)
        .collect();

    for def in METRICS_CATALOG {
        if def.surface == MetricSurface::Host {
            assert!(
                !all_emitted.contains(def.name),
                "host-surface entry {:?} must not be emitted by a mapper",
                def.name
            );
        }
    }

    // Sanity check: at least one host-surface entry exists.
    assert!(
        METRICS_CATALOG.iter().any(|d| d.surface == MetricSurface::Host),
        "expected at least one host-surface catalog entry (e.g., assayer_reckoning_duration_seconds)"
    );
}

/// The catalogue cannot promise more than the code delivers: given a summary
/// and a report populated richly enough to reach every branch, every entry not
/// reserved for the host is produced by one mapper or the other. Together with
/// the reverse direction this closes the contract in both ways — a name in the
/// catalogue is a name an operator can build a dashboard on, and a declared
/// metric that nothing emits is a broken promise the build refuses to ship.
///
/// ´claim:metrics:every-catalogue-entry-not-reserved-for-the-host-is-actually-produced-by-a-mapper´
/// ´test:crate:every-catalog-entry-is-emitted´
#[test]
fn every_catalog_entry_is_emitted() {
    // Build a summary and full report with enough entities to trigger
    // per-entity metrics, then verify every catalog name appears.
    let summary = HealthSummary {
        convergence_stage: CompositeConvergenceStage::SisterConverging,
        platt_state: PlattConvergenceState::Converging,
        last_delta_cal: 0.05,
        platt_refits_completed: 12,
        total_labels: 500,
        eligible_labels: 450,
        total_assessments: 10_000,
        degradation: AssessmentDegradationSnapshot {
            total_degraded: 20,
            signals_sanitised: 5,
            signals_shape_mismatched: 4,
            signals_unknown: 6,
            sentinel_slots_zeroed: 2,
            features_sanitised: 3,
            model_fallbacks: 1,
            batch_init_observations_skipped: 0,
            zero_sentinel_assessments: 7,
        },
        blend_statistics: BlendStatistics {
            mean: 0.15,
            p10: 0.02,
            p50: 0.12,
            p90: 0.30,
            p99: 0.45,
            fraction_anchor_dominated: 0.08,
            w_infinity_estimate: 0.05,
            steady_state: true,
            window_size: 500,
        },
        feature_stable_outcome_drift: true,
        quantile_error_concentration: Some(1.8),
        cp5_reverts: 1,
        cp6_reverts: 0,
        auc_aggregate: Some(0.82),
        auc_recent: Some(0.79),
        max_diagonal_ratio: 42.0,
        max_floor_share: 0.25,
        max_sync_error: 0.001,
        max_prior_induced_sync_error: 0.0,
        max_sync_error_residual: 0.001,
        any_cascade_terminus: false,
        label_path_stopped: false,
        platt_kappa_sister: 1.5,
        platt_kappa_anchor: 1.2,
        p_positive_global: 0.37,
        p_positive_eligible: 0.61,
        health_events_dropped: 0,
        drift_resets: 4,
    };

    let mut report = test_system_report_full(2, 2, 1);
    // Ensure discrimination is present for auc_recent.
    report.discrimination = Some(crate::health::DiscriminationMetrics {
        auc_recent: Some(0.75),
        ..Default::default()
    });

    let summary_samples = health_summary_to_samples(&summary);
    let report_samples = full_report_to_samples(&report);

    let mut emitted: HashSet<&str> = HashSet::new();
    for s in &summary_samples {
        emitted.insert(s.name);
    }
    for s in &report_samples {
        emitted.insert(s.name);
    }

    let mut missing = Vec::new();
    for def in METRICS_CATALOG {
        // Host-surface entries are declared for naming consistency only —
        // no mapper emits them (´dec:metrics:catalogue-membership´).
        if def.surface == MetricSurface::Host {
            continue;
        }
        if !emitted.contains(def.name) {
            missing.push(def.name);
        }
    }

    assert!(missing.is_empty(), "catalog metrics not emitted by any mapper: {missing:?}");
}

/// The reverse direction closes the contract: every sample either mapper
/// emits is catalogued, under its declared type and its declared label keys.
/// The catalogue decides it is a compile-time constant precisely so it
/// cannot drift from what the code emits, but nothing mechanical held the
/// emitting half of that promise — a mapper could emit a name no host
/// registered, and the sample would arrive unclassifiable. Walking every
/// emitted sample back into the catalogue is the half of the correspondence
/// the forward test cannot see (´dec:metrics:compile-time-catalogue´).
///
/// ´claim:metrics:every-emitted-sample-is-catalogued-under-its-declared-type-and-label-keys´
/// ´test:crate:every-emitted-sample-is-catalogued´
#[test]
fn every_emitted_sample_is_catalogued() {
    use std::collections::HashMap as StdHashMap;

    let catalogue: StdHashMap<&str, &crate::metrics::MetricDefinition> =
        METRICS_CATALOG.iter().map(|def| (def.name, def)).collect();

    // The same richly populated fixtures the forward direction walks.
    let summary = default_health_summary();
    let report = test_system_report_full(2, 2, 1);

    let mut samples = health_summary_to_samples(&summary);
    samples.extend(full_report_to_samples(&report));

    for sample in &samples {
        let def = catalogue
            .get(sample.name)
            .unwrap_or_else(|| panic!("emitted sample {:?} is not in the catalogue", sample.name));
        assert_eq!(
            sample.metric_type, def.metric_type,
            "{:?} emitted as a different kind than the catalogue declares",
            sample.name
        );
        assert_ne!(
            def.surface,
            MetricSurface::Host,
            "{:?} is reserved for the host and must not be emitted",
            sample.name
        );
        let emitted_keys: Vec<&str> = sample.labels.iter().map(|(key, _)| *key).collect();
        assert_eq!(
            emitted_keys, def.labels,
            "{:?} emitted under different label keys than the catalogue declares",
            sample.name
        );
    }
}

/// The repaired count reaches the export under the name that says what it now counts, carrying each model's own reading. The metric was `assayer_cholesky_regularised_total` and counted the retired factorisation shift; with the shift gone it would have exported a constant zero for every model for ever, which is a measurement promised and not delivered. Renaming it with the quantity is what keeps the promise, and this reads the two models' distinct values through the mapper to show the rename carried the wiring rather than only the string.
///
/// ´claim:metrics:the-repaired-count-reaches-the-export-under-the-name-of-what-it-counts´
/// ´test:crate:full-report-exports-the-repaired-count´
#[test]
fn full_report_exports_the_repaired_count() {
    let report = test_system_report(0, 0);
    let samples = full_report_to_samples(&report);

    let read = |model: &str| {
        samples
            .iter()
            .find(|s| {
                s.name == "assayer_precision_floored_rebuilds_total" && s.labels.iter().any(|(k, v)| *k == "model" && v == model)
            })
            .unwrap_or_else(|| panic!("the repaired count is exported for {model}"))
            .value
    };

    assert!(
        (read(&model_label(&ModelId::Operational)) - 3.0).abs() < f64::EPSILON,
        "the operational model's three repairs reach the export",
    );
    assert!(
        (read(&model_label(&ModelId::Sister)) - 1.0).abs() < f64::EPSILON,
        "and the sister's one, so the reading is per model rather than a constant",
    );
    assert!(
        !samples.iter().any(|s| s.name == "assayer_cholesky_regularised_total"),
        "the name of the retired device is gone rather than exported beside its replacement",
    );
}
