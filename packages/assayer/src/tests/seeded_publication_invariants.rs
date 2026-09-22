// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`assessment_write_set_seeded_sweep`] | assess | Every drawn assessment changes exactly the enumerated measurement state, leaves the label-authorised state bit-identical, and records the complete request in the pending probe. The generator draws twenty-four finite entity-persistent scalar signals from `[0, 1)`, positive scenario-time advances from one second through one minute, and distinct entity keys. Every case uses the complete builder, first completes a two-observation cold ramp, then leaves a ten-observation late-Sentinel bootstrap nine observations complete before the drawn assessment. That forced setup reaches both observational acquisition mechanisms; a registered identity, a reporting Sentinel, the concordance window, the signal cache and the pending buffer make every enumerated assessment-side mutation non-vacuous. (´inv:guarantee:assess-only´) (´inv:guarantee:evidence-authority´) (´inv:runtime:enumerated-writes´) |
//! | [`lifecycle_publication_seeded_sweep`] | lifecycle | Every lifecycle direction presents one complete pre-event snapshot to an assessment begun while publication is held and one complete post-event model set after the named barriers. The generator cycles registration and deregistration for Sentinels, outcome axes and identity dimensions six times each across thirty-six cases while drawing axis spatiality, entity keys and positive scenario-time advances. Every case carries a live Sentinel and axis model, while destructive cases also start with their target live. The cycle therefore reaches every model-set rule and every destructive direction by construction rather than probability; the owner block fixes the pre-publication assessment, and the identity, lifecycle, model, slot and layout barriers and probes verify one settled version and one dimension semantics after release. (´inv:guarantee:lifecycle-publication´) (´inv:dimension:version-consistency´) (´inv:registry:model-set-serialisation´) |
//! | [`axis_lifecycle_seeded_sweep`] | lifecycle | Outcome-axis registration is exact extension for every existing full-space model, and deregistration applies the independent regularised-Schur result to every survivor while leaving the anchor fixed. The generator draws twenty-four worlds, alternates spatial and non-spatial target axes, varies the existing-axis population from none through two, and draws two outcomes and an entity key. Every case trains the existing model set before registering and deregistering the target, so exact extension observes non-prior means and covariance rather than interchangeable diagonal cells. Spatial cases add live Sentinel coordinates and alternate the drawn outcomes through the remainder of one full minimum recomputation cadence so the removed block participates in a non-trivial posterior and the published covariance is an observable inverse for the precision that the probe contract deliberately withholds. The transition-derived removal set preserves the stable batch tail, and the independent covariance-to-precision, Schur and precision-to-covariance route checks the destructive transition within the configured Schur offset plus the published inverse-pair synchronisation ceiling; non-spatial cases exercise the zero-width model-set edge. (´inv:guarantee:axis-lifecycle´) |
//! | [`publication_staleness_seeded_sweep`] | lifecycle | A positive scenario-time advance appears exactly as publication age, and the next label publication resets that age to zero at a later version. The generator draws thirty-two positive whole-second ages from one second through one day and both adverse and benign freshening labels. Every case therefore enters the stale region before crossing the label barrier into a fresh publication; model probes record scenario-owned observation time on both sides, so elapsed execution time cannot satisfy the predicate. (´inv:guarantee:staleness´) |
//!
//! Seeded real-engine packs for assessment mutation authority, lifecycle publication, model-set shape, axis transformations, and publication age.

use std::collections::HashMap;
use std::sync::mpsc;
use std::time::Duration;

use faer::linalg::solvers::DenseSolveCore;
use faer::{Mat, Side};

use crate::Persistence;
use crate::assessment::RiskAssessment;
use crate::config::types::AssayerConfig;
use crate::feature::standardisation::StandardisationPhase;
use crate::health::{DriftHealth, SystemHealthReport};
use crate::owner::commands::{IdentityDimensionRegistration, OutcomeAxisRegistration, SentinelRegistration};
use crate::resonance::channel::ChannelPolicy;
use crate::testing::signals::scalar_decl;
use crate::testing::{
    DEFAULT_TOLERANCES, LabelSpec, PublishedModelBlock, RuntimeLayout, World, attach_golden_reporting_sentinel,
    regularised_schur_complement, run_seeded_sweep, test_infrastructure,
};
use crate::types::{DimensionId, IdentityBudget, ModelId, OutcomeAxisId, OutcomeEligibility, SentinelId, SpatialFeaturePolicy};

const CHANNEL: &str = "default";
const BASE_SENTINEL: &str = "base-sentinel";

#[derive(Clone, Copy, Debug)]
struct AssessmentWriteCase {
    elapsed_seconds: u64,
    signal: f64,
    entity: u64,
}

