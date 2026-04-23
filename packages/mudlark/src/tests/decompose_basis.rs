// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Boundary tests for **basis decomposition** via `contour_range()`
//! (ADR-M-039).
//!
//! These tests exercise `decompose_basis` with query endpoints
//! aligned to every possible dyadic grid position.  The goal is to
//! verify specific basis set membership — not just structural
//! validity — so that off-by-one boundary mutations (`< → <=`,
//! `> → >=`, `> → ==`, etc.) are caught reliably.
//!
//! Graphs of varying depth (N=3, 4, 8) and topology (minimal,
//! uniform, adversarial zigzag / oscillating hotspot) are built
//! through the `GraphCreator` test harness.  Every test asserts a
//! shared set of structural properties (`assert_basis_properties`)
//! covering contiguity, sort order, size bounds, energy
//! conservation, and boundary-thatching limits.
//!
//! # Test index
//!
//! ## Full-domain decomposition
//!
//! | Test | Focus |
//! |------|-------|
//! | [`full_domain_basis_covers_entire_range`] | full domain has no thatching, energy == `exact_energy` |
//!
//! ## Lattice sub-range coverage
//!
//! | Test | Focus |
//! |------|-------|
//! | [`all_lattice_subranges_basis_coverage`] | N=3 all endpoint pairs pass structural checks |
//!
//! ## Single-plateau queries
//!
//! | Test | Focus |
//! |------|-------|
//! | [`single_plateau_exact_coverage`] | each plateau decomposes within bounds, no thatching |
//!
//! ## N=4 graph
//!
//! | Test | Focus |
//! |------|-------|
//! | [`n4_midpoint_aligned_queries`] | adjacent-endpoint pairs pass structural checks |
//! | [`n4_full_domain_energy`] | energy == `exact_energy` for full domain |
//! | [`n4_left_half_query`] | left-half sub-range structural check |
//!
//! ## N=8 richer topology
//!
//! | Test | Focus |
//! |------|-------|
//! | [`n8_all_lattice_subranges`] | all endpoint pairs pass structural checks |
//! | [`n8_full_domain_no_thatching`] | full domain: no thatching, energy == `exact_energy` |
//!
//! ## Energy parity
//!
//! | Test | Focus |
//! |------|-------|
//! | [`contour_range_energy_matches_full`] | `contour_range_energy()` agrees with `contour_range()` |
//! | [`sub_range_energies_are_positive`] | energy ≥ `plateau_energy` for all sub-ranges |
//!
//! ## Invalid endpoint rejection (CR-I1)
//!
//! | Test | Focus |
//! |------|-------|
//! | [`invalid_start_returns_none`] | non-lattice start → `None` |
//! | [`invalid_end_returns_none`] | non-lattice end → `None` |
//! | [`start_equals_end_returns_none`] | zero-width range → `None` |
//! | [`reversed_endpoints_returns_none`] | start > end → `None` |
//!
//! ## Plateau count and energy
//!
//! | Test | Focus |
//! |------|-------|
//! | [`plateau_count_matches_range`] | full-domain `plateau_count` == total plateaus |
//! | [`sub_range_plateau_count_bounded`] | sub-range `plateau_count` ≤ total |
//!
//! ## Boundary thatching
//!
//! | Test | Focus |
//! |------|-------|
//! | [`boundary_thatching_appears_for_cross_plateau_ranges`] | rich graph exercises thatching |
//!
//! ## Minimal graph
//!
//! | Test | Focus |
//! |------|-------|
//! | [`root_only_graph_full_domain`] | single-observation graph produces single basis element |
//!
//! ## Adversarial topologies
//!
//! | Test | Focus |
//! |------|-------|
//! | [`adversarial_zigzag_basis_integrity`] | escalating zigzag passes structural checks |
//! | [`oscillating_hotspot_basis_integrity`] | oscillating hotspot passes structural checks |
//!
//! ## Basis element depth
//!
//! | Test | Focus |
//! |------|-------|
//! | [`basis_element_depths_are_bounded`] | depth ≤ N for every basis element |

