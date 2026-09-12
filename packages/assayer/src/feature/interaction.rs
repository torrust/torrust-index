// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`type1_feature_count_scales_with_sentinels`] | feature | A per-Sentinel template yields exactly one instance for each registered Sentinel and none at all when there are none, unaffected by how many competitive cells exist. A template is a rule rather than a feature, and the population it ranges over is what turns it into concrete slots — so registering a Sentinel widens the vector by one per such template. |
//! | [`type2_feature_count_is_one_named_pair`] | feature | The named pair costs exactly one dimension, whatever the population. It names two Sentinels, so it is present while both are registered and absent otherwise, and adding a hundred more Sentinels adds nothing to it. That is the whole difference between it and the wildcard: at the reference configuration's eight Sentinels a wildcard buys twenty-eight coordinates and a named pair buys one, so a declaration that meant the named pair and got the wildcard would move every learned weight, the factorisation cost, and the dimension identity the reference tables state (´def:feature:template-named-pair´). |
//! | [`type3_feature_count_is_combinations`] | feature | The wildcard yields one instance per unordered pair, so it stays empty at zero and at one Sentinel and then grows quadratically — twenty-eight instances at eight Sentinels. A pair feature has nothing to say until there are two parties to it, and the quadratic growth is the cost that makes the wildcard the expensive kind to declare (´def:feature:template-wildcard´). It is the third template kind, not the second: the position matters, because the second is the named pair and the two differ by a factor of twenty-eight at the reference configuration. |
//! | [`type4_feature_count_always_one`] | feature | A template whose operands are both fleet-wide yields exactly one instance, with an empty population as much as with a hundred Sentinels and a hundred cells. Neither operand belongs to any entity, so there is nothing for the template to range over and its slot never moves as the population churns. |
//! | [`type5_feature_count_scales_with_cells`] | feature | A competitive-cell template ranges over cells rather than Sentinels: its instance count follows the total number of competitive cells and ignores the Sentinel population entirely, held here at eight throughout. Each template names the one population it depends on, so cell entry and exit resize only the templates that actually concern cells. |
//! | [`total_interaction_count_formula`] | feature | The interaction block's width is simply the sum of what each declared template generates under the current population — five per-Sentinel templates across eight Sentinels giving forty slots. Templates do not share or overlap instances, so an operator can size the block from the template list and the population counts alone. |
//! | [`interaction_id_equality`] | feature | An interaction instance is identified by its template together with the entity it was generated for: two instances of the same template for different Sentinels are different identities, and the same template and Sentinel names the same one. Instances are looked up by identity to find their index, so identity has to survive a rebuild rather than depending on where the instance happened to land last time. |
//! | [`interaction_id_per_pair_ordering`] | feature | A pair identity is not symmetric: naming the same two Sentinels in the opposite order produces a different identity. The identity does not canonicalise the pair for its callers, so a pair must always be built in one settled order — earlier registration first — or a lookup will miss the instance that was generated. |
//! | [`interaction_id_per_cell`] | feature | A per-cell identity carries the dimension alongside the cell, both recoverable from it intact. Cells are numbered within their own dimension, so the cell alone would not distinguish two cells that happen to occupy the same position in different dimensions. |
//! | [`type1_product_computed`] | feature | An interaction slot holds the plain product of its two operands, each read from wherever it lives in the vector — here a Sentinel's feature at an offset inside that Sentinel's slot, past the occupancy indicator, times an aggregate read by absolute index. The product is what lets a linear model express a conjunction, and it is formed from the vector itself rather than from any private copy of the inputs. |
//! | [`type1_absent_sentinel_zeros`] | feature | A Sentinel contributing nothing zeroes its own interaction however loud the shared operand is. Zero is the identity-destroying element of a product, so a non-reporting Sentinel cannot borrow significance from a context feature it had no part in raising — the conjunction requires both halves. |
//! | [`type4_product_computed`] | feature | cites (´claim:feature:an-interaction-slot-holds-the-plain-product-of-its-two-operands-read-from-their-own-positions´) |
//! | [`type5_active_cell_product`] | feature | A competitive indicator acts as a gate on its context: an active cell's indicator stands at one, so its interaction carries the context value through unchanged. Multiplying by a zero-or-one indicator is how the model is given a context feature conditioned on membership, without needing a separate feature for the conditioned and unconditioned cases. |
//! | [`type5_inactive_cell_zeros`] | feature | cites (´claim:feature:a-competitive-indicator-gates-its-context-passing-it-through-when-the-cell-is-active´) |
//! | [`compute_interactions_writes_correct_positions`] | feature | With two templates of different kinds active over two Sentinels, each instance lands at the index its own identity maps to and carries its own operands' product: the two per-Sentinel instances differ from each other and both from the fleet-wide one. A single pass fills the whole block, so the identity-to-index map is the only thing keeping instances from overwriting one another. |
//! | [`compute_interactions_unstandardised`] | feature | Products are formed from the raw assembled values, before any standardisation touches the vector: operands of five and ten give fifty, not the near-zero a product of already-centred values would give. The ordering matters because a product of standardised terms is a different quantity from a standardised product, and the interaction slot is itself standardised afterwards as a feature in its own right. |

