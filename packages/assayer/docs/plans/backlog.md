# The Assayer Backlog · `plan:assayer:documentation-migration-campaign`

This file holds only open Assayer work, the terrain that explains it, and the standing decisions that still govern the corpus. Completed work leaves the file; git history and the landed audit, outline, register, specification, and record documents are its archive.

The label at each heading or environment head is that environment's mint. A parenthesized label in running text is a same-owner citation, and an imported root citation carries the `INDEX` prefix. Every open item below is pinned to the live surface or governing head that makes it actionable.

## Charter · `sec:assayer:charter`

**Desideratum (What done looks like)** · `goal:assayer:migration-goal`

The Assayer documentation corpus remains under the label calculus and the repo check. Every open migration obligation has one owning backlog entry; an entry leaves this file when its work and acceptance evidence land.

**Convention (Status grammar)** · `conv:assayer:status-grammar`

Every entry is **OPEN** or **PARKED**. OPEN work has an executable acceptance condition. PARKED work names the revisit condition. ACTIVE and completed state belong to work execution and git history, not to the live backlog.

**Convention (Outcome classes)** · `conv:assayer:outcome-classes`

Every backlog entry carries exactly one outcome class from this closed vocabulary. **RULING** ends in a recorded decision among unresolved alternatives. **ENGINEERING** ends in implemented behaviour and its focused tests. **GUARDRAIL** ends in a repeatable generator, check, lint, or execution gate. **RETIREMENT** ends in deletion or an explicit rejection that removes a legacy surface. **VERIFICATION** ends in rederived evidence and corrected documentary truth without a new operational capability as its primary result. **PARKED** ends only when its stated revisit condition becomes true; it is not active work before then. A class describes the entry's terminal outcome, not every intermediate step or prerequisite.

**Convention (Wave discipline)** · `conv:assayer:wave-discipline`

Work lands one entry or one coherent entry cluster at a time. A document rewrite proceeds audit, labeled outline, outline review, rewrite, lint, and publication. Tooling needed to enforce a document convention lands before the document relies on it (`goal:assayer:tools-over-discipline`).

**Desideratum (Tools over discipline)** · `goal:assayer:tools-over-discipline`

Every convention this corpus adopts must be enforced by a tool rather than by care. A wave that discovers a reliance on author diligence records the tooling gap before proceeding.

## Terrain · `sec:assayer:terrain`

**Data (The remaining Assayer work)** · `data:assayer:terrain-survey`

The specification, decision-record set, testing plan, conformance audit, and repair campaign have landed. The remaining local work now divides into register-generation adoption, the Companion boundary, landscape reproduction, source retirement, enforced quality gates, and upstream-boundary dispositions. The inventory prerequisite those groups consume is already in place: the checked policy `profile.legacy-conform` and the burn family `legacy.implementation` hold the legacy implementation's unlabelled remainder at zero under the Assayer adoption (`dec:assayer:burn-lists`). Work owned by the repository's own linter and configuration packages remains in those packages' backlogs, and condition-bound work remains PARKED here rather than entering an active campaign.

**Decision (The tag system is superseded)** · `dec:assayer:tags-superseded`

The label calculus supersedes the former two-level tag system. No new tag-form reference may enter the corpus; the repository rule and its burn ratchet are (`dec:migration:superseded-forms`) and (`req:migration:burn-ratchet`).

## Tooling · `sec:assayer:tooling`

This group delivers repeatable corpus inventories. The checked legacy-surface policy that later retirement consumes is installed; what remains here is the record-register form.

**Entry (Generated register sections adopt the linter policy)** · `entry:assayer:tool-generated-registers`

**Outcome class: ENGINEERING.**

**OPEN.** Generation wins, and the mechanism is not built here — it is generalized into a proper policy of the repository's linter, owned by that package under its backlog head ``entry:indexlinter:register-projection-policy`` (the citation graph gives this package no reach into the linter's labels, so the pointer is displayed). That linter is a tool the repository carries and this package builds without. What remains in this package is adoption: when the linter policy lands, the record register's tracking table becomes a generated, nonparticipating region under exact regeneration and staleness checking. The live record register (`reg:assayer:decision-record-register`) is the acceptance surface, and this entry closes when it regenerates cleanly under the deployed policy.

## Standing rulings · `sec:assayer:rulings`

