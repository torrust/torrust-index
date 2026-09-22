// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Model-owner thread: single-writer event loop.
//!
//! The model owner is the sole writer of all mutable model state.
//! It consumes two bounded crossbeam channels:
//!
//! 1. **Command channel** (capacity 64) — lifecycle, checkpoint, shutdown.
//! 2. **Observation channel** (capacity `N_init`) — raw vectors offered to
//!    the cold prior-mass ramp (´alg:standardisation:batch-initialisation´).
//! 3. **Label channel** (configurable capacity) — sequenced labels.
//!
//! The main loop follows the drain-before-process protocol
//! (´dec:concurrency:channel-preemption´): all pending commands are
//! drained before a single label is processed. This guarantees lifecycle
//! events (dimension changes) are applied before any label that might
//! depend on them.
//!
//! # What a Command Is Ordered Behind
//!
//! Because commands preempt, a command is ordered behind the commands sent
//! before it and behind nothing else. The loop drains commands first, then
//! observations, then takes one label, so a command entering the queue while
//! observations are waiting is handled with those observations still queued.
//! Every synchronisation barrier built on a command therefore has to name the
//! queue it drains itself: `Checkpoint` drains the label channel, and
//! `ObservationBarrier` drains the observation channel. Neither drains the
//! other's, and neither could be made to by the loop's ordering alone — the
//! command's position in its own queue carries no information about a
//! different queue's contents.
//!
//! # Panic Handling
//!
//! If the model-owner thread panics, both channel receivers disconnect.
//! Subsequent `try_send` calls on the sender side return
//! `Err(Disconnected)`, which the API layer maps to
//! `LabelError::ModelOwnerShutdown`: the loss reaches the submitter
//! rather than being absorbed here (´dec:concurrency:no-silent-drop´).
//!
//! # Cross-References
//!
//! - (´dec:concurrency:single-steward´) — the working copy this loop is sole writer of
//! - (´dec:concurrency:channel-preemption´) — commands drain before the next label
//! - (´dec:concurrency:named-thread´) — the thread this state is moved into
//! - (´dec:concurrency:no-silent-drop´) — what a disconnected channel owes the submitter

use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::AtomicU64;

use crossbeam_channel::{Receiver, Select, Sender};
use tracing::{debug, info, warn};

use super::commands::{CheckpointRequest, LifecycleSubmission, ModelOwnerCommand, ObservationBarrierRequest, SequencedLabel};
use crate::config::types::{AssayerConfig, PersistenceConfig};
use crate::feature::standardisation::FeatureClass;
use crate::health::{
    ConcordanceTracker, DriftResetCounter, HealthEvent, IdentityConvergenceTracker, LifecycleHealthEvent,
    PlattConvergenceTracker, emit_health_event,
};
use crate::identity::IdentityDimensionInfra;
use crate::ledger::OutcomeLedger;
use crate::owner::label_path::{CalibrationBuffer, HealthCounters, LabelPipelineConfig};
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::WorkingCopy;
use crate::types::DimensionId;

// ═══════════════════════════════════════════════════════════════════════════════
// ModelOwner
// ═══════════════════════════════════════════════════════════════════════════════

