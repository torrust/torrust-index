## Part (Reference) · `part:spec:reference`

Parts I through VII say what the system is, how it computes, how it runs and what follows from it. Part VIII is where a reader looks something up. Three chapters: the configuration surface, in tables; the output structures, which are defined elsewhere and gathered here as pointers; and the document's two registers of record, the formal properties it guarantees and the limitations it discloses.

A reference Part earns its keep by being checkable. Every number below is stated once, in the table that owns it, and the chapters that spend it cite the table rather than repeating the value — which is the only arrangement under which two copies of a constant cannot drift apart.

### Chapter (Configuration) · `chap:spec:configuration`

Sixteen tables and roughly ninety parameters: every value a host may set, and every default the system assumes when the host sets nothing. The chapter fixes no behaviour of its own. Each table is the configuration face of an environment specified earlier, and the constraint column is part of the specification — a value outside it is a configuration error, not a tuning choice.

The chapter's purpose is that its numbers are right. Every row names a constant or an exposed field, and a mechanical projection checks those relationships rather than relying on a hand count.

**Convention (Citing a parameter)** · `conv:config:parameter-citation`

A parameter is a row, and a row is not an environment. Nothing in this document cites a parameter directly; a chapter that fixes a parameter's meaning cites the table that carries its value, and the table cites back to the environments whose parameters it tabulates.

The two directions do different work and both are needed. Outward from the table, the citation says what a number means: a reader who finds a rate of $0.9998$ follows the table's citation to the environment that explains what is being forgotten and how fast. Inward from a chapter, the citation says where the number lives, so that a chapter discussing a bound never restates the bound's value and can never disagree with it. What no citation does is name a row, which is why a parameter's identity is always the pair of its table and its symbol.

#### Core configuration · `sec:config:core`

Thirteen tables covering the Core's own surface, ordered as the models that consume them appear in the earlier Parts. The Derivation Function's configuration, the optional rendering layer's, and the Companion's follow the division, because none of the three is Core state.

**Table (Core risk model parameters)** · `tab:config:risk-model`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_\text{opr}$ (operational forgetting) | 0.9995 | $(0, 1)$ | (`tab:risk:forgetting-rates`) |
| $\gamma_\text{inh}$ (sister and anchor forgetting) | 0.9998 | $(\gamma_\text{opr}, 1)$ | (`tab:risk:forgetting-rates`) |
| $\lambda_\text{prior}$ (prior precision) | 0.1 | $> 0$ | (`def:risk:model-triple`) |
| $\lambda_\text{floor}$ (prior replenishment) | 0.001 | $> 0$, $\leq \lambda_\text{prior}$ | (`req:gaussian:prior-replenishment-floor`) |
| $c$ (leverage safety factor) | 5 | $\geq 1$ | (`alg:update:sherman-morrison`) |
| $N_\text{recompute}$ (Cholesky recomputation interval) | 1,000 | $\geq 100$ | (`alg:gaussian:condition-adaptive-recompute`) |
| $\kappa_\text{growth}$ (conditioning-growth trigger) | 2 | $> 1$ | (`alg:gaussian:condition-adaptive-recompute`) |
| $\varepsilon_\text{Schur}$ (Schur regularisation) | $10^{-4}$ | $> 0$ | (`alg:gaussian:regularised-schur`) |
| $\kappa_\text{posture}$ (host posture ceiling on a removed block) | $10^8$ | $> 1$ | (`alg:gaussian:regularised-schur`) |
| $\gamma_\kappa$ (feature compression rate) | 0.9998 | $(0, 1)$ | (`def:risk:compression-scale`) |
| $w_\text{ceiling}$ (importance weight ceiling) | 100 | $\geq 1$ | (`def:weighting:balancing-weights`) |
| $P_{+,0}$ (initial positive-valence rate) | 0.5 | $(0, 1)$ | (`def:weighting:initial-value`) |