These decisions are not completed tasks. They remain because audit records, outlines, registers, or future work still cite or apply them.

**Decision (Scaffolding retirement)** · `dec:assayer:scaffolding-retirement`

The pre-rewrite specification and tag document retire only after a recorded cross-audit proves their useful content represented in the replacement. That retirement has occurred, but the decision remains the rule cited by the audit that justified it.

**Decision (Names move to the calculus)** · `dec:assayer:naming-schema`

All Assayer identities use the label calculus. Section locators, tag forms, and numbered record names carry no identity in the migrated corpus. The concrete record schema is (`dec:assayer:record-naming-schema-text`).

**Decision (The record naming schema)** · `dec:assayer:record-naming-schema-text`

Each Assayer decision record owns one short, singular, content-derived area. Every environment in the record mints under that area; the filename is the area with no ordering prefix; and the opening decision head is the record's citable identity. Running prose names the record by subject and cites that identity. The live record-set outline (`plan:assayer:decision-record-architecture`) fixes the area set, and the record register (`reg:assayer:decision-record-register`) fixes the projection.

**Decision (The specification is authoritative)** · `dec:assayer:spec-authoritative`

`docs/spec.md` is authoritative, including Part IV. Where code and specification diverge, code is the legacy party: the shipped pre-rewrite derivation vocabulary is tracked for migration and is never written back into the specification. That tracking is in place: the one surviving legacy site carries its derived inventory label at the marker line, and the checked policy `profile.legacy-conform` with the burn family `legacy.implementation` holds the unlabelled remainder at zero under the Assayer adoption (`dec:assayer:burn-lists`).

**Decision (Burn lists govern legacy migration)** · `dec:assayer:burn-lists`

Every local legacy family is enumerated in an exact burn register that may only shrink. The repository-wide ratchet is (`req:migration:burn-ratchet`); this decision retains the Assayer adoption and its local sequencing. Roman-numeral locators are section references like any other, while quotations of external standards retain only their per-rule exemptions.

**Decision (The record set is a projection)** · `dec:assayer:projection-principle`

The decision records are a projection of the specification: layer documents project its concepts, and the exact record set projects the conceptual choices of the layers. No legacy record partition has tenure. The live projection is the record-set outline (`plan:assayer:decision-record-architecture`) and the record register (`reg:assayer:decision-record-register`).

**Decision (Upstream APIs held)** · `dec:assayer:upstream-apis-scope`

`docs/upstream-apis.md` remains outside this campaign except for genuine bugs found by cross-audits. Its own migration belongs to a future campaign.

**Decision (The testing plan migrates last)** · `dec:assayer:testing-plan`

The testing plan migrated rather than retiring. Its live identity is (`plan:assayer:test-debt-plan`); this decision remains the ordering rule cited by its migration audit.

**Decision (The test documentation policy)** · `dec:assayer:test-documentation-policy`

A test's documentation is one authored sentence carried where the test is, and every table that repeats it is generated. Each covered test carries an authored claim mint or a sibling-claim citation followed by its derived test label. The generated projections are the in-file test index and the per-folder matrix, each named by a repository-wide convention this package adopts rather than defines. The policy behind them is repository-wide too; this decision is the Assayer adoption of it, cited by the landed plan and audit.

**Decision (Audit before conformance)** · `dec:assayer:conformance-audit-first`

No conformance repair begins before a full audit, except improvements to the policy-labeled notices themselves and separately authorized harness infrastructure. The audit has closed; its durable record is (`rep:assayer:code-conformance-campaign`).

**Decision (The repair waves are ratified in part)** · `dec:assayer:repair-waves-ratified`

The first authorization admitted waves one, three, four, five, six, and nine in the sequencing register's dependency order and held the remaining waves for explicit decisions of their own. It remains because the complete authorization (`dec:assayer:repair-waves-ratified-entire`) cites and supersedes it.

**Decision (Code comments cite, they do not restate)** · `dec:assayer:code-comments-cite`

Assayer code comments state only what code cannot show. Where the corpus already states a decision, commentary cites the governing head instead of restating prose that can drift.

**Decision (The held waves are ratified and the campaign is authorized entire)** · `dec:assayer:repair-waves-ratified-entire`

All repair waves are authorized under the audit's sequencing register. The pre-deployment compatibility choices are accepted for value changes, layout invalidation, public-surface repair, host-set channel capacity, and the decision-landscape build. The campaign has landed, while this ruling remains cited by its conformance record.

