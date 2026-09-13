# The Assayer reference-burn campaign · `rep:assayer:reference-retirement-census`

This scout report is a census and campaign design. It changes no specification, record, source, burn register, or reference. Retired forms are exhibited in double-backtick spans, and commands that must spell them stand in fenced blocks, so this report does not join the debt it measures.

## Executive result · `sec:assayer:reference-burn-executive`

This section and everything below it down to (`sec:assayer:reference-burn-residual-litter`) was measured on draft ``bd904945`` and is the historical record of the campaign as designed. The corpus has moved since. A reader acting on figures rather than on method should read (`sec:assayer:reference-burn-restated`) instead, which restates every number that has drifted and marks each with the commit it was measured at.

The campaign contains exactly 2,785 burnable occurrences in 174 files: 1,858 section-mark forms, 920 lettered record numbers, and seven ambiguous bare record numbers. A separate audit found 58 canonical root-record citations; they are the owner's exception and stay. Six occurrences are hard cases because their exact targets are headings in an unlabelled local API outline. The proposed execution is one adoption wave plus twelve corpus waves, ordered from governing prose and public/core code through subsystems and tests. The expected corpus delta is therefore 2,785 to zero while the 58 root forms remain unchanged.

An eighth family has since been ruled into the campaign and is described in (`sec:assayer:reference-burn-residual-litter`): residual litter, 122 occurrences in 61 files that carry a retired identity and that no recognizer above selects. It is additional to the totals in this paragraph, and it adds one sweep wave to the twelve.

## Reproducible method · `sec:assayer:reference-burn-method`

The linter is the participation-aware oracle — a tool the repository carries, which this package builds without. Its live generated tables, not the handwritten summaries below those tables, are the authoritative per-file census under the burn discipline (`req:migration:burn-ratchet`). Run these from the repository root, the first with the linter's release binary in place of ``linter``:

```text
linter burn --root .

rg -n -o '\bADR-T-[0-9]{3}\b' \
  packages/assayer/src packages/assayer/docs packages/assayer/adr \
  --glob '!reference-burn-campaign.md'

rg -n -o '\bADR-[0-9]{3}\b' \
  packages/assayer/src packages/assayer/docs packages/assayer/adr \
  --glob '!reference-burn-campaign.md'

rg -n '§' packages/assayer/src packages/assayer/tests \
  packages/assayer/docs packages/assayer/adr \
  --glob '!reference-burn-campaign.md'
```

The first command exactly reproduces 1,858 and 920 and the file rows below. The two record searches audit the positive root exception and the unprefixed gap in the current recognizer. The final search is deliberately a candidate/context search: raw grep sees strings and displayed Markdown which the linter correctly does not count. Target classification used the linter's grammar verbatim: one optional space after the mark; then a numeric or Roman locator token, or a document token plus locator; Markdown code spans and blocks do not participate; Rust strings do not participate and comments do. Its classified total was reconciled to the linter's 1,858 before any target map was accepted.

The reproducible count checkpoints were: the burn oracle in 0.72 s, grouped root/bare-record audit in 0.56 s, participation-aware section classification in 1.12 s, record-target grouping in 0.17 s, and the joined per-file/wave census in 0.05 s. All are 2026-08-24 historical measurements on draft ``bd904945`` plus this report.

## Classification rule · `sec:assayer:reference-burn-classification`

Apply this rule mechanically, in this order.

1. A root record keeps its canonical identity, ``ADR-T-NNN``. Root records label themselves in that form in their title. Running prose should retain the ordinary Markdown link where one exists. This exception is grammatical, not path-based: a positive recognizer must never select the ``T`` series.
2. When Assayer needs an environment of a form root policy also states, it cites the Assayer record that states that form, in the ordinary same-owner spelling. Root policy is reachable from every member (`rule:layers:root-is-reachable`) and an import still requires an owner Assayer reaches (`rule:layers:import-must-be-reached`), so the imported spelling remains open where no local record carries the form; what it is not is a substitute for a record this package owes itself.
3. A bare form ``ADR-NNN`` is never inferred to mean a root record. It is ambiguous with the retired local numbering and is burnable. If context proves the root is meant, replace it with the canonical ``ADR-T-NNN`` link; otherwise resolve the owning record and cite its label.
4. Every ``ADR-L-NNN`` naming Assayer's retired numbered record set is burnable. The filename may now be a named record, but a filename is not the replacement: cite the one existing environment label that states the gloss at the use site.
5. The owner's only exception was the global root series. Consequently the current linter's other retired series, ``ADR-M-NNN``, ``ADR-R-NNN`` and ``ADR-S-NNN``, are burnable when they occur in Assayer. Cite an imported label only when the layer graph permits it. Assayer reaches Mudlark and Sentinel; it does not reach render-text-as-image. An unreachable owner is a hard case, not permission to cite across the graph.
6. Every participating section-mark form is in the burn family. If it really points to a specification, record, table, plan, or upstream assertion, replace it with the single existing label covering the gloss. If it is only a local ordinal heading, remove the mark and ordinal and retain the descriptive heading; it has no referent, so manufacturing a citation would be a mint and a category error. No occurrence is silently deleted.

The rule is intentionally asymmetric: ``ADR-T-NNN`` stays, while both ``ADR-NNN`` and ``ADR-L-NNN`` burn. This is how the machinery honors the owner exception without exempting a directory that also contains litter.

## Numbered-record census and replacement map · `sec:assayer:reference-burn-records`

The orchestrator's raw total of about 65 is exact but mixes two classes. There are 58 legitimate root forms and seven burnable bare forms. The legitimate forms spread over fourteen distinct root records: one at 18 occurrences, one at 15, one at six, three at three each, two at two each, and six at one each. They stay.

All seven bare forms are in ``packages/assayer/docs/plans/backlog.md``. The replacement map is mechanical: two ``ADR-008`` occurrences become the canonical root-record link ``ADR-T-008``; two ``ADR-009`` become ``ADR-T-009``; and one each of ``ADR-010``, ``ADR-017`` and ``ADR-018`` becomes its corresponding ``T``-qualified root link.

The live lettered-record family is 920 occurrences in 133 files. The table is grouped by referenced target. “Route” names the current record or imported label family; the sweep chooses the one label matching the sentence's gloss, never all labels in a parenthesis.

| Exhibited target | Count | Replacement route and representative existing heads |
| --- | ---: | --- |
| ``ADR-L-110`` | 9 | Ownership: ``dec:ownership:feed-forward``, ``dec:ownership:graph-custody``, ``dec:ownership:coordinate-width`` |
| ``ADR-L-120`` | 8 | Concurrency: ``dec:concurrency:snapshot-swap``, ``dec:concurrency:per-request-load``, ``dec:concurrency:index-independence`` |
| ``ADR-L-130`` | 14 | Substrate: ``dec:substrate:dense-dynamic``, ``dec:substrate:anchor-invariant``, ``dec:substrate:covariance-only`` |
| ``ADR-L-140`` | 12 | Concurrency owner/channel heads, especially ``dec:concurrency:single-steward`` and ``dec:concurrency:channel-preemption`` |
| ``ADR-L-150`` | 10 | Concurrency failure/thread heads, especially ``dec:concurrency:no-silent-drop`` and ``dec:concurrency:named-thread`` |
| ``ADR-L-160`` | 45 | Durability and retention: checkpoint/journal, decay-once, compatibility, framing, and journal-subset heads |
| ``ADR-L-170`` | 20 | Degradation: ``dec:degradation:infallible-core``, ``dec:degradation:retain-and-flag``, ``dec:degradation:error-partition`` |
| ``ADR-L-210`` | 6 | Substrate symmetry, raw-access, trust-level, backend, and serialisation heads |
| ``ADR-L-220`` | 8 | Substrate backend isolation and mirror-direction heads |
| ``ADR-L-230`` | 18 | Retention snapshot/exclusion heads and the matching substrate serialisation heads |
| ``ADR-L-240`` | 24 | Retention pending-map, reduced-precision, lazy-eviction, and journal-subset heads |
| ``ADR-L-250`` | 19 | Memory coordinate-depth key, depth walk, root permanence, and periodic sweep heads |
| ``ADR-L-260`` | 52 | Memory lazy rows, graph owner, observation surface, competitive publication, and ownership heads |
| ``ADR-L-270`` | 12 | Retention cache-preencoded and cache-losable heads |
| ``ADR-L-310`` | 24 | Ordering assessment-function, batch-timestamp, core-boundary, and closed-crossing heads |
| ``ADR-L-320`` | 30 | Ordering label-function, decay-at-head, publish-at-end, and two-phase-validation heads |
| ``ADR-L-330`` | 35 | Ordering synchronous-reception, maintenance-first, lock-order, and no-sequencing heads |
| ``ADR-L-340`` | 23 | Clock two-domain/clamp/shared-function/lazy-application heads; durability owns decay-once |
| ``ADR-L-350`` | 25 | Vector block-order, sole-resolver, semantic-template, and unstandardised-base heads |
| ``ADR-L-360`` | 24 | Vector label-time assembly, statistic timing, acquisition, transient, and prior heads |
| ``ADR-L-370`` | 46 | Health publication, tracker, convergence, and calibration-settlement heads |
| ``ADR-L-410`` | 16 | Posterior three-step update, combined factor, identity substitution, and leverage heads |
| ``ADR-L-420`` | 54 | Posterior repair cascade, inversion, trigger, cadence, conditioning, and refactor heads |
| ``ADR-L-430`` | 22 | Posterior half-solve, infallible correction, conditioning guard, and debug-oracle heads |
| ``ADR-L-440`` | 39 | Derivation pure-transform, constants, ordered-actions, allowlisted-imports, and closed-input heads |
| ``ADR-L-450`` | 32 | Calibration per-regime-search, pure-refit, drift-reset, shared-transition, and indexed-form heads |
| ``ADR-L-460`` | 19 | Calibration raw-weight, clamped-form, unconditional-refit, and shared-transition heads |
| ``ADR-L-510`` | 35 | Surface assessment/boundary heads plus ordering where the gloss is execution order |
| ``ADR-L-520`` | 21 | Surface label-submission and consumption heads plus degradation for failure semantics |
| ``ADR-L-530`` | 22 | Surface reception, slot-map, isolation, and diagnostic-ack heads |
| ``ADR-L-540`` | 67 | Construction defaults/validation/build heads, split by the specific field named |
| ``ADR-L-550`` | 54 | Health tier, publication, tracker, report, convergence, and diagnostic-entry heads |
| ``ADR-L-560`` | 44 | Construction lifecycle/build heads plus the matching vector and memory decisions |
| ``ADR-L-570`` | 25 | Metrics passive-export, catalogue, neutral-sample, membership, and cardinality heads |
| ``ADR-L-580`` | 3 | Surface read-only guidance, three categories, bounded scans, and guidance opacity |
| ``ADR-M-018`` | 1 | Mudlark hard-budget claim imported as ``[MUDLARK-claim:observe:a-budgeted-graph-never-exceeds-its-budget-because-observation-evicts-before-returning]`` |
| ``ADR-M-041`` | 2 | Mudlark Pewei depth/extraction claims, including ``[MUDLARK-claim:pewei:limiting-extraction-depth-keeps-at-most-that-many-layers-and-no-more-energy-than-a-full-extraction]`` |

