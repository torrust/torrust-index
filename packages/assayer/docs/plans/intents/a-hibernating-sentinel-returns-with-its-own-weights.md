# Keeping a Hibernating Sentinel's Own Weights · `plan:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights`

This plan keeps the promise that a hibernating Sentinel returns with its own aged model block, without moving learning retained by the live Sentinels or importing correlations with measurement surfaces that did not coexist with it (´claim:lifespan:a-hibernating-sentinel-returns-with-its-own-weights´).

## What the promise says, precisely · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-promise`

Let Sentinel $A$ hibernate at persistent time $t_0$ and steward label sequence $L_0$, and let it return at $t_1$ after $n=L_1-L_0$ accepted labels and $\Delta t=(t_1-t_0)$ hours. The two independent clocks compose once per full-dimension model $m$ as $d_m=\gamma_m^n\gamma_{t,\mathrm{core}}^{\Delta t}$ (´def:temporal:two-mechanisms´) (´dec:posterior:combined-factor´).

Write $S_A$ for $A$'s slot positions at hibernation and $S'_A$ for the positions assigned to the same identifier after re-registration. The archive contains the mean sub-vector $\mu_{S_A}$, the precision block $B_{S_AS_A}$, the covariance block $\Sigma_{S_AS_A}$, and the standardisation entries, but no row or column coupling $S_A$ to any other block (´alg:registry:hibernation´).

For every restored model $m$, the measurable return is $\mu'_{S'_A}=\mu_{S_A}$, $B'_{S'_AS'_A}=d_m B_{S_AS_A}$, and $\Sigma'_{S'_AS'_A}=d_m^{-1}\Sigma_{S_AS_A}$. Lazy posterior decay widens confidence without moving the estimate, so “aged weights” means the archived posterior block under this transformation rather than a scaled mean (´alg:temporal:lazy-application´).

The operational model uses `ModelConfig::gamma_opr`, the sister model uses `ModelConfig::gamma_inh`, and both use `TemporalConfig::gamma_t_core`. The fixture selects rates, an accepted-label count, and a non-zero clock advance for which both factors differ from one and from each other by at least ten times the `Tolerances::default` field returned by `World::tol`; that margin is a declared non-vacuity guard derived from the harness tolerance, not a new product bound (´tab:assayer:harness-scenario-tolerances´).

Restoration is expected only while the configured archive lifetime remains open, the registration recreates the archived layout, every archived precision block is admissible, and each model's aged minimum precision diagonal remains at or above `ModelConfig::lambda_floor` (´entry:construction:hibernation´) (´req:gaussian:prior-replenishment-floor´). The setup guard checks these preconditions before the result oracle runs, so a return at the prior means either a reported setup refusal or a failed restoration rather than an ambiguous assertion (´dec:harness:guarded-fixtures´) (´dec:harness:separate-validation´).

Let $K$ be the union of every position belonging to live Sentinel $B$ and Sentinel $C$, where $C$ registers only after $A$ hibernates. Immediately before and after $A$ returns, $\mu_K$, $B_{KK}$, and $\Sigma_{KK}$ are bit-identical: registration first extends the old posterior as an exact marginal and changes no existing entry (´thm:gaussian:extension´).

Every cross-block entry $B'_{S'_AK}$ and $\Sigma'_{S'_AK}$ is exactly zero. In particular, the $A$–$C$ block has no evidence because the two surfaces never operated together, while the preservation limit also discards every old $A$–$B$ coupling (´cav:limitation:hibernation´).

Scaled-block comparisons use the `Tolerances::default` field returned by `World::tol`; unchanged entries and zero cross-blocks use `Tolerances::exact`. The setup guard separately requires $A$'s archived block to have moved from its pre-learning reading, the absence stream to move the $B$–$C$ reading, and every restored model to stay above the replenishment floor, each by a margin derived from those declared quantities (´tab:assayer:harness-scenario-tolerances´).

## What the code offers today · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-current-code`

The public lifecycle surface exposes `Assayer::hibernate_sentinel`, `Assayer::register_sentinel`, and `SentinelRegistration`. Re-registration restores only when the host supplies the same `SentinelId`, and both public calls publish their model phase asynchronously under the two-phase visibility contract (´dec:construction:two-phase-visibility´) (´entry:construction:hibernation´).

