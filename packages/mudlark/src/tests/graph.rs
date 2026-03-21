// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate-level tests for **`GvGraph` internals** that require direct
//! access to private fields (`node_count`, `live_depth_evict`,
//! `live_depth_create`, `violations`) or `pub(crate)` methods
//! (`adjust_depth_gates`, `check_evictions_bounded`, `gnode_depth`).
//!
//! These tests live at the crate level because the APIs under test are
//! intentionally hidden from the public surface — they are
//! implementation details of the adaptive depth-gate machinery
//! (ADR-M-017) and the eviction subsystem.  All other graph tests live
//! in `tests/graph_*.rs` integration test files or in more targeted
//! crate test modules (`config_validation.rs`, `evict.rs`, etc.).
//!
//! # Test index
//!
//! ## Shadow fields / config accessors
//!
//! | Test | Focus |
//! |------|-------|
//! | [`shadow_fields_match_config_on_construction`] | depth gates and headroom mirror `Config` at construction |
//! | [`soft_limit_computed_correctly`] | `soft_limit()` derived from `budget` and `headroom` |
//!
//! ## `adjust_depth_gates` (ADR-M-017)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`tighten_when_over_budget`] | gates decrease when `node_count > soft_limit` |
//! | [`relax_when_under_alpha_budget`] | gates increase when `node_count < α × soft_limit` |
//! | [`dead_zone_no_change`] | gates unchanged inside the dead-zone band |
//! | [`floor_prevents_excessive_tightening`] | repeated tightening bottoms out at the floor |
//! | [`no_ceiling_relax_above_initial`] | relaxation can exceed initial config values |
//! | [`noop_when_budget_is_none`] | adjustment is a no-op when `budget` is `None` |
//! | [`di3_maintained_through_full_tighten_relax_cycle`] | D-I3 (`D_create < D_evict`) holds across a full tighten→relax cycle |
//!
//! ## `check_evictions` / `check_evictions_bounded`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`check_evictions_unbounded_evicts_all`] | unbounded eviction removes all eligible candidates |
//! | [`check_evictions_bounded_respects_limit`] | bounded variant respects the per-call limit |
//! | [`check_evictions_return_value_matches_count_delta`] | return value equals `count_before − count_after` |
//! | [`check_evictions_trailing_rebalance_clears_violations`] | trailing rebalance drains the violation queue |
//! | [`check_evictions_noop_when_none_eligible`] | no-op when no nodes sit beyond `D_evict` |
//!
//! ## `gnode_depth` (`pub(crate)`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`gnode_depth_method_matches_free_function`] | method agrees with the free `gnode_depth_from_interval` function |
//!
//! ## Semi-internal `get` (needs `build_evictable_graph`)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`get_semi_internal_left_child_trims_right`] | left-child semi-internal trims the right half |
//! | [`get_semi_internal_right_child_trims_left`] | right-child semi-internal trims the left half |
//! | [`get_semi_internal_depth_is_trimmed_depth`] | returned cell depth equals `arena_depth + 1` |
//!
//! ## Compile-time trait assertions
//!
//! | Test | Focus |
//! |------|-------|
//! | `Send + Sync` | `GvGraph` and `Config` are `Send + Sync` (ADR-M-007) |

use crate::invariants::assert_invariants;
use crate::testing::{budget_config, default_config, evictable_config, plan_evictable, run};
use crate::{Config, GvGraph};

// ── Shared helpers (private-field access) ───────────────────

/// Build a graph with terminal entries past `D_evict`.
///
/// Uses `N=8` (domain `[0,256)`), `depth_create=3`, `depth_evict=4`.
/// After filling the tree, lowers `D_evict` to the floor (2) so
/// deep V-entries become eligible. D-I3 is respected.
fn build_evictable_graph() -> GvGraph<u64, u64, 8> {
    let mut g: GvGraph<u64, u64, 8> = run(evictable_config(), &plan_evictable());
    // buffer = 4 - 3 = 1, floor = 1 + 1 = 2.
    g.live_depth_evict = 2;
    g.live_depth_create = 2 - g.depth_buffer();
    g
}

// ── Shadow fields / config accessors ────────────────────────

