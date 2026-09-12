### Chapter (Guarantees and Known Limitations) · `chap:spec:guarantees-and-limitations`

The document's two registers of record. The first says what the system guarantees; the second says what it cannot do. They are written as one chapter because they answer one question between them — what a host may rely on — and a reader who has only the first half of that answer has been misled by omission.

Two rules govern the chapter. **A formal property is stated once.** Most are stated here; seven are stated at the chapter that owns them, where the property is the whole content of an environment, and this chapter cites those seven rather than restating them. **A limitation is stated once, here**, and every chapter that creates an exposure cites it. There are forty-two.

**Architectural and operational properties.** Each names a property of the system's structure or behaviour.

**Invariant (Assessment writes only what is enumerated)** · `inv:guarantee:assess-only`

The Core never selects an action. An assessment reads state, computes an estimate, and writes only the enumerated set of measurement updates that its own specification lists (`inv:runtime:enumerated-writes`); no branch of it chooses, recommends or applies a disposition (`prin:principle:measure-not-decide`).

The property is the architecture's load-bearing separation rather than a convenience: it is what lets a host change its policy without retraining, and what makes every other guarantee here a guarantee about measurement rather than about behaviour.

**Invariant (The derivation is pure)** · `inv:guarantee:derivation-purity`

The derivation reads no state outside its arguments, writes no state, and is deterministic: the same inputs produce the same output on any call, in any order, on any process (`pf:landscape:purity`).

Purity is what makes the derivation cheap to call and safe to call repeatedly, and it is the premise every replay argument in this document rests on (`sig:landscape:derivation-function`).

**Invariant (Assessment preserves evidence authority)** · `inv:guarantee:evidence-authority`

An assessment advances only the bookkeeping its own specification enumerates (`inv:runtime:enumerated-writes`) and the observational measurement its two acquisition mechanisms take; it advances no state that an outcome taught. An accepted cold standardisation observation reaches only a snapshot later than the one its own request was answered from, retires at most $1/N_\text{init}$ of the ramp's base mass, and travels with the phase, the accepted count and the snapshot version that place it (`alg:standardisation:batch-initialisation`).

The property is the honest form of a promise this document once made too broadly. Repeated derivations of one held assessment are bit-identical, which is the invariant above; repeated assessments of one subject are not promised identical risk values across coordinate versions — an accepted observation is residue, deliberately, and the ramp exists in order to leave it. What is guaranteed instead is that the residue is observational. No model parameter, no calibration state, no outcome-memory value and no continuing standardisation average moves anywhere but on the label path, so a host that reads two different values for one subject knows the difference is a coordinate system it can name from the version rather than learning it cannot see. Each accepted cold observation publishes one ramp advance, and every assessment carries the phase, accepted count and version that place its coordinate state.

**Invariant (The Companion is independent of the Core)** · `inv:guarantee:companion-independence`

The Companion reads no Core state and no Core state depends on Companion state. The two meet only in the derivation, which consumes an output of each and writes back to neither (`inv:companion:boundary`).

The independence is why the Companion can be replaced wholesale by a host with its own estimator, and why a Companion failure degrades the landscape's uncertainty rather than corrupting the Core's posterior.

