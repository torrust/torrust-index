// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Periodic Ledger collection scheduler.
//!
//! A dedicated maintenance thread that invokes the batched sweep on each
//! Sentinel's Ledger on a fixed cadence, hourly by default
//! (´alg:ledger:garbage-collection´), with the sweep bounded in both
//! phases (´dec:memory:periodic-sweep´). The eligibility floor and the
//! collection horizon come from the Ledger configuration rather than
//! from this caller.
//!
//! # Shutdown
//!
//! The scheduler shuts down when its control channel is dropped
//! (detected via `recv_timeout` returning `Disconnected`), mirroring
//! the checkpoint scheduler.

use std::sync::Arc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crossbeam_channel::Receiver;
use tracing::{debug, info};

use super::gc::{GcLimits, garbage_collect_batched};
use super::{OutcomeLedger, SweepConfig};
use crate::testing::Clock;

/// The fixed sweep cadence: the periodic strategy scans each Sentinel's
/// Ledger hourly by default (´alg:ledger:garbage-collection´).
///
/// ´const:assayer:collection-sweep-cadence´ (´alg:const:seconds´)
/// ´const:assayer:collection-sweep-cadence-seconds-3600´
pub const SWEEP_INTERVAL: Duration = Duration::from_secs(3600);

/// Spawns the Ledger collection scheduler thread.
///
/// On each tick the thread reads the present through the injected clock
/// and sweeps every Sentinel's Ledger with the configured floor and
/// horizon. It shuts down when `control_rx` is disconnected (the sender
/// is dropped by `Assayer::drop`).
///
/// # Thread Name
///
/// `"{instance_id}-ledger-gc"`
pub fn spawn_ledger_gc_scheduler(
    instance_id: &str,
    interval: Duration,
    ledger: Arc<OutcomeLedger>,
    sweep: SweepConfig,
    clock: Arc<dyn Clock>,
    control_rx: Receiver<()>,
) -> JoinHandle<()> {
    let thread_name = format!("{instance_id}-ledger-gc");
    let name_clone = thread_name.clone();

    thread::Builder::new()
        .name(thread_name)
        .spawn(move || {
            info!("ledger collection scheduler started (interval {:?})", interval);

            loop {
                match control_rx.recv_timeout(interval) {
                    Ok(()) | Err(crossbeam_channel::RecvTimeoutError::Timeout) => {
                        // Interval elapsed or explicit wake-up — sweep.
                    }
                    Err(crossbeam_channel::RecvTimeoutError::Disconnected) => {
                        info!("ledger collection scheduler shutting down");
                        return;
                    }
                }

                let outcome = sweep_all(&ledger, sweep, &*clock);
                debug!(
                    removed = outcome.removed,
                    scanned = outcome.scanned_entries,
                    "periodic ledger sweep complete"
                );
            }
        })
        .unwrap_or_else(|e| panic!("failed to spawn ledger GC scheduler thread {name_clone:?}: {e}"))
}

/// Sweeps every Sentinel's Ledger once and returns the merged outcome.
fn sweep_all(ledger: &OutcomeLedger, sweep: SweepConfig, clock: &dyn Clock) -> super::GcOutcome {
    let now = clock.now();
    let mut merged = super::GcOutcome::default();

    for sentinel_id in ledger.sentinel_ids() {
        let Some(sentinel_ledger) = ledger.get_arc(sentinel_id) else {
            continue;
        };
        let outcome = garbage_collect_batched(&sentinel_ledger, GcLimits::default(), sweep.floor, sweep.horizon, &now);
        merged.removed += outcome.removed;
        merged.scanned_entries += outcome.scanned_entries;
        merged.read_lock_acquisitions += outcome.read_lock_acquisitions;
        merged.write_lock_acquisitions += outcome.write_lock_acquisitions;
        merged.max_scan_window = merged.max_scan_window.max(outcome.max_scan_window);
        merged.max_delete_batch = merged.max_delete_batch.max(outcome.max_delete_batch);
    }

    merged
}
