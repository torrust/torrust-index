# A Mis-Standardised Arrival Returns to Scale · `plan:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale`

Keeping (´claim:feature:a-mis-standardised-arrival-bootstraps-back-into-scale´) establishes that a late Sentinel whose reported z-scores are grossly displaced from their class prior acquires an empirical scale over the configured per-Sentinel bootstrap horizon, while the production label path bounds the influence of an observation made before that acquisition completes.

## What the promise says, precisely · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-precise-promise`

The witness begins only after the instance-wide cold ramp is in service and a balanced scalar-signal population has produced directional separation, so “joining a trained deployment” is an observed precondition rather than elapsed time or an assumed label count. The existing population-split witness supplies the public training shape and its separation checks (´test:integration:scalar-signal-population-split-converges-directionally´).

Registration creates a new contiguous Sentinel slot consisting of occupancy followed by the fixed-width extraction (´def:extraction:slot´). The chain z-score group supplies the selected positions, with cell, root, maximum, mean, gradient, and spread views retaining per-axis identity (´tab:extraction:chain-z-scores´); the witness drives the cell and root maxima on all axes through a deterministic two-point population at 5 and 15 and does not describe derived zero gradients as displaced report z-scores.

Before the late Sentinel has supplied a bootstrap sample, every selected z-score position carries mean zero and variance one from the feature-class priors (´tab:standardisation:class-priors´), with the assignment derived from the map and fixed extraction layout (´req:standardisation:class-assignment´). A report value at 5 or 15 therefore enters the first assessment at essentially the same standardised magnitude, which makes the mis-standardisation directly measurable.

The per-Sentinel algorithm opens an accumulator at registration, accepts only assessments in which that Sentinel reports, accumulates one slot vector per such assessment, blends empirical moments into the slot at completion, and releases the accumulator (´alg:standardisation:sentinel-bootstrap´). The configured quantities are $N_\text{boot}=100$ accepted reporting assessments and $\alpha_\text{boot}=0.8$, with the remaining fifth left on the prior (´tab:config:standardisation´).

For an equal population at 5 and 15 the selected raw z-score positions have empirical mean 10 and population variance 25. On an arm carrying no labels during acquisition, the specified blend fixes the post-bootstrap mean at 8; under the implementation's componentwise variance blend the variance is 20.2, and both endpoints then have standardised magnitude below 2 rather than between 5 and 15.

Observation and continuing learning have different authority: the bootstrap moves later standardisation snapshots from assessment-time observations, while the continuing average moves at label time only after the full-vector cold phase is in service (´req:standardisation:timing´). The assessment supplying the first bootstrap observation scores against the snapshot it already acquired, so its deliberately extreme value is still a prior-scale label input (´dec:ordering:score-before-evolve´).

For that label the update computes $h=\hat\phi^\top\Sigma\hat\phi$ and $w_\text{eff}=\min(w_\text{target},w_\text{ceiling},c/(h+\varepsilon))$. The default $c=5$ limits the precision increase along the observation direction to a factor of at most six (´prop:update:leverage-bound´), and the record requires the weight to be reduced before mutation rather than repaired afterwards (´dec:posterior:leverage-before´).

The measurable completion is therefore joint across matched scale and leverage arms: each late slot starts on class priors, each event arrives only after one hundred reporting assessments, the unlabeled arm moves to the exact configured blend, the labeled arm puts both report endpoints below magnitude two, the extreme first label records effective weights satisfying $w_\text{eff}h\leq5$, and every assessment remains finite. The late-arrival caveat remains true during the bounded acquisition window (´cav:limitation:late-standardisation´), while the two-acquisition decision explains why the already trained coordinates do not restart with the new slot (´dec:vector:two-acquisitions´).

## What the code offers today · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-code-today`

