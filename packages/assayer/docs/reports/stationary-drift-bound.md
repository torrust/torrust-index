# The stationary-drift bounds are two different bounds · `rep:vector:standardisation-and-platt-bound-separation`

This report is evidence for a decision, not the decision. It was commissioned as a sensitivity study of the re-standardisation variance-drift bound and of a standing finding that calls a stationary drift allowance about five hundred six search brackets wide. The first finding of the study is that those are two different quantities. The specification passage bounds a feature coordinate's movement between assessment and label. The search-bracket finding measures a test-only tolerance on the fitted Platt sharpness. Both happen to contain the decimal `0.1`; they have different units, clocks, mechanisms and authorities.

The headline is therefore two-part. **The specification's `0.1 Δk` variance term is conservative, but it can bind on clip-boundary paths and stationarity does not make such paths impossible. The five-hundred-six arithmetic reproduces for the Platt fixture, not for that term, and stationarity does not prevent finite refit samples from crossing the Platt tolerance.** The recommendation, sized below, is to make the Platt scenario displacement-anchored and relative in `log κ`, while leaving the re-standardisation bound independent and composing its eventual repair with the ratified cold-ramp delta. The decision remains open.

The report changes no specification, record, test or implementation. As a prose carrier it cites existing heads and mints none, under the label calculus.

## The cited bound, in its own units · `sec:vector:drift-standardisation-bound`

The pending entry stores raw features. Assessment standardises a raw coordinate against the immutable snapshot then current; label handling reconstructs that same raw coordinate against statistics that may have advanced in the meantime (`def:runtime:pending-entry`); `packages/assayer/docs/spec/data-model-features.md:674-678`. The coordinate on each side is dimensionless,

$$\hat\phi_j = \frac{\phi_j-\bar\mu_j}{\sqrt{\bar v_j}+\varepsilon},$$

and the mismatch is the absolute difference between its assessment-time and label-time values. The transform and its stability epsilon are stated at `packages/assayer/docs/spec/data-model-features.md:529-543` and implemented at `packages/assayer/src/feature/standardisation.rs:211-236`.

The horizon is **`Δk` intervening processed labels**, not assessments, requests or elapsed time. Assessments read a fixed published coordinate system and do not advance its continuing EWMA; each processed label advances the working means and variances once, after the models have consumed the old coordinates, then publishes (`req:standardisation:timing`); `packages/assayer/docs/spec/data-model-features.md:545-589` and `packages/assayer/src/owner/label_path.rs:891-904`. A deployment that receives no labels therefore accumulates no continuing standardisation movement even if requests continue. A label rate changes only how much wall time a given `Δk` occupies.

The passage decomposes the first-order coordinate movement into mean and variance terms (`bound:standardisation:restandardisation`): `packages/assayer/docs/spec/data-model-features.md:674-691`.

$$
|\Delta\hat\phi_j|
\mathrel{\lesssim}
\frac{|\Delta\bar\mu_j|}{\sqrt{\bar v_j}}
+
\frac{|\phi_j-\bar\mu_j|\,|\Delta\bar v_j|}
     {2\bar v_j^{3/2}}.
$$

The less-than-or-approximately relation above is deliberate. The specification prints an inequality, but the variance term is the derivative of inverse square root evaluated at one variance. Over a finite change, the exact term uses the difference of two reciprocal square roots. The following arithmetic is the passage's own first-order sizing, not a promotion of it to a global theorem.

### Every factor in the `0.1` coefficient · `sec:vector:drift-coefficient-factors`

Let `a = 1 - γ_std`, let `c` be the update clip width in current standard deviations, let `d` be the clipped update's deviation in those units, and let `z` be the stored raw coordinate's deviation. The shipped values are `γ_std = 0.9998`, `c = 10`, `v_floor = 10^-4` and `ε = 10^-8` (`tab:config:standardisation`); `packages/assayer/docs/spec/reference-configuration.md:207-225` and `packages/assayer/src/feature/standardisation.rs:179-205`.

The implementation updates from the old mean exactly as the prose says:

$$
\Delta\bar\mu = a(x_c-\bar\mu), \qquad
\bar v' = (1-a)\bar v+a(x_c-\bar\mu)^2.
$$

Clipping, both updates and the variance floor are visible together at `packages/assayer/src/feature/standardisation.rs:245-296`. Dividing by the old standard deviation gives the one-label factors:

| Factor | Shipped calculation | Dimensionless movement per label |
| --- | ---: | ---: |
| EWMA complement | $a=1-0.9998$ | $0.0002$ |
| Mean, clip boundary | $ac=0.0002\times10$ | $0.002$ |
| Relative variance, exact EWMA step at the boundary | $a(c^2-1)=0.0002\times99$ | $0.0198$ |
| Variance coordinate term, exact step | $(z/2)a(c^2-1)$ at $z=c=10$ | $0.099$ |
| Variance coordinate term, passage's coarse rounding | $(z/2)ac^2$ at $z=c=10$ | $0.100$ |

The advertised variance coefficient is thus reproduced: it is five from the inverse-square-root derivative, one hundred from the squared ten-standard- deviation clip boundary, and `0.0002` from the EWMA complement. It is **not** a count of search brackets. Adding the passage's `0.002` mean coefficient yields `0.102` per label under its coarse ingredients, printed as about `0.1`; retaining the EWMA's `-v` term yields `0.101`. Across fifty intervening labels those give `5.1` and `5.05`, respectively, reproducing the stated `5.1` ceiling.

For the passage's typical case, putting both the stored coordinate and the variance-driving observations within two standard deviations gives the coarse variance coefficient

$$
\frac{2}{2}\times0.0002\times2^2=0.0008.
$$

Keeping the global clip-boundary mean allowance gives `50 × (0.002 + 0.0008) = 0.14`, exactly the stated typical figure. If both mean and variance inputs are actually restricted to two standard deviations and the retention term is kept, the corresponding first-order total is only `50 × (0.0004 + 0.0006) = 0.05`. The stated `0.14` is consequently an envelope assembled from a global mean extreme and a typical variance case, not the expected movement of a stationary feature.

## What the five-hundred-six calculation actually verifies · `sec:vector:drift-platt-calculation`

The five-hundred-six figure belongs to Platt calibration. The harness declares an absolute `platt_kappa` tolerance of `0.1`, independently of the standardisation tolerance of `0.14` (`tab:assayer:harness-scenario-tolerances`); `packages/assayer/docs/plans/testing_plan.md:651-669` and `packages/assayer/src/testing/tolerances.rs:70-88`. The fitter searches in `log κ` to a declared width below `τ = 10^-4` (`alg:platt:fitting`); `packages/assayer/docs/spec/core-models-risk.md:379-395` and `packages/assayer/src/risk/calibration.rs:88-119`.

The shipped overlap ladder has the hand-derived optimum `κ* = 1.976282462220253`; its reciprocal is `0.5060005435035591`. The derivation, strict convexity argument and constant are at `packages/assayer/src/risk/calibration.rs:630-659`. At that optimum, one declared log-bracket quantum maps to κ-space as

$$q_\kappa=\kappa^*(e^\tau-1)=0.000197638127964.$$

Therefore

$$\frac{0.1}{q_\kappa}=505.975243898.$$

The first-order mapping uses `q_κ ≈ κ*τ` and gives

$$\frac{0.1}{\kappa^*\tau}=506.000543504,$$

which reproduces the standing figure. Golden section starts with log-width `log(100)-log(0.01)=9.210340371976184`; twenty-four contractions make the first checked width below tolerance, `0.000088832587844`. Against that actual last bracket the same absolute tolerance is about `569.6` local bracket widths. The five-hundred-six statement is thus a valid, conservative comparison to the **declared maximum local resolution** at this fixture's optimum. It is not an exact count of the algorithm's final brackets, and it has no dimensional route to the feature-coordinate mismatch.

## Why stationarity does not prove that the Platt tolerance never binds · `sec:vector:drift-stationarity-limits`

The pipeline scenario repeats one deterministic forty-row ladder. Its first two refits end on whole repetitions in the same order, so exponential recency weighting multiplies each rung's accumulated weight by a common factor and the optimum is bit-identical. That is a property of the fixture geometry, not of stationarity in general (`test:crate:platt-sharpness-holds-between-successive-refits`); `packages/assayer/src/tests/label_pipeline.rs:1355-1435`.

