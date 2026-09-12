# Definitive specification changes for the uncertainty step · `rep:spec:standardisation-ramp-deltas`

This report proposes the specification changes the uncertainty-step ruling would carry. It changes no specification, record, implementation or test. As a prose carrier it cites existing heads and mints none, under the label calculus. The proposal is grounded in [The cold-start uncertainty step](cold-start-uncertainty-step.md) and [Two clocks at the Sentinel boundary](sentinel-two-clocks.md), with every relied-on premise checked again against the current tree.

## Design position for ratification · `sec:spec:ramp-design-position`

Ratify an **observation-authorised, finite prior-mass ramp**. Cold construction must publish the tabulated feature-class priors. The first $N_\text{init} = 100$ accepted raw assessment vectors must then replace those priors gradually, one hundredth of the prior population at a time, rather than remaining invisible until one wholesale replacement. Each request must score against the immutable snapshot it acquired before its observation advances the ramp. Every accepted advance must publish a later snapshot and report the standardisation phase and accepted-observation count on both the compact assessment health and the full health surface. The existing snapshot version is the coordinate version; a second version counter would duplicate authority.

The phase has three values without renaming the two public values that already ship: `WaitingForInit` at count zero, `Transitioning` from the first accepted observation through the one before the target, and `InService` at the target. Maturity is exactly the reported count divided by $N_\text{init}$ and is not a separate field that could disagree. The continuing standardisation EWMA remains label-authorised and begins only in `InService`. The per-Sentinel bootstrap remains observation-authorised and otherwise unchanged: it is lifecycle-scoped, already retains one fifth of the prior, and already has a named limitation. This ruling removes the cold full-vector cliff; it does not silently broaden into a redesign of the late-Sentinel mechanism.

For accepted observation $n$, let $a_n = n/N_\text{init}$, let $(\mu_0, v_0)$ be the ramp's base moments, and let $(\mu_{e,n}, v_{e,n})$ be the Welford moments of the accepted raw observations. Every non-bias position publishes

```text
mu_n = (1 - a_n) * mu_0 + a_n * mu_e,n
v_n  = max(v_floor,
           (1 - a_n) * v_0 + a_n * v_e,n
           + a_n * (1 - a_n) * (mu_e,n - mu_0)^2)
```

The last term is the between-population variance. Omitting it would make the variance neither the prior's nor the observed mixture's. At the target the prior weight is exactly zero and the published moments are exactly empirical; no asymptotic tolerance or new rate is introduced. A lifecycle reset takes the currently published moments as the new base and never rolls the coordinate system back to priors. The cold ramp state is checkpointed so a restore resumes the same count and mixture rather than declaring a partial transition complete.

This chooses the direction on the table over the earlier study's label-gated candidate. Standardisation describes input geometry, and selective or absent labels are the wrong authority for the assessed population. The Sentinel precedent is score-before-evolve, gradual online movement and disclosed maturity, not universal immobility. The purity promise must therefore be stated as three separate truths: derivation of one held assessment is bit-identical; assessment does not advance posteriors, calibration or other outcome-learned state; and an accepted cold observation may move only a later, explicitly phased and versioned coordinate snapshot. “Repeated assessment leaves no residue” is not retained.

The recommended constant is the existing $N_\text{init}=100$; no new tuning surface is warranted. The recommendation is falsified if the end-to-end held- prior fixture produces any single accepted-observation uncertainty move larger than one tenth of its complete prior-to-empirical endpoint movement. The standard-library audit below finds the largest move at the first observation, about 1.8 per cent of the endpoint movement, so the chosen horizon clears that bar with margin. A failure of that criterion requires dual-coordinate continuity, not a different undocumented ramp constant.

Study basis: the mechanism and asynchronous count gate are verified at `packages/assayer/docs/reports/cold-start-uncertainty-step.md:10-103`; the authority split and gradual, visible recommendation are at `packages/assayer/docs/reports/sentinel-two-clocks.md:11-32` and `packages/assayer/docs/reports/sentinel-two-clocks.md:425-440`.

## Specification delta census · `sec:spec:ramp-delta-census`

The ruling requires seventeen specification deltas: fifteen replacements and two new passages whose heads the ADR lane must name and mint. Sixteen further passages are in the ruling's reach and remain truthful. The detailed entries below quote the current text, give replacement text in the specification's voice, and state the study evidence for each result.

| Delta | Passage | Disposition |
| --- | --- | --- |
| D1 | `packages/assayer/docs/spec/data-model-features.md:545-565` | Replace completion-only timing with the observation ramp and score-before-evolve order. |
| D2 | `packages/assayer/docs/spec/data-model-features.md:567-589` | Gate the continuing EWMA on `InService`. |
| D3 | `packages/assayer/docs/spec/data-model-features.md:618-622` | Correct the class count and replace the label-half-life claim with the cold ramp horizon. |
| D4 | `packages/assayer/docs/spec/data-model-features.md:641-672` | Replace wholesale batch publication with the finite prior-mass algorithm. |
| D5 | `packages/assayer/docs/spec/data-model-features.md:674-691` | Separate transition mismatch from the existing label-time mismatch bound. |
| D6 | `packages/assayer/docs/spec/data-model-features.md:693-706` | Define lifecycle reset against current moments without rollback. |
| D7 | `packages/assayer/docs/spec/analysis-warmup.md:100-127` | Describe the ramp and remove two stale shipped-status claims. |
| D8 | `packages/assayer/docs/spec/analysis-warmup.md:173-198` | Make the first-assessment and empirical milestones phase-aware. |
| D9 | `packages/assayer/docs/spec/operations-assessment.md:55-88` | Split raw assembly, ramp offer and snapshot standardisation in pipeline step five. |
| D10 | `packages/assayer/docs/spec/operations-assessment.md:91-117` | Preserve the three direct writes while naming the observational enqueue and label-state prohibition. |
| D11 | `packages/assayer/docs/spec/operations-assessment.md:307-337` | Add phase and accepted-observation count to compact health. |
| D12 | New health-monitoring passage after `packages/assayer/docs/spec/analysis-monitoring.md:17-27` | Define the compact/full standardisation-transition surface. |
| D13 | `packages/assayer/docs/spec/reference-configuration.md:208-228` | Keep the value one hundred, change `N_init` to ramp-horizon semantics and remove a stale documentation-defect claim. |
| D14 | `packages/assayer/docs/spec/operations-concurrency.md:129-145` | Give standardisation a transition-time observation cadence and an in-service label cadence. |
| D15 | `packages/assayer/docs/spec/operations-temporal.md:36-56` | Record the finite observation-indexed ramp beside the continuing label-indexed EWMA. |
| D16 | `packages/assayer/docs/spec/operations-runtime.md:140-146` | Qualify pre-seeding: it advances standardisation only in service. |
| D17 | New guarantee passage after `packages/assayer/docs/spec/reference-guarantees.md:35-45` | State the assessment evidence-authority and versioned-drift promises beside derivation purity. |

