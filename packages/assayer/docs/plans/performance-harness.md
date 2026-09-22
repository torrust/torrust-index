# The performance subject is rebuilt on the shared harness · `plan:assayer:performance-harness`

The maintained performance route is one Criterion suite over the real Assayer scenario. It turns the seven ignored stopwatch witnesses into repeatable measurements, preserves their generous budgets as documented comparison floors, and makes the specification's cost model rather than a machine-dependent assertion define what belongs in the subject (`chap:spec:convergence-and-resources`) (`dec:harness:performance-benchmarks`).

## The subject · `sec:assayer:performance-harness-subject`

Performance means the cost of the paths and retained populations the package promises to operate. Assessment reads posterior state at quadratic cost, label publication updates it at quadratic cost per eligible model, and the pending population dominates memory at deployment scale (`tab:resource:assessment-cost`) (`tab:resource:label-cost`) (`tab:resource:memory`). Construction and shutdown bound entry to and exit from that operating state; pre-seeding is the synchronous bulk route into the ordinary label path (`dec:construction:post-seeding`); the two health surfaces deliberately occupy different cost tiers (`dec:health:tiered-queries`); and reference-dimension marginalisation pays the lifecycle cost of removing a source (`alg:registry:sentinel-deregistration`).

Six Criterion groups cover that subject. The group boundary follows the operation whose scaling term or lifecycle meaning the specification names, while benchmark identifiers inside a group distinguish cases or measured regions.

| Criterion group | Measured operation and cases | Cost-model anchor | Stopwatch-witness disposition | Documented comparison floor |
| --- | --- | --- | --- | --- |
| `assessment` | `World::core_assess` and `World::derive_for_request` over cold and reference-width worlds, with calls as throughput | (`tab:resource:assessment-cost`) and (`alg:runtime:assessment-pipeline`) | The combined boundary witness (`test:bench:bench-assessment`) retires because one elapsed figure conceals the assessment/label split; its round-trip floor remains a cross-group comparison | Ten thousand assess–derive–label round trips within sixty seconds, or six milliseconds per completed round trip |
| `label_publication` | Complete `World::label` publication through its barrier and the dense model-update region inside that same real-engine publication, with labels as throughput | (`tab:resource:label-cost`) and (`alg:runtime:update-path`) | The attribution witness (`test:bench:bench-label-publication`) becomes two measured regions in this group; the label half of the combined boundary witness moves here | No asserted floor for the attribution split; the historical six-millisecond round-trip floor remains documented across this group and `assessment` |
| `construction_shutdown` | Build and drop one `World`, including owner-thread join, across declared configurations | construction and the reference configuration (`tab:resource:reference-configuration`) | The construction witness (`test:bench:bench-construction-shutdown`) becomes this group | One hundred complete build/drop cycles within thirty seconds, or three hundred milliseconds per cycle |
| `pre_seed` | One synchronous bulk `Assayer::pre_seed` call over declared populations, with entries as throughput | (`dec:construction:post-seeding`) and (`tab:resource:label-cost`) | The pre-seed witness (`test:bench:bench-pre-seed`) becomes this group | One thousand accepted entries within thirty seconds |
| `health` | `Assayer::health_summary` and `Assayer::full_health_report` as separate identifiers over cold and populated worlds, with calls as throughput | (`dec:health:tiered-queries`) | The summary and full-report witnesses (`test:bench:bench-health-summary`) (`test:bench:bench-full-health-report`) become the two identifiers | Ten thousand summaries within ten seconds, or one millisecond per call; one hundred full reports within five seconds, or fifty milliseconds per call |
| `reference_marginalisation` | `World::deregister_sentinel` after guarded reference-width training, with completed removals as throughput | (`tab:resource:reference-configuration`) and (`alg:registry:sentinel-deregistration`) | The direct-model Schur witness (`test:bench:bench-reference-marginalisation`) becomes the real lifecycle benchmark; its private model constructor retires | Ten reference-dimension marginalisations within three seconds, or three hundred milliseconds per completed removal |

The per-assessment group keeps `World::core_assess` separate from assessment plus host derivation because derivation is outside the Core cost table and is linear in the action count. The pair shows both the specification's subject and the path a host actually calls without folding label publication into either (`tab:resource:assessment-cost`).

The label-publication group preserves the profile witness's question rather than only its loop. One identifier measures a completed publication through the queue barrier; the second uses a test-support region recorder on the same owner-thread execution to return only the elapsed duration accumulated inside the dense model-update block. Criterion's custom-iteration timing consumes that duration, so both regions remain measurements of one real-engine path rather than a benchmark-only reconstruction of models, ledgers, stores, and channels.

