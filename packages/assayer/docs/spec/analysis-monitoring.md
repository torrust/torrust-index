### Chapter (Health Monitoring) · `chap:spec:health-monitoring`

The system is asked to say when it is not working. This chapter is that apparatus: drift in the models' own predictions, discrimination against outcomes, the numerical health of the posteriors, statistical signatures of corrupted labels, the cross-layer quantities that reveal whether the layers are being used at all, and three scenarios that show the whole loop turning.

Every environment below states its requirement at full strength. The observability summary (`tab:monitoring:observability-summary`) indexes those environments rather than specifying an additional quantity.

**Invariant (Health monitoring reports and never acts)** · `inv:monitoring:report-only`

No diagnostic in this chapter modifies a model, a threshold, a rate, or a routing decision. Each computes a quantity, reports it, and stops.

The posture is deliberate and it is what makes the diagnostics safe to add. A monitor that acted would close a loop between the system's estimate of its own health and the estimates that health is measured on, and the resulting behaviour would be neither the specified behaviour nor a diagnosable departure from it. Reporting keeps the loop open and puts the host at the point where it closes.

**Table (Standardisation transition health)** · `tab:monitoring:standardisation-transition`

The cold standardisation ramp's progress is reported on both health surfaces and gates nothing (`inv:monitoring:report-only`).

| Reading | Compact assessment snapshot | Full health report |
| --- | --- | --- |
| Standardisation phase | Of the coordinate snapshot the result was scored in | At the moment the report was taken |
| Accepted cold-ramp observations | The count that snapshot represents | The count at the moment the report was taken |
| Model snapshot version | Present, fixing the coordinate state the result used | Present, as the report's own join key |
| Features at the variance floor | Absent; it is per-position | Present, as before |

The three phases are readings of one count rather than a second quantity (`alg:standardisation:batch-initialisation`). `WaitingForInit` is a count of zero and moments that are the class priors exactly; `Transitioning` is a positive count below $N_\text{init}$ and moments that are a mixture; `InService` is a count of $N_\text{init}$ with no cold-prior mass left. A reader holding the count holds the phase, and the phase is reported anyway, because a quantity a reader must reverse-engineer from the state it describes is one a reader will eventually reverse-engineer wrongly and then alert on.

The compact pair and the full pair answer different questions, and a host needs both. The compact pair is retrospective: it describes the coordinate system one result was computed in, which is what makes two results comparable or tells a reader that they are not. The full pair is current: it says where the ramp stands now, which is what an operator watching a cold start is asking. During a transition the two disagree by however many advances fell between them, and that disagreement is information rather than an inconsistency.

Queue refusal and contention counters remain degradation diagnostics and are not read as accepted observations. A refused observation advances no count, so the gap between requests served and observations accepted is exactly what those counters report; a reader who added them to the accepted count would be counting each skipped observation twice and would place the ramp further along than it is.

The phase and the accepted count are stored state and are carried on the compact assessment snapshot and the full health report exactly as the table specifies. Inferring the phase from variance floors is invalid because that inference cannot separate a genuinely small observed variance from a position the ramp has not moved yet.

**Algorithm (Per-model drift accumulators)** · `alg:monitoring:drift-cusums`

Each Core model — operational, sister, anchor, and each outcome axis model — maintains two one-sided accumulators over the residual between what it predicted and what the label reported:

$$S^+_t = \max\big(0,\; S^+_{t-1} + (r - \hat{\rho}(\phi)) - \kappa_\text{drift}\big)$$

$$S^-_t = \max\big(0,\; S^-_{t-1} + (\hat{\rho}(\phi) - r) - \kappa_\text{drift}\big)$$

Here $r$ is the model's training target (`def:risk:target`), or the axis target for an axis model (`def:axis:training-target`), and $\hat{\rho}(\phi)$ is the prediction made at **assessment time** and stored in the pending entry (`def:runtime:pending-entry`) rather than a prediction recomputed after the update. The distinction matters: a post-update prediction would compare the model against a target it has already moved towards, and the accumulator would understate every drift it exists to catch.

The noise allowance $\kappa_\text{drift}$ is what the accumulator forgives before it begins to accumulate, at a default of $0.1$. It sets the sensitivity of the whole mechanism: too small and ordinary noise accumulates, too large and real drift is absorbed silently.

The recursions are updated from the assessment-time prediction, and both the allowance and reset threshold are read from `MonitoringConfig`. The allowance boundary is inclusive and the reset boundary is strict.

**Table (Running drift diagnostics)** · `tab:monitoring:drift-diagnostics`

| Diagnostic | Computation | What it detects |
| --- | --- | --- |
| Mean absolute residual | Running average of $\lvert r - \hat{\rho}(\phi) \rvert$ | Overall prediction quality |
| Steps since reset | Counter since the last accumulator reset | How long drift has been accumulating |
| Residual sign | Running average of $\text{sign}(r - \hat{\rho}(\phi))$ | Sustained directional bias |

The three answer different questions and are reported together because a reader needs all three to interpret any one. A large mean residual with no sign bias is noise; a small mean residual with a persistent sign bias is a model drifting in one direction and is the more serious finding. The step counter supplies the duration that turns either into a rate.

**Table (The reset mechanisms)** · `tab:monitoring:drift-resets`

| Trigger | Condition | Effect |
| --- | --- | --- |
| Automatic | $S^+ > h$ or $S^- > h$, default $h = 10$ | Reset the accumulators and report a drift event |
| Host-initiated | A manual reset request | Reset the accumulators |
| Post-lifecycle | After a Sentinel or axis lifecycle event | Reset the accumulators for all models |
| Post-calibration | After a calibration shift exceeding $0.1$ | Reset the accumulators for the affected regime |

Three of the four resets exist because the residual's meaning changed rather than because the drift resolved. A lifecycle event alters the feature space the prediction was made in (`alg:registry:sentinel-registration`), and a calibration shift alters the scale the residual is measured on (`alg:platt:drift-integration`); in both cases the accumulated evidence is about a quantity that no longer exists, and carrying it forward would report a drift that is really a change of coordinates.

A drift flag derived from these accumulators belongs in the compact health snapshot, so that a host reading a single assessment can see that the model behind it is drifting (`schema:output:health-snapshot`). The automatic reset reports a drift event, and the compact flag rises when any accumulator reaches half the automatic-reset threshold so an assessment can report the approach before the reset clears it.

**Algorithm (Discrimination by rank)** · `alg:monitoring:auc`

At each calibration refit (`tab:platt:refit-cadence`) the Core computes rank discrimination from the prediction-outcome pairs held in the calibration buffer (`constr:platt:buffer`). Four figures:

The aggregate figure is computed over all buffer entries. The sister-regime and anchor-regime figures are computed over the entries whose blend weight falls below and at or above $0.3$ respectively, which separates the two regimes the blend actually operates in (`def:risk:subspace-blend`). Each is reported only when its partition holds at least $N_\text{auc,min}$ positive and $N_\text{auc, min}$ negative outcomes, default twenty, because a rank statistic over fewer is noise with a decimal point.

