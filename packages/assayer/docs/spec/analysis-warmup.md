## Part (Properties and Analysis) · `part:spec:properties-and-analysis`

Parts I through VI specified what the system is and how it runs. Part VII asks what follows from it. How long a deployment waits before its output means anything, what the running system costs, what the composed design detects that no single layer of it detects alone, and how an operator sees any of it going wrong — these are the four chapters, and their common subject is consequence rather than construction.

Most of what follows computes from parameters fixed in earlier Parts rather than fixing anything of its own, and the Part is written to keep that distinction visible: an environment that binds says so, and an environment that merely restates a threshold cites the chapter that fixed it.

### Chapter (System Initialisation and Warm-Up) · `chap:spec:initialisation-and-warmup`

A deployment does not begin useful. The chapter says how it becomes so: the stages it passes through, the two quantities that must bootstrap themselves before anything downstream is trustworthy, what convergence looks like at three traffic profiles, and how the layers recover after the ground moves under them.

Two of these bind and the rest illustrate. The bootstraps are procedures the implementation must perform. The stages, the milestones, the profiles and the recovery timelines compute consequences of thresholds fixed in the calibration, Companion and feature chapters, and a restatement that cites its sources binds nothing of its own.

**Table (The six warm-up stages)** · `tab:warmup:stages`

| Stage | Extent | What has converged | What the host can expect |
| --- | --- | --- | --- |
| Cold start | First ~30 labels | Nothing; all means at $\mathbf{0}$ | $\hat{p} \approx 0.5$ everywhere, maximal uncertainty, no discrimination |
| Anchor emergence | ~30–80 labels | The anchor's fifteen parameters | First directional estimates; uncertainty still wide |
| Pre-calibration | ~80–200 labels | Direction, not magnitude | Scores ordered correctly, probabilities unreliable |
| Sister convergence | ~200–$2p$ labels | The sister's per-Sentinel features | Sentinel-specific discrimination; calibration fitted at least once |
| Interaction maturity | ~$2p$–$10p$ labels | Interaction features, in cascade order | Joint and cross-Sentinel structure, arriving unevenly |
| Steady state | Beyond | All parameters; a dynamic equilibrium | Calibrated assessments with characterised uncertainty |

The stages are approximate descriptions of a continuous process, not transitions the implementation performs, and they are given as one table rather than six identities for that reason. The anchor converges first because it is smallest (`def:risk:anchor-model`); the blend weight is high while it dominates and falls towards a small residual as the sister converges (`def:risk:subspace-blend`). Sister convergence is measured in eligible labels, so its wall-clock extent depends on the eligibility rate (`tab:eligibility:training`), and the interaction features arrive in the order the convergence cascade fixes (`tab:feature:interaction-convergence`). The importance trackers are unstable through the first two stages for the ordinary reason that they are estimated from few samples (`tab:weighting:trackers`). Forgetting never stops, so the last stage is an equilibrium rather than an end state.

The identity layer converges in parallel, on its own timescales:

| What converges | Timescale |
| --- | --- |
| Identity graph spatial structure | Minutes to hours |
| Competitive set stabilisation | Hours to days |
| Per-cell indicator weight convergence | Days to weeks |
| Per-range by alarm conjunctions | Weeks |

The Companion converges in parallel too, and on a timeline determined entirely by the rate at which contributing challenge labels arrive (`bound:companion:convergence`).

**Algorithm (The concordance threshold bootstrap)** · `alg:warmup:concordance-bootstrap`

The aggregate feature concordance thresholds require a rolling window of sub-scores before they can be calibrated at all (`alg:feature:concordance-recalibration`), and the window has to be filled before it can be read. Three phases:

Before the first calibration — the first thousand assessments — the initial value $\theta_a = 0.5$ is used unchanged.

At the thousandth assessment the accumulated sub-scores are read and their eightieth percentile replaces the initial value.

Every thousand assessments thereafter the thresholds are recalibrated from a rolling window of the ten thousand most recent sub-scores, one entry per reporting Sentinel.

The thresholds are held as separate atomic state outside the model snapshot. Changing them therefore triggers neither a calibration refit nor a drift accumulator reset, which is what keeps a threshold move from being read downstream as a change in the model's behaviour (`sec:feature:aggregate`).

The initial value is one half, the cadence is one thousand assessments, the percentile is the eightieth, and the rolling population is the ten thousand most recent sub-scores with one entry per reporting Sentinel.

**Algorithm (The standardisation bootstrap)** · `alg:warmup:standardisation-bootstrap`

Standardisation begins from feature-class priors and must replace them with measurements, in two situations that differ in what is new.

