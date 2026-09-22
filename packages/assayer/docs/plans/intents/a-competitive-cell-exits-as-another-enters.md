# Competitive-cell turnover preserves information and width · `plan:assayer:intent-a-competitive-cell-exits-as-another-enters`

Keeping (´claim:identity:a-competitive-cell-exits-as-another-enters´) establishes that a decay-driven transfer of traffic can replace one competitive cell through the live maintenance and lifecycle path while transferring the departing cell's cross-information, starting the arriving indicator at zero, and keeping every dimension-bearing structure at the width implied by the new competitive set.

## What the promise says, precisely · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-precise-promise`

Let $p_0$ be the pre-turnover model width, $C_0$ the total competitive-cell count, $E$ the number of exiting cells, $A$ the number of entering cells, $T_5$ the number of competitive interaction templates declared for the affected dimension, and $q=1+T_5$ the indicator and interaction width owned by one competitive cell. The competitive set is data-dependent, and each of its cells owns one indicator position (´def:keyspace:competitive-set´) (´def:keyspace:competitive-indicators´).

With every other registry count fixed, the feature-vector formula and interaction lifecycle rule give $C_1=C_0-E+A$, an exit-chain width $p_{\mathrm{exit}}=p_0-qE$, and a final width $p_1=p_0+q(A-E)$ (´tab:feature:dimension-formula´) (´alg:feature:lifecycle-interactions´). The promised one-for-one changeover requires $E=A=1$, so $C_1=C_0$, $p_{\mathrm{exit}}=p_0-q$, and $p_1=p_0$; equal final widths do not erase the narrower exit publication between them.

Cell importance decays at the dimension's host-declared hourly spatial rate. The specification describes the default spatial rate as halving importance in about a fortnight, and the implementation separately proves that its per-interval attenuation composes to that hourly rate and that a supplied rate replaces the default (´tab:keyspace:decay-rates´) (´test:unit:decay-cadence-composes-to-hourly-rate´) (´test:unit:dimension-state-uses-registered-spatial-decay-rate´). The witness uses the maintenance harness's forced attenuation of $0.01$ only as a deterministic stimulus and accepts the resulting exit and entry events, not the attenuation value itself, as evidence of turnover.

For pre-exit precision partitioned into surviving indices $k$ and the exiting cell's indices $r$, the required surviving precision is $B' = B_{kk}-B_{kr}(B_{rr}+\delta I)^{-1}B_{rk}$, where $\delta=\lambda_{\mathrm{prior}}\varepsilon_{\mathrm{Schur}}$ and the default $\varepsilon_{\mathrm{Schur}}$ is $10^{-4}$ (´alg:gaussian:regularised-schur´). A deliberately planted non-zero $B_{kr}$ makes the correction observable as transferred cross-information rather than a vacuous equality with $B_{kk}$ (´def:gaussian:schur-complement´).

Each arriving cell extends the operational model, sister model, and every outcome-axis model by $q$ independent prior coordinates while leaving the anchor fixed; the indicator's mean is zero, its precision and covariance cross-blocks are zero, its diagonal is the configured prior, and its standardisation mean and variance are one half and one quarter (´alg:keyspace:cell-entry´) (´thm:gaussian:extension´). Each exit jointly removes that cell's indicator and competitive-interaction coordinates from the full-dimensional models and standardisation before the semantic map is rebuilt (´alg:keyspace:cell-exit´).

Every exit and entry publication must satisfy $p=\lvert\mu\rvert=\operatorname{rows}(B)=\operatorname{rows}(\Sigma)$ for the operational, sister, and outcome-axis models and $p=\lvert\text{means}\rvert=\lvert\text{variances}\rvert=\lvert\text{classes}\rvert$ for standardisation, with the map resolving those same positions (´inv:dimension:version-consistency´). Elementwise matrix comparisons use the harness's one-shot tolerance $\tau=10^{-9}$, and the fixture requires the correction's maximum absolute entry to exceed $100\tau$, a fixed separation derived from that declared tolerance rather than fitted to an observed result (´tab:assayer:harness-scenario-tolerances´).

## What the code offers today · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-code-today`

The production path remains `Assayer::register_identity_dimension` followed by entity-bearing `Assayer::assess` calls: assessment encodes the entity, offers its coordinate through `IdentityDimensionInfra::observe`, and reads the last published competitive set, while identity maintenance detects set differences and offers a `LifecycleSubmission` to the model owner (´dec:memory:observation-surface´).

