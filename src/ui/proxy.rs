use std::sync::LazyLock;

use bytes::Bytes;
use torrust_index_render_text_as_image::{render_text_to_png, RenderParams};

use crate::cache::image::manager::Error;

struct ErrorImages {
    url_is_unreachable: Bytes,
    url_is_not_an_image: Bytes,
    image_too_big: Bytes,
    user_quota_met: Bytes,
    unauthenticated: Bytes,
}

fn render_error_png(text: &str) -> Bytes {
    let params = RenderParams {
        text,
        ..Default::default()
    };

    render_text_to_png(&params).map_or_else(
        |e| {
            tracing::error!("Failed to render error image: {e}");
            // 1×1 transparent PNG fallback (67 bytes).
            Bytes::from_static(include_bytes!("fallback_1x1.png"))
        },
        Bytes::from,
    )
}

static ERROR_IMAGES: LazyLock<ErrorImages> = LazyLock::new(|| ErrorImages {
    url_is_unreachable: render_error_png("Image proxy error: URL is unreachable"),
    url_is_not_an_image: render_error_png("Image proxy error: URL is not an image"),
    image_too_big: render_error_png("Image proxy error: image too big"),
    user_quota_met: render_error_png("Image proxy error: user quota met"),
    unauthenticated: render_error_png("Image proxy error: unauthenticated"),
});

/// Maps a cache image error to a PNG image with an error message rendered as
/// text.
///
/// The rendered images are cached on first use via [`LazyLock`].
#[must_use]
pub fn map_error_to_image(error: &Error) -> Bytes {
    let images = &*ERROR_IMAGES;
    match error {
        Error::UrlIsUnreachable => images.url_is_unreachable.clone(),
        Error::UrlIsNotAnImage => images.url_is_not_an_image.clone(),
        Error::ImageTooBig => images.image_too_big.clone(),
        Error::UserQuotaMet => images.user_quota_met.clone(),
        Error::Unauthenticated => images.unauthenticated.clone(),
    }
}
