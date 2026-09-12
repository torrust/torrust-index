// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`bias_always_one`] | feature | The leading slot is one even when every component handed to assembly is empty, and it survives the standardisation pass unchanged. The model's intercept is carried by that constant, so it has to be present in a vector assembled from nothing at all just as much as in a fully populated one. |
//! | [`phi_length_matches_p`] | feature | Assembly produces a vector of exactly the width the layout declares, and every position in it is finite, even when no component supplied a value. Positions nobody wrote to are zeros rather than uninitialised or absent, so the model's dot product is always over the dimension its weights expect. |
//! | [`aggregates_at_correct_indices`] | feature | The fifteen fleet-wide aggregates land in their given order directly after the bias, each keeping the value it was handed. Their positions are the ones interaction templates and the anchor projection address by absolute index, so a permutation here would silently redirect every one of those references. |
//! | [`sentinel_occupancy_one_when_present`] | feature | A Sentinel that supplied an occupied extraction has its slot opened with an indicator of one, even though every extracted feature behind it is zero. Measured calm and no measurement at all produce the same zeros, and this one place is what lets the model tell them apart. |
//! | [`sentinel_occupancy_zero_when_absent`] | feature | A registered Sentinel that supplied no extraction at all still owns its slot, opened with an indicator of zero. The slot is reserved by registration rather than by reporting, so a Sentinel falling silent for a round does not shift every later feature along and invalidate the model's weights. |
//! | [`competitive_indicator_active`] | feature | Exactly the cells named active carry an indicator of one; every other registered cell keeps a zero at its own index. The indicators are a membership statement about this particular request, so a cell that exists but is not being competed in this round must read as absent rather than merely unwritten. |
//! | [`identity_aggregate_features`] | feature | Each identity dimension's block opens with its three structural aggregates — coverage depth, active fraction and total importance — in that order at the block's start. These are the positions label-time assembly can re-derive later, which is why they occupy the front of the block rather than being interleaved with the measurements behind them. |
//! | [`assembly_interaction_correct_product`] | feature | cites (´claim:feature:an-interaction-slot-holds-the-plain-product-of-its-two-operands-read-from-their-own-positions´) |
//! | [`assembly_standardisation_applied`] | feature | The vector assembly returns has already been standardised against the running statistics: a feature two deviations above its recorded mean comes back as two, while the bias keeps its constant one. Standardisation is part of assembly rather than a step callers must remember, so no caller can hand the model a vector on the wrong scale. |
//! | [`assembly_signal_features`] | feature | Signal values are written into the signal block in the order supplied, starting at the block's own beginning. Signals are positional by declaration order and carry no identity of their own, so their meaning rests entirely on arriving in the same order they were declared in. |
//! | [`assembly_dimension_measurement`] | feature | The five measurement features — the alarm pair, peak suspicion, the deepest cell's volatility and the competitive-cell indicator — follow the structural aggregates within each dimension's block, in that order. They describe what was measured at a particular moment rather than the dimension's standing shape, which is why they sit behind the prefix that can be recomputed later. |
//! | [`assembly_identity_axis_features`] | feature | An outcome axis's three features are written at the offset the layout itself records for that axis, found by asking the map rather than by assuming a position. Axes are registered dynamically and identified by their own identifiers, so assembly and any later reader agree on where an axis lives only because both consult the same recorded offset. |
//! | [`assembly_sentinel_features_placed`] | feature | An extracted row is copied into its own Sentinel's slot in order, beginning one place past the occupancy indicator. The row's internal ordering is the one extraction fixed, so a feature's offset within an extraction and its offset within a slot differ by exactly that one place — which is what interaction templates count on when they address a Sentinel feature. |
//! | [`assembly_zero_sentinels`] | feature | cites (´claim:feature:assembly-always-produces-a-finite-vector-of-exactly-the-declared-width´) |
//! | [`extract_sub_scores_from_extractions`] | feature | Only occupied extractions contribute sub-scores to the fleet aggregate: an unoccupied one is left out entirely rather than entering as a row of zeros, and the surviving Sentinel's four leading z-scores come through as its own. The aggregate's means, spreads and concordance fractions are all divided by the number of contributors, so counting a silent Sentinel would dilute every one of them toward zero. |
//! | [`label_assembly_uses_frozen_extractions`] | feature | At label time the vector is rebuilt from the extractions frozen when the assessment was made, each landing in its Sentinel's slot exactly as it did then. Learning must attribute an outcome to the evidence the decision was actually taken on, not to whatever those Sentinels are reporting by the time the outcome arrives. |
//! | [`label_narrower_stored_row_blanks_only_the_new_axis_pair`] | feature | A stored row narrower than the slot is placed group by group. Scenario: a spatial outcome axis was registered between assessment and labelling, increasing q from 60 to 62. The stored extraction has 60 positions and the current slot expects 62, and the two extra belong in the middle rather than at the end. The positions the assessment could not describe are the new axis's own pair, and they are the ones left at zero: that axis did not exist when the assessment was made, so zero is the honest value, and the alternative would be refusing to learn from any assessment that predates a configuration change. What is *not* left at zero is the batch context feature, which is the sixth group and follows the pairs (´def:extraction:slot´) — it is carried to its own new last position. A copy that stopped at the stored width instead would leave the batch context value standing where the new pair now lives and zero the batch context's own position (´entry:assayer:wl-feature-frozen-slot-truncation´). |
//! | [`label_wider_stored_row_loses_only_the_retired_axis_pair`] | feature | A stored row wider than the slot is placed group by group, so what it loses is decided by axis ownership rather than by where the slot ends. Scenario: a spatial outcome axis was deregistered between assessment and labelling, decreasing q from 62 to 60. The stored extraction has 62 features; the current slot expects 60. The fixed run copies as itself, the retired axis's pair is dropped, the batch context moves back into the position the pair vacated, and no write exceeds slot_range.end. A stored row wider than its slot loses exactly the retired axis's own pair and nothing else: what is dropped is chosen by the axis that owned it having no position left in the layout, rather than by where the slot happens to end. The batch context feature follows the pairs (´def:extraction:slot´), so a copy truncating at the slot boundary would discard it along with half a pair (´entry:assayer:wl-feature-frozen-slot-truncation´); placed by group it survives at its own new last position. The vector's total width is unchanged and no discarded value appears anywhere in it, so a deregistered axis cannot corrupt a neighbouring Sentinel's slot at label time. |
//! | [`label_identity_frozen_features_placed_from_storage`] | feature | The frozen identity features are placed from storage: behind each dimension's re-derived structural triple, the five stored measurement values land at their positions and the stored per-axis values land at the offset the layout records for the axis, while a dimension with nothing stored keeps zeros there. Sources one and three of the reconstruction read these values from the pending entry (´alg:runtime:reconstruction´) — they recorded what was measured at assessment time, exist nowhere but the buffer, and re-deriving them at label time would attach whatever is true today to a decision taken earlier. |
//! | [`label_slots_fill_three_ways`] | feature | One slot per currently registered Sentinel, filled three ways from the stored entry: the frozen extraction where the Sentinel was reporting, occupancy with zero features where it was active but silent, and zeros where it was registered after the assessment. Telling the second case from the third is exactly what the stored Sentinel lists exist for (´alg:runtime:reconstruction´) — the extraction map alone cannot say whether a Sentinel with no entry was silent or absent. |
//! | [`label_reconstruction_matches_assessment_under_unchanged_state`] | feature | With the working state unchanged between assessment and label, the label-time reconstruction reproduces the assessment-time raw vector exactly, every block equal: the frozen blocks return from storage and the re-derived blocks land on the same values they had, so every index carries the number the assessment computed (´alg:runtime:reconstruction´). Before the pending entry stored the identity features and Sentinel lists, four of the six sources stood at zero in every training vector — this equality is the falsifier for that repair. |
//! | [`label_no_cp3_nan_survives`] | feature | Label-time assembly does not apply CP3 NaN sanitisation. A NaN in frozen Sentinel features should survive label-time assembly (propagated through standardisation). If CP3 were applied, the NaN would be replaced with 0.0. CP5 in the model owner catches NaN corruption at a higher level. A non-finite value in a frozen extraction survives label-time assembly and standardisation rather than being quietly replaced by zero. Sanitising here would turn a corrupt stored row into a plausible-looking training example and teach the model from it; leaving the value intact keeps the corruption detectable by the check that guards the model itself. |
//! | [`frozen_slot_survives_an_axis_registration`] | feature | A pending entry that outlives an axis registration keeps its alignment. The stored extraction was taken with one spatial axis and the slot now carries two, so the extraction is two positions narrower than the slot — and the two are not at the end of it. The per-axis outcome-memory pairs are the fifth of the extraction's six groups and the batch context feature is the sixth (´def:extraction:slot´), so the position the corpus puts last is the one a prefix copy loses: it stopped two short, leaving the batch context value to be read at the position the new pair now owns while the batch context's own position took zero (´entry:assayer:wl-feature-frozen-slot-truncation´). Placed group by group instead, the surviving pair lands at its own axis's position, the pair the assessment never held stays zero, and the batch context is last on both sides. |
//! | [`frozen_slot_survives_a_middle_axis_deregistration`] | feature | A pending entry that outlives a *middle* deregistration keeps every surviving axis's memory on its own axis. Three spatial axes stood when the assessment was made — seven, eight, nine — and axis eight is retired before the label arrives. Ranks alone cannot survive that: axis nine held rank two and now holds rank one, so a placement matching stored rank to current rank would hand axis nine the memory axis eight recorded and drop axis nine's own off the end. That is the boundary the wave-seven repair stated rather than crossed (´entry:assayer:wl-feature-frozen-slot-truncation´). Matched by identity there is no such shift. Axis seven's pair stays where it was, axis nine's pair moves back with the axis itself, the retired axis's pair is dropped rather than donated, and the batch context is last on both sides. The values are hand-picked so the failure is legible: axis eight's pair is (21, 22), and the test asserts those two numbers appear nowhere in the vector at all. |
//! | [`frozen_slot_survives_a_middle_axis_registration`] | feature | A pending entry that outlives a *middle* registration leaves the newcomer's pair blank and carries the later axis past it. The assessment held axes seven and nine; axis eight is registered between them before the label arrives. Axis nine's memory therefore belongs two positions further along than the rank it was stored at, and the position it used to occupy belongs to an axis that did not exist when the assessment was made — so that position takes zero, which is the honest value for evidence nobody recorded. This is the registration case the wave-seven test covers, moved into the middle of the tail: a rank match handles a registration at the end and nothing else. |
//! | [`frozen_slot_drops_a_retired_axis_rather_than_aliasing_a_newcomer`] | feature | Retiring one axis and registering another leaves the newcomer reading zero rather than inheriting the retired axis's memory. This is the case a width check cannot see. Axis eight is deregistered and axis nine registered between the assessment and the label, so the stored extraction and the current slot are exactly the same width — sixty-four — and every position of the stored vector has a position to go to. A rank match would therefore look entirely well-behaved while handing axis nine the outcome memory axis eight accumulated: two axes that never coexisted, joined by nothing but a shared ordinal. Matched by identity, axis eight has no position and its pair is dropped, while axis nine is named by nothing stored and keeps its zeros. The assertion is the strong one — axis eight's values (21, 22) appear nowhere in the reconstructed vector. |