The recent figure is computed over the most recent $N_\text{recent}$ labels, default five hundred. When it departs from the aggregate figure by more than $0.1$ a **discrimination instability flag** is reported. The recent figure is the faster signal and the noisier one; the aggregate lags because the buffer spans about ten days at the reference rate (`cav:limitation:auc-lag`).

The computation uses rank discrimination with tie correction, both regime partitions, and the instability flag, with both configured sample gates. The recent window remains a fixed count rather than a proportion of the buffer, so it holds the constant span the figure's interpretation assumes instead of growing and shrinking with the buffer.

**Definition (Top-decile lift)** · `def:monitoring:top-decile-lift`

$$\text{lift}_{10} = \frac{\text{precision at the top decile}}{P_{+,\text{buffer}}}$$

The precision among the highest-scoring tenth of the buffer, divided by the buffer's own positive rate. It answers the question a rank statistic does not: not whether the ordering is right overall, but whether the top of it is worth acting on, which is the part of the ordering a host actually intervenes on.

**Definition (Per-axis correlation)** · `def:monitoring:axis-correlation`

For each active outcome axis carrying at least the minimum number of reported values, the linear correlation between the axis predictions stored at assessment time and the values the host later reported. It is the axis analogue of rank discrimination, and it is a correlation rather than a rank statistic because axis outcomes are continuous where the risk target is binary (`def:axis:training-target`). It is computed per axis from the calibration rows and reported.

**Table (Interpretation bands)** · `tab:monitoring:interpretation`

| Metric | Good | Moderate | Weak | Near-chance |
| --- | --- | --- | --- | --- |
| Rank discrimination | $\geq 0.80$ | $[0.65, 0.80)$ | $[0.55, 0.65)$ | $< 0.55$ |
| Top-decile lift | $\geq 5$ | $[2, 5)$ | $[1.5, 2)$ | $< 1.5$ |
| Axis correlation | $\geq 0.6$ | $[0.3, 0.6)$ | $[0.1, 0.3)$ | $< 0.1$ |

The bands are reading guidance and fix nothing. They exist because the three metrics have genuinely different scales, and an operator who reads a correlation of $0.5$ as though it were a rank discrimination of $0.5$ draws the opposite conclusion from the correct one.

**Algorithm (Empirical coverage of the reported uncertainty)** · `alg:monitoring:empirical-coverage`

The two-level guarantee establishes that reported uncertainty is conservative in sign — wider than a fixed-model posterior — but bounds neither the magnitude of that conservatism nor its stability (`thm:gaussian:two-level-guarantee`). Every downstream consumer of uncertainty inherits the effective width: the blend weight, the crossover intervals (`def:landscape:credible-intervals`), the fragility flip probabilities (`def:fragility:definition`), and the exploration ranking (`def:guidance:risk-informative`). An unmeasured inflation therefore reorders exploration queues and diffuses decision boundaries without announcing itself anywhere. This diagnostic measures it directly.

No additional per-request state is needed. The calibration buffer already stores the estimate, its uncertainty and the outcome per entry.

For each buffer entry, at the current regime's calibration parameter (`def:platt:objective`), form the probability-scale prediction and its uncertainty (`def:risk:probability`) and compute the standardised residual:

$$z_i = \frac{y_i - \hat{p}_i}{\sigma_{\hat{p},i}}$$

Two coverage fractions are then reported — the empirical fractions of entries with $\lvert z_i \rvert < 1$ and $\lvert z_i \rvert < 1.96$. Under calibrated Gaussian uncertainty these would sit near $0.68$ and $0.95$; the guarantee tells us not to expect exact calibration (`cav:gaussian:fixed-model-departure`), so the diagnostic measures the departure rather than testing a hypothesis. From the same residuals comes the single actionable number, the inflation factor: the $0.975$-quantile of $\lvert z_i \rvert$ divided by $1.96$, which is the factor the uncertainty would need rescaling by to achieve nominal coverage.

| Inflation factor | Interpretation | What a host may do with the intervals |
| --- | --- | --- |
| $< 1.3$ | Approximately honest | Intervals and fragility are quantitatively trustworthy |
| $1.3$–$3$ | Conservative, as the guarantee predicts | Intervals interpretable as bounds; fragility overstates flip risk modestly |
| $> 3$ | Grossly conservative | Widths and exploration ranking dominated by the artefact; read as a convergence-state signal |

The diagnostic is report-only and never rescales the uncertainty (`inv:monitoring:report-only`). Because outcomes are binary the per-entry residual is coarse and coverage is meaningful only in aggregate, so the minimum sample gate is the calibration minimum, per regime (`req:platt:minimum-samples`).

It is computed at each calibration refit and reported, with the inflation factor also carried on the compact health snapshot.

**Definition (The synchronisation error)** · `def:monitoring:synchronisation-error`

$$\epsilon_\text{sync} = \lVert B\tilde{\Sigma} - I \rVert_F$$

The Frobenius norm of the residual between the precision matrix, its maintained inverse, and the identity — a single number over the whole matrix, and the quantity whose growth means the two representations have drifted apart. The quantity this definition fixes is that whole reading, and it is what the health surface publishes under this name. Its threshold $\epsilon_\text{sync,thresh}$, default $10^{-6} \cdot p$, is the value above which a reading is abnormal at that model's width. What the recomputation cadence tests against the threshold is not the whole reading but each of its two components separately, as they are separated below: a residual over the threshold calls for a recomputation and shortens the interval, and a prior-induced component over it calls for a recomputation without shortening, because that component is one no interval reduces and one only a recomputation absorbs (`dec:posterior:adaptive-cadence`). The reading and the threshold are this definition's; which component of the reading the cadence consumes is that decision's, and the two are stated apart because a definition that folded the cadence's choice into the quantity would have to be rewritten whenever the choice moved.

This is the definition's only site in the document. The Gaussian chapter's monitoring procedure cites it rather than restating it (`alg:gaussian:synchronisation-monitor`), and the single site is deliberate: a norm over a matrix and a maximum over that matrix's diagonal are different quantities that a restatement can silently interchange, and the document protects against that by defining the quantity once.

