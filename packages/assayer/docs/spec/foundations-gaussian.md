### Chapter (Mathematical Foundation) · `chap:spec:mathematical-foundation`

The chapter states the algebra the whole lifecycle rests on. A Bayesian linear model maintains a Gaussian posterior over parameters $\theta \in \mathbb{R}^p$:

$$\theta \sim \mathcal{N}(\mu, \Sigma), \qquad B = \Sigma^{-1}$$

where $\mu$ is the posterior mean, $\Sigma$ the posterior covariance, and $B$ the posterior precision. Observations evolve these through a Sherman–Morrison update (`alg:update:sherman-morrison`). Each per-observation update is the exact Bayesian update given the current posterior and the step's effective observation precision; because the leverage bound makes that precision posterior-dependent, the composite posterior is not the posterior of any fixed generative model (`cav:gaussian:fixed-model-departure`). The exact identities for the two operations that change the dimension of $\theta$ are finite, and this chapter states them as results so that the lifecycle chapters may cite them rather than re-derive them. Marginalisation regularises one identity and may fall back from it, so the exact theorem and the numerical contract are kept separate below.

**Theorem (Extension)** · `thm:gaussian:extension`

Adding $r$ new dimensions with an independent prior:

$$\mu' = \begin{pmatrix} \mu \\ \mu_0 \end{pmatrix}, \qquad \Sigma' = \begin{pmatrix} \Sigma & \mathbf{0} \\ \mathbf{0} & \Sigma_0 \end{pmatrix}, \qquad B' = \begin{pmatrix} B & \mathbf{0} \\ \mathbf{0} & B_0 \end{pmatrix}$$

The existing posterior is the exact marginal of the extended posterior over the original dimensions. No information is lost and no approximation is introduced. The new dimensions are independent of the existing ones under the prior; future observations populating both old and new features create the cross-terms through the ordinary update. Extension is what makes registering a Sentinel or an outcome axis at runtime cost nothing: the model that existed before the registration is recoverable from the model that exists after it, exactly.

**Theorem (Marginalisation)** · `thm:gaussian:marginalisation`

Removing $r$ dimensions by integrating them out. Partition the indices into kept ($k$) and removed ($r$):

$$\mu' = \mu_k, \qquad \Sigma' = \Sigma_{kk}, \qquad B' = B_{kk} - B_{kr}\,B_{rr}^{-1}\,B_{rk}$$

The marginal mean is the subvector, the marginal covariance the submatrix, and the marginal precision the Schur complement. All three identities are exact for the current joint posterior. They do not describe the counterfactual posterior that training without the removed features would have produced, because those features may have changed observation leverage along the way. The lifecycle uses the regularised algorithm below, so its numerical precision is an approximation to this theorem whenever the coupling is non-zero and $\delta>0$.

**Definition (Gathered removed partition)** · `def:gaussian:gathered-partition`

The removed set need not be contiguous. The dimension map (`def:dimension:layers`) supplies contiguous ranges for physical blocks and slots and exact range lists for composite lifecycle scopes, which lets the removed partition be identified from a Sentinel identifier, an outcome-axis identifier, an identity-dimension identifier, or a competitive-cell identifier even where the removed indices do not occupy one slice of the layout.

The Schur algebra requires only the partition into kept and removed, never that the removed indices be adjacent. An implementation may therefore gather or permute the removed indices transiently, apply the Schur complement, and compact the surviving layout afterwards. The gathering is a matter of arrangement and changes no result: the marginal is determined by which dimensions are removed, not by where they sat.

**Remark (Why precision and covariance are both tracked)** · `rem:gaussian:dual-tracking`