#[derive(Clone, Copy, Debug)]
struct StalenessCase {
    elapsed_seconds: u64,
    adverse: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum LifecycleKind {
    RegisterSentinel,
    DeregisterSentinel,
    RegisterAxis,
    DeregisterAxis,
    RegisterIdentity,
    DeregisterIdentity,
}

impl LifecycleKind {
    const fn from_index(index: usize) -> Self {
        match index % 6 {
            0 => Self::RegisterSentinel,
            1 => Self::DeregisterSentinel,
            2 => Self::RegisterAxis,
            3 => Self::DeregisterAxis,
            4 => Self::RegisterIdentity,
            _ => Self::DeregisterIdentity,
        }
    }
}

#[derive(Clone, Copy, Debug)]
struct LifecycleCase {
    kind: LifecycleKind,
    spatial: bool,
    elapsed_seconds: u64,
    entity: u64,
}

#[derive(Clone, Copy, Debug)]
struct AxisLifecycleCase {
    spatial: bool,
    existing_axes: usize,
    first_outcome: f64,
    second_outcome: f64,
    entity: u64,
}

#[derive(Debug)]
struct PublishedState {
    layout: RuntimeLayout,
    version: u64,
    layout_generation: u64,
    operational: PublishedModelBlock,
    sister: PublishedModelBlock,
    anchor: PublishedModelBlock,
    axes: HashMap<OutcomeAxisId, PublishedModelBlock>,
    base_slot_width: usize,
}

#[derive(Debug, Eq, PartialEq)]
struct ModelPayload {
    dimension: usize,
    mean: Vec<u64>,
    covariance: Vec<u64>,
    spectral_floor_mass: Option<u64>,
    clamp_floor_mass: Option<Vec<u64>>,
}

#[derive(Debug, Eq, PartialEq)]
struct DriftPayload {
    s_plus: u64,
    s_minus: u64,
    mean_abs_residual: u64,
    sign_ewma: u64,
    steps_since_reset: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct TaughtState {
    models: Vec<ModelPayload>,
    drift: Vec<(ModelId, DriftPayload)>,
    calibration: CalibrationPayload,
    label_integrity: LabelIntegrityPayload,
    ledger: Vec<(SentinelId, usize, u64, u64, u64)>,
}

#[derive(Debug, Eq, PartialEq)]
struct CalibrationPayload {
    kappa_sister: u64,
    kappa_anchor: u64,
    delta_cal: u64,
    refits_completed: u32,
    labels_since_refit: u32,
    buffer_total: usize,
    sister_regime_records: u64,
    anchor_regime_records: u64,
    anchor_regime_frozen: bool,
    labels_since_last_anchor_fit: u64,
}

#[derive(Debug, Eq, PartialEq)]
struct LabelIntegrityPayload {
    total_labels: u64,
    eligible_labels: u64,
    valences_sanitised: u64,
    outcomes_dropped: u64,
    per_action: HashMap<crate::types::Action, u64>,
}

struct PreparedLifecycle {
    world: World,
    axes: Vec<OutcomeAxisId>,
    target_sentinel: Option<SentinelId>,
    target_axis: Option<OutcomeAxisId>,
    target_identity: Option<DimensionId>,
}

fn base_config(instance_id: &str) -> AssayerConfig {
    AssayerConfig {
        instance_id: instance_id.to_owned(),
        infrastructure: test_infrastructure(),
        ..Default::default()
    }
}

fn build_world(instance_id: &str, seed: u64) -> Result<World, String> {
    World::builder(base_config(instance_id))
        .channel(CHANNEL, ChannelPolicy::default())
        .seed(seed)
        .build()
        .map_err(|error| format!("complete WorldBuilder construction failed: {error}"))
}

fn model_payload(block: &PublishedModelBlock) -> ModelPayload {
    ModelPayload {
        dimension: block.dimension,
        mean: block.mean.iter().map(|value| value.to_bits()).collect(),
        covariance: block.covariance.iter().map(|value| value.to_bits()).collect(),
        spectral_floor_mass: block.spectral_floor_mass.map(f64::to_bits),
        clamp_floor_mass: block
            .clamp_floor_mass
            .as_ref()
            .map(|values| values.iter().map(|value| value.to_bits()).collect()),
    }
}

fn drift_payload(drift: &DriftHealth) -> DriftPayload {
    DriftPayload {
        s_plus: drift.s_plus.to_bits(),
        s_minus: drift.s_minus.to_bits(),
        mean_abs_residual: drift.mean_abs_residual.to_bits(),
        sign_ewma: drift.sign_ewma.to_bits(),
        steps_since_reset: drift.steps_since_reset,
    }
}

fn taught_state(world: &World, report: &SystemHealthReport) -> Result<TaughtState, String> {
    let mut models = Vec::new();
    for model in [ModelId::Operational, ModelId::Sister, ModelId::Anchor] {
        let block = world
            .published_model_block(model.clone())
            .map_err(|error| format!("model probe failed for {model:?}: {error}"))?
            .ok_or_else(|| format!("model probe found no block for {model:?}"))?;
        models.push(model_payload(&block));
    }

    let mut drift = Vec::new();
    for model in [ModelId::Operational, ModelId::Sister, ModelId::Anchor] {
        let reading = report
            .drift
            .get(&model)
            .ok_or_else(|| format!("health report found no drift block for {model:?}"))?;
        drift.push((model, drift_payload(reading)));
    }

    let calibration = &report.calibration;
    let label_integrity = &report.label_integrity;
    let ledger = report
        .ledger
        .per_sentinel
        .iter()
        .map(|(&sentinel, health)| {
            (
                sentinel,
                health.entry_count,
                health.root_adverse_rate.to_bits(),
                health.root_compressed_valence.to_bits(),
                health.root_raw_valence.to_bits(),
            )
        })
        .collect();

    Ok(TaughtState {
        models,
        drift,
        calibration: CalibrationPayload {
            kappa_sister: calibration.kappa_sister.to_bits(),
            kappa_anchor: calibration.kappa_anchor.to_bits(),
            delta_cal: calibration.delta_cal.to_bits(),
            refits_completed: calibration.refits_completed,
            labels_since_refit: calibration.labels_since_refit,
            buffer_total: calibration.buffer_total,
            sister_regime_records: calibration.sister_regime_records.to_bits(),
            anchor_regime_records: calibration.anchor_regime_records.to_bits(),
            anchor_regime_frozen: calibration.anchor_regime_frozen,
            labels_since_last_anchor_fit: calibration.labels_since_last_anchor_fit,
        },
        label_integrity: LabelIntegrityPayload {
            total_labels: label_integrity.total_labels,
            eligible_labels: label_integrity.eligible_labels,
            valences_sanitised: label_integrity.valences_sanitised,
            outcomes_dropped: label_integrity.outcomes_dropped,
            per_action: label_integrity.per_action.clone(),
        },
        ledger,
    })
}

fn probe_model(world: &World, model: &ModelId) -> Result<PublishedModelBlock, String> {
    world
        .published_model_block(model.clone())
        .map_err(|error| format!("model probe failed for {model:?}: {error}"))?
        .ok_or_else(|| format!("model probe found no block for {model:?}"))
}

fn capture_published_state(world: &World, axis_ids: &[OutcomeAxisId]) -> Result<PublishedState, String> {
    let layout = world
        .observed_runtime_layout()
        .map_err(|error| format!("runtime layout probe failed: {error}"))?;
    let operational = probe_model(world, &ModelId::Operational)?;
    let sister = probe_model(world, &ModelId::Sister)?;
    let anchor = probe_model(world, &ModelId::Anchor)?;
    let mut axes = HashMap::new();
    for &axis in axis_ids {
        if let Some(block) = world
            .published_model_block(ModelId::OutcomeAxis(axis))
            .map_err(|error| format!("axis model probe failed for {axis:?}: {error}"))?
        {
            axes.insert(axis, block);
        }
    }
    let base_slot = world
        .published_slot_moments(BASE_SENTINEL)
        .map_err(|error| format!("slot probe failed: {error}"))?
        .ok_or_else(|| "base Sentinel has no published slot".to_owned())?;

    let version = operational.published_version;
    let layout_generation = operational.layout_generation;
    for block in std::iter::once(&sister).chain(axes.values()) {
        if block.published_version != version || block.layout_generation != layout_generation {
            return Err(format!(
                "published blocks mix versions: operational=({version}, {layout_generation}), {:?}=({}, {})",
                block.model, block.published_version, block.layout_generation
            ));
        }
        if block.dimension != layout.dimension_map_width
            || block.mean.len() != block.dimension
            || block.covariance.len() != block.dimension * block.dimension
        {
            return Err(format!(
                "model {:?} is inconsistent with layout width {}: dimension={}, mean={}, covariance={}",
                block.model,
                layout.dimension_map_width,
                block.dimension,
                block.mean.len(),
                block.covariance.len()
            ));
        }
    }
    if operational.dimension != layout.dimension_map_width
        || operational.mean.len() != operational.dimension
        || operational.covariance.len() != operational.dimension * operational.dimension
    {
        return Err(format!(
            "operational model is inconsistent with layout width {}: dimension={}, mean={}, covariance={}",
            layout.dimension_map_width,
            operational.dimension,
            operational.mean.len(),
            operational.covariance.len()
        ));
    }
    if base_slot.published_version != version || base_slot.layout_generation != layout_generation {
        return Err(format!(
            "slot publication ({}, {}) disagrees with model publication ({version}, {layout_generation})",
            base_slot.published_version, base_slot.layout_generation
        ));
    }
    if base_slot.means.len() != base_slot.variances.len() || base_slot.means.len() > layout.sentinel_slots_width {
        return Err(format!(
            "slot widths are inconsistent: means={}, variances={}, all_slots={}",
            base_slot.means.len(),
            base_slot.variances.len(),
            layout.sentinel_slots_width
        ));
    }

    Ok(PublishedState {
        layout,
        version,
        layout_generation,
        operational,
        sister,
        anchor,
        axes,
        base_slot_width: base_slot.means.len(),
    })
}

fn axis_registration(id: OutcomeAxisId, spatial: bool) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id,
        name: format!("axis-{}", id.0),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.99,
        spatial_features: if spatial {
            SpatialFeaturePolicy::Enabled
        } else {
            SpatialFeaturePolicy::Disabled
        },
    }
}

