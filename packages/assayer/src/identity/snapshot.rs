// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`snapshot_empty`] | identity | A snapshot of a dimension that has seen nothing reports nothing: no layers, no terminal cells, and no energy. A checkpoint taken before any traffic arrives therefore restores an engine that knows it has learned nothing, rather than one carrying phantom importance. |
//! | [`snapshot_from_pewei_preserves_energy`] | identity | The energy a snapshot reports is the graph's whole total, not merely what its terminal cells hold. Because the extraction keeps transition baselines alongside terminals, importance resting at internal nodes is captured too — an earlier terminals-only snapshot silently lost it, and a checkpoint that loses energy restores a dimension that has forgotten traffic it actually saw. |
//! | [`reconstruction_observations_round_trips`] | identity | A snapshot replays as observations, and a graph rebuilt from them holds exactly the total it started with — the snapshot will confirm the match itself. The rebuilt V-tree need not have the same shape, since observation order affects the tournament rankings; what has to survive a restart is the importance distribution the competitive set is drawn from, and that is what the round trip preserves. |
//! | [`snapshot_serialization_roundtrip`] | identity | A snapshot survives being written out and read back with its energy intact. That is the whole point of holding a serialisable extraction rather than the graph itself: the graph can be cloned but not written to disk, so without this the identity layer could not be checkpointed at all. |

//! Identity graph snapshot for checkpoint persistence.
//!
// Data structures only. A dedicated owner holds the graph a snapshot is taken
// from, and the assessment path never mutates it (´dec:memory:graph-owner´).
#![allow(dead_code)]
//!
//! This module provides the [`IdentityGraphSnapshot`] type used for
//! checkpointing G-V graph state. The snapshot wraps a [`Pewei`] —
//! the significance-ordered extraction produced by
//! [`GvGraph::extract()`](torrust_mudlark::GvGraph::extract).
//!
//! # Checkpoint Strategy
//!
//! `GvGraph` is Clone but not directly serializable. The snapshot stores
//! the full [`Pewei`] (terminals *and* transition baselines) so that
//! reconstruction via [`from_observations()`](torrust_mudlark::GvGraph::from_observations)
//! preserves total energy. Earlier approaches that stored only terminal
//! `(coordinate, intensity)` pairs lost baseline energy at internal nodes.
//!
//! Reconstruction:
//! ```ignore
//! let observations = snapshot.reconstruction_observations();
//! let graph = GvGraph::from_observations(config, observations);
//! ```
//!
//! # Cross-References
//!
//! - (´dec:durability:checkpoint-journal´) — the periodic checkpoint this
//!   snapshot is taken for
//! - (´dec:memory:competitive-publication´) — the per-dimension swap the
//!   maintenance loop publishes through
//! - (´[MUDLARK-claim:pewei:a-snapshot-taken-from-a-live-graph-reconstructs-to-the-graphs-total-energy]´)
//!   — the upstream guarantee that makes storing the full extraction
//!   sufficient

use torrust_mudlark::Pewei;

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Graph Snapshot
// ═══════════════════════════════════════════════════════════════════════════════

/// Snapshot of a G-V graph for checkpoint persistence.
///
/// Wraps a [`Pewei<u128, u64>`] — the significance-ordered extraction
/// that captures both terminal intensities and transition baselines.
/// This preserves total energy across serialization round-trips,
/// unlike terminal-only extraction which loses baseline energy at
/// internal nodes.
///
/// # Reconstruction
///
/// [`reconstruction_observations()`](Self::reconstruction_observations)
/// yields `(coordinate, delta)` pairs suitable for
/// [`GvGraph::from_observations()`](torrust_mudlark::GvGraph::from_observations).
/// The V-tree structure after reconstruction may differ (observation order
/// affects tournament rankings), but the competitive set is equivalent
/// because it depends only on the importance distribution.
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct IdentityGraphSnapshot {
    /// The extracted Pewei snapshot.
    pewei: Pewei<u128, u64>,
}

impl IdentityGraphSnapshot {
    /// Creates a snapshot from a Pewei extraction.
    #[must_use]
    pub const fn from_pewei(pewei: Pewei<u128, u64>) -> Self {
        Self { pewei }
    }

    /// Returns `true` if the snapshot has no nodes.
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.pewei.layers.is_empty()
    }

    /// Returns the total energy captured in this snapshot.
    ///
    /// After reconstruction, `graph.total_sum()` should equal this value.
    #[must_use]
    pub fn total_energy(&self) -> u64 {
        self.pewei.total_energy()
    }

    /// Returns the number of terminal cells.
    #[must_use]
    pub fn terminal_count(&self) -> usize {
        self.pewei.layers.iter().map(|l| l.terminals.len()).sum()
    }

    /// Returns a reference to the underlying Pewei.
    #[must_use]
    pub const fn pewei(&self) -> &Pewei<u128, u64> {
        &self.pewei
    }

    /// Yields `(coordinate, delta)` pairs for graph reconstruction.
    ///
    /// Emits one pair per terminal (`start`, `intensity`) and one per
    /// transition (`start`, `baseline`). Feeding these into
    /// [`GvGraph::from_observations()`](torrust_mudlark::GvGraph::from_observations)
    /// produces a graph whose `total_sum()` equals [`total_energy()`](Self::total_energy).
    pub fn reconstruction_observations(&self) -> impl Iterator<Item = (u128, u64)> + '_ {
        self.pewei.layers.iter().flat_map(|layer| {
            let terminals = layer.terminals.iter().map(|t| (t.start, t.intensity));
            let transitions = layer.transitions.iter().map(|t| (t.start, t.baseline));
            terminals.chain(transitions)
        })
    }

    /// Validates that a reconstructed graph matches this snapshot.
    ///
    /// Returns `true` if the given total sum equals [`total_energy()`](Self::total_energy).
    #[must_use]
    pub fn validate_total_sum(&self, graph_total_sum: u64) -> bool {
        graph_total_sum == self.total_energy()
    }
}

