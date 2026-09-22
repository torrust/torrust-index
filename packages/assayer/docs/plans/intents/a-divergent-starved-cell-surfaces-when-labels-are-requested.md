# Keeping Divergence and Starvation Joined in Label Guidance · `plan:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested`

This plan keeps the promise that public label guidance returns a pending request routed through a cell whose adverse rate is both elevated and supported by too few eligible labels, while excluding controls that carry only one of those conditions (´claim:guidance:a-divergent-starved-cell-surfaces-when-labels-are-requested´).

## What the promise says, precisely · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-promise`

The observable is the `starvation_relief` list returned by the public label-guidance interface, whose budget is independent of the risk-informative and investigation budgets and whose candidates retain the pending assessment identifier needed for labelling (´sig:guidance:interface´). The surface is a synchronous, infallible read of published and retained state (´dec:surface:read-only-guidance´), and its three lists are ranked and truncated independently (´dec:surface:three-categories´).

For a pending request $r$, each reporting Sentinel routes its coordinate to one Ledger cell and contributes $(\bar b-P_+^{\mathrm{eligible}})^+\left(1-n_{\mathrm{elig}}/N_{\mathrm{ledger}}\right)^+$; the request score is the maximum contribution across those Sentinels (´def:guidance:starvation´). The Ledger definition fixes the product rather than either factor alone (´def:ledger:starvation-score´).

The witness fixes the routed cell's remembered adverse-rate average $\bar b$ at $0.15$, requires the system-wide eligible positive rate to remain below $0.06$, and fixes the cell's recent eligible-label count at three. The shipped window is $N_{\mathrm{ledger}}=200$ (´tab:config:ledger´), so the count factor is $1-3/200=0.985$, the expected score is $(0.15-P_+^{\mathrm{eligible}})\times0.985$, and the rate bound makes that score greater than $0.08865$.

The default starvation threshold is $0.01$ (´tab:config:guidance´). The witness therefore stays away from the threshold boundary, which the interface specifies as excluding only scores below the threshold (´sig:guidance:interface´).

Two negative controls make the conjunction measurable. A cell at adverse rate $0.15$ with two hundred eligible labels has a zero count-shortfall factor, while a cell with three eligible labels and adverse rate zero has a zero divergence factor. Neither assessment appears in `starvation_relief`, even though all three remain pending and inside the scan and output budgets.

The Ledger stores the bad-rate average and cumulative eligible count separately (´tab:ledger:entry-state´). Every label updates the average, while only eligible labels increment the count (´alg:ledger:all-layers-update´); the witness uses labels with `Action::Allow`, which are eligible by definition (´tab:eligibility:training´).

## What the code offers today · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-current-code`

The guidance implementation exposes `LabelBudget`, `LabelGuidanceParams`, `LabelRequests`, `LabelCandidate`, and `Assayer::request_labels`. It reads a bounded snapshot of the retained pending map (´dec:retention:pending-map´), loads the eligible rate from the published model snapshot, reads the engine's persistent clock, routes every usable Sentinel extraction through its Ledger, filters scores against the threshold, and returns selected candidates that are still live.

The scorer clips both factors below at zero, caps the count at the private integer form of the two-hundred-label window, multiplies the factors, and applies the threshold before selection. Its per-Sentinel Ledger reads are bounded by the traversal decision (´dec:concurrency:bounded-traversal´); the planned three-candidate fixture cannot approach that ceiling.

The focused unit witness proves the numeric product and both zero-factor cases (´test:unit:starvation-scorer-requires-divergence-and-starvation´), and its neighbour proves that the scorer reads the routed Ledger entry (´test:unit:starvation-scorer-reads-routed-ledger-entry´). Both bypass the public request and label paths.

The nearest crate-level witness seeds a divergent, starved entry through private state and proves that `Assayer::request_labels` returns it with a score near $0.0985$ (´test:crate:request-labels-ranks-seeded-pending-snapshot´). It supplies no well-fed or unremarkable pending control.

The nearest integration witness proves that `Assayer::request_labels` returns a genuinely pending assessment by identifier with a finite risk-informative score (´test:integration:request-labels-public-api-returns-pending-candidate´). It allocates no starvation-relief budget and creates no Ledger history, so no existing test keeps this plan's claim.

The finished harness supplies configured `Scenario` construction through `scenario_with_config`, the real-engine `World`, `World::register_sentinel`, `World::receive_report`, `attach_golden_reporting_sentinel`, `GOLDEN_COORD`, `World::request_with_sentinel`, `World::assess`, `LabelSpec`, `World::label`, `World::flush_labels`, `World::assayer`, and named `Tolerances` (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:single-scenario´). Its report fixture routes `GOLDEN_COORD` through root, depth-four, and depth-eight cells (´dec:assayer:golden-report-stimulus´).

