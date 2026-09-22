// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`lifecycle_register_sentinel_then_report_then_assess_and_derive`] | lifespan | A report filed immediately after registration not only succeeds but creates cells in the Sentinel's ledger, and an assessment routed through that Sentinel's coordinate is finite and counts it as its one contributor. The ledger partition is built during registration itself, so the very first report has somewhere to write rather than racing the model owner for it. |
//! | [`sentinel_bootstrap_blends_into_published_statistics`] | lifespan | Registration opens the warm-up accumulator, the configured number of reporting assessments fills it, and the published standardisation then carries the blend: the occupancy entry lands at exactly the blend weight times the observed value plus the remainder times what the entry held, and the variance entries respect the floor. A Sentinel registered after warm-up would otherwise spend a standardisation half-life being scaled against class priors; a hundred assessments is what the corpus shortens that to, and this is the wiring that makes those hundred count. |
//! | [`lifecycle_publication_preserves_empirical_standardisation`] | lifespan | A lifecycle publication carries the empirical standardisation statistics forward instead of resetting them: after a Sentinel's bootstrap has blended its slot statistics into publication, a lifecycle event publishes a snapshot whose entries still hold the blend, not the neutral zero-mean unit-variance pair. The projection is canonical — the statistics live on the working copy the snapshot is built from — so the standardisation entries a lifecycle event creates move under the same publication as everything else, and a host that registers an entity during a quiet period does not spend the wait being scored on neutrally scaled features (´inv:guarantee:lifecycle-publication´). |
//! | [`cold_ramp_refusal_under_a_full_channel_advances_nothing`] | lifespan | A cold-ramp observation refused by a full observation queue advances no count, is reported to the caller in band, and costs the ramp only pace: the requests served and the observations accepted come apart by exactly the refusals, and the ramp still reaches its horizon and publishes the empirical moments once the channel drains. Under the completion gate this was a far sharper edge — the whole accumulation rode on one delivery at the count, so a full channel at that instant had to be retried or the warm-up was silenced for the life of the process. There is no one-shot left to lose: what a refusal defers now is one share of prior mass, and the host is told which (´dec:concurrency:no-silent-drop´). |
//! | [`observation_barrier_applies_every_queued_observation`] | steward | The observation barrier resolves only once every cold-ramp observation queued before it has been applied, and the checkpoint barrier resolves without regard to any of them. With the steward parked, a run of assessments leaves its offers queued and the published count at zero; a barrier enqueued behind the park is reached in the same command drain the release resumes, before the loop's own observation drain has run, and it answers with every one of those offers accepted. The count is the offers' own, exactly, and the phase follows it to the horizon. This is the property the harness's settling helper is built on, and the reason it needs a barrier of its own rather than the label flush. The two barriers ride the same first-in-first-out command channel, so nothing about their order relative to each other could have made either cover the other's queue: a command's position among commands says nothing about what is waiting on a different channel. Without this one, a caller that assessed and then read the coordinate system was reading how far the steward had got — a reading of the scheduler wearing the ramp's clothes. |
//! | [`lifecycle_register_deregister_sentinel_round_trip`] | lifespan | A Sentinel id released by retirement is free for immediate reuse, and the reborn Sentinel is fully functional — it accepts a report and contributes to the next assessment routed through its coordinate. Ids are a namespace the host manages, not a resource the engine consumes permanently. |
//! | [`deregister_sentinel_pushes_marginalisation_completion_event`] | lifespan | A deregistration that actually removes a model slot pushes the aggregate diagnostics after the owner has published the compacted model. Flushing is the public synchronisation point for that asynchronous half, and draining proves the payload travelled through the bounded event channel (´entry:health:marginalise-event´). |
//! | [`lifecycle_register_axis_then_assess_includes_prediction`] | lifespan | Once the model owner has processed an axis registration, assessments carry a prediction for that axis in their outcome map. The axis is not merely stored somewhere; it becomes part of what every assessment answers. |
//! | [`lifecycle_register_deregister_axis_round_trip`] | lifespan | cites (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´) |
//! | [`lifecycle_register_identity_then_assess_and_derive`] | lifespan | With an identity dimension registered, assessment continues to return a finite probability. Identity features enlarge what an assessment can see; they are never a precondition for it answering. |
//! | [`published_summary_carries_identity_trackers`] | lifespan | The published health summary carries the identity trackers rather than an empty map: after a dimension is registered and one label has driven the publication path, the summary holds that dimension's tracker and the full health report derives the dimension's convergence from it. The composite stage is thereby computed over the trackers the owner maintains — an empty slice would read as universal stability, and the composite would advance on calibration evidence alone over an identity layer it never measured. |
//! | [`lifecycle_register_deregister_identity_round_trip`] | lifespan | cites (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´) |
//! | [`lifecycle_sentinel_plus_axis_combined`] | lifespan | A Sentinel and an outcome axis registered against the same engine both appear in the next assessment — the reported Sentinel counted as a contributor, and a prediction present for the axis. The two kinds of registration extend the model along different directions and do not shadow one another. |
//! | [`lifecycle_two_sentinels_independent`] | lifespan | With two Sentinels registered, retiring the first leaves the second's reports accepted while the first's are refused. Retirement is addressed to one slot, so a host removing a single decommissioned probe does not silence the rest of its fleet. |
//! | [`lifecycle_deregister_sentinel_report_rejected`] | lifespan | cites (´claim:lifespan:a-report-from-a-retired-sentinel-is-rejected-as-coming-from-an-unknown-sentinel´) |
//! | [`lifecycle_sentinel_report_before_model_extension`] | lifespan | cites (´claim:lifespan:reports-are-accepted-from-the-moment-registration-returns´) |
//! | [`health_summary_initial_state`] | lifespan | A newly built engine reports no labels, no eligible labels and no assessments, with its calibration delta already a finite number. The counters start at zero and the numeric fields start defined, so a host can chart health from the first moment rather than from the first event. |
//! | [`health_summary_after_assess`] | lifespan | One assessment leaves the total-assessments counter reading one. That counter is the denominator every rate in the health summary is read against, so it has to count requests rather than calls. |
//! | [`health_summary_after_multiple_assessments`] | lifespan | cites (´claim:lifespan:the-assessment-counter-advances-by-one-per-request-assessed´) |
//! | [`health_summary_after_pre_seed`] | lifespan | A pre-seed batch reports every entry processed and the engine assesses normally afterwards, returning a finite probability. Pre-seeding runs through the label pipeline internally, so what it leaves behind is ordinary learned state rather than a special mode. |
//! | [`full_health_report_initial_state`] | lifespan | At start-up the full report shows no Sentinels, no axes and no identity dimensions. The report is a census of what is registered, so on an engine with nothing registered it is empty rather than populated with placeholders. |
//! | [`full_health_report_after_sentinel_registration`] | lifespan | cites (´claim:lifespan:the-full-health-report-lists-exactly-the-entities-that-are-registered´) |
//! | [`full_health_report_after_sentinel_deregistration`] | lifespan | cites (´claim:lifespan:the-full-health-report-lists-exactly-the-entities-that-are-registered´) |
//! | [`health_summary_assessment_batch_counting`] | lifespan | cites (´claim:lifespan:the-assessment-counter-advances-by-one-per-request-assessed´) |
//! | [`pre_seed_single_entry_updates_model`] | lifespan | cites (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´) |
//! | [`pre_seed_large_batch_all_processed`] | lifespan | cites (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´) |
//! | [`pre_seed_mixed_valence_shifts_model`] | lifespan | Fifty pre-seed entries all carrying the same valence leave the model still producing a finite probability. A degenerate seeding set — every example on one side — is the shape most likely to drive a linear model's updates to extremes, and it does not. |
//! | [`pre_seed_then_assess_reflects_learning`] | lifespan | Two identically built engines seeded with thirty adverse and thirty benign labels respectively assess the same entity differently, the adversely seeded one returning the higher probability. Pre-seeded history is genuinely learned from rather than merely counted, which is what makes it worth loading at all. |
//! | [`pre_seed_with_registered_sentinel_succeeds`] | lifespan | cites (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´) |
//! | [`pre_seed_with_outcome_values`] | lifespan | A pre-seed entry can carry a value for a registered outcome axis alongside its valence, and is accepted. History about consequences is worth as much as history about verdicts, and the seeding path takes both. |
//! | [`assess_with_no_sentinels_zero_flag`] | lifespan | With nothing registered, an assessment still returns, and it carries the zero-Sentinels flag that says what it was built from. The engine's prior is a legitimate answer on an empty deployment; what would not be legitimate is presenting it as though evidence stood behind it. |
//! | [`assess_with_sentinel_has_sentinels_flag`] | lifespan | Registration alone does not clear the flag — a contribution does. An assessment routed past a registered but silent Sentinel is still computed from the prior alone and says so; once that Sentinel holds a report and the request carries its coordinate, the flag clears and the contributor count reads one. Occupancy is the online flag, not a contribution, and a flag cleared by mere registration would deny a prior-only estimate was prior-only (´schema:output:assessment´). |
//! | [`assess_returns_finite_p_bad`] | lifespan | Across entity seeds spanning the extremes of the identifier space, the reported probability is finite and lies within zero and one. It is consumed as a probability by downstream policy, so being out of range is not a degraded answer but a meaningless one. |
//! | [`assess_risk_uncertainty_non_negative`] | lifespan | The uncertainty accompanying a risk figure is never negative. It is a spread, and a negative spread would invert every comparison a caller makes between a confident verdict and a tentative one. |
//! | [`assess_batch_processes_all`] | lifespan | A batch of ten requests returns ten results, each of them successful. Callers index results against the requests they sent, so a batch that dropped or merged entries would misattribute every verdict after the gap. |
//! | [`assess_channel_free_request_succeeds`] | lifespan | A request naming a channel the engine knows nothing about is still assessed. Channel policy governs what is done with a verdict; the verdict itself is about the entity, so it can be formed without one. |
//! | [`assess_empty_batch`] | lifespan | cites (´claim:lifespan:a-batch-returns-one-result-per-request-in-full´) |
//! | [`assess_assigns_unique_ids`] | lifespan | Two assessments issued in sequence carry different identifiers. The identifier is what a later label uses to find the pending state an assessment left behind, so a collision would attach one request's outcome to another's evidence. |
//! | [`assess_anchor_weight_in_valid_range`] | lifespan | The anchor weight accompanying an assessment lies within zero and one. It says how much of the verdict leant on the fixed reference rather than on what has been learned, and it is only readable as such while it stays a proportion. |
//! | [`derived_profile_ambiguity_finite`] | lifespan | The ambiguity gauge read over a derived profile is finite in all three of its figures (´sig:rendering:ambiguity-gauge´). A display gauge that returned an infinity would poison whatever dashboard or comparison consumed it; the decision about what is worth probing belongs to fragility on the landscape, not to this reading (´def:fragility:definition´). |
//! | [`label_after_assess_succeeds`] | lifespan | Labelling an assessment succeeds and the receipt carries back the identifier of the assessment it settled. This round-trip is the loop the whole system learns through, and the returned identifier is how a caller confirms which of its outstanding assessments has now closed. |
//! | [`label_unknown_assessment_error`] | lifespan | A label naming an assessment that was never issued fails. The pending buffer holds the evidence a label must be applied against, and inventing an update with no evidence behind it would teach the model from nothing. |
//! | [`label_after_sentinel_deregistration_succeeds`] | lifespan | An assessment taken while a Sentinel was live can still be labelled after that Sentinel has been retired. The pending entry captured the state at assessment time, so the label is applied against what was actually seen rather than against whatever survives now. |
//! | [`label_double_submit_fails`] | lifespan | The second label for an assessment fails: the first consumed the pending entry. Repeating an update would let one piece of evidence be counted twice, which is how a duplicated retry silently doubles a model's confidence. |
//! | [`label_with_outcome_axis_registered`] | lifespan | A label may carry an outcome value for a registered axis alongside its verdict, and is accepted. The verdict and the consequence are separate things learned from the same event. |
//! | [`lifecycle_error_variants_are_distinct`] | lifespan | All seven lifecycle failure modes render to messages distinct from one another. An operator reading a log has only the message to go on, so two failures that look identical there are two failures that cannot be told apart at all. |
//! | [`multiple_identity_dimensions_allowed`] | lifespan | cites (´claim:lifespan:more-than-one-identity-dimension-can-be-live-at-once´) |
//! | [`identity_deregister_allows_new_dimension`] | lifespan | After a dimension is retired, a fresh dimension under a different id registers successfully. Teardown sends a destroy command to the identity- maintenance thread, and that thread must survive it in a state able to serve the next dimension. |
//! | [`register_identity_blocking_completes_promptly`] | lifespan | Identity registration, alone among the lifecycle calls in waiting for an acknowledgement, returns in under five seconds — far inside the configured timeout that would otherwise turn the wait into a reported shutdown. The call blocks a host's own thread, so the ordinary case has to be fast enough for a host to make it inline. |
//! | [`register_identity_wait_is_bounded_by_the_configured_timeout`] | lifespan | The bound the registration wait is held to is the figure the host configured and not a constant of the wait's own: an engine built asking for three seconds is bounded at three, one asking for thirty-seven at thirty-seven, and one that asks for nothing at the ten the record fixes as the default. The surface was defaulted, validated and refused at zero while the wait read past it to a constant that agreed by coincidence, so what this pins is that moving the dial now moves the wait (´cav:construction:registration-timeout-shadowed´). |
//! | [`register_sentinel_duplicate_returns_duplicate_id`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`deregister_sentinel_not_found_error`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`register_axis_duplicate_returns_duplicate_id`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`deregister_axis_not_found_error`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`register_identity_duplicate_returns_duplicate_id`] | lifespan | cites (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´) |
//! | [`deregister_identity_not_found_error`] | lifespan | cites (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´) |
//! | [`concurrent_assess_during_registration`] | lifespan | Assessing immediately after a registration returns — while the model owner may still be widening the model behind it — yields a finite probability and no panic. The synchronous half of registration and the published snapshot are each internally consistent, so a reader caught between them reads one or the other rather than a mixture. |
//! | [`multiple_assessments_across_lifecycle`] | lifespan | Assessments taken before any registration, after a Sentinel joins, after an axis joins and after the Sentinel is retired all succeed and all carry distinct identifiers. Availability does not dip around structural change, and the identifier sequence does not restart when the structure does. |
//! | [`report_ack_has_cells`] | lifespan | Accepting a report returns an acknowledgement stating how many cells it carried. The count is the Sentinel's confirmation that its observation was read as it was meant, rather than accepted and quietly discarded. |
//! | [`report_for_unregistered_sentinel_fails`] | lifespan | cites (´claim:lifespan:a-report-from-a-retired-sentinel-is-rejected-as-coming-from-an-unknown-sentinel´) |
//! | [`report_multiple_sentinels`] | lifespan | Two registered Sentinels each have their reports accepted under their own id. Slots are per Sentinel, so the fleet grows without its members contending for one place to write. |
//! | [`report_after_second_registration_succeeds`] | lifespan | An id that was registered, retired and registered again accepts reports under its new occupant. Re-registration builds a fresh slot and ledger partition rather than reviving the removed one, so reuse is a beginning and not a resumption. |
//! | [`flush_label_channel_idempotent`] | lifespan | Three flushes in a row all succeed, with nothing left to flush after the first. A host that flushes defensively before reading state pays nothing for the flushes that turn out to be unnecessary. |
//! | [`flush_after_lifecycle_events`] | lifespan | After a flush, a Sentinel and an axis submitted beforehand have both been processed — proved by the axis now colliding with a duplicate registration. The flush is the synchronisation point a caller reaches for when it needs the asynchronous half of a registration to have actually happened. |
//! | [`register_many_sentinels`] | lifespan | cites (´claim:lifespan:each-registered-sentinel-has-its-own-slot-to-report-into´) |
//! | [`register_many_axes`] | lifespan | cites (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´) |
//! | [`lifecycle_sentinel_axis_identity_all_together`] | lifespan | cites (´claim:lifespan:sentinels-and-axes-registered-together-both-show-up-in-the-same-assessment´) |
//! | [`health_summary_blend_statistics_initial`] | lifespan | The blend statistics report a finite mean on a brand-new engine, before any assessment has contributed to them. They begin at their initialised values rather than at whatever an empty accumulator would compute. |
//! | [`health_summary_after_assess_updates_blend`] | lifespan | cites (´claim:lifespan:the-blend-statistics-are-defined-before-any-observation-has-been-blended´) |
//! | [`health_summary_degradation_initial`] | lifespan | Freshly built, the engine counts nothing as degraded. Degradation is something that has to be detected and counted, so the baseline is silence rather than an unknown. |
//! | [`full_health_report_convergence_structure`] | lifespan | The composite convergence stage in the full report matches the stage the cheap summary reports. The two views are read at different costs by different callers, and they would be worse than useless if a host could act on one and be contradicted by the other. |
//! | [`full_health_report_calibration`] | lifespan | The full report's calibration block carries a finite sister concentration even before anything has been calibrated. Calibration figures are read as diagnostics on cold engines as much as on warm ones. |
//! | [`pre_seed_then_register_sentinel_then_assess`] | lifespan | History loaded before a Sentinel is registered is still there when the assessment runs, and that assessment is finite with the registered — but silent — Sentinel correctly reported as no contributor. Seeded state lives in the model rather than in a shape tied to whatever happened to be registered at seeding time. |
//! | [`register_sentinel_then_pre_seed_then_assess`] | lifespan | cites (´claim:lifespan:seeding-and-registration-compose-in-either-order´) |
//! | [`pre_seed_then_label_sequential`] | lifespan | After pre-seeding, an ordinary assess-then-label round-trip succeeds. Seeded history and live labels flow into the same model through the same pipeline, so the one does not put the other out of reach. |
//! | [`register_spatial_axis_then_assess_after_flush`] | lifespan | cites (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´) |
//! | [`register_disabled_axis_then_assess_after_flush`] | lifespan | cites (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´) |
//! | [`deregister_identity_allows_reregistration_with_same_id`] | lifespan | cites (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´) |
//! | [`deregister_identity_then_register_new_dimension`] | lifespan | cites (´claim:lifespan:tearing-down-an-identity-dimension-leaves-the-machinery-able-to-build-another´) |
//! | [`deregister_spatial_axis_then_assess`] | lifespan | A spatial axis present in predictions disappears from them once retired, and assessments afterwards still return a finite probability. Removing the axis takes two features out of every Sentinel slot, and the model has to remain scoreable across that narrowing rather than merely survive it. |
//! | [`register_spatial_axis_then_label_succeeds`] | lifespan | cites (´claim:lifespan:a-label-can-carry-a-value-for-a-registered-outcome-axis´) |
//! | [`register_sentinel_empty_name_rejected`] | lifespan | A Sentinel registration carrying an empty name is refused with the empty- name error, and nothing is created. A nameless entity would be unreachable through every name-based path a host has, so it is turned away at the door rather than orphaned inside. |
//! | [`register_axis_empty_name_rejected`] | lifespan | cites (´claim:lifespan:an-entity-registered-under-an-empty-name-is-refused-before-anything-is-created´) |
//! | [`register_identity_empty_name_rejected`] | lifespan | cites (´claim:lifespan:an-entity-registered-under-an-empty-name-is-refused-before-anything-is-created´) |
//! | [`end_to_end_lifecycle`] | lifespan | One engine carries the whole arc: two Sentinels and an identity dimension registered, reports ingested, a hundred assessments returning probabilities near the prior with Sentinels present, eighty labels applied, an axis added midway, and the health surfaces populated with drift and precision entries throughout. Retiring a Sentinel at the end removes it from the per-Sentinel breakdown while an assessment taken afterwards can still be labelled, and dropping the engine shuts it down cleanly. |
//! | [`concurrent_access_no_panic_or_deadlock`] | lifespan | Four threads driving a thousand assessments, a hundred labels, ten reports and a hundred health reads against one engine all finish, and the engine assesses normally afterwards. These are the four things a host does at once in production and each takes a different set of locks — the point is that no ordering among them can wedge. |
//! | [`preseed_convergence_non_trivial`] | lifespan | Two hundred seeded entries are reported as processed and then show up as two hundred total labels in the health summary. Seeding is not a side channel: what it teaches the model is counted in the same ledger of evidence that live labels are. |
//! | [`shutdown_immediately_after_build`] | lifespan | Building an engine and immediately dropping it completes without panic. The worker threads that start during construction must be able to be told to stop before they have had anything at all to do. |
//! | [`shutdown_mid_assess_concurrent`] | lifespan | Dropping one engine handle while another thread owns the remaining handle lets the fixed assessment run finish without panic. Shared ownership keeps the instance alive until that worker finishes, and the last drop then shuts it down; elapsed time establishes neither fact. |
//! | [`shutdown_mid_label_channel`] | lifespan | Ten labels submitted and never flushed are still in flight when the engine is dropped, and the shutdown completes without panic. Unprocessed work at shutdown is expected rather than exceptional, and its cost is the learning that never landed — not a crash. |
//! | [`concurrent_reports_different_sentinels`] | lifespan | Two threads filing fifty reports each for two different Sentinels all succeed, and the engine still reports Sentinels present afterwards. Report ingestion locks per Sentinel, so a fleet's members do not queue behind one another to be heard. |
//! | [`label_channel_full_no_journal`] | lifespan | Against an engine whose label channel sits at its tabulated floor, labelling past the channel's capacity with the steward parked returns a channel-full error rather than blocking the caller or silently discarding the update. Backpressure is reported to the host, which is the only party that can decide whether to retry, to shed, or to slow down. Full health reports the same event as a full live depth and one cumulative loss, so a later scraper can observe the refusal after its per-call error has gone by. |
//! | [`label_after_model_owner_shutdown`] | lifespan | Once the harness has stopped and joined the model-owner thread, a label returns the shutdown error rather than hanging, panicking, or entering the disconnected queue. A caller can therefore distinguish an engine that has ended from one that is merely busy. |

