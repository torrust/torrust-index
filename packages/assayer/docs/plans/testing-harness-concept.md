# A concept for the Assayer's testing harness · `plan:assayer:testing-harness-concept`

_What the package's testing scaffolding should become, argued from what the forty-four unkept promises actually ask of it. The harness as it stands is described elsewhere and is not restated here (´rep:assayer:testing-architecture´); the standing roadmap that stages its growth is elsewhere too (´sec:assayer:testing-plan-roadmap´). This document is an opinion: it says what the harness should be, what that costs, what it rejects, and how a reader would know the harness had arrived._

The forty-four plans are the design input, and they are an unusually good one. Each was written against the machinery as it is, each names the fixture work it needs before its witness can exist, and each sizes that work. Taken one at a time they read as forty-four independent shopping lists. Taken together they are a specification for a harness, because the same ten things are asked for over and over in slightly different words, and the differences are almost never load-bearing. The concept below is what those ten things look like when they are built once instead of forty-four times.

The argument has one shape throughout. A demand that recurs across many plans is infrastructure whether or not it is built as infrastructure; the only question is whether it is written once in the harness with a stated contract, or many times in tests with a contract nobody wrote down. The second option is not cheaper — the plans' own estimates already total more harness-shaped work than the concept below — and it is strictly worse, because the divergences between copies are invisible until two tests disagree about a barrier and one of them is wrong.

## What the forty-four plans demand · `sec:assayer:testing-harness-concept-demands`

Every plan carries a *What the code offers today* section and a *What is missing* section, and between them they say precisely what harness the plan is short of. One hundred and twenty-three `Entry` heads stand across the forty-four. Classified by the harness capability each depends on, they collapse to ten recurring demands, and forty of the forty-four name at least one.

**Table (The recurring demands and how many plans carry each)** · `tab:assayer:testing-harness-concept-recurring-demands`

| Demand | Plans | What is being asked for |
| --- | --- | --- |
| A read-only state probe | 22 | A test-support-gated projection of engine state — a model block, a published snapshot slot, a precision spectrum, a Ledger rate, a pending entry, a blend component — returning owned copies with no mutation path and no clock read of its own |
| A construction input the builder does not carry | 20 | `WorldBuilder` reaching the whole construction contract: interaction templates, a chosen competitive-cell set, a configured identity contract — the reference layout is not reproducible from the public scenario path today |
| An independently computed oracle | 17 | The asserted quantity recomputed by a second route — a dense Schur complement, a decay recurrence, a width formula, a slot moment, an analytic gap — and compared under a named budget |
| A declarative stream | 13 | An ordered row list played back with a stated barrier policy, checkpoint callbacks, and a progress increment, rather than a hand-written loop per test |
| A fixture that guards its own precondition | 13 | A fixture that refuses to return a value until the state it exists to establish is established, so non-vacuity is the fixture's obligation rather than each caller's |
| A rank or discrimination statistic | 12 | A tie-aware pairwise fold with class gates, computed locally rather than read back from the surface under test |
| A persistence fork | 11 | Checkpoint, quiesce, copy, rebuild from an isolated image on the same injected clock, and compare live against replayed |
| Scenario-level time verbs | 9 | Time travel reached from the scenario rather than through the clock handle, discharging the open persistent-domain stage (´entry:assayer:harness-stage-clock´) |
| A new completion barrier | 6 | A barrier for the identity-maintenance loop, which no present verb covers |
| A specification conflict resolved first | 6 | A promise whose stated figure contradicts the record, the implementation, or a standing witness, and which no fixture can rescue |

Two numbers in that table are the design's strongest evidence and deserve saying plainly. Twenty-two plans want a state probe and twenty-one of them describe it in the same five clauses — cross the right barrier, take one published load, clone the fields, expose no mutation, read no clock — which is a contract that has been written twenty-one times and declared zero times. And six plans want a completion barrier for the identity-maintenance loop, of which three name it `World::flush_identity`, `World::flush_identity_maintenance` and `World::settle_identity` respectively while two more describe the same command sequence without naming a verb at all. Four names for one barrier is what a missing declaration looks like from the inside.

The four plans naming none of the ten are the control, and they are instructive rather than anomalous: they are the promises whose witness is a compile-failure case, a source audit, a single-line citation repair, or a test-local sampler over surfaces that already exist. A harness concept that cannot say which promises it is irrelevant to is not making a claim.

**Table (Each promise and the harness capabilities its plan depends on)** · `tab:assayer:testing-harness-concept-plan-demands`

