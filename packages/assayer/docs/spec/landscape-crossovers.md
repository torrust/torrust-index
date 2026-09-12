### Chapter (The Crossover Landscape) · `chap:spec:crossover-landscape`

The chapter derives the landscape. It fixes how costs scale with posture, the per-action cost model and the differentials between adjacent actions, the decomposition that separates the two evidence sources exactly, and then the crossovers, the dominance probabilities, the covariance and the intervals that follow from it in closed form.

Nothing here iterates and nothing here searches. Every quantity is an elementary function of the risk basis, the declared rewards and the challenge posterior, and the chapter's central result is why: the risk estimate enters every crossover through one shared term, so the entire structure translates rigidly along the logit axis as risk moves, and everything about the structure's *shape* is independent of risk altogether. That is what makes a regime table reusable across assessments, a width exactly known, and an uncertainty attributable to one evidence source rather than to a mixture.

**Definition (The posture-sensitivity functions)** · `def:landscape:posture-sensitivity`

Two multipliers scale the two classes of cost with posture. The cost of missing an adverse observation grows with caution; the cost of restricting a benign one falls with it.

$$s_\text{bad}(\pi) = e^{\beta_b \cdot \ell(\pi)}, \qquad s_\text{good}(\pi) = e^{-\beta_g \cdot \ell(\pi)}$$

At Permissive the pair is about $0.04$ and $9.0$; at Emergency, about $27.0$ and $0.11$. The exponential-on-logit form is chosen for three properties at once: the multipliers are smooth and strictly monotone across the whole posture axis, they are unbounded in both directions so no declared cost ratio is unreachable, and their logarithms are linear in $\ell(\pi)$ — which is what makes every crossover condition solvable in closed form rather than numerically (`thm:landscape:rigid-translation`).

Because these multipliers are the entire operational content of a posture, they are the mechanism behind posture's channel-relativity (`prin:channel:relativity`): two channels declaring different exponents are using the same cursor to mean different things.

#### The per-action cost model · `sec:landscape:cost-model`

The division carries no material of its own. It collects the two extended reward parameters, the per-action cost tables they enter, the differentials between adjacent actions, and the one threshold at which an action's differential can vanish.

**Definition (The extended reward parameters)** · `def:landscape:extended-rewards`

Two of the seven declared rewards (`tab:channel:reward-parameters`) appear only in the cost model, and both are dimensionless fractions rather than costs.

**Slow severity** $\alpha_s \in (0, 1)$, default $0.2$, places Slow on the restriction spectrum between Challenge and Block. At $\alpha_s = 0$ Slow is Challenge; at $\alpha_s = 1$ it is Block. It scales both the friction Slow adds for benign traffic and the share of adverse outcomes Slow prevents, which is why it governs the Slow regime's width and cancels out of the Challenge regime's (`prop:channel:challenge-width`).

**Block catches** $\beta_c \in [0, 1]$, default $1.0$, is the fraction of the challenge mechanism's catching value retained under Block. At $\beta_c = 1$ the channel challenges and then blocks, so the full catching opportunity survives denial; at $\beta_c = 0$ Block preempts the challenge entirely and no intelligence is gathered from denied requests. It is the one parameter that can make an action's adverse-class differential vanish (`thm:landscape:catching-forfeiture`).

**Table (The per-action costs)** · `tab:landscape:action-costs`

Each action carries a benign-class cost and an adverse-class cost. Positive is a cost to the host; negative is a benefit.

| Action | Benign-class cost | At defaults |
| --- | --- | --- |
| Allow | $0$ | $0$ |
| Challenge | $R_f$ | $0.30$ |
| Slow | $R_f(1 + \alpha_s)$ | $0.36$ |
| Block | $\beta_c R_f + R_b + R_p$ | $2.80$ |

| Action | Adverse-class cost | At defaults |
| --- | --- | --- |
| Allow | $R_m$ | $3.00$ |
| Challenge | $(1 - \hat{q}_c)R_m - \hat{q}_c R_c$ | $0.50$ |
| Slow | $(1 - \hat{q}_c)(1 - \alpha_s)R_m - \hat{q}_c R_c$ | $0.20$ |
| Block | $-\beta_c \hat{q}_c R_c$ | $-1.00$ |