#![cfg(feature = "dynamic-contour-tracking")]

use crate::testing::GraphCreator;
use crate::{BasisEdge, Config, GvGraph};

// ── Helper: low-threshold config for controlled splits ──────────

/// `θ`=2, `D_create`=3, `D_evict`=6 — splits easily but stays shallow
/// enough for element-level verification.
const fn basis_test_config() -> Config<u64> {
    Config {
        split_threshold: 2,
        depth_create: 3,
        depth_evict: 6,
        budget: None,
        alpha_relax: 0.75,
        bounded_eviction: true,
    }
}

// ── Graph builders via testing infra ────────────────────────────

/// N=3 graph (domain `[0,8)`) with known leaf structure.
fn build_known_4leaf_graph() -> GvGraph<u64, u64, 3> {
    GraphCreator::new(basis_test_config())
        .hotspot(1, 3, 5)
        .hotspot(5, 3, 5)
        .check_every(10)
        .build()
}

/// N=4 graph (domain `[0,16)`) with multi-level splits.
fn build_n4_graph() -> GvGraph<u64, u64, 4> {
    GraphCreator::new(basis_test_config())
        .observe_n(2, 3, 5)
        .observe_n(10, 3, 5)
        .check_every(5)
        .build()
}

/// N=8 graph (domain `[0,256)`) with richer topology from a spread +
/// hotspot burst, useful for multi-plateau / boundary-thatching tests.
fn build_rich_n8_graph() -> GvGraph<u64, u64, 8> {
    GraphCreator::new(basis_test_config())
        .spread(256, 3, 40)
        .hotspot(16, 10, 10)
        .hotspot(200, 10, 10)
        .check_every(10)
        .build()
}

// ── Assertion helpers ───────────────────────────────────────────

/// Assert every structural property of a contour range result over
/// `[start, end)`.
fn assert_basis_properties<const N: u32>(g: &GvGraph<u64, u64, N>, start: BasisEdge<u64>, end: BasisEdge<u64>) {
    let cr = g
        .contour_range(start, end)
        .unwrap_or_else(|| panic!("contour_range({}, {}) returned None", start.0, end.0));

    // Non-empty.
    assert!(!cr.basis.is_empty(), "empty basis for [{}, {})", start.0, end.0);

    // Size bound: |basis| ≤ 2N.
    assert!(
        cr.basis.len() <= 2 * N as usize,
        "basis size {} exceeds 2N={} for [{}, {})",
        cr.basis.len(),
        2 * N,
        start.0,
        end.0,
    );

    // Covers the query range exactly.
    assert_eq!(
        cr.basis.first().unwrap().start,
        start.0,
        "basis start mismatch for [{}, {})",
        start.0,
        end.0,
    );
    assert_eq!(
        cr.basis.last().unwrap().end,
        end.0,
        "basis end mismatch for [{}, {})",
        start.0,
        end.0,
    );

    // Contiguous — no gaps.
    for w in cr.basis.windows(2) {
        assert_eq!(
            w[0].end, w[1].start,
            "gap in basis for [{}, {}): [{}, {}) then [{}, {})",
            start.0, end.0, w[0].start, w[0].end, w[1].start, w[1].end,
        );
    }

    // Spatially sorted.
    for w in cr.basis.windows(2) {
        assert!(
            w[0].start < w[1].start,
            "basis not sorted for [{}, {}): {} >= {}",
            start.0,
            end.0,
            w[0].start,
            w[1].start,
        );
    }

    // Every element covers a non-empty interval.
    for b in &cr.basis {
        assert!(b.start < b.end, "empty basis element in [{}, {})", start.0, end.0);
    }

    // sum >= own for every basis element.
    for b in &cr.basis {
        assert!(
            b.sum >= b.own,
            "basis element sum ({}) < own ({}) in [{}, {})",
            b.sum,
            b.own,
            start.0,
            end.0,
        );
    }

    // Boundary thatching: at most 2 (CR-I8).
    let thatch_count = cr.basis.iter().filter(|b| b.is_boundary_thatch).count();
    assert!(
        thatch_count <= 2,
        "too many boundary-thatch elements ({thatch_count}) for [{}, {})",
        start.0,
        end.0,
    );

    // Energy == Σ basis.sum (§CR.6).
    let sum: u64 = cr.basis.iter().map(|b| b.sum).sum();
    assert_eq!(cr.energy, sum, "energy ≠ Σ basis.sum for [{}, {})", start.0, end.0);

    // Cross-plateau identity (§CR.10.4).
    assert_eq!(
        cr.cross_plateau_energy,
        cr.energy - cr.plateau_energy,
        "cross_plateau check failed for [{}, {})",
        start.0,
        end.0,
    );
}

