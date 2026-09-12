// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`register_sentinel_extends_model`] | lifespan | A Sentinel joining the system costs a fixed, computable amount of model: one occupancy indicator plus the base extraction block, added to the operational model and to the standardisation vectors alike. With no spatial axes and no interaction templates in play the growth is exactly that slot width, so the dimension a deployment ends up with is predictable from the registrations it made rather than discovered afterwards. |
//! | [`register_sentinel_triggers_early_refit`] | lifespan | Registration does not merely widen the model; it also tells the probability calibrator that its fit is now stale, the early-refit flag being clear beforehand and set afterwards. A new Sentinel's features enter carrying prior coefficients, so leaving the calibrator on its ordinary cadence would let the mapping from score to probability drift for as long as that cadence allows. |
//! | [`competitive_cell_entry_extends_model`] | lifespan | A cell entering the competitive set of a live identity dimension buys exactly one feature — its own indicator — when no Type-5 interaction templates are configured. Cell membership turns over far faster than registration does, so the per-cell cost is the smallest thing that can represent the cell at all. |
//! | [`register_identity_dimension_creates_tracker`] | lifespan | Registering an identity dimension creates the convergence tracker that will watch it and widens the model by both the dimension's own feature block and the cross-dimension aggregate block. The aggregate summarises across dimensions, so it comes into existence with the first dimension rather than waiting for a second one that may never arrive. |
//! | [`register_identity_second_dimension_extends_per_dimension_only`] | lifespan | Once one identity dimension exists, the next one costs only its own per- dimension block: the cross-dimension aggregate is built once and then shared. Charging for it twice would both inflate the dimension and leave two aggregate blocks competing to describe the same thing. |
//! | [`deregister_identity_dimension_removes_tracker`] | lifespan | The convergence tracker for a dimension does not outlive the dimension. That tracker set is the authoritative list the dimension-map rebuild reads, so a tracker left standing would keep resurrecting features for a dimension nothing feeds any more. |
//! | [`consecutive_removals_gather_into_one_chain`] | lifespan | Consecutive removals gather into one chain however many of them arrive and whatever they remove: a Sentinel retirement, an axis retirement and a competitive-cell exit all address their positions by their own entity's identifier in the layout the chain began with, so none of them can observe whether another has run. One chain is one plan, and one plan is what leaves no intermediate layout for a later read to be stale against. |
//! | [`a_registration_seals_the_open_chain`] | lifespan | A registration is order-free with nothing, itself included, so it seals whatever chain is open and becomes a chain of one. How wide a registration is depends on live entity state and where its coordinates land depends on the append, which is to say append order is rank order is layout — two registrations reordered are two different layouts, and a registration beside a removal is a different width. Sealing costs a publication and buys the guarantee that nothing in a chain was decided against a layout another member of it had already moved. |
//! | [`two_identity_dimension_removals_do_not_share_a_chain`] | lifespan | Two identity-dimension removals never share a chain, and the removal family's one exception is where the reason lies: the cross-dimension aggregate block leaves with the *last* dimension, and "last" counts the dimensions still standing rather than describing the one departing. Two of them planned against one entry layout would each see the other still there, neither would take the aggregate, and it would outlive everything it aggregates. Sealed apart, the second reads a count the first has already reduced. Each still chains freely with removals of other kinds, whose index sets are functions of their own entities alone. |
//! | [`an_empty_submission_gathers_one_empty_chain`] | lifespan | An empty submission gathers one empty chain, so it still publishes the version it was given. A caller waiting on that version is waiting for a statement about the state, and the state is the one the submission left. |

//! Lifecycle event handling for model dimension changes.
//!
//! This module processes lifecycle events that affect the model structure:
//! - Sentinel registration/deregistration
//! - Outcome axis registration/deregistration
//! - Identity dimension registration/deregistration
//! - Competitive cell entry/exit
//!
//! Each event may extend or marginalise the model dimensions, update the
//! `DimensionMap`, and trigger early Platt recalibration.
//!
//! Crate-level tests for this module live in `src/tests/lifecycle.rs`.
//!
//! # Cross-References
//!
//! - (´dec:construction:compound-batch´) — a submission's events are gathered into order-free chains, each applying as one plan
//! - (´inv:guarantee:structural-exactness´) — lifecycle operations are exact on the structure
//! - (´dec:calibration:unconditional-refit´) — registration marks the calibration fit stale
//! - (´dec:ordering:label-function´) — the label path these events are sequenced against

use std::sync::Arc;
use std::time::Instant;

use indexmap::IndexMap;

use crate::config::types::AssayerConfig;
use crate::feature::dimension_map::{
    DimensionMap, ID_AXIS_FEATURES_PER_AXIS, ID_CROSS_DIM_FEATURE_COUNT, ID_DIM_BASE_FEATURE_COUNT, SENTINEL_OCCUPANCY_WIDTH,
    SENTINEL_Q_BASE, SENTINEL_SPATIAL_FEATURES_PER_AXIS,
};
use crate::feature::interaction::{InteractionId, InteractionTemplate};
use crate::feature::permutation::LayoutPermutation;
use crate::feature::standardisation::{FeatureClass, assign_feature_classes, compact_standardisation, extend_standardisation};
use crate::health::{IdentityConvergenceTracker, PlattConvergenceTracker};
use crate::identity::CompetitiveCellId;
use crate::ledger::OutcomeLedger;
use crate::owner::commands::{EventOutcome, EventResult, LifecycleEvent, LifecycleResult, LifecycleSubmission};
use crate::snapshot::shared::SharedState;
use crate::snapshot::working::WorkingCopy;
use crate::types::{DimensionId, OutcomeAxisId, SentinelId};

// ═══════════════════════════════════════════════════════════════════════════════
// Order-Free Chains
// ═══════════════════════════════════════════════════════════════════════════════

/// What one lifecycle operation does to the layout, at the granularity the
/// commutation predicate reasons about.
///
/// Two operations are *order-free* when applying them in either order publishes
/// the same state. The only thing that can distinguish the two orders is what
/// each operation reads: one that reads a count or a position the other has
/// already changed sees a different world depending on when it runs, and one
/// that reads nothing but its own entity's identifier does not.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum OperationShape {
    /// Appends coordinates, reading live entity state to decide how many, and
    /// takes their positions from the append itself.
    Extension,
    /// Removes one Sentinel's own slot together with the interaction columns
    /// naming it, both addressed from the map by that Sentinel's identifier.
    SentinelRemoval,
    /// Removes one outcome axis's triple inside every identity block and its
    /// outcome-memory pair inside every Sentinel slot, both addressed from the
    /// map by that axis's identifier.
    AxisRemoval,
    /// Removes one identity dimension's own block, its competitive indicators
    /// and their Type-5 columns — and, when it is the last dimension standing,
    /// the cross-dimension aggregate block as well.
    IdentityDimensionRemoval,
    /// Removes one competitive cell's indicator and the Type-5 columns naming
    /// that cell.
    CellRemoval,
}

impl OperationShape {
    /// The shape of one lifecycle event. The hibernating removals share their
    /// destructive twins' shape, because what they add is a copy of the
    /// departing block and not a different set of positions
    /// (´alg:registry:hibernation´).
    const fn of(event: &LifecycleEvent) -> Self {
        match event {
            LifecycleEvent::RegisterSentinel(_)
            | LifecycleEvent::RegisterOutcomeAxis(_)
            | LifecycleEvent::RegisterIdentityDimension { .. }
            | LifecycleEvent::CompetitiveCellEntry { .. } => Self::Extension,
            LifecycleEvent::DeregisterSentinel(_) | LifecycleEvent::HibernateSentinel(_) => Self::SentinelRemoval,
            LifecycleEvent::DeregisterOutcomeAxis(_) | LifecycleEvent::HibernateOutcomeAxis(_) => Self::AxisRemoval,
            LifecycleEvent::DeregisterIdentityDimension(_) => Self::IdentityDimensionRemoval,
            LifecycleEvent::CompetitiveCellExit { .. } => Self::CellRemoval,
        }
    }

    /// Whether two operations of these shapes are order-free with respect to
    /// each other, which is what admits them to one chain.
    ///
    /// The predicate is conservative by construction: a pair is order-free only
    /// where the argument below is made for it, and every unproven pair breaks
    /// the chain. Breaking is never wrong. The sealed chain applies and
    /// publishes, and the breaking operation then reads a freshly published
    /// layout — which is exactly the state a submission carrying it alone would
    /// have handed it. Proving a further pair widens chains later without
    /// changing what any of them mean.
    ///
    /// **Removal against removal is order-free**, with one exception below.
    /// Each removal addresses its positions by its own entity's identifier in
    /// the entry map — a Sentinel's slot and the interaction columns naming it,
    /// an axis's pair offset and its identity triples, a cell's indicator — so
    /// no removal's index set is a function of whether another has already run.
    /// The sets are disjoint except where two removals genuinely name one
    /// coordinate (an interaction column naming both departing Sentinels; an
    /// axis triple inside a departing dimension's block), and a coordinate named
    /// twice is removed once under either order. What the joint marginalisation
    /// then does with the union is what the two sequential ones would have done:
    /// Gaussian marginalisation composes, so conditioning `A` away and then `B`
    /// leaves the same posterior over the survivors as conditioning `A ∪ B` away
    /// in one Schur complement (´thm:gaussian:marginalisation´). The surviving
    /// layout is order-free too, because compaction preserves the entry order of
    /// whatever it keeps and the entry order does not depend on which removal
    /// ran first.
    ///
    /// **The exception is two identity-dimension removals.** That removal is the
    /// one whose index set is not a function of its own entity alone: the
    /// cross-dimension aggregate block leaves with the *last* dimension, and
    /// "last" is a count of the dimensions still standing rather than a property
    /// of the dimension departing. Two of them in one chain would each read the
    /// entry count, neither would see itself as last, and the aggregate would
    /// outlive every dimension it aggregates. Sealing the chain between them is
    /// what makes the second read a count the first has already reduced.
    ///
    /// **An extension is order-free with nothing, itself included.** How many
    /// coordinates a registration appends is read from live entity state — a
    /// Sentinel's slot width counts the spatial axes, an axis's per-Sentinel
    /// cost counts the Sentinels, an identity dimension pays for the aggregate
    /// block only when it is the first — so a registration standing beside any
    /// other operation appends a different number of coordinates depending on
    /// which ran first. And appended coordinates take their positions from the
    /// append, so even two registrations that appended equal widths would lay
    /// them down in the order they arrived: append order is rank order is
    /// layout. A registration is therefore a chain of one, which is a chain like
    /// any other.
    ///
    /// A same-identifier registration and retirement submitted together land on
    /// a defined behaviour by this rule rather than by a special case: the two
    /// shapes do not chain, so the registration publishes and the retirement
    /// then removes what it published, in the order the two were submitted.
    const fn is_order_free_with(self, other: Self) -> bool {
        !matches!(
            (self, other),
            // An extension's width is read from live entity state and its
            // coordinates land where the append put them, so both depend on
            // when it runs.
            (Self::Extension, _)
                | (_, Self::Extension)
                // "Last dimension" is a count of what is still standing, so two
                // of these planned against one layout would each see the other.
                | (Self::IdentityDimensionRemoval, Self::IdentityDimensionRemoval)
        )
    }
}