//! Crate-level tests for feature vector assembly (´chap:spec:feature-vector´).
//!
//! Assembly is where the layout stops being a plan and becomes a vector: each
//! component the system has computed is written into the indices the dimension
//! map assigned it, interactions are formed from those raw values, and the whole
//! vector is standardised in one pass. These tests fix where each component
//! lands and that nothing is left unwritten.
//!
//! Label-time assembly runs the same layout against extractions frozen at
//! assessment time, and the population may have changed in between. The tests
//! for that path fix what happens when a stored row no longer matches the slot
//! it must go into, and which parts of the vector are deliberately left blank
//! because they described conditions that no longer exist.

use std::collections::HashMap;

use indexmap::IndexMap;

use crate::feature::assembly::{
    CrossDimensionAggregateFeatures, CurrentLabelState, FrozenPendingView, IdentityAggregateFeatures, IdentityAxisFeatures,
    IdentityDimensionFeatures, IdentityMeasurementFeatures, assemble_and_standardise_assessment, assemble_assessment_raw,
    assemble_phi_for_label, assemble_phi_for_label_raw, extract_sub_scores,
};
use crate::feature::dimension_map::DimensionMap;
use crate::feature::interaction::{FeatureSelector, InteractionId, InteractionTemplate};
use crate::feature::standardisation::StandardisationConfig;
use crate::identity::CompetitiveCellId;
use crate::pending::{SentinelExtraction, StoredFeatures};
use crate::testing::dimensions::{axes, n_axes};
use crate::testing::{assert_finite, assert_near};
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ─────────────────────────────────────────────────────────────────────────────
// Test helpers
// ─────────────────────────────────────────────────────────────────────────────

fn test_dim_map() -> DimensionMap {
    let sentinels: IndexMap<SentinelId, ()> = [(SentinelId(1), ()), (SentinelId(2), ())].into_iter().collect();
    let dims = vec![DimensionId(1)];
    let cells = vec![CompetitiveCellId::new(0, 8), CompetitiveCellId::new(256, 8)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = std::iter::once((DimensionId(1), cells)).collect();

    DimensionMap::rebuild_no_interactions(0, &sentinels, &dims, &comp, &[])
}

fn identity_features(
    aggregate: IdentityAggregateFeatures,
    measurement: IdentityMeasurementFeatures,
    axis_features: HashMap<OutcomeAxisId, IdentityAxisFeatures>,
) -> HashMap<DimensionId, IdentityDimensionFeatures> {
    std::iter::once((
        DimensionId(1),
        IdentityDimensionFeatures::new(aggregate, measurement, axis_features),
    ))
    .collect()
}

fn zero_means_unit_vars(p: usize) -> (Vec<f64>, Vec<f64>) {
    (vec![0.0; p], vec![1.0; p])
}

// ─────────────────────────────────────────────────────────────────────────────
// assemble_and_standardise_assessment tests
// ─────────────────────────────────────────────────────────────────────────────

/// The leading slot is one even when every component handed to assembly is
/// empty, and it survives the standardisation pass unchanged. The model's
/// intercept is carried by that constant, so it has to be present in a vector
/// assembled from nothing at all just as much as in a fully populated one.
///
/// ´claim:feature:the-bias-slot-is-one-in-every-assembled-vector-including-an-empty-one´
/// ´test:crate:bias-always-one´
#[test]
fn bias_always_one() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig::default();

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    assert_eq!(phi[0], 1.0, "bias must be 1.0");
    assert_finite(&phi, "phi");
}

/// Assembly produces a vector of exactly the width the layout declares, and
/// every position in it is finite, even when no component supplied a value.
/// Positions nobody wrote to are zeros rather than uninitialised or absent, so
/// the model's dot product is always over the dimension its weights expect.
///
/// ´claim:feature:assembly-always-produces-a-finite-vector-of-exactly-the-declared-width´
/// ´test:crate:phi-length-matches-p´
#[test]
fn phi_length_matches_p() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig::default();

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    assert_eq!(phi.len(), dm.p);
    assert_finite(&phi, "phi");
}

/// The fifteen fleet-wide aggregates land in their given order directly after
/// the bias, each keeping the value it was handed. Their positions are the ones
/// interaction templates and the anchor projection address by absolute index,
/// so a permutation here would silently redirect every one of those references.
///
/// ´claim:feature:the-fifteen-aggregate-features-land-in-order-directly-after-the-bias´
/// ´test:crate:aggregates-at-correct-indices´
#[test]
fn aggregates_at_correct_indices() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let agg = [
        1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0, 15.0,
    ];

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &agg,
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    // Aggregates are at indices 1-15 (after bias)
    for (i, &expected) in agg.iter().enumerate() {
        assert_eq!(phi[1 + i], expected, "aggregate[{i}] mismatch");
    }
    assert_finite(&phi, "phi");
}

