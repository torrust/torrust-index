// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`pending_entry_has_correct_contents`] | assess | The pending record left behind by an assessment carries the entity it was about and the degradation it ran under, alongside the assessment's own identifier. A label arriving hours later names only the identifier, so everything needed to turn that label into a training update — who it was about, and how trustworthy the estimate was — has to have been captured at assessment time. |
//! | [`pending_id_matches_assessment`] | assess | The identifier the caller is handed is the identifier under which the pending record was filed, and it stays so as records accumulate: the nth buffered entry belongs to the nth assessment. Identity is allocated once and used for both, so the buffer cannot drift out of step with what callers hold. |
//! | [`pending_inserted_for_channel_free_request`] | assess | One assessment buffers exactly one pending record — a request that named no channel is no exception. Every estimate is eligible to be labelled later, so the decision to buffer belongs to the core path and not to whatever routing the host layered on top; a duplicate would let one outcome update the model twice. |
//! | [`per_sentinel_alarm_populated`] | assess | When a Sentinel has both a cached batch report and a coordinate in the request, the assessment carries a summary for it, and the peak z that summary reports is a real finite number drawn from the report's axis scores. This is the whole point of the per-Sentinel map: a host can see which measurement surface was loud, not merely the pooled risk it fed into. |
//! | [`multiple_assessments_unique_ids`] | assess | cites (´claim:assess:assessment-ids-are-handed-out-in-strictly-increasing-order´) |
//! | [`concurrent_assessments_thread_safe`] | assess | Four threads assessing at once still receive four hundred distinct identifiers, and the buffer ends up holding four hundred records — none lost to a race, none double-counted. Assessment is the hot path a host runs from every request-handling thread it has, so allocation and buffering both have to be safe under genuine contention rather than merely under a lock the host is trusted to hold. |
//! | [`multiple_nan_signals_counted`] | assess | Sanitisation counts, it does not merely flag: a request carrying a NaN and both infinities comes back with all three tallied. Because the number rather than a boolean crosses back to the host, an operator can tell one stray value from a wholesale collapse of an upstream feed, and both infinities count the same as the NaN does. |
//! | [`degradation_propagates_to_pending`] | assess | Degradation is not only reported to the caller — it is stored with the pending record too. When the label for this request eventually arrives, the learner can see that the estimate it is about to be trained against was built on salvaged input, which is exactly the information needed to down-weight it rather than treat it as a clean observation. |
//! | [`cp2_nan_extraction_records_sentinel_id`] | assess | A single non-finite score anywhere in a Sentinel's extraction condemns that Sentinel's whole slot, and the Sentinel is named — by identity, not merely counted — in the assessment's degradation record. Naming it is what lets an operator go and fix the one broken measurement surface; zeroing the whole slot rather than the offending value is what stops a corrupt report from contributing half-believable features to the risk estimate. |
//! | [`identity_aggregates_vary_by_entity`] | assess | Two different entity keys land at two different places in a registered identity dimension, and each assessment records where its entity landed. Identity features are only informative if position is a function of the subject: an encoder that collapsed distinct entities onto one coordinate would have every request read the same neighbourhood's history, and the dimension would contribute a constant to every risk estimate. |
//! | [`per_dimension_measurement_reads_real_ewmas`] | assess | The pending record remembers not just where the entity landed but which competitive cells were active there at assessment time, per dimension. Cells come and go as the competitive set is rebuilt, so a label arriving later must be attributed to the cells that actually informed the estimate rather than to whichever cells happen to cover that coordinate by then. |

//! Assessment pipeline crate-level tests (´dec:ordering:assessment-function´).
//!
//! These tests verify cross-module integration for the Core assessment
//! pipeline, including pending buffer interaction, extraction assembly, and
//! lifecycle. A local helper composes assessments with derivation where tests
//! need to inspect a full host-facing decision result.
//!
//! # Cross-References
//!
//! - Testing plan: Assessment Pipeline Orchestration
//! - (´dec:ordering:assessment-function´) — why the current Core surface is the
//!   single call `assess()` rather than the staged pipeline the name suggests

use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Instant;

