# Competitive-cell turnover preserves information and width · `plan:assayer:intent-a-competitive-cell-exits-as-another-enters`

Keeping (´claim:identity:a-competitive-cell-exits-as-another-enters´) establishes that a decay-driven transfer of traffic can replace competitive cells through the live lifecycle while preserving learned cross-information, starting every arriving indicator at zero, and keeping every dimension-bearing structure at the width implied by the new set.

## What the promise says, precisely · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-precise-promise`

Let $p_0$ be the pre-turnover model width, $C_0$ the total competitive-cell count, $E$ the number of exiting cells, $A$ the number of entering cells, $T_5$ the number of configured per-cell interaction templates, and $q=1+T_5$ the width owned by one competitive cell. The competitive set and its one-indicator-per-cell representation make $C$ data-dependent while preserving the identity of every position (´def:keyspace:competitive-set´), (´def:keyspace:competitive-indicators´).

The feature-vector formula makes the restructuring arithmetic $C_1=C_0-E+A$ and $p_1=p_0+q(A-E)$ when every other registry count is fixed (´tab:feature:dimension-formula´), while the interaction lifecycle rule supplies the $T_5$ term (´alg:feature:lifecycle-interactions´). For the promised one-for-one changeover, $E=A=1$, so $C_1=C_0$ and $p_1=p_0$ even though the exit-first chain temporarily reduces the width by $q$ before the entry restores it.

Cell importance decays at the dimension's host-declared hourly spatial rate, with the shipped $0.998$ rate giving roughly a fortnight half-life; crossing the competitive cutoff is the event threshold, not an additional fixed numeric constant (´tab:keyspace:decay-rates´). The witness compresses that elapsed process through the maintenance harness's forced attenuation of $0.01$ and requires both event kinds rather than treating the attenuation itself as the result.

For pre-exit precision partitioned into surviving indices $k$ and all exiting cell indices $r$, the required surviving precision is $B' = B_{kk}-B_{kr}(B_{rr}+\delta I)^{-1}B_{rk}$, where $\delta=\lambda_{\mathrm{prior}}\varepsilon_{\mathrm{Schur}}$ and the default $\varepsilon_{\mathrm{Schur}}$ is $10^{-4}$ (´alg:gaussian:regularised-schur´). A deliberately planted non-zero $B_{kr}$ makes the correction itself observable as transferred cross-information (´def:gaussian:schur-complement´).

Each arriving cell extends the operational model, sister model, and every outcome-axis model by $q$ independent prior coordinates while leaving the anchor fixed; its indicator has mean weight zero, zero precision and covariance cross-blocks, prior diagonal, binary-standardisation mean one half, and variance one quarter (´alg:keyspace:cell-entry´), (´thm:gaussian:extension´). Each exit removes the same semantic coordinates from those models and from standardisation before the map is rebuilt (´alg:keyspace:cell-exit´).

Every published chain must satisfy $p=\lvert\mu\rvert=\operatorname{rows}(B)=\operatorname{rows}(\Sigma)$ for every full-dimensional model and $p=\lvert\text{means}\rvert=\lvert\text{variances}\rvert=\lvert\text{classes}\rvert$ for standardisation, with the map resolving exactly the same semantics (´inv:dimension:version-consistency´). Matrix comparisons use the harness's one-shot tolerance $\tau=10^{-9}$, and the planted correction must have Frobenius norm greater than $100\tau$ so deletion without transfer cannot pass inside rounding slack (´tab:assayer:harness-scenario-tolerances´).

## What the code offers today · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-code-today`

The public path is `Assayer::register_identity_dimension` followed by entity-bearing `Assayer::assess` calls: assessment encodes the entity, offers its coordinate through `IdentityDimensionInfra::observe`, and reads the most recent competitive set; the maintenance loop drains those coordinates, computes set differences, and sends one `LifecycleSubmission` toward the model owner (´dec:memory:observation-surface´).

The `World` harness wraps registration, assessment, labels, virtual time, seeded scenarios, drift budgets, liveness deadlines, and model-owner publication barriers. It can drive the production traffic path, but its barrier does not delimit an identity-maintenance pass and its public observations do not expose cell-to-index resolution or the live precision blocks required by this promise's algebraic oracle.

`MaintenanceHarness` supplies aggressive split configuration, direct observation infrastructure, acknowledged `checkpoint`, forced decay, and intact `ModelOwnerCommand` drainage. The checkpoint drains observations, detects set changes, and acknowledges only after the resulting lifecycle submission has been enqueued, so the planned witness needs no wall-clock sleep.

The lifecycle crate-test module supplies `LifecycleEnv`, which drives `handle_lifecycle_submission` against a real `WorkingCopy`, published `SharedState`, trackers, ledger, and configuration. The handler gathers exits into one removal chain, marginalises every full-dimensional model, rebuilds through the sole semantic resolver, then applies each entry as its own extension chain and publishes the result (´dec:vector:sole-resolver´), (´dec:construction:compound-batch´), (´test:crate:lifecycle-publishes-snapshot´).

