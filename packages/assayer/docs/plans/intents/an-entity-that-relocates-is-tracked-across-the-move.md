# Tracking a Relocating Entity Without a Rating Gap · `plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move`

This plan keeps the promise (´claim:identity:an-entity-that-relocates-is-tracked-across-the-move´) by establishing that one semantic entity retains an adverse rating while its regional identity cells, Sentinel routing, and spatial outcome memory hand responsibility from an abandoned region to a destination.

## What the promise says, precisely · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-promise`

The executable entity key is composite: a stable facet $s$ names the entity and a regional facet $r$ names its current location. One identity dimension encodes $s$ and another encodes $r$, matching the public multi-encoder use of one opaque key while respecting that the identity layer itself partitions host-declared ranges rather than maintaining primary per-entity state (´mot:keyspace:purpose´).

Let $A$ and $B$ be disjoint regional prefixes beneath a shared ancestor, let $C_s(t)$ and $C_r(t)$ be the active stable and regional identity-cell chains, and let $K_L(x,t)$ and $b_L(x,t)$ be the deepest Outcome Ledger key and its decayed adverse rate at coordinate $x$. The assessment protocol computes both identity coordinates, queues the observation, and reads the currently published competitive set (´alg:keyspace:observation-protocol´).

Before relocation the fixture establishes $C_s(t_0)\neq\varnothing$, $C_r(t_0)$ containing a cell in $A$, $K_L(A,t_0)$ at the reported $A$ leaf, and a target probability $p_T(t_0)$ above a matched benign control $p_C(t_0)$ by a scenario margin $\delta_p=10^{-4}$. The margin is ten times the harness's `LIFECYCLE_SUFFICIENCY_DRIFT` budget, so the word “high-risk” denotes an established public separation rather than an unspecified absolute policy threshold.

The move changes only $r$ and the request's Sentinel coordinate. During the handover, $C_s$ retains at least one cell covering the stable facet; the old regional cells leave and destination cells enter through the competitive lifecycle (´alg:keyspace:cell-entry´) and (´alg:keyspace:cell-exit´); and every assessed handover state satisfies $p_T-p_C>\delta_p$.

The Sentinel report sequence is $R_A=\{\text{root},P,A\}$, $R_P=\{\text{root},P\}$, and $R_B=\{\text{root},P,B\}$. At $R_P$, both the current report and the Ledger resolve the moved coordinate through a surviving ancestor; at $R_B$, the destination entry exists before the report becomes visible, as required by the report-reception publication order (´req:publication:report-reception´).

Every adverse label updates the deepest present Ledger entry and all of its ancestors (´alg:ledger:all-layers-update´), so the witness requires $b_L(B)$ and the destination regional identity outcome rate to rise from their neutral creation values while the shared ancestor remains informed. Ledger reads always find at least the permanent root (´def:ledger:root-semantics´), and their depth walk supplies the finest surviving memory rather than a fabricated neutral result (´dec:memory:depth-walk´).

The fixture keeps the abandoned Ledger entry below the configured absence-deletion threshold while it takes decay readings, then retires it with further destination reports. With the shipped rates $\gamma_{t,\mathrm{id}}=0.998$ and $\gamma_{t,L}=0.999$ per hour, the retained regional identity outcome and Ledger adverse rate follow their respective closed forms; the Ledger reading is approximately halved at twenty-nine days and is approximately one eighth of its peak at eighty-seven days (´tab:config:temporal´).

The configured $N_{\mathrm{absent}}=3$ makes report-count retirement distinct from elapsed-time dissolution (´tab:config:ledger´). The measurable composition is therefore not instantaneous fine-cell replacement: the specification permits a bounded structural detection gap, but requires degraded-resolution ancestor coverage throughout it (´scenario:monitoring:displacement´).

## What the code offers today · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-code`

The public path is `Assayer::register_identity_dimension`, `Assayer::receive_sentinel_report`, `Assayer::assess`, `Assayer::submit_label`, and `Assayer::full_health_report`. `IdentityDimensionRegistration` accepts a function from `EntityKey` to a coordinate, so two encoders can read the stable and regional fields of one composite key without production mutation.

The world harness supplies `scenario_with_config`, `World::assayer`, `World::clock`, `World::request_with_sentinel`, `World::assess`, `World::label`, `World::flush_labels`, and named entities, while its report fixtures supply `make_cell_report` for the three report shapes. The completed skeleton owns these deterministic verbs and the liveness-aware lifecycle barriers (´entry:assayer:harness-stage-skeleton´).

The identity maintenance loop drains observations, computes set differences, emits exits before entries in one lifecycle submission, and publishes the per-dimension competitive index. Its swap discipline is fixed by the memory record (´dec:memory:competitive-publication´), but the production loop still wakes on its own cadence and its checkpoint command does not apply due decay.