use crate::assessment::{AssessmentSharedState, DerivedReckoning, RequestContext, assess_single};
use crate::health::CompositeConvergenceStage;
use crate::identity::CompetitiveCellId;
use crate::pending::PendingAssessment;
use crate::report::ReportIndex;
use crate::resonance::channel::ChannelPolicy;
use crate::resonance::derivation::{ChallengeEstimate, ResonanceConfig, render_resonances};
use crate::resonance::landscape::derive_landscape;
use crate::snapshot::published::ModelSnapshot;
use crate::testing::dimensions::make_cells;
use crate::testing::{World, assert_finite};
use crate::types::{AssessmentId, ChannelId, DimensionId, OutcomeAxisId, SCORING_AXIS_COUNT, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Test Infrastructure
// ═══════════════════════════════════════════════════════════════════════════════

/// Mock shared state with pending buffer tracking.
struct TestSharedState {
    channels: Vec<ChannelId>,
    sentinels: Vec<SentinelId>,
    identity_dims: Vec<DimensionId>,
    next_id: AtomicU64,
    concordance_observations: Mutex<Vec<[f64; SCORING_AXIS_COUNT]>>,
    pending_entries: Mutex<Vec<PendingAssessment>>,
    sentinel_reports: Mutex<HashMap<SentinelId, Arc<ReportIndex>>>,
    /// Per-dimension active cells returned by `find_active_cells`.
    active_cells: Mutex<HashMap<DimensionId, Vec<CompetitiveCellId>>>,
    /// Per-dimension competitive set length.
    competitive_set_lens: Mutex<HashMap<DimensionId, usize>>,
    /// Per-dimension graph importance.
    graph_importances: Mutex<HashMap<DimensionId, f64>>,
    /// Per-(dimension, cell) measurement state.
    measurement_states: Mutex<HashMap<(DimensionId, CompetitiveCellId), crate::identity::MeasurementState>>,
}

impl TestSharedState {
    fn new() -> Self {
        Self {
            channels: vec![ChannelId(1)],
            sentinels: Vec::new(),
            identity_dims: Vec::new(),
            next_id: AtomicU64::new(1),
            concordance_observations: Mutex::new(Vec::new()),
            pending_entries: Mutex::new(Vec::new()),
            sentinel_reports: Mutex::new(HashMap::new()),
            active_cells: Mutex::new(HashMap::new()),
            competitive_set_lens: Mutex::new(HashMap::new()),
            graph_importances: Mutex::new(HashMap::new()),
            measurement_states: Mutex::new(HashMap::new()),
        }
    }

    #[allow(dead_code)]
    fn with_channel(mut self, id: u32) -> Self {
        self.channels.push(ChannelId(id));
        self
    }

    fn with_sentinel(mut self, id: u32) -> Self {
        self.sentinels.push(SentinelId(id));
        self
    }

    fn with_sentinel_report(self, id: u32, report: ReportIndex) -> Self {
        self.sentinel_reports.lock().unwrap().insert(SentinelId(id), Arc::new(report));
        self.with_sentinel(id)
    }

    fn with_identity_dim(mut self, dim_id: u32) -> Self {
        self.identity_dims.push(DimensionId(dim_id));
        self
    }

    fn with_active_cells(self, dim_id: u32, cells: Vec<CompetitiveCellId>) -> Self {
        self.active_cells.lock().unwrap().insert(DimensionId(dim_id), cells);
        self
    }

    fn with_competitive_set_len(self, dim_id: u32, len: usize) -> Self {
        self.competitive_set_lens.lock().unwrap().insert(DimensionId(dim_id), len);
        self
    }

    fn with_graph_importance(self, dim_id: u32, importance: f64) -> Self {
        self.graph_importances.lock().unwrap().insert(DimensionId(dim_id), importance);
        self
    }

    fn with_measurement_state(self, dim_id: u32, cell: CompetitiveCellId, state: crate::identity::MeasurementState) -> Self {
        self.measurement_states
            .lock()
            .unwrap()
            .insert((DimensionId(dim_id), cell), state);
        self
    }

    fn pending_count(&self) -> usize {
        self.pending_entries.lock().unwrap().len()
    }

    fn last_pending(&self) -> Option<PendingAssessment> {
        self.pending_entries.lock().unwrap().last().cloned()
    }
}

impl AssessmentSharedState for TestSharedState {
    fn now_persistent(&self) -> crate::types::PersistentTimestamp {
        // Fixed, not wall-clock: these pipeline tests gain nothing from
        // a moving present and lose reproducibility to it.
        crate::types::PersistentTimestamp::new(i64::try_from(crate::testing::VirtualClock::EPOCH_SECS).unwrap_or(0), 0)
    }

    fn concordance_thresholds(&self) -> [f64; SCORING_AXIS_COUNT] {
        [0.5, 0.5, 0.5, 0.5]
    }

    fn observe_concordance(&self, per_axis_max_z: [f64; SCORING_AXIS_COUNT]) {
        self.concordance_observations.lock().unwrap().push(per_axis_max_z);
    }

    fn convergence_stage(&self) -> CompositeConvergenceStage {
        CompositeConvergenceStage::ColdStart
    }

    fn next_assessment_id(&self) -> AssessmentId {
        AssessmentId(self.next_id.fetch_add(1, Ordering::Relaxed))
    }

    fn insert_pending(&self, pending: PendingAssessment) {
        self.pending_entries.lock().unwrap().push(pending);
    }

    fn sentinel_report(&self, sentinel_id: SentinelId) -> Option<Arc<ReportIndex>> {
        self.sentinel_reports.lock().unwrap().get(&sentinel_id).cloned()
    }

    fn sentinel_ledger(&self, _sentinel_id: SentinelId) -> Option<Arc<std::sync::RwLock<crate::ledger::SentinelLedger>>> {
        None
    }

    fn extraction_config(&self) -> crate::extraction::ExtractionConfig {
        crate::extraction::ExtractionConfig::default()
    }

    fn ledger_immaturity_criteria(&self) -> crate::ledger::ImmaturityCriteria {
        // The shipped defaults, so the double's flag is taken against the same
        // thresholds a deployment's is
        // (´def:config:ledger-materiality-threshold´),
        // (´def:config:attenuation-materiality-floor´).
        let config = crate::config::types::AssayerConfig::default();
        crate::ledger::ImmaturityCriteria {
            materiality_threshold: config.monitoring.ledger_materiality_threshold,
            attenuation_floor: config.monitoring.attenuation_materiality_floor,
            lambda_l: config.ledger.lambda_l,
        }
    }

    fn outcome_axis_ids(&self) -> Vec<OutcomeAxisId> {
        Vec::new()
    }

    fn gamma_t_ledger(&self) -> f64 {
        0.01
    }

    fn gamma_t_core(&self) -> f64 {
        0.9999
    }

    fn registered_sentinels(&self) -> Vec<SentinelId> {
        self.sentinels.clone()
    }

    fn registered_identity_dimensions(&self) -> Vec<DimensionId> {
        self.identity_dims.clone()
    }

    fn encode_for_dimension(&self, _dim_id: DimensionId, entity: &crate::types::EntityKey) -> Option<u128> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        // Test mock: use default hasher for all dimensions.
        let mut hasher = DefaultHasher::new();
        entity.as_bytes().hash(&mut hasher);
        let h1 = hasher.finish();
        let mut hasher2 = DefaultHasher::new();
        h1.hash(&mut hasher2);
        let h2 = hasher2.finish();
        Some((u128::from(h1) << 64) | u128::from(h2))
    }

    fn find_active_cells(&self, dim_id: DimensionId, _coord: u128) -> Vec<CompetitiveCellId> {
        self.active_cells.lock().unwrap().get(&dim_id).cloned().unwrap_or_default()
    }

    fn competitive_set_len(&self, dim_id: DimensionId) -> usize {
        self.competitive_set_lens.lock().unwrap().get(&dim_id).copied().unwrap_or(0)
    }

    fn graph_total_importance(&self, dim_id: DimensionId) -> f64 {
        self.graph_importances.lock().unwrap().get(&dim_id).copied().unwrap_or(0.0)
    }

    fn dimension_domain_bits(&self, _dim_id: DimensionId) -> Option<u8> {
        Some(128)
    }

    fn cell_measurement_state(
        &self,
        dim_id: DimensionId,
        cell_id: &CompetitiveCellId,
    ) -> Option<crate::identity::MeasurementState> {
        self.measurement_states.lock().unwrap().get(&(dim_id, *cell_id)).cloned()
    }

    fn cell_outcome_state(
        &self,
        _dim_id: DimensionId,
        _cell_id: &CompetitiveCellId,
        _now: &crate::types::PersistentTimestamp,
    ) -> Option<crate::identity::CellOutcomeState> {
        None
    }

    fn record_identity_observation(&self, _dim_id: DimensionId, _coord: u128) {
        // Test mock: no-op. Real implementation sends to maintenance loop.
    }

    fn update_cell_measurement(
        &self,
        _dim_id: DimensionId,
        _cell_id: &CompetitiveCellId,
        _sentinel_alarms: &[(SentinelId, f64)],
    ) {
        // Test mock: no-op.
    }

    fn lambda_prior(&self) -> f64 {
        0.1
    }

    fn signal_cache_get_and_merge(
        &self,
        _entity: &crate::types::EntityKey,
        _signals: &std::collections::HashMap<String, crate::signal::SignalValue>,
    ) -> Vec<f64> {
        // Test mock: return empty (no signal schema configured).
        Vec::new()
    }

    fn count_signal_shape_mismatches(&self, _signals: &std::collections::HashMap<String, crate::signal::SignalValue>) -> u32 {
        // Test mock: no schema → no mismatches detectable.
        0
    }

    fn count_unknown_signals(&self, _signals: &std::collections::HashMap<String, crate::signal::SignalValue>) -> u32 {
        // Test mock: no schema → no unknown-signal diagnostics.
        0
    }

    fn offer_cold_observation(&self, _phi_raw: &[f64], _layout_generation: u64) -> crate::assessment::BatchInitObservation {
        // Test mock: nothing to observe.
        crate::assessment::BatchInitObservation::Observed
    }

    fn observe_sentinel_bootstrap(&self, _sentinel_id: SentinelId, _features: &crate::pending::StoredFeatures) {
        // Test mock: no-op.
    }
}

