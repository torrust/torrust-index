# The cold-start uncertainty step · `rep:vector:cold-standardisation-uncertainty`

This report is evidence for a decision, not the decision. It traces the reported risk uncertainty through the cold-start standardisation transition, tests the commissioned empirical-coverage hypothesis, audits the purity claim, and sizes the available reconciliations. It changes neither implementation nor record. As a prose carrier it cites existing heads and mints none, under the label calculus.

The mechanism verdict is **verified as batch-standardisation completion and refuted as empirical-coverage engagement**. The hundredth accepted raw assessment vector completes a transient accumulator; the model owner then publishes a snapshot whose empirical means and variances replace the class priors. With the held-back priors, the five cold aggregate rates/fractions and one cumulative sum make the squared standardised-vector norm `10.159999194375581` instead of `1`. The uncertainty ratio is therefore `sqrt(10.159999194375581) = 3.1874753637284132`, which is the reported `3.1875x`; including the shipped blend guard gives the recorded binary64 value `2.5199205337637465` exactly, against `0.7905694150420949`. The empirical-coverage inflation factor is report-only, is computed only during a labelled Platt refit, and is never multiplied into `RiskBasis::uncertainty` (`alg:monitoring:empirical-coverage`).

There is one material premise error. The two measured magnitudes and the near-hundred timing are source-consistent, but the stated temporal arrow is not. With `init_from_priors` enabled at construction, source requires the first regime to report about `2.52`; completion replaces those priors with the empirical statistics of the identical raw vectors and requires the later regime to report about `0.79`. The recorded `0.79 -> 2.52` arrow is reversed. A per-reckoning capture containing assessment identifier, uncertainty and snapshot version would settle whether the measurement log or the source under test differed; no such capture is in the tree.

## The value the purity test reads · `sec:vector:cold-purity-observation`

The integration test calls `World::derive_default`, which first asks Core for one assessment and then passes that held assessment, the channel policy, the challenge estimate and derivation configuration to the stateless derivation. Its five-scalar fingerprint reads `RiskBasis::uncertainty` alongside the three probabilities and anchor weight; it does not read `HealthSnapshot::uncertainty_inflation` (`packages/assayer/tests/derive_purity.rs:49-74` and `packages/assayer/tests/derive_purity.rs:136-177`). The world's virtual clock starts at a fixed epoch, so time correction is one throughout this scenario (`packages/assayer/src/testing/world.rs:70-95` and `packages/assayer/src/testing/world.rs:165-205`).

The uncertainty path is:

1. The empty world assembles a sixteen-position raw vector: one bias followed by fifteen aggregate values. With no reporting Sentinel, every aggregate is exactly zero (`packages/assayer/src/feature/aggregate.rs:135-151` and `packages/assayer/src/feature/assembly.rs:278-320`).
2. The assessment offers that **raw** vector to batch initialisation and only then standardises it against the means and variances in the immutable snapshot (`packages/assayer/src/assessment.rs:1111-1140`).
3. The sister and anchor quadratic forms, blend weight, linear predictors, mixed variance and effective calibration parameter are computed from the standardised vector (`packages/assayer/src/risk/blend.rs:150-213`).
4. `RiskBasis::from_blend` takes the square root of the mixed variance and computes `p_bad * (1 - p_bad) * sigma_eff / kappa_eff` (`packages/assayer/src/assessment.rs:268-323`).
5. The risk basis is returned and also frozen into the pending entry; only after the risk has been computed does assessment copy the unrelated health inflation field (`packages/assayer/src/assessment.rs:1201-1283` and `packages/assayer/src/assessment.rs:1348-1365`).

Every direct input is therefore accounted for: raw features; snapshot means and variances; sister and anchor means and covariances; the anchor projection; snapshot age and Core time decay; sister and anchor Platt parameters; and the blend guard. The test fixes time, has zero model means, covariance `10I` from the default prior precision `0.1`, and both Platt parameters at one (`packages/assayer/src/config/types.rs:168-220`, `packages/assayer/src/model/bayesian.rs:145-175`, and `packages/assayer/src/snapshot/working.rs:251-268`).

### The actual count gate · `sec:vector:cold-count-gate`

