// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Deterministic RNG for integration tests.
//!
//! Single, stable LCG algorithm — same constants as the one in
//! [`torrust_mudlark::testing`], chosen so that repeated runs and
//! parallel runs under `cargo test` give identical sequences.
//!
//! Tests must supply a seed explicitly via [`WorldBuilder::seed`](super::WorldBuilder::seed);
//! there is no "default" seed. Reproducibility is a contract.

/// Linear-congruential generator used for all integration-test
/// randomness.
///
/// Not cryptographically secure. Not suitable for statistical sampling
/// beyond what integration tests need (drawing indices, flipping
/// coins, jittering coordinates).
#[derive(Debug, Clone)]
pub struct TestRng {
    state: u64,
}

impl TestRng {
    /// Seed the RNG. The seed is the only input; identical seeds
    /// produce identical streams.
    #[must_use]
    pub const fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    /// Advance the state and return the raw 64-bit output.
    pub const fn next_u64(&mut self) -> u64 {
        self.state = self
            .state
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        self.state
    }

    /// Uniform f64 in `[0, 1)` with 53 bits of entropy.
    #[allow(clippy::cast_precision_loss)]
    pub fn next_f64(&mut self) -> f64 {
        (self.next_u64() >> 11) as f64 / (1u64 << 53) as f64
    }

    /// Uniform integer in `[0, n)`. Returns `0` when `n == 0`.
    pub const fn next_range(&mut self, n: u64) -> u64 {
        if n == 0 {
            return 0;
        }
        self.next_u64() % n
    }

    /// Fair coin flip.
    pub const fn coin(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }
}
