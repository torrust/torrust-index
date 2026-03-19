// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Rebalance stress tests: large-tree scenarios that exercise the
//! proportional iteration safety-net.
//!
//! These tests construct V-trees directly (via [`SpikedVTree`]) to
//! create worst-case violation patterns that cascade through thousands
//! of queue iterations.  They prove that the proportional limit
//! (`20 × node_count`, floor 10 000) accommodates legitimate workloads
//! that would trip a small fixed constant.

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::rebalance::{find_violated_nodes, rebalance};
use crate::tests::init_tracing;
use crate::tests::spiked_vtree::SpikedVTree;

// ── Tests ───────────────────────────────────────────────────────

/// A valid balanced tree with thousands of spiked entries produces a
/// cascading rebalance that exceeds 10 000 iterations.  The
/// proportional safety-net must accommodate this without panicking.
///
/// Construction: depth-13 balanced 2-node tree (8 192 entries,
/// 16 383 total nodes).  The first 2 of every 4 entries are spiked
/// to 10 000; the rest stay at 1.  Each spiked entry's uncle
/// subtree has low intensity → violated (V-I3).  The resulting
/// cascade generates well over 10 000 queue iterations (including
/// guard-skipped stale entries).
#[test]
fn large_violation_cascade_exceeds_fixed_10k_limit() {
    let _t = init_tracing();

    let (mut vnodes, _root, mut violations) = SpikedVTree::balanced(13).base(1u64).spike(10_000u64, 4, 2).build();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    gnodes.alloc(GNode {
        lo: 0,
        hi: 1,
        sum: 0,
        own: 0,
        left: None,
        right: None,
        parent: None,
        entry: None,
    });

    assert_eq!(SpikedVTree::<u64>::balanced(13).leaf_count(), 1 << 13);
    assert!(
        violations.len() > 2_000,
        "expected >2 000 initial violations, got {}",
        violations.len(),
    );

    // With the old MAX_ITERATIONS = 10_000 constant this would
    // have panicked.  The proportional limit handles it.
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(
        find_violated_nodes(&vnodes).is_empty(),
        "all violations should be resolved after rebalance",
    );
}

/// Smaller tree (depth 10, 1 024 entries) with alternating groups
/// spiked — verifies the cascade shape is correct even at moderate
/// scale.
#[test]
fn moderate_cascade_alternating_groups_spiked() {
    let _t = init_tracing();

    let (mut vnodes, _root, mut violations) = SpikedVTree::balanced(10).base(1u64).spike(5_000u64, 4, 2).build();
    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    gnodes.alloc(GNode {
        lo: 0,
        hi: 1,
        sum: 0,
        own: 0,
        left: None,
        right: None,
        parent: None,
        entry: None,
    });

    assert!(
        !violations.is_empty(),
        "should have violations after spiking alternating groups",
    );

    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(
        find_violated_nodes(&vnodes).is_empty(),
        "all violations should be resolved after rebalance",
    );
}