### Standardisation chapter · `sec:spec:ramp-standardisation-chapter`

#### D1 — where statistics live and what feeds them · `sec:spec:ramp-statistic-custody`

Current, `packages/assayer/docs/spec/data-model-features.md:545-565`:

> “1. **Batch initialisation** (`alg:standardisation:batch-initialisation`) accumulates raw assessment-time vectors and applies empirical statistics when complete.”
>
> “The first two observe on the assessment path, and that observation is an auxiliary-accumulator write (`inv:runtime:enumerated-writes`), not a model write. The tracking average is indexed by labels and carries no time-indexed decay: a deployment that stops receiving labels stops moving its coordinate system, which is the correct behaviour for a coordinate system.”

Proposed replacement:

> The standardisation statistics are part of the published snapshot. An assessment acquires one immutable coordinate system, scores against it, and mutates no statistic in that snapshot. Three mechanisms feed later snapshots, and all three publish through the model owner:
>
> 1. **Cold-start prior-mass ramp.** Accepted raw assessment-time vectors replace the cold base moments in equal increments through the batch-initialisation algorithm.
> 2. **Per-Sentinel bootstrap.** Raw slot values for one new Sentinel accumulate and blend in when complete.
> 3. **Exponential tracking.** Once cold standardisation is in service, the working copy advances at the label-time standardisation step and nowhere else.
>
> The first two are observation-authorised. Their assessment-path action is a bounded enqueue or auxiliary-accumulator write, not an update to outcome-learned state. A successfully accepted cold observation may publish only after the assessment that supplied it has fixed its snapshot. Exponential tracking is label-indexed, carries no time-indexed decay, and is inactive while the cold ramp is waiting or transitioning.

Study evidence: the two-clock study distinguishes observational geometry from label-learned truth and identifies the current cold acquisition as the timing anomaly at `packages/assayer/docs/reports/sentinel-two-clocks.md:265-298`.

#### D2 — label-time standardisation · `sec:spec:ramp-label-time-standardisation`

Current, `packages/assayer/docs/spec/data-model-features.md:567-589`:

> “5. **Update the statistics** at rate $\gamma_\text{std}=0.9998$, the mean toward the clipped value and the variance toward its squared deviation from the previous mean. 6. **Apply the variance floor.** 7. **Publish.**”

Proposed replacement:

> At label time, read the current statistics, standardise the reconstructed raw vector, and update the models before any statistic moves. If and only if the full-vector standardisation phase is `InService`, clip the raw vector for the statistics update, advance the mean and variance at $\gamma_\text{std}=0.9998$, apply the variance floor, and publish. In `WaitingForInit` and `Transitioning`, the label uses the published coordinate system and does not also move it; the cold observation ramp is the sole owner of full-vector standardisation during those phases. The bias position is never standardised.

Study evidence: current continuing EWMAs already advance on the label path, and the recommendation preserves that authority rather than mixing two updates in one phase; see `packages/assayer/docs/reports/sentinel-two-clocks.md:55-83` and `packages/assayer/docs/reports/sentinel-two-clocks.md:265-281`.

#### D3 — class count and how long priors matter · `sec:spec:ramp-class-prior-horizon`

Current, `packages/assayer/docs/spec/data-model-features.md:618-622`:

> “Twenty classes, and the values are approximations of the settled distribution rather than guesses: a binary indicator at even odds has mean one half and variance one quarter exactly, and a z-score is defined to have mean zero and unit variance. The priors matter for about one standardisation half-life — some three thousand five hundred labels at the specified rate — after which the running statistics have replaced them.”

Proposed replacement:

> Nineteen classes, and the values are approximations of the settled distribution rather than guesses. At cold start the priors carry all standardisation mass before the first accepted observation, lose exactly $1/N_\text{init}$ of that mass on each ramp advance, and carry none at the target. After the phase enters service, the label-indexed EWMA tracks later distribution change at the specified rate; its half-life describes continuing adaptation, not cold-prior retirement.

Study evidence: the endpoint study verifies that the non-zero class priors are the initial regime and empirical moments the later regime at `packages/assayer/docs/reports/cold-start-uncertainty-step.md:129-168` and `packages/assayer/docs/reports/cold-start-uncertainty-step.md:213-249`.

#### D4 — batch initialisation becomes a finite prior-mass ramp · `sec:spec:ramp-batch-initialisation`

Current, `packages/assayer/docs/spec/data-model-features.md:641-672`:

> “After the first hundred assessments, the class priors are replaced wholesale by empirical statistics.”
>
> “3. **Apply** the empirical mean and variance through the model owner's command channel, and release the accumulator.”
>

Proposed replacement:

> Cold full-vector standardisation is a finite prior-mass ramp over the first $N_\text{init}$ accepted raw assessment vectors.
>
> 1. **Start.** Cold construction publishes the feature-class prior moments, phase `WaitingForInit` and accepted-observation count zero.
> 2. **Offer after acquisition.** An assessment assembles its raw vector after acquiring its model snapshot and offers that vector without blocking. The assessment continues to standardise and score against the acquired snapshot.
> 3. **Accumulate.** The model owner advances numerically stable empirical mean and variance state for each accepted vector. Non-finite entries retain the existing element-by-element skip semantics.
> 4. **Mix.** At accepted count $n$, set $a_n=n/N_\text{init}$ and publish the mixture mean and variance defined by the prior-mass equations above. Bias is untouched. The phase is `Transitioning` while $0<n<N_\text{init}$.
> 5. **Publish progress.** Every accepted advance publishes one immutable model snapshot, increments its snapshot version and carries the accepted count. A full command channel skips the observation, reports that load condition in band, and does not increment the accepted count.
> 6. **Complete.** At $n=N_\text{init}$, prior mass is zero, the moments are the empirical moments of the accepted sample, the phase becomes `InService`, and the completion event is emitted.
> 7. **Persist.** A checkpoint stores the phase, count, base moments and empirical sufficient statistics. Restore resumes them exactly.
>
> A lifecycle change never applies an old-layout observation to a new layout. It discards the incompatible empirical sample, takes the atomically published lifecycle moments as the new base, resets the accepted count to zero and does not roll any published position back.

Study evidence: the present cliff is one asynchronous completion command after the count target, not empirical coverage; see `packages/assayer/docs/reports/cold-start-uncertainty-step.md:78-126`. The later study recommends score-before-evolve, a gradual transition and explicit count and phase at `packages/assayer/docs/reports/sentinel-two-clocks.md:425-440`.

#### D5 — re-standardisation mismatch · `sec:spec:ramp-coordinate-mismatch`

Current, `packages/assayer/docs/spec/data-model-features.md:674-691`:

> “The pending buffer stores raw features (`def:runtime:pending-entry`), so a label re-standardises them against statistics that have moved since the assessment.”
>
> “Over $\Delta k$ intervening labels the mean drift is about $0.002\,\Delta k$ per position.”

Proposed replacement:

> The pending buffer stores raw features, so a label may re-standardise them against a later standardisation snapshot. In `InService`, the existing $\Delta k$ label-time bound applies unchanged. During `Transitioning`, the coordinate movement is instead the finite prior-mass ramp: one accepted observation replaces exactly $1/N_\text{init}$ of base mass, and the compact health phase, accepted count and snapshot version identify the two coordinate states. That mass bound does not assert a linear bound on risk uncertainty, because the standardised vector enters directional quadratic forms and a nonlinear probability transform. The leverage bound continues to limit what one later label update can do to the posterior.

Study evidence: the mixing point is directional and has no independent geometry factor; see `packages/assayer/docs/reports/sentinel-two-clocks.md:85-141`.

#### D6 — lifecycle events during the ramp · `sec:spec:ramp-lifecycle-reset`

Current, `packages/assayer/docs/spec/data-model-features.md:693-706`:

> “A lifecycle event that adds positions gives each new position its class prior; one that removes positions deletes their entries. The map update and the standardisation update are published together as one atomic lifecycle state change (`inv:guarantee:lifecycle-publication`).”

Proposed replacement:

> A lifecycle event that adds positions gives each new position its class prior; one that removes positions deletes their entries. The map and standardisation vectors publish atomically. If the cold ramp is active, that publication becomes the ramp's new base: incompatible sufficient statistics are discarded, the accepted count resets, and existing positions retain their just-published moments rather than reverting to their original priors. The next accepted raw vector must be assembled under that same layout.

Study evidence: the gradual option requires reset and dimension-churn semantics, which the endpoint study identifies as specification work at `packages/assayer/docs/reports/cold-start-uncertainty-step.md:353-365`.

### Warm-up account · `sec:spec:ramp-warmup-account`

#### D7 — the standardisation bootstrap · `sec:spec:ramp-standardisation-bootstrap`

Current, `packages/assayer/docs/spec/analysis-warmup.md:100-127`:

> “Batch initialisation covers the system's own cold start. After the first $N_\text{init}=100$ assessments the empirical mean and variance replace the class priors, which corrects gross mismatches between the priors and the deployment's actual feature distributions (`alg:standardisation:batch-initialisation`).”
>
> “The two halves stand differently against the code. Batch initialisation ships, at the specified sample count and against the specified priors (`tab:standardisation:class-priors`). The per-Sentinel bootstrap does not: no per-Sentinel bootstrap statistics are accumulated and none are blended.”

Proposed replacement:

> Standardisation begins from feature-class priors and acquires deployment measurements in two situations. At system cold start, each accepted raw vector advances the finite prior-mass ramp; the first advances the phase to `Transitioning`, and accepted observation $N_\text{init}$ removes the last prior mass and enters `InService`. The host sees the phase and accepted count throughout, so skipped observations and asynchronous owner publication cannot be mistaken for assessment count.
>
> A Sentinel registered later still uses the per-Sentinel bootstrap. Its slot begins from class priors, accumulates $N_\text{boot}$ reporting assessments, and then blends the empirical moments at $\alpha_\text{boot}=0.8$. Both acquisition mechanisms ship after the cold-ramp implementation; the per-Sentinel accumulator and blend already exist in the current package. Their different publication shapes are explicit rather than concealed.

Study evidence: the two-clock study directly finds the warm-up denial stale against current assessment, command and owner code at `packages/assayer/docs/reports/sentinel-two-clocks.md:299-323`.

#### D8 — warm-up milestones · `sec:spec:ramp-warmup-milestones`

Current, `packages/assayer/docs/spec/analysis-warmup.md:173-198`:

> | First risk assessment | 1 assessment, at $\hat{p} \approx 0.5$ and maximal uncertainty | | Standardisation empirical | ~100 assessments |

Proposed replacement rows:

> | First risk assessment | One assessment, scored in the feature-class-prior coordinate system; phase `WaitingForInit` and accepted count zero in its acquired snapshot | | Standardisation empirical | $N_\text{init}$ accepted cold-ramp observations; about one hundred assessments absent skips or owner backlog |