Cold construction opens a batch accumulator at the snapshot's feature width with `n_init`, whose default is one hundred. Each assessment increments it once after raw assembly. At the target it sends a `BatchInitComplete` command and disables the local accumulator only after successful delivery (`packages/assayer/src/feature/standardisation.rs:178-207`, `packages/assayer/src/feature/bootstrap.rs:190-289`, and `packages/assayer/src/lib.rs:730-743` and `packages/assayer/src/lib.rs:1271-1306`).

The model-owner command overwrites every non-bias mean and variance, flooring the variance, then increments the snapshot version and publishes the result. The command is asynchronous, so the target is reached on the hundredth total assessment but a tight caller can observe the new snapshot a few calls later. That accounts for a transition around reckoning 102--104 without invoking a second counter (`packages/assayer/src/owner/thread.rs:299-315` and `packages/assayer/src/owner/thread.rs:540-583`). The purity test's baseline is itself the first accumulator observation; loop iteration 99 supplies total assessment one hundred.

This is also the only count-gated branch on the numeric path to the observed field in the label-free, Sentinel-free scenario. The concordance and blend trackers also count assessments, but their default publication intervals are one thousand, and neither supplies an input to the current assessment's uncertainty. Pending-buffer pressure is far above this scenario's count.

### Why empirical coverage is not the switch · `sec:vector:cold-coverage-exclusion`

The commissioned suspect is structurally downstream and read-only. Its source forms standardised residuals from **labelled calibration rows**, applies a per-regime sample gate, and reports the 0.975 quantile of absolute residuals divided by `1.96` (`packages/assayer/src/health/published.rs:367-429`). That computation runs inside `refit_platt_calibration`, after a fit, and its result is cached in the published health summary only when discrimination is available (`packages/assayer/src/risk/calibration.rs:343-425` and `packages/assayer/src/owner/label_path.rs:789-838`). A pure assessment adds no calibration row and cannot trigger a Platt refit.

The assessment-side trait default and production implementation merely read the last cached factor for `HealthSnapshot`; neither feeds it into the blend or risk basis (`packages/assayer/src/assessment.rs:740-748`, `packages/assayer/src/lib.rs:1049-1056`, and `packages/assayer/src/assessment.rs:1282`). The governing algorithm says the same thing in terms: the diagnostic is report-only and never rescales uncertainty (`alg:monitoring:empirical-coverage`); `packages/assayer/docs/spec/analysis-monitoring.md:160-205`. The hypothesis is therefore refuted, not merely unsupported.

## Closed-form reproduction · `sec:vector:cold-uncertainty-reproduction`

The class assignment gives aggregate offsets 7--10 and 13 the rate-or-fraction class, offset 14 the cumulative-sum class, and the remaining aggregate positions the z-score class (`packages/assayer/src/feature/standardisation.rs:424-443`). The priors table and implementation agree: rate/fraction has mean `0.3` and variance `0.05`, cumulative sum has mean `2` and variance `25`, z-score has zero and one, and the standardisation denominator adds `epsilon = 1e-8` (`tab:standardisation:class-priors`); `packages/assayer/docs/spec/data-model-features.md:591-624` and `packages/assayer/src/feature/standardisation.rs:127-151` and `packages/assayer/src/feature/standardisation.rs:196-205`.

On the raw zero aggregates, one rate coordinate and the cumulative-sum coordinate become

```text
r = -0.3 / (sqrt(0.05) + 1e-8)
c = -2.0 / (sqrt(25.0) + 1e-8)
```

The bias is not standardised. Five `r` coordinates and one `c` coordinate are non-zero, so

```text
||phi_prior||^2 = 1 + 5*r^2 + c^2
                = 10.159999194375581.
```

The anchor projection gathers the bias and only aggregate offsets 0--5 in this empty layout; those six aggregates are z-scores and remain zero. Its six absent identity positions and two computed positions are also zero. Thus both anchor and projected-sister quadratic forms are `10`, the blend guard leaves the anchor weight only about `1e-13`, both model means give raw score zero, both calibration parameters are one, and `p_bad = 0.5` (`packages/assayer/src/feature/dimension_map.rs:512-540` and `packages/assayer/src/feature/dimension_map.rs:761-787` and `packages/assayer/src/risk/blend.rs:47-62` and `packages/assayer/src/risk/blend.rs:165-213`). To displayed precision, the effective variance is therefore the sister form `10 * ||phi||^2`.

The two regimes follow directly:

```text
u_neutral = 0.5*(1-0.5)*sqrt(10*1)/1
          = 0.7905694150420949

u_prior   = 0.5*(1-0.5)*sqrt(10*10.159999194375581)/1
          = 2.5199205337638602

u_prior/u_neutral = sqrt(10.159999194375581)
                  = 3.1874753637284132.
```

That last ratio omits only the intentionally tiny blend guard. Applying the implementation's exact mixture, with equal zero predictors, closes the final digits:

```text
w = 1 - 10/(10 + 1e-12)
  = 1.000310945187266e-13

sigma2_prior_effective
  = (1-w)*(10*10.159999194375581) + w*10
  = 101.59999194374664

u_prior_exact = 0.25*sqrt(101.59999194374664)
              = 2.5199205337637465

u_prior_exact/u_neutral = 3.1874753637282693.
```

A standard-library-only Python check, run from the worktree, printed:

```text
ratio_measured=3.1874753637282693
rate=-1.3416407264998764 cusum=-0.39999999920000001 norm2=10.159999194375581
before_closed=0.79056941504209488
after_closed=2.5199205337638602 ratio_closed=3.1874753637284132
after_delta=-1.1368683772161603e-13
w=1.000310945187266e-13
sigma2=101.59999194374664
u_exact_blend=2.5199205337637465
delta_measured=0
real 0.00
user 0.00
sys 0.00
```

The guard explains the unguarded closed form's `1.14e-13` difference and reproduces the recorded value exactly; it is also far below the test's `2e-9` budget. No measured quantity is missing from the magnitude derivation.

## Why the shipped zeros hide the transition · `sec:vector:cold-neutral-moment-mask`

What ships is not the class-prior vector. Cold construction publishes mean zero and variance one at every position, while retaining the now-correct class vector; the call to `init_from_priors` is deliberately absent (`packages/assayer/src/snapshot/working.rs:230-268`). Against the empty world's raw vector this produces the neutral standardised vector `(1, 0, ..., 0)` and the `0.7905694150420949` uncertainty.

After one hundred identical raw vectors, Welford's accumulator reports their population mean -- bias one, every aggregate zero -- and zero variance. The owner skips the bias and applies `max(variance, 1e-4)` elsewhere. Subtracting an empirical mean of zero from every raw zero still gives zero, irrespective of the new denominator. The post-completion standardised vector is therefore the same `(1, 0, ..., 0)`, and the uncertainty remains `0.7905694150420949` (`packages/assayer/src/feature/bootstrap.rs:245-289` and `packages/assayer/src/owner/thread.rs:555-568`). Both regimes compute the same quadratic forms, blend and probability derivative. The shipped degeneracy does not remove the switch; it makes the affine transformation's before-and-after outputs coincide on this request.

With held-back priors, the pre-completion vector instead contains the six non-zero coordinates derived above, so the two transformations no longer coincide. The empirical replacement then removes them. This proves both halves of the commissioned precision: shipped zeros hide the boundary, and non-zero priors reveal it.

## The purity question · `sec:vector:cold-purity-boundary`

There are two different contracts in the corpus, and the textual verdict depends on which one is being named.

**Literal derivation contract: no violation.** The binding derivation invariant says the transform reads only its arguments, writes nothing and returns the same output for the same inputs (`inv:guarantee:derivation-purity`). Its proof is structural and applies to the separately callable derivation function (`pf:landscape:purity`). The derivation record deliberately prefers an import guard to behavioural sampling (`dec:derivation:pure-transform`); `packages/assayer/adr/derivation.md:10-31` and `packages/assayer/adr/derivation.md:64-80`. The first purity test holds one assessment fixed and calls that transform repeatedly; it remains bit-identical. Batch initialisation happens before that boundary and changes the input to a later derivation, not the result returned for one fixed input.

**Assessment contract: the count write is expressly allowed.** The assessment specification says assessment writes the pending entry, concordance window and per-dimension measurement state, then adds batch initialisation and per-Sentinel bootstrap as temporary structural writes while active (`inv:runtime:enumerated-writes`); `packages/assayer/docs/spec/operations-assessment.md:70-118`. Thus “a read writes nothing” is not the governing assessment contract. A completed batch initialiser publishing empirical standardisation is the specified protocol: after the first hundred assessments the priors are replaced wholesale (`alg:standardisation:batch-initialisation`); `packages/assayer/docs/spec/data-model-features.md:641-672`.

