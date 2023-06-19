//! API handlers for the [`tag`](crate::web::api::v1::contexts::tag) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::{IntoResponse, Json, Response};

use super::forms::{AddTagForm, DeleteTagForm};
use super::responses::{added_tag, deleted_tag};
use crate::common::AppData;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::{self};

/// It handles the request to get all the tags.
///
/// It returns:
///
/// - `200` response with a json containing the tag list [`Vec<TorrentTag>`](crate::models::torrent_tag::TorrentTag).
/// - Other error status codes if there is a database error.
///
/// Refer to the [API endpoint documentation](crate::web::api::v1::contexts::tag)
/// for more information about this endpoint.
///
/// # Errors
///
/// It returns an error if there is a database error.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(State(app_data): State<Arc<AppData>>) -> Response {
    match app_data.tag_repository.get_all().await {
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
    Extract(maybe_bearer_token): Extract,
    extract::Json(add_tag_form): extract::Json<AddTagForm>,
) -> Response {
    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

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
    Extract(maybe_bearer_token): Extract,
    extract::Json(delete_tag_form): extract::Json<DeleteTagForm>,
) -> Response {
    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match app_data.tag_service.delete_tag(&delete_tag_form.tag_id, &user_id).await {
        Ok(_) => deleted_tag(delete_tag_form.tag_id).into_response(),
        Err(error) => error.into_response(),
    }
}
