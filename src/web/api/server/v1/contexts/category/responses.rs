//! API responses for the the [`category`](crate::web::api::server::v1::contexts::category) API
//! context.
use axum::Json;

use crate::web::api::server::v1::responses::OkResponseData;

/// Response after successfully creating a new category.
pub fn added_category(category_name: &str) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: category_name.to_string(),
    })
}

/// Response after successfully deleting a new category.
pub fn deleted_category(category_name: &str) -> Json<OkResponseData<String>> {
    Json(OkResponseData {
        data: category_name.to_string(),
    })
}
