use std::env;

use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::Query;
use async_std::fs::create_dir_all;
use async_std::prelude::*;
use futures::{AsyncWriteExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};

use crate::common::{Username, WebAppData};
use crate::config::TorrustConfig;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{NewTorrentResponse, OkResponse};
use crate::models::user::{Claims, User};
use crate::models::torrent_listing::TorrentListing;
use crate::utils::parse_torrent;
use std::io::Cursor;
use std::io::{Write};
use crate::models::torrent_file::Torrent;
use std::error::Error;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/new")
                .route(web::post().to(create_torrent)))
            .service(web::resource("/upload/{id}")
                .route(web::post().to(upload_torrent)))
            .service(web::resource("/download/{id}")
                .route(web::get().to(download_torrent)))
            .service(web::resource("/{id}")
                .route(web::get().to(get_torrent)))
    );
}

#[derive(Debug, Deserialize)]
pub struct CreateTorrent {
    pub title: String,
    pub description: String,
    pub category: String,
}

pub async fn get_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = req.match_info().get("id").unwrap();

    let res = sqlx::query_as!(
        TorrentListing,
        r#"SELECT * FROM torrust_torrents
           WHERE torrent_id = ?"#,
        torrent_id
    )
        .fetch_all(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: res
    }))
}

pub async fn create_torrent(req: HttpRequest, payload: web::Json<CreateTorrent>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_from_request(&req).await?;

    let res = sqlx::query!(
        "SELECT category_id FROM torrust_categories WHERE name = ?",
        payload.category
    )
        .fetch_one(&app_data.database.pool)
        .await;

    let row = match res {
        Ok(row) => row,
        Err(e) => return Err(ServiceError::InvalidCategory),
    };

    let res = sqlx::query!(
        r#"INSERT INTO torrust_torrents (uploader_id, title, description, category_id)
        VALUES ($1, $2, $3, $4)
        RETURNING torrent_id as "torrent_id: i64""#,
        user.user_id,
        payload.title,
        payload.description,
        row.category_id
    )
        .fetch_one(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse {
            torrent_id: res.torrent_id
        }
    }))
}

pub async fn download_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = get_torrent_id_from_request(&req)?;

    let user = app_data.auth.get_user_from_request(&req).await;

    let filepath = format!("{}/{}", app_data.cfg.storage.upload_path, torrent_id.to_string() + ".torrent");

    let mut torrent = match parse_torrent::read_torrent_from_file(&filepath) {
        Ok(torrent) => Ok(torrent),
        Err(e) => {
            println!("{:?}", e);
            Err(ServiceError::InternalServerError)
        }
    }?;

    if user.is_ok() {
        let personal_announce_url = app_data.auth.get_personal_announce_url(&user.unwrap()).await;
        // this would mean the connection with the tracker is not ok
        if personal_announce_url.is_none() { return Err(ServiceError::InternalServerError) }
        torrent.announce = Some(personal_announce_url.unwrap());
    } else {
        torrent.announce = Some(app_data.cfg.tracker.url.clone());
    }

    let buffer = match parse_torrent::encode_torrent(&torrent) {
        Ok(v) => Ok(v),
        Err(e) => {
            println!("{:?}", e);
            Err(ServiceError::InternalServerError)
        }
    }?;

    Ok(HttpResponse::Ok()
        .content_type("application/x-bittorrent")
        .body(buffer)
    )
}

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_from_request(&req).await?;

    let torrent_id = get_torrent_id_from_request(&req)?;

    let torrent_listing = match app_data.database.get_torrent_by_id(torrent_id).await {
        None => Err(ServiceError::TorrentNotFound),
        Some(v) => Ok(v)
    }?;

    verify_torrent_ownership(&user, &torrent_listing).await?;

    let mut torrent = get_torrent_from_payload(payload).await?;
    torrent.set_torrust_config(&app_data.cfg);

    let filepath = format!("{}/{}", app_data.cfg.storage.upload_path, torrent_id.to_string() + ".torrent");

    save_torrent_file(&filepath, &torrent).await?;

    let _res = app_data.database.update_torrent_info_hash(torrent_id, torrent.info_hash()).await?;

    Ok(HttpResponse::Ok())
}

async fn verify_torrent_ownership(user: &User, torrent_listing: &TorrentListing) -> Result<(), ServiceError> {
    match torrent_listing.uploader_id == user.user_id {
        true => Ok(()),
        false => Err(ServiceError::BadRequest)
    }
}

async fn save_torrent_file(filepath: &str, torrent: &Torrent) -> Result<(), ServiceError> {
    let torrent_bytes = match parse_torrent::encode_torrent(torrent) {
        Ok(v) => Ok(v),
        Err(_) => Err(ServiceError::InternalServerError)
    }?;

    let mut f = match async_std::fs::File::create(&filepath).await {
        Ok(v) => Ok(v),
        Err(_) => Err(ServiceError::InternalServerError)
    }?;

    match AsyncWriteExt::write_all(&mut f, &torrent_bytes.as_slice()).await {
        Ok(v) => Ok(v),
        Err(_) => Err(ServiceError::InternalServerError)
    }?;

    Ok(())
}

fn get_torrent_id_from_request(req: &HttpRequest) -> Result<i64, ServiceError> {
    match req.match_info().get("id") {
        None => Err(ServiceError::BadRequest),
        Some(torrent_id) => {
            match torrent_id.parse() {
                Err(_) => Err(ServiceError::BadRequest),
                Ok(v) => Ok(v)
            }
        }
    }
}

async fn get_torrent_from_payload(mut payload: Multipart) -> Result<Torrent, ServiceError> {
    let buffer = vec![0u8];
    let mut cursor = Cursor::new(buffer);

    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.content_type().to_string() != "application/x-bittorrent" {
            return Err(ServiceError::InvalidFileType)
        }

        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            cursor.write_all(&data)?;
        }
    }

    let position = cursor.position() as usize;
    let inner = cursor.get_ref();

    match parse_torrent::decode_torrent(&inner[..position]) {
        Ok(torrent) => Ok(torrent),
        Err(_) => Err(ServiceError::InvalidTorrentFile)
    }
}
