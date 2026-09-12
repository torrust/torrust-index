# Torrust Assayer · `guide:assayer:bayesian-posture-engine`

An online-learning dynamically resizing linear risk estimation model for the Torrust Index.

## Overview · `sec:assayer:readme-overview`

The Assayer estimates risk from the measurement reports Sentinels produce, and learns online from the labelled outcomes the host reports back. Its dimension resizes as Sentinels register and deregister — a registration extends every model but the fixed-dimension anchor, a deregistration marginalises the same set — and what it produces is a risk estimate: the Assayer measures, and the decision on the estimate is the host's. The Core combines signals from multiple Sentinels into `RiskAssessment` values. Hosts may then call the pure Derivation Function, `derive_reckoning()`, to turn an assessment plus channel policy and Companion Tracker evidence into decision-layer tags.

## Architecture · `sec:assayer:readme-architecture`

The crate is organised around three runtime components:

- **Core** — stateful learning and channel-free risk assessment via `assess()`.
- **Derivation Function** — pure `derive_reckoning()` output from explicit host inputs.
- **Companion Tracker** — host-owned challenge-effectiveness estimation.

The Core maintains multiple Bayesian linear models that evolve over time:

- **Operational model (V)** — The primary model used for live decisions
- **Sister model** — A slower-decaying copy for drift detection
- **Anchor model** — A fixed-dimension reference for baseline comparison
- **Outcome-axis models** — Per-axis models for different signal types

All model updates occur on a dedicated model-owner thread, ensuring serialised writes. Readers access model state through lock-free snapshots via `arc-swap`.

## Development Status · `sec:assayer:readme-status`

**Layer 1 (Structural Commitments) is complete.** All seven load-bearing records are implemented and tested, together with three pulled forward from the layers above.

**Layer 2 (Representations) is complete.** All six work packages are implemented and tested, and the representation records are fully realised.

**Layer 3 (The Operational Cycle) is complete.** All runtime data-flow paths are implemented: `assess()` produces core risk assessments, `derive_reckoning()` provides pure decision-layer derivation from explicit inputs, `label()` processes the 17-step learning path, and `receive_sentinel_report()` validates, indexes, and maintains the Ledger. The operational-cycle records are fully realised.

**Layer 4 (Mathematical Foundations) is complete.** All six algorithm records are implemented: Sherman–Morrison update, Cholesky recomputation, Schur complement marginalisation, resonance derivation, Platt calibration, and blend weight computation. Additionally, outcome axis evaluation and drift accumulators are implemented.

**Layer 5 (Contracts and Boundaries) is substantially complete.** The `AssayerBuilder` pattern, the full `AssayerConfig` hierarchy, all registration types, Core `assess()`, host-side `derive_reckoning()`, `label()`, `receive_sentinel_report()`, health queries, lifecycle methods, and metrics catalog are implemented.

The crate compiles and the Layer 1–5 implementation surfaces are in place.

What is functional:

- **Model infrastructure** — `BayesianLinearModel` with create, extend, marginalise, serialise, reconstruct; `SymmetricMatrix` wrapper with full operation surface; `faer` bridge with Cholesky (plain + regularised), inverse, solve
- **Concurrency** — `ArcSwap` snapshot and report-index publication (structural lock-free reads); model owner thread with dual channels and drain-before-process; bounded Outcome Ledger reads; checkpoint + journal with atomic checkpoint and crash-safe recovery
- **Data structures** — error model with all public error enums and `DegradationContext`; shared types with all ID newtypes, timestamps, decay functions; signal layer with shapes, encoding, schema layout, LRU cache
- **Report ingestion** — `ReportIndex` with depth-walk routing; `SentinelSlot` with bootstrap accumulator; Outcome Ledger with EWMA tracking and cell-set maintenance
- **Identity** — G-V Graph instances with dedicated maintenance thread; competitive set change detection; cell outcome state with observation channels
- **Pending buffer** — `DashMap` + FIFO with capacity + expiry eviction; full pending assessment/context entry types
- **Snapshot** — all `ModelSnapshot` fields typed and populated; `DimensionMap` with structural index tracking
- **Assessment and derivation path** — feature assembly, blend weight computation, channel-free risk assessment, pure resonance derivation, outcome axis evaluation
- **Label path** — Sherman–Morrison update with leverage bounds, Cholesky recomputation, Platt calibration, drift accumulators
- **Lifecycle** — Schur complement marginalisation with information preservation
- **Construction** — `AssayerBuilder` with eager validation, `AssayerConfig` hierarchy, registration types

Current open follow-up work is tracked in the [deferral register](docs/deferred.md); the documentation campaign that governs this corpus is tracked in its [backlog](docs/plans/backlog.md), whose charter is (`sec:assayer:charter`).