/// A Sentinel that supplied an occupied extraction has its slot opened with an
/// indicator of one, even though every extracted feature behind it is zero.
/// Measured calm and no measurement at all produce the same zeros, and this one
/// place is what lets the model tell them apart.
///
/// ´claim:feature:an-occupied-extraction-opens-its-slot-with-an-indicator-of-one-even-when-its-features-are-all-zero´
/// ´test:crate:sentinel-occupancy-one-when-present´
#[test]
fn sentinel_occupancy_one_when_present() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let mut extractions = HashMap::new();
    extractions.insert(
        SentinelId(1),
        SentinelExtraction::new(
            0,
            vec![0.0; dm.sentinel_feature_width],
            true, // occupancy = true
        ),
    );

    let phi = assemble_and_standardise_assessment(
        &dm,
        &extractions,
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    assert_eq!(phi[slot.start], 1.0, "occupancy should be 1.0");
}

/// A registered Sentinel that supplied no extraction at all still owns its
/// slot, opened with an indicator of zero. The slot is reserved by registration
/// rather than by reporting, so a Sentinel falling silent for a round does not
/// shift every later feature along and invalidate the model's weights.
///
/// ´claim:feature:a-registered-sentinel-that-supplied-nothing-keeps-its-slot-with-an-indicator-of-zero´
/// ´test:crate:sentinel-occupancy-zero-when-absent´
#[test]
fn sentinel_occupancy_zero_when_absent() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig::default();

    // No extraction for Sentinel 2
    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    let slot = dm.sentinel_slots.get(&SentinelId(2)).unwrap();
    assert_eq!(phi[slot.start], 0.0, "occupancy should be 0.0 for absent Sentinel");
}

/// Exactly the cells named active carry an indicator of one; every other
/// registered cell keeps a zero at its own index. The indicators are a
/// membership statement about this particular request, so a cell that exists but
/// is not being competed in this round must read as absent rather than merely
/// unwritten.
///
/// ´claim:feature:only-the-cells-named-active-carry-an-indicator-of-one-while-the-rest-stay-zero´
/// ´test:crate:competitive-indicator-active´
#[test]
fn competitive_indicator_active() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let active_cell = CompetitiveCellId::new(0, 8);
    let mut active_cells = HashMap::new();
    active_cells.insert(DimensionId(1), vec![active_cell]);

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &active_cells,
        &means,
        &vars,
        &config,
    );

    let cell_map = dm.competitive_indices.get(&DimensionId(1)).unwrap();
    let idx = *cell_map.get(&active_cell).unwrap();
    assert_eq!(phi[idx], 1.0, "active cell indicator should be 1.0");

    // Check inactive cell
    let inactive_cell = CompetitiveCellId::new(256, 8);
    let inactive_idx = *cell_map.get(&inactive_cell).unwrap();
    assert_eq!(phi[inactive_idx], 0.0, "inactive cell indicator should be 0.0");
}

/// Each identity dimension's block opens with its three structural aggregates —
/// coverage depth, active fraction and total importance — in that order at the
/// block's start. These are the positions label-time assembly can re-derive
/// later, which is why they occupy the front of the block rather than being
/// interleaved with the measurements behind them.
///
/// ´claim:feature:the-three-structural-identity-aggregates-open-each-dimensions-block-in-order´
/// ´test:crate:identity-aggregate-features´
#[test]
fn identity_aggregate_features() {
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dims = vec![DimensionId(1)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();

    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &dims, &comp, &[]);
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let id_features = identity_features(
        IdentityAggregateFeatures::new(0.5, 0.3, 0.8),
        IdentityMeasurementFeatures::default(),
        HashMap::new(),
    );

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &id_features,
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    let range = dm.id_dim_ranges.get(&DimensionId(1)).unwrap();
    assert_eq!(phi[range.start], 0.5);
    assert_eq!(phi[range.start + 1], 0.3);
    assert_eq!(phi[range.start + 2], 0.8);
}

/// Run through the full assembly rather than against a hand-filled vector, an
/// interaction still holds the product of its two operands — a Sentinel feature
/// that arrived through an extraction times an aggregate that arrived as a
/// separate argument. The operands reach their positions by quite different
/// routes, and the interaction reads them from the vector after both have
/// landed.
///
/// (´claim:feature:an-interaction-slot-holds-the-plain-product-of-its-two-operands-read-from-their-own-positions´)
/// ´test:crate:assembly-interaction-correct-product´
#[test]
fn assembly_interaction_correct_product() {
    // Setup: 1 Sentinel, 1 Type-1 interaction.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let templates = vec![InteractionTemplate::type1(0, FeatureSelector::AggregateFeature(0))]; // sentinel offset=0, context=agg[0]
    let dm = DimensionMap::rebuild(0, &sentinels, &[], &IndexMap::new(), &[], templates);

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Aggregate: agg[0] = 2.5
    let agg = {
        let mut a = [0.0_f64; 15];
        a[0] = 2.5;
        a
    };

    // Sentinel extraction with feature[0] = 0.8
    let mut extraction_features = vec![0.0f32; dm.sentinel_feature_width];
    extraction_features[0] = 0.8;

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, extraction_features, true));

    let phi = assemble_and_standardise_assessment(
        &dm,
        &extractions,
        &agg,
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    // Find interaction index.
    let int_id = InteractionId::per_sentinel(0, SentinelId(1));
    let idx = dm.interaction_indices[&int_id];

    // Product: 0.8 × 2.5 = 2.0 (f32 precision)
    assert_near(phi[idx], 2.0, 1e-6, "interaction product");
    assert_finite(&phi, "phi");
}

/// The vector assembly returns has already been standardised against the
/// running statistics: a feature two deviations above its recorded mean comes
/// back as two, while the bias keeps its constant one. Standardisation is part
/// of assembly rather than a step callers must remember, so no caller can hand
/// the model a vector on the wrong scale.
///
/// ´claim:feature:assembly-returns-an-already-standardised-vector-with-the-bias-left-alone´
/// ´test:crate:assembly-standardisation-applied´
#[test]
fn assembly_standardisation_applied() {
    let dm = test_dim_map();

    // Means and variances for non-identity transformation.
    let mut means = vec![0.0; dm.p];
    let mut vars = vec![1.0; dm.p];

    // Set agg[0] mean=3, var=4 → feature 5, standardised = (5-3)/√4 = 1.0
    means[1] = 3.0; // agg[0] is at index 1
    vars[1] = 4.0;

    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let agg = {
        let mut a = [0.0_f64; 15];
        a[0] = 5.0;
        a
    };

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &agg,
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    // Standardised: (5 - 3) / √4 = 1.0
    assert_near(phi[1], 1.0, 1e-14, "standardised agg[0]");

    // Bias should still be 1.0 (never standardised).
    assert_eq!(phi[0], 1.0, "Bias unchanged");
    assert_finite(&phi, "phi");
}

/// Signal values are written into the signal block in the order supplied,
/// starting at the block's own beginning. Signals are positional by declaration
/// order and carry no identity of their own, so their meaning rests entirely on
/// arriving in the same order they were declared in.
///
/// ´claim:feature:signal-values-are-written-into-the-signal-block-in-the-order-supplied´
/// ´test:crate:assembly-signal-features´
#[test]
fn assembly_signal_features() {
    // Create dimension map with 3 signal features.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(3, &sentinels, &[], &IndexMap::new(), &[]);

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let signals = [1.5, 2.5, 3.5];

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &signals,
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    assert_eq!(phi[dm.sig_range.start], 1.5);
    assert_eq!(phi[dm.sig_range.start + 1], 2.5);
    assert_eq!(phi[dm.sig_range.start + 2], 3.5);
}

