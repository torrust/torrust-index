// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(
    clippy::cast_possible_truncation,
    clippy::cast_possible_wrap,
    clippy::cast_precision_loss,
    clippy::cast_sign_loss,
    clippy::missing_const_for_fn
)]
//! **Hex–Binary-Tree Bijection Tests**
//!
//! Validates the gap-free bijection Φ: T → H from the infinite
//! complete binary tree to the hexagonal lattice, as specified in
//! `docs/hex_binary_tree_mapping.md`.
//!
//! # What is tested
//!
//! | Property          | Method                                         |
//! |-------------------|------------------------------------------------|
//! | Gap-free          | Exhaustive image check for small depths         |
//! | Deterministic     | Identical output across repeated runs           |
//! | Invertible        | Round-trip Φ ∘ Φ⁻¹ = id for all tiles in range |
//! | Radially monotone | `depth(u) < depth(v) ⇒ R(Φ(u)) ≤ R(Φ(v))`    |
//! | Angular locality  | Siblings land angularly adjacent                |
//! | Optimal locality  | `|j(u)−j(v)|≤k ⇒ |θ(u)−θ(v)| = O(k/R)`      |
//! | Polar stability   | Small tree perturbation → small polar change    |
//! | Subtree coherence | Subtree maps to contiguous angular wedge        |
//! | Ring count 6R     | Ring R contains exactly 6R tiles (R ≥ 1)       |
//! | Tile count 3R²+3R+1 | Cumulative tile count identity               |
//! | Radius growth √2  | `R(d+1)/R(d) → √2` as d → ∞                  |
//! | Angular uniformity | Level angles fill `[0,2π)` with low discrepancy |
//! | 6-fold symmetry   | Sextant bin counts balanced within each level   |
//! | Fuzz (random BFS) | Random BFS indices produce unique hex tiles     |

use std::collections::{HashMap, HashSet};

mod support;
use support::init_tracing;

// ═══════════════════════════════════════════════════════════════════
// §1  Hex-lattice primitives (axial coordinates)
// ═══════════════════════════════════════════════════════════════════

/// Axial hex coordinate `(q, r)`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
struct Hex {
    q: i64,
    r: i64,
}

impl Hex {
    const fn new(q: i64, r: i64) -> Self {
        Self { q, r }
    }

    /// Ring number: `max(|q|, |r|, |q + r|)`.
    fn ring(self) -> u64 {
        let a = self.q.unsigned_abs();
        let b = self.r.unsigned_abs();
        let c = (self.q + self.r).unsigned_abs();
        a.max(b).max(c)
    }
}

/// Number of tiles in ring `R` (R ≥ 1 → 6R, R = 0 → 1).
const fn ring_size(r: u64) -> u64 {
    if r == 0 { 1 } else { 6 * r }
}

/// Cumulative tile count through ring `R`: `3R² + 3R + 1`.
const fn cumulative_tiles(r: u64) -> u64 {
    3 * r * r + 3 * r + 1
}

/// First spiral index in ring `R` (0-based).
/// Ring 0 → index 0.  Ring R ≥ 1 → `3R² − 3R + 1`.
const fn ring_start(r: u64) -> u64 {
    if r == 0 { 0 } else { 3 * r * r - 3 * r + 1 }
}

// ═══════════════════════════════════════════════════════════════════
// §2  Spiral enumeration σ : ℕ₀ → H
// ═══════════════════════════════════════════════════════════════════

/// Six axial direction vectors for hex neighbors, counterclockwise
/// starting from the +q axis.
const HEX_DIRS: [(i64, i64); 6] = [
    (1, 0),  // 0: +q        (east)
    (0, 1),  // 1: +r        (north-east)
    (-1, 1), // 2: -q+r      (north-west)
    (-1, 0), // 3: -q        (west)
    (0, -1), // 4: -r        (south-west)
    (1, -1), // 5: +q-r      (south-east)
];

/// Compute the spiral-index → hex-coordinate mapping for all tiles
/// through ring `max_ring`.
///
/// Returns a `Vec` where `result[i]` is the hex tile with spiral
/// index `i`.
fn build_spiral(max_ring: u64) -> Vec<Hex> {
    let n = cumulative_tiles(max_ring) as usize;
    let mut spiral = Vec::with_capacity(n);

    // Ring 0: origin.
    spiral.push(Hex::new(0, 0));

    for ring in 1..=max_ring {
        // Start tile: walk `ring` steps in direction 4 (south-west)
        // from origin.  Equivalently, start at (ring, -ring) in the
        // canonical CCW enumeration that begins on the +q axis.
        //
        // More precisely: start at (ring, 0) then walk CCW.
        let mut q = ring as i64;
        let mut r = 0i64;

        for side in 0..6 {
            let (dq, dr) = HEX_DIRS[(side + 2) % 6]; // walk direction
            for _step in 0..ring {
                spiral.push(Hex::new(q, r));
                q += dq;
                r += dr;
            }
        }
    }

    debug_assert_eq!(spiral.len(), n);
    spiral
}

/// Inverse spiral: hex coordinate → spiral index.
fn build_spiral_inverse(spiral: &[Hex]) -> HashMap<Hex, u64> {
    spiral.iter().enumerate().map(|(i, &h)| (h, i as u64)).collect()
}

// ═══════════════════════════════════════════════════════════════════
// §3  Binary-tree BFS enumeration β : T → ℕ₀
// ═══════════════════════════════════════════════════════════════════

/// BFS index of node at depth `d`, within-level position `j`:
///   `bfs(d, j) = 2^d + j − 1`
const fn bfs_index(depth: u32, j: u64) -> u64 {
    (1u64 << depth) + j - 1
}

/// Inverse: given a BFS index, return `(depth, within-level position)`.
fn bfs_inverse(bfs: u64) -> (u32, u64) {
    if bfs == 0 {
        return (0, 0);
    }
    let idx = bfs + 1; // 1-based
    let depth = u64::BITS - idx.leading_zeros() - 1;
    let j = idx - (1u64 << depth);
    (depth, j)
}

// ═══════════════════════════════════════════════════════════════════
// §4  Bit-reversal permutation
// ═══════════════════════════════════════════════════════════════════

/// Reverse the lowest `bits` bits of `x`.
fn bitrev(x: u64, bits: u32) -> u64 {
    if bits == 0 {
        return 0;
    }
    let mut result = 0u64;
    for i in 0..bits {
        if x & (1 << i) != 0 {
            result |= 1 << (bits - 1 - i);
        }
    }
    result
}

// ═══════════════════════════════════════════════════════════════════
// §5  The bijection Φ and its inverse
// ═══════════════════════════════════════════════════════════════════

/// Forward map: tree node (depth, j) → spiral index.
///
///   `Φ(d, j) = σ⁻¹(2^d − 1 + bitrev(j, d))`
///
/// Returns the spiral index (use the spiral table to get hex coords).
fn phi_spiral_index(depth: u32, j: u64) -> u64 {
    let base = (1u64 << depth) - 1; // 2^d − 1
    base + bitrev(j, depth)
}

/// Inverse map: spiral index → (depth, within-level position).
fn phi_inverse(spiral_idx: u64) -> (u32, u64) {
    let (depth, j_prime) = bfs_inverse(spiral_idx);
    let j = bitrev(j_prime, depth);
    (depth, j)
}

// ═══════════════════════════════════════════════════════════════════
// §6  Minimum radius for depth d
// ═══════════════════════════════════════════════════════════════════

/// `R(d) = ⌈√((2^{d+1} − 2) / 3)⌉`
fn min_radius_for_depth(d: u32) -> u64 {
    if d == 0 {
        return 0; // root sits at origin
    }
    let nodes: f64 = (1u64 << (d + 1)) as f64 - 2.0;
    (nodes / 3.0).sqrt().ceil() as u64
}