A stationary deployment fixes the distribution; it does not make two finite samples identical. As a reachability check, a standard-library Python study sampled IID rows from the shipped ladder's fixed score frequencies and fixed per-rung outcome probabilities, applied the shipped `0.9998` recency weights, and solved the same stationary equation. A fixed seed was used and no script was retained. Across five thousand trials:

| Stationary comparison | Fraction with $\lvert \Delta\kappa \rvert>0.1$ | Median | 95th percentile |
| --- | ---: | ---: | ---: |
| First two nested samples, 200 then 400 rows | 0.6332 | 0.1405 | 0.4145 |
| Full 2,000-row rolling buffer, 200-row advance | 0.0132 | 0.0281 | 0.0804 |

These are sensitivity results for the ladder, not probability forecasts for an unknown deployment. They are enough to refute **never**: ordinary stationary sampling crosses the current tolerance with positive probability, often during the first pair of refits and occasionally after the buffer is full. A local Fisher-information calculation agrees in scale: the approximate standard deviation of the early `200 → 400` difference is `0.2020`, and of a full-buffer two-hundred-row advance is `0.0408`. The deterministic scenario measures repeatability of its authored sequence; it cannot certify stationary-population drift.

## Sensitivity of the re-standardisation envelope · `sec:vector:drift-standardisation-sensitivity`

There is no policy threshold in the bound: no label is rejected and no reset is triggered when the right-hand side reaches a number. “Starts binding” therefore needs a comparison scale. This study uses one standardised coordinate unit as the point where mismatch is no longer small relative to the feature scale. That is a sensitivity marker, not a new requirement.

### Label rate changes the calendar, not the bound · `sec:vector:drift-label-rate`

The specification's operating profiles span fifty, two hundred and one thousand labels a day (`tab:warmup:stages`); `packages/assayer/docs/spec/analysis-warmup.md:130-145`. The reference pending latency is one hour and entries expire after twenty-four hours (`tab:config:pending-buffer`); `packages/assayer/docs/spec/reference-configuration.md:255-283`. At those label rates the shipped first-order envelope reads:

| Labels/day | Intervening labels in one hour | Typical envelope in one hour | Clip-boundary envelope in one hour | Wall time containing 50 labels |
| ---: | ---: | ---: | ---: | ---: |
| 50 | 2.083 | 0.0058 | 0.2125 | 24 h |
| 200 | 8.333 | 0.0233 | 0.8500 | 6 h |
| 1,000 | 41.667 | 0.1167 | 4.2500 | 1.2 h |

The table uses the passage's `0.0028` typical total and `0.102` unrounded coarse clip total per label. Thus the fifty-label example is just beyond the high profile's reference median latency, but within the pending horizon for every profile. It is not the reference latency translated into labels.

On the one-coordinate-unit comparison, the mixed typical envelope reaches one after about `1/0.0028 = 357` intervening labels. The exact one-step clip linearisation reaches one after about `1/0.101 = 9.90` labels. At the three profile rates those horizons are respectively about `171`, `42.9` and `8.57` hours for the typical envelope, and `4.75`, `1.19` and `0.238` hours for the clip envelope. A typical mismatch cannot reach one before expiry at the low or medium profile under this linear envelope; a sustained clip-boundary path can do so under all three. Nothing in stationarity alone excludes such a finite sample path.

### The EWMA rate scales the envelope linearly · `sec:vector:drift-ewma-rate`

Holding the fifty-label horizon and clip width fixed gives:

| `γ_std` | Typical passage envelope | Clip-boundary passage envelope |
| ---: | ---: | ---: |
| 0.99900 | 0.700 | 25.500 |
| 0.99950 | 0.350 | 12.750 |
| **0.99980 shipped** | **0.140** | **5.100** |
| 0.99990 | 0.070 | 2.550 |
| 0.99995 | 0.035 | 1.275 |

Every entry is proportional to `1 - γ_std`. The shipped rate's half-life is `log(1/2)/log(0.9998) = 3465.389` processed labels, which explains why ordinary short-window movement is small without turning the clip envelope into a stationarity theorem.

### Displacement magnitude supplies the cubic sensitivity · `sec:vector:drift-displacement-sensitivity`

For a one-step local comparison where the stored coordinate magnitude and the intervening clipped deviation have the same magnitude `d = |z|`, retaining the EWMA's old-variance term gives

$$B_1(d)=a\left[d+\frac{d(d^2-1)}{2}\right].$$

