// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Buffer oscillation gap tests.
//!
//! §IDEA M-11 review identified that `buffer=1` (`D_evict − D_create = 1`)
//! creates a pathological pattern: catalytic splits create entries that,
//! after rebalance contraction deepens them by +2 levels, land beyond
//! `D_evict` and become immediately eviction-eligible.  The cycle
//! repeats on subsequent observations.
//!
//! These tests confirm:
//! - Buffer=1 exhibits measurable create-then-evict oscillation.
//! - Buffer=2 significantly reduces eviction waste.
//! - Buffer=3 further reduces or eliminates oscillation.
//! - **All invariants hold** throughout regardless of buffer value
//!   (oscillation is a performance concern, not a correctness bug).
//!
//! # Test index
//!
//! ## Core comparisons
//!
//! | Test | Focus |
//! |------|-------|
//! | [`hotspot_oscillation_buffer_comparison`] | sustained single-coord hotspot |
//! | [`spread_oscillation_buffer_comparison`] | uniform load across domain |
//! | [`zigzag_oscillation_buffer_comparison`] | adversarial alternating extremes |
//! | [`oscillating_hotspot_buffer_comparison`] | burst-length oscillating hotspot pattern |
//!
//! ## Structural properties
//!
//! | Test | Focus |
//! |------|-------|
//! | [`stabilisation_latency_ordering`] | rounds until eviction stops |
//! | [`invariants_hold_every_round_for_all_buffers`] | per-round invariant sweep |
//! | [`energy_conserved_for_all_buffers`] | root sum equals total observed delta |
//! | [`wider_buffer_retains_more_nodes`] | node count monotone with buffer width |
//! | [`buffer_1_round_trace`] | detailed per-round trace for manual inspection |
//!
//! ## Edge cases
//!
//! | Test | Focus |
//! |------|-------|
//! | [`buffer_1_maximum_pressure`] | minimum valid buffer; maximum eviction pressure |
//! | [`sub_threshold_no_oscillation`] | delta ≤ θ triggers no splits or eviction |
//! | [`random_spray_buffer_comparison`] | stochastic load still shows monotone eviction |
//!
//! ## Worst-case patterns
//!
//! | Test | Focus |
//! |------|-------|
//! | [`worst_case_dueling_hotspots`] | alternating high-intensity coords |
//! | [`worst_case_escalating_adversarial`] | asymmetric intensity growth |
//! | [`worst_case_hotspot_migration`] | abrupt hotspot relocation |
//! | [`worst_case_rotating_hotspots`] | 4-coord round-robin via `interleaved_hotspots` |
//! | [`worst_case_growth_spike_relax`] | spread → spike → relaxation via `burst` |
//! | [`worst_case_skewed_power_law`] | power-law intensity distribution |
//! | [`worst_case_rapid_microbursts`] | very short bursts at many coords |
//! | [`worst_case_intensity_inversion`] | complete V-tree re-ranking |
//! | [`worst_case_invariants_hold_at_buffer_1`] | per-round sweep of all worst-cases |
//!
//! ## Breaking buffer=4
//!
//! | Test | Focus |
//! |------|-------|
//! | [`break_buffer_4_hyper_escalation`] | step ≫ θ adversarial zigzag |
//! | [`break_buffer_4_multi_front_battle`] | 8-coord escalating round-robin |
//! | [`break_buffer_4_burst_inversion`] | rapid dominance inversion |
//! | [`break_buffer_4_staircase`] | progressive new competitor introduction |
//! | [`break_buffer_4_combined_assault`] | layered pattern assault |
//! | [`break_buffer_4_energy_conserved`] | energy conservation under deep config |
//! | [`break_buffer_4_invariants_hold`] | per-round sweep of all breaking patterns |

use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::Plan;
use torrust_mudlark::{Config, GvGraph};

mod support;
use support::init_tracing;

// ── Helpers ─────────────────────────────────────────────────────

/// Per-round statistics for diagnosing oscillation behaviour.
#[derive(Debug, Clone)]
struct RoundStats {
    round: usize,
    /// Node count before `observe()`.
    before: u32,
    /// Node count after `observe()` (splits may have occurred).
    after_observe: u32,
    /// Entries evicted by `check_evictions()`.
    evicted: u32,
    /// Node count after eviction.
    after_evict: u32,
}

impl RoundStats {
    /// A *futile cycle*: split created entries during `observe()`
    /// and `check_evictions()` immediately removed some.
    const fn is_futile_cycle(&self) -> bool {
        self.after_observe > self.before && self.evicted > 0
    }
}