**The reading carries its own resolution, and a residual below that resolution is not a measurement.** The reading is computed by forming the product $B\tilde\Sigma$ and taking the Frobenius norm of its distance from the identity. The standard entrywise bound for a floating-point matrix product at width $p$ is $\lvert \operatorname{fl}(B\tilde\Sigma) - B\tilde\Sigma \rvert \le \gamma_p \lvert B \rvert \lvert \tilde\Sigma \rvert$ with $\gamma_p = pu/(1-pu)$ at unit roundoff $u$, so the reading carries an absolute uncertainty of at most $\gamma_p \lVert\, \lvert B \rvert \lvert \tilde\Sigma \rvert \,\rVert_F$. That is the reading's resolution, and it is reported beside the reading. Two properties make it the right bar for the residual below. It is read off the *operands* and not off the answer, so a pair whose products cancel by orders before they reach their sum reports a resolution orders above one whose products do not — which is the case a competitive geometry produces and the case a figure scaled from the answer would misjudge in both directions. And it introduces no constant of its own: $\gamma_p$ is the width's, and the operand magnitudes are measured. A residual at or below this level is the rounding of the subtraction that produced it rather than evidence of drift, and is reported as being at resolution. It is reported and not acted on. A pair that has drifted far apart reports a resolution large enough to cover any residual at all, so a resolution is not permitted to overrule the threshold: doing so would fall silent exactly where the reading had stopped meaning anything.

**The reading has two components and they have different meanings, and both are computed and reported.** One is the rounding the incremental maintenance accumulates, which is what the monitor exists to catch and what a recomputation removes. The other is put there by the prior: the replenishment clamp raises a diagonal entry on every label at which a coordinate has decayed below the floor (`req:gaussian:prior-replenishment-floor`), and the maintained covariance does not follow that addition, so the tracked pair disagrees by the clamp's contribution seen through the covariance — $\lVert \operatorname{diag}(c)\tilde\Sigma \rVert_F$ for a clamp contribution $c$, exactly rather than approximately, since under decay the precision matrix is its decayed self plus exactly that contribution. The contribution that belongs in that expression is the one accumulated since the last adopted recomputation, not the one the model has accumulated over its life: a recomputation takes the covariance from the precision matrix the clamp has already raised, so everything added before it is absorbed into the pair and is no longer a disagreement to attribute. The two coincide only on a model that has never recomputed. On a model with coordinates resting at the floor the second component dominates the first by orders, it scales with the recomputation interval rather than with the width, and no recomputation reduces it below one interval's worth. The spectral floor has no term here, and that is a consequence of where it is applied rather than of its size: it is added to the precision matrix at the recomputation that is rebuilding the covariance from that same matrix (`dec:posterior:spectral-floor`), so the pair it leaves is consistent by construction and it contributes nothing for the monitor to read.

That component is conservative in direction — the covariance is wider than the inverse precisely along the directions the model has no evidence for — so it is not a fault, and it is reported rather than removed. What is no longer true is that it is indistinguishable from drift. Both components are quantities the model can compute, since the clamp's contribution is tracked per coordinate, and the monitor computes both at each recomputation and publishes three figures per model: the measured reading $\epsilon_\text{sync}$ as defined above, the part of it the prior put there, and the residual, which is the measured reading less that part, floored at zero. The maxima of all three across the models are carried on the compact health tier. The residual is the quantity the cadence acts on (`dec:posterior:adaptive-cadence`), which is what ends the behaviour this definition previously recorded as a consequence: a model with exhausted coordinates no longer shortens its interval to the floor and stays there on a quantity shortening does not remove. The threshold above is unchanged and is applied to the residual; the measured reading remains what this definition fixes, and is what the health surface publishes under the name. One consequence is worth stating for whoever reads the surface: on a model with coordinates at the floor the measured figure is large and grows with the recomputation interval, so lengthening the interval raises it. That is the prior's contribution scaling and not a degradation — the reading is $\sqrt{n}(1-\gamma^k)\gamma^{-k}$ for $n$ floored coordinates at forgetting rate $\gamma$ over an interval of $k$ labels, in which the floor's own value cancels — and it is conservative in direction. An operator watching for drift reads the residual (`test:crate:the-studys-reading-is-the-prior-and-its-residual-is-rounding`).

**Definition (The condition estimate, and the condition number)** · `def:monitoring:condition-number`

$$\rho(B) = \max_j B_{jj} \,/\, \min_j B_{jj} \qquad \kappa(B) = \lambda_\text{max}(B) \,/\, \lambda_\text{min}(B)$$

Two quantities, not one, and the distinction is what this definition exists to keep. The first is the ratio of the largest to the smallest diagonal entry: an $O(p)$ pass, read on every label. For a symmetric positive definite matrix it is a *lower bound* on the condition number and not an estimate of it, because the extreme diagonal entries are Rayleigh quotients at the coordinate directions and the extreme eigenvalues are the extrema over all directions. The gap is not a technicality at the widths a deployment reaches: the replenishment floor fixes the bound's denominator (`req:gaussian:prior-replenishment-floor`) while the quantity it bounds runs orders above it.

The second is the condition number itself, from the extreme eigenvalues. It costs a decomposition, so it is taken where a decomposition is already being paid for — at each recomputation of the covariance — and nowhere else (`alg:gaussian:condition-adaptive-recompute`). It is what the health surface reports as the model's conditioning, and it is absent until the model's first recomputation has measured one, because a condition number nobody has computed is not a condition number.

The cheap ratio is used relatively rather than absolutely. Growth of the ratio past the value recorded at the last recomputation, by a factor $\kappa_\text{growth}$ with a default of 2, calls for another one. It is not tested against a fixed threshold: the floor pins its denominator, so a fixed threshold on it is met permanently once any dimension rests at the floor, and permanently is not a trigger.

**Requirement (The bound is published as a bound)** · `req:monitoring:conditioning-bound-named`

Where both readings are published, each carries a name that says which it is, and the field promising the condition number is never filled from the ratio that bounds it. A lower bound published under the name of the quantity it bounds is a misreport a consumer cannot detect, because the two agree in shape, in units, and in the direction they move.

**Requirement (Dimensions at the prior floor)** · `req:monitoring:replenishment-floor`

At each recomputation the number of dimensions sitting at the precision floor is counted and reported (`req:gaussian:prior-replenishment-floor`). The count is the direct measure of how much of the feature space carries no evidence: a dimension at the floor has been replenished back to its prior and is contributing nothing but its prior to every estimate.

**Algorithm (Per-entity concordance)** · `alg:monitoring:per-entity-concordance`

For each entity carrying at least $N_\text{conc,min}$ labels, default five, the running fraction of its labels where the sign of the reported outcome agrees with the sign of the risk estimate. An entity whose concordance falls below the aggregate rank discrimination by more than $\delta_\text{conc}$, default $0.25$, is flagged.

The comparison is against the system's own aggregate performance rather than against a fixed threshold, which is what makes the diagnostic meaningful in a deployment whose discrimination is genuinely poor: it looks for entities that are anomalous relative to the system, not for entities the system happens to predict badly along with everything else. It is the sharpest available signal for label corruption concentrated on particular entities.

Label-time accumulation retains a bounded map keyed by entity, compares each sufficiently sampled running sign-agreement fraction to the aggregate AUC, and reports the flag, sample count and fraction. Insertion at capacity evicts the least recently observed entity and reports the lifetime eviction count; the diagnostic changes no model or decision.

**Definition (Alarm-outcome disagreement accumulators)** · `def:monitoring:alarm-outcome-cusums`

