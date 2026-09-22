### Chapter (Detection Analysis) · `chap:spec:detection-analysis`

The chapter argues rather than specifies. Its subject is what the composed design achieves that no layer of it achieves alone, and its claims are about the system as assembled in the earlier Parts rather than constraints on how those Parts are built.

Two environments are exceptions and they are marked as such: what the divergence measures and where a host may read it, and the rule that the Core reports one figure per assessment and knows nothing of channels. The rest is reasoning, and it is carried rather than compressed because one piece of it — the admission that the avoidance argument covers the measurement layer and not the memory layer — is what keeps the laundering exposure honest.

**Table (The detection stack)** · `tab:detection:stack`

| Layer | Mechanism | What must be normal for avoidance | Convergence |
| --- | --- | --- | --- |
| Per-Sentinel, per-value | Ancestor chain (`tab:extraction:chain-z-scores`) | Each value normal at every scale | Hours |
| Per-Sentinel, cross-cell | Coordination (`tab:extraction:coordination`) | Cross-cell score distribution normal | Hours |
| Cross-Sentinel, per-axis | Aggregate features (`tab:feature:aggregate-block`) | Each axis's cross-Sentinel profile | Days |
| Cross-Sentinel, compound | Slots and interactions (`tab:feature:default-interaction-set`) | Sentinel-specific and joint patterns | Weeks–months |
| Risk estimation, sister | Unconfounded training (`def:risk:model-triple`) | Inherent risk survives confounding | Weeks |
| Risk estimation, anchor | Coarse and measurement-only (`prop:risk:anchor-measurement-only`) | Correct direction while the sister is starved | Days |
| Spatial history | Outcome Ledger (`def:ledger:purpose`) | Adverse regions never flagged | Days–weeks |
| Identity history | Competitive cells (`tab:keyspace:outcome-state`) | Key range profiles and transitions | Days–weeks |
| Outcome prediction | Axis models (`def:axis:per-axis-model`) | Predicted secondary outcomes | Weeks |

The table's substantive claim is in its third column. Each layer addresses a different avoidance strategy, so a source constructed to look normal at one layer is not thereby normal at another, and the layers converge on different timescales, so the stack is never uniformly cold: the fast layers cover while the slow ones learn. That is the composition's whole detection argument in one view, and the two discussions below are its two halves — what the composition buys, and what it does not.

**Discussion (Cross-Sentinel compound detection)** · `disc:detection:compound`

Per-Sentinel identity survives aggregation, because each Sentinel's features occupy a named slot rather than being averaged away (`def:extraction:slot`). The Core can therefore learn that a pattern matters at one Sentinel and not at another, which a Sentinel-count-invariant summary alone could never express. Joint structure across Sentinels is reachable two ways: explicitly, by naming the pair a host expects to matter (`def:feature:template-named-pair`), and implicitly, by letting the wildcard template enumerate the pairs (`def:feature:template-wildcard`). The aggregate features sit alongside both and provide the summary that does not depend on how many Sentinels are reporting (`sec:feature:aggregate`).

What the architecture discards is worth stating as plainly as what it keeps. Within-batch, per-observation differentiation from ambient cell state is gone. Two requests arriving in the same batch at the same Sentinel cell receive identical per-Sentinel features, and are distinguished only by their entity, their signals, cross-Sentinel variation and Ledger state. The Core reads batch summaries and never per-request scores, so nothing that happens between two reports is visible to it at all (`cav:limitation:inter-report`). This is a deliberate trade of resolution for the ability to read many Sentinels at once, and it is the reason the stack's fastest layer is measured in hours rather than in requests.

**Discussion (The multi-objective avoidance dilemma)** · `disc:detection:avoidance-dilemma`

A source seeking to avoid the whole stack faces three objectives in mutual tension. Per-value avoidance pushes towards uniformity. Cross-cell diversity requires variation. Cross-Sentinel consistency requires profiles that are jointly normal, not merely normal one at a time. Satisfying any one of them makes the others harder.

