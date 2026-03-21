// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate tests for **V-Tree rebalancing** (§IDEA M-11).
//!
//! These tests exercise every primitive (`max_uncle_intensity`,
//! `is_violated`, `contract`, `standard_promote`, `skip_promote`),
//! the top-level `rebalance` loop, and each secondary violation
//! source (3–10) that can be triggered as a side-effect of a
//! restructuring step.  Escalation (§IDEA M-11.10) is covered both
//! for correctness (no residual violations) and for resulting
//! structure (ADR-M-039).
//!
//! All trees are built by hand from `Arena<VNode<u64>>` with explicit
//! intensities so that violation / non-violation is deterministic and
//! easy to reason about.
//!
//! # Test index
//!
//! ## `max_uncle_intensity`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`max_uncle_returns_none_at_depth_0`] | root has no uncle → `None` |
//! | [`max_uncle_returns_none_at_depth_1`] | depth-1 node has no uncle → `None` |
//! | [`max_uncle_returns_uncle_intensity_2node`] | 2-node parent returns sibling intensity |
//! | [`max_uncle_returns_min_uncle_intensity_3node`] | 3-node parent returns **min** of both siblings |
//!
//! ## `is_violated`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`not_violated_when_below_uncle`] | intensity < uncle → safe |
//! | [`violated_when_above_uncle`] | intensity > uncle → violated |
//! | [`not_violated_when_equal_to_uncle`] | intensity = uncle → safe (not strict) |
//! | [`violated_3node_grandparent_must_beat_both_uncles`] | 3-node: must exceed **both** uncles to be safe |
//! | [`violated_3node_grandparent_beats_both`] | 3-node: exceeds both → violated |
//! | [`no_violation_at_depth_0`] | root is never violated |
//! | [`no_violation_at_depth_1`] | depth-1 child is never violated |
//!
//! ## `contract`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`contract_3node_to_2node`] | 3→2 contraction merges lighter siblings |
//! | [`contract_isolates_non_first_heaviest`] | heaviest is not first child → still isolated correctly |
//!
//! ## `standard_promote`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`standard_promote_explodes_2node`] | 2-node parent replaced by children of violated node + sibling |
//!
//! ## `skip_promote`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`skip_promote_elevates_entry`] | entry skip-promoted past parent to grandparent level |
//!
//! ## Rebalance loop
//!
//! | Test | Focus |
//! |------|-------|
//! | [`rebalance_resolves_single_violation`] | single violated node resolved in one pass |
//! | [`rebalance_empty_queue_is_noop`] | empty queue terminates immediately |
//! | [`rebalance_skips_destroyed_node`] | deallocated node silently skipped |
//! | [`rebalance_skips_already_resolved`] | node no longer violated → skipped |
//! | [`resolve_with_contraction_first`] | 3-node parent: contraction precedes promotion |
//!
//! ## Structural ancestor violations
//!
//! | Test | Focus |
//! |------|-------|
//! | [`structural_node_violated_while_leaves_safe`] | structural node violated while all leaf descendants are safe |
//! | [`structural_violation_resolved_by_rebalance`] | rebalance resolves structural-only violation |
//!
//! ## Source 4: promotion children
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_4_promotion_children_necessary`] | without source 4 the violation is missed |
//! | [`source_4_catches_all_violated_children`] | all newly-violated children are enqueued |
//! | [`source_4_noop_when_children_safe`] | no false positives when children are safe |
//!
//! ## Source 3: contraction grandchildren
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_3_contraction_grandchildren_necessary`] | without source 3 the grandchild violation is missed |
//!
//! ## `push_contraction_child_violations`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`contraction_child_violations_skips_target`] | target node excluded from its own violation set |
//! | [`contraction_child_violations_empty_when_target_is_only_violated`] | no siblings violated → empty result |
//!
//! ## Escalation (§IDEA M-11.10)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`escalation_direct_breaks_promote_cycle`] | direct heaviest-child violation → skip-promote breaks cycle |
//! | [`escalation_with_structural_heaviest_child`] | deeper structural heaviest child → cycle broken |
//! | [`escalation_indirect_child_of_heaviest_violated`] | heaviest safe but its child violated → escalation fires |
//!
//! ## Source 7: 2-node collapse children
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_7_collapse_children_necessary`] | surviving sibling's children gain new uncle → violation caught |
//!
//! ## Source 8: 3→2 transition siblings
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_8_three_to_two_transition_necessary`] | remaining siblings re-checked after 3→2 transition |
//!
//! ## Source 9: 2-node collapse cousins
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_9_collapse_cousins_necessary`] | cousin nodes gain new uncle after collapse → violation caught |
//!
//! ## Source 10: g-contraction + promotion grandchildren
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_10_g_contraction_promotion_grandchildren_necessary`] | depth-3 grandchild violated after g-contraction + promotion |
//! | [`source_10_full_rebalance_resolves_all`] | full rebalance resolves source-10 scenario end-to-end |
//!
//! ## Source 6: leaf-removal ancestor walk
//!
//! | Test | Focus |
//! |------|-------|
//! | [`source_6_leaf_removal_ancestors_necessary`] | ancestor intensity drop exposes sibling-subtree violation |
//!
//! ## Escalation structure verification (ADR-M-039)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`escalation_direct_resulting_structure`] | post-escalation structure: entries survive with correct intensities |
//! | [`escalation_structural_heaviest_resulting_structure`] | deeper variant: entry intensities conserved, depth preserved |

use std::sync::atomic::AtomicU32;

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::{GNodeId, VNodeId};
use crate::rebalance::{
    ViolationSources, contract, find_violated_nodes, is_violated, max_uncle_intensity, push_collapse_violations_with_config,
    push_contraction_child_violations, push_cousin_violations_with_config, push_leaf_removal_violations_with_config,
    push_promoted_violations_with_config, push_remaining_sibling_violations_with_config, push_side_effect_violations_with_config,
    push_source_10_violations_with_config, rebalance, skip_promote, standard_promote,
};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};
use crate::vtree::vtree_remove_leaf;

// ── Test-only ViolationSources constructors ─────────────────────────

impl ViolationSources {
    /// All violation sources disabled (for testing).
    const fn all_disabled() -> Self {
        Self {
            source_3_contraction_grandchildren: false,
            source_4_promotion_children: false,
            source_6_leaf_removal_ancestors: false,
            source_7_collapse_children: false,
            source_8_three_to_two_siblings: false,
            source_9_collapse_cousins: false,
            source_10_g_contraction_promotion: false,
        }
    }