/// Build a config with identical θ and `D_create`, varying only the
/// buffer width.  No budget, so eviction must be triggered manually
/// via `check_evictions()` — this isolates the depth-gap effect from
/// budget dynamics.
const fn buffer_config(buffer: u32) -> Config<u64> {
    Config {
        split_threshold: 5,
        depth_create: 3,
        depth_evict: 3 + buffer,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

/// Apply a [`Plan`] to a fresh graph, calling `check_evictions()`
/// after every `observe()` and collecting per-round oscillation data.
///
/// This simulates the steady-state pipeline that a budget config
/// would enforce automatically, isolating the depth-gap effect.
fn collect_stats<const N: u32>(buffer: u32, plan: &Plan<u64, u64>) -> (GvGraph<u64, u64, N>, Vec<RoundStats>) {
    collect_stats_cfg::<N>(buffer_config(buffer), plan)
}

/// Like [`collect_stats`] but with an arbitrary config.
fn collect_stats_cfg<const N: u32>(config: Config<u64>, plan: &Plan<u64, u64>) -> (GvGraph<u64, u64, N>, Vec<RoundStats>) {
    let mut g = GvGraph::<u64, u64, N>::new(config);
    let mut stats = Vec::with_capacity(plan.len());

    for (round, &(coord, delta)) in plan.observations.iter().enumerate() {
        let before = g.node_count();
        g.observe(coord, delta);
        let after_observe = g.node_count();
        let evicted = g.check_evictions();
        let after_evict = g.node_count();

        stats.push(RoundStats {
            round,
            before,
            after_observe,
            evicted,
            after_evict,
        });
    }

    (g, stats)
}

/// Apply a [`Plan`] with `check_evictions()` after each round,
/// asserting invariants every step.  Returns the final graph.
fn run_with_eviction_checked<const N: u32>(buffer: u32, plan: &Plan<u64, u64>) -> GvGraph<u64, u64, N> {
    let config = buffer_config(buffer);
    let mut g = GvGraph::<u64, u64, N>::new(config);
    for &(coord, delta) in &plan.observations {
        g.observe(coord, delta);
        g.check_evictions();
        assert_invariants(&g);
    }
    g
}

/// Aggregate stat: total eviction count.
fn total_evictions(stats: &[RoundStats]) -> u32 {
    stats.iter().map(|s| s.evicted).sum()
}

/// Aggregate stat: rounds that contained a futile split-then-evict cycle.
fn futile_cycle_count(stats: &[RoundStats]) -> usize {
    stats.iter().filter(|s| s.is_futile_cycle()).count()
}

/// Round at which the last eviction occurred (0-based).
/// Returns `None` when no evictions happened.
fn last_eviction_round(stats: &[RoundStats]) -> Option<usize> {
    stats.iter().rev().find(|s| s.evicted > 0).map(|s| s.round)
}

/// Print a summary table via `tracing::info!` for post-mortem analysis
/// (visible with `RUST_LOG=info cargo test -- --nocapture`).
fn log_summary(label: &str, stats: &[RoundStats]) {
    let evictions = total_evictions(stats);
    let futile = futile_cycle_count(stats);
    let last = last_eviction_round(stats);
    let peak = stats.iter().map(|s| s.after_observe).max().unwrap_or(0);
    let final_count = stats.last().map_or(0, |s| s.after_evict);

    tracing::info!(
        label,
        evictions,
        futile,
        last_eviction_round = ?last,
        peak_node_count = peak,
        final_node_count = final_count,
    );
}

/// Record result stats into the current span's `Empty` fields.
///
/// The caller must have created a span with matching `Empty` field
/// names (see [`result_span!`]).  When the span closes, all recorded
/// fields appear in the standard tracing-subscriber output together
/// with `time.busy` / `time.idle`.
fn record_stats(span: &tracing::Span, stats: &[RoundStats]) {
    span.record("rounds", stats.len());
    span.record("evictions", total_evictions(stats));
    span.record("futile", futile_cycle_count(stats));
    span.record("peak_nodes", stats.iter().map(|s| s.after_observe).max().unwrap_or(0));
    span.record("final_nodes", stats.last().map_or(0, |s| s.after_evict));
    span.record("last_eviction", format!("{:?}", last_eviction_round(stats)).as_str());
}

/// Create a debug-level span with `Empty` result fields, suitable
/// for [`record_stats`].  The span name is `"buffer_run"`; the
/// `pattern` and `buffer` fields are recorded at creation.
macro_rules! result_span {
    ($pattern:expr_2021, $buffer:expr_2021) => {
        tracing::debug_span!(
            "buffer_run",
            pattern = $pattern,
            buffer = $buffer,
            rounds = tracing::field::Empty,
            evictions = tracing::field::Empty,
            futile = tracing::field::Empty,
            peak_nodes = tracing::field::Empty,
            final_nodes = tracing::field::Empty,
            last_eviction = tracing::field::Empty,
        )
    };
}

// ── Hotspot: concentrated load at a single coordinate ───────────

/// A sustained hotspot is the simplest trigger for the oscillation gap.
/// Repeated observations at the same coordinate cascade splits downward
/// until the depth gate blocks them.  With buffer=1 the freshly created
/// entries land beyond `D_evict` and are immediately evicted, only to be
/// re-created on the next observation.
#[test]
fn hotspot_oscillation_buffer_comparison() {
    let _t = init_tracing();

    // 80 observations at coord 42, each with delta=6 (above θ=5).
    let plan = Plan::new().hotspot(42, 6, 80);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    // Invariants must hold for every buffer width.
    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("buffer=1", &s1);
    log_summary("buffer=2", &s2);
    log_summary("buffer=3", &s3);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    // Core property: eviction waste should be non-increasing as
    // buffer grows.  Larger buffer ⟹ entries land farther from the
    // eviction frontier, so fewer become eligible.
    assert!(e1 >= e2, "buffer=1 evictions ({e1}) should be ≥ buffer=2 ({e2})");
    assert!(e2 >= e3, "buffer=2 evictions ({e2}) should be ≥ buffer=3 ({e3})");

    // Buffer=1 and buffer=3 should be distinguishable.
    // (Buffer=1 evicts more total entries over the same workload.)
    assert!(
        e1 > e3 || (e1 == 0 && e3 == 0),
        "buffer=1 should exhibit strictly more evictions than buffer=3 \
         unless the workload triggers zero evictions for both: e1={e1}, e3={e3}"
    );
}

// ── Spread: uniform load across the domain ──────────────────────

/// A `spread` workload distributes observations uniformly, growing the
/// tree breadth-first.  As the V-Tree deepens, buffer=1 starts
/// losing entries to eviction while buffer≥2 retains them.
#[test]
fn spread_oscillation_buffer_comparison() {
    let _t = init_tracing();

    // Uniform observations cycling over [0, 256), delta=6.
    let plan = Plan::new().spread(256, 6, 300);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("spread buf=1", &s1);
    log_summary("spread buf=2", &s2);
    log_summary("spread buf=3", &s3);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    assert!(e1 >= e2, "spread: buffer=1 evictions ({e1}) ≥ buffer=2 ({e2})");
    assert!(e2 >= e3, "spread: buffer=2 evictions ({e2}) ≥ buffer=3 ({e3})");
}

// ── Adversarial zigzag: alternating ends of the domain ──────────

/// Zigzag workloads force the V-Tree to repeatedly re-rank entries,
/// maximising the chance that rebalance contraction deepens entries
/// past `D_evict`.
#[test]
fn zigzag_oscillation_buffer_comparison() {
    let _t = init_tracing();

    // Alternate between coords 0 and 255 with delta=6.
    let plan = Plan::new().zigzag(0, 255, 6, 120);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("zigzag buf=1", &s1);
    log_summary("zigzag buf=2", &s2);
    log_summary("zigzag buf=3", &s3);

    let e1 = total_evictions(&s1);
    let e3 = total_evictions(&s3);

    // Under adversarial load, buffer=1 should still evict at least
    // as much as buffer=3.
    assert!(e1 >= e3, "zigzag: buffer=1 evictions ({e1}) ≥ buffer=3 ({e3})");
}

// ── Oscillating hotspot: burst-length alternation ───────────────

/// [`Plan::oscillating_hotspot`] is the canonical pattern for buffer
/// oscillation: every `burst` observations the target switches between
/// two coordinates.  The burst length controls how deep each side
/// builds before abandonment.
#[test]
fn oscillating_hotspot_buffer_comparison() {
    let _t = init_tracing();

    // Burst=10: build 10 obs at coord 0, then 10 at coord 255, for 12 cycles.
    let plan = Plan::new().oscillating_hotspot(0, 255, 8, 10, 12);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("osc buf=1", &s1);
    log_summary("osc buf=2", &s2);
    log_summary("osc buf=3", &s3);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    assert!(e1 >= e2, "osc: buffer=1 evictions ({e1}) ≥ buffer=2 ({e2})");
    assert!(e2 >= e3, "osc: buffer=2 evictions ({e2}) ≥ buffer=3 ({e3})");

    // Oscillating hotspot is the textbook futile-cycle trigger.
    let futile = futile_cycle_count(&s1);
    tracing::info!(futile, "oscillating hotspot futile cycles at buffer=1");
}

// ── Stabilisation latency ───────────────────────────────────────

/// Measures how long (in observation rounds) before the graph stops
/// oscillating.  Buffer=1 should stabilise later (or never, for a
/// sustained workload) compared to buffer≥2.
#[test]
fn stabilisation_latency_ordering() {
    let _t = init_tracing();

    // Fixed-length burst followed by zero-delta silence.
    let plan = Plan::new().hotspot(42, 6, 40).observe_n(42, 0, 20);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("stab buf=1", &s1);
    log_summary("stab buf=2", &s2);
    log_summary("stab buf=3", &s3);

    let last1 = last_eviction_round(&s1);
    let last3 = last_eviction_round(&s3);

    tracing::info!(?last1, ?last3, "stabilisation comparison");

    // If both experienced eviction, buffer=1 should stabilise no
    // earlier than buffer=3.
    if let (Some(l1), Some(l3)) = (last1, last3) {
        assert!(
            l1 >= l3,
            "buffer=1 should stabilise no earlier than buffer=3: last1={l1}, last3={l3}"
        );
    }
}

// ── Invariants always hold ──────────────────────────────────────

/// Regardless of oscillation, invariants must hold after every single
/// observation.  This uses a tight loop with per-round checks.
#[test]
fn invariants_hold_every_round_for_all_buffers() {
    let _t = init_tracing();

    let plan = Plan::new().hotspot(42, 6, 50);

    for buffer in 1..=3 {
        run_with_eviction_checked::<8>(buffer, &plan);
    }
}

// ── Energy conservation ─────────────────────────────────────────

/// Eviction absorbs energy into the parent (ADR-M-014), so the root
/// sum must equal the total observed delta regardless of buffer width.
#[test]
fn energy_conserved_for_all_buffers() {
    let _t = init_tracing();

    let plan = Plan::new().hotspot(42, 6, 60);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();

    for buffer in 1..=3 {
        let (g, _) = collect_stats::<8>(buffer, &plan);
        let root_sum = g.total_sum();
        assert_eq!(
            root_sum, total,
            "energy conservation violated for buffer={buffer}: root_sum={root_sum}, expected={total}"
        );
        assert_invariants(&g);
    }
}

// ── Node-count growth comparison ────────────────────────────────

/// With more buffer, the tree retains more entries because fewer are
/// eviction-eligible.  Final node count should be non-decreasing
/// with buffer width (same workload, same θ, same `D_create`).
#[test]
fn wider_buffer_retains_more_nodes() {
    let _t = init_tracing();

    let plan = Plan::new().spread(256, 6, 200);

    let (g1, _) = collect_stats::<8>(1, &plan);
    let (g2, _) = collect_stats::<8>(2, &plan);
    let (g3, _) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    let n1 = g1.node_count();
    let n2 = g2.node_count();
    let n3 = g3.node_count();

    tracing::info!(n1, n2, n3, "final node counts by buffer width");

    assert!(n1 <= n2, "buffer=1 should retain ≤ buffer=2 nodes: {n1} vs {n2}");
    assert!(n2 <= n3, "buffer=2 should retain ≤ buffer=3 nodes: {n2} vs {n3}");
}

// ── Per-round trace for manual inspection ───────────────────────

/// Detailed per-round trace of buffer=1 behaviour, useful for manual
/// inspection with `RUST_LOG=info cargo test buffer_1_round_trace -- --nocapture`.
#[test]
fn buffer_1_round_trace() {
    let _t = init_tracing();

    let plan = Plan::new().hotspot(42, 6, 30);
    let (g, stats) = collect_stats::<8>(1, &plan);

    for s in &stats {
        tracing::info!(
            round = s.round,
            before = s.before,
            after_observe = s.after_observe,
            evicted = s.evicted,
            after_evict = s.after_evict,
            futile = s.is_futile_cycle(),
        );
    }

    assert_invariants(&g);
}

// ═══════════════════════════════════════════════════════════════
//  Edge cases
// ═══════════════════════════════════════════════════════════════

/// Buffer=1 is the minimum valid buffer (`D_create < D_evict` is
/// required).  It represents maximum eviction pressure: entries
/// created at the depth gate are only one level away from `D_evict`.
/// Invariants must still hold, and eviction count must be ≥ any
/// wider buffer.
#[test]
fn buffer_1_maximum_pressure() {
    let _t = init_tracing();

    let plan = Plan::new().hotspot(42, 6, 60);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (_, s2) = collect_stats::<8>(2, &plan);
    let (_, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    log_summary("buf=1 pressure", &s1);

    assert!(e1 >= e2, "buffer=1 evictions ({e1}) should be ≥ buffer=2 ({e2})");
    assert!(e1 >= e3, "buffer=1 evictions ({e1}) should be ≥ buffer=3 ({e3})");

    // Energy conservation.
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g1.total_sum(), total, "energy conservation violated at buffer=1");
}

/// Sub-threshold observations (accumulated energy < θ at every node)
/// never trigger splits, so the tree stays at the root.  Buffer width
/// should have no effect: zero evictions for all buffers.
#[test]
fn sub_threshold_no_oscillation() {
    let _t = init_tracing();

    // 4 observations of delta=1 at the same coord → root sum=4 < θ=5.
    // No node ever exceeds threshold, so no splits occur.
    let plan = Plan::new().observe_n(42, 1, 4);

    for buffer in 1..=3 {
        let (g, stats) = collect_stats::<8>(buffer, &plan);
        let evictions = total_evictions(&stats);
        let futile = futile_cycle_count(&stats);
        assert_invariants(&g);
        assert_eq!(
            evictions, 0,
            "sub-threshold: buffer={buffer} should have 0 evictions, got {evictions}"
        );
        assert_eq!(
            futile, 0,
            "sub-threshold: buffer={buffer} should have 0 futile cycles, got {futile}"
        );
    }
}

/// Stochastic load via [`Plan::random_spray`] still exhibits the
/// monotone eviction ordering: wider buffer ⟹ fewer evictions.
#[test]
fn random_spray_buffer_comparison() {
    let _t = init_tracing();

    let plan = Plan::new().random_spray(0xDEAD_BEEF, 256, 6, 300);

    let (g1, s1) = collect_stats::<8>(1, &plan);
    let (g2, s2) = collect_stats::<8>(2, &plan);
    let (g3, s3) = collect_stats::<8>(3, &plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary("spray buf=1", &s1);
    log_summary("spray buf=2", &s2);
    log_summary("spray buf=3", &s3);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    assert!(e1 >= e2, "spray: buffer=1 evictions ({e1}) ≥ buffer=2 ({e2})");
    assert!(e2 >= e3, "spray: buffer=2 evictions ({e2}) ≥ buffer=3 ({e3})");

    // Energy conservation.
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    assert_eq!(g1.total_sum(), total, "spray energy conservation at buffer=1");
}

// ═══════════════════════════════════════════════════════════════
//  Worst-case patterns
// ═══════════════════════════════════════════════════════════════

/// Run a three-buffer comparison on an arbitrary plan, asserting
/// monotone eviction ordering and logging diagnostics.
fn compare_buffers(label: &str, plan: &Plan<u64, u64>) {
    let (g1, s1) = collect_stats::<8>(1, plan);
    let (g2, s2) = collect_stats::<8>(2, plan);
    let (g3, s3) = collect_stats::<8>(3, plan);

    assert_invariants(&g1);
    assert_invariants(&g2);
    assert_invariants(&g3);

    log_summary(&format!("{label} buf=1"), &s1);
    log_summary(&format!("{label} buf=2"), &s2);
    log_summary(&format!("{label} buf=3"), &s3);

    let e1 = total_evictions(&s1);
    let e2 = total_evictions(&s2);
    let e3 = total_evictions(&s3);

    assert!(e1 >= e2, "{label}: buffer=1 evictions ({e1}) should be ≥ buffer=2 ({e2})");
    assert!(e2 >= e3, "{label}: buffer=2 evictions ({e2}) should be ≥ buffer=3 ({e3})");

    // Energy conservation.
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();
    for (buf, g) in [(1, &g1), (2, &g2), (3, &g3)] {
        let root_sum = g.total_sum();
        assert_eq!(
            root_sum, total,
            "{label} buf={buf}: energy conservation violated: root_sum={root_sum}, expected={total}"
        );
    }
}

// ── Worst-case 1: Dueling hotspots ──────────────────────────────

/// Two coordinates alternate receiving high-intensity observations.
/// Each round, the winner's entries climb toward the V-root and
/// push the loser's entries deeper via contraction.  With buffer=1,
/// the loser's entries land past `D_evict` and get evicted, only
/// to be re-split on their next observation — the textbook futile
/// cycle.
#[test]
fn worst_case_dueling_hotspots() {
    let _t = init_tracing();

    // 200 rounds alternating between coords 10 and 200,
    // both above θ=5 so each triggers splits.
    let plan = Plan::new().zigzag(10, 200, 8, 200);
    compare_buffers("duel", &plan);

    let (_, s1) = collect_stats::<8>(1, &plan);
    let futile = futile_cycle_count(&s1);
    tracing::info!(futile, "dueling hotspots futile cycles at buffer=1");
}

// ── Worst-case 2: Escalating adversarial zigzag ─────────────────

/// One side gets escalating intensity while the other stays constant.
/// This forces asymmetric 3→2 contraction preprocessing (§IDEA M-10.1),
/// which is the mechanism that adds +2 V-depth — the root cause
/// of the buffer=1 oscillation.
#[test]
fn worst_case_escalating_adversarial() {
    let _t = init_tracing();

    // lo gets base + i*step, hi stays at base.
    // Escalation makes lo's entries dominate, repeatedly
    // contracting hi's subtree deeper.
    let plan = Plan::new().adversarial_zigzag(0, 255, 6, 2, 200);
    compare_buffers("escalate", &plan);
}

// ── Worst-case 3: Hotspot migration ─────────────────────────────

/// Build deep at one location, then abruptly shift to the opposite
/// end.  The old entries are deep in the V-tree; the new hotspot
/// forces rebalance to demote them further.  With buffer=1 the old
/// entries can't survive even a single demotion past `D_evict`.
#[test]
fn worst_case_hotspot_migration() {
    let _t = init_tracing();

    // Phase 1: 50 obs at coord 0 — drives deep splits on the left spine.
    // Phase 2: 50 obs at coord 255 — new hotspot pushes old entries down.
    // Phase 3: 50 obs back at coord 0 — re-creates evicted entries.
    let plan = Plan::new().hotspot(0, 8, 50).hotspot(255, 8, 50).hotspot(0, 8, 50);

    compare_buffers("migrate", &plan);

    let (_, s1) = collect_stats::<8>(1, &plan);
    let (_, s3) = collect_stats::<8>(3, &plan);

    // The migration pattern should amplify the difference between
    // buffer=1 and buffer=3.
    let e1 = total_evictions(&s1);
    let e3 = total_evictions(&s3);
    tracing::info!(e1, e3, "migration eviction gap");
    assert!(e1 > e3, "migration: buffer=1 ({e1}) should strictly exceed buffer=3 ({e3})");
}

// ── Worst-case 4: Rotating hotspots ─────────────────────────────

/// Cycle through 4 hotspot locations.  Each burst promotes one
/// subtree while demoting the other three.  With buffer=1, entries
/// from the previous hotspot get pushed past `D_evict` on every
/// rotation.
///
/// Uses [`Plan::interleaved_hotspots`] — the canonical multi-coord
/// round-robin pattern from the testing infra.
#[test]
fn worst_case_rotating_hotspots() {
    let _t = init_tracing();

    // 4 locations at spacing=64, 15 obs per burst, 5 full rotations = 300 total.
    let plan = Plan::new().interleaved_hotspots(0, 4, 64, 8, 15, 5);

    compare_buffers("rotate", &plan);

    let (_, s1) = collect_stats::<8>(1, &plan);
    let futile = futile_cycle_count(&s1);
    tracing::info!(futile, "rotating hotspots futile cycles at buffer=1");
}

// ── Worst-case 5: Growth → spike → relaxation ───────────────────

/// Fill the tree breadth-first (spread), then hit it with a
/// concentrated hotspot that creates intense V-Tree competition,
/// then dilute with gentle spread.  The spike phase forces deep
/// contraction that pushes spread entries past `D_evict` with
/// buffer=1.
///
/// Uses [`Plan::burst`] for the growth→spike phases, followed by
/// a gentle relaxation spread.
#[test]
fn worst_case_growth_spike_relax() {
    let _t = init_tracing();

    let plan = Plan::new()
        .burst(256, 6, 100, 42, 50, 50) // grow then spike
        .spread(256, 1, 100); // relaxation: gentle background load

    compare_buffers("spike", &plan);
}

// ── Worst-case 6: Skewed power-law ──────────────────────────────

/// Lower coordinates receive disproportionately high intensity,
/// creating a maximally imbalanced V-Tree.  High-intensity entries
/// cluster near the V-root; low-intensity entries in the upper
/// range are pushed deep.  With buffer=1, the low-intensity
/// entries are evicted the moment they land past `D_evict`.
#[test]
fn worst_case_skewed_power_law() {
    let _t = init_tracing();

    let plan = Plan::new().skewed(256, 300);
    compare_buffers("skewed", &plan);
}

// ── Worst-case 7: Rapid-fire micro-bursts ───────────────────────

/// Very short bursts (3 obs) at 8 different locations, cycled
/// rapidly.  Each micro-burst is barely enough to trigger a split,
/// but contraction from the next burst immediately pushes the
/// entries down.  This maximises the ratio of splits to stable
/// entries with buffer=1.
#[test]
fn worst_case_rapid_microbursts() {
    let _t = init_tracing();

    let coords = [0u64, 32, 64, 96, 128, 160, 192, 224];
    let mut plan = Plan::new();
    for _ in 0..20 {
        for &c in &coords {
            // Just 3 obs at delta=8: enough to exceed θ=5 and split.
            plan = plan.observe_n(c, 8, 3);
        }
    }

    compare_buffers("microburst", &plan);
}

// ── Worst-case 8: Intensity inversion ───────────────────────────

/// Build up high intensity at one coord, then "invert" by giving
/// the exact opposite end equally high intensity.  Phase 1 builds
/// a deep left spine; phase 2 challenges it with a deep right spine.
/// The V-Tree must completely re-rank, causing maximum contraction
/// churn.
#[test]
fn worst_case_intensity_inversion() {
    let _t = init_tracing();

    let plan = Plan::new()
        .hotspot(0, 10, 80) // left spine dominates
        .hotspot(255, 10, 80); // right spine inverts the ranking

    compare_buffers("invert", &plan);

    // The inversion should create a clear gap between buffer=1 and 3.
    let (_, s1) = collect_stats::<8>(1, &plan);
    let (_, s3) = collect_stats::<8>(3, &plan);
    let e1 = total_evictions(&s1);
    let e3 = total_evictions(&s3);
    tracing::info!(e1, e3, "inversion eviction gap");
}

// ── Worst-case invariant sweep ──────────────────────────────────

/// Run all worst-case patterns with per-round invariant checking
/// at buffer=1 (the most fragile configuration).
#[test]
fn worst_case_invariants_hold_at_buffer_1() {
    let _t = init_tracing();

    let patterns: Vec<(&str, Plan<u64, u64>)> = vec![
        ("duel", Plan::new().zigzag(10, 200, 8, 100)),
        ("escalate", Plan::new().adversarial_zigzag(0, 255, 6, 2, 100)),
        ("migrate", Plan::new().hotspot(0, 8, 30).hotspot(255, 8, 30).hotspot(0, 8, 30)),
        ("spike", Plan::new().spread(256, 6, 50).hotspot(42, 50, 30)),
        ("skewed", Plan::new().skewed(256, 100)),
        ("invert", Plan::new().hotspot(0, 10, 40).hotspot(255, 10, 40)),
    ];

    for (label, plan) in &patterns {
        tracing::info!(label, obs = plan.len(), "checking invariants every round");
        run_with_eviction_checked::<8>(1, plan);
    }
}

// ═══════════════════════════════════════════════════════════════
//  Breaking buffer=4
// ═══════════════════════════════════════════════════════════════
//
//  The +2 depth increase from a single 3→2 contraction means
//  buffer=3 can already absorb one contraction.  To break
//  buffer=4, we need cascading contractions within a single
//  `observe()` — the violation queue can chain multiple swaps,
//  each deepening an entry further.
//
//  Strategies:
//  1. Extreme intensity escalation (step ≫ θ) — forces violent
//     re-ranking with many queued violations.
//  2. Deeper `D_create` + larger domain — entries start at higher
//     V-depth, so even modest deepening overshoots `D_evict`.
//  3. Multi-front competition — many coords battle for ranking,
//     creating complex V-tree topology with long contraction chains.
//  4. Burst-inversion cycles — rapidly swap who dominates with
//     massive deltas, forcing complete V-tree re-ranking.

/// Config with a deeper starting point: `D_create`=5, buffer=4 → `D_evict`=9.
/// The higher `D_create` means entries are positioned deeper in
/// the V-tree to begin with, so cascading contraction can push
/// them past `D_evict` more easily.
const fn deep_buffer_config(buffer: u32) -> Config<u64> {
    Config {
        split_threshold: 3, // low θ → more frequent splits
        depth_create: 5,
        depth_evict: 5 + buffer,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

/// Run a buffer comparison at 3, 4, and 5 with the deep config,
/// asserting invariants and logging diagnostics.
fn compare_deep_buffers(label: &str, plan: &Plan<u64, u64>) {
    for buffer in 3..=5 {
        let span = result_span!(label, buffer);
        let _enter = span.enter();
        let cfg = deep_buffer_config(buffer);
        let (g, stats) = collect_stats_cfg::<16>(cfg, plan);
        assert_invariants(&g);
        record_stats(&span, &stats);
    }
}

/// Hyper-escalating adversarial: step=10 (2× the split threshold).
/// The enormous intensity gap forces the V-tree into extreme
/// contraction cascades — one side's entries get pushed down many
/// levels in a single `observe()`.
#[test]
fn break_buffer_4_hyper_escalation() {
    let _t = init_tracing();

    // step=10 means lo receives intensity 6 + i*10 while hi
    // stays at 6.  After 50 rounds lo's intensity is ~500
    // while hi's is still 6 — a 100:1 imbalance.
    let plan = Plan::new().adversarial_zigzag(0, 65535, 6, 10, 400);

    for buffer in 3..=5 {
        let span = result_span!("hyper", buffer);
        let _enter = span.enter();
        let cfg = deep_buffer_config(buffer);
        let (g, stats) = collect_stats_cfg::<16>(cfg, &plan);
        assert_invariants(&g);
        record_stats(&span, &stats);
    }

    // Also try with the default shallow config (θ=5, D_create=3).
    // Shallow tree + step=10 creates extreme imbalance;
    // buffer=4 still suffers futile cycles, buffer=5 may too.
    for buffer in 3..=10 {
        let span = result_span!("hyper-shallow", buffer);
        let _enter = span.enter();
        let (g, s) = collect_stats::<16>(buffer, &plan);
        assert_invariants(&g);
        record_stats(&span, &s);
    }
    let (_, s4) = collect_stats::<16>(4, &plan);
    let f4 = futile_cycle_count(&s4);
    assert!(
        f4 > 0,
        "hyper-escalation (shallow) should produce futile cycles at buffer=4: got {f4}"
    );
}

/// Multi-front escalating battle: 8 coords compete, each getting
/// a burst of escalating intensity in round-robin.  The constant
/// re-ranking creates long contraction chains.
///
/// This is the strongest buffer=4 breaker found: the many-front
/// competition creates V-trees deep enough that cascading
/// contraction overshoots even a 4-level buffer.
#[test]
fn break_buffer_4_multi_front_battle() {
    let _t = init_tracing();

    let coords: [u64; 8] = [0, 1024, 2048, 4096, 8192, 16384, 32768, 49152];
    let mut plan = Plan::new();
    for round in 0..30u64 {
        for &c in &coords {
            // Escalating: each round the delta grows.
            let delta = 4 + round * 3;
            plan = plan.observe_n(c, delta, 5);
        }
    }

    let mut all_stats = Vec::new();
    for buffer in 3..=5 {
        let span = result_span!("multi-front", buffer);
        let _enter = span.enter();
        let (g, stats) = collect_stats_cfg::<16>(deep_buffer_config(buffer), &plan);
        assert_invariants(&g);
        record_stats(&span, &stats);
        all_stats.push((buffer, stats));
    }

    let f3 = futile_cycle_count(&all_stats[0].1);
    let f4 = futile_cycle_count(&all_stats[1].1);
    let f5 = futile_cycle_count(&all_stats[2].1);
    assert!(
        f4 > 0,
        "multi-front battle should produce futile cycles at buffer=4: got {f4}"
    );

    // Monotone: wider buffer ⟹ fewer futile cycles.
    assert!(
        f3 >= f4 && f4 >= f5,
        "futile cycles should be non-increasing: buf3={f3}, buf4={f4}, buf5={f5}"
    );
}

/// Burst-inversion cycle: rapidly alternate which end dominates
/// with massive delta swings.  Each inversion forces a complete
/// V-tree re-ranking — the deepening cascades compound.
#[test]
fn break_buffer_4_burst_inversion() {
    let _t = init_tracing();

    let mut plan = Plan::new();
    for i in 0..20u64 {
        let big_delta = 50 + i * 20;
        if i % 2 == 0 {
            plan = plan.observe_n(0, big_delta, 10);
        } else {
            plan = plan.observe_n(65535, big_delta, 10);
        }
    }

    compare_deep_buffers("burst-inv", &plan);
}

/// Adversarial staircase: introduce new competing coords one by
/// one, each with progressively higher intensity.  The late-comers
/// dominate and push all earlier entries deeper.  With enough
/// stairs, even buffer=4 entries get pushed past `D_evict`.
#[test]
fn break_buffer_4_staircase() {
    let _t = init_tracing();

    let mut plan = Plan::new();
    // 12 "stairs" — each new coord dominates the V-tree.
    for step in 0..12u64 {
        let coord = step * 5000;
        let delta = 10 + step * 15;
        plan = plan.observe_n(coord, delta, 20);
    }

    compare_deep_buffers("staircase", &plan);
}

/// Combined assault: spread → adversarial escalation → inversion →
/// micro-burst.  Layers every mechanism that causes contraction
/// churn.
#[test]
fn break_buffer_4_combined_assault() {
    let _t = init_tracing();

    let plan = Plan::new()
        .spread(65536, 4, 200) // fill the tree
        .adversarial_zigzag(0, 65535, 6, 8, 200) // escalating competition
        .hotspot(32768, 100, 50) // spike in the middle
        .adversarial_zigzag(65535, 0, 6, 12, 200); // reverse escalation

    compare_deep_buffers("assault", &plan);
}

/// Invariants hold for all buffer=4 breaking patterns.
#[test]
fn break_buffer_4_invariants_hold() {
    let _t = init_tracing();

    let patterns: Vec<(&str, Plan<u64, u64>)> = vec![
        ("hyper", Plan::new().adversarial_zigzag(0, 65535, 6, 10, 200)),
        ("burst-inv", {
            let mut p = Plan::new();
            for i in 0..10u64 {
                let d = 50 + i * 20;
                p = if i % 2 == 0 {
                    p.observe_n(0, d, 10)
                } else {
                    p.observe_n(65535, d, 10)
                };
            }
            p
        }),
        ("staircase", {
            let mut p = Plan::new();
            for step in 0..8u64 {
                p = p.observe_n(step * 5000, 10 + step * 15, 15);
            }
            p
        }),
        ("multi-front", {
            let coords: [u64; 8] = [0, 1024, 2048, 4096, 8192, 16384, 32768, 49152];
            let mut p = Plan::new();
            for round in 0..15u64 {
                for &c in &coords {
                    p = p.observe_n(c, 4 + round * 3, 5);
                }
            }
            p
        }),
        (
            "assault",
            Plan::new()
                .spread(65536, 4, 100)
                .adversarial_zigzag(0, 65535, 6, 8, 100)
                .hotspot(32768, 100, 30)
                .adversarial_zigzag(65535, 0, 6, 12, 100),
        ),
    ];

    for (label, plan) in &patterns {
        let cfg = deep_buffer_config(4);
        let mut g = GvGraph::<u64, u64, 16>::new(cfg);
        for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
            g.observe(coord, delta);
            g.check_evictions();
            assert_invariants(&g);
            if (i + 1) % 50 == 0 {
                tracing::info!(label, step = i + 1, nodes = g.node_count(), "checkpoint");
            }
        }
    }
}

/// Energy conservation holds under the deep buffer config.
#[test]
fn break_buffer_4_energy_conserved() {
    let _t = init_tracing();

    let plan = Plan::new().adversarial_zigzag(0, 65535, 6, 10, 200).hotspot(32768, 100, 50);
    let total: u64 = plan.observations.iter().map(|&(_, d)| d).sum();

    for buffer in 3..=5 {
        let cfg = deep_buffer_config(buffer);
        let (g, _) = collect_stats_cfg::<16>(cfg, &plan);
        let root_sum = g.total_sum();
        assert_eq!(
            root_sum, total,
            "deep config energy conservation violated for buffer={buffer}: \
             root_sum={root_sum}, expected={total}"
        );
        assert_invariants(&g);
    }
}
