// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Sentinel constructors and scenario builders for tests.

use torrust_sentinel::{BatchReport, Sentinel128, SentinelConfig};

use super::config::{integration_config, test_config};
use super::generators::cell_values;

/// Pre-populate a sentinel with two distinct value ranges so the graph
/// creates cells and the analysis set is non-trivial.
pub fn seeded_sentinel() -> Sentinel128 {
    let mut s = Sentinel128::new(test_config()).unwrap();
    // Two values with well-separated leading bits → distinct cells.
    s.ingest(&[
        0xF000_0000_0000_0000_0000_0000_0000_0001,
        0x1000_0000_0000_0000_0000_0000_0000_0002,
    ]);
    s
}

// ─── §9.1.1 — ScenarioBuilder ──────────────────────────────

/// Builder for common test scenarios.
///
/// Constructs a [`Sentinel128`] seeded with traffic across one
/// or more leading-nibble ranges and optionally warmed through
/// multiple batches.
pub struct ScenarioBuilder {
    config: SentinelConfig<u64>,
    seed_ranges: Vec<(u128, usize)>,
    warm_batches: usize,
}

impl ScenarioBuilder {
    pub fn new() -> Self {
        Self {
            config: integration_config(),
            seed_ranges: Vec::new(),
            warm_batches: 0,
        }
    }

    pub fn config(mut self, cfg: SentinelConfig<u64>) -> Self {
        self.config = cfg;
        self
    }

    /// Add a seed range: feed `count` sequential values with the
    /// given leading `nibble` during initial seeding.
    pub fn seed_range(mut self, nibble: u128, count: usize) -> Self {
        self.seed_ranges.push((nibble, count));
        self
    }

    /// Number of warm-up batches to run after seeding.
    pub const fn warm_batches(mut self, n: usize) -> Self {
        self.warm_batches = n;
        self
    }

    /// Build a sentinel that has been seeded and warmed.
    pub fn build(self) -> Sentinel128 {
        self.build_with_reports().0
    }

    /// Build and return both the sentinel and the warm-up reports.
    pub fn build_with_reports(self) -> (Sentinel128, Vec<BatchReport<u128>>) {
        let batch = self.make_batch();
        let mut s = Sentinel128::new(self.config).unwrap();
        let mut reports = Vec::new();

        if !batch.is_empty() {
            // Seed phase.
            reports.push(s.ingest(&batch));

            // Warm-up phase.
            for _ in 0..self.warm_batches {
                reports.push(s.ingest(&batch));
            }
        }

        (s, reports)
    }

    /// Flatten all seed ranges into a single batch of values.
    fn make_batch(&self) -> Vec<u128> {
        self.seed_ranges
            .iter()
            .flat_map(|&(nibble, count)| cell_values(nibble, count))
            .collect()
    }
}
