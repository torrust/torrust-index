# The cadence's floor is held by declined rebuilds, and nothing published says why they were declined · `rep:assayer:declined-rebuild-cadence`

This report is a reading and its interpretation. It changes no record, no specification and no implementation, and the two repairs it identifies are left as open questions rather than taken. What it does change is the instruments: both now print the count of rebuilds a model declined beside the count it ran, which is what turned the previous reading's elimination into a measurement.

The question it answers was left open by the preceding study of the synchronisation reading (`rep:assayer:synchronisation-drift-study`). That work separated the reading into the part the replenishment clamp puts there and the residual the arithmetic accumulates, and moved the cadence onto the residual (`def:monitoring:synchronisation-error`), (`dec:posterior:adaptive-cadence`). Two models nevertheless stayed pinned at the adaptive floor with residuals three orders under their own threshold and no cascade terminus events at all, which left the declined-rebuild arm of the cadence as the only remaining explanation and no way to see it. That arm is now visible, and it is the whole of the answer.

## The reading · `sec:assayer:declined-cadence-reading`

The dense competitive schedule was run once at this head: 3400 labels in 152.3 s, no cascade terminus event on any model, no labels reverted at the fifth numeric checkpoint, and no marginalisations. The instrument is the harness that grows a competitive geometry until the feature width is in the thousands (`claim:wellness:a-dense-competitive-schedule-leaves-every-precision-matrix-factorisable`); each checkpoint prints one aggregate row and one line per model, and the per-model lines now carry the rebuilds run and the rebuilds declined.

**Table (The last three checkpoints, per model)** · `tab:assayer:declined-cadence-checkpoints`

| Labels | Model | Measured | Prior-induced | Residual | Interval | Rebuilds | Declined |
| --- | --- | --- | --- | --- | --- | --- | --- |
| 3000 | Anchor | 5.800e-12 | 0 | 5.800e-12 | 1000 | 3 | 0 |
| 3000 | Operational | 1.004e-6 | 0 | 1.004e-6 | 100 | 6 | 5 |
| 3000 | OutcomeAxis | 2.331e6 | 2.331e6 | 5.878e-4 | 1000 | 3 | 0 |
| 3000 | Sister | 7.317e-7 | 0 | 7.317e-7 | 100 | 6 | 5 |
| 3200 | Anchor | 5.800e-12 | 0 | 5.800e-12 | 1000 | 3 | 0 |
| 3200 | Operational | 9.521e-7 | 0 | 9.521e-7 | 100 | 8 | 7 |
| 3200 | OutcomeAxis | 1.179e5 | 1.179e5 | 3.494e-1 | 500 | 4 | 0 |
| 3200 | Sister | 7.896e-7 | 0 | 7.896e-7 | 100 | 8 | 7 |
| 3400 | Anchor | 5.800e-12 | 0 | 5.800e-12 | 1000 | 3 | 0 |
| 3400 | Operational | 9.721e-7 | 0 | 9.721e-7 | 100 | 10 | 9 |
| 3400 | OutcomeAxis | 1.179e5 | 1.179e5 | 3.494e-1 | 500 | 4 | 0 |
| 3400 | Sister | 8.480e-7 | 0 | 8.480e-7 | 100 | 10 | 9 |

**The answer is nine in ten, and it is confined to two models.** Of the ten rebuilds the operational and the sister model each ran by label 3400, nine were declined — the first was adopted and every one after it was not. The outcome-axis model declined none of its four and the anchor none of its three. The two models declining their rebuilds are exactly the two the previous reading found pinned at the adaptive floor of a hundred labels, and the arithmetic of how they got there is legible in the checkpoints: both stood at the configured thousand through label 1800, halved to 500 at 2000 on their first decline, to 250 at 2600 on their second, to 125 at 2800 on their third, and reached the floor at 3000 on their fifth. Nothing else was acting. There were no terminus events to halve on, and their residual — which is their whole reading, since neither carries any prior-induced component at all — sat at about one part in a million against a threshold of at least 1.2e-3 at these widths, so the cadence's other ground for shortening was never within three orders of firing.

