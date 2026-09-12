# Whether the Platt movement comparison needs a named threshold · `rep:calibration:platt-movement-threshold-evidence`

This report is evidence for a decision, not the decision. The displacement-anchored direction has already fixed the comparison coordinate as relative log-sharpness and retained two authored stimuli: a four-percent score scaling must be admitted and a ten-percent scaling must be rejected. The open question is whether the interval between them should contain one named movement threshold and, if so, what stationary false-alarm cost that name buys at the buffers and cadences the product can actually use.

The report changes no specification, record, test or implementation. As a prose carrier it cites existing heads and mints none, under the label calculus.

All source and test locations in this report are observations at base commit `47246d5ad7453b1723d6af3491a9dfd93c004ea5`. A sibling code lane may change the tolerance sites after that commit; such a change does not alter the stationary sizing here.

The headline is: **do not name one unconditional threshold.** The two stimuli support an interval, not a point. If a complete fixture-only oracle is nevertheless wanted, `log(1.07) = 0.067658648` is the rounded candidate with nearly equal log-margin to both stimuli. It is not a deployment guarantee: across the operating surface below, its stationary per-comparison false-alarm frequency ranges from about one in a thousand to nearly nine in ten, and at the default rolling buffer its expected ten-percent displacement signal never crosses it between adjacent refits. The decision remains open.

## The decision surface · `sec:calibration:platt-decision-surface`

At the base commit, the old absolute allowance is still declared as `platt_kappa = 0.1` in `packages/assayer/src/testing/tolerances.rs:41,70-88`. Three comparison functions consume it. The flat-score unit scenario admits two bit-identical search-bound artefacts and demonstrates that any width is vacuous there (`test:unit:flat-scores-leave-successive-refits-bit-identical`); `packages/assayer/src/risk/calibration.rs:899-964`. The displacement unit scenario scales every ladder score by `1.04` and `1.10`, then admits the smaller absolute κ movement and rejects the larger (`test:unit:overlap-ladder-drift-bound-admits-and-rejects`); `packages/assayer/src/risk/calibration.rs:1007-1062`. The label-pipeline scenario compares the first two refits of one repeated forty-row ladder (`test:crate:platt-sharpness-holds-between-successive-refits`); `packages/assayer/src/tests/label_pipeline.rs:1355-1435`. Those are the complete test-tolerance consumption sites at the named base commit.

The commissioned ruling changes the comparison coordinate, not those scenarios' roles. The displacement scenario pins two decisions in relative log-sharpness: `log(1.04)` is inside and `log(1.10)` is outside. The successive-refit scenario checks movement between adjacent fits, although its deterministic repeated sequence happens to return bit-identical fits. A named threshold would govern all three test comparisons: it would fill in a single decision boundary between the two displacement stimuli and give the successive-refit comparison a numerical allowance. The flat-score comparison remains zero in relative coordinates and still passes without sizing anything. Without a named threshold, only the two one-sided stimulus decisions are contractual; movement in their open interval remains deliberately unspecified.

The production drift-integration threshold is a separate mechanism. Production already computes the absolute log-ratio after a refit and resets prediction drift accumulators when that movement exceeds `0.1` (`alg:platt:drift-integration`); `packages/assayer/docs/spec/core-models-risk.md:452-470` and `packages/assayer/src/risk/calibration.rs:379-421`. It governs accumulator validity after a changed calibration map, not test acceptance of stationary refit movement. This report neither sizes nor proposes changing that reset.

## Stationary movement experiment · `sec:calibration:platt-stationary-experiment`

The experiment extends the prior report's IID overlap-ladder probe rather than inventing a new population. Each authored forty-row block has scores `-4, -2, 0, 2, 4`, eight rows per score, and positive counts `1, 2, 4, 6, 7`. The construction and its unique population optimum `κ* = 1.976282462220253` are in `packages/assayer/src/risk/calibration.rs:630-683`; the pipeline carries the same ladder at `packages/assayer/src/tests/label_pipeline.rs:195-217`.

