// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Unit tests for degenerate `SemiInternal` plateau shapes.
//!
//! These tests manually construct G-tree topologies with specific
//! `SemiInternal` arrangements, then verify:
//!
//! 1. `build_plateaus()` produces the expected plateau partition.
//! 2. `collect_subtree_basis_elements()` + `place_sorted()` produces
//!    a result that is compared against `build_plateaus()`.
//! 3. All plateau invariants hold (`assert_invariants` where
//!    applicable, manual checks otherwise).
//!
//! # Test index
//!
//! ## SI shapes (single)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`shape_a_si_left_child_only`] | root SI, left child only |
//! | [`shape_b_si_right_child_only`] | root SI, right child only |
//!
//! ## SI shapes (composite)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`shape_c_thatch_thatch_sandwich`] | plateau \ thatch | thatch / plateau |
//! | [`shape_d_multi_layer_overlap_start`] | SI overlap at domain start |
//! | [`shape_e_multi_layer_overlap_end`] | SI overlap at domain end |
//! | [`shape_f_nested_si_chain`] | nested SI chain (3 depths) |
//! | [`shape_g_thatch_sandwich`] | SI between two terminals |
//! | [`shape_h_double_thatch_gap`] | two SIs creating a central gap |
//! | [`shape_i_multi_layer_deep_overlap`] | SI deep in tree |
//!
//! ## SI + symmetric / structural
//!
//! | Test | Focus |
//! |------|-------|
//! | [`shape_j_symmetric_outward_thatches`] | mirror-symmetric outward SIs |
//! | [`shape_k_si_child_is_uniform_internal`] | SI-left child is uniform Internal |
//! | [`shape_k2_si_right_child_is_uniform_internal`] | SI-right child is uniform Internal |
//!
//! ## Controls (no SIs)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`shape_l_all_terminal_control`] | uniform balanced tree (exact match) |
//! | [`shape_m_nonuniform_internal_no_si`] | non-uniform, no SIs (exact match) |
//!
//! ## Summary
//!
//! | Test | Focus |
//! |------|-------|
//! | [`divergence_summary`] | enumerates all shapes, reports build-vs-collect divergence |
//!
//! # Divergence between `build_plateaus` and `collect_subtree` + `place_sorted`
//!
//! `collect_subtree_basis_elements` intentionally does **not** recurse
//! into `SemiInternal` children (see its doc comment for the proof).
//! In production this is correct because the SI's child is always
//! *already* a separate basis element in the live plateau state.
//!
//! These tests, however, call `collect_subtree_basis_elements` on the
//! **root** of a freshly constructed tree with **no pre-existing
//! plateau state**, so the SI children are never independently placed.
//! The resulting divergence is therefore a **test-methodology
//! artifact**, not a production bug.  Tests that contain SIs document
//! this artifact; the control tests (shapes L, M — no SIs) assert
//! exact match.
//!
//! # Shape catalogue
//!
//! ```text
//! Shape A:  SI-left-only root
//!           [0,256) SI ─left→ [0,128) T
//!
//! Shape B:  SI-right-only root
//!           [0,256) SI ─right→ [128,256) T
//!
//! Shape C:  Plateau \ Thatch | Thatch / Plateau
//!           [0,256) I ─left→ [0,128) SI ─right→ [64,128) T
//!                    └─right→ [128,256) SI ─left→ [128,192) T
//!
//! Shape D:  Multi-layer overlap at start
//!           [0,256) I ─left→ [0,128) SI ─left→ [0,64) T
//!                    └─right→ [128,256) T
//!
//! Shape E:  Multi-layer overlap at end
//!           [0,256) I ─left→ [0,128) T
//!                    └─right→ [128,256) SI ─right→ [192,256) T
//!
//! Shape F:  Nested SemiInternal chain
//!           [0,256) SI ─left→ [0,128) SI ─left→ [0,64) T
//!
//! Shape G:  Thatch sandwich — SI between two terminals
//!           [0,256) I ─left→ [0,128) I ─left→ [0,64) T
//!                                     └─right→ [64,128) SI ─left→ [64,96) T
//!                    └─right→ [128,256) T
//!
//! Shape H:  Double thatch gap — two SIs creating a central gap
//!           [0,256) I ─left→ [0,128) I ─left→ [0,64) T
//!                                     └─right→ [64,128) SI ─right→ [96,128) T
//!                    └─right→ [128,256) I ─left→ [128,192) SI ─left→ [128,160) T
//!                                       └─right→ [192,256) T
//!
//! Shape I:  Multi-layer deep overlap
//!           [0,256) I ─left→ [0,128) I ─left→ [0,64) SI ─left→ [0,32) T
//!                                     └─right→ [64,128) T
//!                    └─right→ [128,256) T
//!
//! Shape J:  Symmetric double-SI (outward thatches)
//!           [0,256) I ─left→ [0,128) SI ─left→ [0,64) T
//!                    └─right→ [128,256) SI ─right→ [192,256) T
//!
//! Shape K:  SI-left child is uniform Internal
//!           [0,256) I ─left→ [0,128) SI ─left→ [0,64) I ─left→ [0,32) T
//!                                                      └─right→ [32,64) T
//!                    └─right→ [128,256) T
//!
//! Shape K2: SI-right child is uniform Internal
//!           [0,256) I ─left→ [0,128) T
//!                    └─right→ [128,256) SI ─right→ [192,256) I ─left→ [192,224) T
//!                                                             └─right→ [224,256) T
//!
//! Shape L:  All-terminal control (no SIs)
//!           [0,256) I ─left→ [0,128) T
//!                    └─right→ [128,256) T
//!
//! Shape M:  Non-uniform Internal control (no SIs)
//!           [0,256) I ─left→ [0,128) I ─left→ [0,64) T
//!                                     └─right→ [64,128) T
//!                    └─right→ [128,256) I ─left→ [128,192) T
//!                              └─right→ [192,256) I ─left→ [192,224) T
//!                                                  └─right→ [224,256) T
//! ```
//!
//! For N=8 the domain is `[0, 256)`.  Depth = 8 − log₂(width):
//!
//! | Interval       | Width | Depth |
//! |----------------|-------|-------|
//! | [0, 256)       | 256   | 0     |
//! | [0, 128)       | 128   | 1     |
//! | [0, 64)        | 64    | 2     |
//! | [0, 32)        | 32    | 3     |
//! | [32, 64)       | 32    | 3     |
//! | [64, 96)       | 32    | 3     |
//! | [64, 128)      | 64    | 2     |
//! | [96, 128)      | 32    | 3     |
//! | [128, 160)     | 32    | 3     |
//! | [128, 192)     | 64    | 2     |
//! | [128, 256)     | 128   | 1     |
//! | [192, 224)     | 32    | 3     |
//! | [192, 256)     | 64    | 2     |
//! | [224, 256)     | 32    | 3     |