The two class-rate trackers are not configured here. Their decay rates are the model forgetting rates above — the global tracker takes the operational rate and the eligible tracker the inherited one — because a tracker that forgot at a different speed from the model it weights would balance the model against a population the model no longer sees (`tab:weighting:trackers`).

The two Schur parameters are worth one qualification: the relative epsilon is tabulated here, and the absolute term is formed where the marginalisation runs against the prior precision of the model being marginalised — which for an outcome axis is the precision that axis's own registration set rather than the prior tabulated above. Both parameters reach every general and rank-one marginalisation call site. A factor or ceiling outside the domains tabulated here is refused at construction rather than carried into the arithmetic.

The conditioning-growth trigger is a multiple rather than a threshold, and the change of kind matters more than the figure. It is compared against the cheap diagonal ratio the last recomputation recorded, not against the ratio itself, because the replenishment floor pins that ratio's denominator and a fixed threshold on it is therefore met permanently as soon as one dimension rests at the floor — permanently being no trigger at all (`alg:gaussian:condition-adaptive-recompute`). The default of two is chosen for the rate it implies rather than for the level it names: a recomputation records the ratio it has just measured, so the trigger fires at most once per doubling of that ratio, which is logarithmic in how far the matrix has grown rather than proportional to the labels absorbed. A smaller multiple would fire on the ordinary movement of the diagonal under decay and replenishment; a much larger one would let the ratio climb through an order of magnitude before the counter's own interval elapsed, which leaves the trigger doing nothing the counter was not going to do anyway.

The spectral floor the posterior maintains is not tabulated, because a deployment does not set it. It is derived at each recomputation from the largest eigenvalue measured there and the model's width, as the least value for which a floating-point factorisation of the matrix is assured (`dec:posterior:spectral-floor`), and it is applied there too — where the covariance is being rebuilt from the same matrix, so that the pair stays consistent. A figure a host cannot move belongs at the decision that derives it, on the same reasoning the lower condition guard is stated at its own algorithm.

The posture ceiling is the upper of the two condition guards and the only one tabulated. The lower guard is a package-side arithmetic limit rather than a parameter a deployment sets, and it is stated at the algorithm that owns it (`alg:gaussian:regularised-schur`); a table of host parameters is the wrong place for a figure a host cannot move.

**Table (The eligibility policy)** · `tab:config:eligibility`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Challenge failure counts as unconfounded | true | boolean | (`req:host:eligibility-policy`) |

One switch, set at construction, deciding whether a failed Challenge that was not independently investigated is admitted as unconfounded evidence for the sister and anchor models. The default admits it, on the reasoning that a challenge failure is informative about the entity rather than about the challenge, and a host whose challenge population is selected by the very estimate it feeds should set the switch the other way.

The predicate follows the selected row for Challenge while preserving the ground-truth override and the fixed eligibility of the other actions.

**Table (Outcome axis per-axis defaults)** · `tab:config:axis`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_a$ (axis forgetting) | $\gamma_\text{inh}$ | $(0, 1)$ | (`def:axis:training-target`) |
| $\kappa_{a,0}$ (initial compression scale) | 1.0 | $> 0$ | (`alg:registry:axis-registration`) |
| Training eligibility mode | Eligible only | eligible only, or all labels | (`def:registry:outcome-axis`) |
| Spatial features | Enabled | enabled or disabled | (`disc:registry:spatial-policy`) |

Every value is a per-axis default supplied at registration, so two axes on one deployment may differ in all four. The forgetting rate inherits the sister and anchor rate rather than the operational one because an axis model learns from the same slow-moving evidence the inherited models do.

The two enumerations are the ones that cost: the eligibility mode decides which question the axis answers, and the spatial policy decides whether the axis adds per-cell state to every identity dimension.

