// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Harness for identity-maintenance-loop tests.
//!
//! Provides a small vocabulary on top of the crate's internal
//! `spawn_identity_maintenance_thread` so tests in `src/tests/` and
//! `tests/` can drive the loop without repeating the same
//! boilerplate (spawning the thread, sending a `CreateDimension`
//! command, waiting for acks, draining the model owner channel, etc.).
//!
//! # Shape
//!
//! - [`MaintenanceHarness`] owns the thread handle and both channels.
//!   `Drop` sends `Shutdown` and joins the thread, so tests never leak.
//! - [`setup_dimension`] builds a matching pair of the crate's
//!   `IdentityDimensionInfra` + `ObservationChannel`.
//! - [`default_mudlark_config`] / [`aggressive_split_config`] are the
//!   two Mudlark configs most tests reach for.
//! - [`collect_lifecycle_events`] flattens `ModelOwnerCommand`s.
//!
//! The harness deliberately exposes the raw channels and infra so
//! low-level tests (e.g. `src/tests/identity_maintenance.rs`) can
//! still poke at state the DSL does not wrap yet.
//!
//! # Reachable queue inventory
//!
//! - The identity-maintenance command queue and every registered dimension's observation queue are crossed by [`MaintenanceHarness::flush_identity_maintenance`]. Its checkpoint marker runs after earlier maintenance commands, drains the observation queues and deferred signal-cache writes, detects competitive-set changes, offers the resulting lifecycle submissions to the model-owner queue, and publishes graph snapshots before acknowledging. It does not cover work offered after those drains, periodic decay that is not yet due, or application of lifecycle submissions by a model owner.
//! - The maintenance seam's model-owner queue is exposed as [`MaintenanceHarness::model_owner_rx`]. [`MaintenanceHarness::flush_identity_maintenance`] guarantees that the completed cycle has offered its lifecycle submissions before return, so a following [`MaintenanceHarness::drain_model_owner`] observes every submission that the bounded queue accepted. The barrier does not make a saturated queue accept a submission and this specialised seam deliberately runs no model owner that could apply or publish one.
//! - The shared scenario's identity path is crossed by [`super::World::flush_identity_maintenance`]. It composes the maintenance acknowledgement with the model-owner publication barrier, so lifecycle submissions accepted from that maintenance cycle have been applied and published on return. Its exclusions are documented on that verb.
//! - The model-owner command and label queues are crossed by [`super::World::flush_labels`], while cold-ramp observations are crossed by [`super::World::flush_observations`]. Each verb documents the other queue as excluded; their existing owners retain those barriers.
//! - The periodic Ledger-GC cycle is crossed by [`super::World::flush_ledger_gc`]. Its explicit trigger carries the scheduler's merged outcome only after every Sentinel visible to that cycle has been swept. It does not cover a Sentinel registered after the cycle takes its Sentinel-id snapshot, an entry inserted after its Ledger takes the cycle's key snapshot, model-owner publication, or a later periodic cycle.
//! - The checkpoint-scheduler trigger queue is crossed by [`super::World::flush_checkpoint_scheduler`]. Its explicit trigger is acknowledged only after the scheduler has placed the checkpoint request on the model-owner command queue. It does not wait for the owner to process that downstream request, drain its labels, write the checkpoint, or publish; [`super::World::flush_labels`] crosses that separate queue.
//! - The health-event receiver is crossed by [`super::World::drain_health_events`]. Every production health-event sender belongs to the model-owner thread, so the verb first crosses identity maintenance and its resulting owner work, then the cold-ramp observation queue, then the owner command and label queues once more before draining. It returns every event the bounded receiver accepted from work offered before those crossings and leaves the receiver empty at the drain point; it does not recreate an event refused by the bounded receiver, include work offered after its queue was crossed, or run a future periodic scheduler cycle.
//! - Crate-local snapshot-version and publication polls, and waits in the specialised multi-channel harness, do not identify additional queues: they duplicate completion meaning for the model-owner, label, or cold-ramp queues above. Their migration remains with the polling-retirement and scenario-unification entries rather than widening the identity-maintenance barrier (´entry:assayer:harness-polling-retirement´) (´entry:assayer:harness-scenario-unification´).
//! - The inventory is at its fixpoint: every asynchronous queue reachable by a test has the queue-specific barrier named above, and no reachable queue remains uncovered. The closed-barrier-set entry is therefore closed (´entry:assayer:harness-closed-barriers´).

