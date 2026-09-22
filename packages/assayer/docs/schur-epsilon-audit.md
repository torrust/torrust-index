# The Schur epsilon is a stability factor, not an accuracy budget · `rep:posterior:schur-regularisation-meaning`

This report is evidence for a decision on the specification-side meaning of $\varepsilon_\text{Schur}$. It audits the regularised Schur complement, its interaction with prior scale and forgetting, the condition and spectrum guards, the approximation introduced by the offset, and the implementation surface that existed before the concurrent factor-based integration.

The report mints no identity. It cites existing heads under the label calculus. Source observations are at base commit `cc8105e3734988cabe1a36a0a4e464afd25bd1f4`. The truth repairs described below landed on this lane after that base. The concurrently announced change to store the factor and derive the offset from live prior precision is discussed only as an anticipated design; it is not treated as present-base evidence.

The headline is: **the default $10^{-4}$ is an arbitrary-but-numerically-safe engineering choice in ordinary-scale cases, not a specified accuracy guarantee.** At the reference prior it removes about $0.01\%$ of a correction carried by a healthy prior-scale eigenmode and about $0.99\%$ at the tabulated diagonal replenishment scale. Those percentages do not bound the error of the surviving precision. A valid, guard-admitted scalar example makes the default surviving precision about one hundred and one times the exact marginal precision. The offset always biases in that direction: more precision and a narrower covariance.

## Findings for ruling · `sec:posterior:schur-ruling-findings`

