// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Turning a harness that stops making progress into a harness that fails.
//!
//! Every wait an integration test takes on the label path is unbounded. A
//! flush sends a checkpoint command to the model owner and blocks on an
//! acknowledgement channel with no timeout; an assessment and a label
//! submission wait on the same thread's queues. That is the right shape for
//! the engine — a host wants the barrier, not a partial one — but it means a
//! model owner that stops answering leaves the test thread parked in a wait
//! its own loop conditions are never reached from. The test then reports
//! nothing at all: no assertion fails, no message is printed, and the run ends
//! when whatever is running it gives up on it.
//!
//! Two things here turn that into a failure. The first watches for the owner's
//! own death directly, because a panicking thread runs the process-wide panic
//! hook before it unwinds: a hook that recognises the owner by name can print
//! its message and end the process while the reason is still in hand. The
//! second watches for the absence of progress, which catches every other way a
//! wait can fail to return, and says how far the harness had got.
//!
//! Both are opt-in per test binary. Neither is armed by including this module.
//!
//! The third thing here is not opt-in: the deadline every bounded ack wait in
//! the harness and in the crate's own tests is written against. It answers the
//! same question those two do — is the loop still answering — for the waits
//! that already carry a timeout, and it answers it in one place instead of
//! once per call site.

use std::io::Write;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};

/// The exit status a failing test binary returns.
///
/// Ending the process with it is what makes a harness that cannot continue
/// count as a failed test rather than as a run that produced no verdict.
const TEST_FAILURE_STATUS: i32 = 101;

/// The substring that names the engine's model-owner thread.
///
/// The owner is spawned as `<instance>-model-owner`, so the instance id a
/// harness chose is a prefix and the suffix is the part every owner shares.
const OWNER_THREAD_MARKER: &str = "model-owner";

/// How long a harness wait for an acknowledgement stands before the test fails.
///
/// An ack is a liveness question and never a latency one. The harness sends a
/// command, a loop answers it, and the only thing the wait decides is whether
/// the loop is still answering at all; no test asserts anything about how long
/// the answer took, and a test that wanted to say the answer arrives quickly
/// would have to say so itself. The deadline is here for one reason: a loop
/// that has stopped answering makes its own test fail, with a message naming
/// the wait, instead of parking the test thread until whatever is running the
/// suite gives up on the whole run.
///
/// That is also why the number is generous rather than tight. A coverage build
/// instruments every function in the crate and its dependencies, and the same
/// suite then takes about a hundred times as long as an ordinary run; a
/// deadline sized for the ordinary run turns the instrumented run into a
/// fabricated failure, and a suite that cannot be measured under
/// instrumentation is a suite whose coverage nobody sees. Nothing is bought by
/// failing sooner, because a deadline is not a budget: the wait it bounds is
/// the same wait either way, and it expires only when the answer is never
/// coming. So it is sized from the instrumented measurement rather than the
/// ordinary one, with room for a coverage job that runs the whole workspace on
/// a machine far smaller than the one the measurement used. A hung loop still
/// fails; it fails later.
pub const ACK_DEADLINE: Duration = Duration::from_secs(300);

/// How long a harness wait for a piece of published state stands before the
/// test fails.
///
/// The same number as [`ACK_DEADLINE`], because it answers the same question.
/// Waiting for the snapshot version to reach a target, for a queue to drain or
/// for a thread to retire is a liveness question and never a latency one: the
/// wait ends when the loop answers, and the deadline decides only whether the
/// loop is still answering at all. Nothing is bought by failing sooner, and
/// everything is lost by failing too soon — an instrumented build of this
/// crate runs about a hundred times slower, so a deadline sized for an
/// ordinary run fabricates a failure in the coverage job and leaves the suite
/// unmeasurable. A test that wants to assert something about how quickly state
/// arrives has to say so itself, with its own assertion on elapsed time.
///
/// It carries its own name because the call sites read better for it and
/// because the two could legitimately diverge later; while they agree, this is
/// defined in terms of the other so that they cannot drift apart by accident.
pub const STATE_DEADLINE: Duration = ACK_DEADLINE;