/// The five measurement features — the alarm pair, peak suspicion, the deepest
/// cell's volatility and the competitive-cell indicator — follow the structural
/// aggregates within each dimension's block, in that order. They describe what
/// was measured at a particular moment rather than the dimension's standing
/// shape, which is why they sit behind the prefix that can be recomputed later.
///
/// ´claim:feature:the-five-identity-measurement-features-follow-the-structural-aggregates-in-each-block´
/// ´test:crate:assembly-dimension-measurement´
#[test]
fn assembly_dimension_measurement() {
    // Create dimension map with one identity dimension.
    let sentinels: IndexMap<SentinelId, ()> = IndexMap::new();
    let dims = vec![DimensionId(1)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();

    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &dims, &comp, &[]);
    assert!(dm.id_dim_ranges.contains_key(&DimensionId(1)), "Dimension range should exist");

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let id_features = identity_features(
        IdentityAggregateFeatures::default(),
        IdentityMeasurementFeatures::new(1.0, 2.0, 3.0, 4.0, 1.0),
        HashMap::new(),
    );

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &id_features,
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    let range = dm.id_dim_ranges.get(&DimensionId(1)).unwrap();
    assert_eq!(phi[range.start + 3], 1.0); // max_alarm
    assert_eq!(phi[range.start + 4], 2.0); // mean_alarm
    assert_eq!(phi[range.start + 5], 3.0); // max_suspicion
    assert_eq!(phi[range.start + 6], 4.0); // volatility_deepest
    assert_eq!(phi[range.start + 7], 1.0); // has_competitive_cell
}

/// An outcome axis's three features are written at the offset the layout itself
/// records for that axis, found by asking the map rather than by assuming a
/// position. Axes are registered dynamically and identified by their own
/// identifiers, so assembly and any later reader agree on where an axis lives
/// only because both consult the same recorded offset.
///
/// ´claim:feature:an-outcome-axiss-three-features-land-at-the-offset-the-layout-records-for-that-axis´
/// ´test:crate:assembly-identity-axis-features´
#[test]
fn assembly_identity_axis_features() {
    let sentinels: IndexMap<SentinelId, ()> = IndexMap::new();
    let dims = vec![DimensionId(1)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();
    let axis_ids = [OutcomeAxisId(7)];

    let dm = DimensionMap::rebuild_with_axes_no_interactions(0, &sentinels, &dims, &comp, &[], &axis_ids);
    let dim_range = dm.id_dim_ranges.get(&DimensionId(1)).unwrap().clone();
    let axis_offset = dm.identity_axis_offset(OutcomeAxisId(7)).unwrap();

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let mut axis_features = HashMap::new();
    axis_features.insert(OutcomeAxisId(7), IdentityAxisFeatures::new(0.25, 42.0, 0.5));
    let id_features = identity_features(
        IdentityAggregateFeatures::default(),
        IdentityMeasurementFeatures::default(),
        axis_features,
    );

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &id_features,
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    assert_eq!(phi[dim_range.start + axis_offset], 0.25);
    assert_eq!(phi[dim_range.start + axis_offset + 1], 42.0);
    assert_eq!(phi[dim_range.start + axis_offset + 2], 0.5);
}

/// An extracted row is copied into its own Sentinel's slot in order, beginning
/// one place past the occupancy indicator. The row's internal ordering is the
/// one extraction fixed, so a feature's offset within an extraction and its
/// offset within a slot differ by exactly that one place — which is what
/// interaction templates count on when they address a Sentinel feature.
///
/// ´claim:feature:an-extracted-row-is-copied-into-its-slot-in-order-beginning-one-place-past-the-occupancy´
/// ´test:crate:assembly-sentinel-features-placed´
#[test]
fn assembly_sentinel_features_placed() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Create extraction with known feature values.
    let mut features = vec![0.0f32; dm.sentinel_feature_width];
    features[0] = 1.0;
    features[1] = 2.0;
    features[2] = 3.0;

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));

    let phi = assemble_and_standardise_assessment(
        &dm,
        &extractions,
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    // Features start at slot.start + 1 (after occupancy).
    assert_eq!(phi[slot.start + 1], 1.0);
    assert_eq!(phi[slot.start + 2], 2.0);
    assert_eq!(phi[slot.start + 3], 3.0);
}

/// With no Sentinels registered at all, assembly still returns the sixteen
/// slots the unconditional blocks occupy, bias included and every value finite.
/// A system before its first Sentinel registers can therefore be assessed
/// against fleet-wide evidence alone rather than failing for want of a
/// contributor.
///
/// (´claim:feature:assembly-always-produces-a-finite-vector-of-exactly-the-declared-width´)
/// ´test:crate:assembly-zero-sentinels´
#[test]
fn assembly_zero_sentinels() {
    // No Sentinels registered.
    let dm = DimensionMap::rebuild_no_interactions(0, &IndexMap::new(), &[], &IndexMap::new(), &[]);
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig::default();

    let phi = assemble_and_standardise_assessment(
        &dm,
        &HashMap::new(),
        &[0.0; 15],
        &HashMap::new(),
        &CrossDimensionAggregateFeatures::default(),
        &[],
        &HashMap::new(),
        &means,
        &vars,
        &config,
    );

    // Should still have bias + aggregates.
    assert_eq!(phi.len(), 16); // bias(1) + agg(15)
    assert_eq!(phi[0], 1.0, "Bias present");
    assert_finite(&phi, "phi");
}

// ─────────────────────────────────────────────────────────────────────────────
// extract_sub_scores tests
// ─────────────────────────────────────────────────────────────────────────────

/// Only occupied extractions contribute sub-scores to the fleet aggregate: an
/// unoccupied one is left out entirely rather than entering as a row of zeros,
/// and the surviving Sentinel's four leading z-scores come through as its own.
/// The aggregate's means, spreads and concordance fractions are all divided by
/// the number of contributors, so counting a silent Sentinel would dilute every
/// one of them toward zero.
///
/// ´claim:feature:only-occupied-extractions-contribute-sub-scores-to-the-fleet-aggregate´
/// ´test:crate:extract-sub-scores-from-extractions´
#[test]
fn extract_sub_scores_from_extractions() {
    let mut extractions = HashMap::new();

    // Extraction with known z-scores
    let mut features = vec![0.0f32; 60];
    features[0] = 1.0; // N z-score
    features[1] = 2.0; // D z-score
    features[2] = 0.5; // S z-score
    features[3] = 0.25; // C z-score (exact in f32)

    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));

    // Non-reporting Sentinel (occupancy = false)
    extractions.insert(SentinelId(2), SentinelExtraction::new(0, vec![0.0; 60], false));

    let sub_scores = extract_sub_scores(&extractions);

    // Only reporting Sentinel should be included
    assert_eq!(sub_scores.len(), 1);
    let (sid, scores) = &sub_scores[0];
    assert_eq!(*sid, SentinelId(1));
    assert_eq!(scores.z_scores[0], 1.0);
    assert_eq!(scores.z_scores[1], 2.0);
    assert_eq!(scores.z_scores[2], 0.5);
    assert_eq!(scores.z_scores[3], 0.25);
}

// ─────────────────────────────────────────────────────────────────────────────
// assemble_phi_for_label tests
// ─────────────────────────────────────────────────────────────────────────────

/// An empty frozen view for label-assembly tests that exercise one source.
fn empty_frozen<'a>(
    extractions: &'a HashMap<SentinelId, SentinelExtraction>,
    active: &'a [SentinelId],
    reporting: &'a [SentinelId],
    stored_spatial: &'a [OutcomeAxisId],
) -> FrozenPendingView<'a> {
    static EMPTY_BASE: std::sync::OnceLock<HashMap<DimensionId, StoredFeatures>> = std::sync::OnceLock::new();
    static EMPTY_AXIS: std::sync::OnceLock<HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>>> =
        std::sync::OnceLock::new();
    FrozenPendingView {
        extractions,
        active_sentinels: active,
        reporting_sentinels: reporting,
        signals: &[],
        base_features: EMPTY_BASE.get_or_init(HashMap::new),
        axis_features: EMPTY_AXIS.get_or_init(HashMap::new),
        spatial_axis_ids: stored_spatial,
    }
}

