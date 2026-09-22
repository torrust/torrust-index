# Keeping the Restriction Gap Between Risk Estimates · `plan:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates`

Keeping (´claim:risk:restriction-opens-a-gap-between-operational-and-sister-estimates´) establishes that identical unconfounded traffic leaves the operational and sister estimates together, restriction plus investigated counterfactuals separates them in the positive direction, and a later regime change moves the faster-forgetting operational estimate first.

## What the promise says, precisely · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-promise`

The operational model estimates realised risk from every labelled outcome, while the sister estimates inherent risk from the unconfounded subset (´def:risk:model-triple´). The eligibility table makes `Action::Allow` eligible, makes an ordinary `Action::Block` ineligible for the sister, and lets `LabelSpec::ground_truth` override that block because investigation established the counterfactual outcome (´tab:eligibility:training´) (´conv:eligibility:ground-truth´).

The intent's direct quantity is $D_s(\phi)=\hat{\rho}_\text{inh}(\phi)-\hat{\rho}_\text{opr}(\phi)$. The implementation exposes `RiskBasis::p_bad_sister` and `RiskBasis::p_bad_operational` under the assessment's shared `RiskBasis::kappa_eff`, so the witness recovers the two raw scores and their difference as $D_s=\kappa_\text{eff}[\operatorname{logit}(p_\text{sister})-\operatorname{logit}(p_\text{operational})]$; the current API test establishes that these fields are public and finite, not that the two models have separated (´test:crate:risk-basis-has-14-fields´).

The specification gives the unqualified symbol $\Delta$ to $D_\text{eff}=\hat{\rho}_\text{eff}-\hat{\rho}_\text{opr}$ and requires that quantity on the risk basis (´def:risk:intervention-effectiveness´) (´schema:risk:basis´). The implementation exposes it as `RiskBasis::intervention_effectiveness`; the witness observes it beside $D_s$ and never substitutes one gap for the other.

Neither the claim nor the specification fixes numeric bands for “near zero,” “opens,” or “stays.” The witness therefore owns an unrestricted ceiling $\epsilon_0=0.02$ for four final baseline checkpoints and a restricted floor $g_\text{min}=0.10$, together with $D_\text{eff}>0$, high-over-low sister ordering, and four consecutive restricted checkpoints; each cut must retain measured headroom rather than be fitted after the run.

The configured values are $\gamma_\text{opr}=0.9995$ and $\gamma_\text{inh}=0.9998$ (´tab:config:risk-model´). Their label half-lives are $H_\text{opr}=\lceil\log(1/2)/\log(\gamma_\text{opr})\rceil$, about 1,400 labels, and $H_\text{inh}=\lceil\log(1/2)/\log(\gamma_\text{inh})\rceil$, about 3,500 labels, matching the specification's forgetting table (´tab:risk:forgetting-rates´).

For the regime-change arm, let $M_m(k)=|\hat{\rho}_m(k)-\hat{\rho}_m(0)|$ on the changed high-score probe after $k$ common eligible labels. At one quarter, one half, and one operational half-life, both estimates must move in the new direction and $M_\text{opr}(k)>M_\text{inh}(k)$; at $k=H_\text{opr}$ the test-owned stronger cut is $M_\text{opr}(k)\geq1.5M_\text{inh}(k)$. The specification supports only the analytical ordering of retained evidence through $\gamma^k$, not that behavioural ratio (´def:temporal:two-mechanisms´) (´tab:risk:forgetting-rates´).

The model update applies each updated model's configured label decay together with elapsed-time decay as one factor per label (´alg:runtime:update-path´) (´dec:posterior:combined-factor´). Scenario time remains fixed throughout this witness, so the time-indexed factor is unity and cannot supply the observed firstness.

## What the code offers today · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-current-code`

