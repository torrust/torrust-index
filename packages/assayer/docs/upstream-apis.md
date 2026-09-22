# Assayer Consumption of the Mudlark and Sentinel APIs · `rep:assayer:upstream-api-boundaries`

_How the Assayer uses the public surfaces of `torrust-mudlark` and `torrust-sentinel`. Maps every consumed type, method, and field to its Assayer purpose._

> **Status (Layers 2–4 complete).** The identity layer (`sec:assayer:upstream-ownership-surface`), Sentinel observational surface (`sec:assayer:upstream-observational-surface`), and risk estimation pipeline are realized: maintenance loop, competitive set discovery, observation pipeline, checkpoint snapshot, report ingestion, validation, per-Sentinel feature extraction, Outcome Ledger integration, and model invocation are implemented and tested. The import listings (`sec:assayer:upstream-import-discipline`) show the **realized** imports. The module locations (`sec:assayer:upstream-module-locations`) reflect completed implementation.

Status labels used below:

| Label | Meaning |
|---|---|
| **Implemented** | The field or method is read on the production path today. |
| **Partial** | A subset is read or stored; richer spec diagnostics are still tracked in `deferred.md`. |
| **Specified, deferred** | Required by the spec or ADR trail, but not implemented yet. |
| **Not consumed** | Present in the upstream API but intentionally ignored by the Assayer. |

---

## 1. The Two Relationships · `sec:assayer:upstream-two-relationships`

The Assayer has precisely two relationships with its upstream crates, and they are structurally different:

| Relationship | Type | From | To | Mechanism | Boundary |
|---|---|---|---|---|---|
| Sentinel → Assayer | **Observational** | `torrust-sentinel` | `torrust-assayer` | `BatchReport<u128>` as owned value | Feed-forward: no handle held, no write-back |
| Assayer → Mudlark | **Ownership** | `torrust-assayer` | `torrust-mudlark` | `GvGraph<u128, u64, 128>` as owned struct fields | Structural: full `&mut` access to own instances |

The observational relationship is mediated entirely by detached report types — the Assayer never sees a `SpectralSentinel` instance, never queries one, and never modifies one. The ownership relationship gives the Assayer full control over its identity-layer G-V Graph instances, which it creates, mutates, decays, and destroys throughout their lifecycle.

The feed-forward invariant (`inv:guarantee:feed-forward`), decided at (`dec:ownership:feed-forward`), governs the Sentinel relationship: no Assayer state — risk estimates, model parameters, outcome history, anything — flows back to any Sentinel. The Mudlark relationship is internal: the identity G-V Graphs are the Assayer's own state, not external measurement instruments.

```
                        torrust-mudlark
                        (GvGraph, Config,
                         Cell, Node, GNodeId, …)
                              ↑                ↑
                              │                │
                    torrust-sentinel      torrust-assayer
                    (SpectralSentinel,     (Assayer, models,
                     BatchReport, …)       identity G-V Graphs, …)
                              ↑              /
                              └─ report ────┘
                                 types only
```

---

## 2. Sentinel Report Types — The Observational Surface · `sec:assayer:upstream-observational-surface`

### 2.1 What the Assayer Receives · `sec:assayer:upstream-received-report`

The sole interface between any Sentinel and the Assayer is `receive_sentinel_report(&self, SentinelId, BatchReport<u128>)` (`dec:surface:diagnostic-ack`). The `BatchReport<u128>` is an owned, detached value — the Sentinel that produced it is unreachable.

```rust
// The complete type consumed
pub struct BatchReport<C: Copy + Debug> {
    pub cell_reports:          Vec<CellReport<C>>,         // ← CONSUMED
    pub ancestor_reports:      Vec<CellReport<C>>,         // ← CONSUMED
    pub coordination_reports:  Vec<CoordinationReport<C>>,  // ← CONSUMED
    pub contour:               ContourSnapshot,             // ← PARTIAL (report-level metadata)
    pub health:                HealthReport,                 // ← NOT CONSUMED
    pub analysis_set_summary:  AnalysisSetSummary,          // ← PARTIAL (structure/normalisation metadata)
}
```

Five top-level fields are consumed. The per-cell scoring fields are the primary signal; `contour` and `analysis_set_summary` currently provide report-level structure and normalisation metadata. The inline Sentinel `HealthReport` is Sentinel-internal diagnostics, enumerated at (`tab:boundary:not-consumed`); the Assayer does not read or pass it through.

### 2.2 The Ingestion Pipeline · `sec:assayer:upstream-ingestion-pipeline`

On receiving a report, maintenance completing before the new index becomes visible (`dec:ordering:maintenance-first`) and reception touching no other state (`dec:surface:reception-isolation`), the Assayer:

1. **Validates** the report (structural: dyadic intervals, root present; data quality: NaN/Inf scores)
2. **Builds a `ReportIndex`** — a `HashMap<LedgerKey, ReportCellEntry>` keyed by `(lo: u128, depth: u8)`, not by `GNodeId`
3. **Maintains the Ledger** — creates/tracks/deletes Ledger entries based on which cells appear and disappear
4. **Publishes** the new index via per-Sentinel `ArcSwap`

The resulting `ReportIndex` is then read by `assess()` during per-Sentinel extraction (`alg:runtime:assessment-pipeline`), which is one function rather than a stage pipeline (`dec:ordering:assessment-function`).

> **Layer 3 status.** The ingestion pipeline is fully implemented: `report/ingestion.rs` validates incoming `BatchReport<u128>`, constructs `ReportIndex` from cell and ancestor reports, maintains the Ledger via cell-set change detection, and publishes via `ArcSwap`. Structural and data-quality validation lives in `report/validation.rs`. Rich diagnostic relay for contour and analysis-set metadata is partial; open health-report surfaces are tracked in `deferred.md`.

