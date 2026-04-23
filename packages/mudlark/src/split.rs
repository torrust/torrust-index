// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Catalytic and bootstrap splits (§IDEA M-10).
//!
//! Splitting creates two G-children for a terminal G-node and
//! registers them as new V-entries. The parent's V-entry freezes
//! (children intercept all future observations). Catalytic splits
//! are violation-free (§IDEA M-10.4).

use std::sync::atomic::{AtomicU32, Ordering};

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::graph::GvGraph;
use crate::handle::{GSlotPointer, VSlotPointer};
use crate::rebalance::{Nd, contract, push_promoted_violations, push_side_effect_violations};
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};
use crate::vtree::{propagate_evictable_flags, v_depth};

/// Attempt to split a G-node, subject to guards and depth gates.
///
/// Implements §IDEA M-10.1. Checks:
/// 1. Terminal? (no G-children)
/// 2. Interval divisible? (midpoint > lo)
/// 3. Sum > θ?
/// 4. Entry exists?
/// 5. Bootstrap vs catalytic path
/// 6. Preprocessing: contract 3-node parent if needed
/// 7. Depth gate: `v_depth(entry) <= D_create`
///
/// # Panics
///
/// Panics if the entry's V-parent link is inconsistent (the parent
/// was confirmed to exist but is `None` after the bootstrap check).
pub fn attempt_split<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &mut GvGraph<C, V, N>,
    g_id: GSlotPointer,
) {
    let g = graph.gnodes.get(g_id.index());

    // Guard: must be terminal (no G-children).
    if g.left.is_some() || g.right.is_some() {
        return;
    }

    // Guard: interval must be divisible.
    let mid = C::midpoint(g.lo, g.hi);
    if mid.partial_cmp(&g.lo) != Some(std::cmp::Ordering::Greater) {
        return;
    }

    // Guard: sum must exceed split threshold θ.
    if g.sum.partial_cmp(&graph.config.split_threshold) != Some(std::cmp::Ordering::Greater) {
        return;
    }

    // Guard: entry must exist.
    let Some(entry_id) = g.entry else {
        return;
    };

    // Bootstrap case: entry is V-root with no parent.
    if graph.vnodes.get(entry_id.index()).parent.is_none() {
        bootstrap_split(graph, g_id);
        return;
    }

    // Preprocessing: if V-parent is 3-node, contract first.
    let p_id = graph.vnodes.get(entry_id.index()).parent.unwrap();
    if let VKind::Structural { children, .. } = &graph.vnodes.get(p_id.index()).kind
        && children.len() == 3
    {
        let _span = tracing::debug_span!(
            "split_preprocess",
            p = %Nd(&graph.vnodes, p_id),
        )
        .entered();
        let merged = contract(&mut graph.vnodes, p_id);
        push_side_effect_violations(&graph.vnodes, p_id, &mut graph.violations);
        push_side_effect_violations(&graph.vnodes, merged, &mut graph.violations);
        push_promoted_violations(&graph.vnodes, p_id, &mut graph.violations);
    }

    // Depth gate (checked against post-contraction position).
    // Re-read entry_id — the G-node's entry link is unchanged by contraction.
    let entry_id = graph.gnodes.get(g_id.index()).entry.unwrap();
    if v_depth(&graph.vnodes, entry_id) > graph.live_depth_create {
        return;
    }

    catalytic_split(graph, g_id);
}

