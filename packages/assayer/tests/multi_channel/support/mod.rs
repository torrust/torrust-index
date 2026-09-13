// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Helpers this binary is the only caller of.
//!
//! The rest of the integration suite reaches the harness through
//! [`torrust_assayer::testing`]; what is here answers to the enrichment grid and
//! the derivation matrices, which nothing else runs. It is private and compiled
//! once, so an item that loses its last caller is reported as dead rather than
//! covered by a blanket — which is what stopped being possible when this stopped
//! being a module thirteen binaries included by path.

use torrust_assayer::SignalDeclaration;
use torrust_assayer::testing::WorldBuilder;

/// Trait alias for scenario builders that can receive a request-signal schema.
pub trait ScenarioBuilderExt: Sized {
    /// Attach the supplied signal declarations to the builder.
    fn signal_schema(self, schema: Vec<SignalDeclaration>) -> Self;
}

impl ScenarioBuilderExt for WorldBuilder {
    fn signal_schema(self, schema: Vec<SignalDeclaration>) -> Self {
        Self::signal_schema(self, schema)
    }
}

mod assertions;
mod constants;
mod derivation;
mod enrichment;
mod labels;
mod outcomes;
mod policy;
mod rewards;

pub use assertions::*;
pub use constants::*;
pub use derivation::*;
pub use enrichment::*;
pub use labels::*;
pub use outcomes::*;
pub use policy::*;
pub use rewards::*;
