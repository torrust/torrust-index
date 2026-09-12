// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`maintenance_thread_starts_and_shuts_down`] | identity | The maintenance thread comes up on demand and stands down when told to, leaving its join to succeed rather than panicking or hanging. The loop otherwise sits in a timed wait forever, so an engine that could not retire it would leak a thread per instance and never shut down cleanly. |
//! | [`decay_cadence_composes_to_hourly_rate`] | identity | Sixty applications of the per-minute attenuation compose to exactly the tabulated hourly spatial decay rate, and that rate halves importance on the fourteen-day timescale the corpus describes — between thirteen and sixteen days — rather than within a day. The cadence is thereby an implementation detail: however often the loop wakes, an hour of decay is one application of the hourly rate, so who stays competitive is governed by the table and not by the loop's schedule. |
//! | [`dimension_state_uses_registered_spatial_decay_rate`] | identity | The dimension state derives its interval attenuation from the host-supplied hourly spatial decay rate rather than the shipped default, and one hour of intervals composes back to that supplied rate. |
//! | [`competitive_exit_resets_measurement_and_reentry_retains_outcome`] | identity | Competitive exit zeros measurement state while preserving outcome state, and re-entry reuses that outcome unchanged instead of replacing the cell with a neutral state. |
//! | [`graph_interval_cleanup_removes_only_evicted_state`] | identity | A budgeted graph is driven until it evicts an interval that existed in the preceding full census. Cleanup removes retained state for that evicted interval while preserving the root interval that remains live. The removal is therefore tied to real graph eviction rather than merely to competitive standing or a fabricated absent key. |

//! Identity maintenance loop and command types.
//!
// This loop is the dedicated owner: it holds every identity graph, and the
// assessment path never mutates one (´dec:memory:graph-owner´).
#![allow(dead_code)]
//!
//! This module provides the maintenance loop that runs on a dedicated thread
//! and manages identity dimension state:
//!
//! - [`MaintenanceCommand`] — Commands from model owner to maintenance loop
//! - [`IdentityMaintenanceLoop`] — The maintenance thread implementation
//!
//! # Thread Architecture
//!
//! The identity maintenance loop is a separate thread that:
//! 1. Drains observation queues from `assess()` threads
//! 2. Applies observations to G-V graphs (with `Δ=1`)
//! 3. Detects competitive set changes from graph splits/merges
//! 4. Emits lifecycle events for cell entries and exits
//! 5. Coordinates with checkpoints by draining queues and publishing snapshots
//!
//! # Cross-References
//!
//! - (´dec:memory:maintenance-cadences´) — why the cycle and the decay answer
//!   to different things
//! - (´alg:keyspace:cell-entry´) — what an entry into the competitive set owes
//! - (´alg:keyspace:cell-exit´) — and what an exit owes
//! - (´dec:memory:competitive-publication´) — the per-dimension swap this loop
//!   publishes through

use std::collections::{HashMap, HashSet};
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};

use crossbeam_channel::{Receiver, Sender, TryRecvError};
use torrust_mudlark::{Config as MudlarkConfig, GvGraph};
use tracing::{debug, info, trace, warn};

use super::competitive::{CompetitiveCellId, CompetitiveSetIndex};
use super::snapshot::IdentityGraphSnapshot;
use super::{CellMutableState, IdentityDimensionInfra};
use crate::owner::commands::{LifecycleEvent, LifecycleSubmission, ModelOwnerCommand};
use crate::types::DimensionId;

/// Applies the competitive lifecycle split to per-cell state.
///
/// Exit discards measurement evidence only. An entry reuses any retained cell
/// and therefore warm-starts its outcome side, while a genuinely new interval
/// receives neutral state (´tab:keyspace:measurement-state´).
fn apply_competitive_cell_state_changes(
    cell_state: &mut HashMap<CompetitiveCellId, Mutex<CellMutableState>>,
    exits: &[CompetitiveCellId],
    entries: &[CompetitiveCellId],
) {
    for cell in exits {
        if let Some(state) = cell_state.get(cell) {
            let mut state = state.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
            state.measurement = super::MeasurementState::default();
        }
    }
    for cell in entries {
        cell_state.entry(*cell).or_insert_with(|| Mutex::new(CellMutableState::new()));
    }
}

