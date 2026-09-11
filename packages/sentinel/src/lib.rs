// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![forbid(unsafe_code)]

//! # Spectral Sentinel
//!
//! Hierarchical online subspace anomaly detection for positionally
//! structured observation streams.
//!
//! Spectral Sentinel combines Mudlark's adaptive spatial index with a layer
//! of low-rank statistical trackers. Mudlark ranks spatial entries by
//! observation volume; Spectral Sentinel selects significant V-Tree entries,
//! closes them under G-tree ancestry, and scores incoming batches against
//! learned subspace models for that selected structure.
//!
//! The core engine, [`SpectralSentinel`], is generic over the coordinate
//! type `C`, accumulator type `V`, and bit-width `N`, mirroring Mudlark's
//! `GvGraph<C, V, N>` triple. Each cell tracker analyses the suffix bits
//! `[d, N)` at width `w = N - d`, where `d` is the cell's G-tree depth.
//!
//! That width is bounded at both ends. Below, a tracker needs at least two
//! dimensions before a residual means anything. Above, a coordinate value
//! reaches its tracker as a centred bit vector, and [`CentredBits`] carries
//! its values in a fixed array of 128 slots — so a width past that has
//! nowhere to put its bits, and is refused at construction rather than
//! modelled at whatever width the array happens to hold. [`Sentinel128`]
//! sits exactly at that ceiling.
//!
//! The crate provides two convenience aliases:
//!
//! - [`Sentinel128`] — `SpectralSentinel<u128, u64, 128>`, the default
//!   for IPv6-class domains.
//! - [`Sentinel64`] — `SpectralSentinel<u64, u64, 64>`, for 64-bit
//!   domains.
//!
//! Input values must have **hierarchical positional structure**: leading
//! bits define coarse groupings and successive bits refine them. IPv6-like
//! address spaces are a natural fit. Pseudo-random identifiers, hashes,
//! UUIDs, and nonces have no meaningful suffix structure for the model to
//! learn, so Spectral Sentinel will still process them but the measurements
//! will not be useful.
//!
//! # Architecture
//!
//! Spectral Sentinel has three cooperating layers:
//!
//! - **Spatial substrate** — a [`torrust_mudlark::GvGraph`] tracks volume
//!   over the coordinate domain and ranks spatial entries by accumulated
//!   observation volume.
//! - **Analysis selector** — [`AnalysisSet`] chooses significant V-Tree
//!   entries, then closes them under G-tree ancestry so every selected cell
//!   has a complete model chain back to the root.
//! - **Analysis engine** — per-cell subspace trackers score four raw axes:
//!   novelty, displacement, surprise, and coherence. A second coordination
//!   tier models cross-cell score patterns.
//!
//! **Feed-forward invariant.** Spectral Sentinel always observes the spatial
//! layer with `Delta = 1` per input value. Anomaly scores never flow back
//! into Mudlark's importance accounting. Spatial adaptation is driven by
//! observation volume only; the host controls temporal policy through
//! explicit decay operations.
//!
//! # Design Principle
//!
//! **Spectral Sentinel measures; the host decides.**
//!
//! Public report types carry raw statistical measurements: scores,
//! baselines, drift accumulators, maturity, geometry, structural summaries,
//! and health snapshots. They do not contain threat levels, recommended
//! actions, or policy decisions. The consuming application interprets the
//! measurements in its own domain.
//!
//! # Three-Surface Visibility Model
//!
//! Every public symbol belongs to one of three surfaces. Modules remain
//! crate-private and public types are re-exported flat from the crate root,
//! giving downstream users one canonical import path.
//!
//! - **Surface 1 — Readouts:** Lightweight data users inspect after an
//!   observation cycle: [`BatchReport`], [`CellReport`],
//!   [`CoordinationReport`], score distributions, baseline snapshots,
//!   structural and health summaries, [`AnalysisSet`], [`AnalysisEntry`],
//!   [`CentredBits`], and related report records.
//! - **Surface 2 — Engine:** Opaque operational API users configure and
//!   drive: [`SpectralSentinel`], [`SentinelConfig`], [`NoiseSchedule`],
//!   [`SvdStrategy`], [`Sentinel128`], [`Sentinel64`],
//!   [`CentredBitSource`], and [`GNodeId`] for subtree decay calls.
//! - **Surface 3 — Internals:** EWMA state, SVD update plumbing, tracker
//!   machinery, staging, CUSUM, and warming-thread implementation details.
//!
//! Exception: a few support types are re-exported as `#[doc(hidden)]` for
//! tests, diagnostics, and benchmarks. They are not part of the ordinary
//! downstream API surface.

