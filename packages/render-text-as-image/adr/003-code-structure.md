# ADR-R-003: Code Structure and Module Design

**Status:** Decided
**Date:** 2026-03-18
**Relates to:** [ADR-R-001](001-dependency-selection.md) (dependency
split between `ab_glyph` and `image` — module boundaries follow the
dependency boundaries), [ADR-R-002](002-embedded-font-selection.md)
(the `font` module loads the asset chosen in ADR-R-002)

## Context

The initial implementation was a single ~210-line `lib.rs` mixing
several concerns: public API types, font loading, text layout,
glyph rasterisation, pixel blending, PNG encoding, and numeric
helpers. This works but makes each concern harder to test, read,
and evolve independently.

## Options Considered

| Option | Structure | Pros | Cons |
| ------ | --------- | ---- | ---- |
| A. Single file | Everything in `lib.rs` | Simple; no module plumbing | Concerns are interleaved; hard to test in isolation |
| B. Flat modules | One module per concern, re-exported from `lib.rs` | Each module is focused and short; easy to navigate | Slightly more files |
| C. Nested modules | Group related modules under sub-modules (e.g. `render/blend.rs`) | Extra hierarchy | Over-engineered for a ~200-line crate |

## Decision

**Option B — Flat module tree.**

```text
src/
├── lib.rs       — re-exports public API; crate-level docs
├── types.rs     — Rgba, RenderParams, RenderError
├── font.rs      — FONT_BYTES constant, font loading helper
├── layout.rs    — measure_text(), image dimensions
├── render.rs    — render_text_to_png() orchestration
├── blend.rs     — blend_source_over()
├── encode.rs    — PNG encoding wrapper
└── tests/       — crate-level tests (per AGENTS.md §Test Locations)
    └── mod.rs
```

### Module responsibilities

| Module   | Public surface                        | Dependencies            |
| -------- | ------------------------------------- | ----------------------- |
| `types`  | `Rgba`, `RenderParams`, `RenderError` | std only                |
| `font`   | `embedded_font() -> FontRef<'static>` | `ab_glyph`              |
| `layout` | `TextLayout { width, height, baseline }` | `ab_glyph`           |
| `blend`  | `blend_source_over()`                 | `image` (pixel type)    |
| `encode` | `encode_rgba_png()`                   | `image`                 |
| `render` | `render_text_to_png()`                | all of the above        |

### Public API surface

```rust
pub use render::render_text_to_png;
pub use types::{Rgba, RenderError, RenderParams};
```

Only these four items are `pub`. Everything else is `pub(crate)` or
private.

### Numeric helpers

`f32_to_u32_sat` and `f32_to_u8_sat` are small, single-use helpers.
They stay as private functions within the modules that use them —
a dedicated module would be over-abstraction.

### Test placement

Following the project convention (AGENTS.md §Test Locations),
crate-level tests go in `src/tests/` and exercise the public API
via `render_text_to_png`. Unit tests for individual modules
(`blend`, `layout`) remain inline.

## Consequences

- The flat `lib.rs` is replaced by focused modules, each under
  ~50 lines.
- The public API (`Rgba`, `RenderParams`, `RenderError`,
  `render_text_to_png`) does not change.
- Internal refactors to layout, blending, or encoding do not ripple
  across the whole file.
- Adding a new output format (e.g. SVG) means adding a new module
  without touching existing ones.
