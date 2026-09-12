// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`hibernated_sentinel_returns_aged_on_both_clocks`] | lifespan | A Sentinel that is deregistered with hibernation and registered again gets its own slot weights back, aged by the product of the two clocks — the time-indexed rate over the elapsed hours and each model's own label-indexed rate over the labels processed meanwhile — rather than reset to the prior. The mean comes back untouched and the covariance comes back inflated by the reciprocal of the same factor, which is what a loss of confidence in an unchanged estimate is. The operational and sister models age by different amounts because they forget at different rates, so a restore that applied one factor to both would quietly re-tune the sister. |
//! | [`restored_sentinel_cross_feature_blocks_are_zero`] | lifespan | Nothing coupling the returning Sentinel to anything else comes back: every cross-feature entry of the precision and the covariance between the restored slot and the rest of the vector is exactly zero, however strongly the two were correlated before the Sentinel left. Extension writes zero off-diagonal blocks and the archive carries no couplings to overwrite them with, so the correlation is relearned from nothing. |
//! | [`expired_hibernation_archive_is_refused`] | lifespan | An archive older than the configured lifetime is refused, and the registration extends at the prior instead. The expiry is wall-clock and nothing else consults it: the same record one hour on the near side of the boundary is restored, so the refusal is the boundary rather than a general failure to restore. |
//! | [`re_registration_ends_the_hibernation_archive`] | lifespan | Registering the identifier again ends the archive's usability whatever the registration did with the record: the record is taken from the archive rather than read, so a Sentinel that returns, is deregistered destructively and returns a second time comes back at the prior. Without that, one hibernation would keep answering every later registration of the same identifier for as long as the lifetime allowed. |
//! | [`hibernation_refuses_a_changed_arrangement`] | lifespan | A registration creating a differently shaped block is refused rather than written over: a Sentinel hibernated with no spatial outcome axes registered returns into a slot two positions wider once one has been, and the archived block describes positions that arrangement does not have. The registration takes the prior, which is the fallback the algorithm names for a configuration that has changed. |
//! | [`hibernated_outcome_axis_returns_with_its_own_block`] | lifespan | An outcome axis hibernates under the same protocol, with the same scope: the positions it holds inside every identity dimension's block come back aged, while its own prediction model — destroyed when it left — is created fresh at the prior. The axis's beliefs over the whole feature space are not preserved by hibernation and were never claimed to be; what is preserved is the block its own features occupy in the models that survived its absence. |
//! | [`identity_dimensions_carry_no_hibernation_payload`] | lifespan | An identity dimension carries no hibernation payload at all: deregistering one and registering it again leaves the archive empty and the dimension's block at the prior, whatever it held before. The specification's hibernation names a Sentinel's block and, by reference, an axis's; it names nothing for an identity dimension, and a payload invented for it would be design rather than derivation. |
//! | [`hibernation_archive_crosses_a_checkpoint`] | persistence | The archive crosses a checkpoint whole, in the encoding the checkpoint already uses: a record written before the restart is the same record afterwards, generation, clocks, layout and every model's block included. The archive is keyed by identifiers a host will register again, so one that did not survive the restart would turn every later return into a silent extension at the prior. |
//! | [`hibernation_admits_a_block_on_its_spectrum`] | lifespan | The archive's admission reads its blocks' spectra where nothing between the file and the model read anything but a diagonal, and the witness is the suite's own: ones on the diagonal and twos off it, whose minimum diagonal is strictly positive and whose least eigenvalue is minus one. Both halves stand here — the floor test admits the block at any floor below one, and the verdict refuses it naming the pivot — because the whole content of the repair is that the two tests answer different questions and only one of them is about definiteness. The floor test itself is untouched: it says whether an entity has been forgotten, which an indefinite block is not. |
//! | [`indefinite_archived_block_leaves_the_registration_at_the_prior`] | lifespan | A registration meeting an archive whose block is indefinite extends at the prior, exactly as it does for a record that is expired or laid out differently. The refusal changes nothing about the disposition: the arm the lifecycle already had logs it and lets the extension stand, and the prior is a definite block by construction, so the alternative to the archived block is always a matrix the model may hold. That is what makes refusing here cheap — the fallback existed before the verdict did. |
//! | [`the_floor_splits_one_record_between_two_models`] | lifespan | The replenishment floor is decided per model rather than per record, and one record is answered both ways at once. Two models hold the same archived block and age it on their own label-indexed rates; across twenty thousand labels the operational rate takes its copy below the floor while the slower sister rate leaves its copy well above it, and the restore reports one model restored and one at the prior. A record-level refusal could not express this: a single verdict would have to be wrong about one of the two, which is why the floor is not among the admission refusals and its answer is a count. The wall clock is held still so that what splits the record is the label rates alone. |