The lifecycle owner takes the archive record during registration, extends every full-dimension model at the prior, admits the record, and calls `WorkingCopy::restore_self_structure` to overwrite only the returning slot. That method obtains one decay per model from the archived label sequence and persistent timestamp, scales precision and covariance reciprocally, leaves the mean unchanged, restores standardisation only when at least one model accepts its block, and reports models left at the prior (´entry:construction:hibernation´).

The finished shared harness supplies `scenario_with_config`, `World::assayer`, `World::register_sentinel`, `World::receive_report`, `World::settle_cold_ramp_with`, `World::advance`, `World::flush_observations`, `World::flush_labels`, `World::published_model_block`, `World::published_slot_moments`, `World::tol`, `LabelSpec`, and `cycle_request` on the one `World` vocabulary (´tab:assayer:harness-implementation-library-roster´). Time now moves through `World::advance`, which advances both clock domains and crosses the work its movement makes due; the earlier direct `World::clock` advance is no longer the scenario contract (´entry:assayer:harness-scenario-time´) (´cor:clock:harness-control´).

`PublishedModelBlock` returns one owned published mean and covariance with its dimension and publication metadata, while `PublishedSlotMoments` returns one owned standardisation slot; both cross `World::flush_labels` and load one snapshot. Precision is deliberately absent, and the probe contract cannot add it (´entry:assayer:harness-probe-contract´) (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

Long learning streams can use `PlaybackRow`, `PlaybackBarrierPolicy`, `PlaybackBarrier::FlushLabels`, `PlaybackProgress`, and `playback`, so the runner rather than the test owns settling cadence and progress (´entry:assayer:harness-tape-runner´) (´dec:harness:declarative-playback´). The independent `decay_recurrence` oracle evaluates the two-clock product by a route distinct from production exponentiation (´entry:assayer:harness-oracle-tier´) (´dec:harness:oracle-tier´).

The crate witness `tests::hibernation::hibernated_sentinel_returns_aged_on_both_clocks` hand-seeds a block and proves operational and sister ageing of mean, precision, and covariance (´test:crate:hibernated-sentinel-returns-aged-on-both-clocks´). Its sibling `tests::hibernation::restored_sentinel_cross_feature_blocks_are_zero` establishes a coupling directly in working state and proves restored precision and covariance cross-blocks are zero (´test:crate:restored-sentinel-cross-feature-blocks-are-zero´). Those direct crate-level readings are the correct home for precision excluded from published probes (´cav:retention:probe-boundary´).

Those focused crate tests mint narrower claims and drive `handle_lifecycle_submission` directly; neither cites this intent's claim, learns with a live Sentinel while $A$ is absent, admits a new Sentinel during the absence, or traverses the public lifecycle and publication barriers. The promise therefore remains unkept even though its ageing and zero-coupling mechanisms have focused coverage.

The integration witness `register_deregister_round_trip` exercises the destructive name-first lifecycle path (´test:integration:register-deregister-round-trip´), while `root_destroyed_on_deregister_then_fresh_on_reregister` proves that destructive re-registration begins a fresh lifetime (´test:integration:root-destroyed-on-deregister-then-fresh-on-reregister´). Neither invokes the hibernating disposition or observes model parameters.

## The witness · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-witness`

The public witness belongs in the integration target `lifecycle_algebra` as `hibernating_sentinel_returns_with_its_own_weights`; its module index states the measurable promise, and its documentation cites this intent and the claim.

- Setup: construct a seeded scenario with `scenario_with_config`, deliberately distinct `ModelConfig::gamma_opr`, `ModelConfig::gamma_inh`, and `TemporalConfig::gamma_t_core`, an archive lifetime longer than the planned interval, and a floor low enough for both aged blocks. Register reporting Sentinel $B$ before $A$, retain $A$'s `SentinelId` and matching `SentinelRegistration`, drive the cold ramp through `World::settle_cold_ramp_with`, and guard the observed settled phase before training.

- Initial learning: define a subject-owned `PlaybackRow` whose request contains reports from $A$ and $B$, whose accepted label is built with `LabelSpec`, and whose `PlaybackRow::play` method chooses no wait. Run the typed rows through `playback` with a `PlaybackBarrierPolicy` containing `PlaybackBarrier::FlushLabels`; the guard requires the post-training $A$ readings to differ from their pre-training readings by the declared non-vacuity margin (´dec:harness:declarative-playback´) (´dec:harness:guarded-fixtures´).

- Archive reading: load operational and sister `PublishedModelBlock` values and $A$'s `PublishedSlotMoments`. Resolve $A$'s positions from the slot ranges copied by the same model projection, then retain its mean and covariance self-blocks and its complete standardisation moments; each reading has already crossed the publication barrier and loaded one snapshot (´dec:harness:probe-contract´).

- Hibernation boundary: call `Assayer::hibernate_sentinel` through `World::assayer` with the retained identifier, then cross `World::flush_labels`. Require $A$'s published slot moments to be absent, register reporting Sentinel $C$ through `World::register_sentinel`, and capture a settled $B$–$C$ baseline. This escape hatch is confined to the hibernating call that `World` does not wrap; names, time, barriers, reports, labels, and readings remain on the shared scenario.

- Absence stimulus: move persistent time by the declared $\Delta t$ through `World::advance`, then play exactly $n$ accepted alternating rows with $B$ and $C$ co-active under the same label-barrier policy. The completed `PlaybackProgress` and accepted label results establish the exponent rather than inferring it from elapsed execution (´cor:clock:harness-control´) (´dec:harness:no-ad-hoc-waits´).

- Absence observation: take pre-return operational and sister projections, require the semantic $B$–$C$ mean or covariance subspace to have moved from its post-registration baseline by the non-vacuity margin, and retain its complete mean and covariance blocks as the published state restoration must not move.

- Return stimulus: call `Assayer::register_sentinel` through `World::assayer` with the retained registration, then cross `World::flush_labels`. Resolve the returned $A$ range and the live $B$ and $C$ ranges from each final one-load projection rather than carrying numeric positions across the layout change.

- Ageing oracle: compute each model's factor with `decay_recurrence` from the configured label rate, the accepted row count, the configured time rate, and the declared clock advance. Compare $A$'s final published mean and covariance self-block with its archived reading under $\mu'=\mu$ and $\Sigma'=d_m^{-1}\Sigma$, compare final `PublishedSlotMoments` with the archived moments bit for bit, and require the restored block to remain distinguishable from its pre-learning reading (´dec:harness:oracle-tier´).

- Isolation oracle: compare the semantic $B$–$C$ mean and covariance subspace before and after return bit for bit, including its learned coupling, then require every published covariance entry between $A$ and either $B$ or $C$ to be exact zero. The existing direct crate witnesses remain the precision evidence for $B'=d_mB$ and zero precision cross-blocks because a public probe may not carry $B$ (´cav:retention:probe-boundary´).

- Completion observation: require $A$'s original identifier to have a published slot again, ingest one well-formed report through each Sentinel with `World::receive_report`, obtain well-formed assessments, and finish with clean health so parameter agreement cannot conceal a degraded lifecycle path.

The fails-before is an implementation that takes the hibernation record but leaves $A$ at the extension prior. It preserves registry success, finite assessments, survivor readings, and clean health, so the destructive lifecycle witnesses remain green, while the returned mean, covariance, standardisation, and non-prior assertions reject it. The focused crate assertions separately reject a missing clock factor, a shared model rate, incorrect precision scaling, or restored stale precision couplings.

## What is missing · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-missing`

**Entry (Subject-local identity-preserving hibernation driver)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-hibernation-verbs`

The integration target `lifecycle_algebra` needs a small subject-local helper that retains $A$'s `SentinelId` and `SentinelRegistration`, invokes `Assayer::hibernate_sentinel` and `Assayer::register_sentinel` through `World::assayer`, and crosses `World::flush_labels` after each call. It adds no shared lifecycle vocabulary and no production behavior; its purpose is to keep the identifier and barrier obligations together while the rest of the scenario remains name-first.

**Entry (Published model projections identify Sentinel slots)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-model-block-probe`

The `testing::probes` projection needs `PublishedModelBlock` to carry an owned mapping from each live `SentinelId` to its half-open slot range, copied from the same snapshot load that supplies the mean and covariance. This is dimension-map metadata already present in the publication, not precision or a second load, so it stays inside the probe and retention contracts while letting the witness compare semantic blocks across layout changes (´dec:harness:probe-contract´) (´cav:retention:probe-boundary´).

**Entry (The public hibernation integration witness)** · `entry:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-integration-witness`

The integration target `lifecycle_algebra` needs `hibernating_sentinel_returns_with_its_own_weights`, its indexed documentation, subject-owned playback rows, guarded setup readings, the two published-model snapshots per boundary, the `decay_recurrence` expectations, survivor and cross-covariance comparisons, restored standardisation comparison, and final public assessments described above. It consumes the existing focused precision tests rather than widening the published probe.

## Risks and open questions · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-risks`

**Observation (Ageing changes confidence, not the posterior mean)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-ageing-semantics`

The phrase “weights aged” can be read as multiplying $\mu$, but the specification fixes the executable meaning: $\mu$ returns unchanged while $B$ scales down and $\Sigma$ scales up reciprocally. The public witness states the mean and covariance equations, while the focused crate witness also states the precision equation; neither treats a multiplied mean as acceptable.

**Observation (Semantic blocks move when the layout moves)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-semantic-blocks`

$A$ can occupy a different numeric range after $C$ joins. Every comparison resolves $A$, $B$, and $C$ from the slot mapping in the projection taken at that boundary; retaining numeric indices or relying on registration order could compare unrelated features and pass while restoration was wrong.

**Observation (Exact, approximate, and excluded readings have different jobs)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-numerical-oracles`

The decay oracle contains repeated multiplication and block scaling, so its published comparison uses the declared one-shot tolerance. Survivor entries copied by extension and new covariance couplings stored as zero use exact equality. Working precision is neither approximated nor reconstructed from covariance in the integration target: the direct crate tests read it at its permitted boundary (´tab:assayer:harness-scenario-tolerances´) (´cav:retention:probe-boundary´).

**Observation (The fixture keeps every refusal path idle)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-refusal-preconditions`

Expiry, layout mismatch, an inadmissible archived matrix, and decay below the replenishment floor all specify a return at the prior rather than failed registration. The setup records the live layout, stays inside the configured lifetime, keeps spatial-axis membership unchanged, and checks the factor and floor inequality before interpreting a prior-like return as evidence against hibernation.

**Observation (The combined scenario adds the missing evidence)** · `obs:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-compositional-scope`

The focused crate tests already hold ageing and zeroing at the precision boundary. The new public witness earns the intent by placing accepted public label learning, forward-only time, and a new registration between archive and restore, then proving through one-load published readings that the returning, surviving, and newly admitted blocks coexist under one lifecycle. No new precision projection is admissible or needed.

No product decision remains open. The specification fixes preservation and ageing, the construction record fixes custody and refusal behavior, and the harness record fixes the probe boundary; the remaining work is the narrow slot-range projection and the public scenario.

## Acceptance · `sec:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights-acceptance`

The implementing report names cargo integration target `lifecycle_algebra` and Rust function `hibernating_sentinel_returns_with_its_own_weights`, confirms its module index and documentation cite the intent and claim, and shows the coverage report resolving the claim to that witness.

The report confirms that `PublishedModelBlock` obtains slot ranges from the same published load as its existing fields, remains owned and read-only, carries no precision, and stays absent from a sealed build. It also confirms that the subject-local lifecycle helper preserves one host-supplied `SentinelId` and crosses `World::flush_labels` after both public lifecycle calls.

The reported assertions show a non-vacuous archived block, distinct operational and sister decay factors, unchanged returned means, reciprocally scaled published covariance blocks, restored standardisation, accepted absence learning that materially moved the $B$–$C$ subspace, bit-identical survival of that subspace across $A$'s return, exact zero published $A$ cross-covariances, live post-return assessments, and clean health.

The report shows `tests::hibernation::hibernated_sentinel_returns_aged_on_both_clocks`, `tests::hibernation::restored_sentinel_cross_feature_blocks_are_zero`, and the new integration function passing under the package's prescribed test, lint, formatting, and corpus gates, with unrelated failures separated explicitly.

The report states the deliberately broken prior-reset behavior and names the published archived-block assertion that rejects it; it also identifies the focused precision assertions that reject a missing clock factor, a shared model rate, incorrect precision scaling, and stale precision cross-terms. No executed mutation is required.
