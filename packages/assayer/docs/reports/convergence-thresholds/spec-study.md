# Convergence thresholds: what the tree specifies · `rep:health:threshold-authority-study`

This study is evidence for a decision, not the decision. It reads the specification, the diagnostic ladders, and the construction surface together. The caveat that commissioned it says that ten threshold fields are validated and never read and that wiring stopped because no specified consumers exist (`cav:health:dead-convergence-thresholds`); `packages/assayer/adr/health.md:221-274`. That central finding survives inspection. One supporting count does not, and is corrected below.

**Convergence in the resource specification**

The resource chapter uses *convergence* as a budget, not as a diagnostic enum. Its only statistical rule is that a Bayesian linear model with $p$ parameters needs on the order of $2p$ observations before data shape the posterior more than the prior. The chapter calls that “the whole of the rule of thumb.” It then defines two parallel timelines, with full-system time the maximum of Core time and Companion time. Core time is itself the sum of spatial, warm-up, baseline, and Assayer terms; the Assayer term alone is $2p/(R_\text{label}f_\text{elig})$. The Companion term is $20/R_\text{contributing}$ (`bound:resource:convergence-budget`); `packages/assayer/docs/spec/analysis-resources.md:14-49`.

Those names are budget components:

| Surface | Names | Quantity |
| --- | --- | --- |
| Full system | Core, Companion | The later of two elapsed times |
| Core sequence | Spatial, warm-up, baseline, Assayer | A sum of four elapsed times |
| Assayer model | No stage name | $2p$ eligible labels divided by eligible-label arrival rate |
| Companion | No Core stage name | Target effective sample size divided by contributing-label rate |

At the reference raw rate of two hundred labels a day and sixty per cent eligibility, the eligible arrival rate is one hundred and twenty a day. The chapter evaluates four declared model sizes: $p=286,638,682,910$, requiring 572, 1,276, 1,364, and 1,820 eligible labels respectively. Their displayed times are about 4.8, 10.6, 11.4, and 15.2 days (`bound:resource:convergence-budget`); `packages/assayer/docs/spec/analysis-resources.md:46-65`. The reference configuration derives $p=638$ block by block rather than estimating it (`tab:resource:reference-configuration`); `packages/assayer/docs/spec/analysis-resources.md:166-197`.

