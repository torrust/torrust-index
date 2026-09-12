// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`register_sentinel_extends_all_models`] | lifespan | A Sentinel registration widens the operational model and its sister together, leaving the two at exactly the same dimension. The sister exists to be compared against the operational model, and a comparison between models of different widths is not a comparison at all. |
//! | [`register_sentinel_grows_the_standardisation_vectors_together`] | lifespan | The running means, the running variances and the per-feature class assignments all grow alongside the model and end at a common length. Standardisation is read positionally against the feature vector, so a disagreement of even one entry would silently rescale every feature after it. |
//! | [`register_sentinel_creates_outcome_ledger_partition`] | lifespan | Registration creates the Sentinel's outcome-ledger partition and roots it, so there is somewhere for its first observation to land. Ledger entries hang beneath that root, and a partition without one would have to be repaired on the ingestion path instead. |
//! | [`deregister_sentinel_marginalises_models`] | lifespan | Retiring a Sentinel reduces the model dimension: the departed Sentinel's block is marginalised out by Schur complement rather than left in place holding stale coefficients. What the model learned about the remaining features survives the removal, which is the whole reason to marginalise rather than rebuild. |
//! | [`deregister_sentinel_returns_marginalisation_diagnostics`] | lifespan | The completion payload is the aggregate formed during the chain application the removal was part of, not a later query of the lifetime ledger. On a fresh environment the two agree, and the error-kind partition accounts for every full-model marginalisation the application performed (´entry:health:marginalise-event´). |
//! | [`chain_operations_share_one_marginalisation_aggregate`] | lifespan | Two Sentinel retirements submitted together are one chain, and one chain is one Schur computation: both results carry the same chain index and the same aggregate, and the aggregate counts one full-model marginalisation per model rather than one per retirement. Reporting the same aggregate to both is not a convenience. The correction was computed once, over the union of the two removals' positions, against one partition of one matrix; there is no honest way to split its error partition back out per index set, because no per-index-set computation was ever performed. What a caller learns from its result is what happened to the models while its operation was being applied, and the chain is what was applied (´entry:health:marginalise-event´). |
//! | [`deregister_sentinel_compacts_standardisation`] | lifespan | The standardisation vectors shrink when a Sentinel's features are marginalised away, not merely the model itself. Compaction is what keeps every surviving feature's statistics at the index the model now expects to find it at. |
//! | [`deregister_sentinel_removes_outcome_ledger_partition`] | lifespan | Deregistration removes the Sentinel's ledger partition along with its model features. Sentinel ids are reusable, so a partition left standing would hand some future Sentinel the accumulated outcomes of its predecessor. |
//! | [`deregister_unknown_sentinel_is_noop`] | lifespan | A deregistration event for a Sentinel with no slot in the dimension map reports success and leaves the dimension untouched. The model owner processes events the public surface has already validated, so failing here would abort a whole compound batch over an entity that is already gone. |
//! | [`register_axis_extends_per_sentinel`] | lifespan | Registering a spatially-enabled outcome axis grows the model by two features in every registered Sentinel's slot — a compressed and a raw reading of the axis apiece. The axis is meant to be read spatially, which means each Sentinel's own view of it has to exist. |
//! | [`register_axis_zero_sentinels_noop`] | lifespan | With no Sentinels registered, adding an outcome axis leaves the dimension exactly where it was. Per-Sentinel features are per Sentinel: zero Sentinels buys zero of them, and the axis waits to be paid for until there is something to pay for. |
//! | [`register_axis_disabled_spatial_no_sentinel_growth`] | lifespan | An axis registered with spatial features disabled adds nothing to any Sentinel slot, yet its own outcome model is still created and present in the working copy. The spatial policy governs how the axis is measured, not whether it is predicted. |
//! | [`register_axis_with_identity_adds_per_dimension_features`] | lifespan | Even a non-spatial axis grows every live identity dimension's block by three features, recording its offset within that block. Identity is where an axis is read per entity rather than per Sentinel, so that reading exists regardless of whether the Sentinels also carry the axis. |
//! | [`register_axis_with_two_identity_dimensions_standardises_both`] | lifespan | Axis registration extends standardisation for every registered identity dimension: with two dimensions live, a new axis gives each dimension the compressed–raw–stability class triple, each position carrying its class prior, and the vectors end exactly as long as the models the same event extended. The reference configuration registers two identity dimensions, so a push that stopped after one triple would leave the reference deployment's second dimension standardising its axis features against the neutral resize fallback instead of the classes the corpus assigns (´req:standardisation:class-assignment´). The triples are read at the map's per-axis offset inside each dimension's own block rather than at the tail of the vector, because the rearrangement the rebuild now performs has already carried them from where the extension appended them to where the canonical layout puts them (´entry:assayer:wl-owner-layout-misalignment´); reading the tail would pin the misalignment as the design. |
//! | [`register_spatial_axis_with_identity_adds_identity_and_sentinel_features`] | lifespan | A spatial axis registered while both Sentinels and an identity dimension are live pays for both shapes at once: two features in each Sentinel slot plus three in the identity block. The per-Sentinel and per-identity views answer different questions about the same axis, so neither substitutes for the other. |
//! | [`deregister_axis_marginalises`] | lifespan | Retiring a spatial axis shrinks the model again, removing the per-Sentinel features its registration added. Without that, a deployment experimenting with axes would ratchet upward in dimension for the lifetime of the process. |
//! | [`deregister_axis_prunes_ledger_per_axis_rows`] | lifespan | The per-axis readings an axis accumulated are pruned from every ledger entry — the root and its children alike — when that axis is retired. Axis ids can be registered again later, and a surviving row would let the new axis read its predecessor's history as its own warm start. |
//! | [`register_identity_extends_and_creates_tracker`] | lifespan | cites (´claim:lifespan:the-first-identity-dimension-brings-its-own-block-and-the-cross-dimension-aggregate-block-with-it´) |
//! | [`deregister_identity_removes_tracker_and_marginalises`] | lifespan | cites (´claim:lifespan:deregistering-an-identity-dimension-takes-its-convergence-tracker-with-it´) |
//! | [`cell_entry_extends_model`] | lifespan | cites (´claim:lifespan:a-competitive-cell-entering-adds-its-indicator-and-nothing-more-when-no-interaction-templates-exist´) |
//! | [`cell_exit_marginalises_model`] | lifespan | When a cell exits the competitive set the model shrinks again, its indicator marginalised away with no manual bookkeeping — the rebuild that followed the entry had already recorded the cell, so the exit finds it. Competitive membership turns over constantly, so entry and exit have to be exact inverses or the dimension would climb with every reshuffle. |
//! | [`cell_lifecycle_events_feed_identity_tracker`] | lifespan | The competitive-set changes a submission carries land on the dimension's convergence tracker: three cell entries record three entries against the rebuilt current set, and a following mixed submission adds its exit and entry to the tallies while the tracked set follows the map. The measurement machinery was complete and fed by nothing; this is the feeding, and it is what stops the composite from reading steady state over an identity layer it has never measured. |
//! | [`compound_batch_processes_all_events`] | lifespan | A submission carrying several unrelated events — two Sentinel registrations and an identity dimension — yields one result per event, each reporting its own outcome. However the events are grouped for application, the answer to each of them is still its own: grouping is about the work, not about the reporting. |
//! | [`lifecycle_publishes_snapshot`] | lifespan | Processing a submission publishes a new snapshot carrying exactly the version the caller supplied, advancing past the version that stood before. Readers take the published snapshot rather than the working copy, so a structural change that was not published would be invisible however thoroughly it was applied. |
//! | [`lifecycle_signals_completion`] | lifespan | A submission carrying a completion channel receives its result on that channel once processing finishes. This is what lets a caller that must not proceed until the model has stopped referencing something — identity- dimension teardown above all — actually wait for it. |
//! | [`standardisation_consistent_after_extend`] | lifespan | cites (´claim:lifespan:standardisation-grows-with-the-model-and-its-three-vectors-stay-the-same-length´) |
//! | [`anchor_dimension_unchanged`] | lifespan | Through Sentinel registrations and an identity dimension alike, the anchor model's dimension never moves. The anchor is the fixed reference the blend falls back on, and a reference that changed shape whenever the structure did would be no anchor. |
//! | [`axis_model_extended_on_subsequent_sentinel_registration`] | lifespan | An axis model created at one dimension is itself extended when a Sentinel registers afterwards, ending at the operational model's new width with the rebuilt dimension map agreeing. Axis models are scored against the same feature vector as the operational model, so any of them lagging behind would be reading the wrong columns. |
//! | [`axis_model_marginalised_on_subsequent_sentinel_deregistration`] | lifespan | cites (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´) |
//! | [`axis_model_extended_on_subsequent_identity_dimension_registration`] | lifespan | cites (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´) |
//! | [`axis_model_extended_on_subsequent_cell_entry`] | lifespan | cites (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´) |
//! | [`all_axis_models_extended_when_new_axis_registered`] | lifespan | Registering a spatial axis while another axis already exists extends the older axis's model too, and creates the new one directly at the post- extension dimension rather than extending it afterwards. The new axis never existed at the narrower width, so it has nothing to be extended from. |
//! | [`register_then_deregister_restores_dimension`] | lifespan | Registering an identity dimension and then retiring it leaves the model at exactly its original dimension — the per-dimension block and the cross- dimension aggregate both go back. Structural change is meant to be reversible in size as well as in effect, or a long-lived deployment slowly pays for everything it ever tried. |
//! | [`two_identity_dimensions_in_one_submission_share_one_aggregate_block`] | lifespan | Two identity dimensions registered in one submission pay for the cross-dimension aggregate block once between them, and retired in one submission they take it with them, leaving the model exactly where it started. Neither pair shares a chain: a registration is order-free with nothing, and two identity-dimension removals are the removal family's one exception, so the second operation of each pair reads a map the first has already changed. The block's arrival and its departure are both conditioned on a count of the dimensions standing — the first dimension pays for it, the last takes it — and a count is exactly what one shared entry layout cannot tell two operations apart by. Planned against a single layout the pair would have bought the block twice and then abandoned it, and the rebuild's width check is what would have noticed. That is why the predicate excludes the pairing rather than assuming a removal's positions are always a function of its own entity. |
//! | [`compound_batch_2_exits_1_entry`] | lifespan | Two cell exits and one entry submitted together net to a single feature removed, and the batch publishes once per order-free chain rather than once per event: the two exits are order-free with each other and apply as one plan, the entry is order-free with nothing and applies as its own, so two snapshots go out at consecutive versions. Amortising the rebuild across events that commute is what makes batching worth doing under churn; doing it across events that do not would be batching an ordering away. |
//! | [`cross_entity_cascade_r27`] | lifespan | cites (´claim:lifespan:a-spatial-axis-costs-two-features-in-every-sentinel-slot´) |
//! | [`peak_registration_then_full_deregistration_restores_min_dimension`] | lifespan | Built to twenty Sentinels and ten spatial axes, the dimension is exactly the closed form the feature counts predict, with every Sentinel's slot range disjoint from its neighbours and inside the model. Torn back down to one Sentinel and no axes it returns to the minimum for that configuration, and the lone surviving slot starts precisely where the signal block ends and runs to the end — the index space is compacted rather than merely shortened, so nothing addresses a hole left by a departed neighbour. |
//! | [`gauntlet_deregister_register_churn_100_cycles`] | lifespan | A hundred rounds of retiring a randomly chosen Sentinel and registering a fresh one hold the operational model, the sister and the dimension map at one common width, with the standardisation vectors matching and the pool conserved at eight throughout. No slot range ever overlaps another or runs past the model, and no entry of any model's mean or of the running statistics ever stops being finite — repeated marginalisation and extension neither leak indices nor accumulate numerical damage. |
//! | [`middle_axis_retirement_keeps_each_pair_with_its_own_axis`] | lifespan | Retiring a spatial outcome axis from the middle of the layout renumbers every later axis's outcome-memory pair, and the permutation that rearranges the parameters keys that pair by the number rather than by the axis that owns it. With a distinguishable value planted at every coordinate first, each surviving axis's pair — in every Sentinel slot, across the posterior mean, the covariance, the precision and both standardisation vectors — still carries what that axis itself learned, and the retired axis's memory is gone rather than donated to its rank inheritor. The identity block's per-axis triple, keyed by the axis rather than by a rank, is the control beside it. |
//! | [`middle_axis_retirement_permutation_is_the_identity`] | lifespan | The rearrangement that carries a retirement into canonical order is the identity, and reaching it is what proves the two drops cancelled: the marginalisation compacted the retired axis's own pair out of the middle of every slot tail, the key filter dropped the *last*-ranked pair key instead, and the permutation those two produce together moves nothing because they leave the surviving axes at the same ranks in the same order. A refusal would leave the parameters exactly where the handlers put them, and a rearrangement would move them; reading only the values cannot tell a rearrangement that landed correctly from one that never ran. Pinning which of the three this path takes is what stops the doc from narrating one mechanism while the code runs another. |
//! | [`last_axis_retirement_keeps_each_pair_with_its_own_axis`] | lifespan | Retiring the last-ranked spatial axis renumbers nothing, so the drop the marginalisation makes and the drop the key filter makes are the same positions rather than merely the same count. It is the case the rank keying was always right on, and it stands here as the floor the middle case has to match. |
//! | [`first_axis_retirement_keeps_each_pair_with_its_own_axis`] | lifespan | Retiring the first-ranked spatial axis renumbers every other axis, which is the widest renumbering a single retirement can cause: the two drops are then at opposite ends of the slot tail. The cancellation is exact there too, so the distance between them is not what makes it work. |
//! | [`two_retirements_in_one_submission_apply_as_one_chain`] | lifespan | Two spatial axes retired in one submission are order-free with each other, so they gather into one chain and apply as one plan: a single joint marginalisation over the union of their positions, one rebuild, one publication. The surviving axis's outcome-memory pair in every Sentinel slot still carries what that axis learned across all five channels the invariant binds, its identity triple agrees as the control, and neither retired axis's memory is left anywhere in the vector. This is the boundary a single-event argument cannot reach, and it is where the sequential applier failed. It took each axis's rank from the authoritative model map, which every handler updated as it ran, and the Sentinel slot positions from the dimension map, which the rebuild refreshed only once after all of a submission's events. The second retirement therefore mixed a fresh rank with stale positions and addressed coordinates the first had already compacted away, which the marginalisation's own precondition refused. Both halves of that mixture are gone: a removal now addresses its pair from the map by the axis's own identifier, exactly as it already addressed its identity triple, and the chain holds that one map still until every operation in it has been planned (´entry:assayer:wl-owner-dereg-scan´). |
//! | [`chain_break_publishes_per_chain_and_survivors_keep_their_memory`] | lifespan | A submission whose events are not all order-free with one another is applied as several chains, publishes one snapshot per chain, and still leaves every survivor holding its own memory. Retiring one spatial axis, registering another, then retiring a third breaks the chain twice — a registration reads live entity state to decide how wide it is and takes its positions from the append, so it is order-free with nothing — and the three chains publish at three consecutive versions. The break is what makes the case safe rather than what makes it awkward: the operation that seals a chain begins from the layout that chain published, so the middle registration extends a vector the first retirement has already compacted, and the last retirement addresses a map the registration's own rebuild has already refreshed. The surviving pre-existing axis's pair, in every Sentinel slot and across all five bound channels, carries what it learned before any of it happened. |

