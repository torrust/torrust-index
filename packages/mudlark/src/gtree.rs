// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! G-Tree operations: spatial routing and sum propagation.
//!
//! Implements §IDEA M-5.2 (routing) and §IDEA M-8 Step 4 (sum propagation).
//! See ADR-M-012 for the decision to use recomputation (not delta
//! propagation) on the `observe()` hot path.

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::handle::GNodeId;
use crate::traits::{Accumulator, Coordinate};

/// Route to the terminal or semi-internal G-node that should receive
/// an observation at coordinate `x`.
///
/// Iterative descent through the G-Tree. Handles all three node states
/// uniformly:
/// - Terminal (0 children): returns immediately.
/// - Semi-internal (1 child): routes to the child or falls through.
/// - Internal (2 children): always routes to a child.
///
/// See §IDEA M-5.2 `route_to_receiver`.
#[must_use]
pub fn route_to_receiver<C: Coordinate, V: Accumulator>(gnodes: &Arena<GNode<C, V>>, root: GNodeId, x: C) -> GNodeId {
    let mut current = root;
    loop {
        let g = gnodes.get(current.index());
        let mid = C::midpoint(g.lo, g.hi);
        if x < mid {
            if let Some(left) = g.left {
                current = left;
            } else {
                return current;
            }
        } else if let Some(right) = g.right {
            current = right;
        } else {
            return current;
        }
    }
}

/// Compute the G-Tree depth of a dyadic interval `[lo, hi)`.
///
/// Depth = `n − log₂(width)`. For integer coordinates this uses
/// `trailing_zeros` on the integer width; for float coordinates it
/// uses `f64::log2`.
///
/// This is the free-function form (no `GvGraph` needed). The
/// [`GvGraph::gnode_depth`] method wraps this with the const-generic
/// `N` for convenience.
///
/// # Panics
///
/// Panics if `width` is zero (degenerate interval).
#[must_use]
#[inline]
pub fn gnode_depth_from_interval<C: Coordinate>(lo: C, hi: C, n: u32) -> u32 {
    let width_f64 = C::width(lo, hi).to_f64();
    debug_assert!(width_f64 > 0.0, "gnode_depth_from_interval: zero-width interval");
    // log2 of a power-of-two width. For integers this is exact;
    // for floats f64::log2 returns an exact integer for 2^k inputs.
    //
    // Cast via i32 (not u32) so that sub-unit float widths
    // (log2 < 0) produce depths > n instead of saturating to n.
    #[allow(clippy::cast_possible_truncation)]
    let log2_width = width_f64.log2() as i32;
    #[allow(clippy::cast_possible_wrap, clippy::cast_sign_loss)]
    let depth = (n as i32 - log2_width) as u32;
    depth
}

/// Recompute `sum = own + left.sum + right.sum` from `start` upward
/// to the G-Tree root, directly reasserting G-I1 at each level.
///
/// Used by `observe()` on the hot path instead of
/// `propagate_g_sums` (moved to tests), avoiding the need for subtraction on
/// `Accumulator`. See ADR-M-012 for the full analysis.
///
/// Cost: `O(depth_geo)`, with one extra sibling read per level vs
/// delta propagation.
pub fn recompute_g_sums<C: Coordinate, V: Accumulator>(gnodes: &mut Arena<GNode<C, V>>, start: GNodeId) {
    let mut current = Some(start);
    while let Some(id) = current {
        let (left_sum, right_sum) = {
            let g = gnodes.get(id.index());
            let l = g.left.map_or_else(V::zero, |l| gnodes.get(l.index()).sum);
            let r = g.right.map_or_else(V::zero, |r| gnodes.get(r.index()).sum);
            (l, r)
        };
        let g = gnodes.get_mut(id.index());
        g.sum = V::add(g.own, V::add(left_sum, right_sum));
        current = g.parent;
    }
}

/// Recompute `g.sum` for every G-node in a subtree, bottom-up.
///
/// `preorder` must list nodes in pre-order (root first). The function
/// iterates in reverse (post-order: leaves first) so each node's
/// children are already correct when it is visited.
///
/// Cost: $O(|\text{preorder}|)$.
///
/// Used by `decay()` after scaling `g.own` values.
pub fn recompute_g_sums_subtree<C: Coordinate, V: Accumulator>(gnodes: &mut Arena<GNode<C, V>>, preorder: &[GNodeId]) {
    for &gid in preorder.iter().rev() {
        let (left_sum, right_sum) = {
            let g = gnodes.get(gid.index());
            let l = g.left.map_or_else(V::zero, |l| gnodes.get(l.index()).sum);
            let r = g.right.map_or_else(V::zero, |r| gnodes.get(r.index()).sum);
            (l, r)
        };
        let g = gnodes.get_mut(gid.index());
        g.sum = V::add(g.own, V::add(left_sum, right_sum));
    }
}

#[cfg(test)]
mod tests {}