**What that costs is a cubic factorisation every hundred labels, thrown away each time.** A declined rebuild leaves the model holding the pair it already had. The work is done, the result is measured, the measurement refuses it, and the only lasting effect is that the recovery streak is cleared and the interval is halved — so the next one comes sooner, and is declined too. The engine is in a stable state that produces the maximum possible rate of discarded work, and it entered that state on a model whose drift was already three orders under the threshold that defines drift worth acting on.

## The contrast, and what it rules out · `sec:assayer:declined-cadence-contrast`

**An isolated model at the same published conditioning declines nothing.** The obvious explanation for the schedule's declines is that at this conditioning the two quantities the adoption test compares arrive at the same magnitude: the maintained pair's drift and a fresh inverse's own rounding residual become comparable, the comparison between them is decided by rounding noise, and about half the rebuilds are lost. A crate test was written to check that, and it refutes it (`claim:bayes:an-isolated-model-at-the-schedules-conditioning-adopts-every-rebuild`). A model of width four hundred is driven to a synchronised state through the real update path at the operational family's own forgetting rate, with the feature magnitudes spread over four decades so that the precision diagonal spans from just above the replenishment floor to about 1e5 — a diagonal ratio of 8.776e7, which is the figure the schedule reports. Ten rebuilds follow at the adaptive floor's own interval, and every one of them is adopted: the drift before the last is 9.220e-12, the drift the rebuilt pair carries is 4.096e-12, the mean ratio of after to before over the ten is 0.318, and the threshold at this width is 4.000e-4. The fresh inverse is the better of the two readings every time, by a factor of three. The same measurement at widths two hundred, eight hundred and twelve hundred returns the same verdict, so the width does not separate the two cases either.

**So the declines are a property of the schedule's matrices and not of the conditioning figure they publish, and the figure is why the two can come apart.** The cheap diagonal ratio is a lower bound on the condition number and not a measurement of it, which the recomputation record states as the reason its own conditioning arm is relative rather than absolute (`dec:posterior:recomputation-trigger`), and which the self-guarding study establishes in the requirement's own words: the replenishment floor is coordinatewise, and a small eigenvalue can sit beneath large equal diagonal entries through the off-diagonal structure (`req:gaussian:prior-replenishment-floor`), (`rep:assayer:self-guarding-model-study`). A competitive geometry produces exactly that structure. A cell and the parent it was split from fire together and carry nearly the same value; the width grows between one rebuild and the next; coordinates are added at the prior while the schedule runs. An isolated model built to the published ratio has none of that, and a rebuild's inverse is accurate to the true conditioning rather than to the reported one — which is what puts the schedule's after-drift above its before-drift and this model's below.

**Both cases are kept, because the pair is the finding.** Neither run alone says anything: the schedule shows declines without saying what produced them, and the isolated model shows adoptions at a conditioning that was supposed to produce declines. Together they say that the published conditioning does not predict the adoption verdict, which is a fact about the instrument as much as about the engine.

## The diagnostic gap · `sec:assayer:declined-cadence-gap`

**The adoption test declines on either of two conditions, and nothing published says which one fired.** A rebuild is carried when its result is finite, when the drift measured after it is at or below the drift measured before it, and when that drift after is under the model's width-scaled threshold (`dec:posterior:measured-adoption`). Two of those can fail independently: the rebuild can be no better than what the model already had, or it can be better and still over the threshold. The first is a statement about the rebuild, the second about the model. The detailed precision health carries the drift standing *before* the last rebuild and does not carry the drift after it, so a reader of the schedule's output can see that nine rebuilds were declined and cannot see why any of them was.

**The evidence for which one fires here is circumstantial, and it points at the first.** The drift before each of these rebuilds is about 1e-6 against a threshold of at least 1.2e-3, so had the rebuild merely reproduced what the model held, its after-drift would have been under the threshold with three orders to spare. For the threshold condition to be what failed, the rebuild's own inverse would have to be a thousand times worse than the pair it was replacing — which is possible at a conditioning the published ratio understates, and is exactly what the measurement cannot distinguish from the ordering condition failing on its own. The model's baseline record holds both figures; only the published detail does not.

**Desk item, no ruling needed.** Publish the last rebuild's after-drift and its adoption verdict on the detail tier — one field beside the readings already there, in the health surface rather than in the model — and re-read the schedule. That reading would settle in one run which of the two conditions the operational and sister models are failing, and neither repair below can be chosen without it. The work is outside this lane's scope.