**Table (Temporal parameters)** · `tab:config:temporal`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_{t,\text{core}}$ (core model time decay) | 0.9999 per hour | $(0, 1)$ | (`tab:temporal:decay-inventory`) |
| $\gamma_{t,L}$ (Ledger time decay) | 0.999 per hour | $(0, 1)$ | (`def:ledger:time-decay`) |
| $\gamma_{t,\text{id}}$ (identity cell time decay) | 0.998 per hour | $(0, 1)$ | (`tab:keyspace:decay-rates`) |

Three rates, each indexed by elapsed time rather than by label count, and ordered by how fast the thing they govern should be allowed to be forgotten. The Core's own models decay slowest because their parameters are the deployment's accumulated knowledge; the identity layer decays fastest because a competitive landscape reshapes itself in days.

The identity layer's per-label rates are a separate mechanism and are not host-settable, so a host tuning temporal behaviour tunes these three and nothing else.

**Table (Identity layer parameters)** · `tab:config:identity`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $L_d$ (competitive depth cutoff) | 3 | $\geq 1$ | (`alg:keyspace:observation-protocol`) |
| $\lambda_\text{id}$ (outcome EWMA rate) | 0.95 | $(0, 1)$ | (`tab:keyspace:outcome-state`) |
| $\lambda_m$ (measurement EWMA rate) | 0.95 | $(0, 1)$ | (`tab:keyspace:measurement-state`) |
| Signal cache capacity | 100,000 | $\geq 1$ | (`tab:keyspace:signal-cache`) |
| Graph budget per dimension | Host-supplied | $> 0$ | (`def:registry:identity-dimension`) |
| Split threshold per dimension | Host-supplied | $> 0$ | (`def:registry:identity-dimension`) |

The table is mixed by construction and says so in its own rows: the last two are declared per dimension at registration and have no system-wide default, and the depth cutoff is likewise supplied rather than defaulted. The measurement rate sets how long a competitive cell takes to be trusted — about twenty assessments per cell per dimension at the value above — which is the figure the maturity diagnostics are read against.

The two host-supplied rows have no default, which is what the table already claims of them.

**Table (Ledger parameters)** · `tab:config:ledger`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\lambda_L$ (Ledger EWMA rate) | 0.999 | $(0, 1)$ | (`tab:ledger:entry-state`) |
| $\gamma_{t,L}$ (time-indexed decay) | 0.999 per hour | $(0, 1)$ | (`def:ledger:time-decay`) |
| $N_\text{absent}$ (consecutive-absence deletion threshold) | 3 | $[1, 255]$ | (`alg:ledger:entry-deletion`) |
| $N_\text{ledger}$ (recent eligible label window) | 200 | $\geq 10$ | (`def:ledger:starvation-score`) |
| Collection floor on the entry average | $10^{-6}$ | $> 0$ | (`alg:ledger:garbage-collection`) |
| Collection horizon | 60 days | $> 0$ | (`alg:ledger:garbage-collection`) |

The time-indexed rate is the same parameter as the Ledger row of the temporal table and is repeated here because a reader sizing the Ledger needs it beside the label-indexed rate it competes with. The starvation window is a count of recent eligible labels and is a different parameter from the resolution-adequacy threshold in the monitoring table, which is also a label count and is permanently separate from it. The absence threshold's upper end is where the count that carries it ends: consecutive absences are counted in eight bits, so a threshold above 255 is one the count can never reach, and a deletion rule that can never fire is not a slower rule but an absent one.


