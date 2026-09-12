# Two clocks at the Sentinel boundary · `rep:vector:observation-and-evidence-clocks`

This report is evidence for a decision, not the decision. It re-examines the [cold-start uncertainty step](cold-start-uncertainty-step.md) through two clocks: empirical measurement state that may advance from observations, and outcome knowledge that may advance only from accepted labels. The distinction is tested against source and specification rather than assumed. The report changes no implementation, specification or record. As a prose carrier it cites existing heads and mints none, under the label calculus.

The headline is **the Sentinel boundary embodies the separation, but not the strong claim that all learned state is label-gated**. A Sentinel continuously learns observation geometry and emits yardstick-applied measurements; it never receives Core outcomes. The Core then learns the relationship between those measurements and labelled outcomes. Feed-forward independence makes that a real architectural seam (`tab:architecture:sentinel-properties`); `packages/assayer/docs/spec/foundations-purpose.md:295-317`. The Core breaks the seam internally where an observation-completed standardisation snapshot changes the feature vector used inside quadratic forms with label-trained covariance, without preserving the old coordinate system or naming which clock moved.

The recommendation is **keep the observational clock moving, but make its publication gradual, versioned and visible; do not move batch initialisation to the label clock**. The closest Sentinel precedent is not merely an EWMA. It warms a tracker before it enters reports, scores each live batch against prior state before evolving, uses exponential transitions once online, and reports maturity beside the measurement. Core should enable the held class priors, acquire deployment geometry from assessments, replace the wholesale switch by a declared transition, and expose the standardisation phase, observation count and coordinate version on every assessment. A stricter continuity requirement may add dual-coordinate counterfactuals or posterior reparameterisation, but a purported scalar split into independent “geometry” and “knowledge” factors is not mathematically available from the present blend. The decision remains open.

## The two quantities in Core · `sec:vector:clocks-core-quantities`

The proposed distinction is real as an **authority distinction**. Unlabelled traffic can teach the system what its inputs look like. It cannot teach whether an outcome was adverse, how a model coefficient should move toward that outcome, or whether reported probabilities and intervals cover observed outcomes. The implementation nevertheless has more than two physical counters, and several states sit on each side.

### What advances without a label · `sec:vector:clocks-observation-advance`

| State | Tick and update | Where it can be observed |
| --- | --- | --- |
| Batch-initialisation moments | One accepted raw feature vector per assessment while active. Welford mean and second moment advance before that request is standardised; completion is delivered asynchronously and replaces every non-bias snapshot mean and variance at once. | Later assessments read the newly published coordinate system. `packages/assayer/src/assessment.rs:1107-1138`; `packages/assayer/src/feature/bootstrap.rs:190-294`; `packages/assayer/src/lib.rs:1271-1306`; `packages/assayer/src/owner/thread.rs:540-584`. |
| Per-Sentinel bootstrap moments | One raw occupied slot whenever that Sentinel reports in an assessment; a completed sample is blended into the slot at weight `0.8` and published. | Later features from that Sentinel move. `packages/assayer/src/assessment.rs:940-985`; `packages/assayer/src/lib.rs:1309-1342`; `packages/assayer/src/owner/thread.rs:587-642`. |
| Identity measurement EWMAs | Every assessment updates each active identity cell's per-Sentinel alarm averages at complement `0.05`, then updates volatility from the squared change. The updated state is read into the same assessment's raw features. | Current and later risks can move without a label whenever identity dimensions exist. `packages/assayer/src/assessment.rs:1008-1052`; `packages/assayer/src/lib.rs:1215-1247`; `packages/assayer/src/identity/cell_state.rs:78-109`. |
| Concordance window and thresholds | One aggregate four-axis observation per assessment enters a rolling window. At the configured cadence a percentile replaces the thresholds used by later aggregate features. | Later raw feature vectors move at a publication boundary. `packages/assayer/src/assessment.rs:1057-1071`; `packages/assayer/src/health/concordance.rs:75-106`; `packages/assayer/src/health/concordance.rs:168-197`. |
| Blend-health window and EWMA | Every returned assessment contributes its anchor weight; the EWMA advances on every observation and the percentile view republishes periodically. | Health only; it is not read back into risk. `packages/assayer/src/api/assess.rs:48-64`; `packages/assayer/src/health/blend_stats.rs:103-120`; `packages/assayer/src/health/blend_stats.rs:187-207`. |
| Sentinel report index | Each received Sentinel batch report atomically replaces the cached report independently of labels. | Extraction immediately sees a different measurement surface. `packages/assayer/docs/spec/operations-assessment.md:11-18`. |

