// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Staging area for deferred cell warm-up (S1, §ALGO S-18.2).
//!
//! New analysis cells are enqueued here for background noise injection
//! instead of being warmed inline during `ingest()`. This decouples
//! cell-creation latency from the observation hot path.
//!
//! # Lifecycle
//!
//! 1. **Enqueue** — `reconcile_analysis_set()` creates a `CellState` and
//!    enqueues it with `staging.enqueue(gnode, cell, target_rounds)`.
//! 2. **Warm** — the background thread (or synchronous drain) picks the
//!    highest-priority cell (by volume) and injects noise batches.
//! 3. **Ready** — when a cell completes all its target rounds, it moves
//!    to the ready queue.
//! 4. **Promote** — `take_ready()` returns completed cells for insertion
//!    into the live `cells` map.
//!
//! # Thread safety (Step 3)
//!
//! When background warming is enabled, the staging area lives behind
//! `Arc<Mutex<StagingArea>>`. The background thread takes cells out
//! via [`take_highest_priority`] (moving them to `in_flight`), does
//! the expensive noise injection *without* holding the lock, then
//! returns the cell via [`finish_warming`] or [`return_warming`].
//! The main thread can enqueue, evict, and promote while the
//! background thread is working.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`warming_cell_is_ready_when_complete`] | staging | A cell leaves the warming set only once it has served the whole schedule its depth called for: one round short and it is still warming. The target is a promise about how much noise the tracker's baselines were built from, so honouring it partially would put a half-formed reference into the live set. |
//! | [`warming_cell_is_ready_when_over_target`] | staging | cites (´claim:staging:a-cell-is-ready-only-once-it-has-completed-its-target-rounds´) |
//! | [`enqueue_zero_rounds_goes_directly_to_ready`] | staging | A cell whose schedule asks for no noise at all never enters the warming set: it is placed straight on the ready queue and can be promoted on the next pass. Warming is work done only where the schedule says it is needed, so a zero-round cell costs nothing to stage and waits for nothing. |
//! | [`enqueue_with_rounds_goes_to_warming`] | staging | A cell with rounds still to serve waits in the warming set, out of the ready queue — and it counts as present in the staging area from the moment it is enqueued. Presence is what stops the reconciler enqueuing the same cell twice while its warm-up is still outstanding. |
//! | [`take_ready_empties_queue`] | staging | Collecting the ready cells hands them over whole and leaves the queue empty behind them. Promotion moves ownership rather than copying it, so a cell cannot be promoted into the live map twice however often the observation path drains the staging area. |
//! | [`remove_from_warming`] | staging | Eviction finds a cell wherever it currently sits — here part-warmed, with rounds still outstanding — and afterwards the area no longer reports it as present. A cell that has left the analysis set must stop consuming warming effort immediately rather than at the end of its schedule. |
//! | [`remove_from_ready`] | staging | cites (´claim:staging:eviction-reaches-a-cell-in-whichever-state-it-is-being-held´) |
//! | [`remove_nonexistent_returns_false`] | staging | Asking to evict a cell the area never held is answered with "nothing removed" rather than a fault. Eviction is driven by graph rebalancing, which knows what left the analysis set but not which of those cells were ever staged, so a miss has to be an ordinary outcome. |
//! | [`clear_empties_everything`] | staging | Clearing empties every holding at once — warming, ready and in-flight alike — so that after a reset the total is zero rather than a residue in whichever state escaped the sweep. A sentinel being reset must not promote cells warmed against a model it has just discarded. |
//! | [`warm_one_batch_completes_single_round_cell`] | staging | A warming step that completes a cell's last round finalises it and moves it to the ready queue in the same step, so the cell is never left sitting complete but unclaimed. Finalisation is where the drift reference is seeded from the baselines the noise just built and the accumulated evidence is cleared — the injected noise must shape what counts as normal without itself counting as anomalous history. |
//! | [`warm_one_batch_returns_false_when_empty`] | staging | A warming step with nothing to warm reports that it did no work rather than failing or fabricating a round. The background thread drives this call in a loop, so "no work" is the signal that lets it go idle instead of spinning. |
//! | [`warm_one_batch_incremental_progress`] | staging | Warming advances one round per step: a cell needing several rounds stays in the warming set across the intermediate calls and moves to ready only on the step that finishes it. Splitting the work this way is the whole point of deferring warm-up — the cost of bringing a new cell online is spread over many steps instead of landing inside one observation call. |
//! | [`warm_one_batch_picks_highest_volume`] | staging | When several cells are waiting, the step spends its round on the one carrying the most traffic, leaving the quieter cell still warming. Volume is the cached importance of the backing graph node, so the cells the host is most likely to be asking about come online first, and — because a busy ancestor outweighs its own descendants — ancestors tend to arrive before the cells beneath them. |
//! | [`contains_checks_both_warming_and_ready`] | staging | Presence is answered across every state a staged cell can occupy: a cell still warming and a cell already waiting to be promoted both answer yes. The caller asking is deciding whether a cell needs creating, and it must not be told "absent" merely because the cell has moved on within the staging area. |
//! | [`take_highest_priority_moves_to_in_flight`] | staging | Checking a cell out for background work takes the busiest waiting cell and marks it in flight, leaving the others warming; while it is away it still counts as present in the staging area. That is what makes the expensive noise injection safe to do without holding the lock: the main thread can see the cell is spoken for even though the warming map no longer holds it. |
//! | [`equal_volumes_take_the_shallower_cell_first`] | staging | Equal volumes resolve to the shallower cell rather than the deeper one. A tie is the ordinary case for a pair of siblings the moment they are created, and the rule the queue exists to serve is that a busy ancestor is warmed before the cells beneath it. Identifiers are handed out as the tree grows downward, so the larger of two is always the newer and deeper cell, and resolving a tie toward it inverts the rule exactly. This path and the synchronous drain are two ways of serving one queue, so they must not disagree about which cell comes next. |
//! | [`a_newly_queued_cell_carries_its_volume`] | staging | A cell joining the queue carries its volume with it instead of waiting for a later pass to supply one. The queue is served highest volume first, so a cell admitted at zero is indistinguishable from a cell with no traffic behind it, and a field of zeroes is decided entirely by the tie-break — which puts the newest and deepest cell first, the inverse of what the queue is for. Refreshing the cached volumes before the new cells are added rather than after leaves every one of them in exactly that state until some later pass happens to refresh again. |
//! | [`return_warming_restores_cell`] | staging | A cell handed back unfinished rejoins the warming set and stops being in flight, with its accumulated rounds intact. Background warming can therefore be interrupted between rounds — the thread need not carry a cell to completion once it has taken it. |
//! | [`finish_warming_moves_to_ready`] | staging | A cell handed back finished joins the ready queue instead of the warming set, and is no longer in flight. Which of the two return paths the background thread takes is what decides the cell's fate, so completion is declared by the worker that did the rounds rather than re-derived by the staging area. |
//! | [`eviction_of_in_flight_cell_discards_on_return`] | staging | A cell evicted while a background thread was working on it is discarded when it comes back, not resurrected: the eviction sweep removes its in-flight mark, and a return with no mark to clear keeps nothing. The warming work already spent is lost, which is the deliberate trade — a cell that has left the analysis set must not reappear in it because a thread happened to be holding it. |
//! | [`gnode_set_includes_all_states`] | staging | cites (´claim:staging:presence-is-answered-across-every-state-a-staged-cell-can-occupy´) |

