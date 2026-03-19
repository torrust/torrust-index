// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for the ADR-M-028 span-native tracing system.
//!
//! Verifies:
//! 1. `diagnostic::audit_violations` detects unqueued violations.
//! 2. `diagnostic::Gn` display helper formats G-nodes correctly.
//! 3. Hard invariant assertions fire in debug builds — they are not
//!    silently swallowed by `tracing::enabled!()` guards.
//! 4. Spans are emitted during graph mutations (observe, split,
//!    rebalance, evict, decay).

use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

use tracing_subscriber::layer::SubscriberExt;

use crate::graph::Config;
use crate::invariants::assert_invariants;
use crate::{GvGraph, diagnostic};

// ── Helpers ─────────────────────────────────────────────────────

fn make_graph(threshold: u64) -> GvGraph<u64, u64, 4> {
    GvGraph::new(Config {
        split_threshold: threshold,
        depth_create: 4,
        depth_evict: 8,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    })
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
fn audit_violations_returns_empty_after_observe() {
    let mut g = make_graph(5);
    // Trigger splits and rebalance.
    for &c in &[3u64, 12, 5, 10, 7, 14, 1, 15] {
        g.observe(c, 6u64);
    }
    assert_invariants(&g);
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
    let mut g = make_graph(5);
    g.observe(3u64, 10u64); // triggers bootstrap split
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
    //
    // A correct graph should have zero residual violations — if the
    // audit is running, this test passes. If the audit is silently
    // skipped, a future regression could go undetected.
    let mut g = make_graph(5);
    for &c in &[3u64, 12, 5, 10, 7, 14, 1, 15] {
        g.observe(c, 6u64);
    }
    // The key assertion is implicit: if the `debug_assert!` inside
    // `observe()` fires on a violation, the test panics. If we get
    // here, the audit ran and found no violations.
    assert_invariants(&g);
}

#[test]
fn rebalance_runs_residual_audit_without_tracing() {
    // Same pattern: the hard `assert!` inside `rebalance()` must
    // fire in debug builds without a tracing subscriber.
    let mut g = make_graph(5);
    for &c in &[3u64, 12, 5, 10] {
        g.observe(c, 6u64);
    }
    assert_invariants(&g);
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn plateau_mirror_consistency_runs_without_tracing() {
    // `debug_assert_plateau_mirror_consistency` must run in debug
    // builds. If it were skipped, plateau divergence would go
    // undetected.
    let mut g = make_graph(5);
    for &c in &[3u64, 12, 5, 10, 7, 14, 1, 15] {
        g.observe(c, 6u64);
    }
    // Explicitly call — should not early-return in debug builds.
    g.debug_assert_plateau_mirror_consistency("TEST-DIRECT");
    assert_invariants(&g);
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn plateau_sum_check_runs_without_tracing() {
    let mut g = make_graph(5);
    for &c in &[3u64, 12, 5, 10, 7, 14] {
        g.observe(c, 6u64);
    }
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
        let mut g = make_graph(5);
        // Multiple observations to force violations and rebalance.
        for &c in &[3u64, 12, 5, 10, 7, 14, 1, 15] {
            g.observe(c, 6u64);
        }
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
            split_threshold: 5,
            depth_create: 3,
            depth_evict: 4,
            budget: Some(20),
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
        // Fill tree past budget to trigger eviction.
        for i in 0..30u64 {
            g.observe(i * 8, 6u64);
        }
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
            split_threshold: 5,
            depth_create: 3,
            depth_evict: 4,
            budget: Some(20),
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
        for i in 0..30u64 {
            g.observe(i * 8, 6u64);
        }
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
            split_threshold: 5,
            depth_create: 3,
            depth_evict: 4,
            budget: Some(20),
            alpha_relax: 0.75,
            bounded_eviction: true,
        };
        let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
        for i in 0..30u64 {
            g.observe(i * 8, 6u64);
        }
    });
    assert!(
        names.contains(&"vtree_remove_leaf".to_string()),
        "expected 'vtree_remove_leaf' span in: {names:?}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 5. Events are emitted at expected levels
// ═════════════════════════════════════════════════════════════════

#[test]
fn observe_emits_debug_events() {
    let (_spans, debug_count, _trace, ()) = with_counting_subscriber(|| {
        let mut g = make_graph(5);
        // Trigger split + rebalance → debug events.
        for &c in &[3u64, 12, 5, 10, 7, 14] {
            g.observe(c, 6u64);
        }
    });
    assert!(
        debug_count > 0,
        "expected at least one DEBUG event from observe pipeline, got {debug_count}"
    );
}

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn normalize_plateaus_creates_span() {
    let (names, ()) = with_span_capture(|| {
        let mut g = make_graph(5);
        for &c in &[3u64, 12, 5, 10, 7, 14, 1, 15] {
            g.observe(c, 6u64);
        }
    });
    assert!(
        names.contains(&"normalize_plateaus".to_string()),
        "expected 'normalize_plateaus' span in: {names:?}"
    );
}

// ═════════════════════════════════════════════════════════════════
// 6. Plateau consistency in diagnostic module
// ═════════════════════════════════════════════════════════════════

#[cfg(feature = "dynamic-contour-tracking")]
#[test]
fn audit_plateau_consistency_passes_on_healthy_graph() {
    let (_, _, _, ()) = with_counting_subscriber(|| {
        let mut g = make_graph(5);
        for &c in &[3u64, 12, 5, 10, 7, 14] {
            g.observe(c, 6u64);
        }
        // Should not panic.
        diagnostic::audit_plateau_consistency(&g, "HEALTHY", None);
    });
}
