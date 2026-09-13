## Part (The Operational Cycle) · `part:spec:operational-cycle`

Parts II through V specified the three components one at a time: what the Core sees, what it learns, what the derivation computes, and what the Companion estimates. Part VI specifies them running. Observations enter, assessments leave, labels return and models move; two of those flows share state and must not wait on each other; and every decaying quantity in the system ages under one of two mechanisms whose rates are inventoried in one place.

The Part is written from the host's side of each interface. What the host declares, what it must promise, what it receives per request, what it reports back, what may block what, and what forgets at which rate — these are the six chapters, and their common subject is the contract rather than the arithmetic. Every environment that closes against the operational surface records its current position there. Label-time reconstruction now reproduces the raw vector its prediction used; the remaining departures are named at their own sites.

### Chapter (Runtime Declarations and the Host Contract) · `chap:spec:host-contract`

The chapter fixes what a host says before the system runs and what it promises while it does. Three categories of construction configuration, two lifecycle declarations the host initiates, the label-reporting contract every later mechanism reads, the pre-seeding path that shortens the cold start, and one piece of deployment guidance about severity. The sign convention every one of these depends on is fixed at the front of the document and is cited here rather than restated (`conv:valence:sign`).

**Requirement (What the host declares at construction)** · `req:host:construction`

The Core is constructed from three categories of configuration and no others: the model parameters, the eligibility policy, and the signal schema.

No channel policy. No reward parameters. No posture. No action space. The Core has no concept of a decision to be made, and the configuration surface is where that claim is either kept or quietly broken, which is why the absence is stated as a requirement rather than left as an observation. A host wanting decisions declares its channel policy to the derivation, separately and per channel, and may declare several against one Core (`sig:landscape:derivation-function`).

The separation is the construction-time expression of the principle that the Core measures and does not decide (`prin:principle:measure-not-decide`). It is what makes one Core serve many channels, and what makes a Core replaceable without renegotiating a single reward.

**Requirement (The eligibility policy)** · `req:host:eligibility-policy`

The eligibility policy is one declaration, made at construction, governing whether a challenge label without an investigated outcome counts as unconfounded evidence. It defaults to counting.

The declaration is needed because the Core's label interface does not carry the challenge result (`tab:host:label-reporting`), so the eligibility table cannot distinguish a challenge that caught something from one that did not (`tab:eligibility:training`). The policy therefore governs all non-ground-truth challenge labels uniformly: under the default they reach the sister and anchor models, and under its negation they are treated as confounded and do not. A host needing the finer distinction has a better instrument than this switch — it marks investigated challenge outcomes with the ground-truth flag, which bypasses the criterion for exactly the labels whose outcome was established rather than inferred (`conv:eligibility:ground-truth`).

The parameter is tabulated with the rest of the configuration surface (`tab:config:eligibility`). Challenge follows the declaration, while the other actions and ground-truth override follow the eligibility table.

**Requirement (Outcome axis registration)** · `req:host:axis-registration`

An outcome axis is registered by the host, at any time, before or after traffic has begun. Registration is a lifecycle event and not a configuration setting: it extends every non-anchor model in the system, and the extension is exact rather than approximate (`alg:registry:axis-registration`).

The registration record carries what the axis is, how its outcomes are compressed, how fast it forgets, whether it participates in spatial features, and which eligibility mode governs its training (`schema:registry:axis-record`). The host supplies all of it; the Core infers none of it and cannot, since an axis is a statement about what the host cares to predict.

Because registration may happen at any time, a deployment need not know its full set of axes in advance, which is the property that makes axes worth having at all: the alternative is a schema fixed before the first useful question has been asked. The protocol, its scaling limits and its lifecycle mechanics are specified where the registries are (`sec:registry:outcome-axis`).

**Requirement (The signal schema declaration)** · `req:host:schema-declaration`

The signal schema is declared once, at construction, and fixed thereafter. Each declaration names a signal, its shape, and whether it persists with the entity or lives for a request (`schema:signal:shapes`).

The schema is what sets the width of the signal block in the feature vector, so it is not a piece of configuration that can be revised while models hold parameters indexed against it: a signal added later would move every index after it, and the exactness the lifecycle machinery provides is defined over registrations, not over redefinitions of a declared block (`req:signal:schema-fixed`).

