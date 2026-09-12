# The Assayer Testing Plan · `plan:assayer:test-debt-plan`

This is the Assayer's testing plan: what the test corpus still owes the system, in what order the debts are worth paying, and which piece of harness infrastructure unblocks which unkept promise. It is planning rather than indexing. What each test establishes is written where the test is, and the tables saying which tests cover what are generated from those statements, so nothing here restates a count that a report already computes.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a citation, of a label minted in a document or in code alike. Every label minted here has area `assayer`, and every kind is drawn from a fixed registry of environment kinds. A test is cited by the derived label the repository's test-label profile mints for it, so a reference that names a renamed or deleted test fails the check rather than quietly going stale.

## What this plan is · `sec:assayer:testing-plan-standing`

**Observation (The plan reviewed a document that has since retired)** · `obs:assayer:testing-plan-standing`

This plan began as a review of the scenario matrix: a judgment of that catalogue's strengths, its gaps, and where the remaining effort was worth spending. The matrix does not survive as a document (`tab:testdocs:monolith-homes`), and three of the review's four subjects went with it. The promises it catalogued are now statements standing where they are established, or, where nothing establishes them yet, statements of intent in the test tree's own readme. The divisions it grouped them under are replaced by the areas the statements turned out to want, whose stakes stand in the area register. The tracker counting its coverage is replaced by the linter's coverage report.

What the review found does survive, because none of it was a fact about the document's shape. A promise nobody has written a test for is still unwitnessed after the catalogue naming it is gone; a test whose stated expectation is the wrong one is still wrong; and an ordering of the remaining work is still worth having. Those three things are the whole of this plan, and the sections below carry them as entries, observations, and one register of priority.

**Convention (Where the counts come from)** · `conv:assayer:testing-plan-counts`

This plan states no coverage figure of its own. How many tests carry a statement, how many statements no test witnesses, and how those divide by area are computed by the linter's coverage report from the same census the check judges against, and a plan that copied them would be a second reading of one corpus, drifting from the first at the speed of the corpus.

The rule this fixes is narrow and worth stating exactly, because the temptation recurs. A number that answers *how much of the corpus is in some state* belongs to the report. A number that is part of a promise — a tolerance, a label count a scenario runs to, a threshold a test asserts against — belongs to the statement making the promise, and is written here only where this plan is proposing the promise and no statement carries it yet.

**Summary (What landed while this plan waited)** · `summ:assayer:testing-plan-discharged`

Four of the review's findings are discharged, and they are recorded here rather than deleted so that a reader meeting a stale copy of the plan can tell what happened to them.

Batch assessment independence, proposed here as a missing scenario and separately as a process recommendation, is kept: the interface's promise that each request in a batch is processed with no cross-request state (`req:runtime:assessment-interface`) is witnessed by (`test:integration:assess-batch-matches-forward-and-reverse-singletons`).

The convergence smoke the plan asked for in the harness's first stage is kept by (`test:integration:scalar-signal-population-split-converges-directionally`), which runs a scalar signal population split to a directional separation rather than to a convergence story.

The recovery and failure semantics the plan found thinnest gained five public witnesses. Label-channel overflow is witnessed twice, with and without a journal, by (`test:integration:label-channel-overflow-without-journal-consumes-pending`) and (`test:integration:label-channel-overflow-with-journal-reports-journaled-not-enqueued`). Owner shutdown is witnessed three times — at the label surface, through the durable record, and across a restart — by (`test:integration:label-reports-model-owner-shutdown-after-owner-exit`), (`test:integration:label-reports-model-owner-shutdown-with-journal-preserves-durable-record`), and (`test:integration:label-reports-model-owner-shutdown-journal-replays-on-restart`). Identity observation overflow is witnessed by (`test:integration:identity-observation-overflow-is-reported-and-assessments-stay-finite`), and lifecycle churn across a Cholesky recompute by (`test:integration:lifecycle-churn-around-cholesky-recompute-keeps-fast-path-valid`).

The triage the plan asked for — taking each promise nobody had audited and either linking it to existing coverage or confirming it unstarted — was performed by the campaign that retired the matrix, which is why the intents now standing in the test tree's readme are a smaller and better-founded set than the unaudited count this plan once carried. That count is retired with the tracker it was read from.

## The promises no test keeps yet · `sec:assayer:testing-plan-gaps`

