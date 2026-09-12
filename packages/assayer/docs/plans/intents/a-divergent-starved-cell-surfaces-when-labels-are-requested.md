# Keeping Divergence and Starvation Joined in Label Guidance · `plan:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested`

This plan keeps the promise that public label guidance returns a pending request routed through a cell whose adverse rate is both elevated and supported by too few eligible labels, while excluding controls that carry only one of those conditions (´claim:guidance:a-divergent-starved-cell-surfaces-when-labels-are-requested´).

## What the promise says, precisely · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-promise`

The observable is the `starvation_relief` list returned by the public label-guidance interface, whose budget is independent of the risk-informative and investigation budgets and whose candidates retain the pending assessment identifier needed for labelling (´sig:guidance:interface´). The surface is a synchronous, infallible read of published and retained state (´dec:surface:read-only-guidance´), and its three lists are ranked and truncated independently (´dec:surface:three-categories´).

For a pending request $r$, each reporting Sentinel routes its coordinate to one Ledger cell and contributes $(\bar b-P_+^{\mathrm{eligible}})^+\left(1-n_{\mathrm{elig}}/N_{\mathrm{ledger}}\right)^+$; the request score is the maximum contribution across those Sentinels (´def:guidance:starvation´). The Ledger definition fixes the product rather than either factor alone (´def:ledger:starvation-score´).

The witness fixes the routed cell's remembered adverse-rate average $\bar b$ at $0.15$, reads a system-wide eligible positive rate below $0.06$, and fixes the cell's recent eligible-label count at three. The shipped window is $N_{\mathrm{ledger}}=200$ (´tab:config:ledger´), so the count factor is $1-3/200=0.985$ and the expected score is $(0.15-P_+^{\mathrm{eligible}})\times0.985$.

The configured starvation threshold is $0.01$ by default (´tab:config:guidance´). The positive score stands well above that threshold; equality at the threshold is outside this promise.

Two negative controls make the conjunction measurable. A cell at adverse rate $0.15$ with two hundred eligible labels has a zero count-shortfall factor, while a cell with three eligible labels and adverse rate zero has a zero divergence factor. Neither assessment appears in `starvation_relief`, even though all three remain pending and inside the scan and output budgets.

The Ledger stores the bad-rate average and the cumulative eligible count separately (´tab:ledger:entry-state´). Every label updates the average, while only eligible labels increment the count (´alg:ledger:all-layers-update´); the test uses `Allow` labels, which are eligible by definition (´tab:eligibility:training´).

## What the code offers today · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-current-code`

The guidance implementation exposes `LabelBudget`, `LabelGuidanceParams`, `LabelRequests`, `LabelCandidate`, and the public implementation reached by `Assayer::request_labels`. It reads a bounded snapshot of the retained pending map (´dec:retention:pending-map´), loads the eligible rate from the published model snapshot, reads the engine's persistent clock, routes every usable Sentinel extraction through its Ledger, filters scores below the threshold, and returns the selected live candidates.

The scorer clips both factors at zero, caps the count at the private integer form of the two-hundred-label window, multiplies the factors, and applies the threshold before selection. Its per-Sentinel Ledger reads are bounded by the traversal decision (´dec:concurrency:bounded-traversal´); the planned three-candidate fixture cannot approach that ceiling.

The focused unit witness already proves the numeric product and both zero-factor cases (´test:unit:starvation-scorer-requires-divergence-and-starvation´), and another proves that the scorer reads the routed Ledger entry (´test:unit:starvation-scorer-reads-routed-ledger-entry´). These tests bypass the public request and selection path.

The nearest crate-level witness seeds an exact divergent, starved entry and proves that a real `request_labels` call returns it with a score near $0.0985$ (´test:crate:request-labels-ranks-seeded-pending-snapshot´). It reaches private state directly and supplies no well-fed or unremarkable pending control.

The nearest integration witness proves that the public call returns a genuinely pending assessment by identifier with a finite risk-informative score (´test:integration:request-labels-public-api-returns-pending-candidate´). It allocates no starvation-relief budget and creates no Ledger history.

The completed harness supplies configured `Scenario` construction, `World`, a virtual clock shared with the engine, named Sentinel registration, `attach_golden_reporting_sentinel`, `GOLDEN_COORD`, request construction with a Sentinel coordinate, assessment, `LabelSpec`, label submission, the label-publication barrier, access to `Assayer::request_labels`, and named tolerances (´entry:assayer:harness-stage-skeleton´). The default tolerance bundle distinguishes one-shot arithmetic from accumulated EWMA comparisons (´tab:assayer:harness-scenario-tolerances´).

The standing testing plan has no gap entry covering this promise.

## The witness · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-witness`

The witness belongs in the integration binary beside the existing public guidance test (´test:integration:request-labels-public-api-returns-pending-candidate´); its documentation and module-index row cite the intent rather than minting a second claim.

- Setup: build one seeded scenario with `p_plus_init = 0.05`, `ledger.lambda_l = 0.85`, the ordinary eligible-tracker rate, one default channel, and the unadvanced virtual clock. Both configuration values lie inside the declared Core and Ledger domains (´tab:config:risk-model´) (´tab:config:ledger´).

- Reporting surface: register three named Sentinels and attach the canonical golden report to each. Every training and candidate request uses `GOLDEN_COORD`, except one benign background request at coordinate zero used to make the positive Sentinel's root differ from its routed depth-eight cell.

- Divergent-starved history: send two benign eligible labels and then one adverse eligible label through the first Sentinel's depth-eight cell. Starting from zero at $\lambda_L=0.85$, the first adverse observation leaves that cell at exactly $1-0.85=0.15$ with an eligible count of three. Send one further benign label outside that child cell so an implementation reading the root produces a different score.