## Two open questions · `sec:assayer:declined-cadence-questions`

Both of these are refinements of a design that is working as written. Neither is a defect report and neither is drafted here as text.

**First: should a decline where the drift before the rebuild was already under the threshold shorten the interval at all?** The cadence shortens on a decline because a declined rebuild has left the drift it measured standing, so the model needs looking at sooner. That argument holds when the drift left standing is drift worth acting on. When the drift standing before the rebuild was already under the threshold, nothing needed fixing, and a rebuild that failed to improve on a healthy pair has told the engine that the pair is healthy rather than that it is not. The change would be confined to the interval computation and to the outcome the model applies: the not-adopted arm would consult the drift measured before the rebuild rather than halving unconditionally, and the outcome the model applies would have to pass that figure to it, which it already carries on the baseline. What would *not* change is the alarm the mechanism exists to raise — a decline where the drift before was over the threshold and the rebuild did not improve it is still a model that needs looking at sooner, and would still halve. Nor would the adoption test itself change: the rebuild would still be declined and its result still discarded, since a covariance that is not an improvement is not one whatever the cadence does about it. The cost of getting this wrong in the permissive direction is a model that drifts on a long interval; the cost of leaving it as it is, is the reading above.

**Second: should the residual be judged relative to the reading it was taken out of?** The outcome-axis model in the table publishes a measured reading of 1.179e5, a prior-induced component of 1.179e5, and a residual of 3.494e-1 — a residual that is the difference of two numbers of order 1e5 and is therefore their relative rounding, about three parts in a million. It is nevertheless compared against an absolute threshold, it exceeds it, and the model shortened from 1000 to 500 at label 3200 on the strength of it. The residual is a well-defined quantity while the prior-induced part is a modest share of the reading, and it stops carrying information once that part dominates by many orders: below the cancellation level there is nothing left in it but the rounding of the subtraction. The candidate is to compare the residual to the reading it came from, or to floor it at the reading's own rounding level, before testing it against the threshold. This is a different question from the first and touches a different comparison, and it is raised here because the same run shows both.

## What was measured, and where · `sec:assayer:declined-cadence-provenance`

The schedule's reading is one run of the dense competitive schedule at this head, taken after the declined column landed: 3400 labels, 152.3 s, terminus 0. The contrast is one crate test in the recomputation suite, deterministic in its inputs and carrying its own numbers in its documentation, run under the package's default suite. Both instruments print the declined count now — the width sweep's per-model sample line and the dense schedule's per-model line, with the largest of the declines on the schedule's aggregate row, taken as a maximum rather than a sum because a sum over four models answers the question for none of them.

## Which condition fires · `sec:assayer:declined-cadence-after-drift`

**The declines are the ordering condition, and the threshold arm is nowhere near firing.** The detail tier now carries the drift a rebuild measured after itself, its factorisation's verdict and its adoption, beside the drift that stood before it, and both instruments print them (`dec:health:tiered-queries`), (`dec:posterior:measured-adoption`). The schedule was re-run once at the head that publishes them: 3400 labels in 131.4 s, no cascade terminus on any model, no labels reverted at the fifth numeric checkpoint, and the same nine declines in ten on the operational and the sister model that the reading above found. At every one of the last three checkpoints those two models report an after-drift about three to four times their before-drift. The fresh inverse is the worse of the two readings, the first condition fails on its own, and the second is satisfied at the same moment with three orders to spare.

**Table (The last three checkpoints, after-drift against before-drift and against the threshold)** · `tab:assayer:declined-cadence-after-drift`

