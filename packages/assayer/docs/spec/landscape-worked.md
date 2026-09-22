### Chapter (Worked Landscapes) · `chap:spec:worked-landscapes`

Four landscapes, worked end to end from a risk basis and a channel policy to crossovers, regimes and fragility, followed by a summary of how the design treats each of its own concerns.

The chapter is expository in force and exact in content. Every figure below is computed from the preceding chapters at stated inputs and stated to three decimals, which makes the four examples a complete numeric acceptance oracle for Chapters 13 through 15: an implementation reproducing these tables has the decomposition, the covariance, the dominance thresholds and the widths right, and one that misses a figure has a defect the figure localises. Each example was chosen to isolate a property that is easy to get wrong — the risk-independence of the regime table, the attribution of uncertainty to a source, the cost of declaring an extra action — rather than to illustrate a typical deployment.

**Setup (Assumptions common to all four landscapes)** · `setup:landscape:worked-assumptions`

All four examples use the default sensitivity exponents $(\beta_b, \beta_g) = (1.5, 1.0)$, the default rewards, $\beta_c = 1$, and the Companion at the uniform prior, giving $\hat{q}_c = 0.5$ and $\sigma^2_{\hat{q}_c} = 1/12 \approx 0.0833$. Only the risk basis and, in the fourth, the declared action set vary.

| Quantity | Value |
| --- | --- |
| Allow to Challenge differentials | $0.30$ / $2.50$ |
| Challenge to Slow differentials | $0.06$ / $0.30$ |
| Slow to Block differentials | $2.44$ / $1.20$ |
| Offsets: Allow to Challenge, Challenge to Slow, Slow to Block | $-0.848$, $-0.644$, $+0.284$ |
| Challenge sensitivity, every transition | $\pm 0.8$ |
| Widths: Challenge, Slow | $0.204$ / $0.928$ |
| Challenge-driven variance per crossover | $0.0533$ |

Every constant above is derived, not declared: the differentials from the cost tables (`def:landscape:reward-differentials`), the offsets and sensitivities from the decomposition (`eq:landscape:rigid-decomposition`), the widths as differences of adjacent offsets, and the per-crossover challenge variance as the squared sensitivity times the posterior variance. None of them depends on the risk estimate, which is why they are stated once here and not repeated in each example. Each example then reports its evidence sources, its crossovers, its regimes, and its fragility at representative postures; the same four landscapes rendered appear with the rendering layer (`ex:rendering:examples`).

**Example (A low-risk known entity)** · `ex:landscape:low-risk`

Risk basis: $\hat{p} = 0.05$, $\sigma_\text{eff} = 0.42$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.95/0.05)/2.5 = 2.944/2.5 = 1.178$ and $\sigma_u = 0.42/2.5 = 0.168$, so $\sigma_u^2 = 0.0282$; the challenge posterior is the uniform prior.

| Transition | Location, logit (posture) | Variance | Interval, logit | Interval, posture |
| --- | --- | --- | --- | --- |
| Allow to Challenge | $0.330$ ($0.582$) | $0.0816$ | $[-0.27,\; 1.86]$ | $[0.43,\; 0.87]$ |
| Challenge to Slow | $0.534$ ($0.630$) | $0.0816$ | $[-0.06,\; 2.06]$ | $[0.48,\; 0.89]$ |
| Slow to Block | $1.462$ ($0.812$) | $0.0816$ | $[0.87,\; 2.99]$ | $[0.70,\; 0.95]$ |

| Action | Width | Width sd | Domination probability |
| --- | --- | --- | --- |
| Allow | boundary | — | $0$ |
| Challenge | $0.204$ | $0.462$ | $0.375$ |
| Slow | $0.928$ | $0$ | $0$ |
| Block | boundary | — | $0$ |

