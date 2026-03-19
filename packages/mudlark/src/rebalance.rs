// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! V-Tree rebalancing: violation detection, contraction, and promotion.
//!
//! Implements §IDEA M-11.  The max-uncle constraint (V-I3) may be
//! violated when an entry's intensity grows past its max uncle.
//! The rebalance loop finds and resolves all violations via
//! contraction (3-node → 2-node) and promotion (standard or skip).
//!
//! # Violation sources (§IDEA M-11.12)
//!
//! 1. The observed entry (intensity exceeded uncle).
//! 2. Structural ancestors (propagated sum exceeds uncle).
//! 3. Contraction side-effects (§IDEA M-11.11.1).
//! 4. Promotion side-effects (§IDEA M-11.11.1).
//! 5. Split preprocessing (§IDEA M-10.1).
//! 6. V-Tree leaf removal (§IDEA M-9.2): intensity decrease propagates to
//!    all ancestors via `propagate_v_sums_from`, weakening uncle
//!    coverage at every level. Ancestor walk required (§IDEA M-11.11.3).
//! 7. 2-node collapse: promoted node's children (§IDEA M-11.11.3).
//! 8. 3→2 transition: remaining siblings' children (§IDEA M-11.11.3).
//! 9. 2-node collapse: cousins' children (§IDEA M-11.11.3).
//! 10. g-contraction + promotion: intermediate-level grandchildren (§IDEA M-11.12).
//!
//! # Testing Violation Sources
//!
//! Each violation source has a dedicated inline function. For testing,
//! use [`ViolationSources`] to selectively enable/disable sources and
//! prove each source catches violations that others miss.
//!
//! # Tracing
//!
//! Operations emit structured [`tracing`] spans and events:
//!
//! | Level   | What                                                  |
//! |---------|-------------------------------------------------------|
//! | `DEBUG` | Tree operations (contract, promote, resolve)          |
//! | `TRACE` | Guard skips, violation-queue bookkeeping              |
//! | `WARN`  | Unexpected tree states (defensive checks)             |
//! | `ERROR` | Infinite-loop safety net (should never fire)          |
//!
//! See ADR-M-003 for the eager violation tracking strategy.

use std::fmt;
use std::sync::atomic::{AtomicU32, Ordering};

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::{GNodeId, VNodeId};
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::{DEPTH_STALE, PackedChildren, VKind, VNode};
use crate::vtree::{
    invalidate_depth_subtree, propagate_evictable_flags, recompute_structural_intensity, replace_child_in_parent,
    update_parent_cached_intensity, v_depth,
};

// ── Violation Source Configuration ──────────────────────────────────
//
// Each violation source can be enabled/disabled for testing purposes.
// This allows unit tests to prove that each source is necessary by
// showing violations are not caught with the source disabled.

/// Configuration for which violation sources are enabled.
///
/// Used primarily for testing to prove each source catches violations
/// that other sources miss. In production, all sources are enabled.
#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct ViolationSources {
    /// Source 3: Contraction side-effects at grandchildren (§IDEA M-11.11.1).
    pub source_3_contraction_grandchildren: bool,
    /// Source 4: Promotion side-effects at children (§IDEA M-11.11.1).
    pub source_4_promotion_children: bool,
    /// Source 6: Leaf-removal ancestor walk (§IDEA M-11.11.3).
    pub source_6_leaf_removal_ancestors: bool,
    /// Source 7: Collapse — promoted node's children (§IDEA M-11.11.3).
    pub source_7_collapse_children: bool,
    /// Source 8: 3→2 transition — remaining siblings' children (§IDEA M-11.11.3).
    pub source_8_three_to_two_siblings: bool,
    /// Source 9: Collapse — cousins' children (§IDEA M-11.11.3).
    pub source_9_collapse_cousins: bool,
    /// Source 10: g-contraction + promotion — merged node's grandchildren (§IDEA M-11.12).
    pub source_10_g_contraction_promotion: bool,
}

impl Default for ViolationSources {
    /// All sources enabled by default (production behavior).
    fn default() -> Self {
        Self::all_enabled()
    }
}

impl ViolationSources {
    /// All violation sources enabled (production default).
    #[inline]
    #[must_use]
    pub const fn all_enabled() -> Self {
        Self {
            source_3_contraction_grandchildren: true,
            source_4_promotion_children: true,
            source_6_leaf_removal_ancestors: true,
            source_7_collapse_children: true,
            source_8_three_to_two_siblings: true,
            source_9_collapse_cousins: true,
            source_10_g_contraction_promotion: true,
        }
    }
}

// ── Tracing display helpers ─────────────────────────────────────────
//
// Zero-allocation `Display` wrappers for `%field` in tracing macros.
// They borrow the arena and format on demand — no heap allocation
// unless a subscriber actually captures the formatted output.

/// Compact node label: `v5(S2,42)` or `v3(E,18)` or `v9(DEAD)`.
pub struct Nd<'a, V: Accumulator>(pub &'a Arena<VNode<V>>, pub VNodeId);

impl<V: Accumulator> fmt::Display for Nd<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let idx = self.1.index();
        if !self.0.is_occupied(idx) {
            return write!(f, "v{idx}(DEAD)");
        }
        let n = self.0.get(idx);
        match &n.kind {
            VKind::Entry { .. } => write!(f, "v{idx}(E,{:?})", n.intensity),
            VKind::Structural { children, .. } => {
                write!(f, "v{idx}(S{},{:?})", children.len(), n.intensity)
            }
        }
    }
}

/// Bracketed children list: `[v1(18), v2(15)]` or `∅` for entries.
struct Ch<'a, V: Accumulator>(&'a Arena<VNode<V>>, VNodeId);

impl<V: Accumulator> fmt::Display for Ch<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match &self.0.get(self.1.index()).kind {
            VKind::Entry { .. } => f.write_str("∅"),
            VKind::Structural { children, .. } => {
                f.write_str("[")?;
                for i in 0..children.len() {
                    if i > 0 {
                        f.write_str(", ")?;
                    }
                    let (id, int) = children.get(i);
                    write!(f, "v{}({:?})", id.index(), int)?;
                }
                f.write_str("]")
            }
        }
    }
}

/// Full violation context: ancestry chain with uncle info.
///
/// Example output:
/// `v3(E,18) ← v5(S2,21) [v3(18), v4(3)] ← v7(S2,42) [v5(21), v6(21)]  uncle_max=21`
pub struct Ctx<'a, V: Accumulator>(pub &'a Arena<VNode<V>>, pub VNodeId);

