// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

// ── Test RNG Implementations ────────────────────────────────────

use crate::traits::Rng;

/// Simple LCG for reproducible pseudo-random sequences in tests.
pub struct TestLcgRng(pub u64);

impl Rng for TestLcgRng {
    #[allow(clippy::cast_precision_loss)] // intentional: top 53 bits → f64
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Deterministic RNG that returns a fixed value.
pub struct FixedRng(pub f64);

impl Rng for FixedRng {
    fn next_f64(&mut self) -> f64 {
        self.0
    }
}

/// Deterministic RNG that cycles through a sequence of values.
pub struct SeqRng {
    values: Vec<f64>,
    idx: usize,
}

impl SeqRng {
    #[must_use]
    pub const fn new(values: Vec<f64>) -> Self {
        Self { values, idx: 0 }
    }
}

impl Rng for SeqRng {
    fn next_f64(&mut self) -> f64 {
        let v = self.values[self.idx % self.values.len()];
        self.idx += 1;
        v
    }
}
