# The Assayer Decision-Record Audit · `rep:assayer:decision-mining-census`

This is the census and audit of the 37 decision records under `packages/assayer/adr`, the mining entry of the record pipeline in [the campaign backlog](backlog.md) (`rep:assayer:decision-mining-census`). It reads the records as ore, not as artifacts: under the projection ruling (`dec:assayer:projection-principle`) no record has tenure, so the unit of audit is the individual decision, not the file that carries it. For each decision the audit asks what was actually decided — as distinct from what the record explains, restates from the specification, or narrates about implementation history — whether the decision still binds the code, and which conceptual choice of which layer it serves. The output is organised by layer and conceptual choice, and every live decision is assigned a home in a projected record set that parameterizes the outline entry (`plan:assayer:decision-record-architecture`). No record is edited by this wave.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in fenced blocks and double-backtick spans is displayed without participating. Every label minted here has area `assayer`, and every kind is drawn from a fixed registry of environment kinds, under the grammar of the label calculus. Legacy record numbers of the superseded naming scheme (`dec:assayer:naming-schema`) are written in code spans throughout, never bare, so that no locator of the old world can be mistaken for a citation in the new one.

## Method and measurement basis · `sec:assayer:adr-audit-method`

**Convention (What the audit counts)** · `conv:assayer:adr-audit-counting`

A *decision* is a recorded choice between alternatives that binds the implementation: it could have gone the other way, and the record says which way it went. Four things are counted separately and are not decisions. A *rationale* argues for a decision already stated. A *restatement* reproduces a definition the specification or another record owns. A *narration* reports implementation status, history, or sequencing. A *measurement* reports a cost, size, or latency observed from an implementation. Counts of decisions below are counts of the first kind alone; the corpus's line budget is reported against all five.

Line, block, and reference counts are taken mechanically over the 37 files. A *fenced-block line* is any line inside a fenced region including its two delimiters; a *table row* is a line beginning with a pipe outside such a region; *prose* is what remains after fenced blocks, table rows, and blank lines are removed.

**Data (Measurement basis)** · `data:assayer:adr-audit-basis`

Measured 2026-08-19 against the tree at backlog commit `d886ba4a`. Subjects: the 37 files `packages/assayer/adr/[1-5]NN-*.md`, 42,454 lines and 1.99 MB in total; the five layer documents `packages/assayer/docs/layers/layer[1-5]-*.md`; and the hand-maintained master register `packages/assayer/docs/adrs.md` at 1,697 lines. Code was checked at `packages/assayer/src`, 168 Rust files in the package's own tree and 693 in the test profile's census. The shape benchmark is `adr/014-label-calculus.md` at 272 lines, with `adr/028-environment-kinds.md` at 660 lines as the upper comparison for a record that must carry large registry tables.

**Convention (Disposition vocabulary)** · `conv:assayer:adr-disposition-vocabulary`

Every mined decision carries exactly one of four dispositions. **LIVE** — the decision still binds the code, and the rewritten set must carry it. **SUPERSEDED** — a later record, or the specification rewrite, replaced the choice; the decision survives only as the history of a choice now made differently. **ABSORB** — the content defines what the system *is* rather than recording a choice between alternatives, and belongs in the specification; the disposition is a candidate for the spec pipeline (`plan:assayer:specification-rewrite-contract`), not a decision this audit may execute. **DEAD** — the decision describes work that was abandoned or never built and that nothing now proposes to build.

A fifth state is deliberately absent: there is no "keep because it exists". Under (`dec:assayer:projection-principle`) that is not a disposition.

**Convention (Sampling discipline for liveness)** · `conv:assayer:adr-liveness-sampling`

Liveness was not verified exhaustively, and this audit claims no such coverage. Twenty-nine checks were made, each read in the source and never inferred from a name match, chosen by three rules: every decision whose record self-declares a status weaker than the register claims for it, since that disagreement is itself evidence of drift (`reg:assayer:adr-status-contradictions`); every decision named in a contradiction below, since adjudicating one against the code is the only way to say which side is live; and any decision whose check was cheap and decisive — a public type, an exported function, a dependency, a trait. Checks requiring a pipeline to be read end to end were skipped. Cost, latency, and memory figures were not re-measured at all. Each check is cited by ``path:line`` in (`reg:assayer:adr-spotchecks`).

## The corpus as found · `sec:assayer:adr-corpus`

**Data (Headline shape)** · `data:assayer:adr-corpus-shape`

42,454 lines over 37 files: a mean of 1,147 lines per record against the 272-line benchmark, a factor of 4.2. The corpus divides as 16,697 lines inside fenced blocks (39.3%), 16,523 lines of prose (38.9%), 5,668 blank lines (13.4%), and 3,567 table rows (8.4%). It carries 433 fenced blocks and 1,408 heading lines.

The comparison with the benchmark is not the one the size ratio suggests. `adr/014-label-calculus.md` is 38.6% prose against the corpus's 38.9%: the prose *share* is the same. What differs is the remaining three fifths — in the benchmark, 37.1% blank against 21.7% fenced, a budget spent on air; in the corpus, 39.3% fenced against 13.4% blank, a budget spent on transcribed Rust. The record set is, by line, more code than prose.

**Observation (The corpus carries no environments)** · `obs:assayer:adr-no-environments`

Like the specification (`obs:assayer:spec-nonparticipation`), the record set has no environment heads. Its environment-shaped material sits in 1,408 markdown headings and 883 bold-lead paragraphs: 2,291 candidate sites across 37 records, a mean of 62 per record, against the benchmark's 38 environments in one 272-line document. The benchmark spends roughly 7.2 lines per environment; the corpus spends 18.5 lines per candidate site, and most of those sites are section numbering rather than named concepts. The outline wave inherits an inventory problem of a different shape from the specification's: the specification had 456 tag identities wanting kinds, while the records have no identities at all and want the inventory invented.

**Table (The record set as found)** · `tab:assayer:adr-corpus-inventory`

Lines are file lines. "Self-status" is the record's own status header, verbatim to its first clause. "Register" is what `docs/adrs.md` claims for the same record. Rows where the two disagree are the subject of (`reg:assayer:adr-status-contradictions`).

| Record | Lines | Self-status | Register |
| --- | --- | --- | --- |
| `110-feed-forward-boundary` | 710 | Accepted | Implemented |
| `120-published-state` | 876 | Accepted | Implemented |
| `130-dynamic-dimension-bayesian-model-type` | 1,508 | Accepted | Implemented |
| `140-structure-before-observation` | 1,324 | Accepted | Implemented |
| `150-learning-thread` | 929 | Accepted | Implemented |
| `160-state-persistence-and-recovery` | 1,401 | Accepted (revised) | Accepted; follow-ups open |
| `170-error-model-and-numeric-robustness` | 989 | Accepted (revised) | Implemented |
| `210-posterior-matrix` | 1,153 | Accepted | Implemented |
| `220-linear-algebra-foundation` | 934 | Accepted | Implemented |
| `230-published-snapshot` | 1,027 | Accepted | Implemented |
| `240-pending-assessment-record` | 939 | Accepted | Implemented |
| `250-outcome-ledger` | 954 | **Proposed** | **Implemented** |
| `260-key-space-identity-layer` | 1,094 | Accepted | Implemented |
| `270-signal-cache` | 1,034 | Accepted | Implemented |
| `310-assessment-pipeline` | 878 | Accepted | Implemented |
| `320-learning-pipeline` | 1,480 | Accepted | Implemented |
| `330-sentinel-report-reception` | 713 | Accepted | Implemented |
| `340-temporal-governance` | 1,109 | Implemented | Implemented |
| `350-feature-vector` | 1,597 | Proposed | Proposed, supersedes previous |
| `360-online-standardisation` | 1,710 | Proposed | Partly implemented |
| `370-initialisation-and-convergence` | 1,538 | Proposed | Proposed |
| `410-leverage-bounded-update` | 678 | Accepted | Implemented |
| `420-cholesky-recomputation-strategy` | 1,491 | **Proposed** | **Implemented** |
| `430-schur-complement-for-marginalisation` | 1,008 | Accepted | Implemented |
| `440-resonance-derivation` | 1,204 | **Proposed** | **Implemented** |
| `450-platt-calibration` | 1,006 | **Proposed** | **Implemented** |
| `460-subspace-restricted-blend` | 1,158 | Implemented (Layer 4) | Implemented |
| `470-challenge-effectiveness-model` | 406 | Proposed | Proposed |
| `510-assessment-contract` | 1,691 | Accepted; canonicalized | Implemented |
| `520-label-contract` | 1,175 | **Proposed** | **Implemented** |
| `530-report-reception-contract` | 1,256 | **Proposed** | **Implemented** |
| `540-construction-contract` | 1,721 | Accepted (revised) | Accepted; migration pending |
| `550-health-and-diagnostics` | 2,006 | **Proposed** | **Implemented** |
| `560-lifecycle-contract` | 1,745 | **Proposed** | **Implemented** |
| `570-metrics-and-observability` | 1,337 | Accepted (complete) | Implemented |
| `580-label-guidance-api` | 232 | Accepted (complete) | Implemented |
| `590-companion-tracker-contract` | 443 | Proposed | Proposed |

