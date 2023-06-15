//! API handlers for the [`torrent`](crate::web::api::v1::contexts::torrent) API
//! context.
use std::io::{Cursor, Write};
use std::sync::Arc;

use axum::extract::{Multipart, State};
use axum::response::{IntoResponse, Response};

use super::responses::new_torrent_response;
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::models::torrent::TorrentRequest;
use crate::models::torrent_tag::TagId;
use crate::routes::torrent::Create;
use crate::utils::parse_torrent;
use crate::web::api::v1::extractors::bearer_token::Extract;

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
        Err(err) => return err.into_response(),
    };

    let torrent_request = match get_torrent_request_from_payload(multipart).await {
        Ok(torrent_request) => torrent_request,
        Err(err) => return err.into_response(),
    };

    let info_hash = torrent_request.torrent.info_hash().clone();

    match app_data.torrent_service.add_torrent(torrent_request, user_id).await {
        Ok(torrent_id) => new_torrent_response(torrent_id, &info_hash).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Extracts the [`TorrentRequest`] from the multipart form payload.
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