use std::collections::BTreeMap;
use std::sync::Arc;
use std::thread::JoinHandle;

use crossbeam_channel::{self, Receiver, Sender};
use torrust_mudlark::Config as MudlarkConfig;

use super::liveness::ACK_DEADLINE;
use crate::identity::{
    CompetitiveCellId, IdentityDimension, IdentityDimensionInfra, MaintenanceCommand, ObservationChannel,
    spawn_identity_maintenance_thread,
};
use crate::owner::commands::{LifecycleEvent, ModelOwnerCommand};
use crate::types::{DimensionId, EntityKey};

const FIXTURE_OBSERVATION_CAPACITY: usize = 10_000;
const MAINTENANCE_COMMAND_CAPACITY: usize = 64;

// ═════════════════════════════════════════════════════════════════════
// Mudlark config fixtures
// ═════════════════════════════════════════════════════════════════════

/// Default Mudlark config for maintenance-loop tests.
///
/// Low split threshold so splits trigger easily on modest observation
/// counts.
#[must_use]
pub const fn default_mudlark_config() -> MudlarkConfig<u64> {
    MudlarkConfig {
        split_threshold: 5,
        depth_create: 20,
        depth_evict: 30,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

/// Very aggressive split config: threshold 3, generous depth limits.
#[must_use]
pub const fn aggressive_split_config() -> MudlarkConfig<u64> {
    MudlarkConfig {
        split_threshold: 3,
        depth_create: 40,
        depth_evict: 60,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

// ═════════════════════════════════════════════════════════════════════
// Dimension fixture
// ═════════════════════════════════════════════════════════════════════

/// One graph snapshot's complete fixture-relevant reading.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceGraphBaseline {
    /// Whether the graph snapshot carries no nodes.
    pub is_empty: bool,
    /// Total observation energy retained by the graph.
    pub total_energy: u64,
    /// Number of terminal cells retained by the graph.
    pub terminal_count: usize,
}

/// Complete state measured from the dimension fixture and both views of its observation queue.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceDimensionBaseline {
    /// Dimension identifier.
    pub dimension_id: DimensionId,
    /// Human-readable dimension name.
    pub name: String,
    /// Host description of the key space.
    pub description: String,
    /// Host declaration of coordinate meaning.
    pub coordinate_semantics: String,
    /// Domain bit width.
    pub domain_bits: u8,
    /// Maximum competitive-cell depth.
    pub depth_cutoff: u8,
    /// Encoder result for the empty key.
    pub encoded_empty_key: u128,
    /// Encoder result for a non-empty fixture probe key.
    pub encoded_probe_key: u128,
    /// Every published competitive cell.
    pub competitive_cells: Vec<CompetitiveCellId>,
    /// Published total importance, represented by its exact floating-point bits.
    pub published_total_importance_bits: u64,
    /// Every cell carrying mutable state.
    pub tracked_cells: Vec<CompetitiveCellId>,
    /// Observations dropped by the shared infrastructure.
    pub infrastructure_observations_dropped: u64,
    /// Complete active-indicator count distribution.
    pub active_indicator_counts: BTreeMap<usize, u64>,
    /// Published graph snapshot, when one exists.
    pub graph_snapshot: Option<MaintenanceGraphBaseline>,
    /// Capacity declared by the observation-channel fixture.
    pub observation_capacity: usize,
    /// Pending observations read through the observation-channel fixture.
    pub observation_pending: usize,
    /// Overflows recorded by the observation-channel fixture.
    pub observation_overflow_count: u64,
    /// Capacity read back through the infrastructure's sender.
    pub infrastructure_observation_capacity: Option<usize>,
    /// Pending observations read back through the infrastructure's sender.
    pub infrastructure_observation_pending: usize,
}

/// Complete state measured from a newly spawned maintenance harness.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceHarnessBaseline {
    /// Whether the harness still owns the thread handle.
    pub thread_handle_present: bool,
    /// Name read from the spawned thread.
    pub thread_name: Option<String>,
    /// Whether the thread is still live at the fixture boundary.
    pub thread_alive: bool,
    /// Capacity read back from the maintenance-command queue.
    pub command_capacity: Option<usize>,
    /// Occupancy read back from the maintenance-command queue.
    pub command_pending: usize,
    /// Capacity read back from the model-owner queue.
    pub model_owner_capacity: Option<usize>,
    /// Occupancy read back from the model-owner queue.
    pub model_owner_pending: usize,
}

/// Complete maintenance state measured at a guarded fixture boundary.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MaintenanceFixtureBaseline {
    /// Dimension and observation-queue construction state.
    Dimension(MaintenanceDimensionBaseline),
    /// Maintenance-thread and channel construction state.
    Harness(MaintenanceHarnessBaseline),
}

