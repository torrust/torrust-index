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

## Introduction readiness · `sec:assayer:introduction-readiness`

**Entry (The introduction describes the package a host can use)** · `entry:assayer:re-cut-readiness`

**Outcome class: VERIFICATION.**

**OPEN.** The introduction's front door (`guide:assayer:bayesian-posture-engine`) describes implementation layers but supplies neither a runnable integration nor the configuration and public-surface map needed to use the crate. The installation, worked usage, operational model, API map, and development routes supplied by the spatial index (`[MUDLARK-guide:mudlark:overview]`) and measurement instrument (`[SENTINEL-guide:sentinel:overview]`) establish the corresponding reader needs; their historical version literals, superseded references, and commit conventions are not authority for this package. The re-cut carries the following checkable README obligations.

- **Purpose and suitability:** the opening states the host's problem, labelled-outcome learning from detached Sentinel reports and host signals, calibrated risk with uncertainty, and the separation between Core belief, host-owned challenge evidence, pure decision landscapes, and optional rendering; it links the purpose and principles (`chap:spec:purpose-and-principles`).
- **Stability and installation:** the dependency example uses the package's declared `0.1.0` line, states its pre-stable API status and inherited Rust `1.90` minimum, and distinguishes a version requirement from verified registry availability; it neither copies Sentinel's stale `0.1` example nor promises a published release without publication evidence.
- **Quick start:** a complete public-API example declares the required command-channel capacity, explicitly supplies a signal schema even when empty, builds the engine, registers and receives a real report, assesses a request, derives a landscape, submits a corresponding label, and shuts down; every identifier and argument is supplied, and the README is included in executable doctests rather than an ignored or uncompiled example.
- **Construction and lifecycle:** the example and API map distinguish eager validation, cold start and restore, post-construction registration and pre-seeding, asynchronous label acknowledgement from publication, lifecycle completion, and shutdown, citing the validated lifecycle (`rec:construction:validated-lifecycle`) and host-facing contracts (`rec:surface:host-facing-contracts`).
- **Derivation API:** every `derive_reckoning()` call is replaced by the actual `derive_landscape(&RiskAssessment, &ChannelPolicy, impl Into<ChallengePosteriorInput>) -> DecisionLandscape` surface; `render_resonances(&DecisionLandscape, &RiskBasis, &ResonanceConfig) -> ResonanceProfile` is separately optional, and policy validation precedes derivation (`sig:landscape:derivation-function`) (`sig:rendering:contract`).
- **Model roles:** operational and sister training populations and forgetting rates are distinguished; the anchor's fixed subspace and the registration-derived widths are explained; outcome-axis models predict host-declared labelled quantities rather than input signal types (`def:axis:per-axis-model`) (`tab:eligibility:training`).
- **Runtime architecture:** a data-flow account names model stewardship, independently published model and report snapshots, the identity-maintenance thread, the Ledger-GC scheduler, and the optional persistence scheduler; it separates those long-lived threads from the bounded scoped helpers used for label updates (`rec:concurrency:snapshot-stewardship`) (`rec:memory:spatial-state-custody`).
- **Concurrency corrections:** the statement that all model updates execute on the model-owner thread is replaced by owner-serialised mutation with an operational helper, an eligible sister-and-anchor helper, and axis updates on the owner; the receiver inventory includes commands, cold-ramp observations, and labels, with commands drained first and observations before labels.
- **Read-path limits:** lock-free acquisition refers specifically to swapped snapshots; the surrounding assessment path still uses concurrent maps and mutex-protected pending and signal-cache state, so it makes no whole-call wait-free or no-contention guarantee (`dec:concurrency:snapshot-swap`) (`dec:retention:pending-map`).
- **Configuration:** a complete inventory names every current field, nesting, type, unit, default or explicit absence of a default, admissible range, and operational effect across the Core hierarchy, external Platt and Schur groups, registrations, signal declarations, interaction templates, channel policy, rendering, and Companion settings; declared defaults are checked against their actual implementations and linked to the configuration chapter and construction parameters (`chap:spec:configuration`) (`tab:construction:parameters`).
- **Configuration edge cases:** the inventory explicitly records `command_channel_capacity = None` as a construction refusal, the required signal-schema declaration, derived pending capacity from request rate and label latency, `persistence = None`, persistence's `serde` requirement, and the current empty interaction-template default; any specification/default discrepancy is disclosed rather than silently presented as conformance (`dec:construction:eager-validation`).
- **Public API map:** the map gives actual signatures and reachable import paths for assessment, label guidance through `request_labels`, label submission, report reception, construction, registrations and removals, hibernation, pre-seeding, health, metrics, challenge providers, landscape queries, and rendering; private implementation types and feature-gated test affordances are identified without being advertised as normal host constructors (`rec:surface:host-facing-contracts`) (`rec:challenge:companion-evidence-model`).
- **Documentation map:** reader tasks point by label to the specification (`spec:spec:measurement-judgement-interpreter`), the structural, representation, operational, numerical, and contract outlines (`spec:structural:architectural-commitments`) (`spec:representation:state-representations`) (`spec:operational:assessment-and-learning-cycle`) (`spec:numerics:model-mathematics`) (`spec:contracts:public-contracts`), and the interface map (`app:spec:interface-map`); the upstream consumption audit is identified as an upstream boundary reference, not a host API manual (`rep:assayer:upstream-api-boundaries`).
- **Record index:** the README links the live record register (`reg:assayer:decision-record-register`) and explains the entry points for custody, concurrency, durability, posterior maintenance, construction, surface, derivation, challenge, and harness decisions; its inventory distinguishes the decision set from the constants, migration, and conventions records and is re-counted at the re-cut instead of retaining the obsolete layer work-package census.
- **Development status:** blanket claims that every layer and record is fully implemented and tested are replaced by demonstrated current capabilities and explicit open acceptance conditions; the outdated layer work-package counts and the synchronous reading of the label pipeline are removed, with the conformance campaign and live backlog supplying the evidence (`rep:assayer:code-conformance-campaign`) (`plan:assayer:documentation-migration-campaign`).
- **Snapshot qualification:** the claim that every snapshot field is populated is qualified by publication path: working-copy projection defaults the calibration-buffer summary, label publication patches it, and lifecycle and cold-ramp publication do not; the README does not equate a typed field with a live measurement (`todo:code:project-the-live-calibration-buffer-summary`).
- **Numerical guarantees:** marginalisation's information-preservation claim states its regularisation and correction-discard limits; the numerical studies are linked for the Schur stability factor and floating-point update certificate, rather than converting exact algebra into an unconditional numerical guarantee (`rep:posterior:schur-regularisation-meaning`) (`rep:posterior:rank-one-verification-certificate`) (`dec:posterior:conditioning-guard`).
- **Testing:** the development section gives package-scoped test, all-feature, release, doctest, rustdoc, and no-default-feature commands, distinguishes the sealed `--lib` build from self-dev-dependency feature unification, and explains the shared scenario and coverage inventories through the testing architecture and plan (`rep:assayer:testing-architecture`) (`plan:assayer:test-debt-plan`); counts carry their measured revision, feature set, and ignored-test disposition.
- **Benchmarks and resources:** the development section names the maintained Criterion `performance` target, its workload groups, the `cargo bench -p torrust-assayer` measurement route and compile-only route, and the scheduled report; it links the measurement contract and dense-width resource bounds without presenting historical timing floors as universal latency guarantees (`sec:assayer:performance-harness-measurement-contract`) (`chap:spec:convergence-and-resources`).
- **Features:** the README states that default features are empty, describes the actual optional `serde` closure and persistence consequences, and identifies `test-support` as opt-in test affordances enabled by the self dev-dependency rather than access control that prevents a downstream consumer from enabling it.
- **Follow-ups and limitations:** open work points to the live backlog, including Companion contract coverage and landscape reproduction (`entry:assayer:companion-contract`) (`entry:assayer:landscape-verification`); the deferral register is described as the retained record of implemented or retired deferrals, since it currently has no open entries (`reg:assayer:decision-deferral-register`); calibration warm-up, evidence starvation, pending expiry, fixed input width, and dense-model cost are stated where they affect integration.
- **Licence and label register:** the licence section explicitly matches the inherited `AGPL-3.0-only` declaration and carries no imported linking exception; copyright wording is checked against the package notices, the existing claim-area register is retained as contributor documentation, and new references use the current label calculus rather than retired package-prefix tags (`rec:conventions:cited-forms`).

The re-cut's series presents the final implementation rather than the accumulated repair history. The target order is the following; each body states the resulting behaviour and why it matters in one coherent narrative, with additional detail only where its own diff needs it.

1. `docs(assayer): define the risk and decision contracts` — The specification, layer outlines, decision records, registers, studies, and live plans establish the package's boundaries and numerical obligations without retrospective completion claims.
2. `feat(assayer): implement online risk estimation` — The package manifest and complete compiled module closure, including the clock and gated harness modules and internal tests those modules declare, implement the current assessment, learning, lifecycle, persistence, health, and decision surfaces against those contracts.
3. `test(assayer): exercise host contracts and measured workloads` — Integration suites, their shared support, and the Criterion target make host-visible invariants and completed-operation measurements reproducible without duplicating private construction or waiting logic.
4. `docs(assayer): document a runnable host integration` — The README and its doctest inclusion connect installation, configuration, current signatures, operational limits, and the existing document map in a checked reader path.
5. `build(workspace): register the Assayer package` — The workspace manifest declares the package so the normal workspace commands include it, with no unrelated member or dependency change.
6. `build(deps): lock the Assayer dependency closure` — The lockfile alone records the closure resolved from the re-cut manifests, and its body enumerates every addition, removal, or version movement against the chosen base instead of repeating the old nine-entry claim.
7. `chore(corpus): register the Assayer documentation owner` — Only the owner, reach, policy, assembly, and derived-register wiring absent from the chosen base is added outside the package, with each required row justified by the corpus contract it enables.
8. `ci(assayer): verify the package and publish benchmark evidence` — Only the missing package boundary check, compile-only benchmark gate, and scheduled measurement wiring are added outside the package, with correctness gates distinguished from timing evidence.

