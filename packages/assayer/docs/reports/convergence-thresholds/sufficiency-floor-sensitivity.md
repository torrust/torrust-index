# Companion sufficiency floor: sensitivity to arrival and decay · `rep:challenge:sufficiency-floor-sensitivity`

This study is evidence for a decision, not the decision. The sufficiency floor moved to the host-owned Companion at its shipped value of fifty while the value question remained open (`cav:challenge:sufficiency-threshold-open`); `packages/assayer/adr/challenge.md:165-189`. The specification instead targets twenty effective observations, and its warm-up reading aid distinguishes about ten contributing labels for a first useful estimate from about twenty for practically useful variance (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321` and (`tab:warmup:milestones`); `packages/assayer/docs/spec/analysis-warmup.md:166-190`.

**What the source counts**

Each channel stores current Beta counts $\alpha,\beta$ separately from prior counts $\alpha_0,\beta_0$. The default prior is $\operatorname{Beta}(1,1)$ and the default hourly retention factor is $\gamma=0.9998$ (`packages/assayer/src/risk/challenge.rs:85-116` and `packages/assayer/src/risk/challenge.rs:175-225`). At a read time $t$, the effective sample size is exactly

$$n_\text{eff}(t)=(\alpha(t)+\beta(t))-(\alpha_0+\beta_0).$$

The prior therefore contributes zero effective samples. A normal observed outcome first decays both data-contributed count deltas and then adds exactly one to either $\alpha$ or $\beta$; it never adds to both (`packages/assayer/src/risk/challenge.rs:234-269` and `packages/assayer/src/risk/challenge.rs:381-402`).

External evidence uses the same counts but has a different increment. Both failure and pass arguments must be finite and non-negative, and *each* is clamped independently to the channel's injection ceiling, one thousand by default, after decay and before addition (`packages/assayer/src/risk/challenge.rs:189-225` and `packages/assayer/src/risk/challenge.rs:290-327`). One call on a default channel can add at most two thousand effective samples, not one thousand in total; repeated calls have no cumulative ceiling. None of the candidate floors in this study is globally unreachable if a host injects evidence. All later “unreachable” results mean unreachable from regularly spaced, one-count observed outcomes without injection.

**The actual decay law and its laziness**

For an elapsed $\Delta h$ hours the source computes

$$d=\gamma^{\Delta h},\qquad \alpha' = \alpha_0+d(\alpha-\alpha_0),\qquad \beta' = \beta_0+d(\beta-\beta_0).$$

It follows directly, rather than from a textbook approximation, that decay maps $n_\text{eff}$ to $d n_\text{eff}$. The exact algebraic half-life of the shipped $\gamma=0.9998$ is $\ln(1/2)/\ln(0.9998)=3{,}465.389$ hours, or $144.391$ days; the source accurately describes that as about one hundred and forty-five days (`packages/assayer/src/risk/challenge.rs:189-205` and `packages/assayer/src/risk/challenge.rs:234-256`).

Decay is lazy in two distinct ways. A write decays stored counts first, applies the increment, and advances `t_last`; a read computes decayed local counts and does not mutate the state (`packages/assayer/src/risk/challenge.rs:248-269` and `packages/assayer/src/risk/challenge.rs:330-373`). The shared elapsed-time helper treats a backward clock as zero elapsed time and clamps any single gap to 8,760 hours, so a read more than a year after the last write applies one year's decay rather than the whole gap (`packages/assayer/src/types.rs:321` and `packages/assayer/src/types.rs:418-437` and `packages/assayer/src/numerics.rs:61-115`). The targeting tiers below have interarrival gaps of only thirty to three hundred hours, so this clamp does not affect their steady-state results.

**What the health report says**

The tracker owns a default floor of fifty effective samples beyond the prior, relocated unchanged and explicitly marked as undecided (`packages/assayer/src/risk/challenge.rs:428-443`). The report method does not implicitly read that constant: its caller supplies a threshold. For every registered channel it publishes the current point estimate and variance, effective sample size, the inclusive comparison $n_\text{eff}\mathrel{\geq}\text{threshold}$, and the override flag and values (`packages/assayer/src/risk/challenge.rs:547-605`).

An active override replaces the reported estimate and variance but neither the underlying effective sample size nor its sufficiency comparison. Observed and injected evidence continue to change the Beta state under the override (`packages/assayer/src/risk/challenge.rs:270-317` and `packages/assayer/src/risk/challenge.rs:550-570`). Thus the specification's zero convergence time under an override describes immediate operational use of the override, not the health boolean becoming true (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321`.

**The specification's rates and decay-free times**

The contributing-rate table fixes four tiers at two hundred labels a day: 0.08 untargeted, 0.24 weakly targeted, 0.48 moderately targeted, and 0.80 well targeted contributing labels a day (`data:companion:contributing-rate`); `packages/assayer/docs/spec/companion-tracking.md:133-153`. The convergence bound then states $T=n_\text{target}/R$ and displays two hundred and fifty, forty-two, and twenty-five days for the first, third, and fourth tiers at target twenty (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321`.

The exact arithmetic is $20/0.08=250$, $20/0.48=41.666\ldots$, and $20/0.80=25$ days. The displayed forty-two is therefore whole-day rounding, not an exact source for a rate: dividing twenty by forty-two would recover 0.476190 rather than the table's exact 0.48. The omitted weak tier gives $20/0.24=83.333\ldots$ days, displayed elsewhere as eighty-three (`packages/assayer/docs/spec/analysis-warmup.md:145-154`).

Applied mechanically, the bound gives this decay-free sensitivity table. Sixteen is included as the source-derived candidate established below.

| Targeting tier | Rate/day | Floor 10 | Floor 16 | Floor 20 | Floor 50 |
| --- | ---: | ---: | ---: | ---: | ---: |
| Untargeted | 0.08 | 125.0 d | 200.0 d | 250.0 d | 625.0 d |
| Weakly targeted | 0.24 | 41.7 d | 66.7 d | 83.3 d | 208.3 d |
| Moderately targeted | 0.48 | 20.8 d | 33.3 d | 41.7 d | 104.2 d |
| Well targeted | 0.80 | 12.5 d | 20.0 d | 25.0 d | 62.5 d |

These are not source-accurate times to *effective* samples. They count arrivals without subtracting what decays between them. That omission is material even at twenty and becomes decisive at a steady-state ceiling.

**Source-accurate time to the first crossing**

For a regular rate $R$ per day, let one observed outcome arrive every $h=24/R$ hours and let $x_k$ be effective samples immediately after outcome $k$. The source's decay-before-increment order gives

$$d=\gamma^h,\qquad x_{k+1}=d x_k+1,\qquad x_k=\frac{1-d^k}{1-d}.$$

Outcome mix does not enter: failure and pass each add one effective sample (`packages/assayer/src/risk/challenge.rs:248-269` and `packages/assayer/src/risk/challenge.rs:381-402`). Starting at the prior, with the first contribution one interarrival period after construction, the exact first crossing and the time after which the boolean stays true throughout the following interarrival periods are:

| Targeting tier | Floor 10, first / sustained | Floor 16, first / sustained | Floor 20, first / sustained | Floor 50, first / sustained |
| --- | ---: | ---: | ---: | ---: |
| Untargeted | 187.5 / 212.5 d | 562.5 / 950.0 d | Never | Never |
| Weakly targeted | 50.0 / 50.0 d | 83.3 / 83.3 d | 108.3 / 108.3 d | 966.7 d / never |
| Moderately targeted | 22.9 / 22.9 d | 37.5 / 37.5 d | 47.9 / 47.9 d | 143.8 / 145.8 d |
| Well targeted | 13.8 / 13.8 d | 21.3 / 21.3 d | 27.5 / 27.5 d | 75.0 / 75.0 d |

“Sustained” means that a read immediately before the next regularly spaced outcome also passes; it is not a source latch. The source recomputes the inclusive comparison on every read, so a channel can regress below the floor as evidence ages (`packages/assayer/src/risk/challenge.rs:330-402`). Real arrivals need not be regular: bursts can cross a floor that their long-run average cannot sustain, followed by a decay below it. The table is therefore the exact regular- cadence interpretation of the specification's constant-rate premise, not an empirical forecast.

**The steady-state ceiling**

Taking $k$ to infinity in the source recurrence yields two limits. Immediately after a regular contribution the ceiling is

$$C_+(R)=\frac{1}{1-\gamma^{24/R}},$$

and immediately before the next it is

$$C_-(R)=\frac{\gamma^{24/R}}{1-\gamma^{24/R}}=C_+(R)-1.$$

The specification already gives the continuous-flow approximation $C_\text{flow}=R/(-24\ln\gamma)$ and calls it about seventeen at 0.08 a day and one hundred and sixty-seven at 0.80 a day (`alg:companion:decay`); `packages/assayer/docs/spec/companion-tracking.md:155-179`. The source's discrete write semantics put that approximation between the pre- and post-write limits:

| Targeting tier | $C_\text{flow}$ | $C_+$ | $C_-$ | Clears 20? | Clears 50? |
| --- | ---: | ---: | ---: | --- | --- |
| Untargeted, 0.08/day | 16.665 | 17.170 | 16.170 | Never | Never |
| Weakly targeted, 0.24/day | 49.995 | 50.497 | 49.497 | Sustained | After writes only |
| Moderately targeted, 0.48/day | 99.990 | 100.491 | 99.491 | Sustained | Sustained |
| Well targeted, 0.80/day | 166.650 | 167.150 | 166.150 | Sustained | Sustained |

This exposes an internal premise error. The convergence bound calls twenty an effective-sample target and says the untargeted tier reaches it after two hundred and fifty days, but the same specification's decay section predicts a steady-state value of about seventeen, and the shipped discrete law has a hard regular-cadence post-write ceiling of 17.170. Untargeted observation therefore never reaches twenty under the assumptions used to state the bound; 250 days is the raw-arrival time, not the effective-sample convergence time (`alg:companion:decay`); `packages/assayer/docs/spec/companion-tracking.md:155-179` and (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321`.

For any floor $F>1$, the critical rate for *ever* crossing after a regular write is obtained from $C_+(R)=F$:

$$R_\text{ever}(F)=\frac{24\ln\gamma}{\ln(1-1/F)}.$$

At or below that rate the finite recurrence approaches the floor from below and never reaches it. The stronger rate for eventually staying above the floor throughout the interval follows from $C_-(R)=F$:

$$R_\text{sustain}(F)=\frac{24\ln\gamma}{\ln(F/(F+1))}.$$

| Floor | $R_\text{ever}$/day | $R_\text{sustain}$/day | Realistic tier verdict |
| ---: | ---: | ---: | --- |
| 10 | 0.04556 | 0.05037 | Every stated tier sustains it |
| 16 | 0.07438 | 0.07918 | Every stated tier sustains it |
| 20 | 0.09359 | 0.09839 | Untargeted never; the other three sustain it |
| 50 | 0.23762 | 0.24242 | Untargeted never; weak can cross only after writes; moderate and well sustain it |

These critical rates describe observed one-count evidence. A host can bypass the observed-arrival ceiling by injecting counts, while an override makes the estimate operational immediately but leaves the boolean unchanged (`packages/assayer/src/risk/challenge.rs:270-317` and `packages/assayer/src/risk/challenge.rs:520-570`).

**Numeric cross-check**

Two one-off Python 3 standard-library calculations evaluated the closed forms and iterated the recurrence. No script was retained. Their relevant output was:

```text
exact_half_life_hours=3465.389318
exact_half_life_days=144.391222
tier=untargeted rate=0.08/day continuous=16.665000 post=17.170000 pre=16.170000
tier=weakly     rate=0.24/day continuous=49.995000 post=50.496667 pre=49.496667
tier=moderately rate=0.48/day continuous=99.990000 post=100.490833 pre=99.490833
tier=well       rate=0.80/day continuous=166.649999 post=167.150499 pre=166.150499
floor=10 critical_ever=0.045562/day critical_sustained=0.050367/day
floor=16 critical_ever=0.074382/day critical_sustained=0.079184/day
floor=20 critical_ever=0.093589/day critical_sustained=0.098390/day
floor=50 critical_ever=0.237616/day critical_sustained=0.242416/day
cross untargeted floor=20 never
cross weakly floor=50 966.667d/232events
wall=0.01 s
sustain untargeted floor=16 950.000d/76events
sustain weakly floor=50 never
wall=0.01 s
```

**What each candidate means**

Ten is the warm-up table's first-useful-estimate milestone. It is reachable and sustainable in every stated tier, but declares sufficiency at the point the same table deliberately distinguishes from practically useful variance (`tab:warmup:milestones`); `packages/assayer/docs/spec/analysis-warmup.md:166-190`.

Sixteen is the analysis-derived candidate: it is the greatest whole-number floor below the untargeted tier's 16.170 pre-write steady-state limit. Seventeen would eventually pass immediately after an untargeted contribution but fall below the floor before the next one; its sustained critical rate is 0.08399 a day, above the stated 0.08. Sixteen therefore means “every declared targeting tier can eventually remain sufficient under regular arrivals.” It has no corpus warrant for estimator usefulness, and its untargeted path takes about nine hundred and fifty days to become sustained because it sits so close to the ceiling.

Twenty is both the specification's explicit convergence target and the warm-up table's practically-useful-variance milestone (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321` and (`tab:warmup:milestones`); `packages/assayer/docs/spec/analysis-warmup.md:166-190`. It means that an untargeted channel honestly remains insufficient under the shipped decay law, while weakly, moderately, and well-targeted channels can sustain sufficiency.

Fifty is the compatibility value carried through relocation. The caveat says the corpus cannot derive it, and neither the ten- nor twenty-observation anchor supports it (`cav:challenge:sufficiency-threshold-open`); `packages/assayer/adr/challenge.md:165-181`. Operationally it excludes the untargeted tier permanently, lets the weak tier first flash true only after about 967 days without ever staying true between regular contributions, and requires about 144 days moderately targeted or 75 days well targeted.

**Study recommendation, and the decision it awaits**

Choose **twenty**. It is the only candidate that names the corpus's intended quality level twice: the Companion convergence target and practically useful variance. The analysis does not rescue the bound's untargeted timeline; it shows that an untargeted stream cannot support that quality under the chosen decay. That is useful diagnostic truth, not a reason to lower the meaning of “sufficient.” A host that needs Companion use at that tier must target more effectively, inject defensible evidence, or supply an override.

Ten should remain the first-useful milestone rather than become sufficient. Sixteen is a defensible third candidate only if the priority is that every declared tier can eventually stay green; it is a rate-derived compatibility cut with no statistical-usefulness anchor and an untargeted wait of about two and three-fifths years. Fifty has neither an anchor nor broad reachability and should not survive solely because it shipped.

The immediate consequences remain narrow. Sufficiency is a reported boolean and never gates output or behaviour (`dec:health:reports-never-gates`); `packages/assayer/adr/health.md:123-136`. The tracker is not constructed by any runtime path (`cav:challenge:unwired`); `packages/assayer/adr/challenge.md:135-164`. An exact-name search finds one read of the relocated constant outside its declaration: a crate test that pins fifty and explicitly passes it to `health_report`, not a production caller (`packages/assayer/src/risk/challenge.rs:428-451` and `packages/assayer/src/tests/label_pipeline.rs:1102-1140`). The ruling is nevertheless owed before a host treats this diagnostic as an operational promise.

**Premise audit**

- Verified: fifty is the relocated shipped default, explicitly underivable and held open (`packages/assayer/src/risk/challenge.rs:428-443` and `packages/assayer/adr/challenge.md:165-181`).
- Verified: twenty is the specification target; ten and twenty are the first- useful and practically-useful warm-up anchors (`packages/assayer/docs/spec/companion-tracking.md:308-321` and `packages/assayer/docs/spec/analysis-warmup.md:166-190`).
- Verified: the source uses 0.9998 hourly retention, decay-before-write, pure decay-on-read, and an inclusive floor comparison (`packages/assayer/src/risk/challenge.rs:189-205` and `packages/assayer/src/risk/challenge.rs:234-269` and `packages/assayer/src/risk/challenge.rs:330-402`).
- Rounding qualification: forty-two days is the nearest-whole-day display for the exact moderate rate 0.48; $20/42$ does not recover that rate (`packages/assayer/docs/spec/companion-tracking.md:133-153`).
- Error: the decay-free convergence bound says untargeted evidence reaches effective sample size twenty, while the specified and shipped decay laws put its steady-state ceiling below twenty (`packages/assayer/docs/spec/companion-tracking.md:155-179` and `packages/assayer/docs/spec/companion-tracking.md:308-321`).
- Qualification: the bound's zero time under override is true for using the overridden estimate, not for the source's `sufficient_evidence` boolean (`packages/assayer/src/risk/challenge.rs:270-286` and `packages/assayer/src/risk/challenge.rs:547-570`).
- Qualification: no production path reads the relocated constant, but one crate test does; “nothing reads it” is exact only when “nothing” means runtime code (`packages/assayer/src/tests/label_pipeline.rs:1102-1140`).
