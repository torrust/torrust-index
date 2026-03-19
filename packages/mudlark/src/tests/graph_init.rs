// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Integration tests for graph initialization.

use crate::graph::GvGraph;
use crate::testing::default_config;
use crate::vnode::VKind;

#[test]
fn initialize_single_root() {
    let graph: GvGraph<u64, u64, 32> = GvGraph::new(default_config());
    assert_eq!(graph.node_count(), 1);
    assert!(graph.v_root().is_some());

    // G-root covers [0, 2^32).
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, 1u64 << 32);
    assert_eq!(g.sum, 0);
    assert_eq!(g.own, 0);
    assert!(g.left.is_none());
    assert!(g.right.is_none());
    assert!(g.parent.is_none());
    assert!(g.entry.is_some());

    // V-root is an entry backed by the G-root.
    let v_root = graph.v_root().unwrap();
    let v = graph.vnodes().get(v_root.index());
    assert_eq!(v.intensity, 0);
    assert!(v.parent.is_none());
    if let VKind::Entry { gnode, is_exposed, .. } = &v.kind {
        assert_eq!(*gnode, graph.g_root());
        assert!(is_exposed);
    } else {
        panic!("V-root should be an entry");
    }
}

#[test]
fn initialize_full_domain() {
    // N == C::BITS: domain_max returns u64::MAX.
    let graph: GvGraph<u64, u64, 64> = GvGraph::new(default_config());
    let g = graph.gnodes().get(graph.g_root().index());
    assert_eq!(g.lo, 0);
    assert_eq!(g.hi, u64::MAX);
}