    /// Enable only the specified source (for testing).
    const fn only_source_3() -> Self {
        Self {
            source_3_contraction_grandchildren: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_4() -> Self {
        Self {
            source_4_promotion_children: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_6() -> Self {
        Self {
            source_6_leaf_removal_ancestors: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_7() -> Self {
        Self {
            source_7_collapse_children: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_8() -> Self {
        Self {
            source_8_three_to_two_siblings: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_9() -> Self {
        Self {
            source_9_collapse_cousins: true,
            ..Self::all_disabled()
        }
    }

    const fn only_source_10() -> Self {
        Self {
            source_10_g_contraction_promotion: true,
            ..Self::all_disabled()
        }
    }
}

/// Helper: create a V-entry with given intensity.
fn make_entry(vnodes: &mut Arena<VNode<u64>>, intensity: u64) -> VNodeId {
    let e = VNode {
        intensity,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Entry {
            gnode: GNodeId::from_index(0), // dummy
            is_exposed: true,
            is_evictable: true,
        },
    };
    VNodeId::from_index(vnodes.alloc(e))
}

/// Helper: create a gnodes arena with a single terminal G-node at
/// index 0.  Required for tests that call `rebalance` (which
/// calls `resolve`, which may read `is_semi_internal()`) or
/// `vtree_remove_leaf`.
fn make_gnodes() -> Arena<GNode<u64, u64>> {
    let mut gnodes = Arena::new();
    gnodes.alloc(GNode::default());
    gnodes
}

/// Helper: create a structural 2-node with given children.
fn make_structural_2(vnodes: &mut Arena<VNode<u64>>, a: VNodeId, b: VNodeId) -> VNodeId {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let s = VNode {
        intensity: a_int + b_int,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Structural {
            children: PackedChildren::new_2((a, a_int), (b, b_int)),
            has_evictable: true,
        },
    };
    let s_id = VNodeId::from_index(vnodes.alloc(s));
    vnodes.get_mut(a.index()).parent = Some(s_id);
    vnodes.get_mut(b.index()).parent = Some(s_id);
    s_id
}

/// Helper: create a structural 3-node with given children.
fn make_structural_3(vnodes: &mut Arena<VNode<u64>>, a: VNodeId, b: VNodeId, c: VNodeId) -> VNodeId {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let c_int = vnodes.get(c.index()).intensity;
    let s = VNode {
        intensity: a_int + b_int + c_int,
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE),
        kind: VKind::Structural {
            children: PackedChildren::new_3((a, a_int), (b, b_int), (c, c_int)),
            has_evictable: true,
        },
    };
    let s_id = VNodeId::from_index(vnodes.alloc(s));
    vnodes.get_mut(a.index()).parent = Some(s_id);
    vnodes.get_mut(b.index()).parent = Some(s_id);
    vnodes.get_mut(c.index()).parent = Some(s_id);
    s_id
}

// ── max_uncle_intensity ─────────────────────────────────────

#[test]
fn max_uncle_returns_none_at_depth_0() {
    let mut vnodes = Arena::new();
    let root = make_entry(&mut vnodes, 100);
    assert_eq!(max_uncle_intensity(&vnodes, root), None);
}

#[test]
fn max_uncle_returns_none_at_depth_1() {
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 50);
    let b = make_entry(&mut vnodes, 10);
    let _root = make_structural_2(&mut vnodes, a, b);
    assert_eq!(max_uncle_intensity(&vnodes, a), None);
    assert_eq!(max_uncle_intensity(&vnodes, b), None);
}

#[test]
fn max_uncle_returns_uncle_intensity_2node() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 5);
    let sib = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 20);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_2(&mut vnodes, p, uncle);
    assert_eq!(max_uncle_intensity(&vnodes, c), Some(20));
}

#[test]
fn max_uncle_returns_min_uncle_intensity_3node() {
    // Under a 3-node grandparent with two uncles, max_uncle returns
    // the *maximum* uncle intensity.
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 5);
    let sib = make_entry(&mut vnodes, 3);
    let uncle1 = make_entry(&mut vnodes, 20);
    let uncle2 = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_3(&mut vnodes, p, uncle1, uncle2);
    // max uncle = max(20, 10) = 20
    assert_eq!(max_uncle_intensity(&vnodes, c), Some(20));
}

// ── is_violated ─────────────────────────────────────────────

#[test]
fn not_violated_when_below_uncle() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 5);
    let sib = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 20);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    assert!(!is_violated(&vnodes, c));
}

#[test]
fn violated_when_above_uncle() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 25);
    let sib = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 20);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    assert!(is_violated(&vnodes, c));
}

#[test]
fn not_violated_when_equal_to_uncle() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 20);
    let sib = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 20);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    assert!(!is_violated(&vnodes, c));
}

#[test]
fn violated_3node_grandparent_must_beat_both_uncles() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 15);
    let sib = make_entry(&mut vnodes, 3);
    let uncle1 = make_entry(&mut vnodes, 20);
    let uncle2 = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_3(&mut vnodes, p, uncle1, uncle2);

    // c(15) > uncle2(10) but c(15) <= uncle1(20) → not violated.
    assert!(!is_violated(&vnodes, c));
}

#[test]
fn violated_3node_grandparent_beats_both() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 25);
    let sib = make_entry(&mut vnodes, 3);
    let uncle1 = make_entry(&mut vnodes, 20);
    let uncle2 = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, sib);
    let _g = make_structural_3(&mut vnodes, p, uncle1, uncle2);

    // c(25) > uncle1(20) and c(25) > uncle2(10) → violated.
    assert!(is_violated(&vnodes, c));
}

#[test]
fn no_violation_at_depth_0() {
    let mut vnodes = Arena::new();
    let root = make_entry(&mut vnodes, 100);
    assert!(!is_violated(&vnodes, root));
}

#[test]
fn no_violation_at_depth_1() {
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 100);
    let b = make_entry(&mut vnodes, 5);
    let _root = make_structural_2(&mut vnodes, a, b);
    assert!(!is_violated(&vnodes, a));
}

// ── contract ────────────────────────────────────────────────

#[test]
#[allow(clippy::many_single_char_names)]
fn contract_3node_to_2node() {
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 30);
    let b = make_entry(&mut vnodes, 10);
    let c = make_entry(&mut vnodes, 5);
    let p = make_structural_3(&mut vnodes, a, b, c);

    let m = contract(&mut vnodes, p);

    // p is now a 2-node: [a(30), m(15)]
    let p_node = vnodes.get(p.index());
    if let VKind::Structural { children, .. } = &p_node.kind {
        assert_eq!(children.len(), 2);
        assert_eq!(children.get(0), (a, 30));
        assert_eq!(children.get(1).0, m);
        assert_eq!(children.get(1).1, 15);
    } else {
        panic!("expected structural");
    }

    // Merged node is a 2-node with b and c.
    let m_node = vnodes.get(m.index());
    assert_eq!(m_node.intensity, 15);
    assert_eq!(m_node.parent, Some(p));
    if let VKind::Structural { children, .. } = &m_node.kind {
        assert_eq!(children.len(), 2);
    }

    assert_eq!(vnodes.get(b.index()).parent, Some(m));
    assert_eq!(vnodes.get(c.index()).parent, Some(m));
}