The maintenance test already proves that heavy decay plus traffic in a distant region emits exits before entries in one submission (´test:crate:batch-entry-exit´). The lifecycle tests separately prove entry growth (´test:crate:cell-entry-extends-model´), exit shrinkage (´test:crate:cell-exit-marginalises-model´), and the net width and two-chain publication count for two exits plus one entry (´test:crate:compound-batch-2-exits-1-entry´); the model test isolates non-zero Schur transfer (´test:crate:information-transfer´). None carries the real maintenance submission into a correlated lifecycle model and checks all three effects together.

The standing testing plan has no entry covering the base one-indicator turnover promise; its directly interlocking entry is the separate interaction-churn gap, which covers per-cell interaction features when several cells change in one batch (´entry:assayer:gap-interaction-churn´).

## The witness · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-witness`

**Decision (The setup joins the two existing production seams)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-setup`

The new crate test extends the lifecycle crate-test module. It creates a `MaintenanceHarness` and a `LifecycleEnv` for the same dimension identifier, registers one outcome axis in the lifecycle environment so an axis model participates (´test:crate:axis-model-extended-on-subsequent-cell-entry´), and leaves competitive interaction templates empty so $q=1$ and the standing interaction gap remains separate.

A coordinate burst comfortably above the aggressive split threshold of three is offered through `IdentityDimensionInfra::observe`, an acknowledged maintenance checkpoint yields the initial entry submissions, and the bridge applies those submissions intact to the lifecycle environment. The test records the semantic cell map and width only after both sides agree on the initial set.

The fixture then installs the same finite symmetric positive-definite covariance and a distinguishable mean in the operational, sister, and axis models. Every standing cell indicator is coupled to the stable bias coordinate, each cell mean is non-zero, and the independently reconstructed precision confirms a correction norm above $100\tau$ for any non-empty exiting subset.

**Decision (A decay-and-volume shift is the stimulus)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-stimulus`

The test forces attenuation $0.01$, offers a second burst at a distant coordinate, and invokes `MaintenanceHarness::checkpoint`; it retains submission boundaries when draining the model-owner commands. The selected submission must contain at least one exit and one entry, all exits must precede the first entry, and the before/after cell-set difference must equal the event identities.

The bridge submits that unmodified event vector once to `LifecycleEnv::submit_reporting_version`, so the production chain gathering, joint removal, extension, map rebuild, standardisation update, and publication logic all participate. A test-only publication tap records every chain snapshot, and the returned last version must account for the exit chain and every entry chain described by the compound-batch rule.

**Decision (Semantic indices and an independent Schur calculation form the oracle)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-observation`

Before applying the turnover, the oracle resolves every outgoing cell through the pre-event map, forms $k$ as the ascending complement, and computes the regularised Schur result from copied pre-event blocks without calling the production marginalisation helper. Afterward it resolves surviving and arriving cells through the new map rather than carrying positional assumptions across compaction.

For each full-dimensional model in the fixture, the assertion compares the retained precision with the independent Schur result within $\tau$, confirms the correction norm exceeds $100\tau$, and rejects a fallback diagnostic. It then checks every arriving indicator's mean is zero, its precision and covariance cross-blocks are zero within $\tau$, and its diagonal and standardisation moments equal the configured default prior and binary priors.

The structural assertion checks that the exit-chain snapshot has $C=C_0-E$ and $p=p_0-E$, that the final snapshot has $C_1=C_0-E+A$ and $p_1=p_0-E+A$, and that every snapshot agrees across the operational, sister, axis-model, map, and standardisation widths while the anchor width stays fixed. Every exited cell is absent after the exit chain, every entering cell first appears with its entry chain, and a balanced turnover records the promised equality $p_1=p_0$.

**Decision (Each plausible shortcut has its own red assertion)** · `dec:assayer:intent-a-competitive-cell-exits-as-another-enters-fails-before`

A broken implementation that slices out rows and columns leaves $B_{kk}$ and fails the non-vacuous Schur comparison; one that transfers the departing coefficient into a newly allocated slot gives an arriving indicator non-zero mean; one that omits an extension, compaction, model, or standardisation update fails the additive width equalities. Reversing entry and exit order fails the event-order and publication-count assertions before any final-width coincidence can hide it.

## What is missing · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-missing`

**Entry (An intact maintenance-to-lifecycle bridge)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-turnover-bridge`

Add approximately 35–50 lines to the lifecycle crate-test module for a test fixture that drains only lifecycle commands, preserves each `LifecycleSubmission` boundary and event order, applies it through `LifecycleEnv`, advances publication versions, and returns the before/after semantic cell sets. It depends on the completed harness skeleton (´entry:assayer:harness-stage-skeleton´) and has no dependency on a sibling intent lane.

**Entry (A chain-publication recorder)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-chain-recorder`

Add approximately 30–45 lines around the lifecycle fixture's publication sink for a test-only record of every snapshot stored while one submission is applied, preserving version order without polling or sleeps. It depends on the turnover bridge, remains unavailable to production callers, and makes intermediate exit width directly observable.