**Data (The legacy reference load)** · `data:assayer:adr-legacy-load`

The record set carries 2,701 record references by number of the `ADR-x-NNN` form and 2,533 section-sign locators into the specification: 5,234 legacy reference forms, a mean of 141 per record, none of which resolves under the calculus. This is the volume the naming ruling (`dec:assayer:naming-schema`) must move inside the records themselves, beside the 984 such references the code carries and the 748 in the rest of the docs tree (`obs:assayer:drift-code-citations`). The record set is the largest single reservoir of the old scheme in the repository, holding more by-number references than the code and the remaining documentation combined.

The distribution is not uniform. `510-assessment-contract` carries 142 record references, `560-lifecycle-contract` 139, `550-health-and-diagnostics` 131, and `340-temporal-governance` 121; `580-label-guidance-api` carries 17 and `470-challenge-effectiveness-model` 21. The two shortest records are also the two least entangled, which is the same fact seen twice.

**Observation (Five references leave the corpus)** · `obs:assayer:adr-cross-corpus-refs`

The 2,701 by-number references name 39 distinct records, but only 37 records exist. The two extras are not dangling: they are `ADR-S-002` (four occurrences) and `ADR-M-036` (one), records of the Sentinel and Mudlark corpora. Under the calculus these are the only five references in the record set that are genuinely *imports* rather than same-owner citations, and the naming-schema entry (`dec:assayer:record-naming-schema-text`) must register a prefix for each upstream corpus before they can be written. No reference in the record set points at a record of the Assayer's own set that does not exist; the by-number scheme is internally complete, and fails only by being unresolvable rather than by dangling.

## The projection has no middle term · `sec:assayer:adr-projection-gap`

**Observation (The layer documents are rollups, not conceptual outlines)** · `obs:assayer:adr-layers-not-outlines`

The projection ruling (`dec:assayer:projection-principle`) fixes three terms: the specification's concepts, the layers' conceptual choices, and the record set derived from them. The middle term does not exist. All five layer documents are organised *by record number* — the body of each is a run of sections headed `## N. ADR-L-NNN: Title`, each with a "The Decision" subsection restating that record's decision — so they project nothing from the specification. They are summaries of the records, inheriting the record partition wholesale rather than fixing the choices from which a partition could be derived.

Two consequences follow, and they run in opposite directions. The registers entry (`reg:assayer:decision-record-register`) is therefore not a reformatting task: the layer documents must be *written*, not converted, because the conceptual choices they are supposed to fix have never been stated anywhere. And this audit cannot read the conceptual choices off the corpus; it must derive them, which is the next environment's business.