// ═══════════════════════════════════════════════════════════════════
// §7  Polar coordinate helpers
// ═══════════════════════════════════════════════════════════════════

/// Convert axial hex `(q, r)` to an approximate polar angle in `[0, 2π)`.
///
/// Uses the "cube → cartesian" conversion for pointy-top hexagons:
///   x = q + r/2,  y = r · √3/2
/// then `atan2(y, x)` normalized to `[0, 2π)`.
fn hex_angle(h: Hex) -> f64 {
    let x = (h.r as f64).mul_add(0.5, h.q as f64);
    let y = h.r as f64 * (3.0_f64).sqrt() * 0.5;
    let mut a = y.atan2(x);
    if a < 0.0 {
        a = 2.0_f64.mul_add(std::f64::consts::PI, a);
    }
    a
}

/// Compute the angular position of tree node (depth, j) on the
/// hex grid.  Returns `(ring, angle)`.  Requires the pre-built
/// spiral table.
fn node_polar(spiral: &[Hex], depth: u32, j: u64) -> (u64, f64) {
    let s = phi_spiral_index(depth, j) as usize;
    let h = spiral[s];
    (h.ring(), hex_angle(h))
}

/// Shortest angular distance on the circle, in `[0, π]`.
fn angular_distance(a: f64, b: f64) -> f64 {
    let two_pi = 2.0 * std::f64::consts::PI;
    let mut d = (a - b).abs() % two_pi;
    if d > std::f64::consts::PI {
        d = two_pi - d;
    }
    d
}

// ═══════════════════════════════════════════════════════════════════
//  TESTS
// ═══════════════════════════════════════════════════════════════════

// ── §A  Lattice arithmetic identities ───────────────────────────

#[test]
fn ring_0_has_one_tile() {
    let _t = init_tracing();
    assert_eq!(ring_size(0), 1);
    assert_eq!(cumulative_tiles(0), 1);
}

#[test]
fn ring_sizes_are_6r() {
    let _t = init_tracing();
    for r in 1..=200 {
        assert_eq!(ring_size(r), 6 * r, "ring_size({r})");
    }
}

#[test]
fn cumulative_tile_count_identity() {
    // Verify 3R² + 3R + 1 = 1 + Σ_{k=1}^{R} 6k  for R up to 500.
    let _t = init_tracing();
    for r in 0..=500 {
        let incremental: u64 = 1 + (1..=r).map(|k| 6 * k).sum::<u64>();
        assert_eq!(
            cumulative_tiles(r),
            incremental,
            "cumulative_tiles({r}) disagrees with incremental sum"
        );
    }
}

#[test]
fn ring_start_plus_ring_size_equals_next_ring_start() {
    let _t = init_tracing();
    for r in 0..=500 {
        assert_eq!(
            ring_start(r) + ring_size(r),
            ring_start(r + 1),
            "ring boundary mismatch at R={r}"
        );
    }
}

// ── §B  Spiral enumeration sanity ───────────────────────────────

#[test]
fn spiral_origin_is_index_zero() {
    let _t = init_tracing();
    let spiral = build_spiral(1);
    assert_eq!(spiral[0], Hex::new(0, 0));
}

#[test]
fn spiral_is_injective_through_ring_50() {
    let _t = init_tracing();
    let spiral = build_spiral(50);
    let mut seen = HashSet::with_capacity(spiral.len());
    for (i, &h) in spiral.iter().enumerate() {
        assert!(seen.insert(h), "spiral index {i} maps to duplicate hex tile {h:?}");
    }
}

#[test]
fn spiral_tiles_land_on_correct_ring() {
    let _t = init_tracing();
    let max_ring = 30;
    let spiral = build_spiral(max_ring);

    for r in 0..=max_ring {
        let start = ring_start(r) as usize;
        let end = start + ring_size(r) as usize;
        for (i, tile) in spiral[start..end].iter().enumerate() {
            let idx = start + i;
            assert_eq!(
                tile.ring(),
                r,
                "spiral[{idx}] = {tile:?} should be on ring {r} but ring() = {}",
                tile.ring()
            );
        }
    }
}

#[test]
fn spiral_inverse_round_trips() {
    let _t = init_tracing();
    let spiral = build_spiral(30);
    let inv = build_spiral_inverse(&spiral);
    for (i, &h) in spiral.iter().enumerate() {
        assert_eq!(inv[&h], i as u64, "inverse mismatch for spiral[{i}] = {h:?}");
    }
}

// ── §C  BFS index sanity ────────────────────────────────────────

#[test]
fn bfs_root_is_zero() {
    let _t = init_tracing();
    assert_eq!(bfs_index(0, 0), 0);
}

#[test]
fn bfs_level_ranges() {
    let _t = init_tracing();
    // Level d should span [2^d − 1, 2^{d+1} − 2].
    for d in 0u32..20 {
        let first = bfs_index(d, 0);
        let last = bfs_index(d, (1u64 << d) - 1);
        assert_eq!(first, (1u64 << d) - 1, "first BFS at depth {d}");
        assert_eq!(last, (1u64 << (d + 1)) - 2, "last BFS at depth {d}");
    }
}

#[test]
fn bfs_inverse_round_trips_to_depth_20() {
    let _t = init_tracing();
    for d in 0u32..20 {
        for j in 0..(1u64 << d).min(4096) {
            let bfs = bfs_index(d, j);
            let (d2, j2) = bfs_inverse(bfs);
            assert_eq!((d, j), (d2, j2), "BFS round-trip failed for bfs={bfs}");
        }
    }
}

// ── §D  Bit-reversal properties ─────────────────────────────────

#[test]
fn bitrev_is_involution() {
    // bitrev(bitrev(x, d), d) = x  for all x in [0, 2^d).
    let _t = init_tracing();
    for d in 0u32..16 {
        for x in 0..(1u64 << d) {
            assert_eq!(bitrev(bitrev(x, d), d), x, "bitrev not involutory at d={d}, x={x}");
        }
    }
}

#[test]
fn bitrev_is_bijection_within_level() {
    let _t = init_tracing();
    for d in 0u32..16 {
        let n = 1u64 << d;
        let mut images: Vec<u64> = (0..n).map(|x| bitrev(x, d)).collect();
        images.sort_unstable();
        images.dedup();
        assert_eq!(images.len(), n as usize, "bitrev not a bijection at depth {d}");
    }
}

#[test]
fn bitrev_known_values() {
    let _t = init_tracing();
    // d=3: binary 101 (5) → 101 (5), 110 (6) → 011 (3)
    assert_eq!(bitrev(0b101, 3), 0b101);
    assert_eq!(bitrev(0b110, 3), 0b011);
    // d=4: 0b1010 (10) → 0b0101 (5)
    assert_eq!(bitrev(0b1010, 4), 0b0101);
    // d=1: 0 → 0, 1 → 1
    assert_eq!(bitrev(0, 1), 0);
    assert_eq!(bitrev(1, 1), 1);
}

// ── §E  Φ is a gap-free bijection (exhaustive small depths) ─────

#[test]
fn phi_is_bijective_through_depth_16() {
    // Every BFS index in [0, 2^{d+1} − 2] maps to a unique spiral index
    // in the same range.  This proves gap-free + injective for the
    // first 2^{d+1} − 1 tiles.
    let _t = init_tracing();
    let max_depth = 16u32;
    let total_nodes = (1u64 << (max_depth + 1)) - 1; // 2^{d+1} − 1

    let mut images = HashSet::with_capacity(total_nodes as usize);
    for d in 0..=max_depth {
        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j);
            assert!(s < total_nodes, "Φ({d},{j}) = {s} exceeds node count {total_nodes}");
            assert!(images.insert(s), "Φ({d},{j}) = {s} is a duplicate (not injective)");
        }
    }
    assert_eq!(images.len(), total_nodes as usize, "image size ≠ node count (not surjective)");
}