/// Maintenance fixture whose return boundary applies a baseline guard.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum MaintenanceFixture {
    /// The dimension and observation-channel constructor.
    SetupDimension,
    /// The default-capacity maintenance-harness constructor.
    Spawn,
    /// The explicit-capacity maintenance-harness constructor.
    SpawnWithCapacity,
}

impl std::fmt::Display for MaintenanceFixture {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::SetupDimension => formatter.write_str("setup_dimension"),
            Self::Spawn => formatter.write_str("MaintenanceHarness::spawn"),
            Self::SpawnWithCapacity => formatter.write_str("MaintenanceHarness::spawn_with_capacity"),
        }
    }
}

/// Independently derived declaration for one maintenance fixture.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct MaintenanceFixtureDeclaration {
    /// Fixture that owns the declaration.
    pub fixture: MaintenanceFixture,
    /// Exact state expected at the fixture boundary.
    pub initial: MaintenanceFixtureBaseline,
}

impl MaintenanceFixtureDeclaration {
    /// Derives the complete state declared by [`setup_dimension`].
    #[must_use]
    pub fn setup_dimension(dim_id: u32) -> Self {
        Self {
            fixture: MaintenanceFixture::SetupDimension,
            initial: MaintenanceFixtureBaseline::Dimension(MaintenanceDimensionBaseline {
                dimension_id: DimensionId(dim_id),
                name: format!("test-{dim_id}"),
                description: "test hierarchy".to_owned(),
                coordinate_semantics: "prefix groups".to_owned(),
                domain_bits: 128,
                depth_cutoff: 24,
                encoded_empty_key: 0,
                encoded_probe_key: 0,
                competitive_cells: Vec::new(),
                published_total_importance_bits: 0.0_f64.to_bits(),
                tracked_cells: Vec::new(),
                infrastructure_observations_dropped: 0,
                active_indicator_counts: BTreeMap::new(),
                graph_snapshot: None,
                observation_capacity: FIXTURE_OBSERVATION_CAPACITY,
                observation_pending: 0,
                observation_overflow_count: 0,
                infrastructure_observation_capacity: Some(FIXTURE_OBSERVATION_CAPACITY),
                infrastructure_observation_pending: 0,
            }),
        }
    }

    /// Derives the complete state declared by [`MaintenanceHarness::spawn`].
    #[must_use]
    pub fn spawn(instance_id: &str) -> Self {
        Self::harness(
            MaintenanceFixture::Spawn,
            instance_id,
            MaintenanceHarness::DEFAULT_MODEL_OWNER_CAPACITY,
        )
    }

    /// Derives the complete state declared by [`MaintenanceHarness::spawn_with_capacity`].
    #[must_use]
    pub fn spawn_with_capacity(instance_id: &str, model_owner_capacity: usize) -> Self {
        Self::harness(MaintenanceFixture::SpawnWithCapacity, instance_id, model_owner_capacity)
    }

    fn harness(fixture: MaintenanceFixture, instance_id: &str, model_owner_capacity: usize) -> Self {
        Self {
            fixture,
            initial: MaintenanceFixtureBaseline::Harness(MaintenanceHarnessBaseline {
                thread_handle_present: true,
                thread_name: Some(format!("{instance_id}-identity-maintenance")),
                thread_alive: true,
                command_capacity: Some(MAINTENANCE_COMMAND_CAPACITY),
                command_pending: 0,
                model_owner_capacity: Some(model_owner_capacity),
                model_owner_pending: 0,
            }),
        }
    }

    fn admits(&self, fixture: MaintenanceFixture, measured: &MaintenanceFixtureBaseline) -> bool {
        self.fixture == fixture && measured == &self.initial
    }
}