fn identity_registration(id: DimensionId) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id,
        name: format!("identity-{}", id.0),
        description: "seeded lifecycle identity".to_owned(),
        coordinate_semantics: "leading bytes select a prefix".to_owned(),
        domain_bits: 128,
        depth_cutoff: 10,
        budget: IdentityBudget::for_depth_cutoff(10),
        encode: |entity| {
            let bytes = entity.as_bytes();
            let mut prefix = [0_u8; 8];
            let copied = bytes.len().min(prefix.len());
            prefix[..copied].copy_from_slice(&bytes[..copied]);
            u128::from(u64::from_le_bytes(prefix))
        },
    }
}

fn matrix_from_covariance(block: &PublishedModelBlock) -> Mat<f64> {
    Mat::from_fn(block.dimension, block.dimension, |row, column| {
        block.covariance[column * block.dimension + row]
    })
}

fn invert_independently(matrix: &Mat<f64>) -> Result<Mat<f64>, String> {
    if matrix.nrows() != matrix.ncols() {
        return Err(format!(
            "cannot invert non-square {} by {} matrix",
            matrix.nrows(),
            matrix.ncols()
        ));
    }
    let factor = matrix
        .llt(Side::Lower)
        .map_err(|error| format!("independent SPD factorisation failed: {error:?}"))?;
    Ok(factor.inverse())
}

fn compare_matrix(actual: &PublishedModelBlock, expected: &Mat<f64>, relative_tolerance: f64) -> Result<(), String> {
    if actual.dimension != expected.nrows() || expected.nrows() != expected.ncols() {
        return Err(format!(
            "matrix dimensions disagree: actual={}, expected={} by {}",
            actual.dimension,
            expected.nrows(),
            expected.ncols()
        ));
    }
    for row in 0..actual.dimension {
        for column in 0..actual.dimension {
            let got = actual.covariance[column * actual.dimension + row];
            let want = expected[(row, column)];
            let tolerance = relative_tolerance * want.abs().max(1.0);
            if (got - want).abs() > tolerance {
                return Err(format!(
                    "covariance[{row}, {column}]={got}, expected {want} within {tolerance}"
                ));
            }
        }
    }
    Ok(())
}

fn check_exact_extension(
    before: &PublishedModelBlock,
    after: &PublishedModelBlock,
    insert_at: usize,
    added: usize,
    prior_variance: f64,
) -> Result<(), String> {
    if after.dimension != before.dimension + added {
        return Err(format!(
            "model {:?} grew from {} to {}, expected {} added coordinates",
            before.model, before.dimension, after.dimension, added
        ));
    }
    for before_index in 0..before.dimension {
        let after_index = if before_index < insert_at {
            before_index
        } else {
            before_index + added
        };
        if before.mean[before_index].to_bits() != after.mean[after_index].to_bits() {
            return Err(format!(
                "model {:?} changed retained mean {before_index} at extended index {after_index}",
                before.model
            ));
        }
        for before_other in 0..before.dimension {
            let after_other = if before_other < insert_at {
                before_other
            } else {
                before_other + added
            };
            let before_value = before.covariance[before_other * before.dimension + before_index];
            let after_value = after.covariance[after_other * after.dimension + after_index];
            if before_value.to_bits() != after_value.to_bits() {
                return Err(format!(
                    "model {:?} changed retained covariance [{before_index}, {before_other}] at [{after_index}, {after_other}]",
                    before.model
                ));
            }
        }
    }
    for index in insert_at..insert_at + added {
        if after.mean[index].to_bits() != 0.0_f64.to_bits() {
            return Err(format!(
                "model {:?} gave new mean {index} the value {}",
                before.model, after.mean[index]
            ));
        }
        for other in 0..after.dimension {
            let value = after.covariance[other * after.dimension + index];
            if other == index && value.to_bits() != prior_variance.to_bits() {
                return Err(format!(
                    "model {:?} gave new covariance diagonal {index} the value {value}, expected {prior_variance}",
                    before.model
                ));
            }
            if other != index && value.to_bits() != 0.0_f64.to_bits() {
                return Err(format!(
                    "model {:?} gave new covariance cross-cell [{index}, {other}] the value {}",
                    before.model, value
                ));
            }
        }
    }
    Ok(())
}

