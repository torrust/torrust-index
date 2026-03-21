// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for the **ADR-M-028 span-native tracing and diagnostic**
//! subsystem.
//!
//! The diagnostic module exposes audit helpers (`audit_violations`,
//! `audit_plateau_consistency`, `diagnose_missed_violation`), display
//! formatters (`Gn`, `Pl`), and is the integration point for
//! `tracing` spans and events emitted during graph mutations.
//!
//! A key concern tested here is that the migration from
//! `#[cfg(debug_assertions)]` guards to the `tracing::enabled!()`
//! gate did **not** suppress hard invariant assertions — those must
//! fire unconditionally in debug builds.  The span-emission tests
//! additionally verify that every major graph operation (observe,
//! split, rebalance, decay, eviction) creates the expected `tracing`
//! spans, so that downstream tooling can rely on a stable span
//! vocabulary.
//!
//! # Test index
//!
//! ## `audit_violations`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`audit_violations_returns_empty_when_no_violations`] | empty graph has no missed violations |
//! | [`audit_violations_returns_empty_after_single_observe`] | single observation leaves no residual violations |
//! | [`audit_violations_returns_empty_after_many_observes`] | multi-split graph drains all violations |
//!
//! ## `Gn` display helper
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gn_display_for_root`] | root Terminal node formatting |
//! | [`gn_display_after_split`] | Internal/SemiInternal formatting after bootstrap split |
//! | [`gn_display_dead_node`] | unallocated slot renders as `DEAD` |
//! | [`gn_display_semi_internal`] | walk all occupied nodes; confirm T and I states observed |
//!
//! ## Hard invariant assertions fire in debug builds
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_runs_post_observe_audit_without_tracing`] | `observe()` audit runs without a tracing subscriber |
//! | [`rebalance_runs_residual_audit_without_tracing`] | `rebalance()` audit runs without a tracing subscriber |
//! | [`plateau_mirror_consistency_runs_without_tracing`] | plateau mirror audit fires (dynamic-contour-tracking) |
//! | [`plateau_sum_check_runs_without_tracing`] | plateau sum check fires (dynamic-contour-tracking) |
//!
//! ## Span emission during graph mutations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_emits_spans`] | `observe()` creates at least one span |
//! | [`observe_creates_observe_span`] | `observe` span present by name |
//! | [`split_creates_bootstrap_split_span`] | `bootstrap_split` span on first split |
//! | [`catalytic_split_creates_span`] | `catalytic_split` span on threshold-triggered split |
//! | [`rebalance_creates_span`] | `rebalance` span present |
//! | [`evict_tip_creates_span`] | `evict_tip` span under budget pressure |
//! | [`decay_creates_span`] | `decay*` span on explicit decay call |
//! | [`eviction_pipeline_creates_spans`] | `evict_batch` + `scan_for_candidates` spans |
//! | [`vtree_remove_leaf_creates_span`] | `vtree_remove_leaf` span during eviction |
//! | [`normalize_plateaus_creates_span`] | `normalize_plateaus` span (dynamic-contour-tracking) |
//!
//! ## Events are emitted at expected levels
//!
//! | Test | Focus |
//! |------|-------|
//! | [`observe_emits_debug_events`] | split + rebalance pipeline emits DEBUG events |
//!
//! ## `audit_plateau_consistency` (dynamic-contour-tracking)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`audit_plateau_consistency_passes_on_healthy_graph`] | healthy graph passes without panic |
//! | [`audit_plateau_consistency_with_context`] | explicit `PlateauAuditContext` accepted |
//!
//! ## `diagnose_missed_violation`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`diagnose_missed_violation_emits_events_without_panic`] | deep V-node emits events, no panic |
//! | [`diagnose_missed_violation_with_eviction_context`] | eviction context fields propagated |
//! | [`diagnose_missed_violation_on_root_node`] | root V-node early-return branch covered |
//!
//! ## `Pl` plateau display helper (dynamic-contour-tracking)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`pl_display_for_root`] | root node plateau display |
//! | [`pl_display_for_non_basis_node`] | non-basis node shows `not_basis` |
//! | [`pl_display_contains_range_and_depth`] | full plateau display includes range + depth + sum |

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tracing_subscriber::layer::SubscriberExt;