/// Removes state whose exact interval no longer exists in the graph.
///
/// A full PEWEI is the graph's interval-existence query: both its terminal and
/// transition entries are live graph intervals. Competitive exit alone does
/// not remove an interval from this census, while graph eviction does.
fn graph_interval_ids(graph: &GvGraph<u128, u64, 128>) -> HashSet<CompetitiveCellId> {
    let pewei = graph.extract();
    let mut existing = HashSet::new();
    for layer in pewei.layers {
        for terminal in layer.terminals {
            #[allow(clippy::cast_possible_truncation)]
            let depth = terminal.depth as u8;
            existing.insert(CompetitiveCellId::new(terminal.start, depth));
        }
        for transition in layer.transitions {
            #[allow(clippy::cast_possible_truncation)]
            let depth = transition.depth as u8;
            existing.insert(CompetitiveCellId::new(transition.start, depth));
        }
    }
    existing
}

/// Drops retained cell state unless the cell's exact interval is still present
/// in the graph's full interval census.
fn retain_existing_graph_intervals(
    cell_state: &mut HashMap<CompetitiveCellId, Mutex<CellMutableState>>,
    graph: &GvGraph<u128, u64, 128>,
) {
    let existing = graph_interval_ids(graph);
    cell_state.retain(|cell, _| existing.contains(cell));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Default maintenance loop cycle interval.
///
/// The wait below is on the command channel, so a command wakes the loop at
/// once and this figure governs only what nothing signals: observations are
/// drained by polling rather than by a wakeup, so it bounds how long a recorded
/// coordinate waits before it reaches a graph, and how long a competitive-set
/// change waits before it is published (´dec:memory:maintenance-cadences´).
///
/// A shipped operating point; nothing here measures a pass against it. It is
/// not independent of the decay cadence below, whose tolerated overshoot this
/// figure bounds (´rem:memory:decay-cadence-composes´).
///
/// ´const:assayer:maintenance-cycle-cadence´ (´alg:const:form´)
/// ´const:assayer:maintenance-cycle-cadence-form-x53c90da6´
const DEFAULT_CYCLE_INTERVAL: Duration = Duration::from_millis(100);

/// Default hourly spatial decay rate for graph importance
/// (´tab:keyspace:decay-rates´): half-life near fourteen days.
///
/// The rate ships as tabulated (´tab:config:identity´).
///
/// ´const:assayer:identity-cell-forgetting-rate´ (´alg:const:scalar´)
/// ´const:assayer:identity-cell-forgetting-rate-scalar-0p998´
const DEFAULT_HOURLY_ATTENUATION: f64 = 0.998;

/// Decay cycle interval (how often to apply temporal decay).
///
/// Not a second statement of the decay timescale: what is applied each interval
/// is the hourly rate above re-derived for that interval, so composing an
/// hour's worth returns the tabulated hourly factor and the cadence leaves the
/// timescale untouched (´dec:memory:maintenance-cadences´).
///
/// Freed of the timescale, the figure answers only to cost against resolution,
/// and is a shipped operating point on that ground. It is bounded below by the
/// wake cadence: the factor is derived from the nominal interval but applied
/// whenever at least that long has passed, so realised decay runs slow by at
/// most one wake per application (´rem:memory:decay-cadence-composes´).
///
/// ´const:assayer:spatial-decay-cadence´ (´alg:const:seconds´)
/// ´const:assayer:spatial-decay-cadence-seconds-60´
const DEFAULT_DECAY_INTERVAL: Duration = Duration::from_secs(60);

/// Converts an hourly decay rate into the per-interval attenuation whose
/// composition over one hour reproduces the hourly rate exactly, so the
/// cadence is an implementation choice rather than a change of timescale
/// (´tab:keyspace:decay-rates´).
fn per_interval_attenuation(hourly_rate: f64, interval: Duration) -> f64 {
    crate::numerics::decay_factor(hourly_rate, interval.as_secs_f64() / 3600.0)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Maintenance Command
// ═══════════════════════════════════════════════════════════════════════════════

/// Commands sent to the identity maintenance loop.
#[derive(Debug)]
pub enum MaintenanceCommand {
    /// Shut down the maintenance loop.
    Shutdown,

    /// Prepare for checkpoint: drain queues, publish snapshots, acknowledge.
    PrepareCheckpoint(Sender<()>),

    /// Create a new identity dimension.
    CreateDimension {
        /// Mudlark configuration for the G-V graph.
        config: MudlarkConfig<u64>,
        /// Dimension identifier.
        dimension_id: DimensionId,
        /// Maximum depth for competitive cells.
        depth_cutoff: u8,
        /// Hourly spatial decay rate for graph importance.
        spatial_decay_rate: f64,
        /// Receiver for observation coordinates.
        observation_rx: Receiver<u128>,
        /// Shared infrastructure for this dimension.
        infra: Arc<IdentityDimensionInfra>,
        /// Acknowledgement channel.
        ack: Sender<()>,
    },

    /// Destroy an identity dimension.
    DestroyDimension {
        /// Dimension to destroy.
        dimension_id: DimensionId,
    },

    /// Force immediate decay on a specific dimension (test-only).
    ///
    /// Applies `attenuation` to the dimension's graph, then runs
    /// change detection. Acknowledges after all side effects complete.
    #[cfg(test)]
    ForceDecay {
        /// Dimension to decay.
        dimension_id: DimensionId,
        /// Attenuation factor (0.0 = total decay, 1.0 = no-op).
        attenuation: f64,
        /// Acknowledgement channel.
        ack: Sender<()>,
    },
}

// ═══════════════════════════════════════════════════════════════════════════════
// Dimension State
// ═══════════════════════════════════════════════════════════════════════════════

/// Per-dimension state held by the maintenance loop.
struct DimensionState {
    /// Dimension identifier.
    id: DimensionId,

    /// The G-V graph tracking importance distribution.
    graph: GvGraph<u128, u64, 128>,

    /// Maximum depth for competitive cells.
    depth_cutoff: u8,

    /// Current competitive set (for change detection).
    current_set: HashSet<CompetitiveCellId>,

    /// Receiver for observation coordinates.
    observation_rx: Receiver<u128>,

    /// Shared infrastructure reference.
    infra: Arc<IdentityDimensionInfra>,

    /// Last decay time.
    last_decay: Instant,

    /// Attenuation factor for decay.
    attenuation: f64,

    /// Decay interval.
    decay_interval: Duration,
}

impl DimensionState {
    /// Creates a new dimension state.
    fn new(
        id: DimensionId,
        config: MudlarkConfig<u64>,
        depth_cutoff: u8,
        spatial_decay_rate: f64,
        observation_rx: Receiver<u128>,
        infra: Arc<IdentityDimensionInfra>,
        now: Instant,
    ) -> Self {
        Self {
            id,
            graph: GvGraph::new(config),
            depth_cutoff,
            current_set: HashSet::new(),
            observation_rx,
            infra,
            last_decay: now,
            attenuation: per_interval_attenuation(spatial_decay_rate, DEFAULT_DECAY_INTERVAL),
            decay_interval: DEFAULT_DECAY_INTERVAL,
        }
    }

    /// Returns `true` if temporal decay is due as of `now`.
    fn decay_due(&self, now: Instant) -> bool {
        now.saturating_duration_since(self.last_decay) >= self.decay_interval
    }

    /// Computes the current competitive set from the PEWEI.
    ///
    /// Uses [`GvGraph::extract_to()`] to produce a
    /// depth-limited PEWEI containing only V-tree BFS depths
    /// `0..=depth_cutoff`.  Every cell (terminal and transition) in
    /// the result is collected as a [`CompetitiveCellId`]. The depth
    /// limit is upstream's guarantee, not this loop's
    /// (´[MUDLARK-claim:pewei:limiting-extraction-depth-keeps-at-most-that-many-layers-and-no-more-energy-than-a-full-extraction]´).
    ///
    /// The depth limit avoids extracting the deep V-tree layers that
    /// account for the bulk of nodes in large trees — savings are
    /// exponential in `(D - depth_cutoff)` for balanced trees.
    ///
    /// This naturally tracks structural changes from decay: when
    /// decay causes V-tree rebalancing, cells may shift layers.  A
    /// cell that was inside the cutoff before decay may fall outside
    /// it afterward, producing an exit.  New splits introduce new
    /// cells that enter.
    fn compute_competitive_set(&self) -> HashSet<CompetitiveCellId> {
        let mut set = HashSet::new();

        // No observations yet → no competitive cells.
        if self.graph.total_sum() == 0 {
            return set;
        }

        let pewei = self.graph.extract_to(u32::from(self.depth_cutoff));

        for layer in &pewei.layers {
            for t in &layer.terminals {
                #[allow(clippy::cast_possible_truncation)]
                let depth = t.depth as u8;
                set.insert(CompetitiveCellId::new(t.start, depth));
            }

            for t in &layer.transitions {
                #[allow(clippy::cast_possible_truncation)]
                let depth = t.depth as u8;
                set.insert(CompetitiveCellId::new(t.start, depth));
            }
        }

        set
    }

    /// Extracts a snapshot of the graph for checkpointing.
    ///
    /// Uses [`GvGraph::extract()`] to produce a [`Pewei`] that captures
    /// both terminal intensities and transition baselines, preserving
    /// total energy across serialization round-trips.
    fn extract_snapshot(&self) -> IdentityGraphSnapshot {
        IdentityGraphSnapshot::from_pewei(self.graph.extract())
    }

    /// Publishes the current competitive cells and graph total as one index.
    #[allow(clippy::cast_precision_loss)] // Justified: graph totals are traffic counts and remain exact below 2^53
    fn publish_competitive_index(&self) {
        let total_importance = self.graph.total_sum() as f64;
        let index = CompetitiveSetIndex::from_cells_and_total(self.current_set.iter().copied(), total_importance);
        self.infra.competitive_set.store(Arc::new(index));
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Maintenance Loop
// ═══════════════════════════════════════════════════════════════════════════════

/// The identity maintenance loop running on a dedicated thread.
///
/// Manages G-V graphs for all identity dimensions, processes observations,
/// detects competitive set changes, and coordinates checkpoints.
pub struct IdentityMaintenanceLoop {
    /// Command receiver.
    command_rx: Receiver<MaintenanceCommand>,

    /// Model owner command sender (for lifecycle events).
    model_owner_tx: Sender<ModelOwnerCommand>,

    /// Per-dimension state.
    dimensions: Vec<DimensionState>,

    /// Cycle interval.
    cycle_interval: Duration,

    /// The engine's source of time.
    ///
    /// Decay here fires on an interval rather than on a request, so a
    /// wall-clock reading would make whether it fired at all depend on
    /// how long the surrounding test happened to take.
    clock: Arc<dyn crate::testing::Clock>,

    /// The signal cache whose deferred writes this loop drains: the
    /// assessment path enqueues cache mutations rather than performing
    /// them (´inv:runtime:enumerated-writes´), and this thread applies
    /// them on its cycle beside the identity observation drain.
    signal_cache: Arc<crate::signal::SignalCache>,
}

impl IdentityMaintenanceLoop {
    /// Creates a new maintenance loop.
    pub fn new(
        command_rx: Receiver<MaintenanceCommand>,
        model_owner_tx: Sender<ModelOwnerCommand>,
        clock: Arc<dyn crate::testing::Clock>,
        signal_cache: Arc<crate::signal::SignalCache>,
    ) -> Self {
        Self {
            command_rx,
            model_owner_tx,
            dimensions: Vec::new(),
            cycle_interval: DEFAULT_CYCLE_INTERVAL,
            clock,
            signal_cache,
        }
    }

    /// Runs the maintenance loop until shutdown.
    ///
    /// This method should be called on a dedicated thread.
    pub fn run(mut self) {
        info!("Identity maintenance loop started");

        loop {
            // 1. Check for commands (non-blocking)
            match self.drain_commands() {
                ControlFlow::Continue => {}
                ControlFlow::Shutdown => {
                    info!("Identity maintenance loop shutting down");
                    return;
                }
            }

            // 2. Drain observation queues
            self.drain_observations();

            // 2.5. Apply the signal cache's deferred writes — the
            // assessment path's enqueue, this thread's mutation
            // (´inv:runtime:enumerated-writes´).
            self.signal_cache.drain_deferred_writes();

            // 3. Detect competitive set changes and emit events
            self.detect_and_emit_changes();

            // 4. Periodic decay (if due)
            self.apply_decay();

            // 5. Sleep or wait for commands
            match self.command_rx.recv_timeout(self.cycle_interval) {
                Ok(cmd) => {
                    if matches!(self.handle_command(cmd), ControlFlow::Shutdown) {
                        info!("Identity maintenance loop shutting down");
                        return;
                    }
                }
                Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                    // Normal cycle timeout, continue
                }
                Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                    info!("Command channel disconnected, shutting down");
                    return;
                }
            }
        }
    }

    /// Drains all pending commands.
    fn drain_commands(&mut self) -> ControlFlow {
        loop {
            match self.command_rx.try_recv() {
                Ok(cmd) => {
                    if matches!(self.handle_command(cmd), ControlFlow::Shutdown) {
                        return ControlFlow::Shutdown;
                    }
                }
                Err(TryRecvError::Empty) => return ControlFlow::Continue,
                Err(TryRecvError::Disconnected) => return ControlFlow::Shutdown,
            }
        }
    }

    /// Handles a single command.
    fn handle_command(&mut self, cmd: MaintenanceCommand) -> ControlFlow {
        match cmd {
            MaintenanceCommand::Shutdown => {
                debug!("Received Shutdown command");
                ControlFlow::Shutdown
            }

            MaintenanceCommand::PrepareCheckpoint(ack) => {
                debug!("Preparing checkpoint");
                self.prepare_checkpoint();
                let _ = ack.send(());
                ControlFlow::Continue
            }

            MaintenanceCommand::CreateDimension {
                config,
                dimension_id,
                depth_cutoff,
                spatial_decay_rate,
                observation_rx,
                infra,
                ack,
            } => {
                debug!(?dimension_id, "Creating dimension");
                let state = DimensionState::new(
                    dimension_id,
                    config,
                    depth_cutoff,
                    spatial_decay_rate,
                    observation_rx,
                    infra,
                    self.clock.now_monotonic(),
                );
                self.dimensions.push(state);
                let _ = ack.send(());
                ControlFlow::Continue
            }

            MaintenanceCommand::DestroyDimension { dimension_id } => {
                debug!(?dimension_id, "Destroying dimension");
                self.dimensions.retain(|d| d.id != dimension_id);
                ControlFlow::Continue
            }

            #[cfg(test)]
            MaintenanceCommand::ForceDecay {
                dimension_id,
                attenuation,
                ack,
            } => {
                debug!(?dimension_id, attenuation, "Forced decay (test)");
                if let Some(dim) = self.dimensions.iter_mut().find(|d| d.id == dimension_id) {
                    let root = dim.graph.g_root();
                    // TODO ´todo:code:reconcile-this-test-helper-s-q´: reconcile this test helper's q value
                    // with the production uniform-decay contract (`q = 1.0`),
                    // which is the spatial decay the layer declares
                    // (´tab:keyspace:decay-rates´).
                    // Uniform decay (q=0): all nodes scaled by the same
                    // factor.  This matches the production `apply_decay`
                    // path and avoids selective-decay edge cases where
                    // the root survives annihilation (0^0 = 1).
                    dim.graph.decay(root, attenuation, 0.0);
                }
                // Decay is a pure value operation — it does not cause
                // immediate competitive set changes.  Changes emerge on
                // the next observation cycle when fresh evidence interacts
                // with the decayed landscape, and are published on that
                // cycle (´dec:memory:competitive-publication´).
                let _ = ack.send(());
                ControlFlow::Continue
            }
        }
    }

    /// Drains observation queues for all dimensions.
    fn drain_observations(&mut self) {
        for dim in &mut self.dimensions {
            let mut count = 0;
            while let Ok(coord) = dim.observation_rx.try_recv() {
                // Feed-forward enforcement: Δ=1 is hardcoded here
                dim.graph.observe(coord, 1u64);
                count += 1;
            }
            if count > 0 {
                if let Ok(mut cell_state) = dim.infra.cell_state.write() {
                    retain_existing_graph_intervals(&mut cell_state, &dim.graph);
                }
                trace!(dimension = ?dim.id, count, "Drained observations");
            }
        }
    }

    /// Detects competitive set changes and emits lifecycle events.
    fn detect_and_emit_changes(&mut self) {
        for dim in &mut self.dimensions {
            let new_set = dim.compute_competitive_set();
            let set_changed = new_set != dim.current_set;

            if set_changed {
                // Compute exits and entries
                let exits: Vec<_> = dim.current_set.difference(&new_set).copied().collect();
                let entries: Vec<_> = new_set.difference(&dim.current_set).copied().collect();

                debug!(
                    dimension = ?dim.id,
                    exits = exits.len(),
                    entries = entries.len(),
                    "Competitive set changed"
                );

                if let Ok(mut cell_state) = dim.infra.cell_state.write() {
                    apply_competitive_cell_state_changes(&mut cell_state, &exits, &entries);
                }

                // Emit lifecycle events: exits first, then entries
                // (marginalisation before extension)
                let mut events = Vec::with_capacity(exits.len() + entries.len());

                for cell in exits {
                    events.push(LifecycleEvent::CompetitiveCellExit { dimension: dim.id, cell });
                }

                for cell in entries {
                    events.push(LifecycleEvent::CompetitiveCellEntry { dimension: dim.id, cell });
                }

                // Send lifecycle submission to model owner
                let submission = LifecycleSubmission {
                    events,
                    completion: None,
                };

                if let Err(e) = self.model_owner_tx.try_send(ModelOwnerCommand::Lifecycle(submission)) {
                    warn!("Failed to send lifecycle events: {e}");
                }

                dim.current_set = new_set;
            }

            #[allow(clippy::cast_precision_loss)]
            let total_importance = dim.graph.total_sum() as f64;
            if set_changed || dim.infra.load_competitive_set().total_importance().to_bits() != total_importance.to_bits() {
                dim.publish_competitive_index();
            }
        }
    }

    /// Applies periodic decay to graphs if due.
    fn apply_decay(&mut self) {
        let now = self.clock.now_monotonic();
        for dim in &mut self.dimensions {
            if dim.decay_due(now) {
                let root = dim.graph.g_root();
                dim.graph.decay(root, dim.attenuation, 1.0);
                dim.last_decay = now;
                dim.publish_competitive_index();
                trace!(dimension = ?dim.id, "Applied decay");
            }
        }
    }

    /// Prepares for checkpoint by draining queues, detecting changes,
    /// and publishing snapshots.
    ///
    /// Change detection runs after draining so that observations
    /// processed during the drain are reflected in the competitive
    /// set and any resulting lifecycle events are emitted before
    /// the checkpoint ack.
    fn prepare_checkpoint(&mut self) {
        // Drain all observation queues
        self.drain_observations();

        // Apply the signal cache's deferred writes, so a checkpoint
        // barrier is also a cache synchronisation point.
        self.signal_cache.drain_deferred_writes();

        // Detect competitive set changes from the drained observations
        self.detect_and_emit_changes();

        // Publish graph snapshots
        for dim in &self.dimensions {
            let snapshot = dim.extract_snapshot();
            dim.infra.graph_snapshot.store(Arc::new(Some(snapshot)));
        }
    }
}

/// Control flow result from command handling.
enum ControlFlow {
    /// Continue the loop.
    Continue,
    /// Shut down the loop.
    Shutdown,
}

// ═══════════════════════════════════════════════════════════════════════════════
// Thread Spawning
// ═══════════════════════════════════════════════════════════════════════════════

/// Spawns the identity maintenance thread.
///
/// Returns the thread handle and command sender.
pub fn spawn_identity_maintenance_thread(
    instance_id: &str,
    model_owner_tx: Sender<ModelOwnerCommand>,
    clock: Arc<dyn crate::testing::Clock>,
    signal_cache: Arc<crate::signal::SignalCache>,
) -> (std::thread::JoinHandle<()>, Sender<MaintenanceCommand>) {
    let (command_tx, command_rx) = crossbeam_channel::bounded(64);

    let thread_name = format!("{instance_id}-identity-maintenance");
    let maintenance_loop = IdentityMaintenanceLoop::new(command_rx, model_owner_tx, clock, signal_cache);

    let handle = std::thread::Builder::new()
        .name(thread_name.clone())
        .spawn(move || maintenance_loop.run())
        .unwrap_or_else(|e| panic!("failed to spawn identity maintenance thread {thread_name:?}: {e}"));

    (handle, command_tx)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    fn test_graph_config() -> MudlarkConfig<u64> {
        MudlarkConfig {
            split_threshold: 10,
            depth_create: 8,
            depth_evict: 10,
            budget: Some(1_000),
            alpha_relax: 0.75,
            bounded_eviction: true,
        }
    }

    fn test_infra() -> (Arc<IdentityDimensionInfra>, Receiver<u128>) {
        let (observation_tx, observation_rx) = crossbeam_channel::bounded(16);
        let dimension =
            super::super::IdentityDimension::new(DimensionId(1), "test", "test hierarchy", "prefix groups", 128, 8, |_| 0);
        (
            Arc::new(IdentityDimensionInfra::new(dimension, observation_tx)),
            observation_rx,
        )
    }

    /// The maintenance thread comes up on demand and stands down when told to,
    /// leaving its join to succeed rather than panicking or hanging. The loop
    /// otherwise sits in a timed wait forever, so an engine that could not
    /// retire it would leak a thread per instance and never shut down cleanly.
    ///
    /// ´claim:identity:the-maintenance-thread-obeys-a-shutdown-command-and-joins-cleanly´
    /// ´test:unit:maintenance-thread-starts-and-shuts-down´
    #[test]
    fn maintenance_thread_starts_and_shuts_down() {
        let (model_owner_tx, _model_owner_rx) = crossbeam_channel::bounded::<ModelOwnerCommand>(64);
        let (handle, command_tx) = spawn_identity_maintenance_thread(
            "test",
            model_owner_tx,
            Arc::new(crate::testing::SystemClock),
            Arc::new(crate::signal::SignalCache::new(
                1,
                Arc::new(crate::signal::SignalSchemaIndex::from_declarations(&[]).expect("empty schema")),
            )),
        );

        // Send shutdown
        command_tx.send(MaintenanceCommand::Shutdown).unwrap();

        // Thread should exit within 200ms
        let result = handle.join();
        assert!(result.is_ok());
    }

    /// Sixty applications of the per-minute attenuation compose to exactly the
    /// tabulated hourly spatial decay rate, and that rate halves importance on
    /// the fourteen-day timescale the corpus describes — between thirteen and
    /// sixteen days — rather than within a day. The cadence is thereby an
    /// implementation detail: however often the loop wakes, an hour of decay is
    /// one application of the hourly rate, so who stays competitive is governed
    /// by the table and not by the loop's schedule.
    ///
    /// ´claim:identity:the-per-interval-attenuation-composes-to-the-tabulated-hourly-rate-and-a-fortnight-half-life´
    /// ´test:unit:decay-cadence-composes-to-hourly-rate´
    #[test]
    fn decay_cadence_composes_to_hourly_rate() {
        let per_minute = per_interval_attenuation(DEFAULT_HOURLY_ATTENUATION, DEFAULT_DECAY_INTERVAL);

        // Sixty per-minute applications = one hourly application.
        let hourly = per_minute.powi(60);
        assert!(
            (hourly - DEFAULT_HOURLY_ATTENUATION).abs() < 1e-12,
            "composed hourly factor {hourly} != tabulated {DEFAULT_HOURLY_ATTENUATION}"
        );

        // Half-life in hours at the hourly rate: log base rate of one half.
        let half_life_hours = 0.5_f64.log(DEFAULT_HOURLY_ATTENUATION);
        let half_life_days = half_life_hours / 24.0;
        assert!(
            (13.0..16.0).contains(&half_life_days),
            "half-life {half_life_days} days is not the corpus's fortnight"
        );
    }

    /// The dimension state derives its interval attenuation from the
    /// host-supplied hourly spatial decay rate rather than the shipped default,
    /// and one hour of intervals composes back to that supplied rate.
    ///
    /// ´claim:identity:the-graph-uses-the-spatial-decay-rate-supplied-at-registration´
    /// ´test:unit:dimension-state-uses-registered-spatial-decay-rate´
    #[test]
    fn dimension_state_uses_registered_spatial_decay_rate() {
        let (infra, observation_rx) = test_infra();
        let supplied_rate = 0.81;
        let state = DimensionState::new(
            DimensionId(1),
            test_graph_config(),
            8,
            supplied_rate,
            observation_rx,
            infra,
            Instant::now(),
        );

        assert!((state.attenuation.powi(60) - supplied_rate).abs() < 1e-12);
    }

    /// Competitive exit zeros measurement state while preserving outcome
    /// state, and re-entry reuses that outcome unchanged instead of replacing
    /// the cell with a neutral state.
    ///
    /// ´claim:identity:competitive-exit-resets-measurement-while-outcome-warm-starts-reentry´
    /// ´test:unit:competitive-exit-resets-measurement-and-reentry-retains-outcome´
    #[test]
    fn competitive_exit_resets_measurement_and_reentry_retains_outcome() {
        let cell = CompetitiveCellId::new(0, 8);
        let mut state = CellMutableState::new();
        state.measurement.step_count = 9;
        state.measurement.volatility = 0.25;
        state.outcome.adverse_rate_ewma = 0.75;
        let outcome_before = state.outcome.clone();
        let mut states = HashMap::from([(cell, Mutex::new(state))]);

        apply_competitive_cell_state_changes(&mut states, &[cell], &[]);
        {
            let state = states[&cell].lock().expect("cell state");
            assert_eq!(state.measurement.step_count, 0);
            assert!((state.measurement.volatility - 0.0).abs() < f64::EPSILON);
            assert_eq!(state.outcome, outcome_before);
        }

        apply_competitive_cell_state_changes(&mut states, &[], &[cell]);
        let state = states[&cell].lock().expect("cell state after re-entry");
        assert_eq!(state.outcome, outcome_before);
    }

    /// A budgeted graph is driven until it evicts an interval that existed in
    /// the preceding full census. Cleanup removes retained state for that
    /// evicted interval while preserving the root interval that remains live.
    /// The removal is therefore tied to real graph eviction rather than merely
    /// to competitive standing or a fabricated absent key.
    ///
    /// ´claim:identity:retained-outcome-state-is-cleaned-only-when-its-graph-interval-no-longer-exists´
    /// ´test:unit:graph-interval-cleanup-removes-only-evicted-state´
    #[test]
    fn graph_interval_cleanup_removes_only_evicted_state() {
        let mut graph = GvGraph::<u128, u64, 128>::new(MudlarkConfig {
            split_threshold: 1,
            depth_create: 3,
            depth_evict: 4,
            budget: Some(20),
            alpha_relax: 0.75,
            bounded_eviction: true,
        });
        let root = CompetitiveCellId::new(0, 0);
        let mut previous = graph_interval_ids(&graph);
        let mut evicted = None;

        for step in 0_u128..4_096 {
            let coord = (step % 64) << 122;
            graph.observe(coord, 2_u64);
            let current = graph_interval_ids(&graph);
            if let Some(cell) = previous.difference(&current).next().copied() {
                evicted = Some(cell);
                break;
            }
            previous = current;
        }

        let evicted = evicted.expect("budget pressure should evict a previously live graph interval");
        assert!(!graph_interval_ids(&graph).contains(&evicted));
        assert!(graph_interval_ids(&graph).contains(&root));
        let mut states = HashMap::from([
            (root, Mutex::new(CellMutableState::new())),
            (evicted, Mutex::new(CellMutableState::new())),
        ]);

        retain_existing_graph_intervals(&mut states, &graph);

        assert!(states.contains_key(&root));
        assert!(!states.contains_key(&evicted));
    }
}
