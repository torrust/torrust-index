//! API handlers for the [`torrent`](crate::web::api::v1::contexts::torrent) API
//! context.
use std::io::{Cursor, Write};
use std::str::FromStr;
use std::sync::Arc;

use axum::extract::{self, Multipart, Path, Query, State};
use axum::response::{IntoResponse, Redirect, Response};
use axum::Json;
use log::debug;
use serde::Deserialize;
use uuid::Uuid;

use super::errors;
use super::forms::UpdateTorrentInfoForm;
use super::responses::{new_torrent_response, torrent_file_response};
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::models::info_hash::InfoHash;
use crate::models::torrent_tag::TagId;
use crate::services::torrent::{AddTorrentRequest, ListingRequest};
use crate::services::torrent_file::generate_random_torrent;
use crate::utils::parse_torrent;
use crate::web::api::v1::auth::get_optional_logged_in_user;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::extractors::user_id::ExtractLoggedInUser;
use crate::web::api::v1::responses::OkResponseData;
use crate::web::api::v1::routes::API_VERSION_URL_PREFIX;

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
    ExtractLoggedInUser(user_id): ExtractLoggedInUser,
    multipart: Multipart,
) -> Response {
    let add_torrent_form = match build_add_torrent_request_from_payload(multipart).await {
        Ok(torrent_request) => torrent_request,
        Err(error) => return error.into_response(),
    };

    match app_data.torrent_service.add_torrent(add_torrent_form, user_id).await {
        Ok(response) => new_torrent_response(&response).into_response(),
        Err(error) => error.into_response(),
    }
}

#[derive(Deserialize)]
pub struct InfoHashParam(pub String);

impl InfoHashParam {
    fn lowercase(&self) -> String {
        self.0.to_lowercase()
    }
}

/// Returns the torrent as a byte stream `application/x-bittorrent`.
///
/// # Errors
///
/// Returns an error if the torrent info-hash is invalid.
#[allow(clippy::unused_async)]
pub async fn download_torrent_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    Path(info_hash): Path<InfoHashParam>,
) -> Response {
    let Ok(info_hash) = InfoHash::from_str(&info_hash.lowercase()) else {
        return errors::Request::InvalidInfoHashParam.into_response();
    };

    debug!("Downloading torrent: {:?}", info_hash.to_hex_string());

    if let Some(redirect_response) = redirect_to_download_url_using_canonical_info_hash_if_needed(&app_data, &info_hash).await {
        debug!("Redirecting to URL with canonical info-hash");
        redirect_response
    } else {
        let opt_user_id = match get_optional_logged_in_user(maybe_bearer_token, app_data.clone()).await {
            Ok(opt_user_id) => opt_user_id,
            Err(error) => return error.into_response(),
        };

        let torrent = match app_data.torrent_service.get_torrent(&info_hash, opt_user_id).await {
            Ok(torrent) => torrent,
            Err(error) => return error.into_response(),
        };

        let Ok(bytes) = parse_torrent::encode_torrent(&torrent) else {
            return ServiceError::InternalServerError.into_response();
        };

        torrent_file_response(
            bytes,
            &format!("{}.torrent", torrent.info.name),
            &torrent.canonical_info_hash_hex(),
        )
    }
}

async fn redirect_to_download_url_using_canonical_info_hash_if_needed(
    app_data: &Arc<AppData>,
    info_hash: &InfoHash,
) -> Option<Response> {
    match app_data
        .torrent_info_hash_repository
        .find_canonical_info_hash_for(info_hash)
        .await
    {
        Ok(Some(canonical_info_hash)) => {
            if canonical_info_hash != *info_hash {
                return Some(
                    Redirect::temporary(&format!(
                        "/{API_VERSION_URL_PREFIX}/torrent/download/{}",
                        canonical_info_hash.to_hex_string()
                    ))
                    .into_response(),
                );
            }
            None
        }
        Ok(None) => None,
        Err(error) => Some(error.into_response()),
    }
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
    let Ok(info_hash) = InfoHash::from_str(&info_hash.lowercase()) else {
        return errors::Request::InvalidInfoHashParam.into_response();
    };

    if let Some(redirect_response) = redirect_to_details_url_using_canonical_info_hash_if_needed(&app_data, &info_hash).await {
        redirect_response
    } else {
        let opt_user_id = match get_optional_logged_in_user(maybe_bearer_token, app_data.clone()).await {
            Ok(opt_user_id) => opt_user_id,
            Err(error) => return error.into_response(),
        };

        match app_data.torrent_service.get_torrent_info(&info_hash, opt_user_id).await {
            Ok(torrent_response) => Json(OkResponseData { data: torrent_response }).into_response(),
            Err(error) => error.into_response(),
        }
    }
}