**Decision (Deferrals live in the records)** · `dec:assayer:deferred-home`

Each deferral lives in the decision record that owns the deferred choice. `docs/deferred.md` is a checked register of citations out of those records and holds no parallel statement of the deferred decisions.

## Campaign strategy · `sec:assayer:campaign-strategy`

**Recommendation only.** The ordering and lane boundaries in this section decide nothing. They describe the shortest dependency path exposed by the study and leave every unresolved choice in the ruling queue below.

**Recommended order.** Begin with the tooling group (`sec:assayer:tooling`). Its register-generation adoption waits on the linter policy it cites rather than on anything in this file, so it starts when that policy lands and blocks nothing while it waits. In parallel, the independent dispositions in the upstream-boundary group (`sec:assayer:campaign-upstream-boundaries`) may be prepared for decision, since that work has no code dependency.

Take the Companion group (`sec:assayer:campaign-companion`) next: its arrangement is settled (`dec:challenge:arrangement-final`), and the replacement and configuration work now share that ratified contract. Landscape reproduction (`sec:assayer:campaign-landscape`) follows the Companion contract because the posterior form is one of the executable projection's inputs.

The shared testing-harness campaign (´sec:assayer:campaign-testing-harness´) follows the dependency order retained by the concept (´sec:assayer:testing-harness-concept-migration´): barriers precede local-wait retirement, scenario time and complete construction precede probes and oracles, the persistence fork follows those shared surfaces, and scenario unification proceeds through every touched test. Shared playback remains outside that active order until its runtime-budget revisit condition fires (´entry:assayer:harness-tape-runner´).

Finish active source work with source policy and retirement (`sec:assayer:campaign-source-retirement`). Its inventory prerequisite is already met — the checked policy and burn family hold the legacy implementation's unlabelled remainder at zero — so retirement begins against a register rather than against a search. Establish the declared performance route (`sec:assayer:campaign-performance-route`) after those surfaces stabilise, so its baseline and schedule measure the system being retained rather than an intermediate one. The condition-bound entries in the observability (`sec:assayer:campaign-observability`) and claim-area (`sec:assayer:campaign-claim-areas`) groups remain outside the active order until their stated revisit conditions are met.

**Ruling queue.** These entries need a recorded decision before their choice-dependent implementation or corpus edit can start:

- (`entry:assayer:landscape-verification`) — every underdetermined reading and the surfaced forms of crossover covariance and regime-width uncertainty.
- (`entry:assayer:dropped-upstream-dispositions`) — the producing-set correspondence and abstraction-boundary inventory dispositions.
- (´entry:assayer:harness-convergence-module-ruling´) — the convergence witness layout before concentrated long-running witnesses land.
- (´entry:assayer:harness-registration-lift-ruling´) — exact extension, a later-label counterfactual, or registration plus first report.
- (´entry:assayer:harness-benign-stream-contract-ruling´) — one derivable contract for the benign stream's horizon, rates, and terminal risk.
- (´entry:assayer:harness-discrimination-gate-ruling´) — the configured class gate or a revised reachable boundary.
- (´entry:assayer:harness-importance-ceiling-ruling´) — one consistent choice of ceiling, balance, and expected arithmetic.

The maintained performance route in (`entry:assayer:performance-tests-gated`) is also a recommended decision before CI work, because the alternatives impose different standing execution costs; the entry's terminal outcome remains a GUARDRAIL. The parked identity tunability, upstream API, and claim-area entries are not an active ruling queue: their existing revisit conditions must fire first.

**Immediately dispatchable bounded lanes.** Within the source-retirement group, the retirement of legacy code and stale scaffolding is a bounded entry lane that can start now against the installed inventory, and the lint-suppression audit is a second. The notice-disposition entry in the same group builds its own recognizer before it can enumerate what it owns, so it is not bounded in the same way. Every other active entry is held either by a decision in the queue above or, in the tooling group's case, by the linter policy it cites. Neither of the two named lanes requires a decision to begin.

## Shared testing-harness campaign · `sec:assayer:campaign-testing-harness`

These entries stay together because they turn one shared testing contract into executable work while preserving a distinct outcome, acceptance condition, and live pin for every independently closable obligation.

