# Withholding discrimination before three positive calibration labels · `plan:assayer:intent-discrimination-is-withheld-until-there-are-positives`

Keeping (´claim:wellness:discrimination-is-withheld-until-there-are-positives´) establishes through the public health surface that zero, one, or two positive calibration labels never produce a numeric discrimination reading.

## What the promise says, precisely · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-promise`

Let $P_+$ be the raw count of accepted calibration rows whose label has positive valence and let $P_-$ be the corresponding non-positive count. The promise requires every health observation taken after a completed refit with $P_+ \in \{0,1,2\}$ to report aggregate discrimination as absent, even when $P_-$ and the total row count are ample.

Positive valence is the binary outcome stored on each calibration-buffer row, and the buffer is populated on the label path from the assessment-time effective estimate and the eventual outcome (´constr:platt:buffer´).

Three is the default positive-outcome floor $n_\text{cal,pos}$ for fitting a regime's calibration parameter; the same floor exists independently for negative outcomes, beside a default thirty-record weighted regime floor (´req:platt:minimum-samples´), (´tab:config:calibration´).

Rank discrimination has a separate and more conservative gate. Aggregate, recent, sister-regime, and anchor-regime AUC are each reported only when their own population contains at least $N_\text{auc,min}$ positive and negative rows, default twenty (´alg:monitoring:auc´), and conforming configuration cannot lower that gate below five (´tab:config:monitoring´).

The promise is therefore a lower-bound guarantee rather than an exact AUC boundary: fewer than three positives must yield absence, while neither the promise nor the specification says that three positives earn a number.

The measurable witness observes completed refits at cumulative positive counts zero, one, and two and requires `HealthSummary::auc_aggregate` and `SystemHealthReport::discrimination` to remain `None`. A later control at five positives under the minimum valid AUC gate requires both public tiers to carry an aggregate figure, proving that the earlier absences came from evaluated evidence rather than from a health path that never published.

Health remains diagnostic: withholding this reading changes no assessment, label, or policy behaviour (´dec:health:reports-never-gates´).

## What the code offers today · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-code`

The public configuration exposes `PlattConfig::n_refit`, `PlattConfig::n_cal_pos`, and `MonitoringConfig::min_auc_class_count` under the calibration and monitoring registers (´tab:config:calibration´), (´tab:config:monitoring´); construction validates the declaration eagerly and enforces five as the smallest public AUC class gate (´dec:construction:eager-validation´).

The calibration module appends `CalibrationEntry` values on the label path, applies the weighted regime sample floors, calls discrimination computation during refit, and returns no discrimination object when aggregate AUC is absent. This follows the per-regime minimum-sample decision (´dec:calibration:per-regime-search´).

The discrimination producer computes each optional AUC from raw positive and negative row counts, the label path retains a successful result, and the public health interface projects that cache into `Assayer::health_summary()` and `Assayer::full_health_report()`; the computation belongs to refit rather than query time (´dec:health:cached-discrimination´), and the two query shapes are the intended cost tiers (´dec:health:tiered-queries´).

The world harness supplies configured seeded construction, the underlying `Assayer` accessor, and `World::flush_labels()`. Its label-specification builder supplies deterministic adverse and benign labels, with adverse meaning positive valence. These pieces are already part of the completed harness skeleton (´entry:assayer:harness-stage-skeleton´).

The existing assessment integration binary already constructs configured scenarios, submits public assessments and labels through `submit_assessment_label`, waits on the label barrier, and reads both health tiers; its existing surface witness demonstrates the same barrier-to-report pattern (´test:integration:standardisation-transition-reported-on-both-surfaces´).

The nearest mechanism test gives the fitter fifty negatives and two positives and verifies that the regime is not refitted (´test:unit:minimum-sample-guard´). It does not inspect either public health tier.

The nearest discrimination tests call the internal metric helper directly: one verifies absence over a two-row sample (´test:crate:discrimination-insufficient-samples-returns-default´), and another verifies suppression immediately below a configured five-per-class gate and presence at equality (´test:crate:discrimination-honours-configured-class-gate-and-recent-window´). Neither crosses assessment, label ingestion, refit, cache publication, and the host-facing query.

The standing gap register has no Entry covering this promise (´sec:assayer:testing-plan-gaps´).

## The witness · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-witness`

The integration test `discrimination_is_withheld_until_positive_evidence_reaches_the_public_gate` belongs beside the existing configured-scenario and full-health witnesses in the assessment integration binary (´test:integration:standardisation-transition-reported-on-both-surfaces´).