The finished harness exposes one `World` vocabulary with `World::assess`, `World::label`, `World::flush_labels`, `World::flush_observations`, `World::advance`, and `World::travel_to`; `score_verified_request_schema` and `with_score_verified` supply the two fixed signal populations, and `LabelSpec` supplies action, valence, and ground-truth provenance (´tab:assayer:harness-implementation-library-roster´). The witness does not move time, and every asynchronous label reading follows `World::flush_labels`, so it needs neither a raw clock nor a local wait (´dec:harness:no-ad-hoc-waits´).

`World::trained_state` now returns a `TrainedStateFixture` only after its `TrainedStateBaseline` reports balanced training classes, held-out class counts, endpoint separation, and pairwise rank. That guarded fixture retains the scalar population already used by the directional convergence test and keeps setup validity separate from this promise's result assertions (´entry:assayer:harness-guarded-fixtures´) (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´) (´test:integration:scalar-signal-population-split-converges-directionally´).

The shared runner accepts a subject-owned `PlaybackRow`, applies its `PlaybackBarrierPolicy`, invokes each `PlaybackCheckpoint` after the selected barriers, and advances `PlaybackProgress` at the complete row boundary. `PlaybackBatchSize` bounds materialisation cost only: every row still settles separately, and a label policy uses `PlaybackBarrier::FlushLabels` (´entry:assayer:harness-tape-runner´) (´dec:harness:declarative-playback´) (´test:unit:held-barrier-completes-before-its-checkpoint´) (´test:unit:batch-size-preserves-mid-batch-publication-boundaries´).

The shared oracle tier supplies `decay_recurrence` with specification-formula provenance. It can derive retained old-evidence fractions at every forgetting checkpoint without reusing the production power route, but it cannot predict the fitted model-score movement and therefore does not justify the test-owned 1.5 cut (´entry:assayer:harness-oracle-tier´) (´dec:harness:oracle-tier´) (´test:crate:decay-oracle-applies-each-clock-as-a-recurrence´).

Existing tests establish the ingredients without keeping this promise. `steps4_8_eligibility_block_no_ground_truth` and `steps4_8_ground_truth_overrides` distinguish the two tracker populations, while `ground_truth_flag_overrides_block_eligibility` reaches the override through the public surface (´test:crate:steps4-8-eligibility-block-no-ground-truth´) (´test:crate:steps4-8-ground-truth-overrides´) (´test:integration:ground-truth-flag-overrides-block-eligibility´). `p_bad_sister_and_operational_differ_after_labels` asserts only that the published sub-probabilities remain finite and in range after labels, and `intervention_effectiveness_formula` asserts only a finite cold-start value below its loose ceiling; neither creates a restriction regime, asserts a sustained positive gap, or compares adaptation rates (´test:crate:p-bad-sister-and-operational-differ-after-labels´) (´test:crate:intervention-effectiveness-formula´).

The default configuration test pins both promised rates, and `Assayer::config` lets the witness read them back from each constructed world before deriving checkpoints (´test:crate:config-default-compiles´) (´tab:config:risk-model´). No harness capability required by the witness remains pending.

## The witness · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-witness`

The integration target `learning_convergence` gains `learning_convergence::restriction_opens_a_gap_between_operational_and_sister_estimates` beside the stationary public-API convergence smoke. One test owns an independently constructed restriction arm and forgetting arm so the eligibility split and rate split cannot impersonate one another.

- Setup: call `World::trained_state` twice with the fixed seed and distinct instance names, retain each returned `TrainedStateBaseline` as setup evidence, assert the two rates through `Assayer::config`, derive both half-life counts, and use `decay_recurrence` with zero elapsed hours to record the expected retained-evidence ordering. The virtual clock is not advanced.

- Playback: define one test-local row type implementing `PlaybackRow`; each row builds one score-verified request, calls `World::assess`, submits one `LabelSpec` through `World::label`, and chooses no wait. Drive every long phase through `playback` with `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels])`, named `PlaybackCheckpoint` callbacks, `PlaybackProgress`, and a bounded `PlaybackBatchSize`; the runner, not the row, owns publication completion (´dec:harness:declarative-playback´).