### 2.3 Per-Cell Extraction: `CellReport<u128>` · `sec:assayer:upstream-cell-report`

The `CellReport` is the primary data carrier. One report exists for each competitive cell that processed observations (in `cell_reports`) and each ancestor tracker (in `ancestor_reports`). The Assayer extracts features from both — competitive cells provide cell-level measurements; ancestor reports provide the multi-scale chain views.

#### Fields consumed by the Assayer · `sec:assayer:upstream-cell-report-consumed`

| `CellReport` Field | Type | Assayer Feature Group | Assayer Use | Reference |
|---|---|---|---|---|
| `start` | `C` | Routing | Maps to `LedgerKey.lo` for spatial indexing | (`def:runtime:ledger-index`) |
| `end` | `C` | Routing | Interval endpoint; validates dyadic property | (`def:runtime:report-index`) |
| `depth` | `u32` | Routing + Structure | Maps to `LedgerKey.depth`; chain length feature | (`tab:extraction:chain-structure`) |
| `sample_count` | `usize` | Batch context | `log(1 + sample_count)` — 1 feature | (`tab:extraction:batch-context`) |
| `rank` | `usize` | Chain structure | `rank / cap` — structural feature | (`tab:extraction:chain-structure`) |
| `energy_ratio` | `f64` | Chain structure | Cell/root energy ratio — 2 features | (`tab:extraction:chain-structure`) |
| `scores` | `AnomalyScores` | Chain z-scores + CUSUMs | 36 features (z-scores + CUSUMs across 4 axes × views) | (`tab:extraction:chain-z-scores`) and (`tab:extraction:chain-cusums`) |
| `maturity` | `TrackerMaturity` | Chain structure | `1 - noise_influence` — maturity feature | (`tab:extraction:chain-structure`) |
| `geometry` | `ScoringGeometry` | Chain structure + Health | `rank / cap` (via `cap`); encoding effectiveness | (`tab:extraction:chain-structure`) and (`def:monitoring:encoding-effectiveness`) |
| `is_competitive` | `bool` | Internal routing | Distinguishes competitive from ancestor-only cells | — |

#### Fields present but not consumed · `sec:assayer:upstream-cell-report-unconsumed`

| `CellReport` Field | Type | Why Not Used |
|---|---|---|
| `gnode_id` | `GNodeId` | Sentinel-internal arena handle; meaningless across process boundary. The Assayer keys by `(start, depth)` instead. |
| `analysis_width` | `usize` | Derivable from `N - depth`; not independently useful |
| `top_singular_value` | `f64` | Subsumed by `energy_ratio` and `rank` for the Assayer's purposes |
| `per_sample` | `Option<Vec<SampleScore>>` | The Assayer reads batch summaries, not per-sample scores (`tab:boundary:not-consumed`) |

### 2.4 Score Extraction: `AnomalyScores` and `ScoreDistribution` · `sec:assayer:upstream-score-distribution`

The four-axis anomaly scores are the heart of per-Sentinel extraction. Each axis produces the same set of fields:

```rust
pub struct AnomalyScores {
    pub novelty:      ScoreDistribution,   // N axis
    pub displacement: ScoreDistribution,   // D axis
    pub surprise:     ScoreDistribution,   // S axis
    pub coherence:    ScoreDistribution,   // C axis
}
```

From each `ScoreDistribution`, the Assayer reads:

| `ScoreDistribution` Field | Type | Assayer Feature | Notes |
|---|---|---|---|
| `max_z_score` | `f64` | **Chain z-scores** (cell, root, max, mean, gradient, spread views) | Primary signal — 24 features across 6 views × 4 axes (`tab:extraction:chain-z-scores`) |
| `mean_z_score` | `f64` | Optional batch-mean extension | Not default; available for deployments needing batch-vs-sample discrimination |
| `baseline.mean` | `f64` | Implicit in z-score | Not extracted independently; consumed through z-score normalisation |
| `baseline.variance` | `f64` | Implicit in z-score | Same |
| `cusum.accumulator` | `f64` | **Chain CUSUMs** (cell, root, max views) | 12 features across 3 views × 4 axes (`tab:extraction:chain-cusums`) |
| `cusum.slow_baseline` | `BaselineSnapshot` | — | Not extracted directly; the CUSUM accumulator captures the drift signal |
| `cusum.steps_since_reset` | `u64` | — | Available for diagnostics but not an extracted feature |
| `clip_pressure` | `f64` | Stored diagnostic field | Not extracted as a feature; richer health relay is deferred |
| `min`, `max`, `mean` | `f64` | — | Raw score statistics; z-scores preferred for cross-cell comparability |

**Key design choice**: the Assayer uses `max_z_score` (the z-score of the per-sample maximum), not `mean_z_score` (the z-score of the batch mean). This preserves sensitivity to isolated extreme observations within high-volume cells (`tab:extraction:chain-z-scores`).

### 2.5 Coordination Reports: `CoordinationReport<u128>` · `sec:assayer:upstream-coordination-report`

Coordination reports detect anomalous patterns in the distribution of scores *across* spatially related cells (`tab:extraction:coordination`). The Assayer extracts 12 features:

