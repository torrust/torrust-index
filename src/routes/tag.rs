use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::models::response::OkResponse;
use crate::models::torrent_tag::TagId;
use crate::routes::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/tag")).service(
            web::resource("")
                .route(web::post().to(create))
                .route(web::delete().to(delete)),
        ),
    );
    cfg.service(web::scope(&format!("/{API_VERSION}/tags")).service(web::resource("").route(web::get().to(get_all))));
}

/// Get Tags
///
/// # Errors
///
/// This function will return an error if unable to get tags from database.
pub async fn get_all(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let tags = app_data.tag_repository.get_all().await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: tags }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Create {
    pub name: String,
}

/// Create Tag
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the requesting user id from the request.
/// * Get the compact user from the user id.
/// * Add the new tag to the database.
pub async fn create(req: HttpRequest, payload: web::Json<Create>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_request(&req).await?;

    app_data.tag_service.add_tag(&payload.name, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.to_string(),
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delete {
    pub tag_id: TagId,
}

/// Delete Tag
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the requesting user id from the request.
/// * Get the compact user from the user id.
/// * Delete the tag from the database.
pub async fn delete(req: HttpRequest, payload: web::Json<Delete>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_request(&req).await?;

    app_data.tag_service.delete_tag(&payload.tag_id, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: payload.tag_id }))
}