| Promise | Demands |
| --- | --- |
| (´plan:assayer:intent-a-competitive-cell-exits-as-another-enters´) | probe, builder, oracle, barrier, persistence |
| (´plan:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause´) | builder, rank |
| (´plan:assayer:intent-a-concurrently-updated-average-reads-whole-or-not-at-all´) | none — a citation repair on a standing witness |
| (´plan:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation´) | specification conflict |
| (´plan:assayer:intent-a-divergent-starved-cell-surfaces-when-labels-are-requested´) | oracle |
| (´plan:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights´) | probe, oracle, stream, persistence, guard |
| (´plan:assayer:intent-a-measurement-only-feature-contradicts-stale-history´) | probe, oracle, time, persistence |
| (´plan:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale´) | probe, builder, oracle, rank, guard |
| (´plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it´) | probe, stream, guard, specification conflict |
| (´plan:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from´) | probe, builder, stream, barrier, time, persistence, rank |
| (´plan:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels´) | oracle, time, persistence |
| (´plan:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk´) | probe, stream, specification conflict |
| (´plan:assayer:intent-a-systematically-mislabelled-entity-parts-from-the-aggregate-within-ten-labels´) | rank, specification conflict |
| (´plan:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration´) | stream |
| (´plan:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate´) | builder, time, persistence |
| (´plan:assayer:intent-a-wide-pool-churning-at-random-keeps-its-width-and-its-inverse-pair´) | oracle, persistence |
| (´plan:assayer:intent-an-axis-nobody-reports-stays-at-its-prior´) | probe, builder |
| (´plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move´) | probe, oracle, barrier, time, guard |
| (´plan:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge´) | builder, oracle, stream, rank, guard |
| (´plan:assayer:intent-discrimination-is-withheld-until-there-are-positives´) | rank, guard |
| (´plan:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone´) | barrier, rank, guard |
| (´plan:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost´) | none — test-local samplers over standing surfaces |
| (´plan:assayer:intent-injected-pseudo-counts-are-clamped-at-a-thousand´) | none — a test-local subscriber layer |
| (´plan:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively´) | probe, builder, time, persistence |
| (´plan:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind´) | probe, builder, oracle, stream, rank |
| (´plan:assayer:intent-no-mutable-handle-on-a-sentinel´) | none — a compile-failure case and a source audit |
| (´plan:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing´) | builder, oracle, stream, rank |
| (´plan:assayer:intent-one-investigated-label-moves-every-subsystem-at-once´) | probe |
| (´plan:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift´) | builder, oracle, rank, guard |
| (´plan:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched´) | probe |
| (´plan:assayer:intent-restore-time-decay-is-applied-exactly-once´) | probe, builder, oracle, stream, time, persistence, guard |
| (´plan:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates´) | probe, stream |
| (´plan:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination´) | probe, builder, rank, guard |
| (´plan:assayer:intent-sentinel-specific-uncertainty-does-not-wake-the-anchor´) | persistence, guard |
| (´plan:assayer:intent-signals-and-competitive-cells-stand-independently´) | probe, builder, barrier |
| (´plan:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor´) | oracle, time, specification conflict |
| (´plan:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix´) | probe, builder, stream, persistence |
| (´plan:assayer:intent-the-anchor-catches-a-regime-change-first´) | builder, stream |
| (´plan:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance´) | specification conflict |
| (´plan:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after´) | probe, stream |
| (´plan:assayer:intent-the-schur-complement-is-the-analytical-marginal´) | probe, builder, oracle, guard |
| (´plan:assayer:intent-the-two-action-crossover-sits-at-the-q-matched-half-power-boundary´) | builder, oracle, rank |
| (´plan:assayer:intent-three-decay-clocks-run-independently´) | probe, builder, oracle, barrier, time, guard |
| (´plan:assayer:intent-without-refactorisation-the-inverse-pair-drifts-apart´) | probe, builder |

Three further demands are unanimous enough that they are conditions on the whole harness rather than rows in a table. Every one of the forty-four states its stimulus as deterministic from a declared seed. Forty-three carry a *fails-before* arm naming the deliberate defect their witness must catch. Thirty-eight ask that a run which stops making progress fail with a message rather than park. Those three are already the harness's stated discipline, which is the most encouraging fact in this document: the demands do not ask for a different harness, they ask for more of the one that exists.

One last count is a warning rather than a demand. Thirty of the forty-four need a production or pending-form change before their witness can be written at all — retaining a per-model predictor, publishing a raw anchor score, correcting a rank-local calculation. The harness is not the critical path for those, and a harness concept that implied otherwise would be overselling itself.

## The present harness against those demands · `sec:assayer:testing-harness-concept-present`

The harness meets the unanimous conditions well. Determinism is a contract rather than a default — a build that never declared a seed refuses (´sec:assayer:testing-architecture-scenario´). Time is injected in both domains from one handle, and the injection is measured rather than assumed: the monotonic half removed a whole class of load-scaled divergence (´entry:assayer:harness-stage-clock-monotonic´). Stall reporting exists and is honest about what it is for — the deadlines answer liveness and never latency, which is the right reading and the one a shrunken deadline would destroy (´obs:assayer:testing-architecture-deadlines-are-liveness´). The tolerance and drift bundles already distinguish a figure the specification states from the harness's own reading of one, with a table saying which is which (´tab:assayer:harness-scenario-tolerances´). The assertion tier already pairs every domain reader with a budget-taking twin, which is the exact shape the oracle demand wants one layer up (´sec:assayer:testing-architecture-assertions´).

It meets four of the ten demands awkwardly, in the specific sense that a test can get what it needs but only by writing something the harness should have written.