impl<V: Accumulator> fmt::Display for Ctx<'_, V> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (vnodes, c) = (self.0, self.1);
        write!(f, "{}", Nd(vnodes, c))?;
        let Some(p) = vnodes.get(c.index()).parent else {
            return f.write_str(" (root)");
        };
        write!(f, " ← {} {}", Nd(vnodes, p), Ch(vnodes, p))?;
        if let Some(g) = vnodes.get(p.index()).parent {
            write!(f, " ← {} {}", Nd(vnodes, g), Ch(vnodes, g))?;
        }
        if let Some(u) = max_uncle_intensity(vnodes, c) {
            write!(f, "  uncle_max={u:?}")?;
        }
        Ok(())
    }
}

// ── Violation detection (§IDEA M-11.2) ──────────────────────────────

/// Returns the maximum uncle intensity for node `c`, or `None` if `c`
/// has no grandparent (depth < 2 → no uncle relationship).
///
/// An uncle of `c` is a sibling of `c`'s parent `p` under
/// grandparent `g`.
#[must_use]
pub fn max_uncle_intensity<V: Accumulator>(vnodes: &Arena<VNode<V>>, c: VNodeId) -> Option<V> {
    let parent = vnodes.get(c.index()).parent?;
    let grandparent = vnodes.get(parent.index()).parent?;

    let g = vnodes.get(grandparent.index());
    if let VKind::Structural { children, .. } = &g.kind {
        let mut max_int = None;
        for i in 0..children.len() {
            let (id, intensity) = children.get(i);
            if id != parent {
                max_int = Some(max_int.map_or(intensity, |cur| if intensity > cur { intensity } else { cur }));
            }
        }
        max_int
    } else {
        None
    }
}

/// Whether node `c` violates the max-uncle constraint (V-I3).
///
/// A node is violated when `c.int > max { u.int : u ∈ uncles(c) }`.
/// Returns false if `c` has no grandparent (no uncle relationship).
#[must_use]
pub fn is_violated<V: Accumulator>(vnodes: &Arena<VNode<V>>, c: VNodeId) -> bool {
    let c_int = vnodes.get(c.index()).intensity;
    max_uncle_intensity(vnodes, c).is_some_and(|max_uncle| c_int > max_uncle)
}

// ── Contraction (§IDEA M-11.3) ──────────────────────────────────────

/// Contract a 3-node parent `p` into a 2-node by isolating the
/// heaviest child and merging the other two under a new structural
/// node.
///
/// Returns the ID of the newly created merged structural node.
///
/// See §IDEA M-11.3.
///
/// # Panics
///
/// Panics if `p` is not a structural 3-node.
pub fn contract<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, p: VNodeId) -> VNodeId {
    let _span = tracing::debug_span!(
        "contract",
        p = %Nd(vnodes, p),
        children = %Ch(vnodes, p),
    )
    .entered();

    // Read the 3-node's children.
    let (heaviest_idx, children_data) = {
        let node = vnodes.get(p.index());
        let children = match &node.kind {
            VKind::Structural { children, .. } => children,
            VKind::Entry { .. } => panic!("contract: p must be structural"),
        };
        assert!(children.len() == 3, "contract: p must be a 3-node");
        let h = children.heaviest_child_index();
        let data: [(VNodeId, V); 3] = [children.get(0), children.get(1), children.get(2)];
        (h, data)
    };

    // Identify isolate vs merge pair.
    let isolate = children_data[heaviest_idx];
    let mut merge = Vec::with_capacity(2);
    for (i, &child) in children_data.iter().enumerate() {
        if i != heaviest_idx {
            merge.push(child);
        }
    }
    let (a_id, a_int) = merge[0];
    let (b_id, b_int) = merge[1];

    // Compute has_evictable for the merged node.
    let a_terminal = node_has_evictable(vnodes, a_id);
    let b_terminal = node_has_evictable(vnodes, b_id);

    // Compute merged node depth (same as p's children, i.e. p_depth + 1)
    let p_depth = vnodes.get(p.index()).cached_depth.load(Ordering::Relaxed);
    let m_depth = if p_depth == DEPTH_STALE { DEPTH_STALE } else { p_depth + 1 };

    // Create the merged structural node.
    let merged = VNode {
        intensity: V::add(a_int, b_int),
        parent: Some(p),
        cached_depth: AtomicU32::new(m_depth),
        kind: VKind::Structural {
            children: PackedChildren::new_2((a_id, a_int), (b_id, b_int)),
            has_evictable: a_terminal || b_terminal,
        },
    };
    let m_id = VNodeId::from_index(vnodes.alloc(merged));

    // Re-parent a and b.
    vnodes.get_mut(a_id.index()).parent = Some(m_id);
    vnodes.get_mut(b_id.index()).parent = Some(m_id);

    // Invalidate depth cache for a and b subtrees (they moved down one level)
    invalidate_depth_subtree(vnodes, a_id);
    invalidate_depth_subtree(vnodes, b_id);

    // Rewrite p as a 2-node: [isolate, merged].
    let merged_int = V::add(a_int, b_int);
    let iso_terminal = node_has_evictable(vnodes, isolate.0);
    let m_terminal = a_terminal || b_terminal;

    let p_node = vnodes.get_mut(p.index());
    if let VKind::Structural { children, has_evictable } = &mut p_node.kind {
        *children = PackedChildren::new_2(isolate, (m_id, merged_int));
        *has_evictable = iso_terminal || m_terminal;
    }

    // Propagate terminal flags upward from p.
    propagate_evictable_flags(vnodes, p);

    tracing::debug!(
        merged = %Nd(vnodes, m_id),
        result = %Ch(vnodes, p),
        "complete",
    );
    m_id
}

// ── Standard Promote (§IDEA M-11.4) ─────────────────────────────────

