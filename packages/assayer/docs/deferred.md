# The Assayer Deferral Register · `reg:assayer:decision-deferral-register`

This register holds the Assayer's open deferrals — decisions the implementation deliberately did not make, each named by a label that is now that deferral's identity. Every later document that touches a deferral cites its label rather than re-describing it.

The register is temporary by design. Under the ruling *Deferrals live in the records* ((`dec:assayer:deferred-home`) in [the campaign backlog](plans/backlog.md)), each deferral migrates into the decision record that owns the deferred decision as the records are rewritten: the record states the deferral in full under a label of its own area, and the entry here keeps its head — which other documents cite — with its body reduced to the citation into the record. Every retirement claim the register carries names the live implementation locus that is its evidence (`sec:assayer:defer-retirements`).

**That migration is now complete, and so is the register.** No entry below is open: every deferral this file ever held has been implemented or retired at the record that owns it, and each stands under (`sec:assayer:defer-retirements`) with its disposition and its reason. What remains is the linted list this file was always going to shrink to: a stable set of heads that other documents cite, each resolving into the record where the deferral was stated in full, its code evidence given, and its completion trigger fixed. Nothing here needs re-verifying against the code, because nothing here claims anything about the code — the records do.

An empty register is not a closed one. The file stays because the heads are cited from across the corpus and because the next deferral will be entered here in the same grammar as the last.

The items were re-verified against the code as it stood when each was written, not carried over on faith from the 2026-05-17 list, and that verification travelled into the records with them. Items that the code had already overtaken were retired rather than dropped; they are recorded under (`sec:assayer:defer-retirements`), which is the one section below that is not a citation list.

Area is `assayer` throughout; deferral items carry kind `entry` per the kind registry, and the head-and-mint grammar is that of the label calculus. The linter's carrier covers this register, so the grammar is enforced mechanically on every check run.

## Architectural deferrals · `sec:assayer:defer-architecture`

Every entry this section held has now completed, and each stands under (`sec:assayer:defer-retirements`); the section is kept for the citations that point at it rather than for open work.

## Health report deferrals · `sec:assayer:defer-health`

These values were absent from the health report or present but placeholder-backed when the audit found them. The grouping is historical: it records where each deferral was found, not where it is now stated. Every one of the section's original entries has now completed or been retired, and they stand under (`sec:assayer:defer-retirements`); the section is kept for the citations that point at it rather than for open work.

## Retirements and audit findings · `sec:assayer:defer-retirements`

The original register entries above retain their current dispositions at their owning records. Implemented, audit-resolved and formally retired entries are recorded here rather than dropped. A retirement is not always a delivery: an entry whose subject the corpus superseded is retired on the ruling that says so, and the reason travels in the row.

**Entry (Hibernation for deregistered Sentinels and axes, retired)** · `entry:assayer:defer-lifecycle-hibernation`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:construction:hibernation`). A Sentinel and an outcome axis may each be removed hibernating rather than destructively, and a registration of the same identifier before the archive expires re-extends the models with the entity's own block aged on both clocks instead of with the prior (`claim:lifespan:a-hibernating-sentinel-returns-with-its-own-weights-aged-on-both-clocks`).

The definition STOP that held this entry is discharged by a minimal archive contract rather than by an approximation of one. The specification fixed the mathematics whole and the archive not at all, so what was chosen is the narrowest answer to each of the six open questions: the package's own checkpoint encoding with a record generation beside it, custody inside the Assayer's working copy and its whole-state checkpoint, a configured wall-clock expiry defaulting to ninety days, a restore window that a re-registration closes, the specified decay treatment and nothing else, and no payload at all for an identity dimension. None of the six extends what hibernation preserves: every cross-feature block still comes back at zero (`claim:lifespan:a-restored-sentinels-cross-feature-blocks-come-back-at-exactly-zero`).

**Entry (Cross-layer feedback latency, retired)** · `entry:assayer:defer-cross-layer-latency`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:cross-layer-latency`). The Sentinel wire contract now carries the age of a batch's oldest observation at emission, so the first stage of the observation-to-publication total is measurable at last; report ingestion retains that age beside the arrival, the label path folds all four stages into one accumulator at publication, and full health reports the decomposition (`claim:wellness:the-feedback-latency-decomposes-into-four-stages-and-the-report-stage-is-a-lower-bound`).