fn check_assessment_write_case(case: &AssessmentWriteCase) -> Result<(), String> {
    let mut config = base_config("assessment-write-sweep");
    config.standardisation.n_init = 2;
    config.standardisation.n_boot = 10;
    let mut world = World::builder(config)
        .channel(CHANNEL, ChannelPolicy::default())
        .signal_schema(vec![scalar_decl("drawn", Persistence::Entity)])
        .seed(0xA55E_5500_0000_0001 ^ case.entity)
        .build()
        .map_err(|error| format!("complete assessment WorldBuilder construction failed: {error}"))?;

    let ramp = world.request(CHANNEL, "ramp").with_signal("drawn", 0.25);
    world.settle_cold_ramp_with(&[ramp]);
    let dimension = world
        .register_identity("identity")
        .map_err(|error| format!("identity registration failed: {error}"))?;
    attach_golden_reporting_sentinel(&mut world, BASE_SENTINEL);

    let training = world.assess(
        world
            .request_with_sentinel(CHANNEL, "training", BASE_SENTINEL, 0)
            .with_signal("drawn", 0.75),
    );
    world
        .flush_observations()
        .map_err(|error| format!("first bootstrap observation barrier failed: {error}"))?;
    world
        .label(LabelSpec::adverse(training.id).ground_truth().build())
        .map_err(|error| format!("training label failed: {error}"))?;
    world
        .flush_labels()
        .map_err(|error| format!("training publication barrier failed: {error}"))?;
    for index in 1_u128..9 {
        let _pending = world.assess(
            world
                .request_with_sentinel(CHANNEL, &format!("bootstrap-{index}"), BASE_SENTINEL, index)
                .with_signal("drawn", 0.75),
        );
    }
    world
        .flush_observations()
        .map_err(|error| format!("bootstrap baseline observation barrier failed: {error}"))?;
    world
        .flush_identity_maintenance()
        .map_err(|error| format!("bootstrap baseline identity barrier failed: {error}"))?;
    world
        .advance(Duration::from_secs(case.elapsed_seconds))
        .map_err(|error| format!("scenario-time advance failed: {error}"))?;

    let before_report = world.assayer().full_health_report();
    let before_taught = taught_state(&world, &before_report)?;
    let before_slot = world
        .published_slot_moments(BASE_SENTINEL)
        .map_err(|error| format!("before slot probe failed: {error}"))?
        .ok_or_else(|| "before slot probe found no Sentinel".to_owned())?;

    let entity = format!("drawn-{}", case.entity);
    let assessment = world.assess(
        world
            .request_with_sentinel(CHANNEL, &entity, BASE_SENTINEL, u128::from(case.entity))
            .with_signal("drawn", case.signal),
    );
    let pending = world
        .pending_entry_view(assessment.id)
        .ok_or_else(|| "assessment did not insert its pending entry".to_owned())?;
    let signals = pending
        .single_precision_signals()
        .ok_or_else(|| "pending entry did not retain single-precision signals".to_owned())?;
    if signals.len() != 1 || (f64::from(signals[0]) - case.signal).abs() > f64::from(f32::EPSILON) {
        return Err(format!(
            "pending signal projection {signals:?} does not carry drawn value {}",
            case.signal
        ));
    }
    if !pending.active_cell_counts.contains_key(&dimension) {
        return Err(format!("pending entry does not enumerate identity dimension {dimension:?}"));
    }

    world
        .flush_observations()
        .map_err(|error| format!("bootstrap observation barrier failed: {error}"))?;
    world
        .flush_identity_maintenance()
        .map_err(|error| format!("identity observation barrier failed: {error}"))?;
    let after_report = world.assayer().full_health_report();
    let after_taught = taught_state(&world, &after_report)?;
    let after_slot = world
        .published_slot_moments(BASE_SENTINEL)
        .map_err(|error| format!("after slot probe failed: {error}"))?
        .ok_or_else(|| "after slot probe found no Sentinel".to_owned())?;

    if before_taught != after_taught {
        return Err(format!(
            "assessment changed outcome-taught state: before={before_taught:#?}, after={after_taught:#?}"
        ));
    }
    if after_report.assessment_degradation.total_assessments != before_report.assessment_degradation.total_assessments + 1 {
        return Err(format!(
            "assessment count moved from {} to {}, expected one step",
            before_report.assessment_degradation.total_assessments, after_report.assessment_degradation.total_assessments
        ));
    }
    if after_report.concordance.total_observations != before_report.concordance.total_observations + 1
        || after_report.concordance.window_size != before_report.concordance.window_size + 1
    {
        return Err(format!(
            "concordance delta is not one observation: before=({}, {}), after=({}, {})",
            before_report.concordance.total_observations,
            before_report.concordance.window_size,
            after_report.concordance.total_observations,
            after_report.concordance.window_size
        ));
    }
    let before_identity = &before_report.identity_dimensions[&dimension].active_indicator_count_distribution;
    let after_identity = &after_report.identity_dimensions[&dimension].active_indicator_count_distribution;
    let before_identity_total = before_identity.values().sum::<u64>();
    let after_identity_total = after_identity.values().sum::<u64>();
    if after_identity_total != before_identity_total + 1 {
        return Err(format!(
            "identity measurement delta is not one observation: before={before_identity:?}, after={after_identity:?}"
        ));
    }
    if after_report.buffer.insertions != before_report.buffer.insertions + 1
        || after_report.buffer.utilisation != before_report.buffer.utilisation + 1
    {
        return Err(format!(
            "pending-buffer delta is not one insertion: before=({}, {}), after=({}, {})",
            before_report.buffer.insertions,
            before_report.buffer.utilisation,
            after_report.buffer.insertions,
            after_report.buffer.utilisation
        ));
    }
    if after_report.signal_cache.misses != before_report.signal_cache.misses + 1
        || after_report.signal_cache.size != before_report.signal_cache.size + 1
    {
        return Err(format!(
            "signal-cache delta is not one new entity: before={:?}, after={:?}",
            before_report.signal_cache, after_report.signal_cache
        ));
    }
    if before_slot.means == after_slot.means && before_slot.variances == after_slot.variances {
        return Err("second late-Sentinel bootstrap observation left every slot moment unchanged".to_owned());
    }
    if before_slot.phase != StandardisationPhase::InService || after_slot.phase != StandardisationPhase::InService {
        return Err(format!(
            "late bootstrap did not run over an in-service ramp: before={:?}, after={:?}",
            before_slot.phase, after_slot.phase
        ));
    }
    Ok(())
}