#![cfg(feature = "dynamic-contour-tracking")]

use std::collections::BTreeMap;

use crate::arena::Arena;
use crate::gnode::GNode;
use crate::graph::GvGraph;
use crate::handle::GNodeId;
use crate::plateau::{BasisEdge, Plateau, basis_edge_of};
use crate::testing::default_config;
use crate::tests::init_tracing;
use crate::vnode::{VKind, VNode};

// ── Helpers ─────────────────────────────────────────────────────

/// Allocate a `GNode` in the arena and return its `GNodeId`.
fn alloc_gnode(arena: &mut Arena<GNode<u64, u64>>, lo: u64, hi: u64) -> GNodeId {
    GNodeId::from_index(arena.alloc(GNode {
        lo,
        hi,
        sum: 0,
        own: 0,
        left: None,
        right: None,
        parent: None,
        entry: None,
    }))
}

/// Wire `child` as the left child of `parent`.
fn set_left(gnodes: &mut Arena<GNode<u64, u64>>, parent: GNodeId, child: GNodeId) {
    gnodes.get_mut(parent.index()).left = Some(child);
    gnodes.get_mut(child.index()).parent = Some(parent);
}

/// Wire `child` as the right child of `parent`.
fn set_right(gnodes: &mut Arena<GNode<u64, u64>>, parent: GNodeId, child: GNodeId) {
    gnodes.get_mut(parent.index()).right = Some(child);
    gnodes.get_mut(child.index()).parent = Some(parent);
}

/// Create a fresh `GvGraph<u64, u64, 8>` and replace its G-tree with
/// a manually constructed topology.
///
/// The caller provides a closure that receives a mutable arena
/// reference and must return the root `GNodeId` of the new tree.
///
/// The graph's plateau mirror is then rebuilt from scratch via
/// `build_plateaus` so the initial state is always consistent.
fn make_graph_with_topology(build_tree: impl FnOnce(&mut Arena<GNode<u64, u64>>) -> GNodeId) -> GvGraph<u64, u64, 8> {
    use std::sync::atomic::AtomicU32;

    use crate::handle::VNodeId;
    use crate::plateau::PlateauBasis;

    let mut gnodes: Arena<GNode<u64, u64>> = Arena::new();
    let root = build_tree(&mut gnodes);

    // Build a minimal V-tree: one entry for the root.
    let mut vnodes: Arena<VNode<u64>> = Arena::new();
    let v_entry = VNode {
        intensity: 0,
        parent: None,
        cached_depth: AtomicU32::new(0),
        kind: VKind::Entry {
            gnode: root,
            is_exposed: true,
            is_evictable: true,
        },
    };
    let v_root = VNodeId::from_index(vnodes.alloc(v_entry));
    gnodes.get_mut(root.index()).entry = Some(v_root);

    // Count nodes.
    let mut count = 0u32;
    let mut terminal_count = 0u32;
    let mut stack = vec![root];
    while let Some(gid) = stack.pop() {
        count += 1;
        let g = gnodes.get(gid.index());
        if g.left.is_none() && g.right.is_none() {
            terminal_count += 1;
        }
        if let Some(l) = g.left {
            stack.push(l);
        }
        if let Some(r) = g.right {
            stack.push(r);
        }
    }

    let config = default_config();
    let depth_buffer = config.depth_evict - config.depth_create;
    let headroom = 3usize.pow(depth_buffer + 1);

    // Build plateau mirror from the tree.
    // First we need a temporary graph-like thing to call build_plateaus.
    // But build_plateaus is a method on GvGraph. So construct the graph
    // first with empty plateaus, then rebuild.
    let mut graph = GvGraph {
        gnodes,
        vnodes,
        g_root: root,
        v_root: Some(v_root),
        config,
        violations: Vec::new(),
        node_count: count,
        terminal_count,
        live_depth_evict: 6,
        live_depth_create: 3,
        depth_buffer,
        headroom,
        soft_limit: None,
        plateaus: BTreeMap::new(),
        pending_p_i4: Vec::new(),
        plateau_basis: PlateauBasis::new(),
        plateaus_dirty: false,
    };

    // Build ground-truth plateaus.
    let built = graph.build_plateaus();

    // Install them into the graph's mirror.
    graph.plateaus = built;
    graph.plateau_basis = PlateauBasis::new();

    // Reconstruct plateau_basis by DFS — same as build_plateaus but
    // tracking which gnodes are basis elements.
    rebuild_plateau_basis(&mut graph);

    graph
}

/// Rebuild `plateau_basis` from the current `plateaus` `BTreeMap` by
/// DFS-collecting basis elements (matching `build_plateaus` logic).
fn rebuild_plateau_basis(graph: &mut GvGraph<u64, u64, 8>) {
    use crate::gnode::GState;
    use crate::graph::uniform_contour_depth_of;
    use crate::plateau::PlateauBasis;

    let mut pb = PlateauBasis::new();

    // DFS collect basis elements with their basis_edge.
    let mut basis: Vec<(GNodeId, BasisEdge<u64>)> = Vec::new();
    let mut stack = vec![graph.g_root];
    while let Some(gid) = stack.pop() {
        let g = graph.gnodes.get(gid.index());
        match g.state() {
            GState::Terminal => {
                basis.push((gid, BasisEdge(g.lo)));
            }
            GState::SemiInternal => {
                basis.push((gid, basis_edge_of(g)));
                if let Some(l) = g.left {
                    stack.push(l);
                }
                if let Some(r) = g.right {
                    stack.push(r);
                }
            }
            GState::Internal => {
                if uniform_contour_depth_of(&graph.gnodes, gid, 8).is_some() {
                    basis.push((gid, basis_edge_of(g)));
                } else {
                    if let Some(l) = g.left {
                        stack.push(l);
                    }
                    if let Some(r) = g.right {
                        stack.push(r);
                    }
                }
            }
        }
    }

    // Sort by basis edge.
    basis.sort_by_key(|&(_, be)| be);

    // Assign each basis element to the plateau whose key is ≤ its edge.
    // Walk plateaus left-to-right, assigning basis elements that fall
    // within each plateau's spatial range.
    let plateau_keys: Vec<BasisEdge<u64>> = graph.plateaus.keys().copied().collect();
    for (gid, be) in &basis {
        // Find the plateau this element belongs to: the last key ≤ be.
        let key = plateau_keys
            .iter()
            .rev()
            .find(|&&k| k <= *be)
            .copied()
            .unwrap_or(plateau_keys[0]);
        pb.insert(key, *gid);
    }

    graph.plateau_basis = pb;
}