The long history can use a subject-owned `PlaybackRow` and the shared `playback` runner with `PlaybackBarrierPolicy` selecting `PlaybackBarrier::FlushLabels`; the runner owns the barrier cadence and reports the first incomplete row (´dec:harness:declarative-playback´). The focused crate witness establishes that a selected label barrier completes before its checkpoint (´test:crate:selected-label-barrier-completes-before-checkpoint´).

The owned probes `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`; the shared `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width` oracle tier; `TrainedStateFixture` and `TrainedStateBaseline`; and `run_seeded_sweep` have landed (´tab:assayer:harness-implementation-library-roster´), but none is the right mechanism here: the witness reads only public guidance and health state, no shared oracle computes the starvation formula, the exact finite history is not a reusable trained-state precondition, and the claim quantifies over one configured state rather than a seed space (´dec:harness:probe-contract´) (´dec:harness:oracle-tier´) (´dec:harness:guarded-fixtures´) (´dec:harness:seeded-sweeps´).

## The witness · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-witness`

The witness belongs in the `assess_integration` integration target beside `request_labels_public_api_returns_pending_candidate` (´test:integration:request-labels-public-api-returns-pending-candidate´); its documentation and module-index row cite the intent rather than minting a second claim.

- Setup: build one `Scenario` with `scenario_with_config`, set `ModelConfig::p_plus_init` to $0.05$ and `LedgerConfig::lambda_l` to $0.85$, retain the ordinary eligible-tracker rate, declare one default channel, and leave scenario time unadvanced. Both configured rates lie inside the declared Core and Ledger domains (´tab:config:risk-model´) (´tab:config:ledger´).

- Reporting surface: call `attach_golden_reporting_sentinel` for three named Sentinels. Every history and candidate request uses `GOLDEN_COORD`, except one benign background row at coordinate zero that makes the first Sentinel's root state differ from its routed depth-eight cell.

- Declarative history: define a private `GuidanceLabelRow` implementing `PlaybackRow`; each row names one Sentinel, entity, coordinate, and benign or adverse `LabelSpec`, and `PlaybackRow::play` uses `World::request_with_sentinel`, `World::assess`, and `World::label` without waiting. Drive the complete ordered history through `playback` with `PlaybackBarrierPolicy` containing only `PlaybackBarrier::FlushLabels`, an empty checkpoint slice, a fresh `PlaybackProgress`, and the default `PlaybackBatchSize`, so every label crosses its queue's declared publication barrier before the next row begins (´dec:harness:declarative-playback´) (´cor:concurrency:harness-barriers´).

- Divergent-starved history: play two benign eligible rows and then one adverse eligible row through the first Sentinel's depth-eight cell. Starting from zero at $\lambda_L=0.85$, the adverse row leaves that cell at exactly $1-0.85=0.15$ with an eligible count of three. Then play the benign coordinate-zero row through the same Sentinel so the root changes while the depth-eight cell does not.

- Divergent-fed history: play one hundred and ninety-nine benign eligible rows and then one adverse eligible row through the second Sentinel's depth-eight cell. Its adverse-rate average is likewise $0.15$, while its eligible count is the two-hundred-label window and therefore zeroes the starvation factor.

- Ordinary-starved history: play three benign eligible rows through the third Sentinel's depth-eight cell. Its adverse-rate average remains zero and its count remains short of the window, so the divergence factor excludes it.

- Time: do not call `World::advance` or `World::travel_to`; no elapsed-time decay applies, and all movement would have to use those forward-only scenario verbs (´dec:clock:two-domains´) (´tab:assayer:harness-implementation-library-roster´).

- Pending population: call `World::assess` on one fresh `World::request_with_sentinel` through each prepared cell and retain all three assessment identifiers. After playback, read `HealthSummary::p_positive_eligible` through `World::assayer` and require a finite value below $0.06$ before using it in the score formula.

- Stimulus: call `Assayer::request_labels` once with zero budgets for the other categories, a starvation-relief budget of three, the default scan limit, and threshold $0.01$.

- Observation and assertion: require `starvation_relief` to contain exactly the divergent-starved assessment, require the fed and ordinary identifiers to be absent, and compare its score with $(0.15-P_+^{\mathrm{eligible}})\times(1-3/200)$ under the `Tolerances::ewma` bound obtained from `World::tol`, the named bound for an expectation containing an EWMA value (´tab:assayer:harness-scenario-tolerances´).

The fails-before has distinct signatures. Dropping the divergence factor admits the ordinary-starved control; dropping or misapplying the count factor admits the divergent-fed control; adding rather than multiplying admits both controls; reading the root instead of the routed child changes the score; omitting public starvation selection returns no positive candidate; and using a stale or global positive rate fails the numeric oracle.

