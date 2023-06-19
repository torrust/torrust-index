use actix_web::http::StatusCode;
use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::cache::image::manager::Error;
use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::ui::proxy::{load_error_images, map_error_to_image};
use crate::web::api::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/proxy")).service(web::resource("/image/{url}").route(web::get().to(get_proxy_image))),
    );

    load_error_images();
}

/// Get the proxy image.
///
/// # Errors
///
/// This function will return `Ok` only for now.
pub async fn get_proxy_image(req: HttpRequest, app_data: WebAppData, path: web::Path<String>) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await.ok();

    match user_id {
        Some(user_id) => {
            // Get image URL from URL path
            let encoded_image_url = path.into_inner();
            let image_url = urlencoding::decode(&encoded_image_url).unwrap_or_default();

            match app_data.proxy_service.get_image_by_url(&image_url, &user_id).await {
                Ok(image_bytes) => {
                    // Returns the cached image.
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("image/png")
                        .append_header(("Cache-Control", "max-age=15552000"))
                        .body(image_bytes))
                }
                Err(e) =>
                // Returns an error image.
                // Handling status codes in the frontend other tan OK is quite a pain.
                // Return OK for now.
                {
                    Ok(HttpResponse::build(StatusCode::OK)
                        .content_type("image/png")
                        .append_header(("Cache-Control", "no-cache"))
                        .body(map_error_to_image(&e)))
                }
            }
        }
        None => {
            // Unauthenticated users can't see images.
            Ok(HttpResponse::build(StatusCode::OK)
                .content_type("image/png")
                .append_header(("Cache-Control", "no-cache"))
                .body(map_error_to_image(&Error::Unauthenticated)))
        }
    }
}
