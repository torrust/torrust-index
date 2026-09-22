# A stale adverse history dissolves on the clock · `plan:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels`

Keeping (´claim:ledger:a-stale-adverse-history-dissolves-without-fresh-labels´) establishes that a cell's adverse outcome memory and its resulting excess risk follow the configured elapsed-time decay after the live Sentinel report becomes neutral, even when no further label reaches the region.

## What the promise says, precisely · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-promise`

The stale state is the Ledger entry's adverse-rate, compressed-valence, and raw-valence averages, the three base features extracted from outcome memory rather than from the current report (´tab:extraction:ledger-features´). A burst writes all three at every containing layer, while a read selects the deepest entry covering the request coordinate (´alg:ledger:all-layers-update´) (´dec:memory:depth-walk´).

Write the configured hourly retention as $\gamma_{t,L}$ and elapsed hours after the last label as $h$. With no later label, each stored Ledger feature contributes its value at the cutoff multiplied by $D(h)=\gamma_{t,L}^{h}$; read-time evaluation is pure and does not advance the stored timestamp (´def:ledger:time-decay´) (´alg:temporal:lazy-application´).

The shipped configuration fixes $\gamma_{t,L}=0.999$ per hour (´tab:config:temporal´). Its half-life is about twenty-nine days, so the analytic retained fractions are about one half, one quarter, and one eighth after twenty-nine, fifty-eight, and eighty-seven days; under complete label starvation those factors are the only motion in the Ledger (´thm:temporal:ledger-decay-bound´).

The measurable baseline is a sibling cell in the same current report with identical neutral measurement fields and no outcome history. The adverse and baseline assessments therefore share report staleness, batch features, aggregate features, standardisation state, model parameters, and anchor inputs; only the three Ledger positions in the per-Sentinel slot differ (´alg:runtime:extraction-routing´) (´tab:feature:aggregate-block´).

Let $\Delta\rho(h)=\rho_{\mathrm{eff,adverse}}(h)-\rho_{\mathrm{eff,baseline}}(h)$. The anchor projection contains no per-Sentinel slot, and the blend weight is computed only over the anchor's shared subspace (´def:risk:anchor-model´) (´def:risk:subspace-blend´), so matched current inputs make the anchor score, blend weight, and calibration parameter equal across the pair and give $\Delta\rho(h)=D(h)\Delta\rho(0)$.

The setup requires $\Delta\rho(0)>100\epsilon_{\mathrm{default}}$ so the curve cannot pass vacuously, and every milestone compares the observed raw-risk gap with the specification-derived expectation within the harness's generic one-shot tolerance (´tab:assayer:harness-scenario-tolerances´). The public probability $p_{\mathrm{bad}}=\sigma(\rho_{\mathrm{eff}}/\kappa_{\mathrm{eff}})$ need not inherit an exponential ratio because the sigmoid is nonlinear; it remains above the matched baseline while the positive raw gap remains (´def:risk:probability´).

The absence of fresh labels begins after the history cutoff. Report refreshes and assessments may continue, but the snapshot version and both calibration-record counts remain fixed, distinguishing clock-only recovery from hidden training; the guarantee is the bounded dissolution of stale influence, not the acquisition of contrary evidence (´inv:guarantee:ledger-floor´) (´alg:valence:contamination-loop´).

## What the code offers today · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-code-today`

`LedgerEntry::read_decayed` computes one `decay_factor_since` value and multiplies every stored base and per-axis average without mutating the entry, and extraction passes that owned view into the Ledger feature positions selected by deepest-covering-entry routing (´def:ledger:time-decay´) (´alg:runtime:extraction-routing´).

The assessment path reads persistent and monotonic time from the injected clock, passes `config.temporal.gamma_t_ledger` into extraction, and returns owned `RiskAssessment` readings containing `RiskBasis::rho_eff`, `RiskBasis::p_bad`, `RiskBasis::anchor_weight`, `RiskBasis::kappa_eff`, both calibration-record counts, and the health snapshot's publication version (´schema:output:assessment´) (´dec:clock:two-domains´) (´dec:clock:shared-functions´).