//! Hibernation crate-level tests: what a hibernating deregistration keeps,
//! what a re-registration gets back, and on what terms
//! (´alg:registry:hibernation´), (´cav:limitation:hibernation´).
//!
//! Every test here drives the steward's own lifecycle handler with an injected
//! wall clock, because the restore's two questions — has the archive expired,
//! and how far has the block aged — are both questions about elapsed time, and
//! a test that could not move the clock could ask neither.
//!
//! # Cross-References
//!
//! - (´alg:registry:hibernation´) — the archived payload and the restore's ageing
//! - (´cav:limitation:hibernation´) — the cross-feature structure that is not kept
//! - (´rem:registry:axis-hibernation´) — an outcome axis under the same protocol
//! - (´entry:construction:hibernation´) — the archive contract these tests hold

use std::collections::HashMap;

use faer::Col;

use crate::config::types::AssayerConfig;
use crate::health::{IdentityConvergenceTracker, PlattConvergenceTracker};
use crate::ledger::OutcomeLedger;
use crate::linalg::symmetric::SymmetricMatrix;
use crate::model::hibernation::HibernatedBlock;
use crate::owner::commands::{LifecycleEvent, LifecycleSubmission};
use crate::owner::lifecycle::handle_lifecycle_submission;
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::{ModelConfig, WorkingCopy};
use crate::testing::registrations::{axis_reg, sentinel_reg, spatial_axis_reg};
use crate::types::{DimensionId, ModelId, OutcomeAxisId, PersistentTimestamp, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Test Infrastructure
// ═══════════════════════════════════════════════════════════════════════════════

/// Seconds in an hour, for writing the injected clock in the units the decay
/// is expressed in.
const SECONDS_PER_HOUR: i64 = 3_600;

/// A lifecycle environment whose wall clock the test supplies per submission.
struct HibernationEnv {
    working: WorkingCopy,
    shared: SharedState,
    platt_tracker: PlattConvergenceTracker,
    identity_trackers: HashMap<DimensionId, IdentityConvergenceTracker>,
    outcome_ledger: OutcomeLedger,
    config: AssayerConfig,
    version: u64,
}

impl HibernationEnv {
    fn new() -> Self {
        let model_config = ModelConfig::default();
        let working = WorkingCopy::cold_start(16, &model_config, 1000);
        let snapshot = working.to_snapshot(0);
        let shared = SharedState::new(snapshot);

        Self {
            working,
            shared,
            platt_tracker: PlattConvergenceTracker::new(),
            identity_trackers: HashMap::new(),
            outcome_ledger: OutcomeLedger::new(),
            config: AssayerConfig::default(),
            version: 0,
        }
    }

    /// Submits a batch at a stated wall-clock reading, in hours from the
    /// instance's own zero.
    fn submit_at(&mut self, events: Vec<LifecycleEvent>, at_hours: i64) {
        self.version += 1;
        let submission = LifecycleSubmission {
            events,
            completion: None,
        };
        let result = handle_lifecycle_submission(
            &mut self.working,
            submission,
            &self.shared,
            &mut self.platt_tracker,
            &mut self.identity_trackers,
            &self.outcome_ledger,
            &self.config,
            &mut self.version,
            std::time::Instant::now(),
            &PersistentTimestamp::new(at_hours * SECONDS_PER_HOUR, 0),
        );
        drop(result.expect("lifecycle submission should succeed"));
    }

    /// The half-open index range a Sentinel's slot occupies.
    fn slot(&self, id: SentinelId) -> std::ops::Range<usize> {
        self.working
            .dimension_map
            .sentinel_slots
            .get(&id)
            .expect("the Sentinel has a slot")
            .to_range()
    }

    /// The three positions an outcome axis holds inside the one registered
    /// identity dimension's block.
    fn axis_identity_positions(&self, axis: OutcomeAxisId, dimension: DimensionId) -> std::ops::Range<usize> {
        let offset = self
            .working
            .dimension_map
            .identity_axis_offset(axis)
            .expect("the axis has an identity offset");
        let start = self.working.dimension_map.id_dim_ranges[&dimension].start;
        (start + offset)..(start + offset + 3)
    }

    /// Writes a distinguishable block over `positions` in the operational and
    /// sister models, so what comes back from the archive can be told from
    /// what an extension at the prior would have left.
    fn seed_block(&mut self, positions: &std::ops::Range<usize>, precision: f64, covariance: f64) -> HibernatedBlock {
        let width = positions.len();
        #[allow(clippy::cast_precision_loss)] // Justified: test widths are tens of positions.
        let block = HibernatedBlock {
            mean: (0..width).map(|i| 0.5 + i as f64).collect(),
            precision: SymmetricMatrix::identity_scaled(width, precision),
            covariance: SymmetricMatrix::identity_scaled(width, covariance),
        };
        self.working
            .operational
            .restore_self_structure_block(positions.start, &block, 1.0);
        self.working.sister.restore_self_structure_block(positions.start, &block, 1.0);
        block
    }
}

/// The diagonal of a model's precision at one position.
fn precision_at(model: &crate::model::bayesian::BayesianLinearModel, index: usize) -> f64 {
    model.precision().as_inner()[(index, index)]
}

/// The diagonal of a model's covariance at one position.
fn covariance_at(model: &crate::model::bayesian::BayesianLinearModel, index: usize) -> f64 {
    model.covariance().as_inner()[(index, index)]
}

// ═══════════════════════════════════════════════════════════════════════════════
// The round trip and its ageing
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel that is deregistered with hibernation and registered again gets
/// its own slot weights back, aged by the product of the two clocks — the
/// time-indexed rate over the elapsed hours and each model's own label-indexed
/// rate over the labels processed meanwhile — rather than reset to the prior.
/// The mean comes back untouched and the covariance comes back inflated by the
/// reciprocal of the same factor, which is what a loss of confidence in an
/// unchanged estimate is. The operational and sister models age by different
/// amounts because they forget at different rates, so a restore that applied
/// one factor to both would quietly re-tune the sister.
///
/// ´claim:lifespan:a-hibernating-sentinel-returns-with-its-own-weights-aged-on-both-clocks´
/// ´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´
#[test]
fn hibernated_sentinel_returns_aged_on_both_clocks() {
    let mut env = HibernationEnv::new();
    env.config.temporal.gamma_t_core = 0.9;
    env.config.model.gamma_opr = 0.99;
    env.config.model.gamma_inh = 0.95;

    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    let seeded = env.seed_block(&slot, 4.0, 0.25);

    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    assert!(
        env.working.hibernation.holds_sentinel(SentinelId(1)),
        "a hibernating deregistration leaves a record behind"
    );

    // Ten hours and five labels later.
    env.working.last_processed_label_seq = 5;
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 10);

    let restored = env.slot(SentinelId(1));
    assert_eq!(restored.len(), slot.len(), "the slot returns at the same width");

    let time_factor = 0.9_f64.powi(10);
    let operational_factor = time_factor * 0.99_f64.powi(5);
    let sister_factor = time_factor * 0.95_f64.powi(5);

    for (offset, index) in restored.enumerate() {
        assert!(
            (env.working.operational.mu()[index] - seeded.mean[offset]).abs() < 1e-12,
            "position {offset}: the mean returns unaged, expected {}, got {}",
            seeded.mean[offset],
            env.working.operational.mu()[index],
        );
        let expected_precision = 4.0 * operational_factor;
        assert!(
            (precision_at(&env.working.operational, index) - expected_precision).abs() < 1e-12,
            "position {offset}: operational precision should be {expected_precision}, got {}",
            precision_at(&env.working.operational, index),
        );
        let expected_covariance = 0.25 / operational_factor;
        assert!(
            (covariance_at(&env.working.operational, index) - expected_covariance).abs() < 1e-9,
            "position {offset}: operational covariance should be {expected_covariance}, got {}",
            covariance_at(&env.working.operational, index),
        );
        let expected_sister = 4.0 * sister_factor;
        assert!(
            (precision_at(&env.working.sister, index) - expected_sister).abs() < 1e-12,
            "position {offset}: sister precision should be {expected_sister}, got {}",
            precision_at(&env.working.sister, index),
        );
    }

    assert!(
        (operational_factor - sister_factor).abs() > 1e-3,
        "the two models age by different amounts, or this test proves nothing about the rates"
    );
}