**Invariant (The blend operates within the anchor's subspace)** · `inv:guarantee:blend-subspace`

Anchor activation depends only on uncertainty in the anchor's own subspace, and is insensitive to Sentinel-specific and axis-specific uncertainty (`def:risk:subspace-blend`).

Without the restriction the anchor would activate whenever any part of the feature space was uncertain, which is most of the time on a growing deployment, and the system would fall back to its coarsest model exactly when its finer ones were becoming useful.

**Invariant (Ledger influence is floored, never eliminated)** · `inv:guarantee:ledger-floor`

Stale reputation decays to insignificance within a bounded interval regardless of label flow, because the Ledger's decay is indexed by elapsed time and not only by labels (`thm:temporal:ledger-decay-bound`). The bound holds under the stated condition that time-indexed decay is enabled (`def:ledger:time-decay`).

The guarantee is directed at the contamination loop rather than at freshness: a memory that decayed only on labels would hold a restricted entity's reputation fixed for as long as the restriction suppressed its labels.

**Invariant (Axis predictions never enter the derivation)** · `inv:guarantee:outcome-neutrality`

Outcome axis predictions are not inputs to the derivation. Axis features influence risk through the Core, as designed; axis *outputs* reach the host and stop there (`def:landscape:outcome-neutrality`).

The boundary is what keeps an axis a measurement rather than a second opinion. A host wanting an axis to affect a decision composes it into its own policy, where the composition is visible and reviewable, instead of having it silently enter a crossover.

**Invariant (Model drift is detected and reported)** · `inv:guarantee:drift-visibility`

Each Core model's calibration drift is measured against its own predictions and reported, so that a model degrading against the population it serves is observable without an external evaluation (`alg:monitoring:drift-cusums`).

The measurement is one-sided in each direction and accumulates, which is what lets a small persistent bias be detected long before it is visible in an aggregate error rate. The allowance the accumulators forgive before growing is tabulated with the monitoring configuration (`tab:config:monitoring`).

**Invariant (The Companion's update is conjugate)** · `inv:guarantee:conjugacy`

The Companion's update is exact conjugate arithmetic: no approximation is introduced and none accumulates across any number of updates (`alg:companion:update`).

Exactness matters here more than its cheapness. The Companion's estimate is built from a very thin stream of contributing labels, so an approximation error that accumulated would be indistinguishable from evidence and would be trusted as such.

**Invariant (A recorded assessment replays exactly)** · `inv:guarantee:replay`

Given a stored risk assessment, the channel policy it was derived under, and the challenge posterior of that moment, the landscape is reproducible exactly, without access to any model state (`ex:runtime:composition`).

This is what makes an audit trail an audit trail: the three stored values are the complete input, so a decision can be reconstructed months later on a system whose models have moved on entirely.

**Invariant (Empirical coverage is tracked and reported)** · `inv:guarantee:discrimination-visibility`

Rank discrimination, top-decile lift, per-axis correlation and the empirical coverage of the reported uncertainty are all computed and reported, so that a host can see both whether the ordering is right and whether the stated uncertainty is honest (`alg:monitoring:auc`).

The last of the four is the one the property is really about. Discrimination says the ordering is useful; coverage says the intervals mean what they claim, and the two can fail independently. All four are computed and reported. The coverage fractions and the inflation factor are formed from the calibration rows at each refit, gated at the calibration minimum per regime, and the inflation factor rides the compact snapshot besides (`alg:monitoring:empirical-coverage`).

**Lifecycle and numerical properties.** Each states a property of a dimension change or posterior computation.

**Invariant (Lifecycle structure is preserved and approximation is explicit)** · `inv:guarantee:structural-exactness`

Extension is an exact transformation of the posterior, and the unregularised Schur identity is its exact marginalisation (`thm:gaussian:extension`), (`thm:gaussian:marginalisation`). The regularised marginalisation preserves the declared partition and mean subvector while applying the correction or its explicit kept-block fallback. Its numerical posterior is therefore an approximation whenever the offset changes a non-zero correction.

The structural property is what makes a changing feature space tractable at all: the same dimensions are removed from every model and each result is adopted atomically. The approximation is separately observable, and as a computed figure rather than only as an outcome: every event reports what its regularisation retained over the exact marginal, labelled as a measurement or as a bound, and a discarded correction reports the whole of itself (`req:gaussian:marginalisation-error-reported`). The corpus still states no cumulative bound for repeated regularised or fallback marginalisations. What it now carries is a cumulative report of them, which is a monitor standing where the bound is absent rather than the bound itself.

**Invariant (Axis lifecycle preserves structure with declared marginalisation)** · `inv:guarantee:axis-lifecycle`

Registering an outcome axis is exact extension on every model it touches. Deregistering one applies the declared regularised marginalisation on every surviving model, so an axis set may be reshaped at runtime without retraining anything (`alg:axis:lifecycle`).

The guarantee is the structural preservation above applied to the one lifecycle a host is most likely to exercise repeatedly, since axes are cheap to propose and easy to retire. The registration handler's own under-extension is recorded at that algorithm rather than here (`alg:registry:axis-registration`).

**Invariant (Precision is maintained within its bounds)** · `inv:guarantee:precision`

A spectral floor bounds the posterior precision matrix's least eigenvalue from below at every recomputation, which is where the floor is measured and applied and where the factorisation that needs it is performed, so the matrix is factorisable in the working arithmetic at every point at which it is factorised (`dec:posterior:spectral-floor`). Between recomputations the forgetting scales the spectrum by a positive factor, which preserves positive definiteness exactly and costs only margin — bounded by that factor over the interval, and absorbed by the next recomputation's own measurement. The replenishment floor bounds each diagonal entry and does not bound the spectrum (`req:gaussian:prior-replenishment-floor`); the synchronisation monitor detects the drift between the tracked precision and covariance before it becomes numerically material (`alg:gaussian:synchronisation-monitor`).

The three are separate guarantees and the distinction is the whole content of the statement. The spectral floor is what makes factorisability a precondition the model maintains rather than an outcome it hopes for, which is what lets a refused factorisation be read as a verdict against the invariant rather than as a condition to repair. It is a bound on a definite matrix and not a route into the definite cone: a matrix already outside the cone is refused rather than floored, since the amount that would carry a negative eigenvalue in is a repair of the damage rather than the least value the arithmetic requires. The coordinatewise floor is a modelling statement about how much confidence a single feature may lose and bounds no eigenvalue: equal positive diagonal entries can coexist with an arbitrarily small eigenvalue through the off-diagonal structure, which the requirement it names says in its own words. The monitor catches the drift neither floor addresses.

What the statement previously claimed was that the replenishment floor bounded the condition number, which the requirement it cited refutes in its own text. The clause is not weakened here but relocated: it becomes true of a floor on the spectrum, and the floor on the diagonal keeps the narrower guarantee it always had.

**Algebraic and conditional properties.** Each is a claim a deployment must satisfy under its stated premises.

**Invariant (The per-observation update is exact)** · `inv:guarantee:per-observation-exactness`

Each update is exact Bayesian inference given the current posterior and the step's effective precision, and the composite posterior that results from a sequence of them is conservative — wider than a fixed-model posterior over the same evidence (`thm:gaussian:two-level-guarantee`).

The two halves are different kinds of claim and the second is the one that matters operationally: exactness is per step, and what a host relies on across steps is the direction of the error rather than its absence (`prop:gaussian:conservatism`).

**Invariant (The blend's variance takes the stated form)** · `inv:guarantee:blend-variance`

The blended estimate's uncertainty is the mixture variance under the blend weight, in the three-term form the derivation gives, and not the weighted average of the component variances (`prop:risk:blend-variance`).

The distinction is the whole content of the property. The missing term is the disagreement between the two models being blended, and a consumer that dropped it would report a confident estimate at exactly the moments when the system's two views of an entity disagreed most.

**Invariant (The crossover set translates rigidly)** · `inv:guarantee:rigid-translation`

Every crossover decomposes exactly into a shared term and a per-action offset, so a change in the shared term moves the whole crossover set together without altering the spacing or the order of its members (`thm:landscape:rigid-translation`).

Rigidity is what makes a landscape summarisable: a host can reason about one scalar rather than about a vector of positions. The landscape carries the shared term once and the per-crossover offsets separately.

**Invariant (Landscape uncertainty has exactly two sources)** · `inv:guarantee:two-source-uncertainty`

All uncertainty in the landscape is attributable, per crossover, to the risk estimate's variance or to the challenge posterior's, and to nothing else (`thm:landscape:crossover-covariance`).

The exhaustiveness is the useful part: a host reading a wide crossover interval can always decompose it into the two, and therefore always knows whether more labels or more challenge evidence is what would narrow it (`def:fragility:definition`). The landscape's shared risk uncertainty and per-crossover sensitivities carry the two contributions.

**Invariant (Dominance is reported and never enforced)** · `inv:guarantee:dominance`

Dominance is reported as a probability in closed form, and the landscape's shape does not depend on it: the crossover vector keeps one entry per adjacent action pair whether or not an action is dominated (`thm:landscape:dominance`).

Reporting rather than pruning is what keeps the landscape's structure fixed across calls, so a host comparing two assessments compares like with like. Any suppression of a dominated action is a display decision taken downstream and has no counterpart here. The landscape reports one domination probability per fixed-shape regime and leaves suppression to the renderer.

**Invariant (Reported uncertainty is never narrower than the evidence)** · `inv:guarantee:honest-uncertainty`

The posterior reflects the evidence as it was labelled. The guarantee is conditional on the host's side of the contract: label correctness is the host's responsibility, and the Core trusts every label it is given (`thm:gaussian:two-level-guarantee`).

The condition is not a disclaimer but the property's actual content. A system that corrected for suspected label error would be reporting a narrower uncertainty than its evidence supports, which is the failure this property exists to forbid; the cost is that a corrupted label stream produces confident wrong answers, and the limitations below say so.

**Boundary and exploration properties.** Each states what the host may discover or vary without changing the measured structure.

**Invariant (Encoding consequences are visible to the host)** · `inv:guarantee:encoding-transparency`

Each Sentinel's coordinate semantics — what its dimensions mean, what a shared prefix asserts, and what the Core has therefore assumed about them — are recorded at registration and reported back (`schema:registry:sentinel-record`), so that a host can discover the consequences of its encoding choices without reading the Core's internals (`chap:spec:output-structures`).

Transparency here is the counterweight to the encoding contract's demands: the document asks a host to supply a hierarchical key space and owes it, in exchange, a legible account of what was made of that space.

**Invariant (Regime structure is independent of posture)** · `inv:guarantee:posture-independence`

The landscape is computed without reference to the host's posture. Posture is a cursor a host moves over a completed landscape, so two hosts reading one assessment at two postures read the same structure and differ only in where they stand on it (`def:channel:posture`).

The independence is what allows a single derivation to serve every channel and every reviewer at once, and it is why a landscape can be stored and re-read later at a posture nobody had chosen when it was computed. Derivation takes no posture, and the two posture-indexed utilities read the completed landscape afterward.

**Invariant (The system exposes an exploration signal)** · `inv:guarantee:exploration`

Exploration is a joint property of two halves. The Core supplies the estimate's uncertainty, which says where evidence is thin (`def:guidance:risk-informative`); the derivation supplies fragility, which says where that thinness would change a decision (`def:fragility:definition`). Together they identify the assessments worth learning from.

Either half alone is much weaker. Uncertainty without fragility ranks cases the system is unsure about, most of which do not matter; fragility without uncertainty ranks cases near a boundary, most of which are near it for good reason. Core guidance ranks risk-informative pending requests, and the landscape's fragility utility reports decision sensitivity at a caller's posture for host-side composition.

**The seven stated elsewhere.** Each of these is the whole content of an environment at the chapter that owns it, and repeating the statement here would create a second copy able to disagree with the first. Four keep this register's own area because the guarantee is what they are: the feed-forward invariant (`inv:guarantee:feed-forward`), non-blocking assessment (`inv:guarantee:non-blocking`), bounded staleness (`inv:guarantee:staleness`) and lifecycle publication (`inv:guarantee:lifecycle-publication`). Three take their own chapter's area because the environment there is the statement and the guarantee is its force: the two dimension-map properties (`inv:dimension:covering`), (`inv:dimension:contiguity`), and the landscape's freedom from presentation (`inv:landscape:presentation-free`). A reader wanting the whole picture reads the twenty-two above and these seven at their own sites — twenty-nine properties in all.

Three further properties are stated of the optional rendering layer and are scoped to it rather than to the primary output (`app:spec:resonance-rendering`); they constrain a display and bind nothing a host decides on.

**Figure (Information flow)** · `fig:architecture:information-flow`

```
Sentinels ──► Core ──► Host
                ▲         │
                │         ├──► derivation ──► landscape ──► Host
                │         │        ▲
   Host ──labels──┘       │        │
                          │   Companion ──posterior──┘
                          │        ▲
                          │        │
                          └──challenge results──► Companion
```

Every arrow is unidirectional. The Core never reads from the derivation or from the Companion. The derivation reads from both and writes to neither. The Companion reads challenge outcomes from the host and writes to neither of the others. The only cycle in the diagram passes through the host, which is the feed-forward invariant drawn rather than stated.

The figure is a summary and binds nothing of its own; the complete inventory of what crosses each boundary, in which direction, and what never crosses at all, is the interface appendix's (`tab:boundary:verification`).

**The limitations.** Forty-two of them, each naming something the system cannot do or cannot see, with the cause that makes it so and whatever bounds the damage. They are graded four ways — structural, managed under stated conditions, empirical or default-valued, and the host's responsibility — and the grade is part of the statement: a structural limitation is one no mechanism in this design removes, and the rest are bounded and instrumented.

Three deserve a reader's attention before the rest, because each names an exposure that a naive design leaves entirely silent rather than merely unsolved. Symmetric decay forgives an adversary on the same schedule it forgives a stale reputation (`cav:limitation:laundering`). The challenge population is chosen by the estimate the challenge evidence feeds (`cav:limitation:challenge-policy-loop`). And the honesty of reported uncertainty is measurable only by a coverage diagnostic that must actually be computed (`alg:monitoring:empirical-coverage`). A design without those three named would not be wrong; it would be quiet.

**Caveat (Outcomes are observed on one side of the action only)** · `cav:limitation:valence`

The host acts on the system's estimate and then observes only what happened under that action, so the counterfactual is never labelled. The system learns from a censored bandit (`prin:valence:censored-asymmetry`). Exploration, the investigation criterion and the measurement-only anchor bound the damage and none of them removes it (`disc:valence:mitigation-layers`).

**Caveat (The Ledger absorbs the consequences of the actions it informs)** · `cav:limitation:ledger-contamination`

Spatial outcome memory is written from labels produced under decisions the same memory shaped, so a cell's history is partly a record of how it has been treated (`alg:valence:contamination-loop`). Time-indexed decay and the measurement-only anchor bound the loop under the stated conditions (`def:ledger:time-decay`).

**Caveat (The same loop reaches spatially-enabled axes)** · `cav:limitation:axis-contamination`

An outcome axis registered with spatial features carries per-cell state written from the same censored labels, so the contamination is not confined to the risk Ledger (`disc:valence:axis-pathway`). The axis pathway is graded rather than binary, which changes the loop's shape and not its existence, and the same mitigations apply.

**Caveat (The host's policy shapes the population the Core learns from)** · `cav:limitation:host-loop`

Restriction reduces the label flow from the entities restricted, so the population the models train on is the population the host has chosen to keep serving (`scenario:monitoring:atrophy`). Time-indexed decay and ancestor routing bound the effect; three scenarios in the monitoring chapter work through what it looks like when they do not.

**Caveat (Interaction features converge slowly and unevenly)** · `cav:limitation:interaction-convergence`

An interaction feature is informative only where its two constituents co-occur, and co-occurrence rates vary by orders of magnitude across a template set, so interactions arrive in a cascade rather than together (`tab:feature:interaction-convergence`). A deployment reading its convergence as a single number will misread it.

**Caveat (The challenge estimate converges only as labels arrive)** · `cav:limitation:challenge-convergence`

The Companion's posterior narrows as the inverse of its contributing label count, and nothing accelerates that (`bound:companion:convergence`). The override and the injection interface let a host supply prior evidence, which shortens the wait without changing the rate.

**Caveat (Contributing labels are thin in realistic deployments)** · `cav:limitation:challenge-thin`

Only challenged requests whose outcome was observed contribute, and on a typical deployment that is a small fraction of one per cent of traffic (`data:companion:contributing-rate`). The posterior expresses the resulting cold start honestly, as width, which is a correct report of a real shortage rather than a fix for it.

**Caveat (The challenge population is chosen by the policy it informs)** · `cav:limitation:challenge-policy-loop`

Challenge effectiveness is a property of the challenged population, and that population is selected by the estimate the effectiveness figure feeds (`scenario:monitoring:starvation`). The loop is slow and damped rather than divergent, and an override inherits the caveat rather than escaping it.

**Caveat (Narrowing the action space narrows the evidence)** · `cav:limitation:challenge-narrow`

A channel that declares fewer actions produces fewer distinguishable outcomes, and the challenge regime in particular can be narrowed until it is never chosen (`prop:channel:challenge-width`). The dominance probability is reported per request so that the narrowing is visible rather than silent (`cav:channel:action-space-sizing`).

**Caveat (Symmetric decay is exploitable in the forgiving direction)** · `cav:limitation:laundering`

The Ledger forgets an adverse history at the same rate it forgets a stale good one, and an entity that generates benign volume can accelerate its own forgiveness (`disc:ledger:laundering`). The exposure is accepted deliberately: an asymmetric decay would make stale reputation permanent, which is the worse failure. The Ledger is one of several detection pathways and the others still fire.

**Caveat (The Ledger is structurally uninformative for low-traffic cells)** · `cav:limitation:ledger-low-traffic`

Time-indexed decay attenuates a cell's accumulated evidence below materiality whenever labels arrive more slowly than the decay removes them, which is the normal condition for most cells (`data:ledger:attenuation`). Detection in those cells runs through the immediate-convergence features instead (`tab:ledger:low-maturity-detection`).

**Caveat (A fresh entry carries no evidence and inherits none)** · `cav:limitation:fresh-entry`

A newly created Ledger entry starts from the prior on every layer, and nothing is transferred from a parent or a neighbour (`alg:ledger:entry-creation`). The interval before it says anything is long by the attenuation analysis, and it is covered by the per-Sentinel features that converge immediately.

**Caveat (The anchor floor bounds how far the blend can be pulled)** · `cav:limitation:anchor-floor`

The anchor retains a small residual influence at every blend weight, arising from the leverage differential and Schur inflation rather than from a declared minimum (`dec:risk:anchor-floor`). It provides contamination resistance, and because it is emergent its exact magnitude is a property of the deployment rather than a guarantee.

**Caveat (The models are linear in the feature vector)** · `cav:limitation:linear`

The Core's models are linear, so any structure that is not expressible as a weighted sum of features must be supplied as a feature (`mot:feature:interaction-overview`). Interaction templates give quadratic reach and no more, and a genuinely non-linear relationship is approximated rather than learned.

**Caveat (The wildcard template's feature count grows quadratically)** · `cav:limitation:interaction-type-three`

A cross-Sentinel wildcard template creates one feature per unordered pair, so its count grows as the square of the Sentinel count and its convergence takes months (`def:feature:template-wildcard`). It is off by default, which is the mitigation: a host that enables it should know it has bought a long wait.

**Caveat (Cross-Sentinel structure is learned only through interactions)** · `cav:limitation:cross-sentinel-gap`

Nothing but an interaction feature can express a pattern that is unremarkable in each Sentinel alone and meaningful across two, so joint concealment is undetectable for as long as the relevant interaction is unconverged (`tab:detection:convergence-window`). The anchor supplies partial coverage during the window.

**Caveat (The posterior can become ill-conditioned)** · `cav:limitation:conditioning`

Forgetting erodes precision in directions the data stops exercising, and a posterior whose condition number grows without bound eventually produces unreliable updates (`alg:gaussian:condition-adaptive-recompute`). The prior replenishment floor and adaptive recomputation bound it (`req:gaussian:prior-replenishment-floor`), under those stated conditions and not otherwise.

**Caveat (The importance-weight ceiling binds at extreme class rates)** · `cav:limitation:ceiling`

The static ceiling on balancing weights binds once the positive-valence rate falls below roughly half a per cent, after which rare positives are under- weighted relative to the balance the weighting exists to restore (`def:weighting:balancing-weights`). The ceiling is configurable. Health reports the binding fraction and positive gradient balance independently for the operational and eligible sister streams, accumulated from the weights each label actually used.

**Caveat (The compression scale assumes a stationary target)** · `cav:limitation:kappa-nonstationary`

An axis's compression scale adapts to the distribution of values it has seen, so the target a model was trained against shifts retroactively when the distribution moves (`def:axis:adaptive-compression`). Forgetting makes the shift self-correcting over time, and a large jump is a quantity worth watching rather than an error.

**Caveat (Axis training inherits the eligibility confound)** · `cav:limitation:axis-confounding`

An axis trained on eligible labels only sees values from the population the host's policy admitted, so the axis answers a question conditioned on that policy (`def:axis:training-target`). The eligibility mode is per axis precisely so that the choice is deliberate; neither setting removes the confound, they choose which one to carry.

**Caveat (Cross-axis structure is emergent and unmodelled)** · `cav:limitation:cross-axis`

Where two axes are registered, each model can predict the other's values only through their shared features, and the quality of that prediction depends on how often the two are co-reported (`prop:axis:cross-axis-prediction`). Nothing models the relationship directly and nothing reports how well it is holding.

**Caveat (Each axis costs features and labels)** · `cav:limitation:axis-cost`

Registering an axis adds features to every model that carries axis features and creates a model of its own, so the cost is quadratic in the feature dimension and the convergence cost is a fresh label budget (`tab:registry:axis-scaling`). The spatial feature policy is the lever, and the per-axis models parallelise.

**Caveat (Chain depth is bounded and visible in the features)** · `cav:limitation:chain-depth`

The statistical properties of every chain-derived view vary with the chain's length, so a value from a shallow chain and the same value from a deep one do not mean the same thing (`tab:extraction:chain-z-scores`). The chain-length feature is supplied alongside so that the model can condition on it (`tab:extraction:chain-structure`), which disambiguates rather than removes the dependence.

**Caveat (The maximum view carries an order-statistics bias)** · `cav:limitation:chain-maximum`

The expected maximum of independent draws grows with the number of draws, so a deep chain produces a larger maximum than a shallow one on identical data (`rem:extraction:order-statistic-bias`). The chain-length feature lets the model absorb the growth, and an explicit correction is available and not applied by default.

**Caveat (Nothing between reports is observable)** · `cav:limitation:inter-report`

The Core sees batch reports, so anything that begins and ends between two reports leaves no trace in any feature (`def:extraction:report-staleness`). Report staleness and the batch context features tell the model how much time a report is standing for, which is what makes the trade legible rather than what closes the gap.

**Caveat (The signal cache evicts, and eviction loses evidence)** · `cav:limitation:signal-cache`

Host signals are held in a bounded cache, and an entry evicted before its label arrives is simply absent from the reconstructed vector (`tab:keyspace:signal-cache`). Eviction is least-recently-used and the cache is decoupled from competitive standing, so an eviction costs a signal rather than an entity's history.

**Caveat (Competitive churn resets measurement state)** · `cav:limitation:competitive-churn`

An entity entering or leaving a competitive set triggers extension or marginalisation, and the measurement state of the affected range restarts its convergence (`alg:keyspace:cell-exit`). The operations are low-rank and infrequent — a few times a day on a typical deployment — which bounds the cost without eliminating the restart.

**Caveat (A bad encoding degrades everything and announces nothing)** · `cav:limitation:silent-encoding`

A key space supplied without real hierarchical structure produces features that carry no information, and every downstream layer degrades gracefully enough that nothing fails visibly (`rem:encoding:downstream-fallback`). The designated early warning is the association reading (`def:monitoring:slot-association`), the mean absolute correlation between the block's coordinates and the outcome read against the correlation a coordinate telling the outcome nothing earns. It is computed and reported per Sentinel and per identity dimension, and a block that has fallen to the null floor is a block whose coordinates are telling the outcome nothing over the window the reading remembers.

The role sits there rather than on the weight-mass reading (`def:monitoring:encoding-effectiveness`) because that reading moves the wrong way. Standardised-space ridge weights on coordinates that predict nothing settle at noise scale rather than at zero, and a model that has fit its stream has no residual left with which to shrink them, so a hash-fed Sentinel's mean slot weight *rises* towards a working one's as the model converges — measured, between $0.76$ and $1.05$ of a perfect control's figure through nine thousand six hundred labels. An indicator that climbs while the thing it warns about is getting worse is not a weak warning; it is the opposite of one, and a deployment reading it as an early warning would be reassured by exactly the convergence that should have alarmed it.

The warning is still a prompt to look rather than a diagnosis. An association at the floor says the model's stream carries no relationship between this block's coordinates and the outcome, and does not say whether the encoding or the phenomenon is the reason. What it now supports that it did not before is the second step: whether the block can be retired is the contribution reading (`def:monitoring:slot-contribution`), which is the removal cost itself.

**Caveat (The Core sees effects, never Sentinel-internal operations)** · `cav:limitation:abstraction`

The Core observes cells appearing and disappearing in reported sets and never the operations inside a Sentinel that produced them, so it can trace causation and cannot correspond operations (`alg:runtime:cell-set-maintenance`). The contour snapshot supplies diagnostic context for a human, not a correspondence for the model.

**Caveat (A Sentinel added late standardises against a moving target)** · `cav:limitation:late-standardisation`

A newly registered Sentinel's features enter a running standardisation whose statistics reflect the features it does not have, so its own values are mis-scaled until its statistics accumulate (`alg:standardisation:sentinel-bootstrap`). The per-Sentinel bootstrap shortens the interval substantially and does not remove it.

**Caveat (Outputs before the first refit are uncalibrated)** · `cav:limitation:pre-calibration`

The calibration parameter is unfitted until enough labelled outcomes have accumulated, so probabilities emitted before the first refit are ordered correctly and scaled arbitrarily (`rem:warmup:pre-calibration`). Pre-seeding and the reported calibration maturity are what let a host tell the two intervals apart.

**Caveat (A label arriving after eviction is lost)** · `cav:limitation:buffer-eviction`

An outcome reported for an assessment already evicted from the pending buffer cannot be applied to any model: the call fails and the evidence is gone (`req:runtime:buffer-capacity`). Sizing capacity and expiry to the deployment's actual label latency is the whole of the mitigation.

**Caveat (Discrimination metrics lag the change they measure)** · `cav:limitation:auc-lag`

Every discrimination figure is computed over a buffer spanning roughly ten days at the reference label rate, so a change in the models' ordering quality is visible only after it has been true for a while (`alg:monitoring:auc`). The recent window is the faster and noisier signal, and reading it against the aggregate is what the instability flag is for.

**Caveat (A regime change can mimic feature-stable drift)** · `cav:limitation:mimic-regime`

Outcomes moving while features hold steady is the signature of corrupted labels and also the signature of a legitimate shift in what the same behaviour means (`def:monitoring:feature-stable-drift`). The flag reports the conjunction and cannot distinguish the two causes; distinguishing them is the host's, with context the Core does not have.

**Caveat (Patient corruption is not detectable by these means)** · `cav:limitation:patient-corruption`

Label corruption held below the per-entity flagging threshold and spread across many entities passes every integrity diagnostic in this document (`cav:monitoring:integrity-scope`). Its impact is bounded by the same forgetting that bounds everything else, which limits persistence rather than providing detection.

**Caveat (Investigation requests presuppose a pipeline the Core lacks)** · `cav:limitation:investigation-pipeline`

The Core trusts the ground-truth flag unconditionally, and has no means to audit the pipeline that sets it (`conv:eligibility:ground-truth`). A compromised investigation pipeline is therefore indistinguishable from a working one, and its integrity is the host's responsibility in the full sense.

**Caveat (Severity discrimination is cold until severity labels arrive)** · `cav:limitation:severity-cold`

The binary risk target carries no severity information, so a deployment has no severity discrimination at all until an axis trained on severity has converged (`rem:host:severity-discrimination`). Registering that axis early is the mitigation, and it costs the axis's own label budget.

**Caveat (Severity enters only through the target, not the features)** · `cav:limitation:severity-path`

Because the risk target is binary, severity reaches the risk models only insofar as severe events change the features, and never as a magnitude (`def:risk:target`). The design commits to this: a dedicated outcome axis is the route by which severity becomes a first-class quantity.

**Caveat (The landscape is only as good as the declared costs)** · `cav:limitation:reward-sensitivity`

Crossover positions move substantially under small changes in the ratios between declared costs, so a landscape is a statement about the host's declared preferences rather than an objective fact (`data:channel:reward-sensitivity`). A surprising crossover position is a reason to check the declaration first (`rem:channel:verify-costs`).

**Caveat (Fragility assumes a Gaussian crossover distribution)** · `cav:limitation:fragility-gaussian`

The default flip-probability computation uses the stated Gaussian marginal approximation, which is an approximation wherever the challenge posterior is far from symmetric (`def:fragility:definition`). An evaluation exact in the challenge parameter is available for conjugate posteriors and is not the default.

**Caveat (The divergence attributes to no single cause)** · `cav:limitation:divergence-attribution`

External restriction signals are not in the feature vector, so the measured divergence between what was predicted and what occurred cannot be decomposed per feature with any reliability (`def:detection:divergence-scope`). The aggregate direction is trustworthy; the per-feature attribution is approximate and should be read as a hint.

**Caveat (Hibernation preserves self-structure only)** · `cav:limitation:hibernation`

Hibernating a Sentinel retains the parameters that concern it alone and discards every cross-term it participated in, so waking it relearns its interactions from nothing (`alg:registry:hibernation`). The archive copies the departing entity's own positions out of every full-dimension model before the marginalisation removes them, and the restore writes that block back over the positions an extension has just created, where the couplings the extension zeroed stay zero because the record carries none to overwrite them with. A deployment that asks for the hibernating disposition therefore gets the entity's own mean, precision and covariance back, aged on both clocks, and falls back to the prior wherever the archive has expired, the arrangement has changed, or a model's block has aged past the replenishment floor (`req:gaussian:prior-replenishment-floor`). What it does not get is the couplings, which return at exactly zero however strong they were, nor the outcome memory, which ends with the entity that fed it (`rem:registry:axis-hibernation`).