See the [record index](docs/adrs.md) and the [layer documents](docs/layers/) for current architecture details.

## The area register · `sec:assayer:area-register`

Every claim this package mints names an area, and this register says what each area is for. The requirement it answers is (`req:testdocs:area-register`): an owner minting claims carries one register in its own prose, an entry per area, whose head prose states the stake — what is lost if claims of this area fail.

The stake is the register's whole reason to exist. How many areas are in use and how many statements stand in each is a census the linter takes; which files an area's claims live in is a query. Neither can compute what an area is for, and no individual claim states it, because a claim says what the system does rather than why anyone should care that it does it.

The vocabulary was not fixed in advance. It is what the statements turned out to want, censused after they were written and curated into the entries below, which is the order (`conv:testdocs:claim-areas`) fixes and the one thing about this register that would have been wrong the other way round. An area set chosen before the sentences existed would have been a taxonomy that had met no sentence.

Two of the entries are honest about being mixtures rather than pretending otherwise. The structural guarantees about the source tree and the timing floors under the operating cycle share one area on the ground that neither is visible from any single call site, which is a real property but a thin one; and the area covering whole-engine composition is held together by running everything at once rather than by a subject. Both are candidates for splitting when their claims grow enough to force it, and saying so here is cheaper than discovering it later.

An unregistered area is a report line rather than a finding, so a claim minted in a new area is admitted and counted, and this register is what catches up.

**Section (api)** · `sec:assayer:area-api`

If this fails: the published surface becomes something a host must probe at runtime rather than read once. A derived reckoning that is sometimes complete and sometimes not forces every caller to write conditional handling around fields that are in fact always there, and those conditionals harden into assumptions the engine can never afterwards revise. The area fixes what the public entry points hand back and what they refuse: what a reckoning always carries, what a request may omit, and which of the several consistent ways a condition is reported must agree with the others.

**Section (assess)** · `sec:assayer:area-assess`

If this fails: there is no such thing as what the engine believes about an entity — only what it believes given a routing decision the host has already made. Assessment stops being total, so an unidentifiable subject raises an error on the request-handling path instead of returning a prior-shaped answer, and the host builds a fallback risk model of its own to cover the gap. This is the hot path every request-handling thread runs, so what it records about the moment of estimation — which competitive cells were active, which measurement surfaces reported — is also the only context a label arriving later can be attributed against.

**Section (audit)** · `sec:assayer:area-audit`

If this fails: the crate's standing guarantees hold only by habit. The layering between the core model and the decision layer becomes a convention rather than a fact about the source tree, placeholder scaffolding regains a home to be written back into, and the timing floors under the operating cycle go unwatched until a host discovers them under load. These are properties of the artefact and of its speed that no behavioural test observes, because nothing at any one call site can see them.

**Section (bayes)** · `sec:assayer:area-bayes`

If this fails: the model no longer learns what the evidence says. A prior that starts committed, an extension that hands new coordinates borrowed correlation, or an incremental path that drifts away from the exact recomputation all produce the same outcome — a posterior shaped confidently by arithmetic rather than by outcomes. Nothing downstream can undo it, because nothing downstream knows the belief is wrong.

**Section (builder)** · `sec:assayer:area-builder`

If this fails: a host cannot know what it deployed. Shipped defaults that drift from the specification mean the documented system and the running system are two different systems, and a configuration round-trip that alters untouched fields turns the ordinary workflow — serialise the defaults, edit a handful of values, read the result back — into a source of changes nobody made. The area also fixes what a declaration costs: how signals and templates each widen the model, and that machinery a host did not ask for is never started.

**Section (channel)** · `sec:assayer:area-channel`

If this fails: the decision layer leaks back into the model. Two channels priced differently stop being two readings of one belief and become two beliefs, so a host can no longer assess once and spend that estimate across its login, API and transaction surfaces — it re-runs the core for each, and the estimates disagree for reasons that have nothing to do with the entity. What separates channels must be their declared actions and the prices they put on outcomes, and nothing else may cross.

**Section (dossier)** · `sec:assayer:area-dossier`

If this fails: the engine learns from geometry that was never reported. A batch is the only account the engine has of what a Sentinel saw, so a key that cannot recognise the same region across successive reports, a depth that wraps on the way into storage, or one misaligned cell admitted beside good ones corrupts regional history quietly: the numbers stay finite and plausible, and the region they describe does not exist. The quiet batch matters as much as the loud one — a Sentinel with nothing to say must be recorded as having reported, not as absent.

**Section (errors)** · `sec:assayer:area-errors`

