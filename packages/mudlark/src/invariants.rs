// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Invariant checker for the dual-tree value-stratified index.
//!
//! Validates every structural invariant defined in idea.md as a
//! post-condition. Called after every mutation in every test.
//!
//! The function itself is unconditionally compiled so integration
//! tests in `tests/` can call it.

use crate::gnode::GState;
use crate::graph::GvGraph;
#[cfg(feature = "dynamic-contour-tracking")]
use crate::graph::uniform_contour_depth_of;
use crate::gtree::gnode_depth_from_interval;
use crate::handle::{GNodeId, VNodeId};
#[cfg(feature = "dynamic-contour-tracking")]
use crate::plateau::BasisEdge;
use crate::rebalance::is_violated;
use crate::traits::{Accumulator, Coordinate, Inspectable};
use crate::vnode::VKind;

// ── Formatting helpers ──────────────────────────────────────────────

/// Single-character label for a `GState` value.
const fn state_label(s: GState) -> &'static str {
    match s {
        GState::Terminal => "T",
        GState::SemiInternal => "S",
        GState::Internal => "I",
    }
}

/// Format an `Option<GNodeId>` as its index or a fallback string.
fn fmt_optional_gnode(opt: Option<GNodeId>, fallback: &str) -> String {
    opt.map_or_else(|| fallback.to_string(), |id| format!("{}", id.index()))
}

/// Human-readable ancestor depth name (1 = parent, 2 = grandparent, …).
fn ancestor_name(depth: usize) -> String {
    match depth {
        1 => "parent".to_string(),
        2 => "grandparent".to_string(),
        3 => "great-grandparent".to_string(),
        n => format!("{}x-great-grandparent", n - 2),
    }
}

/// Validate all structural invariants of the dual-tree index.
///
/// Intended for use in tests only (`#[cfg(test)]` callers), but the
/// function itself is unconditionally compiled so integration tests
/// in `tests/` can call it.
///
/// # Panics
///
/// Panics with a descriptive message listing every invariant that is
/// violated.  The panic message includes all failures, not just the
/// first.
pub fn assert_invariants<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>) {
    let errors = check_all_invariants(graph);
    if !errors.is_empty() {
        let msg = errors.join("\n");
        panic!("Invariant violations ({} total):\n{msg}", errors.len());
    }
}

/// Check only plateau-related invariants. Used for debugging.
#[cfg(feature = "dynamic-contour-tracking")]
#[allow(dead_code)]
pub fn check_plateau_only<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    check_plateau_btreemap_key_consistency(graph, errors);
    check_plateau_basis_consistency(graph, errors);
    check_p_i1_i_keys_are_contour_steps(graph, errors);
    check_p_i1_ii_tile_contiguity(graph, errors);
    check_p_i1_iii_run_contains_tile(graph, errors);
    check_p_i2_basis_minimality(graph, errors);
    check_p_i3_basis_disjointness(graph, errors);
    check_p_i4_thatch_one_hop(graph, errors);
}

/// Check only P-I3 basis disjointness. Used for debugging.
#[cfg(feature = "dynamic-contour-tracking")]
#[allow(dead_code)]
pub fn check_p_i3_only<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    check_p_i3_basis_disjointness(graph, errors);
}

// ── Diagnostic dump ─────────────────────────────────────────────────

/// Describe this node's semi-internal lineage (is it a child of a
/// semi-internal, or does it have a grandparent that is a semi-internal
/// child, etc.).
fn semi_internal_lineage<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    gnode: GNodeId,
) -> String {
    let mut parts = Vec::new();

    // Check if this node is a child of a semi-internal.
    let g = graph.gnodes().get(gnode.index());
    if let Some(parent_id) = g.parent {
        let parent = graph.gnodes().get(parent_id.index());
        if parent.state() == GState::SemiInternal {
            parts.push(format!("SI-child (par=G({}))", parent_id.index()));
        }
    }

    // Walk ancestors: for each, if THEY are a child of a semi-internal,
    // that means this node has an ancestor who is an SI-child.
    let mut cur = g.parent;
    let mut depth = 1; // 1=parent, 2=grandparent, etc.
    while let Some(anc_id) = cur {
        let anc = graph.gnodes().get(anc_id.index());
        if let Some(anc_parent_id) = anc.parent {
            let anc_parent = graph.gnodes().get(anc_parent_id.index());
            if anc_parent.state() == GState::SemiInternal {
                parts.push(format!(
                    "{} is SI-child (G({}) under G({}))",
                    ancestor_name(depth),
                    anc_id.index(),
                    anc_parent_id.index()
                ));
            }
        }
        cur = anc.parent;
        depth += 1;
    }

    if parts.is_empty() {
        String::new()
    } else {
        format!("  [{}]", parts.join("; "))
    }
}

/// Dump the full G-tree structure to a `String` for debugging.
///
/// Shows each G-node with its interval, state, parent, children,
/// sum, own, V-entry, and basis membership. The tree is printed
/// in arena-allocation order (by gnode index).
#[allow(dead_code)]
pub fn dump_gtree<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>) -> String {
    use std::fmt::Write;
    let mut out = String::new();
    writeln!(
        out,
        "═══ G-Tree dump (root=GNodeId({}), {} nodes) ═══",
        graph.g_root().index(),
        graph.node_count()
    )
    .unwrap();

    for (idx, g) in graph.gnodes().iter_occupied() {
        let gnode_id = GNodeId::from_index(idx);
        let state = state_label(g.state());
        let depth = gnode_depth_from_interval(g.lo, g.hi, N);
        let parent_str = fmt_optional_gnode(g.parent, "None");
        let left_str = fmt_optional_gnode(g.left, "_");
        let right_str = fmt_optional_gnode(g.right, "_");
        #[cfg(feature = "dynamic-contour-tracking")]
        let basis_str = graph
            .plateau_basis()
            .plateau_key(gnode_id)
            .map_or_else(|| "(not basis)".to_string(), |k| format!("basis={k:?}"));
        #[cfg(not(feature = "dynamic-contour-tracking"))]
        let basis_str = "";
        let lineage = semi_internal_lineage(graph, gnode_id);

        writeln!(out,
            "  G({idx:>3}) {state} [{:>6.1}, {:>6.1})  d={depth}  par={parent_str}  L={left_str} R={right_str}  sum={:>8.1} own={:>8.1}  {basis_str}{lineage}",
            g.lo.to_f64(), g.hi.to_f64(), g.sum.to_f64_approx(), g.own.to_f64_approx(),
        ).unwrap();
    }
    out
}

