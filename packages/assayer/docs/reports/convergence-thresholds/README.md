# Convergence-thresholds study · `rep:health:convergence-threshold-study`

Owner-commissioned evidence for the three decisions recorded by (`cav:health:dead-convergence-thresholds`), not the decisions themselves.

- [spec-study.md](spec-study.md) — line-cited comparison of the specification's budgets and six stages, the tracker's five stages, and the twelve-field construction surface.
- [sensitivity-analysis.md](sensitivity-analysis.md) — sensitivity of the live churn, Jaccard, and age cuts, including the direct age-gate verdict.
- [sufficiency-floor-sensitivity.md](sufficiency-floor-sensitivity.md) — source-accurate reachability and timing of the Companion sufficiency floor.
- [decisions.md](decisions.md) — evidence, options, costs, risks, and study recommendations for vocabulary, missing inputs, and Companion ownership.

**Headline findings**

Ten validated fields are dead and two Platt fields are live. No specification stage is named Warming, Converging, or Converged, so no existing authority maps the generic three-stage surface to either shipped ladder. The caveat's “six-of-eight” input count is erroneous: five stage figures lack tracker-local inputs, while only the three observation minima lack package-wide candidate quantities because per-dimension graph importance already exists.

At the shipped cuts, Jaccard adds no independent classification beyond churn under the documented set-difference inputs. The 264-hour age gate is not itself specified: it is an ADR/code proxy derived from the specification's $2p$ budget (`bound:resource:convergence-budget`). Next-whole-day rounding is supported at the reference inputs, next-whole-week is not, and the fixed gate is only an order-of-magnitude proxy away from those inputs.

The study recommends deleting the eight generic identity-stage figures rather than inventing a vocabulary, relocating the real Companion sufficiency threshold to host-owned Companion health, and deleting the Companion minimum-samples field. The decision remains open.
