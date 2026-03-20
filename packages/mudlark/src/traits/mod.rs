// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Chemistry contracts and instrument traits for [`GvGraph`](crate::GvGraph).
//!
//! # Summary
//!
//! This module is **Surface 2 — Film** in the three-surface visibility
//! model ([ADR-M-032]).  It defines the trait bounds that constrain what
//! coordinate spaces (`C`), intensity types (`V`), and observation
//! types are compatible with `GvGraph<C, V, N>`.
//!
//! Blanket implementations cover all standard integer (`u8`–`u128`)
//! and float (`f32`, `f64`) primitives — **most users never implement
//! these traits**, they just pick a concrete type like `u64` and all
//! capabilities are available immediately.
//!
//! # Analogy
//!
//! The trait organisation mirrors the physics of **silver halide
//! film** that is continuously exposed and regressing — never
//! developed, never fixed (see [ADR-M-032] for the full analogy and its
//! explicit limits).
//!
//! The analogy is **load-bearing** for understanding *why* some types
//! are public and others are not.  [`GvGraph`](crate::GvGraph) is the
//! unfixed negative — opaque, actively changing.  Users cannot reach
//! into the emulsion directly; they interact only through
//! **chemistry contracts** (what emulsion and photon types are
//! compatible) and **instruments** (the metering and exposure tools
//! loaded alongside the film).  Everything behind the camera back —
//! arenas, tree nodes, rebalancing — is the emulsion (Surface 3,
//! `pub(crate)`).
//!
//! # Chemistry contracts
//!
//! These traits define what goes *into* the negative and how it
//! combines:
//!
//! | Trait | Silver halide analog | Purpose |
//! |-------|---------------------|---------|
//! | [`Coordinate`] | Film plane geometry | Position type for the spatial domain `[0, 2^N)` |
//! | [`Accumulator`] | Emulsion chemistry (`AgBr`, `AgCl`, `AgI`) | Intensity type: `zero` + `add` + `sub` — different crystals, different combination rules |
//! | [`Attenuatable`] | Thermal sensitivity | How a crystal's latent image responds to scaling — regression, intensification, or annihilation |
//! | [`Weighable`] | Developability measure | Scalar weight projection (`f64`) for proportional sampling |
//! | [`Proratable`] | Partial-grain densitometry | Fractional subdivision for range-query overlap |
//! | [`Inspectable`] | Lab densitometer calibration | Diagnostic `f64` projection for logging and invariant checks |
//! | [`Observation`] | Photon characteristics (wavelength, energy) | How a single event affects the accumulator |
//! | [`ScalableObservation`] | Photon attenuation characteristics | How an observation can scale (multiply) the accumulator — cross-type only |
//! | [`Rng`] | Developer diffusion — thermal Brownian motion | Randomness source for proportional sampling |
//!
//! # Instruments
//!
//! Four traits partition the [`GvGraph`](crate::GvGraph) API by
//! permission level — the metering and exposure tools the user holds:
//!
//! | Trait | Silver halide analog | Permission | Methods |
//! |-------|---------------------|------------|---------|
//! | [`SpatialRead`] | Densitometer | `&self` | `plateaus()`, `get()` |
//! | [`SpatialWrite`] | Incident light (exposure) | `&mut self` | `observe()` |
//! | [`TemporalDecay`] | Thermal regressor | `&mut self` | `decay()` |
//! | [`WeightedSampler`] | Random grain probe | `&self` | `sample()` |
//!
//! # Accumulator capability tree
//!
//! [`Accumulator`] is the core bound on `V`.  Sub-traits extend it
//! with independent, opt-in capabilities — each unlocks specific
//! [`GvGraph`](crate::GvGraph) operations (§IDEA M-8.8):
//!
//! ```text
//! Accumulator (core: zero + add + sub)
//! ├── Attenuatable  → decay()
//! ├── Weighable     → sample(), Transition::snr()
//! ├── Proratable    → range_sum(), Pewei::reconstruct()
//! └── Inspectable   → observe(), extract(), layers()
//! ```
//!
//! > **Note (implementation stricture):** The spec (§IDEA M-8.8.1)
//! > considers `sub` an independent opt-in capability — the core
//! > observation/split/rebalance cycle needs only `zero` + `add`.
//! > This implementation bundles `sub` into `Accumulator` for
//! > ergonomics: PEWEI refinement and range queries use subtraction
//! > pervasively, and splitting it out would force an additional
//! > bound on most public API methods with little practical benefit
//! > (every built-in type supports it).  Custom accumulator types
//! > that cannot subtract can provide a panicking stub.
//!
//! # Which traits do I need?
//!
//! All built-in types (`u8`–`u128`, `f32`, `f64`) implement every
//! sub-trait, so if you use one of those you get all operations for
//! free.  For **custom accumulator types**, implement only what you
//! need:
//!
//! | I want to…                                       | Required bounds on `V`                    |
//! |--------------------------------------------------|-------------------------------------------|
//! | `observe()` + `get()` + `extract()` + `layers()` | [`Inspectable`] (implies [`Accumulator`]) |
//! | …plus `decay()`                                  | + [`Attenuatable`]                        |
//! | …plus `sample()`                                 | + [`Weighable`]                           |
//! | …plus `range_sum()` / `reconstruct()`            | + [`Proratable`]                          |
//! | All operations                                   | All four (any built-in type satisfies all) |
//!
//! Only [`Rng`] and [`Observation`] are designed for user extension.
//!
//! # Examples
//!
//! A single workflow exercising every instrument trait — observe,
//! read, sample, decay, range query, and extract:
//!
//! ```
//! use torrust_mudlark::{Config, GvGraph, Rng};
//!
//! let config = Config {
//!     split_threshold: 10u64,
//!     depth_create: 2,
//!     depth_evict: 5,
//!     budget: Some(128),
//!     alpha_relax: 0.5,
//!     bounded_eviction: true,
//! };
//! let mut g = GvGraph::<u64, u64, 16>::new(config);
//!
//! // SpatialWrite::observe — photon strike on the emulsion.
//! g.observe(100, 5u64);
//! g.observe(200, 3u64);
//!
//! // SpatialRead::get — spot densitometer reading.
//! let cell = g.get(100);
//! assert!(cell.intensity >= 5);
//!
//! // SpatialRead::plateaus — isodensity contour map.
//! let plateaus = g.plateaus();
//! assert!(!plateaus.is_empty());
//!
//! // Proratable: range_sum over the full domain.
//! let total = g.range_sum(..);
//! assert_eq!(total, 8);
//!
//! // WeightedSampler::sample — random grain probe.
//! struct DemoRng(u64);
//! impl Rng for DemoRng {
//!     fn next_f64(&mut self) -> f64 {
//!         self.0 = self.0.wrapping_mul(6364136223846793005).wrapping_add(1);
//!         (self.0 >> 11) as f64 / (1u64 << 53) as f64
//!     }
//! }
//! let mut rng = DemoRng(42);
//! let sampled = g.sample(&mut rng).unwrap();
//! assert!(sampled.intensity > 0);
//!
//! // TemporalDecay::decay — thermal regression at 50%.
//! let root = g.g_root();
//! g.decay(root, 0.5, 0.0);
//! assert!(g.total_sum() < 8);
//!
//! // Inspectable powers extract() — contact print from the negative.
//! let pewei = g.extract();
//! assert!(pewei.layer_count() > 0);
//! ```
//!
//! # References
//!
//! - [ADR-M-032] — Three-surface visibility model
//!
//! [ADR-M-032]: https://github.com/torrust/torrust-index/blob/develop/packages/mudlark/adr/032-three-surface-model.md

// ── Chemistry contracts ─────────────────────────────────────────────

mod coordinate;
pub use coordinate::Coordinate;

mod accumulator;
pub use accumulator::Accumulator;

mod attenuatable;
pub use attenuatable::Attenuatable;

mod weighable;
pub use weighable::Weighable;

mod proratable;
pub use proratable::Proratable;

mod inspectable;
pub use inspectable::Inspectable;

mod observation;
pub use observation::{Observation, ScalableObservation};

mod rng;
pub use rng::Rng;

// ── Instruments ─────────────────────────────────────────────────────

mod spatial_read;
pub use spatial_read::SpatialRead;

mod spatial_write;
pub use spatial_write::SpatialWrite;

mod temporal_decay;
pub use temporal_decay::TemporalDecay;

mod weighted_sampler;
pub use weighted_sampler::WeightedSampler;

#[cfg(test)]
mod tests {}