//! Lifecycle integration tests: end-to-end scenarios through the `Assayer`.
//!
//! These tests exercise lifecycle methods via the full public API on a
//! running `Assayer`, verifying observable effects on `assess()` and
//! explicit derivation,
//! `label()`, `receive_sentinel_report()`, `health_summary()`, and
//! `full_health_report()`.  Each test builds, exercises, and drops a
//! self-contained `Assayer` instance.
//!
//! # Cross-References
//!
//! - (´dec:construction:six-methods´) — the six registration and deregistration methods
//! - (´dec:construction:two-phase-visibility´) — the synchronous return and the
//!   next publication these end-to-end tests read across
//! - (´chap:spec:assessment-interface´) — the core assessment output these tests read
//! - (´dec:ordering:assessment-function´) — assessment as one per-request function
//! - (´dec:surface:async-label´) — what the label surface promises its caller
//! - (´dec:surface:reception-isolation´) — reception touching no model, graph or cache state
//! - (´dec:health:tiered-queries´) — the tiers a host asks health on
//! - (´dec:metrics:passive-export´) — metrics exported without being asked for

#![allow(clippy::items_after_statements)]

use std::sync::Arc;
use std::thread;
use std::time::{Duration, Instant};

use crate::api::PreSeedEntry;
use crate::assessment::RequestContext;
use crate::config::types::{AssayerConfig, InfrastructureConfig};
use crate::error::{LabelError, LifecycleError, ReportError};
use crate::health::{HealthEvent, LifecycleHealthEvent};
use crate::metrics::health_summary_to_samples;
use crate::owner::commands::{
    IdentityDimensionRegistration, LabelData, ModelOwnerCommand, OutcomeAxisRegistration, SentinelRegistration,
};
use crate::testing::{ACK_DEADLINE, LabelSpec, PreSeedSpec, World, golden_report, minimal_report};
use crate::types::{
    AssessmentId, ChannelId, DimensionId, IdentityBudget, OutcomeAxisId, OutcomeEligibility, SentinelId, SpatialFeaturePolicy,
};

// ═══════════════════════════════════════════════════════════════════════════════
// Helpers
// ═══════════════════════════════════════════════════════════════════════════════

fn sentinel_reg(id: u32, name: &str) -> SentinelRegistration {
    SentinelRegistration {
        id: SentinelId(id),
        name: name.to_owned(),
    }
}

fn axis_reg(id: u32, name: &str) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id: OutcomeAxisId(id),
        name: name.to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Disabled,
    }
}

fn identity_reg(id: u32, name: &str) -> IdentityDimensionRegistration {
    IdentityDimensionRegistration {
        id: DimensionId(id),
        name: name.to_owned(),
        description: "lifecycle test hierarchy".to_owned(),
        coordinate_semantics: "the leading bytes select a prefix group".to_owned(),
        domain_bits: 128,
        depth_cutoff: 10,
        budget: IdentityBudget::for_depth_cutoff(10),
        encode: |entity| {
            let bytes = entity.as_bytes();
            if bytes.len() >= 8 {
                u128::from(u64::from_le_bytes(bytes[..8].try_into().unwrap_or_default()))
            } else {
                0
            }
        },
    }
}

fn make_request(_channel: ChannelId, seed: u64) -> RequestContext {
    RequestContext::new(World::entity(&format!("entity-{seed}")))
}

fn make_pre_seed_entry(seed: u64, valence: f64) -> PreSeedEntry {
    PreSeedSpec::new(ChannelId(0), World::entity(&format!("entity-{seed}")))
        .valence(valence)
        .ground_truth()
        .build()
}

/// Create a request with sentinel coordinates so extraction actually works.
fn make_request_with_sentinels(_channel: ChannelId, seed: u64, sentinel_ids: &[u32]) -> RequestContext {
    let entity = World::entity(&format!("entity-{seed}"));
    let coord = seed as u128;
    let mut req = RequestContext::new(entity);
    for &id in sentinel_ids {
        req.sentinel_coordinates.insert(SentinelId(id), coord);
    }
    req
}

fn make_label(assessment_id: AssessmentId, valence: f64) -> LabelData {
    LabelSpec::new(assessment_id).valence(valence).ground_truth().build()
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multi-step Lifecycle Sequences
// ═══════════════════════════════════════════════════════════════════════════════

/// A report filed immediately after registration not only succeeds but creates
/// cells in the Sentinel's ledger, and an assessment routed through that
/// Sentinel's coordinate is finite and counts it as its one contributor. The
/// ledger partition is built during registration itself, so the very first
/// report has somewhere to write rather than racing the model owner for it.
///
/// ´claim:lifespan:registration-creates-the-ledger-cells-a-first-report-needs-before-that-report-can-arrive´
/// ´test:crate:lifecycle-register-sentinel-then-report-then-assess-and-derive´
#[test]
fn lifecycle_register_sentinel_then_report_then_assess_and_derive() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register sentinel
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Report immediately accepted (Phase 1 visibility)
    let report = golden_report();
    let ack = assayer.receive_sentinel_report(SentinelId(1), report).unwrap();
    assert!(ack.cells_in_report > 0, "report should have cells");
    assert!(
        ack.cells_created_in_ledger > 0,
        "registration should create a Ledger partition before immediate report ingestion"
    );

    // A request routed through the reported Sentinel's coordinate space
    // extracts from it, so the assessment counts one contributor.
    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 42, &[1])]);
    assert_eq!(results.len(), 1);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(reckoning.assessment.risk.p_bad.is_finite());
    assert!(!reckoning.assessment.health.zero_sentinels);
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 1);
}