/// Bootstrap split: first split when V-root is a lone entry.
///
/// Creates:
/// - 2 G-children `[lo, mid)` and `[mid, hi)`
/// - 2 V-entries (intensity 0, exposed)
/// - 1 child structural (2-node wrapping the entries)
/// - 1 root structural (2-node: old entry + child structural)
///
/// Flips old entry's `is_exposed` to false. Updates `v_root`.
///
/// See §IDEA M-10.3.
fn bootstrap_split<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &mut GvGraph<C, V, N>, g_id: GSlotPointer) {
    let (lo, hi, entry_id) = {
        let g = graph.gnodes.get(g_id.index());
        (g.lo, g.hi, g.entry.expect("bootstrap_split: g must have an entry"))
    };
    let mid = C::midpoint(lo, hi);
    let _span = tracing::debug_span!("bootstrap_split", g_id = g_id.index(), ?lo, ?hi,).entered();

    // G-Tree: create children.
    let left_id = alloc_g_child(&mut graph.gnodes, lo, mid, g_id);
    let right_id = alloc_g_child(&mut graph.gnodes, mid, hi, g_id);
    graph.gnodes.get_mut(g_id.index()).left = Some(left_id);
    graph.gnodes.get_mut(g_id.index()).right = Some(right_id);

    // V-Tree: create entries for children.
    let le_id = alloc_v_entry(&mut graph.vnodes, &mut graph.gnodes, left_id);
    let re_id = alloc_v_entry(&mut graph.vnodes, &mut graph.gnodes, right_id);

    // V-Tree: child structural (2-node wrapping le, re).
    let cs_id = alloc_v_structural_2(&mut graph.vnodes, le_id, re_id);

    // V-Tree: root structural (2-node: old entry + child structural).
    let entry_int = graph.vnodes.get(entry_id.index()).intensity;
    let root_structural = VNode {
        intensity: entry_int,
        parent: None,
        cached_depth: AtomicU32::new(0), // New V-root
        kind: VKind::Structural {
            children: PackedChildren::new_2((entry_id, entry_int), (cs_id, V::zero())),
            has_evictable: true,
        },
    };
    let root_s_id = VSlotPointer::from_index(graph.vnodes.alloc(root_structural).0);
    graph.vnodes.get_mut(entry_id.index()).parent = Some(root_s_id);
    graph.vnodes.get_mut(cs_id.index()).parent = Some(root_s_id);
    // Set depths for children: entry and cs at depth 1, le/re at depth 2
    graph.vnodes.get(entry_id.index()).cached_depth.store(1, Ordering::Relaxed);
    graph.vnodes.get(cs_id.index()).cached_depth.store(1, Ordering::Relaxed);
    graph.vnodes.get(le_id.index()).cached_depth.store(2, Ordering::Relaxed);
    graph.vnodes.get(re_id.index()).cached_depth.store(2, Ordering::Relaxed);

    // Flip old entry's flags: now internal (both children present).
    if let VKind::Entry {
        is_exposed,
        is_evictable,
        ..
    } = &mut graph.vnodes.get_mut(entry_id.index()).kind
    {
        *is_exposed = false;
        *is_evictable = false;
    }

    // Update graph state.
    graph.v_root = Some(root_s_id);
    graph.node_count += 2;
    // +2 new terminals, −1 parent no longer terminal = net +1.
    graph.terminal_count += 1;

    // Plateau maintenance (no-op without `plateau` feature).
    graph.plateau_after_bootstrap_split(g_id, left_id);

    #[cfg(feature = "dynamic-contour-tracking")]
    if tracing::enabled!(tracing::Level::DEBUG) {
        crate::diagnostic::audit_plateau_consistency(graph, "POST-BOOTSTRAP-SPLIT", None);
    }
}

