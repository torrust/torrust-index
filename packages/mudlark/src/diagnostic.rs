// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Consolidated diagnostic and audit functions (ADR-M-028).
//!
//! Gathers duplicated audit logic from `evict.rs`, `split.rs`, and
//! `rebalance.rs` into a single module.  All output uses structured
//! `tracing` events — callers gate invocation with
//! `tracing::enabled!()` to avoid cost when no subscriber is active.
//!
//! Nothing calls these functions yet — they are wired up in
//! subsequent ADR-M-028 steps.  The `dead_code` allow is removed
//! once all call sites are migrated.

#![allow(dead_code)]

use std::fmt;

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::{GSlotPointer, VSlotPointer};
use crate::rebalance::{self, Ctx};
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::{VKind, VNode};
#[cfg(feature = "dynamic-contour-tracking")]
use crate::{gnode::GState, graph::GvGraph};

// ── Violation audit ─────────────────────────────────────────────────

/// Full-tree audit for violated V-nodes not present in the work queue.
///
/// Performs an `O(n)` scan over all live V-nodes, identifies those
/// that violate V-I3 (max-uncle constraint), and returns any that
/// are **not** already queued in `violations`.
///
/// Each missed node is logged at `ERROR` with full context
/// (node, parent, grandparent, uncle intensities).
///
/// Callers should guard invocation with
/// `tracing::enabled!(tracing::Level::DEBUG)`.
pub fn audit_violations<V: Accumulator + Inspectable>(
    vnodes: &Arena<VNode<V>>,
    violations: &[VSlotPointer],
    checkpoint: &str,
) -> Vec<VSlotPointer> {
    let all_violated = rebalance::find_violated_nodes(vnodes);
    let queued: std::collections::HashSet<usize> = violations.iter().map(|v| v.index()).collect();
    let mut missed = Vec::new();
    for &v in &all_violated {
        if !queued.contains(&v.index()) {
            tracing::error!(
                checkpoint,
                node = %Ctx(vnodes, v),
                "violation NOT in queue",
            );
            missed.push(v);
        }
    }
    missed
}

// ── Plateau consistency audit ───────────────────────────────────────

/// Context for the optional P-I4 spot-check in [`audit_plateau_consistency`].
///
/// When provided, the audit performs an additional check that a
/// semi-internal parent's surviving child belongs to a different
/// plateau than the parent itself.
#[cfg(feature = "dynamic-contour-tracking")]
pub struct PlateauAuditContext {
    pub parent_id: GSlotPointer,
    pub parent_state: GState,
}

/// Consolidated plateau consistency check.
///
/// Verifies:
/// 1. Forward/back map consistency — every basis element's back-pointer
///    agrees with the forward map's key.
/// 2. Plateau count — `plateaus.len() == plateau_basis.plateau_count()`.
/// 3. (Optional) P-I4 spot-check — if `context` is `Some`, and the
///    parent became `SemiInternal`, verifies that the surviving child
///    belongs to a different plateau than the parent.
///
/// Emits `tracing::error!` on any inconsistency.  The function always
/// runs if called — callers gate with `tracing::enabled!()`.
#[cfg(feature = "dynamic-contour-tracking")]
pub fn audit_plateau_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    checkpoint: &str,
    context: Option<&PlateauAuditContext>,
) {
    // 1. Forward/back consistency for all plateaus.
    for &key in graph.plateaus.keys() {
        for &r in graph.plateau_basis.basis_elements(&key) {
            let back = graph.plateau_basis.plateau_key(r);
            if back != Some(key) {
                tracing::error!(checkpoint, ?key, gnode = r.index(), ?back, "basis back-pointer inconsistency");
            }
        }
    }

    // 2. Plateau count matches basis plateau count.
    let map_len = graph.plateaus.len();
    let basis_len = graph.plateau_basis.plateau_count();
    if map_len != basis_len {
        tracing::error!(
            checkpoint,
            map_len,
            basis_len,
            "plateaus.len() != plateau_basis.plateau_count()",
        );
    }

    // 3. P-I4 spot-check (only when context is provided).
    if let Some(ctx) = context
        && ctx.parent_state == GState::SemiInternal
    {
        let g = graph.gnodes.get(ctx.parent_id.index());
        let surviving = g.left.or(g.right);
        if let Some(surviving_id) = surviving {
            let parent_plateau = graph.plateau_basis.plateau_key(ctx.parent_id);
            if let Some(pk) = parent_plateau
                && let Some(ck) = graph.plateau_basis.plateau_key(surviving_id)
                && pk == ck
            {
                tracing::debug!(
                    checkpoint,
                    ?pk,
                    ?ck,
                    parent = ctx.parent_id.index(),
                    surviving = surviving_id.index(),
                    "transient P-I4 overlap (will be repaired)",
                );
            }
        }
    }
}