| Feature | Source Field | Width |
|---|---|---|
| Per-axis max coordination z-score | `scores.{axis}.max_z_score` | 4 |
| Per-axis max coordination CUSUM | `scores.{axis}.cusum.accumulator` | 4 |
| Coordination concordance | Fraction of active contexts with `max_z > θ_coord` | 1 |
| Active context fraction | `coordination_reports.len()` / capacity | 1 |
| Peak context depth | max `depth` across active reports, normalised | 1 |
| Root context max z | max across axes for the shallowest context | 1 |

#### Fields consumed from `CoordinationReport` · `sec:assayer:upstream-coordination-consumed`

| Field | Type | Assayer use |
|---|---|---|
| `scores` | `AnomalyScores` | Coordination z-scores and CUSUMs |
| `depth` | `u32` | Peak context depth feature |
| `cells_reporting` | `usize` | Concordance computation, context fraction |
| `start`, `end` | `C` | Context identification |

#### Fields present but not consumed · `sec:assayer:upstream-coordination-unconsumed`

| Field | Why not used |
|---|---|
| `gnode_id` | Sentinel-internal handle; context identified by spatial extent |
| `rank`, `energy_ratio`, `top_singular_value` | Coordination-level structural properties; not extracted as features |
| `maturity` | Not independently useful at coordination level |
| `geometry` | Coordination operates at fixed $w = 4$; geometry is invariant |
| `per_member` | Per-member scores within coordination; batch-level z-scores suffice |

### 2.6 Structural Summaries · `sec:assayer:upstream-structural-summaries`

Three report-level structures provide structural context:

#### `ContourSnapshot` · `sec:assayer:upstream-contour-snapshot`

| Field | Current Assayer use | Status | Reference |
|---|---|---|---|
| `plateau_count` | Stored in `ReportLevelData` | Implemented | (`def:runtime:report-index`) |
| `cell_count` | Intended per-Sentinel health diagnostic | Specified, deferred | (`chap:spec:output-structures`) |
| `total_importance` | Stored in `ReportLevelData` | Implemented | (`def:runtime:report-index`) |
| `splits_since_last_report` | Intended diagnostic context for Sentinel-internal dynamics | Specified, deferred | (`def:runtime:report-index`) |
| `net_removals_since_last_report` | Intended diagnostic context for Sentinel-internal dynamics | Specified, deferred | (`def:runtime:report-index`) |

#### `AnalysisSetSummary` · `sec:assayer:upstream-analysis-set-summary`

| Field | Current Assayer use | Status | Reference |
|---|---|---|---|
| `competitive_size` | Stored as `competitive_cell_count` | Implemented | (`chap:spec:output-structures`) |
| `full_size` | Stored as `full_set_size`; used by coordination extraction as context capacity | Implemented | (`tab:extraction:coordination`) |
| `depth_range` | Stored for report-level structure and relayed in full health | Implemented | (`tab:extraction:chain-structure`) |
| `importance_range` | Stored and relayed as structural diagnostic context | Implemented | (`chap:spec:output-structures`) |
| `v_depth_range` | Stored and relayed as structural diagnostic context | Implemented | (`chap:spec:output-structures`) |
| `degenerate_cells_skipped` | Stored and relayed for cells excluded by upstream analysis | Implemented | (`chap:spec:output-structures`) |

#### `HealthReport` · `sec:assayer:upstream-health-report`

The Sentinel's inline health report is **not consumed**; the boundary enumerates it at (`tab:boundary:not-consumed`). The Assayer's `SystemHealthReport` builds per-Sentinel health from Assayer-owned state instead: report presence, report age, the latest full structural report payload, Ledger entry count, Ledger root values, and measured per-Sentinel informativeness, together with the fleet-wide reporting coverage completed at (`entry:metrics:sentinel-coverage`). The informativeness value is no longer placeholder-backed: it is the mean absolute published operational weight over the Sentinel's own slot, completed at (`entry:health:sentinel-informativeness`).

### 2.7 The Feature Extraction Mapping (Complete) · `sec:assayer:upstream-feature-mapping`

How each of the 60 + 2$m_s$ per-Sentinel features (`def:extraction:slot`) maps to Sentinel report fields:

| Feature Group | Width | Primary Sentinel Source | Sentinel Type |
|---|---|---|---|
| Chain z-scores (6 views × 4 axes) | 24 | `CellReport.scores.{axis}.max_z_score` at cell depth and ancestors | `ScoreDistribution` |
| Chain CUSUMs (3 views × 4 axes) | 12 | `CellReport.scores.{axis}.cusum.accumulator` at cell and ancestors | `CusumSnapshot` |
| Chain structure | 8 | `CellReport.rank`, `.energy_ratio`, `.maturity.noise_influence`, `.geometry.cap`, `.depth` | Multiple |
| Coordination | 12 | `CoordinationReport.scores`, `.depth`, `.cells_reporting` | `CoordinationReport` |
| Outcome Ledger | 3 + 2$m_s$ | **Not from Sentinel** — from Assayer's own Outcome Ledger | `LedgerEntry` |
| Batch context | 1 | `CellReport.sample_count` | `CellReport` |

The Outcome Ledger features (`tab:extraction:ledger-features`) are the only per-Sentinel features that do NOT originate from the Sentinel report. They come from the Assayer's own spatial outcome memory, read from the `LedgerEntry` covering the request's coordinate.

### 2.8 Sentinel Types NOT Consumed · `sec:assayer:upstream-sentinel-types-unconsumed`

The following `torrust-sentinel` types are available in the crate's public API but are never imported or used by the Assayer:

| Type | Why not used |
|---|---|
| `SpectralSentinel<C, V, N>` | The Assayer never holds a Sentinel handle (feed-forward invariant) |
| `Sentinel128`, `Sentinel64` | Type aliases for SpectralSentinel — same reason |
| `SentinelConfig<V>` | Sentinel configuration is the host's responsibility; Assayer does not validate or inspect it |
| `NoiseSchedule` | Internal warmup configuration — irrelevant to the report consumer |
| `SvdStrategy` | Internal algorithm choice |
| `AnalysisSet<C, V>` | The Sentinel's analysis selection is opaque to the Assayer; the Assayer observes its effects through the reported cell set |
| `AnalysisEntry<C, V>` | Same |
| `SampleScore` | Per-sample scores; the Assayer reads batch summaries only |
| `MemberScore<C>` | Per-member coordination scores; batch-level z-scores suffice |
| `CellInspection<C>` | Diagnostic inspection type for direct Sentinel queries — no query path exists |
| `AxisBaselineSnapshots` | Same |
| `ConfigError`, `ConfigErrors` | Sentinel config validation — not the Assayer's concern |

---

## 3. Mudlark G-V Graph — The Ownership Surface · `sec:assayer:upstream-ownership-surface`

### 3.1 The Identity Layer's Purpose · `sec:assayer:upstream-identity-purpose`

The Assayer's identity layer (`chap:spec:key-space-identity`), whose graphs the package owns outright (`dec:ownership:graph-custody`), maintains one `GvGraph<u128, u64, 128>` instance per registered identity dimension. Each graph discovers significant ranges in a host-declared key space and provides binary competitive-cell indicators to the Bayesian risk models.

The identity G-V Graph serves the same role as the Sentinel's internal G-V Graph — adaptive spatial partitioning driven by observation volume — but with a critical difference: the Assayer owns these graphs directly, calls `observe()` and `decay()` on them, and reads their structure to determine the competitive set. This is a structural relationship, not an observational one.

> **Status.** The core identity-layer plumbing is implemented and tested: G-V graph instances, maintenance thread, observation channels, competitive set discovery, lifecycle event emission, and checkpoint coordination. The identity memory-record tranche is now source-aligned: competitive publication carries total importance (`entry:memory:total-importance`), assessment reads take an exact pure decayed outcome view (`entry:memory:decay-view`), registration metadata is available to diagnostics (`entry:memory:audit-metadata`), and outcome state survives competitive exit until graph eviction (`entry:memory:warm-start`).

### 3.2 G-V Graph Parameterisation · `sec:assayer:upstream-graph-parameterisation`

All identity dimensions use the same concrete instantiation:

```
GvGraph<u128, u64, 128>
```

| Parameter | Value | Rationale |
|---|---|---|
| `C = u128` | Coordinate type | Fixed at u128 across the Assayer (`dec:ownership:coordinate-width`); every registration's `encode` function returns u128 |
| `V = u64` | Accumulator type | Feed-forward: Δ=1 always, pure volume counting; u64 overflows in ~584 billion years at 1 GHz |
| `N = 128` | Domain bit-width | Domain is $[0, 2^{128})$; wasted depths for narrower key spaces cost ~14 KB per dimension |

### 3.3 Configuration · `sec:assayer:upstream-graph-configuration`

Each identity dimension's G-V Graph is configured via `Config<u64>`:

| `Config` Field | Current Identity Layer Use | Source / Status |
|---|---|---|
| `split_threshold` | Controls when a region earns finer resolution | Host-configured via `IdentityBudget.split_threshold`; the preserved default is `10` |
| `depth_create` | Maximum V-Tree depth for new splits | Host-configured via `IdentityBudget.depth_create`; the compatibility constructor defaults it to the competitive `depth_cutoff` |
| `depth_evict` | Minimum V-Tree depth for eviction eligibility | Host-configured via `IdentityBudget.depth_evict`; the compatibility constructor defaults it to `depth_cutoff + 2` |
| `budget` | Hard ceiling on live G-nodes | Host-configured via `IdentityBudget.max_cells` |
| `alpha_relax` | Depth-gate hysteresis | Assayer default (0.75) |
| `bounded_eviction` | Stop early once under budget | `true` (default) |

The `Config::validate()` method is called at construction, rejecting malformed parameters before the graph is created. The public identity budget also carries the maintenance loop's spatial decay rate. `IdentityBudget::for_depth_cutoff` preserves the values the implementation formerly fixed or derived, while a host can override every threshold and the decay rate before registration (`entry:memory:identity-thresholds`).

### 3.4 Methods Used by the Identity Layer · `sec:assayer:upstream-methods-used`

The identity maintenance loop, the dedicated owner holding every identity graph (`dec:memory:graph-owner`), uses the following `GvGraph` methods:

#### Construction · `sec:assayer:upstream-method-construction`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `new` | `Config<u64> -> Self` | `register_identity_dimension()` | Create graph for new dimension |

#### Observation · `sec:assayer:upstream-method-observation`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `observe` | `&mut self, coord: u128, delta: u64` | Maintenance loop drain cycle | Record entity observation; always `delta = 1` |

**Feed-forward enforcement**: The observation API accepts only a coordinate and a delta. The delta is hardcoded to `1u64` inside the maintenance loop. No anomaly score, risk estimate, model output, or any Assayer-computed value can influence the graph's importance signal (`inv:guarantee:feed-forward`), the observation surface accepting a coordinate and nothing else (`dec:memory:observation-surface`).

#### Temporal Decay · `sec:assayer:upstream-method-decay`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `decay` | `&mut self, root: GNodeId, attenuation: f64, q: f64` | Maintenance loop periodic decay | Decay importance to control competitive set evolution |
| `g_root` | `&self -> GNodeId` | Decay call site | Obtain root handle for global decay |