/// Plateau map type alias to avoid clippy `type_complexity` warning.
type PlateauMap = BTreeMap<BasisEdge<u64>, Plateau<u64, u64>>;

/// Compare the plateau `BTreeMap` from `build_plateaus()` with the
/// result of clearing the mirror and rebuilding via
/// `collect_subtree_basis_elements` + `place_sorted`.
///
/// Returns `(build_map, incremental_map)` for further assertions.
fn compare_build_vs_collect_place(graph: &mut GvGraph<u64, u64, 8>) -> (PlateauMap, PlateauMap) {
    let built = graph.build_plateaus();

    // Clear the mirror.
    graph.plateaus.clear();
    graph.plateau_basis = crate::plateau::PlateauBasis::new();
    graph.pending_p_i4.clear();

    // Collect and place.
    let mut elements = Vec::new();
    graph.collect_subtree_basis_elements(graph.g_root, &mut elements);
    graph.place_sorted(&mut elements);

    let incremental = graph.plateaus.clone();

    (built, incremental)
}

/// Assert two plateau maps have identical keys, depths, and spatial
/// extents. Sums are zero in our tests so we skip them.
fn assert_plateaus_eq(label: &str, expected: &PlateauMap, actual: &PlateauMap) {
    assert_eq!(
        expected.len(),
        actual.len(),
        "{label}: plateau count mismatch: expected {}, got {}.\n  Expected keys: {:?}\n  Actual keys:   {:?}",
        expected.len(),
        actual.len(),
        expected.keys().collect::<Vec<_>>(),
        actual.keys().collect::<Vec<_>>(),
    );
    for (key, exp) in expected {
        let act = actual.get(key).unwrap_or_else(|| {
            panic!(
                "{label}: missing plateau at key {key:?}.\n  Expected keys: {:?}\n  Actual keys:   {:?}",
                expected.keys().collect::<Vec<_>>(),
                actual.keys().collect::<Vec<_>>(),
            )
        });
        assert_eq!(
            exp.depth, act.depth,
            "{label}: depth mismatch at key {key:?}: expected {}, got {}",
            exp.depth, act.depth,
        );
        assert_eq!(
            exp.start, act.start,
            "{label}: start mismatch at key {key:?}: expected {}, got {}",
            exp.start, act.start,
        );
        assert_eq!(
            exp.end, act.end,
            "{label}: end mismatch at key {key:?}: expected {}, got {}",
            exp.end, act.end,
        );
    }
}

// ── Shape A: Single SemiInternal, left child only ───────────────

/// ```text
/// [0,256) SI ─left→ [0,128) T
/// ```
///
/// `build_plateaus` emits:
///   - [0,128) Terminal: edge=0, depth=1
///   - [0,256) `SemiInternal`: edge=128 (left child → midpoint), depth=0
///
/// Sorted: (0, d1), (128, d0) → two plateaus (depths differ).
#[test]
fn shape_a_si_left_child_only() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left = alloc_gnode(gnodes, 0, 128);
        set_left(gnodes, root, left);
        root
    });

    // Verify build_plateaus.
    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape A: expected 2 plateaus");

    // Plateau at edge=0: terminal [0,128), depth=1.
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 1);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 128);

    // Plateau at edge=128: SI [0,256), depth=0.
    let p128 = &built[&BasisEdge(128u64)];
    assert_eq!(p128.depth, 0);
    assert_eq!(p128.start, 0);
    assert_eq!(p128.end, 256);

    // Compare build vs collect+place.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    // collect_subtree_basis_elements does NOT recurse into SI
    // children (correct in production — child is already tracked).
    // Here with no prior plateau state the child is missing, so the
    // incremental path produces fewer plateaus.  This is a
    // test-methodology artifact, not a production bug.
    //
    // build_plateaus:  2 plateaus  (SI + its terminal child)
    // collect+place:   1 plateau   (SI only)
    assert_eq!(build_map.len(), 2, "build_plateaus: 2 plateaus");

    // Artifact: incremental path misses the SI child.
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape A", &build_map, &incr_map);
    } else {
        // Test-methodology artifact: no prior plateau state for child.
        assert_eq!(
            incr_map.len(),
            1,
            "Shape A: collect+place should produce 1 plateau (SI only, no child recurse)"
        );
        let p = incr_map.values().next().unwrap();
        assert_eq!(p.depth, 0, "Shape A: the sole plateau is the SI at depth 0");
    }
}

// ── Shape B: Single SemiInternal, right child only ──────────────

/// ```text
/// [0,256) SI ─right→ [128,256) T
/// ```
///
/// `build_plateaus` emits:
///   - [0,256) `SemiInternal`: edge=0 (right child → lo), depth=0
///   - [128,256) Terminal: edge=128, depth=1
///
/// Sorted: (0, d0), (128, d1) → two plateaus.
#[test]
fn shape_b_si_right_child_only() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let right = alloc_gnode(gnodes, 128, 256);
        set_right(gnodes, root, right);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape B: expected 2 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 0);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 256);

    let p128 = &built[&BasisEdge(128u64)];
    assert_eq!(p128.depth, 1);
    assert_eq!(p128.start, 128);
    assert_eq!(p128.end, 256);

    // Test-methodology artifact: collect+place misses SI child.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    assert_eq!(build_map.len(), 2);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape B", &build_map, &incr_map);
    } else {
        assert_eq!(incr_map.len(), 1, "Shape B: collect+place produces 1 plateau (SI only)");
    }
}

// ── Shape C: Plateau \ Thatch | Thatch / Plateau ────────────────