/// Nothing coupling the returning Sentinel to anything else comes back: every
/// cross-feature entry of the precision and the covariance between the
/// restored slot and the rest of the vector is exactly zero, however strongly
/// the two were correlated before the Sentinel left. Extension writes zero
/// off-diagonal blocks and the archive carries no couplings to overwrite them
/// with, so the correlation is relearned from nothing.
///
/// ´claim:lifespan:a-restored-sentinels-cross-feature-blocks-come-back-at-exactly-zero´
/// ´test:crate:restored-sentinel-cross-feature-blocks-are-zero´
#[test]
fn restored_sentinel_cross_feature_blocks_are_zero() {
    let mut env = HibernationEnv::new();
    env.submit_at(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
        ],
        0,
    );

    let first = env.slot(SentinelId(1));
    let second = env.slot(SentinelId(2));

    // One update on a vector that is on in both slots, which is what puts a
    // nonzero coupling between them into the precision.
    let dimension = env.working.operational.dim();
    let phi = Col::from_fn(
        dimension,
        |i| {
            if first.contains(&i) || second.contains(&i) { 1.0 } else { 0.0 }
        },
    );
    let (v, h) = env.working.operational.covariance().quadratic_form_with_product(&phi);
    env.working
        .operational
        .apply_leverage_bounded_update(&phi, &v, h, 1.0, 1.0, 1.0, 0.001);

    let coupling_before = env.working.operational.precision().as_inner()[(first.start, second.start)];
    assert!(
        coupling_before.abs() > 1e-6,
        "the fixture established a coupling to lose, got {coupling_before}"
    );

    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);

    let returned = env.slot(SentinelId(1));
    let survivor = env.slot(SentinelId(2));
    assert!(
        env.working.operational.precision().as_inner()[(returned.start, returned.start)] > 0.1,
        "the returning Sentinel's own block came back rather than standing at the prior"
    );

    for i in returned.clone() {
        for j in (0..env.working.operational.dim()).filter(|j| !returned.contains(j)) {
            assert!(
                env.working.operational.precision().as_inner()[(i, j)] == 0.0,
                "precision coupling ({i}, {j}) should be zero"
            );
            assert!(
                env.working.operational.covariance().as_inner()[(i, j)] == 0.0,
                "covariance coupling ({i}, {j}) should be zero"
            );
        }
    }

    assert!(
        env.working.operational.precision().as_inner()[(survivor.start, survivor.start)] > 0.0,
        "the surviving Sentinel keeps a usable precision of its own"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The restore window
// ═══════════════════════════════════════════════════════════════════════════════

/// An archive older than the configured lifetime is refused, and the
/// registration extends at the prior instead. The expiry is wall-clock and
/// nothing else consults it: the same record one hour on the near side of the
/// boundary is restored, so the refusal is the boundary rather than a general
/// failure to restore.
///
/// ´claim:lifespan:an-archive-past-its-configured-lifetime-is-refused-and-the-registration-takes-the-prior´
/// ´test:crate:expired-hibernation-archive-is-refused´
#[test]
fn expired_hibernation_archive_is_refused() {
    let expiry_hours = i64::from(AssayerConfig::default().hibernation.expiry_days) * 24;

    // Just inside the lifetime: the block comes back.
    let mut inside = HibernationEnv::new();
    inside.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = inside.slot(SentinelId(1));
    drop(inside.seed_block(&slot, 4.0, 0.25));
    inside.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    inside.submit_at(
        vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))],
        expiry_hours - 1,
    );
    let inside_slot = inside.slot(SentinelId(1));
    assert!(
        precision_at(&inside.working.operational, inside_slot.start) > inside.config.model.lambda_prior,
        "a record inside its lifetime is restored"
    );

    // One hour past it: the registration takes the prior.
    let mut outside = HibernationEnv::new();
    outside.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = outside.slot(SentinelId(1));
    drop(outside.seed_block(&slot, 4.0, 0.25));
    outside.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    outside.submit_at(
        vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))],
        expiry_hours + 1,
    );
    let outside_slot = outside.slot(SentinelId(1));
    assert!(
        (precision_at(&outside.working.operational, outside_slot.start) - outside.config.model.lambda_prior).abs() < 1e-12,
        "an expired record leaves the slot at the prior precision"
    );
    assert!(
        outside.working.operational.mu()[outside_slot.start].abs() < 1e-12,
        "an expired record leaves the slot's mean at zero"
    );
    assert!(
        outside.working.hibernation.is_empty(),
        "the expired record does not stay in the archive"
    );
}