// ── Missed violation diagnosis ──────────────────────────────────────

/// Context describing the eviction that preceded a missed violation.
pub struct EvictionContext {
    pub evicted_parent: Option<VSlotPointer>,
    pub evicted_parent_child_count: usize,
    pub collapse_sibling: Option<VSlotPointer>,
}

/// Detailed structural diagnosis of a missed violation.
///
/// Walks the V-tree around `violated` to determine what structural
/// change caused the violation and which source should have caught it.
/// All output is emitted as structured `tracing::error!` events.
///
/// Callers should guard invocation with
/// `tracing::enabled!(tracing::Level::ERROR)`.
#[allow(clippy::too_many_lines)]
pub fn diagnose_missed_violation<V: Accumulator + Inspectable>(
    vnodes: &Arena<VNode<V>>,
    violated: VSlotPointer,
    context: &EvictionContext,
) {
    let v = vnodes.get(violated.index());
    let v_intensity = v.intensity;

    // Get parent and grandparent.
    let Some(parent_id) = v.parent else {
        tracing::error!(
            node = violated.index(),
            "DIAGNOSIS: node has no parent (root?), should not be violated",
        );
        return;
    };

    let parent = vnodes.get(parent_id.index());
    let Some(grandparent_id) = parent.parent else {
        tracing::error!(
            node = violated.index(),
            parent = parent_id.index(),
            "DIAGNOSIS: parent has no grandparent (depth 1?), should not be violated",
        );
        return;
    };

    // Find uncles (siblings of parent under grandparent).
    let grandparent = vnodes.get(grandparent_id.index());
    let uncles: Vec<(VSlotPointer, V)> = match &grandparent.kind {
        VKind::Structural { children, .. } => children.iter().filter(|(id, _)| *id != parent_id).collect(),
        VKind::Entry { .. } => vec![],
    };

    let max_uncle_intensity = uncles.iter().map(|(_, int)| int.to_f64_approx()).fold(0.0_f64, f64::max);

    let uncle_desc: Vec<String> = uncles
        .iter()
        .map(|(id, int)| format!("v{}({})", id.index(), int.to_f64_approx()))
        .collect();

    tracing::error!(
        node = violated.index(),
        intensity = v_intensity.to_f64_approx(),
        parent = parent_id.index(),
        parent_intensity = parent.intensity.to_f64_approx(),
        grandparent = grandparent_id.index(),
        grandparent_intensity = grandparent.intensity.to_f64_approx(),
        ?uncle_desc,
        max_uncle = max_uncle_intensity,
        is_violation = v_intensity.to_f64_approx() > max_uncle_intensity,
        "MISSED VIOLATION DIAGNOSIS",
    );

    tracing::error!(
        evicted_parent = ?context.evicted_parent.map(VSlotPointer::index),
        child_count = context.evicted_parent_child_count,
        collapse_sibling = ?context.collapse_sibling.map(VSlotPointer::index),
        "eviction context (child_count: 2=collapse, 3=3→2)",
    );

    // Analyze why this was missed.
    if let Some(sole) = context.collapse_sibling {
        if is_ancestor(vnodes, sole, violated) {
            tracing::error!(
                node = violated.index(),
                collapse_sibling = sole.index(),
                "node IS a descendant of collapse_sibling → should have been caught by source 7",
            );
            // Check direct vs. indirect descendant.
            let sole_children: Vec<usize> = match &vnodes.get(sole.index()).kind {
                VKind::Structural { children, .. } => (0..children.len()).map(|i| children.get(i).0.index()).collect(),
                VKind::Entry { .. } => vec![],
            };
            if sole_children.contains(&violated.index()) {
                tracing::error!(
                    node = violated.index(),
                    "node IS a direct child of collapse_sibling — source 7 should catch it",
                );
            } else {
                tracing::error!(
                    node = violated.index(),
                    ?sole_children,
                    "node is NOT a direct child — source 7 only checks direct children. MISSING SOURCE.",
                );
            }
        } else {
            tracing::error!(
                node = violated.index(),
                collapse_sibling = sole.index(),
                "node is NOT a descendant of collapse_sibling",
            );
        }
    }

    if let Some(evicted_p) = context.evicted_parent
        && evicted_p == grandparent_id
    {
        tracing::error!(
            node = violated.index(),
            evicted_parent = evicted_p.index(),
            "node's grandparent = evicted_parent → parent is sibling of removed entry. Check source 8.",
        );
    }

    // V-tree ancestry walk.
    let _ancestry_span = tracing::error_span!("vtree_path_to_violated", node = violated.index()).entered();
    let mut current = violated;
    let mut depth = 0_usize;
    loop {
        let n = vnodes.get(current.index());
        let kind = match &n.kind {
            VKind::Entry { .. } => "E",
            VKind::Structural { children, .. } => match children.len() {
                2 => "S2",
                3 => "S3",
                _ => "S?",
            },
        };
        tracing::error!(
            depth,
            node = current.index(),
            kind,
            intensity = n.intensity.to_f64_approx(),
            "ancestry",
        );
        match n.parent {
            Some(p) => {
                current = p;
                depth += 1;
            }
            None => break,
        }
    }
}