impl MaintenanceFixture {
    /// Refuses measured maintenance state that does not satisfy the fixture declaration.
    ///
    /// On success, returns the unchanged measured baseline to the guarded constructor.
    ///
    /// # Errors
    ///
    /// Returns [`MaintenanceFixtureError::BaselineMismatch`] with the complete declaration and measured baseline when any declared fact differs.
    pub fn guard_baseline(
        self,
        expected: &MaintenanceFixtureDeclaration,
        measured: MaintenanceFixtureBaseline,
    ) -> Result<MaintenanceFixtureBaseline, MaintenanceFixtureError> {
        if !expected.admits(self, &measured) {
            return Err(MaintenanceFixtureError::BaselineMismatch {
                fixture: self,
                expected: Box::new(expected.clone()),
                measured: Box::new(measured),
            });
        }
        Ok(measured)
    }
}

/// Error raised when a maintenance fixture does not reproduce its complete declaration.
#[derive(Debug, PartialEq, Eq)]
pub enum MaintenanceFixtureError {
    /// A fixture's measured state differs from its declaration.
    BaselineMismatch {
        /// Fixture applying the guard.
        fixture: MaintenanceFixture,
        /// Complete fixture declaration.
        expected: Box<MaintenanceFixtureDeclaration>,
        /// Complete state read back at the fixture boundary.
        measured: Box<MaintenanceFixtureBaseline>,
    },
}

impl std::fmt::Display for MaintenanceFixtureError {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::BaselineMismatch {
                fixture,
                expected,
                measured,
            } => write!(
                formatter,
                "{fixture} measured maintenance baseline {measured:?} does not satisfy declaration {expected:?}"
            ),
        }
    }
}

impl std::error::Error for MaintenanceFixtureError {}

fn sorted_cells(cells: impl Iterator<Item = CompetitiveCellId>) -> Vec<CompetitiveCellId> {
    let mut cells: Vec<_> = cells.collect();
    cells.sort_by_key(|cell| (cell.depth, cell.lo));
    cells
}

/// Reads back the complete dimension and observation-queue state established by [`setup_dimension`].
#[must_use]
pub fn dimension_fixture_baseline(
    infra: &IdentityDimensionInfra,
    observation: &ObservationChannel,
) -> MaintenanceFixtureBaseline {
    let competitive_set = infra.load_competitive_set();
    let competitive_cells = sorted_cells(competitive_set.cell_ids().copied());
    let published_total_importance_bits = competitive_set.total_importance().to_bits();
    drop(competitive_set);
    let tracked_cells = {
        let state = infra.cell_state.read().unwrap_or_else(std::sync::PoisonError::into_inner);
        sorted_cells(state.keys().copied())
    };
    let graph_snapshot = infra.graph_snapshot.load();
    let graph_snapshot = graph_snapshot.as_ref().as_ref().map(|snapshot| MaintenanceGraphBaseline {
        is_empty: snapshot.is_empty(),
        total_energy: snapshot.total_energy(),
        terminal_count: snapshot.terminal_count(),
    });

    MaintenanceFixtureBaseline::Dimension(MaintenanceDimensionBaseline {
        dimension_id: infra.dimension.id,
        name: infra.dimension.name.clone(),
        description: infra.dimension.description.clone(),
        coordinate_semantics: infra.dimension.coordinate_semantics.clone(),
        domain_bits: infra.dimension.domain_bits,
        depth_cutoff: infra.dimension.depth_cutoff,
        encoded_empty_key: infra.dimension.encode_entity(&EntityKey::new(Vec::<u8>::new())),
        encoded_probe_key: infra.dimension.encode_entity(&EntityKey::from(&b"fixture-probe"[..])),
        competitive_cells,
        published_total_importance_bits,
        tracked_cells,
        infrastructure_observations_dropped: infra.observations_dropped(),
        active_indicator_counts: infra.active_indicator_count_distribution(),
        graph_snapshot,
        observation_capacity: observation.capacity(),
        observation_pending: observation.len(),
        observation_overflow_count: observation.overflow_count(),
        infrastructure_observation_capacity: infra.observation_tx.capacity(),
        infrastructure_observation_pending: infra.observation_tx.len(),
    })
}

