# The synchronisation error is one model's reading, and the cadence does not move it · `rep:assayer:synchronisation-drift-study`

This report is evidence for a decision, not the decision. It was commissioned to measure how far the rank-one-maintained inverse drifts from the recomputed precision matrix at a feature width near a thousand, what bounds that drift, and whether the recomputation interval is the knob that bounds it. The finding left open by the factorisation lane was two readings of the synchronisation error at one label count — `4.8e5` in one run and `2.8e1` in another — against a documented abnormal magnitude near `1e-3`, together with one red run in six of the eight-minute reproduction whose failing assertion was not captured.

The headline is four-part, and none of the four is the drift the commission assumed. **The error is not spread across the model triple: it belongs entirely to the per-outcome-axis model, the only model carrying any dimension at the replenishment floor, while the two wide models running the identical update at the identical width sit eight orders of magnitude lower. It is not drift at all: measured over one label it is `3.29e1` and measured over a hundred and ten labels it is `3.26e1`, so nothing is accumulating between recomputations. The configured cadence therefore cannot move it, and does not — three cadences across a sixteenfold range agree to four significant figures. And the two readings four orders apart are not two runs disagreeing: they are one run before and after the cascade's regularised arm engages, a transition this study reproduces on demand and whose peak it drove to `4.3e6`.**

The finding that carries a cost rather than a doubt is a fifth. **The condition estimate that triggers recomputation is the ratio of the largest diagonal entry of the precision matrix to the smallest, and the smallest is the replenishment floor, because the update clamps it there on every label. Once any dimension rests at the floor the estimate is pinned above its threshold permanently, and the engine performs a cubic factorisation on every label for ever — measured here at 1100 recomputations per twelve hundred labels, against 11 for the same schedule with the estimate below threshold, and 410 seconds of wall against 331.** The synchronisation error is one per cent lower in the cheaper configuration.

The red run is closed, and by a different route than the commission expected. **The failing assertion the factorisation lane could not capture is the Schur correction oracle, a debug-only cross-check rejecting a half-solve and a full-solve that agree to fifteen significant figures — eleven units in the last place against a budget of eight.** It reproduced five times out of five at a narrower competitive cutoff, in seven to twelve seconds against the reproduction's eight minutes, under a single-threaded pool as readily as under the shipped one. It has nothing to do with the drift, and it is absent from a release build.

The report changes no specification, record or implementation. It adds one instrument, an integration test ignored by default, and cites existing heads without minting any.

## What was measured, and through what · `sec:assayer:drift-what-was-measured`

Every reading here is taken through the published health surface: `last_sync_error`, `n_recompute_effective`, `sync_error_shortenings`, `cholesky_recomputes`, `regularisation_count`, `cascade_terminus_count`, `dimensions_at_floor` and `kappa` from the per-model precision detail, the competitive cell distribution from the identity-dimension health, and `cp5_reverts` from the summary. Nothing reaches into the model: the module is private and is not published even under the test-support feature, so a study of the update's arithmetic has to be conducted from outside it. That constraint shapes what can and cannot be concluded below, and the limits section says where.

The instrument drives the same dense competitive geometry the factorisation reproduction drives — two identity dimensions whose encodings place the coordinate in the domain's top four bits, registered at a competitive cutoff of ten, so that six levels of competitive splitting sit past the last depth the encoding can separate. The width is therefore grown by cells that carry no information the geometry did not already have at depth four, which is the shape the original failure was found in. Five configurations move one quantity at a time: the recomputation cadence, the competitive cutoff, the per-label decay rates, the replenishment floor, and the size of the backend's thread pool.

The synchronisation error is `‖BΣ − I‖_F` over the whole matrix (`def:monitoring:synchronisation-error`), and it is recorded only at a recomputation (`alg:gaussian:synchronisation-monitor`), carrying the value measured immediately before that recomputation. That is what makes the effective cadence, rather than the configured one, the quantity a growth curve is actually a function of — and the effective cadence turns out to be one label.

All compute ran on the shared server. The box carries 112 cores and six sibling lanes were running throughout; one-minute load averages are quoted beside every wall time. No reading in this report except the wall times is a timing quantity, so contention affects the cost columns and not the error columns.

## The error belongs to one model · `sec:assayer:drift-one-model`

The single most decisive reading is four rows of one checkpoint: four models, one process, one schedule, one width, one label count.

| Model | Configured decay | Dimensions at floor | `κ̂` | Regularised recomputes | Synchronisation error |
| --- | --- | --- | --- | --- | --- |
| Anchor | 0.9998 | 0 | 4.97e6 | 0 | 4.25e-13 |
| Operational | 0.9995 | 0 | 1.02e8 | 0 | 4.24e-7 |
| Sister | 0.9998 | 0 | 8.94e7 | 0 | 4.36e-7 |
| Outcome axis | registration's own | 26 | 3.69e5 | 384 | 2.94e1 |

