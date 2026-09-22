## Appendix (Resonance Rendering) · `app:spec:resonance-rendering`

The decision landscape is the primary, presentation-free output of the Derivation Function (`sig:landscape:output`). This appendix specifies an optional layer that presents that landscape, together with the classification belief, as a smooth spectrum of competing tags for display, audit visualisation and host interfaces. Nothing here carries decision content beyond the landscape and the risk basis: all decision semantics live on the landscape, and all exploration semantics on fragility (`def:fragility:definition`).

The subordination is deliberate and it is recorded where the alternative was rejected (`dec:landscape:rendering-optional`). The presentation-free landscape comes first; the display constants, Cauchy kernel, crossover-matching placement with its existence test, bandwidths with their floor, magnitudes and dominated-tag treatment form a separate rendering over it.

**Signature (The rendering contract)** · `sig:rendering:contract`

The rendering is a function of a landscape, a risk basis and a display configuration, and of nothing else.

```rust
fn render_resonances(
    landscape: &DecisionLandscape,
    risk: &RiskBasis,
    config: &ResonanceConfig,
) -> ResonanceProfile;

struct ResonanceProfile {
    tags: Vec<TagResonance>,
    posture_domain: (f64, f64),   // (0,1), exclusive
    config_echo: ResonanceConfig, // the profile describes its own rendering
}

struct TagResonance {
    tag: Tag,
    location: f64,  // the tag's peak posture, in (0,1)
    magnitude: f64, // peak endorsement strength, positive
    q: f64,         // bandwidth, positive
    dominated: bool,
}
```

The configuration supplies the eight display constants the primary landscape deliberately excludes, and they are tabulated once, in the reference chapter (`tab:config:rendering`). The echoed configuration is what makes a stored profile self-describing: rendering replay needs no external state, because the profile carries the constants it was rendered under.

The configuration record and its defaults agree field for field with the table that owns them, the profile carries the three fields above, and the separate rendering call takes a landscape, its risk basis and the display configuration (`inv:landscape:presentation-free`).

**Table (The tag spectrum)** · `tab:rendering:spectrum`

Two tag families are presented on one shared posture axis.

| Family | Tags | What the family expresses |
| --- | --- | --- |
| Classification | Good, Suspicious, Malicious | What the system believes the observation *is* — a display of the risk probability and its uncertainty (`def:risk:probability`) |
| Action | Allow, Challenge, Slow, Block | What the system reckons about *doing* — a display of the landscape's crossovers (`def:landscape:action-crossover`) |

Each rendered tag carries three parameters and no others.

| Parameter | Symbol | Domain | Meaning |
| --- | --- | --- | --- |
| Location | $\mu_a$ | $(0,1)$ | The posture at which the tag's relevance peaks |
| Magnitude | $A_a$ | $(0,\infty)$ | Peak endorsement strength |
| Bandwidth | $Q_a$ | $(0,\infty)$ | Certainty of the placement |

Classification belief is posture-independent, and placing Good and Malicious *on* the posture axis is a display choreography that lets the two families overlap legibly (`def:rendering:magnitudes`). It is not a claim that belief peaks at a particular caution level, and a reader who takes it for one has read a drawing convention as a result.

**Equation (The kernel and its normalisation)** · `eq:rendering:kernel`

Each tag contributes at posture $\pi$ through a Cauchy kernel on the logit scale:

$$f_a(\pi) = \frac{A_a}{1 + Q_a^2\,\bigl[\ell(\pi) - \ell(\mu_a)\bigr]^2}, \qquad \ell(x) = \ln\frac{x}{1-x}$$

At any posture the contributions normalise across all rendered tags to a distribution over them:

$$P(\text{tag}_a \mid \pi) = \frac{f_a(\pi)}{\sum_{a'} f_{a'}(\pi)}$$

The joint normalisation across the two families is a display convention and nothing more. These are normalised kernel evaluations under the display constants, not posterior probabilities of any event, and a consumer wanting probabilities within one family renormalises within that family (`sig:rendering:ambiguity-gauge`).

**Proposition (Every rendered tag is present everywhere)** · `prop:rendering:completeness`

Every tag has $f_a(\pi) > 0$ at every $\pi \in (0,1)$, so every rendered tag carries positive probability at every posture. The property follows from Cauchy tail positivity alone: the kernel's denominator is finite for finite argument and its numerator is positive by the domain of $A_a$.

