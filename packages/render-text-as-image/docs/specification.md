# §SPEC R-1 — render-text-as-image Specification

## Status

**Draft**

## Summary

`render-text-as-image` converts a short UTF-8 string into a PNG image. The
primary consumer is the Torrust Index image-proxy endpoint, which returns an
error-placeholder PNG when the upstream image cannot be fetched or is invalid.

## Motivation

The Index serves cached remote images through a proxy. When an error occurs
(unreachable URL, not an image, quota exceeded, etc.) the proxy must still
return a valid `image/png` response so that `<img>` tags in the UI degrade
gracefully. This package owns that rendering.

## §SPEC R-1.1 — Public API

The crate exposes a single entry-point function:

```rust
pub fn render_text_to_png(params: &RenderParams) -> Result<Vec<u8>, RenderError>;
```

### `RenderParams`

| Field              | Type       | Default      | Description                                  |
|--------------------|------------|--------------|----------------------------------------------|
| `text`             | `&str`     | *(required)* | The string to render (≤ 256 bytes).          |
| `font_size_px`     | `f32`      | `20.0`       | Font size in pixels.                         |
| `fg_colour`        | `Rgba`     | white        | Foreground (text) colour, RGBA.              |
| `bg_colour`        | `Rgba`     | `#333333`    | Background fill colour, RGBA.                |
| `padding_em`       | `f32`      | `0.4`        | Padding on all four sides, in em units.      |

A builder or `Default` impl should make it convenient to set only the fields
that differ from the defaults.

The `padding_em` value is resolved to pixels at render time as
`padding_px = (padding_em * font_size_px).round()`, so padding scales
automatically with font size.

### `RenderError`

| Variant            | Cause                                              |
|--------------------|----------------------------------------------------|
| `TextTooLong`      | Input exceeds the 256-byte limit.                  |
| `EncodingFailed`   | The PNG encoder returned an error.                 |

### `Rgba`

A simple `[u8; 4]` new-type (red, green, blue, alpha). The crate may
re-export `image::Rgba` or define its own — this is an implementation detail
so long as the public type is `Copy + Debug + Default`.

## §SPEC R-1.2 — Rendering Behaviour

1. **Layout** — the text is laid out on a single horizontal line. No
   word-wrapping or line-breaking is performed. The image width is determined
   by the glyph advances of the rendered string plus twice the padding. The
   image height is determined by the font metrics (ascent + descent) plus
   twice the padding.

2. **Font** — a single embedded monospaced font (see ADR-R-002) compiled
   into the binary via `include_bytes!()`. There is no runtime font discovery.

3. **Rasterisation** — glyphs are rasterised by `ab_glyph` (see ADR-R-001).
   The rasteriser's default anti-aliasing is used; no additional quality
   knobs are exposed.

4. **Background** — the entire image is filled with `bg_colour` before any
   glyphs are drawn.

5. **Blending** — glyph coverage values are alpha-blended onto the
   background using the standard "source-over" compositing rule.

6. **Output** — the pixel buffer is encoded as a 32-bit RGBA PNG and
   returned as `Vec<u8>`.

## §SPEC R-1.3 — Constraints

- **No system dependencies** — the crate must compile on a minimal
  `rust:alpine` image with no C libraries beyond libc.
- **No I/O** — the function is pure: it takes parameters and returns bytes.
  It must not touch the filesystem or network.
- **Thread-safe** — `render_text_to_png` must be safe to call from multiple
  threads concurrently (`Send + Sync` params, no interior mutability).
- **Deterministic** — identical inputs must produce byte-identical output
  (the PNG encoder must not embed timestamps or random data).

## §SPEC R-1.4 — Integration Point

The Torrust Index calls this crate from `src/ui/proxy.rs` in the
`map_error_to_image` function, which renders the appropriate error message
for each `cache::image::manager::Error` variant.

## Dependencies

Dependency selection is tracked in ADR-R-001.

## Cross-References

- ADR-R-001 — Dependency Selection for Text-to-PNG Rendering.
- ADR-R-002 — Embedded Font Selection.
