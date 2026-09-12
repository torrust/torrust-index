// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Dimension map: structural tracking of feature-vector index ranges.
//!
//! # Tests
//!
//! Crate-level tests: `src/tests/dimension_map.rs`
//!
//! The `DimensionMap` tracks the layout of the feature vector: which indices
//! correspond to bias, aggregate features, identity features, signal features,
//! per-Sentinel slots, interactions, and competitive indicators.
//!
//! # Feature Vector Layout
//!
//! ```text
//! [bias(1) | agg(15) | id_dim(D×(8+3m)) | id_cross(8 if D>0) | signals(p_sig) | slots(n×(q+1)) | int(p_int) | competitive(Σ|E_d|)]
//! ```
//!
//! The full layout is the specification's (´def:feature:vector-structure´).
//! The bias term is always at index 0.
//! Aggregate features occupy indices 1–15. Per-dimension identity blocks
//! follow, one `8 + 3m` block per registered identity dimension. If at
//! least one identity dimension exists, a fixed 8-feature cross-dimension
//! block follows. Signal features follow, then
//! per-Sentinel slots (each of width `q + 1` where `q = 60 + 2*m_s`), then
//! interaction features (products of base features), then competitive
//! indicators (one per competitive cell across all dimensions).
//!
//! # Cross-References
//!
//! - (´dec:vector:block-order´) — that there is exactly one canonical block
//!   order, which this map is the resolver for
//! - (´dec:retention:monolithic-snapshot´) — the snapshot this map is composed
//!   into, built whole in one allocation
//! - (´def:feature:vector-structure´) — the eight block types and their fixed
//!   order
//! - (´tab:feature:dimension-formula´) — the block widths the total dimension
//!   is the sum of
//! - (´tab:feature:aggregate-block´) — the aggregate block's fifteen features
//! - (´sec:feature:interactions´) — the interaction templates and what they
//!   expand to
//! - (´def:extraction:slot´) — the per-Sentinel slot width

use indexmap::IndexMap;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

