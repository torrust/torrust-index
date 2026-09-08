// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Background warming thread (S1 async, §ALGO S-18.2 Step 3).
//!
//! Runs the deferred cell warm-up loop on a dedicated thread, decoupling
//! noise injection latency from the `ingest()` hot path.
//!
//! # Design
//!
//! The thread owns its own `SmallRng` seeded from `noise_seed + 1`
//! (offset to avoid colliding with the main sentinel's RNG sequence).
//! It takes cells out of the staging area one at a time via
//! [`StagingArea::take_highest_priority`], does the expensive noise
//! injection *without holding the lock*, then returns the cell via
//! [`StagingArea::finish_warming`] or [`StagingArea::return_warming`].
//!
//! The main thread notifies the condvar after enqueueing new cells.
//! The thread sleeps on the condvar when there is nothing to warm.
//!
//! # Shutdown
//!
//! The sentinel sets `shutdown` to `true` and notifies the condvar.
//! The thread finishes any in-progress batch, then exits. The sentinel
//! joins the thread in [`WarmingThreadHandle::shutdown`] (called from
//! `Drop` or `reset()`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Condvar, Mutex};
use std::thread::JoinHandle;

use rand::SeedableRng;
use rand::rngs::SmallRng;
use torrust_mudlark::Coordinate;

use super::generate_noise_batch;
use super::staging::StagingArea;

// ─── Public handle ──────────────────────────────────────────

/// Handle to the background warming thread.
///
/// Owns the shutdown flag, condvar, and `JoinHandle`. The staging
/// `Arc<Mutex<StagingArea>>` is shared with the sentinel.
///
/// # Thread safety
///
/// All fields are `Send + Sync`:
/// - `Arc<AtomicBool>`, `Arc<Condvar>`: trivially `Send + Sync`.
/// - `Mutex<Option<JoinHandle<()>>>`: `Send + Sync` because
///   `JoinHandle<()>: Send`.
pub struct WarmingThreadHandle<C: Coordinate> {
    shutdown: Arc<AtomicBool>,
    condvar: Arc<Condvar>,
    /// The join handle is behind a `Mutex` so that `WarmingThreadHandle`
    /// is `Sync` (§ALGO S-18.2 Step 3.5 — `SpectralSentinel: Send + Sync`).
    handle: Mutex<Option<JoinHandle<()>>>,
    _marker: std::marker::PhantomData<C>,
}

impl<C: Coordinate> WarmingThreadHandle<C> {
    /// Spawn the background warming thread.
    ///
    /// # Arguments
    ///
    /// - `staging` — shared staging area (same `Arc` as the sentinel).
    /// - `batch_size` — number of synthetic samples per noise batch.
    /// - `noise_seed` — if `Some`, the thread's RNG is seeded
    ///   deterministically from `seed + 1`. If `None`, seeded from
    ///   system entropy.
    pub fn spawn(staging: &Arc<Mutex<StagingArea<C>>>, batch_size: usize, noise_seed: Option<u64>) -> Self {
        let shutdown = Arc::new(AtomicBool::new(false));
        let condvar = Arc::new(Condvar::new());

        let thread_staging = Arc::clone(staging);
        let thread_shutdown = Arc::clone(&shutdown);
        let thread_condvar = Arc::clone(&condvar);

        // Offset seed by 1 to avoid colliding with the main RNG.
        let rng = noise_seed.map_or_else(
            || SmallRng::from_rng(&mut rand::rng()),
            |s| SmallRng::seed_from_u64(s.wrapping_add(1)),
        );

        let handle = std::thread::Builder::new()
            .name("sentinel-warming".into())
            .spawn(move || {
                warming_loop(thread_staging, thread_condvar, thread_shutdown, batch_size, rng);
            })
            .expect("failed to spawn sentinel warming thread");

        Self {
            shutdown,
            condvar,
            handle: Mutex::new(Some(handle)),
            _marker: std::marker::PhantomData,
        }
    }

