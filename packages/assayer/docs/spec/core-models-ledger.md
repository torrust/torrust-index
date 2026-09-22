### Chapter (The Outcome Ledger) · `chap:spec:outcome-ledger`

The chapter divides into two halves that read very differently. The first is normative: what the Ledger holds, how it is updated and decayed, and what happens when its entries are created, deleted, and collected. The second is analytic, and it exists to establish that the Ledger is structurally uninformative for most cells in most deployments — a conclusion the specification states about its own mechanism rather than leaving to be discovered. Both halves are needed. A reader who takes only the first will over-rely on a memory that is mostly empty; a reader who takes only the second will remove a mechanism that is decisive where it does converge.

**Definition (The Outcome Ledger)** · `def:ledger:purpose`

The Outcome Ledger maintains, for each Sentinel, a layered map of outcome history over that Sentinel's coordinate domain. Each layer corresponds to a depth in the dyadic hierarchy. The root layer, covering the whole domain, always exists; finer layers are created when the Sentinel's batch report carries cells at greater depth. Several layers may cover one coordinate, and feature extraction resolves to the deepest available.

Every layer independently tracks the outcome history of its entire range. A label updates every layer containing its coordinate, from the deepest to the root (`alg:ledger:all-layers-update`), mirroring the multi-scale delivery the spectral Sentinel publishes as its algorithm S-9.3, where an observation reaching a cell also reaches every ancestor on the path above it. Each layer's averages are therefore always current with respect to every label in its range, and no layer is ever stale.

Two properties follow, and between them they eliminate the whole question of state inheritance. Deletion requires no merge, because the next coarser layer already reflects every label the finer layer saw — the only thing lost is spatial discrimination, and a transfer would double-count. Creation requires no inheritance, because the coarser layer continues to exist and be updated underneath; it is the best available prior for the new layer's rate, and it is expressed through the model's learned weights on the coarser layer's features rather than through copied state.

**Summary (When the Ledger is worth its cost)** · `summ:ledger:value`

The Ledger supplies persistent spatial reputation: the ability to remember that a Sentinel cell has historically been associated with adverse outcomes, even after the Sentinel's own measurement baselines have adapted and stopped reporting anything unusual about it. No other feature pathway can do this, because every other pathway is computed from the current report.

The capability requires two conditions at once: the cell's true adverse rate must substantially exceed the population average, and the cell must receive enough eligible labels for its running average to outrun time-indexed decay (`thm:ledger:materiality`). For the majority of cells in typical deployments one or both fail, and the Ledger features carry no information (`cav:limitation:ledger-low-traffic`). Detection in those cells runs through the immediate-convergence pathways instead (`tab:ledger:low-maturity-detection`).

The Ledger is therefore a high-traffic bonus and not a baseline dependency, and the distinction is load-bearing: a deployment that sizes its detection expectations on the Ledger converging will be disappointed everywhere except its busiest cells, and a deployment that removes the Ledger loses precisely the memory its busiest cells depend on.

**Theorem (Ledger materiality)** · `thm:ledger:materiality`

A Ledger bad-rate feature is material — it produces a standardised departure of more than half a standard deviation from the standardisation mean — only when the cell's true adverse rate substantially exceeds the population average *and* its eligible label rate is high enough for the attenuation factor to preserve that excess through time-indexed decay. Neither condition suffices alone.

The two conditions are independent constraints and both bind. A cell at three times the population rate, against a standardisation mean of about $0.03$ and a standard deviation of about $0.04$, needs a steady-state average above $0.05$ to be material — which the attenuation table reaches only above about a hundred eligible labels a day (`data:ledger:attenuation`). A cell at six times the population rate reaches materiality at around ten labels a day, and still fails at three and a half.

The result is the chapter's central negative claim, and it is a theorem rather than an observation because it follows from the two decay constants and the standardisation statistics by arithmetic, not from any deployment's experience. Its practical form is a maturity predicate on each cell and a fleet-wide count of immature cells (`def:monitoring:immature-cells`), so that a deployment can see how much of its Ledger is actually carrying information rather than assuming all of it is.

**Table (Per-entry outcome state)** · `tab:ledger:entry-state`

Every entry, at every depth, carries the same eleven fields.

