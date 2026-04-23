// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

pub mod arena;
pub mod config_validation;
pub mod decay;
#[cfg(feature = "dynamic-contour-tracking")]
pub mod decay_f64_depth;
pub mod decompose_basis;
pub mod diagnostic;
pub mod evict;
pub mod gnode;
pub mod graph;
pub mod graph_init;
pub mod gtree;
pub mod handle;
pub mod invariant_self;
pub mod observe;
pub mod pewei;
pub mod plan_builders;
pub mod plateau;
pub mod rebalance;
pub mod rebalance_stress;
#[cfg(feature = "dynamic-contour-tracking")]
pub mod semi_internal_plateau;
pub mod spiked_vtree;
pub mod split;
pub mod structural;
pub mod traits;
pub mod view;
pub mod vnode;
pub mod vtree;
pub mod worked_example;

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
