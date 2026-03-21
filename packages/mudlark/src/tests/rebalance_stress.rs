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
//!
//! # Test index
//!
//! ## Baselines
//!
//! | Test | Focus |
//! |------|-------|
//! | [`uniform_tree_has_no_violations`] | all leaves same intensity — zero violations |
//! | [`all_spiked_uniform_has_no_violations`] | every leaf spiked to same value — no contrast |
//!
//! ## Cascading rebalance
//!
//! | Test | Focus |
//! |------|-------|
//! | [`large_violation_cascade_exceeds_fixed_10k_limit`] | depth-13, spike 2/4 at 10 000 |
//! | [`moderate_cascade_alternating_groups_spiked`] | depth-10, spike 2/4 at 5 000 |
//! | [`shallow_tree_rebalances_cleanly`] | depth-4, small tree near proportional-limit floor |
//!
//! ## Spike-ratio variations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`high_spike_ratio_three_of_four_no_violations`] | depth-10, spike 3/4 — no contrast |
//! | [`clustered_six_of_eight_creates_violations`] | depth-10, spike 6/8 — clustered spikes |
//! | [`single_spike_in_group_of_four`] | depth-10, spike 1/4 — sparse violations |

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::rebalance::{find_violated_nodes, rebalance};
use crate::tests::init_tracing;
use crate::tests::spiked_vtree::SpikedVTree;

// ── Helpers ─────────────────────────────────────────────────────

/// Create a minimal gnodes arena with a single dummy terminal G-node
/// at index 0, matching the dummy `GNodeId` used by [`SpikedVTree`].
fn make_dummy_gnodes() -> Arena<GNode<u64, u64>> {
    let mut gnodes = Arena::new();
    gnodes.alloc(GNode::default());
    gnodes
}

/// Assert that `rebalance` resolves all violations and preserves the
/// root's total intensity.
fn assert_clean_rebalance(
    vnodes: &mut Arena<crate::vnode::VNode<u64>>,
    root: crate::handle::VNodeId,
    violations: &mut Vec<crate::handle::VNodeId>,
) {
    let mut gnodes = make_dummy_gnodes();
    let root_intensity_before = vnodes.get(root.index()).intensity;

    rebalance(vnodes, &mut gnodes, violations, u32::MAX);

    assert!(
        find_violated_nodes(vnodes).is_empty(),
        "all violations should be resolved after rebalance",
    );
    assert_eq!(
        vnodes.get(root.index()).intensity,
        root_intensity_before,
        "root intensity must be preserved across rebalance",
    );
}

// ── Baselines ───────────────────────────────────────────────────

/// Uniform tree: every leaf has the same intensity → no violations
/// and rebalance is a no-op.
#[test]
fn uniform_tree_has_no_violations() {
    let _t = init_tracing();

    let (vnodes, _root, violations) = SpikedVTree::balanced(8)
        .base(1u64)
        .spike(1u64, 1, 0) // spike count 0 → no spikes
        .build();

    assert!(
        violations.is_empty(),
        "uniform tree should produce zero violations, got {}",
        violations.len(),
    );
    assert!(
        find_violated_nodes(&vnodes).is_empty(),
        "independent scan should also find zero violations",
    );
}

/// Every leaf spiked to the same value → uniform intensity → no
/// violations despite the large spike value.
#[test]
fn all_spiked_uniform_has_no_violations() {
    let _t = init_tracing();

    // spike every leaf (group=1, count=1)
    let (_vnodes, _root, violations) = SpikedVTree::balanced(8).base(1u64).spike(10_000u64, 1, 1).build();

    assert!(
        violations.is_empty(),
        "all-spiked tree has uniform intensity — no violations expected, got {}",
        violations.len(),
    );
}

// ── Cascading rebalance ─────────────────────────────────────────

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

    let (mut vnodes, root, mut violations) = SpikedVTree::balanced(13).base(1u64).spike(10_000u64, 4, 2).build();

    // Sanity: 2^13 = 8 192 leaves.
    assert_eq!(SpikedVTree::<u64>::balanced(13).leaf_count(), 1 << 13);
    assert!(
        violations.len() > 2_000,
        "expected >2 000 initial violations, got {}",
        violations.len(),
    );

    // With the old MAX_ITERATIONS = 10_000 constant this would
    // have panicked.  The proportional limit handles it.
    assert_clean_rebalance(&mut vnodes, root, &mut violations);
}

/// Smaller tree (depth 10, 1 024 entries) with alternating groups
/// spiked — verifies the cascade shape is correct even at moderate
/// scale.
#[test]
fn moderate_cascade_alternating_groups_spiked() {
    let _t = init_tracing();

    let (mut vnodes, root, mut violations) = SpikedVTree::balanced(10).base(1u64).spike(5_000u64, 4, 2).build();

    assert!(
        !violations.is_empty(),
        "should have violations after spiking alternating groups",
    );

    assert_clean_rebalance(&mut vnodes, root, &mut violations);
}

/// Shallow tree (depth 4, only 16 entries): ensures correct
/// behaviour near the proportional-limit floor (10 000).
#[test]
fn shallow_tree_rebalances_cleanly() {
    let _t = init_tracing();

    let (mut vnodes, root, mut violations) = SpikedVTree::balanced(4).base(1u64).spike(10_000u64, 4, 2).build();

    assert!(!violations.is_empty(), "shallow spiked tree should have violations");

    assert_clean_rebalance(&mut vnodes, root, &mut violations);
}

// ── Spike-ratio variations ──────────────────────────────────────

/// High spike ratio (3/4): with only one low-intensity leaf per
/// group the uncle subtrees all carry enough intensity that no entry
/// exceeds its uncle.  This is a valuable boundary case — aggressive
/// spiking does NOT always produce violations.
#[test]
fn high_spike_ratio_three_of_four_no_violations() {
    let _t = init_tracing();

    let (vnodes, _root, violations) = SpikedVTree::balanced(10).base(1u64).spike(10_000u64, 4, 3).build();

    assert!(
        violations.is_empty(),
        "3/4-spiked tree should have no violations (uncle intensity \
         is always >= any single entry), got {}",
        violations.len(),
    );
    assert!(find_violated_nodes(&vnodes).is_empty());
}

/// Clustered 6-of-8: the first 6 of every 8 leaves are spiked.
/// The final pair `(1, 1)` in each octet creates a low-intensity
/// subtree uncle'd against high-intensity subtrees → violations.
#[test]
fn clustered_six_of_eight_creates_violations() {
    let _t = init_tracing();

    let (mut vnodes, root, mut violations) = SpikedVTree::balanced(10).base(1u64).spike(10_000u64, 8, 6).build();

    assert!(!violations.is_empty(), "6/8-clustered spike should produce violations");

    assert_clean_rebalance(&mut vnodes, root, &mut violations);
}

/// Sparse spiking: only 1 of every 4 leaves is spiked — fewer
/// violations, but each spiked leaf is heavily outnumbered by its
/// low-intensity siblings and uncles.
#[test]
fn single_spike_in_group_of_four() {
    let _t = init_tracing();

    let (mut vnodes, root, mut violations) = SpikedVTree::balanced(10).base(1u64).spike(10_000u64, 4, 1).build();

    assert!(!violations.is_empty(), "1/4-spiked tree should have violations");

    assert_clean_rebalance(&mut vnodes, root, &mut violations);
}
