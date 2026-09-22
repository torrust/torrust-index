# Silence widens uncertainty by computed snapshot age · `plan:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor`

Keeping (´claim:risk:silence-inflates-reported-uncertainty-by-the-staleness-factor´) would establish that a host can reproduce the uncertainty widening from the published snapshot age and configured temporal factor while every point-estimate input remains fixed.

## What the promise says, precisely · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-contract`

The controlled comparison holds one published model snapshot, one request, one configuration, and the absence of labels and reports constant, then advances scenario time by a known interval. The harness moves both typed clock domains together, but with no populated Ledger, identity, report, or label state the only assessment input that changes is the monotonic snapshot age.

Let `gamma_t` be the core temporal factor, let `a_0` and `a_1` be the two public `health.snapshot_age_seconds` readings, let `delta_h = (a_1 - a_0) / 3600`, and let `F = 1 / gamma_t^delta_h`.

The reference configuration declares `gamma_t = 0.9999` per hour with `0 < gamma_t < 1` (´tab:config:temporal´), so an advance of one hundred hours gives `F = (1 / 0.9999)^100`, approximately `1.01005` and therefore `1.01` to two decimal places.

The temporal algorithm leaves model means unchanged and multiplies covariance by `F` at read time (´alg:temporal:lazy-application´), and the assessment definition applies that correction to each model's quadratic-form variance without changing its point estimate (´def:runtime:time-correction´).

The public risk schema defines `sigma_eff` as the square root of blended variance and `uncertainty` as its first-order probability-space image (´schema:risk:basis´) (´def:risk:probability´).

The promise therefore assigns `F` to variance and asserts `stale.sigma_eff / fresh.sigma_eff = sqrt(F)` and `stale.uncertainty / fresh.uncertainty = sqrt(F)` when probability, calibration scale, borrowed share, and the between-model disagreement term remain fixed; equivalently, either squared ratio is `F` within the named default tolerance.

`F` does not multiply either public standard-deviation ratio; that reading would require a squared variance correction or a public-field semantic change contrary to the temporal and risk definitions.

“Changes in no other way” means no model, calibration, report, lifecycle, or standardisation change: snapshot version, standardisation phase and accepted count, risk point estimates, blend weight, calibration scale, contribution counts, borrowed share, convergence flag, and empirical coverage inflation remain equal. Assessment identity, pending-buffer bookkeeping, assessment counters, and snapshot age are request-path metadata and are outside that equality.

The blend variance contains both time-corrected within-model variances and a between-model disagreement term (´prop:risk:blend-variance´), so exact factorisation by `F` requires a fixture in which the disagreement term is zero.