One old record may route to several heads because the named records deliberately collapsed numbered sections into semantic decisions. The section locator and the sentence gloss disambiguate; replacing every occurrence of one number with one universal label would be incorrect.

## Section-mark census and replacement map · `sec:assayer:reference-burn-sections`

The 1,858 occurrences divide by what they actually denote:

| Target class | Count | What it points at | Replacement action |
| --- | ---: | --- | --- |
| Old Assayer monolith | 655 | Numbered chapters, sections, tables, or appendices of the pre-split specification | Cite the existing current spec environment; the chapter routing table below is the first index, then the gloss selects the precise head |
| Numbered record subsection | 353 | A numbered subsection of a retired lettered record, on the same comment as that record number | Use the named-record route above and select the environment matching the subsection gloss |
| Local or retired auxiliary outline | 839 | Mostly ordinal Rust headings with no referent; the remainder points into retired layer summaries, testing plans, scenario matrices, spikes, READMEs, and local tables | De-ordinalize true headings; for real auxiliary references cite the existing test/spec/record label named by the gloss |
| Upstream document | 5 | Three real Mudlark/Sentinel-facing assertions and two displayed examples in the audit | Import the reached owner's existing claim for the three; double-backtick the two examples because they are exhibits |
| Current local API outline | 6 | Six headings in ``docs/upstream-apis.md`` | Hard cases: exact target headings have no labels; described mint requests below |

The 839 auxiliary/local forms split reproducibly by the carrier named in their comment. This is the target map for that class:

| Auxiliary target | Count | Replacement route |
| --- | ---: | --- |
| Source-local ordinal or table | 614 | The mark introduces a same-file heading/table or merely orders a comment block. Remove true ordinals; where prose points to a real local asset, cite that asset's existing environment or test label. |
| Retired scenario matrix or scenario slice | 160 | Cite the current ``test:crate:*`` or ``test:integration:*`` identity for the scenario implementation and its existing governing claim; a scenario-title heading itself is de-ordinalized. |
| Retired layer summary | 27 | Select the current named record or spec head stated by the gloss; layer-summary numbers carry no identity into the replacement. |
| Work package or testing plan | 18 | Cite the landed work-plan environment for acceptance intent or the current test identity for implemented evidence. |
| Test README outline | 18 | Cite the current test identity already indexed in the README; Roman chapters and row numbers disappear. |
| Audit or deferral table | 1 | Cite the existing deferral register/table named by the surrounding prose. |
| Retired testing specification | 1 | Cite the current report-ingestion algorithm/test identity; the retired layer-testing number disappears. |

The table is exhaustive and sums to 839. Its first row contains both references and non-references because the same recognizer intentionally burns their common surface shape; the sentence and comment position distinguish the action.

For the 655 old-spec references, this complete chapter-level map groups every locator by target. A chapter label is the replacement only for a chapter-wide claim; a narrower use must select the existing definition, requirement, algorithm, table, invariant, caveat, or other head under that chapter.

| Old locator group | Count | Current routing head |
| --- | ---: | --- |
| ``L-1`` | 12 | ``chap:spec:purpose-and-principles`` |
| ``L-2`` | 2 | ``chap:spec:encoding-contract`` |
| ``L-3`` | 1 | ``chap:spec:valence-asymmetry`` |
| ``L-4`` | 21 | ``chap:spec:mathematical-foundation`` |
| ``L-5`` | 20 | ``chap:spec:registries-and-lifecycle`` |
| ``L-6`` | 48 | ``chap:spec:structured-extraction`` |
| ``L-7`` | 18 | ``chap:spec:key-space-identity`` |
| ``L-8`` | 5 | ``chap:spec:host-signals`` |
| ``L-9`` | 46 | ``chap:spec:feature-vector`` |
| ``L-10`` | 53 | ``chap:spec:core-risk-models`` |
| ``L-11`` | 14 | ``chap:spec:axis-prediction`` |
| ``L-12`` | 23 | ``chap:spec:outcome-ledger`` |
| ``L-13`` | 29 | ``chap:spec:derivation-interface`` |
| ``L-14`` | 7 | ``chap:spec:channel-policy`` |
| ``L-15`` | 19 | ``chap:spec:crossover-landscape`` |
| ``L-16`` | 92 | ``chap:spec:worked-landscapes`` |
| ``L-17`` | 35 | ``chap:spec:exploration-and-guidance`` |
| ``L-18`` | 24 | ``chap:spec:challenge-effectiveness`` |
| ``L-19`` | 9 | ``chap:spec:host-contract`` |
| ``L-20`` | 31 | ``chap:spec:assessment-interface`` |
| ``L-21`` | 11 | ``chap:spec:label-pipeline`` |
| ``L-22`` | 10 | ``chap:spec:concurrency`` |
| ``L-23`` | 1 | ``chap:spec:temporal-governance`` |
| ``L-24`` | 2 | ``chap:spec:initialisation-and-warmup`` |
| ``L-25`` | 6 | ``chap:spec:convergence-and-resources`` |
| ``L-26`` | 42 | ``chap:spec:detection-analysis`` |
| ``L-27`` | 40 | ``chap:spec:health-monitoring`` |
| ``L-28`` | 1 | ``chap:spec:configuration`` |
| ``L-29`` | 16 | ``chap:spec:output-structures`` |
| ``L-30`` | 9 | ``chap:spec:guarantees-and-limitations`` |
| ``L-A`` | 6 | ``app:spec:resonance-rendering`` |
| ``L-H`` | 2 | ``app:spec:background`` or a narrower imported boundary head, selected by the gloss |

Frequent narrow targets already have exact replacements: prediction-drift references route to ``alg:monitoring:drift-cusums``; the precision replenishment floor routes to ``req:gaussian:prior-replenishment-floor``; standardisation timing routes to ``req:standardisation:timing``; label-time standardisation routes to ``alg:standardisation:label-time-procedure``; crossover action locations route to the matching heads in ``landscape-crossovers.md``; and guarantees should cite their ``inv:guarantee:*`` or ``cav:limitation:*`` head, not the chapter head.

The three real upstream cases also have routes. The annihilation assertion uses ``[MUDLARK-claim:attenuation:a-zero-attenuation-factor-annihilates-all-accumulated-value]``. The Pewei snapshot assertion uses ``[MUDLARK-claim:pewei:a-snapshot-taken-from-a-live-graph-reconstructs-to-the-graphs-total-energy]``. The maintenance-loop decay comment is an Assayer publication/change-detection claim and routes to ``dec:memory:competitive-publication``; it need not retain a broad Mudlark chapter pointer. The two audit examples are shown forms, not claims, and should become double-backtick exhibits.

### Full file census · `sec:assayer:reference-burn-files`

The following joined table is the complete per-file census. “Lettered” is the current ``L/M/R/S`` burn recognizer; “bare” is the proposed missing family. The wave column is also the exact file assignment used later.