/// Makes a model-owner panic end the process instead of stranding the test.
///
/// Installs a panic hook that runs the hook already in place — so the owner's
/// own message and location are printed exactly as they would have been — and
/// then, only if the panicking thread is the model owner, ends the process
/// with the failure status. Panics on any other thread are left alone, so an
/// ordinary failed assertion still unwinds into the test harness and a
/// `should_panic` test still passes.
///
/// Arming this is what makes the difference between a test that says the owner
/// died and a test that waits for an answer that is never coming.
pub fn fail_fast_on_model_owner_panic() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        previous(info);
        let current = std::thread::current();
        let name = current.name().unwrap_or_default();
        if name.contains(OWNER_THREAD_MARKER) {
            let mut stderr = std::io::stderr();
            writeln!(
                stderr,
                "the model owner thread '{name}' panicked; ending the process, because every wait \
                 the harness is about to take is on an answer that thread is no longer able to give"
            )
            .ok();
            stderr.flush().ok();
            // Justified: unwinding is not available here. The hook runs on the
            // owner's own thread, and nothing on the test thread is waiting on
            // a value that thread can still produce, so the only way to turn a
            // dead owner into a verdict is to end the process with the status a
            // failed test returns.
            #[allow(clippy::exit)]
            std::process::exit(TEST_FAILURE_STATUS);
        }
    }));
}

/// A counter a harness advances so a watcher can tell progress from a stall.
pub type Progress = Arc<AtomicU64>;

/// Watches a progress counter and ends the process if it stops moving.
///
/// Answers with the counter to advance. The watcher prints where the harness
/// has got to every `heartbeat`, so a run that is killed from outside still
/// carries its own trail, and ends the process with the failure status if the
/// counter has not moved for `stall`. It stops of its own accord once
/// [`finished`] has been called on the counter.
///
/// The stall bound is deliberately not a bound on the whole run: a schedule
/// that legitimately takes a long time should be allowed to, and what is never
/// legitimate is a schedule that stops advancing and stays that way.
#[must_use]
#[allow(clippy::print_stdout)] // Justified: the progress lines are this watcher's output contract, not stray prints — a liveness report a reader never sees is one that has not done its job.
pub fn watch_progress(stall: Duration, heartbeat: Duration) -> Progress {
    let progress = Arc::new(AtomicU64::new(0));
    let watched = Arc::clone(&progress);
    let started = Instant::now();
    std::thread::spawn(move || {
        let mut last_seen = 0_u64;
        let mut last_moved = Instant::now();
        loop {
            std::thread::sleep(heartbeat);
            let now = watched.load(Ordering::Relaxed);
            if now == u64::MAX {
                return;
            }
            let elapsed = started.elapsed().as_secs_f64();
            println!("liveness {elapsed:8.1}s progress={now}");
            std::io::stdout().flush().ok();
            if now != last_seen {
                last_seen = now;
                last_moved = Instant::now();
            } else if last_moved.elapsed() >= stall {
                println!(
                    "liveness: progress has stood at {now} for {:.0}s; ending the process, \
                     because a harness that has stopped advancing is a failed test and not a slow one",
                    last_moved.elapsed().as_secs_f64()
                );
                std::io::stdout().flush().ok();
                // Justified: the watcher runs on its own thread and the test
                // thread is parked in a wait that will not return, so a panic
                // raised here would be caught by nothing and the run would go
                // on standing still.
                #[allow(clippy::exit)]
                std::process::exit(TEST_FAILURE_STATUS);
            }
        }
    });
    progress
}

/// Tells the watcher its work is done.
pub fn finished(progress: &Progress) {
    progress.store(u64::MAX, Ordering::Relaxed);
}

/// Advances the counter by one.
pub fn advanced(progress: &Progress) {
    progress.fetch_add(1, Ordering::Relaxed);
}
