use actix_web::{HttpRequest, HttpResponse, Responder, web};
use serde::{Serialize, Deserialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{OkResponse};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            .service(web::resource("")
                .route(web::get().to(get_categories))
                .route(web::post().to(add_category))
                .route(web::delete().to(delete_category))
            )
    );
}

pub async fn get_categories(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let categories = app_data.database.get_categories().await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: categories
    }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub icon: Option<String>
}

pub async fn add_category(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let _ = app_data.database.insert_category(&payload.name).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone()
    }))
}

pub async fn delete_category(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // check for user
    let user = app_data.auth.get_user_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let _ = app_data.database.delete_category(&payload.name).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone()
    }))
}