Time travel is reached in two hops. The scenario hands back the clock handle and the test advances it, so nine plans independently propose the same fifteen-line pair of scenario verbs, and the persistent-domain stage stays open behind them (´entry:assayer:harness-stage-clock´). The gap is small and its cost is not: a verb on the scenario is a place to put the barrier that must follow a time advance, and a raw handle is not.

Streams are written as loops. The ergonomic verbs collapse an assess-label-flush round into one call, and `cycle_request` flushes the label path once per label — correct for a scenario of tens of labels and the wrong cost model at the horizons these plans run to, which four of them say in as many words. A plan wanting 1,276 labels, or 5,000, or two worlds driven in lockstep, has to write its own batching, its own barrier cadence and its own progress reporting, and no two of them will write the same one.

State is read through escape hatches. The scenario deliberately keeps two open — the engine itself and the owner-thread controls — and they are the right escape hatches to have (´sec:assayer:testing-architecture-scenario´). What they are not is a contract. Twenty-two plans need a narrow projection of internal state, and with no declared probe shape each proposes its own, which is why their estimates for materially identical work range from thirty-five to two hundred and twenty lines.

Construction is incomplete. The builder sets every non-default input explicitly and panics on a malformed policy, which is the right discipline, and it does not carry interaction templates or install a chosen competitive-cell set. The consequence is sharp: the reference configuration is kept at the feature-map boundary by a crate-level witness and cannot be reproduced deterministically from the public scenario path, so twenty plans reach around the builder to get the layout their promise is about.

Two demands the present harness cannot meet at all, and both are structural rather than a matter of missing verbs.

There is no independent-oracle tier. The assertion module reads a payload out of a reckoning and compares it to a caller-supplied expectation under a budget; nothing in the harness recomputes an asserted quantity by a second route. Seventeen plans need exactly that, twelve of them a tie-aware rank statistic, and the standing roadmap concedes the point — the harness advertises a discrimination assertion it does not implement (´entry:assayer:harness-stage-tapes´). An oracle is not an assertion with a different name: an assertion compares a reading to a number a human chose, and an oracle compares a reading to a number a second implementation computed, which is the only comparison that catches a systematic error in the first.

There is no persistence fixture. Eleven plans need to fork an instance — checkpoint it, hold it quiescent, copy its files, rebuild from the copy on the same injected clock, and compare the live instance against the replayed one — and the durability promises the testing plan lists as owed are the same eleven from the other direction (´entry:assayer:gap-recovery-residuals´).

Beneath all of this sits a fact the architecture document records and the plans do not mention, because from inside a single plan it is invisible. The package does not have one scenario model; it has four. The library harness is one. The crate test tree carries a second, with its own configuration builders, its own entity and report constructors, and its own version wait — and thirteen of the fifty-one crate test files use both. The maintenance harness is a third, driving a thread and two channels with its own channel depth and its own barrier verbs (´sec:assayer:testing-architecture-side-harnesses´). The multi-channel support tree is a fourth, at 4,726 lines nearly the size of the library harness and serving one binary, with its own paired-world ceremony and its own reason for restating the harness's challenge routing (´sec:assayer:testing-architecture-multi-channel´). Each model carries its own idea of what a wait is, and the wait is the thing that must not vary.

The crate tree's version wait is the clearest evidence available that this costs something. It polls a shared cell on the real clock, sleeps a millisecond between reads, and on expiry returns the version it happened to see rather than failing. Eleven sites call it. A test built on it cannot fail for the reason it should: an owner that stopped publishing yields a stale version, the caller asserts against it, and the assertion's failure names a number rather than the stall that produced it. That is the failure mode the liveness module exists to abolish, abolished in one tree and standing in another.

## The concept · `sec:assayer:testing-harness-concept-ideas`

Six ideas, each doing work the demands ask for, each with a cost worth stating.

### One scenario model, reached from every surface · `sec:assayer:testing-harness-concept-one-scenario`

There should be one type that owns an instance, one place that says what a barrier is, and one vocabulary of names, and every test — crate, integration, and the thirteen inline modules the architecture document counts as the third surface (´obs:assayer:testing-architecture-three-surfaces´) — should reach it.

The argument is not tidiness. It is that a barrier is a claim about the engine's queues, the queues are shared, and a second scenario model is a second and unreviewed set of claims about them. The one measured instance of this in the package's history is decisive: the residue that made the multi-channel suite flake was not a clock reading but a missing publication barrier on the harness's own lifecycle verbs, and finding it took a snapshot-version trace built for the purpose (´entry:assayer:harness-stage-clock-residual-race´). That investigation was run once, against one model, and its conclusion was installed in one place. A polling wait in a second model is the same bug in a tree that investigation never reached.

Convergence does not mean absorption. The maintenance harness drives a subject the scenario genuinely does not reach and should keep its own shape; what it should not keep is its own barrier vocabulary, which is exactly the thing six plans are asking the scenario for. The support tree specialises correctly — grids, matrices, reward sweeps, paired-world ceremony, none of which another binary runs — and should keep all of it; what should move is the single piece of substantive logic it restates, which the architecture document has already isolated (´obs:assayer:testing-architecture-routing-duplication´). The crate tree's parallel model is the one with no such defence: its configuration builders are the scenario's builder with fewer guarantees, and its version wait is a barrier with the failure removed.

