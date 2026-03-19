//! Embedded font loading.

use ab_glyph::FontRef;

/// The embedded font (Roboto Mono Regular, Apache-2.0).
const FONT_BYTES: &[u8] = include_bytes!("../assets/RobotoMono-Regular.ttf");

/// Return a reference to the embedded font.
///
/// # Panics
///
/// Panics if the embedded font data is invalid (should never happen).
pub fn embedded_font() -> FontRef<'static> {
    FontRef::try_from_slice(FONT_BYTES).expect("embedded font is valid")
}