**Table (Calibration parameters)** · `tab:config:calibration`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $N_\text{cal,buf}$ (buffer capacity) | 2,000 | $\geq 100$ | (`constr:platt:buffer`) |
| $N_\text{refit}$ (periodic refit cadence) | 200 | $\geq 50$ | (`tab:platt:refit-cadence`) |
| $N_\text{cal,min}$ (minimum records per regime) | 30 | $\geq 10$ | (`req:platt:minimum-samples`) |
| $n_\text{cal,pos}$ (minimum positive outcomes) | 3 | $\geq 1$ | (`req:platt:minimum-samples`) |
| $n_\text{cal,neg}$ (minimum negative outcomes) | 3 | $\geq 1$ | (`req:platt:minimum-samples`) |
| $\kappa_0$ (initial calibration parameter) | 1.0 | $> 0$ | (`def:platt:initial-value`) |
| $\kappa_\text{min}$ (search lower bound) | 0.01 | $> 0$ | (`alg:platt:fitting`) |
| $\kappa_\text{max}$ (search upper bound) | 100 | $> \kappa_\text{min}$ | (`alg:platt:fitting`) |
| $\gamma_\text{cal}$ (recency weighting) | $\gamma_\text{inh}$ | $(0, 1)$ | (`def:platt:objective`) |
| $s_\kappa$ (soft transition steepness) | 20 | $> 0$ | (`def:platt:regimes`) |
| $\delta_\text{cal}$ (drift-reset significance) | 0.1 | $> 0$ | (`alg:platt:drift-integration`) |

Eleven parameters over three concerns: what the buffer holds, when a refit is attempted and whether it is allowed to proceed, and how the fitted parameter is searched for and blended across the two regimes. The three sample floors are the ones a host is most likely to want lower and should not: a calibration fitted below them is a curve through noise, and a host that lowers them buys calibrated output that is not calibrated.

The transition steepness is a constant of the numerics rather than a field of the calibration configuration and is therefore not settable.

**Table (Standardisation parameters)** · `tab:config:standardisation`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\gamma_\text{std}$ (standardisation EWMA rate) | 0.9998 | $(0, 1)$ | (`alg:standardisation:label-time-procedure`) |
| $v_\text{floor}$ (variance floor) | $10^{-4}$ | $> 0$ | (`alg:standardisation:label-time-procedure`) |
| $n_\text{std}$ (feature clip width) | 10 | $> 0$ | (`alg:standardisation:label-time-procedure`) |
| $N_\text{init}$ (cold prior-mass ramp horizon) | 100 | $\geq 10$ | (`alg:standardisation:batch-initialisation`) |
| $N_\text{boot}$ (per-Sentinel bootstrap sample count) | 100 | $\geq 10$ | (`alg:standardisation:sentinel-bootstrap`) |
| $\alpha_\text{boot}$ (bootstrap blend factor) | 0.8 | $(0, 1]$ | (`alg:standardisation:sentinel-bootstrap`) |

The clip width and the variance floor are the two that bound the damage a pathological feature can do: the floor stops a constant feature from dividing by nothing, and the clip stops an extreme value from dominating an update. The bootstrap pair governs how quickly a newly registered Sentinel's features become comparable with the rest.

The two per-Sentinel bootstrap fields are a sample count and a blend factor, and the mechanism whose name they carry reads them as such.

The horizon names the accepted-observation span over which the class priors' standardisation mass falls to zero, one share per accepted observation, rather than a count after which those priors are replaced in one publication.

**Table (Concurrency parameters)** · `tab:config:concurrency`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Label queue capacity | 10,000 | $\geq 100$ | (`req:publication:label-queue`) |
| Label queue overflow policy | Refuse newest and count | — | (`req:publication:label-queue`) |
| Publication interval, in labels per publish | 1 | $\geq 1$ | (`req:publication:interval`) |
| Identity dimension lock granularity | Per dimension | — | (`tab:publication:tiers`) |
| Deferred identity queue capacity | Bounded, drop oldest | $\geq 1$ | (`alg:publication:identity-draining`) |

Two of the five are structural rather than numeric and are tabulated because a host reading this table needs to know they are not choices: the lock granularity is a property of the design, and the overflow policy follows from the non-blocking discipline, since a queue that refused work would push back on a caller that must not be pushed back on.

The publication interval sets how many labels the model owner may absorb before publishing the next snapshot. The queue capacity and both bounded structures follow the policies tabulated above.

**Definition (Reference peak request rate)** · `def:config:reference-peak-request-rate-100-per-second`