impl Default for IdentityGraphSnapshot {
    fn default() -> Self {
        Self {
            pewei: Pewei {
                domain_start: 0,
                domain_end: u128::MAX,
                layers: Vec::new(),
                v_depth_limit: None,
            },
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use torrust_mudlark::{Config as MudlarkConfig, GvGraph};

    use super::*;

    fn test_config() -> MudlarkConfig<u64> {
        MudlarkConfig {
            split_threshold: 5,
            depth_create: 20,
            depth_evict: 30,
            budget: None,
            alpha_relax: 0.75,
            bounded_eviction: true,
        }
    }

    /// A snapshot of a dimension that has seen nothing reports nothing: no
    /// layers, no terminal cells, and no energy. A checkpoint taken before any
    /// traffic arrives therefore restores an engine that knows it has learned
    /// nothing, rather than one carrying phantom importance.
    ///
    /// ´claim:identity:a-snapshot-of-an-untouched-dimension-reports-no-cells-and-no-energy´
    /// ´test:unit:snapshot-empty´
    #[test]
    fn snapshot_empty() {
        let snap = IdentityGraphSnapshot::default();
        assert!(snap.is_empty());
        assert_eq!(snap.terminal_count(), 0);
        assert_eq!(snap.total_energy(), 0);
    }

    /// The energy a snapshot reports is the graph's whole total, not merely
    /// what its terminal cells hold. Because the extraction keeps transition
    /// baselines alongside terminals, importance resting at internal nodes is
    /// captured too — an earlier terminals-only snapshot silently lost it, and
    /// a checkpoint that loses energy restores a dimension that has forgotten
    /// traffic it actually saw.
    ///
    /// ´claim:identity:a-snapshot-carries-the-graphs-whole-energy-not-just-what-its-terminals-hold´
    /// ´test:unit:snapshot-from-pewei-preserves-energy´
    #[test]
    fn snapshot_from_pewei_preserves_energy() {
        let config = test_config();
        let mut graph = GvGraph::<u128, u64, 128>::new(config);

        for i in 0..100u128 {
            graph.observe(i * 1000, 1);
        }

        let snap = IdentityGraphSnapshot::from_pewei(graph.extract());
        assert_eq!(snap.total_energy(), graph.total_sum());
    }

    /// A snapshot replays as observations, and a graph rebuilt from them holds
    /// exactly the total it started with — the snapshot will confirm the match
    /// itself. The rebuilt V-tree need not have the same shape, since
    /// observation order affects the tournament rankings; what has to survive a
    /// restart is the importance distribution the competitive set is drawn
    /// from, and that is what the round trip preserves.
    ///
    /// ´claim:identity:a-graph-rebuilt-from-a-snapshots-observations-holds-the-total-energy-it-started-with´
    /// ´test:unit:reconstruction-observations-round-trips´
    #[test]
    fn reconstruction_observations_round_trips() {
        let config = test_config();
        let mut graph = GvGraph::<u128, u64, 128>::new(config.clone());

        for i in 0..100u128 {
            graph.observe(i * 1000, 1);
        }

        let original_total = graph.total_sum();
        let snap = IdentityGraphSnapshot::from_pewei(graph.extract());

        let reconstructed = GvGraph::<u128, u64, 128>::from_observations(config, snap.reconstruction_observations());

        assert_eq!(
            reconstructed.total_sum(),
            original_total,
            "reconstructed graph should have same total_sum"
        );
        assert!(snap.validate_total_sum(reconstructed.total_sum()));
    }

    /// A snapshot survives being written out and read back with its energy
    /// intact. That is the whole point of holding a serialisable extraction
    /// rather than the graph itself: the graph can be cloned but not written to
    /// disk, so without this the identity layer could not be checkpointed at
    /// all.
    ///
    /// ´claim:identity:a-snapshot-survives-serialisation-with-its-energy-intact´
    /// ´test:unit:snapshot-serialization-roundtrip´
    #[test]
    #[cfg(feature = "serde")]
    fn snapshot_serialization_roundtrip() {
        use torrust_mudlark::GvGraph;

        let config = test_config();
        let mut graph = GvGraph::<u128, u64, 128>::new(config);
        graph.observe(0x1234, 10);
        graph.observe(0xABCD, 20);

        let snap = IdentityGraphSnapshot::from_pewei(graph.extract());
        let json = serde_json::to_string(snap.pewei()).unwrap();
        let restored: Pewei<u128, u64> = serde_json::from_str(&json).unwrap();
        let restored_snap = IdentityGraphSnapshot::from_pewei(restored);

        assert_eq!(restored_snap.total_energy(), snap.total_energy());
    }
}