/// Check if `ancestor` is an ancestor of `descendant` in the V-tree.
fn is_ancestor<V: Accumulator>(vnodes: &Arena<VNode<V>>, ancestor: VSlotPointer, mut descendant: VSlotPointer) -> bool {
    while let Some(p) = vnodes.get(descendant.index()).parent {
        if p == ancestor {
            return true;
        }
        descendant = p;
    }
    false
}

// ── Display helpers ─────────────────────────────────────────────────

/// G-node display: `G5(T,[0,128),sum=42)`.
///
/// Shows the arena index, state (T/S/I for Terminal/SemiInternal/Internal),
/// interval, and accumulated sum.
pub struct Gn<'a, C: Coordinate, V: Accumulator + Inspectable>(pub &'a Arena<GNode<C, V>>, pub GSlotPointer);

impl<C: Coordinate, V: Accumulator + Inspectable> fmt::Display for Gn<'_, C, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = self.1.index();
        if !self.0.is_occupied(idx) {
            return write!(f, "G{idx}(DEAD)");
        }
        let g = self.0.get(idx);
        let state = match g.state() {
            crate::gnode::GState::Terminal => "T",
            crate::gnode::GState::SemiInternal => "S",
            crate::gnode::GState::Internal => "I",
        };
        write!(
            f,
            "G{idx}({state},[{},{}),sum={})",
            g.lo.to_f64(),
            g.hi.to_f64(),
            g.sum.to_f64_approx()
        )
    }
}

/// Plateau display: `P([0,128),d=3,sum=42)`.
///
/// Shows the thatched range, depth, and total energy of the plateau.
#[cfg(feature = "dynamic-contour-tracking")]
pub struct Pl<'a, C: Coordinate, V: Accumulator + Inspectable, const N: u32>(pub &'a GvGraph<C, V, N>, pub GSlotPointer);

#[cfg(feature = "dynamic-contour-tracking")]
impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> fmt::Display for Pl<'_, C, V, N> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = self.1.index();
        let key = self.0.plateau_basis.plateau_key(self.1);
        match key {
            Some(k) => {
                if let Some(plateau) = self.0.plateaus.get(&k) {
                    write!(
                        f,
                        "P([{},{}),d={},sum={})",
                        plateau.start.to_f64(),
                        plateau.end.to_f64(),
                        plateau.depth,
                        plateau.sum.to_f64_approx()
                    )
                } else {
                    write!(f, "P(G{idx},key={k:?},NO_PLATEAU)")
                }
            }
            None => write!(f, "P(G{idx},not_basis)"),
        }
    }
}