The reference peak request rate is 100 requests per second — the $R$ of the pending-buffer capacity computation. The value is part of this label's name, so revising it is a re-mint that visits every citation.

**Definition (Reference label latency)** · `def:config:reference-label-latency-3600-seconds`

The reference median labelling latency is 3,600 seconds, one hour — the $L$ of the same computation. The same name-carries-value rule applies: the value is part of this label's name, so revising it is a re-mint that visits every citation.

**Table (Pending buffer parameters)** · `tab:config:pending-buffer`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $R$ (expected peak request rate) | 100 per second | $> 0$ | (`def:config:reference-peak-request-rate-100-per-second`) |
| $L$ (expected median label latency) | 3,600 s | $> 0$ | (`def:config:reference-label-latency-3600-seconds`) |
| Buffer capacity | Computed as $2 \times R \times L$ | $\geq 1$ | (`req:runtime:buffer-capacity`) |
| Expiry horizon | 24 hours | $> 0$ | (`req:runtime:buffer-capacity`) |
| Feature storage precision | Single | single or double | (`def:runtime:storage-precision`) |

The capacity is stated as a computation rather than as a number because the buffer is the system's dominant memory consumer at any serious request rate: it is twice the request rate multiplied by the expected label latency, so a fixed default is wasteful at low rates and silently lossy at high ones. The horizon is the second half of the same sizing argument — an entry older than the horizon is one whose label is not coming.

The capacity is $2 \times R \times L$ from the reference peak request rate (`def:config:reference-peak-request-rate-100-per-second`) and the reference median labelling latency (`def:config:reference-label-latency-3600-seconds`), and the horizon is twenty-four hours.

**Table (Extraction parameters)** · `tab:config:extraction`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $D_\text{chain}$ (chain length normalisation) | 16 | $\geq 1$ | (`tab:extraction:chain-structure`) |

One parameter, and it is the divisor that turns a raw ancestor-chain depth into a feature on a comparable scale. It is the maximum chain depth the encoding is expected to produce, so setting it is a statement about the host's key space rather than a tuning knob.

The parameter rescales two features every Sentinel slot carries — the normalised chain length of the chain-structure block, and the normalised peak context depth of the coordination block (`tab:extraction:coordination`). The chain-length feature is logarithmic in depth before it is divided, while the peak context depth is divided directly.

**Table (Label guidance parameters)** · `tab:config:guidance`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| Default scan limit | 10,000 | $\geq 100$ | (`sig:guidance:interface`) |
| Starvation score threshold | 0.01 | $\geq 0$ | (`def:guidance:starvation`) |

The scan limit is what bounds guidance's cost. Guidance reads stored assessments and never recomputes a model, so its work is linear in this limit and independent of the feature dimension, and a host that raises it pays in scan time and in nothing else. The starvation threshold is the score above which a cell is offered as a candidate for relief.