The series is cut after its Sentinel and corpus-tooling prerequisites are present in the chosen base, and carries none of their already-owned introduction or repair commits. A target item whose complete diff is already in that base contributes no empty commit. Package additions precede workspace activation; the complete Rust module closure is present together, and the assembled tip passes its gates. Subjects use Conventional Commits with a precise scope, an imperative verb for code changes or a concise indicative merits statement for documentary outcomes, and fit a readable subject line without an ellipsis continuation. Bodies describe the final diff's what and why, contain no attribution, no trailers, and no superseded-cut chronology. The lockfile changes only in its own commit. Every other change outside the package has its own commit and justification; unrelated draft changes stay outside the series. The re-cut recomputes its commit count, per-commit paths, lock delta, and verification record against its actual base.

**Acceptance.** The re-cut pull request's README and commit series satisfy every line above; executable README examples pass as doctests, source-backed names and defaults match the re-cut tree, document labels resolve, the package's full verification chain is green, and the final per-commit path inventory proves the package, workspace, lock, corpus, and CI boundaries. Any remaining mismatch is a named open condition rather than a completion claim. No independent harness dependency-table row is required: this entry owns introduction evidence, not a harness capability.

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

The shared testing-harness campaign (´sec:assayer:campaign-testing-harness´) no longer has a construction order to follow. The dependency chain the concept retained has run — barriers before local-wait retirement and shared playback, scenario time and complete construction before probes, the persistence fork after those shared surfaces, and scenario unification through every touched test — and the performance partition ran with it (´sec:assayer:testing-harness-concept-migration´) (´sec:assayer:performance-harness-partition´). What the section now carries is the demand the finished harness is still asked for, grouped by mechanism and ordered by nothing: the owned readings, guarded fixtures, fork extensions, paired playback entry point, and support-tree additions are independent of each other and of the two restore defects beside them, so they may be taken in any order or in parallel (`obs:assayer:testing-architecture-intent-demand`). Four of the thirteen readings close only when a production prerequisite their own plans own is supplied, which delays those readings' closure rather than their start.

Finish active source work with source policy and retirement (`sec:assayer:campaign-source-retirement`). Its inventory prerequisite is already met — the checked policy and burn family hold the legacy implementation's unlabelled remainder at zero — so retirement begins against a register rather than against a search. The declared performance route (`sec:assayer:campaign-performance-route`) is decided and its implementation stays in the shared testing-harness campaign, where its baseline and schedule measure the retained scenario rather than an intermediate one. The condition-bound entries in the observability (`sec:assayer:campaign-observability`) and claim-area (`sec:assayer:campaign-claim-areas`) groups remain outside the active order until their stated revisit conditions are met.

**Ruling queue.** These entries need a recorded decision before their choice-dependent implementation or corpus edit can start:

- (`entry:assayer:landscape-verification`) — every underdetermined reading and the surfaced forms of crossover covariance and regime-width uncertainty.
- (`entry:assayer:harness-registration-lift-ruling`) — which of registration-time widening, the later-label counterfactual, and a compound registration-plus-first-report event the promise keeps. No harness surface waits on the answer; the promise and one witness do.

The maintained performance route is decided as Criterion benchmarks on the shared scenario (`dec:harness:performance-benchmarks`); its fixture, tooling, target, and CI work are entered below and require no further ruling (`sec:assayer:performance-harness-partition`). The parked identity tunability, compile-fail runner, upstream API, and claim-area entries are not an active ruling queue: their existing revisit conditions must fire first.

**Immediately dispatchable bounded lanes.** Within the source-retirement group, the retirement of legacy code and stale scaffolding is a bounded entry lane that can start now against the installed inventory, and the lint-suppression audit is a second. The testing-harness campaign supplies five more, each bounded by the module it lands in: the owned readings (`entry:assayer:harness-demanded-readings`), the guarded fixtures and populations (`entry:assayer:harness-demanded-fixtures`), the persistence-fork extensions (`entry:assayer:harness-fork-extensions`), the paired playback entry point (`entry:assayer:harness-paired-playback`), and the support-tree additions (`entry:assayer:harness-support-tree-additions`). None of those seven lanes requires a decision to begin. The notice-disposition entry builds its own recognizer before it can enumerate what it owns; the compile-fail runner waits on the dependency ruling that parks it (`entry:assayer:harness-compile-fail-runner`), and the convention checker waits on the linter policy it defers to and blocks nothing while it waits (`entry:assayer:harness-convention-checker`).

## Shared testing-harness campaign · `sec:assayer:campaign-testing-harness`

These entries stay together because they turn one shared testing contract into executable work while preserving a distinct outcome, acceptance condition, and live pin for every independently closable obligation; their implementation order, dependency force, parallel sets, and collision points follow the harness implementation route (´plan:assayer:harness-implementation´). The contract itself has landed, so what the section carries forward is what the finished harness is still asked for: the mechanism groups the demand census leaves open (`obs:assayer:testing-architecture-intent-demand`), the convention checker the retirements left ownerless, and the one ruling no mechanism settles, beside the restore defects the persistence fork exposed. Those entries keep the section's rule of one closable obligation per entry, but not its construction order, which has run.

**Entry (Closed queue-barrier set)** · `entry:assayer:harness-closed-barriers`

**Outcome class: ENGINEERING.**

**DONE.** Every asynchronous queue reachable by a test now has one documented, queue-specific barrier with explicit coverage and exclusion. `MaintenanceHarness::flush_identity_maintenance` crosses the specialised seam's command and observation queues through the loop's checkpoint acknowledgement; `World::flush_identity_maintenance` composes that acknowledgement with model-owner publication; `World::flush_labels` crosses owner commands and labels; and `World::flush_observations` crosses the cold-ramp queue. The remaining queues now announce their own completion: `World::flush_ledger_gc` receives the completed sweep outcome, `World::flush_checkpoint_scheduler` receives acknowledgement after the trigger has entered the owner queue while excluding downstream execution, and `World::drain_health_events` orders every model-owner event producer before draining the bounded receiver. Focused witnesses prove the Ledger state is read immediately after its cycle acknowledgement, the checkpoint trigger returns while the downstream owner is parked, and label, observation, and lifecycle health events all precede the drain. The maintenance inventory is at a fixpoint with no reachable queue uncovered; crate-local polls and specialised side-harness waits duplicate the named queue meanings and remain migration work rather than gaps in the set. The live pin remains (´entry:assayer:testing-harness-concept-barrier-set´), and the result implements the declared barrier contract (´cor:concurrency:harness-barriers´).

**Entry (Scenario time verbs)** · `entry:assayer:harness-scenario-time`

**Outcome class: ENGINEERING.**

**DONE.** `World::advance` and `World::travel_to` move the persistent and intra-process readings together while their types and lack of conversion preserve the domain separation: the corollary requires control without conversion, while the concept fixes one verb pair and no engine behaviour needs independently skewed notions of the present. Both verbs return only after identity maintenance and its accepted lifecycle publication settle the monotonic work made due and one Ledger-GC cycle settles the persistent work made due; cold-ramp observations, checkpoint scheduling and health-event consumption remain explicitly outside a clock movement. Forward movement through both typed readings is witnessed by (´test:crate:scenario-advance-moves-both-clock-domains-together´) and (´test:crate:scenario-travel-targets-both-clock-domains-together´), completion through a barrier's own effect by (´test:crate:scenario-advance-completes-the-ledger-gc-cycle-it-makes-due´), and diagnostic refusal before either domain moves by (´test:crate:scenario-travel-refuses-a-backward-target-with-both-instants´). The completed live pin is (´entry:assayer:testing-harness-concept-time-verbs´), and the result implements scenario control of the separated domains (´cor:clock:harness-control´).

**Entry (Complete host declaration builder)** · `entry:assayer:harness-complete-builder`

**Outcome class: ENGINEERING.**

**DONE.** The scenario builder requires an explicit deterministic seed, forwards interaction templates into engine construction, carries a declared expected runtime layout and refuses mismatches, while the gated identity verb installs a declared competitive-cell set through observations and verifies the published result. Missing-seed and invalid-template construction refusals are witnessed by (´test:crate:world-builder-without-seed-is-rejected´) and (´test:crate:world-builder-forwards-invalid-interaction-template´); successful and unobservable cell declarations by (´test:crate:register-identity-with-cells-observes-the-declared-set´) and (´test:crate:register-identity-with-cells-rejects-an-unobservable-set´); and the guarded reference width by (´test:crate:public-scenario-reproduces-reference-layout-p638´). The completed live pin is (´entry:assayer:testing-harness-concept-complete-builder´), and the result fulfils the harness's construction obligation (´cor:construction:harness-declarations´).

**Entry (Probe contract and first projections)** · `entry:assayer:harness-probe-contract`

**Outcome class: ENGINEERING.**

**DONE.** The private `testing::probes` module defines `testing::PublishedModelBlock`, `testing::PublishedSlotMoments`, and `testing::PendingEntryView` as plain owned projections flattened through the gated roster. `World::published_model_block` and `World::published_slot_moments` cross `World::flush_labels` before exactly one published load; `World::pending_entry_view` names its lack of an asynchronous queue dependency and performs exactly one non-consuming buffer lookup. Focused witnesses establish deterministic barrier freshness (´test:crate:published-model-block-crosses-publication-barrier´), one-version coherence (´test:crate:published-model-block-is-from-one-version´), independence from later publication (´test:crate:published-model-block-is-independent-of-later-publication´), scenario-clock provenance (´test:crate:published-model-block-uses-scenario-time´), a complete owned standardisation slot (´test:crate:published-slot-moments-cross-publication-as-one-owned-slot´), and request-scoped retention precision without consumption (´test:crate:pending-entry-view-is-non-consuming-and-keeps-storage-width´). Scalar, `Vec`, `HashMap`, and immutable-slice shapes make engine borrows, guards, shared owners, and mutation routes unrepresentable, while the featureless library leaves the probe module and its roster exports uncompiled. The completed live pin is (´entry:assayer:testing-harness-concept-probe-contract´); the precision matrix and wider durable state remain excluded (´cav:retention:probe-boundary´) (´dec:retention:precision-excluded´), and the gated surface remains sealed from hosts (´cor:surface:test-support-opacity´).

**Entry (Shared tape runner)** · `entry:assayer:harness-tape-runner`

**Outcome class: ENGINEERING.**

