// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![cfg(feature = "dynamic-contour-tracking")]
//! Integration tests for **contour range queries** (§ADR M-037 revised,
//! Phase 4).
//!
//! A contour range decomposes an arbitrary `[start, end)` interval of
//! the plateau lattice into a flat basis of G-node contributions.  The
//! decomposition must satisfy a number of algebraic and structural
//! invariants — energy additivity, boundary-thatch limits, tiling
//! contiguity, and agreement between the cheap energy-only path and the
//! full basis decomposition.  These tests exercise every invariant
//! across all pairwise lattice endpoint combinations, edge-case inputs
//! (non-lattice / stale / reversed endpoints), the `select_plateaus`
//! snap-outward helper, and a separate tree topology to guard against
//! shape-coupling.
//!
//! # Test index
//!
//! ## Basic range properties
//!
//! | Test | Focus |
//! |------|-------|
//! | [`single_plateau_range`]          | basis within `[start, end)`, thatch count ≤ 2 |
//! | [`full_domain_range`]             | single basis element (G-root), `energy == exact_energy` |
//! | [`two_plateau_concatenation`]     | concatenated `exact_energy` equals `range_sum` |
//! | [`split_subadditivity`]           | sub-range `exact_energy ≤ full.exact_energy` |
//! | [`nesting_monotonicity`]          | wider range has ≥ `exact_energy` of any inner range |
//! | [`boundary_count_at_most_two`]    | at most 2 `is_boundary_thatch` elements (CR-I8) |
//!
//! ## Edge cases and error paths
//!
//! | Test | Focus |
//! |------|-------|
//! | [`invalid_endpoint_returns_none`] | non-lattice endpoints produce `None` |
//! | [`stale_endpoint_returns_none`]   | mutation invalidates previous endpoints |
//! | [`empty_range_rejected`]          | `start >= end` returns `None` |
//!
//! ## Energy field invariants
//!
//! | Test | Focus |
//! |------|-------|
//! | [`energy_ordering`]                  | `plateau_energy <= energy`, structural identity |
//! | [`energy_equals_basis_sum`]          | `energy == Σ basis.sum` (§CR.6) |
//! | [`cross_plateau_energy_non_negative`] | `cross_plateau_energy <= energy` (no underflow) |
//! | [`energy_only_matches_full`]         | `contour_range_energy()` matches full decomposition |
//! | [`energy_exact_gap`]                 | full-domain: `energy == exact_energy`; general: `exact_energy == range_sum` |
//!
//! ## Determinism and structural invariants
//!
//! | Test | Focus |
//! |------|-------|
//! | [`determinism`]              | two calls on unmutated graph produce identical result |
//! | [`basis_is_flat_list`]       | basis sorted, contiguous, `start < end`, boundary-aligned |
//! | [`boundary_thatch_flag`]     | ≤ 2 thatching elements, positioned at first/last |
//! | [`no_straddling_ancestors`]  | all contributions are whole `.sum` values |
//!
//! ## `select_plateaus` helper
//!
//! | Test | Focus |
//! |------|-------|
//! | [`select_plateaus_basic`]          | arbitrary coords snap outward to lattice endpoints |
//! | [`select_plateaus_aligned`]        | lattice-aligned input returns unchanged endpoints |
//! | [`select_plateaus_single_plateau`] | both coords in same plateau → span that plateau |
//! | [`select_plateaus_full_domain`]    | `select_plateaus(0, 2^N)` → full domain range |
//! | [`select_plateaus_compose`]        | `select_plateaus → contour_range` round-trip succeeds |
//!
//! ## Additional coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`empty_tree_returns_none`]               | fresh graph: both APIs return zero-energy / `None` |
//! | [`select_plateaus_reversed_returns_none`]  | `lo >= hi` returns `None` |
//! | [`plateau_count_matches_lattice`]          | `plateau_count` equals the number of spanned plateaus |
//! | [`diverse_topology_invariants`]            | invariants hold under a different tree shape |

mod support;
use support::init_tracing;
use torrust_mudlark::invariants::assert_invariants;
use torrust_mudlark::testing::{Plan, contour_range_config, default_config, plan_multi_plateau, run_checked};
use torrust_mudlark::{BasisEdge, GvGraph};

// ── Helpers ─────────────────────────────────────────────────────

