# Rank-one verification needs a floating-point certificate · `rep:posterior:rank-one-verification-certificate`

This study settles the fast-path exception left open by the Schur epsilon audit.  It asks whether the scalar Schur formula makes the rank-one path's diagonal check sufficient, and it distinguishes the exact theorem from the binary64 computation that the implementation actually adopts.

The report mints no identity.  It cites existing heads in this corpus.  Source observations are at base commit `ca1f959ef11b41792675913bebedacbb7deffe8d`. The concurrent ERRTRACK lane is changing the general path's guard and diagnostics; that work is noted only here and is neither assumed nor chased.

The headline is: **the exception is sound in exact arithmetic under an SPD parent and a non-negative offset, but it is unsound for the implemented binary64 computation without a quantitative spectral margin.**  A concrete binary64 parent below is SPD as an exact matrix of its represented values.  The exact regularised result is SPD.  The implemented operation order nevertheless produces a result with two positive diagonal entries and a negative eigenvalue, so the current rank-one check adopts it.

## Findings for ruling · `sec:posterior:rank-one-ruling-findings`

| Finding | Decisive evidence | Verdict |
| --- | --- | --- |
| Exact rank-one marginalisation cannot break an SPD parent under the normal offset | For parent block $B=\left(\begin{smallmatrix}A&u\\u^\top&a\end{smallmatrix}\right)\succ0$, $S_0=A-uu^\top/a\succ0$.  With $\delta\geq0$, $S_\delta=S_0+\delta uu^\top/[a(a+\delta)]\succ0$. | The source comment has a valid exact-arithmetic theorem behind it. |
| Interlacing reduces the exact risk to one eigenvalue | For $S_\delta=A-cuu^\top$ with $A\succ0$ and $c>0$, negative-rank-one interlacing leaves every eigenvalue except the least strictly positive.  The determinant lemma decides the remaining sign by one scalar. | A scalar certificate is possible only when its premises and arithmetic margin are certified. |
| The implemented diagonal check is not sufficient even for an SPD represented parent | The construction below has an exact parent Schur margin of about $8.20\times10^{-19}$.  Exact regularised arithmetic leaves a least eigenvalue about $9.62\times10^{-19}$; the implemented binary64 operations return about $-4.12\times10^{-19}$ while the minimum diagonal is about $1.00\times10^{-7}$. | The exception is presently unsound.  This is a rounding counterexample, not an invalid-parent counterexample. |
| A positive denominator is not the missing margin | The counterexample has $a+\delta=1.00001$.  Its weak direction is almost orthogonal to $u$, so shrinking the rank-one correction does not protect that direction from formation error. | Denominator sign is part of the exact proof, not a floating-point output certificate. |
| The implementation establishes neither exact-proof premise at the adoption point | The path checks a positive finite scalar pivot and denominator, then scans the result's diagonal.  It neither verifies the parent spectrum nor bounds the rank-one formation error.  Checkpoint restoration accepts a symmetric precision without an SPD factorisation. | Conditional soundness would require a certified parent spectral lower bound and a forward-error bound whose gap clears the verification resolution. |
| The fixture's regularisation-only wording is too strong | Its Woodbury coefficient also contains the pivot and coupling quadratic form, as (`rem:gaussian:dual-tracking`) now states. | Say that the fixture demonstrates exactness and observed delta-scaling for those matrices, not a universal delta-only error bound. |

## The exact theorem · `sec:posterior:rank-one-exact-theorem`

Permute the removed dimension last and write the represented parent as

$$
B=
\begin{pmatrix}
A & u\\
u^\top & a
\end{pmatrix},
\qquad
A=B_{kk},\quad u=B_{kr},\quad a=B_{rr}.
$$

The scalar path targets

$$
S_\delta=A-cuu^\top,
\qquad
c=\frac{1}{a+\delta}.
$$

If $B\succ0$, then $a>0$, $A\succ0$, and the scalar Schur complement is

$$
S_0=A-\frac{1}{a}uu^\top\succ0.
$$

For finite $\delta\geq0$,