**DONE.** `testing::playback` now interprets caller-owned implementations of `testing::PlaybackRow` under an ordered `testing::PlaybackBarrierPolicy`, invokes self-validating checkpoints after every selected barrier, advances `testing::PlaybackProgress` once at the complete boundary, and materialises only the caller's non-zero bounded batch while settling each row separately. The compile-shape and ownership recognizer establish typed rows (´test:crate:shared-runner-accepts-unrelated-subject-row-types´); the held-barrier and real label-publication witnesses establish settlement before comparison (´test:unit:held-barrier-completes-before-its-checkpoint´) (´test:crate:selected-label-barrier-completes-before-checkpoint´); deliberately skipped, duplicated, early and later-boundary cases each establish their own checkpoint refusal (´test:unit:skipped-checkpoint-is-rejected´) (´test:unit:duplicated-checkpoint-is-rejected´) (´test:unit:early-checkpoint-is-rejected´) (´test:unit:later-boundary-checkpoint-is-rejected´); a coordination-held row establishes the first-incomplete progress diagnostic without time (´test:unit:stalled-progress-names-the-first-incomplete-row´); and the mid-batch publication witness establishes cost-only batching (´test:unit:batch-size-preserves-mid-batch-publication-boundaries´). The enrichment grid's qualifying alternating label history now supplies its subject-owned rows through the runner and remains covered by the pre-conditioned grid witness (´test:integration:pre-conditioned-three-channel-batch´). The recorded multi-channel budget and breach procedure remain the sole performance evidence, and no playback test or gate reads elapsed time (´sec:assayer:testing-architecture-multi-channel-runtime-budget´) (´dec:harness:declarative-playback´) (´entry:assayer:testing-harness-concept-tape´). The isolated persistence fork is therefore unblocked (´entry:assayer:harness-persistence-fork´) (´cor:durability:harness-fork´).

- **Typed rows.** Every long stimulus is an ordered sequence of subject-owned typed rows interpreted by the shared runner; a subject-specific row shape owned by the runner or a handwritten long playback loop is a violation.
- **Barrier policy.** At every boundary declared settled, the runner guarantees that all barriers required by the selected policy have completed before any checkpoint or settled comparison runs; a focused held-barrier witness must fail if either can run early and must identify the blocked row, without depending on the barrier set's internal shape.
- **Checkpoints.** Every declared checkpoint runs exactly once at its declared row boundary, after the required completion for that row, and observes all work through that boundary; a skipped, duplicated, early, or later-boundary callback is a failure.
- **Progress.** Progress advances exactly once after each completed row and a deliberately stalled witness identifies the first incomplete row; progress remains liveness evidence, and no test or gate turns its timing or the runtime budget into a latency assertion (´cav:harness:progress-not-latency´).
- **Cost-only batches.** Changing a bounded batch size may change dispatch cost but must preserve row order, per-request publication semantics, checkpoint observations, and progress boundaries; a focused mid-batch state-change witness must fail if a batch is treated as one shared view (´cor:ordering:tape-batches´).

**Entry (Independent oracle tier)** · `entry:assayer:harness-oracle-tier`

**Outcome class: ENGINEERING.**

**DONE.** The gated `testing::oracles` tier declares specification-formula provenance for `testing::pairwise_rank`, `testing::regularised_schur_complement`, `testing::decay_recurrence`, and `testing::dimension_width`; the four routes respectively use class-gated positive-negative pair enumeration, pivoted dense Gauss–Jordan elimination, stepwise clock composition, and direct block arithmetic instead of the production sort and rank-sum, Cholesky half-solve, power, and mutable-cursor routes. Focused witnesses separate a dropped tie credit, an unregularised inverse, a single decay application, and an interaction width read from the signal block, while existing discrimination, marginalisation, hibernation, Bayesian composition, and layout callers now consume the shared tier. The live pin is (´entry:assayer:testing-harness-concept-oracles´), and the result implements the shared oracle contract (´dec:harness:oracle-tier´).

**Entry (Guarded fixture default)** · `entry:assayer:harness-guarded-fixtures`

**Outcome class: ENGINEERING.**

**DONE.** `World::trained_state` retains its guarded shared score-verified state. `WorldBuilder::build` when it carries an expected runtime layout, `World::cold`, `scenario`, `scenario_with`, and `scenario_with_config` now derive a complete construction declaration before consuming their inputs, read back the instance identifier, generator state, persistent clock, full channel policies, every runtime-layout block, publication and layout generations, lifecycle populations, ramp phase and accepted count, and label and assessment counts at the fixture boundary, and refuse with that measured baseline beside the declaration. The builder guard establishes that the measured initial layout can grow into its completed-layout declaration, while `World::runtime_layout` retains the exact post-setup comparison rather than being duplicated; each guarded fixture has a focused refusal witness that invalidates one declared fact and checks the complete diagnostic. The generic lifecycle verbs `World::register_sentinel`, `World::deregister_sentinel`, `World::receive_report`, `World::register_axis`, `World::deregister_axis`, `World::register_identity`, `World::deregister_identity`, and `World::register_sentinels` are OUT: they are command or ingestion primitives whose declarations are successful acceptance and, where applicable, publication ordering, already carried by their result and named barrier, while the consuming fixture owns any stronger domain postcondition. The learning verbs `World::cycle_on`, `World::cycle_default`, `World::cycle_benign`, `World::cycle_adverse`, and `cycle_request` are OUT: they run one assessment-label transaction and return its pre-update reckoning without declaring a reusable learned post-state, so their flush is sequencing rather than a fixture baseline. The golden-report helpers `refresh_golden_report` and `attach_golden_reporting_sentinel` are OUT: they validate the synchronous ingestion acknowledgement fields their stimulus declares, but promise no reusable model, graph, or calibration post-state for a second guard to measure. `World::settle_cold_ramp_with` is OUT: it is a queue-driving barrier rather than a constructor, its normal non-empty path already reads the phase and accepted count after each drain and refuses stalled progress with both readings, and an empty driver declares no work and therefore no ramp state to establish. Pure payload builders in `testing::reports`, `testing::registrations`, `LabelSpec`, `PreSeedSpec`, and `DegradationSpec` remain outside the population because they create values rather than subject state. The maintenance population is reconciled to the stable module before its ruling: its earlier nine names predated the barrier work, and the marker-send `MaintenanceHarness::checkpoint` plus the sleeping `MaintenanceHarness::flush_loop` were replaced together by the acknowledged `MaintenanceHarness::flush_identity_maintenance`, so nine becomes the current eight with one item carrying both old meanings rather than two one-for-one renames. Three maintenance constructors are guarded. `maintenance::setup_dimension` derives its complete dimension declaration independently, reads back the identifier, name, descriptions, domain width, cutoff, encoder probes, competitive and tracked cells, published importance, dropped and active-indicator counts, graph snapshot, and both views of the observation queue including capacity, occupancy, and overflow, and refuses a mismatch with the complete measurement. `MaintenanceHarness::spawn` and `MaintenanceHarness::spawn_with_capacity` read back thread-handle presence, liveness and name plus the maintenance-command and model-owner queue capacities and occupancies, including the caller-declared model-owner capacity, before returning. Each has a focused refusal witness. Five items are OUT. `MaintenanceHarness::register_dimension` and `MaintenanceHarness::register_dimension_with_config` are acknowledged command primitives: the exact configuration, identifier, depth cutoff, observation receiver, and infrastructure cross into the loop before acknowledgement, while a consuming fixture owns any stronger reusable postcondition. `MaintenanceHarness::destroy_dimension` is a fire-and-forget command primitive rather than a state constructor; callers requiring ordering cross `MaintenanceHarness::flush_identity_maintenance`, then their oracle decides what release means. `MaintenanceHarness::flush_identity_maintenance` is that barrier, whose acknowledgement is already its completion declaration. `MaintenanceHarness::force_decay` is an acknowledged pure graph-value transformation; a baseline would be the entire graph distribution before and after attenuation, which is the consuming result oracle rather than a reusable fixture precondition. No unguarded state-establishing fixture remains anywhere in the harness, so the entry closes. The live pin is (´entry:assayer:testing-harness-concept-guarded-fixtures´), and the guarded-versus-oracle distinction implements separate validation (´dec:harness:separate-validation´).

**Table (The trained-state fixture figures retain the public convergence population)** · `tab:assayer:harness-trained-state-fixture-figures`

| Figure | Value | Why this figure |
| --- | --- | --- |
| Training population | Five hundred alternating cycles, split equally between benign and adverse labels | Retains the public convergence population so the reusable setup and its standing witness do not teach different deployments (´test:integration:scalar-signal-population-split-converges-directionally´). |
| Training endpoints | Benign score `0.08` without verification; adverse score `0.92` with verification | Retains the two score-verified classes whose learned direction the public witness already establishes (´test:integration:scalar-signal-population-split-converges-directionally´). |
| Held-out rank populations | Twenty examples per class; benign scores `0.02 + 0.012i` and adverse scores `0.75 + 0.012i` | Twenty is the configured minimum per class for rank discrimination, while the disjoint score ranges retain the public witness's independently ranked population (´tab:config:monitoring´) (´test:integration:scalar-signal-population-split-converges-directionally´). |
| Strict trained-state floors | Held-out endpoint gap above `0.10`; tie-corrected pairwise rank above `0.65` | Retains the two jointly necessary readings from the public witness: a post-training endpoint difference and materially above-chance ordering across populations (´test:integration:scalar-signal-population-split-converges-directionally´) (´tab:monitoring:interpretation´). |

**Entry (Isolated persistence fork)** · `entry:assayer:harness-persistence-fork`

**Outcome class: ENGINEERING.**

**DONE.** `World::fork_persistence` quiesces the real instance, captures its checkpoint and journal into separately owned storage, reconstructs the restored arm under the same injected clock, and exposes both durable roots. The common-suffix witness compares every checkpoint field through a schema-exhaustive normalized projection (´test:crate:persistence-fork-common-suffix´); elapsed restoration applies decay once and rejects a deliberately double-decayed projection (´test:crate:persistence-fork-decay-once´); journal bytes establish isolation and root lifetime (´test:crate:persistence-fork-journals-are-isolated´); and a perturbed field is named with both values (´test:crate:persistence-fork-comparator-names-field´). The binding-weight witness closes the elapsed-leverage ordering gap (´test:unit:elapsed-leverage-selects-the-same-binding-weight´), with identity and refusal boundaries retained (´test:unit:temporal-leverage-identity-is-bit-exact´) (´test:unit:invalid-temporal-leverage-refuses-before-mutation´). The clock, whole-state, continued-learning, concordance and constructor-clock restart witnesses now use the fork while retaining their existing assertions. This supplies the concept's capability (´entry:assayer:testing-harness-concept-persistence-fork´) under its corollary (´cor:durability:harness-fork´), consuming the supplied barriers, probe contract and playback runner (´entry:assayer:harness-closed-barriers´) (´entry:assayer:harness-probe-contract´) (´entry:assayer:harness-tape-runner´).

