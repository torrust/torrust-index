// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`rebuild_empty`] | feature | With no Sentinels, no identity dimensions, no signals and no cells, the vector still holds the bias term at index zero and the fifteen aggregate slots after it, and every population-dependent block is empty rather than reserved. The two unconditional blocks are the floor the layout is built up from, so a freshly started system has a valid vector to assemble into before anything at all has registered. |
//! | [`rebuild_one_sentinel_m0`] | feature | Registering a Sentinel widens the vector by a slot one place broader than the extraction it holds — the extra place being the occupancy indicator that says whether the extraction means anything. Every Sentinel costs the same width whether or not it is reporting, which is what lets a slot be addressed by a fixed range for as long as that Sentinel stays registered. |
//! | [`rebuild_one_sentinel_m1`] | feature | cites (´claim:feature:a-sentinel-slot-is-one-place-wider-than-its-extraction-width´) |
//! | [`rebuild_eight_sentinels_m1`] | feature | cites (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´) |
//! | [`rebuild_reference_config_p638`] | feature | The reference configuration the specification documents — eight Sentinels, two identity dimensions with one outcome axis, twenty competitive cells and fourteen interaction templates — assembles to exactly the width that specification states, with each named block at its stated size. This is the one place the layout is checked against the published figure rather than against its own arithmetic, so an implementation that drifted from the specification while staying self-consistent would be caught here. |
//! | [`rebuild_with_signals`] | feature | Declared signal features take a block of their own, sized purely by how many were declared, and the Sentinel slots start after it. Signals come from outside the Sentinel machinery entirely, so they are given their own stretch of the vector instead of being folded into any entity's slot. |
//! | [`rebuild_with_identity_dimension`] | feature | Registering a single identity dimension brings two blocks, not one: that dimension's own features and a cross-dimension summary block that was absent entirely while no dimension existed. The summary describes the worst case across dimensions, which is a meaningless quantity with nothing to range over — so the vector does not reserve slots for it until there is. |
//! | [`rebuild_dimension_with_identity_axis_features`] | feature | Each registered outcome axis adds three features to every identity dimension's block, and the axis's offset within that block is the same for all dimensions — the first axis beginning where the fixed features end, the second three places further on. One offset therefore locates a given axis in any dimension's block, so a reader need not keep a separate table per dimension. |
//! | [`rebuild_two_dimensions`] | feature | A second identity dimension brings a second per-dimension block and its own competitive indicators, but not a second cross-dimension block — that one stays single however many dimensions are registered. Per-dimension features multiply with the dimensions while the summary across them does not, which is what keeps the summary a fixed-position reference the model can rely on. |
//! | [`rebuild_sentinel_slots_contiguous`] | feature | Slots abut: each Sentinel's range begins exactly where the previous one ended, and none of them is empty. A contiguous run means the whole Sentinel region can be addressed as one span, and it leaves no unclaimed index that assembly would silently skip and standardisation would then divide by a prior nothing ever updates. |
//! | [`rebuild_deterministic`] | feature | The same configuration rebuilt twice gives the same layout throughout — width, every block boundary, the slot width and the anchor projection alike. The rebuild is a pure function of the registries, so a layout recovered from a checkpoint or recomputed on another node agrees with the one the weights were learned against, and a stray hash ordering cannot silently permute the meaning of the indices. |
//! | [`sentinel_feature_width_m0`] | feature | A Sentinel's extraction width is sixty when no spatial outcome axes are tracked — the same sixty fixed slots extraction itself produces. The map and the extractor agree on the width by construction rather than by coincidence, which is what makes a slot range a safe destination for an extracted row. |
//! | [`sentinel_feature_width_m3`] | feature | cites (´claim:feature:the-per-sentinel-extraction-width-is-sixty-plus-two-for-each-spatial-axis´) |
//! | [`anchor_projection_indices_complete`] | feature | The anchor model reads a fixed fifteen-slot input, and thirteen of those slots are gathered by index: with an identity dimension registered every one of the thirteen names a position that actually exists, inside the vector's bounds, so an entry pointing past the end would be a fault at anchor-prediction time rather than at rebuild time. The remaining two are computed rather than gathered and hold no index at all (´def:dimension:anchor-projection´) — the fourteenth is a maximum over the four per-axis maximum z-score positions and the fifteenth a logarithm of the reporting count, and neither is a position of the vector to point at. |
//! | [`anchor_projection_indices_no_identity`] | feature | Without any identity dimension the projection's six identity entries are explicitly absent while the bias and aggregate entries around them stay populated. The absence is recorded rather than filled with a plausible index into some other block, so the anchor model can tell a missing input from a zero-valued one. Two further entries are absent for a different reason and stay absent whatever is registered: the fourteenth and fifteenth are computed at projection time, not gathered (´def:dimension:anchor-projection´), so representing them as indices would be the divergence this entry repaired (´entry:assayer:wl-feature-anchor-gathered-pair´). |
//! | [`anchor_subvector_computes_its_last_two_inputs`] | feature | The anchor's last two inputs are computed from the vector and the reporting count rather than read out of it. The fourteenth is the maximum over the four per-axis maximum z-score positions of the aggregate block — so raising any one of the four raises it, and it equals the largest of them — and the fifteenth is the logarithm of one plus the reporting Sentinel count, which is zero when nothing reported and grows with the fleet (´def:dimension:anchor-projection´). Gathering the aggregate block's last two positions instead put coverage and the maximum cumulative sum there: quantities the projection table does not name, and neither of them the input the corpus gives the anchor for judging how much of the fleet its estimate rests on. |
//! | [`competitive_indices_correct`] | feature | Every competitive cell across every dimension maps to its own index, all of them distinct and all inside the competitive block, with the block sized to the total cell count. Two cells sharing an index would have each one's indicator overwriting the other's, so membership in one cell would silently be read as membership in another. |
//! | [`p_matches_model_assertion`] | feature | The vector's total width equals the sum of every block computed independently from the configuration — bias, aggregates, identity blocks, the cross-dimension block, signals, Sentinel slots and competitive indicators. The blocks are laid end to end with nothing between them, so the width can be predicted from the registries without building the map, and no index in the vector belongs to no block. |
//! | [`ledger_core_offset_correct`] | feature | Inside a slot the occupancy indicator comes first and the extraction follows it, running exactly to the slot's end with nothing left over. A consumer can therefore find a Sentinel's extracted row from the slot's start alone, and any mismatch between the declared extraction width and the slot would show up as a boundary that fails to land. |
//! | [`axis_ledger_offsets_correct`] | feature | The per-axis features sit after the sixty core ones and fill the slot to its end, two per axis. The core block therefore keeps identical offsets whatever the axis count, so a model reading a core feature by its offset within the slot reads the same quantity in a deployment tracking two axes as in one tracking none. |
//! | [`rebuild_includes_interactions`] | feature | cites (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´) |
//! | [`rebuild_type1_per_sentinel`] | feature | The rebuild does not merely reserve room for a per-Sentinel template's instances: it registers an index under each Sentinel's own identity, so every registered Sentinel can be looked up and found. Computation later writes each product by that identity, and an instance the rebuild failed to register would simply never be filled. |
//! | [`rebuild_type4_fixed`] | feature | cites (´claim:feature:a-template-over-two-fleet-wide-operands-yields-exactly-one-instance-whatever-the-population´) |
//! | [`rebuild_type5_per_cell`] | feature | cites (´claim:feature:a-competitive-cell-template-yields-one-instance-per-cell-and-ignores-the-sentinel-population´) |
//! | [`rebuild_type5_empty_competitive_set`] | feature | A registered dimension holding no competitive cells generates no interaction slots at all, rather than a placeholder for the dimension itself. Instances belong to cells, not to the dimensions that contain them, so a dimension whose cells have all exited costs nothing until a cell enters again. |
//! | [`rebuild_interaction_indices_unique`] | feature | Across a mixture of template kinds sharing one population, every instance gets a distinct index and all of them fall inside the interaction block. Templates are numbered as well as their entities, so two templates ranging over the same Sentinels produce different identities and cannot be assigned the same slot — which would silently make one product overwrite the other. |
//! | [`rebuild_p_equals_sum_with_interactions`] | feature | cites (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´) |
//! | [`dimension_map_serde_roundtrip`] | feature | A layout written to a checkpoint and read back carries every boundary it held: total width, each block's range, the axis offsets, the slot width and the anchor projection. Model weights persisted alongside it are meaningful only against the exact layout they were learned under, so a restored map that differed in any boundary would silently reinterpret every weight. |
//! | [`identity_operand_follows_its_block_across_a_registration`] | feature | An identity operand survives the registration that moves the block it names. The recommended set draws every identity operand from the cross-dimension block (´tab:feature:default-interaction-set´), and that block's position moves whenever a per-dimension block is added — so a declaration carrying a raw index read an identity feature before the registration and a signal or a slot position afterwards, forming and standardising the product without complaint (´entry:assayer:wl-feature-absolute-operands´). A selector has something in it to resolve, so the rebuild re-resolves it: the same declaration names the same feature at both layouts, at two different indices. |

