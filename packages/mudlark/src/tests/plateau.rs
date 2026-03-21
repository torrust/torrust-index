// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Crate tests for **plateau types**, **ordering**, and the
//! **`basis_edge_of`** helper.
//!
//! A *plateau* is a contiguous region of the G-tree contour that
//! shares a single `BasisEdge` key.  Correct plateau computation
//! depends on `Coordinate::total_cmp` producing a strict total order
//! (including IEEE 754 edge cases like NaN and −0), on `BasisEdge`
//! faithfully forwarding that order so it can serve as a `BTreeMap`
//! key, and on `basis_edge_of` mapping each `GNode` to the right
//! contour edge depending on its terminal / semi-internal / internal
//! status.
//!
//! The `PlateauBasis` tests (feature-gated behind
//! `dynamic-contour-tracking`) exercise the bidirectional index that
//! maps `GNodeId ↔ BasisEdge` and verify forward/back consistency
//! after insertions, removals, and full rebuilds.
//!
//! The integration tests at the end confirm that `GvGraph::plateaus()`
//! produces a gap-free, overlap-free tiling of the full domain, that a
//! fresh graph yields exactly one plateau, and that observations refine
//! the contour into multiple plateaus.
//!
//! # Test index
//!
//! ## `Coordinate::total_cmp`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`total_cmp_integers`] | basic integer ordering (u8, u32, u64) |
//! | [`total_cmp_floats`] | f64 ordering including NaN-after-∞ and −0 < +0 |
//!
//! ## `BasisEdge` — ordering & traits
//!
//! | Test | Focus |
//! |------|-------|
//! | [`basis_edge_ord_eq`] | `Ord` / `Eq` for `BasisEdge<u64>` |
//! | [`basis_edge_copy`] | `Copy` semantics |
//! | [`basis_edge_u64_matches_native_ord`] | sort order matches native `u64` order |
//! | [`basis_edge_f64_nan_sorts_last`] | NaN sorts after +∞ via `total_cmp` |
//! | [`basis_edge_f64_neg_zero_vs_pos_zero`] | −0.0 < +0.0 under IEEE 754 total order |
//! | [`basis_edge_as_btreemap_key`] | usable as `BTreeMap` key (u64) |
//! | [`basis_edge_f64_as_btreemap_key`] | usable as `BTreeMap` key (f64 — the whole point) |
//! | [`basis_edge_debug_format`] | `Debug` output contains inner value |
//!
//! ## `Plateau` — struct, traits & methods
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateau_size`] | struct fits within a cache line (≤ 48 bytes) |
//! | [`plateau_copy_semantics`] | `Copy` round-trip preserves all fields |
//! | [`plateau_debug_clone_partialeq`] | `Debug`, `Clone`, `PartialEq` derive sanity |
//! | [`plateau_ne_when_fields_differ`] | `PartialEq` detects per-field differences |
//! | [`plateau_width_u64`] | `width()` for integer coordinates |
//! | [`plateau_width_f64`] | `width()` for f64 coordinates |
//! | [`plateau_to_span`] | `to_span()` maps fields correctly |
//! | [`plateau_cross_types`] | construction with mixed coordinate/accumulator types |
//!
//! ## Serde (feature-gated)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateau_serde_round_trip`] | JSON round-trip for `Plateau` |
//! | [`basis_edge_serde_round_trip`] | JSON round-trip for `BasisEdge` |
//!
//! ## `PlateauBasis` (feature-gated)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateau_basis_empty`] | fresh basis has zero counts |
//! | [`plateau_basis_insert_lookup`] | insert + forward/back lookup consistency |
//! | [`plateau_basis_remove`] | removal cleans up both maps; last-element cleanup |
//! | [`plateau_basis_remove_nonexistent`] | removing absent key returns `None` |
//! | [`plateau_basis_multiple_plateaus`] | multiple distinct plateau keys coexist |
//! | [`plateau_basis_forward_back_consistency`] | forward ↔ back invariant over 5 nodes / 3 keys |
//! | [`plateau_basis_iter_ordered`] | iteration follows `BTreeMap` (basis-edge) order |
//! | [`plateau_basis_missing_key_returns_empty`] | lookup of absent key yields empty slice |
//! | [`plateau_basis_rebuild_replaces_state`] | `rebuild()` atomically replaces all mappings |
//!
//! ## `basis_edge_of`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`basis_edge_of_terminal`] | terminal node → edge = `lo` |
//! | [`basis_edge_of_terminal_at_zero`] | terminal at domain origin |
//! | [`basis_edge_of_balanced_internal`] | internal (both children) → edge = `lo` |
//! | [`basis_edge_of_semi_internal_child_left`] | left-only child → edge = midpoint |
//! | [`basis_edge_of_semi_internal_child_right`] | right-only child → edge = `lo` |
//! | [`basis_edge_of_unit_terminal`] | smallest possible interval `[7, 8)` |
//!
//! ## Integration — `GvGraph::plateaus()`
//!
//! | Test | Focus |
//! |------|-------|
//! | [`fresh_graph_single_plateau`] | fresh graph → exactly one full-domain plateau |
//! | [`observations_refine_contour`] | concentrated observations split contour |
//! | [`plateaus_partition_full_domain`] | plateaus tile `[0, 2^N)` with no gaps or overlaps |

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::mem::size_of;

