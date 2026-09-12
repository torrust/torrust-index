## Part (The Core Models) · `part:spec:core-models`

Part III defines what the Core learns. Three Bayesian linear models estimate risk from the feature vector and one model per declared outcome axis predicts whatever the host asked to be predicted, all of them updated by a single observation procedure; beneath them a per-Sentinel outcome memory accumulates what happened where. Nothing here references actions, channels, posture, or cost: the Part ends at the risk basis, and what a host does with a risk basis is the next Part's subject.

### Chapter (The Core Risk Models) · `chap:spec:core-risk-models`

The chapter is the densest in the document. Three models share one prior and one update procedure and differ in what they are allowed to learn from; two of them are combined into an effective estimate whose weight is a ratio of uncertainties; the estimate is mapped to a probability by a calibration fitted against outcomes. Around that core sit the three policies that decide which observation reaches which model and with what force — the eligibility table, the importance weights, and the leverage bound — and the forgetting rates that decide how long any of it is remembered.

**Definition (The model triple)** · `def:risk:model-triple`

Three Bayesian linear models estimate risk from the feature vector $\phi$. The operational model trains on every labelled outcome, whatever the host did; the sister model trains only on unconfounded outcomes; the counterfactual anchor model trains on the same unconfounded outcomes as the sister but over a fixed low-dimensional projection.

$$\hat{\rho}_\text{opr}(\phi) = \phi^\top \mu_\text{opr}, \qquad \hat{\rho}_\text{inh}(\phi) = \phi^\top \mu_\text{inh}, \qquad \hat{\rho}_\text{anc}(\tilde{\phi}) = \tilde{\phi}^\top \mu_\text{anc}$$

All three are initialised at $\mu = \mathbf{0}$, $B = \lambda_\text{prior} I$, $\Sigma = \lambda_\text{prior}^{-1} I$, at one shared prior precision.

The triple is the direct consequence of the valence asymmetry (`assum:constraint:valence-asymmetry`). The operational model sees everything and therefore learns a confounded picture: its estimate is the joint effect of inherent risk and of the host's own intervention. The sister model sees only what was left alone and estimates inherent risk, but starves whenever the host restricts. The anchor converges some forty times faster on fifteen parameters and covers the sister's starvation coarsely rather than well. Each is individually insufficient and the three are jointly what the decomposition principle asks for (`prin:principle:multi-level-decomposition`).

The sister's target deserves one qualification. Selection into the unconfounded population is itself a function of the system's own estimate, so the sister estimates the adverse rate conditional on having been allowed, which equals inherent risk only where eligible labels actually reach. In regions the host has always restricted, the sister's estimate is prior extrapolation and not measurement, and host investigation (`conv:eligibility:ground-truth`) is the only mechanism that supplies coverage there.

**Definition (The anchor model)** · `def:risk:anchor-model`

The anchor operates on a fixed fifteen-dimensional vector $\tilde{\phi}$ drawn from the bias, the aggregate block, the cross-dimension identity features, and two quantities computed inline. It is never extended and never marginalised, whatever Sentinel, outcome axis, or identity dimension lifecycle events occur; the projection that feeds it absorbs every such change (`def:dimension:anchor-projection`).

| Index | Feature | Block |
| --- | --- | --- |
| 0 | Bias | Bias |
| 1 | Maximum cell-level maximum z | Aggregate |
| 2 | Standard deviation of cell-level maximum z | Aggregate |
| 3 | Cross-axis product | Aggregate |
| 4 | Axis breadth | Aggregate |
| 5 | Coverage | Aggregate |
| 6 | Maximum cumulative sum | Aggregate |
| 7 | Maximum suspicion | Cross-dimension |
| 8 | Maximum adverse rate | Cross-dimension |
| 9 | Any competitive cell present | Cross-dimension |
| 10 | Maximum compressed valence magnitude | Cross-dimension |
| 11 | Maximum raw valence magnitude | Cross-dimension |
| 12 | Maximum volatility | Cross-dimension |
| 13 | Maximum raw z across Sentinels and axes | Computed |
| 14 | Log count of reporting Sentinels | Computed |

Positions 0 to 12 are gathered from $\phi$; positions 7 to 12 evaluate to zero when no identity dimension is registered, which is how identity lifecycle change is absorbed without touching the model. Positions 13 and 14 are computed inline rather than occupying two further aggregate indices, which is what keeps the aggregate block at its declared width (`tab:feature:aggregate-block`). The cross-dimension aggregation is a maximum throughout, so the anchor fires when any identity view is alarming — the principle the aggregate block already applies across Sentinels.

The projection gathers all fifteen positions exactly as tabulated: positions 0 to 12 through the dimension map, then the computed maximum raw z-score and the logarithm of the reporting-Sentinel count at positions 13 and 14.

**Proposition (The measurement-only anchor feature)** · `prop:risk:anchor-measurement-only`

Anchor position 13 — the maximum raw z-score across every Sentinel and axis — is a pure measurement signal that reaches the model without passing through any outcome memory. It is therefore evidence of a kind no Ledger feature and no identity outcome feature can supply: it reports what the measurement layer sees now, and nothing about what was recorded earlier.

The property is what makes the anchor useful under contamination. When stale outcome memory inflates the identity and cell-level features (`alg:valence:contamination-loop`), position 13 does not move with them, so the anchor can distinguish an alarming measurement layer from a silent one that sits behind a remembered reputation. Position 14, the log count of reporting Sentinels, carries measurement depth alongside it, letting the anchor discount its own estimate when little of the surface is online.

The property is a statement about provenance, not about magnitude. One feature of fifteen cannot outvote a contaminated majority, and it is not asked to: what it supplies is a persistent discrepancy that a drift monitor can accumulate even while the contaminated estimate is the one being reported.

**Definition (The subspace-restricted blend)** · `def:risk:subspace-blend`

The sister and anchor are blended into an effective estimate whose weight is set by their relative uncertainty over the features they share. Let $\tilde{\phi}_g$ be the gathered anchor coordinates — positions 0 to 12, the ones that correspond to an index of $\phi$ — and let $\Sigma_{\text{inh},S_gS_g}$ be the sister covariance restricted to them:

