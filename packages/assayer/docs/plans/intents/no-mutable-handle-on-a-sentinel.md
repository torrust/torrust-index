# Keeping Mutable Sentinel Handles Outside the Assayer · `plan:assayer:intent-no-mutable-handle-on-a-sentinel`

This plan keeps the promise that the Assayer's interpretation layer receives detached measurement values and has no type-level route through which it can mutate a Sentinel, its configuration, or its spatial graph (´claim:assess:no-mutable-handle-on-a-sentinel´).

## What the promise says, precisely · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-promise`

The subject is every production value stored by `Assayer`, every argument and result on its Sentinel registration, report-reception, and assessment surfaces, and every value those surfaces place into `RequestContext`, the report cache, or an assessment. None may be a `SpectralSentinel`, a Sentinel alias, a Sentinel configuration, a mutable reference to one, or an owning smart pointer that affords equivalent mutation.

The permitted measurement crossing is an owned `BatchReport<u128>` paired with a `SentinelId`. The report is a detached statement already produced by the Sentinel, and report reception atomically installs the Core's own index over that value (´alg:runtime:report-reception´); the normative consumed-report inventory is (´tab:boundary:consumed´), and the upstream records deliberately refused by the Core are (´tab:boundary:not-consumed´).

The prohibited crossing is any path from Core to Sentinel or from Core to a Sentinel-owned graph. Both directions are `Never` in the complete boundary inventory (´tab:boundary:verification´), implementing the unconditional one-way flow from measurement through interpretation to the host (´inv:guarantee:feed-forward´).

The relevant quantities are Sentinel-owned configuration and spatial state, not assessment outputs: the split threshold, creation and eviction depth gates, graph budget, importance, and partition structure remain on the Sentinel's side of the wall (´tab:ownership:sentinel-side-inventory´). The witness has no numeric tolerance; it requires a route to those quantities to be absent from the Assayer's type surface, so an attempted mutable crossing is rejected before execution.

The Core's own identity graphs are outside that prohibition. An assessment may enqueue a fixed unit observation for deferred processing under the identity observation protocol (´alg:keyspace:observation-protocol´), while the assessment itself writes no identity graph importance, topology, or competitive set (´inv:runtime:enumerated-writes´). A witness that rejected all graph mutation would erase this deliberate distinction and test a stronger, false promise.

The ownership record fixes report transfer as an owned-value crossing and rejects callbacks or retained handles (´dec:ownership:feed-forward´). It also rejects the handle alternative because that alternative turns the guarantee into call-site discipline (´disc:ownership:handle-alternative´) and requires mechanical enforcement for structural boundaries (´dec:ownership:enforcement´).

## What the code offers today · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-current-code`

The public report-reception surface exposes `Assayer::receive_sentinel_report(&self, SentinelId, BatchReport<u128>) -> Result<ReportAck, ReportError>`; the report crosses by value, and the method lends the report plus Assayer-owned slots, Ledger, configuration, and clock to `report::ingestion::ingest_report` (´alg:runtime:report-reception´).

The public `SentinelRegistration` type contains only `id` and `name`. Registration names a measurement source and allocates Core-owned model and Ledger structure; it does not attach the producing object.

The public `RequestContext` type carries an `EntityKey`, a `sentinel_coordinates` map from `SentinelId` to `u128`, a signal map, and a crate-private test-harness channel hint. The Sentinel-facing value is only the identifier-keyed coordinate; no field carries the producing object, a callback, or its configuration, a shape exercised by (´test:integration:request-context-with-sentinels´).

The engine stores `sentinel_slots` as a map from `SentinelId` to `SentinelSlot`; each slot owns a published `ReportIndex`, ingestion acknowledgement, ingestion lock, counters, and bootstrap state rather than a Sentinel (´dec:surface:slot-map´). Outside the test-gated `testing` module, the only direct `torrust_sentinel` imports are the report records used by `api::report`, `report::ingestion`, and `report::validation`; no production module names `SpectralSentinel`, `Sentinel128`, or `Sentinel64`.