//! Crate-level tests for `DimensionMap` — the floor plan itself
//! (´sec:dimension:map´) and the host declarations whose arrival rebuilds it
//! (´chap:spec:host-contract´).
//!
//! The dimension map is the feature vector's floor plan. Nothing in the vector
//! carries a name at runtime — every consumer addresses features by absolute
//! index — so the map is the single authority on which index means what, and it
//! is rebuilt from the current entity population whenever that population
//! changes.
//!
//! These tests fix the plan block by block: what each block contributes to the
//! total width, that the blocks abut without padding, that a block's size
//! follows only the population it depends on, and that entities keep a stable
//! handle on their own positions across a rebuild or a checkpoint round-trip.

use indexmap::IndexMap;

use crate::feature::dimension_map::DimensionMap;
use crate::identity::CompetitiveCellId;
use crate::testing::dimensions::{cells, dims, make_cells, n_axes, sentinels};
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════
//
// The low-level fixture builders (`sentinels`, `dims`, `cells`,
// `make_cells`) live in [`crate::testing::dimensions`] and are shared
// with the interaction-template tests (`src/feature/interaction.rs`).

// ═══════════════════════════════════════════════════════════════════════════════
// Rebuild tests
// ═══════════════════════════════════════════════════════════════════════════════

/// With no Sentinels, no identity dimensions, no signals and no cells, the
/// vector still holds the bias term at index zero and the fifteen aggregate
/// slots after it, and every population-dependent block is empty rather than
/// reserved. The two unconditional blocks are the floor the layout is built up
/// from, so a freshly started system has a valid vector to assemble into before
/// anything at all has registered.
///
/// ´claim:feature:the-bias-and-aggregate-blocks-exist-unconditionally-while-every-other-block-starts-empty´
/// ´test:crate:rebuild-empty´
#[test]
fn rebuild_empty() {
    let dm = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &[], &IndexMap::new(), &[]);
    // bias(1) + aggregate(15) = 16
    assert_eq!(dm.p, 16);
    assert_eq!(dm.bias_idx, 0);
    assert_eq!(dm.agg_range.len(), 15);
    assert!(dm.sentinel_slots.is_empty());
    assert!(dm.id_dim_ranges.is_empty());
    assert!(dm.id_cross_dim_range.is_none());
    assert!(dm.id_axis_offsets.is_empty());
    assert_eq!(dm.sig_range.len(), 0);
    assert_eq!(dm.competitive_range.len(), 0);
}

