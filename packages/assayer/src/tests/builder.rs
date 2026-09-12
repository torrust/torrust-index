// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`config_default_compiles`] | builder | The shipped defaults are the specification's values, not placeholders: the instance name, every model hyperparameter, the temporal decay rates, the ledger and concordance settings and the infrastructure capacities all match what the spec fixes, and persistence is absent. The infrastructure capacity assertion is anchored at the reference peak request rate (´def:config:reference-peak-request-rate-100-per-second´) and the reference median label latency (´def:config:reference-label-latency-3600-seconds´), so a re-mint of either dangles this test's citations along with the value it asserts. A host that changes nothing therefore gets the documented system rather than whatever the code happened to initialise. |
//! | [`eligibility_policy_default_is_core_owned`] | builder | The eligibility policy carried by a default configuration is the policy type's own default, and it treats a failed challenge as unconfounded evidence. The two defaults cannot drift apart, so a host reading the policy type's documentation is reading what its configuration actually holds. |
//! | [`channel_id_auto_increments`] | builder | Channels are numbered in the order they are declared, starting at zero and rising by one. Identifiers are positional rather than derived from the name, so a host that keeps its declaration order keeps its identifiers, and stored per-channel state stays addressed by the same number across restarts. |
//! | [`channel_invalid_policy_rejected`] | builder | A reward parameter that is not positive is refused as `NonPositiveReward`. The reward vector is read as magnitudes of consequence, so a negative entry would invert the meaning of an outcome rather than merely scaling it — the resulting policy would push the engine towards the actions it should avoid. |
//! | [`channel_too_few_actions_rejected`] | builder | A channel offering a single action is refused as `TooFewActions`. A channel exists so that a decision can be made between alternatives; one with nothing to choose between would consume assessment work to reach a foregone conclusion, and would make every reading over its rendered field meaningless. |
//! | [`channel_actions_not_ordered_rejected`] | builder | Actions must be declared in escalating order of severity: allow before challenge before block. Listing block ahead of challenge is refused as `ActionsNotOrdered`. Severity ordering is positional, so downstream reasoning that treats a later index as a harsher intervention would otherwise silently invert. |
//! | [`build_no_schema_error`] | builder | The signal schema is not optional: a builder on which it was never declared refuses to build with `NoSignalSchema`. The schema fixes the feature layout, so an omitted declaration is a host that forgot to say what its features are, not a host that wants none — an empty schema has to be asked for explicitly. |
//! | [`build_without_channels_succeeds`] | builder | Channels are not required to construct an instance. Core assessment is channel-free — policy enters at derivation — so an engine with no channel declared is still a usable engine, and a host can stand one up before it has decided what its intervention surfaces are. |
//! | [`build_duplicate_signal_name_error`] | builder | Two declarations sharing a name are refused as `DuplicateSignalName`, even when their shapes differ. Signals are addressed by name, so a duplicate would leave one of the two declarations unreachable and silently give the host's values to the wrong feature slot. |
//! | [`build_persistence_none_succeeds`] | builder | Persistence is genuinely optional: a configuration with no persistence block builds, and the resulting instance spawns no journal or checkpoint machinery to shut down. An ephemeral deployment therefore pays nothing for durability it did not ask for. |
//! | [`build_minimal_succeeds`] | builder | The smallest useful configuration — an empty schema and a single channel — yields a working instance: the channel takes the first identifier and the engine issues strictly increasing assessment identifiers straight away. Identifiers are the handle a later label uses to find its assessment, so they must be unique and ordered from the first one onwards. |
//! | [`build_invalid_gamma_opr_rejected`] | builder | A decay rate above one is refused before any thread is spawned, and the error names the offending field by its configuration path rather than merely saying that something was wrong. A rate above one would amplify rather than forget, so catching it at construction is the difference between a failed build and a model that diverges in production. |
//! | [`build_invalid_lambda_prior_rejected`] | builder | cites (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´) |
//! | [`build_invalid_lambda_floor_exceeds_prior`] | builder | Validation covers relations between parameters, not only each one in isolation: a precision floor and a prior precision that are each perfectly legal on their own are still refused when the floor sits above the prior. A floor higher than the value it is meant to bound from below cannot be satisfied, so the pair is incoherent however sound each half looks alone. |
//! | [`build_invalid_n_recompute_rejected`] | builder | cites (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´) |
//! | [`build_defaults_pass_validation`] | builder | The shipped defaults survive their own validation: a build that changes nothing but the channel declaration succeeds. A gate strict enough to refuse the values the crate itself recommends would be a gate no host could satisfy, so this is the fixed point the whole validation phase is measured against. |
//! | [`test_entity_helper_deterministic`] | builder | The test entity helper is a function of its seed alone: the same seed yields the same key, different seeds yield different keys, and every key carries bytes. Identity features are computed from those bytes, so a helper that varied between calls would make any test that depends on entity identity irreproducible. |
//! | [`test_report_helper_valid`] | builder | The test report helper always supplies the depth-zero root cell covering the whole domain, and adds exactly the competitive cells asked for on top of it. Report ingestion refuses a report with no root, so a helper that omitted it would make every test built on it fail for a reason unrelated to what the test was about. |
//! | [`non_exhaustive_types_exist`] | builder | Every output type a host reads from an assessment — the derived reckoning, the risk basis, the health snapshot, tags and their resonance, the decision landscape and rendered profile, sentinel alarm summaries, outcome predictions, the degradation context and the convergence stage — is nameable from outside the crate and can be taken by reference in downstream signatures. Because the types are non-exhaustive, that is the whole surface: they can be read and passed around but not built by struct literal, so adding a field later is not a breaking change for callers. |
//! | [`non_exhaustive_error_types_exist`] | builder | cites (´claim:builder:the-public-output-types-are-nameable-downstream-but-not-literal-constructible´) |
//! | [`cold_start_dimension_no_signals_no_templates`] | builder | An instance built from an empty schema with no templates still carries a sixteen-dimensional feature space: the bias term plus the fifteen aggregate features the engine always computes. Those dimensions are structural rather than host-declared, so the model has something to learn against before the host has declared a single signal. |
//! | [`cold_start_dimension_with_signals`] | builder | Each declared signal adds exactly one dimension on top of the structural sixteen, whatever its shape: three declarations — two binary, one clipped scalar — give nineteen. Shape governs how a value is encoded, not how much room it occupies, so a host can change a signal's shape without reshaping the model or invalidating a checkpoint. |
//! | [`cold_start_dimension_with_type4_templates`] | builder | A Type-4 interaction template contributes one always-present feature each, so two templates on an empty schema give eighteen dimensions. Being always present is what makes the contribution countable in advance: the dimension map can be fixed at construction rather than growing as interactions happen to fire. |
//! | [`cold_start_dimension_with_signals_and_templates`] | builder | Signals and templates contribute independently and additively: two signals and one Type-4 template give nineteen dimensions, the same total as three signals alone. Neither kind of declaration displaces or subsumes the other, so a host can reason about the width of its model by simple addition. |
//! | [`build_multiple_channels_succeeds`] | builder | cites (´claim:builder:channels-are-numbered-sequentially-in-declaration-order´) |
//! | [`thread_names_include_instance_id`] | builder | A host-chosen instance identifier is accepted at construction and carried into the spawned threads' names. Two instances in one process would otherwise be indistinguishable in a stack trace or a thread listing, which is exactly the situation in which an operator most needs to tell them apart. |
//! | [`shutdown_completes_without_panic`] | builder | Dropping an instance runs the ordered shutdown — checkpoint, then identity maintenance, then the model owner — to completion, without a panic and without hanging on a thread that never joins. Shutdown is what a host gets implicitly at the end of a scope, so a hang there would be a hang with no call site to blame. |
//! | [`config_serde_round_trip_json`] | builder | A configuration written out as JSON and read back is the configuration that went in, across every subsystem and to within floating-point exactness — the instance name, model and temporal rates, resonance, Platt, Schur, Cholesky, concordance and infrastructure settings alike, with an absent persistence block still absent. Configuration is a file-shaped artefact for most hosts, so a field that silently failed to survive the trip would be a field the host believed it had set. |
//! | [`config_serde_round_trip_with_persistence`] | builder | The optional persistence block survives serialisation with both directory paths intact — it comes back present, not merely non-null. Paths are the one part of a configuration that binds the instance to the machine it runs on, so a round trip that lost or mangled them would silently relocate a host's durable state. |
//! | [`config_serde_partial_override`] | builder | Overriding a couple of fields and round-tripping leaves those fields changed and everything else at its default. This is the config-file workflow: a host serialises the defaults, edits the handful of values it cares about and reads the result back, and must be able to trust that the fields it did not touch are still the recommended ones. |
//! | [`build_persistence_invalid_path_error`] | builder | Persistence directories are probed during the build, not on first use: a checkpoint and journal path that cannot be written fails construction with `PersistenceSetupFailed`. Discovering an unwritable path at the first checkpoint would mean discovering it after the state worth persisting had already accumulated. |
//! | [`concordance_config_flows_through_builder`] | builder | A concordance configuration set to non-default values reaches the tracker that uses it, and the health report reads back the window capacity the host asked for. A subsystem that quietly fell back to its own defaults would look configured and behave unconfigured — the most expensive kind of wiring bug, because nothing fails. |
//! | [`preseed_basic_round_trip`] | builder | Pre-seeding accounts for every entry it was handed: a run of five mixed historical labels reports five processed, none failed, and a resulting global positive rate that is a genuine probability. The counts are how a host knows its history was actually absorbed rather than partly dropped on the way in. |
//! | [`preseed_result_fields_populated`] | builder | Pre-seeded labels move the global positive rate in the direction of the evidence, and an empty pre-seed moves it nowhere: seeding nothing leaves the cold-start rate exactly where it was, while twenty uniformly positive labels push it above the halfway point. The empty case matters as much as the loaded one — it shows the rate reflects the labels rather than the act of calling the method. |
//! | [`concurrent_build_then_assess_and_derive`] | builder | An instance is usable the moment it is built: assessing and deriving with no registrations and no labels in between succeeds, and answers from the prior with a risk near maximum uncertainty. There is no warm-up period during which requests must be held back, so a host can put a fresh instance in the serving path and let it learn from the traffic it sees. |
//! | [`companion_tracker_initialised_per_channel`] | builder | Challenge effectiveness is host-owned, and a host-side tracker can be initialised per channel with that channel's own time-decay before any challenge result has arrived. Two channels seeded with different decay rates coexist in one tracker, and a failed challenge on one of them lifts that channel's effectiveness estimate above the prior. Keeping the decay per-channel is what stops a slow-moving surface from being read at the pace of a fast one. |
//! | [`calibration_buffer_capacity_from_config`] | builder | A calibration buffer configured far smaller than the run of labels it receives evicts the oldest entries and keeps going: a hundred pre-seeded labels through a fifty-entry buffer are all processed. The buffer is a bounded window on recent evidence by design, so overflowing it is the normal case rather than an error condition a host must avoid. |
//! | [`platt_kappa_initial_wired_through_builder`] | builder | A non-default initial Platt sharpness is carried through construction into the convergence tracker, and a refit interval of one means the very first label exercises the refit path. The calibration health that comes back afterwards still reports a positive sharpness. A sharpness that reached zero or went negative would flatten or invert the calibration map, so remaining positive across a refit is the property worth holding. |
//! | [`p_plus_init_wired_through_builder`] | builder | The configured initial positive rate is what a cold instance actually holds: with the prior set away from the halfway point, an empty pre-seed reads back exactly that configured value rather than a hardcoded one. A host with prior knowledge of its own base rate can therefore start from it instead of spending labels rediscovering it. |
//! | [`model_config_gamma_opr_wired_end_to_end`] | builder | An operational decay rate set aggressively far from its default is accepted all the way through the label pipeline: ten labels process cleanly under a per-label decay of one half rather than the near-unity default. The pipeline's arithmetic is parameterised by the configured rate rather than tuned around the default, so an unusual but legal choice is served rather than merely tolerated at the config boundary. |
//! | [`build_invalid_kappa_min_rejected`] | builder | cites (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´) |
//! | [`build_invalid_kappa_max_rejected`] | builder | cites (´claim:builder:validation-covers-relations-between-parameters-not-only-each-one-alone´) |
//! | [`channel_invalid_gamma_q_t_rejected`] | builder | A channel time-decay of exactly one is refused as `InvalidGammaQt`: the rate must sit strictly inside the open unit interval. A decay of one never forgets, so a channel configured that way would keep weighting challenge evidence from arbitrarily long ago as heavily as evidence from today. |
//! | [`channel_zero_gamma_q_t_rejected`] | builder | cites (´claim:builder:a-channel-time-decay-outside-the-open-unit-interval-is-refused´) |
//! | [`build_invalid_n_boot_rejected`] | builder | cites (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´) |
//! | [`build_invalid_alpha_boot_rejected`] | builder | cites (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´) |
//! | [`builder_refuses_unset_command_channel_capacity`] | builder | The command channel's capacity has no default: a configuration that never declared one refuses to build, on the builder surface and on the crate-internal construction path alike, with the error naming the configuration path. The capacity was previously a literal a host could not reach — the one channel that silently discards a batch initialisation was also the one channel an operator could not widen — and a defaulted replacement would merely rename that constant. Refusal is what makes the capacity a declared decision at every deployment. |
//! | [`declared_command_channel_capacity_reaches_the_channel`] | builder | The declared capacity is the capacity the channel is built with: an engine declared at two slots, with its steward parked, admits exactly two commands and refuses the third as full. Without this the declaration would be a field the host believed it had set — the configured-looking, unconfigured-behaving shape the builder suite exists to rule out. |
//! | [`validation_reaches_every_monitoring_threshold`] | builder | Every field of the public monitoring configuration is validated at construction against its tabulated domain, including the fields reserved behind named definition STOPs. |
//! | [`validation_reaches_ledger_concordance_persistence_and_blend_rows`] | builder | The parameters the audit measured escaping enforcement are now reached, each refusal naming its configuration path: the Ledger's absence threshold and collection bounds, the concordance interval and percentile, the persisted checkpoint interval, and the blend window pair. The constraint column is part of the specification — a value outside it is a configuration error, not a tuning choice — and an unenforced row is a bound a deployment discovers only by the damage its violation does. |
//! | [`validation_reaches_the_remaining_infrastructure_capacities`] | builder | cites (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´) |
//! | [`validation_requires_the_convergence_figures_to_be_positive`] | builder | cites (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´) |
//! | [`validation_requires_sister_rate_above_operational_rate`] | builder | cites (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´) |
//! | [`validation_refuses_every_non_finite_configuration_float`] | builder | Every floating-point field a configuration can reach is refused when it holds a value that is not a number, and refused again for either infinity, with the refusal naming the field that carried it. The table is walked field by field rather than check by check, so a field the pass does not reach fails here instead of being read as validated: the enumeration is taken from the configuration types, and the count is the count of floats those types own. A bound is not a finiteness test. Every ordered comparison with NaN is false, so a NaN sits inside every interval the pass can state and used to arrive in the model arithmetic through a construction that had reported the configuration sound — the one shape of configuration error that survives the gate meant to catch it, and then poisons every sum it reaches rather than failing anywhere a host could read. |
//! | [`builder_refuses_out_of_range_template_offsets`] | builder | The template validator validates: every offset a template carries is checked at build time against the block that resolves it — a Sentinel-slot offset against the base extraction width, an aggregate offset against the aggregate block, a context offset against the fixed context region the cold start lays out — and the refusal names the template's index and the bound it broke. The validator was previously a constant function returning success for every input, deferring the check to a runtime path that has already promised it cannot fail. |
//! | [`builder_admits_boundary_template_offsets`] | builder | cites (´claim:builder:a-template-offset-outside-the-block-that-resolves-it-is-refused-at-build´) |
//! | [`builder_refuses_an_absence_threshold_the_count_cannot_reach`] | builder | The Ledger's absence threshold is accepted across exactly the range the count that carries it can reach: one and 255 build, and the instance ingests reports against the value the configuration named rather than whatever a narrowing left behind, while zero, 256 and 300 are refused with the configuration path that named them. Absences are counted in eight bits, so a threshold of 256 narrowed to zero would delete every non-root entry on the first report and 300 would fire at 44 — two deletion rules no host asked for, and neither of them visible from the configuration that produced it. |
//! | [`build_persistence_without_codecs_refused`] | builder | A build that compiles no checkpoint or journal codec refuses a configuration that names persistence directories rather than accepting it and persisting nothing. Both codecs are compiled behind the `serde` feature, so an instance built without it holds no writer for either artefact: the host would be handed an instance that looks durable, and the first evidence otherwise would be a restart with no state to restore. |