| Field | Update |
| --- | --- |
| Total assessments | Incremented per assessment routed to this cell |
| Per-action count | Incremented by the recorded action at label time |
| Bad-rate average | Averaged at rate $\lambda_L$ on the positive-valence indicator |
| Compressed valence average | Averaged at rate $\lambda_L$ on the compressed valence |
| Raw valence average | Averaged at rate $\lambda_L$ on the raw valence |
| Per-axis compressed average | One per spatially enabled active axis |
| Per-axis raw average | One per spatially enabled active axis |
| Last updated | The timestamp time-indexed decay is measured from |
| Recent eligible label count | Cumulative, saturating against $N_\text{ledger} = 200$ at its consumers |
| Adverse eligible label count | Cumulative and undecayed, over exactly the population the row above counts |
| Eligible arrival window | Exponentially time-weighted eligible arrivals: one added per eligible label, decayed at $\gamma_{t,\text{ledger}}$ |

The averaging rate is $\lambda_L = 0.999$ by default, a label-indexed half-life of about six hundred and ninety-three labels *per cell* — which is the number that matters, and the reason the analytic half of this chapter exists. In low-traffic regions the cell does not receive labels fast enough for the label-indexed half-life to be the operative one, and the effective decay is dominated by elapsed time instead (`def:ledger:time-decay`).

The saturating count is held cumulative and clipped at its consumers rather than clipped in storage, so that one threshold can be raised later without having discarded the evidence that would justify it.

The last two rows are the materiality evidence, and they exist because the averages above them cannot supply it. Every average in this table has already paid elapsed-time decay by the time anything reads it, so a low average is two situations at once — an ordinary cell, and a high-rate cell whose labels arrive too sparsely to hold the excess. The theorem's two conditions are exactly those two situations told apart (`thm:ledger:materiality`), and telling them apart takes the true rate and the arrival rate as evidence in their own right rather than one attenuated average standing in for both.

The pair is undecayed on both sides and conditioned on eligibility on both sides, because the convergence and attenuation figures below are indexed in eligible labels: a rate measured over a wider population would then be attenuated by a factor computed for a narrower one. The window is decayed on the Ledger's own hourly rate rather than on a horizon of its own, which is what makes it readable as an inter-arrival decay — a cell receiving one eligible label every $\Delta h$ hours settles at $L = 1 / (1 - \gamma_{t,\text{ledger}}^{\Delta h})$, so the per-interval decay the attenuation needs is $1 - 1/L$ and no arrival interval has to be estimated separately. Neither field is a second maintenance phase: both are written by the same all-layers update that writes the averages (`alg:ledger:all-layers-update`) and read by the same read-time decay that reads them (`def:ledger:time-decay`).

**Data (Convergence and steady-state attenuation)** · `data:ledger:attenuation`

The Ledger averages face two independent constraints: how fast a running average tracks a true rate, and how much elapsed-time decay erodes it between labels. From a zero start, the bad-rate average after $n$ eligible labels at a cell of true rate $p$ is $\bar{b}_n = p(1 - \lambda_L^n)$.

| Eligible labels | Fraction of true rate | At $p = 0.05$ |
| --- | --- | --- |
| 10 | 1.0% | 0.0005 |
| 50 | 4.9% | 0.0024 |
| 100 | 9.5% | 0.0048 |
| 200 | 18.1% | 0.0091 |
| 500 | 39.4% | 0.0197 |
| 693 | 50.0% | 0.0250 |
| 1,000 | 63.2% | 0.0316 |

Against that, time-indexed decay at $\gamma_{t,L} = 0.999$ per hour erodes the average between labels, so a cell receiving $r$ eligible labels a day settles at a permanently attenuated value:

$$\bar{b}_\infty(p, r) = \frac{(1 - \lambda_L) \cdot p}{1 - \lambda_L \cdot \gamma_{t,L}^{24/r}}$$

| Label rate, per day | Attenuation | At $p = 0.05$ | At $p = 0.15$ |
| --- | --- | --- | --- |
| 1 | 4.0% | 0.0020 | 0.0061 |
| 3.4 | 12.4% | 0.0062 | 0.019 |
| 10 | 29.4% | 0.015 | 0.044 |
| 100 | 80.6% | 0.040 | 0.121 |

