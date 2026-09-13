# A stale adverse history dissolves on the clock · `plan:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels`

Keeping (´claim:ledger:a-stale-adverse-history-dissolves-without-fresh-labels´) establishes that a cell's adverse outcome memory and its resulting excess risk follow the configured elapsed-time decay after the live Sentinel report becomes neutral, even when no further label reaches the region.

## What the promise says, precisely · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-promise`

The stale state is the Ledger entry's adverse-rate, compressed-valence, and raw-valence averages, the three base features extracted from outcome memory rather than from the current report (´tab:extraction:ledger-features´). A burst writes all three at every containing layer, while a read selects the deepest entry covering the request coordinate (´alg:ledger:all-layers-update´) (´dec:memory:depth-walk´).

Write the configured hourly retention as $\gamma_{t,L}$ and elapsed hours after the last label as $h$. With no later label, each stored Ledger feature contributes its value at the cutoff multiplied by $D(h)=\gamma_{t,L}^{h}$; read-time evaluation is pure and does not advance the stored timestamp (´def:ledger:time-decay´) (´alg:temporal:lazy-application´).

The shipped configuration fixes $\gamma_{t,L}=0.999$ per hour (´tab:config:temporal´). Its half-life is about twenty-nine days, so the analytic retained fractions are about one half, one quarter, and one eighth after twenty-nine, fifty-eight, and eighty-seven days; under complete label starvation those factors are the only motion in the Ledger (´thm:temporal:ledger-decay-bound´).

The measurable baseline is a sibling cell in the same current report with identical neutral measurement fields and no outcome history. The adverse and baseline assessments therefore share report staleness, batch features, aggregate features, standardisation state, model parameters, and anchor inputs; only the three Ledger positions in the per-Sentinel slot differ (´alg:runtime:extraction-routing´) (´tab:feature:aggregate-block´).

Let $\Delta\rho(h)=\rho_{\mathrm{eff,adverse}}(h)-\rho_{\mathrm{eff,baseline}}(h)$. The anchor projection contains no per-Sentinel slot, and the blend weight is computed only over the anchor's shared subspace (´def:risk:anchor-model´) (´def:risk:subspace-blend´), so matched current inputs make the anchor score, blend weight, and calibration parameter equal across the pair and give $\Delta\rho(h)=D(h)\Delta\rho(0)$.

The setup requires $\Delta\rho(0)>100\epsilon_{\mathrm{ewma}}$ so the curve cannot pass vacuously, and every milestone compares the observed raw-risk gap with $D(h)\Delta\rho(0)$ within the harness's named EWMA tolerance. The public probability $p_{\mathrm{bad}}=\sigma(\rho_{\mathrm{eff}}/\kappa_{\mathrm{eff}})$ need not halve because the sigmoid is nonlinear; it remains above the matched baseline and approaches it monotonically as the raw gap vanishes (´def:risk:probability´).

The absence of fresh labels begins after the history cutoff. Report refreshes and assessments may continue, but the snapshot version and both calibration-record counts remain fixed, distinguishing clock-only recovery from hidden training; the guarantee is the bounded dissolution of stale influence, not the acquisition of contrary evidence (´inv:guarantee:ledger-floor´) (´alg:valence:contamination-loop´).

## What the code offers today · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-code-today`

The Ledger entry implementation exposes `LedgerEntry::read_decayed` as a non-mutating multiplication of every stored average by `decay_factor_since` (´def:ledger:time-decay´), and the extraction path feeds that view into the three Ledger feature positions selected by deepest-covering-entry routing (´alg:runtime:extraction-routing´).

The assessment operation takes the persistent present from the injected engine clock at the head of each assessment, passes `config.temporal.gamma_t_ledger` into extraction, and exposes `RiskBasis::rho_eff`, `p_bad`, `anchor_weight`, `kappa_eff`, and the calibration-record counts on the public result (´schema:output:assessment´). The two timestamp domains and the shared decay funnel are record-level constraints rather than test conventions (´dec:clock:two-domains´) (´dec:clock:shared-functions´).