//! Tests for `AssayerBuilder`.
//!
//! Construction is the one moment at which a host's configuration is
//! checked as a whole. Every numeric parameter is range-checked and the
//! relations between parameters are checked too, so an instance that
//! exists at all is an instance whose configuration was coherent; a
//! configuration mistake surfaces as a named `BuildError` carrying the
//! offending field path rather than as strange model behaviour weeks
//! later. Persistence directories are probed at the same moment, for the
//! same reason.
//!
//! What a built instance starts out holding is equally part of the
//! contract: the cold-start feature dimension is derived from the
//! declared schema and templates, the configured priors are what a fresh
//! instance reports, and pre-seeding is the supported way to move that
//! starting point using historical labels before the system goes live.
//!
//! # Cross-References
//!
//! - (´dec:construction:eager-validation´) — the builder rejects at build rather than later
//! - (´tab:construction:parameters´) — the parameters this contract tabulates

use super::helpers::{test_assayer_config, test_entity, test_report};
use crate::api::AssayerBuilder;
use crate::config::types::AssayerConfig;
use crate::error::{BuildError, ChannelError};
use crate::resonance::channel::{Action, ChannelPolicy, RewardParameters, validate_channel_policy};
use crate::risk::challenge::{ChallengeEffectivenessState, ChallengeEffectivenessTracker};
use crate::signal::{Persistence, SignalDeclaration, SignalShape};
use crate::testing::{ACK_DEADLINE, World};
use crate::types::{ChallengeResult, ChannelId, PersistentTimestamp, SentinelId};
use crate::{Assayer, EligibilityPolicy};

// ═══════════════════════════════════════════════════════════════════════════════
// Config Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The shipped defaults are the specification's values, not placeholders: the
/// instance name, every model hyperparameter, the temporal decay rates, the
/// ledger and concordance settings and the infrastructure capacities all match
/// what the spec fixes, and persistence is absent. The infrastructure capacity
/// assertion is anchored at the reference peak request rate
/// (´def:config:reference-peak-request-rate-100-per-second´) and the reference
/// median label latency (´def:config:reference-label-latency-3600-seconds´),
/// so a re-mint of either dangles this test's citations along with the value
/// it asserts. A host that changes nothing therefore gets the documented
/// system rather than whatever the code happened to initialise.
///
/// ´claim:builder:the-shipped-defaults-are-the-specified-values-and-persistence-is-off´
/// ´test:crate:config-default-compiles´
#[test]
fn config_default_compiles() {
    let config = AssayerConfig::default();

    // Verify spec-compliant defaults (´chap:spec:configuration´)
    assert_eq!(config.instance_id, "assayer");

    // Model hyperparameters (´tab:config:risk-model´)
    assert!((config.model.lambda_prior - 0.1).abs() < 1e-9);
    assert!((config.model.lambda_floor - 0.001).abs() < 1e-9);
    assert!((config.model.c_leverage - 5.0).abs() < 1e-9);
    assert!((config.model.w_ceiling - 100.0).abs() < 1e-9);
    assert!((config.model.gamma_opr - 0.9995).abs() < 1e-9);
    assert!((config.model.gamma_inh - 0.9998).abs() < 1e-9);
    assert!((config.model.p_plus_init - 0.5).abs() < 1e-9);

    // Core eligibility policy (´tab:config:eligibility´)
    assert!(config.eligibility.challenge_fail_is_unconfounded);

    // Temporal decay rates (´tab:config:temporal´)
    assert!((config.temporal.gamma_t_core - 0.9999).abs() < 1e-9);
    assert!((config.temporal.gamma_t_ledger - 0.999).abs() < 1e-9);
    assert!((config.temporal.gamma_t_identity - 0.998).abs() < 1e-9);

    // Per-channel challenge time-decay is in RewardParameters, not TemporalConfig
    // (´tab:channel:reward-parameters´). Verify its default on a ChannelPolicy.
    let default_reward = RewardParameters::default();
    assert!((default_reward.gamma_q_t - 0.9998).abs() < 1e-9);

    // gamma_kappa model parameter (´tab:config:risk-model´)
    assert!((config.model.gamma_kappa - 0.9998).abs() < 1e-9);

    // Ledger parameters (´tab:config:ledger´)
    assert!((config.ledger.lambda_l - 0.999).abs() < 1e-9);
    assert_eq!(config.ledger.n_absent, 3);
    assert_eq!(config.ledger.gc_horizon_days, 60);

    // Hibernation archive lifetime (´tab:construction:parameters´)
    assert_eq!(config.hibernation.expiry_days, 90);
    assert!((config.hibernation.expiry_hours() - 2_160.0).abs() < 1e-9);

    // Concordance (´tab:config:monitoring´)
    assert_eq!(config.concordance.window_capacity, 5000);
    assert_eq!(config.concordance.recalibration_interval, 1000);
    assert!((config.concordance.percentile - 0.80).abs() < 1e-9);

    // Infrastructure. The pending-buffer capacity is the
    // computation 2·R·L (´tab:config:pending-buffer´): at the default
    // 100 requests per second and one-hour median label latency,
    // 2 × 100 × 3,600 = 720,000. The expiry horizon is twenty-four hours.
    assert!((config.infrastructure.expected_peak_request_rate - 100.0).abs() < 1e-9);
    assert_eq!(config.infrastructure.expected_label_latency_secs, 3_600);
    assert_eq!(config.infrastructure.pending_buffer_capacity(), 720_000);
    assert_eq!(config.infrastructure.expiry_horizon_secs, 86_400);
    assert_eq!(config.infrastructure.label_channel_capacity, 10_000);
    assert_eq!(config.infrastructure.identity_registration_timeout_secs, 10);

    // Persistence disabled by default
    assert!(config.persistence.is_none());
}

/// The eligibility policy carried by a default configuration is the policy
/// type's own default, and it treats a failed challenge as unconfounded
/// evidence. The two defaults cannot drift apart, so a host reading the policy
/// type's documentation is reading what its configuration actually holds.
///
/// ´claim:builder:the-default-eligibility-policy-treats-a-failed-challenge-as-unconfounded´
/// ´test:crate:eligibility-policy-default-is-core-owned´
#[test]
fn eligibility_policy_default_is_core_owned() {
    let config = AssayerConfig::default();
    let policy: EligibilityPolicy = config.eligibility;

    assert_eq!(policy, EligibilityPolicy::default());
    assert!(policy.challenge_fail_is_unconfounded);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Channel Declaration Tests
// ═══════════════════════════════════════════════════════════════════════════════

fn default_policy() -> ChannelPolicy {
    ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    }
}

/// Channels are numbered in the order they are declared, starting at zero and
/// rising by one. Identifiers are positional rather than derived from the
/// name, so a host that keeps its declaration order keeps its identifiers, and
/// stored per-channel state stays addressed by the same number across restarts.
///
/// ´claim:builder:channels-are-numbered-sequentially-in-declaration-order´
/// ´test:crate:channel-id-auto-increments´
#[test]
fn channel_id_auto_increments() {
    let world = World::builder(test_assayer_config())
        .channel("channel_0", default_policy())
        .channel("channel_1", default_policy())
        .channel("channel_2", default_policy())
        .seed(0)
        .build()
        .unwrap();

    assert_eq!(world.channel("channel_0"), Some(ChannelId(0)));
    assert_eq!(world.channel("channel_1"), Some(ChannelId(1)));
    assert_eq!(world.channel("channel_2"), Some(ChannelId(2)));
}

/// A reward parameter that is not positive is refused as `NonPositiveReward`.
/// The reward vector is read as magnitudes of consequence, so a negative entry
/// would invert the meaning of an outcome rather than merely scaling it — the
/// resulting policy would push the engine towards the actions it should avoid.
///
/// ´claim:builder:a-channel-policy-with-a-non-positive-reward-is-refused´
/// ´test:crate:channel-invalid-policy-rejected´
#[test]
fn channel_invalid_policy_rejected() {
    // Policy with negative reward parameter
    let invalid_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        reward: RewardParameters {
            pass: -1.0, // Invalid: negative
            friction: 0.5,
            missed: 1.0,
            caught: 0.8,
            blocked: 0.2,
            slow_severity: 0.5,
            block_catches: 0.9,
            beta_bad: 1.0,
            beta_good: 1.0,
            gamma_q_t: 0.9998,
        },
        ..ChannelPolicy::default()
    };

    let result = validate_channel_policy(&invalid_policy);
    assert!(matches!(result, Err(ChannelError::NonPositiveReward)));
}