The first stage ships as a lower bound and is stated as one on every surface that carries it, the residual between the producer's emission and the report's arrival being deliberately unmeasured. That is what discharges the definition STOP rather than evading it: the STOP stood because no substitute assembled from the remaining three clocks could be honest about the first, and an exact bound that names what it omits is not such a substitute.

**Entry (Ledger materiality: value realised and attenuation limited, retired)** · `entry:assayer:defer-ledger-materiality`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:materiality`). Full health reports the realised value and the attenuation-limited share per Sentinel and fleet-wide, weighted by assessed traffic, against the theorem's own half-deviation boundary. The definition STOP that held this entry is discharged by licensing the Ledger entry to store the evidence the predicate is about — the raw undecayed adverse and eligible counts, and a time-weighted window of recent eligible arrivals — so the theorem's two conditions are answered separately rather than by one attenuated average (`claim:wellness:ledger-materiality-is-traffic-weighted-and-both-of-the-theorems-conditions-bind`), (`claim:wellness:each-arm-of-the-attenuation-limited-predicate-is-decided-at-its-own-boundary`).

**Entry (Intra-label parallelism for independent model updates, retired)** · `entry:assayer:defer-label-parallelism`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:concurrency:label-parallelism`). Repeated release-mode profiling at the representative dense width attributes most label-publication time to the independent rank-one updates, and two fixed scoped helpers now run the operational and sister-plus-anchor groups beside the steward's ordered axis group. Their joined result is bitwise equal to the former sequential order (`claim:concurrency:bounded-label-model-update-groups-preserve-the-sequential-results`).

**Entry (Synchronisation-error visibility, retired)** · `entry:assayer:defer-sync-error-visibility`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:posterior:sync-visibility`). Per-model error, effective interval and shortening count reach detailed precision health, their maximum reaches compact health, and a residual — the reading less the part the replenishment clamp put there — standing above the specified model-width threshold and above the reading's own resolution calls for a recomputation whose adoption halves the interval and restarts recovery (`claim:bayes:the-live-model-shortens-and-restarts-recovery-on-a-needed-rebuild`), (`claim:wellness:a-maintained-synchronisation-error-reaches-both-export-tiers`).

**Entry (Sentinel coverage metric, retired)** · `entry:assayer:defer-metric-sentinel-coverage`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:metrics:sentinel-coverage`). Full health computes the current reporting share of the registered Sentinel fleet and the full-report mapper exports it as `assayer_sentinel_coverage` (`claim:wellness:full-health-coverage-reads-current-report-presence-across-the-registered-fleet`).

**Entry (Zero-Sentinel assessment counter metric, retired)** · `entry:assayer:defer-metric-zero-sentinel-counter`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:metrics:zero-sentinel`). The assessment path advances a dedicated atomic once per prior-only result and the summary mapper exports the live total (`claim:wellness:the-live-assessment-path-counts-each-zero-sentinel-result-exactly-once`).

**Entry (Signal-cache hit-rate and utilisation metrics, retired)** · `entry:assayer:defer-metric-signal-cache`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:metrics:signal-cache`). Full health carries the cache's raw hit, miss, eviction, size and capacity statistics, from which the mapper exports hit rate and utilisation (`claim:wellness:signal-cache-metrics-read-the-live-cache-statistics`).

**Entry (Positive-class prior metrics, retired)** · `entry:assayer:defer-metric-p-positive`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:metrics:positive-prior`). Both published positive-class priors travel unchanged through the health summary into separately named gauges (`claim:wellness:positive-class-prior-metrics-read-the-published-snapshot`).

**Entry (Drift reset counter metric, retired)** · `entry:assayer:defer-metric-drift-resets`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:metrics:drift-resets`). The shared emission helper advances a dedicated atomic before bounded-channel delivery and the summary mapper exports that live total (`claim:wellness:every-emitted-drift-reset-is-counted-whether-or-not-the-event-channel-delivers-it`).

**Entry (Marginalisation completion event and Schur diagnostics payload, retired)** · `entry:assayer:defer-marginalise-event`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:marginalise-event`). Each lifecycle removal carries its event-local aggregate diagnostics in the completion result and pushes the same payload on the bounded health stream (`claim:lifespan:sentinel-deregistration-pushes-its-marginalisation-completion-diagnostics`).

**Entry (Importance-ceiling binding and gradient balance, retired)** · `entry:assayer:defer-importance-ceiling-health`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:importance-ceiling`). Label-time accumulation reports ceiling binding and positive gradient balance independently for operational and eligible sister streams (`claim:labelling:importance-health-accumulates-the-weights-used-by-each-model-stream`).

