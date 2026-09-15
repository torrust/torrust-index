// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for the background warming thread's shutdown handshake.
//!
//! The worker sleeps on a condition variable whose predicate is two facts —
//! whether the staging area holds warming work, and whether shutdown has been
//! asked for — and it holds the staging mutex from the moment it reads them
//! until the wait releases it. A writer that changes either fact outside that
//! mutex can place the change and its wake-up inside that window, where the
//! wake-up reaches a thread that has not yet begun to wait; the worker then
//! sleeps on a predicate that has already changed and nothing changes it
//! again. Shutdown is the transition where that costs a hang rather than a
//! delay, because the joining thread waits for a worker that will never look
//! at the flag again.
//!
//! The test here cannot make the window open on demand: it is a few
//! instructions wide and the scheduler decides. What it can do is take the
//! bet often enough that a lost wake-up shows up as a thread that never
//! finishes, and bound the wait so the failure arrives as a failed assertion
//! rather than as a suite that stops.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`shutdown_returns_under_repeated_spawn_and_stop_cycles`] | warmup | Shutting the warming thread down returns, every time, over a long run of spawn-and-stop cycles that does nothing else — the arrangement that puts the request at its most likely to land while the worker is between reading its predicate and sleeping on it. A shutdown that is lost in that window does not fail loudly: the worker sleeps on, the join waits for it, and the sentinel's own drop never completes, so what a host would see is a process that stops rather than an error it can act on. |
//! | [`dropping_a_sentinel_consumes_a_failed_worker_join`] | warmup | A warming worker can fail before its owner is destroyed. Destruction still completes without unwinding, because the drop path records the failed join instead of turning a background failure into a destructor panic. |
//! | [`selection_refresh_survives_warming_handoffs`] | warmup | Waiting, in-flight and ready cells keep the latest selection flag across both worker return paths. |

use std::sync::mpsc::{self, RecvTimeoutError};
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::config::NoiseSchedule;
use crate::sentinel::staging::StagingArea;
use crate::sentinel::warming_thread::WarmingThreadHandle;
use crate::{SentinelConfig, SpectralSentinel};

/// How many spawn-and-stop cycles the witness runs.
const CYCLES: usize = 1_000;

/// How long the witness waits for those cycles before calling the handshake
/// broken. Well inside the package's five-second per-test budget, and orders
/// of magnitude above the cycles' own cost.
const DEADLINE: Duration = Duration::from_secs(3);

// ─── Shutdown handshake ─────────────────────────────────────

/// Shutting the warming thread down returns, every time, over a long run of
/// spawn-and-stop cycles that does nothing else — the arrangement that puts
/// the request at its most likely to land while the worker is between reading
/// its predicate and sleeping on it. A shutdown that is lost in that window
/// does not fail loudly: the worker sleeps on, the join waits for it, and the
/// sentinel's own drop never completes, so what a host would see is a process
/// that stops rather than an error it can act on.
///
/// ´claim:warmup:shutting-the-warming-thread-down-returns-however-the-request-races-the-worker-going-to-sleep´
/// ´test:crate:shutdown-returns-under-repeated-spawn-and-stop-cycles´
#[test]
fn shutdown_returns_under_repeated_spawn_and_stop_cycles() {
    let (done, finished) = mpsc::channel();

    // The cycles run on their own thread so that a lost wake-up is a
    // deadline this thread can observe. Joining them directly would make
    // the failure a hang, which no assertion can report.
    let cycles = std::thread::spawn(move || {
        for _ in 0..CYCLES {
            let staging = Arc::new(Mutex::new(StagingArea::<u128>::new()));
            let handle = WarmingThreadHandle::spawn(&staging, 4, Some(7)).expect("the environment must grant a warming thread");
            handle.shutdown();
        }
        // The receiver is gone only if the witness has already reported the
        // deadline, so there is nothing for this thread to do about it.
        let _reported = done.send(());
    });

    match finished.recv_timeout(DEADLINE) {
        Ok(()) => {}
        Err(RecvTimeoutError::Timeout) => {
            panic!(
                "{CYCLES} spawn-and-stop cycles did not finish within {DEADLINE:?}: a shutdown request was \
                 stored and notified while the worker was between reading its predicate and sleeping on it, \
                 so the worker never saw it and the join never returned"
            )
        }
        Err(RecvTimeoutError::Disconnected) => panic!("the cycling thread ended without reporting"),
    }

    cycles.join().expect("cycling thread panicked");
}

/// A warming worker can fail before its owner is destroyed. Destruction
/// still completes without unwinding, because the drop path records the failed
/// join instead of turning a background failure into a destructor panic.
///
/// ´claim:warmup:sentinel-destruction-consumes-a-failed-worker-join´
/// ´test:crate:dropping-a-sentinel-consumes-a-failed-worker-join´
#[test]
fn dropping_a_sentinel_consumes_a_failed_worker_join() {
    let config = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(Vec::new()),
        background_warming: true,
        ..SentinelConfig::<u64>::default()
    };
    let sentinel = SpectralSentinel::<u128, u64, 128>::new(config).unwrap();
    sentinel.fail_warming_worker_for_test();

    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| drop(sentinel)));

    assert!(outcome.is_ok(), "dropping the sentinel must consume the failed worker join");
}

/// Waiting, in-flight and ready cells keep the latest selection flag across both worker return paths.
///
/// ´claim:warmup:selection-refresh-survives-warming-handoffs´
/// ´test:crate:selection-refresh-survives-warming-handoffs´
#[test]
fn selection_refresh_survives_warming_handoffs() {
    let config = crate::SentinelConfig::<u64>::default();
    let cell = crate::sentinel::CellState {
        tracker: crate::sentinel::tracker::SubspaceTracker::new(4, &config, config.cusum_slow_decay),
        depth: 1,
        width: 4,
        start: 0_u128,
        end: 16,
        is_competitive: false,
    };
    let gnode = torrust_mudlark::GNodeId::from_parts(1, 0);
    let mut staging = StagingArea::new();
    staging.enqueue(gnode, cell, 2);
    staging.update_competitive(gnode, true);
    assert_eq!(staging.warming_competitive_count(), 1);

    let (_, warming) = staging.take_highest_priority().unwrap();
    staging.update_competitive(gnode, false);
    assert_eq!(staging.warming_competitive_count(), 0);
    staging.return_warming(gnode, warming);
    assert_eq!(staging.warming_competitive_count(), 0);

    let (_, warming) = staging.take_highest_priority().unwrap();
    assert!(!warming.cell.is_competitive);
    staging.update_competitive(gnode, true);
    assert_eq!(staging.warming_competitive_count(), 1);
    staging.finish_warming(gnode, warming.cell);
    let (_, ready) = staging.take_ready().pop().unwrap();
    assert!(ready.is_competitive);

    staging.enqueue(gnode, ready, 0);
    staging.update_competitive(gnode, false);
    assert!(!staging.take_ready().pop().unwrap().1.is_competitive);
}