The cost is a migration touching fifty-one crate test files, and it is the largest single cost in this concept. It is worth paying because it is paid once and because nothing else on this list is trustworthy until it is: a probe contract that says *cross the barrier first* means nothing while two trees disagree about what crossing is.

### Time is a scenario input, and every wait is a named barrier · `sec:assayer:testing-harness-concept-time-and-barriers`

The scenario should own time travel and should own a closed, named set of barriers — one per queue — and no test should ever construct a wait.

Time first. The clock seam is built and correct, and two domains stay separated by construction because one cannot be synthesised from the other (´sec:assayer:testing-architecture-clocks´). What is missing is that the scenario does not expose it, so nine plans propose the same verbs. Putting them on the scenario is fifteen lines and buys something the handle cannot: an advance that must be followed by a barrier can enforce it, and a raw handle cannot even know.

Barriers second, and this is the sharper half. The present set is two — one riding the command channel whose handler drains the label channel, and one for the cold-ramp observations that the first does not cover — and the architecture document is explicit that neither covers the other (´sec:assayer:testing-architecture-scenario´). There is a third queue, the identity-maintenance loop, and it has no barrier at all. Six plans discovered this independently and produced four names for the fix. The concept is that the barrier set is closed and declared: one verb per queue, each named for its queue, each documented with what it does and does not cover, and a scenario verb that needs two of them takes both rather than hoping the first implies the second.

The rule that follows is the one that pays. A test may not write a wait. Not a sleep, not a poll, not a deadline of its own. If a test needs to wait for something, the thing it is waiting for is a queue, and the queue has a barrier or the harness is short one. The twenty sleep sites and the eleven polling calls in the present trees are each a barrier that was never declared, and each is a place where a stalled engine produces a confusing assertion failure instead of a clear one.

The cost is that barriers are stricter than sleeps and will surface real ordering assumptions that sleeps were papering over. That is the benefit stated as a cost.

### A stream is data, played back by the harness · `sec:assayer:testing-harness-concept-tapes`

A long stimulus should be an ordered row list with a declared barrier policy, checkpoint callbacks and a per-row progress increment, played back by one harness runner. Thirteen plans ask for this directly and twenty-one cite the stage that holds it (´entry:assayer:harness-stage-tapes´).

What makes it worth being a harness type rather than a loop each test writes is that the four things around the rows are the hard parts and the rows are the easy part. The barrier cadence decides whether a checkpoint reading is meaningful or is reading the scheduler. The progress increment decides whether a stall fails with a row number or hangs. The checkpoint callback decides whether a failure can say which row the quantity moved at. The bounded-batch policy decides whether the run finishes. Every one of those is a property of playback, none of them is a property of the promise under test, and a test that writes its own gets four chances to get them wrong.

The rows, by contrast, should not be unified. The plans want label rows, report rows, registration rows, paired rows for two worlds and rows carrying an investigation provenance, and a type wide enough for all of them would be a type that says nothing. The runner should take rows by a trait and hold the playback contract; the row shapes belong to the plans.

Two constraints the concept must respect, and both are already recorded. A tape must not assume a batch is a unit of view, because the design declines to promise that (´dec:ordering:batch-timestamp´). And a per-row progress increment is a liveness signal and must never harden into a latency assertion, for the reason the deadlines are already generous.

The cost is real and belongs in the open: the enrichment grid is already a product large enough that a per-case cost change can move the suite's runtime an order of magnitude without anyone noticing, and the plan owes it a measured budget (´entry:assayer:process-grid-runtime´). Tapes add long-running cases. The budget should land before the tapes do, not after.

### State is read through one probe contract · `sec:assayer:testing-harness-concept-probes`

Twenty-two plans need a narrow projection of engine state, and they should get one declared contract rather than twenty-two hand-rolled types.

The contract the plans have already converged on, without coordinating, has five clauses. A probe crosses the barrier for the queue whose state it reads, before reading. It takes exactly one published load, so every field it returns describes one version rather than a walk across several. It returns owned copies and holds no borrow into engine state. It offers no mutation path. It reads no clock of its own, taking any timestamp it needs from the scenario's. Two more clauses belong to the harness rather than to any one plan: a probe is gated on test support and is invisible to a sealed build, which the module gate already achieves structurally rather than by review (´sec:assayer:testing-architecture-reach´); and a probe returns only what the retention record permits a reader to see, which is what keeps a probe from quietly becoming a widened production surface.

That last clause is the one that makes the contract more than convenience. Several plans want to read matrices the durable form deliberately excludes (´dec:retention:precision-excluded´), and each handles it correctly on its own — one takes a spectrum through the existing eigensolver without exposing the matrix, another clones covariances while carrying no precision at all. Correct twenty-two times by hand is not the same as correct by construction, and a declared contract is where the exclusion gets stated once instead of remembered twenty-two times.

The cost is that a contract constrains. A plan that wants a reading the contract forbids has to argue for a change to the contract rather than write a one-off, and that argument will sometimes be lost. That is the contract working.