| Magnitude in current standard deviations | One-label movement | Labels for a one-unit linear envelope | Fifty-label envelope |
| ---: | ---: | ---: | ---: |
| 1 | 0.0002 | 5,000 | 0.010 |
| 2 | 0.0010 | 1,000 | 0.050 |
| 4 | 0.0068 | 147 | 0.340 |
| 6 | 0.0222 | 45.0 | 1.110 |
| 8 | 0.0520 | 19.2 | 2.600 |
| 10 | 0.1010 | 9.90 | 5.050 |

The variance contribution grows cubically because the update squares one displacement and the coordinate derivative contributes another. The large number is therefore doing real work for an extreme path; it is not informative about a well-centred one. A stationary light-tailed distribution makes a long clip-boundary run unlikely, but the specification assumes no tail law that makes it unreachable: any stationary law with nonzero mass near the boundary admits a finite boundary streak with positive probability. Its own prose expressly contemplates the conjunction of an extreme feature and extreme variance evolution at `packages/assayer/docs/spec/data-model-features.md:688-691`.

The current crate scenario exercises only the inside of the envelope. It drives fifty near-stationary uniform observations, then asserts that the reconstructed coordinate is below `0.14`; it supplies no deliberately breaching path (`test:crate:restandardise-mismatch-bounded-at-fifty-labels`); `packages/assayer/src/tests/standardisation.rs:300-385`. That is useful containment coverage, but it cannot establish where the bound starts deciding.

## Sensitivity of the separate Platt tolerance · `sec:vector:drift-platt-sensitivity`

Platt's label rate likewise changes calendar time only. Its periodic cadence is two hundred labels and its buffer holds two thousand (`tab:platt:refit-cadence`); `packages/assayer/docs/spec/core-models-risk.md:350-374` and `packages/assayer/docs/spec/core-models-risk.md:397-425`.

| Labels/day | Periodic refit interval | Full-buffer span |
| ---: | ---: | ---: |
| 50 | 4 days | 40 days |
| 200 | 1 day | 10 days |
| 1,000 | 4.8 hours | 2 days |

At the shipped calibration recency rate of `0.9998`, two thousand weighted rows have an effective independent-weight count of about `1973.75`. Moving the rate to `0.999`, `0.9995`, `0.9999` or `1` changes that count to about `1522.85`, `1848.40`, `1993.36` or `2000`. This alters sampling movement, but not the search resolution or the absolute `0.1` harness tolerance. The three are independent controls.

The search-bracket comparison is directly inverse in the configured log tolerance:

| Declared log tolerance `τ` | Local κ width at `κ*` | Widths inside absolute `0.1` |
| ---: | ---: | ---: |
| $10^{-3}$ | 0.00197727 | 50.57 |
| **$10^{-4}$ shipped** | **0.000197638** | **505.98** |
| $10^{-5}$ | 0.0000197629 | 5,059.98 |

Changing optimizer resolution by a decade changes the headline width count by a decade without changing the workload or the allowed κ displacement. That is why bracket width cannot, by itself, warrant a deployment-stability tolerance.

The existing scale-equivariant fixture gives the complementary displacement surface. Multiplying every score by `1+d` multiplies its optimum κ by the same factor (`test:unit:overlap-ladder-drift-bound-admits-and-rejects`); `packages/assayer/src/risk/calibration.rs:1007-1062`.

| Score-scale displacement | κ movement at `κ*` | Declared-resolution widths |
| ---: | ---: | ---: |
| 0.01% | 0.0001976 | 1.00 |
| 0.1% | 0.0019763 | 10.00 |
| 1% | 0.0197628 | 100.00 |
| 4% | 0.0790513 | 399.98 |
| 5.06% | 0.100000 | 505.97 |
| 10% | 0.197628 | 999.95 |

The current absolute tolerance starts rejecting this fixture at about `5.06%` scale displacement. The unit scenario's four-percent admission and ten-percent rejection therefore bracket it correctly. That result is a meaningful displacement test even though the deterministic stationary-pipeline assertion does not generalise.

## The two candidate re-anchorings · `sec:vector:drift-reanchoring-options`

The candidate names are dimensionally meaningful for the Platt tolerance. They are not two derivations of the re-standardisation `0.1 Δk`; the corpus provides no search bracket for a feature-coordinate EWMA. Applying either Platt formula directly to that passage would be a category error.

