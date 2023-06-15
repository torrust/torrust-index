//! API handlers for the the [`category`](crate::web::api::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::Json;

use super::forms::CategoryForm;
use super::responses::added_category;
use crate::common::AppData;
use crate::databases::database::{self, Category};
use crate::errors::ServiceError;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::{self, OkResponse};

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

/// It adds a new category.
///
/// # Errors
///
/// It returns an error if:
///
/// - The user does not have permissions to create a new category.
/// - There is a database error.
#[allow(clippy::unused_async)]
pub async fn add_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    extract::Json(category_form): extract::Json<CategoryForm>,
) -> Result<Json<OkResponse<String>>, ServiceError> {
    let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await?;

    match app_data.category_service.add_category(&category_form.name, &user_id).await {
        Ok(_) => Ok(added_category(&category_form.name)),
        Err(error) => Err(error),
    }
}