The consequence a display depends on is that no posture ever has an empty or degenerate tag distribution, however extreme the evidence. The property is scoped to this layer and constrains a display; it says nothing a host may rely on about the landscape, which is why it is stated here and not in the register of guarantees.

**Proposition (The rendering is proper)** · `prop:rendering:properness`

$\sum_a P(a \mid \pi) = 1$ for every $\pi \in (0,1)$, by construction of the normalisation (`eq:rendering:kernel`). The statement is worth making because properness is the one thing the joint normalisation does buy, and it is routinely confused with the thing it does not: the rendered distribution sums to one over tags, and that fact carries no evidential claim about any tag. Like completeness, the property is scoped to the display layer.

**Proposition (No parameter is clamped to an endpoint)** · `prop:rendering:no-clamping`

Every location satisfies $\mu_a \in (0,1)$ strictly, and no magnitude or bandwidth is clamped to a boundary of its domain. Boundary-action locations are additionally constrained to $(0.01, 0.49)$ and $(0.51, 0.99)$ (`alg:rendering:placement`) so that a peak stays legible at extreme postures. That constraint is a display bound and not an endpoint clamp: it keeps a tag inside a drawable interval, and it never collapses a tag onto the axis endpoints, where the logit is undefined and the kernel would carry no information about the tag's placement.

**Algorithm (Action tag placement)** · `alg:rendering:placement`

Action tags are placed from the landscape's crossovers (`def:landscape:action-crossover`). An interior action locates at the midpoint of its two bounding crossovers on the logit scale:

$$\ell_{\mu,a_j} = \frac{\ell^*_{j-1 \to j} + \ell^*_{j \to j+1}}{2}, \qquad \mu_{a_j} = \sigma\bigl(\ell_{\mu,a_j}\bigr)$$

A boundary action has only one bounding crossover, and is placed by the **crossover-matching** condition: at the crossover $\ell^*$ between a boundary action $a$ and its adjacent interior action $b$, equal rendered contribution requires $f_a(\ell^*) = f_b(\ell^*)$, which fixes the offset

$$d = \frac{1}{Q_a}\sqrt{\rho}, \qquad \rho \equiv \frac{A_a}{A_b}\Bigl(1 + Q_b^2 w_b^2/4\Bigr) - 1$$

with $w_b$ the interior action's regime width on the logit scale. The first action locates at $\ell^*_{1 \to 2} - d$ and the last at $\ell^*_{J-1 \to J} + d$. Where a channel declares exactly two actions both are boundary tags sharing one crossover, and they are placed at $\ell^* \mp 1/Q$.

1. **Locate the interior actions** at the midpoints of their bounding crossovers.
2. **Test existence at each boundary action.** Where $\rho < 0$ no offset satisfies the matching condition, and the action is treated as a dominated display tag (`alg:rendering:dominated-treatment`).
3. **Locate the boundary actions** at their matched offsets, and constrain them to the drawable intervals (`prop:rendering:no-clamping`).

Two reductions are worth recording because they check the formula. At $A_a = A_b$ and $Q_a = Q_b$ the offset is $w_b/2$, a reflection of the interior action's half-width; at $\rho = 1$ it is $1/Q_a$, the bandwidth's own half-power point.

**Definition (Bandwidths and the display floor)** · `def:rendering:bandwidths`

Classification bandwidths derive from the risk uncertainty, and action bandwidths from the crossover variances.

$$Q_\text{Good} = Q_\text{Malicious} = \frac{c_Q\,\kappa_\text{eff}}{\sigma_\text{eff}}, \qquad Q_\text{Suspicious} = c_{\text{susp},Q}$$

$$Q_{a_j} = \frac{c_Q}{\sqrt{\operatorname{Var}(\ell^*_{j-1 \to j}) + \operatorname{Var}(\ell^*_{j \to j+1})}}, \qquad Q_{a_1} = \frac{c_Q}{\sqrt{2\operatorname{Var}(\ell^*_{1 \to 2})}}$$

The action form treats the two bounding crossovers as independent. That is a deliberate and conservative display simplification, and the exact joint structure is available where it matters (`thm:landscape:crossover-covariance`).

