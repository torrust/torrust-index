# Keeping an Unreported Outcome Axis at Its Prior · `plan:assayer:intent-an-axis-nobody-reports-stays-at-its-prior`

This plan keeps the promise that an outcome axis nobody reports stays at its prior (´claim:lifespan:an-axis-nobody-reports-stays-at-its-prior´); the witness establishes exact prior state through assessment traffic and clean erasure on retirement.

## What the promise says, precisely · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-promise`

For a newly registered axis $a$, the measurable starting state is a zero mean vector $\mu_a$, a precision matrix $B_a = \lambda_\text{prior} I$, and therefore a covariance $\Sigma_a = \lambda_\text{prior}^{-1} I$ at the full post-registration model width. The per-axis model has the same Gaussian structure and lifecycle algebra as the risk models (´def:axis:per-axis-model´), while the prior-only prediction is zero with raw uncertainty $\kappa_a / \sqrt{\lambda_\text{prior}}$ when the clock is fixed (´def:axis:prior-only-prediction´).

The candidate axis owns $r_a$ feature coordinates given by the dimension formula (´tab:feature:dimension-formula´). Registration adds those coordinates to the operational, sister, and every pre-existing outcome-axis model with mean zero, precision $\lambda_\text{prior} I$, covariance $\lambda_\text{prior}^{-1} I$, and zero cross-block terms, which is the independent Gaussian extension (´thm:gaussian:extension´) required by the registration algorithm (´alg:registry:axis-registration´).

The anchor model owns no candidate coordinates and remains unchanged. The registered axis's model and every candidate-owned block in the other non-anchor models remain at those starting values after any finite sequence of assessments that supplies no outcome value for the candidate.

Retirement removes the candidate model, its registry and dimension-map entries, its standardisation slots, and every coordinate it contributed. With an untouched independent block, marginalisation has a zero correction and leaves each surviving mean and precision exactly at its pre-registration value; the recomputed covariance agrees within the scenario drift band (´thm:gaussian:marginalisation´), and destructive deregistration retains no axis payload (´dec:construction:destructive-deregistration´).

The witness uses the configured prior precision rather than the default as an accidental constant. The reference configuration publishes $\lambda_\text{prior}$ as a host-settable model quantity (´tab:config:risk-model´), and the axis registration schema separately names $\kappa_{a,0}$ as compression scale rather than precision (´schema:registry:axis-record´).

## What the code offers today · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-code-today`

The public path begins with the `scenario_with_config` scenario builder, builds the real engine through `WorldBuilder`, registers a Sentinel and axes through `World`, derives assessments through `World::derive_default`, and retires the candidate through `World::deregister_axis`. `World::assayer` remains an escape hatch for an explicit registration record, although this witness only needs the existing helper's compression scale of one.

`World::settle_cold_ramp_with` supplies an accepted request stream before the baseline, `VirtualClock` remains fixed, and `STATE_DEADLINE` bounds publication waits in the liveness harness. The completed harness skeleton already supplies these scenario, lifecycle, deterministic-clock, tolerance, and barrier pieces (´entry:assayer:harness-stage-skeleton´).

The standing testing plan contains no gap entry dedicated to this promise. Its open tapes and health-accessor entry includes model-state inspection and snapshot diffing, which is the nearest standing prerequisite rather than coverage of the promise itself (´entry:assayer:harness-stage-tapes´).

The production outcome-axis registration handler extends existing full-dimensional models with `config.model.lambda_prior`, then constructs the new axis model with `reg.initial_kappa` in the constructor's prior-precision position while also storing that value as `kappa_a`. A configured prior unequal to the compression scale makes that distinction observable immediately.

The published `ModelSnapshot` exposes means and covariances but excludes precision by design (´dec:retention:precision-excluded´). The working precision matrices and the structural state required for the retirement diff remain owner-private, so the public integration harness cannot yet make the exact assertion.

