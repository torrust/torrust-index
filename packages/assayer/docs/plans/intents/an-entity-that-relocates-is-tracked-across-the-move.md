# Tracking a Relocating Entity Without a Rating Gap · `plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move`

This plan keeps the promise (´claim:identity:an-entity-that-relocates-is-tracked-across-the-move´) by establishing that one semantic entity retains an adverse rating while its regional identity cells, Sentinel routing, and spatial outcome memory hand responsibility from an abandoned region to a destination.

## What the promise says, precisely · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-promise`

The executable entity key is composite: a stable facet $s$ names the entity and a regional facet $r$ names its current location. One identity dimension encodes $s$ and another encodes $r$, matching the public multi-encoder use of one opaque key while respecting that the identity layer partitions host-declared ranges rather than maintaining primary per-entity state (´mot:keyspace:purpose´).

Let $A$ and $B$ be disjoint regional prefixes beneath a shared ancestor $P$, let $C_s(t)$ and $C_r(t)$ be the active stable and regional identity-cell chains recorded for an assessment, and let $K_L(x,t)$ and $b_L(x,t)$ be the deepest Outcome Ledger key and its decayed adverse rate at coordinate $x$. Assessment computes both identity coordinates, queues their observations, and reads the currently published competitive sets (´alg:keyspace:observation-protocol´).

For a target $T$ and benign control $C$ matched on every non-identity input, define the public separation $\Delta(t)=p_T(t)-p_C(t)$. A guarded baseline must establish non-empty stable and $A$-regional chains, exact routing to the reported $A$ leaf, non-zero target outcome state in both identity dimensions and the Ledger, and a positive $\Delta(t_0)$ larger than `World::tol().default`. The specification defines no absolute probability at which “high-risk” begins, so the witness carries this observable target-over-control ordering across every handover boundary rather than inventing a policy threshold.

The move changes only $r$ and the request's Sentinel coordinate. The stable chain remains active; the old regional cells leave and destination cells enter through the competitive lifecycle (´alg:keyspace:cell-entry´) (´alg:keyspace:cell-exit´); and every assessed boundary keeps $\Delta(t)$ positive by more than `World::tol().default`.

The Sentinel report sequence is $R_A=\{\text{root},P,A\}$, $R_P=\{\text{root},P\}$, and $R_B=\{\text{root},P,B\}$. Under $R_P$, both report routing and the Ledger resolve $B$ through $P$; reception of $R_B$ creates the exact $B$ Ledger entry before publishing the report that can route to it (´req:publication:report-reception´).

Every adverse label updates the deepest present Ledger entry and all of its ancestors (´alg:ledger:all-layers-update´), so destination traffic raises both $b_L(B)$ and the destination regional identity outcome rate from neutral creation while $P$ remains informed. Ledger reads always find the permanent root, and the depth walk returns the finest surviving entry rather than a fabricated neutral value (´def:ledger:root-semantics´) (´dec:memory:depth-walk´).

With no new label over $h$ elapsed hours, a retained abandoned identity outcome value $q_I$ and Ledger adverse rate $q_L$ satisfy $q_I(h)=q_I(0)\,0.998^h$ and $q_L(h)=q_L(0)\,0.999^h$. The identity value is about halved at fourteen days, while the Ledger value is about halved at twenty-nine days and about one eighth of its peak at eighty-seven days; exact assertions use the hourly powers rather than those descriptive horizons (´tab:keyspace:decay-rates´) (´def:ledger:time-decay´) (´tab:config:temporal´).

The report sequence gives $A$ one consecutive absence at $R_P$ and a second at the first $R_B$. No further report arrives during the decay readings, and the guarded baseline requires the predicted eighty-seven-day Ledger value to remain above the collection floor, so garbage collection cannot erase the quantity being measured (´alg:ledger:garbage-collection´). One further $R_B$ is then the third consecutive absence and deletes $A$ under the configured $N_{\mathrm{absent}}=3$ rule (´alg:ledger:entry-deletion´) (´tab:config:ledger´).