Proposed replacement for the closing source note:

> The standardisation milestone is fixed by the accepted-observation target and the phase transition in the batch-initialisation algorithm. Assessment count is only an operational approximation because contention and queue pressure may refuse observations.

Study evidence: the current target is accepted raw vectors and the visible request count may be later than one hundred; see `packages/assayer/docs/reports/cold-start-uncertainty-step.md:78-103`.

### Assessment, purity and derivation guarantees · `sec:spec:ramp-assessment-guarantees`

#### D9 — assessment pipeline order · `sec:spec:ramp-assessment-order`

Current, `packages/assayer/docs/spec/operations-assessment.md:55-88`:

> “5. **Assemble and standardise** the feature vector: fixed blocks directly, dynamic blocks from the extractions and the identity state, interactions by iterating the compiled triples, and the whole standardised against the snapshot's statistics (`alg:dimension:compilation-pipeline`).”

Proposed replacement for step five:

> 5. **Assemble, offer and standardise.** Assemble the raw feature vector under the acquired dimension map. While cold initialisation is active, offer that raw vector to the standardisation ramp. Then standardise against the means and variances in the snapshot acquired at the start of the call. Acceptance of the observation may affect only a later snapshot.

Study evidence: this is the verified source order at `packages/assayer/docs/reports/cold-start-uncertainty-step.md:24-51` and the Sentinel score-before-evolve precedent at `packages/assayer/docs/reports/sentinel-two-clocks.md:177-196`.

#### D10 — enumerated writes and evidence authority · `sec:spec:ramp-write-authority`

Current, `packages/assayer/docs/spec/operations-assessment.md:91-117`:

> “The assessment path writes exactly three pieces of shared state and nothing else.”
>
> “Nothing else moves. No model parameter, no dimension map, no standardisation statistic, no identity graph structure — importance, topology or competitive set — and no Ledger entry is written by an assessment, ever.”
>
> “Two accumulators are additional structural writes on this path while they are active, at batch initialisation and per-Sentinel bootstrap, and the two deferred writes — the identity observation and the signal-cache insertion — are enqueues rather than mutations of the structures they feed, both drained on the maintenance thread's cycle (`alg:publication:identity-draining`).”

Proposed replacement around the unchanged three-row table:

> The assessment path directly writes exactly the three pieces of shared state in this table. It may additionally enqueue the identity observation, signal cache insertion, cold standardisation observation and per-Sentinel bootstrap observation to their owning threads, and the two bootstrap accumulators may advance while active. None of those actions changes the immutable snapshot the assessment acquired.
>
> An assessment never advances an operational, sister, anchor or outcome posterior; a Platt parameter or calibration diagnostic; an outcome-memory value; or the continuing standardisation EWMA. Those are label-authorised. Accepted cold and per-Sentinel observations may advance observational geometry for later snapshots. The distinction is evidence authority, not a claim that an assessment leaves no residue.

Study evidence: the endpoint study finds no derivation-purity breach but directly refutes the test's residue-free explanation at `packages/assayer/docs/reports/cold-start-uncertainty-step.md:253-291`.

#### D17 — new guarantee beside derivation purity · `sec:spec:ramp-evidence-guarantee`

Current after `packages/assayer/docs/spec/reference-guarantees.md:35-45`:

> No passage separates assessment-time observational movement from label-authorised learning. The existing invariant speaks only about the derivation function.

Proposed new head for the ADR lane to name, own in the guarantees chapter and mint:

> **Invariant (Assessment preserves evidence authority).** Derivation of one held assessment is deterministic and state-free. Assessment may advance only the enumerated bookkeeping and observation-authorised measurement state; it does not advance any outcome-learned state. A cold standardisation observation is applied only to a later immutable snapshot, replaces at most $1/N_\text{init}$ of the ramp's base mass, and is accompanied by phase, accepted count and snapshot version. Repeated assessments are therefore not promised bit-identical risk values across coordinate versions; repeated derivations of one held assessment are.

Study evidence: the two studies agree on literal derivation purity and on the overbroad assessment claim; see `packages/assayer/docs/reports/cold-start-uncertainty-step.md:253-291` and `packages/assayer/docs/reports/sentinel-two-clocks.md:460-470`.

### Reporting and health surfaces · `sec:spec:ramp-health-surfaces`

#### D11 — compact assessment health · `sec:spec:ramp-compact-health`

Current, `packages/assayer/docs/spec/operations-assessment.md:307-337`:

> “Every assessment carries a compact, fixed-size health snapshot — nine fields, about sixty-four bytes — so that a per-request consumer can tell a trustworthy assessment from a provisional one without fetching anything.”
>
> | Snapshot version | The join key into the full health report stream |
>
> “What ships carries five of the nine fields and the version is not among them, so the forensic join this environment exists to establish cannot be made: an assessment log and a health report stream sit side by side with nothing to match them on.”

Proposed replacement:

> Every assessment carries a compact, fixed-size health snapshot with eleven specified fields. Add these rows to the existing table:
>
> | Standardisation phase | `WaitingForInit`, `Transitioning` or `InService` for the acquired coordinate snapshot | | Standardisation observations | Accepted cold-ramp observations represented by the acquired snapshot, from zero through $N_\text{init}$ |
>
> Snapshot version remains the join key and is also the coordinate version: each ramp advance publishes a model snapshot, so phase and count attribute the reason without a second version namespace. Ramp maturity is count divided by $N_\text{init}$ and is not stored separately. After the reporting change, the shipped compact type carries ten of the eleven specified fields; anchor-regime frozen remains the unrelated missing row.

Study evidence: disclosure is mandatory to distinguish a geometry transition from outcome learning; see `packages/assayer/docs/reports/sentinel-two-clocks.md:22-32` and `packages/assayer/docs/reports/sentinel-two-clocks.md:425-440`.