/// Standard promotion: explode a 2-child structural node `c` into
/// its parent `p`, turning `p` into a 3-node.
///
/// **Precondition:** `c` is structural with 2 children. `p` is a
/// 2-node. `c` is a child of `p`.
///
/// See §IDEA M-11.4.
///
/// # Panics
///
/// Panics if `c` is not a structural 2-node, or if `c` has no parent.
pub fn standard_promote<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, c: VNodeId) {
    let p = vnodes.get(c.index()).parent.expect("standard_promote: c must have a parent");
    let _span = tracing::debug_span!(
        "standard_promote",
        c = %Nd(vnodes, c),
        children = %Ch(vnodes, c),
    )
    .entered();

    // Read c's two children.
    let (c1_id, c1_int, c2_id, c2_int) = {
        let node = vnodes.get(c.index());
        match &node.kind {
            VKind::Structural { children, .. } => {
                assert!(children.len() == 2, "standard_promote: c must be a 2-node");
                let (id1, int1) = children.get(0);
                let (id2, int2) = children.get(1);
                (id1, int1, id2, int2)
            }
            VKind::Entry { .. } => panic!("standard_promote: c must be structural"),
        }
    };

    // Find the sibling of c in p.
    let sibling_id = {
        let p_node = vnodes.get(p.index());
        match &p_node.kind {
            VKind::Structural { children, .. } => {
                let mut sib = None;
                for i in 0..children.len() {
                    let (id, _) = children.get(i);
                    if id != c {
                        sib = Some(children.get(i));
                        break;
                    }
                }
                sib.expect("standard_promote: sibling not found")
            }
            VKind::Entry { .. } => panic!("standard_promote: parent must be structural"),
        }
    };

    // Rewrite p as a 3-node: [c1, c2, sibling].
    let sib_terminal = node_has_evictable(vnodes, sibling_id.0);
    let c1_terminal = node_has_evictable(vnodes, c1_id);
    let c2_terminal = node_has_evictable(vnodes, c2_id);

    let p_node = vnodes.get_mut(p.index());
    if let VKind::Structural { children, has_evictable } = &mut p_node.kind {
        *children = PackedChildren::new_3((c1_id, c1_int), (c2_id, c2_int), sibling_id);
        *has_evictable = c1_terminal || c2_terminal || sib_terminal;
    }

    // Re-parent c1, c2 to p.
    vnodes.get_mut(c1_id.index()).parent = Some(p);
    vnodes.get_mut(c2_id.index()).parent = Some(p);

    // Invalidate depth cache for c1 and c2 subtrees (they moved up one level)
    invalidate_depth_subtree(vnodes, c1_id);
    invalidate_depth_subtree(vnodes, c2_id);

    // Destroy c.
    vnodes.dealloc(c.index());

    // Propagate terminal flags upward from p.
    propagate_evictable_flags(vnodes, p);

    tracing::debug!(result = %Ch(vnodes, p), "c destroyed, p is 3-node");
}

// ── Skip Promote (§IDEA M-11.5) ─────────────────────────────────────

/// Skip promotion: elevate node `c` by destroying its parent `p`
/// and installing `c` and its sibling as direct children of
/// grandparent `g`. `g` becomes a 3-node.
///
/// **Precondition:** `c` is an entry or a 3-child structural.
/// `p` is a 2-node. `g` is a 2-node.
///
/// See §IDEA M-11.5.
///
/// # Panics
///
/// Panics if `c` has no parent, or if `p` has no parent (grandparent).
pub fn skip_promote<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, c: VNodeId) -> Option<VNodeId> {
    let p = vnodes.get(c.index()).parent.expect("skip_promote: c must have a parent");
    let g = vnodes.get(p.index()).parent.expect("skip_promote: p must have a grandparent");
    let _span = tracing::debug_span!(
        "skip_promote",
        c = %Nd(vnodes, c),
        p = p.index(),
        g = g.index(),
    )
    .entered();

    // Find the sibling of c under p.
    let (s_id, s_int) = {
        let p_node = vnodes.get(p.index());
        match &p_node.kind {
            VKind::Structural { children, .. } => {
                let mut sib = None;
                for i in 0..children.len() {
                    let (id, int) = children.get(i);
                    if id != c {
                        sib = Some((id, int));
                        break;
                    }
                }
                sib.expect("skip_promote: sibling not found")
            }
            VKind::Entry { .. } => panic!("skip_promote: parent must be structural"),
        }
    };

    let c_int = vnodes.get(c.index()).intensity;

    // Replace p with c and s in g's children (g becomes 3-node).
    // First find the uncle (sibling of p in g).
    let (u_id, u_int) = {
        let g_node = vnodes.get(g.index());
        match &g_node.kind {
            VKind::Structural { children, .. } => {
                let mut uncle = None;
                for i in 0..children.len() {
                    let (id, int) = children.get(i);
                    if id != p {
                        uncle = Some((id, int));
                        break;
                    }
                }
                uncle.expect("skip_promote: uncle not found")
            }
            VKind::Entry { .. } => panic!("skip_promote: grandparent must be structural"),
        }
    };

    // Compute terminal flags.
    let c_terminal = node_has_evictable(vnodes, c);
    let s_terminal = node_has_evictable(vnodes, s_id);
    let u_terminal = node_has_evictable(vnodes, u_id);

    // Rewrite g as a 3-node: [c, s, u].
    let g_node = vnodes.get_mut(g.index());
    if let VKind::Structural { children, has_evictable } = &mut g_node.kind {
        *children = PackedChildren::new_3((c, c_int), (s_id, s_int), (u_id, u_int));
        *has_evictable = c_terminal || s_terminal || u_terminal;
    }

    // Re-parent c and s to g.
    vnodes.get_mut(c.index()).parent = Some(g);
    vnodes.get_mut(s_id.index()).parent = Some(g);

    // Invalidate depth cache for c and s subtrees (they moved up one level)
    invalidate_depth_subtree(vnodes, c);
    invalidate_depth_subtree(vnodes, s_id);

    // Destroy p.
    vnodes.dealloc(p.index());

    // Propagate terminal flags upward from g.
    propagate_evictable_flags(vnodes, g);

    tracing::debug!(result = %Ch(vnodes, g), "p destroyed, g is 3-node");

    // p can't be root (it has a parent g).  No root change.
    None
}

// ── Sibling helper ──────────────────────────────────────────────────

/// Return the sibling of `child` under 2-node `parent`, plus its
/// cached intensity.
///
/// # Panics
///
/// Panics if `parent` is not structural, or if `child` is not found.
fn sibling_of<V: Accumulator>(vnodes: &Arena<VNode<V>>, parent: VNodeId, child: VNodeId) -> (VNodeId, V) {
    let p_node = vnodes.get(parent.index());
    match &p_node.kind {
        VKind::Structural { children, .. } => {
            for i in 0..children.len() {
                let (id, int) = children.get(i);
                if id != child {
                    return (id, int);
                }
            }
            panic!("sibling_of: child not found in parent");
        }
        VKind::Entry { .. } => panic!("sibling_of: parent must be structural"),
    }
}

// ── Legacy Promote (§IDEA M-11.6) ──────────────────────────────────

