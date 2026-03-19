# ADR-M-032: Three-Surface Visibility Model

**Status:** Decided
**Date:** 2026-03-03
**Relates to:** [ADR-M-025](025-public-api-surface.md) (public API surface),
[ADR-M-030](030-graph-module-decomposition.md) (graph module decomposition)
**API:** [§API M-2](../docs/api.md#2-three-surface-visibility-model)
**Surface:** 1 + 2 + 3 (all)

---

## Context

Every ADR in this repository makes visibility decisions — `pub`, `pub(crate)`,
or private — yet no single document states the governing principle behind those
choices. The result is consistent practice but under-theorised rationale:
reviewers must reverse-engineer the policy from 30+ individual decisions.

We need a concise mental model that:

1. Classifies every public symbol into exactly one of a small number of
   _surfaces_.
2. Gives each surface a memorable name so ADR prose can reference it.
3. Makes the _crossing rules_ between surfaces explicit.

---

## Guiding analogy — silver halide film

The crate's operational cycle — expose, regress, measure — corresponds
to the physics of **undeveloped silver halide film** undergoing
**latent image regression**.

In a silver halide emulsion:

1. A **photon strike** liberates an electron in an AgBr crystal, reducing
   Ag⁺ to Ag⁰. Multiple strikes at the same grain build a cluster of
   metallic silver atoms — the **latent image center**.
2. Sub-critical clusters are thermodynamically unstable. Given time (or
   heat) the silver atoms **disperse back** into the halide lattice.
   The grain re-sensitizes and can be exposed again. This is **latent
   image regression**.
3. A **contact print** can be made from the negative at any moment — a
   stable, detached record. The negative itself keeps changing.

PEWEI's `GvGraph` is undeveloped film that is **never fixed**: it is
continuously exposed and regressing. There is no develop/fix/wash
cycle. This single observation anchors the three-surface model.

### Core operation mapping

| PEWEI operation         | Silver halide equivalent                                                                                                                                                                                                                           |
| ----------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `observe(coord, delta)` | Photon strike → latent image center. `delta` accumulation = multi-hit cluster growth on a grain at a specific position on the film plane.                                                                                                          |
| `decay(attenuation, q)` | Latent image regression. Sub-critical silver atom clusters thermally disperse; the grain re-sensitizes. `attenuation` is the survival fraction. `q` selectivity maps to cluster-size threshold — small clusters regress first, large ones persist. |
| `extract()` → `Pewei`   | Contact print from the negative. A stable, detached, `Clone` snapshot (owns heap data). The negative keeps changing underneath.                                                                                                                    |
| `plateaus()`            | Isodensity contour map (densitometry). Measures equal-density zones across the film plane without altering the latent image. `BasisEdge` boundaries are isodensity contours.                                                                       |
| `contour_range(s, e)`   | Microdensitometer strip reading. Scans a section of the negative between two isodensity contour boundaries and decomposes the measurement into a basis set of G-node contributions plus contour range and exact energy.                              |
| `select_plateaus(l, h)` | Aperture selection. Given arbitrary coordinates, snaps outward to the nearest isodensity contour boundaries that span all overlapping density zones, returning endpoints valid for `contour_range()`.                                               |
| `sample()`              | Random developable-grain sampling. Probability a grain is developable is proportional to its exposure — `sample()` returns a random cell with probability proportional to accumulated weight.                                                      |
| `get(coord)`            | Spot densitometer reading at a point.                                                                                                                                                                                                              |

### Extended mapping

| PEWEI concept                        | Silver halide analog                                                                                                                                                                                                                    |
| ------------------------------------ | --------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GvGraph`                            | Undeveloped film — perpetually latent, never fixed                                                                                                                                                                                      |
| `Config` (branching factor, depth)   | Film stock specification — ISO, grain-size distribution, dynamic range; fixed at manufacture before any exposure                                                                                                                        |
| `Accumulator` trait                  | Emulsion chemistry — AgBr, AgCl, AgI, mixed halides; different crystals have different sensitivity profiles and combination rules. Sub-traits (`Attenuatable`, `Weighable`, `Proratable`, `Inspectable`) provide optional capabilities. |
| `Observation` trait                  | Photon characteristics — wavelength, energy; different observations affect the accumulator/emulsion differently                                                                                                                         |
| `Coordinate`                         | Position on the film plane                                                                                                                                                                                                              |
| `GNode` spatial cells                | Individual grains, each occupying a region and accumulating silver independently                                                                                                                                                        |
| `GState`                             | Crystal structure class — isolated grain (terminal), partially-clustered aggregate (semi-internal), fully-subdivided aggregate (internal)                                                                                               |
| `Arena<T>` (slab allocator)          | Gelatin matrix — the transparent physical substrate holding crystals in position. Nobody outside the lab handles gelatin.                                                                                                               |
| `Span`, `Cell`, `Node` (view types)  | Loupe readings — lightweight, informational, detached from the negative                                                                                                                                                                 |
| Contour range decomposition          | Microdensitometer strip analysis — scanning a section of negative and decomposing the density into fully-captured grains, edge grains, and scattered-light fractions                                                                    |
| Subtree-scoped `decay(root, ...)`    | Localized regression via masked heating — physically real (IR laser through a mask), though lab-grade rather than darkroom practice                                                                                                     |
| G-Tree depth = resolution            | Grain size = acuity — fine-grain film (Velvia 50) resolves more than coarse (HP5 at 3200)                                                                                                                                               |
| `rebalance` / `promote` / `contract` | Ostwald ripening during emulsion _manufacture_ — small crystals dissolve and redeposit onto larger ones, redistributing the grain structure                                                                                             |

### Where the analogy breaks

The mapping is **load-bearing** for the expose/regress/print cycle and
the three-surface visibility model. It becomes **illustrative only** at
the tree-structure level. Known breaks:

| PEWEI concept                                          | Why the metaphor fails                                                                                                                                                                                                         |
| ------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| Adaptive spatial subdivision (`split`)                 | Film grain is fixed at manufacture. You cannot grow finer grain where you expose more. PEWEI subdivides on demand. This is the single deepest break — there is no photographic analog for exposure-driven resolution increase. |
| Non-destructive readout (`extract`, `plateaus`, `get`) | You cannot probe a latent image without additional exposure. PEWEI readout is perfectly non-destructive. This is more CCD/CMOS behavior than film.                                                                             |
| Deterministic observation                              | Real photon→latent-image conversion is quantum-probabilistic (~1–3% quantum efficiency). `observe()` is deterministic — every call deposits exactly `delta`.                                                                   |
| Unbounded accumulation                                 | Real grains saturate; once all AgBr is reduced, more photons do nothing (solarization reversal at extreme overexposure). PEWEI accumulators can grow without bound.                                                            |
| Hierarchical tree structure                            | Film is a ~monolayer of randomly scattered grains on a 2D plane. The G-Tree/V-Tree hierarchy has no physical counterpart.                                                                                                      |

---

## Decision

The crate's API is organised into **three surfaces**, named by analogy
with silver halide photography.

### Surface 1 — Prints

Lightweight, read-only **view types** that users hold, inspect, and
store. They are the **contact prints** — stable, self-contained records
detached from the negative. None borrow the graph: no lifetimes, no
`&`-references back into the `GvGraph`. Some are trivially `Copy`
(scalar-sized), others own heap data and are `Clone`-only.

**Spot readings** — single-region measurements from the negative:

| Type         | Sem.   | What it captures                                                                       | Silver halide analog                                                                                                                                                                   |
| ------------ | ------ | -------------------------------------------------------------------------------------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Span<C, V>` | `Copy` | Dyadic interval + single intensity. Purely geometric — no node identity or tree state. | **Densitometer strip reading.** A section of the negative read as one density value over a spatial extent. The simplest possible measurement.                                          |
| `Cell<C, V>` | `Copy` | Terminal G-node snapshot (leaf region). Returned by `sample()`, `get()`.               | **Single-grain loupe reading.** The finest-resolution measurement at a specific point — one grain, one density.                                                                        |
| `Node<C, V>` | `Copy` | Any G-node snapshot — carries `own`, `sum`, and `state`.                               | **Zone reading.** An area measurement that reports both local density (`own`) and aggregate density over all sub-grains (`sum`). The `state` tells you whether the zone has structure. |
| `GState`     | `Copy` | `Terminal` / `SemiInternal` / `Internal` — derived from child pointers, zero storage.  | **Crystal structure class.** Isolated grain (terminal), partially-clustered aggregate (semi-internal), or fully-subdivided aggregate (internal).                                       |

**Contact print** — the full developed image, detached from the negative:

| Type               | Sem.    | What it captures                                                                                                                                                                                                           | Silver halide analog                                                                                                                                                                                                                                                                                              |
| ------------------ | ------- | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- | ----------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Pewei<C, V>`      | `Clone` | Full significance-ordered extraction. Self-contained: no references back to the graph. `domain_start`/`domain_end` record the spatial domain; `layers` ordered by decreasing significance. Owns `Vec<Layer>` — not `Copy`. | **The contact print itself.** Complete developed image detached from the negative. The domain bounds are baked in so the print is self-describing without the original negative. Layers are multi-pass printing — coarse densities first, fine detail overlaid progressively (cf. Kodachrome dye-coupler layers). |
| `Layer<C, V>`      | `Clone` | One BFS depth-level of the V-Tree. Contains `Vec<Transition>` and `Vec<Terminal>` — not `Copy`.                                                                                                                            | **One dye-coupler layer.** Each layer captures one spatial-frequency band of the image, from coarse to fine.                                                                                                                                                                                                      |
| `Transition<C, V>` | `Copy`  | Phase-transition node — a region where finer spatial structure was confirmed. Carries `baseline` (own), `total` (sum), `refinement` (total − baseline).                                                                    | **Crystal-aggregate boundary.** The edge within the emulsion where macro-density (`baseline`) gives way to resolved micro-structure (`refinement`). The aggregate's full energy (`total`) is known, and the fraction attributable to internal structure is separated out.                                         |
| `Terminal<C, V>`   | `Copy`  | Leaf node in the PEWEI — no further subdivision exists.                                                                                                                                                                    | **Fully resolved grain.** The smallest developable unit. No internal structure — a single crystal, a single density.                                                                                                                                                                                              |

**Densitometry** — isodensity contour map and strip analysis:

The G-Tree's bottom contour partitions the film plane into
**isodensity zones** — plateaus of uniform depth. A **contour range
query** scans any lattice-aligned strip of the negative through
these zones and decomposes the reading into fully-captured grain
clusters, edge grains, and scattered-light contributors.

| Type            | Sem.   | What it captures                                                                                            | Silver halide analog                                                                                                                                    |
| --------------- | ------ | ----------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `BasisEdge<C>`  | `Copy` | Coordinate where a plateau's terminal-depth contour begins. `Ord` newtype for `BTreeMap` keys.              | **Isodensity contour boundary.** The coordinate where the emulsion's developing threshold transitions from one density zone to the next.                |
| `Plateau<C, V>` | `Copy` | One plateau on the G-Tree's bottom contour — a contiguous region of uniform depth carrying thatched ranges. | **Isodensity zone.** A region of the negative where all grains sit at the same crystallographic depth — uniform resolution, one characteristic density. |

A strip reading through these zones decomposes into a **basis set**
(ADR-M-037, §CR.2–§CR.6):

| Type                       | Sem.    | What it captures                                                                                                                                          | Silver halide analog                                                                                                                                          |
| -------------------------- | ------- | --------------------------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `BasisElement<C, V>`       | `Copy`  | One element of the minimal G-node cover of a contour range. `is_boundary_thatch` flag marks edge elements whose `.sum` leaks energy outside the range.     | **Grain cluster in the measurement strip.** Most clusters are fully captured; at most two at the edges include scattered-light leakage from outside the strip. |
| `ContourRange<C, V>`       | `Clone` | Full basis-set decomposition plus pre-computed energy fields (§CR.6, §CR.13). Owns a `Vec<BasisElement>` — not `Copy`.                                     | **Densitometry report.** The complete breakdown of a strip reading into basis contributions, contour range energy, and exact energy.                           |
| `ContourRangeEnergy<V>`    | `Copy`  | Energy-only result — scalar fields without the basis set.                                                                                                  | **Summary density reading.** Just the numbers — contour range energy, exact energy, plateau energy — without the per-element attribution.                      |

**Handles** — opaque references back into the emulsion:

| Type      | Sem.   | What it captures                                                           | Silver halide analog                                                                                                                           |
| --------- | ------ | -------------------------------------------------------------------------- | ---------------------------------------------------------------------------------------------------------------------------------------------- |
| `GNodeId` | `Copy` | Opaque handle to a spatial node. `NonZeroU32` index into the G-arena.      | **Grain serial number.** A lab notation that lets you refer back to a specific crystal without exposing its position or chemistry.             |
| `VNodeId` | `Copy` | Opaque handle to a significance node. `NonZeroU32` index into the V-arena. | **Developing priority tag.** Which grain cluster to print first, ordered by intensity — the V-Tree's tournament ranking expressed as a handle. |

**Invariant:** Every Print is `Debug` and carries no mutable
reference into the graph. Most are `Copy`; `Pewei`, `Layer`, and
`ContourRange` are `Clone`-only because they own `Vec`s.

### Surface 2 — Film

The **opaque operational types** through which users expose, measure,
and attenuate the medium — the film you load, the instruments you
meter with, and the chemistry contracts that constrain compatibility.

**Film stock** — manufacturing specification, fixed before loading:

| Type        | Sem.    | What it captures                                                                                                                                                                     | Silver halide analog                                                                                                                                                                                                       |
| ----------- | ------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ | -------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `Config<V>` | `Clone` | Split threshold (θ), depth gates (`D_create`, `D_evict`), budget, relaxation α, `bounded_eviction`. All parameters are fixed at construction; runtime adjustments live on `GvGraph`. | **Film stock specification.** ISO rating, grain-size distribution, dynamic range, reciprocity characteristics — chosen at the store, baked into the emulsion at manufacture. You don't change them after loading the roll. |

**The negative** — the active medium, continuously exposed and regressing:

| Type               | Sem.    | What it captures                                                                                                                      | Silver halide analog                                                                                                                                                                                                                                                                              |
| ------------------ | ------- | ------------------------------------------------------------------------------------------------------------------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `GvGraph<C, V, N>` | `Clone` | The live dual-tree index. Owns both arenas, config, depth gates, violation queue, node counts. `pub(crate)` fields — opaque to users. | **Film loaded in the camera.** The medium is in use — actively accumulating latent image and undergoing regression. Opaque the same way loaded film is: the camera back is closed, you can't see or touch the emulsion directly. You interact only through the metering and exposure instruments. |

**Chemistry contracts** — trait bounds that constrain what emulsions
and photon types are compatible:

| Trait            | What it constrains                                                                                                                         | Silver halide analog                                                                                                                                                                                                                                                                                                                       |
| ---------------- | ------------------------------------------------------------------------------------------------------------------------------------------ | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------ |
| `Coordinate`     | Position type. Must support dyadic arithmetic, total ordering, midpoint, width.                                                            | **Film plane geometry.** The coordinate system of the emulsion surface.                                                                                                                                                                                                                                                                    |
| `Accumulator`    | Intensity type. Must support `zero`, `add`, `sub`.                                                                                         | **Emulsion chemistry.** AgBr, AgCl, AgI — different halide crystals have different sensitivity profiles and combination rules. The `Accumulator` impl _is_ the choice of crystal.                                                                                                                                                          |
| `Attenuatable`   | Multiplicative scaling (`attenuate`). Sub-trait of `Accumulator`. Required by `TemporalDecay`.                                             | **Thermal sensitivity.** How a crystal's latent image responds to heat-driven regression — some halides disperse faster than others.                                                                                                                                                                                                       |
| `Weighable`      | Weight projection to `f64` (`weight`). Sub-trait of `Accumulator`. Required by `WeightedSampler` and `Transition::snr`.                    | **Developability measure.** How exposed a grain is, expressed as a single number — the larger the cluster, the more likely it develops.                                                                                                                                                                                                    |
| `Proratable`     | Fractional subdivision (`prorate`, `scale_by`). Sub-trait of `Accumulator`. Required by range queries, contour range decomposition, and `Pewei::reconstruct`.            | **Partial-grain densitometry.** Estimating the density contribution of a grain that only partially overlaps the measurement aperture.                                                                                                                                                                                                      |
| `Inspectable`    | Diagnostic projection (`to_f64_approx`, `from_f64`). Sub-trait of `Accumulator`. Required by most core operations and `assert_invariants`. | **Lab densitometer calibration.** Converting crystal measurements to a standard numeric scale for logging, display, and quality-control checks.                                                                                                                                                                                            |
| `Observation<V>` | How a single event affects the accumulator.                                                                                                | **Photon characteristics.** Wavelength, energy, spectral interaction with the emulsion. Different observation types change the accumulator differently.                                                                                                                                                                                    |
| `Rng`            | Source of randomness for `sample()` only. User-supplied (`&mut impl Rng`). Not used by `observe()` or `decay()` — those are deterministic. | **Developer diffusion.** When film is dipped in developer, molecules undergo Brownian motion through the gelatin; the probability a molecule initiates reduction at a grain is proportional to the grain's latent image cluster size. The `Rng` is that thermal diffusion — external to the emulsion, driving a weighted random encounter. |

**Instruments** — the metering and exposure interface:

| Trait             | Facet                     | Methods               | Silver halide analog                                                                                                                                                                  |
| ----------------- | ------------------------- | --------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `SpatialRead`     | Densitometry (`&self`)    | `plateaus()`, `get()` | **Densitometer.** Non-destructive measurement of the latent image's density profile.                                                                                                  |
| `WeightedSampler` | Densitometry (`&self`)    | `sample()`            | **Random grain probe.** Pick a grain with probability proportional to its exposure — a statistical metering technique.                                                                |
| `SpatialWrite`    | Exposure (`&mut self`)    | `observe()`           | **Incident light.** Photons striking grains to form latent image centers — the exposure instrument.                                                                                   |
| `TemporalDecay`   | Attenuation (`&mut self`) | `decay()`             | **Thermal regressor.** Heat driving latent image regression — subband-adaptive, so coarse and fine grains can age at different rates. Split from `SpatialWrite` (ADR-M-009 Addendum 3). |

`GvGraph` is opaque — its fields are private; users interact only
through trait impls and inherent methods (e.g. `extract()`, `new()`).

### Surface 3 — Emulsion

`pub(crate)` **internal machinery** — the silver halide crystal
microstructure in gelatin. Users never see or depend on these.

| Module          | Contents                                                                   |
| --------------- | -------------------------------------------------------------------------- |
| `arena`         | `Arena<T>` — typed slab allocator (gelatin matrix)                         |
| `gnode`         | `GNode<C, V>` — full spatial node (grain)                                  |
| `vnode`         | `VNode<V>`, `VKind<V>`, `PackedChildren<V>`                                |
| `gtree`         | G-Tree routing, propagation, recomputation                                 |
| `vtree`         | V-Tree insert, remove, depth, propagation                                  |
| `rebalance`     | Violation detection, promote, contract, resolve                            |
| `split`         | `attempt_split` — subdivision logic                                        |
| `evict`         | `evict_tip`, `scan_for_candidates`                                         |
| `observe`       | `observe()` hot-path implementation                                        |
| `decay`         | `decay()` implementation                                                   |
| `diagnostic`    | Consolidated audit and tracing helpers (ADR-M-028)                           |
| `graph_budget`  | Budget enforcement, depth-gate adjustment, eviction (ADR-M-030 §E)           |
| `graph_extract` | PEWEI extraction and layer iteration (ADR-M-030 §D)                          |
| `graph_plateau` | Plateau mirror maintenance                                                 |
| `graph_query`   | Read-side queries: sampling, point lookup, range sum, contour range decomposition (ADR-M-030 §C, ADR-M-037) |
| `graph_traits`  | Trait impls on `GvGraph`: `SpatialRead`, `SpatialWrite`, etc. (ADR-M-030 §F) |

Two modules are **`pub`** + `#[doc(hidden)]` as testing affordances
but are **not** part of the stable public API:

| Module       | Contents                                               |
| ------------ | ------------------------------------------------------ |
| `invariants` | `assert_invariants`, `dump_gtree`, diagnostic fns      |
| `testing`    | Config presets, plan runner, fluent builder, RNG stubs |

---

## Crossing rules

1. **Print → Film.** Film methods _return_ Prints. A Print never holds
   a `&mut` back into the Film.

2. **Film → Emulsion.** Every Film method is a thin façade that
   delegates to Emulsion functions. The public signature mentions only
   Print and Film types.

3. **Emulsion → Print.** Emulsion code _constructs_ Prints internally
   via struct literals (e.g. `Span { start, end, intensity, depth }`)
   — possible because Print fields are `pub`.

4. **Emulsion ⇛ Film fields.** Emulsion code accesses `GvGraph` fields
   directly (they are `pub(crate)`), never through the public trait
   interface.

---

## ADR tagging convention

Every existing ADR receives a `**Surface:**` metadata line indicating
which surface it primarily governs:

- `Surface: 1 (Prints)` — defines or constrains a view type.
- `Surface: 2 (Film)` — defines or constrains the operational interface.
- `Surface: 3 (Emulsion)` — defines or constrains internal machinery.

ADRs that cross surfaces use the primary surface and note the secondary
in their body text.

---

## Consequences

- **§API M-2** is restructured around the three surfaces instead of a
  flat public/internal split.
- **lib.rs** re-exports are grouped by surface with explanatory comments.
- ADR reviewers can immediately see which surface a decision affects.
- Any ad-hoc visibility punch-lists are retired — the surface tags and
  §API M-2 tables are the single source of truth.
- The analogy section explicitly marks where the metaphor is
  **load-bearing** (expose/regress/print cycle, surface boundaries) vs.
  **illustrative only** (tree structure, adaptive subdivision), so
  future ADR authors know not to lean on the parts that break.