The world harness exposes `WorldBuilder`, `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `cycle_request`, `World::flush_labels`, and `World::settle_cold_ramp_with`. These drive the production builder, lifecycle, report-ingestion, assessment, label, and owner-publication paths with a seeded world and bounded barriers; the completed skeleton records this support (´entry:assayer:harness-stage-skeleton´).

The report-fixture support exposes `make_cell_report`, the canonical report components, and the golden coordinate (´const:assayer:golden-route-coordinate´). A small test-local constructor can build coherent root-and-cell reports whose four maximum z-scores alternate between 5 and 15 without adding a second report DSL.

The registration path opens a width-resizing `BootstrapAccumulator` for the late Sentinel. The assessment path prepends occupancy, observes the extracted slot without blocking, and sends `BootstrapComplete`; the model-owner path applies $\alpha_\text{boot}$ to the published slot moments, emits `SentinelBootstrapComplete`, and publishes a later snapshot (´alg:standardisation:sentinel-bootstrap´).

The nearest crate-level lifecycle witness already proves that registration opens the accumulator, a shortened configured sample fills it, the occupancy mean receives the configured blend, and every resulting variance respects the floor (´test:crate:sentinel-bootstrap-blends-into-published-statistics´). The accumulator's independent per-position Welford statistics are separately kept by (´test:unit:bootstrap-multi-feature-variance´).

The closest public standardisation witness walks the instance-wide cold ramp through its reported trajectory and endpoint (´test:integration:cold-ramp-trajectory-stays-inside-the-falsifier´). That test concerns the other acquisition mechanism and exposes why this witness settles the full-vector ramp before registering the late Sentinel.

The leverage primitive is already checked in isolation: target weight, ceiling, and leverage cap each become the minimum in turn (´test:crate:effective-weight-is-the-smallest-of-target-ceiling-and-leverage-bound´). No current integration support records which limit the production label path selected for a particular model update, and neither compact nor full health exposes a late Sentinel's bootstrap count or its slot moments.

## The witness · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-witness`

The integration witness lives in a focused new standardisation-bootstrap module with its own test index. It constructs matched scale and leverage worlds from the default standardisation configuration, fixed seeds, the score-verified signal schema, and the same alternating scalar-signal training shape as the existing directional convergence witness.

Setup settles each instance-wide cold ramp, trains equal benign low-signal and adverse high-signal populations, and requires the established gap and held-out rank threshold before late registration. The two worlds must agree within the purity budget at that boundary, proving that the lifecycle event enters matched deployments with learned posterior structure and an in-service global coordinate system.

The test registers one Sentinel through each `World`, ingests a valid report whose cell and root maximum z-scores are 15 on every axis, and snapshots the selected slot moments before assessment. Exact mean-zero and variance-one readings prove that the new positions start on the z-score class prior while all pre-existing positions remain in service.

The scale arm leaves its first routed assessment unlabeled. The leverage arm applies one eligible adverse label to its matching assessment; a test-only update trace must show for both full risk models that the leverage cap is the selected minimum and that the effective weight equals $c/(h+\varepsilon)$ within the default one-shot tolerance, while every core model trace, anchor included, satisfies $w_\text{eff}h\leq5$.

The remaining reporting assessments in both arms alternate coherent reports at 5 and 15 until each accumulator has seen exactly $N_\text{boot}$ observations. None carries another label; the scale arm therefore isolates observation-authorised acquisition completely, while the leverage arm confines continuing-average movement to the one deliberately traced label.

After observation ninety-nine the harness drains the owner command queue and requires no completion event; after observation one hundred it drains again and requires exactly one `SentinelBootstrapComplete` event for each registered identifier. The scale arm's read-only slot probe reports mean 8 and, under the componentwise implementation, variance 20.2 within the named tolerances; the leverage arm reports finite positive variances and standardised 5 and 15 endpoints with magnitude below 2 despite the one continuing-average update.

The final public observation routes both endpoint reports through both worlds and requires finite in-range risk, non-negative uncertainty, no feature sanitisation, no model fallback, no dropped health event, and the same registered Sentinel identity. These checks make the recovered coordinate usable rather than accepting a numerically correct moment vector on a failed assessment path.

A deliberately broken implementation that never starts or completes the per-Sentinel accumulator leaves the selected means at zero, emits no completion event, and keeps the report endpoints at magnitudes 5 and 15. One that completes early or counts non-reporting assessments fails the ninety-nine/one-hundred boundary, one that ignores $\alpha_\text{boot}$ fails the scale arm's mean and variance readings, and one that removes leverage bounding records a full requested weight with $w_\text{eff}h>5$ on the extreme first label.

## What is missing · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-missing`

No standing testing-plan gap entry names this promise. The open tape, converged-fixture, snapshot, and domain-assertion stage is the general dependency for the narrow additions below (´entry:assayer:harness-stage-tapes´).

**Entry (A trained-world fixture establishes the arrival boundary)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-world-fixture`

Add approximately 45–65 lines to the shared testing support for a deterministic balanced score-verified training fixture that settles the cold ramp, runs the public labelled cycles, and refuses to return until its held-out gap and pairwise rank checks establish the trained precondition. This entry narrows the converged-fixture part of (´entry:assayer:harness-stage-tapes´) and has no dependency on a sibling intent lane.

**Entry (A read-only slot standardisation probe exposes scale)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-slot-probe`

Add approximately 40–60 lines to the shared testing support for a hidden public snapshot keyed by Sentinel name that returns the published snapshot version and owned copies of that slot's feature classes, means, and variances. The probe permits no mutation, resolves through the dimension map, and depends on the snapshot-support portion of (´entry:assayer:harness-stage-tapes´), not on a production health-field change.