async fn redirect_to_details_url_using_canonical_info_hash_if_needed(
    app_data: &Arc<AppData>,
    info_hash: &InfoHash,
) -> Option<Response> {
    match app_data
        .torrent_info_hash_repository
        .find_canonical_info_hash_for(info_hash)
        .await
    {
        Ok(Some(canonical_info_hash)) => {
            if canonical_info_hash != *info_hash {
                return Some(
                    Redirect::temporary(&format!(
                        "/{API_VERSION_URL_PREFIX}/torrent/{}",
                        canonical_info_hash.to_hex_string()
                    ))
                    .into_response(),
                );
            }
            None
        }
        Ok(None) => None,
        Err(error) => Some(error.into_response()),
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
    let Ok(info_hash) = InfoHash::from_str(&info_hash.lowercase()) else {
        return errors::Request::InvalidInfoHashParam.into_response();
    };

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
            &update_torrent_info_form.category,
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
    let Ok(info_hash) = InfoHash::from_str(&info_hash.lowercase()) else {
        return errors::Request::InvalidInfoHashParam.into_response();
    };

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

#[derive(Debug, Deserialize)]
pub struct UuidParam(pub String);

impl UuidParam {
    fn value(&self) -> String {
        self.0.to_lowercase()
    }
}

/// Returns a random torrent file as a byte stream `application/x-bittorrent`.
///
/// This is useful for testing purposes.
///
/// # Errors
///
/// Returns an error if the torrent info-hash is invalid.
#[allow(clippy::unused_async)]
pub async fn create_random_torrent_handler(State(_app_data): State<Arc<AppData>>, Path(uuid): Path<UuidParam>) -> Response {
    let Ok(uuid) = Uuid::parse_str(&uuid.value()) else {
        return errors::Request::InvalidInfoHashParam.into_response();
    };

    let torrent = generate_random_torrent(uuid);

    let Ok(bytes) = parse_torrent::encode_torrent(&torrent) else {
        return ServiceError::InternalServerError.into_response();
    };

    torrent_file_response(
        bytes,
        &format!("{}.torrent", torrent.info.name),
        &torrent.canonical_info_hash_hex(),
    )
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
async fn build_add_torrent_request_from_payload(mut payload: Multipart) -> Result<AddTorrentRequest, errors::Request> {
    let torrent_buffer = vec![0u8];
    let mut torrent_cursor = Cursor::new(torrent_buffer);

    let mut title = String::new();
    let mut description = String::new();
    let mut category = String::new();
    let mut tags: Vec<TagId> = vec![];

    while let Some(mut field) = payload.next_field().await.unwrap() {
        let name = field.name().unwrap();

        match name {
            "title" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                title = String::from_utf8(data.to_vec()).map_err(|_| errors::Request::TitleIsNotValidUtf8)?;
            }
            "description" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                description = String::from_utf8(data.to_vec()).map_err(|_| errors::Request::DescriptionIsNotValidUtf8)?;
            }
            "category" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                category = String::from_utf8(data.to_vec()).map_err(|_| errors::Request::CategoryIsNotValidUtf8)?;
            }
            "tags" => {
                let data = field.bytes().await.unwrap();
                if data.is_empty() {
                    continue;
                };
                let string_data = String::from_utf8(data.to_vec()).map_err(|_| errors::Request::TagsArrayIsNotValidUtf8)?;
                tags = serde_json::from_str(&string_data).map_err(|_| errors::Request::TagsArrayIsNotValidJson)?;
            }
            "torrent" => {
                let content_type = field.content_type().unwrap();

                if content_type != "application/x-bittorrent" {
                    return Err(errors::Request::InvalidFileType);
                }

                while let Some(chunk) = field
                    .chunk()
                    .await
                    .map_err(|_| (errors::Request::CannotReadChunkFromUploadedBinary))?
                {
                    torrent_cursor
                        .write_all(&chunk)
                        .map_err(|_| (errors::Request::CannotWriteChunkFromUploadedBinary))?;
                }
            }
            _ => {}
        }
    }

    Ok(AddTorrentRequest {
        title,
        description,
        category_name: category,
        tags,
        torrent_buffer: torrent_cursor.into_inner(),
    })
}