Two readings are worth making explicit. The adverse-class cost of Block is negative at the defaults because blocking an adverse source is a benefit and the challenge-then-block sequence still collects the catching value; it is exactly zero at $\beta_c = 0$, where denial buys prevention and no intelligence. And Challenge's adverse-class cost is a mixture rather than a reduction: the share $\hat{q}_c$ is caught and the remaining share is missed, which is why the challenge estimate scales the benefit of every transition into or through Challenge.

**Definition (The reward differentials)** · `def:landscape:reward-differentials`

For an adjacent pair in the declared order, the benign-class differential is the additional cost the more restrictive action imposes on benign traffic, and the adverse-class differential is the benefit it buys on adverse traffic:

$$\Delta_\text{good}^{(j \to j+1)} = C_\text{good}(a_{j+1}) - C_\text{good}(a_j), \qquad \Delta_\text{bad}^{(j \to j+1)} = C_\text{bad}(a_j) - C_\text{bad}(a_{j+1})$$

For the four-action set:

| Transition | $\Delta_\text{good}$ | $\Delta_\text{bad}$ | At defaults |
| --- | --- | --- | --- |
| Allow to Challenge | $R_f$ | $\hat{q}_c(R_m + R_c)$ | $0.30$ / $2.50$ |
| Challenge to Slow | $\alpha_s R_f$ | $(1 - \hat{q}_c)\alpha_s R_m$ | $0.06$ / $0.30$ |
| Slow to Block | $R_b + R_p + (\beta_c - 1 - \alpha_s)R_f$ | $(1 - \hat{q}_c)(1 - \alpha_s)R_m - \hat{q}_c R_c(1 - \beta_c)$ | $2.44$ / $1.20$ |

For the three-action set the Challenge-to-Block differentials are $(\beta_c - 1)R_f + R_b + R_p$ and $(1 - \hat{q}_c)R_m + (\beta_c - 1)\hat{q}_c R_c$, which at the defaults are $2.5$ and $1.5$. They are computed from the per-action costs directly and are *not* the sum of the four-action set's Challenge-to-Slow and Slow-to-Block differentials: removing an action changes which pair is adjacent, not merely which crossovers are reported.

**Theorem (The catching-forfeiture threshold)** · `thm:landscape:catching-forfeiture`

When $\beta_c < 1$ the Slow-to-Block adverse-class differential decreases and can reach zero. Block is dominated exactly when

$$\hat{q}_c > \frac{(1 - \alpha_s)R_m}{(1 - \alpha_s)R_m + R_c(1 - \beta_c)}$$

| $\beta_c$ | Threshold | Reading |
| --- | --- | --- |
| $1.0$ (default) | $1.0$, never reached | Challenge-then-block; Block always viable |
| $0.5$ | $0.706$ | Partial forensic value from denied requests |
| $0.0$ | $0.545$ | Pure block; Block dominated at moderate effectiveness |

The result is a theorem rather than an observation because it follows from the cost tables by algebra: the numerator is the prevention Block adds over Slow and the denominator adds the catching value Block forfeits, so the threshold is the challenge effectiveness at which forfeited intelligence exactly cancels added prevention. Above it, blocking is worse than throttling for adverse traffic and strictly worse for benign traffic, so no posture makes Block optimal.

The crossing is continuous, not a switch: the differential approaches zero, the crossover location grows without bound in the logit, and the regime inverts by a depth that measures how far past the threshold the estimate sits (`thm:landscape:dominance`). A non-positive differential produces an infinite crossover, from which the inversion depth — the quantity that says *how far* — is not recoverable.

**Theorem (The rigid-translation decomposition)** · `thm:landscape:rigid-translation`

Every crossover separates exactly into one term driven by the risk estimate and shared by all of them, and one offset per transition driven by the reward structure and the challenge posterior.

At the crossover posture between adjacent actions the two expected costs are equal,

$$\hat{p} \cdot \Delta_\text{bad}^{(j)} \cdot s_\text{bad}(\pi^*) = (1 - \hat{p}) \cdot \Delta_\text{good}^{(j)} \cdot s_\text{good}(\pi^*)$$

and solving for the crossover's logit, using the identity relating the risk probability to the effective raw estimate and the calibration parameter (`def:risk:probability`), gives the decomposition (`eq:landscape:rigid-decomposition`).