/// Dump the full plateau map to a `String` for debugging.
///
/// Shows each plateau with its basis edge, depth, start/end, sum,
/// and all basis elements with their spatial spans and states.
/// Also flags P-I3 violations inline.
#[cfg(feature = "dynamic-contour-tracking")]
#[allow(dead_code)]
pub fn dump_plateaus<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>) -> String {
    use std::fmt::Write;
    let mut out = String::new();

    // Access the raw field directly — this function is called during
    // incremental mutation hooks where the map may not yet match
    // `build_plateaus()`.  Using `&graph.plateaus` would trigger
    // the debug_assert_eq that lives there.
    #[cfg(feature = "dynamic-contour-tracking")]
    let plateaus = &graph.plateaus;
    #[cfg(not(feature = "dynamic-contour-tracking"))]
    let plateaus = graph.build_plateaus();

    writeln!(
        out,
        "═══ Plateau dump ({} plateaus, {} basis members) ═══",
        plateaus.len(),
        graph.plateau_basis().basis_count()
    )
    .unwrap();

    for (key, plateau) in plateaus {
        // Tiling range: from this key to the next key.
        let next_key = plateaus
            .range(std::ops::RangeFrom {
                start: crate::plateau::BasisEdge(plateau.end),
            })
            .find(|&(k, _)| *k != *key)
            .map(|(k, _)| k.0.to_f64());
        let tile_end = next_key.map_or_else(|| "∞".to_string(), |v| format!("{v:.1}"));

        writeln!(
            out,
            "  Plateau {key:?}  depth={}  tile=[{:.1}, {tile_end})  span=[{:.1},{:.1})  sum={:.1}",
            plateau.depth,
            key.0.to_f64(),
            plateau.start.to_f64(),
            plateau.end.to_f64(),
            plateau.sum.to_f64_approx(),
        )
        .unwrap();

        let elements = graph.plateau_basis().basis_elements(key);
        let raw_ids: Vec<usize> = elements.iter().map(|g| g.index()).collect();
        writeln!(out, "    basis_elements (raw): {raw_ids:?}").unwrap();

        for &gid in elements {
            if !graph.gnodes().is_occupied(gid.index()) {
                writeln!(out, "    !! DANGLING GNodeId({}) — slot deallocated !!", gid.index()).unwrap();
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            let state = state_label(g.state());
            let g_depth = gnode_depth_from_interval(g.lo, g.hi, N);
            let contour_depth = match g.state() {
                GState::Terminal | GState::SemiInternal => g_depth,
                GState::Internal => g_depth + 1,
            };
            let parent_str = g.parent.map_or_else(|| "None".to_string(), |p| format!("G({})", p.index()));
            let left_str = g.left.map_or_else(|| "_".to_string(), |l| format!("G({})", l.index()));
            let right_str = g.right.map_or_else(|| "_".to_string(), |r| format!("G({})", r.index()));
            let lineage = semi_internal_lineage(graph, gid);

            writeln!(out,
                "    G({:>3}) {state} [{:>6.1}, {:>6.1})  d={g_depth} contour_d={contour_depth}  par={parent_str} L={left_str} R={right_str}  sum={:.1}{lineage}",
                gid.index(), g.lo.to_f64(), g.hi.to_f64(), g.sum.to_f64_approx(),
            ).unwrap();
        }
    }

    // Back-map dump for cross-reference.
    writeln!(out, "  ─── Back map ({} entries) ───", graph.plateau_basis().back_map().len()).unwrap();
    let mut back_entries: Vec<_> = graph
        .plateau_basis()
        .back_map()
        .iter()
        .map(|(&gid, &key)| (gid.index(), key))
        .collect();
    back_entries.sort_by_key(|(idx, _)| *idx);
    for (idx, key) in &back_entries {
        writeln!(out, "    G({idx}) → {key:?}").unwrap();
    }

    // Append P-I3 check inline.
    let mut p_i3_errors = Vec::new();
    check_p_i3_basis_disjointness(graph, &mut p_i3_errors);
    if !p_i3_errors.is_empty() {
        writeln!(out, "  ─── P-I3 violations ───").unwrap();
        for e in &p_i3_errors {
            writeln!(out, "    !! {e}").unwrap();
        }
    }

    // Append P-I4 check.
    let mut p_i4_errors = Vec::new();
    check_p_i4_thatch_one_hop(graph, &mut p_i4_errors);
    if !p_i4_errors.is_empty() {
        writeln!(out, "  ─── P-I4 violations ───").unwrap();
        for e in &p_i4_errors {
            writeln!(out, "    !! {e}").unwrap();
        }
    }

    out
}

/// Check all invariants and return errors as a Vec instead of panicking.
///
/// Useful for programmatic testing — the caller can inspect which
/// invariants failed and dump diagnostic info before asserting.
#[allow(dead_code)]
pub fn check_all_invariants<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>) -> Vec<String> {
    let mut errors: Vec<String> = Vec::new();

    check_g_i1_summation(graph, &mut errors);
    check_g_i2_variable_fanout(graph, &mut errors);
    check_g_i4_entry_consistency(graph, &mut errors);
    check_v_i1_structural_sum(graph, &mut errors);
    check_v_i2_branching_factor(graph, &mut errors);
    check_v_i3_max_uncle(graph, &mut errors);
    check_v_i5_entry_leaf(graph, &mut errors);
    check_v_i6_exposed_flag(graph, &mut errors);
    check_v_i6b_evictable_flag(graph, &mut errors);
    check_v_i7_structural_flag(graph, &mut errors);
    check_clean_accounting(graph, &mut errors);
    check_parent_link_consistency(graph, &mut errors);
    check_v_root_consistency(graph, &mut errors);
    check_node_count_consistency(graph, &mut errors);
    check_terminal_count_consistency(graph, &mut errors);
    check_depth_gate_invariants(graph, &mut errors);
    check_hard_budget(graph, &mut errors);
    #[cfg(feature = "dynamic-contour-tracking")]
    {
        check_plateau_btreemap_key_consistency(graph, &mut errors);
        check_plateau_basis_consistency(graph, &mut errors);
        check_plateau_sum_consistency(graph, &mut errors);
        check_plateau_depth_consistency(graph, &mut errors);
        check_p_i1_i_keys_are_contour_steps(graph, &mut errors);
        check_p_i1_ii_tile_contiguity(graph, &mut errors);
        check_p_i1_iii_run_contains_tile(graph, &mut errors);
        check_p_i2_basis_minimality(graph, &mut errors);
        check_p_i3_basis_disjointness(graph, &mut errors);
        check_p_i4_thatch_one_hop(graph, &mut errors);
        check_p_i5_thatch_depth(graph, &mut errors);
    }

    errors
}

