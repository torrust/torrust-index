# The two numeric losses in the valence chapter · `rep:assayer:lost-valence-figures`

This report is evidence for a ruling not yet made, and it is not the ruling. The outline's open register holds the question open in these terms: a directional-coverage figure was changed with no stated basis in either edition and no supporting computation anywhere, and a stated half-life became an unfalsifiable statement deferring to a later analysis; the review confirmed the finding and could not discharge it, on the ground that only whoever changed the figure knows whether the evidence or the older claim is wanted back (`reg:assayer:spec-outline-open`). The section register raised both under the same heading, as the two places where Part I lost falsifiability (`entry:assayer:spec-sections-ch03`).

What the review could not do was go to the history and to the code. This report does both. It changes nothing in the specification; each option it lays out is written so that choosing it is a single stated edit.

## The anchor coverage figure · `sec:assayer:report-valence-anchor`

**Observation (What changed, and where)** · `obs:assayer:report-valence-anchor-change`

One word changed in one sentence. The pre-rewrite text read:

```text
The anchor converges faster (it has fewer parameters), provides directional
coverage earlier (within ~50 labels), and degrades gracefully under starvation.
```

The July text is that sentence with `~30 labels` in place of `~50 labels`, and is otherwise identical to the character. The change landed in commit `6aef2912`, dated 2026-07-06, whose message is `more work`. That commit touched three files and no others: it rewrote the specification, froze the pre-rewrite text as a sibling copy, and added the tag index. No source file, no configuration and no analysis moved with it, so nothing in the commit or its neighbours records a basis. The two files it created were retired later (`a397411b`, 2026-08-19), which is why both editions must be read out of history rather than out of the tree.

**Data (The two figures coexisted before the change)** · `data:assayer:report-valence-anchor-coexistence`

The premise that the rewrite strengthened an unsupported claim does not survive contact with the pre-rewrite text, which carried both numbers at once. Its overview chapter said fifty. Its warm-up chapter said thirty, three times and without hedging: the cold-start stage was the first thirty labels, the anchor emergence stage ran from thirty to eighty, and its convergence chapter stated flatly that the anchor model at fifteen parameters converges within about thirty labels whatever the deployment's size. The pickaxe finds both strings entering the document in the same drafting commits of March 2026 — the inconsistency is original, not introduced.

So the July edit did not raise a claim above its evidence. It brought the overview into line with the chapter that owned the figure, and it moved the one occurrence that disagreed with the other three.

**Data (The figure is derivable, and derives to thirty)** · `data:assayer:report-valence-anchor-derivation`

The current tree states the rule the figure follows. A Bayesian linear model with $p$ parameters needs on the order of $2p$ observations before its posterior is shaped by data rather than by its prior, and that convention is used wherever a convergence figure appears (`bound:resource:convergence-budget`). The anchor is fixed at fifteen dimensions (`def:dimension:anchor-projection`). The arithmetic is therefore $2 \times 15 = 30$, and the budget states that result in its own words. Fifty would require $p = 25$, which is not the anchor's dimension, nor any dimension the anchor has ever had.

Two further figures in the current tree agree with thirty and disagree with fifty. The risk chapter says the anchor converges some forty times faster than the sister (`def:risk:model-triple`): at the standard deployment the sister's budget is $2 \times 638 = 1{,}276$ eligible labels, and $1{,}276 / 30 = 42.5$, which rounds to the stated forty; against fifty the ratio would be $25.5$. The warm-up stages put cold start at the first thirty labels and anchor emergence at thirty to eighty (`tab:warmup:stages`). Fifty is not arbitrary either — it sits inside that emergence band, and reads like a figure for *comfortably useful* rather than for *first useful* — but the band's lower edge is what the sentence is about, and that edge is thirty.

**Observation (The implementation has already chosen)** · `obs:assayer:report-valence-anchor-code`

The shipped convergence-stage computation carries the threshold as a real constant: `packages/assayer/src/health/composite.rs` gates the exit from the anchor-emerging stage at thirty labels, and its planned replacements are documented in `packages/assayer/src/config/types.rs` as a cold-start threshold of thirty eligible labels and an anchor-emergence threshold of eighty — the warm-up table's two boundaries exactly. This is the same cheap settler that closed the bandwidth-floor question in the open register: the number is already chosen by the implementation, and the document's task is transcription.

**Register (What each option would edit)** · `reg:assayer:report-valence-anchor-options`

- **Adopt thirty, and cite the budget.** The valence chapter currently defers — it says the label count at which the anchor's floor becomes useful is a property of the anchor's dimension, stated with the model (`disc:valence:mitigation-layers`). The deferral points at the model definition, and the model definition states no count (`def:risk:anchor-model`); the count lives at the convergence budget. This option restores the number to the sentence and sends the citation to the budget rather than to the model.
- **Keep the deferral, and repair its target.** Leave the sentence's shape alone and redirect the citation to (`bound:resource:convergence-budget`). This is the minimum edit that makes the current text true, because as it stands the deferral is dangling.
- **Restore fifty.** This would put the overview back into conflict with the warm-up table, the convergence budget, the forty-times ratio and the shipped constant, all four of which would then need changing too.

## The contamination-loop half-life · `sec:assayer:report-valence-halflife`

**Observation (What changed, and where)** · `obs:assayer:report-valence-halflife-change`

The pre-rewrite text closed its eight-step loop with a figure:

```text
Under pure label-indexed decay, the loop's effective half-life reaches
approximately 58 days in realistic deployments — far longer than the transient
adverse event that initiated it.
```