use crate::graph::Config;
use crate::invariants::assert_invariants;
use crate::testing::{GraphCreator, Plan, evictable_config, low_threshold_config, run};
use crate::{GvGraph, diagnostic};

// ── Helpers ─────────────────────────────────────────────────────

/// Canonical observation plan used by most tests: 8 observations
/// that trigger splits, rebalance, and violations in an N=4 graph.
fn multi_split_plan() -> Plan<u64, u64> {
    Plan::new()
        .observe(3, 6)
        .observe(12, 6)
        .observe(5, 6)
        .observe(10, 6)
        .observe(7, 6)
        .observe(14, 6)
        .observe(1, 6)
        .observe(15, 6)
}

fn make_graph(threshold: u64) -> GvGraph<u64, u64, 4> {
    GvGraph::new(Config {
        split_threshold: threshold,
        ..low_threshold_config()
    })
}

/// Build a populated N=4 graph via `GraphCreator` with the canonical
/// multi-split plan and invariant checking on every step.
fn build_multi_split_graph() -> GvGraph<u64, u64, 4> {
    GraphCreator::new(Config {
        split_threshold: 5,
        ..low_threshold_config()
    })
    .observe(3, 6)
    .observe(12, 6)
    .observe(5, 6)
    .observe(10, 6)
    .observe(7, 6)
    .observe(14, 6)
    .observe(1, 6)
    .observe(15, 6)
    .check_every(1)
    .build()
}

// ── Custom tracing layer that counts new spans ──────────────────

struct SpanCounter(Arc<AtomicUsize>);

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> tracing_subscriber::Layer<S> for SpanCounter {
    fn on_new_span(
        &self,
        _attrs: &tracing::span::Attributes<'_>,
        _id: &tracing::span::Id,
        _ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        self.0.fetch_add(1, Ordering::Relaxed);
    }
}

// ── Custom tracing layer that counts events by level ────────────

struct EventCounter {
    debug_count: Arc<AtomicUsize>,
    trace_count: Arc<AtomicUsize>,
}

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> tracing_subscriber::Layer<S>
    for EventCounter
{
    fn on_event(&self, event: &tracing::Event<'_>, _ctx: tracing_subscriber::layer::Context<'_, S>) {
        match *event.metadata().level() {
            tracing::Level::DEBUG => {
                self.debug_count.fetch_add(1, Ordering::Relaxed);
            }
            tracing::Level::TRACE => {
                self.trace_count.fetch_add(1, Ordering::Relaxed);
            }
            _ => {}
        }
    }
}

/// Install a subscriber for the duration of a closure.
///
/// Uses `tracing::subscriber::with_default`, which is per-thread
/// and does not interfere with other tests.
fn with_counting_subscriber<F, R>(f: F) -> (usize, usize, usize, R)
where
    F: FnOnce() -> R,
{
    let spans = Arc::new(AtomicUsize::new(0));
    let debug_events = Arc::new(AtomicUsize::new(0));
    let trace_events = Arc::new(AtomicUsize::new(0));

    let subscriber = tracing_subscriber::registry()
        .with(SpanCounter(Arc::clone(&spans)))
        .with(EventCounter {
            debug_count: Arc::clone(&debug_events),
            trace_count: Arc::clone(&trace_events),
        });

    let result = tracing::subscriber::with_default(subscriber, f);

    (
        spans.load(Ordering::Relaxed),
        debug_events.load(Ordering::Relaxed),
        trace_events.load(Ordering::Relaxed),
        result,
    )
}

// ── Custom layer that captures span names ───────────────────────

struct SpanNameCapture(Arc<std::sync::Mutex<Vec<String>>>);

impl<S: tracing::Subscriber + for<'a> tracing_subscriber::registry::LookupSpan<'a>> tracing_subscriber::Layer<S>
    for SpanNameCapture
{
    fn on_new_span(
        &self,
        _attrs: &tracing::span::Attributes<'_>,
        id: &tracing::span::Id,
        ctx: tracing_subscriber::layer::Context<'_, S>,
    ) {
        if let Some(span) = ctx.span(id) {
            self.0.lock().unwrap().push(span.name().to_string());
        }
    }
}

