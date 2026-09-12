// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`health_summary_latency`] | audit | The health summary is cheap enough to poll continuously: ten thousand calls fit inside ten seconds, putting the per-call cost around a millisecond at worst. It exists precisely to be the operation a monitoring loop calls on every scrape, so it is the summary rather than the full report that must stay on a fast path. |
//! | [`full_health_report_latency`] | audit | The full report costs far more per call than the summary and is budgeted accordingly — a hundred calls rather than ten thousand — but it stays an operation a human or an alert can invoke on demand without hesitation. It walks per-Sentinel, per-dimension and per-model state to assemble itself, so its cost is expected to be assembly-bound; what the budget forbids is that cost growing to the point where diagnosing a live system becomes something one must schedule. |
//! | [`label_boundary_latency`] | audit | The whole operating cycle — assess, derive, then label the assessment just made — sustains ten thousand round trips inside a minute, a few milliseconds apiece. This is the loop a host actually lives in, and it spans both boundaries at once: the read path into the published model and the write path handing an outcome back for learning. A cycle that could not keep up would force a host to label only a sample of its traffic. |
//! | [`label_model_update_profile`] | audit | A realistic dense label workload attributes publication latency between the sequential full-dimension model updates and the remainder of the pipeline. It uses the specification's representative width and several outcome axes, excludes periodic factorisation, warms both paths, then emits exact elapsed times and the resulting component share. This is evidence for the parallelism deferral's trigger rather than a pass/fail performance budget. |
//! | [`builder_construction_latency`] | audit | Building an engine and shutting it down again is bounded work, and the pair stays cheap enough to repeat a hundred times over. Construction allocates models and starts the owning thread while shutdown must join it, so the budget covers both halves together — a shutdown that failed to converge promptly would show up here as accumulated time rather than as a hang somebody eventually notices. |
//! | [`preseed_throughput`] | audit | Pre-seeding accepts a thousand historical entries in one call, processes every one of them, and finishes well inside its budget. Bulk import is how a new deployment starts from a body of known outcomes instead of from ignorance, and it is a startup operation someone waits on: both that the count comes back whole and that the wait stays short are what make seeding from history practical rather than nominal. |

//! Performance benchmark tests (`#[ignore]`).
//!
//! These are coarse-grained latency and throughput sanity checks. They
//! are ignored by default and intended for manual or CI-only runs.
//!
//! All scenarios are expressed through the [`crate::testing`] harness
//! ([`World`], [`LabelSpec`]) so they speak the same vocabulary as the
//! integration tests.
//!
//! Each budget below is deliberately generous: these are not measurements of
//! how fast an operation is, but floors under how slow it may become. A
//! failure here means an operation has changed cost by an order of magnitude,
//! which is a design regression rather than a tuning matter.
//!
//! # Cross-References
//!
//! - (´chap:spec:convergence-and-resources´) — the cost model every budget here
//!   is a floor under, tabulated per assessment, per label, and in memory

use std::collections::HashMap;
use std::sync::atomic::AtomicU64;

use crate::config::types::AssayerConfig;
use crate::health::{DegradationContext, PlattConvergenceTracker, create_event_channel};
use crate::ledger::OutcomeLedger;
use crate::model::bayesian::BayesianLinearModel;
use crate::model::update::{DEFAULT_IMPORTANCE_CEILING, DEFAULT_LAMBDA_FLOOR, DEFAULT_LEVERAGE_SAFETY_FACTOR, EPSILON_LEVERAGE};
use crate::owner::commands::{LabelContext, PendingAssessment, SequencedLabel};
use crate::owner::label_path::{CalibrationBuffer, HealthCounters, LabelPipelineConfig, process_label};
use crate::pending::{PendingRiskBasis, StoragePrecision, StoredFeatures};
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters};
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::{ModelConfig, WorkingAxisModel, WorkingCopy};
use crate::testing::{LabelSpec, PreSeedSpec, World};
use crate::types::{AssessmentId, EntityKey, OutcomeAxisId, PersistentTimestamp};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

/// Matches the original benchmark policy (Allow / Challenge / Block) so
/// that latency numbers remain comparable across revisions.
fn bench_policy() -> ChannelPolicy {
    ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    }
}

