//! Source-over alpha blending.

use image::Rgba;

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

/// Blend a foreground colour onto `dst` using source-over compositing.
///
/// `src_alpha` is the effective source alpha (glyph coverage × foreground alpha).
pub fn blend_source_over(dst: &mut Rgba<u8>, fg: [u8; 4], src_alpha: u8) {
    let sa = f32::from(src_alpha) / 255.0;
    let da = f32::from(dst.0[3]) / 255.0;
    let out_a = sa + da * (1.0 - sa);

    if out_a == 0.0 {
        return;
    }

    for (i, &fg_component) in fg.iter().enumerate().take(3) {
        let src_c = f32::from(fg_component) / 255.0;
        let dst_c = f32::from(dst.0[i]) / 255.0;
        let blended = (dst_c * da).mul_add(1.0 - sa, src_c * sa) / out_a;
        dst.0[i] = f32_to_u8_sat((blended * 255.0).round());
    }
    dst.0[3] = f32_to_u8_sat((out_a * 255.0).round());
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fully_opaque_overwrites() {
        let mut dst = Rgba([100, 100, 100, 255]);
        blend_source_over(&mut dst, [200, 200, 200, 255], 255);
        assert_eq!(dst, Rgba([200, 200, 200, 255]));
    }

    #[test]
    fn zero_alpha_is_noop() {
        let mut dst = Rgba([100, 100, 100, 255]);
        let original = dst;
        blend_source_over(&mut dst, [200, 200, 200, 255], 0);
        assert_eq!(dst, original);
    }
}