| Finding | Decisive evidence | Verdict and smallest recommendation |
| --- | --- | --- |
| The commissioned occurrence premise is false in detail | At base, the factor and value appeared in (`alg:gaussian:regularised-schur`), (`tab:config:risk-model`), and the later audit at (`rem:assayer:model-linalg-prose-defects`). The assembled `docs/spec.md` mirrors the first two and is not independent authority. The source comment at `src/model/marginalise.rs:118-119` states the product and default absolute offset, but does not state that the factor itself is $10^{-4}$. | Correct the census: three authored prose statements of the value, two generated mirrors, and no source-comment statement of the factor's value. No default change follows from the census. |
| A narrow sensitivity analysis already exists | (`test:integration:adr-formula-error-scales-with-h-not-delta`) sweeps $\delta$ from $10^{-3}$ to $10^{-11}$ and shows a covariance-extraction error shrinking with it; (`test:integration:error-pattern-holds-at-p20`) repeats the pattern at larger width. Neither converts to $\varepsilon_\text{Schur}$, covers forgetting or guards, or sizes the default. | Reject “no sensitivity analysis anywhere”; retain “no general epsilon sensitivity analysis or error budget”. Credit the fixture as corroboration, not as a default justification. |
| Regularisation has a one-sided statistical bias | For SPD $A=B_{rr}$ and $C=B_{rk}$, $S_\delta-S_0=\delta C^\top A^{-1}(A+\delta I)^{-1}C\succeq0$. Hence $S_\delta^{-1}\preceq S_0^{-1}$. | Add or retain an explicit caveat that regularisation is overconfident relative to the exact marginal; never describe it as uncertainty-conservative. The Gaussian foundation now states the direction. |
| Epsilon alone is not an accuracy budget | The absolute bound contains the removed-block floor and coupling, $\lVert S_\delta-S_0\rVert\leq\delta\lVert C\rVert^2/[a(a+\delta)]$ for $a=\lambda_{\min}(A)$. Relative error also depends on $s=\lambda_{\min}(S_0)$. The near-boundary example below has healthy scalar $A$, passes the raw-block guard, and amplifies the default precision by about two decimal orders. | Keep the default only provisionally. Add a requirement sized in $a$, $\lVert C\rVert$, and $s$, or expose a measured bias proxy; do not claim a universal tolerance from $\varepsilon_\text{Schur}$ alone. |
| The stated forgetting motivation was algebraically false | Uniform decay $A\mapsto qA$, $C\mapsto qC$ makes the exact correction $qC^\top A^{-1}C$, not an order-one term. A diagonal floor changes the later correction to a different, often quadratic decay. | Truth repaired in (`alg:gaussian:regularised-schur`). Retain the numerical motivation as factorisation scale, conditioning, and accumulated drift, not a divergent correction. |
| The replenishment floor is not a spectral floor | A lower bound on every diagonal does not lower-bound $\lambda_{\min}$; an equal-diagonal matrix can be arbitrarily ill-conditioned. Current base `BayesianModel::apply_time_decay` also scales precision without applying the floor required after each forgetting step by (`req:gaussian:prior-replenishment-floor`). | The false spectral bound is repaired. Separately, make bulk decay apply the normative floor or qualify the requirement; this lane touched no Rust. |
| The default is mild only under stated scale assumptions | At $a=\lambda_\text{prior}=0.1$, the default loses $0.009999\%$ of that eigenmode's correction. If a genuine spectral floor gave $a=0.001$, it would lose $0.990099\%$. Neither premise is universal. | Classify $10^{-4}$ as arbitrary-but-safe for ordinary prior/floor modes. Do not call it mathematically justified until the corpus names an allowed loss and demonstrates the spectral premises. |
| The guard and epsilon have no scale-free joint guarantee | For a true raw condition number $10^8$ with $\lambda_{\max}(A)=\lambda_\text{prior}$, the default makes $\kappa(A+\delta I)=10^4$. Scaling $A$ without scaling $\lambda_\text{prior}$ removes that benefit. At base the implementation tests a diagonal ratio, which is only a lower bound on spectral condition. | Specify whether the guard protects raw information quality or regularised solvability. For the latter, guard $A+\delta I$ spectrally; for the former, retain a raw guard but name it semantic. Do not call a diagonal ratio the condition number. |
| The positivity check cannot certify small bias | Since $S_\delta\succeq S_0$, regularisation moves the least eigenvalue toward acceptance. The base general path adds a finite-resolution floor of $8k\varepsilon_\text{mach}\lambda_{\max}$ while (`req:gaussian:positive-definiteness`) says strictly positive; the rank-one path checks only a diagonal minimum. | State the finite-resolution rule in the spec and either apply the spectral rule to rank one or specify a proved fast-path exception. Keep the check as an arithmetic-integrity guard, not an approximation oracle. |
| Fallback is the infinite-epsilon endpoint | As $\delta\to\infty$, the correction vanishes and $S_\delta\to B_{kk}$, the same precision the guard and verification fallbacks adopt. It is the maximum attenuation, not an exact marginal. | Describe fallback as a potentially overconfident degradation and report it. The correction outcome should remain visible through health rather than being treated as a harmless exact path. |
| Current-base configuration is default-only | `AssayerConfig` publicly exposes and serialises `schur`, but `BayesianModel::marginalise` and its rank-one path both construct `SchurConfig::default()` at `src/model/bayesian.rs:370,422`. Builder validation reaches Cholesky regularisation, not `config.schur`. A host change to $\lambda_\text{prior}$ therefore changes the effective factor $\delta/\lambda_\text{prior}$, and a host change to `config.schur` has no effect. | The configuration note is truthfully qualified. Current base is conformant only at defaults; do not describe epsilon as a working host knob. |
| The announced live-prior integration is directionally right but unverified here | Storing a dimensionless factor and deriving $\delta=\lambda_\text{prior}\varepsilon_\text{Schur}$ restores scale covariance if the live model prior and the configured factor reach every call site. That code was concurrent and absent from the audited base. | After integration, verify threading, finite-positive validation, diagnostics, tests with non-default prior precision, and serialised-config migration. Keep this row distinct from present evidence until that verification is done. |
| The input domain is underspecified for machine values | The table says only $\varepsilon_\text{Schur}>0$. Zero removes the lift, negative values can cross an eigenvalue and destroy factorability, non-finite values poison the matrix, and very large values approximate fallback. | Require finite and strictly positive input. Add an upper limit only from an explicit correction-loss budget; no universal upper bound follows from this audit. |
| Repeated lifecycle bias is unbounded by the corpus | Each applied regularisation adds precision relative to that event's exact marginal, while each fallback retains all of $B_{kk}$. The previous exactness promise claimed no compounding although the corpus supplied no sum, product, or observed bound. | Exactness claims are repaired. Add cumulative monitoring or an operational retraining/recompute criterion before restoring any repeated-lifecycle accuracy guarantee. |