#![allow(dead_code)]

//! Interaction feature templates and computation.
//!
//! Interaction features are products of two base features from φ. The template
//! system supports five types of interactions that capture structurally different
//! product classes.
//!
//! # Template Types
//!
//! | Type | Formula | Instance Count |
//! |------|---------|----------------|
//! | Type1 | Sentinel feature × context | n per template |
//! | Type2 | Sentinel pair features | C(n,2) per template |
//! | Type3 | Sentinel feature × aggregate | n per template |
//! | Type4 | Aggregate × context | 1 per template |
//! | Type5 | Competitive indicator × context | `Σ\|E_d\|` per template |
//!
//! # Cross-References
//!
//! - (´dec:vector:semantic-templates´) — that a template names its operands by
//!   what they are rather than where they sit
//! - (´sec:feature:interactions´) — the five template types, the default set
//!   built from them, and what lifecycle events do to them

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::feature::dimension_map::DimensionMap;
use crate::identity::CompetitiveCellId;
use crate::types::{DimensionId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Interaction ID
// ═══════════════════════════════════════════════════════════════════════════════

/// Unique identifier for an interaction feature instance.
///
/// Each interaction template instance has a unique identity determined by the
/// template index and the dynamic entity (if any) that it's associated with.
///
/// # Cross-References
///
/// - (´alg:dimension:compilation-pipeline´) — the resolution stage, which is
///   where one identity per expanded feature comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InteractionId {
    /// Per-Sentinel interaction: template × Sentinel.
    PerSentinel {
        /// Template index in the template list.
        template: usize,
        /// The Sentinel this interaction is associated with.
        sentinel: SentinelId,
    },

    /// Per-Sentinel-pair interaction: template × (Sentinel A, Sentinel B).
    /// Pairs are ordered (A < B in registration order).
    PerPair {
        /// Template index in the template list.
        template: usize,
        /// The first Sentinel in the pair (earlier registration).
        sentinel_a: SentinelId,
        /// The second Sentinel in the pair (later registration).
        sentinel_b: SentinelId,
    },

    /// Fixed interaction: template only, no dynamic entity.
    Fixed {
        /// Template index in the template list.
        template: usize,
    },

    /// Per-competitive-cell interaction: template × (dimension, cell).
    PerCell {
        /// Template index in the template list.
        template: usize,
        /// The dimension containing this cell.
        dimension: DimensionId,
        /// The competitive cell.
        cell: CompetitiveCellId,
    },
}

impl InteractionId {
    /// Creates a per-Sentinel interaction ID.
    #[must_use]
    pub const fn per_sentinel(template: usize, sentinel: SentinelId) -> Self {
        Self::PerSentinel { template, sentinel }
    }

    /// Creates a per-pair interaction ID.
    #[must_use]
    pub const fn per_pair(template: usize, sentinel_a: SentinelId, sentinel_b: SentinelId) -> Self {
        Self::PerPair {
            template,
            sentinel_a,
            sentinel_b,
        }
    }

    /// Creates a fixed interaction ID.
    #[must_use]
    pub const fn fixed(template: usize) -> Self {
        Self::Fixed { template }
    }

