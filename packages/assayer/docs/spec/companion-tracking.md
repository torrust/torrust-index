## Part (The Companion Tracker) · `part:spec:companion-tracker`

Part IV took a challenge posterior as one of the derivation's three inputs and specified the boundary it crosses without saying where it comes from. Part V specifies the component that maintains it: a Beta-Binomial conjugate model over the binary evidence stream of challenge outcomes, decaying toward its prior as that evidence ages, and answering with a point estimate, a variance, and — where the host wants exactness rather than moments — the conjugate shape itself.

The Companion is the smallest component in the architecture and the only one that can be replaced wholesale without touching anything else. It reads no Core state, holds none, and is not reached by the Core at any point; the host feeds it, reads it, and hands what it reads to the derivation. That independence is the Part's organising claim, and the last environment of the chapter is where it is stated as an obligation rather than described as a habit.

### Chapter (Challenge Effectiveness Tracking) · `chap:spec:challenge-effectiveness`

The chapter fixes what challenge effectiveness is, how it is estimated, how the estimate ages, and how a host that knows better than the estimate says so. The order is the order of the model's own life: what the quantity means, what state carries it, what the state begins as, how it is read, how it is written, how thin the evidence is, how it decays, and the two recorded decisions that fix the decay's form and its rate. The override and the injection follow, because they are the answers to the sparsity the rate environment quantifies, and the health report, the convergence bound and the boundary invariant close the chapter with what a host can see, what it must wait for, and what it may rely on.

**Definition (Challenge effectiveness)** · `def:companion:purpose`

Challenge effectiveness $q_c$ is the probability that a challenge correctly identifies an adverse source. It is a property of the challenge mechanism, not of any request, and it enters the decision landscape by two separate channels.

The point estimate $\hat{q}_c$ moves the crossover positions: a high value moves the boundary between allowing and challenging toward the permissive end, because challenges are worth deploying earlier when they work, and a low value moves it the other way (`eq:landscape:crossover`). The posterior variance $\sigma^2_{\hat{q}_c}$ widens the crossover credible intervals through the posture-sensitivity terms, expressing uncertainty about where on the posture axis each boundary belongs (`def:landscape:credible-intervals`). The posterior's *shape*, where it is available, governs the dominance probabilities exactly rather than through a moment approximation (`thm:landscape:dominance`).

Two channels, one quantity, and neither of them a Core concern: the Core's risk estimate is unchanged by any value of $q_c$ whatsoever.