The pending entry, assessment identifier, total count, degradation counters, identity-observation enqueue and signal-cache effects also move on assessment, but they are bookkeeping or storage rather than empirical yardsticks. The governing assessment write set explicitly permits the measurement state, concordance window and pending entry, then names batch initialisation and per-Sentinel bootstrap as temporary structural writes (`inv:runtime:enumerated-writes`); `packages/assayer/docs/spec/operations-assessment.md:91-117`.

### What advances from labelled evidence · `sec:vector:clocks-evidence-advance`

| State | Label authority | Effect on later uncertainty or features |
| --- | --- | --- |
| Operational posterior | Every dimension-matched label, irrespective of sister eligibility, updates precision, covariance and mean. | Its mean affects the separately reported operational probability; its posterior does not set the blended sister uncertainty. `packages/assayer/src/owner/label_path.rs:550-582`; `packages/assayer/src/model/bayesian.rs:780-827`. |
| Sister and anchor posteriors | Only eligible labels update both models; the anchor receives its projected vector. | Their means and covariances set point risk, blend weight and mixed variance. `packages/assayer/src/owner/label_path.rs:584-652`. |
| Outcome-axis posteriors and scale EWMAs | A reported axis value passes that axis's eligibility rule; its scale EWMA and posterior update from the label. | Later axis predictions and label-derived outcome-memory features move. `packages/assayer/src/owner/label_path.rs:654-705`. |
| Outcome memory | Accepted labels update identity-cell and Sentinel Ledger outcome EWMAs. | Later raw feature vectors include adverse rate and compressed and raw valence histories. `packages/assayer/src/owner/label_path.rs:749-765`; `packages/assayer/docs/spec/data-model-extraction.md:152-172`. |
| Platt calibration | Every label adds a calibration row; a qualifying refit replaces the sister and anchor calibration parameters. | `kappa_eff` changes both probability and probability-space uncertainty. `packages/assayer/src/owner/label_path.rs:768-888`. |
| Empirical coverage | Stored assessment uncertainty and later outcomes are evaluated only during a qualifying Platt refit. | The result is copied to health and is never multiplied into risk uncertainty (`alg:monitoring:empirical-coverage`); `packages/assayer/src/health/published.rs:367-428`; `packages/assayer/docs/spec/analysis-monitoring.md:160-200`. |
| Continuing standardisation EWMA | Every processed label, after the models have used the old coordinate system, advances feature means and variances at `gamma_std = 0.9998`. | It changes the coordinate system for later assessments and labels. `packages/assayer/src/owner/label_path.rs:891-904`; `packages/assayer/src/feature/standardisation.rs:245-296`. |

That last row is the first qualification to that framing. Core's cold acquisition is observation-driven, but its continuing moving average is label-driven. It still describes feature distribution rather than outcome truth: the vector record says a standardisation statistic “is not evidence,” even as it puts its continuing updates on the label path to avoid scaling a request partly against itself. It also deliberately carries no elapsed-time decay (`req:standardisation:timing`); `packages/assayer/adr/vector.md:92-122`.

### The exact mixing point · `sec:vector:clocks-mixing-point`

For a raw vector `phi`, assessment first reads snapshot means and variances and forms

```text
phi_hat[j] = (phi[j] - mean[j]) / (sqrt(variance[j]) + epsilon).
```

It then evaluates sister and anchor quadratic forms, their uncertainty ratio, their point estimates, the mixture variance and the blended calibration. Finally it reports

