// SPDX-License-Identifier: AGPL-3.0-only
// SPDX-FileCopyrightText: 2026 Torrust project contributors

//! Tests for [`crate::config`] — the parameters that fix how the sentinel
//! observes, and the validation that decides which combinations of them are
//! coherent enough to run.
//!
//! The configuration is deliberately free of policy: every field controls how
//! the sentinel measures and learns, never what it thinks about what it sees.
//! That division is why validation can be purely local. Each bound exists
//! because a value outside it would make some piece of arithmetic meaningless
//! — a forgetting factor of one never forgets, a stability constant of zero
//! guards no denominator, a clip width of zero rejects every observation —
//! and not because the host's judgement was second-guessed.
//!
//! Three shapes recur. Rates and fractions live strictly inside the unit
//! interval, both endpoints being degenerate. Sizes and capacities must be at
//! least one, since a ceiling of zero leaves the sentinel nothing to work
//! with. And a few fields are bound to each other rather than to constants:
//! the CUSUM reference must be slower than the baseline it judges, eviction
//! depth must sit strictly deeper than creation depth, and the node budget
//! must exceed the headroom the gap between those depths implies.
//!
//! Validation collects every violation rather than stopping at the first, so
//! a host repairs its configuration in one pass. Advice is kept separate from
//! refusal: a combination that is arithmetically sound but empirically poor —
//! warm-up rounds too few for the configured memory — comes back as a warning
//! the host may ignore, never as an error that stops it.
//!
//! # Test index
//!
//! | Test | Area | Claim |
//! |------|------|-------|
//! | [`default_config_is_valid`] | config | The parameters the sentinel ships with satisfy every rule it checks them against. A host that configures nothing at all therefore starts from a coherent measurement setup rather than from a template it must first repair. |
//! | [`default_config_field_values`] | config | Each default holds the value its documentation promises — the long-memory forgetting factor, the modest rank ceiling, the analysis and graph budgets, the noise batch and its fixed seed. The defaults are a calibrated set rather than arbitrary placeholders, so documenting them and shipping them are held to be the same act. |
//! | [`rejects_max_rank_zero`] | config | A capacity ceiling of zero leaves the sentinel nothing to work with: a subspace tracker allowed no basis vectors can model nothing at all, so the value is refused rather than quietly read as a request for a disabled tracker. The refusal names that field and faults nothing else in an otherwise default configuration. |
//! | [`rejects_forgetting_factor_out_of_range`] | config | The forgetting factor must lie strictly inside the unit interval: at one the baseline never forgets and can never adapt, at zero it retains nothing, and outside the interval the exponential weighting stops being a weighting. Both endpoints are refused along with values beyond them, and the refusal carries back the value it saw so the host can tell which of its inputs was faulted. |
//! | [`accepts_forgetting_factor_near_boundaries`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`rejects_rank_update_interval_zero`] | config | cites (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´) |
//! | [`rejects_energy_threshold_out_of_range`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`rejects_eps_not_positive`] | config | The stability constant exists to keep denominators away from zero, so a value at or below zero defeats the only thing it is for. Both are refused, and the refusal reports the offending value rather than silently substituting a workable one. |
//! | [`rejects_cusum_slow_decay_out_of_range`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`rejects_cusum_slow_decay_below_forgetting`] | config | The CUSUM reference must have longer memory than the baseline it is measured against — strictly slower, not merely as slow. A reference adapting as fast as the baseline would follow a gradual drift instead of exposing it, and exposing exactly that drift is what the accumulator exists for. |
//! | [`accepts_cusum_slow_decay_just_above_forgetting`] | config | cites (´claim:config:the-cusum-reference-must-decay-strictly-slower-than-the-baseline-it-judges´) |
//! | [`rejects_cusum_coord_slow_decay_out_of_range`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`rejects_cusum_coord_slow_decay_below_forgetting`] | config | cites (´claim:config:the-cusum-reference-must-decay-strictly-slower-than-the-baseline-it-judges´) |
//! | [`rejects_cusum_allowance_sigmas_negative`] | config | The CUSUM allowance is subtracted from each step's gap before anything accumulates, so a negative allowance would be added to every gap and would manufacture evidence of drift out of ordinary noise. Negative values are therefore refused. |
//! | [`accepts_cusum_allowance_sigmas_zero`] | config | cites (´claim:config:a-noise-allowance-may-never-be-negative-because-it-would-manufacture-the-drift-it-absorbs´) |
//! | [`rejects_clip_sigmas_not_positive`] | config | The clip width bounds how far above the mean an observation may sit and still update the baseline. At zero or below nothing would ever clear the bar, so the baseline would starve rather than be protected — the value is refused instead of being allowed to silence the very updates it guards. |
//! | [`rejects_clip_pressure_decay_out_of_range`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`accepts_clip_pressure_decay_valid`] | config | cites (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´) |
//! | [`rejects_analysis_k_zero`] | config | cites (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´) |
//! | [`accepts_analysis_k_one`] | config | cites (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´) |
//! | [`accepts_analysis_depth_cutoff_zero`] | config | A depth cutoff of zero is admitted rather than refused, and it means something usable: only the V-Tree root remains eligible, which effectively turns adaptive selection off. Zero is a fault in a capacity but a legitimate setting for a depth gate, because a gate that admits nothing deep expresses a policy rather than an incoherence. |
//! | [`rejects_split_threshold_zero`] | config | A split threshold of zero would let a cell subdivide before accumulating any intensity at all, so the graph would fragment on its first observation instead of on evidence of sustained traffic. The threshold must be strictly positive. |
//! | [`rejects_d_create_zero`] | config | cites (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´) |
//! | [`rejects_d_evict_not_greater_than_d_create`] | config | Eviction depth must sit strictly deeper than creation depth, and an equal pair is refused just as an inverted one is. The gap between them is the buffer zone — entries too deep to create children but not yet deep enough to be evicted — so collapsing it would leave a cell eligible for eviction the moment it stopped being eligible to grow. |
//! | [`accepts_d_evict_one_above_d_create`] | config | cites (´claim:config:the-eviction-depth-must-sit-strictly-deeper-than-the-creation-depth-so-a-buffer-zone-exists´) |
//! | [`rejects_budget_zero`] | config | cites (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´) |
//! | [`rejects_budget_below_headroom`] | config | The node budget must exceed what the depth gates themselves imply: a buffer zone of a given width can hold a number of nodes growing as a power of three, and a budget merely equal to that leaves the graph no room to manoeuvre inside its own gates. Equality is refused, not merely shortfall. |
//! | [`accepts_budget_just_above_headroom`] | config | cites (´claim:config:the-node-budget-must-exceed-the-headroom-the-depth-gates-imply-and-equalling-it-is-not-enough´) |
//! | [`rejects_depth_buffer_whose_headroom_cannot_be_represented`] | config | Past a certain width the headroom the depth gates imply stops being a number the machine can hold, and the depth pair is refused on its own terms rather than measured against a figure that wrapped. The requirement grows as a power of three, so a buffer in the forties already exceeds the addressable range; computing it and comparing anyway would either abort the validation that promised to return its faults, or silently compare the budget against a small wrapped remainder and admit a configuration that cannot hold. The refusal names the two depths, since they are what the host must change. |
//! | [`reports_a_shortfall_at_the_widest_representable_depth_buffer`] | config | cites (´claim:config:a-depth-buffer-whose-headroom-cannot-be-represented-is-refused-on-its-own-terms´) |
//! | [`rejects_nan_in_every_floating_point_field`] | config | Every floating-point field refuses a non-number, because the ordered comparisons that police the other values cannot see one. A comparison against a non-number is false whichever way it is written, so a bound expressed as a pair of comparisons admits it silently — and the value then spreads, since every product and sum it enters returns a non-number too. A forgetting factor admitted this way reaches the baseline arithmetic and leaves every score afterwards unusable, with nothing in the report to say which field was responsible. The guard therefore sits ahead of the bound rather than inside it. |
//! | [`accepts_infinite_clip_width_and_refuses_infinite_rates`] | config | An infinite value is admitted where the interval is one-sided, because there it names a real limit rather than the absence of one. An infinite clip width is the unclipped configuration — the control arm the package's own clipping study runs against — and it compares correctly against every bound it is checked with, which is precisely what a non-number does not do. The fields whose intervals are two-sided still refuse it, and they refuse it through the bound they already carry rather than through a separate guard. |
//! | [`rejects_noise_batch_size_zero_when_enabled`] | config | A noise batch of no samples is a fault only when the schedule actually asks for rounds: with an active schedule the warm-up would run rounds that feed the tracker nothing. The check is conditional on the schedule rather than absolute, because zero samples per round is coherent when there are no rounds to run. |
//! | [`accepts_noise_batch_size_zero_when_disabled`] | config | cites (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´) |
//! | [`rejects_noise_batch_size_zero_with_geometric_root_and_zero_floor`] | config | cites (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´) |
//! | [`accepts_noise_batch_size_zero_with_geometric_root_and_floor_zero`] | config | cites (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´) |
//! | [`accepts_noise_seed_none`] | config | An absent noise seed is valid and simply changes where the randomness comes from: with a seed the warm-up is reproducible across restarts, without one it is drawn from system entropy. Determinism is offered rather than required, so neither choice counts as a misconfiguration. |
//! | [`rejects_geometric_decay_out_of_range`] | config | The geometric decay is bound to a half-open interval rather than the open one other rates get: values at or below zero and above one are refused, but one itself is not. Zero would collapse the schedule onto its floor immediately, whereas a schedule that never tapers with depth is a legitimate thing to ask for. |
//! | [`rejects_geometric_decay_nan`] | config | A decay that is not a number is refused explicitly, because every comparison against it is false and a range check alone would let it through. The configuration is rejected before such a value could reach the exponentiation and turn every round count into nonsense. |
//! | [`rejects_geometric_root_zero_with_positive_min`] | config | A geometric schedule that starts at no rounds but floors at a positive count contradicts itself: the taper only ever descends from the root, so every depth would be lifted to the floor and the root would describe nothing. That combination is refused — which is why the root is faulted only when the floor is positive. |
//! | [`collects_multiple_errors`] | config | Validation reports every violation it finds rather than stopping at the first, so a host with several bad fields learns of all of them in one pass instead of discovering them one restart at a time. Faulting several fields at once yields at least as many errors, each naming its own field. |
//! | [`geometric_rounds_default`] | config | A geometric schedule scales its round count by the decay at each level of depth and then rests on its floor. Deeper cells are narrower and converge in fewer rounds, so tapering matches warm-up effort to the width a tracker actually has to learn, while the floor keeps even the deepest cells from being warmed with nothing. |
//! | [`geometric_rounds_custom`] | config | cites (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´) |
//! | [`geometric_rounds_root_equals_min`] | config | cites (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´) |
//! | [`geometric_rounds_very_small_decay`] | config | cites (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´) |
//! | [`geometric_rounds_rounding_half_values`] | config | Round counts are whole, and a fractional product is rounded to nearest with halves going away from zero rather than truncated toward it. Truncation would bias every depth downward and compound with the taper, so a cell an exact half-round short is warmed the extra round instead of losing it. |
//! | [`geometric_rounds_large_depth`] | config | At the deepest levels the G-tree can reach, the decayed product has long since collapsed toward zero, and the schedule still answers with its floor rather than with a degenerate number. The depth argument is bounded by the tree's own width, and the arithmetic stays well defined right up to that bound. |
//! | [`geometric_rounds_decay_one_is_constant`] | config | A decay of exactly one leaves the root untouched at every depth, giving a flat schedule that warms deep cells as heavily as shallow ones. This is what makes the upper endpoint worth admitting: uniform warm-up is expressible inside the geometric variant instead of needing a form of its own. |
//! | [`geometric_rounds_root_zero_min_zero`] | config | A geometric schedule with neither root nor floor asks for no rounds at any depth. The variant can therefore express a complete absence of warm-up without switching to the explicit form, and it does so with no special casing: the taper of nothing is nothing, and a floor of nothing lifts it nowhere. |
//! | [`explicit_rounds_for_depth`] | config | An explicit schedule is a direct lookup by depth, and depths past the end of the vector reuse its last entry rather than falling to zero or failing. The tail is the host's statement about all remaining depths, so a short vector still describes an unbounded tree. |
//! | [`explicit_rounds_single_entry`] | config | cites (´claim:config:an-explicit-schedule-is-read-by-depth-and-clamps-to-its-last-entry-beyond-its-length´) |
//! | [`explicit_rounds_non_monotonic`] | config | cites (´claim:config:an-explicit-schedule-is-read-by-depth-and-clamps-to-its-last-entry-beyond-its-length´) |
//! | [`explicit_empty_is_disabled`] | config | An explicit schedule that can never yield a round counts as noise switched off, and the two facts agree: the round count is zero at every depth and the schedule reports itself disabled. That agreement is what lets other rules key off the disabled flag instead of re-deriving it. |
//! | [`explicit_all_zeros_is_disabled`] | config | cites (´claim:config:an-explicit-schedule-that-can-never-yield-a-round-reports-itself-as-noise-switched-off´) |
//! | [`explicit_single_zero_is_disabled`] | config | cites (´claim:config:an-explicit-schedule-that-can-never-yield-a-round-reports-itself-as-noise-switched-off´) |
//! | [`explicit_mixed_zeros_not_disabled`] | config | A single non-zero entry anywhere keeps noise enabled, even where the shallow depths ask for none. A zero at a given depth is a statement about that depth alone, so a schedule may deliberately warm only the deeper cells and still counts as active. |
//! | [`geometric_is_disabled_when_root_and_min_are_zero`] | config | A geometric schedule is disabled only when both its root and its floor are zero, because either one alone still produces rounds. The floor lifts every depth to at least its own count, and the root sets the count at the shallow depths before the taper has descended. Reading only one of the two calls a schedule silent that is still asking for warm-up. |
//! | [`geometric_is_not_disabled_when_min_positive`] | config | cites (´claim:config:a-geometric-schedule-is-disabled-only-when-both-its-root-and-its-floor-are-zero´) |
//! | [`geometric_is_not_disabled_when_root_positive_and_min_zero`] | config | cites (´claim:config:a-geometric-schedule-is-disabled-only-when-both-its-root-and-its-floor-are-zero´) |
//! | [`max_rounds_geometric`] | config | The most a geometric schedule can ever ask for is its root, because the taper only descends from there. A host sizing buffers for warm-up can read the ceiling off the root alone, without evaluating the schedule at any depth. |
//! | [`max_rounds_geometric_floor_above_root`] | config | cites (´claim:config:the-ceiling-of-a-geometric-schedule-is-its-root-because-the-taper-only-descends´) |
//! | [`max_rounds_explicit`] | config | For an explicit schedule the ceiling is the largest entry it holds, not the first. Since the explicit form imposes no ordering, the depth-zero value carries no promise about the rest and the maximum has to be found rather than assumed. |
//! | [`max_rounds_explicit_empty`] | config | cites (´claim:config:the-ceiling-of-an-explicit-schedule-is-its-largest-entry-not-its-first´) |
//! | [`max_rounds_explicit_large`] | config | cites (´claim:config:the-ceiling-of-an-explicit-schedule-is-its-largest-entry-not-its-first´) |
//! | [`noise_schedule_default_matches_doc`] | config | The shipped schedule is the geometric one its documentation describes, and it reports itself active. Its numbers are calibrated rather than arbitrary: the root sits a little above the worst-case baseline convergence measured at the default forgetting factor, and the floor covers deep cells whose convergence scales down with analysis width without vanishing. |
//! | [`default_config_has_no_warnings`] | config | The shipped configuration draws no advisories, because the default schedule was calibrated against the default forgetting factor. Defaults that validated but warned would be an odd thing to ship, so the two sets of defaults are kept consistent with each other. |
//! | [`warns_when_noise_root_too_low_for_lambda_099`] | config | A configuration whose warm-up rounds fall short of what its memory needs is advised, not refused: it is arithmetically sound, but baselines may not converge before real observations arrive, so early scores would be unreliable. Advice and refusal are separate channels — validation would pass this configuration unchanged, and only the warning list carries the concern. |
//! | [`no_warning_when_noise_root_sufficient_for_lambda_095`] | config | How much warm-up is recommended falls with the forgetting factor, because a shorter memory converges sooner: a schedule too thin for a long-memory baseline is adequate for a shorter one. The same schedule draws advice or silence depending on the memory it is paired with, since the recommendation is a relation between the two rather than a property of either. |
//! | [`warns_when_small_batch_and_lambda_095`] | config | Batch size enters the recommendation as well: with few synthetic samples per round, each round buys less convergence, so the same shorter memory demands markedly more rounds and a schedule that was adequate becomes advised against. Warm-up is really measured in observations rather than in rounds, and the recommendation reflects that. |
//! | [`no_warning_for_explicit_schedule_with_enough_rounds`] | config | The advisory judges whatever the schedule actually yields at depth zero, whichever variant it is written in. An explicit schedule generous enough at the root passes the same check a geometric one would, so the recommendation is about warm-up delivered and not about how the host chose to express it. |
//! | [`warning_display_is_informative`] | config | A rendered advisory carries the numbers a host needs in order to act on it: the root it found, the root it recommends, and the forgetting factor that set that recommendation. Advice naming only the problem would leave the reader to re-derive the target. |
//! | [`rejects_coordinate_width_below_the_tracker_minimum`] | config | A coordinate width narrower than the smallest dimension a subspace tracker can model is refused at construction, with the same structured failure the configuration faults carry. The width is a parameter of the type rather than a field of the configuration, so validating the configuration alone can never see it, and the root tracker spans the whole width — at one dimension its lone basis vector spans the entire space, novelty is identically zero, and the tracker reports a settled model of everything while modelling nothing. Refusing is what lets the constructor's success mean the sentinel it returns can measure. |
//! | [`accepts_the_narrowest_modellable_coordinate_width`] | config | cites (´claim:config:a-coordinate-width-below-the-tracker-minimum-is-refused-at-construction´) |
//! | [`collects_a_width_fault_alongside_a_configuration_fault`] | config | cites (´claim:config:a-coordinate-width-below-the-tracker-minimum-is-refused-at-construction´) |