**Entry (Align the intent's uncertainty unit)** · `entry:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-align-unit`

**DONE.** The claim assigns `F` to variance, `sqrt(F)` to both public standard-deviation ratios, and `F` to their squared ratios. The rounded `1.01` example is explanatory rather than an assertion under the much narrower harness tolerance.

## What the code offers today · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-code-surface`

The unified harness supplies `scenario_with_config` and `Scenario` around one real engine, while `World::request` builds a fixed `RequestContext` and `World::assess` returns the public `RiskAssessment` with its risk and health fields (´tab:assayer:harness-implementation-library-roster´).

`World::settle_cold_ramp_with` drives the observation-authorised ramp through `World::flush_observations` until the published phase is in service. The named `World::flush_labels` and `World::flush_identity_maintenance` barriers also exist, but this label-free, identity-free witness submits no work to their queues (´entry:assayer:harness-closed-barriers´).

`World::advance` and `World::travel_to` are the forward-only scenario-time verbs. They move the persistent and intra-process readings together and settle the identity-maintenance and Ledger-GC work made due, while explicitly excluding cold-ramp observations, checkpoint scheduling, and health-event consumption (´entry:assayer:harness-scenario-time´) (´dec:clock:two-domains´).

`HealthSnapshot::snapshot_age_seconds` exposes the same elapsed interval used by the read-time correction (´schema:output:health-snapshot´). The seeded publication witness now proves that `World::advance` appears exactly as public snapshot age and that the next label publication resets the age on a later version (´test:crate:publication-staleness-seeded-sweep´); the older health-surface witness independently checks a ninety-second age together with version, drift, and empirical inflation (´test:crate:snapshot-carries-version-age-drift-and-inflation´).

`decay_recurrence` supplies a specification-formula oracle through arithmetic independent of production exponentiation, and `assert_near` with `World::tol().default` supplies the named `1e-9` absolute comparison tolerance (´dec:harness:oracle-tier´) (´const:assayer:scenario-tolerance-defaults-form-x57224f5b´). Direct exponentiation remains permitted in a test oracle, but this witness can use the shared oracle that has since landed (´rule:clock:no-direct-exponentiation´).

The finished owned probes are `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`; long streams use `PlaybackRow`, `PlaybackBarrierPolicy`, and `playback`; trained setup uses `TrainedStateFixture`; and generated invariant cases use `run_seeded_sweep` (´dec:harness:probe-contract´) (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´). This witness needs none of those surfaces because it reads two public assessments from an unlabelled cold model, drives one fixed time interval, and checks one specified configuration rather than a generated case space.

The closest arithmetic tests prove separately that time correction changes blended variance, leaves the blended point estimate unchanged, and cancels from the raw blend weight (´test:unit:time-correction-affects-variance´) (´test:unit:time-correction-does-not-affect-rho´) (´test:unit:time-correction-cancels-in-weight´). The public assessment target already proves the cold-ramp phase, accepted count, and version join on both health surfaces (´test:integration:standardisation-transition-reported-on-both-surfaces´), but no current test cites this claim or composes public snapshot age, the configured temporal factor, and both public uncertainty ratios.

## The witness · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-witness`

The `assess_integration` integration target is the destination for `assess_integration::silence_widens_uncertainty_from_snapshot_age_alone`, beside the existing public assessment and standardisation-health witnesses.

- Setup: construct a `Scenario` with `scenario_with_config`, `test_infrastructure`, an explicit one-channel configuration whose `temporal.gamma_t_core` is `0.9999`, a fixed seed, no Sentinels, no outcome axes, and one fixed request.
- Setup guard: call `World::settle_cold_ramp_with` with that request, then require the first assessment's public standardisation phase to be in service and its accepted count to equal `standardisation.n_init`. The helper has already crossed `World::flush_observations`, so no sleep, poll, or second wait is admitted; setup validation remains separate from the ratio oracle (´dec:harness:separate-validation´).
- Fresh observation: retain the first `RiskAssessment`, require a nonzero snapshot version and zero snapshot age within `World::tol().default`, require empty per-Sentinel and outcome collections, require clean degradation, and require positive finite `sigma_eff` and `uncertainty`.
- Stimulus: submit no label, report, registration, reset, or lifecycle event; call `World::advance(Duration::from_secs(360_000))`; then assess a clone of the identical request once more. This is one fixed stimulus rather than a long stream, so playback would add no execution semantics.
- Oracle: derive `delta_h` only from the two public snapshot-age readings, call `decay_recurrence(1.0, 1.0, 0, gamma_t_core, delta_h)` to obtain the retention factor, and invert it to obtain `F` (´dec:harness:oracle-tier´). Require the age difference to equal `360_000.0` and `delta_h` to equal `100.0` within the named default tolerance; do not compare the exact oracle with the rounded display value `1.01`.
- Ratio assertions: compare each public standard-deviation ratio with `sqrt(F)` and each squared ratio with `F` through `assert_near` and `World::tol().default`.
- Isolation assertions: require the two assessments to carry the same nonzero snapshot version, standardisation phase and count, `p_bad`, `borrowed_share`, `anchor_weight`, `rho_eff`, `kappa_eff`, `p_bad_sister`, `p_bad_operational`, `intervention_effectiveness`, `anchor_converged`, `n_sentinels_reporting`, both calibration-record counts, empty outcome and per-Sentinel collections, convergence stage, Sentinel coverage, calibration flags, drift flag, empirical `uncertainty_inflation`, and zero-Sentinel flag. Compare floating values within the named default tolerance and exact fields exactly.

The cold prior supplies the exact factorisation: the sister and anchor means agree at zero, so the disagreement term vanishes, and the evidence-only tempering ceiling scales from the same time-corrected prior variance (´dec:risk:evidence-only-uncertainty´).

The fails-before is diagnostic rather than an executed mutation: disabling staleness leaves both ratios at one; using the wrong elapsed unit, temporal factor, or reciprocal yields the wrong reproducible ratio; leaking correction into means moves a point-estimate field; and allowing a publication between readings changes the version or standardisation coordinates.

## What is missing · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-missing`

No harness capability is missing: scenario time, queue barriers, the public readings, the decay oracle, and the named comparator have all landed.

**Entry (Land the public-path witness)** · `entry:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-land-witness`

Add `assess_integration::silence_widens_uncertainty_from_snapshot_age_alone` to the `assess_integration` integration target, add its in-module index row and test mint citing this intent, and regenerate the test catalogue. The test uses only the existing harness surfaces named above.

## Risks and open questions · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-risks`

**Observation (Variance and public uncertainty use different units)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-unit-mismatch`

`F` is the covariance and variance multiplier, so `sigma_eff` and `uncertainty` grow by `sqrt(F)` at fixed probability and calibration scale, and their squares grow by `F`. The shared oracle returns the retention factor and the witness must invert it once; omitting or duplicating that reciprocal tests the opposite quantity.

**Observation (Exact factorisation is fixture-sensitive)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-blend-scope`

A trained blend can carry nonzero sister-anchor disagreement, and that additive term is not itself a covariance aged by `F`. The cold equal-mean fixture proves the staleness mechanism exactly; a trained-state fixture would introduce the term this witness must exclude and would make a universal assertion over arbitrary blends claim more than the blend equation provides.

**Observation (Publication must remain controlled)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-publication-control`

`World::advance` settles time-activated identity maintenance and Ledger GC but does not drain cold-ramp observations, checkpoint scheduling, or health events (´entry:assayer:harness-scenario-time´). Settling the ramp before the baseline and submitting none of the excluded work between readings makes that exclusion harmless, while equality of snapshot version, phase, and accepted count turns any unexpected publication into a fixture failure.

**Observation (Two inflation quantities share a surface)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-inflation-names`

`health.uncertainty_inflation` is an empirical-coverage diagnostic carried from calibration, not the temporal factor. The witness keeps it unchanged and computes temporal `F` from snapshot age plus configuration, preventing an assertion against the unrelated field.

**Observation (Request metadata necessarily moves)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-change-scope`

Two assessments necessarily receive different identifiers and advance request-path counters and pending-buffer state. Excluding those fields while pinning every model-derived, configuration-derived, and coordinate-system quantity makes “changes in no other way” measurable without asserting that assessment is side-effect free.

No maintainer decision blocks the witness: the specification fixes the rate, correction, public units, and blend equation, and every required harness surface exists.

## Acceptance · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-acceptance`

The `assess_integration` integration target contains the test at Rust path `assess_integration::silence_widens_uncertainty_from_snapshot_age_alone`, and that test's documentation cites this intent and appears in the target index and generated catalogue.

The implementation report shows the fresh and stale snapshot ages, unchanged snapshot version and standardisation coordinates, configured `gamma_t_core`, oracle retention factor and `F`, observed public uncertainty ratios and squared ratios, and the named tolerance used.

The report confirms that every invariant risk and health field listed by the witness remains equal and that the diagnostic fails-before cases distinguish disabled, mis-scaled, reciprocal-reversed, mean-changing, and republishing implementations.

The report records successful formatting, corpus linting, the `assess_integration` target, and the package test suite, with no wall-clock sleep, poll, ad hoc deadline, tolerance widening, or executed production mutation.