**Requirement (The conceptual choices below are this audit's derivation)** · `req:assayer:adr-choices-derived`

The conceptual choices that organise (`sec:assayer:adr-disposition`) are derived by this audit from the specification's chapter concepts and the mined decisions together. They are the audit's reading, offered as the starting point for the registers entry (`reg:assayer:decision-record-register`), and they bind nothing: the layer outlines may fix a different set, and if they do, the disposition table's grouping changes with them while the dispositions themselves — which decision is live, superseded, absorbed, or dead — do not. The two judgments are separable on purpose, so that a disagreement about the layer's structure cannot reopen the audit of the record content.

Where a derived choice does not correspond to any current record boundary, that is the finding: it is evidence that the 37-record partition is a partition of *topics as they were written down*, not of *choices the system makes*.

## What the corpus says about itself · `sec:assayer:adr-status`

**Register (Records whose own status contradicts the register)** · `reg:assayer:adr-status-contradictions`

Eight records declare themselves **Proposed** in their own status header while `docs/adrs.md` declares them *Implemented*: `250-outcome-ledger`, `420-cholesky-recomputation-strategy`, `440-resonance-derivation`, `450-platt-calibration`, `520-label-contract`, `530-report-reception-contract`, `550-health-and-diagnostics`, and `560-lifecycle-contract`. In every one of the eight the code agrees with the register and not with the record: the decisions are built. The records are stale in the safe direction — they understate their own standing — but the disagreement means neither document can be trusted as the status source, and eight of 37 is too many to reconcile by hand a second time.

This is the standing rule (`goal:assayer:tools-over-discipline`) seen from the inside, exactly as the specification's empty index illustrated it (`obs:assayer:spec-tag-index-empty`): a status recorded in two places by hand disagreed in 22% of cases within one campaign.

**Observation (The status vocabulary is not a vocabulary)** · `obs:assayer:adr-status-vocabulary`

The 37 status headers use seven distinct forms across three families: Accepted (16), Accepted with a revision date (3), Accepted with a qualifying clause (3), Proposed (13), and Implemented (2). Nothing fixes what they mean. "Accepted" and "Implemented" are used for records in identical standing — `340-temporal-governance` says Implemented and `330-sentinel-report-reception` says Accepted, and both are built and binding — while "Proposed" covers both the eight stale records above and the genuinely unbuilt work of `590-companion-tracker-contract`. A status that does not separate *decided* from *built* cannot answer the only question a reader asks of it. The rewritten set should carry no status field at all: a record that states a decision the code does not implement should say so in a caveat naming the gap, which is a claim the linter and the reader can both check, and the deferral register already demonstrates the form.

## The disposition, by layer and conceptual choice · `sec:assayer:adr-disposition`

The tables below are the audit's core. Each row is one mined decision, grouped under the conceptual choice it serves rather than under the record that carries it, so a choice served by three records appears once with three sources. The "Source" column names records in code spans; the "Disposition" column uses the vocabulary of (`conv:assayer:adr-disposition-vocabulary`); "Evidence" names the spot-check where one was made, by its number in (`reg:assayer:adr-spotchecks`).

**Table (Structural commitments)** · `tab:assayer:adr-disposition-structural`

| Conceptual choice | Decision | Source | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| Direction of information flow | Reports cross the boundary as owned values; no Sentinel handle is ever held | `110` | LIVE | 1 |
| Direction of information flow | The Assayer owns its identity graphs outright; the boundary is external only | `110`, `260` | LIVE | 2 |
| Direction of information flow | Coordinates are fixed at 128 bits; the package is not generic over the type | `110` | LIVE | 3 |
| Direction of information flow | The import boundary is enforced by an allowlist lint, not by convention | `110`, `440` | LIVE | 4 |
| Publication discipline | Learned state is published as an immutable snapshot swapped atomically | `120` | LIVE | 5 |
| Publication discipline | Readers load per request, not per batch, so a mid-batch publish is visible | `120`, `310` | LIVE | — |
| Publication discipline | Per-Sentinel report indices use the same swap discipline independently | `120`, `330` | LIVE | 6 |
| Write serialisation | One steward owns the working copy exclusively; there is no shared mutable model state | `140` | LIVE | 7 |
| Write serialisation | Two channels, and structure preempts observation: commands drain before any label | `140`, `150` | LIVE | 8 |
| Write serialisation | Neither channel may silently drop; overflow is an error return | `140`, `150`, `520` | LIVE | — |
| Write serialisation | The steward is a dedicated named thread, not a pool or an async task | `150` | LIVE | 9 |
| Durability | Periodic whole-state checkpoint plus a self-contained journal for in-flight labels | `160` | LIVE | 10 |
| Durability | No persistence work on the assessment path at all | `160` | LIVE | — |
| Durability | Restore re-applies elapsed decay exactly once; replayed labels see no further decay | `160`, `340` | LIVE | — |
| Durability | Structural compatibility is checked on restore; numerical parameter changes are permitted | `160`, `540` | LIVE | — |
| Failure posture | Core assessment is infallible; anomalies are reported in band as degradation | `170`, `510` | LIVE | 11 |
| Failure posture | Matrix pathologies retain the best available state rather than failing the call | `170`, `420` | LIVE | — |
| Failure posture | Structural contract violations are hard errors; data-quality problems degrade per cell | `170`, `530` | LIVE | 12 |
| Failure posture | Six numeric checkpoints are placed from the boundary inward | `170`, `310`, `320` | ABSORB | — |
| Model representation | The model is dense and dynamically dimensioned, reallocated fresh on lifecycle change | `130` | LIVE | 13 |
| Model representation | The anchor model is never extended or marginalised | `130`, `560` | LIVE | 14 |
| Model representation | The assessment path reads the covariance only, never the precision matrix | `130`, `230` | LIVE | — |

**Table (Representations)** · `tab:assayer:adr-disposition-representation`

| Conceptual choice | Decision | Source | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| Matrix substrate | Symmetry is a type invariant restored after every mutating method | `210` | LIVE | 15 |
| Matrix substrate | No raw mutable access is exposed; all mutation is through named operations | `210` | LIVE | 15 |
| Matrix substrate | Two constructor trust levels separate computed input from stored input | `210` | LIVE | — |
| Matrix substrate | A single linear-algebra backend serves the whole workspace | `220` | LIVE | 16 |
| Matrix substrate | The wrapper isolates the backend so a switch touches no caller | `210`, `220` | LIVE | — |
| Matrix substrate | Serialisation is written by hand rather than derived from the backend | `220`, `160` | LIVE | — |
| Published state composition | One monolithic snapshot, one allocation, one swap | `230` | LIVE | — |
| Published state composition | The precision matrix is excluded and reconstructed on demand | `230` | LIVE | 17 |
| Published state composition | Health state is published through a second, independent swap | `230`, `550` | LIVE | 18 |
| Published state composition | Companion state is excluded from the model snapshot entirely | `230`, `590` | LIVE | 19 |
| In-flight retention | Assessments awaiting a label are held in a concurrent map with a separate eviction queue | `240` | LIVE | 20 |
| In-flight retention | Features are retained at reduced precision to bound the buffer | `240` | LIVE | — |
| In-flight retention | Eviction is lazy and capped per insert rather than swept | `240` | LIVE | — |
| In-flight retention | The journal-serialisable subset omits what replay does not consume | `240`, `160` | LIVE | — |
| In-flight retention | Entity-persistent signals are cached pre-encoded, not re-encoded per read | `270` | LIVE | 21 |
| In-flight retention | The signal cache is not persisted and repopulates through ordinary traffic | `270` | LIVE | — |
| Spatial accumulation | Outcome state is keyed per Sentinel by a coordinate-and-depth pair | `250` | LIVE | — |
| Spatial accumulation | Reads walk depth and exit at the first hit; writes visit every containing layer | `250` | LIVE | — |
| Spatial accumulation | The root entry is permanent; deletion neither merges nor inherits | `250` | LIVE | — |
| Spatial accumulation | Collection is a bounded periodic sweep, not lazy on access | `250` | LIVE | — |
| Spatial accumulation | Per-axis rows materialise on first reference rather than at axis registration | `250`, `320`, `560` | LIVE | — |
| Identity layer | A dedicated thread owns every identity graph; assessment defers observations to it | `260` | LIVE | 22 |
| Identity layer | Competitive sets are published per dimension by the same swap discipline | `260` | LIVE | — |
| Identity layer | Observation queue overflow is degradation, not a hard error | `260`, `170` | LIVE | — |
| Identity layer | The observation surface accepts a coordinate only, structurally enforcing feed-forward | `260`, `110` | LIVE | — |
| Identity layer | Identity and Ledger spatial keys are distinct types despite identical structure | `250`, `260` | LIVE | — |

**Table (The operational cycle)** · `tab:assayer:adr-disposition-cycle`

| Conceptual choice | Decision | Source | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| Assessment path | One monolithic per-request function with factored helpers, not a stage pipeline | `310` | LIVE | — |
| Assessment path | Batch processing is sequential, sharing one timestamp across the batch | `310` | LIVE | — |
| Assessment path | The Core path ends at the risk assessment; derivation is an explicit host step | `310`, `510` | LIVE | 23 |
| Assessment path | The only Core information crossing into derivation is the risk sufficient statistic | `310`, `440`, `510` | LIVE | 4 |
| Learning path | One monolithic label function in five logical groups, executed sequentially | `320` | LIVE | — |
| Learning path | Time decay is applied once at the head of the path, before any model update | `320`, `340`, `410` | LIVE | — |
| Learning path | Both the model snapshot and the health summary publish at the end of the path | `320`, `550` | LIVE | 18 |
| Learning path | The seventeen-step enumeration itself | `320` | ABSORB | — |
| Report ingestion | Reception is synchronous and serialised per Sentinel by a reception lock | `330`, `530` | LIVE | 6 |
| Report ingestion | Validation is two-phase: structural failures are hard, data quality degrades | `330`, `530`, `170` | LIVE | 12 |
| Report ingestion | Ledger maintenance completes before the new index becomes visible | `330`, `530` | LIVE | — |
| Report ingestion | Lock ordering is fixed: reception before Ledger | `330` | LIVE | — |
| Report ingestion | Reports carry no sequence; the last write wins and staleness is a monitoring concern | `530`, `550` | LIVE | — |
| Temporal governance | Two timestamp domains are separated by type: intra-process and persistent | `340` | LIVE | 24 |
| Temporal governance | The decay clamp is embedded in the timestamp API so callers cannot compute unclamped | `340` | LIVE | — |
| Temporal governance | All decay flows through three shared functions; direct exponentiation is prohibited | `340` | LIVE | — |
| Temporal governance | Standardisation statistics carry no time-indexed decay | `340`, `360` | LIVE | — |
| Feature composition | One canonical block order, fixed and total | `350` | LIVE | — |
| Feature composition | A single dimension map is the sole resolver of every feature index | `350` | LIVE | — |
| Feature composition | Interaction templates name operands semantically and are compiled at lifecycle rebuild | `350` | LIVE | — |
| Feature composition | Interactions are computed from unstandardised bases, then standardised once | `350`, `360` | LIVE | — |
| Feature composition | Label-time assembly freezes some blocks and re-derives others | `350`, `320` | LIVE | — |
| Feature composition | The block-by-block width accounting and dimension formula | `350` | ABSORB | — |
| Standardisation | Statistics are observed at assessment time and updated at label time | `360` | LIVE | — |
| Standardisation | Two acquisition mechanisms — batch initialisation and per-Sentinel bootstrap | `360` | LIVE | — |
| Standardisation | Accumulators are transient and deliberately not checkpointed | `360` | LIVE | — |
| Standardisation | Feature classes carry per-class priors derived from the dimension map | `360` | LIVE | — |
| Convergence | Independent concrete trackers per process; no shared trait, no global state machine | `370` | LIVE | — |
| Convergence | The composite stage is computed on demand and may regress after lifecycle change | `370`, `510` | LIVE | — |
| Convergence | Convergence is diagnostic and gates nothing | `370` | LIVE | — |
| Convergence | Early refit is triggered unconditionally by any registration or deregistration | `370`, `460`, `560` | LIVE | — |
| Convergence | The blend-weight-conditioned early-refit check | `370` | SUPERSEDED | — |

**Table (Mathematical foundations)** · `tab:assayer:adr-disposition-numerics`

| Conceptual choice | Decision | Source | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| Incremental update | Read, policy, and mutation are separated into three steps | `410` | LIVE | — |
| Incremental update | Time and label decay combine into one factor computed once per model per label | `410`, `320` | LIVE | — |
| Incremental update | An algebraic identity replaces one matrix-vector product per model | `410` | LIVE | — |
| Incremental update | Leverage is bounded by policy before the update is applied | `410` | LIVE | — |
| Incremental update | The derivation of the update identity and its cost accounting | `410` | ABSORB | — |
| Numerical repair | Repair is a two-phase cascade: plain factorisation first, regularised on failure | `420` | LIVE | 25 |
| Numerical repair | The recomputation trigger is per model and fires on either a counter or a conditioning estimate | `420` | LIVE | — |
| Numerical repair | Poor outcomes halve the interval; sustained clean operation restores it | `420` | LIVE | — |
| Numerical repair | One shared inversion utility serves all three call sites | `420`, `430`, `170` | LIVE | 25 |
| Numerical repair | The cascade terminates by retaining state and flagging, never by failing | `420`, `170` | LIVE | — |
| Marginalisation | The correction is computed by a half-solve Gram matrix rather than a full solve | `430` | LIVE | — |
| Marginalisation | The operation is infallible, falling back to the uncorrected block | `430` | LIVE | — |
| Marginalisation | A conditioning guard skips the correction rather than attempting it | `430` | LIVE | — |
| Marginalisation | The full solve is retained as a debug-only cross-check oracle | `430` | LIVE | — |
| Marginalisation | The covariance is always refactored, never taken from the extracted block | `430`, `130` | LIVE | — |
| Derivation | The derivation function is pure, infallible, and free of Core state | `440` | LIVE | 26 |
| Derivation | Channel constants are recomputed per call because one depends on evolving evidence | `440` | LIVE | — |
| Derivation | The module's imports are restricted by an allowlist enforced as a test | `440` | LIVE | 4 |
| Derivation | The action set is ordered and the function is generic over it | `440` | LIVE | — |
| Derivation | Exploration metrics are derivation-layer quantities the Core neither computes nor consumes | `440`, `510`, `580` | LIVE | — |
| Derivation | The six-step dependency chain and the kernel evaluation guard | `440` | ABSORB | — |
| Calibration | Fitting is a factored per-regime search on a transformed parameter | `450` | LIVE | — |
| Calibration | The refit is a pure function of buffer and configuration, with no side effects | `450` | LIVE | — |
| Calibration | A large calibration shift conservatively resets every drift accumulator | `450` | LIVE | — |
| Calibration | Regime weighting uses a transition function shared with the blend | `450`, `460` | LIVE | 27 |
| Blend | The projected quadratic form is computed by index without allocation | `460` | LIVE | — |
| Blend | Blend weight uses raw forms because the time correction cancels in the ratio | `460` | LIVE | — |
| Blend | Every quadratic form is clamped non-negative | `460` | LIVE | — |
| Challenge estimation | A conjugate Beta-Binomial model with lazy decay toward the prior | `470` | LIVE | 28 |
| Challenge estimation | Override does not pause accumulation; clearing recovers the evolved state | `470` | LIVE | 28 |
| Challenge estimation | Injected evidence is indistinguishable from observed evidence and is capped | `470` | LIVE | 28 |

**Table (Contracts and boundaries)** · `tab:assayer:adr-disposition-contracts`

| Conceptual choice | Decision | Source | Disposition | Evidence |
| --- | --- | --- | --- | --- |
| Assessment surface | The call takes a batch and returns a result per request, with no error channel | `510` | LIVE | 11 |
| Assessment surface | Channel policy is not an input; it belongs to host-side derivation | `510`, `540` | LIVE | — |
| Assessment surface | Health travels inline on every result, distinct from the on-demand query type | `510`, `550` | LIVE | — |
| Assessment surface | Encoding is performed internally; the host supplies raw values | `510` | LIVE | — |
| Assessment surface | An identifier is assigned per assessment and is the label's only handle | `510`, `520` | LIVE | — |
| Assessment surface | The field-by-field structure of the result and its risk basis | `510` | ABSORB | — |
| Label surface | Submission is asynchronous and acknowledges only what is known synchronously | `520` | LIVE | — |
| Label surface | The pending entry is consumed at the boundary, before journalling, irreversibly | `520` | LIVE | — |
| Label surface | Anomalous values are sanitised at the boundary rather than rejected | `520` | LIVE | — |
| Label surface | There is no idempotency, no batch variant, and no completion variant | `520` | LIVE | — |
| Label surface | The valence sign convention: positive is adverse | `520` | ABSORB | — |
| Label surface | What the host did and what the derivation proposes are distinct types | `520`, `440` | LIVE | — |
| Report surface | Reception is synchronous and acknowledges with maintenance diagnostics | `530` | LIVE | — |
| Report surface | The slot map is a concurrent map keyed by Sentinel, distinct from the feature index map | `530` | LIVE | 6 |
| Report surface | Reception touches no model, identity, cache, or calibration state | `530` | LIVE | — |
| Construction surface | A builder validates schema, templates, and parameters eagerly | `540` | LIVE | — |
| Construction surface | Configuration is Core-only; derivation and Companion configuration are host-owned | `540`, `590` | LIVE | — |
| Construction surface | Construction either cold-starts or restores; a structural mismatch cold-starts | `540`, `160` | LIVE | — |
| Construction surface | Threads are spawned at build and named per instance | `540`, `150` | LIVE | 9 |
| Construction surface | Pre-seeding is post-construction and synchronous | `540` | LIVE | — |
| Construction surface | The nine-phase build sequence and the parameter inventory | `540` | ABSORB | — |
| Health surface | Health is published independently of the model snapshot | `550`, `230` | LIVE | 18 |
| Health surface | Events are pushed through one bounded channel that drops with a counter on overflow | `550`, `370`, `420` | LIVE | — |
| Health surface | Queries are tiered by cost, from a cheap summary to a full report | `550` | LIVE | — |
| Health surface | Discrimination metrics are computed once at refit and cached, not on query | `550`, `450` | LIVE | — |
| Health surface | The report type is non-exhaustive so fields may be added compatibly | `550` | LIVE | — |
| Health surface | The catalogue of health sub-types and their field lists | `550` | ABSORB | — |
| Lifecycle surface | Six operations, all taking a shared receiver and returning a result | `560` | LIVE | 29 |
| Lifecycle surface | Visibility is two-phase: infrastructure synchronously, model asynchronously | `560` | LIVE | — |
| Lifecycle surface | Identity deregistration blocks until the snapshot has stopped referencing the dimension | `560` | LIVE | — |
| Lifecycle surface | A compound batch continues past non-fatal failures and publishes once | `560` | LIVE | — |
| Lifecycle surface | Deregistration is destructive by default and hibernating on request | `560` | LIVE | — |
| Lifecycle surface | The cross-entity cascade tables and per-operation timings | `560` | ABSORB | — |
| Metrics surface | Export is passive: the package renders nothing and depends on no monitoring crate | `570` | LIVE | — |
| Metrics surface | The catalogue is a compile-time constant | `570` | LIVE | — |
| Metrics surface | A neutral sample type is the intermediate form the host renders from | `570` | LIVE | — |
| Metrics surface | Cardinality is bounded by construction and documented per dimension | `570` | ABSORB | — |
| Metrics surface | The encoding rules for enumerations, absent values, and booleans | `570` | ABSORB | — |
| Guidance surface | Guidance is a read-only, infallible, synchronous query | `580` | LIVE | 29 |
| Guidance surface | Three independently ranked categories; duplication across them is allowed and recorded | `580` | LIVE | — |
| Guidance surface | Every scan is explicitly bounded, and an exhausted budget shortens output rather than failing | `580` | LIVE | — |
| Guidance surface | Guidance exposes no derivation quantity, model internal, or Companion state | `580`, `510` | LIVE | — |
| Companion surface | Companion state is host-owned and reaches no Core structure | `590`, `230` | LIVE | 19 |
| Companion surface | Estimation is infallible, returning prior, override, or posterior | `590`, `470` | LIVE | 28 |
| Companion surface | The tracker is replaceable through a published trait (`claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts`) | `590` | LIVE | 19 |
| Companion surface | A standalone Companion health surface, separate from Core health | `590` | LIVE | 19 |
| Companion surface | A Companion metrics mapper under its own name prefix | `590`, `570` | DEAD | 19 |

**Data (Disposition totals)** · `data:assayer:adr-disposition-totals`

At the dated audit, the five tables carried 158 distinct decisions: 142 LIVE, 1 SUPERSEDED, 12 ABSORB, and 3 DEAD. Their standing verdicts now carry 144 LIVE, 1 SUPERSEDED, 12 ABSORB, and 1 DEAD. Against 42,454 lines that is one distinct decision per 269 lines of record text — which is, to within three lines, the entire length of the benchmark record. The corpus spends a whole `adr/014-label-calculus.md` per decision it makes.

Two counts must be kept apart, and their ratio is itself a finding. Read record by record, without deduplication, the corpus states **493 decisions**: 115 in the structural series, 51 in the representations, 91 in the operational cycle, 96 in the mathematical foundations, and 140 in the contracts. Deduplicated across records that decide the same thing — the shared function written out twice, the interface specified by both records that use it, the slot structure defined five times — those 493 statements reduce to the 158 distinct decisions tabulated above. The corpus states each decision **3.1 times on average**.

That ratio is the shrinkage argument in one number, and it is measured rather than projected. It also locates the redundancy: the representations series states 51 decisions across 7,135 lines and is the least redundant of the five, while the structural series states 115 across 7,744 — the difference is not subject matter but the density of restatement in the cluster where `140`, `150`, and `160` overlap, which is also where the divergences of (`reg:assayer:adr-contradictions-records`) concentrate.

The ABSORB count is the load-bearing one and it is deliberately conservative: it names only material this audit is confident defines the system rather than a choice about it — pipeline step enumerations, field inventories, width accounting, encoding rules, and the mathematical derivations of the numerics layer. Those twelve rows account for a disproportionate share of the corpus's bulk, because they are precisely the material that attracts fenced blocks and tables. The 39.3% of the corpus inside fenced blocks (`data:assayer:adr-corpus-shape`) is overwhelmingly attached to ABSORB rows and to the restatement documented in (`obs:assayer:adr-duplicate-definition`), not to the 142 live decisions, which are almost all statable in a sentence apiece.

## Overlaps and contradictions · `sec:assayer:adr-contradictions`

**Register (Contradictions between records)** · `reg:assayer:adr-contradictions-records`

Each was verified by reading both texts, and where the code could adjudicate, the source decides. Ordered by severity.

- **One record grounds its central decision in a precedent it misquotes four ways.** `270-signal-cache` defends its lock choice by appeal to `250-outcome-ledger`, stating that the latter's Guarantee Verification table "marks the `assess()` non-delay property as 'Operationally yes' (RwLock write hold ~800 ns, reader wait probability <0.01%)" (`270-signal-cache.md:312`). Read against `250`'s actual table (`250-outcome-ledger.md:904`–`:917`): there is no `assess()` non-delay row; no verdict in the table reads "Operationally yes" — every one reads "Yes"; the reader-wait figure is 5×10⁻⁶ "by arithmetic" (`250-outcome-ledger.md:907`), so the quoted 10⁻⁴ is the *specification bound* `250` is verifying against, twenty times looser than `250`'s result; and ~800 ns is `250`'s figure for labels, not for the write hold, whose worst case is ~50 μs — which `270` cites correctly three lines earlier, using both figures for one quantity. "Operationally yes" appears in exactly one place in the corpus: `270`'s own table (`270-signal-cache.md:981`), where `270` grades itself. An Accepted record rests its argument on a self-citation laundered through a Proposed one.
- **Two records specify opposite mirror directions, and the one that is wrong says so without amending itself.** `210-posterior-matrix` writes the lower triangle and mirrors lower to upper (`210-posterior-matrix.md:319`, `:411`, `:601`). `220-linear-algebra-foundation` specifies the upper triangle mirrored to the lower, attributing it to `210` (`220-linear-algebra-foundation.md:547`, `:552`, `:727`), and specifies a helper for it at length (`:742`–`:750`). Its own review table then concedes that `210` governs, that the implementation writes lower and mirrors lower to upper, and that the helper "was never implemented" (`220-linear-algebra-foundation.md:844`, `:847`) — and the body was never corrected, so the record asserts both. The code decides for `210`: mutation writes the lower triangle and calls a lower-to-upper mirror (`src/linalg/symmetric.rs:434`, `:166`, `:187`), and no such helper exists.
- **The numerical-repair interface is defined incompatibly by the two records that share it.** `420-cholesky-recomputation-strategy` passes a context struct to the shared inversion utility (`420-cholesky-recomputation-strategy.md:232`, `:634`, `:973`, `:1072`); `430-schur-complement-for-marginalisation` passes a bare call site (`430-schur-complement-for-marginalisation.md:554`) and documents the divergence as a departure from "the ADR's original sketch" (`:603`–`:604`) without amending the record that holds the sketch. The two also give the marginalisation entry point different signatures (`420-…:958`, `430-…:508`) and its failure type different shapes, both citing `130` as the source. The code follows `430` (`src/model/bayesian.rs:93`). Both records' guarantee tables certify their own side as verified, so the corpus contains two mechanically unfalsifiable certifications of contradictory states.
- **A function shared by two records is defined in full in both, while one of them states it is defined once.** `450-platt-calibration` and `460-subspace-restricted-blend` each write out the regime-transition function and its constant; `460` then records as a consequence that it is "defined once and imported by both", and warns that inconsistency between two copies "would produce calibration artefacts". The code vindicates the consequence and not the record bodies: the function is defined once (`src/numerics.rs:341`) with a test asserting the sharing (`src/risk/calibration.rs:973`). The decision was right and both records transcribed it anyway.
- **The per-Sentinel slot is defined in five records with three different shapes.** `120-published-state` gives it one field (`120-published-state.md:626`); `330-sentinel-report-reception`, which the register names the "single authoritative definition", gives it five (`330-sentinel-report-reception.md:233`); `360-online-standardisation` gives it four, omitting a field from its constructor (`360-online-standardisation.md:1086`); `530` and `550` restate it again. The register tracks three restatements and not the two that conflict. The code carries five fields and public visibility (`src/report/slot.rs:45`).
- **The label-processing loop and its acknowledgement diverge across the structural cluster.** `160-state-persistence-and-recovery` rewrites the steward loop with an unbiased selection and immediate exit on label disconnect (`160-…:241`–`:286`) while asserting that `140`'s semantics are "preserved" (`:200`–`:202`), discarding the biased selection and command-drain that `140` and `150` both decide. The same pair disagree on the acknowledgement's contents (`140-…:1041` against `160-…:638`) and on whether sanitisation precedes journalling. `160` also validates a restore against two checkpoint fields that its own checkpoint structure does not contain (`160-…:1033`–`:1040` against `:300`–`:424`).
- **The early-refit condition is superseded in place, three times.** `370-initialisation-and-convergence` conditions the early refit on a blend-weight check; `460-subspace-restricted-blend` replaces it with an unconditional trigger and says so three times over; `560` generalises the trigger to every registration event. The superseded check is never stated in full anywhere, so the record set holds a supersession whose object cannot be read. Note the direction: the record marked *Implemented* is superseded by prose inside records marked *Proposed*.
- **The Companion boundary is claimed by two records whose joint arrangement the code does not implement.** `470-challenge-effectiveness-model` owns the model and `590-companion-tracker-contract` the host-facing contract, but `590` decides a host-owned type replaceable through a published trait, while the shipped tracker lives inside the package (`src/risk/challenge.rs:374`), is exported from the crate root (`src/lib.rs:144`), and no such trait exists. The two records do not contradict each other in words but in the arrangement they jointly imply, and the code follows `470`.
- **Two contract records give a slot opposite initial state.** `530-report-reception-contract` has registration create the slot with the bootstrap flag set; `560-lifecycle-contract` has the constructor start it clear. Both are live public contracts describing the same call, and they differ on what the call produces.
- **A record defers a feature another record implements, guarantees, and tests.** `550-health-and-diagnostics` lists the marginalisation completion event among its deferred work — a deferral the register carries forward as live (`entry:assayer:defer-marginalise-event`) — while `560-lifecycle-contract` specifies the event's emission, asserts it in a guarantee row, and names a test for it. The deferral register and this audit both keep it deferred, on the code (`src/health/events.rs:158`); `560` is the outlier, and the reason the deferral looks contested is that one record wrote the future tense as present. Delivery after this audit resolved that historical contradiction: the lifecycle result and bounded health event now carry the event-local aggregate, and (`entry:assayer:defer-marginalise-event`) is retired.
- **A record's pending-entry expiry differs from the configuration record by a factor of twenty-four.** `520-label-contract` reasons about the not-found error on a one-hour expiry; `540-construction-contract` defaults the same parameter to twenty-four hours.
- **A record documents a defect as a guarantee.** `370` computes the before and after states of the calibration tracker both *after* the tracker has mutated, so one convergence event can never fire, while the record's event table and guarantee row both assert that it does. This is the one finding in the audit that is a live code defect rather than a documentation drift, and it is recorded here because the record is where it is visible; fixing it is not this campaign's business, and it is carried to the backlog rather than to the outline.

**Observation (Long records contradict themselves)** · `obs:assayer:adr-self-contradiction`

The contract series makes the pattern visible because its records vary so much in length. Every record in it above a thousand lines contradicts *itself*: the assessment contract decides a channel-free input and then asserts a channel field in its guarantee table; the construction contract describes a configuration hierarchy its own status section says does not exist; the lifecycle contract states that there is no skipped path and then returns that path five times in its own code; the metrics record decides against placeholders while its rationale still argues for them verbatim. The two short records in the series contradict themselves once and never.

The mechanism is the duplication finding turned inward. These records carry recurring trailing sections — guarantee verification, relationship tables, consequence lists, implementation status — that restate the body in another form, and when the body is revised the restatement is not. Nine of the series' contradictions live in a trailing table rather than in the body; deleting the six recurring trailing section types together with three embedded review blocks would remove roughly 1,800 lines and eliminate those nine outright, without touching a decision. Self-contradiction here is not carelessness but arithmetic: restate a decision four times per record and revise one copy.

**Observation (The corpus contradicts itself exactly where it duplicates)** · `obs:assayer:adr-contradiction-mechanism`

Every contradiction above is a divergence between copies of one thing. None is a disagreement about what to decide: no two records argue for opposite choices and both stand. They restate one interface, one function, or one structure, and the copies drift — which is why the code adjudicates every case cleanly, and why in six of the nine the record that is wrong either says so somewhere in its own text or is contradicted by a summary table it carries itself.

That is a stronger finding than a count of defects, because it says the defect has one cause and one remedy. The corpus does not need better reconciliation; it needs to stop making copies (`obs:assayer:adr-duplicate-definition`).

**Register (Contradictions between the records and their registers)** · `reg:assayer:adr-contradictions-register`

The hand-maintained registers — `docs/adrs.md` and the five layer documents — contradict the records and themselves at a rate that is the audit's strongest argument for generated registers (`entry:assayer:tool-generated-registers`).

- **Eight status disagreements**, registered above as (`reg:assayer:adr-status-contradictions`).
- **The metrics catalogue is given five different sizes, and no document matches the code.** `docs/adrs.md` states 55 entries in its record summary and 53 in its consistency checklist. `570-metrics-and-observability` claims 53 in two consequences and its checklist, says "~45 metrics" and "~46 metric names" elsewhere in its own text, and its own catalogue tally sums to 51. The code has exactly 55 (`src/metrics/catalog.rs:128`). So the register contradicts itself, the record contradicts itself three ways, the record contradicts the register, and all of them contradict the code. The reason none of this failed is in the test: it asserts only that the count lies between 40 and 55 (`src/tests/metrics_export.rs:219`, `src/tests/metrics_export.rs:224`), a band wide enough to admit every wrong figure but one. A number restated in six places and checked by a range is a number nobody is maintaining.
- **The layers are numbered twice, differently, eight lines apart.** The register's summary table numbers the layers 0 through 4 while the sentence immediately below it says "Layers 1–5 contain 37 ADRs", and every file name, document title, and cross-reference in the corpus uses 1 through 5. The layer-2 document compounds it, tabulating the records of Layer 3 as belonging to "Layer 2" and those of Layer 5 as "Layer 4".
- **The count of filled specification gaps differs between register and layer document.** `docs/adrs.md` says `230-published-snapshot` fills five gaps and enumerates five; the layer-2 document says six.
- **The register overstates a supersession and misses one.** It records that `510-assessment-contract` "supersedes ``L-310``'s historical sketch", which would retire a record this audit finds live in eleven places; `510` in fact supersedes only `310`'s derivation-output sketch, and depends on `310` throughout. Meanwhile `510` *does* unilaterally supersede part of `170-error-model-and-numeric-robustness` — declaring that record's field list superseded by its own — and the register does not record that at all. The register is wrong in both directions about the same record, which is what a hand-maintained supersession graph decays into.

**Warning (The derivation layer is described by no current document)** · `warn:assayer:adr-derivation-orphan`

The specification audit found that the July rewrite renamed the entire derivation layer and that the code never followed (`warn:assayer:drift-landscape-unimplemented`). The record set settles which way the corpus leans, and it leans decisively away from the specification. Sixteen of the 37 records use the shipped vocabulary; not one record anywhere uses the rewritten specification's Part IV names. The code agrees with the records (`src/assessment.rs:543`, `src/resonance/derivation.rs:164`).

The consequence for this campaign is a genuine deadlock, and it must not be resolved by the outline wave silently. The record set is to be derived from the rewritten specification (`dec:assayer:projection-principle`); for the derivation layer, the rewritten specification describes unbuilt work, while the records and the code describe the shipped system. Projecting the derivation records from the current specification would therefore produce records describing software that does not exist, and mining them from the current records would produce records contradicting the specification they are supposed to project. Nothing in the audit resolves this: it needs the ruling that (`req:assayer:spec-outline-inputs`) already demands, and until that ruling lands the derivation cluster of (`proposal:assayer:adr-projected-set`) is blocked rather than merely unscheduled.

**Observation (Duplicate definition is the corpus's structural defect)** · `obs:assayer:adr-duplicate-definition`

The contradictions above share one mechanism. The corpus's convention is that a record restates, in full, every type it touches, with a note naming the owner; the slot structure is written out in five records, the working copy is assembled across a dozen, and the register's type index exists precisely to reconcile the copies. Restatement is why the corpus is 39.3% fenced blocks, and it is why every drift found above is a drift *between copies* rather than an error in an original.

Under the calculus this convention has no purpose: a citation resolves to the owning environment, so a record that needs another record's type cites it and stops. The single largest source of shrinkage available to the rewrite is therefore not compression of prose but deletion of copies, and it costs nothing a citation does not restore. This is also why the shrinkage estimate of (`data:assayer:adr-projection-arithmetic`) can be made without deciding any open question: the copies can be counted.

## Liveness spot-checks · `sec:assayer:adr-liveness`

**Register (Spot-checks against the code)** · `reg:assayer:adr-spotchecks`

Twenty-nine checks, each read in the source. Numbering is the "Evidence" column of (`sec:assayer:adr-disposition`).

1. Report types are imported as owned values; the package declares the Sentinel dependency for report types only (`Cargo.toml:27`–`:36`).
2. Identity graphs are owned package state, and the deregistration path marginalises and drops them (`src/owner/lifecycle.rs:344`).
3. Coordinates are fixed-width throughout; no coordinate type parameter appears on the public surface (`src/types.rs`).
4. The derivation module's imports are restricted by an allowlist checked as a test (`src/tests/source_audit.rs`).
5. Published state is swapped atomically and loaded lock-free (`src/api/builder.rs:91`, `src/health/blend_stats.rs:110`).
6. The slot carries its own swapped report index and reception lock (`src/report/slot.rs:45`–`:65`).
7. The working copy has one owner and is not shared (`src/snapshot/working.rs`).
8. Commands and labels travel on separate channels (`src/owner/commands.rs:81`).
9. Threads are spawned named per instance (`src/api/builder.rs`).
10. Checkpoint and journal both exist, and the assessment path writes neither (`src/assessment.rs`).
11. The assessment call returns results without an error channel (`src/api/assess.rs`), and degradation travels inline (`src/assessment.rs:478`).
12. Report errors are structural only, four variants (`src/error.rs:117`).
13. The model is dense with fresh allocation on lifecycle change (`src/model/bayesian.rs`).
14. The anchor dimension is invariant and asserted so (`src/snapshot/working.rs:558`).
15. The matrix wrapper exists (`src/linalg/symmetric.rs:70`) and exposes no raw mutable accessor: the only three occurrences of the name in the package are the comments asserting its absence (`src/linalg/symmetric.rs:5`, `src/linalg/symmetric.rs:66`, `src/tests/linalg_symmetric.rs:660`).
16. One linear-algebra backend is declared (`Cargo.toml:31`).
17. The precision matrix is reconstructed by inversion on restore (`src/model/parameters.rs:17`, `src/model/bayesian.rs:543`).
18. Health publishes through its own swap, separate from the model snapshot (`src/health/blend_stats.rs:110`).
19. The audit found Companion state absent from both the working copy and the published snapshot (`src/snapshot/working.rs`, `src/snapshot/published.rs`), the tracker exported from the crate root (`src/lib.rs:144`), and the replacement trait, the standalone Companion health surface, and the Companion metrics mapper absent from the package. That is the audit's dated finding; this note now changes the replacement-trait and health-surface verdicts. The replacement trait is now LIVE: it is public, exported from the crate root, and implemented by the shipped tracker (`claim:risk:the-shipped-tracker-answers-the-replacement-surface-with-its-exact-posterior-counts`). The standalone Companion health surface is now LIVE: its table includes the sufficiency-verdict row (`tab:companion:health`). The challenge record decides that verdict's host-owned floor (`dec:challenge:sufficiency-floor-owned-here`).
20. The pending buffer is a concurrent map with a separate eviction queue (`src/pending/buffer.rs:82`).
21. The signal cache stores pre-encoded values (`src/signal/cache.rs`).
22. Identity maintenance runs on its own loop (`src/identity/maintenance_loop.rs:423`).
23. The assessment result is the Core output and derivation is a separate exported call (`src/lib.rs:142`).
24. The two timestamp domains are distinct types, and the decay clamp is embedded in the persistent one (`src/types.rs`, `MAX_DECAY_HOURS` at `src/lib.rs:193`).
25. One inversion utility (`src/linalg/bridge.rs:191`) serves periodic recomputation (`src/model/recompute.rs:423`), marginalisation (`src/model/bayesian.rs:93`), and revert (`src/model/bayesian.rs:543`) — the three call sites the records name.
26. The derivation entry point is a pure exported function (`src/resonance/derivation.rs:164`).
27. The regime transition is defined once and shared (`src/numerics.rs:341`), with a test asserting the sharing (`src/risk/calibration.rs:973`).
28. The conjugate model, its lazy decay, its override, and its capped injection all exist (`src/risk/challenge.rs:98`, `src/risk/challenge.rs:221`, `src/risk/challenge.rs:252`, `src/risk/challenge.rs:272`).
29. Six lifecycle methods (`src/api/lifecycle.rs:65` through `src/api/lifecycle.rs:391`) and the guidance query (`src/api/guidance.rs:25`) are present as decided.

**Observation (The challenge model ships without a caller)** · `obs:assayer:adr-challenge-unwired`

The challenge cluster needs a disposition finer than the four-way vocabulary allows. Every decision of `470-challenge-effectiveness-model` is implemented — conjugate state, lazy decay, the override that keeps accumulating underneath, the capped injection — and exported (`src/risk/challenge.rs:98` onward, `src/lib.rs:144`). Nothing calls them. The model's constructors appear only in tests and test support (`src/testing/world.rs:164`, `src/tests/label_pipeline.rs:410`, `src/tests/builder.rs:944`), at no site on the assessment, label, or lifecycle path; the record admits as much in one line, which is the most useful sentence in the cluster.

The `470` rows were therefore marked LIVE — they bound a shipped library type the rewrite must carry — while three affected `590` rows were marked DEAD. The health-surface row is LIVE today for the reasons in note 19; the other two remain DEAD. The pair is coherent; the arrangement is not. The corpus holds a finished model, a contract for a different model, and no connection between them, and the rewrite should not smooth that over: whichever way the Companion question is ruled, the resulting record owes its reader the fact that the mathematics shipped before anything consumed it.

**Observation (Visibility drift is systematic, not incidental)** · `obs:assayer:adr-visibility-drift`

Two of the twenty-nine checks turned up the same defect independently: the records declare crate visibility for a type the code exports publicly. It holds for the command enumeration and for the per-Sentinel slot, and in both cases every record agrees with every other record and all of them disagree with the source. The rewritten records should not transcribe visibility qualifiers at all. A visibility is a fact of an asset, and the repository's test-label profile already derives labels from asset facts; a record that wants to say "this type is internal" is making a claim the code can answer, and under (`goal:assayer:tools-over-discipline`) it should be made to.

## The projected record set · `sec:assayer:adr-projection`

**Caveat (This is input to the outline, not a design)** · `cav:assayer:adr-projection-nonbinding`

The grouping below is the audit's proposal and binds nothing. Under (`dec:assayer:projection-principle`) the record set is derived from the layer outlines, and those outlines do not yet exist (`obs:assayer:adr-layers-not-outlines`); the entry that derives the set is (`plan:assayer:decision-record-architecture`), and the entry that fixes the conceptual choices it derives from is (`reg:assayer:decision-record-register`). What this proposal contributes is a demonstration that the 125 live decisions *admit* a partition of roughly one third the current cardinality, together with the specific clustering the mining suggests. If the layer outlines fix different choices, the proposal changes; the dispositions do not.

Names below are descriptive, not conformant: the naming schema is decided by (`dec:assayer:record-naming-schema-text`) and no name here anticipates it.

**Proposal (The projected record set)** · `proposal:assayer:adr-projected-set`

Thirteen records, against the current 37.

| Proposed record | Scope in one line | Mined from |
| --- | --- | --- |
| The feed-forward boundary | What the package owns, what it merely observes, and the lint that keeps the difference structural. | `110`, part of `260` |
| Concurrency and publication | One writer, immutable published state, lock-free readers, and structure preempting observation across all three runtime paths. | `120`, `140`, `150` |
| Durability and recovery | What survives restart, what is deliberately rebuilt, and the once-only decay rule that makes replay sound. | `160`, part of `340` |
| Degradation posture | Why the Core surface is infallible, what degrades in band, and where the boundary between structural error and data quality falls. | `170`, error parts of `510`, `520`, `530` |
| The model substrate | The dense dynamic model, its symmetric matrix invariant, the single backend, and the exclusion of the precision matrix from published state. | `130`, `210`, `220`, part of `230` |
| Published and transient state | What is published, what is held per entity in flight, and what is cached and allowed to be lost. | `230`, `240`, `270` |
| Spatial accumulation | Outcome and identity state keyed by coordinate and depth: routing, lifecycle, and why the two key types stay distinct. | `250`, `260` |
| The runtime paths | Assessment, label, and report reception as three orderings over shared state, with their lock ordering and visibility rules. | `310`, `320`, `330`, part of `530` |
| Features and standardisation | Block order, semantic index resolution, and how statistics are acquired and updated without decaying. | `350`, `360`, part of `340` |
| Posterior maintenance | The bounded incremental update, the repair cascade that keeps it sound, and the marginalisation correction. | `410`, `420`, `430` |
| Calibration and blending | Fitting the calibration, computing the blend, and the transition function they share. | `450`, `460`, part of `370` |
| The host contract | Construction, lifecycle, and the four public surfaces as one contract: what the host must supply and what it may never see. | `510`, `520`, `530`, `540`, `560`, `580` |
| Observability | Health publication, the event channel, tiered queries, and passive metric export. | `550`, `570`, part of `370` |

Two clusters are deliberately not in the list. The **derivation** cluster — mined from `440` and the derivation parts of `310` and `510` — is blocked by (`warn:assayer:adr-derivation-orphan`) and cannot be scoped until the Part IV ruling lands. The **Companion** cluster — mined from `470` and `590` — is scopable but should not be written until someone decides whether the shipped arrangement or `590`'s contract is the intended one, since three of its decisions were then DEAD. Two remain DEAD today, so that caution still holds.

**Table (Which proposed record owns which deferral)** · `tab:assayer:adr-deferral-homes`

Under the ruling that deferrals live in the records (`dec:assayer:deferred-home`), each of the 28 audited deferrals migrates into the record owning the deferred decision. The assignment below is the audit's reading, and it is complete: every entry of the register has a home.

Only the six proposed records that would own a deferral are listed; the other seven own none.

| Proposed record | Deferrals it would own | Count |
| --- | --- | --- |
| Observability | importance-ceiling health; per-entity concordance; alarm outcome accumulators; encoding effectiveness; per-Sentinel informativeness; synchronisation-error visibility; full structural relay; cross-layer latency; buffer eviction visibility; Sentinel coverage metric; zero-Sentinel counter metric; signal-cache metrics; drift-reset counter metric | 13 |
| Spatial accumulation | host-configurable identity thresholds; identity audit metadata; published identity total importance; identity outcome decay view; competitive warm-start retention; Ledger depth histogram; Ledger cell-set deltas; Ledger immature count; Ledger materiality | 9 |
| The model substrate | marginalisation completion event; Schur correction skip counter | 2 |
| Published and transient state | pending-buffer eviction counter; positive-class prior metrics | 2 |
| Concurrency and publication | intra-label parallelism | 1 |
| The host contract | hibernation for deregistered entities | 1 |

Two facts fall out of the assignment. Twenty-two of the 28 deferrals land in just two proposed records — thirteen in **Observability**, the same concentration the deferral register already shows in its own sectioning, and nine in **Spatial accumulation** — while seven of the thirteen proposed records own none at all. A record set derived from the deferrals would therefore be badly unbalanced, which is an argument for deriving it from the conceptual choices instead. But the concentration is a real signal about where the implementation is least finished, and the Observability record should expect to be the longest of the thirteen.

**Data (The arithmetic of the projection)** · `data:assayer:adr-projection-arithmetic`

The order-of-magnitude expectation the backlog entry records (`rep:assayer:decision-mining-census`) is supported, and the evidence is countable rather than aspirational.

Thirteen proposed records against 37 is a cardinality reduction of 2.8. At the benchmark's 272 lines a thirteen-record set is 3,536 lines; at `adr/028-environment-kinds.md`'s 660 lines — the fair figure for the three or four records that must carry real tables — a mixed set lands near 5,000. Against 42,454 that is a reduction between 8.5 and 12, which brackets the order of magnitude from both sides.

Three independent measurements say the same thing. The corpus states each distinct decision 3.1 times (`data:assayer:adr-disposition-totals`), so deduplication alone removes about two thirds of the decision text before a word is compressed. The corpus is 39.3% fenced blocks, nearly all of it the restatement that citation replaces (`obs:assayer:adr-duplicate-definition`). And the corpus holds one distinct decision per 269 lines where the benchmark holds one environment per 7.2.

**Caveat (Short is not the same as sharp)** · `cav:assayer:adr-short-not-sharp`

The two shortest records tempt an inference the evidence does not support, and the distinction matters because the rewrite will be judged on length. `470-challenge-effectiveness-model`, at 406 lines, is short because its subject is small — two counters and an addition — not because it is disciplined: 55% fenced blocks, the highest proportion in its series; no alternatives section at all, presenting its model as inevitable rather than chosen; the same defaults printed twice within five lines; two summary sections doing one job. It is a thin record of a small decision.

`580-label-guidance-api`, at 232 lines, is the genuine article: nine decisions where the corpus mean is one distinct decision per 269 lines, roughly a fifth interface description against a series norm approaching half, 17 by-number references against a corpus mean of 141, none of the recurring trailing sections — so nothing that can fall out of step — and the only record in its series with no contradiction of any kind, internal or cross-record. It states its own gap plainly and scopes one open item rather than deferring vaguely.

One record in 37 already has the target shape, and it is the one that does none of what the other thirty-six do. Enough to show the shape is reachable in this corpus and not only in the benchmark; too few to call it a house style.

## Audit verdict · `sec:assayer:adr-audit-verdict`

**Summary (What this audit found)** · `summ:assayer:adr-audit-verdict`

The 37 records state 493 decisions which reduce to 158 distinct ones, of which 142 still bind the code. The decisions are in good health; the documents carrying them are not. The corpus states each decision 3.1 times and is more code than prose by line, because its convention is to restate every type it touches rather than to refer to it — and every contradiction found here, without exception, is a drift between copies rather than a disagreement about what to decide. Its self-description has decayed accordingly: eight records disagree with the master register about their own status, the metrics catalogue is given five different sizes across three documents and matches the code in none of them, the layer set is numbered two ways eight lines apart, the one cross-record conflict the register adjudicates was settled in the direction the code contradicts, and every record in the contract series above a thousand lines contradicts itself. The middle term of the projection does not exist — the layer documents summarise the records instead of fixing the choices the records should serve — so the record set has never been derived from anything. And the derivation layer is described by no current document: the records and the code agree with a specification edition the rewrite superseded.

Almost none of this is a defect of the decisions. It is what a corpus looks like when identity, reference, and status are maintained by hand, which is the campaign's premise (`goal:assayer:tools-over-discipline`) demonstrated a third time, after the specification's empty index and the deferral register's three stale entries.

**Requirement (What the outline waves must carry forward)** · `req:assayer:adr-outline-inputs`

Seven findings bind the entries downstream of this one.

1. Derive, do not partition. The 142 live decisions of (`sec:assayer:adr-disposition`) are the material; the thirteen-record grouping of (`proposal:assayer:adr-projected-set`) is a demonstration that they compress, not a design. The registers entry (`reg:assayer:decision-record-register`) fixes the conceptual choices first, and it must write them rather than convert them (`obs:assayer:adr-layers-not-outlines`).
2. Rule on the derivation layer before scoping its records (`warn:assayer:adr-derivation-orphan`). It is the one cluster this audit cannot scope, and the ruling it needs is the same one (`req:assayer:spec-outline-inputs`) already demands.
3. Carry the 12 ABSORB rows into the specification outline (`plan:assayer:specification-rewrite-contract`) as candidates, not as decisions taken. They are the bulk of the corpus's fenced blocks and the reason its line count so far exceeds its decision count.
4. Cite, never restate. The rewritten records may not reproduce a type another record or the specification owns (`obs:assayer:adr-duplicate-definition`). This is the single change that produces most of the shrinkage, and the one that would have prevented every contradiction in (`reg:assayer:adr-contradictions-records`) — every one of which is a drift between copies (`obs:assayer:adr-contradiction-mechanism`).
5. Carry no recurring trailing sections. Guarantee-verification tables, relationship tables, consequence lists, and implementation-status blocks restate the body and then fall out of step with it (`obs:assayer:adr-self-contradiction`); the linter's graph reports already answer what the relationship tables were for, mechanically.
6. Carry no status field (`obs:assayer:adr-status-vocabulary`). Where a record states a decision the code has not implemented, it says so in a caveat naming the gap, on the deferral register's pattern.
7. Give each of the 28 deferrals the home of (`tab:assayer:adr-deferral-homes`), and expect the Observability record to carry thirteen of them.
8. Register upstream prefixes before the rewrite begins: five references leave the corpus (`obs:assayer:adr-cross-corpus-refs`) and become imports, which the naming entry (`dec:assayer:record-naming-schema-text`) must provide for.

**Caveat (What this audit did not do)** · `cav:assayer:adr-audit-limits`

The decision census is a reading, not a mechanical extraction. Both figures — 493 stated and 158 distinct — count what this audit judged to meet (`conv:assayer:adr-audit-counting`); another reader would draw the line between decision and rationale differently and merge a different set of near-duplicates. The restatement ratio of 3.1 is the more robust number, because both its terms move together under a change of threshold. The grouping under conceptual choices is likewise the audit's: a reader who split the publication and serialisation choices, or merged the two spatial accumulators, would get a different 158. Dispositions attach to decisions, not to the grouping, and survive it (`req:assayer:adr-choices-derived`).

Liveness is sampled: twenty-nine checks over 158 distinct decisions, chosen where decisive rather than at random, so an unchecked LIVE row rests on the record's claim plus the absence of contrary evidence. No cost, latency, or memory figure was re-measured — a caution the corpus invites, since one record gives three different figures for one recalibration cost within its own text.

The contradiction register is not a completeness claim. Contradictions were found by following the restatement convention to its copies and by adjudicating the eight status disagreements; a conflict between records that never restate the same type, and whose status agrees, would not have surfaced. The overlap analysis is at the level of decisions and types, not prose, so two records arguing one rationale in different words are recorded as one decision with two sources.

Finally, the proposed set was not costed against the specification. It groups live decisions by affinity as the mining revealed them, and it has not been checked that each proposed record corresponds to a concept the rewritten specification actually carries — which is exactly the check (`reg:assayer:decision-record-register`) exists to perform.
