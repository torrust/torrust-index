# Keeping Mutable Sentinel Handles Outside the Assayer · `plan:assayer:intent-no-mutable-handle-on-a-sentinel`

This plan keeps the promise that the Assayer's interpretation layer receives detached measurement values and has no type-level route through which it can mutate a Sentinel, its configuration, or its spatial graph (´claim:assess:no-mutable-handle-on-a-sentinel´).

## What the promise says, precisely · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-promise`

The subject is every production value stored by `Assayer`, every argument and result on its Sentinel registration, report-reception, and assessment surfaces, and every value those surfaces place into `RequestContext`, the report cache, or an assessment. None may be a `SpectralSentinel`, a Sentinel alias, a Sentinel configuration, a mutable reference to one, or an owning smart pointer that affords equivalent mutation.

The permitted measurement crossing is an owned `BatchReport<u128>` paired with a `SentinelId`. The report is a detached statement already produced by the Sentinel, and report reception atomically installs the Core's own index over that value (´alg:runtime:report-reception´); the normative consumed-report inventory is (´tab:boundary:consumed´), and the upstream records deliberately refused by the Core are (´tab:boundary:not-consumed´).

The prohibited crossing is any path from Core to Sentinel or from Core to a Sentinel-owned graph. Both directions are `Never` in the complete boundary inventory (´tab:boundary:verification´), implementing the unconditional one-way flow from measurement through interpretation to the host (´inv:guarantee:feed-forward´).

The relevant quantities are Sentinel-owned configuration and state, not assertion outputs: the split threshold, creation and eviction depths, graph budget, temporal policy, graph importance, and partition topology. The witness has no numeric tolerance; it requires those quantities to be unnameable through an Assayer-held handle, so changing any of them from the interpretation side is rejected before execution.

The Core's own identity graphs are outside that prohibition. An assessment may enqueue a fixed unit observation for deferred processing under the identity observation protocol (´alg:keyspace:observation-protocol´), while the assessment itself writes no identity graph importance, topology, or competitive set (´inv:runtime:enumerated-writes´). A witness that rejected all graph mutation would erase this deliberate distinction and test a stronger, false promise.

The ownership record fixes report transfer as an owned-value crossing and rejects callbacks or retained handles (´dec:ownership:feed-forward´). It also rejects the handle alternative because that alternative turns the guarantee into call-site discipline (´disc:ownership:handle-alternative´) and requires mechanical enforcement for structural boundaries (´dec:ownership:enforcement´).

## What the code offers today · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-current-code`

The public report-reception surface exposes `Assayer::receive_sentinel_report(&self, SentinelId, BatchReport<u128>)`; the report crosses by value, and the method passes only a shared Assayer reference plus a borrowed report value to the internal ingestion function (´alg:runtime:report-reception´).

The public `SentinelRegistration` type contains only `id` and `name`. Registration names a measurement source and allocates Core-owned model and Ledger structure; it does not attach the producing object.

The public `RequestContext` type contains a map from `SentinelId` to `u128` coordinate. The request therefore identifies where to read inside a cached report without carrying the Sentinel that produced it, a shape exercised by (´test:integration:request-context-with-sentinels´).

The engine stores `sentinel_slots` as a map from `SentinelId` to the package's report slot (´dec:surface:slot-map´). The production source contains no `SpectralSentinel`, `Sentinel128`, or `Sentinel64` occurrence; its direct upstream imports name report records only.

The world harness mirrors the same boundary: `World::register_sentinel` supplies identifier and name, `World::receive_report` supplies an owned batch report, and `World::assess` supplies a request context. The completed harness skeleton provides these runtime verbs (´entry:assayer:harness-stage-skeleton´), but its virtual clock, seeded random source, drift budgets, publication barriers, and liveness deadlines provide no oracle for a compile-time absence.

The nearest runtime tests prove that report values are accepted after registration (´test:crate:report-accepted-after-registration´) and that registration churn can race live assessments safely (´test:integration:concurrent-assess-under-lifecycle-change-returns-valid-assessments´). Both can remain green after a mutable handle is added, because neither asks the compiler to reject that new route.

The standing source audits keep the adjacent Core-to-derivation boundary closed through import and token scans (´test:crate:resonance-module-does-not-reference-core-model-state´), (´test:crate:core-model-modules-do-not-import-decision-layer´), and (´test:crate:core-and-api-paths-do-not-call-derivation´). They establish the repository's fail-closed scanning pattern but do not inspect the Sentinel type surface.

The standing testing plan has no Entry covering this promise. It instead records the choice between structural audit and compilation failure under (´obs:assayer:reframe-mutable-handle´); this plan selects a compilation-failure witness backed by a closed source-surface audit.

The request-context documentation already contains a Rustdoc `compile_fail` example for the absence of a channel field. No package-level UI-test driver or UI fixture directory exists, and the package manifest declares no compile-fail test dependency.

## The witness · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-witness`

The witness is joint: a UI test proves that the actual report-reception boundary refuses a mutable Sentinel where it accepts a detached report, and a crate-level source audit ensures that a differently named handle cannot bypass that one probe.