Taken at 3200 labels, 1126 competitive cells, configured cadence 400.

Three of the four candidate mechanisms the commission named are refuted by this table alone, and none of the refutations needs a further run.

**The rank-one update's own rounding is refuted.** The operational and sister models run the identical Sherman–Morrison update over the identical schedule at the same width and report `4.2e-7` and `4.4e-7`. Whatever is worth `2.9e1` on the fourth row is not a property of the update, because the update is the same on all four.

**Leverage magnitude is refuted, and by a natural experiment rather than by absence.** The fifth checkpoint reverted no label at any reading of any instrument configuration, and none in five of the six reproduction runs. In the sixth it reverted eight, out of 3400 labels. That run reports a synchronisation error of `3.062e1` against the `3.063e1` and `3.064e1` of the runs that reverted nothing — the same value to four significant figures. Refused updates therefore occur, and when they occur they move the error by less than a part in a thousand.

**The conditioning of `B` is refuted, and refuted the wrong way round.** The model with the largest error has the *smallest* reported condition estimate, `3.7e5` against the operational model's `1.0e8`. The estimate is a ratio of diagonal extremes rather than a spectral quantity (`alg:gaussian:condition-adaptive-recompute`), and its own documentation calls it a lower bound on the true conditioning, so it cannot see a collinearity that leaves every diagonal entry large. The two quantities are anti-correlated across the table: a factor of 276 more conditioning goes with a factor of 7e7 less error. A monitor watching `κ̂` for the onset of this error would watch the wrong models.

What survives is the fourth row's remaining distinction: the outcome-axis model is the only one with dimensions resting at the replenishment floor, and the only one whose recomputations are being regularised. Those two facts are not independent — the regularised arm is what fires when the plain factorisation cannot proceed — and separating them is what the rest of this report is about.

The count of floored dimensions is 26 at every reading, in every configuration, while the width grows from 950 to 1372 cells. A count that does not move while the geometry grows by four hundred cells is a fixed sub-block rather than a growing population, which is worth recording as a fact about where the floor bites and is not explained here.

## Growth against labels since the last recompute · `sec:assayer:drift-growth`

The commission asked for the error as a function of labels since the last recompute. The engine does not permit that function to have more than one point, and finding out why is the substance of the answer.

Across every configuration measured, the recomputation count rose by exactly 100 for every 100 labels driven. At configured cadence 100, the count went 2590 → 3090 over 500 labels; at cadence 400, 2290 → 2790 over the same 500; at cadence 1600, 2261 → 2761. **The engine recomputes on every label at this width, whatever the cadence is configured to.** The reason is in the trigger: recomputation fires on a counter *or* on conditioning (`dec:posterior:recomputation-trigger`), and the conditioning estimate sits at `2e5` to `1e8` against a threshold of `1e5`, so the condition arm is satisfied continuously and the counter arm is never reached. The trigger is evaluated once per model per label.

The recorded synchronisation error is therefore the drift accumulated over exactly one label, in every reading this study took and — on this evidence — in the readings the original finding took. There is no hundred-label interval over which drift accumulates, because no hundred-label interval occurs.

The commission also asked for the reading at two widths smaller than the reproduction's, and those exist. Sweeping the competitive cutoff is not how they were taken: at cutoffs of six and eight the Schur oracle aborts the model owner before any measure phase begins, five attempts out of five, so that sweep measures a different defect. The abort is not the only obstacle in that direction, and it is the less durable of the two. At a cutoff of six the geometry settles at a few dozen competitive cells under the thread pool as it ships — 48 and 52 in the runs recorded below — against the hundred the instrument requires before it will treat a reading as being about the engine rather than about the wall the run stopped at. A cutoff sweep can therefore reproduce the oracle's behaviour and still not measure the drift, and repairing the oracle does not change that: the single-threaded attempts reached 152 and 186 cells before the oracle ended them, so the precondition is one a serial pool can clear at that cutoff and the shipped one cannot. Growing one cutoff to three cell targets reaches the same question by a path the engine survives.

| Competitive cells | Dimensions at floor | Recomputes per label | Synchronisation error |
| --- | --- | --- | --- |
| 444 – 626 | 26 | 1 | 5.15e-2 – 5.29e-2 |
| 776 | 26 | 1 | 9.38e0 |
| 850 | 26 | 1 | 5.23e1 |
| 886 | 26 | 1 | 1.23e3 |
| 950 | 26 | 1 | 2.07e4 |
| 1022 | 26 | 1 | 2.62e5 |
| 1060 – 1372 | 26 | 1 | 2.82e1 – 3.33e1 |

**The width dependence is not a power law; it is three regimes.** Below about seven hundred competitive cells the error is flat at five hundredths — already fifty times the magnitude the definition calls abnormal, but stable and small. Between about seven hundred and a thousand it climbs through five orders of magnitude to a peak above `2e5`. Above about a thousand and sixty it collapses onto the plateau near thirty that every other sweep in this study measured, and creeps from there as `p^0.65`.

