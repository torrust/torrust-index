//! Crate-level tests for the public API.

use crate::types::MAX_TEXT_BYTES;
use crate::{render_text_to_png, RenderError, RenderParams, Rgba};

// --- Rendering tests ---

#[test]
fn empty_text_produces_valid_png() {
    let params = RenderParams {
        text: "",
        ..Default::default()
    };
    let png = render_text_to_png(&params).unwrap();
    assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn short_text_produces_valid_png() {
    let params = RenderParams {
        text: "hello",
        ..Default::default()
    };
    let png = render_text_to_png(&params).unwrap();
    assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
    assert!(png.len() > 50);
}

#[test]
fn text_too_long_is_rejected() {
    let long = "x".repeat(MAX_TEXT_BYTES + 1);
    let params = RenderParams {
        text: &long,
        ..Default::default()
    };
    assert!(matches!(render_text_to_png(&params), Err(RenderError::TextTooLong)));
}

#[test]
fn deterministic_output() {
    let params = RenderParams {
        text: "test",
        ..Default::default()
    };
    let a = render_text_to_png(&params).unwrap();
    let b = render_text_to_png(&params).unwrap();
    assert_eq!(a, b);
}

#[test]
fn custom_colours_produce_valid_png() {
    let params = RenderParams {
        text: "colour",
        fg_colour: Rgba::BLACK,
        bg_colour: Rgba::WHITE,
        ..Default::default()
    };
    let png = render_text_to_png(&params).unwrap();
    assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
}

#[test]
fn zero_padding_produces_valid_png() {
    let params = RenderParams {
        text: "tight",
        padding_em: 0.0,
        ..Default::default()
    };
    let png = render_text_to_png(&params).unwrap();
    assert_eq!(&png[..4], &[0x89, b'P', b'N', b'G']);
}

// --- Type default / constant tests ---

#[test]
fn rgba_default_is_white() {
    assert_eq!(Rgba::default(), Rgba::WHITE);
}

#[test]
fn rgba_white_has_expected_components() {
    assert_eq!(Rgba::WHITE.0, [255, 255, 255, 255]);
}

#[test]
fn rgba_black_has_expected_components() {
    assert_eq!(Rgba::BLACK.0, [0, 0, 0, 255]);
}

#[test]
fn render_params_default_values() {
    let p = RenderParams::default();
    assert_eq!(p.text, "");
    assert!((p.font_size_px - 20.0).abs() < f32::EPSILON);
    assert_eq!(p.fg_colour, Rgba::WHITE);
    assert_eq!(p.bg_colour, Rgba([0x33, 0x33, 0x33, 0xFF]));
    assert!((p.padding_em - 0.4).abs() < f32::EPSILON);
}

// --- RenderError Display / Error tests ---

#[test]
fn render_error_text_too_long_display() {
    let err = RenderError::TextTooLong;
    let msg = err.to_string();
    assert!(msg.contains("256"), "should mention the byte limit: {msg}");
}

#[test]
fn render_error_encoding_failed_display() {
    let err = RenderError::EncodingFailed("oops".into());
    let msg = err.to_string();
    assert!(msg.contains("oops"), "should contain the inner message: {msg}");
}

#[test]
fn render_error_implements_std_error() {
    let err = RenderError::TextTooLong;
    let _: &dyn std::error::Error = &err;
}

#[test]
fn render_error_debug_format() {
    let err = RenderError::TextTooLong;
    let dbg = format!("{err:?}");
    assert!(dbg.contains("TextTooLong"));
}