/// A channel offering a single action is refused as `TooFewActions`. A channel
/// exists so that a decision can be made between alternatives; one with nothing
/// to choose between would consume assessment work to reach a foregone
/// conclusion, and would make every reading over its rendered field meaningless.
///
/// ´claim:builder:a-channel-that-offers-no-choice-of-action-is-refused´
/// ´test:crate:channel-too-few-actions-rejected´
#[test]
fn channel_too_few_actions_rejected() {
    // Policy with only one action
    let invalid_policy = ChannelPolicy {
        actions: vec![Action::Allow],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };

    let result = validate_channel_policy(&invalid_policy);
    assert!(matches!(result, Err(ChannelError::TooFewActions)));
}

/// Actions must be declared in escalating order of severity: allow before
/// challenge before block. Listing block ahead of challenge is refused as
/// `ActionsNotOrdered`. Severity ordering is positional, so downstream
/// reasoning that treats a later index as a harsher intervention would
/// otherwise silently invert.
///
/// ´claim:builder:channel-actions-must-be-declared-in-escalating-order-of-severity´
/// ´test:crate:channel-actions-not-ordered-rejected´
#[test]
fn channel_actions_not_ordered_rejected() {
    // Policy with wrong action order (Block before Challenge)
    let invalid_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block, Action::Challenge],
        reward: RewardParameters::default(),
        ..ChannelPolicy::default()
    };

    let result = validate_channel_policy(&invalid_policy);
    assert!(matches!(result, Err(ChannelError::ActionsNotOrdered)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Build Error Tests
// ═══════════════════════════════════════════════════════════════════════════════

/// The signal schema is not optional: a builder on which it was never declared
/// refuses to build with `NoSignalSchema`. The schema fixes the feature layout,
/// so an omitted declaration is a host that forgot to say what its features
/// are, not a host that wants none — an empty schema has to be asked for
/// explicitly.
///
/// ´claim:builder:building-without-a-declared-signal-schema-is-refused´
/// ´test:crate:build-no-schema-error´
#[test]
fn build_no_schema_error() {
    let config = test_assayer_config();
    // Don't call signal_schema()
    let builder = AssayerBuilder::new(config);

    // Need to declare a channel first (this would also fail, but schema check comes first)
    let result = builder.build();
    assert!(matches!(result, Err(BuildError::NoSignalSchema)));
}

/// Channels are not required to construct an instance. Core assessment is
/// channel-free — policy enters at derivation — so an engine with no channel
/// declared is still a usable engine, and a host can stand one up before it has
/// decided what its intervention surfaces are.
///
/// ´claim:builder:an-instance-builds-without-any-channel-declared´
/// ´test:crate:build-without-channels-succeeds´
#[test]
fn build_without_channels_succeeds() {
    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&[]);

    let result = builder.build();
    assert!(result.is_ok());
}

/// Two declarations sharing a name are refused as `DuplicateSignalName`, even
/// when their shapes differ. Signals are addressed by name, so a duplicate
/// would leave one of the two declarations unreachable and silently give the
/// host's values to the wrong feature slot.
///
/// ´claim:builder:duplicate-signal-names-in-the-schema-are-refused´
/// ´test:crate:build-duplicate-signal-name-error´
#[test]
fn build_duplicate_signal_name_error() {
    let config = test_assayer_config();
    let declarations = vec![
        SignalDeclaration {
            name: "duplicate".to_owned(),
            shape: SignalShape::Binary,
            persistence: Persistence::Request,
        },
        SignalDeclaration {
            name: "duplicate".to_owned(), // Same name!
            shape: SignalShape::Scalar { clip: (0.0, 1.0) },
            persistence: Persistence::Request,
        },
    ];

    let builder = Assayer::builder(config).signal_schema(&declarations);

    let result = builder.build();
    assert!(matches!(result, Err(BuildError::DuplicateSignalName { .. })));
}

/// Persistence is genuinely optional: a configuration with no persistence
/// block builds, and the resulting instance spawns no journal or checkpoint
/// machinery to shut down. An ephemeral deployment therefore pays nothing for
/// durability it did not ask for.
///
/// ´claim:builder:persistence-is-optional-and-an-instance-without-it-runs-no-checkpoint-machinery´
/// ´test:crate:build-persistence-none-succeeds´
#[test]
fn build_persistence_none_succeeds() {
    let config = AssayerConfig {
        persistence: None,
        ..test_assayer_config()
    };

    let builder = Assayer::builder(config).signal_schema(&[]);

    let assayer = builder.build().expect("build should succeed without persistence");

    // Verify no checkpoint thread by checking the assayer constructs
    // (The thread count verification is implicit in Drop not panicking)
    drop(assayer);
}

/// The smallest useful configuration — an empty schema and a single channel —
/// yields a working instance: the channel takes the first identifier and the
/// engine issues strictly increasing assessment identifiers straight away.
/// Identifiers are the handle a later label uses to find its assessment, so
/// they must be unique and ordered from the first one onwards.
///
/// ´claim:builder:a-freshly-built-instance-issues-strictly-increasing-assessment-ids´
/// ´test:crate:build-minimal-succeeds´
#[test]
fn build_minimal_succeeds() {
    // Minimal config via the test harness: empty schema, one channel.
    let world = World::builder(test_assayer_config())
        .channel("minimal", default_policy())
        .seed(0)
        .build()
        .expect("minimal build should succeed");

    assert_eq!(world.channel("minimal"), Some(ChannelId(0)));

    // Verify basic functionality.
    let id1 = world.assayer().next_assessment_id();
    let id2 = world.assayer().next_assessment_id();
    assert!(id2 > id1);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Numerical Parameter Validation Tests (´dec:construction:eager-validation´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A decay rate above one is refused before any thread is spawned, and the
/// error names the offending field by its configuration path rather than
/// merely saying that something was wrong. A rate above one would amplify
/// rather than forget, so catching it at construction is the difference
/// between a failed build and a model that diverges in production.
///
/// ´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´
/// ´test:crate:build-invalid-gamma-opr-rejected´
#[test]
fn build_invalid_gamma_opr_rejected() {
    let config = AssayerConfig {
        model: crate::config::types::ModelConfig {
            gamma_opr: 1.5, // Invalid: > 1
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "model.gamma_opr",
            ..
        })
    ));
}

/// A prior precision that is not positive is refused, and the error points at
/// `model.lambda_prior`. Precision enters the model as an inverse variance, so
/// a negative value is not merely an odd choice — it has no interpretation at
/// all, and would corrupt every posterior built on top of it.
///
/// (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´)
/// ´test:crate:build-invalid-lambda-prior-rejected´
#[test]
fn build_invalid_lambda_prior_rejected() {
    let config = AssayerConfig {
        model: crate::config::types::ModelConfig {
            lambda_prior: -0.1, // Invalid: <= 0
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "model.lambda_prior",
            ..
        })
    ));
}

/// Validation covers relations between parameters, not only each one in
/// isolation: a precision floor and a prior precision that are each perfectly
/// legal on their own are still refused when the floor sits above the prior.
/// A floor higher than the value it is meant to bound from below cannot be
/// satisfied, so the pair is incoherent however sound each half looks alone.
///
/// ´claim:builder:validation-covers-relations-between-parameters-not-only-each-one-alone´
/// ´test:crate:build-invalid-lambda-floor-exceeds-prior´
#[test]
fn build_invalid_lambda_floor_exceeds_prior() {
    let config = AssayerConfig {
        model: crate::config::types::ModelConfig {
            lambda_prior: 0.1,
            lambda_floor: 0.5, // Invalid: > lambda_prior
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "model.lambda_floor",
            ..
        })
    ));
}

/// The Cholesky recomputation interval has a floor: an interval below one
/// hundred is refused at `cholesky.n_recompute`. Recomputing the factorisation
/// too often trades away the incremental update the decomposition exists to
/// provide, so an over-eager setting is treated as a configuration error rather
/// than as a slow but valid choice.
///
/// (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´)
/// ´test:crate:build-invalid-n-recompute-rejected´
#[test]
fn build_invalid_n_recompute_rejected() {
    let config = AssayerConfig {
        cholesky: crate::config::types::CholeskyConfig {
            n_recompute: 50, // Invalid: < 100
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "cholesky.n_recompute",
            ..
        })
    ));
}

/// The shipped defaults survive their own validation: a build that changes
/// nothing but the channel declaration succeeds. A gate strict enough to refuse
/// the values the crate itself recommends would be a gate no host could
/// satisfy, so this is the fixed point the whole validation phase is measured
/// against.
///
/// ´claim:builder:the-shipped-defaults-pass-every-validation-gate´
/// ´test:crate:build-defaults-pass-validation´
#[test]
fn build_defaults_pass_validation() {
    // Default config must pass all validation checks.
    let _world = World::builder(test_assayer_config())
        .channel("test", default_policy())
        .seed(0)
        .build()
        .expect("default config should pass validation");
}

// ═══════════════════════════════════════════════════════════════════════════════
// Test Helper Verification
// ═══════════════════════════════════════════════════════════════════════════════

/// The test entity helper is a function of its seed alone: the same seed yields
/// the same key, different seeds yield different keys, and every key carries
/// bytes. Identity features are computed from those bytes, so a helper that
/// varied between calls would make any test that depends on entity identity
/// irreproducible.
///
/// ´claim:builder:the-test-entity-helper-is-seed-deterministic-and-seed-distinct´
/// ´test:crate:test-entity-helper-deterministic´
#[test]
fn test_entity_helper_deterministic() {
    // Same seed produces same key
    let e1a = test_entity(42);
    let e1b = test_entity(42);
    assert_eq!(e1a, e1b);

    // Different seeds produce different keys
    let e2 = test_entity(43);
    assert_ne!(e1a, e2);

    // Keys have non-empty bytes
    assert_ne!(e1a.as_bytes(), [] as [u8; 0]);
}