### A reading is compared against a second computation · `sec:assayer:testing-harness-concept-oracles`

The harness should carry an oracle tier: a small set of independent recomputations of quantities the system publishes, each compared under a named budget from the existing bundle.

Seventeen plans need one and twelve of those need the same one — a tie-aware pairwise rank fold with class gates and a unit-interval check. Twelve local implementations of a rank statistic is twelve chances to get ties wrong in twelve different ways, against a published calculation whose own rank-locality one plan is proposing to correct. That is the case for the tier in miniature: an oracle is worth having precisely when the thing it checks might be wrong, and a per-test oracle written from the same misunderstanding as the code checks nothing.

The tier's discipline follows from the tolerance table's, and the analogy is exact. That table already distinguishes a bound the specification states, the harness's reading of a condition the specification states, and the harness's own choice, and says which each field is (´tab:assayer:harness-scenario-tolerances´). An oracle needs the same declaration: whether it computes the specification's formula, an independent route to the same quantity, or a reference implementation the harness chose. An oracle whose provenance is undeclared is a second opinion from an unnamed source.

Which oracles the tier holds is decided by the counts, not by ambition: the rank fold, a dense regularised Schur complement, a decay recurrence, a width formula from the dimension blocks. Four, not forty. The tier should stay small enough that every member is read by several tests, because an oracle with one caller is a test helper wearing a costume.

The cost is that an oracle can be wrong and is not itself witnessed by anything. It is answered by keeping the tier small, by requiring each member to be reached through a different route than the production path rather than by calling into it, and by the fails-before arm every plan already carries — a defect deliberately introduced must move the oracle and the reading apart.

### A fixture guards its own precondition · `sec:assayer:testing-harness-concept-guarded-fixtures`

A fixture that exists to establish a state should refuse to return until the state holds, and should say what it checked.

Thirteen plans state this in their own words — a fixture that refuses to return until its held-out gap and pairwise rank checks establish the trained precondition, a builder that returns a bound probe only after its three baselines are non-zero, a constructor that asserts its declared class counts before emitting a row. They are all describing the same move: non-vacuity is an obligation of the thing that sets up the state, not of the thing that asserts about it.

The reason this belongs in the harness rather than in each test is that a vacuous pass is the failure mode a test suite cannot detect about itself. An assertion that a trained model separates two classes passes trivially against a model that trained on nothing, and the passing test looks exactly like a working one. The harness already has one instance of the right shape — a health assertion that fails on any degradation counter while deliberately admitting the signals that report and never act (´sec:assayer:testing-architecture-assertions´) — and it should have the general one: a fixture returns a value that can only have come from a checked state.

This idea also settles a reframing the plan already owes. Separation by identity is currently promised as discrimination close to one, at a label budget where that figure is plausible but not guaranteed, so a failure would carry no information (´obs:assayer:reframe-identity-threshold´). A guarded fixture is what makes the honest version assertable: the fixture establishes and reports the cell's own adverse average near one half, and the test then asserts the separation identity carries above it. The threshold stops being a hope about sampling and becomes a comparison against a measured baseline.

The cost is that guarded fixtures fail during development, often, and for reasons that are about the fixture rather than the promise. That is the cost of finding out early rather than reading a green suite that proves nothing.

### The alternatives this concept rejects · `sec:assayer:testing-harness-concept-rejected`

Five alternatives are available and each is rejected on its merits.

**Leave it distributed and let each plan build what it needs.** This is the default and it is what the forty-four plans currently describe. It is rejected because the plans' own estimates already price it: the harness-shaped entries across the forty-four total well over a thousand lines of near-duplicate fixture work, against a concept whose new pieces are sized below. The distributed option is not cheaper, and it buys twenty-two probe shapes, twelve rank folds and four barrier names.

**Introduce mocks or doubles for the engine.** Rejected without hesitation. The scenario owns a real instance and there are no mocks anywhere in the harness, which is why a passing test is evidence about the system. Every demand in the table above is satisfiable with a real engine and a narrow reader; not one of them asks for a substituted component.

**Widen the production surface so state is readable without a probe.** Rejected. The opacity of the guidance surface is carried structurally (´dec:surface:guidance-opacity´), the durable form excludes what it excludes for stated reasons, and a reading added to the public surface for a test's benefit is a promise to hosts that no host asked for. The probe contract exists precisely so that the answer to *make this readable* is a gated projection rather than a wider contract.

**Absorb the multi-channel support tree into the library harness.** Rejected. Its subject is one seam and its specialisation — the twelve-point enrichment grid, the derivation matrices, the reward sweep — is correct and unshared. Absorbing it would put a thousand lines with one caller into a module every test links. The only piece that should move is the routing step it restates, and it restates that one for a stated structural reason rather than by oversight.

**Adopt a property-testing framework for the invariant packs.** Rejected here, though the packs themselves are worth wanting (´entry:assayer:harness-property-packs´). The package takes no new dependencies for this, the seeded generator and the declared-seed discipline already give reproducible randomised sweeps, and a framework's shrinking is of limited value against failures whose minimal case is a barrier ordering rather than a small input.