Two one-sided accumulators over the disagreement between what the Sentinels measure and what the host reports. The first accumulates where Sentinel alarm is high and the reported outcome is benign; the second where alarm is low and the reported outcome is adverse.

They are directional on purpose, because the two directions mean different things. Sustained accumulation in the first is consistent with a measurement layer firing on something the host does not consider harmful, or with adverse outcomes being reported as benign. Sustained accumulation in the second is consistent with harm the measurement layer cannot see. Either indicates a systematic disconnect between measurement and reporting, which is exactly the failure no single-layer diagnostic detects.

Both accumulators are maintained per Sentinel and per scoring axis. Label-time reporting crosses the frozen assessment-time alarm values with the eventual outcome, updates the two directional Page recursions with the configured thresholds and allowance below, and exposes the accumulated values without changing assessment or learning. Their boundaries are inclusive.

**Definition (Feature-stable outcome drift)** · `def:monitoring:feature-stable-drift`

The conjunction of two conditions: a rising prediction drift accumulator, and a feature distribution that is not moving. Together they say the outcome distribution is shifting while the inputs are not.

The conjunction is the diagnostic; neither half means much alone. It is consistent with label corruption and equally consistent with a legitimate regime change, and it cannot distinguish them (`cav:limitation:mimic-regime`) — which is why it is reported as a signature to investigate rather than as a finding. It is computed on the label path and reported, and its configured stability boundary is independent of the drift threshold.

**Definition (Quantile error concentration)** · `def:monitoring:quantile-concentration`

Partition the calibration buffer into ten quantiles of the risk estimate and report the ratio of the maximum quantile error to the mean quantile error. A ratio above three indicates miscalibration concentrated in part of the range rather than spread across it.

The distinction matters because a model can be well calibrated on average and badly calibrated exactly where a host acts. A concentrated error in the top quantile is an operational problem that an aggregate calibration figure conceals entirely. It is computed on the label path and reported.

**Caveat (What label-integrity monitoring cannot see)** · `cav:monitoring:integrity-scope`

The Core trusts labels unconditionally (`inv:guarantee:honest-uncertainty`), and the diagnostics above surface statistical signatures of corruption without ever establishing it. Four blind spots follow, and they are stated because a monitoring surface that is silent about its own scope invites the reading that silence means health.

Corruption that mimics a regime change is undetectable, because the signature is the same one a legitimate regime change produces (`cav:limitation:mimic-regime`). Patient targeted corruption that stays below the per-entity flagging thresholds is invisible by construction (`cav:limitation:patient-corruption`). A compromised investigation pipeline corrupts the sister model with no in-system mitigation whatever, because the ground-truth flag is trusted without qualification (`conv:eligibility:ground-truth`) and the Core has no means to audit it (`cav:limitation:investigation-pipeline`); this one is the host's responsibility and cannot be made otherwise. And every discrimination metric here lags the change it measures, the recent window being a faster and noisier signal rather than a solution (`cav:limitation:auc-lag`).

**Definition (Measurement maturity coverage)** · `def:monitoring:maturity-coverage`

$$\text{maturity coverage} = \frac{\sum_{c \,:\, \eta_c < \eta_\text{mature}} n_c}{\sum_c n_c}$$

The share of assessed traffic falling in cells whose measurement has matured, weighted by the traffic each cell carries. Below eighty per cent, the Core's per-Sentinel features are substantially influenced by immature measurement and should be read accordingly (`tab:keyspace:measurement-state`).

The traffic weighting is what makes the figure operational rather than descriptive: a deployment can have most of its cells immature and most of its traffic mature, and it is the second that determines whether the features being acted on are trustworthy.

Full health scans the current report indexes, applies the configured strict noise threshold to each cell, and weights the qualifying cells by their sample counts. It reports the resulting ratio as absent only when there is no assessed traffic to divide by.

**Definition (The resolution-utilisation ratio)** · `def:monitoring:resolution-utilisation`

$$\text{resolution utilisation} = \frac{\lvert \{c : n_{\text{elig},c} \geq N_\text{adequate}\} \rvert}{\lvert \mathcal{A} \rvert}$$

The fraction of cells whose spatial discrimination is backed by outcome evidence, where $N_\text{adequate}$, default twenty, is the eligible-label count at which a cell's discrimination is considered evidenced. Below fifty per cent, most of the system's spatial discrimination is not yet backed by Core-side outcome evidence.

The ratio carries a second duty beyond its own threshold. It is the instrument the detection chapter directs a host to consult before reading intervention effectiveness at all, because the divergence is uninterpretable exactly where eligible-label density is inadequate (`def:detection:divergence-scope`).

Full health scans the current Ledger entries against the configured adequacy threshold, counts equality as adequate, and divides by the same active-entry population.

**Definition (End-to-end feedback latency)** · `def:monitoring:feedback-latency`

$$T_\text{feedback} = T_\text{report} + T_\text{label} + T_\text{queue} + T_\text{publish}$$

The total time from a Sentinel's observation to that evidence being reflected in a published model, decomposed into the four stages that contribute to it. The decomposition is the point: the dominant term is almost always the human in the loop, and a deployment that responds to slow feedback by tuning its queue or its publication interval is optimising the two smallest terms.

All four stages are measured, and the first is measured as a lower bound.

$T_\text{report}$ runs from a Sentinel's observation to the report carrying it reaching the Assayer. The wire contract carries the age of a batch's oldest observation at the moment the Sentinel emitted the report, measured wholly on that Sentinel's own monotonic clock, and the report index retains it beside the arrival stamp. That age is a lower bound on the stage and never the stage itself: it stops at emission, and how long the host then held the report before handing it over is an explicitly unmeasured residual. Measuring the residual would mean comparing two machines' clocks, and the age's whole value is that it compares none. Every surface carrying the stage says at least, and the total inherits the qualification.

The three remaining stages are differences between instants on the Assayer's one injected monotonic clock, so each is a property of the deployment rather than of how busy the machine was. $T_\text{label}$ runs from that report's reception to the host's label arriving at the public label boundary — the human in the loop, normally the whole of the total. $T_\text{queue}$ runs from that arrival to the model owner taking the label up, which is the wait on the channel between two threads and is invisible unless the arrival travels with the label. $T_\text{publish}$ runs from the model owner taking it up to the new snapshot being stored.

An assessment reads several Sentinels, whose reports arrived at different moments. The stages are anchored to the earliest-arrived report that contributed an extraction, because the stage measures how long evidence waited and the longest wait among the contributors is the one the assessment carried. All four fold into one accumulator, one observation per published label, so they are smoothed over the same stream and the total is a total of one journey rather than a sum of averages taken over four different populations. A label that fails a numeric checkpoint publishes no model and contributes nothing: the journey it was on did not end. A label replayed from the journal contributes only the publish stage, its earlier instants belonging to a monotonic domain the restart ended. Full health reports the four stages, their total, and the worst report-stage lower bound across the reports the fleet currently holds; per-Sentinel health reports that Sentinel's own lower bound beside the arrival-measured report age, unsummed, because what separates the two is the unmeasured residual.