The promised composition is not instantaneous fine-cell replacement. Source displacement permits a bounded structural detection gap and requires degraded-resolution ancestor coverage throughout it; the witness therefore distinguishes the stable identity rating and $P$-level Ledger evidence from eventual destination takeover (´scenario:monitoring:displacement´).

## What the code offers today · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-code`

The production path is `Assayer::register_identity_dimension`, `Assayer::receive_sentinel_report`, `Assayer::assess`, `Assayer::label`, and `Assayer::full_health_report`. `IdentityDimensionRegistration` accepts an encoder from `EntityKey` to a coordinate, so two encoders can read the stable and regional facets of one composite key without a production mutation.

The unified harness supplies `World::trained_state`, `World::assayer`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::label`, `World::flush_observations`, `World::flush_labels`, `World::flush_identity_maintenance`, `World::advance`, `World::travel_to`, `World::block_model_owner_for_test`, `World::tol`, and `make_cell_report`. Scenario time moves both timestamp domains forward and returns only after identity maintenance, accepted lifecycle publication, and one Ledger collection cycle settle; tests no longer need a clock accessor, sleep, poll, or local deadline (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:no-ad-hoc-waits´).

`PlaybackRow`, `PlaybackBarrier`, `PlaybackBarrierPolicy`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback` provide declarative long stimuli with per-row queue boundaries, checkpoints, and progress (´dec:harness:declarative-playback´). `TrainedStateFixture` and `TrainedStateBaseline` provide a measured converged starting state, with setup validation kept separate from the relocation result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´). `OracleProvenance::SpecificationFormula` and `decay_recurrence` provide the independent multiplicative-decay calculation (´dec:harness:oracle-tier´).

The identity-maintenance barrier now drains prior observations, detects competitive changes, publishes the competitive index, and waits for accepted lifecycle work to reach the model owner. Forward scenario time and lifecycle publication are exercised by the harness and seeded publication witnesses (´test:crate:scenario-advance-moves-both-clock-domains-together´) (´test:crate:lifecycle-publication-seeded-sweep´).

The shipped projections are `PublishedModelBlock`, `PublishedSlotMoments`, and `PendingEntryView`. They obey the owned, read-only, one-load probe contract, but `PendingEntryView` exposes only active-cell counts, and none of the three exposes an assessment's exact identity chains, a retained identity outcome value, a Sentinel route key, or one nominated Ledger entry (´dec:harness:probe-contract´) (´entry:assayer:harness-probe-contract´).

Focused tests already establish mixed identity exit and entry in one maintenance submission, report-cell creation before index publication, pure Ledger decay on the twenty-nine-day timeline, and identity re-entry with retained outcome state (´test:crate:batch-entry-exit´) (´test:crate:cell-set-before-index-swap´) (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´) (´test:unit:competitive-exit-resets-measurement-and-reentry-retains-outcome´). No test mints or cites the relocation claim, and none composes those mechanisms into one public risk trajectory.

## The witness · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-witness`

The integration target is `identity_layer` and the function is `an_entity_that_relocates_is_tracked_across_the_move`, beside the existing lifecycle scenarios. Its module index describes the composed handover rather than restating any one focused mechanism (´test:integration:identity-churn-preserves-finite-assess-and-derive´).

- Guarded setup: start from `World::trained_state` and retain its measured `TrainedStateBaseline`, then register one spatial outcome axis, one Sentinel, and stable and regional identity dimensions whose encoders read separate fields of the same composite entity key. The subject-specific setup returns only after it has measured the declared report shape, stable and $A$ regional chains, non-zero target identity and Ledger outcome state, a positive matched-control separation, and an eighty-seven-day Ledger projection above the collection floor (´dec:harness:guarded-fixtures´).

- Initial report: construct $R_A$ with `make_cell_report`, ingest it through `World::receive_report`, and require its `ReportAck` to count the three declared cells with no degradation. The fixture uses no report cell beyond root, $P$, and $A$, so later creation and deletion counts have only one possible non-root referent.

- Initial population: send the fixed target/control population through `playback` using a subject-owned `PlaybackRow` and `PlaybackBarrierPolicy::new` ordered as `PlaybackBarrier::FlushObservations`, `PlaybackBarrier::FlushIdentityMaintenance`, then `PlaybackBarrier::FlushLabels`. A `PlaybackCheckpoint` validates the guarded baseline after the declared final row, while `PlaybackProgress` identifies any incomplete row; no loop terminates on a polled state (´dec:harness:declarative-playback´) (´test:crate:selected-label-barrier-completes-before-checkpoint´).