**Entry (Closed queue-barrier set)** · `entry:assayer:harness-closed-barriers`

**Outcome class: ENGINEERING.**

**OPEN.** Give every asynchronous queue reachable by a test one documented barrier; each barrier names its coverage and exclusion, and focused tests stall or pass on the correct queue. The live pin is (´entry:assayer:testing-harness-concept-barrier-set´), and the result implements the declared barrier contract (´cor:concurrency:harness-barriers´).

**Entry (Scenario time verbs)** · `entry:assayer:harness-scenario-time`

**Outcome class: ENGINEERING.**

**OPEN.** Make the scenario advance and target both clock domains without backward travel and automatically cross every barrier the operation requires. The live pin is (´entry:assayer:testing-harness-concept-time-verbs´), and the result implements scenario control of the separated domains (´cor:clock:harness-control´).

**Entry (Complete host declaration builder)** · `entry:assayer:harness-complete-builder`

**Outcome class: ENGINEERING.**

**OPEN.** Declare every required construction input through one builder, reject invalid declarations at construction, and make deterministic seeds explicit. The live pin is (´entry:assayer:testing-harness-concept-complete-builder´), and the result completes the harness's construction obligation (´cor:construction:harness-declarations´).

**Entry (Probe contract and first projections)** · `entry:assayer:harness-probe-contract`

**Outcome class: ENGINEERING.**

**OPEN.** Enforce gating, one published load, owned copies, no mutation, scenario time, retention limits, and sealed-build invisibility through the contract, with focused tests for its first projections. The live pin is (´entry:assayer:testing-harness-concept-probe-contract´); excluded state remains outside the contract (´cav:retention:probe-boundary´), and the gated surface remains sealed from hosts (´cor:surface:test-support-opacity´).

**Entry (Shared tape runner)** · `entry:assayer:harness-tape-runner`

**Outcome class: PARKED.**

**PARKED.** Revisit only when (´entry:assayer:process-grid-runtime´) records a runtime budget and tolerance; then create an ENGINEERING entry whose acceptance covers typed rows, barrier policy, checkpoints, progress, and cost-only batches. The live pin is (´entry:assayer:testing-harness-concept-tape´), and the future runner remains governed by declarative playback (´dec:harness:declarative-playback´).

**Entry (Independent oracle tier)** · `entry:assayer:harness-oracle-tier`

**Outcome class: ENGINEERING.**

**OPEN.** Make the rank fold, regularised Schur complement, decay recurrence, and dimension-width formula declare provenance, avoid production routes, serve multiple callers, and each separate from a deliberately defective arm. The live pin is (´entry:assayer:testing-harness-concept-oracles´), and the result implements the shared oracle contract (´dec:harness:oracle-tier´).

**Entry (Guarded fixture default)** · `entry:assayer:harness-guarded-fixtures`

**Outcome class: ENGINEERING.**

**OPEN.** Make every state-establishing fixture check its named precondition before returning and report the measured baseline on failure; tests distinguish setup failure from result disagreement. The live pin is (´entry:assayer:testing-harness-concept-guarded-fixtures´), and the distinction implements separate validation (´dec:harness:separate-validation´).

**Entry (Isolated persistence fork)** · `entry:assayer:harness-persistence-fork`

**Outcome class: ENGINEERING.**

**OPEN.** Quiesce a witness, checkpoint it, copy it into isolated storage, rebuild it under the same injected clock, run both arms, and compare one normalized durable projection. The live pin is (´entry:assayer:testing-harness-concept-persistence-fork´), and the result implements the isolated fork contract (´cor:durability:harness-fork´).

**Entry (Unify the Assayer scenario model)** · `entry:assayer:harness-scenario-unification`

**Outcome class: RETIREMENT.**

**OPEN.** Remove the parallel crate configuration, entity, report, and wait vocabularies; make all caller surfaces reach the shared scenario while documented specialised side harnesses remain. The live pin is (´entry:assayer:testing-harness-concept-merge-scenario-models´), and completion makes the single-scenario contract descriptive (´cav:harness:incremental-adoption´).

**Entry (Retire polling, sleeps, and local waits)** · `entry:assayer:harness-polling-retirement`

**Outcome class: RETIREMENT.**

