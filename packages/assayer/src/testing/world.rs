// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! The [`World`] — top-level context for integration-test scenarios.
//!
//! # Scope (Stage 1)
//!
//! This skeleton supports only the verbs that work against today's
//! public API without engine-side changes:
//!
//! - Construction via [`WorldBuilder`] (config, signal schema, channels).
//! - Sentinel register / deregister by name.
//! - Sentinel report ingestion by name.
//! - Raw access to the underlying [`Assayer`] for verbs the harness
//!   does not yet wrap (escape hatch, to be narrowed as the DSL grows).
//! - Seeded RNG, virtual clock handle, named tolerances.
//!
//! # Not yet wired
//!
//! - Time travel that the engine *observes* in its decay. The engine
//!   already takes both timestamp domains from this world's clock, so
//!   report staleness is deterministic; what remains is the rest of
//!   the persistent-domain call sites and the `advance` / `travel_to`
//!   verbs themselves.
//! - Scripted label / report tapes.
//! - State snapshots for bit-identical diffing.
//! - Domain-aware assertion builders (`assert_ledger_ewma(…).near(…)`).
//!
//! Each of those arrives as its own focused change; the shape here
//! reserves the slot without faking the behaviour.

use std::collections::HashMap;
use std::convert::Infallible;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};

use torrust_sentinel::BatchReport;

use super::clock::Clock as _;
use super::label_spec::LabelSpec;
use super::liveness::ACK_DEADLINE;
use super::names::{AxisName, ChannelName, IdentityName, NameRegistry, SentinelName};
use super::rng::TestRng;
use super::tolerances::{DEFAULT_TOLERANCES, Tolerances};
use super::virtual_clock::VirtualClock;
use crate::Assayer;
use crate::api::{AssayerBuilder, LabelAck, ReportAck};
use crate::assessment::{DerivedReckoning, RequestContext, RiskAssessment};
use crate::config::types::{AssayerConfig, PersistenceConfig};
use crate::error::{BuildError, ChannelError, LabelError, LifecycleError, ReportError};
use crate::owner::commands::{
    IdentityDimensionRegistration, LabelData, ModelOwnerCommand, OutcomeAxisRegistration, SentinelRegistration,
};
use crate::resonance::channel::{ChannelPolicy, validate_channel_policy};
use crate::resonance::derivation::{ResonanceConfig, render_resonances};
use crate::resonance::landscape::derive_landscape;
use crate::risk::challenge::{ChallengeEffectivenessState, ChallengeEffectivenessTracker};
use crate::signal::SignalDeclaration;
use crate::types::{
    Action, AssessmentId, ChallengeResult, ChannelId, DimensionId, EntityKey, IdentityBudget, OutcomeAxisId, OutcomeEligibility,
    SentinelId, SpatialFeaturePolicy,
};

// ═════════════════════════════════════════════════════════════════════
// WorldBuilder
// ═════════════════════════════════════════════════════════════════════

/// Staged builder for a [`World`].
///
/// Every non-default input is set explicitly; there is no implicit
/// `default()` for the whole thing because determinism requires at
/// minimum a seed.
pub struct WorldBuilder {
    /// Engine configuration.
    config: AssayerConfig,
    /// Signal schema declarations.
    signal_schema: Vec<SignalDeclaration>,
    /// Channels to declare at build time (name, policy).
    channels: Vec<(ChannelName, ChannelPolicy)>,
    /// Seed for the test RNG.
    seed: Option<u64>,
    /// Clock the harness will expose; shared with the engine in Stage 2.
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
            seed: None,
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

    /// Overrides the default tolerances.
    #[must_use]
    pub const fn tolerances(mut self, tolerances: Tolerances) -> Self {
        self.tolerances = tolerances;
        self
    }