The July text replaced it with a statement that the half-life would be dominated by label arrival and arbitrarily long in a fully starved cell, deferring the quantity to the Ledger's timescale analysis. Commit `6aef2912` made both edits and a third in the same breath: the old timescale table's row reading `~1 label/week → ~58 days` became a time-indexed bound, and survives in the current tree as *bounded by elapsed time* (`tab:ledger:loop-timescales`). The figure was not dropped in isolation; its supporting table cell went with it, in one commit, by an editor who evidently looked at both.

**Data (Candidate derivations, tested against the old text)** · `data:assayer:report-valence-halflife-candidates`

The decay machinery is fully specified, so every candidate can be checked. The Ledger decays at $\gamma_{t,L} = 0.999$ per hour (`def:ledger:time-decay`) and its running averages decay at $\lambda_L = 0.999$ per eligible label; both are the shipped defaults. With $\ln 2 = 0.693147$ and $-\ln 0.999 = 0.00100050$:

| Candidate | Arithmetic | Result |
| --- | --- | --- |
| The old text's own account: label-indexed decay at one label a week | $693$ labels at $1/\text{week}$ | $693$ weeks, about $4{,}850$ days |
| Two elapsed-time half-lives, the quarter point | $2 \times 692.8\,\text{h} = 1{,}385.6\,\text{h}$ | $57.7$ days |
| The label rate at which label-only decay gives 58 days | $692.8 / 58$ | $11.9$ labels a day |
| The actual half-life at that rate, both decays running | $\ln 2 / [(1 + 11.9/24) \times 0.00100050]$ | $19.3$ days |

The first line is the derivation the old text asserted, and it misses by a factor of about eighty-four. The second reproduces $58$ to three significant figures, and it is not a half-life at all: it is the quarter point, which the old text tabulated itself, on the row reading $58$ days against a decayed average of $0.254$ — and $0.999^{1392} = 0.2484$ confirms the row. The likeliest genesis of the figure is therefore a transcription: the quarter point read off that table and written down as a half-life.

One reconstruction saves the number as a statement about the *loop* rather than about the memory. A feedback loop whose restriction pathway returns half of the decayed signal has its effective decay rate halved, and a loop gain of one half doubles $28.9$ days into $57.7$. Nothing in either edition states a loop gain, and no gain is derivable from the specified machinery, so this is a reconstruction of what the sentence could have meant and not evidence of what it did mean.

**Observation (No deployment can exhibit fifty-eight days)** · `obs:assayer:report-valence-halflife-unreachable`

The two decays multiply. A cell receiving $r$ eligible labels a day decays by $\gamma_{t,L}$ each hour and by $\lambda_L$ each label, so its effective half-life is $\ln 2 / [(1 + r/24) \times 0.00100050]$ hours, which is largest at $r = 0$ and equals $28.87$ days there. Every positive label rate makes it shorter: at one eligible label a week it is $28.70$ days, and at the $11.9$ labels a day that label-only arithmetic would need for $58$ it is $19.3$ days. Fifty-eight days is not merely underived; it is unreachable under the shipped constants, in the starved limit and everywhere else. The old text's neighbouring claim that elapsed-time decay binds below about $3.4$ eligible labels a day is also not what its own stated formula computes — that formula evaluates to about $1.0$ — and the current tree has already re-derived the same $3.4$ from the attenuation ceiling instead (`data:ledger:attenuation`).

**Register (What each option would edit)** · `reg:assayer:report-valence-halflife-options`

- **Keep the deferral as written.** The valence chapter already says the quantitative timescales are worked in the Ledger's table and takes its numbers from there (`alg:valence:contamination-loop`), and it already states the $29$-day figure it needs at the point of use. Nothing is edited.
- **State the computed bound in place of the lost figure.** Replace *arbitrarily long in a fully starved cell* with the derived sup: the effective half-life is largest in the starved limit and equals $28.9$ days there, so the loop's persistence is bounded at twenty-nine days for every label rate. This recovers falsifiability with a number the machinery yields, and it is stronger than the old sentence because it is a bound rather than a typical case.
- **Restore fifty-eight days.** This would restore a figure the specification's own decay model excludes, and would need the timescale table's quarter-point row restored alongside it to be readable at all.

## What the evidence supports · `sec:assayer:report-valence-verdict`

**Summary (The two recommendations, with confidence)** · `summ:assayer:report-valence-verdicts`

On the coverage figure: adopt thirty, and repair the deferral's target so that it cites the convergence budget rather than the model definition. Confidence high. The number is derived by the tree's own stated rule from the anchor's own dimension, agreed by three independent statements in the current text, and already chosen by the shipped constant; what the review read as an unevidenced strengthening was the repair of an inconsistency the pre-rewrite text carried from its first draft. The residual judgement is editorial, not factual: whether the overview carries the figure or the citation.

On the half-life: keep the current deferral, and take the second option's sentence if the falsifiability is wanted back. Confidence high on the negative half — fifty-eight days should not be restored, because it is unreachable under the shipped decay constants and its only consistent reading in the old text is a quarter point mislabelled as a half-life. Confidence moderate between the two remaining forms, since both are true and the choice is whether the overview carries a bound of its own or defers the quantity entirely.

**Caveat (What this report did not do)** · `cav:assayer:report-valence-scope`

It did not ask the author: both changes landed in a commit whose message is `more work`, so intent is reconstructed from the text and the code alone, and each reconstruction is marked as one. It did not reopen whether the $2p$ rule is the right convergence criterion for a fifteen-dimensional model — that rule is the tree's own, used wherever a convergence figure appears, and displacing it would change the resource chapter rather than this one. And it did not edit the specification: every option above stands until a ruling takes one.