If this fails: a host cannot tell retryable backpressure from accepted durable work. These failures cross thread boundaries — caller, model owner, maintenance side — so an error that cannot be sent between them is flattened into a string at the seam and loses the one distinction that decides what to do next: a full queue drains, a departed owner never will. A host that cannot separate the two either retries forever or discards work that would have landed.

**Section (extract)** · `sec:assayer:area-extract`

If this fails: the interpretation layer corrupts the measurement layer. A Sentinel's chain is reduced to the handful of indicators the model actually weighs, so an axis that borrows another's magnitude, a peak drawn from the wrong depth, or a quiet Sentinel rendered as a gap rather than as zeros each puts a number in front of the model that no observation produced — and every risk estimate downstream of it is suspect without ever looking wrong.

**Section (feature)** · `sec:assayer:area-feature`

If this fails: the model's columns stop meaning what they meant. Slots are addressed through a layout that is registered as the deployment grows, so an offset assumed rather than asked for, an occupancy indicator misplaced, or a completion boundary read one observation early gives the model coefficients learned against one column and applied to another's values. Standardisation and the interaction slots then compound the error instead of revealing it, because both read the vector by position too.

**Section (guidance)** · `sec:assayer:area-guidance`

If this fails: labelling effort is spent where it buys least. These scores decide which assessments a human is asked to adjudicate, so a ranking that ignores the doubt the model already recorded, or that chases regions merely for being unremarkable, leaves the uncertain estimates and the starved regions unlabelled — and a region starved of evidence keeps the very reputation that starved it.

**Section (identity)** · `sec:assayer:area-identity`

If this fails: the contamination loop wins. A region's opinions are earned only from outcomes actually attributed to it, so a freshly promoted cell that inherits a reputation, or a moving average that travels further on one outcome than its smoothing factor allows, lets a single adverse event fix that region's standing — which sustains restriction, which starves the evidence that would have corrected it. The area also holds the maintenance side, where that geometry is rebuilt: entities are identified along several independent axes at once, each must grow without disturbing the others, and a cut taken for a checkpoint must be the cut that was asked for.

**Section (labelling)** · `sec:assayer:area-labelling`

If this fails: one bad number from the host poisons the model instead of costing one observation. Values arriving at the label boundary are repaired per axis rather than refused wholesale, so a broken repair either discards multi-axis observations that were mostly sound or lets a non-finite valence through into every scalar downstream. The area also carries eligibility: establishing what actually happened is what returns blocked traffic to the base rate, and without it the system only ever learns from the requests it chose to let through.

**Section (ledger)** · `sec:assayer:area-ledger`

If this fails: regional history stops being a record of what happened there. Entries decay geometrically and are keyed to the geometry a Sentinel reports, so a stamp left unreplaced discounts an observation for time that passed before it happened, a cell dropped on one unlucky cycle of absence loses months of accumulation, and a recycled axis identifier reads its predecessor's readings as its own warm start. None of this is visible at the point of use — the ledger returns a plausible number either way.

**Section (lifespan)** · `sec:assayer:area-lifespan`

If this fails: deployments cannot evolve. Adding a data source has to cost a fixed, computable amount of model and has to tell the probability calibrator that its fit is now stale; removing one has to take back exactly what it added and nothing besides. Where that does not hold, a registration enters carrying prior coefficients that are read as learned ones, and a deregistration either strands history for a reusable identifier or takes a neighbour's history with it — so the only safe deployment becomes the one that never changes shape.

**Section (linalg)** · `sec:assayer:area-linalg`

If this fails: the model's matrix arithmetic quietly stops being the arithmetic it claims to be. A factorisation that refuses without naming the offending pivot turns an indefinite dimension into an unattributable failure; a half-triangle update that disagrees with the full square puts a value into the covariance that nobody derived. The matrices are where the model keeps what it has learned, so an error here is not one wrong result — it is a wrong belief from then on.

**Section (metrics)** · `sec:assayer:area-metrics`

If this fails: the operator's dashboards and alerts describe a system that is not this one. The export is an interface rather than a by-product: an unprefixed name merges with another subsystem's series in the host's collector, a field held internally but never exported cannot be alerted on at all, and a catalogue that grows to whatever the mappers happen to emit was never designed. What is lost is not a measurement — it is the ability to have been watching.

**Section (numerics)** · `sec:assayer:area-numerics`

If this fails: time itself becomes a source of wrong answers. Values are decayed lazily, whenever they next happen to be touched, so the arithmetic has to be indifferent to how often that is and to clocks that step backwards: a pair of instants read the wrong way round amplifies old evidence rather than discounting it, and a factor permitted to reach exactly zero erases the difference between an entry that is very stale and one that never existed.