    /// Builds the [`World`].
    ///
    /// # Errors
    ///
    /// Returns [`WorldBuildError`] if the underlying engine rejects
    /// the configuration (invalid channel policy, malformed schema,
    /// failed startup).
    ///
    /// # Panics
    ///
    /// Panics if [`Self::seed`] was not called — determinism is a
    /// contract, not a default.
    pub fn build(self) -> Result<World, WorldBuildError> {
        let seed = self
            .seed
            .expect("WorldBuilder::seed(…) is required — reproducibility is mandatory");

        let mut names = NameRegistry::default();

        // The engine reads time through the same clock the harness
        // exposes, so `World::advance` moves the engine's present and
        // an untouched clock keeps it still.
        let engine_clock: Arc<dyn super::clock::Clock> = self.clock.clone();
        let builder: AssayerBuilder = Assayer::builder(self.config)
            .signal_schema(&self.signal_schema)
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

        Ok(World {
            assayer,
            clock: self.clock,
            rng: TestRng::new(seed),
            names,
            channel_policies,
            challenge_tracker: Mutex::new(challenge_tracker),
            assessment_channels: Mutex::new(HashMap::new()),
            tol: self.tolerances,
        })
    }
}

/// Errors that can arise when building a [`World`].
#[derive(Debug)]
pub enum WorldBuildError {
    /// Channel policy rejected by the engine.
    Channel(ChannelError),
    /// Engine startup failed.
    Build(BuildError),
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
            Self::Channel(e) => write!(f, "channel declaration failed: {e}"),
            Self::Build(e) => write!(f, "assayer build failed: {e}"),
        }
    }
}

impl std::error::Error for WorldBuildError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Channel(e) => Some(e),
            Self::Build(e) => Some(e),
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
    /// Deterministic clock shared with the engine (Stage 2).
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
    /// Convenience for scenarios whose shape is fully captured by
    /// "fresh engine, one 4-action channel, seeded RNG" — which is
    /// the majority of the Stage 1 integration tests. Tests that
    /// need multiple channels, a non-default policy, or a shared
    /// clock should still spell out the full [`Self::builder`] call.
    ///
    /// # Panics
    ///
    /// Panics if the engine rejects the default configuration; this
    /// would indicate a regression in the crate itself, not a test
    /// error, so failing loudly is the correct behaviour.
    #[must_use]
    pub fn cold(instance_id: &str, seed: u64) -> Self {
        Self::builder(AssayerConfig {
            instance_id: instance_id.to_owned(),
            // The capacity is host-set with no default; the harness
            // declares the fixture value.
            infrastructure: super::test_infrastructure(),
            ..Default::default()
        })
        .channel("default", ChannelPolicy::default())
        .seed(seed)
        .build()
        .expect("cold world should build with default channel policy")
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
    /// Uses the conventional harness encoding: the first eight bytes
    /// of the entity key are interpreted as a little-endian `u64` and
    /// zero-extended to a 128-bit coordinate (matching
    /// [`src/tests/lifecycle_integration.rs`](../../../src/tests/lifecycle_integration.rs)
    /// `identity_reg`). `domain_bits` and `depth_cutoff` take the
    /// specification-default values (128 and 10), and
    /// [`IdentityBudget::default()`] supplies the resource budget.
    pub fn register_identity(&mut self, name: impl Into<IdentityName>) -> Result<DimensionId, LifecycleError> {
        let name = name.into();
        let id = self.names.alloc_identity_id();
        self.assayer.register_identity_dimension(IdentityDimensionRegistration {
            id,
            name: name.0.to_owned(),
            description: "scenario identity hierarchy".to_owned(),
            coordinate_semantics: "the leading bytes select a prefix group".to_owned(),
            domain_bits: 128,
            depth_cutoff: 10,
            budget: IdentityBudget::for_depth_cutoff(10),
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

    /// Record a challenge outcome, then submit the core label data.
    pub fn label_with_challenge_result(&self, data: LabelData, result: ChallengeResult) -> Result<LabelAck, LabelError> {
        let _recorded = self.record_challenge_result(data.assessment_id, result);
        self.label(data)
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
/// [`World::cycle_on`] covers named-channel/entity requests. The enrichment-grid
/// tests
/// (´claim:channel:a-channels-policy-shape-moves-only-its-action-tags-and-never-the-shared-core´)
/// often need richer public requests (request-scoped signals, reported Sentinel
/// coordinates, or future fixture tapes), so this helper keeps that cycle
/// reusable without adding more production harness surface.
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