fn prepare_lifecycle_world(case: &LifecycleCase) -> Result<PreparedLifecycle, String> {
    let mut world = build_world("lifecycle-publication-sweep", 0x11FE_C7CE_0000_0000 ^ case.entity)?;
    world
        .register_sentinel(BASE_SENTINEL)
        .map_err(|error| format!("base Sentinel registration failed: {error}"))?;
    let base_axis = world
        .register_axis("base-axis", false)
        .map_err(|error| format!("base axis registration failed: {error}"))?;

    let mut target_sentinel = None;
    let mut target_axis = None;
    let mut target_identity = None;
    match case.kind {
        LifecycleKind::DeregisterSentinel => {
            target_sentinel = Some(
                world
                    .register_sentinel("target-sentinel")
                    .map_err(|error| format!("target Sentinel setup failed: {error}"))?,
            );
        }
        LifecycleKind::DeregisterAxis => {
            target_axis = Some(
                world
                    .register_axis("target-axis", case.spatial)
                    .map_err(|error| format!("target axis setup failed: {error}"))?,
            );
        }
        LifecycleKind::DeregisterIdentity => {
            target_identity = Some(
                world
                    .register_identity("target-identity")
                    .map_err(|error| format!("target identity setup failed: {error}"))?,
            );
        }
        LifecycleKind::RegisterSentinel | LifecycleKind::RegisterAxis | LifecycleKind::RegisterIdentity => {}
    }
    world
        .advance(Duration::from_secs(case.elapsed_seconds))
        .map_err(|error| format!("scenario-time advance failed: {error}"))?;
    let mut axes = vec![base_axis];
    if let Some(axis) = target_axis {
        axes.push(axis);
    }
    Ok(PreparedLifecycle {
        world,
        axes,
        target_sentinel,
        target_axis,
        target_identity,
    })
}

fn during_assessment(world: &World, entity: u64) -> RiskAssessment {
    world.assess(world.request(CHANNEL, &format!("during-{entity}")))
}

fn check_lifecycle_case(case: &LifecycleCase) -> Result<(), String> {
    let PreparedLifecycle {
        world,
        mut axes,
        target_sentinel,
        mut target_axis,
        target_identity,
    } = prepare_lifecycle_world(case)?;
    let register_sentinel = SentinelId(900);
    let register_axis = OutcomeAxisId(900);
    let register_identity = DimensionId(900);
    if case.kind == LifecycleKind::RegisterAxis {
        axes.push(register_axis);
        target_axis = Some(register_axis);
    }
    let before = capture_published_state(&world, &axes)?;
    let owner_block = world.block_model_owner_for_test();
    let during = std::thread::scope(|scope| {
        let (started_tx, started_rx) = mpsc::sync_channel(0);
        let world_ref = &world;
        let kind = case.kind;
        let spatial = case.spatial;
        let task = scope.spawn(move || {
            if kind == LifecycleKind::DeregisterIdentity {
                started_tx
                    .send(())
                    .map_err(|error| format!("identity deregistration start signal failed: {error}"))?;
            }
            match kind {
                LifecycleKind::RegisterSentinel => world_ref
                    .assayer()
                    .register_sentinel(SentinelRegistration {
                        id: register_sentinel,
                        name: "registered-during-draw".to_owned(),
                    })
                    .map_err(|error| format!("Sentinel registration failed: {error}"))?,
                LifecycleKind::DeregisterSentinel => world_ref
                    .assayer()
                    .deregister_sentinel(target_sentinel.ok_or_else(|| "Sentinel deregistration case has no target".to_owned())?)
                    .map_err(|error| format!("Sentinel deregistration failed: {error}"))?,
                LifecycleKind::RegisterAxis => world_ref
                    .assayer()
                    .register_outcome_axis(axis_registration(register_axis, spatial))
                    .map_err(|error| format!("axis registration failed: {error}"))?,
                LifecycleKind::DeregisterAxis => world_ref
                    .assayer()
                    .deregister_outcome_axis(target_axis.ok_or_else(|| "axis deregistration case has no target".to_owned())?)
                    .map_err(|error| format!("axis deregistration failed: {error}"))?,
                LifecycleKind::RegisterIdentity => world_ref
                    .assayer()
                    .register_identity_dimension(identity_registration(register_identity))
                    .map_err(|error| format!("identity registration failed: {error}"))?,
                LifecycleKind::DeregisterIdentity => world_ref
                    .assayer()
                    .deregister_identity_dimension(
                        target_identity.ok_or_else(|| "identity deregistration case has no target".to_owned())?,
                    )
                    .map_err(|error| format!("identity deregistration failed: {error}"))?,
            }
            if kind != LifecycleKind::DeregisterIdentity {
                started_tx
                    .send(())
                    .map_err(|error| format!("lifecycle submission signal failed: {error}"))?;
            }
            Ok::<_, String>(())
        });
        started_rx
            .recv()
            .map_err(|error| format!("lifecycle submission acknowledgement failed: {error}"))?;
        let assessment = during_assessment(&world, case.entity);
        owner_block.release();
        task.join().map_err(|_| "lifecycle command thread panicked".to_owned())??;
        Ok::<_, String>(assessment)
    })?;

    world
        .flush_identity_maintenance()
        .map_err(|error| format!("in-flight identity barrier failed: {error}"))?;
    world
        .publish_lifecycle()
        .map_err(|error| format!("lifecycle publication barrier failed: {error}"))?;
    let after = capture_published_state(&world, &axes)?;
    let post = world.assess(world.request(CHANNEL, &format!("post-{}", case.entity)));

    if during.health.snapshot_version != before.version {
        return Err(format!(
            "in-flight assessment read version {}, expected pre-event version {}",
            during.health.snapshot_version, before.version
        ));
    }
    if post.health.snapshot_version != after.version {
        return Err(format!(
            "post-event assessment read version {}, expected settled version {}",
            post.health.snapshot_version, after.version
        ));
    }
    if after.version <= before.version || after.layout_generation <= before.layout_generation {
        return Err(format!(
            "lifecycle publication did not advance version and generation: before=({}, {}), after=({}, {})",
            before.version, before.layout_generation, after.version, after.layout_generation
        ));
    }
    if model_payload(&before.anchor) != model_payload(&after.anchor) {
        return Err("lifecycle event changed the fixed anchor model".to_owned());
    }

    let before_axis_present = target_axis.is_some_and(|axis| before.axes.contains_key(&axis));
    let after_axis_present = target_axis.is_some_and(|axis| after.axes.contains_key(&axis));
    match case.kind {
        LifecycleKind::RegisterSentinel => {
            if after.layout.sentinel_slots_width <= before.layout.sentinel_slots_width {
                return Err("Sentinel registration did not extend the published slot block".to_owned());
            }
        }
        LifecycleKind::DeregisterSentinel => {
            if after.layout.sentinel_slots_width >= before.layout.sentinel_slots_width {
                return Err("Sentinel deregistration did not marginalise the published slot block".to_owned());
            }
        }
        LifecycleKind::RegisterAxis => {
            if before_axis_present || !after_axis_present {
                return Err(format!(
                    "registered axis presence is before={before_axis_present}, after={after_axis_present}"
                ));
            }
        }
        LifecycleKind::DeregisterAxis => {
            if !before_axis_present || after_axis_present {
                return Err(format!(
                    "deregistered axis presence is before={before_axis_present}, after={after_axis_present}"
                ));
            }
        }
        LifecycleKind::RegisterIdentity => {
            if after.layout.identity_dimensions_width <= before.layout.identity_dimensions_width {
                return Err("identity registration did not extend the published identity block".to_owned());
            }
        }
        LifecycleKind::DeregisterIdentity => {
            if after.layout.identity_dimensions_width >= before.layout.identity_dimensions_width {
                return Err("identity deregistration did not marginalise the published identity block".to_owned());
            }
        }
    }

    let before_width = before.layout.dimension_map_width;
    let after_width = after.layout.dimension_map_width;
    for (model, before_block, after_block) in [
        (ModelId::Operational, &before.operational, &after.operational),
        (ModelId::Sister, &before.sister, &after.sister),
    ] {
        if before_block.dimension != before_width || after_block.dimension != after_width {
            return Err(format!(
                "model-set transition left {model:?} at {} -> {} against layouts {before_width} -> {after_width}",
                before_block.dimension, after_block.dimension
            ));
        }
    }
    for (&axis, before_block) in &before.axes {
        if let Some(after_block) = after.axes.get(&axis)
            && (before_block.dimension != before_width || after_block.dimension != after_width)
        {
            return Err(format!(
                "axis model {axis:?} did not follow full-space transition: {} -> {} against {before_width} -> {after_width}",
                before_block.dimension, after_block.dimension
            ));
        }
    }
    if before.base_slot_width > before.layout.sentinel_slots_width || after.base_slot_width > after.layout.sentinel_slots_width {
        return Err("base slot escaped the published slot block".to_owned());
    }
    Ok(())
}