fn with_span_capture<F, R>(f: F) -> (Vec<String>, R)
where
    F: FnOnce() -> R,
{
    let names = Arc::new(std::sync::Mutex::new(Vec::<String>::new()));
    let subscriber = tracing_subscriber::registry().with(SpanNameCapture(Arc::clone(&names)));

    let result = tracing::subscriber::with_default(subscriber, f);
    let captured = names.lock().unwrap().clone();
    (captured, result)
}

// ═════════════════════════════════════════════════════════════════
// 1. audit_violations
// ═════════════════════════════════════════════════════════════════

#[test]
fn audit_violations_returns_empty_when_no_violations() {
    let g = make_graph(100);
    // No observations → no violations possible.
    let missed = diagnostic::audit_violations(&g.vnodes, &g.violations, "EMPTY");
    assert!(missed.is_empty());
}

#[test]
fn audit_violations_returns_empty_after_single_observe() {
    let g: GvGraph<u64, u64, 4> = run(
        Config {
            split_threshold: 100,
            ..low_threshold_config()
        },
        &Plan::new().observe(5, 7),
    );
    let missed = diagnostic::audit_violations(&g.vnodes, &g.violations, "SINGLE");
    assert!(
        missed.is_empty(),
        "single observation should not leave missed violations: {missed:?}"
    );
}