/// Build a tree with multiple plateaus for contour range testing.
///
/// Uses N=8 (domain `[0, 256)`), low split threshold to force
/// multiple depth levels and hence multiple plateaus.
fn build_multi_plateau_tree() -> GvGraph<u64, u64, 8> {
    let g = run_checked::<u64, u64, 8>(contour_range_config(), &plan_multi_plateau(), 10);
    assert_invariants(&g);
    g
}

/// Collect all valid endpoint-lattice points (plateau basis edges
/// plus the domain sentinel `2^N`).
fn lattice_endpoints(g: &GvGraph<u64, u64, 8>) -> Vec<BasisEdge<u64>> {
    let plateaus = g.plateaus();
    let mut eps: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    eps.push(BasisEdge(256u64)); // domain sentinel 2^8
    eps
}

// ════════════════════════════════════════════════════════════════════
// Basic range properties (tests 1–6)
// ════════════════════════════════════════════════════════════════════

// ── Test 1: single_plateau_range ────────────────────────────────

/// Range spanning exactly one plateau: basis elements all within
/// `[start, end)`, `is_boundary_thatch` count ≤ 2.
#[test]
fn single_plateau_range() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    // Each pair of adjacent endpoints spans one plateau.
    for pair in endpoints.windows(2) {
        let start = pair[0];
        let end = pair[1];
        let cr = g
            .contour_range(start, end)
            .expect("adjacent lattice endpoints should be valid");

        // At most 2 boundary thatching elements (general invariant CR-I8).
        let thatch_count = cr.basis.iter().filter(|b| b.is_boundary_thatch).count();
        assert!(
            thatch_count <= 2,
            "single plateau range [{}, {}): boundary thatch count {} > 2",
            start.0,
            end.0,
            thatch_count,
        );

        // All basis elements should have tiles within [start, end).
        for b in &cr.basis {
            assert!(
                b.start >= start.0 && b.end <= end.0,
                "basis element [{}, {}) not within [{}, {})",
                b.start,
                b.end,
                start.0,
                end.0,
            );
        }
    }
}

// ── Test 2: full_domain_range ───────────────────────────────────

/// `[first_plateau, 2^N)`: single basis element (G-root),
/// `energy == exact_energy == total_sum()`.
#[test]
fn full_domain_range() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let plateaus = g.plateaus();

    let start = *plateaus.keys().next().unwrap();
    let end = BasisEdge(256u64);

    let cr = g.contour_range(start, end).expect("full domain range is valid");

    // §CR.8.2: single basis element (G-root).
    assert_eq!(cr.basis.len(), 1, "expected exactly one basis element (G-root)");
    assert_eq!(cr.basis[0].gnode_id, g.g_root());
    assert!(!cr.basis[0].is_boundary_thatch);

    // For full domain: energy == exact_energy == total_sum().
    assert_eq!(cr.energy, g.total_sum());
    assert_eq!(cr.exact_energy, g.total_sum());
    assert_eq!(cr.energy, cr.exact_energy);
}

// ── Test 3: two_plateau_concatenation ───────────────────────────

/// Energy consistency across concatenated ranges: the combined range's
/// `exact_energy` should equal `range_sum(a..c)`.
#[test]
fn two_plateau_concatenation() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    // Need at least 3 endpoints to form two adjacent ranges.
    if endpoints.len() < 3 {
        return;
    }

    for triple in endpoints.windows(3) {
        let a = triple[0];
        let b = triple[1];
        let c = triple[2];

        let left = g.contour_range(a, b).unwrap();
        let right = g.contour_range(b, c).unwrap();
        let combined = g.contour_range(a, c).unwrap();

        // The concatenated range's exact_energy should equal `range_sum(a..c)`.
        let expected_energy = g.range_sum(a.0..c.0);
        assert_eq!(
            combined.exact_energy, expected_energy,
            "concatenated range [{}, {}) exact_energy mismatch",
            a.0, c.0,
        );

        // Sub-range exact energies should not exceed the combined range.
        assert!(
            left.exact_energy <= combined.exact_energy,
            "[{}, {}) exact_energy {} > [{}, {}) exact_energy {}",
            a.0,
            b.0,
            left.exact_energy,
            a.0,
            c.0,
            combined.exact_energy,
        );
        assert!(
            right.exact_energy <= combined.exact_energy,
            "[{}, {}) exact_energy {} > [{}, {}) exact_energy {}",
            b.0,
            c.0,
            right.exact_energy,
            a.0,
            c.0,
            combined.exact_energy,
        );
    }
}

