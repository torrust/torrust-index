use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::common::WebAppData;
use crate::config;
use crate::errors::ServiceResult;
use crate::models::response::OkResponse;
use crate::web::api::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/settings"))
            .service(
                web::resource("")
                    .route(web::get().to(get_all_handler))
                    .route(web::post().to(update_handler)),
            )
            .service(web::resource("/name").route(web::get().to(get_site_name_handler)))
            .service(web::resource("/public").route(web::get().to(get_public_handler))),
    );
}

/// Get Settings
///
/// # Errors
///
/// This function will return an error if unable to get user from database.
pub async fn get_all_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let all_settings = app_data.settings_service.get_all(&user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: all_settings }))
}

/// Update the settings
///
/// # Errors
///
/// Will return an error if:
///
/// - There is no logged-in user.
/// - The user is not an administrator.
/// - The settings could not be updated because they were loaded from env vars.
pub async fn update_handler(
    req: HttpRequest,
    payload: web::Json<config::TorrustBackend>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let new_settings = app_data.settings_service.update_all(payload.into_inner(), &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: new_settings }))
}

/// Get Public Settings
///
/// # Errors
///
/// This function should not return an error.
pub async fn get_public_handler(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let public_settings = app_data.settings_service.get_public().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: public_settings }))
}

/// Get Name of Website
///
/// # Errors
///
/// This function should not return an error.
pub async fn get_site_name_handler(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let site_name = app_data.settings_service.get_site_name().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: site_name }))
}