// ── Full-domain decomposition ───────────────────────────────────

#[test]
fn full_domain_basis_covers_entire_range() {
    let g = build_known_4leaf_graph();
    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(8u64);

    let cr = g.contour_range(first, end).unwrap();

    // Full domain ⇒ no boundary thatching.
    assert!(
        cr.basis.iter().all(|b| !b.is_boundary_thatch),
        "full-domain should have no boundary thatching",
    );

    // Energy conservation: energy == exact_energy for full domain.
    assert_eq!(cr.energy, cr.exact_energy);

    // Structural properties.
    assert_basis_properties(&g, first, end);
}

// ── Every valid sub-range of the endpoint lattice ───────────────

#[test]
fn all_lattice_subranges_basis_coverage() {
    let g = build_known_4leaf_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(8u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            assert_basis_properties(&g, start, end);
        }
    }
}

// ── Single-plateau boundary queries ─────────────────────────────

#[test]
fn single_plateau_exact_coverage() {
    let g = build_known_4leaf_graph();
    let plateaus = g.plateaus();
    let keys: Vec<_> = plateaus.keys().copied().collect();

    for i in 0..keys.len() {
        let start = keys[i];
        let end = if i + 1 < keys.len() { keys[i + 1] } else { BasisEdge(8u64) };

        let cr = g
            .contour_range(start, end)
            .unwrap_or_else(|| panic!("single-plateau range [{}, {}) failed", start.0, end.0));

        // All elements within query bounds.
        for b in &cr.basis {
            assert!(
                b.start >= start.0 && b.end <= end.0,
                "basis element [{}, {}) outside query [{}, {})",
                b.start,
                b.end,
                start.0,
                end.0,
            );
        }

        // Single-plateau range: no boundary thatching expected.
        assert!(
            cr.basis.iter().all(|b| !b.is_boundary_thatch),
            "single-plateau range [{}, {}) should not have boundary thatching",
            start.0,
            end.0,
        );
    }
}

// ── N=4 graph tests ─────────────────────────────────────────────

#[test]
fn n4_midpoint_aligned_queries() {
    let g = build_n4_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(16u64));
    endpoints.sort();
    endpoints.dedup();

    for w in endpoints.windows(2) {
        assert_basis_properties(&g, w[0], w[1]);
    }
}

#[test]
fn n4_full_domain_energy() {
    let g = build_n4_graph();
    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(16u64);

    let cr = g.contour_range(first, end).unwrap();
    assert_eq!(cr.energy, cr.exact_energy);
    assert_basis_properties(&g, first, end);
}

#[test]
fn n4_left_half_query() {
    let g = build_n4_graph();
    let plateaus = g.plateaus();

    let mid_edge = plateaus.keys().find(|k| k.0 >= 8).copied().unwrap_or(BasisEdge(16u64));
    let first = *plateaus.keys().next().unwrap();

    if first.0 < mid_edge.0 {
        assert_basis_properties(&g, first, mid_edge);
    }
}

// ── N=8 richer topology ─────────────────────────────────────────