| Wave | File | Section marks | Lettered records | Bare records | Total |
| --- | --- | ---: | ---: | ---: | ---: |
| B1 | ``packages/assayer/docs/plans/backlog.md`` | 0 | 0 | 7 | 7 |
| B1 | ``packages/assayer/docs/plans/spec-audit.md`` | 132 | 0 | 0 | 132 |
| B1 | ``packages/assayer/docs/plans/spec-sections/parts-3-4.md`` | 47 | 0 | 0 | 47 |
| B1 | ``packages/assayer/docs/upstream-apis.md`` | 46 | 22 | 0 | 68 |
| B2 | ``packages/assayer/src/api/assess.rs`` | 5 | 6 | 0 | 11 |
| B2 | ``packages/assayer/src/api/builder.rs`` | 17 | 9 | 0 | 26 |
| B2 | ``packages/assayer/src/api/guidance.rs`` | 1 | 1 | 0 | 2 |
| B2 | ``packages/assayer/src/api/health.rs`` | 4 | 11 | 0 | 15 |
| B2 | ``packages/assayer/src/api/label.rs`` | 8 | 11 | 0 | 19 |
| B2 | ``packages/assayer/src/api/lifecycle.rs`` | 8 | 10 | 0 | 18 |
| B2 | ``packages/assayer/src/api/preseed.rs`` | 3 | 4 | 0 | 7 |
| B2 | ``packages/assayer/src/api/report.rs`` | 6 | 10 | 0 | 16 |
| B2 | ``packages/assayer/src/assessment.rs`` | 82 | 63 | 0 | 145 |
| B2 | ``packages/assayer/src/config/mod.rs`` | 0 | 1 | 0 | 1 |
| B2 | ``packages/assayer/src/config/types.rs`` | 32 | 16 | 0 | 48 |
| B2 | ``packages/assayer/src/error.rs`` | 18 | 14 | 0 | 32 |
| B2 | ``packages/assayer/src/lib.rs`` | 11 | 40 | 0 | 51 |
| B2 | ``packages/assayer/src/types.rs`` | 17 | 13 | 0 | 30 |
| B3 | ``packages/assayer/src/linalg/bridge.rs`` | 7 | 14 | 0 | 21 |
| B3 | ``packages/assayer/src/linalg/convert.rs`` | 4 | 2 | 0 | 6 |
| B3 | ``packages/assayer/src/linalg/mod.rs`` | 0 | 2 | 0 | 2 |
| B3 | ``packages/assayer/src/linalg/serde_support.rs`` | 4 | 2 | 0 | 6 |
| B3 | ``packages/assayer/src/linalg/symmetric.rs`` | 6 | 12 | 0 | 18 |
| B3 | ``packages/assayer/src/model/bayesian.rs`` | 11 | 32 | 0 | 43 |
| B3 | ``packages/assayer/src/model/marginalise.rs`` | 11 | 3 | 0 | 14 |
| B3 | ``packages/assayer/src/model/mod.rs`` | 0 | 6 | 0 | 6 |
| B3 | ``packages/assayer/src/model/parameters.rs`` | 0 | 3 | 0 | 3 |
| B3 | ``packages/assayer/src/model/recompute.rs`` | 12 | 7 | 0 | 19 |
| B3 | ``packages/assayer/src/model/update.rs`` | 7 | 5 | 0 | 12 |
| B3 | ``packages/assayer/src/numerics.rs`` | 8 | 7 | 0 | 15 |
| B4 | ``packages/assayer/src/extraction/alarm.rs`` | 7 | 0 | 0 | 7 |
| B4 | ``packages/assayer/src/extraction/chain.rs`` | 7 | 0 | 0 | 7 |
| B4 | ``packages/assayer/src/extraction/coordination.rs`` | 6 | 0 | 0 | 6 |
| B4 | ``packages/assayer/src/extraction/cusum.rs`` | 7 | 0 | 0 | 7 |
| B4 | ``packages/assayer/src/extraction/ledger_features.rs`` | 5 | 0 | 0 | 5 |
| B4 | ``packages/assayer/src/extraction/mod.rs`` | 14 | 2 | 0 | 16 |
| B4 | ``packages/assayer/src/extraction/structure.rs`` | 5 | 0 | 0 | 5 |
| B4 | ``packages/assayer/src/feature/aggregate.rs`` | 6 | 0 | 0 | 6 |
| B4 | ``packages/assayer/src/feature/assembly.rs`` | 15 | 10 | 0 | 25 |
| B4 | ``packages/assayer/src/feature/bootstrap.rs`` | 7 | 4 | 0 | 11 |
| B4 | ``packages/assayer/src/feature/dimension_map.rs`` | 20 | 4 | 0 | 24 |
| B4 | ``packages/assayer/src/feature/interaction.rs`` | 10 | 5 | 0 | 15 |
| B4 | ``packages/assayer/src/feature/mod.rs`` | 0 | 4 | 0 | 4 |
| B4 | ``packages/assayer/src/feature/standardisation.rs`` | 19 | 6 | 0 | 25 |
| B5 | ``packages/assayer/src/identity/cell_state.rs`` | 8 | 3 | 0 | 11 |
| B5 | ``packages/assayer/src/identity/competitive.rs`` | 5 | 3 | 0 | 8 |
| B5 | ``packages/assayer/src/identity/dimension.rs`` | 3 | 3 | 0 | 6 |
| B5 | ``packages/assayer/src/identity/maintenance_loop.rs`` | 8 | 7 | 0 | 15 |
| B5 | ``packages/assayer/src/identity/mod.rs`` | 3 | 4 | 0 | 7 |
| B5 | ``packages/assayer/src/identity/observation.rs`` | 5 | 4 | 0 | 9 |
| B5 | ``packages/assayer/src/identity/snapshot.rs`` | 4 | 2 | 0 | 6 |
| B5 | ``packages/assayer/src/ledger/cell_set.rs`` | 5 | 4 | 0 | 9 |
| B5 | ``packages/assayer/src/ledger/entry.rs`` | 13 | 5 | 0 | 18 |
| B5 | ``packages/assayer/src/ledger/gc.rs`` | 1 | 1 | 0 | 2 |
| B5 | ``packages/assayer/src/ledger/mod.rs`` | 6 | 3 | 0 | 9 |
| B5 | ``packages/assayer/src/ledger/routing.rs`` | 4 | 1 | 0 | 5 |
| B5 | ``packages/assayer/src/ledger/sentinel_ledger.rs`` | 3 | 2 | 0 | 5 |
| B5 | ``packages/assayer/src/signal/cache.rs`` | 8 | 6 | 0 | 14 |
| B5 | ``packages/assayer/src/signal/mod.rs`` | 2 | 1 | 0 | 3 |
| B5 | ``packages/assayer/src/signal/schema.rs`` | 4 | 2 | 0 | 6 |
| B5 | ``packages/assayer/src/signal/value.rs`` | 13 | 3 | 0 | 16 |
| B6 | ``packages/assayer/src/owner/commands.rs`` | 19 | 43 | 0 | 62 |
| B6 | ``packages/assayer/src/owner/label_path.rs`` | 45 | 44 | 0 | 89 |
| B6 | ``packages/assayer/src/owner/lifecycle.rs`` | 27 | 21 | 0 | 48 |
| B6 | ``packages/assayer/src/owner/mod.rs`` | 0 | 4 | 0 | 4 |
| B6 | ``packages/assayer/src/owner/thread.rs`` | 13 | 17 | 0 | 30 |
| B7 | ``packages/assayer/src/pending/buffer.rs`` | 4 | 5 | 0 | 9 |
| B7 | ``packages/assayer/src/pending/entry.rs`` | 12 | 9 | 0 | 21 |
| B7 | ``packages/assayer/src/pending/mod.rs`` | 2 | 2 | 0 | 4 |
| B7 | ``packages/assayer/src/persistence/checkpoint.rs`` | 12 | 8 | 0 | 20 |
| B7 | ``packages/assayer/src/persistence/journal.rs`` | 7 | 6 | 0 | 13 |
| B7 | ``packages/assayer/src/persistence/mod.rs`` | 1 | 3 | 0 | 4 |
| B7 | ``packages/assayer/src/persistence/recovery.rs`` | 12 | 11 | 0 | 23 |
| B7 | ``packages/assayer/src/persistence/scheduler.rs`` | 1 | 1 | 0 | 2 |
| B7 | ``packages/assayer/src/snapshot/mod.rs`` | 0 | 6 | 0 | 6 |
| B7 | ``packages/assayer/src/snapshot/published.rs`` | 10 | 12 | 0 | 22 |
| B7 | ``packages/assayer/src/snapshot/shared.rs`` | 2 | 3 | 0 | 5 |
| B7 | ``packages/assayer/src/snapshot/working.rs`` | 18 | 21 | 0 | 39 |
| B8 | ``packages/assayer/src/guidance/mod.rs`` | 2 | 1 | 0 | 3 |
| B8 | ``packages/assayer/src/health/blend_stats.rs`` | 4 | 2 | 0 | 6 |
| B8 | ``packages/assayer/src/health/composite.rs`` | 7 | 7 | 0 | 14 |
| B8 | ``packages/assayer/src/health/concordance.rs`` | 10 | 8 | 0 | 18 |
| B8 | ``packages/assayer/src/health/counters.rs`` | 2 | 2 | 0 | 4 |
| B8 | ``packages/assayer/src/health/degradation.rs`` | 5 | 6 | 0 | 11 |
| B8 | ``packages/assayer/src/health/drift.rs`` | 9 | 2 | 0 | 11 |
| B8 | ``packages/assayer/src/health/events.rs`` | 9 | 6 | 0 | 15 |
| B8 | ``packages/assayer/src/health/identity_tracker.rs`` | 11 | 6 | 0 | 17 |
| B8 | ``packages/assayer/src/health/mod.rs`` | 2 | 7 | 0 | 9 |
| B8 | ``packages/assayer/src/health/platt_tracker.rs`` | 13 | 9 | 0 | 22 |
| B8 | ``packages/assayer/src/health/published.rs`` | 5 | 6 | 0 | 11 |
| B8 | ``packages/assayer/src/health/summary.rs`` | 12 | 11 | 0 | 23 |
| B8 | ``packages/assayer/src/metrics/catalog.rs`` | 7 | 7 | 0 | 14 |
| B8 | ``packages/assayer/src/metrics/mapper.rs`` | 9 | 5 | 0 | 14 |
| B8 | ``packages/assayer/src/metrics/mod.rs`` | 0 | 3 | 0 | 3 |
| B8 | ``packages/assayer/src/report/index.rs`` | 11 | 16 | 0 | 27 |
| B8 | ``packages/assayer/src/report/ingestion.rs`` | 12 | 11 | 0 | 23 |
| B8 | ``packages/assayer/src/report/mod.rs`` | 2 | 4 | 0 | 6 |
| B8 | ``packages/assayer/src/report/slot.rs`` | 8 | 9 | 0 | 17 |
| B8 | ``packages/assayer/src/report/validation.rs`` | 7 | 2 | 0 | 9 |
| B8 | ``packages/assayer/src/testing/asserts.rs`` | 2 | 0 | 0 | 2 |
| B8 | ``packages/assayer/src/testing/degradation_spec.rs`` | 1 | 1 | 0 | 2 |
| B8 | ``packages/assayer/src/testing/mod.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/testing/reports.rs`` | 5 | 0 | 0 | 5 |
| B8 | ``packages/assayer/src/testing/tolerances.rs`` | 2 | 0 | 0 | 2 |
| B9 | ``packages/assayer/src/resonance/channel.rs`` | 19 | 4 | 0 | 23 |
| B9 | ``packages/assayer/src/resonance/derivation.rs`` | 58 | 6 | 0 | 64 |
| B9 | ``packages/assayer/src/resonance/exploration.rs`` | 3 | 2 | 0 | 5 |
| B9 | ``packages/assayer/src/resonance/mod.rs`` | 4 | 1 | 0 | 5 |
| B9 | ``packages/assayer/src/resonance/tags.rs`` | 6 | 1 | 0 | 7 |
| B9 | ``packages/assayer/src/risk/blend.rs`` | 13 | 13 | 0 | 26 |
| B9 | ``packages/assayer/src/risk/calibration.rs`` | 24 | 20 | 0 | 44 |
| B9 | ``packages/assayer/src/risk/challenge.rs`` | 29 | 2 | 0 | 31 |
| B9 | ``packages/assayer/src/risk/mod.rs`` | 1 | 1 | 0 | 2 |
| B10 | ``packages/assayer/src/tests/builder.rs`` | 50 | 17 | 0 | 67 |
| B10 | ``packages/assayer/src/tests/health_convergence.rs`` | 11 | 0 | 0 | 11 |
| B10 | ``packages/assayer/src/tests/health_infrastructure.rs`` | 11 | 1 | 0 | 12 |
| B10 | ``packages/assayer/src/tests/identity.rs`` | 5 | 1 | 0 | 6 |
| B10 | ``packages/assayer/src/tests/identity_maintenance.rs`` | 17 | 1 | 0 | 18 |
| B10 | ``packages/assayer/src/tests/ingestion.rs`` | 12 | 1 | 0 | 13 |
| B10 | ``packages/assayer/src/tests/label_pipeline.rs`` | 8 | 7 | 0 | 15 |
| B10 | ``packages/assayer/src/tests/ledger.rs`` | 38 | 0 | 0 | 38 |
| B10 | ``packages/assayer/src/tests/lifecycle.rs`` | 17 | 5 | 0 | 22 |
| B10 | ``packages/assayer/src/tests/lifecycle_api.rs`` | 6 | 1 | 0 | 7 |
| B10 | ``packages/assayer/src/tests/lifecycle_integration.rs`` | 28 | 7 | 0 | 35 |
| B10 | ``packages/assayer/src/tests/owner.rs`` | 18 | 1 | 0 | 19 |
| B10 | ``packages/assayer/src/tests/pending.rs`` | 6 | 1 | 0 | 7 |
| B10 | ``packages/assayer/src/tests/persistence.rs`` | 22 | 9 | 0 | 31 |
| B10 | ``packages/assayer/src/tests/snapshot.rs`` | 9 | 3 | 0 | 12 |
| B11 | ``packages/assayer/src/tests/aggregate.rs`` | 1 | 0 | 0 | 1 |
| B11 | ``packages/assayer/src/tests/api_surfaces.rs`` | 12 | 4 | 0 | 16 |
| B11 | ``packages/assayer/src/tests/assembly.rs`` | 4 | 0 | 0 | 4 |
| B11 | ``packages/assayer/src/tests/assess_pipeline.rs`` | 6 | 2 | 0 | 8 |
| B11 | ``packages/assayer/src/tests/benchmarks.rs`` | 7 | 0 | 0 | 7 |
| B11 | ``packages/assayer/src/tests/dimension_map.rs`` | 8 | 0 | 0 | 8 |
| B11 | ``packages/assayer/src/tests/guidance.rs`` | 2 | 1 | 0 | 3 |
| B11 | ``packages/assayer/src/tests/helpers.rs`` | 2 | 0 | 0 | 2 |
| B11 | ``packages/assayer/src/tests/metrics_export.rs`` | 7 | 6 | 0 | 13 |
| B11 | ``packages/assayer/src/tests/mod.rs`` | 8 | 7 | 0 | 15 |
| B11 | ``packages/assayer/src/tests/model_bayesian.rs`` | 1 | 2 | 0 | 3 |
| B11 | ``packages/assayer/src/tests/model_marginalise.rs`` | 4 | 2 | 0 | 6 |
| B11 | ``packages/assayer/src/tests/model_recompute.rs`` | 4 | 3 | 0 | 7 |
| B11 | ``packages/assayer/src/tests/numerics.rs`` | 3 | 0 | 0 | 3 |
| B11 | ``packages/assayer/src/tests/outcome_drift.rs`` | 8 | 0 | 0 | 8 |
| B11 | ``packages/assayer/src/tests/report.rs`` | 8 | 0 | 0 | 8 |
| B11 | ``packages/assayer/src/tests/resonance_crossover.rs`` | 3 | 0 | 0 | 3 |
| B11 | ``packages/assayer/src/tests/resonance_golden.rs`` | 7 | 0 | 0 | 7 |
| B11 | ``packages/assayer/src/tests/resonance_robustness.rs`` | 30 | 7 | 0 | 37 |
| B11 | ``packages/assayer/src/tests/resonance_sufficiency.rs`` | 8 | 0 | 0 | 8 |
| B11 | ``packages/assayer/src/tests/resonance_tags.rs`` | 4 | 0 | 0 | 4 |
| B11 | ``packages/assayer/src/tests/source_audit.rs`` | 3 | 4 | 0 | 7 |
| B11 | ``packages/assayer/src/tests/standardisation.rs`` | 14 | 0 | 0 | 14 |
| B11 | ``packages/assayer/src/tests/types.rs`` | 3 | 0 | 0 | 3 |
| B12 | ``packages/assayer/tests/assess_integration.rs`` | 11 | 1 | 0 | 12 |
| B12 | ``packages/assayer/tests/derive_purity.rs`` | 6 | 1 | 0 | 7 |
| B12 | ``packages/assayer/tests/edge_cases.rs`` | 10 | 1 | 0 | 11 |
| B12 | ``packages/assayer/tests/identity_layer.rs`` | 9 | 2 | 0 | 11 |
| B12 | ``packages/assayer/tests/label_integration.rs`` | 4 | 2 | 0 | 6 |
| B12 | ``packages/assayer/tests/learning_convergence.rs`` | 4 | 0 | 0 | 4 |
| B12 | ``packages/assayer/tests/lifecycle_algebra.rs`` | 6 | 1 | 0 | 7 |
| B12 | ``packages/assayer/tests/marginalise_rank1_formula.rs`` | 8 | 3 | 0 | 11 |
| B12 | ``packages/assayer/tests/multi_channel.rs`` | 7 | 0 | 0 | 7 |
| B12 | ``packages/assayer/tests/outcome_ledger.rs`` | 11 | 1 | 0 | 12 |
| B12 | ``packages/assayer/tests/resonance_geometry.rs`` | 13 | 1 | 0 | 14 |
| B12 | ``packages/assayer/tests/scenarios.rs`` | 9 | 1 | 0 | 10 |
| B12 | ``packages/assayer/tests/signal.rs`` | 19 | 1 | 0 | 20 |
| B12 | ``packages/assayer/tests/support/assertions.rs`` | 7 | 0 | 0 | 7 |
| B12 | ``packages/assayer/tests/support/constants.rs`` | 3 | 0 | 0 | 3 |
| B12 | ``packages/assayer/tests/support/derivation.rs`` | 43 | 0 | 0 | 43 |
| B12 | ``packages/assayer/tests/support/enrichment.rs`` | 13 | 0 | 0 | 13 |
| B12 | ``packages/assayer/tests/support/labels.rs`` | 11 | 0 | 0 | 11 |
| B12 | ``packages/assayer/tests/support/outcomes.rs`` | 8 | 0 | 0 | 8 |
| B12 | ``packages/assayer/tests/support/policy.rs`` | 4 | 0 | 0 | 4 |
| B12 | ``packages/assayer/tests/support/rewards.rs`` | 4 | 0 | 0 | 4 |
| B12 | ``packages/assayer/tests/support/signals.rs`` | 3 | 0 | 0 | 3 |
| **All** | **174 files** | **1,858** | **920** | **7** | **2,785** |