/// Legacy promotion: elevate entry `c` to grandparent level and
/// bequeath `c`'s vacated seat to a new zero-intensity entry
/// for the missing G-child.
///
/// **Precondition:** `c` is a V-entry backing a semi-internal G-node.
/// `p` is a 2-node. `g` is a 2-node.
///
/// Returns the newly created `GNodeId` so the caller can handle
/// `node_count` and plateau maintenance.
///
/// See §IDEA M-11.6.
///
/// # Panics
///
/// Panics if `c` is not an entry, if its G-node is not semi-internal,
/// or if `c` has no parent or grandparent.
pub fn legacy_promote<C: Coordinate, V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    c: VNodeId,
) -> GNodeId {
    let p = vnodes.get(c.index()).parent.expect("legacy_promote: c must have a parent");
    let g = vnodes
        .get(p.index())
        .parent
        .expect("legacy_promote: p must have a grandparent");

    let gnode_id = match &vnodes.get(c.index()).kind {
        VKind::Entry { gnode, .. } => *gnode,
        VKind::Structural { .. } => panic!("legacy_promote: c must be an entry"),
    };

    debug_assert!(
        gnodes.get(gnode_id.index()).is_semi_internal(),
        "legacy_promote: backing G-node must be semi-internal"
    );

    let _span = tracing::debug_span!(
        "legacy_promote",
        c = %Nd(vnodes, c),
        p = p.index(),
        g = g.index(),
        gnode = gnode_id.index(),
    )
    .entered();

    // ── G-Tree: create the missing child ────────────────────────
    let gn = gnodes.get(gnode_id.index());
    let (new_lo, new_hi) = gn
        .uncovered_range()
        .expect("legacy_promote: semi-internal must have uncovered range");
    let new_child = GNode {
        lo: new_lo,
        hi: new_hi,
        sum: V::zero(),
        own: V::zero(),
        left: None,
        right: None,
        parent: Some(gnode_id),
        entry: None,
    };
    let new_child_id = GNodeId::from_index(gnodes.alloc(new_child));

    // Link into parent's empty slot.
    {
        let gn = gnodes.get_mut(gnode_id.index());
        if gn.left.is_none() {
            gn.left = Some(new_child_id);
        } else {
            debug_assert!(gn.right.is_none(), "legacy_promote: expected empty right slot");
            gn.right = Some(new_child_id);
        }
    }

    // ── V-Tree: create entry for new child ──────────────────────
    // ne takes c's place at the same depth
    let c_depth = vnodes.get(c.index()).cached_depth.load(Ordering::Relaxed);
    let ne = VNode {
        intensity: V::zero(),
        parent: Some(p),
        cached_depth: AtomicU32::new(c_depth), // Same depth as c was
        kind: VKind::Entry {
            gnode: new_child_id,
            is_exposed: true,   // new terminal: on the contour
            is_evictable: true, // new terminal: no dependents
        },
    };
    let ne_id = VNodeId::from_index(vnodes.alloc(ne));
    gnodes.get_mut(new_child_id.index()).entry = Some(ne_id);

    // ── V-Tree: replace c with ne in p's children ───────────────
    let c_int = vnodes.get(c.index()).intensity;
    replace_child_in_parent(vnodes, p, c, ne_id, V::zero());

    // ── V-Tree: lift c to g (g becomes 3-node) ─────────────────
    let (u_id, u_int) = sibling_of(vnodes, g, p);

    // c is about to become frozen (internal, 2 G-children) → not evictable.
    let c_evictable = false;
    let p_evictable = node_has_evictable(vnodes, p);
    let u_evictable = node_has_evictable(vnodes, u_id);

    let p_int = vnodes.get(p.index()).intensity;
    let g_node = vnodes.get_mut(g.index());
    if let VKind::Structural { children, has_evictable } = &mut g_node.kind {
        *children = PackedChildren::new_3((c, c_int), (p, p_int), (u_id, u_int));
        *has_evictable = c_evictable || p_evictable || u_evictable;
    }

    vnodes.get_mut(c.index()).parent = Some(g);
    // c moved up one level: update its depth
    let new_c_depth = if c_depth == DEPTH_STALE { DEPTH_STALE } else { c_depth - 1 };
    vnodes.get(c.index()).cached_depth.store(new_c_depth, Ordering::Relaxed);

    // ── Freeze c: gnode now has 2 G-children ────────────────────
    if let VKind::Entry {
        is_exposed,
        is_evictable,
        ..
    } = &mut vnodes.get_mut(c.index()).kind
    {
        *is_exposed = false; // internal: fully shielded
        *is_evictable = false; // has dependents
    }

    // ── Recompute p's intensity ─────────────────────────────────
    recompute_structural_intensity(vnodes, p);
    let new_p_int = vnodes.get(p.index()).intensity;
    update_parent_cached_intensity(vnodes, p, new_p_int);

    // ── Propagate evictable flags ───────────────────────────────
    propagate_evictable_flags(vnodes, p);
    propagate_evictable_flags(vnodes, g);

    tracing::debug!(
        new_gnode = new_child_id.index(),
        new_ventry = ne_id.index(),
        "legacy_promote complete: c lifted to g, new child created",
    );

    new_child_id
}

// ── Violation queue helpers (§IDEA M-11.11) ────────────────────────

/// Push grandchildren of `node` that are newly violated.
///
/// After restructuring at `node`, its grandchildren may have weakened
/// uncle shields.  See §IDEA M-11.11.1.
pub fn push_side_effect_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, node: VNodeId, violations: &mut Vec<VNodeId>) {
    push_side_effect_violations_with_config(vnodes, node, violations, ViolationSources::all_enabled());
}

/// Push grandchildren violations with configurable source (source 3).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_side_effect_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    node: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_3_contraction_grandchildren {
        return;
    }
    push_grandchild_violations(vnodes, node, violations);
}

/// Push source-10 violations: grandchildren of the merged node after
/// g-contraction + promotion (§IDEA M-11.12).
///
/// Same grandchild scan as source 3, but gated by `source_10` for
/// independent test coverage.  In production, both sources are enabled.
pub fn push_source_10_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, node: VNodeId, violations: &mut Vec<VNodeId>) {
    push_source_10_violations_with_config(vnodes, node, violations, ViolationSources::all_enabled());
}

/// Configurable variant of [`push_source_10_violations`].
#[inline]
pub fn push_source_10_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    node: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_10_g_contraction_promotion {
        return;
    }
    push_grandchild_violations(vnodes, node, violations);
}