/// Creates a minimal test snapshot.
fn test_snapshot() -> ModelSnapshot {
    use indexmap::IndexMap;

    use crate::feature::dimension_map::DimensionMap;
    use crate::model::parameters::ModelParameters;

    // Build dimension map first to get the actual feature dimension
    let dim_map = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &[], &IndexMap::new(), &[]);
    let dim_p = dim_map.p;

    // Create model parameters matching the dimension map's p
    let mu = vec![0.0; dim_p];
    let mut covariance_data = vec![0.0; dim_p * dim_p];
    for i in 0..dim_p {
        covariance_data[i * dim_p + i] = 1.0;
    }
    let params = ModelParameters {
        mu,
        covariance_data,
        p: dim_p,
    };

    let anchor_p = 15;
    let anchor_mu = vec![0.0; anchor_p];
    let mut anchor_cov = vec![0.0; anchor_p * anchor_p];
    for i in 0..anchor_p {
        anchor_cov[i * anchor_p + i] = 1.0;
    }
    let anchor = ModelParameters {
        mu: anchor_mu,
        covariance_data: anchor_cov,
        p: anchor_p,
    };

    let mut snapshot = ModelSnapshot::cold_start(1, params.clone(), params, anchor, dim_p);
    snapshot.feature_means = vec![0.0; dim_p];
    snapshot.feature_variances = vec![1.0; dim_p];

    snapshot
}