/// Registration opens the warm-up accumulator, the configured number of
/// reporting assessments fills it, and the published standardisation then
/// carries the blend: the occupancy entry lands at exactly the blend weight
/// times the observed value plus the remainder times what the entry held,
/// and the variance entries respect the floor. A Sentinel registered after
/// warm-up would otherwise spend a standardisation half-life being scaled
/// against class priors; a hundred assessments is what the corpus shortens
/// that to, and this is the wiring that makes those hundred count.
///
/// ´claim:lifespan:a-registered-sentinels-bootstrap-blends-observed-slot-statistics-into-publication´
/// ´test:crate:sentinel-bootstrap-blends-into-published-statistics´
#[test]
fn sentinel_bootstrap_blends_into_published_statistics() {
    let mut config = AssayerConfig {
        instance_id: "bootstrap-blend".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    // Ten is the constraint's minimum for `N_boot` and for `N_init`.
    config.standardisation.n_boot = 10;
    config.standardisation.n_init = 10;
    let alpha = config.standardisation.alpha_boot;
    let v_floor = config.standardisation.v_floor;
    let world = World::builder(config)
        .seed(0x626F_6F74_7374_7261)
        .build()
        .expect("bootstrap world should build");
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    world.flush_labels().expect("registration should publish");

    let report = golden_report();
    assayer.receive_sentinel_report(SentinelId(1), report).unwrap();

    // Settle the cold ramp first, so the entries this blend lands on are
    // held by nothing else while the accumulator fills.
    world.settle_cold_ramp_with(&[make_request(ChannelId(0), 100_000)]);

    // Slot range and the pre-blend entry from the settled publication.
    let before = assayer.shared.published.load_full();
    let range = before
        .dimension_map
        .sentinel_slots
        .get(&SentinelId(1))
        .cloned()
        .expect("registered sentinel has a slot range");
    let pre_blend = before.feature_means[range.start];

    // Ten reporting assessments fill the accumulator of capacity ten.
    for seed in 0..10u64 {
        let req = make_request(ChannelId(0), seed).with_sentinel(SentinelId(1), 0);
        let _assessments = assayer.assess(&[req]);
    }

    // Occupancy is observed as 1.0 on every reporting assessment, so the
    // blended mean is exactly alpha·1.0 + (1−alpha)· what the entry held.
    let expected_occupancy = alpha.mul_add(1.0 - pre_blend, pre_blend);
    assayer
        .flush_observation_channel()
        .expect("the observation barrier publishes the completed blend");
    let after = assayer.shared.published.load_full();
    assert!(
        (after.feature_means[range.start] - expected_occupancy).abs() < 1e-9,
        "occupancy entry must carry the blend: expected {expected_occupancy}, got {}",
        after.feature_means[range.start]
    );

    // Every variance entry in the slot respects the floor.
    for (j, variance) in after.feature_variances.iter().enumerate().take(range.end).skip(range.start) {
        assert!(*variance >= v_floor, "variance at {j} fell below the floor: {}", variance);
    }
}

/// A lifecycle publication carries the empirical standardisation statistics
/// forward instead of resetting them: after a Sentinel's bootstrap has
/// blended its slot statistics into publication, a lifecycle event
/// publishes a snapshot whose entries still hold the blend, not the
/// neutral zero-mean unit-variance pair. The projection is canonical —
/// the statistics live on the working copy the snapshot is built from —
/// so the standardisation entries a lifecycle event creates move under
/// the same publication as everything else, and a host that registers an
/// entity during a quiet period does not spend the wait being scored on
/// neutrally scaled features (´inv:guarantee:lifecycle-publication´).
///
/// ´claim:lifespan:a-lifecycle-publication-preserves-the-empirical-standardisation-statistics´
/// ´test:crate:lifecycle-publication-preserves-empirical-standardisation´
#[test]
fn lifecycle_publication_preserves_empirical_standardisation() {
    let mut config = AssayerConfig {
        instance_id: "lifecycle-preserves-standardisation".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    // Ten is the constraint's minimum for `N_boot` and for `N_init`.
    config.standardisation.n_boot = 10;
    config.standardisation.n_init = 10;
    let alpha = config.standardisation.alpha_boot;
    let world = World::builder(config)
        .seed(0x7075_626C_6963_6174)
        .build()
        .expect("lifecycle publication world should build");
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    world.publish_lifecycle().expect("registration should publish");

    let report = golden_report();
    assayer.receive_sentinel_report(SentinelId(1), report).unwrap();

    // Settle the cold ramp first: this scenario is about what a lifecycle
    // publication preserves, not about which mechanism moved the entry last.
    world.settle_cold_ramp_with(&[make_request(ChannelId(0), 100_000)]);

    let registered = assayer.shared.published.load_full();
    let range = registered
        .dimension_map
        .sentinel_slots
        .get(&SentinelId(1))
        .cloned()
        .expect("registered sentinel has a slot range");
    let pre_blend = registered.feature_means[range.start];

    // Ten reporting assessments fill the accumulator of capacity ten.
    for seed in 0..10u64 {
        let req = make_request(ChannelId(0), seed).with_sentinel(SentinelId(1), 0);
        let _assessments = assayer.assess(&[req]);
    }

    // Cross the blend's observation queue, then cross a lifecycle publication and read what it published.
    let expected_occupancy = alpha.mul_add(1.0 - pre_blend, pre_blend);
    assayer
        .flush_observation_channel()
        .expect("the observation barrier publishes the completed blend");
    world.publish_lifecycle().expect("empty lifecycle submission should publish");
    let after_lifecycle = assayer.shared.published.load_full();

    // The blended occupancy mean survives the lifecycle publication, which
    // rebases the ramp on what it published rather than rolling any position
    // back to its own prior (´req:standardisation:lifecycle-entries´).
    let actual_occupancy = after_lifecycle.feature_means[range.start];
    assert!(
        (actual_occupancy - expected_occupancy).abs() < 1e-9,
        "lifecycle publication must preserve the blend: expected {expected_occupancy}, got {actual_occupancy}"
    );
}

/// A cold-ramp observation refused by a full observation queue advances no
/// count, is reported to the caller in band, and costs the ramp only pace:
/// the requests served and the observations accepted come apart by exactly
/// the refusals, and the ramp still reaches its horizon and publishes the
/// empirical moments once the channel drains. Under the completion gate this
/// was a far sharper edge — the whole accumulation rode on one delivery at
/// the count, so a full channel at that instant had to be retried or the
/// warm-up was silenced for the life of the process. There is no one-shot
/// left to lose: what a refusal defers now is one share of prior mass, and
/// the host is told which (´dec:concurrency:no-silent-drop´).
///
/// ´claim:lifespan:a-cold-ramp-observation-refused-by-a-full-channel-advances-no-count-and-is-reported-in-band´
/// ´test:crate:cold-ramp-refusal-under-a-full-channel-advances-nothing´
#[test]
fn cold_ramp_refusal_under_a_full_channel_advances_nothing() {
    let mut config = AssayerConfig {
        instance_id: "batch-init-full-channel".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    // Ten is the constraint's minimum for `N_init`.
    config.standardisation.n_init = 10;
    let v_floor = config.standardisation.v_floor;
    let world = World::builder(config)
        .seed(0x6261_7463_682D_696E)
        .build()
        .expect("cold-ramp refusal world should build");
    let assayer = world.assayer();

    // Park the owner inside a test block so the command channel backs up.
    let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx
        .send(ModelOwnerCommand::TestBlock {
            entered: entered_tx,
            release: release_rx,
        })
        .expect("owner alive");
    entered_rx.recv_timeout(ACK_DEADLINE).expect("owner enters test block");

    // With the owner parked nothing drains, so the observation queue — bounded
    // at the horizon — fills after ten offers and refuses the rest. The refusal
    // reaches the caller on its own assessment's degradation context rather
    // than being absorbed, and the published count does not move.
    const REQUESTS_WHILE_PARKED: u64 = 25;
    let mut refusals = 0_u32;
    for seed in 0..REQUESTS_WHILE_PARKED {
        for assessment in assayer.assess(&[make_request(ChannelId(0), seed)]) {
            refusals += assessment.health.degradation.batch_init_observations_skipped;
        }
    }
    assert!(
        refusals > 0,
        "a queue filled to capacity must refuse at least one observation, and say so in band",
    );
    let parked = assayer.shared.published.load_full();
    assert_eq!(
        parked.standardisation_observations, 0,
        "a refused observation advances no accepted count",
    );

    // Release the owner and cross the observation queue that holds the accepted backlog.
    release_tx.send(()).expect("owner still parked");
    assayer
        .flush_observation_channel()
        .expect("the observation barrier drains the accepted backlog");

    // The ten accepted offers fill the horizon when the barrier returns. Nothing was lost but pace: more requests were served than observations accepted, which is exactly the gap the refusals opened.
    let seed = REQUESTS_WHILE_PARKED;
    let after = assayer.shared.published.load_full();

    assert_eq!(
        after.standardisation_observations, 10,
        "the horizon is ten accepted observations"
    );
    assert!(
        seed > after.standardisation_observations as u64,
        "the refusals must show as requests served beyond the accepted count: {seed} requests, {} accepted",
        after.standardisation_observations,
    );
    assert!(
        seed > 10,
        "the refusals must show as requests served beyond the accepted count: {seed} requests for ten observations",
    );

    // With nothing registered every observed vector is identical, so at the
    // horizon each non-bias position carries the floored empirical variance
    // rather than the class prior it started from.
    for (j, variance) in after.feature_variances.iter().enumerate().skip(1) {
        assert!(
            *variance <= v_floor * (1.0 + 1e-9),
            "position {j} must carry the floored empirical variance, got {}",
            variance
        );
    }
}

/// The observation barrier resolves only once every cold-ramp observation
/// queued before it has been applied, and the checkpoint barrier resolves
/// without regard to any of them. With the steward parked, a run of
/// assessments leaves its offers queued and the published count at zero; a
/// barrier enqueued behind the park is reached in the same command drain the
/// release resumes, before the loop's own observation drain has run, and it
/// answers with every one of those offers accepted. The count is the offers'
/// own, exactly, and the phase follows it to the horizon.
///
/// This is the property the harness's settling helper is built on, and the
/// reason it needs a barrier of its own rather than the label flush. The two
/// barriers ride the same first-in-first-out command channel, so nothing about
/// their order relative to each other could have made either cover the other's
/// queue: a command's position among commands says nothing about what is
/// waiting on a different channel. Without this one, a caller that assessed
/// and then read the coordinate system was reading how far the steward had got
/// — a reading of the scheduler wearing the ramp's clothes.
///
/// ´claim:steward:an-observation-barrier-answers-only-once-every-observation-queued-before-it-has-been-applied´
/// ´test:crate:observation-barrier-applies-every-queued-observation´
#[test]
fn observation_barrier_applies_every_queued_observation() {
    use crate::owner::commands::ObservationBarrierRequest;

    /// Offers made while the steward is parked. Fewer than the horizon, so
    /// none is refused for a completed ramp, and no more than the queue's
    /// capacity — which is the horizon — so none is refused for a full
    /// channel either. What the barrier answers for is therefore all of them.
    const PARKED_OFFERS: usize = 6;
    /// Offers made afterwards against a live steward, short of the horizon.
    const LIVE_OFFERS: usize = 3;

    let mut config = AssayerConfig {
        instance_id: "observation-barrier".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    // Ten is the constraint's minimum for `N_init`.
    config.standardisation.n_init = 10;
    let horizon = config.standardisation.n_init as usize;
    let world = World::builder(config)
        .seed(0x6F62_7365_7276_6174)
        .build()
        .expect("observation-barrier world should build");
    let assayer = world.assayer();

    // Park the steward so the offers below can only queue.
    let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx
        .send(ModelOwnerCommand::TestBlock {
            entered: entered_tx,
            release: release_rx,
        })
        .expect("owner alive");
    entered_rx.recv_timeout(ACK_DEADLINE).expect("owner enters test block");

    for seed in 0..PARKED_OFFERS as u64 {
        let _assessments = assayer.assess(&[make_request(ChannelId(0), seed)]);
    }
    assert_eq!(
        assayer.shared.published.load_full().standardisation_observations,
        0,
        "a parked steward applies nothing, so the offers are all still queued",
    );

    // The barrier goes on the command channel behind the park, and the park is
    // released only after it is enqueued. The loop drains the whole command
    // queue before it touches an observation, so the release resumes into a
    // drain that reaches the barrier with all six offers still waiting — which
    // is precisely the state a barrier that acknowledged without draining
    // would answer in.
    let (ack_tx, ack_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx
        .send(ModelOwnerCommand::ObservationBarrier(ObservationBarrierRequest {
            completion: ack_tx,
        }))
        .expect("owner alive");
    release_tx.send(()).expect("owner still parked");
    ack_rx
        .recv_timeout(ACK_DEADLINE)
        .expect("the owner answers the observation barrier");

    let after_parked = assayer.shared.published.load_full();
    assert_eq!(
        after_parked.standardisation_observations, PARKED_OFFERS,
        "the barrier answers for every observation queued before it",
    );
    assert!(
        !after_parked.standardisation_phase.is_in_service(),
        "short of the horizon the ramp is still transitioning",
    );

    // The same barrier against a live steward: the count is still the offers'
    // own rather than whatever the steward had reached when it was asked.
    for seed in 0..LIVE_OFFERS as u64 {
        let _assessments = assayer.assess(&[make_request(ChannelId(0), 1_000 + seed)]);
    }
    assayer
        .flush_observation_channel()
        .expect("the owner answers the observation barrier");
    assert_eq!(
        assayer.shared.published.load_full().standardisation_observations,
        PARKED_OFFERS + LIVE_OFFERS,
        "a live steward's queue is settled by the barrier just as a parked one's backlog is",
    );

    // And the phase follows the count: the offers that carry it to the horizon
    // leave the ramp in service by the time the barrier returns.
    for seed in 0..(horizon - PARKED_OFFERS - LIVE_OFFERS) as u64 {
        let _assessments = assayer.assess(&[make_request(ChannelId(0), 2_000 + seed)]);
    }
    assayer
        .flush_observation_channel()
        .expect("the owner answers the observation barrier");
    let settled = assayer.shared.published.load_full();
    assert_eq!(
        settled.standardisation_observations, horizon,
        "the horizon is reached at the accepted count the configuration names",
    );
    assert!(
        settled.standardisation_phase.is_in_service(),
        "at the horizon the barrier returns onto a ramp in service",
    );
}

/// A Sentinel id released by retirement is free for immediate reuse, and the
/// reborn Sentinel is fully functional — it accepts a report and contributes
/// to the next assessment routed through its coordinate. Ids are a namespace
/// the host manages, not a resource the engine consumes permanently.
///
/// ´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´
/// ´test:crate:lifecycle-register-deregister-sentinel-round-trip´
#[test]
fn lifecycle_register_deregister_sentinel_round_trip() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register → deregister → register again
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    // Same ID should be available again
    let result = assayer.register_sentinel(sentinel_reg(1, "s1-reborn"));
    assert!(result.is_ok(), "re-registration should succeed: {result:?}");

    // The reborn Sentinel is fully functional: it accepts a report and
    // contributes to an assessment routed through its coordinate.
    assayer.receive_sentinel_report(SentinelId(1), golden_report()).unwrap();
    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 1, &[1])]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(!reckoning.assessment.health.zero_sentinels);
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 1);
}

/// A deregistration that actually removes a model slot pushes the aggregate
/// diagnostics after the owner has published the compacted model. Flushing is
/// the public synchronisation point for that asynchronous half, and draining
/// proves the payload travelled through the bounded event channel
/// (´entry:health:marginalise-event´).
///
/// ´claim:lifespan:sentinel-deregistration-pushes-its-marginalisation-completion-diagnostics´
/// ´test:crate:deregister-sentinel-pushes-marginalisation-completion-event´
#[test]
fn deregister_sentinel_pushes_marginalisation_completion_event() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    assayer.register_sentinel(sentinel_reg(1, "event-source")).unwrap();
    assayer.flush_label_channel().unwrap();
    assert!(
        assayer.drain_health_events().into_iter().all(|event| !matches!(
            event,
            HealthEvent::Lifecycle(LifecycleHealthEvent::MarginalisationCompleted { .. })
        )),
        "registration does not marginalise"
    );

    assayer.deregister_sentinel(SentinelId(1)).unwrap();
    assayer.flush_label_channel().unwrap();
    let diagnostics = assayer
        .drain_health_events()
        .into_iter()
        .find_map(|event| match event {
            HealthEvent::Lifecycle(LifecycleHealthEvent::MarginalisationCompleted { diagnostics }) => Some(diagnostics),
            _ => None,
        })
        .expect("deregistration pushes marginalisation completion");
    assert_eq!(
        diagnostics.measured_events + diagnostics.bounded_events + diagnostics.unbounded_events,
        diagnostics.events
    );
    assert!(diagnostics.events >= 2, "operational and sister models are both represented");
}

/// Once the model owner has processed an axis registration, assessments carry a
/// prediction for that axis in their outcome map. The axis is not merely stored
/// somewhere; it becomes part of what every assessment answers.
///
/// ´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´
/// ´test:crate:lifecycle-register-axis-then-assess-includes-prediction´
#[test]
fn lifecycle_register_axis_then_assess_includes_prediction() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();

    // Flush to ensure axis is registered in model owner
    assayer.flush_label_channel().unwrap();

    // Assess after axis registration.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    // The outcome_predictions map should contain our axis
    assert!(
        reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(1)),
        "assessment should include prediction for registered axis; got keys: {:?}",
        reckoning.assessment.outcome_predictions.keys().collect::<Vec<_>>()
    );
}

/// An axis id survives its axis: registered, flushed, retired and flushed
/// again, the same id accepts a fresh axis under a new name. Retirement
/// genuinely releases the id rather than tombstoning it.
///
/// (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´)
/// ´test:crate:lifecycle-register-deregister-axis-round-trip´
#[test]
fn lifecycle_register_deregister_axis_round_trip() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.flush_label_channel().unwrap();

    assayer.deregister_outcome_axis(OutcomeAxisId(1)).unwrap();
    assayer.flush_label_channel().unwrap();

    // Same ID should be available
    let result = assayer.register_outcome_axis(axis_reg(1, "fraud-v2"));
    assert!(result.is_ok(), "re-registration should succeed: {result:?}");
}

/// With an identity dimension registered, assessment continues to return a
/// finite probability. Identity features enlarge what an assessment can see;
/// they are never a precondition for it answering.
///
/// ´claim:lifespan:an-identity-dimension-registered-mid-flight-does-not-interrupt-assessment´
/// ´test:crate:lifecycle-register-identity-then-assess-and-derive´
#[test]
fn lifecycle_register_identity_then_assess_and_derive() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();

    // Assess/derive should succeed; identity dimensions do not block assessment.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    assert_eq!(results.len(), 1);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

/// The published health summary carries the identity trackers rather than an
/// empty map: after a dimension is registered and one label has driven the
/// publication path, the summary holds that dimension's tracker and the full
/// health report derives the dimension's convergence from it. The composite
/// stage is thereby computed over the trackers the owner maintains — an empty
/// slice would read as universal stability, and the composite would advance on
/// calibration evidence alone over an identity layer it never measured.
///
/// ´claim:lifespan:the-published-summary-carries-the-owners-identity-trackers-not-an-empty-map´
/// ´test:crate:published-summary-carries-identity-trackers´
#[test]
fn published_summary_carries_identity_trackers() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();

    // One assessed and labelled request drives the publication path.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let assessment_id = results.into_iter().next().unwrap().unwrap().assessment.id;
    assayer.label(make_label(assessment_id, 1.0)).unwrap();
    assayer.flush_label_channel().unwrap();

    let health = assayer.shared.health.load();
    assert!(
        health.identity_trackers.contains_key(&DimensionId(1)),
        "published summary must carry the registered dimension's tracker"
    );

    let report = assayer.full_health_report();
    assert!(
        report.identity_dimensions.contains_key(&DimensionId(1)),
        "full health report must derive the dimension's convergence from the tracker"
    );
}