The Ledger's cell-set machinery already creates exact report keys and retains absent cells to the configured threshold (´alg:runtime:cell-set-maintenance´). Its routing and entry machinery sends reads to the deepest hit, updates every containing key, and computes read-time decay (´dec:memory:depth-walk´) and (´def:ledger:time-decay´). Report acknowledgements expose created and deleted counts, and full health exposes depth histograms and aggregate rates, but neither surface identifies the regional key behind a count or exposes one cell's decayed value.

The nearest integration witness registers multiple encoders and waits for competitive geometry before reading it (´test:integration:the-association-reading-separates-a-keyed-dimension-from-a-null-one´). The nearest Ledger integration witness proves only that the root survives sustained public traffic (´test:integration:root-survives-many-assessments-within-lifetime´).

Focused crate witnesses establish immediate report-cell appearance (´test:crate:cell-set-appear-then-read´), the twenty-nine-day Ledger timeline (´test:crate:ledger-decay-twenty-nine-day-half-life-timeline´), and identity exit/re-entry state semantics (´test:unit:competitive-exit-resets-measurement-and-reentry-retains-outcome´). None holds one public risk trajectory across all three mechanisms.

The standing testing plan explicitly assigns this promise to the partly completed persistent-clock stage (´entry:assayer:harness-stage-clock´); no separate standing gap entry covers it.

## The witness · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-witness`

The witness belongs in the integration identity-layer module beside the public lifecycle scenarios (´test:integration:identity-churn-preserves-finite-assess-and-derive´), and its module index describes the composed handover rather than any one mechanism below it.

- Setup: build a seeded world with one outcome axis, one Sentinel, stable and regional identity dimensions at depth cutoff three, `IdentityBudget::for_depth_cutoff(3)`, and three reports whose leaf cells $A$ and $B$ are disjoint beneath one reported ancestor $P$.

- Initial population: alternate benign controls across several stable facets with adverse target requests carrying composite key $(s_T,A)$ and Sentinel coordinate $A$; settle the cold ramp and each identity-maintenance round until probes show a stable target cell, an $A$ regional cell, and an $A$ Ledger leaf.

- Initial assertion: require non-zero adverse state in the target's stable cell, the $A$ regional cell, and the $A$ Ledger leaf; require $p_T(A)-p_C(A)>\delta_p$; and record the exact active-cell chains, route keys, snapshot versions, and peak rates.

- Handover stimulus: publish $R_P$, assess composite key $(s_T,B)$ at Sentinel coordinate $B$ before draining its deferred identity observation, and then take a synchronized probe. This is the detection-gap reading: the stable chain remains active, the report and Ledger route through $P$, and the target remains separated from a control assessed against the same held publication.

- Regional rebalance: advance the virtual clock by the declared interval, drive bounded bursts at $B$ with no further $A$ traffic, and synchronize after every burst until the regional probe records at least one $A$ exit and one $B$ entry. Each post-publication target/control pair is assessed from one held model version and retains the same probability margin.

- Destination stimulus: publish $R_B$, require its `ReportAck` to report the destination Ledger creation with no premature deletion of $A$, then apply adverse labels at $B$ until both destination per-cell rates exceed ten times `World::tol().ewma` and the target remains above its matched control.

- Decay observation: with no further $A$ labels, read the retained $A$ identity outcome and Ledger states at zero, twenty-nine, fifty-eight, and eighty-seven days. Every present value agrees with $q_0\gamma^{\Delta h}$ within `World::tol().ewma`; eviction of the identity graph interval is accepted only as the terminal absence of that stale state, never as a neutral value returned while it remains.

- Retirement observation: submit enough further $R_B$ reports to cross $N_{\mathrm{absent}}$, require the acknowledgement to name the $A$ Ledger deletion, require $K_L(A)$ to fall back to $P$ or root, and require a final moved-target assessment to retain the established probability margin with clean health.

- Assertion: the schedule records every deliberate boundary in order and requires at least one live rating source at each one: stable identity evidence throughout, ancestor Ledger evidence in the report gap, destination identity and Ledger evidence after takeover, and no stale $A$ value after its specified decay or retirement.

- Fails-before: an implementation that derives all identity coordinates from the mutable region loses the stable active cell and lets the first $B$ assessment collapse to the control; report publication before Ledger creation yields a destination report whose exact key has no entry; leaf-only label routing leaves $P$ neutral in the gap; missing destination outcome updates leave $B$ at zero; and wall-clock or label-indexed stale-state decay misses the closed-form schedule.

## What is missing · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-missing`

