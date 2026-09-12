# Convergence thresholds: the three decisions, sized · `rep:health:threshold-decision-options`

This document presents three owner decisions. Its recommendations are the study's, not rulings. The commissioning caveat separates the stage vocabulary, the per-dimension inputs, and the Companion figures because each can be decided without choosing either of the others (`cav:health:dead-convergence-thresholds`); `packages/assayer/adr/health.md:221-274`.

**Decision one: the stage vocabulary**

**Evidence.** The configuration names Warming, Converging, and Converged across eight identity-stage figures (`packages/assayer/src/config/types.rs:446-468`). The specification names six approximate warm-up stages instead: Cold start, Anchor emergence, Pre-calibration, Sister convergence, Interaction maturity, and Steady state (`tab:warmup:stages`); `packages/assayer/docs/spec/analysis-warmup.md:32-55`. The composite implements those six names in enum form (`packages/assayer/src/health/composite.rs:32-65`). The per-dimension tracker implements a different five-place ladder: Initial, Volatile, Stabilising, Maturing, Stable (`packages/assayer/src/health/identity_tracker.rs:138-233`). Exact-case search finds none of the configuration's three stage names anywhere in `packages/assayer/docs/spec/`, and the specification's monitoring-parameter table contains none of the twelve convergence fields (`tab:config:monitoring`); `packages/assayer/docs/spec/reference-configuration.md:327-350`.

There is also a difference of kind. The six specification stages are approximate descriptions of continuous whole-system warm-up, whereas the five tracker stages are executable per-dimension classifications. The configuration fields combine per-dimension cells, observations, and graph importance. That shape points toward the identity tracker, while the names point toward neither ladder.

**Option A — map the three names onto the five-stage identity ladder.** A progress-preserving candidate would put Initial before Warming, treat Volatile and early Stabilising as Warming, treat later Stabilising and Maturing as Converging, and reserve Converged for Stable. A threshold-entry candidate would instead make the Warming figures admit Stabilising, the Converging figures admit Maturing, and the Converged figures admit Stable. Either candidate must decide how the new counts combine with the existing churn, Jaccard, and age gates, and whether a threshold is an entry condition or a completion condition.

- Cost: medium implementation after a large specification decision; the stage function, health reports, transition events, persistence fixtures, and tests all need the chosen conjunctions.
- Risk: high semantic risk. Both plausible mappings change what the same three words mean, and neither is recoverable from the field names. A host could receive a later-sounding stage while an existing live cut still says the dimension is unstable.

**Option B — map the three names onto the six-stage composite.** A plausible compression is Cold start before Warming; Anchor emergence and Pre-calibration as Warming; Sister convergence and Interaction maturity as Converging; and Steady state as Converged.

- Cost: medium code change but large semantic work. The composite would need per-dimension figures aggregated into a whole-system verdict, with a rule for the weakest, average, or total dimension.
- Risk: very high mismatch. Cell count and graph importance do not describe anchor emergence or Platt calibration. This option makes fields that look local govern stages whose earlier transitions are label- and calibration- based (`packages/assayer/src/health/composite.rs:137-215`).

**Option C — rename and reshape the fields to an existing ladder.** For the identity ladder, configuration could expose the quantities actually read: volatile and stable churn cuts, their overlap confirmations, and maturity age. For the composite, it could expose the eligible-label boundaries and Platt cuts the six stages actually consume. Either form removes the false three-stage alias rather than trying to define it after shipment.

- Cost: medium-to-large public configuration migration. Serialized config and source users lose field compatibility; defaults and validation need a new specification table.
- Risk: moderate. The names become truthful, but making today's private identity cuts host-settable creates a new support surface. The sensitivity study also shows that the shipped Jaccard cuts add no independent classification under the documented inputs, so copying every private constant into public config would preserve redundancy rather than justify it.

**Option D — delete the eight stage figures.** Keep the two live Platt fields, remove the generic three-stage surface, and continue reporting the existing five- and six-stage ladders.

- Cost: small internally, medium for public compatibility. Builder validation and its tests shrink, but downstream struct literals and serialized configuration may need migration.
- Risk: low diagnostic risk, because no production path reads the fields now. The primary risk is user-visible API removal, not changed runtime behaviour.