use std::collections::{BTreeMap, BTreeSet};

use rand::rngs::SmallRng;
use torrust_mudlark::{Coordinate, GNodeId, GvGraph, Inspectable};

use super::{CellState, generate_noise_batch};

// ─── WarmingCell ────────────────────────────────────────────

/// Progress state of a cell being warmed in the background.
pub struct WarmingCell<C: Coordinate> {
    /// The cell under construction.
    pub cell: CellState<C>,

    /// Total noise rounds required (from `NoiseSchedule::rounds_for_depth()`).
    pub target_rounds: u32,

    /// Rounds completed so far.
    pub completed_rounds: u32,

    /// Accumulated per-round mean score vectors (4D) for coordination
    /// warm-up at promotion time.
    pub round_scores: Vec<[f64; 4]>,

    /// Cached volume (g.sum) for priority ordering, erased to `f64`.
    /// Updated by the main thread during `reconcile_analysis_set()` via
    /// [`StagingArea::update_volumes`].
    pub volume: f64,
}

impl<C: Coordinate> WarmingCell<C> {
    /// Whether this cell has completed all its target noise rounds.
    #[must_use]
    pub const fn is_ready(&self) -> bool {
        self.completed_rounds >= self.target_rounds
    }
}

// ─── StagingArea ────────────────────────────────────────────

/// Staging area for cells undergoing deferred noise warm-up.
///
/// In synchronous mode this is owned directly by the sentinel.
/// In background-warming mode it lives behind `Arc<Mutex<_>>`
/// and is shared between the main thread and the warming thread.
///
/// `ingest()` reads only the `ready` queue (via `take_ready()`).
pub struct StagingArea<C: Coordinate> {
    /// Cells currently being warmed, keyed by `GNodeId`.
    ///
    /// `BTreeMap` for deterministic iteration order (ADR-S-005).
    warming: BTreeMap<GNodeId, WarmingCell<C>>,

    /// Cells that completed warming since the last promotion.
    ready: Vec<(GNodeId, CellState<C>)>,

    /// Cells currently checked out by the background warming thread.
    ///
    /// These are logically still "in the staging area" — they have
    /// been temporarily removed from `warming` so the thread can
    /// work on them without holding the lock. [`contains`] and
    /// [`retain_in_set`] account for them.
    in_flight: BTreeSet<GNodeId>,
}

impl<C: Coordinate> StagingArea<C> {
    /// Create an empty staging area.
    pub const fn new() -> Self {
        Self {
            warming: BTreeMap::new(),
            ready: Vec::new(),
            in_flight: BTreeSet::new(),
        }
    }

    // ── Enqueue / promote ───────────────────────────────

    /// Enqueue a newly created cell for background warming.
    ///
    /// The cell is added to the warming set with zero completed rounds.
    /// If `target_rounds` is zero, the cell goes directly to the ready
    /// queue (no warming needed).
    pub fn enqueue(&mut self, gnode: GNodeId, cell: CellState<C>, target_rounds: u32) {
        if target_rounds == 0 {
            self.ready.push((gnode, cell));
            return;
        }

        self.warming.insert(
            gnode,
            WarmingCell {
                cell,
                target_rounds,
                completed_rounds: 0,
                round_scores: Vec::with_capacity(target_rounds as usize),
                volume: 0.0,
            },
        );
    }

    /// Take all ready cells for promotion into the live cells map.
    ///
    /// The caller inserts them into `self.cells` and fires coordination
    /// warm-up as needed.
    pub fn take_ready(&mut self) -> Vec<(GNodeId, CellState<C>)> {
        std::mem::take(&mut self.ready)
    }

    // ── Background-thread interface (Step 3) ────────────

