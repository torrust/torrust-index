### Chapter (Per-Sentinel Structured Extraction) · `chap:spec:structured-extraction`

The chapter fixes what the Core takes from one Sentinel's batch report. Every width here is a width the extraction produces, in the order stated. The extraction is identical for every Sentinel and deterministic given a report.

**Setup (From batch report to fixed width)** · `setup:extraction:from-batch-report`

A batch report offers, for each reported competitive cell that processed observations, scores at every level of the cell's ancestor chain, across the four scoring axes, together with coordination context and a structural summary of the report as a whole (`tab:architecture:sentinel-properties`). That is a variable-length structure: it grows with the chain's depth, with the number of reported cells, and with the coordination tier's occupancy.

The extraction compresses it to a fixed-width vector per Sentinel, preserving per-axis identity, multi-scale structure, coordination signal, and outcome history — the identity-preservation principle (`prin:principle:identity-preservation`) applied at the point where variable length has to become fixed width. Six groups are appended in a fixed order: chain z-scores, chain CUSUMs, chain structure, coordination, outcome-memory features, and batch context. The order is part of the contract, because every offset downstream is derived from it.

**Table (Chain z-scores)** · `tab:extraction:chain-z-scores`

Six complementary views over the per-axis maximum z-scores along the ancestor chain, four axes each, twenty-four features. The per-axis maxima are the maximum-z-score field the Sentinel algorithm document publishes at its section 14.4 — the z-score of the per-sample maximum, not that of the batch mean, and the two are separate fields of the report. The distinction preserves sensitivity to an isolated extreme: in a cell taking ten thousand observations in a batch, one anomalous observation still produces a high maximum z-score, where the batch mean is diluted by the rest.

| View | What it captures | Width |
| --- | --- | --- |
| Cell | Local anomaly, at the receiving cell | 4 |
| Root | Global anomaly, at the whole domain | 4 |
| Max | The worst anomaly at any scale, element-wise across levels | 4 |
| Mean | The average across scales — sustained multi-scale signal | 4 |
| Gradient | Cell minus root — positive means the anomaly is localised | 4 |
| Spread | The standard deviation across levels — one loud level against a sustained one | 4 |

Per-axis identity survives all six views: the novelty column tracks novelty at every view and never mixes with displacement. The mean and spread views carry depth-dependent statistical properties — a mean of two from a three-level chain is noisier than the same value from a seven-level chain — and the chain-length feature of (`tab:extraction:chain-structure`) is what lets the model disambiguate them.