Maintaining both $B$ and $\Sigma$ accumulates small floating-point drift between periodic Cholesky recomputations. In exact arithmetic $\Sigma_{kk} = (B')^{-1}$; in floating-point the extracted covariance and the Schur-complement precision inherit independent errors from their two parent matrices, so the pair can drift apart while each remains individually plausible.

The remedy is scaled to the operation. For general marginalisation of larger lifecycle blocks, recompute $\Sigma' = (B')^{-1}$ by Cholesky factorisation after the Schur complement, re-establishing exact dual tracking at that point. For low-rank marginalisation with small $r$ — competitive cell exit (`alg:keyspace:cell-exit`) being the case that occurs often — the covariance path may extract $\Sigma_{kk}$ directly. The discrepancy from the regularised low-rank step depends on the offset relative to the removed block's spectrum and on its coupling to the kept block; $\delta$ alone is not an error bound. The ordinary periodic recomputation schedule clears the resulting dual-tracking discrepancy, not the marginalisation bias that produced it.

**Definition (The Schur complement)** · `def:gaussian:schur-complement`

The Schur complement has a precise meaning for this posterior, and it is not merely the algebraic residue of eliminating a block. Information that the removed dimensions carried about the kept dimensions — encoded in the off-diagonal blocks — is transferred to the kept dimensions' precision through the correction term $B_{kr}\,B_{rr}^{-1}\,B_{rk}$.

Under the exact complement, what is learned is therefore not lost with what taught it. Information gained from a removed Sentinel's features about the relationship between its alarm patterns and the identity features survives that Sentinel's deregistration through the change to the surviving precision. The regularised complement attenuates that change, and the fallback discards it.

**Algorithm (Regularised Schur complement)** · `alg:gaussian:regularised-schur`

Before a replenishment floor intervenes, uniform forgetting by $q=\gamma^t$ scales both a removed block and its coupling: $B_{rr}=qA$ and $B_{kr}=qC$. The exact correction then scales as $qC A^{-1}C^\top$, not as a quantity of order one. Once diagonal replenishment intervenes the two blocks need not share even that scaling. The numerical risk is a small or ill-conditioned factorisation and accumulated matrix drift, not a correction that uniform decay makes diverge.

On marginalisation, compute the regularised form:

$$B' = B_{kk} - B_{kr}\,(B_{rr} + \delta I)^{-1}\,B_{rk}$$

with $\delta = \lambda_\text{prior} \cdot \varepsilon_\text{Schur}$, at a default $\varepsilon_\text{Schur} = 10^{-4}$. This bounds the correction's magnitude and keeps the inverted block positive definite.

The offset also changes the answer in exact arithmetic. If $S_0$ is the exact Schur complement and $S_\delta$ the regularised one, then

$$S_\delta-S_0 = \delta B_{kr}B_{rr}^{-1}(B_{rr}+\delta I)^{-1}B_{rk} \succeq 0.$$

Regularisation therefore retains more precision than the exact marginal and produces a narrower covariance. Positive definiteness does not bound this bias: the bias moves the least eigenvalue in the passing direction.

Two guards stand before the correction, and they answer different questions. The upper one is the host's posture. Where the raw condition number $\kappa(B_{rr})$ exceeds a host-declared ceiling, default $10^8$, the block is refused as a source: the deployment has said it will not fold information of that quality into its surviving models, and no arithmetic makes that judgement for it. The lower one is an arithmetic limit a deployment cannot move. Where the regularised block the correction is actually solved against has $\kappa(B_{rr}+\delta I) > 10^{12}$, the solve carries no significant digits and the correction is declined on arithmetic grounds alone. Both set $B' = B_{kk}$, discarding the transferred information rather than computing it from a block that cannot support the computation, and the diagnostics name which of the two refused.

The separation is forced by the offset itself. Regularisation changes the condition number to

$$\kappa(B_{rr}+\delta I) =\frac{\lambda_{\max}(B_{rr})+\delta}{\lambda_{\min}(B_{rr})+\delta},$$

so at the reference prior a raw $10^8$ becomes a regularised $10^4$. A single threshold read before the lift therefore refuses blocks the regularised solve would handle comfortably, and one read after it says nothing about the quality of the information being folded in; the two questions have different answers on the same matrix. Both figures come from one spectrum of $B_{rr}$, since adding $\delta I$ shifts every eigenvalue by $\delta$ and does nothing else.