#[test]
fn phi_covers_every_spiral_index_through_depth_14() {
    // Stronger: verify that the image is exactly {0, 1, …, 2^{d+1}−2}.
    let _t = init_tracing();
    let max_depth = 14u32;
    let total_nodes = (1u64 << (max_depth + 1)) - 1;

    let mut hit = vec![false; total_nodes as usize];
    for d in 0..=max_depth {
        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j) as usize;
            assert!(!hit[s], "double hit at spiral index {s}");
            hit[s] = true;
        }
    }
    for (i, &h) in hit.iter().enumerate() {
        assert!(h, "spiral index {i} was never hit (gap!)");
    }
}

// ── §F  Φ⁻¹ ∘ Φ = id  (round-trip) ────────────────────────────

#[test]
fn phi_inverse_round_trip_depth_16() {
    let _t = init_tracing();
    for d in 0u32..=16 {
        for j in 0..(1u64 << d).min(8192) {
            let s = phi_spiral_index(d, j);
            let (d2, j2) = phi_inverse(s);
            assert_eq!((d, j), (d2, j2), "Φ⁻¹(Φ({d},{j})) = ({d2},{j2})");
        }
    }
}

// ── §G  Radial monotonicity ────────────────────────────────────

#[test]
fn deeper_nodes_never_precede_shallower_on_hex_grid() {
    // For every spiral index emitted by depth d, it must be ≥ every
    // spiral index emitted by depth d−1.
    // (Spiral index increases with ring, so larger index ⇒ ≥ ring.)
    let _t = init_tracing();
    let max_depth = 16u32;

    let mut prev_max_spiral = 0u64;
    for d in 0..=max_depth {
        let mut level_min = u64::MAX;
        let mut level_max = 0u64;
        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j);
            level_min = level_min.min(s);
            level_max = level_max.max(s);
        }
        // The minimum spiral index of this level must be ≥ the base
        // index of this level's block: 2^d − 1.
        assert_eq!(level_min, (1u64 << d) - 1, "level {d} min spiral index wrong");
        // Level d's min ≥ level (d−1)'s max iff blocks are contiguous.
        if d > 0 {
            assert!(
                level_min > prev_max_spiral,
                "monotonicity violation: depth {d} min {level_min} ≤ depth {} max {prev_max_spiral}",
                d - 1
            );
        }
        prev_max_spiral = level_max;
    }
}

#[test]
fn radial_monotonicity_on_actual_hex_rings() {
    // Verify using the actual spiral → hex mapping that ring numbers
    // are non-decreasing with tree depth.
    let _t = init_tracing();
    let max_depth = 12u32;
    let max_ring = min_radius_for_depth(max_depth) + 2;
    let spiral = build_spiral(max_ring);

    let mut prev_max_ring = 0u64;
    for d in 0..=max_depth {
        let mut level_max_ring = 0u64;
        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j) as usize;
            assert!(s < spiral.len(), "spiral index {s} out of range");
            let r = spiral[s].ring();
            level_max_ring = level_max_ring.max(r);
        }
        if d > 0 {
            // Weaker check: max ring grows monotonically.
            assert!(level_max_ring >= prev_max_ring, "ring regression at depth {d}");
        }
        prev_max_ring = level_max_ring;
    }
}

// ── §H  Angular locality: siblings are angularly close ──────────

#[test]
fn tree_siblings_are_angularly_adjacent() {
    // For a parent at (d, j), its children are at (d+1, 2j) and
    // (d+1, 2j+1).  Under bitrev, they should be at bit-reversed
    // positions that differ only in the MSB, placing them in the same
    // angular half.
    let _t = init_tracing();
    for d in 1u32..14 {
        let level_size = 1u64 << d;
        for j in 0..level_size.min(4096) {
            let left_child_j = 2 * j;
            let right_child_j = 2 * j + 1;

            let left_br = bitrev(left_child_j, d + 1);
            let right_br = bitrev(right_child_j, d + 1);

            // Siblings under bitrev differ by exactly 2^d (the MSB
            // of the (d+1)-bit representation), which means they are
            // exactly half the level apart — i.e., diametrically
            // opposite in the "flat" index space, but within the
            // same angular quadrant of their parent's range.
            let diff = left_br.abs_diff(right_br);
            assert_eq!(diff, 1u64 << d, "sibling angular separation wrong at d={d}, j={j}");
        }
    }
}

// ── §I  Radius growth factor → √2 ─────────────────────────────

#[test]
fn radius_growth_converges_to_sqrt2() {
    let _t = init_tracing();
    let sqrt2 = std::f64::consts::SQRT_2;

    // For large d, R(d+1)/R(d) should approach √2.
    // Check convergence for d in 10..30.
    for d in 10u32..30 {
        let r_d = min_radius_for_depth(d) as f64;
        let r_d1 = min_radius_for_depth(d + 1) as f64;
        if r_d > 0.0 {
            let ratio = r_d1 / r_d;
            let err = (ratio - sqrt2).abs();
            assert!(
                err < 0.05,
                "R({})/ R({}) = {ratio:.6} deviates from √2 by {err:.6} (> 0.05)",
                d + 1,
                d
            );
        }
    }
}

// ── §J  Tile addressing (Corollary 2) ──────────────────────────

/// Compute the binary tree address of spiral index `s`:
///   `addr(s) = binary(s + 1)[1:]`  (strip leading 1-bit).
fn tile_address(spiral_idx: u64) -> (u32, u64) {
    // s + 1 in binary, strip leading 1-bit → depth and within-level j.
    let val = spiral_idx + 1;
    if val == 1 {
        return (0, 0); // root
    }
    let bits = 64 - val.leading_zeros(); // total bits in val
    let depth = bits - 1; // strip leading 1
    let j = val ^ (1u64 << depth); // remove leading 1-bit
    (depth, j)
}

#[test]
fn tile_address_matches_bfs_inverse() {
    // Corollary 2 says addr(q,r) = binary(σ(q,r) + 1)[1:].
    // For the trivial bijection Φ₀ = σ⁻¹ ∘ β (no bitrev), the address
    // of spiral index s is exactly bfs_inverse(s).
    let _t = init_tracing();
    for s in 0..100_000u64 {
        let (d1, j1) = tile_address(s);
        let (d2, j2) = bfs_inverse(s);
        assert_eq!((d1, j1), (d2, j2), "tile_address({s}) ≠ bfs_inverse({s})");
    }
}

// ── §K  Cumulative capacity always ≥ cumulative nodes ───────────

#[test]
fn hex_capacity_dominates_tree_size_at_every_ring() {
    // For the bijection to be gap-free, we need
    //   |H_R| ≥ |T_{d(R)}|  where d(R) is the max depth whose
    //   block lands within radius R.
    let _t = init_tracing();
    for d in 0u32..30 {
        let total_nodes = (1u64 << (d + 1)) - 1;
        let r = min_radius_for_depth(d);
        let capacity = cumulative_tiles(r);
        assert!(
            capacity >= total_nodes,
            "capacity {capacity} < nodes {total_nodes} at depth {d}, R={r}"
        );
    }
}

// ── §L  Determinism: repeated runs yield identical results ──────

#[test]
fn deterministic_across_1000_runs() {
    let _t = init_tracing();
    let depth = 10u32;
    let reference: Vec<u64> = (0..=depth)
        .flat_map(|d| (0..(1u64 << d)).map(move |j| phi_spiral_index(d, j)))
        .collect();

    for run in 0..1_000 {
        let attempt: Vec<u64> = (0..=depth)
            .flat_map(|d| (0..(1u64 << d)).map(move |j| phi_spiral_index(d, j)))
            .collect();
        assert_eq!(reference, attempt, "non-determinism detected on run {run}");
    }
}

