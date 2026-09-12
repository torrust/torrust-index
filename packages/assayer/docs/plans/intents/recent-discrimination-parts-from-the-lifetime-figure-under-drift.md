# Recent discrimination parts from the lifetime figure under drift · `plan:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift`

Keeping (´claim:wellness:recent-discrimination-parts-from-the-lifetime-figure-under-drift´) establishes that the public health surface preserves a visible, directional separation between recent and aggregate discrimination when a changed population loses the ranking quality learned from its history.

## What the promise says, precisely · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-promise`

The aggregate quantity is the rank-discrimination AUC over every entry currently retained in the calibration buffer, while the recent quantity is the same statistic over the newest configured count of entries; both quantities and their minimum positive and negative class gates are defined by (´alg:monitoring:auc´).

The reference configuration retains 2,000 calibration rows and refits every 200 labels (´tab:config:calibration´), takes the newest 500 labels as the recent population, and requires at least 20 positive and 20 negative rows in each reported population (´tab:config:monitoring´).

The measurable assertion is directional: after a stable, strongly ranked prefix and a rank-neutral recent suffix whose adverse base rate rises, `auc_aggregate - auc_recent > 0.1`; the detailed report also sets `discrimination_instability` because the specified absolute separation is strictly greater than 0.1 (´alg:monitoring:auc´).

The word lifetime denotes the whole bounded calibration buffer rather than all process history: the circular buffer stores the assessment-time effective estimate, outcome, regime weight, uncertainty, importance weight, and order needed by the metric, and evicts its oldest row at capacity (´constr:platt:buffer´).

The witness keeps both class counts above their gates and the calibration fit above its separate evidence floor (´req:platt:minimum-samples´), so absence of either AUC cannot masquerade as stability.

The readings remain diagnostics only: the report exposes the drift without changing scoring or policy (´inv:monitoring:report-only´), and the package guarantee requires rank discrimination to remain visible through the health surface (´inv:guarantee:discrimination-visibility´).

## What the code offers today · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-code`

The public monitoring configuration exposes `min_auc_class_count` and `recent_discrimination_window`, while the public calibration configuration exposes the buffer capacity and refit cadence; builder validation rejects an invalid override before a scenario starts (´dec:construction:eager-validation´).

The published health state carries `DiscriminationMetrics` with `auc_aggregate`, `auc_recent`, and `discrimination_instability`, and the monitoring-aware discrimination calculation computes them when calibration refits.

The public health API gives the test path: `Assayer::health_summary()` returns both cached AUCs and `platt_refits_completed`, while `Assayer::full_health_report()` returns the complete discrimination record; the cheap and detailed tiers are the intended split (´dec:health:tiered-queries´), and discrimination is intentionally cached from one refit to the next (´dec:health:cached-discrimination´).

The world harness supplies seeded `WorldBuilder` construction, `World::settle_cold_ramp_with`, public assessment derivation, ordered label submission, `World::flush_labels`, the `Assayer` access path, and the independent label and observation barriers.

The signal fixtures supply `score_verified_request_schema` and `with_score_verified`; the label fixture supplies deterministic adverse and benign `LabelSpec` values; the liveness harness supplies model-owner fail-fast and progress deadlines.

The completed scenario skeleton already owns the deterministic world, labels, barriers, and tolerances (´entry:assayer:harness-stage-skeleton´), while the standing tape entry explicitly owns long ordered streams, snapshot capture, convergence predicates, and domain assertions (´entry:assayer:harness-stage-tapes´).

The nearest public witness proves that the score-verified request path can learn a directional population split and measures a host-side pairwise AUC (´test:integration:scalar-signal-population-split-converges-directionally´).

The nearest metric witness feeds calibration rows directly and raises the instability flag when a recent half differs from the aggregate (´test:crate:instability-flag-rises-on-recent-departure-from-aggregate´); it bypasses the public assess-label-refit-publication path and does not compare either published value with an independently ranked population.

## The witness · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-witness`

The integration test joins the existing convergence scenario (´test:integration:scalar-signal-population-split-converges-directionally´) and drives only the public API plus the public, documentation-hidden testing harness.

Setup builds a seeded world with the score-verified schema, the default 2,000-row buffer, 500-row recent window, 20-per-class AUC gate, and 200-label refit cadence, then settles the cold ramp with representative low-score and high-score requests before recording any labels.

The stable prefix contains 1,500 closed-loop assessments at a five-percent adverse base rate, with adverse outcomes assigned to the high score rung and benign outcomes to the low score rung; its label count and class balance are fixture assertions rather than assumptions.

At the last prefix refit the test records `platt_refits_completed`, requires both AUCs to exist, and requires the prefix-only independent oracle to show strong and mutually consistent recent and aggregate ranking before drift begins.

The recent suffix freezes 500 assessments before revealing any of their labels, preventing label-by-label model adaptation from becoming an accidental time signal; it retains each public assessment's `rho_eff`, assessment identifier, score rung, and eventual outcome in submission order.

The suffix is divided into equal deterministic blocks whose adverse fraction rises from the prefix rate toward one half; a rotating, score-balanced assignment of adverse rows makes outcome rank-neutral within the suffix while keeping every score rung and both outcomes represented throughout the change.

Labels are then submitted in tape order and acknowledged in bounded chunks, with model-owner fail-fast and progress checks around each barrier; the final label is number 2,000, so the ordinary cadence publishes a fresh discrimination record for the complete buffer.

Observation reads the cheap summary and detailed report only after the final label barrier, verifies that `platt_refits_completed` advanced beyond the prefix stamp, and extracts the detailed `DiscriminationMetrics` record.

An independent Mann–Whitney oracle reranks the full tape and its newest 500 rows separately with average ranks for ties, verifies each class gate, and computes aggregate and recent AUCs without calling the implementation under test.

