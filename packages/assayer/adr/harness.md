# Harness · `rec:harness:shared-testing-contract`

This record fixes the shared testing contract for Assayer: one vocabulary for constructing a subject, controlling it, observing it and deciding when its asynchronous work is complete. The contract binds the crate test tree, the integration-test binaries and the inline unit-test modules — all three caller surfaces (`obs:assayer:testing-architecture-three-surfaces`) reached by the testing architecture (`rep:assayer:testing-architecture`).

The tests exercise the assessment interface, label pipeline and concurrency model without redefining them (`chap:spec:assessment-interface`) (`chap:spec:label-pipeline`) (`chap:spec:concurrency`). The structural and public-contract layers remain the sources of product behaviour (`spec:structural:architectural-commitments`) (`spec:contracts:public-contracts`), and the concurrency and surface records remain its decision owners (`rec:concurrency:snapshot-stewardship`) (`rec:surface:host-facing-contracts`).

This record decides about tests and changes no specification behaviour. It is the testing-only exception admitted on the ground that dividing a common contract among behaviour records would copy or mis-home it (`dec:assayer:record-harness-record`). Its choices form the standing discipline (`sec:assayer:testing-harness-concept-ideas`); implementation order remains planning (`sec:assayer:testing-plan-roadmap`), and authored test claims remain governed by the test documentation policy (`dec:assayer:test-documentation-policy`).

**Decision (One scenario owns the common vocabulary)** · `dec:harness:single-scenario`

One Assayer-owning scenario type carries the common instance, names, time and barriers. The single steward remains the only owner of the working copy (`dec:concurrency:single-steward`) under the layer's one-writer choice (`dec:structural:write-serialisation`) and the specification's concurrency contract (`chap:spec:concurrency`); the scenario coordinates access to that subject and does not become another owner.

The rejected alternative is a parallel scenario model for each caller surface. The existing scenario and multi-channel arrangements show why that loses (`sec:assayer:testing-architecture-scenario`) (`sec:assayer:testing-architecture-multi-channel`): a second model carries a second vocabulary for shared queues, while the common choice exists precisely to make those claims singular (`sec:assayer:testing-harness-concept-one-scenario`).

The landed skeleton supplies the common starting point (`entry:assayer:harness-stage-skeleton`), and the publication-barrier residue shows the cost of leaving one model outside it (`entry:assayer:harness-stage-clock-residual-race`). Convergence is therefore a contract, while the migration needed to make it true everywhere remains bounded below (`obs:assayer:testing-harness-concept-merge-has-no-sponsor`).

**Decision (Tests use declared barriers and author no waits)** · `dec:harness:no-ad-hoc-waits`

A test waits only by calling the declared barrier for the queue whose completion it needs. The label queue and deferred identity drain retain their specified meanings (`req:publication:label-queue`) (`alg:publication:identity-draining`); the harness names how a test observes their completion under the same single-writer discipline (`dec:structural:write-serialisation`) (`dec:concurrency:single-steward`).

The rejected alternative is a test-authored sleep, poll, deadline or compound wait. It loses because elapsed time is not evidence that a queue drained, and a poll can turn a stopped worker into a stale-value assertion. The liveness architecture gives the failure a named home (`sec:assayer:testing-architecture-liveness`), while lifecycle operations that require two-phase visibility retain that existing order (`dec:construction:two-phase-visibility`).

The barrier rule is the closed-set choice argued for the scenario (`sec:assayer:testing-harness-concept-time-and-barriers`). The earlier residual-race evidence is decisive about the diagnostic difference: a missing publication barrier is reported as missing completion only when the harness owns the wait (`entry:assayer:harness-stage-clock-residual-race`).

**Decision (Long stimuli are declarative playback)** · `dec:harness:declarative-playback`

A long stimulus is a sequence of typed rows interpreted by one runner. The row type belongs to the test subject; the runner owns barrier policy, checkpoint callbacks, bounded batching and progress reporting. Assessment, label and report-reception meanings remain the specification's (`req:runtime:assessment-interface`) (`req:runtime:label-interface`) (`alg:runtime:report-reception`), and playback merely drives the assessment and learning paths (`dec:operational:assessment-path`) (`dec:operational:learning-path`).

The rejected alternative is a handwritten stimulus loop per test. It loses because each loop would separately decide when state is settled, where a failing row is named and whether a checkpoint reads completed work. A playback batch inherits the existing cost boundary without gaining a shared view: timestamps remain batch-wide and published snapshots remain per-request (`dec:ordering:batch-timestamp`) (`dec:concurrency:per-request-load`).

