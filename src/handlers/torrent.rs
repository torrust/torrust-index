use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Query};
use futures::{AsyncWriteExt, StreamExt, TryStreamExt};
use serde::{Deserialize};
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{NewTorrentResponse, OkResponse, TorrentResponse};
use crate::models::torrent::TorrentRequest;
use crate::utils::parse_torrent;
use crate::common::{WebAppData};
use std::io::Cursor;
use std::io::{Write};
use crate::models::torrent_file::{Torrent, File};
use crate::AsCSV;
use std::option::Option::Some;
use sqlx::{FromRow};
use crate::databases::database::Sorting;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/upload")
                .route(web::post().to(upload_torrent)))
            .service(web::resource("/download/{id}")
                .route(web::get().to(download_torrent)))
            .service(web::resource("/{id}")
                .route(web::get().to(get_torrent))
                .route(web::put().to(update_torrent))
                .route(web::delete().to(delete_torrent)))
    );
    cfg.service(
        web::scope("/torrents")
            .service(web::resource("")
                .route(web::get().to(get_torrents)))
    );
}

#[derive(FromRow)]
pub struct TorrentCount {
    pub count: i32,
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

#[derive(Debug, Deserialize)]
pub struct TorrentSearch {
    page_size: Option<u8>,
    page: Option<u32>,
    sort: Option<Sorting>,
    // expects comma separated string, eg: "?categories=movie,other,app"
    categories: Option<String>,
    search: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct TorrentUpdate {
    title: Option<String>,
    description: Option<String>
}

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    let mut torrent_request = get_torrent_request_from_payload(payload).await?;

    // update announce url to our own tracker url
    torrent_request.torrent.set_torrust_config(&app_data.cfg).await;

    let category = app_data.database.get_category_from_name(&torrent_request.fields.category).await
        .map_err(|_| ServiceError::InvalidCategory)?;

    let username = user.username;
    let info_hash = torrent_request.torrent.info_hash();
    let title = torrent_request.fields.title;
    //let category = torrent_request.fields.category;
    let description = torrent_request.fields.description;
    //let current_time = current_time() as i64;
    let file_size = torrent_request.torrent.file_size();
    let mut seeders = 0;
    let mut leechers = 0;

    if let Ok(torrent_info) = app_data.tracker.get_torrent_info(&info_hash).await {
        seeders = torrent_info.seeders;
        leechers = torrent_info.leechers;
    }

    let torrent_id = app_data.database.insert_torrent_and_get_id(username, info_hash, title, category.category_id, description, file_size, seeders, leechers).await?;

    // whitelist info hash on tracker
    let _ = app_data.tracker.whitelist_info_hash(torrent_request.torrent.info_hash()).await;

    let settings = app_data.cfg.settings.read().await;

    let upload_folder = settings.storage.upload_path.clone();
    let filepath = format!("{}/{}", upload_folder, torrent_id.to_string() + ".torrent");

    drop(settings);

    // save torrent file to uploads folder
    // if fails, delete torrent from database and return error
    if save_torrent_file(&upload_folder, &filepath, &torrent_request.torrent).await.is_err() {
        let _ = app_data.database.delete_torrent(torrent_id).await;
        return Err(ServiceError::InternalServerError)
    }

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse {
            torrent_id
        }
    }))
}

