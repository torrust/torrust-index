# ADR-R-001: Dependency Selection for Text-to-PNG Rendering

**Status:** Decided
**Date:** 2026-03-18
**Relates to:** [ADR-R-002](002-embedded-font-selection.md) (font
selection — the chosen rasteriser constrains which font formats are
usable), [ADR-R-003](003-code-structure.md) (module design — module
boundaries follow the dependency split)

## Context

The `render-text-as-image` package needs to produce PNG images
containing rendered text. The previous implementation used the
external crate `text-to-png`. We are replacing it with our own
package to control the dependency tree and keep it minimal.

**Constraints:**

- **No system-level dependencies** — no FreeType, Fontconfig,
  Harfbuzz, etc. The package must compile and run on a bare
  `rust:alpine` image.
- **Embedded font** — ship a single font file (or compile one in)
  so there is zero runtime font discovery.
- **PNG output only** — no need for JPEG, SVG, or any other format.
- **Small text only** — the use-case is short labels / placeholders,
  not paragraphs of rich text.

## Options Considered

| Option | Crates | Transitive deps | Pros | Cons |
| ------ | ------ | --------------- | ---- | ---- |
| A. `ab_glyph` + `image` | Font parsing + rasterisation; image buffer + PNG | ~19 | Pure Rust, no system deps; both crates are well-maintained ecosystem standards | `image` pulls in unused codecs — trimmed with `default-features = false, features = ["png"]` |
| B. `ab_glyph` + `png` | Font parsing + rasterisation; PNG encoder only | ~7 | Minimal dependency count; `png` is the encoder `image` delegates to | More manual work: must compose pixel buffer and drive the encoder directly |
| C. `fontdue` + `png` | Font parsing + rasterisation; PNG encoder only | ~5 | Fewest total transitive deps; `fontdue` designed for simplicity and speed | `fontdue` is less widely adopted than `ab_glyph` |

## Decision

**Option A — `ab_glyph` + `image` with trimmed features.**

```toml
[dependencies]
ab_glyph = "0.2"
image = { version = "0.25", default-features = false, features = ["png"] }
```

`image` is used with `default-features = false` so only the PNG
codec is compiled. This keeps the effective dependency count close to
Option B while retaining the ergonomic `ImageBuffer` API for pixel
manipulation.

### Rationale

- `ab_glyph` is the most widely used pure-Rust font rasteriser.
- `image`'s `ImageBuffer` eliminates manual stride arithmetic and
  provides well-tested PNG encoding.
- Disabling default features removes the JPEG, GIF, BMP, and TIFF
  codecs, making the effective dep tree comparable to the low-level
  options.

## Consequences

- `ab_glyph` and `image` become direct dependencies of the
  `torrust-index-render-text-as-image` crate.
- The `image` feature gate must be maintained — enabling default
  features accidentally would pull in unwanted codecs.
- Future format additions (e.g. WebP) can be enabled by adding a
  single feature flag.