    /// Remove the highest-priority warming cell for background processing.
    ///
    /// The cell moves from `warming` to `in_flight`. The caller is
    /// responsible for returning it via [`return_warming`] or
    /// [`finish_warming`].
    ///
    /// Priority is by cached `volume` (largest first), ties resolving to the
    /// smaller `GNodeId` — the shallower, earlier-allocated cell — so that
    /// this path and the synchronous drain order equal-volume cells the same
    /// way, ancestors before the cells beneath them.
    pub fn take_highest_priority(&mut self) -> Option<(GNodeId, WarmingCell<C>)> {
        let (&gnode, _) = self.warming.iter().max_by(|(a_gnode, a), (b_gnode, b)| {
            a.volume
                .partial_cmp(&b.volume)
                .unwrap_or(std::cmp::Ordering::Equal)
                // Reversed, because `max_by` keeps the last of equal maxima
                // and the map iterates in ascending id order: comparing ids
                // backwards makes the smallest id the maximum.
                .then_with(|| b_gnode.cmp(a_gnode))
        })?;
        let wc = self.warming.remove(&gnode)?;
        self.in_flight.insert(gnode);
        Some((gnode, wc))
    }

    /// Return an in-flight cell to the warming map (not yet ready).
    ///
    /// If the cell was evicted while in-flight (removed from
    /// `in_flight` by [`retain_in_set`]), the cell is silently
    /// discarded — the work is wasted but correctness is preserved.
    pub fn return_warming(&mut self, gnode: GNodeId, wc: WarmingCell<C>) {
        if self.in_flight.remove(&gnode) {
            self.warming.insert(gnode, wc);
        }
        // else: evicted while in-flight — discard.
    }

    /// Complete an in-flight cell and add it to the ready queue.
    ///
    /// If the cell was evicted while in-flight, it is silently
    /// discarded.
    pub fn finish_warming(&mut self, gnode: GNodeId, cell: CellState<C>) {
        if self.in_flight.remove(&gnode) {
            self.ready.push((gnode, cell));
        }
        // else: evicted while in-flight — discard.
    }

    // ── Warm-one-batch (used by background thread) ──────

    /// Pick the highest-priority warming cell (largest `volume`) and
    /// inject one noise batch. Returns `true` if a batch was injected.
    ///
    /// Used by the Step 1 unit tests and by the synchronous fallback
    /// in [`warm_one_batch`] with graph-volume updates. The background
    /// thread uses [`take_highest_priority`] instead to avoid holding
    /// the lock during expensive noise injection.
    #[allow(dead_code)] // used in tests
    pub fn warm_one_batch<V: Inspectable, const N: u32>(
        &mut self,
        graph: &GvGraph<C, V, N>,
        batch_size: usize,
        rng: &mut SmallRng,
    ) -> bool {
        // Find the highest-priority warming cell.
        let Some((&gnode, _)) = self
            .warming
            .iter()
            .max_by(|(_, a), (_, b)| a.volume.partial_cmp(&b.volume).unwrap_or(std::cmp::Ordering::Equal))
        else {
            return false;
        };

        // Remove temporarily to satisfy the borrow checker, then re-insert.
        let Some(mut wc) = self.warming.remove(&gnode) else {
            return false;
        };

        // Generate and inject one noise batch.
        let noise = generate_noise_batch(wc.cell.width, batch_size, rng);
        let slices: Vec<&[f64]> = noise.iter().map(Vec::as_slice).collect();
        #[allow(clippy::cast_possible_truncation)] // depth ≤ 128, fits in u8
        let report = wc.cell.tracker.observe(&slices, wc.cell.depth as u8, true);

        wc.round_scores.push([
            report.scores.novelty.mean,
            report.scores.displacement.mean,
            report.scores.surprise.mean,
            report.scores.coherence.mean,
        ]);
        wc.completed_rounds += 1;

        if wc.is_ready() {
            // Finalize: seed CUSUM slow from baselines and reset.
            wc.cell.tracker.seed_cusum_slow_from_baselines();
            wc.cell.tracker.reset_cusum();
            wc.cell.tracker.reset_clip_pressure();
            self.ready.push((gnode, wc.cell));
        } else {
            self.warming.insert(gnode, wc);
        }

        // Update volumes from the graph for remaining warming cells.
        for (&g, wc) in &mut self.warming {
            wc.volume = graph.gnode_info(g).map_or(0.0, |info| info.sum.to_f64_approx());
        }

        true
    }

    // ── Query / eviction ────────────────────────────────

    /// Check if a `GNodeId` is currently in the staging area
    /// (warming, ready, or in-flight).
    pub fn contains(&self, gnode: GNodeId) -> bool {
        self.warming.contains_key(&gnode) || self.in_flight.contains(&gnode) || self.ready.iter().any(|(g, _)| *g == gnode)
    }

    /// Set of all `GNodeId`s currently in the staging area.
    ///
    /// Useful for diagnostics and testing.
    #[allow(dead_code)] // used in tests; no longer needed for routing
    pub fn gnode_set(&self) -> BTreeSet<GNodeId> {
        let mut set: BTreeSet<GNodeId> = self.warming.keys().copied().collect();
        set.extend(&self.in_flight);
        for (g, _) in &self.ready {
            set.insert(*g);
        }
        set
    }

    /// Remove a cell from the staging area (e.g. evicted by graph rebalance).
    ///
    /// Returns `true` if the cell was found and removed.
    #[allow(dead_code)] // used in tests, will be called from Step 3 thread lifecycle
    pub fn remove(&mut self, gnode: GNodeId) -> bool {
        if self.warming.remove(&gnode).is_some() {
            return true;
        }
        if self.in_flight.remove(&gnode) {
            return true;
        }
        let before = self.ready.len();
        self.ready.retain(|(g, _)| *g != gnode);
        self.ready.len() < before
    }

