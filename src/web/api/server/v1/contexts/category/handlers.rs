//! API handlers for the the [`category`](crate::web::api::server::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::{IntoResponse, Json, Response};

use super::forms::{AddCategoryForm, DeleteCategoryForm};
use super::responses::{added_category, deleted_category, Category};
use crate::common::AppData;
use crate::web::api::server::v1::extractors::user_id::ExtractLoggedInUser;
use crate::web::api::server::v1::responses::{self};

/// It handles the request to get all the categories.
///
/// It returns:
///
/// - `200` response with a json containing the category list [`Vec<Category>`](crate::databases::database::Category).
/// - Other error status codes if there is a database error.
///
/// Refer to the [API endpoint documentation](crate::web::api::server::v1::contexts::category)
/// for more information about this endpoint.
///
/// # Errors
///
/// It returns an error if there is a database error.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(State(app_data): State<Arc<AppData>>) -> Response {
    match app_data.category_repository.get_all().await {
        Ok(categories) => {
            let categories: Vec<Category> = categories.into_iter().map(Category::from).collect();
            Json(responses::OkResponseData { data: categories }).into_response()
        }
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
    ExtractLoggedInUser(user_id): ExtractLoggedInUser,
    extract::Json(category_form): extract::Json<AddCategoryForm>,
) -> Response {
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
    ExtractLoggedInUser(user_id): ExtractLoggedInUser,
    extract::Json(category_form): extract::Json<DeleteCategoryForm>,
) -> Response {
    // code-review: why do we need to send the whole category object to delete it?
    // And we should use the ID instead of the name, because the name could change
    // or we could add support for multiple languages.

    match app_data.category_service.delete_category(&category_form.name, &user_id).await {
        Ok(()) => deleted_category(&category_form.name).into_response(),
        Err(error) => error.into_response(),
    }
}