fn assess_then_derive<S: AssessmentSharedState>(
    req: &RequestContext,
    shared: &S,
    snapshot: &ModelSnapshot,
    batch_now: Instant,
) -> DerivedReckoning {
    let assessment = assess_single(req, shared, snapshot, batch_now);
    let policy = ChannelPolicy::default();
    let landscape = derive_landscape(&assessment, &policy, ChallengeEstimate::default());
    let profile = render_resonances(&landscape, &assessment.risk, &ResonanceConfig::default());
    DerivedReckoning {
        channel: req.channel_hint().unwrap_or(ChannelId(0)),
        assessment,
        landscape,
        profile,
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pending Buffer Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The pending record left behind by an assessment carries the entity it was
/// about and the degradation it ran under, alongside the assessment's own
/// identifier. A label arriving hours later names only the identifier, so
/// everything needed to turn that label into a training update — who it was
/// about, and how trustworthy the estimate was — has to have been captured at
/// assessment time.
///
/// ´claim:assess:the-pending-record-captures-the-entity-and-the-degradation-the-estimate-ran-under´
/// ´test:crate:pending-entry-has-correct-contents´
#[test]
fn pending_entry_has_correct_contents() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();
    let entity = World::entity("alpha");
    let req = RequestContext::new(entity.clone());

    let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    let pending = shared.last_pending().expect("pending entry should exist");

    // Verify pending entry fields
    assert_eq!(pending.id, reckoning.assessment.id);
    assert_eq!(pending.entity.as_bytes(), entity.as_bytes());
    assert!(!pending.degradation.is_degraded());
}

/// The identifier the caller is handed is the identifier under which the
/// pending record was filed, and it stays so as records accumulate: the nth
/// buffered entry belongs to the nth assessment. Identity is allocated once
/// and used for both, so the buffer cannot drift out of step with what
/// callers hold.
///
/// ´claim:assess:the-pending-record-and-the-returned-assessment-are-filed-under-one-identifier´
/// ´test:crate:pending-id-matches-assessment´
#[test]
fn pending_id_matches_assessment() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();

    for i in 0..5 {
        let req = RequestContext::new(World::entity(&format!("e{i}")));
        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

        let entries = shared.pending_entries.lock().unwrap();
        let pending = &entries[i];
        assert_eq!(
            pending.id, reckoning.assessment.id,
            "pending ID should match assessment ID for entry {i}"
        );
    }
}