**Definition (Per-Sentinel weight mass)** · `def:monitoring:encoding-effectiveness`

For Sentinel $s$, let $\mathcal{S}_s$ be the feature positions in its own slot and let $w_j$ be the corresponding published operational model weights. Its weight mass is the mean absolute slot weight

$$I_s = \frac{1}{\lvert \mathcal{S}_s \rvert} \sum_{j \in \mathcal{S}_s} \lvert w_j \rvert.$$

The quantity is how much weight the model carries on the Sentinel's published features (`def:extraction:slot`), and that is the whole of what it says. It is not a reading of whether the outcome depends on those features, and the two must not be read off each other. Ridge weights on standardised coordinates that tell the outcome nothing do not settle at zero — they settle at noise scale — and a model whose fit leaves no residual has nothing left with which to shrink them further, so the figure a structureless Sentinel earns *rises* towards a working one's as the model converges. Driven against a perfectly predictive control on one labelled stream, a Sentinel fed a cryptographic hash of the request index read between $0.76$ and $1.05$ of the control's figure through nine thousand six hundred labels and $0.80$ at nineteen thousand two hundred; a Sentinel that had found nothing to split on at all settled near $0.30$ and stopped declining. A block carrying no weight is a genuine finding and this reading shows it. A block carrying weight is not thereby a block the outcome depends on.

What the outcome depends on is read at (`def:monitoring:slot-association`), and what removing the block would cost is read at (`def:monitoring:slot-contribution`). Both are taken over the same block as this one and neither is a function of the weights alone, which is what puts them out of reach of the convergence artefact above. The reading is computed from each Sentinel's own slot and reported per Sentinel on the health surface, beside those two.

An earlier design made a three-component heuristic the principal quantity:

$$E_s = \frac{1}{3}\Big[\underbrace{(1 - k_s / \text{cap}_s)}_{\text{rank headroom}} + \underbrace{\min(1, \text{coord conc}_s / \xi_\text{coord})}_{\text{coordination activity}} + \underbrace{(1 - P_s / \lvert \mathcal{A}_s \rvert)^+}_{\text{plateau efficiency}}\Big].$$

There $\xi_\text{coord}$ was a proposed coordination scale, and values below $0.1$ after several hundred batch reports were to indicate an encoding indistinguishable from random (`cav:limitation:silent-encoding`). That formula, its scale, its reading threshold, and its three-component semantics are superseded design history, not current diagnostic or configuration requirements. The legacy row is retired outright at (`entry:health:encoding-effectiveness`) rather than held open against an approximation of it, and the mean-slot-weight successor above is the public encoding-transparency reading in its place. A host with access to the Sentinel's own wavelet portrait still has a stronger structural signal: phase transitions with large baselines should fall at boundaries that mean something in the host's coordinate encoding, and repeated investment at boundaries with no domain interpretation says more than the aggregate diagnostic does.

**Definition (Per-dimension weight mass)** · `def:monitoring:dimension-informativeness`

Write $I(\mathcal{P})$ for the mean absolute published operational model weight over a non-empty set of feature positions $\mathcal{P}$:

$$I(\mathcal{P}) = \frac{1}{\lvert \mathcal{P} \rvert} \sum_{j \in \mathcal{P}} \lvert w_j \rvert.$$

The per-Sentinel reading above is $I_s = I(\mathcal{S}_s)$. For registered identity dimension $d$, let $\mathcal{D}_d$ be the $8 + 3m$ positions of that dimension's own block (`tab:keyspace:dimension-features`). Its weight mass is the same functional restricted to that block:

$$I_d = I(\mathcal{D}_d).$$

The reading carries the same meaning here as it does over a slot and inherits the same limit: it says how much weight the model holds on the dimension's own block and nothing about whether the outcome depends on that block. The two were driven apart directly. Two worlds were run whose traffic differed in exactly one thing — whether the entity a request arrived under was drawn from the half of the pool its outcome named, or from the whole pool by a hash of the request index — and the null dimension's figure sat between $67\%$ and $143\%$ of the keyed one's across the schedule, crossing it in both directions. A dimension's own association and contribution readings are (`def:monitoring:slot-association`) and (`def:monitoring:slot-contribution`) restricted to $\mathcal{D}_d$, exactly as this one is $I$ restricted to it.

Nothing is estimated or accumulated here that was not already published. The restriction is a grouping of weights the operational model already carries, taken at the block extents the published layout already names (`schema:dimension:map-record`), so a rebuild that moves a dimension's block moves the reading with it and no second copy of the layout can go stale against the first.

The functional has exactly one composition law, and it is worth stating because it is weaker than it looks. For disjoint $\mathcal{P}$ and $\mathcal{Q}$,

$$I(\mathcal{P} \cup \mathcal{Q}) = \frac{\lvert \mathcal{P} \rvert\, I(\mathcal{P}) + \lvert \mathcal{Q} \rvert\, I(\mathcal{Q})}{\lvert \mathcal{P} \rvert + \lvert \mathcal{Q} \rvert},$$

which follows from the sum over the union being the sum of the sums. So the per-dimension readings compose, weighted by position count, into the reading over the whole per-dimension family — and where exactly one dimension is registered the two coincide. What does not follow is any relation between a dimension's reading and a Sentinel's. The layout gives every block type a disjoint extent (`def:feature:vector-structure`), so $\mathcal{S}_s \cap \mathcal{D}_d = \emptyset$ for every Sentinel and every dimension: $I_s$ is not an average of per-dimension readings, and $I_d$ is not a share of a Sentinel's. The two are siblings under one recipe, asking the same question of two disjoint populations of positions, and neither refines the other.

The law is a property of averaging over positions and not of this reading in particular, so it is worth saying which of the three block readings inherit it and which does not. The association reading (`def:monitoring:slot-association`) does, over the same count weights and for the same reason: it too is a mean over the positions of a block. The contribution reading (`def:monitoring:slot-contribution`) does not, and no aggregate of it over blocks is published anywhere. What it measures is the cost of removing one block from a score every block contributes to, which is a joint quantity: two blocks carrying the same information each cost nothing to remove alone and cost the whole of it to remove together, so the count-weighted mean of their separate readings is not the reading over their union and there is no weighting that would make it one.

Three families of position are outside $\mathcal{D}_d$, and each is excluded for its own reason rather than by convention. The cross-dimension block aggregates across dimensions by maximum (`tab:keyspace:cross-dimension-features`), so it belongs to no single dimension; attributing it to each would count one set of weights once per registered dimension. The dimension's competitive indicators (`def:dimension:competitive-range`) are excluded because their count turns over with every restructuring, so folding them in would make the reading track set churn rather than encoding quality, where the block above is fixed-width for the life of the dimension; and because the host duty reads them separately, beside this reading rather than inside it (`req:keyspace:host-duties`), which a reading that had already absorbed them could not let it do — the separate reading being the cell weight mass tabulated with the identity-health metrics (`tab:monitoring:dimension-health`), the same fold over that other population. Interaction features are excluded on the same ground the per-Sentinel reading excludes them: an interaction position carries the weight of a product and is attributable to the operand pair, not to either operand alone (`sec:feature:interactions`). Each reading is over the block the layout gives its own entity, and over nothing it merely feeds.