## Hard cases · `sec:assayer:reference-burn-hard-cases`

Exactly six participating occurrences lack a covering identity for their exact target. Each occurs once in ``packages/assayer/docs/upstream-apis.md``. The requests below describe, but do not mint, the needed heads. Their owner is Assayer and their carrier is that document.

| Exhibited locator | Requested named content |
| --- | --- |
| ``§3`` | Identity-layer ownership surface: purpose, instantiated graph, and ownership relationship |
| ``§2`` | Sentinel observational surface: detached report types and the consumption boundary |
| ``§6`` | Realized import discipline: the exact Sentinel and Mudlark imports the package consumes |
| ``§8`` | Module-location inventory: which Assayer modules own each consumed upstream role |
| ``§3.4`` | Used-method inventory: the Mudlark methods Assayer calls and why |
| ``§3.5`` | Deliberate non-consumption inventory: the Mudlark methods Assayer does not call and why |

If those mints are declined, the alternative is to replace each pointer with a direct sentence or table link under an already labelled ownership head; that is a content rewrite and must be ruled explicitly. The ordinal Rust headings are not included as hard cases: they have no target at all, and their non-silent resolution is de-ordinalization.

## Residual litter · `sec:assayer:reference-burn-residual-litter`

One further family is ruled into the campaign. It covers the three shapes that carry a retired identity and that no current recognizer selects, and it is one family rather than three because the three share a cause: each is what an earlier sweep, or the retired convention's own notation, left standing after the part a recognizer could see had gone. The census below is as of draft ``fe59a601``.

The family is **residual litter**: 122 occurrences in 61 files, of which 45 are work-package numbers, 44 are bare lettered locators, and 33 are word-shaped section marks. None of the three is counted by any of the seven declared families, and the count is therefore additional to the campaign's live totals rather than a re-partition of them.

### What the three shapes are · `sec:assayer:reference-burn-residual-shapes`

*Work-package numbers.* A retired work plan numbered its deliverables, and the numbers outlived the plan. The form is ``WP-`` and a dotted number, and it occurs almost entirely in module-doc provenance lists — a header naming the work package a module was built under. It is a private identity scheme of one retired document, which is the shape the campaign's scenario and division families already have.

*Bare lettered locators.* The form is a lettered locator standing with no prefix and no mark: ``L-NNN`` where the corpus's retired record is ``ADR-L-NNN``, or ``L-N`` where the old specification's chapter locator was introduced by a mark. It is not a new naming scheme. It is the existing record and section families' own forms with the part the recognizer reads removed, so the occurrence keeps its full referential force while becoming invisible to the gate that was built to see it.

*Word-shaped section marks.* The form is the mark followed by a word rather than a number: a document token whose locator has gone (``§SPEC`` with nothing after it), an ordinal placeholder copied from a convention's own meta-notation (``§N``), or the mark introducing a named rather than numbered division (the mark, one space, and a capitalised phrase). The current recognizer requires a digit or a Roman run, so all three fall outside it.

### Token boundaries · `sec:assayer:reference-burn-residual-boundaries`

These are stated so the recognizer bite implements them without judgment. All three take the campaign's existing participation boundary unchanged (`conv:migration:burn-surface-reading`): prose is read as its format and a form in code font or a fenced block is displayed rather than referenced; code is read for its comments alone.

