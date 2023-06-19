use actix_web::http::StatusCode;
use actix_web::{web, HttpResponse, Responder};

use crate::errors::ServiceResult;
use crate::services::about::{index_page, license_page};
use crate::web::api::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/about"))
            .service(web::resource("").route(web::get().to(get)))
            .service(web::resource("/license").route(web::get().to(license))),
    );
}

/// Get About Section HTML
///
/// # Errors
///
/// This function will not return an error.
#[allow(clippy::unused_async)]
pub async fn get() -> ServiceResult<impl Responder> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(index_page()))
}

/// Get the License in HTML
///
/// # Errors
///
/// This function will not return an error.
#[allow(clippy::unused_async)]
pub async fn license() -> ServiceResult<impl Responder> {
    Ok(HttpResponse::build(StatusCode::OK)
        .content_type("text/html; charset=utf-8")
        .body(license_page()))
}