/// Walks the ordered event queue, gathering maximal runs of operations that are
/// order-free with respect to one another.
///
/// Order is never discarded: the chains partition the queue into consecutive
/// runs and the runs are applied in the order the events arrived. A chain is
/// maximal — it grows until an operation arrives that is not order-free with
/// something already in it, and that operation opens the next chain. The test
/// is against every member of the open chain rather than against its last
/// member, because being order-free is a pairwise relation and a run is
/// order-free only when every pair in it is.
///
/// An empty submission gathers one empty chain, so it still publishes the
/// version it was given: a caller waiting on that version is waiting for a
/// statement about the state, and the state is the one the submission left.
fn gather_chains(events: &[LifecycleEvent]) -> Vec<std::ops::Range<usize>> {
    if events.is_empty() {
        let empty_chain = 0..0;
        return vec![empty_chain];
    }

    let mut chains = Vec::new();
    let mut start = 0;
    while start < events.len() {
        let mut end = start + 1;
        while end < events.len() {
            let candidate = OperationShape::of(&events[end]);
            if events[start..end]
                .iter()
                .all(|open| OperationShape::of(open).is_order_free_with(candidate))
            {
                end += 1;
            } else {
                break;
            }
        }
        chains.push(start..end);
        start = end;
    }
    chains
}

// ═══════════════════════════════════════════════════════════════════════════════
// Lifecycle Handler
// ═══════════════════════════════════════════════════════════════════════════════

/// Processes a lifecycle submission containing multiple events.
///
/// The ordered queue is gathered into maximal chains of order-free operations
/// ([`gather_chains`]), and each chain is applied as ONE plan computed entirely
/// against the layout the previous publication left — the chain's *entry
/// layout*:
///
/// 1. every operation contributes the positions it removes, addressed against
///    that one entry layout, and performs its own identity-keyed side effects
/// 2. one joint marginalisation over the union of those positions
/// 3. one compaction of the standardisation vectors over the same union
/// 4. one `DimensionMap::rebuild()`
/// 5. one `ArcSwap` publish, at the chain's own version
///
/// Completion is signalled once, after the last chain.
///
/// This is what makes a whole family of staleness defects structurally
/// impossible rather than individually patched. Inside a chain there is no
/// mid-application mutation for a map read to be stale against, because every
/// index is computed against one immutable entry layout; and an operation that
/// breaks the chain begins from a freshly published layout by construction. The
/// measured case is two spatial-axis retirements in one submission, where the
/// second handler mixed a rank the first had already renumbered with slot
/// positions the first had already compacted away
/// (´entry:assayer:wl-owner-dereg-scan´).
///
/// # Arguments
///
/// * `working` — Mutable working copy of model state
/// * `submission` — The lifecycle submission with ordered events
/// * `shared` — Shared state for snapshot publication
/// * `platt_tracker` — Platt convergence tracker (for early refit marking)
/// * `identity_trackers` — Per-dimension identity trackers
/// * `config` — Configuration
/// * `version` — the version the first chain publishes at; advanced once per
///   further chain and left at the last version published, so the caller's
///   counter follows the publications rather than the submissions
/// * `publish_timestamp` — Monotonic instant supplied by the engine clock
///
/// # Returns
///
/// The result of processing each event, each naming the chain whose application
/// it was part of.
///
/// # Cross-References
///
/// - (´dec:construction:compound-batch´) — the batch contract this handler keeps
#[allow(clippy::unnecessary_wraps)] // error paths deferred to Contracts and Boundaries (Layer 5)
#[allow(clippy::too_many_arguments)] // Matches the lifecycle handler signature convention
pub fn handle_lifecycle_submission(
    working: &mut WorkingCopy,
    submission: LifecycleSubmission,
    shared: &SharedState,
    platt_tracker: &mut PlattConvergenceTracker,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
    outcome_ledger: &OutcomeLedger,
    config: &AssayerConfig,
    version: &mut u64,
    publish_timestamp: Instant,
    now: &crate::types::PersistentTimestamp,
) -> LifecycleResult {
    let mut results = Vec::with_capacity(submission.events.len());
    let chains = gather_chains(&submission.events);
    let chain_count = chains.len();

    for (chain_index, chain) in chains.into_iter().enumerate() {
        let chain_events = &submission.events[chain.clone()];

        results.extend(apply_chain(
            working,
            chain_events,
            chain.start,
            chain_index,
            platt_tracker,
            identity_trackers,
            outcome_ledger,
            config,
            now,
        ));

        // Rebuild DimensionMap from authoritative entity state after this
        // chain's single application.
        rebuild_dimension_map(working, chain_events, identity_trackers);
        resync_standardisation(working);

        // Verify invariant: dim_map.p == model.p()
        debug_assert_eq!(
            working.dimension_map.p,
            working.operational.dim(),
            "DimensionMap.p must match model dimension after a chain's application"
        );

        // Post-lifecycle drift reset (´tab:monitoring:drift-resets´): a
        // Sentinel or axis event alters the feature space the predictions
        // were made in, so evidence accumulated across it would report a
        // drift that is really a change of coordinates. Accumulators reset
        // for all models; the smoothed diagnostics survive. The chain is the
        // granularity because the chain is what changes the coordinates.
        if chain_events.iter().any(changes_feature_space) {
            for drift in working.drift_state.values_mut() {
                drift.reset_cusums();
            }
        }

        if chain_index + 1 == chain_count {
            finish_submission(working, &submission.events, identity_trackers, config, now);
        }

        // Publish this chain's snapshot.
        let snapshot = working.to_snapshot_at(*version, publish_timestamp);
        shared.published.store(Arc::new(snapshot));
        if chain_index + 1 < chain_count {
            *version += 1;
        }
    }

    // Signal completion if channel provided
    if let Some(tx) = submission.completion {
        drop(tx.send(Ok(results.clone())));
    }

    Ok(results)
}

/// Whether an event moves the feature space the predictions were made in.
const fn changes_feature_space(event: &LifecycleEvent) -> bool {
    matches!(
        event,
        LifecycleEvent::RegisterSentinel(_)
            | LifecycleEvent::DeregisterSentinel(_)
            | LifecycleEvent::HibernateSentinel(_)
            | LifecycleEvent::RegisterOutcomeAxis(_)
            | LifecycleEvent::DeregisterOutcomeAxis(_)
            | LifecycleEvent::HibernateOutcomeAxis(_)
    )
}

/// The submission-scoped work that runs once, before the last chain publishes.
///
/// Both readings are of the submission rather than of any one chain: the
/// archive sweep is the steward's own pass over storage, and the trackers are
/// told what a submission did to the competitive sets against the set the last
/// rebuild left standing.
fn finish_submission(
    working: &mut WorkingCopy,
    events: &[LifecycleEvent],
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) {
    // Reclaim expired archive records on the steward's own pass. Expiry is
    // enforced lazily rather than by a thread that exists to wait for it: the
    // restore reads the same wall clock, so an expired record is refused
    // whether or not this sweep has reached it, and the sweep's job is to stop
    // the storage outliving the permission to use it
    // (´alg:registry:hibernation´), (´dec:concurrency:single-steward´).
    let expired = working.hibernation.expire(now, config.hibernation.expiry_hours());
    if expired > 0 {
        tracing::info!(expired, "Hibernation archive records reclaimed at expiry");
    }

    // Record this submission's competitive-set changes on the trackers,
    // per dimension, against the rebuilt current set
    // (´tab:monitoring:dimension-health´).
    record_identity_change_events(events, working, identity_trackers);
}