/// Shared grandchild-scan logic used by sources 3 and 10.
fn push_grandchild_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, node: VNodeId, violations: &mut Vec<VNodeId>) {
    let child_ids: Vec<VNodeId> = match &vnodes.get(node.index()).kind {
        VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).collect(),
        VKind::Entry { .. } => return,
    };
    for child_id in child_ids {
        if let VKind::Structural { children, .. } = &vnodes.get(child_id.index()).kind {
            for i in 0..children.len() {
                let (gc_id, _) = children.get(i);
                if is_violated(vnodes, gc_id) {
                    tracing::trace!(gc = %Nd(vnodes, gc_id), at = %Nd(vnodes, node), "side-effect violation");
                    violations.push(gc_id);
                }
            }
        }
    }
}

/// Push direct children of `node` that are violated.
///
/// After promotion or contraction, children move to a new uncle
/// context.  The spec's `find_deepest_violated_node()` (§IDEA M-11.8) would
/// discover these; our queue-based approach pushes them explicitly.
pub fn push_promoted_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, node: VNodeId, violations: &mut Vec<VNodeId>) {
    push_promoted_violations_with_config(vnodes, node, violations, ViolationSources::all_enabled());
}

/// Push promoted violations with configurable source (source 4).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_promoted_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    node: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_4_promotion_children {
        return;
    }
    let child_ids: Vec<VNodeId> = match &vnodes.get(node.index()).kind {
        VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).collect(),
        VKind::Entry { .. } => return,
    };
    for child_id in child_ids {
        if is_violated(vnodes, child_id) {
            tracing::trace!(child = %Nd(vnodes, child_id), at = %Nd(vnodes, node), "promoted violation");
            violations.push(child_id);
        }
    }
}

/// Push direct children of `node` that are violated, excluding `skip`.
///
/// Used after Phase 1 contraction: children may be violated at the
/// grandparent level, but we must NOT re-enqueue the node being
/// resolved (Phase 2 handles it).
pub fn push_contraction_child_violations<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    node: VNodeId,
    skip: VNodeId,
    violations: &mut Vec<VNodeId>,
) {
    let child_ids: Vec<VNodeId> = match &vnodes.get(node.index()).kind {
        VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).collect(),
        VKind::Entry { .. } => return,
    };
    for child_id in child_ids {
        if child_id != skip && is_violated(vnodes, child_id) {
            tracing::trace!(
                child = %Nd(vnodes, child_id),
                skip = skip.index(),
                "contraction-child violation",
            );
            violations.push(child_id);
        }
    }
}

/// Walk ancestors from `start` to the root, pushing siblings' children
/// that are newly violated after a leaf removal.
///
/// `vtree_remove_leaf` calls `propagate_v_sums_from`, which decreases
/// the intensity of every structural ancestor from the change point to
/// the root.  At each level, the decreased ancestor may now be a weaker
/// uncle to its siblings' children.  See §IDEA M-11.11.3.
///
/// `start` should be the structural change point: the parent in the
/// 3→2 case, or the grandparent in the 2-node collapse case.
///
/// See §IDEA M-11.11.3.
pub fn push_leaf_removal_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, start: VNodeId, violations: &mut Vec<VNodeId>) {
    push_leaf_removal_violations_with_config(vnodes, start, violations, ViolationSources::all_enabled());
}

/// Push leaf-removal ancestor walk violations with configurable source (source 6).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_leaf_removal_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    start: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_6_leaf_removal_ancestors {
        return;
    }
    let mut ancestor = start;
    while let Some(parent) = vnodes.get(ancestor.index()).parent {
        // Iterate siblings of `ancestor` under `parent`.
        let sibling_ids: Vec<VNodeId> = match &vnodes.get(parent.index()).kind {
            VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).filter(|&id| id != ancestor).collect(),
            VKind::Entry { .. } => break,
        };

        // Check each sibling's children for violations.
        for sib_id in sibling_ids {
            if let VKind::Structural { children, .. } = &vnodes.get(sib_id.index()).kind {
                for i in 0..children.len() {
                    let (child_id, _) = children.get(i);
                    if is_violated(vnodes, child_id) {
                        tracing::trace!(
                            child = %Nd(vnodes, child_id),
                            weakened_uncle = %Nd(vnodes, ancestor),
                            "leaf-removal violation",
                        );
                        violations.push(child_id);
                    }
                }
            }
        }

        ancestor = parent;
    }
}

/// Check the surviving sibling's children after a 2-node collapse.
///
/// When a 2-node parent collapses, the surviving sibling `sole` is
/// re-parented to the grandparent `g`. The surviving sibling's children
/// now have entirely different uncles (siblings of `sole` under `g`)
/// and may become violated.
///
/// This is violation source 7 (§IDEA M-11.11.3). The ancestor walk in
/// `push_leaf_removal_violations` doesn't catch this because it walks
/// *from* the grandparent upward, checking siblings' children — but
/// `sole` is now a child of the grandparent, not a sibling.
///
/// See §IDEA M-11.11.3.
pub fn push_collapse_violations<V: Accumulator>(vnodes: &Arena<VNode<V>>, sole: VNodeId, violations: &mut Vec<VNodeId>) {
    push_collapse_violations_with_config(vnodes, sole, violations, ViolationSources::all_enabled());
}

/// Push collapse violations with configurable source (source 7).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_collapse_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    sole: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_7_collapse_children {
        return;
    }
    tracing::debug!(sole = sole.index(), "push_collapse_violations: checking node");
    push_children_violations(vnodes, sole, "collapse (source 7)", violations);
}

/// Check the remaining siblings' children after a 3→2 transition.
///
/// When an entry is removed from a 3-node parent, the remaining siblings'
/// children lose that entry as a potential uncle. Their `max_uncle` may
/// decrease, causing violations.
///
/// This is violation source 8 (§IDEA M-11.11.3). The ancestor walk in
/// `push_leaf_removal_violations` walks UPWARD from P, checking siblings
/// of P — not children of P's remaining children.
///
/// `p` should be the parent node that transitioned from 3→2.
/// `removed` is the ID of the removed child (to skip it).
///
/// See §IDEA M-11.11.3.
pub fn push_remaining_sibling_violations<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    p: VNodeId,
    removed: VNodeId,
    violations: &mut Vec<VNodeId>,
) {
    push_remaining_sibling_violations_with_config(vnodes, p, removed, violations, ViolationSources::all_enabled());
}

/// Push 3→2 transition violations with configurable source (source 8).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_remaining_sibling_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    p: VNodeId,
    removed: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_8_three_to_two_siblings {
        return;
    }
    // Get the remaining children of p (excluding removed).
    let remaining: Vec<VNodeId> = match &vnodes.get(p.index()).kind {
        VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).filter(|&id| id != removed).collect(),
        VKind::Entry { .. } => return,
    };

    // Check each remaining sibling's children for violations.
    for sibling in remaining {
        push_children_violations(vnodes, sibling, "3→2 transition (source 8)", violations);
    }
}

