// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Plan Presets ────────────────────────────────────────────────
//
//  Pre-built observation plans for common test scenarios.
//  Each returns a `Plan<u64, u64>` that can be replayed under any
//  config via `run()` / `run_checked()` / `GraphCreator::build()`.

use super::plan::Plan;

// ── Degenerate / single-hotspot ─────────────────────────────────

/// Left-deep chain: all observations at coordinate 0.
/// Forces maximum depth on the left spine.
#[must_use]
pub fn plan_left_deep(delta: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().hotspot(0, delta, n)
}

/// Right-deep chain: all observations at `domain - 1`.
#[must_use]
pub fn plan_right_deep(domain: u64, delta: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().hotspot(domain - 1, delta, n)
}

/// Single hotspot: all observations at one coordinate.
#[must_use]
pub fn plan_single_hotspot(coord: u64, delta: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().hotspot(coord, delta, n)
}

// ── Adversarial / zigzag ────────────────────────────────────────

/// Adversarial zigzag between domain extremes.
#[must_use]
pub fn plan_adversarial(domain: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().adversarial_zigzag(0, domain - 1, 5, 3, n)
}

// ── Budget / pressure ───────────────────────────────────────────

/// Budget pressure burst: even spread, then concentrated spike.
#[must_use]
pub fn plan_budget_burst(domain: u64, n_spread: usize, n_hot: usize) -> Plan<u64, u64> {
    Plan::new().burst(domain, 6, n_spread, 0, 50, n_hot)
}

/// Multi-phase: growth → pressure → relaxation.
///
/// `n` controls the base phase size.  Total observations ≈ `3.5 × n`
/// (growth `n`, pressure `n/2`, relaxation `2n`).
#[must_use]
pub fn plan_growth_pressure_relax(domain: u64, n: usize) -> Plan<u64, u64> {
    Plan::new()
        .spread(domain, 6, n)
        .hotspot(0, 50, n / 2)
        .spread(domain, 1, 2 * n)
}

// ── Range-tree ──────────────────────────────────────────────────

/// Plan equivalent to the inline `build_range_tree()` helper.
///
/// Produces a multi-node tree with known structure for range-sum,
/// sample, get, extract, and layers tests.
#[must_use]
pub fn plan_range_tree() -> Plan<u64, u64> {
    Plan::new().observe(32, 15).observe(96, 5).observe(200, 20)
}

// ── Eviction ────────────────────────────────────────────────────

/// Plan that builds deep terminals for eviction testing (N=8).
#[must_use]
pub fn plan_evictable() -> Plan<u64, u64> {
    Plan::new()
        .observe(0, 6)
        .observe(128, 6)
        .observe(64, 6)
        .observe(192, 6)
        .observe(32, 6)
        .observe(96, 6)
        .observe(160, 6)
        .observe(224, 6)
}

// ── Advanced topology ───────────────────────────────────────────

/// Cousin rivalry: adjacent subtrees `[0, 64)` and `[128, 192)` in
/// N=8 domain.  Creates concurrent violations in subtrees that share
/// a common ancestor.
#[must_use]
pub fn plan_cousin_rivalry(n: usize) -> Plan<u64, u64> {
    Plan::new().cousin_rivalry(0, 128, 64, 5, 3, n)
}

/// Adversarial cousins at coords 1 and 3 (share parent at depth 3
/// in N=4 domain).  Gentle buildup, then interleaved spikes.
#[must_use]
pub fn plan_adversarial_cousins_n4() -> Plan<u64, u64> {
    Plan::new().adversarial_cousins(1, 3, 3, 10, 50, 30, 8)
}

/// Adversarial cousins at coords 32 and 96 (share grandparent at
/// depth 1 in N=8 domain).  Larger scale version.
#[must_use]
pub fn plan_adversarial_cousins_n8() -> Plan<u64, u64> {
    Plan::new().adversarial_cousins(32, 96, 5, 20, 80, 60, 15)
}

/// Fractal fill: dyadic midpoints with sustained bursts.
/// Builds the V-Tree "top-down" with intensity at every level.
#[must_use]
pub fn plan_fractal_fill(domain: u64, depth: u32) -> Plan<u64, u64> {
    Plan::new().nested_bursts(domain, 10, depth, 5)
}

/// Diamond convergence plan: converge from extremes, then diverge.
#[must_use]
pub fn plan_diamond(domain: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().diamond(domain, 8, n)
}

/// Multi-topology stress: chains multiple diverse patterns to
/// exercise many violation sources in a single plan.
#[must_use]
pub fn plan_multi_topology_stress(domain: u64) -> Plan<u64, u64> {
    Plan::new()
        .spread(domain, 5, 30)
        .adversarial_cousins(1, 3, 3, 10, 50, 30, 8)
        .fractal_spray(domain, 8, 4)
        .neighborhood_burst(domain / 2, 4, 10, 20)
        .mirrored_growth(domain, 6, 20)
}

/// Phase-shifted near-neighbors: two adjacent coords oscillating
/// out of phase.
#[must_use]
pub fn plan_phase_shifted(a: u64, b: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().phase_shifted_oscillation(a, b, 20, 2, n / 4, 4)
}

/// Gray-code walk across the full domain.
#[must_use]
pub fn plan_gray_code(domain: u64, n: usize) -> Plan<u64, u64> {
    Plan::new().bit_flip_walk(domain, 8, n)
}