/// Registering the identifier again ends the archive's usability whatever the
/// registration did with the record: the record is taken from the archive
/// rather than read, so a Sentinel that returns, is deregistered destructively
/// and returns a second time comes back at the prior. Without that, one
/// hibernation would keep answering every later registration of the same
/// identifier for as long as the lifetime allowed.
///
/// ´claim:lifespan:a-re-registration-consumes-the-archive-so-a-second-return-comes-back-at-the-prior´
/// ´test:crate:re-registration-ends-the-hibernation-archive´
#[test]
fn re_registration_ends_the_hibernation_archive() {
    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    drop(env.seed_block(&slot, 4.0, 0.25));

    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);

    let first_return = env.slot(SentinelId(1));
    assert!(
        precision_at(&env.working.operational, first_return.start) > env.config.model.lambda_prior,
        "the first return takes the archived block"
    );
    assert!(
        env.working.hibernation.is_empty(),
        "the registration consumed the record rather than leaving it readable"
    );

    // A destructive removal archives nothing, so the second return has
    // nothing to take.
    env.submit_at(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 0);
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);

    let second_return = env.slot(SentinelId(1));
    assert!(
        (precision_at(&env.working.operational, second_return.start) - env.config.model.lambda_prior).abs() < 1e-12,
        "the second return stands at the prior"
    );
}