//! Lifecycle event handling crate-level tests: the mutations that registration
//! and deregistration make (´dec:construction:six-methods´).
//!
//! These tests verify cross-module integration for lifecycle events,
//! including sentinel registration/deregistration, outcome axis, identity
//! dimension, and competitive cell lifecycle events, as well as compound
//! batches and dimension consistency invariants.
//!
//! # Cross-References
//!
//! - (´dec:construction:compound-batch´) — lifecycle events arriving together and publishing once
//! - (´inv:guarantee:axis-lifecycle´) — registration and deregistration as exact extension and marginalisation

use std::collections::HashMap;

use crate::config::types::AssayerConfig;
use crate::health::{IdentityConvergenceTracker, PlattConvergenceTracker};
use crate::identity::CompetitiveCellId;
use crate::ledger::OutcomeLedger;
use crate::owner::commands::{EventOutcome, LifecycleEvent, LifecycleSubmission};
use crate::owner::lifecycle::handle_lifecycle_submission;
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::{ModelConfig, WorkingCopy};
use crate::testing::ACK_DEADLINE;
use crate::testing::registrations::{axis_reg, sentinel_reg, spatial_axis_reg};
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Test Infrastructure
// ═══════════════════════════════════════════════════════════════════════════════

struct LifecycleEnv {
    working: WorkingCopy,
    shared: SharedState,
    platt_tracker: PlattConvergenceTracker,
    identity_trackers: HashMap<DimensionId, IdentityConvergenceTracker>,
    outcome_ledger: OutcomeLedger,
    config: AssayerConfig,
}

impl LifecycleEnv {
    fn new() -> Self {
        let model_config = ModelConfig::default();
        let working = WorkingCopy::cold_start(16, &model_config, 1000);
        let snapshot = working.to_snapshot(0);
        let shared = SharedState::new(snapshot);

        Self {
            working,
            shared,
            platt_tracker: PlattConvergenceTracker::new(),
            identity_trackers: HashMap::new(),
            outcome_ledger: OutcomeLedger::new(),
            config: AssayerConfig::default(),
        }
    }

    fn submit(&mut self, events: Vec<LifecycleEvent>, version: u64) -> Vec<crate::owner::commands::EventResult> {
        self.submit_reporting_version(events, version).0
    }

    /// Submits a batch whose first chain publishes at `first_version`, and
    /// reports the version the last chain published at — `first_version` plus
    /// one for every chain after the first
    /// (´dec:construction:compound-batch´).
    fn submit_reporting_version(
        &mut self,
        events: Vec<LifecycleEvent>,
        first_version: u64,
    ) -> (Vec<crate::owner::commands::EventResult>, u64) {
        let mut version = first_version;
        let submission = LifecycleSubmission {
            events,
            completion: None,
        };
        let result = handle_lifecycle_submission(
            &mut self.working,
            submission,
            &self.shared,
            &mut self.platt_tracker,
            &mut self.identity_trackers,
            &self.outcome_ledger,
            &self.config,
            &mut version,
            std::time::Instant::now(),
            &crate::types::PersistentTimestamp::new(0, 0),
        );
        (result.expect("lifecycle submission should succeed"), version)
    }
}

// ─── Registration helpers ────────────────────────────────────────────────────

// `sentinel_reg`, `axis_reg` and `spatial_axis_reg` live in
// [`crate::testing::registrations`] — shared with the other lifecycle tests.

// ═══════════════════════════════════════════════════════════════════════════════
// Sentinel Lifecycle Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel registration widens the operational model and its sister
/// together, leaving the two at exactly the same dimension. The sister exists
/// to be compared against the operational model, and a comparison between
/// models of different widths is not a comparison at all.
///
/// ´claim:lifespan:the-operational-and-sister-models-are-widened-together-and-stay-the-same-size´
/// ´test:crate:register-sentinel-extends-all-models´
#[test]
fn register_sentinel_extends_all_models() {
    let mut env = LifecycleEnv::new();
    let initial_op_dim = env.working.operational.dim();
    let initial_sis_dim = env.working.sister.dim();

    let results = env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].outcome, EventOutcome::Success);

    // Both models should be extended by 61 (slot_width: 1 occ + 60 q_base, no interactions)
    assert!(
        env.working.operational.dim() > initial_op_dim,
        "operational model should grow"
    );
    assert!(env.working.sister.dim() > initial_sis_dim, "sister model should grow");
    assert_eq!(
        env.working.operational.dim(),
        env.working.sister.dim(),
        "operational and sister should have same dimension"
    );
}

/// The running means, the running variances and the per-feature class
/// assignments all grow alongside the model and end at a common length.
/// Standardisation is read positionally against the feature vector, so a
/// disagreement of even one entry would silently rescale every feature after
/// it.
///
/// ´claim:lifespan:standardisation-grows-with-the-model-and-its-three-vectors-stay-the-same-length´
/// ´test:crate:register-sentinel-grows-the-standardisation-vectors-together´
#[test]
fn register_sentinel_grows_the_standardisation_vectors_together() {
    let mut env = LifecycleEnv::new();
    let initial_len = env.working.feature_means.len();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);

    assert!(env.working.feature_means.len() > initial_len, "feature_means should grow");
    assert_eq!(
        env.working.feature_means.len(),
        env.working.feature_variances.len(),
        "means and variances should have same length"
    );
    assert_eq!(
        env.working.feature_means.len(),
        env.working.feature_classes.len(),
        "means and classes should have same length"
    );
}

/// Registration creates the Sentinel's outcome-ledger partition and roots it,
/// so there is somewhere for its first observation to land. Ledger entries hang
/// beneath that root, and a partition without one would have to be repaired on
/// the ingestion path instead.
///
/// ´claim:lifespan:a-registered-sentinel-gets-a-rooted-ledger-partition-of-its-own´
/// ´test:crate:register-sentinel-creates-outcome-ledger-partition´
#[test]
fn register_sentinel_creates_outcome_ledger_partition() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);

    assert!(
        env.outcome_ledger.contains(SentinelId(1)),
        "registered Sentinel should have a ledger partition"
    );
    let lock = env.outcome_ledger.get(SentinelId(1)).expect("sentinel ledger");
    let guard = lock.read().unwrap();
    assert!(guard.has_root(), "new Sentinel ledger should be rooted");
}

/// Retiring a Sentinel reduces the model dimension: the departed Sentinel's
/// block is marginalised out by Schur complement rather than left in place
/// holding stale coefficients. What the model learned about the remaining
/// features survives the removal, which is the whole reason to marginalise
/// rather than rebuild.
///
/// ´claim:lifespan:retiring-a-sentinel-shrinks-the-model-rather-than-leaving-its-columns-behind´
/// ´test:crate:deregister-sentinel-marginalises-models´
#[test]
fn deregister_sentinel_marginalises_models() {
    let mut env = LifecycleEnv::new();

    // First register a sentinel
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    let dim_after_register = env.working.operational.dim();

    // Register the sentinel in the dimension map so deregister can find it.
    // The cold-start dimension is 16, so register appends at [16, 16+61=77).
    env.working.dimension_map.sentinel_slots.insert(
        SentinelId(1),
        crate::feature::dimension_map::IndexRange::new(16, dim_after_register),
    );

    // Deregister
    let results = env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 2);

    assert_eq!(results[0].outcome, EventOutcome::Success);
    assert!(
        env.working.operational.dim() < dim_after_register,
        "operational model should shrink after deregistration"
    );
}

/// The completion payload is the aggregate formed during the chain application
/// the removal was part of, not a later query of the lifetime ledger. On a
/// fresh environment the two agree, and the error-kind partition accounts for
/// every full-model marginalisation the application performed
/// (´entry:health:marginalise-event´).
///
/// ´claim:lifespan:lifecycle-completion-carries-the-chain-applications-aggregate-marginalisation-diagnostics´
/// ´test:crate:deregister-sentinel-returns-marginalisation-diagnostics´
#[test]
fn deregister_sentinel_returns_marginalisation_diagnostics() {
    let mut env = LifecycleEnv::new();
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    let end = env.working.operational.dim();
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, end));

    let results = env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 2);
    let diagnostics = results[0]
        .marginalisation
        .as_ref()
        .expect("a real removal carries aggregate diagnostics");
    assert_eq!(
        diagnostics.measured_events + diagnostics.bounded_events + diagnostics.unbounded_events,
        diagnostics.events
    );
    assert_eq!(diagnostics.events, env.working.marginalisation_errors.events);
    assert_eq!(
        diagnostics.corrections_skipped,
        env.working.marginalisation_errors.corrections_skipped()
    );
}