## What the factor changes · `sec:posterior:schur-factor-effect`

Write

$$
A=B_{rr},\qquad C=B_{rk},\qquad
S_0=B_{kk}-C^\top A^{-1}C,
$$

and let

$$
S_\delta=B_{kk}-C^\top(A+\delta I)^{-1}C,
\qquad \delta=\lambda_\text{prior}\varepsilon_\text{Schur}.
$$

For SPD $A$, the resolvent identity gives

$$
E_\delta=S_\delta-S_0
=\delta C^\top A^{-1}(A+\delta I)^{-1}C\succeq0.
$$

Three consequences settle most of the audit.

First, $E_\delta$ is monotone in the sense relevant here: increasing the offset attenuates every correction eigenmode and moves the answer toward $B_{kk}$. In an eigenmode of $A$ with eigenvalue $a_i$, the fraction of that mode's correction lost to regularisation is

$$
L_i(\delta)=1-\frac{a_i}{a_i+\delta}
=\frac{\delta}{a_i+\delta}.
$$

Second, the direction is not conservative for uncertainty. Since $S_\delta\succeq S_0\succ0$, inversion reverses the order and gives $S_\delta^{-1}\preceq S_0^{-1}$. The regularised posterior is narrower than the exact marginal. The positivity verification cannot see this error because it makes the verified matrix more positive.

Third, a dimensionless factor is the right scaling form. If every precision and $\lambda_\text{prior}$ are rescaled by $r>0$, then $\delta$ also scales by $r$ and $S_\delta$ scales by $r$ without changing a relative decision. Holding an absolute $\delta$ while changing the prior destroys that covariance. The announced live-prior design repairs this property if it is actually threaded.

For $a=\lambda_{\min}(A)>0$, the ordinary spectral-norm bound is

$$
\lVert E_\delta\rVert_2
\leq
\frac{\delta\lVert C\rVert_2^2}{a(a+\delta)}.
$$

If $s=\lambda_{\min}(S_0)>0$, the covariance error additionally obeys the coarse bound

$$
\lVert S_0^{-1}-S_\delta^{-1}\rVert_2
\leq \frac{\lVert E_\delta\rVert_2}{s^2}.
$$

The factors $a$, $\lVert C\rVert$, and $s$ are why the bare statement “error is of the order of the regularisation” is not a bound. The narrow rank-one fixture (`test:integration:woodbury-is-the-exact-rank1-correction`) correctly exposes a coefficient proportional to $\delta$ for its matrices; the coefficient contains the pivot and coupling and is not uniformly one.

## The headline epsilon sweep · `sec:posterior:schur-epsilon-sweep`

The table evaluates the exact mode-loss formula across the commissioned range. The healthy column uses $a=0.1$, the default prior precision. The floor column uses $a=0.001$, but is conditional on a true eigenvalue floor, which the diagonal clamp does not establish. The final column starts from a true raw condition number of $10^8$ with $\lambda_{\max}(A)=0.1$ and reports $(\lambda_{\max}+\delta)/(\lambda_{\min}+\delta)$.

