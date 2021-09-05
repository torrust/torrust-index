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
use crate::models::torrent_listing::TorrentListing;
use crate::models::user::{User, Claims};
use crate::models::response::{OkResponse, NewTorrentResponse};
use actix_web::web::Query;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/new")
                .route(web::post().to(create_torrent)))
            .service(web::resource("/upload/{id}")
                .route(web::post().to(upload_torrent)))
            .service(web::resource("/")
                .route(web::get().to(get_torrents)))
    );
}

#[derive(Debug, Deserialize)]
pub struct DisplayInfo {
    page_size: Option<i32>,
    page: Option<i32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateTorrent {
    pub title: String,
    pub description: String,
    pub category: String,
}

pub async fn get_torrents(info: Query<DisplayInfo>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let page = info.page.unwrap_or(0);
    let page_size = info.page_size.unwrap_or(30);
    let offset = page * page_size;

    let res = sqlx::query_as!(
        TorrentListing,
        r#"SELECT * FROM torrust_torrents LIMIT $1, $2"#,
        offset,
        page_size
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
        r#"INSERT INTO torrust_torrents (uploader_id, title, description, category)
        VALUES ($1, $2, $3, $4)
        RETURNING torrent_id as "torrent_id: i64""#,
        user.user_id,
        payload.title,
        payload.description,
        payload.category
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
