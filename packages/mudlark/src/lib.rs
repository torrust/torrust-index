// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![forbid(unsafe_code)]

//! # Mudlark
//!
//! (Named after an Australian informal name for the Magpie-lark,
//! a small bird whose mated pairs like to duet together.)
//!
//! A Dual-Tree Value-Stratified Index
//! (φ-Bounded Geometric-Value Graph, or [`GvGraph`]), where φ is the
//! split-threshold parameter that governs when a spatial region earns
//! finer resolution. The index provides adaptive-resolution spatial
//! queries and entropy-optimal proportional sampling.
//!
//! The structure consists of two trees sharing a common set of G-nodes:
//!
//! - **G-Tree (Geometric Tree):** A binary tree over dyadic intervals that
//!   answers spatial queries and tracks accumulated values.
//!
//! - **V-Tree (Value Tree):** A dynamic tournament bracket (branching factor
//!   2 or 3) governed by the max-uncle constraint, ensuring high-intensity
//!   entries reside near the root for efficient proportional sampling.
//!
//! The two trees protect each other: the V-Tree decides what earns spatial
//! resolution, while the G-Tree's routing controls which entries receive
//! observations. Together they provide entropy-optimal proportional sampling
//! within factor 1.44 of Shannon entropy (see §IDEA M-7 for the proof
//! sketch).
//!
//! The value type `V` (implementing [`Accumulator`]) must be
//! non-negative and start from zero — that is, `zero()` is both the
//! smallest possible value and the identity for addition.
//!
//! The formal spec defines a general importance interface with a menu
//! of algebraic properties that a value type *may* satisfy (see
//! §IDEA M-1.6). Different property profiles enable different
//! features — for example, a signed type could power governance but
//! would lose proportional sampling, and a type whose "empty" value
//! isn't zero would need extra work after every split.
//!
//! This crate implements only the **Standard configuration** (§IDEA M-2.6.6):
//! all properties hold, giving every [`GvGraph`] the full
//! feature set — violation-free splits, a fast path for evicting
//! empty nodes, well-defined sampling weights, and a logarithmic
//! depth guarantee — with no branching or fallback code
//! paths. All built-in numeric types (`u8`–`u128`, `f32`, `f64`)
//! satisfy this automatically; custom types need only uphold the
//! same contract. Signed integer types (`i8`–`i128`) are not
//! provided because their zero is not the smallest value —
//! they cannot satisfy the Standard configuration without
//! fallback paths for every feature they break.
//!
//! The **P**rogressive **E**ntropic-**W**avelet **E**xposure **I**mage
//! ([`Pewei`]) is the extracted significance-ordered snapshot of the
//! dynamic [`GvGraph`].
//!
//! # Three-surface visibility model (ADR-M-032)
//!
//! Every public symbol belongs to exactly one of three surfaces.
//! The compiler enforces the boundary: Surface 3 modules are
//! `pub(crate)`, so downstream crates cannot depend on internal
//! machinery.
//!
//! - **Surface 1 — Prints:** Lightweight view types users hold and
//!   inspect. `Copy` spot-readings (`Cell`, `Span`, `Node`, `GState`,
//!   `BasisEdge`, `Plateau`, `Transition`, `Terminal`), contour range
//!   decomposition types (`BasisElement`, `ContourRange`,
//!   `ContourRangeEnergy`), and
//!   owned extraction snapshots (`Pewei`, `Layer`).
//! - **Surface 2 — Film:** Opaque operational types users interact
//!   with (`GvGraph`, `Config`, traits).
//! - **Surface 3 — Emulsion:** `pub(crate)` internal machinery
//!   (arenas, tree nodes, routing, rebalance, split, evict, …).
//!
//! Exception: the `invariants` and `testing` modules are `pub`
//! + `#[doc(hidden)]` as testing affordances — downstream
//!   integration tests may call `assert_invariants()` and reuse
//!   shared test infrastructure (config presets, plan runner, etc.).

// ── Surface 1 — Prints (view types users hold and inspect) ──────
//
// Modules are private; types are re-exported flat from the crate
// root so there is exactly one canonical path per public type.
pub(crate) mod contour_range;
pub(crate) mod gnode; // also hosts GNode (Surface 3) — see module header
pub(crate) mod handle;
pub(crate) mod pewei;
pub(crate) mod plateau;
pub(crate) mod view;

// ── Surface 2 — Film (opaque operational types) ─────────────────
pub(crate) mod graph;
pub(crate) mod traits;

// ── Surface 3 — Emulsion (internal machinery) ───────────────────
//
// Crate-private. No downstream crate can import from these.
// All external access goes through the flat re-exports below
// and the public methods on GvGraph.
pub(crate) mod arena;
pub(crate) mod decay;
pub(crate) mod diagnostic;
pub(crate) mod evict;
pub(crate) mod graph_budget;
pub(crate) mod graph_extract;
pub(crate) mod graph_plateau;
pub(crate) mod graph_query;
pub(crate) mod graph_traits;
pub(crate) mod gtree;
pub(crate) mod observe;
pub(crate) mod rebalance;
pub(crate) mod split;
pub(crate) mod vnode;
pub(crate) mod vtree;

// Testing affordance (1 of 2, ADR-M-032): invariant checkers
// remain pub so integration tests can call assert_invariants().
// (The other exception is `testing` below.)
#[doc(hidden)]
pub mod invariants;

// Testing affordance (2 of 2, ADR-M-032 §E2): shared test
// infrastructure (config presets, plan runner, fluent builder,
// RNG stubs). Public so both crate and integration tests use
// the same helpers.
#[doc(hidden)]
pub mod testing;

#[cfg(test)]
mod tests;

// ── Public re-exports (ADR-M-032) ──────────────────────────────────
//
// One canonical path per public type. Downstream code should
// `use torrust_mudlark::{GvGraph, Config, Cell, …}` — never reach
// into sub-modules.
//
// Surface 1 — Prints (ADR-M-032 §S1, view types):
//   contour_range: BasisElement, ContourRange, ContourRangeEnergy
//             (contour range decomposition, ADR-M-037)
//   gnode:    GState           (node state enum)
//   graph:    GNodeInfo        (G-node structural snapshot, ADR-M-036)
//   handle:   GNodeId, VNodeId (opaque arena handles)
//   pewei:    Pewei, Layer     (extraction snapshots, Clone-only)
//             Transition, Terminal (Copy leaf/phase nodes)
//   plateau:  BasisEdge, Plateau  (contour map)
//   view:     Cell, Node, Span    (spot readings)
//
// Surface 2 — Film (ADR-M-032 §S2, operational types):
//   graph:    Config, GvGraph
//   traits:   Accumulator, Attenuatable, Coordinate, Inspectable,
//             Observation, Proratable, Rng, SpatialRead,
//             SpatialWrite, TemporalDecay, Weighable,
//             WeightedSampler
pub use contour_range::{BasisElement, ContourRange, ContourRangeEnergy};
pub use gnode::GState;
pub use graph::{Config, GNodeInfo, GvGraph};
pub use handle::{GNodeId, VNodeId};
pub use pewei::{Layer, Pewei, Terminal, Transition};
pub use plateau::{BasisEdge, Plateau};
pub use traits::{
    Accumulator, Attenuatable, Coordinate, Inspectable, Observation, Proratable, Rng, SpatialRead, SpatialWrite, TemporalDecay,
    Weighable, WeightedSampler,
};
pub use view::{Cell, Node, Span};