$$\sigma^2_{\text{inh},S_g}(\phi) = \tilde{\phi}_g^\top \Sigma_{\text{inh},S_gS_g} \, \tilde{\phi}_g, \qquad w(\phi) = \left(1 - \frac{\sigma^2_\text{anc}(\tilde{\phi})}{\sigma^2_{\text{inh},S_g}(\phi) + \varepsilon}\right)^+$$

$$\hat{\rho}_\text{eff}(\phi) = (1 - w) \cdot \hat{\rho}_\text{inh}(\phi) + w \cdot \hat{\rho}_\text{anc}(\tilde{\phi})$$

The restriction to the shared subspace is the point of the construction. The sister's uncertainty about Sentinel-specific directions has nothing to do with whether the anchor is the better estimator over the features both models hold, and a weight computed from the sister's full uncertainty would activate the anchor every time a new Sentinel arrived. The two computed anchor positions are excluded from the sister-side covariance because they correspond to no index of $\phi$ and could enter only through an explicit linearisation.

All three uncertainties are read from the time-corrected covariance (`def:runtime:time-correction`), so the weight moves with elapsed time as well as with labels. The point estimates do not.

**Remark (How the blend persists)** · `rem:risk:blend-mechanisms`

The weight does not decay to zero. Two structural mechanisms keep the sister's shared-subspace uncertainty above the anchor's in steady state.

The first is the leverage bound differential. The sister's higher dimension makes its leverage larger, so the bound (`prop:update:leverage-bound`) caps its effective weight more often and more severely than the anchor's, and its precision over the shared subspace accumulates more slowly for that reason alone.

The second is marginal covariance inflation. The sister's marginal covariance over the shared features exceeds the inverse of the corresponding precision submatrix, because correlations between anchor features and Sentinel-specific features inflate the margin through the Schur complement structure (`def:gaussian:schur-complement`).

Both persist. The weight converges to a small positive residual — two to ten per cent, depending on how correlated the deployment's features are — so the sister contributes the great majority of the estimate and the anchor keeps a floor under it. Early in convergence the trajectory runs the other way: both models sit at the shared prior, the anchor's quadratic form spans two extra computed directions, and the weight starts near zero, rises steeply as the anchor's fifteen dimensions concentrate, then declines toward its residual as the sister catches up. A Sentinel deregistration can push it transiently to zero again, when the marginalisation briefly raises the sister's shared-subspace precision above the anchor's.

**Proposition (The blended uncertainty)** · `prop:risk:blend-variance`

The blended uncertainty is the variance of the two-component mixture the weight defines:

$$\sigma^2_\text{eff}(\phi) = (1-w)\sigma^2_\text{inh}(\phi) + w\sigma^2_\text{anc}(\tilde{\phi}) + w(1-w)\bigl(\hat{\rho}_\text{inh}(\phi) - \hat{\rho}_\text{anc}(\tilde{\phi})\bigr)^2$$

The first two terms carry within-model uncertainty and the third carries between-model uncertainty: the extra variance from not knowing which of the two is right. The third term is what makes anchor–sister disagreement visible in the reported uncertainty rather than silently averaged away, and it is the term that vanishes at both extremes of the weight.

Two qualifications hold the formula to what it claims. The weight is a heuristic precision ratio and not a posterior model probability, so the mixture-variance form is the correct bookkeeping given that weight rather than a claim of Bayesian model averaging. And the first term uses the sister's *full* uncertainty, not its shared-subspace restriction: the restriction applies to the weight alone, because uncertainty about Sentinel-specific directions is real uncertainty about the estimate even when it says nothing about which model to prefer.

**Decision (Why the floor is emergent rather than declared)** · `dec:risk:anchor-floor`

The persistent residual weight is a consequence of the mathematics, not a designed feature, and the question is whether to codify it as an explicit floor $w \geq w_\text{floor}$. It is not codified, and the reasoning is recorded here because the behaviour it protects is easy to lose by accident.

The residual supplies three properties. It keeps a measurement-only contribution in every assessment, too small to override a contaminated estimate but large enough to leave a persistent discrepancy for a drift monitor (`prop:risk:anchor-measurement-only`). It keeps the between-model variance term alive, so a regime change that invalidates the sister's weights shows up as disagreement rather than as unwarranted confidence. And it responds to lifecycle events without being told about them: a new Sentinel raises the sister's leverage, the bound fires harder, the weight rises, and the anchor covers the re-convergence with no lifecycle-aware scheduling anywhere.

An explicit floor would replicate all three and would add a parameter to a regime that already behaves correctly. The emergent floor is preferred because its magnitude scales with the deployment's own feature structure, which a configured constant cannot do. What the deployment gives up is the guarantee that the floor is there at all (`cav:limitation:anchor-floor`).

**Definition (The risk probability)** · `def:risk:probability`

The effective estimate is mapped to a probability through a sigmoid whose steepness is the calibration parameter:

$$\hat{p} = \sigma\!\left(\frac{\hat{\rho}_\text{eff}}{\kappa_\text{eff}}\right), \qquad \sigma_{\hat{p}} = \hat{p}(1-\hat{p}) \cdot \frac{\sigma_\text{eff}}{\kappa_\text{eff}}$$

where $\kappa_\text{eff}$ is the effective calibration parameter (`def:platt:regimes`), $\sigma_\text{eff}$ the square root of the blended variance, and $\sigma(x) = (1 + e^{-x})^{-1}$.

The uncertainty on the probability follows from the sigmoid's derivative identity and is a first-order propagation, not an exact transformation of the posterior. The $\sigma_\text{eff}$ it propagates is evidence-only along the request's own direction, so the widening is already in the figure the host reads rather than left for a consumer to apply (`dec:risk:evidence-only-uncertainty`). It is reported because a probability without one is not usable by a decision layer that must weigh a confident estimate differently from a tentative one, and the propagation is accurate wherever the estimate is not close to saturation.