// ── Test 4: split_subadditivity ─────────────────────────────────

/// Split at interior lattice point: each sub-range `exact_energy`
/// should not exceed the full range `exact_energy`.
#[test]
fn split_subadditivity() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 2)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let full = g.contour_range(start, end).unwrap();

            // Split at every interior lattice point.
            for mid in endpoints.iter().take(j).skip(i + 1).copied() {
                let left = g.contour_range(start, mid).unwrap();
                let right = g.contour_range(mid, end).unwrap();

                assert!(
                    left.exact_energy <= full.exact_energy,
                    "left [{}, {}) exact_energy {} > full [{}, {}) exact_energy {}",
                    start.0,
                    mid.0,
                    left.exact_energy,
                    start.0,
                    end.0,
                    full.exact_energy,
                );
                assert!(
                    right.exact_energy <= full.exact_energy,
                    "right [{}, {}) exact_energy {} > full [{}, {}) exact_energy {}",
                    mid.0,
                    end.0,
                    right.exact_energy,
                    start.0,
                    end.0,
                    full.exact_energy,
                );
            }
        }
    }
}

// ── Test 5: nesting_monotonicity ────────────────────────────────

/// Wider range has ≥ `exact_energy` of any inner range.
#[test]
fn nesting_monotonicity() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let outer = g.contour_range(endpoints[i], endpoints[j]).unwrap();

            for a in i..j {
                for b in (a + 1)..=j {
                    let inner = g.contour_range(endpoints[a], endpoints[b]).unwrap();
                    assert!(
                        inner.exact_energy <= outer.exact_energy,
                        "inner [{}, {}) exact_energy {} > outer [{}, {}) exact_energy {}",
                        endpoints[a].0,
                        endpoints[b].0,
                        inner.exact_energy,
                        endpoints[i].0,
                        endpoints[j].0,
                        outer.exact_energy,
                    );
                }
            }
        }
    }
}

// ── Test 6: boundary_count_at_most_two ──────────────────────────

/// For any valid contour range, the number of basis elements with
/// `is_boundary_thatch == true` is at most 2 (CR-I8).
#[test]
fn boundary_count_at_most_two() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let cr = g.contour_range(endpoints[i], endpoints[j]).unwrap();
            let thatch_count = cr.basis.iter().filter(|b| b.is_boundary_thatch).count();
            assert!(
                thatch_count <= 2,
                "boundary thatch count {} > 2 for [{}, {})",
                thatch_count,
                endpoints[i].0,
                endpoints[j].0,
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Edge cases and error paths (tests 7–8, 14)
// ════════════════════════════════════════════════════════════════════

// ── Test 7: invalid_endpoint_returns_none ────────────────────────

/// Non-lattice endpoints produce `None`.
#[test]
fn invalid_endpoint_returns_none() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let plateaus = g.plateaus();
    let first_key = *plateaus.keys().next().unwrap();

    // A coordinate that is not a plateau basis edge and not the
    // domain sentinel.
    let non_lattice = BasisEdge(7u64);

    assert!(
        !plateaus.contains_key(&non_lattice),
        "test assumption: 7 should not be a plateau basis edge",
    );

    // Non-lattice start.
    assert!(
        g.contour_range(non_lattice, BasisEdge(256u64)).is_none(),
        "non-lattice start should return None",
    );

    // Non-lattice end (not domain sentinel, not a plateau key).
    assert!(
        g.contour_range(first_key, non_lattice).is_none(),
        "non-lattice end should return None",
    );

    // Both non-lattice.
    assert!(
        g.contour_range(non_lattice, BasisEdge(13u64)).is_none(),
        "both non-lattice should return None",
    );

    // contour_range_energy should also return None.
    assert!(
        g.contour_range_energy(non_lattice, BasisEdge(256u64)).is_none(),
        "contour_range_energy with non-lattice start should return None",
    );
}

// ── Test 8: stale_endpoint_returns_none ─────────────────────────