The floored-dimension count is 26 in every row. The same twenty-six dimensions rest at the floor at a width of 444, where the error is `5.2e-2`, and at a width of 1126, where it is `2.9e1` — five hundred and sixty times larger for two and a half times the width, at the same cadence of one recomputation per label. Whatever the floored dimensions contribute, it is not what makes the error move.

What does move with it is the collinearity the geometry is accumulating. The competitive cutoff is six levels deeper than the encoding can separate, so every cell added past depth four is an indicator feature linearly dependent on ones already present. The climb begins where those cells start to outnumber the informative ones, and it stops where the regularised arm begins firing on every recomputation and bounds the inverse by construction. That is the same transition the next section reads from the other direction.

That reframes the magnitude. `2.9e1` is not a hundred labels of accumulation; it is what one rank-one update leaves behind, which is far too large to be accumulation of any kind and points instead at a standing gap that is re-established immediately after each recomputation. The transient below shows what that gap is made of.

## The transient, and the two readings four orders apart · `sec:assayer:drift-transient`

Every run passes through the same transition, and it is the transition that produced the finding's two irreconcilable numbers.

| Labels | Cells | Regularised recomputes | Synchronisation error |
| --- | --- | --- | --- |
| 2700 | 950 | 0 | 3.07e4 |
| 2800 | 984 | 0 | 1.46e5 |
| 2900 | 1022 | 84 | 2.75e1 |
| 3000 | 1060 | 184 | 2.82e1 |
| 3100 | 1100 | 284 | 2.88e1 |
| 3200 | 1126 | 384 | 2.94e1 |

Configured cadence 400, outcome-axis model.

While the plain factorisation is still succeeding, the error runs between `1.7e4` and `1.5e5`. From the label at which the regularised arm begins to fire, it falls by three to four orders of magnitude and settles onto a smooth curve just under `3e1` that thereafter only creeps upward with the width. Every configuration shows this, at the same place, with the same before-and-after magnitudes.

**The finding's `4.8e5` and `2.8e1` are the two sides of this transition.** They are not two runs that disagreed, and no seed or clock would have reconciled them, because they are readings of two different regimes of one run. The larger number is the plain factorisation inverting a matrix whose smallest eigenvalues have gone far below anything the diagonal floor bounds; the smaller is the regularised arm's shifted inverse, which is bounded by construction. The diagonal shift the factorisation lane installed at the replenishment floor is visible here doing exactly what it was installed to do — and the health surface reports its success as a persistent error, because the monitor compares the shifted inverse against the unshifted matrix.

The settled value creeps with the width as roughly its square root: fitted over cells 1060 to 1372, the exponent is 0.65, with the error moving from `2.82e1` to `3.33e1` while the floored-dimension count stays at 26. An error that moves with the width while the floored count does not is spread across the whole matrix rather than concentrated on 26 diagonal entries.

## The spread across runs · `sec:assayer:drift-run-spread`

The commission asked how much of the run-to-run spread the wall-clock decay explains. The answer is none of it, and the reason is structural rather than statistical.

A harness world is built with a virtual clock, and the engine is handed the same clock the harness holds, so time advances only when a test advances it. The label path reads both timestamp domains through that injected clock (`dec:clock:two-domains`), and every decay flows through the shared functions it feeds (`dec:clock:shared-functions`). Neither the reproduction nor this instrument advances it. The elapsed-time decay factor is therefore exactly one at every label of every run in this study, and the operational weights do not decay in wall-clock time here at all. The standing finding that they do, and that a seed therefore cannot fix which recompute meets a drifted matrix, does not hold for these harnesses. Whether it holds for a deployment reading a system clock is a different question this study does not reach.

Measured spread, at a fixed label count of 3400 and a fixed width of 1200 cells, over three independent processes:

| Run | Synchronisation error | `κ̂` | Regularised recomputes | CP5 reverts | Wall |
| --- | --- | --- | --- | --- | --- |
| 1 | 3.064e1 | 1.012e8 | 515 | 0 | 317.6 s |
| 2 | 3.064e1 | 9.394e7 | 542 | 0 | 318.3 s |
| 3 | 3.064e1 | 1.079e8 | 611 | 0 | 310.9 s |
| 4 | 3.063e1 | 1.066e8 | 331 | 0 | 308.1 s |
| 5 | 3.063e1 | 1.017e8 | 595 | 0 | 313.4 s |
| 6 | 3.062e1 | 1.018e8 | 348 | 8 | 316.4 s |