/// A registration creating a differently shaped block is refused rather than
/// written over: a Sentinel hibernated with no spatial outcome axes registered
/// returns into a slot two positions wider once one has been, and the archived
/// block describes positions that arrangement does not have. The registration
/// takes the prior, which is the fallback the algorithm names for a
/// configuration that has changed.
///
/// ´claim:lifespan:an-archive-whose-arrangement-has-changed-is-refused-rather-than-written-over´
/// ´test:crate:hibernation-refuses-a-changed-arrangement´
#[test]
fn hibernation_refuses_a_changed_arrangement() {
    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    let width_before = slot.len();
    drop(env.seed_block(&slot, 4.0, 0.25));

    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);
    env.submit_at(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        0,
    );
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);

    let returned = env.slot(SentinelId(1));
    assert!(
        returned.len() > width_before,
        "the spatial axis widened the slot, which is what makes the arrangement different"
    );
    assert!(
        (precision_at(&env.working.operational, returned.start) - env.config.model.lambda_prior).abs() < 1e-12,
        "a block from another arrangement is refused and the slot stands at the prior"
    );
    assert!(
        env.working.hibernation.is_empty(),
        "the refused record is consumed by the registration all the same"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The other two registries
// ═══════════════════════════════════════════════════════════════════════════════

/// An outcome axis hibernates under the same protocol, with the same scope:
/// the positions it holds inside every identity dimension's block come back
/// aged, while its own prediction model — destroyed when it left — is created
/// fresh at the prior. The axis's beliefs over the whole feature space are not
/// preserved by hibernation and were never claimed to be; what is preserved is
/// the block its own features occupy in the models that survived its absence.
///
/// ´claim:lifespan:a-hibernating-outcome-axis-returns-with-its-own-block-and-a-fresh-prediction-model´
/// ´test:crate:hibernated-outcome-axis-returns-with-its-own-block´
#[test]
fn hibernated_outcome_axis_returns_with_its_own_block() {
    let mut env = HibernationEnv::new();
    env.config.temporal.gamma_t_core = 0.9;
    env.config.model.gamma_opr = 0.99;

    env.submit_at(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 0);
    env.submit_at(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 0);

    let positions = env.axis_identity_positions(OutcomeAxisId(1), DimensionId(1));
    let seeded = env.seed_block(&positions, 4.0, 0.25);

    env.submit_at(vec![LifecycleEvent::HibernateOutcomeAxis(OutcomeAxisId(1))], 0);
    assert!(
        env.working.hibernation.holds_axis(OutcomeAxisId(1)),
        "a hibernating axis deregistration leaves a record behind"
    );

    env.working.last_processed_label_seq = 5;
    env.submit_at(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 10);

    let returned = env.axis_identity_positions(OutcomeAxisId(1), DimensionId(1));
    let factor = 0.9_f64.powi(10) * 0.99_f64.powi(5);
    for (offset, index) in returned.enumerate() {
        assert!(
            (env.working.operational.mu()[index] - seeded.mean[offset]).abs() < 1e-12,
            "position {offset}: the axis's mean returns unaged",
        );
        let expected = 4.0 * factor;
        assert!(
            (precision_at(&env.working.operational, index) - expected).abs() < 1e-12,
            "position {offset}: the axis's precision should be {expected}, got {}",
            precision_at(&env.working.operational, index),
        );
    }

    // The axis's own prediction model is new, and new means at the prior the
    // registration named across its whole width
    // (´alg:registry:axis-registration´) — an axis model's prior is its own
    // declared initial scale rather than the core models' λ.
    let axis_model = &env.working.outcome_models[&OutcomeAxisId(1)];
    for index in 0..axis_model.model.dim() {
        assert!(
            (precision_at(&axis_model.model, index) - axis_model.kappa_a).abs() < 1e-12,
            "the fresh axis model stands at its declared prior at position {index}",
        );
    }
}

/// An identity dimension carries no hibernation payload at all: deregistering
/// one and registering it again leaves the archive empty and the dimension's
/// block at the prior, whatever it held before. The specification's
/// hibernation names a Sentinel's block and, by reference, an axis's; it names
/// nothing for an identity dimension, and a payload invented for it would be
/// design rather than derivation.
///
/// ´claim:lifespan:an-identity-dimension-carries-no-hibernation-payload-and-is-relearned´
/// ´test:crate:identity-dimensions-carry-no-hibernation-payload´
#[test]
fn identity_dimensions_carry_no_hibernation_payload() {
    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 0);

    let block = env.working.dimension_map.id_dim_ranges[&DimensionId(1)].to_range();
    drop(env.seed_block(&block, 4.0, 0.25));

    env.submit_at(vec![LifecycleEvent::DeregisterIdentityDimension(DimensionId(1))], 0);
    assert!(
        env.working.hibernation.is_empty(),
        "an identity deregistration archives nothing"
    );

    env.submit_at(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 0);
    let returned = env.working.dimension_map.id_dim_ranges[&DimensionId(1)].to_range();
    for index in returned {
        assert!(
            (precision_at(&env.working.operational, index) - env.config.model.lambda_prior).abs() < 1e-12,
            "the returning dimension's position {index} stands at the prior",
        );
        assert!(
            env.working.operational.mu()[index].abs() < 1e-12,
            "the returning dimension's position {index} has a mean of zero",
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Custody
// ═══════════════════════════════════════════════════════════════════════════════

/// The archive crosses a checkpoint whole, in the encoding the checkpoint
/// already uses: a record written before the restart is the same record
/// afterwards, generation, clocks, layout and every model's block included.
/// The archive is keyed by identifiers a host will register again, so one that
/// did not survive the restart would turn every later return into a silent
/// extension at the prior.
///
/// ´claim:persistence:the-hibernation-archive-crosses-a-checkpoint-whole´
/// ´test:crate:hibernation-archive-crosses-a-checkpoint´
#[cfg(feature = "serde")]
#[test]
fn hibernation_archive_crosses_a_checkpoint() {
    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    drop(env.seed_block(&slot, 4.0, 0.25));
    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);

    let before = env
        .working
        .hibernation
        .sentinel(SentinelId(1))
        .expect("the record is in the archive")
        .clone();

    let payload = env
        .working
        .to_checkpoint_payload(PersistentTimestamp::new(0, 0), Vec::new(), Vec::new());
    let bytes = crate::serde_codec::serialize(&payload).expect("the payload serialises");
    let restored: crate::persistence::checkpoint::CheckpointPayload =
        crate::serde_codec::deserialize(&bytes).expect("the payload deserialises");

    let after = restored
        .hibernation
        .sentinel(SentinelId(1))
        .expect("the record crossed the checkpoint");

    assert_eq!(after.version, before.version, "the record's generation crosses");
    assert_eq!(after.layout, before.layout, "the record's layout crosses");
    assert_eq!(
        after.archived_label_seq, before.archived_label_seq,
        "the record's label clock crosses"
    );
    assert_eq!(after.blocks.len(), before.blocks.len(), "every model's block crosses");
    for ((before_id, before_block), (after_id, after_block)) in before.blocks.iter().zip(after.blocks.iter()) {
        assert_eq!(before_id, after_id, "the blocks cross in order, by model");
        assert_eq!(before_block.mean, after_block.mean, "the block's mean crosses");
        for i in 0..before_block.precision.dim() {
            assert!(
                (before_block.precision.as_inner()[(i, i)] - after_block.precision.as_inner()[(i, i)]).abs() < 1e-15,
                "the block's precision crosses at {i}",
            );
            assert!(
                (before_block.covariance.as_inner()[(i, i)] - after_block.covariance.as_inner()[(i, i)]).abs() < 1e-15,
                "the block's covariance crosses at {i}",
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// The definiteness verdict on an archived block
// ═══════════════════════════════════════════════════════════════════════════════

/// The archive's admission reads its blocks' spectra where nothing between the
/// file and the model read anything but a diagonal, and the witness is the
/// suite's own: ones on the diagonal and twos off it, whose minimum diagonal is
/// strictly positive and whose least eigenvalue is minus one. Both halves stand
/// here — the floor test admits the block at any floor below one, and the
/// verdict refuses it naming the pivot — because the whole content of the
/// repair is that the two tests answer different questions and only one of them
/// is about definiteness. The floor test itself is untouched: it says whether an
/// entity has been forgotten, which an indefinite block is not.
///
/// ´claim:lifespan:an-archived-block-is-admitted-on-its-spectrum-and-the-floor-test-is-not-widened-to-carry-it´
/// ´test:crate:hibernation-admits-a-block-on-its-spectrum´
#[test]
fn hibernation_admits_a_block_on_its_spectrum() {
    use crate::model::hibernation::{RestoreRefusal, carries_a_definite_precision, survives_floor};

    let width = 3;
    let mut data = faer::Mat::zeros(width, width);
    for i in 0..width {
        data[(i, i)] = 1.0;
    }
    data[(0, 1)] = 2.0;
    data[(1, 0)] = 2.0;
    let witness = HibernatedBlock {
        mean: vec![0.0; width],
        precision: SymmetricMatrix::from_computation(data),
        covariance: SymmetricMatrix::identity_scaled(width, 1.0),
    };

    // Red: the question the floor test asks is answered yes, because the
    // block has not been forgotten. It is the right answer to that question.
    assert!(
        survives_floor(&witness, 1.0, 0.5),
        "the witness must pass the floor test, or it witnesses nothing"
    );

    // Green: the question the verdict asks is answered no, and it says where.
    let refusal = carries_a_definite_precision(&witness).expect_err("an indefinite block must be refused");
    assert_eq!(
        refusal,
        RestoreRefusal::NotPositiveDefinite { pivot: 1 },
        "the second pivot is the one the factorisation cannot take: 1 - 2² is negative"
    );

    // And a block the model may hold passes both.
    let healthy = HibernatedBlock {
        mean: vec![0.0; width],
        precision: SymmetricMatrix::identity_scaled(width, 4.0),
        covariance: SymmetricMatrix::identity_scaled(width, 0.25),
    };
    assert!(survives_floor(&healthy, 1.0, 0.5), "a healthy block is not forgotten");
    assert_eq!(
        carries_a_definite_precision(&healthy),
        Ok(()),
        "and a healthy block is a matrix the model may hold"
    );
}

/// A registration meeting an archive whose block is indefinite extends at the
/// prior, exactly as it does for a record that is expired or laid out
/// differently. The refusal changes nothing about the disposition: the arm the
/// lifecycle already had logs it and lets the extension stand, and the prior is
/// a definite block by construction, so the alternative to the archived block is
/// always a matrix the model may hold. That is what makes refusing here cheap —
/// the fallback existed before the verdict did.
///
/// ´claim:lifespan:an-indefinite-archived-block-leaves-the-registration-at-the-prior´
/// ´test:crate:indefinite-archived-block-leaves-the-registration-at-the-prior´
#[test]
fn indefinite_archived_block_leaves_the_registration_at_the_prior() {
    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    drop(env.seed_block(&slot, 4.0, 0.25));
    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);

    // Corrupt one archived block the way a damaged artefact would: symmetric
    // to the bit, every diagonal entry strictly positive, one eigenvalue at
    // minus one. Only the operational model's block is touched, so the test
    // also pins that a record with one corrupt block is a corrupt record.
    let mut record = env
        .working
        .hibernation
        .take_sentinel(SentinelId(1))
        .expect("hibernating stores a record");
    let width = record.blocks[0].1.width();
    assert!(width >= 2, "the witness needs two coordinates to disagree over");
    let mut data = faer::Mat::zeros(width, width);
    for i in 0..width {
        data[(i, i)] = 1.0;
    }
    data[(0, 1)] = 2.0;
    data[(1, 0)] = 2.0;
    record.blocks[0].1.precision = SymmetricMatrix::from_computation(data);
    env.working.hibernation.store_sentinel(SentinelId(1), record);

    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);

    let returned = env.slot(SentinelId(1));
    assert!(
        (precision_at(&env.working.operational, returned.start) - env.config.model.lambda_prior).abs() < 1e-12,
        "a refused record leaves the slot at the prior precision"
    );
    assert!(
        (precision_at(&env.working.sister, returned.start) - env.config.model.lambda_prior).abs() < 1e-12,
        "including the sister, whose own block was not the corrupt one"
    );
    assert!(
        env.working.operational.mu()[returned.start].abs() < 1e-12,
        "a refused record leaves the slot's mean at zero"
    );
    assert!(
        env.working.hibernation.is_empty(),
        "and the refused record does not stay in the archive"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The grain the floor is decided at
// ═══════════════════════════════════════════════════════════════════════════════

/// The replenishment floor is decided per model rather than per record, and one
/// record is answered both ways at once. Two models hold the same archived
/// block and age it on their own label-indexed rates; across twenty thousand
/// labels the operational rate takes its copy below the floor while the slower
/// sister rate leaves its copy well above it, and the restore reports one model
/// restored and one at the prior. A record-level refusal could not express
/// this: a single verdict would have to be wrong about one of the two, which is
/// why the floor is not among the admission refusals and its answer is a count.
/// The wall clock is held still so that what splits the record is the label
/// rates alone.
///
/// ´claim:lifespan:the-replenishment-floor-splits-one-record-between-two-models´
/// ´test:crate:the-floor-splits-one-record-between-two-models´
#[test]
fn the_floor_splits_one_record_between_two_models() {
    use crate::model::hibernation::survives_floor;

    let mut env = HibernationEnv::new();
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let slot = env.slot(SentinelId(1));
    let archived_precision = 4.0;
    let archived_covariance = 0.25;
    drop(env.seed_block(&slot, archived_precision, archived_covariance));
    env.submit_at(vec![LifecycleEvent::HibernateSentinel(SentinelId(1))], 0);

    // Take the record out before the identifier returns, so the registration
    // extends at the prior and the restore driven below is the only one to run.
    let record = env
        .working
        .hibernation
        .take_sentinel(SentinelId(1))
        .expect("hibernating stores a record");
    env.submit_at(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 0);
    let returned = env.slot(SentinelId(1));

    // Twenty thousand labels on the label clock and nothing on the wall clock:
    // the models' own rates are then the only thing that differs between them.
    let model_config = &env.config.model;
    let label_seq = record.archived_label_seq + 20_000;
    env.working.last_processed_label_seq = label_seq;
    let now = record.archived_at;

    // The split is arithmetic before it is a report: against the one shared
    // floor the faster rate sinks its copy of the block and the slower does not.
    let decay_opr = record.decay(&now, label_seq, 1.0, model_config.gamma_opr);
    let decay_inh = record.decay(&now, label_seq, 1.0, model_config.gamma_inh);
    let block = record
        .block(&ModelId::Operational)
        .expect("the record holds the operational block");
    assert!(
        !survives_floor(block, decay_opr, model_config.lambda_floor),
        "the operational rate must take this block below the floor, or the record is not split"
    );
    assert!(
        survives_floor(block, decay_inh, model_config.lambda_floor),
        "and the sister rate must leave the very same block above it"
    );

    let report = env
        .working
        .restore_self_structure(&record, returned.start, &now, model_config, 1.0);

    assert_eq!(
        report.restored, 1,
        "exactly the model whose aged copy stood above the floor took its block"
    );
    assert_eq!(
        report.at_prior, 1,
        "and exactly the model whose aged copy sank below it keeps the extension's prior"
    );
    assert!(
        (precision_at(&env.working.operational, returned.start) - model_config.lambda_prior).abs() < 1e-12,
        "the model that sank is left at the prior precision the extension wrote"
    );
    // What surviving the floor buys is the archived block aged, and the two
    // halves of the pair move opposite ways: the precision is scaled by the
    // model's own factor and the covariance by its reciprocal.
    let aged_precision = archived_precision * decay_inh;
    let aged_covariance = archived_covariance / decay_inh;
    assert!(
        (precision_at(&env.working.sister, returned.start) - aged_precision).abs() < 1e-12,
        "while the model that survived carries the archived precision scaled by its own decay"
    );
    assert!(
        (covariance_at(&env.working.sister, returned.start) - aged_covariance).abs() < 1e-12,
        "with the covariance inflated by the reciprocal of that same factor"
    );
}