Below about three and a half eligible labels a day the elapsed-time decay dominates outright and the average cannot exceed an eighth of the true rate however many labels eventually accumulate. The same attenuation applies to every per-cell average, compressed, raw and per-axis alike, since all of them share both rates.

The attenuation is evaluated from the stored arrival window rather than from an estimated rate, and the two are the same figure written differently. A window holding $L$ time-weighted eligible arrivals has settled at $\gamma_{t,L}^{24/r} = 1 - 1/L$, and substituting that into the expression above gives

$$A(L) = \frac{1 - \lambda_L}{(1 - \lambda_L) + \lambda_L / L}$$

which reproduces the four tabulated rows to the precision they are stated at. Writing it this way removes the step in which a rate is recovered from the evidence and then exponentiated back, so nothing about the arrival pattern has to be assumed beyond what the window already holds. The two limits read correctly on their own: a window decayed to a single arrival gives $1 - \lambda_L$, which is what one isolated label leaves in an average, and a window filling faster than the decay empties it approaches one.

**Algorithm (The all-layers update)** · `alg:ledger:all-layers-update`

At label time, for each Sentinel that was reporting when the assessment was made:

1. **Find** every entry whose interval contains the request's coordinate for this Sentinel, from deepest to shallowest. The root always exists, so at least one is always found.
2. **Decay** each entry at write time for the elapsed interval since its last update, before applying anything (`def:ledger:time-decay`). The eligible arrival window pays the same factor as the averages; the raw counts pay nothing, being undecayed by construction.
3. **Update the bad-rate average** toward the positive-valence indicator at rate $\lambda_L$.
4. **Update the compressed valence average** toward $\tanh(v / \kappa_v)$.
5. **Update the raw valence average** toward the raw valence.
6. **Update the per-axis averages**, compressed and raw, for each reported axis with spatial features enabled.
7. **Increment** the per-action count by the recorded action.
8. **Increment** the recent eligible label count if the label is eligible (`tab:eligibility:training`), add one to the eligible arrival window, and increment the adverse eligible label count when that eligible label is adverse. Then set the last-updated timestamp.

Every entry containing the coordinate is updated, whether or not a finer entry also contains it; each independently tracks the complete history of its own range. The cost is one pass of averages per containing depth per Sentinel per label, and in practice entries exist at only a few depths — typically three to eight — so the cost is bounded by the hierarchy's realised shape rather than by its possible one.

**Definition (Time-indexed Ledger decay)** · `def:ledger:time-decay`

The Ledger decays with elapsed time far faster than the core models do:

$$\gamma_{t,\text{ledger}} = 0.999 \text{ per hour}, \qquad \text{a half-life of about 29 days}$$

against the models' hourly rate with its half-life near two hundred and ninety days, and the identity layer's faster fourteen (`tab:keyspace:decay-rates`).

The decay is applied in two modes, and the pair is what makes the Ledger readable without locking. At write time, during label processing, the elapsed interval is measured, every average is scaled by the decay over it, the label is applied, and the timestamp advances. At read time, during assessment, the decayed value is computed as a pure function of the stored value and the elapsed interval, without modifying the entry at all. The two agree exactly: read-time decay produces the value that persisting the decay would have produced, which is why an assessment can touch Ledger entries without taking a write lock (`inv:guarantee:non-blocking`).

The fast rate is the primary mitigation for the contamination loop (`alg:valence:contamination-loop`). However few eligible labels a cell receives, its stale reputation dissolves on a twenty-nine day timescale; under label-indexed decay alone a starved cell's reputation would persist far longer, and the elapsed-time rate imposes a hard bound the loop cannot escape (`inv:guarantee:ledger-floor`).

#### Entry lifecycle · `sec:ledger:entry-lifecycle`

The division carries no material of its own. It collects the three events that alter the set of entries — creation, deletion, and collection — and the justification they share, all of them consequences of all-layers routing rather than mechanisms in their own right.

**Algorithm (Entry creation)** · `alg:ledger:entry-creation`

When a cell appears in a Sentinel's batch report and no entry exists at that exact interval, an entry is created with neutral state.

1. **Zero** every average: bad rate, compressed valence, raw valence, and every per-axis average.
2. **Zero** the total assessments, every per-action count, and the recent eligible label count.
3. **Zero** the materiality evidence: the adverse eligible label count and the eligible arrival window. A cell with no arrivals names no arrival rate and a cell with no eligible labels names no true rate, so both read as absent rather than as zero until evidence arrives (`thm:ledger:materiality`).
4. **Stamp** the last-updated timestamp at the current time.