// ── G-I1: Summation invariant ───────────────────────────────────────

fn check_g_i1_summation<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, g) in graph.gnodes().iter_occupied() {
        let left_sum = g.left.map_or(0.0, |l| graph.gnodes().get(l.index()).sum.to_f64_approx());
        let right_sum = g.right.map_or(0.0, |r| graph.gnodes().get(r.index()).sum.to_f64_approx());
        let expected = g.own.to_f64_approx() + left_sum + right_sum;
        let actual = g.sum.to_f64_approx();
        if (expected - actual).abs() > 1e-9 {
            errors.push(format!(
                "G-I1 violated at G-node {idx}: expected sum={expected}, actual sum={actual} \
                 (own={}, left_sum={left_sum}, right_sum={right_sum})",
                g.own.to_f64_approx()
            ));
        }
    }
}

// ── G-I2: Variable fanout ───────────────────────────────────────────

fn check_g_i2_variable_fanout<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, g) in graph.gnodes().iter_occupied() {
        let count = usize::from(g.left.is_some()) + usize::from(g.right.is_some());
        if count > 2 {
            errors.push(format!("G-I2 violated at G-node {idx}: {count} children (max 2)"));
        }
    }
}

// ── G-I4: Entry consistency ─────────────────────────────────────────

fn check_g_i4_entry_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, g) in graph.gnodes().iter_occupied() {
        if let Some(v_id) = g.entry {
            if !graph.vnodes().is_occupied(v_id.index()) {
                errors.push(format!(
                    "G-I4 violated at G-node {idx}: entry V-node {} is not occupied",
                    v_id.index()
                ));
                continue;
            }
            let v = graph.vnodes().get(v_id.index());

            // Intensity must match own.
            let g_own = g.own.to_f64_approx();
            let v_int = v.intensity.to_f64_approx();
            if (g_own - v_int).abs() > 1e-9 {
                errors.push(format!(
                    "G-I4 violated at G-node {idx}: g.own={g_own}, entry.intensity={v_int}"
                ));
            }

            // Back-link: entry must be a VKind::Entry pointing back.
            match &v.kind {
                VKind::Entry { gnode, .. } => {
                    let g_id = GNodeId::from_index(idx);
                    if *gnode != g_id {
                        errors.push(format!(
                            "G-I4 violated at G-node {idx}: entry's gnode={gnode:?}, expected {g_id:?}"
                        ));
                    }
                }
                VKind::Structural { .. } => {
                    errors.push(format!(
                        "G-I4 violated at G-node {idx}: entry is a structural V-node, not an entry"
                    ));
                }
            }
        }
    }
}

// ── V-I1: Structural sum consistency ────────────────────────────────

fn check_v_i1_structural_sum<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Structural { children, .. } = &v.kind {
            // Check that node intensity equals sum of children's intensities.
            let mut sum = 0.0_f64;
            for i in 0..children.len() {
                let (child_id, cached_int) = children.get(i);

                // Verify cached intensity matches authoritative child intensity.
                if !graph.vnodes().is_occupied(child_id.index()) {
                    errors.push(format!(
                        "V-I1 violated at V-node {idx}: child {} is not occupied",
                        child_id.index()
                    ));
                    continue;
                }
                let actual_int = graph.vnodes().get(child_id.index()).intensity;
                if (cached_int.to_f64_approx() - actual_int.to_f64_approx()).abs() > 1e-9 {
                    errors.push(format!(
                        "V-I1 cached intensity mismatch at V-node {idx}, child {}: \
                         cached={}, actual={}",
                        child_id.index(),
                        cached_int.to_f64_approx(),
                        actual_int.to_f64_approx()
                    ));
                }
                sum += cached_int.to_f64_approx();
            }

            let node_int = v.intensity.to_f64_approx();
            if (node_int - sum).abs() > 1e-9 {
                errors.push(format!(
                    "V-I1 violated at V-node {idx}: intensity={node_int}, sum of children={sum}"
                ));
            }
        }
    }
}

// ── V-I2: Branching factor ──────────────────────────────────────────

fn check_v_i2_branching_factor<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Structural { children, .. } = &v.kind {
            let len = children.len();
            if len != 2 && len != 3 {
                errors.push(format!("V-I2 violated at V-node {idx}: {len} children (must be 2 or 3)"));
            }
        }
    }
}

// ── V-I3: Max-uncle constraint ──────────────────────────────────────

fn check_v_i3_max_uncle<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        let v_id = VNodeId::from_index(idx);
        if is_violated(graph.vnodes(), v_id) {
            let int = v.intensity.to_f64_approx();
            let uncle = crate::rebalance::max_uncle_intensity(graph.vnodes(), v_id).map_or(f64::NAN, Inspectable::to_f64_approx);
            errors.push(format!("V-I3 violated at V-node {idx}: intensity={int}, max_uncle={uncle}"));
        }
    }
}

// ── V-I5: Entry-leaf ────────────────────────────────────────────────