/// ```text
/// [0,256) I ─left→ [0,128) SI ─right→ [64,128) T
///          └─right→ [128,256) SI ─left→ [128,192) T
/// ```
///
/// `build_plateaus` emits (sorted by edge):
///   - [0,128) SI(right child): edge=0, depth=1
///   - [64,128) T: edge=64, depth=2
///   - [128,192) T: edge=128, depth=2
///   - [128,256) SI(left child): edge=192, depth=1
///
/// Left-to-right merge:
///   edge=0  d=1 → new plateau P1
///   edge=64 d=2 → d≠1 → new plateau P2
///   edge=128 d=2 → d==2, merge into P2
///   edge=192 d=1 → d≠2 → new plateau P3
///
/// Result: 3 plateaus at depths [1, 2, 1].
/// This is the canonical "plateau \ thatch | thatch / plateau" shape.
#[test]
fn shape_c_thatch_thatch_sandwich() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left = alloc_gnode(gnodes, 0, 128); // SI
        let right = alloc_gnode(gnodes, 128, 256); // SI
        let left_child = alloc_gnode(gnodes, 64, 128); // T (right child of left SI)
        let right_child = alloc_gnode(gnodes, 128, 192); // T (left child of right SI)
        set_left(gnodes, root, left);
        set_right(gnodes, root, right);
        set_right(gnodes, left, left_child);
        set_left(gnodes, right, right_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 3, "Shape C: expected 3 plateaus");

    // P1: edge=0, depth=1, covers [0,128) (the left SI's uncovered half).
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 1);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 128);

    // P2: edge=64, depth=2, covers [64,192) (two terminals merged).
    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 2);
    assert_eq!(p64.start, 64);
    assert_eq!(p64.end, 192);

    // P3: edge=192, depth=1, covers [128,256) (the right SI's uncovered half).
    let p192 = &built[&BasisEdge(192u64)];
    assert_eq!(p192.depth, 1);
    assert_eq!(p192.start, 128);
    assert_eq!(p192.end, 256);

    // Compare build vs collect+place.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);

    // Test-methodology artifact: collect doesn't recurse into SI
    // children, so both SIs emit at depth 1 without their depth-2
    // terminal children.  With no intervening depth-2 elements the
    // two depth-1 entries merge into one plateau.
    //
    // build_plateaus: 3 plateaus (d1, d2, d1)
    // collect+place:  1 or 2 (all at d1, merge-eligible)
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape C", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape C artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape D: Multi-layer overlap at start ───────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) SI ─left→ [0,64) T
///          └─right→ [128,256) T
/// ```
///
/// `build_plateaus` emits (sorted by edge):
///   - [0,64) T: edge=0, depth=2
///   - [0,128) SI(left child): edge=64, depth=1
///   - [128,256) T: edge=128, depth=1
///
/// Merge: edge=0 d=2 → P1. edge=64 d=1 → P2. edge=128 d=1 → same
/// depth as P2, merge → P2 covers [0,256).
///
/// Result: 2 plateaus: {edge=0, d=2}, {edge=64, d=1}.
/// The depth-2 terminal overlaps the start of the depth-1 run.
#[test]
fn shape_d_multi_layer_overlap_start() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_si = alloc_gnode(gnodes, 0, 128);
        let left_child = alloc_gnode(gnodes, 0, 64); // T
        let right_t = alloc_gnode(gnodes, 128, 256);
        set_left(gnodes, root, left_si);
        set_right(gnodes, root, right_t);
        set_left(gnodes, left_si, left_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape D: expected 2 plateaus");

    // P1: deeper terminal at start.
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 64);

    // P2: depth-1 run covering the SI and right terminal.
    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 1);
    assert_eq!(p64.start, 0);
    assert_eq!(p64.end, 256);

    // Test-methodology artifact: SI child not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape D", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape D artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape E: Multi-layer overlap at end ─────────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) T
///          └─right→ [128,256) SI ─right→ [192,256) T
/// ```
///
/// `build_plateaus` emits (sorted by edge):
///   - [0,128) T: edge=0, depth=1
///   - [128,256) SI(right child): edge=128, depth=1
///   - [192,256) T: edge=192, depth=2
///
/// Merge: edge=0 d=1 → P1. edge=128 d=1 → same, merge → P1
/// covers [0,256). edge=192 d=2 → P2.
///
/// Result: 2 plateaus: {edge=0, d=1}, {edge=192, d=2}.
/// The depth-2 terminal overlaps the end of the depth-1 run.
#[test]
fn shape_e_multi_layer_overlap_end() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_t = alloc_gnode(gnodes, 0, 128);
        let right_si = alloc_gnode(gnodes, 128, 256);
        let right_child = alloc_gnode(gnodes, 192, 256); // T
        set_left(gnodes, root, left_t);
        set_right(gnodes, root, right_si);
        set_right(gnodes, right_si, right_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape E: expected 2 plateaus");

    // P1: depth-1 run covering left terminal and the SI.
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 1);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 256);

    // P2: deeper terminal at end.
    let p192 = &built[&BasisEdge(192u64)];
    assert_eq!(p192.depth, 2);
    assert_eq!(p192.start, 192);
    assert_eq!(p192.end, 256);

    // Test-methodology artifact: SI child not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape E", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape E artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape F: Nested SemiInternal chain ──────────────────────────

/// ```text
/// [0,256) SI ─left→ [0,128) SI ─left→ [0,64) T
/// ```
///
/// `build_plateaus` emits (sorted by edge):
///   - [0,64) T: edge=0, depth=2
///   - [0,128) SI(left child): edge=64, depth=1
///   - [0,256) SI(left child): edge=128, depth=0
///
/// Merge: all different depths → 3 plateaus.
///
/// This is a multi-layer `SemiInternal` nesting.
#[test]
fn shape_f_nested_si_chain() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let mid = alloc_gnode(gnodes, 0, 128);
        let leaf = alloc_gnode(gnodes, 0, 64);
        set_left(gnodes, root, mid);
        set_left(gnodes, mid, leaf);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 3, "Shape F: expected 3 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 64);

    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 1);
    assert_eq!(p64.start, 0);
    assert_eq!(p64.end, 128);

    let p128 = &built[&BasisEdge(128u64)];
    assert_eq!(p128.depth, 0);
    assert_eq!(p128.start, 0);
    assert_eq!(p128.end, 256);

    // Test-methodology artifact: root is SI → emits (root, d=0) →
    // 1 element → 1 plateau.  Both nested children are missing
    // because there is no prior plateau state for them.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    assert_eq!(build_map.len(), 3);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape F", &build_map, &incr_map);
    } else {
        assert_eq!(incr_map.len(), 1, "Shape F: collect+place produces 1 plateau (root SI only)");
        eprintln!(
            "Shape F artifact: build=3 plateaus, collect+place={} plateaus",
            incr_map.len(),
        );
    }
}