## What is missing · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-missing`

No production surface, shared harness extension, new probe, shared oracle, guarded fixture, sweep, clock advancement, or sibling intent work blocks the witness. Only the subject-owned playback row and the integration witness remain to be written.

**Entry (The public cell-history driver)** · `entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-cell-history-driver`

The `assess_integration` target needs a private `GuidanceLabelRow` carrying the row name, Sentinel name, entity, coordinate, and label valence and implementing `PlaybackRow` by assessing and labelling through `World`. It adds no barrier of its own; `playback` applies `PlaybackBarrier::FlushLabels` after every row, as required by the shared playback contract (´dec:harness:declarative-playback´).

**Entry (The three-cell public guidance witness)** · `entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-integration-witness`

The same target needs `a_divergent_starved_cell_surfaces_when_labels_are_requested`, which builds the configured scenario, attaches the three reported Sentinels, supplies the three histories as playback rows, creates the pending requests, reads the public eligible rate, calls public guidance once, and makes the positive, negative, routing, and numeric assertions. It depends only on (´entry:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-cell-history-driver´).

## Risks and open questions · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-risks`

**Observation (A fast Ledger rate is fixture geometry)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-fast-ledger-rate`

The non-default $\lambda_L=0.85$ is a valid configuration that reaches the promised $0.15$ state in one adverse update after zero-valued history. It changes no scoring rule and removes a long convergence prelude; the default $0.999$ would require roughly one hundred and sixty adverse updates to reach the same state and would test no additional guidance behaviour.

**Observation (The public eligible rate closes the score oracle)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-live-base-rate`

Eligible labels move the shared base-rate tracker by the specified recurrence (´alg:weighting:tracker-update´), so the configured $0.05$ initial value is not the score input after playback. Reading the published eligible rate after the final selected label barrier keeps the oracle independent of copied tracker arithmetic, while the explicit $0.06$ upper guard proves the lower-base-rate precondition before the score assertion.

**Observation (The two-hundred window has two carriers today)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-window-carrier`

The specification tabulates $N_{\mathrm{ledger}}$ as a configurable Ledger parameter (´tab:config:ledger´), while `LedgerConfig` exposes no such field and the guidance implementation carries private scalar and integer constants fixed at two hundred (´const:assayer:starvation-relief-window-scalar-200p0´) (´const:assayer:starvation-relief-window-integer-count-200´). The promised value is therefore testable, and the denominator and fed control pin it; whether a later surface exposes it as host configuration is outside this promise.

**Observation (Time, queues, and ranking are bounded away)** · `obs:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-deterministic-boundaries`

Scenario time never advances, each playback row selects only the label-publication barrier, only three live pending assessments remain, and the output budget can hold all three. `World::flush_labels` deliberately excludes the cold-ramp observation queue, but this witness reads neither accepted observation counts nor standardisation state, so adding `PlaybackBarrier::FlushObservations` would assert completion the result does not need (´cor:concurrency:harness-barriers´). No elapsed-time allowance, random sample, ordering tie, scan truncation, or cross-world comparison enters the assertion, and `Tolerances::ewma` is the declared bound for the only accumulated quantity.

No maintainer decision remains open for this witness; the configuration-carrier discrepancy is a later API question rather than a choice that changes the test.

## Acceptance · `sec:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested-acceptance`

The implementing lane adds the integration test target `assess_integration` function `a_divergent_starved_cell_surfaces_when_labels_are_requested` and its private `GuidanceLabelRow` beside `request_labels_public_api_returns_pending_candidate` (´test:integration:request-labels-public-api-returns-pending-candidate´). The test documentation cites this intent, the module index describes the conjunction and controls, and the generated integration matrix resolves the intent to that witness.

The targeted integration test passes and the coverage report removes this intent from the unwitnessed set. The package's prescribed formatting, lint, documentation, feature, and test gates pass, with unrelated failures separated explicitly.

The evidence reports the configured $0.15$ histories, the three-versus-two-hundred eligible counts, the public eligible base rate below $0.06$, the expected and observed starvation scores, the sole returned assessment identifier, the two excluded control identifiers, and unchanged scenario time.

The evidence states which deliberate defect each positive, negative, routing, and numeric assertion rejects; no executed mutation is required.

Acceptance excludes private Ledger mutation, a new production accessor, a test-support Ledger probe, a shared harness seeding hook, sleeps, polls, direct clock movement, probabilistic workloads, a broad drift allowance, and separate worlds for the controls, because the finished public scenario and playback surfaces construct and observe the complete state deterministically.