- Initial assertion: assess the target and matched control while `World::block_model_owner_for_test` holds model publication, record each assessment's published snapshot version and exact active-cell chains, and release the guard before any barrier. Require both assessments to use one model version, the target to route through the stable and $A$ regional chains and the $A$ Ledger leaf, every seeded outcome rate to be non-zero, and the target/control separation to exceed `World::tol().default`.

- Detection-gap boundary: ingest $R_P$, assess composite keys $(s_T,B)$ and $(s_C,B)$ before explicitly crossing `World::flush_identity_maintenance`, and capture each assessment's active chains and routing. The stable target chain remains, no destination regional cell is yet assumed, both Sentinel and Ledger routes are $P$, both risks use one held model version, and the target remains above the control.

- Regional takeover: drive a fixed, finite sequence of $B$ rows and forward intervals, with every time step expressed by `World::advance`, through the same playback policy. At declared checkpoints, require at least one $A$ regional exit and one $B$ regional entry, then assess the target and control under one held model version and require the stable chain, destination regional chain, and positive separation. The schedule is data, not a retry-until-settled loop (´dec:harness:no-ad-hoc-waits´).

- Destination report and learning: ingest $R_B$, require `ReportAck::cells_created_in_ledger` to be one and `ReportAck::cells_deleted_from_ledger` to be zero, then play the declared destination label rows under the same three-barrier policy. The final checkpoint requires the exact $B$ Ledger entry and destination regional identity outcome to exceed `World::tol().ewma`, requires $P$ to remain non-zero, and repeats the matched risk comparison.

- Decay observation: submit no label or report and use `World::travel_to` to reach cumulative fourteen-, twenty-nine-, fifty-eight-, and eighty-seven-day checkpoints. For every retained abandoned identity state and for the $A$ Ledger state, compare the owned read with `decay_recurrence(q_0, 1.0, 0, \gamma_t, h)` within `World::tol().ewma`; a graph eviction is accepted only as terminal absence of the old identity state, never as a neutral value while that state remains. The Ledger entry must remain present through the eighty-seven-day reading by the fixture's collection-floor guard (´dec:harness:oracle-tier´) (´test:crate:decay-oracle-applies-each-clock-as-a-recurrence´).

- Retirement boundary: ingest one further $R_B$, require `ReportAck::cells_deleted_from_ledger` to be one, require the exact $A$ key to be absent, and require routing at $A$ to fall back to $P$ or root. A final target/control pair must retain the stable chain, the positive public separation, and clean health.

- Fails-before: deriving both dimensions from the mutable region loses stable identity evidence; publishing $R_B$ before Ledger creation exposes a report route without its exact entry; leaf-only label routing leaves $P$ neutral; omitting destination outcome updates leaves the new identity and Ledger states at zero; using label count or wall time for decay misses the independent hourly recurrences; and deleting at either the second or fourth absence fails the exact acknowledgement schedule.

## What is missing · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-missing`

**Entry (Assessment-bound active-cell reading)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-identity-barrier`

`testing::probes` must extend `PendingEntryView` with owned `CompetitiveCellId` chains keyed by `DimensionId`, copied from the same single pending-entry lookup that already supplies their counts. This binds each risk reading to the identity publication it actually used and avoids pretending that a playback batch or two adjacent assessments share a view (´dec:harness:probe-contract´) (´cor:ordering:tape-batches´).

**Entry (Exact relocation-state projection)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-state-probe`

`testing::probes` must add an owned, test-gated relocation projection that takes named dimension coordinates, one Sentinel coordinate, and nominated identity and Ledger keys. After `World::flush_identity_maintenance` and `World::flush_labels`, it loads each independently published index once, copies the active chains and route key, reads decayed identity and Ledger adverse rates at the scenario timestamp, reports nominated-key presence, and exposes no borrow or mutation path. It reports each source's version or boundary separately and does not claim a global atomic snapshot (´dec:harness:probe-contract´).

