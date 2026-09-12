# Marginalising a Sentinel leaves its information behind · `plan:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind`

Keeping (´claim:lifespan:marginalising-a-sentinel-leaves-its-information-behind´) establishes that destructive Sentinel deregistration preserves the learned marginal over surviving features through the specified Schur update and leaves enough retained signal for held-out discrimination to remain materially better than a cold model.

## What the promise says, precisely · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-precise-promise`

Let the learned precision before deregistration be partitioned as retained coordinates $k$ and the departing Sentinel's coordinates $r$. Exact marginalisation requires $B' = B_{kk} - B_{kr}B_{rr}^{-1}B_{rk}$, with the retained mean and covariance equal to the corresponding pre-removal marginal (´thm:gaussian:marginalisation´) and the correction carrying the removed block's cross-information into the surviving model (´def:gaussian:schur-complement´).

The operational path uses $B'_\delta = B_{kk} - B_{kr}(B_{rr}+\delta I)^{-1}B_{rk}$, where $\delta = \lambda_{\mathrm{prior}}\epsilon_{\mathrm{Schur}}$, $\epsilon_{\mathrm{Schur}}$ defaults to $10^{-4}$, the raw posture ceiling defaults to $10^8$, the regularised solvability ceiling is $10^{12}$, and failure falls back to $B_{kk}$ (´alg:gaussian:regularised-schur´). The reference risk configuration fixes $\lambda_{\mathrm{prior}}=0.1$ and the same regularisation and posture values (´tab:config:risk-model´).

Deregistering the Sentinel applies that operation to the operational model, sister model, and every outcome-axis model before the registry, routing, standardisation, and dimension map publish the reduced state (´alg:registry:sentinel-deregistration´). Destructive deregistration is the default lifecycle decision specifically because marginalisation carries information into the survivors (´dec:construction:destructive-deregistration´).

The matrix witness measures relative Frobenius error between the independently computed $B'_\delta$ and the published survivor precision at the harness default tolerance of $10^{-9}$, verifies a non-zero correction above that tolerance, and verifies that at least one selected retained diagonal remains greater than $\lambda_{\mathrm{prior}}$ by more than that tolerance. The same witness verifies $B'_\delta \preceq B_{kk}$ and a strict decrease in at least one coupled retained direction, the ordering already isolated by (´test:crate:schur-vs-bkk-ordering´).

The diagnostic witness requires one additional marginalisation event, no additional skipped correction, and an event loss fraction below $1$; the specified error ledger records trace loss and assigns loss fraction $1$ to a fallback (´req:gaussian:marginalisation-error-reported´). This precondition keeps the test on the regularised Schur branch while retaining separate coverage of the infallible $B_{kk}$ fallback fixed by (´dec:posterior:infallible-correction´).

The behavioural witness uses a balanced held-out tape with at least twenty adverse and twenty benign examples, the minimum class counts fixed for rank AUC (´alg:monitoring:auc´). Its provisional graceful-degradation bounds are post-removal AUC at least $0.65$, pre-to-post AUC loss at most $0.10$, and post-removal advantage over an otherwise identical cold control greater than the harness AUC tolerance of $0.005$; the first two values reuse the monitoring vocabulary for moderate discrimination and instability (´tab:monitoring:interpretation´), while the tolerance comes from (´tab:assayer:harness-scenario-tolerances´).

The training tape runs for at least the model's $2p$ eligible-label budget so the posterior is shaped by observations rather than its prior (´bound:resource:convergence-budget´). Held-out requests never enter the labelled training population, and the before, after, and cold-control AUC calculations use the same examples and pairwise rank statistic.

## What the code offers today · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-code-offers`

The public integration path starts with the world harness: `scenario_with` and `WorldBuilder` construct a deterministic `World`, `World::register_sentinel` installs the source, `World::receive_report` and `World::request_with_sentinel` create routed observations, `World::label` supplies outcomes, and `World::deregister_sentinel` drives the production lifecycle operation and settles publication. `World::cycle`, `World::flush`, the virtual clock, the seeded `TestRng`, drift budgets, and liveness deadlines provide deterministic barriers rather than sleeps.

The report fixtures supply `golden_report`, `make_cell_report`, `GOLDEN_COORD`, and `attach_golden_reporting_sentinel`, which can give one Sentinel a label-correlated routed feature. The signal fixture supplies the retained score-verified request signal, and the label specification supplies adverse and benign labels.