The system's own cold start is covered by the prior-mass ramp. Each accepted raw assessment vector retires an equal share of the priors' standardisation mass: the first advances the phase to `Transitioning`, and the accepted observation at $N_\text{init} = 100$ retires the last share and enters `InService` (`alg:standardisation:batch-initialisation`). What this corrects is the gross mismatch between the priors and the deployment's actual feature distributions, and it corrects it over the whole horizon rather than at one step. The phase and the accepted count are carried on every assessment throughout (`schema:output:health-snapshot`), so a host is never left reading the coordinate state off the assessment count: a refused observation and an owner still working through its backlog both leave the ramp behind the requests served, and neither is a fault.

The per-Sentinel bootstrap covers a Sentinel that registers later, into a system whose statistics have long since settled. Its features are standardised against the class priors until $N_\text{boot}$ assessments have accumulated — default one hundred — at which point bootstrapped statistics for that Sentinel are blended in (`alg:standardisation:sentinel-bootstrap`). Without it a late Sentinel would be standardised against a coordinate system that describes everything except itself (`cav:limitation:late-standardisation`).

The per-Sentinel bootstrap opens an accumulator at registration, fills it from assessments that Sentinel reports in, and blends it into the slot at the specified weight and sample count. The cold ramp starts from the specified priors (`tab:standardisation:class-priors`), retires one equal share of their mass per accepted raw vector, and publishes every advance. Its phase and accepted count travel on the compact and full health surfaces, so each result identifies its coordinate state.

**Example (Convergence at three deployment profiles)** · `ex:warmup:profiles`

Every Core figure below is the convergence budget evaluated at the profile's own rates (`bound:resource:convergence-budget`), and every Companion figure is its own bound evaluated at the profile's contributing-label rate (`bound:companion:convergence`). All three profiles assume sixty per cent eligibility and the reference dimension (`tab:resource:reference-configuration`).

| Profile | Core models | Companion, untargeted | Companion, well targeted | With override |
| --- | --- | --- | --- | --- |
| Low traffic — 10 req/s, 50 labels/day | ~42.5 days | Never | 135 days | **~42.5 days** |
| Medium traffic — 100 req/s, 200 labels/day | ~10.6 days | Never | 28 days | **~10.6 days** |
| High traffic — 1,000 req/s, 1,000 labels/day | ~2.1 days | 58 days | ~5 days | **~2.1 days** |

The Companion columns are the bound's decayed times rather than its arrival times, which is why two of them are never: at fifty and at two hundred labels a day the untargeted contributing rate puts the effective sample size ceiling below twenty, so no amount of waiting reaches it (`bound:companion:convergence`).

The Companion's own timeline varies by an order of magnitude with targeting quality alone, because the quantity that matters is not the challenge rate but the rate at which challenges produce contributing labels (`data:companion:contributing-rate`):

| Targeting quality | Contributing labels/day | Time to a usable estimate |
| --- | --- | --- |
| Untargeted, at the population rate | 0.08 | Never |
| Weakly targeted | 0.24 | 108 days |
| Moderately targeted | 0.48 | 48 days |
| Well targeted | 0.80 | 28 days |
| Host override supplied | — | 0 days |

The reading is the same at every profile: the Companion binds, unless the host supplies an override — and at untargeted rates it does not merely bind but never arrives at a usable estimate at all (`alg:companion:override`). A host that supplies one waits for the Core alone. A host that can supply neither an override nor targeted challenges will see wide crossover intervals for months (`def:landscape:credible-intervals`), and should consider the three-action space, which is markedly more robust to challenge effectiveness uncertainty (`prop:channel:challenge-width`). Hosts reading risk assessments without the derivation are unaffected by the Companion's timeline entirely.

**Table (The milestones)** · `tab:warmup:milestones`

