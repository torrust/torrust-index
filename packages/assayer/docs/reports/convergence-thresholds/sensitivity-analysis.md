# Convergence thresholds: sensitivity of the live ladder · `rep:health:identity-ladder-sensitivity`

This analysis asks how the stage reported by the shipped identity tracker moves when its live cuts move. It does not select replacement cuts. The ladder and its figures are fixed in the health record (`tab:health:identity-stability-cuts`); `packages/assayer/adr/health.md:138-185`, and the code applies them in the order Initial, Volatile, Stabilising, Maturing, Stable (`packages/assayer/src/health/identity_tracker.rs:138-172`).

**What the live quantities mean**

For two consecutive competitive sets, let $d$ be the number of entries plus exits and let $a$ be the intersection size. The tracker's raw churn is

$$c=\frac{d}{|S_0|+|S_1|}=\frac{d}{2a+d},$$

and its raw Jaccard overlap is

$$J=\frac{a}{|S_0\cup S_1|}=\frac{a}{a+d}.$$

The implementation computes exactly those quantities and applies the same one-tenth EWMA weight to both (`packages/assayer/src/health/identity_tracker.rs:97-136` and `packages/assayer/src/health/identity_tracker.rs:291-343`). For correctly reported set differences they are transforms of one another:

$$J=\frac{1-c}{1+c}, \qquad c=\frac{1-J}{1+J}.$$

Production generates entries and exits as the set differences themselves (`packages/assayer/src/identity/maintenance_loop.rs:472-483`) and passes those events to the tracker grouped by dimension (`packages/assayer/src/owner/lifecycle.rs:176-210`). The relation therefore applies to the shipped path, not only to an idealised input.

The initial EWMAs are $c_0=0$ and $J_0=1$. Because $(1-c)/(1+c)$ is convex, Jensen's inequality survives the common EWMA weights: the smoothed overlap is always at least the transform of the smoothed churn. This makes the two shipped Jaccard cuts redundant under the documented input contract:

- if smoothed churn is at most 0.05, smoothed Jaccard is at least $0.95/1.05=0.904762$, already above the stable Jaccard cut 0.9;
- if smoothed Jaccard is below 0.5, smoothed churn must exceed one third, already above the volatile churn cut 0.3.

Thus the reported stage is sensitive to the shipped stable churn ceiling and volatile churn floor, but not independently sensitive to the shipped Jaccard cuts. Jaccard would become decisive only if its cuts were tightened past the corresponding churn transforms, or if callers violated the set-difference input contract. The health record describes the Jaccard coordinate as confirmation; the calculation makes that confirmation exact at the shipped operating points (`dec:health:identity-stability-cuts`); `packages/assayer/adr/health.md:138-167`.

**Sensitivity of the stable churn ceiling**

The stable test is inclusive at the boundary: churn equal to 0.05 passes, while anything greater remains Stabilising. After one complete replacement of a previously settled set, the one-tenth EWMA moves from zero to 0.1. If later updates report no change, the smoothed churn after $n$ such updates is $0.1(0.9)^n$. The number of quiet updates needed to pass is:

| Stable churn ceiling | Quiet updates after the replacement |
| ---: | ---: |
| 0.04 | 9 |
| **0.05 shipped** | **7** |
| 0.06 | 5 |

A one-percentage-point move therefore changes this recovery example by two tracker updates. The stable Jaccard cut does not change the reported stage in the same example: overlap is already 0.9 after the replacement and rises with each quiet update, while churn remains the binding coordinate.

There is an operational qualification. Production calls the tracker only for a dimension whose competitive set emitted an entry or exit; an unchanged set causes the maintenance loop to continue without an event (`packages/assayer/src/identity/maintenance_loop.rs:472-483`). The tracker source itself asks for a future zero-change tick path (`packages/assayer/src/health/identity_tracker.rs:112-113`). The table therefore measures sensitivity per tracker update, as the tracker API and its tests define it, not per assessment or per hour in today's production path. A dimension that forms and then changes no further may never receive the quiet updates that let it reach the age gate. This cadence effect is larger than a small change to the 0.05 cut.