**Entry (Composite relocation fixture and integration witness)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-integration-witness`

The `identity_layer` target must add the two-field encoders, the $R_A/R_P/R_B$ report builder, a subject-owned `PlaybackRow`, the guarded relocation baseline layered on `TrainedStateFixture`, the fixed handover schedule, and `an_entity_that_relocates_is_tracked_across_the_move`. Setup reports its measured state through `TrainedStateBaseline` and the relocation-specific guard, while the result assertions use the projection and `decay_recurrence`; the two responsibilities remain separate (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

## Risks and open questions · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-risks`

**Observation (Relocation needs two key facets)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-entity-semantics`

Keeping the same opaque `EntityKey` while changing only a Sentinel coordinate cannot make an identity competitive cell leave or enter, while replacing the whole key gives the Core no basis for relating the two requests. The composite key preserves a stable facet and changes a regional facet, which is the only reading that exercises the stable-rating and regional-turnover clauses together. If “region” means only a Sentinel coordinate, the identity entry and exit clause needs a narrower promise rather than a different fixture.

**Observation (High-risk has no absolute cut)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-risk-threshold`

The specification supplies no probability at which a rating becomes “high-risk.” The guard therefore establishes a non-vacuous target/control separation on matched inputs, and every result boundary requires the target to remain above the control by more than the harness's named generic tolerance; it never substitutes an action threshold or the unrelated `LIFECYCLE_SUFFICIENCY_DRIFT` exact-agreement budget.

**Observation (A competitive set is not coverage)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-coverage-source`

Identity competitive cells are nested, gapped, and explicitly not a cover (´def:keyspace:competitive-set´). The no-window assertion consequently names the stable identity facet and Outcome Ledger ancestor routing as the live evidence during the regional gap; it does not require an old or new regional indicator to be active at every instant.

**Observation (Independent publications need assessment-bound evidence)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-publication-boundaries`

Identity observations are deferred, model lifecycle publication follows maintenance, and report reception is synchronous. A barrier establishes completion of prior work but does not freeze future identity maintenance, and a playback batch is not a shared view; the witness therefore records each assessment's pending active chains and snapshot version, uses named barriers at deliberate state boundaries, and treats the relocation projection's independently owned readings as separate sources rather than one fictitious atomic snapshot (´dec:harness:no-ad-hoc-waits´) (´dec:harness:probe-contract´).

**Observation (Dissolution and deletion answer different questions)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-decay-versus-deletion`

Elapsed-time decay changes the value returned for a retained stale entry without mutating storage (´def:ledger:time-decay´), report absence deletes an entry at its count threshold, and collection may delete an old immaterial entry on a separate horizon (´alg:ledger:entry-deletion´) (´alg:ledger:garbage-collection´). The schedule stops reports after the second absence, proves the Ledger value stays above the collection floor through its decay readings, and sends the third absence only afterwards, making the three mechanisms independently visible.

## Acceptance · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-acceptance`

The implementation report identifies integration target `identity_layer` and function `an_entity_that_relocates_is_tracked_across_the_move`, confirms that its module index and documentation cite the intent claim and mint one unique integration-test label, and shows the generated coverage projection resolving the claim to that witness.

The report identifies the assessment-bound active-cell extension and exact relocation projection, confirms their test-support gating, owned return values, named barriers, single loads, scenario-time decay, and explicit separation of independently published sources.

The evidence names the stable and regional cell chains at every boundary, at least one old-region exit and destination entry, the report and Ledger route keys during ancestor fallback, destination creation before visibility, the target/control separation at every observed handover state, destination identity and Ledger rates above their non-vacuity floors, exact decay-oracle comparisons, abandoned-entry deletion at the third absence, final fallback, and clean health.

The report shows the focused `identity_layer` witness and the existing entry, exit, routing, barrier, playback, fixture, and decay tests passing under the package's prescribed formatting, linting, test, and corpus-name gates, with unrelated failures separated explicitly.

The report states which assertion rejects loss of the stable facet, publication before destination creation, leaf-only updates, missing destination learning, wrong-clock decay, premature retirement, and absent retirement. No executed mutation is required.
