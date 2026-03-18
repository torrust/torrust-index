//! Crate-level tests for the public API.

use crate::types::MAX_TEXT_BYTES;
use crate::{render_text_to_png, RenderError, RenderParams};

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