// -- Surface 1 — Readouts (data users hold and inspect) -----------
//
// Modules are private; types are re-exported flat from the crate root
// so there is exactly one canonical path per public type.
pub(crate) mod analysis_set;
pub(crate) mod observation;
pub(crate) mod report;

// -- Surface 2 — Engine (opaque operational API) ------------------
pub(crate) mod config;
pub(crate) mod sentinel;

// -- Surface 3 — Internals (implementation machinery) -------------
//
// Crate-private. Downstream crates cannot import from these modules;
// all external access goes through the flat re-exports below and the
// public methods on SpectralSentinel.
pub(crate) mod ewma;
pub(crate) mod maths;

// README doc-tests: compile and run every code block in the README
// as part of `cargo test --doc`.
#[cfg(doctest)]
#[doc = include_str!("../README.md")]
mod _readme {}

#[cfg(test)]
mod tests;

// -- Public re-exports --------------------------------------------
//
// One canonical path per public type. Downstream code should use
// `torrust_sentinel::{SpectralSentinel, SentinelConfig, BatchReport, ...}`
// rather than reaching into submodules.
//
// Surface 1 — Readouts:
//   analysis_set: AnalysisEntry, AnalysisSet
//   observation:  CentredBits
//   report:       BatchReport, CellReport, CoordinationReport,
//                 AnomalyScores, ScoreDistribution, baseline/drift,
//                 maturity, geometry, contour, health, and summaries
//
// Surface 2 — Engine:
//   config:       ConfigError, ConfigErrors, ConfigWarning,
//                 NoiseSchedule, SentinelConfig
//   maths:        SvdStrategy
//   observation:  CentredBitSource
//   sentinel:     SpectralSentinel
//   aliases:      Sentinel128, Sentinel64
//   mudlark:      GNodeId for subtree-oriented operations
pub use analysis_set::{AnalysisEntry, AnalysisSet};
pub use config::{ConfigError, ConfigErrors, ConfigWarning, NoiseSchedule, SentinelConfig};
#[doc(hidden)]
pub use ewma::EwmaStats;
#[doc(hidden)]
pub use maths::SubspaceUpdate;
pub use maths::SvdStrategy;
pub use observation::{CentredBitSource, CentredBits};
#[doc(hidden)]
pub use report::TrackerReport;
pub use report::{
    AnalysisSetSummary, AnomalyScores, AxisBaselineSnapshots, BaselineSnapshot, BatchReport, CellInspection, CellReport,
    ClipPressureDistribution, ContourSnapshot, CoordinationHealth, CoordinationReport, CusumSnapshot, GeometryDistribution,
    HealthReport, MaturityDistribution, MemberScore, RankDistribution, SampleScore, ScoreDistribution, ScoringGeometry,
    TrackerMaturity,
};
pub use sentinel::SpectralSentinel;
// Re-export the G-V Graph node handle for use with `decay_subtree()`.
pub use torrust_mudlark::GNodeId;

/// 128-bit sentinel: full `u128` domain, `u64` counts, 128-bit width.
pub type Sentinel128 = SpectralSentinel<u128, u64, 128>;

/// 64-bit sentinel: `u64` domain, `u64` counts, 64-bit width.
pub type Sentinel64 = SpectralSentinel<u64, u64, 64>;

/// Minimum suffix width for a functional subspace tracker.
///
/// At `dim < MIN_TRACKER_DIM`, the tracker cannot form a
/// meaningful basis or compute residuals. Cells below this
/// threshold are excluded from the analysis set; a cell at it
/// is kept, which is what makes the value a minimum rather
/// than a floor the analysis set sits above.
///
/// The value 2 ensures at least one residual degree of freedom
/// ($d - k \geq 1$ when $k = 1$). A tracker with `dim = 1` can
/// technically run but produces identically-zero residuals (novelty)
/// since the single basis vector spans the entire space — making it
/// statistically useless.
///
/// See ADR-S-011 for rationale.
pub(crate) const MIN_TRACKER_DIM: usize = 2;

/// Widest coordinate width the observation path can carry.
///
/// A coordinate value reaches a tracker as a centred bit vector, and
/// [`CentredBits`] holds its values in a fixed array of this many slots.
/// The bridge that produces those vectors, [`CentredBitSource`], is open to
/// a coordinate type of any width, and the spatial layer asks only that `N`
/// fit the coordinate type — so a wider type carrying a wider `N` would
/// otherwise build an `N`-dimensional tracker fed from a vector that can
/// never hold more than this many values. The columns past the end of that
/// vector are not missing data the arithmetic would notice: they arrive as
/// zeros, which centred bits never are, so novelty, residual and rank would
/// all be computed over a constant the coordinate stream never produced.
/// Cell depth travels the same path in a single byte, which this ceiling
/// keeps honest as well.
pub(crate) const MAX_TRACKER_DIM: usize = 128;
