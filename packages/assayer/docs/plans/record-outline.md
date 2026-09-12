# The Assayer Record-Set Outline · `plan:assayer:decision-record-architecture`

This is the derivation of the decision-record set: the labeled outline of each projected record and the master outline of the set itself, the outline entry of the record pipeline in [the campaign backlog](backlog.md) (`plan:assayer:decision-record-architecture`). It derives the set from the five layer outlines under [docs/layers](../layers), which fix the conceptual choices the specification leaves open, per the projection ruling (`dec:assayer:projection-principle`). The record census [adr-audit.md](adr-audit.md) is mining evidence and not a design: its thirteen-cluster proposal is input (`cav:assayer:adr-projection-nonbinding`), and where the derivation departs from it the departure is recorded with its reason.

The document practices the discipline it serves. The label at each heading or environment head is that environment's mint; a parenthesized label in running text is a same-owner citation; material in double-backtick spans is displayed without participating. Every label minted here has area `assayer`. The labels of the *projected* records are displayed, never cited, because those records do not exist: nothing in this file may resolve into a document that has not been written. Legacy record numbers of the superseded naming scheme (`dec:assayer:naming-schema`) appear in code spans only.

## Method and derivation rule · `sec:assayer:record-outline-method`

**Convention (What this outline is and is not)** · `conv:assayer:record-outline-status`

This is an outline, not a set of records. It fixes each projected record's area, its identity head, the environments it will carry, and what it cites — the sentence the rewrite writes to. It fixes no prose. It also fixes nothing about ordering: the schema retires numbering entirely (`dec:assayer:record-naming-schema-text`), and the sequence of the tables below follows the layer outlines for readability, carrying no identity.

**Requirement (The derivation rule)** · `req:assayer:record-derivation-rule`

One record covers one coherent cluster of decisions that the layer choices imply, and every live decision has exactly one home. Three rules settle the hard cases, in order.

1. **A choice is not a record.** Where two layer choices answer the same question at different depths — the matrix substrate and the backend behind it — they share a record. Where one layer choice serves two questions with different owners, it is the *choice* that splits, not the decision: each half lands in the record that owns its question, and neither restates the other.
2. **A record earns each environment.** The benchmark is a record that fixes a discipline rather than enumerating a registry, and the band is roughly eight to twenty environments. A cluster below the floor is folded into its nearest neighbour unless it is a genuine cross-cutting owner; a cluster above the ceiling is split on a real seam, never on a line count.
3. **Cite, never restate.** No record reproduces a structure the specification or another record owns (`req:assayer:adr-outline-inputs`). Every environment below is a thing this record decides, qualifies, or refuses — never a copy.

