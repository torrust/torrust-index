// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The [`World`] is the top-level context for test scenarios.
//!
//! [`WorldBuilder`] declares configuration, schemas, channels, interactions, construction baselines, deterministic time, and the required seed before creating the real engine. The finished world supplies name-based lifecycle and report operations, assessment and label cycles, forward-only scenario time, queue-specific barriers, guarded fixtures, and owned state probes. Raw [`Assayer`] access remains available for product verbs that do not need a harness translation.

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::atomic::Ordering;
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime};

use torrust_sentinel::BatchReport;

use super::clock::Clock as _;
use super::fixtures::{TrainedStateFixture, TrainedStateFixtureError};
use super::label_spec::LabelSpec;
use super::liveness::ACK_DEADLINE;
use super::names::{AxisName, ChannelName, IdentityName, NameRegistry, SentinelName};
use super::probes::{PendingEntryView, PublishedModelBlock, PublishedSlotMoments};
use super::rng::TestRng;
use super::tolerances::{DEFAULT_TOLERANCES, Tolerances};
use super::virtual_clock::VirtualClock;
use crate::Assayer;
use crate::api::{AssayerBuilder, LabelAck, ReportAck};
use crate::assessment::{DerivedReckoning, RequestContext, RiskAssessment};
use crate::config::types::{AssayerConfig, PersistenceConfig};
use crate::error::{BuildError, ChannelError, LabelError, LifecycleError, ReportError};
use crate::feature::dimension_map::{AGGREGATE_FEATURE_COUNT, DimensionMap};
use crate::feature::interaction::{InteractionId, InteractionTemplate};
use crate::feature::standardisation::StandardisationPhase;
use crate::identity::{CompetitiveCellId, MaintenanceCommand};
use crate::owner::commands::{
    IdentityDimensionRegistration, LabelData, LifecycleSubmission, ModelOwnerCommand, OutcomeAxisRegistration,
    SentinelRegistration,
};
use crate::resonance::channel::{ChannelPolicy, validate_channel_policy};
use crate::resonance::derivation::{ResonanceConfig, render_resonances};
use crate::resonance::landscape::derive_landscape;
use crate::risk::challenge::{ChallengeEffectivenessState, ChallengeEffectivenessTracker};
use crate::signal::SignalDeclaration;
use crate::types::{
    Action, AssessmentId, ChallengeResult, ChannelId, DimensionId, EntityKey, IdentityBudget, OutcomeAxisId, OutcomeEligibility,
    PersistentTimestamp, SentinelId, SpatialFeaturePolicy,
};

// ═════════════════════════════════════════════════════════════════════
// WorldBuilder
// ═════════════════════════════════════════════════════════════════════

/// Published feature-vector widths observed by a scenario.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct RuntimeLayout {
    /// Total dimension-map width.
    pub dimension_map_width: usize,
    /// Bias block width.
    pub bias_width: usize,
    /// Aggregate block width.
    pub aggregate_width: usize,
    /// Combined width of every per-dimension identity block.
    pub identity_dimensions_width: usize,
    /// Cross-dimension identity block width.
    pub identity_cross_dimension_width: usize,
    /// Declared-signal block width.
    pub signal_width: usize,
    /// Combined width of every Sentinel slot.
    pub sentinel_slots_width: usize,
    /// Interaction block width.
    pub interaction_width: usize,
    /// Competitive-indicator block width.
    pub competitive_cells_width: usize,
}

impl RuntimeLayout {
    /// Reads the widths carried by one published dimension map.
    fn from_dimension_map(map: &DimensionMap) -> Self {
        Self {
            dimension_map_width: map.p,
            bias_width: 1,
            aggregate_width: map.agg_range.len(),
            identity_dimensions_width: map
                .id_dim_ranges
                .values()
                .map(crate::feature::dimension_map::IndexRange::len)
                .sum(),
            identity_cross_dimension_width: map
                .id_cross_dim_range
                .as_ref()
                .map_or(0, crate::feature::dimension_map::IndexRange::len),
            signal_width: map.sig_range.len(),
            sentinel_slots_width: map
                .sentinel_slots
                .values()
                .map(crate::feature::dimension_map::IndexRange::len)
                .sum(),
            interaction_width: map.interaction_range.len(),
            competitive_cells_width: map.competitive_range.len(),
        }
    }

    const fn is_complete(self) -> bool {
        self.dimension_map_width
            == self.bias_width
                + self.aggregate_width
                + self.identity_dimensions_width
                + self.identity_cross_dimension_width
                + self.signal_width
                + self.sentinel_slots_width
                + self.interaction_width
                + self.competitive_cells_width
    }

    const fn can_grow_into(self, expected: Self) -> bool {
        self.is_complete()
            && expected.is_complete()
            && self.bias_width == expected.bias_width
            && self.aggregate_width == expected.aggregate_width
            && self.signal_width == expected.signal_width
            && self.identity_dimensions_width <= expected.identity_dimensions_width
            && self.identity_cross_dimension_width <= expected.identity_cross_dimension_width
            && self.sentinel_slots_width <= expected.sentinel_slots_width
            && self.interaction_width <= expected.interaction_width
            && self.competitive_cells_width <= expected.competitive_cells_width
            && self.dimension_map_width <= expected.dimension_map_width
    }
}

/// Every reward and posture field retained for one constructed channel.
#[derive(Clone, Copy, Debug)]
pub struct ChannelPolicyBaseline {
    /// Opportunity cost of full denial.
    pub pass: f64,
    /// Cost of challenge to benign cases.
    pub friction: f64,
    /// Cost when an adverse outcome is not prevented.
    pub missed: f64,
    /// Value of identifying an adverse source.
    pub caught: f64,
    /// Cost of denial to benign cases.
    pub blocked: f64,
    /// Throttle severity.
    pub slow_severity: f64,
    /// Fraction of challenge catching retained under Block.
    pub block_catches: f64,
    /// Adverse posture-sensitivity exponent.
    pub beta_bad: f64,
    /// Benign posture-sensitivity exponent.
    pub beta_good: f64,
    /// Challenge-effectiveness time-decay.
    pub gamma_q_t: f64,
    /// Reserved suspicious-shaping width.
    pub neutral_zone: f64,
}

impl ChannelPolicyBaseline {
    const fn from_policy(policy: &ChannelPolicy) -> Self {
        Self {
            pass: policy.reward.pass,
            friction: policy.reward.friction,
            missed: policy.reward.missed,
            caught: policy.reward.caught,
            blocked: policy.reward.blocked,
            slow_severity: policy.reward.slow_severity,
            block_catches: policy.reward.block_catches,
            beta_bad: policy.reward.beta_bad,
            beta_good: policy.reward.beta_good,
            gamma_q_t: policy.reward.gamma_q_t,
            neutral_zone: policy.neutral_zone,
        }
    }
}

impl PartialEq for ChannelPolicyBaseline {
    fn eq(&self, other: &Self) -> bool {
        self.pass.to_bits() == other.pass.to_bits()
            && self.friction.to_bits() == other.friction.to_bits()
            && self.missed.to_bits() == other.missed.to_bits()
            && self.caught.to_bits() == other.caught.to_bits()
            && self.blocked.to_bits() == other.blocked.to_bits()
            && self.slow_severity.to_bits() == other.slow_severity.to_bits()
            && self.block_catches.to_bits() == other.block_catches.to_bits()
            && self.beta_bad.to_bits() == other.beta_bad.to_bits()
            && self.beta_good.to_bits() == other.beta_good.to_bits()
            && self.gamma_q_t.to_bits() == other.gamma_q_t.to_bits()
            && self.neutral_zone.to_bits() == other.neutral_zone.to_bits()
    }
}

impl Eq for ChannelPolicyBaseline {}

/// A complete readback of one harness-owned derivation channel.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ChannelBaseline {
    /// Name declared by the scenario.
    pub name: &'static str,
    /// Identifier assigned in declaration order.
    pub id: ChannelId,
    /// Ordered actions retained for derivation.
    pub actions: Vec<Action>,
    /// Every scalar retained from the channel policy, or absence if the policy was lost.
    pub policy: Option<ChannelPolicyBaseline>,
}

impl ChannelBaseline {
    fn declared(name: ChannelName, id: ChannelId, policy: &ChannelPolicy) -> Self {
        Self {
            name: name.0,
            id,
            actions: policy.actions.clone(),
            policy: Some(ChannelPolicyBaseline::from_policy(policy)),
        }
    }
}

/// Complete construction state measured at a fixture's return boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldConstructionBaseline {
    /// Engine instance identifier.
    pub instance_id: String,
    /// Current deterministic generator state.
    pub rng_state: u64,
    /// Persistent clock reading shared with the engine.
    pub clock: PersistentTimestamp,
    /// Every published feature block width.
    pub runtime_layout: RuntimeLayout,
    /// Published snapshot version.
    pub published_version: u64,
    /// Published layout generation.
    pub layout_generation: u64,
    /// Complete channel declarations in assignment order.
    pub channels: Vec<ChannelBaseline>,
    /// Sentinels present in the published layout.
    pub sentinel_count: usize,
    /// Outcome-axis models present in the published snapshot.
    pub outcome_axis_count: usize,
    /// Identity dimensions present in the published layout.
    pub identity_dimension_count: usize,
    /// Published cold-ramp phase.
    pub standardisation_phase: StandardisationPhase,
    /// Accepted cold-ramp observations.
    pub standardisation_observations: usize,
    /// Published labels processed.
    pub total_labels: u64,
    /// Published eligible labels processed.
    pub eligible_labels: u64,
    /// Assessments accepted by the constructed engine.
    pub total_assessments: u64,
}

/// The independently derived baseline and optional completed-layout target declared by a fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct WorldConstructionDeclaration {
    /// Exact state expected immediately after construction.
    pub initial: WorldConstructionBaseline,
    /// Optional layout the scenario declares it will reach after later setup.
    pub expected_runtime_layout: Option<RuntimeLayout>,
}

impl WorldConstructionDeclaration {
    fn admits(&self, measured: &WorldConstructionBaseline) -> bool {
        measured == &self.initial
            && self
                .expected_runtime_layout
                .is_none_or(|expected| measured.runtime_layout.can_grow_into(expected))
    }
}

/// Construction fixture whose return boundary is applying a baseline guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ConstructionFixture {
    /// A builder carrying an expected completed layout.
    WorldBuilder,
    /// The conventional cold-world constructor.
    Cold,
    /// The conventional scenario constructor.
    Scenario,
    /// The builder-configuring scenario constructor.
    ScenarioWith,
    /// The explicit-config scenario constructor.
    ScenarioWithConfig,
}

