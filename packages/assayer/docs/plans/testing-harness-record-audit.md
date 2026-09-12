# The testing-harness record audit · `rep:assayer:testing-harness-record-audit`

This report projects the testing-harness concept (´plan:assayer:testing-harness-concept´) onto the Assayer decision-record system. It is evidence and allocation, not a decision, register, backlog, or record: every proposed label is displayed in a double-backtick span, and every table is a reading table under the outline's column convention (´conv:assayer:record-outline-columns´).

## Method · `sec:assayer:testing-harness-record-audit-method`

The reading began with the backlog's standing rulings, the record-set outline, the record register, the deferral register, and the repository label-calculus summary. It then covered every environment in the record set, the concept and its architecture and testing-plan context, the cited specification parts, the full layer set, and the entry heads of the full intent-plan set. Each intent plan in the specification-conflict subset was then read in full.

Claims about presence, absence, counts, areas, kinds, citations, and collisions were checked twice: first by reading the environment that establishes the claim, then by a repository search over the Assayer corpus. Claims about what a record decides were made only after reading every environment head and body in that record; a filename, table position, or concept summary never stood in for that reading.

The measurement basis is the draft commit recorded below. Later corpus movement changes the census and requires the searches to be repeated before any work package mints a proposed head.

Every table in this report is a reading table. None has the tracking-table semantics stated by the record register (´conv:assayer:record-register-tracking´).

**Table (Measurement and reading basis)** · `tab:assayer:testing-harness-record-audit-basis`

| Reading | Exact result |
| --- | --- |
| Draft commit | ``95ff5a21b00a07e47cae631af26b9226ba4090fa`` |
| Record-system documents read in full | backlog, record-set outline, record register, deferral register, repository label-calculus summary |
| Decision records read in full | 18 |
| Record environments read | 298 |
| Environment kinds used by those records | ``cav``, ``conv``, ``cor``, ``dec``, ``disc``, ``entry``, ``rec``, ``reg``, ``rem``, ``rule``, ``tab`` |
| Concept environments read | 37 |
| Specification documents selected and read in full | 6 |
| Layer documents read in full | 5 |
| Intent-plan entry-head sets read | 44 |
| Specification-conflict intent plans read in full | 6 |
| Intent plans needing a production change | 30 |

**Table (Citation extraction and collision proof)** · `tab:assayer:testing-harness-record-audit-citation-proof`

| Corpus slice | Occurrences | Unique labels | Mints found | Collisions |
| --- | ---: | ---: | ---: | ---: |
| Citations extracted from the concept | 106 | 76 | 76 | 0 |
| Citations written by this audit | 277 | 134 | 134 | 0 |
| Proposed labels displayed by this audit | 125 | 44 | 0 | 0 |

The occurrence count records repeated use; the unique count is the resolution and collision basis. The concept list was extracted before its cited documents were selected, so the specification parts, layers, and intent-plan head reads follow the concept's actual citation surface rather than an assumed harness bibliography.

**Table (The record-set environment census)** · `tab:assayer:testing-harness-record-audit-record-census`

| Area | Environments |
| --- | ---: |
| ``ownership`` | 11 |
| ``concurrency`` | 17 |
| ``durability`` | 13 |
| ``degradation`` | 12 |
| ``substrate`` | 15 |
| ``retention`` | 19 |
| ``memory`` | 23 |
| ``ordering`` | 20 |
| ``clock`` | 10 |
| ``vector`` | 15 |
| ``posterior`` | 23 |
| ``calibration`` | 14 |
| ``surface`` | 21 |
| ``construction`` | 17 |
| ``health`` | 28 |
| ``metrics`` | 14 |
| ``challenge`` | 13 |
| ``derivation`` | 13 |
| **Total** | **298** |

## The concept, decomposed · `sec:assayer:testing-harness-record-audit-decomposition`

The primary census classifies each minted concept environment once. A decision chooses among alternatives; a corollary follows from a decision already made; a caveat or deferral limits a decision or leaves a bounded question open; a convention governs repeatable practice; work is an entry with a deliverable; evidence reports or organizes a reading without deciding it.

**Table (Every minted concept environment by record vocabulary)** · `tab:assayer:testing-harness-record-audit-concept-environments`

| Concept environment | Class | Reading |
| --- | --- | --- |
| (´plan:assayer:testing-harness-concept´) | Evidence | The plan's identity decides nothing. |
| (´sec:assayer:testing-harness-concept-demands´) | Evidence | A container for the demand census. |
| (´tab:assayer:testing-harness-concept-recurring-demands´) | Evidence | Counts recurring capabilities. |
| (´tab:assayer:testing-harness-concept-plan-demands´) | Evidence | Maps intent plans to capabilities. |
| (´sec:assayer:testing-harness-concept-present´) | Evidence | Reports the current harness. |
| (´sec:assayer:testing-harness-concept-ideas´) | Evidence | A container for the six choices. |
| (´sec:assayer:testing-harness-concept-one-scenario´) | Decision | Chooses one shared scenario vocabulary over parallel scenario models. |
| (´sec:assayer:testing-harness-concept-time-and-barriers´) | Decision | Chooses scenario-controlled time and a closed barrier set over test-authored waits. |
| (´sec:assayer:testing-harness-concept-tapes´) | Decision | Chooses declarative rows and shared playback over local stimulus loops. |
| (´sec:assayer:testing-harness-concept-probes´) | Decision | Chooses one gated read-only projection contract over local state readers or a wider public surface. |
| (´sec:assayer:testing-harness-concept-oracles´) | Decision | Chooses a small independent oracle tier over repeated local recomputations. |
| (´sec:assayer:testing-harness-concept-guarded-fixtures´) | Decision | Chooses fixtures that establish and report their preconditions over unchecked setup. |
| (´sec:assayer:testing-harness-concept-rejected´) | Decision | Rejects five alternative harness boundaries; the five choices are accounted separately below. |
| (´sec:assayer:testing-harness-concept-changes´) | Evidence | A container for sized work. |
| (´entry:assayer:testing-harness-concept-kept´) | Work | Preserves the named working pieces and changes no contract. |
| (´entry:assayer:testing-harness-concept-barrier-set´) | Work | Builds the closed barrier set. |
| (´entry:assayer:testing-harness-concept-time-verbs´) | Work | Adds scenario time verbs. |
| (´entry:assayer:testing-harness-concept-complete-builder´) | Work | Completes the harness's host declarations. |
| (´entry:assayer:testing-harness-concept-probe-contract´) | Work | Builds the contract and first projections. |
| (´entry:assayer:testing-harness-concept-tape´) | Work | Builds shared playback. |
| (´entry:assayer:testing-harness-concept-oracles´) | Work | Builds the small oracle tier. |
| (´entry:assayer:testing-harness-concept-guarded-fixtures´) | Work | Makes checked preconditions the fixture default. |
| (´entry:assayer:testing-harness-concept-persistence-fork´) | Work | Builds isolated checkpoint-and-restore comparison. |
| (´entry:assayer:testing-harness-concept-merge-scenario-models´) | Work | Migrates the parallel crate model into the shared scenario. |
| (´entry:assayer:testing-harness-concept-retire-polling´) | Work | Removes sleeps, polls, and locally constructed waits. |
| (´entry:assayer:testing-harness-concept-retire-callerless´) | Work | Removes callerless helpers and stale staging notes. |
| (´sec:assayer:testing-harness-concept-migration´) | Convention | Orders work by dependency and folds unification into touched test files. |
| (´sec:assayer:testing-harness-concept-risks´) | Evidence | A container for bounded caveats and open questions. |
| (´obs:assayer:testing-harness-concept-not-the-critical-path´) | Caveat or deferral | Limits the benefit claim because most plans wait elsewhere. |
| (´obs:assayer:testing-harness-concept-probe-versus-retention´) | Caveat or deferral | Exposes the unresolved witness home for state retention excludes. |
| (´obs:assayer:testing-harness-concept-tape-versus-batch´) | Corollary | Applies the already-decided non-view status of batches to playback. |
| (´obs:assayer:testing-harness-concept-progress-not-latency´) | Caveat or deferral | Prevents a liveness counter from becoming a timing promise. |
| (´obs:assayer:testing-harness-concept-destination-concentration´) | Caveat or deferral | Leaves the convergence-module split unresolved. |
| (´obs:assayer:testing-harness-concept-oracle-trust´) | Caveat or deferral | Limits what independent recomputation can prove. |
| (´obs:assayer:testing-harness-concept-merge-has-no-sponsor´) | Caveat or deferral | Makes incremental adoption a precondition for the shared-model claim. |
| (´obs:assayer:testing-harness-concept-guard-and-oracle´) | Decision | Keeps guards and oracles separate because their failure meanings differ. |
| (´sec:assayer:testing-harness-concept-acceptance´) | Evidence | Collects acceptance conditions; it does not select an alternative. |

