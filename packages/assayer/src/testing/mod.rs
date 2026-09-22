// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Test infrastructure shared by inline tests, crate tests, integration targets, and benchmarks.
//!
//! This module is public but hidden from generated API documentation. It exposes the public subject surface wherever that surface can express a scenario; where a test needs a barrier, a parked steward, or an owned reading of published state, the harness reaches crate-internal machinery and supplies that guarantee. Tests therefore describe the state they need without reconstructing scheduler waits.
//!
//! # Shape
//!
//! The harness is organised around a single top-level context, [`World`], which owns an [`Assayer`](crate::Assayer) plus the auxiliary state a test needs to speak in names rather than identifiers:
//!
//! ```text
//! World
//! ├── Assayer          (the real engine — no mocks)
//! ├── VirtualClock     (deterministic time shared with the engine)
//! ├── TestRng          (seeded LCG, reproducible)
//! ├── NameRegistry     ("S1" → SentinelId, "default" → ChannelId, …)
//! └── Tolerances       (named epsilons used by asserts)
//! ```
//!
//! The deterministic-time boundary is (´dec:clock:two-domains´); names replace raw identifiers throughout the harness.
//!
//! # Vocabulary
//!
//! The principal verbs map directly to scenario sentences:
//!
//! | Scenario phrase | Harness verb |
//! | --- | --- |
//! | "register a Sentinel" | [`World::register_sentinel`] |
//! | "deregister Sentinel A" | [`World::deregister_sentinel`] |
//! | "ingest a Sentinel report" | [`World::receive_report`] |
//! | "process an assessment" | [`World::assess`] |
//! | "submit a label and settle publication" | [`World::cycle_on`] |
//! | "advance 29 days" | [`World::advance`] |
//! | "travel to an instant" | [`World::travel_to`] |
//!
//! The surrounding modules supply construction declarations, guarded fixtures, liveness barriers, owned probes, independent oracles, declarative playback, reproducible sweeps, report builders, and assertions. Their intended items are flattened below so callers share one testing vocabulary.
//!
//! # Sealing
//!
//! Only the clock crosses into a sealed featureless build: production reads time through [`Clock`], while the harness proper compiles only for crate tests or the `test-support` feature. The flattened public test surface still requires a caller census when capabilities retire, because a public item is not diagnosed as dead code merely because its test callers disappeared.

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
#[cfg(all(test, feature = "serde"))]
mod durable_projection;
#[cfg(all(test, feature = "serde"))]
mod durable_value;
#[cfg(any(test, feature = "test-support"))]
mod fixtures;
#[cfg(all(feature = "serde", any(test, feature = "test-support")))]
mod fork;
#[cfg(any(test, feature = "test-support"))]
mod label_spec;
#[cfg(any(test, feature = "test-support"))]
mod liveness;
#[cfg(any(test, feature = "test-support"))]
pub mod maintenance;
#[cfg(any(test, feature = "test-support"))]
mod names;
#[cfg(any(test, feature = "test-support"))]
mod oracles;
#[cfg(any(test, feature = "test-support"))]
pub mod performance;
#[cfg(any(test, feature = "test-support"))]
mod playback;
#[cfg(any(test, feature = "test-support"))]
mod preseed_spec;
#[cfg(any(test, feature = "test-support"))]
mod probes;
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
mod sweeps;
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
#[cfg(all(test, feature = "serde"))]
pub(crate) use self::durable_projection::DurableProjection;
#[cfg(any(test, feature = "test-support"))]
pub use self::fixtures::{TrainedStateBaseline, TrainedStateFixture, TrainedStateFixtureError};
#[cfg(all(feature = "serde", any(test, feature = "test-support")))]
pub use self::fork::PersistenceFork;
#[cfg(all(test, feature = "serde"))]
pub(crate) use self::fork::checkpoint as durable_checkpoint;
#[cfg(any(test, feature = "test-support"))]
pub use self::label_spec::LabelSpec;
#[cfg(any(test, feature = "test-support"))]
pub use self::liveness::{
    ACK_DEADLINE, Progress, STATE_DEADLINE, advanced, fail_fast_on_model_owner_panic, finished, watch_progress,
};
#[cfg(any(test, feature = "test-support"))]
pub use self::names::{AxisName, ChannelName, IdentityName, SentinelName};
#[cfg(any(test, feature = "test-support"))]
pub use self::oracles::*;
#[cfg(any(test, feature = "test-support"))]
pub use self::playback::{
    PlaybackBarrier, PlaybackBarrierPolicy, PlaybackBatchSize, PlaybackCheckpoint, PlaybackCheckpointError, PlaybackError,
    PlaybackProgress, PlaybackRow, PlaybackRowLocation, PlaybackStall, playback,
};
#[cfg(any(test, feature = "test-support"))]
pub use self::preseed_spec::PreSeedSpec;
#[cfg(any(test, feature = "test-support"))]
pub use self::probes::{PendingEntryView, PublishedModelBlock, PublishedSlotMoments};
#[cfg(any(test, feature = "test-support"))]
pub use self::reports::*;
#[cfg(any(test, feature = "test-support"))]
pub use self::rng::TestRng;
#[cfg(any(test, feature = "test-support"))]
pub use self::scenario::{Scenario, scenario, scenario_with, scenario_with_config};
#[cfg(any(test, feature = "test-support"))]
pub use self::signals::{
    score_verified_request_schema, with_degraded_score_verified, with_score_verified, with_standard_final_score_verified,
    with_standard_training_score_verified,
};
#[cfg(any(test, feature = "test-support"))]
pub use self::source_scan::{LineVisit, SourceTreeScanner, manifest_dir};
#[cfg(any(test, feature = "test-support"))]
pub use self::sweeps::run_seeded_sweep;
#[cfg(any(test, feature = "test-support"))]
pub use self::tolerances::{DEFAULT_TOLERANCES, Tolerances};
#[cfg(any(test, feature = "test-support"))]
pub use self::virtual_clock::VirtualClock;
#[cfg(any(test, feature = "test-support"))]
pub use self::world::{
    ChannelBaseline, ChannelPolicyBaseline, CompetitiveCellSpec, ConstructionFixture, ModelOwnerBlockGuard, RuntimeLayout, World,
    WorldBuildError, WorldBuilder, WorldConstructionBaseline, WorldConstructionDeclaration, WorldFixtureError, cycle_request,
};
#[cfg(any(test, feature = "test-support"))]
pub use crate::feature::interaction::{FeatureSelector, InteractionTemplate};
#[cfg(any(test, feature = "test-support"))]
pub use crate::identity::CompetitiveCellId;
