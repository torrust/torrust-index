use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::OkResponse;
use crate::models::torrent_tag::TagId;
use crate::routes::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/tag"))
            .service(
            web::resource("")
                .route(web::post().to(add_tag))
                .route(web::delete().to(delete_tag)),
            )
    );
    cfg.service(
        web::scope(&format!("/{API_VERSION}/tags"))
            .service(
            web::resource("")
                .route(web::get().to(get_tags))
            )
    );
}

pub async fn get_tags(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let tags = app_data.torrent_tag_repository.get_tags().await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: tags }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AddTag {
    pub name: String,
}

pub async fn add_tag(req: HttpRequest, payload: web::Json<AddTag>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_request(&req).await?;

    let user = app_data.user_repository.get_compact(&user_id).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let tag = app_data.torrent_tag_repository.add_tag(&payload.name).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: tag,
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeleteTag {
    pub tag_id: TagId,
}

pub async fn delete_tag(
    req: HttpRequest,
    payload: web::Json<DeleteTag>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_request(&req).await?;

    let user = app_data.user_repository.get_compact(&user_id).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let tag = app_data.torrent_tag_repository.delete_tag(&payload.tag_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: tag,
    }))
}
