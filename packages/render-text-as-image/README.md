# render-text-as-image

A small, pure-Rust library that renders short text strings into PNG images.
No system-level dependencies (no FreeType, Fontconfig, etc.) — it ships an
embedded monospaced font and works out of the box on minimal containers.

## Usage

```rust
use torrust_index_render_text_as_image::{render_text_to_png, RenderParams};

let params = RenderParams {
    text: "hello world",
    ..Default::default()
};
let png_bytes = render_text_to_png(&params).unwrap();
```

## Documentation

- [Specification](docs/specification.md) (§SPEC R-1)
- [ADR 001 — Dependency Selection](adr/001-dependency-selection.md)
- [ADR 002 — Embedded Font Selection](adr/002-embedded-font-selection.md)
- [ADR 003 — Code Structure](adr/003-code-structure.md)