**Sensitivity of the volatile churn floor**

The volatile comparison is strict: churn equal to 0.3 remains Stabilising; greater churn reports Volatile. Starting from a settled EWMA, repeated complete replacements give smoothed churn $1-(0.9)^n$:

| Volatile churn floor | Complete replacements before Volatile |
| ---: | ---: |
| 0.25 | 3 |
| **0.30 shipped** | **4** |
| 0.35 | 5 |

A five-percentage-point move shifts the crossing by one maximal-change update. The shipped volatile Jaccard cut would cross only on the seventh consecutive complete replacement, after the churn gate has already reported Volatile on the fourth. Moving the Jaccard cut from 0.45 through 0.55 moves its own crossing from the eighth through the sixth update, but still does not change the stage while the 0.3 churn gate remains.

The substantive width is therefore the band between churn 0.05 and 0.3. Below it, metrics permit Maturing or Stable; inside it, the report is Stabilising; above it, Volatile. The sixfold separation makes modest noise around one cut unlikely to jump across the whole ladder, while each boundary remains locally discontinuous.

**Numeric cross-check**

A one-off Python 3 standard-library calculation checked the transforms, event crossings, and age tables below. Its relevant output was:

```text
stable churn 0.05 -> J=0.904762
volatile churn 0.30 -> J=0.538462
stable J     0.90 -> churn=0.052632
volatile J   0.50 -> churn=0.333333
stable ceilings 0.04, 0.05, 0.06 -> 9, 7, 5 quiet updates
volatile floors 0.25, 0.30, 0.35 -> 3, 4, 5 replacements
scenario-box extrema -> 0.715 days and 91.000 days
real 0.01
user 0.00
sys 0.00
```

No external package, network access, or compiler was used.

**Where the age gate is specified**

The specification does **not** state a two-hundred-and-sixty-four-hour identity gate. It states the general $2p$ rule and the deployment arithmetic (`bound:resource:convergence-budget`); `packages/assayer/docs/spec/analysis-resources.md:14-65`. The concrete gate is fixed by the health record (`tab:health:identity-stability-cuts`); `packages/assayer/adr/health.md:176-198`, and shipped as `MATURITY_HOURS_THRESHOLD = 264.0` (`const:assayer:dimension-maturity-age-scalar-264p0`); `packages/assayer/src/health/identity_tracker.rs:345-362`. It is therefore an implementation/ADR proxy derived from a specification budget, not a threshold the specification itself binds.

At the reference deployment the derivation is:

$$2p=2\times638=1{,}276\text{ eligible labels},$$

$$R_\text{elig}=200\text{ labels/day}\times0.60=120\text{ eligible labels/day},$$

$$T=1{,}276/120=10.6333\text{ days}=255.2\text{ hours}.$$

The shipped 264 hours are eleven days, a margin of 8.8 hours over that arrival. The source test recomputes these inputs, requires the gate to be at least the eligible-label arrival, requires less than twenty-four hours of excess, and separately refuses the raw-rate division (`packages/assayer/src/health/identity_tracker.rs:509-549`).

The one-hundred-and-twenty-per-day provenance is explicit rather than inferred from traffic folklore. The resource table states two hundred labels a day at sixty per cent eligibility and then says its day figures use the resulting one hundred and twenty eligible labels a day (`bound:resource:convergence-budget`); `packages/assayer/docs/spec/analysis-resources.md:46-62`. The warm-up profile independently names the same medium rate and states that all profiles assume sixty per cent eligibility (`ex:warmup:profiles`); `packages/assayer/docs/spec/analysis-warmup.md:129-141`.

**Age sensitivity to parameter count**

Holding two hundred labels a day and sixty per cent eligibility, the four model sizes already evaluated by the specification give:

| Deployment | $p$ | Derived days | Whole-day ceiling | Does 264 h cover it? |
| --- | ---: | ---: | ---: | --- |
| Minimal | 286 | 4.767 | 5 | Yes |
| Standard | 638 | 10.633 | 11 | Yes |
| Standard+ | 682 | 11.367 | 12 | No |
| Large | 910 | 15.167 | 16 | No |

These are not invented bounds: the parameter counts are the specification's four declared deployments (`bound:resource:convergence-budget`); `packages/assayer/docs/spec/analysis-resources.md:52-61`. The result also finds a premise error in the commissioning caveat. It says eleven days covers Standard+ (`packages/assayer/adr/health.md:208-212`), but the specification's own 11.4-day row and the unrounded 11.367-day calculation exceed eleven days.

**Age sensitivity to eligibility**

The corpus supplies one reference eligibility fraction, sixty per cent; it does not supply an empirical range. To expose sensitivity without pretending to have such evidence, this table uses an explicitly analytic bracket of forty to eighty per cent around the reference, holding $p=638$ and two hundred raw labels a day:

| Eligibility fraction | Eligible labels/day | Derived days | Whole-day ceiling |
| ---: | ---: | ---: | ---: |
| 0.40 | 80 | 15.950 | 16 |
| **0.60 reference** | **120** | **10.633** | **11** |
| 0.80 | 160 | 7.975 | 8 |

The endpoints are scenario values, not measured deployment claims. Their role is to show the inverse dependence: a one-third decrease from the reference fraction increases the gate by half, while a one-third increase reduces it by a quarter. Any decision that needs an empirical eligibility range still needs deployment data.

**Age sensitivity to label arrival**

Holding $p=638$ and sixty per cent eligibility, the specification's three traffic profiles supply the rate range (`ex:warmup:profiles`); `packages/assayer/docs/spec/analysis-warmup.md:129-141`:

| Raw labels/day | Eligible labels/day | Derived days | Whole-day ceiling |
| ---: | ---: | ---: | ---: |
| 50 | 30 | 42.533 | 43 |
| **200 reference** | **120** | **10.633** | **11** |
| 1,000 | 600 | 2.127 | 3 |

Arrival rate is the dominant stated sensitivity. Across the declared parameter counts, the declared traffic rates, and the analytic eligibility bracket, the corner calculations span 0.715 days ($p=286$, eighty per cent, one thousand a day) to 91 days ($p=910$, forty per cent, fifty a day). The fixed eleven-day gate is therefore a reference-deployment proxy, not a deployment-invariant arrival bound.

The age cut has a direct but narrow effect on the stage function. Once metrics pass, ages below the cut report Maturing and ages at or above it report Stable; the comparison is inclusive at the cut (`packages/assayer/src/health/identity_tracker.rs:160-171`). Because the composite requires every identity tracker to be Stable, changing the cut can also hold the composite at InteractionMaturing or release it to SteadyState (`packages/assayer/src/health/composite.rs:202-215`). It changes no model or output behaviour because convergence is diagnostic only (`dec:health:reports-never-gates`).

**Age-gate verdict — the direct answer.** The $2p$ budget and its reference inputs are specified, but the 264-hour gate itself is not: it is an ADR/code proxy. At exactly $p=638$, sixty per cent eligibility, and two hundred labels a day, rounding 255.2 hours to the next whole day is the tightest stated rounding and the source test pins that interpretation. Rounding to the next whole week would produce fourteen days and add about 80.8 hours; no cited source supports that quantum. Across the specification's parameter and traffic ranges, however, and even a moderate analytic eligibility bracket, the derived gate spans about one day to about three months. The specification itself calls $2p$ the whole of a rule of thumb. The evidence therefore supports **next-whole-day rounding for the declared reference calculation, rejects next-whole-week as an underived pad, and treats 264 hours as only an order-of-magnitude figure when it is applied to deployments whose inputs are not the reference inputs.**