/// After mutation that removes an endpoint, previous range returns
/// `None`.
#[test]
fn stale_endpoint_returns_none() {
    let _t = init_tracing();

    // Build initial structure using the same config but fewer observations.
    let initial_plan = Plan::new().hotspot(16, 5, 10).hotspot(48, 5, 10);
    let mut g = run_checked::<u64, u64, 8>(contour_range_config(), &initial_plan, 10);
    assert_invariants(&g);

    let endpoints_before = lattice_endpoints(&g);

    // Mutate heavily — add many observations in a different region
    // to potentially cause splits/restructuring that changes the
    // plateau lattice.
    for _ in 0..100 {
        g.observe(200u64, 10u64);
    }
    assert_invariants(&g);

    // Any old endpoint that is no longer on the lattice should
    // cause `None`.
    for ep in &endpoints_before {
        if *ep != BasisEdge(256u64) && !g.plateaus().contains_key(ep) {
            let result = g.contour_range(*ep, BasisEdge(256u64));
            assert!(result.is_none(), "stale endpoint {} should return None after mutation", ep.0);
        }
    }

    let endpoints_after = lattice_endpoints(&g);

    // All current lattice endpoints should still work.
    if endpoints_after.len() >= 2 {
        let start = endpoints_after[0];
        let end = *endpoints_after.last().unwrap();
        assert!(
            g.contour_range(start, end).is_some(),
            "current lattice endpoints should still be valid",
        );
    }
}

// ── Test 14: empty_range_rejected ───────────────────────────────

/// `start >= end` returns `None`.
#[test]
fn empty_range_rejected() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    // start == end → empty range.
    for ep in &endpoints {
        assert!(
            g.contour_range(*ep, *ep).is_none(),
            "start == end ({}) should return None",
            ep.0,
        );
        assert!(
            g.contour_range_energy(*ep, *ep).is_none(),
            "energy: start == end ({}) should return None",
            ep.0,
        );
    }

    // start > end → reversed range.
    if endpoints.len() >= 2 {
        let first = endpoints[0];
        let last = *endpoints.last().unwrap();
        assert!(g.contour_range(last, first).is_none(), "start > end should return None");
        assert!(
            g.contour_range_energy(last, first).is_none(),
            "energy: start > end should return None",
        );
    }
}

// ════════════════════════════════════════════════════════════════════
// Energy field invariants (tests 9–12, 17)
// ════════════════════════════════════════════════════════════════════

// ── Test 9: energy_ordering ─────────────────────────────────────

/// `plateau_energy <= energy` for every valid contour range with
/// non-negative observations.
#[test]
fn energy_ordering() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let cr = g.contour_range(endpoints[i], endpoints[j]).unwrap();

            // Plateau energy ≤ energy (cross_plateau_energy ≥ 0 under P1).
            assert!(
                cr.plateau_energy <= cr.energy,
                "plateau_energy {} > energy {} for [{}, {})",
                cr.plateau_energy,
                cr.energy,
                endpoints[i].0,
                endpoints[j].0,
            );

            // cross_plateau_energy = energy - plateau_energy (structural).
            assert_eq!(
                cr.cross_plateau_energy,
                cr.energy - cr.plateau_energy,
                "cross_plateau_energy mismatch for [{}, {})",
                endpoints[i].0,
                endpoints[j].0,
            );
        }
    }
}

// ── Test 10: energy_equals_basis_sum ────────────────────────────

/// `energy == Σ basis[i].sum` (§CR.6).
#[test]
fn energy_equals_basis_sum() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();

            let sum: u64 = cr.basis.iter().map(|b| b.sum).sum();
            assert_eq!(
                cr.energy, sum,
                "energy ({}) != Σ basis.sum ({}) for [{}, {})",
                cr.energy, sum, start.0, end.0,
            );
        }
    }
}

// ── Test 11: cross_plateau_energy_non_negative ──────────────────

/// `cross_plateau_energy <= energy` (i.e. non-negative for unsigned).
///
/// For unsigned integer accumulators the type prevents negatives,
/// but we assert the computation didn't underflow via wrapping.
#[test]
fn cross_plateau_energy_non_negative() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let cr = g.contour_range(endpoints[i], endpoints[j]).unwrap();

            assert!(
                cr.cross_plateau_energy <= cr.energy,
                "cross_plateau_energy {} > energy {} for [{}, {}) — possible underflow",
                cr.cross_plateau_energy,
                cr.energy,
                endpoints[i].0,
                endpoints[j].0,
            );
        }
    }
}