## What changes and what stays · `sec:assayer:testing-harness-concept-changes`

Sizes below are the concept's own estimates, informed by the plans' estimates for the same work where they overlap. Each entry names the demands it discharges from the table above.

**Entry (Kept unchanged)** · `entry:assayer:testing-harness-concept-kept`

Most of the harness is right and should not be touched. The clock pair keeps both domains from one handle and panics on backward travel, which is the contract made unmissable. The name vocabularies keep raw integers out of every scenario and keep the registry behind the boundary. The tolerance and drift bundles keep their values and their declarations; the deadlines keep their generous sizing for the reason already argued. The standard signal schema and the three fluent specification builders keep their shape — a builder whose `new` fills clean defaults and whose setters return `Self` is the right ergonomic for a host-facing payload with many fields. The report fixtures keep their figures and their warrant: authored stimulus with a discriminating ladder, declared with reasons, is a better arrangement than a regenerated golden and should not be disturbed by anything here (´dec:assayer:golden-report-stimulus´). The source scanner keeps its run-time root resolution, which is the module's substance. The empty folder matrices keep their role as the only structural marker of a scaffolding folder (´obs:assayer:testing-architecture-empty-matrices´).

**Entry (New: the closed barrier set)** · `entry:assayer:testing-harness-concept-barrier-set`

One verb per queue, named for its queue, each documenting what it covers and what it does not; the identity-maintenance barrier is the new one, sending the maintenance checkpoint, waiting under the existing deadline while the loop drains observations and emits cell changes, then crossing the model-owner barrier so every emitted lifecycle event is published before return. Approximately 60–90 lines including the documentation that makes the set closed. Discharges the barrier demand for (´plan:assayer:intent-entities-sharing-a-cell-are-separated-by-identity-alone´), (´plan:assayer:intent-signals-and-competitive-cells-stand-independently´), (´plan:assayer:intent-three-decay-clocks-run-independently´), (´plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move´), (´plan:assayer:intent-a-competitive-cell-exits-as-another-enters´) and (´plan:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from´). First in the order, because everything else rests on it.

**Entry (New: scenario time verbs)** · `entry:assayer:testing-harness-concept-time-verbs`

Forward-only advance and travel on the scenario, delegating to the clock and enforcing the barrier an advance requires. Approximately 20–30 lines. Discharges the remaining scenario-vocabulary portion of (´entry:assayer:harness-stage-clock´) for the nine plans that ask, among them (´plan:assayer:intent-a-stale-adverse-history-dissolves-without-fresh-labels´) and (´plan:assayer:intent-a-trickle-of-untargeted-challenges-sharpens-belief-and-stops-short-of-a-usable-estimate´). The larger half of that stage — the remaining production call sites reading the persistent domain — is production work and is not this concept's.

**Entry (New: the builder reaches the whole construction contract)** · `entry:assayer:testing-harness-concept-complete-builder`

Interaction templates forwarded to the engine builder, an identity registration that installs a chosen competitive-cell set through a gated lifecycle verb, and a check that the resulting runtime layout matches the reference widths a crate witness already keeps. Approximately 70–110 lines. Discharges the builder demand for twenty plans and is what makes the reference configuration reproducible from the public scenario path — the precondition for (´plan:assayer:intent-the-anchor-catches-a-regime-change-first´) and (´plan:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix´).

**Entry (New: the probe contract and its first projections)** · `entry:assayer:testing-harness-concept-probe-contract`

The five-clause contract written down once, with the gating and retention clauses stated, plus the three projections the counts justify first: a published model block, a published slot's standardisation moments, and a pending entry's request-scoped view. Approximately 200–280 lines for the contract and the three, against roughly 900 lines if the twenty-two plans each build their own. Discharges the probe demand for (´plan:assayer:intent-restore-time-decay-is-applied-exactly-once´), (´plan:assayer:intent-marginalising-a-sentinel-leaves-its-information-behind´), (´plan:assayer:intent-the-schur-complement-is-the-analytical-marginal´), (´plan:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale´), (´plan:assayer:intent-signals-and-competitive-cells-stand-independently´) and (´plan:assayer:intent-an-axis-nobody-reports-stays-at-its-prior´) directly, and sets the shape the remaining sixteen follow.

**Entry (New: the tape runner)** · `entry:assayer:testing-harness-concept-tape`

One playback runner holding the barrier policy, the bounded-batch cadence, the checkpoint callback and the progress increment, taking rows through a trait; row shapes stay with the plans that need them. Approximately 150–200 lines plus its own focused tests. Discharges the stream demand for thirteen plans and the streaming half of (´entry:assayer:harness-stage-tapes´), including (´plan:assayer:intent-a-tenfold-base-rate-shift-trips-the-cumulative-sum-and-resets-calibration´), (´plan:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing´), (´plan:assayer:intent-restriction-opens-a-gap-between-operational-and-sister-estimates´) and (´plan:assayer:intent-cross-sentinel-correlation-is-caught-only-once-interactions-converge´). Also the piece that makes (´entry:assayer:gap-convergence-depth´) writable, which the priority register puts highest (´reg:assayer:testing-plan-priorities´).