Fragility at Normal reads modal action Allow with flip probability about zero, the nearest crossover being $4.1$ standard deviations away; at posture $0.6$, where $\ell = +0.405$, it reads modal action Challenge with flip probability about $0.72$ and a challenge share of about $0.65$.

The reading is the point of the example. The risk evidence is strong and the challenge evidence is absent, and the landscape says so structurally rather than in a caveat: the challenge term is $0.0533$ of each crossover's $0.0816$, the Challenge regime's width is less certain than its own value, and Challenge is dominated with probability $0.375$. The Slow width, by contrast, is exact. The single highest-value evidence investment on this channel is challenge-outcome observability, not labels — a conclusion no component could reach alone.

**Example (A high-risk new entity)** · `ex:landscape:high-risk`

Risk basis: $\hat{p} = 0.72$, $\sigma_\text{eff} = 0.74$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.28/0.72)/2.5 = -0.944/2.5 = -0.378$ and $\sigma_u = 0.74/2.5 = 0.296$, so $\sigma_u^2 = 0.0876$; the uniform prior again.

| Transition | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- |
| Allow to Challenge | $-1.226$ ($0.227$) | $0.141$ | $[-2.16,\; -0.02]$ |
| Challenge to Slow | $-1.022$ ($0.265$) | $0.141$ | $[-1.95,\; 0.19]$ |
| Slow to Block | $-0.094$ ($0.477$) | $0.141$ | $[-1.03,\; 1.12]$ |

The regime table is *identical* to the first example — Challenge at width $0.204$, standard deviation $0.462$ and domination probability $0.375$; Slow at width $0.928$ and standard deviation zero — and that identity is what the example exists to show. Only the locations moved, all three by the same $-1.556$, which is the change in the shared term and nothing else (`thm:landscape:rigid-translation`).

Fragility at Normal reads modal action Slow with flip probability about $0.31$, the two bounding crossovers sitting at $-1.022$ and $-0.094$; at Elevated it reads Block with flip probability about $0.09$. The higher effective uncertainty inflates every crossover variance to $0.141$, so the challenge contribution falls to about thirty-eight per cent and the two evidence sources now contribute comparably — the same channel, the same policy, and a different answer to what should be bought next.

**Example (Medium risk under high uncertainty)** · `ex:landscape:medium-risk`

Risk basis: $\hat{p} = 0.35$, $\sigma_\text{eff} = 0.88$, $\kappa_\text{eff} = 1.0$. Evidence sources: $u = \ln(0.65/0.35)/2.5 = 0.619/2.5 = 0.248$ and $\sigma_u = 0.88/2.5 = 0.352$, so $\sigma_u^2 = 0.124$; the uniform prior.

| Transition | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- |
| Allow to Challenge | $-0.600$ ($0.354$) | $0.177$ | $[-1.62,\; 0.52]$ |
| Challenge to Slow | $-0.396$ ($0.402$) | $0.177$ | $[-1.42,\; 0.72]$ |
| Slow to Block | $+0.531$ ($0.630$) | $0.177$ | $[-0.49,\; 1.65]$ |

The regimes are again unchanged: Challenge at $0.204$ with standard deviation $0.462$ and domination probability $0.375$, Slow at $0.928$ with standard deviation zero.

This is the genuinely uncertain assessment, and it inverts the first example's conclusion. The risk term dominates every crossover variance — $0.124$ of $0.177$, about seventy per cent — so here the binding deficit is labels and not challenge outcomes, and the fragility decomposition attributes it that way. Fragility at Normal reads Allow with flip probability about $0.20$; at posture $0.5$, where the logit is zero, it reads Slow with flip probability about $0.35$. Almost every operating point sits near a boundary. This is exactly the request the Core's own guidance would rank highest for labelling (`def:guidance:risk-informative`), and the landscape confirms it from the decision side rather than by coincidence.

**Example (A three-action channel)** · `ex:landscape:three-action`

