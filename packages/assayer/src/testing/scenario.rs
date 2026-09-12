// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Scenario scaffolding: a world and its tracing span, built together.
//!
//! A scenario test wants two things at once — a [`World`] under a known
//! instance id and seed, and a tracing span that names the test so its output
//! is attributable and its duration visible. Binding them into one value means
//! a test opens with a single statement and neither half can be forgotten:
//! a span dropped early stops labelling the very work it was opened for.
//!
//! The subscriber installed here writes through the test writer, so `cargo
//! test` captures the output and shows it only on failure. Installing it is
//! idempotent — the first scenario in a binary wins and the rest are no-ops.

use tracing::span::EnteredSpan;

use super::test_infrastructure;
use super::world::{World, WorldBuildError, WorldBuilder};
use crate::config::types::AssayerConfig;

/// Initialise a `tracing` subscriber scoped to the current test.
///
/// Respects `RUST_LOG` for filtering (default: no output unless set).
/// Uses `.with_test_writer()` so output is captured by `cargo test`
/// and only shown on failure (or with `--nocapture`).
///
/// Span close events include elapsed time; with `RUST_LOG=info` every
/// test prints its wall-clock duration on close, making slow tests
/// easy to spot.
///
/// **Must** bind the return value to keep the span alive for the
/// duration of the test: `let _t = init_tracing();`
pub fn init_tracing() -> EnteredSpan {
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

/// Bundle of test scaffolding returned by [`scenario`].
///
/// Holds the [`World`] and the tracing span guard together so a
/// single `let s = scenario(...);` keeps both alive for the test's
/// lifetime. Field access (`s.world`) replaces the
/// previous two-line `let _t = init_tracing(); let world = ...;`
/// preamble.
pub struct Scenario {
    /// The `World` under test.
    ///
    /// Declared first so it is dropped first: Rust drops struct fields in
    /// declaration order, and dropping the `World` is what shuts the
    /// `Assayer` down and joins its threads. That work has to happen inside
    /// the span, which is why the span guard is the last field.
    pub world: World,
    /// Tracing span, declared last so it is dropped last and the span is
    /// still open while the `World` above it shuts down. The span's close
    /// event therefore times the scenario's whole lifetime, shutdown
    /// included, rather than stopping at the last statement of the test body.
    _tracing: EnteredSpan,
}

impl std::ops::Deref for Scenario {
    type Target = World;

    fn deref(&self) -> &Self::Target {
        &self.world
    }
}

impl std::ops::DerefMut for Scenario {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.world
    }
}

/// Build a cold [`Scenario`] under the given instance id and seed.
///
/// Equivalent to:
///
/// ```text
/// let _t = init_tracing();
/// let world = World::cold(instance_id, seed);
/// ```
///
/// but bundles the two so the test body reads as a single statement.
/// Tests that need a non-default channel set should use
/// [`scenario_with`].
#[must_use]
pub fn scenario(instance_id: &str, seed: u64) -> Scenario {
    let tracing = init_tracing();
    let world = World::cold(instance_id, seed);
    Scenario {
        world,
        _tracing: tracing,
    }
}

/// Build a configured [`Scenario`] under the given instance id and seed.
///
/// This keeps the same "tracing span + world" lifetime bundling as
/// [`scenario`] while allowing callers to customise the underlying
/// [`WorldBuilder`] before it is built.
///
/// The starting builder has only the instance id and seed applied;
/// callers remain responsible for declaring at least one channel.
pub fn scenario_with(
    instance_id: &str,
    seed: u64,
    configure: impl FnOnce(WorldBuilder) -> WorldBuilder,
) -> Result<Scenario, WorldBuildError> {
    scenario_with_config(
        AssayerConfig {
            instance_id: instance_id.to_owned(),
            // The capacity is host-set with no default; the suite
            // declares the harness's fixture value.
            infrastructure: test_infrastructure(),
            ..Default::default()
        },
        seed,
        configure,
    )
}

/// Build a configured [`Scenario`] from an explicit [`AssayerConfig`].
///
/// Most tests should use [`scenario_with`]. This variant exists for scenarios
/// that need to tune fixture-level configuration while keeping the same tracing
/// span and seeded-world construction path.
pub fn scenario_with_config(
    config: AssayerConfig,
    seed: u64,
    configure: impl FnOnce(WorldBuilder) -> WorldBuilder,
) -> Result<Scenario, WorldBuildError> {
    let tracing = init_tracing();
    let builder = World::builder(config).seed(seed);
    let world = configure(builder).build()?;
    Ok(Scenario {
        world,
        _tracing: tracing,
    })
}