A token *opens* where a word does not continue — the character before it is not a letter, a digit, an underscore or a hyphen — which is the boundary the unprefixed-record rule already uses and is adopted here verbatim rather than restated differently.

*Work-package numbers.* The token is ``WP-`` opening a token, then a digit run, then any number of further digit runs each joined by one full stop. The token ends at the first character that is neither a digit nor a joining full stop, and a trailing full stop belongs to the sentence rather than to the token. The bound is by enumeration (`conv:migration:burn-family`): the eight forms the retired plan actually numbered, which are ``WP-2``, ``WP-8``, and ``WP-4.0`` through ``WP-4.5``. An unbounded reading would count every capital-letter pair followed by a number in the corpus and could never reach zero.

*Bare lettered locators.* The token is ``L-`` opening a token, then a digit run, then optionally further dot-joined digit runs. Two exclusions make it a family rather than a pattern. First, a locator standing inside a reference the section recognizer has already read is not a second occurrence: where a mark, or a mark and a document token, immediately precedes the locator, the whole is one reference and it belongs to the section family. Second, the bound is by enumeration over the numbers the two retired documents actually carried — the old specification's chapter locators and the lettered record series — both of which this report already tabulates in full above. A locator outside those two enumerations is some other document's numbering and is not this corpus's debt.

*Word-shaped section marks.* The token is the mark, then optionally one space, then a run that the existing locator reader rejects and whose first character is an ASCII letter. This is deliberately the exact complement of the section rule over the same mark: the two together select every occurrence of the mark in the declared surfaces, with no occurrence in both and none in neither. Zero is reachable for the pair because numbering carries no identity anywhere (`dec:migration:superseded-forms`), so a mark surviving in Assayer prose is debt whatever follows it.

One boundary is shared and must be decided once. The retired convention wrote ranges and lists — a doubled mark introducing several locators, and locators joined by commas, solidus or a dash — and it wrapped them across comment lines. A mark and document token standing at the end of one comment line with the locator on the next is **one** reference: a recognizer that read each line alone would report a word-shaped mark on the first line and a bare locator on the second, counting one reference twice and offering two replacements where one is owed. The recognizer must therefore resolve comment leaders away and read the comment as one region, exactly as (`conv:migration:burn-surface-reading`) already requires. Exactly one site in the corpus turns on this, and it is named in the census below.

### Census · `sec:assayer:reference-burn-residual-census`

Bare lettered locators divide by whether they have a head, and the division is mechanical: a locator is a *continuation* when the text before it, with one comment-line wrap resolved away, is a list separator preceded by another locator or record reference, or is a mark and document token; otherwise it is an *orphan*. The division decides the replacement, so it is drawn here rather than left to the sweep.

| Sub-shape | Count | What it is |
| --- | ---: | --- |
| Work-package number | 45 | A retired plan's deliverable number, mostly in module-doc provenance lists |
| Locator, continuation | 25 | A range or list sibling sharing the head the section or record rule already read |
| Locator, continuation across a wrap | 1 | The head is a mark and document token ending the previous comment line |
| Locator, orphan | 18 | A lettered record or chapter locator standing alone with its prefix or mark gone |
| Word-shaped mark, document token | 25 | A mark and document token whose locator has gone |
| Word-shaped mark, ordinal placeholder | 3 | The literal meta-notation of a convention, copied as though it were a locator |
| Word-shaped mark, named division | 5 | The mark introducing a named rather than numbered division |
| **All** | **122** |  |

The single wrap-joined site is ``packages/assayer/src/tests/resonance_crossover.rs``, whose mark and document token end one comment line and whose locator opens the next. It is the one occurrence that a line-at-a-time recognizer would double-count.

The per-file census is the exact file assignment for the sweep.

| Wave | File | Work package | Locator | Mark | Total |
| --- | --- | ---: | ---: | ---: | ---: |
| B2 | ``packages/assayer/src/api/health.rs`` | 1 | 0 | 0 | 1 |
| B2 | ``packages/assayer/src/api/label.rs`` | 1 | 0 | 0 | 1 |
| B2 | ``packages/assayer/src/api/lifecycle.rs`` | 1 | 0 | 0 | 1 |
| B2 | ``packages/assayer/src/api/preseed.rs`` | 1 | 0 | 0 | 1 |
| B2 | ``packages/assayer/src/api/report.rs`` | 1 | 0 | 0 | 1 |
| B4 | ``packages/assayer/src/extraction/mod.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/cell_state.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/competitive.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/dimension.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/maintenance_loop.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/mod.rs`` | 0 | 1 | 1 | 2 |
| B5 | ``packages/assayer/src/identity/observation.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/identity/snapshot.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/cell_set.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/entry.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/gc.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/mod.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/routing.rs`` | 0 | 0 | 1 | 1 |
| B5 | ``packages/assayer/src/ledger/sentinel_ledger.rs`` | 0 | 1 | 1 | 2 |
| B6 | ``packages/assayer/src/owner/commands.rs`` | 0 | 1 | 0 | 1 |
| B6 | ``packages/assayer/src/owner/label_path.rs`` | 8 | 0 | 0 | 8 |
| B6 | ``packages/assayer/src/owner/thread.rs`` | 0 | 0 | 1 | 1 |
| B7 | ``packages/assayer/src/pending/entry.rs`` | 0 | 2 | 0 | 2 |
| B7 | ``packages/assayer/src/persistence/checkpoint.rs`` | 1 | 13 | 0 | 14 |
| B7 | ``packages/assayer/src/persistence/journal.rs`` | 2 | 0 | 0 | 2 |
| B7 | ``packages/assayer/src/snapshot/published.rs`` | 0 | 2 | 0 | 2 |
| B7 | ``packages/assayer/src/snapshot/working.rs`` | 0 | 5 | 1 | 6 |
| B8 | ``packages/assayer/src/health/blend_stats.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/health/counters.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/health/published.rs`` | 2 | 1 | 0 | 3 |
| B8 | ``packages/assayer/src/health/summary.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/metrics/catalog.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/metrics/mapper.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/metrics/mod.rs`` | 1 | 0 | 0 | 1 |
| B8 | ``packages/assayer/src/report/index.rs`` | 0 | 2 | 0 | 2 |
| B8 | ``packages/assayer/src/report/mod.rs`` | 0 | 0 | 2 | 2 |
| B8 | ``packages/assayer/src/report/slot.rs`` | 0 | 1 | 0 | 1 |
| B8 | ``packages/assayer/src/testing/reports.rs`` | 0 | 0 | 1 | 1 |
| B9 | ``packages/assayer/src/resonance/channel.rs`` | 0 | 1 | 0 | 1 |
| B9 | ``packages/assayer/src/resonance/derivation.rs`` | 0 | 3 | 0 | 3 |
| B9 | ``packages/assayer/src/risk/calibration.rs`` | 2 | 1 | 0 | 3 |
| B9 | ``packages/assayer/src/risk/challenge.rs`` | 0 | 1 | 0 | 1 |
| B10 | ``packages/assayer/src/tests/builder.rs`` | 0 | 0 | 2 | 2 |
| B10 | ``packages/assayer/src/tests/label_pipeline.rs`` | 0 | 1 | 1 | 2 |
| B10 | ``packages/assayer/src/tests/lifecycle.rs`` | 0 | 0 | 1 | 1 |
| B10 | ``packages/assayer/src/tests/lifecycle_integration.rs`` | 8 | 0 | 0 | 8 |
| B10 | ``packages/assayer/src/tests/persistence.rs`` | 1 | 1 | 0 | 2 |
| B11 | ``packages/assayer/src/tests/api_surfaces.rs`` | 6 | 3 | 0 | 9 |
| B11 | ``packages/assayer/src/tests/assess_pipeline.rs`` | 0 | 0 | 1 | 1 |
| B11 | ``packages/assayer/src/tests/helpers.rs`` | 1 | 0 | 0 | 1 |
| B11 | ``packages/assayer/src/tests/mod.rs`` | 0 | 1 | 2 | 3 |
| B11 | ``packages/assayer/src/tests/model_bayesian.rs`` | 0 | 1 | 1 | 2 |
| B11 | ``packages/assayer/src/tests/resonance_crossover.rs`` | 0 | 1 | 1 | 2 |
| B11 | ``packages/assayer/src/tests/resonance_robustness.rs`` | 0 | 0 | 1 | 1 |
| B11 | ``packages/assayer/src/tests/resonance_tags.rs`` | 0 | 0 | 1 | 1 |
| B11 | ``packages/assayer/src/tests/source_audit.rs`` | 1 | 0 | 0 | 1 |
| B12 | ``packages/assayer/tests/multi_channel.rs`` | 0 | 0 | 2 | 2 |
| — | ``packages/assayer/ci/lint_assayer.sh`` | 1 | 0 | 0 | 1 |
| — | ``packages/assayer/docs/plans/adr-audit.md`` | 0 | 1 | 0 | 1 |
| — | ``packages/assayer/docs/plans/conformance-audit.md`` | 1 | 0 | 0 | 1 |
| — | ``packages/assayer/src/tests/resonance_exploration.rs`` | 0 | 0 | 1 | 1 |
| **All** | **61 files** | **45** | **44** | **33** | **122** |

Four rows stand outside the campaign's existing wave table and are worth naming. ``packages/assayer/src/tests/resonance_exploration.rs`` is a file that did not exist when the census above was taken. ``adr-audit.md`` and ``conformance-audit.md`` are planning documents that carried none of the three original shapes and so never entered the wave table. ``packages/assayer/ci/lint_assayer.sh`` stands outside every surface any burn list declares: the family's reach is the register's to name (`conv:migration:burn-surfaces`), and a shell script carrying a provenance comment is either inside the reach or is deliberately outside it. The machinery bite must rule, and the recommendation here is to include it, because the occurrence is a reference the corpus makes and its exclusion would be an artefact of the surface list rather than a judgment about the form.

### Classification · `sec:assayer:reference-burn-residual-classification`

The rule mirrors the campaign's own and is applied in this order.

