//! Public API types for the `render-text-as-image` crate.

/// Maximum input length in bytes.
pub const MAX_TEXT_BYTES: usize = 256;

/// An RGBA colour value (red, green, blue, alpha), each in `0..=255`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Rgba(pub [u8; 4]);

impl Rgba {
    pub const WHITE: Self = Self([255, 255, 255, 255]);
    pub const BLACK: Self = Self([0, 0, 0, 255]);
}

impl Default for Rgba {
    fn default() -> Self {
        Self::WHITE
    }
}

/// Parameters for [`crate::render_text_to_png`].
pub struct RenderParams<'a> {
    pub text: &'a str,
    pub font_size_px: f32,
    pub fg_colour: Rgba,
    pub bg_colour: Rgba,
    pub padding_em: f32,
}

impl Default for RenderParams<'_> {
    fn default() -> Self {
        Self {
            text: "",
            font_size_px: 20.0,
            fg_colour: Rgba::WHITE,
            bg_colour: Rgba([0x33, 0x33, 0x33, 0xFF]),
            padding_em: 0.4,
        }
    }
}

/// Errors returned by [`crate::render_text_to_png`].
#[derive(Debug)]
pub enum RenderError {
    TextTooLong,
    EncodingFailed(String),
}

impl std::fmt::Display for RenderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TextTooLong => write!(f, "input text exceeds {MAX_TEXT_BYTES} bytes"),
            Self::EncodingFailed(msg) => write!(f, "PNG encoding failed: {msg}"),
        }
    }
}

impl std::error::Error for RenderError {}