/// Two Sentinel retirements submitted together are one chain, and one chain is
/// one Schur computation: both results carry the same chain index and the same
/// aggregate, and the aggregate counts one full-model marginalisation per model
/// rather than one per retirement.
///
/// Reporting the same aggregate to both is not a convenience. The correction
/// was computed once, over the union of the two removals' positions, against one
/// partition of one matrix; there is no honest way to split its error partition
/// back out per index set, because no per-index-set computation was ever
/// performed. What a caller learns from its result is what happened to the
/// models while its operation was being applied, and the chain is what was
/// applied (´entry:health:marginalise-event´).
///
/// ´claim:lifespan:the-operations-of-one-chain-each-receive-that-chains-single-marginalisation-aggregate´
/// ´test:crate:chain-operations-share-one-marginalisation-aggregate´
#[test]
fn chain_operations_share_one_marginalisation_aggregate() {
    let mut env = LifecycleEnv::new();
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2)))], 2);

    let results = env.submit(
        vec![
            LifecycleEvent::DeregisterSentinel(SentinelId(1)),
            LifecycleEvent::DeregisterSentinel(SentinelId(2)),
        ],
        3,
    );

    assert_eq!(results.len(), 2, "one result per operation, whatever they were applied as");
    assert_eq!(
        results[0].chain_index, results[1].chain_index,
        "two removals are order-free, so they name one application",
    );

    let first = results[0]
        .marginalisation
        .as_ref()
        .expect("a removal that contributed positions carries its chain's aggregate");
    let second = results[1]
        .marginalisation
        .as_ref()
        .expect("a removal that contributed positions carries its chain's aggregate");
    assert_eq!(first, second, "one computation reported to both, not two computations");

    // The discriminating count: the operational model and its sister were
    // marginalised once each. A per-event application would have marginalised
    // each of them twice and left the lifetime ledger at four.
    assert_eq!(
        env.working.marginalisation_errors.events, 2,
        "one joint marginalisation across the two full-dimension models",
    );
    assert_eq!(first.events, env.working.marginalisation_errors.events);
}

/// The standardisation vectors shrink when a Sentinel's features are
/// marginalised away, not merely the model itself. Compaction is what keeps
/// every surviving feature's statistics at the index the model now expects to
/// find it at.
///
/// ´claim:lifespan:standardisation-is-compacted-in-step-with-marginalisation´
/// ´test:crate:deregister-sentinel-compacts-standardisation´
#[test]
fn deregister_sentinel_compacts_standardisation() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    let len_after_register = env.working.feature_means.len();

    // Register the sentinel in the dimension map so deregister can find it.
    let dim_after = env.working.operational.dim();
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, dim_after));

    env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 2);

    assert!(
        env.working.feature_means.len() < len_after_register,
        "standardisation should shrink after deregistration"
    );
}

/// Deregistration removes the Sentinel's ledger partition along with its model
/// features. Sentinel ids are reusable, so a partition left standing would hand
/// some future Sentinel the accumulated outcomes of its predecessor.
///
/// ´claim:lifespan:a-retired-sentinels-ledger-partition-does-not-outlive-it´
/// ´test:crate:deregister-sentinel-removes-outcome-ledger-partition´
#[test]
fn deregister_sentinel_removes_outcome_ledger_partition() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    assert!(
        env.outcome_ledger.contains(SentinelId(1)),
        "setup should create ledger partition"
    );

    let dim_after_register = env.working.operational.dim();
    env.working.dimension_map.sentinel_slots.insert(
        SentinelId(1),
        crate::feature::dimension_map::IndexRange::new(16, dim_after_register),
    );

    env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 2);

    assert!(
        !env.outcome_ledger.contains(SentinelId(1)),
        "deregistered Sentinel should not retain a ledger partition"
    );
}

/// A deregistration event for a Sentinel with no slot in the dimension map
/// reports success and leaves the dimension untouched. The model owner
/// processes events the public surface has already validated, so failing here
/// would abort a whole compound batch over an entity that is already gone.
///
/// ´claim:lifespan:retiring-a-sentinel-the-dimension-map-never-held-changes-nothing-and-is-not-a-failure´
/// ´test:crate:deregister-unknown-sentinel-is-noop´
#[test]
fn deregister_unknown_sentinel_is_noop() {
    let mut env = LifecycleEnv::new();
    let dim_before = env.working.operational.dim();

    let results = env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(999))], 1);

    assert_eq!(results[0].outcome, EventOutcome::Success);
    assert_eq!(
        env.working.operational.dim(),
        dim_before,
        "model dimension should not change for unknown sentinel"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Axis Lifecycle Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Registering a spatially-enabled outcome axis grows the model by two features
/// in every registered Sentinel's slot — a compressed and a raw reading of the
/// axis apiece. The axis is meant to be read spatially, which means each
/// Sentinel's own view of it has to exist.
///
/// ´claim:lifespan:a-spatial-axis-costs-two-features-in-every-sentinel-slot´
/// ´test:crate:register-axis-extends-per-sentinel´
#[test]
fn register_axis_extends_per_sentinel() {
    let mut env = LifecycleEnv::new();

    // Register 2 sentinels first
    env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
        ],
        1,
    );

    // Add slots to dimension map so they count.
    // Cold-start p=16, each sentinel adds 61, so: S1=[16..77), S2=[77..138).
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, 77));
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(2), crate::feature::dimension_map::IndexRange::new(77, 138));

    let dim_before = env.working.operational.dim();

    // Must use spatial_features: Enabled to trigger per-Sentinel extension
    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        2,
    );

    // 2 sentinels × 2 features per axis = 4 new features
    let expected_increase = 2 * 2;
    assert_eq!(
        env.working.operational.dim() - dim_before,
        expected_increase,
        "model should grow by 2 × n_sentinels for spatial axis"
    );
}

/// With no Sentinels registered, adding an outcome axis leaves the dimension
/// exactly where it was. Per-Sentinel features are per Sentinel: zero Sentinels
/// buys zero of them, and the axis waits to be paid for until there is
/// something to pay for.
///
/// ´claim:lifespan:an-axis-registered-before-any-sentinel-exists-costs-no-dimension´
/// ´test:crate:register-axis-zero-sentinels-noop´
#[test]
fn register_axis_zero_sentinels_noop() {
    let mut env = LifecycleEnv::new();
    let dim_before = env.working.operational.dim();

    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 1);

    assert_eq!(
        env.working.operational.dim(),
        dim_before,
        "with no sentinels, axis registration should not change dimension"
    );
}

/// An axis registered with spatial features disabled adds nothing to any
/// Sentinel slot, yet its own outcome model is still created and present in the
/// working copy. The spatial policy governs how the axis is measured, not
/// whether it is predicted.
///
/// ´claim:lifespan:a-non-spatial-axis-is-modelled-without-touching-any-sentinel-slot´
/// ´test:crate:register-axis-disabled-spatial-no-sentinel-growth´
#[test]
fn register_axis_disabled_spatial_no_sentinel_growth() {
    let mut env = LifecycleEnv::new();

    // Register 2 sentinels
    env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
        ],
        1,
    );

    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, 77));
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(2), crate::feature::dimension_map::IndexRange::new(77, 138));

    let dim_before = env.working.operational.dim();

    // Register axis with spatial_features: Disabled
    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 2);

    // No per-Sentinel features should be added
    assert_eq!(
        env.working.operational.dim(),
        dim_before,
        "Disabled spatial policy should not add per-Sentinel features"
    );
    // But the outcome model should exist in the snapshot
    assert!(
        env.working.outcome_models.contains_key(&OutcomeAxisId(1)),
        "outcome model should be created regardless of spatial policy"
    );
}

/// Even a non-spatial axis grows every live identity dimension's block by three
/// features, recording its offset within that block. Identity is where an axis
/// is read per entity rather than per Sentinel, so that reading exists
/// regardless of whether the Sentinels also carry the axis.
///
/// ´claim:lifespan:an-axis-adds-three-features-to-each-identity-dimension-whatever-its-spatial-policy´
/// ´test:crate:register-axis-with-identity-adds-per-dimension-features´
#[test]
fn register_axis_with_identity_adds_per_dimension_features() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    let dim_before = env.working.operational.dim();

    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 2);

    assert_eq!(
        env.working.operational.dim() - dim_before,
        3,
        "disabled axis with one identity dimension should add 3 identity-axis features"
    );
    assert!(env.working.dimension_map.id_axis_offsets.contains_key(&OutcomeAxisId(1)));
}

/// Axis registration extends standardisation for every registered identity
/// dimension: with two dimensions live, a new axis gives each dimension the
/// compressed–raw–stability class triple, each position carrying its class
/// prior, and the vectors end exactly as long as the models the same event
/// extended. The reference configuration registers two identity dimensions,
/// so a push that stopped after one triple would leave the reference
/// deployment's second dimension standardising its axis features against the
/// neutral resize fallback instead of the classes the corpus assigns
/// (´req:standardisation:class-assignment´). The triples are read at the
/// map's per-axis offset inside each dimension's own block rather than at the
/// tail of the vector, because the rearrangement the rebuild now performs has
/// already carried them from where the extension appended them to where the
/// canonical layout puts them (´entry:assayer:wl-owner-layout-misalignment´);
/// reading the tail would pin the misalignment as the design.
///
/// ´claim:lifespan:axis-registration-extends-standardisation-for-every-identity-dimension´
/// ´test:crate:register-axis-with-two-identity-dimensions-standardises-both´
#[test]
fn register_axis_with_two_identity_dimensions_standardises_both() {
    use crate::feature::standardisation::FeatureClass;

    let mut env = LifecycleEnv::new();
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(2) }], 2);

    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 3);

    assert_eq!(env.working.feature_means.len(), env.working.operational.dim());
    let triple = [
        FeatureClass::OutcomeAxisCompressed,
        FeatureClass::OutcomeAxisRaw,
        FeatureClass::OutcomeAxisStability,
    ];
    let offset = env
        .working
        .dimension_map
        .identity_axis_offset(OutcomeAxisId(1))
        .expect("the registered axis has a per-axis offset");
    for d in [DimensionId(1), DimensionId(2)] {
        let range = &env.working.dimension_map.id_dim_ranges[&d];
        for (k, expected) in triple.iter().enumerate() {
            let j = range.start + offset + k;
            assert_eq!(
                env.working.feature_classes[j], *expected,
                "class at the axis triple's own position {j} (identity dimension {d:?})"
            );
            let (mu, var) = expected.prior();
            assert!(
                (env.working.feature_means[j] - mu).abs() < 1e-12,
                "mean at {j} must be the class prior {mu}, got {}",
                env.working.feature_means[j]
            );
            assert!(
                (env.working.feature_variances[j] - var).abs() < 1e-12,
                "variance at {j} must be the class prior {var}, got {}",
                env.working.feature_variances[j]
            );
        }
    }
}

/// A spatial axis registered while both Sentinels and an identity dimension are
/// live pays for both shapes at once: two features in each Sentinel slot plus
/// three in the identity block. The per-Sentinel and per-identity views answer
/// different questions about the same axis, so neither substitutes for the
/// other.
///
/// ´claim:lifespan:the-two-extension-shapes-of-an-axis-add-rather-than-replacing-one-another´
/// ´test:crate:register-spatial-axis-with-identity-adds-identity-and-sentinel-features´
#[test]
fn register_spatial_axis_with_identity_adds_identity_and_sentinel_features() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
        ],
        2,
    );
    let dim_before = env.working.operational.dim();

    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        3,
    );

    assert_eq!(
        env.working.operational.dim() - dim_before,
        2 * 2 + 3,
        "spatial axis with one identity dimension should add 2n + 3 features"
    );
    assert!(env.working.dimension_map.id_axis_offsets.contains_key(&OutcomeAxisId(1)));
}