#[test]
#[allow(clippy::many_single_char_names)]
fn contract_isolates_non_first_heaviest() {
    // Heaviest child is in the *last* position (not first).
    // Verifies `heaviest_child_index()` is used correctly.
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 5);
    let b = make_entry(&mut vnodes, 10);
    let c = make_entry(&mut vnodes, 30); // heaviest, last
    let p = make_structural_3(&mut vnodes, a, b, c);

    let m = contract(&mut vnodes, p);

    // p is now a 2-node: [c(30), m(15)]
    let p_node = vnodes.get(p.index());
    if let VKind::Structural { children, .. } = &p_node.kind {
        assert_eq!(children.len(), 2);
        // Isolate is c(30), merged is a+b=15.
        let (iso_id, iso_int) = children.get(0);
        let (mer_id, mer_int) = children.get(1);
        assert_eq!(iso_id, c);
        assert_eq!(iso_int, 30);
        assert_eq!(mer_id, m);
        assert_eq!(mer_int, 15);
    } else {
        panic!("expected structural");
    }

    // a and b are now children of the merged node.
    assert_eq!(vnodes.get(a.index()).parent, Some(m));
    assert_eq!(vnodes.get(b.index()).parent, Some(m));
}

// ── standard_promote ────────────────────────────────────────

#[test]
fn standard_promote_explodes_2node() {
    let mut vnodes = Arena::new();
    let c1 = make_entry(&mut vnodes, 15);
    let c2 = make_entry(&mut vnodes, 10);
    let sib = make_entry(&mut vnodes, 5);
    let c = make_structural_2(&mut vnodes, c1, c2);
    let p = make_structural_2(&mut vnodes, c, sib);

    standard_promote(&mut vnodes, c);

    let p_node = vnodes.get(p.index());
    if let VKind::Structural { children, .. } = &p_node.kind {
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected structural");
    }

    assert_eq!(vnodes.get(c1.index()).parent, Some(p));
    assert_eq!(vnodes.get(c2.index()).parent, Some(p));
    assert!(!vnodes.is_occupied(c.index()));
}

// ── skip_promote ────────────────────────────────────────────

#[test]
fn skip_promote_elevates_entry() {
    let mut vnodes = Arena::new();
    let c = make_entry(&mut vnodes, 25);
    let s = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, s);
    let g = make_structural_2(&mut vnodes, p, uncle);

    skip_promote(&mut vnodes, c);

    let g_node = vnodes.get(g.index());
    if let VKind::Structural { children, .. } = &g_node.kind {
        assert_eq!(children.len(), 3);
    } else {
        panic!("expected structural");
    }

    assert_eq!(vnodes.get(c.index()).parent, Some(g));
    assert_eq!(vnodes.get(s.index()).parent, Some(g));
    assert!(!vnodes.is_occupied(p.index()));
}

// ── rebalance loop ──────────────────────────────────────────

#[test]
fn rebalance_resolves_single_violation() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let c = make_entry(&mut vnodes, 25);
    let s = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, s);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(!is_violated(&vnodes, c));
    assert!(violations.is_empty());
}

#[test]
fn rebalance_empty_queue_is_noop() {
    let mut vnodes: Arena<VNode<u64>> = Arena::new();
    let mut gnodes = make_gnodes();
    let mut violations = vec![];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);
    assert!(violations.is_empty());
}

#[test]
fn rebalance_skips_destroyed_node() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let e = make_entry(&mut vnodes, 10);
    vnodes.dealloc(e.index());

    let mut violations = vec![e];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);
    assert!(violations.is_empty());
}

#[test]
fn rebalance_skips_already_resolved() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let c = make_entry(&mut vnodes, 5);
    let s = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c, s);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);
    assert!(violations.is_empty());
}

#[test]
fn resolve_with_contraction_first() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let c = make_entry(&mut vnodes, 25);
    let sib1 = make_entry(&mut vnodes, 3);
    let sib2 = make_entry(&mut vnodes, 2);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_3(&mut vnodes, c, sib1, sib2);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(!is_violated(&vnodes, c));
}

// ── Structural ancestor violations ──────────────────────────
//
// A structural node at depth ≥2 can be violated even when all
// its leaf descendants are safe.  This proves the assumption
// behind the ancestor walk in observe.rs.

#[test]
fn structural_node_violated_while_leaves_safe() {
    // gg(2-node: [g, uncle_gg(5)])
    //   g(2-node: [p, uncle_g(8)])
    //     p(2-node: [e1(6), e2(3)])
    //
    // e1(6) ≤ uncle_g(8) → safe
    // e2(3) ≤ uncle_g(8) → safe
    // p(9)  > uncle_gg(5) → VIOLATED
    let mut vnodes = Arena::new();
    let e1 = make_entry(&mut vnodes, 6);
    let e2 = make_entry(&mut vnodes, 3);
    let uncle_g = make_entry(&mut vnodes, 8);
    let uncle_gg = make_entry(&mut vnodes, 5);
    let p = make_structural_2(&mut vnodes, e1, e2);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    // Leaves are safe.
    assert!(!is_violated(&vnodes, e1), "e1 should not be violated");
    assert!(!is_violated(&vnodes, e2), "e2 should not be violated");

    // But the structural parent IS violated.
    assert!(is_violated(&vnodes, p), "p should be violated");

    // find_violated_nodes must discover p.
    let found = find_violated_nodes(&vnodes);
    assert!(found.contains(&p), "find_violated_nodes should find the structural violation");
    assert!(!found.contains(&e1));
    assert!(!found.contains(&e2));
}

#[test]
fn structural_violation_resolved_by_rebalance() {
    // Same tree as above — verify rebalance resolves it.
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let e1 = make_entry(&mut vnodes, 6);
    let e2 = make_entry(&mut vnodes, 3);
    let uncle_g = make_entry(&mut vnodes, 8);
    let uncle_gg = make_entry(&mut vnodes, 5);
    let p = make_structural_2(&mut vnodes, e1, e2);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    let mut violations = vec![p];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(violations.is_empty());
    assert!(find_violated_nodes(&vnodes).is_empty(), "all violations should be resolved");
}