/// The model-owner thread state.
///
/// Constructed by `Assayer::build()` and moved into a dedicated
/// `std::thread`. Owns the `WorkingCopy` and publishes snapshots
/// to `SharedState` via `ArcSwap`.
pub struct ModelOwner {
    /// The mutable working copy of all model state.
    working: WorkingCopy,
    /// Shared state for lock-free snapshot publication.
    shared: Arc<SharedState>,
    /// Receiver for commands (lifecycle, checkpoint, shutdown).
    command_rx: Receiver<ModelOwnerCommand>,
    /// Cold-ramp observations, on their own channel so the ramp's own
    /// traffic cannot crowd out control commands
    /// (´alg:standardisation:batch-initialisation´).
    observation_rx: Receiver<crate::owner::commands::ColdRampObservation>,
    /// Receiver for sequenced labels.
    label_rx: Receiver<SequencedLabel>,
    /// Current snapshot version counter.
    version: u64,
    /// Persistence configuration (´dec:durability:checkpoint-journal´).
    /// `None` if persistence is disabled.
    persistence: Option<PersistenceConfig>,
    /// Outcome ledger for Sentinel updates.
    outcome_ledger: Arc<OutcomeLedger>,
    /// Concordance tracker shared with the assessment path for checkpointing.
    concordance: Arc<ConcordanceTracker>,
    /// Platt calibration convergence tracker.
    platt_tracker: PlattConvergenceTracker,
    /// Buffer of calibration entries, circular (´constr:platt:buffer´).
    calibration_buffer: CalibrationBuffer,
    /// Per-dimension identity convergence trackers.
    identity_trackers: HashMap<DimensionId, IdentityConvergenceTracker>,
    /// Assayer configuration (for lifecycle extension parameters).
    config: Arc<AssayerConfig>,
    /// The engine's source of time, shared with the public surface.
    clock: Arc<dyn crate::testing::Clock>,
    /// Label pipeline configuration.
    label_config: LabelPipelineConfig,
    /// Channel for health events.
    event_tx: Sender<HealthEvent>,
    /// Counter for dropped health events.
    dropped_counter: Arc<AtomicU64>,
    /// Counter for emitted drift-reset events.
    drift_reset_counter: Arc<DriftResetCounter>,
    /// Shared identity dimension infrastructure (step 13).
    identity_dimensions: Arc<std::sync::RwLock<HashMap<DimensionId, Arc<IdentityDimensionInfra>>>>,
    /// Monotonic total-label counter for Platt refit triggers.
    /// TODO ´todo:code:add-restore-a-separate-checkpointed´: add/restore a separate checkpointed
    /// `WorkingCopy::total_eligible_labels` counter for composite
    /// convergence. Do not use this total-label counter for stage
    /// boundaries 1–3.
    label_count: u64,
    /// Health counters for the published health summary
    /// (´dec:health:independent-publication´).
    health_counters: HealthCounters,
    /// Structural metadata captured in every checkpoint for mismatch
    /// detection on restore (´dec:durability:structural-compatibility´).
    #[cfg(feature = "serde")]
    structural_metadata: crate::persistence::checkpoint::StructuralMetadata,
    /// Identity maintenance command sender, for the checkpoint's
    /// prepare-and-publish coordination. `None` in tests that construct
    /// the owner without the maintenance thread.
    identity_maintenance_tx: Option<Sender<crate::identity::MaintenanceCommand>>,
    /// The journal writer shared with the `label()` append path. The
    /// checkpoint's truncation runs under this mutex so it serialises
    /// with concurrent appends.
    #[cfg(feature = "serde")]
    journal: Option<Arc<std::sync::Mutex<crate::persistence::journal::JournalWriter>>>,
}