/// Catalytic split: general case.
///
/// **Precondition:** `g` is terminal, `g.entry` has a 2-node V-parent,
/// `v_depth(entry) <= D_create`.
///
/// Creates:
/// - 2 G-children
/// - 2 V-entries + 1 structural wrapping them
/// - Adds structural as 3rd child of V-parent (2→3-node)
///
/// Flips entry's `is_exposed` to false. Propagates evictable flags.
///
/// See §IDEA M-10.2.
#[allow(clippy::too_many_lines)] // Plateau maintenance adds ~50 lines.
fn catalytic_split<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &mut GvGraph<C, V, N>, g_id: GSlotPointer) {
    let (lo, hi, entry_id) = {
        let g = graph.gnodes.get(g_id.index());
        (g.lo, g.hi, g.entry.expect("catalytic_split: g must have an entry"))
    };
    let mid = C::midpoint(lo, hi);
    let _span = tracing::debug_span!("catalytic_split", g_id = g_id.index(), ?lo, ?hi, ?mid,).entered();
    let p_id = graph
        .vnodes
        .get(entry_id.index())
        .parent
        .expect("catalytic_split: entry must have a parent");

    // G-Tree: create children.
    let left_id = alloc_g_child(&mut graph.gnodes, lo, mid, g_id);
    let right_id = alloc_g_child(&mut graph.gnodes, mid, hi, g_id);
    graph.gnodes.get_mut(g_id.index()).left = Some(left_id);
    graph.gnodes.get_mut(g_id.index()).right = Some(right_id);

    // V-Tree: create entries for children.
    let le_id = alloc_v_entry(&mut graph.vnodes, &mut graph.gnodes, left_id);
    let re_id = alloc_v_entry(&mut graph.vnodes, &mut graph.gnodes, right_id);

    // Get parent's depth to compute s_depth
    let p_depth = graph.vnodes.get(p_id.index()).cached_depth.load(Ordering::Relaxed);
    let s_depth = if p_depth == DEPTH_STALE { DEPTH_STALE } else { p_depth + 1 };
    let child_depth = if s_depth == DEPTH_STALE { DEPTH_STALE } else { s_depth + 1 };

    // V-Tree: structural wrapping the entries.
    let s = VNode {
        intensity: V::zero(),
        parent: Some(p_id),
        cached_depth: AtomicU32::new(s_depth),
        kind: VKind::Structural {
            children: PackedChildren::new_2((le_id, V::zero()), (re_id, V::zero())),
            has_evictable: true,
        },
    };
    let s_id = VSlotPointer::from_index(graph.vnodes.alloc(s).0);
    graph.vnodes.get_mut(le_id.index()).parent = Some(s_id);
    graph.vnodes.get_mut(re_id.index()).parent = Some(s_id);
    // Set child depths
    graph
        .vnodes
        .get(le_id.index())
        .cached_depth
        .store(child_depth, Ordering::Relaxed);
    graph
        .vnodes
        .get(re_id.index())
        .cached_depth
        .store(child_depth, Ordering::Relaxed);

    // Add s as third child of p (2-node → 3-node).
    let p = graph.vnodes.get_mut(p_id.index());
    if let VKind::Structural { children, .. } = &mut p.kind {
        children.add_child(s_id, V::zero());
    }

    // Flip entry's flags: now internal (both children present).
    if let VKind::Entry {
        is_exposed,
        is_evictable,
        ..
    } = &mut graph.vnodes.get_mut(entry_id.index()).kind
    {
        *is_exposed = false;
        *is_evictable = false;
    }

    // Propagate evictable flags upward from p.
    propagate_evictable_flags(&mut graph.vnodes, p_id);

    // Update node count.
    graph.node_count += 2;
    // +2 new terminals, −1 parent no longer terminal = net +1.
    graph.terminal_count += 1;

    // Plateau maintenance (no-op without `plateau` feature).
    graph.plateau_after_catalytic_split(g_id, left_id);

    #[cfg(feature = "dynamic-contour-tracking")]
    if tracing::enabled!(tracing::Level::DEBUG) {
        crate::diagnostic::audit_plateau_consistency(graph, "POST-CATALYTIC-SPLIT", None);
    }
}

// ── Allocation helpers ──────────────────────────────────────────────

/// Allocate a terminal G-child covering `[lo, hi)` under `parent`.
fn alloc_g_child<C: Coordinate, V: Accumulator>(
    gnodes: &mut Arena<GNode<C, V>>,
    lo: C,
    hi: C,
    parent: GSlotPointer,
) -> GSlotPointer {
    let g = GNode {
        lo,
        hi,
        sum: V::zero(),
        own: V::zero(),
        left: None,
        right: None,
        parent: Some(parent),
        entry: None,
    };
    GSlotPointer::from_index(gnodes.alloc(g).0)
}

/// Allocate a zero-intensity V-entry for `gnode` and link the G-node to it.
fn alloc_v_entry<C: Coordinate, V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    gnode: GSlotPointer,
) -> VSlotPointer {
    let e = VNode {
        intensity: V::zero(),
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE), // Caller sets depth
        kind: VKind::Entry {
            gnode,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let e_id = VSlotPointer::from_index(vnodes.alloc(e).0);
    gnodes.get_mut(gnode.index()).entry = Some(e_id);
    e_id
}

/// Allocate a structural 2-node wrapping two V-children and parent them.
fn alloc_v_structural_2<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, a: VSlotPointer, b: VSlotPointer) -> VSlotPointer {
    let a_int = vnodes.get(a.index()).intensity;
    let b_int = vnodes.get(b.index()).intensity;
    let s = VNode {
        intensity: V::add(a_int, b_int),
        parent: None,
        cached_depth: AtomicU32::new(DEPTH_STALE), // Caller sets depth
        kind: VKind::Structural {
            children: PackedChildren::new_2((a, a_int), (b, b_int)),
            has_evictable: true, // new entries are always exposed (terminal → evictable)
        },
    };
    let s_id = VSlotPointer::from_index(vnodes.alloc(s).0);
    vnodes.get_mut(a.index()).parent = Some(s_id);
    vnodes.get_mut(b.index()).parent = Some(s_id);
    s_id
}

#[cfg(test)]
mod tests {}