impl ConstructionFixture {
    /// Refuses a measured construction state that does not satisfy the fixture declaration.
    ///
    /// # Errors
    ///
    /// Returns [`WorldBuildError::ConstructionBaselineMismatch`] with the complete declaration and measured baseline when any checked fact differs.
    pub fn guard_baseline(
        self,
        expected: &WorldConstructionDeclaration,
        measured: WorldConstructionBaseline,
    ) -> Result<WorldConstructionBaseline, WorldBuildError> {
        if !expected.admits(&measured) {
            return Err(WorldBuildError::ConstructionBaselineMismatch {
                fixture: self,
                expected: Box::new(expected.clone()),
                measured: Box::new(measured),
            });
        }
        Ok(measured)
    }
}

impl std::fmt::Display for ConstructionFixture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::WorldBuilder => formatter.write_str("WorldBuilder::build"),
            Self::Cold => formatter.write_str("World::cold"),
            Self::Scenario => formatter.write_str("scenario"),
            Self::ScenarioWith => formatter.write_str("scenario_with"),
            Self::ScenarioWithConfig => formatter.write_str("scenario_with_config"),
        }
    }
}

/// Declared observation workload and competitive set for one identity dimension.
#[derive(Clone, Debug)]
pub struct CompetitiveCellSpec {
    /// Entity/count pairs to submit through assessment.
    observations: Vec<(EntityKey, usize)>,
    /// Exact competitive cells the resulting published layout must contain.
    expected_cells: Vec<CompetitiveCellId>,
    /// Identity graph resources that govern the declared workload.
    budget: IdentityBudget,
}

impl CompetitiveCellSpec {
    /// Declares the exact competitive cells setup must produce.
    #[must_use]
    pub fn new(mut expected_cells: Vec<CompetitiveCellId>) -> Self {
        expected_cells.sort_unstable_by_key(|cell| (cell.lo, cell.depth));
        Self {
            observations: Vec::new(),
            expected_cells,
            budget: IdentityBudget::for_depth_cutoff(10),
        }
    }

    /// Sets the identity graph resource declaration used for registration.
    #[must_use]
    pub const fn budget(mut self, budget: IdentityBudget) -> Self {
        self.budget = budget;
        self
    }

    /// Adds repeated observations of one entity to the setup workload.
    #[must_use]
    pub fn observe_n(mut self, entity: EntityKey, count: usize) -> Self {
        self.observations.push((entity, count));
        self
    }
}

/// Staged builder for a [`World`].
///
/// Every non-default input is set explicitly; there is no implicit
/// `default()` for the whole thing because determinism requires at
/// minimum a seed.
#[derive(Clone)]
pub struct WorldBuilder {
    /// Engine configuration.
    config: AssayerConfig,
    /// Signal schema declarations.
    signal_schema: Vec<SignalDeclaration>,
    /// Channels to declare at build time (name, policy).
    channels: Vec<(ChannelName, ChannelPolicy)>,
    /// Interaction templates to validate and install at build time.
    interaction_templates: Vec<InteractionTemplate>,
    /// Seed for the test RNG.
    seed: Option<u64>,
    /// Reference layout a finished scenario must reproduce.
    expected_runtime_layout: Option<RuntimeLayout>,
    /// Clock exposed by the harness and shared with the engine.
    clock: Arc<VirtualClock>,
    /// Tolerances exposed on the finished [`World`].
    tolerances: Tolerances,
}

impl WorldBuilder {
    /// Starts a new builder with a given [`AssayerConfig`].
    #[must_use]
    pub fn new(config: AssayerConfig) -> Self {
        Self {
            config,
            signal_schema: Vec::new(),
            channels: Vec::new(),
            interaction_templates: Vec::new(),
            seed: None,
            expected_runtime_layout: None,
            clock: Arc::new(VirtualClock::epoch()),
            tolerances: DEFAULT_TOLERANCES,
        }
    }

    /// Sets the signal schema.
    #[must_use]
    pub fn signal_schema(mut self, declarations: Vec<SignalDeclaration>) -> Self {
        self.signal_schema = declarations;
        self
    }

    /// Sets the interaction templates the engine validates and retains.
    #[must_use]
    pub fn interaction_templates(mut self, templates: Vec<InteractionTemplate>) -> Self {
        self.interaction_templates = templates;
        self
    }

    /// Declares a channel that will be present at build time.
    ///
    /// Channels are declared in the order given; their `ChannelId`s
    /// are assigned by the engine in that same order (starting at 0).
    ///
    /// # Panics
    ///
    /// Panics on a malformed policy: the harness is a host, and the
    /// well-formedness check is the host's at policy load
    /// (´dec:derivation:ordered-actions´).
    #[must_use]
    pub fn channel(mut self, name: impl Into<ChannelName>, policy: ChannelPolicy) -> Self {
        crate::resonance::channel::validate_channel_policy(&policy)
            .unwrap_or_else(|e| panic!("channel policy must be well-formed at load: {e}"));
        self.channels.push((name.into(), policy));
        self
    }

    /// Sets the RNG seed (required).
    #[must_use]
    pub const fn seed(mut self, seed: u64) -> Self {
        self.seed = Some(seed);
        self
    }

    /// Declares the runtime layout the completed scenario must reproduce.
    #[must_use]
    pub const fn expected_runtime_layout(mut self, expected: RuntimeLayout) -> Self {
        self.expected_runtime_layout = Some(expected);
        self
    }

    /// Overrides the virtual clock. Useful when multiple `World`s
    /// should share a clock.
    #[must_use]
    pub fn clock(mut self, clock: Arc<VirtualClock>) -> Self {
        self.clock = clock;
        self
    }

    /// Enables persistence with the given checkpoint and journal directories.
    #[must_use]
    pub fn persistence_dirs(mut self, checkpoint_dir: impl Into<PathBuf>, journal_dir: impl Into<PathBuf>) -> Self {
        self.config.persistence = Some(PersistenceConfig::new(checkpoint_dir.into(), journal_dir.into()));
        self
    }

    fn construction_declaration(&self, seed: u64) -> WorldConstructionDeclaration {
        let signal_width = self
            .signal_schema
            .iter()
            .map(|declaration| declaration.shape.feature_width())
            .sum();
        let interaction_width = self
            .interaction_templates
            .iter()
            .filter(|template| matches!(template, InteractionTemplate::Type4 { .. }))
            .count();
        let dimension_map_width = 1 + AGGREGATE_FEATURE_COUNT + signal_width + interaction_width;
        let channels = self
            .channels
            .iter()
            .enumerate()
            .map(|(index, (name, policy))| {
                ChannelBaseline::declared(
                    *name,
                    ChannelId(u32::try_from(index).expect("test channel index fits in u32")),
                    policy,
                )
            })
            .collect();

        WorldConstructionDeclaration {
            initial: WorldConstructionBaseline {
                instance_id: self.config.instance_id.clone(),
                rng_state: seed,
                clock: self.clock.now(),
                runtime_layout: RuntimeLayout {
                    dimension_map_width,
                    bias_width: 1,
                    aggregate_width: AGGREGATE_FEATURE_COUNT,
                    identity_dimensions_width: 0,
                    identity_cross_dimension_width: 0,
                    signal_width,
                    sentinel_slots_width: 0,
                    interaction_width,
                    competitive_cells_width: 0,
                },
                published_version: 1,
                layout_generation: 0,
                channels,
                sentinel_count: 0,
                outcome_axis_count: 0,
                identity_dimension_count: 0,
                standardisation_phase: StandardisationPhase::WaitingForInit,
                standardisation_observations: 0,
                total_labels: 0,
                eligible_labels: 0,
                total_assessments: 0,
            },
            expected_runtime_layout: self.expected_runtime_layout,
        }
    }

    /// Builds the [`World`].
    ///
    /// # Errors
    ///
    /// Returns [`WorldBuildError`] if the seed was not declared or the
    /// underlying engine rejects the configuration (invalid channel policy,
    /// malformed schema, failed startup), or if a builder carrying an expected
    /// runtime layout does not reproduce its independently derived construction
    /// baseline or cannot grow from that baseline into the declared layout.
    pub fn build(self) -> Result<World, WorldBuildError> {
        let seed = self.seed.ok_or(WorldBuildError::MissingSeed)?;
        let construction_declaration = self.construction_declaration(seed);
        #[cfg(feature = "serde")]
        let fork_builder = self.clone();

        let mut names = NameRegistry::default();

        // The engine reads time through the same clock the harness
        // exposes, so `World::advance` moves the engine's present and
        // an untouched clock keeps it still.
        let engine_clock: Arc<dyn super::clock::Clock> = self.clock.clone();
        let builder: AssayerBuilder = Assayer::builder(self.config)
            .signal_schema(&self.signal_schema)
            .interaction_templates(self.interaction_templates)
            .clock(engine_clock);
        let mut channel_policies = HashMap::new();
        let mut challenge_tracker = ChallengeEffectivenessTracker::new();
        for (index, (name, policy)) in self.channels.into_iter().enumerate() {
            validate_channel_policy(&policy)?;
            let id = ChannelId(u32::try_from(index).expect("test channel index fits in u32"));
            let alpha_0 = ChallengeEffectivenessState::DEFAULT_ALPHA_0;
            let beta_0 = ChallengeEffectivenessState::DEFAULT_BETA_0;
            let state = ChallengeEffectivenessState {
                alpha: alpha_0,
                beta: beta_0,
                t_last: self.clock.now(),
                alpha_0,
                beta_0,
                gamma_qt: policy.reward.gamma_q_t,
                injection_ceiling: ChallengeEffectivenessState::DEFAULT_INJECTION_CEILING,
                override_active: None,
            };
            challenge_tracker.insert_channel(id, state);
            names.channels.push((name, id));
            channel_policies.insert(id, policy);
        }

        let assayer = builder.build()?;

        let world = World {
            assayer,
            clock: self.clock,
            rng: TestRng::new(seed),
            names,
            channel_policies,
            challenge_tracker: Mutex::new(challenge_tracker),
            assessment_channels: Mutex::new(HashMap::new()),
            tol: self.tolerances,
            expected_runtime_layout: self.expected_runtime_layout,
            construction_declaration,
            #[cfg(feature = "serde")]
            fork_builder,
            #[cfg(feature = "serde")]
            persistence_root: None,
        };
        if world.expected_runtime_layout.is_some() {
            world.guard_construction_fixture(ConstructionFixture::WorldBuilder)?;
        }
        Ok(world)
    }
}

