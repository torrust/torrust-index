// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Contour range types — the basis-set decomposition of a lattice-aligned query.
//!
//! A **contour range query** (§CR.2–§CR.6) decomposes a half-open
//! interval `[start, end)` whose endpoints lie on the **endpoint
//! lattice** — the set of plateau basis edges plus the domain
//! sentinel `2^N` — into a single **basis set** of G-nodes forming
//! a minimal cover (§CR.2).
//!
//! Each element of the basis set is a [`BasisElement`].  At most two
//! elements are **boundary thatching semi-internals** (§CR.3.2),
//! flagged by [`BasisElement::is_boundary_thatch`].
//!
//! [`ContourRange`] captures the full decomposition together with
//! pre-computed energy fields.  [`ContourRangeEnergy`] is a
//! lightweight `Copy` alternative that carries only the energy
//! scalars.
//!
//! All three public types are re-exported flat from the crate root.
//!
//! | Type                          | Visibility | Purpose                                   |
//! |-------------------------------|------------|-------------------------------------------|
//! | [`BasisElement<C, V>`]        | `pub`      | One element of the basis set (§CR.2)      |
//! | [`ContourRange<C, V>`]        | `pub`      | Full decomposition + energy (§CR.2–§CR.6) |
//! | [`ContourRangeEnergy<V>`]     | `pub`      | Energy-only result (`Copy`)               |

use std::collections::BTreeMap;

use crate::handle::GNodeId;
use crate::plateau::{BasisEdge, Plateau};
use crate::traits::{Accumulator, Coordinate};
#[cfg(debug_assertions)]
use crate::traits::{Inspectable, Proratable};

// ── BasisElement ─────────────────────────────────────────────────────

/// A basis element of a contour range decomposition (§CR.2).
///
/// Part of the minimal G-node cover of `[start, end)`.  Every
/// basis element's effective tile is pairwise disjoint from every
/// other's (CR-I3) and their union covers the range (CR-I2).
///
/// At most two basis elements in any contour range are boundary
/// thatching semi-internals (§CR.3.2), flagged by
/// `is_boundary_thatch`.  Their `.sum` includes energy from
/// territory outside the range (the surviving child's half).
///
/// # Fields
///
/// | Field                | Meaning                                            |
/// |----------------------|----------------------------------------------------|
/// | `gnode_id`           | Handle of the G-node in the G-Tree                 |
/// | `start`              | Effective tile start (inclusive)                    |
/// | `end`                | Effective tile end (exclusive)                     |
/// | `own`                | Direct accumulation (`g.own`)                      |
/// | `sum`                | Total accumulation (`g.sum`)                       |
/// | `depth`              | G-Tree depth of this node                          |
/// | `is_boundary_thatch` | `true` → boundary thatching semi-internal (§CR.3.2)|
///
/// # Examples
///
/// Basis elements appear in the decomposition as the minimal cover
/// of the query range:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, BasisEdge};
///
/// let cfg = Config {
///     split_threshold: 2u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// for _ in 0..10 {
///     g.observe(16, 5u64);
///     g.observe(48, 5u64);
/// }
///
/// // Full-domain range: the G-root is the single basis element.
/// let first = *g.plateaus().keys().next().unwrap();
/// let cr = g.contour_range(first, BasisEdge(256u64)).unwrap();
///
/// assert!(!cr.basis.is_empty());
/// let elem = &cr.basis[0];
/// assert!(elem.start < elem.end);
/// assert!(elem.sum >= elem.own);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct BasisElement<C: Coordinate, V: Accumulator> {
    /// Arena handle of the backing G-node.
    pub gnode_id: GNodeId,
    /// Effective tile start (inclusive).
    pub start: C,
    /// Effective tile end (exclusive).
    pub end: C,
    /// Direct accumulation (`g.own`).
    pub own: V,
    /// Total accumulation (`g.sum`).
    pub sum: V,
    /// G-Tree depth.
    pub depth: u32,
    /// `true` if this is a boundary thatching semi-internal (§CR.3.2).
    ///
    /// At most two per contour range — one at each boundary.  When
    /// `true`, this element's `.sum` includes energy from outside
    /// `[range.start, range.end)` via its surviving child.
    pub is_boundary_thatch: bool,
}

