use actix_web::{web, Responder, HttpResponse, HttpRequest};
use actix_multipart::Multipart;
use async_std::fs::create_dir_all;
use async_std::prelude::*;
use futures::{StreamExt, TryStreamExt, AsyncWriteExt};
use serde::{Deserialize, Serialize};
use crate::errors::{ServiceError, ServiceResult};
use crate::utils::parse_torrent;
use crate::common::{WebAppData, Username};
use std::env;
use crate::config::TorrustConfig;
use crate::models::user::{User, Claims};
use std::io::Cursor;
use std::io::{Write};
use crate::models::torrent_file::Torrent;
use std::error::Error;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/new")
                .route(web::post().to(create_torrent)))
            .service(web::resource("/upload")
                .route(web::post().to(upload_torrent)))
    );
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct CreateTorrent {
    pub name: String,
    pub description: String,
    pub categories: Vec<String>,
}

pub async fn create_torrent(req: HttpRequest, payload: web::Json<CreateTorrent>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = match app_data.auth.get_user_from_request(&req).await {
        Ok(user) => Ok(user),
        Err(e) => Err(e)
    }?;

    println!("{:?}", user.username);
    Ok(HttpResponse::Ok())
}

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // let torrent_id = req.match_info().get("id").unwrap();

    let mut torrent = get_torrent_from_payload(payload).await?;
    torrent.set_torrust_config(&app_data.cfg);

    println!("{:?}", torrent);
    println!("{:?}", torrent.info_hash());

    Ok(HttpResponse::Ok())
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