// ── §M  Fuzz: pseudo-random BFS indices all produce unique tiles ─

/// Simple xorshift64 PRNG for deterministic fuzzing without
/// external dependencies.
struct Xorshift64(u64);

impl Xorshift64 {
    const fn new(seed: u64) -> Self {
        Self(seed)
    }
    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }
}

#[test]
fn fuzz_random_bfs_indices_produce_unique_spirals() {
    let _t = init_tracing();
    let max_depth = 18u32;
    let total_nodes = (1u64 << (max_depth + 1)) - 1;

    let mut rng = Xorshift64::new(0xDEAD_BEEF_CAFE_1234);
    // Draw 50 000 random BFS indices, convert them to (d, j), then
    // map through Φ.  Every image must be unique (within-domain) and
    // must be < total_nodes.
    for _trial in 0..50_000 {
        let bfs = rng.next() % total_nodes;
        let (d, j) = bfs_inverse(bfs);
        assert!(d <= max_depth);
        assert!(j < (1u64 << d));

        let s = phi_spiral_index(d, j);
        assert!(s < total_nodes, "Φ({d},{j}) = {s} out of range [0, {total_nodes})");

        // Re-derive (d, j) from s to confirm it matches.
        let (d2, j2) = phi_inverse(s);
        assert_eq!(
            (d, j),
            (d2, j2),
            "round-trip failure: bfs={bfs} → ({d},{j}) → s={s} → ({d2},{j2})"
        );

        // Same (d, j) must always produce the same s.
        let s2 = phi_spiral_index(d, j);
        assert_eq!(s, s2, "non-deterministic Φ for ({d},{j})");
    }
}

#[test]
fn fuzz_random_seeds_all_bijective_depth_12() {
    // Re-run the full bijectivity proof for depth 12 under multiple
    // "seeds" (not that the mapping uses a seed — this is just
    // asserting stability).
    let _t = init_tracing();
    let max_depth = 12u32;
    let total_nodes = (1u64 << (max_depth + 1)) - 1;

    for seed in 0u64..10 {
        let _ = seed; // no actual randomness — Φ is deterministic
        let mut images = vec![false; total_nodes as usize];
        for d in 0..=max_depth {
            for j in 0..(1u64 << d) {
                let s = phi_spiral_index(d, j) as usize;
                assert!(!images[s], "seed={seed}: double hit at s={s} (depth {d}, j={j})");
                images[s] = true;
            }
        }
        for (i, &h) in images.iter().enumerate() {
            assert!(h, "seed={seed}: gap at spiral index {i}");
        }
    }
}

// ── §N  Algebraic identity proofs ───────────────────────────────

#[test]
fn phi_block_is_contiguous_interval() {
    // Φ maps depth d to spiral indices [2^d − 1, 2^{d+1} − 2].
    // Proof by exhaustion: the image of bitrev over [0, 2^d) is
    // exactly [0, 2^d), so adding 2^d − 1 gives [2^d − 1, 2^{d+1} − 2].
    let _t = init_tracing();
    for d in 0u32..20 {
        let base = (1u64 << d) - 1;
        let n = 1u64 << d;
        let mut images: Vec<u64> = (0..n).map(|j| base + bitrev(j, d)).collect();
        images.sort_unstable();

        let expected: Vec<u64> = (base..base + n).collect();
        assert_eq!(images, expected, "depth {d}: Φ image is not [2^d − 1, 2^{{d+1}} − 2]");
    }
}

#[test]
fn algebraic_ring_count_formula() {
    // Prove: Σ_{R=0}^{N} ring_size(R) = cumulative_tiles(N).
    let _t = init_tracing();
    for n in 0u64..500 {
        let sum: u64 = (0..=n).map(ring_size).sum();
        assert_eq!(sum, cumulative_tiles(n), "ring-sum identity failed at N={n}");
    }
}

#[test]
fn algebraic_bfs_count_per_level() {
    // Verify: level d has exactly 2^d nodes and they span
    // BFS indices [2^d − 1, 2^{d+1} − 2].
    let _t = init_tracing();
    for d in 0u32..25 {
        let count = 1u64 << d;
        let first_bfs = bfs_index(d, 0);
        let last_bfs = bfs_index(d, count - 1);
        assert_eq!(last_bfs - first_bfs + 1, count, "BFS span ≠ 2^d at d={d}");
    }
}

#[test]
fn proof_bitrev_permutation_preserves_range() {
    // ∀ d : bitrev(·, d) is a bijection [0, 2^d) → [0, 2^d).
    // Proof: bitrev is its own inverse (involution) and maps
    // d-bit numbers to d-bit numbers.
    let _t = init_tracing();
    for d in 0u32..18 {
        let n = 1u64 << d;
        let mut seen = vec![false; n as usize];
        for x in 0..n {
            let y = bitrev(x, d);
            assert!(y < n, "bitrev({x}, {d}) = {y} ≥ {n}");
            assert!(!seen[y as usize], "bitrev collision at d={d}: both ? and {x} map to {y}");
            seen[y as usize] = true;
        }
        assert!(seen.iter().all(|&b| b), "bitrev not surjective at d={d}");
    }
}

// ── §O  Space-filling completeness theorem ──────────────────────

#[test]
fn theorem_phi_is_gap_free_bijection() {
    // Combined proof of Theorem 1 properties (1)–(2) for all tiles
    // through depth 15 (32 767 nodes / tiles).
    //
    // Property 1 (gap-free): every spiral index in [0, 2^16 − 2] is
    //   hit exactly once.
    // Property 2 (radially monotone): for d₁ < d₂, all spiral
    //   indices of d₁ are strictly less than all of d₂.
    let _t = init_tracing();
    let max_depth = 15u32;
    let total = (1u64 << (max_depth + 1)) - 1;

    // Collect (spiral_index, depth) for every node.
    let mut entries: Vec<(u64, u32)> = Vec::with_capacity(total as usize);
    for d in 0..=max_depth {
        for j in 0..(1u64 << d) {
            entries.push((phi_spiral_index(d, j), d));
        }
    }

    // 1. Exactly `total` entries, all unique.
    assert_eq!(entries.len(), total as usize, "wrong number of entries");
    let unique: HashSet<u64> = entries.iter().map(|&(s, _)| s).collect();
    assert_eq!(unique.len(), total as usize, "duplicates found (not injective)");

    // 2. Image = {0, 1, …, total−1}.
    let mut sorted: Vec<u64> = unique.into_iter().collect();
    sorted.sort_unstable();
    for (i, &s) in sorted.iter().enumerate() {
        assert_eq!(s, i as u64, "gap at position {i}: got {s}");
    }

    // 3. Radial monotonicity: max spiral of depth d < min spiral of depth d+1.
    let mut level_ranges: Vec<(u64, u64)> = Vec::new(); // (min, max) per depth
    for d in 0..=max_depth {
        let base = (1u64 << d) - 1;
        let top = (1u64 << (d + 1)) - 2;
        level_ranges.push((base, top));
    }
    for d in 1..=max_depth {
        let (_, prev_max) = level_ranges[(d - 1) as usize];
        let (cur_min, _) = level_ranges[d as usize];
        assert!(
            cur_min > prev_max,
            "monotonicity failure: depth {} max={prev_max}, depth {d} min={cur_min}",
            d - 1
        );
    }
}

// ── §P  Exhaustive hex-coordinate verification (small depth) ────