### Resolution-anchored · `sec:vector:drift-resolution-anchor`

The precise relative formulation is

$$|\Delta\log\kappa|\leq m\tau,$$

or, at a starting sharpness κ, an upward absolute allowance `κ(e^(mτ)-1)`. The pure resolution candidate takes `m = 1`; at the ladder optimum it allows `0.000197638`. The current `0.1` is equivalent to about `m = 506` only at this κ and this tolerance.

This candidate binds numerical repeatability: the same authored rows processed by the same deterministic fitter should not move by more than its resolving power. It does not bind statistical stability. In the stationary simulation above, the median early movement is about seven hundred eleven one-bracket allowances and the full-buffer median about one hundred forty-two. Treating one bracket as a stationary-population ceiling would reject ordinary sampling movement.

Its attraction is small scope. A test can compare log κ against `golden_tolerance`; no production behaviour changes and the core specification needs no edit. Its failures are coupling to an optimizer implementation detail, moving whenever that detail is tuned, and saying nothing about a deployment's calibration quality. If promoted from deterministic repeatability to a product guarantee, it would need a sampling model the specification does not have.

### Displacement-anchored · `sec:vector:drift-displacement-anchor`

For Platt, use the coordinate the fitter and shipped drift integration already use:

$$|\Delta\log\kappa|\leq\log(1+d_\text{admit}).$$

Taking `d_admit = 0.05` gives `0.048790164` in log κ. The existing four-percent stimulus moves by `log(1.04) = 0.039220713` and passes; the ten-percent stimulus moves by `log(1.10) = 0.095310180` and fails. Unlike an absolute κ allowance, that contract keeps the same meaning at κ of `0.2`, `2` or `20`. It is also the rounded relative equivalent of the current absolute tolerance at the fixture optimum, `0.1/κ* = 0.050600054`, so it preserves the present decision while removing its scale dependence. It remains separate from the production `delta_cal_threshold = 0.1`, which already measures a log ratio and governs drift integration rather than test tolerance (`alg:platt:drift-integration`); `packages/assayer/src/risk/calibration.rs:379-421` and `packages/assayer/src/owner/label_path.rs:865-888`.

For in-service re-standardisation, the dimensionally corresponding displacement anchor is the path sensitivity

$$
B_\text{disp} =
\sum_{t=1}^{\Delta k}
(1-\gamma_\text{std})
\left(
|d_t|+\frac{|z_t|}{2}|d_t^2-1|
\right),
$$

where `d_t` is the clipped observation and `z_t` the stored raw feature, both expressed in the pre-update coordinate system at step `t`. This says what actually makes the bound large. Substituting the configured clip gives the coarse global envelope; substituting an operating displacement gives the bite tables above. A finite-horizon guarantee should compute the exact difference of the endpoint transforms rather than call this first-order sum exact.

The specification churn is local: replace the two unexplained rate sentences in the mismatch passage with the parameterised in-service sensitivity and retain the clip and two-standard-deviation evaluations as examples. No production code must change if this remains an analytic bound. Making it an enforced or reported per-pending-entry quantity is a different, high-churn proposal: label time would need the assessment coordinate state or a sufficient movement history, neither of which the raw pending vector presently carries. The test-plan wording and the two Platt tolerance consumers would separately move from absolute κ to log κ in `packages/assayer/src/testing/tolerances.rs`, `packages/assayer/src/risk/calibration.rs` and `packages/assayer/src/tests/label_pipeline.rs`.

The displacement candidate can still fail by choosing an operational magnitude with no warrant in the corpus. Five per cent is sized from the existing admitted and rejected stimuli; it is not evidence that five per cent is the correct product limit. Stationary finite-sample movement can also cross it. A deployment-level false-alarm guarantee would require buffer-size and distribution assumptions, not another deterministic tolerance.

## Recommendation and its falsifier · `sec:vector:drift-recommendation-falsifier`

**Recommendation: displacement-anchored.** For the mismatch passage, state the in-service sensitivity in `d_t`, `z_t` and `1 - γ_std`, keep `0.1 Δk` only as the configured clip-boundary evaluation, and describe it as a first-order envelope rather than a stationary-deployment tolerance. For Platt, compare relative movement in log κ and keep the explicit four-percent admitted and ten-percent rejected stimuli; use `log(1.05)` only if one named threshold between them is wanted. Rename the stationary pipeline claim to the narrower fact it proves: deterministic repeatability of the periodic ladder.