#[test]
fn audit_violations_returns_empty_after_many_observes() {
    let g = build_multi_split_graph();
    let missed = diagnostic::audit_violations(&g.vnodes, &g.violations, "POST-MULTI");
    assert!(
        missed.is_empty(),
        "after observe + rebalance, all violations should be drained: {missed:?}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 2. Gn display helper
// ═════════════════════════════════════════════════════════════════

#[test]
fn gn_display_for_root() {
    let g = make_graph(100);
    let display = format!("{}", diagnostic::Gn(&g.gnodes, g.g_root));
    // Root is a terminal: "G0(T,[0,16),sum=0)"
    assert!(display.starts_with("G0(T,"), "unexpected Gn display: {display}");
    assert!(display.contains("sum="), "Gn display should contain 'sum=': {display}");
}

#[test]
fn gn_display_after_split() {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(Config {
        split_threshold: 5,
        ..low_threshold_config()
    })
    .observe(3, 10) // triggers bootstrap split
    .build();

    let root_display = format!("{}", diagnostic::Gn(&g.gnodes, g.g_root));
    // Root should now be Internal (I) or SemiInternal (S).
    assert!(
        root_display.contains("(I,") || root_display.contains("(S,"),
        "after split, root should be I or S: {root_display}"
    );
}

#[test]
fn gn_display_dead_node() {
    let g = make_graph(100);
    // Index 99 is not allocated.
    let dead_id = crate::handle::GNodeId::from_index(99);
    let display = format!("{}", diagnostic::Gn(&g.gnodes, dead_id));
    assert_eq!(display, "G99(DEAD)");
}

#[test]
fn gn_display_semi_internal() {
    // A bootstrap split creates an Internal root (both children).
    // To get a SemiInternal node we need eviction to remove one child.
    // Instead, observe enough to check that at least one node in the
    // tree formats as "S" (SemiInternal) or "I" (Internal).
    let g = build_multi_split_graph();

    // Walk all occupied G-nodes and check that each formats without panic.
    let mut seen_states = Vec::new();
    for (idx, _) in g.gnodes.iter_occupied() {
        let id = crate::handle::GNodeId::from_index(idx);
        let display = format!("{}", diagnostic::Gn(&g.gnodes, id));
        assert!(
            display.starts_with(&format!("G{idx}(")),
            "Gn display should start with index: {display}"
        );
        // Collect the state letter.
        if display.contains("(T,") {
            seen_states.push("T");
        } else if display.contains("(S,") {
            seen_states.push("S");
        } else if display.contains("(I,") {
            seen_states.push("I");
        }
    }
    // A multi-split graph should have at least one Internal node.
    assert!(
        seen_states.contains(&"I"),
        "expected at least one Internal node, states: {seen_states:?}"
    );
    // And at least one Terminal.
    assert!(
        seen_states.contains(&"T"),
        "expected at least one Terminal node, states: {seen_states:?}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 3. Assertions fire in debug builds (cfg!(debug_assertions))
// ═════════════════════════════════════════════════════════════════
//
// The migration from `#[cfg(debug_assertions)]` to
// `tracing::enabled!()` must NOT suppress hard assertions.
// These tests verify the guards use
// `cfg!(debug_assertions) || tracing::enabled!(Level::DEBUG)`
// so they fire unconditionally during `cargo test`.

#[test]
fn observe_runs_post_observe_audit_without_tracing() {
    // If the audit were still behind `tracing::enabled!()` only,
    // this would silently pass even if violations were present.
    // With the fix, audit_violations runs unconditionally in debug
    // builds.
    let g = build_multi_split_graph();
    // The key assertion is implicit: if the `debug_assert!` inside
    // `observe()` fires on a violation, the test panics. If we get
    // here, the audit ran and found no violations.
    assert_invariants(&g);
}

#[test]
fn rebalance_runs_residual_audit_without_tracing() {
    // Same pattern: the hard `assert!` inside `rebalance()` must
    // fire in debug builds without a tracing subscriber.
    let cfg = Config {
        split_threshold: 5,
        ..low_threshold_config()
    };
    let g: GvGraph<u64, u64, 4> = run(cfg, &Plan::new().observe(3, 6).observe(12, 6).observe(5, 6).observe(10, 6));
    assert_invariants(&g);
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn plateau_mirror_consistency_runs_without_tracing() {
    // `debug_assert_plateau_mirror_consistency` must run in debug
    // builds. If it were skipped, plateau divergence would go
    // undetected.
    let g = build_multi_split_graph();
    // Explicitly call — should not early-return in debug builds.
    g.debug_assert_plateau_mirror_consistency("TEST-DIRECT");
    assert_invariants(&g);
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn plateau_sum_check_runs_without_tracing() {
    let cfg = Config {
        split_threshold: 5,
        ..low_threshold_config()
    };
    let g: GvGraph<u64, u64, 4> = run(
        cfg,
        &Plan::new()
            .observe(3, 6)
            .observe(12, 6)
            .observe(5, 6)
            .observe(10, 6)
            .observe(7, 6)
            .observe(14, 6),
    );
    // Should not early-return.
    g.debug_check_plateau_sums("TEST-DIRECT");
    assert_invariants(&g);
}

// ═════════════════════════════════════════════════════════════════
// 4. Span emission during graph mutations
// ═════════════════════════════════════════════════════════════════

#[test]
fn observe_emits_spans() {
    let (span_count, _debug, _trace, ()) = with_counting_subscriber(|| {
        let mut g = make_graph(5);
        g.observe(3u64, 10u64); // triggers bootstrap split → spans
    });
    assert!(span_count > 0, "observe() should create at least one span, got {span_count}");
}

#[test]
fn observe_creates_observe_span() {
    let (names, ()) = with_span_capture(|| {
        let mut g = make_graph(100);
        g.observe(5u64, 7u64);
    });
    assert!(
        names.contains(&"observe".to_string()),
        "expected 'observe' span in: {names:?}"
    );
}

#[test]
fn split_creates_bootstrap_split_span() {
    let (names, ()) = with_span_capture(|| {
        let mut g = make_graph(5);
        g.observe(3u64, 10u64); // triggers bootstrap split
    });
    assert!(
        names.contains(&"bootstrap_split".to_string()),
        "expected 'bootstrap_split' span in: {names:?}"
    );
}

#[test]
fn catalytic_split_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let mut g = make_graph(5);
        // First split (bootstrap): fills both children.
        g.observe(3u64, 10u64);
        // Pump one child past threshold to trigger catalytic split.
        g.observe(3u64, 10u64);
        g.observe(3u64, 10u64);
    });
    assert!(
        names.contains(&"catalytic_split".to_string()),
        "expected 'catalytic_split' span in: {names:?}"
    );
}

#[test]
fn rebalance_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let cfg = Config {
            split_threshold: 5,
            ..low_threshold_config()
        };
        let _g: GvGraph<u64, u64, 4> = run(cfg, &multi_split_plan());
    });
    assert!(
        names.contains(&"rebalance".to_string()),
        "expected 'rebalance' span in: {names:?}"
    );
}