The same risk basis as the first example — $\hat{p} = 0.05$, $\sigma_\text{eff} = 0.42$, $\kappa_\text{eff} = 1.0$, the uniform prior — with Slow removed from the declared set. The evidence sources are unchanged at $u = 1.178$ and $\sigma_u = 0.168$, both being risk-only quantities. Allow to Challenge is unchanged because its differential pair is unchanged; Challenge to Block uses the three-action differentials $2.5$ and $1.5$, giving an offset of $\ln(2.5/1.5)/2.5 = +0.204$.

| Transition | Offset | Sensitivity | Location, logit (posture) | Variance | Interval, logit |
| --- | --- | --- | --- | --- | --- |
| Allow to Challenge | $-0.848$ | $-0.8$ | $0.330$ ($0.582$) | $0.0816$ | $[-0.27,\; 1.86]$ |
| Challenge to Block | $+0.204$ | $+0.8$ | $1.382$ ($0.799$) | $0.0816$ | $[0.44,\; 2.51]$ |

| Action | Width | Width sd | Domination probability |
| --- | --- | --- | --- |
| Allow | boundary | — | $0$ |
| Challenge | $1.052$ | $0.462$ | $0.067$ |
| Block | boundary | — | $0$ |

Two further sensitivities hold at these defaults and are stated as oracle figures rather than derived again. A crossover's location moves by $1/(\beta_b + \beta_g) = 0.400$ logit units per unit change in the logarithm of its governing differential ratio, identically for every transition; and a crossover's location moves by its own challenge sensitivity, $\pm 0.800$, per unit change in the challenge estimate. Both follow from the decomposition and neither depends on the action set.

Removing Slow turns Challenge from a $0.204$-wide sliver into a $1.052$-wide regime, a factor of $5.2$, and drops its domination probability from $0.375$ to $0.067$: at the uniform prior Challenge is very likely viable in the three-action channel and close to a coin flip in the four-action one. The width uncertainty is unchanged at $0.462$, since it depends only on the difference of adjacent sensitivities and the posterior variance (`prop:landscape:width-variance`). This is the decision-theoretic content behind the sizing guidance (`cav:channel:action-space-sizing`): declare Slow only if it is a genuinely distinct operational response, because its presence costs Challenge most of its regime.

**Summary (How the landscape treats each design concern)** · `summ:landscape:design-summary`

| Aspect | Treatment |
| --- | --- |
| Inputs | Risk assessment, channel policy, challenge posterior (`sig:landscape:derivation-function`) |
| Purity | Deterministic, stateless, closed form, presentation-free (`pf:landscape:purity`) |
| Reward differentials | Per-action cost tables with two extended parameters, fully reproducible (`def:landscape:reward-differentials`) |
| Rigid translation | Exact separation into a shared term and per-transition offsets; widths risk-independent (`thm:landscape:rigid-translation`) |
| Crossover covariance | Rank two, closed form; widths carry zero risk variance (`prop:landscape:width-variance`) |
| Dominance | Exact incomplete-beta probability, reported and never enforced (`thm:landscape:dominance`) |
| Credible intervals | Quantile push-through, exact in the challenge effectiveness; honest wide-but-bounded cold start (`def:landscape:credible-intervals`) |
| Optimal action | Upper-envelope recovery, correct under inverted crossovers (`alg:landscape:optimal-action`) |
| Outcome neutrality | Axis prediction outputs never reach the derivation (`def:landscape:outcome-neutrality`) |
| Multiple channels | One assessment, one derivation call per channel; the Core's cost paid once (`ex:architecture:compositional-dividends`) |
| Replay | The three stored inputs reproduce the landscape exactly (`pf:landscape:purity`) |

Eleven aspects, and the table restates rather than decides: every row's content is fixed at the environment it cites, and a reader who finds the two disagreeing should believe the citation. It is collected here because the four examples above are the only place the whole design is exercised at once, and a summary adjacent to worked figures is checkable in a way that a summary standing alone is not.