/// At label time the vector is rebuilt from the extractions frozen when the
/// assessment was made, each landing in its Sentinel's slot exactly as it did
/// then. Learning must attribute an outcome to the evidence the decision was
/// actually taken on, not to whatever those Sentinels are reporting by the time
/// the outcome arrives.
///
/// ´claim:feature:label-time-assembly-rebuilds-from-the-extractions-frozen-at-assessment-time´
/// ´test:crate:label-assembly-uses-frozen-extractions´
#[test]
fn label_assembly_uses_frozen_extractions() {
    let dm = test_dim_map();
    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Frozen extractions from assessment time.
    let mut features = vec![0.0f32; dm.sentinel_feature_width];
    features[0] = 5.0;

    let mut frozen_extractions = HashMap::new();
    frozen_extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];

    let frozen = empty_frozen(&frozen_extractions, &lists, &lists, &[]);
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    assert_eq!(phi[slot.start + 1], 5.0, "Frozen feature preserved");
    assert_finite(&phi, "phi");
}

/// A stored row narrower than the slot is placed group by group.
///
/// Scenario: a spatial outcome axis was registered between assessment and
/// labelling, increasing q from 60 to 62. The stored extraction has 60
/// positions and the current slot expects 62, and the two extra belong in the
/// middle rather than at the end.
///
/// The positions the assessment could not describe are the new axis's own
/// pair, and they are the ones left at zero: that axis did not exist when the
/// assessment was made, so zero is the honest value, and the alternative would
/// be refusing to learn from any assessment that predates a configuration
/// change. What is *not* left at zero is the batch context feature, which is
/// the sixth group and follows the pairs (´def:extraction:slot´) — it is
/// carried to its own new last position. A copy that stopped at the stored
/// width instead would leave the batch context value standing where the new
/// pair now lives and zero the batch context's own position
/// (´entry:assayer:wl-feature-frozen-slot-truncation´).
///
/// ´claim:feature:a-stored-row-narrower-than-the-current-slot-is-copied-as-far-as-it-goes-and-the-rest-left-zero´
/// ´test:crate:label-narrower-stored-row-blanks-only-the-new-axis-pair´
#[test]
fn label_narrower_stored_row_blanks_only_the_new_axis_pair() {
    // Current dim_map with 1 spatial axis → q = 62, slot = 63.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &n_axes(1));
    assert_eq!(dm.sentinel_feature_width, 62, "q should be 62 with m_s=1");

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Stored extraction from assessment time when m_s was 0 → q_old = 60.
    // With no spatial axes the batch context is the sixtieth position, the
    // pairs being empty.
    let mut features = vec![0.0f32; 60];
    features[0] = 7.0;
    features[59] = 3.0; // the batch context feature

    let mut frozen_extractions = HashMap::new();
    frozen_extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];

    let frozen = empty_frozen(&frozen_extractions, &lists, &lists, &[]);
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    // The fixed run ahead of the pairs copies as itself.
    assert_eq!(phi[slot.start + 1], 7.0, "First stored feature copied");
    // The newly registered axis's own pair is what stays at zero.
    assert_eq!(phi[slot.start + 60], 0.0, "New axis compressed valence is zero");
    assert_eq!(phi[slot.start + 61], 0.0, "New axis raw valence is zero");
    // And the batch context is carried to its own new last position.
    assert_eq!(phi[slot.start + 62], 3.0, "Batch context placed last");
}

/// A stored row wider than the slot is placed group by group, so what it
/// loses is decided by axis ownership rather than by where the slot ends.
///
/// Scenario: a spatial outcome axis was deregistered between assessment and
/// labelling, decreasing q from 62 to 60. The stored extraction has 62
/// features; the current slot expects 60. The fixed run copies as itself, the
/// retired axis's pair is dropped, the batch context moves back into the
/// position the pair vacated, and no write exceeds slot_range.end.
///
/// A stored row wider than its slot loses exactly the retired axis's own pair
/// and nothing else: what is dropped is chosen by the axis that owned it having
/// no position left in the layout, rather than by where the slot happens to
/// end. The batch context feature
/// follows the pairs (´def:extraction:slot´), so a copy truncating at the slot
/// boundary would discard it along with half a pair
/// (´entry:assayer:wl-feature-frozen-slot-truncation´); placed by group it
/// survives at its own new last position. The vector's total width is
/// unchanged and no discarded value appears anywhere in it, so a deregistered
/// axis cannot corrupt a neighbouring Sentinel's slot at label time.
///
/// ´claim:feature:a-stored-row-wider-than-the-current-slot-loses-the-positions-whose-axis-the-layout-retired´
/// ´test:crate:label-wider-stored-row-loses-only-the-retired-axis-pair´
#[test]
fn label_wider_stored_row_loses_only_the_retired_axis_pair() {
    // Current dim_map with 0 spatial axes → q = 60, slot = 61.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &[]);
    assert_eq!(dm.sentinel_feature_width, 60, "q should be 60 with m_s=0");

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Stored extraction from assessment time when m_s was 1 → q_old = 62.
    // Positions 59 and 60 are the retired axis's pair; 61 is the batch context.
    let mut features = vec![0.0f32; 62];
    features[0] = 9.0;
    features[59] = 99.0; // retired axis compressed valence
    features[60] = 98.0; // retired axis raw valence
    features[61] = 4.0; // the batch context feature

    let mut frozen_extractions = HashMap::new();
    frozen_extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];

    // The assessment was taken with axis one spatially enabled; it has since
    // been deregistered, so the current layout gives it no position at all.
    let stored_spatial = axes(&[1]);
    let frozen = empty_frozen(&frozen_extractions, &lists, &lists, &stored_spatial);
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    // The fixed run copies as itself and the batch context takes the last
    // position, the retired axis's pair being the only thing dropped.
    assert_eq!(phi[slot.start + 1], 9.0, "First stored feature copied");
    assert_eq!(phi[slot.start + 60], 4.0, "Batch context placed last");
    // Verify no write past slot end: element after slot belongs to next block.
    assert_eq!(phi.len(), dm.p, "Vector length unchanged");
    // The retired axis's pair (99.0, 98.0) appears nowhere in the vector.
    assert!(
        phi.iter().all(|&v| (v - 99.0).abs() > 1e-9 && (v - 98.0).abs() > 1e-9),
        "the retired axis's values must not survive anywhere in the vector"
    );
}

