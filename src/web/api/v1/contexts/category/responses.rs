//! API responses for the the [`category`](crate::web::api::v1::contexts::category) API
//! context.
use axum::Json;

use crate::web::api::v1::responses::OkResponse;

/// Response after successfully creating a new category.
pub fn added_category(category_name: &str) -> Json<OkResponse<String>> {
    Json(OkResponse {
        data: category_name.to_string(),
    })
}