Decay is called periodically (governed by `decay_interval` in the `DimensionState`). The root handle from `g_root()` specifies global decay across the entire graph. The owner's cadences (`dec:memory:maintenance-cadences`) specify production identity decay as uniform with `q=1.0`; the test-only `ForceDecay` path is not part of the production contract.

#### Competitive Set Discovery · `sec:assayer:upstream-method-extraction`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `extract_to` | `&self, v_depth_limit: u32 -> Pewei<u128, u64>` | Change detection pass | Depth-limited PEWEI extraction for competitive set |
| `total_sum` | `&self -> u64` | Change detection guard and competitive-set publication | Skip extraction when graph has no observations; publish `total_importance` |

The maintenance loop calls `extract_to(depth_cutoff)` (`[MUDLARK-claim:pewei:limiting-extraction-depth-keeps-at-most-that-many-layers-and-no-more-energy-than-a-full-extraction]`) to produce a depth-limited `Pewei` containing only V-tree BFS depths `0..=depth_cutoff`. Every cell — both `Terminal` and `Transition` — in the resulting PEWEI layers forms the competitive set $\mathcal{E}_d$. The loop compares the current set against the previous set (HashSet difference) to detect entries and exits.

The depth limit avoids extracting the deep V-tree layers that account for the bulk of nodes in large trees — savings are exponential in $(D - \text{depth\_cutoff})$ for balanced trees.

**From each `Terminal` and `Transition` in the PEWEI layers:**

| PEWEI Field | Identity Layer Use |
|---|---|
| `start` | Maps to `CompetitiveCellId.lo` |
| `depth` | Maps to `CompetitiveCellId.depth` (cast to `u8`) |

> **Design change from original plan.** The original design called for `layers()` (the BFS iterator yielding `Node` views). The implementation uses `extract_to()` instead because:
> 1. `extract_to()` provides a natural depth cutoff without post-filtering
> 2. The PEWEI structure is reused for checkpoint snapshots
> 3. `layers()` is still available and used in tests for graph inspection

#### Accessors and Health · `sec:assayer:upstream-method-accessors`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `total_sum` | `&self -> u64` | Health metrics, competitive set guard, `CompetitiveSetIndex::total_importance` | (`tab:keyspace:dimension-features`) |
| `extract` | `&self -> Pewei<u128, u64>` | Cell-state cleanup after eviction and checkpoint coordination | Exact full-graph interval census (`entry:memory:warm-start`) |

`total_sum()` is called in production: `compute_competitive_set()` checks it to skip extraction on empty graphs, and competitive publication copies the same value into the index. The logarithmic total-importance feature of (`tab:keyspace:dimension-features`) reads it from the same ArcSwap as active-cell routing rather than reconstructing it (`entry:memory:total-importance`).

Mudlark does not expose a single-interval membership accessor. The maintenance owner therefore uses the equivalent exact query: a full `extract()` and a census of every terminal and transition interval. State is cleaned only when its interval is absent from that census, so competitive exit can retain outcome state without retaining it past G-V Graph eviction (`entry:memory:warm-start`).

#### Checkpoint Support · `sec:assayer:upstream-method-checkpoint`

| Method | Signature | Where Called | Purpose |
|---|---|---|---|
| `extract` | `&self -> Pewei<u128, u64>` | Checkpoint coordination | Capture full graph structure for serialisation |
| `from_observations` | `Config<V>, impl Iterator<Item=(C,V)> -> Self` | Checkpoint restoration | Reconstruct graph from snapshot |

The maintenance loop publishes graph snapshots via `ArcSwap` during checkpoint coordination, competitive sets publishing per dimension under the swap discipline (`dec:memory:competitive-publication`). The snapshot wraps the full `Pewei` (terminals *and* transition baselines) produced by `extract()`. This preserves total energy across serialization round-trips — earlier terminal-only approaches lost baseline energy at internal nodes.

Reconstruction uses `from_observations()` fed by `IdentityGraphSnapshot::reconstruction_observations()`, which yields `(coordinate, delta)` pairs from both terminals and transitions. The V-tree structure after reconstruction may differ (observation order affects tournament rankings), but the competitive set is equivalent because it depends on the importance distribution.

#### Manual Eviction · `sec:assayer:upstream-method-eviction`

| Method | Signature | Available | Called in Layer 2 |
|---|---|---|---|
| `check_evictions` | `&mut self -> u32` | Yes | No |

Available for explicit maintenance passes after decay. Not called in the current implementation — `observe()` handles budget-guarded eviction internally, and decay-induced evictions are detected through competitive set change detection rather than explicit eviction calls.

### 3.5 Methods NOT Used by the Identity Layer · `sec:assayer:upstream-methods-unused`

The following `GvGraph` methods are available but not called by the Assayer's identity layer in production code:

| Method | Why not used |
|---|---|
| `get` | Point queries use the published `CompetitiveSetIndex::find_active()`, not the live graph |
| `layers` | Replaced by `extract_to()` for competitive set discovery (depth-limited PEWEI is more efficient; see (`sec:assayer:upstream-methods-used`)). Used in tests for graph inspection. |
| `layers_to` | Available but not needed — `extract_to()` serves the same depth-limiting purpose and produces a reusable `Pewei` |
| `sample` | The identity layer discovers significance through PEWEI extraction, not random sampling |
| `range_sum` | No range-sum queries needed; importance tracked through `total_sum()` |
| `contour_range` | No strip-query decomposition needed; competitive set is the primary output |
| `contour_range_energy` | Same |
| `plateaus` | Identity graphs are pure volume counters (Δ=1); the retired $E_s$ encoding effectiveness formula (rank headroom, coordination activity, plateau efficiency) was designed for Sentinels with SVD trackers and coordination contexts — none of which exist in the identity layer — and no reconstruction of it replaced it here (`entry:health:encoding-effectiveness`). What the identity layer has instead is the successor reading's own per-dimension restriction (`def:monitoring:dimension-informativeness`), the mean absolute published operational weight over a dimension's own block, read beside the identity-health metrics (`tab:monitoring:dimension-health`) — among them the cell weight mass, the same fold taken over the dimension's competitive-indicator positions instead, and the coverage share — which are the direct measures of identity dimension value. |
| `select_plateaus` | No lattice-aligned queries |
| `is_ancestor_of` | Ancestry relationships determined through interval containment on `CompetitiveCellId`, not G-node handles |
| `gnode_info` | Individual node inspection not needed; PEWEI extraction provides everything for competitive set discovery |
| `gnode_children` | Refinement topology is internal; the identity layer cares about significance ranking, not child linkage |
| `build_plateaus` | Same as `plateaus` — identity graphs have no use for plateau structure |
| `reset` | Graphs are deregistered and recreated, not reset |
| `check_evictions` | Available but not called; budget enforcement happens internally through `observe()` |

### 3.6 Mudlark Trait Requirements · `sec:assayer:upstream-trait-requirements`

The identity layer's concrete parameterisation `GvGraph<u128, u64, 128>` satisfies all required trait bounds through the built-in implementations:

| Trait | Required by | Satisfied by |
|---|---|---|
| `Coordinate` (for `u128`) | All `GvGraph` methods | Built-in `u128` impl |
| `Accumulator` (for `u64`) | All `GvGraph` methods | Built-in `u64` impl |
| `Inspectable` (for `u64`) | `observe()`, `extract()`, `extract_to()` | Built-in `u64` impl |
| `Attenuatable` (for `u64`) | `decay()` | Built-in `u64` impl |

The Assayer never implements any Mudlark trait. It consumes only built-in type instantiations.

### 3.7 Mudlark Types Consumed by the Identity Layer · `sec:assayer:upstream-mudlark-types-consumed`

| Type | Surface | How the Assayer Consumes It | Realized in L1 |
|---|---|---|---|
| `GvGraph<u128, u64, 128>` | Surface 2 (Film) | Owned per dimension; full `&mut` access | ✓ |
| `Config<u64>` | Surface 2 (Film) | Constructed at dimension registration | ✓ |
| `Pewei<u128, u64>` | Surface 1 (Print) | From `extract()` and `extract_to()` for checkpoint and competitive set discovery | ✓ |
| `GNodeId` | Surface 1 (Print) | Passed to `decay()` via `g_root()`; never stored as persistent key | ✓ (implicit via `g_root()` return) |
| `GState` | Surface 1 (Print) | Used in tests for graph inspection (`layers()` filtering) | Tests only |
| `Node<u128, u64>` | Surface 1 (Print) | Yielded by `layers()` in tests; production competitive set uses PEWEI types | Tests only |
| `Cell<u128, u64>` | Surface 1 (Print) | Available from `get()` but NOT used | — |

### 3.8 Mudlark Types NOT Consumed · `sec:assayer:upstream-mudlark-types-unconsumed`

| Type | Why not used |
|---|---|
| `Span<u128, u64>` | No span-based queries |
| `ContourRange<u128, u64>` | No contour-range decomposition |
| `ContourRangeEnergy<u64>` | Same |
| `BasisElement<u128, u64>` | No basis-set decomposition |
| `BasisEdge<u128>` | No plateau queries — identity graphs have no use for plateau structure (see (`sec:assayer:upstream-methods-unused`)) |
| `Plateau<u128, u64>` | Same |

---

## 4. The Feed-Forward Boundary · `sec:assayer:upstream-feed-forward-boundary`

### 4.1 Sentinel → Assayer (Report Types) · `sec:assayer:upstream-boundary-sentinel`

The feed-forward invariant is enforced **at the type level**, mechanically rather than by review (`dec:ownership:enforcement`):

| Enforcement mechanism | What it prevents |
|---|---|
| No `SpectralSentinel` import | Cannot hold a handle to any Sentinel |
| `BatchReport<u128>` is `Clone + Send` (owned) | Report is detached from the producing Sentinel |
| `receive_sentinel_report(&self, SentinelId, BatchReport<u128>)` | The `&self` receiver provides no `&mut Sentinel` path |
| CI lint: `! grep -rn 'SpectralSentinel' packages/assayer/src/` | Automated enforcement in CI |

**No Assayer output influences any Sentinel.** Risk estimates, model parameters, outcome history, exploration signals, resonance derivations — nothing computed by the Assayer is visible to any Sentinel. Sentinels produce reports; the Assayer reads them. The arrow is unidirectional.

### 4.2 Assayer → Identity G-V Graphs (Owned State) · `sec:assayer:upstream-boundary-graphs`

The feed-forward invariant within the Assayer is enforced **at the API level**, the observation surface accepting a coordinate and nothing else (`dec:memory:observation-surface`):

| Enforcement mechanism | What it prevents |
|---|---|
| Observation API: `graph.observe(coord, 1u64)` — only coordinate + fixed Δ | Anomaly scores, risk estimates, and model outputs cannot influence importance |
| `1u64` hardcoded in maintenance loop | No runtime path can change the observation delta |
| Observation deferred via bounded channel | `assess()` sends coordinates via `try_send`; the maintenance loop processes them independently |