**The synchronisation error is reproducible to the four significant figures the harness prints; the trajectory around it is not, and not by a little.** The error spans 0.07 per cent across the six. The condition estimate spans 15 per cent, and the regularised-recompute count spans 85 per cent — 331 against 611 — so the six runs do substantially different arithmetic and arrive at the same error anyway. One of them refuses eight updates at the fifth checkpoint and five refuse none, with no effect on the error either. Seven further independent worlds across the cadence, decay and floor sweeps agree at matched widths to the same four figures.

That is the substantive answer: the quantity is not chaotic, it is structural. What varies between runs is which pivots the factorisation has to repair, and the mechanism for that variation is available without appeal to any clock — above a width of 512 the backend is given a parallel path at a threshold of 512 (`dec:substrate:backend-isolation`), taken by both the factorisation and the matrix product the error itself is computed from (`dec:substrate:backend-isolation`), and the order in which partial sums arrive from a thread pool is not fixed by a seed. A single-threaded confirmation was run, and it confirms the mechanism while refuting its role in the red run: at a competitive cutoff of six the parallel runs settle at 48 and 52 competitive cells where the single-threaded runs had reached 152 and 186, so the arithmetic ordering moves the trajectory a long way, and the oracle failure that ends those runs occurs in both modes alike.

## The threshold · `sec:assayer:drift-threshold`

The configured threshold is `1e-6` per dimension, multiplied by the model's width (`def:config:synchronisation-threshold`). At the widths measured it is near `1.1e-3`, which is also the magnitude the definition's own table calls abnormal.

| Model | Error | Threshold at this width | Ratio |
| --- | --- | --- | --- |
| Anchor | 4.25e-13 | ~1.1e-3 | 2.6e-10 |
| Operational | 4.24e-7 | ~1.1e-3 | 3.9e-4 |
| Sister | 4.36e-7 | ~1.1e-3 | 4.0e-4 |
| Outcome axis | 2.94e1 | ~1.1e-3 | 2.6e4 |

The threshold does fire, and it fires only where it should: the shortening count is zero on every model except the outcome-axis one. On that model it fires exactly as many times as it takes to halve the configured cadence down to its floor of 100 — no shortenings from a configured 100, two from 400, four from 1600 — and then stops, having nothing left to halve: the cadence floor is 100 (`dec:posterior:adaptive-cadence`).

**So the threshold is never what triggers a recompute in practice, in two distinct senses.** It is not a recomputation trigger at all by construction; it only shortens the interval (`dec:posterior:adaptive-cadence`), and the trigger is the counter or the conditioning. And the interval it shortens has already stopped mattering, because the conditioning arm is recomputing on every label regardless. In one decay configuration the shortening count reached 11, which means the interval recovered on runs of clean recomputes and was re-shortened repeatedly — an oscillation that costs bookkeeping and changes nothing, because the effective cadence was one label throughout.

## The interval, and what it costs · `sec:assayer:drift-interval-cost`

| Configured cadence | Effective cadence reached | Shortenings | Error at 1060 cells | at 1126 | at 1200 | at 1350 |
| --- | --- | --- | --- | --- | --- | --- |
| 100 | 100 | 0 | 2.8179e1 | 2.9442e1 | — | — |
| 400 | 100 | 2 | 2.8171e1 | 2.9442e1 | 3.0632e1 | 3.2890e1 |
| 1600 | 100 | 4 | 2.8161e1 | 2.9433e1 | 3.0640e1 | 3.2892e1 |

**A sixteenfold change in the configured interval moves the error by less than one part in a thousand.** Shortening the interval does not bound the error, and cannot, for the reason the growth section gives: all three configurations are already recomputing once per label, so all three are measuring the same one-label quantity. The interval is not a knob on this error; it is a knob that has been overridden by the conditioning trigger.

The cost side is the more useful half of the answer. Recomputing on every label at this width is what the runs actually pay: the reproduction drives 3400 labels in 311 to 318 seconds of wall, which is 92 milliseconds per label, and a cubic factorisation at a width of twelve hundred is the right order for that figure. The instrument's settings each reached their seven-minute wall having driven between 3775 and 3875 labels. Those numbers were taken with six sibling lanes on the box, load average 8.8 to 14.2 of 112 cores, so they are upper bounds on an idle machine's cost rather than measurements of it; the error columns beside them are unaffected, being ratios of matrix entries.

The per-label cost is therefore already the cost of the shortest possible interval. There is no cheaper cadence available to buy, and no more expensive one to sell.

## What the replenishment floor does to the reading · `sec:assayer:drift-floor-sweep`

The floor is the one configured quantity every model reads alike — the decay rates are configured per model family (`tab:risk:forgetting-rates`), and a per-outcome-axis model takes its rate from its own registration, so a decay sweep moves every model except the one carrying the error. The sweep drives the same geometry, cadence and decay at floors of `1e-4`, `1e-3` and `1e-2`, and it settles the diagnosis by producing a result neither candidate reading predicted.