Nothing is inherited from any ancestor. The ancestors continue to exist beneath the new entry and continue to be updated by every label in their range (`alg:ledger:all-layers-update`), so for coordinates inside the new entry's range the new entry is what extraction reads while its ancestors keep accumulating the same labels.

The cost is stated rather than hidden: the features extracted from this entry are zero until labels accumulate, and at the convergence rates of the attenuation analysis that is a long time (`cav:limitation:fresh-entry`). Detection during the interval runs through the immediate-convergence pathways (`tab:ledger:low-maturity-detection`). This is the designed operating mode for a new entry, not a gap in it.

**Algorithm (Entry deletion)** · `alg:ledger:entry-deletion`

When an entry below the root has been absent from the batch report for $N_\text{absent}$ consecutive report cycles — three by default — it is deleted.

1. **Count** consecutive report cycles in which the entry's interval does not appear.
2. **Delete** the entry outright once the count reaches the threshold.
3. **Serve** subsequent coordinates in its former range from the deepest surviving entry, which is already current.

No merge, and no state transfer. The nearest ancestor already reflects every label that passed through the deleted entry's range, having been updated continuously throughout its existence, so a merge would count every one of those labels a second time.

The threshold exists to absorb transience. A cell that drops out of one report — zero observations this cycle, or a passing internal dynamic of the Sentinel — and returns before the threshold finds its entry intact, with no deletion, no recreation, and no loss. The behaviour the threshold governs is deletion, and the entry does not persist in any dormant form after it: an implementation that names this threshold for a suspension it does not perform should correct the name to match what happens.

**Algorithm (Garbage collection)** · `alg:ledger:garbage-collection`

An entry whose averages have all decayed below $10^{-6}$, and whose last label arrived longer ago than the collection horizon — sixty days by default — may be removed. The root is never collected. No merge on collection either; the ancestor is already current.

Two strategies are admissible, and the choice is the deployment's.

1. **Sweep periodically**, the default: a maintenance task scans each Sentinel's Ledger on a fixed cadence, hourly by default, removing entries that meet both predicates. This is the correct strategy for hash-keyed or flat spatial indices, where an expired neighbour has no cheap meaning, and it keeps variable collection cost off the assessment path.
2. **Collect lazily on access**, admissible only where an expired neighbour is cheaply defined, as in a trie: reading or writing an entry reclaims expired neighbours in the same index within a bounded step count.

Either way the per-Sentinel write budget binds for any operation holding the Ledger write lock (`inv:guarantee:staleness`). The floor and horizon are deliberately conservative: collection is a storage economy and never a correctness mechanism, and an entry collected while it still carried information would be indistinguishable, to every consumer, from a cell that had never been seen.

**Justification (Why no state transfers)** · `just:ledger:no-state-transfer`

Neither creation nor deletion moves state, and the two cases fail for opposite reasons.

Inheritance on creation would transfer outcome history from a broader range to a narrower one, which assumes the broader range's adverse rate applies uniformly across its sub-ranges. That assumption is precisely the one the Sentinel refuted by creating a finer cell at all: the cell exists because the Sentinel found structure the coarser range was hiding. Starting from zero assumes nothing, and the ancestor's rate remains available through the model's learned weights on the ancestor's own features — a better path, because the model conditions on every other feature simultaneously where a copied average would condition on nothing.

Merging on deletion would double-count. Under all-layers routing the ancestor's average already includes every label that reached the deleted entry, so folding the child's average in again would inflate the ancestor by the child's entire history. Simple deletion is not the convenient choice among several; it is the only correct one.

#### Contamination and starvation · `sec:ledger:contamination`

The division carries no material of its own. It collects the loop's timescales and its binding constraint, the starvation-relief score, and the opposite-direction exposure that the same symmetric decay creates.

