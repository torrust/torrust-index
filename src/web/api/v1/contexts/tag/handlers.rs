//! API handlers for the [`tag`](crate::web::api::v1::contexts::tag) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::Json;

use super::forms::{AddTagForm, DeleteTagForm};
use super::responses::{added_tag, deleted_tag};
use crate::common::AppData;
use crate::databases::database;
use crate::errors::ServiceError;
use crate::models::torrent_tag::TorrentTag;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::{self, OkResponse};

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
pub async fn get_all_handler(
    State(app_data): State<Arc<AppData>>,
) -> Result<Json<responses::OkResponse<Vec<TorrentTag>>>, database::Error> {
    match app_data.tag_repository.get_all().await {
        Ok(tags) => Ok(Json(responses::OkResponse { data: tags })),
        Err(error) => Err(error),
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
) -> Result<Json<OkResponse<String>>, ServiceError> {
    let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await?;

    match app_data.tag_service.add_tag(&add_tag_form.name, &user_id).await {
        Ok(_) => Ok(added_tag(&add_tag_form.name)),
        Err(error) => Err(error),
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
) -> Result<Json<OkResponse<String>>, ServiceError> {
    let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await?;

    match app_data.tag_service.delete_tag(&delete_tag_form.tag_id, &user_id).await {
        Ok(_) => Ok(deleted_tag(delete_tag_form.tag_id)),
        Err(error) => Err(error),
    }
}