**Entry (Clocked identity-maintenance barrier)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-identity-barrier`

Approximately seventy to one hundred lines on the test-support and test-gated maintenance command surfaces add `World::advance` and a bounded `World::settle_identity` operation. The operation drains observations, applies every decay interval due on the injected monotonic clock, detects and publishes competitive changes, waits for their model-owner lifecycle submission, and returns the before/after publication versions without a real sleep. It depends on the remaining persistent-clock work (´entry:assayer:harness-stage-clock´), uses the two domains without conflating them (´dec:clock:two-domains´), uses the harness's `STATE_DEADLINE`, and has no dependency on another intent lane.

**Entry (Exact relocation-state probe)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-state-probe`

Approximately one hundred to one hundred and forty lines on the test-support surface add a synchronous read-only probe carrying the published model version, active `CompetitiveCellId` chains for named dimension coordinates, retained per-cell identity outcome views, the exact Ledger route key, presence of nominated Ledger keys, and their decayed adverse rates at the injected persistent time. The probe exposes no mutable handle, reads the same per-dimension competitive publication and Ledger guards used by assessment, depends on (´entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-identity-barrier´), and has no dependency on another intent lane.

**Entry (Composite relocation fixture and integration witness)** · `entry:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-integration-witness`

Approximately one hundred and twenty to one hundred and seventy lines in the integration identity-layer module add the composite-key encoders, the $R_A/R_P/R_B$ report fixture, the target/control trainer, the ordered observation schedule, independent decay oracles, non-vacuity guards, and the public assertions above. It depends on both preceding entries and reuses the existing report builder, scenario tolerances, `LabelSpec`, and label barrier rather than introducing a general tape.

## Risks and open questions · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-risks`

**Observation (Relocation needs two key facets)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-entity-semantics`

Keeping the same opaque `EntityKey` while changing only a Sentinel coordinate cannot make an identity competitive cell leave or enter, while replacing the whole key gives the Core no basis for calling the two requests one entity. The composite key preserves a stable facet and changes a regional facet, the same multi-field shape the nearest integration witness already drives. If “region” is intended to mean only a Sentinel coordinate, the identity entry/exit clause of the promise requires a narrower statement; this plan proceeds with the only interpretation that exercises every clause.

**Observation (High-risk has no absolute cut)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-risk-threshold`

The specification supplies no probability at which a rating becomes “high-risk.” The witness therefore establishes the rating before the move as a target/control separation larger than a named harness drift budget and carries that same inequality across every boundary; it never substitutes a channel action threshold or a visually large probability.

**Observation (A competitive set is not coverage)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-coverage-source`

Identity competitive cells are nested, gapped, and explicitly not a cover (´def:keyspace:competitive-set´). The no-window assertion consequently names the stable identity facet and Outcome Ledger ancestor routing as the live sources during the regional gap; it does not require an old or new regional indicator to be active at every instant.

**Observation (Two asynchronous publications must be delimited)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-publication-boundaries`

Identity observations are deferred and model lifecycle publication follows the maintenance loop, while report reception is synchronous. A sleep or a fixed label count cannot identify which combination an assessment read. The barrier and versioned probe delimit each observable state, and target/control comparisons derive from one held publication so scheduler order cannot manufacture or conceal a gap.

**Observation (Dissolution and deletion answer different questions)** · `obs:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-decay-versus-deletion`

Elapsed-time decay changes the value returned for a retained stale entry without mutating storage (´def:ledger:time-decay´), whereas report absence eventually removes the entry. The schedule keeps $A$ below the absence threshold through the decay milestones and crosses the threshold only afterwards, making a wrong clock and a wrong absence count independently visible.

The remaining semantic question is whether “the new region earns its own rating” requires both regional identity outcome state and Outcome Ledger state. This plan requires both because the promise names the layers as a composition; narrowing it to either one would repeat an already focused mechanism witness.

## Acceptance · `sec:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move-acceptance`

The implementing lane's report identifies the new test beside the existing identity lifecycle witness (´test:integration:identity-churn-preserves-finite-assess-and-derive´), confirms its module index and documentation carry the intent citation and a unique integration-test mint, and shows the intent resolving to that witness in the generated test index.

The report identifies the clocked identity barrier and exact relocation probe, confirms both are test-gated, bounded by the harness liveness deadline, driven only by the injected clock, and read-only apart from the explicit synchronization action.

The reported evidence names the stable and regional cell chains at every boundary, at least one old-region exit and destination entry, the report and Ledger route keys during ancestor fallback, destination creation before visibility, the target/control probability margin at every observed handover state, destination identity and Ledger rates above their non-vacuity floor, the two independent decay schedules, abandoned-entry deletion, final fallback, and clean health.

The report shows the targeted integration test and the existing focused entry, exit, routing, and decay witnesses passing under the package's prescribed formatting, linting, debug, release, feature, documentation, and no-default-feature gates, with unrelated failures separated explicitly.

The report states which public or probe assertion rejects each deliberately broken behavior: loss of the stable facet, publication before destination creation, leaf-only updates, missing destination learning, wrong-clock decay, and premature or absent retirement. No executed mutation is required.