/// Registering a Sentinel widens the vector by a slot one place broader than
/// the extraction it holds — the extra place being the occupancy indicator that
/// says whether the extraction means anything. Every Sentinel costs the same
/// width whether or not it is reporting, which is what lets a slot be addressed
/// by a fixed range for as long as that Sentinel stays registered.
///
/// ´claim:feature:a-sentinel-slot-is-one-place-wider-than-its-extraction-width´
/// ´test:crate:rebuild-one-sentinel-m0´
#[test]
fn rebuild_one_sentinel_m0() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);
    // bias(1) + agg(15) + slot(61) = 77
    assert_eq!(dm.p, 77);
    assert_eq!(dm.sentinel_slots.len(), 1);
    let slot = &dm.sentinel_slots[&SentinelId(1)];
    assert_eq!(slot.len(), 61); // occ(1) + q(60) = 61
}

/// Tracking one spatial outcome axis widens the extraction by two and the slot
/// by two with it, the single occupancy place still riding on top. The
/// occupancy overhead is per Sentinel and not per feature, so a deployment that
/// tracks more outcome axes pays for the axes alone.
///
/// (´claim:feature:a-sentinel-slot-is-one-place-wider-than-its-extraction-width´)
/// ´test:crate:rebuild-one-sentinel-m1´
#[test]
fn rebuild_one_sentinel_m1() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &n_axes(1));
    // bias(1) + agg(15) + slot(63) = 79
    assert_eq!(dm.p, 79);
    let slot = &dm.sentinel_slots[&SentinelId(1)];
    assert_eq!(slot.len(), 63); // occ(1) + q(62) = 63
}

/// At a realistic size — eight Sentinels, two identity dimensions, twenty
/// competitive cells — the width is still just the blocks added up, each
/// contributing exactly what its own population dictates. Nothing about scale
/// introduces alignment padding or a shared block, so the layout can be
/// reasoned about the same way at eight Sentinels as at one.
///
/// (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´)
/// ´test:crate:rebuild-eight-sentinels-m1´
#[test]
fn rebuild_eight_sentinels_m1() {
    let cells_a = make_cells(10);
    let cells_b = make_cells(10);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &cells_a), (2, &cells_b)]);
    let sents = sentinels(&[10, 20, 30, 40, 50, 60, 70, 80]);

    let dm = DimensionMap::rebuild_no_interactions(0, &sents, &dim_list, &comp, &n_axes(1));

    // bias(1) + agg(15) + id_dim(2×8) + id_cross(8) + slots(8×63) + comp(20)
    // = 1 + 15 + 16 + 8 + 504 + 20 = 564
    // This legacy helper omits real outcome-axis IDs and interaction features.
    let expected = 1 + 15 + 16 + 8 + 8 * 63 + 20;
    assert_eq!(dm.p, expected);
    assert_eq!(dm.sentinel_slots.len(), 8);
}

/// The reference configuration the specification documents — eight Sentinels,
/// two identity dimensions with one outcome axis, twenty competitive cells and
/// fourteen interaction templates — assembles to exactly the width that
/// specification states, with each named block at its stated size. This is the
/// one place the layout is checked against the published figure rather than
/// against its own arithmetic, so an implementation that drifted from the
/// specification while staying self-consistent would be caught here.
///
/// ´claim:feature:the-reference-configuration-assembles-to-the-width-the-specification-documents´
/// ´test:crate:rebuild-reference-config-p638´
#[test]
fn rebuild_reference_config_p638() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let cells_a = make_cells(10);
    let cells_b = make_cells(10);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &cells_a), (2, &cells_b)]);
    let sents = sentinels(&[10, 20, 30, 40, 50, 60, 70, 80]);
    let axis_ids = [OutcomeAxisId(1)];
    let mut templates: Vec<_> = (0..5)
        .map(|i| InteractionTemplate::type1(i, FeatureSelector::AggregateFeature(0)))
        .collect();
    templates.extend((0..8).map(|i| InteractionTemplate::type4(i, FeatureSelector::AggregateFeature(0))));
    templates.push(InteractionTemplate::type5(FeatureSelector::AggregateFeature(0)));

    let dm = DimensionMap::rebuild_with_axes(0, &sents, &dim_list, &comp, &n_axes(1), &axis_ids, templates);

    assert_eq!(dm.id_dim_ranges[&DimensionId(1)].len(), 11);
    assert_eq!(dm.id_dim_ranges[&DimensionId(2)].len(), 11);
    assert_eq!(dm.id_cross_dim_range.as_ref().unwrap().len(), 8);
    assert_eq!(dm.interaction_range.len(), 68);
    assert_eq!(dm.competitive_range.len(), 20);
    assert_eq!(dm.p, 638);
}

/// Declared signal features take a block of their own, sized purely by how many
/// were declared, and the Sentinel slots start after it. Signals come from
/// outside the Sentinel machinery entirely, so they are given their own stretch
/// of the vector instead of being folded into any entity's slot.
///
/// ´claim:feature:declared-signal-features-occupy-a-block-of-their-own-sized-by-the-declared-count´
/// ´test:crate:rebuild-with-signals´
#[test]
fn rebuild_with_signals() {
    let dm = DimensionMap::rebuild_no_interactions(5, &sentinels(&[1]), &[], &IndexMap::new(), &[]);
    // bias(1) + agg(15) + sig(5) + slot(61) = 82
    assert_eq!(dm.p, 82);
    assert_eq!(dm.sig_range.len(), 5);
}

/// Registering a single identity dimension brings two blocks, not one: that
/// dimension's own features and a cross-dimension summary block that was absent
/// entirely while no dimension existed. The summary describes the worst case
/// across dimensions, which is a meaningless quantity with nothing to range
/// over — so the vector does not reserve slots for it until there is.
///
/// ´claim:feature:the-cross-dimension-block-appears-with-the-first-identity-dimension-and-is-absent-without-one´
/// ´test:crate:rebuild-with-identity-dimension´
#[test]
fn rebuild_with_identity_dimension() {
    let c = make_cells(5);
    let dim_list = dims(&[1]);
    let comp = cells(&[(1, &c)]);

    let dm = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &dim_list, &comp, &[]);
    // bias(1) + agg(15) + id_dim(8) + id_cross(8) + comp(5) = 37
    assert_eq!(dm.p, 37);
    assert_eq!(dm.id_dim_ranges.len(), 1);
    assert_eq!(dm.id_dim_ranges[&DimensionId(1)].len(), 8);
    assert_eq!(dm.id_cross_dim_range.as_ref().unwrap().len(), 8);
    assert_eq!(dm.competitive_range.len(), 5);
}