The finished harness exposes one real-engine `World`; `World::advance` and `World::travel_to` move both clock domains forward and settle identity maintenance plus one Ledger collection cycle, while `World::flush_observations`, `World::flush_labels`, and `World::flush_identity_maintenance` remain the queue-specific barriers (´dec:harness:single-scenario´) (´entry:assayer:harness-scenario-time´) (´entry:assayer:harness-closed-barriers´) (´tab:assayer:harness-implementation-library-roster´).

Long histories now use a subject-owned `PlaybackRow` sequence interpreted by `playback` under a `PlaybackBarrierPolicy`; `decay_recurrence` supplies the independent specification-formula route for the retained fraction; and `World::trained_state` returns a `TrainedStateFixture` with its measured `TrainedStateBaseline` only after its general trained-state preconditions hold (´dec:harness:declarative-playback´) (´dec:harness:oracle-tier´) (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´entry:assayer:harness-tape-runner´) (´entry:assayer:harness-oracle-tier´) (´entry:assayer:harness-guarded-fixtures´).

The owned `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` probes obey the finished single-load projection contract, but intentionally expose no Ledger entry or extracted Ledger feature. This witness therefore reads the public assessment and composes it with the direct Ledger mechanism tests instead of widening the probe surface (´dec:harness:probe-contract´) (´tab:assayer:harness-implementation-library-roster´).

The report fixtures retain `make_cell_report`, so the witness can author three structurally identical sibling cells without a new engine or harness primitive; authored reports remain stimulus rather than recorded output (´dec:assayer:golden-report-stimulus´). `run_seeded_sweep` is available for quantified generated cases, but this promise fixes one configured decay schedule and therefore needs no seed sweep (´dec:harness:seeded-sweeps´).

The direct mechanism witness drives fifty adverse updates into a `LedgerEntry` and proves its bad-rate view follows the day-zero through day-eighty-seven analytic timeline without mutating storage (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´). The numerical half-life witness proves the configured hourly factor (´test:integration:decay-factor-29-day-half-life´), the extraction witness proves deepest-cell selection (´test:unit:ledger-read-takes-the-deepest-covering-entry´), the scenario-time witness proves an advance moves both clock domains and crosses the Ledger-cycle barrier (´test:crate:scenario-advance-completes-the-ledger-gc-cycle-it-makes-due´), and the travelled-entry witness proves a later neutral Ledger root takes the scenario's persistent present (´test:crate:neutral-ledger-state-takes-travelled-scenario-time´).

No test mints or cites this intent's claim. The existing public Ledger lifetime witness exercises sustained assessment routing (´test:integration:root-survives-many-assessments-within-lifetime´), but neither it nor the mechanism tests compose a label-starved cell, neutral live reports, forward-only scenario time, and the public risk basis.

## The witness · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-witness`

The witness belongs in the `outcome_ledger` integration target as `outcome_ledger::stale_adverse_history_dissolves_without_fresh_labels`, beside the public Ledger lifetime scenario represented by (´test:integration:root-survives-many-assessments-within-lifetime´); its module index and test documentation cite the intent and the claim.

- Setup: call `fail_fast_on_model_owner_panic`, obtain a seeded `TrainedStateFixture` through `World::trained_state`, retain its `TrainedStateBaseline` for failure diagnostics, register one Sentinel, ingest a test-local report containing three structurally identical sibling cells beneath one common ancestor, and use `World::published_slot_moments` to require that the newly published slot belongs to the fixture's in-service standardisation state before Ledger-specific training begins (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´dec:harness:probe-contract´).

- Training stimulus: define a test-local row implementing `PlaybackRow`, alternate the nearest whole-row form of the specification's roughly six-hundred-and-ninety-three-label Ledger half-life of adverse rows through the first cell with the same number of benign rows through the second, and run those rows through `playback` with `PlaybackBarrierPolicy::new([PlaybackBarrier::FlushLabels])`, an empty `PlaybackCheckpoint` slice, a `PlaybackProgress` reading, and a bounded `PlaybackBatchSize`. Every request carries the same neutral live measurements and signal values, so the balanced outcomes keep the population mix symmetric while the two leaf histories become the discriminating input; the third cell receives no label and remains the neutral baseline (´tab:ledger:entry-state´) (´dec:harness:declarative-playback´).

- Neutralisation: after playback's final label barrier, ingest the same report with identical neutral current measurements at all three cells and declare that instant $h=0$. No label, assessment-label cycle, or pre-seed operation occurs after this cutoff.