fn check_v_i5_entry_leaf<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    // Every entry should not appear as a structural node's child
    // in a way that itself has children. Actually, the simpler check:
    // every V-node that is an Entry should not have any child in the
    // V-tree (ensured by VKind discriminant). And every V-node that
    // appears as a child in a PackedChildren must have its parent
    // pointing back correctly. We check parent-link consistency
    // separately, so here just verify entries have no children
    // (structurally guaranteed) and structurals are not entries.
    //
    // Additionally: verify no entry is referenced as a structural child
    // that itself is structural. (This is structurally guaranteed by
    // the type system, but document the intent.)
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Structural { children, .. } = &v.kind {
            for i in 0..children.len() {
                let (child_id, _) = children.get(i);
                if !graph.vnodes().is_occupied(child_id.index()) {
                    errors.push(format!(
                        "V-I5 violated at V-node {idx}: child {} is not occupied",
                        child_id.index()
                    ));
                }
            }
        }
    }
}

// ── V-I6: Terminal flag on entries ──────────────────────────────────

fn check_v_i6_exposed_flag<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Entry { gnode, is_exposed, .. } = &v.kind {
            if !graph.gnodes().is_occupied(gnode.index()) {
                errors.push(format!(
                    "V-I6 violated at V-node {idx}: backing G-node {} is not occupied",
                    gnode.index()
                ));
                continue;
            }
            let g = graph.gnodes().get(gnode.index());
            let expected = g.uncovered_range().is_some();
            if *is_exposed != expected {
                errors.push(format!(
                    "V-I6 violated at V-node {idx}: is_exposed={is_exposed}, \
                     expected={expected} (state={:?})",
                    g.state()
                ));
            }
        }
    }
}

// ── V-I6b: Evictable flag on entries ────────────────────────────────

fn check_v_i6b_evictable_flag<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Entry {
            gnode,
            is_evictable,
            is_exposed,
            ..
        } = &v.kind
        {
            if !graph.gnodes().is_occupied(gnode.index()) {
                continue; // Already caught by V-I6.
            }
            let g = graph.gnodes().get(gnode.index());
            let expected = g.is_terminal();
            if *is_evictable != expected {
                errors.push(format!(
                    "V-I6b violated at V-node {idx}: is_evictable={is_evictable}, \
                     expected={expected} (state={:?})",
                    g.state()
                ));
            }
            // Implied: is_evictable ⟹ is_exposed
            if *is_evictable && !*is_exposed {
                errors.push(format!(
                    "V-I6b violated at V-node {idx}: is_evictable=true but is_exposed=false"
                ));
            }
        }
    }
}

// ── V-I7: Structural has_evictable flag ──────────────────────────

fn check_v_i7_structural_flag<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (idx, v) in graph.vnodes().iter_occupied() {
        if let VKind::Structural { children, has_evictable } = &v.kind {
            let expected = (0..children.len()).any(|i| {
                let (child_id, _) = children.get(i);
                if !graph.vnodes().is_occupied(child_id.index()) {
                    return false;
                }
                let child = graph.vnodes().get(child_id.index());
                match &child.kind {
                    VKind::Entry { is_evictable, .. } => *is_evictable,
                    VKind::Structural { has_evictable, .. } => *has_evictable,
                }
            });
            if *has_evictable != expected {
                errors.push(format!(
                    "V-I7 violated at V-node {idx}: has_evictable={has_evictable}, \
                     expected={expected}"
                ));
            }
        }
    }
}

// ── Clean accounting ────────────────────────────────────────────────

fn check_clean_accounting<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let mut total_v = 0.0_f64;
    for (_, v) in graph.vnodes().iter_occupied() {
        if matches!(v.kind, VKind::Entry { .. }) {
            total_v += v.intensity.to_f64_approx();
        }
    }
    let g_root_sum = graph.gnodes().get(graph.g_root().index()).sum.to_f64_approx();
    if (total_v - g_root_sum).abs() > 1e-9 {
        errors.push(format!(
            "Clean accounting violated: V-entry sum={total_v}, G-root sum={g_root_sum}"
        ));
    }
}

// ── Parent link consistency ─────────────────────────────────────────

fn check_parent_link_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    // G-Tree parent links.
    for (idx, g) in graph.gnodes().iter_occupied() {
        let g_id = GNodeId::from_index(idx);
        if let Some(left) = g.left
            && graph.gnodes().is_occupied(left.index())
        {
            let child_parent = graph.gnodes().get(left.index()).parent;
            if child_parent != Some(g_id) {
                errors.push(format!(
                    "G-parent link: G-node {idx}'s left child {}'s parent is {:?}, expected {g_id:?}",
                    left.index(),
                    child_parent
                ));
            }
        }
        if let Some(right) = g.right
            && graph.gnodes().is_occupied(right.index())
        {
            let child_parent = graph.gnodes().get(right.index()).parent;
            if child_parent != Some(g_id) {
                errors.push(format!(
                    "G-parent link: G-node {idx}'s right child {}'s parent is {:?}, expected {g_id:?}",
                    right.index(),
                    child_parent
                ));
            }
        }
    }

    // V-Tree parent links.
    for (idx, v) in graph.vnodes().iter_occupied() {
        let v_id = VNodeId::from_index(idx);
        if let VKind::Structural { children, .. } = &v.kind {
            for i in 0..children.len() {
                let (child_id, _) = children.get(i);
                if graph.vnodes().is_occupied(child_id.index()) {
                    let child_parent = graph.vnodes().get(child_id.index()).parent;
                    if child_parent != Some(v_id) {
                        errors.push(format!(
                            "V-parent link: V-node {idx}'s child {}'s parent is {:?}, expected {v_id:?}",
                            child_id.index(),
                            child_parent
                        ));
                    }
                }
            }
        }

        // If this node has a parent, verify the parent lists it as a child.
        if let Some(p_id) = v.parent
            && graph.vnodes().is_occupied(p_id.index())
        {
            let parent = graph.vnodes().get(p_id.index());
            if let VKind::Structural { children, .. } = &parent.kind {
                if children.find_index(v_id).is_none() {
                    errors.push(format!(
                        "V-parent link: V-node {idx} has parent {}, but parent does not list it as a child",
                        p_id.index()
                    ));
                }
            } else {
                errors.push(format!(
                    "V-parent link: V-node {idx} has parent {}, but parent is not structural",
                    p_id.index()
                ));
            }
        }
    }
}

// ── V-root consistency ──────────────────────────────────────────────

