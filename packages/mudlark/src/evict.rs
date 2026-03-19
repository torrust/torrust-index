// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Eviction: tip removal and candidate scanning (Phase 3).
//!
//! Implements §IDEA M-12 (contraction). A terminal G-node is
//! evicted by absorbing its energy into the parent, removing its
//! V-entry, and deallocating the arena slot.
//!
//! See ADR-M-013 (eligibility), ADR-M-014 (absorption), ADR-M-015 (scan).

#[cfg(feature = "dynamic-contour-tracking")]
use crate::gnode::GState;
use crate::graph::GvGraph;
use crate::handle::VNodeId;
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::VKind;
use crate::{rebalance, vtree};

/// Evict a single terminal G-node via its V-entry (ADR-M-014).
///
/// Performs:
/// 1. Absorption: `parent.own += child.sum`.
/// 2. Unlink child from parent (`left = None` or `right = None`).
/// 3. Update parent's V-entry intensity; propagate V-sums.
/// 4. Ancestor-walk violation check (ADR-M-003 push pattern).
/// 5. Update `is_exposed` → propagate `has_evictable`.
/// 6. Remove V-entry via `vtree_remove_leaf()`.
/// 7. Deallocate G-node; decrement `node_count`.
///
/// # Panics
///
/// - If the V-entry is not backed by a terminal G-node.
/// - If the G-node has no parent (G-root eviction is forbidden).
#[allow(clippy::too_many_lines)] // Plateau maintenance adds ~40 lines.
pub fn evict_tip<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &mut GvGraph<C, V, N>, v_id: VNodeId) {
    let span = tracing::debug_span!(
        "evict_tip",
        v_id = v_id.index(),
        gnode = tracing::field::Empty,
        parent = tracing::field::Empty,
    )
    .entered();

    // Read the backing G-node from the V-entry.
    let gnode_id = match &graph.vnodes.get(v_id.index()).kind {
        VKind::Entry { gnode, is_evictable, .. } => {
            debug_assert!(*is_evictable, "evict_tip: V-entry {} is not evictable", v_id.index());
            *gnode
        }
        VKind::Structural { .. } => panic!("evict_tip: V-node {} is structural, not an entry", v_id.index()),
    };
    span.record("gnode", gnode_id.index());

    // Safety: G-root must never be evicted.
    assert_ne!(gnode_id, graph.g_root, "evict_tip: cannot evict the G-root");

    let parent_id = graph
        .gnodes
        .get(gnode_id.index())
        .parent
        .expect("evict_tip: terminal G-node must have a parent");
    span.record("parent", parent_id.index());

    // ── 1. Absorption: parent.own += child.sum (ADR-M-014) ────────
    let child_sum = graph.gnodes.get(gnode_id.index()).sum;
    let parent_sum_before = graph.gnodes.get(parent_id.index()).sum;

    graph.gnodes.get_mut(parent_id.index()).own = V::add(graph.gnodes.get(parent_id.index()).own, child_sum);

    // ── 2. Unlink child from parent ─────────────────────────────
    {
        let p = graph.gnodes.get_mut(parent_id.index());
        if p.left == Some(gnode_id) {
            p.left = None;
        } else if p.right == Some(gnode_id) {
            p.right = None;
        } else {
            panic!(
                "evict_tip: G-node {} is not a child of parent {}",
                gnode_id.index(),
                parent_id.index()
            );
        }
    }

    // Verify G-sum invariant: parent.sum should be unchanged (ADR-M-014 proof).
    // Recompute to assert: sum = own_new + remaining_child.sum
    {
        let p = graph.gnodes.get(parent_id.index());
        let left_sum = p.left.map_or_else(V::zero, |l| graph.gnodes.get(l.index()).sum);
        let right_sum = p.right.map_or_else(V::zero, |r| graph.gnodes.get(r.index()).sum);
        let recomputed = V::add(p.own, V::add(left_sum, right_sum));
        debug_assert!(
            (recomputed.to_f64_approx() - parent_sum_before.to_f64_approx()).abs() < 1e-9,
            "evict_tip: G-sum invariant violation after absorption: \
             recomputed={}, expected={}",
            recomputed.to_f64_approx(),
            parent_sum_before.to_f64_approx()
        );
        // Write it back to ensure exact bit-level correctness.
        graph.gnodes.get_mut(parent_id.index()).sum = recomputed;
    }

    // ── 3. Update parent's V-entry intensity ────────────────────
    let p_entry_id = graph
        .gnodes
        .get(parent_id.index())
        .entry
        .expect("evict_tip: parent must have V-entry (has dependents)");
    {
        let p_own = graph.gnodes.get(parent_id.index()).own;
        graph.vnodes.get_mut(p_entry_id.index()).intensity = p_own;
        vtree::update_parent_cached_intensity(&mut graph.vnodes, p_entry_id, p_own);
        vtree::propagate_v_sums(&mut graph.vnodes, p_entry_id);

        // ── 4. Ancestor-walk violation check (ADR-M-003 pattern) ──
        let mut check_id = Some(p_entry_id);
        while let Some(id) = check_id {
            if rebalance::is_violated(&graph.vnodes, id) {
                graph.violations.push(id);
            }
            check_id = graph.vnodes.get(id.index()).parent;
        }
    }

    // ── 5. Update exposed/evictable flags ───────────────────────
    // Parent may transition Internal → SemiInternal or Terminal.
    {
        let p = graph.gnodes.get(parent_id.index());
        let parent_is_exposed = p.uncovered_range().is_some();
        let parent_is_evictable = p.is_terminal();
        let p_entry_id = p.entry.expect("evict_tip: parent must have V-entry (has dependents)");
        if let VKind::Entry {
            is_exposed,
            is_evictable,
            ..
        } = &mut graph.vnodes.get_mut(p_entry_id.index()).kind
        {
            *is_exposed = parent_is_exposed;
            *is_evictable = parent_is_evictable;
        }
        vtree::propagate_evictable_flags(&mut graph.vnodes, p_entry_id);
    }

    // ── 6. Remove V-entry via vtree_remove_leaf() ───────────────
    // Save the structural change point before removal: the parent
    // survives in the 3→2 case; in the 2-node collapse case the
    // parent is destroyed and the grandparent becomes the change point.
    //
    // Also capture the surviving sibling in the 2-node collapse case
    // for source-7 violation checking (§IDEA M-11.11.3).
    let v_parent = graph.vnodes.get(v_id.index()).parent;
    let (child_count, change_point, collapse_sibling) = v_parent.map_or((0, None, None), |p| {
        let count = match &graph.vnodes.get(p.index()).kind {
            VKind::Structural { children, .. } => children.len(),
            VKind::Entry { .. } => 0,
        };
        match count {
            3 => (3, Some(p), None), // 3→2: parent survives
            2 => {
                // 2-node collapse: find the surviving sibling.
                let sibling = match &graph.vnodes.get(p.index()).kind {
                    VKind::Structural { children, .. } => {
                        // Find the child that is NOT v_id.
                        let (c0, _) = children.get(0);
                        let (c1, _) = children.get(1);
                        if c0 == v_id { Some(c1) } else { Some(c0) }
                    }
                    VKind::Entry { .. } => None,
                };
                (2, graph.vnodes.get(p.index()).parent, sibling) // collapse: grandparent
            }
            _ => (count, None, None),
        }
    });

    graph.v_root = vtree::vtree_remove_leaf(&mut graph.vnodes, &mut graph.gnodes, v_id, graph.v_root);

    // ── 6b. Leaf-removal violation walk (§IDEA M-11.11.3, source 6) ─────
    // The intensity decrease propagates to all ancestors; at each
    // level the decreased ancestor may be a weaker uncle.  Walk
    // from the change point to the root, checking siblings' children.
    if let Some(start) = change_point {
        rebalance::push_leaf_removal_violations(&graph.vnodes, start, &mut graph.violations);
    }

    // ── 6c. Sibling-removal violation checks (sources 7 & 8) ────
    // When a sibling is removed, the remaining siblings' children lose
    // that sibling as a potential uncle. Two cases:
    //
    // - 2-node collapse (source 7, §IDEA M-11.11.3): The surviving sibling is
    //   re-parented to the grandparent. Its children now have entirely
    //   different uncles.
    //
    // - 2-node collapse cousins (source 9, §IDEA M-11.11.3): The OTHER children
    //   of the grandparent have their children potentially violated
    //   because those children's uncle changed from v_parent to sole.
    //
    // - 3→2 transition (source 8, §IDEA M-11.11.3): The remaining siblings'
    //   children lose the removed entry as a potential uncle. Their
    //   max_uncle may decrease.
    match child_count {
        2 => {
            // Collapse: check the surviving sibling's children (source 7).
            if let Some(sole) = collapse_sibling {
                tracing::debug!(sole = sole.index(), "evict_tip: calling push_collapse_violations");
                rebalance::push_collapse_violations(&graph.vnodes, sole, &mut graph.violations);

                // Also check the cousins' children (source 9).
                // change_point is the grandparent in the collapse case.
                if let Some(grandparent) = change_point {
                    tracing::debug!(
                        sole = sole.index(),
                        grandparent = grandparent.index(),
                        "evict_tip: calling push_cousin_violations (source 9)",
                    );
                    rebalance::push_cousin_violations(&graph.vnodes, sole, grandparent, &mut graph.violations);
                }
            }
        }
        3 => {
            // 3→2: check all remaining children of the parent.
            // v_id is already removed from the parent's children list,
            // so we check all current children.
            if let Some(p) = v_parent {
                tracing::debug!(parent = p.index(), "evict_tip: calling push_remaining_sibling_violations");
                rebalance::push_remaining_sibling_violations(
                    &graph.vnodes,
                    p,
                    v_id, // already removed, so filter is a no-op
                    &mut graph.violations,
                );
            }
        }
        _ => {}
    }

    // ── 6d. Violation source diagnosis ──────────────────────────
    // For any missed violations, analyze exactly why they became
    // violated to identify the missing source.
    if tracing::enabled!(tracing::Level::ERROR) {
        let ctx = crate::diagnostic::EvictionContext {
            evicted_parent: v_parent,
            evicted_parent_child_count: child_count,
            collapse_sibling,
        };
        let all_violated = rebalance::find_violated_nodes(&graph.vnodes);
        let queued: std::collections::HashSet<usize> = graph.violations.iter().map(|v| v.index()).collect();
        for v in all_violated {
            if !queued.contains(&v.index()) {
                crate::diagnostic::diagnose_missed_violation(&graph.vnodes, v, &ctx);
            }
        }
    }

    // ── 7. Deallocate G-node and decrement count ────────────────
    // Snapshot parent state before deallocation for plateau maintenance.
    #[cfg(feature = "dynamic-contour-tracking")]
    let parent_snapshot = {
        let pg = graph.gnodes.get(parent_id.index());
        (pg.state(), pg.lo, pg.hi)
    };

    graph.gnodes.dealloc(gnode_id.index());
    graph.node_count -= 1;
    // The evicted node was a terminal: −1.
    // If the parent is now terminal (both children gone): +1 → net 0.
    graph.terminal_count -= 1;
    if graph.gnodes.get(parent_id.index()).is_terminal() {
        graph.terminal_count += 1;
    }

    // 8. Plateau maintenance (no-op without `plateau` feature).
    #[cfg(feature = "dynamic-contour-tracking")]
    {
        let (parent_state_after, parent_lo, parent_hi) = parent_snapshot;
        plateau_after_evict(graph, gnode_id, parent_id, parent_state_after, parent_lo, parent_hi);

        if tracing::enabled!(tracing::Level::DEBUG) {
            crate::diagnostic::audit_plateau_consistency(
                graph,
                "POST-EVICT",
                Some(&crate::diagnostic::PlateauAuditContext {
                    parent_id,
                    parent_state: parent_state_after,
                }),
            );
        }
    }
}