- Guarded initial observation: derive the adverse and baseline requests in one `World::derive_for_requests` batch, require equal `RiskBasis::anchor_weight` and `RiskBasis::kappa_eff`, require $\Delta\rho(0)>100\epsilon_{\mathrm{default}}$ using `World::tol().default`, require adverse `RiskBasis::p_bad` to exceed baseline, and record both risk bases, the health snapshot's publication version, and both calibration-record counts. Report every measured value if this Ledger-specific setup guard fails rather than treating setup failure as a decay result (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

- Clock stimulus: call `World::advance` for successive intervals whose cumulative positions are seven, fourteen, twenty-nine, fifty-eight, eighty-seven, and two hundred and ninety days. After each advance, ingest the unchanged neutral report before taking the paired assessment; the refresh holds report staleness at zero and recreates the untouched neutral leaf if the Ledger cycle included in the time verb collected it, without supplying outcome evidence (´entry:assayer:harness-scenario-time´).

- Analytic observation: obtain $D(h)$ from `decay_recurrence(1.0, 1.0, 0, gamma_t_ledger, elapsed_hours)`, not from the production exponentiation route. Because the blend definition says the common `RiskBasis::anchor_weight` may move with elapsed time, compare the observed public gap with $D(h)\Delta\rho(0)[1-w(h)]/[1-w(0)]$ within `World::tol().default`; the unresolved disagreement with the simpler equation in the precise promise remains the first open question below (´def:risk:subspace-blend´) (´dec:harness:oracle-tier´) (´test:crate:decay-oracle-applies-each-clock-as-a-recurrence´).

- Recovery observation: at every checkpoint require the adverse raw-risk gap to remain positive and to agree with the adopted analytic expression, require adverse `RiskBasis::p_bad` to remain above its same-checkpoint baseline, and show that the two-hundred-and-ninety-day Ledger retention factor from `decay_recurrence` is below one thousandth. Do not require probability differences or the baseline assessment itself to follow the Ledger ratio: the sigmoid is nonlinear, and the blend and uncertainty continue to read elapsed time.

- Isolation observation: at every checkpoint require the pair's `RiskBasis::anchor_weight` and `RiskBasis::kappa_eff` to agree, require the publication version and both calibration-record counts to equal their cutoff values, repeat one paired read without moving the clock and require both raw gaps to agree within `World::tol().default`, and finish with `assert_health_clean`.

The fails-before is an implementation that consults the wall clock, omits read-time decay, waits for the next label to apply decay, uses the core model's slower hourly rate, decays only part of the Ledger feature triplet, persists decay on a read, or ignores the blend factor the specification says can move with elapsed time. Such an implementation leaves the public gap flat, follows the wrong analytic expression, or changes on the repeated same-instant read even though the component controls may remain green.

## What is missing · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-missing`

**Entry (World owns the persistent-time verbs)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-world-time-verbs`

Add the time-driven checkpoint sequence to `outcome_ledger::stale_adverse_history_dissolves_without_fresh_labels`: cumulative milestones are reached only through `World::advance`, and each return already includes identity-maintenance publication and one Ledger collection cycle. No time verb or barrier remains to be added to the harness (´entry:assayer:harness-scenario-time´) (´entry:assayer:harness-closed-barriers´).

**Entry (A controlled three-cell report fixture)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-three-cell-report-fixture`

Add a test-local report builder in the `outcome_ledger` integration target that composes `make_cell_report` into one root, one shared ancestor, and three sibling competitive cells with identical neutral score arrays. The helper returns authored input only, uses distinct coordinates for the adverse, benign-training, and untouched baseline leaves, and needs no production or shared-harness change (´dec:assayer:golden-report-stimulus´).

**Entry (The clock-only recovery integration witness)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-recovery-witness`

Add `outcome_ledger::stale_adverse_history_dissolves_without_fresh_labels` with its module-index row, claim citation, subject-owned playback row, measured setup guard, cutoff, virtual-time milestones, oracle-derived retained fractions, public risk comparison, no-training checks, repeated-read purity check, and clean-health assertion. It needs no Ledger-state accessor, new probe, persistence fixture, random sampling, drift-budget widening, or wall-clock deadline; acceptance of its raw-gap oracle does require resolution of the specification conflict recorded below.

## Risks and open questions · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-risks`