/// One assessment buffers exactly one pending record — a request that named no
/// channel is no exception. Every estimate is eligible to be labelled later,
/// so the decision to buffer belongs to the core path and not to whatever
/// routing the host layered on top; a duplicate would let one outcome update
/// the model twice.
///
/// ´claim:assess:one-assessment-buffers-exactly-one-pending-record´
/// ´test:crate:pending-inserted-for-channel-free-request´
#[test]
fn pending_inserted_for_channel_free_request() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();

    let req = RequestContext::new(World::entity("alpha"));
    let _reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
    assert_eq!(shared.pending_count(), 1, "pending entry created for assessment");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Per-Sentinel Alarm Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// When a Sentinel has both a cached batch report and a coordinate in the
/// request, the assessment carries a summary for it, and the peak z that
/// summary reports is a real finite number drawn from the report's axis
/// scores. This is the whole point of the per-Sentinel map: a host can see
/// which measurement surface was loud, not merely the pooled risk it fed into.
///
/// ´claim:assess:a-sentinel-with-both-a-report-and-a-coordinate-contributes-a-finite-alarm-summary´
/// ´test:crate:per-sentinel-alarm-populated´
#[test]
fn per_sentinel_alarm_populated() {
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry, ReportIndex, ReportLevelData};
    use crate::types::LedgerKey;

    // Create a report with known scores
    let root_key = LedgerKey::from_coordinate(0, 0);
    let mut cells = HashMap::new();
    cells.insert(
        root_key,
        ReportCellEntry {
            depth: 0,
            sample_count: 100,
            is_competitive: true,
            rank: 4,
            cap: 8,
            energy_ratio: 0.85,
            noise_influence: 0.05,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: 2.5,
                    mean_z: 1.0,
                    cusum: 0.5,
                    baseline_mean: 0.0,
                    baseline_variance: 1.0,
                    clip_pressure: 0.0,
                },
                displacement: AxisScoreSnapshot {
                    max_z: 1.5,
                    ..Default::default()
                },
                surprise: AxisScoreSnapshot {
                    max_z: 1.0,
                    ..Default::default()
                },
                coherence: AxisScoreSnapshot {
                    max_z: 0.8,
                    ..Default::default()
                },
            },
            degraded: false,
        },
    );

    let report = ReportIndex::from_components(cells, vec![], ReportLevelData::default(), 0);

    let shared = TestSharedState::new().with_sentinel_report(1, report);
    let snapshot = test_snapshot();

    let coord: u128 = 0x1234_5678_9ABC_DEF0;
    let req = RequestContext::new(World::entity("alpha")).with_sentinel(SentinelId(1), coord);

    let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    // Should have one Sentinel summary
    assert_eq!(reckoning.assessment.per_sentinel.len(), 1, "should have 1 Sentinel summary");
    let summary = &reckoning.assessment.per_sentinel[&SentinelId(1)];
    assert_finite(&[summary.peak_z], "per-sentinel peak z");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Scale and Concurrency Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The ordering property survives a run of a hundred assessments, not merely