Full health computes $I_d$ from the same published operational model the per-Sentinel reading is taken from and reports it per registered identity dimension, beside the per-Sentinel figure and under the same absence discipline: a dimension whose block the published map does not carry reports absence, while a dimension whose block carries no weight reports zero, because zero is the reading the host duty acts on. The producer path is held by (`test:crate:dimension-informativeness-reads-the-operational-dimension-block`), the separation of a weighted block from a silent one and the composition law by (`test:crate:dimension-informativeness-separates-a-weighted-block-from-a-silent-one`), and the export's treatment of absence by (`test:crate:dimension-informativeness-export-carries-measurement-and-absence`).

**Definition (Slot–outcome association)** · `def:monitoring:slot-association`

Let $\mathcal{B}$ be one entity's own block of feature positions — a Sentinel's slot $\mathcal{S}_s$ or an identity dimension's block $\mathcal{D}_d$. For each labelled request write $\hat\varphi$ for the standardised feature vector it was scored on and $y \in \{-1, +1\}$ for the sign of its outcome. Over a weighted window of such requests, let $r_j$ be the sample correlation between coordinate $j$ and the outcome, and let $\mathcal{V} \subseteq \mathcal{B}$ be the coordinates whose values varied over the window. The block's association is the mean absolute correlation over those coordinates, expressed as a multiple of the correlation a coordinate telling the outcome nothing would earn from a window of that length:

$$A(\mathcal{B}) = \frac{1}{\lvert \mathcal{V} \rvert} \sum_{j \in \mathcal{V}} \lvert r_j \rvert \Big/ \sqrt{\frac{2}{\pi n_\text{eff}}}, \qquad n_\text{eff} = \frac{(\sum_k v_k)^2}{\sum_k v_k^2}.$$

The divisor is the whole of what makes the reading legible. A coordinate that says nothing about the outcome still earns a sample correlation: over a window of $n$ its correlation is approximately normal about zero with standard deviation $1/\sqrt{n}$, so the expectation of its absolute value is $\sqrt{2/\pi n}$. That figure depends on the window and on nothing else — not on the block's width, not on the model, not on what the block is a block of — which is what lets one calibration serve every block on the surface and lets two blocks of different widths and different ages be read against each other. A block reading near $1$ is a block earning what noise earns.

The weights $v_k$ are the balancing weights the operational update carries (`def:weighting:balancing-weights`), decayed at that update's own combined per-label and time factor, so the stretch of traffic the reading describes is the stretch the operational weights describe. The effective sample size is the window those decayed weights amount to rather than a count of labels: under a constant stream at forgetting rate $\gamma$ it settles at $(1 + \gamma)/(1 - \gamma)$, and it is what the floor above is a function of. Neither reading is published until it reaches thirty, because below that the floor is an asymptotic statement about a quantity that has not yet earned it.

Coordinates that never varied leave the mean rather than entering it as zeros. A block is a fixed extent whose positions are occupied as its entity's traffic occupies them, and averaging a zero in for a position that held one value all window would report a mostly-idle block as quieter than its live coordinates are — which is the reading saying the block is uninformative when what is true is that most of it is unused.

The stored evidence is bounded by the block and not by the stream: three decayed moment vectors of the block's own width — the first moment of each coordinate, its second, and its cross moment with the outcome — and three scalars, being the decayed weight sum, the decayed sum of squared weights, and the decayed weighted outcome sum. The outcome's second moment is not among them because the outcome is a sign and its square is its weight. Summed over every block of a layout this is under three vectors of the full width; at the reference dimension it is some fifteen kilobytes. Nothing of the labelled stream itself is retained.

Composition follows the count-weighted law the weight-mass reading has (`def:monitoring:dimension-informativeness`), with the count of varying coordinates as the weight, because the reading is a mean over a block's positions exactly as that one is. Where two blocks are read over windows of the same effective length the floor is common to both and the calibrated multiples compose directly; where the windows differ the raw means compose and the calibration does not, which is the ordinary caution that two figures divided by different denominators do not average.

The alert band is three to fifteen times the floor. The band's lower edge is the side that matters and it does not move with the window: a null block earns the floor at every length, by construction. The upper side does move, because a genuinely carrying block's raw correlation is a property of its traffic while the floor shrinks as $1/\sqrt{n_\text{eff}}$, so a longer window lifts a carrying block's multiple and widens the margin rather than narrowing it. A deployment that shortens the window must re-read the upper edge before relying on it.

The producers accumulate on the model owner's thread at label time, before the label reaches any model, and the reduced reading rides the published health summary per Sentinel and per registered identity dimension under the same absence discipline the weight-mass reading keeps: a block that has not yet carried a window's worth of labels reports absence, and a block that has and earns the floor reports the floor's multiple, because that is the reading a host acts on.

Two conditions make a dimension readable. Its encoding must place entity identity where the competitive mechanism can partition it, because the block is a fold over the cells that mechanism finds: an encoding placing identity below the depth cutoff gives every entity one prefix at every depth the cutoff allows, and the block then describes one undivided population however the traffic is drawn. At least one outcome axis must also be registered, because a dimension's outcome history is its per-axis positions (`tab:keyspace:dimension-features`) and a block carrying none of them has no position an outcome is ever written to. Neither is a threshold to tune. Each is a way for a dimension to be unreadable that reads, on this surface, exactly as a dimension telling the outcome nothing — which is worth a host's knowing before it revises an encoding the reading was never able to see.

**Definition (Slot contribution)** · `def:monitoring:slot-contribution`

Over the same block $\mathcal{B}$ and the same weighted window, write $\rho = \hat\varphi \cdot \mu$ for the score the operational model gave a labelled request, $\rho_\mathcal{B} = \sum_{j \in \mathcal{B}} \hat\varphi_j \mu_j$ for the part of that score the block supplied, and $\ell(\rho, y)$ for the binary cross-entropy of the logistic read of a score against an outcome. The block's contribution is what removing it from the score would have cost, as a fraction of the loss the score carried:

$$C(\mathcal{B}) = \frac{\sum_k v_k \big[\ell(\rho_k - \rho_{\mathcal{B},k},\, y_k) - \ell(\rho_k, y_k)\big]}{\sum_k v_k\, \ell(\rho_k, y_k)}.$$