The fixture first asserts that the oracle's recent AUC is near chance and that its aggregate-minus-recent separation clears 0.1 by a guard margin; these guards distinguish a malformed drift tape from a product failure.

The product assertions require both public AUCs to lie in the unit interval, require each to match its corresponding oracle within the named AUC tolerance (´tab:assayer:harness-scenario-tolerances´), require `auc_aggregate - auc_recent > 0.1`, require `discrimination_instability`, and require the cheap summary to carry the detailed report's same cached pair.

The fails-before is a deliberately broken implementation that copies the aggregate into the recent slot, selects rows by score order instead of label order, compares regime partitions instead of time partitions, reuses whole-buffer ranks for the recent subset, omits the strict flag, or fails to publish the completed refit; at least one direction, range, oracle-agreement, flag, freshness, or surface-consistency assertion then fails.

## What is missing · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-missing`

The standing testing plan already covers this promise through the open long-stream fixture entry (´entry:assayer:harness-stage-tapes´); no new standing gap entry is invented.

**Entry (Ordered two-phase discrimination tape)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-ordered-tape`

This entry adds approximately 80 lines to the shared testing harness for a tape that captures assessment-time `rho_eff`, defers a frozen cohort's labels, replays those labels in order, flushes bounded chunks, and reports fixture counts plus refit progress. It depends on (´entry:assayer:harness-stage-skeleton´) and realizes the relevant part of (´entry:assayer:harness-stage-tapes´); it has no sibling-lane dependency.

**Entry (Independent subset-rank AUC oracle)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-auc-oracle`

This entry adds approximately 45 lines to the shared testing harness for local sorting, average tie ranks, class-gate validation, unit-interval validation, and named-tolerance comparison. It depends on the ordered-tape entry and the metric definition (´alg:monitoring:auc´), and it has no sibling-lane dependency.

**Entry (Moving-base-rate rank-collapse fixture)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-drift-fixture`

This entry adds approximately 55 lines to the integration convergence test module for the stable prefix, frozen recent blocks, rotating score-balanced outcomes, and fixture guards. It depends on the ordered-tape and AUC-oracle entries and has no sibling-lane dependency.

**Entry (Recent-subset reranking correction)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-subset-reranking`

This entry changes approximately 30 lines in the published discrimination calculation so every filtered AUC population receives ranks local to that population before its Mann–Whitney sum. It depends on the AUC-oracle entry for evidence, applies equally to recent and regime subsets, and has no sibling-lane dependency.

**Entry (Public drift witness)** · `entry:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-public-witness`

This entry adds approximately 45 lines to the integration convergence test module for refit freshness, public range and oracle agreement, directional separation, instability, and cheap-versus-detailed report consistency. It depends on all four preceding entries and has no sibling-lane dependency.

## Risks and open questions · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-risks`

**Observation (Aggregate is bounded history)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-bounded-history`

The promise's lifetime wording can suggest process-lifetime accumulation, but the specification and code define the comparator over the bounded calibration buffer. Filling exactly one default-capacity buffer removes eviction history from this witness while preserving the specified aggregate.

**Observation (Prevalence drift does not imply AUC drift)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-prevalence-and-ranking`

A base-rate shift alone does not force rank discrimination to move when the conditional score ordering is unchanged. The witness therefore treats the shifting base rate as the population context and the stated recent collapse as a simultaneous loss of score-outcome association; a prevalence-only interpretation would require a different metric or a revision of the promise.

**Observation (Filtered populations require local ranks)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-local-ranks`

The current implementation assigns ranks across the whole buffer before filtering the recent and regime populations, while the Mann–Whitney formula subtracts the triangular term for the filtered population. The resulting subset values can leave the unit interval, so the oracle and range assertions deliberately expose the defect rather than accommodate it.

**Observation (Refit freshness is observable)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-refit-freshness`

The health record is cached at refit rather than recomputed by a query (´dec:health:cached-discrimination´). Exact label counts and an advancing `platt_refits_completed` stamp make the final observation deterministic without a manual-refit hook or elapsed-time wait.

**Observation (Numerical slack belongs in the fixture guard)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-numerical-slack`

The contractual threshold remains the strict value 0.1, while the fixture targets a visibly larger oracle separation and uses the AUC tolerance only for implementation-versus-oracle agreement. This keeps floating-point noise away from the contract boundary and prevents a tolerance from weakening the promise.

**Observation (Order and concurrency remain controlled)** · `obs:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-ordering`

Assessment ingestion and label processing have distinct barriers, and per-label closed-loop adaptation can couple chronology to predictions. Cold-ramp settlement, frozen suffix assessment, ordered label playback, explicit label barriers, and progress deadlines remove scheduler timing from the measured effect.

## Acceptance · `sec:assayer:intent-recent-discrimination-parts-from-the-lifetime-figure-under-drift-acceptance`

The implementation report identifies the integration test and every harness or production file changed, records the configured buffer, recent window, class gate, refit cadence, prefix and suffix sizes, base-rate schedule, seed, and AUC tolerance, and confirms that no private metric helper supplies the expected values.

The report shows the prefix oracle guard, final full-buffer and recent oracle AUCs, corresponding public AUCs, their directional difference, the instability flag, the prefix and final refit stamps, both class counts, and agreement between the cheap and detailed health tiers.

The report includes a red-before result for the subset-ranking defect or an equivalent deliberately weakened implementation, a green result after the correction and witness land, and the package's required formatting, lint, integration, and release-test gates with their commands and verdicts.

Acceptance requires deterministic repetition under the fixed seed, no wall-clock sleeps, no widened production visibility, no assertion derived from the implementation under test, and a final public result satisfying `auc_aggregate - auc_recent > 0.1` with both values inside the unit interval.