fn check_v_root_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    if let Some(root_id) = graph.v_root() {
        if !graph.vnodes().is_occupied(root_id.index()) {
            errors.push(format!("V-root consistency: v_root {} is not occupied", root_id.index()));
            return;
        }
        let root = graph.vnodes().get(root_id.index());
        if root.parent.is_some() {
            errors.push(format!(
                "V-root consistency: v_root {} has parent {:?}, expected None",
                root_id.index(),
                root.parent
            ));
        }
    }
}

// ── Node count consistency ──────────────────────────────────────────

fn check_node_count_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let actual = graph.gnodes().count();
    let expected = graph.node_count();
    if actual != expected {
        errors.push(format!("Node count: graph.node_count()={expected}, arena count={actual}"));
    }
}

// ── Terminal count consistency ──────────────────────────────────────

fn check_terminal_count_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    #[allow(clippy::cast_possible_truncation)] // arena can't exceed u32::MAX G-nodes
    let actual = graph.gnodes().iter_occupied().filter(|(_, g)| g.is_terminal()).count() as u32;
    let expected = graph.terminal_count();
    if actual != expected {
        errors.push(format!(
            "Terminal count: graph.terminal_count()={expected}, arena walk={actual}"
        ));
    }
}

// ── Hard budget (ADR-M-018) ───────────────────────────────────────────

fn check_hard_budget<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    if let Some(budget) = graph.config().budget
        && graph.node_count() as usize > budget
    {
        errors.push(format!(
            "Hard budget violated (ADR-M-018): node_count ({}) > budget ({})",
            graph.node_count(),
            budget
        ));
    }
}

// ── D-I3: Depth gate invariants (Phase 3, ADR-M-017) ─────────────────

fn check_depth_gate_invariants<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let d_create = graph.depth_create();
    let d_evict = graph.depth_evict();
    let buffer = graph.depth_buffer();

    // D-I3: D_create < D_evict (strict ordering).
    if d_create >= d_evict {
        errors.push(format!(
            "D-I3: live_depth_create ({d_create}) must be < live_depth_evict ({d_evict})"
        ));
    }

    // Floor: D_evict >= buffer + 1.
    if d_evict < buffer + 1 {
        errors.push(format!(
            "D-I3 floor: live_depth_evict ({d_evict}) < depth_buffer ({buffer}) + 1"
        ));
    }

    // Buffer consistency: D_create == D_evict - buffer.
    // Only check if floor holds (avoids underflow).
    if d_evict >= buffer && d_create != d_evict - buffer {
        errors.push(format!(
            "D-I3 buffer: live_depth_create ({d_create}) != live_depth_evict ({d_evict}) - depth_buffer ({buffer})"
        ));
    }
}

// ══════════════════════════════════════════════════════════════════════
// Plateau invariant checks (ADR-M-026, Step 2F)
// ══════════════════════════════════════════════════════════════════════

// ── Plateau BTreeMap key consistency ────────────────────────────────

/// For each `(key, plateau)` in the `BTreeMap`: `plateau.basis_edge == key`.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_plateau_btreemap_key_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    for (&key, plateau) in &graph.plateaus {
        if plateau.basis_edge != key {
            errors.push(format!(
                "Plateau key consistency: BTreeMap key {key:?} != plateau.basis_edge {:?}",
                plateau.basis_edge
            ));
        }
    }
}

// ── Plateau basis consistency ───────────────────────────────────────

/// Bidirectional forward/back map consistency and liveness.
///
/// - Every forward entry `(key, gnodes)`: each gnode has `back[gnode] == key`.
/// - Every back entry `(gnode, key)`: `forward[key]` contains `gnode`.
/// - Every basis element references a live (occupied) arena slot.
/// - `plateaus.keys()` == `plateau_basis.forward.keys()`.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_plateau_basis_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();

    // Forward → back consistency + liveness.
    for (&key, gnodes) in pb.iter() {
        for &gid in gnodes {
            // Liveness check.
            if !graph.gnodes().is_occupied(gid.index()) {
                errors.push(format!(
                    "Plateau basis: basis element {gid:?} in plateau {key:?} is not a live arena slot"
                ));
                continue;
            }
            // Back-pointer check.
            match pb.plateau_key(gid) {
                Some(back_key) if back_key == key => {} // OK
                Some(back_key) => {
                    errors.push(format!(
                        "Plateau basis forward→back: {gid:?} in forward[{key:?}] but back[{gid:?}] = {back_key:?}"
                    ));
                }
                None => {
                    errors.push(format!(
                        "Plateau basis forward→back: {gid:?} in forward[{key:?}] but not in back map"
                    ));
                }
            }
        }
    }

    // Back → forward consistency.
    for (&gid, &key) in pb.back_map() {
        let elements = pb.basis_elements(&key);
        if !elements.contains(&gid) {
            errors.push(format!(
                "Plateau basis back→forward: back[{gid:?}] = {key:?} but {gid:?} not in forward[{key:?}]"
            ));
        }
    }

    // plateaus.keys() == plateau_basis.forward.keys()
    let btree_keys: Vec<_> = graph.plateaus.keys().copied().collect();
    let basis_keys: Vec<_> = pb.iter().map(|(&k, _)| k).collect();
    if btree_keys != basis_keys {
        errors.push(format!(
            "Plateau basis: plateaus.keys() ({} entries) != plateau_basis.forward.keys() ({} entries)",
            btree_keys.len(),
            basis_keys.len()
        ));
    }
}

// ── Plateau sum consistency ─────────────────────────────────────────

/// For each plateau: `p.sum == Σ gnodes[r].sum for r in basis_elements(key)`.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_plateau_sum_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();
    for (&key, plateau) in &graph.plateaus {
        let expected: f64 = pb
            .basis_elements(&key)
            .iter()
            .filter(|&&gid| graph.gnodes().is_occupied(gid.index()))
            .map(|&gid| graph.gnodes().get(gid.index()).sum.to_f64_approx())
            .sum();
        let actual = plateau.sum.to_f64_approx();
        if (expected - actual).abs() > 1e-9 {
            errors.push(format!(
                "Plateau sum: key {key:?}: expected sum={expected}, actual sum={actual}"
            ));
        }
    }
}