```text
p_bad       = sigmoid(rho_eff / kappa_eff)
uncertainty = p_bad * (1 - p_bad) * sqrt(variance_eff) / kappa_eff.
```

The source order and formula are direct (`def:risk:probability`); `packages/assayer/src/assessment.rs:1107-1201`; `packages/assayer/src/risk/blend.rs:165-212`; `packages/assayer/src/assessment.rs:269-328`.

The clock provenance of those factors is:

| Factor | Provenance |
| --- | --- |
| Raw `phi` | Current Sentinel measurements and signals, observation-driven identity measurement EWMAs, and label-driven Ledger and identity outcome memory. It is already a mixed-provenance vector. |
| Snapshot means and variances | Class or neutral priors initially; batch and per-Sentinel acquisition from assessments; continuing EWMA from labels. |
| `phi_hat` | Current raw value expressed in the currently published coordinate system. The cold-start step changes this factor. |
| Sister and anchor means and covariances | Priors until accepted labelled updates; thereafter outcome-evidence state. Covariance is also widened at read time for elapsed snapshot age. `packages/assayer/src/assessment.rs:1156-1184`; `packages/assayer/docs/spec/operations-assessment.md:145-167`. |
| Blend weight | A ratio of quadratic forms. It couples the observation-sensitive `phi_hat` to both label-sensitive covariances; it belongs to neither clock alone. |
| `rho_eff` and `p_bad` | Label-trained means evaluated at `phi_hat`, then divided by label-fitted calibration. The derivative `p_bad * (1 - p_bad)` is therefore mixed too. |
| `variance_eff` | Quadratic forms and a between-model mean term, all evaluated at the current feature geometry, then time-corrected. |
| Empirical coverage inflation | Label-derived report-only diagnosis; absent from the formula. |

The step study's numeric mechanism therefore survives the new framing unchanged: an observation-completed coordinate snapshot meets a label-authorised covariance inside `phi_hat^T Sigma phi_hat`. The new framing changes the remedy, not the cause. It also rules out a unique multiplication of one “geometry factor” by one “knowledge factor”: a quadratic form is directional, the blend weight itself changes with both inputs, and the probability derivative adds another coupling. The anchor makes a global affine split harder still because two of its inputs are computed after standardisation, including a maximum across four coordinates (`def:risk:anchor-model`); `packages/assayer/src/feature/dimension_map.rs:752-778`; `packages/assayer/docs/spec/core-models-risk.md:53-87`.

## What Sentinel does with observation geometry · `sec:vector:clocks-sentinel-geometry`

Sentinel has no outcome-label path. Its “learned” state is an unsupervised model of observations: spatial volume and topology, a low-rank subspace, latent means and variances, per-axis baselines, drift accumulators and coordination contexts. The word *learned* therefore cannot mean *label-gated* in this comparison. The relevant boundary is between learning input geometry and learning its relationship to outcomes.

### Cadence and ordering · `sec:vector:clocks-cadence-ordering`

One `ingest` call counts spatial importance once per coordinate value. It then collects the rows routed through each online tracker and calls that tracker once with its non-empty per-cell batch. The same happens to a coordination tracker once an active context has enough reporting cells. Thus spatial volume is per-event; subspace evolution, latent statistics, score baselines and CUSUM are per non-empty tracker batch; maturity counts the rows in that batch `packages/sentinel/docs/algorithm.md:197-199`; `packages/sentinel/docs/algorithm.md:1264-1346`; `packages/sentinel/src/sentinel/tracker.rs:815-845`.

The central temporal rule is the design's own sentence: **“Scoring precedes evolution — the batch is measured against the prior model, then the model updates.”** Raw four-axis scores and their z-scores are computed before the subspace, latent distribution and fast baseline absorb the current batch `packages/sentinel/docs/algorithm.md:386-426`; `packages/sentinel/src/sentinel/tracker.rs:180-230`; `packages/sentinel/src/sentinel/tracker.rs:659-733`.