The unified `World` now owns the real Assayer, moves virtual time forward through `World::advance` and `World::travel_to`, and exposes `World::flush_identity_maintenance`, which crosses maintenance and accepted model-owner publication before returning. Its `RuntimeLayout` reports published block widths, and `PublishedModelBlock` returns an owned mean and covariance from one publication; the probe contract deliberately excludes working precision and the semantic dimension map, so this surface cannot establish the exact Schur transfer or identify the entering and exiting coordinates (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:probe-contract´).

The specialised `MaintenanceHarness` now has guarded construction, forced decay, `MaintenanceHarness::flush_identity_maintenance`, and `MaintenanceHarness::drain_model_owner`. Its barrier drains prior observations, detects set changes, offers lifecycle submissions, and publishes the graph before acknowledging; it deliberately does not apply submissions because this side harness has no model owner (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:guarded-fixtures´) (´dec:harness:specialised-side-harnesses´). The former checkpoint vocabulary is gone, and `collect_lifecycle_events` is unsuitable for this witness because it flattens submission boundaries.

The lifecycle crate-test module already supplies `LifecycleEnv` and `LifecycleEnv::submit_reporting_version`, which drive `handle_lifecycle_submission` against a real `WorkingCopy`, `SharedState`, trackers, ledger, and configuration. The production handler groups ordered events into maximal order-free chains, gathers all exits in a chain into one removal, rebuilds through the semantic resolver, applies each entry as its own extension chain, and publishes once per chain (´dec:vector:sole-resolver´) (´dec:construction:compound-batch´) (´test:crate:lifecycle-publishes-snapshot´).

The maintenance test proves that forced decay plus traffic in a distant region emits exits before entries in one submission (´test:crate:batch-entry-exit´). Lifecycle tests separately prove entry growth (´test:crate:cell-entry-extends-model´), exit shrinkage (´test:crate:cell-exit-marginalises-model´), axis-model extension (´test:crate:axis-model-extended-on-subsequent-cell-entry´), and the net width and two-chain publication count for two exits plus one entry (´test:crate:compound-batch-2-exits-1-entry´); the model test isolates non-zero Schur transfer (´test:crate:information-transfer´). None carries a maintenance-produced one-exit, one-entry submission into correlated operational, sister, and axis models and checks transfer, zero-start, and both published widths together.

The shared oracle tier has landed: `regularised_schur_complement` evaluates the specification formula by pivoted dense elimination, `DimensionBlocks` with `dimension_width` evaluates the feature-width formula, and `DEFAULT_TOLERANCES` supplies the named comparison budget (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:oracle-tier´). Focused tests establish the regularised scalar route and distinguish it from an unregularised inverse (´test:crate:regularised-schur-oracle-matches-scalar-formula´) (´test:crate:regularised-schur-oracle-separates-unregularised-inverse´).

The base promise remains unkept: no test mints or cites (´claim:identity:a-competitive-cell-exits-as-another-enters´). The testing plan's neighbouring open entry concerns competitive interaction features under multi-cell churn and therefore does not replace this indicator-only composition witness (´entry:assayer:gap-interaction-churn´).

## The witness · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-witness`

**Decision (The setup joins the two existing production seams)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-setup`

The new crate test extends `tests::lifecycle`. It creates a guarded `MaintenanceHarness` dimension and a `LifecycleEnv` with the same dimension identifier, registers one non-spatial outcome axis so an axis model participates, and declares no competitive interaction templates so $T_5=0$ and $q=1$. It applies the initial maintenance-produced submissions intact, then records the pre-turnover semantic position keys and verifies the initial width with `dimension_width` (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´) (´dec:harness:oracle-tier´).

After the initial cell set is stable, the setup installs the same finite symmetric positive-definite correlated state in the operational, sister, and axis models. Each standing cell indicator has non-zero precision coupling to the stable bias coordinate, the model means are distinguishable, and a read-back guard reports the measured dimensions, spectra, cell identities, and correction floor before the stimulus is admitted; those setup checks remain separate from the result oracle (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

**Decision (A decay-and-volume shift is the stimulus)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-stimulus`

The test applies `MaintenanceHarness::force_decay` with attenuation $0.01$, offers a finite burst at a distant coordinate through `IdentityDimensionInfra::observe`, and crosses `MaintenanceHarness::flush_identity_maintenance`. It then drains `ModelOwnerCommand`s once and requires exactly one `LifecycleSubmission` containing exactly one `LifecycleEvent::CompetitiveCellExit` followed by exactly one `LifecycleEvent::CompetitiveCellEntry`; the before/after competitive-set difference must name those same cells (´test:crate:batch-entry-exit´) (´dec:harness:specialised-side-harnesses´).