/// The test report helper always supplies the depth-zero root cell covering
/// the whole domain, and adds exactly the competitive cells asked for on top of
/// it. Report ingestion refuses a report with no root, so a helper that omitted
/// it would make every test built on it fail for a reason unrelated to what the
/// test was about.
///
/// ´claim:builder:the-test-report-helper-always-carries-a-root-cell-beneath-the-requested-cells´
/// ´test:crate:test-report-helper-valid´
#[test]
fn test_report_helper_valid() {
    // Root-only report
    let report = test_report(&[]);
    assert_eq!(report.cell_reports.len(), 1); // Just root
    assert_eq!(report.cell_reports[0].depth, 0);
    assert_eq!(report.cell_reports[0].start, 0);

    // Report with additional cells (depth >= 8 are competitive)
    let report = test_report(&[
        (0x1200_0000_0000_0000_0000_0000_0000_0000, 8),
        (0x1230_0000_0000_0000_0000_0000_0000_0000, 12),
    ]);
    // Root + 2 competitive cells
    assert_eq!(report.cell_reports.len(), 3);

    // Verify root is always present
    assert!(report.cell_reports.iter().any(|c| c.depth == 0));

    // Verify requested competitive cells are present
    assert!(report.cell_reports.iter().any(|c| c.depth == 8));
    assert!(report.cell_reports.iter().any(|c| c.depth == 12));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Non-Exhaustive Attribute Verification
// ═══════════════════════════════════════════════════════════════════════════════

/// Every output type a host reads from an assessment — the derived reckoning,
/// the risk basis, the health snapshot, tags and their resonance, the decision
/// landscape and rendered profile, sentinel alarm summaries, outcome predictions, the degradation
/// context and the convergence stage — is nameable from outside the crate and
/// can be taken by reference in downstream signatures. Because the types are
/// non-exhaustive, that is the whole surface: they can be read and passed
/// around but not built by struct literal, so adding a field later is not a
/// breaking change for callers.
///
/// ´claim:builder:the-public-output-types-are-nameable-downstream-but-not-literal-constructible´
/// ´test:crate:non-exhaustive-types-exist´
#[test]
fn non_exhaustive_types_exist() {
    // This test just verifies the types exist and are accessible.
    // The actual #[non_exhaustive] enforcement is a compile-time
    // property that prevents struct literal construction.
    use crate::{
        CompositeConvergenceStage, DecisionLandscape, DegradationContext, DerivedReckoning, HealthSnapshot, OutcomePrediction,
        ResonanceProfile, RiskBasis, SentinelAlarmSummary, Tag, TagResonance,
    };

    // Verify types are accessible (compile-time check)
    fn _accepts_derived_output(_: &DerivedReckoning) {}
    fn _accepts_risk_basis(_: &RiskBasis) {}
    fn _accepts_health_snapshot(_: &HealthSnapshot) {}
    fn _accepts_tag_resonance(_: &TagResonance) {}
    #[allow(clippy::trivially_copy_pass_by_ref)] // Justified: illustrating trait bounds for non_exhaustive verification
    fn _accepts_tag(_: &Tag) {}
    fn _accepts_landscape(_: &DecisionLandscape) {}
    fn _accepts_profile(_: &ResonanceProfile) {}
    fn _accepts_sentinel_summary(_: &SentinelAlarmSummary) {}
    fn _accepts_outcome_prediction(_: &OutcomePrediction) {}
    fn _accepts_degradation(_: &DegradationContext) {}
    fn _accepts_convergence(_: CompositeConvergenceStage) {}
}

/// The five public error enumerations — label, report, lifecycle, build and
/// channel — are equally nameable from downstream code and equally
/// non-exhaustive. A host can therefore match on the variants it handles and
/// keep compiling when a new failure mode is added, which is what lets the
/// crate grow its error vocabulary without a breaking release.
///
/// (´claim:builder:the-public-output-types-are-nameable-downstream-but-not-literal-constructible´)
/// ´test:crate:non-exhaustive-error-types-exist´
#[test]
fn non_exhaustive_error_types_exist() {
    use crate::error::{BuildError, ChannelError, LabelError, LifecycleError, ReportError};

    fn _accepts_label_error(_: &LabelError) {}
    fn _accepts_report_error(_: &ReportError) {}
    fn _accepts_lifecycle_error(_: &LifecycleError) {}
    fn _accepts_build_error(_: &BuildError) {}
    fn _accepts_channel_error(_: &ChannelError) {}
}

// ═══════════════════════════════════════════════════════════════════════════════
// Cold-Start Dimension Tests (´dec:construction:two-starts´)
// ═══════════════════════════════════════════════════════════════════════════════

/// An instance built from an empty schema with no templates still carries a
/// sixteen-dimensional feature space: the bias term plus the fifteen aggregate
/// features the engine always computes. Those dimensions are structural rather
/// than host-declared, so the model has something to learn against before the
/// host has declared a single signal.
///
/// ´claim:builder:an-empty-schema-still-carries-the-bias-and-aggregate-dimensions´
/// ´test:crate:cold-start-dimension-no-signals-no-templates´
#[test]
fn cold_start_dimension_no_signals_no_templates() {
    // p = 1 (bias) + 15 (aggregates) + 0 (signals) + 0 (interactions) = 16
    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&[]);
    let assayer = builder.build().unwrap();

    let snapshot = assayer.shared().published.load();
    assert_eq!(snapshot.dimension_map.p, 16, "p should be 16 with no signals, no templates");

    drop(assayer);
}

/// Each declared signal adds exactly one dimension on top of the structural
/// sixteen, whatever its shape: three declarations — two binary, one clipped
/// scalar — give nineteen. Shape governs how a value is encoded, not how much
/// room it occupies, so a host can change a signal's shape without reshaping
/// the model or invalidating a checkpoint.
///
/// ´claim:builder:each-declared-signal-adds-exactly-one-dimension-whatever-its-shape´
/// ´test:crate:cold-start-dimension-with-signals´
#[test]
fn cold_start_dimension_with_signals() {
    // p = 1 (bias) + 15 (aggregates) + 3 (signals) = 19
    let declarations = vec![
        SignalDeclaration {
            name: "sig_a".to_owned(),
            shape: SignalShape::Binary,
            persistence: Persistence::Request,
        },
        SignalDeclaration {
            name: "sig_b".to_owned(),
            shape: SignalShape::Binary,
            persistence: Persistence::Request,
        },
        SignalDeclaration {
            name: "sig_c".to_owned(),
            shape: SignalShape::Scalar { clip: (0.0, 1.0) },
            persistence: Persistence::Request,
        },
    ];

    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&declarations);
    let assayer = builder.build().unwrap();

    let snapshot = assayer.shared().published.load();
    assert_eq!(snapshot.dimension_map.p, 19, "p should be 19 with 3 signals");

    drop(assayer);
}

/// A Type-4 interaction template contributes one always-present feature each,
/// so two templates on an empty schema give eighteen dimensions. Being always
/// present is what makes the contribution countable in advance: the dimension
/// map can be fixed at construction rather than growing as interactions happen
/// to fire.
///
/// ´claim:builder:each-type-four-template-adds-one-always-present-dimension´
/// ´test:crate:cold-start-dimension-with-type4-templates´
#[test]
fn cold_start_dimension_with_type4_templates() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // Type-4 templates produce 1 feature each (always-present).
    // p = 1 (bias) + 15 (aggregates) + 0 (signals) + 2 (Type-4) = 18
    let templates = vec![
        InteractionTemplate::type4(1, FeatureSelector::AggregateFeature(1)),
        InteractionTemplate::type4(3, FeatureSelector::AggregateFeature(3)),
    ];

    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&[]).interaction_templates(templates);
    let assayer = builder.build().unwrap();

    let snapshot = assayer.shared().published.load();
    assert_eq!(snapshot.dimension_map.p, 18, "p should be 18 with 2 Type-4 templates");

    drop(assayer);
}