pub async fn download_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let torrent_id = get_torrent_id_from_request(&req)?;

    let settings = app_data.cfg.settings.read().await;

    // optional
    let user = app_data.auth.get_user_compact_from_request(&req).await;

    let filepath = format!("{}/{}", settings.storage.upload_path, torrent_id.to_string() + ".torrent");

    let mut torrent = match parse_torrent::read_torrent_from_file(&filepath) {
        Ok(torrent) => Ok(torrent),
        Err(e) => {
            println!("{:?}", e);
            Err(ServiceError::InternalServerError)
        }
    }?;

    let tracker_url = settings.tracker.url.clone();

    drop(settings);

    // add personal tracker url or default tracker url
    match user {
        Ok(user) => {
            let personal_announce_url = app_data.tracker.get_personal_announce_url(user.user_id).await.unwrap_or(tracker_url);
            torrent.announce = Some(personal_announce_url.clone());
            if let Some(list) = &mut torrent.announce_list {
                let vec = vec![personal_announce_url];
                list.insert(0, vec);
            }
        },
        Err(_) => {
            torrent.announce = Some(tracker_url);
        }
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

pub async fn get_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // optional
    let user = app_data.auth.get_user_compact_from_request(&req).await;

    let settings = app_data.cfg.settings.read().await;

    let torrent_id = get_torrent_id_from_request(&req)?;

    let torrent_listing = app_data.database.get_torrent_from_id(torrent_id).await?;

    let category = app_data.database.get_category_from_id(torrent_listing.category_id).await?;

    let mut torrent_response = TorrentResponse::from_listing(torrent_listing);

    torrent_response.category = category;

    let filepath = format!("{}/{}", settings.storage.upload_path, torrent_response.torrent_id.to_string() + ".torrent");

    let tracker_url = settings.tracker.url.clone();

    drop(settings);

    if let Ok(torrent) = parse_torrent::read_torrent_from_file(&filepath) {
        // add torrent file/files to response
        if let Some(files) = torrent.info.files {
            torrent_response.files = Some(files);
        } else {
            // todo: tidy up this code, it's error prone
            let file = File {
                path: vec![torrent.info.name],
                length: torrent.info.length.unwrap_or(0),
                md5sum: None
            };

            torrent_response.files = Some(vec![file]);
        }

        // add additional torrent tracker/trackers to response
        if let Some(trackers) = torrent.announce_list {
            for tracker in trackers {
                torrent_response.trackers.push(tracker[0].clone());
            }
        }
    }

    // add self-hosted tracker url
    match user {
        Ok(user) => {
            // if no user owned tracker key can be found, use default tracker url
            let personal_announce_url = app_data.tracker.get_personal_announce_url(user.user_id).await.unwrap_or(tracker_url);
            // add personal tracker url to front of vec
            torrent_response.trackers.insert(0, personal_announce_url);
        },
        Err(_) => {
            torrent_response.trackers.insert(0, tracker_url);
        }
    }

    // add magnet link
    let mut magnet = format!("magnet:?xt=urn:btih:{}&dn={}", torrent_response.info_hash, urlencoding::encode(&torrent_response.title));

    // add trackers from torrent file to magnet link
    for tracker in &torrent_response.trackers {
        magnet.push_str(&format!("&tr={}", urlencoding::encode(tracker)));
    }

    torrent_response.magnet_link = magnet;

    // get realtime seeders and leechers
    if let Ok(torrent_info) = app_data.tracker.get_torrent_info(&torrent_response.info_hash).await {
        torrent_response.seeders = torrent_info.seeders;
        torrent_response.leechers = torrent_info.leechers;
    }

    Ok(HttpResponse::Ok().json(OkResponse {
        data: torrent_response
    }))
}

pub async fn update_torrent(req: HttpRequest, payload: web::Json<TorrentUpdate>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    let torrent_id = get_torrent_id_from_request(&req)?;

    let torrent_listing = app_data.database.get_torrent_from_id(torrent_id).await?;

    // check if user is owner or administrator
    if torrent_listing.uploader != user.username && !user.administrator { return Err(ServiceError::Unauthorized) }

    // update torrent title
    if let Some(title) = &payload.title {
        let _res = app_data.database.update_torrent_title(torrent_id, title).await?;
    }

    // update torrent description
    if let Some(description) = &payload.description {
        let _res = app_data.database.update_torrent_description(torrent_id, description).await?;
    }

    let torrent_listing = app_data.database.get_torrent_from_id(torrent_id).await?;
    let torrent_response = TorrentResponse::from_listing(torrent_listing);

    Ok(HttpResponse::Ok().json(OkResponse {
        data: torrent_response
    }))
}

pub async fn delete_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator { return Err(ServiceError::Unauthorized) }

    let torrent_id = get_torrent_id_from_request(&req)?;

    // needed later for removing torrent from tracker whitelist
    let torrent_listing = app_data.database.get_torrent_from_id(torrent_id).await?;

    let _res = app_data.database.delete_torrent(torrent_id).await?;

    // remove info_hash from tracker whitelist
    let _ = app_data.tracker.remove_info_hash_from_whitelist(torrent_listing.info_hash).await;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse {
            torrent_id
        }
    }))
}

// eg: /torrents?categories=music,other,movie&search=bunny&sort=size_DESC
pub async fn get_torrents(params: Query<TorrentSearch>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let sort = params.sort.unwrap_or(Sorting::UploadedDesc);

    let page = params.page.unwrap_or(0);

    // make sure the min page size = 10
    let page_size = match params.page_size.unwrap_or(30) {
        0 ..= 9 => 10,
        v => v
    };

    let offset = (page * page_size as u32) as u64;

    let categories = params.categories.as_csv::<String>().unwrap_or(None);

    let torrents_response = app_data.database.get_torrents_search_sorted_paginated(&params.search, &categories, &sort, offset, page_size as u8).await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: torrents_response
    }))
}

async fn save_torrent_file(upload_folder: &str, filepath: &str, torrent: &Torrent) -> Result<(), ServiceError> {
    let torrent_bytes = match parse_torrent::encode_torrent(torrent) {
        Ok(v) => Ok(v),
        Err(_) => Err(ServiceError::InternalServerError)
    }?;

    // create torrent upload folder if it does not exist
    async_std::fs::create_dir_all(&upload_folder).await?;

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