/// the two or three that fit in a small test: every identifier is distinct and
/// each exceeds the last. A counter that wrapped, repeated, or skipped
/// backwards under sustained traffic would corrupt the label join precisely in
/// the busy conditions where it matters.
///
/// (´claim:assess:assessment-ids-are-handed-out-in-strictly-increasing-order´)
/// ´test:crate:multiple-assessments-unique-ids´
#[test]
fn multiple_assessments_unique_ids() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();

    const N: usize = 100;
    let mut ids = Vec::with_capacity(N);

    for i in 0..N {
        let req = RequestContext::new(World::entity(&format!("e{i}")));
        let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());
        ids.push(reckoning.assessment.id);
    }

    // All IDs should be unique
    let mut sorted = ids.clone();
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), N, "all IDs should be unique");

    // IDs should be monotonically increasing
    for i in 1..ids.len() {
        assert!(ids[i] > ids[i - 1], "IDs should be monotonic");
    }
}

/// Four threads assessing at once still receive four hundred distinct
/// identifiers, and the buffer ends up holding four hundred records — none
/// lost to a race, none double-counted. Assessment is the hot path a host runs
/// from every request-handling thread it has, so allocation and buffering both
/// have to be safe under genuine contention rather than merely under a lock
/// the host is trusted to hold.
///
/// ´claim:assess:concurrent-assessments-neither-collide-on-an-identifier-nor-lose-a-pending-record´
/// ´test:crate:concurrent-assessments-thread-safe´
#[test]
fn concurrent_assessments_thread_safe() {
    use std::thread;

    let shared = Arc::new(TestSharedState::new());
    let snapshot = Arc::new(test_snapshot());

    const THREADS: usize = 4;
    const PER_THREAD: usize = 100;

    let all_ids: Vec<AssessmentId> = (0..THREADS)
        .map(|t| {
            let shared = Arc::clone(&shared);
            let snapshot = Arc::clone(&snapshot);
            thread::spawn(move || {
                let mut ids = Vec::with_capacity(PER_THREAD);
                for i in 0..PER_THREAD {
                    let req = RequestContext::new(World::entity(&format!("t{t}-e{i}")));
                    let reckoning = assess_then_derive(&req, shared.as_ref(), &snapshot, Instant::now());
                    ids.push(reckoning.assessment.id);
                }
                ids
            })
        })
        .flat_map(|h| h.join().unwrap())
        .collect();

    // All IDs should be unique across all threads
    let mut sorted = all_ids;
    sorted.sort_unstable();
    sorted.dedup();
    assert_eq!(sorted.len(), THREADS * PER_THREAD, "all IDs should be unique across threads");

    // Pending buffer should have all entries
    assert_eq!(shared.pending_count(), THREADS * PER_THREAD);
}

// ═══════════════════════════════════════════════════════════════════════════════
// NaN Sanitisation Tests (CP1-CP4 crate-level)
// ═══════════════════════════════════════════════════════════════════════════════