/// Each registered outcome axis adds three features to every identity
/// dimension's block, and the axis's offset within that block is the same for
/// all dimensions — the first axis beginning where the fixed features end, the
/// second three places further on. One offset therefore locates a given axis in
/// any dimension's block, so a reader need not keep a separate table per
/// dimension.
///
/// ´claim:feature:each-outcome-axis-adds-three-features-at-the-same-offset-in-every-identity-block´
/// ´test:crate:rebuild-dimension-with-identity-axis-features´
#[test]
fn rebuild_dimension_with_identity_axis_features() {
    let dim_list = dims(&[1]);
    let axis_ids = [OutcomeAxisId(1), OutcomeAxisId(2)];

    let dm = DimensionMap::rebuild_with_axes_no_interactions(0, &IndexMap::new(), &dim_list, &IndexMap::new(), &[], &axis_ids);

    // bias(1) + agg(15) + id_dim(8 + 2×3) + id_cross(8) = 38
    assert_eq!(dm.p, 38);
    assert_eq!(dm.id_axis_offsets.len(), 2);
    assert_eq!(dm.id_axis_offsets[&OutcomeAxisId(1)], 8);
    assert_eq!(dm.id_axis_offsets[&OutcomeAxisId(2)], 11);
    assert_eq!(dm.id_dim_ranges[&DimensionId(1)].len(), 14);
}

/// A second identity dimension brings a second per-dimension block and its own
/// competitive indicators, but not a second cross-dimension block — that one
/// stays single however many dimensions are registered. Per-dimension features
/// multiply with the dimensions while the summary across them does not, which
/// is what keeps the summary a fixed-position reference the model can rely on.
///
/// ´claim:feature:per-dimension-blocks-multiply-with-the-dimensions-while-the-cross-dimension-block-stays-single´
/// ´test:crate:rebuild-two-dimensions´
#[test]
fn rebuild_two_dimensions() {
    let cells_a = make_cells(10);
    let cells_b = make_cells(10);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &cells_a), (2, &cells_b)]);

    let dm = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &dim_list, &comp, &[]);
    // bias(1) + agg(15) + id_dim(2×8) + id_cross(8) + comp(20) = 60
    assert_eq!(dm.p, 60);
    assert_eq!(dm.id_dim_ranges.len(), 2);
    assert_eq!(dm.competitive_range.len(), 20);
}

/// Slots abut: each Sentinel's range begins exactly where the previous one
/// ended, and none of them is empty. A contiguous run means the whole Sentinel
/// region can be addressed as one span, and it leaves no unclaimed index that
/// assembly would silently skip and standardisation would then divide by a
/// prior nothing ever updates.
///
/// ´claim:feature:sentinel-slots-are-laid-contiguously-with-no-gap-between-one-slot-and-the-next´
/// ´test:crate:rebuild-sentinel-slots-contiguous´
#[test]
fn rebuild_sentinel_slots_contiguous() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1, 2, 3]), &[], &IndexMap::new(), &[]);
    assert_eq!(dm.sentinel_slots.len(), 3);

    let mut prev_end = None;
    for (_, slot) in &dm.sentinel_slots {
        if let Some(end) = prev_end {
            assert_eq!(slot.start, end, "slots must be contiguous");
        }
        assert!(slot.start < slot.end, "slot must be non-empty");
        prev_end = Some(slot.end);
    }
}

/// The same configuration rebuilt twice gives the same layout throughout —
/// width, every block boundary, the slot width and the anchor projection alike.
/// The rebuild is a pure function of the registries, so a layout recovered from
/// a checkpoint or recomputed on another node agrees with the one the weights
/// were learned against, and a stray hash ordering cannot silently permute the
/// meaning of the indices.
///
/// ´claim:feature:rebuilding-from-the-same-configuration-yields-an-identical-layout´
/// ´test:crate:rebuild-deterministic´
#[test]
fn rebuild_deterministic() {
    let c = make_cells(5);
    let dim_list = dims(&[1]);
    let comp = cells(&[(1, &c)]);
    let sents = sentinels(&[10, 20]);

    let dm1 = DimensionMap::rebuild_no_interactions(3, &sents, &dim_list, &comp, &n_axes(1));
    let dm2 = DimensionMap::rebuild_no_interactions(3, &sents, &dim_list, &comp, &n_axes(1));

    assert_eq!(dm1.p, dm2.p);
    assert_eq!(dm1.agg_range, dm2.agg_range);
    assert_eq!(dm1.competitive_range, dm2.competitive_range);
    assert_eq!(dm1.sentinel_feature_width, dm2.sentinel_feature_width);
    assert_eq!(dm1.anchor_projection_indices, dm2.anchor_projection_indices);
}

/// A Sentinel's extraction width is sixty when no spatial outcome axes are
/// tracked — the same sixty fixed slots extraction itself produces. The map and
/// the extractor agree on the width by construction rather than by coincidence,
/// which is what makes a slot range a safe destination for an extracted row.
///
/// ´claim:feature:the-per-sentinel-extraction-width-is-sixty-plus-two-for-each-spatial-axis´
/// ´test:crate:sentinel-feature-width-m0´
#[test]
fn sentinel_feature_width_m0() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);
    assert_eq!(dm.sentinel_feature_width, 60);
}