The classification form is an identity rather than a choice. Starting from $Q_\text{class} = c_Q \bigl/ \bigl(\sigma_{\hat{p}} / [\hat{p}(1-\hat{p})]\bigr)$ and substituting $\sigma_{\hat{p}} = \hat{p}(1-\hat{p})\,\sigma_\text{eff} / \kappa_\text{eff}$, the $\hat{p}(1-\hat{p})$ factors cancel and the bandwidth is independent of the point estimate (`def:risk:probability`). The classification compression governs where a tag is drawn, not how sharply (`def:rendering:magnitudes`).

**The display floor.** Each action bandwidth is raised to a floor,

$$Q_{a_j} \leftarrow \max\bigl(Q_{a_j},\, Q_\text{floor}\bigr), \qquad Q_\text{floor} = 0.3,$$

and the floor is falsifiable rather than decorative because the condition under which it binds is stated. It binds exactly when a crossover's total uncertainty exceeds $c_Q^2 / (2 Q_\text{floor}^2) = 5.56$. At full challenge sensitivity $\beta_c = 1$ that threshold is never reached and the floor never binds. At $\beta_c = 0$ with the Companion at cold start the Slow-to-Block crossover uncertainty is $6.48$, and the floor binds for Slow and Block until roughly twenty contributing labels have accumulated.

The floor is a display safeguard for wide-interval regimes only, and it is not a substitute for honest uncertainty: the landscape's credible intervals carry the cold-start width undiminished (`def:landscape:credible-intervals`), and a consumer wanting the width the evidence supports reads them rather than the rendered bandwidth. The display configuration carries the floor at $0.3$, applied as a maximum against the raw bandwidth.

**Definition (Magnitudes, relevance and attenuation)** · `def:rendering:magnitudes`

Each tag's magnitude is a relevance term attenuated by the risk uncertainty:

$$A_a = R_a \cdot \alpha(\sigma_{\hat{p}}), \qquad \alpha(\sigma_{\hat{p}}) = \frac{1}{1 + c_A\,\sigma^2_{\hat{p}}}$$

**Action relevance** is the width of the action's optimal regime on the posture axis, $R_{a_j} = \pi^*_{j \to j+1} - \pi^*_{j-1 \to j}$, with the outermost regimes closed at $0$ and $1$. A dominated action receives the dominated magnitude instead (`alg:rendering:dominated-treatment`).

**Classification relevance and placement** render the risk probability and its uncertainty:

$$\mu_\text{Good} = \sigma\bigl(-c_\ell\,\ell(\hat{p})\bigr), \qquad \mu_\text{Malicious} = \sigma\bigl(c_\ell\,\ell(\hat{p})\bigr), \qquad \mu_\text{Suspicious} = 0.5$$

$$R_\text{Good} = 1 - \hat{p}, \qquad R_\text{Malicious} = \hat{p}, \qquad R_\text{Suspicious} = c_\text{susp} \cdot 4\hat{p}(1-\hat{p})$$

The compression $c_\ell$ keeps the Good tag off the extreme tail at very low risk, where an uncompressed logit would drive the location past the drawable interval. Suspicious is centred at $0.5$ and peaks in magnitude near $\hat{p} = 0.5$, which is where irreducible ambiguity belongs. The neutral zone a channel declares is a policy quantity and is not this band (`def:channel:neutral-zone`): the two are drawn on one axis and mean different things, and the display is the only place they meet.

**Algorithm (Dominated-tag display treatment)** · `alg:rendering:dominated-treatment`

Three conditions mark a tag as dominated for display, and they have one effect between them.

1. **Reported domination.** The landscape reports the action's domination probability above the display threshold, which defaults to $0.5$ (`thm:landscape:dominance`).
2. **Crossover inversion.** The action's regime has zero or negative width at the point estimate, so its two bounding crossovers do not bracket it.
3. **Existence failure.** The boundary placement test fails, $\rho < 0$, and no matched offset exists (`alg:rendering:placement`).

Where any of the three holds, the tag's magnitude is set to $\varepsilon_\text{mono}$, its bandwidth to $Q_\text{min}$, and its dominated flag to true. A tag so treated has negligible rendered probability at every posture — of order $10^{-4}$ or below — though never zero, by completeness (`prop:rendering:completeness`).