**Entry (Restore ages the Ledger arrival load with its timestamp)** · `entry:assayer:restore-ledger-arrival-load-decay`

**Outcome class: ENGINEERING.**

**OPEN.** The Ledger's write-time decay scales `eligible_arrival_load` with the averages before advancing `last_updated` (``src/ledger/entry.rs:227``), and its pure load reading uses that same timestamp (``src/ledger/entry.rs:299``). Recovery scales the averages and advances the timestamp without scaling the arrival load (``src/persistence/recovery.rs:192``), losing that elapsed interval for the load. Establish an isolated same-clock witness with a nonzero arrival load and a nonzero restore interval, then make the restored load and its next read retain exactly the elapsed aging required by (´def:ledger:time-decay´) and (´dec:durability:decay-once´). The zero-gap control must remain exact; the witness must distinguish an omitted factor from a second application.

**Entry (Restore preserves the age of block-legibility evidence)** · `entry:assayer:restore-block-legibility-decay`

**Outcome class: ENGINEERING.**

**OPEN.** The label path passes the operational model's combined temporal and label factor into block-legibility evidence (``src/owner/label_path.rs:822``), but recovery restores owner state unchanged while applying bulk elapsed decay only to models, Ledger averages and identity outcomes (``src/persistence/recovery.rs:172``). A restored label therefore supplies only its post-construction temporal factor to sums whose downtime has not been materialized. Establish the intended restore treatment under (´def:monitoring:slot-association´) and (´def:monitoring:slot-contribution´), and witness a nonempty block's persisted evidence across a nonzero same-clock fork interval and common suffix (´cor:durability:harness-fork´). Retain the distinct label-only standardisation and class-rate decisions (´dec:vector:no-time-decay´) (´dec:weighting:no-time-decay´); do not assign a blanket rate to owner state.

**Entry (Unify the Assayer scenario model)** · `entry:assayer:harness-scenario-unification`

**Outcome class: RETIREMENT.**

**DONE.** Fixture-side construction in crate, integration, inline, and benchmark coverage now enters through `testing::World` or its `testing::Scenario` alias; common configuration, entity, report, derivation, lifecycle-publication, and completion meanings have one owner. `World::publish_lifecycle` is the one added verb: it submits an empty lifecycle publication and crosses the label-publication barrier, while its contract excludes report, identity-maintenance, checkpoint-scheduler, and durability completion. Constructor-contract tests retain direct `AssayerBuilder` calls only where builder state or an exact `BuildError` is the subject and no world can yet exist. The maintenance seam remains local, and the multi-channel support tree retains its grid, matrices, reward perturbations, row shapes, and paired-world ceremony while using `WorldBuilder::signal_schema` directly (´dec:harness:specialised-side-harnesses´). The source recognizer finds no retired declaration or fixture-side raw engine constructor, and the generated roster, per-file indexes, and folder matrices are clean; the live pin is (´entry:assayer:testing-harness-concept-merge-scenario-models´), so the single-scenario contract is descriptive (´cav:harness:incremental-adoption´).

| Item | Callers at the moved base | Disposition | Shared replacement or retained reason |
| --- | --- | --- | --- |
| Crate configuration constructors | `crate::tests::builder`, `crate::tests::harness_builder`, `crate::tests::performance_fixtures`, `crate::tests::snapshot`, `crate::tests::api_surfaces`, `crate::tests::health_infrastructure`, `crate::tests::ledger`, `crate::tests::lifecycle_api`, `crate::tests::lifecycle_integration`, `crate::tests::owner`, `crate::tests::guidance`, and `crate::tests::persistence` | Retired | Explicit `AssayerConfig` values enter `WorldBuilder`; constructor-contract cases retain the production builder as their subject. |
| Model and persistence configuration constructors | `crate::tests::builder`, `crate::tests::lifecycle_integration`, `crate::tests::owner`, and `crate::tests::persistence` | Retired | Explicit `ModelConfig` and `PersistenceConfig` values are carried by `WorldBuilder`. |
| Complete engine constructors used as fixtures | Crate lifecycle, owner, guidance, persistence, API, health, ledger, snapshot, and performance-fixture subjects, plus integration scenarios | Retired | `World::builder`, `scenario_with`, or `Scenario` owns the engine; exact production-builder contract tests remain direct because construction is their asserted subject. |
| Entity shorthands | `crate::tests::builder`, `crate::tests::persistence`, `tests::scenarios`, and `tests::signal` | Retired | `World::entity` supplies the canonical deterministic mapping; direct `EntityKey` constructor tests remain value-type tests rather than scenario fixtures. |
| Configurable report constructor | `crate::tests::builder` | Retired | `minimal_report` and `golden_report_4cell` already carry the asserted root and competitive-cell shapes; metrics report rows remain specialised mapper inputs. |
| Assessment-plus-derivation compatibility trait | `crate::tests::lifecycle_integration` and `crate::tests::persistence` | Retired | `World::derive_for_requests` owns assessment and derivation sequencing. |
| Local lifecycle publication and checkpoint waits | `crate::tests::lifecycle_integration` and `crate::tests::persistence` | Retired | `World::publish_lifecycle`, `World::flush_labels`, and `World::flush_identity_maintenance` name the queue coverage; the polling-retirement prerequisite had already removed version polls and sleeps. |
| `ScenarioBuilderExt` | Multi-channel enrichment setup | Retired | `WorldBuilder::signal_schema` is called directly. |
| Maintenance and multi-channel side harnesses | Maintenance barrier tests and the multi-channel enrichment suite | Retained | Their thread seam, grid, matrices, rewards, row shapes, and paired-world ceremony are the declared subject-specific boundary (´dec:harness:specialised-side-harnesses´). |

**Entry (Retire polling, sleeps, and local waits)** · `entry:assayer:harness-polling-retirement`

**Outcome class: RETIREMENT.**

**DONE.** The base census searched `thread::sleep|wait_for_version|wait_for_|spin_loop|yield_now` through crate tests, integration tests, and test support outside `testing::liveness` and found forty-eight syntactic sites in eleven source modules. The two polling helpers and every completion guess have retired behind the queue-specific barriers below; three elapsed-time exit sites and two additional condition-driven reader loops found by the widened audit are also classified below. Six raw matches remain at the tip, each beside its site-local ruling: two sleeps measure monotonic expiry, one sleep separates real-clock timestamp samples, one sleep paces a fixed concurrency workload, and two yields widen fixed concurrency interleavings without awaiting state. The repository linter has no rule for authored polling or waits, and no rule or exemption was added here; that ownerless enforcement gap remains the finding at (´obs:assayer:harness-implementation-convention-enforcement´). With the semantic search clean and every base and widened site accounted for, the live pin remains (´entry:assayer:testing-harness-concept-retire-polling´), and the result makes the no-wait decision descriptive (´dec:harness:no-ad-hoc-waits´).

**Table (Polling-retirement census)** · `tab:assayer:harness-polling-retirement-census`

| Source surface | Base hits | Disposition |
| --- | ---: | --- |
| `tests::health_infrastructure` | Sixteen | The label-count helper and its sleeping loop were deleted; thirteen callers now cross `World::flush_labels`, and the event caller crosses `World::drain_health_events`. |
| `tests::helpers` | Two | The version helper and its sleeping loop were deleted. |
| `tests::owner` | Eight | The helper import and five version calls now use `Assayer::flush_label_channel`; the shutdown poll now joins its thread, and the label-depth poll now crosses the label-publication barrier. |
| `tests::persistence` | Eight | The helper import and six version calls now cross `Assayer::flush_label_channel`, `World::flush_labels`, or the composed `World::flush_identity_maintenance` according to the queue under assertion; the post-restore counter poll now crosses the label-publication barrier. |
| `tests::lifecycle_integration` | Seven | Four observation, lifecycle, and shutdown sleeps now use `Assayer::flush_observation_channel`, `World::shut_down_model_owner_for_test`, or exact ownership completion; both cold-ramp yields now cross the observation barrier and prove queue completion or monotonic progress; one fixed-workload concurrency sleep remains ruled as stimulus pacing. |
| `tests::identity_maintenance` | One | The yield remains ruled as interleaving stimulus inside a fixed-count reader workload. |
| `tests::label_pipeline` | One | The sleep remains ruled as the real-clock interval whose before-and-after timestamps are measured. |
| `tests::ledger` | One | The yield remains ruled as interleaving stimulus inside the rehash workload. |
| `tests::pending` | Two | Both sleeps remain ruled as the real monotonic expiry horizon the eviction tests measure. |
| `derive_purity` | One | The pacing sleep now crosses `World::flush_observations`, preserving one published cold-ramp step per sample. |
| `synchronisation_drift` | One | The copied liveness implementation, including its heartbeat sleep, retired in favour of the shared `testing::liveness` exports. |
| Widened elapsed-exit audit | Three | One synchronisation-drift setting wall and two precision-definiteness exit conditions retired; both finite instruments now complete their declared stimuli while the shared liveness watcher owns stall detection. |
| Widened condition-loop audit | Two | The snapshot and model-owner reader loops remain ruled as fixed concurrency workloads whose writers supply completion; the Ledger writer loop is the same retained stimulus site already counted by its yield above. |

**Entry (Retire callerless helpers and stale staging notes)** · `entry:assayer:harness-stale-helper-retirement`

**Outcome class: RETIREMENT.**

**DONE.** The moved-base census retained only candidate items with live inline, crate, integration, benchmark, or harness callers and removed every residual callerless entry point with its supporting state. Module headers now describe the declared construction, scenario-time, barrier, fixture, probe, playback, oracle, sweep, report, and assertion surfaces; the implementation and architecture rosters agree with those declarations and exports. Commit `edd8582d9` had already replaced the synchronisation-drift target's private liveness copy with common imports, discharging that observation as part of the same architecture reconciliation (´obs:assayer:testing-architecture-liveness-copy´). The source and generated-documentation recognizer closes the live pin (´entry:assayer:testing-harness-concept-retire-callerless´) and discharges the unreported-decay and staging-note findings (´obs:assayer:testing-architecture-unreported-decay´) (´obs:assayer:testing-architecture-staging-notes´).

**Entry (Split the convergence subject by topic)** · `entry:assayer:harness-convergence-split`

**Outcome class: ENGINEERING.**

