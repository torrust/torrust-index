# §ADR R-002 — Embedded Font Selection

## Status

**Proposed**

## Context

The `render-text-as-image` package (§SPEC R-1) must ship with an embedded font
so that it has zero runtime font discovery and no system-level font
dependencies. The font will be compiled into the binary via `include_bytes!()`.

Requirements:

- **Monospaced** — the primary use-case is short labels / placeholders where
  consistent character width simplifies layout.
- **Permissive licence** — the font must be redistributable under the project's
  licence terms (AGPL-3.0-only). Fonts under SIL OFL-1.1, Apache-2.0, or
  the Bitstream Vera licence all satisfy this.
- **Reasonable file size** — a single weight/style; we do not need the full
  family.
- **Good ASCII coverage** — broad Unicode coverage is a bonus but not required.

## Candidates

| Font             | Licence        | Regular .ttf size | Notes                              |
|------------------|----------------|-------------------|------------------------------------|
| Roboto Mono      | Apache-2.0     | ~130 KB           | Monospaced, good for labels.       |
| Noto Sans Mono   | OFL-1.1        | ~470 KB           | Broad Unicode coverage.            |
| DejaVu Sans Mono | Bitstream Vera | ~340 KB           | Well-known fallback font.          |

## Decision

*To be decided.* — record the chosen font here once agreed upon.

## Consequences

- The chosen font file will be committed into
  `packages/render-text-as-image/assets/`.
- The font's licence file must be included alongside it.

## See Also

- §ADR R-001 — Dependency Selection for Text-to-PNG Rendering.
- §SPEC R-1 — render-text-as-image Specification.