1. A continuation locator resolves **with its head, not beside it**. The head and its siblings are one reference to one or more divisions; the sweep selects the label matching each gloss, and where one label already covers the whole range the siblings disappear into it rather than acquiring citations of their own. Replacing a range with a parenthesis of every label under it is the error (`sec:assayer:reference-burn-records`) already warns against.
2. An orphan locator resolves as the lettered-record rule resolves its own forms: the filename is not the replacement, and the sweep cites the one existing environment label stating the gloss at the use site.
3. A word-shaped mark carrying a document token resolves to the label of what the token names, when the surrounding sentence names a real target. Where the mark introduces a local ordinal or a named heading with no referent, the mark is removed and the descriptive heading retained — the de-ordinalization the campaign already applies, and for the same reason: manufacturing a citation for a heading that points nowhere would be a mint and a category error.
4. A work-package number is **attributive** in nearly every occurrence: a provenance list saying which work package a module was built under makes no claim that a label could carry, and the campaign has no environment for "this module was built under that plan". Plain de-numbering is therefore the ordinary action — the number goes and the descriptive phrase stays. Where a sentence instead uses the number to make a claim about behaviour, the claim's own label is the replacement.
5. No occurrence is silently deleted. De-numbering is not deletion: the sentence survives with its number gone, and where nothing survives, the occurrence is a hard case rather than a line to remove.

### Interaction with the waves · `sec:assayer:reference-burn-residual-waves`

Residual litter takes **one dedicated wave** rather than a share of each existing one, and the census is what settles it. Five of the sixty-one files belong to B2, a wave that has already run to zero on all three original families; folding residual litter into the existing table would reopen a closed wave and put its gate in the odd position of failing a wave that had passed. Four more files are in no wave at all. The remaining files spread thinly across B4 to B12 at one to fourteen occurrences each, which is not enough anywhere to be worth the coordination cost of amending nine wave definitions.

The wave is therefore **B13**, and it is gated behind its own adoption bite in the same order the campaign already uses: no wave may sweep a family before the recognizer and register that count it exist, because the ratchet is what makes the sweep verifiable. B13's file set is exactly the census table above, and its expected delta is 122 to zero with no other family moving.

| Wave | Shape | Files | Litter delta | Wall budget | Judgment |
| --- | --- | ---: | ---: | ---: | --- |
| B0R | Recognizer, declaration, register, tests | — | 0 | 1.5–2 h | Judgment-bearing |
| B13 | Residual litter across the corpus | 61 | 122 | 2–3 h | Mixed; the continuation locators require review, de-numbering is mechanical |

B13 does not change the pre-wave-10 gate's shape, but it does change its content: the gate becomes eight families reading zero rather than three, and B13 joins B12 as a wave that must land before it.

## Burn mechanism and proposed entries · `sec:assayer:reference-burn-mechanism`

The mechanism is governed by the repository's migration-disciplines record. A declaration in the linter's burn module names the family, its one recognizer, register, prose surfaces, and code surfaces, and a sibling legacy module implements the section and record recognizers. A read-only burn run scans the declared surfaces and verifies the generated rows exactly. Growth fails at the new occurrence; a stale row also fails after removal; ``--write`` regenerates only the marked region. Closing a burn requires zero occurrences, an empty generated register retained as a regrowth gate, and the normal checker clean (`conv:migration:burn-one-recognizer`).

Two campaign entries already exist and should not be broadened:

- Keep ``section-sign references`` exactly as ``Shape::Legacy(&[LegacyRule::SectionNumber])`` with its existing register and Assayer prose/code surfaces. It already counts all 1,858 participating forms.
- Keep ``retired record numbers`` exactly as ``Shape::Legacy(&[LegacyRule::RecordNumber])``. Its positive series grammar is ``L/M/R/S`` and therefore cannot flag canonical root ``T`` forms.

Add one declaration, rather than weakening either existing family:

```text
family: "ambiguous unprefixed record numbers"
shape: Shape::Legacy(&[LegacyRule::UnprefixedRecordNumber])
register: the unprefixed-record-number burn register
prose: ASSAYER_PROSE
code: ASSAYER_CODE
```

The new recognizer must match a token-boundary ``ADR-`` followed immediately by exactly three decimal digits and a non-digit boundary. It must not accept a series letter, so canonical root forms are excluded by construction rather than by file exception. The existing global exclusions remain the linter's own package and this package's burn-register directory. Adoption also needs the new rule wired through finding/reporting, adoption policy, unit/corpus tests, and the new generated register. The expected initial count is seven in one file; closure leaves the declaration and empty register in place.

That declaration landed and has since closed, and the campaign has closed with it: every one of this package's burn lists reached zero, the register documents that presented them were removed once nothing was left to enumerate, and the declarations themselves were retired, an undeclared pair being the statement that the policy no longer applies to this owner. The exhibit above therefore names a register that no longer exists, and is kept as the record of what was proposed. The retirement it proposed is now written into the root record rather than proposed in a report. The residual-litter family needs a declaration of the same shape, but it cannot simply be added beside this one, because two of its three shapes widen what the shared lint reads rather than declaring a private scheme. What that costs, and the amendment it is owed, are in (`sec:assayer:reference-burn-residual-retirement`).

## Sweep waves · `sec:assayer:reference-burn-waves`

Wave B0 adopts the missing recognizer/register and refreshes the two existing register narratives. B1 through B12 are the corpus waves in the full census table. Each wave is one commit-sized bite; file lists are exact there and totals are exact here.

B0's exact seven files are four in the linter's own package — its legacy and burn modules, its adoption data, and its corpus test — together with the new unprefixed-record-number register and the existing section-reference and record-number registers whose stale handwritten census summaries must be corrected. The existing generated rows do not change in B0.

| Wave | Shape | Files | Section delta | Lettered delta | Bare delta | Wall budget | Judgment |
| --- | --- | ---: | ---: | ---: | ---: | ---: | --- |
| B0 | Linter rule, tests, declaration, generated register | 7 | 0 | 0 | 0 | 1.5–2 h | Judgment-bearing |
| B1 | Backlog, audit plans, upstream API map | 4 | 225 | 22 | 7 | 4–6 h | Judgment-bearing; includes all hard cases |
| B2 | Public API, assessment, config, root types/errors | 14 | 212 | 209 | 0 | 6–8 h | Judgment-bearing core contract |
| B3 | Numerics, linear algebra, model | 12 | 70 | 95 | 0 | 2–3 h | Judgment-bearing mathematics |
| B4 | Extraction and feature assembly | 14 | 128 | 35 | 0 | 2–3 h | Mixed; mechanical after label selection |
| B5 | Identity, ledger, signal | 17 | 95 | 54 | 0 | 2–3 h | Mixed; includes upstream claims |
| B6 | Owner loop and label path | 5 | 104 | 129 | 0 | 4–5 h | Judgment-bearing ordering |
| B7 | Pending, persistence, snapshots | 12 | 81 | 87 | 0 | 2–3 h | Mixed durability semantics |
| B8 | Guidance, health, metrics, reports, test helpers | 26 | 158 | 131 | 0 | 4–6 h | Judgment-bearing health split |
| B9 | Risk and resonance | 9 | 157 | 50 | 0 | 3–4 h | Judgment-bearing formulas |
| B10 | Stateful crate tests | 15 | 258 | 55 | 0 | 4–6 h | Mechanical once B2–B8 land |
| B11 | Remaining crate tests | 24 | 157 | 38 | 0 | 3–4 h | Mostly mechanical |
| B12 | Integration tests and support | 22 | 213 | 15 | 0 | 3–5 h | Mostly mechanical; scenario glosses require review |
| **Total corpus** |  | **174** | **1,858** | **920** | **7** | **39.5–55 h** |  |

The gate for every wave is the normal checker plus read-only ``burn --root .``. The named family counts must fall by exactly that wave's deltas, no other burn may grow, generated rows must be neither stale nor manually edited, and the 58 canonical root forms must remain 58. B10–B12 are small-model eligible after the source waves have fixed the labels; B1–B3, B6, B8, and B9 require semantic review. B4, B5, and B7 can be split by the same table if a judgment case blocks an otherwise mechanical file. After B12 the three campaign families read zero; this is the required pre-wave-10 gate.

Two clauses of that gate have been restated and the restatement governs. The per-wave deltas above are the design figures; what each wave must actually clear is in (`sec:assayer:reference-burn-restated-waves`), and B1 to B3 have run while B4 and B6 are part-run, so a gate set from the table above would now fail on work already done correctly. The root-form clause is restated in (`sec:assayer:reference-burn-restated-root-gate`): the count is 67 rather than 58, and the clause is a rule about direction rather than a fixed number, because the corpus gains root forms whenever a document legitimately cites a root record. The pre-wave-10 gate also now includes B13 and reads eight families at zero rather than three.

## Recording the residual-litter retirement · `sec:assayer:reference-burn-residual-retirement`

A burn family is a count; a retirement is a recorded choice. The campaign's seven declared families record theirs in two different places, and which place a new family belongs in is not a matter of taste. This section reads the precedent, applies it, and stops where it reaches a root record.

### What the seven families actually do · `sec:assayer:reference-burn-retirement-precedent`

Every family, without exception, states *what it counts* in its own register preamble, as a local specialization of (`conv:migration:burn-family`) naming the bound — the interval for a family bounded by range, the sentences for one bounded by enumeration. That much is uniform and belongs to the machinery bite that creates the register.

Where the families divide is on *that the form is retired at all*.

Four of them — the section marks, the lettered records, the tag forms and the unprefixed record numbers — are rules of the shared checker's migration lint, and their retirement is recorded in the root record's (`dec:migration:superseded-forms`). That record opens by saying the calculus supersedes four reference forms and closes by saying a fifth rule would be a fifth retirement on the same terms. The count is written into the record deliberately: the record says that the value grows only by decision, and that this is the point of writing it down there.

The other three — scenario numbers, division names and unlabelled to-do notices — have no lint rule. They are one retired document's private identity scheme, or a profile's remainder, and their retirement lives in the owner's own decision and in the register preamble beside the bound. No root record was touched to create any of them, and none needed to be: a family with no corresponding lint rule declares its own recognizer beside its register.

### Which precedent residual litter falls under · `sec:assayer:reference-burn-retirement-applied`

The answer splits across the family's three shapes, and the split is the substantive finding of this section.

