# §ADR R-003 — Code Structure and Module Design

## Status

**Accepted**

## Context

The temp implementation in `lib.rs` is a single ~210-line file containing
public types, private helpers, the rendering entry point, and unit tests. This
works but mixes several concerns:

1. **Public API types** — `Rgba`, `RenderParams`, `RenderError`.
2. **Font loading** — the embedded font constant and font construction.
3. **Text layout** — measuring glyph advances, computing image dimensions.
4. **Rasterisation & blending** — drawing glyphs onto a pixel buffer.
5. **PNG encoding** — serialising the pixel buffer to PNG bytes.
6. **Numeric helpers** — saturating float-to-integer casts.

As the package matures we want each concern isolated so it can be tested and
understood independently.

## Decision

Split the crate into the following module tree:

```text
src/
├── lib.rs            — re-exports public API; crate-level docs
├── types.rs          — Rgba, RenderParams, RenderError
├── font.rs           — FONT_BYTES constant, font loading helper
├── layout.rs         — measure_text(), image dimensions
├── render.rs         — render_text_to_png() orchestration
├── blend.rs          — blend_source_over()
├── encode.rs         — PNG encoding wrapper
└── tests/            — crate-level tests (per AGENTS.md §Test Locations)
    └── mod.rs
```

### Module responsibilities

| Module      | Public surface                           | Dependencies              |
|-------------|------------------------------------------|---------------------------|
| `types`     | `Rgba`, `RenderParams`, `RenderError`    | std only                  |
| `font`      | `embedded_font() -> FontRef<'static>`    | `ab_glyph`                |
| `layout`    | `TextLayout { width, height, baseline }` | `ab_glyph`                |
| `blend`     | `blend_source_over()`                    | `image` (pixel type only) |
| `encode`    | `encode_rgba_png()`                      | `image`                   |
| `render`    | `render_text_to_png()`                   | all of the above          |

### `lib.rs` re-exports

```rust
pub use render::render_text_to_png;
pub use types::{Rgba, RenderError, RenderParams};
```

Only these four items form the public API. Everything else is
`pub(crate)` or private.

### Numeric helpers

The `f32_to_u32_sat` and `f32_to_u8_sat` functions are small and only used
inside `render` and `blend`. They stay as private functions within the
modules that use them rather than getting their own module — no need to
over-abstract two-line helpers.

### Tests

Following the project convention (AGENTS.md §Test Locations), crate-level
tests go in `src/tests/`. These test the public API through `render_text_to_png`
and validate PNG output properties. Unit tests for individual modules
(e.g. `blend`, `layout`) remain inline in those modules.

## Consequences

- The flat `lib.rs` is replaced by focused modules, each under ~50 lines.
- The public API surface (`Rgba`, `RenderParams`, `RenderError`,
  `render_text_to_png`) does not change.
- Internal refactors to layout, blending, or encoding do not ripple across
  the whole file.

## See Also

- §SPEC R-1 — render-text-as-image Specification.
- §ADR R-001 — Dependency Selection.
- §ADR R-002 — Embedded Font Selection.