// ── Test 12: energy_only_matches_full ───────────────────────────

/// `contour_range_energy()` matches the energy fields of
/// `contour_range()`.
#[test]
fn energy_only_matches_full() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];

            let full = g.contour_range(start, end).unwrap();
            let energy = g.contour_range_energy(start, end).unwrap();

            assert_eq!(full.energy, energy.energy, "energy mismatch for [{}, {})", start.0, end.0);
            assert_eq!(
                full.exact_energy, energy.exact_energy,
                "exact_energy mismatch for [{}, {})",
                start.0, end.0,
            );
            assert_eq!(
                full.plateau_energy, energy.plateau_energy,
                "plateau_energy mismatch for [{}, {})",
                start.0, end.0,
            );
            assert_eq!(
                full.cross_plateau_energy, energy.cross_plateau_energy,
                "cross_plateau_energy mismatch for [{}, {})",
                start.0, end.0,
            );
            assert_eq!(
                full.plateau_count, energy.plateau_count,
                "plateau_count mismatch for [{}, {})",
                start.0, end.0,
            );
        }
    }
}

// ── Test 17: energy_exact_gap ───────────────────────────────────

/// Verify energy vs `exact_energy` relationship (§CR.13.6).
///
/// - For full-domain range: `energy == exact_energy` ($B = A = 0$).
/// - For general ranges: gap is indeterminate, but `exact_energy`
///   must equal `range_sum(start..end)`.
#[test]
fn energy_exact_gap() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    // Full-domain: energy == exact_energy.
    {
        let start = endpoints[0];
        let end = *endpoints.last().unwrap();
        let cr = g.contour_range(start, end).unwrap();
        assert_eq!(
            cr.energy, cr.exact_energy,
            "full-domain energy ({}) != exact_energy ({})",
            cr.energy, cr.exact_energy,
        );
    }

    // For all ranges: exact_energy == range_sum(start..end).
    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();
            let rs = g.range_sum(start.0..end.0);
            assert_eq!(
                cr.exact_energy, rs,
                "exact_energy ({}) != range_sum ({}) for [{}, {})",
                cr.exact_energy, rs, start.0, end.0,
            );
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Determinism and structural invariants (tests 13, 15–16, 18)
// ════════════════════════════════════════════════════════════════════

// ── Test 13: determinism ────────────────────────────────────────

/// Two calls on unmutated graph produce identical results.
#[test]
fn determinism() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];

            let a = g.contour_range(start, end).unwrap();
            let b = g.contour_range(start, end).unwrap();

            assert_eq!(a, b, "determinism failure for [{}, {}): two calls differ", start.0, end.0);

            let ea = g.contour_range_energy(start, end).unwrap();
            let eb = g.contour_range_energy(start, end).unwrap();
            assert_eq!(ea, eb, "energy determinism failure for [{}, {})", start.0, end.0);
        }
    }
}

// ── Test 15: basis_is_flat_list ─────────────────────────────────

/// Assert `cr.basis` is a flat list: all elements have `start < end`,
/// tiles are sorted by `start` and contiguous, and the first/last
/// elements span the range boundaries.
#[test]
fn basis_is_flat_list() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();

            // Non-empty basis.
            assert!(!cr.basis.is_empty(), "basis should not be empty for [{}, {})", start.0, end.0);

            // Every element has start < end.
            for b in &cr.basis {
                assert!(
                    b.start < b.end,
                    "basis element has start ({}) >= end ({}) for range [{}, {})",
                    b.start,
                    b.end,
                    start.0,
                    end.0,
                );
            }

            // Sorted by start.
            for pair in cr.basis.windows(2) {
                assert!(
                    pair[0].start < pair[1].start,
                    "basis not sorted: [{}, {}) before [{}, {})",
                    pair[0].start,
                    pair[0].end,
                    pair[1].start,
                    pair[1].end,
                );
            }

            // Contiguous: each element's end == next element's start.
            for pair in cr.basis.windows(2) {
                assert_eq!(
                    pair[0].end, pair[1].start,
                    "gap in basis tiling: [{}, {}) then [{}, {})",
                    pair[0].start, pair[0].end, pair[1].start, pair[1].end,
                );
            }

            // First element starts at range start, last ends at range end.
            assert_eq!(
                cr.basis.first().unwrap().start,
                start.0,
                "first basis element doesn't start at range start for [{}, {})",
                start.0,
                end.0,
            );
            assert_eq!(
                cr.basis.last().unwrap().end,
                end.0,
                "last basis element doesn't end at range end for [{}, {})",
                start.0,
                end.0,
            );
        }
    }
}

