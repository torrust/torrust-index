# Metrics · `rec:metrics:passive-export-catalogue`

This record owns what the package exports for monitoring, and its central discipline is negative: the package renders nothing. It realises the metrics choice (`dec:contracts:metrics-surface`).

It is a small record on purpose — three decisions and a catalogue it deliberately declines to count. That refusal is the record's sharpest content rather than an omission, and the reason for it is measured: the census found the catalogue's size given five different values across three documents, matching the code in none of them (`reg:assayer:adr-contradictions-register`).

**Decision (Export is passive and renders nothing)** · `dec:metrics:passive-export`

The package emits no formatted output and depends on no monitoring crate. It exposes values, and a host renders them in whatever format its own monitoring stack speaks (`src/metrics/mapper.rs`). What must be reported per assessment is the specification's (`req:detection:per-channel-reporting`).

Passivity is a dependency decision before it is a formatting one. A rendering crate in this package would be a version constraint imposed on every host, for a format only the host can know is right — and hosts disagree about that format for reasons the package has no visibility into. Rendering nothing means the package never has an opinion it would have to be argued out of, and never blocks a host on a dependency upgrade it did not choose.

**Decision (The catalogue is a compile-time constant)** · `dec:metrics:compile-time-catalogue`

The catalogue of exported metrics is a compile-time constant rather than a runtime registry that emitters register into (`src/metrics/catalog.rs`).

The property this buys is that the catalogue cannot drift from what the code emits: there is no window in which a metric exists but has not registered itself, and no path on which registration is skipped. A runtime registry would make the catalogue a fact about execution — order-dependent, and answerable only by running the program — where a constant makes it a fact about the source, which is checkable without running anything.

**Decision (A neutral sample type is the intermediate form)** · `dec:metrics:neutral-sample`

Values leave the package as a neutral sample type carrying a name, a value and its labels (`src/metrics/mapper.rs`). It commits to no exposition format. What those values describe is the health record's, and the cheap summary is what the mapper reads (`dec:health:tiered-queries`).

The neutrality is what makes the passivity above implementable rather than merely stated. Without a shared intermediate the package would either hand out its internal health types — making every field of them a public contract, so that no health field could change without breaking an exporter — or hand out rendered text, which is the dependency just refused. One narrow type in between keeps the health surface free to evolve and the host free to render.

**Decision (What belongs in the catalogue, and what each entry declares)** · `dec:metrics:catalogue-membership`

Membership is not a matter of taste. A metric belongs exactly when a mapper emits it from one of the two health surfaces, and the correspondence holds in both directions: an entry no mapper emits does not belong, and a sample no entry declares must not be emitted. The one deliberate exception is a third kind of entry, which reserves a name for a metric the host maintains around its own call site; those are declared so that a host naming such a metric names it the same way this package would, and no mapper emits them.

Each entry declares five things and no more: the metric's name, whether it is a counter or a gauge, its help text, the label dimensions it carries, and which surface produces it. The surface is the field a host reasons with. It separates the cheap summary from the expensive full report and is therefore what a host filters on to choose a scrape cadence, rather than a fact about what the metric means.

Both directions of the membership rule are tests rather than sentences, which is what the standing preference asks of a mechanical property (`goal:assayer:tools-over-discipline`). That is what makes this a decision about what the catalogue *is* rather than a description of what it currently holds, and it is why nothing here enumerates it.