#### D12 — new standardisation transition health passage · `sec:spec:ramp-transition-health`

Current after `packages/assayer/docs/spec/analysis-monitoring.md:17-27`:

> No environment defines the standardisation transition's compact and full reporting fields together.

Proposed new head for the ADR lane to name, own in health monitoring and mint:

> **Table (Standardisation transition health).** Standardisation transition health reports and never gates. The compact assessment snapshot and full health report both carry phase and accepted cold-ramp observation count. The compact snapshot also carries the model snapshot version that fixes the coordinate state used for that result. The full report continues to carry features at the variance floor. `WaitingForInit` means count zero and class-prior moments; `Transitioning` means a positive count below $N_\text{init}$ and mixed moments; `InService` means count $N_\text{init}$ and no cold-prior mass. Queue refusal and contention counters remain degradation diagnostics and do not masquerade as accepted observations.

Study evidence: Sentinel reports maturity and geometry so a host can attribute movement, and the Core recommendation imports that disclosure discipline at `packages/assayer/docs/reports/sentinel-two-clocks.md:204-234` and `packages/assayer/docs/reports/sentinel-two-clocks.md:425-440`.

### Configuration reference and the other timing tables · `sec:spec:ramp-configuration-timing`

#### D13 — standardisation configuration · `sec:spec:ramp-configuration`

Current, `packages/assayer/docs/spec/reference-configuration.md:208-228`:

> | $N_\text{init}$ (batch initialisation sample count) | 100 | $\geq 10$ | (`alg:standardisation:batch-initialisation`) |
>
> “All six values ship as tabulated. The two bootstrap rows carry a documentation defect rather than a value defect: the package's own field comments describe them as a replicate count and a confidence level, which is what those names would mean in a resampling procedure and is not what they mean here. The numbers are right and the prose beside them is not.”

Proposed replacement table row:

> | $N_\text{init}$ (cold prior-mass ramp horizon) | 100 | $\geq 10$ | the batch-initialisation algorithm |

Proposed replacement closing paragraph:

> All six values ship as tabulated after the cold-ramp implementation. The value of $N_\text{init}$ is unchanged; its semantics are the accepted-observation horizon over which prior mass falls to zero, not a gate followed by wholesale publication. The two per-Sentinel bootstrap fields are already documented as sample count and blend factor and are consumed by that mechanism; there is no remaining field-comment defect in those rows.

Study evidence: the count is already the correct operating point and the defect is the publication cliff, not the number itself; see `packages/assayer/docs/reports/cold-start-uncertainty-step.md:78-103` and `packages/assayer/docs/reports/sentinel-two-clocks.md:425-440`.

#### D14 — publication staleness · `sec:spec:ramp-publication-staleness`

Current, `packages/assayer/docs/spec/operations-concurrency.md:129-145`:

> | Standardisation statistics | Labels processed since the last publication | Per label, per feature |

Proposed replacement row:

> | Standardisation statistics | Accepted cold observations queued since the last ramp publication; labels processed since the last in-service publication | Per accepted observation while transitioning; per label in service |

Study evidence: current source has two clocks for this state, and the cold assessment clock is the isolated exception; see `packages/assayer/docs/reports/sentinel-two-clocks.md:55-83`.

#### D15 — temporal inventory · `sec:spec:ramp-temporal-inventory`

Current, `packages/assayer/docs/spec/operations-temporal.md:36-56`:

> | Standardisation statistics | Label-indexed | $0.9998$ | Coordinate system evolution |

Proposed replacement row:

> | Standardisation statistics | Observation-indexed finite ramp before service; label-indexed EWMA in service | $1/N_\text{init}$ base-mass replacement, then $0.9998$ | Coordinate system acquisition and continuing evolution |

Study evidence: the authority census verifies cold assessment acquisition and continuing label-time updates at `packages/assayer/docs/reports/sentinel-two-clocks.md:33-83`.

#### D16 — pre-seeding · `sec:spec:ramp-preseeding`

Current, `packages/assayer/docs/spec/operations-runtime.md:140-146`:

> “A host holding historical outcomes may preload them, and they travel the ordinary label path rather than a shortcut: standardisation statistics move, both class-rate trackers move, per-axis compression scales move, and every eligible model updates, under the same eligibility rules live labels obey (`alg:runtime:update-path`).”

Proposed replacement:

> A host holding historical outcomes may preload them through the ordinary label path. Posteriors, class-rate trackers and per-axis compression scales move under the live eligibility rules. Standardisation statistics move on that path only when the full-vector phase is `InService`; before then the cold observation ramp owns the coordinate transition, so pre-seeding cannot race or be overwritten by it.

Study evidence: preserving separate evidence authorities is the central two-clock recommendation at `packages/assayer/docs/reports/sentinel-two-clocks.md:33-83`.

### Passages confirmed truthful and unchanged · `sec:spec:ramp-unchanged-passages`

Each row is a separate passage in the census. “Unchanged” is the proposed text.