The suppression is purely presentational and has no counterpart in the landscape, where dominance is always a continuous reading and is reported rather than enforced (`inv:guarantee:dominance`). That asymmetry is the point: a display must choose whether to draw a tag, and the landscape must not, because a structure that appeared and disappeared with the evidence would not be a continuous function of it. Rendering evaluates the continuous domination probability on the landscape against the display threshold and combines that first trigger with crossover inversion and existence failure before marking the tag.

**Example (The four landscapes, rendered)** · `ex:rendering:examples`

The four worked landscapes render at the default display configuration, and the renderings are stated here rather than beside the landscapes so that the presentation-free reading of each example stands on its own (`setup:landscape:worked-assumptions`).

The low-risk landscape (`ex:landscape:low-risk`), at $\hat{p} = 0.05$ and $\sigma_\text{eff} = 0.42$, renders as follows.

| Tag | Location | Magnitude | Bandwidth |
| --- | --- | --- | --- |
| Good | $0.814$ | $0.946$ | $2.38$ |
| Suspicious | $0.500$ | $0.095$ | $0.50$ |
| Malicious | $0.186$ | $0.050$ | $2.38$ |
| Allow | $0.257$ | $0.580$ | $2.48$ |
| Challenge | $0.606$ | $0.048$ | $2.48$ |
| Slow | $0.731$ | $0.181$ | $2.48$ |
| Block | $0.874$ | $0.187$ | $2.48$ |

Renormalised within the action family, the same rendering gives the following probabilities at the postures a host is likeliest to read.

| Posture | Allow | Challenge | Slow | Block |
| --- | --- | --- | --- | --- |
| Permissive, $0.10$ | $91.9\%$ | $1.6\%$ | $4.0\%$ | $2.5\%$ |
| Normal, $0.30$ | $96.5\%$ | $0.9\%$ | $1.8\%$ | $0.8\%$ |
| $0.582$, the Allow-to-Challenge crossover | $30.0\%$ | $30.2\%$ | $32.3\%$ | $7.4\%$ |
| Elevated, $0.60$ | $25.8\%$ | $30.3\%$ | $36.2\%$ | $7.7\%$ |
| Emergency, $0.90$ | $5.4\%$ | $1.5\%$ | $11.4\%$ | $81.7\%$ |

The three-action landscape (`ex:landscape:three-action`) renders Challenge as a broad tag — location $0.702$, magnitude $0.216$, bandwidth $2.48$ — reaching plurality at the Allow-to-Challenge crossover and majority by the elevated posture. That is the wider challenge regime made visible (`prop:channel:challenge-width`), and it is the clearest demonstration of what the rendering is for: a width that the landscape states as a number becomes a shape a reader can see. The medium-risk landscape (`ex:landscape:medium-risk`) and the high-risk landscape (`ex:landscape:high-risk`) render by the same construction, and the crossover-matching condition holds at every crossover of all four to the precision stated.


**Signature (Tag probabilities and the ambiguity gauge)** · `sig:rendering:ambiguity-gauge`

Two utilities read a rendered profile at a posture.

```rust
fn tag_probabilities(profile: &ResonanceProfile, posture: f64) -> HashMap<Tag, f64>;
fn profile_ambiguity(profile: &ResonanceProfile, posture: f64) -> ProfileAmbiguity;

struct ProfileAmbiguity {
    total: f64,          // entropy of the joint rendered distribution
    classification: f64, // entropy within the classification family
    action: f64,         // entropy within the action family
}
```

The first applies the kernel and normalises (`eq:rendering:kernel`). The second returns the Shannon entropy of the rendered tag distribution, decomposed by renormalising within each family.

**The gauge is not a value of information, and the disclaimer is part of the specification rather than a note about it.** Entropy of the rendered distribution weights uncertainty by nothing; a value of information weights it by decision sensitivity. The two agree only by coincidence, and they disagree exactly where a host would act on the difference — at a posture where the distribution is broad but every tag leads to the same action, the entropy is high and the information value is nil. Decision-aware exploration therefore uses fragility, which is defined on the landscape and not on this rendering (`def:fragility:definition`), and the fragility interpretation is where the two surfaces are told apart (`tab:fragility:interpretation`). The type is named for ambiguity precisely to sever the association.
