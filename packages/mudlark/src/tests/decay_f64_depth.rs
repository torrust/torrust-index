// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Focused diagnostic test for f64 plateau depth inconsistency.
//!
//! ## Root cause hypothesis
//!
//! `gnode_depth_from_interval(lo, hi, N)` computes:
//!
//!     depth = N - (width.log2() as u32)
//!
//! For f64 intervals narrower than 1.0, `log2(width)` is negative.
//! The saturating cast `(-1.0_f64 as u32)` yields `0`, so `depth`
//! is capped at `N` regardless of how narrow the interval is.
//!
//! This means a parent `[2, 3)` (width=1, depth=N) and its child
//! `[2, 2.5)` (width=0.5, depth=N) compute the **same depth**.
//! When the split creates G-children at `child_depth` = N, the
//! parent (now Internal) contributes `g_depth + 1 = N + 1` to the
//! plateau, while the child terminals contribute `N`. The plateau
//! gets `depth = max(N+1, N) = N+1`, and the depth consistency
//! invariant fires because some basis elements contribute depth N
//! while the plateau says N+1.
//!
//! The original error (surfaced via the `decay_f64_tree` unit test):
//!
//!   Plateau depth: key BasisEdge(2.0), basis element `GNodeId(11)` (Terminal):
//!     element contributes depth 4, plateau.depth=5
//!
//! Run with:
//!   `cargo test --test decay_f64_depth -- --nocapture`

use crate::graph::GvGraph;
use crate::gtree::gnode_depth_from_interval;
use crate::invariants::{check_all_invariants, dump_gtree, dump_plateaus};
use crate::testing::{Plan, f64_deep_config, f64_default_config};
use crate::tests::init_tracing;
use crate::traits::Coordinate;

// ── Test 1: demonstrate the depth computation saturation ────────

/// Show that `gnode_depth_from_interval` saturates at `N` for f64
/// intervals narrower than 1.0.
#[test]
fn f64_depth_saturates_at_n() {
    let _t = init_tracing();

    tracing::debug!("gnode_depth_from_interval saturation for f64, N=4");
    tracing::debug!("domain = [0, 16)  domain_max = 2^4 = 16.0");

    let cases: &[(f64, f64, &str)] = &[
        (0.0, 16.0, "root"),
        (0.0, 8.0, "depth-1 left"),
        (0.0, 4.0, "depth-2"),
        (2.0, 4.0, "depth-3 (width=2)"),
        (2.0, 3.0, "depth-4 (width=1)"),
        (2.0, 2.5, "CHILD (width=0.5) ← should be depth 5"),
        (2.5, 3.0, "CHILD (width=0.5) ← should be depth 5"),
        (2.0, 2.25, "width=0.25 ← should be depth 6"),
    ];

    for &(lo, hi, label) in cases {
        let width = hi - lo;
        let log2_raw = width.log2();
        #[allow(clippy::cast_possible_truncation, clippy::cast_sign_loss)]
        let log2_as_u32 = log2_raw as u32;
        let depth = gnode_depth_from_interval(lo, hi, 4);
        let saturated = if log2_raw < 0.0 { " ◀◀◀ SATURATED" } else { "" };
        tracing::debug!(
            "  [{lo:>6.2}, {hi:>6.2})  w={width:<6.2}  log2(w)={log2_raw:>5.1}  \
             as_u32={log2_as_u32}  depth={depth}  {label}{saturated}",
        );
    }

    // Verify the saturation.
    let depth_parent = gnode_depth_from_interval(2.0_f64, 3.0_f64, 4);
    let depth_child = gnode_depth_from_interval(2.0_f64, 2.5_f64, 4);
    tracing::debug!("Parent [2.0, 3.0) depth = {depth_parent}");
    tracing::debug!("Child  [2.0, 2.5) depth = {depth_child}");
    tracing::debug!(
        "EQUAL? {} ← both saturate to N=4 despite being different tree levels",
        depth_parent == depth_child,
    );

    // Show the knock-on effect on plateau depth.
    tracing::debug!("After split: parent becomes Internal at g_depth={depth_parent}");
    tracing::debug!("  → Internal contributes depth {}", depth_parent + 1);
    tracing::debug!(
        "  → recompute_plateau sees max({}, {}) = {}",
        depth_parent + 1,
        depth_child,
        depth_parent + 1
    );
    tracing::debug!("  → but child Terminal contributes depth {depth_child}");
    tracing::debug!(
        "  → INVARIANT VIOLATION: child contributes {} ≠ plateau.depth {}",
        depth_child,
        depth_parent + 1,
    );
}

// ── Test 2: replay f64 tree, check invariants per observation ───