The completed harness skeleton already covers the builder, lifecycle operations, report and label injection, deterministic randomness, scenario tolerances, and assertion primitives needed here (´entry:assayer:harness-stage-skeleton´). It does not expose a posterior partition keyed by the dimension map, a reusable converged correlated tape, or a held-out AUC comparison.

The nearest algebraic coverage proves full cross-term transfer (´test:crate:information-transfer´), the Schur-versus-$B_{kk}$ ordering (´test:crate:schur-vs-bkk-ordering´), and an extend-observe-marginalise round trip (´test:crate:round-trip-extend-observe-marginalise´). These tests keep the matrix primitive but do not drive Sentinel deregistration through the public API or measure retained discrimination.

The nearest lifecycle crate test proves that deregistration shrinks all learned models (´test:crate:deregister-sentinel-marginalises-models´), while repeated public registration and deregistration is already kept finite and healthy (´test:integration:lifecycle-churn-register-deregister-32-rounds´). Neither distinguishes a correct Schur transfer from discarding or resetting the learned survivor state.

Existing convergence coverage already demonstrates deterministic population-split learning and computes held-out pairwise AUC through public assessments (´test:integration:scalar-signal-population-split-converges-directionally´). Its local scoring helper and single-signal fixture are a model for the behavioural half of this witness, not reusable harness API today.

## The witness · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-witness`

The integration test joins the existing public lifecycle story (´test:integration:lifecycle-churn-register-deregister-32-rounds´) and adds its own test mint and module-index entry when implemented.

Setup constructs a treatment world and a cold-control world from the same scenario, seed, virtual-clock origin, signal schema, standardisation ramp, and model configuration. The treatment registers one informative Sentinel; every request also carries a retained score-verified signal, and the report route and retained signal are deliberately correlated with each other and with the balanced adverse/benign label tape.

The fixture settles the cold standardisation ramp, feeds at least $2p$ eligible training labels, flushes every lifecycle phase through bounded harness barriers, and reserves a balanced held-out tape. A pre-removal probe records $B_{kk}$, $B_{rr}$, $B_{kr}$, the retained dimension identities, $\lambda_{\mathrm{prior}}$, Schur configuration, and marginalisation counters; a non-zero cross-block norm makes the test's transfer premise explicit.

The first observation scores the held-out tape with the informative Sentinel present. The stimulus is one call to `World::deregister_sentinel`, after which the test flushes the publication barrier and confirms the Sentinel is absent from subsequent requests and model dimensions.

The second observation uses the probe to read the published reduced precision and scores the identical held-out cases without the retired coordinate. The cold control scores those same retained-signal-only cases without receiving the treatment's training labels.

The matrix assertion independently computes the regularised Schur complement from the pre-removal blocks, compares every reduced entry and dimension identity with the published state, checks the ordering and prior-retention conditions, and confirms the event diagnostics show a performed rather than skipped correction. This makes the algebraic mechanism, not AUC alone, the causal witness.

The behavioural assertion applies the three provisional AUC bounds from the precise-promise section. It reports the before, after, and cold AUC values together with class counts, label budget, feature dimension, and random seed so a failure identifies whether transfer, discrimination, or fixture liveness moved.

A deliberately broken implementation that rebuilds survivor models from the prior produces precision at the cold floor and post-removal AUC near the cold control. An implementation that merely deletes the departing rows and columns publishes $B_{kk}$, so the independent Schur comparison, strict correction ordering, and event diagnostic fail even if its overconfident scores happen to retain AUC. An implementation that marginalises the wrong dimensions fails the dimension identity and full-matrix comparison.

## What is missing · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-missing`

The standing gap that covers this promise's missing infrastructure is the open tape, convergence, snapshot, and domain-assertion stage (´entry:assayer:harness-stage-tapes´); no standing entry names this promise itself.

**Entry (A correlated deregistration tape makes transfer observable)** · `entry:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-correlated-tape`

The fixture adds approximately 90–130 lines to the shared testing harness for deterministic training and held-out tapes that pair report routes, retained signal values, and balanced labels, derive $p$ and the $2p$ budget, record class counts, and compute pairwise AUC without labelling held-out cases. This entry depends on (´entry:assayer:harness-stage-tapes´) and reuses the completed scenario skeleton; it has no dependency on a sibling intent lane.

**Entry (A posterior partition probe exposes the event boundary)** · `entry:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-posterior-probe`