Three consequences follow and the rest of the chapter rests on them. The shared term is common to every crossover, so as the risk estimate varies the entire crossover set translates rigidly along the logit axis without changing spacing or order (`inv:guarantee:rigid-translation`). All risk-driven uncertainty in the landscape is therefore the single scalar $\sigma_u = \sigma_\text{eff} / ((\beta_b + \beta_g)\kappa_\text{eff})$, applied once rather than propagated per transition. And the offsets depend on the rewards and the challenge estimate alone, so every regime width is independent of the risk estimate — which is why a regime table computed for one assessment is the regime table for every assessment on that channel at that posterior. The shared term is carried once on the landscape, and each crossover record carries its offset and challenge sensitivity.

**Equation (The rigid decomposition)** · `eq:landscape:rigid-decomposition`

$$\ell^*_{j \to j+1} = \underbrace{\frac{-\hat{\rho}_\text{eff}}{(\beta_b + \beta_g)\,\kappa_\text{eff}}}_{u} \;+\; \underbrace{\frac{1}{\beta_b + \beta_g}\ln\frac{\Delta_\text{good}^{(j)}}{\Delta_\text{bad}^{(j)}(\hat{q}_c)}}_{b_j(\hat{q}_c)}$$

The offsets and their sensitivities to the challenge estimate, at the default rewards and $\hat{q}_c = 0.5$:

| Transition | $b_j$ | $s_j = \partial \ell^*_j / \partial \hat{q}_c$ | At defaults |
| --- | --- | --- | --- |
| Allow to Challenge | $-0.848$ | $-1/[(\beta_b + \beta_g)\,\hat{q}_c]$ | $-0.8$ |
| Challenge to Slow | $-0.644$ | $+1/[(\beta_b + \beta_g)(1 - \hat{q}_c)]$ | $+0.8$ |
| Slow to Block | $+0.284$ | $[(1 - \alpha_s)R_m + R_c(1 - \beta_c)] \,/\, [(\beta_b + \beta_g)\,\Delta_\text{bad}^{(\text{S}\to\text{B})}]$ | $+0.8$ |
| Challenge to Block, three-action | $+0.204$ | $+1/[(\beta_b + \beta_g)(1 - \hat{q}_c)]$ | $+0.8$ |

Interior regime widths are differences of adjacent offsets and are therefore constants of the reward structure and the challenge posterior: $w_C = 0.204$ and $w_S = 0.928$ in the four-action set, and $w_C = 1.052$ in the three-action set. The Slow width carries no challenge term at all at $\beta_c = 1$, so it is exactly known however uncertain the Companion is.

**Definition (The action crossover)** · `def:landscape:action-crossover`

A crossover is the boundary between two adjacent declared actions, reported as one record per adjacent pair.

| Field | Carrying |
| --- | --- |
| Transition | The adjacent-in-declared-order pair the boundary lies between |
| Location | The crossover's logit, the shared term plus this transition's offset |
| Offset | This transition's offset at the point estimate |
| Challenge sensitivity | The derivative of the location in the challenge estimate |
| Variance | The two-source variance at this crossover (`thm:landscape:crossover-covariance`) |
| Interval | The ninety-five per cent credible interval on the logit scale (`def:landscape:credible-intervals`) |

Under dominance a crossover may be inverted relative to its neighbours — the lower boundary of an interior action lying above its upper boundary — and the record is reported unchanged. That is the landscape's honest statement that the intervening action has no viable regime at the point estimate, and the magnitude of the inversion is how far from viability it sits. Consumers recover the optimal action from the cost-curve envelope rather than from the record's order (`alg:landscape:optimal-action`). The crossovers form a public vector on the landscape with one record per adjacent pair. Each record carries the fields above and also retains the adjacent cost differentials used by the optimal-action utility.

**Equation (The crossover)** · `eq:landscape:crossover`

$$\pi^*_{j \to j+1} = \sigma\bigl(\ell^*_{j \to j+1}\bigr), \qquad \ell^*_{j \to j+1} = u + b_j(\hat{q}_c)$$

The posture-scale crossover is the logistic image of the logit-scale one, and the landscape reports both because they answer different questions: the logit scale is where the arithmetic is linear and the uncertainty is Gaussian, and the posture scale is where the host's cursor lives. Every derived quantity — the variance, the interval, the width, the covariance — is computed on the logit scale and mapped afterward, never the reverse, since the logistic map is nonlinear and an interval transformed after computation is exact where a variance transformed after computation is not.

