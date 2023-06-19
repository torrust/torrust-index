use std::io::{Cursor, Write};
use std::str::FromStr;

use actix_multipart::Multipart;
use actix_web::web::Query;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sqlx::FromRow;

use crate::common::WebAppData;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::info_hash::InfoHash;
use crate::models::response::{NewTorrentResponse, OkResponse};
use crate::models::torrent::TorrentRequest;
use crate::models::torrent_tag::TagId;
use crate::services::torrent::ListingRequest;
use crate::utils::parse_torrent;
use crate::web::api::API_VERSION;

pub fn init(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope(&format!("/{API_VERSION}/torrent"))
            .service(web::resource("/upload").route(web::post().to(upload_torrent_handler)))
            .service(web::resource("/download/{info_hash}").route(web::get().to(download_torrent_handler)))
            .service(
                web::resource("/{info_hash}")
                    .route(web::get().to(get_torrent_info_handler))
                    .route(web::put().to(update_torrent_info_handler))
                    .route(web::delete().to(delete_torrent_handler)),
            ),
    );
    cfg.service(
        web::scope(&format!("/{API_VERSION}/torrents")).service(web::resource("").route(web::get().to(get_torrents_handler))),
    );
}

#[derive(FromRow)]
pub struct Count {
    pub count: i32,
}

#[derive(Debug, Deserialize)]
pub struct Create {
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<TagId>,
}

impl Create {
    /// Returns the verify of this [`Create`].
    ///
    /// # Errors
    ///
    /// This function will return an `BadRequest` error if the `title` or the `category` is empty.
    pub fn verify(&self) -> Result<(), ServiceError> {
        if self.title.is_empty() || self.category.is_empty() {
            Err(ServiceError::BadRequest)
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, Deserialize)]
pub struct Update {
    title: Option<String>,
    description: Option<String>,
    tags: Option<Vec<TagId>>,
}

/// Upload a Torrent to the Index
///
/// # Errors
///
/// This function will return an error if there was a problem uploading the
/// torrent.
pub async fn upload_torrent_handler(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let torrent_request = get_torrent_request_from_payload(payload).await?;

    let info_hash = torrent_request.torrent.info_hash().clone();

    let torrent_service = app_data.torrent_service.clone();

    let torrent_id = torrent_service.add_torrent(torrent_request, user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse { torrent_id, info_hash },
    }))
}

/// Returns the torrent as a byte stream `application/x-bittorrent`.
///
/// # Errors
///
/// Returns `ServiceError::BadRequest` if the torrent info-hash is invalid.
pub async fn download_torrent_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let info_hash = get_torrent_info_hash_from_request(&req)?;
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await.ok();

    let torrent = app_data.torrent_service.get_torrent(&info_hash, user_id).await?;

    let buffer = parse_torrent::encode_torrent(&torrent).map_err(|_| ServiceError::InternalServerError)?;

    Ok(HttpResponse::Ok().content_type("application/x-bittorrent").body(buffer))
}

/// Get Torrent from the Index
///
/// # Errors
///
/// This function will return an error if unable to get torrent info.
pub async fn get_torrent_info_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let info_hash = get_torrent_info_hash_from_request(&req)?;
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await.ok();

    let torrent_response = app_data.torrent_service.get_torrent_info(&info_hash, user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: torrent_response }))
}

/// Update a Torrent in the Index
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the user id from the request.
/// * Get the torrent info-hash from the request.
/// * Update the torrent info.
pub async fn update_torrent_info_handler(
    req: HttpRequest,
    payload: web::Json<Update>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    let info_hash = get_torrent_info_hash_from_request(&req)?;
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let torrent_response = app_data
        .torrent_service
        .update_torrent_info(&info_hash, &payload.title, &payload.description, &payload.tags, &user_id)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: torrent_response }))
}

/// Delete a Torrent from the Index
///
/// # Errors
///
/// This function will return an error if unable to:
///
/// * Get the user id from the request.
/// * Get the torrent info-hash from the request.
/// * Delete the torrent.
pub async fn delete_torrent_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let info_hash = get_torrent_info_hash_from_request(&req)?;
    let user_id = app_data.auth.get_user_id_from_actix_web_request(&req).await?;

    let deleted_torrent_response = app_data.torrent_service.delete_torrent(&info_hash, &user_id).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: deleted_torrent_response,
    }))
}

/// It returns a list of torrents matching the search criteria.
/// Eg: `/torrents?categories=music,other,movie&search=bunny&sort=size_DESC`
///
/// # Errors
///
/// Returns a `ServiceError::DatabaseError` if the database query fails.
pub async fn get_torrents_handler(criteria: Query<ListingRequest>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrents_response = app_data.torrent_service.generate_torrent_info_listing(&criteria).await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: torrents_response }))
}

fn get_torrent_info_hash_from_request(req: &HttpRequest) -> Result<InfoHash, ServiceError> {
    match req.match_info().get("info_hash") {
        None => Err(ServiceError::BadRequest),
        Some(info_hash) => match InfoHash::from_str(info_hash) {
            Err(_) => Err(ServiceError::BadRequest),
            Ok(v) => Ok(v),
        },
    }
}

async fn get_torrent_request_from_payload(mut payload: Multipart) -> Result<TorrentRequest, ServiceError> {
    let torrent_buffer = vec![0u8];
    let mut torrent_cursor = Cursor::new(torrent_buffer);

    let mut title = String::new();
    let mut description = String::new();
    let mut category = String::new();
    let mut tags: Vec<TagId> = vec![];

    while let Ok(Some(mut field)) = payload.try_next().await {
        match field.content_disposition().get_name().unwrap() {
            "title" | "description" | "category" | "tags" => {
                let data = field.next().await;

                if data.is_none() {
                    continue;
                }

                let wrapped_data = &data.unwrap().map_err(|_| ServiceError::BadRequest)?;
                let parsed_data = std::str::from_utf8(wrapped_data).map_err(|_| ServiceError::BadRequest)?;

                match field.content_disposition().get_name().unwrap() {
                    "title" => title = parsed_data.to_string(),
                    "description" => description = parsed_data.to_string(),
                    "category" => category = parsed_data.to_string(),
                    "tags" => tags = serde_json::from_str(parsed_data).map_err(|_| ServiceError::BadRequest)?,
                    _ => {}
                }
            }
            "torrent" => {
                if *field.content_type().unwrap() != "application/x-bittorrent" {
                    return Err(ServiceError::InvalidFileType);
                }

                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    torrent_cursor.write_all(&data)?;
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

    // make sure that the pieces key has a length that is a multiple of 20
    if let Some(pieces) = torrent.info.pieces.as_ref() {
        if pieces.as_ref().len() % 20 != 0 {
            return Err(ServiceError::InvalidTorrentPiecesLength);
        }
    }

    Ok(TorrentRequest { fields, torrent })
}
