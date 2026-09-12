// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Fluent builder for [`DegradationContext`] fixtures.
//!
//! `DegradationContext` is `#[non_exhaustive]` and aggregates seven
//! independent degradation signals (signals sanitised, shape mismatches,
//! unknown signals, NaN sentinels, degraded report sentinels,
//! sanitised features, degraded models). Tests usually want to flip just one knob; this
//! builder fills the rest with the clean defaults so a test can say
//!
//! ```ignore
//! let ctx = DegradationSpec::new().signals_sanitised(1).build();
//! assert!(ctx.is_degraded());
//! ```
//!
//! without restating the other fields.

use crate::health::DegradationContext;
use crate::types::{ModelId, SentinelId};

/// Fluent builder for [`DegradationContext`].
#[derive(Clone, Debug, Default)]
pub struct DegradationSpec {
    /// Underlying context, mutated by the builder methods.
    ctx: DegradationContext,
}

impl DegradationSpec {
    /// Starts with the clean default context.
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    /// Sets [`DegradationContext::signals_sanitised`] (CP1 counter).
    #[must_use]
    pub const fn signals_sanitised(mut self, n: u32) -> Self {
        self.ctx.signals_sanitised = n;
        self
    }

    /// Sets [`DegradationContext::signals_shape_mismatched`] — the counter
    /// for values the boundary zero-fills rather than rejects
    /// (´dec:surface:sanitise-not-reject´).
    #[must_use]
    pub const fn signals_shape_mismatched(mut self, n: u32) -> Self {
        self.ctx.signals_shape_mismatched = n;
        self
    }

    /// Sets [`DegradationContext::signals_unknown`] (CP1 counter).
    #[must_use]
    pub const fn signals_unknown(mut self, n: u32) -> Self {
        self.ctx.signals_unknown = n;
        self
    }

    /// Populates [`DegradationContext::nan_sentinels`] from raw u32 IDs.
    #[must_use]
    pub fn nan_sentinels(mut self, ids: &[u32]) -> Self {
        self.ctx.nan_sentinels = ids.iter().copied().map(SentinelId).collect();
        self
    }

    /// Populates [`DegradationContext::degraded_report_sentinels`] from
    /// raw u32 IDs.
    #[must_use]
    pub fn degraded_report_sentinels(mut self, ids: &[u32]) -> Self {
        self.ctx.degraded_report_sentinels = ids.iter().copied().map(SentinelId).collect();
        self
    }

    /// Sets [`DegradationContext::features_sanitised`] (CP3 counter).
    #[must_use]
    pub const fn features_sanitised(mut self, n: u32) -> Self {
        self.ctx.features_sanitised = n;
        self
    }

    /// Populates [`DegradationContext::degraded_models`] (CP4 fallbacks).
    #[must_use]
    pub fn degraded_models<I>(mut self, models: I) -> Self
    where
        I: IntoIterator<Item = ModelId>,
    {
        self.ctx.degraded_models = models.into_iter().collect();
        self
    }

    /// Finalises the builder into a [`DegradationContext`].
    #[must_use]
    pub fn build(self) -> DegradationContext {
        self.ctx
    }
}