This recommendation is falsified if the required contract is an **a-priori, configuration-only ceiling** that can be evaluated without observing or retaining displacement. In that case the displacement variables are unavailable by design, and the clip-boundary configuration envelope is the right conservative form. It is also falsified as a deployment-level Platt guarantee unless a host supplies a target false-alarm probability and a sampling model; the seeded study shows that stationarity alone cannot carry that meaning.

The resolution candidate remains useful as a separate optimizer-repeatability oracle. It should not be called the stationary-drift bound and should not share the displacement threshold.

## Composition with ratified delta D5 · `sec:vector:drift-ramp-composition`

Delta D5 will split the current mismatch passage by standardisation phase. Its ratified text says the finite prior-mass ramp governs `Transitioning`, while the existing `Δk` bound applies unchanged in `InService`; `packages/assayer/docs/reports/uncertainty-step-spec-changes.md:253-283`.

Apply both decisions in one passage as follows:

1. Keep D5's `Transitioning` branch unchanged: one accepted observation replaces exactly `1/N_init` of base mass, and phase, count and snapshot version identify the coordinate states. Do not apply the in-service EWMA sum during that phase.
2. In D5's `InService` branch, replace “the existing Δk label-time bound applies unchanged” with the displacement-parameterised sensitivity above. Evaluate the configured `γ_std = 0.9998`, clip width ten and illustrative two-standard- deviation case there, and mark the finite-change caveat.
3. Retain D5's leverage sentence. Coordinate mismatch and posterior leverage are different layers: the first sizes the changed input, the second limits what one later label update can do with it.

This composition neither double-counts nor leaves a clock gap. Transition movement is bounded by ramp mass; continuing movement is label-indexed and bounded by its actual displacement geometry. The Platt recommendation is orthogonal and must not be transplanted into D5: its 506 figure, optimizer bracket and κ units do not belong in the standardisation chapter.

## Premise audit · `sec:vector:drift-premise-audit`

- **Verified:** the re-standardisation mechanism is the mismatch between the assessment-time scoring coordinate and the label-time model-update coordinate for one stored raw feature. Its horizon is intervening processed labels.
- **Verified:** `0.1 Δk`, `5.1` at fifty labels and the typical `0.14` all reproduce from the shipped `0.9998` rate and clip width ten, subject to the stated first-order and coarse-rounding qualifications.
- **Verified, but for a different mechanism:** `506.0005` reproduces from the absolute Platt harness tolerance, ladder optimum and declared golden-search log tolerance. The exact exponential mapping gives `505.9752`; the actual final golden bracket makes the ratio about `569.6`.
- **Refuted:** the specification's variance-drift coefficient does not translate into Platt search brackets. The shared decimal `0.1` is coincidence, not a conversion.
- **Refuted:** a stationary deployment can never cross the Platt tolerance. Stationarity fixes a population, while finite refit samples move. The deterministic pipeline fixture suppresses that movement by construction.
- **Refuted:** stationarity makes the re-standardisation clip envelope unreachable. It makes sustained extremes improbable under an additional tail model; it does not make finite extreme paths impossible, and the specification provides no excluding tail model.
- **Qualified:** the specification prints a finite-change inequality but sizes its variance term with a local derivative. Its `0.14` typical result also combines the global mean extreme with a two-standard-deviation variance case. Both are useful conservative arithmetic and should not be described as exact stationary expectations.
- **Qualified:** “never binds” has no native meaning for the mismatch passage, because it is an envelope and not an acceptance gate. On the explicit one-standard-unit comparison it becomes material after about ten extreme intervening labels or about three hundred fifty-seven labels under the mixed typical envelope.

## Method and limits · `sec:vector:drift-method-limits`

All arithmetic used Python's standard library; no retained script, external data, compiler or network was used. Closed forms were cross-checked against the shipped constants. The stochastic reachability exercise used seed `0xD21F7` and is a sensitivity probe of an IID version of the authored ladder, not a claim that real deployment rows are IID or ladder-distributed. No empirical deployment trace exists in the corpus, so the report can refute impossibility and size the candidate surfaces, but cannot choose a product false-alarm rate.