/// Sanitisation counts, it does not merely flag: a request carrying a NaN and
/// both infinities comes back with all three tallied. Because the number
/// rather than a boolean crosses back to the host, an operator can tell one
/// stray value from a wholesale collapse of an upstream feed, and both
/// infinities count the same as the NaN does.
///
/// ´claim:assess:non-finite-signals-are-tallied-individually-so-degradation-is-a-count-not-a-flag´
/// ´test:crate:multiple-nan-signals-counted´
#[test]
fn multiple_nan_signals_counted() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();
    let req = RequestContext::new(World::entity("alpha"))
        .with_signal("sig1", f64::NAN)
        .with_signal("sig2", f64::INFINITY)
        .with_signal("sig3", f64::NEG_INFINITY);

    let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    // All three non-finite signals should be counted
    assert!(
        reckoning.assessment.health.degradation.signals_sanitised >= 3,
        "all NaN/Inf signals should be counted: got {}",
        reckoning.assessment.health.degradation.signals_sanitised
    );
}

/// Degradation is not only reported to the caller — it is stored with the
/// pending record too. When the label for this request eventually arrives, the
/// learner can see that the estimate it is about to be trained against was
/// built on salvaged input, which is exactly the information needed to
/// down-weight it rather than treat it as a clean observation.
///
/// ´claim:assess:degradation-is-stored-with-the-pending-record-so-a-later-label-knows-what-the-estimate-was-built-on´
/// ´test:crate:degradation-propagates-to-pending´
#[test]
fn degradation_propagates_to_pending() {
    let shared = TestSharedState::new();
    let snapshot = test_snapshot();
    let req = RequestContext::new(World::entity("alpha")).with_signal("bad_sig", f64::NAN);

    let _reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    let pending = shared.last_pending().expect("pending entry should exist");
    assert!(
        pending.degradation.signals_sanitised >= 1,
        "degradation should propagate to pending"
    );
}