**Observation (The raw gap carries the analytic curve)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-raw-gap-oracle`

The precise-promise equation omits a factor that its cited blend definition requires: that definition says the blend weight moves with elapsed time, while the matched-cell construction proves only that the two cells have equal weights at the same checkpoint. On that specification, the public raw gap is $D(h)\Delta\rho(0)[1-w(h)]/[1-w(0)]$, not $D(h)\Delta\rho(0)$; the latter becomes valid only if the blend weight is clock-invariant. The specification must choose between those statements before the raw-gap acceptance oracle can be final (´def:risk:subspace-blend´).

**Observation (The fixture proves its effect before timing it)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-effect-liveness`

`TrainedStateFixture` guards a general score-verified model state, not a learned coefficient on the new Sentinel's Ledger positions. The playback history therefore has its own measured cutoff guard: the initial raw gap, equal same-checkpoint blend inputs, class balance, publication completion, and health are reported before time moves; weakening that guard or widening the result tolerance cannot repair a fixture that failed to establish Ledger influence (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

**Observation (Neutral report refresh is not fresh outcome evidence)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-report-refresh-is-not-a-label`

`World::advance` deliberately completes one Ledger collection cycle, so the untouched zero-valued leaf can disappear after the collection horizon. Refreshing the unchanged neutral report after the advance recreates that leaf before the paired read, resets report staleness, and keeps current measurement surfaces equal; it neither writes a Ledger average nor trains a model, and the fixed publication version and calibration counts make that separation observable (´entry:assayer:harness-scenario-time´) (´alg:ledger:entry-creation´).

**Observation (Baseline is a limit with a controlled finite proxy)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-baseline-is-a-limit`

The specification gives the Ledger's exponential curve and no finite absolute `RiskBasis::p_bad` distance called recovered. The same-checkpoint untouched cell is the public baseline, the direct Ledger timeline is the mechanism control, and the oracle-derived final retention factor is below one thousandth without inventing a product threshold. The finished probe boundary exposes no Ledger view, so an integration test must not add private Ledger access merely to duplicate the crate-level mechanism witness (´dec:harness:probe-contract´) (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´).

**Observation (The asynchronous surfaces are held still)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-determinism`

Cold-ramp settlement occurs before training, playback crosses `PlaybackBarrier::FlushLabels` after every row, paired observations use one batch time, no identity dimension is registered, and `World::advance` replaces sleeping and polling. `fail_fast_on_model_owner_panic` is failure containment rather than a timing oracle, and neither playback progress nor a drift budget is evidence for departure from the analytic comparison (´dec:harness:no-ad-hoc-waits´) (´dec:harness:declarative-playback´).

The harness has no remaining implementation blocker for this witness. The open issue is the specification's treatment of elapsed time in the blend weight, which determines the final public-gap equation but does not alter the Ledger decay curve or the directional recovery observation.

## Acceptance · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-acceptance`

The implementing work adds `outcome_ledger::stale_adverse_history_dissolves_without_fresh_labels` to the `outcome_ledger` integration target, adds its module-index row and claim citation, and lands the three entries above without changing the shared harness.

Before the witness's public raw-gap assertion is accepted, the specification conflict in (´obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-raw-gap-oracle´) is resolved and the test uses the resulting equation. The result tolerance is `World::tol().default`; the Ledger retention factors come only from `decay_recurrence`, and no fitted tolerance or production exponentiation supplies an expected value (´dec:harness:oracle-tier´).

The test output reports the guarded cutoff baseline; observed and expected public raw gaps at every milestone; the adverse and baseline `RiskBasis::p_bad` trajectory; same-checkpoint blend weight and calibration parameter; stable publication version and calibration-record counts; the final retained fraction; repeated-read purity; playback completion; and clean health.

The coverage report resolves the intent and claim to the new integration witness, while the direct Ledger timeline, numerical half-life, deepest-routing, scenario-time, playback-barrier, and decay-oracle witnesses remain green as independent mechanism controls.

The report states which analytic or isolation assertion rejects each deliberately broken behaviour described above. An executed mutation is not required, and no fresh label, private Ledger access, direct `VirtualClock` movement, real-time sleep, statistical sampling, seed sweep, or tolerance widening is admitted into the witness.