**Entry (New: the oracle tier)** · `entry:assayer:testing-harness-concept-oracles`

Four members with declared provenance: a tie-aware pairwise rank fold with class gates, a dense regularised Schur complement, a decay recurrence, and a width formula over the dimension blocks. Approximately 180–240 lines. Discharges the oracle demand for seventeen plans and the rank demand for twelve, and retires the discrimination assertion the roadmap advertises without implementing.

**Entry (New: guarded fixtures as the default shape)** · `entry:assayer:testing-harness-concept-guarded-fixtures`

A trained-state fixture that returns only after its own held-out and class-count checks pass, and a stated convention that every fixture establishing a state checks it. Approximately 90–130 lines for the first fixture and the convention; subsequent fixtures carry the cost in their own plans. Discharges the guard demand for thirteen plans and makes the identity-separation reframing assertable.

**Entry (New: the persistence fork)** · `entry:assayer:testing-harness-concept-persistence-fork`

Checkpoint, quiesce, copy, rebuild from an isolated image on the same injected clock, and a normalised state projection with a field-aware comparator. Approximately 250–350 lines, the largest new piece, and the one whose size the plans themselves already estimate at that scale. Discharges the persistence demand for eleven plans and is the infrastructure behind (´entry:assayer:gap-recovery-residuals´).

**Entry (Merged: the four scenario models become one)** · `entry:assayer:testing-harness-concept-merge-scenario-models`

The crate tree's parallel configuration builders and constructors fold into the scenario's builder; the maintenance harness keeps its subject and adopts the barrier set; the support tree keeps its specialisation and delegates the routing step it restates. Approximately 300–450 lines of change across fifty-one crate test files and the support tree, almost all of it mechanical. No plan asks for this and every plan depends on it, which is the usual signature of infrastructure debt.

**Entry (Retired: polling waits and sleeps)** · `entry:assayer:testing-harness-concept-retire-polling`

The crate tree's version wait and the twenty sleep sites are replaced by barriers from the closed set. The version wait's own failure mode is the argument: it returns a stale reading on expiry rather than failing, so a stalled owner presents as a wrong number. Approximately 80–120 lines of change. Nothing new is needed to do this beyond the barrier set.

**Entry (Retired: the callerless items and the stale staging notes)** · `entry:assayer:testing-harness-concept-retire-callerless`

Seven harness items presently have no caller outside their defining module and none is reported, because the feature that makes the harness reachable also makes every flattened item public (´obs:assayer:testing-architecture-unreported-decay´). Each should be given a caller by the work above or deleted. The module documentation's staging notes name several items that do not exist and mark as pending two that have landed, so the documentation is currently neither a roster of what exists nor a list of what does not (´obs:assayer:testing-architecture-staging-notes´); it should be rewritten against the concept. The synchronisation-drift target's private copy of the liveness module should become imports, the condition its own header names for that having been met (´obs:assayer:testing-architecture-liveness-copy´). Approximately 60–100 lines, mostly deletion.

## Migration · `sec:assayer:testing-harness-concept-migration`

Nothing here requires a big-bang rewrite, and the order is decided by what unblocks the most work per line rather than by what is most interesting.

The barrier set is first and is a precondition for honesty in everything after it. It is small, it unblocks six plans on its own, and until it exists the instruction *cross the barrier first* has no referent for the identity queue. Retiring the polling waits follows immediately and in the same pass, because a barrier set that coexists with a silently-degrading poll has not replaced anything. These two together are the smallest change with the largest effect on what a failure means.

The scenario time verbs and the complete builder come next, in either order, both small and both blocking more plans than their size suggests — nine and twenty respectively. The builder in particular is the difference between a reference-configuration promise being testable and being approximated.

The probe contract is fourth, and the sequencing matters: it should be written with its first three projections rather than as a contract alone, because a contract nobody has instantiated is a guess. Twenty-two plans then follow its shape instead of inventing theirs, and the marginal cost of the nineteenth probe is small.

The oracle tier is fifth. It can be built before the tape and should be, because several of the plans it serves need no long stream at all, and because the rank fold's absence is currently blocking twelve plans for the sake of forty lines.

The tape runner is sixth and should wait until the enrichment grid has the measured runtime budget it is owed. Building long-running playback into a suite whose per-case cost is unmeasured is how a suite becomes slow without anyone being able to say when.

The persistence fork is seventh: the largest, the most self-contained, and the one with the fewest dependents blocked behind it.

The merge of the scenario models runs alongside all of it rather than as a phase. Each crate test file that is touched for another reason moves to the single model at that point, and the barrier and polling retirements above will touch most of them anyway. Treating it as a phase of its own would produce one enormous change that nobody can review; treating it as a rule applied to every file already being edited finishes it at no visible cost.

Two things should not wait for any of this. The six plans blocked on a specification conflict are blocked on a decision, not on a fixture, and no harness work moves them. The thirty plans needing a production change need it whether or not this concept lands.

## Risks and open questions · `sec:assayer:testing-harness-concept-risks`

**Observation (The harness is not the critical path for most of the forty-four)** · `obs:assayer:testing-harness-concept-not-the-critical-path`