**Entry (Per-entity concordance, retired)** · `entry:assayer:defer-per-entity-concordance`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:per-entity-concordance`). A bounded per-entity map reports running sign agreement and flag status, with least-recently-observed eviction and a lifetime eviction count (`claim:wellness:per-entity-concordance-flags-at-the-gate-and-evicts-the-least-recently-observed`).

**Entry (Per-Sentinel alarm outcome CUSUMs, retired)** · `entry:assayer:defer-alarm-outcome-cusums`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:alarm-cusums`). Directional Page accumulators cross frozen alarms with eventual outcomes per Sentinel and axis and are reported on Sentinel health (`claim:labelling:alarm-outcome-cusums-accumulate-per-sentinel-axis-and-direction`).

**Entry (Full Sentinel structural diagnostic relay, retired)** · `entry:assayer:defer-sentinel-structural-relay`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:structural-relay`). Full health relays the latest report's complete contour and analysis-set structural payload from one immutable report index (`claim:wellness:sentinel-structural-health-relays-the-latest-report-diagnostics`).

**Entry (Per-Sentinel informativeness backing value, retired)** · `entry:assayer:defer-sentinel-informativeness`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:health:sentinel-informativeness`). Full health takes the mean absolute published operational weight over each Sentinel's own slot and reports that measured value per Sentinel, the reading the specification now fixes (`def:monitoring:encoding-effectiveness`), with a producer-path test proving the reported value folds from the same slot and is not a constant (`test:crate:informativeness-reads-the-operational-slot-weights`).

**Entry (Encoding effectiveness, retired)** · `entry:assayer:defer-encoding-effectiveness`

**RETIRED, superseded.** The owning record retires the legacy row formally at (`entry:health:encoding-effectiveness`), on a ruling rather than on a completion trigger met. The three-component rank-headroom, coordination-activity and plateau-efficiency formula this entry stood against was superseded rather than built, and the reading that took its place is the delivered per-Sentinel informativeness value retired at (`entry:assayer:defer-sentinel-informativeness`). No approximation of the historical quantity is reported and none is owed.

**Entry (Schur correction skip counter, retired)** · `entry:assayer:defer-schur-skip-counter`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:posterior:skip-counter`). Every full-dimension model's marginalisation diagnostics are folded into the working copy's checkpointed error ledger (`src/snapshot/working.rs:873`), skipped corrections are read from its fallback count (`src/model/marginalise.rs:533`), and published health and the host report project that measured count (`src/owner/label_path.rs:1382`, `src/api/health.rs:378`).

**Entry (Pending-buffer eviction counter, retired)** · `entry:assayer:defer-pending-eviction-counter`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:retention:eviction-counter`). Successful lazy removal of a live pending entry increments the interval counter, while cleanup of an identifier already consumed by a label does not (`src/pending/buffer.rs`). Assessment insertion is followed by the take-and-clear read, and its value travels inline on `HealthSnapshot.pending_buffer_evictions` (`src/assessment.rs`).

**Entry (Pending-buffer eviction visibility, retired)** · `entry:assayer:defer-buffer-eviction-visibility`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:retention:eviction-visibility`). Buffer health carries the lifetime live-eviction fraction and the optional age of the oldest live entry (`src/health/summary.rs`), assembled from the pending buffer against the injected monotonic clock (`src/api/health.rs`).

**Entry (Host-configurable identity graph thresholds, retired)** · `entry:assayer:defer-identity-thresholds`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:identity-thresholds`). Registration exposes the graph's split, create, evict and spatial-decay inputs, with the former derived and fixed values available as explicit defaults (`claim:identity:registration-copies-host-graph-thresholds-without-deriving-them-downstream`).

**Entry (Identity dimension audit metadata, retired)** · `entry:assayer:defer-identity-audit-metadata`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:audit-metadata`). The host description and coordinate semantics cross registration into the stored dimension and public health diagnostics without entering graph or model behaviour (`claim:identity:dimension-audit-metadata-is-reported-and-does-not-drive-core-behaviour`).

**Entry (Published identity total importance, retired)** · `entry:assayer:defer-identity-total-importance`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:total-importance`). The competitive-set index publishes the graph total beside its cells, maintenance refreshes it, and assessment reads that scalar instead of the checkpoint reconstruction (`claim:identity:published-total-importance-equals-the-checkpoint-reconstruction-it-replaces`).