**Table (Health monitoring parameters)** · `tab:config:monitoring`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\kappa_\text{drift}$ (drift noise allowance) | 0.1 | $\geq 0$ | (`alg:monitoring:drift-cusums`) |
| $h$ (accumulator threshold) | 10 | $> 0$ | (`alg:monitoring:drift-cusums`) |
| $N_\text{auc,min}$ (minimum class count) | 20 | $\geq 5$ | (`alg:monitoring:auc`) |
| $N_\text{recent}$ (recent discrimination window) | 500 | $\geq 50$ | (`alg:monitoring:auc`) |
| $\epsilon_\text{sync,thresh}$ (synchronisation threshold) | $10^{-6} \cdot p$ | $> 0$ | (`def:config:synchronisation-threshold`) |
| $N_\text{conc,min}$ (minimum labels for concordance) | 5 | $\geq 3$ | (`def:config:concordance-minimum-labels`) |
| $\delta_\text{conc}$ (concordance deficit threshold) | 0.25 | $(0, 0.5)$ | (`def:config:concordance-deficit-threshold`) |
| $\theta_\text{alarm}$ (strong alarm threshold) | 3.0 | $> 0$ | (`def:config:strong-alarm-threshold`) |
| $\theta_\text{quiet}$ (no-alarm threshold) | 0.5 | $\geq 0$ | (`def:config:no-alarm-threshold`) |
| $\kappa_\text{lab}$ (alarm-outcome noise allowance) | 0.02 | $\geq 0$ | (`def:config:alarm-outcome-noise-allowance`) |
| $\theta_\text{stability}$ (feature stability threshold) | 0.001 | $> 0$ | (`def:config:feature-stability-threshold`) |
| $\eta_\text{mature}$ (maturity threshold) | 0.1 | $(0, 1)$ | (`def:config:maturity-threshold`) |
| $N_\text{adequate}$ (resolution adequacy threshold) | 20 | $\geq 1$ | (`def:config:resolution-adequacy-threshold`) |
| $\gamma_\text{latency}$ (feedback latency EWMA rate) | 0.99 | $(0, 1)$ | (`def:config:feedback-latency-ewma-rate`) |
| $N_\text{material}$ (Ledger materiality threshold) | 100 | $\geq 1$ | (`def:config:ledger-materiality-threshold`) |
| $A_\text{min}$ (attenuation materiality floor) | 0.25 | $(0, 1)$ | (`def:config:attenuation-materiality-floor`) |

Sixteen thresholds over five concerns — drift, discrimination, numerical precision, label integrity, and cross-layer observability — and they are gathered into one table because they govern diagnostics rather than model meaning. Fifteen are reading thresholds on reported quantities. The synchronisation threshold is the one operational exception: crossing it halves the recomputation interval under the adaptive-cadence rule while leaving the posterior update itself unchanged (`dec:posterior:adaptive-cadence`).

The monitoring configuration carries all sixteen rows, defaults them to the tabulated values, and rejects values outside their tabulated domains at construction.

**Definition (Synchronisation threshold)** · `def:config:synchronisation-threshold`

The synchronisation threshold is $\epsilon_\text{sync,thresh} = 10^{-6} \cdot p$. It governs when the reported synchronisation error calls for the covariance to be recomputed (`def:monitoring:synchronisation-error`). The comparison is against the residual — the reading less the part of it the replenishment clamp put there — because that is the part a recomputation can remove. The same threshold governs the other component of the reading, the part the replenishment clamp put there: a contribution over it calls for the recomputation that is the only operation which absorbs it, though it does not shorten the interval (`dec:posterior:adaptive-cadence`). A third figure is published and governs nothing — the resolution the reading carries, derived from the width and the operand magnitudes rather than set, which says how much of the residual the arithmetic could have manufactured. Both health reports carry the Frobenius-norm diagnostic, the prior-induced component, the residual, the resolution and its flag, and the comparison reads the configured per-dimension coefficient. Equality leaves the cadence unchanged and the smallest represented crossing calls for a recomputation, which shortens the interval where it is adopted.

**Definition (Minimum labels for concordance)** · `def:config:concordance-minimum-labels`

The per-entity concordance gate is $N_\text{conc,min} = 5$ labels. Below it an entity's agreement between reported outcomes and risk estimates is not read (`alg:monitoring:per-entity-concordance`). Equality reaches the gate.

**Definition (Concordance deficit threshold)** · `def:config:concordance-deficit-threshold`

The concordance deficit threshold is $\delta_\text{conc} = 0.25$. It governs when an entity's concordance falls far enough below aggregate discrimination to be flagged (`alg:monitoring:per-entity-concordance`). Equality is not flagged; a threshold immediately below the same gap flags it.

**Definition (Strong alarm threshold)** · `def:config:strong-alarm-threshold`

The strong alarm threshold is $\theta_\text{alarm} = 3.0$. It supplies the high-alarm side of the alarm-outcome disagreement accumulators (`def:monitoring:alarm-outcome-cusums`). The configured boundary is inclusive while a value immediately outside it stays quiet.