The completed world-harness skeleton supplies a real `Assayer`, a shared `VirtualClock`, deterministic construction, named Sentinel registration and report reception, requests carrying a Sentinel coordinate, paired batch derivation, adverse and benign labels, label and observation barriers, cold-ramp settlement, named tolerances, and clean-health assertions (´entry:assayer:harness-stage-skeleton´). `World::clock().advance(...)` can already move both injected time domains without sleeping.

The report-fixture support supplies `make_cell_report` and complete golden reports, but no report fixture contains several sibling cells with identical controllable live measurements. The standing fixture policy treats such readings as authored stimulus (´dec:assayer:golden-report-stimulus´), so building that shape is test data rather than a new engine surface.

The nearest mechanism witness drives fifty adverse updates directly into a `LedgerEntry` and proves its bad-rate view follows the day-zero through day-eighty-seven analytic timeline without mutating storage (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´). It bypasses the injected clock, report routing, feature assembly, learned weights, and public risk result.

The public numerical witness proves that `0.999` retains about one half over twenty-nine days (´test:integration:decay-factor-29-day-half-life´), while the extraction witness proves deepest-cell selection (´test:unit:ledger-read-takes-the-deepest-covering-entry´). The public Ledger integration suite proves the root survives sustained assessment traffic (´test:integration:root-survives-many-assessments-within-lifetime´), but none of these witnesses composes a label-starved cell, virtual time, neutral live reports, and the risk basis.

The standing testing plan covers this promise directly: the persistent-clock stage names it among the decay-driven surfaces it gates and records that the assessment and label sites already use the injected clock, while the scenario-level time verbs remain unfinished (´entry:assayer:harness-stage-clock´).

## The witness · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-witness`

The witness belongs beside the public Ledger lifetime and risk-basis scenarios represented by (´test:integration:root-survives-many-assessments-within-lifetime´), and its module index and test documentation cite the intent.

- Setup: arm `fail_fast_on_model_owner_panic`, build one seeded `World` at the default temporal configuration, register one Sentinel, ingest a report with three sibling depth-eight cells beneath one common ancestor, and settle the cold standardisation ramp using requests through all three coordinates.

- Training stimulus: keep the adverse and benign training cells' live report features identical, then alternate roughly seven hundred ground-truth adverse labels through the first with the same number of benign labels through the second. The balanced outcomes keep the global intercept away from saturation while the per-cell Ledger histories become the discriminating inputs; the third cell receives no label and remains the neutral baseline.

- Neutralisation: flush the final label, ingest a report in which all three sibling cells carry identical neutral current measurements, and declare that instant $h=0$. No call to `label`, `cycle_request`, or `pre_seed` occurs after this cutoff.

- Initial observation: derive the adverse and baseline requests together, require equal `anchor_weight` and `kappa_eff`, require $\Delta\rho(0)>100\epsilon_{\mathrm{ewma}}$, require `p_bad` for the adverse cell to exceed the baseline, and record the snapshot version, calibration counts, and baseline risk basis.

- Clock stimulus: advance only virtual time through successive intervals whose cumulative positions are days seven, fourteen, twenty-nine, fifty-eight, eighty-seven, and two hundred and ninety. Before each paired assessment, receive the same neutral report so report staleness stays zero and a collected empty baseline entry is recreated neutrally without supplying outcome evidence.

- Analytic observation: compute each expected factor independently in the test as `gamma_t_ledger.powf(24.0 * days)` and require the observed $\Delta\rho$ to equal that factor times $\Delta\rho(0)$ within `world.tol().ewma`. Direct exponentiation is an independent test oracle and is explicitly outside the production prohibition (´rule:clock:no-direct-exponentiation´).

- Recovery observation: require the adverse `p_bad` to move monotonically toward the same-checkpoint baseline, require the baseline `rho_eff` and `p_bad` to remain within the named default tolerance of their cutoff readings, and require the two-hundred-and-ninety-day raw excess to be below one thousandth of its cutoff magnitude.

- Isolation observation: at every checkpoint require the pair's anchor weight and effective calibration parameter to agree, require snapshot version and calibration counts to equal the cutoff values, repeat one paired read without moving the clock and require the raw gaps to agree, and finish with clean health.

The fails-before is an implementation that consults the wall clock, omits read-time decay, waits for the next label to apply decay, uses the core model's slower hourly rate, decays only part of the Ledger feature triplet, or persists decay on a read. Such an implementation leaves the raw-risk gap flat, follows the wrong curve, or changes on the repeated same-instant read even though the existing component tests may remain green.

