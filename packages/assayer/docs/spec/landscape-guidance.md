### Chapter (Exploration and Label Guidance) · `chap:spec:exploration-and-guidance`

Two exploration signals answer two different questions, and the chapter keeps them apart because a host that conflates them will buy the wrong evidence.

Decision fragility asks whether resolving *this* decision at *this* posture would change what the host does. It is a function of the landscape and needs no model state. Core label guidance asks which pending requests would most improve the Core's own estimates, and it is a function of model state and needs no landscape. The first is a decision-quality signal, the second a learning signal; they rank differently, they are computed in different components, and the last environment of the chapter is the only place they meet.

#### Decision fragility · `sec:fragility:decision`

The division carries no material of its own. It collects the fragility record, the shares that attribute its uncertainty to a source, and the bands that say what a value of it is worth acting on.

**Definition (Decision fragility)** · `def:fragility:definition`

Fragility is read from a landscape at a posture and reports four things.

| Field | Carrying |
| --- | --- |
| Posture | The posture the reading was taken at |
| Modal action | The action optimal at the point-estimate crossovers |
| Flip probability | The probability that the true optimal action differs from the modal one |
| Risk share | The fraction of the bounding-crossover variance from the shared risk term |
| Challenge share | The fraction from the challenge posterior |

The modal action is recovered from the cost-curve envelope (`alg:landscape:optimal-action`), so a posture lying inside a dominated action's inverted region is handled correctly rather than reported as that action. The flip probability is the mass of the bounding crossover distributions lying on the far side of the posture's logit; for an interior modal action both boundaries contribute and the rank-two covariance governs the joint computation (`thm:landscape:crossover-covariance`), and inside an inverted region the crossovers bounding the *optimal* regime from the envelope are used rather than the raw adjacent records. The default computation uses the Gaussian approximation on the crossover marginals, and an implementation may substitute exact evaluation in the challenge effectiveness by quantile integration where the posterior is conjugate (`cav:limitation:fragility-gaussian`).

The two shares are the reason fragility is worth computing at all. They split the bounding variance into its risk-driven and challenge-driven parts and so answer the operational question directly: to firm up this decision at this posture, buy labels or buy challenge-outcome observability. No single component can make that call, because neither the Core nor the Companion can see the other source. The five-field fragility record is read from a landscape at the caller's posture; the separate rendered-profile utility reports the three Shannon entropies that measure display ambiguity instead (`sig:rendering:ambiguity-gauge`).

**Table (Reading a flip probability)** · `tab:fragility:interpretation`

| Flip probability | Reading | Label priority |
| --- | --- | --- |
| Above $0.35$ | The decision is effectively unresolved at this posture | Highest |
| $0.15$ to $0.35$ | The boundary is within reach of realistic evidence | High |
| $0.05$ to $0.15$ | The modal action is stable against moderate surprise | Moderate |
| Below $0.05$ | Labelling will not change the action | Low |

Fragility is a first-order proxy for value of information, and the bands are calibrated to that reading rather than to any statistical convention. An observation has decision value only when it might change the action taken, so a request deep inside a regime has near-zero fragility however uncertain its risk estimate is in absolute terms: a high probability-scale uncertainty far from any boundary changes nothing about what to do, and buying a label for it buys knowledge and no decision.

This is the essential difference from an entropy-style ambiguity gauge, which weights uncertainty by nothing at all. Fragility weights it by decision sensitivity, which is why the two rank the same population differently and why the substitution is not a rename (`inv:guarantee:exploration`). Pure model-uncertainty reduction is covered independently, and better, by the Core's own guidance (`def:guidance:risk-informative`); the two signals are complements and a host with budget for both should spend on both.

#### Core label guidance · `sec:guidance:core`

The division carries no material of its own. It collects the guidance interface, its three scoring criteria, the policy that keeps the three lists independent, and the pattern that composes them with fragility.

**Signature (The label guidance interface)** · `sig:guidance:interface`

The Core answers a budget and a small parameter set with three ranked lists of candidate requests. It reads its own state only; no landscape, no policy, and no posture reach it.

| Group | Field | Carrying |
| --- | --- | --- |
| Budget | Risk-informative, investigation, starvation-relief | How many candidates each list may return |
| Parameters | Scan limit | How many stored requests are examined |
| Parameters | Starvation threshold | The score below which a starvation candidate is not offered |
| Requests | Three lists | The candidates, one list per category |
| Candidate | Assessment identifier, entity key, timestamp, score, cross-membership | What a caller needs to act on one candidate and to see where else it appears |

Three properties are load-bearing. The budget is per category rather than global, so a host can buy learning in one direction without starving another. The scan limit bounds the work: guidance reads stored assessments and never recomputes a model, so its cost is linear in the limit and independent of the model dimension. And the candidate carries the assessment identifier rather than a copy of the assessment, so a label reported against it rejoins the stored entry by identity (`tab:config:guidance`).