use crate::feature::interaction::{FeatureSelector, InteractionId, InteractionTemplate, InteractionTriple};
use crate::feature::permutation::SLOT_AXIS_PAIR_OFFSET;
use crate::identity::CompetitiveCellId;
use crate::types::{DimensionId, OutcomeAxisId, SCORING_AXIS_COUNT, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Constants
// ═══════════════════════════════════════════════════════════════════════════════

/// Number of global aggregate features.
///
/// 15 features: max/mean/std cell-level max z (3), per-axis max z (4),
/// per-axis concordance (4), cross-axis product (1), axis breadth (1),
/// coverage (1), max CUSUM (1). The extent the aggregate block occupies in
/// the layout, fixed for the life of the deployment
/// (´tab:feature:aggregate-block´).
///
/// ´const:assayer:aggregate-block-width´ (´alg:const:count´)
/// ´const:assayer:aggregate-block-width-count-15´
pub const AGGREGATE_FEATURE_COUNT: usize = 15;

/// Number of fixed structural and measurement features per identity dimension.
///
/// The eight written at assessment time, the fixed half of the `8 + 3m`
/// per-dimension block (´tab:keyspace:dimension-features´).
///
/// ´const:assayer:dimension-block-structural-width´ (´alg:const:count´)
/// ´const:assayer:dimension-block-structural-width-count-8´
pub const ID_DIM_BASE_FEATURE_COUNT: usize = 8;

/// Number of per-axis identity outcome features.
///
/// `mean_compressed_axis_ewma`, `mean_raw_axis_ewma`, `axis_stability` — the
/// three written at label time per registered axis
/// (´tab:keyspace:dimension-features´).
///
/// ´const:assayer:dimension-block-axis-stride´ (´alg:const:count´)
/// ´const:assayer:dimension-block-axis-stride-count-3´
pub const ID_AXIS_FEATURES_PER_AXIS: usize = 3;

/// Number of cross-dimension aggregate identity features.
///
/// The fixed block present exactly where at least one dimension is registered
/// (´tab:keyspace:cross-dimension-features´).
///
/// ´const:assayer:cross-dimension-block-width´ (´alg:const:count´)
/// ´const:assayer:cross-dimension-block-width-count-8´
pub const ID_CROSS_DIM_FEATURE_COUNT: usize = 8;

/// Number of anchor projection slots.
///
/// The anchor model uses a fixed 15-dimensional projection of the full
/// feature vector: 1 bias + 6 aggregate + 6 identity + 2 aggregate. The
/// projection is what holds that width fixed while the vector underneath it
/// changes (´def:dimension:anchor-projection´).
///
/// ´const:assayer:anchor-projection-width´ (´alg:const:count´)
/// ´const:assayer:anchor-projection-width-count-15´
const ANCHOR_PROJECTION_COUNT: usize = 15;

/// Offset of the first per-axis maximum z-score inside the aggregate block.
///
/// The block opens with the maximum, mean and standard deviation of the
/// cell-level maximum z, and the four per-axis maxima follow them
/// (´tab:feature:aggregate-block´).
///
/// ´const:assayer:aggregate-per-axis-max-z-offset´ (´alg:const:count´)
/// ´const:assayer:aggregate-per-axis-max-z-offset-count-3´
const AGGREGATE_PER_AXIS_MAX_Z_OFFSET: usize = 3;

/// Position of the computed per-axis maximum inside the anchor input.
///
/// The fourteenth of the fifteen, counting from one
/// (´def:dimension:anchor-projection´).
///
/// ´const:assayer:anchor-per-axis-max-z-index´ (´alg:const:count´)
/// ´const:assayer:anchor-per-axis-max-z-index-count-13´
const ANCHOR_PER_AXIS_MAX_Z_INDEX: usize = 13;

/// Position of the computed reporting-count logarithm inside the anchor input.
///
/// The fifteenth of the fifteen, counting from one
/// (´def:dimension:anchor-projection´).
///
/// ´const:assayer:anchor-reporting-count-index´ (´alg:const:count´)
/// ´const:assayer:anchor-reporting-count-index-count-14´
const ANCHOR_REPORTING_COUNT_INDEX: usize = 14;

/// Base per-Sentinel extraction width `q_base`.
///
/// `q = q_base + 2 * m_s` where `m_s` is the number of spatial outcome axes.
/// `q_base = 24 (z-scores) + 12 (coordination) + 8 (chain) + 12 (outcome) + 3 (ledger) + 1 (batch) = 60`,
/// the sum of the six extraction groups (´def:extraction:slot´).
///
/// ´const:assayer:extraction-base-width´ (´alg:const:count´)
/// ´const:assayer:extraction-base-width-count-60´
pub const SENTINEL_Q_BASE: usize = 60;

/// Per spatial-axis additional extraction features.
///
/// Each spatial outcome axis adds 2 features per Sentinel extraction
/// (compressed axis EWMA + raw axis EWMA), the `2 m_s` term of the extraction
/// width (´def:extraction:slot´).
///
/// ´const:assayer:extraction-spatial-stride´ (´alg:const:count´)
/// ´const:assayer:extraction-spatial-stride-count-2´
pub const SENTINEL_SPATIAL_FEATURES_PER_AXIS: usize = 2;

/// The occupancy indicator adds 1 feature per Sentinel slot.
///
/// Each slot in φ is `[occ | g_s]` with total width `q + 1`: the prefix that
/// tells a silent Sentinel from an offline one (´def:extraction:slot´).
///
/// ´const:assayer:slot-occupancy-prefix´ (´alg:const:count´)
/// ´const:assayer:slot-occupancy-prefix-count-1´
pub const SENTINEL_OCCUPANCY_WIDTH: usize = 1;

// ═══════════════════════════════════════════════════════════════════════════════
// IndexRange
// ═══════════════════════════════════════════════════════════════════════════════

/// A serializable half-open range `[start, end)` of feature-vector indices.
///
/// `std::ops::Range<usize>` does not implement `Serialize`/`Deserialize`,
/// so we use this lightweight wrapper for checkpoint persistence.
#[derive(Clone, Debug, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct IndexRange {
    /// Inclusive lower bound.
    pub start: usize,
    /// Exclusive upper bound.
    pub end: usize,
}

impl IndexRange {
    /// Creates a new half-open range `[start, end)`.
    #[inline]
    #[must_use]
    pub const fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    /// Returns a `std::ops::Range<usize>` for iteration / slicing.
    #[inline]
    #[must_use]
    pub const fn to_range(&self) -> std::ops::Range<usize> {
        self.start..self.end
    }

    /// Number of elements in the range.
    #[inline]
    #[must_use]
    pub const fn len(&self) -> usize {
        self.end - self.start
    }

    /// Returns `true` when `start == end`.
    #[inline]
    #[must_use]
    pub const fn is_empty(&self) -> bool {
        self.start == self.end
    }
}