| Labels | Model | Before | After | After / before | After / threshold | Adopted |
| --- | --- | --- | --- | --- | --- | --- |
| 3000 | Anchor | 4.184e-12 | 2.287e-13 | 0.055 | 2.2e-10 | yes |
| 3000 | Operational | 5.792e-7 | 1.774e-6 | 3.063 | 1.7e-3 | no |
| 3000 | OutcomeAxis | 2.331e6 | 1.473e-5 | 6.3e-12 | 1.4e-2 | yes |
| 3000 | Sister | 6.109e-7 | 2.353e-6 | 3.852 | 2.2e-3 | no |
| 3200 | Anchor | 4.184e-12 | 2.287e-13 | 0.055 | 2.0e-10 | yes |
| 3200 | Operational | 6.084e-7 | 1.905e-6 | 3.131 | 1.7e-3 | no |
| 3200 | OutcomeAxis | 1.179e5 | 1.932e-5 | 1.6e-10 | 1.7e-2 | yes |
| 3200 | Sister | 6.630e-7 | 2.816e-6 | 4.247 | 2.5e-3 | no |
| 3400 | Anchor | 4.184e-12 | 2.287e-13 | 0.055 | 1.9e-10 | yes |
| 3400 | Operational | 6.321e-7 | 1.833e-6 | 2.900 | 1.5e-3 | no |
| 3400 | OutcomeAxis | 1.179e5 | 1.932e-5 | 1.6e-10 | 1.6e-2 | yes |
| 3400 | Sister | 6.968e-7 | 2.677e-6 | 3.842 | 2.2e-3 | no |

The threshold is a millionth of the model's width and the width is not a published quantity, so the last column is computed against a millionth of the competitive-cell count the same checkpoint carries — 1060, 1126 and 1200 cells, giving thresholds of at least 1.060e-3, 1.126e-3 and 1.200e-3. Every competitive cell contributes one indicator position and the score carries more besides, so that count is a lower bound on the width and the quoted ratios are upper bounds on the ratio. They are three orders under one either way, which is the only precision the answer needs.

**The two models that adopt every rebuild are the reading's own control.** In the same run and at the same checkpoints, the anchor model's after-drift is a twentieth of its before-drift and the outcome-axis model's is under a part in a billion of its own, and both adopt. So the ordering comparison is not failing everywhere at this conditioning, and it is not failing because the two readings have collapsed into rounding noise: two models pass it by wide margins in the same publication that shows two failing it by a factor of three. The isolated model of the contrast above passes it by a factor of three in the other direction, and this run measures directly what that contrast could only reach by elimination — the schedule's matrices make a fresh inverse *worse* than the maintained pair, at a diagonal ratio of 9.391e7 that says nothing about it.

**What that settles for the first question, and what it leaves open.** The first candidate would have the not-adopted arm consult the drift standing before the rebuild rather than halve unconditionally (`sec:assayer:declined-cadence-questions`). Every decline in this run is a decline whose before-drift is three orders under the threshold, so under that candidate none of them would shorten the interval, and the operational and the sister model would leave the adaptive floor entirely — the cadence collapse the reading above documented is exactly the population the candidate addresses, and it is the whole of it. What the reading also settles is that the decline itself is correct in every case: the rebuild really was the worse of the two readings, so the adoption test is declining the right answers and only the cadence's response to a decline is in question. What it does not settle is the permissive direction's cost, which is a model drifting on a long interval while its rebuilds keep failing to improve on it; nothing in this run exhibits that state, because no model here has a before-drift near its threshold at all.

**And what it says about the second.** The residual arm is not what is shortening these two models: their prior-induced component is zero, their whole reading is residual, and that residual is three orders under the threshold throughout. The second candidate therefore touches none of the declines, and it is confined to the outcome-axis model, which declines nothing. That model is nevertheless the sharper witness for it in this run than in the one above: at the same measured reading of 1.179e5 and the same prior-induced figure of 1.179e5, its residual reads 1.133e0 here where the earlier run read 3.494e-1. The two runs differ by a factor of three in a quantity whose two operands agree to every printed digit, which is what a residual below the cancellation level is — the rounding of the subtraction, varying run to run — and the absolute threshold it is compared against does not vary with it.

**Provenance.** One run of the dense competitive schedule at the head that publishes the three readings, taken after they landed: 3400 labels, 131.4 s, terminus 0, no revert at the fifth numeric checkpoint. The figures in the table are the per-model lines of that run's last three checkpoints, read from the detailed precision health, and the ratios are computed from them.


## What the restructure measured · `sec:assayer:declined-cadence-restructure`

The reading above raised a question and the answer restructured the flow: the cadence and the cheap check now trigger a *measurement*, and only a bad measurement rebuilds (`dec:posterior:recomputation-trigger`). This section is what both instruments said about that at the head which implements it, run against the base the lane started from on the same box and the same toolchain. Two of its findings are not what the design expected, and they are stated first because the tables are read differently once they are known.