**Definition (The tracker's state)** · `def:companion:state`

A tracker carries five things: two pseudo-counts, the time of its last write, its configuration, and an optional host override.

| Field | Carrying |
| --- | --- |
| $\alpha$ | The pseudo-count of challenge failures — an adverse source identified |
| $\beta$ | The pseudo-count of challenge passes — an adverse source not identified |
| $t_\text{last}$ | The time of the most recent write, from which elapsed-time decay is measured |
| Configuration | The two prior pseudo-counts, the decay rate, and the injection ceiling |
| Override | The host-supplied estimate and variance, when one is in force |

The host creates one tracker per channel, or shares one across channels whose challenge mechanisms are identical, and nothing in the state refers to the Core: no model parameters, no feature vector, no Ledger entry, no assessment identifier. The initial state is the prior (`def:companion:prior`).

**Definition (The prior)** · `def:companion:prior`

The default prior is $\text{Beta}(1, 1)$: the uniform distribution on the unit interval, expressing no directional belief about whether challenges work. Its point estimate is $0.5$ and its variance is $1/12$. A host with a view configures the two prior pseudo-counts to express it.

| Prior | $\hat{q}_{c,0}$ | $\sigma^2_0$ | The belief it expresses |
| --- | --- | --- | --- |
| $\text{Beta}(1, 1)$ | $0.50$ | $0.0833$ | No prior knowledge; the default |
| $\text{Beta}(2, 2)$ | $0.50$ | $0.0500$ | Moderate effectiveness expected |
| $\text{Beta}(5, 2)$ | $0.71$ | $0.0256$ | Challenges usually work |
| $\text{Beta}(2, 5)$ | $0.29$ | $0.0256$ | Challenges usually fail to catch |
| $\text{Beta}(5, 5)$ | $0.50$ | $0.0227$ | Confident that effectiveness is moderate |

The prior is also the point the decay returns to, which is why a host choosing one is choosing two things at once: where the estimate starts and where it ends if the evidence stops (`alg:companion:decay`).

**Algorithm (Reading the estimate)** · `alg:companion:inference`

A read returns the posterior at the moment of reading. The pseudo-counts are first decayed to the present as a pure function of elapsed time, without mutating anything, and the moments follow from the decayed pair:

$$\hat{q}_c = \frac{\alpha}{\alpha + \beta}, \qquad \sigma^2_{\hat{q}_c} = \frac{\alpha\beta}{(\alpha + \beta)^2(\alpha + \beta + 1)}$$

The effective sample size is $n_\text{eff} = (\alpha + \beta) - (\alpha_0 + \beta_0)$: the data-contributed pseudo-counts net of the prior. Two reads are specified over the same state. One returns the moments, for a host consuming a point and a variance directly. The other returns the posterior the derivation takes as its third input, carrying the conjugate shape where no override is in force and a moment-matched summary where one is (`sig:companion:posterior`). Reading decays and does not write, which is the Ledger's convention applied to a second component for the same reason: a read must not be a write (`alg:temporal:lazy-application`). Each estimate, posterior and health read computes the decayed pseudo-counts as locals, so an idle tracker regresses toward its prior without the read mutating stored state.

**Algorithm (The conjugate update)** · `alg:companion:update`

On a contributing label the stored counts are decayed to the label's timestamp and then incremented by one, a failure to $\alpha$ and a pass to $\beta$:

$$\alpha \leftarrow \alpha + \mathbb{1}[\text{fail}], \qquad \beta \leftarrow \beta + \mathbb{1}[\text{pass}]$$

This is the exact conjugate update for a Beta prior under Binomial evidence. No approximation accumulates over observations, however many arrive and in whatever order, which is why the tracker's uncertainty can be trusted at every sample size rather than only asymptotically.

A label contributes when three conditions hold jointly: the host challenged, the outcome was adverse under the sign convention (`conv:valence:sign`), and the challenge produced an observable result. A challenge of a benign request satisfies the first and third and is not evidence, because $q_c$ is defined against adverse sources specifically and a benign request passing a challenge says nothing about the mechanism's catch rate. Feeding the tracker is the host's duty; nothing in the Core routes a label here.

**Data (The contributing-label rate)** · `data:companion:contributing-rate`

The evidence base is thin, and the arithmetic that says how thin is worth carrying because it determines everything else in the chapter:

$$R_\text{contributing} = R_\text{label} \times f_\text{chall} \times f_\text{adv|chall} \times f_\text{obs}$$

the factors being the total label rate, the fraction of labelled requests challenged, the fraction of those with an adverse outcome, and the fraction with an observed result. At a one-percent challenge rate, a five-percent adverse rate and eighty-percent observability, four requests in ten thousand contribute.

| Targeting quality | $f_{\text{adv\|chall}}$ | Contributing labels per day | Days for twenty to arrive |
| --- | --- | --- | --- |
| Untargeted, at the population rate | $0.05$ | $0.08$ | $250$ |
| Weakly targeted | $0.15$ | $0.24$ | $83$ |
| Moderately targeted | $0.30$ | $0.48$ | $42$ |
| Well targeted | $0.50$ | $0.80$ | $25$ |

The last column counts arrivals and not what survives them, which is the distinction the convergence bound turns on: at the untargeted rate twenty contributing labels arrive and twenty effective samples never stand together (`bound:companion:convergence`).

The table is stated at two hundred labels a day. Targeting is the only factor a host can move by an order of magnitude, and moving it is exactly what the landscape is for, so a deployment that uses the landscape to aim its challenges buys its own evidence six to ten times faster than one that does not.

**Algorithm (Pseudo-count decay)** · `alg:companion:decay`

At each write, with $\Delta t$ the hours since the last one, the stored counts are blended toward the prior before the write is applied:

$$\alpha \leftarrow \alpha_0 + \gamma_{q,t}^{\Delta t}(\alpha - \alpha_0), \qquad \beta \leftarrow \beta_0 + \gamma_{q,t}^{\Delta t}(\beta - \beta_0)$$

Data-contributed pseudo-counts decay exponentially and the prior does not, so the point estimate regresses toward the prior mean and the variance rises toward the prior variance. Both are correct when evidence goes stale: an estimate held up by year-old observations should say so in its uncertainty and should not go on asserting a mean it no longer has grounds for.

| Time since the last contributing label | Evidence retained |
| --- | --- |
| One week | $96.6\%$ |
| One month | $86.7\%$ |
| Three months | $65.2\%$ |
| Six months | $42.5\%$ |
| One year | $17.3\%$ |

At a constant arrival rate $\lambda_c$ per day the steady-state effective sample size is $n_\text{eff,ss} = \lambda_c / (-24 \ln \gamma_{q,t})$: about $167$ at $0.8$ contributing labels a day, and about $17$ at $0.08$.

**Decision (Blending toward the prior rather than scaling the counts)** · `dec:companion:decay-form`

The alternative form multiplies both counts by the decay factor and floors them at the prior, preserving their ratio exactly. It answers a narrower question — how certain am I about what I last measured — and leaves the point estimate where it was. It is the right form when the challenge mechanism is known to be stable and only precision degrades with age.

The blend toward the prior is specified as the default because it makes the weaker assumption. Stale evidence should cost the host its confidence *and* its position, and a host that wants to keep the position has to say so.

The choice is not free. Blending pulls the steady-state point estimate toward the prior mean: at $n_\text{eff,ss} = 17$ and a true $q_c$ of $0.9$ the bias is about $-0.04$, displacing the allow-to-challenge crossover by roughly one hundredth of a logit; at $n_\text{eff,ss} = 167$ it is about $-0.005$ and disappears into the noise. A host with a stable mechanism and a thin evidence stream may prefer the multiplicative form, and that is a configuration choice rather than a change of default.

**Decision (The decay rate)** · `dec:companion:decay-rate`

The rate fixes the trade between steady-state precision and responsiveness to a mechanism that has actually changed, and the two pull in opposite directions over the same parameter.

A faster rate — a twenty-nine-day half-life, say — would notice a mechanism change within weeks, at the cost of a steady-state effective sample size near three at the pessimistic arrival rate, where the prior would dominate permanently and the tracker would be an expensive way to return $0.5$. A slower rate — a five-hundred-and-seventy-eight-day half-life — would accumulate more evidence and go on asserting a catch rate the mechanism stopped having years earlier.

The default sits between them on the assumption that challenge mechanisms change on a timescale of months. A host that has just changed its own mechanism should not wait for the decay to discover it from sparse evidence; it should override or inject (`alg:companion:override`) and (`alg:companion:injection`).

**Definition (The challenge-effectiveness decay rate)** · `def:companion:challenge-decay`

The rate is $\gamma_{q,t} = 0.9998$ per hour, a half-life of about one hundred and forty-five days. It is the Companion's own parameter and belongs to the Companion: it governs how fast challenge-outcome evidence ages and nothing else, and it is independent of every Core rate, none of which it is derived from or constrained by.

The siting matters because the rate is easy to mistake for a property of the channel it is read against. It is not. Two channels sharing one challenge mechanism share this rate because they share the tracker, and two channels with different mechanisms differ in it because they have different trackers — in neither case does the channel's declared reward structure have anything to say about it. The rate therefore belongs to the Companion tracker configuration and not to the channel policy's reward parameters (`tab:channel:reward-parameters`).

**Algorithm (The host override)** · `alg:companion:override`

A host that knows the catch rate better than the evidence does may say so. An override supplies a point estimate and, optionally, a variance; while it is in force both reads return the supplied values, the second as a moment-matched posterior rather than a conjugate one. Where the variance is omitted the default is $q_c(1 - q_c)/101$ — the variance of about a hundred pseudo-observations at the overridden value, which is confident without being absolute and is why the moment-matched posterior downstream is faithful rather than degenerate.

Beneath the override the Beta model goes on accumulating and decaying as usual, so clearing the override reveals a model that has been learning throughout rather than one frozen at the moment of the override.

The override is the single highest-leverage action available at deployment. An untargeted deployment never reaches an effective sample size of twenty at all, and a weakly targeted one waits about a hundred and eight days for it (`bound:companion:convergence`); a host with a pilot study, a historical figure, or an informed guess waits for neither, and leaves the Core's own convergence as the only thing to wait for (`bound:resource:convergence-budget`). A host using the derivation should override at deployment unless it truly has no basis for an estimate.

**Algorithm (Evidence injection)** · `alg:companion:injection`

Injection adds pseudo-counts directly: so many failures, so many passes, decayed to the present first and then added. Both arguments must be non-negative and neither need be an integer, since discounted external evidence is naturally fractional. Injected counts age at the same rate as observed ones, because they are the same kind of thing.

A ceiling, one thousand by default, caps each injection; a larger value is clamped and the clamping is reported. Without it a single injection could seat itself so deep that the decay would take years to reach it, which would make the non-stationarity handling of this chapter decorative.

Injection and override are independent and compose. An override governs what the reads return; an injection changes what the model knows. A host with a precise external figure overrides, a host with external counts injects, and a host with both injects the counts, overrides with the figure, and clears the override once the model has absorbed the evidence.

**Table (The Companion health report)** · `tab:companion:health`

The tracker reports its own health, separately from the Core's, because it converges separately and fails separately.

| Field | Carrying |
| --- | --- |
| Estimate and variance | The current posterior moments, from the override where one is in force |
| Pseudo-counts | The raw $\alpha$ and $\beta$, always from the Beta model |
| Effective sample size | The data-contributed counts net of the prior |
| Sufficiency verdict | Whether that effective sample size has reached the host-owned floor, twenty by default, which an override does not bring forward |
| Prior contribution | The prior's share of the total mass, so a host can see when it dominates |
| Contributing labels, lifetime | How much evidence has ever arrived |
| Contributing rate | An exponentially weighted estimate of arrivals per day |
| Variance contribution fraction | The share of crossover variance attributable to $\hat{q}_c$ |
| Override state | Whether an override is in force, and its values |
| Days since the last contributing label | Wall-clock silence, which the decay is measured in |

The variance contribution fraction is the most actionable field. Above one half, the crossover intervals are dominated by challenge uncertainty and the host should override, inject, or challenge more; between a tenth and a half it is a material contributor worth watching; below a tenth it is negligible. Its per-request generalisation is the landscape's challenge share (`def:fragility:definition`), and this is the fleet-level summary of the same quantity. The ten rows together distinguish a thin tracker from a stale one.

The sufficiency verdict is a comparison and not a second sample-size field. Its floor belongs to the Companion beside the tracker whose health report reads it, rather than to the Core convergence configuration (`dec:challenge:sufficiency-floor-owned-here`), and what the verdict compares against that floor is the override-independent effective sample size this chapter's convergence bound fixes (`bound:companion:convergence`). Reporting it gates nothing (`dec:health:reports-never-gates`).

**Bound (Convergence)** · `bound:companion:convergence`

The Companion converges independently of the Core. Counting contributing labels as they arrive and ignoring what decays between them, the time for a target effective sample size to arrive is the target over the contributing rate:

$$T_{\hat{q}_c} = \frac{n_\text{target}}{R_\text{contributing}}$$

At twenty target observations this arrival time is two hundred and fifty days untargeted, eighty-three weakly targeted, forty-two moderately targeted, and twenty-five well targeted. An override is not a fifth entry of zero in that list. It makes the reported estimate usable at once (`alg:companion:override`), and that is the whole of what it shortens: the effective sample size the health report carries (`tab:companion:health`) is taken from the decayed pseudo-counts net of the prior whether or not an override is in force, so the evidence a channel must observe before it counts as sufficient accrues under an override at exactly the rate it would without one.

That arithmetic is the no-decay approximation and is kept here as one, because the correction it needs is not a rounding. Pseudo-counts decay between arrivals (`alg:companion:decay`), so a channel fed at a regular rate does not accumulate without limit. With one contributing label every $h = 24 / R_\text{contributing}$ hours the effective sample size approaches

$$C_+ = \frac{1}{1 - \gamma_{q,t}^{\,h}}$$

immediately after a contribution, and $C_+ - 1$ immediately before the next. A target above $C_+$ is never reached at that rate however long the host waits, and a target between the two is reached but not held between contributions:

| Targeting quality | Contributing labels per day | Ceiling $C_+$ | Time to $n_\text{eff} = 20$ |
| --- | --- | --- | --- |
| Untargeted, at the population rate | $0.08$ | $17.17$ | Never |
| Weakly targeted | $0.24$ | $50.50$ | $108$ days |
| Moderately targeted | $0.48$ | $100.49$ | $48$ days |
| Well targeted | $0.80$ | $167.15$ | $28$ days |

So twenty effective observations are reached and held by weakly, moderately and well targeted channels, and by no untargeted one. The two hundred and fifty days above is the time twenty contributing labels take to *arrive* at the population rate, not a time at which twenty of them are still standing: at that rate the decay carries evidence away faster than arrivals replace it well before twenty accumulates, and the ceiling is a little over seventeen. The three tiers that do converge take longer than the no-decay row says — a hundred and eight days rather than eighty-three, forty-eight rather than forty-two, and twenty-eight rather than twenty-five.

Full system convergence is the larger of the two components' times, so the Companion is usually what binds: the Core reaches useful estimates in about ten days at the reference configuration, and the Companion does not (`bound:resource:convergence-budget`). A host using assessments without the derivation is unaffected, because it never reads $q_c$ at all.

The bound also carries an exposure it cannot remove. The quantity is really the catch rate over the population the host chooses to challenge, and better targeting changes that population — plausibly toward more capable adversaries — which the tracker reads as drift and re-converges over its long decay horizon while the crossovers move under it (`cav:limitation:challenge-policy-loop`). An override inherits the same relativity rather than escaping it, being calibrated on some earlier population of its own. Both the sparsity and the policy loop are disclosed among the document's limitations (`cav:limitation:challenge-thin`) and (`cav:limitation:challenge-convergence`).

**Invariant (The Companion's boundary)** · `inv:companion:boundary`

The Companion has three interfaces and they are of three different kinds.

To the Core: none. It reads no model parameter, no feature vector, no Ledger entry and no Core state of any description, it writes nothing there, and it influences no risk estimate, no importance weight, no eligibility determination and no learned quantity. The Core, symmetrically, holds no Companion state (`inv:guarantee:companion-independence`).

To the derivation: one, read-only. At each derivation the host supplies the Companion's posterior as an argument; the derivation evaluates quantiles and tail probabilities from it and does not modify it, feed back into it, or retain anything derived from it (`sig:companion:posterior`).

To the host: one, bidirectional. The host feeds contributing labels, reads the estimate, the posterior and the health report, and may override or inject.

Three interfaces, one of them empty, is the whole boundary, and the emptiness is the load-bearing one: it is what lets the Companion be replaced, or omitted entirely, without any consequence for what the Core learns. The boundary appendix lists it alongside the document's other crossings (`app:spec:interface-map`).

**Requirement (The replacement surface)** · `req:companion:replacement-trait`

The Beta-Binomial tracker is one estimator of challenge effectiveness and the specification requires no more of a replacement than an interface: a method returning the scalar estimate, and a posterior method defaulting to a moment-matched Beta built from that estimate.

The defaulting is what makes the surface worth having. A provider of a scalar and a variance — an external analytics pipeline, a lookup table over historical trials, a constant a host is confident in — participates fully in the landscape's interval and dominance machinery at first-order fidelity without implementing anything further, while a provider holding a genuine conjugate posterior overrides the second method and gets exact quantiles. The only constraints are that the estimate lie strictly inside the unit interval and the variance be strictly positive.

Neither the Core nor the derivation is affected by which provider is in place, which is the point: this is the one extension point in the architecture where a host can substitute its own science. The provider trait admits both the exact decayed posterior and a scalar-only provider through the specified moment-matched default.