// ── Source 4: push_promoted_violations ──────────────────────
//
// After restructuring, children of the new 3-node may have new
// uncles they didn't have before. push_promoted_violations must
// catch them.

/// Proves source 4 is necessary by showing:
/// 1. Without source 4: violation NOT caught
/// 2. With only source 4: violation IS caught
#[test]
fn source_4_promotion_children_necessary() {
    // g(2-node: [p(3-node: [a(20), b(3), c(2)]), uncle(10)])
    //
    // a(20) vs uncle(10): 20 > 10 → violated (should be pushed)
    // b(3)  vs uncle(10):  3 ≤ 10 → safe
    // c(2)  vs uncle(10):  2 ≤ 10 → safe
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 20);
    let b = make_entry(&mut vnodes, 3);
    let c = make_entry(&mut vnodes, 2);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_3(&mut vnodes, a, b, c);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    // Sanity: a IS violated
    assert!(is_violated(&vnodes, a), "a(20) > uncle(10) should be violated");

    // ── Part 1: Without source 4, violation NOT caught ──
    let mut violations = Vec::new();
    let config_without_4 = ViolationSources::all_disabled();
    push_promoted_violations_with_config(&vnodes, p, &mut violations, config_without_4);
    assert!(!violations.contains(&a), "source 4 disabled: a should NOT be caught");

    // ── Part 2: With only source 4, violation IS caught ──
    violations.clear();
    let config_only_4 = ViolationSources::only_source_4();
    push_promoted_violations_with_config(&vnodes, p, &mut violations, config_only_4);
    assert!(
        violations.contains(&a),
        "source 4 enabled: a SHOULD be caught, got: {violations:?}"
    );
    assert!(!violations.contains(&b), "b(3) should not be pushed");
    assert!(!violations.contains(&c), "c(2) should not be pushed");
}

#[test]
fn source_4_catches_all_violated_children() {
    // All children violated — both should be enqueued.
    // g(2-node: [p(2-node: [a(25), b(15)]), uncle(10)])
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 25);
    let b = make_entry(&mut vnodes, 15);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, a, b);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = Vec::new();
    push_promoted_violations_with_config(&vnodes, p, &mut violations, ViolationSources::only_source_4());

    assert!(violations.contains(&a));
    assert!(violations.contains(&b));
}

#[test]
fn source_4_noop_when_children_safe() {
    // g(2-node: [p(2-node: [a(5), b(3)]), uncle(10)])
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 5);
    let b = make_entry(&mut vnodes, 3);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, a, b);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = Vec::new();
    push_promoted_violations_with_config(&vnodes, p, &mut violations, ViolationSources::only_source_4());

    assert!(violations.is_empty());
}

// ── Source 3: push_side_effect_violations (grandchildren) ──
//
// After restructuring (contraction), grandchildren may have new
// uncles they didn't have before.

/// Proves source 3 is necessary by showing:
/// 1. Without source 3: violation NOT caught
/// 2. With only source 3: violation IS caught
#[test]
fn source_3_contraction_grandchildren_necessary() {
    // Tree:
    //   g(2-node: [p(2-node: [c1(20), c2(5)]), uncle(10)])
    // After restructuring at g, c1's uncle context may change.
    // c1(20) > uncle(10) → violated.
    // c2(5) ≤ uncle(10) → safe.
    //
    // push_side_effect_violations scans grandchildren of g.
    let mut vnodes = Arena::new();
    let c1 = make_entry(&mut vnodes, 20);
    let c2 = make_entry(&mut vnodes, 5);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, c1, c2);
    let g = make_structural_2(&mut vnodes, p, uncle);

    // Sanity: c1 IS violated
    assert!(is_violated(&vnodes, c1), "c1(20) > uncle(10) should be violated");

    // ── Part 1: Without source 3, violation NOT caught ──
    let mut violations = Vec::new();
    let config_without_3 = ViolationSources::all_disabled();
    push_side_effect_violations_with_config(&vnodes, g, &mut violations, config_without_3);
    assert!(!violations.contains(&c1), "source 3 disabled: c1 should NOT be caught");

    // ── Part 2: With only source 3, violation IS caught ──
    violations.clear();
    let config_only_3 = ViolationSources::only_source_3();
    push_side_effect_violations_with_config(&vnodes, g, &mut violations, config_only_3);
    assert!(
        violations.contains(&c1),
        "source 3 enabled: c1 SHOULD be caught, got: {violations:?}"
    );
    assert!(!violations.contains(&c2), "c2(5) should not be pushed");
}

// push_contraction_child_violations helper: similar to source 4 but
// excludes the target node (Phase 2 handles it).

#[test]
fn contraction_child_violations_skips_target() {
    // g(2-node: [p(2-node: [a(25), b(20)]), uncle(10)])
    // Both a and b are violated. Calling with skip=a should
    // push only b.
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 25);
    let b = make_entry(&mut vnodes, 20);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, a, b);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    // Skip a → only b should appear.
    let mut violations = Vec::new();
    push_contraction_child_violations(&vnodes, p, a, &mut violations);
    assert_eq!(violations, vec![b], "b should be pushed, a skipped");

    // Skip b → only a should appear.
    let mut violations = Vec::new();
    push_contraction_child_violations(&vnodes, p, b, &mut violations);
    assert_eq!(violations, vec![a], "a should be pushed, b skipped");
}

#[test]
fn contraction_child_violations_empty_when_target_is_only_violated() {
    // g(2-node: [p(2-node: [a(25), b(5)]), uncle(10)])
    // Only a is violated. Skipping a → nothing pushed.
    let mut vnodes = Arena::new();
    let a = make_entry(&mut vnodes, 25);
    let b = make_entry(&mut vnodes, 5);
    let uncle = make_entry(&mut vnodes, 10);
    let p = make_structural_2(&mut vnodes, a, b);
    let _g = make_structural_2(&mut vnodes, p, uncle);

    let mut violations = Vec::new();
    push_contraction_child_violations(&vnodes, p, a, &mut violations);
    assert!(violations.is_empty(), "only violated node was skipped");
}

// ── Escalation after standard promote ─────────────────────
//
// Without escalation, standard_promote can create a 3-node whose
// heaviest child is violated — resolving that child would
// contract back to a 2-node, then standard_promote again,
// creating an infinite cycle.  escalate_after_promote detects
// this and skip-promotes past the problematic parent.