/// Creates a test dimension + shared infra + observation channel.
///
/// The dimension has a trivial encoder (`|_| 0`) and a 10 000-slot observation channel. Before returning, the fixture reads back the full dimension declaration, both views of the empty observation queue, and every initially empty published or mutable state surface, and refuses a mismatch with the complete measured baseline.
///
/// # Panics
///
/// Panics only if the values constructed entirely within this fixture do not reproduce its declaration, which no caller input other than the dimension identifier can arrange.
#[must_use]
pub fn setup_dimension(dim_id: u32) -> (Arc<IdentityDimensionInfra>, ObservationChannel) {
    let declaration = MaintenanceFixtureDeclaration::setup_dimension(dim_id);
    let dim = IdentityDimension::new(
        DimensionId(dim_id),
        format!("test-{dim_id}"),
        "test hierarchy",
        "prefix groups",
        128,
        24,
        |_| 0,
    );
    let obs = ObservationChannel::with_capacity(FIXTURE_OBSERVATION_CAPACITY);
    let infra = Arc::new(IdentityDimensionInfra::new(dim, obs.sender()));
    MaintenanceFixture::SetupDimension
        .guard_baseline(&declaration, dimension_fixture_baseline(&infra, &obs))
        .expect("setup_dimension should reproduce its complete declaration");
    (infra, obs)
}

// ═════════════════════════════════════════════════════════════════════
// Lifecycle event helpers
// ═════════════════════════════════════════════════════════════════════

/// Flatten every `LifecycleEvent` out of a batch of `ModelOwnerCommand`s.
#[must_use]
pub fn collect_lifecycle_events(cmds: &[ModelOwnerCommand]) -> Vec<&LifecycleEvent> {
    cmds.iter()
        .filter_map(|cmd| {
            if let ModelOwnerCommand::Lifecycle(sub) = cmd {
                Some(sub.events.iter())
            } else {
                None
            }
        })
        .flatten()
        .collect()
}

// ═════════════════════════════════════════════════════════════════════
// MaintenanceHarness
// ═════════════════════════════════════════════════════════════════════

/// Test harness for the crate's internal `spawn_identity_maintenance_thread`.
///
/// The harness owns the spawned thread and its command / model-owner
/// channels. Dropping it sends `Shutdown` and joins the thread so
/// tests never leak threads on panic.
pub struct MaintenanceHarness {
    handle: Option<JoinHandle<()>>,
    command_tx: Sender<MaintenanceCommand>,
    /// Commands emitted by the maintenance loop toward the model owner.
    pub model_owner_rx: Receiver<ModelOwnerCommand>,
}

impl MaintenanceHarness {
    /// Default capacity for the model-owner command channel.
    ///
    /// A harness-side depth, wide enough that a test draining the receiver
    /// after the loop has run never blocks the maintenance thread against it.
    ///
    /// It has no counterpart in a deployment, so nothing outside the harness
    /// fixes it: the harness declares it, chosen wide rather than derived, and
    /// what it must be is wider than any single test's emission
    /// (´tab:assayer:harness-construction-figures´).
    ///
    /// ´const:assayer:model-owner-channel-depth´ (´alg:const:count´)
    /// ´const:assayer:model-owner-channel-depth-count-256´
    pub const DEFAULT_MODEL_OWNER_CAPACITY: usize = 256;

    /// Spawns a maintenance thread with the default model-owner channel capacity ([`Self::DEFAULT_MODEL_OWNER_CAPACITY`]). The fixture returns only after reading back the live thread's name and both empty bounded queues against its complete declaration.
    ///
    /// # Panics
    ///
    /// Panics if the platform refuses to spawn the named thread or if the spawned harness does not reproduce its complete declaration.
    #[must_use]
    pub fn spawn(instance_id: &str) -> Self {
        let declaration = MaintenanceFixtureDeclaration::spawn(instance_id);
        Self::spawn_declared(&declaration, instance_id, Self::DEFAULT_MODEL_OWNER_CAPACITY)
    }