**Section (pending)** · `sec:assayer:area-pending`

If this fails: an outcome arrives with nothing to attribute it to. This buffer holds the assessment context across the gap between an estimate and the label that judges it, so an entry lost, an unknown identifier that panics instead of reporting absence, or an unbounded scan run from a read-only query breaks the deferred loop at its weakest moment — labels come back from the host late, duplicated and sometimes simply wrong, and this is what absorbs that. Eviction is amortised for the same reason: no single caller may be made to pay for the whole backlog.

**Section (persistence)** · `sec:assayer:area-persistence`

If this fails: a warm restart is a fiction. A checkpoint that is not a faithful account of the model, or a replay that applies a retried record a second time, restores a system that never ran. A single flipped byte accepted in silence is the worst outcome available here, because a subtly wrong precision matrix is far harder to notice, and far harder to recover from, than a file that will not open at all.

**Section (resonance)** · `sec:assayer:area-resonance`

If this fails: the host's posture-indexed decision surface does not match what the model believes. Crossovers between actions sit in the wrong place, tag shapes are wrong, and an action that cannot compete vanishes from the output instead of being pinned to a known and tiny footprint — so an operator reading the field cannot tell a dominated action from an absent one, and the escalation ladder recommends steps the arithmetic never justified.

**Section (risk)** · `sec:assayer:area-risk`

If this fails: calibrated probabilities drift away from reality. The anchor earns weight only by being more certain than the sister on the features the two share, and a staleness correction has to cancel out of that ratio rather than tilt it — otherwise a cold-start model borrows confidence from one that knows no more than it does, and a refit with nothing to learn from moves the constants anyway. What reaches the decision layer is then a number whose scale nobody can vouch for, and every threshold set against it is set against nothing.

**Section (scenario)** · `sec:assayer:area-scenario`

If this fails: panics, non-finite values or silently wrong answers on configurations the specification says are valid. This area drives the whole engine the way a host does — degraded inputs, evicted identifiers, hashed categoricals, the delicate ends of the probability range, a rejected message followed by an ordinary one — and it is where a failure that lives in the composition rather than in any one part becomes visible, because no single component can be made to show it.

**Section (signal)** · `sec:assayer:area-signal`

If this fails: the host's own components and the model's columns stop lining up. The schema is what lets the encoding path write a value straight into its column without assembling a whole vector first, so a name the schema never declared answered with a plausible-looking offset, or a declaration altered on construction, writes one quantity into another's slot. Shapes belong here for the same reason: a count spanning orders of magnitude that is not compressed on the way in dominates every other feature the model can see.

**Section (snapshot)** · `sec:assayer:area-snapshot`

If this fails: every restart discards training without saying so. The snapshot is the published account of the models and their calibration, so a mean perturbed on the way out, a precision matrix rebuilt from covariance that no longer inverts it, or a cold start that begins anywhere other than neutral each hands back a working copy that disagrees with the model it was taken from. Because the disagreement lives in the beliefs themselves, nothing downstream is in a position to detect it.

**Section (steward)** · `sec:assayer:area-steward`

If this fails: production traffic backs up on operations it should never have waited on. One thread owns the model and applies every change to it, so an owner that does not wake promptly on shutdown leaves the host hanging at teardown, a handle dropped without a closing call strands a thread, and a batch that happens to carry no events blocks its caller forever on a completion that never comes. The area also fixes the vocabulary of structural change itself: every way of altering the model's shape is matched by its inverse, so nothing can be added that cannot be taken away again.

**Section (types)** · `sec:assayer:area-types`

If this fails: identity and time stop being dependable underneath everything else. An instant that does not restore to the instant it came from makes every decay computed after a checkpoint wrong; an interval allowed to come back negative can un-decay evidence or drive a factor above one; identities that collide where they ought to differ collapse per-axis models into a single entry. None of it is visible where it happens — it surfaces as an inexplicable number several layers further up.

**Section (wellness)** · `sec:assayer:area-wellness`

If this fails: silent degradation. The system is confidently wrong and nobody knows: a window that counts slots it never wrote dilutes every percentile it reports, a counter that skips or double-counts spends the refit budget at a cadence unrelated to the evidence that earned it, and a health channel that blocks or panics when nobody is listening turns the observer into a participant in the work it observes. Self-diagnosis has to stay a bystander, and it has to describe the run that actually happened rather than the one the configuration described.

## License · `sec:assayer:readme-license`

Copyright (c) 2024 The Torrust Developers.

This program is free software: you can redistribute it and/or modify it under the terms of the GNU Affero General Public License as published by the Free Software Foundation, version 3.

See [LICENSE](../../LICENSE) for details.