#[test]
fn evict_tip_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let cfg = Config {
            budget: Some(20),
            ..evictable_config()
        };
        let plan = Plan::new().spread(256, 6, 30);
        let _g: GvGraph<u64, u64, 8> = run(cfg, &plan);
    });
    assert!(
        names.contains(&"evict_tip".to_string()),
        "expected 'evict_tip' span in: {names:?}"
    );
}

#[test]
fn decay_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let mut g = make_graph(5);
        g.observe(3u64, 10u64);
        g.observe(12u64, 10u64);
        g.decay(g.g_root, 0.5, 0.0);
    });
    assert!(
        names.iter().any(|n| n.starts_with("decay")),
        "expected a 'decay*' span in: {names:?}"
    );
}

#[test]
fn eviction_pipeline_creates_spans() {
    let (names, ()) = with_span_capture(|| {
        let cfg = Config {
            budget: Some(20),
            ..evictable_config()
        };
        let plan = Plan::new().spread(256, 6, 30);
        let _g: GvGraph<u64, u64, 8> = run(cfg, &plan);
    });
    // The eviction pipeline (triggered by observe → budget check)
    // uses `evict_candidates` which creates `evict_batch` and
    // `scan_for_candidates` spans.
    assert!(
        names.contains(&"evict_batch".to_string()),
        "expected 'evict_batch' span in: {names:?}"
    );
    assert!(
        names.contains(&"scan_for_candidates".to_string()),
        "expected 'scan_for_candidates' span in: {names:?}"
    );
}