Per axis, the Sentinel maintains:

- a fast EWMA mean and variance for z-scores, updated from retained sample moments once per tracker batch;
- a slower EWMA over the same retained samples as the CUSUM reference;
- a clip-pressure EWMA over the batch's clipped fraction; and
- a one-sided CUSUM whose input is the raw batch mean and whose output never feeds a model.

The fast and slow rules, clipping and detector separation are explicit at `packages/sentinel/docs/algorithm.md:685-792`. Coordination is the same machinery one level up: it consumes raw per-cell batch-mean score vectors, first centres them against a four-axis running mean, then learns its own normalisation and its own four fast/slow baseline pairs `packages/sentinel/docs/algorithm.md:912-940`; `packages/sentinel/docs/algorithm.md:944-996`.

### Smoothness, warm-up and the discontinuities that remain · `sec:vector:clocks-transition-behaviour`

For an already-online axis, the moving baselines do not perform a count-gated replacement. Fast state uses `old * lambda + batch * (1 - lambda)`, slow state uses the same form at a larger decay, and clip pressure has its own EWMA. At the defaults their documented half-lives are about sixty-nine tracker batches, about six hundred ninety-three batches and about fourteen batches respectively `packages/sentinel/docs/algorithm.md:760-771`; `packages/sentinel/docs/algorithm.md:864-874`. A standard-library arithmetic check gave `68.9675639365284`, `692.8005491785` and `13.5134073339649`; Core's `0.9998` standardisation rate gives about three thousand four hundred sixty-five label updates. These remain discrete per-batch moves, but each new sample has a fixed fractional influence rather than acquiring wholesale authority at a count threshold.

Cold EWMA seeding is discontinuous: the first accepted batch replaces placeholder mean and variance directly, and slow-from-fast warm-up copies one baseline into another while resetting CUSUM and clip pressure `packages/sentinel/src/ewma.rs:131-161`; `packages/sentinel/docs/algorithm.md:1525-1535`. Sentinel keeps those transitions off the production measurement surface. The root warms synchronously before the first ingest; later cells are Created, Warming and only then Online, and neither Created nor Warming cells receive real observations or contribute reports `packages/sentinel/docs/algorithm.md:1567-1590`; `packages/sentinel/docs/algorithm.md:1638-1644`.

The broader premise that Sentinel never moves a reported quantity discontinuously is false. A reported CUSUM is clamped to zero by its update and may be reset by warm-up or host action; rank adapts at an interval; coordination contexts and cell reports appear or disappear with lifecycle; and promotion changes report resolution atomically. These are named detector, geometry or availability changes, not a hidden batch replacement of an online baseline. The report carries rank, geometry, maturity, both baselines, accumulator and clip pressure so a host can tell which condition moved `packages/sentinel/docs/algorithm.md:773-791`; `packages/sentinel/docs/algorithm.md:1906-2033`.

There is also a within-report timing nuance. Z-scores and the fast-baseline snapshot are captured before the current baseline update, but CUSUM, slow baseline and clip pressure are snapshotted after their current-batch updates `packages/sentinel/src/sentinel/tracker.rs:692-745`. The report is coherent about what each field means, but it is not one universal instant of adaptive state.

## What crosses the boundary · `sec:vector:clocks-boundary-surface`

The Sentinel principle is stated as **“Measure, don't decide.”** Outputs are raw statistical quantities rather than threat levels or actions, and the host owns interpretation `packages/sentinel/docs/algorithm.md:27-35`; `packages/sentinel/adr/001-measures-not-opinions.md:22-51`. The Assayer side adds the stronger feed-forward property: Core does not depend on Sentinel internals, no Core output influences a Sentinel, and “the relationship between the Core and each Sentinel is purely observational” (`tab:architecture:sentinel-properties`); `packages/assayer/docs/spec/foundations-purpose.md:295-317`.