    /// Spawns a maintenance thread with an explicit model-owner channel capacity, useful for back-pressure tests. The fixture returns only after reading back the live thread's name and both empty bounded queues against its complete declaration.
    ///
    /// # Panics
    ///
    /// Panics if the platform refuses to spawn the named thread or if the spawned harness does not reproduce its complete declaration.
    #[must_use]
    pub fn spawn_with_capacity(instance_id: &str, model_owner_capacity: usize) -> Self {
        let declaration = MaintenanceFixtureDeclaration::spawn_with_capacity(instance_id, model_owner_capacity);
        Self::spawn_declared(&declaration, instance_id, model_owner_capacity)
    }

    fn spawn_declared(declaration: &MaintenanceFixtureDeclaration, instance_id: &str, model_owner_capacity: usize) -> Self {
        let (model_owner_tx, model_owner_rx) = crossbeam_channel::bounded::<ModelOwnerCommand>(model_owner_capacity);
        let (handle, command_tx) = spawn_identity_maintenance_thread(
            instance_id,
            model_owner_tx,
            std::sync::Arc::new(super::SystemClock),
            std::sync::Arc::new(crate::signal::SignalCache::new(
                1,
                std::sync::Arc::new(crate::signal::SignalSchemaIndex::from_declarations(&[]).expect("empty schema")),
            )),
        );
        let harness = Self {
            handle: Some(handle),
            command_tx,
            model_owner_rx,
        };
        declaration
            .fixture
            .guard_baseline(declaration, harness.maintenance_baseline())
            .expect("maintenance harness should reproduce its complete declaration");
        harness
    }

    /// Access the raw `command_tx` sender. Prefer the named verbs on
    /// the harness when one exists.
    #[must_use]
    pub const fn command_tx(&self) -> &Sender<MaintenanceCommand> {
        &self.command_tx
    }

    /// The spawned thread's name.
    ///
    /// # Panics
    ///
    /// Panics if the harness has already been joined.
    #[must_use]
    pub fn thread_name(&self) -> String {
        self.handle
            .as_ref()
            .and_then(|h| h.thread().name().map(str::to_owned))
            .unwrap_or_default()
    }

    /// Reads back the complete thread and queue state established by a maintenance-harness constructor.
    #[must_use]
    pub fn maintenance_baseline(&self) -> MaintenanceFixtureBaseline {
        let thread_handle_present = self.handle.is_some();
        let thread_name = self
            .handle
            .as_ref()
            .and_then(|handle| handle.thread().name().map(str::to_owned));
        let thread_alive = self.handle.as_ref().is_some_and(|handle| !handle.is_finished());
        MaintenanceFixtureBaseline::Harness(MaintenanceHarnessBaseline {
            thread_handle_present,
            thread_name,
            thread_alive,
            command_capacity: self.command_tx.capacity(),
            command_pending: self.command_tx.len(),
            model_owner_capacity: self.model_owner_rx.capacity(),
            model_owner_pending: self.model_owner_rx.len(),
        })
    }

    /// Registers a dimension with the maintenance loop using [`default_mudlark_config`] and waits for the acknowledgement.
    ///
    /// This item is outside the guarded-fixture population: it is a convenience form of the acknowledged [`Self::register_dimension_with_config`] command primitive, and a consuming fixture owns any stronger reusable postcondition.
    pub fn register_dimension(
        &self,
        dimension_id: DimensionId,
        depth_cutoff: u8,
        observation_rx: Receiver<u128>,
        infra: Arc<IdentityDimensionInfra>,
    ) {
        self.register_dimension_with_config(default_mudlark_config(), dimension_id, depth_cutoff, observation_rx, infra);
    }

    /// Registers a dimension with a caller-supplied Mudlark config and waits for the acknowledgement.
    ///
    /// This item is outside the guarded-fixture population: the command carries the complete configuration, identifier, cutoff, queue, and infrastructure into the loop, and its acknowledgement says that exact declaration has been installed. A second baseline would duplicate that acknowledgement rather than establish a stronger fixture state.
    pub fn register_dimension_with_config(
        &self,
        config: MudlarkConfig<u64>,
        dimension_id: DimensionId,
        depth_cutoff: u8,
        observation_rx: Receiver<u128>,
        infra: Arc<IdentityDimensionInfra>,
    ) {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        self.command_tx
            .send(MaintenanceCommand::CreateDimension {
                config,
                dimension_id,
                depth_cutoff,
                spatial_decay_rate: 0.998,
                observation_rx,
                infra,
                restored: None,
                ack: ack_tx,
            })
            .expect("send CreateDimension");
        ack_rx.recv_timeout(ACK_DEADLINE).expect("dimension creation ack");
    }