**OPEN.** Make search and the linter find no test-authored sleep, poll, or wait outside the harness liveness implementation, with every removed site naming its replacement barrier. The live pin is (´entry:assayer:testing-harness-concept-retire-polling´), and completion enforces the no-wait decision (´dec:harness:no-ad-hoc-waits´).

**Entry (Retire callerless helpers and stale staging notes)** · `entry:assayer:harness-stale-helper-retirement`

**Outcome class: RETIREMENT.**

**OPEN.** Give every harness item a caller or delete it, remove stale staging names, and make generated module documentation report no orphan. The live pin is (´entry:assayer:testing-harness-concept-retire-callerless´); completion closes the unreported-decay and staging-note findings (´obs:assayer:testing-architecture-unreported-decay´) (´obs:assayer:testing-architecture-staging-notes´).

**Entry (Rule the convergence-module layout)** · `entry:assayer:harness-convergence-module-ruling`

**Outcome class: RULING.**

**OPEN.** Decide whether to retain or split the convergence subject before concentrated long-running witnesses land; record the runtime, maintenance, and claim-cohesion criterion and update the deferral citation. The live pin is (´obs:assayer:testing-harness-concept-destination-concentration´), and the ruling cites both the owning caveat (´cav:harness:convergence-concentration´) and deferral head (´entry:assayer:defer-harness-convergence-layout´).

**Entry (Rule exact extension versus registration lift)** · `entry:assayer:harness-registration-lift-ruling`

**Outcome class: RULING.**

**OPEN.** Select shared-block widening, a later-label counterfactual, or registration plus first report; align the specification, record, and intent before fixture work. The live pin is (´plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it´).

**Entry (Rule the benign-stream numerical contract)** · `entry:assayer:harness-benign-stream-contract-ruling`

**Outcome class: RULING.**

**OPEN.** Reconcile the fixed stream length, reference rates, and promised terminal risk into one derivable contract, then update the intent's expected bounds. The live pin is (´plan:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk´).

**Entry (Rule the discrimination class gate)** · `entry:assayer:harness-discrimination-gate-ruling`

**Outcome class: RULING.**

**OPEN.** Choose the configured AUC class gate or change the specification and validation so the literal lower gate is reachable; the public witness then asserts the selected boundary. The live pin is (´plan:assayer:intent-discrimination-is-withheld-until-there-are-positives´).

**Entry (Rule the importance ceiling arithmetic)** · `entry:assayer:harness-importance-ceiling-ruling`

**Outcome class: RULING.**

**OPEN.** Retain ceiling 500 with balance one half, retain one third with ceiling 250, or explicitly revise the equation or positive rate before an expected value lands. The live pin is (´plan:assayer:intent-the-importance-ceiling-and-not-the-data-sets-gradient-balance´).

**Entry (Measured enrichment-grid runtime budget)** · `entry:assayer:process-grid-runtime`

**Outcome class: VERIFICATION.**

**OPEN.** Record the maintained command, baseline distribution, tolerance, and overloaded rerun rule where the suite is defined; a budget breach becomes a repeatable finding. The live pin is (´sec:assayer:testing-architecture-multi-channel´), and this evidence is the revisit condition for the parked tape runner (´entry:assayer:harness-tape-runner´).

**Entry (Remaining production persistent-clock sites)** · `entry:assayer:harness-stage-clock`

**Outcome class: ENGINEERING.**

**OPEN.** Route every production persistent-time read through the injected clock, pass focused decay and restore tests, and cite scenario-only verbs as separately completed work. The live pin is (´dec:clock:two-domains´); the harness-facing remainder belongs to scenario time (´entry:assayer:harness-scenario-time´).

**Entry (Seeded invariant packs)** · `entry:assayer:harness-property-packs`

**Outcome class: ENGINEERING.**

**OPEN.** Cover the named mathematical invariants with reproducible seeded sweeps, print the seed on failure, attach the sweeps to existing claims, and add no dependency. The live pin is (´dec:posterior:debug-oracle´), and the implementation follows the framework-free sweep decision (´dec:harness:seeded-sweeps´).

## Health and observability · `sec:assayer:campaign-observability`

This group's producer-to-public-output delivery has landed, including its loss counters, configured reading thresholds, and generated whole-report view. What remains here is condition-bound and waits on its stated revisit condition.

**Entry (Host-tunable identity convergence)** · `entry:assayer:identity-health-tunability`

**Outcome class: PARKED.**