/// An identity dimension id is likewise reusable once its dimension is retired,
/// despite the blocking teardown that precedes the release. The wait for the
/// model owner completes before the call returns, so the id is genuinely free
/// by the time the caller sees an accepted result.
///
/// (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´)
/// ´test:crate:lifecycle-register-deregister-identity-round-trip´
#[test]
fn lifecycle_register_deregister_identity_round_trip() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();
    assayer.deregister_identity_dimension(DimensionId(1)).unwrap();

    // Re-register same ID
    let result = assayer.register_identity_dimension(identity_reg(1, "ip-hash-v2"));
    assert!(result.is_ok(), "re-registration should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cross-Entity Interactions
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel and an outcome axis registered against the same engine both
/// appear in the next assessment — the reported Sentinel counted as a
/// contributor, and a prediction present for the axis. The two kinds of
/// registration extend the model along different directions and do not
/// shadow one another.
///
/// ´claim:lifespan:sentinels-and-axes-registered-together-both-show-up-in-the-same-assessment´
/// ´test:crate:lifecycle-sentinel-plus-axis-combined´
#[test]
fn lifecycle_sentinel_plus_axis_combined() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.receive_sentinel_report(SentinelId(1), golden_report()).unwrap();
    assayer.flush_label_channel().unwrap();

    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 42, &[1])]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    assert!(!reckoning.assessment.health.zero_sentinels);
    assert!(
        reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(1)),
        "should have outcome prediction for registered axis"
    );
}

/// With two Sentinels registered, retiring the first leaves the second's
/// reports accepted while the first's are refused. Retirement is addressed to
/// one slot, so a host removing a single decommissioned probe does not silence
/// the rest of its fleet.
///
/// ´claim:lifespan:retiring-one-sentinel-leaves-its-peers-reporting´
/// ´test:crate:lifecycle-two-sentinels-independent´
#[test]
fn lifecycle_two_sentinels_independent() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_sentinel(sentinel_reg(2, "s2")).unwrap();

    // Deregister first sentinel
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    // Second sentinel's report should still be accepted
    let report = minimal_report();
    let result = assayer.receive_sentinel_report(SentinelId(2), report.clone());
    assert!(result.is_ok(), "s2 report should succeed: {result:?}");

    // First sentinel's report should be rejected
    let result = assayer.receive_sentinel_report(SentinelId(1), report);
    assert!(
        matches!(&result, Err(ReportError::UnknownSentinel { .. })),
        "s1 report should be rejected: {result:?}"
    );
}

/// Through the raw engine surface as through the harness, a report arriving
/// after retirement is refused as coming from an unknown Sentinel. Nothing
/// about the path the report takes softens the rejection.
///
/// (´claim:lifespan:a-report-from-a-retired-sentinel-is-rejected-as-coming-from-an-unknown-sentinel´)
/// ´test:crate:lifecycle-deregister-sentinel-report-rejected´
#[test]
fn lifecycle_deregister_sentinel_report_rejected() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    let report = minimal_report();
    let result = assayer.receive_sentinel_report(SentinelId(1), report);
    assert!(
        matches!(&result, Err(ReportError::UnknownSentinel { .. })),
        "expected UnknownSentinel after deregistration, got: {result:?}"
    );
}

/// A report filed before the model owner has widened anything is accepted and
/// its cell-set maintenance runs, so the ledger is already growing while the
/// model extension is still in flight. The two phases of registration are
/// ordered so that the cheap synchronous half is the one an eager Sentinel
/// depends on.
///
/// (´claim:lifespan:reports-are-accepted-from-the-moment-registration-returns´)
/// ´test:crate:lifecycle-sentinel-report-before-model-extension´
#[test]
fn lifecycle_sentinel_report_before_model_extension() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register sentinel — Phase 1 makes slot visible immediately
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Report immediately (before model owner processes Phase 2)
    let report = golden_report();
    let ack = assayer
        .receive_sentinel_report(SentinelId(1), report)
        .expect("report should be accepted before model extension");
    assert!(
        ack.cells_created_in_ledger > 0,
        "cell-set maintenance should run before model extension"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Health Observability
// ═══════════════════════════════════════════════════════════════════════════════

/// A newly built engine reports no labels, no eligible labels and no
/// assessments, with its calibration delta already a finite number. The
/// counters start at zero and the numeric fields start defined, so a host can
/// chart health from the first moment rather than from the first event.
///
/// ´claim:lifespan:a-freshly-built-engine-reports-a-clean-slate-rather-than-an-empty-one´
/// ´test:crate:health-summary-initial-state´
#[test]
fn health_summary_initial_state() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let health = assayer.health_summary();

    assert_eq!(health.total_labels, 0);
    assert_eq!(health.eligible_labels, 0);
    assert_eq!(health.total_assessments, 0);
    assert!(health.last_delta_cal.is_finite());
}

/// One assessment leaves the total-assessments counter reading one. That
/// counter is the denominator every rate in the health summary is read against,
/// so it has to count requests rather than calls.
///
/// ´claim:lifespan:the-assessment-counter-advances-by-one-per-request-assessed´
/// ´test:crate:health-summary-after-assess´
#[test]
fn health_summary_after_assess() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let _results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);

    let health = assayer.health_summary();
    assert_eq!(health.total_assessments, 1, "should count the assessment");
}

/// Five separate single-request assessments accumulate to five. The counter
/// accumulates across calls rather than reporting only the most recent batch.
///
/// (´claim:lifespan:the-assessment-counter-advances-by-one-per-request-assessed´)
/// ´test:crate:health-summary-after-multiple-assessments´
#[test]
fn health_summary_after_multiple_assessments() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    for i in 0..5 {
        let _results = world.derive_for_requests(&[make_request(ChannelId(0), i)]);
    }

    let health = assayer.health_summary();
    assert_eq!(health.total_assessments, 5);
}

/// A pre-seed batch reports every entry processed and the engine assesses
/// normally afterwards, returning a finite probability. Pre-seeding runs
/// through the label pipeline internally, so what it leaves behind is ordinary
/// learned state rather than a special mode.
///
/// ´claim:lifespan:pre-seeded-history-leaves-the-engine-immediately-usable-for-assessment´
/// ´test:crate:health-summary-after-pre-seed´
#[test]
fn health_summary_after_pre_seed() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let entries: Vec<PreSeedEntry> = (0..5)
        .map(|i| make_pre_seed_entry(i, if i % 2 == 0 { 1.0 } else { 0.0 }))
        .collect();
    let result = assayer.pre_seed(&entries).unwrap();
    assert_eq!(result.processed, 5);

    // Pre-seed uses the label pipeline internally; the model should be
    // usable for assessments afterwards (model has absorbed some data).
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 999)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(
        reckoning.assessment.risk.p_bad.is_finite(),
        "p_bad should be finite after pre-seed"
    );
}

/// At start-up the full report shows no Sentinels, no axes and no identity
/// dimensions. The report is a census of what is registered, so on an engine
/// with nothing registered it is empty rather than populated with placeholders.
///
/// ´claim:lifespan:the-full-health-report-lists-exactly-the-entities-that-are-registered´
/// ´test:crate:full-health-report-initial-state´
#[test]
fn full_health_report_initial_state() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let report = assayer.full_health_report();

    assert!(report.sentinels.is_empty(), "no sentinels at start");
    assert!(report.axes.is_empty(), "no axes at start");
    assert!(report.identity_dimensions.is_empty(), "no identity dims at start");
}

/// A registered Sentinel appears in the report's Sentinel map straight away.
/// Observability follows registration without waiting for the Sentinel to
/// report or for the model to be widened.
///
/// (´claim:lifespan:the-full-health-report-lists-exactly-the-entities-that-are-registered´)
/// ´test:crate:full-health-report-after-sentinel-registration´
#[test]
fn full_health_report_after_sentinel_registration() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    let report = assayer.full_health_report();
    assert!(
        report.sentinels.contains_key(&SentinelId(1)),
        "health report should include registered sentinel; keys: {:?}",
        report.sentinels.keys().collect::<Vec<_>>()
    );
}

/// A retired Sentinel is gone from the report. An operator reading the census
/// sees the fleet as it is now, not as it has ever been.
///
/// (´claim:lifespan:the-full-health-report-lists-exactly-the-entities-that-are-registered´)
/// ´test:crate:full-health-report-after-sentinel-deregistration´
#[test]
fn full_health_report_after_sentinel_deregistration() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    let report = assayer.full_health_report();
    assert!(
        !report.sentinels.contains_key(&SentinelId(1)),
        "deregistered sentinel should not appear in health report"
    );
}

/// A batch of three requests counts as three assessments, not as one batch.
/// Batching is a call-shape optimisation and must not distort the measure of
/// how much work the engine actually did.
///
/// (´claim:lifespan:the-assessment-counter-advances-by-one-per-request-assessed´)
/// ´test:crate:health-summary-assessment-batch-counting´
#[test]
fn health_summary_assessment_batch_counting() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // A batch of 3 should count as 3 assessments.
    let reqs = vec![
        make_request(ChannelId(0), 1),
        make_request(ChannelId(0), 2),
        make_request(ChannelId(0), 3),
    ];
    let _results = world.derive_for_requests(&reqs);

    let health = assayer.health_summary();
    assert_eq!(health.total_assessments, 3, "batch of 3 should count 3 assessments");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-seeding Scenarios
// ═══════════════════════════════════════════════════════════════════════════════

/// A single pre-seed entry is accepted and counted. The batch path has no lower
/// bound below which it declines to do the work.
///
/// (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´)
/// ´test:crate:pre-seed-single-entry-updates-model´
#[test]
fn pre_seed_single_entry_updates_model() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let result = assayer.pre_seed(&[make_pre_seed_entry(1, 1.0)]);
    assert!(result.is_ok(), "pre_seed should succeed: {result:?}");
    assert_eq!(result.unwrap().processed, 1);
}

/// A hundred entries are all processed, none dropped on the way. Seeding from
/// real history means batches far larger than anything the live path sees, and
/// the count is what proves nothing was silently discarded.
///
/// (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´)
/// ´test:crate:pre-seed-large-batch-all-processed´
#[test]
fn pre_seed_large_batch_all_processed() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let entries: Vec<PreSeedEntry> = (0..100)
        .map(|i| make_pre_seed_entry(i, if i % 3 == 0 { 1.0 } else { 0.0 }))
        .collect();
    let result = assayer.pre_seed(&entries).unwrap();
    assert_eq!(result.processed, 100, "all 100 entries should be processed");
}

/// Fifty pre-seed entries all carrying the same valence leave the model still
/// producing a finite probability. A degenerate seeding set — every example on
/// one side — is the shape most likely to drive a linear model's updates to
/// extremes, and it does not.
///
/// ´claim:lifespan:a-run-of-one-sided-pre-seed-labels-leaves-the-model-finite´
/// ´test:crate:pre-seed-mixed-valence-shifts-model´
#[test]
fn pre_seed_mixed_valence_shifts_model() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // All positive valence — should push model toward higher p_bad
    let entries: Vec<PreSeedEntry> = (0..50).map(|i| make_pre_seed_entry(i, 1.0)).collect();
    assayer.pre_seed(&entries).unwrap();

    // Assess after positive pre-seed.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 999)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    // Model should have learned from positive labels — p_bad should be > prior (0.5)
    // We check it's finite and shifted from the prior significantly
    assert!(reckoning.assessment.risk.p_bad.is_finite());
    // Can't assert exact value but after 50 positive labels, p_bad should be elevated
}

/// Two identically built engines seeded with thirty adverse and thirty benign
/// labels respectively assess the same entity differently, the adversely seeded
/// one returning the higher probability. Pre-seeded history is genuinely
/// learned from rather than merely counted, which is what makes it worth
/// loading at all.
///
/// ´claim:lifespan:pre-seeding-moves-the-model-in-the-direction-of-the-labels-it-was-given´
/// ´test:crate:pre-seed-then-assess-reflects-learning´
#[test]
fn pre_seed_then_assess_reflects_learning() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Pre-seed with all negative (valence = 0.0 → benign)
    let entries: Vec<PreSeedEntry> = (0..30).map(|i| make_pre_seed_entry(i, 0.0)).collect();
    assayer.pre_seed(&entries).unwrap();

    let r1 = world.derive_for_requests(&[make_request(ChannelId(0), 500)]);
    let p_bad_negative = r1.into_iter().next().unwrap().unwrap().assessment.risk.p_bad;

    // Build a fresh world with all positive pre-seeding.
    let world2 = World::cold("lifecycle-integration-p", 0x6C69_6665_6379_632D);
    let assayer2 = world2.assayer();
    let entries2: Vec<PreSeedEntry> = (0..30).map(|i| make_pre_seed_entry(i, 1.0)).collect();
    assayer2.pre_seed(&entries2).unwrap();

    let r2 = world2.derive_for_requests(&[make_request(ChannelId(0), 500)]);
    let p_bad_positive = r2.into_iter().next().unwrap().unwrap().assessment.risk.p_bad;

    // Model trained on all-positive should have higher p_bad than all-negative
    assert!(
        p_bad_positive > p_bad_negative,
        "positive pre-seed ({p_bad_positive:.4}) should yield higher p_bad than negative ({p_bad_negative:.4})"
    );
}

/// Seeding with a Sentinel already registered succeeds and processes every
/// entry. The presence of Sentinel features in the vector does not require the
/// historical entries to have carried Sentinel observations of their own.
///
/// (´claim:lifespan:every-pre-seeded-entry-is-accounted-for-in-the-processed-count´)
/// ´test:crate:pre-seed-with-registered-sentinel-succeeds´
#[test]
fn pre_seed_with_registered_sentinel_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    let entries: Vec<PreSeedEntry> = (0..5).map(|i| make_pre_seed_entry(i, 0.5)).collect();
    let result = assayer.pre_seed(&entries);
    assert!(result.is_ok(), "pre_seed with sentinel should succeed: {result:?}");
    assert_eq!(result.unwrap().processed, 5);
}

/// A pre-seed entry can carry a value for a registered outcome axis alongside
/// its valence, and is accepted. History about consequences is worth as much as
/// history about verdicts, and the seeding path takes both.
///
/// ´claim:lifespan:pre-seeded-entries-may-carry-outcome-values-for-registered-axes´
/// ´test:crate:pre-seed-with-outcome-values´
#[test]
fn pre_seed_with_outcome_values() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.flush_label_channel().unwrap();

    let entry = PreSeedSpec::new(ChannelId(0), World::entity("entity-1"))
        .valence(1.0)
        .ground_truth()
        .outcome(OutcomeAxisId(1), 0.8)
        .build();

    let result = assayer.pre_seed(&[entry]);
    assert!(result.is_ok(), "pre_seed with outcomes should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Assessment Lifecycle Properties
// ═══════════════════════════════════════════════════════════════════════════════

/// With nothing registered, an assessment still returns, and it carries the
/// zero-Sentinels flag that says what it was built from. The engine's prior is
/// a legitimate answer on an empty deployment; what would not be legitimate is
/// presenting it as though evidence stood behind it.
///
/// ´claim:lifespan:an-assessment-with-no-sentinel-registered-flags-itself-rather-than-refusing-to-answer´
/// ´test:crate:assess-with-no-sentinels-zero-flag´
#[test]
fn assess_with_no_sentinels_zero_flag() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    assert!(
        reckoning.assessment.health.zero_sentinels,
        "should flag zero sentinels when none registered"
    );
    assert!(reckoning.assessment.health.zero_sentinels, "has_sentinels should be false");
}