/// Plateau maintenance after evicting a terminal G-node.
///
/// The evicted terminal may or may not have been a basis element.
/// If it was covered by an ancestor balanced internal, that
/// ancestor is the basis element and must be decomposed (same
/// pattern as `catalytic_split`'s ancestor walk).
///
/// Three cases for the parent's covering basis element:
///  (a) Parent itself is a basis element → remove, displace
///      surviving child (if `SemiInternal`), place parent.
///  (b) An ancestor is a basis element → decompose: remove
///      ancestor, displace all siblings along the path
///      (including surviving child), place parent.
///  (c) No ancestor is a basis element → the parent's region
///      was covered by *descendant* basis elements (from prior
///      decompositions). Just place parent; no decomposition.
///
/// The parent is placed BEFORE displaced elements so that its
/// newly-created plateau acts as a spatial boundary.  After all
/// placements, a tiling repair relocates any elements from a
/// neighbouring plateau that ended up on the wrong side of the
/// parent's boundary.
#[cfg(feature = "dynamic-contour-tracking")]
#[allow(clippy::too_many_lines)]
fn plateau_after_evict<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &mut GvGraph<C, V, N>,
    gnode_id: crate::handle::GNodeId,
    parent_id: crate::handle::GNodeId,
    parent_state_after: GState,
    parent_lo: C,
    parent_hi: C,
) {
    let _span = tracing::debug_span!(
        "plateau_after_evict",
        gnode = gnode_id.index(),
        parent = parent_id.index(),
        ?parent_state_after,
    )
    .entered();

    // Step 1 — Remove evicted child from basis (if present).
    // NOTE: we defer fixup_plateau until AFTER the ancestor walk
    // so the BTreeMap tile structure remains stable for the
    // tile coverage check.
    let evicted_key = graph.plateau_basis.remove(gnode_id);

    // Step 2 — Find the covering basis element for the parent's
    // region. Collect displaced nodes (thatched elements that
    // need their own basis membership).
    let mut displaced: Vec<crate::handle::GNodeId> = Vec::new();
    let mut displaced_extra: Vec<(crate::handle::GNodeId, u32)> = Vec::new();

    let ancestor_key = if let Some(key) = graph.plateau_basis.remove(parent_id) {
        // Case (a): parent was the basis element.

        // If parent was Internal → SemiInternal, the surviving
        // child was covered by the parent and needs its own
        // basis element.
        if parent_state_after == GState::SemiInternal {
            let g = graph.gnodes.get(parent_id.index());
            if let Some(sib) = g.left.or(g.right) {
                // Remove from current plateau if already a basis element.
                if let Some(ok) = graph.plateau_basis.remove(sib) {
                    graph.fixup_plateau(ok);
                }
                displaced.push(sib);
            }
        }

        Some(key)
    } else {
        // Walk up from parent to find the ancestor basis element
        // whose plateau tile actually covers the parent's region.
        let mut path: Vec<crate::handle::GNodeId> = vec![parent_id];
        let mut cur = graph.gnodes.get(parent_id.index()).parent;
        let mut found = None;

        // The parent's plateau key (where it wants to live).
        let parent_key = {
            let pg = graph.gnodes.get(parent_id.index());
            crate::plateau::basis_edge_of(pg)
        };

        while let Some(anc) = cur {
            if let Some(&anc_key) = graph.plateau_basis.plateau_key(anc).as_ref() {
                // Candidate ancestor found. Check if its tile
                // actually covers the parent's basis edge:
                // tile = [anc_key, next_key_in_btreemap)
                let next_key = graph
                    .plateaus
                    .range((std::ops::Bound::Excluded(anc_key), std::ops::Bound::Unbounded))
                    .next()
                    .map(|(&k, _)| k);
                let tile_covers = parent_key >= anc_key && next_key.is_none_or(|nk| parent_key < nk);

                if tile_covers {
                    // Case (b): ancestor's tile covers parent.
                    // Now remove it and decompose.
                    graph.plateau_basis.remove(anc);

                    // Surviving child at parent level.
                    if parent_state_after == GState::SemiInternal {
                        let g = graph.gnodes.get(parent_id.index());
                        if let Some(sib) = g.left.or(g.right) {
                            if let Some(ok) = graph.plateau_basis.remove(sib) {
                                graph.fixup_plateau(ok);
                            }
                            displaced.push(sib);
                        }
                    }

                    // Siblings along the walk path (parent → ancestor).
                    for &path_node in &path {
                        let par = graph
                            .gnodes
                            .get(path_node.index())
                            .parent
                            .expect("path node must have a parent");
                        let pg = graph.gnodes.get(par.index());
                        let sibling = if pg.left == Some(path_node) { pg.right } else { pg.left };
                        if let Some(sib_id) = sibling {
                            if displaced.contains(&sib_id) {
                                continue;
                            }
                            if let Some(ok) = graph.plateau_basis.remove(sib_id) {
                                graph.fixup_plateau(ok);
                            }
                            displaced.push(sib_id);
                        }
                    }

                    found = Some(anc_key);
                    break;
                }
                // Ancestor is a basis element but its tile doesn't
                // cover the parent → skip, continue walking.
            }
            path.push(anc);
            cur = graph.gnodes.get(anc.index()).parent;
        }

        // Case (c): no covering ancestor found — descendant-covered.
        found
    };

    // Step 3 — Fixup: now that the ancestor walk is complete, we
    // can safely fix up the evicted child's old plateau AND the
    // ancestor's old plateau (if found).
    if let Some(ek) = evicted_key {
        graph.fixup_plateau(ek);
    }
    if let Some(ak) = ancestor_key {
        // Don't double-fixup if the ancestor was the same plateau.
        if evicted_key != Some(ak) {
            graph.fixup_plateau(ak);
        }
    }

    // Step 4+5 combined — Merge parent into displaced list, collect
    // all basis elements, sort by BasisEdge, place left-to-right
    // (ADR-M-031 §3a).
    //
    // Before placing, evacuate any existing basis elements whose
    // plateau is a right-neighbour of the parent at the same depth.
    // Without this, `place_basis_element` can merge the parent with
    // an existing right-neighbour before deeper displaced elements
    // (which belong *between* them) have been placed — producing a
    // plateau that `build_plateaus`'s strict sorted sweep would
    // never create.
    let parent_depth = match parent_state_after {
        GState::Terminal | GState::SemiInternal => crate::gtree::gnode_depth_from_interval(parent_lo, parent_hi, N),
        GState::Internal => {
            unreachable!("evict_tip: parent cannot remain Internal after eviction")
        }
    };

    let parent_be = crate::plateau::BasisEdge(parent_lo);

    // Evacuate right-adjacent same-depth plateaus that could be
    // incorrectly merged with the parent during sorted placement.
    // We look for plateaus whose BasisEdge is in [parent_hi, ...) and
    // at the same depth, that are spatially adjacent (start <= parent_hi).
    {
        let right_keys: Vec<crate::plateau::BasisEdge<C>> = graph
            .plateaus
            .range(crate::plateau::BasisEdge(parent_hi)..)
            .take_while(|(_, p)| p.start.total_cmp(&parent_hi) != std::cmp::Ordering::Greater)
            .filter(|(_, p)| p.depth == parent_depth)
            .map(|(&k, _)| k)
            .collect();
        for rk in right_keys {
            let members: Vec<crate::handle::GNodeId> = graph.plateau_basis.basis_elements(&rk).iter().copied().collect();
            if !members.is_empty() {
                tracing::trace!(
                    ?rk,
                    parent_depth,
                    members = ?members.iter().map(|g| g.index()).collect::<Vec<_>>(),
                    "evacuating right-adjacent same-depth plateau for evict placement",
                );
                for &m in &members {
                    graph.plateau_basis.remove(m);
                }
                graph.plateaus.remove(&rk);
                for &m in &members {
                    graph.collect_subtree_basis_elements(m, &mut displaced_extra);
                }
            }
        }

        // Also evacuate left-adjacent same-depth plateaus.
        let left_keys: Vec<crate::plateau::BasisEdge<C>> = graph
            .plateaus
            .range(..parent_be)
            .rev()
            .take_while(|(_, p)| p.end.total_cmp(&parent_lo) != std::cmp::Ordering::Less)
            .filter(|(_, p)| p.depth == parent_depth)
            .map(|(&k, _)| k)
            .collect();
        for lk in left_keys {
            let members: Vec<crate::handle::GNodeId> = graph.plateau_basis.basis_elements(&lk).iter().copied().collect();
            if !members.is_empty() {
                tracing::trace!(
                    ?lk,
                    parent_depth,
                    members = ?members.iter().map(|g| g.index()).collect::<Vec<_>>(),
                    "evacuating left-adjacent same-depth plateau for evict placement",
                );
                for &m in &members {
                    graph.plateau_basis.remove(m);
                }
                graph.plateaus.remove(&lk);
                for &m in &members {
                    graph.collect_subtree_basis_elements(m, &mut displaced_extra);
                }
            }
        }
    }

    let mut to_place = Vec::new();
    to_place.push((parent_id, parent_depth));
    for &sib_id in &displaced {
        graph.collect_subtree_basis_elements(sib_id, &mut to_place);
    }
    to_place.extend(displaced_extra);
    graph.place_sorted(&mut to_place);
}