| Passage | Current sentence or row | Why unchanged; study evidence |
| --- | --- | --- |
| `packages/assayer/docs/spec/data-model-features.md:529-543` | “Standardisation makes the single prior defensible by making the positions comparable.” | The ruling changes acquisition timing, not purpose. The two-clock study confirms coordinate infrastructure at lines 265-275. |
| `packages/assayer/docs/spec/data-model-features.md:591-617` | “Every position starts at the prior mean and variance of its class,” followed by the nineteen-row table. | The table values and starting-moment rule are the authority the ruling enables. The endpoint study verifies the implemented values at lines 129-146. |
| `packages/assayer/docs/spec/data-model-features.md:624-639` | “Every position is assigned a class, and the assignment is derived from three authorities and configured by nobody.” | Class assignment is already correct and is not a timing decision. The endpoint study verifies the assignment path at lines 129-136. |
| `packages/assayer/docs/spec/data-model-features.md:710-730` | “The bootstrap shortens that to a hundred assessments.” | The late-Sentinel mechanism remains observation-authorised and keeps its retained-prior blend. The two-clock study verifies it at lines 45-49 and 288-292. |
| `packages/assayer/docs/spec/reference-guarantees.md:601-608` | “The per-Sentinel bootstrap shortens the interval substantially and does not remove it.” | This is the honest residual after leaving that lifecycle-scoped bootstrap unchanged. The two-clock study verifies the caveat at lines 282-292. |
| `packages/assayer/docs/spec/reference-guarantees.md:35-45` | “The derivation reads no state outside its arguments, writes no state, and is deterministic: the same inputs produce the same output on any call, in any order, on any process (`pf:landscape:purity`).” | The endpoint study expressly finds no breach at lines 253-263. D17 adds assessment scope and does not weaken this invariant. |
| `packages/assayer/docs/spec/landscape-derivation.md:145-166` | “The derivation reads no global state, writes no state, allocates no persistent resource, has no side effect, and returns the same landscape for the same three inputs.” | The proof concerns one held assessment and remains exact. The endpoint study verifies that boundary at lines 253-263. |
| `packages/assayer/docs/spec/landscape-guidance.md:201-206` | “The derivation cannot compute the Core's guidance, because that depends on stored model state a pure function does not hold (`pf:landscape:purity`).” | The state boundary is unchanged; the cold ramp lives in Core before derivation. |
| `packages/assayer/docs/spec/landscape-worked.md:177-186` | “Purity | Deterministic, stateless, closed form, presentation-free (`pf:landscape:purity`)” and “Replay | The three stored inputs reproduce the landscape exactly (`pf:landscape:purity`).” | Both rows concern repeated derivation, not repeated assessment. |
| `packages/assayer/docs/spec/appendix-interfaces.md:108` | “Derivation Function to Core | Never | It reads the risk basis and writes nothing (`inv:guarantee:derivation-purity`).” | The ruling changes how a later risk basis may be produced, not what derivation writes. |
| `packages/assayer/docs/spec/reference-outputs.md:9-35` | “And the system health report is the Core's health surface, emitted on its own cadence rather than per assessment, and joined to the assessment stream by the snapshot version the compact snapshot carries (`schema:output:health-snapshot`).” | Reusing snapshot version as coordinate version fulfils this existing account; D11 and D12 supply the missing fields. |
| `packages/assayer/docs/spec/analysis-monitoring.md:17-27` | “No diagnostic in this chapter modifies a model, a threshold, a rate, or a routing decision. Each computes a quantity, reports it, and stops.” | Phase and count diagnose the ramp; they do not drive it. The two-clock study requires disclosure, not a health gate. |
| `packages/assayer/docs/spec/data-model-registries.md:107` | “6. **Extend standardisation.** Append entries at the feature-class priors.” | Sentinel lifecycle extension remains correct; D6 defines only an active-ramp reset. |
| `packages/assayer/docs/spec/data-model-registries.md:267` | “5. **Extend standardisation.** Append entries at the feature-class priors.” | Outcome-axis lifecycle extension remains correct. |
| `packages/assayer/docs/spec/data-model-registries.md:399` | “4. **Extend standardisation.** Append entries at the feature-class priors.” | Identity-dimension lifecycle extension remains correct. |
| `packages/assayer/docs/spec/data-model-identity.md:329-331` | “2. **Extend standardisation** at the class priors: a binary indicator at mean one half and variance one quarter, an interaction product at mean zero and unit variance.” | The concrete priors match the table and remain the correct lifecycle base. |

## Derivation chain: the ADR · `sec:spec:ramp-record-derivation`

The follow-on ADR must record these decision heads. The names below are descriptions for the ADR lane; they are not labels and this report mints none.

1. **Cold standardisation is a finite prior-mass ramp** — owned by the feature vector and standardisation record; record the mixture equations, $N_\text{init}=100$, the three phases and exact empirical endpoint.
2. **Observation advances only a later coordinate snapshot** — owned by ordering and concurrency; record snapshot acquisition before the ramp offer, single- steward application, per-advance publication and in-band refusal reporting.
3. **Assessment preserves outcome-evidence authority** — owned by ordering and derivation boundaries; record the three purity promises and reject residue-free assessment wording.
4. **Transition state is reported, not inferred** — owned by surface and health; record phase and count on compact and full health, reuse snapshot version, and derive maturity rather than storing it.
5. **Cold ramp progress survives compatible restore** — owned by vector and durability; checkpoint sufficient statistics and resume, while a structural mismatch cold-starts under the existing compatibility rule.

The vector record needs substantive amendments. Its statistic-timing decision must distinguish cold observation acquisition from continuing label updates (`dec:vector:statistic-timing`). Its two-acquisitions decision must replace “priors until completion” with mixed moments during transition (`dec:vector:two-acquisitions`). Its transient-accumulator decision must retain transience for per-Sentinel bootstrap but make cold ramp progress part of the whole-state checkpoint (`dec:vector:transient-accumulators`). Its derived-prior decision remains the source of the initial moments (`dec:vector:derived-class-priors`).

The derivation record's pure-transform decision is reaffirmed, not amended (`dec:derivation:pure-transform`). The surface record's inline-health decision needs only the new schema fields, not a new transport rule (`dec:surface:inline-health`). The concurrency record already supplies immutable swap, per-request acquisition, one steward and visible overflow (`dec:concurrency:snapshot-swap`) (`dec:concurrency:per-request-load`) (`dec:concurrency:single-steward`) (`dec:concurrency:no-silent-drop`). The durability record's whole-state checkpoint and structural compatibility already warrant persisting the ramp (`dec:durability:checkpoint-journal`) (`dec:durability:structural-compatibility`). The health record's tiering, non-exhaustive report and report-only posture remain unchanged (`dec:health:tiered-queries`) (`dec:health:non-exhaustive-report`) (`dec:health:reports-never-gates`).