use crate::config::*;

// ── Default config ──────────────────────────────────────────

/// The parameters the sentinel ships with satisfy every rule it checks them
/// against. A host that configures nothing at all therefore starts from a
/// coherent measurement setup rather than from a template it must first
/// repair.
///
/// ´claim:config:the-shipped-defaults-satisfy-every-rule-validation-checks´
/// ´test:crate:default-config-is-valid´
#[test]
fn default_config_is_valid() {
    SentinelConfig::<u64>::default().validate().unwrap();
}

/// Each default holds the value its documentation promises — the long-memory
/// forgetting factor, the modest rank ceiling, the analysis and graph budgets,
/// the noise batch and its fixed seed. The defaults are a calibrated set rather
/// than arbitrary placeholders, so documenting them and shipping them are held
/// to be the same act.
///
/// ´claim:config:every-default-field-holds-the-value-its-documentation-promises´
/// ´test:crate:default-config-field-values´
#[test]
fn default_config_field_values() {
    let cfg = SentinelConfig::<u64>::default();
    cfg.validate().unwrap();

    // Core subspace.
    assert_eq!(cfg.max_rank, 16);
    assert!((cfg.forgetting_factor - 0.99).abs() < f64::EPSILON);
    assert_eq!(cfg.rank_update_interval, 100);
    assert!((cfg.energy_threshold - 0.90).abs() < f64::EPSILON);
    assert!((cfg.eps - 1e-6).abs() < f64::EPSILON);

    // CUSUM / EWMA.
    assert!((cfg.cusum_slow_decay - 0.999).abs() < f64::EPSILON);
    assert!((cfg.cusum_coord_slow_decay - 0.999).abs() < f64::EPSILON);
    assert!((cfg.cusum_allowance_sigmas - 0.5).abs() < f64::EPSILON);
    assert!((cfg.clip_sigmas - 3.0).abs() < f64::EPSILON);
    assert!((cfg.clip_pressure_decay - 0.95).abs() < f64::EPSILON);
    assert!(!cfg.per_sample_scores);

    // Analysis.
    assert_eq!(cfg.analysis_k, 1024);
    assert_eq!(cfg.analysis_depth_cutoff, 6);

    // G-V Graph.
    assert_eq!(cfg.split_threshold, 100);
    assert_eq!(cfg.d_create, 3);
    assert_eq!(cfg.d_evict, 6);
    assert_eq!(cfg.budget, 100_000);

    // Noise injection.
    assert_eq!(cfg.noise_batch_size, 16);
    assert_eq!(cfg.noise_seed, Some(42));
    assert!(!cfg.background_warming);
}