**DONE.** The convergence subject now has the four decision-named integration targets: `learning_convergence` retains the standing witness, while `lifecycle_identity_convergence`, `posterior_convergence`, and `calibration_discrimination_convergence` carry honest empty indexes until their assigned witnesses land. Every affected intent plan names its assigned topic target, the package test roster discovers all four targets, and the catalogue projections are at a fixpoint with no stale index or matrix. No topic target defines a local fixture, setup, support, or ceremony item, so every acceptance clause holds. The live pin remains (´dec:harness:convergence-split´).

**Entry (Benchmark projection coverage)** · `entry:assayer:performance-benchmark-projections`

**Outcome class: ENGINEERING.**

**DONE.** The test census and classifier now carry the benchmark root and bench area, every registered Criterion function derives its standard-place label, and the in-file index, claim coverage and per-folder matrix project from those assets. The focused linter witnesses are ``test:unit:discovers-shorthand-registered-benchmarks-once``, ``test:unit:discovers-configured-and-qualified-benchmarks``, ``test:unit:leaves-unregistered-benchmark-helpers-out-of-the-census``, ``test:unit:ignores-registration-syntax-outside-item-position``, ``test:unit:censuses-a-package-without-a-benchmark-root-as-before``, ``test:unit:classifies-the-benchmark-root-as-bench``, ``test:unit:derives-a-registered-benchmarks-label``, ``test:unit:accepts-a-registered-benchmark-at-the-standard-place``, ``test:unit:orders-bench-after-the-harness-areas``, ``test:crate:projects-a-benchmark-file-and-folder``, ``test:crate:detects-a-stale-benchmark-index-and-matrix`` and ``test:crate:counts-a-benchmark-claim-as-coverage``; they are displayed rather than imported because the Assayer owner has no reach into the linter owner. The workspace benchmark projections now settle to a fixpoint, and the live pins remain (`plan:assayer:performance-harness`) and (`dec:harness:performance-benchmarks`).

**Entry (Performance fixtures and fakes)** · `entry:assayer:performance-harness-fixtures`

**Outcome class: ENGINEERING.**

**DONE.** The performance harness owns one six-row case table, fixed-clock and no-persistence guards, structurally acknowledged synthetic reports, declared label and pre-seed populations, guarded reference layouts and removal, and a test-support label-region recorder reached through `World`. The table, layouts and population mixes are witnessed by (´test:crate:performance-case-table-declares-the-six-plan-rows-once´), (´test:crate:every-performance-case-returns-its-declared-runtime-layout´) and (´test:crate:seeded-populations-report-every-declared-count-and-mix´); invalid fixture preconditions by (´test:crate:fixed-clock-origin-guard-rejects-a-different-origin´), (´test:crate:no-persistence-guard-rejects-configured-durable-paths´), (´test:crate:synthetic-report-guard-rejects-an-invalid-acknowledgement´), (´test:crate:pre-seed-completion-guard-rejects-a-partial-result´) and (´test:crate:reference-removal-guard-rejects-trivial-training´); the completed recorder by (´test:crate:label-update-recorder-returns-only-a-completed-dense-publication-duration´); and the real-engine boundary by (´test:crate:performance-fixtures-do-not-import-private-engine-constructors´). The completed prerequisite is (´entry:assayer:harness-complete-builder´), and the live pins remain (`plan:assayer:performance-harness`) and (`dec:harness:performance-benchmarks`).

**Entry (Criterion target and stopwatch retirement)** · `entry:assayer:performance-criterion-target`

**Outcome class: ENGINEERING.**

**DONE.** The single `performance` Criterion target carries the assessment, label-publication, construction/shutdown, pre-seed, health and reference-marginalisation groups over the guarded shared fixtures, and its maintained evidence is (´test:bench:bench-assessment´), (´test:bench:bench-label-publication´), (´test:bench:bench-construction-shutdown´), (´test:bench:bench-pre-seed´), (´test:bench:bench-health-summary´), (´test:bench:bench-full-health-report´) and (´test:bench:bench-reference-marginalisation´). Those identifiers now own the seven performance claims and their documented comparison floors, the generated indexes and matrices resolve them, and the ignored stopwatch witnesses and their private setup have retired. The completed prerequisites are (´entry:assayer:performance-benchmark-projections´) and (´entry:assayer:performance-harness-fixtures´); the live pins remain (`plan:assayer:performance-harness`) and (`dec:harness:performance-benchmarks`).

**Entry (Scheduled performance route)** · `entry:assayer:performance-ci-route`

**Outcome class: ENGINEERING.**

**DONE.** The scheduled and manually dispatched benchmark route runs the Assayer Criterion target under the optimised profile on the stable and nightly toolchains, isolates their target directories, records the runner, fixture cases, comparison settings and commit beside each complete report, and retains the artifacts for a bounded release-comparison window without deriving a verdict from a wall-clock floor. The package lint names the maintained measurement command and compiles its target without executing the measurements, while the existing deployment route continues to compile every bench target. A successful manual artifact is the one acceptance item only an upstream GitHub run can prove and remains pending the next upstream cut. The completed prerequisite is (´entry:assayer:performance-criterion-target´); the live pins remain (`plan:assayer:performance-harness`) and (`dec:harness:performance-benchmarks`).

**Entry (Measured enrichment-grid runtime budget)** · `entry:assayer:process-grid-runtime`

**Outcome class: VERIFICATION.**

**DONE.** The maintained command, pooled twelve-sample baseline distribution, range-derived tolerance and overloaded rerun rule are recorded with the suite (´sec:assayer:testing-architecture-multi-channel-runtime-budget´). The command selects the complete `multi_channel` target; the distribution records every definitive wall and boundary load on the stated machine; the 221 ms tolerance derives directly from the observed extremes; and the rerun rule requires two comparable serialized repeats before a breach becomes a finding. The budget is informational and no test or gate reads elapsed time. This evidence satisfies the revisit condition for the parked tape runner (´entry:assayer:harness-tape-runner´).

**Entry (Remaining production persistent-clock sites)** · `entry:assayer:harness-stage-clock`

**Outcome class: ENGINEERING.**

**DONE.** The production census found the direct `PersistentTimestamp::now` constructor and four callers: `CellOutcomeState::new_neutral`, `LedgerEntry::new_neutral`, `JournalEntry::unsequenced`, and `ChallengeEffectivenessState::new`. Every timestamp-bearing constructor now takes one explicit `PersistentTimestamp`, threaded from the nearest holder of `Clock`; their timestamp-bearing defaults are gone, and `PersistentTimestamp::now` is deleted, so the only production wall-clock read in this domain is the injected `SystemClock::now` boundary. A neutral Ledger root created after scenario travel carries that travelled reading (´test:crate:neutral-ledger-state-takes-travelled-scenario-time´), while a production checkpoint restored after a later scenario advance decays from the two scenario readings alone (´test:crate:restore-measures-downtime-on-the-injected-clock´). This closes the production half of (´dec:clock:two-domains´); the scenario verbs and their barriers remain separately completed at (´entry:assayer:harness-scenario-time´), and raw monotonic reads remain assigned to (´entry:assayer:wl-staleness-raw-clock´).

**Entry (Seeded invariant packs)** · `entry:assayer:harness-property-packs`

**Outcome class: ENGINEERING.**

**DONE.** The framework-free `testing::run_seeded_sweep` gives every rejected case its declared seed, zero-based case index, complete drawn input, and witness reason (`dec:harness:seeded-sweeps`). The census below is the population this entry adjudicated: forty-three labels are sweep-shaped and witnessed, none is blocked, and sixteen are not sweep-shaped. The eight formerly blocked rows were re-censused against the landed complete builder, closed barriers, scenario time, probe contract, and independent oracle tier; each has the per-draw observation route named below. A claim enters the census when its minted statement itself quantifies over a drawable numeric, combinatorial, or state-transition space; a finite reference fixture, a statement about one configured constant, or a statistical aggregate with no per-draw predicate does not become a property pack merely because its existing witness repeats work.