**Work-package numbers follow the second precedent.** The form is one retired plan's private numbering, bounded by enumeration over the eight values that plan carried. It is the scenario-number family's situation exactly. Its retirement is recordable in the register preamble the machinery bite writes, together with the decision that put it there, and no root record is involved.

**The other two shapes follow the first, and that is where this lane stops.** A bare lettered locator is not a new naming scheme: it is the third rule's own form with the record prefix removed. A word-shaped mark is not a new naming scheme either: it is the second rule's own mark with the number gone. A recognizer that selected them would not be declaring a private scheme beside a register — it would be widening what the shared migration lint reads, which is the one thing (`dec:migration:superseded-forms`) reserves to itself.

The root record has already faced this exact question once and answered it, and its answer is why this cannot be done quietly. When the unprefixed record number needed covering, the record considered widening the third rule to reach it and refused, on the ground that a rule reading the series letter would, if widened to accept a number carrying none, flag the canonical form the corpus keeps. It made a fourth rule instead. The same reasoning applies here with the same force: widening the section rule to accept a word would make it read the mark wherever it stands, and widening the record rule to accept a prefixless locator brings it within one character of the chapter locators of every corpus that still numbers its chapters. Both need bounds, and a bound is exactly what a record records.

**This lane therefore does not amend the root record.** A root record is amended deliberately, by its owner, and not as a consequence of a report about a corpus that imports it. The amendment this family needs is proposed below, and it is owed a ruling before the recognizer bite can start on shapes two and three. The work-package shape is unblocked and can proceed without it.

### The proposed amendment, and the decision it awaits · `sec:assayer:reference-burn-retirement-proposal`

The amendment is to (`dec:migration:superseded-forms`). It does not add a rule; it records that two rules already taken reach the degraded spellings of their own forms, and it supplies the bound that keeps each finite. Stated as an addition to that decision, after its fourth rule:

> *Forms degraded past their own recognizer.* A retirement reaches the shape it retired, and not merely the spelling a recognizer happened to read first. Two of the rules above have a degraded spelling that carries the same reference while falling outside the rule's own grammar, and both are retired on the same terms as the form they degrade from.
>
> A locator sign introducing a word rather than a number is the second rule's own mark with its number removed — by a sweep that took the number and left the sign, or by a convention that wrote the sign before a named division. The sign is what makes a reference a pointer, so a sign pointing at nothing is the retired form in its last stage rather than a different form. The rule reaches it, and the two readings together exhaust the sign: every occurrence of the sign in a counted surface is one or the other, which is what keeps zero reachable for the pair.
>
> A record locator standing without its record prefix is the third rule's own form with the prefix removed. This one is bounded and the bound is load-bearing: the rule reaches such a locator only where its number is one the retiring corpus actually issued, enumerated in the register. Without that bound the rule would read every hyphenated letter-and-number in the corpus, and a corpus that still numbers its own divisions would find its numbering counted as another's debt — which is the reach the bounded-family convention already refuses.
>
> Neither is a fifth retirement. A fifth rule would still be a fifth retirement on the terms above; these two are the second and the third, stated to the edge of what they retired.

Two things about the shape of this proposal are deliberate. It is written as a clause of the existing decision rather than as a new rule, because the honest description of these forms is that they are the same retirement reaching further, and a corpus that counted them as a new family would be recording a choice nobody made. And it supplies the bound in the same breath as the reach, because the record's own refusal of the unbounded family is the standard any widening has to meet.

**Ruled: the proposal above is superseded, and no record is amended.** The amendment was not taken, and the ground is that the reading was the defect rather than the rule: the linter should track all these forms, including across a comment line boundary, which is a defect in the linter's tokenization layer and not in the policy the record states.

The proposal stands above rather than being deleted, because this document keeps the reasoning a ruling resolves. Its substantive argument survives the ruling: a degraded spelling belongs to the family whose form it degrades, and that is how the linter now reads the mark. What it got wrong is where the remedy lay. (`dec:migration:superseded-forms`) is unamended, the migration lint's policy is the four rules it always had, and no family was added.

What changed is the tokenization layer. Every recognizer is now handed a run of adjacent comment lines joined into one region with the leaders resolved away — the reading this family's own recognizer already had and the two shared ones did not — so a reference the retired convention wrapped across a comment line boundary is one reference to every rule that reads its shape. And the mark has one reader, exhaustive by construction: a companion continuing the run of marks that opened the reference before it, a section reference, a word-shaped mark, or nothing.

One consequence deserves stating plainly rather than being left to be noticed. The section rule's reach did grow by one spelling: a heading quoted after the mark is now a section reference where before it was read by nothing. That is the ruling's own content rather than a liberty taken beside it — it is the second rule reading its own form's degraded spelling — and it was made without amending the record, on the ruling that this is what the rule already retired. The doubled mark is not a widening at all: its companion already carried the reference the section rule read, and what changed is that the pair opens one reference at its first mark instead of one at its second.

The order below is answered as far as a ruling can answer it. Steps 1 and 2 stand as written: the register is declared and counts what stands, and the sweep that empties it is still owed. Step 3 is answered — the amendment was put and refused. Step 4 does not arise, because it was conditional on an amendment that was not made; shapes two and three stay in this family's own ratchet, judging nothing, exactly as that family's recogniser convention said while its register stood. Repairing the layer made one occurrence newly visible, a reference wrapped across a comment line boundary in ``packages/assayer/src/tests/resonance_crossover.rs``, and it is registered in the section-reference list rather than swept: making debt visible and burning it are two different lanes.

### The order the work should land in · `sec:assayer:reference-burn-retirement-order`

The root record models the ordering, in its own account of how its fourth rule arrived: the register counted the shape while the retirement was unwritten, the register reached zero, the retirement was written down, and only then did the rule join the lint's policy. That order is not an accident of history, and it is the order this family should follow.

1. Declare the family and its register, and let it census what stands. A register may carry occurrences the moment it is declared; that is the normal case rather than an irregularity.
2. Sweep B13 and bring the register to zero.
3. Take the amendment above to a recorded decision, and record the retirement if it is ruled.
4. Only then let the recognizer join the shared lint's policy for shapes two and three.

Adding the rule to the policy first would hold every document in the corpus to a rule no record states. Adding it while occurrences still stand would fail the documents carrying them for a rule that arrived after they were written. Both are the failures the discipline exists to prevent, and the campaign has already paid for learning them once.

## Restated figures · `sec:assayer:reference-burn-restated`

Every figure above this section was measured on draft ``bd904945`` and is kept as the historical record of what the campaign found when it was designed. The corpus has moved since: three waves have run, the linter has grown three families the report never saw, and documents have been written. This section restates the figures that a reader would otherwise act on, each marked with the commit it was measured at, and it is the section to trust where the two differ. All figures below are as of draft ``fe59a601``, taken from the live registers rather than from any handwritten summary (`req:migration:burn-ratchet`).

### The declared families · `sec:assayer:reference-burn-restated-families`

The report describes a linter carrying six declared burns and 2,891 occurrences, and proposes a seventh. Seven are now declared and the two the report proposed to leave alone have both fallen a long way.

| Family | Occurrences | Files | Standing |
| --- | ---: | ---: | --- |
| section-sign references | 1,342 | 140 | Falling; B1–B3 run, B4 and B6 part-run |
| retired record numbers | 588 | 106 | Falling; same waves |
| ambiguous unprefixed record numbers | 0 | 0 | Closed by B1; register kept as a regrowth gate |
| superseded tag forms | 7 | 4 | Not a campaign family; four planning documents |
| retired scenario numbers | 99 | 24 | Not a campaign family |
| retired division names | 7 | 4 | Not a campaign family |
| unlabelled to-do notices | 0 | 0 | Empty; a profile remainder rather than a reference form |
| **All declared** | **2,043** | | |
| residual litter (proposed) | 122 | 61 | Not yet declared; this report's new family |

The report's proposed seventh entry landed and did its work: the seven bare forms it counted are gone, the register is empty, and the retirement is now written into the root record rather than proposed in a report. Three further families arrived that the report never anticipated, none of them a reference form of the kind the campaign was designed around.

Two of the report's premise-audit statements have expired and are corrected here. The register footers it called stale — 1,860 and 931 against generated sums of 1,858 and 920 — now read 1,342 over 140 files and 588 over 106 files, which are the generated sums exactly; the defect was repaired rather than merely noted. And the "six declared burns and 2,891 total occurrences" it recorded as exact at its base is now seven and 2,043.

### The remaining waves · `sec:assayer:reference-burn-restated-waves`

The per-wave deltas in (`sec:assayer:reference-burn-waves`) are the campaign's design figures at ``bd904945``. What each wave must actually clear is below. No file has entered either original family since the census was taken, so the wave table's file assignment still covers both families entirely; only the counts moved.

| Wave | Files | Section, designed | Section, live | Lettered, designed | Lettered, live |
| --- | ---: | ---: | ---: | ---: | ---: |
| B1 | 4 | 225 | 0 | 22 | 0 |
| B2 | 14 | 212 | 0 | 209 | 0 |
| B3 | 12 | 70 | 0 | 95 | 0 |
| B4 | 14 | 128 | 124 | 35 | 34 |
| B5 | 17 | 95 | 95 | 54 | 54 |
| B6 | 5 | 104 | 99 | 129 | 124 |
| B7 | 12 | 81 | 81 | 87 | 87 |
| B8 | 26 | 158 | 158 | 131 | 131 |
| B9 | 9 | 157 | 157 | 50 | 50 |
| B10 | 15 | 258 | 258 | 55 | 55 |
| B11 | 24 | 157 | 157 | 38 | 38 |
| B12 | 22 | 213 | 213 | 15 | 15 |
| **Total** | **174** | **1,858** | **1,342** | **920** | **588** |

B1, B2 and B3 have run to zero on both families. B4 and B6 are part-run — five occurrences and ten respectively have already gone, swept as look-alikes by neighbouring lanes rather than by their own wave — so their gates must be set from the live column, not the designed one. A wave whose gate demanded its designed delta would now fail on work already done correctly.

### The canonical root-form gate · `sec:assayer:reference-burn-restated-root-gate`

Every wave gate in this report ends with the clause that the 58 canonical root forms must remain 58. The figure is stale and the clause is also mis-stated, and both are worth fixing because this gate is the one that protects the owner's only exception from the sweep that surrounds it.