Thirty of forty-four need a production or pending-form change before their witness can be written, and six more need a specification question answered. A harness concept measured by how many promises it turns into tests will read as a failure; measured by how much duplicate fixture work it prevents and how many failures it makes legible, it is doing its job. The honest claim is that the concept removes the harness from the critical path, not that it was the only thing on it.

**Observation (A probe contract can collide with the retention record)** · `obs:assayer:testing-harness-concept-probe-versus-retention`

Several plans want readings the durable form deliberately excludes, and each currently arranges its own compliance. Writing the exclusion into the probe contract is the right move and it creates a real tension: the contract must forbid what the record forbids, and a plan whose promise is genuinely about an excluded quantity then has no probe available to it. The resolution is not to soften the clause but to say where such a promise's witness belongs — at the crate level, where the state is reachable without a contract at all — and the concept does not yet say that sharply enough.

**Observation (A tape must not assume a batch is a unit of view)** · `obs:assayer:testing-harness-concept-tape-versus-batch`

The design declines to promise that requests in one batch share a view (´dec:ordering:batch-timestamp´), and the suite has already once been built on the opposite assumption and corrected (´entry:assayer:mid-batch-identity-movement´). A playback runner batching rows for cost reasons is an easy place to reintroduce the assumption silently, because the batching is a harness decision and the comparison is a test's. The runner's contract must state that a batch is a cost boundary and never a consistency boundary, and the tests that compare readings across rows must take a barrier between them.

**Observation (A progress increment could harden into a latency assertion)** · `obs:assayer:testing-harness-concept-progress-not-latency`

The deadlines are liveness numbers and are generous for a stated reason, including that an instrumented coverage build runs about a hundred times slower. A per-row progress increment sits close enough to a per-row timing that the temptation to assert on it will arrive. It should be declined in the same terms: a test that wants to say an answer arrives quickly asserts on elapsed time itself and says so.

**Observation (Seventeen plans land in one module)** · `obs:assayer:testing-harness-concept-destination-concentration`

Seventeen of the forty-four place their witness beside the same standing convergence witness. The concentration is not itself wrong — that module is where learning behaviour belongs — but seventeen long-running tests in one binary is a runtime and a review bottleneck, and the retired one-file-per-division intent was already found broken for six of seventeen files by the study that retired it (´obs:assayer:harness-integration-slices´). Whether the convergence subject wants splitting before those seventeen land is an open question this concept does not answer.

**Observation (An oracle tier is unwitnessed by construction)** · `obs:assayer:testing-harness-concept-oracle-trust`

Nothing checks the oracles. Keeping the tier to four members read by many tests, requiring each to reach its answer by a route the production path does not take, and leaning on the fails-before arm every plan already carries are the three mitigations, and none of them is proof. An oracle that shares a misunderstanding with the code it checks is silent, and the only real defence is that the tier is small enough to be read carefully by a person.

**Observation (The merge is the one cost no plan is paying for)** · `obs:assayer:testing-harness-concept-merge-has-no-sponsor`

Every other entry here is work some plan already needs and has already sized. The convergence of the four scenario models is work no plan asks for, which means it will be the first thing deferred and the thing whose deferral quietly invalidates the rest. Folding it into every file already being edited, rather than proposing it as a phase, is the concept's answer, and it is a procedural answer to a structural risk.

**Observation (Two demands may be one)** · `obs:assayer:testing-harness-concept-guard-and-oracle`

A fixture that guards its own precondition and an oracle that recomputes a reading are doing related work: both establish that a number means what the test thinks it means. Whether they should be one mechanism — a fixture that returns its own oracle-computed baseline alongside the state it established — is a real design question. The concept keeps them separate because their failure modes differ: a guard failing means the setup did not work, and an oracle disagreeing means the system is wrong, and a mechanism that conflated them would report the second as the first.

## Acceptance · `sec:assayer:testing-harness-concept-acceptance`

The harness has arrived when a reader can check the following without running anything but the linter and a search.

One scenario type is reachable from all three caller surfaces, and the crate test tree's parallel configuration builders, entity and report constructors are gone. No test tree outside the harness's own liveness module contains a sleep, a poll, or a wait constructed by a test; every wait is a call to a member of a barrier set whose documentation says what each member covers and what it does not. Time travel is reached from the scenario, and the clock handle is no longer the vocabulary a test uses.

Every projection of engine state in the test trees is an instance of one declared probe contract, and the contract's clauses about gating and retention are written down where a new probe's author will read them. Every long stimulus is rows plus a playback call rather than a hand-written loop, and no test writes its own batch cadence. Every comparison that could be checked by a second computation is checked by a member of the oracle tier, each member declaring whether it computes a specification formula, an independent route, or a reference the harness chose.

Three measurements complete it, and they are measurements rather than inspections. The suite's runtime has a recorded budget with a tolerance, so a grid that grows is a finding rather than a slow drift. No harness item lacks a caller, which the module's own documentation can then state truthfully again. And the next plan written against the harness names, in its `Entry` list, no piece of harness that does not already exist — which is the only test of whether the ten demands were the right ten.