/// Retiring a spatial axis shrinks the model again, removing the per-Sentinel
/// features its registration added. Without that, a deployment experimenting
/// with axes would ratchet upward in dimension for the lifetime of the process.
///
/// ´claim:lifespan:retiring-an-axis-gives-back-the-features-it-took-from-the-sentinel-slots´
/// ´test:crate:deregister-axis-marginalises´
#[test]
fn deregister_axis_marginalises() {
    let mut env = LifecycleEnv::new();

    // Register 1 sentinel
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    // Register sentinel slot in dimension map so deregister_axis can find it.
    let dim_after = env.working.operational.dim();
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, dim_after));

    // Register spatial axis so per-Sentinel features are added
    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        2,
    );
    let dim_after_axis = env.working.operational.dim();

    // Deregister axis
    env.submit(vec![LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(1))], 3);

    assert!(
        env.working.operational.dim() < dim_after_axis,
        "model should shrink after axis deregistration"
    );
}

/// The per-axis readings an axis accumulated are pruned from every ledger entry
/// — the root and its children alike — when that axis is retired. Axis ids can
/// be registered again later, and a surviving row would let the new axis read
/// its predecessor's history as its own warm start.
///
/// ´claim:lifespan:retiring-an-axis-prunes-its-per-axis-rows-from-every-ledger-entry´
/// ´test:crate:deregister-axis-prunes-ledger-per-axis-rows´
#[test]
fn deregister_axis_prunes_ledger_per_axis_rows() {
    // Step 4 of the deregistration protocol drops the per-cell running averages
    // rather than keeping them (´alg:registry:axis-deregistration´):
    // deregistering a spatial axis must remove its per-axis EWMA rows from
    // every Ledger entry so a subsequent re-registration starts from zero.
    use crate::ledger::entry::LedgerEntry;
    use crate::types::LedgerKey;

    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    env.outcome_ledger.create_sentinel(SentinelId(1));
    let dim_after = env.working.operational.dim();
    env.working
        .dimension_map
        .sentinel_slots
        .insert(SentinelId(1), crate::feature::dimension_map::IndexRange::new(16, dim_after));

    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        2,
    );

    // Seed per-axis rows on root and a child entry, emulating label-time lazy
    // append (´dec:memory:lazy-axis-rows´).
    {
        let lock = env.outcome_ledger.get(SentinelId(1)).expect("sentinel ledger");
        let mut guard = lock.write().unwrap();
        guard.root_mut().per_axis.push((OutcomeAxisId(1), 0.42, 0.13));

        let mut child = LedgerEntry::new_neutral();
        child.per_axis.push((OutcomeAxisId(1), 0.9, 0.7));
        guard.insert(LedgerKey::new(0, 8), child);
    }

    env.submit(vec![LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(1))], 3);

    let lock = env.outcome_ledger.get(SentinelId(1)).expect("sentinel ledger");
    let guard = lock.read().unwrap();
    for (key, entry) in guard.entries() {
        assert!(
            !entry.per_axis.iter().any(|(id, _, _)| *id == OutcomeAxisId(1)),
            "per-axis row must be pruned after deregister at {key:?}"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension Lifecycle Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// Driven through the full submission path rather than the handler alone, an
/// identity-dimension registration both widens the model and leaves a
/// convergence tracker behind for the new dimension. The tracker is created in
/// the same step as the features it will watch, so there is never a published
/// dimension that nothing is tracking.
///
/// (´claim:lifespan:the-first-identity-dimension-brings-its-own-block-and-the-cross-dimension-aggregate-block-with-it´)
/// ´test:crate:register-identity-extends-and-creates-tracker´
#[test]
fn register_identity_extends_and_creates_tracker() {
    let mut env = LifecycleEnv::new();
    let dim_before = env.working.operational.dim();

    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);

    assert!(
        env.working.operational.dim() > dim_before,
        "model should grow for identity dimension"
    );
    assert!(
        env.identity_trackers.contains_key(&DimensionId(1)),
        "identity tracker should be created"
    );
}

/// Retiring an identity dimension through the submission path removes its
/// tracker and shrinks the model back down in the same step. Tracker and
/// features are two views of one registration, and they end together.
///
/// (´claim:lifespan:deregistering-an-identity-dimension-takes-its-convergence-tracker-with-it´)
/// ´test:crate:deregister-identity-removes-tracker-and-marginalises´
#[test]
fn deregister_identity_removes_tracker_and_marginalises() {
    let mut env = LifecycleEnv::new();

    // Register
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    let dim_after_register = env.working.operational.dim();

    // Deregister
    env.submit(vec![LifecycleEvent::DeregisterIdentityDimension(DimensionId(1))], 2);

    assert!(
        !env.identity_trackers.contains_key(&DimensionId(1)),
        "identity tracker should be removed"
    );
    assert!(
        env.working.operational.dim() < dim_after_register,
        "model should shrink after identity deregistration"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Cell Lifecycle Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A cell entering the competitive set of a genuinely registered identity
/// dimension widens the model. Here the dimension is established by its own
/// lifecycle event rather than injected by hand, so the entry handler's
/// registration guard is satisfied the way production satisfies it.
///
/// (´claim:lifespan:a-competitive-cell-entering-adds-its-indicator-and-nothing-more-when-no-interaction-templates-exist´)
/// ´test:crate:cell-entry-extends-model´
#[test]
fn cell_entry_extends_model() {
    let mut env = LifecycleEnv::new();

    // A competitive cell requires its identity dimension to exist.
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    let dim_before = env.working.operational.dim();

    let cell = CompetitiveCellId::new(0, 8);
    env.submit(
        vec![LifecycleEvent::CompetitiveCellEntry {
            dimension: DimensionId(1),
            cell,
        }],
        2,
    );

    assert!(env.working.operational.dim() > dim_before, "model should grow on cell entry");
}

/// When a cell exits the competitive set the model shrinks again, its indicator
/// marginalised away with no manual bookkeeping — the rebuild that followed the
/// entry had already recorded the cell, so the exit finds it. Competitive
/// membership turns over constantly, so entry and exit have to be exact
/// inverses or the dimension would climb with every reshuffle.
///
/// ´claim:lifespan:a-cell-leaving-the-competitive-set-gives-its-indicator-back´
/// ´test:crate:cell-exit-marginalises-model´
#[test]
fn cell_exit_marginalises_model() {
    let mut env = LifecycleEnv::new();

    // A competitive cell requires its identity dimension to exist.
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);

    // Enter a cell
    let cell = CompetitiveCellId::new(0, 8);
    env.submit(
        vec![LifecycleEvent::CompetitiveCellEntry {
            dimension: DimensionId(1),
            cell,
        }],
        2,
    );
    let dim_after_entry = env.working.operational.dim();

    // The DimensionMap rebuild after entry already tracks the cell in
    // competitive_indices, so no manual insertion is needed.

    // Exit the cell
    env.submit(
        vec![LifecycleEvent::CompetitiveCellExit {
            dimension: DimensionId(1),
            cell,
        }],
        3,
    );

    assert!(
        env.working.operational.dim() < dim_after_entry,
        "model should shrink on cell exit"
    );
}

/// The competitive-set changes a submission carries land on the dimension's
/// convergence tracker: three cell entries record three entries against the
/// rebuilt current set, and a following mixed submission adds its exit and
/// entry to the tallies while the tracked set follows the map. The measurement
/// machinery was complete and fed by nothing; this is the feeding, and it is
/// what stops the composite from reading steady state over an identity layer
/// it has never measured.
///
/// ´claim:lifespan:a-submissions-cell-entries-and-exits-are-recorded-on-that-dimensions-convergence-tracker´
/// ´test:crate:cell-lifecycle-events-feed-identity-tracker´
#[test]
fn cell_lifecycle_events_feed_identity_tracker() {
    let mut env = LifecycleEnv::new();
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);

    let tracker = env.identity_trackers.get(&DimensionId(1)).expect("tracker created");
    assert_eq!(tracker.total_entries, 0);

    let cells: Vec<CompetitiveCellId> = (0..3).map(|i| CompetitiveCellId::new(i * 100, 8)).collect();
    env.submit(
        cells
            .iter()
            .map(|&cell| LifecycleEvent::CompetitiveCellEntry {
                dimension: DimensionId(1),
                cell,
            })
            .collect(),
        2,
    );

    let tracker = env.identity_trackers.get(&DimensionId(1)).expect("tracker present");
    assert_eq!(tracker.total_entries, 3, "three entries recorded from one submission");
    assert_eq!(tracker.total_exits, 0);
    assert_eq!(tracker.current_cell_count, 3, "current set read from the rebuilt map");

    // One exit and one fresh entry in a single mixed submission.
    let newcomer = CompetitiveCellId::new(900, 8);
    env.submit(
        vec![
            LifecycleEvent::CompetitiveCellExit {
                dimension: DimensionId(1),
                cell: cells[0],
            },
            LifecycleEvent::CompetitiveCellEntry {
                dimension: DimensionId(1),
                cell: newcomer,
            },
        ],
        3,
    );

    let tracker = env.identity_trackers.get(&DimensionId(1)).expect("tracker present");
    assert_eq!(tracker.total_entries, 4);
    assert_eq!(tracker.total_exits, 1);
    assert_eq!(tracker.current_cell_count, 3, "set size follows the map through the turnover");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Compound & Invariant Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// A submission carrying several unrelated events — two Sentinel registrations
/// and an identity dimension — yields one result per event, each reporting its
/// own outcome. However the events are grouped for application, the answer to
/// each of them is still its own: grouping is about the work, not about the
/// reporting.
///
/// ´claim:lifespan:every-event-in-a-submission-is-processed-and-reported-on-individually´
/// ´test:crate:compound-batch-processes-all-events´
#[test]
fn compound_batch_processes_all_events() {
    let mut env = LifecycleEnv::new();

    let results = env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
            LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) },
        ],
        1,
    );

    assert_eq!(results.len(), 3, "all 3 events should produce results");
    for result in &results {
        assert_eq!(result.outcome, EventOutcome::Success);
    }
}

/// Processing a submission publishes a new snapshot carrying exactly the
/// version the caller supplied, advancing past the version that stood before.
/// Readers take the published snapshot rather than the working copy, so a
/// structural change that was not published would be invisible however
/// thoroughly it was applied.
///
/// ´claim:lifespan:a-processed-submission-publishes-a-snapshot-stamped-with-the-version-it-was-given´
/// ´test:crate:lifecycle-publishes-snapshot´
#[test]
fn lifecycle_publishes_snapshot() {
    let mut env = LifecycleEnv::new();
    let snap_before = env.shared.published.load();
    let version_before = snap_before.version;

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 5);

    let snap_after = env.shared.published.load();
    assert_eq!(snap_after.version, 5, "snapshot should be published with the given version");
    assert!(snap_after.version > version_before);
}

/// A submission carrying a completion channel receives its result on that
/// channel once processing finishes. This is what lets a caller that must not
/// proceed until the model has stopped referencing something — identity-
/// dimension teardown above all — actually wait for it.
///
/// ´claim:lifespan:a-submission-that-asked-to-be-waited-on-signals-its-completion-channel´
/// ´test:crate:lifecycle-signals-completion´
#[test]
fn lifecycle_signals_completion() {
    let mut env = LifecycleEnv::new();
    let (tx, rx) = crossbeam_channel::bounded(1);

    let submission = LifecycleSubmission {
        events: vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))],
        completion: Some(tx),
    };

    let _result = handle_lifecycle_submission(
        &mut env.working,
        submission,
        &env.shared,
        &mut env.platt_tracker,
        &mut env.identity_trackers,
        &env.outcome_ledger,
        &env.config,
        &mut 1,
        std::time::Instant::now(),
        &crate::types::PersistentTimestamp::new(0, 0),
    );

    let completion = rx.recv_timeout(ACK_DEADLINE);
    assert!(completion.is_ok(), "completion channel should receive result");
}

