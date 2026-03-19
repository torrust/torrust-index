// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use std::cmp::Ordering;
use std::collections::BTreeMap;
use std::mem::size_of;

use crate::gnode::GNode;
use crate::handle::GNodeId;
use crate::plateau::basis_edge_of;
use crate::{BasisEdge, Coordinate, Plateau};

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