    /// Sends `DestroyDimension` without awaiting an acknowledgement; the command itself has no acknowledgement channel.
    ///
    /// This item is outside the guarded-fixture population: it is a fire-and-forget command primitive, not a constructor of reusable subject state. A caller that needs ordering follows it with [`Self::flush_identity_maintenance`], and the consuming oracle decides what release means for the scenario.
    pub fn destroy_dimension(&self, dimension_id: DimensionId) {
        self.command_tx
            .send(MaintenanceCommand::DestroyDimension { dimension_id })
            .expect("send DestroyDimension");
    }

    /// Crosses the identity-maintenance queue and waits for the loop's acknowledgement.
    ///
    /// # What returning covers, and what it does not
    ///
    /// The `PrepareCheckpoint` marker follows every earlier maintenance command. Its handler drains every registered observation queue and the deferred signal-cache writes, detects competitive-set changes, offers their lifecycle submissions to the model-owner queue, and publishes fresh graph snapshots before acknowledging under [`ACK_DEADLINE`]. The acknowledgement is the loop's own completion signal; no elapsed-time observation participates.
    ///
    /// Work offered after the handler drains its queue is not covered. Neither is periodic decay that is not yet due, application of an accepted lifecycle submission by a model owner, nor a lifecycle submission that the bounded model-owner queue refused. This specialised harness exposes that receiver directly; [`super::World::flush_identity_maintenance`] is the composed barrier for tests that run the real model owner and need accepted lifecycle changes published there.
    ///
    /// This item is outside the guarded-fixture population: it is the module's queue barrier, and its acknowledgement already carries the completion declaration that callers require.
    pub fn flush_identity_maintenance(&self) {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        self.command_tx
            .send(MaintenanceCommand::PrepareCheckpoint(ack_tx))
            .expect("send PrepareCheckpoint");
        ack_rx.recv_timeout(ACK_DEADLINE).expect("identity-maintenance barrier ack");
    }

    /// Forces decay on a dimension and waits for the ack.
    ///
    /// Decay is a pure value operation — call [`Self::flush_identity_maintenance`] afterwards to trigger change detection on the barrier cycle.
    ///
    /// This item is outside the guarded-fixture population: its acknowledgement carries completion of the requested graph transformation, while a baseline would have to compare the entire graph distribution before and after attenuation. That comparison is the consuming result oracle, not a reusable fixture precondition.
    ///
    /// Only available under `#[cfg(test)]` because the underlying
    /// [`MaintenanceCommand::ForceDecay`] variant is itself
    /// test-only.
    #[cfg(test)]
    pub fn force_decay(&self, dimension_id: DimensionId, attenuation: f64) {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        self.command_tx
            .send(MaintenanceCommand::ForceDecay {
                dimension_id,
                attenuation,
                ack: ack_tx,
            })
            .expect("send ForceDecay");
        ack_rx.recv_timeout(ACK_DEADLINE).expect("force_decay ack");
    }

    /// Drains every currently-pending `ModelOwnerCommand`.
    #[must_use]
    pub fn drain_model_owner(&self) -> Vec<ModelOwnerCommand> {
        let mut cmds = Vec::new();
        while let Ok(cmd) = self.model_owner_rx.try_recv() {
            cmds.push(cmd);
        }
        cmds
    }

    /// Drops every currently-pending `ModelOwnerCommand` without
    /// returning them. Use when a test only wants to reset the channel
    /// between phases.
    pub fn clear_model_owner(&self) {
        while self.model_owner_rx.try_recv().is_ok() {}
    }

    /// Shuts the loop down explicitly and joins the thread.
    ///
    /// Equivalent to just dropping the harness, but useful when a test
    /// wants to assert the thread exited cleanly before the end of
    /// scope.
    pub fn shutdown(mut self) {
        self.shutdown_in_place();
    }

    fn shutdown_in_place(&mut self) {
        if let Some(handle) = self.handle.take() {
            drop(self.command_tx.send(MaintenanceCommand::Shutdown));
            handle.join().expect("maintenance thread should exit cleanly");
        }
    }
}

impl Drop for MaintenanceHarness {
    fn drop(&mut self) {
        self.shutdown_in_place();
    }
}