/// Two extensions of different shapes applied in one submission — a Sentinel
/// slot and an identity block — still leave the three standardisation vectors
/// at a common length. The invariant holds across mixed batches, not only after
/// a single well-understood event.
///
/// (´claim:lifespan:standardisation-grows-with-the-model-and-its-three-vectors-stay-the-same-length´)
/// ´test:crate:standardisation-consistent-after-extend´
#[test]
fn standardisation_consistent_after_extend() {
    let mut env = LifecycleEnv::new();

    env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) },
        ],
        1,
    );

    assert_eq!(
        env.working.feature_means.len(),
        env.working.feature_variances.len(),
        "means and variances must agree"
    );
    assert_eq!(
        env.working.feature_means.len(),
        env.working.feature_classes.len(),
        "means and classes must agree"
    );
}

/// Through Sentinel registrations and an identity dimension alike, the anchor
/// model's dimension never moves. The anchor is the fixed reference the blend
/// falls back on, and a reference that changed shape whenever the structure did
/// would be no anchor.
///
/// ´claim:lifespan:the-anchor-model-is-never-resized-by-a-lifecycle-event´
/// ´test:crate:anchor-dimension-unchanged´
#[test]
fn anchor_dimension_unchanged() {
    let mut env = LifecycleEnv::new();
    let anchor_dim_before = env.working.anchor.dim();

    env.submit(
        vec![
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1))),
            LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(2))),
            LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) },
        ],
        1,
    );

    assert_eq!(
        env.working.anchor.dim(),
        anchor_dim_before,
        "anchor model dimension must never change; was {}, now {}",
        anchor_dim_before,
        env.working.anchor.dim(),
    );
}

// ─── Axis-model dimension invariant (´inv:guarantee:axis-lifecycle´) ─────────
//
// The invariant: "every outcome axis prediction model undergoes exact
// extension or marginalisation on each lifecycle event." These tests
// regression-guard that axis models stay dimension-consistent with
// operational/sister across subsequent lifecycle events.

/// An axis model created at one dimension is itself extended when a Sentinel
/// registers afterwards, ending at the operational model's new width with the
/// rebuilt dimension map agreeing. Axis models are scored against the same
/// feature vector as the operational model, so any of them lagging behind would
/// be reading the wrong columns.
///
/// ´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´
/// ´test:crate:axis-model-extended-on-subsequent-sentinel-registration´
#[test]
fn axis_model_extended_on_subsequent_sentinel_registration() {
    let mut env = LifecycleEnv::new();

    // 1. Register axis A at baseline dimension.
    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 1);
    let p_after_axis = env.working.operational.dim();
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_axis,
        "axis model must start at operational dim"
    );

    // 2. Register a Sentinel — axis model must grow with operational.
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 2);
    let p_after_sentinel = env.working.operational.dim();
    assert!(p_after_sentinel > p_after_axis, "sentinel registration must extend p");
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_sentinel,
        "axis model must track operational across sentinel registration"
    );
    assert_eq!(
        env.working.dimension_map.p, p_after_sentinel,
        "DimensionMap.p must match after rebuild"
    );
}

/// The tracking runs downward too: retiring a Sentinel shrinks the axis model
/// along with operational and sister, all three landing on the same reduced
/// width. Extension and marginalisation are one obligation seen from either
/// side.
///
/// (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´)
/// ´test:crate:axis-model-marginalised-on-subsequent-sentinel-deregistration´
#[test]
fn axis_model_marginalised_on_subsequent_sentinel_deregistration() {
    let mut env = LifecycleEnv::new();

    // Register a Sentinel first so the deregister path has something to remove.
    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    // Register axis A (now at larger dim).
    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 2);
    let p_before_dereg = env.working.operational.dim();
    assert_eq!(env.working.outcome_models[&OutcomeAxisId(1)].model.dim(), p_before_dereg);

    // Deregister the Sentinel — axis model must shrink with operational.
    env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(1))], 3);
    let p_after_dereg = env.working.operational.dim();
    assert!(p_after_dereg < p_before_dereg, "sentinel deregister must reduce p");
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_dereg,
        "axis model must track operational across sentinel deregistration"
    );
    assert_eq!(env.working.sister.dim(), p_after_dereg, "sister must track operational");
}

/// An identity-dimension registration widens every existing axis model as well.
/// The obligation attaches to the dimension changing, not to which kind of
/// entity changed it.
///
/// (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´)
/// ´test:crate:axis-model-extended-on-subsequent-identity-dimension-registration´
#[test]
fn axis_model_extended_on_subsequent_identity_dimension_registration() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 1);
    let p_after_axis = env.working.operational.dim();

    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 2);
    let p_after_identity = env.working.operational.dim();
    assert!(p_after_identity > p_after_axis);
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_identity,
        "axis model must extend when an identity dimension is registered"
    );
}

/// A competitive cell entering widens the axis models too, small as that change
/// is. One feature is enough to desynchronise a model, so the cheapest
/// lifecycle event is held to the same rule as the most expensive.
///
/// (´claim:lifespan:every-outcome-axis-model-tracks-the-operational-dimension-through-later-lifecycle-events´)
/// ´test:crate:axis-model-extended-on-subsequent-cell-entry´
#[test]
fn axis_model_extended_on_subsequent_cell_entry() {
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 1);
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 2);
    let p_before_cell = env.working.operational.dim();

    env.submit(
        vec![LifecycleEvent::CompetitiveCellEntry {
            dimension: DimensionId(1),
            cell: CompetitiveCellId::new(0, 8),
        }],
        3,
    );
    let p_after_cell = env.working.operational.dim();
    assert!(p_after_cell > p_before_cell, "cell entry must extend p");
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_cell,
        "axis model must extend on competitive cell entry"
    );
}

/// Registering a spatial axis while another axis already exists extends the
/// older axis's model too, and creates the new one directly at the post-
/// extension dimension rather than extending it afterwards. The new axis never
/// existed at the narrower width, so it has nothing to be extended from.
///
/// ´claim:lifespan:a-new-axis-both-widens-the-axes-that-preceded-it-and-is-born-at-the-width-it-created´
/// ´test:crate:all-axis-models-extended-when-new-axis-registered´
#[test]
fn all_axis_models_extended_when_new_axis_registered() {
    // Registering axis B while axis A exists must extend axis A's model too
    // (spatial axis B grows every existing model by 2 × n_sentinels).
    let mut env = LifecycleEnv::new();

    env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(1)))], 1);
    env.submit(vec![LifecycleEvent::RegisterOutcomeAxis(axis_reg(OutcomeAxisId(1)))], 2);
    let p_before_axis_b = env.working.operational.dim();
    assert_eq!(env.working.outcome_models[&OutcomeAxisId(1)].model.dim(), p_before_axis_b);

    // Register a spatial axis B: this should extend operational + sister +
    // axis A's model by 2 × n_sentinels.
    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(2)))],
        3,
    );
    let p_after_axis_b = env.working.operational.dim();
    assert!(p_after_axis_b > p_before_axis_b, "spatial axis must extend p");
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(1)].model.dim(),
        p_after_axis_b,
        "existing axis A must be extended when a new axis B is registered"
    );
    assert_eq!(
        env.working.outcome_models[&OutcomeAxisId(2)].model.dim(),
        p_after_axis_b,
        "new axis B must be created at post-extension dim"
    );
}

/// Registering an identity dimension and then retiring it leaves the model at
/// exactly its original dimension — the per-dimension block and the cross-
/// dimension aggregate both go back. Structural change is meant to be
/// reversible in size as well as in effect, or a long-lived deployment slowly
/// pays for everything it ever tried.
///
/// ´claim:lifespan:a-registration-and-its-matching-retirement-return-the-dimension-to-where-they-found-it´
/// ´test:crate:register-then-deregister-restores-dimension´
#[test]
fn register_then_deregister_restores_dimension() {
    let mut env = LifecycleEnv::new();
    let dim_before = env.working.operational.dim();

    // Register identity dimension (adds per-dimension + cross-dimension features)
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);
    assert!(env.working.operational.dim() > dim_before);

    // Deregister the same dimension (removes per-dimension + cross-dimension features)
    env.submit(vec![LifecycleEvent::DeregisterIdentityDimension(DimensionId(1))], 2);
    assert_eq!(
        env.working.operational.dim(),
        dim_before,
        "register then deregister should restore original dimension"
    );
}

/// Two identity dimensions registered in one submission pay for the
/// cross-dimension aggregate block once between them, and retired in one
/// submission they take it with them, leaving the model exactly where it
/// started. Neither pair shares a chain: a registration is order-free with
/// nothing, and two identity-dimension removals are the removal family's one
/// exception, so the second operation of each pair reads a map the first has
/// already changed.
///
/// The block's arrival and its departure are both conditioned on a count of the
/// dimensions standing — the first dimension pays for it, the last takes it —
/// and a count is exactly what one shared entry layout cannot tell two
/// operations apart by. Planned against a single layout the pair would have
/// bought the block twice and then abandoned it, and the rebuild's width check
/// is what would have noticed. That is why the predicate excludes the pairing
/// rather than assuming a removal's positions are always a function of its own
/// entity.
///
/// ´claim:lifespan:two-identity-dimensions-in-one-submission-buy-one-cross-dimension-aggregate-block-and-return-it´
/// ´test:crate:two-identity-dimensions-in-one-submission-share-one-aggregate-block´
#[test]
fn two_identity_dimensions_in_one_submission_share_one_aggregate_block() {
    use crate::feature::dimension_map::{ID_CROSS_DIM_FEATURE_COUNT, ID_DIM_BASE_FEATURE_COUNT};

    let mut env = LifecycleEnv::new();
    let dim_before = env.working.operational.dim();

    let (_registered, last_version) = env.submit_reporting_version(
        vec![
            LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) },
            LifecycleEvent::RegisterIdentityDimension { id: DimensionId(2) },
        ],
        1,
    );
    assert_eq!(
        last_version, 2,
        "a registration is order-free with nothing, so two of them are two chains",
    );
    assert_eq!(
        env.working.operational.dim(),
        dim_before + 2 * ID_DIM_BASE_FEATURE_COUNT + ID_CROSS_DIM_FEATURE_COUNT,
        "two per-dimension blocks and one aggregate block between them",
    );

    let (_retired, last_version) = env.submit_reporting_version(
        vec![
            LifecycleEvent::DeregisterIdentityDimension(DimensionId(1)),
            LifecycleEvent::DeregisterIdentityDimension(DimensionId(2)),
        ],
        3,
    );
    assert_eq!(last_version, 4, "two identity-dimension removals never share a chain");
    assert_eq!(
        env.working.operational.dim(),
        dim_before,
        "the aggregate block leaves with the last dimension standing",
    );
}