**Entry (Identity outcome read-time decay view, retired)** · `entry:assayer:defer-identity-decay-view`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:decay-view`). Identity outcome state exposes an exact pure decay view through the shared clock function, and assessment-time outcome features read it at the assessment's persistent timestamp (`claim:identity:a-decayed-outcome-view-is-exactly-the-mutating-decay-result-without-changing-storage`).

**Entry (Competitive-cell outcome warm-start retention, retired)** · `entry:assayer:defer-competitive-warm-start`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:warm-start`). Exit resets measurement only, re-entry reuses the retained outcome, and a full graph-interval census removes state only after its interval is evicted (`claim:identity:retained-outcome-state-is-cleaned-only-when-its-graph-interval-no-longer-exists`).

**Entry (Ledger depth-tier distribution, retired)** · `entry:assayer:defer-ledger-depth-histogram`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:depth-histogram`). Full health scans every active entry into its depth bin, and the focused test distinguishes the distribution from the single maximum-depth reading (`claim:wellness:ledger-health-counts-every-active-entry-at-its-own-depth`).

**Entry (Ledger cell-set deltas in health, retired)** · `entry:assayer:defer-ledger-cell-deltas`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:cell-deltas`). Per-report creations and deletions accumulate in per-Sentinel process-lifetime counters and are reported on Ledger health across successive acknowledgements (`claim:wellness:ledger-cell-set-deltas-accumulate-per-sentinel-across-reports`).

**Entry (Ledger immature cell count, retired)** · `entry:assayer:defer-ledger-immature-count`

**RETIRED, implemented.** The owning record's completion trigger is met at (`entry:memory:immature-count`). Full health applies the strict eligible-label threshold per entry and reports both per-Sentinel and fleet totals (`claim:wellness:ledger-immaturity-is-strictly-below-the-eligible-label-floor`).

- **Identity `observations_dropped` backing counters** — RETIRED, implemented. Per-dimension overflow counters exist and are incremented on drop (`src/identity/mod.rs:140`, `src/identity/mod.rs:189`, read at `src/identity/mod.rs:198`), surfaced on identity dimension health (`src/api/health.rs:276`, field at `src/health/summary.rs:354`), and exported as a metric (`src/metrics/mapper.rs:432`, `src/metrics/mapper.rs:435`, catalogued at `src/metrics/catalog.rs:477`). The zero seen at `src/api/health.rs:284` is the default for a dimension known only to the convergence tracker, not a placeholder.
- **Metrics row: per-model synchronisation error and shortening counters** — RETIRED, duplicate. The metrics-section row named the same deferral as the health-section row, differing only in which end of the same missing plumbing it described. Both are now retired at (`entry:assayer:defer-sync-error-visibility`), whose owning record states the delivered per-model visibility and shortening rule.
- **Metrics row: immature Ledger cell count metric** — RETIRED, duplicate. The row's prerequisite — an immature-cell total on Ledger health — was precisely the health-section deferral now retired at (`entry:assayer:defer-ledger-immature-count`). That health prerequisite is implemented; this historical duplicate does not assert separate metric export work.

One entry's status was **ambiguous** on inspection and has now been resolved deliberately. (`entry:assayer:defer-ledger-cell-deltas`) remained live because the trigger said *accumulated per Sentinel* while only per-report acknowledgements existed. The health-side lifetime counters now satisfy that narrower reading, so the implemented retirement above closes the ambiguity rather than treating an acknowledgement as sufficient by drift.

Two metric entries were found to be **narrower than recorded** at the audit cut: (`entry:assayer:defer-metric-signal-cache`) and (`entry:assayer:defer-metric-p-positive`) already had their backing values and needed only surfacing. The same audit correction applied within (`entry:assayer:defer-metric-sentinel-coverage`). Those three have since completed and are retired above. The narrower-scope observation for (`entry:assayer:defer-sync-error-visibility`) has since completed and joined them.

The following were retired in earlier passes and are recorded so they are not re-added. All four were re-verified in this audit.

- Feature-stable outcome drift and quantile error concentration are computed and reported as their definitions require (`def:monitoring:feature-stable-drift`) and (`def:monitoring:quantile-concentration`).
- Sentinel inline health-report pass-through is Sentinel-internal diagnostics that the Assayer does not consume, and is not a Core deferral.
- Companion challenge effectiveness health is host-owned and exposed through the Companion health report; Core metric names for challenge effectiveness are host-surface naming conventions, not Core health deferrals.
- Signal-cache evictions had a backing counter and export before the later activation added hit rate and utilisation; that activation is now retired at (`entry:assayer:defer-metric-signal-cache`).