## What is missing · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-missing`

**Entry (World owns the persistent-time verbs)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-world-time-verbs`

Add `World::advance(Duration)` and `World::travel_to(SystemTime)` as thin forward-only calls into the existing `VirtualClock`, approximately fifteen lines plus their harness test-index coverage. This consumes the remaining scenario-vocabulary portion of (´entry:assayer:harness-stage-clock´), depends on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´), and has no dependency on another intent lane.

**Entry (A controlled three-cell report fixture)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-three-cell-report-fixture`

Add a test-local builder of approximately fifty lines to the public Ledger integration suite that constructs one root, one shared ancestor, and three sibling competitive cells with separately selectable but structurally identical score arrays. It composes from `make_cell_report` and the existing report types, depends on no production change or sibling lane, and makes the hot, benign-training, and untouched baseline coordinates explicit.

**Entry (The clock-only recovery integration witness)** · `entry:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-recovery-witness`

Add the focused scenario and module-index row to that integration suite, approximately ninety lines, with the setup, balanced history, cutoff, milestones, analytic raw-gap oracle, probability recovery, no-training checks, repeated-read purity check, and health assertion above. It depends on the two entries in this section and needs no Ledger-state accessor, tape, persistence fixture, random sampling, drift-budget widening, or wall-clock deadline.

## Risks and open questions · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-risks`

**Observation (The raw gap carries the analytic curve)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-raw-gap-oracle`

Probability differences do not inherit exponential ratios through a sigmoid, and asserting that `p_bad` halves would strengthen the promise into a false equation. The matched-cell construction makes `rho_eff` the analytic oracle and retains `p_bad` as the public directional recovery observation.

**Observation (The fixture proves its effect before timing it)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-effect-liveness`

The alternating training cells make Ledger state predictive without driving the intercept toward certainty, but the learned coefficient remains an empirical consequence of the deterministic fixture. The initial raw-gap floor and equal-anchor preconditions fail loudly if that consequence is absent; weakening the floor or widening the curve tolerance cannot repair such a fixture.

**Observation (Neutral report refresh is not fresh outcome evidence)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-report-refresh-is-not-a-label`

Refreshing the unchanged neutral report at each milestone resets report staleness and keeps the two current measurement surfaces equal. It neither writes a Ledger average nor trains a model, and the fixed snapshot version and calibration counts make that separation observable.

**Observation (Baseline is a limit with a controlled finite proxy)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-baseline-is-a-limit`

The specification gives an exponential curve and no finite absolute `p_bad` distance called recovered. The same-checkpoint untouched cell is the baseline, the analytic raw gap proves convergence to it, and the final milestone makes the residual less than a thousandth of its initial magnitude without inventing a product threshold; an operational absolute threshold belongs in configuration or specification before a test can enforce it.

**Observation (The asynchronous surfaces are held still)** · `obs:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-determinism`

Cold-ramp settlement occurs before training, every training label is followed by the harness barrier, paired observations use one batch time, no identity dimension is registered, and virtual time replaces sleeping. The model-owner panic hook is failure containment rather than a timing oracle, and no drift budget justifies departure from the analytic curve.

No maintainer decision blocks this witness: the specification fixes the rate and curve, the public risk basis supplies the raw observable, and the controlled baseline gives the word recovery a measurable referent.

## Acceptance · `sec:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels-acceptance`

The implementing lane's report names the integration test and its claim citation, the three entries it landed, and the exact commands and green outputs for formatting, linting, Assayer tests with all targets and features, the no-default-features spot check, release tests, documentation, and documentation tests.

The report shows the initial non-vacuous raw-risk gap; observed and expected retained fractions at every milestone; the adverse and baseline `p_bad` trajectory; stable baseline, anchor weight, calibration parameter, snapshot version, and calibration counts; repeated-read purity; and clean health.

The coverage report resolves the intent to the new integration witness, while the existing direct Ledger timeline and numerical half-life witnesses remain green as mechanism controls.

The report states which analytic or isolation assertion rejects each deliberately broken behaviour described above. An executed mutation is not required, and no fresh label, private Ledger access, real-time sleep, statistical sampling, or tolerance widening is admitted into the witness.