// ── Test 16: boundary_thatch_flag ───────────────────────────────

/// At most 2 elements have `is_boundary_thatch == true`.  They
/// appear at range boundaries (first and/or last in the sorted
/// basis).
#[test]
fn boundary_thatch_flag() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();

            let thatch_indices: Vec<usize> = cr
                .basis
                .iter()
                .enumerate()
                .filter(|(_, b)| b.is_boundary_thatch)
                .map(|(idx, _)| idx)
                .collect();

            assert!(
                thatch_indices.len() <= 2,
                "more than 2 thatch elements ({}) for [{}, {})",
                thatch_indices.len(),
                start.0,
                end.0,
            );

            // Thatching elements should be at the boundaries (first
            // and/or last position).
            let last_idx = cr.basis.len().saturating_sub(1);
            for &idx in &thatch_indices {
                assert!(
                    idx == 0 || idx == last_idx,
                    "thatch element at index {} (not 0 or {}) for [{}, {})",
                    idx,
                    last_idx,
                    start.0,
                    end.0,
                );
            }
        }
    }
}

// ── Test 18: no_straddling_ancestors ────────────────────────────

/// All contributions are whole `.sum` values — no fractional or
/// pro-rated elements.  Verifies that `energy == Σ basis.sum` and
/// that `own <= sum` for every basis element.
#[test]
fn no_straddling_ancestors() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();

            // Every basis element contributes its whole .sum to energy.
            let total: u64 = cr.basis.iter().map(|b| b.sum).sum();
            assert_eq!(
                cr.energy, total,
                "energy ({}) != Σ basis.sum ({}) — suggests pro-ration for [{}, {})",
                cr.energy, total, start.0, end.0,
            );

            // own ≤ sum for every basis element (own is a component of sum).
            for b in &cr.basis {
                assert!(
                    b.own <= b.sum,
                    "basis element [{}, {}): own ({}) > sum ({}) for range [{}, {})",
                    b.start,
                    b.end,
                    b.own,
                    b.sum,
                    start.0,
                    end.0,
                );
            }
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// select_plateaus (tests 19–23)
// ════════════════════════════════════════════════════════════════════

// ── Test 19: select_plateaus_basic ──────────────────────────────

/// Arbitrary coordinates snap outward to lattice endpoints.
/// Returned endpoints are valid for `contour_range()`.
#[test]
fn select_plateaus_basic() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();

    let test_pairs: &[(u64, u64)] = &[(10, 60), (30, 200), (1, 255)];

    for &(lo, hi) in test_pairs {
        if let Some((start, end)) = g.select_plateaus(lo, hi) {
            // Start ≤ lo (outward snap left).
            assert!(start.0 <= lo, "select_plateaus({}, {}): start {} > lo", lo, hi, start.0);
            // End ≥ hi (outward snap right).
            assert!(end.0 >= hi, "select_plateaus({}, {}): end {} < hi", lo, hi, end.0);

            // Returned endpoints are valid for contour_range().
            let cr = g.contour_range(start, end);
            assert!(
                cr.is_some(),
                "select_plateaus({}, {}) → ({}, {}) rejected by contour_range",
                lo,
                hi,
                start.0,
                end.0,
            );
        }
    }
}

// ── Test 20: select_plateaus_aligned ────────────────────────────

/// Lattice-aligned input returns unchanged endpoints.
#[test]
fn select_plateaus_aligned() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let lo = endpoints[i].0;
            let hi = endpoints[j].0;

            if let Some((start, end)) = g.select_plateaus(lo, hi) {
                assert_eq!(
                    start.0, lo,
                    "aligned select_plateaus({}, {}): start {} != lo",
                    lo, hi, start.0,
                );
                assert_eq!(end.0, hi, "aligned select_plateaus({}, {}): end {} != hi", lo, hi, end.0);
            }
        }
    }
}