    /// Retain only cells whose `GNodeId` is in `keep`.
    ///
    /// Evicts warming, ready, *and* in-flight cells that exited the
    /// analysis set. In-flight cells evicted here will be silently
    /// discarded when the background thread tries to return them
    /// (the `in_flight` entry is gone, so [`return_warming`] /
    /// [`finish_warming`] no-ops).
    pub fn retain_in_set(&mut self, keep: &BTreeSet<GNodeId>) {
        self.warming.retain(|gnode, _| keep.contains(gnode));
        self.ready.retain(|(gnode, _)| keep.contains(gnode));
        self.in_flight.retain(|gnode| keep.contains(gnode));
    }

    // ── Volume update ───────────────────────────────────

    /// Update cached volumes for warming cells from the G-V Graph.
    ///
    /// Called by the main thread after G-V Graph observation so the
    /// background thread can prioritise cells by current importance.
    pub fn update_volumes<V: Inspectable, const N: u32>(&mut self, graph: &GvGraph<C, V, N>) {
        for (&g, wc) in &mut self.warming {
            wc.volume = graph.gnode_info(g).map_or(0.0, |info| info.sum.to_f64_approx());
        }
    }

    // ── Synchronous drain (Step 2 fallback) ─────────────

    /// Drain all warming cells to completion synchronously.
    ///
    /// Processes cells in descending g.sum (volume) order, matching
    /// the background thread's priority rule (§ALGO S-11.6.2,
    /// ADR-S-019). This ensures ancestors come online before
    /// descendants even in synchronous mode.
    #[allow(clippy::cast_possible_truncation)] // depth ≤ 128, fits u8
    pub fn drain_all_synchronous(&mut self, batch_size: usize, rng: &mut SmallRng) {
        // Sort by cached volume (g.sum) descending, then GNodeId for
        // deterministic tie-breaking (ADR-S-005).
        let mut gnodes: Vec<(GNodeId, f64)> = self.warming.iter().map(|(&gnode, wc)| (gnode, wc.volume)).collect();
        gnodes.sort_by(|a, b| {
            b.1.partial_cmp(&a.1)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| a.0.cmp(&b.0))
        });