The finished test surface is unified around `Scenario` and `World`. `World::register_sentinel` supplies identifier and name, `World::receive_report` supplies an owned `BatchReport<u128>`, `World::assess` supplies a `RequestContext`, time moves through `World::advance` or `World::travel_to`, and completion is expressed by `World::flush_observations`, `World::flush_labels`, or `World::flush_identity_maintenance` for the queue concerned (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:single-scenario´) (´dec:harness:no-ad-hoc-waits´).

The same roster now provides the owned probes `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView` (´dec:harness:probe-contract´); declarative `PlaybackRow` execution through `playback` under a `PlaybackBarrierPolicy`, with `PlaybackCheckpoint` and `PlaybackProgress` (´dec:harness:declarative-playback´); `OracleProvenance`, `pairwise_rank`, `regularised_schur_complement`, `decay_recurrence`, and `dimension_width` (´dec:harness:oracle-tier´); guarded `TrainedStateFixture` and `TrainedStateBaseline` setup with separate result validation (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´); and `run_seeded_sweep` (´dec:harness:seeded-sweeps´). None supplies evidence about a compile-time absence, so this witness needs no scenario, time movement, barrier, reading, playback, numeric oracle, trained fixture, or seed sweep.

The nearest runtime tests prove that a report value is accepted after registration (´test:crate:report-accepted-after-registration´) and that registration churn can race live assessments safely (´test:integration:concurrent-assess-under-lifecycle-change-returns-valid-assessments´). The request-context test proves coordinate carriage, and these lifecycle tests prove report and assessment behaviour; all can remain green after a second mutable-handle route is added because none asks the compiler to reject that route or cites this claim.

The standing source audits keep the adjacent Core-to-derivation boundary closed through import and token scans (´test:crate:resonance-module-does-not-reference-core-model-state´), (´test:crate:core-model-modules-do-not-import-decision-layer´), and (´test:crate:core-and-api-paths-do-not-call-derivation´). They establish the repository's fail-closed scanning pattern but do not inspect the Sentinel type surface.

The standing testing plan records the choice between structural audit and compilation failure under (´obs:assayer:reframe-mutable-handle´); this plan selects a compilation-failure witness backed by a closed Sentinel source-surface audit.

The request-context documentation contains a Rustdoc `compile_fail` example for the absence of a channel field. No package-level diagnostic-checking UI driver or fixture exists, and the package manifest declares no compile-fail test dependency.

## The witness · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-witness`

The witness is joint: a UI test proves that the actual report-reception boundary refuses a mutable Sentinel where it accepts a detached report, and a crate-level source audit ensures that a differently named direct upstream handle cannot bypass that one probe.

- Setup: a UI fixture imports `Assayer` and `SentinelId` from `torrust_assayer` and `Sentinel128` from `torrust_sentinel`, then declares a function taking `&Assayer`, `SentinelId`, and `&mut Sentinel128`. It needs no constructor, `World`, clock, thread, report fixture, label, or runtime execution.

- Stimulus: the fixture passes the mutable Sentinel argument as the second payload argument to `Assayer::receive_sentinel_report`, the concrete public entry point through which Sentinel measurements cross today.

- Compile-time observation: the compiler reports that the method requires an owned `BatchReport<u128>` and received a mutable Sentinel reference. The UI driver requires this fixture to fail compilation and pins the diagnostic at the payload type mismatch rather than at unrelated setup.

- Surface observation: a crate test walks production modules with `SourceTreeScanner`, excludes the crate-test and test-gated harness trees, admits only the three exact existing `torrust_sentinel` report-record import lines in `api::report`, `report::ingestion`, and `report::validation`, and rejects every other import or fully qualified occurrence. A changed import list, a wildcard import, or a new module is rejected rather than interpreted, so the allowlist fails closed over direct dependency paths.

- Assertion: the UI driver cites the intent and succeeds only while the forbidden crossing does not type-check; the source audit cites the same intent and succeeds only while the production source names upstream report values and no upstream engine, alias, configuration, callback, or mutable-handle carrier.

- Fails-before: a deliberately weakened `receive_sentinel_report` that accepts `&mut Sentinel128` makes the negative fixture compile, causing the UI driver to report that a case expected not to compile succeeded. A weakened design that adds another direct upstream handle through a method, field, alias, or module instead changes or adds a dependency occurrence and fails the closed source-surface audit.

