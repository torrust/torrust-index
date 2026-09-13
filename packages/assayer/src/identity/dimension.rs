// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`dimension_construction`] | identity | A dimension holds the identity it was registered under and the shape of the coordinate space it partitions: its name, its domain width, and the depth beyond which cells stop being tracked. Those three are what the maintenance loop later reasons about, so a dimension that quietly altered them would leave the loop partitioning a space nobody asked for. |
//! | [`dimension_encode_entity`] | identity | The encoder places an entity at a fixed point in the dimension's coordinate space: the same key always lands on the same coordinate, and two different keys land apart. Stability is what makes an entity's history accumulate in one cell rather than scattering, and separation is what stops one entity's traffic being read as another's. |
//! | [`dimension_debug_format`] | identity | A dimension's debug form identifies it by name and identifier, so an operator reading a log can tell which dimension a message came from. The encoding function is a boxed closure with nothing printable about it, and is rendered as a placeholder rather than being omitted. |

//! Identity dimension configuration.
//!
// Data structures only. A dedicated owner holds every dimension's graph, and
// the assessment path never mutates one (´dec:memory:graph-owner´).
#![allow(dead_code)]
//!
//! This module provides the [`IdentityDimension`] type that holds the
//! configuration and encoding function for a single identity dimension.
//!
//! # Cross-References
//!
//! - (´schema:keyspace:dimension-record´) — what a registered dimension
//!   declares
//! - (´req:keyspace:encoding-contract´) — what the encoding function owes
//! - (´dec:ownership:coordinate-width´) — why the coordinate is carried at one
//!   fixed internal width

use std::sync::Arc;

use crate::types::{DimensionId, EntityKey};

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension
// ═══════════════════════════════════════════════════════════════════════════════

/// Configuration and encoding function for an identity dimension.
///
/// Each dimension provides a mapping from entity keys to 128-bit coordinates
/// in a dyadic domain. The dimension's G-V graph tracks which regions of
/// this coordinate space have significant traffic, creating a competitive
/// set of cells.
///
/// # Encoding Function
///
/// The `encode` function maps an entity key to a 128-bit coordinate. This
/// is typically a hash function that distributes entities uniformly across
/// the domain. The same entity should always produce the same coordinate.
///
/// # Depth Cutoff
///
/// The `depth_cutoff` limits how deep in the G-tree competitive cells can
/// be. Cells deeper than this are not tracked as competitive. This prevents
/// tracking individual entities, focusing instead on population segments.
pub struct IdentityDimension {
    /// Unique identifier for this dimension.
    pub id: DimensionId,

    /// Human-readable name for this dimension.
    pub name: String,

    /// What this key space is, in the host's own words.
    pub description: String,

    /// What coordinates mean, how they are encoded, and whether shared
    /// prefixes assert shared context.
    pub coordinate_semantics: String,

    /// Domain bit-width (typically 128).
    pub domain_bits: u8,

    /// Maximum depth for competitive cells.
    ///
    /// Cells at depth > `depth_cutoff` are not tracked as competitive.
    pub depth_cutoff: u8,

    /// Entity-to-coordinate encoding function.
    ///
    /// Maps an entity key to a 128-bit coordinate in the dimension's domain.
    pub encode: Arc<dyn Fn(&EntityKey) -> u128 + Send + Sync>,
}

impl IdentityDimension {
    /// Creates a new identity dimension.
    ///
    /// # Arguments
    ///
    /// * `id` — Unique dimension identifier
    /// * `name` — Human-readable name
    /// * `description` — Host description of the key space
    /// * `coordinate_semantics` — Host declaration of coordinate meaning
    /// * `domain_bits` — Domain bit-width (typically 128)
    /// * `depth_cutoff` — Maximum depth for competitive cells
    /// * `encode` — Entity-to-coordinate encoding function
    pub fn new(
        id: DimensionId,
        name: impl Into<String>,
        description: impl Into<String>,
        coordinate_semantics: impl Into<String>,
        domain_bits: u8,
        depth_cutoff: u8,
        encode: impl Fn(&EntityKey) -> u128 + Send + Sync + 'static,
    ) -> Self {
        Self {
            id,
            name: name.into(),
            description: description.into(),
            coordinate_semantics: coordinate_semantics.into(),
            domain_bits,
            depth_cutoff,
            encode: Arc::new(encode),
        }
    }

