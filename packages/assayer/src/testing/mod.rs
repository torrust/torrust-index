// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Test infrastructure: the integration-test harness.
//!
//! This module is **public (but `#[doc(hidden)]`)** so that both crate
//! tests (`src/tests/`) and integration tests (`tests/`) share the
//! same helpers. What it exposes is the public surface wherever the public
//! surface can express it; where a scenario needs a guarantee the engine
//! deliberately declines to give a host — a barrier the asynchronous halves
//! can be read across, a parked steward, the published snapshot read directly
//! rather than through a report — the harness reaches for the crate-internal
//! machinery and hands the scenario the guarantee instead. That is the
//! harness's job: a test that reconstructed those waits for itself would be
//! measuring the scheduler.
//!
//! # Shape
//!
//! The harness is organised around a single top-level context — the
//! [`World`] — that owns an [`Assayer`](crate::Assayer) instance plus
//! the auxiliary state a test needs to speak in names rather than IDs:
//!
//! ```text
//! World
//! ├── Assayer          (the real engine — no mocks)
//! ├── VirtualClock     (deterministic time; see Staging below)
//! ├── TestRng          (seeded LCG, reproducible)
//! ├── NameRegistry     ("S1" → SentinelId, "default" → ChannelId, …)
//! └── Tolerances       (named epsilons used by asserts)
//! ```
//!
//! The deterministic-time boundary is (´dec:clock:two-domains´); names replace
//! raw identifiers throughout the harness.
//!
//! # Vocabulary
//!
//! The verb set is deliberately small and maps 1:1 to English
//! sentences in `tests/README.md`:
//!
//! | README phrase                          | Harness verb                             |
//! |----------------------------------------|------------------------------------------|
//! | "register a Sentinel"                  | [`World::register_sentinel`]             |
//! | "deregister Sentinel A"                | [`World::deregister_sentinel`]           |
//! | "ingest a Sentinel report"             | [`World::receive_report`]                |
//! | "process an assessment"                | `World::assess` *(Stage 3)*              |
//! | "inject 50 labels"                     | `world.inject(LabelTape::…)` *(Stage 3)* |
//! | "advance 29 days"                      | `world.advance(days(29))` *(Stage 2)*    |
//!
//! # Staging
//!
//! The harness lands in three stages (see `tests/README.md` vision):
//!
//! 1. **Stage 1 — Skeleton.** Module layout, name registry, seeded
//!    RNG, tolerances, clock trait, a [`World`] that wraps today's
//!    public API. No time travel yet.
//! 2. **Stage 2, monotonic — Clock injection** *(landed).* The engine
//!    takes its clock from [`crate::api::AssayerBuilder`], and the
//!    monotonic readings behind report staleness and the maintenance
//!    thread's decay interval come from it. Staleness no longer
//!    depends on how busy the machine is.
//! 3. **Stage 2, persistent — Clock injection** *(landed).* The live
//!    Companion tracker's construction, updates, and reads take the
//!    persistent present from the [`World`] clock. Read-time decay no
//!    longer depends on when the suite runs.
//! 4. **Stage 3 — Tapes and fixtures.** `LabelTape`, `ReportTape`,
//!    `World::converged()` factory, and the full scenario port.
//!
//! # Sealing
//!
//! Only the clock crosses into a sealed (featureless) build: production reads
//! its time through [`Clock`], so that submodule compiles unconditionally while
//! everything else — the harness proper — is gated on the same condition that
//! decides the module's visibility. A gate on visibility alone would leave the
//! whole harness compiled and crate-private in a host build, which is to say
//! dead, and the module used to answer for that with a blanket `dead_code`
//! allowance. Gating what compiles instead means there is nothing dead to
//! silence, and a helper that genuinely loses its last caller is reported.

// Documenting every harness helper is not what the harness is for; the four
// documentation lints the crate warns on are answered here, module-wide, and
// nothing else is.

mod clock;

#[cfg(any(test, feature = "test-support"))]
mod virtual_clock;

#[cfg(any(test, feature = "test-support"))]
mod asserts;
#[cfg(any(test, feature = "test-support"))]
mod construction;
#[cfg(any(test, feature = "test-support"))]
mod degradation_spec;
#[cfg(any(test, feature = "test-support"))]
pub mod dimensions;
#[cfg(any(test, feature = "test-support"))]
mod drift;
#[cfg(any(test, feature = "test-support"))]
mod label_spec;
#[cfg(any(test, feature = "test-support"))]
mod liveness;
#[cfg(any(test, feature = "test-support"))]
pub mod maintenance;
#[cfg(any(test, feature = "test-support"))]
mod names;
#[cfg(any(test, feature = "test-support"))]
mod preseed_spec;
#[cfg(any(test, feature = "test-support"))]
pub mod registrations;
#[cfg(any(test, feature = "test-support"))]
mod reports;
#[cfg(any(test, feature = "test-support"))]
mod rng;
#[cfg(any(test, feature = "test-support"))]
mod scenario;
#[cfg(any(test, feature = "test-support"))]
pub mod signals;
#[cfg(any(test, feature = "test-support"))]
mod source_scan;
#[cfg(any(test, feature = "test-support"))]
mod tolerances;
#[cfg(any(test, feature = "test-support"))]
mod world;

#[cfg(any(test, feature = "test-support"))]
pub use self::asserts::*;
pub use self::clock::{Clock, SystemClock};
#[cfg(any(test, feature = "test-support"))]
pub use self::construction::{TEST_COMMAND_CHANNEL_CAPACITY, test_infrastructure};
#[cfg(any(test, feature = "test-support"))]
pub use self::degradation_spec::DegradationSpec;
#[cfg(any(test, feature = "test-support"))]
pub use self::drift::{DIVERGENT_OUTCOME_REPLAY_DRIFT, LIFECYCLE_SUFFICIENCY_DRIFT, PURITY_DRIFT};
#[cfg(any(test, feature = "test-support"))]
pub use self::label_spec::LabelSpec;
#[cfg(any(test, feature = "test-support"))]
pub use self::liveness::{
    ACK_DEADLINE, Progress, STATE_DEADLINE, advanced, fail_fast_on_model_owner_panic, finished, watch_progress,
};
#[cfg(any(test, feature = "test-support"))]
pub use self::names::{AxisName, ChannelName, IdentityName, SentinelName};
#[cfg(any(test, feature = "test-support"))]
pub use self::preseed_spec::PreSeedSpec;
#[cfg(any(test, feature = "test-support"))]
pub use self::reports::*;
#[cfg(any(test, feature = "test-support"))]
pub use self::rng::TestRng;
#[cfg(any(test, feature = "test-support"))]
pub use self::scenario::{Scenario, init_tracing, scenario, scenario_with, scenario_with_config};
#[cfg(any(test, feature = "test-support"))]
pub use self::signals::{
    score_verified_request_schema, with_degraded_score_verified, with_score_verified, with_standard_final_score_verified,
    with_standard_training_score_verified,
};
#[cfg(any(test, feature = "test-support"))]
pub use self::source_scan::{LineVisit, SourceTreeScanner, manifest_dir};
#[cfg(any(test, feature = "test-support"))]
pub use self::tolerances::{DEFAULT_TOLERANCES, Tolerances};
#[cfg(any(test, feature = "test-support"))]
pub use self::virtual_clock::VirtualClock;
#[cfg(any(test, feature = "test-support"))]
pub use self::world::{ModelOwnerBlockGuard, World, WorldBuildError, WorldBuilder, cycle_request};
