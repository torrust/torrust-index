//! Text measurement and image dimension calculation.

use ab_glyph::{PxScaleFont, ScaleFont as _};

/// The result of measuring a line of text.
pub struct TextLayout {
    /// Total horizontal advance of all glyphs (pixels).
    pub width: f32,
    /// Line height: ascent − descent (pixels).
    pub height: f32,
    /// Distance from the top of the line to the baseline (pixels).
    pub baseline: f32,
}

/// Measure a single line of text at the given scale.
pub fn measure_text<F: ab_glyph::Font>(scaled_font: &PxScaleFont<&F>, text: &str) -> TextLayout {
    let mut total_advance = 0.0_f32;
    let mut last_glyph_id = None;

    for ch in text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        if let Some(prev) = last_glyph_id {
            total_advance += scaled_font.kern(prev, glyph_id);
        }
        total_advance += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }

    let ascent = scaled_font.ascent();
    let descent = scaled_font.descent();

    TextLayout {
        width: total_advance,
        height: ascent - descent,
        baseline: ascent,
    }
}
