use actix_web::{web, HttpRequest, HttpResponse, Responder};
use serde::{Deserialize, Serialize};

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::OkResponse;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category").service(
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
    let categories = app_data.database.get_categories().await?;

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
    // check for user
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let _ = app_data.database.insert_category_and_get_id(&payload.name).await?;

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

    // check for user
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    app_data.database.delete_category(&payload.name).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: payload.name.clone(),
    }))
}