**The test's own claim: violated as written.** Its prose was stronger than both governing texts. It said assessing without labelling “leaves no residue,” that all five risk scalars remain inside the drift band, and that “only the reckoning counter moves” — the claim then minted as ``claim:scenario:repeated-assessment-of-one-request-leaves-no-state-behind``, exhibited here because the ratified ruling later retired that test and its wording; the successor states evidence authority instead (`claim:scenario:repeated-assessment-advances-observational-geometry-and-no-outcome-learned-state`). At the study's writing, even on the shipped tree, each call inserts a pending entry and advances concordance, batch-init and blend observations; with priors enabled, the promised uncertainty stability also fails. The test checks the five scalars and clean flags, not the asserted absence of those allowed writes.

The two-way verdict is therefore: **no breach of the binding derivation-purity contract and no forbidden assessment write, but a direct breach of this integration test's literal scalar-stability claim and of its read-idempotence spirit**. The test is useful as a detector of a host-visible warm-up boundary; its “only counter” explanation is not a faithful statement of the assessment contract.

## Count and window census · `sec:vector:cold-threshold-census`

The following table separates assessment-driven state from labelled health. “Host-visible” includes later assessments and the public health surfaces, not only the five-scalar fingerprint.

| Trigger or window | What changes | Host-visible? |
| --- | --- | --- |
| Every assessment | Allocates an identifier, inserts a pending entry, increments `total_assessments`, records degradation, appends one concordance observation and one blend weight, and, while active, offers observations to batch and Sentinel bootstrap accumulators (`packages/assayer/src/assessment.rs:1111-1136`, `packages/assayer/src/assessment.rs:1240-1365`, and `packages/assayer/src/api/assess.rs:28-63`). | Yes: identifier and risk immediately; total count, pending utilisation and tracker health through health reports. |
| `n_init = 100` accepted raw assessment vectors | Batch standardisation sends one completion, overwrites non-bias statistics, emits an event and publishes a new model snapshot (`alg:standardisation:batch-initialisation`). Contention skips an observation and a full command channel defers completion, so the visible request count can exceed one hundred. | Yes: all later standardised features, risk uncertainty, blend, point predictions and outcome intervals can change. This is the studied switch. |
| `n_boot = 100` accepted reporting assessments for one newly registered Sentinel | That Sentinel's occupancy-plus-slot empirical moments are blended at `alpha_boot = 0.8`, an event is emitted and a new snapshot is published (`alg:standardisation:sentinel-bootstrap`); `packages/assayer/src/lib.rs:1310-1343` and `packages/assayer/src/owner/thread.rs:590-643`. | Yes: later risk and predictions can change. Unlike batch completion, the completion send is currently best-effort and the accumulator is released even if it is dropped. |
| Concordance window `5,000`; recalibration every `1,000` assessments | One aggregate four-axis row per assessment enters a circular window; every interval publishes its 80th-percentile thresholds (`packages/assayer/src/health/concordance.rs:45-88` and `packages/assayer/src/health/concordance.rs:169-203`). | Yes in full health, and thresholds feed later aggregate concordance features. No effect in the zero-Sentinel purity world because every row is zero. |
| Blend window `5,000`; publish every `1,000` assessments | Every blend weight enters the window and a seeded EWMA; percentiles, anchor-dominated fraction and steady-state flag are republished at the interval (`packages/assayer/src/health/blend_stats.rs:33-58` and `packages/assayer/src/health/blend_stats.rs:167-250`). | Health-only; it does not feed the risk calculation. |
| Pending capacity, default `720,000`; expiry one day; at most `16` evictions per insert | Unlabelled assessments remain as label context until age or pressure evicts them. The capacity derives from twice the default request rate times default label latency (`req:runtime:buffer-capacity`); `packages/assayer/src/config/types.rs:508-573` and `packages/assayer/src/pending/buffer.rs:47-61` and `packages/assayer/src/pending/buffer.rs:132-152`. | Yes: full health reports utilisation, and eviction makes an old assessment unlabelable. The purity claim's “no residue” is already false here. |
| Every accepted label; calibration buffer cap `2,000` | Adds the assessment-time uncertainty row, evicting the oldest at capacity; increments the Platt cadence; re-standardises stored raw features against current moments; updates models and standardisation EWMAs; publishes model and health (`constr:platt:buffer`); `packages/assayer/src/risk/calibration.rs:158-190` and `packages/assayer/src/owner/label_path.rs:789-900`. | Yes throughout later assessments and health. Pure reads cannot reach it. |
| Platt cadence `200` labels, an explicit early flag, or anchor reactivation after more than `200` labels with sufficient anchor samples | Re-fits eligible regimes, updates both calibration parameters, caches discrimination/coverage, resets affected drift accumulators when the fit moves far enough, and publishes on the label path (`packages/assayer/src/health/platt_tracker.rs:115-149` and `packages/assayer/src/owner/label_path.rs:804-890`). Sentinel registration is the only production early-refit marker today. | Yes: later probability and uncertainty divide by the new effective calibration parameter; health reports fit state and cached diagnostics. |
| Calibration minimum `30` weighted samples per regime, plus at least `3` positive and `3` negative | Governs whether each Platt regime is actually fitted (`req:platt:minimum-samples`); `packages/assayer/src/risk/calibration.rs:74-111` and `packages/assayer/src/risk/calibration.rs:357-384`. Assessment separately reports sister calibration mature at weighted count `30` (`packages/assayer/src/assessment.rs:95-104` and `packages/assayer/src/assessment.rs:1259-1264`). | Yes: fitted calibration changes risk; maturity is embedded in each assessment's health. |
| AUC gate `20` positive **and** `20` negative per partition; recent window `500` | At a refit, aggregate, sister, anchor and recent AUCs become available only when each selected partition meets both class counts; recent selects the newest five hundred rows (`alg:monitoring:auc`); `packages/assayer/src/health/published.rs:231-267` and `packages/assayer/src/health/published.rs:297-345` and `packages/assayer/src/health/published.rs:459-485`. | Health-only. The early aggregate length check is forty, but class balance is the effective gate. |
| Axis-correlation gate `50` rows per axis | At a refit, per-axis point-biserial correlation becomes available for an axis (`packages/assayer/src/health/published.rs:250-260` and `packages/assayer/src/health/published.rs:529-552`). | Health-only. |
| Coverage gate `30` usable rows per `< 0.3` or `>= 0.3` blend partition, evaluated only during a refit that also yields aggregate AUC | Opens residuals from each qualified partition; invalid or non-positive stored uncertainties count toward neither gate nor fractions. The cached factor remains the most recent successful refit's value until another successful refit replaces it (`alg:monitoring:empirical-coverage`). | Yes only as `HealthSnapshot.uncertainty_inflation` and full discrimination health; never as a multiplier on reported risk uncertainty. |
| Composite counts: labels `1`, labels `30` plus a first refit, refits `2`, then configured `5` refits with `delta_cal <= 0.01` | Moves the health stage from ColdStart through AnchorEmerging, PreCalibration, SisterConverging and InteractionMaturing; the last move also requires every identity tracker stable (`packages/assayer/src/health/composite.rs:102-215`). | Health-only stage changes embedded in later assessments. Counts are total labels today despite the source's eligible-label notice. |
| Cholesky cadence, initially `1,000` eligible model updates and adaptive after each outcome | Recomputes model covariance from precision and can shorten or restore the effective interval (`packages/assayer/src/config/types.rs:348-369` and `packages/assayer/src/model/bayesian.rs:842-903`). | Yes: covariance is an input to later uncertainty; this is label evidence, not a pure-read gate. |

