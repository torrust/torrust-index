# Whether the clock record earns its place · `rep:assayer:temporal-record-verdict`

This report is evidence for a ruling not yet made, and it is not the ruling. The derivation's open register poses the question in these terms: the temporal choice was given a record of its own at the cost of a three-decision record, the alternative is folding all three into ``durability`` and letting ``posterior`` and ``composition`` cite it there, and the judgment "is the closest call in the set and the one most worth a second reading" (`reg:assayer:record-outline-open`). The reasoning that made the call is at (`dec:assayer:record-clock-record`). This report goes to the code and to the census, changes nothing, and carries one correction: the measured copy-drift the argument is usually summarised with belongs to a different shared function than the one the record would own.

## What the record would hold · `sec:assayer:report-clock-holds`

The layer choice is (`dec:operational:temporal-governance`), which names two expected owners: durability for the persistence half, features and standardisation for the decay inventory's model-side rates. Three of its four statements are the candidate record's; the fourth, that standardisation statistics carry no time-indexed decay, goes to ``composition`` under either ruling and is not in dispute.

**Data (The two domains are separated by type, and one is never persisted)** · `data:assayer:report-clock-domains`

The persistent domain is `PersistentTimestamp` (`src/types.rs:268`), a seconds-and-nanoseconds pair that is serialisable and survives a restart. The intra-process domain is the standard library's monotonic instant, reached through one free function (`src/types.rs:585`). The separation is structural rather than conventional: the persistent type's only conversions are to and from wall-clock system time (`src/types.rs:308`), no conversion exists in either direction between the two domains, and the monotonic type appears nowhere under `src/persistence/` — its occurrences are confined to the in-memory snapshots (`src/snapshot/working.rs:116`, `src/snapshot/published.rs:111`). The outline's claim that the intra-process domain is never persisted is exact.

**Data (The clamp is embedded in both domain accessors, not only the persistent one)** · `data:assayer:report-clock-clamp`

One constant bounds elapsed time at a year (`src/types.rs:260`). The persistent accessor clamps twice — a backward clock jump returns zero (`src/types.rs:358`) and the ceiling applies on the way out (`src/types.rs:367`) — and the intra-process accessor applies the same ceiling from the same constant (`src/types.rs:578`). Neither domain exposes an unclamped elapsed-hours reading, so a caller cannot compute an unclamped interval without leaving the timestamp interface. This is broader than the census recorded: its spot-check located the clamp in the persistent type alone (`reg:assayer:adr-spotchecks`), where the code embeds it in both accessors against one shared constant. That is the stronger property and the one the record should state.

**Data (One funnel, three entries, one exponentiation)** · `data:assayer:report-clock-funnel`

Three shared functions carry all decay, matching the census row exactly. The funnel's mouth is `decay_factor` (`src/numerics.rs:68`) and the single exponentiation in the package's production sources is its last line (`src/numerics.rs:80`). The two domain-specific entries are thin: one takes two persistent timestamps (`src/numerics.rs:111`), one an elapsed duration (`src/numerics.rs:144`), each clamping through its domain's accessor and then delegating. The mouth re-asserts the clamp invariant as a debug assertion (`src/numerics.rs:69`), so the two domains converge on one arithmetic that re-checks what the type discipline already guaranteed. The prohibition is mechanised twice — a lint (`ci/lint_assayer.sh:138`) and an in-tree source audit (`src/tests/owner.rs:1020`) — while the test tree uses the operator deliberately as an independent oracle, recomputing the expected factor from the stated formula rather than through the shared function (`src/tests/ledger.rs:1524`, `src/tests/model_bayesian.rs:849`). The nine environments stand as outlined at (`entry:assayer:record-clock`), the three decisions above joined by a rule, a convention, a discussion, a corollary and two remarks; none restates another record's material.

## The case for the record · `sec:assayer:report-clock-keep`

