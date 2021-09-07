use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Query, Form, Json};
use async_std::fs::create_dir_all;
use async_std::prelude::*;
use futures::{AsyncWriteExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{NewTorrentResponse, OkResponse};
use crate::models::torrent::{TorrentListing, TorrentRequest, TorrentResponse};
use crate::utils::parse_torrent;
use crate::common::{WebAppData, Username};
use crate::models::user::{User, Claims};
use std::io::Cursor;
use std::io::{Write};
use crate::models::torrent_file::Torrent;
use std::error::Error;
use crate::utils::time::current_time;
use std::collections::HashMap;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/upload")
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

impl CreateTorrent {
    pub fn verify(&self) -> Result<(), ServiceError>{
        if !self.title.is_empty() && !self.category.is_empty() {
            return Ok(())
        }

        Err(ServiceError::BadRequest)
    }
}

pub async fn get_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = req.match_info().get("id").unwrap();

    let mut res: TorrentResponse = sqlx::query_as!(
        TorrentResponse,
        r#"SELECT * FROM torrust_torrents
           WHERE torrent_id = ?"#,
        torrent_id
    )
        .fetch_one(&app_data.database.pool)
        .await?;

    // get realtime seeders and leechers
    // todo: config option to disable realtime tracker info
    if let Ok(torrent_info) = app_data.tracker.get_torrent_info(&res.info_hash).await {
        res.seeders = torrent_info.seeders;
        res.leechers = torrent_info.leechers;
    }

    Ok(HttpResponse::Ok().json(OkResponse {
        data: res
    }))
}

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_from_request(&req).await?;

    let mut torrent_request = get_torrent_request_from_payload(payload).await?;

    // println!("{:?}", torrent_request.torrent);

    // update announce url to our own tracker url
    torrent_request.torrent.set_torrust_config(&app_data.cfg);

    let res = sqlx::query!(
        "SELECT category_id FROM torrust_categories WHERE name = ?",
        torrent_request.fields.category
    )
        .fetch_one(&app_data.database.pool)
        .await;

    let row = match res {
        Ok(row) => row,
        Err(e) => return Err(ServiceError::InvalidCategory),
    };

    let username = user.username;
    let info_hash = torrent_request.torrent.info_hash();
    let title = torrent_request.fields.title;
    let category = torrent_request.fields.category;
    let description = torrent_request.fields.description;
    let current_time = current_time() as i64;
    let file_size = torrent_request.torrent.file_size();
    let mut seeders = 0;
    let mut leechers = 0;

    if let Ok(torrent_info) = app_data.tracker.get_torrent_info(&info_hash).await {
        seeders = torrent_info.seeders;
        leechers = torrent_info.leechers;
    }

    // println!("{:?}", (&username, &info_hash, &title, &category, &description, &current_time, &file_size));

    let torrent_id = app_data.database.insert_torrent_and_get_id(username, info_hash, title, row.category_id, description, file_size, seeders, leechers).await?;

    // whitelist info hash on tracker
    let _r = app_data.tracker.whitelist_info_hash(torrent_request.torrent.info_hash()).await;

    let filepath = format!("{}/{}", app_data.cfg.storage.upload_path, torrent_id.to_string() + ".torrent");
    save_torrent_file(&filepath, &torrent_request.torrent).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse {
            torrent_id
        }
    }))
}

pub async fn download_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = get_torrent_id_from_request(&req)?;

    // optional
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
        let unwrapped_user = user.unwrap();
        let personal_announce_url = app_data.tracker.get_personal_announce_url(&unwrapped_user).await?;
        torrent.announce = Some(personal_announce_url.clone());
        if let Some(list) = &mut torrent.announce_list {
            let mut vec = Vec::new();
            vec.push(personal_announce_url);
            list.insert(0, vec);
        }
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

async fn verify_torrent_ownership(user: &User, torrent_listing: &TorrentListing) -> Result<(), ServiceError> {
    match torrent_listing.uploader == user.username {
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

async fn get_torrent_request_from_payload(mut payload: Multipart) -> Result<TorrentRequest, ServiceError> {
    let torrent_buffer = vec![0u8];
    let mut torrent_cursor = Cursor::new(torrent_buffer);

    let mut title = "".to_string();
    let mut description = "".to_string();
    let mut category = "".to_string();

    while let Ok(Some(mut field)) = payload.try_next().await {
        let content_type = field.content_disposition().unwrap();
        let name = content_type.get_name().unwrap();

        match name {
            "title" | "description" | "category" => {
                let data = field.next().await;
                if data.is_none() { continue }
                let wrapped_data = &data.unwrap().unwrap();
                let parsed_data = std::str::from_utf8(&wrapped_data).unwrap();

                match name {
                    "title" => { title = parsed_data.to_string() }
                    "description" => { description = parsed_data.to_string() }
                    "category" => { category = parsed_data.to_string() }
                    _ => {}
                }
            }
            "torrent" => {
                if field.content_type().to_string() != "application/x-bittorrent" {
                    return Err(ServiceError::InvalidFileType)
                }

                while let Some(chunk) = field.next().await {
                    let data = chunk.unwrap();
                    torrent_cursor.write_all(&data)?;
                }
            }
            _ => {}
        }
    }

    let fields = CreateTorrent {
        title,
        description,
        category,
    };

    fields.verify()?;

    let position = torrent_cursor.position() as usize;
    let inner = torrent_cursor.get_ref();

    let torrent = match parse_torrent::decode_torrent(&inner[..position]) {
        Ok(torrent) => Ok(torrent),
        Err(_) => Err(ServiceError::InvalidTorrentFile)
    }?;

    Ok(TorrentRequest {
        fields,
        torrent,
    })
}