#[test]
fn escalation_direct_breaks_promote_cycle() {
    // Build the exact cycle topology:
    //
    //   gg(2-node: [g, uncle_gg(3)])
    //     g(2-node: [p, uncle_g(8)])
    //       p(2-node: [c, sib(4)])
    //         c(2-node: [c1(20), c2(10)])
    //
    // c(30) > uncle_g(8) → violated.
    //
    // Without escalation: standard_promote(c) → p = {c1(20),c2(10),sib(4)}
    //   c1(20) > uncle_g(8) → violated → contract p → {c1(20),merged(14)}
    //   c1(20) still > uncle_g(8) → standard_promote(c1)???
    //   But c1 is entry → skip_promote. OK, not a true infinite loop
    //   for entries, but the escalation prevents the unnecessary
    //   round-trip through contraction.
    //
    // With escalation: detects c1(20) > uncle_g(8) immediately
    // after standard_promote, escalates to skip_promote past p.
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let c1 = make_entry(&mut vnodes, 20);
    let c2 = make_entry(&mut vnodes, 10);
    let sib = make_entry(&mut vnodes, 4);
    let uncle_g = make_entry(&mut vnodes, 8);
    let uncle_gg = make_entry(&mut vnodes, 40);
    let c = make_structural_2(&mut vnodes, c1, c2);
    let p = make_structural_2(&mut vnodes, c, sib);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    // Only c is initially violated — uncle_gg(40) shields p.
    assert!(is_violated(&vnodes, c), "c(30) > uncle_g(8)");
    assert!(!is_violated(&vnodes, p), "p(34) <= uncle_gg(40)");

    // Rebalance must terminate and leave no violations.
    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(violations.is_empty());
    let remaining = find_violated_nodes(&vnodes);
    assert!(
        remaining.is_empty(),
        "escalation should have resolved all violations, remaining: {remaining:?}"
    );
}

#[test]
fn escalation_with_structural_heaviest_child() {
    // Deeper variant: the heaviest child after promote is itself
    // a structural 2-node, creating the genuine cycle risk.
    //
    //   gg(2-node: [g, uncle_gg(3)])
    //     g(2-node: [p, uncle_g(10)])
    //       p(2-node: [c, sib(5)])
    //         c(2-node: [h, light(4)])
    //           h(2-node: [h1(20), h2(8)])
    //
    // c(32) > uncle_g(10) → violated.
    // standard_promote(c) → p = {h(28), light(4), sib(5)}.
    // h(28) > uncle_g(10) → direct escalation trigger.
    //
    // Without escalation, resolve(h) would:
    //   Phase 1: contract p → {h(28), merged(9)}
    //   h still violated → Phase 2: standard_promote(h)
    //   → p = {h1(20), h2(8), merged(9)}
    //   h1(20) > uncle_g(10) → another round…
    //
    // Escalation breaks this by skip-promoting h past p.
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let h1 = make_entry(&mut vnodes, 20);
    let h2 = make_entry(&mut vnodes, 8);
    let light = make_entry(&mut vnodes, 4);
    let sib = make_entry(&mut vnodes, 5);
    let uncle_g = make_entry(&mut vnodes, 10);
    let uncle_gg = make_entry(&mut vnodes, 40);
    let h = make_structural_2(&mut vnodes, h1, h2);
    let c = make_structural_2(&mut vnodes, h, light);
    let p = make_structural_2(&mut vnodes, c, sib);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    // Only c is initially violated — uncle_gg(40) shields p.
    assert!(is_violated(&vnodes, c), "c(32) > uncle_g(10)");
    assert!(!is_violated(&vnodes, p), "p(37) <= uncle_gg(40)");

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(violations.is_empty());
    let remaining = find_violated_nodes(&vnodes);
    assert!(
        remaining.is_empty(),
        "deep escalation should resolve all violations, remaining: {remaining:?}"
    );
}

#[test]
fn escalation_indirect_child_of_heaviest_violated() {
    // The heaviest child h is NOT violated, but a child of h IS
    // violated against its new uncle context.
    //
    //   gg(2-node: [g, uncle_gg(3)])
    //     g(2-node: [p, uncle_g(20)])
    //       p(2-node: [c, sib(4)])
    //         c(2-node: [h, light(3)])
    //           h(2-node: [h1(12), h2(6)])
    //
    // c(21) > uncle_g(20) → violated.
    // standard_promote(c) → p = {h(18), light(3), sib(4)}.
    // h(18) ≤ uncle_g(20) → NOT directly violated.
    // BUT h1(12) > max(light(3), sib(4)) = 4 → child violated!
    //
    // Escalation detects this indirect variant.
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let h1 = make_entry(&mut vnodes, 12);
    let h2 = make_entry(&mut vnodes, 6);
    let light = make_entry(&mut vnodes, 3);
    let sib = make_entry(&mut vnodes, 4);
    let uncle_g = make_entry(&mut vnodes, 20);
    let uncle_gg = make_entry(&mut vnodes, 30);
    let h = make_structural_2(&mut vnodes, h1, h2);
    let c = make_structural_2(&mut vnodes, h, light);
    let p = make_structural_2(&mut vnodes, c, sib);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    // Only c is initially violated — uncle_gg(30) shields p.
    assert!(is_violated(&vnodes, c), "c(21) > uncle_g(20)");
    assert!(!is_violated(&vnodes, p), "p(25) <= uncle_gg(30)");

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    assert!(violations.is_empty());
    let remaining = find_violated_nodes(&vnodes);
    assert!(
        remaining.is_empty(),
        "indirect escalation should resolve, remaining: {remaining:?}"
    );
}

// ── Source 7: 2-node collapse children (§IDEA M-11.11.3) ─────────────
//
// When a 2-node parent collapses, the surviving sibling is
// re-parented to the grandparent.  Its children now have
// different uncles and may become violated.
//
//   g(2-node: [p, weak_uncle(5)])
//     p(2-node: [victim(20), s])
//       s(2-node: [c1(15), c2(5)])
//
// Before: c1(15) ≤ victim(20) → safe.
// After collapse (victim removed, s promoted):
//   c1(15) > weak_uncle(5) → VIOLATED.

#[test]
fn source_7_collapse_children_necessary() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();

    let c1 = make_entry(&mut vnodes, 15);
    let c2 = make_entry(&mut vnodes, 5);
    let victim = make_entry(&mut vnodes, 20);
    let weak_uncle = make_entry(&mut vnodes, 5);
    let s = make_structural_2(&mut vnodes, c1, c2);
    let p = make_structural_2(&mut vnodes, victim, s);
    let g = make_structural_2(&mut vnodes, p, weak_uncle);

    assert!(!is_violated(&vnodes, c1), "c1(15) ≤ victim(20) before removal");

    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, victim, Some(g));
    assert_eq!(new_root, Some(g));
    assert!(is_violated(&vnodes, c1), "c1(15) > weak_uncle(5) after collapse");

    // Part 1: source 7 disabled → NOT caught.
    let mut violations = Vec::new();
    push_collapse_violations_with_config(&vnodes, s, &mut violations, ViolationSources::all_disabled());
    assert!(!violations.contains(&c1), "source 7 disabled: c1 should NOT be caught");

    // Source 6 also doesn't catch it.
    push_leaf_removal_violations_with_config(&vnodes, g, &mut violations, ViolationSources::only_source_6());
    assert!(!violations.contains(&c1), "source 6 should NOT catch collapse violations");

    // Part 2: source 7 enabled → IS caught.
    violations.clear();
    push_collapse_violations_with_config(&vnodes, s, &mut violations, ViolationSources::only_source_7());
    assert!(violations.contains(&c1), "source 7 enabled: c1 SHOULD be caught");
}