// ── ContourRange ─────────────────────────────────────────────────────

/// A contour-range decomposition of the G-Tree over a lattice-aligned
/// half-open interval (§CR.2–§CR.6).
///
/// Contains the basis set (§CR.2) and pre-computed energy fields
/// (§CR.6, §CR.13).  Obtain one by calling
/// [`GvGraph::contour_range()`](crate::GvGraph::contour_range).
///
/// Not `Copy` — contains a `Vec`.  Derives `Clone`.
///
/// # Fields
///
/// | Field                   | Meaning                                              |
/// |-------------------------|------------------------------------------------------|
/// | `start`                 | Range start (inclusive), on the endpoint lattice      |
/// | `end`                   | Range end (exclusive), on the endpoint lattice        |
/// | `basis`                 | Basis set: minimal G-node cover (§CR.2)              |
/// | `energy`                | Contour range energy (§CR.6): Σ `basis.sum`          |
/// | `exact_energy`          | Exact energy (§CR.13): `range_sum(start..end)`       |
/// | `plateau_energy`        | Sum of individual plateau energies (§CR.10.4)        |
/// | `cross_plateau_energy`  | energy − `plateau_energy` (§CR.10.4)                 |
/// | `plateau_count`         | Number of constituent plateaus                       |
///
/// # Examples
///
/// Obtain a full decomposition and inspect its basis set:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, BasisEdge, ContourRange};
///
/// let cfg = Config {
///     split_threshold: 2u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// for _ in 0..10 {
///     g.observe(16, 5u64);
///     g.observe(48, 5u64);
/// }
///
/// // Full-domain contour range.
/// let first = *g.plateaus().keys().next().unwrap();
/// let cr = g.contour_range(first, BasisEdge(256u64)).unwrap();
///
/// assert_eq!(cr.start, first.0);
/// assert_eq!(cr.end, 256);
/// assert!(cr.energy > 0);
/// // energy vs exact_energy gap is indeterminate (§CR.13.6);
/// // for the full-domain case they are equal (no boundary thatching).
/// assert_eq!(cr.energy, cr.exact_energy);
/// assert!(cr.basis.iter().filter(|b| b.is_boundary_thatch).count() <= 2);
/// assert_eq!(cr.cross_plateau_energy, cr.energy - cr.plateau_energy);
/// ```
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContourRange<C: Coordinate, V: Accumulator> {
    /// Range start (inclusive).  Lies on the endpoint lattice.
    pub start: C,
    /// Range end (exclusive).  Lies on the endpoint lattice.
    pub end: C,

    // ── Basis set (§CR.2) ────────────────────────────────────
    /// The basis set: minimal G-node cover of `[start, end)`,
    /// in spatial order (sorted by `start`).
    ///
    /// Size: ≤ 2N.  Elements with `is_boundary_thatch == true`
    /// number at most 2 (§CR.3.2, CR-I8).
    pub basis: Vec<BasisElement<C, V>>,

    // ── Energy (§CR.6, §CR.13) ───────────────────────────────
    /// Contour range energy (§CR.6): `Σ basis[i].sum`.
    ///
    /// Includes boundary thatching — semi-internal basis elements'
    /// `.sum` values leak energy outside the range.
    pub energy: V,

    /// Exact energy (§CR.13): `range_sum(start..end)`.
    ///
    /// Pro-rates G-nodes at the boundary under the uniform-within-
    /// cell assumption.  **Not generally ordered** relative to
    /// `energy` (§CR.13.6): the gap is $B - A$ where $B$ is
    /// boundary thatching leakage and $A$ is ancestor pro-ration,
    /// both non-negative under P1 but with indeterminate net sign.
    pub exact_energy: V,

    /// Sum of individual plateau energies within the range (§CR.10.4).
    pub plateau_energy: V,

    /// Cross-plateau energy (§CR.10.4): `energy − plateau_energy`.
    pub cross_plateau_energy: V,

    /// Number of constituent plateaus.
    pub plateau_count: usize,
}

// ── ContourRangeEnergy ───────────────────────────────────────────────