**Table (The loop's timescales)** · `tab:ledger:loop-timescales`

The contamination loop runs through four timescales, and which one binds decides how long a contaminated reputation lasts.

| Timescale | Typical value | What it governs |
| --- | --- | --- |
| Sentinel baseline recovery | Hours to about a day | Measurement features normalising after an adverse event |
| Label-indexed Ledger decay | About 693 labels per cell | The averaging half-life in labels |
| Effective Ledger decay when starved | Bounded by elapsed time | The half-life when eligible labels are rare |
| Time-indexed Ledger decay | About 29 days | The hard bound, whatever the label flow |

Below about three and a half eligible labels a day the elapsed-time decay dominates the effective decay, that being the rate under which steady-state attenuation cannot clear its ceiling (`data:ledger:attenuation`). For most cells in most deployments this is the binding constraint, and the consequence is the reassuring half of the analysis: the loop's persistence is bounded at twenty-nine days however starved the cell becomes. A transient average driven to unity decays to about $0.84$ after a week, about half after twenty-nine days, and to an eighth after eighty-seven — by which point the model's learned weight on the feature produces negligible elevation. The measurement-only anchor feature runs beneath all of it, unaffected (`prop:risk:anchor-measurement-only`).

**Definition (The starvation-relief score)** · `def:ledger:starvation-score`

Label guidance identifies cells whose Ledger state is starved of eligible feedback by scoring the two conditions together:

$$\text{starvation}(s, \text{cell}) = \bigl(\bar{b}_{s,\text{cell}} - P_+^\text{eligible}\bigr)^+ \cdot \left(1 - \frac{n_{\text{elig},s,\text{cell}}}{N_\text{ledger}}\right)^+$$

The first factor measures how far the cell's remembered adverse rate diverges above the system-wide eligible base rate; the second measures how far the cell falls short of the label count at which its average would be trusted. Both are clipped below at zero, so a cell that is either unremarkable or well fed scores nothing at all.

The product is the point. A cell with a high remembered rate and plenty of eligible labels needs no relief — its average is earned. A cell with few labels and an unremarkable rate is merely quiet. It is the conjunction that identifies the cell where the loop is closed: a reputation the system is acting on, held up by evidence too thin to have tested it, and not being tested because the system is acting on it. Those are the cells where an investigated label buys the most, which is what the guidance interface is for (`def:guidance:starvation`).

**Discussion (Reputation laundering and symmetric decay)** · `disc:ledger:laundering`

Everything above concerns stale adverse reputation lasting too long. The decay architecture is symmetric, and an adversary can work the forgiving half of it.

An adversary controlling a key range can generate genuinely benign traffic through it — real, unobjectionable activity earning eligible negative labels — and each such label drives the cell's average and the learned competitive weight downward. After enough benign volume the range reads as clean, and an attack launched from inside it starts from a low-suspicion baseline. This is much cheaper than evading the measurement layer, which demands being statistically indistinguishable from normal at every scale at once; laundering the memory layer demands only sending traffic that is genuinely normal, which by construction it is. Alongside it runs scheduled forgiveness: because elapsed-time decay dissolves reputation on a fixed schedule regardless of label flow, even a detected and confirmed attack's reputational cost expires on that schedule, and the decay cannot distinguish a range that reformed from one that waited out the timer.

Mitigation is partial and structural. A laundered range that turns adverse still trips the measurement-only anchor feature, the aggregate cross-Sentinel features, and the competitive interactions where the range is competitive (`def:feature:template-competitive`) — the Ledger being one pathway of seven (`tab:ledger:low-maturity-detection`). But no mechanism distinguishes an earned benign label from a manufactured one, because the Core trusts every label it is given (`inv:guarantee:honest-uncertainty`). The symmetric-decay trade was resolved deliberately toward curing staleness, false-positive persistence being the more common operational pathology, and the forgiveness exposure is the accepted cost of that choice (`cav:limitation:laundering`).

**Definition (Root entry semantics)** · `def:ledger:root-semantics`

The root entry, at depth zero, carries three distinguished roles under all-layers routing.

It is the universal fallback: any coordinate without a finer entry reads from the root during extraction, which covers every region the Sentinel has never refined deeply enough to report cells for. It is the domain-wide baseline: its average is the running adverse rate across every labelled request routed through this Sentinel, and since every label updates it, the root is the richest, most stable and most current entry in the Ledger. And it is the contamination anchor: being the most heavily smoothed entry, it is the most resistant to any single transient, where finer entries are volatile precisely because they see fewer labels.

The root is created at Sentinel registration and destroyed only at Sentinel deregistration. It is exempt from collection unconditionally (`alg:ledger:garbage-collection`), and the exemption is not an optimisation detail: a Ledger whose root had been collected would have no fallback for unrefined coordinates and no baseline to read finer entries against.

**Table (Ledger against identity outcome state)** · `tab:ledger:versus-identity`

The identity layer's per-cell outcome state is structurally parallel to the Ledger and answers a different question.

| | Sentinel Outcome Ledger | Identity competitive cell state |
| --- | --- | --- |
| Indexed by | Sentinel cell, a spatial coordinate | Identity cell, an entity key range |
| Exists for | Every cell ever seen | Competitive cells only |
| Updated at | Label time | Label time |
| Read at | Assessment time, per Sentinel | Assessment time, through per-dimension aggregates |
| Primary purpose | Per-Sentinel spatial risk features | Per-axis identity context and warm start |
| Contamination mitigation | Elapsed-time decay over 29 days | Elapsed-time decay over 14 days |

The Ledger answers what happens to traffic *through* a network, geographic or categorical region; the identity layer answers what happens to traffic *from* a key range. Both enter the feature vector and neither is redundant with the other: a request can come from a well-behaved entity through a dangerous region or the reverse, and the two states disagree in exactly the cases that matter most. The faster identity decay reflects the difference in what is being remembered — an entity's behaviour can change on its own initiative, where a region's character changes more slowly (`tab:keyspace:outcome-state`).

**Table (Detection at low Ledger maturity)** · `tab:ledger:low-maturity-detection`

The Ledger features are one of seven pathways from a Sentinel to the risk model. When they are immaterial — as they are for most cells (`thm:ledger:materiality`) — detection runs through the other six.

| Pathway | Features | Convergence | Needs the Ledger |
| --- | --- | --- | --- |
| Chain z-scores (`tab:extraction:chain-z-scores`) | 24 | Immediate, from Sentinel baselines | No |
| Chain cumulative sums (`tab:extraction:chain-cusums`) | 12 | Immediate | No |
| Chain structure (`tab:extraction:chain-structure`) | 8 | Immediate | No |
| Coordination (`tab:extraction:coordination`) | 12 | Immediate | No |
| Batch context (`tab:extraction:batch-context`) | 1 | Immediate | No |
| Ledger features (`tab:extraction:ledger-features`) | $3 + 2m_s$ | Weeks to months | Yes |
| Aggregate cross-Sentinel features (`tab:feature:aggregate-block`) | 15 | Immediate | No |

Fifty-seven of the per-Sentinel slot's features carry information from the first batch report, before any label has been processed; the Ledger features, five to ten per cent of the slot, are the only ones needing per-cell outcome history. Outside the slot, competitive indicators populate from the first assessment, per-dimension measurement state tracks alarm profiles independently of any label, and the measurement-only anchor feature bypasses the Ledger entirely (`prop:risk:anchor-measurement-only`).

The architecture is designed so the Ledger is a bonus at high traffic rather than a dependency at any. Where both materiality conditions hold it supplies what no other pathway can — memory that a neighbourhood was dangerous after the measurement baselines have adapted — and where they do not, detection falls back to the immediate pathways as a designed mode rather than a degraded one.

**Decision (Why the Ledger rate is fixed)** · `dec:ledger:no-adaptive-rate`

A traffic-adaptive averaging rate — faster where labels are plentiful, slower where they are scarce — would improve the attenuation factor for exactly the low-traffic cells the analysis shows are uninformative, and it is the obvious repair. It is not adopted, and the reasoning is recorded because the repair will keep suggesting itself.

The elapsed-time decay's primary purpose is contamination mitigation: it must dissolve stale reputation on a fixed schedule whatever the traffic (`def:ledger:time-decay`). An adaptive label-indexed rate that slowed decay at starved cells would extend the contamination loop's persistence in precisely the cells most vulnerable to starvation — the cells where a reputation is least tested and most likely to be wrong. The repair would buy sensitivity in the low-traffic regime at the cost of the one guarantee that makes the low-traffic regime survivable.

The fixed rate is therefore the correct choice, and the attenuation cost is accepted as the price of contamination resistance. What the deployment gets in exchange is a bound it can reason about: no cell's remembered reputation outlives the horizon, whoever stops sending labels and for whatever reason.
