# Keeping the Restriction Gap Between Risk Estimates · `plan:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates`

Keeping (´claim:risk:restriction-opens-a-gap-between-operational-and-sister-estimates´) establishes that identical unconfounded traffic leaves the operational and sister estimates together, restriction plus investigated counterfactuals separates them in the positive direction, and a later regime change moves the faster-forgetting operational estimate first.

## What the promise says, precisely · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-promise`

The operational model estimates realised risk from every labelled outcome, while the sister estimates inherent risk from the unconfounded subset (´def:risk:model-triple´). The eligibility table makes `Allow` eligible, makes an ordinary `Block` ineligible for the sister, and lets the ground-truth flag override that block because investigation established the counterfactual outcome (´tab:eligibility:training´) and (´conv:eligibility:ground-truth´).

The intent's direct quantity is $D_s(\phi)=\hat{\rho}_\text{inh}(\phi)-\hat{\rho}_\text{opr}(\phi)$. The public assessment exposes both submodel probabilities under one $\kappa_\text{eff}$, so the witness recovers it as $D_s=\kappa_\text{eff}[\operatorname{logit}(p_\text{sister})-\operatorname{logit}(p_\text{operational})]$ from the risk basis (´schema:risk:basis´).

The specification gives the unqualified symbol $\Delta$ to a neighbouring quantity, $D_\text{eff}=\hat{\rho}_\text{eff}-\hat{\rho}_\text{opr}$, which the code exposes directly as `intervention_effectiveness` (´def:risk:intervention-effectiveness´). The witness names the direct sister gap and the blended intervention measure separately and observes both; it never treats one as an oracle for the other.

The unrestricted threshold is $|D_s| \leq \epsilon_0$ with $\epsilon_0=0.02$ on every final baseline checkpoint. This is a test-owned meaning of “near zero,” deliberately wider than floating-point equality because the models receive the same ordered labels under distinct forgetting factors.

The restricted threshold is $D_s \geq g_\text{min}$ with $g_\text{min}=0.10$, together with $D_\text{eff}>0$, at four consecutive checkpoints spanning the final two operational half-lives. The sister must also rank the high-risk probe above the low-risk control at every checkpoint, so a positive gap cannot be manufactured by both estimates collapsing below an untrained prior.

The configuration values are $\gamma_\text{opr}=0.9995$ and $\gamma_\text{inh}=0.9998$ (´tab:config:risk-model´). The corresponding label half-lives are $H_\text{opr}=\lceil\log(1/2)/\log(\gamma_\text{opr})\rceil$, about 1,400 labels, and $H_\text{inh}$, about 3,500 labels, as fixed by (´tab:risk:forgetting-rates´).

For the regime-change arm, let $M_m(k)=|\hat{\rho}_m(k)-\hat{\rho}_m(0)|$ on the changed-class probe after $k$ common eligible labels. At $k=H_\text{opr}$ both estimates must move in the new direction and $M_\text{opr}(k) \geq 1.5M_\text{inh}(k)$; the factor is a conservative behavioural cut below the roughly twofold separation implied by the two retained old-evidence fractions at that checkpoint.

The model update applies each model's configured label decay inside the same bounded posterior operation (´alg:runtime:update-path´), and the posterior record requires label and elapsed-time decay to form one factor per model per label (´dec:posterior:combined-factor´). The clock stays fixed during this witness, so only the rates named by the promise contribute to firstness.

## What the code offers today · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-current-code`

The scenario builder supplies configured seeded construction, and the world harness supplies `World`, `settle_cold_ramp_with`, public request and assessment verbs, `label`, `flush_labels`, and `cycle_request`. The observation and label barriers put each checkpoint after a known publication rather than after a sleep.

The scalar-signal fixture supplies the same score and verification schema used by the nearest directional convergence witness (´test:integration:scalar-signal-population-split-converges-directionally´). Its high and low request populations already demonstrate that the public assess-then-label path can learn a held-out ordering.

The label fixture can independently set `Action::Allow`, `Action::Block`, adverse or benign valence, and the ground-truth override. The nearest eligibility integration witness proves that a block-trained control is excluded where the configured challenge population is admitted (´test:integration:challenge-without-result-follows-eligibility-policy´).