The inflation factor's full lifecycle is thus: absent on cold health; an assessment stores its own uncertainty in a pending entry; a later label copies that value into the calibration row; a qualifying Platt trigger computes post-fit residuals; per-regime usable-row gates choose the residual population; the factor is cached only if aggregate AUC exists; label step 17 publishes it; future assessments copy it into embedded health; another successful refit replaces it; and checkpointed owner health and calibration state restore it. No stage of that lifecycle mutates `RiskBasis::uncertainty`.

## What enabling the priors changes beyond this test · `sec:vector:cold-prior-effects`

`init_from_priors` returns the class-derived mean and variance vectors and is currently called nowhere (`packages/assayer/src/feature/standardisation.rs:360-382`). Enabling it at schema cold start changes more than one assertion:

- Every assessment standardises its raw vector against those moments before computing operational, sister, anchor and outcome-axis predictions. The changed risk basis also reaches resonance, guidance inputs and the pending label context (`packages/assayer/src/assessment.rs:1111-1205` and `packages/assayer/src/assessment.rs:1348-1365`).
- Every later label reconstructs raw features and re-standardises them against the working copy before updating operational, sister and outcome-axis models; early labels would therefore learn in the class-prior coordinate system (`packages/assayer/src/owner/label_path.rs:440-478`).
- Batch completion and per-Sentinel bootstrap replace or blend those starting moments, and lifecycle extension, compaction and permutation carry them into every rebuilt layout (`packages/assayer/src/owner/thread.rs:540-643` and `packages/assayer/src/snapshot/working.rs:661-711`).
- Snapshots and checkpoints persist the vectors; health derives features at the variance floor, its standardisation phase and feature-stable outcome-drift flag from them (`packages/assayer/src/snapshot/working.rs:270-327`, `packages/assayer/src/owner/label_path.rs:1264-1290`, and `packages/assayer/src/owner/label_path.rs:1823-1870`).