$$
S_\delta
=S_0+
\left(\frac{1}{a}-\frac{1}{a+\delta}\right)uu^\top
=S_0+\frac{\delta}{a(a+\delta)}uu^\top
\succ0.
$$

This is the exact proof anticipated by (`req:gaussian:positive-definiteness`).  Zero offset is also safe under an SPD parent; a positive offset merely adds a positive-semidefinite term.  The proof does not survive a non-SPD parent, a negative offset, a non-positive denominator, or non-finite arithmetic.

The same result can be read through interlacing.  Let $\lambda_1\leq\cdots\leq\lambda_k$ be the eigenvalues of $A$ and $\mu_1\leq\cdots\leq\mu_k$ those of $A-cuu^\top$, with $c>0$.  Rank-one interlacing gives

$$
\mu_1\leq\lambda_1\leq\mu_2\leq\lambda_2
\leq\cdots\leq\mu_k\leq\lambda_k.
$$

Because $A\succ0$, only $\mu_1$ can be non-positive.  The matrix determinant lemma isolates its sign.  For

$$
h=u^\top A^{-1}u,
$$

it gives

$$
\det(S_\delta)=\det(A)(1-ch),
\qquad
1-ch=\frac{a+\delta-h}{a+\delta}.
$$

The parent condition is equivalently $a-h>0$, so $\delta\geq0$ makes the last ratio positive.  Thus interlacing plus the determinant sign proves the same result and explains what a scalar certificate would have to know.  Computing $h$ from $A$ requires a solve unless a trustworthy factor or inverse relation is already available.

The determinant also gives the exact failure boundary.  Whenever $a+\delta>0$, the regularised result is SPD exactly when $a+\delta-h>0$.  A negative offset can therefore leave the denominator positive while crossing the output boundary once $\delta\leq h-a$.  At the boundary the one exposed eigenvalue is zero; beyond it exactly one eigenvalue is negative.  A non-SPD parent removes even the premise that the other eigenvalues are protected.

## What floating point changes · `sec:posterior:rank-one-floating-arithmetic`

The source does not perform one exact rank-one subtraction.  For each lower triangle entry it effectively computes

$$
\widehat d=\operatorname{fl}(a+\delta),\qquad
\widehat c=\operatorname{fl}(1/\widehat d),
$$

$$
\widehat q_{ij}
=\operatorname{fl}
 \left(\operatorname{fl}(\widehat c u_i)u_j\right),
\qquad
\widehat s_{ij}=\operatorname{fl}(A_{ij}-\widehat q_{ij}),
$$

then mirrors the lower triangle.  The exact matrix $cuu^\top$ has rank one and is positive semidefinite.  The entrywise rounded $\widehat Q$ need have neither rank one nor the same nullspace.  Exact rank-one interlacing therefore says nothing conclusive about the final $\widehat S=A\mathbin{-}\widehat Q$.

This is not only a theoretical loss of structure.  In the counterexample below, $\det(\widehat Q)$ is about $1.39\times10^{-18}$ rather than zero, so the rounded correction is full rank.  Its small extra component lies in the direction in which the parent has its least margin.

## An implemented-formula counterexample · `sec:posterior:rank-one-counterexample`

Take the default $\lambda_\text{prior}=0.1$ and $\varepsilon_\text{Schur}=10^{-4}$, so the source computes $\delta=10^{-5}$.  The following hexadecimal literals fix every matrix input to one exact binary64 value:

```python
a = float.fromhex("0x1.0000000000000p+0")
u0 = float.fromhex("0x1.0000000000000p+0")
u1 = float.fromhex("0x1.999999999999ap-4")

A00 = float.fromhex("0x1.0000000000001p+0")
A11 = float.fromhex("0x1.47ae147ae147ep-7")
A10 = float.fromhex("0x1.999999999999cp-4")
```

In decimal, the parent is

$$
B=
\begin{pmatrix}
1.0000000000000002 & 0.10000000000000003 & 1\\
0.10000000000000003 & 0.010000000000000005 & 0.1\\
1 & 0.1 & 1
\end{pmatrix}.
$$