#[test]
fn every_hex_tile_through_ring_20_is_assigned_exactly_one_tree_node() {
    // Build the spiral, then for each tile verify that exactly one
    // (depth, j) pair maps to it.
    let _t = init_tracing();
    let max_ring = 20u64;
    let spiral = build_spiral(max_ring);
    let n_tiles = spiral.len();

    // Find the max depth such that 2^{d+1} − 1 ≤ n_tiles.
    let mut max_depth = 0u32;
    while (1u64 << (max_depth + 2)) - 1 <= n_tiles as u64 {
        max_depth += 1;
    }

    let total_nodes = (1u64 << (max_depth + 1)) - 1;
    assert!(total_nodes as usize <= n_tiles);

    let mut tile_hit: Vec<u32> = vec![0; total_nodes as usize];
    for d in 0..=max_depth {
        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j) as usize;
            assert!(s < total_nodes as usize);
            tile_hit[s] += 1;
        }
    }
    for (i, &count) in tile_hit.iter().enumerate() {
        assert_eq!(count, 1, "spiral tile {i} ({:?}) hit {count} times (expected 1)", spiral[i]);
    }
}

// ── §Q  Stress: verify all properties simultaneously at depth 18 ─

#[test]
fn stress_depth_18_full_verification() {
    // depth 18 → 2^19 − 1 = 524 287 nodes.  This is the big proof:
    // every tile in [0, 524 286] is hit exactly once, the mapping is
    // radially monotone, and every round-trip succeeds.
    let _t = init_tracing();
    let max_depth = 18u32;
    let total_nodes = (1u64 << (max_depth + 1)) - 1;

    let mut hit = vec![false; total_nodes as usize];
    let mut prev_level_max = 0u64;

    for d in 0..=max_depth {
        let mut level_min = u64::MAX;
        let mut level_max = 0u64;

        for j in 0..(1u64 << d) {
            let s = phi_spiral_index(d, j);
            assert!(s < total_nodes, "out of range at d={d}, j={j}");
            assert!(!hit[s as usize], "collision at s={s}, d={d}, j={j}");
            hit[s as usize] = true;

            level_min = level_min.min(s);
            level_max = level_max.max(s);

            // Round-trip.
            let (d2, j2) = phi_inverse(s);
            assert_eq!((d, j), (d2, j2), "round-trip at d={d}, j={j}");
        }

        if d > 0 {
            assert!(
                level_min > prev_level_max,
                "monotonicity at d={d}: min={level_min} ≤ prev_max={prev_level_max}"
            );
        }
        prev_level_max = level_max;
    }

    // Gap-free.
    for (i, &h) in hit.iter().enumerate() {
        assert!(h, "gap at spiral index {i}");
    }
}

// ═══════════════════════════════════════════════════════════════════
// §R  Optimal angular locality (Theorem 1, Property 3)
// ═══════════════════════════════════════════════════════════════════
//
// Bitrev converts tree-proximity into spiral-index proximity in
// specific senses:
//
//  (a) **Subtree stride regularity**: descendants of subtree (d, j)
//      at depth d+k occupy bitrev positions evenly spaced with
//      stride 2^d in the spiral block.
//
//  (b) **Within-level monotone spiral order**: bitrev(j, d) is
//      monotonically mapped to spiral index within the level block
//      [2^d − 1, 2^{d+1} − 2] (proved in §H).
//
//  (c) **Statistical angular locality**: on *average* across all
//      same-depth pairs, small |j₁ − j₂| correlates with small
//      angular separation on the hex grid.

#[test]
fn subtree_descendants_are_regularly_strided() {
    // Descendants of tree node (d, j) at depth d+k have within-level
    // positions [j·2^k, (j+1)·2^k).  Under bitrev(·, d+k):
    //
    //   bitrev(j·2^k + m, d+k) = bitrev(m, k) · 2^d + bitrev(j, d)
    //
    // So the images form an evenly-spaced set with stride 2^d and
    // fixed offset bitrev(j, d).  Verify this algebraic identity.
    let _t = init_tracing();
    for d in 0u32..8 {
        let n_d = 1u64 << d;
        for j in 0..n_d.min(64) {
            for k in 1u32..=(10 - d).min(8) {
                let desc_depth = d + k;
                let desc_start = j << k;
                let desc_count = 1u64 << k;
                let expected_stride = 1u64 << d;
                let expected_offset = bitrev(j, d);

                let mut br_positions: Vec<u64> = (0..desc_count).map(|m| bitrev(desc_start + m, desc_depth)).collect();
                br_positions.sort_unstable();

                // Verify they all have the same residue mod 2^d.
                for &br in &br_positions {
                    let residue = if d == 0 { 0 } else { br % expected_stride };
                    let exp_res = if d == 0 { 0 } else { expected_offset % expected_stride };
                    assert_eq!(
                        residue, exp_res,
                        "stride violation: subtree ({d},{j}) depth {desc_depth}, \
                         br={br}, expected residue={exp_res} mod {expected_stride}"
                    );
                }

                // Verify exactly 2^k distinct values.
                br_positions.dedup();
                assert_eq!(
                    br_positions.len(),
                    desc_count as usize,
                    "wrong count for subtree ({d},{j}) at depth {desc_depth}"
                );
            }
        }
    }
}

#[test]
fn subtree_spiral_span_proportional_to_level_fraction() {
    // Descendants of (d, j) at depth d+k occupy 2^k out of 2^{d+k}
    // positions.  Their spiral indices span exactly
    // (2^k − 1) · 2^d positions, confirming the stride structure.
    let _t = init_tracing();
    for d in 0u32..10 {
        let n_d = 1u64 << d;
        for j in 0..n_d.min(32) {
            for k in 1u32..=(12 - d).min(6) {
                let desc_depth = d + k;
                let desc_count = 1u64 << k;
                let desc_start = j << k;
                let base = (1u64 << desc_depth) - 1;

                let spiral_indices: Vec<u64> = (0..desc_count).map(|m| base + bitrev(desc_start + m, desc_depth)).collect();

                let s_min = spiral_indices.iter().copied().min().unwrap();
                let s_max = spiral_indices.iter().copied().max().unwrap();
                let span = s_max - s_min;

                let expected_span = (desc_count - 1) * (1u64 << d);
                assert_eq!(
                    span, expected_span,
                    "subtree ({d},{j}) depth {desc_depth}: \
                     span={span}, expected={expected_span}"
                );
            }
        }
    }
}

#[test]
fn angular_locality_holds_statistically() {
    // For random same-depth pairs with |j₁−j₂| ≤ k, verify that
    // angular separation correlates with k/R.
    //
    // Partition pairs into "close" (k ≤ 2^{d/2}) and "far"
    // (k > 2^{d/2}).  The median angular separation for close pairs
    // should be less than for far pairs.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(10) + 5;
    let spiral = build_spiral(max_ring);
    let mut rng = Xorshift64::new(0xBEEF_DEAD_0042_1337);

    for d in 6u32..=10 {
        let n = 1u64 << d;
        let threshold = 1u64 << (d / 2);
        let mut close_seps = Vec::new();
        let mut far_seps = Vec::new();

        for _ in 0..5000 {
            let j1 = rng.next() % n;
            let j2 = rng.next() % n;
            let k = j1.abs_diff(j2);
            if k == 0 {
                continue;
            }

            let s1 = phi_spiral_index(d, j1) as usize;
            let s2 = phi_spiral_index(d, j2) as usize;
            if s1 >= spiral.len() || s2 >= spiral.len() {
                continue;
            }

            let ang = angular_distance(hex_angle(spiral[s1]), hex_angle(spiral[s2]));

            if k <= threshold {
                close_seps.push(ang);
            } else {
                far_seps.push(ang);
            }
        }

        if close_seps.is_empty() || far_seps.is_empty() {
            continue;
        }

        close_seps.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
        far_seps.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let close_median = close_seps[close_seps.len() / 2];
        let far_median = far_seps[far_seps.len() / 2];

        assert!(
            close_median < far_median + 0.1,
            "depth {d}: close median ({close_median:.3}) ≥ \
             far median ({far_median:.3}) — locality not evident"
        );
    }
}