use crate::gnode::GNode;
use crate::handle::GNodeId;
use crate::plateau::basis_edge_of;
use crate::testing::{GraphCreator, default_config};
use crate::{BasisEdge, Coordinate, GvGraph, Plateau};

// ── Coordinate::total_cmp ───────────────────────────────────

#[test]
fn total_cmp_integers() {
    assert_eq!(0u64.total_cmp(&1u64), Ordering::Less);
    assert_eq!(5u32.total_cmp(&5u32), Ordering::Equal);
    assert_eq!(10u8.total_cmp(&3u8), Ordering::Greater);
}

#[test]
fn total_cmp_floats() {
    assert_eq!(Coordinate::total_cmp(&0.0f64, &1.0f64), Ordering::Less);
    assert_eq!(Coordinate::total_cmp(&5.0f64, &5.0f64), Ordering::Equal);
    // NaN sorts after +∞
    assert_eq!(Coordinate::total_cmp(&f64::NAN, &f64::INFINITY), Ordering::Greater);
    // -0.0 < +0.0 under total_cmp
    assert_eq!(Coordinate::total_cmp(&(-0.0f64), &0.0f64), Ordering::Less);
}

// ── BasisEdge ───────────────────────────────────────────────

#[test]
fn basis_edge_ord_eq() {
    let a = BasisEdge(4u64);
    let b = BasisEdge(8u64);
    let c = BasisEdge(4u64);
    assert!(a < b);
    assert_eq!(a, c);
    assert_ne!(a, b);
}

