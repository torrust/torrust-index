// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Trait implementations on `GvGraph`: `SpatialRead`,
//! `SpatialWrite`, `TemporalDecay`, `WeightedSampler`.
//!
//! Extracted from `graph.rs` per ADR-M-030 Phase F.
//! See ADR-M-009 (trait decomposition) and §API M-5.5.

use crate::graph::GvGraph;
use crate::traits::{Accumulator, Coordinate, Inspectable};

// ── SpatialRead impl ─────────────────────────────────────────────────

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> crate::traits::SpatialRead for GvGraph<C, V, N> {
    type Coord = C;
    type Accum = V;

    fn plateaus(
        &self,
    ) -> std::borrow::Cow<'_, std::collections::BTreeMap<crate::plateau::BasisEdge<C>, crate::plateau::Plateau<C, V>>> {
        self.plateaus()
    }

    fn get(&self, coord: C) -> crate::view::Cell<C, V> {
        self.get(coord)
    }
}

// ── SpatialWrite impl ────────────────────────────────────────────────

impl<C: Coordinate, V: Accumulator + Inspectable, const N: u32> crate::traits::SpatialWrite for GvGraph<C, V, N> {
    fn observe<O: crate::traits::Observation<V>>(&mut self, coord: C, delta: O) {
        self.observe(coord, delta);
    }
}

// ── TemporalDecay impl ──────────────────────────────────────────────

impl<C: Coordinate, V: Accumulator + crate::traits::Attenuatable + Inspectable, const N: u32> crate::traits::TemporalDecay
    for GvGraph<C, V, N>
{
    fn decay(&mut self, root: crate::handle::GNodeId, attenuation: f64, q: f64) {
        self.decay(root, attenuation, q);
    }
}

// ── WeightedSampler impl ─────────────────────────────────────────────

impl<C: Coordinate, V: Accumulator + Inspectable + crate::traits::Weighable, const N: u32> crate::traits::WeightedSampler
    for GvGraph<C, V, N>
{
    fn sample(&self, rng: &mut impl crate::traits::Rng) -> Option<crate::view::Cell<C, V>> {
        self.sample(rng)
    }
}
