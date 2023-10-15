//! API handlers for the the [`category`](crate::web::api::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::{IntoResponse, Json, Response};

use super::forms::{AddCategoryForm, DeleteCategoryForm};
use super::responses::{added_category, deleted_category};
use crate::common::AppData;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::{self};

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
pub async fn get_all_handler(State(app_data): State<Arc<AppData>>) -> Response {
    match app_data.category_repository.get_all().await {
        Ok(categories) => Json(responses::OkResponseData { data: categories }).into_response(),
        Err(error) => error.into_response(),
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
    extract::Json(category_form): extract::Json<AddCategoryForm>,
) -> Response {
    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match app_data.category_service.add_category(&category_form.name, &user_id).await {
        Ok(_) => added_category(&category_form.name).into_response(),
        Err(error) => error.into_response(),
    }
}

/// It deletes a category.
///
/// # Errors
///
/// It returns an error if:
///
/// - The user does not have permissions to delete category.
/// - There is a database error.
#[allow(clippy::unused_async)]
pub async fn delete_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    extract::Json(category_form): extract::Json<DeleteCategoryForm>,
) -> Response {
    // code-review: why do we need to send the whole category object to delete it?
    // And we should use the ID instead of the name, because the name could change
    // or we could add support for multiple languages.

    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match app_data.category_service.delete_category(&category_form.name, &user_id).await {
        Ok(()) => deleted_category(&category_form.name).into_response(),
        Err(error) => error.into_response(),
    }
}