The values are not discretionary. The specification table fixes each class's prior mean and variance, and the assignment requirement says map position, fixed extraction layout and signal schema determine the class (`tab:standardisation:class-priors`) (`req:standardisation:class-assignment`). The vector record makes the same choice: priors are used before empirical statistics and classes are derived rather than declared (`dec:vector:derived-class-priors`); `packages/assayer/adr/vector.md:126-170`. Wave seven completed the assignment half and intentionally held the initialisation half.

## Options, costs and recommendation · `sec:vector:cold-owner-options`

### Option A -- engage empirical statistics continuously · `sec:vector:cold-continuous-statistics`

Blend prior moments toward batch moments as accepted observations accumulate, or blend the two standardised outputs over a declared band. A one-way batch transition needs a ramp more than hysteresis; hysteresis matters only if reset or dimension churn can move the phase backward.

- **Cost:** medium. The accumulator must publish progress or interim moments, the model owner must publish more often, and the precise mixture-of-moments formula, contention semantics, reset behaviour and checkpoint treatment need specification and tests.
- **Risk:** medium-to-high. A scheduler-dependent accepted-observation count becomes a continuously host-visible input. Interpolating variances naively is not the variance of the mixed populations, while interpolating standardised outputs changes the model coordinate system on every early request.
- **Blast radius:** every early risk, outcome interval, pending context, resonance profile and subsequent label update. It removes the jump but not the fact that pure reads move later answers.
- **Held-prior unblock:** technically yes, but it answers the test by spreading the difference across many failures smaller than its budget unless the contract explicitly permits that drift. It is not the most honest match to the test's stated idempotence.

### Option B -- initialise the empirical factor consistently with the priors · `sec:vector:cold-prior-initialised-factor`

As commissioned, this option is not a repair: the empirical-coverage factor is report-only, starts absent, and no read of it reaches the risk uncertainty. Seeding it cannot change either magnitude. Making it a multiplier would reverse the governing report-only rule and alter every downstream uncertainty consumer (`inv:monitoring:report-only`).

The viable standardisation analogue is to seed the batch accumulator with class-prior pseudo-moments, or reparameterise model means and covariances when the affine coordinate system changes so predictions are continuous at the boundary.

- **Cost:** medium for pseudo-moments and large for exact model reparameterisation across operational, sister, anchor and outcome models.
- **Risk:** pseudo-count strength is a new unsourced parameter and generally reduces rather than eliminates the difference. Exact agreement for every raw vector is impossible between distinct affine maps unless the model is transformed with them.
- **Blast radius:** the full early assessment and label surface; repurposing empirical coverage additionally changes a report-only diagnostic into behaviour.
- **Held-prior unblock:** only the model-reparameterisation form honestly guarantees boundary continuity. Seeding the unrelated inflation factor does not unblock anything.

### Option C -- key empirical engagement on labelled evidence · `sec:vector:cold-labelled-engagement`

Keep class priors for assessments until raw vectors return with accepted labels, and make a labelled-evidence count complete or advance empirical standardisation. Pure reads can fill pending storage but can never change the coordinate system used by another read.