/// Check the COUSINS of the promoted node after a 2-node collapse.
///
/// When `v_parent` collapses into `sole`, the OTHER children of grandparent
/// (the "cousins" of `v_parent`) have their children potentially violated
/// because those children's uncle changed from `v_parent` to `sole`.
///
/// Example:
/// ```text
///              grandparent
///              /         \
///          v_parent     cousin_sibling
///           /              /     \
///        sole          child1   child2
/// ```
/// After `v_parent` collapses → sole promoted to grandparent:
/// ```text
///              grandparent
///              /         \
///           sole      cousin_sibling
///                        /     \
///                    child1   child2  ← may be violated!
/// ```
///
/// child1 and child2's uncle changed from `v_parent` (high intensity) to
/// sole (possibly lower intensity). If child1.intensity > sole.intensity,
/// child1 becomes violated.
///
/// This is violation source 9 (§IDEA M-11.11.3).
///
/// * `v_parent` - the node that collapsed (no longer in tree structure)
/// * `sole` - the promoted child
/// * `grandparent` - the grandparent of the evicted entry
pub fn push_cousin_violations<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    sole: VNodeId,
    grandparent: VNodeId,
    violations: &mut Vec<VNodeId>,
) {
    push_cousin_violations_with_config(vnodes, sole, grandparent, violations, ViolationSources::all_enabled());
}

/// Push cousin violations with configurable source (source 9).
///
/// Used for testing to prove this source catches violations others miss.
#[inline]
pub fn push_cousin_violations_with_config<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    sole: VNodeId,
    grandparent: VNodeId,
    violations: &mut Vec<VNodeId>,
    config: ViolationSources,
) {
    if !config.source_9_collapse_cousins {
        return;
    }
    // Get the OTHER children of grandparent (excluding sole, which was just promoted)
    let cousins: Vec<VNodeId> = match &vnodes.get(grandparent.index()).kind {
        VKind::Structural { children, .. } => children.iter().map(|(id, _)| id).filter(|&id| id != sole).collect(),
        VKind::Entry { .. } => return,
    };

    tracing::debug!(
        sole = sole.index(),
        grandparent = grandparent.index(),
        cousins = ?cousins.iter().map(|c| c.index()).collect::<Vec<_>>(),
        "push_cousin_violations: checking cousins' children (source 9)",
    );

    // Check each cousin's children for violations.
    for cousin in cousins {
        push_children_violations(vnodes, cousin, "collapse cousins (source 9)", violations);
    }
}

/// Check a node's direct children for violations.
///
/// Helper function for source 7 (collapse), source 8 (3→2 transition),
/// and source 9 (collapse cousins).
fn push_children_violations<V: Accumulator>(
    vnodes: &Arena<VNode<V>>,
    node: VNodeId,
    source: &str,
    violations: &mut Vec<VNodeId>,
) {
    // Only structural nodes have children to check.
    let children: Vec<VNodeId> = match &vnodes.get(node.index()).kind {
        VKind::Structural { children, .. } => (0..children.len()).map(|i| children.get(i).0).collect(),
        VKind::Entry { .. } => {
            tracing::debug!(
                node = node.index(),
                source,
                "push_children_violations: node is entry, no children",
            );
            return;
        }
    };

    tracing::debug!(
        node = node.index(),
        children = ?children.iter().map(|c| c.index()).collect::<Vec<_>>(),
        source,
        "push_children_violations: checking children",
    );

    for child_id in children {
        let violated = is_violated(vnodes, child_id);
        tracing::debug!(
            child = child_id.index(),
            violated,
            source,
            "push_children_violations: child check",
        );
        if violated {
            tracing::trace!(
                child = %Nd(vnodes, child_id),
                parent = %Nd(vnodes, node),
                source,
                "sibling-removal violation",
            );
            violations.push(child_id);
        }
    }
}

// ── Internal helpers ─────────────────────────────────────────────────

/// Whether a V-node carries the evictable flag (entry or structural).
fn node_has_evictable<V: Accumulator>(vnodes: &Arena<VNode<V>>, id: VNodeId) -> bool {
    match &vnodes.get(id.index()).kind {
        VKind::Entry { is_evictable, .. } => *is_evictable,
        VKind::Structural { has_evictable, .. } => *has_evictable,
    }
}

/// Get the child count of a structural node.  Returns 0 for entries.
fn structural_child_count<V: Accumulator>(vnodes: &Arena<VNode<V>>, id: VNodeId) -> usize {
    match &vnodes.get(id.index()).kind {
        VKind::Structural { children, .. } => children.len(),
        VKind::Entry { .. } => 0,
    }
}

/// Whether any direct child of `node` is violated (V-I3).
///
/// Used by escalation: if a grandchild of `p` through the heaviest
/// child `h` is violated, resolving it would contract `p` (undoing
/// the promote) and create a merged node whose standard-promote
/// recreates the 3-node — an infinite cycle.
fn any_child_violated<V: Accumulator>(vnodes: &Arena<VNode<V>>, node: VNodeId) -> bool {
    match &vnodes.get(node.index()).kind {
        VKind::Structural { children, .. } => {
            for i in 0..children.len() {
                let (child_id, _) = children.get(i);
                if is_violated(vnodes, child_id) {
                    return true;
                }
            }
            false
        }
        VKind::Entry { .. } => false,
    }
}

// ── Escalation (§IDEA M-11.10) ──────────────────────────────────────