**Table (Primary decomposition counts)** · `tab:assayer:testing-harness-record-audit-decomposition-counts`

| Class | Count |
| --- | ---: |
| Decision | 8 |
| Corollary | 1 |
| Caveat or deferral | 6 |
| Convention | 1 |
| Work | 12 |
| Evidence | 9 |
| **Total** | **37** |

The concept's bold alternative paragraphs and acceptance bullets do not mint environments, but their content still constrains the projection. The following readings prevent the primary census from hiding them.

**Table (Embedded alternatives and acceptance conditions)** · `tab:assayer:testing-harness-record-audit-embedded-content`

| Embedded item | Record-vocabulary reading | Projection consequence |
| --- | --- | --- |
| Leave fixture and wait design distributed | Alternative to one scenario contract | Rejected by proposed decision ``dec:harness:single-scenario``. |
| Substitute mocks or doubles for the engine | Alternative execution subject | Rejected by proposed decision ``dec:harness:real-engine``. |
| Widen the production surface for tests | Alternative observation boundary | Already rejected by (´dec:surface:guidance-opacity´) and constrained by (´dec:retention:precision-excluded´); the new probe decision cites rather than re-decides it. |
| Absorb the specialised multi-channel support tree | Alternative module boundary | Rejected by proposed decision ``dec:harness:specialised-side-harnesses``. |
| Add a property-testing framework | Alternative sweep mechanism | Rejected by proposed decision ``dec:harness:seeded-sweeps``. |
| One scenario reaches all three test caller surfaces | Acceptance condition | Belongs to ``dec:harness:single-scenario`` and its incremental-adoption caveat. |
| Every wait names one queue and no test constructs a wait | Acceptance condition | Belongs to ``dec:harness:no-ad-hoc-waits`` and ``cor:concurrency:harness-barriers``. |
| Time travel is a scenario verb | Acceptance condition | Belongs to ``cor:clock:harness-control``. |
| Every permitted state projection uses the probe contract | Acceptance condition | Belongs to ``dec:harness:probe-contract`` subject to ``cav:retention:probe-boundary``. |
| Every long stimulus uses shared playback | Acceptance condition | Belongs to ``dec:harness:declarative-playback``. |
| Every eligible comparison uses a declared oracle provenance | Acceptance condition | Belongs to ``dec:harness:oracle-tier``. |
| Runtime, caller liveness, and future-plan vocabulary are measured | Acceptance condition | Belongs to distinct backlog verification and retirement work, not to a decision record. |

## Already decided, contradicted, or open · `sec:assayer:testing-harness-record-audit-disposition`

The records do not contradict the concept's primary decision set. They already decide several premises and constraints, and the projection must cite those decisions instead of treating the concept as their source. The architecture report contradicts the desired shared-model state as an implementation reading, not as a decision record.

**Table (Disposition of every primary decision environment)** · `tab:assayer:testing-harness-record-audit-decision-disposition`

| Concept decision | Already decided | Contradicted | Open decision to record |
| --- | --- | --- | --- |
| (´sec:assayer:testing-harness-concept-one-scenario´) | The engine has one steward (´dec:concurrency:single-steward´), but no record governs test-scenario multiplicity. | None in the records; the four-model implementation reading is (´sec:assayer:testing-architecture-scenario´) and (´sec:assayer:testing-architecture-multi-channel´). | One shared Assayer-owning scenario, with specialised side harnesses retained. |
| (´sec:assayer:testing-harness-concept-time-and-barriers´) | Two clock domains are decided by (´dec:clock:two-domains´), and lifecycle publication is decided by (´dec:construction:two-phase-visibility´). | None. | The test contract forbids ad hoc waits; scenario control of both time domains is a clock corollary, and named queue barriers are a concurrency corollary. |
| (´sec:assayer:testing-harness-concept-tapes´) | Per-request loads and the absence of a batch view are decided by (´dec:concurrency:per-request-load´) and (´dec:ordering:batch-timestamp´). | None. | Rows plus shared playback, while batching remains only a cost boundary. |
| (´sec:assayer:testing-harness-concept-probes´) | Snapshot composition and excluded precision are decided by (´dec:retention:monolithic-snapshot´) and (´dec:retention:precision-excluded´); production opacity is decided by (´dec:surface:guidance-opacity´). | None. | A gated, owned-copy, read-only probe contract and a crate-level home for witnesses of excluded state. |
| (´sec:assayer:testing-harness-concept-oracles´) | The posterior record already keeps a debug oracle for one numerical repair (´dec:posterior:debug-oracle´), and the testing plan classifies tolerance provenance (´tab:assayer:harness-scenario-tolerances´). | None. | A small shared tier, provenance declaration, route independence, and fails-before obligation. |
| (´sec:assayer:testing-harness-concept-guarded-fixtures´) | Convergence reports and never gates runtime behavior (´dec:health:reports-never-gates´); the distinction does not decide test fixture validity. | None. | A fixture must establish and report its own non-vacuity precondition. |
| (´sec:assayer:testing-harness-concept-rejected´) | Production widening is already rejected by (´dec:surface:guidance-opacity´); the harness already has to declare construction inputs (´dec:assayer:harness-declares´). | None. | Real engine, specialised side-harness boundary, and seeded framework-free sweeps remain open; distributed design is settled with the scenario decision. |
| (´obs:assayer:testing-harness-concept-guard-and-oracle´) | No record equates these mechanisms. | None. | Keep them separate because setup failure and result disagreement require different diagnostics. |

**Observation (The concept adds citations for existing decisions, not replacement decisions)** · `obs:assayer:testing-harness-record-audit-existing-decisions`

The formalized concept cites the clock, ordering, concurrency, retention, surface, posterior, health, and construction heads above. It does not mint a duplicate decision for the separated time domains, per-request publication, batch visibility, excluded precision, public opacity, host declarations, or the existing numerical debug oracle.

## The projection · `sec:assayer:testing-harness-record-audit-projection`

The projection principle makes the records a projection of layer choices, and the derivation rule fixes a unique coherent home for every live decision (´dec:assayer:projection-principle´) (´req:assayer:record-derivation-rule´). A testing-harness contract controls tests rather than specifying Assayer behavior, so it cannot be manufactured as another layer-derived product decision. Its behavior-specific consequences belong in existing records; its cross-cutting test semantics need a dedicated record only if keeping them elsewhere would copy or mis-home them.