/// Replay the exact f64 tree construction step-by-step. Checks
/// plateau invariants after **every** observation. On first failure,
/// dumps full G-tree + plateau state with basis membership using
/// the library's `dump_gtree` and `dump_plateaus` helpers.
#[test]
fn trace_f64_depth_inconsistency_step_by_step() {
    let _t = init_tracing();

    let config = f64_default_config();
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(config);

    // 10 rounds of alternating observations at coords 2.0 and 10.0.
    let plan: Plan<f64, f64> = (0..10).fold(Plan::new(), |p, _| p.observe(2.0, 10.0).observe(10.0, 5.0));

    let mut nc_prev = g.node_count();

    for (step, &(coord, delta)) in plan.observations.iter().enumerate() {
        let step1 = step + 1; // 1-based
        g.observe(coord, delta);
        let nc = g.node_count();

        // Log structural changes.
        if nc != nc_prev {
            tracing::debug!("Step {step1}: observe({coord}, {delta}) — nodes {nc_prev}→{nc}");

            // Flag sub-unit intervals (the symptom).
            for (i, gn) in g.gnodes().iter_occupied() {
                let lo_f = Coordinate::to_f64(gn.lo);
                let hi_f = Coordinate::to_f64(gn.hi);
                let w = hi_f - lo_f;
                if w < 1.0 {
                    let d = gnode_depth_from_interval(gn.lo, gn.hi, 4);
                    tracing::debug!(
                        "  ⚠ Sub-unit interval: G({i}) [{:.3}, {:.3}) w={w:.4} \
                         → depth saturates to {d} (true depth ≈ {:.0})",
                        lo_f,
                        hi_f,
                        4.0 - w.log2(),
                    );
                }
            }
        }

        // Soft check: collect errors without panicking.
        let errs = check_all_invariants(&g);
        if !errs.is_empty() {
            tracing::debug!("INVARIANT VIOLATION at step {step1}");
            for e in &errs {
                tracing::debug!("  ✗ {e}");
            }
            tracing::debug!("\n{}", dump_gtree(&g));
            tracing::debug!("{}", dump_plateaus(&g));

            // Extra: show depth contribution per basis element.
            tracing::debug!("── Depth contribution analysis ──");
            for (key, elems) in g.debug_plateau_basis() {
                let plateaus = g.plateaus();
                let p = &plateaus[&key];
                for &(idx, lo, hi, state, g_depth) in &elems {
                    let contributes = if state == "Internal" { g_depth + 1 } else { g_depth };
                    let mismatch = if contributes == p.depth { "" } else { " ◀◀◀ MISMATCH" };
                    let lo_f = Coordinate::to_f64(lo);
                    let hi_f = Coordinate::to_f64(hi);
                    let w = hi_f - lo_f;
                    tracing::debug!(
                        "  {key:?} → G({idx}) {state} [{:.2}, {:.2}) w={w:.3} \
                         g_depth={g_depth} contributes={contributes} plateau.depth={}{mismatch}",
                        lo_f,
                        hi_f,
                        p.depth,
                    );
                }
            }

            panic!("Invariant violation at step {step1}. See stderr for full diagnosis.");
        }

        nc_prev = nc;
    }

    tracing::debug!("All 20 observations passed with invariants.");
    tracing::debug!("{}", dump_plateaus(&g));

    // Now decay — this was the original trigger.
    tracing::debug!("Calling decay(g_root, 0.5, 0.0) ...");
    g.decay(g.g_root(), 0.5, 0.0);
    tracing::debug!("{}", dump_plateaus(&g));

    let post_decay_errs = check_all_invariants(&g);
    assert!(
        post_decay_errs.is_empty(),
        "Post-decay invariant violations: {post_decay_errs:?}"
    );
}

// ── Test 3: minimal reproduction ────────────────────────────────

/// Minimal f64 tree that triggers the depth saturation bug.
/// Feeds observations exclusively at coord=2.0 to force repeated
/// splits into sub-unit-width intervals where `gnode_depth_from_interval`
/// saturates.
#[test]
fn minimal_f64_depth_saturation() {
    let _t = init_tracing();

    let config = f64_deep_config();
    let mut g: GvGraph<f64, f64, 4> = GvGraph::new(config);

    // 100 observations at a single hotspot to force deep splits.
    let plan: Plan<f64, f64> = Plan::new().hotspot(2.0, 10.0, 100);

    tracing::debug!("Minimal f64 depth saturation reproduction");
    let mut nc_prev = g.node_count();

    for (i, &(coord, delta)) in plan.observations.iter().enumerate() {
        let step = i + 1; // 1-based
        g.observe(coord, delta);
        let nc = g.node_count();

        if nc != nc_prev {
            tracing::debug!("observation {step}: nodes {nc_prev}→{nc}");

            // Flag sub-unit intervals.
            for (idx, gn) in g.gnodes().iter_occupied() {
                let lo_f = Coordinate::to_f64(gn.lo);
                let hi_f = Coordinate::to_f64(gn.hi);
                let w = hi_f - lo_f;
                if w < 1.0 {
                    let d = gnode_depth_from_interval(gn.lo, gn.hi, 4);
                    tracing::debug!(
                        "  ⚠ G({idx}) [{:.4}, {:.4}) w={w:.6} depth={d} true_depth≈{:.1}",
                        lo_f,
                        hi_f,
                        4.0 - w.log2(),
                    );
                }
            }

            let errs = check_all_invariants(&g);
            if !errs.is_empty() {
                tracing::debug!("BUG REPRODUCED at observation {step}");
                for e in &errs {
                    tracing::debug!("  ✗ {e}");
                }
                tracing::debug!("\n{}", dump_gtree(&g));
                tracing::debug!("{}", dump_plateaus(&g));
                tracing::debug!(
                    "Root cause: f64 interval width < 1.0 → log2(w) negative → \
                     `as u32` saturates to 0 → depth capped at N={}, \
                     but Internal parent contributes N+1={}.",
                    4,
                    5,
                );
                panic!("Depth saturation bug reproduced at observation {step}.");
            }

            nc_prev = nc;
        }
    }

    tracing::debug!("All 100 observations passed (no sub-unit splits triggered).");
}
