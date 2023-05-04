use actix_web::{web, HttpRequest, HttpResponse, Responder};

use crate::bootstrap::config::ENV_VAR_DEFAULT_CONFIG_PATH;
use crate::common::WebAppData;
use crate::config::TorrustConfig;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::OkResponse;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/settings")
            .service(
                web::resource("")
                    .route(web::get().to(get_settings))
                    .route(web::post().to(update_settings)),
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

    let settings: tokio::sync::RwLockReadGuard<TorrustConfig> = app_data.cfg.settings.read().await;

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

pub async fn update_settings(
    req: HttpRequest,
    payload: web::Json<TorrustConfig>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let _ = app_data
        .cfg
        .update_settings(payload.into_inner(), ENV_VAR_DEFAULT_CONFIG_PATH)
        .await;

    let settings = app_data.cfg.settings.read().await;

    Ok(HttpResponse::Ok().json(OkResponse { data: &*settings }))
}