// ── Plateau depth consistency ───────────────────────────────────────

/// For each basis element in each plateau, verify depth matches `plateau.depth`.
///
/// - Terminal: `gnode_depth == p.depth`.
/// - Internal (balanced): `gnode_depth + 1 == p.depth` (contour is children's depth).
/// - Semi-internal: `gnode_depth == p.depth` (uncovered half at parent's depth).
#[cfg(feature = "dynamic-contour-tracking")]
fn check_plateau_depth_consistency<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();
    for (&key, plateau) in &graph.plateaus {
        for &gid in pb.basis_elements(&key) {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue; // Liveness already checked above.
            }
            let g = graph.gnodes().get(gid.index());
            let g_depth = gnode_depth_from_interval(g.lo, g.hi, N);
            let expected_depth = match g.state() {
                GState::Terminal | GState::SemiInternal => g_depth,
                GState::Internal => {
                    let Some(d) = crate::graph::uniform_contour_depth_of(graph.gnodes(), gid, N) else {
                        errors.push(format!(
                            "Plateau depth: key {key:?}, basis element {gid:?} (Internal): \
                             uniform_contour_depth_of returned None — internal basis \
                             element has non-uniform contour depth",
                        ));
                        continue;
                    };
                    d
                }
            };
            if expected_depth != plateau.depth {
                errors.push(format!(
                    "Plateau depth: key {key:?}, basis element {gid:?} ({:?}): \
                     element contributes depth {expected_depth}, plateau.depth={}",
                    g.state(),
                    plateau.depth,
                ));
            }
        }
    }
}

// ── P-I1: Deterministic tiling ─────────────────────────────────────

/// Compute the contour tile of a basis element: the interval of the
/// domain that this element covers in the contour step function.
///
/// - Terminal / balanced internal: full `[lo, hi)`.
/// - Semi-internal: the uncovered half-interval.
#[cfg(feature = "dynamic-contour-tracking")]
fn tile_of<C: Coordinate, V: Accumulator>(g: &crate::gnode::GNode<C, V>) -> (C, C) {
    match g.state() {
        GState::Terminal | GState::Internal => (g.lo, g.hi),
        GState::SemiInternal => g.uncovered_range().expect("semi-internal must have uncovered range"),
    }
}

/// Walk the G-tree contour and return the sorted step coordinates
/// where the contour depth changes.  The first step is always 0
/// (domain start).
#[cfg(feature = "dynamic-contour-tracking")]
fn contour_steps<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>) -> Vec<(C, u32)> {
    use crate::gtree::gnode_depth_from_interval;

    // Collect (lo, depth) for every contour cell (terminal or
    // null-child semi-internal half).
    let mut cells: Vec<(C, u32)> = Vec::new();
    let mut stack = vec![graph.g_root()];
    while let Some(gid) = stack.pop() {
        let g = graph.gnodes().get(gid.index());
        match g.state() {
            GState::Terminal => {
                let d = gnode_depth_from_interval(g.lo, g.hi, N);
                cells.push((g.lo, d));
            }
            GState::SemiInternal => {
                // The uncovered half is a contour cell.
                let (ulo, _uhi) = g.uncovered_range().unwrap();
                let d = gnode_depth_from_interval(g.lo, g.hi, N);
                cells.push((ulo, d));
                // Recurse into the surviving child.
                if let Some(l) = g.left {
                    stack.push(l);
                }
                if let Some(r) = g.right {
                    stack.push(r);
                }
            }
            GState::Internal => {
                if let Some(l) = g.left {
                    stack.push(l);
                }
                if let Some(r) = g.right {
                    stack.push(r);
                }
            }
        }
    }
    cells.sort_by(|a, b| a.0.total_cmp(&b.0));

    // Extract step coordinates: position 0 is always a step; after
    // that, every depth change is a step.
    let mut steps: Vec<(C, u32)> = Vec::new();
    for &(lo, depth) in &cells {
        if steps.is_empty() || steps.last().unwrap().1 != depth {
            steps.push((lo, depth));
        }
    }
    steps
}

/// **P-I1(i): Keys = contour steps.**
///
/// The `BTreeMap` keys must be exactly the set of contour step
/// coordinates — where the contour depth function changes value
/// (plus the domain origin 0).
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i1_i_keys_are_contour_steps<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let plateaus = &graph.plateaus;
    if plateaus.is_empty() {
        errors.push("P-I1(i): plateaus BTreeMap is empty (must have at least 1 plateau)".into());
        return;
    }

    let steps = contour_steps(graph);
    let btree_keys: Vec<C> = plateaus.keys().map(|k| k.0).collect();
    let step_coords: Vec<C> = steps.iter().map(|&(c, _)| c).collect();

    if btree_keys.len() != step_coords.len() {
        errors.push(format!(
            "P-I1(i): plateau count ({}) != contour step count ({})",
            btree_keys.len(),
            step_coords.len()
        ));
    }

    // Check each key matches and depth matches.
    for (i, (&(coord, depth), btree_coord)) in steps.iter().zip(btree_keys.iter()).enumerate() {
        if coord.total_cmp(btree_coord) != std::cmp::Ordering::Equal {
            errors.push(format!(
                "P-I1(i): step {i}: contour step at {coord:?}, BTreeMap key at {btree_coord:?}"
            ));
        }
        // Also verify depth matches.
        if let Some(plateau) = plateaus.get(&BasisEdge(coord))
            && plateau.depth != depth
        {
            errors.push(format!(
                "P-I1(i): step {i} at {coord:?}: contour depth={depth}, plateau.depth={}",
                plateau.depth
            ));
        }
    }

    // Consecutive plateaus must have different depths (maximality).
    let depths: Vec<u32> = plateaus.values().map(|p| p.depth).collect();
    for w in depths.windows(2) {
        if w[0] == w[1] {
            // Find the key for the second one.
            let keys: Vec<_> = plateaus.keys().collect();
            let idx = depths.windows(2).position(|d| d[0] == d[1]).unwrap();
            errors.push(format!(
                "P-I1(i) maximality: consecutive plateaus {:?} and {:?} both have depth {}",
                keys[idx],
                keys[idx + 1],
                w[0]
            ));
        }
    }
}