- Divergent-fed history: send one hundred and ninety-nine benign eligible labels and then one adverse eligible label through the second Sentinel's depth-eight cell. Its adverse-rate average is likewise $0.15$, while its eligible count is exactly the two-hundred-label window and therefore zeroes the starvation factor.

- Ordinary-starved history: send three benign eligible labels through the third Sentinel's depth-eight cell. Its adverse-rate average remains zero and its count remains short of the window, so divergence alone excludes it.

- Synchronisation: submit each label against the assessment that produced it and cross `World::flush_labels` before relying on the resulting state. Advance neither clock domain; the score's read-time Ledger decay is then inert under the separation of timestamp domains (´dec:clock:two-domains´).

- Pending population: assess one fresh request through each prepared cell without labelling it. Retain all three assessment identifiers, then read `health_summary().p_positive_eligible` and require a finite value below $0.06$ before using it as the score oracle.

- Stimulus: call `request_labels` once with zero budgets for the other categories, a starvation-relief budget of three, the default scan limit, and threshold $0.01$.

- Observation and assertion: require `starvation_relief` to contain exactly the divergent-starved assessment, require the fed and ordinary identifiers to be absent, and compare its score with $(0.15-P_+^{\mathrm{eligible}})\times(1-3/200)$ at `world.tol().default`.

The fails-before has distinct signatures. Dropping the divergence factor admits the ordinary-starved control; dropping or misapplying the count factor admits the divergent-fed control; adding rather than multiplying admits both controls; reading the root instead of the routed child misses the exact score; omitting the public starvation selection returns no positive candidate; and computing from a stale or global positive rate fails the score oracle.

## What is missing · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-missing`

No production surface, shared harness extension, tape, random generator, clock advancement, drift budget, or sibling intent lane blocks the witness.

**Entry (The public cell-history driver)** · `entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-cell-history-driver`

A private helper beside the existing public guidance witness (´test:integration:request-labels-public-api-returns-pending-candidate´), roughly fifteen lines long, repeatedly constructs a request through a named Sentinel and coordinate, assesses it, builds an eligible benign or adverse `LabelSpec`, submits the label, and flushes it. It depends only on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´) and keeps the state-producing path public.

**Entry (The three-cell public guidance witness)** · `entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-integration-witness`

A focused integration test of roughly sixty lines adds the configured scenario, three reported Sentinels, the three histories, the pending requests, the public eligible-rate reading, the one guidance call, and the positive and negative assertions. It depends on (´entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-cell-history-driver´), updates the file's test index and generated integration matrix, and depends on no other lane.

## Risks and open questions · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-risks`

**Observation (A fast Ledger rate is fixture geometry)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-fast-ledger-rate`

The non-default $\lambda_L=0.85$ is a valid configuration that reaches the promised $0.15$ state in one adverse update after zero-valued history. It changes no scoring rule and removes a long convergence prelude; pinning the default $0.999$ would require about a hundred and sixty adverse updates and would test the same guidance state less directly.

**Observation (The public eligible rate closes the score oracle)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-live-base-rate`

Eligible labels move the shared base-rate tracker by the specified recurrence (´alg:weighting:tracker-update´), so a literal $0.05$ score input would cease to describe the state after training. Reading the published eligible rate after the final label barrier keeps the oracle independent of copied tracker arithmetic while the explicit upper bound preserves the promise's lower-base-rate precondition.

**Observation (The two-hundred window has two carriers today)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-window-carrier`

The specification tabulates $N_{\mathrm{ledger}}$ as a configurable Ledger parameter (´tab:config:ledger´), while `LedgerConfig` exposes no such field and the guidance implementation carries private scalar and integer constants fixed at two hundred (´const:assayer:starvation-relief-window-scalar-200p0´) (´const:assayer:starvation-relief-window-integer-count-200´). The promised value is therefore testable and the witness pins it through the denominator and fed control; whether a later surface exposes it as host configuration is outside this promise.

**Observation (Time, scheduling, and ranking are removed from the result)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-deterministic-boundaries`

The virtual clock never advances, each label crosses the existing publication barrier, only three live pending assessments remain, and the output budget can hold all three. No elapsed-time allowance, random sample, ordering tie, scan truncation, or cross-world comparison enters the assertion, and the one-shot tolerance is sufficient.

No maintainer decision remains open for this witness; the configuration-carrier discrepancy is a later API question rather than a choice that changes the test.

## Acceptance · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-acceptance`

The implementing lane's report identifies the new integration test and private history helper beside the existing public guidance witness (´test:integration:request-labels-public-api-returns-pending-candidate´), and confirms that the test documentation, module index, and generated integration matrix resolve the intent to that witness.

The report shows the targeted integration test passing, the coverage report removing this intent from the unwitnessed set, and the package's prescribed formatting, lint, documentation, default-feature, no-default-feature, debug, and release gates passing, with unrelated failures separated explicitly.

The reported evidence includes the configured $0.15$ histories, the three-versus-two-hundred eligible counts, the public eligible base rate below $0.06$, the expected and observed starvation scores, the sole returned assessment identifier, the two excluded control identifiers, and the unchanged virtual time.

The report states which deliberate defect each positive, negative, routing, and numeric oracle rejects; no executed mutation is required.

Acceptance excludes private Ledger mutation, a new production accessor, a shared harness seeding hook, sleeps, clock travel, probabilistic workloads, a broad drift allowance, and separate worlds for the controls, because the existing public scenario path constructs and observes the complete state deterministically.