| Label | Census disposition | Reason or dependency |
|---|---|---|
| (`inv:guarantee:assess-only`) | in this lane | Draw complete worlds and requests; `World::pending_entry_view`, `World::published_model_block`, `World::published_slot_moments`, public health readback and `World::advance` expose the permitted writes, unchanged taught state and scenario-time Ledger read. |
| (`inv:guarantee:derivation-purity`) | in this lane | Draw complete derivation arguments and compare two independently invoked results bit for bit. |
| (`inv:guarantee:evidence-authority`) | in this lane | Draw requests after both cold-ramp and late-Sentinel bootstrap setup; model, slot and pending projections plus public health readback distinguish observational residue from unchanged outcome-taught state. |
| (`inv:guarantee:companion-independence`) | not sweep-shaped | This is a structural absence of dependency between two components, not a predicate over drawn values. |
| (`inv:guarantee:blend-subspace`) | in this lane | Draw shared coordinates and arbitrary excluded coordinates, perturb only the excluded subspace, and compare the weight. |
| (`inv:guarantee:ledger-floor`) | in this lane | Draw a stored value, elapsed interval, and enabled rate, then check that decay discounts without eliminating it. |
| (`inv:guarantee:outcome-neutrality`) | not sweep-shaped | This is an absence from the derivation signature and dependency graph. |
| (`inv:guarantee:drift-visibility`) | not sweep-shaped | The statement is an existence and publication claim about an aggregate trajectory, not one per-draw predicate. |
| (`inv:guarantee:conjugacy`) | in this lane | Draw prior counts and an outcome, then compare the update with exact Beta count arithmetic. |
| (`inv:guarantee:replay`) | in this lane | Draw the complete recorded argument triple and compare replayed landscapes bit for bit. |
| (`inv:guarantee:discrimination-visibility`) | not sweep-shaped | The statement requires the presence of four report families and aggregate histories rather than a predicate on one drawn input. |
| (`inv:guarantee:structural-exactness`) | in this lane | Draw extension and marginalisation geometries and compare the preserved blocks with independent matrix oracles. |
| (`inv:guarantee:axis-lifecycle`) | in this lane | Draw spatial and non-spatial registration and deregistration with trained Sentinels and existing axes; transition projections derive the insertion and removal maps, the supported minimum recomputation cadence makes the covariance-only projection an inverse for the withheld precision, and `testing::regularised_schur_complement` independently exposes exact extension and declared marginalisation. |
| (`inv:guarantee:per-observation-exactness`) | not sweep-shaped | Re-checking the landed independent tier finds specification oracles for rank, Schur marginalisation, decay and dimension width, but no leverage-bounded Bayesian update oracle; without that second computation the statement's sequential conservatism is not a per-draw predicate. |
| (`inv:guarantee:blend-variance`) | in this lane | Draw two component moments and compare the result with the independent mixture identity. |
| (`inv:guarantee:rigid-translation`) | in this lane | Draw risk bases at fixed policy and posterior, then compare crossover offsets, widths, and their common translation. |
| (`inv:guarantee:two-source-uncertainty`) | in this lane | Draw both uncertainty sources and compare every crossover variance with their stated sum. |
| (`inv:guarantee:dominance`) | not sweep-shaped | The report half is an existence claim and the never-enforced half is an absence from policy control flow. |
| (`inv:guarantee:honest-uncertainty`) | not sweep-shaped | Its predicate needs an external truth-and-coverage oracle over a population rather than a per-draw value oracle. |
| (`inv:guarantee:encoding-transparency`) | not sweep-shaped | This is a host-visible field and failure-mode existence claim. |
| (`inv:guarantee:posture-independence`) | not sweep-shaped | This is a structural absence of posture from the regime derivation. |
| (`inv:guarantee:exploration`) | not sweep-shaped | This is an existence and composition claim about an exposed signal. |
| (`inv:guarantee:feed-forward`) | not sweep-shaped | This is a structural direction-of-dependency claim. |
| (`inv:guarantee:non-blocking`) | not sweep-shaped | Scheduler latency and lock progress have no deterministic per-case predicate under this framework-free generator. |
| (`inv:publication:ledger-concurrency`) | not sweep-shaped | Whole-value visibility depends on adversarial thread schedules, not a serial drawable input space. |
| (`inv:guarantee:staleness`) | in this lane | Draw scenario-time advances and publication resets; `World::advance`, the assessment health snapshot and the model projection's scenario-owned observation time expose both positive age and self-correction without elapsed time. |
| (`inv:guarantee:lifecycle-publication`) | in this lane | Draw all six lifecycle directions; `World::block_model_owner_for_test`, the lifecycle barriers and one-version model projections make the pre-publication and post-publication states separately observable. |
| (`inv:landscape:presentation-free`) | not sweep-shaped | This is a structural output and import boundary. |
| (`inv:companion:boundary`) | not sweep-shaped | This is a structural ownership boundary over reads and writes. |
| (`inv:runtime:enumerated-writes`) | in this lane | Draw reporting, signals and identity state together; pending, model and slot projections plus concordance, identity-measurement and signal-cache health expose every enumerated mutation and the forbidden taught-state delta. |
| (`inv:dimension:covering`) | in this lane | Draw complete layouts and mark every routed index exactly once. |
| (`inv:dimension:contiguity`) | in this lane | Draw complete layouts and compare every adjacent block boundary. |
| (`inv:dimension:version-consistency`) | in this lane | Draw registration and deregistration across all registry kinds; settled model and slot projections carry version, layout generation and dimensions that can be checked against the complete runtime layout at every state. |
| (`inv:monitoring:report-only`) | not sweep-shaped | This is an absence of health-state edges into behaviour. |
| (`inv:registry:model-set-serialisation`) | in this lane | Draw every registry event in both directions; lifecycle barriers and model projections expose which full-space models changed, which axis model appeared or disappeared, and that the anchor stayed fixed. |
| (`claim:numerics:the-decay-factor-always-lands-in-the-half-open-unit-interval`) | in this lane | Draw valid rates and elapsed intervals and check the returned factor's range. |
| (`claim:numerics:decay-composes-over-consecutive-intervals`) | in this lane | Draw one rate and two intervals and compare sequential with summed decay. |
| (`claim:numerics:the-sigmoid-inverts-the-logit-across-the-open-unit-interval`) | in this lane | Draw probabilities away from the endpoints and check the round trip. |
| (`claim:linalg:an-operation-leaves-its-result-bitwise-symmetric-not-merely-symmetric-to-tolerance`) | in this lane | Draw symmetric matrices, vectors, and operation parameters and compare mirrored cells by bits. |
| (`claim:linalg:extension-keeps-the-original-block-gives-new-dimensions-an-independent-prior-and-zeroes-the-cross-blocks`) | in this lane | Draw an original matrix, extension width, and prior diagonal and check every resulting region. |
| (`claim:linalg:the-quadratic-form-stays-non-negative-on-a-positive-definite-matrix`) | in this lane | Draw positive-definite matrices and directions and check the form's sign. |
| (`claim:linalg:the-quadratic-form-equals-the-naive-double-sum-over-every-entry`) | in this lane | Draw matrices and directions and compare with an independent double sum. |
| (`claim:linalg:the-fused-call-returns-the-same-product-and-scalar-as-the-two-separate-ones`) | in this lane | Draw matrices and directions and compare the fused and separate routes. |
| (`claim:ledger:the-bad-rate-stays-within-the-unit-interval-under-any-history`) | in this lane | Draw a prior entry, update, and retention values and check one transition; independent cases avoid history-prefix masking. |
| (`claim:ledger:the-compressed-valence-ewma-stays-inside-the-range-of-the-values-it-averages`) | in this lane | Draw a prior entry, bounded valence, and retention and check the convex update. |
| (`claim:ledger:a-decayed-reading-is-always-positive-and-never-exceeds-the-stored-value`) | in this lane | Draw stored values, rates, and elapsed intervals and check the returned view. |
| (`claim:ledger:reading-a-decayed-view-does-not-mutate-the-stored-entry`) | in this lane | Draw complete entries and read times, then compare the stored entry before and after. |
| (`claim:ledger:the-root-receives-every-write-exactly-once-whatever-the-coordinate`) | in this lane | Draw coordinates and layer populations and compare the root tally before and after one write. |
| (`claim:resonance:the-ambiguity-gauge-is-never-negative-in-any-of-its-three-decompositions`) | in this lane | Draw non-degenerate kernel inputs and check all three entropy readings. |
| (`claim:resonance:total-information-value-is-the-chain-rule-sum-over-the-class-action-partition`) | in this lane | Draw non-degenerate kernel inputs and compare the reported total with the independent chain-rule sum. |
| (`claim:wellness:the-stable-inverse-tanh-recovers-its-argument-across-the-whole-working-range`) | in this lane | Draw working-range values and check the hyperbolic round trip. |
| (`claim:resonance:the-regime-table-is-independent-of-the-risk-basis`) | in this lane | Draw two risk bases under one policy and posterior and compare risk-free regime fields. |
| (`claim:resonance:landscape-shape-is-a-function-of-the-declaration-alone`) | in this lane | Draw valid action declarations and evidence and compare output cardinalities with the declaration. |
| (`claim:resonance:one-argument-triple-determines-the-landscape-to-the-bit`) | in this lane | Draw the complete argument triple and compare independent calls by bits. |
| (`claim:bayes:the-corrected-marginal-never-claims-more-precision-than-the-bare-submatrix-and-usually-claims-less`) | in this lane | Draw positive-definite partitioned precision matrices and compare the Schur result with the kept block. |
| (`claim:risk:a-challenge-outcome-adds-one-count-to-its-own-side-and-leaves-the-other-untouched`) | in this lane | Draw valid prior counts and either outcome and compare the posterior with exact count addition. |
| (`claim:risk:the-three-term-mixture-variance-is-non-negative-across-the-whole-input-range`) | in this lane | Draw valid component moments and a weight and check the independent mixture result. |
| (`claim:risk:the-reported-component-variances-recompose-into-the-blended-variance-and-respect-the-subspace-ordering`) | in this lane | Draw diagonal covariances, features, and a shared subspace and recompose the reported diagnostics independently. |
| (`claim:risk:the-blended-variance-is-exactly-the-variance-of-the-two-component-mixture`) | in this lane | Draw component moments through the real blend and compare with the independent second-moment identity. |

The already-covered population is recorded separately and is not part of those census totals: posterior positive-definiteness after rank-one update and marginalisation (`inv:guarantee:precision`) (`claim:bayes:marginalisation-yields-a-smaller-model-that-is-still-positive-on-every-surviving-diagonal`), routed feature-layout width (`claim:feature:the-vectors-width-is-exactly-the-sum-of-its-blocks-with-nothing-between-them`), decay monotonicity in interval and rate (`claim:numerics:decay-never-increases-as-the-interval-lengthens`) (`claim:numerics:a-higher-rate-retains-more-over-the-same-interval`), and boundary-crossover matching (`claim:resonance:adjacent-boundary-tags-meet-at-exactly-the-crossover-the-spec-places-them-around`). Kraft equality preservation remains outside Assayer: the specification deliberately imports that theorem from the Mudlark corpus, and Assayer owns neither a claim nor a contour-enumeration fixture for it. No dependency is added, and the live debug-oracle pin remains (`dec:posterior:debug-oracle`).

The twenty-nine declared-seed packs witness all forty-three sweep-shaped rows: `tests::numerics::decay_range_seeded_sweep`, `tests::numerics::decay_composition_seeded_sweep`, `tests::numerics::sigmoid_logit_seeded_sweep`, `tests::numerics::atanh_tanh_seeded_sweep`, `tests::linalg_symmetric::symmetric_operations_seeded_sweep`, `tests::linalg_symmetric::extension_structure_seeded_sweep`, `tests::linalg_symmetric::quadratic_form_non_negative_seeded_sweep`, `tests::linalg_symmetric::quadratic_form_naive_seeded_sweep`, `tests::linalg_symmetric::quadratic_form_fused_seeded_sweep`, `tests::ledger::ledger_bad_rate_seeded_sweep`, `tests::ledger::ledger_compressed_valence_seeded_sweep`, `tests::ledger::ledger_decayed_read_seeded_sweep`, `tests::ledger::ledger_read_purity_seeded_sweep`, `tests::ledger::ledger_root_routing_seeded_sweep`, `tests::seeded_risk_invariants::challenge_conjugacy_seeded_sweep`, `tests::seeded_risk_invariants::blend_variance_non_negative_seeded_sweep`, `tests::seeded_risk_invariants::blend_component_diagnostics_seeded_sweep`, `tests::seeded_risk_invariants::blend_mixture_identity_seeded_sweep`, `tests::resonance_landscape::landscape_shape_seeded_sweep`, `tests::resonance_landscape::landscape_replay_seeded_sweep`, `tests::resonance_landscape::landscape_translation_and_variance_seeded_sweep`, `tests::dimension_map::layout_covering_and_contiguity_seeded_sweep`, `tests::model_marginalise::corrected_marginal_structure_seeded_sweep`, `tests::resonance_ambiguity::ambiguity_non_negative_seeded_sweep`, `tests::resonance_ambiguity::ambiguity_chain_rule_seeded_sweep`, `tests::seeded_publication_invariants::assessment_write_set_seeded_sweep`, `tests::seeded_publication_invariants::lifecycle_publication_seeded_sweep`, `tests::seeded_publication_invariants::axis_lifecycle_seeded_sweep`, and `tests::seeded_publication_invariants::publication_staleness_seeded_sweep`. The sixteen non-sweep rows stay ruled out for the reasons in the census, including the checked absence of a leverage-bounded update oracle. With no blocked row and no unwitnessed sweep-shaped row, this entry is closed; the wider campaign state remains with its remaining entries.

