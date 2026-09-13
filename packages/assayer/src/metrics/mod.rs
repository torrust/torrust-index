// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Metrics export types and mappers: the package exposes values and renders
//! none of them (´dec:metrics:passive-export´).
//!
//! Provides the metric catalog ([`METRICS_CATALOG`]), the intermediate
//! [`MetricSample`] type, and two mapper functions that convert health
//! data structures into metric samples for external monitoring.
//!
//! The Assayer is a library crate — it does not own an HTTP endpoint
//! or a wire format. The host renders [`MetricSample`] vectors to
//! Prometheus text format, OTLP, `StatsD`, or any other system.
//!
//! # Usage
//!
//! ```ignore
//! use torrust_assayer::metrics::{health_summary_to_samples, full_report_to_samples};
//!
//! let summary = assayer.health_summary();
//! let samples = health_summary_to_samples(&summary);
//! // render `samples` to your monitoring system
//! ```
//!
//! # Cross-References
//!
//! - (´dec:metrics:neutral-sample´) — the neutral form values leave in, and
//!   why it commits to no exposition format
//! - (´dec:health:tiered-queries´) — the two health surfaces the mappers read
//! - (´dec:metrics:compile-time-catalogue´) — why the catalogue this module
//!   exposes cannot drift from what the code emits

mod catalog;
mod mapper;

pub use catalog::{METRICS_CATALOG, MetricDefinition, MetricSurface, MetricType};
pub use mapper::{MetricSample, full_report_to_samples, health_summary_to_samples, model_label, option_to_metric};