| $\varepsilon_\text{Schur}$ | $\delta$ | correction lost at $a=0.1$ | correction lost at $a=0.001$ | regularised $\kappa$ from raw $10^8$ |
| ---: | ---: | ---: | ---: | ---: |
| $10^{-2}$ | $10^{-3}$ | $0.990099\%$ | $50\%$ | $101$ |
| $10^{-3}$ | $10^{-4}$ | $0.0999001\%$ | $9.090909\%$ | $1.00099\times10^3$ |
| **$10^{-4}$** | **$10^{-5}$** | **$0.009999\%$** | **$0.990099\%$** | **$10^4$** |
| $10^{-5}$ | $10^{-6}$ | $0.00099999\%$ | $0.0999001\%$ | $9.99011\times10^4$ |
| $10^{-6}$ | $10^{-7}$ | $0.0000999999\%$ | $0.009999\%$ | $9.901\times10^5$ |
| $10^{-7}$ | $10^{-8}$ | $0.00001\%$ | $0.00099999\%$ | $9.09091\times10^6$ |
| $10^{-8}$ | $10^{-9}$ | $0.000001\%$ | $0.0001\%$ | $5.0\times10^7$ |

This curve explains why $10^{-4}$ looks safe in routine examples: its loss is small when every relevant $a_i$ is a substantial fraction of prior precision. It also explains why $10^{-2}$ is not a harmless upper endpoint at the floor: half of that correction disappears. None of the rows bounds relative error in $S_\delta$, because the exact Schur margin $s$ is absent.

The endpoints identify the semantics. At $\varepsilon=0$, $S_\delta=S_0$ for invertible $A$, but the lift supplies no help at a singular or numerically lost pivot. As $\varepsilon\to\infty$, $S_\delta\to B_{kk}$ and regularisation becomes the fallback. A negative offset crosses a pole at $\delta=-\lambda_i(A)$; infinity and not-a-number have no matrix semantics.

## Healthy does not mean small relative bias · `sec:posterior:schur-guarded-bias`

A scalar removed block is enough to falsify a universal interpretation of the default. Take

$$
A=0.1,\qquad B_{kk}=1,\qquad
C=\sqrt{0.0999999}.
$$

The parent block is SPD, its exact Schur complement is $S_0=10^{-6}$, its condition number is about $1.21\times10^7$, and the removed scalar's condition number is one. The $10^8$ removed-block guard therefore admits it. Every regularised answer remains positive, so spectrum verification admits it too.

| $\varepsilon_\text{Schur}$ | $S_\delta/S_0$ | regularised covariance divided by exact covariance |
| ---: | ---: | ---: |
| $10^{-2}$ | $9901.9802$ | $0.00010099$ |
| $10^{-3}$ | $1000$ | $0.001$ |
| **$10^{-4}$** | **$100.989901$** | **$0.00990198$** |
| $10^{-5}$ | $10.99989$ | $0.09091$ |
| $10^{-6}$ | $1.999998$ | $0.5000005$ |
| $10^{-7}$ | $1.09999989$ | $0.909091$ |
| $10^{-8}$ | $1.00999999$ | $0.990099$ |

The example is deliberately near the exact Schur boundary. That is not a cheat: the spec has no lower bound on that margin, and the output verification checks only that the regularised result is positive. The example says neither that deployments normally occupy this geometry nor that $10^{-4}$ must be changed. It says the current inputs and guards cannot support a universal accuracy claim.

## Forgetting changes absolute and relative error differently · `sec:posterior:schur-forgetting-error`

Let a valid scalar family begin with prior-scale pivot $a_0=0.1$ and exact correction $R_0=0.4$. If uniform forgetting scales both the pivot and coupling by $q$, then

$$
R_0(q)=qR_0,\qquad
R_\delta(q)=qR_0\frac{qa_0}{qa_0+\delta}.
$$

The correction therefore decays with $q$. At the default factor its relative attenuation crosses one half when $q=10^{-4}$, but its absolute size has already fallen by the same factor. If the specified diagonal floor intervenes at $0.001$, the pivot stops at the floor while the coupling continues to decay; in this scalar construction the exact correction then decays as $q^2$ and the regularisation loss stays near $0.99\%$.