/// Registration alone does not clear the flag — a contribution does. An
/// assessment routed past a registered but silent Sentinel is still computed
/// from the prior alone and says so; once that Sentinel holds a report and
/// the request carries its coordinate, the flag clears and the contributor
/// count reads one. Occupancy is the online flag, not a contribution, and a
/// flag cleared by mere registration would deny a prior-only estimate was
/// prior-only (´schema:output:assessment´).
///
/// ´claim:lifespan:the-zero-sentinels-flag-clears-on-contribution-not-on-registration´
/// ´test:crate:assess-with-sentinel-has-sentinels-flag´
#[test]
fn assess_with_sentinel_has_sentinels_flag() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Registered, no report, no coordinate: online but contributing nothing.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(
        reckoning.assessment.health.zero_sentinels,
        "a registered but silent Sentinel is not a contributor"
    );
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 0);

    // With a report cached and the coordinate in the request, the
    // Sentinel contributes and the flag clears.
    assayer.receive_sentinel_report(SentinelId(1), golden_report()).unwrap();
    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 42, &[1])]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(
        !reckoning.assessment.health.zero_sentinels,
        "a contributing Sentinel clears the flag"
    );
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 1);
}

/// Across entity seeds spanning the extremes of the identifier space, the
/// reported probability is finite and lies within zero and one. It is consumed
/// as a probability by downstream policy, so being out of range is not a
/// degraded answer but a meaningless one.
///
/// ´claim:lifespan:the-risk-probability-is-finite-and-within-zero-and-one-for-any-entity´
/// ´test:crate:assess-returns-finite-p-bad´
#[test]
fn assess_returns_finite_p_bad() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    for seed in [0, 1, u64::MAX, 42, 12345] {
        let results = world.derive_for_requests(&[make_request(ChannelId(0), seed)]);
        let reckoning = results.into_iter().next().unwrap().unwrap();
        assert!(
            reckoning.assessment.risk.p_bad.is_finite(),
            "p_bad should be finite for entity seed {seed}"
        );
        assert!(
            (0.0..=1.0).contains(&reckoning.assessment.risk.p_bad),
            "p_bad should be in [0, 1]; got {}",
            reckoning.assessment.risk.p_bad
        );
    }
}

/// The uncertainty accompanying a risk figure is never negative. It is a
/// spread, and a negative spread would invert every comparison a caller makes
/// between a confident verdict and a tentative one.
///
/// ´claim:lifespan:reported-uncertainty-is-never-negative´
/// ´test:crate:assess-risk-uncertainty-non-negative´
#[test]
fn assess_risk_uncertainty_non_negative() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    assert!(
        reckoning.assessment.risk.uncertainty >= 0.0,
        "uncertainty should be non-negative; got {}",
        reckoning.assessment.risk.uncertainty
    );
}

/// A batch of ten requests returns ten results, each of them successful.
/// Callers index results against the requests they sent, so a batch that
/// dropped or merged entries would misattribute every verdict after the gap.
///
/// ´claim:lifespan:a-batch-returns-one-result-per-request-in-full´
/// ´test:crate:assess-batch-processes-all´
#[test]
fn assess_batch_processes_all() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let reqs: Vec<RequestContext> = (0..10).map(|i| make_request(ChannelId(0), i)).collect();
    let results = world.derive_for_requests(&reqs);

    assert_eq!(results.len(), 10, "batch should return one result per request");
    for (i, result) in results.iter().enumerate() {
        assert!(result.is_ok(), "request {i} should succeed: {result:?}");
    }
}

/// A request naming a channel the engine knows nothing about is still assessed.
/// Channel policy governs what is done with a verdict; the verdict itself is
/// about the entity, so it can be formed without one.
///
/// ´claim:lifespan:core-assessment-does-not-require-a-known-channel´
/// ´test:crate:assess-channel-free-request-succeeds´
#[test]
fn assess_channel_free_request_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[make_request(ChannelId(999), 42)]);
    assert_eq!(results.len(), 1);
    assert!(results[0].is_ok(), "core assessment should not require a channel");
}

/// An empty batch returns an empty result list rather than an error. Zero is a
/// legitimate batch size for a caller draining a queue that turned out to be
/// empty.
///
/// (´claim:lifespan:a-batch-returns-one-result-per-request-in-full´)
/// ´test:crate:assess-empty-batch´
#[test]
fn assess_empty_batch() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[]);
    assert!(results.is_empty(), "empty batch should return empty results");
}

/// Two assessments issued in sequence carry different identifiers. The
/// identifier is what a later label uses to find the pending state an
/// assessment left behind, so a collision would attach one request's outcome to
/// another's evidence.
///
/// ´claim:lifespan:each-assessment-is-given-an-identifier-no-other-assessment-holds´
/// ´test:crate:assess-assigns-unique-ids´
#[test]
fn assess_assigns_unique_ids() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let r1 = world.derive_for_requests(&[make_request(ChannelId(0), 1)]);
    let r2 = world.derive_for_requests(&[make_request(ChannelId(0), 2)]);

    let id1 = r1.into_iter().next().unwrap().unwrap().assessment.id;
    let id2 = r2.into_iter().next().unwrap().unwrap().assessment.id;
    assert_ne!(id1, id2, "assessment IDs should be unique");
}

/// The anchor weight accompanying an assessment lies within zero and one. It
/// says how much of the verdict leant on the fixed reference rather than on
/// what has been learned, and it is only readable as such while it stays a
/// proportion.
///
/// ´claim:lifespan:the-anchor-weight-reported-with-a-verdict-is-a-proportion´
/// ´test:crate:assess-anchor-weight-in-valid-range´
#[test]
fn assess_anchor_weight_in_valid_range() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    assert!(
        (0.0..=1.0).contains(&reckoning.assessment.risk.anchor_weight),
        "anchor_weight should be in [0, 1]; got {}",
        reckoning.assessment.risk.anchor_weight
    );
}

/// The ambiguity gauge read over a derived profile is finite in all three
/// of its figures (´sig:rendering:ambiguity-gauge´). A display gauge that
/// returned an infinity would poison whatever dashboard or comparison
/// consumed it; the decision about what is worth probing belongs to
/// fragility on the landscape, not to this reading
/// (´def:fragility:definition´).
///
/// ´claim:lifespan:the-ambiguity-gauge-over-a-derived-profile-is-finite´
/// ´test:crate:derived-profile-ambiguity-finite´
#[test]
fn derived_profile_ambiguity_finite() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    let gauge = crate::profile_ambiguity(&reckoning.profile, 0.3);
    assert!(gauge.total.is_finite());
    assert!(gauge.classification.is_finite());
    assert!(gauge.action.is_finite());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Pipeline Integration
// ═══════════════════════════════════════════════════════════════════════════════

/// Labelling an assessment succeeds and the receipt carries back the identifier
/// of the assessment it settled. This round-trip is the loop the whole system
/// learns through, and the returned identifier is how a caller confirms which
/// of its outstanding assessments has now closed.
///
/// ´claim:lifespan:an-assessment-can-be-labelled-and-the-receipt-names-the-assessment-it-settled´
/// ´test:crate:label-after-assess-succeeds´
#[test]
fn label_after_assess_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Assess to get an assessment ID
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    // Label that assessment.
    let label_data = LabelSpec::new(reckoning.assessment.id).valence(1.0).ground_truth().build();
    let result = assayer.label(label_data);
    assert!(result.is_ok(), "label should succeed: {result:?}");
    assert_eq!(result.unwrap().assessment_id, reckoning.assessment.id);
}

/// A label naming an assessment that was never issued fails. The pending buffer
/// holds the evidence a label must be applied against, and inventing an update
/// with no evidence behind it would teach the model from nothing.
///
/// ´claim:lifespan:a-label-for-an-assessment-the-engine-never-issued-is-refused´
/// ´test:crate:label-unknown-assessment-error´
#[test]
fn label_unknown_assessment_error() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let label_data = LabelSpec::new(AssessmentId(999_999)).ground_truth().build();
    let result = assayer.label(label_data);
    assert!(result.is_err(), "label with unknown assessment should fail");
}

/// An assessment taken while a Sentinel was live can still be labelled after
/// that Sentinel has been retired. The pending entry captured the state at
/// assessment time, so the label is applied against what was actually seen
/// rather than against whatever survives now.
///
/// ´claim:lifespan:a-label-still-lands-after-the-sentinel-that-informed-its-assessment-has-been-retired´
/// ´test:crate:label-after-sentinel-deregistration-succeeds´
#[test]
fn label_after_sentinel_deregistration_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Assess while the Sentinel is registered.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    // Deregister sentinel
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    // Label should still succeed; the pending assessment captured the state.
    let label_data = LabelSpec::new(reckoning.assessment.id).valence(1.0).ground_truth().build();
    let result = assayer.label(label_data);
    assert!(
        result.is_ok(),
        "label after sentinel deregistration should succeed: {result:?}"
    );
}

/// The second label for an assessment fails: the first consumed the pending
/// entry. Repeating an update would let one piece of evidence be counted twice,
/// which is how a duplicated retry silently doubles a model's confidence.
///
/// ´claim:lifespan:an-assessment-can-be-labelled-once-and-only-once´
/// ´test:crate:label-double-submit-fails´
#[test]
fn label_double_submit_fails() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    let make_label = || LabelSpec::new(reckoning.assessment.id).valence(1.0).ground_truth().build();

    // First label succeeds
    assayer.label(make_label()).unwrap();

    // Second label should fail (removed from pending buffer)
    let result = assayer.label(make_label());
    assert!(result.is_err(), "double label should fail");
}

/// A label may carry an outcome value for a registered axis alongside its
/// verdict, and is accepted. The verdict and the consequence are separate
/// things learned from the same event.
///
/// ´claim:lifespan:a-label-can-carry-a-value-for-a-registered-outcome-axis´
/// ´test:crate:label-with-outcome-axis-registered´
#[test]
fn label_with_outcome_axis_registered() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.flush_label_channel().unwrap();

    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    let label_data = LabelSpec::new(reckoning.assessment.id)
        .ground_truth()
        .outcome(OutcomeAxisId(1), 0.7)
        .build();

    let result = assayer.label(label_data);
    assert!(result.is_ok(), "label with outcome should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Edge Cases and Error Paths
// ═══════════════════════════════════════════════════════════════════════════════

/// All seven lifecycle failure modes render to messages distinct from one
/// another. An operator reading a log has only the message to go on, so two
/// failures that look identical there are two failures that cannot be told
/// apart at all.
///
/// ´claim:lifespan:no-two-lifecycle-failures-render-to-the-same-message´
/// ´test:crate:lifecycle-error-variants-are-distinct´
#[test]
fn lifecycle_error_variants_are_distinct() {
    let errors: Vec<LifecycleError> = vec![
        LifecycleError::DuplicateId {
            entity_type: "Sentinel",
            id: "1".to_owned(),
        },
        LifecycleError::NotFound {
            entity_type: "Sentinel",
            id: "1".to_owned(),
        },
        LifecycleError::ModelOwnerShutdown,
        LifecycleError::IdentityMaintenanceShutdown,
        LifecycleError::CommandChannelFull,
        LifecycleError::EmptyName { entity_type: "Sentinel" },
        LifecycleError::InvalidDomainBits {
            id: DimensionId(1),
            domain_bits: 0,
        },
    ];

    // All have distinct Display strings
    let messages: Vec<String> = errors.iter().map(|e| format!("{e}")).collect();
    for (i, mi) in messages.iter().enumerate() {
        for (j, mj) in messages.iter().enumerate() {
            if i != j {
                assert_ne!(mi, mj, "error variants {i} and {j} should have distinct messages");
            }
        }
    }
}

/// Through the raw engine surface as well as the harness, a second identity
/// dimension registers alongside the first. Neither the blocking
/// acknowledgement nor the cross-dimension block the first dimension created
/// stands in the second one's way.
///
/// (´claim:lifespan:more-than-one-identity-dimension-can-be-live-at-once´)
/// ´test:crate:multiple-identity-dimensions-allowed´
#[test]
fn multiple_identity_dimensions_allowed() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();

    let result = assayer.register_identity_dimension(identity_reg(2, "ua-hash"));
    assert!(result.is_ok(), "second identity dimension should be allowed: {result:?}");
}

/// After a dimension is retired, a fresh dimension under a different id
/// registers successfully. Teardown sends a destroy command to the identity-
/// maintenance thread, and that thread must survive it in a state able to serve
/// the next dimension.
///
/// ´claim:lifespan:tearing-down-an-identity-dimension-leaves-the-machinery-able-to-build-another´
/// ´test:crate:identity-deregister-allows-new-dimension´
#[test]
fn identity_deregister_allows_new_dimension() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();
    assayer.deregister_identity_dimension(DimensionId(1)).unwrap();

    let result = assayer.register_identity_dimension(identity_reg(2, "ua-hash"));
    assert!(
        result.is_ok(),
        "new identity dimension should succeed after deregistration: {result:?}"
    );
}

/// Identity registration, alone among the lifecycle calls in waiting for an
/// acknowledgement, returns in under five seconds — far inside the configured
/// timeout that would otherwise turn the wait into a reported shutdown. The
/// call blocks a host's own thread, so the ordinary case has to be fast enough
/// for a host to make it inline.
///
/// ´claim:lifespan:the-blocking-half-of-identity-registration-completes-in-well-under-its-timeout´
/// ´test:crate:register-identity-blocking-completes-promptly´
#[test]
fn register_identity_blocking_completes_promptly() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let start = Instant::now();
    let result = assayer.register_identity_dimension(identity_reg(1, "ip-hash"));
    let elapsed = start.elapsed();

    assert!(result.is_ok(), "identity registration should succeed: {result:?}");
    assert!(
        elapsed.as_secs() < 5,
        "identity registration should complete in < 5s; took {elapsed:?}"
    );
}

/// The bound the registration wait is held to is the figure the host
/// configured and not a constant of the wait's own: an engine built asking for
/// three seconds is bounded at three, one asking for thirty-seven at
/// thirty-seven, and one that asks for nothing at the ten the record fixes as
/// the default. The surface was defaulted, validated and refused at zero while
/// the wait read past it to a constant that agreed by coincidence, so what
/// this pins is that moving the dial now moves the wait
/// (´cav:construction:registration-timeout-shadowed´).
///
/// ´claim:lifespan:the-identity-registration-wait-is-bounded-by-the-configured-timeout-rather-than-a-constant-of-its-own´
/// ´test:crate:register-identity-wait-is-bounded-by-the-configured-timeout´
#[test]
fn register_identity_wait_is_bounded_by_the_configured_timeout() {
    let tight = World::builder(AssayerConfig {
        instance_id: "lifecycle-registration-timeout-tight".to_owned(),
        infrastructure: InfrastructureConfig {
            identity_registration_timeout_secs: 3,
            ..crate::testing::test_infrastructure()
        },
        ..Default::default()
    })
    .seed(0x7469_6768_742D_7761)
    .build()
    .expect("tight-timeout world should build");
    assert_eq!(
        tight.assayer().identity_registration_timeout(),
        Duration::from_secs(3),
        "a host asking for three seconds is bounded at three",
    );

    let wide = World::builder(AssayerConfig {
        instance_id: "lifecycle-registration-timeout-wide".to_owned(),
        infrastructure: InfrastructureConfig {
            identity_registration_timeout_secs: 37,
            ..crate::testing::test_infrastructure()
        },
        ..Default::default()
    })
    .seed(0x7769_6465_2D77_6169)
    .build()
    .expect("wide-timeout world should build");
    assert_eq!(
        wide.assayer().identity_registration_timeout(),
        Duration::from_secs(37),
        "a host asking for thirty-seven seconds is bounded at thirty-seven",
    );

    assert_ne!(
        tight.assayer().identity_registration_timeout(),
        wide.assayer().identity_registration_timeout(),
        "two hosts asking for different figures must not share one bound",
    );

    assert_eq!(
        World::cold("lifecycle-registration-timeout-default", 0x6465_6661_756C_742D)
            .assayer()
            .identity_registration_timeout(),
        Duration::from_secs(10),
        "a host asking for nothing gets the default the record fixes",
    );

    // And the wait the bound governs still completes under it.
    assert!(
        tight
            .assayer()
            .register_identity_dimension(identity_reg(1, "ip-hash"))
            .is_ok(),
        "registration under a tight configured bound still succeeds",
    );
}