**PARKED.** Revisit the later decision named at (`cav:health:dead-convergence-thresholds`) only when the specification defines the identity-convergence ladder and the quantities a host setting would control, or when a concrete host requirement supplies those semantics. Do not revive the deleted threshold names as an interim surface.

## Companion boundary and determinism · `sec:assayer:campaign-companion`

This group proceeds against one coherent Companion contract (`dec:challenge:arrangement-final`): the replaceable estimator and clocked health report are the whole surface, challenge metric emission remains with the host, and no mapper is added. What remains is clock-controlled behaviour and coverage against that arrangement.

**Entry (The Companion meets its replaceable contract)** · `entry:assayer:companion-contract`

**Outcome class: ENGINEERING.**

**OPEN.** Complete the behaviour behind the final two-surface arrangement (`dec:challenge:arrangement-final`): make reads apply elapsed-time decay without mutation (`alg:companion:inference`), keep the Companion's decay rate in Companion-owned configuration (`def:companion:challenge-decay`), complete its health report (`tab:companion:health`), and exercise the public estimator replacement surface (`req:companion:replacement-trait`). Acceptance is clock-controlled read-decay coverage, shared-tracker configuration independent of channel rewards, every health field populated from its named source, and a non-Beta provider consumed through the replacement contract without editing the concrete tracker.

## Decision-landscape reproducibility · `sec:assayer:campaign-landscape`

This group delivers one executable oracle from the presentation-free landscape through every published worked figure and rendering rider.

**Entry (Landscape figures and riders reproduce)** · `entry:assayer:landscape-verification`

**Outcome class: VERIFICATION.**

**OPEN.** Recompute the worked landscapes and rendered examples from the shipped, presentation-free landscape surface (`chap:spec:worked-landscapes`) and (`ex:rendering:examples`), and resolve every underdetermined reading by a recorded decision rather than an implementation guess. The verification must also decide and test the surfaced form of crossover covariance and regime-width uncertainty (`thm:landscape:crossover-covariance`) and (`prop:landscape:width-variance`). Acceptance is an executable projection that reproduces every published figure, fails on each known defective figure, and links every added corpus contract to its governing head.

## Upstream boundary dispositions · `sec:assayer:campaign-upstream-boundaries`

This group delivers explicit ownership and disposition of Assayer's upstream boundary documents without silently reopening the separately scoped API map.

**Entry (The dropped upstream dispositions close)** · `entry:assayer:dropped-upstream-dispositions`

**Outcome class: RULING.**

**OPEN.** Take the remaining upstream-reference recommendations to a recorded decision: the producing-set correspondence (`rec:assayer:dropped-upstream-producing-set`) and the abstraction-boundary inventory (`rec:assayer:dropped-upstream-boundary-range`). Acceptance is an explicit recorded disposition for both, the accepted content applied at its owning surface, and the report amended with citations to the decisions that closed it.

**Entry (The upstream API map receives its own campaign)** · `entry:assayer:upstream-api-campaign`

**Outcome class: PARKED.**

**PARKED.** The API boundary map remains outside this campaign by ruling (`dec:assayer:upstream-apis-scope`), despite being a live Assayer document with its own migration surface (`rep:assayer:upstream-api-boundaries`). Revisit when that future campaign opens or a cross-audit finds a genuine boundary bug; at that point re-census its labels, status claims, deferral links, imports, and module-location inventory against the live upstream crates before editing.

## Source policy and retirement · `sec:assayer:campaign-source-retirement`

This group delivers a source tree whose remaining notices and suppressions are owned by checked policy and whose legacy compatibility surfaces have retired.

**Entry (The admitted source notices have owners)** · `entry:assayer:code-notice-disposition`

**Outcome class: RETIREMENT.**

