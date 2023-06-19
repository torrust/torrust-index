//! API handlers for the [`torrent`](crate::web::api::v1::contexts::torrent) API
//! context.
use std::io::{Cursor, Write};
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{self, Multipart, Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use super::forms::UpdateTorrentInfoForm;
use super::responses::{new_torrent_response, torrent_file_response};
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::models::info_hash::InfoHash;
use crate::models::torrent::{Create, TorrentRequest};
use crate::models::torrent_tag::TagId;
use crate::services::torrent::ListingRequest;
use crate::utils::parse_torrent;
use crate::web::api::v1::auth::get_optional_logged_in_user;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::OkResponseData;

/// Upload a new torrent file to the Index
///
/// # Errors
///
/// This function will return an error if
///
/// - The user does not have permission to upload the torrent file.
/// - The submitted torrent file is not a valid torrent file.
#[allow(clippy::unused_async)]
pub async fn upload_torrent_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    multipart: Multipart,
) -> Response {
    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    let torrent_request = match get_torrent_request_from_payload(multipart).await {
        Ok(torrent_request) => torrent_request,
        Err(error) => return error.into_response(),
    };

    let info_hash = torrent_request.torrent.info_hash().clone();

    match app_data.torrent_service.add_torrent(torrent_request, user_id).await {
        Ok(torrent_id) => new_torrent_response(torrent_id, &info_hash).into_response(),
        Err(error) => error.into_response(),
    }
}

#[derive(Deserialize)]
pub struct InfoHashParam(pub String);

/// Returns the torrent as a byte stream `application/x-bittorrent`.
///
/// # Errors
///
/// Returns `ServiceError::BadRequest` if the torrent info-hash is invalid.
#[allow(clippy::unused_async)]
pub async fn download_torrent_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    Path(info_hash): Path<InfoHashParam>,
) -> Response {
    let Ok(info_hash) = InfoHash::from_str(&info_hash.0) else { return ServiceError::BadRequest.into_response() };

    let opt_user_id = match get_optional_logged_in_user(maybe_bearer_token, app_data.clone()).await {
        Ok(opt_user_id) => opt_user_id,
        Err(error) => return error.into_response(),
    };

    let torrent = match app_data.torrent_service.get_torrent(&info_hash, opt_user_id).await {
        Ok(torrent) => torrent,
        Err(error) => return error.into_response(),
    };

    let Ok(bytes) = parse_torrent::encode_torrent(&torrent) else { return ServiceError::InternalServerError.into_response() };

    torrent_file_response(bytes)
}

