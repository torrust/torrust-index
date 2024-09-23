use std::sync::Once;

use bytes::Bytes;
use text_to_png::TextRenderer;

use crate::cache::image::manager::Error;

pub static ERROR_IMAGE_LOADER: Once = Once::new();

static mut ERROR_IMAGE_URL_IS_UNREACHABLE: Bytes = Bytes::new();
static mut ERROR_IMAGE_URL_IS_NOT_AN_IMAGE: Bytes = Bytes::new();
static mut ERROR_IMAGE_TOO_BIG: Bytes = Bytes::new();
static mut ERROR_IMAGE_USER_QUOTA_MET: Bytes = Bytes::new();
static mut ERROR_IMAGE_UNAUTHENTICATED: Bytes = Bytes::new();

const ERROR_IMG_FONT_SIZE: u8 = 16;
const ERROR_IMG_COLOR: &str = "Red";

const ERROR_IMAGE_URL_IS_UNREACHABLE_TEXT: &str = "Could not find image.";
const ERROR_IMAGE_URL_IS_NOT_AN_IMAGE_TEXT: &str = "Invalid image.";
const ERROR_IMAGE_TOO_BIG_TEXT: &str = "Image is too big.";
const ERROR_IMAGE_USER_QUOTA_MET_TEXT: &str = "Image proxy quota met.";
const ERROR_IMAGE_UNAUTHENTICATED_TEXT: &str = "Sign in to see image.";

pub fn load_error_images() {
    ERROR_IMAGE_LOADER.call_once(|| unsafe {
        ERROR_IMAGE_URL_IS_UNREACHABLE = generate_img_from_text(ERROR_IMAGE_URL_IS_UNREACHABLE_TEXT);
        ERROR_IMAGE_URL_IS_NOT_AN_IMAGE = generate_img_from_text(ERROR_IMAGE_URL_IS_NOT_AN_IMAGE_TEXT);
        ERROR_IMAGE_TOO_BIG = generate_img_from_text(ERROR_IMAGE_TOO_BIG_TEXT);
        ERROR_IMAGE_USER_QUOTA_MET = generate_img_from_text(ERROR_IMAGE_USER_QUOTA_MET_TEXT);
        ERROR_IMAGE_UNAUTHENTICATED = generate_img_from_text(ERROR_IMAGE_UNAUTHENTICATED_TEXT);
    });
}

#[allow(static_mut_refs)]
pub fn map_error_to_image(error: &Error) -> Bytes {
    // todo: remove "#[allow(static_mut_refs)]" attribute by assigning a owner
    // to the static mutable variables ERROR_IMAGE_*. Maybe the proxy service.

    load_error_images();
    unsafe {
        match error {
            Error::UrlIsUnreachable => ERROR_IMAGE_URL_IS_UNREACHABLE.clone(),
            Error::UrlIsNotAnImage => ERROR_IMAGE_URL_IS_NOT_AN_IMAGE.clone(),
            Error::ImageTooBig => ERROR_IMAGE_TOO_BIG.clone(),
            Error::UserQuotaMet => ERROR_IMAGE_USER_QUOTA_MET.clone(),
            Error::Unauthenticated => ERROR_IMAGE_UNAUTHENTICATED.clone(),
        }
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