**Entry (A correlated full-model turnover state and independent oracle)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-correlated-oracle`

Add approximately 70–100 lines to the lifecycle crate-test module for a positive-definite covariance fixture shared by the operational, sister, and one axis model, semantic extraction of kept and removed indices, an independent dense regularised-Schur calculation, correction and matrix norms, and checks of new-coordinate prior structure. It depends on the turnover bridge, chain recorder, and existing named tolerance bundle, with no sibling-lane dependency.

**Entry (The composed competitive-turnover test)** · `entry:assayer:intent-a-competitive-cell-exits-as-another-enters-composed-witness`

Add approximately 80–110 lines to the lifecycle crate-test module plus its module-index row for initial traffic, correlated-state installation, forced decay, distant traffic, intact submission application, and the algebraic and structural assertions above. It depends on the bridge, chain-recorder, and oracle entries and reuses `MaintenanceHarness`, `LifecycleEnv`, aggressive split configuration, checkpoint acknowledgements, and liveness deadlines.

## Risks and open questions · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-risks`

**Observation (The event boundary is lower than the public scenario surface)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-observation-boundary`

The exact invariant is observable only where semantic indices and precision coexist, so the witness is a crate test even though traffic enters through the same observation surface as public assessment. A later read-only `World` posterior probe and identity-maintenance barrier could lift the composition to the public identity-layer integration suite beside its existing assessment witness (´test:integration:identity-register-allows-assess-and-derive´); making those broad facilities prerequisites would add more machinery than this promise needs.

**Observation (The forced decay compresses time but does not redefine the rate)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-decay-seam`

Forced attenuation deterministically reaches the set-change boundary and does not test the host rate's elapsed-time composition. The existing decay-rate tests own that arithmetic; this witness accepts only the emitted exit-and-entry submission as evidence that the chosen stimulus reached turnover.

**Observation (Regularisation and fallback qualify transfer)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-regularisation`

The operational algorithm transfers the regularised correction and may deliberately fall back to $B_{kk}$ on an unusable removed block. The planted spectrum stays far inside both condition ceilings, the test rejects fallback diagnostics, and the oracle compares the declared regularised result rather than claiming the exact unregularised theorem.

**Observation (Empty interaction templates define this promise's boundary)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-interaction-boundary`

The base witness proves $q=1$ and states the general $q=1+T_5$ arithmetic without claiming that interaction resolution moved correctly. Exercising non-zero $T_5$, several simultaneous cells, and template recompilation belongs to the standing interaction-churn entry, not to a hidden second promise here.

**Observation (Lifecycle delivery has an uncovered loss mode)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-delivery-risk`

The harness channel is deliberately wide, so this witness cannot cover a full model-owner queue. Production currently advances and publishes the maintenance side even when its non-blocking lifecycle send fails, a distinct structural risk already recorded by (´entry:assayer:wl-identity-lifecycle-drop´); this test neither closes nor masks that entry.

**Observation (Semantic lookup absorbs unstable within-group order)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-event-order-risk`

Exit-before-entry ordering is stable, but iteration within each set-difference group is not yet deterministic (´entry:assayer:wl-identity-nondeterministic-event-order´). The oracle keys cells semantically and asserts only the required group order, so process-specific ranks cannot make the test flaky or accidentally bless rank as identity.

**Observation (Correctness does not discharge the frequent-path cost)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-operating-cost`

The composed witness proves live structural adaptation without rebuilding the Assayer, but it does not prove the quadratic low-rank cost stated for frequent competitive exits. The currently unbuilt low-rank route remains tracked independently by (´entry:assayer:wl-low-rank-exit-unbuilt´).

**Observation (The entry prior matches configuration only at the default)** · `obs:assayer:intent-a-competitive-cell-exits-as-another-enters-prior-source`

The competitive-entry handler currently passes a literal $0.1$ prior precision while other lifecycle extensions read `config.model.lambda_prior`. This promise requires a zero arriving weight, so the witness uses the matching default and asserts that zero plus its independent cross-block; a non-default-prior conformance test either requires the handler to use configuration or deliberately establishes that competitive cells alone have a fixed prior, which is a separate decision.

## Acceptance · `sec:assayer:intent-a-competitive-cell-exits-as-another-enters-acceptance`

The implementation report names the new crate-test mint and path, the two traffic coordinates, split threshold, forced attenuation, seed, initial and final cell counts, exit and entry counts, pre- and post-widths, chain count, and publication versions.

It reports the removed indices by semantic cell identity, the correction norm, maximum error from the independent regularised Schur oracle for every full-dimensional model, fallback diagnostics, and the arriving indicators' means, cross-block norms, prior diagonals, and standardisation moments against $\tau$.

It shows the cell-set and additive-width equations, every model/map/statistics width agreement, and the fixed anchor width; it also records that the $B_{kk}$ shortcut, inherited entering weight, and one-sided width-update failures are separated from the passing fixture by explicit assertions.

It shows repeated focused runs without sleeps, all named Assayer package gates and the corpus linter green, no new public production surface, no interaction-churn claim, and no closure claimed for the lifecycle-delivery or low-rank-cost entries.