- Shared baseline: feed each world alternating high-score adverse `Action::Allow` and low-score benign `Action::Allow` rows for four sister half-lives, then retain four checkpoints spanning the final two operational half-lives. Every row is eligible for both models; each checkpoint requires $|D_s|\leq0.02$ and high-over-low sister ordering.

- Restriction stimulus: continue the restriction world in repeating five-row blocks containing three high-score benign `Action::Block` labels without ground truth, one high-score adverse `Action::Block` label with ground truth, and one low-score benign `Action::Allow` label. The operational model receives all five outcomes, while the sister receives the investigated high adverse and allowed low benign outcomes.

- Restriction observation: after four sister half-lives of eligible evidence, retain four settled checkpoints separated by half an operational half-life. At each checkpoint call `World::assess` for the fixed high and low probes, read the owned `RiskAssessment::risk`, recover both raw scores and $D_s$ with `stable_logit`, and read $D_\text{eff}$ from `RiskBasis::intervention_effectiveness`; require each inverted probability to lie in $[10^{-6},1-10^{-6}]$ before applying the logit.

- Restriction assertion: every retained checkpoint satisfies $D_s\geq0.10$, $D_\text{eff}>0$, and high-over-low sister ordering, and its direct gap exceeds the unrestricted ceiling by at least $0.08$.

- Forgetting stimulus: after the second world's shared baseline, record the two raw high-probe scores, then flip only the association between the fixed populations: high becomes benign and low becomes adverse, every action remains `Action::Allow`, and every label remains common to both models.

- Forgetting observation: retain settled high- and low-probe readings after one quarter, one half, and one operational half-life of common labels. At each checkpoint both high-probe raw scores move downward, the operational movement exceeds the sister movement, and the low probe moves upward; at the final checkpoint the operational-to-sister movement ratio is at least 1.5. Compare those behavioural readings with the separately reported `decay_recurrence` retained-evidence fractions without treating the oracle as a model-score predictor (´dec:harness:oracle-tier´).

- Health and liveness: every checkpoint reports finite risk values, holds the probability interior guard, and leaves health clean. The row ledger reports submitted, eligible, investigated, blocked, and flushed counts, while `PlaybackProgress` reaches completion and names the first incomplete row on failure without creating a runtime assertion (´cav:harness:progress-not-latency´).

The fails-before is specific. Admitting ordinary blocked outcomes to the sister collapses the restricted gap; ignoring the ground-truth override removes the sister's adverse high-risk evidence and loses its high-over-low ordering; excluding blocks from the operational model collapses the gap in the other direction; equalising or swapping the configured rates loses the operational movement lead.

## What is missing · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-missing`

**Entry (A bounded public label tape)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-bounded-label-tape`

The `learning_convergence` target still needs its subject-owned restriction row and deterministic schedule constructors. The row implements `PlaybackRow` over one request and one label, binds the live assessment identifier, and returns without crossing a barrier; `playback` supplies the existing barrier, checkpoint, progress, and bounded-materialisation semantics. No new shared runner or repetition field is required (´entry:assayer:harness-tape-runner´).

**Entry (A safe direct-gap probe)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-direct-gap-probe`

The same target needs a one-caller `direct_sister_gap` helper over `RiskBasis`. It checks `RiskBasis::p_bad_sister` and `RiskBasis::p_bad_operational` against the fixed interior guard, applies `stable_logit`, multiplies by the same reading's `RiskBasis::kappa_eff`, and reports both $D_s$ and `RiskBasis::intervention_effectiveness` on failure. This remains a local assertion helper rather than a shared oracle because it has one promised caller (´dec:harness:oracle-tier´).

**Entry (A stationary two-population fixture)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-stationary-population-fixture`

The target needs a thin promise-owned setup function that consumes `World::trained_state`, verifies the returned world's configured rates, derives half-life checkpoints, and emits the unrestricted, restricted, and flipped row schedules. The trained state and its guard have landed; this function must not duplicate them or turn the baseline guard into the result oracle (´entry:assayer:harness-guarded-fixtures´) (´dec:harness:separate-validation´).

**Entry (The restriction-gap integration witness)** · `entry:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-integration-witness`

The missing `learning_convergence::restriction_opens_a_gap_between_operational_and_sister_estimates` test composes those local pieces with the landed harness. It owns the numeric cuts, checkpoint ledger, health controls, and deliberate-defect evidence; it needs no production API or shared-harness change.

## Risks and open questions · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-risks`