| Cells | Error at floor `1e-4` | at `1e-3` | at `1e-2` |
| --- | --- | --- | --- |
| 1126 | 2.9686e1 | 2.9438e1 | 2.9123e1 |
| 1200 | 3.0879e1 | 3.0639e1 | 3.0323e1 |
| 1280 | 3.2021e1 | 3.1781e1 | 3.1486e1 |
| 1350 | 3.3130e1 | 3.2895e1 | 3.2592e1 |

**A hundredfold change in the floor moves the settled error by 1.6 per cent, and moves it downward.** The shifted-inverse account predicted growth with the floor and is refuted as an account of the magnitude. The clamp account predicted flatness in the floor and survives this contrast, having already been refuted as an *accumulation* story by the effective cadence.

What the sweep does establish is sharper than either, and it comes from the columns beside the error.

| Floor | `κ̂` observed | Implied maximum diagonal | Recomputes per 1200 labels | Wall for 3800 labels |
| --- | --- | --- | --- | --- |
| `1e-4` | 2.0e6 – 4.1e6 | 200 – 410 | 1100 | 420.0 s |
| `1e-3` | 2.0e5 – 3.7e5 | 200 – 370 | 1101 | 409.9 s |
| `1e-2` | 1.9e4 – 4.0e4 | 190 – 400 | 11 | 331.2 s |

The condition estimate is exactly proportional to the reciprocal of the floor, and the maximum diagonal it implies is the same 200 to 400 in all three. That is the estimate's definition showing through: it is the ratio of the largest diagonal entry to the smallest (`alg:gaussian:condition-adaptive-recompute`), and the smallest is the floor, because the clamp puts it there. **`κ̂` is therefore not a reading of the conditioning at all once any dimension rests at the floor. It is `max_diag / λ_floor`, a configured constant times a quantity that moves slowly, and at the shipped floor of `1e-3` with a maximum diagonal near 300 it stands at `3e5` against a threshold of `1e5` permanently.**

That is what makes the engine recompute on every label, and the third column is what it costs: at a floor of `1e-2` the estimate falls below the threshold, the conditioning arm stops firing, and the recomputation count over twelve hundred labels drops from 1100 to 11. The shipped configuration sits a factor of three below the point at which this stops happening.

And the error is unchanged. At a floor of `1e-2` the recorded value is the drift accumulated over about a hundred and ten labels rather than over one, and it is `3.26e1` where the one-label figure was `3.29e1`. **The same quantity, measured over one label and over a hundred and ten, differs by one per cent.** That is the decisive statement of this study's first question: the synchronisation error does not grow with labels since the last recompute, because it is not accumulation. It is a standing quantity re-established immediately after every recomputation, whose size is set by the width — as `p^0.65` across the plateau, and far more steeply through the climb that reaches it — and by nothing else the study could move.

The transient behaves the other way and is worth recording. Its peak rises with the floor and its duration lengthens, reaching `4.3e6` at the shipped floor before the regularised arm engages. The commissioning finding's `4.8e5` sits comfortably inside that range, which is the last piece of evidence that the two readings four orders apart were one run's two regimes.

## The red run · `sec:assayer:drift-red-run`

**The failing assertion is captured, and it is not the drift.** It is the Schur correction oracle, a debug-only cross-check that computes the same correction two ways and asserts per-element agreement (`dec:posterior:debug-oracle`). It aborts the model owner, which is why the original run produced no verdict: the owner dies, the driving thread parks on an acknowledgement that is never coming, and whatever is running the test eventually gives up on it. The instrument arms a panic hook that recognises the owner by thread name and ends the process with the failure status, which is what turned the silence into the line below.

```
Schur oracle: correction[28,29] half-solve=2.997529035533703e3 full-solve=2.997529035533708e3 diff=5.00e-12 tolerance=3.66e-12
```

The two methods agree to fifteen significant figures. The disagreement is eleven units in the last place of a quantity near three thousand, against a budget of eight.

The oracle's tolerance is `2γ_n·magnitude + 1e-12` with `γ_n = nu/(1 − nu)` and `n` the factor's dimension. Solving the printed tolerance back gives `n = 4`. The eight units are the whole budget rather than its Wilkinson half: `2γ_4·2.9975e3 = 2.66e-12` is 5.85 units in the last place, and the `1e-12` absolute term supplies the remaining 2.20. But neither side is a four-term dot product. Each is a triangular solve followed by a product, and a triangular solve's error carries the conditioning of the factor it solves against, which the plain `γ_n` does not. The factor is exactly where the replenishment floor has put the smallest pivot, so the conditioning the bound omits is the conditioning the floor creates.

Four further attempts were run at the narrower cutoffs, two with the backend's thread pool as it ships and two with a single-threaded pool.