- **Cost:** medium. The accumulator moves from assessment observation to the label reconstruction path, its unit becomes an accepted or eligible label, and the batch-initialisation algorithm, warm-up account, persistence/reset rules and tests need a decision and coordinated edits.
- **Risk:** deployments with sparse or no labels remain on priors longer; selective labels may estimate a biased feature distribution. Choosing all accepted labels versus only model-eligible labels is a real semantic choice.
- **Blast radius:** the transition still changes later assessments and model updates, but only after host-provided evidence already authorises learning; unlabelled repeated reads remain stable.
- **Held-prior unblock:** yes. This most directly reconciles the non-zero priors with the purity test's intended “nothing taught the world” premise.

### Option D -- ship nothing and caveat the hold · `sec:vector:cold-held-prior-caveat`

Retain the shipped all-zero/unit-variance vector, leave `init_from_priors` unused, and document that cold standardisation is neutral rather than class-derived until batch completion.

- **Cost:** smallest immediate change: this report plus a future explicit caveat/ruling.
- **Risk:** preserves known non-conformance with the fixed prior table and makes some features materially too quiet during cold start. It also leaves the overbroad purity claim and the inverted measurement arrow stale.
- **Blast radius:** no runtime change; existing hosts retain current cold-start numbers.
- **Held-prior unblock:** no. It makes the block durable rather than resolving it.

## Study recommendation, and the decision it awaits · `sec:vector:cold-recommendation`

Choose **Option C**, with accepted labelled evidence as the initial candidate unit and an explicit decision if eligibility should narrow it. It is the only sized option that both enables the tabulated priors and makes the test's “untaught world” premise mechanically true: an assessment can still perform the specification's enumerated bookkeeping, but only evidence can change the coordinate system that determines a later risk. Option A is a reasonable follow-on if continuity at the labelled transition is also wanted. Do not initialise or apply empirical coverage as a corrective multiplier; that would repair the wrong mechanism and overturn its report-only authority.

## Premise audit · `sec:vector:cold-premise-audit`

- **Verified:** the purity test begins at line 146, fingerprints uncertainty, runs one baseline plus one thousand repeated assessments, permits `2e-9`, and claims only the reckoning counter moves (`packages/assayer/tests/derive_purity.rs:49-74` and `packages/assayer/tests/derive_purity.rs:136-177` and `packages/assayer/tests/support/constants.rs:34-48`).
- **Verified:** the backlog chronicle says the priors were “derived but held back” because a pure read stepped reported uncertainty near the hundredth reckoning; it identifies the isolated prior-initialisation half (`packages/assayer/docs/plans/backlog.md:2104-2122`).
- **Verified:** the conformance audit's wave-seven repaired paragraph records the two values, the near-hundred timing, the purity falsifier, the two-half isolation, and commit `33f1b406` (`packages/assayer/docs/plans/conformance-audit.md:2662-2682`).
- **Verified:** current construction assigns the classes but deliberately leaves means zero and variances one, with the same `0.79`/`2.52` hold noted in source (`packages/assayer/src/snapshot/working.rs:230-268`).
- **Verified and sharpened:** the actual switch is the configured batch target one hundred plus asynchronous owner publication. Neither rebuild class recomputation nor empirical coverage is on the label-free trigger path.
- **Verified:** the magnitudes and rounded ratio reproduce from shipped constants, class priors and the empty raw vector. The unguarded form differs by about `1.14e-13`; applying the shipped guard reproduces the recorded `2.5199205337637465` exactly.
- **Error:** the recorded chronological `0.79 -> 2.52` arrow conflicts with the source that was temporarily present in `33f1b406`: `init_from_priors` makes the initial snapshot non-degenerate, while batch completion replaces priors with identical-vector empirical zeros. Source requires `2.52 -> 0.79`.
- **Error:** the test's “leaves no residue” and “only the reckoning counter moves” prose conflicts with the governing assessment write-set, the test's own pending insert, and the batch/concordance/blend observations. The scalar assertion is real; its explanation overstates assessment purity.
- **Stale governing prose found:** the empirical-coverage algorithm still says nothing in the package computes the diagnostic, but the conformance audit records the wave-nine implementation and current source computes and publishes it (`entry:assayer:wl-empirical-coverage`); `packages/assayer/docs/spec/analysis-monitoring.md:199-208` and `packages/assayer/docs/plans/conformance-audit.md:521-578`. This staleness does not affect the algorithm's binding report-only clause.

This study used no Cargo command, compiler, network access, Rust edit, specification edit or record edit. Its only arithmetic executable was the timed standard-library Python command quoted above.