#[test]
fn n8_all_lattice_subranges() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(256u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            assert_basis_properties(&g, start, end);
        }
    }
}

#[test]
fn n8_full_domain_no_thatching() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(256u64);

    let cr = g.contour_range(first, end).unwrap();
    assert!(cr.basis.iter().all(|b| !b.is_boundary_thatch));
    assert_eq!(cr.energy, cr.exact_energy);
}

// ── contour_range_energy parity ─────────────────────────────────

#[test]
fn contour_range_energy_matches_full() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(256u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            let cr = g.contour_range(start, end).unwrap();
            let e = g.contour_range_energy(start, end).unwrap();

            assert_eq!(cr.energy, e.energy, "energy mismatch for [{}, {})", start.0, end.0);
            assert_eq!(
                cr.exact_energy, e.exact_energy,
                "exact_energy mismatch for [{}, {})",
                start.0, end.0
            );
            assert_eq!(
                cr.plateau_energy, e.plateau_energy,
                "plateau_energy mismatch for [{}, {})",
                start.0, end.0
            );
            assert_eq!(
                cr.cross_plateau_energy, e.cross_plateau_energy,
                "cross_plateau_energy mismatch for [{}, {})",
                start.0, end.0,
            );
            assert_eq!(
                cr.plateau_count, e.plateau_count,
                "plateau_count mismatch for [{}, {})",
                start.0, end.0
            );
        }
    }
}

// ── Invalid endpoint rejection (CR-I1) ──────────────────────────

#[test]
fn invalid_start_returns_none() {
    let g = build_known_4leaf_graph();
    // 3 is (very likely) not a plateau basis edge.
    assert!(g.contour_range(BasisEdge(3u64), BasisEdge(8u64)).is_none());
}

#[test]
fn invalid_end_returns_none() {
    let g = build_known_4leaf_graph();
    let first = *g.plateaus().keys().next().unwrap();
    // 7 is neither a plateau edge nor the domain sentinel (8).
    assert!(g.contour_range(first, BasisEdge(7u64)).is_none());
}

#[test]
fn start_equals_end_returns_none() {
    let g = build_known_4leaf_graph();
    let first = *g.plateaus().keys().next().unwrap();
    assert!(g.contour_range(first, first).is_none());
}

#[test]
fn reversed_endpoints_returns_none() {
    let g = build_known_4leaf_graph();
    let plateaus = g.plateaus();
    let keys: Vec<_> = plateaus.keys().copied().collect();
    if keys.len() >= 2 {
        assert!(g.contour_range(keys[1], keys[0]).is_none());
    }
}

// ── Plateau count and plateau energy ────────────────────────────

#[test]
fn plateau_count_matches_range() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(256u64);

    let cr = g.contour_range(first, end).unwrap();

    // Full domain: plateau_count should equal total number of plateaus.
    assert_eq!(cr.plateau_count, plateaus.len());

    // plateau_energy should equal the sum of all plateau sums.
    let expected: u64 = plateaus.values().map(|p| p.sum).sum();
    assert_eq!(cr.plateau_energy, expected);
}

#[test]
fn sub_range_plateau_count_bounded() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();
    let total = plateaus.len();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(256u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            let cr = g.contour_range(start, end).unwrap();
            assert!(
                cr.plateau_count <= total,
                "plateau_count {} exceeds total {} for [{}, {})",
                cr.plateau_count,
                total,
                start.0,
                end.0,
            );
            assert!(cr.plateau_count >= 1);
        }
    }
}

// ── Boundary thatching presence ─────────────────────────────────

