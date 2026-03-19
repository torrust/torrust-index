# §ADR R-001 — Dependency Selection for Text-to-PNG Rendering

## Status

**Accepted** — Option A (`ab_glyph` + `image`)

## Context

The `render-text-as-image` package (§SPEC R-1) needs to produce PNG images
containing rendered text. The previous implementation used the external crate
`text-to-png`. We are replacing it with our own package so we control the
dependency tree and can keep it minimal.

Key constraints:

- **No system-level dependencies** (no FreeType, Fontconfig, Harfbuzz, etc.).
  The package must compile and run on a bare `rust:alpine` image.
- **Embedded font** — ship a single font file (or compile one in) so there is
  zero runtime font discovery.
- **PNG output only** — no need for JPEG, SVG, or any other format.
- **Small text only** — the use-case is short labels / placeholders, not
  paragraphs of rich text.

## Options Evaluated

### Option A — `ab_glyph` + `image`

| Crate      | Purpose                         | Transitive deps |
|------------|---------------------------------|-----------------|
| `ab_glyph` | Font parsing & rasterisation    | ~4              |
| `image`    | In-memory image buffer + PNG    | ~15             |

Pros:
- Pure Rust, no system deps.
- `ab_glyph` is well-maintained and widely used in the ecosystem.
- `image` is the de-facto standard for image I/O in Rust.

Cons:
- `image` pulls in codec support we don't need (JPEG, GIF, …). This can be
  trimmed with `default-features = false, features = ["png"]`.

### Option B — `ab_glyph` + `png` (low-level)

| Crate      | Purpose                     | Transitive deps |
|------------|-----------------------------|-----------------|
| `ab_glyph` | Font parsing & rasterisation| ~4              |
| `png`      | PNG encoder only            | ~3              |

Pros:
- Minimal dependency count.
- `png` is the encoder that `image` itself delegates to.

Cons:
- More manual work: we must compose the pixel buffer ourselves and drive the
  PNG encoder directly.

### Option C — `fontdue` + `png`

| Crate     | Purpose                       | Transitive deps |
|-----------|-------------------------------|-----------------|
| `fontdue` | Font parsing & rasterisation  | ~2              |
| `png`     | PNG encoder only              | ~3              |

Pros:
- `fontdue` is designed for simplicity and speed; very small dep tree.
- Fewest total transitive dependencies.

Cons:
- `fontdue` is less widely adopted than `ab_glyph`.


## Decision

**Option A** — `ab_glyph` + `image` with trimmed features:

```toml
[dependencies]
ab_glyph = "0.2"
image = { version = "0.25", default-features = false, features = ["png"] }
```

`image` is used with `default-features = false` so only the PNG codec is
compiled. This keeps the effective dependency count close to Option B while
retaining the ergonomic `ImageBuffer` API.

## Consequences

- The chosen crates become direct dependencies of
  `torrust-index-render-text-as-image`.

## See Also

- §ADR R-002 — Embedded Font Selection.