fn check_axis_lifecycle_case(case: &AxisLifecycleCase) -> Result<(), String> {
    const EXISTING_AXIS_NAMES: [&str; 3] = ["existing-axis-0", "existing-axis-1", "existing-axis-2"];
    const RECOMPUTE_LABELS: u32 = 100;

    let mut config = base_config("axis-lifecycle-sweep");
    config.cholesky.n_recompute = RECOMPUTE_LABELS;
    let mut world = World::builder(config)
        .channel(CHANNEL, ChannelPolicy::default())
        .seed(0xA715_11FE_0000_0000 ^ case.entity)
        .build()
        .map_err(|error| format!("complete WorldBuilder construction failed: {error}"))?;
    attach_golden_reporting_sentinel(&mut world, BASE_SENTINEL);
    let mut existing_axes = Vec::new();
    for name in EXISTING_AXIS_NAMES.into_iter().take(case.existing_axes) {
        existing_axes.push(
            world
                .register_axis(name, false)
                .map_err(|error| format!("existing axis {name} registration failed: {error}"))?,
        );
    }
    let seed_assessment = world.assess(world.request_with_sentinel(
        CHANNEL,
        &format!("axis-seed-{}", case.entity),
        BASE_SENTINEL,
        u128::from(case.entity),
    ));
    let mut seed_label = LabelSpec::new(seed_assessment.id).valence(case.first_outcome);
    for &axis in &existing_axes {
        seed_label = seed_label.outcome(axis, case.first_outcome);
    }
    world
        .label(seed_label.ground_truth().build())
        .map_err(|error| format!("pre-registration training label failed: {error}"))?;
    world
        .flush_labels()
        .map_err(|error| format!("pre-registration training publication failed: {error}"))?;
    let before_registration = capture_published_state(&world, &existing_axes)?;
    let target = world
        .register_axis("target-axis", case.spatial)
        .map_err(|error| format!("target axis registration failed: {error}"))?;
    let mut all_axes = existing_axes.clone();
    all_axes.push(target);
    let after_registration = capture_published_state(&world, &all_axes)?;
    let added = after_registration.layout.dimension_map_width - before_registration.layout.dimension_map_width;
    if after_registration.base_slot_width != after_registration.layout.sentinel_slots_width {
        return Err(format!(
            "axis oracle requires one Sentinel slot, found base width {} inside total width {}",
            after_registration.base_slot_width, after_registration.layout.sentinel_slots_width
        ));
    }
    let slot_start = after_registration.layout.bias_width
        + after_registration.layout.aggregate_width
        + after_registration.layout.identity_dimensions_width
        + after_registration.layout.identity_cross_dimension_width
        + after_registration.layout.signal_width;
    let target_start = slot_start + before_registration.base_slot_width - 1;
    let prior_variance = 1.0 / base_config("axis-lifecycle-sweep").model.lambda_prior;

    check_exact_extension(
        &before_registration.operational,
        &after_registration.operational,
        target_start,
        added,
        prior_variance,
    )?;
    check_exact_extension(
        &before_registration.sister,
        &after_registration.sister,
        target_start,
        added,
        prior_variance,
    )?;
    for axis in &existing_axes {
        check_exact_extension(
            &before_registration.axes[axis],
            &after_registration.axes[axis],
            target_start,
            added,
            prior_variance,
        )?;
    }
    if !after_registration.axes.contains_key(&target) {
        return Err("target axis model was not constructed in the registration publication".to_owned());
    }
    if model_payload(&before_registration.anchor) != model_payload(&after_registration.anchor) {
        return Err("axis registration changed the fixed anchor model".to_owned());
    }

    for label_index in 0..RECOMPUTE_LABELS - 1 {
        let outcome = if label_index % 2 == 0 {
            case.first_outcome
        } else {
            case.second_outcome
        };
        let assessment = world.assess(world.request_with_sentinel(
            CHANNEL,
            &format!("axis-{label_index}-{}", case.entity),
            BASE_SENTINEL,
            u128::from(case.entity),
        ));
        world
            .label(
                LabelSpec::new(assessment.id)
                    .valence(outcome)
                    .outcome(target, outcome)
                    .ground_truth()
                    .build(),
            )
            .map_err(|error| format!("axis training label {label_index} failed: {error}"))?;
        world
            .flush_labels()
            .map_err(|error| format!("axis training publication {label_index} failed: {error}"))?;
    }

    let before_deregistration = capture_published_state(&world, &all_axes)?;
    let removed: Vec<_> = (target_start..target_start + added).collect();
    let kept: Vec<_> = (0..before_deregistration.layout.dimension_map_width)
        .filter(|index| removed.binary_search(index).is_err())
        .collect();
    let delta = base_config("axis-lifecycle-sweep")
        .schur
        .delta(base_config("axis-lifecycle-sweep").model.lambda_prior);
    let mut expected_covariances = HashMap::new();
    for block in std::iter::once(&before_deregistration.operational)
        .chain(std::iter::once(&before_deregistration.sister))
        .chain(existing_axes.iter().map(|axis| &before_deregistration.axes[axis]))
    {
        let precision = invert_independently(&matrix_from_covariance(block))?;
        let marginal_precision = regularised_schur_complement(&precision, &removed, delta)
            .ok_or_else(|| format!("independent Schur oracle refused model {:?}", block.model))?;
        let expected_covariance = invert_independently(&marginal_precision)?;
        expected_covariances.insert(block.model.clone(), expected_covariance);
    }

    world
        .deregister_axis("target-axis")
        .map_err(|error| format!("target axis deregistration failed: {error}"))?;
    let after_deregistration = capture_published_state(&world, &all_axes)?;
    if after_deregistration.axes.contains_key(&target) {
        return Err("target axis model survived deregistration".to_owned());
    }
    if model_payload(&before_deregistration.anchor) != model_payload(&after_deregistration.anchor) {
        return Err("axis deregistration changed the fixed anchor model".to_owned());
    }

    for block in std::iter::once(&after_deregistration.operational)
        .chain(std::iter::once(&after_deregistration.sister))
        .chain(existing_axes.iter().map(|axis| &after_deregistration.axes[axis]))
    {
        let expected = expected_covariances
            .get(&block.model)
            .ok_or_else(|| format!("no independent marginal covariance for model {:?}", block.model))?;
        let before = match block.model {
            ModelId::Operational => &before_deregistration.operational,
            ModelId::Sister => &before_deregistration.sister,
            ModelId::OutcomeAxis(axis) => &before_deregistration.axes[&axis],
            ModelId::Anchor => return Err("anchor entered the full-space marginalisation set".to_owned()),
        };
        let tolerance = delta + DEFAULT_TOLERANCES.precision_sync_ceiling(before.dimension);
        compare_matrix(block, expected, tolerance).map_err(|error| {
            format!(
                "{error}; removed={removed:?}, registration={:?}->{:?}, deregistration={:?}->{:?}",
                before_registration.layout, after_registration.layout, before_deregistration.layout, after_deregistration.layout
            )
        })?;
        if block.mean.len() != kept.len() {
            return Err(format!(
                "model {:?} mean width did not shrink by {}",
                block.model,
                removed.len()
            ));
        }
        for (after_index, &before_index) in kept.iter().enumerate() {
            if block.mean[after_index].to_bits() != before.mean[before_index].to_bits() {
                return Err(format!(
                    "model {:?} changed retained mean {before_index} at compacted index {after_index}",
                    block.model
                ));
            }
        }
    }
    Ok(())
}