/// Two cell exits and one entry submitted together net to a single feature
/// removed, and the batch publishes once per order-free chain rather than once
/// per event: the two exits are order-free with each other and apply as one
/// plan, the entry is order-free with nothing and applies as its own, so two
/// snapshots go out at consecutive versions. Amortising the rebuild across
/// events that commute is what makes batching worth doing under churn; doing it
/// across events that do not would be batching an ordering away.
///
/// ´claim:lifespan:a-compound-batch-nets-its-events-and-publishes-once-per-order-free-chain´
/// ´test:crate:compound-batch-2-exits-1-entry´
#[test]
fn compound_batch_2_exits_1_entry() {
    // Plan acceptance criterion: 2 exits + 1 entry → one joint marginalisation
    // for the exits, one publish per chain.
    let mut env = LifecycleEnv::new();

    // A competitive cell requires its identity dimension to exist.
    env.submit(vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }], 1);

    // Enter 3 cells
    let cells: Vec<CompetitiveCellId> = (0..3).map(|i| CompetitiveCellId::new(i * 100, 8)).collect();
    for (i, &cell) in cells.iter().enumerate() {
        env.submit(
            vec![LifecycleEvent::CompetitiveCellEntry {
                dimension: DimensionId(1),
                cell,
            }],
            (i + 2) as u64,
        );
    }

    // After entry, each cell adds 1 feature (indicator only; no Type-5 without templates).
    // The DimensionMap rebuild tracks them in competitive_indices automatically.
    let dim_after_3_entries = env.working.operational.dim();

    let snap_before = env.shared.published.load();
    let version_before = snap_before.version;

    // Submit 2 exits + 1 entry in a single batch
    let new_cell = CompetitiveCellId::new(999, 8);
    let (results, last_version) = env.submit_reporting_version(
        vec![
            LifecycleEvent::CompetitiveCellExit {
                dimension: DimensionId(1),
                cell: cells[0],
            },
            LifecycleEvent::CompetitiveCellExit {
                dimension: DimensionId(1),
                cell: cells[1],
            },
            LifecycleEvent::CompetitiveCellEntry {
                dimension: DimensionId(1),
                cell: new_cell,
            },
        ],
        10,
    );

    // All 3 events should succeed
    assert_eq!(results.len(), 3);
    for r in &results {
        assert_eq!(r.outcome, EventOutcome::Success);
    }

    // Net effect: each exit removes 1 feature (indicator index tracked in
    // competitive_indices); each entry adds 1 (indicator only; no Type-5 without templates).
    // So: -1 -1 +1 = -1 net from the compound batch.
    assert_eq!(
        env.working.operational.dim(),
        dim_after_3_entries - 1,
        "2 exits (removing 1 each) + 1 entry (adding 1) should net to -1"
    );

    // Two chains, so two publishes: the exits at the version we passed, the
    // entry at the next one.
    assert_eq!(last_version, 11, "the exits publish at 10 and the entry, sealing them, at 11");
    let snap_after = env.shared.published.load();
    assert_eq!(
        snap_after.version, 11,
        "the last chain's publication is the standing snapshot"
    );
    assert!(snap_after.version > version_before);
}

/// With three Sentinels standing, a spatial axis adds six features rather than
/// a fixed block: the cost scales with the population the axis has to be read
/// across, and the standardisation vectors follow it exactly. This cascade is
/// what makes the final dimension a function of what is registered rather than
/// of the order it was registered in.
///
/// (´claim:lifespan:a-spatial-axis-costs-two-features-in-every-sentinel-slot´)
/// ´test:crate:cross-entity-cascade-r27´
#[test]
fn cross_entity_cascade_r27() {
    // R-27: register spatial axis with 3 Sentinels → p increases by
    // 3×2 (per-Sentinel axis features).
    let mut env = LifecycleEnv::new();

    // Register 3 Sentinels
    for i in 1..=3 {
        env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(i)))], i as u64);
    }

    // Register them in the dimension map so the axis registration can count them.
    // Cold-start p=16, each Sentinel adds 61 (1 occ + 60 q_base, no interactions).
    // S1=[16..77), S2=[77..138), S3=[138..199).
    for i in 0_u32..3 {
        let start = 16 + i as usize * 61;
        let end = start + 61;
        env.working
            .dimension_map
            .sentinel_slots
            .insert(SentinelId(i + 1), crate::feature::dimension_map::IndexRange::new(start, end));
    }

    let dim_before_axis = env.working.operational.dim();

    // Register a spatial axis (must use Enabled to trigger per-Sentinel extension)
    env.submit(
        vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(1)))],
        4,
    );

    // 3 Sentinels × 2 features per axis = 6 new features
    let dim_after_axis = env.working.operational.dim();
    let increase = dim_after_axis - dim_before_axis;
    assert_eq!(
        increase,
        3 * 2,
        "spatial axis registration with 3 Sentinels should add 3×2=6 features; got {increase}"
    );

    // Standardisation should be consistent
    assert_eq!(env.working.feature_means.len(), env.working.feature_variances.len());
    assert_eq!(env.working.feature_means.len(), env.working.feature_classes.len());
}

/// Built to twenty Sentinels and ten spatial axes, the dimension is exactly the
/// closed form the feature counts predict, with every Sentinel's slot range
/// disjoint from its neighbours and inside the model. Torn back down to one
/// Sentinel and no axes it returns to the minimum for that configuration, and
/// the lone surviving slot starts precisely where the signal block ends and
/// runs to the end — the index space is compacted rather than merely shortened,
/// so nothing addresses a hole left by a departed neighbour.
///
/// ´claim:lifespan:tearing-a-peak-configuration-back-down-compacts-the-index-space-with-no-stale-slot-surviving´
/// ´test:crate:peak-registration-then-full-deregistration-restores-min-dimension´
#[test]
fn peak_registration_then_full_deregistration_restores_min_dimension() {
    use crate::feature::dimension_map::SENTINEL_Q_BASE;

    const N_SENTINELS: u32 = 20;
    const N_SPATIAL_AXES: u32 = 10;

    let mut env = LifecycleEnv::new();
    let p_cold = env.working.operational.dim();

    // Peak registration: register 20 Sentinels, then 10 spatial axes.
    // One submission per event so the rebuild picks up each change
    // before the next handler reads `dimension_map.sentinel_slots.len()`.
    let mut version: u64 = 1;
    for i in 1..=N_SENTINELS {
        env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(i)))], version);
        version += 1;
    }

    // After 20 Sentinels, no axes yet: slot_width = 1 + q_base.
    let slot_width_no_axis = 1 + SENTINEL_Q_BASE;
    let p_after_sentinels = p_cold + (N_SENTINELS as usize) * slot_width_no_axis;
    assert_eq!(
        env.working.operational.dim(),
        p_after_sentinels,
        "p after 20 Sentinels should be cold-start + 20·slot_width"
    );
    assert_eq!(env.working.dimension_map.sentinel_slots.len(), N_SENTINELS as usize);

    // Register 10 spatial axes — each contributes 2 features per Sentinel.
    for i in 1..=N_SPATIAL_AXES {
        env.submit(
            vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(i)))],
            version,
        );
        version += 1;
    }

    let p_peak = p_after_sentinels + (N_SPATIAL_AXES as usize) * 2 * (N_SENTINELS as usize);
    assert_eq!(
        env.working.operational.dim(),
        p_peak,
        "p at peak should grow by 2·n_sentinels per spatial axis"
    );
    assert_eq!(
        env.working.dimension_map.p, p_peak,
        "DimensionMap.p must agree with model dim"
    );
    assert_eq!(
        env.working.outcome_models.len(),
        N_SPATIAL_AXES as usize,
        "10 outcome models at peak",
    );

    // Structural invariant: every Sentinel slot range lies within
    // `[0, p_peak)` and the 20 slot ranges are pair-wise disjoint.
    {
        let mut ranges: Vec<_> = env.working.dimension_map.sentinel_slots.values().cloned().collect();
        ranges.sort_by_key(|r| r.start);
        for w in ranges.windows(2) {
            assert!(w[0].end <= w[1].start, "slot ranges must be disjoint");
        }
        for r in &ranges {
            assert!(r.end <= p_peak, "no slot range may extend past p_peak");
        }
    }

    // Full deregistration: deregister all 10 axes, then 19 Sentinels.
    // Axes first so the subsequent Sentinel deregistrations happen
    // against a clean (no-spatial-axis) outcome-model set.
    for i in 1..=N_SPATIAL_AXES {
        env.submit(vec![LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(i))], version);
        version += 1;
    }

    assert!(
        env.working.outcome_models.is_empty(),
        "outcome models must be empty after deregistering all axes",
    );
    assert_eq!(
        env.working.operational.dim(),
        p_after_sentinels,
        "dim after axis teardown should match the 20-sentinel baseline",
    );

    for i in 2..=N_SENTINELS {
        env.submit(vec![LifecycleEvent::DeregisterSentinel(SentinelId(i))], version);
        version += 1;
    }

    // Post-deregistration invariants.
    let p_min = p_cold + slot_width_no_axis;
    assert_eq!(
        env.working.operational.dim(),
        p_min,
        "p should implode to cold-start + 1 Sentinel = {p_min}",
    );
    assert_eq!(
        env.working.dimension_map.p, p_min,
        "DimensionMap.p must agree after implosion"
    );
    assert_eq!(
        env.working.dimension_map.sentinel_slots.len(),
        1,
        "only the surviving Sentinel should remain in sentinel_slots",
    );
    assert!(
        env.working.dimension_map.sentinel_slots.contains_key(&SentinelId(1)),
        "SentinelId(1) must be the surviving slot",
    );

    // No stale indices: the lone surviving slot lies fully inside
    // `[sig_range.end, p_min)`, confirming the rebuild fully compacted
    // the index space.
    let surviving = env.working.dimension_map.sentinel_slots[&SentinelId(1)].clone();
    let sig_end = env.working.dimension_map.sig_range.end;
    assert_eq!(surviving.start, sig_end, "surviving slot must start at sig_range.end");
    assert_eq!(surviving.end, p_min, "surviving slot must cover all the way to p_min");

    // Standardisation vectors must agree with `p_min`.
    assert_eq!(env.working.feature_means.len(), p_min);
    assert_eq!(env.working.feature_variances.len(), p_min);
    assert_eq!(env.working.feature_classes.len(), p_min);
}