**Convention (How a record's area is chosen)** · `conv:assayer:record-area-choice`

An area is a short, singular, content-derived noun, unique across the owner. The collision policy is settled: where a concept is *stated* by the specification and merely *decided about* by a record, the specification's area stands and the record takes a different word (`reg:assayer:spec-outline-open`). Every area below was checked against the forty-one areas the specification outline claims (`tab:assayer:spec-outline-areas`), the five layer areas — ``structural``, ``representation``, ``operational``, ``numerics``, ``contracts`` — and this directory's own ``assayer``. Eight of the audit's descriptive nouns collide and are renamed; the renamings are in (`tab:assayer:record-area-renames`). Two words the specification deliberately left free — ``concurrency`` and ``health``, which it avoided by taking ``publication`` and ``monitoring`` — are claimed here, which is what that avoidance was for.

**Convention (Reading the per-record tables, and why they are not tracking tables)** · `conv:assayer:record-outline-columns`

Three columns per row. *Kind* is the registry kind the head carries, drawn from the registry of environment kinds. *Label* is the full label that head mints, displayed. *Scope* is one line saying what the environment is for. The first row of every table is the record's identity decision: one ``dec`` under the record's own area, stating the decision that is the record's reason to exist (`dec:assayer:record-naming-schema-text`). Each landed record also carries a ``rec`` head naming the record itself, above that decision; the tables below do not enumerate it, because it names the document rather than an environment inside it, and a row for it would count the record twice.

These tables are deliberately **not** the tracking tables of the outline tracking tool. The recognition contract is stated at (`conv:migration:tracking-columns`); what is local is this outline's choice to stand outside it. That tool reads a table as a tracking declaration exactly when its header names a Head column and a Document column, and it would then verify every claimed head against a document that does not exist. Every row would fail, correctly and uselessly. Conversion to live tracking tables happens per record, in the commit that lands it (`reg:assayer:decision-record-register`), and the master register and the layer outlines gain their tracking tables in the same wave (`reg:assayer:decision-record-register`). Until then the header keeps the tables inert and the labels sit in double-backtick spans so that no cell mints.

## The derived set · `sec:assayer:record-set`

**Table (The derived record set)** · `tab:assayer:record-set-master`

Eighteen records against the current thirty-seven, all eighteen scoped here — seventeen at the derivation, and the eighteenth once the ruling that blocked it landed (`dec:assayer:record-derivation-scope`). "Serves" names the layer choices the record is derived from. "Live" is the count of live decisions the record homes, from the audit's disposition tables (`sec:assayer:adr-disposition`).

| Area | Scope in one line | Serves | Live | Deferrals |
| --- | --- | --- | --- | --- |
| ``ownership`` | What the package owns outright, what it merely observes, and the lint that keeps the difference structural rather than conventional. | (`dec:structural:boundary-direction`), (`dec:structural:boundary-enforcement`), (`dec:structural:coordinate-width`) | 4 | — |
| ``concurrency`` | One writer, immutable published state, lock-free readers, and structure preempting observation. | (`dec:structural:publication`), (`dec:structural:write-serialisation`) | 7 | 1 |
| ``durability`` | What survives a restart, what is deliberately rebuilt, and the once-only decay rule that makes replay sound. | (`dec:structural:durability`) | 4 | — |
| ``degradation`` | Why the Core surface cannot fail, what degrades in band instead, where the structural/data-quality line falls, and what fixes each guard's magnitude. | (`dec:structural:failure-posture`) | 3 | — |
| ``substrate`` | The dense dynamic model, the symmetry invariant of its matrices, and the single backend held behind a wrapper. | (`dec:structural:model-shape`), (`dec:representation:matrix-substrate`), (`dec:representation:backend-isolation`) | 9 | — |
| ``retention`` | What the snapshot carries, what is held per entity in flight, and what is cached and allowed to be lost. | (`dec:representation:published-composition`), (`dec:representation:independent-publication`), (`dec:representation:pending-retention`), (`dec:representation:cache-losable`) | 10 | 2 |
| ``memory`` | State keyed by coordinate and depth: routing, custody, lifecycle, and why the two key types stay distinct. | (`dec:representation:spatial-accumulation`), (`dec:representation:identity-ownership`), (`dec:representation:key-types-distinct`) | 10 | 9 |
| ``ordering`` | The three runtime paths as orderings over shared state, with their lock ordering, their visibility rules, and the guarantee they decline. | (`dec:operational:assessment-path`), (`dec:operational:core-derivation-split`), (`dec:operational:learning-path`), (`dec:operational:report-ingestion`), (`dec:operational:no-report-sequencing`) | 12 | — |
| ``clock`` | What time means to the system: two domains separated by type, one clamp, and one funnel every decay flows through. | (`dec:operational:temporal-governance`) | 3 | — |
| ``vector`` | How the feature vector is assembled and how its statistics are acquired, updated, and deliberately not decayed. | (`dec:operational:feature-composition`), (`dec:operational:standardisation`) | 10 | — |
| ``posterior`` | The bounded incremental update, the single factorisation whose refusal is a verdict, the spectral floor that keeps the matrix factorisable, and the marginalisation correction that may decline. | (`dec:numerics:incremental-update`), (`dec:numerics:repair-cascade`), (`dec:numerics:recomputation-trigger`), (`dec:numerics:marginalisation-correction`) | 14 | 2 |
| ``calibration`` | Fitting the calibration, computing the blend, the function they share, and the one trigger that fires unconditionally. | (`dec:numerics:calibration`), (`dec:numerics:blend`), (`dec:numerics:shared-transition`), (`dec:operational:convergence-diagnostic`) | 8 | — |
| ``surface`` | The four call surfaces the host uses at runtime: what each promises synchronously and what it may never see. | (`dec:contracts:assessment-surface`), (`dec:contracts:label-surface`), (`dec:contracts:report-surface`), (`dec:contracts:guidance-surface`) | 17 | — |
| ``construction`` | How an instance is built and reshaped: eager validation, the two starts, and lifecycle's two-phase visibility. | (`dec:contracts:construction-surface`), (`dec:contracts:lifecycle-surface`) | 10 | 1 |
| ``health`` | What the package may be asked about its own condition, published on its own cadence and never acting on what it finds. | (`dec:contracts:health-surface`) | 8 | 8 |
| ``metrics`` | Passive export: a neutral sample type, a compile-time catalogue, and no rendering. | (`dec:contracts:metrics-surface`) | 3 | 5 |
| ``challenge`` | The conjugate challenge estimate, its host ownership, and the fact that it shipped before anything consumed it. | (`dec:numerics:challenge-model`), (`dec:contracts:companion-boundary`) | 5 | — |
| ``derivation`` | The host-side derivation: purity, its closed input set, and what it may not reach back into. | (`dec:numerics:derivation-purity`) | 5 | — |

**Table (Areas renamed against the audit's descriptive nouns)** · `tab:assayer:record-area-renames`

The audit's names were declared descriptive and explicitly not anticipating the schema. Eight collide with a specification area and take a different word; the remaining renamings are splits or merges rather than collisions.

| Audit's noun | Area taken | Why |
| --- | --- | --- |
| the feed-forward boundary | ``ownership`` | ``boundary`` is the specification's interface-map area; the record decides custody, not the map |
| concurrency and publication | ``concurrency`` | ``publication`` is the specification's; ``concurrency`` was left free by that avoidance and is claimed here |
| the model substrate | ``substrate`` | no collision; shortened to the singular noun |
| published and transient state | ``retention`` | ``publication`` is the specification's, and the record's three halves share one question — how long state lives and what its loss costs |
| spatial accumulation | ``memory`` | ``keyspace`` and ``ledger`` are both the specification's; the shared verb of both stores is accumulation |
| the runtime paths | ``ordering`` | ``runtime`` is the specification's; the layer fixes the orderings and not the steps (`conv:operational:path-granularity`) |
| features and standardisation | ``vector`` | both ``feature`` and ``standardisation`` are the specification's; the record's subject is assembly order and statistic timing |
| the host contract | ``surface``, ``construction`` | ``host`` is the specification's, and the cluster splits — see (`dec:assayer:record-surface-split`) |
| observability | ``health``, ``metrics`` | ``monitoring`` is the specification's; ``health`` was left free by that avoidance, and the cluster splits — see (`dec:assayer:record-health-split`) |
| the Companion cluster | ``challenge`` | ``companion`` is the specification's; the record's object is the challenge estimate |
| (new) | ``clock`` | ``temporal`` is the specification's — see (`dec:assayer:record-clock-record`) |

**Data (Coverage arithmetic)** · `data:assayer:record-coverage`

The audit's five disposition tables carry 158 distinct decisions, of which 142 are LIVE (`data:assayer:adr-disposition-totals`). Every one has exactly one home and no decision has two. By record: ``ownership`` 4, ``concurrency`` 7, ``durability`` 4, ``degradation`` 3, ``substrate`` 9, ``retention`` 10, ``memory`` 10, ``ordering`` 12, ``clock`` 3, ``vector`` 10, ``posterior`` 14, ``calibration`` 8, ``surface`` 17, ``construction`` 10, ``health`` 8, ``metrics`` 3, ``challenge`` 5, ``derivation`` 5. The sum is 142.

Read by layer, the sums reconcile against the audit's tables: structural 21 live of 22 rows, representation 26 of 26, operational cycle 29 of 32, numerics 29 of 31, contracts 37 of 47 — totalling 142 live, 12 ABSORB, 1 SUPERSEDED, 3 DEAD against 158 rows. The 12 ABSORB rows are carried to the specification as candidates and are homed by no record here; the SUPERSEDED early-refit condition survives only as the history the unconditional trigger replaced (`dec:operational:convergence-diagnostic`); the 3 DEAD Companion rows are named by ``challenge`` as an arrangement nothing implements, which is not the same as homing them.

The spread is wide and deliberately so. ``surface`` at 17 and ``posterior`` at 14 are the two records that must carry real registry-shaped tables, and a record that enumerates a registry, rather than the benchmark, is the fair length comparison for them. ``degradation``, ``clock`` and ``metrics`` at three decisions each are the set's small sharp records, on the evidence that a short record can be the genuine article (`cav:assayer:adr-short-not-sharp`).

**Table (Deferral homes in the derived set)** · `tab:assayer:record-deferral-homes`

Every deferral the register held open at this derivation has a home, and the counts below are that derivation's rather than a running census. The assignment follows the layer outlines where they state one, and the audit's reading (`tab:assayer:adr-deferral-homes`) elsewhere; three assignments differ from the audit and are marked. Each deferral's citation migrates into its owning record as an ``entry`` under that record's area, and `docs/deferred.md` ends as a register of citations out of the records (`dec:assayer:deferred-home`).

| Area | Deferrals it owns | Count |
| --- | --- | --- |
| ``health`` | (`entry:assayer:defer-encoding-effectiveness`), (`entry:assayer:defer-sentinel-informativeness`), (`entry:assayer:defer-cross-layer-latency`) | 3 |
| ``memory`` | (`entry:assayer:defer-ledger-materiality`) | 1 |
| ``metrics`` | — | 0 |
| ``retention`` | — | 0 |
| ``posterior`` | — | 0 |
| ``concurrency`` | — | 0 |
| ``construction`` | (`entry:assayer:defer-lifecycle-hibernation`) | 1 |

Three departures from the audit's assignment, each on a layer outline's instruction. The synchronisation-error deferral moves from the observability cluster to ``posterior``, because layer 4 sends it to the record owning the decision it qualifies (`entry:numerics:diagnostics-open`), a posterior-maintenance decision. The buffer-eviction-rate deferral moved from the observability cluster to ``retention``, because layer 2 sent it there with the eviction counter it shared a prerequisite with; both have since completed (`entry:representation:buffer-open`). And the positive-class prior metric moved the other way, into ``metrics``, because layer 5 listed it among the deferrals hanging off the metrics choice (`entry:contracts:observability-open`). All five metric deferrals have since completed and their register entries are retired; the observability count was unchanged at thirteen across the two records that replaced it at the derivation cut.

**Decision (A record of the clock earns its place)** · `dec:assayer:record-clock-record`

The temporal choice (`dec:operational:temporal-governance`) names two expected owners — durability for the persistence half, features and standardisation for the decay inventory's model-side rates. That is the shape the census identified as the corpus's structural defect: a statement with two owners is a statement about to be copied (`obs:assayer:adr-duplicate-definition`). The choice is therefore given one owner of its own, area ``clock``.

The judgment turns on separability. The fourth of the audit's four temporal rows — that standardisation statistics carry no time-indexed decay — is genuinely a standardisation decision and goes to ``vector``, which cites the specification for the same choice made there (`dec:weighting:no-time-decay`). The other three are one indivisible group: two domains separated by type, the clamp embedded in the timestamp interface, and the funnel through shared decay functions with direct exponentiation prohibited everywhere else. None of the three is a persistence decision — the intra-process domain is never persisted at all — so ``durability`` would be made to own a rule about a type it never writes, and ``posterior`` would be made to own an arithmetic prohibition binding on sites it does not contain.

The cost is honest and is recorded rather than hidden: three decisions and nine environments put ``clock`` at the floor of the band. The alternative — folding it into ``durability`` — is listed as an open question at (`reg:assayer:record-outline-open`).

**Decision (The host contract splits in two)** · `dec:assayer:record-surface-split`

The audit's host-contract cluster gathers six layer choices and 27 live decisions, 19% of the whole set in one record. That is the shape the census found fails: every contract record above a thousand lines contradicts itself (`obs:assayer:adr-self-contradiction`), and deriving a record that must carry 27 decisions walks back into it deliberately.

The seam is real and not arithmetic. Four choices are about *traffic* — what the host calls at runtime and what each call promises on return: assessment, label, report, guidance. Two are about *shape* — how the instance is brought into existence and how it is reshaped afterwards: construction and lifecycle. They share a reader but not a question, and the second pair has its own natural gravity: every numeric default the traffic surfaces reason about is a parameter of the construction contract and is stated there once (`dec:contracts:construction-surface`), which is exactly the relation a citation serves and a merge would dissolve.

**Decision (Observability splits into health and metrics)** · `dec:assayer:record-health-split`

Layer 5 names one observability record for two distinct choices (`dec:contracts:health-surface`) and (`dec:contracts:metrics-surface`), and the audit predicted that record would be the longest of its set (`tab:assayer:adr-deferral-homes`). Undivided it would carry 11 live decisions and 13 deferrals — 26 environments, over the ceiling — and its two halves answer different questions: health decides what the package may be *asked* and on what cadence it publishes, metrics decides that the package *renders nothing*. The deferrals divide cleanly on the same seam, eight health-shaped and five metric-shaped, which is corroboration rather than convenience.

The convergence trackers' reporting, which layer 3 sends to the observability record (`dec:operational:convergence-diagnostic`), goes to ``health``: a tracker that gates nothing and only reports is a health concern, and the refit trigger that same choice fixes goes to ``calibration``, which owns what the trigger acts on.

**Decision (The derivation record's scope, and the ruling that unblocked it)** · `dec:assayer:record-derivation-scope`

This record was held back, alone in the set. Its blocker was the Part IV deadlock: the specification's rewrite renamed the derivation layer and the implementation never followed, so projecting the record from the specification would describe unbuilt work while mining it from the shipped vocabulary would contradict the specification it is meant to project (`warn:assayer:adr-derivation-orphan`). Scoping the record meant choosing the vocabulary, and choosing the vocabulary *was* the ruling, so neither layer 3 nor this outline could take it.

The ruling landed as a general one rather than a local one: the specification is authoritative, Part IV included, and where the code and the specification diverge the code is the legacy party (`dec:assayer:spec-authoritative`). That settles the vocabulary without this outline choosing it. The record projects Part IV, states the decisions the corpus made, and names the shipped divergences as gaps rather than absorbing them — which is the same treatment the specification's own Part IV gives them. The area word ``derivation`` survives the ruling on its own merits: it is the specification's noun for the function, and it collides with none of the forty-four areas that specification claims (`tab:assayer:spec-outline-areas`).

What the record homes is unchanged from what was fixed under the block: five live decisions — the derivation's purity and freedom from Core state, the per-call recomputation of channel constants, the inward-facing import allowlist, the ordered generic action set, and the exploration quantity the Core neither computes nor consumes — all from (`dec:numerics:derivation-purity`). Its Core-side counterpart was already owned: the split itself belongs to ``ordering`` (`dec:operational:core-derivation-split`). The outline is at (`entry:assayer:record-derivation`).

**Caveat (The Companion cluster is scoped, and its arrangement is not)** · `cav:assayer:record-challenge-scope`

The Companion cluster is scopable and is scoped here as ``challenge``. What is open is not its scope but its arrangement: three of its decisions are DEAD — a tracker replaceable through a published trait, a standalone Companion health surface, and a metrics mapper under its own prefix — while the specification still requires the replacement surface (`req:companion:replacement-trait`), so the gap is between the specification and the code rather than inside the corpus (`cav:contracts:companion-arrangement`). The record is therefore scoped to what is true today — host ownership, infallible estimation, the conjugate model — and owes its reader two facts it must not smooth over: that the mathematics shipped before anything consumed it (`obs:assayer:adr-challenge-unwired`), and that the arrangement the contract describes does not exist. Both are environments in its table, not omissions.

**Decision (Dispositions for the content the layer rewrite dropped)** · `dec:assayer:record-dropped-content`

Two bodies of material left the layer documents in the rewrite and want a disposition rather than silence.

**Per-decision source maps** — the mapping from each decision to the legacy records that stated it — **retire in place**. They stay in the audit (`sec:assayer:adr-disposition`), which is the campaign's evidence document and keeps them for the rewrite wave to read. They must not migrate into the records: the maps are keyed by legacy record number, which survives nowhere in migrated prose (`dec:assayer:record-naming-schema-text`), so migrating them would seed the new set with the very occurrences the burn list exists to drive to zero (`dec:assayer:burn-lists`). A record's provenance is a fact about the campaign, not a claim the record makes.

**Layer 4's epsilon inventory** — a table of the numerical guard constants, each with its value, its owning record and its site, closing with the assertion that no two are accidentally shared — **retires as a table, and its content divides**. Each constant belongs to the record that owns the decision it guards: the leverage guard, the Schur regularisation offset and the Schur conditioning threshold to ``posterior``, which also gained the spectral floor when the regularised factorisation the inventory's regularisation threshold guarded was retired; the blend guard to ``calibration``; the standardisation denominator to ``vector``; the monotonicity guard to ``derivation``. Records state the guard and cite the code rather than transcribing the figure, on the audit's own reasoning that a fact the code can answer should be made to answer it (`obs:assayer:adr-visibility-drift`) — and on the demonstrated cost of the alternative, a number restated in six places and checked by a range (`reg:assayer:adr-contradictions-register`). The distinctness claim is the one part that has no single owner and cannot be cited into existence; the recommendation is that it become a test rather than a sentence, under the standing rule (`goal:assayer:tools-over-discipline`). Both halves are carried to (`reg:assayer:record-outline-open`) because the second needs a tool that does not exist.

## The structural records · `sec:assayer:record-outlines-structural`

**Entry (Record: ownership)** · `entry:assayer:record-ownership`

Ten environments, four live decisions. The record fixes custody: what the package holds outright, what it only ever sees as a value already given to it, and the mechanical check that makes the difference a fact rather than a convention. It is the smallest structural record and the one most other records lean on, because the feed-forward direction is a premise everywhere downstream.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:ownership:feed-forward`` | reports cross the boundary as owned values; the package holds no handle to any Sentinel and calls nothing back |
| ``dec`` | ``dec:ownership:graph-custody`` | the package owns its identity graphs outright, so the boundary is external only |
| ``dec`` | ``dec:ownership:coordinate-width`` | coordinates are carried at one fixed internal width and the package is not generic over the type |
| ``dec`` | ``dec:ownership:enforcement`` | the import surface is restricted by an allowlist checked as a test, not by review |
| ``rule`` | ``rule:ownership:import-allowlist`` | the allowlist rule as an asset: what it forbids, where it runs, and what its failure reads as |
| ``cor`` | ``cor:ownership:structural-discharge`` | the feed-forward guarantee is discharged by what exists rather than by when anyone calls |
| ``rem`` | ``rem:ownership:width-lift`` | why host-declared domains of any width lift into the one internal width |
| ``disc`` | ``disc:ownership:handle-alternative`` | the rejected handle or trait object, and why it makes the boundary a convention |
| ``disc`` | ``disc:ownership:generic-alternative`` | the rejected coordinate type parameter, and what it would have cost the keyed structures |
| ``cav`` | ``cav:ownership:lint-scope`` | the guard is a build-time lint and its single test covers one path; the gap is stated, not narrated away |

Citation spine. Cites the specification for what may and may not be consumed and for the verification the check must hold, and for the coordinate lift. Cited by ``memory`` for the observation surface that makes the direction structural inward, by ``ordering`` for what crosses into derivation, by ``derivation`` for the same allowlist applied inward, and by ``challenge`` for the Companion's independence.

**Entry (Record: concurrency)** · `entry:assayer:record-concurrency`

Sixteen environments, seven live decisions, no live deferrals. The record fixes how many writers there are, how their work becomes visible, and what a reader is guaranteed. It carries the collapse of the census's sharpest structural contradiction: three records held two selection disciplines for one loop, each asserting the others preserved.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:concurrency:snapshot-swap`` | learned state reaches readers only as an immutable snapshot swapped atomically |
| ``dec`` | ``dec:concurrency:per-request-load`` | readers load per request, so a publication landing mid-batch is visible to the remainder of that batch |
| ``dec`` | ``dec:concurrency:index-independence`` | per-Sentinel report indices publish under the same discipline, independently of the model |
| ``dec`` | ``dec:concurrency:single-steward`` | one steward owns the working copy exclusively; there is no shared mutable model state |
| ``dec`` | ``dec:concurrency:channel-preemption`` | two channels, and the command channel drains before any observation is processed |
| ``dec`` | ``dec:concurrency:no-silent-drop`` | neither channel may drop silently; overflow is an error return, never a discarded item |
| ``dec`` | ``dec:concurrency:named-thread`` | the steward is a dedicated named thread, so its identity is fixed and its stalls attributable |
| ``dec`` | ``dec:concurrency:bounded-traversal`` | a traversal under a lock other threads share is cut into bounded pieces, so no walk holds the lock for its own length |
| ``dec`` | ``dec:concurrency:deferral-depth`` | the two queues carrying work off a hot path are bounded and count their overflow, and neither ever blocks its producer |
| ``cor`` | ``cor:concurrency:reader-obligations`` | what a reader is owed: never blocking on a writer, bounded in staleness rather than in freshness |
| ``reg`` | ``reg:concurrency:collapsed-loop`` | the one loop and one selection discipline, and the three legacy statements this replaces |
| ``disc`` | ``disc:concurrency:lock-alternative`` | the rejected lock over shared mutable state, and the guarantee it trades away |
| ``disc`` | ``disc:concurrency:per-batch-alternative`` | the rejected per-batch load, and the visibility it trades for a marginal saving |
| ``rem`` | ``rem:concurrency:compound-drain`` | a compound structural event drains as one unit, and what that costs the steward |
| ``cav`` | ``cav:concurrency:guidance-truncation`` | a guidance call reaching its per-Sentinel ceiling truncates rather than reacquiring, and what the skipped cells cost a candidate's score |
| ``entry`` | ``entry:concurrency:label-parallelism`` | completed: profiled independent model updates run in two fixed scoped helpers beneath the single steward |

Citation spine. Cites the specification for the publication interval, the snapshot definition, the queue requirement and the three reader guarantees, and the completed profiling record (`entry:concurrency:label-parallelism`). Cited by ``retention`` for what the swap installs, by ``memory`` for the per-dimension publication of competitive sets, by ``ordering`` for where publication falls in the label path, and by ``construction`` for the two-phase visibility that follows from serialisation.

**Entry (Record: durability)** · `entry:assayer:record-durability`

Twelve environments, four live decisions. The record fixes what survives a restart and what is deliberately rebuilt, and it owns the one ordering the census found decided both ways: sanitisation against journalling.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:durability:checkpoint-journal`` | a periodic whole-state checkpoint plus a journal of labels in flight, each entry self-contained enough to replay alone; a checkpoint whose precision matrix a factorisation refuses is refused with it rather than cold-started over |
| ``dec`` | ``dec:durability:assess-no-persistence`` | the assessment path does no persistence work at all |
| ``dec`` | ``dec:durability:decay-once`` | restore re-applies elapsed decay exactly once, and replayed labels see no further decay |
| ``dec`` | ``dec:durability:structural-compatibility`` | structural compatibility is checked on restore and a numerical parameter change is permitted across it; one counter's meaning moved under a slot that did not, and the residual is admitted rather than bumped away |
| ``dec`` | ``dec:durability:ramp-resumption`` | cold ramp progress is checkpointed and resumes where it stopped rather than restarting at the priors |
| ``dec`` | ``dec:durability:file-framing`` | both files open with a fixed header — a signature sharing a three-byte prefix, then a generation |
| ``conv`` | ``conv:durability:sanitise-before-journal`` | a submission is sanitised at the boundary before it is journalled, stated here and nowhere else |
| ``cor`` | ``cor:durability:header-width`` | neither header width is chosen: each is the sum of the fields the framing gives that file |
| ``cor`` | ``cor:durability:replay-exactness`` | what the once-only rule buys: a recorded assessment replays exactly |
| ``reg`` | ``reg:durability:collapsed-divergences`` | the acknowledgement contents and the sanitisation order, and the two legacy statements this replaces |
| ``rem`` | ``rem:durability:write-enumeration`` | why the writes the runtime performs are enumerated exhaustively rather than described |
| ``cav`` | ``cav:durability:restore-field-mismatch`` | a legacy restore validated against checkpoint fields the checkpoint did not contain; what the rewrite must check instead |

Citation spine. Cites the specification for the assess-only and replay guarantees and the enumerated writes, ``clock`` for the persistent timestamp domain the checkpoint stores, and ``concurrency`` for the steward that performs the writes. Cited by ``surface`` for the boundary consumption order the label surface makes observable, and by ``construction`` for the cold-start-or-restore rule.

**Entry (Record: degradation)** · `entry:assayer:record-degradation`

Eleven environments, three live decisions. A small sharp record: it decides one posture and states its faces at three other layers, so that no other record has to argue for infallibility from first principles.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:degradation:infallible-core`` | Core assessment does not fail; there is no error channel on the assessment call |
| ``dec`` | ``dec:degradation:retain-and-flag`` | matrix pathologies retain the best available state and flag it rather than failing the call, and the posture stops at the persistence boundary |
| ``dec`` | ``dec:degradation:error-partition`` | a structural contract violation is a hard error; a data-quality problem degrades the affected cell and is reported |
| ``dec`` | ``dec:degradation:guard-magnitudes`` | a guard's magnitude is fixed by the arithmetic of its own site, never by a figure chosen once and carried across sites |
| ``tab`` | ``tab:degradation:guard-magnitudes`` | the eight guards, by site, each with the magnitude it stands at and the argument that puts it there |
| ``cor`` | ``cor:degradation:in-band-travel`` | degradation and health travel with the result, which is what makes the absent error channel affordable |
| ``disc`` | ``disc:degradation:fallible-alternative`` | the rejected fallible assessment, and why the host has no means to act on a numeric pathology |
| ``rem`` | ``rem:degradation:numerics-face`` | a refused factorisation on the maintenance path is this posture at the numerics layer, not a second decision — and the posture stops at the persistence boundary |
| ``rem`` | ``rem:degradation:queue-face`` | observation-queue overflow is degradation on the same reasoning |
| ``cav`` | ``cav:degradation:guard-transcription`` | the magnitudes above are authored and nothing checks them against the code; the gap is stated rather than closed |
| ``cav`` | ``cav:degradation:checkpoint-placement`` | the placement of the numeric checkpoints defines the system rather than deciding about it, and goes to the specification |

Citation spine. Cites the specification for the assessment and health schemas. Cited by ``posterior`` for the error partition and for the numerics face a refused factorisation realises, by ``memory`` for queue overflow, by ``ordering`` for two-phase validation, and by ``surface`` for the assessment call's missing error channel — four citations replacing four restatements.

## The representation records · `sec:assayer:record-outlines-representation`

**Entry (Record: substrate)** · `entry:assayer:record-substrate`

Fourteen environments, nine live decisions. The record fixes what the posterior is carried in. It owns the mirror direction, which two legacy records specified oppositely while each attributed the choice to the other.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:substrate:dense-dynamic`` | the model is dense and dynamically dimensioned, reallocated fresh on a lifecycle change rather than grown in place |
| ``dec`` | ``dec:substrate:anchor-invariant`` | the anchor model is never extended and never marginalised; its dimension is invariant across every lifecycle event |
| ``dec`` | ``dec:substrate:covariance-only`` | the assessment path reads the covariance only and never the precision matrix |
| ``dec`` | ``dec:substrate:symmetry-invariant`` | symmetry is a type invariant restored after every mutating operation, not a duty callers are trusted with |
| ``dec`` | ``dec:substrate:no-raw-access`` | no raw mutable access is exposed; all mutation goes through named operations |
| ``dec`` | ``dec:substrate:trust-levels`` | two constructor trust levels separate a matrix computed by the package from one restored from storage |
| ``dec`` | ``dec:substrate:single-backend`` | a single linear-algebra backend serves the whole workspace |
| ``dec`` | ``dec:substrate:backend-isolation`` | the wrapper isolates the backend so that replacing it touches no caller |
| ``dec`` | ``dec:substrate:hand-serialisation`` | serialisation is written by hand, so the persisted form does not change when the backend does |
| ``conv`` | ``conv:substrate:mirror-direction`` | mutation writes one triangle and mirrors it into the other, in one fixed direction, stated once |
| ``reg`` | ``reg:substrate:collapsed-mirror`` | the two legacy directions, which one the code holds, and the concession never carried into a body |
| ``cor`` | ``cor:substrate:premise-standing`` | why the invariant lets the extension and marginalisation results be applied without revalidating their premise per call |
| ``rem`` | ``rem:substrate:cost-ownership`` | the isolation is what makes the tabulated operation costs a property of the algorithm rather than of the dependency |
| ``disc`` | ``disc:substrate:sparse-alternative`` | the rejected sparse representation, and why a dense feature vector leaves it no zeros to buy |

Citation spine. Cites the specification for the two Gaussian results, the precision guarantee, the operation costs, the anchor model and the model triple. Cited by ``posterior`` for every mutation it performs, by ``retention`` for the precision matrix's exclusion, and by ``construction`` for what a lifecycle reallocation costs.

**Entry (Record: retention)** · `entry:assayer:record-retention`

Eighteen environments, ten live decisions, two deferrals. One question in three registers: how long state lives, and what its loss costs. The three answers are deliberately different, and the record's work is keeping them apart.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:retention:monolithic-snapshot`` | the published model state is one snapshot built in one allocation and installed with one swap |
| ``dec`` | ``dec:retention:precision-excluded`` | the precision matrix is excluded and reconstructed on demand by the paths that need it |
| ``dec`` | ``dec:retention:health-independent`` | health state publishes through a second swap, so neither publication waits on the other |
| ``dec`` | ``dec:retention:companion-excluded`` | Companion state is excluded from the model snapshot entirely and is not published by the package at all |
| ``dec`` | ``dec:retention:pending-map`` | an assessment awaiting its label is held in a concurrent map with a separate eviction queue |
| ``dec`` | ``dec:retention:reduced-precision`` | in-flight features are retained at reduced precision, under a declared capacity |
| ``dec`` | ``dec:retention:lazy-eviction`` | eviction is lazy and capped per insert rather than swept, so no caller pays for a sweep |
| ``dec`` | ``dec:retention:journal-subset`` | the journal-serialisable subset omits what replay does not consume |
| ``dec`` | ``dec:retention:cache-preencoded`` | entity-persistent signals are cached pre-encoded rather than re-encoded per read |
| ``dec`` | ``dec:retention:cache-losable`` | the cache is not persisted, is not checkpointed, and repopulates through ordinary traffic |
| ``rem`` | ``rem:retention:eviction-bound-figure`` | what the cap of sixteen buys: the length of one critical section, which is a different question from whether there is a cap |
| ``rem`` | ``rem:retention:loss-asymmetry`` | why the buffer and the cache are separate choices: losing a pending entry loses a submitted label, losing a cache entry loses only work |
| ``rem`` | ``rem:retention:exclusion-asymmetry`` | why the two exclusions from the snapshot are separate: one publishes on its own cadence, the other the package does not own |
| ``disc`` | ``disc:retention:split-swap-alternative`` | the rejected snapshot of independently swapped parts, and the mixed state it admits |
| ``disc`` | ``disc:retention:unbounded-alternative`` | the rejected unbounded retention, and the guarantee the host could not use |
| ``cav`` | ``cav:retention:evicted-label`` | a label arriving after its entry is evicted is lost, stated rather than promised away |
| ``entry`` | ``entry:retention:eviction-counter`` | deferred: an eviction counter on the assessment snapshot |
| ``entry`` | ``entry:retention:eviction-visibility`` | deferred: eviction rate and oldest-entry age, sharing the counter's prerequisite |

Citation spine. Cites the specification for the snapshot definition, the pending entry and its storage precision, the buffer capacity and concurrency requirements, the dual-tracking remark and the two limitation caveats; cites ``substrate`` for what the snapshot carries and ``concurrency`` for the swap discipline it publishes under. Cited by ``ordering`` for what publishes at the end of the label path, by ``surface`` for the entry the label call consumes, and by ``health`` for the independent health swap.

**Entry (Record: memory)** · `entry:assayer:record-memory`

Twenty-two environments, ten live decisions, nine deferrals — the largest deferral load in the set, which is a measurement of where the implementation is least finished rather than a defect of the record. The record fixes how state keyed by position is stored, reached, and mutated, for both spatial stores.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:memory:coordinate-depth-key`` | state accumulated over the coordinate space is keyed by a coordinate-and-depth pair |
| ``dec`` | ``dec:memory:depth-walk`` | reads walk depth and exit at the first hit; writes visit every containing layer |
| ``dec`` | ``dec:memory:root-permanence`` | the root entry is permanent, and deleting an entry neither merges its evidence upward nor passes it down |
| ``dec`` | ``dec:memory:periodic-sweep`` | collection is a bounded periodic sweep rather than lazy work on the access path |
| ``dec`` | ``dec:memory:lazy-axis-rows`` | per-axis rows materialise on first reference, so an axis nobody reports on costs nothing |
| ``dec`` | ``dec:memory:graph-owner`` | a dedicated owner holds every identity graph and the assessment path never mutates one |
| ``dec`` | ``dec:memory:observation-surface`` | the observation surface accepts a coordinate and nothing else |
| ``dec`` | ``dec:memory:competitive-publication`` | competitive sets are published per dimension under the swap discipline |
| ``dec`` | ``dec:memory:overflow-degrades`` | observation-queue overflow is degradation rather than a hard error |
| ``dec`` | ``dec:memory:maintenance-cadences`` | the graph owner's two cadences answer to different things, and a command wakes it rather than the timeout |
| ``dec`` | ``dec:memory:key-types-distinct`` | the two spatial key types stay distinct despite identical structure, because a missing entry means different things |
| ``rem`` | ``rem:memory:read-write-tradeoff`` | a read is bounded by depth while a write pays for the hierarchy that makes the read cheap |
| ``rem`` | ``rem:memory:decay-cadence-composes`` | why the decay cadence is not a decay timescale: exact in the arithmetic, inexact in the schedule |
| ``entry`` | ``entry:memory:identity-thresholds`` | completed: host-configurable identity graph thresholds |
| ``entry`` | ``entry:memory:audit-metadata`` | completed: identity dimension audit metadata |
| ``entry`` | ``entry:memory:total-importance`` | completed: published identity total importance |
| ``entry`` | ``entry:memory:decay-view`` | completed: an identity outcome read-time decay view |
| ``entry`` | ``entry:memory:warm-start`` | completed: competitive-cell outcome warm-start retention |
| ``entry`` | ``entry:memory:depth-histogram`` | completed: the depth-tier distribution |
| ``entry`` | ``entry:memory:cell-deltas`` | completed: cell-set deltas in health |
| ``entry`` | ``entry:memory:immature-count`` | completed: the immature-cell count |
| ``entry`` | ``entry:memory:materiality`` | completed: materiality, value realised and attenuation limited |

Citation spine. Cites the specification for the all-layers update, the root semantics, the ledger index, the ledger concurrency invariant, the observation protocol, the encoding contract, and the table distinguishing the two stores; cites ``ownership`` for the direction the coordinate-only surface enforces, ``degradation`` for the overflow posture, and ``concurrency`` for the swap. Cited by ``ordering`` for maintenance completing before an index becomes visible, and by ``health`` for what the deferred diagnostics would surface.

## The operational records · `sec:assayer:record-outlines-operational`

**Entry (Record: ordering)** · `entry:assayer:record-ordering`

Nineteen environments, twelve live decisions. The record fixes the three orderings and nothing about their steps: where a path is enumerated, the enumeration is the specification's and is cited.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:ordering:assessment-function`` | assessment is one per-request function with factored helpers, not a pipeline of scheduled stages |
| ``dec`` | ``dec:ordering:batch-timestamp`` | a batch is processed sequentially sharing one timestamp, so two requests cannot disagree about the present |
| ``dec`` | ``dec:ordering:evidence-authority`` | an assessment moves observational geometry and no outcome-learned state; the separation is which authority a write answers to |
| ``dec`` | ``dec:ordering:score-before-evolve`` | an assessment scores against the snapshot it acquired before it offered its vector to the ramp |
| ``dec`` | ``dec:ordering:core-boundary`` | the Core's work ends at the risk assessment; turning it into a recommendation is a separate host step |
| ``dec`` | ``dec:ordering:closed-crossing`` | the only Core information crossing into derivation is the risk sufficient statistic |
| ``dec`` | ``dec:ordering:label-function`` | the label path is one function executed sequentially in ordered groups |
| ``dec`` | ``dec:ordering:decay-at-head`` | time decay is applied once at the head of the label path, before any model update |
| ``dec`` | ``dec:ordering:publish-at-end`` | model snapshot and health summary both publish at the end of the path, never partway |
| ``dec`` | ``dec:ordering:synchronous-reception`` | reception is synchronous and serialised per Sentinel, so reports from different Sentinels proceed independently |
| ``dec`` | ``dec:ordering:two-phase-validation`` | validation is two-phase: a structural failure rejects, a data-quality problem degrades the cell |
| ``dec`` | ``dec:ordering:maintenance-first`` | accumulator maintenance completes before the new index becomes visible |
| ``dec`` | ``dec:ordering:lock-order`` | lock ordering is fixed and total: reception before the accumulator, never the reverse |
| ``dec`` | ``dec:ordering:no-sequencing`` | reports carry no sequence; the last write wins and a late report simply loses |
| ``conv`` | ``conv:ordering:granularity`` | this record fixes the shape of an ordering, never the enumeration of its steps |
| ``disc`` | ``disc:ordering:staged-alternative`` | the rejected staged pipeline, and the visibility surface it adds at every boundary |
| ``disc`` | ``disc:ordering:lazy-decay-alternative`` | the rejected lazy per-reader decay, and why a value that depends on when it is read is worse than a cost |
| ``disc`` | ``disc:ordering:single-call-alternative`` | the rejected single call returning a recommendation, and the host economics it would import |
| ``cav`` | ``cav:ordering:derivation-half`` | this record fixes only the Core's side of the split; the far side is ``derivation`` |

Citation spine. Cites the specification for the three path enumerations, the two interfaces, the label cost bound, the derivation signature and basis, the staleness measure and the inter-report caveat; cites ``degradation`` for the validation posture, ``clock`` for the decay it applies, ``retention`` for what publishes, and ``memory`` for the maintenance it waits on. Cited by ``surface`` for what each call promises, and by ``vector`` for the two assembly times.

**Entry (Record: clock)** · `entry:assayer:record-clock`

Nine environments, three live decisions — the set's floor, on the reasoning at (`dec:assayer:record-clock-record`). The record makes time a type discipline rather than a convention, and exists so that three other records can cite it instead of restating it.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:clock:two-domains`` | two timestamp domains, one intra-process and one persistent, separated by type so neither can be used where the other is required |
| ``dec`` | ``dec:clock:embedded-clamp`` | the clamp on elapsed time is embedded in the timestamp interface, so a caller cannot compute an unclamped interval |
| ``dec`` | ``dec:clock:shared-functions`` | all decay flows through a small set of shared functions |
| ``rule`` | ``rule:clock:no-direct-exponentiation`` | direct exponentiation is prohibited at every site outside the shared functions |
| ``conv`` | ``conv:clock:lazy-application`` | decay is applied lazily at the point of use, over the mechanisms the specification distinguishes |
| ``disc`` | ``disc:clock:single-type-alternative`` | the rejected single timestamp type with a convention about which values persist |
| ``cor`` | ``cor:clock:clamp-unbypassable`` | why the prohibition is what keeps the clamp from being bypassed by arithmetic |
| ``rem`` | ``rem:clock:persistence-inheritance`` | what durability inherits: which domain a checkpoint stores and what a restore recomputes |
| ``rem`` | ``rem:clock:standardisation-exception`` | why the statistics carry no time-indexed decay at all, and where that choice is owned |

Citation spine. Cites the specification for lazy application, the two mechanisms, the decay inventory, and the no-time-decay choice made for the class-rate trackers. Cited by ``durability`` for the persistent domain and the once-only restore, by ``posterior`` for the combined decay factor, by ``vector`` for the exception, and by ``ordering`` for where decay is applied.

**Entry (Record: vector)** · `entry:assayer:record-vector`

Fourteen environments, ten live decisions. The record fixes how the feature vector is assembled and how the statistics it is standardised against are acquired — two halves that must be one record because the order between them is the decision.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:vector:block-order`` | the feature vector has one canonical block order, fixed and total |
| ``dec`` | ``dec:vector:sole-resolver`` | a single dimension map is the sole resolver of every feature index; no site computes an offset independently |
| ``dec`` | ``dec:vector:semantic-templates`` | interaction templates name their operands semantically and are compiled at lifecycle rebuild |
| ``dec`` | ``dec:vector:unstandardised-bases`` | interactions are computed from unstandardised bases and standardised once afterwards |
| ``dec`` | ``dec:vector:label-time-assembly`` | label-time assembly freezes the blocks fixed at assessment and re-derives those that were not |
| ``dec`` | ``dec:vector:statistic-timing`` | statistics are observed at assessment time and updated at label time |
| ``dec`` | ``dec:vector:no-time-decay`` | standardisation statistics carry no time-indexed decay at all |
| ``dec`` | ``dec:vector:two-acquisitions`` | two acquisition mechanisms and no third: a batch initialisation and a per-Sentinel bootstrap |
| ``dec`` | ``dec:vector:transient-accumulators`` | accumulators are transient and deliberately not checkpointed |
| ``dec`` | ``dec:vector:prior-mass-ramp`` | cold standardisation is a finite prior-mass ramp: each accepted vector retires an equal share of the class priors' mass |
| ``dec`` | ``dec:vector:derived-class-priors`` | feature classes carry per-class priors derived from the dimension map rather than declared |
| ``rem`` | ``rem:vector:map-obligations`` | what the dimension map must hold — covering, contiguous per block, versioned — and why a stale consumer is detected rather than misled |
| ``cav`` | ``cav:vector:late-standardisation`` | a Sentinel added late standardises against a moving target; the bootstrap bounds the cost rather than removing it |
| ``cav`` | ``cav:vector:width-accounting`` | the block-by-block width accounting defines the vector rather than deciding about it, and goes to the specification |

Citation spine. Cites the specification for the vector structure, the three dimension invariants, the compilation pipeline, the timing requirement, the bootstrap algorithm, the class-assignment requirement and the late-standardisation caveat; cites ``clock`` for the decay rule it is the exception to. Cited by ``ordering`` for the two assembly times and by ``posterior`` for the vector it updates against.

## The numerics records · `sec:assayer:record-outlines-numerics`

**Entry (Record: posterior)** · `entry:assayer:record-posterior`

Twenty-two environments, fourteen live decisions, no live deferrals. The set's densest record and one of two for which the registry record rather than the benchmark is the fair length comparison. It owns the shared inversion utility, whose interface the census found specified incompatibly by the two records that used it, each certifying its own side as verified.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:posterior:three-step-update`` | read the quantities the policy needs, decide the update the policy permits, and only then mutate |
| ``dec`` | ``dec:posterior:combined-factor`` | time decay and label decay combine into one factor computed once per model per label, never applied twice |
| ``dec`` | ``dec:posterior:identity-substitution`` | an algebraic identity replaces one matrix-vector product per model |
| ``dec`` | ``dec:posterior:leverage-before`` | leverage is bounded by policy before the update is applied, never repaired afterwards |
| ``dec`` | ``dec:posterior:repair-cascade`` | a factorisation is attempted plainly and once; its refusal is the verdict that the matrix is not positive definite in the working arithmetic, not a condition to repair |
| ``dec`` | ``dec:posterior:cascade-never-fails`` | a refusal on the maintenance path retains the maintained covariance and flags it; a refusal on any restore path fails the call, and the caller decides what to do about it |
| ``dec`` | ``dec:posterior:one-inversion-utility`` | one shared inversion utility serves every call site, taking no numerical configuration at all; what a call site decides is the disposition of a refusal, outside the utility |
| ``dec`` | ``dec:posterior:recomputation-trigger`` | recomputation is decided per model and fires on a counter whose interval conditioning sets, on growth away from the last rebuild's own record, or at once on a diagonal entry that is not finite or not positive |
| ``dec`` | ``dec:posterior:adaptive-cadence`` | a poor outcome halves the interval and a run of clean ones restores it: down fast, up slowly — and what makes an outcome poor is the drift residual left when the prior's own contribution is taken out |
| ``dec`` | ``dec:posterior:spectral-floor`` | the least eigenvalue is held at the least value the working arithmetic requires, derived from the measured largest and applied at the recomputation rather than at the label |
| ``dec`` | ``dec:posterior:measured-adoption`` | a rebuilt covariance is adopted on the drift measured after the rebuild, never on the factorisation's own verdict |
| ``dec`` | ``dec:posterior:half-solve`` | the marginalisation correction is computed through a half-solve Gram matrix rather than a full solve |
| ``dec`` | ``dec:posterior:infallible-correction`` | the correction is infallible, falling back to the uncorrected block, whose definiteness the parent's own floor supplies rather than leaves assumed |
| ``dec`` | ``dec:posterior:conditioning-guard`` | a conditioning guard skips the correction rather than attempting a computation it expects to fail — the argument that has since retired the shifted factorisation, made first here |
| ``dec`` | ``dec:posterior:debug-oracle`` | the full solve is retained as a debug-only cross-check, so the cheap path has something to be wrong against |
| ``dec`` | ``dec:posterior:always-refactor`` | the covariance is always refactored afterwards and never taken from the extracted block |
| ``tab`` | ``tab:posterior:guards`` | the four numerical guards this record's decisions install, by site and purpose, without transcribing their values |
| ``reg`` | ``reg:posterior:collapsed-interface`` | the one utility interface and the one marginalisation signature, and the two legacy specifications this replaces |
| ``disc`` | ``disc:posterior:revert-alternative`` | the rejected mutate-then-inspect-and-revert, and the state a revert must reconstruct |
| ``cav`` | ``cav:posterior:conditioning`` | the posterior can become ill-conditioned; the plain factorisation, the relative rebuild check and the spectral floor are what the system does about it, not a claim it will not |
| ``entry`` | ``entry:posterior:skip-counter`` | discharged: the correction skip counter, which makes the guards observable when they decline, and the computed error each event reports |
| ``entry`` | ``entry:posterior:sync-visibility`` | completed: per-model and aggregate visibility of the measured reading, the part the prior put there and the residual, with the interval shortening on the residual |

Citation spine. Cites the specification for the leverage-bounded update and its bound, the per-observation exactness, the condition-adaptive recomputation, the synchronisation monitor with its measure and its threshold, the replenishment floor and the requirement that counts the dimensions resting on it, the Schur complement, the positive-definiteness requirement, the marginalisation result and the requirement that its error be reported, the hibernation algorithm whose fallback a refused block takes, the conditioning caveat and the leverage-weighting discussion; cites ``substrate`` for every mutation it performs, ``degradation`` for the error partition and the posture a refusal realises, and ``clock`` for the decay factor's provenance. Cited by ``calibration`` for the posterior it calibrates and by ``construction`` for what a lifecycle change costs.

**Entry (Record: calibration)** · `entry:assayer:record-calibration`

Thirteen environments, eight live decisions. The record owns the fit, the blend, and the transition function they share — promoted to a decision of its own because the census measured the alternative's cost: one function written out in full in two records, one of which recorded as a consequence that it was defined only once.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:calibration:per-regime-search`` | fitting is a factored per-regime search on a transformed parameter, under a minimum-sample floor |
| ``dec`` | ``dec:calibration:pure-refit`` | the refit is a pure function of the buffer and the configuration: it mutates nothing and returns a parameter |
| ``dec`` | ``dec:calibration:drift-reset`` | a large calibration shift conservatively resets every drift accumulator |
| ``dec`` | ``dec:calibration:shared-transition`` | the regime-transition function and its constant are defined in exactly one place and used by both the fit and the blend |
| ``dec`` | ``dec:calibration:indexed-form`` | the projected quadratic form is computed by index without allocating a projection |
| ``dec`` | ``dec:calibration:raw-weight-forms`` | the blend weight uses raw forms, because the time correction cancels in the ratio |
| ``dec`` | ``dec:calibration:clamped-forms`` | every quadratic form is clamped non-negative, so rounding degrades into zero rather than into a negative variance |
| ``dec`` | ``dec:calibration:host-specified-movement-bound`` | no movement bound on the calibration parameter ships; a host that wants an operational one specifies it |
| ``dec`` | ``dec:calibration:unconditional-refit`` | early refit is triggered unconditionally by any registration or deregistration |
| ``cor`` | ``cor:calibration:subspace-containment`` | the blend is restricted to the anchor's subspace and stays there |
| ``reg`` | ``reg:calibration:collapsed-supersession`` | the superseded conditional refit check stated in full, and the three legacy statements that replaced it without ever reading it |
| ``rem`` | ``rem:calibration:sharing-rationale`` | why two agreeing copies are not sharing, and what inconsistency between them would produce |
| ``cav`` | ``cav:calibration:pre-calibration`` | outputs before the first refit are uncalibrated, and what a reader may trust before it |

Citation spine. Cites the specification for the fitting algorithm, the objective, the minimum-sample requirement, the drift integration, the subspace blend and its two guarantees, the blend-variance proposition, the regime structure and its transition discussion, the pre-calibration caveat and the warm-up stages; cites ``posterior`` for the posterior it reads. Cited by ``health`` for the discrimination metrics computed at refit and for the tracker reporting.

**Entry (Record: derivation)** · `entry:assayer:record-derivation`

Twelve environments, five live decisions. The record fixes the far side of the Core/derivation split: what the transform is, what may cross into it, and what it may not reach back into. It is the set's last record and the only one scoped after the conformance audit opened, which is why three of its environments are about the distance between what the corpus decided and what the package ships.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:derivation:pure-transform`` | the derivation is a pure, infallible, stateless transform: no Core state, no clock, no randomness, no error channel |
| ``dec`` | ``dec:derivation:per-call-constants`` | channel constants are recomputed on every call, because one of them tracks challenge evidence that evolves |
| ``dec`` | ``dec:derivation:ordered-actions`` | the transform is generic over an ordered action set, so any declared subset derives through one path |
| ``dec`` | ``dec:derivation:allowlisted-imports`` | the derivation module is the figure the import allowlist is drawn around, and why a total guard beats a sampling one |
| ``dec`` | ``dec:derivation:exploration-layer`` | the exploration quantity is a derivation-layer quantity the Core neither computes nor consumes |
| ``rule`` | ``rule:derivation:closed-inputs`` | the standing rule over the crossing: what may be added to it, and what admitting it would cost |
| ``reg`` | ``reg:derivation:collapsed-crossing`` | the four sizes the corpus gave the crossing, and the signature that settles it by argument rather than by count |
| ``rem`` | ``rem:derivation:exploration-split`` | why the Core's learning signal and the derivation's decision signal are complements rather than duplicates |
| ``disc`` | ``disc:derivation:enumerated-alternative`` | the rejected enumeration of the four known action sets, and the declared subsets it cannot serve |
| ``cav`` | ``cav:derivation:guard-absorbed`` | the one guard the retired constant inventory sent here has since been fully homed by the specification |
| ``cav`` | ``cav:derivation:shipped-surface`` | the former four-gap projection: three repaired divergences and one conformant reserved field, leaving the live surface aligned with the decided split |
| ``cav`` | ``cav:derivation:unmeasured`` | the formerly unread cluster is measured against the live resonance modules, and the old warning remains only as the reason the audit did not trust the original gap list |

Two environments this outline first drew were cut at the rewrite, both on the third derivation rule. An allowlist asset of this record's own restates one the structural record already carries, whose four checks include this record's (`rule:ownership:import-allowlist`); and the monotonicity guard, which the epsilon disposition assigned here before the specification's rewrite gave the dominated-tag treatment a home with its figure, its three triggers and its reasoning, is now cited rather than stated — the caveat above records where it went.

Citation spine. Cites the specification for the derivation's signature and its three arguments, the purity proof and the purity guarantee, the presentation-free invariant, the landscape output and the rendering decision, the challenge posterior's two variants, the declared action set and its narrowing caveat, the reward differentials and the challenge interaction, outcome-prediction neutrality, fragility and the risk-informative criterion, and the exploration guarantee; cites ``ownership`` for the outward allowlist this one mirrors, ``ordering`` for the closed crossing it stands on the far side of, ``surface`` for the guidance opacity it is the other half of, and ``challenge`` for the estimate it consumes. Cited by nothing in the set, because every record that would cite it was written while it was blocked — which is a fact about the order the set was landed in rather than about the record's standing.

## The contract records · `sec:assayer:record-outlines-contracts`

**Entry (Record: surface)** · `entry:assayer:record-surface`

Twenty environments, seventeen live decisions — the ceiling of the band, after the split at (`dec:assayer:record-surface-split`). The record fixes the four runtime call surfaces. It carries no status field and no trailing table restating its own body, which is the mechanism behind every self-contradiction the census found in the legacy contract series.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:surface:batch-infallible`` | the assessment call takes a batch and returns a result per request with no error channel |
| ``dec`` | ``dec:surface:no-channel-input`` | channel policy is not an input; it belongs to host-side derivation |
| ``dec`` | ``dec:surface:inline-health`` | degradation and health travel inline on every result, distinct from the on-demand query |
| ``dec`` | ``dec:surface:internal-encoding`` | encoding is performed internally and the host supplies raw values |
| ``dec`` | ``dec:surface:assessment-identifier`` | an identifier is assigned per assessment and is the label's only handle back to it |
| ``dec`` | ``dec:surface:async-label`` | label submission is asynchronous and acknowledges only what is known synchronously; the one permanent state the surface carries is the stop taken when the engine cannot rebuild a working copy it has judged corrupt |
| ``dec`` | ``dec:surface:consume-at-boundary`` | the pending entry is consumed at the boundary, before journalling and irreversibly, behind the single refusal taken ahead of the removal |
| ``dec`` | ``dec:surface:sanitise-not-reject`` | anomalous values are sanitised at the boundary rather than rejected |
| ``dec`` | ``dec:surface:no-variants`` | there is no idempotency, no batch variant, and no completion variant |
| ``dec`` | ``dec:surface:distinct-action-types`` | what the host did and what a derivation proposes are distinct types |
| ``dec`` | ``dec:surface:diagnostic-ack`` | reception acknowledges with maintenance diagnostics, so a caller learns what its report cost |
| ``dec`` | ``dec:surface:slot-map`` | slots are held in a concurrent map keyed by Sentinel, distinct from the feature index map |
| ``dec`` | ``dec:surface:reception-isolation`` | reception touches no model, identity graph, cache, or calibration state |
| ``dec`` | ``dec:surface:read-only-guidance`` | guidance is a read-only, infallible, synchronous query |
| ``dec`` | ``dec:surface:three-categories`` | three independently ranked categories, with duplication across them allowed and recorded rather than removed |
| ``dec`` | ``dec:surface:bounded-scans`` | every scan is explicitly bounded, and an exhausted budget shortens the output rather than failing the call |
| ``dec`` | ``dec:surface:guidance-opacity`` | guidance exposes no derivation quantity, no model internal, and no Companion state |
| ``conv`` | ``conv:surface:no-status`` | no surface carries a status field; a decided-but-unbuilt choice is stated in a caveat naming the gap |
| ``reg`` | ``reg:surface:collapsed-contradictions`` | the channel field, the slot's shape and the not-found expiry, and where each is now owned |
| ``cav`` | ``cav:surface:no-local-figures`` | this record states no numeric default of its own; every one is a parameter of the construction contract |

Citation spine. Cites the specification for the two interfaces, the reporting contract, the assessment and health schemas, the slot definition and its offsets, the guidance signature, the three guidance categories, the no-deduplication choice and the exploration guarantee; cites ``degradation`` for the missing error channel, ``durability`` for the consumption order, ``ordering`` for reception's serialisation, ``retention`` for the entry it consumes, and ``construction`` for every figure. Cited by ``health`` for the inline-versus-queried distinction.

**Entry (Record: construction)** · `entry:assayer:record-construction`

Sixteen environments, ten live decisions, one deferral. The record fixes how an instance comes into existence and how it is reshaped afterwards, and it is the one place in the set a numeric default is stated.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:construction:eager-validation`` | a builder validates schema, templates and every parameter eagerly, so a misconfiguration fails at construction |
| ``dec`` | ``dec:construction:core-only-config`` | configuration is Core-only; derivation and Companion configuration are host-owned and the builder does not accept them |
| ``dec`` | ``dec:construction:two-starts`` | construction either cold-starts or restores, and a structural mismatch on restore cold-starts rather than failing; a checkpoint that was read and agreed, carrying a matrix the model may not hold, fails the build instead |
| ``dec`` | ``dec:construction:named-threads`` | threads are spawned at build and named per instance |
| ``dec`` | ``dec:construction:post-seeding`` | pre-seeding is a post-construction synchronous step, not a builder argument |
| ``dec`` | ``dec:construction:six-methods`` | lifecycle is six operations over eight calls, all taking a shared receiver and returning a result |
| ``dec`` | ``dec:construction:two-phase-visibility`` | infrastructure becomes visible synchronously on return, the model asynchronously at the next publication |
| ``dec`` | ``dec:construction:blocking-deregistration`` | identity deregistration blocks until the published snapshot has stopped referencing the dimension |
| ``dec`` | ``dec:construction:compound-batch`` | a compound batch continues past non-fatal failures and publishes once per order-free chain |
| ``dec`` | ``dec:construction:destructive-deregistration`` | deregistration is destructive by default; the hibernating variant is asked for by name |
| ``rem`` | ``rem:construction:registration-timeout`` | what the registration wait is waiting for, and why its bound is a shipped operating point rather than a derived figure |
| ``tab`` | ``tab:construction:parameters`` | every numeric default the surfaces reason about, stated once here and nowhere else |
| ``reg`` | ``reg:construction:collapsed-initial-state`` | the state a registration creates a slot in, and the two legacy contracts that gave the same call opposite answers |
| ``cav`` | ``cav:construction:registration-timeout-shadowed`` | the registration timeout was declared twice and read once; the configured value now reaches the wait |
| ``cav`` | ``cav:construction:schema-fixity`` | the schema is declared once and fixed thereafter; what a host cannot change afterwards |
| ``entry`` | ``entry:construction:hibernation`` | completed: the archive contract for hibernating Sentinels and axes, what it does and does not preserve, and the definiteness verdict an archived block is admitted on |

Citation spine. Cites the specification for the construction and schema requirements, the axis registration, the two operation tables, the compound-event requirement, the lifecycle-publication guarantee, the model-set serialisation rule, the hibernation algorithm and caveat, the pre-seeding algorithm and the pending-buffer configuration table; cites ``durability`` for the compatibility rule, ``concurrency`` for the serialisation the two-phase visibility follows from, and ``substrate`` for what a reallocation costs. Cited by ``surface`` for every figure it declines to state.

**Entry (Record: health)** · `entry:assayer:record-health`

Twenty-seven environments, eight live decisions, eight deferrals. The record fixes what the package may be asked about its own condition. It reports and never acts, and it is the set's clearest case of a record whose deferral load is a finding about the implementation rather than about the record.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:health:independent-publication`` | health is published independently of the model snapshot, on its own cadence |
| ``dec`` | ``dec:health:bounded-events`` | events are pushed through one bounded channel that drops with a counter rather than blocking a producer |
| ``dec`` | ``dec:health:tiered-queries`` | queries are tiered by cost, so a frequent poller is not forced to pay for a report it does not read; a reading a model has not taken is absent rather than filled, and the label path's stop is the one condition both tiers carry |
| ``dec`` | ``dec:health:cached-discrimination`` | discrimination metrics are computed once at refit and cached, not recomputed per query |
| ``dec`` | ``dec:health:non-exhaustive-report`` | the report type is non-exhaustive, so a field may be added without breaking a host |
| ``dec`` | ``dec:health:concrete-trackers`` | each process carries its own concrete convergence tracker; there is no shared trait and no global state machine |
| ``dec`` | ``dec:health:on-demand-composite`` | the composite stage is computed on demand and may regress after a lifecycle change rather than ratcheting |
| ``dec`` | ``dec:health:reports-never-gates`` | convergence is reported and never enforced: no path blocked, no output withheld, no behaviour changed |
| ``dec`` | ``dec:health:standardisation-phase-reported`` | the cold ramp's phase and accepted count are reported quantities on both surfaces rather than inferred from a variance floor |
| ``dec`` | ``dec:health:identity-stability-cuts`` | a dimension's stability is read on two smoothed coordinates calibrated to agree, never on either alone |
| ``dec`` | ``dec:health:alarm-composite-scale`` | the two scales the per-Sentinel alarm composite compresses a peak z-score and a peak accumulator onto |
| ``dec`` | ``dec:health:calibration-settled-cuts`` | calibration is called settled on a movement bound and a refit count together, and converging at a lower pair |
| ``tab`` | ``tab:health:identity-stability-cuts`` | the cuts the identity ladder is read against, each with the figure that fixes it |
| ``cav`` | ``cav:health:tracker-event-defect`` | a convergence event that cannot fire, because both states are read after the tracker has mutated; carried, not concealed |
| ``cav`` | ``cav:health:identity-maturity-arithmetic`` | one age gate stands against a deployment-dependent arrival, and the budget it reads counts eligible labels |
| ``cav`` | ``cav:health:dead-convergence-thresholds`` | the convergence-threshold surface a host could set changed nothing and is gone; the two live fields moved to the records that own them |
| ``rem`` | ``rem:health:axis-correlation-evidence`` | what fifty samples buys an axis correlation, and why the reading waits for them |
| ``rem`` | ``rem:health:inline-versus-queried`` | the health a result carries inline and the state this swap publishes are distinct things |
| ``entry`` | ``entry:health:importance-ceiling`` | completed: importance-ceiling binding and gradient balance |
| ``entry`` | ``entry:health:per-entity-concordance`` | completed: per-entity concordance |
| ``entry`` | ``entry:health:alarm-cusums`` | completed: per-Sentinel alarm outcome accumulators |
| ``entry`` | ``entry:health:encoding-effectiveness`` | retired: the legacy three-component encoding-effectiveness row, superseded rather than built |
| ``entry`` | ``entry:health:sentinel-informativeness`` | completed: per-Sentinel informativeness backing value |
| ``entry`` | ``entry:health:structural-relay`` | completed: the full Sentinel structural diagnostic relay |
| ``entry`` | ``entry:health:marginalise-event`` | completed: the marginalisation completion event and its diagnostics payload |
| ``entry`` | ``entry:health:cross-layer-latency`` | completed: end-to-end feedback latency across the three paths, its report stage a lower bound |
| ``reg`` | ``reg:health:collapsed-event-claim`` | retired: the marginalisation event now ships, resolving the legacy contradiction |

Citation spine. Cites the specification for the report-only invariant, the health snapshot schema, the two visibility guarantees, the warm-up stages and milestones and the pre-calibration remark; cites ``retention`` for the independent swap, ``calibration`` for what refit produces, and ``memory`` for the state its deferred diagnostics would reach into. Cited by ``surface`` for the inline health it does not own and by ``metrics`` for what export renders from.

**Entry (Record: metrics)** · `entry:assayer:record-metrics`

Thirteen environments, three live decisions, five deferrals. A small sharp record. Its central discipline is negative — the package renders nothing — and its central caution is that the catalogue's size is stated nowhere outside the code, because the census found that number given five different values across three documents and matching the code in none.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:metrics:passive-export`` | the package renders nothing and depends on no monitoring crate; export is passive |
| ``dec`` | ``dec:metrics:compile-time-catalogue`` | the catalogue is a compile-time constant rather than a runtime registry, so it cannot drift from what the code emits |
| ``dec`` | ``dec:metrics:neutral-sample`` | a neutral sample type is the intermediate form a host renders from |
| ``dec`` | ``dec:metrics:catalogue-membership`` | a metric belongs exactly when a mapper emits it from one of the two health surfaces, and the correspondence holds both ways |
| ``conv`` | ``conv:metrics:pin-churn`` | the catalogue's pin is derived from its own source text, so every edit to it mints a new one |
| ``conv`` | ``conv:metrics:size-unstated`` | the catalogue's size is not stated here or anywhere outside the code, and why |
| ``cor`` | ``cor:metrics:bounded-cardinality`` | cardinality is bounded by construction; what that rules out |
| ``cav`` | ``cav:metrics:encoding-absorbed`` | the encoding rules for enumerations, absent values and booleans define the form rather than deciding about it, and go to the specification |
| ``entry`` | ``entry:metrics:sentinel-coverage`` | completed: a Sentinel coverage metric |
| ``entry`` | ``entry:metrics:zero-sentinel`` | completed: a zero-Sentinel assessment counter |
| ``entry`` | ``entry:metrics:signal-cache`` | completed: signal-cache hit-rate and utilisation metrics |
| ``entry`` | ``entry:metrics:positive-prior`` | completed: positive-class prior metrics |
| ``entry`` | ``entry:metrics:drift-resets`` | completed: a drift reset counter |

Citation spine. Cites the specification for the per-channel reporting requirement and the observability summary; cites ``health`` for what it exports and ``retention`` for the cache its deferred metrics would measure. Cited by nothing in the set, which is the correct shape for a passive boundary.

**Entry (Record: challenge)** · `entry:assayer:record-challenge`

Twelve environments, five live decisions. The record carries a finished model, a contract for a different arrangement, and no connection between them, and it says so rather than presenting a coherent design that does not exist.

| Kind | Label | Scope |
| --- | --- | --- |
| ``dec`` | ``dec:challenge:conjugate-model`` | the challenge estimate is a conjugate model updated in closed form, with lazy decay toward the prior |
| ``dec`` | ``dec:challenge:override-accumulates`` | a host override does not pause accumulation underneath it, so clearing one recovers the evolved state |
| ``dec`` | ``dec:challenge:capped-injection`` | injected evidence is indistinguishable from observed evidence and is capped |
| ``dec`` | ``dec:challenge:host-ownership`` | Companion state is owned by the host and reaches no Core structure |
| ``dec`` | ``dec:challenge:infallible-estimation`` | estimation is infallible, returning a prior, an override, or a posterior |
| ``dec`` | ``dec:challenge:sufficiency-floor-owned-here`` | the effective-sample floor at which a channel's estimate is called sufficiently evidenced is host-owned and stands beside the tracker that reads it |
| ``cor`` | ``cor:challenge:structural-independence`` | absence from the working copy and from the published snapshot is what makes the independence structural rather than asserted |
| ``rem`` | ``rem:challenge:decay-by-blending`` | why decay blends toward the prior rather than scaling the counts |
| ``cav`` | ``cav:challenge:unwired`` | every decision here is implemented and exported, and nothing on any runtime path calls it |
| ``cav`` | ``cav:challenge:arrangement-open`` | the replacement trait and the metrics mapper do not exist and the specification still requires the first, while the standalone health surface does ship and the record's old denial of it is corrected |
| ``cav`` | ``cav:challenge:sufficiency-threshold-open`` | the floor stands at twenty effective samples, and one declared tier can never reach it |
| ``cav`` | ``cav:challenge:thin-evidence`` | the estimate converges only as labels arrive, and contributing labels are thin in realistic deployments |

Citation spine. Cites the specification for the update, decay, override and injection algorithms, the conjugacy guarantee, the decay-form choice, the convergence bound, the boundary invariant, the two limitation caveats and the replacement-trait requirement; cites ``retention`` for the exclusion and ``ownership`` for the direction. Cited by nothing in the set, which is the arrangement's whole problem.

## Open questions · `sec:assayer:record-outline-open`

**Register (What the derivation could not settle)** · `reg:assayer:record-outline-open`

Six questions. Four are judgments this outline made and is exposing rather than burying; two are genuinely open.

1. **The clock record.** The derivation gives the temporal choice its own record on the reasoning at (`dec:assayer:record-clock-record`), at the cost of a three-decision record. The alternative is folding all three into ``durability`` and letting ``posterior`` and ``vector`` cite it there, which costs ``durability`` a decision about a type it never persists. The judgment is recorded as made; it is the closest call in the set and the one most worth a second reading.
2. **The two splits.** ``surface``/``construction`` and ``health``/``metrics`` both depart from a layer outline's stated expectation of a single owner (`dec:assayer:record-surface-split`) and (`dec:assayer:record-health-split`). Both departures are splits on a seam the layer itself draws — the layers state six and two *distinct* choices respectively — so neither contradicts a layer; but the layers name the clusters in prose, and if those namings are read as binding rather than expectant, both merge back and the set is sixteen.
3. **SETTLED — the two area words are ratified.** The derivation's original nouns for these two records were candidly weak; the ratified replacements are ``memory`` (the spatial outcome stores — the specification's own phrase for the machinery is the persistent spatial outcome memory) and ``vector`` (the feature vector and its standardisation — the single artifact every decision in the record shapes). Both unclaimed, singular, content-derived, and drawn from the specification's own vocabulary.
4. **SETTLED — the derivation record's area stands.** ``derivation`` was chosen as a neutral word both vocabularies use, so that the placeholder would not prejudge the Part IV ruling. The ruling has landed and the area survives it on its own merits rather than by default (`dec:assayer:record-derivation-scope`).
5. **The epsilon distinctness claim needs a tool, not a sentence.** The disposition at (`dec:assayer:record-dropped-content`) sends each guard constant to its owning record and declines to transcribe values, which leaves the inventory's closing assertion — that no two constants are accidentally shared — with no owner. Made mechanical it is a test; left in prose it is exactly the kind of hand-maintained cross-document claim the campaign exists to remove. Recommended as a test; the tool does not exist and this outline does not specify it.
6. **The set has not been costed against the specification.** Like the audit's proposal before it (`cav:assayer:adr-audit-limits`), this derivation groups choices by the layer outlines and has not verified that each record corresponds to a concept the specification carries under a citable environment. The spine notes above name what each record will cite in prose; resolving those names to labels happens per record, at the rewrite, and a record that cannot find its citations is evidence against its own scope.

**Caveat (What this outline did not do)** · `cav:assayer:record-outline-limits`

It fixed no prose, wrote no record, and settled no ordering. The environment counts began as the derivation's reading of what each record must carry and have since been brought to the landed set, so each is now a census of the environments its record holds rather than a projection of them; where a record folded two environments into one or split one into two, the count follows the record and the record is the authority. The two counts move independently and are not two readings of one quantity: a decision minted after the derivation gains its record an environment without gaining it a live decision, because the live count is the audit's and the audit closed. The coverage arithmetic *is* a contract: every one of the 142 live decisions has exactly one home here, and a rewrite that leaves one homeless or homes one twice has departed from the derivation and must say so.