The bridge passes that submission's ordered event vector once to `LifecycleEnv::submit_reporting_version`. The production chain gathering, joint removal, extension, semantic rebuild, standardisation update, and publication logic therefore run unchanged, while the test-only observer copies the exit-chain and entry-chain working and published states in version order (´dec:construction:compound-batch´).

**Decision (Semantic indices and the shared Schur oracle form the result oracle)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-observation`

Before applying turnover, the oracle resolves the outgoing cell through the pre-event semantic position keys, forms $k$ as their ascending complement, copies each full-dimensional model's precision, and calls `regularised_schur_complement` with the deployment's $\delta$. Afterward it resolves every retained and arriving position through the new semantic keys rather than carrying a compacted numeric index across the rebuild (´dec:harness:oracle-tier´) (´dec:vector:sole-resolver´).

For the operational, sister, and axis models, the result oracle compares every retained precision entry with the shared Schur result within `DEFAULT_TOLERANCES.default`, confirms that the planted correction clears $100\tau$, and rejects `EventOutcome::SchurFallback`. It verifies that the arriving indicator has zero mean, zero covariance and precision cross-blocks, the configured prior on both diagonals, and the binary standardisation moments required by competitive entry (´alg:keyspace:cell-entry´) (´alg:gaussian:regularised-schur´).

The structural oracle calls `dimension_width` for the pre-event, exit-chain, and entry-chain populations. It requires $C_{\mathrm{exit}}=C_0-1$, $p_{\mathrm{exit}}=p_0-1$, $C_1=C_0$, and $p_1=p_0$; each recorded publication must agree across its dimension map, operational, sister, axis-model, and standardisation widths, while the anchor remains bit-identical. The outgoing cell is absent after the exit chain, and the incoming cell appears only in the entry chain.

**Decision (Each plausible shortcut has its own red assertion)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-fails-before`

A bare $B_{kk}$ slice fails the non-vacuous Schur comparison; transferring the departing coefficient into the new slot fails the zero-mean assertion; omitting an extension, compaction, model, or standardisation update fails a named width equality. Entry-before-exit fails the event order and the recorded intermediate width, while collapsing both chains into one publication fails the versioned publication record even though the final width happens to equal the initial width.

## What is missing · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-missing`

**Entry (An intact maintenance-to-lifecycle bridge)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-turnover-bridge`

Add a test-local helper in `tests::lifecycle` that drains `MaintenanceHarness::drain_model_owner`, rejects unexpected command kinds, preserves each `LifecycleSubmission` boundary and event order, and passes the owned event vector with an explicit first publication version to `LifecycleEnv::submit_reporting_version`. `collect_lifecycle_events` cannot supply this shape because it returns borrowed events flattened across submissions.

**Entry (A chain-publication recorder)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-chain-recorder`

Add a test-only observed form beside `handle_lifecycle_submission` that delegates to the same production implementation and invokes a callback with the chain's `WorkingCopy` and newly built snapshot at the publication boundary. `LifecycleEnv` copies each version's semantic map, model means, covariance and precision, standardisation, and anchor into an owned record; the ordinary handler and public production surface remain unchanged, and no polling or elapsed-time inference is introduced.

**Entry (A guarded correlated full-model turnover state)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-correlated-oracle`

Add a guarded fixture local to `tests::lifecycle` that installs one finite symmetric positive-definite correlated state into the operational, sister, and axis models and returns only after reading back their common width, semantic cell positions, usable removed-block spectra, non-zero correction floor, and unchanged standardisation lengths. The result calculation reuses `regularised_schur_complement`, `dimension_width`, and `DEFAULT_TOLERANCES`; no second local implementation of the landed oracle tier is missing.

**Entry (The composed competitive-turnover test)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-composed-witness`

Add `tests::lifecycle::competitive_cell_turnover_preserves_information_and_width` and its module-index row. The test composes the bridge, recorder, guarded correlated state, exact one-exit/one-entry maintenance stimulus, shared Schur and width oracles, prior assertions, and explicit fails-before arms without adding a production API or claiming competitive-interaction coverage.

## Risks and open questions · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-risks`

