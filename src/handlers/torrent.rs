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

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/new")
                .route(web::post().to(create_torrent)))
            .service(web::resource("/upload/{id}")
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
            return Err(ServiceError::InvalidFileType)
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