#[test]
fn shadow_fields_match_config_on_construction() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.depth_create(), 3);
    assert_eq!(graph.depth_evict(), 6);
    assert_eq!(graph.depth_buffer(), 3);
    assert_eq!(graph.config().depth_create, 3);
    assert_eq!(graph.config().depth_evict, 6);
    assert_eq!(graph.soft_limit(), None);
    assert_eq!(graph.headroom(), 81);
}

#[test]
fn soft_limit_computed_correctly() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    assert_eq!(graph.headroom(), 81);
    assert_eq!(graph.soft_limit(), Some(19));
}

// ── adjust_depth_gates (ADR-M-017, private node_count) ────────

#[test]
fn tighten_when_over_budget() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    g.node_count = 20;
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 5);
    assert_eq!(g.depth_create(), 2);
}

#[test]
fn relax_when_under_alpha_budget() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    g.node_count = 10;
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 7);
    assert_eq!(g.depth_create(), 4);
}

#[test]
fn dead_zone_no_change() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    g.node_count = 16;
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 6);
    assert_eq!(g.depth_create(), 3);
}

#[test]
fn floor_prevents_excessive_tightening() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    g.node_count = 100;
    g.adjust_depth_gates();
    g.adjust_depth_gates();
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 4);
    assert_eq!(g.depth_create(), 1);
    assert!(g.depth_create() < g.depth_evict());
}

#[test]
fn no_ceiling_relax_above_initial() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));
    g.node_count = 1;
    g.adjust_depth_gates();
    g.adjust_depth_gates();
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 9);
    assert_eq!(g.depth_create(), 6);
}

#[test]
fn noop_when_budget_is_none() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    g.node_count = 1000;
    g.adjust_depth_gates();
    assert_eq!(g.depth_evict(), 6);
    assert_eq!(g.depth_create(), 3);
}

#[test]
fn di3_maintained_through_full_tighten_relax_cycle() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(100));

    g.node_count = 100;
    for _ in 0..10 {
        g.adjust_depth_gates();
        assert!(g.depth_create() < g.depth_evict(), "D-I3 during tighten");
        assert_eq!(g.depth_buffer(), g.depth_evict() - g.depth_create());
    }
    let at_floor = g.depth_evict();

    g.node_count = 1;
    for _ in 0..10 {
        g.adjust_depth_gates();
        assert!(g.depth_create() < g.depth_evict(), "D-I3 during relax");
        assert_eq!(g.depth_buffer(), g.depth_evict() - g.depth_create());
    }
    assert!(g.depth_evict() > at_floor);
}

// ── check_evictions (private live_depth_evict) ──────────────

#[test]
fn check_evictions_unbounded_evicts_all() {
    let mut g = build_evictable_graph();
    let count_before = g.node_count();
    let evicted = g.check_evictions();
    assert!(evicted > 0, "should evict at least one candidate");
    assert_eq!(g.node_count(), count_before - evicted);
    assert_invariants(&g);
}

#[test]
fn check_evictions_bounded_respects_limit() {
    let mut g = build_evictable_graph();
    let evicted = g.check_evictions_bounded(1);
    assert!(evicted <= 1);
    assert_invariants(&g);
}

#[test]
fn check_evictions_return_value_matches_count_delta() {
    let mut g = build_evictable_graph();
    let count_before = g.node_count();
    let evicted = g.check_evictions();
    assert_eq!(count_before - g.node_count(), evicted);
}

#[test]
fn check_evictions_trailing_rebalance_clears_violations() {
    let mut g = build_evictable_graph();
    let _evicted = g.check_evictions();
    assert!(g.violations.is_empty(), "violations should be drained by trailing rebalance");
    assert_invariants(&g);
}

#[test]
fn check_evictions_noop_when_none_eligible() {
    let mut g: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    g.observe(100u64, 10u64);
    let evicted = g.check_evictions();
    assert_eq!(evicted, 0);
    assert_invariants(&g);
}

// ── gnode_depth (pub(crate)) ────────────────────────────────

