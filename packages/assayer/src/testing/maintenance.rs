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

use std::sync::Arc;
use std::thread::JoinHandle;
use std::time::Duration;

use crossbeam_channel::{self, Receiver, Sender};
use torrust_mudlark::Config as MudlarkConfig;

use super::liveness::ACK_DEADLINE;
use crate::identity::{
    IdentityDimension, IdentityDimensionInfra, MaintenanceCommand, ObservationChannel, spawn_identity_maintenance_thread,
};
use crate::owner::commands::{LifecycleEvent, ModelOwnerCommand};
use crate::types::DimensionId;

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

/// Creates a test dimension + shared infra + observation channel.
///
/// The dimension has a trivial encoder (`|_| 0`) and a 10 000-slot
/// observation channel. The returned `ObservationChannel` can be
/// `.into_receiver()`'d for the maintenance loop while `infra` is
/// cloned for readers.
#[must_use]
pub fn setup_dimension(dim_id: u32) -> (Arc<IdentityDimensionInfra>, ObservationChannel) {
    let dim = IdentityDimension::new(
        DimensionId(dim_id),
        format!("test-{dim_id}"),
        "test hierarchy",
        "prefix groups",
        128,
        24,
        |_| 0,
    );
    let obs = ObservationChannel::with_capacity(10_000);
    let infra = Arc::new(IdentityDimensionInfra::new(dim, obs.sender()));
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

    /// Spawns a maintenance thread with the default model-owner channel
    /// capacity ([`Self::DEFAULT_MODEL_OWNER_CAPACITY`]).
    #[must_use]
    pub fn spawn(instance_id: &str) -> Self {
        Self::spawn_with_capacity(instance_id, Self::DEFAULT_MODEL_OWNER_CAPACITY)
    }

    /// Spawns a maintenance thread with an explicit model-owner channel
    /// capacity (useful for back-pressure tests).
    #[must_use]
    pub fn spawn_with_capacity(instance_id: &str, model_owner_capacity: usize) -> Self {
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
        Self {
            handle: Some(handle),
            command_tx,
            model_owner_rx,
        }
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

    /// Registers a dimension with the maintenance loop using
    /// [`default_mudlark_config`] and waits for the ack.
    pub fn register_dimension(
        &self,
        dimension_id: DimensionId,
        depth_cutoff: u8,
        observation_rx: Receiver<u128>,
        infra: Arc<IdentityDimensionInfra>,
    ) {
        self.register_dimension_with_config(default_mudlark_config(), dimension_id, depth_cutoff, observation_rx, infra);
    }

    /// Registers a dimension with a caller-supplied Mudlark config and
    /// waits for the ack.
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
                ack: ack_tx,
            })
            .expect("send CreateDimension");
        ack_rx.recv_timeout(ACK_DEADLINE).expect("dimension creation ack");
    }

    /// Sends `DestroyDimension` without awaiting any ack (the command
    /// itself has no ack channel).
    pub fn destroy_dimension(&self, dimension_id: DimensionId) {
        self.command_tx
            .send(MaintenanceCommand::DestroyDimension { dimension_id })
            .expect("send DestroyDimension");
    }

    /// Sends `PrepareCheckpoint` and waits for the ack.
    ///
    /// Returns once the loop has drained pending observations and
    /// published a fresh graph snapshot.
    pub fn checkpoint(&self) {
        let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
        self.command_tx
            .send(MaintenanceCommand::PrepareCheckpoint(ack_tx))
            .expect("send PrepareCheckpoint");
        ack_rx.recv_timeout(ACK_DEADLINE).expect("checkpoint ack");
    }

    /// Forces the maintenance loop to drain all pending observations
    /// and complete at least one full cycle (incl.
    /// `detect_and_emit_changes`).
    ///
    /// 1. [`Self::checkpoint`] drains observation queues and acks.
    /// 2. Sleeps 200 ms (~2× the default cycle interval) so the loop
    ///    re-enters `detect_and_emit_changes` after the checkpoint.
    ///
    /// Use this in place of a bare `thread::sleep` whenever a test
    /// observes competitive-set or lifecycle-event side effects.
    pub fn flush_loop(&self) {
        self.checkpoint();
        std::thread::sleep(Duration::from_millis(200));
    }

    /// Forces decay on a dimension and waits for the ack.
    ///
    /// Decay is a pure value operation — call [`Self::flush_loop`]
    /// afterwards to trigger change detection on the next cycle.
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

    /// Drains every currently-pending `ModelOwnerCommand` and returns
    /// the flattened lifecycle events owned by those commands.
    #[must_use]
    pub fn drain_lifecycle_events(&self) -> Vec<LifecycleEvent> {
        self.drain_model_owner()
            .into_iter()
            .flat_map(|cmd| match cmd {
                ModelOwnerCommand::Lifecycle(sub) => sub.events,
                _ => Vec::new(),
            })
            .collect()
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