**Definition (The risk-informative criterion)** · `def:guidance:risk-informative`

Risk-informative candidates are the requests the Core is least certain about. The score is the probability-scale uncertainty carried on the stored risk basis (`schema:risk:basis`):

$$\text{score}_\text{risk}(r) = \sigma_{\hat{p}, r}$$

The criterion measures exactly what a label would reduce. It needs no recomputation, since the uncertainty was computed at assessment time and stored, and it needs no reference to any decision: a high-uncertainty request is informative about the model whether or not it sits near any action boundary, which is precisely the difference from fragility (`def:fragility:definition`). A host buying only on this criterion learns efficiently and may never resolve the decisions it actually faces; a host buying only on fragility resolves its boundaries and may leave the model uncertain everywhere else.

**Definition (The investigation criterion)** · `def:guidance:investigation`

Investigation candidates are the requests where the Core suspects danger and lacks unconfounded evidence. The score is the anchor's blend weight from the stored risk basis.

The reasoning is indirect and worth stating. A high blend weight means the anchor is carrying much of the estimate, which happens when the sister model is uncertain, which happens where unconfounded outcomes have not reached — the regions the host has been restricting. Those are exactly the requests where an ordinary label says little, because the outcome was determined by the host's own action, and where an investigated outcome carrying the ground-truth flag says a great deal (`conv:eligibility:ground-truth`).

The criterion therefore ranks by the *shape* of the evidence rather than by its quantity, and it is the only guidance category whose value depends on the host having an investigation capability at all. A host without one should set its budget to zero rather than receive candidates it cannot act on (`cav:limitation:investigation-pipeline`).

**Definition (The starvation-relief criterion)** · `def:guidance:starvation`

Starvation-relief candidates are requests routed through Sentinel cells whose outcome memory is acting on a reputation that eligible feedback has not tested. The score takes the worst such cell across the reporting Sentinels:

$$\text{score}_\text{starv}(r) = \max_s \bigl(\bar{b}_{s,\text{cell}} - P_+^\text{eligible}\bigr)^+ \cdot \left(1 - \frac{n_{\text{elig},s,\text{cell}}}{N_\text{ledger}}\right)^+$$

The two factors are the divergence of the cell's remembered adverse rate above the system-wide eligible base rate, and the shortfall of its eligible label count against the count at which that rate would be trusted. Both are clipped below at zero, so a cell that is unremarkable or well fed scores nothing, and the product is what identifies the closed loop: a reputation the system is acting on, held up by evidence too thin to have tested it, and untested because the system is acting on it (`def:ledger:starvation-score`).

The criterion is the fourth mitigation layer of the contamination loop, and the only one that acts rather than waits — the others bound how long a stale reputation persists, this one buys the evidence that would settle it (`alg:valence:contamination-loop`).

**Decision (Independent lists rather than deduplication)** · `dec:guidance:no-deduplication`

Each list is sorted by its own score and truncated to its own budget, and a request appearing in two categories appears in both. It is not deduplicated; instead the candidate records which other categories it belongs to.

The decision is recorded because deduplication is the obvious alternative and is wrong in a way that is easy to miss. The three scores are not comparable — one is a probability-scale standard deviation, one a blend weight, one a product of a rate divergence and a count shortfall — so a deduplicating pass would have to rank across incomparable scales, and whichever rule it chose would silently reweight the host's declared per-category budget. Independent lists keep each budget meaning what the host set it to mean.

The cross-membership field then recovers everything deduplication would have offered, and more: a request that is uncertain *and* starved *and* worth investigating is visible as such, and a host that wants to prefer such requests can, while a host that wants exactly the budget it asked for in each category still gets it.

**Example (Composing fragility with Core guidance)** · `ex:fragility:composition`

A host wanting action-informative guidance composes the two halves itself. It assesses its pending requests, derives a landscape per assessment against its channel policy and the Companion's current posterior, reads fragility at its operating posture, and ranks by flip probability — routing each request's budget by whichever share dominates its bounding variance.

This is a composition pattern and not a Core interface, and the boundary is deliberate. The Core cannot compute fragility, because fragility depends on the host's declared costs and posture, which the Core is specified not to know (`prin:principle:measure-not-decide`). The derivation cannot compute the Core's guidance, because that depends on stored model state a pure function does not hold (`pf:landscape:purity`). Composition is where the two meet, and the host is the only place it can happen.

The routing is the part no single-component signal could produce. A high risk share sends the budget to labelling; a high challenge share sends it to challenge-outcome observability instead — more challenges executed and reported, or an override or injection where the host has evidence from elsewhere (`alg:companion:override`) and (`alg:companion:injection`). The host composes the two landscape utilities with the Core guidance lists (`sig:landscape:utilities`).