/// Errors that can arise when building a [`World`].
#[derive(Debug)]
pub enum WorldBuildError {
    /// The deterministic RNG seed was not declared.
    MissingSeed,
    /// Channel policy rejected by the engine.
    Channel(ChannelError),
    /// Engine startup failed.
    Build(BuildError),
    /// A construction fixture did not reproduce its complete declaration.
    ConstructionBaselineMismatch {
        /// Fixture applying the guard.
        fixture: ConstructionFixture,
        /// Complete baseline and optional completed layout declared by the fixture.
        expected: Box<WorldConstructionDeclaration>,
        /// Complete baseline read back after construction.
        measured: Box<WorldConstructionBaseline>,
    },
}

impl From<ChannelError> for WorldBuildError {
    fn from(e: ChannelError) -> Self {
        Self::Channel(e)
    }
}

impl From<BuildError> for WorldBuildError {
    fn from(e: BuildError) -> Self {
        Self::Build(e)
    }
}

impl std::fmt::Display for WorldBuildError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingSeed => f.write_str("world seed was not declared"),
            Self::Channel(e) => write!(f, "channel declaration failed: {e}"),
            Self::Build(e) => write!(f, "assayer build failed: {e}"),
            Self::ConstructionBaselineMismatch {
                fixture,
                expected,
                measured,
            } => write!(
                f,
                "{fixture} measured construction baseline {measured:?} does not satisfy declaration {expected:?}"
            ),
        }
    }
}

impl std::error::Error for WorldBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::MissingSeed | Self::ConstructionBaselineMismatch { .. } => None,
            Self::Channel(e) => Some(e),
            Self::Build(e) => Some(e),
        }
    }
}

/// Errors reported by state-establishing scenario fixtures and layout guards.
#[derive(Debug)]
pub enum WorldFixtureError {
    /// Identity registration was rejected by the engine.
    Lifecycle(LifecycleError),
    /// The identity-maintenance command queue was full.
    IdentityMaintenanceBusy,
    /// The identity-maintenance thread was unavailable.
    IdentityMaintenanceUnavailable,
    /// The identity-maintenance thread did not acknowledge its barrier.
    IdentityMaintenanceTimedOut,
    /// The Ledger-GC scheduler was unavailable.
    LedgerGcUnavailable,
    /// The Ledger-GC scheduler did not acknowledge its cycle.
    LedgerGcTimedOut,
    /// Persistence is disabled or the checkpoint scheduler was unavailable.
    CheckpointSchedulerUnavailable,
    /// The checkpoint scheduler did not acknowledge its trigger cycle.
    CheckpointSchedulerTimedOut,
    /// A targeted scenario-time move requested an instant before the current reading.
    BackwardTimeTravel {
        /// Reading held by the persistent clock when travel was requested.
        current: PersistentTimestamp,
        /// Earlier persistent instant the scenario requested.
        requested: PersistentTimestamp,
    },
    /// The model owner rejected the publication barrier.
    ModelOwner(LabelError),
    /// The declared competitive cells did not reach the published layout.
    CompetitiveCellsNotObserved {
        /// Registered identity dimension.
        dimension: DimensionId,
        /// Exact declared set.
        expected: Vec<CompetitiveCellId>,
        /// Exact set observed in the published dimension map.
        observed: Vec<CompetitiveCellId>,
    },
    /// The published runtime widths differ from the builder declaration.
    RuntimeLayoutMismatch {
        /// Layout declared by the builder.
        expected: Box<RuntimeLayout>,
        /// Layout observed in one published snapshot.
        observed: Box<RuntimeLayout>,
    },
    /// Recorder activation was requested while a sample was already active.
    #[cfg(any(test, feature = "test-support"))]
    LabelUpdateRecorderAlreadyActive,
    /// Recorder drain was requested without an active sample.
    #[cfg(any(test, feature = "test-support"))]
    LabelUpdateRecorderInactive,
    /// Recorder drain was requested before a publication completed the region.
    #[cfg(any(test, feature = "test-support"))]
    LabelUpdateRecorderIncomplete,
}

impl From<LifecycleError> for WorldFixtureError {
    fn from(error: LifecycleError) -> Self {
        Self::Lifecycle(error)
    }
}

impl From<LabelError> for WorldFixtureError {
    fn from(error: LabelError) -> Self {
        Self::ModelOwner(error)
    }
}

#[cfg(any(test, feature = "test-support"))]
impl From<crate::snapshot::shared::LabelUpdateRecorderError> for WorldFixtureError {
    fn from(error: crate::snapshot::shared::LabelUpdateRecorderError) -> Self {
        match error {
            crate::snapshot::shared::LabelUpdateRecorderError::AlreadyActive => Self::LabelUpdateRecorderAlreadyActive,
            crate::snapshot::shared::LabelUpdateRecorderError::Inactive => Self::LabelUpdateRecorderInactive,
            crate::snapshot::shared::LabelUpdateRecorderError::Incomplete => Self::LabelUpdateRecorderIncomplete,
        }
    }
}

impl std::fmt::Display for WorldFixtureError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Lifecycle(error) => write!(f, "identity registration failed: {error}"),
            Self::IdentityMaintenanceBusy => f.write_str("identity-maintenance command queue is full"),
            Self::IdentityMaintenanceUnavailable => f.write_str("identity-maintenance thread is unavailable"),
            Self::IdentityMaintenanceTimedOut => f.write_str("identity-maintenance barrier timed out"),
            Self::LedgerGcUnavailable => f.write_str("Ledger-GC scheduler is unavailable"),
            Self::LedgerGcTimedOut => f.write_str("Ledger-GC barrier timed out"),
            Self::CheckpointSchedulerUnavailable => f.write_str("checkpoint scheduler is unavailable"),
            Self::CheckpointSchedulerTimedOut => f.write_str("checkpoint-scheduler barrier timed out"),
            Self::BackwardTimeTravel { current, requested } => {
                write!(
                    f,
                    "scenario time cannot move backward: current {current:?}, requested {requested:?}"
                )
            }
            Self::ModelOwner(error) => write!(f, "model-owner publication barrier failed: {error}"),
            Self::CompetitiveCellsNotObserved {
                dimension,
                expected,
                observed,
            } => write!(
                f,
                "identity dimension {dimension:?} published competitive cells {observed:?}, expected {expected:?}"
            ),
            Self::RuntimeLayoutMismatch { expected, observed } => {
                write!(
                    f,
                    "published runtime layout {observed:?} differs from declared layout {expected:?}"
                )
            }
            #[cfg(any(test, feature = "test-support"))]
            Self::LabelUpdateRecorderAlreadyActive => f.write_str("label-update recorder sample is already active"),
            #[cfg(any(test, feature = "test-support"))]
            Self::LabelUpdateRecorderInactive => f.write_str("label-update recorder sample is inactive"),
            #[cfg(any(test, feature = "test-support"))]
            Self::LabelUpdateRecorderIncomplete => f.write_str("label-update recorder sample has no completed publication"),
        }
    }
}

impl std::error::Error for WorldFixtureError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Lifecycle(error) => Some(error),
            Self::ModelOwner(error) => Some(error),
            Self::IdentityMaintenanceBusy
            | Self::IdentityMaintenanceUnavailable
            | Self::IdentityMaintenanceTimedOut
            | Self::LedgerGcUnavailable
            | Self::LedgerGcTimedOut
            | Self::CheckpointSchedulerUnavailable
            | Self::CheckpointSchedulerTimedOut
            | Self::BackwardTimeTravel { .. }
            | Self::CompetitiveCellsNotObserved { .. }
            | Self::RuntimeLayoutMismatch { .. } => None,
            #[cfg(any(test, feature = "test-support"))]
            Self::LabelUpdateRecorderAlreadyActive | Self::LabelUpdateRecorderInactive | Self::LabelUpdateRecorderIncomplete => {
                None
            }
        }
    }
}

// ═════════════════════════════════════════════════════════════════════
// World
// ═════════════════════════════════════════════════════════════════════

/// Top-level test context. See the module-level docs for the shape.
pub struct World {
    /// The real engine under test — no mocks.
    assayer: Assayer,
    /// Deterministic clock shared with the engine.
    clock: Arc<VirtualClock>,
    /// Seeded RNG for all test-side randomness.
    rng: TestRng,
    /// Name → ID registry for Sentinels, axes, channels.
    names: NameRegistry,
    /// Host-owned channel policies for test-side derivation.
    channel_policies: HashMap<ChannelId, ChannelPolicy>,
    /// Host-owned challenge effectiveness tracker for test-side derivation.
    challenge_tracker: Mutex<ChallengeEffectivenessTracker>,
    /// Host-side mapping from assessment ID to derivation channel.
    assessment_channels: Mutex<HashMap<AssessmentId, ChannelId>>,
    /// Domain-specific tolerances used by asserts.
    tol: Tolerances,
    /// Reference layout declared by the scenario builder.
    expected_runtime_layout: Option<RuntimeLayout>,
    /// Complete construction state independently derived from the builder inputs.
    construction_declaration: WorldConstructionDeclaration,
    /// Construction inputs retained so a fork cannot silently change its schema or policies.
    #[cfg(feature = "serde")]
    fork_builder: WorldBuilder,
    /// Declared last: engine threads shut down before their owned storage is removed.
    #[cfg(feature = "serde")]
    persistence_root: Option<super::fork::PersistenceRoot>,
}

/// Guard that releases a test-owned model-owner block when dropped.
pub struct ModelOwnerBlockGuard {
    /// Sender used to release the parked owner thread.
    release_tx: Option<crossbeam_channel::Sender<()>>,
}

impl ModelOwnerBlockGuard {
    /// Releases the owner thread before the guard goes out of scope.
    pub fn release(mut self) {
        self.release_now();
    }

    /// Sends the release token once.
    fn release_now(&mut self) {
        if let Some(release_tx) = self.release_tx.take() {
            let _release_result = release_tx.send(());
        }
    }
}

impl Drop for ModelOwnerBlockGuard {
    fn drop(&mut self) {
        self.release_now();
    }
}

impl World {
    /// Start a new builder with the given engine configuration.
    #[must_use]
    pub fn builder(config: AssayerConfig) -> WorldBuilder {
        WorldBuilder::new(config)
    }