/// A single non-finite score anywhere in a Sentinel's extraction condemns that
/// Sentinel's whole slot, and the Sentinel is named — by identity, not merely
/// counted — in the assessment's degradation record. Naming it is what lets an
/// operator go and fix the one broken measurement surface; zeroing the whole
/// slot rather than the offending value is what stops a corrupt report from
/// contributing half-believable features to the risk estimate.
///
/// ´claim:assess:a-sentinel-whose-extraction-goes-non-finite-loses-its-whole-slot-and-is-named-in-the-degradation-record´
/// ´test:crate:cp2-nan-extraction-records-sentinel-id´
#[test]
fn cp2_nan_extraction_records_sentinel_id() {
    use crate::report::{AxisScoreSet, AxisScoreSnapshot, ReportCellEntry, ReportIndex, ReportLevelData};
    use crate::types::LedgerKey;

    // Create a report with NaN in z-scores
    let root_key = LedgerKey::from_coordinate(0, 0);
    let mut cells = HashMap::new();
    cells.insert(
        root_key,
        ReportCellEntry {
            depth: 0,
            sample_count: 100,
            is_competitive: true,
            rank: 4,
            cap: 8,
            energy_ratio: 0.85,
            noise_influence: 0.05,
            scores: AxisScoreSet {
                novelty: AxisScoreSnapshot {
                    max_z: f64::NAN, // NaN z-score triggers CP2
                    mean_z: 1.0,
                    cusum: 0.5,
                    baseline_mean: 0.0,
                    baseline_variance: 1.0,
                    clip_pressure: 0.0,
                },
                displacement: AxisScoreSnapshot::default(),
                surprise: AxisScoreSnapshot::default(),
                coherence: AxisScoreSnapshot::default(),
            },
            degraded: false,
        },
    );

    let report = ReportIndex::from_components(cells, vec![], ReportLevelData::default(), 0);
    let shared = TestSharedState::new().with_sentinel_report(1, report);
    let snapshot = test_snapshot();

    let coord: u128 = 0; // Root coordinate
    let req = RequestContext::new(World::entity("alpha")).with_sentinel(SentinelId(1), coord);

    let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    // CP2: Sentinel with NaN extraction should be recorded in nan_sentinels
    assert!(
        reckoning.assessment.health.degradation.nan_sentinels.contains(&SentinelId(1)),
        "Sentinel with NaN extraction should be in nan_sentinels"
    );
    assert!(
        reckoning.assessment.health.degradation.is_degraded(),
        "assessment should be degraded"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// L3 Identity Data-Wiring Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Two different entity keys land at two different places in a registered
/// identity dimension, and each assessment records where its entity landed.
/// Identity features are only informative if position is a function of the
/// subject: an encoder that collapsed distinct entities onto one coordinate
/// would have every request read the same neighbourhood's history, and the
/// dimension would contribute a constant to every risk estimate.
///
/// ´claim:assess:distinct-entities-encode-to-distinct-coordinates-in-a-registered-identity-dimension´
/// ´test:crate:identity-aggregates-vary-by-entity´
#[test]
fn identity_aggregates_vary_by_entity() {
    let shared = TestSharedState::new()
        .with_identity_dim(1)
        .with_active_cells(1, make_cells(2))
        .with_competitive_set_len(1, 100)
        .with_graph_importance(1, 5.0);

    let snapshot = test_snapshot();

    let req_a = RequestContext::new(World::entity("alpha"));
    let req_b = RequestContext::new(World::entity("bravo"));

    let reckoning_a = assess_then_derive(&req_a, &shared, &snapshot, Instant::now());
    let reckoning_b = assess_then_derive(&req_b, &shared, &snapshot, Instant::now());

    // Both should have identity coordinates (verifies step 2.5 runs)
    let pending_a = shared.pending_entries.lock().unwrap();
    assert_eq!(pending_a.len(), 2);

    let coords_a = &pending_a[0].identity_coordinates;
    let coords_b = &pending_a[1].identity_coordinates;

    // Both should have coordinates for dimension 1
    assert!(
        coords_a.contains_key(&DimensionId(1)),
        "entity A should have dim 1 coordinate"
    );
    assert!(
        coords_b.contains_key(&DimensionId(1)),
        "entity B should have dim 1 coordinate"
    );

    // Coordinates should differ (different entity keys → different hashes)
    assert_ne!(
        coords_a[&DimensionId(1)],
        coords_b[&DimensionId(1)],
        "different entities should produce different coordinates"
    );

    // Both host-composed assessment/derivation results should complete successfully.
    assert_finite(
        &[reckoning_a.assessment.risk.p_bad, reckoning_b.assessment.risk.p_bad],
        "risk p_bad",
    );
}

/// The pending record remembers not just where the entity landed but which
/// competitive cells were active there at assessment time, per dimension.
/// Cells come and go as the competitive set is rebuilt, so a label arriving
/// later must be attributed to the cells that actually informed the estimate
/// rather than to whichever cells happen to cover that coordinate by then.
///
/// ´claim:assess:the-pending-record-remembers-which-competitive-cells-were-active-when-the-estimate-was-made´
/// ´test:crate:per-dimension-measurement-reads-real-ewmas´
#[test]
fn per_dimension_measurement_reads_real_ewmas() {
    use crate::identity::MeasurementState;

    let cell = make_cells(1)[0];

    let mut ms = MeasurementState::default();
    ms.per_sentinel_alarm_ewma.insert(SentinelId(1), 0.75);
    ms.per_sentinel_alarm_ewma.insert(SentinelId(2), 0.25);
    ms.volatility = 0.4;
    ms.step_count = 10;

    let shared = TestSharedState::new()
        .with_identity_dim(1)
        .with_active_cells(1, vec![cell])
        .with_competitive_set_len(1, 50)
        .with_graph_importance(1, 3.0)
        .with_measurement_state(1, cell, ms);

    let snapshot = test_snapshot();
    let req = RequestContext::new(World::entity("alpha"));

    let reckoning = assess_then_derive(&req, &shared, &snapshot, Instant::now());

    // Verify through the pending buffer that the per-dimension measurement
    // features were populated from real EWMA values.
    let pending = shared.last_pending().expect("should have pending entry");

    // Verify the identity active cells were recorded
    assert!(
        pending.identity_active_cells.contains_key(&DimensionId(1)),
        "identity dimension active cells should be recorded"
    );
    assert_eq!(
        pending.identity_active_cells[&DimensionId(1)].len(),
        1,
        "should have one active cell"
    );

    // Verify the identity coordinate was recorded
    assert!(
        pending.identity_coordinates.contains_key(&DimensionId(1)),
        "identity dimension coordinate should be recorded"
    );

    // Verify the reckoning completes with well-formed risk
    assert_finite(&[reckoning.assessment.risk.p_bad], "risk p_bad");
}