The nearest lifecycle round trip proves name resolution after axis removal (´test:integration:register-deregister-axis-round-trip´), and the churn cycle proves repeated structural health and finiteness (´test:integration:axis-register-deregister-churn-cycle´). The nearest ledger witness proves that registration does not perturb a risk-basis output (´test:integration:outcome-axis-registration-does-not-perturb-risk-basis´); none of the three observes the candidate model's exact prior, its blocks in every other model, or the post-retirement posterior identity.

## The witness · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-witness`

**Decision (One barriered lifecycle witnesses the whole promise)** · `dec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-one-barriered-lifecycle`

The witness belongs beside the existing axis lifecycle round trip (´test:integration:register-deregister-axis-round-trip´) and churn cycle (´test:integration:axis-register-deregister-churn-cycle´) because it drives only public lifecycle and assessment operations through `World`; the new inspection surface is test support and not a production snapshot contract.

- **Setup.** Build a seeded scenario with $\lambda_\text{prior}=1/4$, one registered but silent Sentinel, and one incumbent non-spatial outcome axis, then settle the cold ramp and capture the survivor baseline. Register a spatial candidate with the harness's $\kappa_{a,0}=1$, giving it two owned coordinates and making its expected covariance diagonal four and its expected raw uncertainty two.
- **Registration observation.** Cross the owner barrier and capture the candidate model, all non-anchor models, the anchor control, the dimension map, the axis registry, and the standardisation shape. Resolve the candidate's coordinate range only through the dimension map, whose role as sole index resolver is fixed by (´dec:vector:sole-resolver´).
- **Stimulus.** Derive $n_\text{init}+1$ assessments from a fixed request while the virtual clock remains fixed, submit no labels, and retain the candidate outcome prediction from every assessment. The count is tied to the configured cold-ramp horizon and represents traffic beyond one complete warm-up rather than a small hand-picked sample.
- **Traffic observation.** Cross the same owner barrier after the batch and capture the same projection. Assessment may move observational geometry but no outcome-learned state (´dec:ordering:evidence-authority´), so this comparison isolates an accidental write to posterior or lifecycle state.
- **Prior assertions.** At registration and after traffic, assert candidate $\mu=0$, $B=(1/4)I$, $\Sigma=4I$, raw prediction zero, and raw uncertainty two. In the operational, sister, and incumbent-axis models, assert the two candidate coordinates have zero mean, prior precision and covariance blocks, and zero coupling to every survivor coordinate; assert that the anchor projection is unchanged.
- **Retirement assertion.** Deregister the candidate, cross the barrier, and compare the survivor projection with the pre-registration baseline. Model membership, registry membership, dimension-map keys, and standardisation width agree exactly; surviving means and precision matrices agree exactly; surviving covariance matrices use the declared default drift budget because retirement refactors them by construction (´dec:posterior:always-refactor´).
- **Fails-before.** The current constructor initializes the candidate with $B=I$ and $\Sigma=I$ because it passes $\kappa_{a,0}=1$ where the configured prior is $1/4$, so the first internal checkpoint fails before traffic. A deliberately broken assessment path that updates an unreported model changes a later matrix checkpoint, and a broken removal that retains or removes the wrong coordinates breaks the final survivor diff.

Exact zeros, dimensions, registry membership, and the precision sentinels use the bit-identical tolerance; covariance values produced by factorisation and public prediction scalars use the default tolerance. Both budgets come from the harness tolerance register rather than local literals (´tab:assayer:harness-scenario-tolerances´).

## What is missing · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-missing-support`

**Entry (A barriered posterior-state probe)** · `entry:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-posterior-state-probe`

Add a testing-only owner command and `World` accessor that return owned copies of each model's identifier, mean, precision, covariance, and update metadata together with the dimension map, axis registry, standardisation classes, and anchor control. The command itself is the publication barrier, its response uses `STATE_DEADLINE`, and the surface remains hidden from production documentation in the same manner as the existing integration-test helpers.