/// A hundred rounds of retiring a randomly chosen Sentinel and registering a
/// fresh one hold the operational model, the sister and the dimension map at
/// one common width, with the standardisation vectors matching and the pool
/// conserved at eight throughout. No slot range ever overlaps another or runs
/// past the model, and no entry of any model's mean or of the running
/// statistics ever stops being finite — repeated marginalisation and extension
/// neither leak indices nor accumulate numerical damage.
///
/// ´claim:lifespan:a-hundred-cycles-of-retire-and-replace-leave-the-model-consistent-finite-and-the-same-size´
/// ´test:crate:gauntlet-deregister-register-churn-100-cycles´
#[test]
fn gauntlet_deregister_register_churn_100_cycles() {
    use crate::testing::TestRng;

    const INITIAL_SENTINELS: u32 = 8;
    const CYCLES: usize = 100;

    let mut env = LifecycleEnv::new();
    let mut version: u64 = 1;

    // Pre-populate the pool of 8 Sentinels.
    for i in 1..=INITIAL_SENTINELS {
        env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(i)))], version);
        version += 1;
    }

    // Track which Sentinel ids are currently live.
    let mut live: Vec<SentinelId> = (1..=INITIAL_SENTINELS).map(SentinelId).collect();
    let mut next_id: u32 = INITIAL_SENTINELS + 1;

    let mut rng = TestRng::new(0xDEAD_BEEF_CAFE_F00D);

    // 100 random deregister/register cycles.
    for _cycle in 0..CYCLES {
        // Pick a random live Sentinel to retire.
        let idx = rng.next_range(live.len() as u64) as usize;
        let victim = live.swap_remove(idx);

        env.submit(vec![LifecycleEvent::DeregisterSentinel(victim)], version);
        version += 1;

        // Register a fresh replacement.
        let fresh = SentinelId(next_id);
        next_id += 1;
        env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(fresh))], version);
        version += 1;
        live.push(fresh);

        // ── Per-cycle invariants ────────────────────────────────────
        let p = env.working.dimension_map.p;

        // (1) Dimension consistency across all three dimension-carrying surfaces.
        assert_eq!(env.working.operational.dim(), p, "operational.dim must equal dimension_map.p");
        assert_eq!(env.working.sister.dim(), p, "sister.dim must equal dimension_map.p");

        // (2) Standardisation-vector lengths match p.
        assert_eq!(env.working.feature_means.len(), p, "feature_means len must equal p");
        assert_eq!(env.working.feature_variances.len(), p, "feature_variances len must equal p");
        assert_eq!(env.working.feature_classes.len(), p, "feature_classes len must equal p");

        // (3) Steady-state Sentinel count is conserved.
        assert_eq!(
            env.working.dimension_map.sentinel_slots.len(),
            INITIAL_SENTINELS as usize,
            "Sentinel count must stay at 8 across the churn",
        );

        // (4) Slot ranges are disjoint and bounded by p.
        let mut ranges: Vec<_> = env.working.dimension_map.sentinel_slots.values().cloned().collect();
        ranges.sort_by_key(|r| r.start);
        for w in ranges.windows(2) {
            assert!(w[0].end <= w[1].start, "slot ranges must be disjoint");
        }
        for r in &ranges {
            assert!(r.end <= p, "slot range must lie within [0, p)");
        }

        // (5) No NaN/Inf leaked into model means.
        for (i, x) in env.working.operational.mu().iter().enumerate() {
            assert!(x.is_finite(), "operational.mu[{i}] must be finite (got {x})");
        }
        for (i, x) in env.working.sister.mu().iter().enumerate() {
            assert!(x.is_finite(), "sister.mu[{i}] must be finite (got {x})");
        }
        for (i, x) in env.working.anchor.mu().iter().enumerate() {
            assert!(x.is_finite(), "anchor.mu[{i}] must be finite (got {x})");
        }

        // (6) Standardisation stats stay finite.
        for (i, &m) in env.working.feature_means.iter().enumerate() {
            assert!(m.is_finite(), "feature_means[{i}] must be finite (got {m})");
        }
        for (i, &v) in env.working.feature_variances.iter().enumerate() {
            assert!(
                v.is_finite() && v >= 0.0,
                "feature_variances[{i}] must be finite and non-negative (got {v})"
            );
        }
    }

    // Terminal liveness: the churn actually exercised the pool
    // (at least `CYCLES` distinct Sentinel ids were allocated).
    assert!(
        next_id >= INITIAL_SENTINELS + CYCLES as u32,
        "Sentinel pool must have rotated through at least {CYCLES} fresh ids"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// The rank-keyed outcome-memory pair across a spatial-axis retirement
// ═══════════════════════════════════════════════════════════════════════════════

/// The value planted at pre-event coordinate `idx` of the posterior mean.
///
/// Every coordinate gets its own value and the map back is exact, so a value
/// read after the event names the position it was learned at rather than merely
/// showing that something is there.
fn planted_mu(idx: usize) -> f64 {
    1_000.0 + idx as f64
}

/// The value planted at pre-event coordinate `idx` of the running means.
fn planted_mean(idx: usize) -> f64 {
    2_000.0 + idx as f64
}

/// The variance planted at pre-event coordinate `idx`. Kept positive, which is
/// what the standardisation divides by.
fn planted_variance(idx: usize) -> f64 {
    3_000.0 + idx as f64
}

/// The covariance planted on the diagonal at pre-event coordinate `idx`.
///
/// The planted covariance is diagonal, so the precision it inverts to is
/// diagonal too and every off-diagonal coupling between the retired block and
/// the kept block is exactly zero. The Schur correction is then identically
/// zero and the kept block a marginalisation leaves behind is the kept block it
/// was handed, position for position — which is what makes the reading below a
/// measurement of the rearrangement rather than of the linear algebra.
fn planted_covariance(idx: usize) -> f64 {
    (idx as f64).mul_add(1e-3, 1.0)
}

/// Recovers the pre-event coordinate a planted value was laid down at, or
/// `None` when the value is not one this fixture planted.
///
/// A value that does not land on a coordinate — because it was computed rather
/// than carried, or because it names a position outside the pre-event vector —
/// comes back as `None` rather than as a plausible index, so a failure message
/// says "no position this fixture planted" instead of naming an innocent one.
fn recover(value: f64, base: f64, scale: f64, p: usize) -> Option<usize> {
    let scaled = (value - base) / scale;
    let rounded = scaled.round();
    if (scaled - rounded).abs() > 1e-6 {
        return None;
    }
    let idx = usize::try_from(rounded as i64).ok()?;
    (idx < p).then_some(idx)
}

/// Plants a distinguishable value at every coordinate the working copy holds:
/// the posterior mean, the covariance diagonal, and the two standardisation
/// vectors.
fn plant_indexed_state(env: &mut LifecycleEnv) {
    use crate::model::bayesian::BayesianLinearModel;
    use crate::types::ModelId;

    let p = env.working.operational.dim();
    let mut params = env.working.operational.to_parameters();
    params.mu = (0..p).map(planted_mu).collect();
    params.covariance_data = vec![0.0; p * p];
    for idx in 0..p {
        params.covariance_data[idx * p + idx] = planted_covariance(idx);
    }
    env.working.operational = BayesianLinearModel::from_parameters(
        &params,
        &crate::model::parameters::FloorMass::default(),
        ModelConfig::default().lambda_prior,
        ModelId::Operational,
        1_000,
    )
    .expect("the planted covariance is diagonal and positive, so its inverse exists");

    for idx in 0..p {
        env.working.feature_means[idx] = planted_mean(idx);
        env.working.feature_variances[idx] = planted_variance(idx);
    }
}

/// Names, in the pre-event map's own terms, what sat at one of its positions.
///
/// A rank is not a name, so a pair's rank is resolved through the pre-event
/// layout order into the axis that owned it. That resolution is the whole
/// point: an assertion that reports "rank 1" cannot say whether the wrong
/// memory arrived, and one that reports "axis 8" can.
fn name_pre_event_position(map: &crate::feature::dimension_map::DimensionMap, idx: usize) -> String {
    use crate::feature::permutation::PositionKey;

    let keys = map.position_keys();
    match keys.get(idx) {
        Some(PositionKey::SentinelAxisPair(sentinel, rank, half)) => {
            let axis = map.slot_axis_offsets.keys().nth(*rank);
            format!("the outcome-memory pair of {axis:?} (spatial rank {rank}, half {half}) in {sentinel:?}'s slot")
        }
        Some(PositionKey::IdentityAxis(dim, axis, within)) => {
            format!("{dim:?}'s identity triple for {axis:?}, offset {within}")
        }
        Some(other) => format!("{other:?}"),
        None => format!("position {idx}, which the pre-event map does not name"),
    }
}

/// The state a fixture reads back after a retirement, with everything the
/// assertions need to name what they found.
struct RetirementReading {
    pre_map: crate::feature::dimension_map::DimensionMap,
    post_map: crate::feature::dimension_map::DimensionMap,
    mu: Vec<f64>,
    covariance_diagonal: Vec<f64>,
    precision_diagonal: Vec<f64>,
    feature_means: Vec<f64>,
    feature_variances: Vec<f64>,
    pre_p: usize,
    /// How many snapshots the submission published, which is how many order-free
    /// chains its events gathered into.
    chains_published: u64,
    /// What the real path's permutation did: applied, bypassed as the identity,
    /// or refused. Read from the pre-event map against the finally published
    /// one, so across a submission that broke its chain it reports the two
    /// layouts end to end rather than any single chain's rearrangement.
    permutation: String,
}

impl RetirementReading {
    /// Asserts that the four channels at `post_idx` all carry what was learned
    /// at `pre_idx`, and says whose memory arrived instead when they do not.
    fn assert_carries(&self, post_idx: usize, pre_idx: usize, what: &str) {
        let checks: [(&str, f64, f64, f64, f64); 4] = [
            ("posterior mean", self.mu[post_idx], 1_000.0, 1.0, planted_mu(pre_idx)),
            (
                "covariance diagonal",
                self.covariance_diagonal[post_idx],
                1.0,
                1e-3,
                planted_covariance(pre_idx),
            ),
            (
                "running mean",
                self.feature_means[post_idx],
                2_000.0,
                1.0,
                planted_mean(pre_idx),
            ),
            (
                "running variance",
                self.feature_variances[post_idx],
                3_000.0,
                1.0,
                planted_variance(pre_idx),
            ),
        ];
        for (channel, found, base, scale, expected) in checks {
            let arrived = recover(found, base, scale, self.pre_p);
            assert!(
                (found - expected).abs() <= 1e-6 * expected.abs().max(1.0),
                "{channel} at post-event position {post_idx} — {what} — should carry what was learned at \
                 pre-event position {pre_idx} ({}), but carries {found}, which is what was learned at {}. \
                 The permutation the real path computed was {}.",
                name_pre_event_position(&self.pre_map, pre_idx),
                arrived.map_or_else(
                    || "no position this fixture planted".to_owned(),
                    |i| format!("pre-event position {i} ({})", name_pre_event_position(&self.pre_map, i)),
                ),
                self.permutation,
            );
        }

        // The precision is the one channel the permutation touches as a matrix
        // rather than as a vector, and a row moved without its column would
        // show here and nowhere else.
        let expected_precision = 1.0 / planted_covariance(pre_idx);
        let found = self.precision_diagonal[post_idx];
        let arrived = recover(1.0 / found, 1.0, 1e-3, self.pre_p);
        assert!(
            (found - expected_precision).abs() <= 1e-6 * expected_precision,
            "precision diagonal at post-event position {post_idx} — {what} — should carry the precision \
             learned at pre-event position {pre_idx} ({}), but carries {found}, whose reciprocal is what was \
             planted at {}. The permutation the real path computed was {}.",
            name_pre_event_position(&self.pre_map, pre_idx),
            arrived.map_or_else(
                || "no position this fixture planted".to_owned(),
                |i| format!("pre-event position {i} ({})", name_pre_event_position(&self.pre_map, i)),
            ),
            self.permutation,
        );
    }
}

/// Builds a deployment with `sentinels` Sentinels, one identity dimension and
/// the given spatial axes, plants a distinguishable value at every coordinate,
/// then submits `retirements` and reads everything back off the published
/// snapshot.
fn retire_axes_after_planting(sentinels: &[u32], axes: &[u32], retirements: &[u32]) -> RetirementReading {
    apply_after_planting(
        sentinels,
        axes,
        retirements
            .iter()
            .map(|&a| LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(a)))
            .collect(),
    )
}

/// Builds the same deployment, plants the same distinguishable values, then
/// submits `events` as ONE submission and reads everything back off the last
/// snapshot it published.
///
/// The submission is what the reading is about: its events may gather into one
/// order-free chain or into several, and how many they gathered into is
/// reported alongside the values.
fn apply_after_planting(sentinels: &[u32], axes: &[u32], events: Vec<LifecycleEvent>) -> RetirementReading {
    use crate::feature::permutation::LayoutPermutation;

    let mut env = LifecycleEnv::new();
    let mut version = 1;
    for &s in sentinels {
        env.submit(vec![LifecycleEvent::RegisterSentinel(sentinel_reg(SentinelId(s)))], version);
        version += 1;
    }
    env.submit(
        vec![LifecycleEvent::RegisterIdentityDimension { id: DimensionId(1) }],
        version,
    );
    version += 1;
    for &a in axes {
        env.submit(
            vec![LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(a)))],
            version,
        );
        version += 1;
    }

    let pre_map = env.working.dimension_map.clone();
    let pre_p = env.working.operational.dim();
    plant_indexed_state(&mut env);

    let (_results, last_version) = env.submit_reporting_version(events, version);
    let chains_published = last_version - version + 1;

    let snapshot = env.shared.published.load();
    let post_map = snapshot.dimension_map.clone();
    let permutation = match LayoutPermutation::from_extension(&pre_map, &post_map) {
        None => "refused: the two layouts do not describe the same vector".to_owned(),
        Some(p) if p.is_identity() => "bypassed: it came out the identity, so the compaction alone placed the values".to_owned(),
        Some(_) => "applied: it was not the identity, so it moved values".to_owned(),
    };

    let post_p = snapshot.operational.p;
    let covariance_diagonal = (0..post_p)
        .map(|i| snapshot.operational.covariance_data[i * post_p + i])
        .collect();
    let precision_diagonal = (0..post_p)
        .map(|i| env.working.operational.precision().as_inner()[(i, i)])
        .collect();

    RetirementReading {
        pre_map,
        post_map,
        mu: snapshot.operational.mu.clone(),
        covariance_diagonal,
        precision_diagonal,
        feature_means: snapshot.feature_means.clone(),
        feature_variances: snapshot.feature_variances.clone(),
        pre_p,
        chains_published,
        permutation,
    }
}