The composition therefore raises avoidance to a constrained multi-scale, multi-cell, multi-source, multi-axis optimisation with no known efficient solution except one: produce values genuinely indistinguishable from normal at every scale simultaneously, which is not avoidance but ordinary behaviour. That is the argument the stack exists to support.

Its scope is narrower than it first appears, and the qualification is the most important sentence in the chapter. The argument concerns evasion of the *measurement* layer. It does not apply to the *memory* layer. The Ledger's write path runs through labels on cells (`def:ledger:purpose`) and the identity layer's competitive weights are driven by observed, labelled traffic (`def:keyspace:competitive-set`); both can therefore be moved by traffic that is genuinely normal. Being statistically normal defeats measurement-layer detection and is precisely what memory-layer laundering requires, so the two exposures are complementary rather than alternative, and the more accessible of the two is the one this argument does not cover (`cav:limitation:laundering`).

**Table (The cross-Sentinel convergence window)** · `tab:detection:convergence-window`

| Template type | Convergence | Coverage during convergence |
| --- | --- | --- |
| Aggregate by context (`def:feature:template-aggregate`) | Days | Available from the first days |
| Per-Sentinel by context (`def:feature:template-per-sentinel`) | Weeks | Available from the first weeks |
| Cross-Sentinel, named pairs (`def:feature:template-named-pair`) | Months | Partial: aggregate features give coarse coverage |
| Cross-Sentinel, wildcard (`def:feature:template-wildcard`) | Months | Partial: aggregate features give coarse coverage |

The window is a consequence of the convergence cascade rather than a separate schedule (`tab:feature:interaction-convergence`). What is unavailable during the months the cross-Sentinel templates take is one specific capability: detecting sources that evade each Sentinel independently while producing concealment patterns correlated across them (`cav:limitation:cross-sentinel-gap`). The lower tiers provide continuous coverage throughout, so the window is a period of reduced resolution and not a gap in detection.

**Definition (What the divergence measures and where it may be read)** · `def:detection:divergence-scope`

The divergence is the difference between the blended unconfounded estimate and the operational model's realised risk:

$$\Delta(\phi) = \hat{\rho}_\text{eff}(\phi) - \hat{\rho}_\text{opr}(\phi)$$

A positive divergence means the unconfounded estimate exceeds what the host's own restricted population realises — the host's interventions are doing work (`def:risk:intervention-effectiveness`). It is reported once per assessment on the risk basis.

The scope condition is the definition's substance. The divergence is interpretable only where unconfounded coverage exists. The sister term is identified as inherent risk only where eligible labels exist (`def:risk:model-triple`), so in feature regions the host has always restricted, that term is prior extrapolation rather than measurement, and the difference there measures the distance between an extrapolation and a confounded estimate. It does not measure intervention value, and it does not announce that it is not doing so.

A host must therefore read the divergence only where eligible-label density is adequate, and the system offers two instruments for knowing where that holds: the resolution-utilisation ratio, which reports the fraction of cells whose discrimination is backed by outcome evidence (`def:monitoring:resolution-utilisation`), and the starvation guidance, which names the cells most lacking it (`def:guidance:starvation`). Host investigation is the only mechanism that restores identification in a region that has always been restricted (`conv:eligibility:ground-truth`). Where the restriction is driven by information the feature vector does not carry, the aggregate figure stays directionally correct while per-feature attribution becomes misleading (`cav:limitation:divergence-attribution`).

The divergence is computed and carried exactly as defined above, and its scope condition is read through the two instruments this definition names.

**Requirement (One figure per assessment and no channel dimension)** · `req:detection:per-channel-reporting`

The Core reports a single divergence per assessment. It carries no channel dimension, because the Core has no concept of a channel and cannot acquire one without acquiring the decision surface it is built not to have (`prin:principle:measure-not-decide`).

A host that wants intervention effectiveness per channel therefore computes it per channel, by aggregating the per-assessment figures through its own routing knowledge. This is not a deficiency to be repaired by adding a dimension. The routing is the host's, the channel definition is the host's, and a Core that reported per-channel figures would be asserting a partition it does not own (`sig:landscape:derivation-function`).