## On the harness, not beside it · `sec:assayer:performance-harness-shared-scenario`

Every benchmark constructs a `World` through the shared scenario, drives the real `Assayer`, crosses declared barriers, and expresses inputs as harness specifications. This is the common vocabulary and real-engine boundary already fixed for test consumers (`dec:harness:single-scenario`) (`dec:harness:real-engine`). Setup that establishes a population is a guarded fixture: it checks dimensions, counts, mix, queue completion, and health before Criterion admits the sample (`dec:harness:guarded-fixtures`). Long setup streams use declarative playback with reproducible seeds; playback progress remains liveness evidence and never enters the measured duration (`dec:harness:declarative-playback`) (`dec:harness:seeded-sweeps`) (`cav:harness:progress-not-latency`).

The ordinary witnesses already reach much of that vocabulary. `World` owns a real engine, deterministic clock, seeded generator, names, policies, and barriers (`sec:assayer:testing-architecture-scenario`) (`sec:assayer:testing-architecture-clocks`); `WorldBuilder` and the lifecycle verbs own construction and registrations (`sec:assayer:testing-architecture-vocabularies`); `LabelSpec` and `PreSeedSpec` own individual payloads (`sec:assayer:testing-architecture-specs`); and the report fixtures own fixed valid Sentinel stimuli (`sec:assayer:testing-architecture-reports`). What is missing is population-scale composition and benchmark-safe observation, not another engine constructor.

| Hand-built witness piece | Shared-harness correspondence | What the harness must gain |
| --- | --- | --- |
| The three-action benchmark policy, configured infrastructure, engine, and shutdown | `WorldBuilder::channel`, `test_infrastructure`, the real `Assayer` inside `World`, and `World` drop already own them | A named performance-world fixture that selects a declared case and guards the resulting layout |
| A wall clock read directly by the stopwatch and the profile's `SystemClock` | Each `World` already owns one `VirtualClock`, shared with its engine | A fixed-clock case constructor that freezes the existing clock during a sample and guards its origin; no new clock implementation |
| The profile's `WorkingCopy`, operational, sister, anchor and outcome models, `SharedState`, model configuration, and publication version | The real engine behind `World` already owns and publishes this state | A reference-layout fixture, dependent on the complete host-declaration builder (`entry:assayer:harness-complete-builder`), that reaches declared widths without exposing private constructors |
| The profile's `OutcomeLedger`, pending assessment, stored features, calibration buffer, convergence tracker, health counters, drift counter, event channel, and identity maps | A request through `World` creates the pending entry, and the real label owner owns every downstream component | Seeded request-and-label populations plus a test-support label-region recorder reached through `World`; the recorder obeys the read-only, gated probe discipline (`dec:harness:probe-contract`) |
| The witnesses' implicit absence of persistence | A `World` with no persistence configuration already performs no durable writes | A named no-op persistence case that selects that existing mode and guards that no durable path is configured; an in-memory production abstraction is unnecessary |
| One fixed report shape and direct model widths | Golden and minimal report fixtures already build valid fixed shapes | A synthetic Sentinel-report generator parameterised by declared Sentinel count, slot width, cell count, and outcome-axis width, with structural acknowledgement and clean health as its guard |
| One `LabelSpec` or `PreSeedSpec` built at a time | Both fluent payload builders already exist | Seeded populations at declared sizes, adverse/benign mixes, eligibility, outcome-axis mixes, and entity cardinalities, returned only after their counts and mix are checked |
| A direct `BayesianLinearModel` trained and marginalised outside the scenario | `World` registration, assessment, labelling, and deregistration reach the lifecycle operation on the real engine | A guarded reference-dimension lifecycle fixture that proves the starting width, departing block width, non-trivial training, and non-fallback correction before yielding a timed deregistration case |

Fixture values live in one case table and never as unexplained benchmark literals. The implementation may tune Criterion sampling independently, but changing one of these populations is a reviewed change to the workload being compared.

