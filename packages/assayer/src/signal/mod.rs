// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Signal layer for feature encoding and caching.
//!
//! This module provides the signal layer infrastructure for the Assayer:
//!
//! - **`SignalShape`** — Encoding specifications for different signal types.
//! - **`SignalValue`** — Runtime signal values (numeric, categorical, vector).
//! - **`SignalDeclaration`** — Named signals with shape and persistence.
//! - **`SignalSchemaIndex`** — Feature space layout from declarations.
//! - **`SignalCache`** — LRU cache for entity-persistent features.
//!
//! # Architecture
//!
//! The signal layer sits between raw input signals and the Bayesian model:
//!
//! ```text
//! Input Signals → Schema Index → Encode → Cache → Feature Vector → Model
//! ```
//!
//! # Tests
//!
//! Signal layer tests are organised by visibility level (see AGENTS.md):
//!
//! - **Integration tests** (`tests/signal.rs`) — Public API: `encode_signal`,
//!   `SignalSchemaIndex`, `SignalCache`, `SignalCacheHealth`.
//! - **Crate tests** (`src/tests/signal_schema.rs`) — `pub(crate)` API:
//!   `SignalSchemaIndex::get`.
//!
//! # Cross-References
//!
//! - (´schema:signal:shapes´) — the declared shapes and what each encodes to
//! - (´dec:retention:cache-preencoded´) — why entity-persistent signals are
//!   cached already encoded
//! - (´dec:health:concrete-trackers´) — why the cache carries its own health
//!   counters

mod cache;
mod schema;
mod value;

// Re-exports for crate-internal use and integration tests.
// Items are `pub` (not `pub(crate)`) to allow re-export from the crate root,
// but remain hidden from the public API because this module is private.
pub use cache::{SignalCache, SignalCacheHealth};
pub use schema::{SignalSchemaIndex, expand_signal_classes};
#[allow(unused_imports)]
pub use value::{Persistence, SignalDeclaration, SignalShape, SignalValue, encode_signal, shape_matches};