The independent Companion budget is specified elsewhere as target effective sample size over contributing-label rate, evaluated at twenty target observations (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321`. Neither budget introduces Warming, Converging, or Converged as a named stage.

**The specification's six-stage reading aid**

The warm-up chapter names exactly six stages: **Cold start**, **Anchor emergence**, **Pre-calibration**, **Sister convergence**, **Interaction maturity**, and **Steady state**. Their extents are approximately the first thirty labels, thirty to eighty, eighty to two hundred, two hundred to $2p$, $2p$ to $10p$, and beyond. The table says what has converged and what a host can expect at each point (`tab:warmup:stages`); `packages/assayer/docs/spec/analysis-warmup.md:32-41`.

The chapter immediately limits the table's authority: these are approximate descriptions of a continuous process, not transitions the implementation performs. Sister wall time depends on eligible-label rate; identity structure, competitive-set stabilisation, per-cell weights, and range interactions converge in parallel on separate timescales; the Companion also runs in parallel (`tab:warmup:stages`); `packages/assayer/docs/spec/analysis-warmup.md:43-68`. The later milestone table likewise calls itself a reading aid whose figures are fixed elsewhere, not a new source of thresholds (`tab:warmup:milestones`); `packages/assayer/docs/spec/analysis-warmup.md:166-196`.

The implementation's composite uses the same six-place vocabulary in Rust form: `ColdStart`, `AnchorEmerging`, `PreCalibration`, `SisterConverging`, `InteractionMaturing`, and `SteadyState` (`packages/assayer/src/health/composite.rs:32-65`). Its actual transitions are not the table's displayed extents: it leaves cold start after one total label, enters pre-calibration after thirty total labels and a Platt fit, then uses two refits, the two live Platt thresholds, and finally universal identity stability (`packages/assayer/src/health/composite.rs:102-131` and `packages/assayer/src/health/composite.rs:137-215`). The source itself records the total-versus-eligible-label and boundary gaps. That implementation drift is separate from the dead eight identity-stage figures.

**The identity tracker's five-stage ladder**

Each identity dimension instead reports **Initial**, **Volatile**, **Stabilising**, **Maturing**, or **Stable**. Initial means no entry or exit has ever been recorded. Volatile means smoothed Jaccard overlap is below one half or smoothed churn is above three tenths. Stabilising is the band before overlap reaches nine tenths and churn falls to one twentieth. Maturing means those set metrics pass but the age proxy has not reached two hundred and sixty-four hours. Stable means both the set metrics and age pass (`packages/assayer/src/health/identity_tracker.rs:138-172` and `packages/assayer/src/health/identity_tracker.rs:192-233`). The cuts are fixed in the health record, not in construction configuration (`tab:health:identity-stability-cuts`); `packages/assayer/adr/health.md:138-185`.

The state available to that decision is narrow: two EWMAs, current cell count, the previous cell identifiers, lifetime entry and exit totals, and first-seen time (`packages/assayer/src/health/identity_tracker.rs:41-75`). A change event computes churn from entries plus exits divided by current plus previous set size, computes Jaccard from the two sets, and smooths both at weight one tenth (`packages/assayer/src/health/identity_tracker.rs:97-136`). The five-stage ladder contains none of the three names used by the dead configuration fields.

**The twelve-field construction surface**

`ConvergenceThresholdConfig` has twelve fields (`packages/assayer/src/config/types.rs:414-496`):

| Group | Field | Default | Shipped use after construction |
| --- | --- | ---: | --- |
| Platt | `platt_converged_delta_cal` | 0.01 | Live |
| Platt | `platt_converged_refit_count` | 5 | Live |
| Warming | `warming_min_cells` | 10 | None |
| Warming | `warming_min_obs` | 50 | None |
| Converging | `converging_min_cells` | 100 | None |
| Converging | `converging_min_obs` | 500 | None |
| Converging | `converging_min_importance` | 10.0 | None |
| Converged | `converged_min_cells` | 500 | None |
| Converged | `converged_min_obs` | 2,000 | None |
| Converged | `converged_min_importance` | 100.0 | None |
| Companion | `challenge_sufficient_evidence` | 50 | None |
| Companion | `challenge_min_samples` | 10 | None |

Construction requires positive Platt values, non-zero Warming values, positive Converging importance, increasing cell and observation thresholds, increasing Converging-to-Converged importance, and non-zero Companion values. A failure rejects the build (`packages/assayer/src/api/builder.rs:409-478`). Validation therefore supplies plausibility, but not a consumer.

The two Platt fields are demonstrably live. The full convergence object is copied into the label-pipeline configuration (`packages/assayer/src/owner/label_path.rs:213-242`); those two fields are read around refits (`packages/assayer/src/owner/label_path.rs:806-842`) and are passed to the composite stage computation (`packages/assayer/src/owner/label_path.rs:1231-1244`). Repository-wide name search finds no corresponding production read of the other ten; outside their definition and defaults they occur in construction validation, tests of that validation, and, for one Companion metric name, the metrics catalogue. This confirms the caveat's ten-dead/two-live premise (`cav:health:dead-convergence-thresholds`).

The specification's monitoring-configuration table is also negative evidence. It lists seventeen parameters over drift, discrimination, numerical precision, label integrity, and cross-layer observability, but none of these twelve convergence fields (`tab:config:monitoring`); `packages/assayer/docs/spec/reference-configuration.md:327-350`. An exact-case search of `packages/assayer/docs/spec/` finds no occurrence of `Warming`, `Converging`, or `Converged`. The specification therefore supplies neither the names nor a mapping for the three-stage vocabulary.

**The mismatch, and a corrected premise**

The wiring block is real and has two independent parts.

First, there is no vocabulary map. The configuration's three names do not occur as stages in the specification; they map neither to the tracker's five names nor to the composite's six. Choosing a map would decide semantics rather than implement them (`cav:health:dead-convergence-thresholds`); `packages/assayer/adr/health.md:242-258`.

Second, the eight identity-stage figures are not a coherent input bundle for the tracker. The three cell thresholds have a clear tracker-local candidate, `current_cell_count` (`packages/assayer/src/health/identity_tracker.rs:50-75`). The three observation thresholds have no cumulative per-dimension observation counter there. Production drains a per-pass local count and feeds each graph, but does not retain or hand that count to the convergence tracker (`packages/assayer/src/identity/maintenance_loop.rs:457-469`).

The caveat's stronger statement that no per-dimension graph importance is aggregated anywhere is false in this tree. Each dimension has a graph snapshot whose `total_energy()` returns whole-graph energy (`packages/assayer/src/identity/snapshot.rs:68-92`); the assessment interface explicitly exposes per-dimension graph total importance and reads it for every assessment (`packages/assayer/src/assessment.rs:795-807` and `packages/assayer/src/assessment.rs:1018-1039`). The live implementation reads it from the dimension's graph snapshot (`packages/assayer/src/lib.rs:1106-1130`). It is not handed to `IdentityConvergenceTracker`, its publication is checkpoint-coupled, and the configuration does not say whether thresholds 10.0 and 100.0 apply to raw energy or the `ln(1+x)` feature. Connecting it still needs design, but creating an aggregate from nothing does not.

Consequently, “six of eight lack a counterpart quantity” is not a count the current tree supports. On a strict tracker-local reading, the three cell fields have a counterpart and the three observation plus two importance fields do not: five of eight lack an input *on the tracker*. On a package-wide reading, both importance fields also have an existing candidate, leaving only the three observation fields without a cumulative counterpart. This premise error does not unblock wiring: vocabulary, observation semantics, importance scale, publication freshness, and ownership remain unspecified. It does reduce the honest size of the missing-input implementation.

**What the specification does not decide**

No cited source decides any of the following:

- whether the three configuration names describe the identity ladder, the composite ladder, or another summary;
- whether a threshold marks entry into a stage or completion of it;
- what one per-dimension “observation” is, or whether it is cumulative, decayed, eligible-only, or reset by lifecycle change;
- whether graph importance means raw total energy, its logarithmic feature, or another aggregate, and how fresh the value must be;
- how cells, observations, importance, Jaccard, churn, and age combine when they disagree;
- whether either Companion figure belongs to Core construction.

The specification does decide that convergence health reports and never gates behaviour (`dec:health:reports-never-gates`); `packages/assayer/adr/health.md:123-136`. That limits the consequence of a bad mapping to a false diagnostic, but does not make inventing one a repair.