**Definition (No-alarm threshold)** · `def:config:no-alarm-threshold`

The no-alarm threshold is $\theta_\text{quiet} = 0.5$. It supplies the low-alarm side of the alarm-outcome disagreement accumulators (`def:monitoring:alarm-outcome-cusums`). The configured boundary is inclusive while a value immediately outside it stays quiet.

**Definition (Alarm-outcome noise allowance)** · `def:config:alarm-outcome-noise-allowance`

The alarm-outcome noise allowance is $\kappa_\text{lab} = 0.02$. It is exactly the disagreement either alarm-outcome accumulator forgives before growing (`def:monitoring:alarm-outcome-cusums`).

**Definition (Feature stability threshold)** · `def:config:feature-stability-threshold`

The feature stability threshold is $\theta_\text{stability} = 0.001$. It is the feature-distribution half of the feature-stable outcome-drift conjunction (`def:monitoring:feature-stable-drift`). It is independent of the drift accumulator threshold: equality keeps the flag down and a crossing raises it.

**Definition (Maturity threshold)** · `def:config:maturity-threshold`

The maturity threshold is $\eta_\text{mature} = 0.1$. A cell strictly below it contributes its traffic to measurement-maturity coverage (`def:monitoring:maturity-coverage`).

**Definition (Resolution adequacy threshold)** · `def:config:resolution-adequacy-threshold`

The resolution adequacy threshold is $N_\text{adequate} = 20$ eligible labels. It is the per-cell evidence gate in resolution utilisation (`def:monitoring:resolution-utilisation`). Equality is adequate and the entry immediately below it is not.

**Definition (Feedback latency EWMA rate)** · `def:config:feedback-latency-ewma-rate`

The feedback-latency EWMA rate is $\gamma_\text{latency} = 0.99$. It is the weight kept on the existing reading when a new one arrives, and it smooths all four stages of the end-to-end latency — report, label, queue and publication (`def:monitoring:feedback-latency`). One observation is folded per published label, on the model owner's thread, so the rate sets a horizon in labels rather than in time: at this default a stage's reading reflects roughly the last hundred labels, which is what makes it a diagnostic of the deployment rather than of its most recent label.

A stage's first observation is taken as its reading outright rather than smoothed from zero. At a rate this close to one an accumulator started at zero would spend hundreds of labels climbing out of a figure nothing measured, and the early reading is exactly when an operator is watching.

**Definition (Ledger materiality threshold)** · `def:config:ledger-materiality-threshold`

The Ledger materiality threshold is $N_\text{material} = 100$ eligible labels. It governs the fleet-wide immature-cell count and one arm of the per-assessment immaturity predicate (`def:monitoring:immature-cells`). Full health reads the strict threshold and reports the count (`entry:memory:immature-count`). The per-assessment predicate reads the same value and applies the same strict comparison to the one entry the extraction routing reached (`def:runtime:alarm-summary`), so a deployment that moves this number moves both readings together and cannot move one alone.

**Definition (Attenuation materiality floor)** · `def:config:attenuation-materiality-floor`

The attenuation materiality floor is $A_\text{min} = 0.25$. It is the other arm of the per-assessment Ledger immaturity predicate (`def:runtime:alarm-summary`). The attenuation it is compared against is computable at every entry from the stored arrival window (`tab:ledger:entry-state`). The domain is the open unit interval, which is what lets the empty-window reading be settled once and for all: an entry whose window has decayed to nothing takes the attenuation formula's limit of zero, and zero lies below every floor this definition admits.

**Beyond the Core's own tables.** Three surfaces remain, and none of them is Core state: the policy the Derivation Function is called with, the display constants of the optional rendering layer, and the Companion's own configuration.