A host uncertain about its signals should declare the superset it might use rather than plan to extend, and accept the cost of a few dimensions that carry nothing. The shapes and persistence classes are defined in the signal chapter (`chap:spec:host-signals`).

**Table (The label-reporting contract)** · `tab:host:label-reporting`

A label is reported against a stored assessment and carries five fields.

| Field | Required | Carrying |
| --- | --- | --- |
| Assessment identifier | Yes | The stored assessment the outcome belongs to |
| Action taken | Yes | What the host actually did, as an observed fact |
| Valence | Yes | The outcome's magnitude, whose sign fixes the risk target |
| Outcomes | No | Per-axis values; an axis not reported is not updated |
| Ground truth | Yes | The host's assertion that the outcome was investigated |

The action taken need not match any action any landscape indicated. The host is free to do anything, including something no derivation suggested, and the Core learns from what was done rather than from what was advised — which is what makes the eligibility table a statement about evidence rather than about compliance (`tab:eligibility:training`).

The ground-truth flag is trusted unconditionally, because the Core has no means of checking it. A corrupted investigation pipeline that systematically mislabels outcomes corrupts the sister model directly and is not detectable by the document's monitoring, which sees labels and not their provenance (`cav:limitation:investigation-pipeline`) and (`cav:monitoring:integrity-scope`).

Two fields of the composed system's label flow are deliberately not here. The challenge result goes to the Companion, because challenge effectiveness is not a Core quantity (`alg:companion:update`). Host posture metadata goes nowhere, because no Core model reads it.

**Algorithm (Pre-seeding)** · `alg:host:pre-seeding`

A host holding historical outcomes may preload them, and they travel the ordinary label path rather than a shortcut: both class-rate trackers move, per-axis compression scales move, and every eligible model updates, under the same eligibility rules live labels obey (`alg:runtime:update-path`). Standardisation statistics move on that path only once the full-vector phase is in service (`alg:standardisation:batch-initialisation`). Before then the cold observation ramp owns the coordinate transition, so a pre-seeded label cannot race it and cannot be overwritten by it.

Pre-seeding is worth most at cold start, where it can carry a deployment through the first warm-up stages before a single live request arrives, so the first assessment is directional rather than a restatement of the prior (`tab:warmup:stages`). It also seats the importance-weight trackers at a real base rate instead of an assumed one, and can converge a severity axis ahead of traffic. Where the publication interval permits, a pre-seed batch publishes once at its end rather than once per historical label (`req:publication:interval`).

The constraints are the interesting part. A pre-seeded label needs a stored assessment to attach to, so either the buffer is pre-seeded in step (`def:runtime:pending-entry`) or a simplified path synthesises a feature vector from the historical record — which in turn requires the signal schema and the Sentinel registry to be declared first. Either arrangement uses the specified pending entry without adding a channel field to the Core.

**Remark (Combining risk and severity)** · `rem:host:severity-discrimination`

A deployment where a small loss and a large one demand different responses should register a severity axis before live operation, because the risk model will not supply the difference. Its target is binary and answers whether an outcome will be adverse, never how adverse (`def:risk:target`). Valence exponentially weighted averages carry severity into the features, but slowly: roughly twenty labels per entity and a thousand per cell before they mean anything (`data:ledger:attenuation`). The axis model gives a calibrated severity prediction from its first eligible labels instead (`def:axis:per-axis-model`).

Without pre-seeding the axis converges in about twice the model dimension in eligible labels — some thirteen days at the reference configuration — and until then severity discrimination rests on those near-zero averages (`bound:resource:convergence-budget`) and (`cav:limitation:severity-cold`). Pre-seeding with enough historical labels carrying the axis value closes the gap entirely (`alg:host:pre-seeding`).

The combination is the host's, deliberately. The Core keeps probability and severity apart because their weighting is a loss function and only the host holds one (`def:landscape:outcome-neutrality`). The simplest composition ranks by the risk probability times a host utility over the predicted severity. A host using the derivation has a more principled option: modulate the posture by predicted severity, reading the landscape higher for severe predictions and lower for mild ones. Because posture and prior log-odds are interchangeable on a channel, that is a shift of the read-point and not an override of the model (`prin:channel:relativity`). Neither alone suffices, and the separation is what keeps the choice visible (`cav:limitation:severity-path`).
