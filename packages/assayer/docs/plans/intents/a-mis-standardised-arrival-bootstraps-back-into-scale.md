# A Mis-Standardised Arrival Returns to Scale · `plan:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale`

Keeping (´claim:feature:a-mis-standardised-arrival-bootstraps-back-into-scale´) establishes that a late Sentinel whose reported z-scores are grossly displaced from their class prior acquires an empirical scale over the configured per-Sentinel bootstrap horizon, while the production label path bounds the influence of an observation made before that acquisition completes.

## What the promise says, precisely · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-precise-promise`

The witness begins only after the instance-wide cold ramp is in service and a balanced scalar-signal population has produced directional separation, so “joining a trained deployment” is a measured precondition rather than elapsed time or an assumed label count. The existing population-split witness supplies the public training shape (´test:integration:scalar-signal-population-split-converges-directionally´), and the guarded trained-state fixture now measures its class counts, held-out gap, and pairwise rank before returning (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

Registration creates a contiguous Sentinel slot consisting of occupancy followed by the fixed-width extraction (´def:extraction:slot´). The chain z-score group supplies cell, root, maximum, mean, gradient, and spread views while retaining each of the four axes (´tab:extraction:chain-z-scores´); the witness sets every reported chain level to the same endpoint, 5 or 15, so the cell, root, maximum, and mean positions carry that endpoint while the derived gradients and spreads remain zero.

Before the late Sentinel has supplied a bootstrap sample, each selected z-score position carries mean zero and variance one from its feature-class prior (´tab:standardisation:class-priors´), with the assignment derived from the dimension map and fixed extraction layout (´req:standardisation:class-assignment´). A raw endpoint of 5 or 15 therefore enters the first assessment at essentially that standardised magnitude, making the arrival's scale error directly measurable.

The per-Sentinel algorithm opens an accumulator at registration, accepts one slot vector only from an assessment in which that Sentinel reports, blends empirical moments into the slot at completion, and releases the accumulator (´alg:standardisation:sentinel-bootstrap´). The configured horizon is $N_\text{boot}=100$ accepted reporting assessments and the configured blend is $\alpha_\text{boot}=0.8$, leaving one fifth of the previous moments in place (´tab:config:standardisation´).

An equal population at 5 and 15 has empirical mean 10 and population variance 25 at each selected position. On an arm with no labels during acquisition, the implementation's componentwise blend produces mean $0.8(10)+0.2(0)=8$ and variance $0.8(25)+0.2(1)=20.2$; the resulting endpoint magnitudes are $3/(\sqrt{20.2}+\varepsilon)$ and $7/(\sqrt{20.2}+\varepsilon)$, both below 2. The exact variance is an implementation-conformance reading because the bootstrap algorithm specifies a blend of empirical statistics but gives no variance equation.

Observation and continuing learning have different authority: the bootstrap moves later standardisation snapshots from assessment-time observations, while the continuing average moves at label time only after the full-vector cold phase is in service (´req:standardisation:timing´). The assessment supplying the first bootstrap observation scores against the snapshot it already acquired, so its deliberately extreme value remains a prior-scale label input (´dec:ordering:score-before-evolve´).

For that label the production path computes the stored leverage $h=\hat\phi^\top\Sigma\hat\phi$, derives the current-time policy leverage $h_\text{policy}=h/f_t$, and selects $w_\text{eff}=\min(w_\text{target},w_\text{ceiling},c/(h_\text{policy}+\varepsilon))$. The witness holds scenario time fixed across the labelled assessment, so $f_t=1$ and the two leverage readings agree. The default $c=5$ limits the precision increase along the observation direction to a factor of at most six (´prop:update:leverage-bound´), and the weight is reduced before any model mutation (´dec:posterior:leverage-before´).

Completion is joint across matched scale and leverage arms: each late slot starts on class priors, no completion occurs through the ninety-ninth reporting assessment, the configured hundredth assessment produces one completion for the registered Sentinel, the unlabeled arm reaches the componentwise blend, both arms place the 5 and 15 endpoints below magnitude 2, the first extreme label satisfies $w_\text{eff}h_\text{policy}\leq5$ in every updated model, and all public assessments remain finite. The late-arrival caveat remains observable only during this bounded acquisition window (´cav:limitation:late-standardisation´), while the two-acquisition decision explains why registering the new slot does not restart the trained coordinates (´dec:vector:two-acquisitions´).

## What the code offers today · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-code-today`

The unified real-engine harness exposes `WorldBuilder`, `World::register_sentinel`, `World::receive_report`, `World::request_with_sentinel`, `World::assess`, `World::label`, `World::flush_labels`, `World::flush_observations`, `World::drain_health_events`, and `World::settle_cold_ramp_with`. Its queue-specific barriers replace every test-authored sleep or poll, and `World::advance` plus `World::travel_to` are the only scenario-time verbs (´tab:assayer:harness-implementation-library-roster´) (´dec:harness:no-ad-hoc-waits´).

`World::trained_state` returns a `TrainedStateFixture` containing a `World` and its measured `TrainedStateBaseline`; it settles the cold ramp, trains the standard score-verified population, and refuses a world that misses its declared class counts, held-out gap, or pairwise-rank floors. Its focused witness confirms that the measured precondition travels with the returned world (´test:crate:trained-state-fixture-returns-its-measured-precondition´), so this intent no longer needs to design a trained-world fixture.

The report and signal support expose `golden_report`, `make_cell_report`, `GOLDEN_COORD`, and `with_score_verified`. A test-local constructor can retain the coherent golden topology while setting every chain level's four maximum z-scores to one endpoint; these are authored stimuli rather than derived outputs (´dec:assayer:golden-report-stimulus´) (´tab:assayer:harness-golden-figures´).

The declarative runner exposes `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackBarrier`, `PlaybackBatchSize`, `PlaybackCheckpoint`, `PlaybackProgress`, and `playback`. Subject-owned rows apply report, assessment, and optional label actions, while the runner alone owns bounded batching, the ordered barrier policy, exact checkpoints, and progress (´dec:harness:declarative-playback´) (´tab:assayer:harness-implementation-library-roster´).

`World::published_slot_moments` returns one owned `PublishedSlotMoments` from one publication after the model-owner barrier. It supplies the complete slot means and variances together with phase, cold-ramp observation count, version, and layout generation, but deliberately supplies neither feature classes nor a per-Sentinel bootstrap count (´dec:harness:probe-contract´). Its focused witness verifies the single-publication owned reading (´test:crate:published-slot-moments-cross-publication-as-one-owned-slot´).

The production registration path opens a width-aware `BootstrapAccumulator`; assessment prepends occupancy and observes the extracted slot, `ModelOwnerCommand::BootstrapComplete` applies the configured componentwise blend, and `ConvergenceEvent::SentinelBootstrapComplete` reports publication. The nearest crate witness proves that a configured reporting sample fills the accumulator and changes published moments (´test:crate:sentinel-bootstrap-blends-into-published-statistics´), while the independent Welford test proves population variance per position (´test:unit:bootstrap-multi-feature-variance´). Neither test establishes the trained late-arrival promise.

The leverage primitive already proves that target weight, ceiling, and leverage cap each become the selected minimum (´test:crate:effective-weight-is-the-smallest-of-target-ceiling-and-leverage-bound´). The current test-support surface has no bounded per-label reading of the production path's requested weight, policy leverage, selected effective weight, and model identity.

## The witness · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-witness`

The `lifecycle_identity_convergence` integration target remains the destination because late admission is the principal subject (´dec:harness:convergence-split´). The test obtains two `TrainedStateFixture` values from `World::trained_state` with the same seed and distinct instance names, records both `TrainedStateBaseline` values, and requires exact class-count agreement, gap and rank agreement within `DEFAULT_TOLERANCES.default`, and paired held-out public risks within `PURITY_DRIFT` before either world changes (´tab:assayer:harness-scenario-tolerances´).

Each world then registers the same late Sentinel name. A `World::published_slot_moments` reading must show the full-vector phase in service and mean zero with variance one at the sixteen cell, root, maximum, and mean z-score offsets derived from the occupancy-first slot layout; the reading's cold-ramp count is recorded only as the instance-wide phase evidence, not misidentified as per-Sentinel progress. `World::drain_health_events` then clears every setup event and requires no bootstrap completion for the new identifier before playback begins.

The test-local playback row ingests one coherent endpoint report, builds the routed request with `World::request_with_sentinel` and `with_score_verified`, and assesses it. Only the leverage arm's first row submits `LabelSpec::adverse` with `LabelSpec::ground_truth`; every later row and every scale-arm row remains unlabeled. Scenario time stays fixed throughout, so the leverage policy introduces no elapsed-time correction.

Each arm supplies one hundred alternating 15 and 5 rows to `playback` in one-row `PlaybackBatchSize` chunks under `PlaybackBarrierPolicy` containing `PlaybackBarrier::FlushLabels`. That barrier orders each synchronously accumulated slot observation and any resulting `ModelOwnerCommand::BootstrapComplete` before its checkpoint without confusing the command queue with the separate cold-ramp observation queue (´dec:harness:declarative-playback´) (´cor:concurrency:harness-barriers´).

`PlaybackCheckpoint` callbacks after zero-based rows 98 and 99 call `World::drain_health_events` and filter for the registered identifier. The first requires no `ConvergenceEvent::SentinelBootstrapComplete`; the second requires exactly one, and the completed `PlaybackProgress` must account for the full stream. A final drain requires no duplicate completion.

The scale arm's final `PublishedSlotMoments` must report mean 8 and variance 20.2 at every selected offset within `DEFAULT_TOLERANCES.default`, the one-shot arithmetic budget declared by the harness (´tab:assayer:harness-scenario-tolerances´). Both arms independently standardise raw endpoints 5 and 15 from their returned means and variances and require each magnitude to remain below 2; the labelled arm is not compared with the scale arm's exact moments because its first label legitimately moved the retained fifth before bootstrap completion.

For the first labelled row, the operational and sister readings must identify the leverage limit as the selected minimum within `DEFAULT_TOLERANCES.default`. Every updated core-model reading, including the anchor's reduced projection, must satisfy $w_\text{eff}h_\text{policy}\leq5$; selection is not required for the anchor because its projection has different geometry.

Final endpoint requests pass through `World::derive_for_request` in both worlds. Each result must carry finite in-range risk, non-negative uncertainty, the registered Sentinel identity, and no degraded assessment, while `assert_health_clean` excludes feature sanitisation, model fallback, a stopped label path, a cascade terminus, and dropped health events.

A broken implementation that never starts or completes the accumulator leaves the selected means near their prior and emits no completion. One that completes early or counts non-reporting assessments fails the ninety-nine/hundred boundary, one that ignores $\alpha_\text{boot}$ fails the scale arm's exact componentwise readings, and one that removes leverage bounding selects the requested weight or violates $w_\text{eff}h_\text{policy}\leq5$ on the extreme first label.

## What is missing · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-missing`

The shared trained fixture, slot probe, declarative playback, and barriers have landed. What remains is test-local assembly around those surfaces, one bounded production-path trace, and the integration witness itself; the historical harness-stage umbrella owns none of this work.

**Entry (Matched trained worlds establish the arrival boundary)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-world-fixture`

In the `lifecycle_identity_convergence` target, add test-local setup that calls `World::trained_state` twice with the declared seed, reports both returned baselines, checks paired public readings before registration, and returns the two guarded worlds. No new shared fixture is required because `TrainedStateFixture` already owns the trained-state precondition (´dec:harness:guarded-fixtures´) (´test:crate:trained-state-fixture-returns-its-measured-precondition´).

**Entry (A two-point playback row drives the late slot)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-slot-probe`

In the same target, add a test-local report constructor and `PlaybackRow` implementation that preserve the golden topology, set every chain level to endpoint 5 or 15, ingest the report, route the named Sentinel, attach the score-verified signal, assess once, and optionally submit the first ground-truth adverse label. Use the landed `PublishedSlotMoments` projection for readings rather than adding another snapshot accessor (´dec:harness:probe-contract´) (´test:crate:published-slot-moments-cross-publication-as-one-owned-slot´).

**Entry (A scoped update trace identifies the binding limit)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-update-trace`

Add test-support instrumentation around the production label-path decision and an owned, single-label reading under `testing::probes`. For each updated model it records model identity, stored leverage, elapsed-time factor, policy leverage, requested weight, ceiling, safety factor, epsilon, and selected effective weight; activation is scoped to one `World`, the publication barrier completes the sample before it is drained, and no matrix, mutable handle, or host-visible production field is exposed (´dec:harness:probe-contract´).

**Entry (The late-arrival integration witness joins scale and leverage)** · `entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-integration-witness`

Add the indexed integration test for the matched guarded worlds, late registrations, two-point playback, ninety-nine/hundred checkpoints, completion events, slot-moment assertions, first-label leverage assertions, final public assessments, and the test mint that cites the intent. This entry depends on (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-world-fixture´), (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-slot-probe´), and (´entry:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-update-trace´).

## Risks and open questions · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-risks`

**Observation (Into scale has a fixture oracle rather than a global threshold)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-scale-oracle`

The specification gives priors, sample count, and blend weight but no universal absolute definition of “in scale.” This witness derives its bound from the declared two-point population: the scale arm's post-blend mean and componentwise variance put the larger residual at $7/(\sqrt{20.2}+\varepsilon)<2$. The assertion therefore describes this fixture and does not promote 2 to a package-wide threshold.

**Observation (The bootstrap variance blend lacks an equation)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-variance-equation`

The per-Sentinel algorithm says to blend empirical statistics and the implementation blends means and variances componentwise, while the cold-ramp algorithm explicitly includes a between-population variance term. The magnitude-below-two result holds for the implemented componentwise value of 20.2; the exact variance assertion remains implementation conformance until the specification either ratifies that equation or specifies a mixture term.

**Observation (Bootstrap completion crosses the command queue)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-completion-ordering`

The hundredth assessment releases its accumulator after offering `ModelOwnerCommand::BootstrapComplete`, and the owner publishes the blend and emits the health event later. One-row playback with `PlaybackBarrier::FlushLabels` keeps the command queue clear and orders each checkpoint after publication; `World::drain_health_events` then orders all event producers before reading the receiver. No sleep, request count alone, or cold-ramp observation barrier substitutes for those boundaries.

**Observation (One early label also moves the retained standardisation mass)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-label-tracking-overlap`

Because the full-vector phase is in service, the leverage arm's first label advances the continuing average and changes the moments onto which bootstrap later leaves its fifth of retained mass (´req:standardisation:timing´). A shared exact-moment oracle would contradict the production path: the unlabeled scale arm owns the exact mean and componentwise variance, while the labelled arm owns the leverage assertion and its independently derived endpoint bound.

**Observation (Leverage is a directional precision guarantee)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-leverage-oracle`

The specification guarantees a factor on directional precision, not a fixed probability delta (´prop:update:leverage-bound´). The witness therefore checks the production decision and $w_\text{eff}h_\text{policy}\leq c$ rather than fitting a post-arrival risk threshold; calibration and the nonlinear risk blend cannot turn that precision factor into a universal probability difference.

**Observation (Accepted bootstrap samples are reporting assessments)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-accepted-sample-domain`

The configured horizon counts successful observations of the new Sentinel's slot, not all requests served by the deployment. Every playback row carries that Sentinel coordinate and a fresh valid report, while pre-registration and non-reporting assessments are excluded. `PublishedSlotMoments::observation_count` belongs to the instance-wide cold ramp and must not be used as the bootstrap counter; the controlled row boundary and matching completion event provide that evidence.

**Observation (The trained precondition is measured independently)** · `obs:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-trained-precondition`

`TrainedStateFixture` fixes the default configuration and refuses to return until its class counts, held-out gap, and pairwise rank clear their declared guards. The result checks remain separate from those setup facts (´dec:harness:separate-validation´), so a failure identifies whether training never established the precondition, the late bootstrap failed, or the leverage path changed.

## Acceptance · `sec:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale-acceptance`

The integration target is `lifecycle_identity_convergence`, and the test function is `lifecycle_identity_convergence::a_mis_standardised_arrival_bootstraps_back_into_scale`. Its module index describes the witness, and its test documentation mints the integration-test label and cites the kept claim.

The failure report prints the fixed seed, both `TrainedStateBaseline` values, configured $N_\text{boot}$ and $\alpha_\text{boot}$, selected slot offsets, initial and final moments, derived endpoint magnitudes, completion events by identifier, and each first-label model decision.

The test proves each late Sentinel begins on mean zero and variance one at the selected z-score positions, remains incomplete through ninety-nine reporting assessments, completes on the configured hundredth, gives the scale arm mean 8 and componentwise variance 20.2 within `DEFAULT_TOLERANCES.default`, and gives both arms endpoint magnitudes below 2.

The test proves the operational and sister updates select the leverage cap, every updated core model satisfies the factor-six directional precision guarantee, and final public assessments remain finite and healthy. Its falsifiers cover disabled bootstrap, early completion, an ignored blend factor, and disabled leverage bounding.

The implementation passes the package's formatting, lint, test, and corpus-name gates with no sleep, poll, ignored diagnostic, production observability expansion, or claim of coverage for another intent.