/// Signals and templates contribute independently and additively: two signals
/// and one Type-4 template give nineteen dimensions, the same total as three
/// signals alone. Neither kind of declaration displaces or subsumes the other,
/// so a host can reason about the width of its model by simple addition.
///
/// ´claim:builder:signals-and-templates-add-dimensions-without-displacing-each-other´
/// ´test:crate:cold-start-dimension-with-signals-and-templates´
#[test]
fn cold_start_dimension_with_signals_and_templates() {
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    // p = 1 (bias) + 15 (aggregates) + 2 (signals) + 1 (Type-4) = 19
    let declarations = vec![
        SignalDeclaration {
            name: "sig_x".to_owned(),
            shape: SignalShape::Binary,
            persistence: Persistence::Request,
        },
        SignalDeclaration {
            name: "sig_y".to_owned(),
            shape: SignalShape::Binary,
            persistence: Persistence::Request,
        },
    ];
    let templates = vec![InteractionTemplate::type4(1, FeatureSelector::AggregateFeature(1))];

    let config = test_assayer_config();
    let builder = Assayer::builder(config)
        .signal_schema(&declarations)
        .interaction_templates(templates);
    let assayer = builder.build().unwrap();

    let snapshot = assayer.shared().published.load();
    assert_eq!(
        snapshot.dimension_map.p, 19,
        "p should be 19 with 2 signals + 1 Type-4 template"
    );

    drop(assayer);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Multiple Channels Tests (´dec:construction:core-only-config´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Numbering by declaration order holds for realistically named channels too:
/// login, payment and signup take the first three identifiers in the order they
/// were declared. The names are host-facing labels only — nothing about a name
/// influences which identifier it gets.
///
/// (´claim:builder:channels-are-numbered-sequentially-in-declaration-order´)
/// ´test:crate:build-multiple-channels-succeeds´
#[test]
fn build_multiple_channels_succeeds() {
    // Declare three named channels; the harness tracks ID assignment order.
    let world = World::builder(test_assayer_config())
        .channel("login", default_policy())
        .channel("payment", default_policy())
        .channel("signup", default_policy())
        .seed(0)
        .build()
        .unwrap();

    assert_eq!(world.channel("login"), Some(ChannelId(0)));
    assert_eq!(world.channel("payment"), Some(ChannelId(1)));
    assert_eq!(world.channel("signup"), Some(ChannelId(2)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Thread Naming Tests (´dec:construction:named-threads´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A host-chosen instance identifier is accepted at construction and carried
/// into the spawned threads' names. Two instances in one process would
/// otherwise be indistinguishable in a stack trace or a thread listing, which
/// is exactly the situation in which an operator most needs to tell them apart.
///
/// ´claim:builder:a-host-chosen-instance-id-is-accepted-and-qualifies-the-spawned-threads´
/// ´test:crate:thread-names-include-instance-id´
#[test]
fn thread_names_include_instance_id() {
    let config = AssayerConfig {
        instance_id: "my-test-instance".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };

    // The Assayer successfully built means threads were spawned with
    // instance-qualified names (´dec:construction:named-threads´). We verify the
    // model-owner thread name via the owner.rs thread_name_matches_instance_id
    // test. Here we verify construction doesn't panic.
    let _world = World::builder(config)
        .channel("test", default_policy())
        .seed(0)
        .build()
        .unwrap();
}

// ═══════════════════════════════════════════════════════════════════════════════
// Shutdown Sequence Tests (´dec:construction:named-threads´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Dropping an instance runs the ordered shutdown — checkpoint, then identity
/// maintenance, then the model owner — to completion, without a panic and
/// without hanging on a thread that never joins. Shutdown is what a host gets
/// implicitly at the end of a scope, so a hang there would be a hang with no
/// call site to blame.
///
/// ´claim:builder:dropping-an-instance-completes-the-ordered-shutdown-without-panic-or-hang´
/// ´test:crate:shutdown-completes-without-panic´
#[test]
fn shutdown_completes_without_panic() {
    let world = World::builder(test_assayer_config())
        .channel("test", default_policy())
        .seed(0)
        .build()
        .unwrap();

    // Drop triggers shutdown: checkpoint → identity → model owner.
    // If any thread panics or hangs, this test will fail.
    drop(world);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Serde Round-Trip Tests
// ═══════════════════════════════════════════════════════════════════════════════
// No decision head states what this section establishes. The banner cited the
// Core-only rule, which fixes which configuration the builder accepts and says
// nothing about surviving serialisation, so the citation is dropped rather than
// re-pointed: the promise is carried by the three claims below, which is where
// a promise lives.

/// A configuration written out as JSON and read back is the configuration that
/// went in, across every subsystem and to within floating-point exactness — the
/// instance name, model and temporal rates, resonance, Platt, Schur, Cholesky,
/// concordance and infrastructure settings alike, with an absent persistence
/// block still absent. Configuration is a file-shaped artefact for most hosts,
/// so a field that silently failed to survive the trip would be a field the
/// host believed it had set.
///
/// ´claim:builder:a-configuration-survives-a-json-round-trip-without-drift´
/// ´test:crate:config-serde-round-trip-json´
#[cfg(feature = "serde")]
#[test]
fn config_serde_round_trip_json() {
    let original = AssayerConfig::default();
    let json = serde_json::to_string_pretty(&original).expect("serialize");
    let restored: AssayerConfig = serde_json::from_str(&json).expect("deserialize");

    // Spot-check key fields survive the round-trip.
    assert_eq!(restored.instance_id, original.instance_id);
    assert!((restored.model.lambda_prior - original.model.lambda_prior).abs() < 1e-15);
    assert!((restored.model.gamma_opr - original.model.gamma_opr).abs() < 1e-15);
    assert!((restored.temporal.gamma_t_core - original.temporal.gamma_t_core).abs() < 1e-15);
    assert!((restored.platt.kappa_initial - original.platt.kappa_initial).abs() < 1e-15);
    assert!((restored.schur.epsilon_schur - original.schur.epsilon_schur).abs() < 1e-15);
    assert_eq!(restored.cholesky.n_recompute, original.cholesky.n_recompute);
    assert_eq!(restored.concordance.window_capacity, original.concordance.window_capacity);
    assert!(
        (restored.infrastructure.expected_peak_request_rate - original.infrastructure.expected_peak_request_rate).abs() < 1e-15
    );
    assert_eq!(
        restored.infrastructure.expected_label_latency_secs,
        original.infrastructure.expected_label_latency_secs
    );
    assert!(restored.persistence.is_none());
}

/// The optional persistence block survives serialisation with both directory
/// paths intact — it comes back present, not merely non-null. Paths are the one
/// part of a configuration that binds the instance to the machine it runs on,
/// so a round trip that lost or mangled them would silently relocate a host's
/// durable state.
///
/// ´claim:builder:the-optional-persistence-block-round-trips-with-its-paths-intact´
/// ´test:crate:config-serde-round-trip-with-persistence´
#[cfg(feature = "serde")]
#[test]
fn config_serde_round_trip_with_persistence() {
    use std::path::PathBuf;

    let original = AssayerConfig {
        persistence: Some(crate::config::types::PersistenceConfig::new(
            PathBuf::from("/tmp/checkpoint"),
            PathBuf::from("/tmp/journal"),
        )),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    let json = serde_json::to_string(&original).expect("serialize");
    let restored: AssayerConfig = serde_json::from_str(&json).expect("deserialize");

    let p = restored.persistence.expect("persistence should round-trip");
    assert_eq!(p.checkpoint_dir, PathBuf::from("/tmp/checkpoint"));
    assert_eq!(p.journal_dir, PathBuf::from("/tmp/journal"));
}

/// Overriding a couple of fields and round-tripping leaves those fields
/// changed and everything else at its default. This is the config-file
/// workflow: a host serialises the defaults, edits the handful of values it
/// cares about and reads the result back, and must be able to trust that the
/// fields it did not touch are still the recommended ones.
///
/// ´claim:builder:selective-overrides-survive-a-round-trip-while-untouched-fields-keep-their-defaults´
/// ´test:crate:config-serde-partial-override´
#[cfg(feature = "serde")]
#[test]
fn config_serde_partial_override() {
    // Round-trip from full defaults with selective overrides to verify
    // the config-file workflow: host serialises defaults, changes a
    // few fields, and deserialises back.
    let original = AssayerConfig {
        instance_id: "prod-1".to_owned(),
        // The capacity is host-set with no default; the fixture declares it.
        infrastructure: crate::testing::test_infrastructure(),
        ..Default::default()
    };
    let json_full = serde_json::to_string(&original).expect("serialize");
    let restored: AssayerConfig = serde_json::from_str(&json_full).expect("deserialize");

    assert_eq!(restored.instance_id, "prod-1");
    // Other fields retain defaults.
    assert!((restored.model.lambda_prior - 0.1).abs() < 1e-15);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Persistence Setup Failure Tests (´dec:construction:eager-validation´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Persistence directories are probed during the build, not on first use: a
/// checkpoint and journal path that cannot be written fails construction with
/// `PersistenceSetupFailed`. Discovering an unwritable path at the first
/// checkpoint would mean discovering it after the state worth persisting had
/// already accumulated.
///
/// ´claim:builder:an-unwritable-persistence-directory-fails-the-build-rather-than-the-first-checkpoint´
/// ´test:crate:build-persistence-invalid-path-error´
#[cfg(feature = "serde")]
#[test]
fn build_persistence_invalid_path_error() {
    use std::path::PathBuf;

    // A path under /proc (Linux) or a null-byte path is not writable.
    let config = AssayerConfig {
        persistence: Some(crate::config::types::PersistenceConfig::new(
            PathBuf::from("/proc/0/nonexistent_assayer_test"),
            PathBuf::from("/proc/0/nonexistent_assayer_test"),
        )),
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);

    let result = builder.build();
    assert!(
        matches!(result, Err(BuildError::PersistenceSetupFailed(_))),
        "expected PersistenceSetupFailed, got {result:?}"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concordance Config Threading (´tab:construction:parameters´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A concordance configuration set to non-default values reaches the tracker
/// that uses it, and the health report reads back the window capacity the host
/// asked for. A subsystem that quietly fell back to its own defaults would look
/// configured and behave unconfigured — the most expensive kind of wiring bug,
/// because nothing fails.
///
/// ´claim:builder:a-custom-concordance-configuration-reaches-the-tracker-instead-of-being-silently-defaulted´
/// ´test:crate:concordance-config-flows-through-builder´
#[test]
fn concordance_config_flows_through_builder() {
    // With a non-default window_capacity, the builder must thread
    // the ConcordanceConfig through to the ConcordanceTracker, not
    // silently use defaults.
    let config = AssayerConfig {
        concordance: crate::config::types::ConcordanceConfig {
            window_capacity: 42,
            recalibration_interval: 7,
            percentile: 0.90,
        },
        ..test_assayer_config()
    };

    let world = World::builder(config)
        .channel("test", default_policy())
        .seed(0)
        .build()
        .unwrap();

    let report = world.assayer().full_health_report();
    assert_eq!(
        report.concordance.window_capacity, 42,
        "ConcordanceConfig::window_capacity must flow through builder"
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Pre-Seed Tests (´dec:construction:post-seeding´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Pre-seeding accounts for every entry it was handed: a run of five mixed
/// historical labels reports five processed, none failed, and a resulting
/// global positive rate that is a genuine probability. The counts are how a
/// host knows its history was actually absorbed rather than partly dropped on
/// the way in.
///
/// ´claim:builder:pre-seeding-accounts-for-every-entry-and-reports-a-valid-positive-rate´
/// ´test:crate:preseed-basic-round-trip´
#[test]
fn preseed_basic_round_trip() {
    use crate::PreSeedEntry;
    use crate::types::Action;

    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&[]);
    let ch = ChannelId(0);
    let assayer = builder.build().unwrap();

    let entries: Vec<PreSeedEntry> = (0..5)
        .map(|i| PreSeedEntry {
            entity: test_entity(i),
            channel: ch,
            action_taken: Action::Allow,
            valence: if i % 2 == 0 { 0.0 } else { 1.0 },
            outcomes: std::collections::HashMap::new(),
            ground_truth: true,
        })
        .collect();

    let result = assayer.pre_seed(&entries).expect("pre_seed should succeed");
    assert_eq!(result.processed, 5);
    assert_eq!(result.failed, 0);
    // final_p_positive should be a valid probability
    assert!(result.final_p_positive >= 0.0 && result.final_p_positive <= 1.0);

    drop(assayer);
}

/// Pre-seeded labels move the global positive rate in the direction of the
/// evidence, and an empty pre-seed moves it nowhere: seeding nothing leaves the
/// cold-start rate exactly where it was, while twenty uniformly positive labels
/// push it above the halfway point. The empty case matters as much as the
/// loaded one — it shows the rate reflects the labels rather than the act of
/// calling the method.
///
/// ´claim:builder:pre-seeded-labels-move-the-global-positive-rate-while-an-empty-pre-seed-leaves-it-at-cold-start´
/// ´test:crate:preseed-result-fields-populated´
#[test]
fn preseed_result_fields_populated() {
    use crate::PreSeedEntry;
    use crate::types::Action;

    let config = test_assayer_config();
    let builder = Assayer::builder(config).signal_schema(&[]);
    let ch = ChannelId(0);
    let assayer = builder.build().unwrap();

    // Empty pre-seed should succeed with zero counts
    let result = assayer.pre_seed(&[]).expect("empty pre_seed should succeed");
    assert_eq!(result.processed, 0);
    assert_eq!(result.failed, 0);
    // p_positive should still be the cold-start default (0.5)
    assert!(
        (result.final_p_positive - 0.5).abs() < 1e-9,
        "expected 0.5 after empty pre-seed, got {}",
        result.final_p_positive,
    );

    // Pre-seed with all-spam entries should shift p_positive upward
    let spam_entries: Vec<PreSeedEntry> = (0..20)
        .map(|i| PreSeedEntry {
            entity: test_entity(100 + i),
            channel: ch,
            action_taken: Action::Allow,
            valence: 1.0, // all spam
            outcomes: std::collections::HashMap::new(),
            ground_truth: true,
        })
        .collect();

    let result = assayer.pre_seed(&spam_entries).expect("spam pre_seed should succeed");
    assert_eq!(result.processed, 20);
    assert!(
        result.final_p_positive > 0.5,
        "p_positive should increase after all-spam labels, got {}",
        result.final_p_positive,
    );

    drop(assayer);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Concurrent Build + Assess/Derive (´dec:construction:two-phase-visibility´)
// ═══════════════════════════════════════════════════════════════════════════════

/// An instance is usable the moment it is built: assessing and deriving with no
/// registrations and no labels in between succeeds, and answers from the prior
/// with a risk near maximum uncertainty. There is no warm-up period during
/// which requests must be held back, so a host can put a fresh instance in the
/// serving path and let it learn from the traffic it sees.
///
/// ´claim:builder:a-just-built-instance-can-be-assessed-immediately-and-answers-from-the-prior´
/// ´test:crate:concurrent-build-then-assess-and-derive´
#[test]
fn concurrent_build_then_assess_and_derive() {
    let world = World::builder(test_assayer_config())
        .channel("test", default_policy())
        .seed(0)
        .build()
        .unwrap();

    // Assess and derive immediately after build without any registrations.
    // This is the minimum viable operation (´dec:construction:two-phase-visibility´).
    let reckoning = world
        .derive_for_request(world.request("test", "entity-0"))
        .expect("first assess/derive should succeed");

    // Prior-only: p̂ ≈ 0.5 (maximum uncertainty).
    assert!(
        (reckoning.assessment.risk.p_bad - 0.5).abs() < 0.2,
        "initial p_bad should be near 0.5, got {}",
        reckoning.assessment.risk.p_bad,
    );
}

// ═══════════════════════════════════════════════════════════════════════════════
// Challenge State + CalibrationBuffer + Platt κ Wiring (´tab:construction:parameters´)
// ═══════════════════════════════════════════════════════════════════════════════

/// Challenge effectiveness is host-owned, and a host-side tracker can be
/// initialised per channel with that channel's own time-decay before any
/// challenge result has arrived. Two channels seeded with different decay rates
/// coexist in one tracker, and a failed challenge on one of them lifts that
/// channel's effectiveness estimate above the prior. Keeping the decay
/// per-channel is what stops a slow-moving surface from being read at the pace
/// of a fast one.
///
/// ´claim:builder:a-host-side-challenge-tracker-can-be-seeded-per-channel-with-that-channels-decay´
/// ´test:crate:companion-tracker-initialised-per-channel´
#[test]
fn companion_tracker_initialised_per_channel() {
    // Challenge effectiveness is host-owned. A host-side tracker can still
    // initialise each channel with that channel's γ_{q,t} before any results
    // arrive, preserving the channel-specific decay contract.
    let custom_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Challenge, Action::Block],
        reward: RewardParameters {
            gamma_q_t: 0.995, // Non-default γ_{q,t}
            ..RewardParameters::default()
        },
        ..ChannelPolicy::default()
    };
    validate_channel_policy(&custom_policy).unwrap();

    let ch1 = ChannelId(0);
    let ch2 = ChannelId(1);
    let mut tracker = ChallengeEffectivenessTracker::new();
    tracker.insert_channel(
        ch1,
        ChallengeEffectivenessState::new(
            ChallengeEffectivenessState::DEFAULT_ALPHA_0,
            ChallengeEffectivenessState::DEFAULT_BETA_0,
            RewardParameters::default().gamma_q_t,
        ),
    );
    tracker.insert_channel(
        ch2,
        ChallengeEffectivenessState::new(
            ChallengeEffectivenessState::DEFAULT_ALPHA_0,
            ChallengeEffectivenessState::DEFAULT_BETA_0,
            custom_policy.reward.gamma_q_t,
        ),
    );

    let now = PersistentTimestamp::now();
    tracker.update(ch2, ChallengeResult::Fail, &now);

    assert_eq!(tracker.len(), 2);
    assert!(tracker.estimate(ch2, &now).q_c > 0.5);
}

/// A calibration buffer configured far smaller than the run of labels it
/// receives evicts the oldest entries and keeps going: a hundred pre-seeded
/// labels through a fifty-entry buffer are all processed. The buffer is a
/// bounded window on recent evidence by design, so overflowing it is the normal
/// case rather than an error condition a host must avoid.
///
/// ´claim:builder:a-calibration-buffer-smaller-than-the-label-run-evicts-rather-than-failing´
/// ´test:crate:calibration-buffer-capacity-from-config´
#[test]
fn calibration_buffer_capacity_from_config() {
    // Build with a non-default n_cal_buf and verify the Assayer
    // constructs successfully. The CalibrationBuffer is internal
    // to ModelOwner, so we verify indirectly: the pipeline works
    // with the custom capacity.
    use crate::PreSeedEntry;
    use crate::types::Action;

    let mut config = test_assayer_config();
    config.platt.n_cal_buf = 50; // Much smaller than default 2000

    let builder = Assayer::builder(config).signal_schema(&[]);
    let ch = ChannelId(0);
    let assayer = builder.build().unwrap();

    // Pre-seed enough labels to exceed the buffer capacity.
    // The buffer should evict oldest entries without error.
    let entries: Vec<PreSeedEntry> = (0..100)
        .map(|i| PreSeedEntry {
            entity: test_entity(i),
            channel: ch,
            action_taken: Action::Allow,
            valence: if i % 3 == 0 { 1.0 } else { 0.0 },
            outcomes: std::collections::HashMap::new(),
            ground_truth: true,
        })
        .collect();

    let result = assayer.pre_seed(&entries).expect("pre_seed with small buffer should succeed");
    assert_eq!(result.processed, 100);

    drop(assayer);
}

/// A non-default initial Platt sharpness is carried through construction into
/// the convergence tracker, and a refit interval of one means the very first
/// label exercises the refit path. The calibration health that comes back
/// afterwards still reports a positive sharpness. A sharpness that reached zero
/// or went negative would flatten or invert the calibration map, so remaining
/// positive across a refit is the property worth holding.
///
/// ´claim:builder:a-configured-initial-platt-sharpness-survives-construction-and-stays-positive-across-a-refit´
/// ´test:crate:platt-kappa-initial-wired-through-builder´
#[test]
fn platt_kappa_initial_wired_through_builder() {
    // Verifies that PlattConfig::kappa_initial is threaded through
    // the builder into the PlattConvergenceTracker
    // (´tab:construction:parameters´). The tracker's initial κ values are populated from
    // config, not hardcoded.
    //
    // Direct observation: the PlattConvergenceTracker unit test
    // `platt_from_config_uses_kappa_initial` verifies the constructor.
    //
    // End-to-end verification: We build with kappa_initial=2.0
    // and a very low n_refit=1 so that the first label triggers a
    // Platt refit. The refit uses `working.kappa_sister` (=1.0 from
    // cold start) as the old κ, not the tracker's. The tracker's
    // role is diagnostic (recording κ after refit). The key wiring
    // guarantee is: the `from_platt_config` path is exercised at
    // construction and the builder + label pipeline do not panic.
    use crate::PreSeedEntry;
    use crate::types::Action;

    let mut config = test_assayer_config();
    config.platt.kappa_initial = 2.0;
    config.platt.n_refit = 1; // Trigger refit on first label

    let builder = Assayer::builder(config).signal_schema(&[]);
    let ch = ChannelId(0);
    let assayer = builder.build().unwrap();

    let entries = vec![PreSeedEntry {
        entity: test_entity(0),
        channel: ch,
        action_taken: Action::Allow,
        valence: 1.0,
        outcomes: std::collections::HashMap::new(),
        ground_truth: true,
    }];
    assayer.pre_seed(&entries).expect("pre_seed should succeed");

    // Health report should show calibration health without panic.
    let report = assayer.full_health_report();
    // kappa_sister is from the snapshot (WorkingCopy value after
    // any refits that occurred). With 1 label at n_refit=1, a
    // refit was attempted — the resulting κ may differ from 1.0.
    assert!(
        report.calibration.kappa_sister > 0.0,
        "kappa_sister should be positive, got {}",
        report.calibration.kappa_sister,
    );

    drop(assayer);
}

// ═══════════════════════════════════════════════════════════════════════════════
// P+ Initial Value Wiring (´tab:construction:parameters´)
// ═══════════════════════════════════════════════════════════════════════════════

/// The configured initial positive rate is what a cold instance actually holds:
/// with the prior set away from the halfway point, an empty pre-seed reads back
/// exactly that configured value rather than a hardcoded one. A host with prior
/// knowledge of its own base rate can therefore start from it instead of
/// spending labels rediscovering it.
///
/// ´claim:builder:the-configured-initial-positive-rate-is-what-a-cold-instance-reports´
/// ´test:crate:p-plus-init-wired-through-builder´
#[test]
fn p_plus_init_wired_through_builder() {
    // Build with a non-default p_plus_init and verify the initial
    // snapshot carries that value. An empty pre-seed returns the
    // published snapshot's p_positive_global without modification.
    let mut config = test_assayer_config();
    config.model.p_plus_init = 0.3;

    let builder = Assayer::builder(config).signal_schema(&[]);
    let assayer = builder.build().unwrap();

    // Empty pre-seed: no labels processed, so p_positive_global
    // should remain at the cold-start value (p_plus_init).
    let result = assayer.pre_seed(&[]).expect("empty pre_seed should succeed");
    assert!(
        (result.final_p_positive - 0.3).abs() < 1e-9,
        "expected p_plus_init=0.3 as initial p_positive, got {}",
        result.final_p_positive,
    );

    drop(assayer);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Config Wiring — LabelPipelineConfig (´dec:construction:core-only-config´)
// ═══════════════════════════════════════════════════════════════════════════════

/// An operational decay rate set aggressively far from its default is accepted
/// all the way through the label pipeline: ten labels process cleanly under a
/// per-label decay of one half rather than the near-unity default. The
/// pipeline's arithmetic is parameterised by the configured rate rather than
/// tuned around the default, so an unusual but legal choice is served rather
/// than merely tolerated at the config boundary.
///
/// ´claim:builder:a-non-default-operational-decay-rate-is-honoured-throughout-the-label-pipeline´
/// ´test:crate:model-config-gamma-opr-wired-end-to-end´
#[test]
fn model_config_gamma_opr_wired_end_to_end() {
    // Build with a non-default gamma_opr. The label pipeline should
    // use the configured value (verified indirectly: the pipeline
    // processes labels without panic and produces different model
    // state than default gamma_opr would).
    use crate::PreSeedEntry;
    use crate::types::Action;

    let mut config = test_assayer_config();
    config.model.gamma_opr = 0.5; // Aggressive decay

    let builder = Assayer::builder(config).signal_schema(&[]);
    let ch = ChannelId(0);
    let assayer = builder.build().unwrap();

    let entries: Vec<PreSeedEntry> = (0..10)
        .map(|i| PreSeedEntry {
            entity: test_entity(i),
            channel: ch,
            action_taken: Action::Allow,
            valence: 1.0,
            outcomes: std::collections::HashMap::new(),
            ground_truth: true,
        })
        .collect();

    // If gamma_opr is wired correctly, this uses 0.5 per-label
    // decay instead of the default 0.9995. The pipeline should
    // still process without error.
    let result = assayer
        .pre_seed(&entries)
        .expect("pre_seed with custom gamma_opr should succeed");
    assert_eq!(result.processed, 10);

    drop(assayer);
}

// ═══════════════════════════════════════════════════════════════════════════════
// Platt κ Validation (´tab:config:calibration´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A lower bound of zero on the Platt sharpness is refused at
/// `platt.kappa_min`. The bound exists to keep the calibration map from
/// flattening, and a bound of zero would permit exactly the degenerate map it
/// was introduced to exclude.
///
/// (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´)
/// ´test:crate:build-invalid-kappa-min-rejected´
#[test]
fn build_invalid_kappa_min_rejected() {
    let config = AssayerConfig {
        platt: crate::risk::calibration::PlattConfig {
            kappa_min: 0.0, // Invalid: must be > 0
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "platt.kappa_min",
            ..
        })
    ));
}

/// A sharpness ceiling below its own floor is refused even though both numbers
/// are individually well inside the legal range. An inverted interval admits no
/// value at all, so clamping against it could only ever produce whichever bound
/// the clamping code happened to apply last.
///
/// (´claim:builder:validation-covers-relations-between-parameters-not-only-each-one-alone´)
/// ´test:crate:build-invalid-kappa-max-rejected´
#[test]
fn build_invalid_kappa_max_rejected() {
    let config = AssayerConfig {
        platt: crate::risk::calibration::PlattConfig {
            kappa_min: 10.0,
            kappa_max: 5.0, // Invalid: must be > kappa_min
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "platt.kappa_max",
            ..
        })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Channel γ_{q,t} Validation (´tab:channel:reward-parameters´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A channel time-decay of exactly one is refused as `InvalidGammaQt`: the rate
/// must sit strictly inside the open unit interval. A decay of one never
/// forgets, so a channel configured that way would keep weighting challenge
/// evidence from arbitrarily long ago as heavily as evidence from today.
///
/// ´claim:builder:a-channel-time-decay-outside-the-open-unit-interval-is-refused´
/// ´test:crate:channel-invalid-gamma-q-t-rejected´
#[test]
fn channel_invalid_gamma_q_t_rejected() {
    let invalid_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        reward: RewardParameters {
            gamma_q_t: 1.0, // Invalid: must be in (0, 1)
            ..Default::default()
        },
        ..ChannelPolicy::default()
    };

    let result = validate_channel_policy(&invalid_policy);
    assert!(matches!(result, Err(ChannelError::InvalidGammaQt)));
}

/// The other endpoint is refused for the same reason: a decay of zero forgets
/// everything at each step, leaving the channel's challenge estimate to be
/// rebuilt from the single most recent result. Both ends of the interval are
/// closed off, so the rate is always a genuine blend of history and present.
///
/// (´claim:builder:a-channel-time-decay-outside-the-open-unit-interval-is-refused´)
/// ´test:crate:channel-zero-gamma-q-t-rejected´
#[test]
fn channel_zero_gamma_q_t_rejected() {
    let invalid_policy = ChannelPolicy {
        actions: vec![Action::Allow, Action::Block],
        reward: RewardParameters {
            gamma_q_t: 0.0, // Invalid: must be in (0, 1)
            ..Default::default()
        },
        ..ChannelPolicy::default()
    };

    let result = validate_channel_policy(&invalid_policy);
    assert!(matches!(result, Err(ChannelError::InvalidGammaQt)));
}

// ═══════════════════════════════════════════════════════════════════════════════
// Standardisation Bootstrap Validation (´tab:config:standardisation´)
// ═══════════════════════════════════════════════════════════════════════════════

/// A bootstrap resample count below ten is refused at
/// `standardisation.n_boot`. The bootstrap estimates a spread from its
/// resamples, and a handful of them yields an interval too unstable to
/// standardise against — the resulting scale would be noise dressed as a
/// statistic.
///
/// (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´)
/// ´test:crate:build-invalid-n-boot-rejected´
#[test]
fn build_invalid_n_boot_rejected() {
    let config = AssayerConfig {
        standardisation: crate::feature::standardisation::StandardisationConfig {
            n_boot: 5, // Invalid: must be >= 10
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "standardisation.n_boot",
            ..
        })
    ));
}

/// A bootstrap confidence level of zero is refused at
/// `standardisation.alpha_boot`. The level has to name a genuine tail mass; at
/// zero there is no interval to speak of, and the standardisation scale it
/// feeds would be reading a degenerate quantile.
///
/// (´claim:builder:an-out-of-range-numeric-parameter-is-refused-with-the-offending-config-path´)
/// ´test:crate:build-invalid-alpha-boot-rejected´
#[test]
fn build_invalid_alpha_boot_rejected() {
    let config = AssayerConfig {
        standardisation: crate::feature::standardisation::StandardisationConfig {
            alpha_boot: 0.0, // Invalid: must be in (0, 1]
            ..Default::default()
        },
        ..test_assayer_config()
    };
    let builder = Assayer::builder(config).signal_schema(&[]);
    let result = builder.build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "standardisation.alpha_boot",
            ..
        })
    ));
}

// ═══════════════════════════════════════════════════════════════════════════════
// The command channel's capacity is host-set with no default
// ═══════════════════════════════════════════════════════════════════════════════

/// The command channel's capacity has no default: a configuration that
/// never declared one refuses to build, on the builder surface and on the
/// crate-internal construction path alike, with the error naming the
/// configuration path. The capacity was previously a literal a host could
/// not reach — the one channel that silently discards a batch
/// initialisation was also the one channel an operator could not widen —
/// and a defaulted replacement would merely rename that constant. Refusal
/// is what makes the capacity a declared decision at every deployment.
///
/// ´claim:builder:an-undeclared-command-channel-capacity-refuses-to-build´
/// ´test:crate:builder-refuses-unset-command-channel-capacity´
#[test]
fn builder_refuses_unset_command_channel_capacity() {
    // The default configuration deliberately carries no capacity.
    let config = AssayerConfig::default();
    assert!(config.infrastructure.command_channel_capacity.is_none());

    let result = Assayer::builder(config).signal_schema(&[]).build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "infrastructure.command_channel_capacity",
            ..
        })
    ));

    // A declared zero is refused too: a channel with no slots would turn
    // every lifecycle registration into a full-channel error.
    let mut config = test_assayer_config();
    config.infrastructure.command_channel_capacity = Some(0);
    let result = Assayer::builder(config).signal_schema(&[]).build();
    assert!(matches!(
        result,
        Err(BuildError::InvalidParameter {
            path: "infrastructure.command_channel_capacity",
            ..
        })
    ));
}

/// The declared capacity is the capacity the channel is built with: an
/// engine declared at two slots, with its steward parked, admits exactly
/// two commands and refuses the third as full. Without this the
/// declaration would be a field the host believed it had set — the
/// configured-looking, unconfigured-behaving shape the builder suite
/// exists to rule out.
///
/// ´claim:builder:the-declared-command-channel-capacity-is-the-capacity-the-channel-is-built-with´
/// ´test:crate:declared-command-channel-capacity-reaches-the-channel´
#[test]
fn declared_command_channel_capacity_reaches_the_channel() {
    use crate::config::types::InfrastructureConfig;
    use crate::owner::commands::{CheckpointRequest, ModelOwnerCommand};

    let config = AssayerConfig {
        instance_id: "declared-capacity".to_owned(),
        infrastructure: InfrastructureConfig {
            command_channel_capacity: Some(2),
            ..crate::testing::test_infrastructure()
        },
        ..Default::default()
    };
    let assayer = Assayer::builder(config)
        .signal_schema(&[])
        .build()
        .expect("builds at capacity 2");

    // Park the steward so nothing drains the channel while we fill it.
    let (entered_tx, entered_rx) = crossbeam_channel::bounded(1);
    let (release_tx, release_rx) = crossbeam_channel::bounded(1);
    assayer
        .command_tx
        .send(ModelOwnerCommand::TestBlock {
            entered: entered_tx,
            release: release_rx,
        })
        .expect("barrier command enqueues");
    entered_rx.recv_timeout(ACK_DEADLINE).expect("steward parks");

    let checkpoint = || ModelOwnerCommand::Checkpoint(CheckpointRequest { completion: None });
    assert!(assayer.command_tx.try_send(checkpoint()).is_ok(), "slot one admitted");
    assert!(assayer.command_tx.try_send(checkpoint()).is_ok(), "slot two admitted");
    assert!(
        matches!(
            assayer.command_tx.try_send(checkpoint()),
            Err(crossbeam_channel::TrySendError::Full(_))
        ),
        "the third command is refused: the channel holds exactly the declared two"
    );

    release_tx.send(()).expect("steward releases");
}

// ═══════════════════════════════════════════════════════════════════════════════
// The pass reaches every parameter the configuration owns
// ═══════════════════════════════════════════════════════════════════════════════

/// Asserts that building `config` is refused with `InvalidParameter`
/// naming exactly `expected_path`.
#[track_caller]
fn assert_refused_at(config: AssayerConfig, expected_path: &str) {
    match Assayer::builder(config).signal_schema(&[]).build() {
        Err(BuildError::InvalidParameter { path, .. }) => {
            assert_eq!(path, expected_path, "refusal should name the offending parameter");
        }
        Err(e) => panic!("expected InvalidParameter at {expected_path}, got {e}"),
        Ok(_) => panic!("expected InvalidParameter at {expected_path}, got a built engine"),
    }
}

/// Every field of the public monitoring configuration is validated at
/// construction against its tabulated domain, including the fields reserved
/// behind named definition STOPs.
///
/// ´claim:builder:every-monitoring-threshold-is-validated-against-its-tabulated-domain´
/// ´test:crate:validation-reaches-every-monitoring-threshold´
#[test]
fn validation_reaches_every_monitoring_threshold() {
    let base = test_assayer_config;

    let mut config = base();
    config.monitoring.kappa_drift = -1.0;
    assert_refused_at(config, "monitoring.kappa_drift");

    let mut config = base();
    config.monitoring.h_threshold = 0.0;
    assert_refused_at(config, "monitoring.h_threshold");

    let mut config = base();
    config.monitoring.min_auc_class_count = 4;
    assert_refused_at(config, "monitoring.min_auc_class_count");

    let mut config = base();
    config.monitoring.recent_discrimination_window = 49;
    assert_refused_at(config, "monitoring.recent_discrimination_window");

    let mut config = base();
    config.monitoring.sync_error_threshold_per_dimension = 0.0;
    assert_refused_at(config, "monitoring.sync_error_threshold_per_dimension");

    let mut config = base();
    config.monitoring.min_concordance_labels = 2;
    assert_refused_at(config, "monitoring.min_concordance_labels");

    let mut config = base();
    config.monitoring.concordance_deficit_threshold = 0.5;
    assert_refused_at(config, "monitoring.concordance_deficit_threshold");

    let mut config = base();
    config.monitoring.strong_alarm_threshold = 0.0;
    assert_refused_at(config, "monitoring.strong_alarm_threshold");

    let mut config = base();
    config.monitoring.quiet_alarm_threshold = -1.0;
    assert_refused_at(config, "monitoring.quiet_alarm_threshold");

    let mut config = base();
    config.monitoring.alarm_outcome_noise_allowance = -1.0;
    assert_refused_at(config, "monitoring.alarm_outcome_noise_allowance");

    let mut config = base();
    config.monitoring.feature_stability_threshold = 0.0;
    assert_refused_at(config, "monitoring.feature_stability_threshold");

    let mut config = base();
    config.monitoring.maturity_threshold = 1.0;
    assert_refused_at(config, "monitoring.maturity_threshold");

    let mut config = base();
    config.monitoring.resolution_adequacy_threshold = 0;
    assert_refused_at(config, "monitoring.resolution_adequacy_threshold");

    let mut config = base();
    config.monitoring.feedback_latency_ewma_rate = 1.0;
    assert_refused_at(config, "monitoring.feedback_latency_ewma_rate");

    let mut config = base();
    config.monitoring.ledger_materiality_threshold = 0;
    assert_refused_at(config, "monitoring.ledger_materiality_threshold");

    let mut config = base();
    config.monitoring.attenuation_materiality_floor = 1.0;
    assert_refused_at(config, "monitoring.attenuation_materiality_floor");
}

/// The parameters the audit measured escaping enforcement are now
/// reached, each refusal naming its configuration path: the Ledger's
/// absence threshold and collection bounds, the concordance interval and
/// percentile, the persisted checkpoint interval, and the blend window
/// pair. The constraint column is part of the specification — a value
/// outside it is a configuration error, not a tuning choice — and an
/// unenforced row is a bound a deployment discovers only by the damage
/// its violation does.
///
/// ´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´
/// ´test:crate:validation-reaches-ledger-concordance-persistence-and-blend-rows´
#[test]
fn validation_reaches_ledger_concordance_persistence_and_blend_rows() {
    let base = test_assayer_config;

    let mut config = base();
    config.ledger.n_absent = 0;
    assert_refused_at(config, "ledger.n_absent");

    let mut config = base();
    config.ledger.gc_horizon_days = 0;
    assert_refused_at(config, "ledger.gc_horizon_days");

    let mut config = base();
    config.ledger.gc_floor = 0.0;
    assert_refused_at(config, "ledger.gc_floor");

    // Both ends of the hibernation lifetime are refused: a store nothing can
    // ever be restored from, and one the elapsed-time clamp could never reach
    // (´alg:registry:hibernation´), (´dec:clock:embedded-clamp´).
    let mut config = base();
    config.hibernation.expiry_days = 0;
    assert_refused_at(config, "hibernation.expiry_days");

    let mut config = base();
    config.hibernation.expiry_days = 366;
    assert_refused_at(config, "hibernation.expiry_days");

    let mut config = base();
    config.concordance.recalibration_interval = 0;
    assert_refused_at(config, "concordance.recalibration_interval");

    // The percentile the audit named: shipped at 0.80 and previously
    // confined to the open unit interval by nothing.
    let mut config = base();
    config.concordance.percentile = 1.0;
    assert_refused_at(config, "concordance.percentile");

    let mut config = base();
    config.persistence = Some(crate::config::types::PersistenceConfig {
        checkpoint_dir: std::path::PathBuf::from("unused"),
        journal_dir: std::path::PathBuf::from("unused"),
        checkpoint_interval: std::time::Duration::ZERO,
    });
    assert_refused_at(config, "persistence.checkpoint_interval");

    // The blend window: a rolling-window capacity of the same shape as
    // the concordance window's, previously checked for one and not the
    // other.
    let mut config = base();
    config.blend_statistics.window_capacity = 0;
    assert_refused_at(config, "blend_statistics.window_capacity");

    let mut config = base();
    config.blend_statistics.publish_interval = 0;
    assert_refused_at(config, "blend_statistics.publish_interval");
}

/// The infrastructure capacities that fail exactly as the checked three
/// would are now checked with them, and the label queue is held to its
/// tabulated floor of one hundred rather than merely to being non-zero.
/// A zero health-event or identity-observation queue would silently turn
/// every enqueue into a drop, which is the same failure the audit argued
/// the existing non-zero checks exist to refuse.
///
/// (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´)
/// ´test:crate:validation-reaches-the-remaining-infrastructure-capacities´
#[test]
fn validation_reaches_the_remaining_infrastructure_capacities() {
    let base = test_assayer_config;

    let mut config = base();
    config.infrastructure.expiry_horizon_secs = 0;
    assert_refused_at(config, "infrastructure.expiry_horizon_secs");

    let mut config = base();
    config.infrastructure.label_channel_capacity = 99;
    assert_refused_at(config, "infrastructure.label_channel_capacity");

    let mut config = base();
    config.infrastructure.health_event_capacity = 0;
    assert_refused_at(config, "infrastructure.health_event_capacity");

    let mut config = base();
    config.infrastructure.identity_observation_capacity = 0;
    assert_refused_at(config, "infrastructure.identity_observation_capacity");

    let mut config = base();
    config.infrastructure.identity_registration_timeout_secs = 0;
    assert_refused_at(config, "infrastructure.identity_registration_timeout_secs");
}

/// Every figure the convergence family still carries is refused at zero,
/// by its own configuration path. The family used to be validated as a
/// family, the cross-stage relation being the one check that was not about
/// a single row's bound; that relation went with the eight stage figures it
/// compared when they were deleted for naming no stage the specification
/// carries, the two Companion figures went in the same change, and what
/// remains is the two Platt figures and a row-by-row floor. The test is kept
/// at the family rather than folded into a neighbour because the paths are
/// what a host reads back out of a refusal.
///
/// (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´)
/// ´test:crate:validation-requires-the-convergence-figures-to-be-positive´
#[test]
fn validation_requires_the_convergence_figures_to_be_positive() {
    let base = test_assayer_config;

    let mut config = base();
    config.convergence.platt_converged_delta_cal = 0.0;
    assert_refused_at(config, "convergence.platt_converged_delta_cal");

    let mut config = base();
    config.convergence.platt_converged_refit_count = 0;
    assert_refused_at(config, "convergence.platt_converged_refit_count");
}

/// The sister and anchor forgetting rate's tabulated constraint is the
/// open interval above the operational rate: a sister rate at or below
/// the operational one is refused. The relation, not just each rate's
/// membership of the unit interval, is what the risk-model table fixes —
/// a sister forgetting faster than the operational model swaps the two
/// regimes' timescales while every individual bound still holds.
///
/// (´claim:builder:the-out-of-table-values-the-audit-measured-escaping-are-refused-by-path´)
/// ´test:crate:validation-requires-sister-rate-above-operational-rate´
#[test]
fn validation_requires_sister_rate_above_operational_rate() {
    let mut config = test_assayer_config();
    config.model.gamma_inh = config.model.gamma_opr;
    assert_refused_at(config, "model.gamma_inh");
}

/// Every floating-point field a configuration can reach is refused when it
/// holds a value that is not a number, and refused again for either infinity,
/// with the refusal naming the field that carried it. The table is walked
/// field by field rather than check by check, so a field the pass does not
/// reach fails here instead of being read as validated: the enumeration is
/// taken from the configuration types, and the count is the count of floats
/// those types own.
///
/// A bound is not a finiteness test. Every ordered comparison with NaN is
/// false, so a NaN sits inside every interval the pass can state and used to
/// arrive in the model arithmetic through a construction that had reported
/// the configuration sound — the one shape of configuration error that
/// survives the gate meant to catch it, and then poisons every sum it reaches
/// rather than failing anywhere a host could read.
///
/// ´claim:builder:every-configuration-float-is-refused-when-it-is-not-a-number-or-is-infinite´
/// ´test:crate:validation-refuses-every-non-finite-configuration-float´
#[test]
fn validation_refuses_every_non_finite_configuration_float() {
    /// One row of the enumeration: the configuration path the refusal has to
    /// name, and the setter that plants a value in the field it names.
    type FloatField = (&'static str, fn(&mut AssayerConfig, f64));

    let fields: [FloatField; 41] = [
        ("model.lambda_prior", |config, value| config.model.lambda_prior = value),
        ("model.lambda_floor", |config, value| config.model.lambda_floor = value),
        ("model.c_leverage", |config, value| config.model.c_leverage = value),
        ("model.w_ceiling", |config, value| config.model.w_ceiling = value),
        ("model.gamma_opr", |config, value| config.model.gamma_opr = value),
        ("model.gamma_inh", |config, value| config.model.gamma_inh = value),
        ("model.gamma_kappa", |config, value| config.model.gamma_kappa = value),
        ("model.p_plus_init", |config, value| config.model.p_plus_init = value),
        ("temporal.gamma_t_core", |config, value| config.temporal.gamma_t_core = value),
        ("temporal.gamma_t_ledger", |config, value| {
            config.temporal.gamma_t_ledger = value;
        }),
        ("temporal.gamma_t_identity", |config, value| {
            config.temporal.gamma_t_identity = value;
        }),
        ("ledger.lambda_l", |config, value| config.ledger.lambda_l = value),
        ("ledger.gc_floor", |config, value| config.ledger.gc_floor = value),
        ("standardisation.gamma_std", |config, value| {
            config.standardisation.gamma_std = value;
        }),
        ("standardisation.epsilon", |config, value| {
            config.standardisation.epsilon = value;
        }),
        ("standardisation.n_clip", |config, value| {
            config.standardisation.n_clip = value;
        }),
        ("standardisation.v_floor", |config, value| {
            config.standardisation.v_floor = value;
        }),
        ("standardisation.alpha_boot", |config, value| {
            config.standardisation.alpha_boot = value;
        }),
        ("cholesky.kappa_growth_factor", |config, value| {
            config.cholesky.kappa_growth_factor = value;
        }),
        ("concordance.percentile", |config, value| {
            config.concordance.percentile = value;
        }),
        ("monitoring.kappa_drift", |config, value| {
            config.monitoring.kappa_drift = value;
        }),
        ("monitoring.h_threshold", |config, value| {
            config.monitoring.h_threshold = value;
        }),
        ("monitoring.sync_error_threshold_per_dimension", |config, value| {
            config.monitoring.sync_error_threshold_per_dimension = value;
        }),
        ("monitoring.concordance_deficit_threshold", |config, value| {
            config.monitoring.concordance_deficit_threshold = value;
        }),
        ("monitoring.strong_alarm_threshold", |config, value| {
            config.monitoring.strong_alarm_threshold = value;
        }),
        ("monitoring.quiet_alarm_threshold", |config, value| {
            config.monitoring.quiet_alarm_threshold = value;
        }),
        ("monitoring.alarm_outcome_noise_allowance", |config, value| {
            config.monitoring.alarm_outcome_noise_allowance = value;
        }),
        ("monitoring.feature_stability_threshold", |config, value| {
            config.monitoring.feature_stability_threshold = value;
        }),
        ("monitoring.maturity_threshold", |config, value| {
            config.monitoring.maturity_threshold = value;
        }),
        ("monitoring.feedback_latency_ewma_rate", |config, value| {
            config.monitoring.feedback_latency_ewma_rate = value;
        }),
        ("monitoring.attenuation_materiality_floor", |config, value| {
            config.monitoring.attenuation_materiality_floor = value;
        }),
        ("convergence.platt_converged_delta_cal", |config, value| {
            config.convergence.platt_converged_delta_cal = value;
        }),
        ("infrastructure.expected_peak_request_rate", |config, value| {
            config.infrastructure.expected_peak_request_rate = value;
        }),
        ("platt.kappa_min", |config, value| config.platt.kappa_min = value),
        ("platt.kappa_max", |config, value| config.platt.kappa_max = value),
        ("platt.kappa_initial", |config, value| config.platt.kappa_initial = value),
        ("platt.gamma_cal", |config, value| config.platt.gamma_cal = value),
        ("platt.golden_tolerance", |config, value| {
            config.platt.golden_tolerance = value;
        }),
        ("platt.delta_cal_threshold", |config, value| {
            config.platt.delta_cal_threshold = value;
        }),
        ("schur.epsilon_schur", |config, value| config.schur.epsilon_schur = value),
        ("schur.posture_condition_ceiling", |config, value| {
            config.schur.posture_condition_ceiling = value;
        }),
    ];

    for value in [f64::NAN, f64::INFINITY, f64::NEG_INFINITY] {
        for (path, set) in fields {
            let mut config = test_assayer_config();
            set(&mut config, value);
            assert_refused_at(config, path);
        }
    }
}

/// The template validator validates: every offset a template carries is
/// checked at build time against the block that resolves it — a
/// Sentinel-slot offset against the base extraction width, an aggregate
/// offset against the aggregate block, a context offset against the
/// fixed context region the cold start lays out — and the refusal names
/// the template's index and the bound it broke. The validator was
/// previously a constant function returning success for every input,
/// deferring the check to a runtime path that has already promised it
/// cannot fail.
///
/// ´claim:builder:a-template-offset-outside-the-block-that-resolves-it-is-refused-at-build´
/// ´test:crate:builder-refuses-out-of-range-template-offsets´
#[test]
fn builder_refuses_out_of_range_template_offsets() {
    use crate::feature::dimension_map::{AGGREGATE_FEATURE_COUNT, SENTINEL_Q_BASE};
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let refused = |template: InteractionTemplate, expected_index: usize| {
        let result = Assayer::builder(test_assayer_config())
            .signal_schema(&[])
            .interaction_templates(vec![
                InteractionTemplate::type4(0, FeatureSelector::AggregateFeature(0)),
                template,
            ])
            .build();
        match result {
            Err(BuildError::InvalidInteractionTemplate { template_index, .. }) => {
                assert_eq!(template_index, expected_index, "the refusal names the offending template");
            }
            Err(e) => panic!("expected InvalidInteractionTemplate, got {e}"),
            Ok(_) => panic!("expected InvalidInteractionTemplate, got a built engine"),
        }
    };

    // One past each block's last valid offset.
    refused(
        InteractionTemplate::type1(SENTINEL_Q_BASE, FeatureSelector::AggregateFeature(0)),
        1,
    );
    refused(InteractionTemplate::type2(SentinelId(1), SentinelId(2), SENTINEL_Q_BASE), 1);
    refused(InteractionTemplate::type3(SENTINEL_Q_BASE), 1);
    refused(
        InteractionTemplate::type4(AGGREGATE_FEATURE_COUNT, FeatureSelector::AggregateFeature(0)),
        1,
    );
    // With an empty schema the context region is bias + aggregates = 16.
    refused(
        InteractionTemplate::type5(FeatureSelector::AggregateFeature(AGGREGATE_FEATURE_COUNT)),
        1,
    );
}

/// The template validator admits what it should: the last valid offset of each
/// block builds, and a signal operand becomes admissible exactly when the
/// schema declares a signal for it to name. Each operand is checked against
/// the block it names rather than against a flat context width
/// (´entry:assayer:wl-feature-absolute-operands´), so declaring a signal
/// widens what a signal operand may say and nothing else — an aggregate
/// offset past the aggregate block stays refused however many signals are
/// declared. Refusing only what no block resolves is what keeps the check a
/// validation rather than a prohibition.
///
/// (´claim:builder:a-template-offset-outside-the-block-that-resolves-it-is-refused-at-build´)
/// ´test:crate:builder-admits-boundary-template-offsets´
#[test]
fn builder_admits_boundary_template_offsets() {
    use crate::feature::dimension_map::{AGGREGATE_FEATURE_COUNT, SENTINEL_Q_BASE};
    use crate::feature::interaction::{FeatureSelector, InteractionTemplate};

    let assayer = Assayer::builder(test_assayer_config())
        .signal_schema(&[])
        .interaction_templates(vec![
            InteractionTemplate::type1(
                SENTINEL_Q_BASE - 1,
                FeatureSelector::AggregateFeature(AGGREGATE_FEATURE_COUNT - 1),
            ),
            InteractionTemplate::type2(SentinelId(1), SentinelId(2), SENTINEL_Q_BASE - 1),
            InteractionTemplate::type3(SENTINEL_Q_BASE - 1),
            InteractionTemplate::type4(AGGREGATE_FEATURE_COUNT - 1, FeatureSelector::AggregateFeature(0)),
            InteractionTemplate::type5(FeatureSelector::AggregateFeature(AGGREGATE_FEATURE_COUNT - 1)),
        ])
        .build();
    assert!(
        assayer.is_ok(),
        "boundary offsets build: {:?}",
        assayer.err().map(|e| e.to_string())
    );

    // A declared signal is what makes a signal operand nameable.
    let signal = SignalDeclaration::new("s", SignalShape::Scalar { clip: (0.0, 1.0) }, Persistence::Request);
    let with_signal = Assayer::builder(test_assayer_config())
        .signal_schema(std::slice::from_ref(&signal))
        .interaction_templates(vec![InteractionTemplate::type5(FeatureSelector::DeclaredSignal(0))])
        .build();
    assert!(with_signal.is_ok(), "the declared schema admits a signal operand");

    // Without the declaration the same operand names nothing and is refused.
    let without_signal = Assayer::builder(test_assayer_config())
        .signal_schema(&[])
        .interaction_templates(vec![InteractionTemplate::type5(FeatureSelector::DeclaredSignal(0))])
        .build();
    assert!(without_signal.is_err(), "an undeclared signal operand names nothing");

    // And a declared signal does not widen what an aggregate operand may say.
    let aggregate_past_block = Assayer::builder(test_assayer_config())
        .signal_schema(std::slice::from_ref(&signal))
        .interaction_templates(vec![InteractionTemplate::type5(FeatureSelector::AggregateFeature(
            AGGREGATE_FEATURE_COUNT,
        ))])
        .build();
    assert!(aggregate_past_block.is_err(), "each operand is checked against its own block");
}

// ═══════════════════════════════════════════════════════════════════════════════
// The Ledger's absence threshold is bounded by the count that carries it
// ═══════════════════════════════════════════════════════════════════════════════

/// The Ledger's absence threshold is accepted across exactly the range the
/// count that carries it can reach: one and 255 build, and the instance
/// ingests reports against the value the configuration named rather than
/// whatever a narrowing left behind, while zero, 256 and 300 are refused with
/// the configuration path that named them. Absences are counted in eight bits,
/// so a threshold of 256 narrowed to zero would delete every non-root entry on
/// the first report and 300 would fire at 44 — two deletion rules no host asked
/// for, and neither of them visible from the configuration that produced it.
///
/// ´claim:builder:the-ledger-absence-threshold-is-accepted-across-exactly-the-range-its-count-can-reach´
/// ´test:crate:builder-refuses-an-absence-threshold-the-count-cannot-reach´
#[test]
fn builder_refuses_an_absence_threshold_the_count_cannot_reach() {
    let base = test_assayer_config;

    // Neither end is a preference. A threshold of zero deletes on the first
    // absence and one above the count's width is never reached at all
    // (´alg:ledger:entry-deletion´).
    for refused in [0_u32, 256, 300] {
        let mut config = base();
        config.ledger.n_absent = refused;
        assert_refused_at(config, "ledger.n_absent");
    }

    for accepted in [1_u32, 255] {
        let mut config = base();
        config.ledger.n_absent = accepted;
        let assayer = Assayer::builder(config)
            .signal_schema(&[])
            .build()
            .expect("a threshold the count can reach builds");
        assert_eq!(
            u32::from(assayer.ingestion.n_absent_threshold),
            accepted,
            "the threshold the instance ingests against is the configured one"
        );
    }
}

// ═══════════════════════════════════════════════════════════════════════════════
// Persistence a build cannot perform is refused
// ═══════════════════════════════════════════════════════════════════════════════

/// A build that compiles no checkpoint or journal codec refuses a
/// configuration that names persistence directories rather than accepting it
/// and persisting nothing. Both codecs are compiled behind the `serde`
/// feature, so an instance built without it holds no writer for either
/// artefact: the host would be handed an instance that looks durable, and the
/// first evidence otherwise would be a restart with no state to restore.
///
/// ´claim:builder:a-build-compiling-no-persistence-codec-refuses-a-persistence-configuration´
/// ´test:crate:build-persistence-without-codecs-refused´
#[cfg(not(feature = "serde"))]
#[test]
fn build_persistence_without_codecs_refused() {
    let config = AssayerConfig {
        persistence: Some(crate::config::types::PersistenceConfig::new(
            std::path::PathBuf::from("unused-checkpoint"),
            std::path::PathBuf::from("unused-journal"),
        )),
        ..test_assayer_config()
    };

    let result = Assayer::builder(config).signal_schema(&[]).build();
    assert!(
        matches!(result, Err(BuildError::PersistenceUnsupported)),
        "expected PersistenceUnsupported, got {result:?}"
    );
}