The common scenario is the execution home (`sec:assayer:testing-architecture-scenario`), and the playback contract is the shared choice (`sec:assayer:testing-harness-concept-tapes`). The staged tape work and its runtime prerequisite remain plan concerns (`entry:assayer:harness-stage-tapes`) (`entry:assayer:process-grid-runtime`), not reasons to weaken the standing contract.

**Decision (State readers obey one probe contract)** · `dec:harness:probe-contract`

A permitted test-support probe crosses the relevant barrier, performs one published load, returns owned copies, exposes no mutation and reads time only from the scenario. Those clauses preserve the definition of a published snapshot and the structural-exactness guarantee (`def:publication:model-snapshot`) (`inv:guarantee:structural-exactness`) under the representation choices for published and in-flight state (`dec:representation:published-composition`) (`dec:representation:pending-retention`).

The rejected alternatives are local state readers with unstated rules and a wider production surface. The first loses because repeated readers can mix versions or retain borrows; the second loses because guidance is intentionally opaque and precision is intentionally excluded (`dec:surface:guidance-opacity`) (`dec:retention:precision-excluded`). A test need does not create a host promise.

The testing gate already separates the reachable support surface (`sec:assayer:testing-architecture-reach`). The contract formalises that boundary (`sec:assayer:testing-harness-concept-probes`), including the hard consequence that a witness of excluded state needs a direct crate-level home rather than a nonconforming probe (`obs:assayer:testing-harness-concept-probe-versus-retention`). The staged work may add projections only inside that limit (`entry:assayer:harness-stage-tapes`).

**Decision (Shared oracles declare an independent route)** · `dec:harness:oracle-tier`

A small shared oracle tier recomputes published quantities by declared routes and compares them under named budgets. Each oracle declares whether it implements a specification formula, an independent derivation or a reference chosen by the harness; the tolerance register supplies the corresponding provenance discipline (`tab:assayer:harness-scenario-tolerances`).

The rejected alternative is repeated local recomputation, including an expected value obtained through the production route it purports to check. It loses because duplication multiplies interpretations while route reuse checks only self-consistency. The per-observation and precision invariants state what the comparisons must witness (`inv:guarantee:per-observation-exactness`) (`inv:guarantee:precision`), the numerics layer owns their mathematics (`spec:numerics:model-mathematics`), and the retained debug oracle is the narrow precedent (`dec:posterior:debug-oracle`).

The existing assertion tier remains a reader and comparator rather than being renamed an oracle (`sec:assayer:testing-architecture-assertions`). The shared tier is the distinct second-computation choice (`sec:assayer:testing-harness-concept-oracles`), bounded by the fact that independence mitigates but cannot eliminate oracle error (`obs:assayer:testing-harness-concept-oracle-trust`).

**Decision (Fixtures establish and report their preconditions)** · `dec:harness:guarded-fixtures`

A state-establishing fixture returns only after checking its named precondition, and a failure reports the measured state that failed the check. This makes non-vacuity the setup's obligation before a result assertion applies. The evidence-authority invariant remains the source of what assessment may establish (`inv:guarantee:evidence-authority`), while convergence remains diagnostic and never a runtime gate (`dec:operational:convergence-diagnostic`) (`dec:health:reports-never-gates`).

The rejected alternative is unchecked setup followed by an assertion that assumes training, separation or activation occurred. It loses because a vacuous pass is indistinguishable from a working fixture. The current assertion architecture provides the reporting shape (`sec:assayer:testing-architecture-assertions`), and the guarded-fixture choice makes it general (`sec:assayer:testing-harness-concept-guarded-fixtures`).

Measured baselines are preferred to hoped-for thresholds, as the identity-separation reframing demonstrates (`obs:assayer:reframe-identity-threshold`). A guard may use independent arithmetic, but it does not thereby become the result oracle (`obs:assayer:testing-harness-concept-guard-and-oracle`).

**Decision (Fixture guards and result oracles stay separate)** · `dec:harness:separate-validation`

Fixture guards validate that setup produced the state a test requires; result oracles validate what the system did from that state. The distinction preserves evidence authority (`inv:guarantee:evidence-authority`) and the rule that convergence reports rather than controls behaviour (`dec:operational:convergence-diagnostic`) (`dec:health:reports-never-gates`).