| Case | Population and preparation | Dimension declaration | Label or entry mix | Channel policy |
| --- | --- | --- | --- | --- |
| `cold` | No labels, reports, or pre-seed entries | Full model width 16; anchor width 15 | None | Allow, Challenge, Block |
| `reference_steady` | Eight reporting Sentinels, one spatial outcome axis, two identity dimensions with about ten competitive cells each, and 1,276 eligible labels | Full model width 638; one departing Sentinel block is 68 coordinates | Seeded balanced benign/adverse ground truth, with every accepted label counted | Allow, Challenge, Block |
| `dense_label_profile` | Eight warm-up labels before each measured batch of 128 | Full model width 646 from 630 signal positions, with five non-spatial outcome axes and a 15-coordinate anchor | Every label is eligible and adverse, and every outcome axis is present | Allow, Challenge, Block |
| `preseed_thousand` | One bulk population of 1,000 entries on a fresh cold world | Full model width 16; anchor width 15 | One adverse ground-truth entry in every five, four benign | Allow, Challenge, Block |
| `health_reference` | The `reference_steady` population, with reports and labels settled before measurement | Full model width 638, with all health model rows present | The settled reference mix; no labels occur during the measurement | Allow, Challenge, Block |
| `reference_removal` | Fifty seeded eligible labels make the departing block non-trivial before each fresh removal sample | Full model width 638, removing 68 coordinates and retaining 570 | Alternating benign/adverse ground truth, with the correction guard satisfied | Allow, Challenge, Block |

The fixed clock, no-op persistence mode, report generator, and seeded populations are fixtures rather than engine doubles. Each returns only after its table row is true and reports every measured precondition on failure. The engine, label owner, health publication, pending store, ledger, calibration state, and lifecycle path remain the production implementations.

## The measurement contract · `sec:assayer:performance-harness-measurement-contract`

The `performance` bench target uses Criterion `0.8` with `html_reports` and declares `harness = false`, exactly the dependency line and target mode already used by the sibling packages. One target with six groups is the chosen layout. Sentinel demonstrates that related operations remain filterable as groups inside one target, while Mudlark's several targets separate independent algorithm families with different fixtures; Assayer has one real-engine scenario, shared population cases, and cross-group baselines, so multiple target crates would duplicate the very construction vocabulary this plan removes.

Throughput is declared wherever a natural unit exists: calls for assessment and health, labels for publication, entries for pre-seeding, cycles for construction/shutdown, and completed removals for marginalisation. Setup uses Criterion's setup or batched forms and is outside the measured interval; every asynchronous operation's declared barrier is inside the interval, so enqueue speed cannot stand in for completed work.

The historical budgets are floors under how slow an operation may become, not measurements of how fast it is. They remain beside the corresponding groups as documented baselines a reader compares with Criterion estimates and confidence intervals. No benchmark panics because a wall-clock estimate crossed one, no ordinary test asserts one, and no CI verdict is derived from one; correctness guards may still reject an invalid fixture before timing begins.

Every group warms for three seconds. Low-cost assessment and health identifiers take one hundred samples; label publication, construction/shutdown, pre-seed, and reference marginalisation take twenty because their samples include completed asynchronous or cubic work. The suite records Criterion's raw estimates, confidence intervals, change analysis, and HTML report, so a noisy point estimate is evidence with its distribution rather than a red status with no explanation.

Cargo executes benchmarks under its optimised bench profile, which inherits the release settings unless the workspace overrides it. The maintained manual command is `cargo bench -p torrust-assayer`. The package lint script names that command and verifies that the declared target remains present without running the expensive measurement; a scheduled or manually dispatched workflow runs the command and publishes the complete Criterion report as an artifact.

The existing deployment workflow already compiles every workspace bench target on both configured toolchains, so adding the target puts its imports and registrations under the push gate without making a shared runner a performance oracle. The measurement workflow gives each toolchain its own target directory and never shares compiled artifacts between toolchains. A scheduled run records toolchain, runner image, processor identity, load context, commit, and fixture case alongside its report; a suspicious movement is rerun once, and baseline revision requires comparable quiet-run evidence.

## Rejected alternatives · `sec:assayer:performance-harness-rejections`

Scheduling the ignored stopwatch tests as they stand is rejected. A standard-library timer around a hand-written loop produces one machine-dependent aggregate, and an asserted generous floor turns load into a test verdict while supplying no distribution, change estimate, or durable report.

Retiring the witnesses without replacement is rejected. The cost model makes per-assessment, per-label, memory, and lifecycle claims, so leaving the package with no maintained performance evidence would make those claims assumptions at exactly the reference sizes where their shape matters.

Keeping Criterion benchmarks off the shared harness is rejected. Private model constructors, ledgers, pending entries, channels, and clocks would create a second population of worlds that could drift from the scenario and bypass the real queue, publication, and lifecycle behaviour the harness contract exists to keep singular (`dec:harness:performance-benchmarks`).

## Risks and open questions · `sec:assayer:performance-harness-risks`

