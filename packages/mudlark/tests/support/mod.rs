// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

#![allow(dead_code, unused_imports)]
//! Shared helpers for integration tests.
//!
//! Include with `mod support;` in each integration test file that
//! needs these utilities.

// ── Tracing ─────────────────────────────────────────────────────

/// Init tracing subscriber and return an INFO-level span named after
/// the current test.
///
/// Respects `RUST_LOG` for filtering (default: no output unless set).
/// Uses `.with_test_writer()` so output is captured by `cargo test`
/// and only shown on failure (or with `--nocapture`).
///
/// Span close events include elapsed time.  With `RUST_LOG=info` (or
/// higher) every test prints its wall-clock duration on close, making
/// it easy to spot slow tests:
///
/// ```text
/// INFO test{name=budget_200_sustained_load}: close time.busy=412ms
/// ```
///
/// **Must** bind the return value to keep the span alive for the
/// duration of the test: `let _t = init_tracing();`
pub fn init_tracing() -> tracing::span::EnteredSpan {
    use tracing_subscriber::fmt::format::FmtSpan;

    drop(
        tracing_subscriber::fmt()
            .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
            .with_span_events(FmtSpan::CLOSE)
            .with_test_writer()
            .try_init(),
    );

    let test_name = std::thread::current().name().unwrap_or("unknown").to_string();
    tracing::info_span!("test", name = %test_name).entered()
}

// ── RNG ─────────────────────────────────────────────────────────

/// Release-mode LCG for integration tests.
///
/// Same constants as `testing::TestLcgRng` but lives outside the
/// `#[cfg(debug_assertions)]` gate so it compiles in release mode.
pub struct TestRng(pub u64);

impl torrust_mudlark::Rng for TestRng {
    #[allow(clippy::cast_precision_loss)]
    fn next_f64(&mut self) -> f64 {
        self.0 = self
            .0
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        (self.0 >> 11) as f64 / (1u64 << 53) as f64
    }
}

/// Deterministic RNG that always returns a fixed value.
pub struct FixedRng(pub f64);

impl torrust_mudlark::Rng for FixedRng {
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

impl torrust_mudlark::Rng for SeqRng {
    fn next_f64(&mut self) -> f64 {
        let v = self.values[self.idx % self.values.len()];
        self.idx += 1;
        v
    }
}

/// Serialize any `serde::Serialize` value to a JSON string.
#[cfg(feature = "serde")]
pub fn to_json<T: serde::Serialize>(value: &T) -> String {
    serde_json::to_string(value).expect("serialization should not fail")
}

/// Deserialize a JSON string into any `serde::de::DeserializeOwned` value.
#[cfg(feature = "serde")]
pub fn from_json<T: serde::de::DeserializeOwned>(json: &str) -> T {
    serde_json::from_str(json).expect("deserialization failed")
}
