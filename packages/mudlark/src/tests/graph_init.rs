// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate tests for **graph initialization** (`GvGraph::new`).
//!
//! A freshly constructed `GvGraph` must satisfy a strict set of
//! post-conditions: the G-root spans the full coordinate domain, the
//! V-root is an exposed entry backed by that G-root, aggregate
//! counters start at their identity values, and the full invariant
//! suite passes.  These tests lock down those guarantees across
//! different values of *N* and for both integer and floating-point
//! coordinate types.
//!
//! # Test index
//!
//! ## G-root properties
//!
//! | Test | Focus |
//! |------|-------|
//! | [`g_root_covers_full_domain_n32`] | domain `[0, 2^32)` for `N = 32` |
//! | [`g_root_covers_full_domain_n8`] | domain `[0, 256)` for `N = 8` |
//! | [`g_root_covers_full_domain_n64`] | domain `[0, u64::MAX)` for `N = 64` |
//! | [`g_root_is_first_arena_slot`] | root occupies arena index 0 |
//! | [`g_root_fields_are_zeroed`] | `sum` and `own` start at zero |
//! | [`g_root_has_no_children_or_parent`] | no children, no parent link |
//! | [`g_root_entry_links_to_v_root`] | `entry` field points to V-root |
//!
//! ## V-root properties
//!
//! | Test | Focus |
//! |------|-------|
//! | [`v_root_is_present`] | V-root handle is `Some` |
//! | [`v_root_is_entry_backed_by_g_root`] | `VKind::Entry` backed by G-root |
//! | [`v_root_is_exposed_and_evictable`] | exposed and evictable flags set |
//! | [`v_root_has_zero_intensity_and_depth`] | intensity = 0, depth = 0, no parent |
//!
//! ## Aggregate counters
//!
//! | Test | Focus |
//! |------|-------|
//! | [`node_count_is_one`] | exactly one G-node after init |
//! | [`terminal_count_is_one`] | exactly one terminal (the root) |
//! | [`total_sum_is_zero`] | total energy is zero |
//! | [`no_pending_violations`] | no pending invariant violations |
//!
//! ## Config round-trip
//!
//! | Test | Focus |
//! |------|-------|
//! | [`config_accessor_returns_supplied_config`] | config fields survive construction |
//! | [`budget_accessor_mirrors_config`] | `budget()` reflects config (unbounded and bounded) |
//!
//! ## Invariants
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fresh_graph_satisfies_all_invariants`] | full invariant suite on a fresh graph |
//! | [`fresh_budgeted_graph_satisfies_all_invariants`] | full invariant suite with a budget cap |
//!
//! ## Coordinate / value type variants
//!
//! | Test | Focus |
//! |------|-------|
//! | [`f64_graph_initializes_correctly`] | f64 coordinate + value type initializes cleanly |

use std::sync::atomic::Ordering;

use crate::graph::GvGraph;
use crate::handle::GNodeId;
use crate::invariants::assert_invariants;
use crate::testing::{budget_config, default_config, f64_default_config};
use crate::vnode::VKind;

// ── G-root properties ───────────────────────────────────────────

#[test]
fn g_root_covers_full_domain_n32() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, 1u64 << 32);
}

#[test]
fn g_root_covers_full_domain_n8() {
    let graph: GvGraph<u64, u64, 8> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, 256);
}

#[test]
fn g_root_covers_full_domain_n64() {
    // N == C::BITS: `domain_max` returns `u64::MAX`.
    let graph: GvGraph<u64, u64, 64> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, u64::MAX);
}

#[test]
fn g_root_is_first_arena_slot() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.g_root(), GNodeId::from_index(0));
}

#[test]
fn g_root_fields_are_zeroed() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.sum, 0);
    assert_eq!(g.own, 0);
}

#[test]
fn g_root_has_no_children_or_parent() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert!(g.left.is_none());
    assert!(g.right.is_none());
    assert!(g.parent.is_none());
}

#[test]
fn g_root_entry_links_to_v_root() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.entry, graph.v_root());
}

// ── V-root properties ───────────────────────────────────────────

#[test]
fn v_root_is_present() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert!(graph.v_root().is_some());
}

#[test]
fn v_root_is_entry_backed_by_g_root() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let v = graph.vnodes().get(graph.v_root().unwrap().index());
    match &v.kind {
        VKind::Entry { gnode, .. } => assert_eq!(*gnode, graph.g_root()),
        VKind::Structural { .. } => panic!("V-root should be an Entry, not Structural"),
    }
}

#[test]
fn v_root_is_exposed_and_evictable() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let v = graph.vnodes().get(graph.v_root().unwrap().index());
    match &v.kind {
        VKind::Entry {
            is_exposed,
            is_evictable,
            ..
        } => {
            assert!(is_exposed, "root entry should be exposed");
            assert!(is_evictable, "root entry should be evictable");
        }
        VKind::Structural { .. } => panic!("V-root should be an Entry"),
    }
}

#[test]
fn v_root_has_zero_intensity_and_depth() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    let v = graph.vnodes().get(graph.v_root().unwrap().index());
    assert_eq!(v.intensity, 0);
    assert!(v.parent.is_none());
    assert_eq!(v.cached_depth.load(Ordering::Relaxed), 0);
}

// ── Aggregate counters ──────────────────────────────────────────

#[test]
fn node_count_is_one() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.node_count(), 1);
}

#[test]
fn terminal_count_is_one() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.terminal_count(), 1);
}

#[test]
fn total_sum_is_zero() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.total_sum(), 0u64);
}

#[test]
fn no_pending_violations() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert!(!graph.has_pending_violations());
}

// ── Config round-trip ───────────────────────────────────────────

#[test]
fn config_accessor_returns_supplied_config() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.config().split_threshold, 5);
    assert_eq!(graph.config().depth_create, 3);
    assert_eq!(graph.config().depth_evict, 6);
}

#[test]
fn budget_accessor_mirrors_config() {
    let unbounded: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(unbounded.budget(), None);

    let budgeted: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(500));
    assert_eq!(budgeted.budget(), Some(500));
}

// ── Invariants ──────────────────────────────────────────────────

#[test]
fn fresh_graph_satisfies_all_invariants() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_invariants(&graph);
}

#[test]
fn fresh_budgeted_graph_satisfies_all_invariants() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(budget_config(500));
    assert_invariants(&graph);
}

// ── Coordinate / value type variants ────────────────────────────

#[test]
fn f64_graph_initializes_correctly() {
    let graph: GvGraph<f64, f64, 32> = GvGraph::new(f64_default_config());
    assert_eq!(graph.node_count(), 1);
    assert_eq!(graph.terminal_count(), 1);
    assert!(graph.total_sum().abs() < f64::EPSILON);
    assert!(graph.v_root().is_some());
    assert_invariants(&graph);
}