    /// Creates a per-cell interaction ID.
    #[must_use]
    pub const fn per_cell(template: usize, dimension: DimensionId, cell: CompetitiveCellId) -> Self {
        Self::PerCell {
            template,
            dimension,
            cell,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interaction Template
// ═══════════════════════════════════════════════════════════════════════════════

/// An interaction feature template.
///
/// Each template, combined with the current set of registered entities
/// (Sentinels, competitive cells), generates one or more concrete interaction
/// features in φ. The generated features are assigned contiguous indices in
/// the interaction block.
///
/// One compiled interaction: an output index and its two operand indices.
///
/// The pipeline's third stage is "a flat list of index triples, one per
/// feature: output, first operand, second operand", and it is what feature
/// assembly iterates, "with no name lookup and no template interpretation left
/// in it" (´alg:dimension:compilation-pipeline´). Compilation is recomputed on
/// every map rebuild, so a triple is only ever read against the layout that
/// emitted it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct InteractionTriple {
    /// Index of the interaction feature this triple writes.
    pub output: usize,
    /// Index of the first operand.
    pub operand_a: usize,
    /// Index of the second operand.
    pub operand_b: usize,
}

/// What an interaction operand names, rather than where it currently sits.
///
/// The declaration stage of the compilation pipeline carries "two operand
/// selectors naming a slot feature, an aggregate feature, an identity feature,
/// a declared signal, or the owning competitive indicator"
/// (´alg:dimension:compilation-pipeline´), and the later stages are recomputed
/// on every map rebuild precisely so that a declaration survives a layout
/// change. A raw index into the vector has nothing in it to resolve, so a
/// rebuild cannot carry it: the recommended set draws every identity operand
/// from the cross-dimension block, and that block moves whenever a
/// per-dimension block is added (´tab:feature:default-interaction-set´).
///
/// Three of these name a block and an offset within it, and resolve against
/// the map. Two are relative to the entity the template expands over and are
/// resolved at expansion time instead.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum FeatureSelector {
    /// A position inside the expanding Sentinel's own slot, by extraction
    /// offset past the occupancy indicator.
    SlotFeature(usize),
    /// A position inside the aggregate block, by offset. The aggregate block
    /// never moves.
    AggregateFeature(usize),
    /// A position inside the cross-dimension identity block, by offset. Absent
    /// — and read as zero — while no identity dimension is registered.
    IdentityFeature(usize),
    /// A position inside the declared signal block, by offset.
    DeclaredSignal(usize),
    /// The competitive indicator of the cell the template is expanding over.
    OwningCompetitiveIndicator,
}

/// Templates are declared at construction time and do not change at runtime.
/// The concrete feature set changes when the entity population changes
/// (Sentinel registration, competitive cell entry/exit).
///
/// # Cross-References
///
/// - (´sec:feature:interactions´) — the five template types and the default
///   set built from them
///
/// TODO(2026-05-22) ´todo:code:migrate-this-legacy-offset-based-enum´: migrate this legacy offset-based enum to the
/// canonical `InteractionTemplate { template_type, operand_a, operand_b }`
/// struct with `FeatureSelector` operands, so the operands are named by what
/// they are rather than by offset (´dec:vector:semantic-templates´).
#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum InteractionTemplate {
    /// Per-Sentinel × context. One feature per registered Sentinel.
    ///
    /// `φ_int = φ_sent[S, sentinel_offset] × φ[context_idx]`
    ///
    /// Lifecycle: added on Sentinel registration; removed on deregistration.
    Type1 {
        /// Offset of the Sentinel feature within the slot (after occupancy).
        sentinel_feature_offset: usize,
        /// What the context operand names.
        context: FeatureSelector,
    },

    /// Named Sentinel pair. Exactly one feature, for the two Sentinels the
    /// declaration names (´def:feature:template-named-pair´).
    ///
    /// `φ_int = φ_sent[A, offset] × φ_sent[B, offset]`
    ///
    /// Lifecycle: present only while both named Sentinels are registered —
    /// added when the second of the two registers, removed when either
    /// deregisters. One dimension, whatever the population.
    Type2 {
        /// The first Sentinel the declaration names.
        sentinel_a: SentinelId,
        /// The second Sentinel the declaration names.
        sentinel_b: SentinelId,
        /// Offset of the feature within each Sentinel's slot.
        feature_offset: usize,
    },

    /// Wildcard Sentinel pair. One feature per unordered pair of registered
    /// Sentinels (´def:feature:template-wildcard´).
    ///
    /// `φ_int = φ_sent[S1, offset] × φ_sent[S2, offset]`
    ///
    /// Lifecycle: one feature added per existing Sentinel when a new Sentinel
    /// registers; removed for all pairs involving a deregistered Sentinel.
    /// The block is quadratic in the population, which is the cost the corpus
    /// attaches to this template and to no other.
    Type3 {
        /// Offset of the feature within each Sentinel's slot.
        feature_offset: usize,
    },

    /// Aggregate × context. One feature regardless of Sentinel population.
    ///
    /// `φ_int = φ_agg[agg_offset] × φ[context_idx]`
    ///
    /// Lifecycle: always present. Never added or removed.
    Type4 {
        /// Offset within the aggregate block.
        agg_offset: usize,
        /// What the context operand names.
        context: FeatureSelector,
    },

    /// Competitive cell × context. One feature per active competitive cell.
    ///
    /// `φ_int = φ_comp[cell] × φ[context_idx]`
    ///
    /// Lifecycle: added on `CompetitiveCellEntry`; removed on `CompetitiveCellExit`.
    Type5 {
        /// What the context operand names.
        context: FeatureSelector,
    },
}

impl InteractionTemplate {
    /// Creates a Type-1 (Sentinel × context) template.
    #[must_use]
    pub const fn type1(sentinel_feature_offset: usize, context: FeatureSelector) -> Self {
        Self::Type1 {
            sentinel_feature_offset,
            context,
        }
    }

    /// Creates a Type-2 (named Sentinel pair) template.
    #[must_use]
    pub const fn type2(sentinel_a: SentinelId, sentinel_b: SentinelId, feature_offset: usize) -> Self {
        Self::Type2 {
            sentinel_a,
            sentinel_b,
            feature_offset,
        }
    }

    /// Creates a Type-3 (wildcard Sentinel pair) template.
    #[must_use]
    pub const fn type3(feature_offset: usize) -> Self {
        Self::Type3 { feature_offset }
    }