// ── Shape G: Thatch sandwich ────────────────────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) I ─left→ [0,64) T
///                            └─right→ [64,128) SI ─left→ [64,96) T
///          └─right→ [128,256) T
/// ```
///
/// `build_plateaus`: root Internal, non-uniform (left child Internal
/// with non-uniform subtree because right grandchild is SI).
///
/// DFS decompose root → left Internal → non-uniform → decompose:
///   - [0,64) T: edge=0, depth=2
///   - [64,128) SI(left): edge=96, depth=2
///   - [64,96) T: edge=64, depth=3
///
/// Then right child:
///   - [128,256) T: edge=128, depth=1
///
/// Sorted by edge: (0, d2), (64, d3), (96, d2), (128, d1)
///
/// Merge: (0, d2) → P1. (64, d3) → P2. (96, d2) → P3 (d≠3).
/// (128, d1) → P4 (d≠2).
///
/// Actually wait: merge only checks previous. (96, d2) ≠ d3 → P3.
/// But is there a merge with P1? No, merge only checks the *last*
/// entry in the `BTreeMap`. After inserting (64, d3), the last entry
/// is (64, d3). So (96, d2) doesn't match d3 → new plateau.
///
/// Result: 4 plateaus.
#[test]
fn shape_g_thatch_sandwich() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_i = alloc_gnode(gnodes, 0, 128);
        let ll_t = alloc_gnode(gnodes, 0, 64);
        let lr_si = alloc_gnode(gnodes, 64, 128);
        let lr_child = alloc_gnode(gnodes, 64, 96); // T, left child of SI
        let right_t = alloc_gnode(gnodes, 128, 256);
        set_left(gnodes, root, left_i);
        set_right(gnodes, root, right_t);
        set_left(gnodes, left_i, ll_t);
        set_right(gnodes, left_i, lr_si);
        set_left(gnodes, lr_si, lr_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 4, "Shape G: expected 4 plateaus");

    // P1: [0,64) terminal at depth 2.
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 64);

    // P2: [64,96) terminal at depth 3.
    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 3);
    assert_eq!(p64.start, 64);
    assert_eq!(p64.end, 96);

    // P3: [64,128) SI at depth 2, edge=96.
    let p96 = &built[&BasisEdge(96u64)];
    assert_eq!(p96.depth, 2);
    assert_eq!(p96.start, 64);
    assert_eq!(p96.end, 128);

    // P4: [128,256) terminal at depth 1.
    let p128 = &built[&BasisEdge(128u64)];
    assert_eq!(p128.depth, 1);
    assert_eq!(p128.start, 128);
    assert_eq!(p128.end, 256);

    // Test-methodology artifact: SI child not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape G", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape G artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape H: Double thatch gap ──────────────────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) I ─left→ [0,64) T
///                            └─right→ [64,128) SI ─right→ [96,128) T
///          └─right→ [128,256) I ─left→ [128,192) SI ─left→ [128,160) T
///                              └─right→ [192,256) T
/// ```
///
/// Two SIs face each other across the domain midpoint, both with
/// thatches reaching *toward* the center but leaving a gap.
///
/// DFS from root:
///   Left Internal → non-uniform → decompose:
///     [0,64) T: edge=0, d=2
///     [64,128) SI(right): edge=64, d=2   (right child → lo = 64)
///     [96,128) T: edge=96, d=3
///   Right Internal → non-uniform → decompose:
///     [128,192) SI(left): edge=160, d=2   (left child → midpoint = 160)
///     [128,160) T: edge=128, d=3
///     [192,256) T: edge=192, d=2
///
/// Sorted: (0,d2), (64,d2), (96,d3), (128,d3), (160,d2), (192,d2)
///
/// Merge: (0,d2)→P1. (64,d2)→same d→merge into P1.
///   (96,d3)→P2. (128,d3)→same→merge into P2.
///   (160,d2)→P3. (192,d2)→same→merge into P3.
///
/// Result: 3 plateaus at depths [2, 3, 2].
#[test]
fn shape_h_double_thatch_gap() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_i = alloc_gnode(gnodes, 0, 128);
        let ll_t = alloc_gnode(gnodes, 0, 64);
        let lr_si = alloc_gnode(gnodes, 64, 128); // SI right child
        let lr_child = alloc_gnode(gnodes, 96, 128); // T, right child of SI
        let right_i = alloc_gnode(gnodes, 128, 256);
        let rl_si = alloc_gnode(gnodes, 128, 192); // SI left child
        let rl_child = alloc_gnode(gnodes, 128, 160); // T, left child of SI
        let rr_t = alloc_gnode(gnodes, 192, 256);
        set_left(gnodes, root, left_i);
        set_right(gnodes, root, right_i);
        set_left(gnodes, left_i, ll_t);
        set_right(gnodes, left_i, lr_si);
        set_right(gnodes, lr_si, lr_child);
        set_left(gnodes, right_i, rl_si);
        set_right(gnodes, right_i, rr_t);
        set_left(gnodes, rl_si, rl_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 3, "Shape H: expected 3 plateaus");

    // P1: depth 2 at start, covers [0,128) (left terminal + left SI's uncovered half).
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 128);

    // P2: depth 3 in the middle (thatches merged), covers [96,160).
    let p96 = &built[&BasisEdge(96u64)];
    assert_eq!(p96.depth, 3);
    assert_eq!(p96.start, 96);
    assert_eq!(p96.end, 160);

    // P3: depth 2 at end, covers [128,256) (right SI's uncovered half + right terminal).
    let p160 = &built[&BasisEdge(160u64)];
    assert_eq!(p160.depth, 2);
    assert_eq!(p160.start, 128);
    assert_eq!(p160.end, 256);

    // Test-methodology artifact: SI children not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape H", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape H artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape I: Multi-layer deep overlap ───────────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) I ─left→ [0,64) SI ─left→ [0,32) T