    /// Build a cold [`World`] with a single `"default"` channel
    /// configured with [`ChannelPolicy::default`].
    ///
    /// Convenience for scenarios whose shape is fully captured by "fresh engine, one 4-action channel, seeded RNG". Tests that need multiple channels, a non-default policy, declarations, or an expected construction baseline should spell out the full [`Self::builder`] call.
    ///
    /// # Panics
    ///
    /// Panics if the engine rejects the default configuration; this
    /// would indicate a regression in the crate itself, not a test
    /// error, or if the complete measured construction baseline differs from
    /// the cold-world declaration, so failing loudly is the correct behaviour.
    #[must_use]
    pub fn cold(instance_id: &str, seed: u64) -> Self {
        let world = Self::builder(AssayerConfig {
            instance_id: instance_id.to_owned(),
            // The capacity is host-set with no default; the harness
            // declares the fixture value.
            infrastructure: super::test_infrastructure(),
            ..Default::default()
        })
        .channel("default", ChannelPolicy::default())
        .seed(seed)
        .build()
        .expect("cold world should build with default channel policy");
        world
            .guard_construction_fixture(ConstructionFixture::Cold)
            .expect("cold world should reproduce its complete declaration");
        world
    }

    /// Builds the standard score-verified training population and returns only after its class counts, held-out gap, and pairwise rank preconditions have been measured and guarded.
    ///
    /// # Errors
    ///
    /// Returns [`TrainedStateFixtureError`] when real-engine construction or label publication fails, or when the measured baseline does not satisfy the fixture's declared trained-state precondition.
    ///
    /// # Panics
    ///
    /// Panics only if the default channel policy becomes malformed, the model owner cannot answer the cold-ramp barrier, or the cold ramp stops advancing; none can be arranged by this fixture's inputs.
    pub fn trained_state(instance_id: &str, seed: u64) -> Result<TrainedStateFixture, TrainedStateFixtureError> {
        TrainedStateFixture::establish(instance_id, seed)
    }

    /// Returns the complete state declaration derived from this world's construction inputs.
    #[must_use]
    pub const fn construction_declaration(&self) -> &WorldConstructionDeclaration {
        &self.construction_declaration
    }

    /// Reads back the complete domain baseline established by construction.
    #[must_use]
    pub fn construction_baseline(&self) -> WorldConstructionBaseline {
        let snapshot = self.assayer.shared.published.load();
        let health = self.assayer.shared.health.load();
        let channels = self
            .names
            .channels
            .iter()
            .map(|(name, id)| {
                let policy = self.channel_policies.get(id);
                ChannelBaseline {
                    name: name.0,
                    id: *id,
                    actions: policy.map_or_else(Vec::new, |policy| policy.actions.clone()),
                    policy: policy.map(ChannelPolicyBaseline::from_policy),
                }
            })
            .collect();

        WorldConstructionBaseline {
            instance_id: self.assayer.config.instance_id.clone(),
            rng_state: self.rng.state(),
            clock: self.clock.now(),
            runtime_layout: RuntimeLayout::from_dimension_map(&snapshot.dimension_map),
            published_version: snapshot.version,
            layout_generation: snapshot.layout_generation,
            channels,
            sentinel_count: snapshot.dimension_map.sentinel_slots.len(),
            outcome_axis_count: snapshot.outcome_models.len(),
            identity_dimension_count: snapshot.dimension_map.id_dim_ranges.len(),
            standardisation_phase: snapshot.standardisation_phase,
            standardisation_observations: snapshot.standardisation_observations,
            total_labels: health.total_labels,
            eligible_labels: health.eligible_labels,
            total_assessments: self.assayer.total_assessments.load(Ordering::Relaxed),
        }
    }

    pub(super) fn guard_construction_fixture(&self, fixture: ConstructionFixture) -> Result<(), WorldBuildError> {
        fixture
            .guard_baseline(&self.construction_declaration, self.construction_baseline())
            .map(drop)
    }

    // ─── Accessors ────────────────────────────────────────────────

    /// Borrow the underlying [`Assayer`] (escape hatch).
    #[must_use]
    pub const fn assayer(&self) -> &Assayer {
        &self.assayer
    }

    /// Force the model-owner thread to exit for failure-surface tests.
    pub fn shut_down_model_owner_for_test(&mut self) {
        drop(self.assayer.command_tx.send(ModelOwnerCommand::Shutdown));
        if let Some(handle) = self.assayer.threads.model_owner.take() {
            handle.join().expect("model-owner thread should join cleanly");
        }
    }

    /// Block the model-owner thread until the returned guard is dropped.
    pub fn block_model_owner_for_test(&self) -> ModelOwnerBlockGuard {
        let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
        let (release_tx, release_rx) = crossbeam_channel::bounded(1);

        self.assayer
            .command_tx
            .send(ModelOwnerCommand::TestBlock {
                entered: entered_tx,
                release: release_rx,
            })
            .expect("model-owner test block command should enqueue");

        entered_rx
            .recv_timeout(ACK_DEADLINE)
            .expect("model-owner thread should enter test block");

        ModelOwnerBlockGuard {
            release_tx: Some(release_tx),
        }
    }

    /// Borrow the virtual clock.
    #[must_use]
    pub const fn clock(&self) -> &Arc<VirtualClock> {
        &self.clock
    }

    // ─── Time verbs ──────────────────────────────────────────────

    /// Advances both clock domains by `delta` and settles the work that the movement makes due.
    ///
    /// The persistent and intra-process readings move together, but remain separate values with separate types: callers cannot convert one into the other or skew one domain independently. After moving them, the scenario crosses identity maintenance, including accepted lifecycle publication, and one complete Ledger-GC cycle. Those are the asynchronous operations whose decisions read the virtual monotonic and persistent presents respectively.
    ///
    /// Cold-ramp observations, the checkpoint scheduler, and the health-event receiver are excluded. Advancing the virtual clock neither submits a cold-ramp observation nor advances the schedulers' real waiting intervals, and draining health events would consume host-visible output rather than settle a time-activated producer. Callers cross those queue-specific barriers when they perform those operations.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] when identity maintenance, model-owner publication, or the Ledger-GC scheduler cannot complete its barrier.
    pub fn advance(&self, delta: Duration) -> Result<(), WorldFixtureError> {
        self.clock.advance(delta);
        self.settle_advanced_time()
    }