This parent is SPD as an exact real matrix of those binary64 values.  Since $a=1$, its exact scalar Schur complement is $R=A-uu^\top$.  Standard-library `Fraction.from_float` arithmetic gives

| Certificate for $R$ | Exact-input value rounded for display |
| --- | ---: |
| $R_{00}$ | $2.2204460492503131\times10^{-16}$ |
| $R_{11}$ | $4.3021142204224816\times10^{-18}$ |
| $R_{10}$ | $2.7755575615628914\times10^{-17}$ |
| $\det(R)$ | $1.8488927466117464\times10^{-34}$ |
| $\lambda_{\min}(R)$ | $8.1981026378623755\times10^{-19}$ |

The first diagonal and determinant are strictly positive, so the symmetric two-dimensional $R$ is SPD; the scalar Schur criterion then proves $B\succ0$. This avoids using a floating eigensolver to assume the very premise being tested.

Now follow the source order exactly:

```python
delta = 0.1 * 1e-4
c = 1.0 / (a + delta)
q00, q11, q10 = (c*u0)*u0, (c*u1)*u1, (c*u1)*u0
s00, s11, s10 = A00-q00, A11-q11, A10-q10
```

The results are:

| Entry | $A$ | $\widehat Q$ | $\widehat S$ |
| --- | ---: | ---: | ---: |
| $00$ | $1.0000000000000002$ | $0.9999900000999989$ | $9.999900001278483\times10^{-6}$ |
| $11$ | $0.010000000000000005$ | $0.00999990000099999$ | $9.999900001458895\times10^{-8}$ |
| $10$ | $0.10000000000000003$ | $0.0999990000099999$ | $9.999900001389506\times10^{-7}$ |

Treating the resulting binary64 entries as exact values, again with standard-library rational and decimal arithmetic, gives

| Verdict quantity | Value |
| --- | ---: |
| exact-arithmetic $\lambda_{\min}(S_\delta)$ | $+9.6182687776616104\times10^{-19}$ |
| implemented $\min_i\widehat S_{ii}$ | $+9.999900001458895\times10^{-8}$ |
| implemented $\det(\widehat S)$ | $-4.1632947096364487\times10^{-24}$ |
| implemented $\lambda_{\min}(\widehat S)$ | $-4.1221151905610734\times10^{-19}$ |
| shared spectral resolution at width two | $3.588204933639888\times10^{-20}$ |

The rank-one path takes its condition estimate as one because $a$ is positive and finite.  Its denominator is the comfortably positive $1.00001$.  Its diagonal scan returns true, so `marginalise_rank1` adopts $\widehat S$ and extracts the maintained covariance block.  The general spectral rule would refuse the negative eigenvalue under (`const:assayer:schur-verification-resolution`).

The example is deliberately close to a spectral boundary.  That is the domain the verification exists to police, and the corpus expressly permits an ill-conditioned posterior (`cav:posterior:conditioning`).  More importantly, the implementation states no lower spectral margin as a precondition.  The checkpoint restoration route accepts a symmetric precision matrix without an SPD factorisation, and the rank-one path itself tests neither the parent's spectrum nor its margin.

## The precise missing precondition · `sec:posterior:rank-one-missing-precondition`

Let $m>0$ be a certified live lower bound on $\lambda_{\min}(B)$, let $E=\widehat S-S_\delta$ be the complete formation error, and let $\tau$ be the finite-resolution margin used by the output contract.  The exact Schur complement satisfies

$$
\lambda_{\min}(S_0)\geq\lambda_{\min}(B)\geq m.
$$

One way to see the first inequality is that $S_0^{-1}$ is a principal block of $B^{-1}$, so its largest eigenvalue cannot exceed that of $B^{-1}$.  For $\delta\geq0$, $S_\delta\succeq S_0$.  Weyl's perturbation bound therefore gives

$$
\lambda_{\min}(\widehat S)
\geq m-\lVert E\rVert_2
\geq m-\lVert E\rVert_F.
$$

