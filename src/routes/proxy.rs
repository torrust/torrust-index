use std::sync::Once;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::http::StatusCode;
use bytes::Bytes;

use crate::cache::image::manager::Error;
use crate::common::WebAppData;
use crate::errors::ServiceResult;

static ERROR_IMAGE_LOADER: Once = Once::new();
static mut ERROR_IMAGE_UNAUTHENTICATED: Bytes = Bytes::new();

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/proxy")
            .service(web::resource("/image/{url}")
                .route(web::get().to(get_proxy_image)))
    );

    load_error_images();
}

fn load_error_images() {
    ERROR_IMAGE_LOADER.call_once(|| unsafe {
        ERROR_IMAGE_UNAUTHENTICATED = Bytes::from(std::fs::read("resources/images/sign_in_to_see_img.png").unwrap());
    });
}

pub async fn get_proxy_image(req: HttpRequest, app_data: WebAppData, path: web::Path<String>) -> ServiceResult<impl Responder> {
    // Check for optional user.
    let opt_user = app_data.auth.get_user_compact_from_request(&req).await.ok();

    let encoded_url = path.into_inner();
    let url = urlencoding::decode(&encoded_url).unwrap_or_default();

    match app_data.image_cache_manager.get_image_by_url(&url, opt_user).await {
        Ok(image_bytes) => {
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("image/png")
                .append_header(("Cache-Control", "max-age=15552000"))
                .body(image_bytes))
        }
        // todo: add other error images.
        Err(e) => unsafe {
            // Handling status codes in the frontend other tan OK is quite a pain.
            // Return OK for now.
            let (_status_code, error_image_bytes): (StatusCode, Bytes) = match e {
                Error::UrlIsUnreachable => (StatusCode::GATEWAY_TIMEOUT, ERROR_IMAGE_UNAUTHENTICATED.clone()),
                Error::UrlIsNotAnImage => (StatusCode::BAD_REQUEST, ERROR_IMAGE_UNAUTHENTICATED.clone()),
                Error::ImageTooBig => (StatusCode::BAD_REQUEST, ERROR_IMAGE_UNAUTHENTICATED.clone()),
                Error::UserQuotaMet => (StatusCode::TOO_MANY_REQUESTS, ERROR_IMAGE_UNAUTHENTICATED.clone()),
                Error::Unauthenticated => (StatusCode::UNAUTHORIZED, ERROR_IMAGE_UNAUTHENTICATED.clone())
            };

            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("image/png")
                .append_header(("Cache-Control", "no-cache"))
                .body(error_image_bytes))
        }
    }
}