For balance `π`, the experiment holds the ladder's two class-conditional score distributions fixed and changes only the class mixture. Positive scores are drawn in proportions `1:2:4:6:7`; negative scores in `7:6:4:2:1`. Each row then receives the converged balancing weight `w+ = 1/(2π)` or `w- = 1/(2(1-π))`. That is the specified and shipped equal-halves rule (`def:weighting:balancing-weights`); `packages/assayer/docs/spec/core-models-risk.md:657-680` and `packages/assayer/src/risk/challenge.rs:790-821`. This label-shift construction keeps the population optimum fixed while exposing the extra sampling movement caused by rarer, heavier positive rows.

The operating grid is deliberately configuration-shaped:

| Axis | Values studied | Corpus anchor |
| --- | --- | --- |
| Populated buffer `B` | 100, 200, 500, 1,000, 2,000 | Capacity is at least 100 and defaults to 2,000 (`tab:config:calibration`) |
| Periodic advance `C` | 50, 100, 200 labels | Cadence is at least 50 and defaults to 200 (`tab:platt:refit-cadence`) |
| Positive balance `π` | 0.10, 0.25, 0.50 | The corpus calls positives typically rarer, tabulates 0.10, and the authored ladder is balanced (`tab:weighting:trackers`) |
| Calibration recency | `γ_cal = 0.9998` | The source default is the inherited forgetting rate (`def:platt:objective`) |

The numeric configuration and constraints are together at `packages/assayer/docs/spec/reference-configuration.md:179-201`; the source defaults are at `packages/assayer/src/risk/calibration.rs:82-117`. The 0.25 balance is an interpolation point, not a shipped constant. The 0.10 balance is both a weighting example and a fitting-module operating fixture (`test:unit:synthetic-convergence-kappa-2`); `packages/assayer/src/risk/calibration.rs:719-747`.

For each cell, five thousand trials draw one stationary stream. The first fit reads a full `B`-row window; the second reads the full window `C` rows later, so the windows overlap by `B-C` rows when `C < B` and are disjoint when `C >= B`. Within a window the newest row has weight one and each older row receives another factor of `0.9998`, matching the fitter's newest-first scan (`alg:platt:fitting`); `packages/assayer/src/risk/calibration.rs:341-455`. The fitted root is solved directly from the same weighted cross-entropy stationarity equation. The reported random variable is

$$D = |\log\kappa_{t+C} - \log\kappa_t|.$$

The seed is `0xD21F7`, as in the prior report. The extension enumerates balances in the order 0.10, 0.25, 0.50 and shares each balance's draw across its buffer and cadence cells; sharing reduces Monte Carlo noise in comparisons between cells without making their marginal distributions different. No one of these cells is asserted to be a deployment forecast.

## Movement quantiles across the operating range · `sec:calibration:platt-movement-quantiles`

First, the prior experiment reproduces exactly under its original seed and draw order. Its headline percentages were in **absolute κ**, so the relative-log columns are new and must not be silently substituted for them:

| Stationary comparison | $P(\lvert Δκ \rvert>0.1)$ | $D_{50}$ | $D_{90}$ | $D_{95}$ | $D_{99}$ | $P(D>\log 1.04)$ | $P(D>\log 1.05)$ | $P(D>\log 1.07)$ |
| --- | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| Early nested, 200 then 400 rows | 63.32% | 0.071325 | 0.172634 | 0.203637 | 0.276651 | 71.60% | 65.06% | 51.94% |
| Full 2,000-row buffer, 200-row advance | 1.32% | 0.014136 | 0.034283 | 0.040777 | 0.051627 | 5.94% | 1.70% | 0.08% |

The old 63.32% and 1.32% are therefore reproduced, as are the old absolute-κ medians `0.140476` and `0.028157` and 95th percentiles `0.414458` and `0.080375`. Early occupancy is not a small version of the full-buffer distribution: at `log(1.07)`, about half the stationary early comparisons cross while fewer than one in a thousand of this balanced full-buffer sample cross. Any named threshold would at minimum have to know whether the buffer is still growing.

