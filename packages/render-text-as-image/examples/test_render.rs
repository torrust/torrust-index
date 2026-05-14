#![allow(clippy::print_stdout)]

use std::fs;

use torrust_index_render_text_as_image::{RenderParams, Rgba, render_text_to_png};

fn main() {
    let params = RenderParams {
        text: "Image proxy error: URL is unreachable",
        font_size_px: 32.0,
        fg_colour: Rgba::WHITE,
        bg_colour: Rgba([0x33, 0x33, 0x33, 0xFF]),
        padding_em: 0.5,
    };
    let png = render_text_to_png(&params).expect("render failed");
    let path = "/tmp/render-test-output.png";
    fs::write(path, &png).expect("write failed");
    println!("Wrote {path} ({} bytes)", png.len());
}