#[test]
fn basis_edge_copy() {
    let a = BasisEdge(42u64);
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn basis_edge_u64_matches_native_ord() {
    let mut edges: Vec<BasisEdge<u64>> = vec![BasisEdge(8), BasisEdge(0), BasisEdge(4), BasisEdge(12)];
    edges.sort();
    let coords: Vec<u64> = edges.iter().map(|e| e.0).collect();
    assert_eq!(coords, vec![0, 4, 8, 12]);
}

#[test]
fn basis_edge_f64_nan_sorts_last() {
    let mut edges: Vec<BasisEdge<f64>> = vec![BasisEdge(f64::NAN), BasisEdge(1.0), BasisEdge(f64::INFINITY), BasisEdge(0.0)];
    edges.sort();
    // 0.0, 1.0, +∞, NaN
    assert!((edges[0].0 - 0.0).abs() < f64::EPSILON);
    assert!((edges[1].0 - 1.0).abs() < f64::EPSILON);
    assert!(edges[2].0.is_infinite());
    assert!(edges[3].0.is_nan());
}

#[test]
fn basis_edge_f64_neg_zero_vs_pos_zero() {
    let neg = BasisEdge(-0.0f64);
    let pos = BasisEdge(0.0f64);
    // Under IEEE 754 totalOrder, -0.0 < +0.0
    assert!(neg < pos);
    assert_ne!(neg, pos);
}

#[test]
fn basis_edge_as_btreemap_key() {
    let mut map = BTreeMap::new();
    map.insert(BasisEdge(4u64), "a");
    map.insert(BasisEdge(0u64), "b");
    map.insert(BasisEdge(8u64), "c");
    let keys: Vec<u64> = map.keys().map(|k| k.0).collect();
    assert_eq!(keys, vec![0, 4, 8]);
}

#[test]
fn basis_edge_f64_as_btreemap_key() {
    // This is the entire point of BasisEdge: f64 doesn't impl Ord,
    // but BasisEdge<f64> does via Coordinate::total_cmp.
    let mut map = BTreeMap::new();
    map.insert(BasisEdge(4.0f64), "a");
    map.insert(BasisEdge(0.0f64), "b");
    assert_eq!(map.len(), 2);
    assert_eq!(map.keys().map(|k| k.0).collect::<Vec<_>>(), vec![0.0, 4.0]);
}

#[test]
fn basis_edge_debug_format() {
    let e = BasisEdge(42u64);
    let s = format!("{e:?}");
    assert!(s.contains("42"), "Debug should include inner value, got: {s}");
}

// ── Plateau ─────────────────────────────────────────────────

#[test]
fn plateau_size() {
    // basis_edge: 8, start: 8, end: 8, depth: 4, sum: 8,
    // + alignment padding → verify ≤ 48.
    let size = size_of::<Plateau<u64, u64>>();
    assert!(
        size <= 48,
        "Plateau<u64, u64> is {size} bytes, expected ≤ 48 \
             (well under cache line)"
    );
}

#[test]
fn plateau_copy_semantics() {
    let a = Plateau::<u64, u64> {
        basis_edge: BasisEdge(0),
        start: 0,
        end: 8,
        depth: 0,
        sum: 42,
    };
    let b = a; // Copy
    assert_eq!(a, b);
}

#[test]
fn plateau_debug_clone_partialeq() {
    let p = Plateau::<u64, u64> {
        basis_edge: BasisEdge(4),
        start: 4,
        end: 8,
        depth: 2,
        sum: 100,
    };
    let _debug_str = format!("{p:?}"); // Debug
    let q = p; // Copy (also Clone)
    assert_eq!(p, q); // PartialEq
}

#[test]
fn plateau_ne_when_fields_differ() {
    let base = Plateau::<u64, u64> {
        basis_edge: BasisEdge(0),
        start: 0,
        end: 8,
        depth: 1,
        sum: 50,
    };

    // Differ in sum.
    let different_sum = Plateau { sum: 99, ..base };
    assert_ne!(base, different_sum);

    // Differ in depth.
    let different_depth = Plateau { depth: 3, ..base };
    assert_ne!(base, different_depth);

    // Differ in end.
    let different_end = Plateau { end: 16, ..base };
    assert_ne!(base, different_end);

    // Differ in basis_edge.
    let different_edge = Plateau {
        basis_edge: BasisEdge(4),
        ..base
    };
    assert_ne!(base, different_edge);
}

#[test]
fn plateau_width_u64() {
    let p = Plateau::<u64, u64> {
        basis_edge: BasisEdge(4),
        start: 4,
        end: 12,
        depth: 1,
        sum: 50,
    };
    assert_eq!(p.width(), 8);
}

#[test]
fn plateau_width_f64() {
    let p = Plateau::<f64, f64> {
        basis_edge: BasisEdge(0.0),
        start: 0.0,
        end: 16.0,
        depth: 0,
        sum: 3.5,
    };
    assert!((p.width() - 16.0).abs() < f64::EPSILON);
}

#[test]
fn plateau_to_span() {
    let p = Plateau::<u64, u64> {
        basis_edge: BasisEdge(2),
        start: 2,
        end: 6,
        depth: 2,
        sum: 77,
    };
    let s = p.to_span();
    assert_eq!(s.start, 2);
    assert_eq!(s.end, 6);
    assert_eq!(s.intensity, 77); // sum → intensity
    assert_eq!(s.depth, 2);
}

#[test]
fn plateau_cross_types() {
    // u32 coordinate, u32 accumulator.
    let _ = Plateau::<u32, u32> {
        basis_edge: BasisEdge(0),
        start: 0,
        end: 16,
        depth: 0,
        sum: 100,
    };
    // f64 coordinate, f64 accumulator.
    let _ = Plateau::<f64, f64> {
        basis_edge: BasisEdge(0.0),
        start: 0.0,
        end: 8.0,
        depth: 1,
        sum: std::f64::consts::PI,
    };
    // u128 coordinate, u64 accumulator.
    let _ = Plateau::<u128, u64> {
        basis_edge: BasisEdge(0),
        start: 0,
        end: 1 << 64,
        depth: 0,
        sum: 42,
    };
}

// ── Serde (feature-gated) ───────────────────────────────────

#[cfg(feature = "serde")]
#[test]
fn plateau_serde_round_trip() {
    let p = Plateau::<u64, u64> {
        basis_edge: BasisEdge(0),
        start: 0,
        end: 16,
        depth: 0,
        sum: 999,
    };
    let json = serde_json::to_string(&p).unwrap();
    let q: Plateau<u64, u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(p, q);
}

#[cfg(feature = "serde")]
#[test]
fn basis_edge_serde_round_trip() {
    let e = BasisEdge(42u64);
    let json = serde_json::to_string(&e).unwrap();
    let f: BasisEdge<u64> = serde_json::from_str(&json).unwrap();
    assert_eq!(e, f);
}

// ── PlateauBasis ────────────────────────────────────────────

#[cfg(feature = "dynamic-contour-tracking")]
mod plateau_basis_tests {
    use super::*;
    use crate::plateau::PlateauBasis;

    #[test]
    fn plateau_basis_empty() {
        let pb = PlateauBasis::<u64>::new();
        assert_eq!(pb.plateau_count(), 0);
        assert_eq!(pb.basis_count(), 0);
    }

    #[test]
    fn plateau_basis_insert_lookup() {
        let mut pb = PlateauBasis::<u64>::new();
        let g0 = GNodeId::from_index(0);
        let g1 = GNodeId::from_index(1);

        pb.insert(BasisEdge(0), g0);
        pb.insert(BasisEdge(0), g1);

        assert!(pb.contains(g0));
        assert!(pb.contains(g1));
        assert_eq!(pb.plateau_key(g0), Some(BasisEdge(0)));
        assert_eq!(pb.plateau_key(g1), Some(BasisEdge(0)));
        assert_eq!(pb.basis_elements(&BasisEdge(0)).len(), 2);
        assert_eq!(pb.plateau_count(), 1);
        assert_eq!(pb.basis_count(), 2);
    }

    #[test]
    fn plateau_basis_remove() {
        let mut pb = PlateauBasis::<u64>::new();
        let g0 = GNodeId::from_index(0);
        let g1 = GNodeId::from_index(1);

        pb.insert(BasisEdge(0), g0);
        pb.insert(BasisEdge(0), g1);

        // Remove g0.
        let key = pb.remove(g0);
        assert_eq!(key, Some(BasisEdge(0)));
        assert!(!pb.contains(g0));
        assert!(pb.contains(g1));
        assert_eq!(pb.basis_elements(&BasisEdge(0)).len(), 1);
        assert!(pb.basis_elements(&BasisEdge(0)).contains(&g1));

        // Remove g1 — last element, forward entry should be cleaned up.
        let key = pb.remove(g1);
        assert_eq!(key, Some(BasisEdge(0)));
        assert_eq!(pb.plateau_count(), 0);
        assert_eq!(pb.basis_count(), 0);
    }

    #[test]
    fn plateau_basis_remove_nonexistent() {
        let mut pb = PlateauBasis::<u64>::new();
        let g42 = GNodeId::from_index(42);
        assert_eq!(pb.remove(g42), None);
    }

    #[test]
    fn plateau_basis_multiple_plateaus() {
        let mut pb = PlateauBasis::<u64>::new();
        let g0 = GNodeId::from_index(0);
        let g1 = GNodeId::from_index(1);
        let g2 = GNodeId::from_index(2);

        pb.insert(BasisEdge(0), g0); // plateau at key 0
        pb.insert(BasisEdge(4), g1); // plateau at key 4
        pb.insert(BasisEdge(4), g2); // another basis element for key 4

        assert_eq!(pb.plateau_count(), 2);
        assert_eq!(pb.basis_count(), 3);
        assert_eq!(pb.plateau_key(g0), Some(BasisEdge(0)));
        assert_eq!(pb.plateau_key(g1), Some(BasisEdge(4)));
        assert_eq!(pb.plateau_key(g2), Some(BasisEdge(4)));
        assert_eq!(pb.basis_elements(&BasisEdge(0)).len(), 1);
        assert_eq!(pb.basis_elements(&BasisEdge(4)).len(), 2);
    }

    #[test]
    fn plateau_basis_forward_back_consistency() {
        let mut pb = PlateauBasis::<u64>::new();
        let gnodes: Vec<GNodeId> = (0..5).map(GNodeId::from_index).collect();

        pb.insert(BasisEdge(0), gnodes[0]);
        pb.insert(BasisEdge(0), gnodes[1]);
        pb.insert(BasisEdge(8), gnodes[2]);
        pb.insert(BasisEdge(8), gnodes[3]);
        pb.insert(BasisEdge(12), gnodes[4]);

        // Every back-pointer references a valid forward entry containing it.
        for (&gnode, &key) in pb.back_map() {
            let basis = pb.basis_elements(&key);
            assert!(
                basis.contains(&gnode),
                "back-pointer for {gnode:?} → key {key:?} but \
                 forward list does not contain it"
            );
        }

        // Every forward entry's elements have a matching back-pointer.
        for (key, elements) in pb.iter() {
            for &gnode in elements {
                assert_eq!(
                    pb.plateau_key(gnode),
                    Some(*key),
                    "forward entry at key {key:?} contains {gnode:?} \
                     but back-pointer says {:?}",
                    pb.plateau_key(gnode)
                );
            }
        }
    }

    #[test]
    fn plateau_basis_iter_ordered() {
        let mut pb = PlateauBasis::<u64>::new();
        pb.insert(BasisEdge(8), GNodeId::from_index(0));
        pb.insert(BasisEdge(0), GNodeId::from_index(1));
        pb.insert(BasisEdge(4), GNodeId::from_index(2));

        let keys: Vec<u64> = pb.iter().map(|(k, _)| k.0).collect();
        assert_eq!(keys, vec![0, 4, 8]); // BTreeMap order via BasisEdge
    }

    #[test]
    fn plateau_basis_missing_key_returns_empty() {
        let pb = PlateauBasis::<u64>::new();
        assert!(pb.basis_elements(&BasisEdge(42)).is_empty());
    }

    #[test]
    fn plateau_basis_rebuild_replaces_state() {
        let mut pb = PlateauBasis::<u64>::new();
        let g0 = GNodeId::from_index(0);
        let g1 = GNodeId::from_index(1);
        let g2 = GNodeId::from_index(2);

        // Populate with initial state.
        pb.insert(BasisEdge(0), g0);
        pb.insert(BasisEdge(0), g1);
        assert_eq!(pb.plateau_count(), 1);
        assert_eq!(pb.basis_count(), 2);

        // Rebuild with entirely different assignments.
        pb.rebuild(vec![(BasisEdge(10), g1), (BasisEdge(10), g2), (BasisEdge(20), g0)]);

        // Old state is gone.
        assert_eq!(pb.plateau_count(), 2);
        assert_eq!(pb.basis_count(), 3);
        assert_eq!(pb.plateau_key(g0), Some(BasisEdge(20)));
        assert_eq!(pb.plateau_key(g1), Some(BasisEdge(10)));
        assert_eq!(pb.plateau_key(g2), Some(BasisEdge(10)));
        assert!(pb.basis_elements(&BasisEdge(0)).is_empty());
        assert_eq!(pb.basis_elements(&BasisEdge(10)).len(), 2);
        assert_eq!(pb.basis_elements(&BasisEdge(20)).len(), 1);
    }
} // mod plateau_basis_tests

// ── basis_edge_of ───────────────────────────────────────────

/// Helper: create a terminal `GNode` [lo, hi).
fn make_terminal_gnode(lo: u64, hi: u64) -> GNode<u64, u64> {
    GNode {
        lo,
        hi,
        sum: 0,
        own: 0,
        left: None,
        right: None,
        parent: None,
        entry: None,
    }
}

/// Helper: create an internal `GNode` [lo, hi) with both children.
fn make_internal_gnode(lo: u64, hi: u64) -> GNode<u64, u64> {
    GNode {
        lo,
        hi,
        sum: 0,
        own: 0,
        left: Some(GNodeId::from_index(100)),
        right: Some(GNodeId::from_index(101)),
        parent: None,
        entry: None,
    }
}

/// Helper: create a semi-internal `GNode` with child on the given side.
fn make_semi_internal_gnode(lo: u64, hi: u64, child_on_left: bool) -> GNode<u64, u64> {
    GNode {
        lo,
        hi,
        sum: 0,
        own: 0,
        left: if child_on_left { Some(GNodeId::from_index(100)) } else { None },
        right: if child_on_left { None } else { Some(GNodeId::from_index(101)) },
        parent: None,
        entry: None,
    }
}

#[test]
fn basis_edge_of_terminal() {
    // Terminal [4, 8): edge = lo = 4
    let g = make_terminal_gnode(4, 8);
    assert_eq!(basis_edge_of(&g), BasisEdge(4));
}

#[test]
fn basis_edge_of_terminal_at_zero() {
    // Terminal [0, 8): edge = lo = 0
    let g = make_terminal_gnode(0, 8);
    assert_eq!(basis_edge_of(&g), BasisEdge(0));
}

#[test]
fn basis_edge_of_balanced_internal() {
    // Internal [0, 8) with both children: edge = lo = 0
    let g = make_internal_gnode(0, 8);
    assert_eq!(basis_edge_of(&g), BasisEdge(0));
}

#[test]
fn basis_edge_of_semi_internal_child_left() {
    // Semi-internal [0, 8) with left child [0, 4):
    // Uncovered half is [4, 8) → edge = midpoint = 4
    let g = make_semi_internal_gnode(0, 8, true);
    assert_eq!(basis_edge_of(&g), BasisEdge(4));
}

#[test]
fn basis_edge_of_semi_internal_child_right() {
    // Semi-internal [0, 8) with right child [4, 8):
    // Uncovered half is [0, 4) → edge = lo = 0
    let g = make_semi_internal_gnode(0, 8, false);
    assert_eq!(basis_edge_of(&g), BasisEdge(0));
}

#[test]
fn basis_edge_of_unit_terminal() {
    // Smallest possible terminal [7, 8): edge = 7
    let g = make_terminal_gnode(7, 8);
    assert_eq!(basis_edge_of(&g), BasisEdge(7));
}

// ── Integration — GvGraph::plateaus() ───────────────────────

#[test]
fn fresh_graph_single_plateau() {
    let g: GvGraph<u64, u64, 8> = GraphCreator::new(default_config()).build();
    let p = g.plateaus();
    assert_eq!(p.len(), 1, "fresh graph should have exactly one plateau");

    let only = p.values().next().unwrap();
    assert_eq!(only.start, 0);
    assert_eq!(only.end, 256); // 2^8
    assert_eq!(only.depth, 0);
    assert_eq!(only.sum, 0);
    assert_eq!(only.width(), 256);
}

#[test]
fn observations_refine_contour() {
    let g: GvGraph<u64, u64, 8> = GraphCreator::new(default_config()).hotspot(42, 10, 5).build();

    let p = g.plateaus();
    assert!(
        p.len() >= 2,
        "concentrated observations should refine contour, got {} plateaus",
        p.len()
    );

    // Every plateau's basis_edge matches its map key.
    for (&key, plateau) in p.iter() {
        assert_eq!(key, plateau.basis_edge, "key-consistency invariant");
    }
}

#[test]
fn plateaus_partition_full_domain() {
    // Build a graph with spread observations so multiple plateaus exist.
    let g: GvGraph<u64, u64, 8> = GraphCreator::new(default_config()).spread(256, 10, 20).build();

    let p = g.plateaus();

    // Verify the plateaus tile [0, 256) with no gaps or overlaps.
    let mut expected_start = 0u64;
    for plateau in p.values() {
        assert_eq!(
            plateau.start, expected_start,
            "gap or overlap: expected start {expected_start}, got {}",
            plateau.start
        );
        assert!(
            plateau.start < plateau.end,
            "empty plateau: [{}, {})",
            plateau.start,
            plateau.end
        );
        expected_start = plateau.end;
    }
    assert_eq!(expected_start, 256, "plateaus should cover up to 2^8 = 256");
}