/// The frozen identity features are placed from storage: behind each
/// dimension's re-derived structural triple, the five stored measurement
/// values land at their positions and the stored per-axis values land at the
/// offset the layout records for the axis, while a dimension with nothing
/// stored keeps zeros there. Sources one and three of the reconstruction read
/// these values from the pending entry (´alg:runtime:reconstruction´) — they
/// recorded what was measured at assessment time, exist nowhere but the
/// buffer, and re-deriving them at label time would attach whatever is true
/// today to a decision taken earlier.
///
/// ´claim:feature:label-time-assembly-places-the-frozen-identity-features-from-storage´
/// ´test:crate:label-identity-frozen-features-placed-from-storage´
#[test]
fn label_identity_frozen_features_placed_from_storage() {
    // Dimension map with two identity dimensions and one outcome axis.
    let sentinels: IndexMap<SentinelId, ()> = IndexMap::new();
    let dims = vec![DimensionId(1), DimensionId(2)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();
    let axis_ids = [OutcomeAxisId(7)];
    let dm = DimensionMap::rebuild_with_axes_no_interactions(0, &sentinels, &dims, &comp, &[], &axis_ids);
    let range = dm.id_dim_ranges.get(&DimensionId(1)).unwrap().clone();
    let axis_offset = dm.identity_axis_offset(OutcomeAxisId(7)).unwrap();

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Frozen storage for dimension 1 only; dimension 2 stored nothing.
    let base_features: HashMap<DimensionId, StoredFeatures> =
        HashMap::from([(DimensionId(1), vec![1.5f32, 0.5, 2.25, 0.125, 1.0].into())]);
    let axis_features: HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>> = HashMap::from([(
        DimensionId(1),
        HashMap::from([(OutcomeAxisId(7), vec![0.25f32, 42.0, 0.5].into())]),
    )]);

    let extractions = HashMap::new();
    let frozen = FrozenPendingView {
        extractions: &extractions,
        active_sentinels: &[],
        reporting_sentinels: &[],
        signals: &[],
        base_features: &base_features,
        axis_features: &axis_features,
        spatial_axis_ids: &[],
    };
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    // The five frozen measurement values sit behind the structural triple.
    assert_eq!(phi[range.start + 3], 1.5, "max alarm placed from storage");
    assert_eq!(phi[range.start + 4], 0.5, "mean alarm placed from storage");
    assert_eq!(phi[range.start + 5], 2.25, "max suspicion placed from storage");
    assert_eq!(phi[range.start + 6], 0.125, "volatility placed from storage");
    assert_eq!(phi[range.start + 7], 1.0, "competitive indicator placed from storage");
    // The frozen per-axis values sit at the axis's recorded offset.
    assert_eq!(phi[range.start + axis_offset], 0.25);
    assert_eq!(phi[range.start + axis_offset + 1], 42.0);
    assert_eq!(phi[range.start + axis_offset + 2], 0.5);
    // A dimension with nothing stored keeps zeros behind its triple.
    let range2 = dm.id_dim_ranges.get(&DimensionId(2)).unwrap();
    for (i, &val) in phi[range2.start + 3..range2.end].iter().enumerate() {
        assert_eq!(val, 0.0, "unstored dimension position {i} stays zero");
    }
}

/// One slot per currently registered Sentinel, filled three ways from the
/// stored entry: the frozen extraction where the Sentinel was reporting,
/// occupancy with zero features where it was active but silent, and zeros
/// where it was registered after the assessment. Telling the second case from
/// the third is exactly what the stored Sentinel lists exist for
/// (´alg:runtime:reconstruction´) — the extraction map alone cannot say
/// whether a Sentinel with no entry was silent or absent.
///
/// ´claim:feature:label-time-slots-fill-three-ways-from-the-stored-sentinel-lists´
/// ´test:crate:label-slots-fill-three-ways´
#[test]
fn label_slots_fill_three_ways() {
    // Three registered Sentinels now; at assessment time, Sentinel 1 was
    // reporting, Sentinel 2 was active but silent, Sentinel 3 not yet
    // registered.
    let sentinels: IndexMap<SentinelId, ()> = [(SentinelId(1), ()), (SentinelId(2), ()), (SentinelId(3), ())]
        .into_iter()
        .collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &[]);

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    let mut features = vec![0.0f32; dm.sentinel_feature_width];
    features[0] = 5.0;
    let mut frozen_extractions = HashMap::new();
    frozen_extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));

    let active = [SentinelId(1), SentinelId(2)];
    let reporting = [SentinelId(1)];
    let frozen = empty_frozen(&frozen_extractions, &active, &reporting, &[]);
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    let slot1 = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    assert_eq!(phi[slot1.start], 1.0, "reporting Sentinel: stored occupancy");
    assert_eq!(phi[slot1.start + 1], 5.0, "reporting Sentinel: stored features");

    let slot2 = dm.sentinel_slots.get(&SentinelId(2)).unwrap();
    assert_eq!(phi[slot2.start], 1.0, "active-but-silent Sentinel: occupancy");
    for (i, &val) in phi[slot2.start + 1..slot2.end].iter().enumerate() {
        assert_eq!(val, 0.0, "active-but-silent Sentinel: feature {i} zero");
    }

    let slot3 = dm.sentinel_slots.get(&SentinelId(3)).unwrap();
    for (i, &val) in phi[slot3.start..slot3.end].iter().enumerate() {
        assert_eq!(val, 0.0, "later-registered Sentinel: position {i} zero");
    }
}

/// With the working state unchanged between assessment and label, the
/// label-time reconstruction reproduces the assessment-time raw vector
/// exactly, every block equal: the frozen blocks return from storage and the
/// re-derived blocks land on the same values they had, so every index carries
/// the number the assessment computed (´alg:runtime:reconstruction´). Before
/// the pending entry stored the identity features and Sentinel lists, four of
/// the six sources stood at zero in every training vector — this equality is
/// the falsifier for that repair.
///
/// ´claim:feature:reconstruction-under-unchanged-state-reproduces-the-assessment-vector-exactly´
/// ´test:crate:label-reconstruction-matches-assessment-under-unchanged-state´
#[test]
fn label_reconstruction_matches_assessment_under_unchanged_state() {
    // A dimension map exercising every block: two Sentinels, one identity
    // dimension with competitive cells, one outcome axis, two signals.
    let sentinels: IndexMap<SentinelId, ()> = [(SentinelId(1), ()), (SentinelId(2), ())].into_iter().collect();
    let dims = vec![DimensionId(1)];
    let cells = vec![CompetitiveCellId::new(0, 8), CompetitiveCellId::new(256, 8)];
    let comp: IndexMap<DimensionId, Vec<CompetitiveCellId>> = std::iter::once((DimensionId(1), cells)).collect();
    let axis_ids = [OutcomeAxisId(7)];
    let dm = DimensionMap::rebuild_with_axes_no_interactions(2, &sentinels, &dims, &comp, &[], &axis_ids);

    // Assessment-time inputs, every value exactly representable in f32 so
    // the storage round trip is exact.
    let mut features = vec![0.0f32; dm.sentinel_feature_width];
    features[0] = 5.0;
    features[3] = -3.5;
    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    // Sentinel 2 was active but silent: occupancy, no features (as the
    // assessment path records it).
    extractions.insert(SentinelId(2), SentinelExtraction::new(0, Vec::new(), true));

    let mut aggregates = [0.0f64; 15];
    for (i, v) in aggregates.iter_mut().enumerate() {
        *v = i as f64 * 0.25;
    }
    let triple = IdentityAggregateFeatures::new(0.5, 0.25, 0.75);
    let measurement = IdentityMeasurementFeatures::new(1.5, 0.5, 2.25, 0.125, 1.0);
    let mut axis_map = HashMap::new();
    axis_map.insert(OutcomeAxisId(7), IdentityAxisFeatures::new(0.25, 42.0, 0.5));
    let id_features = identity_features(triple.clone(), measurement.clone(), axis_map.clone());
    let cross = CrossDimensionAggregateFeatures {
        max_suspicion: 2.25,
        max_alarm: 1.5,
        max_volatility: 0.125,
        max_adverse_rate: 0.5,
        max_abs_compressed_valence: 0.75,
        max_abs_raw_valence: 3.0,
        has_any_competitive_cell: 1.0,
        max_coverage_depth: 0.5,
    };
    let signals = [0.5f64, -0.25];
    let active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> =
        HashMap::from([(DimensionId(1), vec![CompetitiveCellId::new(0, 8)])]);

    let phi_assessment = assemble_assessment_raw(&dm, &extractions, &aggregates, &id_features, &cross, &signals, &active_cells);

    // The label-time view: the frozen half exactly as the pending entry
    // stores it, the current half unchanged since assessment.
    let base_features: HashMap<DimensionId, StoredFeatures> = HashMap::from([(
        DimensionId(1),
        StoredFeatures::store(&measurement.as_slice(), crate::pending::StoragePrecision::Single),
    )]);
    let axis_features: HashMap<DimensionId, HashMap<OutcomeAxisId, StoredFeatures>> = HashMap::from([(
        DimensionId(1),
        axis_map
            .iter()
            .map(|(&axis, af)| {
                (
                    axis,
                    StoredFeatures::store(&af.as_slice(), crate::pending::StoragePrecision::Single),
                )
            })
            .collect(),
    )]);
    let identity_aggregates: HashMap<DimensionId, IdentityAggregateFeatures> = HashMap::from([(DimensionId(1), triple)]);
    let active = [SentinelId(1), SentinelId(2)];
    let reporting = [SentinelId(1)];

    let frozen = FrozenPendingView {
        extractions: &extractions,
        active_sentinels: &active,
        reporting_sentinels: &reporting,
        signals: &signals,
        base_features: &base_features,
        axis_features: &axis_features,
        spatial_axis_ids: &[],
    };
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross,
        active_cells: &active_cells,
    };

    let phi_label = assemble_phi_for_label_raw(&frozen, &current, &dm);

    assert_eq!(phi_label.len(), phi_assessment.len(), "same width");
    for (i, (&l, &a)) in phi_label.iter().zip(phi_assessment.iter()).enumerate() {
        assert_eq!(l, a, "position {i} differs between reconstruction and assessment");
    }
}