    /// Wake the background thread (call after enqueueing new cells).
    pub fn notify(&self) {
        self.condvar.notify_one();
    }

    /// Signal the thread to stop and wait for it to exit.
    ///
    /// Safe to call multiple times (subsequent calls are no-ops).
    pub fn shutdown(&self) {
        self.shutdown.store(true, Ordering::Release);
        self.condvar.notify_one();

        let handle = self.handle.lock().expect("warming handle poisoned").take();
        if let Some(handle) = handle {
            handle.join().expect("warming thread panicked");
        }
    }
}

impl<C: Coordinate> Drop for WarmingThreadHandle<C> {
    fn drop(&mut self) {
        self.shutdown();
    }
}

// ─── Thread loop ────────────────────────────────────────────

/// Main loop of the background warming thread.
///
/// 1. Wait on the condvar until there is warming work or shutdown.
/// 2. Take the highest-priority cell **out** of the staging area.
/// 3. Release the lock.
/// 4. Perform one noise batch (the expensive part).
/// 5. Re-acquire the lock and put the cell back (or move to ready).
/// 6. Repeat.
///
/// The lock is held only for O(|warming|) queue operations, never
/// during the SVD / noise injection. This keeps contention with the
/// main thread minimal.
#[allow(clippy::needless_pass_by_value)] // Arcs are moved from the thread closure
fn warming_loop<C: Coordinate>(
    staging: Arc<Mutex<StagingArea<C>>>,
    condvar: Arc<Condvar>,
    shutdown: Arc<AtomicBool>,
    batch_size: usize,
    mut rng: SmallRng,
) {
    loop {
        // ── Step 1: Wait for work ───────────────────────
        let work = {
            let mut guard = staging.lock().expect("staging mutex poisoned");

            while !guard.has_warming_work() && !shutdown.load(Ordering::Acquire) {
                guard = condvar.wait(guard).expect("staging condvar poisoned");
            }

            if shutdown.load(Ordering::Acquire) {
                // Shutdown requested — exit immediately. Any remaining
                // warming cells will be discarded by `reset()` / `Drop`.
                break;
            }

            // ── Step 2: Take a cell out ─────────────────
            guard.take_highest_priority()
        };
        // Lock released here.

        // ── Step 3: Do one batch of noise injection ─────
        let Some((gnode, mut wc)) = work else {
            // Spurious wake or concurrent take — loop back.
            continue;
        };

        let noise = generate_noise_batch(wc.cell.width, batch_size, &mut rng);
        let slices: Vec<&[f64]> = noise.iter().map(Vec::as_slice).collect();
        #[allow(clippy::cast_possible_truncation)] // depth ≤ 128, fits u8
        let report = wc.cell.tracker.observe(&slices, wc.cell.depth as u8, true);

        wc.round_scores.push([
            report.scores.novelty.mean,
            report.scores.displacement.mean,
            report.scores.surprise.mean,
            report.scores.coherence.mean,
        ]);
        wc.completed_rounds += 1;

        // ── Step 4: Return the cell ─────────────────────
        let mut guard = staging.lock().expect("staging mutex poisoned");

        if wc.is_ready() {
            wc.cell.tracker.seed_cusum_slow_from_baselines();
            wc.cell.tracker.reset_cusum();
            wc.cell.tracker.reset_clip_pressure();
            guard.finish_warming(gnode, wc.cell);
        } else {
            guard.return_warming(gnode, wc);
        }

        drop(guard);
    }
}

// ─── Compile-time safety ────────────────────────────────────

/// Verify `WarmingThreadHandle` is `Send + Sync` so it can live
/// inside `SpectralSentinel` without breaking the sentinel's own
/// `Send + Sync` obligation.
const _: () = {
    const fn assert_send_sync<T: Send + Sync>() {}
    assert_send_sync::<WarmingThreadHandle<u128>>();
};
