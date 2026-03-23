//! Core rendering orchestration.

use ab_glyph::{Font as _, PxScale, ScaleFont as _};
use image::{ImageBuffer, Rgba, RgbaImage};

use crate::blend::blend_source_over;
use crate::encode::encode_rgba_png;
use crate::font::embedded_font;
use crate::layout::measure_text;
use crate::types::{MAX_TEXT_BYTES, RenderError, RenderParams};

/// Saturating cast from `f32` to `u32`, clamped to `[0, u32::MAX]`.
fn f32_to_u32_sat(v: f32) -> u32 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    if v <= 0.0 {
        0
    } else if v >= f64::from(u32::MAX) as f32 {
        u32::MAX
    } else {
        v as u32
    }
}

/// Saturating cast from `f32` to `u8`, clamped to `[0, 255]`.
fn f32_to_u8_sat(v: f32) -> u8 {
    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    if v <= 0.0 {
        0
    } else if v >= 255.0 {
        255
    } else {
        v as u8
    }
}

/// Render `params.text` into a PNG byte buffer.
///
/// See §SPEC R-1.1 for the full API contract.
///
/// # Errors
///
/// Returns [`RenderError::TextTooLong`] if the input exceeds 256 bytes.
/// Returns [`RenderError::EncodingFailed`] if PNG encoding fails.
///
/// # Panics
///
/// Panics if the embedded font data is invalid (should never happen).
pub fn render_text_to_png(params: &RenderParams<'_>) -> Result<Vec<u8>, RenderError> {
    if params.text.len() > MAX_TEXT_BYTES {
        return Err(RenderError::TextTooLong);
    }

    let font = embedded_font();
    let scale = PxScale::from(params.font_size_px);
    let scaled_font = font.as_scaled(scale);

    let padding_px = f32_to_u32_sat((params.padding_em * params.font_size_px).round());

    // §SPEC R-1.2 — Layout: single horizontal line.
    let layout = measure_text(&scaled_font, params.text);

    let img_w = (f32_to_u32_sat(layout.width.ceil()) + padding_px * 2).max(1);
    let img_h = (f32_to_u32_sat(layout.height.ceil()) + padding_px * 2).max(1);

    // §SPEC R-1.2 — Background fill.
    let bg = Rgba(params.bg_colour.0);
    let mut image: RgbaImage = ImageBuffer::from_pixel(img_w, img_h, bg);

    // §SPEC R-1.2 — Rasterise and blend glyphs.
    let fg = params.fg_colour.0;
    #[allow(clippy::cast_precision_loss)] // padding_px is always small
    let padding_f = padding_px as f32;
    let mut cursor_x = padding_f;
    let baseline_y = padding_f + layout.baseline;

    let mut last_glyph_id = None;
    for ch in params.text.chars() {
        let glyph_id = scaled_font.glyph_id(ch);
        if let Some(prev) = last_glyph_id {
            cursor_x += scaled_font.kern(prev, glyph_id);
        }

        let glyph = glyph_id.with_scale_and_position(scale, ab_glyph::point(cursor_x, baseline_y));

        if let Some(outlined) = scaled_font.outline_glyph(glyph) {
            let bounds = outlined.px_bounds();
            #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
            outlined.draw(|off_x, off_y, coverage| {
                // Use signed intermediates so negative glyph bearings
                // don't get clamped to 0 by a saturating u32 cast.
                let base_x = bounds.min.x.floor() as i32;
                let base_y = bounds.min.y.floor() as i32;
                let sx = base_x + off_x as i32;
                let sy = base_y + off_y as i32;
                // Safety: sx/sy are non-negative after the >= 0 checks.
                #[allow(clippy::cast_sign_loss)]
                if sx >= 0 && sy >= 0 && (sx as u32) < img_w && (sy as u32) < img_h {
                    #[allow(clippy::cast_sign_loss)]
                    let (px, py) = (sx as u32, sy as u32);
                    // §SPEC R-1.2 — source-over alpha blend.
                    let alpha = f32_to_u8_sat((coverage * f32::from(fg[3])).round());
                    let pixel = image.get_pixel_mut(px, py);
                    blend_source_over(pixel, fg, alpha);
                }
            });
        }

        cursor_x += scaled_font.h_advance(glyph_id);
        last_glyph_id = Some(glyph_id);
    }

    // §SPEC R-1.2 — Encode as 32-bit RGBA PNG.
    encode_rgba_png(&image)
}