The estimate is roughly 110–160 lines across the owner command enum, the owner loop, the world harness, and a small testing projection type. It depends on the inspection and snapshot-diffing work in (´entry:assayer:harness-stage-tapes´) and has no sibling-intent dependency.

**Entry (Axis-block prior and survivor assertions)** · `entry:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-axis-block-assertions`

Add testing assertions that resolve an axis-owned range from the captured dimension map, check a diagonal prior block plus zero cross-couplings in every full-dimensional model, and compare the survivor projection while excluding publication sequence and lifecycle-version counters. The helper reports the model and semantic coordinate of the first mismatch, which keeps an ordering defect distinct from a numerical defect.

The estimate is roughly 60–90 lines in the existing testing-assertion module plus exports through the testing facade. It depends on the posterior-state probe above and the existing tolerance registry; it introduces no production API and no sibling-intent dependency.

## Risks and open questions · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-risks-open-questions`

**Observation (The axis prior has two plausible configuration homes)** · `obs:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-prior-source`

The promise names one $\lambda_\text{prior}$, existing-model extension reads `ModelConfig::lambda_prior`, and the registration schema has no axis-specific precision field; the reference-configuration prose also describes a Schur offset using the precision set by an axis registration. One resolution keeps the global model prior for every axis, while the other adds an explicit axis prior-precision field. The witness proceeds with the global prior because that is the value the public type currently exposes and the value every foreign block already receives; choosing the second resolution changes the setup and the registration schema but not the assertions' shape.

**Observation (Assessment traffic and omitted labels are distinct stimuli)** · `obs:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-assessments-versus-labels`

The promise quantifies assessments, so the primary witness submits no labels. Extending it to labels that report an incumbent axis while omitting the candidate would also exercise partial outcome reporting (´req:registry:partial-outcome-reporting´), but current model-wide label forgetting can move the candidate-owned blocks in foreign models even when the candidate has no value. A broader reading therefore requires a semantic choice between exempting never-observed coordinates from forgetting and narrowing “alongside it” to assessment-only traffic; that choice does not weaken the primary assessment witness.

**Observation (The exactness boundary follows the algebra)** · `obs:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-exactness-boundary`

Zero coupling makes the precision correction on retirement exactly zero, while the covariance is deliberately regenerated by factorisation. Requiring bit identity for the regenerated covariance would test a decomposition's rounding history rather than clean marginalisation, so the survivor covariance alone uses the default drift budget and every structural or precision assertion remains exact.

**Observation (Determinism excludes unrelated maintenance)** · `obs:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-determinism-boundary`

The scenario registers no identity dimension, advances no virtual time, and submits no label, leaving competitive maintenance, time decay, label ordering, and eligibility outside the witness. A silent Sentinel still gives the candidate a non-empty spatial block, and the incumbent axis makes extension into another axis model non-vacuous.

## Acceptance · `sec:assayer:intent-an-axis-nobody-reports-stays-at-its-prior-acceptance`

The implementing lane adds the two harness entries, adds one witness beside the existing axis lifecycle round trip (´test:integration:register-deregister-axis-round-trip´), and cites the promise from that test's documentation without minting a second claim.

Its report identifies the configured $\lambda_\text{prior}$ and $\kappa_{a,0}$ sentinels, the derived candidate width, the assessment count, the owner barriers crossed, and the exact and tolerance-based assertion groups.

Its evidence includes the fails-before result from the unmodified constructor, the passing registration and post-traffic prior checkpoints after repair, the passing post-retirement survivor diff, and the existing nearby lifecycle and ledger tests remaining green.

Acceptance also records the disposition of the axis-prior configuration ambiguity. If the registration schema gains an axis-specific prior, the test takes that field explicitly; otherwise it continues to demonstrate that compression scale and global prior precision are independent quantities.