**The base in these tables is not the base this reading described, and the difference is in the instrument rather than in the engine.** Re-run today at the lane's base commit, the dense schedule reports the anchor, operational and sister models each running three rebuilds with none declined but one, at intervals of a thousand, a thousand and five hundred. The nine-in-ten declines and the hundred-label intervals this reading opened with do not reproduce there. What does reproduce, and reproduces exactly, is the width sweep: at its widest setting the base still shows the operational and the sister model declining sixteen of nineteen rebuilds with both pinned at the adaptive floor. The two instruments differ in what they hold fixed — the sweep drives three widths to fixed targets and samples every hundred labels, while the schedule grows a competitive geometry continuously — and the population the declines live in is a property of the matrices a schedule happens to produce rather than of the conditioning it publishes, which is this reading's own central finding (`sec:assayer:declined-cadence-contrast`). Between that reading and this one the detail tier gained the after-drift and the verdict, the cadence moved onto the residual, and the toolchain moved a nightly; any of the three changes the last bits of a competitive geometry, and the schedule's declines are decided in the last bits. The honest statement is that the schedule's decline population is not stable across those changes and the sweep's is, so the sweep is the instrument the claim rests on.

**The wall time per label rises rather than falls, and the reason is an order and not an accident.** The measurement takes two matrix products — one for the drift and one for the magnitude product its resolution is read from — and a product is `O(p³)`, the same order as the factorisation the measurement exists to avoid. So the split is cheaper only where it removes more factorisations than it adds products. On the sweep, where the base pays fifty-eight rebuilds and the head pays twelve, it very nearly is; on the schedule, where today's base pays fourteen rebuilds and the head pays five rebuilds and nine measurements, it is not. The figures are in the fourth table and they are the deliverable rather than a caveat: a design that wants both the split and the wall time needs a cheaper probe, and two are visible from here — the resolution's magnitude product can be replaced by the `O(p²)` bound `‖B‖_F‖Σ‖_F`, which is looser and still rigorous, or it can be computed only where the residual is over the threshold, since it no longer decides anything.

**Table (The dense competitive schedule at 3400 labels, base against head)** · `tab:assayer:restructure-dense`

| Model | Base rebuilds | Head rebuilds | Head measurements | Base interval | Head interval | Head alarms | Base reading | Head reading |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Anchor | 3 | 0 | 3 | 1000 | 1000 | 0 | 3.814e-12 | 8.553e-11 |
| Operational | 3 | 0 | 3 | 1000 | 1000 | 0 | 1.466e-6 | 6.082e-6 |
| OutcomeAxis | 5 | 5 | 0 | 500 | 500 | 0 | 7.683e2 | 7.683e2 |
| Sister | 3 | 0 | 3 | 500 | 1000 | 0 | 1.051e-6 | 4.298e-6 |

No cascade terminus on any model and no label reverted at the fifth numeric checkpoint, at either commit. The base's one decline is the sister model's, and at head there is no decline at all: the three models carrying no replenishment clamp pay no factorisation over the whole run, and the outcome-axis model pays exactly what it paid before.

**Table (The width sweep at its widest setting, 3200 labels, base against head)** · `tab:assayer:restructure-sweep`

| Model | Base rebuilds | Base declined | Head rebuilds | Head measurements | Head alarms | Base interval | Head interval | Base residual | Head residual |
| --- | --- | --- | --- | --- | --- | --- | --- | --- | --- |
| Anchor | 8 | 0 | 0 | 8 | 0 | 400 | 400 | 4.575e-12 | 6.562e-11 |
| Operational | 19 | 16 | 0 | 8 | 0 | 100 | 400 | 1.208e-6 | 9.268e-6 |
| OutcomeAxis | 12 | 0 | 12 | 0 | 0 | 400 | 400 | 4.355e-5 | 1.161e-5 |
| Sister | 19 | 16 | 0 | 8 | 0 | 100 | 400 | 1.013e-6 | 7.147e-6 |

**This is the reading's own question answered.** The two models this report found declining nine rebuilds in ten and held at the adaptive floor by those declines alone now run no rebuild at all over thirty-two hundred labels, raise no alarm, and sit four times off the floor. Nothing was weakened to achieve it: the adoption test is unchanged, and the declines that used to happen no longer happen because the rebuilds that used to be declined are no longer run. The outcome-axis model's rebuild count is identical at both commits, which is the control — it rebuilds on its floored-count arm and on its clamp's contribution, and neither of those moved.