| Attempt | Pool | Wall to failure | Half-solve | Diff | Tolerance | Diff / tolerance |
| --- | --- | --- | --- | --- | --- | --- |
| width sweep | as shipped | 9.5 s | 2.997529035533703e3 | 5.00e-12 | 3.66e-12 | 1.37 |
| par-1 | as shipped | 125.2 s | -2.096839946761072e3 | 6.37e-12 | 5.66e-12 | 1.13 |
| par-2 | as shipped | 123.0 s | -1.824588360434785e3 | 5.23e-12 | 5.05e-12 | 1.04 |
| ser-1 | single thread | 10.2 s | -2.328880906135455e2 | 1.53e-12 | 1.52e-12 | 1.007 |
| ser-2 | single thread | 7.5 s | -1.757368890895766e3 | 6.59e-12 | 4.12e-12 | 1.60 |

**Five attempts, five failures, at five different matrix positions, every one of them within a factor of 1.01 to 1.60 of the tolerance.** An oracle that fails by between one per cent and sixty per cent of its own budget, at whichever entry happens to round worst, is not detecting an algorithmic divergence; it is sitting inside the noise of the arithmetic it audits. Its own documentation says so in as many words: a bound tighter than the error of the arithmetic reports the arithmetic rather than the algorithm, and an algorithmic fault — a transposed block, a solve against the wrong factor — would move an entry by a share of its magnitude rather than by its last bits. Nothing here moves by more than the last bits.

Two consequences follow, and both matter more than the drift the study was commissioned about.

**The single-threaded pool fails too, and fails sooner.** The backend's parallel path is therefore refuted as the cause: the intermittency is the ordinary sensitivity of a boundary case to whatever rounding the run happens to produce, not the thread pool's reduction order. What the pool does change is the trajectory, and substantially — at a cutoff of six the parallel runs settle at 48 and 52 competitive cells while the single-threaded runs had reached 152 and 186 before they failed. That is a large divergence from one seed, and it is the mechanism behind the run-to-run variation in the condition estimate and the regularised-recompute counts reported above, with the clock ruled out.

**A nine-second reproduction now exists for an eight-minute one.** At a competitive cutoff of six or eight the oracle fails in seven to twelve seconds under a single-threaded pool, against the reproduction's five to eight minutes and its one-in-six rate at a cutoff of ten. That narrow configuration is not in the repository, and this report was wrong to say the instrument's width sweep is it: the sweep drives a competitive cutoff of ten at every setting, as does every other sweep the instrument ships, and the cutoff of six that produced the readings above was a working-tree edit that was never committed. Reproducing the abort means editing the cutoff down, which is what the lane that repaired the tolerance did — at a cutoff of six the retired bound aborts in 8.7 seconds and the derived one produces no oracle line at all in 79. Nor is the narrow configuration a place a drift reading could be taken even with the oracle quiet: at that cutoff the competitive cells settle in the dozens under the shipped pool, below the hundred the instrument's own precondition requires, so the configuration reproduces the oracle's behaviour and measures nothing about the drift.

The reproduction itself was run six times at this commit and passed six times.

| Run | Result | Labels | Cells | `κ̂` | Regularised | Terminus | CP5 | Wall |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| 1 | pass | 3400 | 1200 | 1.012e8 | 515 | 0 | 0 | 317.6 s |
| 2 | pass | 3400 | 1200 | 9.394e7 | 542 | 0 | 0 | 318.3 s |
| 3 | pass | 3400 | 1200 | 1.079e8 | 611 | 0 | 0 | 310.9 s |
| 4 | pass | 3400 | 1200 | 1.066e8 | 331 | 0 | 0 | 308.1 s |
| 5 | pass | 3400 | 1200 | 1.017e8 | 595 | 0 | 0 | 313.4 s |
| 6 | pass | 3400 | 1200 | 1.018e8 | 348 | 0 | 8 | 316.4 s |

Every log is kept at `.lane/repro-N.log` in the lane checkout. So the standing supposition that the drift is the likeliest cause of the intermittent red run is refuted rather than supported. The cascade-terminus count the reproduction asserts on was zero at every checkpoint of every run, and every run reached 1200 competitive cells against the 1000 its second assertion requires, so none passed by stopping at a wall.

## Recommendation and its falsifier · `sec:assayer:drift-recommendation-falsifier`

**Recommendation: widen the oracle, fix the trigger, fix the reading. Do not shorten the interval, and do not change the update.** The commission offered three courses and the measurements pick none of them cleanly, because the drift the commission was asked to bound turns out not to be drift: over one label it is `3.29e1` and over a hundred and ten labels it is `3.26e1`. There is nothing accumulating that a shorter interval could catch.

