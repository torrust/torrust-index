// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

use torrust_mudlark::Rng;

/// Release-mode LCG for benchmarks.
///
/// Same constants as `testing::TestLcgRng` but lives outside the
/// `#[cfg(debug_assertions)]` gate so it compiles in release mode.
#[allow(dead_code)]
pub struct BenchRng(pub u64);

impl Rng for BenchRng {
    #[allow(clippy::cast_precision_loss)]
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}