**Theorem (Dominance as a probability)** · `thm:landscape:dominance`

Because every regime width is independent of the risk estimate (`thm:landscape:rigid-translation`), every domination condition reduces to a threshold on the challenge effectiveness alone. Dominance is therefore reported as a probability under the challenge posterior, exact and closed form for the conjugate variant:

$$P(\text{action } a \text{ dominated}) = I_{\theta_a}(\alpha, \beta) \quad \text{or} \quad 1 - I_{\theta_a}(\alpha, \beta)$$

with $I$ the regularised incomplete beta function and $\theta_a$ the action's threshold, the direction taken per row:

| Action | Dominated when | $\theta_a$ at defaults | $P$ at the uniform prior |
| --- | --- | --- | --- |
| Challenge, four-action | $q_c \leq R_m/(2R_m + R_c)$ | $0.375$ | $0.375$ |
| Challenge, three-action | $q_c \leq R_m R_f/[(R_b + R_p)(R_m + R_c) + R_m R_f]$ | $0.067$ | $0.067$ |
| Block, at $\beta_c < 1$ | $q_c$ above the forfeiture threshold (`thm:landscape:catching-forfeiture`) | $1.0$ at $\beta_c = 1$ | $0$ |
| Slow, at defaults | never; its width is positive unconditionally | — | $0$ |

Dominance is reported and never structurally enforced (`inv:guarantee:dominance`): the crossover vector keeps one entry per adjacent declared pair whatever the probability says, so the landscape's shape stays a continuous function of the evidence and a Companion update moves inversion depths and probabilities smoothly rather than adding or removing structure. An action at intermediate probability — Challenge at Companion cold start, at $0.375$, is the canonical case — is reported with both its point-estimate regime and its probability, so the host sees an open evidential question rather than a settled fact. Display suppression of dominated actions belongs to the rendering layer and has no counterpart here (`alg:rendering:dominated-treatment`). The moment-matched posterior evaluates the same expression on its moment-matched Beta. The regularised incomplete-beta mass for every regime is reported as the domination probability.

**Theorem (The crossover covariance)** · `thm:landscape:crossover-covariance`

The landscape's uncertainty has exactly two independent sources, which gives the crossover vector a covariance of rank two:

$$\operatorname{Cov}(\ell^*_j, \ell^*_k) = \sigma_u^2 + s_j s_k \sigma^2_{\hat{q}_c}$$

The diagonal entries are the per-crossover variances the records report. The off-diagonal structure is what any functional of several crossovers needs: the variance of a linear combination is $(\sum_j c_j)^2 \sigma_u^2 + (\sum_j c_j s_j)^2 \sigma^2_{\hat{q}_c}$, and treating the crossovers as independent gets that wrong in both directions depending on the signs.

Adjacent covariance is frequently negative, and the sign is informative rather than pathological. Allow-to-Challenge has sensitivity $-0.8$ and Challenge-to-Slow has $+0.8$, so their covariance is $\sigma_u^2 - 0.64 \sigma^2_{\hat{q}_c}$: challenge evidence that raises the estimate moves the two boundaries apart, widening Challenge from both sides at once. A consumer judging the joint stability of an interior action's regime must use this structure (`inv:guarantee:two-source-uncertainty`). Each crossover's variance takes the specified two-source form, and the shared risk uncertainty and each crossover's challenge sensitivity let a consumer recover every off-diagonal entry without a materialised matrix.

**Proposition (Regime widths carry no risk uncertainty)** · `prop:landscape:width-variance`

The variance of an interior regime's width is

$$\operatorname{Var}(w_j) = (s_{j+1} - s_j)^2 \,\sigma^2_{\hat{q}_c}$$

The shared term cancels identically between the two bounding crossovers, which is the covariance restatement of the rigid decomposition (`thm:landscape:rigid-translation`): widths are functions of the reward structure and the challenge posterior only, so no amount of risk uncertainty makes a width less certain.

At the defaults with the uniform prior, the Challenge width's variance is $(1.6)^2 \times 0.0833 = 0.213$ — a standard deviation of about $0.46$ against a point width of $0.204$, so at Companion cold start the width is less certain than its own value — while the Slow width's variance is exactly zero.