**Remark (The maximum view's order-statistic bias)** · `rem:extraction:order-statistic-bias`

The max view is biased upward under the null. The expected maximum of $d$ independent draws grows as $O(\sqrt{\ln d})$, so a deep chain produces a larger maximum than a shallow one from identical per-level distributions, and the difference is the chain's depth rather than the data's anomaly.

The chain-length feature is supplied in the same block precisely so that the model has the information to learn the correction, rather than the extraction applying one. A deployment needing precise null-hypothesis calibration of the max view should apply an extreme-value correction at the point of use; the extraction does not, because a correction applied here would be applied to every deployment on the strength of an assumption only some of them make.

**Remark (The optional batch-mean extension)** · `rem:extraction:batch-mean-extension`

Where the distinction between one loud observation and a sustained batch-level elevation is diagnostically important, a deployment may add the per-axis batch-mean z-scores at the cell level as four further features per Sentinel. At eight Sentinels this is thirty-two features, roughly a five per cent increase in the dimension.

It is a configuration option and not a default. The default extraction reads the maximum because that is the view that survives dilution, and a deployment that wants both is paying dimensions for a second view of the same axis — a trade worth making only where the two have been observed to disagree.

**Table (Chain CUSUMs)** · `tab:extraction:chain-cusums`

Three views over the per-axis cumulative-sum statistics along the chain, four axes each, twelve features. Where the z-scores say how far the current observation departs, the CUSUMs say how long a departure has been accumulating.

| View | Width |
| --- | --- |
| Cell CUSUM, four axes | 4 |
| Root CUSUM, four axes | 4 |
| Max CUSUM, element-wise across levels, four axes | 4 |

**Table (Chain structure)** · `tab:extraction:chain-structure`

Eight features describing the shape of the chain rather than its scores. Two of them are read from the Sentinel's maturity record and not computed by the Core, as the Sentinel algorithm document's section 14.6 defines it.

| Feature | Width |
| --- | --- |
| Cell rank against its cap | 1 |
| Root rank against its cap | 1 |
| Cell energy ratio | 1 |
| Root energy ratio | 1 |
| Cell maturity | 1 |
| Root maturity | 1 |
| Chain length, $\log_2(1 + d)$ normalised by the maximum chain depth | 1 |
| Report staleness, $\log(1 + \Delta t_\text{report})$ | 1 |

The chain length is the block's most-used feature elsewhere: it conditions the depth-dependent views of (`tab:extraction:chain-z-scores`) and it is what makes the order-statistic bias learnable rather than structural.

**Definition (Report staleness)** · `def:extraction:report-staleness`

Report staleness is the wall-clock time since the Sentinel generated the cached batch report, entered logarithmically. Because the Core reads batch summaries rather than per-request scores, the information lost to staleness depends on how far the Sentinel's baselines have moved since the report — a function of elapsed time and of observation volume, of which the Core observes only the first.

The batch context feature (`tab:extraction:batch-context`) supplies a proxy for the second. A cell with a high sample count in the cached report is likely under high throughput, and its baselines are moving proportionally faster, so the model can learn a throughput-adjusted discount from the product of the two features. The proxy breaks under a sudden change in observation rate — a burst arriving after the report — which is exactly the case where staleness matters most. That is a property of a batched measurement interface rather than a defect of the proxy: the Core cannot observe Sentinel state between reports at all, and a deployment needing sub-report sensitivity must shorten the interval rather than ask the extraction for what the interface does not carry.

**Table (Coordination features)** · `tab:extraction:coordination`

Twelve features read from the batch report's coordination summary, which the Sentinel algorithm document specifies at its section 14.9. Each carries a default for the case where the report has no coordination context, and the default is zero throughout, which is the value the absence means rather than a value chosen to be safe.

| Feature | Width | Default when absent |
| --- | --- | --- |
| Per-axis maximum coordination z-score | 4 | 0 |
| Per-axis maximum coordination CUSUM | 4 | 0 |
| Coordination concordance: the fraction of active contexts above the coordination threshold | 1 | 0 |
| Active context fraction | 1 | 0 |
| Peak context depth, normalised by the maximum chain depth | 1 | 0 |
| Root context maximum z-score across axes | 1 | 0 |

Coordination context members are a subset of the reported competitive cells, so these features never reference a cell the rest of the extraction has not seen.

**Table (Outcome memory features)** · `tab:extraction:ledger-features`

Three base features and two per spatially-enabled outcome axis, read from the per-Sentinel outcome memory (`def:ledger:purpose`) rather than from the batch report. Each Sentinel cell holds a decaying history of what happened to requests routed through it, and these features are that history at the request's cell.

| Feature | Width | Condition |
| --- | --- | --- |
| Cell adverse-rate running average | 1 | Always |
| Compressed valence running average | 1 | Always |
| Raw valence running average | 1 | Always |
| Compressed axis running average | 1 | Per active axis with spatial features enabled |
| Raw axis running average | 1 | Per active axis with spatial features enabled |

Whether an axis contributes its two features is the spatial-feature policy of (`disc:registry:spatial-policy`), so the block's width follows the axis registry rather than the Sentinel's report. The convergence and steady-state materiality of these averages are analysed where the memory is specified (`data:ledger:attenuation`); the extraction reads them and asserts nothing about them.

**Table (Batch context)** · `tab:extraction:batch-context`

One feature, the logarithm of the reported cell's sample count in the current batch.

| Feature | Width |
| --- | --- |
| $\log(1 + \text{cell sample count})$ | 1 |

Alone it says how busy the cell was. Its value is in combination: it is the throughput proxy that makes report staleness interpretable (`def:extraction:report-staleness`), and one feature is a cheap price for turning another feature from ambiguous to conditional.

**Definition (Extraction width and the per-Sentinel slot)** · `def:extraction:slot`

The total extraction width per Sentinel is the sum of the six groups:

$$q = 24 + 12 + 8 + 12 + (3 + 2m_s) + 1 = 60 + 2m_s$$

where $m_s$ is the number of active outcome axes with spatial features enabled. Each active Sentinel occupies a contiguous slot of $q + 1$ positions in the feature vector: an occupancy indicator, then the extraction.

$$\text{slot}_s = \bigl[\underbrace{1}_\text{occ} \;\big|\; \underbrace{\mathbf{g}_s}_{q}\bigr] \in \mathbb{R}^{q+1}$$

The occupancy indicator is one whenever the Sentinel is active, whether or not it has data for this particular request, and zero when it has no batch report in the current cycle. It is the prefix that makes the coverage states (`tab:registry:coverage-states`) legible to the model: without it, a silent Sentinel and an offline one present identical zeros.

**Convention (Extraction and slot offsets)** · `conv:extraction:offsets`

Two offset conventions are in use, and both are needed. The extraction reference (`tab:extraction:reference-index`) indexes features within the extraction itself, starting at zero, which is the convention extraction logic is written in. The dimension map (`def:dimension:layers`) reports offsets within the slot, where the occupancy indicator sits at zero and extraction position $i$ sits at slot offset $i + 1$, which is the convention assembly and lifecycle operations are written in.

The two differ by exactly one, which is why they must be named rather than inferred. An offset quoted without its convention is ambiguous by one position in a vector where every position means something different, and the two conventions are kept apart here so that neither site has to guess.

**Remark (Why the extraction stays lean)** · `rem:extraction:lean-extraction`

The extraction is a compression and not a relay. It carries what a risk model can use and leaves the report's structural detail in the report, because a feature the model cannot learn a weight for costs a dimension and returns nothing.

The lever this leaves a deployment is real: dropping the spread and mean chain z-score views saves eight features per Sentinel, giving $q = 52 + 2m_s$, and is worth taking where the label rate is the binding constraint. It is a deployment choice rather than a default, and the choice is between a slightly poorer view of multi-scale structure and a materially faster convergence.

**Remark (Single-precision extraction, double-precision models)** · `rem:extraction:single-precision`

The extraction is computed and stored in single precision; every model that consumes it holds its parameters in double. The split is stated here because it is a property of the interface between the two, and neither side states it alone.

The choice is deliberate on both sides. Extraction values are scores and running averages whose useful precision is a few significant figures, and they are stored per Sentinel per pending request, so their width is a memory cost paid at volume — the same reasoning that fixes the pending buffer's storage precision (`def:runtime:storage-precision`). Model parameters accumulate over hundreds of thousands of updates and are inverted, so their precision is a numerical requirement rather than a preference. Values are upcast where they cross, and nothing downstream may assume the extraction carries more precision than it was stored at.