**OPEN.** The code slice carries labeled notices that are neither owned by a cited decision-record deferral under (`dec:assayer:deferred-home`) nor separated into the entries below; the set runs to dozens rather than to a handful, which is the only thing about its size that does not go stale. This entry states no count of it, and it cannot cite one either. The to-do profile's census tallies covered and labelled notices for the workspace rather than for a package, and the narrower set this entry owns — the notices no record deferral and no sibling entry has taken — is exactly what the acceptance criterion below builds a recognizer for, so nothing publishes that figure yet. A number written here would be a hand copy of a reading nothing re-takes, and the two removals this paragraph goes on to record are the very motion that makes such a copy wrong. Until the recognizer lands, the set is enumerated where it stands: every notice carries its derived notice-kind label at its standard place, so the crate's sources are the register and reading them is the count. They range across timestamp validation (`todo:code:make-this-a-hard-release-invariant`), concordance observation (`todo:code:feed-the-concordancetracker-one-n-d`), semantic interaction templates (`todo:code:migrate-this-legacy-offset-based-enum`), and test-oracle disposition (`todo:test:tighten-this-once-the-derivation-path`). The cross-layer health projection this list also carried is gone rather than disposed of: the field it stood for now has a producer, so the notice was removed with the placeholder it described (`entry:health:cross-layer-latency`). The Ledger routing notice this list carried has gone the same way rather than been dispositioned: the assessment path now takes the depth walk the notice asked for, so the notice left with the divergence it disclosed (`entry:assayer:wl-ledger-root-only-read`). The conformant Companion isolation notice also needs an explicit rejection and retirement rather than implementation (`todo:code:only-tests-construct-the-challenge-effectiveness`). Acceptance is a crate source-audit test that admits only notices citing a live record deferral or a distinct OPEN/PARKED entry, followed by disposition of every notice owned here and a clean package source-audit test run.

**Entry (Legacy code and stale scaffolding retire)** · `entry:assayer:legacy-code-retirement`

**Outcome class: RETIREMENT.**

**OPEN.** The checked policy `profile.legacy-conform` and the burn family `legacy.implementation` under (`dec:assayer:burn-lists`) inventory the legacy implementation and hold its unlabelled remainder at zero; they do not retire the code after it is known. Remove or replace the stale layer scaffolding (`todo:code:retire-stale-layer-scaffolding`), the legacy `DimensionMapSnapshot` (`todo:code:retire-legacy-dimensionmapsnapshot`), and the snapshot `DriftState` compatibility re-export (`todo:code:retire-the-snapshot-driftstate-re-export`) as their callers migrate. Acceptance is the checked legacy-implementation register at zero, all three source notices absent, and both linter check and burn clean.

**Entry (Lint suppressions carry checked reasons)** · `entry:assayer:lint-suppression-justifications`

**Outcome class: GUARDRAIL.**

**OPEN.** The slice contains hundreds of `allow` sites: some carry a local, specific reason, while others carry no adjacent reason or stale layer-era prose. Audit each suppression under (`todo:code:justify-or-retire-lint-suppressions`), removing it where the code no longer needs it and recording a narrow justification where it remains. Acceptance is a third feed-forward rule in `ci/lint_assayer.sh` that rejects an unjustified `allow` or `expect`, that script returning success, and the package clippy gate remaining clean without a crate-wide exception for the new rule.

## Declared performance route · `sec:assayer:campaign-performance-route`

This group delivers one maintained command and schedule for every performance witness, with no ignored or dependency-only alternative left outside the gate.

**Entry (Performance tests run under a declared gate)** · `entry:assayer:performance-tests-gated`

**Outcome class: GUARDRAIL.**

**OPEN.** Five coarse performance witnesses are ignored by default (`test:crate:health-summary-latency`) (`test:crate:full-health-report-latency`) (`test:crate:label-boundary-latency`) (`test:crate:builder-construction-latency`) (`test:crate:preseed-throughput`), as is the reference-dimension marginalisation witness (`test:crate:schur-complement-perf`), while the manifest declares Criterion without a benchmark target. Choose one maintained performance route: either register and gate Criterion benchmarks or schedule the ignored release-profile witnesses and remove the unused dependency. Acceptance is the chosen performance command named in `ci/lint_assayer.sh`, that command passing, and no ignored performance test or benchmark dependency left outside the declared route.

## Claim-area evolution · `sec:assayer:campaign-claim-areas`

This group preserves coherent test-claim stakes by waiting for evidence that a mixed area has outgrown one statement, then proving any eventual split exactly.

**Entry (Mixed claim areas split when they grow)** · `entry:assayer:area-register-split`

**Outcome class: PARKED.**

**PARKED.** The area register identifies `audit` and `scenario` as deliberate mixtures (`sec:assayer:area-register`). Revisit when either area gains enough claims that one stake statement no longer describes what is lost when its claims fail; then split the area, migrate its claims, and let the area-register and claim-profile checks prove the move complete.
