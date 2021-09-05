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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/new")
                .route(web::post().to(create_torrent)))
            .service(web::resource("/upload/{id}")
                .route(web::post().to(upload_torrent)))
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

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = req.match_info().get("id").unwrap();

    let filepath =
        format!("{}/{}", app_data.cfg.storage.upload_path, torrent_id);

    create_dir_all(&filepath).await?;
    save_torrent_file(&filepath, payload).await?;

    Ok(HttpResponse::Ok())
}

async fn save_torrent_file(path: &str, mut payload: Multipart) -> Result<(), ServiceError> {
    while let Ok(Some(mut field)) = payload.try_next().await {
        if field.content_type().to_string() != "application/x-bittorrent" {
            return Err(ServiceError::InvalidFileType);
        }

        let content_type = field
            .content_disposition()
            .ok_or_else(|| ServiceError::InvalidTorrentFile)?;

        let filename = content_type
            .get_filename()
            .ok_or_else(|| ServiceError::InvalidTorrentFile)?;

        let filepath =
            format!("{}/{}", path, sanitize_filename::sanitize(&filename));

        let mut f = async_std::fs::File::create(&filepath).await?;

        // Field in turn is stream of *Bytes* object
        while let Some(chunk) = field.next().await {
            let data = chunk.unwrap();
            AsyncWriteExt::write_all(&mut f, &data).await?;
        }

        let torrent = parse_torrent::read_torrent(&filepath)?;

        println!("name:\t\t{}", torrent.info.name);
        println!("announce:\t{:?}", torrent.announce);
        println!("nodes:\t\t{:?}", torrent.nodes);
        if let &Some(ref al) = &torrent.announce_list {
            for a in al {
                println!("announce list:\t{}", a[0]);
            }
        }
        println!("httpseeds:\t{:?}", torrent.httpseeds);
        println!("creation date:\t{:?}", torrent.creation_date);
        println!("comment:\t{:?}", torrent.comment);
        println!("created by:\t{:?}", torrent.created_by);
        println!("encoding:\t{:?}", torrent.encoding);
        println!("piece length:\t{:?}", torrent.info.piece_length);
        println!("private:\t{:?}", torrent.info.private);
        println!("root hash:\t{:?}", torrent.info.root_hash);
        println!("md5sum:\t\t{:?}", torrent.info.md5sum);
        println!("path:\t\t{:?}", torrent.info.path);
        if let &Some(ref files) = &torrent.info.files {
            for f in files {
                println!("file path:\t{:?}", f.path);
                println!("file length:\t{}", f.length);
                println!("file md5sum:\t{:?}", f.md5sum);
            }
        }
    }

    Err(ServiceError::InvalidTorrentFile)
}