/// Three spatial outcome axes widen the extraction to sixty-six, two places per
/// axis. The map derives the width from the axis count rather than being told
/// it, so adding an outcome axis to a deployment cannot leave the layout and
/// the extractor disagreeing about how much room a Sentinel needs.
///
/// (´claim:feature:the-per-sentinel-extraction-width-is-sixty-plus-two-for-each-spatial-axis´)
/// ´test:crate:sentinel-feature-width-m3´
#[test]
fn sentinel_feature_width_m3() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &n_axes(3));
    assert_eq!(dm.sentinel_feature_width, 66);
}

/// The anchor model reads a fixed fifteen-slot input, and thirteen of those
/// slots are gathered by index: with an identity dimension registered every
/// one of the thirteen names a position that actually exists, inside the
/// vector's bounds, so an entry pointing past the end would be a fault at
/// anchor-prediction time rather than at rebuild time. The remaining two are
/// computed rather than gathered and hold no index at all
/// (´def:dimension:anchor-projection´) — the fourteenth is a maximum over the
/// four per-axis maximum z-score positions and the fifteenth a logarithm of
/// the reporting count, and neither is a position of the vector to point at.
///
/// ´claim:feature:every-populated-anchor-projection-entry-names-a-position-inside-the-vector´
/// ´test:crate:anchor-projection-indices-complete´
#[test]
fn anchor_projection_indices_complete() {
    let c = make_cells(5);
    let dim_list = dims(&[1]);
    let comp = cells(&[(1, &c)]);
    let sents = sentinels(&[10, 20, 30, 40, 50, 60, 70, 80]);

    let dm = DimensionMap::rebuild_no_interactions(0, &sents, &dim_list, &comp, &n_axes(1));

    // The thirteen gathered entries are populated when an identity dimension exists.
    for (i, idx) in dm.anchor_projection_indices.iter().enumerate().take(13) {
        assert!(idx.is_some(), "gathered anchor entry {i} should be Some");
    }

    // The two computed entries hold no index.
    assert!(dm.anchor_projection_indices[13].is_none(), "the per-axis maximum is computed");
    assert!(dm.anchor_projection_indices[14].is_none(), "the reporting count is computed");

    // All indices should be within the feature vector.
    for idx in dm.anchor_projection_indices.iter().flatten() {
        assert!(*idx < dm.p, "anchor index {idx} >= p={}", dm.p);
    }
}

/// Without any identity dimension the projection's six identity entries are
/// explicitly absent while the bias and aggregate entries around them stay
/// populated. The absence is recorded rather than filled with a plausible index
/// into some other block, so the anchor model can tell a missing input from a
/// zero-valued one. Two further entries are absent for a different reason and
/// stay absent whatever is registered: the fourteenth and fifteenth are
/// computed at projection time, not gathered
/// (´def:dimension:anchor-projection´), so representing them as indices would
/// be the divergence this entry repaired
/// (´entry:assayer:wl-feature-anchor-gathered-pair´).
///
/// ´claim:feature:the-anchor-projections-identity-entries-are-recorded-absent-when-no-identity-dimension-exists´
/// ´test:crate:anchor-projection-indices-no-identity´
#[test]
fn anchor_projection_indices_no_identity() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);

    // Bias and aggregate indices (0–6) should be Some.
    for (i, index) in dm.anchor_projection_indices.iter().enumerate().take(7) {
        assert!(index.is_some(), "anchor[{i}] should be Some");
    }

    // Identity entries 7–12 should be None when no identity dimension exists.
    for (i, index) in dm.anchor_projection_indices.iter().enumerate().take(13).skip(7) {
        assert!(index.is_none(), "anchor[{i}] should be None without identity");
    }

    // Entries 13–14 are computed, never gathered — absent whatever is registered.
    assert!(dm.anchor_projection_indices[13].is_none());
    assert!(dm.anchor_projection_indices[14].is_none());
}

/// The anchor's last two inputs are computed from the vector and the reporting
/// count rather than read out of it. The fourteenth is the maximum over the
/// four per-axis maximum z-score positions of the aggregate block — so raising
/// any one of the four raises it, and it equals the largest of them — and the
/// fifteenth is the logarithm of one plus the reporting Sentinel count, which
/// is zero when nothing reported and grows with the fleet
/// (´def:dimension:anchor-projection´). Gathering the aggregate block's last
/// two positions instead put coverage and the maximum cumulative sum there:
/// quantities the projection table does not name, and neither of them the
/// input the corpus gives the anchor for judging how much of the fleet its
/// estimate rests on.
///
/// ´claim:feature:the-anchor-computes-its-per-axis-maximum-and-its-reporting-count´
/// ´test:crate:anchor-subvector-computes-its-last-two-inputs´
#[test]
fn anchor_subvector_computes_its_last_two_inputs() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);

    let mut phi = vec![0.0; dm.p];
    // The four per-axis maxima sit at aggregate offsets 3 to 6.
    let per_axis_start = dm.agg_range.start + 3;
    phi[per_axis_start] = 1.5;
    phi[per_axis_start + 1] = 4.25;
    phi[per_axis_start + 2] = 0.5;
    phi[per_axis_start + 3] = 2.0;
    // Coverage and the maximum cumulative sum, which the gather used to take.
    phi[dm.agg_range.start + 13] = 0.125;
    phi[dm.agg_range.start + 14] = 9.75;

    let tilde = dm.extract_anchor_subvector(&phi, 15, 7);

    assert_eq!(tilde.len(), 15);
    assert!(
        (tilde[13] - 4.25).abs() < 1e-12,
        "the fourteenth is the largest per-axis maximum, got {}",
        tilde[13]
    );
    assert!(
        (tilde[14] - 8.0_f64.ln()).abs() < 1e-12,
        "the fifteenth is ln(1 + 7), got {}",
        tilde[14]
    );

    // Nothing reporting makes the fifteenth exactly zero.
    let none_reporting = dm.extract_anchor_subvector(&phi, 15, 0);
    assert!(none_reporting[14].abs() < 1e-12, "ln(1 + 0) is zero");
}