The public risk assessment exposes `RiskAssessment.risk.p_bad_sister`, `p_bad_operational`, `kappa_eff`, and `intervention_effectiveness` as the risk basis specifies (´schema:risk:basis´). Existing crate tests keep the sub-probabilities finite after labels and keep the blended gap near zero at cold start, but neither creates a restriction regime nor checks sustained direction (´test:crate:p-bad-sister-and-operational-differ-after-labels´) and (´test:crate:intervention-effectiveness-formula´).

The public model configuration exposes both forgetting factors through `AssayerConfig.model`, with the promised defaults fixed by the core risk parameter table (´tab:config:risk-model´). Configured construction therefore lets the test assert the defaults it is about before it uses them to derive the two half-life checkpoints.

Label submission is asynchronous by contract (´dec:surface:async-label´), while the model and health publications occur at the end of the label path (´dec:ordering:publish-at-end´). A long-stream helper must batch submissions within the configured capacities and cross `flush_labels` before reading a checkpoint.

The cold standardisation ramp is an independent source of motion. `World::settle_cold_ramp_with` already removes it before paired streams are compared, following the evidence-authority invariant that an accepted assessment observation may move a later coordinate state (´inv:guarantee:evidence-authority´).

The standing testing plan assigns this promise directly to the open stage-three tape and fixture work (´entry:assayer:harness-stage-tapes´). `LabelTape`, a converged-world fixture, and a gap-specific assertion do not exist today.

## The witness · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-witness`

The integration witness belongs beside the stationary public-API convergence smoke (´test:integration:scalar-signal-population-split-converges-directionally´). One labeled test contains a restriction arm and an independent forgetting arm so the eligibility split and the rate split cannot impersonate one another.

- Setup: build two seeded worlds with the scalar score schema, the default channel, the fixed virtual clock, and the exact default model rates; settle each cold ramp with the same balanced high-score and low-score requests.

- Shared baseline: replay high-score adverse `Allow` labels and low-score benign `Allow` labels in alternating order for four sister half-lives, sampling the last four checkpoints. Every label is eligible, both models see the same feature-target pairs, $|D_s|$ remains at or below $\epsilon_0$, and the high probe remains above the low probe.

- Restriction stimulus: continue one world in repeating five-label blocks containing three high-score benign `Block` labels without ground truth, one high-score adverse `Block` label with ground truth, and one low-score benign `Allow` label. The operational model sees all five outcomes; the sister sees the investigated high adverse and the allowed low benign outcomes.

- Restriction observation: read the high and low probes before the tape labels at each checkpoint, recover $D_s$ from the public sub-probabilities and `kappa_eff`, and read $D_\text{eff}$ from `intervention_effectiveness`. Every probability entering a logit must lie in $[\eta,1-\eta]$ for $\eta=10^{-6}$, turning saturation into a named failure rather than an infinite derived score.

- Restriction assertion: after the stream has supplied four sister half-lives of eligible evidence, retain four checkpoints separated by half an operational half-life; all four satisfy $D_s \geq 0.10$, $D_\text{eff}>0$, and the high-over-low sister ordering, while exceeding the unrestricted ceiling by at least $0.08$.

- Forgetting setup: the second world finishes the same unrestricted baseline, records both raw high-probe scores, then flips only the association between the two fixed request populations: high becomes benign and low becomes adverse, with every action still `Allow` and every label therefore common to both models.

- Forgetting observation: sample the high probe at one quarter, one half, and one operational half-life after the flip. Both raw scores move downward at every checkpoint, and at the final checkpoint the operational movement exceeds 1.5 times the sister movement; the low probe moves upward as the sign-control.

- Health control: every checkpoint reports finite risk values and no numerical degradation, the label path remains live, and the tape records submitted, eligible, investigated, blocked, and flushed counts so the causal population is visible in a failure.

The fails-before is specific. Admitting ordinary blocked outcomes to the sister makes both models learn the suppressed realised population and collapses the restricted gap; ignoring the ground-truth override starves the sister of the adverse high-risk evidence and loses its high-over-low ordering; excluding blocks from the operational model collapses the gap in the other direction; equalising or swapping the two configured rates loses the 1.5 movement lead.

## What is missing · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-missing`

**Entry (A bounded public label tape)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-bounded-label-tape`

A `LabelTape` fixture represents request builders, action, valence, investigation provenance, repetition, and checkpoint cadence as data, submits bounded assessment and label batches, flushes once per chunk, and returns the pre-update risk bases selected by the schedule. Estimate: approximately 100–150 lines within the testing harness plus focused helper tests; it is the narrow slice of (´entry:assayer:harness-stage-tapes´) this promise requires and has no sibling-lane dependency.

**Entry (A safe direct-gap probe)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-direct-gap-probe`