A sound floating-point exception could consequently require a proved bound $\eta\geq\lVert E\rVert_F$ and the strict margin

$$
m>\eta+\tau,
$$

together with finite inputs, $\delta\geq0$, and a finite positive denominator. The current implementation has no certified $m$ and computes no $\eta$. Positive diagonals provide neither.  This quantitative gap is the missing precondition; “SPD parent and positive denominator” alone describes the exact problem, not the computed one.

## What a cheap certificate can prove · `sec:posterior:rank-one-certificate-scope`

The determinant scalar is cheap only after expensive or unproved information has been supplied.  With $A\succ0$,

$$
g=1-cu^\top A^{-1}u>0
$$

is necessary and sufficient for $A-cuu^\top\succ0$.  A fresh factorisation and solve for $A^{-1}u$ cost $O(k^3)$, so this does not preserve the rank-one path's cost.  Exact dual tracking would give another scalar identity: if $\Sigma=B^{-1}$ and $\sigma$ is the removed coordinate's covariance diagonal, then

$$
u^\top A^{-1}u=a-\frac{1}{\sigma},
\qquad
g=\frac{\delta+1/\sigma}{a+\delta}.
$$

The rank-one path has $\sigma$ available, but the premise is unavailable. Precision and covariance deliberately drift between recomputations (`rem:gaussian:dual-tracking`), and the implementation has no rigorous residual bound from which to turn the approximate identity into an inequality.  Reading the scalar anyway would exchange one unproved exception for another.

There is a genuine certificate of the matrix actually computed that stays quadratic.  For the stored symmetric $\widehat S$, define outward-rounded row sums and Gershgorin bounds

$$
r_i\geq\sum_{j\ne i}|\widehat s_{ij}|,
\qquad
L=\min_i(\widehat s_{ii}-r_i),
\qquad
U=\max_i(\widehat s_{ii}+r_i).
$$

Gershgorin gives

$$
\lambda_{\min}(\widehat S)\geq L,
\qquad
\lambda_{\max}(\widehat S)\leq U.
$$

Thus, at width $k$, the sufficient test

$$
L>8k\varepsilon_\text{mach}\max(U,0)
$$

implies the same finite-resolution decision as the shared spectral verifier. The row sums can be accumulated while $\widehat S$ is formed, at $O(k^2)$ time and $O(k)$ extra state.  Their rounding must be bounded outward; an ordinary under-rounded sum is not a certificate.

This test is sufficient and not necessary.  Dense SPD matrices need not be strictly diagonally dominant, so an inconclusive result says nothing.  On the counterexample, the second row already has a negative lower bound because its off-diagonal magnitude exceeds its diagonal.  A sound hybrid can accept on the Gershgorin bound and call the spectral verifier otherwise.  Whether that keeps the frequent path quadratic depends on an acceptance-rate measurement the corpus does not contain.

## Cost of aligning the paths · `sec:posterior:rank-one-path-alignment-cost`

The current rank-one path forms an outer product, subtracts it, and extracts a covariance block.  Those operations are $O(k^2)$; its diagonal scan is $O(k)$. Calling `eigenvalue_min_max` computes a symmetric eigendecomposition at $O(k^3)$ and would dominate the path, changing the cost class promised for low-rank marginalisation (`tab:gaussian:operation-costs`).

The base module's existing measurement reports that spectral verification adds roughly twenty-two milliseconds at surviving width $k=570$.  A cubic extrapolation to width $637$, the surviving width when one coordinate leaves the reference model, is roughly thirty-one milliseconds:

$$
22\left(\frac{637}{570}\right)^3\ \text{ms}\approx30.7\ \text{ms}.
$$

That is an extrapolation, not a rank-one benchmark.  This lane ran no Cargo command, as required.  It is enough to establish the price: the existing spectral check is likely material on the frequent path, not a free reuse of machinery.  A Cholesky attempt could have a smaller constant but remains cubic and does not by itself implement the shared eigenvalue-resolution rule.

## Options · `sec:posterior:rank-one-owner-options`