    /// Creates a Type-4 (aggregate × context) template.
    #[must_use]
    pub const fn type4(agg_offset: usize, context: FeatureSelector) -> Self {
        Self::Type4 { agg_offset, context }
    }

    /// Creates a Type-5 (competitive cell × context) template.
    #[must_use]
    pub const fn type5(context: FeatureSelector) -> Self {
        Self::Type5 { context }
    }

    /// Returns the number of features this template generates under a
    /// population.
    ///
    /// The named pair takes the registered set rather than its size, because
    /// its width depends on *which* Sentinels are registered and not on how
    /// many: it is one dimension while both the Sentinels it names are
    /// present and none otherwise (´def:feature:template-named-pair´). Every
    /// other kind reads only the counts.
    ///
    /// # Arguments
    ///
    /// * `sentinels` — Registered Sentinel identifiers
    /// * `n_competitive_cells` — Total competitive cells across all dimensions
    #[must_use]
    pub fn feature_count(&self, sentinels: &[SentinelId], n_competitive_cells: usize) -> usize {
        let n_sentinels = sentinels.len();
        match self {
            Self::Type1 { .. } => n_sentinels,
            Self::Type2 {
                sentinel_a, sentinel_b, ..
            } => usize::from(sentinels.contains(sentinel_a) && sentinels.contains(sentinel_b)),
            Self::Type3 { .. } => n_sentinels.saturating_sub(1) * n_sentinels / 2, // C(n,2)
            Self::Type4 { .. } => 1,
            Self::Type5 { .. } => n_competitive_cells,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interaction Computation
// ═══════════════════════════════════════════════════════════════════════════════

/// Computes interaction features from the assembled base features.
///
/// Called after all base blocks (bias, agg, identity, sig, Sentinel slots,
/// competitive indicators) are filled, BEFORE standardisation. Interaction
/// products use raw (unstandardised) base features.
///
/// # Arguments
///
/// * `phi` — The feature vector to modify (base features already filled)
/// * `dim_map` — The dimension map with template and index information
///
/// # Cross-References
///
/// - (´dec:vector:unstandardised-bases´) — why these products are formed from
///   raw operands, with the standardisation pass running once afterwards
///
/// The loop is flat: the map carries the compiled triples, so there is no name
/// lookup and no template interpretation left on this path
/// (´alg:dimension:compilation-pipeline´). A template instance whose operand
/// names nothing in the current layout emitted no triple, and its output
/// position keeps the zero the vector was initialised with.
pub fn compute_interactions(phi: &mut [f64], dim_map: &DimensionMap) {
    for triple in &dim_map.resolved_interactions {
        let Some(&a) = phi.get(triple.operand_a) else { continue };
        let Some(&b) = phi.get(triple.operand_b) else { continue };
        if let Some(slot) = phi.get_mut(triple.output) {
            *slot = a * b;
        }
    }
}

/// Calculates the total number of interaction features for a template set.
///
/// # Arguments
///
/// * `templates` — The interaction templates
/// * `sentinels` — Registered Sentinel identifiers
/// * `n_competitive_cells` — Total competitive cells across all dimensions
///
/// # Returns
///
/// The total interaction feature count `p_int`.
#[must_use]
pub fn total_interaction_count(templates: &[InteractionTemplate], sentinels: &[SentinelId], n_competitive_cells: usize) -> usize {
    templates
        .iter()
        .map(|t| t.feature_count(sentinels, n_competitive_cells))
        .sum()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    /// A per-Sentinel template yields exactly one instance for each registered
    /// Sentinel and none at all when there are none, unaffected by how many
    /// competitive cells exist. A template is a rule rather than a feature, and
    /// the population it ranges over is what turns it into concrete slots — so
    /// registering a Sentinel widens the vector by one per such template.
    ///
    /// ´claim:feature:a-per-sentinel-template-yields-one-instance-for-each-registered-sentinel´
    /// ´test:unit:type1-feature-count-scales-with-sentinels´
    #[test]
    fn type1_feature_count_scales_with_sentinels() {
        let template = InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0));
        assert_eq!(template.feature_count(&ids(0), 10), 0);
        assert_eq!(template.feature_count(&ids(1), 10), 1);
        assert_eq!(template.feature_count(&ids(5), 10), 5);
        assert_eq!(template.feature_count(&ids(8), 10), 8);
    }

    /// The named pair costs exactly one dimension, whatever the population.
    /// It names two Sentinels, so it is present while both are registered and
    /// absent otherwise, and adding a hundred more Sentinels adds nothing to
    /// it. That is the whole difference between it and the wildcard: at the
    /// reference configuration's eight Sentinels a wildcard buys twenty-eight
    /// coordinates and a named pair buys one, so a declaration that meant the
    /// named pair and got the wildcard would move every learned weight, the
    /// factorisation cost, and the dimension identity the reference tables
    /// state (´def:feature:template-named-pair´).
    ///
    /// ´claim:feature:a-named-pair-template-yields-exactly-one-instance-while-both-its-sentinels-are-registered´
    /// ´test:unit:type2-feature-count-is-one-named-pair´
    #[test]
    fn type2_feature_count_is_one_named_pair() {
        let template = InteractionTemplate::type2(SentinelId(1), SentinelId(3), 0);
        assert_eq!(template.feature_count(&[], 10), 0, "neither named Sentinel present");
        assert_eq!(
            template.feature_count(&[SentinelId(1)], 10),
            0,
            "one of the two is not a pair"
        );
        assert_eq!(
            template.feature_count(&[SentinelId(1), SentinelId(3)], 10),
            1,
            "both present is exactly one dimension"
        );
        assert_eq!(
            template.feature_count(&ids(8), 10),
            1,
            "and eight Sentinels is still exactly one"
        );
        assert_eq!(
            template.feature_count(&[SentinelId(0), SentinelId(2)], 10),
            0,
            "two Sentinels that are not the named two buy nothing"
        );
    }

    /// The wildcard yields one instance per unordered pair, so it stays empty
    /// at zero and at one Sentinel and then grows quadratically —
    /// twenty-eight instances at eight Sentinels. A pair feature has nothing
    /// to say until there are two parties to it, and the quadratic growth is
    /// the cost that makes the wildcard the expensive kind to declare
    /// (´def:feature:template-wildcard´). It is the third template kind, not
    /// the second: the position matters, because the second is the named pair
    /// and the two differ by a factor of twenty-eight at the reference
    /// configuration.
    ///
    /// ´claim:feature:a-pair-template-yields-one-instance-per-unordered-pair-and-none-below-two-sentinels´
    /// ´test:unit:type3-feature-count-is-combinations´
    #[test]
    fn type3_feature_count_is_combinations() {
        let template = InteractionTemplate::type3(0);
        assert_eq!(template.feature_count(&ids(0), 10), 0); // C(0,2) = 0
        assert_eq!(template.feature_count(&ids(1), 10), 0); // C(1,2) = 0
        assert_eq!(template.feature_count(&ids(2), 10), 1); // C(2,2) = 1
        assert_eq!(template.feature_count(&ids(3), 10), 3); // C(3,2) = 3
        assert_eq!(template.feature_count(&ids(8), 10), 28); // C(8,2) = 28
    }

    /// A template whose operands are both fleet-wide yields exactly one
    /// instance, with an empty population as much as with a hundred Sentinels
    /// and a hundred cells. Neither operand belongs to any entity, so there is
    /// nothing for the template to range over and its slot never moves as the
    /// population churns.
    ///
    /// ´claim:feature:a-template-over-two-fleet-wide-operands-yields-exactly-one-instance-whatever-the-population´
    /// ´test:unit:type4-feature-count-always-one´
    #[test]
    fn type4_feature_count_always_one() {
        let template = InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(0));
        assert_eq!(template.feature_count(&ids(0), 0), 1);
        assert_eq!(template.feature_count(&ids(100), 100), 1);
    }

