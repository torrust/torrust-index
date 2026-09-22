// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Periodic checkpoint scheduler.
//!
//! A dedicated thread that periodically sends
//! `ModelOwnerCommand::Checkpoint` via the command channel.
//! The model-owner thread handles the actual checkpoint write.
//!
//! # Shutdown
//!
//! The scheduler shuts down when its control channel is dropped
//! (detected via `recv_timeout` returning `Disconnected`).
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — the periodic whole-state checkpoint this thread triggers

use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::{Receiver, Sender};
use tracing::{debug, info, warn};

use crate::owner::commands::{CheckpointRequest, ModelOwnerCommand};

// ═══════════════════════════════════════════════════════════════════════════════
// Scheduler
// ═══════════════════════════════════════════════════════════════════════════════

/// An explicit request for one checkpoint-scheduler trigger cycle.
///
/// Completion means the scheduler has placed its checkpoint request on the
/// model-owner command queue. Processing and writing that checkpoint belong to
/// the model owner and are deliberately outside this signal.
#[derive(Debug)]
pub struct CheckpointTrigger {
    /// Receives the signal after the checkpoint request enters the owner queue.
    completion: Sender<()>,
}

impl CheckpointTrigger {
    /// Creates a trigger whose receiver is the scheduler-cycle signal.
    #[cfg(any(test, feature = "test-support"))]
    pub(crate) const fn new(completion: Sender<()>) -> Self {
        Self { completion }
    }

    /// Announces that the checkpoint request entered the owner queue.
    fn complete(self) {
        let _completion_result = self.completion.send(());
    }
}

/// Spawns a checkpoint scheduler thread.
///
/// The thread waits on `control_rx` for up to `interval` between checkpoint
/// requests. It shuts down when that receiver is disconnected (the sender is
/// dropped by `Assayer::drop`).
///
/// Returns the thread handle.
///
/// # Thread Name
///
/// `"{instance_id}-checkpoint"`
pub fn spawn_checkpoint_scheduler(
    instance_id: &str,
    interval: Duration,
    command_tx: Sender<ModelOwnerCommand>,
    control_rx: Receiver<CheckpointTrigger>,
) -> JoinHandle<()> {
    let thread_name = format!("{instance_id}-checkpoint");
    let name_clone = thread_name.clone();

    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            info!("checkpoint scheduler started (interval {:?})", interval);

            loop {
                // Wait for either the interval to expire or shutdown signal.
                let trigger = match control_rx.recv_timeout(interval) {
                    Ok(trigger) => Some(trigger),
                    Err(crossbeam_channel::RecvTimeoutError::Timeout) => None,
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        info!("checkpoint scheduler shutting down");
                        return;
                    }
                };

                debug!("sending periodic checkpoint request");
                let request = CheckpointRequest { completion: None };
                if command_tx.send(ModelOwnerCommand::Checkpoint(request)).is_err() {
                    warn!("command channel disconnected; checkpoint scheduler exiting");
                    return;
                }
                if let Some(trigger) = trigger {
                    trigger.complete();
                }
            }
        })
        .unwrap_or_else(|e| panic!("failed to spawn checkpoint scheduler thread {name_clone:?}: {e}"))
}
