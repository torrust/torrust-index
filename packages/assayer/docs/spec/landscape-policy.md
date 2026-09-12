### Chapter (Channel Policy) · `chap:spec:channel-policy`

The channel policy is everything the host declares about one channel: which actions it can actually take, what each mistake costs it, how fast those costs grow with caution, and where on the resulting axis it currently wants to stand. The Core has no concept of any of it. A policy is handed to the derivation on each call and never registered, which is what lets one assessment serve several channels with different action sets and different costs (`ex:architecture:compositional-dividends`).

The chapter is mostly numbers, and the numbers are consequential. Two of them decide what a posture means; five decide where the boundaries between actions fall; one is accepted and read by nothing. The chapter states each, states how sensitive the landscape is to it, and says plainly which of the surprises a host may meet are the declared costs speaking rather than the derivation misbehaving.

**Definition (Posture as the host's cursor)** · `def:channel:posture`

The posture $\pi \in (0, 1)$ is the host's declared level of caution on this channel. It is not an input to the derivation. The landscape is computed without reference to it and describes the crossover structure at every level of caution at once; the posture selects a read-point on that structure.

Given $\pi$, the host locates $\ell(\pi) = \ln\bigl(\pi / (1 - \pi)\bigr)$ among the crossover locations and recovers the optimal action there (`alg:landscape:optimal-action`) and its fragility (`def:fragility:definition`). Posture is the navigator's heading; the landscape is the chart at every heading; the host chooses a heading and reads the chart at that bearing.

The separation is what makes the landscape reusable. A host comparing two operating points, or staging a response across several, computes one landscape and reads it twice, and the two readings are guaranteed consistent because they came from one object rather than from two derivations that might have seen different evidence (`inv:guarantee:posture-independence`). The derivation takes no posture, returns the whole landscape, and exposes the optimal-action and fragility utilities as separate posture-indexed reads.

**Table (The named posture constants)** · `tab:channel:posture-constants`

Four named constants are provided as conveniences.

| Name | $\pi$ | Reading |
| --- | --- | --- |
| Permissive | $0.1$ | Minimise friction; accept risk |
| Normal | $0.3$ | Balance friction and risk |
| Elevated | $0.6$ | Lean toward caution |
| Emergency | $0.9$ | Minimise risk; accept friction |

The host may use any value in the open unit interval. The four are conveniences and not privileged operating points: nothing in the derivation tests for them, no threshold is defined at them, and a landscape read at $0.31$ is neither more nor less well-defined than one read at Normal. They exist so that operational documentation and incident procedure have names to use, and so that a host changing posture under pressure is choosing among prepared positions rather than inventing a number.

Their meanings do not transfer between channels (`prin:channel:relativity`).

**Principle (Posture is channel-relative)** · `prin:channel:relativity`

A posture means nothing on its own. Its operational content is the pair of cost multipliers it induces (`def:landscape:posture-sensitivity`), and those depend on the channel's own sensitivity exponents (`def:channel:sensitivity-exponents`).

Elevated under $\beta_b = 3$ expresses a materially more cautious risk attitude than Elevated under $\beta_b = 1.5$: the same cursor position, the same name, and a different multiplier on the cost of missing an adverse outcome. Postures therefore do not transfer between channels whose exponents differ, and neither do the named constants. A host operating one posture dial across several channels should hold the exponents constant across them, or treat each channel's posture as its own quantity and resist the arithmetic that would average them.

There is a sharper way to say it. The landscape depends on the risk estimate only through a rigid translation of the whole crossover set along the logit axis (`thm:landscape:rigid-translation`), so on a single channel posture and prior log-odds of risk are interchangeable currencies: the posture axis is the risk axis viewed in a mirror. Two channels with different exponents put different scales on that mirror, which is exactly why a posture read on one says nothing about the other.

**Definition (The declared action set)** · `def:channel:actions`

The action set is the ordered list of actions the host can take on this channel, from most permissive to most restrictive. The standard set is Allow, Challenge, Slow, Block; any subset of at least two, in that order, is valid.

An action means one thing to the derivation: a position in the order, with a benign-class cost and an adverse-class cost attached (`tab:landscape:action-costs`). The derivation knows nothing about how the host executes it. Challenge tests the entity; Slow restricts throughput; Block denies. Those readings are the host's, and the derivation's arithmetic is identical for any four actions carrying those costs under any other names.

The order is load-bearing rather than decorative. Crossovers are computed between adjacent declared actions (`def:landscape:action-crossover`), and the reward differentials that locate them are differences between neighbours in the declared order (`def:landscape:reward-differentials`), so declaring the set out of order does not produce a differently-ordered landscape — it produces a wrong one. Any ordered subset of two or more is accepted.

**Caveat (What narrowing the action space costs)** · `cav:channel:action-space-sizing`

The declared actions should correspond to operationally distinct capabilities. Declaring an action the host cannot genuinely execute differently from its neighbour fractures the neighbour's regime without improving any decision.

| Operational capability | Recommended action set | Challenge character |
| --- | --- | --- |
| Allow or deny only | Allow, Block | No Challenge regime |
| Allow, test, or deny | Allow, Challenge, Block | Wide — about a fifth of the posture axis at defaults |
| Allow, test, throttle, or deny | Allow, Challenge, Slow, Block | Narrow — about a twentieth of the posture axis at defaults |
| Allow, throttle, or deny | Allow, Slow, Block | No Challenge regime |

The exposure runs in both directions and neither is a defect of the derivation. Declaring Slow beside Challenge when the two are the same operational response costs Challenge most of its regime, and the arithmetic is quantified rather than asserted (`ex:landscape:three-action`). Narrowing the set instead costs evidence: an action never taken produces no outcomes, and a channel that removes Challenge stops generating the challenge results its own effectiveness estimate is built from (`cav:limitation:challenge-narrow`). A host who finds the four-action Challenge regime surprisingly narrow should first check that Slow is a meaningfully different response, and adopt the three-action set if it is not.

**Table (The reward parameters)** · `tab:channel:reward-parameters`

Seven declared parameters drive every crossover position on the channel.

| Parameter | Symbol | Default | Carrying |
| --- | --- | --- | --- |
| Pass | $R_p$ | $1.0$ | Opportunity cost of full denial |
| Friction | $R_f$ | $0.3$ | Cost of challenging a benign case |
| Missed | $R_m$ | $3.0$ | Cost when an adverse outcome is not prevented |
| Caught | $R_c$ | $2.0$ | Value of identifying an adverse source |
| Blocked | $R_b$ | $1.5$ | Cost of denial to a benign case |
| Slow severity | $\alpha_s$ | $0.2$ | Throttle severity relative to full restriction |
| Block catches | $\beta_c$ | $1.0$ | Fraction of challenge catching retained under Block |

The last two are extended parameters: they appear only inside the derivation's cost model and their per-action semantics are stated there (`def:landscape:extended-rewards`) rather than restated here. Only ratios matter, since the crossover locations depend on the parameters through the logarithm of a differential ratio (`thm:landscape:rigid-translation`) — a channel that doubles every reward gets the identical landscape.


**Data (Crossover position against the reward ratio)** · `data:channel:reward-sensitivity`

The Allow-to-Challenge transition is the most consequential boundary on most channels. Its position is governed by the ratio of the benign-class cost of challenging, $\Delta_\text{good} = R_f$, to the adverse-class benefit of challenging, $\Delta_\text{bad} = \hat{q}_c(R_m + R_c)$.

| $R_m$ | $R_f$ | Ratio | $\pi^*$ at $\hat{p} = 0.1$ | at $\hat{p} = 0.3$ | at $\hat{p} = 0.5$ |
| --- | --- | --- | --- | --- | --- |
| $3$ (default) | $0.3$ | $1{:}8$ | $0.508$ | $0.375$ | $0.300$ |
| $10$ | $0.3$ | $1{:}20$ | $0.421$ | $0.297$ | $0.232$ |
| $30$ | $0.3$ | $1{:}53$ | $0.329$ | $0.222$ | $0.169$ |
| $100$ | $0.3$ | $1{:}170$ | $0.236$ | $0.152$ | $0.114$ |

Computed at $\hat{q}_c = 0.5$, $\beta_b = 1.5$, $\beta_g = 1.0$, $R_c = 2$.

As the ratio falls — missing an adverse outcome growing more expensive relative to friction — the crossover moves toward Permissive and the Allow regime shrinks. That is the correct decision-theoretic behaviour and not a saturation artefact: if the host has declared that a miss costs a hundred and seventy times what a challenge does, almost everything should be challenged.

**Data (How the challenge estimate moves the crossover)** · `data:channel:challenge-interaction`

Challenge effectiveness scales the adverse-class benefit of challenging linearly, so the Allow-to-Challenge crossover moves with it.

At the default rewards and $\hat{q}_c = 0.5$, the crossover sits at $\pi^* = 0.508$ for a risk estimate of $0.1$. At $\hat{q}_c = 0.1$ — challenges rarely identifying an adverse source — the same crossover moves to $0.663$: much less of the posture axis favours Challenge, because challenging buys much less. The reward-sensitivity table is therefore read together with the Companion's current posterior (`alg:companion:inference`) and not on its own.

The posterior's spread matters as well as its centre. Higher challenge uncertainty widens the credible interval of every crossover that depends on $\hat{q}_c$, through that crossover's sensitivity term (`thm:landscape:crossover-covariance`), and it does so without moving the point estimate at all. A host seeing wide boundaries and a stable modal action is looking at challenge uncertainty rather than risk uncertainty, and the decomposition says which (`def:fragility:definition`). The derivation reads the point estimate into the policy constants and the variance into each crossover's uncertainty.

**Remark (Verify the declared costs before reading the landscape)** · `rem:channel:verify-costs`

A host who finds the crossover positions surprising should suspect the declaration before the derivation.

The landscape is a faithful rendering of the declared reward structure. If it says challenge nearly everything, the declared cost of a miss says that missing an adverse outcome is very expensive relative to friction; if the Challenge regime is a sliver, the declared action set says Slow is a distinct response and the declared costs say it is nearly as good as challenging for adverse traffic. In both cases the remedy is the declaration, not the landscape (`cav:limitation:reward-sensitivity`).

The practical form is a short pre-flight. Are all declared actions genuinely distinct operational responses (`cav:channel:action-space-sizing`)? Are the five costs on one scale, and do their ratios match what the business would actually trade? Are the exponents the ones this channel's risk attitude wants, rather than the ones another channel used (`def:channel:sensitivity-exponents`)? A landscape derived from unexamined defaults is a decision-theoretic statement the host has not actually made.

**Proposition (The challenge regime width)** · `prop:channel:challenge-width`

In the four-action set the Challenge regime's width in logit space is fixed by the reward structure and the challenge estimate alone:

$$w_C = \frac{1}{\beta_b + \beta_g}\left[\ell(\hat{q}_c) + \ln\!\left(1 + \frac{R_c}{R_m}\right)\right]$$

At the default $\hat{q}_c = 0.5$ and $R_c / R_m = 2/3$ this is $0.204$ logit units, about a twentieth of the posture axis. The width is independent of $\alpha_s$, $R_f$, $R_b$, $R_p$ and $\beta_c$; the slow-severity parameter enters both differentials at the Challenge-to-Slow transition as a common factor and cancels, so it governs the Slow regime's width and not this one. It is independent of the risk estimate for the same reason every regime width is (`thm:landscape:rigid-translation`).

Three levers widen it, and only three. Raise the catching value relative to the cost of a miss; supply evidence of high challenge effectiveness through the Companion; or remove Slow from the action set, which is by far the largest of the three and takes Challenge to about a fifth of the axis (`ex:landscape:three-action`). Each interior landscape regime reports the width and its variance directly, and the rendering expresses the width through its magnitudes, so a host can observe any of the three levers acting.

**Definition (The sensitivity exponents)** · `def:channel:sensitivity-exponents`

Two exponents govern how fast the cost of each kind of error grows with posture: $\beta_b$ for missing adverse observations, $\beta_g$ for restricting benign ones. Defaults: $\beta_b = 1.5$, $\beta_g = 1.0$.

They are policy fields of the channel and not derivation constants, because they are a statement about the host's risk attitude rather than about the arithmetic: a channel whose adverse outcomes are catastrophic and whose friction is cheap declares a high $\beta_b$, and one protecting a latency-sensitive path declares a low one. Their sum appears as the divisor of every crossover location and of the shared uncertainty (`thm:landscape:rigid-translation`), so raising both together compresses the whole landscape toward the centre of the posture axis without reordering anything, while raising one alone shifts it.

Because they fix what a posture means, they are the mechanism behind posture's channel-relativity (`prin:channel:relativity`), and a host that changes them has changed the meaning of every posture previously recorded on the channel.

**Definition (The neutral zone)** · `def:channel:neutral-zone`

The neutral zone is a declared policy field, default $0.1$, which affects neither the landscape nor any decision read from it.

It is accepted and inert by design. The quantity it names — the band around neutrality within which a classification is reported as unremarkable — is a property of the display spectrum and is defined by the rendering layer (`def:rendering:magnitudes`), where it can be changed without changing any decision. Admitting it in the channel policy keeps one declaration site for everything the host says about a channel; reading it in the derivation would make a decision boundary a function of a display parameter, which the presentation-free invariant forbids (`inv:landscape:presentation-free`).

The field is therefore specified as carried and unread, and an implementation that begins reading it in the derivation has broken an invariant rather than fixed an oversight.
