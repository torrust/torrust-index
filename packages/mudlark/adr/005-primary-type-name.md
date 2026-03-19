# ADR-M-005: Primary Type Name

**Status:** Decided  
**Date:** 2026-02-24  
**Spec:** §§IDEA M-1–20 (dual-tree structure),
§§PEWEI M-1–13 (output snapshot),
§PEWEI M-13 (name expansion rationale)  
**API:** [§API M-5.2](../docs/api.md#52-gvgraphc-v-n--the-negative) (GvGraph),
[§API M-4.2](../docs/api.md#42-contact-print) (Pewei),
[§API M-5.1](../docs/api.md#51-configv) (Config)  
**Relates to:** [ADR-M-006](006-generic-parameters.md) (generic parameters —
governs `GvGraph<C, V, N>` signature),
[ADR-M-021](021-pewei-output-representation.md) (PEWEI output representation),
[ADR-M-032](032-three-surface-model.md) (three-surface visibility model)  
**Surface:** 1 + 2 (Prints, Film)

## Context

The crate needs names for three things: the live dual-tree index
(§§IDEA M-1–20), the extracted snapshot it produces
(§§PEWEI M-1–13), and the crate itself. The names must be consistent
with the three-surface visibility model (ADR-M-032) where Film
(Surface 2) produces Prints (Surface 1).

The crate is published as `torrust-mudlark` within the Torrust
workspace (Rust import path: `torrust_mudlark`). The package
directory retains the codename `mudlark`.

## Options Considered

| Option | Crate     | Graph type | Output type                    | Precedent                |
| ------ | --------- | ---------- | ------------------------------ | ------------------------ |
| A      | `pewei`   | `GvGraph`  | `Pewei`                        | `serde` / `Serializer`   |
| B      | `gvgraph` | `GvGraph`  | `Pewei`                        | crate = primary type     |
| C      | `pewei`   | `GvGraph`  | `Image`                        | plain English            |
| D      | `pewei`   | `GvGraph`  | both (`Pewei` + `Image` alias) | dual names               |
| E      | `pewei`   | `Graph`    | `Pewei`                        | qualified `pewei::Graph` |

## Decision

**Option A.**

| Item           | Name      | Rationale                                                                                     |
| -------------- | --------- | --------------------------------------------------------------------------------------------- |
| Crate          | `pewei`   | Memorable, searchable, names the concept (published as `torrust-mudlark`)                     |
| Primary struct | `GvGraph` | Geometric-Value Graph — names the mechanism (ADR-M-006: `GvGraph<C, V, N>`)                     |
| Output struct  | `Pewei`   | Progressive Entropic-Wavelet Exposure Image — the expansion's final word is literally "image" |
| Config         | `Config`  | Simple, namespaced by crate                                                                   |

Follows the `serde` precedent: crate is `serde`, primary trait is
`Serializer`/`Deserializer`, not `Serde`. Similarly: crate is
`torrust-mudlark`, primary type is `GvGraph`, signature output is `Pewei`.

The naming naturally produces the three-surface metaphor (ADR-M-032):
`GvGraph` is the negative (Surface 2 — Film), `Pewei` is the contact
print (Surface 1 — Prints). `graph.extract()` is making a contact
print from the negative.

## Consequences

- `use torrust_mudlark::GvGraph;` — clear, descriptive import.
- `graph.extract()` returns `Pewei<C, V>` — the negative yields a
  stable, self-contained contact print detached from it (ADR-M-032).
- Crate name ≠ primary type name. Slightly unusual for Rust crates,
  but well-precedented by `serde`, `tokio`, `tracing`.