/// It returns a list of torrents matching the search criteria.
///
/// Eg: `/torrents?categories=music,other,movie&search=bunny&sort=size_DESC`
///
/// # Errors
///
/// It returns an error if the database query fails.
#[allow(clippy::unused_async)]
pub async fn get_torrents_handler(State(app_data): State<Arc<AppData>>, Query(criteria): Query<ListingRequest>) -> Response {
    match app_data.torrent_service.generate_torrent_info_listing(&criteria).await {
        Ok(torrents_response) => Json(OkResponseData { data: torrents_response }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get Torrent from the Index
///
/// # Errors
///
/// This function returns an error if:
///
/// - The info-hash is not valid.
/// - Ot there was a problem getting the torrent info from the database.
#[allow(clippy::unused_async)]
pub async fn get_torrent_info_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    Path(info_hash): Path<InfoHashParam>,
) -> Response {
    let Ok(info_hash) = InfoHash::from_str(&info_hash.0) else { return ServiceError::BadRequest.into_response() };

    let opt_user_id = match get_optional_logged_in_user(maybe_bearer_token, app_data.clone()).await {
        Ok(opt_user_id) => opt_user_id,
        Err(error) => return error.into_response(),
    };

    match app_data.torrent_service.get_torrent_info(&info_hash, opt_user_id).await {
        Ok(torrent_response) => Json(OkResponseData { data: torrent_response }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Update a the torrent info
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the user id from the request.
/// * Get the torrent info-hash from the request.
/// * Update the torrent info.
#[allow(clippy::unused_async)]
pub async fn update_torrent_info_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    Path(info_hash): Path<InfoHashParam>,
    extract::Json(update_torrent_info_form): extract::Json<UpdateTorrentInfoForm>,
) -> Response {
    let Ok(info_hash) = InfoHash::from_str(&info_hash.0) else { return ServiceError::BadRequest.into_response() };

    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match app_data
        .torrent_service
        .update_torrent_info(
            &info_hash,
            &update_torrent_info_form.title,
            &update_torrent_info_form.description,
            &update_torrent_info_form.tags,
            &user_id,
        )
        .await
    {
        Ok(torrent_response) => Json(OkResponseData { data: torrent_response }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Delete a torrent.
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the user ID from the request.
/// * Get the torrent info-hash from the request.
/// * Delete the torrent.
#[allow(clippy::unused_async)]
pub async fn delete_torrent_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    Path(info_hash): Path<InfoHashParam>,
) -> Response {
    let Ok(info_hash) = InfoHash::from_str(&info_hash.0) else { return ServiceError::BadRequest.into_response() };

    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    match app_data.torrent_service.delete_torrent(&info_hash, &user_id).await {
        Ok(deleted_torrent_response) => Json(OkResponseData {
            data: deleted_torrent_response,
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

/// Extracts the [`TorrentRequest`] from the multipart form payload.
///
/// # Errors
///
/// It will return an error if:
///
/// - The text fields do not contain a valid UTF8 string.
/// - The torrent file data is not valid because:
///    - The content type is not `application/x-bittorrent`.
///    - The multipart content is invalid.
///    - The torrent file pieces key has a length that is not a multiple of 20.
///    - The binary data cannot be decoded as a torrent file.
async fn get_torrent_request_from_payload(mut payload: Multipart) -> Result<TorrentRequest, ServiceError> {
    let torrent_buffer = vec![0u8];
    let mut torrent_cursor = Cursor::new(torrent_buffer);

    let mut title = String::new();
    let mut description = String::new();
    let mut category = String::new();
    let mut tags: Vec<TagId> = vec![];

    while let Some(mut field) = payload.next_field().await.unwrap() {
        let name = field.name().unwrap().clone();

        match name {
            "title" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                title = String::from_utf8(data.to_vec()).map_err(|_| ServiceError::BadRequest)?;
            }
            "description" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                description = String::from_utf8(data.to_vec()).map_err(|_| ServiceError::BadRequest)?;
            }
            "category" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                category = String::from_utf8(data.to_vec()).map_err(|_| ServiceError::BadRequest)?;
            }
            "tags" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                let string_data = String::from_utf8(data.to_vec()).map_err(|_| ServiceError::BadRequest)?;
                tags = serde_json::from_str(&string_data).map_err(|_| ServiceError::BadRequest)?;
            }
            "torrent" => {
                let content_type = field.content_type().unwrap().clone();

                if content_type != "application/x-bittorrent" {
                    return Err(ServiceError::InvalidFileType);
                }

                while let Some(chunk) = field.chunk().await.map_err(|_| (ServiceError::BadRequest))? {
                    torrent_cursor.write_all(&chunk)?;
                }
            }
            _ => {}
        }
    }

    let fields = Create {
        title,
        description,
        category,
        tags,
    };

    fields.verify()?;

    let position = usize::try_from(torrent_cursor.position()).map_err(|_| ServiceError::InvalidTorrentFile)?;
    let inner = torrent_cursor.get_ref();

    let torrent = parse_torrent::decode_torrent(&inner[..position]).map_err(|_| ServiceError::InvalidTorrentFile)?;

    // Make sure that the pieces key has a length that is a multiple of 20
    // code-review: I think we could put this inside the service.
    if let Some(pieces) = torrent.info.pieces.as_ref() {
        if pieces.as_ref().len() % 20 != 0 {
            return Err(ServiceError::InvalidTorrentPiecesLength);
        }
    }

    Ok(TorrentRequest { fields, torrent })
}