| $q$ | unfloored exact correction | unfloored loss | floored exact correction | floored loss |
| ---: | ---: | ---: | ---: | ---: |
| $1$ | $0.4$ | $0.009999\%$ | $0.4$ | $0.009999\%$ |
| $10^{-2}$ | $0.004$ | $0.990099\%$ | $0.004$ | $0.990099\%$ |
| $10^{-4}$ | $4\times10^{-5}$ | $50\%$ | $4\times10^{-7}$ | $0.990099\%$ |
| $10^{-6}$ | $4\times10^{-7}$ | $99.009901\%$ | $4\times10^{-11}$ | $0.990099\%$ |
| $10^{-8}$ | $4\times10^{-9}$ | $99.990001\%$ | $4\times10^{-15}$ | $0.990099\%$ |

With the tabulated label-indexed rates, $q=10^{-2}$ arrives after roughly nine thousand operational labels or twenty-three thousand sister labels; the unfloored $q=10^{-4}$ crossover arrives after roughly eighteen thousand or forty-six thousand respectively (`tab:risk:forgetting-rates`). These are not deployment forecasts. Observations replenish some blocks, coupling need not decay uniformly, time decay composes with label decay, and the floor changes the trajectory.

The present-base distinction is important. The label update applies the diagonal clamp, while `BayesianModel::apply_time_decay` scales both matrices and does not clamp. Thus the spec's “after each forgetting step” floor and the implementation's offline bulk-decay regime are not the same system. The regularisation may still be useful in the latter; the spec cannot use the floor and the unfloored asymptote in the same proof.

## Guard, regularisation, and verification are three different decisions · `sec:posterior:schur-decision-separation`

For $a_{\min}$ and $a_{\max}$, regularisation changes the true condition number to

$$
\kappa(A+\delta I)
=\frac{a_{\max}+\delta}{a_{\min}+\delta}.
$$

At the reference scale, a raw $10^8$ boundary becomes $10^4$ under the default offset. A pre-regularisation guard at $10^8$ therefore refuses some matrices that the regularised solve could make much easier. That can still be coherent if the raw guard is a statement about information quality rather than arithmetic; the spec currently calls it an alternative numerical guard and does not make that distinction.

The present implementation adds a second mismatch. It computes $\max_i A_{ii}/\min_i A_{ii}$, a lower bound on spectral condition, before regularisation. For

$$
A=r\begin{pmatrix}1&1-10^{-12}\\1-10^{-12}&1\end{pmatrix},
$$

the diagonal ratio is one while the raw spectral condition is about $2\times10^{12}$. At $r=1$, the default offset reduces it to about $2\times10^5$; at $r=10^8$, the same offset leaves it near $1.82\times10^{12}$. The factor is scale-covariant only when the block scale and the prior scale move together.

The output verification protects another matrix, $S_\delta$. The general path's resolution floor is $8k\varepsilon_\text{mach}\lambda_{\max}(S_\delta)$ (`const:assayer:schur-verification-resolution`). At the reference surviving width near $570$, that resolves a condition number near $10^{12}$. This is not inconsistent with a $10^8$ raw-block guard: the matrices and purposes differ. It is also not a joint theorem. No bound in the corpus maps one threshold to the other, and neither bounds $E_\delta$.

The requirement says the minimum eigenvalue must be strictly positive (`req:gaussian:positive-definiteness`). The finite floor is the implementable version of that decision and should be stated. The rank-one path instead checks only its minimum diagonal, despite the same requirement's explicit argument that a diagonal test is insufficient. An SPD parent and positive offset prove the mathematical rank-one result positive, but a diagonal scan is not a backward-error analysis of the computed result. One contract should be chosen, with the exception stated if the quadratic fast path is worth it.

## Present configuration and the anticipated integration · `sec:posterior:schur-configuration-integration`

At the audited base, `SchurConfig` stores absolute `delta` and a condition guard threshold. It is a public, serialisable field of `AssayerConfig`, but production marginalisation does not receive that field. Both general and rank-one paths construct `SchurConfig::default()`, and the builder does not validate the exposed Schur values. The shipped defaults happen to agree:

$$0.1\times10^{-4}=10^{-5}.$$