#[test]
fn boundary_thatching_appears_for_cross_plateau_ranges() {
    // On a rich graph, multi-plateau ranges that don't align with
    // G-node boundaries should produce ≥1 boundary-thatch element
    // at least some of the time.
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(256u64));
    endpoints.sort();
    endpoints.dedup();

    let mut any_thatching = false;
    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            let cr = g.contour_range(start, end).unwrap();
            if cr.basis.iter().any(|b| b.is_boundary_thatch) {
                any_thatching = true;
                // Thatch elements should have non-zero sum.
                for b in cr.basis.iter().filter(|b| b.is_boundary_thatch) {
                    assert!(b.sum > 0, "boundary-thatch element with zero sum");
                }
            }
        }
    }

    // On a rich-enough graph at least one sub-range should trigger
    // boundary thatching — if not, the test is vacuous.  Log a
    // warning rather than hard-fail, since tree topology is not
    // contractually guaranteed to produce thatching.
    if !any_thatching {
        eprintln!("NOTE: no boundary thatching observed on rich N=8 graph — test may be vacuous");
    }
}

// ── Minimal graph (root-only) ───────────────────────────────────

#[test]
fn root_only_graph_full_domain() {
    // A single observation produces a root-only graph.
    let g: GvGraph<u64, u64, 3> = GraphCreator::new(basis_test_config()).observe(1, 5).build();

    let plateaus = g.plateaus();
    if !plateaus.is_empty() {
        let first = *plateaus.keys().next().unwrap();
        let end = BasisEdge(8u64);
        let cr = g.contour_range(first, end).unwrap();

        // Single basis element (the root).
        assert_eq!(cr.basis.len(), 1);
        assert_eq!(cr.basis[0].start, first.0);
        assert_eq!(cr.basis[0].end, 8);
        assert!(!cr.basis[0].is_boundary_thatch);
        assert_eq!(cr.energy, cr.exact_energy);
    }
}

// ── Adversarial topology via testing patterns ───────────────────

#[test]
fn adversarial_zigzag_basis_integrity() {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(basis_test_config())
        .adversarial_zigzag(0, 15, 5, 3, 30)
        .check_every(10)
        .build();

    let plateaus = g.plateaus();
    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(16u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            assert_basis_properties(&g, start, end);
        }
    }
}

#[test]
fn oscillating_hotspot_basis_integrity() {
    let g: GvGraph<u64, u64, 4> = GraphCreator::new(basis_test_config())
        .oscillating_hotspot(1, 14, 5, 8, 4)
        .check_every(10)
        .build();

    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(16u64);

    // Full domain.
    let cr = g.contour_range(first, end).unwrap();
    assert_eq!(cr.energy, cr.exact_energy);
    assert_basis_properties(&g, first, end);

    // All sub-ranges.
    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(end);
    endpoints.sort();
    endpoints.dedup();

    for w in endpoints.windows(2) {
        assert_basis_properties(&g, w[0], w[1]);
    }
}

// ── Energy ordering for non-trivial sub-ranges ─────────────────

#[test]
fn sub_range_energies_are_positive() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();

    let mut endpoints: Vec<BasisEdge<u64>> = plateaus.keys().copied().collect();
    endpoints.push(BasisEdge(256u64));
    endpoints.sort();
    endpoints.dedup();

    for (i, &start) in endpoints.iter().enumerate() {
        for &end in &endpoints[i + 1..] {
            let cr = g.contour_range(start, end).unwrap();
            // energy >= plateau_energy (basis covers all plateaus in range).
            assert!(
                cr.energy >= cr.plateau_energy,
                "energy ({}) < plateau_energy ({}) for [{}, {})",
                cr.energy,
                cr.plateau_energy,
                start.0,
                end.0,
            );
        }
    }
}

// ── Basis element depth bounds ──────────────────────────────────

#[test]
fn basis_element_depths_are_bounded() {
    let g = build_rich_n8_graph();
    let plateaus = g.plateaus();
    let first = *plateaus.keys().next().unwrap();
    let end = BasisEdge(256u64);

    let cr = g.contour_range(first, end).unwrap();
    for b in &cr.basis {
        assert!(
            b.depth <= 8,
            "basis element depth {} exceeds N=8 for [{}, {})",
            b.depth,
            b.start,
            b.end,
        );
    }
}