The identity G-V Graph's spatial structure evolves based solely on observation volume — which entities appear and how often. The competitive set reflects traffic patterns, not risk judgments. This is the same design as the Sentinel's internal G-V Graph: importance = accumulated Δ=1 observations, never anomaly scores.

> **Layer 2 verified.** The `drain_observations()` method in `maintenance_loop.rs` calls `graph.observe(coord, 1u64)` with the hardcoded delta. The bounded channel (`crossbeam_channel` with 100K capacity) and `try_send` are implemented in `observation.rs`.

---

## 5. The `GNodeId` Policy · `sec:assayer:upstream-gnodeid-policy`

`GNodeId` appears in both relationships, but the Assayer treats it differently in each:

### 5.1 From Sentinel Reports · `sec:assayer:upstream-gnodeid-from-reports`

`CellReport.gnode_id` and `CoordinationReport.gnode_id` are **ignored**. These are arena handles into the Sentinel's internal G-V Graph — they identify slots in an arena the Assayer cannot access. The Assayer keys all spatial state by `(start, depth)` pairs, which are stable dyadic interval identifiers independent of any arena.

The `GNodeId` fields are present in the received types because `CellReport` is a generic Surface 1 type designed for multiple consumers (including the Sentinel's own diagnostics). The Assayer reads through them without storing or referencing them.

### 5.2 From Owned Identity G-V Graphs · `sec:assayer:upstream-gnodeid-from-graphs`

`GNodeId` values from the Assayer's own G-V Graph instances are used in one narrow context:

| Use | Method | Lifetime |
|---|---|---|
| Decay root | `g_root()` → `decay(root, ...)` | Transient — used and discarded within the same maintenance cycle |

**`GNodeId` is never stored as a persistent key** in any Assayer data structure. The `CompetitiveCellId { lo: u128, depth: u8 }` type serves as the stable spatial identifier, the two spatial key types staying distinct (`dec:memory:key-types-distinct`), mirroring the `LedgerKey { lo: u128, depth: u8 }` pattern the Outcome Ledger keys by coordinate and depth (`dec:memory:coordinate-depth-key`). Both use dyadic interval coordinates that remain meaningful across graph mutations (splits, evictions, absorptions) — unlike `GNodeId`, which may become stale after any structural operation.

The root handle from `g_root()` is structurally safe — the root can never be evicted.

> **Design change.** The original plan expected `GNodeId` to appear in `layers()` output during competitive set discovery. The switch to `extract_to()` means `GNodeId` is no longer encountered during competitive set computation — only via `g_root()` for decay.

---

## 6. Import Discipline · `sec:assayer:upstream-import-discipline`

### 6.1 From `torrust-sentinel` · `sec:assayer:upstream-imports-sentinel`

**Realized imports:**

```rust
// report/ingestion.rs
use torrust_sentinel::BatchReport;

// report/validation.rs
use torrust_sentinel::{
    BatchReport,
    CellReport,
    CoordinationReport,
    ScoreDistribution,
};

// testing/reports.rs (test utilities)
use torrust_sentinel::{
    AnalysisSetSummary,
    AnomalyScores,
    BaselineSnapshot,
    BatchReport,
    CellReport,
    ClipPressureDistribution,
    ContourSnapshot,
    CoordinationHealth,
    CoordinationReport,
    CusumSnapshot,
    GeometryDistribution,
    HealthReport,
    MaturityDistribution,
    RankDistribution,
    ScoreDistribution,
    ScoringGeometry,
    TrackerMaturity,
};
```

**Never imported from `torrust-sentinel`:**

```rust
// PROHIBITED — enforced by CI lint
use torrust_sentinel::sentinel::*;        // Never
use torrust_sentinel::SpectralSentinel;   // Never
use torrust_sentinel::Sentinel128;        // Never
use torrust_sentinel::SentinelConfig;     // Never
use torrust_sentinel::NoiseSchedule;      // Never
use torrust_sentinel::analysis_set::*;    // Never
```

### 6.2 From `torrust-mudlark` · `sec:assayer:upstream-imports-mudlark`

**Realized imports:**

```rust
// identity/maintenance_loop.rs
use torrust_mudlark::{Config as MudlarkConfig, GvGraph};

// identity/snapshot.rs
use torrust_mudlark::Pewei;

// src/tests/identity_maintenance.rs and testing/maintenance.rs (test-only)
use torrust_mudlark::{Config as MudlarkConfig, GState, GvGraph};
```

The Assayer does NOT import `Weighable`, `Proratable`, `Rng`, or any of the instrument traits (`SpatialRead`, `SpatialWrite`, `TemporalDecay`, `WeightedSampler`) — it calls `GvGraph` methods directly, not through trait bounds. No Mudlark traits are imported in Layer 2.

### 6.3 `GNodeId` Import Resolution · `sec:assayer:upstream-imports-gnodeid`

`GNodeId` is imported from `torrust_mudlark` in `testing/reports.rs` for constructing Sentinel report fixtures. In production code, it appears only as the return type of `g_root()` and is used transiently as an argument to `decay()`. The import comes from `torrust_mudlark::GNodeId` (the source crate), not from Sentinel re-exports.

---

## 7. The Abstraction Boundary · `sec:assayer:upstream-abstraction-boundary`

The Assayer interacts with the Sentinel at the **report boundary** and with the Mudlark at the **method boundary**. These are very different abstraction levels:

### 7.1 What the Assayer Knows About Sentinels · `sec:assayer:upstream-knowledge-sentinels`

The Assayer knows:
- Reports contain cells with four-axis anomaly scores, baselines, CUSUMs, maturity, and geometry
- Reports contain coordination summaries across spatially related cells
- Reports contain structural metadata (contour snapshot, analysis set summary)
- Cells have dyadic intervals identified by `(start, end, depth)`
- Competitive cells and ancestor cells are distinguished by `is_competitive`
- The root cell (depth 0) is always present

The Assayer does **not** know or reason about:
- How the Sentinel's G-V Graph partitions the domain
- How the competitive set is selected ($K$-top selection, V-Tree depth cutoff)
- How the warm-up pipeline operates (noise injection, staging, maturity convergence)
- How the subspace tracker works (SVD, rank adaptation, forgetting)
- How scores are computed (projection, residual, Mahalanobis, coherence)
- How baselines evolve (EWMA, clip pressure, slow baseline)
- How coordination contexts are formed (internal G-node subtrees)
- Why cells appear or disappear between reports

### 7.2 What the Assayer Knows About Its Own G-V Graphs · `sec:assayer:upstream-knowledge-graphs`

The Assayer knows:
- How to create a graph with specific configuration
- How to feed observations (coordinate + Δ=1)
- How to apply temporal decay (attenuation, selectivity)
- How to extract significance-ordered structure via PEWEI
- How to determine the competitive set from depth-limited extraction
- How to read structural health (total sum)
- How to snapshot state for checkpointing and reconstruct from snapshots

The Assayer does **not** need to know:
- How the V-Tree rebalances after observations
- How splits are triggered and executed
- How evictions work internally
- How budget enforcement adjusts depth gates
- How the arena manages memory

This is the designed separation: the G-V Graph handles the spatial intelligence; the Assayer harvests the results.

---

## 8. Module Location in `torrust-assayer` · `sec:assayer:upstream-module-locations`

The per-crate API consumption is localised to specific Assayer modules:

### Sentinel Report Types · `sec:assayer:upstream-modules-sentinel`

| Assayer Module | Sentinel Types Used | Purpose | Status |
|---|---|---|---|
| `report/index.rs` | Assayer-side `ReportCellEntry`, `AxisScoreSet`, etc. | Type definitions and depth-walk routing | ✓ L1 |
| `report/slot.rs` | `SentinelSlot` with `ArcSwap<ReportIndex>` | Per-Sentinel slot management | ✓ L1 |
| `report/ingestion.rs` | `BatchReport<u128>` → `ReportIndex` | Entry point; validation pipeline | ✓ L2 |
| `report/validation.rs` | `BatchReport`, `CellReport`, `CoordinationReport`, `ScoreDistribution` | Structural and data-quality checks | ✓ L2 |
| `extraction/chain.rs` | `ReportCellEntry` (Assayer type) | Z-score extraction across ancestor chain | ✓ L2 |
| `extraction/cusum.rs` | `ReportCellEntry` (Assayer type) | CUSUM feature extraction | ✓ L2 |
| `extraction/structure.rs` | `ReportCellEntry` (Assayer type) | Structural features (rank, energy, maturity) | ✓ L2 |
| `extraction/coordination.rs` | `CoordinationEntry` (Assayer type) | Coordination features | ✓ L2 |
| `extraction/alarm.rs` | `ReportCellEntry`, `CoordinationEntry` (Assayer types) | `SentinelAlarmSummary` computation | ✓ L2 |
| `testing/reports.rs` | Full Sentinel report types | Golden report fixtures and builders | ✓ L2 |

### Mudlark G-V Graph · `sec:assayer:upstream-modules-mudlark`

| Assayer Module | Mudlark Types Used | Purpose | Status |
|---|---|---|---|
| `identity/maintenance_loop.rs` | `GvGraph`, `Config` (as `MudlarkConfig`) | Graph ownership, observe, decay, extract_to, extract, g_root | ✓ L1 |
| `identity/snapshot.rs` | `Pewei` | Checkpoint snapshot wrapping and reconstruction | ✓ L1 |
| `identity/competitive.rs` | (no direct Mudlark imports) | Competitive set discovery from PEWEI data | ✓ L1 |
| `identity/dimension.rs` | (no direct Mudlark imports) | Dimension configuration | ✓ L1 |
| `identity/observation.rs` | (none — sends coordinates via channel) | Observation deferral | ✓ L1 |

---

## 9. Versioning and Compatibility · `sec:assayer:upstream-versioning`

| Dependency | Version constraint | What breaks the Assayer on upstream change |
|---|---|---|
| `torrust-mudlark` | Workspace path | Any change to `GvGraph::observe()`, `decay()`, `extract()`, `extract_to()`, `new()`, `g_root()` signatures; any change to `Config`, `Pewei` field layout |
| `torrust-sentinel` | Workspace path (report types only) | Any change to `BatchReport`, `CellReport`, `CoordinationReport` field names or types; removal of any consumed field |

Neither dependency is versioned through `crates.io` — both are workspace-local path dependencies. Breaking changes are caught at compile time by the same workspace build.

**Safe upstream changes (do not affect the Assayer):**

- Adding new fields to `CellReport`, `CoordinationReport`, or `BatchReport` — the Assayer destructures only the fields it needs
- Adding new methods to `GvGraph` — the Assayer calls only its own subset
- Changes to Sentinel-internal types (`SubspaceTracker`, `StagingArea`, etc.) — invisible behind the report boundary
- Changes to Mudlark Surface 3 types (arena internals, rebalance logic, etc.) — invisible behind the method boundary
- Performance changes in `observe()`, `decay()`, `extract_to()` — the Assayer's identity layer tolerates any wall-clock cost; maintenance runs on a dedicated thread