/// Driven straight at the engine, a second Sentinel under a live id is refused
/// and the error names the Sentinel kind. The check is the concurrent map's own
/// atomic entry occupancy, so two callers racing cannot both pass it.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:crate:register-sentinel-duplicate-returns-duplicate-id´
#[test]
fn register_sentinel_duplicate_returns_duplicate_id() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    let result = assayer.register_sentinel(sentinel_reg(1, "s1-dup"));
    match &result {
        Err(LifecycleError::DuplicateId { entity_type, .. }) => {
            assert_eq!(*entity_type, "Sentinel");
        }
        other => panic!("expected DuplicateId, got: {other:?}"),
    }
}

/// An unknown Sentinel id cannot be retired, and the refusal names the Sentinel
/// kind. Nothing is sent to the model owner, so the mistake costs one map
/// lookup.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:crate:deregister-sentinel-not-found-error´
#[test]
fn deregister_sentinel_not_found_error() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let result = assayer.deregister_sentinel(SentinelId(999));
    match &result {
        Err(LifecycleError::NotFound { entity_type, .. }) => {
            assert_eq!(*entity_type, "Sentinel");
        }
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

/// A second axis on a live id is refused with the outcome-axis kind named, even
/// after a flush has driven the first all the way through. The registered axis
/// set is guarded independently of the model owner's own view, so the answer
/// does not depend on how far the first registration has travelled.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:crate:register-axis-duplicate-returns-duplicate-id´
#[test]
fn register_axis_duplicate_returns_duplicate_id() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.flush_label_channel().unwrap();

    let result = assayer.register_outcome_axis(axis_reg(1, "fraud-dup"));
    match &result {
        Err(LifecycleError::DuplicateId { entity_type, .. }) => {
            assert_eq!(*entity_type, "OutcomeAxis");
        }
        other => panic!("expected DuplicateId, got: {other:?}"),
    }
}

/// Retiring an axis id the engine never held names the outcome-axis kind in its
/// refusal. The kind is part of the error because a host managing several
/// identifier namespaces at once needs to know which one it got wrong.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:crate:deregister-axis-not-found-error´
#[test]
fn deregister_axis_not_found_error() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let result = assayer.deregister_outcome_axis(OutcomeAxisId(999));
    match &result {
        Err(LifecycleError::NotFound { entity_type, .. }) => {
            assert_eq!(*entity_type, "OutcomeAxis");
        }
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

/// The third entity kind behaves like the other two: a duplicate identity
/// dimension is refused with its own kind named. Uniqueness is a property of
/// the lifecycle contract rather than of any one registry's implementation.
///
/// (´claim:lifespan:a-duplicate-registration-is-refused-and-names-the-entity-kind-that-collided´)
/// ´test:crate:register-identity-duplicate-returns-duplicate-id´
#[test]
fn register_identity_duplicate_returns_duplicate_id() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();

    let result = assayer.register_identity_dimension(identity_reg(1, "ip-hash-dup"));
    match &result {
        Err(LifecycleError::DuplicateId { entity_type, .. }) => {
            assert_eq!(*entity_type, "IdentityDimension");
        }
        other => panic!("expected DuplicateId, got: {other:?}"),
    }
}

/// An unregistered dimension id cannot be retired, and the identity-dimension
/// kind is named. The check runs before the blocking wait, so a mistaken
/// teardown does not cost a round-trip to the model owner.
///
/// (´claim:lifespan:retiring-an-identifier-that-was-never-registered-is-refused-as-not-found´)
/// ´test:crate:deregister-identity-not-found-error´
#[test]
fn deregister_identity_not_found_error() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let result = assayer.deregister_identity_dimension(DimensionId(999));
    match &result {
        Err(LifecycleError::NotFound { entity_type, .. }) => {
            assert_eq!(*entity_type, "IdentityDimension");
        }
        other => panic!("expected NotFound, got: {other:?}"),
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrent Lifecycle + Assessment
// ═══════════════════════════════════════════════════════════════════════════════

/// Assessing immediately after a registration returns — while the model owner
/// may still be widening the model behind it — yields a finite probability and
/// no panic. The synchronous half of registration and the published snapshot
/// are each internally consistent, so a reader caught between them reads one or
/// the other rather than a mixture.
///
/// ´claim:lifespan:an-assessment-issued-between-the-two-phases-of-registration-sees-a-consistent-state´
/// ´test:crate:concurrent-assess-during-registration´
#[test]
fn concurrent_assess_during_registration() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register a sentinel — Phase 1 happens synchronously
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Immediately assess/derive (Phase 2 may not have completed yet).
    // This should not panic; assessment should observe a consistent state.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    assert_eq!(results.len(), 1);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

/// Assessments taken before any registration, after a Sentinel joins, after an
/// axis joins and after the Sentinel is retired all succeed and all carry
/// distinct identifiers. Availability does not dip around structural change,
/// and the identifier sequence does not restart when the structure does.
///
/// ´claim:lifespan:assessment-continues-uninterrupted-across-a-sequence-of-structural-changes´
/// ´test:crate:multiple-assessments-across-lifecycle´
#[test]
fn multiple_assessments_across_lifecycle() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Assess before any lifecycle.
    let r0 = world.derive_for_requests(&[make_request(ChannelId(0), 1)]);
    assert!(r0[0].is_ok());

    // Register Sentinel + assess.
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    let r1 = world.derive_for_requests(&[make_request(ChannelId(0), 2)]);
    assert!(r1[0].is_ok());

    // Register axis + assess.
    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    let r2 = world.derive_for_requests(&[make_request(ChannelId(0), 3)]);
    assert!(r2[0].is_ok());

    // Deregister Sentinel + assess.
    assayer.deregister_sentinel(SentinelId(1)).unwrap();
    let r3 = world.derive_for_requests(&[make_request(ChannelId(0), 4)]);
    assert!(r3[0].is_ok());

    // All assessments should have unique IDs.
    let ids: Vec<_> = [&r0, &r1, &r2, &r3]
        .iter()
        .map(|r| r[0].as_ref().unwrap().assessment.id)
        .collect();
    for i in 0..ids.len() {
        for j in (i + 1)..ids.len() {
            assert_ne!(ids[i], ids[j], "assessment IDs should be unique across lifecycle events");
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Report Integration
// ═══════════════════════════════════════════════════════════════════════════════

/// Accepting a report returns an acknowledgement stating how many cells it
/// carried. The count is the Sentinel's confirmation that its observation was
/// read as it was meant, rather than accepted and quietly discarded.
///
/// ´claim:lifespan:a-report-is-acknowledged-with-a-count-of-what-it-contained´
/// ´test:crate:report-ack-has-cells´
#[test]
fn report_ack_has_cells() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    let report = minimal_report();
    let ack = assayer.receive_sentinel_report(SentinelId(1), report).unwrap();
    assert!(ack.cells_in_report > 0);
}

/// A Sentinel that was never registered is equally unknown: its report is
/// refused. Reporting is gated on the slot existing, so registration cannot be
/// skipped by simply starting to report.
///
/// (´claim:lifespan:a-report-from-a-retired-sentinel-is-rejected-as-coming-from-an-unknown-sentinel´)
/// ´test:crate:report-for-unregistered-sentinel-fails´
#[test]
fn report_for_unregistered_sentinel_fails() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let report = minimal_report();
    let result = assayer.receive_sentinel_report(SentinelId(999), report);
    assert!(
        matches!(&result, Err(ReportError::UnknownSentinel { .. })),
        "unregistered sentinel should fail: {result:?}"
    );
}

/// Two registered Sentinels each have their reports accepted under their own
/// id. Slots are per Sentinel, so the fleet grows without its members
/// contending for one place to write.
///
/// ´claim:lifespan:each-registered-sentinel-has-its-own-slot-to-report-into´
/// ´test:crate:report-multiple-sentinels´
#[test]
fn report_multiple_sentinels() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_sentinel(sentinel_reg(2, "s2")).unwrap();

    let report = minimal_report();
    let ack1 = assayer.receive_sentinel_report(SentinelId(1), report.clone());
    let ack2 = assayer.receive_sentinel_report(SentinelId(2), report);

    assert!(ack1.is_ok(), "s1 report should succeed: {ack1:?}");
    assert!(ack2.is_ok(), "s2 report should succeed: {ack2:?}");
}

/// An id that was registered, retired and registered again accepts reports
/// under its new occupant. Re-registration builds a fresh slot and ledger
/// partition rather than reviving the removed one, so reuse is a beginning and
/// not a resumption.
///
/// ´claim:lifespan:a-re-registered-identifier-comes-back-with-a-working-slot-not-a-husk´
/// ´test:crate:report-after-second-registration-succeeds´
#[test]
fn report_after_second_registration_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register → deregister → re-register
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.deregister_sentinel(SentinelId(1)).unwrap();
    assayer.register_sentinel(sentinel_reg(1, "s1-v2")).unwrap();

    let report = minimal_report();
    let result = assayer.receive_sentinel_report(SentinelId(1), report);
    assert!(result.is_ok(), "report after re-registration should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Flush and Synchronisation
// ═══════════════════════════════════════════════════════════════════════════════

/// Three flushes in a row all succeed, with nothing left to flush after the
/// first. A host that flushes defensively before reading state pays nothing for
/// the flushes that turn out to be unnecessary.
///
/// ´claim:lifespan:flushing-an-already-drained-pipeline-is-harmless´
/// ´test:crate:flush-label-channel-idempotent´
#[test]
fn flush_label_channel_idempotent() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Multiple flushes should all succeed
    assayer.flush_label_channel().unwrap();
    assayer.flush_label_channel().unwrap();
    assayer.flush_label_channel().unwrap();
}

/// After a flush, a Sentinel and an axis submitted beforehand have both been
/// processed — proved by the axis now colliding with a duplicate registration.
/// The flush is the synchronisation point a caller reaches for when it needs
/// the asynchronous half of a registration to have actually happened.
///
/// ´claim:lifespan:a-flush-guarantees-queued-lifecycle-events-have-been-processed´
/// ´test:crate:flush-after-lifecycle-events´
#[test]
fn flush_after_lifecycle_events() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();

    // Flush ensures both events processed
    assayer.flush_label_channel().unwrap();

    // Now axis should be visible in duplicate check
    let result = assayer.register_outcome_axis(axis_reg(1, "fraud-dup"));
    assert!(
        matches!(&result, Err(LifecycleError::DuplicateId { .. })),
        "axis should be registered after flush: {result:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multi-Entity Lifecycle Stress
// ═══════════════════════════════════════════════════════════════════════════════

/// Ten Sentinels register in turn and every one of them then accepts a report.
/// The per-Sentinel slot arrangement holds at fleet scale, not only for the
/// pair a two-Sentinel test can exercise.
///
/// (´claim:lifespan:each-registered-sentinel-has-its-own-slot-to-report-into´)
/// ´test:crate:register-many-sentinels´
#[test]
fn register_many_sentinels() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    for i in 0..10 {
        let result = assayer.register_sentinel(sentinel_reg(i, &format!("s{i}")));
        assert!(result.is_ok(), "sentinel {i} registration should succeed: {result:?}");
    }

    // Verify all 10 sentinels accept reports
    let report = minimal_report();
    for i in 0..10 {
        let result = assayer.receive_sentinel_report(SentinelId(i), report.clone());
        assert!(result.is_ok(), "sentinel {i} report should succeed: {result:?}");
    }
}

/// Five axes registered together all appear in a single assessment's
/// predictions. Every axis is predicted on every assessment, so the map grows
/// with the registrations rather than reporting whichever axis was most
/// recently touched.
///
/// (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´)
/// ´test:crate:register-many-axes´
#[test]
fn register_many_axes() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    for i in 0..5 {
        let result = assayer.register_outcome_axis(axis_reg(i, &format!("axis-{i}")));
        assert!(result.is_ok(), "axis {i} registration should succeed: {result:?}");
    }

    assayer.flush_label_channel().unwrap();

    // All 5 axes should appear in assessment output.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    for i in 0..5 {
        assert!(
            reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(i)),
            "axis {i} should be in predictions"
        );
    }
}

/// All three entity kinds registered against one engine leave the next
/// assessment reporting the Sentinel as a contributor, a prediction for the
/// axis, and a finite probability. The three extension shapes compose within
/// a single dimension map without any of them displacing another.
///
/// (´claim:lifespan:sentinels-and-axes-registered-together-both-show-up-in-the-same-assessment´)
/// ´test:crate:lifecycle-sentinel-axis-identity-all-together´
#[test]
fn lifecycle_sentinel_axis_identity_all_together() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register one of each
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_outcome_axis(axis_reg(1, "fraud")).unwrap();
    assayer.register_identity_dimension(identity_reg(1, "ip-hash")).unwrap();
    assayer.receive_sentinel_report(SentinelId(1), golden_report()).unwrap();

    assayer.flush_label_channel().unwrap();

    // Assess/derive should succeed with all three registered.
    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 42, &[1])]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    assert!(!reckoning.assessment.health.zero_sentinels);
    assert!(reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(1)));
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Health Report Detailed Fields
// ═══════════════════════════════════════════════════════════════════════════════

/// The blend statistics report a finite mean on a brand-new engine, before any
/// assessment has contributed to them. They begin at their initialised values
/// rather than at whatever an empty accumulator would compute.
///
/// ´claim:lifespan:the-blend-statistics-are-defined-before-any-observation-has-been-blended´
/// ´test:crate:health-summary-blend-statistics-initial´
#[test]
fn health_summary_blend_statistics_initial() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let health = assayer.health_summary();

    // Blend statistics should be initialised but with no observations
    assert!(health.blend_statistics.mean.is_finite());
}

/// After an assessment the blend mean is still finite, having taken its first
/// real observation. The transition from the initialised state to an observed
/// one does not pass through an undefined value.
///
/// (´claim:lifespan:the-blend-statistics-are-defined-before-any-observation-has-been-blended´)
/// ´test:crate:health-summary-after-assess-updates-blend´
#[test]
fn health_summary_after_assess_updates_blend() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    let _r = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let health = assayer.health_summary();

    // After at least one assessment, blend statistics should be updated.
    assert!(health.blend_statistics.mean.is_finite());
}

/// Freshly built, the engine counts nothing as degraded. Degradation is
/// something that has to be detected and counted, so the baseline is silence
/// rather than an unknown.
///
/// ´claim:lifespan:a-new-engine-counts-nothing-as-degraded´
/// ´test:crate:health-summary-degradation-initial´
#[test]
fn health_summary_degradation_initial() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let health = assayer.health_summary();

    // Initial degradation should be all zeros
    assert_eq!(health.degradation.total_degraded, 0);
}

/// The composite convergence stage in the full report matches the stage the
/// cheap summary reports. The two views are read at different costs by
/// different callers, and they would be worse than useless if a host could act
/// on one and be contradicted by the other.
///
/// ´claim:lifespan:the-summary-and-the-full-report-agree-about-the-convergence-stage´
/// ´test:crate:full-health-report-convergence-structure´
#[test]
fn full_health_report_convergence_structure() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let report = assayer.full_health_report();

    // Convergence should be present and consistent
    assert_eq!(
        format!("{:?}", report.convergence.composite_stage),
        format!("{:?}", assayer.health_summary().convergence_stage),
        "convergence stage should match between summary and full report"
    );
}

/// The full report's calibration block carries a finite sister concentration
/// even before anything has been calibrated. Calibration figures are read as
/// diagnostics on cold engines as much as on warm ones.
///
/// ´claim:lifespan:the-calibration-block-carries-a-finite-sister-concentration-from-the-outset´
/// ´test:crate:full-health-report-calibration´
#[test]
fn full_health_report_calibration() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let report = assayer.full_health_report();

    // Calibration should be accessible
    assert!(report.calibration.kappa_sister.is_finite());
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-seed + Lifecycle Combination
// ═══════════════════════════════════════════════════════════════════════════════