fn build_world() -> World {
    World::builder(AssayerConfig {
        instance_id: "benchmarks".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    })
    .channel("default", bench_policy())
    .seed(0xBEEF_CAFE)
    .build()
    .expect("benchmark world should build")
}

fn make_pre_seed_entry(world: &World, seed: u64, valence: f64) -> crate::api::PreSeedEntry {
    let channel = world.channel("default").expect("default channel declared");
    PreSeedSpec::new(channel, World::entity(&format!("seed-{seed}")))
        .valence(valence)
        .ground_truth()
        .build()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Health Summary Latency
// ═══════════════════════════════════════════════════════════════════════════════

/// The health summary is cheap enough to poll continuously: ten thousand calls
/// fit inside ten seconds, putting the per-call cost around a millisecond at
/// worst. It exists precisely to be the operation a monitoring loop calls on
/// every scrape, so it is the summary rather than the full report that must
/// stay on a fast path.
///
/// ´claim:audit:the-health-summary-is-cheap-enough-to-poll-on-every-monitoring-scrape´
/// ´test:crate:health-summary-latency´
#[test]
#[ignore = "performance benchmark"]
fn health_summary_latency() {
    let world = build_world();

    let start = std::time::Instant::now();
    for _ in 0..10_000 {
        let _ = world.assayer().health_summary();
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_millis() < 10_000,
        "10,000 health_summary calls took {elapsed:?} (budget: <10s)",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Full Health Report Latency
// ═══════════════════════════════════════════════════════════════════════════════

/// The full report costs far more per call than the summary and is budgeted
/// accordingly — a hundred calls rather than ten thousand — but it stays an
/// operation a human or an alert can invoke on demand without hesitation. It
/// walks per-Sentinel, per-dimension and per-model state to assemble itself,
/// so its cost is expected to be assembly-bound; what the budget forbids is
/// that cost growing to the point where diagnosing a live system becomes
/// something one must schedule.
///
/// ´claim:audit:the-full-health-report-stays-affordable-to-assemble-on-demand-despite-walking-every-entity´
/// ´test:crate:full-health-report-latency´
#[test]
#[ignore = "performance benchmark"]
fn full_health_report_latency() {
    let world = build_world();

    let start = std::time::Instant::now();
    for _ in 0..100 {
        drop(world.assayer().full_health_report());
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 5,
        "100 full_health_report calls took {elapsed:?} (budget: <5s)",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Boundary Latency
// ═══════════════════════════════════════════════════════════════════════════════

/// The whole operating cycle — assess, derive, then label the assessment just
/// made — sustains ten thousand round trips inside a minute, a few milliseconds
/// apiece. This is the loop a host actually lives in, and it spans both
/// boundaries at once: the read path into the published model and the write
/// path handing an outcome back for learning. A cycle that could not keep up
/// would force a host to label only a sample of its traffic.
///
/// ´claim:audit:the-full-assess-derive-label-cycle-sustains-thousands-of-round-trips-a-minute´
/// ´test:crate:label-boundary-latency´
#[test]
#[ignore = "performance benchmark"]
fn label_boundary_latency() {
    let world = build_world();

    let start = std::time::Instant::now();
    for i in 0_u64..10_000 {
        let entity = format!("e{i}");
        let reckoning = world
            .derive_for_request(world.request("default", &entity))
            .expect("assess/derive should succeed");
        drop(world.label(LabelSpec::adverse(reckoning.assessment.id).ground_truth().build()));
    }
    let elapsed = start.elapsed();

    assert!(
        elapsed.as_secs() < 60,
        "10,000 assess/derive+label calls took {elapsed:?} (budget: <60s)",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Model-Update Attribution
// ═══════════════════════════════════════════════════════════════════════════════

/// Published resource table's representative dense width with several
/// non-spatial outcome axes (´tab:registry:axis-scaling´).
const PROFILE_DIMENSION: usize = 646;
/// Signal positions needed to bring the fixed bias-plus-aggregate layout to
/// the representative dense width.
const PROFILE_SIGNAL_DIMENSION: usize = PROFILE_DIMENSION - 16;
/// Outcome-axis models in the representative workload.
const PROFILE_AXES: usize = 5;
/// Labels timed after warming both paths.
const PROFILE_LABELS: u64 = 128;
/// Warm-up labels excluded from the measurement.
const PROFILE_WARMUP: u64 = 8;
/// Keeps periodic factorisation out of the profile so the measured component
/// is the sequential rank-one update block named by the deferral.
const PROFILE_RECOMPUTE_INTERVAL: u32 = 10_000;

/// Complete owner-thread state for the instrumented label publication path.
struct LabelProfileEnv {
    working: WorkingCopy,
    shared: SharedState,
    ledger: OutcomeLedger,
    platt_tracker: PlattConvergenceTracker,
    calibration_buffer: CalibrationBuffer,
    config: LabelPipelineConfig,
    event_tx: crossbeam_channel::Sender<crate::health::HealthEvent>,
    _event_rx: crossbeam_channel::Receiver<crate::health::HealthEvent>,
    dropped_counter: AtomicU64,
    drift_reset_counter: crate::health::DriftResetCounter,
    identity_dimensions:
        std::sync::RwLock<HashMap<crate::types::DimensionId, std::sync::Arc<crate::identity::IdentityDimensionInfra>>>,
    health_counters: HealthCounters,
}

impl LabelProfileEnv {
    /// Builds the representative model population before either timed region.
    fn new() -> Self {
        let signal_classes = vec![crate::feature::standardisation::FeatureClass::ZScore; PROFILE_SIGNAL_DIMENSION];
        let mut working = WorkingCopy::cold_start_from_schema(
            PROFILE_SIGNAL_DIMENSION,
            Vec::new(),
            &signal_classes,
            &ModelConfig::default(),
            PROFILE_RECOMPUTE_INTERVAL,
            100,
        );
        for axis in 0..PROFILE_AXES {
            working.outcome_models.insert(
                OutcomeAxisId(axis as u32),
                WorkingAxisModel {
                    name: format!("profile-axis-{axis}"),
                    description: "profiling-only representative outcome axis".to_owned(),
                    model: BayesianLinearModel::new(PROFILE_DIMENSION, 0.1, PROFILE_RECOMPUTE_INTERVAL),
                    kappa_a: 1.0,
                    gamma: 0.9998,
                    spatial: false,
                    eligibility: crate::types::OutcomeEligibility::default(),
                },
            );
        }
        let snapshot = working.to_snapshot(0);
        let shared = SharedState::new(snapshot);
        let (event_tx, event_rx) = create_event_channel(crate::health::DEFAULT_EVENT_CHANNEL_CAPACITY);
        let config = LabelPipelineConfig {
            n_recompute: PROFILE_RECOMPUTE_INTERVAL,
            kappa_growth_factor: f64::MAX,
            ..LabelPipelineConfig::default()
        };

        Self {
            working,
            shared,
            ledger: OutcomeLedger::new(),
            platt_tracker: PlattConvergenceTracker::new(),
            calibration_buffer: CalibrationBuffer::default(),
            config,
            event_tx,
            _event_rx: event_rx,
            dropped_counter: AtomicU64::new(0),
            drift_reset_counter: crate::health::DriftResetCounter::new(),
            identity_dimensions: std::sync::RwLock::new(HashMap::new()),
            health_counters: HealthCounters::default(),
        }
    }

    /// Processes one eligible label through publication.
    fn process(&mut self, label: &SequencedLabel, version: u64) {
        process_label(
            &mut self.working,
            label,
            &self.shared,
            &self.ledger,
            &mut self.platt_tracker,
            &mut self.calibration_buffer,
            &self.identity_dimensions,
            &HashMap::new(),
            [0.0; crate::types::SCORING_AXIS_COUNT],
            &self.config,
            &crate::testing::SystemClock,
            &self.event_tx,
            &self.dropped_counter,
            &self.drift_reset_counter,
            version,
            version,
            &mut self.health_counters,
        );
    }
}

/// One label whose reconstructed vector exercises the representative width and
/// whose outcome payload updates every representative axis.
fn profile_label() -> SequencedLabel {
    let assessment_id = AssessmentId(1);
    let values: Vec<f64> = (0..PROFILE_SIGNAL_DIMENSION)
        .map(|index| if index % 2 == 0 { 0.25 } else { -0.25 })
        .collect();
    let pending = PendingAssessment {
        spatial_axis_ids: Vec::new(),
        id: assessment_id,
        timestamp: std::time::Instant::now(),
        persistent_timestamp: PersistentTimestamp::now(),
        entity: EntityKey::new(b"label-profile".to_vec()),
        sentinel_extractions: HashMap::new(),
        identity_coordinates: HashMap::new(),
        identity_active_cells: HashMap::new(),
        active_sentinels: Vec::new(),
        reporting_sentinels: Vec::new(),
        entity_base_features: HashMap::new(),
        entity_axis_features: HashMap::new(),
        signal_features: StoredFeatures::store(&values, StoragePrecision::Double),
        risk_basis: PendingRiskBasis::default(),
        outcome_predictions: HashMap::new(),
        degradation: DegradationContext::default(),
        report_origin: None,
    };
    let mut spec = LabelSpec::new(assessment_id)
        .valence(1.0)
        .action(Action::Allow)
        .ground_truth();
    for axis in 0..PROFILE_AXES {
        spec = spec.outcome(OutcomeAxisId(axis as u32), 0.75);
    }
    SequencedLabel {
        seq: Some(1),
        label: spec.build(),
        context: LabelContext::Full(Box::new(pending)),
        arrived_at: None,
    }
}

/// Applies the same dense rank-one update block without reconstruction,
/// calibration, diagnostics, snapshot construction, or publication.
fn profile_sequential_model_updates(
    full_models: &mut [BayesianLinearModel],
    anchor: &mut BayesianLinearModel,
    phi: &faer::Col<f64>,
    anchor_phi: &faer::Col<f64>,
) {
    for model in full_models {
        let (v, h) = model.covariance().quadratic_form_with_product(phi);
        let weight = crate::model::update::compute_effective_weight(
            1.0,
            DEFAULT_IMPORTANCE_CEILING,
            DEFAULT_LEVERAGE_SAFETY_FACTOR,
            h,
            EPSILON_LEVERAGE,
        );
        model.apply_leverage_bounded_update(phi, &v, h, weight, 1.0, 0.9998, DEFAULT_LAMBDA_FLOOR);
    }
    let (v, h) = anchor.covariance().quadratic_form_with_product(anchor_phi);
    let weight = crate::model::update::compute_effective_weight(
        1.0,
        DEFAULT_IMPORTANCE_CEILING,
        DEFAULT_LEVERAGE_SAFETY_FACTOR,
        h,
        EPSILON_LEVERAGE,
    );
    anchor.apply_leverage_bounded_update(anchor_phi, &v, h, weight, 1.0, 0.9998, DEFAULT_LAMBDA_FLOOR);
}

/// A realistic dense label workload attributes publication latency between the
/// sequential full-dimension model updates and the remainder of the pipeline.
/// It uses the specification's representative width and several outcome axes,
/// excludes periodic factorisation, warms both paths, then emits exact elapsed
/// times and the resulting component share. This is evidence for the
/// parallelism deferral's trigger rather than a pass/fail performance budget.
///
/// ´claim:audit:the-label-profile-attributes-publication-latency-to-the-sequential-model-update-block´
/// ´test:crate:label-model-update-profile´
#[test]
#[ignore = "profiling evidence"]
fn label_model_update_profile() {
    let label = profile_label();
    let mut publication = LabelProfileEnv::new();

    let phi_values: Vec<f64> = (0..PROFILE_DIMENSION)
        .map(|index| if index % 2 == 0 { 0.25 } else { -0.25 })
        .collect();
    let phi = crate::linalg::convert::vec_to_col(&phi_values);
    let anchor_phi = crate::linalg::convert::vec_to_col(&phi_values[..15]);
    let mut full_models: Vec<BayesianLinearModel> = (0..(2 + PROFILE_AXES))
        .map(|_| BayesianLinearModel::new(PROFILE_DIMENSION, 0.1, PROFILE_RECOMPUTE_INTERVAL))
        .collect();
    let mut anchor = BayesianLinearModel::new(15, 0.1, PROFILE_RECOMPUTE_INTERVAL);

    for version in 1..=PROFILE_WARMUP {
        publication.process(&label, version);
        profile_sequential_model_updates(&mut full_models, &mut anchor, &phi, &anchor_phi);
    }
    assert_eq!(
        publication.working.operational.labels_since_recompute(),
        PROFILE_WARMUP as u32
    );
    assert_eq!(publication.working.sister.labels_since_recompute(), PROFILE_WARMUP as u32);
    assert_eq!(publication.working.anchor.labels_since_recompute(), PROFILE_WARMUP as u32);
    assert!(
        publication
            .working
            .outcome_models
            .values()
            .all(|axis| axis.model.labels_since_recompute() == PROFILE_WARMUP as u32),
        "every representative outcome model must be live in the measured path",
    );

    let update_start = std::time::Instant::now();
    for _ in 0..PROFILE_LABELS {
        profile_sequential_model_updates(&mut full_models, &mut anchor, &phi, &anchor_phi);
    }
    let update_elapsed = update_start.elapsed();

    let publication_start = std::time::Instant::now();
    for offset in 1..=PROFILE_LABELS {
        publication.process(&label, PROFILE_WARMUP + offset);
    }
    let publication_elapsed = publication_start.elapsed();

    let update_ns = update_elapsed.as_nanos();
    let publication_ns = publication_elapsed.as_nanos();
    let update_share = update_ns as f64 / publication_ns as f64;
    std::hint::black_box(
        full_models
            .iter()
            .map(BayesianLinearModel::labels_since_recompute)
            .sum::<u32>(),
    );
    println!(
        "label-profile dimension={PROFILE_DIMENSION} axes={PROFILE_AXES} labels={PROFILE_LABELS} \
         sequential_updates_ns={update_ns} publication_ns={publication_ns} update_share={update_share:.6}",
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Builder Construction Latency
// ═══════════════════════════════════════════════════════════════════════════════

/// Building an engine and shutting it down again is bounded work, and the pair
/// stays cheap enough to repeat a hundred times over. Construction allocates
/// models and starts the owning thread while shutdown must join it, so the
/// budget covers both halves together — a shutdown that failed to converge
/// promptly would show up here as accumulated time rather than as a hang
/// somebody eventually notices.
///
/// ´claim:audit:building-and-shutting-down-an-engine-is-bounded-work-that-can-be-repeated-freely´
/// ´test:crate:builder-construction-latency´
#[test]
#[ignore = "performance benchmark"]
fn builder_construction_latency() {
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let world = build_world();
        drop(world);
    }
    let elapsed = start.elapsed();

    assert!(elapsed.as_secs() < 30, "100 builds took {elapsed:?} (budget: <30s)");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-seed Throughput
// ═══════════════════════════════════════════════════════════════════════════════

/// Pre-seeding accepts a thousand historical entries in one call, processes
/// every one of them, and finishes well inside its budget. Bulk import is how
/// a new deployment starts from a body of known outcomes instead of from
/// ignorance, and it is a startup operation someone waits on: both that the
/// count comes back whole and that the wait stays short are what make seeding
/// from history practical rather than nominal.
///
/// ´claim:audit:bulk-pre-seeding-accepts-a-thousand-historical-entries-in-one-call-and-processes-every-one´
/// ´test:crate:preseed-throughput´
#[test]
#[ignore = "performance benchmark"]
fn preseed_throughput() {
    let world = build_world();

    let entries: Vec<_> = (0..1_000)
        .map(|i| make_pre_seed_entry(&world, i, if i % 5 == 0 { 1.0 } else { -1.0 }))
        .collect();

    let start = std::time::Instant::now();
    let result = world.assayer().pre_seed(&entries).unwrap();
    let elapsed = start.elapsed();

    assert_eq!(result.processed, 1000);
    assert!(
        elapsed.as_secs() < 30,
        "1,000 pre-seed entries took {elapsed:?} (budget: <30s)",
    );
}