/// Scan the V-Tree for eviction candidates (ADR-M-013, ADR-M-015).
///
/// Pre-order DFS from the V-root. Prunes subtrees where
/// `has_evictable == false`. Collects eligible `VNodeId`s:
/// entries with `depth_V > D_evict`, `is_evictable == true`,
/// and backing G-node ≠ G-root.
///
/// Returns a `Vec<VNodeId>` of candidates (Phase 1 of the
/// two-phase collect-then-evict pattern).
pub fn scan_for_candidates<C: Coordinate, V: Accumulator, const N: u32>(graph: &GvGraph<C, V, N>) -> Vec<VNodeId> {
    let _span = tracing::trace_span!("scan_for_candidates").entered();
    let mut candidates = Vec::new();
    if let Some(v_root) = graph.v_root {
        scan_dfs(graph, v_root, 0, &mut candidates);
    }
    tracing::trace!(candidates = candidates.len(), "scan complete");
    candidates
}

/// Internal DFS worker for `scan_for_candidates`.
fn scan_dfs<C: Coordinate, V: Accumulator, const N: u32>(
    graph: &GvGraph<C, V, N>,
    v_id: VNodeId,
    depth: u32,
    candidates: &mut Vec<VNodeId>,
) {
    let node = graph.vnodes.get(v_id.index());
    match &node.kind {
        VKind::Entry { gnode, is_evictable, .. } => {
            if depth > graph.live_depth_evict && *is_evictable && *gnode != graph.g_root {
                candidates.push(v_id);
            }
        }
        VKind::Structural { children, has_evictable } => {
            // Prune: no evictable entries in this subtree.
            if !has_evictable {
                return;
            }
            for i in 0..children.len() {
                let (child_id, _) = children.get(i);
                scan_dfs(graph, child_id, depth + 1, candidates);
            }
        }
    }
}

#[cfg(test)]
mod tests {}