**Entry (A scoped update trace identifies the binding limit)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-update-trace`

Add approximately 70–100 lines of test-support instrumentation around the production label-path weight decision, recording model identity, $h$, requested weight, ceiling, safety factor, and effective weight for labels issued by one `World`. The trace is bounded, disabled outside test support, drained through a hidden public harness accessor, and depends on no sibling intent lane.

**Entry (The late-arrival integration witness joins scale and leverage)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-integration-witness`

Add approximately 130–170 lines to the focused standardisation-bootstrap integration module for the matched trained worlds, late registrations, two-point report streams, first-label trace assertion, completion events, slot-moment oracle, final public health checks, module test index, and the test mint that cites the intent. This entry depends on (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-world-fixture´), (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-slot-probe´), and (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-update-trace´).

## Risks and open questions · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-risks`

**Observation (Into scale has a fixture oracle rather than a global threshold)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-scale-oracle`

The specification gives the priors, sample count, and blend weight but no universal absolute definition of “in scale.” One alternative adds a normative bound for every report distribution; the narrower alternative derives the bound from a declared fixture. This witness uses the latter: the scale arm's two-point population makes the post-blend mean exact, and both arms place the supplied endpoints below magnitude two without claiming that two is a package-wide threshold.

**Observation (The bootstrap variance blend lacks an equation)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-variance-equation`

The per-Sentinel algorithm says to blend empirical statistics and the implementation blends mean and variance componentwise, while the cold-ramp algorithm explicitly includes a between-population variance term. One alternative ratifies the current componentwise value of 20.2; the other gives per-Sentinel bootstrap the mixture term. The magnitude-below-two assertion passes under either, while the exact variance assertion remains an implementation-conformance check until the specification selects one formula.

**Observation (Bootstrap completion crosses the command queue)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-completion-ordering`

The last assessment releases its accumulator after enqueueing completion, and the owner publishes the blend later. The witness orders on `World::flush_labels` and the matching completion event after that assessment; it never substitutes a sleep, a request count, or the global observation barrier for completion of the separate per-Sentinel command.

**Observation (One early label also moves the retained standardisation mass)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-label-tracking-overlap`

Because the deployment-wide phase is in service, the continuing average updates after the leverage arm's first label and changes the moments onto which the bootstrap later leaves its fifth of retained mass. A single-arm exact mean of 8 would therefore contradict the production path. The matched unlabeled scale arm owns the exact blend oracle, while the labeled arm owns the leverage oracle and the distribution-derived magnitude bound.

**Observation (Leverage is a directional precision guarantee)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-leverage-oracle`

The specification guarantees a factor on directional precision, not a fixed probability delta. One alternative adds a fixture-specific post-arrival risk threshold or paired scale-first control; the direct alternative records the actual production weight decision and checks $w_\text{eff}h\leq c$. The witness takes the direct alternative because calibration and the nonlinear risk blend cannot turn prior-mass or precision factors into a universal probability difference.

**Observation (Accepted bootstrap samples are reporting assessments)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-accepted-sample-domain`

The configured horizon counts observations of the new Sentinel's slot, not all requests served by the deployment. Every counted request in the witness carries the Sentinel coordinate and a fresh valid report, and control assessments before registration or without the Sentinel are excluded from the boundary assertion.

**Observation (The trained precondition is measured independently)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-precondition`

A fixed training count can become stale as model width or defaults change. The fixture retains a generous deterministic budget but admits the late registration only after public held-out separation passes, so a failure says whether the world never became trained, the bootstrap failed, or the leverage path changed.

## Acceptance · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-acceptance`

The implementation report identifies the integration-test mint and path, fixed seeds, configured $N_\text{boot}$ and $\alpha_\text{boot}$, each arm's pre-arrival training gap and rank result, selected slot offsets, initial and final moments, derived endpoint magnitudes, completion-event counts, and the leverage arm's first-label model traces.

The report shows each late Sentinel begins on mean zero and variance one at the selected z-score positions, remains incomplete after ninety-nine reporting assessments, completes on the configured hundredth, gives the scale arm mean 8 and in-scale endpoints in both arms, preserves finite healthy public assessments, and emits each completion once for the correct identifier.

The report shows the operational and sister updates select the leverage cap and every core-model trace satisfies the factor-six precision guarantee, including the anchor's aggregate route. It also includes a controlled failure demonstration or equivalent mutation evidence for disabled bootstrap and disabled leverage bounding.

The report records the selected variance interpretation, all named package gates, repeated deterministic runs, approximate line additions against each entry, and no unbounded polling, wall-clock sleep, ignored diagnostic, production observability expansion, or coverage claim for another intent.