/// Energy-only result of a contour range query (§CR.6, §CR.13).
///
/// Lightweight [`Copy`] alternative to [`ContourRange`] that carries
/// only the scalar energy fields, not the basis set.
/// Obtain one by calling
/// [`GvGraph::contour_range_energy()`](crate::GvGraph::contour_range_energy).
///
/// # Fields
///
/// | Field                   | Meaning                                              |
/// |-------------------------|------------------------------------------------------|
/// | `energy`                | Contour range energy (§CR.6): Σ `basis.sum`          |
/// | `exact_energy`          | Exact energy (§CR.13): `range_sum(start..end)`       |
/// | `plateau_energy`        | Sum of individual plateau energies                   |
/// | `cross_plateau_energy`  | energy − `plateau_energy` (§CR.10.4)                 |
/// | `plateau_count`         | Number of constituent plateaus                       |
///
/// # Examples
///
/// The energy-only variant carries the same scalar fields as
/// [`ContourRange`] without allocating decomposition vectors:
///
/// ```
/// use torrust_mudlark::{Config, GvGraph, BasisEdge, ContourRangeEnergy};
///
/// let cfg = Config {
///     split_threshold: 2u64,
///     depth_create: 3,
///     depth_evict: 6,
///     budget: None,
///     alpha_relax: 0.75,
///     bounded_eviction: true,
/// };
/// let mut g = GvGraph::<u64, u64, 8>::new(cfg);
/// for _ in 0..10 {
///     g.observe(16, 5u64);
///     g.observe(48, 5u64);
/// }
///
/// let first = *g.plateaus().keys().next().unwrap();
/// let e = g.contour_range_energy(first, BasisEdge(256u64)).unwrap();
///
/// // Energy fields match the full decomposition.
/// let cr = g.contour_range(first, BasisEdge(256u64)).unwrap();
/// assert_eq!(e.energy, cr.energy);
/// assert_eq!(e.exact_energy, cr.exact_energy);
/// assert_eq!(e.plateau_count, cr.plateau_count);
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct ContourRangeEnergy<V: Accumulator> {
    /// Contour range energy (§CR.6): `Σ basis[i].sum`.
    pub energy: V,
    /// Exact energy (§CR.13): `range_sum(start..end)`.
    pub exact_energy: V,
    /// Sum of individual plateau energies within the range.
    pub plateau_energy: V,
    /// Cross-plateau energy (§CR.10.4): `energy − plateau_energy`.
    pub cross_plateau_energy: V,
    /// Number of constituent plateaus.
    pub plateau_count: usize,
}

// ── Lattice validation helpers (Phase 2, ADR-M-037) ─────────────────

/// Validate contour range endpoints against the endpoint lattice.
///
/// Returns `None` if either endpoint is not in the lattice or
/// `start >= end`.
///
/// The endpoint lattice `𝓔 = {a_0, …, a_{P-1}} ∪ {2^N}` (§CR.1).
///
/// # CR-I1 requirements
///
/// - `start` must be a plateau basis edge.
/// - `end` must be a plateau basis edge **or** the domain sentinel
///   `2^N`.
/// - `start < end`.
pub fn validate_endpoints<C: Coordinate, V: Accumulator>(
    plateaus: &BTreeMap<BasisEdge<C>, Plateau<C, V>>,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
    domain_end: C,
) -> Option<()> {
    // CR-I1: start must be a plateau basis edge.
    if !plateaus.contains_key(&start) {
        return None;
    }
    // CR-I1: end must be a basis edge or the domain sentinel.
    if end != BasisEdge(domain_end) && !plateaus.contains_key(&end) {
        return None;
    }
    // CR-I1: start < end.
    if start >= end {
        return None;
    }
    Some(())
}

/// Sum of individual plateau energies within `[start, end)`.
///
/// Collects plateaus whose basis edge `a_j ∈ [start, end)` and
/// returns `(Σ P.sum, count)`.
///
/// **plateaus(𝓡)** = `{ P_j : a_j ∈ 𝓔 ∩ [start, end) }`
pub fn compute_plateau_energy<C: Coordinate, V: Accumulator>(
    plateaus: &BTreeMap<BasisEdge<C>, Plateau<C, V>>,
    start: BasisEdge<C>,
    end: BasisEdge<C>,
) -> (V, usize) {
    let mut sum = V::zero();
    let mut count = 0usize;
    for (_, p) in plateaus.range(start..end) {
        sum = V::add(sum, p.sum);
        count += 1;
    }
    (sum, count)
}