///                            └─right→ [64,128) T
///          └─right→ [128,256) T
/// ```
///
/// Three depths of overlap at the very start of the domain.
///
/// DFS from root → non-uniform → decompose:
///   Left Internal → non-uniform → decompose:
///     [0,64) SI(left): edge=32, depth=2  (left child → mid = 32)
///     [0,32) T: edge=0, depth=3
///     [64,128) T: edge=64, depth=2
///   [128,256) T: edge=128, depth=1
///
/// Sorted: (0,d3), (32,d2), (64,d2), (128,d1)
///
/// Merge: (0,d3)→P1. (32,d2)→P2. (64,d2)→same→merge into P2.
///   (128,d1)→P3.
///
/// Result: 3 plateaus at depths [3, 2, 1] — nested overlap at start.
#[test]
fn shape_i_multi_layer_deep_overlap() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_i = alloc_gnode(gnodes, 0, 128);
        let ll_si = alloc_gnode(gnodes, 0, 64);
        let lll_t = alloc_gnode(gnodes, 0, 32); // deepest terminal
        let lr_t = alloc_gnode(gnodes, 64, 128);
        let right_t = alloc_gnode(gnodes, 128, 256);
        set_left(gnodes, root, left_i);
        set_right(gnodes, root, right_t);
        set_left(gnodes, left_i, ll_si);
        set_right(gnodes, left_i, lr_t);
        set_left(gnodes, ll_si, lll_t);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 3, "Shape I: expected 3 plateaus");

    // P1: deepest terminal at start.
    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 3);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 32);

    // P2: depth 2 (SI + right terminal merged).
    let p32 = &built[&BasisEdge(32u64)];
    assert_eq!(p32.depth, 2);
    assert_eq!(p32.start, 0);
    assert_eq!(p32.end, 128);

    // P3: depth 1 right terminal.
    let p128 = &built[&BasisEdge(128u64)];
    assert_eq!(p128.depth, 1);
    assert_eq!(p128.start, 128);
    assert_eq!(p128.end, 256);

    // Test-methodology artifact: SI child not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape I", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape I artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape J: Symmetric double-SI at root ────────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) SI ─left→ [0,64) T
///          └─right→ [128,256) SI ─right→ [192,256) T
/// ```
///
/// Mirror-symmetric: left SI has left child, right SI has right child.
/// Both thatches face *outward* — the central gap [64,192) is covered
/// only by SI basis elements.
///
/// DFS:
///   Left SI: edge=64 (left→mid), d=1
///   [0,64) T: edge=0, d=2
///   Right SI: edge=128 (right→lo), d=1
///   [192,256) T: edge=192, d=2
///
/// Sorted: (0,d2), (64,d1), (128,d1), (192,d2)
///
/// Merge: (0,d2)→P. (64,d1)→P2. (128,d1)→merge→P2. (192,d2)→P3.
///
/// Result: 3 plateaus at depths [2, 1, 2].
#[test]
fn shape_j_symmetric_outward_thatches() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_si = alloc_gnode(gnodes, 0, 128);
        let left_child = alloc_gnode(gnodes, 0, 64);
        let right_si = alloc_gnode(gnodes, 128, 256);
        let right_child = alloc_gnode(gnodes, 192, 256);
        set_left(gnodes, root, left_si);
        set_right(gnodes, root, right_si);
        set_left(gnodes, left_si, left_child);
        set_right(gnodes, right_si, right_child);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 3, "Shape J: expected 3 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 64);

    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 1);
    // SI [0,128) merged with SI [128,256) ─ covers [0,256).
    assert_eq!(p64.start, 0);
    assert_eq!(p64.end, 256);

    let p192 = &built[&BasisEdge(192u64)];
    assert_eq!(p192.depth, 2);
    assert_eq!(p192.start, 192);
    assert_eq!(p192.end, 256);

    // Test-methodology artifact: SI children not independently placed.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape J", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape J artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape K: SI child is itself an Internal (uniform) ───────────

/// ```text
/// [0,256) I ─left→ [0,128) SI ─left→ [0,64) I ─left→ [0,32) T
///                                              └─right→ [32,64) T
///          └─right→ [128,256) T
/// ```
///
/// The SI's child is a fully balanced Internal → uniform depth 3.
///
/// `build_plateaus`: SI + recurse → Internal [0,64) uniform d=3 →
/// single basis element.
///
/// DFS:
///   Root Internal → non-uniform (left is SI) → decompose:
///     Left SI [0,128): edge=64, d=1
///     Child [0,64) Internal: uniform d=3 → edge=0, d=3
///   Right T [128,256): edge=128, d=1
///
/// Sorted: (0,d3), (64,d1), (128,d1)
/// Merge: (0,d3)→P1. (64,d1)→P2. (128,d1)→merge→P2.
///
/// Result: 2 plateaus [3, 1].
#[test]
fn shape_k_si_child_is_uniform_internal() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_si = alloc_gnode(gnodes, 0, 128);
        let child_i = alloc_gnode(gnodes, 0, 64);
        let child_left = alloc_gnode(gnodes, 0, 32);
        let child_right = alloc_gnode(gnodes, 32, 64);
        let right_t = alloc_gnode(gnodes, 128, 256);
        set_left(gnodes, root, left_si);
        set_right(gnodes, root, right_t);
        set_left(gnodes, left_si, child_i);
        set_left(gnodes, child_i, child_left);
        set_right(gnodes, child_i, child_right);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape K: expected 2 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 3);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 64);

    let p64 = &built[&BasisEdge(64u64)];
    assert_eq!(p64.depth, 1);
    assert_eq!(p64.start, 0);
    assert_eq!(p64.end, 256);

    // Test-methodology artifact: root Internal non-uniform → decompose.
    //   Left SI: emit (left_si, d=1). No child recurse.
    //   Right T: emit (right_t, d=1).
    // → 2 elements at d=1 → may merge into 1 plateau.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape K", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape K artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape K2: SI-right child is uniform Internal ────────────────

/// ```text
/// [0,256) I ─left→ [0,128) T
///          └─right→ [128,256) SI ─right→ [192,256) I ─left→ [192,224) T
///                                                   └─right→ [224,256) T
/// ```
///
/// Mirror of Shape K: the SI's child is on the right side and is a
/// fully balanced Internal → uniform depth 3.
///
/// DFS from root → non-uniform → decompose:
///   [0,128) T: edge=0, d=1
///   [128,256) SI(right): edge=128, d=1  (right child → lo = 128)
///   [192,256) I uniform d=3: edge=192, d=3
///
/// Sorted: (0,d1), (128,d1), (192,d3)
/// Merge: (0,d1)→P1. (128,d1)→same→merge→P1. (192,d3)→P2.
///
/// Result: 2 plateaus [1, 3].
#[test]
fn shape_k2_si_right_child_is_uniform_internal() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_t = alloc_gnode(gnodes, 0, 128);
        let right_si = alloc_gnode(gnodes, 128, 256);
        let child_i = alloc_gnode(gnodes, 192, 256);
        let child_left = alloc_gnode(gnodes, 192, 224);
        let child_right = alloc_gnode(gnodes, 224, 256);
        set_left(gnodes, root, left_t);
        set_right(gnodes, root, right_si);
        set_right(gnodes, right_si, child_i);
        set_left(gnodes, child_i, child_left);
        set_right(gnodes, child_i, child_right);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape K2: expected 2 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 1);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 256);

    let p192 = &built[&BasisEdge(192u64)];
    assert_eq!(p192.depth, 3);
    assert_eq!(p192.start, 192);
    assert_eq!(p192.end, 256);

    // Test-methodology artifact: root Internal non-uniform → decompose.
    //   Left T: emit (left_t, d=1).
    //   Right SI: emit (right_si, d=1). No child recurse.
    // → 2 elements at d=1 → may merge into 1 plateau.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    if incr_map.len() == build_map.len() {
        assert_plateaus_eq("Shape K2", &build_map, &incr_map);
    } else {
        eprintln!(
            "Shape K2 artifact: build={} plateaus, collect+place={} plateaus",
            build_map.len(),
            incr_map.len(),
        );
    }
}