/// Label-time assembly does not apply CP3 NaN sanitisation.
///
/// A NaN in frozen Sentinel features should survive label-time assembly
/// (propagated through standardisation). If CP3 were applied, the NaN
/// would be replaced with 0.0. CP5 in the model owner catches NaN
/// corruption at a higher level.
///
/// A non-finite value in a frozen extraction survives label-time assembly and
/// standardisation rather than being quietly replaced by zero. Sanitising here
/// would turn a corrupt stored row into a plausible-looking training example
/// and teach the model from it; leaving the value intact keeps the corruption
/// detectable by the check that guards the model itself.
///
/// ´claim:feature:a-non-finite-frozen-feature-survives-label-time-assembly-rather-than-being-sanitised´
/// ´test:crate:label-no-cp3-nan-survives´
#[test]
fn label_no_cp3_nan_survives() {
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &[]);

    let (means, vars) = zero_means_unit_vars(dm.p);
    let config = StandardisationConfig {
        epsilon: 0.0,
        ..Default::default()
    };

    // Frozen extraction with a NaN feature.
    let mut features = vec![0.0f32; dm.sentinel_feature_width];
    features[0] = f32::NAN;

    let mut frozen_extractions = HashMap::new();
    frozen_extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];

    let frozen = empty_frozen(&frozen_extractions, &lists, &lists, &[]);
    let aggregates = [0.0; 15];
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &HashMap::new(),
        cross_dimension_features: &CrossDimensionAggregateFeatures::default(),
        active_cells: &HashMap::new(),
    };

    let phi = assemble_phi_for_label(&frozen, &current, &dm, &means, &vars, &config);

    let slot = dm.sentinel_slots.get(&SentinelId(1)).unwrap();
    // NaN propagates through standardisation: (NaN - 0) / 1 = NaN.
    assert!(
        phi[slot.start + 1].is_nan(),
        "NaN must survive label-time assembly (no CP3 sanitisation)"
    );
}

/// A pending entry that outlives an axis registration keeps its alignment.
/// The stored extraction was taken with one spatial axis and the slot now
/// carries two, so the extraction is two positions narrower than the slot —
/// and the two are not at the end of it. The per-axis outcome-memory pairs are
/// the fifth of the extraction's six groups and the batch context feature is
/// the sixth (´def:extraction:slot´), so the position the corpus puts last is
/// the one a prefix copy loses: it stopped two short, leaving the batch
/// context value to be read at the position the new pair now owns while the
/// batch context's own position took zero
/// (´entry:assayer:wl-feature-frozen-slot-truncation´). Placed group by group
/// instead, the surviving pair lands at its own axis's position, the pair the
/// assessment never held stays zero, and the batch context is last on both
/// sides.
///
/// ´claim:feature:a-frozen-extraction-keeps-its-groups-across-an-axis-registration´
/// ´test:crate:frozen-slot-survives-an-axis-registration´
#[test]
fn frozen_slot_survives_an_axis_registration() {
    // At assessment time: one spatial axis, so the extraction is 62 wide.
    let stored_axes = 1usize;
    let stored_width = 60 + 2 * stored_axes;
    let mut features = vec![0.0f32; stored_width];
    features[0] = 7.0; // a fixed-group position, ahead of the pairs
    features[59] = 3.0; // the one axis's compressed valence
    features[60] = 4.0; // and its raw valence
    features[61] = 9.0; // the batch context feature, last

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];
    let stored_spatial = axes(&[1]);
    let frozen = empty_frozen(&extractions, &lists, &lists, &stored_spatial);

    // By label time a second spatial axis is registered behind the first: the
    // slot is 64 wide and axis one keeps the position it already had.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &n_axes(2));

    let aggregates = [0.0; 15];
    let cross = CrossDimensionAggregateFeatures::default();
    let identity_aggregates: HashMap<DimensionId, IdentityAggregateFeatures> = HashMap::new();
    let active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> = HashMap::new();
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross,
        active_cells: &active_cells,
    };

    let phi = assemble_phi_for_label_raw(&frozen, &current, &dm);

    let slot = &dm.sentinel_slots[&SentinelId(1)];
    let features_start = slot.start + 1;
    assert_eq!(slot.end - features_start, 64, "two spatial axes widen the slot to 64");

    assert!((phi[features_start] - 7.0).abs() < 1e-12, "the fixed run copies as itself");
    assert!(
        (phi[features_start + 59] - 3.0).abs() < 1e-12,
        "the stored axis keeps its compressed valence at its own position"
    );
    assert!(
        (phi[features_start + 60] - 4.0).abs() < 1e-12,
        "and its raw valence beside it"
    );
    assert!(
        phi[features_start + 61].abs() < 1e-12,
        "the axis registered after the assessment takes zero"
    );
    assert!(phi[features_start + 62].abs() < 1e-12, "both halves of it, not one");
    assert!(
        (phi[features_start + 63] - 9.0).abs() < 1e-12,
        "and the batch context is last on both sides, got {}",
        phi[features_start + 63]
    );
}

/// A pending entry that outlives a *middle* deregistration keeps every
/// surviving axis's memory on its own axis.
///
/// Three spatial axes stood when the assessment was made — seven, eight, nine —
/// and axis eight is retired before the label arrives. Ranks alone cannot
/// survive that: axis nine held rank two and now holds rank one, so a placement
/// matching stored rank to current rank would hand axis nine the memory axis
/// eight recorded and drop axis nine's own off the end. That is the boundary
/// the wave-seven repair stated rather than crossed
/// (´entry:assayer:wl-feature-frozen-slot-truncation´).
///
/// Matched by identity there is no such shift. Axis seven's pair stays where it
/// was, axis nine's pair moves back with the axis itself, the retired axis's
/// pair is dropped rather than donated, and the batch context is last on both
/// sides. The values are hand-picked so the failure is legible: axis eight's
/// pair is (21, 22), and the test asserts those two numbers appear nowhere in
/// the vector at all.
///
/// ´claim:feature:a-stored-pair-follows-its-own-axis-across-a-middle-deregistration´
/// ´test:crate:frozen-slot-survives-a-middle-axis-deregistration´
#[test]
fn frozen_slot_survives_a_middle_axis_deregistration() {
    // At assessment time: three spatial axes, so the extraction is 66 wide —
    // fifty-nine fixed positions, three pairs, one batch context.
    let mut features = vec![0.0f32; 66];
    features[0] = 7.0; // a fixed-group position, ahead of the pairs
    features[59] = 11.0; // axis seven's compressed valence
    features[60] = 12.0; // and its raw valence
    features[61] = 21.0; // axis eight's pair, the axis about to be retired
    features[62] = 22.0;
    features[63] = 31.0; // axis nine's pair, the successor that must not shift
    features[64] = 32.0;
    features[65] = 5.0; // the batch context feature, last

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];
    let stored_spatial = axes(&[7, 8, 9]);
    let frozen = empty_frozen(&extractions, &lists, &lists, &stored_spatial);

    // By label time axis eight is gone: two pairs, the slot 64 wide.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &axes(&[7, 9]));

    let aggregates = [0.0; 15];
    let cross = CrossDimensionAggregateFeatures::default();
    let identity_aggregates: HashMap<DimensionId, IdentityAggregateFeatures> = HashMap::new();
    let active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> = HashMap::new();
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross,
        active_cells: &active_cells,
    };

    let phi = assemble_phi_for_label_raw(&frozen, &current, &dm);

    let slot = &dm.sentinel_slots[&SentinelId(1)];
    let features_start = slot.start + 1;
    assert_eq!(slot.end - features_start, 64, "two surviving axes leave the slot 64 wide");

    assert!((phi[features_start] - 7.0).abs() < 1e-12, "the fixed run copies as itself");
    assert!(
        (phi[features_start + 59] - 11.0).abs() < 1e-12,
        "axis seven keeps its compressed valence, ahead of the retirement"
    );
    assert!((phi[features_start + 60] - 12.0).abs() < 1e-12, "and its raw valence");
    assert!(
        (phi[features_start + 61] - 31.0).abs() < 1e-12,
        "axis nine's own compressed valence moves back with the axis, got {}",
        phi[features_start + 61]
    );
    assert!(
        (phi[features_start + 62] - 32.0).abs() < 1e-12,
        "and its own raw valence beside it, got {}",
        phi[features_start + 62]
    );
    assert!(
        (phi[features_start + 63] - 5.0).abs() < 1e-12,
        "the batch context is last on both sides, got {}",
        phi[features_start + 63]
    );
    assert!(
        phi.iter().all(|&v| (v - 21.0).abs() > 1e-9 && (v - 22.0).abs() > 1e-9),
        "the retired axis's pair is dropped, not donated to its successor"
    );
}