/// Every competitive cell across every dimension maps to its own index, all of
/// them distinct and all inside the competitive block, with the block sized to
/// the total cell count. Two cells sharing an index would have each one's
/// indicator overwriting the other's, so membership in one cell would silently
/// be read as membership in another.
///
/// ´claim:feature:every-competitive-cell-maps-to-a-distinct-index-inside-the-competitive-block´
/// ´test:crate:competitive-indices-correct´
#[test]
fn competitive_indices_correct() {
    let cells_a = make_cells(5);
    let cells_b = make_cells(3);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &cells_a), (2, &cells_b)]);

    let dm = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &dim_list, &comp, &[]);

    // Total competitive indicator count.
    assert_eq!(dm.competitive_range.len(), 8);

    // Each cell should map to a unique index within the competitive range.
    let mut all_indices: Vec<usize> = Vec::new();
    for (_, cell_map) in &dm.competitive_indices {
        for (_, &idx) in cell_map {
            assert!(
                idx >= dm.competitive_range.start && idx < dm.competitive_range.end,
                "cell index {idx} outside competitive range {:?}",
                dm.competitive_range
            );
            all_indices.push(idx);
        }
    }
    all_indices.sort_unstable();
    all_indices.dedup();
    assert_eq!(all_indices.len(), 8, "all indices should be unique");
}

/// The vector's total width equals the sum of every block computed
/// independently from the configuration — bias, aggregates, identity blocks,
/// the cross-dimension block, signals, Sentinel slots and competitive
/// indicators. The blocks are laid end to end with nothing between them, so the
/// width can be predicted from the registries without building the map, and no
/// index in the vector belongs to no block.
///
/// ´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´
/// ´test:crate:p-matches-model-assertion´
#[test]
fn p_matches_model_assertion() {
    let c = make_cells(7);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &c), (2, &c)]);
    let sents = sentinels(&[10, 20]);

    let dm = DimensionMap::rebuild_no_interactions(4, &sents, &dim_list, &comp, &n_axes(2));

    // Compute expected p from blocks:
    let bias = 1;
    let agg = 15;
    let id_dim = 2 * 8; // 2 dims × 8 features
    let id_cross = 8;
    let sig = 4;
    let q = 60 + 2 * 2; // m_s=2
    let slot_width = 1 + q; // occ + q
    let sentinel_total = 2 * slot_width;
    let competitive = 7 + 7; // 7 cells per dim × 2 dims
    let expected = bias + agg + id_dim + id_cross + sig + sentinel_total + competitive;

    assert_eq!(dm.p, expected);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Feature layout tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Inside a slot the occupancy indicator comes first and the extraction follows
/// it, running exactly to the slot's end with nothing left over. A consumer can
/// therefore find a Sentinel's extracted row from the slot's start alone, and
/// any mismatch between the declared extraction width and the slot would show
/// up as a boundary that fails to land.
///
/// ´claim:feature:the-occupancy-indicator-opens-each-slot-and-the-extraction-exactly-fills-the-remainder´
/// ´test:crate:ledger-core-offset-correct´
#[test]
fn ledger_core_offset_correct() {
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &[]);
    let slot = &dm.sentinel_slots[&SentinelId(1)];

    // Slot = [occ(1) | g_s(q)] where q = q_base = 60 (m_s=0).
    // Occupancy at slot.start, core extraction at slot.start + 1.
    let occ_idx = slot.start;
    let core_start = occ_idx + 1;
    let core_end = core_start + dm.sentinel_feature_width;

    assert_eq!(core_end, slot.end, "core end should match slot end");
    assert_eq!(slot.len(), 1 + 60, "slot = occ(1) + q_base(60)");
    assert_eq!(core_start, slot.start + 1, "core starts after occupancy");
}

/// The per-axis features sit after the sixty core ones and fill the slot to its
/// end, two per axis. The core block therefore keeps identical offsets whatever
/// the axis count, so a model reading a core feature by its offset within the
/// slot reads the same quantity in a deployment tracking two axes as in one
/// tracking none.
///
/// ´claim:feature:the-per-axis-features-follow-the-core-sixty-and-fill-the-slot-to-its-end´
/// ´test:crate:axis-ledger-offsets-correct´
#[test]
fn axis_ledger_offsets_correct() {
    // With 2 spatial axes, q = q_base + 2*m_s = 60 + 4 = 64.
    // Slot: [occ(1) | core(60) | axis0(2) | axis1(2)]
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels(&[1]), &[], &IndexMap::new(), &n_axes(2));
    let slot = &dm.sentinel_slots[&SentinelId(1)];

    assert_eq!(dm.sentinel_feature_width, 64);
    assert_eq!(slot.len(), 1 + 64, "slot = occ(1) + q(64)");

    // Axis features start at offset q_base=60 within the extraction portion.
    let extraction_start = slot.start + 1; // after occupancy
    let axis_start = extraction_start + 60; // after core features
    // 2 axes × 2 features = 4 features at [axis_start..axis_start+4].
    assert_eq!(slot.end, axis_start + 4, "axis features fill remainder");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Interaction rebuild tests (´alg:feature:lifecycle-interactions´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Declaring a single template adds one more slot to the vector and one entry
/// to the index map, the two agreeing on the count. Interactions are a block
/// like any other: they widen the vector by what the templates generate, and
/// the map gains a way to reach each one.
///
/// (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´)
/// ´test:crate:rebuild-includes-interactions´
#[test]
fn rebuild_includes_interactions() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // 1 Sentinel, 1 Type-4 template (fixed), no competitive cells.
    let templates = vec![InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(0))];
    let dm = DimensionMap::rebuild(0, &sentinels(&[1]), &[], &IndexMap::new(), &[], templates);

    // bias(1) + agg(15) + slot(61) + int(1) = 78
    assert_eq!(dm.p, 78);
    assert_eq!(dm.interaction_range.len(), 1);
    assert_eq!(dm.interaction_indices.len(), 1);
}