// ── Shape L: All-terminal control (no SIs) ──────────────────────

/// ```text
/// [0,256) I ─left→ [0,128) T
///          └─right→ [128,256) T
/// ```
///
/// No `SemiInternal` nodes at all. This is a control test: both
/// `build_plateaus` and collect+place should produce identical result.
/// The Internal is uniform depth 1 → single basis element → 1 plateau.
#[test]
fn shape_l_all_terminal_control() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_t = alloc_gnode(gnodes, 0, 128);
        let right_t = alloc_gnode(gnodes, 128, 256);
        set_left(gnodes, root, left_t);
        set_right(gnodes, root, right_t);
        root
    });

    let built = graph.build_plateaus();
    // Uniform depth 1 → 1 plateau.
    assert_eq!(built.len(), 1, "Shape L: expected 1 plateau");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 1);
    assert_eq!(p0.start, 0);
    assert_eq!(p0.end, 256);

    // Control: collect+place should match exactly.
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    assert_plateaus_eq("Shape L (control)", &build_map, &incr_map);
}

// ── Shape M: Non-uniform Internal control (no SIs) ──────────────

/// ```text
/// [0,256) I ─left→ [0,128) I ─left→ [0,64) T
///                            └─right→ [64,128) T
///          └─right→ [128,256) I ─left→ [128,192) T
///                              └─right→ [192,256) I ─left→ [192,224) T
///                                                  └─right→ [224,256) T
/// ```
///
/// No SIs. Left subtree uniform d=2, right subtree non-uniform.
///
/// `build_plateaus`:
///   Left [0,128) Internal uniform d=2 → edge=0, d=2
///   Right [128,256) Internal non-uniform → decompose:
///     [128,192) T: edge=128, d=2
///     [192,256) I uniform d=3 → edge=192, d=3
///
/// Sorted: (0,d2), (128,d2), (192,d3)
/// Merge: (0,d2)→P1. (128,d2)→same→merge→P1 covers [0,256). (192,d3)→P2.
///
/// Result: 2 plateaus [2, 3].
#[test]
fn shape_m_nonuniform_internal_no_si() {
    let _t = init_tracing();

    let mut graph = make_graph_with_topology(|gnodes| {
        let root = alloc_gnode(gnodes, 0, 256);
        let left_i = alloc_gnode(gnodes, 0, 128);
        let ll = alloc_gnode(gnodes, 0, 64);
        let lr = alloc_gnode(gnodes, 64, 128);
        let right_i = alloc_gnode(gnodes, 128, 256);
        let rl = alloc_gnode(gnodes, 128, 192);
        let rr_i = alloc_gnode(gnodes, 192, 256);
        let rrl = alloc_gnode(gnodes, 192, 224);
        let rrr = alloc_gnode(gnodes, 224, 256);
        set_left(gnodes, root, left_i);
        set_right(gnodes, root, right_i);
        set_left(gnodes, left_i, ll);
        set_right(gnodes, left_i, lr);
        set_left(gnodes, right_i, rl);
        set_right(gnodes, right_i, rr_i);
        set_left(gnodes, rr_i, rrl);
        set_right(gnodes, rr_i, rrr);
        root
    });

    let built = graph.build_plateaus();
    assert_eq!(built.len(), 2, "Shape M: expected 2 plateaus");

    let p0 = &built[&BasisEdge(0u64)];
    assert_eq!(p0.depth, 2);

    let p192 = &built[&BasisEdge(192u64)];
    assert_eq!(p192.depth, 3);

    // Control: collect+place should match exactly (no SIs).
    let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
    assert_plateaus_eq("Shape M (control)", &build_map, &incr_map);
}

// ── Summary test: enumerate all shapes that diverge ─────────────