**Observation (The event boundary is lower than the public scenario surface)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-observation-boundary`

`World::flush_identity_maintenance` now covers accepted lifecycle application and publication, so the barrier is no longer missing. The remaining boundary is retained state: `PublishedModelBlock` intentionally omits precision and `RuntimeLayout` exposes widths rather than semantic cell-to-index resolution, exactly as the probe contract requires (´dec:harness:probe-contract´). The exact invariant therefore remains a crate test; widening the public probe would violate the finished retention boundary merely to relocate this witness.

**Observation (The forced decay compresses time but does not redefine the rate)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-decay-seam`

Forced attenuation deterministically reaches the set-change boundary and does not test elapsed-time composition. The existing unit witnesses own the default and host-supplied hourly-rate arithmetic (´test:unit:decay-cadence-composes-to-hourly-rate´) (´test:unit:dimension-state-uses-registered-spatial-decay-rate´); this witness accepts only the emitted one-exit, one-entry submission as evidence that its stimulus reached turnover.

**Observation (Regularisation and fallback qualify transfer)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-regularisation`

The operational algorithm transfers the regularised correction and deliberately falls back to $B_{kk}$ when either conditioning guard refuses the removed block (´alg:gaussian:regularised-schur´). The guarded setup keeps the removed spectrum inside both ceilings, the test rejects fallback diagnostics, and the oracle uses the deployment's configured $\delta$ rather than claiming the exact unregularised theorem.

**Observation (Empty interaction templates define this promise's boundary)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-interaction-boundary`

The base witness proves $q=1$ and states the general $q=1+T_5$ arithmetic without claiming that competitive interaction resolution moved correctly. Non-zero $T_5$, several simultaneous cells, and template recompilation belong to the standing interaction-churn entry (´entry:assayer:gap-interaction-churn´).

**Observation (Lifecycle delivery has an uncovered loss mode)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-delivery-risk`

The specialised harness uses a deliberately wide model-owner channel, so this witness cannot cover saturation. Production can publish the maintenance-side competitive set after a non-blocking lifecycle send fails, a distinct structural risk recorded by (´entry:assayer:wl-identity-lifecycle-drop´); this test neither closes nor masks that entry.

**Observation (Semantic lookup absorbs unstable within-group order)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-event-order-risk`

Exit-before-entry grouping is stable, while order within either set-difference group remains process-dependent (´entry:assayer:wl-identity-nondeterministic-event-order´). Requiring exactly one event in each group removes that ordering degree of freedom, and semantic position keys ensure that compaction or later multi-cell variants cannot turn process-specific rank into identity.

**Observation (Correctness does not discharge the frequent-path cost)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-operating-cost`

The composed witness proves live structural adaptation without rebuilding the Assayer, but it does not prove the low-rank cost the specification prices for frequent competitive exits. The still-unbuilt low-rank route remains tracked independently by (´entry:assayer:wl-low-rank-exit-unbuilt´).

**Observation (The entry prior matches configuration only at the default)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-prior-source`

The competitive-entry handler still passes a literal $0.1$ prior precision while other lifecycle extensions read `config.model.lambda_prior`. This witness uses the matching default and verifies the zero mean, independent cross-blocks, and prior diagonal promised here; non-default prior conformance requires either routing configuration into competitive entry or an explicit decision that this lifecycle alone owns a fixed prior.

## Acceptance · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-acceptance`

The crate-test target `torrust_assayer` indexes and runs `tests::lifecycle::competitive_cell_turnover_preserves_information_and_width`, whose documentation cites the kept claim. Its report identifies the initial, exiting, and entering cells semantically; shows exactly one exit before exactly one entry in the intact maintenance submission; and records the pre-event, exit-chain, and entry-chain publication versions and cell sets.

For the operational, sister, and axis models, the report gives the removed semantic positions, configured $\delta$, correction maximum, maximum elementwise error from `regularised_schur_complement`, and fallback outcome. It also gives the arriving indicator's mean, covariance and precision cross-block maxima, prior diagonals, and standardisation moments against `DEFAULT_TOLERANCES.default`.

The report evaluates all three widths with `dimension_width`, shows the cell-count and model-width equations, and records agreement among every model, map, and standardisation width at each publication plus a bit-identical anchor. Explicit assertions separately reject $B_{kk}$ slicing, inherited entering weight, one-sided width updates, reversed event order, and a missing intermediate publication.

Focused repeated runs contain no sleeps, polling, local deadlines, or long playback; the package gates and corpus linter are green. The implementation adds no public production surface, makes no interaction-churn claim, and claims no closure for lifecycle delivery, non-deterministic multi-event ordering, non-default competitive priors, or low-rank cost.