/// The rebuild does not merely reserve room for a per-Sentinel template's
/// instances: it registers an index under each Sentinel's own identity, so
/// every registered Sentinel can be looked up and found. Computation later
/// writes each product by that identity, and an instance the rebuild failed to
/// register would simply never be filled.
///
/// ´claim:feature:the-rebuild-registers-an-interaction-index-under-each-sentinels-own-identity´
/// ´test:crate:rebuild-type1-per-sentinel´
#[test]
fn rebuild_type1_per_sentinel() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // 3 Sentinels, 1 Type-1 template → 3 interaction features.
    let templates = vec![InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0))];
    let dm = DimensionMap::rebuild(0, &sentinels(&[1, 2, 3]), &[], &IndexMap::new(), &[], templates);

    assert_eq!(dm.interaction_range.len(), 3, "Type-1: one per Sentinel");
    assert_eq!(dm.interaction_indices.len(), 3);

    // Verify each Sentinel has an interaction ID.
    for &id in &[SentinelId(1), SentinelId(2), SentinelId(3)] {
        use crate::feature::interaction::InteractionId;
        let int_id = InteractionId::per_sentinel(0, id);
        assert!(
            dm.interaction_indices.contains_key(&int_id),
            "Missing interaction for Sentinel {:?}",
            id
        );
    }
}

/// Eight fleet-wide templates claim eight slots with no Sentinels registered at
/// all — the layout agreeing with the templates' own count rule. These slots
/// are the part of the interaction block that never moves as the population
/// churns, which makes them the safest place for the model's most stable
/// weights.
///
/// (´claim:feature:a-template-over-two-fleet-wide-operands-yields-exactly-one-instance-whatever-the-population´)
/// ´test:crate:rebuild-type4-fixed´
#[test]
fn rebuild_type4_fixed() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // 8 Type-4 templates → 8 interaction features regardless of Sentinel count.
    let templates: Vec<_> = (0..8)
        .map(|i| InteractionTemplate::type4(i, FeatureSelector::AggregateFeature(0)))
        .collect();

    // Zero Sentinels still produces 8 Type-4 features.
    let dm = DimensionMap::rebuild(0, &IndexMap::new(), &[], &IndexMap::new(), &[], templates);

    assert_eq!(dm.interaction_range.len(), 8, "Type-4: 8 fixed features");
    assert_eq!(dm.interaction_indices.len(), 8);
}

/// A single competitive-cell template claims one slot for every cell across
/// every dimension — twenty in all here, ten from each of two dimensions — with
/// no Sentinels registered. Cells from different dimensions are separate
/// instances even where they occupy the same position within their own
/// dimension, so the block's size follows the total cell population.
///
/// (´claim:feature:a-competitive-cell-template-yields-one-instance-per-cell-and-ignores-the-sentinel-population´)
/// ´test:crate:rebuild-type5-per-cell´
#[test]
fn rebuild_type5_per_cell() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let c = make_cells(10);
    let dim_list = dims(&[1, 2]);
    // 10 cells per dimension = 20 total.
    let comp = cells(&[(1, &c), (2, &c)]);

    // 1 Type-5 template → 20 interaction features (one per competitive cell).
    let templates = vec![InteractionTemplate::type5(FeatureSelector::AggregateFeature(0))];
    let dm = DimensionMap::rebuild(0, &IndexMap::new(), &dim_list, &comp, &[], templates);

    assert_eq!(dm.interaction_range.len(), 20, "Type-5: one per cell");
    assert_eq!(dm.interaction_indices.len(), 20);
}

/// A registered dimension holding no competitive cells generates no interaction
/// slots at all, rather than a placeholder for the dimension itself. Instances
/// belong to cells, not to the dimensions that contain them, so a dimension
/// whose cells have all exited costs nothing until a cell enters again.
///
/// ´claim:feature:a-registered-dimension-with-no-competitive-cells-generates-no-interaction-slots´
/// ´test:crate:rebuild-type5-empty-competitive-set´
#[test]
fn rebuild_type5_empty_competitive_set() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // Dimension registered but with 0 competitive cells.
    let dim_list = dims(&[1]);
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();

    let templates = vec![InteractionTemplate::type5(FeatureSelector::AggregateFeature(0))];
    let dm = DimensionMap::rebuild(0, &IndexMap::new(), &dim_list, &comp, &[], templates);

    // Type-5 produces 0 features when no competitive cells.
    assert_eq!(dm.interaction_range.len(), 0);
    assert_eq!(dm.interaction_indices.len(), 0);
}