impl ModelOwner {
    /// Creates a new `ModelOwner`.
    #[must_use]
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        working: WorkingCopy,
        shared: Arc<SharedState>,
        command_rx: Receiver<ModelOwnerCommand>,
        observation_rx: Receiver<crate::owner::commands::ColdRampObservation>,
        label_rx: Receiver<SequencedLabel>,
        outcome_ledger: Arc<OutcomeLedger>,
        concordance: Arc<ConcordanceTracker>,
        event_tx: Sender<HealthEvent>,
        dropped_counter: Arc<AtomicU64>,
        drift_reset_counter: Arc<DriftResetCounter>,
        identity_dimensions: Arc<std::sync::RwLock<HashMap<DimensionId, Arc<IdentityDimensionInfra>>>>,
        config: Arc<AssayerConfig>,
        clock: Arc<dyn crate::testing::Clock>,
    ) -> Self {
        let label_config = LabelPipelineConfig::from_assayer_config(&config);

        // Initialise the Platt tracker with κ from config
        // (´def:platt:initial-value´).
        let platt_tracker = PlattConvergenceTracker::from_platt_config(&config.platt);

        // Calibration buffer at configured capacity (´constr:platt:buffer´).
        let calibration_buffer = CalibrationBuffer::new(config.platt.n_cal_buf);

        Self {
            working,
            shared,
            command_rx,
            observation_rx,
            label_rx,
            version: 1,
            persistence: None,
            outcome_ledger,
            concordance,
            platt_tracker,
            calibration_buffer,
            identity_trackers: HashMap::new(),
            config,
            clock,
            label_config,
            event_tx,
            dropped_counter,
            drift_reset_counter,
            identity_dimensions,
            label_count: 0,
            health_counters: HealthCounters::default(),
            #[cfg(feature = "serde")]
            structural_metadata: crate::persistence::checkpoint::StructuralMetadata::default(),
            identity_maintenance_tx: None,
            #[cfg(feature = "serde")]
            journal: None,
        }
    }

    /// Shares the journal writer with the owner so the checkpoint's
    /// sequence-aware truncation runs under the append path's mutex.
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn with_journal(mut self, journal: Option<Arc<std::sync::Mutex<crate::persistence::journal::JournalWriter>>>) -> Self {
        self.journal = journal;
        self
    }

    /// Sets the persistence configuration.
    #[must_use]
    pub fn with_persistence(mut self, persistence: Option<PersistenceConfig>) -> Self {
        self.persistence = persistence;
        self
    }

    /// Sets the identity maintenance sender used by the checkpoint's
    /// prepare-and-publish coordination.
    #[must_use]
    pub fn with_identity_maintenance(mut self, tx: Sender<crate::identity::MaintenanceCommand>) -> Self {
        self.identity_maintenance_tx = Some(tx);
        self
    }

    /// Restores the owner-held state a checkpoint captured beside the
    /// working copy (´dec:durability:checkpoint-journal´).
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn with_restored_owner_state(mut self, state: crate::persistence::checkpoint::OwnerCheckpointState) -> Self {
        self.calibration_buffer = state.calibration_buffer;
        self.platt_tracker = state.platt_tracker;
        self.identity_trackers = state.identity_trackers.into_iter().collect();
        self.label_count = state.label_count;
        self.health_counters = state.health_counters;
        self
    }

    /// Sets the structural metadata captured in every checkpoint for
    /// mismatch detection on restore (´dec:durability:structural-compatibility´).
    #[cfg(feature = "serde")]
    #[must_use]
    pub fn with_structural_metadata(mut self, metadata: crate::persistence::checkpoint::StructuralMetadata) -> Self {
        self.structural_metadata = metadata;
        self
    }

    // ─────────────────────────────────────────────────────────────────────
    // Main loop
    // ─────────────────────────────────────────────────────────────────────

    /// Runs the model-owner event loop until `Shutdown` is received
    /// or both channels disconnect.
    ///
    /// This is the entry point called from `std::thread::spawn`.
    pub fn run(mut self) {
        info!("model-owner thread started");

        loop {
            // Step 1: Drain ALL pending commands, structure first
            // (´dec:concurrency:channel-preemption´).
            loop {
                match self.command_rx.try_recv() {
                    Ok(cmd) => {
                        if self.handle_command(cmd) {
                            info!("model-owner thread shutting down");
                            return;
                        }
                    }
                    Err(crossbeam_channel::TryRecvError::Empty) => break,
                    Err(crossbeam_channel::TryRecvError::Disconnected) => {
                        warn!("command channel disconnected; model-owner shutting down");
                        return;
                    }
                }
            }

            // Step 1b: Drain pending cold-ramp observations, after the
            // commands and before the labels. Observations are measurement
            // rather than control, so they never delay a registration or a
            // shutdown; they precede labels because the ramp owns the
            // coordinate system a label will be standardised against
            // (´alg:standardisation:batch-initialisation´).
            while let Ok(observation) = self.observation_rx.try_recv() {
                self.apply_cold_ramp_observation(&observation.phi_raw, observation.layout_generation);
            }

            // Step 2: Wait for next event (command, observation or label).
            // Biased select preserves command-before-label ordering when
            // several receivers are ready at the wait point.
            let mut sel = Select::new_biased();
            let cmd_idx = sel.recv(&self.command_rx);
            let observation_idx = sel.recv(&self.observation_rx);
            let label_idx = sel.recv(&self.label_rx);

            let oper = sel.select();
            match oper.index() {
                i if i == cmd_idx => {
                    if let Ok(cmd) = oper.recv(&self.command_rx) {
                        if self.handle_command(cmd) {
                            info!("model-owner thread shutting down");
                            return;
                        }
                    } else {
                        warn!("command channel disconnected; model-owner shutting down");
                        return;
                    }
                }
                i if i == observation_idx => {
                    // A disconnected observation channel means the engine is
                    // gone; commands and labels decide shutdown, not this.
                    if let Ok(observation) = oper.recv(&self.observation_rx) {
                        self.apply_cold_ramp_observation(&observation.phi_raw, observation.layout_generation);
                    }
                }
                i if i == label_idx => {
                    if let Ok(label) = oper.recv(&self.label_rx) {
                        self.handle_label(&label);
                    } else {
                        debug!("label channel disconnected");
                        // Label channel disconnected but commands may still arrive.
                        // Continue draining commands until shutdown or command disconnect.
                        self.drain_commands_only();
                        return;
                    }
                }
                _ => unreachable!("unexpected select index"),
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Command handling
    // ─────────────────────────────────────────────────────────────────────

    /// Handles a single command. Returns `true` if the loop should exit.
    fn handle_command(&mut self, cmd: ModelOwnerCommand) -> bool {
        match cmd {
            ModelOwnerCommand::Lifecycle(submission) => {
                self.handle_lifecycle(submission);
                false
            }
            ModelOwnerCommand::Checkpoint(request) => {
                self.handle_checkpoint(request);
                false
            }
            ModelOwnerCommand::ObservationBarrier(request) => {
                self.handle_observation_barrier(&request);
                false
            }
            ModelOwnerCommand::Shutdown => true,
            ModelOwnerCommand::ResetDriftAccumulators => {
                // Host-initiated trigger (´tab:monitoring:drift-resets´):
                // the accumulators reset, the smoothed diagnostics
                // survive, exactly as the automatic trigger spares them.
                for drift in self.working.drift_state.values_mut() {
                    drift.reset_cusums();
                }
                info!("host-initiated drift accumulator reset applied to all models");
                false
            }
            ModelOwnerCommand::BootstrapComplete { sentinel_id, stats } => {
                self.apply_sentinel_bootstrap(sentinel_id, &stats);
                false
            }
            ModelOwnerCommand::TestBlock { entered, release } => {
                let _entered_result = entered.send(());
                let _release_result = release.recv();
                false
            }
        }
    }

    /// Handles a lifecycle submission.
    ///
    /// Layer 1 (structural): log events, send success results on
    /// completion channel. No actual model changes.
    fn handle_lifecycle(&mut self, submission: LifecycleSubmission) {
        // The version the submission's first chain publishes at. A submission
        // whose events split into several order-free chains publishes one
        // snapshot per chain and leaves the counter at the last version it
        // published (´dec:construction:compound-batch´).
        self.version += 1;

        let result = crate::owner::lifecycle::handle_lifecycle_submission(
            &mut self.working,
            submission,
            &self.shared,
            &mut self.platt_tracker,
            &mut self.identity_trackers,
            &self.outcome_ledger,
            &self.config,
            &mut self.version,
            self.clock.now_monotonic(),
            &self.clock.now(),
        );
        if let Ok(results) = result {
            // One marginalisation happened per chain, not per event, so one
            // event is emitted per chain. Every result in a chain carries the
            // same aggregate; emitting it once per result would count one Schur
            // computation as many in the lifetime ledger
            // (´entry:health:marginalise-event´).
            let mut emitted_for: Option<usize> = None;
            for event_result in results {
                if emitted_for == Some(event_result.chain_index) {
                    continue;
                }
                if let Some(diagnostics) = event_result.marginalisation {
                    emitted_for = Some(event_result.chain_index);
                    emit_health_event(
                        HealthEvent::Lifecycle(LifecycleHealthEvent::MarginalisationCompleted { diagnostics }),
                        &self.event_tx,
                        &self.dropped_counter,
                    );
                }
            }
        }
    }

    /// Handles a checkpoint request.
    ///
    /// Drains all pending labels first so the checkpoint captures a
    /// consistent state. This is the contract relied on by
    /// `flush_label_channel` (´alg:host:pre-seeding´).
    ///
    /// The cold-ramp observation queue is *not* drained here, and the
    /// checkpoint is the poorer for it only in the sense that a snapshot is
    /// entitled to be: an observation still in flight has advanced nothing,
    /// so there is nothing of it to capture. What it does mean is that an
    /// acknowledgement from this handler says nothing about the ramp, and a
    /// caller that needs the ramp's arrivals applied asks
    /// [`handle_observation_barrier`](Self::handle_observation_barrier) for
    /// that instead.
    fn handle_checkpoint(&mut self, request: CheckpointRequest) {
        // Drain all pending labels before checkpointing so the snapshot
        // is consistent and `flush_label_channel` can rely on
        // `health_summary()` reflecting every label sent before the
        // checkpoint command.
        while let Ok(label) = self.label_rx.try_recv() {
            self.handle_label(&label);
        }

        let result = self.do_checkpoint();
        if let Some(tx) = request.completion {
            drop(tx.send(result));
        }
    }

    /// Handles an observation barrier.
    ///
    /// Drains the cold-ramp observation queue and applies everything taken
    /// from it, then acknowledges. Because the loop drains the whole command
    /// queue before it touches an observation, this command is reached with
    /// the observations that were queued before it still queued, and draining
    /// here is what puts them behind the acknowledgement.
    ///
    /// Draining to empty rather than counting to a mark is deliberate: the
    /// barrier's promise is that nothing queued before it is outstanding, and
    /// an observation that arrived during the drain is applied early rather
    /// than left waiting, which no caller can be worse off for. The drain
    /// applies each arrival through the same steward path an idle loop would
    /// (´alg:standardisation:batch-initialisation´), so a ramp advanced by a
    /// barrier is in the state it would have reached on its own — the barrier
    /// moves when that happens, never what happens.
    fn handle_observation_barrier(&mut self, request: &ObservationBarrierRequest) {
        while let Ok(observation) = self.observation_rx.try_recv() {
            self.apply_cold_ramp_observation(&observation.phi_raw, observation.layout_generation);
        }
        let _ack_result = request.completion.send(());
    }

    /// Performs the actual checkpoint write + journal truncation.
    #[allow(clippy::unused_self, clippy::unnecessary_wraps)] // no-op without serde feature
    fn do_checkpoint(&self) -> Result<(), std::io::Error> {
        #[cfg(feature = "serde")]
        if let Some(ref persistence) = self.persistence {
            use crate::persistence::checkpoint::{
                IdentityDimensionPayload, OwnerCheckpointState, SentinelLedgerPayload, write_checkpoint,
            };

            let timestamp = self.clock.now();

            // Whole-state capture (´dec:durability:checkpoint-journal´).
            // Ask the identity maintenance loop to drain its queues and
            // publish fresh graph snapshots before they are read below;
            // on any failure the last published snapshots are captured.
            if let Some(ref tx) = self.identity_maintenance_tx {
                let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
                if tx
                    .try_send(crate::identity::MaintenanceCommand::PrepareCheckpoint(ack_tx))
                    .is_ok()
                {
                    if ack_rx.recv_timeout(std::time::Duration::from_secs(5)).is_err() {
                        warn!("identity maintenance did not acknowledge checkpoint preparation in time");
                    }
                } else {
                    warn!("identity maintenance unreachable for checkpoint preparation");
                }
            }

            let ledger_state: Vec<(crate::types::SentinelId, SentinelLedgerPayload)> = self
                .outcome_ledger
                .snapshot()
                .ledgers
                .into_iter()
                .map(|(id, ledger)| (id, SentinelLedgerPayload { ledger }))
                .collect();

            let identity_state: Vec<(DimensionId, IdentityDimensionPayload)> = {
                let guard = self
                    .identity_dimensions
                    .read()
                    .unwrap_or_else(std::sync::PoisonError::into_inner);
                guard
                    .iter()
                    .filter_map(|(&dim_id, infra)| {
                        let snap_guard = infra.graph_snapshot.load();
                        let snapshot = (**snap_guard).as_ref()?.clone();
                        let competitive_cells: Vec<_> = infra.load_competitive_set().cell_ids().copied().collect();
                        let cell_outcome_state = infra
                            .cell_state
                            .read()
                            .map(|cells| {
                                cells
                                    .iter()
                                    .map(|(cell_id, state)| {
                                        let outcome = state
                                            .lock()
                                            .unwrap_or_else(std::sync::PoisonError::into_inner)
                                            .outcome
                                            .clone();
                                        (*cell_id, outcome)
                                    })
                                    .collect()
                            })
                            .unwrap_or_default();
                        Some((
                            dim_id,
                            IdentityDimensionPayload {
                                graph_snapshot: snapshot,
                                competitive_cells,
                                cell_outcome_state,
                            },
                        ))
                    })
                    .collect()
            };

            let mut payload = self.working.to_checkpoint_payload_with_structural(
                timestamp,
                ledger_state,
                identity_state,
                self.structural_metadata.clone(),
            );
            payload.concordance_state = self.concordance.checkpoint_state();
            payload.owner_state = OwnerCheckpointState {
                calibration_buffer: self.calibration_buffer.clone(),
                platt_tracker: self.platt_tracker.clone(),
                identity_trackers: self.identity_trackers.iter().map(|(&id, t)| (id, t.clone())).collect(),
                label_count: self.label_count,
                health_counters: self.health_counters.clone(),
            };

            write_checkpoint(&persistence.checkpoint_path(), &payload)?;
            debug!("checkpoint written");

            // Truncate the journal to the checkpoint's mark, under the
            // same mutex the append path holds: entries the checkpoint
            // absorbed are discarded, a label journalled after the capture
            // is kept (´dec:durability:checkpoint-journal´).
            if let Some(ref journal) = self.journal {
                let mut writer = journal.lock().unwrap_or_else(std::sync::PoisonError::into_inner);
                if let Err(e) = writer.truncate_to(self.working.last_processed_label_seq) {
                    warn!(%e, "failed to truncate journal after checkpoint");
                }
            } else {
                warn!("checkpoint written without a journal handle; journal left untruncated");
            }

            return Ok(());
        }

        debug!("checkpoint request (no-op: persistence not configured)");
        Ok(())
    }

    // ─────────────────────────────────────────────────────────────────────
    // Label handling
    // ─────────────────────────────────────────────────────────────────────

    /// Handles a single sequenced label.
    ///
    /// Delegates to the 17-step pipeline in `label_path::process_label`,
    /// unless the label path has stopped, in which case the label is received
    /// and refused rather than applied.
    ///
    /// Refusing at the receiver rather than at the sender is what makes the
    /// stop complete. Labels the host submitted before the stop are already in
    /// the channel and would otherwise be applied to the working copy the
    /// engine has judged corrupt; draining them here leaves the models exactly
    /// as the stop found them, and leaves the channel free so the sender never
    /// blocks. Nothing else about the loop changes, which is what keeps the
    /// command channel — lifecycle, checkpoint, shutdown — working after the
    /// stop (´dec:concurrency:channel-preemption´).
    fn handle_label(&mut self, label: &SequencedLabel) {
        if let Some(stop) = self.shared.label_path_stop.load_full() {
            warn!(
                assessment_id = %label.label.assessment_id,
                seq = ?label.seq,
                cause = %stop,
                "the label path is stopped; the label was drained and refused, not applied",
            );
            return;
        }

        debug!(
            assessment_id = %label.label.assessment_id,
            seq = ?label.seq,
            "processing label"
        );

        self.version += 1;
        self.label_count += 1;

        crate::owner::label_path::process_label(
            &mut self.working,
            label,
            &self.shared,
            &self.outcome_ledger,
            &mut self.platt_tracker,
            &mut self.calibration_buffer,
            &self.identity_dimensions,
            &self.identity_trackers,
            self.concordance.current_thresholds(),
            &self.label_config,
            &*self.clock,
            &self.event_tx,
            &self.dropped_counter,
            &self.drift_reset_counter,
            self.version,
            self.label_count,
            &mut self.health_counters,
        );
    }

    // ─────────────────────────────────────────────────────────────────────
    // Fallback drain
    // ─────────────────────────────────────────────────────────────────────

    /// Drains remaining commands after the label channel disconnects.
    fn drain_commands_only(&mut self) {
        loop {
            match self.command_rx.recv() {
                Ok(cmd) => {
                    if self.handle_command(cmd) {
                        return;
                    }
                }
                Err(_) => return,
            }
        }
    }

    // ─────────────────────────────────────────────────────────────────────
    // Cold prior-mass ramp (´alg:standardisation:batch-initialisation´)
    // ─────────────────────────────────────────────────────────────────────

    /// Takes one accepted cold observation and publishes the ramp's advance.
    ///
    /// Steps 3 to 6 of (´alg:standardisation:batch-initialisation´). The
    /// steward is the single writer of the standardisation vectors while the
    /// ramp is running (´dec:concurrency:single-steward´), so the mixture is
    /// formed here and nowhere else, and every accepted advance publishes one
    /// immutable snapshot at a new version. A reader counting published
    /// coordinate states is therefore counting accepted observations rather
    /// than requests (´dec:ordering:score-before-evolve´).
    ///
    /// Two arrivals advance nothing. One assembled under a layout since
    /// renumbered describes positions this working copy no longer has at
    /// those indices, and one arriving after the horizon has nothing left to
    /// retire. Neither publishes, and neither increments the count.
    fn apply_cold_ramp_observation(&mut self, phi_raw: &[f64], layout_generation: u64) {
        let v_floor = self.config.standardisation.v_floor;
        let Some(reached_horizon) = self.working.accept_cold_observation(phi_raw, layout_generation, v_floor) else {
            return;
        };

        if reached_horizon {
            info!(
                accepted = self.working.cold_ramp.accepted(),
                horizon = self.working.cold_ramp.horizon(),
                "cold standardisation ramp reached its horizon — the published moments are now empirical"
            );

            // Step 6: the completion event, emitted once because the horizon
            // is reached once, on the bounded channel that counts what it
            // cannot deliver (´dec:health:bounded-events´).
            let event = HealthEvent::Convergence(crate::health::ConvergenceEvent::BatchInitComplete);
            if self.event_tx.try_send(event).is_err() {
                self.dropped_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            }
        }

        // Step 5: one accepted advance, one published snapshot, one version.
        self.version += 1;
        let snapshot = self.working.to_snapshot_at(self.version, self.clock.now_monotonic());
        self.shared.published.store(Arc::new(snapshot));
    }

    /// Blends a completed per-Sentinel bootstrap into that Sentinel's
    /// standardisation entries and publishes — steps 3 and 4 of
    /// (´alg:standardisation:sentinel-bootstrap´): weight `α_boot` on
    /// the empirical statistics, the remainder staying on what the
    /// entries hold.
    fn apply_sentinel_bootstrap(&mut self, sentinel_id: crate::types::SentinelId, stats: &crate::feature::bootstrap::InitStats) {
        let Some(range) = self.working.dimension_map.sentinel_slots.get(&sentinel_id).cloned() else {
            info!(?sentinel_id, "bootstrap completion for an unregistered sentinel; dropped");
            return;
        };

        // A slot width changed by a lifecycle event after completion
        // means these statistics describe a stale layout; step 2's
        // reset rule restarts a live accumulator, so a mismatched
        // completion is dropped rather than blended misaligned.
        if stats.mean.len() != range.end - range.start {
            info!(
                ?sentinel_id,
                stats_len = stats.mean.len(),
                slot_width = range.end - range.start,
                "bootstrap completion at a stale slot width; dropped"
            );
            return;
        }

        let alpha = self.config.standardisation.alpha_boot;
        for (k, j) in (range.start..range.end).enumerate() {
            if matches!(self.working.feature_classes[j], FeatureClass::Bias) {
                continue;
            }
            self.working.feature_means[j] =
                alpha.mul_add(stats.mean[k] - self.working.feature_means[j], self.working.feature_means[j]);
            let blended_variance = alpha.mul_add(
                stats.variance[k] - self.working.feature_variances[j],
                self.working.feature_variances[j],
            );
            self.working.feature_variances[j] = blended_variance.max(self.config.standardisation.v_floor);
        }

        info!(
            ?sentinel_id,
            count = stats.count,
            start = range.start,
            end = range.end,
            "sentinel bootstrap complete — slot statistics blended"
        );

        let event = HealthEvent::Convergence(crate::health::ConvergenceEvent::SentinelBootstrapComplete { sentinel_id });
        if self.event_tx.try_send(event).is_err() {
            self.dropped_counter.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
        }

        // Publish updated snapshot with the blended statistics.
        self.version += 1;
        let snapshot = self.working.to_snapshot_at(self.version, self.clock.now_monotonic());
        self.shared.published.store(Arc::new(snapshot));
    }
}