fn check_staleness_case(case: &StalenessCase) -> Result<(), String> {
    let world = build_world("publication-staleness-sweep", 0x57A1_E000_0000_0000 ^ case.elapsed_seconds)?;
    let initial = probe_model(&world, &ModelId::Operational)?;
    world
        .advance(Duration::from_secs(case.elapsed_seconds))
        .map_err(|error| format!("scenario-time advance failed: {error}"))?;
    let aged_probe = probe_model(&world, &ModelId::Operational)?;
    let aged = world.assess(world.request(CHANNEL, "aged"));
    let observed_delta = aged_probe.observed_at.hours_since(&initial.observed_at) * 3_600.0;
    if (observed_delta - case.elapsed_seconds as f64).abs() > 1e-9 {
        return Err(format!(
            "probe scenario time advanced by {observed_delta}s, expected {}s",
            case.elapsed_seconds
        ));
    }
    if aged.health.snapshot_version != aged_probe.published_version
        || (aged.health.snapshot_age_seconds - case.elapsed_seconds as f64).abs() > 1e-9
    {
        return Err(format!(
            "aged assessment saw version {} at age {}, expected version {} at age {}",
            aged.health.snapshot_version, aged.health.snapshot_age_seconds, aged_probe.published_version, case.elapsed_seconds
        ));
    }

    let label = if case.adverse {
        LabelSpec::adverse(aged.id)
    } else {
        LabelSpec::benign(aged.id)
    };
    world
        .label(label.ground_truth().build())
        .map_err(|error| format!("freshening label failed: {error}"))?;
    world
        .flush_labels()
        .map_err(|error| format!("freshening publication barrier failed: {error}"))?;
    let fresh_probe = probe_model(&world, &ModelId::Operational)?;
    let fresh = world.assess(world.request(CHANNEL, "fresh"));
    if fresh_probe.published_version <= aged_probe.published_version {
        return Err(format!(
            "label publication did not advance version {} beyond {}",
            fresh_probe.published_version, aged_probe.published_version
        ));
    }
    if fresh.health.snapshot_version != fresh_probe.published_version
        || fresh.health.snapshot_age_seconds.to_bits() != 0.0_f64.to_bits()
    {
        return Err(format!(
            "fresh assessment saw version {} at age {}, expected version {} at zero age",
            fresh.health.snapshot_version, fresh.health.snapshot_age_seconds, fresh_probe.published_version
        ));
    }
    Ok(())
}