The live count is **67**, and it is reached from the report's 58 by two additions that are both correct.

| Step | Change | Running total |
| --- | ---: | ---: |
| The audit at ``bd904945`` | — | 58 |
| B1 converted the seven bare forms to canonical root links | +7 | 65 |
| Two reports written since, each citing one root record | +2 | 67 |

The first step is the campaign's own replacement map executed. All seven bare forms stood in ``packages/assayer/docs/plans/backlog.md``, which held 29 root forms and seven bare ones at ``bd904945`` and holds 36 root forms and none now. The seven land as two more citations of a root record the file already cited, two of a root record it had not cited at all, and one each of three further root records, which is exactly the map (`sec:assayer:reference-burn-records`) laid out. A citation converted from an ambiguous form to a canonical one is the campaign working, so the gate must be read as permitting this rise rather than as having been broken by it.

The second step is two documents that did not exist at ``bd904945``: ``packages/assayer/docs/reports/platt-threshold-sensitivity.md`` and ``packages/assayer/docs/reports/stationary-drift-bound.md``, each citing the label-calculus record once. That is the whole of the difference: of the fifteen root records cited in the corpus, the label-calculus record is the only one whose count differs from the report's audit plus B1's conversions, and it differs by exactly two.

The arithmetic is checkable two ways. A participation-aware count over ``packages/assayer`` returns 67; a raw text search returns 86, and the 19-form difference is this report's own exhibits, which stand in double-backtick spans and so are displayed rather than cited (`judg:labels:participation`).

The clause itself should be restated as a rule rather than a number, because a number is the wrong thing to gate on here. A root form is the owner's exception, and the corpus acquires more of them whenever a document legitimately cites a root record — which is a thing the campaign wants, not a regression. What the gate exists to catch is a sweep that *converts or destroys* one, since the recognizers surrounding it read forms that differ from it by a single letter. The clause to carry into each wave is therefore:

> No sweep may reduce the canonical root-form count, and any rise must be attributable to a citation deliberately written. The count stands at 67 as of draft ``fe59a601``.

## Premise audit · `sec:assayer:reference-burn-premises`

- “About 65 numbered ADR occurrences” is exactly 65 raw root/bare occurrences, but only seven are litter. Adding the current 920 lettered burn occurrences makes the campaign's numbered-record debt 927.
- The stated top forms are exact: the environment-kinds registry is 18, the label calculus is 15, and the test-label profile is six. They are all legitimate root forms, not burn debt.
- The section-mark premise was low: the live linter count is 1,858, not about 1,734. It spans 165 files before the bare-record-only backlog row is joined.
- A section mark does not always point at specification numbering. Only 655 do. Another 353 point into retired records, five point upstream, six point into a current local outline, and 839 are local ordinals or auxiliary-document references. Some counted forms therefore point nowhere.
- The check report's six declared burns and 2,891 total occurrences are exact at base. The two relevant live family counts are 1,858 and 920. The prose totals in their register footers are stale (1,860 and 931 respectively); generated row sums and the burn command are authoritative.
- The record register describes four “Assayer” series, but its grammar is actually ``L/M/R/S``. Three live occurrences are Mudlark numbers. The owner's only-root exception resolves them as burnable here; if the owner intended all reachable owners' canonical record numbers to stay, that premise needs a new ruling before B0.
- No active participating record reference uses the ``R`` or ``S`` series. Their appearances in this report are exhibits.

The residual-litter census added three premise findings of its own, all as of draft ``fe59a601``.

- The dispatch figure for work-package numbers was 43 in 23 files. The file count is right and the occurrence count is not: 43 is the number of *lines* carrying the form, and two lines carry two occurrences each, so the census the burn oracle would report is 45. The distinction matters beyond arithmetic, because a register counts occurrences and a ratchet set from a line count would pass a file that grew a second occurrence on a line it already held.
- The repository's own agent instructions, ``AGENTS.md``, still prescribe the forms this campaign retires. They document the mark-and-qualifier reference form, the doubled mark for ranges, and the bare mark-and-ordinal as acceptable within a document that establishes context — which is where the corpus's ordinal placeholders come from. The file stands at the repository root, outside every surface any burn list declares, and it carries two participating section references of its own in its worked examples. The campaign cannot reach a stable zero while the document instructing new writing teaches the retired forms, and this is a repository-level finding rather than something a report about one package should edit.
- The three shapes are not confined to this corpus. A word-shaped mark occurs 79 times outside Assayer, in four of the repository's other corpora, where it is those corpora's live convention rather than anyone's debt. The family's reach is named by its register and must stop at Assayer (`conv:migration:burn-surfaces`): a census counting a scheme nobody has retired would be counting the wrong thing, and could not reach zero without rewriting corpora that never joined the migration.

## Verification record · `sec:assayer:reference-burn-verification`

| Gate | Result | Wall time |
| --- | --- | ---: |
| Base checker, before writing | schema 13; 349 sources; 2,748 mints; 13,018 resolved citations; 2,748 heads; 0 failures; 0 warnings; six burns, 1,968 files scanned, 2,891 occurrences | 6.38 s |
| Skeleton checker, before early commit | schema 13; 350 sources; citation/head figures unchanged; 0 failures; 0 warnings; six burns, 1,973 files scanned, 2,891 occurrences | 6.25 s |
| Uncommitted full-draft checker | 121 failures: discussed labels were accidentally written as single-span mints and report section heads lacked identities. All were converted to exhibits or citations and all structural report heads labelled before any full-report commit. | 6.61 s |
| Complete-census checker, before report commit | schema 13; 350 sources; 2,759 mints; 13,022 resolved citations; 2,759 heads; 0 failures; 0 warnings; six burns, 1,973 files scanned, 2,891 occurrences | 5.99 s |
| Final checker on the committed census | schema 13; 350 sources; 2,759 mints; 13,022 resolved citations; 2,759 heads; 0 failures; 0 warnings; six burns, 1,973 files scanned, 2,891 occurrences | 6.00 s |
| Final read-only burn oracle | six declared families; section marks 1,858 in 165 files; lettered records 920 in 133 files; all 2,891 registered occurrences exact; 0 failures | 0.67 s |

The lane used no cargo and no network. The report checkpoints were ``034e9c1a report(assayer): begin the reference-burn campaign census`` and ``408b9742 report(assayer): map the reference-burn campaign``.

### The residual-litter amendment · `sec:assayer:reference-burn-residual-verification`

The amendment adding the eighth family was written on draft ``fe59a601``. It changed no source, register, recognizer or reference, so the burn oracle is the controlling gate: it must read exactly what it read at base, because a paper lane that moved a family count would have written a burnable form while describing one.

| Gate | Result | Wall time |
| --- | --- | ---: |
| Base checker | schema 13; 354 sources; 2,853 mints; 13,873 resolved citations; 0 failures; 0 warnings | 8.15 s |
| Base burn oracle | seven declared families; 2,043 occurrences | 0.99 s |
| Checker, per commit, four commits | 0 failures; 0 warnings; clean at every checkpoint | 7.66 s, 7.02 s, 6.72 s, 6.43 s |
| Burn oracle, per commit | 2,043 occurrences at every checkpoint, no family moved | under 1 s each |
| Recognizer port, against the live register | The census reader was written from the linter's own grammar and reconciled to it before any figure here was accepted: it reproduces the section family at 1,342 occurrences over 140 files, per file identical | 3.8 s |
| Own-mint check | The report writes zero occurrences of all four shapes — the three new ones and the section mark — and stands in no family's rows | — |

The port reconciliation is the reason the residual figures are quotable. A census that recognized the forms its own way would eventually disagree with the lint about what a reference is, so the reader used here was held to the live register first and only then pointed at the shapes no register counts. Two defects it found in itself before it was trusted are worth recording, because both would have inflated the census: an inline code span whose backtick run was measured against the wrong string, which admitted exhibits as references, and a surface model that read Markdown wherever it stood rather than only under the two prose trees the families declare.

### The census discharged · `sec:assayer:reference-burn-discharged`

The closing wave ran from draft ``cce9d76d`` and every declared family now reads zero. The figure worth recording against that zero is each register's own first census rather than this report's design counts, because a register's birth reading is what the ratchet undertook to bring down and is the only figure ever measured by the rule that afterwards enforced it.

| Family | At declaration | At the closing wave's base | Now |
| --- | ---: | ---: | ---: |
| section-sign references | 5,527 | 1 | 0 |
| retired record numbers | 3,944 | 0 | 0 |
| ambiguous unprefixed record numbers | 7 | 0 | 0 |
| superseded tag forms | 2,040 | 0 | 0 |
| retired scenario numbers | 268 | 0 | 0 |
| retired division names | 31 | 0 | 0 |
| residual litter | 96 | 57 | 0 |
| unlabelled to-do notices | 144 | 0 | 0 |
| **All eight** | **12,057** | **58** | **0** |

Two of those declaration figures differ from what this report states above, and the registers are right in both cases. The residual family is described here as 122 occurrences in 61 files, measured before any recognizer existed to count it; by the time the recognizer landed and the register was declared, the corpus had moved and the first generated census read 96 over 45 files. The section family is described here as 1,858, a design census taken by hand over a narrower reach than the rule the linter finally carried. Neither number was wrong when it was written and neither was ever the ratchet's, which is why every wave gate was set from the live registers instead.

The eighth family took the dedicated wave its census argued for, and the argument held. Five of its files belonged to a wave that had already run to zero, four stood in no wave at all, and the rest spread across nine waves at one to fourteen occurrences each — too thin anywhere to be worth amending nine wave definitions for, and impossible to fold into the first without reopening a closed gate. The closing wave swept the fifty-seven the register carried at its base together with the one section reference the repaired tokenization layer had made newly visible: the wrapped reference this report named in advance as the single site that turns on reading a run of comment lines as one region. It resolved to one citation and one replacement, which is what the caveat promised and what a line-at-a-time reading could not have produced.

What the zero means is what the residual family's own gate said it meant while its register stood: no document in the reach points with a sign that points nowhere, a number whose plan is gone, or a locator whose prefix has been taken. Nothing was deleted to reach it. The ratchets stay, empty, as the gates against regrowth; the retirements they were opened in order to record are the work that follows.