Across all nine cadence-and-balance cells at each full-buffer size, the quantiles occupy these ranges. A range endpoint is a cell quantile, not a confidence interval:

| Buffer `B` | Range of median $D$ | Range of $D_{95}$ | Range of $D_{99}$ |
| ---: | ---: | ---: | ---: |
| 100 | 0.1412–0.3324 | 0.4159–1.0150 | 0.5460–1.4410 |
| 200 | 0.0694–0.2283 | 0.2060–0.6874 | 0.2672–0.9203 |
| 500 | 0.0280–0.0894 | 0.0821–0.2701 | 0.1087–0.3550 |
| 1,000 | 0.0136–0.0458 | 0.0403–0.1334 | 0.0528–0.1775 |
| 2,000 | 0.0069–0.0230 | 0.0202–0.0681 | 0.0268–0.0897 |

The low endpoint in every row is the balanced population with a fifty-label advance. The high endpoint is the ten-percent-positive population at the largest non-equivalent advance; for the hundred-row buffer, advances of one hundred and two hundred are both disjoint and differ only by Monte Carlo noise. More overlap suppresses movement. Rarer positives enlarge it despite balanced expected gradient mass, because the mass arrives in fewer, heavier rows.

The default two-thousand-row buffer makes the cadence and balance effects visible without the buffer-size effect:

| Positive balance | Advance | $D_{50}$ | $D_{95}$ | $D_{99}$ |
| ---: | ---: | ---: | ---: | ---: |
| 10% | 50 | 0.011327 | 0.035522 | 0.049296 |
| 10% | 100 | 0.016663 | 0.048806 | 0.067869 |
| 10% | 200 | 0.023015 | 0.068149 | 0.089725 |
| 25% | 50 | 0.008038 | 0.023721 | 0.031796 |
| 25% | 100 | 0.011430 | 0.033638 | 0.044930 |
| 25% | 200 | 0.016434 | 0.047870 | 0.062613 |
| 50% | 50 | 0.006895 | 0.020218 | 0.026834 |
| 50% | 100 | 0.010269 | 0.028511 | 0.038290 |
| 50% | 200 | 0.013938 | 0.041242 | 0.053696 |

The default balanced cell differs slightly from the independently reproduced prior row because it occurs later in the extension's seeded stream. The spread is ordinary five-thousand-trial Monte Carlo variation; it does not change a candidate decision.

## Candidate thresholds: false alarms and detection lag · `sec:calibration:platt-threshold-tradeoffs`

The candidates and their deterministic stimulus margins are:

| Relative factor | Threshold `T = log(factor)` | Margin above four percent | Margin below ten percent |
| ---: | ---: | ---: | ---: |
| 1.04 | 0.039220713 | 0 | 0.056089467 |
| 1.05 | 0.048790164 | 0.009569451 | 0.046520016 |
| 1.06 | 0.058268908 | 0.019048195 | 0.037041272 |
| **1.07** | **0.067658648** | **0.028437935** | **0.027651531** |
| 1.08 | 0.076961041 | 0.037740328 | 0.018349139 |
| 1.10 | 0.095310180 | 0.056089467 | 0 |

The exact equal-log-margin point is the logarithm of the geometric mean, `sqrt(1.04 × 1.10) = 1.069579357`, or `0.067265446` in log units. Thus `log(1.07)` is the only rounded candidate with real, nearly symmetric margin. `log(1.05)` is much closer to the admitted stimulus. `log(1.04)` has no lower margin, while `log(1.10)` cannot reject the ten-percent stimulus under a strict exceedance rule and is included only as an endpoint control.

For false alarms, the following is the **largest** stationary per-comparison exceedance frequency found among the three cadences and three balances at each full-buffer size. It is an operating-grid envelope, not a universal bound:

| Candidate | `B=100` | `B=200` | `B=500` | `B=1,000` | `B=2,000` |
| --- | ---: | ---: | ---: | ---: | ---: |
| `log(1.04)` | 93.56% | 91.24% | 75.70% | 55.78% | 25.72% |
| `log(1.05)` | 91.84% | 89.14% | 70.52% | 47.52% | 15.70% |
| `log(1.06)` | 90.58% | 87.04% | 65.22% | 39.46% | 9.38% |
| **`log(1.07)`** | **88.90%** | **84.76%** | **60.14%** | **32.50%** | **5.12%** |
| `log(1.08)` | 87.80% | 82.44% | 55.78% | 25.90% | 2.40% |
| `log(1.10)` control | 84.80% | 78.62% | 47.04% | 16.10% | 0.66% |

At the default two-hundred-label cadence and balanced ladder, `log(1.07)` costs 82.00%, 74.90%, 40.46%, 10.10% and 0.10% respectively as the buffer grows through those five sizes. At the default full capacity, changing only the balance from one half to one tenth raises that estimate from 0.10% to 5.12%. At the early `200 → 400` comparison it is 51.94%. A threshold value without buffer occupancy, advance and class-balance conditions consequently has no stable false-alarm meaning.

### The authored comparison · `sec:calibration:platt-authored-comparison`

For the fitting-module fixture, displacement is applied to an entire authored buffer. Scale equivariance makes the comparison exact (`test:unit:overlap-ladder-drift-bound-admits-and-rejects`). Every candidate at or above `log(1.04)` admits the four-percent movement. Every candidate strictly below `log(1.10)` rejects the ten-percent movement on the next comparison. There is no multi-refit lag in that fixture: it supplies two endpoint decisions, not a streaming detector.

### A persistent shift entering the rolling buffer · `sec:calibration:platt-buffered-shift`

To expose the cadence interaction, a separate deterministic calculation starts from a population-full stationary buffer, changes every subsequent score by the authored factor, and refits after each periodic advance. It applies the same recency weights and compares each fit only with its immediate predecessor. The four-percent signal never strictly crosses any candidate: that is the intended admission. For the ten-percent signal at the default two-hundred-label cadence:

| Buffer `B` | Largest adjacent four-percent move | Largest adjacent ten-percent move | Ten-percent candidates detected on next refit |
| ---: | ---: | ---: | --- |
| 100 | 0.039221 | 0.095310 | `log(1.04)` through `log(1.08)` |
| 200 | 0.039221 | 0.095310 | `log(1.04)` through `log(1.08)` |
| 500 | 0.016495 | 0.041253 | `log(1.04)` only |
| 1,000 | 0.008719 | 0.022010 | none |
| 2,000 | 0.004810 | 0.012199 | none |

An endpoint equality is treated according to the authored contract: four percent is admitted and a threshold equal to ten percent does not reject ten percent. Whenever the deterministic ten-percent signal crosses, its first increment is the largest and detection occurs one refit, two hundred labels, after onset. Otherwise no later adjacent comparison crosses before the buffer has completely turned over, so lag is censored as **not detected by this mechanism**. At the default buffer, decreasing the advance to one hundred or fifty labels lowers the largest ten-percent increment further, to `0.006190` or `0.003118`.

This is not a defect in the production drift integration and must not be read as one. The proposed test threshold compares adjacent fitted sharpness values; the production reset consumes its separately named `0.1` log-ratio for accumulator validity (`alg:platt:drift-integration`). Sampling excursions can make a live adjacent comparison cross even where the expected shift does not, but the stationary table shows why such an event cannot be credited as detection without a false-alarm model.

## Recommendation, and the decision it awaits · `sec:calibration:platt-owner-recommendation`

**Recommendation: retain the two stimulus decisions and do not name one Platt movement threshold.** They pin the following monotone contract and no more:

$$
\log(1.04) \leq T < \log(1.10)
$$

would preserve the authored admission and rejection if an implementation later requires a cutoff `T`, but every point in that interval is consistent with the two tests. The stimuli do not choose `log(1.05)`, do not choose the symmetric midpoint, and do not promise any stationary false-alarm probability. The successive deterministic pipeline fixture can keep asserting repeatability without pretending its zero movement sizes a population tolerance.