This is the removal cost itself and not a proxy for it, which is the whole reason it is the reading a retirement decision hangs on: the number a host reads is the number the deregistration would produce. The model regresses a sign rather than a probability, so the logistic is the monotone read that turns a score into one; it is applied to the full score and the ablated score alike, so it moves both by the same map and cannot manufacture a difference between them. A negative reading is a reading and not a defect: it says the block's coordinates were making the score worse.

The evidence is one decayed excess-loss accumulator per block and two scalars — the decayed loss the score carried and the decayed weight — which is three numbers per Sentinel and three per registered dimension. Both quantities are folded at label time from the score the request was actually given. That timing is not a convenience: after the rank-one updates the weights that produced the score are gone, and an ablation taken against the updated model would be an ablation of a score no request received.

The reading does not compose across blocks and no aggregate of it is published. The reason is stated with the composition law it declines to inherit (`def:monitoring:dimension-informativeness`): removal cost is joint, and two blocks that duplicate each other each cost nothing to remove alone.

The alert band is $0.10$ to $0.58$ of the score's loss. A converging model concentrates its reliance rather than spreading it, so a carrying block's contribution strengthens with convergence while a structureless block's tends towards zero.

The producers ride the same accumulation site as the association reading, and the reading is published per Sentinel and per registered identity dimension under the same absence discipline and the same evidence floor. A host uses contribution as a confirming reading per Sentinel and as context rather than a discrimination threshold per dimension. Both of its terms scale with the operational weights that formed the score, so a label count alone does not fix its convergence state; the association reading is invariant to that scale because it is a correlation divided by a floor depending on the window alone.

**Definition (The immature-cell count)** · `def:monitoring:immature-cells`

$$\text{immature cells} = \sum_s \big\lvert \{c \in \mathcal{A}_s : n_{\text{elig},c} < N_\text{material}\} \rvert$$

The fleet-wide count of Ledger cells holding fewer eligible labels than the materiality threshold, default one hundred (`data:ledger:attenuation`). It is the practical form of the materiality result: the count of cells whose Ledger features are attenuated towards uninformativeness because the evidence behind them is too thin to be material.

Full health scans every current Ledger entry against the strict threshold and reports both per-Sentinel and fleet-wide counts (`entry:memory:immature-count`). The materiality threshold has a second consumer, the per-assessment flag on the alarm summary, which applies this same strict comparison to the single entry that answered one request (`def:runtime:alarm-summary`). The comparison is shared; the two readings are not the same reading, and the two differences are worth stating so that neither is read off the other. The population differs: this count ranges over every entry the Ledger holds, the flag over the one entry the extraction routing reached. The predicate differs too, because the flag is a disjunction where this count is a single test — the flag raises on the attenuation condition as well, which this count does not take and which the fleet surface reports separately and traffic-weighted rather than counted (`def:monitoring:ledger-value`). So a deployment cannot recover how often the flag raises from this count, and a cell counted here is a cell the flag would raise on but not conversely.

**Definition (Ledger value realisation)** · `def:monitoring:ledger-value`

$$\text{value realised} = \frac{\sum_s \sum_{c \,:\, \bar{b}_c > \bar{\mu}_b + 0.5\sqrt{\bar{v}_b}} n_c}{\sum_s \sum_c n_c}$$

The share of assessed traffic falling in Ledger cells whose remembered adverse rate stands meaningfully above the fleet mean — that is, the share of traffic for which the Ledger is actually discriminating rather than merely present. The boundary is the theorem's own: a standardised departure of more than half a deviation from the standardisation mean (`thm:ledger:materiality`), and the moments are the ones held for that Sentinel's own bad-rate feature, each Sentinel occupying a distinct standardisation position (`def:extraction:slot`).

Reported alongside it, the attenuation-limited fraction: the share for which the Ledger would discriminate but for insufficient evidence. A cell counts towards it when it is not realising its value, its true adverse rate does clear the same boundary, and the attenuation implied by its own eligible arrival rate leaves the attenuated rate at or below that boundary (`data:ledger:attenuation`). The two conditions are read off separate stored evidence — the raw undecayed adverse and eligible counts for the first, the arrival window for the second (`tab:ledger:entry-state`) — because the theorem holds that neither suffices alone, and an attenuated average cannot stand in for both. A cell whose true rate is ordinary is therefore not attenuation-limited however sparse its labels, and a cell whose arrivals are dense enough to preserve its excess is not attenuation-limited however high its true rate.

The two shares are disjoint by construction, a realising cell being excluded from the attenuation-limited count before its evidence is consulted, so their sum stays inside the whole.

| Value realised | Attenuation-limited | Reading |
| --- | --- | --- |
| $> 60\%$ | $< 20\%$ | High traffic; the Ledger provides broad spatial discrimination |
| $20$–$60\%$ | $20$–$50\%$ | Mixed; discrimination in hotspots, fallback elsewhere |
| $< 20\%$ | $> 50\%$ | Low traffic; Ledger features negligible, detection via measurement and identity (`tab:ledger:low-maturity-detection`) |

The pair is diagnostic where either alone is not: a low realised value with a low attenuation-limited fraction means the Ledger has nothing to say, while a low realised value with a high attenuation-limited fraction means it would have something to say given evidence, and those two call for opposite responses.

Full health scans every current Ledger entry under the read guard, weights it by its own assessed traffic, and reports the pair per Sentinel and fleet-wide (`entry:memory:materiality`).

**Table (Per-dimension identity health)** · `tab:monitoring:dimension-health`

| Metric | What it reveals |
| --- | --- |
| Competitive set size | Whether the dimension is resolving into cells at all |
| Competitive set change rate | Churn, which resets per-cell measurement state |
| Per-cell indicator weight mass | What weight the model carries on the dimension's cells |
| Competitive cell depth distribution | Whether resolution is uniform or concentrated |
| Coverage fraction | The share of traffic falling in competitive cells |
| Active indicator count distribution | How many indicators fire per assessment |

Reported per registered identity dimension (`def:registry:identity-dimension`), these are the only view of whether a declared dimension is earning its width in the feature vector. The convergence snapshot carries the set's size and its change rate; the current competitive index produces the depth distribution; and the assessment path records how many indicators it actually routed through, which full health publishes as the lifetime distribution and folds into the coverage fraction.

The coverage fraction is read off that same tally rather than counted a second time. The tally already separates the two populations the row asks about — its zero key counts exactly the assessments that matched no cell, every other key one that matched at least one — so the share is the complement of the zero key's weight. Two counters incremented at one site can come to disagree; one counter read two ways cannot, and the shape row and the coverage row are then consistent about the same traffic by construction. The share is over the dimension's whole life, matching the tally it comes from: windowing is already represented here by the churn reading, which is smoothed precisely because set membership turns over. Absence is before any traffic has been assessed against the dimension, and zero is traffic that arrived and matched nothing — opposite findings for a host, one saying the set catches nothing and the other that there is nothing yet to catch.