/// After standard-promote makes `p` a 3-node, check whether the
/// heaviest child (or its descendants) would cycle back.
///
/// **Escalation triggers:**
/// - **Direct:** `h` itself is violated at the `g` level.
/// - **Indirect:** a child of `h` is violated at the `p` level.
///   Resolving it would contract `p`, producing a merged node whose
///   standard-promote recreates the 3-node — an infinite cycle.
///
/// **Action:** contract `p` (isolating `h`), optionally contract `g`,
/// then skip-promote `h` up to `g`.
fn escalate_after_promote<V: Accumulator>(vnodes: &mut Arena<VNode<V>>, p: VNodeId, violations: &mut Vec<VNodeId>) {
    let heaviest = match &vnodes.get(p.index()).kind {
        VKind::Structural { children, .. } if children.len() == 3 => children.get(children.heaviest_child_index()).0,
        _ => return,
    };

    let h_direct = is_violated(vnodes, heaviest);
    let h_indirect = !h_direct && any_child_violated(vnodes, heaviest);
    if !h_direct && !h_indirect {
        return;
    }

    let Some(g) = vnodes.get(p.index()).parent else {
        return;
    };
    let _span = tracing::debug_span!(
        "escalate",
        h = %Nd(vnodes, heaviest),
        reason = if h_direct { "direct" } else { "indirect" },
    )
    .entered();

    // Contract p: isolate h, merge the other two.
    let merged = contract(vnodes, p);
    push_side_effect_violations(vnodes, p, violations);
    push_side_effect_violations(vnodes, merged, violations);
    push_contraction_child_violations(vnodes, p, heaviest, violations);

    // Check if contraction resolved the issue.
    let needs_skip = if h_direct {
        is_violated(vnodes, heaviest)
    } else {
        is_violated(vnodes, merged)
    };
    if !needs_skip {
        tracing::debug!("resolved by contraction");
        return;
    }

    // Ensure g is a 2-node before skip-promoting.
    let g_merged = if structural_child_count(vnodes, g) == 3 {
        let g_merged = contract(vnodes, g);
        push_side_effect_violations(vnodes, g, violations);
        push_side_effect_violations(vnodes, g_merged, violations);
        push_promoted_violations(vnodes, g, violations);
        let resolved = !is_violated(vnodes, heaviest) && (h_direct || !is_violated(vnodes, merged));
        if resolved {
            tracing::debug!("resolved by g-contraction");
            return;
        }
        Some(g_merged)
    } else {
        None
    };

    // Skip-promote h past p up to g.
    if let Some(g_id) = vnodes.get(p.index()).parent {
        skip_promote(vnodes, heaviest);
        push_side_effect_violations(vnodes, g_id, violations);
        push_promoted_violations(vnodes, g_id, violations);

        // Source 10 (§IDEA M-11.12): post-promotion push at merged node.
        if let Some(gm) = g_merged {
            push_source_10_violations(vnodes, gm, violations);
        }
    }
}

// ── Resolve (§IDEA M-11.9) ─────────────────────────────────────────

/// Resolve a single violation at node `c`.
///
/// Implements §IDEA M-11.9:
/// 1. If parent is 3-node → contract.  Re-check; return if resolved.
/// 2. If `c` is structural 2-node → standard promote (+ escalation).
/// 3. Else → ensure grandparent is 2-node, then skip promote.
///
/// Side-effect violations are pushed onto `violations`.
///
/// Returns `Some(GNodeId)` if a legacy promotion created a new G-node,
/// `None` otherwise.
///
/// `depth_evict` is the current live $D_{\text{evict}}$ gate.  When a
/// semi-internal entry would qualify for legacy promotion but
/// `v_depth(c) > depth_evict`, the dispatcher falls through to skip
/// promote instead — the heir would land past the eviction threshold
/// and be immediately eligible for eviction (§IDEA M-11.7, §IDEA M-11.9).
pub fn resolve<C: Coordinate, V: Accumulator>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    c: VNodeId,
    violations: &mut Vec<VNodeId>,
    depth_evict: u32,
) -> Option<GNodeId> {
    let _span = tracing::debug_span!("resolve", node = c.index()).entered();
    tracing::debug!(ctx = %Ctx(vnodes, c), "begin");

    let Some(p) = vnodes.get(c.index()).parent else {
        tracing::trace!("no parent — nothing to resolve");
        return None;
    };

    // ── Phase 1: ensure p is a 2-node ───────────────────────────
    if structural_child_count(vnodes, p) == 3 {
        tracing::debug!(p = %Nd(vnodes, p), "phase 1: contracting 3-node parent");
        let merged = contract(vnodes, p);
        push_side_effect_violations(vnodes, p, violations);
        push_side_effect_violations(vnodes, merged, violations);
        push_contraction_child_violations(vnodes, p, c, violations);
        if !is_violated(vnodes, c) {
            tracing::debug!("phase 1: resolved by contraction");
            return None;
        }
    }

    let is_c_structural_2 = matches!(
        &vnodes.get(c.index()).kind,
        VKind::Structural { children, .. } if children.len() == 2
    );

    let mut result = None;

    // ── Phase 2: promote c ──────────────────────────────────────
    if is_c_structural_2 {
        tracing::debug!("phase 2: standard promote");
        standard_promote(vnodes, c);
        push_side_effect_violations(vnodes, p, violations);
        push_promoted_violations(vnodes, p, violations);
        escalate_after_promote(vnodes, p, violations);
    } else {
        tracing::debug!("phase 2: skip promote path");
        let Some(g) = vnodes.get(p.index()).parent else {
            tracing::trace!("no grandparent — cannot skip-promote");
            return None;
        };

        // Ensure g is a 2-node.
        let g_merged = if structural_child_count(vnodes, g) == 3 {
            let merged = contract(vnodes, g);
            push_side_effect_violations(vnodes, g, violations);
            push_side_effect_violations(vnodes, merged, violations);
            push_promoted_violations(vnodes, g, violations);
            if !is_violated(vnodes, c) {
                tracing::debug!("phase 2: resolved by g-contraction");
                return None;
            }
            Some(merged)
        } else {
            None
        };

        // Re-read grandparent after possible contraction.
        if let Some(g_id) = vnodes.get(p.index()).parent {
            // §IDEA M-11.7: Check for semi-internal dispatch + depth gate.
            let is_semi = matches!(
                &vnodes.get(c.index()).kind,
                VKind::Entry { gnode, .. }
                    if gnodes.get(gnode.index()).is_semi_internal()
            );

            if is_semi && v_depth(vnodes, c) <= depth_evict {
                tracing::debug!("phase 2: legacy promote (semi-internal entry)");
                let new_g = legacy_promote(vnodes, gnodes, c);
                result = Some(new_g);
                // Legacy promote replaces c (high intensity) with ne
                // (zero intensity) inside p. This collapses the uncle
                // shield for s's children: they used to have uncle = c,
                // now uncle = ne = 0. Scan p's grandchildren to catch
                // these newly created violations that
                // push_side_effect_violations(g) would miss (they're
                // great-grandchildren of g, one level too deep).
                push_side_effect_violations(vnodes, p, violations);
            } else {
                skip_promote(vnodes, c);
            }
            push_side_effect_violations(vnodes, g_id, violations);
            push_promoted_violations(vnodes, g_id, violations);

            // Source 10 (§IDEA M-11.12): When g-contraction created an
            // intermediate node (merged_g), promotion disperses the
            // aggregate at that level.  Grandchildren of merged_g
            // (depth 3 below g) face a weakened uncle context that
            // push_side_effect_violations(g) cannot reach.
            if let Some(merged) = g_merged {
                push_source_10_violations(vnodes, merged, violations);
            }
        } else {
            tracing::warn!(
                node = %Ctx(vnodes, c),
                "skip-promote path: no grandparent after g-contraction — resolve incomplete",
            );
        }
    }

    // Post-resolve check: if c is still alive and violated, warn.
    if vnodes.is_occupied(c.index()) && is_violated(vnodes, c) {
        tracing::warn!(
            node = %Ctx(vnodes, c),
            "resolve() returning with node STILL violated",
        );
    }

    result
}

