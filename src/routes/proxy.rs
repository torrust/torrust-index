use std::sync::Once;

use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use bytes::Bytes;
use text_to_png::TextRenderer;

use crate::cache::image::manager::Error;
use crate::common::WebAppData;
use crate::errors::ServiceResult;

static ERROR_IMAGE_LOADER: Once = Once::new();

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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(web::scope("/proxy").service(web::resource("/image/{url}").route(web::get().to(get_proxy_image))));

    load_error_images();
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

fn load_error_images() {
    ERROR_IMAGE_LOADER.call_once(|| unsafe {
        ERROR_IMAGE_URL_IS_UNREACHABLE = generate_img_from_text(ERROR_IMAGE_URL_IS_UNREACHABLE_TEXT);
        ERROR_IMAGE_URL_IS_NOT_AN_IMAGE = generate_img_from_text(ERROR_IMAGE_URL_IS_NOT_AN_IMAGE_TEXT);
        ERROR_IMAGE_TOO_BIG = generate_img_from_text(ERROR_IMAGE_TOO_BIG_TEXT);
        ERROR_IMAGE_USER_QUOTA_MET = generate_img_from_text(ERROR_IMAGE_USER_QUOTA_MET_TEXT);
        ERROR_IMAGE_UNAUTHENTICATED = generate_img_from_text(ERROR_IMAGE_UNAUTHENTICATED_TEXT);
    });
}

pub async fn get_proxy_image(req: HttpRequest, app_data: WebAppData, path: web::Path<String>) -> ServiceResult<impl Responder> {
    // Check for optional user.
    let opt_user = app_data.auth.get_user_compact_from_request(&req).await.ok();

    let encoded_url = path.into_inner();
    let url = urlencoding::decode(&encoded_url).unwrap_or_default();

    match app_data.image_cache_manager.get_image_by_url(&url, opt_user).await {
        Ok(image_bytes) => Ok(HttpResponse::build(StatusCode::OK)
            .content_type("image/png")
            .append_header(("Cache-Control", "max-age=15552000"))
            .body(image_bytes)),
        Err(e) => unsafe {
            // Handling status codes in the frontend other tan OK is quite a pain.
            // Return OK for now.
            let (_status_code, error_image_bytes): (StatusCode, Bytes) = match e {
                Error::UrlIsUnreachable => (StatusCode::GATEWAY_TIMEOUT, ERROR_IMAGE_URL_IS_UNREACHABLE.clone()),
                Error::UrlIsNotAnImage => (StatusCode::BAD_REQUEST, ERROR_IMAGE_URL_IS_NOT_AN_IMAGE.clone()),
                Error::ImageTooBig => (StatusCode::BAD_REQUEST, ERROR_IMAGE_TOO_BIG.clone()),
                Error::UserQuotaMet => (StatusCode::TOO_MANY_REQUESTS, ERROR_IMAGE_USER_QUOTA_MET.clone()),
                Error::Unauthenticated => (StatusCode::UNAUTHORIZED, ERROR_IMAGE_UNAUTHENTICATED.clone()),
            };

            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("image/png")
                .append_header(("Cache-Control", "no-cache"))
                .body(error_image_bytes))
        },
    }
}