| Option | What it buys | Price and condition |
| --- | --- | --- |
| Align rank one to the shared spectral contract | Tests the matrix actually about to be adopted, catches invalid parents and formation error, and uses the same finite-resolution boundary as the general path (`req:gaussian:positive-definiteness`). | Turns the path from quadratic to cubic; the base timing suggests roughly tens of milliseconds at reference width.  It is the smallest already-proved change. |
| Add a cheap sufficient fast certificate | An outward-rounded Gershgorin pass preserves $O(k^2)$ when it succeeds; a spectral fallback preserves correctness when it is inconclusive. | Adds a second verification algorithm and needs acceptance-rate and rounding-bound evidence.  Dense healthy matrices may fall through often, so its performance value is unproved. |
| Record the exception as sound | The exact proof is short and the runtime check can remain scalar. | Sound only after the system guarantees finite $\delta\geq0$ and a live parent margin $m>\eta+\tau$ for a proved formation-error bound $\eta$.  No such invariant, checkpoint validation, or error bound exists now, and the exhibited SPD counterexample defeats the current condition. |

The determinant identity is not a fourth option.  Without a cached trustworthy factor, exact dual tracking, or a bounded residual, obtaining its scalar costs a solve or assumes the fact it is meant to certify.  Denominator sign alone is weaker still.

## The flagged fixture comment · `sec:posterior:rank-one-fixture-comment`

For exact dual inputs, the covariance submatrix is $\Sigma_{kk}=S_0^{-1}$ and

$$
S_\delta=S_0+\alpha uu^\top,
\qquad
\alpha=\frac{\delta}{a(a+\delta)}.
$$

Writing $v=\Sigma_{kk}u$ and $h_0=u^\top\Sigma_{kk}u$, Woodbury gives

$$
S_\delta^{-1}
=\Sigma_{kk}-\frac{\alpha}{1+\alpha h_0}vv^\top.
$$

The correction is caused by regularisation in that exact construction, but its size contains $a$, $u$, $\Sigma_{kk}$, and the denominator $1+\alpha h_0$. Production dual-tracking drift adds another discrepancy.  The current phrase that the remaining error is “bounded by regularisation and nothing more” is therefore not an honest general conclusion.

The test comment should say:

> On these freshly inverted fixtures, the Woodbury rank-one correction reproduces the inverse of the implemented regularised precision to rounding. The difference from plain covariance-submatrix extraction is introduced by the offset in this exact-dual construction, but its magnitude also depends on the removed pivot, coupling, and covariance quadratic form.  The sweep shows delta-scaling for these fixtures; delta alone is not a general error bound.

That wording retains what the fixture proves and removes the unsupported universal bound.

## Numerical method and reproducibility · `sec:posterior:rank-one-numerical-method`

The counterexample was evaluated with Python standard-library binary64 operations in the same parenthesisation as `marginalise_rank1`.  Parent definiteness and both two-dimensional determinants were then checked using `fractions.Fraction.from_float`, which treats every represented input exactly. Displayed eigenvalues used high-precision `decimal.Decimal` evaluation of the closed two-dimensional formula.

No random sample, external library, fitted parameter, or retained script is needed.  The hexadecimal operands, operation order, and certificate values are all present above.  The construction can therefore be reproduced independently of a decimal parser's display choices.

## Recommendation · `sec:posterior:rank-one-recommendation`

Align the rank-one path to the full spectral contract now.  It is the only option whose premises the implementation already knows how to establish, and the counterexample shows that the present exception can accept a negative eigenvalue even when the represented parent is exactly SPD.  Reuse the general path's finite-resolution decision rather than introducing a bare $\lambda_{\min}>0$ variant.

If profiling rejects that cost, add the outward-rounded Gershgorin test as a fast accept in front of the spectral verifier and measure its acceptance rate on real marginalisations.  Do not record the current denominator-and-diagonal exception as sound.  It earns that status only after a live parent spectral margin and a complete forward-error bound make $m>\eta+\tau$ an enforced precondition, not an assumption in a comment.