/// Brings the standardisation vectors back onto the map the rebuild just laid
/// out.
///
/// After rebuild, `dimension_map.p == operational.dim()`. If the standardisation
/// vectors are longer (stale cold-start state) or shorter (shouldn't happen),
/// they resize to match.
fn resync_standardisation(working: &mut WorkingCopy) {
    let new_p = working.dimension_map.p;
    let old_len = working.feature_means.len();
    working.feature_means.resize(new_p, 0.0);
    working.feature_variances.resize(new_p, 1.0);

    // Recompute every class from the rebuilt map rather than padding with the
    // generic standardising class (´req:standardisation:class-assignment´).
    // The signal block's classes are the schema's, and no lifecycle event
    // changes the signal block, so they are read back from the vector the
    // construction derived them into. Positions the resize had to invent take
    // their class's prior rather than neutral moments, which is what a
    // mis-classed position costs: it is standardised as though already
    // standardised (´entry:assayer:wl-feature-cold-start-classes´).
    let sig_range = working.dimension_map.sig_range.start..working.dimension_map.sig_range.end;
    let signal_classes: Vec<FeatureClass> = sig_range
        .map(|i| working.feature_classes.get(i).copied().unwrap_or(FeatureClass::ZScore))
        .collect();
    working.feature_classes = assign_feature_classes(&working.dimension_map, &signal_classes);
    for i in old_len..new_p {
        let (mu, var) = working.feature_classes[i].prior();
        working.feature_means[i] = mu;
        working.feature_variances[i] = var;
    }

    // The moments above are what this lifecycle change publishes, and if the
    // cold ramp is still running they become its new base
    // (´req:standardisation:lifecycle-entries´). Every position that survives
    // the change keeps what was just published for it rather than reverting to
    // its own prior, and the sample gathered under the layout being replaced is
    // discarded — a part-gathered vector at the old width says nothing about a
    // position that did not exist when gathering began.
    //
    // The layout generation advances here and nowhere else, so a vector
    // assembled before this change and still in flight is refused on arrival
    // rather than mixed into positions it does not describe.
    working.rebase_cold_ramp();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Chain Application
// ═══════════════════════════════════════════════════════════════════════════════

/// Applies one order-free chain as a single plan against its entry layout.
///
/// The operations are planned in submitted order — each reading the same
/// immutable entry layout, each performing its own identity-keyed side effects —
/// and the positions they name are then removed together: one joint
/// marginalisation, one compaction. There is no intermediate state for a later
/// operation's map read to be stale against, because the state changes once, at
/// the end, for all of them.
///
/// The diagnostics are the chain's rather than any operation's. One Schur
/// computation was performed over the union of the removed positions, and its
/// error partition is a property of that computation; splitting it back out per
/// index set would require attributing a correction that was never computed per
/// index set. Every operation that contributed positions therefore receives the
/// chain's single aggregate, and names the chain it belongs to so a consumer can
/// tell one application's results from another's
/// (´entry:health:marginalise-event´).
#[allow(clippy::too_many_arguments)] // Matches the lifecycle handler signature convention
fn apply_chain(
    working: &mut WorkingCopy,
    chain_events: &[LifecycleEvent],
    first_event_index: usize,
    chain_index: usize,
    platt_tracker: &mut PlattConvergenceTracker,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
    outcome_ledger: &OutcomeLedger,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) -> Vec<EventResult> {
    let mut planned = Vec::with_capacity(chain_events.len());
    let mut removed: Vec<usize> = Vec::new();
    for event in chain_events {
        let plan = plan_single_event(working, event, platt_tracker, identity_trackers, outcome_ledger, config, now);
        removed.extend_from_slice(&plan.remove_indices);
        planned.push(plan);
    }
    removed.sort_unstable();
    removed.dedup();

    let (fell_back, aggregate) = if removed.is_empty() {
        (false, None)
    } else {
        // Marginalise every full-dimension model via the Schur complement
        // (´dec:posterior:half-solve´). The anchor is fixed dimension and is
        // not marginalised (´dec:substrate:anchor-invariant´).
        let (fell_back, diagnostics) = working.marginalise_all_full_dim(&removed, &config.schur);
        if fell_back {
            tracing::warn!(
                positions = removed.len(),
                "Chain marginalisation fell back to B_kk for at least one model"
            );
        }

        // Compact standardisation over the same union, against the same entry
        // width the positions were addressed in.
        let keep: Vec<usize> = (0..working.feature_means.len())
            .filter(|i| removed.binary_search(i).is_err())
            .collect();
        compact_standardisation(
            &mut working.feature_means,
            &mut working.feature_variances,
            &mut working.feature_classes,
            &keep,
        );

        let health: crate::health::MarginalisationHealth = (&diagnostics).into();
        (fell_back, Some(health))
    };

    let chain_outcome = if fell_back {
        EventOutcome::SchurFallback
    } else {
        EventOutcome::Success
    };

    planned
        .into_iter()
        .enumerate()
        .map(|(offset, plan)| {
            let contributed = !plan.remove_indices.is_empty();
            EventResult {
                event_index: first_event_index + offset,
                chain_index,
                outcome: if contributed { chain_outcome.clone() } else { plan.outcome },
                marginalisation: if contributed { aggregate.clone() } else { None },
            }
        })
        .collect()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Single Event Planning
// ═══════════════════════════════════════════════════════════════════════════════

/// Records one submission's competitive-set changes on the per-dimension
/// trackers: entries and exits grouped per dimension, against the current
/// set the rebuilt map holds (´dec:health:on-demand-composite´).
fn record_identity_change_events(
    events: &[LifecycleEvent],
    working: &WorkingCopy,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
) {
    type CellLists = (Vec<CompetitiveCellId>, Vec<CompetitiveCellId>);
    let mut per_dimension: std::collections::HashMap<DimensionId, CellLists> = std::collections::HashMap::new();
    for event in events {
        match event {
            LifecycleEvent::CompetitiveCellEntry { dimension, cell } => {
                per_dimension.entry(*dimension).or_default().0.push(*cell);
            }
            LifecycleEvent::CompetitiveCellExit { dimension, cell } => {
                per_dimension.entry(*dimension).or_default().1.push(*cell);
            }
            _ => {}
        }
    }

    for (dim_id, (entries, exits)) in per_dimension {
        let Some(tracker) = identity_trackers.get_mut(&dim_id) else {
            // Racing a deregistration; the events were dropped above too.
            continue;
        };
        let current_set: Vec<CompetitiveCellId> = working
            .dimension_map
            .competitive_indices
            .get(&dim_id)
            .map(|cells| cells.keys().copied().collect())
            .unwrap_or_default();
        tracker.record_change_event(&entries, &exits, &current_set);
    }
}

/// Plans a single lifecycle event against its chain's entry layout.
///
/// A removal returns the positions it takes and performs its identity-keyed
/// side effects — the ledger partition, the per-axis rows, the tracker, the
/// hibernation archive — without touching the parameters. An extension has
/// nothing to plan and applies itself here, which is sound because an extension
/// is always a chain of one.
fn plan_single_event(
    working: &mut WorkingCopy,
    event: &LifecycleEvent,
    platt_tracker: &mut PlattConvergenceTracker,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
    outcome_ledger: &OutcomeLedger,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) -> PlannedEvent {
    // TODO ´todo:code:mark-early-refit-pending-after-sentinel´: mark `early_refit_pending` after Sentinel,
    // outcome-axis, and identity-dimension registration/deregistration.
    // Current code only does this in the Sentinel registration handler.
    // Competitive-cell entry/exit changes model dimensions too; either wire
    // it here or record the exclusion against the refit trigger
    // (´dec:calibration:unconditional-refit´).
    match event {
        LifecycleEvent::RegisterSentinel(reg) => PlannedEvent::applied(handle_register_sentinel(
            working,
            reg.id,
            platt_tracker,
            outcome_ledger,
            config,
            now,
        )),

        LifecycleEvent::DeregisterSentinel(sentinel_id) => {
            plan_deregister_sentinel(working, *sentinel_id, outcome_ledger, Disposition::Destructive, now)
        }

        LifecycleEvent::HibernateSentinel(sentinel_id) => {
            plan_deregister_sentinel(working, *sentinel_id, outcome_ledger, Disposition::Hibernating, now)
        }

        LifecycleEvent::RegisterOutcomeAxis(reg) => {
            PlannedEvent::applied(handle_register_outcome_axis(working, reg, config, now))
        }

        LifecycleEvent::DeregisterOutcomeAxis(axis_id) => {
            plan_deregister_outcome_axis(working, *axis_id, outcome_ledger, Disposition::Destructive, now)
        }

        LifecycleEvent::HibernateOutcomeAxis(axis_id) => {
            plan_deregister_outcome_axis(working, *axis_id, outcome_ledger, Disposition::Hibernating, now)
        }

        LifecycleEvent::RegisterIdentityDimension { id } => PlannedEvent::applied(handle_register_identity_dimension(
            working,
            *id,
            identity_trackers,
            config,
            now,
        )),

        LifecycleEvent::DeregisterIdentityDimension(dim_id) => {
            plan_deregister_identity_dimension(working, *dim_id, identity_trackers)
        }

        LifecycleEvent::CompetitiveCellEntry { dimension, cell } => {
            PlannedEvent::applied(handle_competitive_cell_entry(working, *dimension, *cell))
        }

        LifecycleEvent::CompetitiveCellExit { dimension, cell } => plan_competitive_cell_exit(working, *dimension, *cell),
    }
}

/// What a removal does with the departing entity's own parameter block.
///
/// The two dispositions differ in exactly one thing: whether the block is
/// copied into the archive before the marginalisation drops it. Everything
/// else a removal does — the Schur complement on the surviving models, the
/// standardisation compaction, the Ledger removal, the map rebuild — is the
/// same under both, because the specification's hibernation is an addition to
/// the deregistration protocol rather than a different protocol
/// (´alg:registry:hibernation´), (´dec:construction:destructive-deregistration´).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Disposition {
    /// Nothing is archived; re-registering the identifier produces a fresh
    /// entity (´dec:construction:destructive-deregistration´).
    Destructive,
    /// The entity's own block is archived against its identifier for a later
    /// registration to age and take back (´alg:registry:hibernation´).
    Hibernating,
}

/// One operation's contribution to its chain's plan.
struct PlannedEvent {
    /// The outcome for an operation that contributes no positions. An operation
    /// that does contribute takes its chain's outcome instead, because the
    /// removal it took part in either fell back or did not, once, for all of
    /// them.
    outcome: EventOutcome,
    /// The positions this operation removes, addressed against the chain's
    /// entry layout. Empty for an extension, and for a removal whose entity the
    /// entry layout does not hold.
    remove_indices: Vec<usize>,
}

impl PlannedEvent {
    /// An operation that applied itself and contributes no positions.
    const fn applied(outcome: EventOutcome) -> Self {
        Self {
            outcome,
            remove_indices: Vec::new(),
        }
    }

    /// A removal that contributes the positions it named.
    const fn removing(remove_indices: Vec<usize>) -> Self {
        Self {
            outcome: EventOutcome::Success,
            remove_indices,
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Sentinel Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles Sentinel registration.
///
/// - Extends all models by `slot_width` + per-Sentinel interaction features
/// - Extends standardisation statistics
/// - Marks early Platt refit pending (´dec:calibration:unconditional-refit´)
///
/// The slot width is `1 (occupancy) + q`, where
/// `q = SENTINEL_Q_BASE + SENTINEL_SPATIAL_FEATURES_PER_AXIS × m_s`.
///
/// Interaction features added for the new Sentinel:
/// - Type-1 templates: 1 per template
/// - Type-2 templates: `n_existing` per template (new pairs with each existing Sentinel)
/// - Type-3 templates: 1 per template
/// - Type-4 / Type-5: 0 (fixed / per-cell, not per-Sentinel)
///
/// Where the archive holds a record for this identifier, the record is taken
/// — taken rather than read, because a re-registration ends the archive's
/// usability whatever it decides to do with the record — and the slot's own
/// positions are written from it instead of standing at the prior
/// (´alg:registry:hibernation´). The interaction features are never
/// restored: they are couplings between this Sentinel and another, and
/// couplings are what hibernation does not keep
/// (´cav:limitation:hibernation´).
fn handle_register_sentinel(
    working: &mut WorkingCopy,
    sentinel_id: SentinelId,
    platt_tracker: &mut PlattConvergenceTracker,
    outcome_ledger: &OutcomeLedger,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) -> EventOutcome {
    outcome_ledger.create_sentinel(sentinel_id);

    // Compute slot width from DimensionMap constants.
    let spatial_axis_count = working.outcome_models.values().filter(|m| m.spatial).count();
    let q = SENTINEL_Q_BASE + SENTINEL_SPATIAL_FEATURES_PER_AXIS * spatial_axis_count;
    let slot_width = SENTINEL_OCCUPANCY_WIDTH + q;

    // Interaction features this registration adds. The wildcard gains one
    // feature per Sentinel already present; the named pair gains its single
    // feature only when this registration completes it — the registering
    // Sentinel is one of the two it names and the other is already there
    // (´def:feature:template-named-pair´). A count of the population cannot
    // express that, so the arm reads the identifiers.
    let slots = &working.dimension_map.sentinel_slots;
    let n_existing = slots.len();
    let interaction_count: usize = working
        .dimension_map
        .interaction_templates
        .iter()
        .map(|t| match t {
            InteractionTemplate::Type1 { .. } => 1,
            InteractionTemplate::Type2 {
                sentinel_a, sentinel_b, ..
            } => {
                let completes = (*sentinel_a == sentinel_id && slots.contains_key(sentinel_b))
                    || (*sentinel_b == sentinel_id && slots.contains_key(sentinel_a));
                usize::from(completes)
            }
            InteractionTemplate::Type3 { .. } => n_existing,
            InteractionTemplate::Type4 { .. } | InteractionTemplate::Type5 { .. } => 0,
        })
        .sum();

    let total_new = slot_width + interaction_count;

    // Where the extension puts the new coordinates, which is where an
    // archived block would be written back.
    let appended_at = working.operational.dim();

    // Extend every full-dimension model — operational, sister, each
    // outcome axis (´thm:gaussian:extension´). The anchor is fixed
    // dimension and is not extended (´dec:substrate:anchor-invariant´).
    working.extend_all_full_dim(total_new, config.model.lambda_prior);

    // TODO ´todo:code:classify-sentinel-slot-sub-groups-per´: classify Sentinel slot sub-groups per Appendix B
    // instead of treating the whole extraction block as ZScore.
    // Extend standardisation: occupancy indicator + extraction z-scores + interactions
    let new_classes: Vec<FeatureClass> = std::iter::once(FeatureClass::OccupancyIndicator)
        .chain(std::iter::repeat_n(FeatureClass::ZScore, q))
        .chain(std::iter::repeat_n(FeatureClass::InteractionProduct, interaction_count))
        .collect();

    extend_standardisation(
        &mut working.feature_means,
        &mut working.feature_variances,
        &mut working.feature_classes,
        &new_classes,
        working.operational.dim(),
    );

    // The Sentinel's own block is the slot alone: the occupancy indicator and
    // the extraction width, which is the q + 1 features the preservation
    // scope names (´alg:registry:hibernation´). The interaction features the
    // same extension appended are couplings to other Sentinels, and couplings
    // are what hibernation does not keep (´cav:limitation:hibernation´).
    let arriving = crate::model::hibernation::HibernationLayout::Sentinel {
        width: slot_width,
        spatial_axes: spatial_axis_count,
    };
    let restored = working
        .hibernation
        .take_sentinel(sentinel_id)
        .map(|record| restore_entity_block(working, &record, appended_at, arriving, config, now, "Sentinel"));

    // Mark early Platt refit pending (´dec:calibration:unconditional-refit´).
    platt_tracker.mark_early_refit();

    tracing::debug!(
        ?sentinel_id,
        slot_width,
        interaction_count,
        total_new,
        ?restored,
        "Registered Sentinel, extended model"
    );

    EventOutcome::Success
}

/// Writes an archived block over the positions an extension has just created,
/// or says why it did not.
///
/// The single place the restore contract is applied, for both registries: the
/// record must be of this build's generation, internally consistent, unexpired
/// against the configured wall-clock lifetime, describing exactly the positions
/// being created, and carrying only blocks the models may hold. A refusal is
/// not a failure: the registration keeps the extension's prior, which is what a
/// registration with no archive gets, and which is the fallback the algorithm
/// itself names (´alg:registry:hibernation´).
///
/// Admission is the whole of what can be refused here. Whether a model's aged
/// block still stands above the replenishment floor is a separate question and
/// is not one of these: each model ages the block at its own rate, so the
/// question is asked per model inside the restore and answered as the report's
/// split between the models restored and the models left at the prior
/// (´req:gaussian:prior-replenishment-floor´). A restore that reports every
/// model at the prior is still an `Ok`, because the record was admitted and
/// each model then answered the floor for itself.
fn restore_entity_block(
    working: &mut WorkingCopy,
    record: &crate::model::hibernation::HibernationRecord,
    appended_at: usize,
    arriving: crate::model::hibernation::HibernationLayout,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
    entity: &'static str,
) -> Result<crate::model::hibernation::RestoreReport, crate::model::hibernation::RestoreRefusal> {
    let elapsed_hours = record.hours_since_archived(now);
    let expiry_hours = config.hibernation.expiry_hours();
    match record.admit(arriving, elapsed_hours, expiry_hours) {
        Ok(()) => {
            let report = working.restore_self_structure(record, appended_at, now, &config.model, config.temporal.gamma_t_core);
            tracing::debug!(
                entity,
                elapsed_hours,
                restored = report.restored,
                at_prior = report.at_prior,
                "Hibernation restore applied"
            );
            Ok(report)
        }
        Err(refusal) => {
            tracing::info!(
                entity,
                ?refusal,
                elapsed_hours,
                expiry_hours,
                "Hibernation archive refused; registration extends at the prior"
            );
            Err(refusal)
        }
    }
}

/// Plans Sentinel deregistration, destructive or hibernating.
///
/// - Names the Sentinel's slot and the interaction columns referencing it
/// - Archives the Sentinel's own slot block, where the disposition is
///   hibernating (´alg:registry:hibernation´)
/// - Drops the Sentinel's ledger partition
///
/// The archive is taken before the marginalisation and from the slot alone.
/// Before, because the Schur complement folds the departing block into the
/// survivors and then drops it, so afterwards there is nothing left to copy
/// (´alg:registry:sentinel-deregistration´). "Before" is now structural rather
/// than merely first in this function: the chain marginalises once, at the end,
/// so every archive a chain takes is read off the entry parameters — a second
/// removal in the same submission can no longer archive a block a first one has
/// already compacted. From the slot alone, because the interaction features this
/// removal also takes are couplings between this Sentinel and another, and the
/// preservation scope is the entity's own structure
/// (´cav:limitation:hibernation´).
fn plan_deregister_sentinel(
    working: &mut WorkingCopy,
    sentinel_id: SentinelId,
    outcome_ledger: &OutcomeLedger,
    disposition: Disposition,
    now: &crate::types::PersistentTimestamp,
) -> PlannedEvent {
    // Find the slot indices to remove.
    let Some(slot_range) = working.dimension_map.sentinel_slots.get(&sentinel_id) else {
        tracing::warn!(?sentinel_id, "Deregister: Sentinel not found in DimensionMap");
        drop(outcome_ledger.remove_sentinel(sentinel_id));
        return PlannedEvent::applied(EventOutcome::Success);
    };
    let slot_positions: Vec<usize> = slot_range.to_range().collect();
    let mut remove_indices: Vec<usize> = slot_positions.clone();

    if disposition == Disposition::Hibernating {
        let spatial_axes = working.outcome_models.values().filter(|m| m.spatial).count();
        let layout = crate::model::hibernation::HibernationLayout::Sentinel {
            width: slot_positions.len(),
            spatial_axes,
        };
        let record = working.archive_self_structure(&slot_positions, layout, now);
        working.hibernation.store_sentinel(sentinel_id, record);
        tracing::debug!(
            ?sentinel_id,
            width = slot_positions.len(),
            spatial_axes,
            "Sentinel block archived for hibernation"
        );
    }

    // Also remove interaction features referencing this Sentinel.
    for (id, &idx) in &working.dimension_map.interaction_indices {
        let references_sentinel = match id {
            InteractionId::PerSentinel { sentinel, .. } => *sentinel == sentinel_id,
            InteractionId::PerPair {
                sentinel_a, sentinel_b, ..
            } => *sentinel_a == sentinel_id || *sentinel_b == sentinel_id,
            InteractionId::Fixed { .. } | InteractionId::PerCell { .. } => false,
        };
        if references_sentinel {
            remove_indices.push(idx);
        }
    }

    remove_indices.sort_unstable();
    remove_indices.dedup();

    drop(outcome_ledger.remove_sentinel(sentinel_id));

    tracing::debug!(?sentinel_id, planned = remove_indices.len(), "Deregistered Sentinel");

    PlannedEvent::removing(remove_indices)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Outcome Axis Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles outcome axis registration.
///
/// - If `spatial_features: Enabled`: extends each Sentinel's slot
///   by 2 features (compressed + raw axis EWMA). Per-axis EWMA
///   rows in `LedgerEntry::per_axis` grow on-demand as
///   `Vec<(OutcomeAxisId, f64, f64)>` entries.
/// - Creates a `WorkingAxisModel` in the working copy.
fn handle_register_outcome_axis(
    working: &mut WorkingCopy,
    reg: &crate::owner::commands::OutcomeAxisRegistration,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) -> EventOutcome {
    use crate::types::SpatialFeaturePolicy;

    let axis_id = reg.id;

    // Spatial features: each Sentinel gains 2 features per axis
    // (compressed + raw) ONLY when spatial_features is Enabled.
    let features_per_sentinel = match reg.spatial_features {
        SpatialFeaturePolicy::Enabled => SENTINEL_SPATIAL_FEATURES_PER_AXIS,
        SpatialFeaturePolicy::Disabled => 0,
    };
    let n_sentinels = working.dimension_map.sentinel_slots.len();
    let spatial_new = n_sentinels * features_per_sentinel;
    let identity_axis_new = working.dimension_map.id_dim_ranges.len() * ID_AXIS_FEATURES_PER_AXIS;
    let total_new = spatial_new + identity_axis_new;

    // Where the extension puts the new coordinates, which is where an
    // archived block would be written back.
    let appended_at = working.operational.dim();

    if total_new > 0 {
        // Extend operational, sister, and every *existing* outcome-axis
        // model. The new axis model is created below at the post-extension
        // dimension and so does not need to be extended itself.
        // The extension itself is exact (´thm:gaussian:extension´).
        working.extend_all_full_dim(total_new, config.model.lambda_prior);

        // Extend standardisation: one class triple per identity dimension
        // (´req:standardisation:class-assignment´).
        let mut new_classes = Vec::with_capacity(total_new);
        for _ in 0..working.dimension_map.id_dim_ranges.len() {
            new_classes.extend_from_slice(&[
                FeatureClass::OutcomeAxisCompressed,
                FeatureClass::OutcomeAxisRaw,
                FeatureClass::OutcomeAxisStability,
            ]);
        }
        new_classes.extend(
            std::iter::repeat_with(|| vec![FeatureClass::OutcomeAxisCompressed, FeatureClass::OutcomeAxisRaw])
                .take(n_sentinels)
                .flatten(),
        );

        extend_standardisation(
            &mut working.feature_means,
            &mut working.feature_variances,
            &mut working.feature_classes,
            &new_classes,
            working.operational.dim(),
        );
    }

    // The axis's own block, before its own model exists: the record never
    // held one for the axis being registered, because the model that would
    // have carried it was destroyed at the deregistration that wrote the
    // record (´alg:registry:axis-deregistration´). The new axis model is
    // therefore created at the prior below, exactly as the registration
    // algorithm says (´alg:registry:axis-registration´), and what the archive
    // restores is this axis's positions inside the models that survived its
    // absence.
    let arriving = crate::model::hibernation::HibernationLayout::OutcomeAxis {
        width: total_new,
        identity_dimensions: working.dimension_map.id_dim_ranges.len(),
        sentinels: n_sentinels,
        spatial: matches!(reg.spatial_features, SpatialFeaturePolicy::Enabled),
    };
    let restored = working
        .hibernation
        .take_axis(axis_id)
        .map(|record| restore_entity_block(working, &record, appended_at, arriving, config, now, "OutcomeAxis"));

    // Create the per-axis model in the working copy so it appears in the
    // published snapshot (required for duplicate detection and assessment).
    let p = working.operational.dim();
    let spatial = matches!(reg.spatial_features, SpatialFeaturePolicy::Enabled);
    let axis_model = crate::snapshot::working::WorkingAxisModel {
        name: reg.name.clone(),
        description: reg.description.clone(),
        model: crate::model::bayesian::BayesianLinearModel::new(p, reg.initial_kappa, config.cholesky.n_recompute),
        kappa_a: reg.initial_kappa,
        gamma: reg.gamma,
        spatial,
        eligibility: reg.eligibility,
    };
    working.outcome_models.insert(axis_id, axis_model);

    tracing::debug!(
        ?axis_id,
        n_sentinels,
        spatial_new,
        identity_axis_new,
        total_new,
        ?restored,
        "Registered outcome axis"
    );

    EventOutcome::Success
}

/// Plans outcome axis deregistration, destructive or hibernating.
///
/// Names the 2 spatial features per Sentinel that were added for this axis (if
/// any) together with its identity triples. Axes registered with `Disabled`
/// spatial features have no per-Sentinel features to remove.
///
/// Both halves are addressed from the entry map by the axis's own identifier:
/// the identity triple at `identity_axis_offset`, the outcome-memory pair at
/// `slot_axis_offset` (´def:extraction:slot´). Neither reads a rank recomputed
/// from the authoritative model map, and that is the whole repair. The rank was
/// the one quantity in this handler that moved as a submission ran — every
/// handler removes its axis from that map as it goes, while the slot geometry
/// the rank indexed into came from a map the rebuild refreshes only after the
/// events — so a second retirement in one submission took a fresh rank into
/// stale positions and addressed coordinates the first had already compacted
/// away (´entry:assayer:wl-owner-dereg-scan´). Reading both halves off the map
/// makes the two halves one layout by construction, and the chain that applies
/// them keeps that layout still until the whole chain has been planned.
///
/// An axis hibernates under the protocol Sentinels hibernate under, with the
/// same preservation scope (´rem:registry:axis-hibernation´). Its own block is
/// the identity triples and, where the spatial policy was on, the
/// outcome-memory pairs — the same positions this removal takes, archived in
/// the order a later registration will append them in rather than in the
/// ascending order the marginalisation needs.
fn plan_deregister_outcome_axis(
    working: &mut WorkingCopy,
    axis_id: OutcomeAxisId,
    outcome_ledger: &OutcomeLedger,
    disposition: Disposition,
    now: &crate::types::PersistentTimestamp,
) -> PlannedEvent {
    // Check spatial status before removing the model.
    let was_spatial = working.outcome_models.get(&axis_id).is_some_and(|m| m.spatial);
    let mut remove_indices: Vec<usize> = Vec::new();

    if let Some(offset) = working.dimension_map.identity_axis_offset(axis_id) {
        for range in working.dimension_map.id_dim_ranges.values() {
            remove_indices.extend((range.start + offset)..(range.start + offset + ID_AXIS_FEATURES_PER_AXIS));
        }
    }

    // Where this axis's outcome-memory pair sits inside every Sentinel slot,
    // taken from the entry map by the axis's own identifier. `None` means the
    // axis owns no pair in this layout, which is what a non-spatial axis and an
    // axis already retired both look like from here.
    let slot_offset = working.dimension_map.slot_axis_offset(axis_id);

    // Remove the per-axis model from the working copy. Order-preserving: the
    // surviving axes' ranks place their identity triples and their
    // outcome-memory pairs, and a swap would silently renumber them.
    working.outcome_models.shift_remove(&axis_id);

    if was_spatial {
        // Drop this axis's per-axis EWMA rows from every Ledger entry, which is
        // step 4 of the per-axis lifecycle protocol
        // (´alg:registry:axis-deregistration´): the averages end with the axis
        // and nothing keeps them for a later return. Done before model
        // marginalisation so a subsequent re-registration of the same ID
        // cannot observe stale EWMAs through `read_decayed`. It is keyed by the
        // axis's identifier and by nothing positional, which is why it stays
        // per event inside a chain where the order of the events is immaterial.
        outcome_ledger.prune_axis(axis_id);

        // Each Sentinel held this axis's outcome-memory pair. Step 1 removes
        // the axis's own features (´alg:registry:axis-deregistration´), so the
        // pair is addressed from the map as the identity half above already
        // is, and by the same kind of key: this axis's own recorded offset
        // rather than a position computed from a count.
        let spatial_features_to_remove = slot_offset.map_or(0, |pair_start| {
            for slot in working.dimension_map.sentinel_slots.values() {
                let start = slot.start + pair_start;
                let end = (start + SENTINEL_SPATIAL_FEATURES_PER_AXIS).min(slot.end);
                remove_indices.extend(start..end);
            }
            working.dimension_map.sentinel_slots.len() * SENTINEL_SPATIAL_FEATURES_PER_AXIS
        });
        tracing::debug!(
            ?axis_id,
            ?slot_offset,
            spatial_features_to_remove,
            "Deregistered spatial outcome axis"
        );
    } else {
        tracing::debug!(
            ?axis_id,
            identity_features = remove_indices.len(),
            "Deregistered non-spatial outcome axis"
        );
    }
    // Taken before the sort, because the append order is what a later
    // registration lays these positions down in: the identity triples in
    // registration order, then the per-Sentinel pairs
    // (´alg:registry:axis-registration´). The ascending order the
    // marginalisation needs is a different order, and writing a block back
    // under it would transpose the two groups.
    if disposition == Disposition::Hibernating && !remove_indices.is_empty() {
        let ordered_positions = remove_indices.clone();
        let layout = crate::model::hibernation::HibernationLayout::OutcomeAxis {
            width: ordered_positions.len(),
            identity_dimensions: working.dimension_map.id_dim_ranges.len(),
            sentinels: working.dimension_map.sentinel_slots.len(),
            spatial: was_spatial,
        };
        let record = working.archive_self_structure(&ordered_positions, layout, now);
        working.hibernation.store_axis(axis_id, record);
        tracing::debug!(
            ?axis_id,
            width = ordered_positions.len(),
            was_spatial,
            "Outcome axis block archived for hibernation"
        );
    }

    remove_indices.sort_unstable();
    remove_indices.dedup();

    tracing::debug!(?axis_id, planned = remove_indices.len(), "Deregistered outcome axis");

    PlannedEvent::removing(remove_indices)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Dimension Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles identity dimension registration.
///
/// - Extends models by one per-dimension identity block, `8 + 3m`
///   (´tab:keyspace:dimension-features´)
/// - Adds the cross-dimension aggregate block when this is the first dimension
/// - Creates `IdentityConvergenceTracker`
fn handle_register_identity_dimension(
    working: &mut WorkingCopy,
    dim_id: DimensionId,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
    config: &AssayerConfig,
    now: &crate::types::PersistentTimestamp,
) -> EventOutcome {
    let per_dimension_features = ID_DIM_BASE_FEATURE_COUNT + working.outcome_models.len() * ID_AXIS_FEATURES_PER_AXIS;
    let cross_dimension_features = if working.dimension_map.id_dim_ranges.is_empty() {
        ID_CROSS_DIM_FEATURE_COUNT
    } else {
        0
    };

    let total_new = per_dimension_features + cross_dimension_features;

    if total_new > 0 {
        // Extend every full-dimension model (´thm:gaussian:extension´).
        working.extend_all_full_dim(total_new, config.model.lambda_prior);

        let mut new_classes = vec![
            FeatureClass::RateOrFraction,
            FeatureClass::RateOrFraction,
            FeatureClass::LogScaled,
            FeatureClass::IdentityMeasurementAlarm,
            FeatureClass::IdentityMeasurementAlarm,
            FeatureClass::IdentityMeasurementSuspicion,
            FeatureClass::IdentityMeasurementVolatility,
            FeatureClass::Binary,
        ];
        for _ in 0..working.outcome_models.len() {
            new_classes.extend_from_slice(&[
                FeatureClass::OutcomeAxisCompressed,
                FeatureClass::OutcomeAxisRaw,
                FeatureClass::OutcomeAxisStability,
            ]);
        }
        if cross_dimension_features > 0 {
            // TODO ´todo:code:cross-dimension-offsets-3-and-7´: cross-dimension offsets 3 and 7 are
            // RateOrFraction under the class-assignment requirement
            // (´req:standardisation:class-assignment´).
            new_classes.extend_from_slice(&[
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionMaxAggregate,
                FeatureClass::CrossDimensionBinary,
                FeatureClass::CrossDimensionMaxAggregate,
            ]);
        }
        extend_standardisation(
            &mut working.feature_means,
            &mut working.feature_variances,
            &mut working.feature_classes,
            &new_classes,
            working.operational.dim(),
        );
    }

    // Create identity convergence tracker, first seen on the injected clock.
    identity_trackers.insert(dim_id, IdentityConvergenceTracker::new(*now));

    tracing::debug!(
        ?dim_id,
        per_dimension_features,
        cross_dimension_features,
        total_new,
        "Registered identity dimension"
    );

    EventOutcome::Success
}

/// Plans identity dimension deregistration.
///
/// Removes the identity tracker and names the dimension's features from the
/// entry `DimensionMap` index ranges.
///
/// The cross-dimension aggregate leaves with the last dimension, and "last" is
/// a count of what is still standing rather than a property of what is
/// departing. That is the one reading in the removal family that is not a
/// function of the departing entity alone, and it is why two of these
/// operations never share a chain: with the chain sealed between them, the
/// second reads a count the first has already reduced, exactly as two
/// single-event submissions would leave it.
fn plan_deregister_identity_dimension(
    working: &WorkingCopy,
    dim_id: DimensionId,
    identity_trackers: &mut std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
) -> PlannedEvent {
    // Remove tracker
    identity_trackers.remove(&dim_id);

    // Collect indices to remove from the DimensionMap.
    let mut remove_indices: Vec<usize> = Vec::new();

    // Per-dimension identity features for this dimension
    // (´tab:keyspace:dimension-features´).
    if let Some(range) = working.dimension_map.id_dim_ranges.get(&dim_id) {
        remove_indices.extend(range.to_range());
    }

    // Cross-dimension aggregate features are removed with the last dimension.
    if working.dimension_map.id_dim_ranges.len() == 1
        && let Some(ref range) = working.dimension_map.id_cross_dim_range
    {
        remove_indices.extend(range.to_range());
    }

    // Competitive cell indicators for this dimension.
    if let Some(cells) = working.dimension_map.competitive_indices.get(&dim_id) {
        remove_indices.extend(cells.values().copied());
    }

    // Type-5 interaction features for competitive cells in this dimension.
    for (id, &idx) in &working.dimension_map.interaction_indices {
        if matches!(id, InteractionId::PerCell { dimension, .. } if *dimension == dim_id) {
            remove_indices.push(idx);
        }
    }

    remove_indices.sort_unstable();
    remove_indices.dedup();

    tracing::debug!(?dim_id, planned = remove_indices.len(), "Deregistered identity dimension");

    PlannedEvent::removing(remove_indices)
}

// ═══════════════════════════════════════════════════════════════════════════════
// Competitive Cell Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// Handles competitive cell entry.
///
/// - Extends models by 1 (indicator) + Type-5 interactions (from templates)
fn handle_competitive_cell_entry(working: &mut WorkingCopy, dimension: DimensionId, cell: CompetitiveCellId) -> EventOutcome {
    // If the identity dimension is no longer registered, drop the
    // event silently. The identity-maintenance loop can emit
    // `CompetitiveCellEntry` events that race with deregistration:
    // the dimension is removed from the tracker set before the queued
    // "competitive set changed" signal reaches the model-owner
    // thread. Extending the model here would drift `model.p()` above
    // `DimensionMap.p` (`rebuild_dimension_map` filters competitive cells
    // against the authoritative identity-dimension set, so cells for
    // a removed dimension are discarded).
    //
    // This mirrors the symmetric guard in `handle_competitive_cell_exit`.
    if !working.dimension_map.id_dim_ranges.contains_key(&dimension) {
        tracing::warn!(
            ?dimension,
            ?cell,
            "Cell entry: dimension not registered, dropping (likely racing with deregister)",
        );
        return EventOutcome::Success;
    }

    // Cell indicator: 1 feature
    let indicator_features = 1;

    // Type-5 interaction features: 1 per Type-5 template
    let type5_count: usize = working
        .dimension_map
        .interaction_templates
        .iter()
        .filter(|t| matches!(t, InteractionTemplate::Type5 { .. }))
        .count();

    let total_new = indicator_features + type5_count;

    if total_new > 0 {
        let prior_precision = 0.1;
        // Extend every full-dimension model (´thm:gaussian:extension´).
        working.extend_all_full_dim(total_new, prior_precision);

        // Extend standardisation in the order the rebuild will lay the two
        // blocks out: the interaction block precedes the competitive
        // indicators (´def:feature:vector-structure´), so the appended
        // classes must arrive that way round for the rearrangement to pair
        // each fresh coordinate with its own class.
        let new_classes: Vec<FeatureClass> = std::iter::repeat_n(FeatureClass::InteractionProduct, type5_count)
            .chain(std::iter::once(FeatureClass::CompetitiveCellIndicator))
            .collect();

        extend_standardisation(
            &mut working.feature_means,
            &mut working.feature_variances,
            &mut working.feature_classes,
            &new_classes,
            working.operational.dim(),
        );
    }

    tracing::debug!(?dimension, ?cell, total_new, "Competitive cell entry");

    EventOutcome::Success
}

/// Plans competitive cell exit.
///
/// - Names the cell's indicator feature
/// - Also names the Type-5 interaction features for this cell
fn plan_competitive_cell_exit(working: &WorkingCopy, dimension: DimensionId, cell: CompetitiveCellId) -> PlannedEvent {
    // Find cell indicator index in DimensionMap.
    let mut remove_indices: Vec<usize> = Vec::new();

    if let Some(idx) = working
        .dimension_map
        .competitive_indices
        .get(&dimension)
        .and_then(|cells| cells.get(&cell).copied())
    {
        remove_indices.push(idx);
    } else {
        tracing::warn!(?dimension, ?cell, "Cell exit: cell not found in DimensionMap");
        return PlannedEvent::applied(EventOutcome::Success);
    }

    // Also remove Type-5 interaction features for this cell.
    for (id, &idx) in &working.dimension_map.interaction_indices {
        if matches!(id, InteractionId::PerCell { dimension: d, cell: c, .. } if *d == dimension && *c == cell) {
            remove_indices.push(idx);
        }
    }

    remove_indices.sort_unstable();
    remove_indices.dedup();

    // TODO(2026-05-23) ´todo:code:route-low-rank´: Route low-rank
    // competitive-cell exits through a Σ_kk extraction path when the
    // removal block is `1 + T5`, falling back to full Cholesky only when
    // the low-rank path is not applicable — the correction's own decision
    // (´dec:posterior:half-solve´) over the chain this exit is applied in
    // (´dec:construction:compound-batch´).
    tracing::debug!(?dimension, ?cell, planned = remove_indices.len(), "Competitive cell exit");

    PlannedEvent::removing(remove_indices)
}

// ═══════════════════════════════════════════════════════════════════════════════
// DimensionMap Rebuild
// ═══════════════════════════════════════════════════════════════════════════════

/// Rebuilds the `DimensionMap` from authoritative entity state after one
/// order-free chain has been applied.
///
/// Constructs the complete entity configuration from the old
/// `DimensionMap` (with deltas from the chain's events applied) and the identity
/// trackers, then calls `DimensionMap::rebuild()` for a full
/// structural recomputation.
///
/// The deltas are the chain's own events rather than the whole submission's:
/// the map this rebuild produces is the layout the chain's snapshot publishes,
/// and it is the entry layout the next chain plans against.
///
/// The model dimension `p` has already been updated by the event
/// handlers via `extend()` / `marginalise()`. The rebuilt `DimensionMap`
/// must produce the same `p` — a mismatch indicates a bug in the
/// incremental feature-count logic.
///
/// # Arguments
///
/// * `working` — The working copy (owns stale `DimensionMap` and `outcome_models`)
/// * `events` — The chain's lifecycle events, which were just applied
/// * `identity_trackers` — Authoritative set of registered identity dimensions
fn rebuild_dimension_map(
    working: &mut WorkingCopy,
    events: &[LifecycleEvent],
    identity_trackers: &std::collections::HashMap<DimensionId, IdentityConvergenceTracker>,
) {
    let old_p = working.dimension_map.p;
    let model_p = working.operational.dim();

    // Step 1: signal features — unchanged by lifecycle events.
    let p_sig = working.dimension_map.sig_range.len();

    // Step 2: Sentinels — apply registration/deregistration deltas.
    let mut sentinels: IndexMap<SentinelId, ()> = working.dimension_map.sentinel_slots.keys().map(|&id| (id, ())).collect();
    for event in events {
        match event {
            LifecycleEvent::RegisterSentinel(reg) => {
                sentinels.insert(reg.id, ());
            }
            LifecycleEvent::DeregisterSentinel(id) | LifecycleEvent::HibernateSentinel(id) => {
                sentinels.swap_remove(id);
            }
            _ => {}
        }
    }

    // Step 3: identity dimensions — from authoritative tracker state.
    let identity_dimensions: Vec<DimensionId> = identity_trackers.keys().copied().collect();

    // Step 4: competitive cells — apply deltas from events.
    let mut competitive_cells: IndexMap<DimensionId, Vec<CompetitiveCellId>> = IndexMap::new();
    for (&dim_id, cells) in &working.dimension_map.competitive_indices {
        competitive_cells.insert(dim_id, cells.keys().copied().collect());
    }
    for event in events {
        match event {
            LifecycleEvent::CompetitiveCellEntry { dimension, cell } => {
                competitive_cells.entry(*dimension).or_default().push(*cell);
            }
            LifecycleEvent::CompetitiveCellExit { dimension, cell } => {
                if let Some(cells) = competitive_cells.get_mut(dimension) {
                    cells.retain(|c| c != cell);
                }
            }
            LifecycleEvent::DeregisterIdentityDimension(dim_id) => {
                competitive_cells.swap_remove(dim_id);
            }
            _ => {}
        }
    }

    // Step 5: outcome axis identities — from authoritative outcome models. The
    // spatial subset is taken as an ordered list of identities rather than as a
    // bare count, because a slot's outcome-memory pairs are addressed by which
    // axis owns each of them and a count cannot say that. Reading the subset off
    // the same insertion-ordered map the assessment path reads is what keeps the
    // tail this rebuild lays out and the tail an extraction fills in one order.
    let outcome_axis_ids: Vec<OutcomeAxisId> = working.outcome_models.keys().copied().collect();
    let spatial_axis_ids: Vec<OutcomeAxisId> = working
        .outcome_models
        .iter()
        .filter_map(|(&axis_id, model)| model.spatial.then_some(axis_id))
        .collect();
    let spatial_axis_count = spatial_axis_ids.len();

    // Step 6: interaction templates — unchanged by lifecycle events.
    let templates = working.dimension_map.interaction_templates.clone();

    // Step 7: full rebuild.
    let new_map = DimensionMap::rebuild_with_axes(
        p_sig,
        &sentinels,
        &identity_dimensions,
        &competitive_cells,
        &spatial_axis_ids,
        &outcome_axis_ids,
        templates,
    );

    if old_p != model_p {
        tracing::debug!(
            old_p,
            model_p,
            rebuilt_p = new_map.p,
            sentinels = sentinels.len(),
            identity = identity_dimensions.len(),
            outcome_axis_count = outcome_axis_ids.len(),
            spatial_axis_count,
            "DimensionMap rebuilt"
        );
    }

    // Rearrange the parameters into the order just computed.
    //
    // Extension appends at the end and the rebuild lays the blocks out
    // canonically, so the two agree only when registrations happen to
    // arrive in canonical order (´entry:assayer:wl-owner-layout-misalignment´).
    // The permutation carries every learned weight and every
    // standardisation entry from the position the append left it at to the
    // position this map now names, which is what keeps the map, the
    // parameters and the statistics on one index semantics
    // (´inv:dimension:version-consistency´). It is the identity whenever the
    // arriving order was already canonical, and is skipped then.
    //
    // A refusal here is the strengthened post-condition firing: the two
    // layouts do not describe the same vector, which the width comparison
    // below cannot see. It leaves the parameters exactly where the handlers
    // left them — the behaviour that stood before this repair — and reports,
    // because a refusal that panicked would take a whole submission's
    // completion with it.
    match LayoutPermutation::from_extension(&working.dimension_map, &new_map) {
        Some(permutation) => working.permute_all_full_dim(&permutation),
        None => {
            tracing::warn!(
                old_p,
                model_p,
                rebuilt_p = new_map.p,
                "Layout permutation refused: the post-event vector and the rebuilt map disagree block for block"
            );
        }
    }

    // Step 8: store the rebuilt map. The model dimension (model_p) was set
    // by the event handlers; the DimensionMap provides the structural
    // layout for subsequent `assess()` and `label()` calls.
    //
    // In single-event submissions (the public API path) these always
    // agree. Compound batches processed in the same submission may
    // diverge when inter-event dependencies shift slot widths; the
    // debug_assert catches this during development.
    debug_assert_eq!(
        new_map.p, model_p,
        "DimensionMap.p ({}) must match model dimension ({}) after lifecycle events",
        new_map.p, model_p,
    );
    debug_assert_eq!(
        new_map.p,
        working.sister.dim(),
        "DimensionMap.p ({}) must match sister model dimension ({})",
        new_map.p,
        working.sister.dim(),
    );
    for (id, axis) in &working.outcome_models {
        debug_assert_eq!(
            new_map.p,
            axis.model.dim(),
            "DimensionMap.p ({}) must match outcome axis {:?} dimension ({}) after lifecycle events",
            new_map.p,
            id,
            axis.model.dim(),
        );
    }

    working.dimension_map = new_map;
}

// ═══════════════════════════════════════════════════════════════════════════════
// Tests
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::feature::dimension_map::IndexRange;
    use crate::snapshot::working::ModelConfig;

    fn make_test_working() -> WorkingCopy {
        // Width 16 keeps the model dimension aligned with the DimensionMap
        // (and so with the standardisation vectors) from construction.
        WorkingCopy::cold_start(16, &ModelConfig::default(), 1000)
    }

    fn make_test_config() -> AssayerConfig {
        AssayerConfig::default()
    }

    /// The gathered chains as half-open `(start, end)` pairs, which is the
    /// shape an assertion can read at a glance.
    fn chain_bounds(events: &[LifecycleEvent]) -> Vec<(usize, usize)> {
        gather_chains(events).into_iter().map(|c| (c.start, c.end)).collect()
    }

    /// A Sentinel joining the system costs a fixed, computable amount of model: one
    /// occupancy indicator plus the base extraction block, added to the operational
    /// model and to the standardisation vectors alike. With no spatial axes and no
    /// interaction templates in play the growth is exactly that slot width, so the
    /// dimension a deployment ends up with is predictable from the registrations it
    /// made rather than discovered afterwards.
    ///
    /// ´claim:lifespan:registering-a-sentinel-grows-the-model-and-its-standardisation-by-one-slot-width´
    /// ´test:unit:register-sentinel-extends-model´
    #[test]
    fn register_sentinel_extends_model() {
        let mut working = make_test_working();
        let mut platt = PlattConvergenceTracker::new();
        let outcome_ledger = OutcomeLedger::new();
        let config = make_test_config();

        let initial_p = working.operational.dim();
        let initial_stats_len = working.feature_means.len();

        let outcome = handle_register_sentinel(
            &mut working,
            SentinelId(1),
            &mut platt,
            &outcome_ledger,
            &config,
            &crate::types::PersistentTimestamp::new(0, 0),
        );

        assert_eq!(outcome, EventOutcome::Success);
        // slot_width = 1 + 60 = 61 (no spatial axes, no interaction templates)
        let expected_new = SENTINEL_OCCUPANCY_WIDTH + SENTINEL_Q_BASE;
        assert_eq!(working.operational.dim(), initial_p + expected_new);
        assert_eq!(working.feature_means.len(), initial_stats_len + expected_new);
    }

    /// Registration does not merely widen the model; it also tells the probability
    /// calibrator that its fit is now stale, the early-refit flag being clear
    /// beforehand and set afterwards. A new Sentinel's features enter carrying
    /// prior coefficients, so leaving the calibrator on its ordinary cadence would
    /// let the mapping from score to probability drift for as long as that cadence
    /// allows.
    ///
    /// ´claim:lifespan:registering-a-sentinel-marks-the-calibrator-for-an-early-refit´
    /// ´test:unit:register-sentinel-triggers-early-refit´
    #[test]
    fn register_sentinel_triggers_early_refit() {
        let mut working = make_test_working();
        let mut platt = PlattConvergenceTracker::new();
        let outcome_ledger = OutcomeLedger::new();
        let config = make_test_config();

        assert!(!platt.early_refit_pending);

        handle_register_sentinel(
            &mut working,
            SentinelId(1),
            &mut platt,
            &outcome_ledger,
            &config,
            &crate::types::PersistentTimestamp::new(0, 0),
        );

        assert!(platt.early_refit_pending);
    }

    /// A cell entering the competitive set of a live identity dimension buys
    /// exactly one feature — its own indicator — when no Type-5 interaction
    /// templates are configured. Cell membership turns over far faster than
    /// registration does, so the per-cell cost is the smallest thing that can
    /// represent the cell at all.
    ///
    /// ´claim:lifespan:a-competitive-cell-entering-adds-its-indicator-and-nothing-more-when-no-interaction-templates-exist´
    /// ´test:unit:competitive-cell-entry-extends-model´
    #[test]
    fn competitive_cell_entry_extends_model() {
        let mut working = make_test_working();

        // Mark the identity dimension as live so the entry handler's
        // registration guard (added symmetric to the exit guard)
        // accepts the event. In production this entry is established
        // by `handle_register_identity_dimension` + rebuild; here we
        // inject it directly to keep this unit test focused on the
        // cell-entry extension contract.
        working
            .dimension_map
            .id_dim_ranges
            .insert(DimensionId(1), IndexRange::new(0, ID_DIM_BASE_FEATURE_COUNT));

        let initial_p = working.operational.dim();
        let cell = CompetitiveCellId::new(0, 8);

        let outcome = handle_competitive_cell_entry(&mut working, DimensionId(1), cell);

        assert_eq!(outcome, EventOutcome::Success);
        // 1 indicator + 0 Type-5 interactions (no templates in cold start)
        assert_eq!(working.operational.dim(), initial_p + 1);
    }

    /// Registering an identity dimension creates the convergence tracker that will
    /// watch it and widens the model by both the dimension's own feature block and
    /// the cross-dimension aggregate block. The aggregate summarises across
    /// dimensions, so it comes into existence with the first dimension rather than
    /// waiting for a second one that may never arrive.
    ///
    /// ´claim:lifespan:the-first-identity-dimension-brings-its-own-block-and-the-cross-dimension-aggregate-block-with-it´
    /// ´test:unit:register-identity-dimension-creates-tracker´
    #[test]
    fn register_identity_dimension_creates_tracker() {
        let mut working = make_test_working();
        let initial_p = working.operational.dim();
        let mut trackers = std::collections::HashMap::new();
        let config = make_test_config();

        assert!(!trackers.contains_key(&DimensionId(1)));

        handle_register_identity_dimension(
            &mut working,
            DimensionId(1),
            &mut trackers,
            &config,
            &crate::types::PersistentTimestamp::new(0, 0),
        );

        assert!(trackers.contains_key(&DimensionId(1)));
        // 8 per-dimension features + 8 cross-dimension features for the first dimension.
        assert_eq!(
            working.operational.dim(),
            initial_p + ID_DIM_BASE_FEATURE_COUNT + ID_CROSS_DIM_FEATURE_COUNT
        );
    }

    /// Once one identity dimension exists, the next one costs only its own per-
    /// dimension block: the cross-dimension aggregate is built once and then
    /// shared. Charging for it twice would both inflate the dimension and leave two
    /// aggregate blocks competing to describe the same thing.
    ///
    /// ´claim:lifespan:a-later-identity-dimension-pays-only-for-its-own-block-because-the-aggregate-block-already-stands´
    /// ´test:unit:register-identity-second-dimension-extends-per-dimension-only´
    #[test]
    fn register_identity_second_dimension_extends_per_dimension_only() {
        let mut working = make_test_working();
        let initial_p = working.operational.dim();
        let mut trackers = std::collections::HashMap::new();
        let config = make_test_config();

        working
            .dimension_map
            .id_dim_ranges
            .insert(DimensionId(1), IndexRange::new(16, 16 + ID_DIM_BASE_FEATURE_COUNT));
        working.dimension_map.id_cross_dim_range = Some(IndexRange::new(24, 24 + ID_CROSS_DIM_FEATURE_COUNT));

        handle_register_identity_dimension(
            &mut working,
            DimensionId(2),
            &mut trackers,
            &config,
            &crate::types::PersistentTimestamp::new(0, 0),
        );

        assert!(trackers.contains_key(&DimensionId(2)));
        assert_eq!(working.operational.dim(), initial_p + ID_DIM_BASE_FEATURE_COUNT);
    }

    /// The convergence tracker for a dimension does not outlive the dimension. That
    /// tracker set is the authoritative list the dimension-map rebuild reads, so a
    /// tracker left standing would keep resurrecting features for a dimension
    /// nothing feeds any more.
    ///
    /// ´claim:lifespan:deregistering-an-identity-dimension-takes-its-convergence-tracker-with-it´
    /// ´test:unit:deregister-identity-dimension-removes-tracker´
    #[test]
    fn deregister_identity_dimension_removes_tracker() {
        let working = make_test_working();
        let mut trackers = std::collections::HashMap::new();
        trackers.insert(
            DimensionId(1),
            IdentityConvergenceTracker::new(crate::types::PersistentTimestamp::new(0, 0)),
        );

        assert!(trackers.contains_key(&DimensionId(1)));

        plan_deregister_identity_dimension(&working, DimensionId(1), &mut trackers);

        assert!(!trackers.contains_key(&DimensionId(1)));
    }

    /// Consecutive removals gather into one chain however many of them arrive
    /// and whatever they remove: a Sentinel retirement, an axis retirement and
    /// a competitive-cell exit all address their positions by their own
    /// entity's identifier in the layout the chain began with, so none of them
    /// can observe whether another has run. One chain is one plan, and one plan
    /// is what leaves no intermediate layout for a later read to be stale
    /// against.
    ///
    /// ´claim:lifespan:consecutive-removals-of-different-entities-gather-into-a-single-order-free-chain´
    /// ´test:unit:consecutive-removals-gather-into-one-chain´
    #[test]
    fn consecutive_removals_gather_into_one_chain() {
        let events = vec![
            LifecycleEvent::DeregisterSentinel(SentinelId(1)),
            LifecycleEvent::DeregisterOutcomeAxis(OutcomeAxisId(7)),
            LifecycleEvent::CompetitiveCellExit {
                dimension: DimensionId(1),
                cell: CompetitiveCellId::new(0, 8),
            },
            LifecycleEvent::HibernateSentinel(SentinelId(2)),
        ];

        assert_eq!(chain_bounds(&events), [(0, 4)]);
    }

    /// A registration is order-free with nothing, itself included, so it seals
    /// whatever chain is open and becomes a chain of one. How wide a
    /// registration is depends on live entity state and where its coordinates
    /// land depends on the append, which is to say append order is rank order
    /// is layout — two registrations reordered are two different layouts, and a
    /// registration beside a removal is a different width. Sealing costs a
    /// publication and buys the guarantee that nothing in a chain was decided
    /// against a layout another member of it had already moved.
    ///
    /// ´claim:lifespan:a-registration-seals-the-open-chain-and-forms-a-chain-of-one´
    /// ´test:unit:a-registration-seals-the-open-chain´
    #[test]
    fn a_registration_seals_the_open_chain() {
        let events = vec![
            LifecycleEvent::DeregisterSentinel(SentinelId(1)),
            LifecycleEvent::DeregisterSentinel(SentinelId(2)),
            LifecycleEvent::RegisterSentinel(crate::testing::registrations::sentinel_reg(SentinelId(3))),
            LifecycleEvent::RegisterSentinel(crate::testing::registrations::sentinel_reg(SentinelId(4))),
            LifecycleEvent::DeregisterSentinel(SentinelId(5)),
        ];

        assert_eq!(chain_bounds(&events), [(0, 2), (2, 3), (3, 4), (4, 5)]);
    }

    /// Two identity-dimension removals never share a chain, and the removal
    /// family's one exception is where the reason lies: the cross-dimension
    /// aggregate block leaves with the *last* dimension, and "last" counts the
    /// dimensions still standing rather than describing the one departing. Two
    /// of them planned against one entry layout would each see the other still
    /// there, neither would take the aggregate, and it would outlive everything
    /// it aggregates. Sealed apart, the second reads a count the first has
    /// already reduced. Each still chains freely with removals of other kinds,
    /// whose index sets are functions of their own entities alone.
    ///
    /// ´claim:lifespan:two-identity-dimension-removals-are-never-gathered-into-one-chain´
    /// ´test:unit:two-identity-dimension-removals-do-not-share-a-chain´
    #[test]
    fn two_identity_dimension_removals_do_not_share_a_chain() {
        let events = vec![
            LifecycleEvent::DeregisterIdentityDimension(DimensionId(1)),
            LifecycleEvent::DeregisterSentinel(SentinelId(1)),
            LifecycleEvent::DeregisterIdentityDimension(DimensionId(2)),
        ];

        assert_eq!(chain_bounds(&events), [(0, 2), (2, 3)]);
    }

    /// An empty submission gathers one empty chain, so it still publishes the
    /// version it was given. A caller waiting on that version is waiting for a
    /// statement about the state, and the state is the one the submission left.
    ///
    /// ´claim:lifespan:an-empty-submission-gathers-one-empty-chain-and-still-publishes´
    /// ´test:unit:an-empty-submission-gathers-one-empty-chain´
    #[test]
    fn an_empty_submission_gathers_one_empty_chain() {
        let events: Vec<LifecycleEvent> = Vec::new();
        assert_eq!(chain_bounds(&events), [(0, 0)]);
    }
}
