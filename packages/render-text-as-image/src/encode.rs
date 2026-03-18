//! PNG encoding.

use image::{ImageEncoder as _, RgbaImage};

use crate::types::RenderError;

/// Encode an RGBA image buffer as PNG bytes.
///
/// # Errors
///
/// Returns [`RenderError::EncodingFailed`] if the PNG encoder fails.
pub fn encode_rgba_png(image: &RgbaImage) -> Result<Vec<u8>, RenderError> {
    let (w, h) = image.dimensions();
    let mut buf = Vec::new();
    let encoder = image::codecs::png::PngEncoder::new(&mut buf);
    encoder
        .write_image(image.as_raw(), w, h, image::ExtendedColorType::Rgba8)
        .map_err(|e| RenderError::EncodingFailed(e.to_string()))?;
    Ok(buf)
}