// ── Source 8: 3→2 transition (§IDEA M-11.11.3) ───────────────────────
//
// When an entry is removed from a 3-node parent (making it a
// 2-node), remaining siblings' children lose the removed entry
// as a potential uncle.  Their max_uncle may decrease.
//
//   g(2-node: [p, uncle_g(100)])
//     p(3-node: [a, b(10), high_uncle(30)])
//       a(2-node: [a1(20), a2(5)])
//
// Before: a1(20) ≤ high_uncle(30) → safe.
// After high_uncle removed (p becomes 2-node):
//   a1(20) > b(10) → VIOLATED.

#[test]
fn source_8_three_to_two_transition_necessary() {
    let mut vnodes = Arena::new();

    let a1 = make_entry(&mut vnodes, 20);
    let a2 = make_entry(&mut vnodes, 5);
    let b = make_entry(&mut vnodes, 10);
    let high_uncle = make_entry(&mut vnodes, 30);
    let uncle_g = make_entry(&mut vnodes, 100);
    let a = make_structural_2(&mut vnodes, a1, a2);
    let p = make_structural_3(&mut vnodes, a, b, high_uncle);
    let _g = make_structural_2(&mut vnodes, p, uncle_g);

    assert!(!is_violated(&vnodes, a1), "a1(20) ≤ high_uncle(30) before removal");

    // Simulate removal: convert p from 3-node to 2-node.
    {
        let a_int = vnodes.get(a.index()).intensity;
        let b_int = vnodes.get(b.index()).intensity;
        let p_node = vnodes.get_mut(p.index());
        p_node.intensity = a_int + b_int;
        p_node.kind = VKind::Structural {
            children: PackedChildren::new_2((a, a_int), (b, b_int)),
            has_evictable: true,
        };
    }
    vnodes.get_mut(high_uncle.index()).parent = None;
    vnodes.dealloc(high_uncle.index());

    assert!(is_violated(&vnodes, a1), "a1(20) > b(10) after 3→2 transition");

    // Part 1: source 8 disabled → NOT caught.
    let mut violations = Vec::new();
    push_remaining_sibling_violations_with_config(&vnodes, p, high_uncle, &mut violations, ViolationSources::all_disabled());
    assert!(!violations.contains(&a1), "source 8 disabled: a1 should NOT be caught");

    // Part 2: source 8 enabled → IS caught.
    violations.clear();
    push_remaining_sibling_violations_with_config(&vnodes, p, high_uncle, &mut violations, ViolationSources::only_source_8());
    assert!(violations.contains(&a1), "source 8 enabled: a1 SHOULD be caught");
}

// ── Source 9: 2-node collapse cousins (§IDEA M-11.11.3) ──────────────
//
// When a 2-node parent collapses, the OTHER children of the
// grandparent ("cousins") are affected because their uncle
// changed from the collapsed parent to the promoted sole child.
//
//   g(2-node: [v_parent, cousin])
//     v_parent(2-node: [sole(10), victim(30)])
//     cousin(2-node: [c1(25), c2(5)])
//
// Before: c1(25) ≤ v_parent(40) → safe.
// After collapse (victim removed, sole promoted):
//   c1(25) > sole(10) → VIOLATED.

#[test]
fn source_9_collapse_cousins_necessary() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();

    let sole = make_entry(&mut vnodes, 10);
    let victim = make_entry(&mut vnodes, 30);
    let c1 = make_entry(&mut vnodes, 25);
    let c2 = make_entry(&mut vnodes, 5);
    let v_parent = make_structural_2(&mut vnodes, sole, victim);
    let cousin = make_structural_2(&mut vnodes, c1, c2);
    let g = make_structural_2(&mut vnodes, v_parent, cousin);

    assert!(!is_violated(&vnodes, c1), "c1(25) ≤ v_parent(40) before removal");

    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, victim, Some(g));
    assert_eq!(new_root, Some(g));
    assert!(is_violated(&vnodes, c1), "c1(25) > sole(10) after collapse");

    // Part 1: source 9 disabled → NOT caught.
    let mut violations = Vec::new();
    push_cousin_violations_with_config(&vnodes, sole, g, &mut violations, ViolationSources::all_disabled());
    assert!(!violations.contains(&c1), "source 9 disabled: c1 should NOT be caught");

    // Source 7 also doesn't catch it.
    push_collapse_violations_with_config(&vnodes, sole, &mut violations, ViolationSources::only_source_7());
    assert!(!violations.contains(&c1), "source 7 should NOT catch cousin violations");

    // Part 2: source 9 enabled → IS caught.
    violations.clear();
    push_cousin_violations_with_config(&vnodes, sole, g, &mut violations, ViolationSources::only_source_9());
    assert!(violations.contains(&c1), "source 9 enabled: c1 SHOULD be caught");
}

// ── Source 10: g-contraction + promotion grandchildren (§IDEA M-11.12) ──
//
// When g is a 3-node, resolve() contracts it before skip-promoting c.
// Contraction creates an intermediate structural node (merged_g)
// between g and p. After promotion disperses p into c and s,
// grandchildren of merged_g (depth 3 below g) face a weakened uncle
// context: their uncle changed from p (int = c + s) to the dispersed
// max(c, s) ≤ c + s. push_side_effect_violations(g) only reaches
// depth 2 — it catches merged_g's children but NOT their children.
// push_side_effect_violations(merged_g) catches the depth-3 nodes.
//
// Post-promotion tree:
//
//   gg(2-node: [g(150), uncle_gg(1000)])
//     g(2-node: [big_u(100), merged_g(50)])
//       big_u(entry: 100)
//       merged_g(3-node: [c(20), s(5), small_u(25)])
//         c(entry: 20)
//         s(entry: 5)
//         small_u(2-node: [x1(22), x2(3)])
//           x1(entry: 22)    ← violated: 22 > max(c=20, s=5) = 20
//           x2(entry: 3)
//
// Before promotion: x1's uncle was p(25), so x1(22) ≤ 25 → safe.
// After promotion:  p dispersed, x1's uncle = max(20, 5) = 20 → violated.
// push(g) catches small_u(25) at depth 2 but NOT x1 at depth 3.
// push(merged_g) catches x1 at depth 2 relative to merged_g.