// ── Test 21: select_plateaus_single_plateau ─────────────────────

/// Both coords in the same plateau → endpoints span exactly that
/// one plateau.
#[test]
fn select_plateaus_single_plateau() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    for pair in endpoints.windows(2) {
        let plateau_start = pair[0].0;
        let plateau_end = pair[1].0;

        if plateau_end - plateau_start < 2 {
            continue; // Too narrow to pick a strict interior point.
        }
        let mid = plateau_start + (plateau_end - plateau_start) / 2;

        if let Some((start, end)) = g.select_plateaus(mid, mid + 1) {
            assert_eq!(
                start.0,
                plateau_start,
                "single-plateau select_plateaus({}, {}): start {} != plateau start {}",
                mid,
                mid + 1,
                start.0,
                plateau_start,
            );
            assert_eq!(
                end.0,
                plateau_end,
                "single-plateau select_plateaus({}, {}): end {} != plateau end {}",
                mid,
                mid + 1,
                end.0,
                plateau_end,
            );
        }
    }
}

// ── Test 22: select_plateaus_full_domain ────────────────────────

/// `select_plateaus(0, 2^N)` → full domain contour range.
#[test]
fn select_plateaus_full_domain() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);

    if let Some((start, end)) = g.select_plateaus(0, 256) {
        assert_eq!(start.0, endpoints[0].0, "full-domain start mismatch");
        assert_eq!(end.0, 256, "full-domain end mismatch");

        let cr = g.contour_range(start, end).unwrap();
        assert_eq!(cr.energy, g.total_sum(), "full-domain energy mismatch");
    } else {
        panic!("select_plateaus(0, 256) returned None on non-empty tree");
    }
}

// ── Test 23: select_plateaus_compose ────────────────────────────

/// `select_plateaus(l, r)` → `contour_range(start, end)` round-trip
/// succeeds.
#[test]
fn select_plateaus_compose() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();

    let test_pairs: &[(u64, u64)] = &[(10, 60), (0, 256), (30, 200), (128, 255)];

    for &(lo, hi) in test_pairs {
        if let Some((start, end)) = g.select_plateaus(lo, hi) {
            let cr = g.contour_range(start, end);
            assert!(
                cr.is_some(),
                "select_plateaus({}, {}) → ({}, {}) rejected by contour_range",
                lo,
                hi,
                start.0,
                end.0,
            );

            assert!(start.0 <= lo, "start {} > lo {}", start.0, lo);
            assert!(end.0 >= hi, "end {} < hi {}", end.0, hi);

            let cr = cr.unwrap();
            assert!(!cr.basis.is_empty(), "select_plateaus({lo}, {hi}) produced empty basis");
        }
    }
}

// ════════════════════════════════════════════════════════════════════
// Additional coverage (tests 24–27)
// ════════════════════════════════════════════════════════════════════

// ── Test 24: empty_tree_returns_none ────────────────────────────

/// A fresh graph with no observations: `contour_range` and
/// `select_plateaus` should handle the degenerate case gracefully.
#[test]
fn empty_tree_returns_none() {
    let _t = init_tracing();
    let g: GvGraph<u64, u64, 8> = GvGraph::new(contour_range_config());

    // The graph has one plateau (the root), so the lattice is {0, 256}.
    let plateaus = g.plateaus();
    assert_eq!(plateaus.len(), 1, "fresh graph should have exactly one plateau");

    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(256u64);

    // Full-domain range on a zero-sum tree should succeed with zero energy.
    let cr = g
        .contour_range(first, end)
        .expect("full-domain on fresh graph should succeed");
    assert_eq!(cr.energy, 0, "fresh graph energy should be 0");
    assert_eq!(cr.exact_energy, 0, "fresh graph exact_energy should be 0");
    assert_eq!(cr.plateau_energy, 0, "fresh graph plateau_energy should be 0");
    assert_eq!(cr.cross_plateau_energy, 0, "fresh graph cross_plateau_energy should be 0");

    // select_plateaus should also work on a fresh graph.
    if let Some((start, sp_end)) = g.select_plateaus(0, 256) {
        let cr2 = g.contour_range(start, sp_end);
        assert!(cr2.is_some(), "select_plateaus round-trip failed on fresh graph");
    }
}