The “already yardstick-applied measurement” description is verified for the fields Core principally consumes, but not for the whole report. A Sentinel report carries raw min, max and mean scores, maximum and mean z-scores, fast and slow baseline state, CUSUM, clip pressure, maturity and geometry `packages/sentinel/docs/algorithm.md:1945-1978`. Assayer's fixed extraction selects six views of maximum z-scores, three views of CUSUMs, structure and maturity, coordination z-scores and CUSUMs, outcome memory and batch context. It does not relay the raw score means or the baseline fields `packages/assayer/docs/spec/data-model-extraction.md:9-24`; `packages/assayer/docs/spec/data-model-extraction.md:26-172`; `packages/assayer/src/extraction/mod.rs:188-259`.

This is a deliberate two-stage statistical boundary:

```text
observations
    -> Sentinel subspace and baselines
    -> z-scores, CUSUMs, maturity and geometry
    -> Core extraction and feature coordinate system
    -> label-trained Core posteriors and calibration
    -> risk and uncertainty.
```

Sentinel therefore supplies a strong precedent for keeping observation adaptation inside measurement and outcome learning outside it. It does **not** establish that Core may never standardise those measurements again: Core has to put z-score, CUSUM, binary, rate, log and outcome-memory positions onto comparable scales for an isotropic model prior. Nor does the boundary make every Core feature measurement-only; outcome memory enters the same raw vector.

## Where the standardisation specification puts the clock · `sec:vector:clocks-specification-timing`

The specification treats standardisation as **coordinate infrastructure**, not as a posterior. Its stated purpose is to make unlike feature units comparable so one isotropic prior is defensible (`def:standardisation:purpose`); `packages/assayer/docs/spec/data-model-features.md:529-543`. It then assigns three feed mechanisms:

1. batch initialisation observes raw assessment vectors and applies empirical statistics on completion;
2. per-Sentinel bootstrap observes raw assessment slots and blends empirical statistics on completion; and
3. exponential tracking advances only at the label-time standardisation step.

The specification expressly calls the first pair assessment-path “auxiliary-accumulator” writes rather than model writes, while the third stops when labels stop (`req:standardisation:timing`); `packages/assayer/docs/spec/data-model-features.md:545-565`.

The class-assignment requirement answers *which prior*, not *when evidence may replace it*. Every position's class is derived from the dimension map, fixed Sentinel extraction layout or signal schema; no caller configures the assignment (`req:standardisation:class-assignment`); `packages/assayer/docs/spec/data-model-features.md:624-639`. The adjacent table fixes the starting moments (`tab:standardisation:class-priors`); `packages/assayer/docs/spec/data-model-features.md:591-622`. Timing is instead unambiguous in batch initialisation: after the first one hundred assessments, empirical mean and variance replace the class priors wholesale (`alg:standardisation:batch-initialisation`); `packages/assayer/docs/spec/data-model-features.md:641-672`. The late-Sentinel bootstrap similarly opens and advances on reporting assessments, then blends its empirical moments (`alg:standardisation:sentinel-bootstrap`); `packages/assayer/docs/spec/data-model-features.md:710-730`.

The warm-up chapter says the same thing in plainer terms: standardisation priors “must replace” themselves with measurements of the deployment's actual feature distribution, and it places that milestone on assessments, well before the label-defined model stages `packages/assayer/docs/spec/analysis-warmup.md:100-117`; `packages/assayer/docs/spec/analysis-warmup.md:171-200`. Moving cold acquisition to labels would therefore be a policy change, not an implementation detail. It would contradict the specified trigger and the vector record's statement that the statistic is distributional infrastructure rather than evidence.

Two corpus defects must not be allowed to decide the ruling accidentally. First, current construction derives classes but publishes neutral zero/unit moments; the call that would initialise the tabulated priors is held back because of this step `packages/assayer/src/snapshot/working.rs:241-275`. Second, the warm-up chapter still claims the per-Sentinel bootstrap does not ship `packages/assayer/docs/spec/analysis-warmup.md:119-127`, while current assessment, command and owner code does accumulate and blend it `packages/assayer/src/assessment.rs:979-985`; `packages/assayer/src/lib.rs:1309-1342`; `packages/assayer/src/owner/thread.rs:587-642`. Neither stale statement licenses freezing the observational clock.