The compile-fail case does not ingest, assess, set a threshold, or force a repartition. Compilation is the observation, and successful compilation under the deliberately weakened signature is the counterexample.

## What is missing · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-missing`

**Entry (The labeled UI compile-fail runner)** · `entry:assayer:intent-no-mutable-handle-on-a-sentinel-ui-runner`

A `trybuild` dev dependency, the integration target `no_mutable_handle_on_a_sentinel`, and the test `no_mutable_handle_on_a_sentinel::mutable_sentinel_is_not_report_payload` add the diagnostic-checking runner, its module test index, one UI fixture, and the checked diagnostic. The test documentation cites the promise and this plan; the entry depends on no runtime harness capability or other intent.

**Entry (The closed Sentinel import surface)** · `entry:assayer:intent-no-mutable-handle-on-a-sentinel-source-surface`

The crate test `tests::source_audit::production_source_names_only_detached_sentinel_reports` reuses `SourceTreeScanner` to exclude `tests` and `testing`, admit the exact report-value import lines already required by production, and reject every other direct dependency occurrence. Its documentation cites the promise and this plan; it depends on (´entry:assayer:intent-no-mutable-handle-on-a-sentinel-ui-runner´) only for the joint coverage statement, not for implementation.

## Risks and open questions · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-risks`

**Observation (A named negative probe is not an existential proof)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-negative-space`

A UI fixture rejects one concrete crossing, not every method name a future API could invent. The closed production-source allowlist is therefore part of the witness rather than optional reinforcement; either half alone leaves a route the other half covers.

**Observation (The source half remains textual)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-textual-limit`

The scanner reads source tokens rather than compiler metadata, so macro expansion or an indirect re-export can evade it, the same limitation already recorded for the structural guard (´cav:ownership:lint-scope´). The UI half covers the live reception signature; introducing a macro or re-export route therefore requires a corresponding compiler-facing negative case rather than a claim that the textual scan covers it.

**Observation (Compiler diagnostics are a maintained fixture)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-diagnostic-stability`

The semantic oracle is failure to compile, while the UI diagnostic snapshot localises that failure. A compiler wording or span change can require an expected-output refresh without weakening the boundary; the implementation report distinguishes such maintenance from a fixture that unexpectedly compiles.

**Observation (No runtime variability enters the result)** · `obs:assayer:intent-no-mutable-handle-on-a-sentinel-no-runtime-oracle`

There are no samples, tolerances, thresholds, waits, clocks, schedules, random draws, or concurrent publications in the witness. Its only variable input is the compiled public type surface, so numerical fragility, timing, and nondeterminism are absent.

No design decision remains open: the specification identifies report reception as the complete Sentinel-to-Core crossing, the code exposes that crossing directly, and the existing source-audit machinery supplies the walk needed for the complementary direct-path check.

## Acceptance · `sec:assayer:intent-no-mutable-handle-on-a-sentinel-acceptance`

The implementation report identifies integration target `no_mutable_handle_on_a_sentinel`, function `no_mutable_handle_on_a_sentinel::mutable_sentinel_is_not_report_payload`, its UI fixture and checked diagnostic, the `trybuild` dev dependency, and crate test `tests::source_audit::production_source_names_only_detached_sentinel_reports`; confirms both test documentation environments cite the promise and this plan; and shows the generated test indexes and coverage report resolving the promise to the joint witness.

The report shows the UI fixture rejected specifically because `Assayer::receive_sentinel_report` requires an owned `BatchReport<u128>` where the fixture supplies `&mut Sentinel128`, and shows the source audit accepting only the three exact production report-import lines while excluding the test-gated harness. It also shows the package's prescribed formatting, lint, documentation, feature, and test gates passing, with unrelated failures separated explicitly.

The report states the inverted fails-before result: changing the reception boundary to accept the mutable Sentinel makes the fixture compile and therefore makes the compile-fail driver fail, while adding a differently named path makes the source audit fail.

Acceptance also records that `SentinelRegistration` still contains only identifier and name, the Sentinel-facing portion of `RequestContext` still contains only identifier-keyed coordinates, the Assayer still stores `SentinelSlot` values rather than Sentinel instances, and the distinction between unreachable Sentinel-owned graphs and mutable Assayer-owned identity graphs remains intact.