The rejected alternative is one mechanism that reports every disagreement as setup failure. It loses because the remedies differ: a failed guard invalidates the fixture, while an oracle disagreement challenges the result. The assertion architecture can report both without conflating them (`sec:assayer:testing-architecture-assertions`), and the proposed fusion is rejected explicitly on that diagnostic ground (`obs:assayer:testing-harness-concept-guard-and-oracle`).

Guarded fixtures remain the default setup shape (`sec:assayer:testing-harness-concept-guarded-fixtures`), including comparisons against measured baselines rather than a threshold the setup may never have reached (`obs:assayer:reframe-identity-threshold`).

**Decision (Shared scenarios exercise the real engine)** · `dec:harness:real-engine`

Shared scenarios exercise a real Assayer instance through its assessment contract (`chap:spec:assessment-interface`) (`dec:contracts:assessment-surface`) and the infallible batch surface that real hosts receive (`dec:surface:batch-infallible`). The numerical paths remain subject to the precision guarantee and repair discipline (`inv:guarantee:precision`) (`dec:numerics:repair-cascade`), with an independent debug oracle where one already exists (`dec:posterior:debug-oracle`).

The rejected alternative is an engine mock or double. It loses because queue ordering, publication, decay and numerical repair are the behaviours the shared scenario exists to witness; substituting their implementation would make a passing test evidence about the substitute. The no-double choice is part of the rejected-alternatives argument (`sec:assayer:testing-harness-concept-rejected`).

Specialised maintenance and multi-channel arrangements may drive the real subject through narrower seams (`sec:assayer:testing-architecture-side-harnesses`) (`sec:assayer:testing-architecture-multi-channel`). Seeded invariant packs may broaden inputs (`entry:assayer:harness-property-packs`), but neither changes what executes.

**Decision (Specialised side harnesses keep only genuine specialisation)** · `dec:harness:specialised-side-harnesses`

Maintenance and multi-channel harnesses remain local where their subjects, row shapes or paired-world ceremony are genuinely specialised (`sec:assayer:testing-architecture-side-harnesses`) (`sec:assayer:testing-architecture-multi-channel`). They reuse shared engine logic and barrier meanings, so specialisation does not become a second contract.

The rejected alternatives are absorbing every side harness into the common scenario and leaving each one wholly independent. Absorption loses because it would make single-caller machinery universal; independence loses because it duplicates the ordering claims shared with the real engine. The explicit boundary is the selected alternative (`sec:assayer:testing-harness-concept-rejected`).

Both sides still exercise the real assessment surface and its precision discipline (`chap:spec:assessment-interface`) (`dec:contracts:assessment-surface`) (`dec:surface:batch-infallible`) (`inv:guarantee:precision`) (`dec:numerics:repair-cascade`). The debug-oracle precedent and seeded-pack work remain available without moving specialised ceremony into the shared tier (`dec:posterior:debug-oracle`) (`entry:assayer:harness-property-packs`).

**Decision (Invariant sweeps are seeded and framework-free)** · `dec:harness:seeded-sweeps`

Invariant packs use declared reproducible seeds and print the seed on failure. They drive the real assessment surface (`chap:spec:assessment-interface`) (`dec:contracts:assessment-surface`) (`dec:surface:batch-infallible`) and test precision and repair through their existing contracts (`inv:guarantee:precision`) (`dec:numerics:repair-cascade`) (`dec:posterior:debug-oracle`).

The rejected alternative is adding a property-testing framework. It loses because the harness already has deterministic generation, while shrinking offers little for failures whose cause is queue order, publication or a numerical path rather than a minimal input. The framework rejection is explicit (`sec:assayer:testing-harness-concept-rejected`), and the planned packs remain the implementation home (`entry:assayer:harness-property-packs`).

The maintenance and multi-channel subjects retain their specialised drivers (`sec:assayer:testing-architecture-side-harnesses`) (`sec:assayer:testing-architecture-multi-channel`); seeded sweeps change input coverage and do not absorb those harnesses.

**Caveat (The contract removes harness duplication, not product blockers)** · `cav:harness:limited-unblocking`

The contract can remove duplicate fixtures, readers and waits, but it cannot settle missing production behaviour or a specification choice. Most promised witnesses have prerequisites elsewhere (`obs:assayer:testing-harness-concept-not-the-critical-path`), and the concentration of future witnesses does not turn those prerequisites into harness work (`obs:assayer:testing-harness-concept-destination-concentration`) (`obs:assayer:harness-integration-slices`).