**Table (Derivation function configuration)** · `tab:config:derivation`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $R_\text{pass}$ (opportunity cost of denial) | 1.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{friction}$ (cost of challenging a good source) | 0.3 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{missed}$ (cost of missing an adverse outcome) | 3.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{caught}$ (reward for identifying an adverse source) | 2.0 | $> 0$ | (`tab:channel:reward-parameters`) |
| $R_\text{blocked}$ (cost of blocking a good source) | 1.5 | $> 0$ | (`tab:channel:reward-parameters`) |
| $\alpha_s$ (throttle severity fraction) | 0.2 | $(0, 1)$ | (`def:landscape:extended-rewards`) |
| $\beta_c$ (block catches fraction) | 1.0 | $[0, 1]$ | (`def:landscape:extended-rewards`) |
| $\beta_b$ (adverse-class sensitivity) | 1.5 | $> 0$ | (`def:channel:sensitivity-exponents`) |
| $\beta_g$ (benign-class sensitivity) | 1.0 | $> 0$ | (`def:channel:sensitivity-exponents`) |
| Neutral zone half-width | 0.1 | $[0, 0.5)$ | (`def:channel:neutral-zone`) |

The Derivation Function takes no configuration of its own beyond this policy, and the policy is supplied per call rather than registered, so one assessment may be derived under several policies at once. Where the declared action set omits an action, the parameters that affect only that action are accepted and ignored rather than rejected, because a host narrowing its action set should not have to prune its cost declaration to match.

The reward and sensitivity fields feed the derivation; the neutral-zone field is accepted and deliberately unread, as its own environment specifies. The display constants the landscape excludes are tabulated separately, immediately below.

**Table (Rendering configuration)** · `tab:config:rendering`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $c_Q$ (bandwidth scale factor) | 1.0 | $> 0$ | (`def:rendering:bandwidths`) |
| $c_{\text{susp},Q}$ (Suspicious bandwidth) | 0.5 | $> 0$ | (`def:rendering:bandwidths`) |
| $Q_\text{floor}$ (display bandwidth floor) | 0.3 | $> 0$ | (`def:rendering:bandwidths`) |
| $c_A$ (uncertainty attenuation coefficient) | 10.0 | $\geq 0$ | (`def:rendering:magnitudes`) |
| $c_\ell$ (classification location compression) | 0.5 | $> 0$ | (`def:rendering:magnitudes`) |
| $c_\text{susp}$ (Suspicious magnitude coefficient) | 0.5 | $> 0$ | (`def:rendering:magnitudes`) |
| $\varepsilon_\text{mono}$ (dominated tag magnitude) | $10^{-6}$ | $> 0$ | (`alg:rendering:dominated-treatment`) |
| $Q_\text{min}$ (dominated tag bandwidth) | 0.01 | $> 0$ | (`alg:rendering:dominated-treatment`) |

Eight presentation constants, and the table is new: two environments of this document cite a rendering configuration table and until now the document did not contain one. The constants belong here rather than on the landscape precisely because the landscape is specified to be free of them, so gathering them into a table of their own is what keeps that separation checkable — a reader can see at a glance that no number in this table reaches any decision quantity.


**Table (Companion tracker configuration)** · `tab:config:companion`

| Parameter | Default | Constraint | Reference |
| --- | --- | --- | --- |
| $\alpha_0$ (prior failure pseudo-count) | 1.0 | $> 0$ | (`def:companion:prior`) |
| $\beta_0$ (prior pass pseudo-count) | 1.0 | $> 0$ | (`def:companion:prior`) |
| $\gamma_{q,t}$ (pseudo-count time decay) | 0.9998 per hour | $(0, 1)$ | (`alg:companion:decay`) |
| Injection ceiling, per call | 1,000 | $> 0$ | (`alg:companion:injection`) |

The prior pair is uniform by construction, which is the honest starting point for a quantity the deployment has no evidence about yet. The decay rate is the Companion's alone and is deliberately independent of every Core rate, since the Companion holds no Core state and shares no timeline with it.