## Options under the two-clock framing · `sec:vector:clocks-owner-options`

### Option C -- move cold acquisition to labelled evidence · `sec:vector:clocks-labelled-acquisition`

Accumulate the batch moments only when an assessment returns with an accepted label. Unlabelled repeated assessments cannot publish a new coordinate system.

- **Purity guarantee:** strongest match to the integration test's scalar stability wording. It still does not make assessment residue-free: pending, measurement, concordance and health state continue moving.
- **Host burn-in:** sparse-label and censored deployments can remain on priors indefinitely; selective labels estimate the distribution of labelled traffic, not necessarily the distribution assessed.
- **Sentinel precedent:** contrary to it. Sentinel adapts measurement geometry from observations and has no labels.
- **Prior unblock:** yes mechanically, but the empirical transition occurs later and can retain the same cliff.
- **Size:** medium implementation and specification change. The sample unit, eligibility rule, replay behaviour and reset/persistence semantics all need an owner decision.

The two-clock framing weakens rather than strengthens this option: it makes an observational statistic wait for evidence of a different kind.

### Keep moving, disclose only · `sec:vector:clocks-disclosure`

Enable the class priors and leave the observation-completed wholesale replacement as specified, but add standardisation phase, accepted-observation count and coordinate version to every assessment; exempt named observation-state advances from the integration test's claim.

- **Purity guarantee:** honestly narrows it to fixed-snapshot derivation and to “no unreported model learning.” It does not preserve repeated-assessment scalar stability.
- **Host burn-in:** the host can see the boundary and can decline to compare across versions, but still receives a step if it consumes both regimes.
- **Sentinel precedent:** matches maturity and geometry disclosure, but not the Sentinel's smooth online baseline transition.
- **Prior unblock:** immediate.
- **Size:** small-to-medium, mostly output schema, health and tests. It explains the cliff rather than removing it.

### Keep moving, smooth the observational transition · `sec:vector:clocks-smoothed-transition`

Acquire assessment moments continuously and blend priors toward them through an EWMA or an explicit finite ramp. Publish coordinate versions throughout and report a maturity measure. The current batch can continue to be scored against prior state before its observation advances the next state.

- **Purity guarantee:** repeated calls legitimately drift, so the integration test must bind per-step movement, phase visibility and fixed-snapshot purity rather than total idempotence.
- **Host burn-in:** no one-batch cliff; early outputs form a declared trajectory. Hosts can gate on maturity as they do on Sentinel noise influence.
- **Sentinel precedent:** closest operational match. Online EWMAs smooth state, score precedes evolution, and cold seeding is kept off the mature report surface.
- **Prior unblock:** yes.
- **Size:** medium-to-large. Publication cadence, concurrency ordering, checkpoint behaviour, variance blending and the chosen half-life must be specified. Smoothing reduces the jump but does not make uncertainty independent of observation history.

### Keep moving, preserve or attribute the output · `sec:vector:clocks-output-attribution`

Treat a coordinate publication as a model-coordinate change. Either transform model means and covariances with the affine change so predictions and quadratic forms are invariant, or evaluate both the old and new coordinates for a bounded transition and report the exact counterfactual delta caused by geometry. The reported result remains one mixed uncertainty, accompanied by an attributable coordinate-change effect rather than a fictitious independent factor.

- **Purity guarantee:** potentially preserves scalar continuity at a publication boundary and gives the strongest “stop the mix” account.
- **Host burn-in:** no unexplained cliff; a dual-coordinate form also gives the host direct evidence of how much geometry moved the answer.
- **Sentinel precedent:** consistent with score-before-evolve and explicit maturity, though more stringent than Sentinel promises.
- **Prior unblock:** yes.
- **Size:** large. The sister and operational affine maps are tractable because the bias carries the centring offset, but precision, covariance and every outcome model must move atomically. The anchor's computed maximum is not one fixed affine coordinate, so exact global reparameterisation needs special treatment or a dual evaluation. Calibration and pending reconstruction also require proof.