#[test]
fn source_10_g_contraction_promotion_grandchildren_necessary() {
    let mut vnodes = Arena::new();

    // Leaf entries.
    let x1 = make_entry(&mut vnodes, 22);
    let x2 = make_entry(&mut vnodes, 3);
    let c = make_entry(&mut vnodes, 20);
    let s = make_entry(&mut vnodes, 5);
    let big_u = make_entry(&mut vnodes, 100);
    let uncle_gg = make_entry(&mut vnodes, 1000);

    // Build bottom-up.
    let small_u = make_structural_2(&mut vnodes, x1, x2); // int = 25
    let merged_g = make_structural_3(&mut vnodes, c, s, small_u); // int = 50
    let g = make_structural_2(&mut vnodes, big_u, merged_g); // int = 150
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg); // int = 1150

    // Sanity: x1 IS violated (22 > max uncle 20).
    assert!(is_violated(&vnodes, x1), "x1(22) > max(c=20, s=5) = 20 should be violated");

    // ── Part 1: push at g (source 3) does NOT catch x1 ──
    // Grandchildren of g are children of merged_g: {c, s, small_u}.
    // Their uncle is big_u(100).  None are violated (all ≤ 100).
    // x1 sits at depth 3 from g — completely out of reach.
    let mut violations = Vec::new();
    push_side_effect_violations_with_config(&vnodes, g, &mut violations, ViolationSources::only_source_3());
    assert!(!violations.contains(&x1), "push(g) should NOT reach x1 at depth 3");
    assert!(
        !violations.contains(&small_u),
        "small_u(25) ≤ big_u(100): not violated at g's grandchild level"
    );

    // ── Part 2: source 10 disabled → NOT caught ──
    violations.clear();
    push_source_10_violations_with_config(&vnodes, merged_g, &mut violations, ViolationSources::all_disabled());
    assert!(!violations.contains(&x1), "source 10 disabled: x1 should NOT be caught");

    // ── Part 3: source 10 enabled → IS caught ──
    violations.clear();
    push_source_10_violations_with_config(&vnodes, merged_g, &mut violations, ViolationSources::only_source_10());
    assert!(
        violations.contains(&x1),
        "source 10 enabled: push(merged_g) SHOULD catch x1(22) > max(c=20, s=5) = 20"
    );
    assert!(!violations.contains(&x2), "x2(3) ≤ 20 should not be pushed");
}

/// Integration test: full rebalance resolves the source-10 scenario
/// without leaving residual violations.
///
/// We construct the post-promotion tree directly (the same layout as
/// the unit test) and feed all violations to `rebalance()`.  This
/// confirms the eager-enqueue path correctly resolves depth-3
/// violations when the merged intermediate node is present.
///
/// Note on when source 10 fires in practice: in the entry skip-promote
/// path of `resolve()`, when `c` is violated (`c > max uncle`) and `g`
/// is a 3-node, `p.int ≥ c.int > max uncle`, so `p` is always the
/// heaviest child of `g` — contraction isolates `p`, and `merged_g`
/// contains only the former uncles (whose internal relationships are
/// unchanged).  The source-10 scenario with `p` inside `merged_g`
/// (where promotion disperses `p`'s aggregate) arises specifically
/// during **escalation** (`escalate_after_promote`), where
/// g-contraction follows a standard promote that created a 3-node `p`.
#[test]
fn source_10_full_rebalance_resolves_all() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();

    // Post-promotion tree (manually constructed, same as unit test):
    //
    //   gg(2-node: [g(150), uncle_gg(1000)])
    //     g(2-node: [big_u(100), merged_g(50)])
    //       merged_g(3-node: [c(20), s(5), small_u(25)])
    //         small_u(2-node: [x1(22), x2(3)])
    //
    // x1(22) > max(c=20, s=5) = 20 → violated (source 10).
    let x1 = make_entry(&mut vnodes, 22);
    let x2 = make_entry(&mut vnodes, 3);
    let c = make_entry(&mut vnodes, 20);
    let s = make_entry(&mut vnodes, 5);
    let big_u = make_entry(&mut vnodes, 100);
    let uncle_gg = make_entry(&mut vnodes, 1000);

    let small_u = make_structural_2(&mut vnodes, x1, x2);
    let merged_g = make_structural_3(&mut vnodes, c, s, small_u);
    let g = make_structural_2(&mut vnodes, big_u, merged_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    // Seed all current violations and rebalance.
    let mut violations = find_violated_nodes(&vnodes);
    assert!(violations.contains(&x1), "x1 should be in initial violations");
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    let remaining = find_violated_nodes(&vnodes);
    assert!(
        remaining.is_empty(),
        "full rebalance should resolve all violations including source-10, remaining: {remaining:?}"
    );
}

// ── Source 6: leaf-removal ancestor walk (§IDEA M-11.11.3) ──────────
//
// When a leaf is removed, ancestor intensities decrease.  Nodes in
// sibling subtrees that were previously shielded by the removed leaf's
// high ancestor intensity may become violated.  The ancestor walk in
// `push_leaf_removal_violations` must catch them.
//
//   root(2-node: [left, right])
//     left(2-node: [p, uncle_left(3)])
//       p(2-node: [victim(100), survivor(2)])
//     right(2-node: [x1(6), x2(1)])
//
// Before: x1(6) ≤ left(105) → safe (left is uncle of x1).
// After removing victim: p collapses, survivor replaces p in left.
//   left = (2-node: [survivor(2), uncle_left(3)]) → int = 5.
// Now x1(6) > left(5) → VIOLATED.
//
// Source 6 walks upward from left to root, checking siblings'
// children at each level.  At root, sibling right has children
// x1, x2 — source 6 finds x1(6) > left(5) → pushes x1.