    /// Moves both clock domains to `target` and settles the work that the movement makes due.
    ///
    /// The target controls the persistent reading directly and the intra-process reading advances by the same offset, preserving their type separation without letting the two domains skew. The method crosses the same identity-maintenance and Ledger-GC barriers, with the same exclusions, as [`Self::advance`].
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::BackwardTimeTravel`] when `target` precedes the current reading, naming both instants, or another [`WorldFixtureError`] when a required barrier cannot complete.
    pub fn travel_to(&self, target: SystemTime) -> Result<(), WorldFixtureError> {
        let current = self.clock.now();
        let requested = PersistentTimestamp::from_system_time(target);
        if requested < current {
            return Err(WorldFixtureError::BackwardTimeTravel { current, requested });
        }
        self.clock.travel_to(target);
        self.settle_advanced_time()
    }

    /// Crosses the queue-specific barriers required after either scenario time verb moves both domains.
    fn settle_advanced_time(&self) -> Result<(), WorldFixtureError> {
        self.flush_identity_maintenance()?;
        let _outcome = self.flush_ledger_gc()?;
        Ok(())
    }

    /// Forks a quiescent persistent World into isolated storage at the same scenario instant.
    ///
    /// # Errors
    /// Returns an error for missing persistence, failed barriers or I/O, or an identity dimension requiring its process-local registration to be rebound. Use [`Self::fork_persistence_after`] to supply those registrations.
    ///
    /// # Panics
    /// Panics only if the model owner cannot enter its bounded parked state, which a live quiescent World cannot arrange.
    #[cfg(feature = "serde")]
    pub fn fork_persistence(self) -> Result<super::PersistenceFork, Box<dyn std::error::Error>> {
        self.fork_persistence_after(Duration::ZERO, |_| Ok(()))
    }

    /// Copies a checkpoint and journal, advances the one shared clock by `elapsed`, and rebuilds from the isolated copy.
    ///
    /// `rebind` supplies process-local identity encode functions through ordinary identity registration. Every original identity must be present afterwards; a missing registration is refused rather than silently dropping durable identity state. The callback must leave scenario time unchanged.
    ///
    /// # Errors
    /// Returns an error for missing persistence, failed barriers or I/O, incompatible construction, or incomplete identity rebinding.
    ///
    /// # Panics
    /// Panics only if the model owner cannot enter its bounded parked state, which a live quiescent World cannot arrange.
    #[cfg(feature = "serde")]
    pub fn fork_persistence_after(
        self,
        elapsed: Duration,
        rebind: impl FnOnce(&Self) -> Result<(), LifecycleError>,
    ) -> Result<super::PersistenceFork, Box<dyn std::error::Error>> {
        super::fork::PersistenceFork::new(self, elapsed, rebind)
    }

    /// Crosses the complete producer barrier set before freezing durable state.
    #[cfg(feature = "serde")]
    pub(super) fn quiesce_persistence(&self) -> Result<(), Box<dyn std::error::Error>> {
        self.flush_identity_maintenance()?;
        self.flush_observations()?;
        self.flush_ledger_gc()?;
        self.flush_checkpoint_scheduler()?;
        self.flush_labels()?;
        Ok(())
    }

    #[cfg(feature = "serde")]
    pub(super) fn rebuild_persistence(
        &self,
        persistence: PersistenceConfig,
        root: super::fork::PersistenceRoot,
    ) -> Result<Self, WorldBuildError> {
        let mut builder = self.fork_builder.clone();
        builder.config.persistence = Some(persistence);
        // A restore is validated against its durable state, not the cold constructor baseline.
        builder.expected_runtime_layout = None;
        let mut restored = builder.build()?;
        restored.persistence_root = Some(root);
        restored.names.clone_from(&self.names);
        restored.rng.clone_from(&self.rng);
        restored.channel_policies.clone_from(&self.channel_policies);
        restored.challenge_tracker = Mutex::new(
            self.challenge_tracker
                .lock()
                .unwrap_or_else(std::sync::PoisonError::into_inner)
                .clone(),
        );
        Ok(restored)
    }

    /// Mutable access to the seeded RNG.
    pub const fn rng(&mut self) -> &mut TestRng {
        &mut self.rng
    }

    /// The tolerances bundle used by asserts.
    #[must_use]
    pub const fn tol(&self) -> &Tolerances {
        &self.tol
    }

    /// Look up a registered Sentinel by name.
    #[must_use]
    pub fn sentinel(&self, name: impl Into<SentinelName>) -> Option<SentinelId> {
        self.names.sentinel(name.into())
    }

    /// Look up a declared channel by name.
    #[must_use]
    pub fn channel(&self, name: impl Into<ChannelName>) -> Option<ChannelId> {
        self.names.channel(name.into())
    }

    /// Look up a registered outcome axis by name.
    #[must_use]
    pub fn axis(&self, name: impl Into<AxisName>) -> Option<OutcomeAxisId> {
        self.names.axis(name.into())
    }

    /// Look up a registered identity dimension by name.
    #[must_use]
    pub fn identity(&self, name: impl Into<IdentityName>) -> Option<DimensionId> {
        self.names.identity(name.into())
    }

    /// Returns the published feature-vector widths after checking the builder's declared reference layout, if one was supplied.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] if the publication barrier fails, or [`WorldFixtureError::RuntimeLayoutMismatch`] when the one published snapshot loaded after that barrier differs from the declared reference layout.
    pub fn runtime_layout(&self) -> Result<RuntimeLayout, WorldFixtureError> {
        let observed = self.observed_runtime_layout()?;
        if let Some(expected) = self.expected_runtime_layout
            && expected != observed
        {
            return Err(WorldFixtureError::RuntimeLayoutMismatch {
                expected: Box::new(expected),
                observed: Box::new(observed),
            });
        }
        Ok(observed)
    }

    /// Returns published feature-vector widths without applying a builder declaration.
    ///
    /// This is the lifecycle fixture's after-state reader: the starting state
    /// is checked by [`Self::runtime_layout`], while a deliberate removal must
    /// be able to observe the smaller layout it just established.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] if the publication barrier fails.
    pub fn observed_runtime_layout(&self) -> Result<RuntimeLayout, WorldFixtureError> {
        self.flush_labels()?;
        let snapshot = self.assayer.shared.published.load();
        Ok(RuntimeLayout::from_dimension_map(&snapshot.dimension_map))
    }

    /// Copies one named model block from a completed publication.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The model-owner publication barrier processes every command and label submitted before this call. Exactly one published snapshot is then loaded, and the selected model's mean, covariance, dimension, floor masses, version, and layout generation are copied into an owned projection. The projection's observation time comes from this scenario's clock.
    ///
    /// Cold-ramp observations are not covered by the publication barrier. The working precision matrix, working-copy metadata, and every engine borrow, guard, shared owner, or mutation route are excluded.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] if the publication barrier fails. Returns `None` inside `Ok` when an outcome-axis model is not present in the published snapshot.
    pub fn published_model_block(&self, model: crate::types::ModelId) -> Result<Option<PublishedModelBlock>, WorldFixtureError> {
        self.flush_labels()?;
        let snapshot = self.assayer.shared.published.load();
        Ok(PublishedModelBlock::from_snapshot(&snapshot, model, self.clock.now()))
    }

    /// Copies the published standardisation moments for one named Sentinel slot.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The model-owner publication barrier processes every command and label submitted before this call. Exactly one published snapshot is then loaded, the name is resolved through the scenario registry, and the complete slot's means and variances are copied with that snapshot's phase, observation count, version, and layout generation.
    ///
    /// Cold-ramp observations are not covered by the publication barrier. Feature classes, other slots, model parameters, the precision matrix, and every engine borrow or mutation route are excluded.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] if the publication barrier fails. Returns `None` inside `Ok` when the name is unknown or its slot is not present in the published layout.
    pub fn published_slot_moments(
        &self,
        name: impl Into<SentinelName>,
    ) -> Result<Option<PublishedSlotMoments>, WorldFixtureError> {
        self.flush_labels()?;
        let Some(sentinel) = self.names.sentinel(name.into()) else {
            return Ok(None);
        };
        let snapshot = self.assayer.shared.published.load();
        Ok(PublishedSlotMoments::from_snapshot(&snapshot, sentinel))
    }

    /// Copies the request inputs retained for one in-flight assessment.
    ///
    /// # What returning covers, and what it does not
    ///
    /// Pending insertion completes synchronously on the assessing thread, so this reading depends on no asynchronous queue and crosses no barrier for form's sake. It performs one non-consuming pending-buffer lookup and copies the retained raw signal block at its actual storage precision together with the active competitive-cell count for each dimension.
    ///
    /// The durable form's excluded identifiers, timestamps, degradation context, and report origin remain excluded, as do the cache, graph, dimension map, models, and every engine borrow or mutation route. A concurrent label or eviction may win before the one lookup and produce `None`; no mixed pending-entry view can result.
    #[must_use]
    pub fn pending_entry_view(&self, assessment: AssessmentId) -> Option<PendingEntryView> {
        let pending = self.assayer.pending_buffer.get_for_guidance(assessment)?;
        Some(PendingEntryView::from_pending(&pending))
    }

    /// Measures the published coordinate block that one Sentinel removal owns.
    ///
    /// The count includes the Sentinel's slot and every expanded interaction
    /// whose identity names it. The published map is loaded once after the
    /// barrier, preserving the probe contract's snapshot discipline.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] if the publication barrier fails.
    pub fn sentinel_departing_block_width(&self, name: impl Into<SentinelName>) -> Result<Option<usize>, WorldFixtureError> {
        self.flush_labels()?;
        let Some(sentinel) = self.names.sentinel(name.into()) else {
            return Ok(None);
        };
        let snapshot = self.assayer.shared.published.load();
        let Some(slot) = snapshot.dimension_map.sentinel_slots.get(&sentinel) else {
            return Ok(None);
        };
        let interactions = snapshot
            .dimension_map
            .interaction_indices
            .keys()
            .filter(|interaction| match interaction {
                InteractionId::PerSentinel { sentinel: owner, .. } => *owner == sentinel,
                InteractionId::PerPair {
                    sentinel_a, sentinel_b, ..
                } => *sentinel_a == sentinel || *sentinel_b == sentinel,
                InteractionId::Fixed { .. } | InteractionId::PerCell { .. } => false,
            })
            .count();
        Ok(Some(slot.len() + interactions))
    }

    // ─── Lifecycle verbs ──────────────────────────────────────────
    //
    // Every lifecycle verb below ends on [`Self::settle`]. The engine's
    // registration path is fire-and-forget by design and the record
    // declines to promise freshness (´cor:concurrency:reader-obligations´),
    // so a scenario that registers and then assesses has no engine-side
    // guarantee that the assessment sees the registration. Scenarios want
    // that guarantee, and the harness is where it belongs.

    /// Wait until the model owner has processed and published everything
    /// submitted so far.
    ///
    /// The command channel is first-in-first-out and the owner drains it
    /// completely before touching an observation, so a checkpoint request
    /// enqueued after a registration is acknowledged only once that
    /// registration has been applied and its snapshot published. That
    /// makes the checkpoint the harness's publication barrier, and it is
    /// the same barrier [`Self::flush_labels`] already relies on.
    ///
    /// # Panics
    ///
    /// Panics if the model owner has shut down or its command channel is
    /// full — either means the engine is gone, which is never the happy
    /// path for a scenario mid-setup.
    fn settle(&self) {
        self.flush_labels()
            .expect("model owner should acknowledge the publication barrier");
    }

    /// Submits an empty lifecycle batch and waits until its publication is visible.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The empty submission itself traverses the lifecycle handler and publishes its resulting snapshot. The following model-owner checkpoint shares the first-in-first-out command queue, so its acknowledgement proves that publication completed before this method returns.
    ///
    /// This barrier does not drain cold-ramp observations, identity-maintenance work, scheduler queues, or health events. Callers needing one of those producers cross its queue-specific barrier separately.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::Lifecycle`] when the lifecycle submission cannot enter the model-owner queue, or [`WorldFixtureError::ModelOwner`] when the following publication barrier fails.
    pub fn publish_lifecycle(&self) -> Result<(), WorldFixtureError> {
        self.assayer
            .command_tx
            .try_send(ModelOwnerCommand::Lifecycle(LifecycleSubmission {
                events: Vec::new(),
                completion: None,
            }))
            .map_err(|error| match error {
                crossbeam_channel::TrySendError::Full(_) => LifecycleError::CommandChannelFull,
                crossbeam_channel::TrySendError::Disconnected(_) => LifecycleError::ModelOwnerShutdown,
            })?;
        self.flush_labels()?;
        Ok(())
    }

    /// Crosses identity maintenance and publishes its accepted lifecycle changes through the model owner.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The maintenance checkpoint follows earlier maintenance commands, drains identity observations and deferred signal-cache writes, detects competitive-set changes, offers their lifecycle submissions, and publishes graph snapshots before acknowledging. The model-owner publication barrier is sent only after that acknowledgement, so every lifecycle submission accepted from the cycle precedes it and has been applied and published when this method returns.
    ///
    /// Identity observations offered after the maintenance drain, periodic identity decay that is not yet due, and lifecycle submissions refused by a saturated model-owner queue are not covered. The model-owner barrier drains labels as part of its own contract, but it does not drain cold-ramp observations; callers needing that queue cross [`Self::flush_observations`] separately.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] when the identity-maintenance command cannot be queued or acknowledged, or when the model owner rejects its publication barrier.
    pub fn flush_identity_maintenance(&self) -> Result<(), WorldFixtureError> {
        let identity_tx = self
            .assayer
            .identity_command_tx
            .as_ref()
            .ok_or(WorldFixtureError::IdentityMaintenanceUnavailable)?;
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        match identity_tx.try_send(MaintenanceCommand::PrepareCheckpoint(ack_tx)) {
            Ok(()) => {}
            Err(crossbeam_channel::TrySendError::Full(_)) => return Err(WorldFixtureError::IdentityMaintenanceBusy),
            Err(crossbeam_channel::TrySendError::Disconnected(_)) => {
                return Err(WorldFixtureError::IdentityMaintenanceUnavailable);
            }
        }
        match ack_rx.recv_timeout(ACK_DEADLINE) {
            Ok(()) => {}
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                return Err(WorldFixtureError::IdentityMaintenanceTimedOut);
            }
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                return Err(WorldFixtureError::IdentityMaintenanceUnavailable);
            }
        }
        self.flush_labels()?;
        Ok(())
    }

    /// Crosses the Ledger-GC scheduler queue and waits for one complete sweep.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The explicit trigger wakes the Ledger-GC scheduler, and its acknowledgement carries the merged outcome only after every Sentinel visible to that cycle has been swept. The acknowledgement is the scheduler's own cycle-completion signal; no observation of elapsed time or Ledger state participates.
    ///
    /// A Sentinel registered after the cycle takes its Sentinel-id snapshot, an entry inserted after its Ledger takes the cycle's key snapshot, and any later periodic cycle are not covered. This barrier neither processes model-owner work nor publishes a snapshot; callers needing those queues use their own barriers.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] when the scheduler cannot accept or acknowledge the trigger.
    pub fn flush_ledger_gc(&self) -> Result<crate::ledger::GcOutcome, WorldFixtureError> {
        let control_tx = self
            .assayer
            .ledger_gc_control_tx
            .as_ref()
            .ok_or(WorldFixtureError::LedgerGcUnavailable)?;
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        match control_tx.send_timeout(crate::ledger::LedgerGcTrigger::new(ack_tx), ACK_DEADLINE) {
            Ok(()) => {}
            Err(crossbeam_channel::SendTimeoutError::Timeout(_)) => return Err(WorldFixtureError::LedgerGcTimedOut),
            Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => {
                return Err(WorldFixtureError::LedgerGcUnavailable);
            }
        }
        match ack_rx.recv_timeout(ACK_DEADLINE) {
            Ok(outcome) => Ok(outcome),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => Err(WorldFixtureError::LedgerGcTimedOut),
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => Err(WorldFixtureError::LedgerGcUnavailable),
        }
    }

    /// Crosses the checkpoint-scheduler trigger queue.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The explicit trigger wakes the checkpoint scheduler, which acknowledges only after its checkpoint request has entered the model-owner command queue. That acknowledgement is the scheduler's own completion signal, so the barrier observes the trigger cycle directly rather than waiting for an interval.
    ///
    /// The model owner may not yet have processed the request when this method returns, so the resulting checkpoint write, its label drain, and its publication are excluded. [`Self::flush_labels`] crosses that downstream queue when a scenario needs the requested checkpoint completed. Persistence-disabled worlds have no checkpoint scheduler and return an error rather than pretending there is a queue to cross.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] when the scheduler is absent or cannot accept or acknowledge the trigger.
    pub fn flush_checkpoint_scheduler(&self) -> Result<(), WorldFixtureError> {
        let control_tx = self
            .assayer
            .checkpoint_control_tx
            .as_ref()
            .ok_or(WorldFixtureError::CheckpointSchedulerUnavailable)?;
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        match control_tx.send_timeout(crate::persistence::scheduler::CheckpointTrigger::new(ack_tx), ACK_DEADLINE) {
            Ok(()) => {}
            Err(crossbeam_channel::SendTimeoutError::Timeout(_)) => {
                return Err(WorldFixtureError::CheckpointSchedulerTimedOut);
            }
            Err(crossbeam_channel::SendTimeoutError::Disconnected(_)) => {
                return Err(WorldFixtureError::CheckpointSchedulerUnavailable);
            }
        }
        match ack_rx.recv_timeout(ACK_DEADLINE) {
            Ok(()) => Ok(()),
            Err(crossbeam_channel::RecvTimeoutError::Timeout) => Err(WorldFixtureError::CheckpointSchedulerTimedOut),
            Err(crossbeam_channel::RecvTimeoutError::Disconnected) => Err(WorldFixtureError::CheckpointSchedulerUnavailable),
        }
    }

    /// Orders every engine health-event producer, then drains the receiver.
    ///
    /// # What returning covers, and what it does not
    ///
    /// Every production [`crate::health::HealthEvent`] sender is owned by the model-owner thread. This barrier first crosses identity maintenance and the resulting model-owner commands and labels, then crosses the cold-ramp observation queue, and finally crosses commands and labels once more before draining. Every event accepted by the bounded receiver from work offered before this call is therefore present in the returned vector, and the receiver is empty at the drain point.
    ///
    /// Work offered after its queue has been crossed is not covered. An event refused because the bounded health-event receiver was full remains represented only by the dropped-event counter, and draining does not recreate it. The barrier does not run a future periodic scheduler cycle; callers needing Ledger collection or a checkpoint trigger cross those queues separately.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] if identity maintenance or the model owner cannot complete one of the producer barriers.
    pub fn drain_health_events(&self) -> Result<Vec<crate::health::HealthEvent>, WorldFixtureError> {
        self.flush_identity_maintenance()?;
        self.flush_observations()?;
        self.flush_labels()?;
        Ok(self.assayer.drain_health_events())
    }

    /// Register a Sentinel under the given name.
    ///
    /// The name is both the engine-visible `name` field and the
    /// harness-side handle used by [`Self::sentinel`] /
    /// [`Self::deregister_sentinel`].
    pub fn register_sentinel(&mut self, name: impl Into<SentinelName>) -> Result<SentinelId, LifecycleError> {
        let name = name.into();
        let id = self.names.alloc_sentinel_id();
        self.assayer.register_sentinel(SentinelRegistration {
            id,
            name: name.0.to_owned(),
        })?;
        self.settle();
        self.names.sentinels.push((name, id));
        Ok(id)
    }

    /// Deregister a previously registered Sentinel.
    ///
    /// Returns [`LifecycleError::NotFound`] if the name was never
    /// registered with this [`World`]. The name-to-ID mapping is
    /// removed *after* the engine accepts the command.
    pub fn deregister_sentinel(&mut self, name: impl Into<SentinelName>) -> Result<(), LifecycleError> {
        let name = name.into();
        let id = self.names.sentinel(name).ok_or_else(|| LifecycleError::NotFound {
            entity_type: "Sentinel",
            id: format!("{name:?}"),
        })?;
        self.assayer.deregister_sentinel(id)?;
        self.settle();
        self.names.sentinels.retain(|(n, _)| *n != name);
        Ok(())
    }

    /// Ingest a Sentinel report for the Sentinel registered under `name`.
    ///
    /// This keeps public integration tests in the same name-first DSL as
    /// lifecycle and reckoning helpers. Unknown names panic because they are
    /// test authoring mistakes, matching [`Self::request`]'s channel lookup
    /// behaviour; engine-side report validation failures are returned.
    pub fn receive_report(&self, name: impl Into<SentinelName>, report: BatchReport<u128>) -> Result<ReportAck, ReportError> {
        let name = name.into();
        let id = self
            .names
            .sentinel(name)
            .unwrap_or_else(|| panic!("Sentinel {:?} not registered on this World", name.0));
        self.assayer.receive_sentinel_report(id, report)
    }

    /// Register an outcome axis under the given name.
    ///
    /// `spatial` selects [`SpatialFeaturePolicy::Enabled`] when `true`
    /// (each existing Sentinel grows by the per-axis features) and
    /// [`SpatialFeaturePolicy::default()`] otherwise. `initial_kappa`
    /// and `gamma` use the values the harness's `registrations`
    /// helpers already settle on for crate tests (1.0 and 0.99
    /// respectively), keeping the two surfaces aligned.
    pub fn register_axis(&mut self, name: impl Into<AxisName>, spatial: bool) -> Result<OutcomeAxisId, LifecycleError> {
        let name = name.into();
        let id = self.names.alloc_axis_id();
        let policy = if spatial {
            SpatialFeaturePolicy::Enabled
        } else {
            SpatialFeaturePolicy::default()
        };
        self.assayer.register_outcome_axis(OutcomeAxisRegistration {
            id,
            name: name.0.to_owned(),
            description: String::new(),
            eligibility: OutcomeEligibility::default(),
            initial_kappa: 1.0,
            gamma: 0.99,
            spatial_features: policy,
        })?;
        self.settle();
        self.names.axes.push((name, id));
        Ok(id)
    }

    /// Deregister a previously registered outcome axis by name.
    pub fn deregister_axis(&mut self, name: impl Into<AxisName>) -> Result<(), LifecycleError> {
        let name = name.into();
        let id = self.names.axis(name).ok_or_else(|| LifecycleError::NotFound {
            entity_type: "OutcomeAxis",
            id: format!("{name:?}"),
        })?;
        self.assayer.deregister_outcome_axis(id)?;
        self.settle();
        self.names.axes.retain(|(n, _)| *n != name);
        Ok(())
    }

    /// Register an identity dimension under the given name.
    ///
    /// Uses the conventional harness encoding: the first eight bytes of the entity key are interpreted as a little-endian `u64` and zero-extended to a 128-bit coordinate. `domain_bits` and `depth_cutoff` take the specification-default values (128 and 10), and [`IdentityBudget::default()`] supplies the resource budget.
    pub fn register_identity(&mut self, name: impl Into<IdentityName>) -> Result<DimensionId, LifecycleError> {
        self.register_identity_with_budget(name, IdentityBudget::for_depth_cutoff(10))
    }

    /// Register an identity dimension with an explicit graph resource declaration.
    fn register_identity_with_budget(
        &mut self,
        name: impl Into<IdentityName>,
        budget: IdentityBudget,
    ) -> Result<DimensionId, LifecycleError> {
        let name = name.into();
        let id = self.names.alloc_identity_id();
        self.assayer.register_identity_dimension(IdentityDimensionRegistration {
            id,
            name: name.0.to_owned(),
            description: "scenario identity hierarchy".to_owned(),
            coordinate_semantics: "the leading bytes select a prefix group".to_owned(),
            domain_bits: 128,
            depth_cutoff: 10,
            budget,
            encode: |entity| {
                let bytes = entity.as_bytes();
                if bytes.len() >= 8 {
                    u128::from(u64::from_le_bytes(bytes[..8].try_into().unwrap_or_default()))
                } else {
                    let mut buf = [0u8; 8];
                    buf[..bytes.len()].copy_from_slice(bytes);
                    u128::from(u64::from_le_bytes(buf))
                }
            },
        })?;
        self.settle();
        self.names.identities.push((name, id));
        Ok(id)
    }

    /// Register an identity dimension and establish its declared competitive cells through real assessment observations.
    ///
    /// The fixture returns only after the identity-maintenance barrier has drained the observations and the model-owner barrier has published the resulting lifecycle changes. The exact set is then read from that published dimension map; a partial, additional, or absent set is a fixture failure rather than a usable scenario.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError`] if registration or either barrier fails, or if the published layout does not contain exactly the declared cells.
    pub fn register_identity_with_cells(
        &mut self,
        name: impl Into<IdentityName>,
        cells: CompetitiveCellSpec,
    ) -> Result<DimensionId, WorldFixtureError> {
        let CompetitiveCellSpec {
            observations,
            expected_cells,
            budget,
        } = cells;
        let id = self.register_identity_with_budget(name, budget)?;
        for (entity, count) in observations {
            let request = RequestContext::new(entity);
            let requests = vec![request; count];
            let _assessments = self.assayer.assess(&requests);
        }
        self.flush_identity_maintenance()?;

        let snapshot = self.assayer.shared.published.load();
        let mut observed: Vec<_> = snapshot
            .dimension_map
            .competitive_indices
            .get(&id)
            .map(|indices| indices.keys().copied().collect())
            .unwrap_or_default();
        observed.sort_unstable_by_key(|cell| (cell.lo, cell.depth));
        if observed != expected_cells {
            return Err(WorldFixtureError::CompetitiveCellsNotObserved {
                dimension: id,
                expected: expected_cells,
                observed,
            });
        }
        Ok(id)
    }

    /// Deregister a previously registered identity dimension by name.
    pub fn deregister_identity(&mut self, name: impl Into<IdentityName>) -> Result<(), LifecycleError> {
        let name = name.into();
        let id = self.names.identity(name).ok_or_else(|| LifecycleError::NotFound {
            entity_type: "IdentityDimension",
            id: format!("{name:?}"),
        })?;
        self.assayer.deregister_identity_dimension(id)?;
        self.settle();
        self.names.identities.retain(|(n, _)| *n != name);
        Ok(())
    }

    // ─── Assess / derive / label verbs ────────────────────────────

    /// Build an [`EntityKey`] from a stable string name.
    ///
    /// Same name → same bytes → same key, across runs and threads.
    /// Tests never hand-roll an `EntityKey`.
    #[must_use]
    pub fn entity(name: &str) -> EntityKey {
        EntityKey::new(name.as_bytes().to_vec())
    }

    /// Build a [`RequestContext`] on a named channel for a named entity.
    ///
    /// # Panics
    ///
    /// Panics if `channel` was not declared at build time — a
    /// scenario that asks for an unknown channel is a test bug.
    #[must_use]
    pub fn request(&self, channel: impl Into<ChannelName>, entity: &str) -> RequestContext {
        let name = channel.into();
        let ch = self
            .names
            .channel(name)
            .unwrap_or_else(|| panic!("channel {:?} not declared on this World", name.0));
        RequestContext::new(Self::entity(entity)).with_channel_hint(ch)
    }

    /// Build a [`RequestContext`] with one Sentinel coordinate attached.
    ///
    /// The Sentinel is addressed by name and must have been registered in
    /// this [`World`]. This is the common public-API shape for scenarios that
    /// first ingest a report and then need the request to route through that
    /// reported Sentinel's dyadic cells.
    #[must_use]
    pub fn request_with_sentinel(
        &self,
        channel: impl Into<ChannelName>,
        entity: &str,
        sentinel: impl Into<SentinelName>,
        coordinate: u128,
    ) -> RequestContext {
        let sentinel = sentinel.into();
        let sentinel_id = self
            .names
            .sentinel(sentinel)
            .unwrap_or_else(|| panic!("Sentinel {:?} not registered on this World", sentinel.0));
        self.request(channel, entity).with_sentinel(sentinel_id, coordinate)
    }

    /// Submit a single request to the channel-free core assessment API.
    #[must_use]
    pub fn assess(&self, req: RequestContext) -> RiskAssessment {
        let mut out = self.assayer.assess(&[req]);
        out.pop().expect("batch of 1 yields 1 assessment")
    }

    /// Harness convenience: assess once, then derive for the request's
    /// channel hint (or the conventional `"default"` channel).
    pub fn derive_for_request(&self, req: RequestContext) -> Result<DerivedReckoning, Infallible> {
        let mut out = self.derive_for_requests(&[req]);
        out.pop().expect("batch of 1 yields 1 result")
    }

    /// Submit a single request to the channel-free core assessment API.
    #[must_use]
    pub fn core_assess(&self, req: RequestContext) -> RiskAssessment {
        self.assess(req)
    }

    /// Derive a host-composed reckoning from an existing assessment.
    #[must_use]
    pub fn derive(&self, assessment: &RiskAssessment, channel: impl Into<ChannelName>) -> DerivedReckoning {
        let channel = channel.into();
        let channel_id = self
            .names
            .channel(channel)
            .unwrap_or_else(|| panic!("channel {:?} not declared on this World", channel.0));
        let policy = self
            .channel_policies
            .get(&channel_id)
            .unwrap_or_else(|| panic!("channel {:?} policy missing on this World", channel.0));
        let challenge = self.challenge_estimate(channel_id);
        let landscape = derive_landscape(assessment, policy, challenge);
        let profile = render_resonances(&landscape, &assessment.risk, &ResonanceConfig::default());
        DerivedReckoning {
            channel: channel_id,
            assessment: assessment.clone(),
            landscape,
            profile,
        }
    }

    fn challenge_estimate(&self, channel_id: ChannelId) -> crate::types::ChallengePosteriorInput {
        // The tracker's conjugate posterior crosses the boundary whole
        // (´sig:companion:posterior´), through the replacement surface
        // (´req:companion:replacement-trait´).
        use crate::risk::challenge::ChallengeEffectivenessProvider;
        self.challenge_tracker
            .lock()
            .unwrap()
            .challenge_posterior(channel_id, &self.clock.now())
            .into()
    }

    /// Runs the given requests until the cold standardisation ramp reaches
    /// its horizon.
    ///
    /// A scenario that compares two assessments of the same subject has to
    /// say which coordinate state it means them to share. While the ramp is
    /// transitioning they do not share one: each request loads the published
    /// snapshot for itself (´dec:concurrency:per-request-load´) and an
    /// accepted observation advances a later one
    /// (´dec:ordering:score-before-evolve´), so even two requests in a single
    /// batch can be answered in coordinate systems one advance apart. That is
    /// the ramp working, not a defect, and the corpus no longer promises
    /// otherwise (´inv:guarantee:evidence-authority´).
    ///
    /// At the horizon the prior mass is gone and the continuing average is
    /// label-authorised, so on a world nothing has labelled the coordinate
    /// system stands still and repeated assessment is stable again. Settling
    /// first is therefore how a scenario that genuinely needs more than one
    /// assessment — two worlds trained in parallel, two entities on one world,
    /// one subject either side of a structural event — stops measuring the
    /// ramp by accident.
    ///
    /// A scenario comparing derivations of a single subject is not one of
    /// those and should not settle for that reason: it holds one assessment
    /// and derives it against each policy in turn
    /// (´claim:api:core-assessment-needs-no-channel-because-policy-belongs-to-derivation´),
    /// so the policies are compared against a fixed estimate rather than
    /// against two readings the ramp took moments apart.
    ///
    /// The scenario's own requests are used, so no foreign subject enters the
    /// world to settle it.
    ///
    /// # The wait
    ///
    /// The requests are the driver: only an accepted observation retires prior
    /// mass, so reaching the horizon means offering vectors, and they are
    /// offered a burst at a time because a burst gets there in a handful of
    /// batches rather than a hundred. The wait between bursts is
    /// [`Self::flush_observations`] and one reading of the published
    /// standardisation state. Each iteration therefore ends at a known
    /// position on the ramp rather than at whatever the steward had reached,
    /// and the loop's exit is a fact about the engine rather than a guess
    /// about its pace.
    ///
    /// What that replaced was a spin: the phase was read off the burst's own
    /// assessments — which are scored against the snapshot they acquired and
    /// so are retrospective by construction — and the loop yielded and offered
    /// another burst until a wall-clock deadline. Under load the deadline was
    /// the thing being measured, and the burst count needed to settle said as
    /// much about the scheduler as about the ramp. Now no wall clock takes
    /// part. The loop terminates because each iteration ends with the queue
    /// empty, so the accepted count it reads has strictly risen or the ramp is
    /// making no progress at all, and a count bounded above by the horizon
    /// cannot rise for ever. A stall is a failure with both counts named
    /// rather than a timeout, which says what happened instead of only when.
    ///
    /// # Panics
    ///
    /// Panics if the model owner cannot answer the barrier, or if a burst of
    /// requests advances the accepted count by nothing — a ramp that neither
    /// reaches its horizon nor moves has stopped, and a scenario waiting on it
    /// would wait for ever.
    pub fn settle_cold_ramp_with(&self, reqs: &[RequestContext]) {
        if reqs.is_empty() {
            return;
        }
        // Offer in bursts rather than one batch at a time. The harness clock
        // stays fixed, but settling still costs suite runtime. A burst reaches
        // the horizon in a handful of batches instead of a hundred.
        let mut burst: Vec<RequestContext> = Vec::with_capacity(reqs.len() * 32);
        while burst.len() < 32 {
            burst.extend_from_slice(reqs);
        }

        let (mut phase, mut accepted) = self.published_standardisation();
        while !phase.is_in_service() {
            let previous = accepted;
            let _assessments = self.assayer.assess(&burst);
            self.flush_observations()
                .expect("model owner should acknowledge the observation barrier");
            (phase, accepted) = self.published_standardisation();
            assert!(
                phase.is_in_service() || accepted > previous,
                "the cold standardisation ramp did not advance: {previous} accepted before the burst, {accepted} after",
            );
        }
    }

    /// Submit a batch of requests. Results are returned in order.
    #[must_use]
    pub fn derive_for_requests(&self, reqs: &[RequestContext]) -> Vec<Result<DerivedReckoning, Infallible>> {
        let assessments = self.assayer.assess(reqs);
        reqs.iter()
            .zip(assessments)
            .map(|(req, assessment)| {
                let channel_id = req
                    .channel_hint()
                    .or_else(|| self.names.channel(ChannelName("default")))
                    .unwrap_or(ChannelId(0));
                let policy = self.channel_policies.get(&channel_id).cloned().unwrap_or_default();
                let challenge = self.challenge_estimate(channel_id);
                let landscape = derive_landscape(&assessment, &policy, challenge);
                let profile = render_resonances(&landscape, &assessment.risk, &ResonanceConfig::default());
                self.assessment_channels.lock().unwrap().insert(assessment.id, channel_id);
                Ok(DerivedReckoning {
                    channel: channel_id,
                    assessment,
                    landscape,
                    profile,
                })
            })
            .collect()
    }

    /// Submit ground-truth label data to the engine.
    pub fn label(&self, data: LabelData) -> Result<LabelAck, LabelError> {
        self.assayer.label(data)
    }

    /// Record a host-observed challenge outcome against a derived reckoning.
    ///
    /// Returns `true` when the assessment ID maps to a known derivation channel.
    #[must_use]
    pub fn record_challenge_result(&self, assessment_id: AssessmentId, result: ChallengeResult) -> bool {
        let Some(channel_id) = self.assessment_channels.lock().unwrap().get(&assessment_id).copied() else {
            return false;
        };

        self.challenge_tracker
            .lock()
            .unwrap()
            .update(channel_id, result, &self.clock.now());
        true
    }

    /// Drain the label channel and wait until queued labels are processed.
    ///
    /// This is the integration-test hook for observing post-label
    /// state deterministically: the production flush API is crate-
    /// internal, but the shared test harness can expose it safely.
    ///
    /// # What returning covers, and what it does not
    ///
    /// Commands and labels. The marker rides the command channel, which is
    /// first-in-first-out, so every lifecycle submission sent before this call
    /// has been applied; the marker's own handler drains the label channel
    /// before acknowledging, so every label sent before this call has been
    /// applied too.
    ///
    /// Cold-ramp observations are neither. They ride a third channel, reached
    /// only once the command queue has been emptied
    /// (´dec:concurrency:channel-preemption´) and untouched by the marker's
    /// handler, so this call is
    /// answered with observations still queued. A scenario that assesses and
    /// then reads the accepted count, the standardisation phase, or anything
    /// scored against the coordinate system is reading whatever the steward
    /// happened to have got through — [`Self::flush_observations`] is the
    /// barrier for that. Two queues, two barriers; no ordering of one against
    /// the other makes either cover both.
    ///
    /// # Errors
    ///
    /// Returns the [`LabelError`] the engine reports when the command channel
    /// is full or the model owner has shut down.
    pub fn flush_labels(&self) -> Result<(), LabelError> {
        self.assayer.flush_label_channel()
    }

    /// Activates the test-support timer for the joined dense model-update region.
    ///
    /// The recorder is read-only with respect to product state and exists only
    /// in test configurations or with the `test-support` feature.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::LabelUpdateRecorderAlreadyActive`] when a
    /// sample is already active.
    #[cfg(any(test, feature = "test-support"))]
    pub fn activate_label_update_recorder(&self) -> Result<(), WorldFixtureError> {
        self.assayer.shared.label_update_recorder.activate()?;
        Ok(())
    }

    /// Drains the duration accumulated by completed dense update publications.
    ///
    /// The publication barrier first completes every label already submitted,
    /// so the returned duration never describes a partially processed label.
    ///
    /// # Errors
    ///
    /// Returns [`WorldFixtureError::ModelOwner`] when the publication barrier
    /// fails, [`WorldFixtureError::LabelUpdateRecorderInactive`] when no sample
    /// is active, or [`WorldFixtureError::LabelUpdateRecorderIncomplete`] when
    /// no publication completed after activation.
    #[cfg(any(test, feature = "test-support"))]
    pub fn drain_label_update_duration(&self) -> Result<std::time::Duration, WorldFixtureError> {
        self.flush_labels()?;
        Ok(self.assayer.shared.label_update_recorder.drain()?)
    }

    /// Wait until every cold-ramp observation offered so far has been applied.
    ///
    /// The sibling of [`Self::flush_labels`] for the queue that call leaves
    /// alone. On return the steward has drained the observation channel and
    /// published each accepted advance, so the accepted count and the
    /// standardisation phase read afterwards are the ones the offers made so
    /// far add up to — a scenario can therefore step the ramp and read where
    /// it stands, rather than sampling it while it moves.
    ///
    /// The barrier settles the queue and nothing else. An offer a full queue
    /// refused was never queued, and an arrival assembled under a superseded
    /// layout is still refused by the steward; neither becomes an accepted
    /// observation because something waited for it.
    ///
    /// # Errors
    ///
    /// Returns the [`LabelError`] the engine reports when the command channel
    /// is full or the model owner has shut down.
    pub fn flush_observations(&self) -> Result<(), LabelError> {
        self.assayer.flush_observation_channel()
    }

    /// The standardisation phase and accepted count of the published snapshot.
    ///
    /// One atomic load of the coordinate state the engine currently serves
    /// from. Paired with [`Self::flush_observations`] this is a reading of
    /// where the ramp stands; on its own it is a reading of where the ramp
    /// happened to be.
    fn published_standardisation(&self) -> (crate::feature::standardisation::StandardisationPhase, usize) {
        let snapshot = self.assayer.shared.published.load();
        (snapshot.standardisation_phase, snapshot.standardisation_observations)
    }

    // ─── Ergonomic shortcuts ──────────────────────────────────────
    //
    // The common scenario shape is "assess and derive on a named
    // channel for a named entity; then maybe label it." These
    // shortcuts collapse that two-line idiom into one verb.

    /// Assess then derive on `channel` for `entity` in one call.
    ///
    /// Equivalent to `self.derive_for_request(self.request(channel, entity))`,
    /// kept as its own verb because every integration test in this
    /// crate spells the longer form at least once per scenario.
    ///
    /// # Panics
    ///
    /// Panics if `channel` was not declared at build time.
    pub fn derive_on(&self, channel: impl Into<ChannelName>, entity: &str) -> Result<DerivedReckoning, Infallible> {
        self.derive_for_request(self.request(channel, entity))
    }

    /// Assess then derive on the conventional `"default"` channel.
    ///
    /// The `World::cold` constructor always declares a `"default"`
    /// channel; tests that don't care about the channel name use
    /// this shortcut.
    ///
    /// # Panics
    ///
    /// Panics if no `"default"` channel was declared.
    pub fn derive_default(&self, entity: &str) -> Result<DerivedReckoning, Infallible> {
        self.derive_on("default", entity)
    }

    /// One round of `assess(channel, entity) → label(...) → flush`.
    ///
    /// The caller supplies a label builder keyed on the assessment ID
    /// the engine just issued. This keeps the common integration-test
    /// setup in one place while still allowing scenarios to vary the
    /// action, valence, ground-truth flag, or challenge result.
    ///
    /// # Panics
    ///
    /// Panics if assessment/derivation, label submission, or subsequent flush fails.
    pub fn cycle_on<F>(&self, channel: impl Into<ChannelName>, entity: &str, build_label: F) -> DerivedReckoning
    where
        F: FnOnce(AssessmentId) -> super::label_spec::LabelSpec,
    {
        let channel = channel.into();
        let r = self
            .derive_on(channel, entity)
            .unwrap_or_else(|e| panic!("cycle_on({}, {entity}): derive failed: {e:?}", channel.0));
        let spec = build_label(r.assessment.id);
        if spec.action_taken() == Action::Challenge
            && let Some(result) = spec.challenge_outcome()
        {
            let _recorded = self.record_challenge_result(r.assessment.id, result);
        }
        self.label(spec.build())
            .unwrap_or_else(|e| panic!("cycle_on({}, {entity}): label failed: {e:?}", channel.0));
        self.flush_labels()
            .unwrap_or_else(|e| panic!("cycle_on({}, {entity}): flush failed: {e:?}", channel.0));
        r
    }

    /// One round of `assess(default, entity) → label(...)`.
    ///
    /// Convenience wrapper over [`Self::cycle_on`] for the common
    /// single-channel harness shape.
    ///
    /// # Panics
    ///
    /// Panics if assessment/derivation, label submission, or subsequent flush fails.
    pub fn cycle_default<F>(&self, entity: &str, build_label: F) -> DerivedReckoning
    where
        F: FnOnce(AssessmentId) -> super::label_spec::LabelSpec,
    {
        self.cycle_on("default", entity, build_label)
    }

    /// One round of `assess → label(benign)` on the `"default"` channel.
    ///
    /// Returns the derived reckoning so the caller can still inspect its
    /// fields. Both calls use `expect`/`unwrap_or_else` internally
    /// — this verb is for tests where the round-trip is part of the
    /// *setup* and any failure is itself the bug.
    ///
    /// # Panics
    ///
    /// Panics if either the assessment/derivation step or the label submission fails.
    pub fn cycle_benign(&self, entity: &str) -> DerivedReckoning {
        self.cycle_default(entity, super::label_spec::LabelSpec::benign)
    }

    /// One round of `assess → label(adverse)` on the `"default"` channel.
    ///
    /// Mirror of [`Self::cycle_benign`] for positive-valence labels.
    ///
    /// # Panics
    ///
    /// Panics if either the assessment/derivation step or the label submission fails.
    pub fn cycle_adverse(&self, entity: &str) -> DerivedReckoning {
        self.cycle_default(entity, super::label_spec::LabelSpec::adverse)
    }

    /// Register a batch of Sentinels by name in declaration order.
    ///
    /// Returns the assigned [`SentinelId`]s in the same order. Useful
    /// for scenarios that need a small pool of Sentinels with no
    /// special per-Sentinel handling.
    ///
    /// # Errors
    ///
    /// Stops at the first registration failure and returns it; any
    /// Sentinels registered before the failure remain registered
    /// (the error path is never the happy path for this shortcut —
    /// scenarios should treat a failure as an unconditional test
    /// abort).
    pub fn register_sentinels<I>(&mut self, names: I) -> Result<Vec<SentinelId>, LifecycleError>
    where
        I: IntoIterator,
        I::Item: Into<SentinelName>,
    {
        let iter = names.into_iter();
        let (lo, _) = iter.size_hint();
        let mut ids = Vec::with_capacity(lo);
        for name in iter {
            ids.push(self.register_sentinel(name)?);
        }
        Ok(ids)
    }
}

/// One round of `assess(request) -> label(...) -> flush` for custom requests.
///
/// [`World::cycle_on`] covers named-channel/entity requests. The enrichment-grid tests (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´) often need richer public requests with request-scoped signals or reported Sentinel coordinates, so this helper keeps that cycle reusable without adding more production harness surface.
#[track_caller]
pub fn cycle_request<F>(world: &World, request: RequestContext, build_label: F) -> DerivedReckoning
where
    F: FnOnce(AssessmentId) -> LabelSpec,
{
    let reckoning = world.derive_for_request(request).expect("cycle_request: assess failed");
    let spec = build_label(reckoning.assessment.id);
    if spec.action_taken() == Action::Challenge
        && let Some(result) = spec.challenge_outcome()
    {
        let _recorded = world.record_challenge_result(reckoning.assessment.id, result);
    }
    world.label(spec.build()).expect("cycle_request: label failed");
    world.flush_labels().expect("cycle_request: flush failed");
    reckoning
}