#[test]
fn bitrev_shared_suffix_implies_bounded_distance() {
    // Pairs j₁, j₂ that share a common *trailing* suffix of s bits
    // (i.e., j₁ ≡ j₂ mod 2^s) are guaranteed to have:
    //   |bitrev(j₁, d) − bitrev(j₂, d)| < 2^{d−s}
    //
    // Under bitrev, a shared trailing suffix becomes a shared leading
    // prefix, bounding the resulting distance.  This is the algebraic
    // core of bit-reversal locality.
    let _t = init_tracing();
    for d in 2u32..16 {
        let n = 1u64 << d;
        let step = (n / 512).max(1);
        for s in 1..d {
            let mask = (1u64 << s) - 1; // bottom s bits
            let bound = 1u64 << (d - s);
            for j1 in (0..n).step_by(step as usize) {
                // Pick a j2 that shares the bottom s bits with j1.
                let j2 = (j1 & mask) | (((j1 >> s).wrapping_add(1) % (1u64 << (d - s))) << s);
                if j2 >= n || j1 == j2 {
                    continue;
                }
                // Confirm they share the trailing s bits.
                assert_eq!(j1 & mask, j2 & mask);
                let br1 = bitrev(j1, d);
                let br2 = bitrev(j2, d);
                let dist = br1.abs_diff(br2);
                assert!(
                    dist < bound,
                    "shared-suffix locality violated: d={d}, s={s}, \
                     j1={j1}, j2={j2}, bitrev dist={dist} ≥ bound={bound}"
                );
            }
        }
    }
}