// ── Rebalance loop (§IDEA M-11.8) ──────────────────────────────────

/// Drain the violations queue, resolving each violation.
///
/// Guards against stale/destroyed nodes and already-resolved
/// violations.  The queue is empty when this function returns.
///
/// See §IDEA M-11.8 and ADR-M-003.
///
/// # Panics
///
/// Panics if the loop exceeds a safety-net iteration limit
/// proportional to the arena size (`20 × node_count`, floor
/// 10 000), indicating a bug in the resolution logic rather
/// than legitimate work.  Large trees with many cascading
/// violations (e.g. after decay or batch eviction) can
/// legitimately exceed a small fixed constant.
/// `depth_evict` is the current live $D_{\text{evict}}$, forwarded to
/// [`resolve`] for the legacy-promotion depth gate (§IDEA M-11.9).
pub fn rebalance<C: Coordinate, V: Accumulator + Inspectable>(
    vnodes: &mut Arena<VNode<V>>,
    gnodes: &mut Arena<GNode<C, V>>,
    violations: &mut Vec<VNodeId>,
    depth_evict: u32,
) -> Vec<GNodeId> {
    let mut new_gnodes = Vec::new();
    // Safety-net ceiling — proportional to tree size so that large
    // but valid workloads are never rejected.  Each resolve() pushes
    // O(1) side-effect entries, so total iterations are bounded by a
    // constant factor of the arena population.  The 10 000 floor
    // keeps small trees protected.
    let max_iterations: u32 = vnodes.count().saturating_mul(20).max(10_000);
    let mut iterations: u32 = 0;
    let mut resolved: u32 = 0;

    let _span = tracing::debug_span!("rebalance", queue = violations.len()).entered();

    // Pre-rebalance audit: detect violations not in the queue.
    if tracing::enabled!(tracing::Level::DEBUG) {
        crate::diagnostic::audit_violations(vnodes, violations, "PRE-REBALANCE");
    }

    while let Some(c) = violations.pop() {
        iterations += 1;
        if iterations > max_iterations {
            tracing::error!(
                iterations,
                max_iterations,
                resolved,
                queue = violations.len(),
                current = c.index(),
                "rebalance safety-net exceeded — dumping queue tail",
            );
            let tail = violations.len().saturating_sub(20);
            for (i, v) in violations[tail..].iter().enumerate() {
                if vnodes.is_occupied(v.index()) {
                    tracing::error!(idx = tail + i, entry = %Ctx(vnodes, *v));
                } else {
                    tracing::error!(idx = tail + i, node = v.index(), "DEAD");
                }
            }
            // In debug builds: crash immediately (it's a bug).
            // In release builds: break out and log — avoids taking
            // down a production process for what may be an edge-case
            // in side-effect accounting rather than a true loop.
            #[cfg(debug_assertions)]
            panic!(
                "rebalance: exceeded {max_iterations} iterations \
                 (queue={}, node=v{}, resolved={resolved})",
                violations.len(),
                c.index(),
            );
            #[cfg(not(debug_assertions))]
            {
                tracing::error!(
                    "breaking out of rebalance loop — \
                     possible bug in violation resolution",
                );
                break;
            }
        }

        // Guard: node may have been destroyed by a prior resolution.
        if !vnodes.is_occupied(c.index()) {
            tracing::trace!(node = c.index(), "skip destroyed");
            continue;
        }
        // Guard: violation may have been resolved as a side effect.
        if !is_violated(vnodes, c) {
            tracing::trace!(node = c.index(), "skip already resolved");
            continue;
        }

        resolved += 1;
        tracing::debug!(
            iter = iterations,
            resolved,
            queue = violations.len(),
            node = %Ctx(vnodes, c),
            "resolving violation",
        );
        if let Some(gid) = resolve(vnodes, gnodes, c, violations, depth_evict) {
            new_gnodes.push(gid);
        }

        // Post-resolve diagnostic: log if the node is still violated.
        if vnodes.is_occupied(c.index()) && is_violated(vnodes, c) {
            tracing::warn!(
                iter = iterations,
                node = %Ctx(vnodes, c),
                "node STILL violated after resolve",
            );
        }

        // Post-resolve full scan: detect violations created by resolve
        // that weren't enqueued.
        if tracing::enabled!(tracing::Level::DEBUG) {
            crate::diagnostic::audit_violations(vnodes, violations, "POST-RESOLVE");
        }
    }

    tracing::debug!(iterations, resolved, "rebalance complete");

    // Full-tree audit: verify the queue-based loop left no violations
    // behind.  O(n) scan — guarded by tracing level.
    // The O(n) scan always runs in debug builds (ensures the assert
    // fires during `cargo test`) and optionally in release when a
    // tracing subscriber is active at DEBUG.
    if cfg!(debug_assertions) || tracing::enabled!(tracing::Level::DEBUG) {
        let remaining = crate::diagnostic::audit_violations(vnodes, violations, "RESIDUAL");
        assert!(
            remaining.is_empty(),
            "rebalance finished with residual violations: {remaining:?}"
        );
    }

    new_gnodes
}

// ── Full-tree violation audit (§IDEA M-11.8) ───────────────────────

/// Full-tree scan for violated nodes, returned deepest-first.
///
/// This is the spec's §IDEA M-11.8 audit function — `O(n)` per call.
/// Used by the `debug_assert!` at the end of [`rebalance`] to confirm
/// the queue-based loop left no violations behind.
#[must_use]
pub fn find_violated_nodes<V: Accumulator>(vnodes: &Arena<VNode<V>>) -> Vec<VNodeId> {
    let mut violated: Vec<(VNodeId, u32)> = Vec::new();
    for (idx, _) in vnodes.iter_occupied() {
        let id = VNodeId::from_index(idx);
        if is_violated(vnodes, id) {
            violated.push((id, v_depth(vnodes, id)));
        }
    }
    // Deepest first (highest depth = processed first).
    violated.sort_by_key(|b| std::cmp::Reverse(b.1));
    violated.into_iter().map(|(id, _)| id).collect()
}

// ── Unit tests ──────────────────────────────────────────────────────

#[cfg(test)]
mod tests {}