The recommendation is a new record with area ``harness``. Like the clock record (´dec:assayer:record-clock-record´), it earns a place by preventing an indivisible discipline from being divided among nominal homes that do not govern it. Folding the whole contract into ``construction`` would make construction own playback, oracle provenance, fixture validity, and wait vocabulary; splitting it across several records would erase the shared rule that makes those pieces a coherent harness. The record remains small enough for the derivation band and carries decisions rather than a registry.

The strongest alternative is no new record: keep test-only choices in the testing plan, add only the behavior-specific corollaries and caveats to existing records, and let the test documentation policy govern evidence. That option preserves the strict layer-derived set and correctly observes that tests do not change the specification. It loses because a plan states an argued opinion rather than a durable decision, while the shared scenario, no-wait rule, probe boundary, oracle provenance, and fixture obligation are standing alternatives that future tests must cite after the implementation plan retires.

**Table (Explicit area-collision check)** · `tab:assayer:testing-harness-record-audit-area-collision`

| Collision domain | Checked set | Result for ``harness`` |
| --- | --- | --- |
| Areas of the eighteen records | ``ownership``, ``concurrency``, ``durability``, ``degradation``, ``substrate``, ``retention``, ``memory``, ``ordering``, ``clock``, ``vector``, ``posterior``, ``calibration``, ``surface``, ``construction``, ``health``, ``metrics``, ``challenge``, ``derivation`` | No collision |
| Specification outline areas | ``architecture``, ``axis``, ``gaussian``, ``platt``, ``config``, ``constraint``, ``companion``, ``publication``, ``detection``, ``eligibility``, ``encoding``, ``landscape``, ``extraction``, ``boundary``, ``fragility``, ``guidance``, ``monitoring``, ``host``, ``keyspace``, ``weighting``, ``guarantee``, ``ledger``, ``limitation``, ``dimension``, ``risk``, ``output``, ``feature``, ``channel``, ``principle``, ``registry``, ``rendering``, ``resource``, ``runtime``, ``signal``, ``update``, ``standardisation``, ``temporal``, ``valence``, ``warmup``, ``terminology``, ``spec`` | No collision |
| Specification avoidances | ``platt`` rather than ``calibration``; ``publication`` rather than ``concurrency``; ``monitoring`` rather than ``health``; ``keyspace`` rather than ``identity`` | No collision and no reserved counterpart |
| Every area carrying a live ``dec`` mint | ``assayer``, ``calibration``, ``challenge``, ``clock``, ``companion``, ``concurrency``, ``construction``, ``contracts``, ``degradation``, ``derivation``, ``durability``, ``encoding``, ``guidance``, ``health``, ``landscape``, ``ledger``, ``memory``, ``metrics``, ``migration``, ``numerics``, ``operational``, ``ordering``, ``ownership``, ``posterior``, ``representation``, ``retention``, ``risk``, ``structural``, ``substrate``, ``surface``, ``vector``, ``weighting`` | No collision |

The noun is short, singular, content-derived, and unique under the area-choice rule (´conv:assayer:record-area-choice´). It names the subject directly and does not take a specification noun merely because the record decides about it.

### Projected environments and homes · `sec:assayer:testing-harness-record-audit-environments`

The proposed record carries an identity head, the shared test-contract decisions, and their caveats. Behavior-specific consequences extend existing records. The concept's migration order stays in the testing plan because it schedules work rather than governing Assayer or its tests after migration.

**Table (Every projected record environment and its home)** · `tab:assayer:testing-harness-record-audit-environments`

| Kind | Proposed label | Scope in one line | Record document | Spine |
| --- | --- | --- | --- | --- |
| ``rec`` | ``rec:harness:shared-testing-contract`` | Names the standing contract shared by Assayer's test caller surfaces. | ``packages/assayer/adr/harness.md`` | H0 |
| ``dec`` | ``dec:harness:single-scenario`` | One Assayer-owning scenario type carries the common instance, names, time, and barriers. | ``packages/assayer/adr/harness.md`` | H1 |
| ``dec`` | ``dec:harness:no-ad-hoc-waits`` | Tests call a declared queue barrier and never author a sleep, poll, or wait. | ``packages/assayer/adr/harness.md`` | H2 |
| ``dec`` | ``dec:harness:declarative-playback`` | Long stimuli are typed rows interpreted by one runner with declared barrier and checkpoint policy. | ``packages/assayer/adr/harness.md`` | H3 |
| ``dec`` | ``dec:harness:probe-contract`` | State readers are gated, single-load, owned-copy, read-only projections bounded by retention. | ``packages/assayer/adr/harness.md`` | H4 |
| ``dec`` | ``dec:harness:oracle-tier`` | A small shared tier recomputes published quantities by declared independent routes. | ``packages/assayer/adr/harness.md`` | H5 |
| ``dec`` | ``dec:harness:guarded-fixtures`` | A state-establishing fixture returns only after it has checked and reported its precondition. | ``packages/assayer/adr/harness.md`` | H6 |
| ``dec`` | ``dec:harness:separate-validation`` | Fixture guards and result oracles remain separate because their failures mean different things. | ``packages/assayer/adr/harness.md`` | H6 |
| ``dec`` | ``dec:harness:real-engine`` | Shared scenarios exercise a real Assayer instance rather than an engine mock or double. | ``packages/assayer/adr/harness.md`` | H7 |
| ``dec`` | ``dec:harness:specialised-side-harnesses`` | Maintenance and multi-channel specialisation remain local while shared engine logic and barriers do not. | ``packages/assayer/adr/harness.md`` | H7 |
| ``dec`` | ``dec:harness:seeded-sweeps`` | Invariant packs use declared reproducible seeds without adding a property-testing framework. | ``packages/assayer/adr/harness.md`` | H7 |
| ``cav`` | ``cav:harness:limited-unblocking`` | The contract removes duplicated fixture work but does not resolve production or specification blockers. | ``packages/assayer/adr/harness.md`` | H8 |
| ``cav`` | ``cav:harness:progress-not-latency`` | Playback progress diagnoses liveness and establishes no elapsed-time promise. | ``packages/assayer/adr/harness.md`` | H3 |
| ``cav`` | ``cav:harness:oracle-trust`` | Route independence and fails-before evidence mitigate but cannot prove an oracle correct. | ``packages/assayer/adr/harness.md`` | H5 |
| ``cav`` | ``cav:harness:convergence-concentration`` | The shared contract does not decide whether the crowded convergence test module splits. | ``packages/assayer/adr/harness.md`` | H8 |
| ``cav`` | ``cav:harness:incremental-adoption`` | The single-scenario claim becomes true only when the last parallel model and wait vocabulary retire. | ``packages/assayer/adr/harness.md`` | H1 |
| ``cor`` | ``cor:clock:harness-control`` | The scenario controls both separated domains, moves only forward, and pairs advances with relevant barriers. | ``packages/assayer/adr/clock.md`` | E1 |
| ``cor`` | ``cor:concurrency:harness-barriers`` | Every asynchronous queue exposed to tests has one named barrier with explicit coverage. | ``packages/assayer/adr/concurrency.md`` | E2 |
| ``cor`` | ``cor:construction:harness-declarations`` | The harness is a host and declares every required construction input, including its deterministic seed. | ``packages/assayer/adr/construction.md`` | E3 |
| ``cor`` | ``cor:ordering:tape-batches`` | A playback batch is a cost boundary and never a consistency or shared-view boundary. | ``packages/assayer/adr/ordering.md`` | E4 |
| ``cor`` | ``cor:durability:harness-fork`` | A persistence witness quiesces, copies, rebuilds in isolation under the same clock, and compares normalized state. | ``packages/assayer/adr/durability.md`` | E5 |
| ``cav`` | ``cav:retention:probe-boundary`` | A probe cannot expose excluded state; a direct crate-level witness owns a promise about that state. | ``packages/assayer/adr/retention.md`` | E6 |
| ``cor`` | ``cor:surface:test-support-opacity`` | Test-only state projections remain unreachable from a sealed host-facing build. | ``packages/assayer/adr/surface.md`` | E7 |