The operational content is a routing rule the landscape can state and no single component can. A host reading a wide width variance knows the remedy is challenge-outcome evidence and not labels, because labels cannot move a quantity that risk uncertainty does not enter (`data:companion:contributing-rate`). The landscape surfaces both the width and this variance on every interior regime; boundary regimes carry neither because their widths are unbounded on one side.

**Definition (Credible intervals by quantile push-through)** · `def:landscape:credible-intervals`

Each crossover is monotone in the challenge effectiveness on its viable domain — Allow-to-Challenge strictly decreasing, Challenge-to-Slow and Challenge-to-Block strictly increasing, Slow-to-Block monotone where Block is undominated — so an interval on the effectiveness maps to an interval on the crossover by evaluating the offset at its endpoints:

$$\text{interval}_{95}(\ell^*_j) = \Bigl[\,u + \min b_j(q_{lo}, q_{hi}) - z_{0.975}\,\sigma_u, \;\; u + \max b_j(q_{lo}, q_{hi}) + z_{0.975}\,\sigma_u \,\Bigr]$$

with the endpoints the posterior's two-and-a-half and ninety-seven-and-a-half per cent quantiles. The composition unions the two sources rather than convolving them, so it is conservative by construction; an implementation wanting tighter coverage may substitute numerical convolution.

The push-through is exact in the challenge effectiveness however nonlinear the offset is — including the steep Slow-to-Block dependence near the forfeiture threshold (`thm:landscape:catching-forfeiture`), where a Gaussian linearisation of the offset is a poor summary and a delta-method interval would understate the uncertainty badly. At the uniform prior the intervals are wide and bounded, which is how the landscape expresses Companion cold start without needing a floor parameter, and they narrow as challenge outcomes accumulate and as the Core's effective uncertainty contracts. Exact quantiles require the conjugate posterior; the moment-only variant supplies them from a moment-matched Beta, and the boundary admits nothing else (`sig:companion:posterior`).

**Algorithm (The computation order)** · `alg:landscape:computation-order`

The landscape and any rendering over it are computed in six steps, each depending only on steps before it.

1. **Crossovers.** The shared term from the risk basis, the offsets from the rewards and the challenge estimate, and their sum (`eq:landscape:rigid-decomposition`). Nothing earlier is needed and nothing later feeds back.
2. **Bandwidths.** The display bandwidths, from the crossover spacing and the two uncertainty sources (`def:rendering:bandwidths`).
3. **Magnitudes.** The tag magnitudes, from the risk basis and the bandwidths (`def:rendering:magnitudes`).
4. **Interior action locations.** The interior action tags placed by crossover matching, which reads the crossovers of step one and the bandwidths of step two (`alg:rendering:placement`).
5. **Boundary action locations.** The outermost action tags placed relative to the interior ones, which therefore cannot be computed before them.
6. **Classification tags.** Placed and scaled from the magnitudes of step three and the bandwidths of step two.

The chain is acyclic, and the specification fixes it because the alternative is a silent disagreement. Steps four and five in particular have a real dependency between them — a boundary tag is placed by reference to its interior neighbour — and computing them in the other order produces different locations without producing an error. The order is stated here, in the chapter that owns step one, so that the whole chain has one authority rather than one per layer.

**Definition (Outcome prediction neutrality)** · `def:landscape:outcome-neutrality`

The derivation reads the risk basis and nothing else of the Core. Stated concretely, what does *not* enter it:

- outcome axis predicted values and their uncertainties (`tab:axis:inference-outputs`)
- outcome axis prediction intervals
- any function of an axis model's parameters
- any Core state beyond the risk basis (`schema:risk:basis`)

What *does* influence the risk estimate, inside the Core and upstream of the derivation, are the historical outcome-axis averages carried as features: the identity layer's per-cell averages (`tab:keyspace:outcome-state`) and the Ledger's per-cell averages for spatially enabled axes (`tab:extraction:ledger-features`).

The distinction is between predictions and history. Historical averages enter the feature vector alongside everything else and influence the risk estimate through learned weights, which is intended — outcome history is informative about risk — and carries the contamination exposure that pathway always carries (`disc:valence:axis-pathway`). Predictions are Core outputs reported to the host and have no pathway into the derivation at all (`inv:guarantee:outcome-neutrality`). The derivation's input carries the assessment, channel policy and challenge posterior, but reads only the assessment's risk basis; no outcome prediction enters the landscape.
