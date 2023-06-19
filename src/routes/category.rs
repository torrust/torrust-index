use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::models::response::OkResponse;
use crate::web::api::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/category")).service(
            web::resource("")
                .route(web::get().to(get))
                .route(web::post().to(add))
                .route(web::delete().to(delete)),
        ),
    );
}

/// Gets the Categories
///
/// # Errors
///
/// This function will return an error if there is a database error.
pub async fn get(app_data: WebAppData) -> ServiceResult<impl Responder> {
    let categories = app_data.category_repository.get_all().await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: categories }))
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Category {
    pub name: String,
    pub icon: Option<String>,
}

/// Adds a New Category
///
/// # Errors
///
/// This function will return an error if unable to get user.
/// This function will return an error if unable to insert into the database the new category.
pub async fn add(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let _category_id = app_data.category_service.add_category(&payload.name, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone(),
    }))
}

/// Deletes a Category
///
/// # Errors
///
/// This function will return an error if unable to get user.
/// This function will return an error if unable to delete the category from the database.
pub async fn delete(req: HttpRequest, payload: web::Json<Category>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // code-review: why do we need to send the whole category object to delete it?
    // And we should use the ID instead of the name, because the name could change
    // or we could add support for multiple languages.

    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    app_data.category_service.delete_category(&payload.name, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone(),
    }))
}
