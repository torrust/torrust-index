# Silence widens uncertainty by computed snapshot age · `plan:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor`

Keeping (´claim:risk:silence-inflates-reported-uncertainty-by-the-staleness-factor´) would establish that a host can reproduce the uncertainty widening from the published snapshot age and configured temporal factor, while every point-estimate input remains fixed.

## What the promise says, precisely · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-contract`

The controlled comparison holds one published model snapshot, one request, one configuration, and the absence of labels or reports constant, then changes only the monotonic assessment time by a known interval.

Let `gamma_t` be the core temporal factor, let `a_0` and `a_1` be the two public `health.snapshot_age_seconds` readings, let `delta_h = (a_1 - a_0) / 3600`, and let `F = 1 / gamma_t^delta_h`.

The reference configuration declares `gamma_t = 0.9999` per hour with `0 < gamma_t < 1` (´tab:config:temporal´), so an advance of one hundred hours gives `F = (1 / 0.9999)^100`, approximately `1.01`.

The temporal algorithm leaves model means unchanged and multiplies covariance by `F` at read time (´alg:temporal:lazy-application´), and the assessment definition applies the same correction to variance rather than to a point estimate (´def:runtime:time-correction´).

The public risk schema calls `sigma_eff` a blended standard deviation and calls `uncertainty` its probability-space delta-method image (´schema:risk:basis´), (´def:risk:probability´).

Consequently the specification-aligned measurable relation is `stale.sigma_eff / fresh.sigma_eff = sqrt(F)` and `stale.uncertainty / fresh.uncertainty = sqrt(F)` in a fixture whose probability and calibration scale are unchanged; equivalently, either squared ratio is `F` within the harness default tolerance.

The literal intent wording instead assigns `F` to the reported standard-deviation ratio. That reading and the specification-aligned relation cannot both hold, so the witness depends on the contract decision recorded below.

“Changes in no other way” means no model, calibration, report, lifecycle, or standardisation change: the snapshot version, standardisation phase and accepted count, risk point estimates, blend weight, calibration scale, contribution counts, borrowed share, convergence flag, and empirical coverage inflation remain equal. Assessment identity, pending-buffer bookkeeping, assessment counters, and snapshot age are request-path metadata and are outside that equality.

The blend variance contains both time-corrected within-model variances and a between-model disagreement term (´prop:risk:blend-variance´), so exact factorisation by `F` requires a fixture in which the disagreement term is zero.

## What the code offers today · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-code-surface`

The public path through the world harness is `World::builder(config)` to construct the real `Assayer`, `World::request` to build a fixed request, and `World::assess` to obtain the public `RiskAssessment` and its `risk` and `health` fields.

`World::settle_cold_ramp_with` deterministically advances the observation-authorised cold ramp to an in-service snapshot without moving virtual time. Once that phase is in service, assessment no longer offers cold-ramp observations, so the comparison can read one immutable snapshot.

`World::clock()` exposes the injected `VirtualClock`, and `VirtualClock::advance(Duration)` advances the monotonic domain used for snapshot publication and assessment age. The clock record separates this duration domain from persistent calendar time (´dec:clock:two-domains´).

The production assessment reads the current monotonic instant, subtracts the snapshot publication instant, passes the elapsed duration and configured `gamma_t` through the shared decay helper, and inverts the result before blend evaluation.

`HealthSnapshot::snapshot_age_seconds` exposes the same elapsed interval to the host (´schema:output:health-snapshot´). The existing crate test advances the virtual clock by ninety seconds and observes exactly that age (´test:crate:snapshot-carries-version-age-drift-and-inflation´).

`World::tol().default` supplies the declared `1e-9` comparison tolerance (´const:assayer:scenario-tolerance-defaults-form-x57224f5b´), and the harness's `assert_near` helper supplies an absolute floating-point assertion.

The clock record requires production to use the shared decay helper but deliberately permits tests to exponentiate directly as an independent oracle (´rule:clock:no-direct-exponentiation´).

The closest integration home is the label-free derivation-purity module: its repeated-assessment witness already separates observation-authorised geometry from outcome-learned state (´test:integration:repeated-assessment-advances-no-outcome-learned-state´), and its cold-ramp witness already treats the reported phase and accepted count as snapshot coordinates (´test:integration:cold-ramp-trajectory-stays-inside-the-falsifier´).

The nearest arithmetic checks prove that time correction changes variance, leaves `rho` alone, and cancels from the raw anchor weight (´test:unit:time-correction-affects-variance´), (´test:unit:time-correction-does-not-affect-rho´), (´test:unit:time-correction-cancels-in-weight´). None drives elapsed time through the public assessment path or checks the host-reproducible ratio.

The standing harness plan records persistent clock travel and names this promise under (´entry:assayer:harness-stage-clock´). Its proposed `World::advance` and `World::travel_to` conveniences remain absent, but this witness already has the monotonic seam it needs through `World::clock().advance` and therefore does not depend on those wrappers.

## The witness · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-witness`

The integration test belongs beside the existing label-free derivation-purity scenarios under a function name such as `silence_widens_uncertainty_from_snapshot_age_alone`.