    /// Encodes an entity key to a coordinate in this dimension.
    #[must_use]
    pub fn encode_entity(&self, entity: &EntityKey) -> u128 {
        (self.encode)(entity)
    }
}

impl std::fmt::Debug for IdentityDimension {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("IdentityDimension")
            .field("id", &self.id)
            .field("name", &self.name)
            .field("description", &self.description)
            .field("coordinate_semantics", &self.coordinate_semantics)
            .field("domain_bits", &self.domain_bits)
            .field("depth_cutoff", &self.depth_cutoff)
            .field("encode", &"<fn>")
            .finish()
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// Simple hash encoder for testing.
    fn test_encoder(entity: &EntityKey) -> u128 {
        // Simple FNV-1a style hash
        let mut hash = 0xcbf2_9ce4_8422_2325_u64;
        for byte in entity.as_bytes() {
            hash ^= u64::from(*byte);
            hash = hash.wrapping_mul(0x0100_0000_01b3);
        }
        u128::from(hash) | (u128::from(hash) << 64)
    }

    /// A dimension holds the identity it was registered under and the shape of
    /// the coordinate space it partitions: its name, its domain width, and the
    /// depth beyond which cells stop being tracked. Those three are what the
    /// maintenance loop later reasons about, so a dimension that quietly
    /// altered them would leave the loop partitioning a space nobody asked for.
    ///
    /// ´claim:identity:a-dimension-keeps-the-name-domain-width-and-depth-cutoff-it-was-registered-with´
    /// ´test:unit:dimension-construction´
    #[test]
    fn dimension_construction() {
        let dim = IdentityDimension::new(
            DimensionId(1),
            "test-dimension",
            "test hierarchy",
            "the high bits identify the test group",
            128,
            24,
            test_encoder,
        );

        assert_eq!(dim.id, DimensionId(1));
        assert_eq!(dim.name, "test-dimension");
        assert_eq!(dim.description, "test hierarchy");
        assert_eq!(dim.coordinate_semantics, "the high bits identify the test group");
        assert_eq!(dim.domain_bits, 128);
        assert_eq!(dim.depth_cutoff, 24);
    }

    /// The encoder places an entity at a fixed point in the dimension's
    /// coordinate space: the same key always lands on the same coordinate, and
    /// two different keys land apart. Stability is what makes an entity's
    /// history accumulate in one cell rather than scattering, and separation is
    /// what stops one entity's traffic being read as another's.
    ///
    /// ´claim:identity:a-dimensions-encoder-is-stable-per-entity-and-separates-distinct-entities´
    /// ´test:unit:dimension-encode-entity´
    #[test]
    fn dimension_encode_entity() {
        let dim = IdentityDimension::new(
            DimensionId(1),
            "test",
            "test hierarchy",
            "prefix groups",
            128,
            24,
            test_encoder,
        );

        let entity1 = EntityKey::new(b"user-123".to_vec());
        let entity2 = EntityKey::new(b"user-456".to_vec());

        let coord1 = dim.encode_entity(&entity1);
        let coord2 = dim.encode_entity(&entity2);

        // Same entity should produce same coordinate
        assert_eq!(coord1, dim.encode_entity(&entity1));

        // Different entities should (very likely) produce different coordinates
        assert_ne!(coord1, coord2);
    }

    /// A dimension's debug form identifies it by name and identifier, so an
    /// operator reading a log can tell which dimension a message came from.
    /// The encoding function is a boxed closure with nothing printable about
    /// it, and is rendered as a placeholder rather than being omitted.
    ///
    /// ´claim:identity:a-dimensions-debug-form-identifies-it-by-name-and-id-and-stands-in-for-its-encoder´
    /// ´test:unit:dimension-debug-format´
    #[test]
    fn dimension_debug_format() {
        let dim = IdentityDimension::new(DimensionId(42), "my-dim", "test hierarchy", "prefix groups", 128, 16, |_| 0);

        let debug = format!("{dim:?}");
        assert!(debug.contains("IdentityDimension"));
        assert!(debug.contains("my-dim"));
        assert!(debug.contains("test hierarchy"));
        assert!(debug.contains("prefix groups"));
        assert!(debug.contains("42"));
    }
}