The requested caveat audit finds **nothing to amend or discharge in the challenge or health records**. At their mints, the four challenge caveats concern an unwired tracker, the absent replacement arrangement, the sufficiency floor and thin evidence at `packages/assayer/adr/challenge.md:135-228`; the three health caveats concern identity maturity arithmetic, retired convergence configuration and a repaired event defect at `packages/assayer/adr/health.md:187-255` and `packages/assayer/adr/health.md:365-382`. None states or warrants cold standardisation. The verified challenge mints are (`cav:challenge:unwired`) (`cav:challenge:arrangement-open`) (`cav:challenge:sufficiency-threshold-open`) (`cav:challenge:thin-evidence`). The verified health mints are (`cav:health:identity-maturity-arithmetic`) (`cav:health:dead-convergence-thresholds`) (`cav:health:tracker-event-defect`). Treating one as overtaken would be a false derivation. The vector record's late-Sentinel caveat remains live and unchanged (`cav:vector:late-standardisation`).

Records affected are therefore `vector` substantively; `ordering`, `surface` and `durability` with new decisions or consequences; and `concurrency`, `health` and `derivation` by reaffirmed citations. `challenge` is audited and unaffected.

## Derivation chain: code and tests · `sec:spec:ramp-implementation-derivation`

| Spec deltas | Code change | Required tests |
| --- | --- | --- |
| D1-D4, D7, D13 | In `packages/assayer/src/snapshot/working.rs`, initialise cold means and variances with `init_from_priors`; store phase, accepted count and cold-ramp state. In `packages/assayer/src/feature/bootstrap.rs`, make the Welford accumulator expose progress and compute the between-population variance. In `packages/assayer/src/feature/standardisation.rs`, add `Transitioning` without renaming the existing enum variants and change `n_init` documentation from completion gate to ramp horizon. | Unit tests for every class prior; phase transitions at zero, one, target-minus-one and target; exact empirical endpoint; between-population variance; bias exclusion; non-finite handling; and the identical-zero fixture's monotone trajectory. |
| D1, D4, D9, D10, D14 | Replace `BatchInitComplete` in `packages/assayer/src/owner/commands.rs` with an accepted-observation/progress command. Move cold ramp mutation under the steward in `packages/assayer/src/owner/thread.rs`. Replace the completion-only mutex path in `packages/assayer/src/lib.rs` with non-blocking submission that reports refusal and increments no accepted count. Carry enough layout identity to refuse a vector assembled under incompatible coordinates. | Integration tests that the current request uses its acquired snapshot; each accepted command advances once; FIFO owner handling cannot skip a count silently; full-channel refusal is in-band and does not advance; lifecycle change refuses old-layout observations; and completion emits once. |
| D2, D16 | In `packages/assayer/src/owner/label_path.rs`, read stored phase instead of inferring it from variance floors and skip the continuing EWMA until `InService`. | Label-path and replay tests proving models still update in transition, standardisation does not, and the first in-service label advances the EWMA exactly once. |
| D4, D6 | Extend `packages/assayer/src/persistence/checkpoint.rs` and the checkpoint conversions in `packages/assayer/src/snapshot/working.rs` with phase, count, base moments and Welford state; bump the checkpoint format once. | Mid-ramp checkpoint round-trip and resume; empirical-phase round-trip; structural mismatch cold-start; lifecycle reset preserves published moments and clears incompatible sufficient statistics. |
| D11, D12 | Add phase and count to `ModelSnapshot` in `packages/assayer/src/snapshot/published.rs`, `HealthSnapshot` in `packages/assayer/src/assessment.rs`, `PublishedHealthSummary` in `packages/assayer/src/health/published.rs`, `StandardisationHealth` in `packages/assayer/src/health/summary.rs`, and the mapper in `packages/assayer/src/api/health.rs`. Update `packages/assayer/src/metrics/mapper.rs` and its catalogue only if the public metrics surface is kept aligned; the phase gauge must distinguish transition or be paired with the count gauge. | Inline/full health agreement at each phase; snapshot version changes on each ramp publication; Serde coverage for the added enum variant and report fields; metrics catalogue/mapper agreement; assessment health reports the snapshot it used, not a later count. |
| D3, D7, D11, D13 | Remove the reversed-arrow hold comment in `packages/assayer/src/snapshot/working.rs`; update Rustdoc that currently says waiting uses priors until one completion and any test indices that repeat the stale class count or phase semantics. | Source-level documentation/corpus checks plus the existing feature-class assignment and builder validation suites. |
| D10, D17 | Split `packages/assayer/tests/derive_purity.rs`: retain `public_derive_reckoning_bit_identical`; replace `thousand_identical_derivations_stable` with an outcome-state authority test and a ramp disclosure/trajectory test; update `identical_inputs_tag_shape_stable` so it compares derivation from one held assessment rather than fresh assessments across coordinate versions. Remove `PURITY_DRIFT` wherever it served only the overbroad assessment claim. | Bit-identical held derivation; no posterior, calibration, outcome-memory or label-EWMA movement under repeated assessments; correct phase/count/version progression; old endpoints retained; largest one-step fixture movement below one tenth of total endpoint movement. |

The code lane must not compensate with empirical-coverage inflation. That field is label-derived and report-only, and neither study finds it on the risk path (`inv:monitoring:report-only`). It must also not create a scalar “geometry uncertainty” factor: the directional quadratic forms and blend do not admit one.

## Premise audit · `sec:spec:ramp-premise-audit`

### Verified claims relied on · `sec:spec:ramp-verified-premises`

