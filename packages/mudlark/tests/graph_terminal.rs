// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for **`terminal_count()`** — the number of leaf
//! (terminal) nodes in a `GvGraph`.
//!
//! A terminal is a node with no children.  A fresh graph contains
//! exactly one terminal (the root).  Every split converts one terminal
//! into a parent and adds two new children, raising the count by one.
//! Eviction prunes subtrees, potentially reducing the count.  Budget
//! constraints and adversarial access patterns must never violate the
//! invariant `1 ≤ terminal_count ≤ node_count`.
//!
//! The suite exercises the full lifecycle: initial state, growth
//! through bootstrap and catalytic splits, monotonicity without
//! eviction, eviction convergence, budget pressure, cross-type
//! instantiation, and adversarial input.
//!
//! # Test index
//!
//! ## Fresh graph
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_initial`] | fresh graph has exactly 1 terminal (the root) |
//!
//! ## Growth via splits
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_after_bootstrap_split`] | root splits → 2 terminals |
//! | [`terminal_count_after_catalytic_split`] | second split → 3 terminals |
//! | [`terminal_count_stays_one_under_no_split`] | high threshold prevents splits: stays 1 |
//! | [`terminal_count_monotone_without_eviction`] | without eviction, terminal count never decreases |
//! | [`terminal_count_leq_node_count`] | `terminal_count` ≤ `node_count` always holds |
//! | [`terminal_count_many_splits_via_run_checked`] | invariant-checked growth over many observations |
//!
//! ## Eviction
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_after_eviction`] | eviction may reduce or maintain terminal count |
//! | [`terminal_count_multiple_eviction_rounds`] | repeated eviction rounds converge |
//!
//! ## Budget
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_budget_constrained`] | terminal count valid under tight budget |
//!
//! ## Cross-type
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_cross_type_u32`] | works with `C=u32, V=u64` instantiation |
//!
//! ## Stress / adversarial
//!
//! | Test | Focus |
//! |------|-------|
//! | [`terminal_count_adversarial_zigzag`] | valid under adversarial zigzag pattern |

use torrust_mudlark::GvGraph;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{
    Plan, budget_config, default_config, evictable_config, no_split_config, plan_adversarial, plan_evictable, run, run_checked,
};

mod support;
use support::init_tracing;

// ── Fresh graph ─────────────────────────────────────────────────

#[test]
fn terminal_count_initial() {
    let _t = init_tracing();
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_eq!(g.terminal_count(), 1);
    assert_invariants(&g);
}

// ── Growth via splits ───────────────────────────────────────────

#[test]
fn terminal_count_after_bootstrap_split() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Trigger bootstrap split (root terminal → 2 children).
    g.observe(0u64, 6u64);
    assert_invariants(&g);
    // Root splits: +2 terminals, −1 parent = net +1 → 2 terminals.
    assert_eq!(g.terminal_count(), 2);
}

#[test]
fn terminal_count_after_catalytic_split() {
    let _t = init_tracing();
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Bootstrap split.
    g.observe(0u64, 6u64);
    assert_invariants(&g);
    assert_eq!(g.terminal_count(), 2);
    // Catalytic split on one of the children.
    g.observe(0u64, 6u64);
    assert_invariants(&g);
    assert_eq!(g.terminal_count(), 3);
}

#[test]
fn terminal_count_stays_one_under_no_split() {
    let _t = init_tracing();
    // θ=100 prevents any split for modest observations.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(no_split_config());
    for i in 0..20u64 {
        g.observe(i % 8, 3u64);
    }
    assert_eq!(g.terminal_count(), 1, "high threshold should prevent splits");
    assert_eq!(g.node_count(), 1);
    assert_invariants(&g);
}

#[test]
fn terminal_count_monotone_without_eviction() {
    let _t = init_tracing();
    // Without budget or eviction, terminal_count can only grow.
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let mut prev = g.terminal_count();
    for i in 0..40u64 {
        g.observe(i % 16, 5u64);
        let cur = g.terminal_count();
        assert!(
            cur >= prev,
            "terminal_count must be monotone without eviction: {prev} → {cur}"
        );
        prev = cur;
    }
    assert_invariants(&g);
}

#[test]
fn terminal_count_leq_node_count() {
    let _t = init_tracing();
    let plan = Plan::new().spread(256, 5, 60);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 1);
    assert!(
        g.terminal_count() <= g.node_count(),
        "terminal_count ({}) must be ≤ node_count ({})",
        g.terminal_count(),
        g.node_count()
    );
}

#[test]
fn terminal_count_many_splits_via_run_checked() {
    let _t = init_tracing();
    // Sustained spread to produce many splits; invariants (including
    // terminal_count consistency) checked every 5 observations.
    let plan = Plan::new().spread(256, 6, 100);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 5);
    assert!(g.terminal_count() > 1, "should have split at least once");
    assert_invariants(&g);
}

// ── Eviction ────────────────────────────────────────────────────

#[test]
fn terminal_count_after_eviction() {
    let _t = init_tracing();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    assert_invariants(&g);
    let tc_before = g.terminal_count();
    let evicted = g.check_evictions();
    assert_invariants(&g);
    if evicted > 0 {
        assert!(
            g.terminal_count() <= tc_before,
            "terminal_count should not increase after eviction"
        );
    }
}

#[test]
fn terminal_count_multiple_eviction_rounds() {
    let _t = init_tracing();
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    assert_invariants(&g);
    // Run several eviction rounds until convergence.
    for _ in 0..5 {
        let evicted = g.check_evictions();
        assert_invariants(&g);
        if evicted == 0 {
            break;
        }
    }
    // After convergence, terminal_count must still be valid.
    assert!(g.terminal_count() >= 1, "must have at least the root");
    assert!(g.terminal_count() <= g.node_count());
}

// ── Budget ──────────────────────────────────────────────────────

#[test]
fn terminal_count_budget_constrained() {
    let _t = init_tracing();
    let plan = Plan::new().spread(16, 5, 50);
    // Invariants (including terminal_count) checked every 5 steps.
    let g = run_checked::<u64, u64, 4>(budget_config(100), &plan, 5);
    assert!(g.terminal_count() >= 1);
    assert_invariants(&g);
}

// ── Cross-type ──────────────────────────────────────────────────

#[test]
fn terminal_count_cross_type_u32() {
    let _t = init_tracing();
    let mut g: GvGraph<u32, u64, 8> = GvGraph::new(default_config());
    assert_eq!(g.terminal_count(), 1);
    g.observe(0u32, 6u64);
    assert_invariants(&g);
    assert_eq!(g.terminal_count(), 2);
}

// ── Stress / adversarial ────────────────────────────────────────

#[test]
fn terminal_count_adversarial_zigzag() {
    let _t = init_tracing();
    let plan = plan_adversarial(256, 80);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 10);
    assert!(g.terminal_count() >= 1);
    assert!(g.terminal_count() <= g.node_count());
    assert_invariants(&g);
}