**Convention (The catalogue's size is stated nowhere outside the code)** · `conv:metrics:size-unstated`

No count of the exported metrics appears in this record, in the specification, or in any other document. A reader wanting the number reads the catalogue.

The rule is written from evidence rather than taste. That number was carried in six places, given five different values across three documents, agreed with the code in none of them, and was guarded by a test asserting only that it fell in a wide band — so the one mechanism that could have caught the drift was calibrated not to. A figure restated across documents is a figure nobody maintains, and the only durable fix is to keep it in the single place that cannot be wrong about it.

**Convention (The catalogue's pin moves whenever the catalogue does)** · `conv:metrics:pin-churn`

The catalogue carries a pin derived from its own source text, so every edit to it — an entry added, a surface changed, a help string reworded — mints a new one. The pin is therefore not an identifier the catalogue keeps; it is a statement about which catalogue, and it is expected to churn.

The churn is the mechanism rather than a cost of it. A document or a comment that depends on the catalogue's exact contents binds itself by citing the pin, and when the catalogue moves that citation names a label nothing mints any more and fails. The dangling citation is the intended outcome: it is the thing that makes somebody re-read the dependency, at the moment the dependency stopped being true, which is precisely what a count restated in prose never did (`conv:metrics:size-unstated`).

This asks nothing of a reader who merely wants to know what is exported; the pin is for dependants, and there is no obligation to cite it. What it must not become is a figure quoted for its own sake, which would be the retired failure wearing a new spelling.

**Corollary (Cardinality is bounded by construction)** · `cor:metrics:bounded-cardinality`

Because the catalogue is fixed at compile time (`dec:metrics:compile-time-catalogue`) and every label dimension is drawn from a declared set, the number of distinct series the package can produce is bounded before it runs.

What that rules out is the failure this kind of surface usually has: unbounded cardinality from a label carrying an open-ended value, which degrades the host's monitoring system rather than this package. The bound is structural — there is no path that mints a metric name or a label value at runtime — so it holds without a sampling policy, a cap, or anything else the host would have to configure.

**Caveat (The encoding rules are definition, not decision)** · `cav:metrics:encoding-absorbed`

The rules for encoding enumerations, absent values and booleans into samples are not decided here. They define the form the samples take rather than choosing a posture about them, and the census marked them for absorption into the specification on that ground (`tab:assayer:adr-disposition-contracts`).

They are named here so a reader looking for them in the metrics record learns where they went, and they are not restated here in any part. What the surface must summarise is likewise the specification's (`tab:monitoring:observability-summary`).

**Entry (Sentinel coverage metric)** · `entry:metrics:sentinel-coverage`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-metric-sentinel-coverage`). Full health computes the share of the registered Sentinel fleet with a current report and carries it on `SystemHealthReport`; the full-report mapper exports that live fraction as the `assayer_sentinel_coverage` gauge (`src/api/health.rs`, `src/health/summary.rs`).

The completion trigger is met without changing assessment semantics. A focused production-world test registers one reporting and one silent Sentinel and proves both the report and mapped sample read one half (`claim:wellness:full-health-coverage-reads-current-report-presence-across-the-registered-fleet`).

**Entry (Zero-Sentinel assessment counter)** · `entry:metrics:zero-sentinel`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-metric-zero-sentinel-counter`). The existing inline flag now advances a dedicated atomic in `AssessmentDegradationCounters` once per prior-only assessment. Its snapshot reaches `HealthSummary` and the summary mapper exports `assayer_zero_sentinel_assessments_total` as a counter (`src/assessment.rs`, `src/health/counters.rs`).

The completion trigger is met on the existing assessment path. A focused live test proves two prior-only assessments advance the mapped count twice and a subsequent reporting-Sentinel assessment does not move it (`claim:wellness:the-live-assessment-path-counts-each-zero-sentinel-result-exactly-once`).

**Entry (Signal-cache hit-rate and utilisation)** · `entry:metrics:signal-cache`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-metric-signal-cache`). `SystemHealthReport` now carries the cache's complete health snapshot — hits, misses, evictions, size and capacity — instead of retaining the eviction scalar alone. The full-report mapper keeps the existing eviction counter and exports the cache's own derived readings as the `assayer_signal_cache_hit_rate` and `assayer_signal_cache_utilisation` gauges (`src/signal/cache.rs`, `src/api/health.rs`). The cache remains the retention record's (`dec:retention:cache-preencoded`).

The completion trigger is met by schema carry and mapping only. A production cache miss, deferred write and subsequent hit prove the report carries the live statistics and both mapped rates equal the cache's derivations (`claim:wellness:signal-cache-metrics-read-the-live-cache-statistics`).

**Entry (Positive-class prior metrics)** · `entry:metrics:positive-prior`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-metric-p-positive`). Health-summary assembly reads both already-published snapshot trackers and the summary mapper exports them as the `assayer_positive_class_prior_global` and `assayer_positive_class_prior_eligible` gauges (`src/snapshot/published.rs`).

The completion trigger is met without changing either tracker. A live label updates the published snapshot and a focused test proves both values travel unchanged through the health summary and mapped samples (`claim:wellness:positive-class-prior-metrics-read-the-published-snapshot`).

**Entry (Drift reset counter)** · `entry:metrics:drift-resets`

Completed, and the register entry this deferral migrated from is retired at (`entry:assayer:defer-metric-drift-resets`). A dedicated atomic `DriftResetCounter` is shared with the model owner, and the single `emit_drift_reset_event` helper advances it before attempting bounded-channel delivery. Both automatic-threshold and calibration-shift event sites use that helper. Health-summary assembly reads the atomic and the summary mapper exports `assayer_drift_resets_total` as a counter (`src/health/counters.rs`, `src/health/events.rs`).

The completion trigger is met for every emitted drift-reset event, including an event the full channel drops. The focused event-path test proves both delivery outcomes advance the count (`claim:wellness:every-emitted-drift-reset-is-counted-whether-or-not-the-event-channel-delivers-it`).