/// Asserts that every surviving axis's outcome-memory pair, in every Sentinel
/// slot, still carries what that axis learned — and that its identity triple
/// does too.
fn assert_surviving_axes_keep_their_own_memory(reading: &RetirementReading, sentinels: &[u32], survivors: &[u32]) {
    for &axis in survivors {
        let axis_id = OutcomeAxisId(axis);
        let pre_offset = reading
            .pre_map
            .slot_axis_offset(axis_id)
            .expect("a surviving spatial axis owned a pair before the retirement");
        let post_offset = reading
            .post_map
            .slot_axis_offset(axis_id)
            .expect("a surviving spatial axis still owns a pair after the retirement");

        for &s in sentinels {
            let sentinel_id = SentinelId(s);
            let pre_slot = reading.pre_map.sentinel_slots[&sentinel_id].start;
            let post_slot = reading.post_map.sentinel_slots[&sentinel_id].start;
            for half in 0..2 {
                reading.assert_carries(
                    post_slot + post_offset + half,
                    pre_slot + pre_offset + half,
                    &format!("axis {axis}'s outcome-memory pair in Sentinel {s}'s slot, half {half}"),
                );
            }
        }

        // The control: the identity block's per-axis triple is keyed by the
        // axis itself rather than by a rank, so it is expected to survive the
        // renumbering, and a fixture that could not show it surviving would not
        // be measuring the keying at all.
        let pre_id_offset = reading
            .pre_map
            .identity_axis_offset(axis_id)
            .expect("a surviving axis owned an identity triple before the retirement");
        let post_id_offset = reading
            .post_map
            .identity_axis_offset(axis_id)
            .expect("a surviving axis still owns an identity triple after the retirement");
        for (&dim_id, post_range) in &reading.post_map.id_dim_ranges {
            let pre_range = reading.pre_map.id_dim_ranges[&dim_id].start;
            for within in 0..3 {
                reading.assert_carries(
                    post_range.start + post_id_offset + within,
                    pre_range + pre_id_offset + within,
                    &format!("axis {axis}'s identity triple in dimension {}, offset {within}", dim_id.0),
                );
            }
        }
    }
}

/// Retiring a spatial outcome axis from the middle of the layout renumbers
/// every later axis's outcome-memory pair, and the permutation that rearranges
/// the parameters keys that pair by the number rather than by the axis that
/// owns it. With a distinguishable value planted at every coordinate first,
/// each surviving axis's pair — in every Sentinel slot, across the posterior
/// mean, the covariance, the precision and both standardisation vectors — still
/// carries what that axis itself learned, and the retired axis's memory is gone
/// rather than donated to its rank inheritor. The identity block's per-axis
/// triple, keyed by the axis rather than by a rank, is the control beside it.
///
/// ´claim:lifespan:retiring-a-middle-spatial-axis-leaves-every-surviving-axis-pair-holding-its-own-memory´
/// ´test:crate:middle-axis-retirement-keeps-each-pair-with-its-own-axis´
#[test]
fn middle_axis_retirement_keeps_each_pair_with_its_own_axis() {
    let reading = retire_axes_after_planting(&[1, 2], &[7, 8, 9], &[8]);
    assert_surviving_axes_keep_their_own_memory(&reading, &[1, 2], &[7, 9]);

    // The retired axis's own memory is gone rather than parked somewhere: no
    // surviving coordinate carries what was learned at its pair positions.
    let retired_offset = reading
        .pre_map
        .slot_axis_offset(OutcomeAxisId(8))
        .expect("the retired axis owned a pair before the retirement");
    for s in [1u32, 2] {
        let pre_slot = reading.pre_map.sentinel_slots[&SentinelId(s)].start;
        for half in 0..2 {
            let orphan = planted_mu(pre_slot + retired_offset + half);
            assert!(
                !reading.mu.iter().any(|&v| (v - orphan).abs() < 1e-9),
                "the retired axis's memory at pre-event position {} survived the retirement; \
                 the permutation the real path computed was {}",
                pre_slot + retired_offset + half,
                reading.permutation,
            );
        }
    }
}

/// The rearrangement that carries a retirement into canonical order is the
/// identity, and reaching it is what proves the two drops cancelled: the
/// marginalisation compacted the retired axis's own pair out of the middle of
/// every slot tail, the key filter dropped the *last*-ranked pair key instead,
/// and the permutation those two produce together moves nothing because they
/// leave the surviving axes at the same ranks in the same order.
///
/// A refusal would leave the parameters exactly where the handlers put them,
/// and a rearrangement would move them; reading only the values cannot tell a
/// rearrangement that landed correctly from one that never ran. Pinning which
/// of the three this path takes is what stops the doc from narrating one
/// mechanism while the code runs another.
///
/// ´claim:lifespan:a-spatial-axis-retirement-produces-the-identity-permutation-rather-than-a-refusal´
/// ´test:crate:middle-axis-retirement-permutation-is-the-identity´
#[test]
fn middle_axis_retirement_permutation_is_the_identity() {
    let reading = retire_axes_after_planting(&[1, 2], &[7, 8, 9], &[8]);
    assert_eq!(
        reading.permutation, "bypassed: it came out the identity, so the compaction alone placed the values",
        "a middle retirement leaves the compacted vector already canonical",
    );
}

/// Retiring the last-ranked spatial axis renumbers nothing, so the drop the
/// marginalisation makes and the drop the key filter makes are the same
/// positions rather than merely the same count. It is the case the rank keying
/// was always right on, and it stands here as the floor the middle case has to
/// match.
///
/// ´claim:lifespan:retiring-the-last-ranked-spatial-axis-leaves-every-surviving-axis-pair-holding-its-own-memory´
/// ´test:crate:last-axis-retirement-keeps-each-pair-with-its-own-axis´
#[test]
fn last_axis_retirement_keeps_each_pair_with_its_own_axis() {
    let reading = retire_axes_after_planting(&[1, 2], &[7, 8, 9], &[9]);
    assert_surviving_axes_keep_their_own_memory(&reading, &[1, 2], &[7, 8]);
}

/// Retiring the first-ranked spatial axis renumbers every other axis, which is
/// the widest renumbering a single retirement can cause: the two drops are then
/// at opposite ends of the slot tail. The cancellation is exact there too, so
/// the distance between them is not what makes it work.
///
/// ´claim:lifespan:retiring-the-first-ranked-spatial-axis-leaves-every-surviving-axis-pair-holding-its-own-memory´
/// ´test:crate:first-axis-retirement-keeps-each-pair-with-its-own-axis´
#[test]
fn first_axis_retirement_keeps_each_pair_with_its_own_axis() {
    let reading = retire_axes_after_planting(&[1, 2], &[7, 8, 9], &[7]);
    assert_surviving_axes_keep_their_own_memory(&reading, &[1, 2], &[8, 9]);
}

/// Two spatial axes retired in one submission are order-free with each other,
/// so they gather into one chain and apply as one plan: a single joint
/// marginalisation over the union of their positions, one rebuild, one
/// publication. The surviving axis's outcome-memory pair in every Sentinel slot
/// still carries what that axis learned across all five channels the invariant
/// binds, its identity triple agrees as the control, and neither retired axis's
/// memory is left anywhere in the vector.
///
/// This is the boundary a single-event argument cannot reach, and it is where
/// the sequential applier failed. It took each axis's rank from the
/// authoritative model map, which every handler updated as it ran, and the
/// Sentinel slot positions from the dimension map, which the rebuild refreshed
/// only once after all of a submission's events. The second retirement
/// therefore mixed a fresh rank with stale positions and addressed coordinates
/// the first had already compacted away, which the marginalisation's own
/// precondition refused. Both halves of that mixture are gone: a removal now
/// addresses its pair from the map by the axis's own identifier, exactly as it
/// already addressed its identity triple, and the chain holds that one map still
/// until every operation in it has been planned
/// (´entry:assayer:wl-owner-dereg-scan´).
///
/// ´claim:lifespan:two-spatial-axes-retired-in-one-submission-apply-as-one-chain-and-the-survivor-keeps-its-own-memory´
/// ´test:crate:two-retirements-in-one-submission-apply-as-one-chain´
#[test]
fn two_retirements_in_one_submission_apply_as_one_chain() {
    let reading = retire_axes_after_planting(&[1, 2], &[7, 8, 9], &[7, 8]);
    assert_eq!(
        reading.chains_published, 1,
        "two removals are order-free with each other, so they are one chain and one publication",
    );
    assert_surviving_axes_keep_their_own_memory(&reading, &[1, 2], &[9]);

    // Neither retired axis's memory is parked anywhere: a joint marginalisation
    // that took one axis's positions and left the other's would show here.
    for retired in [7u32, 8] {
        let retired_offset = reading
            .pre_map
            .slot_axis_offset(OutcomeAxisId(retired))
            .expect("a retired axis owned a pair before the retirement");
        for s in [1u32, 2] {
            let pre_slot = reading.pre_map.sentinel_slots[&SentinelId(s)].start;
            for half in 0..2 {
                let orphan = planted_mu(pre_slot + retired_offset + half);
                assert!(
                    !reading.mu.iter().any(|&v| (v - orphan).abs() < 1e-9),
                    "axis {retired}'s memory at pre-event position {} survived the retirement; \
                     the permutation the real path computed was {}",
                    pre_slot + retired_offset + half,
                    reading.permutation,
                );
            }
        }
    }
}

/// A submission whose events are not all order-free with one another is applied
/// as several chains, publishes one snapshot per chain, and still leaves every
/// survivor holding its own memory. Retiring one spatial axis, registering
/// another, then retiring a third breaks the chain twice — a registration reads
/// live entity state to decide how wide it is and takes its positions from the
/// append, so it is order-free with nothing — and the three chains publish at
/// three consecutive versions.
///
/// The break is what makes the case safe rather than what makes it awkward: the
/// operation that seals a chain begins from the layout that chain published, so
/// the middle registration extends a vector the first retirement has already
/// compacted, and the last retirement addresses a map the registration's own
/// rebuild has already refreshed. The surviving pre-existing axis's pair, in
/// every Sentinel slot and across all five bound channels, carries what it
/// learned before any of it happened.
///
/// ´claim:lifespan:a-submission-that-breaks-its-chain-publishes-once-per-chain-and-every-survivor-keeps-its-own-memory´
/// ´test:crate:chain-break-publishes-per-chain-and-survivors-keep-their-memory´
#[test]
fn chain_break_publishes_per_chain_and_survivors_keep_their_memory() {
    let reading = apply_after_planting(
        &[1, 2],
        &[7, 8, 9],
        vec![
            LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(7)),
            LifecycleEvent::RegisterOutcomeAxis(spatial_axis_reg(OutcomeAxisId(10))),
            LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(8)),
        ],
    );
    assert_eq!(
        reading.chains_published, 3,
        "a registration is order-free with nothing, so it seals the chain before it and opens one of its own",
    );
    assert_surviving_axes_keep_their_own_memory(&reading, &[1, 2], &[9]);
}
