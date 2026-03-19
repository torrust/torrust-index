// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for `terminal_count()`.

use torrust_mudlark::invariants::assert_invariants;
#[allow(unused_imports)]
use torrust_mudlark::testing::{Plan, default_config, evictable_config, plan_evictable, run};
use torrust_mudlark::{Config, GvGraph};

#[test]
fn terminal_count_initial() {
    let g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    assert_eq!(g.terminal_count(), 1);
    assert_invariants(&g);
}

#[test]
fn terminal_count_after_bootstrap_split() {
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    // Trigger bootstrap split (root terminal → 2 children).
    g.observe(0u64, 6u64);
    assert_invariants(&g);
    // Root splits: +2 terminals, −1 parent = net +1 → 2 terminals.
    assert_eq!(g.terminal_count(), 2);
}

#[test]
fn terminal_count_after_catalytic_split() {
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
fn terminal_count_after_evict_to_semi_internal() {
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    assert_invariants(&g);
    let tc_before = g.terminal_count();
    // Evict — some deep entries become eligible.
    let evicted = g.check_evictions();
    assert_invariants(&g);
    if evicted > 0 {
        assert!(g.terminal_count() <= tc_before);
    }
}

#[test]
fn terminal_count_after_evict_to_terminal() {
    let mut g = run::<u64, u64, 8>(evictable_config(), &plan_evictable());
    assert_invariants(&g);
    // Evict all eligible tips — some parents become terminal.
    let _evicted = g.check_evictions();
    assert_invariants(&g);
    // The invariant checker validates terminal_count matches
    // the arena walk, so this is sufficient.
}

#[test]
fn terminal_count_budget_constrained() {
    let cfg = Config {
        split_threshold: 5,
        depth_create: 2,
        depth_evict: 3,
        budget: Some(12),
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 4> = GvGraph::new(cfg);
    // Rapid-fire observations to push the budget.
    for i in 0..50u64 {
        g.observe(i % 16, 5u64);
        assert_invariants(&g);
    }
    // terminal_count is validated by assert_invariants on every step.
}

#[test]
fn terminal_count_cross_type_u32() {
    let cfg = Config {
        split_threshold: 5u64,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u32, u64, 8> = GvGraph::new(cfg);
    assert_eq!(g.terminal_count(), 1);
    g.observe(0u32, 6u64);
    assert_invariants(&g);
    assert_eq!(g.terminal_count(), 2);
}
