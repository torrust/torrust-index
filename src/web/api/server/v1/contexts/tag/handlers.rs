//! API handlers for the [`tag`](crate::web::api::server::v1::contexts::tag) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::{IntoResponse, Json, Response};

use super::forms::{AddTagForm, DeleteTagForm};
use super::responses::{added_tag, deleted_tag};
use crate::common::AppData;
use crate::web::api::server::v1::extractors::optional_user_id::ExtractOptionalLoggedInUser;
use crate::web::api::server::v1::extractors::user_id::ExtractLoggedInUser;
use crate::web::api::server::v1::responses::{self};

/// It handles the request to get all the tags.
///
/// It returns:
///
/// - `200` response with a json containing the tag list [`Vec<TorrentTag>`](crate::models::torrent_tag::TorrentTag).
/// - Other error status codes if there is a database error.
///
/// Refer to the [API endpoint documentation](crate::web::api::server::v1::contexts::tag)
/// for more information about this endpoint.
///
/// # Errors
///
/// It returns an error if there is a database error.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractOptionalLoggedInUser(opt_user_id): ExtractOptionalLoggedInUser,
) -> Response {
    match app_data.tag_service.get_tags(opt_user_id).await {
        Ok(tags) => Json(responses::OkResponseData { data: tags }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// It adds a new tag.
///
/// # Errors
///
/// It returns an error if:
///
/// - The user does not have permissions to create a new tag.
/// - There is a database error.
#[allow(clippy::unused_async)]
pub async fn add_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractLoggedInUser(user_id): ExtractLoggedInUser,
    extract::Json(add_tag_form): extract::Json<AddTagForm>,
) -> Response {
    match app_data.tag_service.add_tag(&add_tag_form.name, &user_id).await {
        Ok(_) => added_tag(&add_tag_form.name).into_response(),
        Err(error) => error.into_response(),
    }
}

/// It deletes a tag.
///
/// # Errors
///
/// It returns an error if:
///
/// - The user does not have permissions to delete tags.
/// - There is a database error.
#[allow(clippy::unused_async)]
pub async fn delete_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractLoggedInUser(user_id): ExtractLoggedInUser,
    extract::Json(delete_tag_form): extract::Json<DeleteTagForm>,
) -> Response {
    match app_data.tag_service.delete_tag(&delete_tag_form.tag_id, &user_id).await {
        Ok(()) => deleted_tag(delete_tag_form.tag_id).into_response(),
        Err(error) => error.into_response(),
    }
}