**Table (What the sweep's whole-run maxima say, base against head)** · `tab:assayer:restructure-sweep-maxima`

| Setting | Labels | Base reading | Head reading | Base residual | Head residual |
| --- | --- | --- | --- | --- | --- |
| width-400 | 1800 | 5.5062e3 | 5.5062e3 | 7.8632e-7 | 3.7546e-6 |
| width-700 | 2700 | 2.7847e2 | 2.7847e2 | 1.5459e-5 | 1.4334e-5 |
| width-1000 | 3200 | 2.7794e2 | 2.7794e2 | 4.3550e-5 | 1.1610e-5 |

The reading is identical at every width to every printed digit, which is the strongest thing either instrument says here: the restructure removed forty-six factorisations from this run and moved the quantity the monitor exists to report by nothing at all. The residuals differ by single figures in the direction expected of a model that rebuilds less often, and every one of them stands three orders under its own threshold.

**Table (Wall time, per instrument, base against head)** · `tab:assayer:restructure-wall`

| Instrument | Labels | Base wall | Head wall | Base per hundred | Head per hundred |
| --- | --- | --- | --- | --- | --- |
| Dense schedule | 3400 | 122.9 s | 145.6 s | 3.61 s | 4.28 s |
| Sweep, width-400 | 1800 | 20.0 s | 22.1 s | 1.11 s | 1.23 s |
| Sweep, width-700 | 2700 | 71.1 s | 83.9 s | 2.63 s | 3.11 s |
| Sweep, width-1000 | 3200 | 123.6 s | 130.4 s | 3.86 s | 4.08 s |

**Two repairs the falsifier made, and neither was in the design it tested.** The first arrangement let all three cheap arms defer to the measurement's verdict, and the outcome-axis model — whose reading is almost entirely its replenishment clamp's, so its residual is always small — stopped the label path outright at width one thousand, its factorisation refused at pivot 1013 after thirty-one hundred labels. The spectral floor is derived from the spectrum and applied at the rebuild and nowhere else (`dec:posterior:spectral-floor`), so a model that never rebuilds never takes a floor, and the arms that were reporting exactly that staleness were being answered by a measurement that cannot speak to it. Those arms now go to the rebuild whatever the measurement says. The second repair is the clamp's own contribution: with the residual alone deciding, the same model ran its reading from `1.7e2` to `2.0e10` across a single rebuild, because that contribution is what the clamp has added since the last rebuild and an adopted rebuild is the only operation that absorbs it. It is now read against the same threshold the residual is, and calls for the rebuild without shortening the interval — the shortening being the half of the old behaviour this reading found wasteful, and the rebuild being the half that does the work.

**The refinement this reading raised as its second question is half taken and half vetoed.** The measurement carries its own resolution, derived rather than chosen: the reading is `‖BΣ − I‖_F` computed from a product whose entrywise error is bounded by `γ_p |B||Σ|` with `γ_p = pu/(1 − pu)` at unit roundoff `u`, so the reading's own resolution is `γ_p ‖ |B| |Σ| ‖_F`. There is no constant in it beyond the width's own, and it is read off the operands rather than off the answer — which matters, because a pair whose products cancel by orders before they reach their sum has a resolution orders above one whose products do not, and this reading's outcome-axis residual of `3.494e-1` at a measured `1.179e5` is exactly such a case. What is vetoed is letting that figure decide. A pair that has drifted far apart reports a resolution large enough to cover any residual at all, and in the failed run above every residual up to `3.0e2` against a threshold of `1.2e-3` read as being at its own resolution while the model went to pieces. The figure is published beside the residual, and the threshold keeps the verdict.

**Provenance.** Four runs on one box at one toolchain, each timed. The base runs are the width sweep and the dense schedule at the lane's base commit, extracted into their own tree and built against their own target directory. The head runs are the same two instruments at the head that implements the restructure. The dense schedule reports 3400 labels and the sweep 1800, 2700 and 3200 at its three widths; the figures in the tables are the last checkpoint of each run, read from the detailed precision health.