The existing architecture also contains callerless helpers and a specialised support tree (`obs:assayer:testing-architecture-unreported-decay`) (`sec:assayer:testing-architecture-multi-channel`). Their retirement is enabled by a common contract, not accomplished by this decision record. The concurrency specification, one-writer layer choice and steward decision remain unchanged (`chap:spec:concurrency`) (`dec:structural:write-serialisation`) (`dec:concurrency:single-steward`).

**Caveat (Playback progress is liveness, not latency)** · `cav:harness:progress-not-latency`

A playback progress counter identifies the row before which execution stopped; it establishes no elapsed-time promise. The existing deadlines already make that distinction (`obs:assayer:testing-architecture-deadlines-are-liveness`), and the same limit binds playback (`obs:assayer:testing-harness-concept-progress-not-latency`). A latency claim requires its own measurement and budget.

This limit applies while playback drives assessment, labels and report reception (`req:runtime:assessment-interface`) (`req:runtime:label-interface`) (`alg:runtime:report-reception`) through the assessment and learning paths (`dec:operational:assessment-path`) (`dec:operational:learning-path`). Batch timestamp and per-request publication semantics remain independent of progress (`dec:ordering:batch-timestamp`) (`dec:concurrency:per-request-load`). The scenario and tape plan supply execution and work sequencing (`sec:assayer:testing-architecture-scenario`) (`sec:assayer:testing-harness-concept-tapes`) (`entry:assayer:harness-stage-tapes`) (`entry:assayer:process-grid-runtime`).

**Caveat (Oracle independence mitigates but does not prove trust)** · `cav:harness:oracle-trust`

An independent route and a fails-before arm reduce the chance that an oracle repeats the production defect; neither proves the oracle correct (`obs:assayer:testing-harness-concept-oracle-trust`). The tier remains deliberately small and read by several tests (`sec:assayer:testing-harness-concept-oracles`), with provenance and budgets declared beside the existing tolerance discipline (`tab:assayer:harness-scenario-tolerances`).

The oracle compares against the per-observation and precision guarantees (`inv:guarantee:per-observation-exactness`) (`inv:guarantee:precision`) using mathematics owned by the numerics layer (`spec:numerics:model-mathematics`). The retained debug oracle is precedent, not proof (`dec:posterior:debug-oracle`), and the assertion architecture remains the comparison surface (`sec:assayer:testing-architecture-assertions`).

**Caveat (The convergence witness layout remains deferred)** · `cav:harness:convergence-concentration`

The deferred question is whether the convergence subject remains one integration-test binary or splits before the concentrated long-running intent witnesses land. Retaining one subject preserves claim cohesion and shared fixtures; splitting it isolates runtime and maintenance cost. The decision must choose between those alternatives using runtime, maintenance and claim-cohesion criteria before the concentrated witnesses arrive. Its deferral identity is (`entry:assayer:defer-harness-convergence-layout`) under the deferred-home ruling (`dec:assayer:deferred-home`).

Nothing in the concurrency specification, single-writer layer choice or steward decision answers file and binary layout (`chap:spec:concurrency`) (`dec:structural:write-serialisation`) (`dec:concurrency:single-steward`). The current concentration and integration-slice history establish the question (`obs:assayer:testing-harness-concept-destination-concentration`) (`obs:assayer:harness-integration-slices`); the specialised support tree and callerless-helper finding show the maintenance risks on either side (`sec:assayer:testing-architecture-multi-channel`) (`obs:assayer:testing-architecture-unreported-decay`). The harness is not the critical path for the product work itself (`obs:assayer:testing-harness-concept-not-the-critical-path`).

**Caveat (Adoption is incremental and the end state is not yet descriptive)** · `cav:harness:incremental-adoption`

The single-scenario decision is the contract, not a claim that every caller already conforms. The scenario and multi-channel architecture still expose parallel vocabularies (`sec:assayer:testing-architecture-scenario`) (`sec:assayer:testing-architecture-multi-channel`), and the merge has no independent sponsor (`obs:assayer:testing-harness-concept-merge-has-no-sponsor`). Adoption therefore proceeds through touched tests until the remaining model and wait vocabulary retire.

The end state remains one scenario around the specification's concurrency model (`chap:spec:concurrency`) and the one-writer decisions (`dec:structural:write-serialisation`) (`dec:concurrency:single-steward`). The common scenario choice names that destination (`sec:assayer:testing-harness-concept-one-scenario`); the landed skeleton and residual-race repair are its present foundation (`entry:assayer:harness-stage-skeleton`) (`entry:assayer:harness-stage-clock-residual-race`).