- Setup: a UI fixture imports `Assayer`, `SentinelId`, and `Sentinel128`, then declares a function taking `&Assayer` and `&mut Sentinel128`. It needs no constructor, `World`, clock, thread, report fixture, label, or runtime execution.

- Stimulus: the fixture passes the mutable Sentinel argument as the second payload argument to `Assayer::receive_sentinel_report`, the concrete public entry point through which Sentinel measurements cross today.

- Compile-time observation: the compiler reports that the method requires an owned `BatchReport<u128>` and received a mutable Sentinel reference. The UI driver requires this fixture to fail compilation and pins the diagnostic at the payload type mismatch rather than at unrelated setup.

- Surface observation: a crate test walks the production source tree with `SourceTreeScanner`, admits only the existing report-record imports at their current ingestion, validation, and public-reception homes, and rejects every other `torrust_sentinel` import or fully qualified occurrence. Wildcard imports are rejected rather than interpreted, so the allowlist fails closed when the dependency exports another type.

- Assertion: the UI driver cites the intent and succeeds only while the forbidden crossing does not type-check; the source audit cites the same intent and succeeds only while the production source names upstream report values and no upstream engine, alias, configuration, callback, or mutable-handle carrier.

- Fails-before: a deliberately weakened `receive_sentinel_report` that accepts `&mut Sentinel128` makes the negative fixture compile, causing the UI driver to fail with the inverted verdict that a case expected not to compile succeeded. A weakened design that adds the handle through another method, field, alias, or module instead fails the closed source-surface audit.

The compile-fail case does not ingest, assess, set a threshold, or force a repartition. Compilation is the observation, and successful compilation under the deliberately weakened signature is the counterexample.

## What is missing · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-missing`

**Entry (The labeled UI compile-fail runner)** · `entry:assayer:intent-no-mutable-handle-on-a-sentinel-ui-runner`

A package dev-dependency and a focused driver of roughly twenty lines add the compile-fail runner, its module test index, the intent citation, and one UI fixture with its expected diagnostic. The runner is a new compile-fail integration module, the fixture and diagnostic live in a new UI-fixture directory, and the dependency is declared in the package manifest; it depends on no runtime harness stage or sibling intent lane.

**Entry (The closed Sentinel import surface)** · `entry:assayer:intent-no-mutable-handle-on-a-sentinel-source-surface`

A crate test of roughly forty lines in the existing source-audit module reuses `SourceTreeScanner` to admit the exact report-value imports already required by production and reject every other direct dependency occurrence. It depends on (´entry:assayer:intent-no-mutable-handle-on-a-sentinel-ui-runner´) only for the joint witness's final coverage statement, not for its implementation, and on no other lane.

## Risks and open questions · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-risks`

**Observation (A named negative probe is not an existential proof)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-negative-space`

A UI fixture rejects one concrete crossing, not every method name a future API could invent. The closed production-source allowlist is therefore part of the witness rather than optional reinforcement; either half alone leaves a route the other half covers.

**Observation (The source half remains textual)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-textual-limit`

The scanner reads source tokens rather than compiler metadata, so macro expansion or an indirect re-export can evade it, the same limitation already recorded for the structural guard (´cav:ownership:lint-scope´). The UI half covers the live reception signature, and acceptance requires any future macro or re-export mechanism to extend the allowlist scanner or add a corresponding UI case rather than declaring the original scan exhaustive.

**Observation (Compiler diagnostics are a maintained fixture)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-diagnostic-stability`

The semantic oracle is failure to compile, while the UI diagnostic snapshot localises that failure. A compiler wording or span change can require an expected-output refresh without weakening the boundary; the implementation report distinguishes such maintenance from a fixture that unexpectedly compiles.

**Observation (No runtime variability enters the result)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-no-runtime-oracle`

There are no samples, tolerances, thresholds, waits, clocks, schedules, random draws, or concurrent publications in the witness. Its only variable input is the compiled public type surface, so numerical fragility, timing, and nondeterminism are absent.

No maintainer decision remains open: the specification identifies report reception as the complete Sentinel-to-Core crossing, the code exposes that crossing directly, and the existing source-audit machinery supplies the complementary closed-world check.

## Acceptance · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-acceptance`

The implementing lane's report identifies the UI driver, fixture, expected diagnostic, source-audit addition, and package dev-dependency; confirms both test documentation environments cite the intent; and shows the generated test indexes and coverage report resolving the intent to the joint witness.

The report shows the UI fixture rejected specifically because `receive_sentinel_report` requires an owned `BatchReport<u128>`, the closed source-surface audit passing over the production source tree, and the package's prescribed formatting, lint, documentation, feature, and test gates passing, with unrelated failures separated explicitly.

The report states the inverted fails-before result: changing the reception boundary to accept the mutable Sentinel makes the fixture compile and therefore makes the compile-fail driver fail, while adding a differently named path makes the source audit fail.

Acceptance also records that `SentinelRegistration` still contains only identifier and name, `RequestContext` still contains only identifier-keyed coordinates, the Assayer still stores report slots rather than Sentinel instances, and the distinction between unreachable Sentinel-owned graphs and mutable Assayer-owned identity graphs remains intact.