    /// A competitive-cell template ranges over cells rather than Sentinels: its
    /// instance count follows the total number of competitive cells and ignores
    /// the Sentinel population entirely, held here at eight throughout. Each
    /// template names the one population it depends on, so cell entry and exit
    /// resize only the templates that actually concern cells.
    ///
    /// ´claim:feature:a-competitive-cell-template-yields-one-instance-per-cell-and-ignores-the-sentinel-population´
    /// ´test:unit:type5-feature-count-scales-with-cells´
    #[test]
    fn type5_feature_count_scales_with_cells() {
        let template = InteractionTemplate::type5(FeatureSelector::AggregateFeature(0));
        assert_eq!(template.feature_count(&ids(8), 0), 0);
        assert_eq!(template.feature_count(&ids(8), 10), 10);
        assert_eq!(template.feature_count(&ids(8), 20), 20);
    }

    /// The interaction block's width is simply the sum of what each declared
    /// template generates under the current population — five per-Sentinel
    /// templates across eight Sentinels giving forty slots. Templates do not
    /// share or overlap instances, so an operator can size the block from the
    /// template list and the population counts alone.
    ///
    /// ´claim:feature:the-interaction-block-width-is-the-sum-of-what-each-template-generates´
    /// ´test:unit:total-interaction-count-formula´
    #[test]
    fn total_interaction_count_formula() {
        // Reference config: 8 Sentinels, 20 competitive cells, 5 Type-1 templates
        let templates = vec![
            InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0)),
            InteractionTemplate::type1(1, FeatureSelector::AggregateFeature(1)),
            InteractionTemplate::type1(2, FeatureSelector::AggregateFeature(2)),
            InteractionTemplate::type1(3, FeatureSelector::AggregateFeature(3)),
            InteractionTemplate::type1(4, FeatureSelector::AggregateFeature(4)),
        ];
        // 5 templates × 8 Sentinels = 40 features
        assert_eq!(total_interaction_count(&templates, &ids(8), 20), 40);
    }

    /// An interaction instance is identified by its template together with the
    /// entity it was generated for: two instances of the same template for
    /// different Sentinels are different identities, and the same template and
    /// Sentinel names the same one. Instances are looked up by identity to find
    /// their index, so identity has to survive a rebuild rather than depending
    /// on where the instance happened to land last time.
    ///
    /// ´claim:feature:an-interaction-identity-is-its-template-together-with-the-entity-it-belongs-to´
    /// ´test:unit:interaction-id-equality´
    #[test]
    fn interaction_id_equality() {
        let id1 = InteractionId::per_sentinel(0, SentinelId(1));
        let id2 = InteractionId::per_sentinel(0, SentinelId(1));
        let id3 = InteractionId::per_sentinel(0, SentinelId(2));
        assert_eq!(id1, id2);
        assert_ne!(id1, id3);
    }

    /// A pair identity is not symmetric: naming the same two Sentinels in the
    /// opposite order produces a different identity. The identity does not
    /// canonicalise the pair for its callers, so a pair must always be built in
    /// one settled order — earlier registration first — or a lookup will miss
    /// the instance that was generated.
    ///
    /// ´claim:feature:a-pair-identity-distinguishes-the-two-orderings-so-pairs-must-be-built-in-one-settled-order´
    /// ´test:unit:interaction-id-per-pair-ordering´
    #[test]
    fn interaction_id_per_pair_ordering() {
        let id1 = InteractionId::per_pair(0, SentinelId(1), SentinelId(2));
        let id2 = InteractionId::per_pair(0, SentinelId(2), SentinelId(1));
        // Different pairs (order matters for identity)
        assert_ne!(id1, id2);
    }

    /// A per-cell identity carries the dimension alongside the cell, both
    /// recoverable from it intact. Cells are numbered within their own
    /// dimension, so the cell alone would not distinguish two cells that happen
    /// to occupy the same position in different dimensions.
    ///
    /// ´claim:feature:a-per-cell-identity-carries-the-dimension-alongside-the-cell´
    /// ´test:unit:interaction-id-per-cell´
    #[test]
    fn interaction_id_per_cell() {
        let cell = CompetitiveCellId::new(0, 8);
        let id = InteractionId::per_cell(0, DimensionId(1), cell);
        match id {
            InteractionId::PerCell {
                template,
                dimension,
                cell: c,
            } => {
                assert_eq!(template, 0);
                assert_eq!(dimension, DimensionId(1));
                assert_eq!(c, cell);
            }
            _ => panic!("Expected PerCell variant"),
        }
    }

    // ─────────────────────────────────────────────────────────────────────────
    // compute_interactions() tests
    // ─────────────────────────────────────────────────────────────────────────

    use indexmap::IndexMap;

    use crate::feature::dimension_map::DimensionMap;

    /// Helper to create an `IndexMap<SentinelId, ()>` from a slice of IDs.
    /// `n` Sentinel identifiers, for the population-dependent counts.
    fn ids(n: u32) -> Vec<SentinelId> {
        (0..n).map(SentinelId).collect()
    }

    fn sentinels(ids: &[u32]) -> IndexMap<SentinelId, ()> {
        ids.iter().map(|&id| (SentinelId(id), ())).collect()
    }

    /// Creates identity dimensions from IDs.
    fn dims(ids: &[u32]) -> Vec<DimensionId> {
        ids.iter().map(|&id| DimensionId(id)).collect()
    }

    /// Creates competitive cells for a set of dimensions.
    fn cells(entries: &[(u32, &[CompetitiveCellId])]) -> IndexMap<DimensionId, Vec<CompetitiveCellId>> {
        entries.iter().map(|&(id, cs)| (DimensionId(id), cs.to_vec())).collect()
    }

    fn make_cells(n: usize) -> Vec<CompetitiveCellId> {
        (0..n)
            .map(|i| CompetitiveCellId::new(u128::try_from(i * 256).unwrap(), 8))
            .collect()
    }

    /// An interaction slot holds the plain product of its two operands, each
    /// read from wherever it lives in the vector — here a Sentinel's feature at
    /// an offset inside that Sentinel's slot, past the occupancy indicator,
    /// times an aggregate read by absolute index. The product is what lets a
    /// linear model express a conjunction, and it is formed from the vector
    /// itself rather than from any private copy of the inputs.
    ///
    /// ´claim:feature:an-interaction-slot-holds-the-plain-product-of-its-two-operands-read-from-their-own-positions´
    /// ´test:unit:type1-product-computed´
    #[test]
    fn type1_product_computed() {
        // Setup: 1 Sentinel, Type-1 template reads slot offset 0 and context at index 1.
        // Sentinel slot starts at 16 (bias=1 + agg=15), occupancy at 16, feature at 17.
        let templates = vec![InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0))]; // offset=0, context=agg[0]
        let dm = DimensionMap::rebuild(0, &sentinels(&[1]), &[], &IndexMap::new(), &[], templates);

        let mut phi = vec![0.0_f64; dm.p];

        // Set agg[0] (context) to 2.5.
        phi[dm.agg_range.start] = 2.5;

        // Set Sentinel slot feature to 0.8 (offset 0, after occupancy).
        let slot = &dm.sentinel_slots[&SentinelId(1)];
        phi[slot.start + 1] = 0.8;

        compute_interactions(&mut phi, &dm);

        // Find the interaction index.
        let int_id = InteractionId::per_sentinel(0, SentinelId(1));
        let idx = dm.interaction_indices[&int_id];

        // Product: 0.8 × 2.5 = 2.0
        assert!((phi[idx] - 2.0).abs() < 1e-14, "Type-1 product = {}", phi[idx]);
    }

    /// A Sentinel contributing nothing zeroes its own interaction however loud
    /// the shared operand is. Zero is the identity-destroying element of a
    /// product, so a non-reporting Sentinel cannot borrow significance from a
    /// context feature it had no part in raising — the conjunction requires both
    /// halves.
    ///
    /// ´claim:feature:a-sentinel-contributing-nothing-zeroes-its-own-interaction-however-loud-the-shared-operand´
    /// ´test:unit:type1-absent-sentinel-zeros´
    #[test]
    fn type1_absent_sentinel_zeros() {
        let templates = vec![InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0))];
        let dm = DimensionMap::rebuild(0, &sentinels(&[1]), &[], &IndexMap::new(), &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        phi[dm.agg_range.start] = 2.5; // context
        // Sentinel features = 0 (absent/not reporting)

        compute_interactions(&mut phi, &dm);

        let int_id = InteractionId::per_sentinel(0, SentinelId(1));
        let idx = dm.interaction_indices[&int_id];

        // Product: 0 × 2.5 = 0
        assert!((phi[idx]).abs() < 1e-14, "Absent Sentinel → 0 product");
    }

    /// A template over two fleet-wide operands works the same way, one reading
    /// its aggregate by offset within the aggregate block and the other by
    /// absolute index into the vector — the two addressing modes resolving to
    /// the same block here. The multiplication does not care what kind of
    /// quantity either operand is; only how each is addressed differs between
    /// template kinds.
    ///
    /// (´claim:feature:an-interaction-slot-holds-the-plain-product-of-its-two-operands-read-from-their-own-positions´)
    /// ´test:unit:type4-product-computed´
    #[test]
    fn type4_product_computed() {
        // Type-4: agg[0] × context[1] = agg[0] × agg[1]
        let templates = vec![InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(1))]; // agg_offset=0, context_idx=2
        let dm = DimensionMap::rebuild(0, &IndexMap::new(), &[], &IndexMap::new(), &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        phi[dm.agg_range.start] = 3.0; // agg[0]
        phi[dm.agg_range.start + 1] = 0.6; // agg[1] = context at index 2

        compute_interactions(&mut phi, &dm);

        let int_id = InteractionId::fixed(0);
        let idx = dm.interaction_indices[&int_id];

        // Product: 3.0 × 0.6 = 1.8
        assert!((phi[idx] - 1.8).abs() < 1e-14, "Type-4 product = {}", phi[idx]);
    }

    /// A competitive indicator acts as a gate on its context: an active cell's
    /// indicator stands at one, so its interaction carries the context value
    /// through unchanged. Multiplying by a zero-or-one indicator is how the
    /// model is given a context feature conditioned on membership, without
    /// needing a separate feature for the conditioned and unconditioned cases.
    ///
    /// ´claim:feature:a-competitive-indicator-gates-its-context-passing-it-through-when-the-cell-is-active´
    /// ´test:unit:type5-active-cell-product´
    #[test]
    fn type5_active_cell_product() {
        let c = make_cells(3);
        let dim_list = dims(&[1]);
        let comp = cells(&[(1, &c)]);

        // Type-5: indicator × context
        let templates = vec![InteractionTemplate::type5(FeatureSelector::AggregateFeature(0))]; // context = agg[0]
        let dm = DimensionMap::rebuild(0, &IndexMap::new(), &dim_list, &comp, &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        phi[dm.agg_range.start] = 2.5; // context

        // Set cell[0] indicator = 1.0 (active)
        let cell_idx = dm.competitive_indices[&DimensionId(1)][&c[0]];
        phi[cell_idx] = 1.0;

        compute_interactions(&mut phi, &dm);

        let int_id = InteractionId::per_cell(0, DimensionId(1), c[0]);
        let idx = dm.interaction_indices[&int_id];

        // Product: 1.0 × 2.5 = 2.5
        assert!((phi[idx] - 2.5).abs() < 1e-14, "Active cell product = {}", phi[idx]);
    }

    /// The other side of the gate: with every indicator at zero, every cell's
    /// interaction reads zero even though the context is strongly elevated. A
    /// cell that is not competing contributes nothing at all, rather than a
    /// muted version of the shared context, so the block stays silent about
    /// cells the request has no relationship to.
    ///
    /// (´claim:feature:a-competitive-indicator-gates-its-context-passing-it-through-when-the-cell-is-active´)
    /// ´test:unit:type5-inactive-cell-zeros´
    #[test]
    fn type5_inactive_cell_zeros() {
        let c = make_cells(3);
        let dim_list = dims(&[1]);
        let comp = cells(&[(1, &c)]);

        let templates = vec![InteractionTemplate::type5(FeatureSelector::AggregateFeature(0))];
        let dm = DimensionMap::rebuild(0, &IndexMap::new(), &dim_list, &comp, &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        phi[dm.agg_range.start] = 2.5;
        // All indicators = 0 (inactive)

        compute_interactions(&mut phi, &dm);

        for cell in &c {
            let int_id = InteractionId::per_cell(0, DimensionId(1), *cell);
            let idx = dm.interaction_indices[&int_id];
            assert!((phi[idx]).abs() < 1e-14, "Inactive cell → 0 product");
        }
    }

    /// With two templates of different kinds active over two Sentinels, each
    /// instance lands at the index its own identity maps to and carries its own
    /// operands' product: the two per-Sentinel instances differ from each other
    /// and both from the fleet-wide one. A single pass fills the whole block, so
    /// the identity-to-index map is the only thing keeping instances from
    /// overwriting one another.
    ///
    /// ´claim:feature:each-instance-is-written-to-the-index-its-own-identity-maps-to´
    /// ´test:unit:compute-interactions-writes-correct-positions´
    #[test]
    fn compute_interactions_writes_correct_positions() {
        // 2 Sentinels, mix of templates.
        let templates = vec![
            InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0)),
            InteractionTemplate::type4(2, FeatureSelector::AggregateFeature(3)),
        ];
        let dm = DimensionMap::rebuild(0, &sentinels(&[1, 2]), &[], &IndexMap::new(), &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        // Set up known values:
        phi[dm.agg_range.start] = 1.0; // agg[0]
        phi[dm.agg_range.start + 2] = 2.0; // agg[2] (for Type-4)
        phi[dm.agg_range.start + 3] = 3.0; // agg[3] (for Type-4 context)

        for (sid, slot) in &dm.sentinel_slots {
            phi[slot.start + 1] = f64::from(sid.0); // Use Sentinel ID as feature value
        }

        compute_interactions(&mut phi, &dm);

        // Type-1 for Sentinel 1: feature[0]=1.0 × agg[0]=1.0 = 1.0
        let idx1 = dm.interaction_indices[&InteractionId::per_sentinel(0, SentinelId(1))];
        assert!((phi[idx1] - 1.0).abs() < 1e-14);

        // Type-1 for Sentinel 2: feature[0]=2.0 × agg[0]=1.0 = 2.0
        let idx2 = dm.interaction_indices[&InteractionId::per_sentinel(0, SentinelId(2))];
        assert!((phi[idx2] - 2.0).abs() < 1e-14);

        // Type-4: agg[2]=2.0 × φ[4]=agg[3]=3.0 = 6.0
        let idx4 = dm.interaction_indices[&InteractionId::fixed(1)];
        assert!((phi[idx4] - 6.0).abs() < 1e-14);
    }

    /// Products are formed from the raw assembled values, before any
    /// standardisation touches the vector: operands of five and ten give fifty,
    /// not the near-zero a product of already-centred values would give. The
    /// ordering matters because a product of standardised terms is a different
    /// quantity from a standardised product, and the interaction slot is
    /// itself standardised afterwards as a feature in its own right.
    ///
    /// ´claim:feature:interaction-products-are-formed-from-raw-values-before-standardisation´
    /// ´test:unit:compute-interactions-unstandardised´
    #[test]
    fn compute_interactions_unstandardised() {
        // Verify interactions use unstandardised (raw) values, not standardised.
        // This test documents the contract: products are computed BEFORE standardisation.
        let templates = vec![InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0))];
        let dm = DimensionMap::rebuild(0, &sentinels(&[1]), &[], &IndexMap::new(), &[], templates);

        let mut phi = vec![0.0_f64; dm.p];
        phi[dm.agg_range.start] = 10.0; // Raw value, would be ~0 if standardised
        let slot = &dm.sentinel_slots[&SentinelId(1)];
        phi[slot.start + 1] = 5.0;

        compute_interactions(&mut phi, &dm);

        let int_id = InteractionId::per_sentinel(0, SentinelId(1));
        let idx = dm.interaction_indices[&int_id];

        // If standardised first (assume mean=10, var=1), product would be ~0.
        // With raw values: 5.0 × 10.0 = 50.0
        assert!((phi[idx] - 50.0).abs() < 1e-14);
    }
}