The new record is the home for shared testing alternatives only. It does not restate queue order, timestamp semantics, checkpoint format, retained state, construction parameters, or public opacity; those remain in their behavior records and are cited by the new heads.

### Citation spines · `sec:assayer:testing-harness-record-audit-citation-spines`

Each spine below names existing specification, layer, record, architecture, concept, and plan heads as applicable. A dash means the environment has no claim in that category, not that a filename substitutes for a citation.

**Table (Citation spines for the projected environments)** · `tab:assayer:testing-harness-record-audit-citation-spines`

| Spine | Specification heads | Layer heads | Record heads | Architecture heads | Concept heads | Plan heads |
| --- | --- | --- | --- | --- | --- | --- |
| H0 | (´chap:spec:assessment-interface´) (´chap:spec:label-pipeline´) (´chap:spec:concurrency´) | (´spec:structural:architectural-commitments´) (´spec:contracts:public-contracts´) | (´rec:concurrency:snapshot-stewardship´) (´rec:surface:host-facing-contracts´) | (´rep:assayer:testing-architecture´) | (´sec:assayer:testing-harness-concept-ideas´) | (´sec:assayer:testing-plan-roadmap´) |
| H1 | (´chap:spec:concurrency´) | (´dec:structural:write-serialisation´) | (´dec:concurrency:single-steward´) | (´sec:assayer:testing-architecture-scenario´) (´sec:assayer:testing-architecture-multi-channel´) | (´sec:assayer:testing-harness-concept-one-scenario´) (´obs:assayer:testing-harness-concept-merge-has-no-sponsor´) | (´entry:assayer:harness-stage-skeleton´) (´entry:assayer:harness-stage-clock-residual-race´) |
| H2 | (´req:publication:label-queue´) (´alg:publication:identity-draining´) | (´dec:structural:write-serialisation´) | (´dec:concurrency:single-steward´) (´dec:construction:two-phase-visibility´) | (´sec:assayer:testing-architecture-liveness´) | (´sec:assayer:testing-harness-concept-time-and-barriers´) | (´entry:assayer:harness-stage-clock-residual-race´) |
| H3 | (´req:runtime:assessment-interface´) (´req:runtime:label-interface´) (´alg:runtime:report-reception´) | (´dec:operational:assessment-path´) (´dec:operational:learning-path´) | (´dec:ordering:batch-timestamp´) (´dec:concurrency:per-request-load´) | (´sec:assayer:testing-architecture-scenario´) (´obs:assayer:testing-architecture-deadlines-are-liveness´) | (´sec:assayer:testing-harness-concept-tapes´) (´obs:assayer:testing-harness-concept-progress-not-latency´) | (´entry:assayer:harness-stage-tapes´) (´entry:assayer:process-grid-runtime´) |
| H4 | (´def:publication:model-snapshot´) (´inv:guarantee:structural-exactness´) | (´dec:representation:published-composition´) (´dec:representation:pending-retention´) | (´dec:retention:precision-excluded´) (´dec:surface:guidance-opacity´) | (´sec:assayer:testing-architecture-reach´) | (´sec:assayer:testing-harness-concept-probes´) (´obs:assayer:testing-harness-concept-probe-versus-retention´) | (´entry:assayer:harness-stage-tapes´) |
| H5 | (´inv:guarantee:per-observation-exactness´) (´inv:guarantee:precision´) | (´spec:numerics:model-mathematics´) | (´dec:posterior:debug-oracle´) | (´sec:assayer:testing-architecture-assertions´) | (´sec:assayer:testing-harness-concept-oracles´) (´obs:assayer:testing-harness-concept-oracle-trust´) | (´tab:assayer:harness-scenario-tolerances´) |
| H6 | (´inv:guarantee:evidence-authority´) | (´dec:operational:convergence-diagnostic´) | (´dec:health:reports-never-gates´) | (´sec:assayer:testing-architecture-assertions´) | (´sec:assayer:testing-harness-concept-guarded-fixtures´) (´obs:assayer:testing-harness-concept-guard-and-oracle´) | (´obs:assayer:reframe-identity-threshold´) |
| H7 | (´chap:spec:assessment-interface´) (´inv:guarantee:precision´) | (´dec:contracts:assessment-surface´) (´dec:numerics:repair-cascade´) | (´dec:surface:batch-infallible´) (´dec:posterior:debug-oracle´) | (´sec:assayer:testing-architecture-side-harnesses´) (´sec:assayer:testing-architecture-multi-channel´) | (´sec:assayer:testing-harness-concept-rejected´) | (´entry:assayer:harness-property-packs´) |
| H8 | (´chap:spec:concurrency´) | (´dec:structural:write-serialisation´) | (´dec:concurrency:single-steward´) | (´obs:assayer:testing-architecture-unreported-decay´) (´sec:assayer:testing-architecture-multi-channel´) | (´obs:assayer:testing-harness-concept-not-the-critical-path´) (´obs:assayer:testing-harness-concept-destination-concentration´) | (´obs:assayer:harness-integration-slices´) |
| E1 | (´chap:spec:temporal-governance´) (´def:temporal:two-mechanisms´) | (´dec:operational:temporal-governance´) | (´dec:clock:two-domains´) (´dec:clock:embedded-clamp´) | (´sec:assayer:testing-architecture-clocks´) | (´sec:assayer:testing-harness-concept-time-and-barriers´) | (´entry:assayer:harness-stage-clock´) |
| E2 | (´req:publication:label-queue´) (´alg:publication:identity-draining´) | (´dec:structural:write-serialisation´) | (´dec:concurrency:single-steward´) (´dec:concurrency:no-silent-drop´) | (´sec:assayer:testing-architecture-liveness´) | (´sec:assayer:testing-harness-concept-time-and-barriers´) | (´entry:assayer:harness-stage-clock-residual-race´) |
| E3 | (´chap:spec:configuration´) (´tab:config:concurrency´) | (´dec:contracts:construction-surface´) | (´dec:construction:eager-validation´) | (´sec:assayer:testing-architecture-vocabularies´) | (´entry:assayer:testing-harness-concept-complete-builder´) | (´dec:assayer:harness-declares´) |
| E4 | (´req:runtime:assessment-interface´) | (´dec:operational:assessment-path´) | (´dec:ordering:batch-timestamp´) (´dec:concurrency:per-request-load´) | (´sec:assayer:testing-architecture-scenario´) | (´obs:assayer:testing-harness-concept-tape-versus-batch´) | (´entry:assayer:mid-batch-identity-movement´) |
| E5 | (´inv:guarantee:replay´) (´inv:guarantee:structural-exactness´) | (´dec:structural:durability´) | (´dec:durability:checkpoint-journal´) (´dec:durability:decay-once´) | (´sec:assayer:testing-architecture-scenario´) | (´entry:assayer:testing-harness-concept-persistence-fork´) | (´entry:assayer:harness-stage-tapes´) |
| E6 | (´def:publication:model-snapshot´) | (´dec:representation:published-composition´) | (´dec:retention:precision-excluded´) | (´sec:assayer:testing-architecture-reach´) | (´obs:assayer:testing-harness-concept-probe-versus-retention´) | (´entry:assayer:harness-stage-tapes´) |
| E7 | (´schema:output:assessment´) | (´dec:contracts:guidance-surface´) | (´dec:surface:guidance-opacity´) | (´sec:assayer:testing-architecture-reach´) | (´sec:assayer:testing-harness-concept-probes´) | (´dec:assayer:test-documentation-policy´) |