**Entry (The feature vector's round trip through a lifecycle event)** · `entry:assayer:gap-assembly-round-trip`

**OPEN.** One dimension map is the sole resolver of every feature index (`dec:vector:sole-resolver`), and the map is tested, and assembly is tested, but the joint property is not: register Sentinels, axes and identity dimensions, build the vector, verify every block stands at its declared index range, put a lifecycle event through, and verify that the map rebuilds and the *same observation* produces the *same features* at the new indices.

The reason to want it as one test rather than as two is that the failure it catches lives between them. A map that rebuilds correctly and an assembler that reads a map correctly can still disagree about which of them re-resolves an index after a width change, and neither component's own tests can see the disagreement.

**Entry (Cross-dimension aggregates under conflicting dimensions)** · `entry:assayer:gap-cross-dimension-conflict`

**OPEN.** The cross-dimension block is a set of max-across-dimensions aggregates (`tab:keyspace:cross-dimension-features`), and no test puts two identity dimensions into conflict: one dimension carrying high suspicion while another carries no alarm at all.

The expectation is joint, which is what makes it worth writing. The aggregate should take the suspicious dimension's value, *and* the per-dimension blocks should both survive intact beside it. A test asserting only the first would pass against an implementation that reached the right maximum by flattening the evidence it maximised over, which is precisely the bug a multi-identity deployment would be hurt by.

**Entry (Calibration buffer weighting at the regime boundary)** · `entry:assayer:gap-regime-boundary-weighting`

**OPEN.** Every calibration record contributes to both regime fits, weighted by its blend weight through a soft transition around the boundary (`constr:platt:buffer`), and the boundary itself sits at a blend weight of $0.3$ (`def:platt:regimes`). No test feeds a population clustered around that boundary and checks that both regimes come out with stable parameters.

This is the population in which the cross-contamination the specification discusses (`disc:platt:regime-transition`) would show itself. Records deep in one regime contribute almost nothing to the other and so cannot exhibit it; records at the boundary contribute substantially to both, and a transition function that is subtly wrong pulls both fits at once rather than either visibly.

**Entry (Interaction lifecycle under competitive cell churn)** · `entry:assayer:gap-interaction-churn`

**OPEN.** Interactions against the competitive set grow and shrink with it, and a competitive cell exiting as another enters is promised (`claim:identity:a-competitive-cell-exits-as-another-enters`). What no promise covers is that the *interaction* features are extended and marginalised in lockstep with those cell events, including the case that makes it hard: several cells entering and exiting inside one lifecycle batch, where a per-event implementation and a per-batch one differ.

**Entry (Outcome axis compression under a scale change)** · `entry:assayer:gap-compression-non-stationarity`

**OPEN.** Each axis carries its own compression scale, adapting on the labels that report it (`def:axis:adaptive-compression`), which means an adapting scale retroactively shifts what historical targets meant. No test feeds a magnitude regime change — axis values stepping up by three orders of magnitude — and verifies the self-correction: a bias proportional to the ratio of the new scale to the old, resolving within the forgetting half-life rather than persisting.

Writing it needs the clock, because the resolution is a claim about a half-life and not about a label count.

**Entry (Convergence beyond the directional smoke)** · `entry:assayer:gap-convergence-depth`

**OPEN.** The learning that makes the system a risk estimator at all is witnessed once, directionally, and the deeper stories are unwritten: that it converges under class imbalance, that a starved class is not starved out, and that the censored asymmetry the foundations single out (`prin:valence:censored-asymmetry`) is handled rather than merely described. This is the widest gap in the corpus relative to what the system is for, and the reason it is wide is infrastructural rather than intellectual — the stories are long-running, and they want the streaming tapes and model-state inspection of (`entry:assayer:harness-stage-tapes`).

**Entry (The recovery residuals)** · `entry:assayer:gap-recovery-residuals`

**OPEN.** The public failure modes are witnessed (`summ:assayer:testing-plan-discharged`); what remains is narrower, more persistence-specific, and mostly below the public surface. Five things: sequence-aware journal truncation under concurrent label writes; journal reopening after a restore at the checkpoint's high-water mark; proof that the ledger, identity, calibration and health payloads all round-trip through a checkpoint rather than the model alone; proof that an identity overflow cannot leak a partially applied competitive-cell lifecycle update; and assertions, crate-level or instrumented, that lifecycle work crossing a Cholesky recompute publishes only internally matched precision, covariance and dimension-map states.

The first three are the durability record's own promises (`dec:durability:checkpoint-journal`) and (`cor:durability:replay-exactness`) taken at their word; the last two are the ones that need instrumentation, because what they assert is the absence of a state that no public surface exposes.

**Entry (The anchor's behavioural coverage)** · `entry:assayer:gap-anchor-behaviour`

**OPEN.** That the anchor's blend is mathematically what it says it is has a witness. That the anchor is *useful* — that it supplies directional coverage while the sister is starved — does not, and it is the claim the anchor exists for. The specification makes the anchor's persistent contribution a design choice rather than a residue (`rem:risk:blend-mechanisms`), with a floor that emerges rather than being declared (`dec:risk:anchor-floor`), so an implementation in which the anchor quietly stopped contributing would satisfy every mathematical test now written and violate the design.

## Reframings the promises want · `sec:assayer:testing-plan-reframings`

**Observation (Sentinel isolation is two promises wearing one name)** · `obs:assayer:reframe-sentinel-isolation`

The promise that a long run of assessments leaves no fingerprint on a Sentinel is stated as a comparison against a fresh Sentinel driven by the identical observation stream with no Assayer attached. That comparison needs a deterministic Sentinel harness that does not exist, and the coverage standing against it today establishes something else entirely — that a snapshot is thread-safe and its version monotone, which is a property of the published state rather than of Sentinel isolation.

The reframing is to split it. That no mutation path into a Sentinel exists in the Assayer's sources is a source-audit question, answerable now, and the boundary is already enforced mechanically rather than by review (`dec:ownership:enforcement`). The behavioural comparison is a separate and later promise that waits on the harness. Splitting them converts one permanently unkeepable promise into one keepable now and one honestly deferred.

**Observation (The mutable-handle promise is a fact about types)** · `obs:assayer:reframe-mutable-handle`

The promise that no mutable reference to a Sentinel can be held is written as though it were an integration test, and it is not: it is a property of the type system, and the package's ownership boundary already fixes it — information crosses in one direction (`dec:ownership:feed-forward`), and the handle that would carry it the other way was considered and rejected (`disc:ownership:handle-alternative`). Report reception takes owned values, so there is nothing to hold.

Two honest treatments exist and the choice is worth making deliberately. Either the promise is discharged by the source audits already standing — (`test:crate:core-model-modules-do-not-import-decision-layer`), (`test:crate:resonance-module-does-not-reference-core-model-state`), and (`test:crate:core-and-api-paths-do-not-call-derivation`) — which are a stronger statement than any behavioural test of the same boundary, since they hold of code that has never run. Or it becomes a compilation-failure test that tries to take the handle and is required not to build. What it must not remain is an integration test, which would assert at runtime a thing the compiler has already refused.

**Observation (The ledger read promise is about depth, not about tearing)** · `obs:assayer:reframe-ledger-atomicity`

The promise that a concurrently updated cell average reads whole or not at all (`claim:ledger:a-concurrently-updated-average-reads-whole-or-not-at-all`) is witnessed by a hundred thousand racing reads against a torn intermediate. That is necessary and it is not the interesting question, because the design prevents a torn value by construction: lock ordering is fixed and total (`dec:ordering:lock-order`), so no reader observes half of one entry.

The question the design leaves genuinely open is one depth up. A write visits every interval containing the coordinate, from the deepest present to the root (`dec:memory:depth-walk`), so the real contract is that a reader cannot observe a ledger in which the deepest layer has been updated and a shallower one has not. That is a multi-entry atomicity assertion, it is not implied by the absence of a torn scalar, and it is what the promise should be reframed to assert.

**Observation (Separation by identity wants a threshold it could fail)** · `obs:assayer:reframe-identity-threshold`

Two entities sharing a Sentinel cell and drawing opposite outcomes are promised to be separated by identity alone (`claim:identity:entities-sharing-a-cell-are-separated-by-identity-alone`), and the expectation is written as discrimination close to one. At the widths this model runs at, five hundred labels split between two entities is around two hundred and fifty per class, which is tight: the figure is plausible if the competitive indicators are sparse and well separated, and it is not something the arithmetic guarantees.

An expectation set that high tests the wrong thing. It passes when the model is working and it also fails when the model is working and the sampling was unlucky, so a failure carries no information. The promise is that identity carries a separation the cell geometry cannot, and the assertion that says so is discrimination significantly above one half, together with the cell's own adverse average sitting near one half — which is the part of the expectation that actually distinguishes the two sources of signal.

## Where the effort goes · `sec:assayer:testing-plan-priorities`

**Register (Priority over the areas)** · `reg:assayer:testing-plan-priorities`

This is the review's ordering of the remaining effort, rewritten against the areas that replaced the divisions it was originally keyed by. It is a judgment and is maintained by hand; nothing generates it and nothing checks it beyond the resolution of its citations.

| Area | Priority | Why here |
| --- | --- | --- |
| (`sec:assayer:area-bayes`) | Highest | This is what the system is for, and it is where the corpus is thinnest against its own purpose (`entry:assayer:gap-convergence-depth`) |
| (`sec:assayer:area-persistence`) | High | A production failure needs a stated behaviour, and the residuals here are the ones no public surface exposes (`entry:assayer:gap-recovery-residuals`) |
| (`sec:assayer:area-risk`) | High | Cold-start safety rests on the anchor, and the anchor's behavioural claim is unwitnessed (`entry:assayer:gap-anchor-behaviour`) |
| (`sec:assayer:area-lifespan`) | High | Deployments change constantly, so lifecycle algebra is exercised in production whether or not it is exercised in tests |
| (`sec:assayer:area-wellness`) | High | Silent degradation is the worst failure mode a monitored system has, which makes the monitoring itself load-bearing |
| (`sec:assayer:area-identity`) | Medium | Well covered per mechanism; the gaps are compositional (`entry:assayer:gap-interaction-churn`) |
| (`sec:assayer:area-channel`) | Medium | Calibration drift is subtle and its boundary population is untested (`entry:assayer:gap-regime-boundary-weighting`) |
| (`sec:assayer:area-feature`) | Medium | The components are covered and the joint round trip is not (`entry:assayer:gap-assembly-round-trip`) |
| (`sec:assayer:area-assess`) | Medium | Performance is a structural constraint here rather than a preference, so a regression is a contract break |
| (`sec:assayer:area-numerics`) | Medium | Strange inputs are rare and destructive, and the scale-change story is open (`entry:assayer:gap-compression-non-stationarity`) |
| (`sec:assayer:area-resonance`) | Low | Already well covered at the crate level against the closed-form derivation |
| (`sec:assayer:area-ledger`) | Low | The routing invariants are covered in depth; what remains wants the clock (`entry:assayer:harness-stage-clock`) |
| (`sec:assayer:area-audit`) | Low | The source audits already carry the boundary claim, more strongly than a behavioural test could |

**Observation (What the priority table lost, and to what)** · `obs:assayer:priority-table-provenance`

The table above is the surviving third of the review's original. Two columns retired rather than migrating, and both retired into something that computes them.

The coverage counts — how many promises in each grouping were kept and how many partly — were the tracker read sideways, and they are the coverage report's answer now. The consequence is worth being explicit about: this plan can go stale in its judgment, which is what a judgment is for, but it can no longer go stale in its arithmetic, because it states none.

The risk column — what is lost if this grouping goes untested — turned out to be the area register's subject rather than the plan's (`req:testdocs:area-register`). One document saying what an area is for is better than one document saying it and another restating it as a risk, and the register is where a claim's area sends a reader anyway.

## Process · `sec:assayer:testing-plan-process`

**Entry (A runtime budget for the enrichment grid)** · `entry:assayer:process-grid-runtime`

**OPEN.** The multi-channel boundary suite sweeps enrichment axes orthogonally to policy axes, and the product is large: roughly forty families against twelve enrichments. Each case builds a full world, registers entities, runs assessments and derives profiles, so the suite's cost is a product of a per-case cost that setup changes can move by an order of magnitude without anybody noticing.

What is owed is a measured budget and a tolerance, recorded where the suite is defined rather than here, so that a grid that grows or a setup that gets heavier is a finding rather than a slowly worsening continuous-integration time. The measurement is the work; the number is not this document's to invent.

**Observation (The mechanism-smoke tier has nowhere left to sit)** · `obs:assayer:process-smoke-tier`

The review proposed a third tier between *partial* and *kept*, for a promise whose primitives are all separately witnessed while the composition is not, so that such a promise would stop reading as though its building blocks were doubtful.

The proposal is retired rather than carried, because the two tiers it sat between no longer exist. A statement is now witnessed or it is an intent, and the distinction the tier wanted to draw is available without a tier: a composite promise whose parts are separately promised says so in its own sentence, and several of the intents standing today do exactly that, naming the mechanisms promised elsewhere and claiming only that they compose. That is the tier's content, written into the statement instead of into a status column, and it survives the retirement of every status column.

**Caveat (What this plan does not audit)** · `cav:assayer:testing-plan-limits`

Two limits. This plan judges what is missing and what is misframed; it does not judge whether a test that exists proves what its statement says it proves. That is a conformance question, and the authoring campaign that wrote the statements left a catalogue of vacuous, misnamed and oracle-free tests for it to start from.

And the entries here are proposals, not designs. Each says what promise is unkept and what a witness would have to establish; none fixes the fixture, the tolerance, or the label count, because those are the writing of the test and are settled against the machinery as it is at the time rather than as it was when the gap was noticed.

## The harness roadmap · `sec:assayer:testing-plan-roadmap`

This roadmap arrived from the retired scenario matrix, which carried it because nothing else did. It is planning rather than indexing — staging, dependency ordering, effort, and which piece of infrastructure unblocks which untested promise — so the plan absorbs it and keeps it alive. The promises it gates are named by the claims that now carry them, so a reader can follow a gate to the statement it holds up rather than to a number.

**Entry (Stage one: the skeleton)** · `entry:assayer:harness-stage-skeleton`

**DONE.** The `World` and `WorldBuilder` pair with a deterministic `TestRng`, a name registry, a tolerances bundle, the `World::cold()` convenience, lifecycle, assess, derive and label verbs, the deterministic `World::cycle_on(...)` and `World::cycle_default(...)` round-trip helpers, `World::flush_labels()` for post-label observation, the `LabelSpec` builder, and the domain-aware assertions including `assert_health_clean` and `assert_risk_basis_near`. Test-local helpers in `support/mod.rs` centralise the channel policy and reward builders and the repeated same-core-moved-decision-layer assertions.

The clock seam landed here too — the `Clock` trait, `SystemClock`, `VirtualClock` and the `WorldBuilder::clock(...)` setter. Only the engine-side wiring that lets time travel influence decay is deferred to the stages below.

**Observation (The drift budgets named the wrong clock)** · `obs:assayer:drift-budgets-misattributed`

Every drift budget in `tests/support/constants.rs` attributed its spread to the `PersistentTimestamp::now()` reads and promised exactness once those were injected. Measurement contradicted that. At the commit where the conformance campaign opened, a cold parallel run of the suite failed ten cells of `multi_channel` and every failing cell was a reported-Sentinel one; no unreported cell failed in any run.

The partition is the proof. An unreported request has no arrival stamp, so its report staleness is exactly zero every time; a reported one measures elapsed time between ingestion and the assessment's batch reading. Staleness enters the feature vector as the logarithm of one plus the elapsed seconds, which the statement over the test establishes (`claim:extract:report-staleness-enters-as-the-log-of-one-plus-the-elapsed-seconds`) — and which near zero is very nearly the elapsed time itself, so a millisecond of extra gap moved `p_bad` by about a thousandth — the largest divergence observed, and some six orders of magnitude more than the hourly decay could account for.

Both of those readings are in the intra-process monotonic domain, not the persistent one. The stage as originally written would therefore have left the widest budgets exactly where they were, which is why it is split below: the two domains (`dec:clock:two-domains`) keeps apart turn out to need separate injections, and only one of them was ever the flake's cause.

**Entry (Stage two, monotonic: staleness stops depending on load)** · `entry:assayer:harness-stage-clock-monotonic`

**DONE.** `Clock` reports both domains from one handle, `AssayerBuilder::clock(...)` sets it with `SystemClock` as the production default, and `World` passes its `VirtualClock` through. The monotonic readings that decide report staleness — the batch head in `assess()` and the arrival stamp in `ingest_report()` — come from that clock, as do the label path's head-of-path readings and the identity maintenance thread's decay interval. `Instant` cannot be synthesised, so the virtual monotonic reading is a real instant captured once at construction plus the accumulated offset.

What this bought is measured rather than assumed. The staleness class of divergence is gone: no reported-only, load-scaled failure has recurred. What it did not buy is exactness. With every budget set to zero the suite still fails, and the surviving spread has two shapes — a floating-point residue at the scale of a few units in the last place, and a coarser residue in the reported cells that alternates between two stable values rather than jittering. The second shape is an ordering race, not a clock reading, and it is left to (`entry:assayer:harness-stage-clock-residual-race`). The budgets therefore stand, now justified by what actually moves them.

**Entry (Stage two, persistent: the decay-driven surface)** · `entry:assayer:harness-stage-clock`

**PARTLY DONE.** The persistent domain shares the seam the monotonic injection built, and the sites on the assessment and label paths and the checkpoint stamp already read through it. What remains is the rest of the production `PersistentTimestamp::now()` call sites — a dozen of them rather than the thirty this entry once claimed, the larger figure having counted in-file test modules — together with `World::advance` and `World::travel_to`.

This entry retires no drift budget; that expectation belonged to the monotonic half and is settled above. What it gates is the decay-driven surface, where a stale adverse history must dissolve on the clock alone (`claim:ledger:a-stale-adverse-history-dissolves-without-fresh-labels`), the three decay horizons must stay separate (`claim:ledger:three-decay-clocks-run-independently`), the two decay indices must compose (`claim:numerics:label-indexed-and-time-indexed-decay-compose-multiplicatively`), and an entity moving between regions must stay tracked across the handover (`claim:identity:an-entity-that-relocates-is-tracked-across-the-move`). It unblocks restore timing, both the continuation promise (`claim:persistence:a-restored-instance-continues-the-run-it-was-cut-from`) and the once-only discounting beside it (`claim:persistence:restore-time-decay-is-applied-exactly-once`). It unblocks the clocked challenge and uncertainty statements — belief sharpened by a trickle that stops short of a usable estimate (`claim:risk:a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate`) and inflation by the staleness factor alone (`claim:risk:silence-inflates-reported-uncertainty-by-the-staleness-factor`) — and the contamination-timing statement (`claim:risk:a-measurement-only-feature-contradicts-stale-history`). Beyond the intents, it gates pending-buffer expiry and the full-purity upgrade of repeated derivation, both of which have witnesses today that stop short of time travel. It also gates the scale-change story of (`entry:assayer:gap-compression-non-stationarity`), whose resolution is a claim about a half-life.

**Entry (The residue the clock does not explain)** · `entry:assayer:harness-stage-clock-residual-race`

**DONE.** The question this entry posed — which of the two candidates the residue is — has an answer, and the answer is both. They were separable by measurement, they have different causes, and only one of them was the harness's to fix.

The instrument was a snapshot-version trace: every per-request snapshot load and every steward command logged its version, so a failing run could say which publication landed where. Runs were compared cell by cell against runs where the same cell passed. Nothing about the diagnosis rests on which mechanism sounded more likely.

The first mechanism was the harness's, and it is fixed. Registration is fire-and-forget on the engine side, so a scenario that registered an outcome axis and then assessed had no guarantee the assessment saw the axis. The trace caught it exactly: two runs whose steward histories were otherwise identical differed only in which snapshot the first assessment read — the one before the axis registration, or the one after. That decided the feature vector the first label trained on, and the two trajectories never reconverged, which is why the divergence presented as two stable values rather than as jitter. Every lifecycle verb on `World` now ends on a publication barrier, and the whole cross-world family that this mechanism dominated — the lifecycle-sufficiency and divergent-outcome cells — has not failed once since.

The second mechanism is not the harness's and not a defect. The identity maintenance thread cycles on its own interval, drains the coordinates the assessment path feeds it, and installs competitive-set changes into the identity infrastructure. Those installations can land between the requests of one batch, and the cross-channel core assertions expect a batch to be a unit of view. (`dec:ordering:batch-timestamp`) says in as many words that it is not: the batch shares a clock and does not share a view. The trace shows the signature that distinguishes this mechanism from the first — three requests of one batch answered against one and the same snapshot version and still disagreeing, because what moved between them lives beside the snapshot rather than in it. Parking the maintenance interval turned six provoked failures out of six into six passes out of six, which is the whole of the proof.

What remains is therefore an expectation to repair rather than a race to close, and it is (`entry:assayer:mid-batch-identity-movement`), which records what became of both the expectation and this reading of the second mechanism.

**Entry (A batch is not a unit of view)** · `entry:assayer:mid-batch-identity-movement`

**DONE.** The expectation this entry named is gone, and the failure it was blamed for turned out to have another cause.

The expectation went first. Nothing in this suite batches requests and expects them to agree any more: a cross-channel comparison holds one assessment and derives every channel from it, which is the arrangement the public surface was designed for and the one the design does promise, the other still being declined (`dec:ordering:batch-timestamp`). That is the second of the two repairs this entry set out, the smaller and honest one, and taking it meant the first was never needed — no test-time block on the maintenance loop was added, and no engine surface moved.

The cause went second, and it took a measurement. What still failed under load after the family stopped batching were the two cross-world matrix witnesses, the lifecycle-sufficiency and divergent-outcome challenge-and-reward families, which deriving once cannot reach at all: their compared quantity is one world against another, so there is no single assessment to hold. On 2026-09-02, sixteen concurrent instances of the `multi_channel` binary at sixteen test threads on a 112-core host: twelve of sixteen instances failed, every failure in those two tests and every one of them the cross-world replay of the risk basis, deltas spread continuously from 1.6e-8 to 1.1e-6 with one resonance q at 1.3e-2. The three tests in the same filter that derive once failed in neither arm, which is the control. Settling both worlds before anything makes them differ — the barrier the paired trainers in the same file already carry and argue for — took that load to sixteen of sixteen green, and then the whole binary green at sixteen instances with every cross-world budget tightened to the floating-point scale.

The mechanism is therefore the standardisation ramp and not the maintenance thread. Two worlds compared while still transitioning stand at different accepted counts, because the ramp advances on accepted observations and the steward applies them asynchronously, so the difference is the scheduler's and the comparison was reading it (`inv:guarantee:evidence-authority`). The earlier attribution was made when the two candidates were not separable by any measurement then in hand; it does not survive the before-and-after pair. Nothing here denies that the maintenance thread installs competitive-set changes between requests — the design permits it and says so — only that no witness in this suite now rests on the question.

The budgets this entry held open came down with it, as it said they should, and the zeroed-budget experiment it asked for was the run above: the cross-world witnesses hold at the floating-point residue their narrow siblings use, and the seven wider constants are retired. Two request-order budgets outlived that pass, because both orders deriving from one held assessment only resembled the same retirement and a budget is not tightened on a resemblance. They were measured afterwards on the same shape and they are retired too. On 2026-09-02, sixteen concurrent instances at sixteen test threads on the same host, twice over: the two constants zeroed first to the floating-point floor their narrow siblings use and then to the purity floor a decade below that, with the whole `multi_channel` binary and the whole `assess_integration` binary run under each arm, and the shipped widths run beforehand as the control. All three arms were green in every instance of both binaries, so the window the gloss named — the forward and reordered batches being separate calls, with a publication landing between them — covered nothing that is on the path. Nor is it on the path for the batch-versus-singleton witness in `assess_integration`, the one site that does issue separate calls: it is handed three worlds whose ramps already stand at the horizon. The budgets therefore left rather than shrank, and the three-armed selector that chose among the widths left with them — every request-order site names the purity budget directly.

The helper tier those budgets were the reason for went with them. Eleven request-order helpers in the integration suite's derivation support existed to carry a caller's budget down to the comparison — a drift-aware variant of each order-replay entry point, and the comparison-mode helper only they reached — and once every site named the purity budget directly, none of them had a caller in any test binary. They are deleted, and the one order-replay primitive still in use holds the purity budget in its own body instead of taking it as an argument. A budget wide enough to be worth choosing is a claim about the shape under test, and a claim of that kind belongs in a measurement and a named constant, where the next reader finds what settled it, rather than in an argument a call site can widen without saying why.

The seam the tier left behind closed with it. What the two batch comparators took as a drift budget, every live caller filled with the purity budget, and the parameter was kept as somewhere a caller with a measured reason to widen it could say so. No caller ever had one, and the reading that made the seam plausible — request-order probes carrying live Sentinel reports and outcome predictions being wider than floating-point residue — is the one measurement retired. Both comparators hold the purity budget in their own bodies now, the full-profile one and the one that drops to the sufficient statistic alike, so the two arms of a single comparison can no longer be read as answering to different budgets. The wider drift-aware helpers around them keep their parameters, and should: the lifecycle-sufficiency and divergent-outcome budgets reach them at live sites, and there the width is a caller's to choose because it is a fact about which two things are being compared.

**Entry (Stage three: tapes, fixtures, and health accessors)** · `entry:assayer:harness-stage-tapes`

**OPEN.** `LabelTape` and `ReportTape`, `World::converged()`, snapshot diffing, persistence fixtures, failure-injection knobs, and domain-specific assertion builders. This is where the long-streaming stories land: a hibernating Sentinel returning with its own weights (`claim:lifespan:a-hibernating-sentinel-returns-with-its-own-weights`), restore after journal replay (`claim:persistence:a-restored-instance-continues-the-run-it-was-cut-from`), drift detection under a moving base rate (`claim:wellness:a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration`) with the two-window divergence beside it (`claim:wellness:recent-discrimination-parts-from-the-lifetime-figure-under-drift`), and the restriction gap between the operational and sister estimates (`claim:risk:restriction-opens-a-gap-between-operational-and-sister-estimates`). The anchor-regime freeze, the concurrent Schur recompute, and the convergence milestones want the same fixtures.

**Entry (Property-based invariant packs)** · `entry:assayer:harness-property-packs`

**OPEN.** Randomised sweeps for the mathematical invariants that are broader than any one story: posterior positive-definiteness after rank-one updates and after marginalisation, Kraft equality preservation, summation invariants for routed feature assembly, monotonicity of an entry under pure decay, and randomised crossover matching beyond the ten-thousand-sample sweep the geometry tests already run. These should attach to existing mechanism claims rather than introduce one-off narrative tests.

**Observation (What survives of the integration-slice map)** · `obs:assayer:harness-integration-slices`

The roadmap used to carry a map from each integration file to the division it served. That map is superseded and was already inaccurate when it was written: the study behind the retirement found the one-file-per-division intent broken for six of the seventeen files, with the worst offender among the newest. Which tests a file holds and what each establishes is now generated — every source carries an index of its own tests, and every folder a matrix of the folder's — so the map has a replacement that cannot drift.

What survives from it is the instruction, which is not derivable and still holds: tests prefer the `World` harness, and escape-hatch access to internals is flagged where it is taken.

## Concordance · `sec:assayer:testing-plan-concordance`

**Decision (The fan-out is the concordance)** · `dec:assayer:testing-plan-record-concordance`

No concordance from the retired numbering to the landed record set is written down, here or anywhere. A retired number is not an identity with an heir — where a stray reference is met, the sentence is resolved against the promise it actually leans on and rewritten to cite the landed head that states it, exactly as this plan did for its own five.

The evidence that forced the question stands as the ruling's ground. The plan as found named five records by retired numbers. Three had one landed record apiece and were rewritten to cite it. The other two — ``ADR-L-160`` and ``ADR-L-170`` — were each absorbed by four landed records: the persistence record's live decisions rehomed across durability, substrate, retention and construction, and the error-model record's across degradation, posterior, ordering and memory. No mapping was available by inspection, and the ruling confirms none is to be invented: the sentences that named them cite the landed heads that state the promises directly (`dec:durability:checkpoint-journal`) and (`dec:degradation:error-partition`), and that sentence-by-sentence resolution is the concordance, complete in itself.

This was the third of the three questions the plan's audit left open (`q:assayer:testing-audit-questions`); all three now stand answered.

## Harness declarations · `sec:assayer:harness-declarations`

**Decision (The harness is a host, and a host declares)** · `dec:assayer:harness-declares`

Some of the construction contract's parameters are host-set with no default, and a construction that never declared one refuses to build (`tab:construction:parameters`). The harness builds instances, so the harness is a host, and it declares like any other.

What that table warrants is the obligation and not the choice. A row saying a host must declare a capacity is the reason there is a figure at all; it is not the reason the figure is sixty-four. A harness value citing only that row would be citing a head that does not fix it, and this section exists so that none has to: every figure the harness declares appears below as a row with its reason, and a harness figure with no row here has no warrant.

The reasons are not all of one kind, and the rows say which kind each is. Some read a bound the specification states. Some preserve a figure a retired literal shipped, so that the behaviour under test stays the behaviour the suite has always exercised. Some are the harness's own choice among many that would serve equally. The third kind is not a defect — a harness that claimed a derivation for every epsilon would be stating more than it knows — but it is only honest while the row says so, and a row that cannot say even that much is a figure standing without a reason and is recorded as one.

**Table (The construction figures the harness declares)** · `tab:assayer:harness-construction-figures`

| Figure | Declared in | Value | Why this figure |
| --- | --- | --- | --- |
| Command-channel capacity | ``src/testing/construction.rs`` | 64 | The figure the retired literal shipped, kept so that the behaviour under test is the behaviour the suite has always exercised. Moving it would move every test that depends on back-pressure without any test having asked. A deployment makes its own declaration and inherits nothing from this one. (`const:assayer:harness-command-channel-depth-count-64`) |
| Model-owner channel capacity | ``src/testing/maintenance.rs`` | 256 | A harness-side depth with no counterpart in a deployment. It exists so that a test draining the receiver only after the loop has run never blocks the maintenance thread against it, so what it must be is wider than any single test's emission. Chosen wide rather than derived. (`const:assayer:model-owner-channel-depth-count-256`) |
| Default test-clock epoch | ``src/testing/virtual_clock.rs`` | 2026-01-01T00:00:00Z | The properties are the requirement: round, stable, comfortably inside the valid timestamp range, and far enough from its end that a scenario projecting months forward does not overflow. The instant carrying them is the harness's own choice among many that would, and the code holds it as whole seconds from the Unix epoch. (`const:assayer:harness-clock-epoch-seconds-1767225600`) |

**Table (The scenario tolerances the harness declares)** · `tab:assayer:harness-scenario-tolerances`

Every field of the default tolerance bundle, what it bounds, and whether its ceiling is a figure the specification states, the harness's reading of a formula the specification states, or the harness's own. The bundle is one constant, ``DEFAULT_TOLERANCES``, and its fields pin together as a single form (`const:assayer:scenario-tolerance-defaults-form-x57224f5b`); no field below pins on its own.

| Field | Value | What it bounds | Standing |
| --- | --- | --- | --- |
| ``default`` | 1e-9 | Generic floating-point comparison of one-shot quantities | The harness's own. A few operations' worth of reassociation, with no accumulation assumed. |
| ``ewma`` | 1e-6 | EWMA values — bad rate, compressed outcome | The harness's own, and looser than ``default`` because an EWMA is a long product of decay factors rather than a one-shot computation. |
| ``auc`` | 5e-3 | Agreement between two AUC readings of one sample | The harness's own. It bounds the recent and aggregate readings where the whole sample fits inside the recent window and the two are therefore the same statistic over the same rows; the departure cut the instability flag is defined on is a different figure and is not this one. |
| ``crossover`` | 1e-6 | Crossover-matching | The harness's reading. The specification states crossover matching as a condition to hold, not as a numeric tolerance, so the ceiling is the harness's and not a figure it took. |
| ``precision_sync_per_dimension`` | 1e-6 | The synchronisation error, the Frobenius norm of the departure of the precision matrix and its maintained inverse from the identity — per unit of model width | A figure the specification states. The threshold is the coefficient times the dimension (`def:monitoring:synchronisation-error`), so the field is the coefficient and the ceiling is taken from it at each assertion's own width; the flat 1e-4 it replaced equalled the specified threshold only at a width of one hundred, which no consuming test exercises. |
| ``standardisation`` | 0.14 | Mismatch between store-time and label-time standardisation | A figure the specification states: the bound for a typical position at a labelling latency of fifty (`bound:standardisation:restandardisation`). The one field here that is a specification figure rather than a reading of one, and the test that uses it sets up that regime deliberately. |
| ``platt_log_sharpness`` | 0.06 | Platt sharpness drift between successive refits, as relative log-sharpness | The harness's own. It bounds the absolute difference of the two log sharpnesses, which is the proportion the sister's sharpness moved by, rather than the distance it moved: sharpness is a scale parameter, so an absolute bound decides differently at every sharpness and the figure it replaced discriminated only near this fixture's own optimum of about two. The pipeline scenario that consumes it proves deterministic repeatability of the periodic ladder and nothing wider — its first two refits end on whole repetitions of the same forty-row cycle in the same order, so the recency weights multiply each rung's accumulated weight by a common factor and the optimum is bit-identical. That is fixture geometry, not a property of stationary workloads in general, and it is not evidence that the bound never binds: a stationary population fixes the distribution and does not make two finite samples identical. The two-sided discrimination is put in the fitting module's own scenario, where a four-percent scaling of every score is admitted and a ten-percent one rejected — the same proportional moves at any optimum, because the comparison is proportional (`cav:assayer:harness-unconsumed-tolerances`). |
| ``bit_identical`` | 0.0 | Exact equality, for bit-identical snapshots | Not a tolerance at all. It is a field so that such a test reads as a named requirement rather than as a bare zero among real epsilons. |

**Decision (The golden report's readings are stimulus the harness authors)** · `dec:assayer:golden-report-stimulus`

The golden report is a hand-built Sentinel report at depths zero, four and eight, with a depth-twelve cell in the four-cell variant, and one coordinate whose prefixes land in every cell it declares (``src/testing/reports.rs``). Its readings — a peak z-score and an accumulator on each of the four scoring axes at each depth — are arguments to the builder that assembles the report. They are what the model is shown, never what the model returned: nothing computes them, nothing regenerates them, and no procedure in this package would reproduce them if they were deleted.

That is worth stating because a fixture of pinned figures ordinarily earns its warrant the other way, from the procedure that regenerates it, and a reader who assumes so here would look for a regeneration step that does not exist and conclude the figures were derived and the derivation lost. They were chosen, and the warrant is the shape they were chosen to have.

The shape is discrimination. The extraction layer presents each axis to the model as several views over the same ancestry — the deepest entry, the shallowest, the per-axis maximum, the mean, the deepest-minus-shallowest gradient and the spread about the mean — and absolute indices are the model's only names for its inputs, so what the fixture must catch is a layout that reads the right number from the wrong slot. Readings that agreed across depths or across axes would let such a mixup pass. So the ladder rises with depth on novelty, displacement and coherence and the readings differ across axes at a depth, while surprise is deliberately held flat so that the gradient and spread slots over it read exactly zero and a foreign axis leaking into one of them shows up as a number where zero belongs.

Within that shape the particular figures are the harness's own. Nothing distinguishes a depth-eight novelty of 2.8 from one of 2.9, and the rows below say so rather than dress the choice as a derivation. What the baseline does earn is a reason: every axis at every depth carries a baseline of mean zero and unit variance, which is why one figure serves as both a reading and its z-score and why the expected feature values can be computed by hand from the table.

**Table (The golden report figures the harness declares)** · `tab:assayer:harness-golden-figures`

Every figure the golden fixtures declare, in the axis order novelty, displacement, surprise, coherence. All nine are declared in ``src/testing/reports.rs`` and pin under the form program.

| Figure | Value | Why this figure |
| --- | --- | --- |
| Routing coordinate | ``0x1234_5678_9ABC_DEF0_1234_5678_9ABC_DEF0`` | Its leading four, eight and twelve bits are the three cells' lower bounds, so one coordinate routes through every cell the scenarios declare. Any pattern with that property would serve equally; this one is legible as a descending run of digits. (`const:assayer:golden-route-coordinate-form-x8ec0eee8`) |
| Root peaks | ``[0.5, 0.3, 1.0, 0.0]`` | The quiet end of the ladder, with a coherence of zero so the gradient over that axis is the whole of its rise. Distinct across the four axes so a mixup within the root view is visible. (`const:assayer:golden-root-peaks-form-xcef69bea`) |
| Depth-four peaks | ``[1.2, 0.7, 1.0, 0.4]`` | An interior rung: above the root and below the cell on every axis but surprise, which is held. Interior values are what separate a mean or a spread view from either endpoint. (`const:assayer:golden-depth-four-peaks-form-x1bb8a3e0`) |
| Depth-eight peaks | ``[2.8, 1.5, 1.0, 0.9]`` | The loud end of the three-cell report, and the deepest entry its views read. The rise from the root is large enough that a gradient reading is unmistakable rather than a rounding difference. (`const:assayer:golden-depth-eight-peaks-form-x731285e9`) |
| Depth-twelve peaks | ``[3.2, 1.8, 1.1, 1.0]`` | The four-cell variant's added rung, continuing the ladder. Surprise steps once here so the acceptance scenario has an axis that is flat over three depths and not over four. (`const:assayer:golden-depth-twelve-peaks-form-xb3bbee46`) |
| Root accumulators | ``[0.1, 0.0, 0.5, 0.0]`` | A second ladder on the same axes, and no fixed multiple of the peaks at any depth, so a view that confused the two families does not land on a matching number. (`const:assayer:golden-root-accumulators-form-x9c0721f9`) |
| Depth-four accumulators | ``[0.3, 0.2, 0.5, 0.1]`` | The interior rung of the accumulator ladder, and the first depth at which all four axes differ. (`const:assayer:golden-depth-four-accumulators-form-x13adce1c`) |
| Depth-eight accumulators | ``[0.7, 0.4, 0.5, 0.3]`` | The deepest accumulator reading of the three-cell report, with surprise still held at its root value. (`const:assayer:golden-depth-eight-accumulators-form-x72d581d8`) |
| Depth-twelve accumulators | ``[0.9, 0.5, 0.6, 0.4]`` | The four-cell variant's rung, with the single surprise step matching the peaks' so both families describe the same scenario. (`const:assayer:golden-depth-twelve-accumulators-form-xef1bf165`) |

**Caveat (Two declared tolerances no assertion consumed)** · `cav:assayer:harness-unconsumed-tolerances`

``auc`` and ``platt_log_sharpness`` were declared, given values, and read by nothing; the search was over every assertion in the package and found no consumer of either. Of the two honest repairs — give each field the assertion its name promises, or delete it and let the field return when a test wants it — the first was taken, on the ground that the assertion each name already promises is one the suite can write without inventing a bound for the number. Each now has one, and the standings above are what those assertions earn rather than what the names promised.

The caveat is kept because the shape it named is the one worth recognising, and because the same wave found it twice: a configurable no consumer reads is the same failure as a tolerance no assertion consumes (`cav:construction:registration-timeout-shadowed`). A figure no test reads cannot be defended by what the test does with it, so it cannot be warranted at all — and the temptation at that point is to invent a bound for it, which is exactly the laundering the table above exists to prevent. What the repair had to avoid was writing an assertion around the number; each of the two got the assertion its name already promised, and the numbers were left where they stood.

One of the two is consumed without yet being load-bearing, and the reason is a fact about the harness rather than about the tolerance. The pipeline fixture gives every label the same risk basis, so every calibration row carries the same linear predictor and the buffer is separable at any sharpness; the fit consequently pins at its search ceiling of a hundred, both refits return that same figure, and the observed drift is exactly zero. The bound would catch a second refit that moved off the ceiling, which is a real guard, but the magnitude decides nothing until the fixture gives the two classes overlapping scores. That is a calibration-fixture job and it is named here rather than solved by tuning a workload backwards from the answer, which would be the laundering again in another form.

**Convention (A golden reading's pin moves whenever the reading does)** · `conv:assayer:golden-pin-churn`

Each of the nine figures pins under the form program, whose slug is taken over the value's own source text, so editing any one reading mints a new pin for that figure. The pin is not a name the fixture keeps; it is a statement about which fixture, and it is expected to churn. The metric catalogue's pin behaves the same way for the same reason (`conv:metrics:pin-churn`).

The churn is the mechanism and not a cost of it. These figures are stimulus rather than derived truth (`dec:assayer:golden-report-stimulus`), so the usual protection — recompute and see whether the answer still holds — is unavailable here: a reading may be edited to anything and the fixture will still build. What stands in its place is that a document or a comment depending on the exact readings binds itself by citing the pin, and when the reading moves that citation names a label nothing mints any more and fails. The dangling citation is the intended outcome, and it is the only thing that makes somebody re-read the dependency at the moment the dependency stopped being true.

A legitimate change to the fixture therefore churns pins by design, and a wave that finds itself re-minting several of them has not hit a defect. What the churn must not become is a reason to avoid editing the fixture, or a figure quoted for its own sake.

**Caveat (Three places the golden shape does not discriminate)** · `cav:assayer:golden-discrimination-limits`

The ladder catches a layout mixup only where its readings actually differ, and there are three places they do not. Each is stated because the decision above rests the whole warrant on discrimination (`dec:assayer:golden-report-stimulus`), and a discriminating shape with undeclared blind spots claims more than it delivers.

Surprise is flat across depths zero, four and eight, so over the three-cell report the deepest, shallowest, maximum and mean views of that axis all read the same figure and the gradient and spread views both read zero. Holding it is deliberate and buys the zero-valued slots the decision describes; what it costs is that the surprise axis alone cannot distinguish those four views from one another. The four-cell variant's single step at depth twelve is what separates them.

The root accumulators carry zero on both displacement and coherence, so at that depth those two slots hold the same figure and a swap between them is invisible. No other depth in either family has a repeated reading.

Because every ladder rises with depth, the deepest-entry view and the per-axis maximum view coincide on every axis of both families, and the golden fixture alone cannot tell a maximum from a last entry. That gap is covered elsewhere rather than left open: a separate case puts the loudest reading at an interior depth, where the two views must disagree (`claim:extract:the-maximum-view-can-be-carried-by-an-interior-depth`).