A domain assertion checks both sub-probabilities against the interior guard, applies a stable logit, multiplies their difference by the assessment's shared `kappa_eff`, and prints the direct and blended gaps together. Estimate: approximately 30–50 lines among the domain-aware assertion helpers with boundary tests; it depends only on the existing public `RiskBasis` fields.

**Entry (A stationary two-population fixture)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-stationary-population-fixture`

A fixture supplies the fixed high and low signal requests, settles the cold ramp, derives half-life counts from the asserted configuration, and emits the unrestricted, restricted, and flipped schedules without random class placement. Estimate: approximately 60–90 lines beside the tape; it depends on the bounded-label-tape entry and is the promise-specific converged fixture within the standing stage-three work.

**Entry (The restriction-gap integration witness)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-integration-witness`

The test extends the stationary public-API convergence witness (´test:integration:scalar-signal-population-split-converges-directionally´) with its own module index row, named thresholds, progress trace, health controls, and deliberate-defect evidence. Estimate: approximately 110–160 lines; it depends on the three entries above and on no production API change.

## Risks and open questions · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-risks`

**Observation (The intent and specification name different gaps)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-two-gap-definitions`

The intent defines $\Delta$ with the sister estimate, while the specification defines and publishes it with the blended effective estimate. Keeping both readings makes the behavioural witness unambiguous without blocking on terminology; a later corpus decision can rename the direct intent quantity or revise the intent to the blended quantity without changing the authored population.

**Observation (Logit inversion needs an interior guard)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-logit-interior`

The public surface carries submodel probabilities rather than both raw submodel scores. Recovering the promised raw gap is exact under their shared calibration factor until a probability saturates numerically, so the interior assertion is part of the witness and the stimulus remains away from extreme scores.

**Observation (Standardisation can look like model movement)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-standardisation-control`

The cold ramp advances from assessment observations and publishes asynchronously. Settling it before the baseline, holding the feature distribution fixed across phases, and reading only after barriers prevents coordinate movement from masquerading as restriction or faster forgetting.

**Observation (Importance weighting moves with eligibility)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-weighting-control`

The global and eligible class-rate trackers see different populations and decay at their respective model rates. The repeating restriction block keeps the sister's admitted population balanced and deterministic, while the separate all-eligible forgetting arm isolates the rate ordering from this necessary consequence of restriction.

**Observation (The magnitude cuts are test-owned)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-test-owned-cuts`

Neither the promise nor the specification fixes a numeric meaning for “near zero,” “opens,” or “stays.” The 0.02 ceiling, 0.10 floor, four-checkpoint persistence window, and 1.5 movement ratio are falsifiable fixture thresholds rather than system constants; the implementation report carries their measured extrema and margin, and a cut with less than twofold observed headroom returns to fixture design rather than widening silently.

**Observation (A long tape needs progress, not a runtime assertion)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-progress-not-latency`

Several sister half-lives make this materially longer than the current directional smoke. The existing liveness watcher and label barrier turn a stalled owner into a diagnostic failure, while batching limits ceremony; elapsed runtime remains verification evidence and never becomes an assertion about statistical behaviour.

## Acceptance · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-acceptance`

The implementation report names the integration test and its cited intent, shows the asserted default rates and derived half-lives, and records the exact tape populations, eligible counts, blocked counts, investigated counts, checkpoint spacing, and fixed seed.

It reports all unrestricted $D_s$ readings, all four restricted $D_s$ and $D_\text{eff}$ readings, both probe orderings, and the minimum observed margin against the 0.02, 0.10, and 0.08 cuts.

It reports the pre-flip and three post-flip raw submodel scores, $M_\text{opr}$, $M_\text{inh}$, their ratio at $H_\text{opr}$, and the opposite movement of the low-score control.

It shows every checkpoint crossing a completed label barrier, the cold ramp settled before measured training, the probability interior guards holding, the health controls clean, and the liveness progress reaching completion without a sleep or wall-clock oracle.

It includes the four deliberate defects and the assertion each one breaks, then shows the focused integration test and the Assayer package verification green in debug and release with the corpus linter resolving every label.