**Entry (Owned readings the intents still lack)** · `entry:assayer:harness-demanded-readings`

**Outcome class: ENGINEERING.**

**OPEN.** Thirteen of the demand census's harness items ask for an owned reading the landed projections do not supply (`obs:assayer:testing-architecture-intent-demand`). Twelve belong in `testing::probes` behind a `World` entry point; the exception is the channel posterior, which reads a host-owned provider rather than projecting published engine state and therefore belongs on `World` itself. Every one is bounded by the probe contract — gated, barrier-crossing, one published load, owned copy, scenario time, no borrow or mutation route, and no precision field (`dec:harness:probe-contract`) (`cav:retention:probe-boundary`) — so each item is a projection and its focused test, never a widened host surface.

- **An entity-bound decay projection.** One owned selector binding an entity, its active identity cell, routed Ledger key, snapshot version, and frozen standardised sister direction, and one owned reading carrying the identity adverse rate, Ledger bad rate, the stored values and timestamps a purity check needs, and the raw, corrected, and mean sister quantities (`plan:assayer:intent-three-decay-clocks-run-independently`).
- **One decayed Ledger cell.** A single-cell Ledger projection whose entry point resolves a named Sentinel and coordinate through the production deepest-available depth walk, takes one read-only source view at the scenario's persistent time, and returns the decayed adverse rate (`plan:assayer:intent-a-measurement-only-feature-contradicts-stale-history`).
- **Sentinel-slot ranges on the published model block.** An owned mapping from each live Sentinel identifier to its half-open slot range, copied from the same snapshot load that already supplies the mean and covariance, so a witness can compare semantic blocks across layout changes (`plan:assayer:intent-a-hibernating-sentinel-returns-with-its-own-weights`).
- **Competitive-cell chains on the pending-entry view.** Owned cell chains keyed by dimension, copied from the same single pending-entry lookup that supplies their counts, so each risk reading binds to the identity publication it actually used (`plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move`).
- **A relocation projection.** A test-gated projection over named dimension coordinates, one Sentinel coordinate, and nominated identity and Ledger keys, loading each independently published index once and reporting every source's version or boundary separately rather than claiming one global atomic snapshot (`plan:assayer:intent-an-entity-that-relocates-is-tracked-across-the-move`).
- **The shared-subspace blend components.** An owner-consistent assessment reading carrying the raw shared-subspace sister form, the anchor form, the blend guard, and the resulting weight; the published block cannot reconstruct them because it lacks the assembled fixed-probe vector and anchor index map, and the specified full sister uncertainty is not the shared form (`plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it`).
- **The channel posterior.** A read-only reader that resolves a named channel and returns its decayed posterior at scenario time through the existing provider, exposing the raw pair without inferring counts from rendered output and without a mutation route (`plan:assayer:intent-one-investigated-label-moves-every-subsystem-at-once`).
- **A joined label route with its post-label state.** The route copied from the live pending entry before labelling, and a post-label state that crosses the label barrier, performs one published load, and copies the publication version, four model means, class rates, routed Ledger entries, routed identity outcome states, and the Companion posterior (`plan:assayer:intent-one-investigated-label-moves-every-subsystem-at-once`).
- **Per-model assessment-time predictors.** An assessment-scoped component-score projection copying the retained operational, sister, and anchor predictors for one assessment, following the pending-entry single-read boundary without widening the public risk basis (`plan:assayer:intent-the-anchor-catches-a-regime-change-first`).
- **Cumulative leverage-binding counts.** An owned snapshot of attempted and fired counts split into positive and non-positive operational updates, crossing the label barrier and loading once, with a focused test proving that ceiling binding alone is not leverage binding (`plan:assayer:intent-the-leverage-bound-fires-at-cold-start-and-rarely-after`).
- **The cap values one admitted operational update consumed.** A bounded record keyed by assessment identifier carrying the valence branch, policy leverage, target weight, configured ceiling, leverage limit, effective weight, and firing predicate, taken from the same pre-mutation operands the model consumed and written only for an admitted update (`plan:assayer:intent-a-stream-with-no-adverse-outcome-converges-to-near-zero-risk`).
- **A scoped update trace naming the binding limit.** An owned single-label reading recording, for each updated model, its identity, stored leverage, elapsed-time factor, policy leverage, requested weight, ceiling, safety factor, epsilon, and selected effective weight, with activation scoped to one scenario and the publication barrier completing the sample before it is drained (`plan:assayer:intent-a-mis-standardised-arrival-bootstraps-back-into-scale`).
- **A named Sentinel's mean and covariance sub-block.** An owned projection resolving the name and range from the same published snapshot that supplies both model payloads, returning the permitted values and no precision, borrow, guard, shared owner, or mutation route (`plan:assayer:intent-label-indexed-and-time-indexed-decay-compose-multiplicatively`).

Four of the thirteen carry a prerequisite that is not harness work and does not move here. The cumulative leverage counts, the admitted update's cap values, and the scoped update trace each need the production label path to publish a value it already computes; the per-model predictors wait on the production retention change the anchor plan enters in its own section (`entry:assayer:intent-the-anchor-catches-a-regime-change-first-model-residual-retention`). Production-change plans keep their own heads and acceptance conditions rather than being copied into a cluster here (`obs:assayer:testing-harness-record-audit-production-plan-homes`).

**Acceptance.** Each of the thirteen readings exists as a named owned projection reached through the scenario, each with a focused test that establishes barrier freshness, one-version coherence, and the absence of a borrow or mutation route, and the featureless library leaves the probe module and its roster exports uncompiled. A reading whose value the production label path must publish closes only when that publication lands, and the four prerequisites above are named in the closing evidence rather than assumed. The live pin is (`obs:assayer:testing-architecture-intent-demand`), and the result extends the landed contract rather than reopening it (`entry:assayer:harness-probe-contract`).

**Entry (Guarded fixtures and populations the intents still lack)** · `entry:assayer:harness-demanded-fixtures`

**Outcome class: ENGINEERING.**

**OPEN.** Six census items ask for a guarded fixture or population, four in `testing::fixtures` and two in `testing::performance` (`obs:assayer:testing-architecture-intent-demand`). Each names a population no landed fixture can be reshaped into, which is what distinguishes them from the target-local ceremony the other plans keep with their subjects: the shared trained state is fixed to its score-verified schema and default construction, so a differently configured population is a second fixture rather than a widened one.

- **A matched alarming-and-normalised report pair.** The fixed-geometry pair in `testing::reports` and the fixture that owns its playback rows and returns a measured baseline only once the report geometry, sister-versus-anchor prediction separation, deepest-cell adverse-rate floor, class counts, and calibration-refit clearance hold (`plan:assayer:intent-a-measurement-only-feature-contradicts-stale-history`).
- **A dense-conditioning growth fixture.** A shared fixture with a measured baseline that constructs the competitive geometry, drives bounded typed rows through playback, identifies the target model, and returns only after a clean adopted baseline satisfies its finite-ratio, stable-floor, regime-headroom, and remaining-interval preconditions, reporting the last settled checkpoint on failure (`plan:assayer:intent-a-conditioning-estimate-past-a-hundred-thousand-forces-a-refactorisation`).
- **A trained Sentinel pair that survives one retirement.** A public guarded pair and its baseline carrying training progress, class counts, contribution and association readings, pre-removal discrimination, and the Schur-skip count, factoring the existing setup out of the slot-legibility target so a second training path does not survive beside it (`plan:assayer:intent-retiring-an-uninformative-sentinel-costs-no-discrimination`).
- **A sparse-reporting interaction fixture.** A subject fixture that configures both scenarios through the landed interaction-template seam, registers the named population, settles each cold ramp on its complete schedule period, and returns only once the schedule counters and runtime layout agree with the width baseline, reporting expected and measured masks and widths without inspecting the terminal condition number (`plan:assayer:intent-sparsely-reporting-sentinels-ill-condition-the-precision-matrix`).
- **An equal-width ill-conditioning pair.** In `testing::performance`, the two-scenario pair whose gate refuses to return unless widths, registrations, accepted-label counts, convergence, true-condition readings, and clean health all hold (`plan:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost`).
- **The paired cadence-and-cost sampler that reads it.** In `testing::performance`, the sampler that constructs typed rows, declares its barrier policies, installs rebuild-counter checkpoints, invokes the landed label-update recorder, alternates arm order, and reduces complete gaps and paired durations for its two callers (`plan:assayer:intent-ill-conditioning-is-paid-for-in-recomputation-cadence-and-per-label-cost`).

**Acceptance.** Each fixture returns only after checking its named precondition and reporting the measured baseline beside its declaration, each carries a focused refusal witness that invalidates one declared fact and checks the complete diagnostic, and no fixture guard doubles as the result oracle (`dec:harness:guarded-fixtures`) (`dec:harness:separate-validation`). The live pin is (`obs:assayer:testing-architecture-intent-demand`), and the additions extend the landed guarded population rather than loosening its default (`entry:assayer:harness-guarded-fixtures`).

**Entry (Persistence-fork extensions)** · `entry:assayer:harness-fork-extensions`

**Outcome class: ENGINEERING.**

**OPEN.** The landed fork supplies a clean two-arm split; three census items ask it for more, and all three come from the restored-instance promise (`obs:assayer:testing-architecture-intent-demand`) (`plan:assayer:intent-a-restored-instance-continues-the-run-it-was-cut-from`).