/// A pending entry that outlives a *middle* registration leaves the newcomer's
/// pair blank and carries the later axis past it.
///
/// The assessment held axes seven and nine; axis eight is registered between
/// them before the label arrives. Axis nine's memory therefore belongs two
/// positions further along than the rank it was stored at, and the position it
/// used to occupy belongs to an axis that did not exist when the assessment was
/// made — so that position takes zero, which is the honest value for evidence
/// nobody recorded.
///
/// This is the registration case the wave-seven test covers, moved into the
/// middle of the tail: a rank match handles a registration at the end and
/// nothing else.
///
/// ´claim:feature:a-stored-pair-passes-over-an-axis-registered-into-the-middle-of-the-tail´
/// ´test:crate:frozen-slot-survives-a-middle-axis-registration´
#[test]
fn frozen_slot_survives_a_middle_axis_registration() {
    // At assessment time: axes seven and nine, so the extraction is 64 wide.
    let mut features = vec![0.0f32; 64];
    features[0] = 7.0;
    features[59] = 11.0; // axis seven's pair
    features[60] = 12.0;
    features[61] = 31.0; // axis nine's pair, at rank one
    features[62] = 32.0;
    features[63] = 5.0; // the batch context feature

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];
    let stored_spatial = axes(&[7, 9]);
    let frozen = empty_frozen(&extractions, &lists, &lists, &stored_spatial);

    // By label time axis eight sits between them: three pairs, slot 66 wide.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &axes(&[7, 8, 9]));

    let aggregates = [0.0; 15];
    let cross = CrossDimensionAggregateFeatures::default();
    let identity_aggregates: HashMap<DimensionId, IdentityAggregateFeatures> = HashMap::new();
    let active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> = HashMap::new();
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross,
        active_cells: &active_cells,
    };

    let phi = assemble_phi_for_label_raw(&frozen, &current, &dm);

    let slot = &dm.sentinel_slots[&SentinelId(1)];
    let features_start = slot.start + 1;
    assert_eq!(slot.end - features_start, 66, "three spatial axes widen the slot to 66");

    assert!((phi[features_start] - 7.0).abs() < 1e-12, "the fixed run copies as itself");
    assert!(
        (phi[features_start + 59] - 11.0).abs() < 1e-12,
        "axis seven is ahead of the registration and does not move"
    );
    assert!((phi[features_start + 60] - 12.0).abs() < 1e-12, "and its raw valence");
    assert!(
        phi[features_start + 61].abs() < 1e-12,
        "the axis registered into the middle takes zero, got {}",
        phi[features_start + 61]
    );
    assert!(phi[features_start + 62].abs() < 1e-12, "both halves of it, not one");
    assert!(
        (phi[features_start + 63] - 31.0).abs() < 1e-12,
        "axis nine's pair moves along with the axis, got {}",
        phi[features_start + 63]
    );
    assert!((phi[features_start + 64] - 32.0).abs() < 1e-12, "and its raw valence");
    assert!(
        (phi[features_start + 65] - 5.0).abs() < 1e-12,
        "the batch context is last on both sides, got {}",
        phi[features_start + 65]
    );
}

/// Retiring one axis and registering another leaves the newcomer reading zero
/// rather than inheriting the retired axis's memory.
///
/// This is the case a width check cannot see. Axis eight is deregistered and
/// axis nine registered between the assessment and the label, so the stored
/// extraction and the current slot are exactly the same width — sixty-four —
/// and every position of the stored vector has a position to go to. A rank
/// match would therefore look entirely well-behaved while handing axis nine the
/// outcome memory axis eight accumulated: two axes that never coexisted, joined
/// by nothing but a shared ordinal.
///
/// Matched by identity, axis eight has no position and its pair is dropped,
/// while axis nine is named by nothing stored and keeps its zeros. The
/// assertion is the strong one — axis eight's values (21, 22) appear nowhere in
/// the reconstructed vector.
///
/// ´claim:feature:a-retired-axis-s-stored-pair-is-dropped-rather-than-aliased-onto-a-newly-registered-axis´
/// ´test:crate:frozen-slot-drops-a-retired-axis-rather-than-aliasing-a-newcomer´
#[test]
fn frozen_slot_drops_a_retired_axis_rather_than_aliasing_a_newcomer() {
    // At assessment time: axes seven and eight, so the extraction is 64 wide.
    let mut features = vec![0.0f32; 64];
    features[0] = 7.0;
    features[59] = 11.0; // axis seven's pair
    features[60] = 12.0;
    features[61] = 21.0; // axis eight's pair, about to be retired
    features[62] = 22.0;
    features[63] = 5.0; // the batch context feature

    let mut extractions = HashMap::new();
    extractions.insert(SentinelId(1), SentinelExtraction::new(0, features, true));
    let lists = [SentinelId(1)];
    let stored_spatial = axes(&[7, 8]);
    let frozen = empty_frozen(&extractions, &lists, &lists, &stored_spatial);

    // By label time axis eight is gone and axis nine has taken its rank. The
    // slot is the same width it was.
    let sentinels: IndexMap<SentinelId, ()> = std::iter::once((SentinelId(1), ())).collect();
    let dm = DimensionMap::rebuild_no_interactions(0, &sentinels, &[], &IndexMap::new(), &axes(&[7, 9]));

    let aggregates = [0.0; 15];
    let cross = CrossDimensionAggregateFeatures::default();
    let identity_aggregates: HashMap<DimensionId, IdentityAggregateFeatures> = HashMap::new();
    let active_cells: HashMap<DimensionId, Vec<CompetitiveCellId>> = HashMap::new();
    let current = CurrentLabelState {
        aggregate_features: &aggregates,
        identity_aggregates: &identity_aggregates,
        cross_dimension_features: &cross,
        active_cells: &active_cells,
    };

    let phi = assemble_phi_for_label_raw(&frozen, &current, &dm);

    let slot = &dm.sentinel_slots[&SentinelId(1)];
    let features_start = slot.start + 1;
    assert_eq!(slot.end - features_start, 64, "one axis out, one in: the width is unchanged");

    assert!((phi[features_start] - 7.0).abs() < 1e-12, "the fixed run copies as itself");
    assert!(
        (phi[features_start + 59] - 11.0).abs() < 1e-12,
        "the axis that survived keeps its own compressed valence"
    );
    assert!((phi[features_start + 60] - 12.0).abs() < 1e-12, "and its raw valence");
    assert!(
        phi[features_start + 61].abs() < 1e-12,
        "the newly registered axis reads zero, not its predecessor's memory, got {}",
        phi[features_start + 61]
    );
    assert!(phi[features_start + 62].abs() < 1e-12, "both halves of it, not one");
    assert!(
        (phi[features_start + 63] - 5.0).abs() < 1e-12,
        "the batch context is last on both sides, got {}",
        phi[features_start + 63]
    );
    assert!(
        phi.iter().all(|&v| (v - 21.0).abs() > 1e-9 && (v - 22.0).abs() > 1e-9),
        "the retired axis's values must not survive anywhere in the vector"
    );
}