The weight-mass row is the block reading's fold taken over a different population of positions: the ones the layout gives the dimension's competitive indicators, one per cell (`def:dimension:competitive-range`). It says what weight the model carries there and, like its sibling, nothing about whether the outcome depends on it. It is absent where the layout names no cell position for the dimension — a set that has not formed has nothing to fold over — and zero where the positions it names carry no weight.

The dimension's weight mass (`def:monitoring:dimension-informativeness`) is reported on the same surface and is deliberately not a seventh row here, and so are the dimension's two legibility readings (`def:monitoring:slot-association`), (`def:monitoring:slot-contribution`). Every metric above is a reading of the competitive set — its size, its churn, its shape, the traffic it catches, the weight the model puts on its cells — while those three are readings taken over the dimension's own block of the feature vector, which is why they are defined with the other monitoring quantities rather than tabulated with these. The two weight-mass readings are therefore siblings across the boundary rather than one figure reported twice: the block is fixed-width for the life of the dimension and the cell positions turn over with every restructuring, so a dimension can carry weight in the one and none in the other, which is exactly the case reading them separately is meant to catch.

Neither of them answers whether the model is using the dimension, and the table does not claim they do. Weight mass is what the model carries, which a block predicting nothing earns about as readily as one that predicts (`def:monitoring:dimension-informativeness`); use is what the association and contribution readings measure. This table and the block readings are therefore read together: the duty that sends a host here sends it to those as well, and it is the association reading among them that the duty's first term names (`req:keyspace:host-duties`). The association reading is the trigger, and the two beside it are context a host weighs rather than figures it acts on alone.

**Table (The observability metrics summary)** · `tab:monitoring:observability-summary`

| Metric | Scope | When meaningful | Alert threshold |
| --- | --- | --- | --- |
| Maturity coverage | System-wide | Once the warm-up pipeline has processed the top cells | $< 80\%$ |
| Resolution utilisation | System-wide | After roughly $N_\text{adequate}$ labels per cell | $< 50\%$ |
| Feedback latency | System-wide | Always | Report stage above a tenth of the total |
| Per-Sentinel weight mass | Per-Sentinel | After the operational weights have learned | No ruled alert threshold, and none is available: the reading does not separate a carrying block from a structureless one |
| Per-dimension weight mass | Per identity dimension | After the operational weights have learned | No ruled alert threshold, on the same ground; read with the dimension's cell weights (`req:keyspace:host-duties`) |
| Slot–outcome association | Per-Sentinel and per identity dimension | Once the block has carried a window of labels | Below three times the null floor, sustained; the same edge applies at both scopes |
| Slot contribution | Per-Sentinel and per identity dimension | Once the block has carried a window of labels | Below $0.10$ of the score's loss, confirming a sustained association alert per Sentinel; contextual per dimension (`def:monitoring:slot-contribution`) |
| Immature Ledger cells | System-wide | After initial warm-up | Deployment-dependent |
| Ledger value realisation | System-wide | After roughly $N_\text{adequate}$ labels per cell | $< 20\%$ in high traffic |
| Uncertainty inflation | System-wide | After the first calibration refit | $> 3$ in steady state |

The table is the operator's index into the cross-layer metrics. The feedback-latency row reads against the total, and the total is a lower bound: the alert compares the report stage against a total that is itself at least what it says, so a deployment crossing the threshold has certainly crossed it and one sitting below it may not have. The per-Sentinel weight-mass row names the canonical reading (`def:monitoring:encoding-effectiveness`).

The per-assessment Ledger immaturity predicate is separate from this table's Ledger rows. The evidence its second arm needs is stored on every entry (`tab:ledger:entry-state`), the attenuation floor it compares against is configured (`def:config:attenuation-materiality-floor`), and the assessment path reads the entry the flag is a statement about (`def:runtime:alarm-summary`). The flag is per assessment and this table is cross-layer, so nothing here aggregates it.

**Scenario (Restriction-induced spatial atrophy)** · `scenario:monitoring:atrophy`

The host restricts a region; traffic there falls; the measurement features that described it normalise because there is nothing left to measure. The risk estimate for that region drifts towards the prior, and a naive reading concludes the region became safe.

Recovery is bounded by the convergence cascade (`bound:resource:convergence-budget`) plus the Ledger's time-indexed decay at a twenty-nine-day half-life (`def:ledger:time-decay`). Throughout, coverage is continuous rather than absent: ancestor routing supplies features from the surviving parent entry, and the measurement-only anchor keeps the direction right while the sister is starved (`prop:risk:anchor-measurement-only`).

**Scenario (Source displacement)** · `scenario:monitoring:displacement`

A source moves, and the competitive set rebalances as importance shifts from the cells it left to the cells it entered (`alg:keyspace:cell-entry`). The old cells lose their standing and the new ones acquire it (`alg:keyspace:cell-exit`). Between the move and the rebalance there is a detection gap, bounded by the same cascade as the atrophy scenario, and ancestor routing again provides continuous coverage across it. The scenario differs from atrophy in cause and not in shape: the system is briefly describing a distribution that has moved.

**Scenario (The restriction-induced starvation loop)** · `scenario:monitoring:starvation`

The loop that closes: a cell is flagged adverse, the host restricts it, the restriction stops eligible labels arriving from it, and the absence of contradicting evidence leaves the flag standing. The cell's reputation sustains itself on the consequences of its own reputation (`alg:valence:contamination-loop`).

Time-indexed Ledger decay breaks it regardless of label flow, because it acts on elapsed time rather than on evidence and therefore acts precisely when no evidence is arriving (`tab:ledger:loop-timescales`). Host investigation accelerates the resolution rather than waiting for it (`conv:eligibility:ground-truth`), and the starvation-relief guidance names the cells where investigation would pay most (`def:guidance:starvation`). The Ledger's own starvation score is the one mechanism here that acts rather than waits (`def:ledger:starvation-score`).

This scenario is also the structural template for the Companion's policy-dependence loop (`cav:limitation:challenge-policy-loop`): better targeting shifts the challenged population, the tracker re-converges over its decay horizon (`def:companion:challenge-decay`), and the shifted estimate feeds back into targeting. That loop is slower and damped, but it has this shape — a host-mediated loop bounded by a decay timescale rather than eliminated.

**Remark (What an operator should take from the scenarios)** · `rem:monitoring:operator-message`

The loop is stable under all three scenarios and recovery is bounded in each. The system provides continuous coverage throughout every recovery period — at degraded resolution, through ancestor routing and the measurement-only anchor, rather than at zero coverage — and that distinction is the one an operator should carry away, because the instinct during a recovery is to intervene as though the system had gone blind.

The three scenarios share a structure worth naming. Each is a loop that passes through the host, and each is bounded by a decay timescale rather than by anything the Core decides (`inv:guarantee:feed-forward`). None of them is eliminated by the design; all of them are made finite by it, and the difference between a bounded loop and an eliminated one is what the health report exists to keep visible (`chap:spec:output-structures`). The parameters governing every threshold in this chapter are tabulated with the configuration surface (`tab:config:monitoring`).
