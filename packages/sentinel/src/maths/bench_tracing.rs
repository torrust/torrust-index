// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Lightweight tracing layer that accumulates per-span-name wall-clock
//! durations.  Used by convergence benchmarks to capture the cost of
//! each SVD strategy without `Instant`-based plumbing in the hot path.
//!
//! # Usage
//!
//! ```ignore
//! let (timing, _guard) = SpanTiming::install();
//! // ... run observe() calls ...
//! let naive_ns = timing.total_ns("svd_naive");
//! let brand_ns = timing.total_ns("svd_brand");
//! ```
//!
//! The example remains ignored because this benchmark-only module is private and compiled only with the crate's tests, so an external doctest cannot name `SpanTiming`.
//!
//! The layer stores enter/exit timestamps per span instance in an
//! `RwLock<HashMap>` and aggregates into cumulative nanoseconds on
//! close.  It is designed for single-threaded benchmark use — the
//! `RwLock` is uncontended.
//!
//! # §-references
//!
//! - ADR-S-016 — Brand's incremental SVD
//! - ADR-M-028 — Span-native tracing

use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::Instant;

use tracing::{Subscriber, span};
use tracing_subscriber::Layer;
use tracing_subscriber::layer::Context;
use tracing_subscriber::prelude::*;
use tracing_subscriber::registry::LookupSpan;

// ════════════════════════════════════════════════════════════
//  Per-span storage (attached via Extensions)
// ════════════════════════════════════════════════════════════

struct SpanEnterTime(Instant);

// ════════════════════════════════════════════════════════════
//  Accumulated timing map
// ════════════════════════════════════════════════════════════

/// Shared timing accumulator.
///
/// Keys are span names (e.g. `"svd_naive"`, `"svd_brand"`,
/// `"phase2_evolve_subspace"`).  Values are cumulative nanoseconds
/// spent inside spans of that name, and the call count.
#[derive(Debug, Clone, Default)]
pub struct SpanTiming {
    inner: Arc<RwLock<TimingMap>>,
}

#[derive(Debug, Default)]
struct TimingMap {
    entries: HashMap<&'static str, TimingEntry>,
}

#[derive(Debug, Default, Clone, Copy)]
struct TimingEntry {
    total_ns: u128,
    count: u64,
}

impl SpanTiming {
    /// Install a new timing layer as the thread-local default subscriber
    /// and return the handle for reading accumulated durations.
    ///
    /// The returned `DefaultGuard` must be kept alive for the duration
    /// of the measurement.  Dropping it uninstalls the subscriber.
    pub fn install() -> (Self, tracing::subscriber::DefaultGuard) {
        let timing = Self::default();
        let layer = SpanTimingLayer {
            timing: timing.inner.clone(),
        };
        // Respect RUST_LOG, defaulting to INFO.  This means
        // `tracing::enabled!(Level::DEBUG)` is false unless the
        // user sets `RUST_LOG=debug` — matching the mudlark
        // convention and preventing the oracle from firing
        // accidentally in release-mode benchmarks.
        let env_filter = tracing_subscriber::EnvFilter::builder()
            .with_default_directive(tracing::Level::INFO.into())
            .from_env_lossy();
        let subscriber = tracing_subscriber::registry().with(env_filter).with(layer);
        let guard = tracing::subscriber::set_default(subscriber);
        (timing, guard)
    }

    /// Cumulative nanoseconds spent inside spans named `name`.
    pub fn total_ns(&self, name: &str) -> u128 {
        self.inner.read().unwrap().entries.get(name).map_or(0, |e| e.total_ns)
    }

    /// Number of times a span named `name` was entered.
    #[allow(dead_code)]
    pub fn call_count(&self, name: &str) -> u64 {
        self.inner.read().unwrap().entries.get(name).map_or(0, |e| e.count)
    }

    /// Reset all accumulated timing.
    pub fn reset(&self) {
        self.inner.write().unwrap().entries.clear();
    }
}

// ════════════════════════════════════════════════════════════
//  Tracing Layer implementation
// ════════════════════════════════════════════════════════════

struct SpanTimingLayer {
    timing: Arc<RwLock<TimingMap>>,
}

impl<S> Layer<S> for SpanTimingLayer
where
    S: Subscriber + for<'a> LookupSpan<'a>,
{
    fn on_enter(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let mut extensions = span.extensions_mut();
            extensions.insert(SpanEnterTime(Instant::now()));
        }
    }

    // The RwLockWriteGuard (`map`) must live as long as `entry` borrows it;
    // there is no earlier drop point.
    #[allow(clippy::significant_drop_tightening)]
    fn on_exit(&self, id: &span::Id, ctx: Context<'_, S>) {
        if let Some(span) = ctx.span(id) {
            let elapsed = {
                let extensions = span.extensions();
                let ns = extensions.get::<SpanEnterTime>().map(|t| t.0.elapsed().as_nanos());
                drop(extensions); // release read-lock before acquiring write-lock
                ns
            };
            if let Some(ns) = elapsed {
                let name = span.name();
                {
                    let mut map = self.timing.write().unwrap();
                    let entry = map.entries.entry(name).or_default();
                    entry.total_ns += ns;
                    entry.count += 1;
                }
            }
        }
    }
}