### Keep neutral moments · `sec:vector:clocks-neutral-moments`

Leave the held class priors unused. This preserves the current degenerate cold case and therefore the current test result, at the cost already established by the sibling study.

- **Purity guarantee:** passes accidentally for the zero-feature fixture.
- **Host burn-in:** retains an undeclared neutral coordinate system until the same observation-count switch.
- **Sentinel precedent:** none; it suppresses the prior mismatch rather than managing adaptation.
- **Prior unblock:** no.
- **Size:** smallest immediate change and largest unresolved conformance debt.

## Study recommendation, and the decision it awaits · `sec:vector:clocks-recommendation`

Choose **keep moving, smooth the observational transition**, with disclosure as a mandatory part of the same change. Keep batch and per-Sentinel acquisition on assessment observations; enable the tabulated class priors; score each request against the coordinate snapshot it acquired; advance observational geometry only for a later snapshot; replace wholesale publication by a declared ramp or EWMA; and carry phase, observation count, maturity and coordinate version in the assessment health. The purity suite should separately assert bit-identical derivation of one held assessment, no label-model movement from assessments, and bounded, explicitly versioned observational drift.

This is the narrowest recommendation that follows the Sentinel precedent without pretending observation is outcome evidence or that smoothing restores idempotence. If continuity is required rather than a bounded trajectory, escalate to dual-coordinate attribution before attempting full posterior reparameterisation. Do not call a feature norm a geometry uncertainty factor: the present directional covariances and blend make that label mathematically false. Do not use empirical-coverage inflation as compensation; it remains a label-derived, report-only diagnostic (`inv:monitoring:report-only`); `packages/assayer/docs/spec/analysis-monitoring.md:190-200`.

## Premise audit · `sec:vector:clocks-premise-audit`

- **Verified:** there are distinct observational and outcome-evidence authorities. Batch and Sentinel bootstrap observe assessments; posterior and calibration updates require labels.
- **Verified:** the cold-start switch is exactly where observation-acquired standardisation enters quadratic forms whose covariance is prior or label-trained. Empirical coverage is not on that path.
- **Verified with scope:** the Sentinel/Core architecture embodies the desired direction. Observation geometry stays upstream and no Core output feeds a Sentinel. That boundary is deliberate and documented.
- **Refined:** Sentinels do not hand Core only standardised scores. Their report contains raw scores and adaptive state too; Assayer extraction chooses maximum z-scores and CUSUMs alongside maturity, structure, outcome memory and context.
- **Refined:** “learned truth advances only on labels” is sound only when *truth* means outcome relationship. Sentinel subspaces, latent distributions, baselines and coordination models are explicitly learned from unlabelled observations.
- **Error in the strong two-scalar reading:** the raw Core vector already mixes observation measurements and labelled outcome memory, and `phi_hat^T Sigma phi_hat` is a directional coupling. No unique independent observational factor and model-knowledge factor exists in the current formula.
- **Error in the no-discontinuity premise:** Sentinel hides cold baseline seeding before reports and smooths online baselines, but CUSUM resets, rank changes, promotion and context lifecycle can move reported fields discontinuously. Its precedent is managed and disclosed change, not universal continuity.
- **Error in the purity test's explanation:** repeated assessment is not residue-free. The binding derivation guarantee applies to one held assessment, while the assessment contract permits the enumerated writes (`inv:guarantee:derivation-purity`); `packages/assayer/tests/derive_purity.rs:49-72`; `packages/assayer/tests/derive_purity.rs:99-176`; `packages/assayer/docs/spec/operations-assessment.md:91-117`.
- **Specification conflict found:** the warm-up chapter's claim that per-Sentinel bootstrap is absent is stale against current code and the feature chapter. This report changes neither.
- **Process:** no Cargo command, compiler, network access, implementation edit, specification edit or record edit was used. The only arithmetic executable was the timed standard-library Python half-life check.