**Study recommendation for decision one.** Choose Option D: delete the eight Warming/Converging/Converged figures and do not invent a map. If convergence cuts are to be configurable, take Option C as a later, separately specified surface named for the ladder and quantities it actually controls. Option A is defensible only after a new specification explicitly chooses one of its competing mappings. Option B should be rejected because the quantities and the composite stages describe different processes.

**Decision two: the missing per-dimension inputs**

**Evidence.** The tracker already stores current cell count, so all three cell thresholds have an obvious candidate input (`packages/assayer/src/health/identity_tracker.rs:50-75`). It has no cumulative per-dimension observation count. The maintenance loop counts observations only in a local drain variable, feeds them to the graph, and discards the count (`packages/assayer/src/identity/maintenance_loop.rs:457-469`).

Contrary to the caveat, per-dimension graph importance is already aggregated. `IdentityGraphSnapshot::total_energy()` returns whole-graph energy (`packages/assayer/src/identity/snapshot.rs:68-92`), the assessment abstraction names it per-dimension graph total importance (`packages/assayer/src/assessment.rs:795-807`), and the live implementation reads it from the dimension's graph snapshot (`packages/assayer/src/lib.rs:1106-1130`). The assessment path then takes `ln(1+x)` before using it as a feature (`packages/assayer/src/assessment.rs:1018-1039`).

This corrects the requested size. The eight fields are three cell minima, three observation minima, and two importance minima. If “counterpart” means a field on the tracker, five lack one: three observation and two importance figures. If it means a quantity anywhere in the package, only the three observation figures lack a cumulative counterpart. There is no consistent enumeration under which six of the eight lack a quantity. The caveat's “six” and “no graph importance” premises are errors; `packages/assayer/adr/health.md:253-269` records them.

**Option A — build the complete data path and retain the figures.** This is not merely adding fields to the tracker. It requires the owner to specify:

- what an observation is — every assessment coordinate delivered to the graph, every processed queue item, every eligible label, or every competitive-set update;
- whether the count is lifetime, decayed, windowed, or reset on registration, graph rebuild, and lifecycle changes;
- whether importance thresholds compare raw graph energy or the logarithmic feature, and how fresh the value must be;
- whether every configured dimension must pass, and how the new quantities combine with churn, Jaccard, and age.

The implementation would then add a cumulative count to maintenance-owned per-dimension state; publish count and raw importance together on every relevant update rather than relying on checkpoint-aged graph snapshots; carry that snapshot across the maintenance/model-owner boundary; hand both values to the tracker or compute the stage over a joined health view; preserve or deliberately reset them across checkpoints; and emit stage transitions even when observation or importance changes without a competitive-set entry or exit. The current production path sends tracker updates only when a set changes (`packages/assayer/src/identity/maintenance_loop.rs:472-533` and `packages/assayer/src/owner/lifecycle.rs:176-210`).

- Cost: large. This crosses assessment, maintenance, publication, health, lifecycle, checkpoint, event, and test surfaces.
- Risk: high until the units and reset rules are specified. A cumulative raw count rewards old traffic forever; a decayed count may regress; checkpoint- aged importance can report a stale stage; and observing every assessment can advance a stage without a single eligible training label.

**Option B — reuse available cells and importance, delete observation minima.** Publish fresh raw total importance beside the competitive set, compare the three cell and two importance thresholds, and remove the three observation fields.

- Cost: medium. It reuses real quantities but still needs publication, scale, mapping, conjunction, and transition decisions.
- Risk: medium-to-high. It avoids an invented observation unit but retains the unspecific three-stage vocabulary and may mistake graph traffic for model convergence.

**Option C — delete the figures that need new semantics.** The commissioning brief calls this “delete the six figures,” but no six-field target exists in the struct. The coherent deletions are either the three observation fields, the five non-cell fields, or all eight stage fields.

- Cost: small for three or five, and still small internally for all eight; public compatibility cost grows with the deletion set.
- Risk: selecting an unexplained six would create a second counting error. A partial deletion also leaves generic stages that still lack a specification.

**Study recommendation for decision two.** Do not build Option A on the present semantics. Delete all eight stage figures together with decision one's generic vocabulary. If the owner retains them, first specify the observation unit, importance scale, freshness, reset rules, and conjunction. Then reuse the graph energy already present, make its publication current, and add only the missing cumulative observation signal. Do not commission a new graph-importance aggregation: that part already exists.

**Decision three: the two Companion figures**

