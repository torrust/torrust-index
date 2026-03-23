# ADR-R-002: Embedded Font Selection

**Status:** Proposed
**Date:** 2026-03-18
**Relates to:** [ADR-R-001](001-dependency-selection.md) (`ab_glyph`
requires a `.ttf` or `.otf` font file — constrains candidate
formats), [ADR-R-003](003-code-structure.md) (`font` module loads
the chosen font via `include_bytes!()`)

## Context

The `render-text-as-image` package must ship with an embedded font
so that it has zero runtime font discovery and no system-level font
dependencies. The font will be compiled into the binary via
`include_bytes!()`.

**Requirements:**

- **Monospaced** — the primary use-case is short labels /
  placeholders where consistent character width simplifies layout.
- **Permissive licence** — the font must be redistributable under
  the project's licence terms (AGPL-3.0-only). Fonts under
  SIL OFL-1.1, Apache-2.0, or the Bitstream Vera licence all
  satisfy this.
- **Reasonable file size** — a single weight/style; the full family
  is not needed.
- **Good ASCII coverage** — broad Unicode coverage is a bonus but
  not required.

## Options Considered

| Option | Font | Licence | Regular `.ttf` size | Notes |
| ------ | ---- | ------- | ------------------- | ----- |
| A. Roboto Mono | Roboto Mono Regular | Apache-2.0 | ~130 KB | Compact, clean monospace; good for labels |
| B. Noto Sans Mono | Noto Sans Mono Regular | OFL-1.1 | ~470 KB | Broad Unicode coverage; larger binary footprint |
| C. DejaVu Sans Mono | DejaVu Sans Mono | Bitstream Vera | ~340 KB | Well-known fallback font; moderate size |

## Decision

*To be decided.* Record the chosen font here once agreed upon.

### Selection criteria weight

Binary size matters because the font is compiled into every build.
Option A is ~3.5× smaller than Option B with sufficient ASCII
coverage for the label use-case. Unless broad Unicode support
becomes a requirement, Option A is the likely choice.

## Consequences

- The chosen `.ttf` file will be committed into
  `packages/render-text-as-image/assets/`.
- The font's licence file must be included alongside it.
- Changing the font later is straightforward — swap the file and
  update the `FONT_BYTES` constant — but constitutes a visual
  breaking change for downstream consumers.