**Observation (The temporal material is already a source in four records' decisions)** · `obs:assayer:report-clock-multiple-sources`

The two-owner shape is not only predicted drift; the census measured its consequence. In the audit's disposition tables the legacy temporal record is named as a source by rows owned by four different records: durability's once-only restore decay, the learning path's decay-at-the-head, its own four temporal rows, and standardisation. That record runs to 1,109 lines and carries 121 by-number references — the fourth most entangled document in a corpus whose structural defect is duplicate definition (`obs:assayer:adr-duplicate-definition`). Material with one owner does not accumulate that shape.

**Data (The funnel's consumers are overwhelmingly not durability's)** · `data:assayer:report-clock-funnel-reach`

Counting references to the decay and elapsed-time functions across the production sources — excluding the tests, the defining module and the re-export — gives 36 references in nine modules, mapping to seven different records:

| Module | Refs | Record it belongs to |
| --- | --- | --- |
| `src/model/bayesian.rs` | 8 | ``posterior`` |
| `src/risk/challenge.rs` | 7 | ``challenge`` |
| `src/snapshot/working.rs` | 5 | ``retention`` |
| `src/persistence/recovery.rs` | 5 | ``durability`` |
| `src/ledger/entry.rs` | 3 | ``accumulation`` |
| `src/owner/label_path.rs` | 2 | ``ordering`` |
| `src/identity/cell_state.rs` | 2 | ``accumulation`` |
| `src/health/identity_tracker.rs` | 2 | ``health`` |
| `src/assessment.rs` | 2 | ``ordering`` |

Durability's share is five references of thirty-six, near a seventh. A fold would make the owner of a seventh of the consumption the owner of the rule governing all of it, and would put an arithmetic prohibition binding on the posterior, the challenge model, the ledger and the identity trackers inside the record about what survives a restart. The restore path shows the seam: recovery reads the elapsed interval once through the clamped persistent accessor (`src/persistence/recovery.rs:115`), then applies the factor three times, once per model family (`src/persistence/recovery.rs:119`, `src/persistence/recovery.rs:145`). The measuring is the candidate record's subject; the "exactly once" is durability's own decision. They meet at one line and are not the same decision.

**Observation (The drift precedent is real but belongs to another record)** · `obs:assayer:report-clock-drift-analogy`

The argument is often stated as though the census caught the *decay* funnel being copied. It did not. The measured case is the regime-transition function: two legacy calibration records each wrote it out in full while one recorded as a consequence that it was "defined once and imported by both", and the code vindicated the consequence and not the bodies (`src/numerics.rs:341`, with a test asserting the sharing at `src/risk/calibration.rs:973`). That material is ``calibration``'s, and the outline already used it there to promote the shared transition to a decision of its own. The two functions are not even the same subject in the source: the defining module separates its decay sections from its shared algorithm section and cites a different legacy record for each. So the drift evidence supports the candidate record by *analogy* — the measured instance of the failure mode a two-owner statement invites — and not as a direct finding about the funnel. The direct evidence is the multiple-source count above.

## The case for the fold · `sec:assayer:report-clock-fold`

**Observation (The fold is what the census itself proposed)** · `obs:assayer:report-clock-audit-projection`

The strongest argument for folding is that the audit's own projected set contains no temporal record. It splits the legacy temporal material two ways — part to durability and recovery, part to features and standardisation — which is the fold exactly, and follows the layer choice's two named owners without departure. The derivation overrode a proposal, not a silence. Three things bound that argument: the projection binds nothing by its own terms (`cav:assayer:adr-projection-nonbinding`); it was mined from the legacy corpus rather than derived from the layers, the method the outline replaced; and the layer's owner names are stated as expectations, the same status the two accepted splits depart from. It remains the best evidence for the fold.

Nor is folding blocked by size. Durability holds four decisions in nine environments; absorbing three decisions with their rule, convention, discussion and corollary would put it near sixteen environments and seven decisions — inside the eight-to-twenty band and roughly the density of ``retention``. Under the calculus a citation costs a resolved label either way. On cost alone the fold is affordable, so the case turns on ownership rather than arithmetic.

**Observation (The fold inverts the set's citation gravity)** · `obs:assayer:report-clock-citation-gravity`

This is where the fold's cheap-citation argument fails. Reading the outline's citation spines, the candidate record is cited by four — ``durability``, ``posterior``, ``composition`` and ``ordering`` — tying it with the most-cited records in the set, where durability is cited by two. The fold dissolves a four-citer into a two-citer, and two surviving citations would point at the persistence record for material with no persistence content: the posterior's combined decay factor and the standardisation exception. A citation that has to be explained at its destination — durability answering why it owns a prohibition on arithmetic in the identity trackers — is the shape the campaign exists to remove, not an instance of citation working.

## What the corpus says about small records · `sec:assayer:report-clock-size`

**Data (Nine environments is inside the band, not below it)** · `data:assayer:report-clock-band`

The set's stated criterion is environments, not decisions: the band is roughly eight to twenty, and the fold rule fires for "a cluster below the floor", with an express exemption for a genuine cross-cutting owner. The candidate record has nine — at the floor of the band and inside it, the same count as ``durability`` and one above ``degradation``. What makes it look marginal is its decision count against a mean near eight, and that is not the criterion the derivation set for itself. On the stated criterion the fold rule does not fire, and its exemption would apply if it did.

**Observation (The set already contains this record, and nobody has questioned it)** · `obs:assayer:report-clock-degradation-precedent`

``degradation`` carries three live decisions in eight environments and is justified in the outline on exactly the argument at issue — that its citations replace four restatements otherwise argued from first principles in four other records. It is one environment smaller, equally cross-cutting, and appears in no open question. Whatever ruling is made should be the same for both, or should say what distinguishes them. The corpus's warning that short is not the same as sharp cuts the other way here: its example is short because its subject is small while being undisciplined in every other respect (`cav:assayer:adr-short-not-sharp`), where the registry's test profile is the positive case at five environments and 638 words. Nine environments over a type discipline, a clamp and a funnel, each mechanised and verifiable in the source, is the second shape.

## What the evidence supports · `sec:assayer:report-clock-verdict`

**Summary (Keep the record, with high confidence)** · `summ:assayer:report-clock-verdict`

Keep it. Confidence high, on three findings that hold independently. By the set's own stated measure the record is inside the size band and the same size as the record it would fold into, so the premise that it is undersized does not survive the criterion. Its funnel serves seven records while durability accounts for about a seventh of the consumption, so the fold would house a cross-cutting rule in a record owning a small minority of what it binds. And it is cited by twice as many records as durability, so the fold inverts rather than simplifies the citation structure.

Confidence is high rather than decisive for two reasons, both recorded rather than smoothed: the audit's own projection folded this material, so a reader who weights the census's mining above the layer-derived method has a real argument rather than an oversight; and the drift evidence usually cited here is by analogy, the direct evidence being the multiple-source count. One residual is not this question at all — the area word is weaker than its content, since the record governs decay as much as time-telling, which belongs with the two weak area words already in the open register.

**Register (What each ruling costs mechanically)** · `reg:assayer:report-clock-mechanics`

- **Keep.** Nothing changes. The record is written in its cluster wave against the nine environments already outlined, with the two precisions this report found folded into its drafting: the clamp stated over both accessors rather than the persistent one, and the funnel's decision citing the three-function shape and the two mechanisations of the prohibition. The open question closes as decided, and no other record is touched.
- **Fold.** Five edits to the outline. The master table loses its row and the set becomes seventeen scoped records. The decision that minted it is struck or inverted, and the rename table's new-area row goes with it. The coverage arithmetic moves three decisions into durability, four to seven, and the sum stays at 142. The durability entry absorbs the three decisions, the rule, the convention, the discussion and the corollary, and the two remarks resolve — persistence inheritance becomes internal to it, the standardisation exception stays as ``composition``'s citation target. And four citation spines change: durability drops its outgoing citation and gains the material, while ``posterior``, ``composition`` and ``ordering`` re-point theirs at durability.
- **The timing cost is one file.** The durability record is being written in the cluster wave now in flight, so a fold ruled after that wave lands means amending a written record rather than declining to write a new one. The three re-pointed spines are free while their records are unwritten: ruled before those waves the fold costs one amendment, ruled after, one amendment and three edits.

**Caveat (What this report did not do)** · `cav:assayer:report-clock-scope`

It did not read the in-flight record files: the durability scope it reasons against is the outline's, and if that record has moved its own boundaries the fold arithmetic moves with it. It did not verify that the candidate record's environments each resolve to a specification citation — the general limitation the outline records for the whole set, settled per record at the rewrite. And it did not reopen the fourth temporal statement, which goes to ``composition`` under both rulings.