/// **P-I1(ii): Tile contiguity.**
///
/// The union of `tile(R)` over a plateau's basis elements must equal
/// `[key, next_key)` — the contour run assigned to this plateau.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i1_ii_tile_contiguity<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let plateaus = &graph.plateaus;
    let pb = graph.plateau_basis();
    let keys: Vec<BasisEdge<C>> = plateaus.keys().copied().collect();
    let domain_max = C::domain_max(N);

    for (i, &key) in keys.iter().enumerate() {
        let next_start = if i + 1 < keys.len() { keys[i + 1].0 } else { domain_max };

        let elements = pb.basis_elements(&key);
        if elements.is_empty() {
            errors.push(format!("P-I1(ii): plateau {key:?} has no basis elements"));
            continue;
        }

        // Collect tile intervals and sort.
        let mut tiles: Vec<(f64, f64)> = Vec::new();
        for &gid in elements {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            let (tlo, thi) = tile_of(g);
            tiles.push((tlo.to_f64(), thi.to_f64()));
        }
        tiles.sort_by(|a, b| a.0.total_cmp(&b.0));

        // Union must be [key.0, next_start).
        let expected_lo = key.0.to_f64();
        let expected_hi = next_start.to_f64();

        if tiles.is_empty() {
            errors.push(format!("P-I1(ii): plateau {key:?}: no live basis tiles"));
            continue;
        }

        // Check left boundary.
        if (tiles[0].0 - expected_lo).abs() > 1e-12 {
            errors.push(format!(
                "P-I1(ii): plateau {key:?}: tile union starts at {}, expected {expected_lo}",
                tiles[0].0
            ));
        }

        // Check right boundary.
        let last_hi = tiles.last().unwrap().1;
        if (last_hi - expected_hi).abs() > 1e-12 {
            errors.push(format!(
                "P-I1(ii): plateau {key:?}: tile union ends at {last_hi}, expected {expected_hi}"
            ));
        }

        // Check no internal gaps (tiles must be contiguous).
        for w in tiles.windows(2) {
            if w[1].0 - w[0].1 > 1e-12 {
                errors.push(format!(
                    "P-I1(ii): plateau {key:?}: gap in tiles between {} and {}",
                    w[0].1, w[1].0
                ));
            }
        }
    }
}

/// **P-I1(iii): Run = basis span union ⊇ tile range.**
///
/// `plateau.start` = min(R.lo), `plateau.end` = max(R.hi) over basis
/// elements. The run `[start, end)` must contain `[key, next_key)`.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i1_iii_run_contains_tile<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let plateaus = &graph.plateaus;
    let pb = graph.plateau_basis();
    let keys: Vec<BasisEdge<C>> = plateaus.keys().copied().collect();
    let domain_max = C::domain_max(N);

    for (i, &key) in keys.iter().enumerate() {
        let plateau = &plateaus[&key];
        let next_start = if i + 1 < keys.len() { keys[i + 1].0 } else { domain_max };

        let elements = pb.basis_elements(&key);
        if elements.is_empty() {
            continue; // Already caught by P-I1(ii).
        }

        // Compute run from basis spans.
        let mut min_lo = f64::INFINITY;
        let mut max_hi = f64::NEG_INFINITY;
        for &gid in elements {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            let lo = g.lo.to_f64();
            let hi = g.hi.to_f64();
            if lo < min_lo {
                min_lo = lo;
            }
            if hi > max_hi {
                max_hi = hi;
            }
        }

        // Verify plateau.start / plateau.end match basis span.
        let p_start = plateau.start.to_f64();
        let p_end = plateau.end.to_f64();
        if (p_start - min_lo).abs() > 1e-12 {
            errors.push(format!(
                "P-I1(iii): plateau {key:?}: start={p_start}, expected min(basis.lo)={min_lo}"
            ));
        }
        if (p_end - max_hi).abs() > 1e-12 {
            errors.push(format!(
                "P-I1(iii): plateau {key:?}: end={p_end}, expected max(basis.hi)={max_hi}"
            ));
        }

        // Verify run ⊇ tile range: [key, next_start).
        let tile_lo = key.0.to_f64();
        let tile_hi = next_start.to_f64();
        if p_start - tile_lo > 1e-12 {
            errors.push(format!(
                "P-I1(iii): plateau {key:?}: run start {p_start} > tile start {tile_lo}"
            ));
        }
        if tile_hi - p_end > 1e-12 {
            errors.push(format!("P-I1(iii): plateau {key:?}: run end {p_end} < tile end {tile_hi}"));
        }
    }
}

// ── P-I2: Minimal deterministic basis ──────────────────────────────

/// Each basis element must be maximally consolidated: if an element's
/// parent also has uniform contour depth equal to the plateau's depth,
/// the parent should be the basis element instead.
///
/// This is a soundness requirement, not merely a canonicalization: by
/// the non-minimal-basis lemma (§IDEA M-5.6.4, after P-I2), using
/// descendants instead of the maximal ancestor undercounts the plateau
/// sum because intermediate internal nodes' frozen `own` values are
/// not captured by their descendants' `sum` fields.
///
/// See §IDEA M-5.6.1 conditions 1–3 and P-I2.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i2_basis_minimality<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();
    for (&key, gnodes_list) in pb.iter() {
        let Some(plateau) = graph.plateaus.get(&key) else { continue };
        let expected_depth = plateau.depth;

        for &gid in gnodes_list {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            // Semi-internal basis elements are not subject to
            // ancestor-consolidation (their parent subtree always
            // contains heterogeneous contour depths).
            if g.state() == GState::SemiInternal {
                continue;
            }

            // Check: does R's parent also have uniform contour depth ==
            // expected_depth?  If so, R should have been consolidated
            // into the parent.
            if let Some(parent_id) = g.parent
                && let Some(parent_depth) = uniform_contour_depth_of(graph.gnodes(), parent_id, N)
                && parent_depth == expected_depth
            {
                errors.push(format!(
                    "P-I2 minimality: basis element G({}) in plateau {key:?} \
                             has parent G({}) with uniform contour depth {parent_depth} \
                             == plateau depth {expected_depth} — parent should be the \
                             basis element instead",
                    gid.index(),
                    parent_id.index(),
                ));
            }
        }
    }
}