- Setup: construct an explicit one-channel configuration with `temporal.gamma_t_core = 0.9999`, a fixed seed, no Sentinels, no outcome axes, and one fixed entity request.
- Setup: call `settle_cold_ramp_with` with that request, flush its barrier as the helper already does, and assert that the resulting phase is in service before taking either measurement.
- Setup: assess once at the fixed epoch and retain the fresh `RiskAssessment`; assert an empty reporting surface, zero label and calibration evidence, equal zero model means through the public point estimates, and a positive finite `sigma_eff` and `uncertainty`.
- Stimulus: submit no label, report, registration, reset, or lifecycle event; advance the shared virtual clock by `Duration::from_secs(100 * 60 * 60)`; assess the identical request once more.
- Observation: derive `delta_h` only from the two public snapshot-age readings, read `gamma_t_core` from `world.assayer().config().temporal.gamma_t_core`, and compute `F = (1.0 / gamma_t_core).powf(delta_h)` in test code.
- Assertion: require the stale snapshot age minus the fresh age to equal `360_000.0`, require both assessments to carry the same nonzero snapshot version and the same standardisation phase and count, and require `F` to agree with the stated one-hundred-hour example within `world.tol().default`.
- Assertion: under the specification-aligned contract, compare each public standard-deviation ratio with `sqrt(F)` and each squared ratio with `F` via `assert_near`; under the literal contract, compare the public ratio itself with `F` after the selected semantic change.
- Assertion: compare `p_bad`, `borrowed_share`, `anchor_weight`, `rho_eff`, `kappa_eff`, `p_bad_sister`, `p_bad_operational`, `intervention_effectiveness`, `anchor_converged`, reporting count, both calibration record counts, outcome collection, standardisation coordinates, snapshot version, drift flag, and empirical `uncertainty_inflation` for equality.

The cold prior supplies the exact factorisation: sister and anchor means agree at zero, so the disagreement term vanishes, and the evidence-only tempering ceiling scales from the same time-corrected prior variance (´dec:risk:evidence-only-uncertainty´).

The fails-before is diagnostic rather than a mutation run: disabling staleness leaves both ratios at one; using the wrong elapsed unit or temporal factor yields the wrong reproducible ratio; leaking correction into means moves a point-estimate field; allowing a publication between readings changes the version or standardisation coordinates and fails the fixture guard.

## What is missing · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-missing`

No new clock, wait, drift-budget, liveness-deadline, scenario-builder, or comparison helper is required. The existing virtual monotonic clock, cold-ramp barrier, public assessment path, and default tolerance are sufficient.

**Entry (Align the intent's uncertainty unit)** · `entry:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-align-unit`

Resolve whether the promise binds the variance factor or the public standard-deviation ratio, then align the intent prose and its future test gloss. The change is approximately 5–12 lines and depends on the unit decision under the observation below; it has no harness or sibling-lane dependency.

**Entry (Land the public-path witness)** · `entry:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-land-witness`

Add the integration test, its module index row, and its generated test-catalog row after the unit is aligned. The change is approximately 55–80 lines and depends on the preceding entry; it uses only existing harness pieces.

## Risks and open questions · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-risks`

**Observation (Variance and public uncertainty use different units)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-unit-mismatch`

The specification-aligned alternative retains `F` as the covariance and variance multiplier, tests `sqrt(F)` on `sigma_eff` and `uncertainty`, and tests `F` on their squares. The literal alternative makes the public standard deviation grow by `F`, which requires a squared variance correction or a public-field semantic change and conflicts with the current temporal and risk definitions. The first alternative preserves the existing dimensional contract and is the narrower resolution.

**Observation (Exact factorisation is fixture-sensitive)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-blend-scope`

A trained blend can carry nonzero sister-anchor disagreement, and that additive term is not itself a covariance aged by `F`. The cold equal-mean fixture proves the staleness mechanism exactly; a universal assertion over arbitrary blends would claim more than the blend equation provides.

**Observation (Publication must remain controlled)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-publication-control`

The asynchronous cold ramp can publish between ordinary cold assessments. Settling it before the baseline removes that source of nondeterminism, while equality of snapshot version, phase, and accepted count makes any unexpected publication a visible fixture failure rather than silent noise.

**Observation (Two inflation quantities share a surface)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-inflation-names`

`health.uncertainty_inflation` is an empirical-coverage diagnostic carried from calibration, not the temporal factor. The witness keeps it unchanged and computes temporal `F` from snapshot age plus configuration, preventing an accidental assertion against the unrelated field.

**Observation (Request metadata necessarily moves)** · `obs:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-change-scope`

Two assessments necessarily receive different identifiers and advance request-path counters and pending-buffer state. Excluding those fields while pinning all model-derived and configuration-derived quantities makes “changes in no other way” measurable without asserting that assessment is side-effect free.

## Acceptance · `sec:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor-acceptance`

The implementation report identifies the selected unit contract, the aligned intent text, and the integration witness that cites the promise.

The report shows the test's fresh and stale snapshot ages, unchanged snapshot version and standardisation coordinates, configured `gamma_t_core`, independently computed `F`, observed public uncertainty ratios, squared ratios, and the tolerance used.

The report confirms that all invariant risk and health fields listed by the witness remain equal and that the fails-before cases distinguish disabled, mis-scaled, mean-changing, and republishing implementations.

The report records successful formatting, corpus linting, the Assayer integration target containing the witness, and the package test suite, with no wall-clock sleeps and no executed production mutation.