**Observation (Attribution needs a test-support region recorder)** · `obs:assayer:performance-harness-attribution-recorder`

Criterion can time the complete public label operation directly, but the dense update share lies inside the owner-thread function. The implementing fixture must add a test-support recorder whose activation, drain, and returned duration are reached through `World`, whose clock cannot alter product state, and whose measured region encloses the same model-update block as ordinary publication. Subtracting two independent end-to-end estimates is rejected because scheduler noise can make the inferred remainder negative or larger than the whole.

**Observation (A shared runner establishes trends, not universal latency)** · `obs:assayer:performance-harness-runner-noise`

Warm-up, controlled sample counts, fresh batched setup, and report artifacts make movement diagnosable but do not make two shared-server runs identical. The documented floors remain context, and a regression judgment reads confidence intervals and a comparable rerun rather than turning the shared runner into an absolute clock.

**Observation (Benchmark folders are absent from the projections)** · `obs:assayer:performance-harness-benchmark-coverage-gap`

The current test census enumerates only package `src` and top-level `tests` roots, and its classifier has only unit, crate, and integration areas. The per-file test index and per-folder matrix consume that same covered-asset set. A top-level ``benches/`` folder is therefore outside both projections today even though the wider code carrier can scan benchmark comments. Tooling must add a benchmark census, classification, standard place, index, matrix, and focused coverage tests before the migrated claims can be re-homed mechanically.

**Observation (Fixture setup must not leak into measurement)** · `obs:assayer:performance-harness-setup-boundary`

Reference convergence, report generation, pending-assessment preparation, and fresh-world construction can dominate the operations they prepare. Criterion batched setup owns those costs except in `construction_shutdown`, where construction and join are the declared subject; guards execute after setup and before the measured interval, while queue barriers that complete measured work remain inside it.

**Observation (Artifact retention remains a workflow choice)** · `obs:assayer:performance-harness-artifact-retention`

The workflow implementation must choose a retention period long enough to compare releases without treating an indefinitely retained runner-specific report as a product guarantee. That choice changes storage policy rather than benchmark semantics and does not block the target or fixtures.

## Acceptance · `sec:assayer:performance-harness-acceptance`

Implementation is accepted when every benchmark constructs and drives the real engine through `World`, the declared fixture table is the only source of workload literals, every state-establishing fixture checks its preconditions, and no benchmark imports a private constructor or authors an asynchronous wait (`dec:harness:performance-benchmarks`).

All seven ignored stopwatch tests are gone or converted according to the subject table. Their claim prose and historical floors live with the benchmark identifiers and appear in the linter's generated catalogue projections; no ignored performance test, orphaned performance claim, or dependency-only Criterion declaration remains.

The manifest carries Criterion `0.8` with `html_reports` and one `performance` target using `harness = false`. The deployment workflow compiles that target, the scheduled or manual workflow executes `cargo bench -p torrust-assayer` under the optimised profile and publishes the report, and the package lint script names the same command and verifies the route without timing it.

The linter covers the top-level benchmark folder in its census, test index, coverage join, and per-folder matrix before benchmark claims move there. Projection generation and corpus checks are green in both directions, including the new benchmark area and the absence of the retired crate-test mints.

## The partition · `sec:assayer:performance-harness-partition`

Four independently gate-able ENGINEERING entries carry the implementation. The tooling and fixture lanes may run in parallel; the benchmark lane waits for both, and the CI lane waits for the benchmark target.

| Backlog entry | Lane-sized result | Dependencies |
| --- | --- | --- |
| Benchmark projection coverage (`entry:assayer:performance-benchmark-projections`) | The linter recognises Criterion benchmark functions under the benchmark root, derives their labels, and projects their file index, folder matrix, and claim coverage | None; lands before claims migrate |
| Performance fixtures and fakes (`entry:assayer:performance-harness-fixtures`) | Fixed-clock and no-persistence cases, synthetic reports, seeded label/pre-seed populations, reference layouts, lifecycle guards, and the label-region recorder join `torrust_assayer::testing` | The existing complete-builder obligation (`entry:assayer:harness-complete-builder`) for the reference layout |
| Criterion target and stopwatch retirement (`entry:assayer:performance-criterion-target`) | The `performance` target implements all six groups, moves the seven claims and documented floors, and deletes the ignored stopwatch tests | Benchmark projection coverage and performance fixtures |
| Scheduled performance route (`entry:assayer:performance-ci-route`) | Deployment compilation, scheduled/manual measurement artifact, per-toolchain build isolation, and lint-script command naming form one maintained route | Criterion target and stopwatch retirement |