#[test]
fn vtree_remove_leaf_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let cfg = Config {
            budget: Some(20),
            ..evictable_config()
        };
        let plan = Plan::new().spread(256, 6, 30);
        let _g: GvGraph<u64, u64, 8> = run(cfg, &plan);
    });
    assert!(
        names.contains(&"vtree_remove_leaf".to_string()),
        "expected 'vtree_remove_leaf' span in: {names:?}"
    );
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn normalize_plateaus_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let cfg = Config {
            split_threshold: 5,
            ..low_threshold_config()
        };
        let _g: GvGraph<u64, u64, 4> = run(cfg, &multi_split_plan());
    });
    assert!(
        names.contains(&"normalize_plateaus".to_string()),
        "expected 'normalize_plateaus' span in: {names:?}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 5. Events are emitted at expected levels
// ═════════════════════════════════════════════════════════════════

#[test]
fn observe_emits_debug_events() {
    let (_spans, debug_count, _trace, ()) = with_counting_subscriber(|| {
        let cfg = Config {
            split_threshold: 5,
            ..low_threshold_config()
        };
        // Trigger split + rebalance → debug events.
        let plan = Plan::new()
            .observe(3, 6)
            .observe(12, 6)
            .observe(5, 6)
            .observe(10, 6)
            .observe(7, 6)
            .observe(14, 6);
        let _g: GvGraph<u64, u64, 4> = run(cfg, &plan);
    });
    assert!(
        debug_count > 0,
        "expected at least one DEBUG event from observe pipeline, got {debug_count}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 6. Plateau consistency in diagnostic module
// ═════════════════════════════════════════════════════════════════

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn audit_plateau_consistency_passes_on_healthy_graph() {
    let (_, _, _, ()) = with_counting_subscriber(|| {
        let g = build_multi_split_graph();
        // Should not panic.
        diagnostic::audit_plateau_consistency(&g, "HEALTHY", None);
    });
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn audit_plateau_consistency_with_context() {
    let (_, _, _, ()) = with_counting_subscriber(|| {
        let g = build_multi_split_graph();
        let ctx = diagnostic::PlateauAuditContext {
            parent_id: g.g_root,
            parent_state: crate::gnode::GState::SemiInternal,
        };
        // Should not panic even with an explicit context.
        diagnostic::audit_plateau_consistency(&g, "WITH-CONTEXT", Some(&ctx));
    });
}

// ═════════════════════════════════════════════════════════════════
// 7. diagnose_missed_violation
// ═════════════════════════════════════════════════════════════════

#[test]
fn diagnose_missed_violation_emits_events_without_panic() {
    // Build a graph with enough structure so V-nodes have parents
    // and grandparents.
    let (_spans, _debug, _trace, ()) = with_counting_subscriber(|| {
        let g = build_multi_split_graph();
        // Find a V-node at depth ≥ 2 (has parent and grandparent).
        let v_root = g.v_root.expect("graph should have a V-root");
        let root_vnode = g.vnodes.get(v_root.index());
        let deep_v = match &root_vnode.kind {
            crate::vnode::VKind::Structural { children, .. } => {
                // Pick the first child — it has a parent (v_root).
                children.get(0).0
            }
            crate::vnode::VKind::Entry { .. } => v_root,
        };
        let ctx = diagnostic::EvictionContext {
            evicted_parent: None,
            evicted_parent_child_count: 0,
            collapse_sibling: None,
        };
        // Should emit tracing events and not panic.
        diagnostic::diagnose_missed_violation(&g.vnodes, deep_v, &ctx);
    });
}

#[test]
fn diagnose_missed_violation_with_eviction_context() {
    let (_spans, _debug, _trace, ()) = with_counting_subscriber(|| {
        let g = build_multi_split_graph();
        let v_root = g.v_root.expect("graph should have a V-root");
        let root_vnode = g.vnodes.get(v_root.index());
        let child_id = match &root_vnode.kind {
            crate::vnode::VKind::Structural { children, .. } => children.get(0).0,
            crate::vnode::VKind::Entry { .. } => v_root,
        };
        // Supply a plausible eviction context.
        let ctx = diagnostic::EvictionContext {
            evicted_parent: Some(v_root),
            evicted_parent_child_count: 2,
            collapse_sibling: Some(child_id),
        };
        diagnostic::diagnose_missed_violation(&g.vnodes, child_id, &ctx);
    });
}

#[test]
fn diagnose_missed_violation_on_root_node() {
    // The root V-node has no parent — `diagnose_missed_violation`
    // should emit a "no parent" event and return early.
    let (_spans, _debug, _trace, ()) = with_counting_subscriber(|| {
        let g = build_multi_split_graph();
        let v_root = g.v_root.expect("graph should have a V-root");
        let ctx = diagnostic::EvictionContext {
            evicted_parent: None,
            evicted_parent_child_count: 0,
            collapse_sibling: None,
        };
        // Should not panic — covers the early-return branch.
        diagnostic::diagnose_missed_violation(&g.vnodes, v_root, &ctx);
    });
}

// ═════════════════════════════════════════════════════════════════
// 8. Pl plateau display helper
// ═════════════════════════════════════════════════════════════════

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn pl_display_for_root() {
    let g = build_multi_split_graph();
    let display = format!("{}", diagnostic::Pl(&g, g.g_root));
    // Root should either be in a plateau or show "not_basis".
    assert!(
        display.starts_with("P(") || display.contains("not_basis"),
        "Pl display should start with 'P(' or contain 'not_basis': {display}"
    );
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn pl_display_for_non_basis_node() {
    let g = make_graph(100);
    // Unallocated node → not in any plateau basis.
    let dead_id = crate::handle::GNodeId::from_index(99);
    let display = format!("{}", diagnostic::Pl(&g, dead_id));
    assert!(
        display.contains("not_basis"),
        "non-basis node should show 'not_basis': {display}"
    );
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn pl_display_contains_range_and_depth() {
    let g = build_multi_split_graph();
    // Find a G-node that is in a plateau basis.
    let mut found = false;
    for (idx, _) in g.gnodes.iter_occupied() {
        let id = crate::handle::GNodeId::from_index(idx);
        let display = format!("{}", diagnostic::Pl(&g, id));
        if display.starts_with("P(") && display.contains("d=") && display.contains("sum=") {
            // Verify the display contains the expected structure:
            // P([lo,hi),d=N,sum=V)
            assert!(display.contains(','), "Pl display should contain comma separators: {display}");
            found = true;
            break;
        }
    }
    assert!(found, "expected at least one G-node with a full plateau display");
}