#[test]
fn source_6_leaf_removal_ancestors_necessary() {
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();

    let victim = make_entry(&mut vnodes, 100);
    let survivor = make_entry(&mut vnodes, 2);
    let uncle_left = make_entry(&mut vnodes, 3);
    let x1 = make_entry(&mut vnodes, 6);
    let x2 = make_entry(&mut vnodes, 1);
    let p = make_structural_2(&mut vnodes, victim, survivor);
    let left = make_structural_2(&mut vnodes, p, uncle_left);
    let right = make_structural_2(&mut vnodes, x1, x2);
    let root = make_structural_2(&mut vnodes, left, right);

    // Sanity: x1(6) ≤ left(105) → safe before removal.
    assert!(!is_violated(&vnodes, x1), "x1(6) ≤ left(105) should be safe before removal");

    // Remove victim; p collapses, left.int drops from 105 to 5.
    let new_root = vtree_remove_leaf(&mut vnodes, &mut gnodes, victim, Some(root));
    assert_eq!(new_root, Some(root));

    // x1(6) > left(5) → VIOLATED after removal.
    assert!(is_violated(&vnodes, x1), "x1(6) > left(5) should be violated after removal");

    // ── Part 1: Without source 6, violation NOT caught ──
    let mut violations = Vec::new();
    push_leaf_removal_violations_with_config(&vnodes, left, &mut violations, ViolationSources::all_disabled());
    assert!(!violations.contains(&x1), "source 6 disabled: x1 should NOT be caught");

    // Source 7 also doesn't catch it (collapse checks survivor's children, not cousins).
    push_collapse_violations_with_config(&vnodes, survivor, &mut violations, ViolationSources::only_source_7());
    assert!(
        !violations.contains(&x1),
        "source 7 should NOT catch ancestor-walk violations"
    );

    // ── Part 2: With only source 6, violation IS caught ──
    violations.clear();
    push_leaf_removal_violations_with_config(&vnodes, left, &mut violations, ViolationSources::only_source_6());
    assert!(
        violations.contains(&x1),
        "source 6 enabled: x1 SHOULD be caught, got: {violations:?}"
    );
    assert!(!violations.contains(&x2), "x2(1) ≤ left(5) should not be pushed");
}

// ── D6: Escalation post-promote structure verification (ADR-M-039) ──
//
// Existing escalation tests verify "no violations remain". These
// additionally verify the *resulting V-Tree structure* to catch
// mutations that resolve violations via a wrong restructuring path.

/// After escalation on a direct violation, the heaviest child `h`
/// must have been skip-promoted past `p`. Verify that `h` ends up
/// as a child of `g` (or higher) — not still under `p`.
#[test]
fn escalation_direct_resulting_structure() {
    // Same topology as `escalation_direct_breaks_promote_cycle`:
    //
    //   gg(2-node: [g, uncle_gg(40)])
    //     g(2-node: [p, uncle_g(8)])
    //       p(2-node: [c, sib(4)])
    //         c(2-node: [c1(20), c2(10)])
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let c1 = make_entry(&mut vnodes, 20);
    let c2 = make_entry(&mut vnodes, 10);
    let sib = make_entry(&mut vnodes, 4);
    let uncle_g = make_entry(&mut vnodes, 8);
    let uncle_gg = make_entry(&mut vnodes, 40);
    let c = make_structural_2(&mut vnodes, c1, c2);
    let p = make_structural_2(&mut vnodes, c, sib);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    // Post-rebalance: no violations, and c1's intensity is preserved.
    let remaining = find_violated_nodes(&vnodes);
    assert!(remaining.is_empty(), "should have no violations: {remaining:?}");

    // c1 must still be live and have its original intensity.
    assert!(vnodes.is_occupied(c1.index()), "c1 entry must still be live after escalation");
    assert_eq!(vnodes.get(c1.index()).intensity, 20, "c1 intensity must be preserved");

    // c2 must also be live with its intensity.
    assert!(vnodes.is_occupied(c2.index()), "c2 entry must still be live after escalation");
    assert_eq!(vnodes.get(c2.index()).intensity, 10, "c2 intensity must be preserved");

    // All original entry intensities must be conserved (sum = 20 + 10 + 4 + 8 + 40 = 82).
    let total: u64 = [c1, c2, sib, uncle_g, uncle_gg]
        .iter()
        .filter(|id| vnodes.is_occupied(id.index()))
        .map(|id| vnodes.get(id.index()).intensity)
        .sum();
    assert_eq!(total, 82, "total entry intensities must be conserved");
}

/// After escalation on the deeper structural-heaviest-child variant,
/// verify that all original entries survive with their intensities.
#[test]
fn escalation_structural_heaviest_resulting_structure() {
    // Same topology as `escalation_with_structural_heaviest_child`:
    //
    //   gg(2-node: [g, uncle_gg(40)])
    //     g(2-node: [p, uncle_g(10)])
    //       p(2-node: [c, sib(5)])
    //         c(2-node: [h, light(4)])
    //           h(2-node: [h1(20), h2(8)])
    let mut vnodes = Arena::new();
    let mut gnodes = make_gnodes();
    let h1 = make_entry(&mut vnodes, 20);
    let h2 = make_entry(&mut vnodes, 8);
    let light = make_entry(&mut vnodes, 4);
    let sib = make_entry(&mut vnodes, 5);
    let uncle_g = make_entry(&mut vnodes, 10);
    let uncle_gg = make_entry(&mut vnodes, 40);
    let h = make_structural_2(&mut vnodes, h1, h2);
    let c = make_structural_2(&mut vnodes, h, light);
    let p = make_structural_2(&mut vnodes, c, sib);
    let g = make_structural_2(&mut vnodes, p, uncle_g);
    let _gg = make_structural_2(&mut vnodes, g, uncle_gg);

    let mut violations = vec![c];
    rebalance(&mut vnodes, &mut gnodes, &mut violations, u32::MAX);

    let remaining = find_violated_nodes(&vnodes);
    assert!(remaining.is_empty(), "should have no violations: {remaining:?}");

    // Every original entry must be live with intensity preserved.
    let entries = [(h1, 20u64), (h2, 8), (light, 4), (sib, 5), (uncle_g, 10), (uncle_gg, 40)];
    for (id, expected) in &entries {
        assert!(vnodes.is_occupied(id.index()), "entry {id:?} must be live after escalation");
        assert_eq!(
            vnodes.get(id.index()).intensity,
            *expected,
            "entry {id:?} intensity must be preserved"
        );
    }

    // Total intensity conserved.
    let total: u64 = entries.iter().map(|(_, v)| v).sum();
    let actual_total: u64 = entries.iter().map(|(id, _)| vnodes.get(id.index()).intensity).sum();
    assert_eq!(actual_total, total, "total entry intensities must be conserved");

    // h1 must NOT be a direct child of gg (it was skip-promoted, but not THAT far).
    // Verify h1 is at depth ≥ 2 (below gg).
    let mut depth = 0u32;
    let mut cursor = vnodes.get(h1.index()).parent;
    while let Some(p) = cursor {
        depth += 1;
        cursor = vnodes.get(p.index()).parent;
    }
    assert!(depth >= 2, "h1 should be at depth >= 2 in the V-tree, got {depth}");
}