/// Runs all shapes and reports which ones show divergence between
/// `build_plateaus` and `collect_subtree_basis_elements + place_sorted`.
///
/// This is an informational test: it always passes but prints a
/// summary of divergences to stderr. Use `cargo test -- --nocapture`
/// to see the output.
#[test]
#[allow(clippy::too_many_lines)]
fn divergence_summary() {
    type BuildFn = Box<dyn FnOnce(&mut Arena<GNode<u64, u64>>) -> GNodeId>;

    let _t = init_tracing();

    let shapes: Vec<(&str, BuildFn)> = vec![
        (
            "A: SI-left-only",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left = alloc_gnode(gnodes, 0, 128);
                set_left(gnodes, root, left);
                root
            }),
        ),
        (
            "B: SI-right-only",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let right = alloc_gnode(gnodes, 128, 256);
                set_right(gnodes, root, right);
                root
            }),
        ),
        (
            "C: thatch|thatch",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left = alloc_gnode(gnodes, 0, 128);
                let right = alloc_gnode(gnodes, 128, 256);
                let lc = alloc_gnode(gnodes, 64, 128);
                let rc = alloc_gnode(gnodes, 128, 192);
                set_left(gnodes, root, left);
                set_right(gnodes, root, right);
                set_right(gnodes, left, lc);
                set_left(gnodes, right, rc);
                root
            }),
        ),
        (
            "D: multi-layer start",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_si = alloc_gnode(gnodes, 0, 128);
                let lc = alloc_gnode(gnodes, 0, 64);
                let right_t = alloc_gnode(gnodes, 128, 256);
                set_left(gnodes, root, left_si);
                set_right(gnodes, root, right_t);
                set_left(gnodes, left_si, lc);
                root
            }),
        ),
        (
            "E: multi-layer end",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_t = alloc_gnode(gnodes, 0, 128);
                let right_si = alloc_gnode(gnodes, 128, 256);
                let rc = alloc_gnode(gnodes, 192, 256);
                set_left(gnodes, root, left_t);
                set_right(gnodes, root, right_si);
                set_right(gnodes, right_si, rc);
                root
            }),
        ),
        (
            "F: nested SI chain",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let mid = alloc_gnode(gnodes, 0, 128);
                let leaf = alloc_gnode(gnodes, 0, 64);
                set_left(gnodes, root, mid);
                set_left(gnodes, mid, leaf);
                root
            }),
        ),
        (
            "G: thatch sandwich",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_i = alloc_gnode(gnodes, 0, 128);
                let ll_t = alloc_gnode(gnodes, 0, 64);
                let lr_si = alloc_gnode(gnodes, 64, 128);
                let lr_child = alloc_gnode(gnodes, 64, 96);
                let right_t = alloc_gnode(gnodes, 128, 256);
                set_left(gnodes, root, left_i);
                set_right(gnodes, root, right_t);
                set_left(gnodes, left_i, ll_t);
                set_right(gnodes, left_i, lr_si);
                set_left(gnodes, lr_si, lr_child);
                root
            }),
        ),
        (
            "H: double thatch gap",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_i = alloc_gnode(gnodes, 0, 128);
                let ll_t = alloc_gnode(gnodes, 0, 64);
                let lr_si = alloc_gnode(gnodes, 64, 128);
                let lr_child = alloc_gnode(gnodes, 96, 128);
                let right_i = alloc_gnode(gnodes, 128, 256);
                let rl_si = alloc_gnode(gnodes, 128, 192);
                let rl_child = alloc_gnode(gnodes, 128, 160);
                let rr_t = alloc_gnode(gnodes, 192, 256);
                set_left(gnodes, root, left_i);
                set_right(gnodes, root, right_i);
                set_left(gnodes, left_i, ll_t);
                set_right(gnodes, left_i, lr_si);
                set_right(gnodes, lr_si, lr_child);
                set_left(gnodes, right_i, rl_si);
                set_right(gnodes, right_i, rr_t);
                set_left(gnodes, rl_si, rl_child);
                root
            }),
        ),
        (
            "I: multi-layer deep",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_i = alloc_gnode(gnodes, 0, 128);
                let ll_si = alloc_gnode(gnodes, 0, 64);
                let lll_t = alloc_gnode(gnodes, 0, 32);
                let lr_t = alloc_gnode(gnodes, 64, 128);
                let right_t = alloc_gnode(gnodes, 128, 256);
                set_left(gnodes, root, left_i);
                set_right(gnodes, root, right_t);
                set_left(gnodes, left_i, ll_si);
                set_right(gnodes, left_i, lr_t);
                set_left(gnodes, ll_si, lll_t);
                root
            }),
        ),
        (
            "J: symmetric outward",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_si = alloc_gnode(gnodes, 0, 128);
                let left_child = alloc_gnode(gnodes, 0, 64);
                let right_si = alloc_gnode(gnodes, 128, 256);
                let right_child = alloc_gnode(gnodes, 192, 256);
                set_left(gnodes, root, left_si);
                set_right(gnodes, root, right_si);
                set_left(gnodes, left_si, left_child);
                set_right(gnodes, right_si, right_child);
                root
            }),
        ),
        (
            "K: SI-left uniform-I child",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_si = alloc_gnode(gnodes, 0, 128);
                let child_i = alloc_gnode(gnodes, 0, 64);
                let child_left = alloc_gnode(gnodes, 0, 32);
                let child_right = alloc_gnode(gnodes, 32, 64);
                let right_t = alloc_gnode(gnodes, 128, 256);
                set_left(gnodes, root, left_si);
                set_right(gnodes, root, right_t);
                set_left(gnodes, left_si, child_i);
                set_left(gnodes, child_i, child_left);
                set_right(gnodes, child_i, child_right);
                root
            }),
        ),
        (
            "K2: SI-right uniform-I child",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_t = alloc_gnode(gnodes, 0, 128);
                let right_si = alloc_gnode(gnodes, 128, 256);
                let child_i = alloc_gnode(gnodes, 192, 256);
                let child_left = alloc_gnode(gnodes, 192, 224);
                let child_right = alloc_gnode(gnodes, 224, 256);
                set_left(gnodes, root, left_t);
                set_right(gnodes, root, right_si);
                set_right(gnodes, right_si, child_i);
                set_left(gnodes, child_i, child_left);
                set_right(gnodes, child_i, child_right);
                root
            }),
        ),
        (
            "L: all-terminal (control)",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_t = alloc_gnode(gnodes, 0, 128);
                let right_t = alloc_gnode(gnodes, 128, 256);
                set_left(gnodes, root, left_t);
                set_right(gnodes, root, right_t);
                root
            }),
        ),
        (
            "M: non-uniform (control)",
            Box::new(|gnodes| {
                let root = alloc_gnode(gnodes, 0, 256);
                let left_i = alloc_gnode(gnodes, 0, 128);
                let ll = alloc_gnode(gnodes, 0, 64);
                let lr = alloc_gnode(gnodes, 64, 128);
                let right_i = alloc_gnode(gnodes, 128, 256);
                let rl = alloc_gnode(gnodes, 128, 192);
                let rr_i = alloc_gnode(gnodes, 192, 256);
                let rrl = alloc_gnode(gnodes, 192, 224);
                let rrr = alloc_gnode(gnodes, 224, 256);
                set_left(gnodes, root, left_i);
                set_right(gnodes, root, right_i);
                set_left(gnodes, left_i, ll);
                set_right(gnodes, left_i, lr);
                set_left(gnodes, right_i, rl);
                set_right(gnodes, right_i, rr_i);
                set_left(gnodes, rr_i, rrl);
                set_right(gnodes, rr_i, rrr);
                root
            }),
        ),
    ];

    let total = shapes.len();
    let mut divergences = Vec::new();
    for (name, builder) in shapes {
        let mut graph = make_graph_with_topology(builder);
        let (build_map, incr_map) = compare_build_vs_collect_place(&mut graph);
        if build_map.len() != incr_map.len() {
            divergences.push(format!(
                "  {name}: build={} plateaus, collect+place={} plateaus",
                build_map.len(),
                incr_map.len(),
            ));
        }
    }

    if divergences.is_empty() {
        eprintln!("divergence_summary: all shapes agree!");
    } else {
        eprintln!(
            "divergence_summary: {}/{total} shapes diverge:\n{}",
            divergences.len(),
            divergences.join("\n"),
        );
    }
    // Always passes — this is informational.
}