/// Every drawn assessment changes exactly the enumerated measurement state, leaves the label-authorised state bit-identical, and records the complete request in the pending probe.
///
/// The generator draws twenty-four finite entity-persistent scalar signals from `[0, 1)`, positive scenario-time advances from one second through one minute, and distinct entity keys. Every case uses the complete builder, first completes a two-observation cold ramp, then leaves a ten-observation late-Sentinel bootstrap nine observations complete before the drawn assessment. That forced setup reaches both observational acquisition mechanisms; a registered identity, a reporting Sentinel, the concordance window, the signal cache and the pending buffer make every enumerated assessment-side mutation non-vacuous.
///
/// (´inv:guarantee:assess-only´)
/// (´inv:guarantee:evidence-authority´)
/// (´inv:runtime:enumerated-writes´)
/// ´claim:assess:one-assessment-mutates-every-and-only-enumerated-measurement-state´
/// ´test:crate:assessment-write-set-seeded-sweep´
#[test]
fn assessment_write_set_seeded_sweep() {
    const CASES: usize = 24;
    const SEED: u64 = 0xA55E_5500_5EED_0018;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| AssessmentWriteCase {
            elapsed_seconds: rng.next_range(60) + 1,
            signal: rng.next_f64(),
            entity: rng.next_u64(),
        },
        check_assessment_write_case,
    );
}

/// Every lifecycle direction presents one complete pre-event snapshot to an assessment begun while publication is held and one complete post-event model set after the named barriers.
///
/// The generator cycles registration and deregistration for Sentinels, outcome axes and identity dimensions six times each across thirty-six cases while drawing axis spatiality, entity keys and positive scenario-time advances. Every case carries a live Sentinel and axis model, while destructive cases also start with their target live. The cycle therefore reaches every model-set rule and every destructive direction by construction rather than probability; the owner block fixes the pre-publication assessment, and the identity, lifecycle, model, slot and layout barriers and probes verify one settled version and one dimension semantics after release.
///
/// (´inv:guarantee:lifecycle-publication´)
/// (´inv:dimension:version-consistency´)
/// (´inv:registry:model-set-serialisation´)
/// ´claim:lifecycle:every-registry-direction-publishes-one-whole-before-and-after-model-set´
/// ´test:crate:lifecycle-publication-seeded-sweep´
#[test]
fn lifecycle_publication_seeded_sweep() {
    const CASES: usize = 36;
    const SEED: u64 = 0x11FE_C7CE_5EED_0024;
    let mut draw_index = 0;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| {
            let case = LifecycleCase {
                kind: LifecycleKind::from_index(draw_index),
                spatial: rng.coin(),
                elapsed_seconds: rng.next_range(60) + 1,
                entity: rng.next_u64(),
            };
            draw_index += 1;
            case
        },
        check_lifecycle_case,
    );
}

/// Outcome-axis registration is exact extension for every existing full-space model, and deregistration applies the independent regularised-Schur result to every survivor while leaving the anchor fixed.
///
/// The generator draws twenty-four worlds, alternates spatial and non-spatial target axes, varies the existing-axis population from none through two, and draws two outcomes and an entity key. Every case trains the existing model set before registering and deregistering the target, so exact extension observes non-prior means and covariance rather than interchangeable diagonal cells. Spatial cases add live Sentinel coordinates and alternate the drawn outcomes through the remainder of one full minimum recomputation cadence so the removed block participates in a non-trivial posterior and the published covariance is an observable inverse for the precision that the probe contract deliberately withholds. The transition-derived removal set preserves the stable batch tail, and the independent covariance-to-precision, Schur and precision-to-covariance route checks the destructive transition within the configured Schur offset plus the published inverse-pair synchronisation ceiling; non-spatial cases exercise the zero-width model-set edge.
///
/// (´inv:guarantee:axis-lifecycle´)
/// ´claim:lifecycle:axis-registration-exactly-extends-and-deregistration-regularised-marginalises-every-surviving-full-space-model´
/// ´test:crate:axis-lifecycle-seeded-sweep´
#[test]
fn axis_lifecycle_seeded_sweep() {
    const CASES: usize = 24;
    const SEED: u64 = 0xA715_11FE_5EED_0018;
    let mut draw_index = 0;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| {
            let case = AxisLifecycleCase {
                spatial: draw_index % 2 == 0,
                existing_axes: usize::try_from(rng.next_range(3)).unwrap_or(0),
                first_outcome: rng.next_f64().mul_add(2.0, -1.0),
                second_outcome: rng.next_f64().mul_add(2.0, -1.0),
                entity: rng.next_u64(),
            };
            draw_index += 1;
            case
        },
        check_axis_lifecycle_case,
    );
}

/// A positive scenario-time advance appears exactly as publication age, and the next label publication resets that age to zero at a later version.
///
/// The generator draws thirty-two positive whole-second ages from one second through one day and both adverse and benign freshening labels. Every case therefore enters the stale region before crossing the label barrier into a fresh publication; model probes record scenario-owned observation time on both sides, so elapsed execution time cannot satisfy the predicate.
///
/// (´inv:guarantee:staleness´)
/// ´claim:lifecycle:publication-age-follows-scenario-time-and-resets-on-the-next-publication´
/// ´test:crate:publication-staleness-seeded-sweep´
#[test]
fn publication_staleness_seeded_sweep() {
    const CASES: usize = 32;
    const SEED: u64 = 0x57A1_EA55_5EED_0020;

    run_seeded_sweep(
        SEED,
        CASES,
        |rng| StalenessCase {
            elapsed_seconds: rng.next_range(86_400) + 1,
            adverse: rng.coin(),
        },
        check_staleness_case,
    );
}