The condition number here is of the spectrum and not of the diagonal. A ratio of diagonal entries is a lower bound on it and no more: a block of equal diagonal can be arbitrarily ill-conditioned and reports a ratio of one, so a guard reading the ratio refuses far less than it appears to. Where the spectrum cannot be computed the ratio stands in, and what it then certifies is only what a lower bound certifies.

Either path is followed by the verification of (`req:gaussian:positive-definiteness`), and a failure there falls back to $B' = B_{kk}$ likewise. Every path reports what its approximation cost (`req:gaussian:marginalisation-error-reported`).

**Requirement (Positive definiteness of the marginalised precision)** · `req:gaussian:positive-definiteness`

After either path of (`alg:gaussian:regularised-schur`), the marginalised precision $B'$ is verified positive definite before it is adopted, and the verification is a factorisation of $B'$ rather than a reading of its diagonal: a Cholesky attempt on $B' - \phi I$ completes if and only if $\lambda_{\min}(B') > \phi$, where $\phi = 8k\varepsilon_\text{mach}\max(U, 0)$ is the finite-resolution floor taken at an outward-rounded bound $U \geq \lambda_{\max}(B')$ (`const:assayer:schur-verification-resolution`). Where the factorisation does not complete, the correction is discarded and $B'= B_{kk}$ is adopted instead.

The requirement is stated of the eigenvalue and not of the diagonal because the two are not the same test. A positive minimum diagonal entry is necessary for positive definiteness and is not sufficient for it: a matrix with strictly positive diagonal can carry a negative eigenvalue through its off-diagonal structure. In exact arithmetic an SPD parent and positive $\delta$ already give $S_\delta \succeq S_0 \succ 0$; this verification detects numerical error or an invalid parent, not the approximation bias. A diagonal test in this position passes cases the verification is meant to catch.

**Requirement (Verification of the rank-one marginalised precision)** · `req:gaussian:rank-one-verification`

The rank-one path removes a single dimension by forming $\widehat S = B_{kk} - \widehat Q$ for an entrywise-computed $\widehat Q_{ij} = \text{fl}(\text{fl}(cu_i)u_j)$ with $c = 1/(a+\delta)$. It verifies $\widehat S$ before adopting it, at the same finite-resolution boundary the general path uses (`req:gaussian:positive-definiteness`), and the verdict is read from a certificate of $\widehat S$ itself. Two certificates discharge it, and a strictly positive diagonal is neither of them.

The first is an outward-rounded Gershgorin bound. For row sums $r_i \geq \sum_{j \neq i} |\widehat s_{ij}|$ and

$$L = \min_i(\widehat s_{ii} - r_i), \qquad U = \max_i(\widehat s_{ii} + r_i),$$

Gershgorin gives $\lambda_{\min}(\widehat S) \geq L$ and $\lambda_{\max}(\widehat S) \leq U$, so at width $k$ the test

$$L > 8k\varepsilon_\text{mach}\max(U, 0)$$

reaches the same decision the shared verifier would reach (`const:assayer:schur-verification-resolution`). Every rounding in it is directed away from acceptance: the row sums round up, $L$ rounds down, and $U$ and the resolution floor round up. A row sum accumulated in round-to-nearest is not a bound and certifies nothing. The certificate is sufficient and not necessary — a dense positive definite matrix need not be strictly diagonally dominant — so an inconclusive result is not a refusal.

The second is the factorisation, taken exactly as the general path takes it. It decides whenever the first is inconclusive.

A non-positive diagonal entry refuses without either. A diagonal entry is the Rayleigh quotient at a coordinate vector, so $\lambda_{\min} \leq \min_i \widehat s_{ii} \leq 0$ while the resolution floor is never negative. That is the one direction in which the diagonal remains evidence, and it is a proof of refusal rather than a licence to adopt.

Adoption on a positive diagonal alone is forbidden, and the prohibition is not a precaution. The path's exact-arithmetic theorem is sound — for $B \succ 0$ and $\delta \geq 0$ the regularised scalar Schur complement is positive definite — and the path does not compute it. The rounded $\widehat Q$ has neither rank one nor the nullspace of $cuu^\top$, so rank-one interlacing no longer confines the risk to a single eigenvalue. A represented parent that is exactly positive definite, with $a + \delta = 1.00001$ and an exact result whose least eigenvalue is about $+9.62 \times 10^{-19}$, reaches this point as a $\widehat S$ whose minimum diagonal is about $10^{-7}$ and whose least eigenvalue is about $-4.12 \times 10^{-19}$. The diagonal admits it; this requirement refuses it. Neither the positive denominator nor the scalar pivot's condition number is the missing margin, both being premises of the exact proof rather than certificates of the computed result.

A refusal falls back to $B' = B_{kk}$ and reports the whole of the discarded correction (`req:gaussian:marginalisation-error-reported`). Every verification reports which certificate decided it, so the share the cheap certificate settles is measured rather than assumed; that share is what the arrangement's cost advantage rests on, and no analysis supplies it in advance.

**Requirement (The marginalisation error is computed and reported)** · `req:gaussian:marginalisation-error-reported`

Every marginalisation reports the approximation it committed, as a figure computed at that event rather than as a tolerance declared in advance. The quantity is $\text{tr}(E_\delta)$ for $E_\delta = S_\delta - S_0$, the precision the regularisation retained over the exact marginal, reported together with the share $\text{tr}(E_\delta)/\text{tr}(C_0)$ of the exact correction $C_0$ that did not survive. Each figure is labelled as a measurement or as an upper bound, and a bound is never presented as a measurement.

The offset alone cannot be that report. The absolute bound

$$\lVert E_\delta\rVert \leq \frac{\delta\lVert B_{rk}\rVert^2}{a(a+\delta)}, \qquad a=\lambda_{\min}(B_{rr}),$$

contains the removed block's least eigenvalue and the coupling, so a value of $\varepsilon_\text{Schur}$ fixes no error at all until those two are known; and the relative error depends further on $\lambda_{\min}(S_0)$, which nothing in this corpus bounds below. The measurement is available whenever the exact correction is numerically reachable — the solve producing it having more resolution than the difference it is taken across — and the bound above stands in otherwise. That is the ill-conditioned regime, which is precisely where $B_{rr}^{-1}$ is untrustworthy and precisely what a raised posture ceiling admits, so the labelling is not a formality.

A discarded correction reports the whole of itself. As $\delta$ grows without bound the correction vanishes and $S_\delta \to B_{kk}$, so a fallback is this same approximation at its maximum rather than an exact path standing beside it, and the share it reports is one.

The per-event figures fold into a cumulative report across lifecycle events. That report is a monitor and not a bound: the corpus composes no law over repeated marginalisations (`inv:guarantee:structural-exactness`), the events acting on different partitions and separated by observation and forgetting. What it answers is whether a deployment is accumulating approximation at a rate that argues for a recompute, which is a question no single event can be asked.

**Table (Operation costs)** · `tab:gaussian:operation-costs`

Costs of the dimension-changing operations and of the per-observation update, at the current parameter count $p$.

| Operation | Cost | When |
| --- | --- | --- |
| Extension by $r$ dimensions | $O(p \cdot r)$ array growth, plus prior initialisation | Sentinel or outcome-axis registration |
| General marginalisation of $r > 1$ dimensions | $O(p'^2 r + p'r^2 + r^3 + p'^3)$ | Sentinel or outcome-axis deregistration |
| Low-rank marginalisation of $1 + T_5$ dimensions | $O(p^2)$ where the cheap verification certificate carries, $O(p^3)$ where it falls through to the factorisation | Competitive cell exit; rank 2 at the default $T_5 = 1$ |
| Per-observation update | $O(p^2)$ Sherman–Morrison at the current $p$ | Every label |

At the reference configuration (`tab:resource:reference-configuration`), where $p = 638$, and with $r = 68$ — one Sentinel's slot together with its interaction dimensions — the Schur complement costs approximately 45M operations. That total is the sum across the three matrices maintained: roughly 21M for the precision update, with the covariance and mean updates supplying the remainder. At 1 GFlop/s this is approximately 45 ms. General marginalisation then performs the Cholesky recomputation of $\Sigma' = (B')^{-1}$ that restores dual tracking, adding $O(p'^3/3)$ work — about 56 ms at the surviving $p' \approx 570$. Registration and deregistration are deployment events, not per-request events, and these figures are to be read against that.

The low-rank row carries two costs because its verification carries two certificates (`req:gaussian:rank-one-verification`). The quadratic figure is the one the frequent path is meant to pay: forming the correction, subtracting it, accumulating the outward-rounded row sums, and extracting the covariance block are all $O(p^2)$. The cubic figure is what an inconclusive certificate costs: a Cholesky factorisation at the surviving width, $O(p^3/3)$ and so the same work as the covariance recomputation named above, of the order of tens of milliseconds at reference width. Which figure a deployment actually pays is not derivable here — strict diagonal dominance is a property of the precision matrices it produces, not of the algorithm — which is why the share is reported per event rather than asserted.

**Theorem (The two-level posterior guarantee)** · `thm:gaussian:two-level-guarantee`

The posterior carries two guarantees at two levels, and the distinction between them is the chapter's central result.

At the level of lifecycle events, extension produces a joint posterior whose marginal equals the pre-extension posterior. The unregularised Schur identity produces the exact marginal over the surviving dimensions. Marginalisation instead uses the declared regularised approximation and may fall back to the kept block (`alg:gaussian:regularised-schur`), so neither unconditional exactness nor zero accumulated approximation is promised (`inv:guarantee:structural-exactness`).

At the level of individual observations, each update is exact given the current posterior as prior (`inv:guarantee:per-observation-exactness`), with an observation precision determined by the importance weight, the ceiling, and the leverage bound. Forgetting and importance weighting are fixed, data-independent components of the observation model and are well-defined generative parameters. The leverage bound is not: the effective precision depends on the current posterior through the observation's leverage, so the sequence of observation models is self-referential — the model generating observation $n$ depends on what was learned from its predecessors.

The per-observation statement and the lifecycle statement have different qualifications. Every extension and every unregularised marginalisation obeys its exact identity; a regularised or fallback marginalisation is a deterministic approximation. Every per-observation update remains exactly the Bayesian update it claims to be. What does not follow, and what is stated separately below, is that their composite is the posterior of any one fixed model.

**Caveat (Departure from the fixed-model reading)** · `cav:gaussian:fixed-model-departure`

The posterior is not the posterior of any fixed generative model. No single model, stated before observing the data, produces this posterior when updated sequentially over the observations, because the leverage bound makes each step's observation precision depend on what the previous steps produced.

What the posterior is remains well defined: the exact result of a determined recursive computation, in which each step applies a Bayesian update with a specific observation precision to the previous posterior. The resulting Gaussian has a mean — a weighted combination of all observations under exponential discounting and class-balanced importance — and a covariance reflecting the accumulated information, widened in the directions where the leverage bound has fired. Both are useful for their intended purposes.

The consequence to disclose is that standard Bayesian calibration guarantees — credible-interval coverage, posterior predictive checks — do not apply directly to this posterior. Platt calibration (`def:platt:regimes`) supplies an empirical calibration that absorbs the leverage bound's effect on point estimates. It does not absorb its effect on uncertainty estimates, and nothing in the system claims otherwise.

**Proposition (The departure is conservative)** · `prop:gaussian:conservatism`

The leverage bound's effect on uncertainty is always conservative: the posterior is wider than the fixed-model posterior would be, never narrower, because the bound discards precision rather than adding it. The system therefore reports more uncertainty than a fixed-model reading would, in every direction and at every step.

The magnitude varies with how often the bound fires. In directions where it fires frequently — high-importance-weight observations in uncertain directions — the posterior may be substantially wider. In directions where it rarely fires — well-populated features in steady state — the posterior is indistinguishable from the fixed-model posterior.

Every downstream consumer of uncertainty inherits the conservatism: the blend weight, the crossover intervals, decision fragility. Because the direction is guaranteed and the magnitude is not, the magnitude is measured rather than assumed: the empirical coverage diagnostic (`alg:monitoring:empirical-coverage`) reports the realised inflation factor, so that a host can read how conservative the reported uncertainty currently is instead of inferring it from this proposition.

**Data (Binding frequency)** · `data:gaussian:binding-frequency`

Measured rates at which the leverage bound actually fires. In steady state it fires on approximately 2–8% of positive-valence labels, the amplification coming from the importance weight, and on less than 0.1% of negative-valence labels. During early convergence it fires on 30–60% of all labels.

The distribution is what makes the departure of (`cav:gaussian:fixed-model-departure`) tolerable in practice. It is concentrated in the convergence transient and in rare feature directions; after convergence with well-populated features, the departure is negligible for the point estimate and modest for the uncertainty. These are empirical figures, not guarantees, and a deployment whose label mix differs materially from the one they were measured under should expect different ones.

**Algorithm (Synchronisation monitoring)** · `alg:gaussian:synchronisation-monitor`

The dual maintenance of $B$ and $\Sigma$ accumulates floating-point error between periodic Cholesky recomputations. The dominant conditioning risk is not the per-step amplification from forgetting — modest even over a thousand steps — but the condition number of $B$ itself, which grows when feature directions receive no observations while forgetting erodes their precision exponentially. For a feature appearing in a fraction $f$ of labels the steady-state precision is approximately $f \cdot \bar{w} / (1 - \gamma)$, and $\kappa(B)$ ranges from order $10$ where every feature is well populated to $10^5$ and beyond for rare interactions between seldom co-active Sentinels.

The reading before a recomputation is taken whether or not the recomputation happens, and it is what decides whether it does. At each visit to a model on the cadence's interval — the cheap arms go to the recomputation directly (`alg:gaussian:condition-adaptive-recompute`) — compute the synchronisation error (`def:monitoring:synchronisation-error`) between the maintained covariance and the precision matrix the model holds, separate it into its two components, and read each against the threshold. Both under it ends the visit: nothing is recomputed, the reading is recorded, and the cadence is told that the visit found nothing to do. Either over it proceeds to the recomputation, which then takes the second reading, between the recomputed covariance and the precision matrix the model will hold, after it. The two differ in what follows: a residual over the threshold is drift the recomputation removes and the interval shortens for, while a prior-induced component over it is the replenishment clamp's contribution, which the recomputation absorbs and which no interval reduces. The recomputation is where the spectral floor is applied (`dec:posterior:spectral-floor`), so those two matrices differ by the floor's own addition and by nothing else, and both readings are against a matrix the model actually holds — before and after respectively. What is excluded is a reading against a matrix the model will never hold: a factorisation that shifts internally and reports its own answer against the shifted copy certifies only that its derivation was consistent. The quantity is the Frobenius norm of the departure of a product from the identity, as its definition states; this chapter cites that definition rather than restating it, so that the monitored quantity and the reported quantity cannot drift apart.

The second reading decides adoption. The recomputed covariance replaces the maintained one only where the reading after is no worse than the reading before and is below the threshold, and otherwise the maintained covariance is kept and the interval goes to its floor. What that failure means follows from when the recomputation ran: it ran because the first reading found real drift, so a recomputed pair that is no better is a model whose drift is real and whose fresh inverse is worse than the pair it holds — an alarm rather than a routine decline (`dec:posterior:measured-adoption`), (`dec:posterior:adaptive-cadence`). The precision matrix and the covariance are adopted together or not at all, because the floor the recomputation applied belongs to both. Adoption on the measurement rather than on the factorisation's verdict is what separates a covariance that is the inverse of the matrix the model will hold from one that is the inverse of some other matrix: a factorisation that succeeded under a further internal shift reports success and returns that matrix's inverse, and the reading after says so. Accepting one reading equal to the other rather than requiring a strict improvement is deliberate — a recomputation of a matrix that has not moved reproduces the covariance already held, and that is not a worse answer.

At each visit, separate the reading into the part the prior put there and the part the arithmetic did, and report all three figures — the measured reading, the prior-induced component, and the residual between them floored at zero — in the health snapshot, together with the resolution the reading carries and whether the residual stood at or below it (`def:monitoring:synchronisation-error`). Where either component exceeds the threshold, recompute; where the residual was the one that exceeded it and the recomputation is adopted, shorten the interval. The separation is what makes the shortening mean something: the prior-induced component is the replenishment clamp's contribution since the last adopted recomputation seen through the covariance, it is not drift, and a recomputation cannot reduce it, so a cadence reacting to the whole reading shortens for ever on a model whose coordinates rest at the floor. The alternative repair is to move the clamp to the recomputation as the spectral floor is, which would make the pair consistent by construction; it is refused because the clamp is required after each forgetting step (`req:gaussian:prior-replenishment-floor`) and deferring it lets a coordinate decay below the floor within an interval. The monitor exists to make the drift visible before it is large enough to matter, so a monitored value that never reaches a report is a monitor that has not been built.

**Algorithm (Condition-adaptive recomputation)** · `alg:gaussian:condition-adaptive-recompute`

Rather than recomputing on a fixed interval, let the conditioning set the interval: recompute when the labels absorbed reach the interval the last recomputation's outcome fixed, or before that when the matrix has moved away from the state that recomputation certified. A fixed interval is either too frequent for a well-conditioned model or too rare for an ill-conditioned one, and which of the two a deployment has is a property of its feature occupancy rather than of its schedule.

What the per-label test fires is a measurement, and the recomputation follows only where that measurement finds drift worth acting on (`dec:posterior:recomputation-trigger`). The test is $O(p)$ and every part of it is relative to the last visit, so that a visit — measurement or recomputation alike — resets everything the test can fire on. Resetting on the measurement is what keeps the arms usable now that most visits stop there: an arm relative to a record only a recomputation could write would fire on every label of a model whose measurements keep finding it healthy, which is the same trap in a new place. The cheap diagonal ratio (`def:monitoring:condition-number`) is compared against the ratio recorded at that recomputation, and growth past it by a factor $\kappa_\text{growth}$, default 2, calls for another; the count of dimensions at the replenishment floor is compared against the count recorded there (`req:monitoring:replenishment-floor`), and a change calls for another, because the evidence structure the recomputation's spectrum was read from has moved. A diagonal entry that is not finite or not positive recomputes immediately, which is the one absolute reading and is absolute because it is not a matter of degree.

A test against a fixed threshold is what this replaces, and it is unusable for a structural reason rather than a tuning one. The recomputation rebuilds the covariance from the precision matrix and does not alter the precision matrix, so any test of a property of that matrix against a constant, once satisfied, stays satisfied through every recomputation it calls for. With the replenishment floor pinning the ratio's denominator, that state is reached as soon as one dimension rests at the floor, and the engine then recomputes on every label for the rest of the deployment's life. Substituting the true condition number for the ratio changes nothing: the trap is the fixed threshold on an unchanged quantity, not the reading's crudeness.

The true condition number is read at the recomputation, where the decomposition it needs is already being paid for, and it is the reported conditioning of the model. The cheap ratio is a trigger input and a published lower bound, never the reported conditioning (`req:monitoring:conditioning-bound-named`).

**Requirement (Prior replenishment floor)** · `req:gaussian:prior-replenishment-floor`

After each forgetting step, clamp each diagonal entry of the precision matrix at a floor: $B_{jj} \geq \lambda_\text{floor}$, at a default $\lambda_\text{floor} = 0.01 \cdot \lambda_\text{prior}$. This arrests coordinatewise diagonal decay. It does not bound $\kappa(B)$: equal positive diagonal entries can coexist with an arbitrarily small eigenvalue through the off-diagonal structure.

The floor injects a small amount of artificial diagonal precision, and the injection is deliberate rather than tolerated. It is equivalent to a weak coordinatewise prior that does not fully decay: a feature coordinate included in the model represents a structural prior belief that it might matter, and no amount of elapsed time without observations is evidence that it does not. A spectral conditioning guarantee still requires a spectral guard.
