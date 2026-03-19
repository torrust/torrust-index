//! Render short text strings as PNG images.
//!
//! See §SPEC R-1 for the full specification.
//! Module structure defined in §ADR R-003.

mod blend;
mod encode;
mod font;
mod layout;
mod render;
mod types;

#[cfg(test)]
mod tests;

pub use render::render_text_to_png;
pub use types::{RenderError, RenderParams, Rgba};