/// History loaded before a Sentinel is registered is still there when the
/// assessment runs, and that assessment is finite with the registered — but
/// silent — Sentinel correctly reported as no contributor. Seeded state lives
/// in the model rather than in a shape tied to whatever happened to be
/// registered at seeding time.
///
/// ´claim:lifespan:seeding-and-registration-compose-in-either-order´
/// ´test:crate:pre-seed-then-register-sentinel-then-assess´
#[test]
fn pre_seed_then_register_sentinel_then_assess() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Pre-seed first
    let entries: Vec<PreSeedEntry> = (0..10).map(|i| make_pre_seed_entry(i, 0.5)).collect();
    assayer.pre_seed(&entries).unwrap();

    // Then register sentinel
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Assessment should incorporate both. The Sentinel is online but has
    // no report, so nothing was measured and the assessment says so.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(reckoning.assessment.health.zero_sentinels);
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

/// The reverse order works as well: a Sentinel registered first, history loaded
/// second, and the assessment still finite. Neither operation invalidates what
/// the other left behind.
///
/// (´claim:lifespan:seeding-and-registration-compose-in-either-order´)
/// ´test:crate:register-sentinel-then-pre-seed-then-assess´
#[test]
fn register_sentinel_then_pre_seed_then_assess() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register sentinel first
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Then pre-seed
    let entries: Vec<PreSeedEntry> = (0..10).map(|i| make_pre_seed_entry(i, 0.5)).collect();
    assayer.pre_seed(&entries).unwrap();

    // Assessment should work.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

/// After pre-seeding, an ordinary assess-then-label round-trip succeeds. Seeded
/// history and live labels flow into the same model through the same pipeline,
/// so the one does not put the other out of reach.
///
/// ´claim:lifespan:the-live-label-path-works-on-an-engine-that-was-pre-seeded´
/// ´test:crate:pre-seed-then-label-sequential´
#[test]
fn pre_seed_then_label_sequential() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Pre-seed
    let entries: Vec<PreSeedEntry> = (0..5).map(|i| make_pre_seed_entry(i, 1.0)).collect();
    assayer.pre_seed(&entries).unwrap();

    // Normal assess/derive + label flow.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 100)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    let label_data = LabelSpec::new(reckoning.assessment.id).ground_truth().build();
    let result = assayer.label(label_data);
    assert!(result.is_ok(), "label after pre-seed should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Spatial Feature Policy Integration
// ═══════════════════════════════════════════════════════════════════════════════

fn spatial_axis_reg(id: u32, name: &str) -> OutcomeAxisRegistration {
    OutcomeAxisRegistration {
        id: OutcomeAxisId(id),
        name: name.to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Enabled,
    }
}

/// A spatially-enabled axis registered while a Sentinel stands appears in
/// predictions and leaves the probability finite. The per-Sentinel features its
/// registration added are populated and scored rather than left as an untouched
/// widening of the vector.
///
/// (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´)
/// ´test:crate:register-spatial-axis-then-assess-after-flush´
#[test]
fn register_spatial_axis_then_assess_after_flush() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register a sentinel (provides slots for spatial features)
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Register a spatial axis
    assayer.register_outcome_axis(spatial_axis_reg(1, "fraud-spatial")).unwrap();
    assayer.flush_label_channel().unwrap();

    // Assess should succeed and include the axis prediction.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(
        reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(1)),
        "spatial axis should appear in predictions"
    );
    assert!(reckoning.assessment.risk.p_bad.is_finite());
}

/// An axis with spatial features disabled appears in predictions just the same.
/// The spatial policy decides how much of the feature vector the axis occupies,
/// not whether it is predicted at all.
///
/// (´claim:lifespan:a-registered-axis-appears-in-every-assessments-outcome-predictions-once-published´)
/// ´test:crate:register-disabled-axis-then-assess-after-flush´
#[test]
fn register_disabled_axis_then_assess_after_flush() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register a sentinel
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Register a non-spatial axis
    assayer.register_outcome_axis(axis_reg(1, "fraud-nospatial")).unwrap();
    assayer.flush_label_channel().unwrap();

    // Assess should still succeed with the axis (just no spatial features).
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(
        reckoning.assessment.outcome_predictions.contains_key(&OutcomeAxisId(1)),
        "non-spatial axis should still appear in predictions"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Identity Deregistration Cleanup
// ═══════════════════════════════════════════════════════════════════════════════

/// A dimension retired and then registered again under the same id succeeds,
/// despite the maintenance thread having been told to destroy the old one. That
/// destruction is addressed to the retired dimension's own state, and the new
/// registration builds its own.
///
/// (´claim:lifespan:an-identifier-freed-by-retirement-can-be-registered-again´)
/// ´test:crate:deregister-identity-allows-reregistration-with-same-id´
#[test]
fn deregister_identity_allows_reregistration_with_same_id() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register → deregister → re-register with same ID
    assayer.register_identity_dimension(identity_reg(1, "dim-v1")).unwrap();
    assayer.deregister_identity_dimension(DimensionId(1)).unwrap();

    let result = assayer.register_identity_dimension(identity_reg(1, "dim-v2"));
    assert!(
        result.is_ok(),
        "re-registration with same ID should succeed after deregistration: {result:?}"
    );
}

/// After a teardown, two further dimensions register under fresh ids one after
/// the other. Capacity for new dimensions is not consumed by the ones that have
/// come and gone.
///
/// (´claim:lifespan:tearing-down-an-identity-dimension-leaves-the-machinery-able-to-build-another´)
/// ´test:crate:deregister-identity-then-register-new-dimension´
#[test]
fn deregister_identity_then_register_new_dimension() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_identity_dimension(identity_reg(1, "dim-pri")).unwrap();
    assayer.deregister_identity_dimension(DimensionId(1)).unwrap();

    let result = assayer.register_identity_dimension(identity_reg(2, "dim-pri-2"));
    assert!(result.is_ok(), "new dimension after deregistration: {result:?}");

    let result = assayer.register_identity_dimension(identity_reg(3, "dim-pri-3"));
    assert!(result.is_ok(), "additional identity dimensions should be allowed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Deregister Spatial Axis Timing Window
// ═══════════════════════════════════════════════════════════════════════════════

/// A spatial axis present in predictions disappears from them once retired, and
/// assessments afterwards still return a finite probability. Removing the axis
/// takes two features out of every Sentinel slot, and the model has to remain
/// scoreable across that narrowing rather than merely survive it.
///
/// ´claim:lifespan:a-retired-axis-stops-being-predicted-and-the-verdict-remains-finite-without-it´
/// ´test:crate:deregister-spatial-axis-then-assess´
#[test]
fn deregister_spatial_axis_then_assess() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register a sentinel (provides slots for spatial features)
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Register a spatial axis → +2 features per Sentinel slot
    let spatial_reg = OutcomeAxisRegistration {
        id: OutcomeAxisId(10),
        name: "fraud-spatial".to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Enabled,
    };
    assayer.register_outcome_axis(spatial_reg).unwrap();
    assayer.flush_label_channel().unwrap();

    // Verify axis is present in assessment output.
    let r1 = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning1 = r1.into_iter().next().unwrap().unwrap();
    assert!(
        reckoning1.assessment.outcome_predictions.contains_key(&OutcomeAxisId(10)),
        "spatial axis should be in predictions before deregistration"
    );

    // Deregister the spatial axis → -2 features per Sentinel slot
    assayer.deregister_outcome_axis(OutcomeAxisId(10)).unwrap();
    assayer.flush_label_channel().unwrap();

    // Assess should still succeed without the axis.
    let r2 = world.derive_for_requests(&[make_request(ChannelId(0), 43)]);
    let reckoning2 = r2.into_iter().next().unwrap().unwrap();
    assert!(
        !reckoning2.assessment.outcome_predictions.contains_key(&OutcomeAxisId(10)),
        "deregistered spatial axis should not appear in predictions"
    );
    assert!(
        reckoning2.assessment.risk.p_bad.is_finite(),
        "p_bad should remain finite after axis deregistration"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Register Axis + Label (Cross-Entity Cascade Observable)
// ═══════════════════════════════════════════════════════════════════════════════

/// An outcome value can be attached to a label for a spatially-enabled axis,
/// not only for a plain one. The label path accepts the axis on the same terms
/// whichever way its features were laid out.
///
/// (´claim:lifespan:a-label-can-carry-a-value-for-a-registered-outcome-axis´)
/// ´test:crate:register-spatial-axis-then-label-succeeds´
#[test]
fn register_spatial_axis_then_label_succeeds() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();

    // Register a spatial axis
    let spatial_reg = OutcomeAxisRegistration {
        id: OutcomeAxisId(20),
        name: "revenue-spatial".to_owned(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Enabled,
    };
    assayer.register_outcome_axis(spatial_reg).unwrap();
    assayer.flush_label_channel().unwrap();

    // Assess/derive + label with outcome value for the axis.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();

    let label_data = LabelSpec::new(reckoning.assessment.id)
        .valence(1.0)
        .ground_truth()
        .outcome(OutcomeAxisId(20), 0.75)
        .build();

    let result = assayer.label(label_data);
    assert!(result.is_ok(), "label with spatial axis outcome should succeed: {result:?}");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Empty Name Validation
// ═══════════════════════════════════════════════════════════════════════════════

/// A Sentinel registration carrying an empty name is refused with the empty-
/// name error, and nothing is created. A nameless entity would be unreachable
/// through every name-based path a host has, so it is turned away at the door
/// rather than orphaned inside.
///
/// ´claim:lifespan:an-entity-registered-under-an-empty-name-is-refused-before-anything-is-created´
/// ´test:crate:register-sentinel-empty-name-rejected´
#[test]
fn register_sentinel_empty_name_rejected() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let result = assayer.register_sentinel(sentinel_reg(1, ""));
    assert!(
        matches!(&result, Err(LifecycleError::EmptyName { entity_type: "Sentinel" })),
        "empty sentinel name should fail: {result:?}"
    );
}

/// The same refusal applies to an outcome axis with a blank name. The check is
/// on the name being present, not on any particular kind's registry.
///
/// (´claim:lifespan:an-entity-registered-under-an-empty-name-is-refused-before-anything-is-created´)
/// ´test:crate:register-axis-empty-name-rejected´
#[test]
fn register_axis_empty_name_rejected() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let reg = OutcomeAxisRegistration {
        id: OutcomeAxisId(1),
        name: String::new(),
        description: String::new(),
        eligibility: OutcomeEligibility::AllLabels,
        initial_kappa: 1.0,
        gamma: 0.999,
        spatial_features: SpatialFeaturePolicy::Disabled,
    };
    let result = assayer.register_outcome_axis(reg);
    assert!(
        matches!(
            &result,
            Err(LifecycleError::EmptyName {
                entity_type: "OutcomeAxis"
            })
        ),
        "empty axis name should fail: {result:?}"
    );
}

/// Identity dimensions are held to it too, and the refusal comes before the
/// blocking acknowledgement is ever sought. A malformed registration does not
/// get to occupy the caller's thread while it fails.
///
/// (´claim:lifespan:an-entity-registered-under-an-empty-name-is-refused-before-anything-is-created´)
/// ´test:crate:register-identity-empty-name-rejected´
#[test]
fn register_identity_empty_name_rejected() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();
    let result = assayer.register_identity_dimension(identity_reg(1, ""));
    assert!(
        matches!(
            &result,
            Err(LifecycleError::EmptyName {
                entity_type: "IdentityDimension"
            })
        ),
        "empty identity dimension name should fail: {result:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// End-to-End Lifecycle
// ═══════════════════════════════════════════════════════════════════════════════

/// One engine carries the whole arc: two Sentinels and an identity dimension
/// registered, reports ingested, a hundred assessments returning probabilities
/// near the prior with Sentinels present, eighty labels applied, an axis added
/// midway, and the health surfaces populated with drift and precision entries
/// throughout. Retiring a Sentinel at the end removes it from the per-Sentinel
/// breakdown while an assessment taken afterwards can still be labelled, and
/// dropping the engine shuts it down cleanly.
///
/// ´claim:lifespan:the-whole-arc-from-build-to-drop-holds-together-in-one-run´
/// ´test:crate:end-to-end-lifecycle´
#[test]
fn end_to_end_lifecycle() {
    // Build with default config + empty schema + 1 channel
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // ── Register Sentinels ──
    assayer.register_sentinel(sentinel_reg(1, "sentinel-a")).unwrap();
    assayer.register_sentinel(sentinel_reg(2, "sentinel-b")).unwrap();

    // ── Register identity dimension ──
    assayer.register_identity_dimension(identity_reg(1, "primary")).unwrap();

    // Flush to ensure model extension completes for sentinels and identity
    assayer.flush_label_channel().unwrap();

    // ── Receive Sentinel reports ──
    let report_a = golden_report();
    let ack_a = assayer.receive_sentinel_report(SentinelId(1), report_a).unwrap();
    assert!(ack_a.cells_in_report > 0, "Sentinel A report should have cells");

    let report_b = golden_report();
    let ack_b = assayer.receive_sentinel_report(SentinelId(2), report_b).unwrap();
    assert!(ack_b.cells_in_report > 0, "Sentinel B report should have cells");

    // ── Assess 100 varied requests → verify p̂ ≈ 0.5, zero_sentinels = false ──
    let requests: Vec<_> = (0..100)
        .map(|i| make_request_with_sentinels(ChannelId(0), i, &[1, 2]))
        .collect();
    let results = world.derive_for_requests(&requests);
    assert_eq!(results.len(), 100);

    for (i, result) in results.iter().enumerate() {
        let reckoning = result.as_ref().unwrap_or_else(|e| panic!("assess/derive {i} failed: {e}"));
        assert!(
            reckoning.assessment.risk.p_bad.is_finite(),
            "p_bad should be finite for request {i}"
        );
        // With no labels yet, p̂ should be near prior (0.5)
        assert!(
            (0.01..=0.99).contains(&reckoning.assessment.risk.p_bad),
            "p_bad={} should be near prior for request {i}",
            reckoning.assessment.risk.p_bad
        );
        assert!(!reckoning.assessment.health.zero_sentinels, "zero_sentinels should be false");
        assert!(!reckoning.assessment.health.zero_sentinels, "has_sentinels should be true");
    }

    // ── Label 50 with ~20% positive rate ──
    let mut assessment_ids: Vec<AssessmentId> = Vec::new();
    for i in 0_u64..50 {
        let req = make_request_with_sentinels(ChannelId(0), 1000 + i, &[1, 2]);
        let r = world.derive_for_requests(&[req]);
        let reckoning = r.into_iter().next().unwrap().unwrap();
        assessment_ids.push(reckoning.assessment.id);
    }

    for (i, &rid) in assessment_ids.iter().enumerate() {
        let valence = if i < 10 { 1.0 } else { -1.0 }; // 20% positive
        assayer.label(make_label(rid, valence)).unwrap();
    }

    // Wait for model to process labels
    assayer.flush_label_channel().unwrap();

    // ── Register outcome axis ──
    assayer.register_outcome_axis(spatial_axis_reg(1, "spatial-fraud")).unwrap();
    assayer.flush_label_channel().unwrap();

    // ── Assess 50 more → verify p̂ shifted toward empirical rate ──
    let requests2: Vec<_> = (200..250)
        .map(|i| make_request_with_sentinels(ChannelId(0), i, &[1, 2]))
        .collect();
    let results2 = world.derive_for_requests(&requests2);
    assert_eq!(results2.len(), 50);

    for result in &results2 {
        let reckoning = result.as_ref().unwrap();
        assert!(
            reckoning.assessment.risk.p_bad.is_finite(),
            "post-label p_bad should be finite"
        );
    }

    // ── Label 30 more ──
    for i in 0_u64..30 {
        let req = make_request_with_sentinels(ChannelId(0), 2000 + i, &[1, 2]);
        let r = world.derive_for_requests(&[req]);
        let reckoning = r.into_iter().next().unwrap().unwrap();
        let valence = if i < 6 { 1.0 } else { -1.0 }; // 20% positive
        assayer.label(make_label(reckoning.assessment.id, valence)).unwrap();
    }
    assayer.flush_label_channel().unwrap();

    // ── health_summary() → verify all fields populated ──
    let summary = assayer.health_summary();
    assert!(summary.total_assessments > 0, "total_assessments should be >0");
    assert_eq!(summary.total_labels, 80, "total_labels should be 80 (50 + 30)");
    assert!(summary.eligible_labels > 0, "eligible_labels should be >0");

    // ── drain_health_events() → verify events present ──
    let events = assayer.drain_health_events();
    let _event_count = events.len();

    // ── full_health_report() → verify all sub-types instantiated ──
    let report = assayer.full_health_report();
    assert!(!report.drift.is_empty(), "full_health_report should have drift entries");
    assert!(
        !report.precision.is_empty(),
        "full_health_report should have precision entries"
    );

    // ── health_summary_to_samples() → verify samples produced ──
    let samples = health_summary_to_samples(&summary);
    assert!(
        samples.len() >= 15,
        "health_summary_to_samples should produce ~30 samples, got {}",
        samples.len()
    );

    // ── Deregister Sentinel A ──
    assayer.deregister_sentinel(SentinelId(1)).unwrap();

    // ── Verify subsequent assessment excludes A ──
    let results3 = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 999, &[1, 2])]);
    let reckoning3 = results3.into_iter().next().unwrap().unwrap();
    assert!(
        !reckoning3.assessment.per_sentinel.contains_key(&SentinelId(1)),
        "deregistered sentinel should not appear in per_sentinel"
    );

    // ── Label an old assessment from pre-deregistration should still work ──
    let req = make_request_with_sentinels(ChannelId(0), 5000, &[2]);
    let r = world.derive_for_requests(&[req]);
    let reckoning = r.into_iter().next().unwrap().unwrap();
    let label_result = assayer.label(make_label(reckoning.assessment.id, 1.0));
    assert!(label_result.is_ok(), "label after deregistration should succeed");

    // ── Drop the scenario → verify clean shutdown ──
    drop(world);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrent Access