Setup builds one seeded scenario with the harness infrastructure, one default channel, a periodic refit cadence of fifty labels, the minimum valid AUC class gate of five, and every other calibration and monitoring value at its default. The calibration buffer remains at its default capacity, so no row is evicted during the witness.

The stimulus submits four blocks of fifty public assessment-label round trips. The blocks add respectively zero, one, one, and three adverse labels, with every remaining label benign, producing cumulative positive counts zero, one, two, and five while negative evidence remains above every relevant floor.

Each block ends with `World::flush_labels()`. The observation then reads `Assayer::health_summary()` and `Assayer::full_health_report()` and verifies that `platt_refits_completed` advanced, so every option assertion is tied to a newly attempted refit rather than to cold-state defaults or a wall-clock guess.

At the first three checkpoints the assertion requires `auc_aggregate.is_none()` on the cheap tier and `discrimination.is_none()` on the detailed tier. These joint assertions establish explicit public absence for every raw positive count below three.

At the five-positive checkpoint the control requires the cheap aggregate AUC to be present and the detailed discrimination object to contain the same present aggregate field. Its numeric value is irrelevant: the control establishes reachability and publication, not ranking quality.

The fixture records the cumulative positive and negative counts locally and asserts them at each checkpoint before reading health, preventing a malformed block schedule from passing as a product result.

The fails-before implementation computes or publishes aggregate AUC whenever both classes are merely non-empty, ignores the configured class gate, substitutes a numeric default for an absent result, or manufactures a detailed discrimination object around absent fields. Such a weakening returns a number at the one- or two-positive checkpoint and fails the public absence assertions; a path that never computes discrimination fails the five-positive control.

No private metric helper supplies an expected result, no score ordering is asserted, and no deliberately broken implementation is executed as part of the witness.

## What is missing · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-missing`

No new harness primitive, fixture module, production accessor, clock seam, drift budget, or liveness deadline is required.

**Entry (Public sparse-positive discrimination witness)** · `entry:assayer:intent-discrimination-is-withheld-until-there-are-positives-public-witness`

This entry adds approximately 55 lines to the existing assessment-and-health integration binary for the configured scenario, four deterministic label blocks, fixture-count guards, refit-freshness checks, absence assertions on both health tiers, and the five-positive reachability control. It depends on (´entry:assayer:harness-stage-skeleton´), requires the generated test index and integration catalogue to be refreshed, and has no sibling-lane dependency.

## Risks and open questions · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-risks`

**Observation (Three and five govern different operations)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-two-thresholds`

The number three gates calibration fitting while the AUC report gate cannot be configured below five. This plan keeps the intent in its literal one-way form and uses five only as a reachability control. If the intended contract is that AUC appears at exactly three positives, the intent conflicts with (´tab:config:monitoring´); the alternatives are to revise the intent to name the configured AUC class gate or to change the specification and public validation before asserting that boundary.

**Observation (Raw labels and effective regime samples are distinct)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-count-semantics`

The promise counts positive calibration labels, AUC counts raw rows per selected population, and calibration fitting counts importance- and regime-weighted samples. The witness makes no inference from fit success and reads only public AUC absence, so weighting cannot turn two raw positive rows into a false pass.

**Observation (Seen and retained differ after eviction)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-retention-scope`

The word seen is naturally cumulative, while AUC is computed over the currently retained circular buffer and the cache preserves the last successful reading until a later successful computation replaces it. The cold-start witness stays below capacity and settles the stated cumulative promise. A stronger rule requiring absence after old positives are evicted is a separate stale-cache contract not established here.

**Observation (Presence has no numerical fragility)** · `obs:assayer:intent-discrimination-is-withheld-until-there-are-positives-determinism`

The oracle is option presence at exact integer class counts. Fixed label blocks, explicit label barriers, a refit counter, and a buffer that never evicts remove timing and scheduling from the observation; the seed and score trajectory cannot move a class count, and no floating-point tolerance enters the assertion.

## Acceptance · `sec:assayer:intent-discrimination-is-withheld-until-there-are-positives-acceptance`

The implementing lane's report identifies the integration test and generated index updates, records the configured refit cadence and AUC gate, and shows cumulative class counts and advancing refit counters at all four checkpoints.

The report shows absent cheap and detailed discrimination readings at zero, one, and two positives, a present aggregate reading on both tiers at five positives, and the inverted fails-before result for an implementation that publishes AUC as soon as both classes are non-empty.

Acceptance includes the package's required formatting, lint, documentation, feature, integration, and release-test gates with commands and verdicts, deterministic repetition without sleeps, no new harness or production surface, and coverage resolution from the new test to (´claim:wellness:discrimination-is-withheld-until-there-are-positives´).