The citation-spine extraction contains the occurrence and unique counts reported later. Every unique citation was searched as a mint and found exactly once before this projection was written; proposals are separately searched as nonparticipating displays and must have no live mint.

## Register, outline, and deferral impacts · `sec:assayer:testing-harness-record-audit-register-impacts`

The register gains a distinct row for every head minted by a record, including the new record identity. The displayed rows below reproduce the register's values but are a reading table: the altered column names and double-backtick values prevent this report from asserting a tracking relation.

**Table (Exact proposed record-register rows)** · `tab:assayer:testing-harness-record-audit-register-rows`

| Proposed Head | Proposed Document |
| --- | --- |
| ``rec:harness:shared-testing-contract`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:single-scenario`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:no-ad-hoc-waits`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:declarative-playback`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:probe-contract`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:oracle-tier`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:guarded-fixtures`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:separate-validation`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:real-engine`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:specialised-side-harnesses`` | ``packages/assayer/adr/harness.md`` |
| ``dec:harness:seeded-sweeps`` | ``packages/assayer/adr/harness.md`` |
| ``cav:harness:limited-unblocking`` | ``packages/assayer/adr/harness.md`` |
| ``cav:harness:progress-not-latency`` | ``packages/assayer/adr/harness.md`` |
| ``cav:harness:oracle-trust`` | ``packages/assayer/adr/harness.md`` |
| ``cav:harness:convergence-concentration`` | ``packages/assayer/adr/harness.md`` |
| ``cav:harness:incremental-adoption`` | ``packages/assayer/adr/harness.md`` |
| ``cor:clock:harness-control`` | ``packages/assayer/adr/clock.md`` |
| ``cor:concurrency:harness-barriers`` | ``packages/assayer/adr/concurrency.md`` |
| ``cor:construction:harness-declarations`` | ``packages/assayer/adr/construction.md`` |
| ``cor:ordering:tape-batches`` | ``packages/assayer/adr/ordering.md`` |
| ``cor:durability:harness-fork`` | ``packages/assayer/adr/durability.md`` |
| ``cav:retention:probe-boundary`` | ``packages/assayer/adr/retention.md`` |
| ``cor:surface:test-support-opacity`` | ``packages/assayer/adr/surface.md`` |

The register block for ``harness`` belongs after the ``clock`` block and before ``posterior`` in the register's existing order; the master-table and outline entry belong after ``clock`` and before ``vector`` in their order. Each extension row belongs at the end of its existing document block, immediately before the next record identity row. This placement lets independently prepared blocks rebase without interleaving rows.

**Table (Exact master-table and outline additions)** · `tab:assayer:testing-harness-record-audit-outline-impact`

| Surface | Proposed addition or correction |
| --- | --- |
| Master-table row | ``harness`` \| The shared testing contract: one scenario, declared barriers and playback, bounded probes, independent oracles, guarded fixtures, and explicit side-harness limits. \| (´dec:assayer:test-documentation-policy´), (´plan:assayer:testing-harness-concept´) \| 10 \| 1 |
| New ruling head | ``dec:assayer:record-harness-record`` states that a testing-only record earns its place because splitting the common contract would copy it across behavior records, while the contract changes no specification behavior. |
| New outline entry | ``entry:assayer:record-harness`` enumerates the ten decisions and five caveats above, plus the record identity required on landing. |
| Set count | The master prose changes from 18 to 19 records and distinguishes the original layer-derived set from the testing-only exception. |
| Coverage arithmetic | The existing 142 layer-projected live decisions remain unchanged; the outline reports 10 additional test-contract decisions and a combined record-set total of 152 without rewriting the audit's historical disposition totals. |
| Open-question register | The strongest no-new-record alternative is recorded as rejected by ``dec:assayer:record-harness-record`` rather than left in (´reg:assayer:record-outline-open´). |

The master row cannot honestly put the harness under a layer decision: the test documentation policy and concept are its sources, and the new ruling explains the controlled exception. Calling it layer-derived would violate the projection principle rather than preserve it.

**Table (Deferral-register impact)** · `tab:assayer:testing-harness-record-audit-deferral-impact`

| Deferred question | Owning record environment | Deferral-register head | Backlog action |
| --- | --- | --- | --- |
| Whether the convergence subject splits before the concentrated intent plans land | ``cav:harness:convergence-concentration`` | ``entry:assayer:defer-harness-convergence-layout`` | ``entry:assayer:harness-convergence-module-ruling`` seeks the decision and cites both heads. |

The deferred choice is stated in full in the harness record under the deferred-home ruling (´dec:assayer:deferred-home´). The deferral register gains only the displayed identity head and a citation to that caveat; it does not repeat the question or acceptance condition. All other concept risks are either decided by a proposed environment, bounded by a caveat without postponing a decision, or expressed as open backlog work.

## The backlog projection · `sec:assayer:testing-harness-record-audit-backlog`

The backlog admits only open work, uses the status grammar (´conv:assayer:status-grammar´), and assigns each entry an outcome from the closed vocabulary (´conv:assayer:outcome-classes´). Rows merge only when outcome, acceptance, and live pin all agree; none of the rows below meets that test with another row.

**Table (Every open concept and testing-plan item projected into the backlog)** · `tab:assayer:testing-harness-record-audit-backlog-entries`

| Title and proposed or moved head | Outcome | Status | Acceptance or revisit condition | Live pin | Relation to existing work |
| --- | --- | --- | --- | --- | --- |
| Closed queue-barrier set — ``entry:assayer:harness-closed-barriers`` | ENGINEERING | OPEN | Every asynchronous queue reachable by a test has one documented barrier; each barrier names coverage and exclusion, and focused tests stall or pass on the correct queue. | (´entry:assayer:testing-harness-concept-barrier-set´) | Splits the barrier half out of (´entry:assayer:harness-stage-tapes´). |
| Scenario time verbs — ``entry:assayer:harness-scenario-time`` | ENGINEERING | OPEN | The scenario advances and targets both clock domains without backward travel and automatically crosses every barrier the operation requires. | (´entry:assayer:testing-harness-concept-time-verbs´) | Merges the harness-facing remainder of (´entry:assayer:harness-stage-clock´); production sites remain in that moved entry. |
| Complete host declaration builder — ``entry:assayer:harness-complete-builder`` | ENGINEERING | OPEN | Every required construction input is declared through one builder, invalid declarations fail at construction, and deterministic seeds are explicit. | (´entry:assayer:testing-harness-concept-complete-builder´) | Extends the DONE skeleton without reopening (´entry:assayer:harness-stage-skeleton´). |
| Probe contract and first projections — ``entry:assayer:harness-probe-contract`` | ENGINEERING | OPEN | The contract enforces gating, one published load, owned copies, no mutation, scenario time, retention limits, and sealed-build invisibility; the first projections have focused tests. | (´entry:assayer:testing-harness-concept-probe-contract´) | Splits state access out of (´entry:assayer:harness-stage-tapes´). |
| Shared tape runner — ``entry:assayer:harness-tape-runner`` | PARKED | PARKED | Revisit only when (´entry:assayer:process-grid-runtime´) records a runtime budget and tolerance; then create an ENGINEERING entry whose acceptance covers typed rows, barrier policy, checkpoints, progress, and cost-only batches. | (´entry:assayer:testing-harness-concept-tape´) | Supersedes the playback portion of (´entry:assayer:harness-stage-tapes´) after the revisit condition fires. |
| Independent oracle tier — ``entry:assayer:harness-oracle-tier`` | ENGINEERING | OPEN | The rank fold, regularised Schur complement, decay recurrence, and dimension-width formula declare provenance, avoid production routes, have multiple callers, and each separates from a deliberately defective arm. | (´entry:assayer:testing-harness-concept-oracles´) | Splits oracle work out of (´entry:assayer:harness-stage-tapes´) and serves the intent plans that currently repeat it. |
| Guarded fixture default — ``entry:assayer:harness-guarded-fixtures`` | ENGINEERING | OPEN | Each state-establishing fixture checks its named precondition before returning and reports the measured baseline on failure; tests distinguish setup failure from result disagreement. | (´entry:assayer:testing-harness-concept-guarded-fixtures´) | Makes the testing-plan reframing at (´obs:assayer:reframe-identity-threshold´) executable. |
| Isolated persistence fork — ``entry:assayer:harness-persistence-fork`` | ENGINEERING | OPEN | A witness quiesces, checkpoints, copies into isolated storage, rebuilds under the same injected clock, runs both arms, and compares one normalized durable projection. | (´entry:assayer:testing-harness-concept-persistence-fork´) | Splits persistence fixtures out of (´entry:assayer:harness-stage-tapes´). |
| Unify the Assayer scenario model — ``entry:assayer:harness-scenario-unification`` | RETIREMENT | OPEN | Parallel crate configuration, entity, report, and wait vocabularies are absent; all three caller surfaces reach the shared scenario, while documented specialised side harnesses remain. | (´entry:assayer:testing-harness-concept-merge-scenario-models´) | Absorbs the migration rule rather than becoming a large standalone rewrite. |
| Retire polling, sleeps, and local waits — ``entry:assayer:harness-polling-retirement`` | RETIREMENT | OPEN | Search and the linter find no test-authored sleep, poll, or wait outside the harness liveness implementation; every removed site names its replacement barrier. | (´entry:assayer:testing-harness-concept-retire-polling´) | Runs with the closed-barrier entry and supersedes the stale crate-tree wait described by (´sec:assayer:testing-harness-concept-present´). |
| Retire callerless helpers and stale staging notes — ``entry:assayer:harness-stale-helper-retirement`` | RETIREMENT | OPEN | Every harness item has a caller or is deleted, stale staging names are absent, and generated module documentation reports no orphan. | (´entry:assayer:testing-harness-concept-retire-callerless´) | Closes the architecture findings at (´obs:assayer:testing-architecture-unreported-decay´) and (´obs:assayer:testing-architecture-staging-notes´). |
| Rule the convergence-module layout — ``entry:assayer:harness-convergence-module-ruling`` | RULING | OPEN | Decide retain or split before concentrated long-running witnesses land; record the runtime, maintenance, and claim-cohesion criterion and update the deferral citation. | (´obs:assayer:testing-harness-concept-destination-concentration´) | New ruling work; it cites ``entry:assayer:defer-harness-convergence-layout`` rather than copying the deferred question. |
| Rule relative growth versus absolute conditioning — ``entry:assayer:harness-conditioning-trigger-ruling`` | RULING | OPEN | Select relative growth with the fixed ratio as fixture regime, or specify an absolute policy with reset or hysteresis; amend the promise and governing heads before its witness. | (´plan:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation´) | One of six distinct specification-conflict plans; no harness entry can resolve it. |
| Rule exact extension versus registration lift — ``entry:assayer:harness-registration-lift-ruling`` | RULING | OPEN | Select shared-block widening, a later-label counterfactual, or registration plus first report; align specification, record, and intent before fixture work. | (´plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it´) | One of six distinct specification-conflict plans; remains independent. |
| Rule the benign-stream numerical contract — ``entry:assayer:harness-benign-stream-contract-ruling`` | RULING | OPEN | Reconcile the fixed stream length, reference rates, and promised terminal risk into one derivable contract, then update the intent's expected bounds. | (´plan:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk´) | One of six distinct specification-conflict plans; the tape runner supplies mechanism only. |
| Rule the discrimination class gate — ``entry:assayer:harness-discrimination-gate-ruling`` | RULING | OPEN | Choose the configured AUC class gate or change the specification and validation so the literal lower gate is reachable; the public witness then asserts the selected boundary. | (´plan:assayer:intent-discrimination-is-withheld-until-there-are-positives´) | One of six distinct specification-conflict plans; remains independent. |
| Rule variance factor versus standard-deviation factor — ``entry:assayer:harness-uncertainty-factor-ruling`` | RULING | OPEN | Decide whether the temporal factor multiplies variance or public standard deviation, align dimensional definitions, and make the intent assert only that form. | (´plan:assayer:intent-silence-inflates-reported-uncertainty-by-the-staleness-factor´) | One of six distinct specification-conflict plans; remains independent. |
| Rule the importance ceiling arithmetic — ``entry:assayer:harness-importance-ceiling-ruling`` | RULING | OPEN | Retain ceiling 500 with balance one half, retain one third with ceiling 250, or explicitly revise the equation or positive rate before an expected value lands. | (´plan:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance´) | One of six distinct specification-conflict plans; remains independent. |
| Measured enrichment-grid runtime budget — move (´entry:assayer:process-grid-runtime´) | VERIFICATION | OPEN | Record the maintained command, baseline distribution, tolerance, and overloaded rerun rule where the suite is defined; a budget breach becomes a repeatable finding. | (´sec:assayer:testing-architecture-multi-channel´) | Moves the live head from the testing plan into the backlog and gates the parked tape runner. |
| Remaining production persistent-clock sites — move (´entry:assayer:harness-stage-clock´) | ENGINEERING | OPEN | Every production persistent-time read goes through the injected clock, focused decay and restore tests pass, and the scenario-only verbs are cited as separate completed work. | (´dec:clock:two-domains´) | Moves the still-open production remainder into the backlog; the testing plan keeps a citation and retires its scenario-verb clause. |
| Seeded invariant packs — move (´entry:assayer:harness-property-packs´) | ENGINEERING | OPEN | Reproducible seeded sweeps cover the named mathematical invariants, print their seed on failure, attach to existing claims, and add no dependency. | (´dec:posterior:debug-oracle´) | Moves the open roadmap entry into the backlog and cites ``dec:harness:seeded-sweeps`` after it lands. |

**Table (Backlog projection counts by outcome class)** · `tab:assayer:testing-harness-record-audit-backlog-counts`

| Outcome class | Count |
| --- | ---: |
| RULING | 7 |
| ENGINEERING | 9 |
| RETIREMENT | 3 |
| VERIFICATION | 1 |
| PARKED | 1 |
| GUARDRAIL | 0 |
| **Total** | **21** |

The production-change intent-plan subset receives no clustering backlog entry. Its plan heads already own distinct engineering, dependencies, and acceptance conditions; a cluster would either copy those conditions or become a title with no executable acceptance. The backlog cites those plan heads only when a plan becomes a campaign prerequisite, and the concept cites its demand table rather than copying their entries.

The specification-conflict plans do belong in the backlog as distinct RULING rows because each asks a different question and ends under a different acceptance condition. A harness mechanism cannot select any of their alternatives.

The concept's kept entry, evidence tables, decided alternatives, record caveats, and acceptance summary do not belong in the backlog. Neither do the DONE skeleton, monotonic-clock, residual-race, and mid-batch roadmap entries. Completed work remains in the testing plan as history or retires to git history; standing decisions remain in records; the backlog contains only the open rows above.

**Table (Existing testing-plan and backlog relations)** · `tab:assayer:testing-harness-record-audit-backlog-relations`

| Existing environment | Formalization result |
| --- | --- |
| (´sec:assayer:testing-plan-gaps´) | Leaves every behavioral gap standing; individual gaps cite the new contract only when their eventual witness consumes it. |
| (´entry:assayer:harness-stage-skeleton´) | Stays DONE and cites the new harness record as the contract its landed pieces implement. |
| (´entry:assayer:harness-stage-clock-monotonic´) | Stays DONE and cites ``cor:clock:harness-control`` only for the scenario-facing consequence. |
| (´entry:assayer:harness-stage-clock´) | Its live mint moves to the backlog; the roadmap retains a citation and a concise historical split between production work and scenario verbs. |
| (´entry:assayer:harness-stage-clock-residual-race´) | Stays DONE; its barrier evidence cites the new barrier decision and concurrency corollary. |
| (´entry:assayer:mid-batch-identity-movement´) | Stays DONE and becomes the evidence cited by ``cor:ordering:tape-batches``. |
| (´entry:assayer:harness-stage-tapes´) | Retires as an umbrella after its open pieces become the discrete backlog rows above; its roadmap prose cites those rows and the new record instead of retaining OPEN status. |
| (´entry:assayer:harness-property-packs´) | Its live mint moves to the backlog; the roadmap cites it and the seeded-sweeps decision. |
| (´entry:assayer:process-grid-runtime´) | Its live mint moves to the backlog; process prose cites it, and the tape runner uses it as the revisit condition. |
| Existing Assayer backlog entries | Remain standing; none shares both a pin and acceptance condition with a projected row. |

## The concept and testing plan after formalization · `sec:assayer:testing-harness-record-audit-after-formalization`

Once a record decides an alternative, the plan no longer presents that alternative as its own standing opinion. It keeps the argument and migration evidence, changes normative sentences into citations of the new heads, and keeps only implementation ordering and sized work as plan content.

**Table (Concept sentences that become citations)** · `tab:assayer:testing-harness-record-audit-concept-rewrites`

| Concept locus | Sentence or claim after formalization |
| --- | --- |
| (´sec:assayer:testing-harness-concept-one-scenario´) | The opening shared-type sentence cites ``dec:harness:single-scenario``; the maintenance and multi-channel exclusions cite ``dec:harness:specialised-side-harnesses``; the migration-cost paragraph cites ``cav:harness:incremental-adoption``. |
| (´sec:assayer:testing-harness-concept-time-and-barriers´) | The time sentence cites ``cor:clock:harness-control``; the closed-set sentence cites ``cor:concurrency:harness-barriers``; the prohibition on local waits cites ``dec:harness:no-ad-hoc-waits``. |
| (´sec:assayer:testing-harness-concept-tapes´) | The opening playback sentence cites ``dec:harness:declarative-playback``; batch semantics cite ``cor:ordering:tape-batches``; progress semantics cite ``cav:harness:progress-not-latency``. |
| (´sec:assayer:testing-harness-concept-probes´) | The contract clauses cite ``dec:harness:probe-contract``; retained-state limits cite ``cav:retention:probe-boundary``; sealed-build invisibility cites ``cor:surface:test-support-opacity``. |
| (´sec:assayer:testing-harness-concept-oracles´) | The tier and provenance sentences cite ``dec:harness:oracle-tier``; the unwitnessed-oracle limit cites ``cav:harness:oracle-trust``. |
| (´sec:assayer:testing-harness-concept-guarded-fixtures´) | The opening obligation cites ``dec:harness:guarded-fixtures``; the distinct failure meaning cites ``dec:harness:separate-validation``. |
| (´sec:assayer:testing-harness-concept-rejected´) | Distributed design cites ``dec:harness:single-scenario``; engine doubles cite ``dec:harness:real-engine``; production widening keeps its existing surface and retention citations; support-tree absorption cites ``dec:harness:specialised-side-harnesses``; the framework alternative cites ``dec:harness:seeded-sweeps``. |
| (´sec:assayer:testing-harness-concept-migration´) | The order remains plan prose, with each phase citing its projected backlog entry and any governing record head; it mints no record convention. |
| (´obs:assayer:testing-harness-concept-not-the-critical-path´) | The limited-benefit claim cites ``cav:harness:limited-unblocking`` and leaves the production-change plans at their own heads. |
| (´obs:assayer:testing-harness-concept-probe-versus-retention´) | The resolution cites ``cav:retention:probe-boundary`` and removes the implication that every state witness must use a probe. |
| (´obs:assayer:testing-harness-concept-tape-versus-batch´) | The risk becomes evidence for ``cor:ordering:tape-batches``. |
| (´obs:assayer:testing-harness-concept-progress-not-latency´) | The risk becomes evidence for ``cav:harness:progress-not-latency``. |
| (´obs:assayer:testing-harness-concept-destination-concentration´) | The question cites ``cav:harness:convergence-concentration``, ``entry:assayer:defer-harness-convergence-layout``, and the ruling backlog entry. |
| (´obs:assayer:testing-harness-concept-oracle-trust´) | The mitigations cite ``cav:harness:oracle-trust`` without claiming proof. |
| (´obs:assayer:testing-harness-concept-merge-has-no-sponsor´) | The procedural answer cites ``cav:harness:incremental-adoption`` and the scenario-unification retirement entry. |
| (´obs:assayer:testing-harness-concept-guard-and-oracle´) | The selected separation cites ``dec:harness:separate-validation``. |
| (´sec:assayer:testing-harness-concept-acceptance´) | Each inspection criterion cites its governing decision or corollary and its backlog evidence; the probe sentence says every permitted test-support projection uses the contract, while a witness of state excluded by retention remains a direct crate-level witness. |

The acceptance correction is load-bearing. Its present universal statement over every state projection conflicts with the concept's own retention-risk resolution, which sends a witness of deliberately excluded state to crate-level direct access. Formalization must narrow the universal to permitted test-support projections rather than weakening the retention record.

**Table (Testing-plan roadmap after formalization)** · `tab:assayer:testing-harness-record-audit-testing-plan-rewrites`

| Roadmap material | Result |
| --- | --- |
| DONE skeleton and monotonic-clock stages | Stay as historical evidence and cite the new record and applicable corollaries. |
| Persistent-clock stage | Its still-open head moves to the backlog; the roadmap cites it and records the harness-facing split as superseded by scenario-time work. |
| Residual-race and mid-batch entries | Stay as completed evidence for barriers and batch semantics. |
| Stage-three umbrella | Retires as OPEN work when its discrete backlog entries land; the roadmap cites those entries rather than duplicating acceptance. |
| Property packs | The head moves to the backlog; the roadmap cites it and the seeded-sweeps decision. |
| Process runtime budget | The head moves to the backlog; the process section cites it. |
| Testing-plan gaps | Stay until their promises are witnessed; formalization changes their infrastructure citations, not their behavioral acceptance. |
| Harness declarations and golden stimulus | Stay as decisions already made by (´dec:assayer:harness-declares´) and (´dec:assayer:golden-report-stimulus´), and the new harness record cites them rather than restating them. |

## Work packages · `sec:assayer:testing-harness-record-audit-work-packages`

Five waves carry the projection. They follow the wave discipline (´conv:assayer:wave-discipline´), keep file overlap to the record register where possible, and make every label-bearing commit pass the corpus gate before another wave builds on it.

**Table (Bounded implementation lanes)** · `tab:assayer:testing-harness-record-audit-work-packages`

| Package | Files edited or created | Environments minted or moved | Register rows | Citations and dependencies |
| --- | --- | --- | ---: | --- |
| P1 — outline the exception | Edit ``packages/assayer/docs/plans/record-outline.md``. | Mint ``dec:assayer:record-harness-record`` and ``entry:assayer:record-harness``; update the master and coverage readings. | 0 | Cites the derivation, area-choice, clock-precedent, testing-policy, concept, and this audit heads. Starts first; P2 depends on P1. |
| P2 — land the harness record | Create ``packages/assayer/adr/harness.md``; edit ``packages/assayer/docs/adrs.md`` and ``packages/assayer/docs/deferred.md``. | Mint all 16 ``harness`` heads in the projection table and ``entry:assayer:defer-harness-convergence-layout``. | 16 | Cites the six spine families H0–H8 as applicable and the new outline ruling. Depends on P1; P3 and P4 depend on P2. |
| P3 — extend time and execution records | Edit ``packages/assayer/adr/clock.md``, ``packages/assayer/adr/concurrency.md``, ``packages/assayer/adr/construction.md``, ``packages/assayer/adr/ordering.md``, and ``packages/assayer/docs/adrs.md``. | Mint ``cor:clock:harness-control``, ``cor:concurrency:harness-barriers``, ``cor:construction:harness-declarations``, and ``cor:ordering:tape-batches``. | 4 | Cites spines E1–E4 and the landed harness record. Depends on P2; runs in parallel with P4. |
| P4 — extend persistence and boundary records | Edit ``packages/assayer/adr/durability.md``, ``packages/assayer/adr/retention.md``, ``packages/assayer/adr/surface.md``, and ``packages/assayer/docs/adrs.md``. | Mint ``cor:durability:harness-fork``, ``cav:retention:probe-boundary``, and ``cor:surface:test-support-opacity``. | 3 | Cites spines E5–E7 and the landed harness record. Depends on P2; runs in parallel with P3. |
| P5 — project open work and rewrite plan claims | Edit ``packages/assayer/docs/plans/backlog.md``, ``packages/assayer/docs/plans/testing-harness-concept.md``, and ``packages/assayer/docs/plans/testing_plan.md``. | Mint the 18 new backlog heads in the backlog table; move (´entry:assayer:process-grid-runtime´), (´entry:assayer:harness-stage-clock´), and (´entry:assayer:harness-property-packs´) into the backlog. | 0 | Cites every landed record environment, the concept work heads, all six conflict-plan heads, and this audit. Depends on P3 and P4. |

P1 runs alone because P2 needs a live outline head. P2 follows and establishes the record identity. P3 and P4 then run in parallel because their record files are disjoint. P5 waits for both so every normative sentence it replaces can cite a live mint and every backlog entry can pin to its governing record.

The shared record register uses a textual insertion protocol. P2 inserts a contiguous ``harness`` block after ``clock`` and before ``posterior``. P3 appends its projected row inside each of the ``clock``, ``concurrency``, ``construction``, and ``ordering`` blocks. P4 appends its projected row inside each of the ``durability``, ``retention``, and ``surface`` blocks. Each lane edits only its complete block hunk; after P3 and P4 rebase on P2, their changes occupy distinct positions and require no row interleaving.

The outline is touched only by P1, the deferral register only by P2, and the planning documents only by P5. No lane edits a record another lane edits.

## Findings · `sec:assayer:testing-harness-record-audit-findings`

**Observation (The records constrain the concept but do not contradict its choices)** · `obs:assayer:testing-harness-record-audit-no-record-conflict`

No primary concept decision contradicts an existing record. The apparent conflicts are premises the records have already settled: the separated clock domains, per-request snapshot load, absence of a batch view, retention exclusions, production opacity, host declarations, and the debug-oracle precedent. Formalization cites those heads and records only the remaining test-contract alternatives.

**Observation (The acceptance universal conflicts with the retention resolution)** · `obs:assayer:testing-harness-record-audit-probe-universal-conflict`

The concept says every test-tree state projection uses the probe contract, while its retention risk sends witnesses of deliberately excluded state to direct crate-level access. Both cannot stand literally. The record keeps retention strict and narrows acceptance to every state projection the test-support contract permits; it does not create a probe for excluded precision or widen a production surface.

**Observation (The current architecture contradicts the target state as evidence, not policy)** · `obs:assayer:testing-harness-record-audit-architecture-gap`

The architecture report finds several scenario models, a polling wait that can return stale progress, a specialised tree that repeats routing, and callerless or stale harness material (´rep:assayer:testing-architecture´). Those readings establish retirement work and fails-before evidence. They do not authorize copying current implementation structure into the record.

**Observation (The new record is a controlled exception to the strict derivation)** · `obs:assayer:testing-harness-record-audit-derivation-exception`

The existing record set projects the layers, which project specification choices. The harness record projects neither a new specification behavior nor a new layer. It earns a further place only through an explicit outline ruling analogous to the clock precedent: a cross-cutting discipline would otherwise be copied or assigned to records that do not govern tests. The implementation lanes must not add the file or register rows before that ruling lands.

**Observation (The specification conflicts remain distinct rulings)** · `obs:assayer:testing-harness-record-audit-six-rulings`

The conditioning trigger, registration lift, benign-stream numerical contract, discrimination class gate, uncertainty multiplier, and importance-ceiling arithmetic present different alternatives and acceptance conditions. No harness decision settles them, and neither a shared ruling nor a fixture implementation may conceal their differences.

**Observation (Production-change intent plans already own their engineering)** · `obs:assayer:testing-harness-record-audit-production-plan-homes`

The production-change plans retain their own heads and acceptance conditions. The backlog must not copy them into a cluster, and the harness record must not claim to unblock behavior whose production prerequisite remains absent.

**Observation (Migration order is planning, not a record convention)** · `obs:assayer:testing-harness-record-audit-migration-not-record`

The barrier-first order and incremental scenario unification remain in the concept and backlog dependency graph. Minting them as a record convention would turn a campaign schedule into a permanent testing contract, so the record carries only the incremental-adoption caveat needed to keep its present-tense claim honest.

**Observation (Playback inherits ordering and liveness limits)** · `obs:assayer:testing-harness-record-audit-playback-limits`

The tape runner may batch for cost but may not imply shared visibility, and its progress counter may identify a stall but may not become a latency budget. P3 must land both consequences before P5 rewrites the concept as decided.

**Observation (The audit never becomes a register)** · `obs:assayer:testing-harness-record-audit-reading-only`

Every table here is a reading table. Proposed heads remain double-backtick displays, proposed document values remain displays, and the exact register rows become assertions only when the record-register tracking table receives them in P2, P3, and P4.

**Observation (Formalization must not widen scope)** · `obs:assayer:testing-harness-record-audit-scope-guards`

The bounded lanes must not change specification behavior, implementation, public API, dependencies, test claims, production metrics, or the production-change intent plans. They must not absorb the specialised maintenance or multi-channel harnesses, invent a new status or outcome class, put closed work in the backlog, or restate a decision whose live head already exists.
