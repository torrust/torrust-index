//! API handlers for the the [`category`](crate::web::api::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::State;
use axum::response::Json;

use crate::common::AppData;
use crate::databases::database::{self, Category};
use crate::web::api::v1::responses;

/// It handles the request to get all the categories.
///
/// It returns:
///
/// - `200` response with a json containing the category list [`Vec<Category>`](crate::databases::database::Category).
/// - Other error status codes if there is a database error.
///
/// Refer to the [API endpoint documentation](crate::web::api::v1::contexts::category)
/// for more information about this endpoint.
///
/// # Errors
///
/// It returns an error if there is a database error.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(
    State(app_data): State<Arc<AppData>>,
) -> Result<Json<responses::OkResponse<Vec<Category>>>, database::Error> {
    match app_data.category_repository.get_all().await {
        Ok(categories) => Ok(Json(responses::OkResponse { data: categories })),
        Err(error) => Err(error),
    }
}