- **Mixing point verified.** The current assessment offers the raw vector to batch initialisation and standardises against its previously acquired snapshot at `packages/assayer/src/assessment.rs:1107-1144`. The standardised vector enters sister and anchor quadratic forms and the blend at `packages/assayer/src/risk/blend.rs:150-201`, then probability uncertainty is `p_bad * (1 - p_bad) * sigma_eff / kappa_eff` at `packages/assayer/src/assessment.rs:291-302`. This matches both studies.
- **Count gate and asynchronous publication verified.** The assessment-side accumulator completes and sends one owner command at `packages/assayer/src/lib.rs:1271-1306`; the steward replaces every non-bias moment and publishes at `packages/assayer/src/owner/thread.rs:540-583`.
- **Two authorities verified.** Cold batch acquisition is assessment-clocked at `packages/assayer/src/lib.rs:1271-1306`, while the continuing full-vector EWMA is label step fifteen at `packages/assayer/src/owner/label_path.rs:892-904`. Per-Sentinel bootstrap is a second, lifecycle-scoped observational acquisition, so “lone anomaly” is valid only for the cold full-vector path, not as a claim that no other assessment-time measurement accumulator exists.
- **Held priors verified.** Cold construction derives classes but publishes neutral zero/unit moments at `packages/assayer/src/snapshot/working.rs:241-268`; `init_from_priors` exists at `packages/assayer/src/feature/standardisation.rs:369-382` and has no production caller.
- **Purity scope verified.** The held-assessment integration test calls the stateless derivation repeatedly at `packages/assayer/tests/derive_purity.rs:123-154`. The separate test at `packages/assayer/tests/derive_purity.rs:156-191` performs fresh assessments and asserts residue-free scalar stability, which is outside the derivation invariant.

### Premise errors and current-tree staleness · `sec:spec:ramp-premise-errors`

- The hold comment still says the sequence steps from about `0.79` to about `2.52` at `packages/assayer/src/snapshot/working.rs:246-255`. With priors enabled, the studies' verified arrow is the reverse: about `2.52` down to about `0.79`.
- The class-prior paragraph says “Twenty classes” at `packages/assayer/docs/spec/data-model-features.md:618`, but its table and the `FeatureClass` enum at `packages/assayer/src/feature/standardisation.rs:73-113` contain nineteen.
- The warm-up chapter says the per-Sentinel bootstrap does not ship at `packages/assayer/docs/spec/analysis-warmup.md:119-127`. Current observation, command and owner code ships it; D7 corrects the stale passage.
- The configuration chapter says `n_boot` and `alpha_boot` have replicate-count and confidence-level field comments at `packages/assayer/docs/spec/reference-configuration.md:223-228`. Current comments name sample count and blend factor correctly at `packages/assayer/src/feature/standardisation.rs:188-195`.
- The compact-health chapter says only five of nine specified fields ship and snapshot version is absent at `packages/assayer/docs/spec/operations-assessment.md:335-337`. Current `HealthSnapshot` at `packages/assayer/src/assessment.rs:541-580` carries eight of the nine semantic rows plus degradation; only anchor-regime frozen is absent.
- A public two-state `StandardisationPhase` already exists at `packages/assayer/src/feature/standardisation.rs:38-58`, but batch completion does not store it. Full health infers phase from how many variances sit at the floor, and only on label publication, at `packages/assayer/src/owner/label_path.rs:1267-1297`. That is not a phase clock and cannot report accepted ramp progress.
- None of the challenge or health record caveats is overtaken, as the mint audit above records. The brief's parenthetical was a verification request, not evidence that such a caveat must exist.

### Ramp audit · `sec:spec:ramp-design-audit`

A standard-library-only calculation applied the proposed mixture to the study's identical-zero fixture, including the shipped standardisation epsilon, variance floor and blend guard. It reproduced the verified endpoints `2.5199205337637465` and `0.79056941504209488`, moved monotonically over the one hundred accepted observations, and found the largest consecutive move from count zero to one: `0.031120373533726653`, about `0.0179954` of the complete endpoint movement. The ramp therefore changes neither endpoint nor direction and spreads the old cliff well inside the stated falsifier.

## Process and verification · `sec:spec:ramp-verification-process`

The worktree was clean on `lane/uspec` at `78c84190aedd67e533e2ea49c10385de325904b9`. No Cargo command, compiler, network access, specification edit, record edit, implementation edit or test edit was used. This report is the only file touched.

| Step | Command or activity | Wall time and result |
| --- | --- | --- |
| Baseline state | `git status --short --branch`; `git log -1 --format=%H` | 0.12 s and 0.00 s; clean `lane/uspec`, expected base. |
| Baseline corpus | prebuilt linter `check --root .` | 6.10 s; 348 label sources, 710 profiled files, no failures or warnings. |
| Evidence reading | complete `sed` reads of both prerequisite studies, plus sibling-style reads | Each timed command completed in 0.00 s; no read was truncated in its final pass. |
| First-phase safety | `git diff --check`, `git status --short`, banned-token fixed-string scan | 0.16 s, 0.18 s and 0.00 s; only this report, no whitespace error, token absent. |
| First commit | stage and commit the report scaffold | The sandboxed stage failed read-only in 0.00 s, the authorised retry took 0.11 s, and commit `8385a366` took 0.40 s. |
| First-commit corpus | prebuilt linter `check --root .` | 6.49 s; 349 label sources, 710 profiled files, no failures or warnings. Only the expected prose-carrier source count changed. |
| Corpus census | timed `rg` searches over specification, records, implementation and tests; targeted `sed` reads | Individual searches took 0.00–0.02 s; targeted reads took 0.00–0.21 s; all exited zero except the expected no-match banned-token scan. |
| Ramp arithmetic | standard-library Python | First invocation exposed a parenthesis syntax error in 0.05 s; corrected invocation exited zero in 0.03 s and produced the figures above. |
| Completed-report corpus | prebuilt linter `check --root .` | 8.42 s; 349 label sources, 13,018 resolved citations, no failures or warnings. |
| Completed-report commit | `git commit` | 0.39 s; `90d51309`, one report file changed. |
| Post-commit corpus | prebuilt linter `check --root .` | 8.51 s; 349 label sources, 13,018 resolved citations, no failures or warnings. |

The one arithmetic syntax failure and the sandboxed staging failure are reported rather than absorbed; neither changed a tracked file or any conclusion. The handoff supplies the final status and checker evidence after the metadata-only closing commit, because a commit cannot contain evidence generated after itself.
