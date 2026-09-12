// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Public API surface modules.
//!
//! - [`builder`] — `AssayerBuilder` for constructing an `Assayer`
//! - [`health`] — `Assayer::health_summary()`, `full_health_report()`, `drain_health_events()`
//! - [`label`] — `Assayer::label()` and `LabelAck`
//! - [`assess`] — `Assayer::assess()`
//! - [`guidance`] — `Assayer::request_labels()`
//! - [`report`] — `Assayer::receive_sentinel_report()`

mod assess;
mod builder;
mod guidance;
mod health;
mod label;
mod lifecycle;
mod preseed;
mod report;

pub use builder::AssayerBuilder;
pub use label::LabelAck;
pub use preseed::{PreSeedEntry, PreSeedResult};
pub use report::ReportAck;