#[test]
fn gnode_depth_method_matches_free_function() {
    let cfg = Config {
        split_threshold: 1,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    };
    let mut g: GvGraph<u64, u64, 8> = GvGraph::new(cfg);
    g.observe(32u64, 15u64);
    g.observe(96u64, 5u64);
    g.observe(200u64, 20u64);

    assert_eq!(g.gnode_depth(g.g_root()), 0);
    let root = g.gnodes.get(g.g_root().index());
    if let Some(left_id) = root.left {
        let left = g.gnodes.get(left_id.index());
        assert_eq!(
            g.gnode_depth(left_id),
            crate::gtree::gnode_depth_from_interval(left.lo, left.hi, 8)
        );
        assert_eq!(g.gnode_depth(left_id), 1);
    }
}

// ── Semi-internal get tests (need build_evictable_graph) ────

#[test]
fn get_semi_internal_left_child_trims_right() {
    let mut g = build_evictable_graph();
    let _evicted = g.check_evictions();
    assert_invariants(&g);

    let mut found = false;
    for (_idx, gn) in g.gnodes.iter_occupied() {
        if gn.state() == crate::gnode::GState::SemiInternal && gn.left.is_some() {
            let mid = u64::midpoint(gn.lo, gn.hi);
            let cell = g.get(mid);
            assert_eq!(cell.start, mid);
            assert_eq!(cell.end, gn.hi);
            found = true;
            break;
        }
    }
    if !found {
        for (_idx, gn) in g.gnodes.iter_occupied() {
            if gn.state() == crate::gnode::GState::SemiInternal && gn.right.is_some() {
                let mid = u64::midpoint(gn.lo, gn.hi);
                let cell = g.get(gn.lo);
                assert_eq!(cell.start, gn.lo);
                assert_eq!(cell.end, mid);
                found = true;
                break;
            }
        }
    }
    assert!(found, "expected at least one semi-internal after eviction");
}

#[test]
fn get_semi_internal_right_child_trims_left() {
    let mut g = build_evictable_graph();
    let _evicted = g.check_evictions();
    assert_invariants(&g);

    let mut found = false;
    for (_idx, gn) in g.gnodes.iter_occupied() {
        if gn.state() == crate::gnode::GState::SemiInternal && gn.right.is_some() {
            let mid = u64::midpoint(gn.lo, gn.hi);
            let cell = g.get(gn.lo);
            assert_eq!(cell.start, gn.lo);
            assert_eq!(cell.end, mid);
            found = true;
            break;
        }
    }
    if !found {
        for (_idx, gn) in g.gnodes.iter_occupied() {
            if gn.state() == crate::gnode::GState::SemiInternal && gn.left.is_some() {
                let mid = u64::midpoint(gn.lo, gn.hi);
                let cell = g.get(mid);
                assert_eq!(cell.start, mid);
                assert_eq!(cell.end, gn.hi);
                found = true;
                break;
            }
        }
    }
    assert!(found, "expected at least one semi-internal after eviction");
}

#[test]
fn get_semi_internal_depth_is_trimmed_depth() {
    let mut g = build_evictable_graph();
    let _evicted = g.check_evictions();
    assert_invariants(&g);

    for (_idx, gn) in g.gnodes.iter_occupied() {
        if gn.state() == crate::gnode::GState::SemiInternal {
            let mid = u64::midpoint(gn.lo, gn.hi);
            let arena_depth = crate::gtree::gnode_depth_from_interval(gn.lo, gn.hi, 8);
            let trimmed_depth = arena_depth + 1;
            let coord = if gn.left.is_some() { mid } else { gn.lo };
            let cell = g.get(coord);
            assert_eq!(cell.depth, trimmed_depth, "trimmed depth should be arena_depth + 1");
            break;
        }
    }
}

// ── Send + Sync compile-time assertions (ADR-M-007) ───────────

const _: () = {
    #[allow(dead_code)]
    fn assert_send_sync<T: Send + Sync>() {}

    #[allow(dead_code)]
    fn check() {
        assert_send_sync::<GvGraph<u64, u64, 32>>();
        assert_send_sync::<GvGraph<u128, f64, 128>>();
        assert_send_sync::<GvGraph<f64, f64, 52>>();
        assert_send_sync::<Config<u64>>();
        assert_send_sync::<Config<f64>>();
    }
};