**First, and cheapest, widen the Schur oracle's tolerance.** It is a debug-only assertion that aborts the model owner on a last-bit disagreement, it failed five times out of five once a narrow enough geometry was driven, and every failure sat between one per cent and sixty per cent above its own budget. Its bound uses a plain `γ_n` in the factor's dimension and omits the conditioning of the triangular solve both sides perform, which is precisely the quantity the replenishment floor inflates. A bound carrying that conditioning, or stated against the operands rather than the answer, is the principled repair; a factor of two on the present constant would have covered every failure observed here. Until it is repaired it is not only a false alarm but a blocker: it aborted every run this study attempted at competitive cutoffs of six and eight, which is one of the two reasons the width axis above has no readings at the two smaller widths, the other being the cell precondition those cutoffs cannot meet.

**This recommendation has been taken, and by the principled repair rather than by the constant.** The oracle's tolerance is now derived from the standard model for what its two routes actually compute: a substitution's forward error carries the componentwise conditioning of the factor it solves against, evaluated without forming an inverse, and the bound is stated against the operands rather than against the answer, which retired the absolute term along with the relative one (`dec:posterior:debug-oracle`). Under it the shipped width sweep and the eight-minute reproduction both complete with no oracle line at all, and the cutoff of six that aborted in 8.7 seconds under the retired bound runs 79 seconds to a clean finish. The reasoning above is left exactly as it was argued; only its status has changed.

**Second, the conditioning trigger is firing on a quantity that is not the conditioning.** The estimate is the ratio of the largest diagonal entry to the smallest, and the smallest is the replenishment floor by construction, because the update clamps it there on every label (`alg:update:sherman-morrison`). Once any dimension rests at the floor the estimate is pinned at `max_diag / λ_floor`, which at the shipped floor and an observed maximum diagonal near 300 is `3e5` against a threshold of `1e5`. It never falls back, so the engine recomputes a cubic factorisation on every label, for ever, on a model whose synchronisation error that recomputation does not improve. The floor sweep measures the cost directly: taking the estimate below its threshold cut recomputations over twelve hundred labels from 1100 to 11 and the wall for the same schedule from 410 to 331 seconds under load, with the error one per cent *lower*. The repair is to make the estimate blind to dimensions sitting at the floor, or to compare it against a floor-aware baseline; raising `λ_floor` produces the same effect but is a modelling change, since the floor bounds how much confidence a coordinate may lose, and this study measured only its numerical consequences.

**Third, the reading cannot currently report an excursion.** The health surface shows `2.9e1` against a threshold of `1.1e-3` on a model behaving, on this evidence, as designed, while the two models carrying the operational risk sit three orders of magnitude *under* their own threshold and are invisible in the same summary, because `max_sync_error` takes the maximum across models (`dec:health:tiered-queries`). An aggregate pinned by one model's structural constant cannot report a real excursion on any other. The threshold's shape is also wrong for the quantity: it scales linearly in the width while the measured error scales as `p^0.65`, so the two diverge in both directions at once. The precedent for the repair is in this package and one lane old — the Schur oracle was rebounded by rounding error as a relative tolerance rather than by an absolute distance, for exactly this reason.

**Shortening the interval is refuted, with the numbers.** A sixteenfold change in the configured cadence moved the error by less than one part in a thousand. Every configuration already halves down to the floor of 100, and every configuration already recomputes once per label, so all three were measuring the same one-label quantity. There is no interval left to buy, and the per-label cost is already what the shortest one would charge.

**Accepting the current margin is refuted too**, but not because the margin is dangerous — on this evidence the arithmetic is sound and no cascade terminus occurred in any run. It is refuted because accepting it means keeping a monitor that reports a structural constant as a permanent fault and a trigger that spends a cubic factorisation per label to no effect. The margin is also not where it looks: the reading is `5.2e-2` at a width of six hundred and `2.9e1` at eleven hundred, so a deployment reading it today is somewhere on a five-order climb rather than at a steady state, and the summary cannot say where. The cost is real and measured; the alarm is not.

This recommendation is falsified if the required contract is that the covariance must be the inverse of the unshifted precision matrix and that `κ̂` must remain a purely local `O(p)` reading. In that case the trigger is behaving correctly by refusing to trust a matrix with a floored dimension, the reading is correct in calling the result an error, and the finding relocates upstream: the geometry is producing a precision matrix whose numerical rank is far below its dimension, because the competitive cutoff is six levels deeper than the encoding can separate and the extra indicator features are therefore linearly dependent by construction. The repair would then be to refuse or merge competitive cells the encoding cannot distinguish, and neither the monitor nor the cadence would be the place to touch. The instrument this lane adds measures that repair as readily as the other two.

## Method and limits · `sec:assayer:drift-method-limits`

Every number above was produced on the shared server through the lane gateway, by the instrument this lane adds and by the existing reproduction, both run under `--ignored` with logs kept. No number was carried over from the commissioning finding except the two readings it reported, which appear only as the quantities to be explained. Arithmetic on the printed readings — ratios, the fitted width exponent — was done by hand from the logged values.