The probe adds approximately 70–100 lines to the shared testing harness for a test-only, hidden public value snapshot that selects a named risk model and returns retained/removed dimension identities, $B_{kk}$, $B_{rr}$, $B_{kr}$, prior precision, Schur configuration, and marginalisation diagnostics without permitting mutation. This entry depends on the snapshot support in (´entry:assayer:harness-stage-tapes´) and on stable dimension-map lookup, and it has no dependency on a sibling intent lane.

**Entry (The public lifecycle witness joins algebra to discrimination)** · `entry:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-integration-witness`

The witness adds approximately 90–130 lines to the public lifecycle integration suite (´test:integration:lifecycle-churn-register-deregister-32-rounds´) for the setup, deregistration stimulus, independent Schur calculation, diagnostic assertions, and three-way AUC comparison. This entry depends on (´entry:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-correlated-tape´), (´entry:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-posterior-probe´), and (´entry:assayer:harness-stage-skeleton´).

## Risks and open questions · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-risks`

**Observation (The promise's precision direction needs a reference state)** · `obs:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-precision-reference`

The Schur formula and existing ordering test make survivor marginal precision no greater than the conditional block $B_{kk}$ immediately before removal, so a literal assertion that precision rises across the deregistration event contradicts the specified mathematics. The consistent alternative reads “rises” against a cold or prior-reset survivor: the chosen retained direction remains above $\lambda_{\mathrm{prior}}$ after removal while the event itself correctly reduces $B_{kk}$ by a positive-semidefinite correction. The competing alternative changes the mathematical contract. The witness uses the cold/prior reference, and the intent wording needs that reference made explicit before implementation acceptance.

**Observation (Graceful discrimination has no lifecycle guarantee threshold)** · `obs:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-discrimination-threshold`

The $0.65$ band and $0.10$ change are monitoring interpretation thresholds, not a normative deregistration bound. One alternative ratifies them for this controlled fixture; another keeps only relational assertions that post-removal AUC exceeds cold by more than $0.005$ and retains more than half of the treatment's pre-removal advantage. The witness provisionally uses the corpus values because they make “gracefully” falsifiable, and implementation acceptance records which alternative becomes the promise's contract.

**Observation (Regularisation and fallback qualify information retention)** · `obs:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-fallback-qualification`

The two-level guarantee explicitly permits attenuation under regularisation and complete loss of cross-information under the $B_{kk}$ fallback (´thm:gaussian:two-level-guarantee´). The fixture keeps the raw and regularised postures comfortably below their ceilings, asserts a non-skipped correction and loss fraction below $1$, and therefore tests the informative regularised branch rather than claiming unconditional retention.

**Observation (Matrix scale and rank statistics need separate tolerances)** · `obs:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-numerical-fragility`

An entrywise absolute comparison can become brittle as training changes matrix scale, while AUC changes in discrete pair-count increments. The matrix check uses a norm-scaled comparison with the harness default tolerance plus a separately bounded correction norm, and the behavioural check uses the AUC tolerance with enough balanced held-out pairs for one inversion to be smaller than that tolerance.

**Observation (Lifecycle publication must be observed, not timed)** · `obs:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-publication-ordering`

Model recomputation, registry publication, and health publication cross one lifecycle boundary. The witness uses `World` cycle and flush barriers, virtual time, and existing liveness deadlines after registration, ramp settlement, training, and deregistration; wall-clock sleeps or opportunistic polling introduce nondeterminism and are outside the fixture.

## Acceptance · `sec:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind-acceptance`

The implementation report identifies the integration-test mint and path, the exact seed and generated dimension, the eligible training-label budget and held-out class counts, the pre-event cross-block and correction norms, and the before, after, and cold-control AUC values.

The report shows the independently calculated regularised Schur complement agrees with the published reduced precision within the declared matrix tolerance, dimension identities agree exactly, at least one coupled survivor direction changes strictly from $B_{kk}$, retained precision remains above the prior reference, and the marginalisation event is recorded without fallback.

The report shows the selected graceful-discrimination bounds pass under repeated deterministic runs, names the resolved precision reference and discrimination threshold alternative, and identifies the expected failing assertions for both the prior-reset and $B_{kk}$-only broken implementations.

The report shows the new fixture and probe are confined to test support, documents their approximate line additions against the three entries, records all named package gates, and reports no unbounded wait, sleep, ignored diagnostic, or unrelated promise coverage.