**Decision (The verdict's uncertainty is evidence-only along its own direction, and the share it borrowed rides beside it)** · `dec:risk:evidence-only-uncertainty`

The uncertainty reported with a verdict is the uncertainty the evidence supports along that request's own features, and the share of the precision behind it that the models' floors were holding up instead travels beside it on the basis (`schema:risk:basis`). Degradation is a source of uncertainty and it has an angle: a model propped up by its floors is not propped up equally in every direction, and a request asks about one. An aggregate says the model is weak somewhere; only a reading taken along the request's own direction can say whether the weakness is in the way of this answer.

The reading the tempering divides by is the share the posterior already tracks. The precision matrix carries the two floors' contributions as mass — identity-shaped from the spectral floor, coordinate-shaped from the replenishment clamp, both decayed with the matrix they are part of — so along any direction $\phi$ the share of posterior precision the prior rather than the evidence is holding up is $s(\phi) = (f\lVert\phi\rVert^2 + \sum_j c_j \phi_j^2) / (\phi^\mathsf{T} B \phi)$ (`dec:posterior:spectral-floor`). It is exact about the split, because the two masses are the amounts the floors added to $B$ in those two shapes rather than estimates of them, and it is an approximation of the full posterior, because a Rayleigh quotient reads one direction and says nothing about the subspace that direction lies in. Both halves of that are the mass record's and are not re-argued here.

The tempering follows by subtraction rather than by a model. Along $\phi$ the posterior precision is the evidence's plus the floors', so the evidence's alone is $(1-s)$ times the posterior's, and the variance that goes with it is the posterior variance divided by $(1-s)$. Nothing is fitted, and no constant is chosen.

Where the correction is exact and where it is a bound is stated rather than left to be discovered. Let $D = fI + \operatorname{diag}(c)$ be the floors' own contribution, so the exact evidence-only variance along $\phi$ is $\phi^\mathsf{T}(B-D)^{-1}\phi$. Writing $z = D^{1/2}\Sigma\phi$ and $M = D^{1/2}\Sigma D^{1/2}$, the reported value expands as $v + z^\mathsf{T}z + (z^\mathsf{T}z)^2/v + \cdots$ and the exact one as $v + z^\mathsf{T}z + z^\mathsf{T}Mz + \cdots$, and the two agree in their first two terms. From there Cauchy–Schwarz in the inner product $\Sigma$ defines gives $(z^\mathsf{T}z)^2 \leq v \cdot z^\mathsf{T}Mz$, and the log-convexity of the moments $z^\mathsf{T}M^k z$ carries that to every later term by induction. So the reported variance is a lower bound on the exact evidence-only variance, with equality wherever the floors' share is even across the coordinates the direction draws its variance from — one coordinate, or a model whose floors bear on every coordinate alike. The correction therefore understates the widening where the bound is loose, which is the direction that under-reports uncertainty and is named here rather than buried: it is the price of a scalar correction along one direction, and the alternative is a matrix inverse per request. The mass the share is computed from errs the other way, crediting a late-arriving coordinate with identity-shaped mass never applied to it (`dec:posterior:spectral-floor`), and the two errors are recorded as opposing rather than as cancelling, because nothing bounds their ratio.

The share is computed from the published covariance, and that is an identity rather than a substitution. The precision matrix is not published, on an argument about bulk that these masses do not engage (`dec:retention:precision-excluded`). But with $\psi = \Sigma\phi$ the denominator collapses: $\psi^\mathsf{T}B\psi = \phi^\mathsf{T}\Sigma B \Sigma \phi = \phi^\mathsf{T}\Sigma\phi$, which is the variance the blend has already computed. So the share is read along $\psi$, and $\psi$ is not an arbitrary stand-in for $\phi$ — it is the direction whose precision holds that variance up, which is the direction the tempering is about. The two coincide wherever $\phi$ is an eigenvector of $B$, because $\Sigma\phi$ is then parallel to $\phi$ and the share is scale-free. What the identity rests on is that the pair the model holds are actually inverses of one another, which is a property the model maintains rather than assumes: a rebuild is adopted on the drift measured after it and a shifted answer is refused by that measurement (`dec:posterior:measured-adoption`).

The blend's share is the same weights in variance space. The blended variance is a three-term mixture (`prop:risk:blend-variance`), and the borrowed part of it is the borrowed part of each component variance weighted as that variance was weighted: $s_\text{eff} = \bigl[(1-w)\,s_\text{inh}\,\sigma^2_\text{inh} + w\,s_\text{anc}\,\sigma^2_\text{anc}\bigr] / \sigma^2_\text{eff}$. Defining it in variance space rather than by averaging the two shares is what makes the tempering apply consistently to the quantity it is applied to, and it settles the mixture's third term without a separate rule: the disagreement between two models is uncertainty their evidence produced, no part of it was lent by a floor, and so it enters the denominator alone and dilutes the share. The subspace restriction the weight is computed over changes nothing here (`def:risk:subspace-blend`): the restriction decides which model to prefer, and each model's share is read over the coordinates that model actually has.

Each model's share is read from its raw covariance, and the staleness correction commutes with the tempering for that reason. Elapsed time inflates a variance; it moves no mass into or out of a precision matrix. The correction therefore multiplies the evidence-only variance exactly as it multiplies the posterior one (`def:runtime:time-correction`), so applying the tempering before or after it gives the same number — which is what lets the ordinary path and the fourth numeric checkpoint's fallback carry one operation in one order rather than two orders that happen to agree.

The widening saturates at the prior-only variance, and the cap is derived rather than chosen. As the share approaches one the quotient diverges, and what must be reported there is not a large number but a statement: the prior is all that is known along this direction. That statement takes the time-corrected prior variance the fourth numeric checkpoint substitutes when the blend's own variance is unusable (`def:axis:prior-only-prediction`), so the cap is that value and not a tolerance standing beside it, and one expression computes both. The ceiling is taken as the larger of the prior-only variance and the variance being capped, so a blended variance already above prior-only, which the disagreement term can produce, is not narrowed by a bound that exists to limit a widening. The tempering is a widening or it is nothing.

Only the share travels, not a second copy of the uncertainty. A host that wants the untempered figure recovers it by multiplying by $\sqrt{1-s}$, except where the tempering saturated, and the share is the thing a host would need anyway to apply a correction of its own. Two fields carrying one quantity in two states is a consistency obligation on every later edit, and the second field would have no reader that the first plus the share does not already have.

The calibration refit and this correction agree rather than double count, and the argument is what they each correct. The calibration buffer records the probability-space uncertainty as reported (`constr:platt:buffer`), including the tempered one, and the empirical-coverage refit — which measures standardised residuals against the reported intervals and inflates when they were too narrow (`alg:monitoring:empirical-coverage`) — sees wider intervals from degraded models and inflates less on their account. That is the correct interaction. The share corrects what the model can know about itself: exactly how much of the precision along this direction it lent itself, a quantity it tracks. The refit corrects what the model cannot know about itself: whether its uncertainties, borrowed mass and all, actually cover outcomes at the rate they claim. Removing the first would leave the second to discover the same deficit slowly, from outcomes, and to apply it as one factor across every direction alike — which is the aggregate reading this decision exists to replace.

One downstream consequence is recorded rather than decided here. Label guidance ranks pending entries by the uncertainty they carry (`sec:guidance:core`), and that uncertainty is now the tempered one, so guidance leans toward requests whose verdicts rested on borrowed precision. That is the direction the exploration signal was already reaching in — toward what the model knows least — read at the resolution a per-direction share provides rather than at the model's average condition. Nothing about what guidance exposes changes, so the opacity that surface owns is untouched (`dec:surface:guidance-opacity`): the ranking moves, the disclosure does not.

The field is named for what is measured. The floor share is a different quantity: the fraction of a model's least eigenvalue the spectral floor is carrying, one reading per model, taken along whichever direction the model is weakest in (`dec:health:tiered-queries`). This one is taken along whichever direction was asked about and counts both floors. It is called the borrowed share because that is what it is a share of — precision the model lent itself — and because "prior share" would overclaim: the prior precision a model is constructed at is not tracked as mass and is not counted here.

**Definition (Intervention effectiveness)** · `def:risk:intervention-effectiveness`

The divergence between the blended unconfounded estimate and the operational model's realised estimate is reported with every assessment:

$$\Delta(\phi) = \hat{\rho}_\text{eff}(\phi) - \hat{\rho}_\text{opr}(\phi)$$

A positive divergence means the unconfounded estimate exceeds the realised one: the host's interventions are suppressing outcomes that would otherwise occur, which is what an intervention is for. A divergence near zero means they are either unnecessary, because inherent risk is low, or ineffective, because adverse outcomes are happening anyway. The measurement does not distinguish those two, and no quantity available to the Core does.

Two qualifications travel with the number. The blended estimate carries a small persistent anchor contribution, so where anchor and sister substantially disagree the divergence mixes intervention effectiveness with model disagreement. And it inherits the sister's scope: it is interpretable only in regions with unconfounded coverage, and is prior extrapolation elsewhere.

**Schema (The risk basis)** · `schema:risk:basis`

The risk basis is the Core's complete risk output and the only Core state a derivation reads.

| Field | Carries |
| --- | --- |
| Effective estimate $\hat{\rho}_\text{eff}$ | The blended unconfounded risk estimate |
| Effective uncertainty $\sigma_\text{eff}$ | The blended standard deviation, evidence-only along this request's direction (`dec:risk:evidence-only-uncertainty`) |
| Calibration parameter $\kappa_\text{eff}$ | The sigmoid steepness at this blend weight |
| Risk probability $\hat{p}$ | The calibrated probability |
| Probability uncertainty $\sigma_{\hat{p}}$ | The propagated standard deviation, carrying the same widening |
| Borrowed share $s$ | How much of the precision behind that uncertainty the models' floors were holding up, in $[0,1]$ (`dec:risk:evidence-only-uncertainty`) |
| Operational estimate $\hat{\rho}_\text{opr}$ | The realised-risk estimate |
| Intervention effectiveness $\Delta$ | The divergence of the two |
| Blend weight $w$ | The anchor's share of the estimate |
| Sister uncertainty $\sigma^2_\text{inh}$ | The sister's own variance |
| Anchor uncertainty $\sigma^2_\text{anc}$ | The anchor's own variance |
| Sister regime record count | Weighted samples behind $\kappa_\text{sister}$ |
| Anchor regime record count | Weighted samples behind $\kappa_\text{anchor}$ |
| Label count | Labels processed at assessment time |

Fourteen fields, and the boundary they draw is the architectural one: a derivation given this and nothing else can compute a complete decision landscape, which is what makes the derivation replaceable and the Core decision-free (`prin:principle:measure-not-decide`). The borrowed share is present for the same kind of reason one step further in: the uncertainty beside it has already been widened by it, so a host that wants the untempered figure, or a correction of its own, needs the number the widening was computed from rather than a second copy of the uncertainty in its earlier state. The two regime counts are present so that a host can treat an assessment as preliminary while the calibration is still thin rather than discovering later that it was; which labels were admitted at all is the neighbouring question, and it is governed by the host's declared eligibility policy (`req:host:eligibility-policy`). The basis crosses to the derivation (`sig:landscape:derivation-function`) and is reported to the host within the assessment (`schema:output:assessment`).

#### Platt calibration · `sec:platt:calibration`

The division carries no material of its own. It collects the purpose of the calibration parameter, the two regimes and their soft transition, the fitting objective and the buffer it reads, the search that minimises it, the sample floor below which it does not run, the cadence at which it does, the initial value, the drift coupling, and what a regime transition does during convergence.

**Motivation (What the calibration parameter is for)** · `mot:platt:purpose`

The raw estimate is directionally correct and arbitrarily scaled. A higher estimate means higher risk, but nothing in the construction fixes what estimate corresponds to what probability, because the target the models train on is a sign and the feature scale is set by standardisation rather than by any outcome. A sigmoid applied to an arbitrarily scaled score produces an arbitrarily scaled probability.

The calibration parameter fixes the scale against outcomes. It is one number per regime, fitted so that among all requests receiving a stated probability, about that fraction actually turn out positive. That is the whole of the claim: calibration makes the probability mean what it says, and does not make the ranking better. A model that discriminates poorly and is perfectly calibrated still discriminates poorly, which is why calibration health and discrimination health are monitored separately.

**Definition (The two regimes)** · `def:platt:regimes`

The sister and the anchor relate scores to probabilities differently, because one reads fine-grained per-Sentinel features and the other coarse aggregates. Two parameters are fitted rather than one, and the effective parameter interpolates between them by blend weight:

$$\kappa_\text{eff}(w) = (1 - g(w)) \cdot \kappa_\text{sister} + g(w) \cdot \kappa_\text{anchor}, \qquad g(w) = \sigma(s_\kappa (w - 0.3))$$

The regime boundary sits at a blend weight of $0.3$ — sister-dominated below, anchor-dominated at or above — and the transition slope is $s_\kappa = 20$.

The transition is soft for a reason that a hard threshold would violate immediately. The blend weight moves continuously with every assessment, and a step change in the calibration parameter at the boundary would produce a discontinuity in the reported probability for two assessments whose underlying estimates differ imperceptibly. At a slope of twenty the interpolation is effectively complete within about a tenth of the weight either side of the boundary, which is narrow enough to keep the regimes distinct and wide enough to keep the probability continuous.

**Definition (The fitting objective)** · `def:platt:objective`

Each regime's parameter minimises a weighted cross-entropy over the calibration buffer, against the binary target $y = \mathbb{1}[v > 0]$:

$$\kappa_R^* = \arg\min_{\kappa > 0}\; -\sum_{i \in R} w_{\text{imp},i} \cdot \gamma_\text{cal}^{|R| - \text{rank}(i)} \cdot \bigl[y_i \log \sigma(\hat{\rho}_i / \kappa) + (1 - y_i) \log(1 - \sigma(\hat{\rho}_i / \kappa))\bigr]$$

Two weights multiply each record. The importance weight is the same one the model update used (`def:weighting:balancing-weights`), so the calibration is fitted against the class balance the model was trained under rather than against the raw one. The recency factor discounts older records geometrically by rank, so the fit tracks the current score distribution rather than the whole history the buffer happens to hold.

Cross-entropy rather than a squared error because the quantity being fitted is a probability, and cross-entropy is the proper scoring rule that a calibrated probability minimises in expectation. Fitting a squared error would produce a parameter that is optimal for a different question than the one the probability answers.

**Construction (The calibration buffer)** · `constr:platt:buffer`

The calibration reads a fixed-capacity circular buffer of $N_\text{cal,buf}$ records, two thousand by default. Each record is written at label time and carries what both regime fits need.

| Field | Carries |
| --- | --- |
| Effective estimate | The blended estimate at assessment time |
| Effective uncertainty | The blended raw uncertainty at assessment time |
| Blend weight | The anchor weight at assessment time |
| Binary outcome | Whether valence was positive, at label time |
| Importance weight | The weight the update applied, at label time |
| Label index | The global label counter |
| Axis predictions | One prediction per active axis, at assessment time |

Every record contributes to *both* regime fits, weighted by its blend contribution: a record at weight $w$ enters the sister fit at $(1 - g(w)) \cdot w_\text{imp}$ and the anchor fit at $g(w) \cdot w_\text{imp}$. A record deep in one regime contributes almost nothing to the other; a record near the boundary contributes substantially to both. A regime's effective sample count is the sum of its weights, and it is that weighted count the sample floor is read against, not a count of records.

The stored uncertainty is not used by the fit. It is there so that the realised coverage of the reported intervals can be measured after the fact (`alg:monitoring:empirical-coverage`), at a cost of eight bytes per record.

**Algorithm (Fitting the calibration parameter)** · `alg:platt:fitting`

The objective is minimised by golden-section search over the logarithm of the parameter.

1. **Bracket** the search on $\log \kappa$ over $[\log \kappa_\text{min}, \log \kappa_\text{max}]$, with $\kappa_\text{min} = 0.01$ and $\kappa_\text{max} = 100$ by default.
2. **Contract** the bracket by golden section until it is narrower than $10^{-4}$ in $\log \kappa$, or until fifty iterations have run.
3. **Publish** the resulting parameter for that regime.

Searching in the logarithm rather than the parameter is what makes a single tolerance meaningful across two orders of magnitude: the parameter is a scale, and a fixed absolute tolerance would be far too coarse at the bottom of the range and far too fine at the top. Golden section rather than a gradient method because the objective is one-dimensional and cheap, and a derivative-free bracket that cannot overshoot is worth more here than asymptotic speed. The cost is fifty passes over the buffer — a hundred thousand operations at default capacity, comfortably under a millisecond, and small enough that the refit cadence can be driven by correctness rather than by budget.

**Requirement (The minimum sample floor)** · `req:platt:minimum-samples`

A regime's parameter is refitted only when its weighted partition of the buffer holds at least $N_\text{cal,min}$ records — thirty by default — with at least $n_\text{cal,pos}$ positive outcomes and $n_\text{cal,neg}$ negative outcomes, three of each by default. Below any of the three floors the regime keeps the parameter it has.

Both floors are needed and neither substitutes for the other. A fit against thirty records that are all negative has no information about where the sigmoid should steepen and would drive the parameter to a bound; a fit against three records of each class has the classes but not the resolution. The floors are low deliberately: they are there to prevent a degenerate fit, not to certify a good one, and a host that treats the floor as a maturity signal has misread it. The two regime counts travel in the risk basis (`schema:risk:basis`) precisely so that the distinction between "fitted" and "fitted well" stays visible to whoever is consuming the probability.

**Table (Refitting cadence)** · `tab:platt:refit-cadence`

Five triggers refit a regime whose sample floor is met.

| Trigger | Condition |
| --- | --- |
| Periodic | Every $N_\text{refit}$ labels, two hundred by default |
| Lifecycle | After any Sentinel or outcome axis registration or deregistration |
| Drift | After any prediction drift accumulator reset |
| Anchor reactivation | The anchor regime meets its floor and more than $N_\text{refit}$ labels have passed since its last fit |
| Manual | A host-initiated recalibration request |

The five are not interchangeable. The periodic trigger tracks slow change; the lifecycle trigger exists because a dimension change alters the score distribution immediately and without warning; the drift trigger couples calibration to the monitor that noticed the scores moved; the reactivation trigger exists because a frozen regime is stale in a way no other trigger would notice (`disc:platt:regime-transition`); and the manual trigger is the host's escape hatch when it knows something the Core cannot see. A deployment that fires only the periodic trigger is calibrated on average and miscalibrated exactly when something changed.

**Definition (The initial calibration parameter)** · `def:platt:initial-value`

Both regimes start at $\kappa_0 = 1.0$.

The value matters less than it appears to. Before labels exist the estimate is near zero, so the probability is near one half whatever the parameter is, and the first refit at around two hundred labels replaces the initial value with one fitted against actual outcomes. What the choice fixes is the behaviour in the window between the first non-trivial estimates and the first fit, and unity is the neutral choice there: it neither compresses nor expands the raw score, so the probability reported in that window is the sigmoid of the score itself and is at least monotone in the quantity it claims to represent.

**Algorithm (Coupling the calibration to drift)** · `alg:platt:drift-integration`

A refit that moves the parameter substantially is itself evidence that the score distribution has changed, and the drift machinery is told so.

1. **Compute**, after each refit, the log-ratio $\delta_\text{cal} = |\log \kappa_\text{new} - \log \kappa_\text{old}|$.
2. **Compare** it against the threshold $\delta_\text{cal} > 0.1$, about a ten per cent change in the parameter.
3. **Reset**, when the threshold is exceeded, the prediction drift accumulators of the affected models, record the event, and report it in the health snapshot (`schema:output:health-snapshot`).

The reset is the point of the coupling. A drift accumulator measures the divergence between predictions and outcomes against a fixed mapping from score to probability; when that mapping is refitted, the accumulated divergence was computed under a mapping that no longer applies, and carrying it forward would attribute the calibration's own correction to the model's drift. Resetting discards genuine evidence along with the stale evidence, which is the accepted cost: a false drift alarm caused by the system's own recalibration is worse than a delayed true one.

**Discussion (Regime transition during convergence)** · `disc:platt:regime-transition`

Convergence runs the blend weight from anchor-dominated to sister-dominated, and that passage produces four calibration effects that are expected and self-correcting rather than faults.

The first fit is unstable. The leverage bound fires on a third to two thirds of early labels (`data:gaussian:binding-frequency`), so the score distribution at two hundred labels is not the steady-state one; the second fit may therefore move the parameter enough to trip the drift threshold. That is a calibration transient, and a threshold crossing at the second refit during convergence is not grounds for investigation (`cav:limitation:pre-calibration`).

The anchor regime then starves. Once the sister converges, most records sit at a weight far below the boundary, the anchor's weighted count falls under its floor, and its parameter freezes. A frozen parameter is harmless while the sister dominates and stale the moment the anchor reactivates — after a Sentinel registration, say — which is why reactivation is a refit trigger of its own and why the frozen state and the labels since the last anchor fit are both reported.

Boundary records cross-contaminate. During the transition many records sit near the boundary and contribute substantially to both fits, pulling each regime's parameter toward the other's. The periodic refit corrects this iteratively, and the first few refits during the transition may oscillate before settling. No mechanism change is warranted: the oscillation is bounded by the same soft transition that causes it, and it ends when the weight distribution stops moving.

#### Training eligibility · `sec:eligibility:training`

**Table (Training eligibility)** · `tab:eligibility:training`

Which observation trains which model is the specification's causal-inference contract, and it is a table rather than a predicate because every row is a separate judgement about confounding.

| Action taken | Outcome | Sister and anchor train | Operational trains | Why |
| --- | --- | --- | --- | --- |
| Allow | Any | Yes | Yes | Unconfounded |
| Challenge | Passed, then any outcome | Yes | Yes | The request proceeded |
| Challenge | Failed | Configurable, included by default | Yes | Policy-governed (`req:host:eligibility-policy`) |
| Slow | Any | No | Yes | Causally confounded |
| Block | Any | No | Yes | Causally confounded |
| Any | Host-investigated | Yes | Yes | Ground truth (`conv:eligibility:ground-truth`) |

The sister and anchor train only on unconfounded outcomes; the operational model trains on every labelled outcome. Slow is confounded and must not train the sister or the anchor: the host slowed the request, the outcome that followed is the outcome of a slowed request, and admitting it teaches the inherent-risk estimate about the host's own behaviour. A failed Challenge does train the sister by default, because the default policy asserts that failing a challenge is a property of the request rather than an effect of having been challenged; a deployment that disputes that sets the policy false and submits investigated challenge outcomes as ground truth instead.

The action recorded here is observed host behaviour, not a recommendation. The same four names appear in the decision landscape as candidate actions, and the two readings must not be conflated: this table asks whether an observation was confounded, and a landscape asks what to do next. The predicate follows the table: Allow is eligible, Slow and Block require ground truth, and Challenge follows the declared policy, with ground truth overriding every action.

**Convention (The ground-truth flag)** · `conv:eligibility:ground-truth`

A label may assert that its outcome was determined by investigation rather than inferred from what the host did. The flag is the host's assertion of exactly that, and it makes the label eligible whatever action was taken.

The flag is the only mechanism that supplies unconfounded coverage in regions the host always restricts, and those regions are precisely where the sister model would otherwise never learn anything. Without it the censored bandit problem is closed: the host restricts where risk is estimated high, the sister sees nothing there, and the estimate is never corrected by evidence.

The Core trusts the flag unconditionally and has no mechanism to verify it. An investigation pipeline that systematically mislabels outcomes corrupts the sister model directly and silently, and no Core-side diagnostic distinguishes a corrupted pipeline from a genuine change in the environment (`cav:limitation:investigation-pipeline`). The flag is a contract discharged by the host, and its integrity is the host's to maintain.

#### Importance weighting · `sec:weighting:importance`

The division carries no material of its own. It collects the two class-rate trackers, the update they run, their initial value, why they carry no time-indexed decay, the weights derived from them, and how those weights interact with the leverage bound.

**Table (The class-rate trackers)** · `tab:weighting:trackers`

The risk target is binary and positive-valence events are typically much rarer than negative ones, so an unweighted posterior gradient is dominated by the majority class. Two trackers estimate the positive-valence rate, and the weights are derived from them.

| Tracker | Updated on | Feeds | Decay rate |
| --- | --- | --- | --- |
| Global | Every label | The operational model | $\gamma_\text{opr}$ |
| Eligible | Eligible labels only (`tab:eligibility:training`) | Sister, anchor, axis models, and the Ledger's neutral reference | $\gamma_\text{inh}$ |

Each tracker decays at the forgetting rate of the models it serves, so the class balance it reports covers the same effective horizon as the evidence those models retain. The matching is the whole design. A tracker faster than its model over-reacts to transient fluctuations in class balance and applies a correction the model's own memory does not warrant; a tracker slower than its model applies stale balance corrections to recent gradients. No new parameter is introduced: both rates are already fixed by the forgetting table (`tab:risk:forgetting-rates`).

An axis with a custom forgetting rate still draws from the eligible tracker, and the resulting mismatch is bounded per label by the rate difference over the complement of the eligible rate — negligible at the default and small anywhere in the supported range. The global and eligible trackers take the operational and inherent rates respectively, matching the model horizons this table assigns them.

**Algorithm (The tracker update)** · `alg:weighting:tracker-update`

Each tracker is an exponentially weighted average of the positive-valence indicator, updated on the labels that qualify for it.

1. **Determine** eligibility from the recorded action and the ground-truth flag — a table lookup available before any model is touched (`tab:eligibility:training`).
2. **Update** the global tracker on every label, and the eligible tracker only on eligible labels, by

   $$P_+ \leftarrow \gamma_P \, P_+ + (1 - \gamma_P) \, \mathbb{1}[v > 0]$$

3. **Derive** the weights from the updated rate before the model update reads them (`alg:runtime:update-path`).

The ordering is what makes the procedure well defined. Eligibility is resolved first because it decides which tracker moves and which models train; the trackers move before the weights are derived so that a label contributes to the balance estimate it is itself weighted against, which keeps the estimator consistent as the rate drifts.

**Definition (The trackers' initial value)** · `def:weighting:initial-value`

Both trackers start at $P_{+,0} = 0.5$, where the two weights are equal at unity. The first labels are therefore weighted evenly, committing to no assumed base rate, and the average converges toward the empirical rate over roughly $3/(1 - \gamma_P)$ labels.

| Tracker | Rate | Labels to converge | At two hundred labels a day |
| --- | --- | --- | --- |
| Global | $0.9995$ | About six thousand | About thirty days |
| Eligible | $0.9998$ | About fifteen thousand eligible | About a hundred and twenty-five days at sixty per cent eligibility |

Through convergence the weights under-correct for imbalance relative to the true rate, so the gradient is biased toward the majority class, and the bias decreases monotonically as the estimate settles. The horizon is long enough that it is worth avoiding rather than waiting out: the initial value is configurable, and a host that knows its base rate can seed it directly, or can pre-seed with historical labels (`alg:host:pre-seeding`) and eliminate the transient entirely.

**Decision (No time-indexed decay on the trackers)** · `dec:weighting:no-time-decay`

The trackers do not decay with elapsed time, only with labels. The decision is recorded because every other stateful quantity in the system does decay with time, and the exception looks like an oversight until the reasoning is stated.

The class balance among labels has not changed because time passed without labels. What has changed is the system's certainty about it, and that certainty is already represented elsewhere: the models' own time-indexed precision decay widens the posterior and reduces the influence of historical labels regardless of what weights they carried. Decaying the tracker as well would represent the same loss of certainty twice, and would do it in a place where the representation is wrong — a decayed rate estimate does not become more uncertain, it drifts toward whatever its decay target is.

After a drought the tracker therefore retains its pre-drought estimate, and the first labels back are weighted on the last known balance. If the balance really did change, the tracker adapts over its own horizon, which is the same horizon the model's forgetting factor uses to discount the evidence trained under the old balance. The two stay in step, which is the property the whole design is built around.

**Definition (The balancing weights)** · `def:weighting:balancing-weights`

The weights are the reciprocal of twice the relevant class rate, capped at a ceiling:

$$w_+ = \min\!\left(\frac{1}{2P_+ + \varepsilon},\; w_\text{ceiling}\right), \qquad w_- = \min\!\left(\frac{1}{2(1 - P_+) + \varepsilon},\; w_\text{ceiling}\right)$$

The ceiling defaults to one hundred for every model. The defining property of this form is that each class contributes exactly half the total gradient weight, at any class rate, until the ceiling binds.

| Positive rate | $w_+$ | $w_-$ | Ratio | Gradient balance |
| --- | --- | --- | --- | --- |
| $0.50$ | $1.0$ | $1.0$ | $1.0$ | 50% |
| $0.10$ | $5.0$ | $0.56$ | $8.9$ | 50% |
| $0.05$ | $10.0$ | $0.53$ | $18.9$ | 50% |
| $0.01$ | $50.0$ | $0.51$ | $98.0$ | 50% |
| $0.005$ | $100.0$ | $0.50$ | $200.0$ | 50% |
| $0.002$ | $100.0$ | $0.50$ | $200.0$ | 28.6% |
| $0.001$ | $100.0$ | $0.50$ | $200.0$ | 16.7% |

The ceiling binds below a rate of $1/(2 w_\text{ceiling})$ — half a per cent at the default — and from there the balance falls linearly with the rate, so the model under-learns from exactly the observations it has fewest of (`cav:limitation:ceiling`). A deployment below that rate can raise the ceiling and accept the added single-observation volatility, pre-seed with historical positive labels, or monitor discrimination directly (`def:monitoring:top-decile-lift`). The balancing form includes the ceiling, so every row of this table describes the weights.

**Discussion (Weighting against the leverage bound)** · `disc:weighting:leverage-interaction`

The importance weight and the leverage bound both cap what one observation does, and which of them binds changes over the life of the deployment.

Early, the bound dominates. A positive-valence label carries a large importance weight and arrives against a near-prior covariance, so its leverage is high and the cap $c/(h + \varepsilon)$ overrides the importance weight entirely; the bound fires on a third to two thirds of positive labels during cold start. That is the layered protection working: the importance ceiling cannot prevent gradient concentration during cold start, and the bound can, because it is computed from the posterior rather than from the class rate.

Later, the weight dominates. As the posterior concentrates, leverage falls for typical feature directions, the bound relaxes, and the importance weight becomes the operative constraint — firing on a few per cent of positive labels and almost never on negative ones in steady state.

The interaction is deliberate rather than incidental. A heavily weighted observation is exactly the observation whose potential to distort the posterior is greatest, and it receives proportionally more scrutiny for that reason. The two mechanisms are not redundant: one bounds the influence of a class, the other bounds the influence of a direction, and a deployment needs both.

**Definition (The risk target)** · `def:risk:target`

The target every risk model trains on is the sign of the valence:

$$r_\rho = 2 \cdot \mathbb{1}[v > 0] - 1 \in \{-1, +1\}$$

Positive valence maps to $+1$ and everything else to $-1$, under the sign convention the host declares (`conv:valence:sign`).

The binary encoding does three things. It aligns with the zero-mean prior, since under balanced weighting the expected target is zero, which is what the prior asserts. It eliminates the conflation of severity with probability: a severity-weighted target would train the model to predict expected loss, the product of probability and magnitude, where the risk estimate is meant to represent probability alone. And it makes the gradient uniform, so every label contributes the same magnitude before weighting and no low-severity event arrives as a near-zero gradient.

Severity is not discarded; it enters through features rather than through the target (`cav:limitation:severity-path`). Identity and Ledger valence averages, compressed and raw, carry severity history into the feature vector, and the model learns whatever relationship holds between that history and the binary outcome. A deployment needing severity predicted in its own right registers a severity outcome axis (`def:registry:outcome-axis`).

**Definition (The feature compression scale)** · `def:risk:compression-scale`

A two-sided compression scale governs the compressed valence features — not the risk target, which is already bounded:

$$r_v = \tanh(v / \kappa_v), \qquad \kappa_v \leftarrow \gamma_\kappa\,\kappa_v + (1 - \gamma_\kappa)\,|v| \quad \text{when } v \neq 0$$

The scale adapts to the typical magnitude of the valence values the deployment actually reports, and the compression maps arbitrary magnitudes into the open unit interval.

Compression is needed because the features are averages and an unbounded input lets one extreme observation dominate an average for a long time; adaptation is needed because the alternative is a configured scale, which asks the host to know its own valence distribution before it has seen one. The hyperbolic tangent preserves ordinal distinctions throughout its range while flattening the tail, so a large loss still reads as larger than a moderate one without reading as a hundred times larger. Per-axis scales adapt at the same rate on the labels reporting their axis (`def:axis:adaptive-compression`).

**Algorithm (The leverage-bounded update)** · `alg:update:sherman-morrison`

Every Bayesian linear model in the document is updated by this procedure, given a prior precision $B$, its inverse $\Sigma$, a forgetting factor $\gamma$, a standardised observation $\hat{\phi}$, a target $r$, a target weight $w_\text{target}$, the ceiling, and a leverage safety factor $c$.

1. **Compute the leverage** from the prior covariance: $v = \Sigma \hat{\phi}$ and $h = \hat{\phi}^\top v$.
2. **Cap the weight:** $w_\text{eff} = \min(w_\text{target},\, w_\text{ceiling},\, c/(h + \varepsilon))$.
3. **Apply forgetting:** $B \leftarrow \gamma B$.
4. **Apply the replenishment floor:** $B_{jj} \leftarrow \max(B_{jj}, \lambda_\text{floor})$ for every $j$ (`req:gaussian:prior-replenishment-floor`).
5. **Update the precision:** $B \leftarrow B + w_\text{eff} \hat{\phi}\hat{\phi}^\top$.
6. **Update the covariance:** $\Sigma \leftarrow \frac{1}{\gamma}\bigl(\Sigma - \frac{w_\text{eff} v v^\top}{\gamma + w_\text{eff} h}\bigr)$.
7. **Update the mean** using the prior covariance-vector product $v$ of step 1, not the covariance step 6 has just replaced:

   $$\mu \leftarrow \mu + \frac{w_\text{eff}\,(r - \hat{\phi}^\top \mu)}{\gamma + w_\text{eff} h}\; v$$

Step 7 is stated against $v$ deliberately. The mean update is the posterior covariance times the weighted residual, and substituting the Sherman–Morrison identity turns that into the prior product divided by the same denominator step 6 uses — algebraically identical, and unambiguous about which covariance is meant, where an ordered procedure whose sixth step has already overwritten $\Sigma$ is not. The covariance is recomputed from the precision by Cholesky factorisation every thousand labels, or sooner when the condition number crosses its threshold, because steps 5 and 6 maintain the two matrices independently and accumulate disagreement between them (`alg:gaussian:condition-adaptive-recompute`).

**Proposition (What the leverage bound guarantees)** · `prop:update:leverage-bound`

The leverage $h = \hat{\phi}^\top \Sigma \hat{\phi}$ measures how far an observation stretches the model along a direction of high posterior uncertainty, and the cap $c/(h + \varepsilon)$ bounds the ratio of posterior to prior precision along any direction to at most $1 + c$. At the default safety factor of five, no single observation can increase the precision along its own direction by more than a factor of six.

The bound fires when $h > c/w_\text{target}$, which makes it a function of the class as much as of the geometry: at a negative label's typical weight it requires a leverage of ten and is rare, while at a positive label's weight in a sparse deployment it requires a tenth and is common.

The guarantee is bought at a cost stated plainly. The cap makes the effective weight depend on the current posterior, so the observation model for the $n$-th label depends on the first $n-1$ — the composite posterior is not the posterior of any fixed generative model (`cav:gaussian:fixed-model-departure`). The departure is one-directional: the result is wider than the fixed-model posterior, concentrated in the directions where the bound fires most (`prop:gaussian:conservatism`). A conservative posterior is the right failure mode for a system whose uncertainty is consumed by a decision layer, and the calibration absorbs the effect on point estimates (`def:platt:regimes`).

**Table (Forgetting rates)** · `tab:risk:forgetting-rates`

Every model forgets along two independent axes: by labels processed, and by time elapsed.

| Model | Label-indexed rate | Half-life in labels | Time-indexed rate, hourly | Time half-life |
| --- | --- | --- | --- | --- |
| Operational | $0.9995$ | About 1,400 | $0.9999$ | About 290 days |
| Sister | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |
| Anchor | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |
| Outcome axes, by default | $0.9998$ | About 3,500 | $0.9999$ | About 290 days |

The operational model forgets two and a half times faster than the others, and the asymmetry follows from what each estimates. Operational risk includes the host's intervention posture, which a host may change deliberately and overnight; inherent and coarse risk reflect the underlying environment, which changes on its own slower schedule. A single shared rate would either make the sister chase the host's policy changes or make the operational model lag them.

The time-indexed rate is uniform and deliberately slow. Its purpose is to guarantee that a model left without labels eventually widens rather than holding a stale posterior forever; it is a floor under forgetting, not a mechanism meant to drive forgetting under normal operation, where the label-indexed rate is faster by orders of magnitude.