/// Across a mixture of template kinds sharing one population, every instance
/// gets a distinct index and all of them fall inside the interaction block.
/// Templates are numbered as well as their entities, so two templates ranging
/// over the same Sentinels produce different identities and cannot be assigned
/// the same slot — which would silently make one product overwrite the other.
///
/// ´claim:feature:interaction-indices-are-distinct-across-templates-and-confined-to-the-interaction-block´
/// ´test:crate:rebuild-interaction-indices-unique´
#[test]
fn rebuild_interaction_indices_unique() {
    use std::collections::HashSet;

    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let c = make_cells(5);
    let dim_list = dims(&[1]);
    let comp = cells(&[(1, &c)]);
    let sents = sentinels(&[10, 20, 30]);

    // Mix of templates: 2 Type-1 (3 each), 1 Type-4, 1 Type-5 (5 each)
    // = 6 + 1 + 5 = 12 total.
    let templates = vec![
        InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type1(1, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type5(FeatureSelector::AggregateFeature(0)),
    ];
    let dm = DimensionMap::rebuild(0, &sents, &dim_list, &comp, &[], templates);

    // All indices must be unique.
    let indices: HashSet<_> = dm.interaction_indices.values().collect();
    assert_eq!(
        indices.len(),
        dm.interaction_indices.len(),
        "All interaction indices must be unique"
    );

    // All indices within interaction range.
    for &idx in dm.interaction_indices.values() {
        assert!(
            idx >= dm.interaction_range.start && idx < dm.interaction_range.end,
            "Index {} outside interaction range {:?}",
            idx,
            dm.interaction_range
        );
    }
}

/// With interactions present the width is still the plain sum of every block,
/// the interaction block contributing exactly what its templates generate over
/// the current population. Interactions are derived features rather than
/// declared ones, but they occupy real indices and are accounted for like any
/// other block.
///
/// (´claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them´)
/// ´test:crate:rebuild-p-equals-sum-with-interactions´
#[test]
fn rebuild_p_equals_sum_with_interactions() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let c = make_cells(5);
    let dim_list = dims(&[1]);
    let comp = cells(&[(1, &c)]);
    let sents = sentinels(&[10, 20]);

    // 2 Type-1 templates (n=2 each = 4) + 1 Type-4 (1) + 1 Type-5 (5 cells) = 10 interactions.
    let templates = vec![
        InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type1(1, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(0)),
        InteractionTemplate::type5(FeatureSelector::AggregateFeature(0)),
    ];
    let dm = DimensionMap::rebuild(3, &sents, &dim_list, &comp, &n_axes(1), templates);

    // bias(1) + agg(15) + id_dim(8) + id_cross(8) + sig(3) + slots(2×63) + int(10) + comp(5)
    let expected = 1 + 15 + 8 + 8 + 3 + (2 * 63) + 10 + 5;
    assert_eq!(dm.p, expected);
    assert_eq!(dm.interaction_range.len(), 10);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serde round-trip
// ═══════════════════════════════════════════════════════════════════════════════

/// A layout written to a checkpoint and read back carries every boundary it
/// held: total width, each block's range, the axis offsets, the slot width and
/// the anchor projection. Model weights persisted alongside it are meaningful
/// only against the exact layout they were learned under, so a restored map
/// that differed in any boundary would silently reinterpret every weight.
///
/// ´claim:feature:a-persisted-layout-restores-every-block-boundary-it-was-holding´
/// ´test:crate:dimension-map-serde-roundtrip´
#[cfg(feature = "serde")]
#[test]
fn dimension_map_serde_roundtrip() {
    let cells_a = make_cells(5);
    let dim_list = dims(&[1, 2]);
    let comp = cells(&[(1, &cells_a), (2, &cells_a)]);
    let sents = sentinels(&[10, 20, 30]);

    let dm = DimensionMap::rebuild_no_interactions(3, &sents, &dim_list, &comp, &n_axes(1));

    let bytes = crate::serde_codec::serialize(&dm).expect("serialize");
    let dm2: DimensionMap = crate::serde_codec::deserialize(&bytes).expect("deserialize");

    assert_eq!(dm.p, dm2.p);
    assert_eq!(dm.bias_idx, dm2.bias_idx);
    assert_eq!(dm.agg_range, dm2.agg_range);
    assert_eq!(dm.id_dim_ranges, dm2.id_dim_ranges);
    assert_eq!(dm.id_cross_dim_range, dm2.id_cross_dim_range);
    assert_eq!(dm.id_axis_offsets, dm2.id_axis_offsets);
    assert_eq!(dm.sig_range, dm2.sig_range);
    assert_eq!(dm.competitive_range, dm2.competitive_range);
    assert_eq!(dm.sentinel_feature_width, dm2.sentinel_feature_width);
    assert_eq!(dm.anchor_projection_indices, dm2.anchor_projection_indices);
    assert_eq!(dm.sentinel_slots.len(), dm2.sentinel_slots.len());
    assert_eq!(dm.competitive_indices.len(), dm2.competitive_indices.len());
}

/// An identity operand survives the registration that moves the block it
/// names. The recommended set draws every identity operand from the
/// cross-dimension block (´tab:feature:default-interaction-set´), and that
/// block's position moves whenever a per-dimension block is added — so a
/// declaration carrying a raw index read an identity feature before the
/// registration and a signal or a slot position afterwards, forming and
/// standardising the product without complaint
/// (´entry:assayer:wl-feature-absolute-operands´). A selector has something in
/// it to resolve, so the rebuild re-resolves it: the same declaration names
/// the same feature at both layouts, at two different indices.
///
/// ´claim:feature:an-identity-operand-resolves-to-the-cross-dimension-block-wherever-the-rebuild-puts-it´
/// ´test:crate:identity-operand-follows-its-block-across-a-registration´
#[test]
fn identity_operand_follows_its_block_across_a_registration() {
    use crate::feature::dimension_map::ID_CROSS_DIM_FEATURE_COUNT;
    use crate::feature::interaction::FeatureSelector;

    let sents = sentinels(&[1]);
    let operand = FeatureSelector::IdentityFeature(1);

    let one_dimension = DimensionMap::rebuild_no_interactions(0, &sents, &dims(&[1]), &IndexMap::new(), &[]);
    let two_dimensions = DimensionMap::rebuild_no_interactions(0, &sents, &dims(&[1, 2]), &IndexMap::new(), &[]);

    let before = one_dimension.resolve_context(operand).expect("resolves at one dimension");
    let after = two_dimensions.resolve_context(operand).expect("resolves at two");

    assert_ne!(before, after, "the cross-dimension block moved");
    assert_eq!(
        before,
        one_dimension.id_cross_dim_range.as_ref().expect("present").start + 1,
        "and it named the block's second position both times"
    );
    assert_eq!(after, two_dimensions.id_cross_dim_range.as_ref().expect("present").start + 1,);

    // With no dimension registered the operand names nothing and is read as
    // zero, the treatment the anchor projection gives its absent entries.
    let no_dimension = DimensionMap::rebuild_no_interactions(0, &sents, &[], &IndexMap::new(), &[]);
    assert!(
        no_dimension.resolve_context(operand).is_none(),
        "absent while no dimension is registered"
    );

    // An offset past the block's width names nothing rather than spilling into
    // whatever follows it.
    assert!(
        one_dimension
            .resolve_context(FeatureSelector::IdentityFeature(ID_CROSS_DIM_FEATURE_COUNT))
            .is_none(),
        "an offset past the block names nothing"
    );
}