// ── Debug-build invariant assertions (Phase 5, ADR-M-037) ───────────

/// Debug-build invariant checks for a contour range decomposition.
///
/// Validates the revised spec invariants (§CR.7):
///
/// - **CR-I8: Boundary thatch count ≤ 2** — at most two basis
///   elements are boundary thatching semi-internals.
/// - **CR-I3 (Disjointness)** — pairwise non-overlapping basis
///   element effective tiles.
/// - **CR-I2 (Complete Tiling)** — basis tiles are contiguous (no
///   internal gaps).
/// - **§CR.6 (Energy)** — `energy == Σ basis.sum`.
/// - **§CR.13.6** — `energy` vs `exact_energy` gap is *indeterminate*
///   (no universal ordering under P1; structural identity only).
/// - **§CR.10.4** — `cross_plateau_energy == energy − plateau_energy`.
///
/// Skipped in release builds.
#[cfg(debug_assertions)]
pub fn debug_assert_contour_range_invariants<C, V>(cr: &ContourRange<C, V>)
where
    C: Coordinate,
    V: Accumulator + Proratable + Inspectable,
{
    // ── CR-I8: Boundary thatch count ≤ 2 ────────────────────
    let thatch_count = cr.basis.iter().filter(|b| b.is_boundary_thatch).count();
    debug_assert!(
        thatch_count <= 2,
        "CR-I8: boundary thatch count = {} > 2 for [{:?}, {:?})",
        thatch_count,
        cr.start,
        cr.end,
    );

    // ── CR-I3: Disjointness ─────────────────────────────────
    // Basis elements sorted by start; pairwise non-overlapping.
    let mut sorted: Vec<_> = cr.basis.iter().collect();
    sorted.sort_by(|a, b| a.start.partial_cmp(&b.start).unwrap_or(std::cmp::Ordering::Equal));
    for pair in sorted.windows(2) {
        debug_assert!(
            pair[0].end <= pair[1].start,
            "CR-I3: overlapping basis elements [{:?}, {:?}) and [{:?}, {:?})",
            pair[0].start,
            pair[0].end,
            pair[1].start,
            pair[1].end,
        );
    }

    // ── CR-I2: Complete tiling ──────────────────────────────
    // Basis tiles are contiguous (no internal gaps).
    for pair in sorted.windows(2) {
        debug_assert!(
            pair[0].end >= pair[1].start,
            "CR-I2: gap between basis tiles [{:?}, {:?}) and [{:?}, {:?})",
            pair[0].start,
            pair[0].end,
            pair[1].start,
            pair[1].end,
        );
    }

    // ── §CR.6: Energy = Σ basis.sum ─────────────────────────
    let sum = cr.basis.iter().fold(V::zero(), |acc, b| V::add(acc, b.sum));
    let energy_f64 = cr.energy.to_f64_approx();
    let sum_f64 = sum.to_f64_approx();
    debug_assert!(
        (energy_f64 - sum_f64).abs() < 1e-10,
        "§CR.6: energy ({energy_f64}) != Σ basis.sum ({sum_f64})",
    );

    // ── §CR.13.6: energy vs exact_energy ────────────────────
    // The gap (energy − exact_energy) = B − A is indeterminate
    // in sign (§CR.13.6).  No universal ordering assertion.
    // Structural identity: energy = exact_energy + B − A (both
    // B, A ≥ 0 under P1).  Verifying this requires computing B
    // and A separately, deferred to dedicated tests (test 17).

    // ── §CR.10.4: cross_plateau_energy consistency ──────────
    let cross_f64 = cr.cross_plateau_energy.to_f64_approx();
    let plateau_f64 = cr.plateau_energy.to_f64_approx();
    let expected_cross = energy_f64 - plateau_f64;
    debug_assert!(
        (cross_f64 - expected_cross).abs() < 1e-10,
        "cross_plateau_energy ({cross_f64}) != energy - plateau_energy ({expected_cross})",
    );
}