**Evidence.** The two fields are `challenge_sufficient_evidence = 50` and `challenge_min_samples = 10`. Their source carries a dated 2026-05-17 note to move Companion-oriented health thresholds out of Core convergence construction configuration (`packages/assayer/src/config/types.rs:470-493`). Construction only checks that each is non-zero (`packages/assayer/src/api/builder.rs:466-478`).

The Companion is explicitly host-owned and outside the Core snapshot and label pipeline (`packages/assayer/src/risk/challenge.rs:412-434`). It has one real sufficiency operation: effective sample size is compared with a threshold supplied by the caller (`packages/assayer/src/risk/challenge.rs:381-402`), and its standalone health report accepts that threshold and publishes the resulting boolean (`packages/assayer/src/risk/challenge.rs:528-587`). This is a semantic counterpart for `challenge_sufficient_evidence`, though the config field is not connected to it.

There is no second minimum-samples gate in that tracker and no production name match for `challenge_min_samples`. The specification uses twenty target effective observations for Companion convergence (`bound:companion:convergence`); `packages/assayer/docs/spec/companion-tracking.md:308-321`, while its warm-up reading aid distinguishes about ten contributing labels for a first useful estimate and about twenty for practically useful variance (`tab:warmup:milestones`); `packages/assayer/docs/spec/analysis-warmup.md:166-190`. Those two milestones could explain the two defaults only after a new ruling; the shipped sufficient- evidence default is fifty, not the specified twenty.

Core composition deliberately refuses the connection. The composite argument is named `_challenge_sufficient`, documented as non-gating, and unused (`packages/assayer/src/health/composite.rs:137-178`). The label path supplies a hard-coded false placeholder (`packages/assayer/src/owner/label_path.rs:1231-1244`). That agrees with the standing decision that health reports and never gates (`dec:health:reports-never-gates`) and with the Companion's host boundary.

**Option A — relocate both fields to a host-owned Companion health config.** Move the sufficiency threshold beside `ChallengeEffectivenessTracker` and pass it to `health_report`. Define a distinct minimum-samples behaviour before moving the second field: withhold a point estimate, mark it provisional, or remove it if sufficiency already expresses the evidence floor.

- Cost: medium because it creates or extends a public host-owned configuration surface and needs a migration from Core config.
- Risk: moderate. Carrying both numbers without distinct semantics recreates the current duplication in a better directory. Choosing fifty, twenty, or ten for sufficiency is itself an owner decision.

**Option B — relocate sufficiency, delete minimum samples.** Keep the real effective-sample threshold where the host calls Companion health, remove the field with no counterpart, and reconcile the threshold's default with the specification's twenty-observation target in the ruling.

- Cost: small-to-medium. The tracker API already accepts the threshold; the work is ownership, defaults, documentation, and compatibility.
- Risk: low. It preserves the one implemented distinction and removes the unexplained second one.

**Option C — delete both from configuration.** Continue making the threshold an explicit argument to `health_report`, as today, and let each host own its value.

- Cost: small internally, with public configuration migration.
- Risk: moderate consistency risk. Different call sites may choose different sufficiency thresholds unless the host centralises the value.

**Option D — connect them to the Core composite.** Feed Companion evidence into the currently unused argument and gate SteadyState or another Core stage.

- Cost: medium implementation and large specification change.
- Risk: unacceptable under current decisions. It violates Companion ownership, makes a Core stage depend on host-driven channel evidence, and contradicts report-only health (`dec:health:reports-never-gates`). Multiple channels would also need an aggregation rule that no source supplies.

**Study recommendation for decision three.** Choose Option B: relocate the one real sufficiency threshold to host-owned Companion health, delete `challenge_min_samples`, and remove the unused Companion argument and field from Core convergence composition. Do not silently change fifty to twenty in the move: the specification's target and the shipped default conflict, so the move must name the intended threshold rather than inherit one. Reject Option D; connection is contrary to both the source's dated relocation note and the report-only boundary.

**Combined size and sequencing**

The smallest coherent ruling is deletion of the eight generic stage figures, relocation of the one real Companion sufficiency threshold, and deletion of the Companion minimum-samples field. It removes dead validation without changing a runtime stage. If the generic stage surface is instead to be wired, the sequence must be vocabulary first, quantity semantics second, data transport third, and code last. Building transport before naming the stages and units would make the implementation the accidental specification.