- **A crash-tail image builder.** A guarded builder that captures a closed image whose journal carries an unabsorbed tail, clones it into several independently owned restore roots, rebuilds each under a caller-supplied numerical and structural configuration, preserves the one injected clock, and refuses an image whose checkpoint, journal tail, or declarations do not match its measured prefix baseline.
- **A durable comparison an integration target can invoke.** An opaque comparison on the fork that captures both projections after the existing persistence barriers and returns equality or the first field-addressed difference, exposing no precision, mutable engine state, or retained borrow, selecting exact, default, moving-average, and discrimination budgets rather than one blanket epsilon, and checking precision-covariance synchronisation separately through the width-scaled ceiling.
- **A replayable registration tape.** A declaration value that records the original Sentinel, outcome-axis, and identity registrations, reapplies their exact identifiers and process-local identity encoder to a restored arm, crosses the lifecycle and identity-publication barriers, and verifies that the runtime layout did not grow; the shared registration verbs allocate fresh identifiers, which is why a restore callback otherwise drops to raw registrations.

The fourth thing that promise's witness lacks is a persistence payload extension rather than a fork capability, and it stays in the plan that names it, together with the unresolved feedback-latency item that plan records (`obs:assayer:testing-harness-record-audit-production-plan-homes`).

**Acceptance.** The three surfaces exist on the fork or its multi-arm successor with focused tests that establish an unabsorbed tail surviving capture, independently owned roots whose journals do not alias, a rebuilt arm carrying its caller's configuration, a named first difference for a deliberately perturbed field, and a restored arm whose identifiers and layout are unchanged after replay; every arm still restores under the one injected clock and applies elapsed decay once. The live pin is (`obs:assayer:testing-architecture-intent-demand`), and the extensions stay within the durability corollary that placed the fork after the shared surfaces (`cor:durability:harness-fork`) (`entry:assayer:harness-persistence-fork`).

**Entry (Multi-world playback entry point)** · `entry:assayer:harness-paired-playback`

**Outcome class: ENGINEERING.**

**OPEN.** One census item asks `testing::playback` for a runner entry point that takes the worlds a row touches and crosses the selected policy on each, in declaration order, before that row's checkpoint and progress boundary (`obs:assayer:testing-architecture-intent-demand`) (`plan:assayer:intent-one-corrupted-label-in-two-hundred-costs-almost-nothing`). The landed runner crosses its policy on the single world it is handed, while a paired witness needs both settled before the next row derives; a row that flushed its partner itself would take back the settling cadence the row contract reserves for the runner, which is the one thing the declarative choice exists to keep out of subject rows (`dec:harness:declarative-playback`).

**Acceptance.** The entry point stands beside the single-world runner and shares its row trait, and a focused witness fails if either world's barrier can be skipped or run late, naming the blocked row without depending on the barrier set's internal shape. Row order, per-request publication semantics, checkpoint observations, and progress boundaries remain unchanged by batch size, and progress remains liveness evidence rather than a latency reading (`cor:ordering:tape-batches`) (`cav:harness:progress-not-latency`). The live pin is (`obs:assayer:testing-architecture-intent-demand`), and the addition consumes the landed runner rather than introducing a second cadence (`entry:assayer:harness-tape-runner`).

**Entry (Support-tree provider fixture and replay comparator)** · `entry:assayer:harness-support-tree-additions`

**Outcome class: ENGINEERING.**

**OPEN.** Two census items ask the retained multi-channel support tree for additions rather than asking the shared harness for them, which is the declared boundary working rather than failing: the tree keeps its specialised grid, matrices, row shapes, and paired ceremony while shared engine logic and barriers stay common (`dec:harness:specialised-side-harnesses`) (`obs:assayer:testing-architecture-intent-demand`). Both come from the tracker-replacement promise (`plan:assayer:intent-replacing-a-challenge-tracker-leaves-the-core-untouched`).

- **An exchangeable challenge-provider fixture.** In the tree's derivation support, a test-local scalar provider and the boxed exchange that makes the public replacement trait the stimulus, rather than simulating replacement with two bare estimates and asserting nothing about the trait the host would use.
- **An exhaustive derivation replay comparator.** In the tree's assertion support, private comparators for every public landscape, crossover, regime, posterior-input, resonance-profile, tag-resonance, and rendering-configuration field, comparing floating-point payloads by bits, variants and actions exactly, and vector shape and order before their elements.

**Acceptance.** The provider fixture drives a replacement through the public trait and its default posterior, and the comparator replaces the partial fingerprint the existing purity witness retains with a field-complete comparison that fails on any single changed field. Neither addition moves shared engine logic or a queue barrier into the tree, and neither widens the scenario with host-owned controls. The live pin is (`obs:assayer:testing-architecture-intent-demand`).

**Entry (Compile-fail runner and its dependency)** · `entry:assayer:harness-compile-fail-runner`

**Outcome class: PARKED.**

**PARKED.** Two census items ask for the one kind of witness neither tree has machinery for: a compile-fail runner whose checked diagnostic is the assertion (`obs:assayer:testing-architecture-intent-demand`). One runs a fixture whose only intended type error is the inaccessible cause beside the three accessible discrepancy fields (`plan:assayer:intent-a-concordance-flag-names-a-discrepancy-without-attributing-a-cause`); the other adds a target whose fixture establishes that a report payload carries no mutable Sentinel handle (`plan:assayer:intent-no-mutable-handle-on-a-sentinel`). Neither needs a production change, a scenario extension, a clock, a barrier, or liveness support; both need a development dependency first.

That dependency is the reason this entry is parked rather than dispatchable. The harness has a stated dependency boundary: the invariant packs were built framework-free, and the rejection of a property-testing framework was argued on the merits — deterministic generation already exists, and shrinking offers little for failures whose cause is queue order, publication, or a numerical path — with every pack that landed adding no dependency (`dec:harness:seeded-sweeps`). A compile-fail runner is not that framework and the argument against it is not the same argument: its value is a diagnostic no runtime test can produce, and its cost is a build-time dependency in the development closure alone. Deciding by analogy would either import a rejection that was never about this, or set the boundary aside without saying so.

Revisit when a recorded decision admits or refuses a compile-fail development dependency against that boundary. If it is admitted, each witness is roughly one target with one fixture and its checked diagnostic; if it is refused, both plans need their promises witnessed another way or recorded as unwitnessable. Nothing else in this campaign waits on either outcome.

**Entry (The harness conventions get a checker)** · `entry:assayer:harness-convention-checker`

**Outcome class: GUARDRAIL.**

**OPEN.** Two findings name one ownerless gap. The polling retirement closed with its semantic search clean and every base and widened site accounted for, but the repository linter has no rule for authored polling or waits and none was added there, so nothing catches the next one (`entry:assayer:harness-polling-retirement`). The adoption caveat states the other half: the single-scenario claim is descriptive because a caller census says so rather than because a compiler diagnostic can say so, since a public test-support surface gives no dead-code signal, and what would make the claim false again is one new caller that declares common scenario state, common barrier meaning, or a wait of its own (`cav:harness:incremental-adoption`). The conventions already state each of those as a recognizable violation, and the implementation plan deliberately left the checker and its corpus reach to a later entry rather than making an unexplained rule authoritative (`sec:assayer:harness-implementation-conventions`) (`obs:assayer:harness-implementation-convention-enforcement`).

The mechanism is not built in this package. It belongs to the repository's linter as a checked policy, requested under that package's own backlog, on the same terms the register-projection adoption took: the rule must reach crate tests, integration targets, inline tests, benchmarks, and the retained support tree alike, and a recognizer only this package could run would leave uncovered exactly the surfaces a violation arrives on (`entry:assayer:tool-generated-registers`). What this entry owns is the rule set, its admissible exceptions, and the adoption.

**Acceptance.** A repeatable check rejects, each with its site, a test-authored sleep, poll, or wait outside `testing::liveness`; a second declaration of common scenario state; and a second barrier meaning. The six sites the retirement census ruled and retained stay admissible beside their rulings — two sleeps measuring monotonic expiry, one separating real-clock timestamp samples, one pacing a fixed concurrency workload, and two yields widening fixed interleavings without awaiting state — so the rule distinguishes an authored completion wait from a stimulus that measures or paces real time, and reopening them is a failure of the rule rather than a finding (`tab:assayer:harness-polling-retirement-census`). Each rule carries a deliberately violating arm that proves it fires, and the check runs clean over this package with no crate-wide exception. The live pins are (`entry:assayer:harness-polling-retirement`) and (`cav:harness:incremental-adoption`).

**Entry (The registration lift needs its ruling)** · `entry:assayer:harness-registration-lift-ruling`

**Outcome class: RULING.**

**OPEN.** The promise says a registration lifts the blend weight and labels lower it, and the corpus has not chosen what the lift is (`plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it`). Three contracts are live: registration-time widening of the shared block; the later-label counterfactual, under which the transition is observed at the first reporting label rather than at registration; and a compound registration-plus-first-report event. No fixture may conceal the difference, because the branch selected decides what the witness is allowed to assert, and two of the three would rewrite the promise before any test is written. The harness settles none of it and is not waiting on it: every mechanism the eventual witness needs is present, and under the later-label branch the remaining work is a target-local boundary helper over owned published readings (`obs:assayer:testing-harness-record-audit-six-rulings`). This is the one proposal the record audit left genuinely open, and it is entered here because an open obligation with no owning entry is the condition this file exists to prevent (`obs:assayer:testing-harness-record-audit-unminted-proposals`) (`goal:assayer:migration-goal`).

**Acceptance.** A recorded decision names which of the three branches the promise keeps and states its reasons against the specification and the implementation rather than against the convenience of a witness, and the promise or the specification is amended so the two agree. The intent plan's transition contract then cites that decision instead of naming the question, and its witness becomes ordinary test-local work. The live pin is (`plan:assayer:intent-a-registration-lifts-the-blend-weight-and-labels-lower-it`).

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

The route is Criterion benchmarks that consume the shared scenario (`dec:harness:performance-benchmarks`). Its executable work is partitioned with the testing contract rather than duplicated here (`sec:assayer:campaign-testing-harness`) (`sec:assayer:performance-harness-partition`): projection coverage and fixtures precede the `performance` target, and the scheduled/manual artifact route follows that target.

## Claim-area evolution · `sec:assayer:campaign-claim-areas`

This group preserves coherent test-claim stakes by waiting for evidence that a mixed area has outgrown one statement, then proving any eventual split exactly.

**Entry (Mixed claim areas split when they grow)** · `entry:assayer:area-register-split`

**Outcome class: PARKED.**

**PARKED.** The area register identifies `audit` and `scenario` as deliberate mixtures (`sec:assayer:area-register`). Revisit when either area gains enough claims that one stake statement no longer describes what is lost when its claims fail; then split the area, migrate its claims, and let the area-register and claim-profile checks prove the move complete.