| Milestone | Requirement |
| --- | --- |
| First risk assessment | 1 assessment, at $\hat{p} \approx 0.5$ and maximal uncertainty, in the feature-class-prior coordinate system — phase `WaitingForInit` and accepted count zero in the snapshot it acquired |
| Standardisation empirical | $N_\text{init}$ accepted cold-ramp observations; about 100 assessments absent skips or owner backlog |
| Anchor emergence | ~30 eligible labels |
| First calibration refit | ~200 labels |
| Aggregate-by-context interaction maturity | ~500 labels |
| Competitive set stabilisation | ~1–3 days at high traffic |
| Sister model convergence | ~$2p$ eligible labels |
| Per-Sentinel-by-context interaction maturity | ~2,000 labels |
| Competitive-range interaction maturity | ~2,000–5,000 labels |
| Cross-Sentinel interaction maturity | ~5,000+ labels |
| **First calibrated assessment** | max(200 labels, the Core's budget) |
| Companion: first useful estimate | ~10 contributing labels |
| Companion: practically useful variance | ~20 contributing labels |
| Companion: host override supplied | Immediate |
| **First calibrated landscape** | max(Core calibrated, Companion usable or overridden) |

The table is a reading aid and not a source. Every threshold in it is fixed somewhere else and cited from there: the refit cadence at the calibration chapter (`tab:platt:refit-cadence`), the interaction maturities at the convergence cascade (`tab:feature:interaction-convergence`), the Companion's two counts at its convergence bound (`bound:companion:convergence`), the sister's budget at the resource chapter (`bound:resource:convergence-budget`), and the standardisation count at batch initialisation (`alg:standardisation:batch-initialisation`). A restatement that cites its sources constrains nothing on its own account, and this one is offered as the single view a reader planning a deployment wants rather than as a further requirement on the implementation.

The standardisation row is the one whose figure is an approximation of a different quantity. What fixes that milestone is the accepted-observation target and the phase transition at it; the assessment count beside it is an operational approximation only, because contention and queue pressure may refuse observations and a deployment may therefore serve more than a hundred requests before the ramp reaches its horizon.

**Remark (What to trust before the first refit)** · `rem:warmup:pre-calibration`

Between the anchor's emergence and the first calibration refit the system produces scores that are ordered correctly and scaled wrongly. The calibration parameter still holds its initial value of one (`def:platt:initial-value`), which is a placeholder rather than a fit, so probabilities carry the model's ranking but not its magnitudes (`cav:limitation:pre-calibration`).

What this costs a host depends on what the host reads. A ranking, a queue order or a relative comparison survives the interval intact. An absolute probability does not, and neither does anything computed from one as though it were calibrated. Hosts using the derivation see approximate crossover locations throughout: interval widths move in the correct direction regardless of the calibration parameter's absolute value, while locations shift with the shared term and settle at the first refit (`thm:landscape:rigid-translation`).

The interval is visible rather than silent. The calibration record counts on the risk basis report how much evidence each regime's fit rests on (`schema:risk:basis`), and a count below the minimum is the explicit signal that the system is operating before calibration (`req:platt:minimum-samples`); the effects of crossing that boundary are analysed with the regimes themselves (`disc:platt:regime-transition`). A host that needs calibrated probabilities from its first day has one remedy, which is to arrive with history: pre-seeding with at least two hundred historical labels moves the deployment past this stage before it serves live traffic (`alg:host:pre-seeding`).

This section is deployment awareness rather than a constraint on the implementation. It tells an operator what not to trust and for how long, and it fixes no threshold that the code could satisfy or violate.

**Example (Recovery after atrophy, a transient event and a policy change)** · `ex:warmup:recovery`

Recovery is not one timescale. After a disturbance the measurement layer returns in hours and the memory layers return over weeks, because they are built to remember, and an operator watching only the fast layer will believe the system has recovered while its slower features still carry the disturbance.

Restriction-induced atrophy, where the host's own restriction removes the traffic a region was known by:

| Time | Core observable | Risk effect |
| --- | --- | --- |
| 0 | Scores in affected cells may normalise | Measurement-derived features fall |
| $N_\text{absent}$ cycles | Affected entries deleted (`alg:ledger:entry-deletion`) | Routing falls back to the ancestor entry |
| 14 days | Identity cell outcome features halved | Moderate identity-derived residual |
| 29 days | Ledger ancestor features reflect recent labels only | Ledger contribution reflects current evidence |

A transient adverse event that ends of its own accord:

| Time after the event ends | Core observable | Risk effect |
| --- | --- | --- |
| Hours | Measurement scores normalise | Measurement features return to baseline |
| 14 days | Identity cell averages halved | Moderate residual through the identity pathway |
| 29 days | Ledger averages halved | Modest residual |
| 87 days | Ledger averages at about an eighth | Negligible |

A host-initiated temporal policy change, where nothing is wrong and the ground has moved anyway: at the next batch report the reported cell set may change and is reconciled by cell-set maintenance (`alg:runtime:cell-set-maintenance`); over hours to days the set stabilises and new entries begin accumulating outcome history; over days to weeks time-indexed decay dissolves the pre-change Ledger state and post-change labels establish new averages.

The two decay horizons behind every figure above are the identity layer's fourteen days (`tab:keyspace:decay-rates`) and the Ledger's twenty-nine (`def:ledger:time-decay`). Neither is a recovery mechanism that acts; both are bounds on how long a stale impression can persist, and the timelines are arithmetic on them rather than claims of their own.
