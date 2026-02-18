use std::sync::OnceLock;

use bytes::Bytes;
use text_to_png::TextRenderer;

use crate::cache::image::manager::Error;

const ERROR_IMG_FONT_SIZE: u8 = 16;
const ERROR_IMG_COLOR: &str = "Red";

const ERROR_IMAGE_URL_IS_UNREACHABLE_TEXT: &str = "Could not find image.";
const ERROR_IMAGE_URL_IS_NOT_AN_IMAGE_TEXT: &str = "Invalid image.";
const ERROR_IMAGE_TOO_BIG_TEXT: &str = "Image is too big.";
const ERROR_IMAGE_USER_QUOTA_MET_TEXT: &str = "Image proxy quota met.";
const ERROR_IMAGE_UNAUTHENTICATED_TEXT: &str = "Sign in to see image.";

struct ErrorImages {
    url_is_unreachable: Bytes,
    url_is_not_an_image: Bytes,
    too_big: Bytes,
    user_quota_met: Bytes,
    unauthenticated: Bytes,
}

static ERROR_IMAGES: OnceLock<ErrorImages> = OnceLock::new();

fn get_error_images() -> &'static ErrorImages {
    ERROR_IMAGES.get_or_init(|| ErrorImages {
        url_is_unreachable: generate_img_from_text(ERROR_IMAGE_URL_IS_UNREACHABLE_TEXT),
        url_is_not_an_image: generate_img_from_text(ERROR_IMAGE_URL_IS_NOT_AN_IMAGE_TEXT),
        too_big: generate_img_from_text(ERROR_IMAGE_TOO_BIG_TEXT),
        user_quota_met: generate_img_from_text(ERROR_IMAGE_USER_QUOTA_MET_TEXT),
        unauthenticated: generate_img_from_text(ERROR_IMAGE_UNAUTHENTICATED_TEXT),
    })
}

#[must_use]
pub fn map_error_to_image(error: &Error) -> Bytes {
    let images = get_error_images();
    match error {
        Error::UrlIsUnreachable => images.url_is_unreachable.clone(),
        Error::UrlIsNotAnImage => images.url_is_not_an_image.clone(),
        Error::ImageTooBig => images.too_big.clone(),
        Error::UserQuotaMet => images.user_quota_met.clone(),
        Error::Unauthenticated => images.unauthenticated.clone(),
    }
}

fn generate_img_from_text(text: &str) -> Bytes {
    let renderer = TextRenderer::default();

    Bytes::from(
        renderer
            .render_text_to_png_data(text, ERROR_IMG_FONT_SIZE, ERROR_IMG_COLOR)
            .unwrap()
            .data,
    )
}
