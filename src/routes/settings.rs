use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::common::WebAppData;
use crate::config;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::OkResponse;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .service(
                web::resource("")
                    .route(web::get().to(get_settings))
                    .route(web::post().to(update_settings_handler)),
            )
            .service(web::resource("/name").route(web::get().to(get_site_name)))
            .service(web::resource("/public").route(web::get().to(get_public_settings))),
    );
}

pub async fn get_settings(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let settings: tokio::sync::RwLockReadGuard<config::TorrustBackend> = app_data.cfg.settings.read().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: &*settings }))
}

pub async fn get_public_settings(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let public_settings = app_data.cfg.get_public().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: public_settings }))
}

pub async fn get_site_name(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let settings = app_data.cfg.settings.read().await;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: &settings.website.name,
    }))
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
pub async fn update_settings_handler(
    req: HttpRequest,
    payload: web::Json<config::TorrustBackend>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let _ = app_data.cfg.update_settings(payload.into_inner()).await;

    let settings = app_data.cfg.settings.read().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: &*settings }))
}