**Observation (The intent and specification name different gaps)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-two-gap-definitions`

The claim defines $\Delta$ with the sister estimate, while the specification defines and publishes $\Delta$ with the blended effective estimate. The witness preserves the claim as $D_s$, preserves the specification's quantity as $D_\text{eff}$, and reports both; neither is an oracle for the other (´def:risk:intervention-effectiveness´).

**Observation (Logit inversion needs an interior guard)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-logit-interior`

The public implementation carries the sister and operational probabilities rather than both corresponding raw scores. Their inverse is exact under the reading's shared `RiskBasis::kappa_eff` only inside the open unit interval, so the fixed interior guard turns saturation into an explicit failure instead of an infinite score.

**Observation (Standardisation can look like model movement)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-standardisation-control`

`World::trained_state` settles the cold ramp before returning, but `PlaybackBarrier::FlushLabels` does not cover observation work. The witness therefore begins only from the guarded trained fixture, keeps the feature distribution fixed across measured phases, and does not infer observation completion from the label barrier (´dec:harness:guarded-fixtures´) (´dec:harness:no-ad-hoc-waits´).

**Observation (Importance weighting moves with eligibility)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-weighting-control`

The global and eligible class-rate trackers see different populations and decay at their respective model rates (´tab:weighting:trackers´). The deterministic restriction block keeps the sister's admitted population balanced, while the independent all-eligible forgetting arm isolates the rate ordering from this necessary consequence of restriction.

**Observation (The magnitude cuts are test-owned)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-test-owned-cuts`

The 0.02 ceiling, 0.10 floor, four-checkpoint persistence window, 0.08 separation margin, probability interior guard, and 1.5 movement ratio are fixture thresholds, not specification constants. Acceptance records every measured extremum and margin; a threshold without clear headroom returns to fixture design instead of being widened silently, and `decay_recurrence` is reported only as evidence-retention context.

**Observation (A long tape needs progress, not a runtime assertion)** · `obs:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-progress-not-latency`

Several sister half-lives make this materially longer than the directional smoke. `PlaybackProgress` and the selected label barrier turn a stopped owner into a row-specific failure, while `PlaybackBatchSize` limits materialisation without changing visibility; elapsed runtime remains verification evidence and never becomes a statistical assertion (´cav:harness:progress-not-latency´) (´test:unit:batch-size-preserves-mid-batch-publication-boundaries´).

## Acceptance · `sec:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates-acceptance`

The implementation report names the integration target `learning_convergence` and function `learning_convergence::restriction_opens_a_gap_between_operational_and_sister_estimates`, cites this intent, records the fixed seed, reports the guarded `TrainedStateBaseline` for both arms, and shows the configured rates, derived half-lives, and `decay_recurrence` retained-evidence fractions.

It reports every unrestricted and restricted checkpoint's two recovered raw scores, $D_s$, $D_\text{eff}$, probability-interior margin, and high-over-low sister ordering, together with the minimum margins against the 0.02, 0.10, and 0.08 cuts.

It reports the pre-flip and three post-flip raw scores for both models and both probes, $M_\text{opr}$, $M_\text{inh}$, their ordering at every checkpoint, their ratio at $H_\text{opr}$, and the retained-evidence values supplied separately by `decay_recurrence`.

It shows that each playback checkpoint followed `PlaybackBarrier::FlushLabels`, the trained fixture settled the cold ramp before measured playback, `PlaybackProgress` completed every row, the count ledger matches the declared populations, and health remained clean without a sleep, poll, local deadline, or wall-clock oracle.

It includes the four deliberate defects and the assertion each breaks, then shows the focused integration test and Assayer package verification green with the corpus linter resolving every label.