// ── Per-field validation: core subspace fields ──────────────

/// A capacity ceiling of zero leaves the sentinel nothing to work with: a
/// subspace tracker allowed no basis vectors can model nothing at all, so the
/// value is refused rather than quietly read as a request for a disabled
/// tracker. The refusal names that field and faults nothing else in an
/// otherwise default configuration.
///
/// ´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´
/// ´test:crate:rejects-max-rank-zero´
#[test]
fn rejects_max_rank_zero() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 0,
        ..SentinelConfig::<u64>::default()
    };
    assert_eq!(cfg.validate(), Err(ConfigErrors(vec![ConfigError::MaxRankZero])));
}

/// The forgetting factor must lie strictly inside the unit interval: at one the
/// baseline never forgets and can never adapt, at zero it retains nothing, and
/// outside the interval the exponential weighting stops being a weighting. Both
/// endpoints are refused along with values beyond them, and the refusal carries
/// back the value it saw so the host can tell which of its inputs was
/// faulted.
///
/// ´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´
/// ´test:crate:rejects-forgetting-factor-out-of-range´
#[test]
fn rejects_forgetting_factor_out_of_range() {
    for &bad in &[0.0, 1.0, -0.1, 1.5] {
        let cfg = SentinelConfig::<u64> {
            forgetting_factor: bad,
            // Keep slow decays above forgetting to avoid cascading errors.
            cusum_slow_decay: 0.999,
            cusum_coord_slow_decay: 0.999,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::ForgettingFactorOutOfRange(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected ForgettingFactorOutOfRange for {bad}"
        );
    }
}

/// The interval is open rather than narrowed by a margin of safety: a factor a
/// hair below one and one a hair above zero are both admitted, provided the
/// CUSUM references above them stay slower still. The line is drawn at the
/// endpoints themselves and nowhere short of them.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:accepts-forgetting-factor-near-boundaries´
#[test]
fn accepts_forgetting_factor_near_boundaries() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.9999,
        cusum_slow_decay: 0.99999,
        cusum_coord_slow_decay: 0.99999,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();

    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.0001,
        cusum_slow_decay: 0.001,
        cusum_coord_slow_decay: 0.001,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// The same floor governs the cadence at which a tracker reassesses its rank:
/// an interval of zero asks for reassessment at no interval at all, so it is
/// refused rather than treated as reassessing constantly.
///
/// (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´)
/// ´test:crate:rejects-rank-update-interval-zero´
#[test]
fn rejects_rank_update_interval_zero() {
    let cfg = SentinelConfig::<u64> {
        rank_update_interval: 0,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.contains(&ConfigError::RankUpdateIntervalZero));
}

/// The energy threshold obeys the same open interval, and for a kindred reason
/// at each end: retaining no fraction of the variance describes nothing, and
/// demanding the whole of it asks for a rank the data cannot justify.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:rejects-energy-threshold-out-of-range´
#[test]
fn rejects_energy_threshold_out_of_range() {
    for &bad in &[0.0, 1.0, -0.5, 1.1] {
        let cfg = SentinelConfig::<u64> {
            energy_threshold: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::EnergyThresholdOutOfRange(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected EnergyThresholdOutOfRange for {bad}"
        );
    }
}

/// The stability constant exists to keep denominators away from zero, so a
/// value at or below zero defeats the only thing it is for. Both are refused,
/// and the refusal reports the offending value rather than silently
/// substituting a workable one.
///
/// ´claim:config:a-stability-constant-must-be-strictly-positive-because-a-zero-guard-guards-nothing´
/// ´test:crate:rejects-eps-not-positive´
#[test]
fn rejects_eps_not_positive() {
    for &bad in &[0.0, -1e-6] {
        let cfg = SentinelConfig::<u64> {
            eps: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::EpsNotPositive(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected EpsNotPositive for {bad}"
        );
    }
}

// ── Per-field validation: CUSUM / EWMA fields ───────────────

/// The slow CUSUM decay is a rate like any other and is held to the same open
/// interval. Its endpoints are refused before the separate question of whether
/// it is slower than the fast baseline is even reached.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:rejects-cusum-slow-decay-out-of-range´
#[test]
fn rejects_cusum_slow_decay_out_of_range() {
    for &bad in &[0.0, 1.0, -0.1, 1.5] {
        let cfg = SentinelConfig::<u64> {
            cusum_slow_decay: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::CusumSlowDecayOutOfRange(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected CusumSlowDecayOutOfRange for {bad}"
        );
    }
}

/// The CUSUM reference must have longer memory than the baseline it is measured
/// against — strictly slower, not merely as slow. A reference adapting as fast
/// as the baseline would follow a gradual drift instead of exposing it, and
/// exposing exactly that drift is what the accumulator exists for.
///
/// ´claim:config:the-cusum-reference-must-decay-strictly-slower-than-the-baseline-it-judges´
/// ´test:crate:rejects-cusum-slow-decay-below-forgetting´
#[test]
fn rejects_cusum_slow_decay_below_forgetting() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.99,
        cusum_slow_decay: 0.98,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.iter().any(|e| matches!(e, ConfigError::CusumSlowDecayTooLow { .. })));
}

/// Any strictly greater value satisfies the ordering; the margin need not be
/// generous. Only equality and below are refused, so the rule is about which
/// memory is the longer one and not about how much longer it is.
///
/// (´claim:config:the-cusum-reference-must-decay-strictly-slower-than-the-baseline-it-judges´)
/// ´test:crate:accepts-cusum-slow-decay-just-above-forgetting´
#[test]
fn accepts_cusum_slow_decay_just_above_forgetting() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.95,
        cusum_slow_decay: 0.951,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// The coordination tier's slow decay is checked against the same open interval
/// as the per-tracker one. Splitting the two fields lets the host set different
/// drift sensitivities per tier without loosening what counts as a rate.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:rejects-cusum-coord-slow-decay-out-of-range´
#[test]
fn rejects_cusum_coord_slow_decay_out_of_range() {
    for &bad in &[0.0, 1.0, -0.1, 1.5] {
        let cfg = SentinelConfig::<u64> {
            cusum_coord_slow_decay: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::CusumCoordSlowDecayOutOfRange(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected CusumCoordSlowDecayOutOfRange for {bad}"
        );
    }
}

/// The ordering binds the coordination tier too: its reference is measured
/// against the same fast baseline and must likewise be slower than it. A second
/// tier buys independent sensitivity, not an exemption.
///
/// (´claim:config:the-cusum-reference-must-decay-strictly-slower-than-the-baseline-it-judges´)
/// ´test:crate:rejects-cusum-coord-slow-decay-below-forgetting´
#[test]
fn rejects_cusum_coord_slow_decay_below_forgetting() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.99,
        cusum_coord_slow_decay: 0.98,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(
        err.0
            .iter()
            .any(|e| matches!(e, ConfigError::CusumCoordSlowDecayTooLow { .. }))
    );
}

/// The CUSUM allowance is subtracted from each step's gap before anything
/// accumulates, so a negative allowance would be added to every gap and would
/// manufacture evidence of drift out of ordinary noise. Negative values are
/// therefore refused.
///
/// ´claim:config:a-noise-allowance-may-never-be-negative-because-it-would-manufacture-the-drift-it-absorbs´
/// ´test:crate:rejects-cusum-allowance-sigmas-negative´
#[test]
fn rejects_cusum_allowance_sigmas_negative() {
    let cfg = SentinelConfig::<u64> {
        cusum_allowance_sigmas: -0.1,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.iter().any(|e| matches!(e, ConfigError::CusumAllowanceNegative(_))));
}

/// Zero sits on the permitted side of that line and means something definite:
/// no tolerance at all, so any positive gap accumulates. Declining to absorb
/// noise is a legitimate choice; inventing signal is not.
///
/// (´claim:config:a-noise-allowance-may-never-be-negative-because-it-would-manufacture-the-drift-it-absorbs´)
/// ´test:crate:accepts-cusum-allowance-sigmas-zero´
#[test]
fn accepts_cusum_allowance_sigmas_zero() {
    let cfg = SentinelConfig::<u64> {
        cusum_allowance_sigmas: 0.0,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// The clip width bounds how far above the mean an observation may sit and
/// still update the baseline. At zero or below nothing would ever clear the
/// bar, so the baseline would starve rather than be protected — the value is
/// refused instead of being allowed to silence the very updates it guards.
///
/// ´claim:config:a-clip-width-must-be-strictly-positive-because-a-width-of-zero-would-reject-every-observation´
/// ´test:crate:rejects-clip-sigmas-not-positive´
#[test]
fn rejects_clip_sigmas_not_positive() {
    for &bad in &[0.0, -1.0] {
        let cfg = SentinelConfig::<u64> {
            clip_sigmas: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0.iter().any(|e| matches!(e, ConfigError::ClipSigmasNotPositive(_))),
            "expected ClipSigmasNotPositive for {bad}"
        );
    }
}

/// The clip-pressure estimate adapts by the same kind of exponential rate and
/// is bound by the same open interval, so contamination tracking cannot be
/// configured either to never adapt or to never remember.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:rejects-clip-pressure-decay-out-of-range´
#[test]
fn rejects_clip_pressure_decay_out_of_range() {
    for &bad in &[0.0, 1.0, -0.1, 1.5] {
        let cfg = SentinelConfig::<u64> {
            clip_pressure_decay: bad,
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::ClipPressureDecayOutOfRange(v) if (*v - bad).abs() < f64::EPSILON)),
            "expected ClipPressureDecayOutOfRange for {bad}"
        );
    }
}

/// An ordinary interior value passes without comment. The rule refuses only the
/// endpoints and what lies beyond them, so a rate chosen for a memory of a few
/// dozen batches is simply admitted.
///
/// (´claim:config:a-rate-must-lie-strictly-inside-the-unit-interval-and-the-refusal-names-the-value-it-saw´)
/// ´test:crate:accepts-clip-pressure-decay-valid´
#[test]
fn accepts_clip_pressure_decay_valid() {
    let cfg = SentinelConfig::<u64> {
        clip_pressure_decay: 0.95,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

// ── Per-field validation: analysis fields ───────────────────

/// The analysis ceiling is a capacity of the same kind: with no competitive
/// cells there is nothing to compete for and no tier to run, so zero is refused
/// rather than read as a request to switch analysis off. Here too the single
/// bad field yields its own error and no cascade.
///
/// (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´)
/// ´test:crate:rejects-analysis-k-zero´
#[test]
fn rejects_analysis_k_zero() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 0,
        ..SentinelConfig::<u64>::default()
    };
    assert_eq!(cfg.validate(), Err(ConfigErrors(vec![ConfigError::AnalysisKZero])));
}

/// The floor really is one rather than some larger practical minimum. A single
/// competitive cell is a coherent configuration, so the bound says that some
/// analysis must happen, not how much.
///
/// (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´)
/// ´test:crate:accepts-analysis-k-one´
#[test]
fn accepts_analysis_k_one() {
    let cfg = SentinelConfig::<u64> {
        analysis_k: 1,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// A depth cutoff of zero is admitted rather than refused, and it means
/// something usable: only the V-Tree root remains eligible, which effectively
/// turns adaptive selection off. Zero is a fault in a capacity but a legitimate
/// setting for a depth gate, because a gate that admits nothing deep expresses
/// a policy rather than an incoherence.
///
/// ´claim:config:a-depth-cutoff-of-zero-is-a-legal-way-to-disable-adaptive-selection-not-an-error´
/// ´test:crate:accepts-analysis-depth-cutoff-zero´
#[test]
fn accepts_analysis_depth_cutoff_zero() {
    let cfg = SentinelConfig::<u64> {
        analysis_depth_cutoff: 0,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

// ── Per-field validation: G-V Graph fields ──────────────────

/// A split threshold of zero would let a cell subdivide before accumulating any
/// intensity at all, so the graph would fragment on its first observation
/// instead of on evidence of sustained traffic. The threshold must be strictly
/// positive.
///
/// ´claim:config:a-split-threshold-must-be-positive-so-a-cell-subdivides-on-traffic-rather-than-immediately´
/// ´test:crate:rejects-split-threshold-zero´
#[test]
fn rejects_split_threshold_zero() {
    let cfg = SentinelConfig::<u64> {
        split_threshold: 0,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());
}

/// The creation depth is held to the same floor: a graph permitted to create at
/// no depth could never grow past its root, so zero is refused.
///
/// (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´)
/// ´test:crate:rejects-d-create-zero´
#[test]
fn rejects_d_create_zero() {
    let cfg = SentinelConfig::<u64> {
        d_create: 0,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());
}

/// Eviction depth must sit strictly deeper than creation depth, and an equal
/// pair is refused just as an inverted one is. The gap between them is the
/// buffer zone — entries too deep to create children but not yet deep enough to
/// be evicted — so collapsing it would leave a cell eligible for eviction the
/// moment it stopped being eligible to grow.
///
/// ´claim:config:the-eviction-depth-must-sit-strictly-deeper-than-the-creation-depth-so-a-buffer-zone-exists´
/// ´test:crate:rejects-d-evict-not-greater-than-d-create´
#[test]
fn rejects_d_evict_not_greater_than_d_create() {
    // Equal.
    let cfg = SentinelConfig::<u64> {
        d_create: 3,
        d_evict: 3,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(
        err.0
            .contains(&ConfigError::DEvictNotGreaterThanDCreate { d_create: 3, d_evict: 3 })
    );

    // Less.
    let cfg = SentinelConfig::<u64> {
        d_create: 5,
        d_evict: 3,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(
        err.0
            .contains(&ConfigError::DEvictNotGreaterThanDCreate { d_create: 5, d_evict: 3 })
    );
}

/// The narrowest possible buffer, a single level, satisfies the ordering. What
/// such a configuration still owes is headroom: a narrow buffer lowers the node
/// count the budget must exceed, and here the budget is set just above that
/// lowered requirement.
///
/// (´claim:config:the-eviction-depth-must-sit-strictly-deeper-than-the-creation-depth-so-a-buffer-zone-exists´)
/// ´test:crate:accepts-d-evict-one-above-d-create´
#[test]
fn accepts_d_evict_one_above_d_create() {
    // d_evict = d_create + 1 is valid, but budget must satisfy headroom.
    // buffer = 1, headroom = 3^2 = 9.
    let cfg = SentinelConfig::<u64> {
        d_create: 3,
        d_evict: 4,
        budget: 10, // > 9
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// A node budget of zero admits no live nodes at all. The sentinel always
/// operates in budgeted mode, so its ceiling cannot be read as the absence of
/// one.
///
/// (´claim:config:a-capacity-of-zero-is-refused-because-it-leaves-the-sentinel-nothing-to-work-with´)
/// ´test:crate:rejects-budget-zero´
#[test]
fn rejects_budget_zero() {
    let cfg = SentinelConfig::<u64> {
        budget: 0,
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());
}

/// The node budget must exceed what the depth gates themselves imply: a buffer
/// zone of a given width can hold a number of nodes growing as a power of
/// three, and a budget merely equal to that leaves the graph no room to
/// manoeuvre inside its own gates. Equality is refused, not merely
/// shortfall.
///
/// ´claim:config:the-node-budget-must-exceed-the-headroom-the-depth-gates-imply-and-equalling-it-is-not-enough´
/// ´test:crate:rejects-budget-below-headroom´
#[test]
fn rejects_budget_below_headroom() {
    // With d_create=3, d_evict=6, buffer=3, headroom=3^4=81.
    // Budget must be > 81.
    let cfg = SentinelConfig::<u64> {
        d_create: 3,
        d_evict: 6,
        budget: 81,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.iter().any(|e| matches!(e, ConfigError::BudgetTooSmall { .. })));
}

/// A single node above the requirement is enough, which pins the comparison as
/// strict rather than as a demand for margin. The headroom rule states the
/// minimum the gates force, and the host is free to sit immediately above
/// it.
///
/// (´claim:config:the-node-budget-must-exceed-the-headroom-the-depth-gates-imply-and-equalling-it-is-not-enough´)
/// ´test:crate:accepts-budget-just-above-headroom´
#[test]
fn accepts_budget_just_above_headroom() {
    let cfg = SentinelConfig::<u64> {
        d_create: 3,
        d_evict: 6,
        budget: 82,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// Past a certain width the headroom the depth gates imply stops being a
/// number the machine can hold, and the depth pair is refused on its own terms
/// rather than measured against a figure that wrapped. The requirement grows as
/// a power of three, so a buffer in the forties already exceeds the addressable
/// range; computing it and comparing anyway would either abort the validation
/// that promised to return its faults, or silently compare the budget against a
/// small wrapped remainder and admit a configuration that cannot hold. The
/// refusal names the two depths, since they are what the host must change.
///
/// ´claim:config:a-depth-buffer-whose-headroom-cannot-be-represented-is-refused-on-its-own-terms´
/// ´test:crate:rejects-depth-buffer-whose-headroom-cannot-be-represented´
#[test]
fn rejects_depth_buffer_whose_headroom_cannot_be_represented() {
    let cfg = SentinelConfig::<u64> {
        d_create: 1,
        d_evict: 41,
        budget: 1_000_000,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.contains(&ConfigError::DepthBufferTooLarge {
        d_create: 1,
        d_evict: 41
    }));
    assert!(
        !err.0.iter().any(|e| matches!(e, ConfigError::BudgetTooSmall { .. })),
        "an unrepresentable requirement is not reported as a budget shortfall"
    );
}

/// The widest buffer whose headroom still fits is validated the ordinary way,
/// which fixes the boundary between the two refusals rather than leaving it to
/// be inferred. At this width the requirement is a real number, so a budget
/// below it comes back as a shortfall naming the figure it fell short of.
///
/// (´claim:config:a-depth-buffer-whose-headroom-cannot-be-represented-is-refused-on-its-own-terms´)
/// ´test:crate:reports-a-shortfall-at-the-widest-representable-depth-buffer´
#[test]
fn reports_a_shortfall_at_the_widest_representable_depth_buffer() {
    let cfg = SentinelConfig::<u64> {
        d_create: 1,
        d_evict: 40,
        budget: 1_000_000,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.iter().any(|e| matches!(e, ConfigError::BudgetTooSmall { .. })));
    assert!(!err.0.iter().any(|e| matches!(e, ConfigError::DepthBufferTooLarge { .. })));
}

// ── Per-field validation: non-numbers and infinities ────────

/// Every floating-point field refuses a non-number, because the ordered
/// comparisons that police the other values cannot see one. A comparison
/// against a non-number is false whichever way it is written, so a bound
/// expressed as a pair of comparisons admits it silently — and the value then
/// spreads, since every product and sum it enters returns a non-number too. A
/// forgetting factor admitted this way reaches the baseline arithmetic and
/// leaves every score afterwards unusable, with nothing in the report to say
/// which field was responsible. The guard therefore sits ahead of the bound
/// rather than inside it.
///
/// ´claim:config:a-non-number-is-refused-in-every-floating-point-field-because-an-ordered-bound-cannot-see-one´
/// ´test:crate:rejects-nan-in-every-floating-point-field´
#[test]
fn rejects_nan_in_every_floating_point_field() {
    let nan = f64::NAN;
    let cases: [(&str, SentinelConfig<u64>); 8] = [
        (
            "forgetting_factor",
            SentinelConfig {
                forgetting_factor: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "energy_threshold",
            SentinelConfig {
                energy_threshold: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "eps",
            SentinelConfig {
                eps: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "cusum_slow_decay",
            SentinelConfig {
                cusum_slow_decay: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "cusum_coord_slow_decay",
            SentinelConfig {
                cusum_coord_slow_decay: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "cusum_allowance_sigmas",
            SentinelConfig {
                cusum_allowance_sigmas: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "clip_sigmas",
            SentinelConfig {
                clip_sigmas: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
        (
            "clip_pressure_decay",
            SentinelConfig {
                clip_pressure_decay: nan,
                ..SentinelConfig::<u64>::default()
            },
        ),
    ];

    for (field, cfg) in cases {
        assert!(cfg.validate().is_err(), "a non-number in {field} must be refused");
    }
}

/// An infinite value is admitted where the interval is one-sided, because
/// there it names a real limit rather than the absence of one. An infinite
/// clip width is the unclipped configuration — the control arm the package's
/// own clipping study runs against — and it compares correctly against every
/// bound it is checked with, which is precisely what a non-number does not do.
/// The fields whose intervals are two-sided still refuse it, and they refuse it
/// through the bound they already carry rather than through a separate guard.
///
/// ´claim:config:an-infinite-value-is-admitted-where-the-interval-is-one-sided-because-there-it-names-a-real-limit´
/// ´test:crate:accepts-infinite-clip-width-and-refuses-infinite-rates´
#[test]
fn accepts_infinite_clip_width_and_refuses_infinite_rates() {
    let unclipped = SentinelConfig::<u64> {
        clip_sigmas: f64::INFINITY,
        ..SentinelConfig::<u64>::default()
    };
    unclipped.validate().unwrap();

    let infinite_rate = SentinelConfig::<u64> {
        forgetting_factor: f64::INFINITY,
        ..SentinelConfig::<u64>::default()
    };
    let err = infinite_rate.validate().unwrap_err();
    assert!(err.0.iter().any(|e| matches!(e, ConfigError::ForgettingFactorOutOfRange(_))));
}

// ── Per-field validation: noise injection fields ────────────

/// A noise batch of no samples is a fault only when the schedule actually asks
/// for rounds: with an active schedule the warm-up would run rounds that feed
/// the tracker nothing. The check is conditional on the schedule rather than
/// absolute, because zero samples per round is coherent when there are no
/// rounds to run.
///
/// ´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´
/// ´test:crate:rejects-noise-batch-size-zero-when-enabled´
#[test]
fn rejects_noise_batch_size_zero_when_enabled() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(vec![5]),
        noise_batch_size: 0,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.contains(&ConfigError::NoiseBatchSizeZero));
}

/// With noise switched off entirely, a batch size of zero passes. The field
/// describes work that will never be requested, so validation has nothing to
/// object to.
///
/// (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´)
/// ´test:crate:accepts-noise-batch-size-zero-when-disabled´
#[test]
fn accepts_noise_batch_size_zero_when_disabled() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Explicit(vec![]),
        noise_batch_size: 0,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// The geometric form reaches the same check through its own reading of what
/// silence is. A positive root with a zero floor still asks for rounds at the
/// shallow depths, so a batch of no samples is faulted there exactly as it is
/// for an explicit schedule. This is the pairing that a floor-only reading of
/// disablement lets through, and letting it through is not a missing warning
/// but a broken tracker: the warm-up runs its rounds, divides by a batch of
/// nothing, and leaves the latent baseline unable to score anything for the
/// rest of the sentinel's life.
///
/// (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´)
/// ´test:crate:rejects-noise-batch-size-zero-with-geometric-root-and-zero-floor´
#[test]
fn rejects_noise_batch_size_zero_with_geometric_root_and_zero_floor() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Geometric {
            root: 450,
            decay: 0.5,
            min: 0,
        },
        noise_batch_size: 0,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.contains(&ConfigError::NoiseBatchSizeZero));
}

/// The geometric form's own accepting side: with root and floor both zero the
/// schedule asks for nothing at any depth, so a batch size of zero describes
/// work that will never be requested and passes for the same reason the empty
/// explicit schedule does.
///
/// (´claim:config:a-noise-batch-of-zero-is-faulted-only-when-the-schedule-actually-asks-for-rounds´)
/// ´test:crate:accepts-noise-batch-size-zero-with-geometric-root-and-floor-zero´
#[test]
fn accepts_noise_batch_size_zero_with_geometric_root_and_floor_zero() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Geometric {
            root: 0,
            decay: 0.5,
            min: 0,
        },
        noise_batch_size: 0,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// An absent noise seed is valid and simply changes where the randomness comes
/// from: with a seed the warm-up is reproducible across restarts, without one
/// it is drawn from system entropy. Determinism is offered rather than
/// required, so neither choice counts as a misconfiguration.
///
/// ´claim:config:an-absent-noise-seed-is-valid-and-trades-reproducibility-for-system-entropy´
/// ´test:crate:accepts-noise-seed-none´
#[test]
fn accepts_noise_seed_none() {
    let cfg = SentinelConfig::<u64> {
        noise_seed: None,
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().unwrap();
}

/// The geometric decay is bound to a half-open interval rather than the open
/// one other rates get: values at or below zero and above one are refused, but
/// one itself is not. Zero would collapse the schedule onto its floor
/// immediately, whereas a schedule that never tapers with depth is a
/// legitimate thing to ask for.
///
/// ´claim:config:the-noise-decay-is-bound-above-zero-and-at-most-one-because-a-schedule-that-never-tapers-is-legitimate´
/// ´test:crate:rejects-geometric-decay-out-of-range´
#[test]
fn rejects_geometric_decay_out_of_range() {
    for &bad in &[0.0, -0.1, 1.5] {
        let cfg = SentinelConfig::<u64> {
            noise_schedule: NoiseSchedule::Geometric {
                root: 50,
                decay: bad,
                min: 10,
            },
            ..SentinelConfig::<u64>::default()
        };
        let err = cfg.validate().unwrap_err();
        assert!(
            err.0
                .iter()
                .any(|e| matches!(e, ConfigError::NoiseScheduleDecayOutOfRange(_))),
            "expected NoiseScheduleDecayOutOfRange for decay={bad}"
        );
    }
}

/// A decay that is not a number is refused explicitly, because every comparison
/// against it is false and a range check alone would let it through. The
/// configuration is rejected before such a value could reach the exponentiation
/// and turn every round count into nonsense.
///
/// ´claim:config:a-decay-that-is-not-a-number-is-refused-explicitly-because-range-comparisons-alone-would-admit-it´
/// ´test:crate:rejects-geometric-decay-nan´
#[test]
fn rejects_geometric_decay_nan() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Geometric {
            root: 50,
            decay: f64::NAN,
            min: 10,
        },
        ..SentinelConfig::<u64>::default()
    };
    assert!(cfg.validate().is_err());
}

/// A geometric schedule that starts at no rounds but floors at a positive count
/// contradicts itself: the taper only ever descends from the root, so every
/// depth would be lifted to the floor and the root would describe nothing.
/// That combination is refused — which is why the root is faulted only when
/// the floor is positive.
///
/// ´claim:config:a-geometric-root-of-zero-under-a-positive-floor-is-refused-as-self-contradictory´
/// ´test:crate:rejects-geometric-root-zero-with-positive-min´
#[test]
fn rejects_geometric_root_zero_with_positive_min() {
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::Geometric {
            root: 0,
            decay: 0.5,
            min: 10,
        },
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.contains(&ConfigError::NoiseScheduleRootZero));
}

// ── Error accumulation ──────────────────────────────────────

/// Validation reports every violation it finds rather than stopping at the
/// first, so a host with several bad fields learns of all of them in one pass
/// instead of discovering them one restart at a time. Faulting several fields
/// at once yields at least as many errors, each naming its own field.
///
/// ´claim:config:validation-reports-every-violation-it-finds-rather-than-stopping-at-the-first´
/// ´test:crate:collects-multiple-errors´
#[test]
fn collects_multiple_errors() {
    let cfg = SentinelConfig::<u64> {
        max_rank: 0,
        analysis_k: 0,
        budget: 0,
        ..SentinelConfig::<u64>::default()
    };
    let err = cfg.validate().unwrap_err();
    assert!(err.0.len() >= 3, "expected at least 3 errors, got {}", err.0.len());
    assert!(err.0.contains(&ConfigError::MaxRankZero));
    assert!(err.0.contains(&ConfigError::AnalysisKZero));
    assert!(err.0.contains(&ConfigError::BudgetZero));
}

// ── NoiseSchedule::rounds_for_depth ─────────────────────────

/// A geometric schedule scales its round count by the decay at each level of
/// depth and then rests on its floor. Deeper cells are narrower and converge in
/// fewer rounds, so tapering matches warm-up effort to the width a tracker
/// actually has to learn, while the floor keeps even the deepest cells from
/// being warmed with nothing.
///
/// ´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´
/// ´test:crate:geometric-rounds-default´
#[test]
fn geometric_rounds_default() {
    let schedule = NoiseSchedule::default();
    // Default: Geometric { root: 450, decay: 0.5, min: 50 }
    assert_eq!(schedule.rounds_for_depth(0), 450);
    assert_eq!(schedule.rounds_for_depth(1), 225);
    assert_eq!(schedule.rounds_for_depth(2), 113); // 450 × 0.25 = 112.5 → 113
    assert_eq!(schedule.rounds_for_depth(3), 56); // 450 × 0.125 = 56.25 → 56
    assert_eq!(schedule.rounds_for_depth(4), 50); // 450 × 0.0625 = 28.125 → 28, but min=50
    assert_eq!(schedule.rounds_for_depth(10), 50); // floored to min
    assert_eq!(schedule.rounds_for_depth(128), 50); // max depth → still min
}

/// The shape is not tied to the shipped numbers: another root, decay and floor
/// taper the same way and settle onto the same kind of plateau. Those three are
/// the schedule's only degrees of freedom, and nothing else is baked in.
///
/// (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´)
/// ´test:crate:geometric-rounds-custom´
#[test]
fn geometric_rounds_custom() {
    let schedule = NoiseSchedule::geometric(400, 0.5, 100);
    assert_eq!(schedule.rounds_for_depth(0), 400);
    assert_eq!(schedule.rounds_for_depth(1), 200);
    assert_eq!(schedule.rounds_for_depth(2), 100);
    assert_eq!(schedule.rounds_for_depth(3), 100); // floored to min
    assert_eq!(schedule.rounds_for_depth(20), 100);
}

/// When root and floor coincide the taper is invisible: every depth returns the
/// same count, because the descent has nowhere to descend to. A constant
/// schedule is expressible without any special case.
///
/// (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´)
/// ´test:crate:geometric-rounds-root-equals-min´
#[test]
fn geometric_rounds_root_equals_min() {
    // When root == min, every depth yields the same value.
    let schedule = NoiseSchedule::geometric(100, 0.5, 100);
    for depth in 0..20 {
        assert_eq!(
            schedule.rounds_for_depth(depth),
            100,
            "depth {depth} should yield root=min=100"
        );
    }
}

/// A steep decay reaches the floor within a level or two, and past that the
/// floor alone determines the answer. It is the floor and not the decay that
/// bounds a deep cell's warm-up, however aggressive the taper.
///
/// (´claim:config:a-geometric-schedule-tapers-with-depth-and-then-rests-on-its-floor´)
/// ´test:crate:geometric-rounds-very-small-decay´
#[test]
fn geometric_rounds_very_small_decay() {
    // decay=0.01 → root × 0.01^depth, hits min almost immediately.
    let schedule = NoiseSchedule::geometric(1000, 0.01, 5);
    assert_eq!(schedule.rounds_for_depth(0), 1000);
    assert_eq!(schedule.rounds_for_depth(1), 10); // 1000 × 0.01 = 10
    assert_eq!(schedule.rounds_for_depth(2), 5); // 1000 × 0.0001 → 0, but min=5
    assert_eq!(schedule.rounds_for_depth(10), 5);
}

/// Round counts are whole, and a fractional product is rounded to nearest with
/// halves going away from zero rather than truncated toward it. Truncation
/// would bias every depth downward and compound with the taper, so a cell an
/// exact half-round short is warmed the extra round instead of losing it.
///
/// ´claim:config:a-fractional-round-count-is-rounded-to-nearest-with-halves-away-from-zero-not-truncated´
/// ´test:crate:geometric-rounds-rounding-half-values´
#[test]
fn geometric_rounds_rounding_half_values() {
    // f64::round() rounds half away from zero.
    // root=100, decay=0.5 → 100, 50, 25, 12.5→13, 6.25→6, 3.125→3, 1.5625→2, 0.78→1
    let schedule = NoiseSchedule::geometric(100, 0.5, 1);
    assert_eq!(schedule.rounds_for_depth(0), 100);
    assert_eq!(schedule.rounds_for_depth(1), 50);
    assert_eq!(schedule.rounds_for_depth(2), 25);
    assert_eq!(schedule.rounds_for_depth(3), 13); // 12.5 rounds to 13
    assert_eq!(schedule.rounds_for_depth(4), 6); // 6.25 rounds to 6
    assert_eq!(schedule.rounds_for_depth(5), 3); // 3.125 rounds to 3
    assert_eq!(schedule.rounds_for_depth(6), 2); // 1.5625 rounds to 2
    assert_eq!(schedule.rounds_for_depth(7), 1); // 0.78125 rounds to 1 = min
}

/// At the deepest levels the G-tree can reach, the decayed product has long
/// since collapsed toward zero, and the schedule still answers with its floor
/// rather than with a degenerate number. The depth argument is bounded by the
/// tree's own width, and the arithmetic stays well defined right up to that
/// bound.
///
/// ´claim:config:the-schedule-stays-well-defined-at-the-deepest-depth-the-g-tree-can-reach´
/// ´test:crate:geometric-rounds-large-depth´
#[test]
fn geometric_rounds_large_depth() {
    // G-tree depth is bounded ≤ 128. Verify numeric stability near that limit.
    let schedule = NoiseSchedule::geometric(400, 0.5, 50);
    assert_eq!(schedule.rounds_for_depth(126), 50);
    assert_eq!(schedule.rounds_for_depth(127), 50);
    assert_eq!(schedule.rounds_for_depth(128), 50);
}

/// A decay of exactly one leaves the root untouched at every depth, giving a
/// flat schedule that warms deep cells as heavily as shallow ones. This is what
/// makes the upper endpoint worth admitting: uniform warm-up is expressible
/// inside the geometric variant instead of needing a form of its own.
///
/// ´claim:config:a-decay-of-one-yields-a-flat-schedule-that-warms-every-depth-alike´
/// ´test:crate:geometric-rounds-decay-one-is-constant´
#[test]
fn geometric_rounds_decay_one_is_constant() {
    // decay=1.0 → root × 1.0^depth = root at every depth.
    let schedule = NoiseSchedule::geometric(42, 1.0, 1);
    for depth in [0, 1, 10, 50, 128] {
        assert_eq!(schedule.rounds_for_depth(depth), 42);
    }
}

/// A geometric schedule with neither root nor floor asks for no rounds at any
/// depth. The variant can therefore express a complete absence of warm-up
/// without switching to the explicit form, and it does so with no special
/// casing: the taper of nothing is nothing, and a floor of nothing lifts it
/// nowhere.
///
/// ´claim:config:a-geometric-schedule-with-no-root-and-no-floor-asks-for-no-rounds-at-any-depth´
/// ´test:crate:geometric-rounds-root-zero-min-zero´
#[test]
fn geometric_rounds_root_zero_min_zero() {
    // Both root and min zero → always 0 rounds.
    let schedule = NoiseSchedule::Geometric {
        root: 0,
        decay: 0.5,
        min: 0,
    };
    assert_eq!(schedule.rounds_for_depth(0), 0);
    assert_eq!(schedule.rounds_for_depth(5), 0);
    assert_eq!(schedule.rounds_for_depth(128), 0);
}

/// An explicit schedule is a direct lookup by depth, and depths past the end of
/// the vector reuse its last entry rather than falling to zero or failing. The
/// tail is the host's statement about all remaining depths, so a short vector
/// still describes an unbounded tree.
///
/// ´claim:config:an-explicit-schedule-is-read-by-depth-and-clamps-to-its-last-entry-beyond-its-length´
/// ´test:crate:explicit-rounds-for-depth´
#[test]
fn explicit_rounds_for_depth() {
    let schedule = NoiseSchedule::Explicit(vec![50, 30, 10]);
    assert_eq!(schedule.rounds_for_depth(0), 50);
    assert_eq!(schedule.rounds_for_depth(1), 30);
    assert_eq!(schedule.rounds_for_depth(2), 10);
    // Beyond vector length → clamps to last entry.
    assert_eq!(schedule.rounds_for_depth(3), 10);
    assert_eq!(schedule.rounds_for_depth(99), 10);
}

/// The degenerate case of that clamping: a lone entry answers for every depth,
/// which is how a uniform explicit schedule is written.
///
/// (´claim:config:an-explicit-schedule-is-read-by-depth-and-clamps-to-its-last-entry-beyond-its-length´)
/// ´test:crate:explicit-rounds-single-entry´
#[test]
fn explicit_rounds_single_entry() {
    let schedule = NoiseSchedule::Explicit(vec![5]);
    assert_eq!(schedule.rounds_for_depth(0), 5);
    assert_eq!(schedule.rounds_for_depth(1), 5);
    assert_eq!(schedule.rounds_for_depth(100), 5);
}

/// Entries are taken exactly as given and in the order given — no monotonic
/// taper is imposed on the explicit form and none is inferred from it. The
/// clamp past the end still reuses the final entry, whatever its relation to
/// the ones before it.
///
/// (´claim:config:an-explicit-schedule-is-read-by-depth-and-clamps-to-its-last-entry-beyond-its-length´)
/// ´test:crate:explicit-rounds-non-monotonic´
#[test]
fn explicit_rounds_non_monotonic() {
    // Explicit is a direct index — non-monotonic is fine.
    let schedule = NoiseSchedule::Explicit(vec![10, 50, 5, 100, 1]);
    assert_eq!(schedule.rounds_for_depth(0), 10);
    assert_eq!(schedule.rounds_for_depth(1), 50);
    assert_eq!(schedule.rounds_for_depth(2), 5);
    assert_eq!(schedule.rounds_for_depth(3), 100);
    assert_eq!(schedule.rounds_for_depth(4), 1);
    // Beyond length → clamp to last entry.
    assert_eq!(schedule.rounds_for_depth(5), 1);
    assert_eq!(schedule.rounds_for_depth(999), 1);
}

// ── NoiseSchedule::is_disabled ──────────────────────────────

/// An explicit schedule that can never yield a round counts as noise switched
/// off, and the two facts agree: the round count is zero at every depth and the
/// schedule reports itself disabled. That agreement is what lets other rules
/// key off the disabled flag instead of re-deriving it.
///
/// ´claim:config:an-explicit-schedule-that-can-never-yield-a-round-reports-itself-as-noise-switched-off´
/// ´test:crate:explicit-empty-is-disabled´
#[test]
fn explicit_empty_is_disabled() {
    let schedule = NoiseSchedule::Explicit(vec![]);
    assert_eq!(schedule.rounds_for_depth(0), 0);
    assert_eq!(schedule.rounds_for_depth(5), 0);
    assert!(schedule.is_disabled());
}

/// A vector of zeros is disabled just as an empty one is. Emptiness is not the
/// criterion — producing nothing is — so the two ways of writing no warm-up are
/// not held apart.
///
/// (´claim:config:an-explicit-schedule-that-can-never-yield-a-round-reports-itself-as-noise-switched-off´)
/// ´test:crate:explicit-all-zeros-is-disabled´
#[test]
fn explicit_all_zeros_is_disabled() {
    let schedule = NoiseSchedule::Explicit(vec![0, 0, 0]);
    assert_eq!(schedule.rounds_for_depth(0), 0);
    assert!(schedule.is_disabled());
}

/// A lone zero entry disables the schedule at every depth, since the clamp past
/// the end reuses that same zero. A one-element vector cannot describe warm-up
/// that begins further down.
///
/// (´claim:config:an-explicit-schedule-that-can-never-yield-a-round-reports-itself-as-noise-switched-off´)
/// ´test:crate:explicit-single-zero-is-disabled´
#[test]
fn explicit_single_zero_is_disabled() {
    let schedule = NoiseSchedule::Explicit(vec![0]);
    assert_eq!(schedule.rounds_for_depth(0), 0);
    assert_eq!(schedule.rounds_for_depth(10), 0);
    assert!(schedule.is_disabled());
}

/// A single non-zero entry anywhere keeps noise enabled, even where the shallow
/// depths ask for none. A zero at a given depth is a statement about that depth
/// alone, so a schedule may deliberately warm only the deeper cells and still
/// counts as active.
///
/// ´claim:config:one-non-zero-entry-anywhere-keeps-noise-enabled-however-many-depths-ask-for-none´
/// ´test:crate:explicit-mixed-zeros-not-disabled´
#[test]
fn explicit_mixed_zeros_not_disabled() {
    // At least one non-zero entry → not disabled.
    let schedule = NoiseSchedule::Explicit(vec![0, 0, 5]);
    assert!(!schedule.is_disabled());
    assert_eq!(schedule.rounds_for_depth(0), 0);
    assert_eq!(schedule.rounds_for_depth(1), 0);
    assert_eq!(schedule.rounds_for_depth(2), 5);
    assert_eq!(schedule.rounds_for_depth(10), 5);
}

/// A geometric schedule is disabled only when both its root and its floor are
/// zero, because either one alone still produces rounds. The floor lifts every
/// depth to at least its own count, and the root sets the count at the shallow
/// depths before the taper has descended. Reading only one of the two calls a
/// schedule silent that is still asking for warm-up.
///
/// ´claim:config:a-geometric-schedule-is-disabled-only-when-both-its-root-and-its-floor-are-zero´
/// ´test:crate:geometric-is-disabled-when-root-and-min-are-zero´
#[test]
fn geometric_is_disabled_when_root_and_min_are_zero() {
    let schedule = NoiseSchedule::Geometric {
        root: 0,
        decay: 0.5,
        min: 0,
    };
    assert!(schedule.is_disabled());
    assert_eq!(schedule.rounds_for_depth(0), 0);
}

/// One side of the same rule: a positive floor keeps the schedule active no
/// matter how steeply it tapers, since every depth is lifted to at least that
/// count.
///
/// (´claim:config:a-geometric-schedule-is-disabled-only-when-both-its-root-and-its-floor-are-zero´)
/// ´test:crate:geometric-is-not-disabled-when-min-positive´
#[test]
fn geometric_is_not_disabled_when_min_positive() {
    let schedule = NoiseSchedule::Geometric {
        root: 10,
        decay: 0.5,
        min: 1,
    };
    assert!(!schedule.is_disabled());
}

/// The other side, and the one a floor-only reading gets wrong: a positive root
/// with a zero floor is not disabled, because the taper starts at the root and
/// the shallow depths are served from it. The depth-zero count is the root
/// itself, so a schedule reported silent here would be one that immediately
/// asks for hundreds of rounds.
///
/// (´claim:config:a-geometric-schedule-is-disabled-only-when-both-its-root-and-its-floor-are-zero´)
/// ´test:crate:geometric-is-not-disabled-when-root-positive-and-min-zero´
#[test]
fn geometric_is_not_disabled_when_root_positive_and_min_zero() {
    let schedule = NoiseSchedule::Geometric {
        root: 450,
        decay: 0.5,
        min: 0,
    };
    assert!(!schedule.is_disabled());
    assert_eq!(schedule.rounds_for_depth(0), 450);
}

// ── NoiseSchedule::max_rounds ───────────────────────────────

/// The most a geometric schedule can ever ask for is its root, because the
/// taper only descends from there. A host sizing buffers for warm-up can read
/// the ceiling off the root alone, without evaluating the schedule at any
/// depth.
///
/// ´claim:config:the-ceiling-of-a-geometric-schedule-is-its-root-because-the-taper-only-descends´
/// ´test:crate:max-rounds-geometric´
#[test]
fn max_rounds_geometric() {
    let schedule = NoiseSchedule::geometric(999, 0.1, 1);
    assert_eq!(schedule.max_rounds(), 999);
}

/// Where the floor stands above the root, the floor is the ceiling. The taper
/// descends from the root, so with a floor above it every depth is lifted to
/// the floor and the schedule yields that count everywhere — the root never
/// being reached at all. Reading the root alone understates what the schedule
/// asks for, and understates it at every depth rather than at some extreme,
/// which matters because this figure is what a host sizes warm-up buffers
/// from. The pairing is admitted by validation, so it is a configuration a
/// host can actually be holding.
///
/// (´claim:config:the-ceiling-of-a-geometric-schedule-is-its-root-because-the-taper-only-descends´)
/// ´test:crate:max-rounds-geometric-floor-above-root´
#[test]
fn max_rounds_geometric_floor_above_root() {
    let schedule = NoiseSchedule::geometric(10, 0.5, 50);
    let cfg = SentinelConfig::<u64> {
        noise_schedule: NoiseSchedule::geometric(10, 0.5, 50),
        ..SentinelConfig::<u64>::default()
    };
    cfg.validate().expect("a floor above the root is an accepted configuration");

    assert_eq!(schedule.rounds_for_depth(0), 50);
    assert_eq!(schedule.max_rounds(), 50);
}

/// For an explicit schedule the ceiling is the largest entry it holds, not the
/// first. Since the explicit form imposes no ordering, the depth-zero value
/// carries no promise about the rest and the maximum has to be found rather
/// than assumed.
///
/// ´claim:config:the-ceiling-of-an-explicit-schedule-is-its-largest-entry-not-its-first´
/// ´test:crate:max-rounds-explicit´
#[test]
fn max_rounds_explicit() {
    // max_rounds returns the maximum value, not the first.
    let schedule = NoiseSchedule::Explicit(vec![10, 50, 30]);
    assert_eq!(schedule.max_rounds(), 50);
}

/// A schedule with no entries has no largest entry, and its ceiling comes back
/// as zero rather than as an absence the caller must handle. Sizing a buffer
/// for a disabled schedule is asking for nothing.
///
/// (´claim:config:the-ceiling-of-an-explicit-schedule-is-its-largest-entry-not-its-first´)
/// ´test:crate:max-rounds-explicit-empty´
#[test]
fn max_rounds_explicit_empty() {
    let schedule = NoiseSchedule::Explicit(vec![]);
    assert_eq!(schedule.max_rounds(), 0);
}

/// The ceiling is reported exactly, up to the widest count the round type can
/// hold, with nothing clamped or lost on the way out. A capacity hint that
/// quietly saturated would be worse than none at all.
///
/// (´claim:config:the-ceiling-of-an-explicit-schedule-is-its-largest-entry-not-its-first´)
/// ´test:crate:max-rounds-explicit-large´
#[test]
fn max_rounds_explicit_large() {
    let schedule = NoiseSchedule::Explicit(vec![u32::MAX]);
    assert_eq!(schedule.max_rounds(), u32::MAX);
}

// ── NoiseSchedule::Default ──────────────────────────────────

/// The shipped schedule is the geometric one its documentation describes, and
/// it reports itself active. Its numbers are calibrated rather than arbitrary:
/// the root sits a little above the worst-case baseline convergence measured at
/// the default forgetting factor, and the floor covers deep cells whose
/// convergence scales down with analysis width without vanishing.
///
/// ´claim:config:the-shipped-noise-schedule-is-the-calibrated-geometric-one-and-it-is-active´
/// ´test:crate:noise-schedule-default-matches-doc´
#[test]
fn noise_schedule_default_matches_doc() {
    let schedule = NoiseSchedule::default();
    match &schedule {
        NoiseSchedule::Geometric { root, decay, min } => {
            assert_eq!(*root, 450);
            assert!((decay - 0.5).abs() < f64::EPSILON);
            assert_eq!(*min, 50);
        }
        NoiseSchedule::Explicit(_) => panic!("default should be Geometric"),
    }
    assert!(!schedule.is_disabled());
}

// ── ConfigWarning ───────────────────────────────────────────

/// The shipped configuration draws no advisories, because the default schedule
/// was calibrated against the default forgetting factor. Defaults that
/// validated but warned would be an odd thing to ship, so the two sets of
/// defaults are kept consistent with each other.
///
/// ´claim:config:the-shipped-defaults-draw-no-advisories-because-the-schedule-was-calibrated-for-the-default-memory´
/// ´test:crate:default-config-has-no-warnings´
#[test]
fn default_config_has_no_warnings() {
    let cfg = SentinelConfig::<u64>::default();
    assert_eq!(cfg.warnings(), [] as [ConfigWarning; 0]);
}

/// A configuration whose warm-up rounds fall short of what its memory needs is
/// advised, not refused: it is arithmetically sound, but baselines may not
/// converge before real observations arrive, so early scores would be
/// unreliable. Advice and refusal are separate channels — validation would pass
/// this configuration unchanged, and only the warning list carries the
/// concern.
///
/// ´claim:config:insufficient-warm-up-is-advice-rather-than-refusal-because-the-configuration-still-runs´
/// ´test:crate:warns-when-noise-root-too-low-for-lambda-099´
#[test]
fn warns_when_noise_root_too_low_for_lambda_099() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.99,
        noise_schedule: NoiseSchedule::geometric(50, 0.5, 10),
        ..SentinelConfig::<u64>::default()
    };
    let warnings = cfg.warnings();
    assert_eq!(warnings.len(), 1);
    assert!(matches!(
        &warnings[0],
        ConfigWarning::NoiseScheduleInsufficient {
            root: 50,
            recommended_root: 450,
            lambda,
        } if (*lambda - 0.99).abs() < f64::EPSILON
    ));
}

/// How much warm-up is recommended falls with the forgetting factor, because a
/// shorter memory converges sooner: a schedule too thin for a long-memory
/// baseline is adequate for a shorter one. The same schedule draws advice or
/// silence depending on the memory it is paired with, since the recommendation
/// is a relation between the two rather than a property of either.
///
/// ´claim:config:the-recommended-warm-up-falls-with-the-forgetting-factor-because-a-shorter-memory-converges-sooner´
/// ´test:crate:no-warning-when-noise-root-sufficient-for-lambda-095´
#[test]
fn no_warning_when_noise_root_sufficient_for_lambda_095() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.95,
        noise_schedule: NoiseSchedule::geometric(50, 0.5, 10),
        ..SentinelConfig::<u64>::default()
    };
    assert_eq!(cfg.warnings(), [] as [ConfigWarning; 0]);
}

/// Batch size enters the recommendation as well: with few synthetic samples per
/// round, each round buys less convergence, so the same shorter memory demands
/// markedly more rounds and a schedule that was adequate becomes advised
/// against. Warm-up is really measured in observations rather than in rounds,
/// and the recommendation reflects that.
///
/// ´claim:config:a-smaller-noise-batch-raises-the-recommended-round-count-because-each-round-buys-less-convergence´
/// ´test:crate:warns-when-small-batch-and-lambda-095´
#[test]
fn warns_when_small_batch_and_lambda_095() {
    // At λ=0.95, b=4, recommended root is 200.
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.95,
        noise_batch_size: 4,
        noise_schedule: NoiseSchedule::geometric(50, 0.5, 10),
        ..SentinelConfig::<u64>::default()
    };
    let warnings = cfg.warnings();
    assert_eq!(warnings.len(), 1);
    assert!(matches!(
        &warnings[0],
        ConfigWarning::NoiseScheduleInsufficient {
            recommended_root: 200,
            ..
        }
    ));
}

/// The advisory judges whatever the schedule actually yields at depth zero,
/// whichever variant it is written in. An explicit schedule generous enough at
/// the root passes the same check a geometric one would, so the recommendation
/// is about warm-up delivered and not about how the host chose to express
/// it.
///
/// ´claim:config:the-advisory-judges-the-rounds-a-schedule-actually-yields-at-depth-zero-whatever-its-variant´
/// ´test:crate:no-warning-for-explicit-schedule-with-enough-rounds´
#[test]
fn no_warning_for_explicit_schedule_with_enough_rounds() {
    let cfg = SentinelConfig::<u64> {
        forgetting_factor: 0.99,
        noise_schedule: NoiseSchedule::Explicit(vec![500, 300, 100]),
        ..SentinelConfig::<u64>::default()
    };
    assert_eq!(cfg.warnings(), [] as [ConfigWarning; 0]);
}

/// A rendered advisory carries the numbers a host needs in order to act on it:
/// the root it found, the root it recommends, and the forgetting factor that
/// set that recommendation. Advice naming only the problem would leave the
/// reader to re-derive the target.
///
/// ´claim:config:a-rendered-advisory-names-the-root-it-found-the-root-it-recommends-and-the-memory-that-set-it´
/// ´test:crate:warning-display-is-informative´
#[test]
fn warning_display_is_informative() {
    let w = ConfigWarning::NoiseScheduleInsufficient {
        root: 50,
        recommended_root: 450,
        lambda: 0.99,
    };
    let msg = w.to_string();
    assert!(msg.contains("50"));
    assert!(msg.contains("450"));
    assert!(msg.contains("0.99"));
}

// ── Construction refusal: coordinate width ──────────────────

/// A coordinate width narrower than the smallest dimension a subspace tracker
/// can model is refused at construction, with the same structured failure the
/// configuration faults carry. The width is a parameter of the type rather than
/// a field of the configuration, so validating the configuration alone can
/// never see it, and the root tracker spans the whole width — at one dimension
/// its lone basis vector spans the entire space, novelty is identically zero,
/// and the tracker reports a settled model of everything while modelling
/// nothing. Refusing is what lets the constructor's success mean the sentinel
/// it returns can measure.
///
/// ´claim:config:a-coordinate-width-below-the-tracker-minimum-is-refused-at-construction´
/// ´test:crate:rejects-coordinate-width-below-the-tracker-minimum´
#[test]
fn rejects_coordinate_width_below_the_tracker_minimum() {
    use crate::SpectralSentinel;

    let Err(err) = SpectralSentinel::<u64, u64, 1>::new(SentinelConfig::<u64>::default()) else {
        panic!("a coordinate width below the tracker minimum must be refused");
    };
    assert!(err.0.contains(&ConfigError::TrackerDimensionTooSmall {
        width: 1,
        minimum: crate::MIN_TRACKER_DIM
    }));
}

/// The narrowest width the tracker can model is admitted, which fixes the
/// boundary rather than leaving it to be inferred from the refusal alone. Two
/// dimensions leave one residual degree of freedom, which is the least that
/// makes a novelty reading mean anything.
///
/// (´claim:config:a-coordinate-width-below-the-tracker-minimum-is-refused-at-construction´)
/// ´test:crate:accepts-the-narrowest-modellable-coordinate-width´
#[test]
fn accepts_the_narrowest_modellable_coordinate_width() {
    use crate::SpectralSentinel;

    let sentinel = SpectralSentinel::<u64, u64, 2>::new(SentinelConfig::<u64>::default()).unwrap();
    assert_eq!(
        sentinel.cells_tracked(),
        1,
        "the root tracker is built at the narrowest width"
    );
}

/// A width fault and a configuration fault come back together rather than one
/// at a time, so a host repairing a sentinel that is wrong in both respects
/// learns both in a single pass. This is the collecting behaviour the
/// configuration's own validation promises, extended to the one fault that
/// validation cannot reach by itself.
///
/// (´claim:config:a-coordinate-width-below-the-tracker-minimum-is-refused-at-construction´)
/// ´test:crate:collects-a-width-fault-alongside-a-configuration-fault´
#[test]
fn collects_a_width_fault_alongside_a_configuration_fault() {
    use crate::SpectralSentinel;

    let cfg = SentinelConfig::<u64> {
        max_rank: 0,
        ..SentinelConfig::<u64>::default()
    };
    let Err(err) = SpectralSentinel::<u64, u64, 1>::new(cfg) else {
        panic!("a coordinate width below the tracker minimum must be refused");
    };
    assert!(
        err.0
            .iter()
            .any(|e| matches!(e, ConfigError::TrackerDimensionTooSmall { .. }))
    );
    assert!(err.0.contains(&ConfigError::MaxRankZero));
}