// ── P-I3: Tile disjointness ─────────────────────────────────────────

/// Tiles of basis elements from different plateaus must not overlap.
///
/// Uses `tile(R)` — the contour-contributing interval — for all basis
/// elements, including semi-internals (whose tile is the uncovered
/// half). This is stronger than the previous check which excluded
/// semi-internals entirely.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i3_basis_disjointness<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    // Collect (tile_lo, tile_hi, key) for all basis elements.
    let pb = graph.plateau_basis();
    let mut tiles: Vec<(f64, f64, BasisEdge<C>)> = Vec::new();
    for (&key, gnodes) in pb.iter() {
        for &gid in gnodes {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            let (tlo, thi) = tile_of(g);
            tiles.push((tlo.to_f64(), thi.to_f64(), key));
        }
    }
    tiles.sort_by(|a, b| a.0.total_cmp(&b.0).then_with(|| a.1.total_cmp(&b.1)));

    for w in tiles.windows(2) {
        let (lo1, hi1, k1) = &w[0];
        let (lo2, _hi2, k2) = &w[1];
        if k1 != k2 && *hi1 > *lo2 + 1e-12 {
            errors.push(format!(
                "P-I3 tile disjointness: tile in plateau {k1:?} [{lo1}, {hi1}) \
                 overlaps tile in plateau {k2:?} [{lo2}, ..)"
            ));
        }
    }
}

// ── P-I4: Thatch one-hop ───────────────────────────────────────────

/// Every semi-internal basis element's surviving child must be covered
/// by a *different* plateau.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i4_thatch_one_hop<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();
    for (&key, gnodes) in pb.iter() {
        for &gid in gnodes {
            if !graph.gnodes().is_occupied(gid.index()) {
                continue;
            }
            let g = graph.gnodes().get(gid.index());
            if g.state() != GState::SemiInternal {
                continue;
            }

            // Find the surviving child.
            let child = g.left.or(g.right);
            let Some(child_id) = child else {
                errors.push(format!("P-I4: semi-internal {gid:?} in plateau {key:?} has no children"));
                continue;
            };

            // The child (or a descendant at child.lo) must be a basis
            // element of a different plateau.
            let child_g = graph.gnodes().get(child_id.index());
            let child_lo = child_g.lo;

            // Route through the BTreeMap to find which plateau covers
            // child.lo.
            let child_plateau_key = graph.plateaus.range(..=BasisEdge(child_lo)).next_back().map(|(&k, _)| k);

            match child_plateau_key {
                Some(ck) if ck == key => {
                    errors.push(format!(
                        "P-I4 thatch one-hop: semi-internal {gid:?} in plateau {key:?} \
                         thatches child {child_id:?}, but child's plateau key {ck:?} == parent's"
                    ));
                }
                None => {
                    errors.push(format!(
                        "P-I4 thatch one-hop: semi-internal {gid:?} in plateau {key:?}: \
                         no plateau found covering child {child_id:?} at lo={child_lo:?}"
                    ));
                }
                Some(_) => {} // OK — different plateau.
            }
        }
    }
}

// ── P-I5: Thatch depth bound ───────────────────────────────────────

/// At each basis element's `lo` coordinate, the number of plateaus
/// covering that point (thatch depth) must be ≤ `d_geo(x) + 1`.
///
/// `d_geo(x)` = depth of the terminal G-node reached by routing `x`.
#[cfg(feature = "dynamic-contour-tracking")]
fn check_p_i5_thatch_depth<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(
    graph: &GvGraph<C, V, N>,
    errors: &mut Vec<String>,
) {
    let pb = graph.plateau_basis();

    // Collect all sample coordinates: every basis element's lo.
    let mut samples: Vec<C> = Vec::new();
    for (_, gnodes) in pb.iter() {
        for &gid in gnodes {
            if graph.gnodes().is_occupied(gid.index()) {
                samples.push(graph.gnodes().get(gid.index()).lo);
            }
        }
    }
    samples.sort_by(Coordinate::total_cmp);
    samples.dedup_by(|a, b| a.total_cmp(b) == std::cmp::Ordering::Equal);

    for x in &samples {
        // Count thatch depth: how many plateaus have a basis element
        // whose [lo, hi) contains x.
        let mut thatch_count = 0u32;
        for (&_key, gnodes) in pb.iter() {
            let covers = gnodes.iter().any(|&gid| {
                if !graph.gnodes().is_occupied(gid.index()) {
                    return false;
                }
                let g = graph.gnodes().get(gid.index());
                g.lo.total_cmp(x) != std::cmp::Ordering::Greater && x.total_cmp(&g.hi) == std::cmp::Ordering::Less
            });
            if covers {
                thatch_count += 1;
            }
        }

        // Compute d_geo(x) by routing from root to the terminal.
        let d_geo = route_to_depth(graph, *x);

        if thatch_count > d_geo + 1 {
            errors.push(format!(
                "P-I5 thatch depth: at x={x:?}, thatch_depth={thatch_count} > d_geo={d_geo} + 1"
            ));
        }
    }
}

/// Route from G-root to the terminal containing `x`, returning its
/// G-Tree depth (number of edges from root).
#[cfg(feature = "dynamic-contour-tracking")]
fn route_to_depth<C: Coordinate, V: Accumulator + Inspectable, const N: u32>(graph: &GvGraph<C, V, N>, x: C) -> u32 {
    let mut cur = graph.g_root();
    for _ in 0..=N + 1 {
        let g = graph.gnodes().get(cur.index());
        if g.is_terminal() {
            return gnode_depth_from_interval(g.lo, g.hi, N);
        }
        let mid = C::midpoint(g.lo, g.hi);
        let next = if x.total_cmp(&mid) == std::cmp::Ordering::Less {
            g.left
        } else {
            g.right
        };
        match next {
            Some(child) => cur = child,
            None => return gnode_depth_from_interval(g.lo, g.hi, N),
        }
    }
    let g = graph.gnodes().get(cur.index());
    gnode_depth_from_interval(g.lo, g.hi, N)
}
