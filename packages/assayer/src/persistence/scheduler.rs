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

/// Spawns a checkpoint scheduler thread.
///
/// The thread sleeps for `interval` between checkpoint requests.
/// It shuts down when `control_rx` is disconnected (the sender is
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
    control_rx: Receiver<()>,
) -> JoinHandle<()> {
    let thread_name = format!("{instance_id}-checkpoint");
    let name_clone = thread_name.clone();

    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            info!("checkpoint scheduler started (interval {:?})", interval);

            loop {
                // Wait for either the interval to expire or shutdown signal.
                match control_rx.recv_timeout(interval) {
                    Ok(()) | Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Interval elapsed or explicit wake-up — send checkpoint request.
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        info!("checkpoint scheduler shutting down");
                        return;
                    }
                }

                debug!("sending periodic checkpoint request");
                let request = CheckpointRequest { completion: None };
                if command_tx.send(ModelOwnerCommand::Checkpoint(request)).is_err() {
                    warn!("command channel disconnected; checkpoint scheduler exiting");
                    return;
                }
            }
        })
        .unwrap_or_else(|e| panic!("failed to spawn checkpoint scheduler thread {name_clone:?}: {e}"))
}