That equality is load-bearing. With the present fixed offset, setting $\lambda_\text{prior}=0.01$ silently yields an effective factor $10^{-3}$; setting it to $1$ yields $10^{-5}$. Changing `config.schur.delta` or its guard threshold changes neither production call site. The qualification now recorded in (`tab:config:risk-model`) is therefore a current-base conformance statement, not a criticism of the announced design.

The announced integration—store $\varepsilon_\text{Schur}$ and derive $\delta$ from the live $\lambda_\text{prior}$—is the right dimensional repair. Its merge should not be inferred from this report. Verification after integration should establish all of the following:

- the configured factor reaches general and rank-one marginalisation;
- the live prior belonging to the model, not another default, forms the product;
- the factor and threshold are finite and in their specified domains;
- non-default prior tests preserve the same effective factor;
- diagnostics expose enough scale to reconstruct the applied offset; and
- serialised configurations that named the old absolute field receive an explicit compatibility or rejection policy.

## Numerical method and reproducibility · `sec:posterior:schur-numerical-method`

The sweep, decay family, near-boundary example, guard counterexample, and verification resolution were evaluated with Python's standard library. The tables are direct evaluations of the displayed closed forms; there is no random sampling, fitted parameter, or hidden dataset. A separate fixed anisotropic two-mode construction, made SPD by choosing positive $A$ and positive exact $S_0$ before forming $B_{kk}$, confirmed that the computed eigenvalues of $E_\delta$ remain non-negative across the sweep.

No script is committed. The equations, parameters, and every reported column are present here, so a retained script would duplicate rather than improve the reproducibility authority. The existing compiled integration fixtures remain useful corroboration for rank one, but this lane ran no Cargo command as required.

## Truth repairs made by the audit · `sec:posterior:schur-truth-repairs`

The following were false statements rather than open recommendations, so the lane repaired them and regenerated `docs/spec.md` with the pinned assembly tool.

| False statement or implication | Disposition |
| --- | --- |
| Uniform decay makes a same-rate Schur correction remain order one | Replaced with the $q$-scaled identity and the distinct post-floor regime in (`alg:gaussian:regularised-schur`). |
| Regularised and fallback marginalisation are exact and introduce no compounding approximation | Qualified across (`thm:gaussian:marginalisation`), (`thm:gaussian:two-level-guarantee`), (`inv:guarantee:structural-exactness`), (`inv:guarantee:axis-lifecycle`), (`alg:axis:lifecycle`), and (`alg:registry:sentinel-deregistration`). |
| A low-rank covariance discrepancy is bounded by the bare offset | Replaced with dependence on removed-block spectrum and coupling in (`rem:gaussian:dual-tracking`). |
| A lower bound on precision diagonals bounds spectral condition number | Replaced with the coordinatewise-only guarantee in (`req:gaussian:prior-replenishment-floor`). |
| The absolute source offset and relative configured factor agree without qualification | Restricted to the shipped defaults in (`tab:config:risk-model`), with the unused current-base configuration stated. |

No default, guard threshold, implementation, record, test, or backlog entry was changed. The smallest next specification amendment is an accuracy caveat or requirement that names an admissible correction loss or Schur-margin error. The smallest next implementation verification is a non-default-prior test of the concurrent factor integration.

## Recommendation · `sec:posterior:schur-recommendation`

Retain $\varepsilon_\text{Schur}=10^{-4}$ provisionally. It occupies a sensible middle of the numerical curve at reference scale: materially stronger conditioning than $10^{-8}$, far less correction attenuation than $10^{-2}$, and modest loss in ordinary prior/floor eigenmodes. The available corpus does not justify changing it to another point.

Do not ratify it as an accuracy guarantee. Ratify instead the factor form, the one-sided overconfidence caveat, finite-positive validation, and one measurable error budget involving removed-block spectrum, coupling, and exact or bounded Schur margin. Decide separately whether the condition guard is about raw information quality or regularised numerical solvability. Finally, make the general and rank-one verification contracts agree, and expose fallback and attenuation often enough that repeated lifecycle approximation is observable.