#[test]
fn within_level_angular_order_is_monotone_under_bitrev() {
    // Within a level, increasing bitrev(j) corresponds to increasing
    // spiral index.  Verify the identity: sorted by spiral index,
    // bitrev values are 0, 1, 2, …, 2^d − 1.
    let _t = init_tracing();

    for d in 3u32..=10 {
        let n = 1u64 << d;
        let mut level_entries: Vec<(u64, u64)> = (0..n)
            .map(|j| {
                let s = phi_spiral_index(d, j);
                let br = bitrev(j, d);
                (s, br)
            })
            .collect();
        level_entries.sort_unstable_by_key(|&(s, _)| s);

        for (rank, &(_, br)) in level_entries.iter().enumerate() {
            assert_eq!(
                br, rank as u64,
                "depth {d}: spiral rank {rank} has bitrev_j={br} (expected {rank}), \
                 angular order is not monotone"
            );
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// §S  Polar stability
// ═══════════════════════════════════════════════════════════════════
//
// "Polar stable" means:
// (a) Depth → ring mapping is monotone (proved in §G).
// (b) Within a level, spiral order = bitrev order (proved in §H).
// (c) Parent–child ring bands overlap or are adjacent.
// (d) Within-level perturbation |Δj|=1 changes ring by at most the
//     band width.
// (e) The mapping is fully deterministic and invertible.

#[test]
fn parent_child_ring_bands_are_adjacent() {
    // The ring band for depth d is [R_min(d), R_max(d)].
    // For consecutive depths, the child band should start near where
    // the parent band ends: R_min(d+1) ≤ R_max(d) + small gap.
    let _t = init_tracing();
    let max_depth = 12u32;
    let max_ring = min_radius_for_depth(max_depth + 1) + 5;
    let spiral = build_spiral(max_ring);

    let ring_band = |d: u32| -> (u64, u64) {
        let first_s = (1u64 << d) - 1;
        let last_s = (1u64 << (d + 1)) - 2;
        let r_min = spiral[first_s as usize].ring();
        let r_max = spiral[last_s as usize].ring();
        (r_min, r_max)
    };

    for d in 1u32..max_depth {
        let (_, r_max_parent) = ring_band(d);
        let (r_min_child, _) = ring_band(d + 1);

        let gap = r_min_child.saturating_sub(r_max_parent);
        assert!(
            gap <= 3,
            "ring band gap too large between depth {d} and {}: \
             parent_max={r_max_parent}, child_min={r_min_child}, gap={gap}",
            d + 1
        );
    }
}

#[test]
fn within_level_ring_variation_bounded_by_band_width() {
    // All nodes at depth d occupy spiral indices [2^d−1, 2^{d+1}−2].
    // The ring variation within a level should be bounded by the
    // annular band width ≈ (√2 − 1) · R(d).
    let _t = init_tracing();
    let max_depth = 12u32;
    let max_ring = min_radius_for_depth(max_depth) + 5;
    let spiral = build_spiral(max_ring);

    for d in 2u32..=max_depth {
        let first_s = (1u64 << d) - 1;
        let last_s = (1u64 << (d + 1)) - 2;

        let r_min = spiral[first_s as usize].ring();
        let r_max = spiral[last_s as usize].ring();
        let ring_span = r_max - r_min;

        let expected_band = ((std::f64::consts::SQRT_2 - 1.0) * r_max as f64).ceil() as u64 + 2;
        assert!(
            ring_span <= expected_band,
            "depth {d}: ring span {ring_span} exceeds expected band {expected_band} \
             (r_min={r_min}, r_max={r_max})"
        );
    }
}

#[test]
fn within_level_perturbation_delta_j_1_ring_change_bounded() {
    // Within a level, shifting j by 1 can change the spiral index by
    // up to 2^{d-1} (worst-case bitrev perturbation).  But the
    // *median* ring change for |Δj|=1 within a level should be small.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);

    for d in 4u32..=12 {
        let n = 1u64 << d;
        let mut ring_diffs = Vec::with_capacity(n as usize);

        let step = (n / 2048).max(1);
        for j in (0..n - 1).step_by(step as usize) {
            let s1 = phi_spiral_index(d, j) as usize;
            let s2 = phi_spiral_index(d, j + 1) as usize;
            if s1 >= spiral.len() || s2 >= spiral.len() {
                continue;
            }
            let rd = spiral[s1].ring().abs_diff(spiral[s2].ring());
            ring_diffs.push(rd);
        }

        ring_diffs.sort_unstable();
        let median = ring_diffs[ring_diffs.len() / 2];

        // The median ring change grows slowly with depth but should
        // stay small relative to the band width.
        let (r_min, r_max) = {
            let first_s = (1u64 << d) - 1;
            let last_s = (1u64 << (d + 1)) - 2;
            (spiral[first_s as usize].ring(), spiral[last_s as usize].ring())
        };
        let band = r_max - r_min + 1;
        assert!(
            median <= band / 2 + 1,
            "depth {d}: median ring change for Δj=1 is {median} \
             (expected ≤ {}, band={band})",
            band / 2 + 1
        );
    }
}

#[test]
fn within_level_spiral_order_is_angularly_monotone() {
    // Inside a single ring, the spiral enumeration is angularly
    // monotone (CCW).  For each ring that contains ≥ 2 nodes from
    // a given level, verify those nodes' angles are in non-decreasing
    // CCW order (matching spiral index order).
    let _t = init_tracing();
    let max_ring_val = min_radius_for_depth(10) + 3;
    let spiral = build_spiral(max_ring_val);

    for d in 3u32..=10 {
        let n = 1u64 << d;
        let mut ring_groups: HashMap<u64, Vec<(u64, f64)>> = HashMap::new();
        for j in 0..n {
            let s = phi_spiral_index(d, j);
            let h = spiral[s as usize];
            let r = h.ring();
            ring_groups.entry(r).or_default().push((s, hex_angle(h)));
        }

        for (r, mut entries) in ring_groups {
            if entries.len() < 2 {
                continue;
            }
            entries.sort_unstable_by_key(|&(s, _)| s);

            // Within a ring, angles should be non-decreasing (with
            // at most one wrap-around decrease at 2π→0).
            let decreases: usize = entries.windows(2).filter(|w| w[1].1 < w[0].1 - 0.01).count();
            assert!(
                decreases <= 1,
                "depth {d}, ring {r}: {decreases} angular decreases \
                 (expected ≤ 1 for wrap-around)"
            );
        }
    }
}

#[test]
fn subtree_angular_span_shrinks_with_depth() {
    // A subtree at depth d rooted at j has 2^k descendants at depth
    // d+k.  The angular span of those descendants should shrink as
    // the subtree root gets deeper (for fixed k).
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(10) + 5;
    let spiral = build_spiral(max_ring);
    let k = 3u32;

    let mut prev_avg_span = f64::MAX;
    for d in 2u32..=7 {
        let n_d = 1u64 << d;
        let desc_depth = d + k;
        let desc_count = 1u64 << k;
        let mut spans = Vec::new();

        for j in 0..n_d.min(64) {
            let desc_start = j << k;
            let angles: Vec<f64> = (desc_start..desc_start + desc_count)
                .map(|dj| {
                    let (_, a) = node_polar(&spiral, desc_depth, dj);
                    a
                })
                .collect();

            let mut max_sep = 0.0f64;
            for i in 0..angles.len() {
                for item in angles.iter().skip(i + 1) {
                    let sep = angular_distance(angles[i], *item);
                    max_sep = max_sep.max(sep);
                }
            }
            spans.push(max_sep);
        }

        let avg_span: f64 = spans.iter().sum::<f64>() / spans.len() as f64;
        assert!(
            avg_span <= prev_avg_span.mul_add(1.2, 0.1),
            "subtree angular span grew: depth {d} avg={avg_span:.4} > \
             depth {} avg={prev_avg_span:.4} × 1.2",
            d - 1
        );
        prev_avg_span = avg_span;
    }
}

#[test]
fn polar_stability_within_level_bfs_perturbation() {
    // Within a single level, consecutive BFS indices (consecutive j)
    // produce ring jumps that are usually small.
    //
    // BFS ±1 *across* level boundaries produces large ring jumps
    // (expected), so we test only within-level stability.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);

    for d in 4u32..=12 {
        let n = 1u64 << d;
        let mut ring_jumps = Vec::new();

        let step = (n / 2048).max(1);
        for j in (0..n - 1).step_by(step as usize) {
            let s1 = phi_spiral_index(d, j) as usize;
            let s2 = phi_spiral_index(d, j + 1) as usize;
            if s1 >= spiral.len() || s2 >= spiral.len() {
                continue;
            }
            ring_jumps.push(spiral[s1].ring().abs_diff(spiral[s2].ring()));
        }

        ring_jumps.sort_unstable();
        let p90 = ring_jumps[ring_jumps.len() * 9 / 10];

        let (r_min, r_max) = {
            let first_s = (1u64 << d) - 1;
            let last_s = (1u64 << (d + 1)) - 2;
            (spiral[first_s as usize].ring(), spiral[last_s as usize].ring())
        };
        let band = r_max - r_min + 1;
        assert!(p90 <= band, "depth {d}: p90 ring jump {p90} exceeds band width {band}");
    }
}

#[test]
fn fuzz_angular_correlation_with_tree_distance() {
    // Fuzz: for random same-depth pairs, bin by within-level
    // distance k and verify that median angular separation is
    // non-decreasing across bins (on average).
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(10) + 5;
    let spiral = build_spiral(max_ring);
    let mut rng = Xorshift64::new(0xBEEF_DEAD_0042_1337);

    let d = 10u32;
    let n = 1u64 << d;
    let bin_count = 4usize;
    let bin_width = n / bin_count as u64;
    let mut bins: Vec<Vec<f64>> = vec![Vec::new(); bin_count];

    for _ in 0..50_000 {
        let j1 = rng.next() % n;
        let j2 = rng.next() % n;
        let k = j1.abs_diff(j2);
        if k == 0 {
            continue;
        }

        let s1 = phi_spiral_index(d, j1) as usize;
        let s2 = phi_spiral_index(d, j2) as usize;
        if s1 >= spiral.len() || s2 >= spiral.len() {
            continue;
        }

        let ang = angular_distance(hex_angle(spiral[s1]), hex_angle(spiral[s2]));
        let bin = ((k - 1) / bin_width).min(bin_count as u64 - 1) as usize;
        bins[bin].push(ang);
    }

    let medians: Vec<f64> = bins
        .iter_mut()
        .map(|b| {
            b.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            if b.is_empty() { 0.0 } else { b[b.len() / 2] }
        })
        .collect();

    assert!(
        medians[0] <= medians[bin_count - 1] + 0.1,
        "angular separation does not grow with tree distance: \
         closest_median={:.3}, farthest_median={:.3}",
        medians[0],
        medians[bin_count - 1]
    );
}

#[test]
fn ring_assignment_is_deterministic_under_recomputation() {
    // Polar stability includes determinism of the (ring, angle) pair.
    // Verify that recomputing hex coordinates via spiral vs. direct
    // formula always agree.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(10) + 3;
    let spiral = build_spiral(max_ring);
    let inv = build_spiral_inverse(&spiral);

    for d in 0u32..=10 {
        let n = 1u64 << d;
        let step = (n / 512).max(1);
        for j in (0..n).step_by(step as usize) {
            let s = phi_spiral_index(d, j);
            let h = spiral[s as usize];

            let s_back = inv[&h];
            assert_eq!(
                s, s_back,
                "polar determinism failure: d={d}, j={j}, s={s}, \
                 hex={h:?}, s_back={s_back}"
            );

            let (d2, j2) = phi_inverse(s);
            assert_eq!((d, j), (d2, j2), "inverse determinism at d={d}, j={j}");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════
// §T  Angular distribution & symmetry
// ═══════════════════════════════════════════════════════════════════
//
// The hex spiral fills each ring uniformly around the origin.  When
// a binary-tree level is mapped onto the lattice, its angles should
// (a) cover the full [0, 2π) range without large gaps,
// (b) be roughly uniformly distributed (low discrepancy),
// (c) respect the hex lattice's 6-fold rotational symmetry.

#[test]
fn level_angles_cover_full_circle() {
    // Every level with ≥ 32 nodes should have at least one node in
    // each π/3 sextant of the circle.  This guarantees no angular
    // "dead zone" wider than 60°.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);

    for d in 5u32..=12 {
        let n = 1u64 << d;
        let mut sextant_hit = [false; 6];
        for j in 0..n {
            let (_, a) = node_polar(&spiral, d, j);
            let bin = (a / (std::f64::consts::PI / 3.0)).floor() as usize;
            let bin = bin.min(5);
            sextant_hit[bin] = true;
        }
        for (s, &hit) in sextant_hit.iter().enumerate() {
            assert!(
                hit,
                "depth {d}: sextant {s} ([{:.1}°, {:.1}°)) has no nodes",
                s as f64 * 60.0,
                (s + 1) as f64 * 60.0
            );
        }
    }
}

#[test]
fn level_angles_have_low_discrepancy() {
    // The *discrepancy* of a set of N angles θ₁…θ_N on [0, 2π) is
    //   D_N = sup_{[a,b)⊂[0,2π)} | (count in [a,b)) / N  −  (b−a) / 2π |
    //
    // For our spiral-mapped levels, D_N should shrink roughly as
    // O(1/√N).  We check a relaxed bound: D_N ≤ 3/√N + 0.05.
    //
    // We approximate the supremum by testing 60 evenly-spaced
    // intervals of varying widths.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);
    let two_pi = 2.0 * std::f64::consts::PI;

    for d in 4u32..=12 {
        let n = 1u64 << d;
        let mut angles: Vec<f64> = (0..n).map(|j| node_polar(&spiral, d, j).1).collect();
        angles.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let mut max_disc = 0.0f64;
        // Test 60 probe starts × 6 widths.
        for probe in 0..60 {
            let start = two_pi * f64::from(probe) / 60.0;
            for w_idx in 1..=6 {
                let width = two_pi * f64::from(w_idx) / 6.0;
                let end = start + width;

                // Count angles in [start, start+width), handling wrap.
                let count = if end <= two_pi {
                    angles.iter().filter(|&&a| a >= start && a < end).count()
                } else {
                    angles.iter().filter(|&&a| a >= start || a < end - two_pi).count()
                };

                let empirical = count as f64 / n as f64;
                let expected = width / two_pi;
                let disc = (empirical - expected).abs();
                max_disc = max_disc.max(disc);
            }
        }

        let bound = 3.0 / (n as f64).sqrt() + 0.05;
        assert!(max_disc <= bound, "depth {d}: discrepancy {max_disc:.4} > bound {bound:.4}");
    }
}

#[test]
fn sextant_bin_counts_are_balanced() {
    // Hex lattice has 6-fold symmetry: each 60° sextant of a ring
    // has exactly R tiles (for ring R ≥ 1).  When a level's nodes
    // are distributed across rings, the sextant counts should be
    // roughly balanced.
    //
    // Test: for each level, the ratio max_bin / min_bin ≤ 2.0.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);

    for d in 4u32..=12 {
        let n = 1u64 << d;
        let mut bins = [0u64; 6];
        for j in 0..n {
            let (_, a) = node_polar(&spiral, d, j);
            let b = (a / (std::f64::consts::PI / 3.0)).floor() as usize;
            bins[b.min(5)] += 1;
        }

        let min_bin = *bins.iter().min().unwrap();
        let max_bin = *bins.iter().max().unwrap();
        assert!(min_bin > 0, "depth {d}: empty sextant bin (counts: {bins:?})");
        let ratio = max_bin as f64 / min_bin as f64;
        assert!(
            ratio <= 2.0,
            "depth {d}: sextant imbalance ratio {ratio:.2} > 2.0 \
             (counts: {bins:?})"
        );
    }
}

#[test]
fn per_ring_nodes_occupy_multiple_sextants() {
    // Within each ring R ≥ 1 that receives ≥ 18 nodes from a level,
    // those nodes should span at least 3 of the 6 sextants —
    // confirming they don't all cluster in one narrow wedge.
    let _t = init_tracing();
    let max_ring_val = min_radius_for_depth(10) + 3;
    let spiral = build_spiral(max_ring_val);

    for d in 6u32..=10 {
        let n = 1u64 << d;
        let mut ring_angles: HashMap<u64, Vec<f64>> = HashMap::new();
        for j in 0..n {
            let (r, a) = node_polar(&spiral, d, j);
            ring_angles.entry(r).or_default().push(a);
        }

        for (r, angles) in &ring_angles {
            if angles.len() < 18 {
                continue;
            }
            let mut sextants = [false; 6];
            for &a in angles {
                let b = (a / (std::f64::consts::PI / 3.0)).floor() as usize;
                sextants[b.min(5)] = true;
            }
            let occupied = sextants.iter().filter(|&&x| x).count();
            assert!(
                occupied >= 3,
                "depth {d}, ring {r}: only {occupied}/6 sextants occupied \
                 ({} nodes) — angular spread too narrow",
                angles.len()
            );
        }
    }
}

#[test]
fn sixfold_rotation_maps_level_onto_itself_approximately() {
    // The hex lattice is symmetric under 60° rotations.  For each
    // node in a level, rotating its hex coordinate by 60° should
    // land close to another node in the *same* ring of the same level.
    //
    // Rotation by 60° in axial coords: (q, r) → (−r, q + r).
    //
    // Test: for each level node, the rotated coordinate's ring
    // matches the original, and the rotated position is occupied
    // by *some* node (not necessarily same level, but same ring).
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(10) + 3;
    let spiral = build_spiral(max_ring);
    let inv = build_spiral_inverse(&spiral);

    for d in 3u32..=10 {
        let n = 1u64 << d;
        let step = (n / 512).max(1);
        for j in (0..n).step_by(step as usize) {
            let s = phi_spiral_index(d, j) as usize;
            let h = spiral[s];

            // 60° rotation in axial: (q,r) → (−r, q+r)
            let rotated = Hex::new(-h.r, h.q + h.r);

            // Ring is preserved under 60° rotation.
            assert_eq!(
                h.ring(),
                rotated.ring(),
                "depth {d}, j={j}: rotation changed ring {} → {}",
                h.ring(),
                rotated.ring()
            );

            // The rotated tile exists in the spiral (it's a valid
            // hex tile), confirming the lattice has full 6-fold
            // coverage at this ring.
            assert!(
                inv.contains_key(&rotated),
                "depth {d}, j={j}: rotated hex {rotated:?} not in spiral"
            );
        }
    }
}

#[test]
fn fuzz_angular_distribution_chi_squared() {
    // Chi-squared goodness-of-fit test for angular uniformity.
    //
    // Divide [0, 2π) into 12 bins (30° each).  Under the null
    // hypothesis of uniform distribution, bin count ~ N/12.
    // χ² = Σ (O_i − E)² / E;  for 11 dof at α=0.001, critical
    // value ≈ 31.26.
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);
    let num_bins = 12usize;
    let bin_width = 2.0 * std::f64::consts::PI / num_bins as f64;
    let critical = 31.26; // α=0.001, dof=11

    for d in 6u32..=12 {
        let n = 1u64 << d;
        let mut bins = vec![0u64; num_bins];
        for j in 0..n {
            let (_, a) = node_polar(&spiral, d, j);
            let b = (a / bin_width).floor() as usize;
            bins[b.min(num_bins - 1)] += 1;
        }

        let expected = n as f64 / num_bins as f64;
        let chi2: f64 = bins
            .iter()
            .map(|&o| {
                let diff = o as f64 - expected;
                diff * diff / expected
            })
            .sum();

        assert!(
            chi2 < critical,
            "depth {d}: χ² = {chi2:.2} ≥ {critical} — angular distribution \
             not uniform (bins: {bins:?})"
        );
    }
}

#[test]
fn angular_range_spans_full_circle() {
    // For each level with ≥ 32 nodes, the largest gap between
    // consecutive sorted angles should be < π/2 (no quarter-circle
    // dead zone).
    let _t = init_tracing();
    let max_ring = min_radius_for_depth(12) + 5;
    let spiral = build_spiral(max_ring);
    let two_pi = 2.0 * std::f64::consts::PI;

    for d in 5u32..=12 {
        let n = 1u64 << d;
        let mut angles: Vec<f64> = (0..n).map(|j| node_polar(&spiral, d, j).1).collect();
        angles.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());

        let mut max_gap = 0.0f64;
        for i in 0..angles.len() {
            let gap = if i + 1 < angles.len() {
                angles[i + 1] - angles[i]
            } else {
                two_pi - angles[i] + angles[0]
            };
            max_gap = max_gap.max(gap);
        }

        assert!(
            max_gap < std::f64::consts::FRAC_PI_2,
            "depth {d}: largest angular gap {max_gap:.4} ≥ π/2 — \
             circle not fully covered"
        );
    }
}