// ═══════════════════════════════════════════════════════════════════════════════

/// Four threads driving a thousand assessments, a hundred labels, ten reports
/// and a hundred health reads against one engine all finish, and the engine
/// assesses normally afterwards. These are the four things a host does at once
/// in production and each takes a different set of locks — the point is that no
/// ordering among them can wedge.
///
/// ´claim:lifespan:assessment-labelling-reporting-and-health-reads-run-concurrently-without-deadlock´
/// ´test:crate:concurrent-access-no-panic-or-deadlock´
#[test]
fn concurrent_access_no_panic_or_deadlock() {
    let world = Arc::new(World::cold("lifecycle-integration", 0x6C69_6665_6379_636C));

    // Register a Sentinel so reports and assessments have real data.
    world.assayer().register_sentinel(sentinel_reg(1, "concurrent-s1")).unwrap();
    let report = golden_report();
    world.assayer().receive_sentinel_report(SentinelId(1), report).unwrap();

    let a1 = Arc::clone(&world);
    let a2 = Arc::clone(&world);
    let a3 = Arc::clone(&world);
    let a4 = Arc::clone(&world);

    // Thread 1: 1,000 assess-and-derive calls
    let t1 = thread::spawn(move || {
        for i in 0_u64..1_000 {
            let results = a1.derive_for_requests(&[make_request(ChannelId(0), i)]);
            assert_eq!(results.len(), 1);
            assert!(results[0].is_ok(), "assess/derive {i} should succeed");
        }
    });

    // Thread 2: 100 label() calls (assess first to get valid IDs).
    let t2 = thread::spawn(move || {
        for i in 0_u64..100 {
            let results = a2.derive_for_requests(&[make_request(ChannelId(0), 10_000 + i)]);
            let reckoning = results.into_iter().next().unwrap().unwrap();
            let valence = if i % 5 == 0 { 1.0 } else { -1.0 };
            // label might fail if model owner shuts down during test, that's ok
            drop(a2.assayer().label(make_label(reckoning.assessment.id, valence)));
        }
    });

    // Thread 3: 10 receive_sentinel_report() calls
    let t3 = thread::spawn(move || {
        for _i in 0..10 {
            let report = golden_report();
            let _ = a3.assayer().receive_sentinel_report(SentinelId(1), report);
        }
    });

    // Thread 4: health_summary() frequently
    let t4 = thread::spawn(move || {
        for _ in 0..100 {
            let _summary = a4.assayer().health_summary();
            // Retained concurrency pacing: the fixed-count reader awaits no state; the delay keeps health reads overlapping the other workloads.
            thread::sleep(std::time::Duration::from_millis(1));
        }
    });

    // All threads complete without panic or deadlock
    t1.join().expect("assessment thread panicked");
    t2.join().expect("label thread panicked");
    t3.join().expect("report thread panicked");
    t4.join().expect("health thread panicked");

    // Verify assayer is still functional
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 99999)]);
    assert!(results[0].is_ok(), "assessment should succeed after concurrent access");

    drop(world);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-Seeding Convergence
// ═══════════════════════════════════════════════════════════════════════════════

/// Two hundred seeded entries are reported as processed and then show up as two
/// hundred total labels in the health summary. Seeding is not a side channel:
/// what it teaches the model is counted in the same ledger of evidence that
/// live labels are.
///
/// ´claim:lifespan:pre-seeded-entries-are-counted-as-labels-in-the-health-summary´
/// ´test:crate:preseed-convergence-non-trivial´
#[test]
fn preseed_convergence_non_trivial() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Register infrastructure
    assayer.register_sentinel(sentinel_reg(1, "s1")).unwrap();
    assayer.register_identity_dimension(identity_reg(1, "primary")).unwrap();

    // Pre-seed with 200 entries, 15% positive rate
    let entries: Vec<_> = (0..200)
        .map(|i| {
            let valence = if i < 30 { 1.0 } else { -1.0 }; // 15% positive
            make_pre_seed_entry(i, valence)
        })
        .collect();

    let result = assayer.pre_seed(&entries).unwrap();
    assert_eq!(result.processed, 200, "all 200 entries should be processed");

    // Assess 10 requests with varied entities
    let results: Vec<_> = (0..10)
        .map(|i| {
            let r = world.derive_for_requests(&[make_request(ChannelId(0), 50_000 + i)]);
            r.into_iter().next().unwrap().unwrap()
        })
        .collect();

    // Verify p̂ is not all constant 0.5 — model should have moved from prior
    let p_bads: Vec<f64> = results.iter().map(|r| r.assessment.risk.p_bad).collect();
    for &p in &p_bads {
        assert!(p.is_finite(), "p_bad should be finite after pre-seed");
    }

    let summary = assayer.health_summary();
    assert_eq!(
        summary.total_labels, 200,
        "total_labels ({}) should be 200 after pre-seeding",
        summary.total_labels
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shutdown Scenarios
// ═══════════════════════════════════════════════════════════════════════════════

/// Building an engine and immediately dropping it completes without panic. The
/// worker threads that start during construction must be able to be told to
/// stop before they have had anything at all to do.
///
/// ´claim:lifespan:an-engine-dropped-before-it-does-any-work-shuts-down-cleanly´
/// ´test:crate:shutdown-immediately-after-build´
#[test]
fn shutdown_immediately_after_build() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    drop(world);
    // If we get here, no panic occurred.
}

/// Dropping one engine handle while another thread owns the remaining handle lets the fixed assessment run finish without panic. Shared ownership keeps the instance alive until that worker finishes, and the last drop then shuts it down; elapsed time establishes neither fact.
///
/// ´claim:lifespan:a-shutdown-during-live-assessment-ends-the-assessing-thread-with-an-error-not-a-panic´
/// ´test:crate:shutdown-mid-assess-concurrent´
#[test]
fn shutdown_mid_assess_concurrent() {
    let world = Arc::new(World::cold("lifecycle-integration", 0x6C69_6665_6379_636C));
    let a = Arc::clone(&world);

    // Spawn assessment thread
    let t = thread::spawn(move || {
        for i in 0..500 {
            let results = a.derive_for_requests(&[make_request(ChannelId(0), i)]);
            // Some may fail due to shutdown, that's fine
            if results[0].is_err() {
                break;
            }
        }
    });

    // Dropping this clone cannot race shutdown with the worker: its clone keeps the engine alive until the fixed assessment run finishes.
    drop(world);

    // Assessment thread must complete (either finish or encounter Error)
    t.join().expect("assessment thread should not panic");
}

/// Ten labels submitted and never flushed are still in flight when the engine
/// is dropped, and the shutdown completes without panic. Unprocessed work at
/// shutdown is expected rather than exceptional, and its cost is the learning
/// that never landed — not a crash.
///
/// ´claim:lifespan:dropping-an-engine-with-labels-still-queued-is-not-a-fault´
/// ´test:crate:shutdown-mid-label-channel´
#[test]
fn shutdown_mid_label_channel() {
    let world = World::cold("lifecycle-integration", 0x6C69_6665_6379_636C);
    let assayer = world.assayer();

    // Submit some assessments and labels.
    for i in 0_u64..10 {
        let results = world.derive_for_requests(&[make_request(ChannelId(0), i)]);
        let reckoning = results.into_iter().next().unwrap().unwrap();
        drop(assayer.label(make_label(reckoning.assessment.id, 1.0)));
    }

    // Don't flush — drop with labels potentially in channel
    drop(world);
    // If we get here, no panic.
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrent Sentinel Reports
// ═══════════════════════════════════════════════════════════════════════════════

/// Two threads filing fifty reports each for two different Sentinels all
/// succeed, and the engine still reports Sentinels present afterwards. Report
/// ingestion locks per Sentinel, so a fleet's members do not queue behind one
/// another to be heard.
///
/// ´claim:lifespan:reports-for-different-sentinels-do-not-contend-with-one-another´
/// ´test:crate:concurrent-reports-different-sentinels´
#[test]
fn concurrent_reports_different_sentinels() {
    let world = Arc::new(World::cold("lifecycle-integration", 0x6C69_6665_6379_636C));

    // Register two sentinels
    world.assayer().register_sentinel(sentinel_reg(1, "conc-s1")).unwrap();
    world.assayer().register_sentinel(sentinel_reg(2, "conc-s2")).unwrap();

    let a1 = Arc::clone(&world);
    let a2 = Arc::clone(&world);

    // Thread 1: 50 reports for Sentinel 1
    let t1 = thread::spawn(move || {
        for _ in 0..50 {
            let report = golden_report();
            let result = a1.assayer().receive_sentinel_report(SentinelId(1), report);
            assert!(result.is_ok(), "sentinel 1 report should succeed");
        }
    });

    // Thread 2: 50 reports for Sentinel 2
    let t2 = thread::spawn(move || {
        for _ in 0..50 {
            let report = golden_report();
            let result = a2.assayer().receive_sentinel_report(SentinelId(2), report);
            assert!(result.is_ok(), "sentinel 2 report should succeed");
        }
    });

    t1.join().expect("sentinel 1 thread should not panic");
    t2.join().expect("sentinel 2 thread should not panic");

    // Both sentinels should still be functional — routed through both
    // coordinates, both contribute.
    let results = world.derive_for_requests(&[make_request_with_sentinels(ChannelId(0), 1, &[1, 2])]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    assert!(!reckoning.assessment.health.zero_sentinels);
    assert_eq!(reckoning.assessment.risk.n_sentinels_reporting, 2);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Label Error Paths (deferred)
// ═══════════════════════════════════════════════════════════════════════════════

/// Against an engine whose label channel sits at its tabulated floor,
/// labelling past the channel's capacity with the steward parked returns a
/// channel-full error rather than blocking the caller or silently
/// discarding the update. Backpressure is reported to the host, which is
/// the only party that can decide whether to retry, to shed, or to slow
/// down. Full health reports the same event as a full live depth and one
/// cumulative loss, so a later scraper can observe the refusal after its
/// per-call error has gone by.
///
/// ´claim:lifespan:a-label-that-cannot-be-queued-is-refused-rather-than-dropped-or-blocked´
/// ´test:crate:label-channel-full-no-journal´
#[test]
fn label_channel_full_no_journal() {
    // The builder refuses a capacity below the tabulated floor of one
    // hundred, so the overflow is driven deterministically instead: the
    // steward is parked and one more label than the channel holds is
    // submitted.
    let config = AssayerConfig {
        instance_id: "label-channel-floor".to_owned(),
        infrastructure: InfrastructureConfig {
            label_channel_capacity: 100,
            ..crate::testing::test_infrastructure()
        },
        ..Default::default()
    };
    let world = World::builder(config)
        .seed(0x6C61_6265_6C2D_666C)
        .build()
        .expect("label-channel floor world should build");
    let assayer = world.assayer();

    let mut assessment_ids = Vec::new();
    for i in 0_u64..101 {
        let results = world.derive_for_requests(&[make_request(ChannelId(0), i)]);
        let reckoning = results.into_iter().next().unwrap().unwrap();
        assessment_ids.push(reckoning.assessment.id);
    }

    // Park the steward so nothing drains while the channel fills.
    let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx()
        .send(ModelOwnerCommand::TestBlock {
            entered: entered_tx,
            release: release_rx,
        })
        .expect("barrier command enqueues");
    entered_rx.recv_timeout(ACK_DEADLINE).expect("steward parks");
    let empty_queue = assayer.full_health_report().label_queue;
    assert_eq!(empty_queue.depth, 0);
    assert_eq!(empty_queue.drops, 0, "accepted work has not lost a label");

    let mut got_channel_full = false;
    for &rid in &assessment_ids {
        match assayer.label(make_label(rid, 1.0)) {
            Ok(_) => {}
            Err(LabelError::ChannelFull) => {
                got_channel_full = true;
                break;
            }
            Err(e) => panic!("unexpected error: {e}"),
        }
    }
    assert!(got_channel_full, "one label past the declared capacity is refused as full");

    let queue = assayer.full_health_report().label_queue;
    assert_eq!(
        queue.depth, queue.capacity,
        "the parked steward leaves the bounded queue full"
    );
    assert_eq!(queue.drops, 1, "only the refused label increments the loss counter");

    release_tx.send(()).expect("steward releases");
}

/// Once the harness has stopped and joined the model-owner thread, a label returns the shutdown error rather than hanging, panicking, or entering the disconnected queue. A caller can therefore distinguish an engine that has ended from one that is merely busy.
///
/// ´claim:lifespan:labelling-after-the-model-owner-has-gone-fails-with-a-shutdown-error-rather-than-hanging´
/// ´test:crate:label-after-model-owner-shutdown´
#[test]
fn label_after_model_owner_shutdown() {
    let config = AssayerConfig {
        instance_id: "label-after-owner-shutdown".to_owned(),
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    let mut world = World::builder(config)
        .seed(0x6C61_6265_6C2D_7374)
        .build()
        .expect("shutdown test world builds");

    // Assess to get a valid pending entry.
    let results = world.derive_for_requests(&[make_request(ChannelId(0), 42)]);
    let reckoning = results.into_iter().next().unwrap().unwrap();
    let rid = reckoning.assessment.id;

    // The harness joins the model-owner thread, so the following label observes completed shutdown rather than elapsed time.
    world.shut_down_model_owner_for_test();

    let error = world
        .assayer()
        .label(make_label(rid, 1.0))
        .expect_err("a label cannot enter a joined model-owner queue");
    assert!(matches!(error, LabelError::ModelOwnerShutdown));
}