The study is conducted entirely from outside the model. The `model` module is private and is not published even under the test-support feature, so the quantities that would settle the remaining question directly are not reachable: the smallest eigenvalue of the precision matrix, the numerical rank, the size of the shift the regularised arm applied, and the synchronisation error measured against the shifted matrix rather than the unshifted one. **The hook this study would ask for is a per-model reading of the regularisation actually applied at the last recomputation, and of `‖BΣ − I‖` taken against the matrix that was inverted.** With that pair the shifted-inverse account is confirmed or refuted in one reading rather than by the inference chain above.

Three further limits. The width proxy throughout is the competitive cell count, which the geometry's own documentation ties to one indicator feature per cell but which is not the model's feature width as the model knows it, and no published reading gives that width. The decay contrast could not be applied to the model carrying the error, for the registration reason given above, so the clamp account is refuted here as an *accumulation* story by the one-label cadence and not by a decay sweep on the affected model. And the single-threaded comparison is not at matched label counts, because every attempt in both modes ended in the oracle failure before its measure phase: it establishes that the trajectories diverge widely and that the failure is independent of the pool, but it does not measure the spread of the synchronisation error under a serial pool, which would need the tolerance repaired first.

Wall times are quoted with the load average that stood beside them; six sibling lanes shared the box throughout and no measurement in this report was taken with exclusive access, which is sound for the error columns and makes every cost column an upper bound.

## Corrections · `sec:assayer:drift-corrections`

Three statements in this report were wrong when it landed, and a fourth has been overtaken by the repair it recommended. They are corrected in place above; this section records what was wrong, what the evidence is, and what the correction changes, so that a reader who has met the earlier text elsewhere can reconcile the two.

**The oracle's budget was split wrongly between its two terms, and two unit systems make the arithmetic look contradictory.** The printed failure reads `diff=5.00e-12 tolerance=3.66e-12` on a half-solve of `2.997529035533703e3`. That magnitude lies in `[2^11, 2^12)`, so one unit in its last place is `2^11·2^-52 = 4.547e-13`: the difference is `5.00e-12 / 4.547e-13 = 11.0` units and the whole tolerance is `3.66e-12 / 4.547e-13 = 8.05`. The headline's eleven units against a budget of eight is therefore right, and stands. What was wrong is the sentence attributing that eight to the four-term Wilkinson budget alone. Solving `2γ_n·|x| + 1e-12 = 3.66e-12` back does give `n = 4`, but the Wilkinson term is then `2γ_4·2.9975e3 = 2.66e-12`, only 5.85 units in the last place, and the absolute `1e-12` supplies the other 2.20. The oracle's own documentation states the same five failures in the other natural unit, `u|x|` with `u = 2^-53`, where `u|x| = 3.328e-13` and the same difference is `15.0` while the same tolerance is `11.0` and the Wilkinson term is exactly `8`. An eleven and an eight occur in both systems and mean different quantities in each, which is the whole of the apparent disagreement between this report and that documentation; the five recorded failures agree row for row once the unit is fixed.

**The narrow configuration was never in the repository.** The red-run section closed by telling whoever repaired the tolerance to use the narrow configuration and calling the instrument's width sweep that configuration. The instrument drives a competitive cutoff of ten at every setting of every sweep it ships, the width sweep included; the cutoff of six and eight that produced the five recorded aborts was a working-tree edit that was never committed. The sentence now says that, and names the reproduction that does exist: editing the cutoff to six aborts in 8.7 seconds under the retired bound and runs 79 seconds to a clean finish under the derived one.

**No drift reading can be taken at those cutoffs under any bound.** The report treated the oracle abort as the only thing standing between it and the two smaller widths the commission asked for. It is not the only thing, and it is the less durable of the two. The instrument will not call a reading about the engine unless the run stands on a hundred competitive cells, and at a cutoff of six the geometry settles at a few dozen under the thread pool as it ships — 48 and 52 in this report's own attempts. The narrow configuration can therefore reproduce the oracle's behaviour and measure nothing about the drift, and repairing the oracle does not change that. The single-threaded attempts reached 152 and 186 cells before the oracle ended them, so the precondition is one a serial pool clears at that cutoff and the shipped pool does not, which is a fact about the geometry rather than about the oracle.

**The first recommendation has been taken.** The oracle's tolerance is no longer `2γ_n·magnitude + 1e-12`. It is derived from the standard model for what the two routes compute — a substitution's forward error carrying the componentwise conditioning of the factor it solves against, evaluated by a comparison-matrix solve rather than by forming an inverse, summed with each product's own rounding and stated against the operands instead of the answer, which retired the absolute term with the relative one (`dec:posterior:debug-oracle`). That is the principled repair this report argued for rather than the factor of two it also offered, and no constant was fitted to the failures. The reasoning in the recommendation stands as it was written; only its status has changed. The report's arithmetic is otherwise unaltered, and no reading, table or conclusion about the synchronisation error is touched by any of the four corrections.