If one complete **fixture-only** oracle is required, use `log(1.07)` and say why: it is the rounded geometric midpoint and leaves about `0.028` log units to either authored stimulus. Do not call it a stationary or deployment threshold. Even in this narrow role it should remain separate from `delta_cal_threshold`.

A deployment-level guarantee requires two host-supplied inputs the corpus still lacks: a target per-comparison false-alarm probability and a sampling model. That model must at least condition on populated buffer or effective sample size, rows advanced since the prior fit, current class balance and importance-weight process, and whether the buffer is growing. Serial dependence, score support and trigger type belong in it if the guarantee is meant for real traffic. The threshold should then be the corresponding conditional movement quantile; there is no evidence that one constant will serve every cell.

The recommendation is falsified if a host supplies those inputs and a supported operating region whose conditional quantiles lie with defensible margin between the two stimuli. It is also displaced, for the narrower test-only question, if a total fixture oracle is explicitly valued more than leaving the interval unspecified; `log(1.07)` is then the supported name. Until one of those conditions holds, naming a point would turn authored spacing into claimed evidence.

## Premise audit · `sec:calibration:platt-premise-audit`

- **Verified at the base commit:** `platt_kappa = 0.1` is still absolute and has exactly the three comparison-function consumers named in the decision surface. This is base-commit evidence only; a sibling code lane is expected to implement the relative-log ruling.
- **Corrected premise:** the prior report's phrase “the two Platt tolerance consumers” collapses the flat-score and displacement unit scenarios into one fitting-module locus. The direct base-commit read census is three comparison functions. The extra function is explicitly vacuous, so the correction changes the consumption inventory but not the sensitivity result.
- **Verified and kept separate:** production already computes a relative log-sharpness movement, but its `0.1` threshold governs drift-accumulator reset. It is not the test tolerance studied here.
- **Reproduced exactly:** with seed `0xD21F7`, the prior study's absolute-κ exceedance figures are 63.32% for the nested `200 → 400` comparison and 1.32% for the full `2,000`-row buffer advanced by 200. Their old medians and 95th percentiles also reproduce to the printed digits.
- **Requalified:** those 63.32% and 1.32% figures compare `|Δκ|` with the retired absolute `0.1`. In relative log-sharpness, `log(1.05)` is crossed by 65.06% and 1.70% of the same samples. The two statements answer different questions.
- **Verified:** scaling every score by a factor scales the population-optimal κ by that factor. Thus the authored movements are exactly `log(1.04)` and `log(1.10)`, apart from fitter resolution.
- **Refuted:** the existing stimuli warrant five percent or any other named point. They warrant one admission, one rejection and the interval between.
- **Refuted as an unconditional claim:** a named threshold has one stationary false-alarm rate. Buffer occupancy, overlap and class balance move the distribution by orders of magnitude within configuration-shaped values.
- **Qualified:** class-balance sensitivity uses the converged equal-halves weights at the true stationary rate. A live tracker adds estimation and adaptation noise; this experiment intentionally isolates finite-buffer movement and does not prove that live movement is smaller.
- **Qualified:** IID ladder rows omit serial clustering, score-support change, tracker transients, lifecycle refits and deployment-specific traffic. They are a seeded sensitivity surface, not a product probability forecast.

## Method and limits · `sec:calibration:platt-method-limits`

All computation used Python's standard library, with no retained script, compiler or network. The old-result reproduction took `16.71 s`; the extended operating grid took `59.69 s`; the expanded old-result quantiles took `14.95 s`; the deterministic replacement calculation took `0.17 s`; and a separate `33.29 s` audit found no search-bound fit among five thousand single-window draws at any studied buffer and balance. The logarithms and stimulus margins took `0.02 s`.

Each simulated probability is an empirical fraction of five thousand trials. At worst its binomial standard error is about 0.7 percentage points; zero observations means zero in that sample, not impossibility. The direct root solve removes golden-section quantisation, whose declared log resolution is `10^-4`, hundreds of times smaller than the lowest candidate. No empirical deployment trace and no owner-selected false-alarm target exists in the corpus, which is why this study can reject an unconditional threshold warrant but cannot manufacture a deployment threshold in its place.