impl From<std::ops::Range<usize>> for IndexRange {
    #[inline]
    fn from(r: std::ops::Range<usize>) -> Self {
        Self {
            start: r.start,
            end: r.end,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// DimensionMap
// ═══════════════════════════════════════════════════════════════════════════════

/// Structural map of feature-vector index ranges.
///
/// Tracks which indices in the `p`-dimensional feature vector correspond
/// to each logical block: bias, aggregates, identity dimensions, signals,
/// Sentinel slots, interactions, and competitive indicators.
///
/// # Invariant
///
/// `p > 0` (at minimum, the bias term exists).
#[derive(Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct DimensionMap {
    /// Index of the bias term (always 0).
    pub bias_idx: usize,
    /// Range of global aggregate features — fifteen of them
    /// (´tab:feature:aggregate-block´).
    pub agg_range: IndexRange,
    /// Per-dimension identity feature ranges, `8 + 3m` per dimension
    /// (´tab:keyspace:dimension-features´).
    pub id_dim_ranges: IndexMap<DimensionId, IndexRange>,
    /// Range of cross-dimension aggregate identity features
    /// (´tab:keyspace:cross-dimension-features´).
    ///
    /// Present whenever at least one identity dimension is registered. 8 features.
    pub id_cross_dim_range: Option<IndexRange>,
    /// Per-axis offset inside each identity dimension block
    /// (´conv:dimension:axis-index´).
    ///
    /// Axis `a` occupies `[range.start + offset, range.start + offset + 3)`
    /// for every registered identity dimension.
    pub id_axis_offsets: IndexMap<OutcomeAxisId, usize>,
    /// Per-axis offset of an outcome-memory pair inside every Sentinel slot
    /// (´def:extraction:slot´).
    ///
    /// Axis `a` occupies `[range.start + offset, range.start + offset + 2)`
    /// for every registered Sentinel. The slot tail's counterpart to
    /// `id_axis_offsets`, and named for the same reason: the pairs are as
    /// dynamic as the identity blocks' per-axis triples, so a reader holding a
    /// pair asks which axis owns it rather than counting ranks from the
    /// beginning of the tail. A rank is only stable while no axis ahead of it
    /// is retired; an identifier is stable under every lifecycle event.
    ///
    /// The entries are in layout order, so `m_s` is this map's length and the
    /// keys read in order are the tail as an extraction fills it — which is
    /// what lets a stored pair taken under one ordering be matched to its own
    /// position under another.
    pub slot_axis_offsets: IndexMap<OutcomeAxisId, usize>,
    /// Range of signal features.
    pub sig_range: IndexRange,
    /// Per-Sentinel slot ranges, each slot `q + 1` features
    /// (´def:extraction:slot´).
    pub sentinel_slots: IndexMap<SentinelId, IndexRange>,
    /// Range of interaction features (´sec:feature:interactions´).
    ///
    /// Products of base features computed at assembly time.
    #[cfg_attr(not(feature = "serde"), allow(dead_code))] // Justified: read by the checkpoint payload and the test surface
    pub interaction_range: IndexRange,
    /// Per-interaction feature indices.
    ///
    /// Each interaction template instance occupies one index. Ordered by
    /// template registration, then by the dynamic entity (Sentinel ID,
    /// Sentinel pair, or competitive cell) within each template.
    pub interaction_indices: IndexMap<InteractionId, usize>,
    /// Declared interaction templates (´sec:feature:interactions´).
    ///
    /// The pipeline's first stage: the symbolic templates as the host wrote
    /// them, retained so a snapshot can rebuild the later stages without the
    /// host (´alg:dimension:compilation-pipeline´).
    pub interaction_templates: Vec<InteractionTemplate>,

    /// Compiled interaction triples.
    ///
    /// The pipeline's third stage: one triple per expanded feature, emitted
    /// beside the resolution map on every rebuild and iterated by feature
    /// assembly (´alg:dimension:compilation-pipeline´). An instance whose
    /// operand names nothing in the current layout emits no triple.
    #[cfg_attr(not(feature = "serde"), allow(dead_code))] // Justified: read by the checkpoint payload and the test surface
    pub resolved_interactions: Vec<InteractionTriple>,
    /// Range of competitive indicator features
    /// (´def:dimension:competitive-range´).
    ///
    /// One binary indicator per competitive cell across all dimensions.
    #[cfg_attr(not(feature = "serde"), allow(dead_code))] // Justified: read by the checkpoint payload and the test surface
    pub competitive_range: IndexRange,
    /// Per-dimension competitive cell index mappings.
    ///
    /// Maps each competitive cell to its absolute index in the feature vector.
    pub competitive_indices: IndexMap<DimensionId, IndexMap<CompetitiveCellId, usize>>,
    /// Per-Sentinel extraction width `q` (excludes occupancy indicator).
    ///
    /// `q = 60 + 2 * m_s` where `m_s` is the spatial outcome axis count.
    #[cfg_attr(not(feature = "serde"), allow(dead_code))] // Justified: read by the checkpoint payload and the test surface
    pub sentinel_feature_width: usize,
    /// Anchor projection indices (´def:dimension:anchor-projection´).
    ///
    /// Maps the 15 anchor-model features to their positions in the full
    /// feature vector. `None` entries indicate identity features not yet
    /// available because no identity dimension is registered.
    ///
    /// TODO(2026-05-22) ´todo:code:replace-this-legacy-gathered-projection´: replace this legacy gathered projection
    /// — the projection whose fixed width is the point (´def:dimension:anchor-projection´) —
    /// with `AnchorProjection { gather: [Option<usize>; 13],
    /// per_axis_max_z_start }` and compute anchor features 13–14 inline.
    pub anchor_projection_indices: [Option<usize>; ANCHOR_PROJECTION_COUNT],
    /// Total dimensionality of the feature vector.
    pub p: usize,
}

impl DimensionMap {
    /// Rebuilds the dimension map from the current entity configuration.
    ///
    /// Uses a sequential cursor to assign index ranges through the blocks in
    /// the order the structure fixes (´def:feature:vector-structure´):
    /// 1. Bias (1)
    /// 2. Global aggregates (15)
    /// 3. Per-dimension identity blocks (`8 + 3m` per dimension)
    /// 4. Cross-dimension identity aggregate block (8, if `D > 0`)
    /// 5. Signal features (`p_sig`)
    /// 6. Per-Sentinel slots (`q + 1` per Sentinel)
    /// 7. Interaction features (template-dependent)
    /// 9. Competitive indicators (1 per competitive cell)
    ///
    /// # Arguments
    ///
    /// * `p_sig` — Number of signal features
    /// * `sentinels` — Registered Sentinel IDs (ordering preserved)
    /// * `identity_dimensions` — Registered identity dimension IDs
    /// * `competitive_cells` — Per-dimension competitive cells
    /// * `spatial_axis_ids` — Ordered spatial outcome axis IDs, whose length is `m_s`
    /// * `interaction_templates` — Declared interaction templates
    #[must_use]
    pub fn rebuild(
        p_sig: usize,
        sentinels: &IndexMap<SentinelId, ()>,
        identity_dimensions: &[DimensionId],
        competitive_cells: &IndexMap<DimensionId, Vec<CompetitiveCellId>>,
        spatial_axis_ids: &[OutcomeAxisId],
        interaction_templates: Vec<InteractionTemplate>,
    ) -> Self {
        Self::rebuild_with_axes(
            p_sig,
            sentinels,
            identity_dimensions,
            competitive_cells,
            spatial_axis_ids,
            &[],
            interaction_templates,
        )
    }

    /// Rebuilds the dimension map from the current entity configuration,
    /// including registered outcome-axis identities for the identity block.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn rebuild_with_axes(
        p_sig: usize,
        sentinels: &IndexMap<SentinelId, ()>,
        identity_dimensions: &[DimensionId],
        competitive_cells: &IndexMap<DimensionId, Vec<CompetitiveCellId>>,
        spatial_axis_ids: &[OutcomeAxisId],
        outcome_axis_ids: &[OutcomeAxisId],
        interaction_templates: Vec<InteractionTemplate>,
    ) -> Self {
        let mut cursor: usize = 0;

        // Block 1: Bias
        let bias_idx = cursor;
        cursor += 1;

        // Block 2: Global aggregates (15 features)
        let agg_start = cursor;
        cursor += AGGREGATE_FEATURE_COUNT;
        let agg_range = IndexRange::new(agg_start, cursor);

        // Block 3: Per-dimension identity features (8 + 3m per dim)
        let mut id_axis_offsets = IndexMap::new();
        for (axis_index, &axis_id) in outcome_axis_ids.iter().enumerate() {
            id_axis_offsets.insert(axis_id, ID_DIM_BASE_FEATURE_COUNT + axis_index * ID_AXIS_FEATURES_PER_AXIS);
        }

        let id_dim_width = ID_DIM_BASE_FEATURE_COUNT + outcome_axis_ids.len() * ID_AXIS_FEATURES_PER_AXIS;
        let mut id_dim_ranges = IndexMap::new();
        for &dim_id in identity_dimensions {
            let start = cursor;
            cursor += id_dim_width;
            id_dim_ranges.insert(dim_id, IndexRange::new(start, cursor));
        }

        // Block 4: Cross-dimension aggregate features (8, if D > 0)
        let id_cross_dim_range = if identity_dimensions.is_empty() {
            None
        } else {
            let start = cursor;
            cursor += ID_CROSS_DIM_FEATURE_COUNT;
            Some(IndexRange::new(start, cursor))
        };

        // Block 5: Signal features
        let sig_start = cursor;
        cursor += p_sig;
        let sig_range = IndexRange::new(sig_start, cursor);

        // Block 6: Per-Sentinel slots
        // Each slot = occupancy (1) + extraction (q).
        // q = SENTINEL_Q_BASE + SENTINEL_SPATIAL_FEATURES_PER_AXIS * m_s
        //
        // The tail's axes are named as they are laid out. `m_s` stays the
        // caller's list length, because that is the length the extraction is
        // built to (´def:extraction:slot´); the offsets are the same list read
        // as positions, so the two are one statement made twice rather than
        // two counts that could drift.
        let mut slot_axis_offsets = IndexMap::new();
        for (axis_index, &axis_id) in spatial_axis_ids.iter().enumerate() {
            slot_axis_offsets.insert(
                axis_id,
                SLOT_AXIS_PAIR_OFFSET + axis_index * SENTINEL_SPATIAL_FEATURES_PER_AXIS,
            );
        }
        debug_assert_eq!(
            slot_axis_offsets.len(),
            spatial_axis_ids.len(),
            "a spatial axis named twice would own one pair and be paid for with two",
        );
        let sentinel_feature_width = SENTINEL_Q_BASE + SENTINEL_SPATIAL_FEATURES_PER_AXIS * spatial_axis_ids.len();
        let slot_width = SENTINEL_OCCUPANCY_WIDTH + sentinel_feature_width;
        let mut sentinel_slots = IndexMap::new();
        for &sentinel_id in sentinels.keys() {
            let start = cursor;
            cursor += slot_width;
            sentinel_slots.insert(sentinel_id, IndexRange::new(start, cursor));
        }

        // Block 7: Interaction features
        // TODO(2026-05-22) ´todo:code:reorder-this-into-the-canonical´: reorder this into the canonical
        // four-phase rebuild: count interactions, assign competitive
        // indicators, then resolve interactions without placeholders/fixup,
        // which is the staging the pipeline states
        // (´alg:dimension:compilation-pipeline´).
        // Count competitive cells for Type-5 templates
        let _n_competitive_cells: usize = competitive_cells.values().map(Vec::len).sum();
        let _n_sentinels = sentinels.len();

        let int_start = cursor;
        let mut interaction_indices = IndexMap::new();

        for (template_idx, template) in interaction_templates.iter().enumerate() {
            match template {
                InteractionTemplate::Type1 { .. } => {
                    // One feature per Sentinel
                    for &sentinel_id in sentinels.keys() {
                        let int_id = InteractionId::per_sentinel(template_idx, sentinel_id);
                        interaction_indices.insert(int_id, cursor);
                        cursor += 1;
                    }
                }
                InteractionTemplate::Type2 {
                    sentinel_a, sentinel_b, ..
                } => {
                    // Exactly one feature, and only while both named Sentinels
                    // are registered (´def:feature:template-named-pair´). The
                    // identifier takes them in registration order.
                    if let (Some(index_a), Some(index_b)) =
                        (sentinels.get_index_of(sentinel_a), sentinels.get_index_of(sentinel_b))
                    {
                        let (first, second) = if index_a <= index_b {
                            (*sentinel_a, *sentinel_b)
                        } else {
                            (*sentinel_b, *sentinel_a)
                        };
                        let int_id = InteractionId::per_pair(template_idx, first, second);
                        interaction_indices.insert(int_id, cursor);
                        cursor += 1;
                    }
                }
                InteractionTemplate::Type3 { .. } => {
                    // One feature per unordered Sentinel pair, C(n,2)
                    // (´def:feature:template-wildcard´) — the quadratic block,
                    // which is this template's cost and no other's.
                    let sentinel_ids: Vec<_> = sentinels.keys().copied().collect();
                    for i in 0..sentinel_ids.len() {
                        for j in (i + 1)..sentinel_ids.len() {
                            let int_id = InteractionId::per_pair(template_idx, sentinel_ids[i], sentinel_ids[j]);
                            interaction_indices.insert(int_id, cursor);
                            cursor += 1;
                        }
                    }
                }
                InteractionTemplate::Type4 { .. } => {
                    // One fixed feature
                    let int_id = InteractionId::fixed(template_idx);
                    interaction_indices.insert(int_id, cursor);
                    cursor += 1;
                }
                InteractionTemplate::Type5 { .. } => {
                    // One feature per competitive cell
                    for &dim_id in identity_dimensions {
                        if let Some(cells) = competitive_cells.get(&dim_id) {
                            for &cell in cells {
                                let int_id = InteractionId::per_cell(template_idx, dim_id, cell);
                                interaction_indices.insert(int_id, cursor);
                                cursor += 1;
                            }
                        }
                    }
                }
            }
        }
        let interaction_range = IndexRange::new(int_start, cursor);

        // Block 8: Competitive indicators (1 per competitive cell)
        let comp_start = cursor;
        let mut competitive_indices = IndexMap::new();
        for &dim_id in identity_dimensions {
            let cells = competitive_cells.get(&dim_id);
            let mut cell_map = IndexMap::new();
            if let Some(cells) = cells {
                for &cell in cells {
                    cell_map.insert(cell, cursor);
                    cursor += 1;
                }
            }
            competitive_indices.insert(dim_id, cell_map);
        }
        let competitive_range = IndexRange::new(comp_start, cursor);

        // Build anchor projection indices (´def:dimension:anchor-projection´).
        //
        // The anchor model's 15 features map to positions in φ:
        //   [0]:     bias
        //   [1–6]:   6 selected aggregate features
        //   [7–12]:  6 cross-dimension identity aggregates (None if D = 0)
        //   [13–14]: computed, not gathered — see below
        let mut anchor_projection_indices = [None; ANCHOR_PROJECTION_COUNT];
        // Index 0: bias
        anchor_projection_indices[0] = Some(bias_idx);
        // Indices 1–6: selected aggregate features (first 6 of the 15)
        for i in 0..6 {
            anchor_projection_indices[1 + i] = Some(agg_start + i);
        }
        // Indices 7–12: cross-dimension identity features.
        if let Some(ref cross) = id_cross_dim_range {
            anchor_projection_indices[7] = Some(cross.start); // max suspicion
            anchor_projection_indices[8] = Some(cross.start + 3); // max adverse rate
            anchor_projection_indices[9] = Some(cross.start + 6); // has any competitive cell
            anchor_projection_indices[10] = Some(cross.start + 4); // max |compressed valence|
            anchor_projection_indices[11] = Some(cross.start + 5); // max |raw valence|
            anchor_projection_indices[12] = Some(cross.start + 2); // max volatility
        }
        // Indices 13–14: computed at projection time, not gathered.
        //
        // The projection fixes fifteen inputs and says in terms that thirteen
        // are gathered and two are computed: the fourteenth is the maximum
        // over the four per-axis maximum z-score positions and the fifteenth
        // the logarithm of one plus the reporting Sentinel count
        // (´def:dimension:anchor-projection´). Gathering the aggregate block's
        // last two positions put coverage and the maximum cumulative sum there
        // instead — two quantities the projection table does not name, one
        // already a fraction and the other a cumulative sum where the corpus
        // asks for a maximum and a logarithm.
        //
        // They stay `None` here, which is what excludes them from the sister
        // model's projected quadratic form: they name no coordinate of φ, so
        // there is no shared-subspace position for them to contribute.

        let p = cursor;
        debug_assert!(p > 0, "DimensionMap must have p > 0");

        let mut map = Self {
            bias_idx,
            agg_range,
            id_dim_ranges,
            id_cross_dim_range,
            id_axis_offsets,
            slot_axis_offsets,
            sig_range,
            sentinel_slots,
            interaction_range,
            interaction_indices,
            interaction_templates,
            resolved_interactions: Vec::new(),
            competitive_range,
            competitive_indices,
            sentinel_feature_width,
            anchor_projection_indices,
            p,
        };

        // Stage 3: compile the flat triples beside the resolution map
        // (´alg:dimension:compilation-pipeline´). Compiled from the finished
        // map rather than from the cursor arithmetic that built it, so the
        // triples resolve their operands through the same `resolve_context`
        // every other reader uses — one authority for what an operand names,
        // not a second copy of it beside the first.
        map.resolved_interactions = map.compile_interactions();
        map
    }

    /// Compiles one index triple per expanded interaction feature.
    ///
    /// A second walk over the declarations, reading the indices the rebuild
    /// has just assigned. An instance whose operand names nothing in the
    /// current layout emits no triple, and the product loop leaves its output
    /// position at zero.
    fn compile_interactions(&self) -> Vec<InteractionTriple> {
        let mut triples: Vec<InteractionTriple> = Vec::with_capacity(self.interaction_indices.len());
        let slot_position = |sentinel_id: SentinelId, offset: usize| -> Option<usize> {
            let range = self.sentinel_slots.get(&sentinel_id)?;
            let idx = range.start + SENTINEL_OCCUPANCY_WIDTH + offset;
            (idx < range.end).then_some(idx)
        };
        let mut emit = |output: Option<&usize>, a: Option<usize>, b: Option<usize>| {
            if let (Some(&output), Some(operand_a), Some(operand_b)) = (output, a, b) {
                triples.push(InteractionTriple {
                    output,
                    operand_a,
                    operand_b,
                });
            }
        };

        for (template_idx, template) in self.interaction_templates.iter().enumerate() {
            match template {
                InteractionTemplate::Type1 {
                    sentinel_feature_offset,
                    context,
                } => {
                    for &sentinel_id in self.sentinel_slots.keys() {
                        let int_id = InteractionId::per_sentinel(template_idx, sentinel_id);
                        emit(
                            self.interaction_indices.get(&int_id),
                            slot_position(sentinel_id, *sentinel_feature_offset),
                            self.resolve_context(*context),
                        );
                    }
                }
                InteractionTemplate::Type2 {
                    sentinel_a,
                    sentinel_b,
                    feature_offset,
                } => {
                    if let (Some(index_a), Some(index_b)) = (
                        self.sentinel_slots.get_index_of(sentinel_a),
                        self.sentinel_slots.get_index_of(sentinel_b),
                    ) {
                        let (first, second) = if index_a <= index_b {
                            (*sentinel_a, *sentinel_b)
                        } else {
                            (*sentinel_b, *sentinel_a)
                        };
                        let int_id = InteractionId::per_pair(template_idx, first, second);
                        emit(
                            self.interaction_indices.get(&int_id),
                            slot_position(*sentinel_a, *feature_offset),
                            slot_position(*sentinel_b, *feature_offset),
                        );
                    }
                }
                InteractionTemplate::Type3 { feature_offset } => {
                    let sentinel_ids: Vec<_> = self.sentinel_slots.keys().copied().collect();
                    for i in 0..sentinel_ids.len() {
                        for j in (i + 1)..sentinel_ids.len() {
                            let int_id = InteractionId::per_pair(template_idx, sentinel_ids[i], sentinel_ids[j]);
                            emit(
                                self.interaction_indices.get(&int_id),
                                slot_position(sentinel_ids[i], *feature_offset),
                                slot_position(sentinel_ids[j], *feature_offset),
                            );
                        }
                    }
                }
                InteractionTemplate::Type4 { agg_offset, context } => {
                    let int_id = InteractionId::fixed(template_idx);
                    let first =
                        (self.agg_range.start + agg_offset < self.agg_range.end).then(|| self.agg_range.start + agg_offset);
                    emit(self.interaction_indices.get(&int_id), first, self.resolve_context(*context));
                }
                InteractionTemplate::Type5 { context } => {
                    for (&dim_id, cell_map) in &self.competitive_indices {
                        for (&cell_id, &indicator_idx) in cell_map {
                            let int_id = InteractionId::per_cell(template_idx, dim_id, cell_id);
                            emit(
                                self.interaction_indices.get(&int_id),
                                Some(indicator_idx),
                                self.resolve_context(*context),
                            );
                        }
                    }
                }
            }
        }
        triples
    }

    /// Rebuilds the dimension map without interaction templates.
    ///
    /// Convenience method for backward compatibility and simple configurations.
    #[must_use]
    pub fn rebuild_no_interactions(
        p_sig: usize,
        sentinels: &IndexMap<SentinelId, ()>,
        identity_dimensions: &[DimensionId],
        competitive_cells: &IndexMap<DimensionId, Vec<CompetitiveCellId>>,
        spatial_axis_ids: &[OutcomeAxisId],
    ) -> Self {
        Self::rebuild(
            p_sig,
            sentinels,
            identity_dimensions,
            competitive_cells,
            spatial_axis_ids,
            Vec::new(),
        )
    }

    /// Rebuilds the dimension map with outcome-axis identities and without
    /// interaction templates.
    #[must_use]
    #[allow(dead_code)] // Justified: test-only rebuild helper; production rebuilds carry interactions
    pub fn rebuild_with_axes_no_interactions(
        p_sig: usize,
        sentinels: &IndexMap<SentinelId, ()>,
        identity_dimensions: &[DimensionId],
        competitive_cells: &IndexMap<DimensionId, Vec<CompetitiveCellId>>,
        spatial_axis_ids: &[OutcomeAxisId],
        outcome_axis_ids: &[OutcomeAxisId],
    ) -> Self {
        Self::rebuild_with_axes(
            p_sig,
            sentinels,
            identity_dimensions,
            competitive_cells,
            spatial_axis_ids,
            outcome_axis_ids,
            Vec::new(),
        )
    }

    /// Resolves a context operand's selector to an index in the feature vector.
    ///
    /// This is the resolution the compilation pipeline recomputes on every map
    /// rebuild, and it is what makes a declaration survive a layout change
    /// (´alg:dimension:compilation-pipeline´). An identity operand resolves
    /// against the cross-dimension block wherever the rebuild has just put it,
    /// rather than against the index that block happened to occupy when the
    /// host wrote the declaration.
    ///
    /// `None` means the operand names nothing in the current layout: an
    /// identity operand while no dimension is registered — read as zero, as the
    /// anchor projection reads its absent identity entries — or an offset past
    /// its block's width. The entity-relative selectors resolve at expansion
    /// time against the Sentinel or cell being expanded, not here.
    #[must_use]
    pub fn resolve_context(&self, selector: FeatureSelector) -> Option<usize> {
        let within = |start: usize, end: usize, offset: usize| {
            let idx = start.checked_add(offset)?;
            (idx < end).then_some(idx)
        };
        match selector {
            FeatureSelector::AggregateFeature(offset) => within(self.agg_range.start, self.agg_range.end, offset),
            FeatureSelector::IdentityFeature(offset) => self
                .id_cross_dim_range
                .as_ref()
                .and_then(|cross| within(cross.start, cross.end, offset)),
            FeatureSelector::DeclaredSignal(offset) => within(self.sig_range.start, self.sig_range.end, offset),
            FeatureSelector::SlotFeature(_) | FeatureSelector::OwningCompetitiveIndicator => None,
        }
    }

    /// Returns the offset of an outcome axis within each per-dimension identity block.
    #[must_use]
    pub fn identity_axis_offset(&self, axis_id: OutcomeAxisId) -> Option<usize> {
        self.id_axis_offsets.get(&axis_id).copied()
    }

    /// Returns the offset, within every Sentinel slot, of a spatial outcome
    /// axis's outcome-memory pair.
    ///
    /// Measured from the slot's own start, so the pair occupies
    /// `[slot.start + offset, slot.start + offset + 2)` — the occupancy
    /// indicator that opens the slot is already counted in.
    ///
    /// `None` means the axis owns no pair in this layout: it was never spatial,
    /// or it has since been deregistered. A caller placing a stored pair reads
    /// that as "this pair has nowhere to go" rather than as "put it wherever
    /// the rank lands", which is what keeps a retirement from donating one
    /// axis's memory to another.
    #[must_use]
    pub fn slot_axis_offset(&self, axis_id: OutcomeAxisId) -> Option<usize> {
        self.slot_axis_offsets.get(&axis_id).copied()
    }

    /// Builds the anchor model's `p_a`-element input from a feature vector.
    ///
    /// Thirteen positions are gathered by index and two are computed: the
    /// fourteenth is the maximum over the four per-axis maximum z-score
    /// positions of the aggregate block, and the fifteenth is the logarithm of
    /// one plus the reporting Sentinel count
    /// (´def:dimension:anchor-projection´). The count is the anchor's one
    /// input for judging how much of the fleet its estimate rests on, and no
    /// position of the feature vector carries it, so the caller supplies it.
    #[must_use]
    pub fn extract_anchor_subvector(&self, phi: &[f64], p_a: usize, reporting_sentinels: usize) -> Vec<f64> {
        let at = |idx: Option<usize>| idx.map_or(0.0, |i| phi.get(i).copied().unwrap_or(0.0));
        let mut out: Vec<f64> = self.anchor_projection_indices.iter().take(p_a).map(|&i| at(i)).collect();

        let per_axis_max_z_start = self.agg_range.start + AGGREGATE_PER_AXIS_MAX_Z_OFFSET;
        if let Some(slot) = out.get_mut(ANCHOR_PER_AXIS_MAX_Z_INDEX) {
            let peak = (0..SCORING_AXIS_COUNT)
                .filter_map(|axis| phi.get(per_axis_max_z_start + axis).copied())
                .fold(f64::NEG_INFINITY, f64::max);
            *slot = if peak.is_finite() { peak } else { 0.0 };
        }
        if let Some(slot) = out.get_mut(ANCHOR_REPORTING_COUNT_INDEX) {
            #[allow(clippy::cast_precision_loss)] // Justified: a Sentinel count is far inside f64's exact integers
            let count = reporting_sentinels as f64;
            *slot = count.ln_1p();
        }
        out
    }
}