// ── Test 25: select_plateaus_reversed_returns_none ──────────────

/// `select_plateaus` with `lo >= hi` returns `None`.
#[test]
fn select_plateaus_reversed_returns_none() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();

    // lo == hi → empty interval.
    assert!(g.select_plateaus(100, 100).is_none(), "lo == hi should return None");

    // lo > hi → reversed interval.
    assert!(g.select_plateaus(200, 50).is_none(), "lo > hi should return None");

    // Zero-width at domain boundary.
    assert!(g.select_plateaus(0, 0).is_none(), "lo == hi == 0 should return None");
    assert!(g.select_plateaus(256, 256).is_none(), "lo == hi == 256 should return None");
}

// ── Test 26: plateau_count_matches_lattice ──────────────────────

/// `plateau_count` equals the number of lattice endpoints strictly
/// within `[start, end)` (i.e. the number of plateaus spanned).
#[test]
fn plateau_count_matches_lattice() {
    let _t = init_tracing();
    let g = build_multi_plateau_tree();
    let endpoints = lattice_endpoints(&g);
    let plateaus = g.plateaus();

    for i in 0..endpoints.len() {
        for j in (i + 1)..endpoints.len() {
            let start = endpoints[i];
            let end = endpoints[j];
            let cr = g.contour_range(start, end).unwrap();

            // Count the plateaus whose basis edge falls in [start, end).
            let expected_count = plateaus.range(start..end).count();

            assert_eq!(
                cr.plateau_count, expected_count,
                "plateau_count {} != expected {} for [{}, {})",
                cr.plateau_count, expected_count, start.0, end.0,
            );
        }
    }
}

// ── Test 27: diverse_topology_invariants ────────────────────────

/// Run the core contour range invariants on a tree with a different
/// topology (using `default_config()` with a spread plan) to ensure
/// the tests aren't coupled to a single tree shape.
#[test]
fn diverse_topology_invariants() {
    let _t = init_tracing();

    let plan = Plan::new().spread(256, 6, 40).hotspot(200, 20, 15);
    let g = run_checked::<u64, u64, 8>(default_config(), &plan, 10);
    assert_invariants(&g);

    let plateaus = g.plateaus();
    let mut eps: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    eps.push(BasisEdge(256u64));

    for i in 0..eps.len() {
        for j in (i + 1)..eps.len() {
            let start = eps[i];
            let end = eps[j];
            let cr = g.contour_range(start, end).unwrap();

            // §CR.6: energy == Σ basis.sum
            let sum: u64 = cr.basis.iter().map(|b| b.sum).sum();
            assert_eq!(cr.energy, sum, "diverse: energy mismatch for [{}, {})", start.0, end.0);

            // §CR.13: exact_energy == range_sum
            let rs = g.range_sum(start.0..end.0);
            assert_eq!(
                cr.exact_energy, rs,
                "diverse: exact_energy mismatch for [{}, {})",
                start.0, end.0,
            );

            // CR-I8: at most 2 boundary thatching elements.
            let thatch_count = cr.basis.iter().filter(|b| b.is_boundary_thatch).count();
            assert!(
                thatch_count <= 2,
                "diverse: thatch count {} > 2 for [{}, {})",
                thatch_count,
                start.0,
                end.0,
            );

            // Structural: basis sorted, contiguous.
            for pair in cr.basis.windows(2) {
                assert!(pair[0].start < pair[1].start, "diverse: basis not sorted");
                assert_eq!(pair[0].end, pair[1].start, "diverse: gap in basis tiling");
            }

            // Boundary alignment.
            assert_eq!(cr.basis.first().unwrap().start, start.0);
            assert_eq!(cr.basis.last().unwrap().end, end.0);

            // plateau_energy ≤ energy.
            assert!(cr.plateau_energy <= cr.energy);

            // cross_plateau_energy = energy - plateau_energy.
            assert_eq!(cr.cross_plateau_energy, cr.energy - cr.plateau_energy);

            // contour_range_energy matches.
            let e = g.contour_range_energy(start, end).unwrap();
            assert_eq!(e.energy, cr.energy);
            assert_eq!(e.exact_energy, cr.exact_energy);
        }
    }
}