        for (gnode, _) in gnodes {
            let Some(mut wc) = self.warming.remove(&gnode) else {
                continue;
            };

            // Warm to completion — same loop as inject_noise_into_cell().
            while !wc.is_ready() {
                let noise = generate_noise_batch(wc.cell.width, batch_size, rng);
                let slices: Vec<&[f64]> = noise.iter().map(Vec::as_slice).collect();
                let report = wc.cell.tracker.observe(&slices, wc.cell.depth as u8, true);

                wc.round_scores.push([
                    report.scores.novelty.mean,
                    report.scores.displacement.mean,
                    report.scores.surprise.mean,
                    report.scores.coherence.mean,
                ]);
                wc.completed_rounds += 1;
            }

            // Finalize: seed CUSUM slow from baselines and reset.
            wc.cell.tracker.seed_cusum_slow_from_baselines();
            wc.cell.tracker.reset_cusum();
            wc.cell.tracker.reset_clip_pressure();
            self.ready.push((gnode, wc.cell));
        }
    }

    // ── Count / clear ───────────────────────────────────

    /// Number of cells currently warming (not yet ready, not in-flight).
    #[allow(dead_code)] // used in tests + Step 6 health reporting
    pub fn warming_count(&self) -> usize {
        self.warming.len()
    }

    /// Number of cells in the ready queue awaiting promotion.
    #[allow(dead_code)] // used in tests + Step 6 health reporting
    pub const fn ready_count(&self) -> usize {
        self.ready.len()
    }

    /// Number of cells currently checked out by the background thread.
    #[allow(dead_code)] // used in tests + Step 6 health reporting
    pub fn in_flight_count(&self) -> usize {
        self.in_flight.len()
    }

    /// Total cells in the staging area (warming + ready + in-flight).
    pub fn total_count(&self) -> usize {
        self.warming.len() + self.ready.len() + self.in_flight.len()
    }

    /// Number of warming cells (including in-flight) that are
    /// competitive targets (not ancestor-only).
    ///
    /// Used to populate `HealthReport::warming_competitive_targets`
    /// (§ALGO S-14.11, ADR-S-019).
    pub fn warming_competitive_count(&self) -> usize {
        self.warming.values().filter(|wc| wc.cell.is_competitive).count()
        // Note: in-flight cells are not counted here because their
        // competitive status may have changed since checkout. This
        // is conservative — the count may undercount by at most the
        // number of in-flight cells (typically 0 or 1).
    }

    /// Whether there are any cells that need warming work.
    ///
    /// Returns `true` if `warming` is non-empty (excludes in-flight
    /// and ready cells — those are already being processed or done).
    pub fn has_warming_work(&self) -> bool {
        !self.warming.is_empty()
    }

    /// Clear all warming, ready, and in-flight cells (for `reset()`).
    pub fn clear(&mut self) {
        self.warming.clear();
        self.ready.clear();
        self.in_flight.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::SentinelConfig;
    use crate::sentinel::tracker::SubspaceTracker;

    fn make_cell(depth: u32) -> CellState<u128> {
        let cfg = SentinelConfig::<u64>::default();
        let width = 128usize.saturating_sub(depth as usize);
        CellState {
            tracker: SubspaceTracker::new(width, &cfg, cfg.cusum_slow_decay),
            depth,
            width,
            start: 0,
            end: u128::MAX >> depth,
            is_competitive: false,
        }
    }

    /// Create a graph with a split so we can extract multiple distinct `GNodeId`s.
    ///
    /// Returns `(graph, root, left_child, right_child)` where the children
    /// are `Option<GNodeId>` (will be `Some` after the split).
    fn split_graph() -> (GvGraph<u128, u64, 128>, GNodeId, GNodeId, GNodeId) {
        use torrust_mudlark::Config as GvConfig;
        let config = GvConfig {
            split_threshold: 2u64,
            depth_create: 3,
            depth_evict: 6,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        let mut graph: GvGraph<u128, u64, 128> = GvGraph::new(config);
        let root = graph.g_root();

        // Observe enough in both halves to trigger a split.
        let lo = 0u128;
        let hi = u128::MAX / 2 + 1;
        for _ in 0..5 {
            graph.observe(lo, 1u64);
            graph.observe(hi, 1u64);
        }

        let children = graph.gnode_children(root).expect("root must exist");
        let left = children.left.expect("root must have left child after split");
        let right = children.right.expect("root must have right child after split");
        (graph, root, left, right)
    }

    /// A cell leaves the warming set only once it has served the whole
    /// schedule its depth called for: one round short and it is still
    /// warming. The target is a promise about how much noise the tracker's
    /// baselines were built from, so honouring it partially would put a
    /// half-formed reference into the live set.
    ///
    /// ´claim:staging:a-cell-is-ready-only-once-it-has-completed-its-target-rounds´
    /// ´test:unit:warming-cell-is-ready-when-complete´
    #[test]
    fn warming_cell_is_ready_when_complete() {
        let cell = make_cell(0);
        let wc = WarmingCell {
            cell,
            target_rounds: 5,
            completed_rounds: 4,
            round_scores: Vec::new(),
            volume: 100.0,
        };
        assert!(!wc.is_ready());

        let cell2 = make_cell(0);
        let wc2 = WarmingCell {
            cell: cell2,
            target_rounds: 5,
            completed_rounds: 5,
            round_scores: Vec::new(),
            volume: 100.0,
        };
        assert!(wc2.is_ready());
    }

    /// Readiness is a threshold rather than an exact match: a cell that has
    /// somehow run past its target is ready, not stuck. Nothing in the
    /// pipeline has to guarantee that rounds land on the target exactly, so
    /// an extra round of noise costs work and never a wedged cell.
    ///
    /// (´claim:staging:a-cell-is-ready-only-once-it-has-completed-its-target-rounds´)
    /// ´test:unit:warming-cell-is-ready-when-over-target´
    #[test]
    fn warming_cell_is_ready_when_over_target() {
        let cell = make_cell(0);
        let wc = WarmingCell {
            cell,
            target_rounds: 3,
            completed_rounds: 10,
            round_scores: Vec::new(),
            volume: 0.0,
        };
        assert!(wc.is_ready());
    }

    /// A cell whose schedule asks for no noise at all never enters the
    /// warming set: it is placed straight on the ready queue and can be
    /// promoted on the next pass. Warming is work done only where the
    /// schedule says it is needed, so a zero-round cell costs nothing to
    /// stage and waits for nothing.
    ///
    /// ´claim:staging:a-cell-that-needs-no-warming-skips-the-queue-and-arrives-ready´
    /// ´test:unit:enqueue-zero-rounds-goes-directly-to-ready´
    #[test]
    fn enqueue_zero_rounds_goes_directly_to_ready() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, _, _) = split_graph();
        let cell = make_cell(0);

        staging.enqueue(root, cell, 0);

        assert_eq!(staging.warming_count(), 0);
        assert_eq!(staging.ready_count(), 1);

        let ready = staging.take_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].0, root);
    }

    /// A cell with rounds still to serve waits in the warming set, out of
    /// the ready queue — and it counts as present in the staging area from
    /// the moment it is enqueued. Presence is what stops the reconciler
    /// enqueuing the same cell twice while its warm-up is still outstanding.
    ///
    /// ´claim:staging:a-cell-counts-as-staged-from-the-moment-it-is-enqueued-not-from-when-it-is-ready´
    /// ´test:unit:enqueue-with-rounds-goes-to-warming´
    #[test]
    fn enqueue_with_rounds_goes_to_warming() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, _, _) = split_graph();
        let cell = make_cell(0);

        staging.enqueue(root, cell, 5);

        assert_eq!(staging.warming_count(), 1);
        assert_eq!(staging.ready_count(), 0);
        assert!(staging.contains(root));
    }

    /// Collecting the ready cells hands them over whole and leaves the queue
    /// empty behind them. Promotion moves ownership rather than copying it,
    /// so a cell cannot be promoted into the live map twice however often
    /// the observation path drains the staging area.
    ///
    /// ´claim:staging:collecting-the-ready-cells-hands-them-over-whole-and-leaves-the-queue-empty´
    /// ´test:unit:take-ready-empties-queue´
    #[test]
    fn take_ready_empties_queue() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, _, _) = split_graph();
        let cell = make_cell(0);

        staging.enqueue(root, cell, 0);
        assert_eq!(staging.ready_count(), 1);

        let ready = staging.take_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(staging.ready_count(), 0);
    }

    /// Eviction finds a cell wherever it currently sits — here part-warmed,
    /// with rounds still outstanding — and afterwards the area no longer
    /// reports it as present. A cell that has left the analysis set must
    /// stop consuming warming effort immediately rather than at the end of
    /// its schedule.
    ///
    /// ´claim:staging:eviction-reaches-a-cell-in-whichever-state-it-is-being-held´
    /// ´test:unit:remove-from-warming´
    #[test]
    fn remove_from_warming() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, _) = split_graph();
        let cell = make_cell(1);

        staging.enqueue(left, cell, 10);
        assert!(staging.contains(left));

        assert!(staging.remove(left));
        assert!(!staging.contains(left));
        assert_eq!(staging.warming_count(), 0);
    }

    /// The other end of the same reach: a cell already finished and waiting
    /// on the ready queue is evicted just as a warming one is. Having
    /// completed its warm-up buys a cell no claim on promotion once it has
    /// left the analysis set.
    ///
    /// (´claim:staging:eviction-reaches-a-cell-in-whichever-state-it-is-being-held´)
    /// ´test:unit:remove-from-ready´
    #[test]
    fn remove_from_ready() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, _, _) = split_graph();
        let cell = make_cell(0);

        staging.enqueue(root, cell, 0);
        assert!(staging.contains(root));

        assert!(staging.remove(root));
        assert!(!staging.contains(root));
        assert_eq!(staging.ready_count(), 0);
    }

    /// Asking to evict a cell the area never held is answered with "nothing
    /// removed" rather than a fault. Eviction is driven by graph rebalancing,
    /// which knows what left the analysis set but not which of those cells
    /// were ever staged, so a miss has to be an ordinary outcome.
    ///
    /// ´claim:staging:evicting-a-cell-the-area-never-held-reports-that-nothing-was-removed´
    /// ´test:unit:remove-nonexistent-returns-false´
    #[test]
    fn remove_nonexistent_returns_false() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, _, _) = split_graph();
        assert!(!staging.remove(root));
    }

    /// Clearing empties every holding at once — warming, ready and in-flight
    /// alike — so that after a reset the total is zero rather than a residue
    /// in whichever state escaped the sweep. A sentinel being reset must not
    /// promote cells warmed against a model it has just discarded.
    ///
    /// ´claim:staging:clearing-empties-every-holding-at-once-so-a-reset-leaves-no-residue´
    /// ´test:unit:clear-empties-everything´
    #[test]
    fn clear_empties_everything() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, right) = split_graph();

        staging.enqueue(left, make_cell(1), 5);
        staging.enqueue(right, make_cell(1), 0);

        assert_eq!(staging.total_count(), 2);

        staging.clear();
        assert_eq!(staging.total_count(), 0);
        assert_eq!(staging.warming_count(), 0);
        assert_eq!(staging.ready_count(), 0);
    }

    /// A warming step that completes a cell's last round finalises it and
    /// moves it to the ready queue in the same step, so the cell is never
    /// left sitting complete but unclaimed. Finalisation is where the drift
    /// reference is seeded from the baselines the noise just built and the
    /// accumulated evidence is cleared — the injected noise must shape what
    /// counts as normal without itself counting as anomalous history.
    ///
    /// ´claim:staging:a-cell-whose-last-round-lands-is-finalised-and-moved-to-ready-in-the-same-step´
    /// ´test:unit:warm-one-batch-completes-single-round-cell´
    #[test]
    fn warm_one_batch_completes_single_round_cell() {
        use rand::SeedableRng;
        let mut staging = StagingArea::<u128>::new();
        let (graph, _, left, _) = split_graph();
        let cell = make_cell(1);

        // Enqueue with target_rounds = 1: one batch should complete it.
        staging.enqueue(left, cell, 1);
        assert_eq!(staging.warming_count(), 1);

        let mut rng = SmallRng::seed_from_u64(42);
        let warmed = staging.warm_one_batch(&graph, 64, &mut rng);
        assert!(warmed);

        // Cell should have moved to ready.
        assert_eq!(staging.warming_count(), 0);
        assert_eq!(staging.ready_count(), 1);

        let ready = staging.take_ready();
        assert_eq!(ready.len(), 1);
        assert_eq!(ready[0].0, left);
    }

    /// A warming step with nothing to warm reports that it did no work
    /// rather than failing or fabricating a round. The background thread
    /// drives this call in a loop, so "no work" is the signal that lets it
    /// go idle instead of spinning.
    ///
    /// ´claim:staging:a-warming-step-with-nothing-to-warm-reports-that-it-did-no-work´
    /// ´test:unit:warm-one-batch-returns-false-when-empty´
    #[test]
    fn warm_one_batch_returns_false_when_empty() {
        use rand::SeedableRng;
        let mut staging = StagingArea::<u128>::new();
        let (graph, _, _, _) = split_graph();
        let mut rng = SmallRng::seed_from_u64(42);

        assert!(!staging.warm_one_batch(&graph, 64, &mut rng));
    }

    /// Warming advances one round per step: a cell needing several rounds
    /// stays in the warming set across the intermediate calls and moves to
    /// ready only on the step that finishes it. Splitting the work this way
    /// is the whole point of deferring warm-up — the cost of bringing a new
    /// cell online is spread over many steps instead of landing inside one
    /// observation call.
    ///
    /// ´claim:staging:warming-advances-one-round-per-step-so-the-cost-of-a-new-cell-is-spread-out´
    /// ´test:unit:warm-one-batch-incremental-progress´
    #[test]
    fn warm_one_batch_incremental_progress() {
        use rand::SeedableRng;
        let mut staging = StagingArea::<u128>::new();
        let (graph, _, left, _) = split_graph();
        let cell = make_cell(1);

        staging.enqueue(left, cell, 3);
        let mut rng = SmallRng::seed_from_u64(42);

        // First batch: still warming.
        assert!(staging.warm_one_batch(&graph, 64, &mut rng));
        assert_eq!(staging.warming_count(), 1);
        assert_eq!(staging.ready_count(), 0);

        // Second batch: still warming.
        assert!(staging.warm_one_batch(&graph, 64, &mut rng));
        assert_eq!(staging.warming_count(), 1);
        assert_eq!(staging.ready_count(), 0);

        // Third batch: completes.
        assert!(staging.warm_one_batch(&graph, 64, &mut rng));
        assert_eq!(staging.warming_count(), 0);
        assert_eq!(staging.ready_count(), 1);
    }

    /// When several cells are waiting, the step spends its round on the one
    /// carrying the most traffic, leaving the quieter cell still warming.
    /// Volume is the cached importance of the backing graph node, so the
    /// cells the host is most likely to be asking about come online first,
    /// and — because a busy ancestor outweighs its own descendants —
    /// ancestors tend to arrive before the cells beneath them.
    ///
    /// ´claim:staging:warming-effort-goes-first-to-the-waiting-cell-carrying-the-most-traffic´
    /// ´test:unit:warm-one-batch-picks-highest-volume´
    #[test]
    fn warm_one_batch_picks_highest_volume() {
        use rand::SeedableRng;
        let mut staging = StagingArea::<u128>::new();
        let (graph, _, left, right) = split_graph();

        // Enqueue two cells with different target_rounds.
        staging.enqueue(left, make_cell(1), 1);
        staging.enqueue(right, make_cell(1), 1);

        // Manually set volumes so right > left.
        staging.warming.get_mut(&left).unwrap().volume = 10.0;
        staging.warming.get_mut(&right).unwrap().volume = 100.0;

        let mut rng = SmallRng::seed_from_u64(42);

        // First warm_one_batch should pick right (higher volume).
        assert!(staging.warm_one_batch(&graph, 64, &mut rng));
        // right should be ready now (target_rounds=1).
        assert!(staging.ready.iter().any(|(g, _)| *g == right));
        assert_eq!(staging.warming_count(), 1);
        assert!(staging.warming.contains_key(&left));
    }

    /// Presence is answered across every state a staged cell can occupy: a
    /// cell still warming and a cell already waiting to be promoted both
    /// answer yes. The caller asking is deciding whether a cell needs
    /// creating, and it must not be told "absent" merely because the cell
    /// has moved on within the staging area.
    ///
    /// ´claim:staging:presence-is-answered-across-every-state-a-staged-cell-can-occupy´
    /// ´test:unit:contains-checks-both-warming-and-ready´
    #[test]
    fn contains_checks_both_warming_and_ready() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, right) = split_graph();

        staging.enqueue(left, make_cell(1), 5); // goes to warming
        staging.enqueue(right, make_cell(1), 0); // goes to ready

        assert!(staging.contains(left));
        assert!(staging.contains(right));
    }

    // ── Step 3 in-flight tests ──────────────────────────

    /// Checking a cell out for background work takes the busiest waiting
    /// cell and marks it in flight, leaving the others warming; while it is
    /// away it still counts as present in the staging area. That is what
    /// makes the expensive noise injection safe to do without holding the
    /// lock: the main thread can see the cell is spoken for even though the
    /// warming map no longer holds it.
    ///
    /// ´claim:staging:a-cell-checked-out-for-background-work-stays-logically-present-while-it-is-away´
    /// ´test:unit:take-highest-priority-moves-to-in-flight´
    #[test]
    fn take_highest_priority_moves_to_in_flight() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, right) = split_graph();

        staging.enqueue(left, make_cell(1), 3);
        staging.enqueue(right, make_cell(1), 3);
        staging.warming.get_mut(&left).unwrap().volume = 10.0;
        staging.warming.get_mut(&right).unwrap().volume = 100.0;

        let (gnode, _wc) = staging.take_highest_priority().unwrap();
        assert_eq!(gnode, right); // highest volume
        assert_eq!(staging.warming_count(), 1); // left remains
        assert_eq!(staging.in_flight_count(), 1);
        assert!(staging.contains(right)); // still logically present
    }

    /// Equal volumes resolve to the shallower cell rather than the deeper one.
    /// A tie is the ordinary case for a pair of siblings the moment they are
    /// created, and the rule the queue exists to serve is that a busy ancestor
    /// is warmed before the cells beneath it. Identifiers are handed out as the
    /// tree grows downward, so the larger of two is always the newer and deeper
    /// cell, and resolving a tie toward it inverts the rule exactly. This path
    /// and the synchronous drain are two ways of serving one queue, so they
    /// must not disagree about which cell comes next.
    ///
    /// ´claim:staging:equal-volumes-resolve-to-the-shallower-cell-so-both-drains-agree´
    /// ´test:unit:equal-volumes-take-the-shallower-cell-first´
    #[test]
    fn equal_volumes_take_the_shallower_cell_first() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, right) = split_graph();
        let (earlier, later) = if left < right { (left, right) } else { (right, left) };

        staging.enqueue(left, make_cell(1), 3);
        staging.enqueue(right, make_cell(1), 3);
        staging.warming.get_mut(&left).unwrap().volume = 42.0;
        staging.warming.get_mut(&right).unwrap().volume = 42.0;

        let (gnode, _wc) = staging.take_highest_priority().unwrap();
        assert_eq!(gnode, earlier, "a tie goes to the earlier, shallower identifier");
        assert_ne!(gnode, later);
    }

    /// A cell joining the queue carries its volume with it instead of waiting
    /// for a later pass to supply one. The queue is served highest volume
    /// first, so a cell admitted at zero is indistinguishable from a cell with
    /// no traffic behind it, and a field of zeroes is decided entirely by the
    /// tie-break — which puts the newest and deepest cell first, the inverse of
    /// what the queue is for. Refreshing the cached volumes before the new
    /// cells are added rather than after leaves every one of them in exactly
    /// that state until some later pass happens to refresh again.
    ///
    /// ´claim:staging:a-cell-joins-the-queue-carrying-its-volume-rather-than-a-zero´
    /// ´test:unit:a-newly-queued-cell-carries-its-volume´
    #[test]
    fn a_newly_queued_cell_carries_its_volume() {
        use crate::config::NoiseSchedule;
        use crate::sentinel::SpectralSentinel;

        // Depth zero takes no rounds, so construction is immediate; every
        // deeper cell takes a long schedule, so cells linger in the queue
        // while the background thread works through them.
        let cfg = SentinelConfig::<u64> {
            split_threshold: 5,
            noise_schedule: NoiseSchedule::Explicit(vec![0, 500]),
            noise_batch_size: 2,
            background_warming: true,
            ..SentinelConfig::<u64>::default()
        };
        let mut s: SpectralSentinel<u128, u64, 128> = SpectralSentinel::new(cfg).unwrap();

        let batch: Vec<u128> = (0..40u128).map(|i| (0xA_u128 << 124) | i).collect();
        for _ in 0..10 {
            s.ingest(&batch);
        }

        let staged = s.staging.lock().expect("staging mutex poisoned");
        assert!(
            !staged.warming.is_empty(),
            "the long schedule must leave cells waiting in the queue"
        );

        for (&gnode, wc) in &staged.warming {
            let node_volume = s.graph.gnode_info(gnode).map_or(0.0, |info| info.sum.to_f64_approx());
            if node_volume > 0.0 {
                assert!(
                    wc.volume > 0.0,
                    "a queued cell whose node carries traffic must carry it into the queue too"
                );
            }
        }
    }

    /// A cell handed back unfinished rejoins the warming set and stops being
    /// in flight, with its accumulated rounds intact. Background warming can
    /// therefore be interrupted between rounds — the thread need not carry a
    /// cell to completion once it has taken it.
    ///
    /// ´claim:staging:a-cell-handed-back-unfinished-rejoins-the-warming-set-with-its-progress-intact´
    /// ´test:unit:return-warming-restores-cell´
    #[test]
    fn return_warming_restores_cell() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, _) = split_graph();

        staging.enqueue(left, make_cell(1), 3);
        let (gnode, wc) = staging.take_highest_priority().unwrap();
        assert_eq!(staging.warming_count(), 0);
        assert_eq!(staging.in_flight_count(), 1);

        staging.return_warming(gnode, wc);
        assert_eq!(staging.warming_count(), 1);
        assert_eq!(staging.in_flight_count(), 0);
    }

    /// A cell handed back finished joins the ready queue instead of the
    /// warming set, and is no longer in flight. Which of the two return
    /// paths the background thread takes is what decides the cell's fate, so
    /// completion is declared by the worker that did the rounds rather than
    /// re-derived by the staging area.
    ///
    /// ´claim:staging:a-cell-handed-back-finished-joins-the-ready-queue-rather-than-the-warming-set´
    /// ´test:unit:finish-warming-moves-to-ready´
    #[test]
    fn finish_warming_moves_to_ready() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, _) = split_graph();

        staging.enqueue(left, make_cell(1), 1);
        let (gnode, wc) = staging.take_highest_priority().unwrap();

        staging.finish_warming(gnode, wc.cell);
        assert_eq!(staging.ready_count(), 1);
        assert_eq!(staging.in_flight_count(), 0);
    }

    /// A cell evicted while a background thread was working on it is
    /// discarded when it comes back, not resurrected: the eviction sweep
    /// removes its in-flight mark, and a return with no mark to clear keeps
    /// nothing. The warming work already spent is lost, which is the
    /// deliberate trade — a cell that has left the analysis set must not
    /// reappear in it because a thread happened to be holding it.
    ///
    /// ´claim:staging:a-cell-evicted-while-in-flight-is-discarded-on-return-rather-than-resurrected´
    /// ´test:unit:eviction-of-in-flight-cell-discards-on-return´
    #[test]
    fn eviction_of_in_flight_cell_discards_on_return() {
        let mut staging = StagingArea::<u128>::new();
        let (_, _, left, right) = split_graph();

        staging.enqueue(left, make_cell(1), 3);
        staging.enqueue(right, make_cell(1), 3);

        // Set left to highest priority so take_highest_priority picks it.
        staging.warming.get_mut(&left).unwrap().volume = 100.0;
        staging.warming.get_mut(&right).unwrap().volume = 10.0;

        // Take left for background warming.
        let (gnode, wc) = staging.take_highest_priority().unwrap();
        assert_eq!(gnode, left);

        // Meanwhile, evict left from the analysis set.
        let mut keep = BTreeSet::new();
        keep.insert(right);
        staging.retain_in_set(&keep);

        // Background thread tries to return the evicted cell — silently discarded.
        staging.return_warming(gnode, wc);
        assert_eq!(staging.warming_count(), 1); // only right
        assert_eq!(staging.in_flight_count(), 0);
        assert!(!staging.contains(gnode)); // left is gone
    }

    /// The enumeration of staged cells spans all three states together —
    /// ready, warming and in-flight — and counts each cell once. It is the
    /// same accounting the presence check gives, offered as a whole set, so
    /// diagnostics and eviction sweeps see exactly the cells the area is
    /// responsible for.
    ///
    /// (´claim:staging:presence-is-answered-across-every-state-a-staged-cell-can-occupy´)
    /// ´test:unit:gnode-set-includes-all-states´
    #[test]
    fn gnode_set_includes_all_states() {
        let mut staging = StagingArea::<u128>::new();
        let (_, root, left, right) = split_graph();

        staging.enqueue(root, make_cell(0), 0); // → ready
        staging.enqueue(left, make_cell(1), 3); // → warming
        staging.enqueue(right, make_cell(1), 3); // → warming

        // Take right to in-flight.
        staging.warming.get_mut(&right).unwrap().volume = 100.0;
        let _taken = staging.take_highest_priority(); // takes right

        let set = staging.gnode_set();
        assert!(set.contains(&root)); // ready
        assert!(set.contains(&left)); // warming
        assert!(set.contains(&right)); // in-flight
        assert_eq!(set.len(), 3);
    }
}
