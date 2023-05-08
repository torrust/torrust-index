use std::io::{Cursor, Write};
use std::str::FromStr;

use actix_multipart::Multipart;
use actix_web::web::Query;
use actix_web::{web, HttpRequest, HttpResponse, Responder};
use futures::{StreamExt, TryStreamExt};
use serde::Deserialize;
use sqlx::FromRow;

use crate::common::WebAppData;
use crate::databases::database::Sorting;
use crate::errors::{ServiceError, ServiceResult};
use crate::models::info_hash::InfoHash;
use crate::models::response::{NewTorrentResponse, OkResponse, TorrentResponse};
use crate::models::torrent::TorrentRequest;
use crate::utils::parse_torrent;
use crate::AsCSV;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/torrent")
            .service(web::resource("/upload").route(web::post().to(upload_torrent)))
            .service(web::resource("/download/{info_hash}").route(web::get().to(download_torrent_handler)))
            .service(
                web::resource("/{id}")
                    .route(web::get().to(get_torrent))
                    .route(web::put().to(update_torrent))
                    .route(web::delete().to(delete_torrent)),
            ),
    );
    cfg.service(web::scope("/torrents").service(web::resource("").route(web::get().to(get_torrents))));
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
    pub fn verify(&self) -> Result<(), ServiceError> {
        if !self.title.is_empty() && !self.category.is_empty() {
            return Ok(());
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
    description: Option<String>,
}

pub async fn upload_torrent(req: HttpRequest, payload: Multipart, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // get torrent and fields from request
    let mut torrent_request = get_torrent_request_from_payload(payload).await?;

    // update announce url to our own tracker url
    torrent_request.torrent.set_torrust_config(&app_data.cfg).await;

    // get the correct category name from database
    let category = app_data
        .database
        .get_category_from_name(&torrent_request.fields.category)
        .await
        .map_err(|_| ServiceError::InvalidCategory)?;

    // insert entire torrent in database
    let torrent_id = app_data
        .database
        .insert_torrent_and_get_id(
            &torrent_request.torrent,
            user.user_id,
            category.category_id,
            &torrent_request.fields.title,
            &torrent_request.fields.description,
        )
        .await?;

    // update torrent tracker stats
    let _ = app_data
        .tracker
        .update_torrent_tracker_stats(torrent_id, &torrent_request.torrent.info_hash())
        .await;

    // whitelist info hash on tracker
    // code-review: why do we always try to whitelist the torrent on the tracker?
    // shouldn't we only do this if the torrent is in "Listed" mode?
    if let Err(e) = app_data
        .tracker
        .whitelist_info_hash(torrent_request.torrent.info_hash())
        .await
    {
        // if the torrent can't be whitelisted somehow, remove the torrent from database
        let _ = app_data.database.delete_torrent(torrent_id).await;
        return Err(e);
    }

    // respond with the newly uploaded torrent id
    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse { torrent_id },
    }))
}

/// Returns the torrent as a byte stream `application/x-bittorrent`.
///
/// # Errors
///
/// Returns `ServiceError::BadRequest` if the torrent infohash is invalid.
pub async fn download_torrent_handler(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let info_hash = get_torrent_info_hash_from_request(&req)?;

    // optional
    let user = app_data.auth.get_user_compact_from_request(&req).await;

    let mut torrent = app_data.database.get_torrent_from_info_hash(&info_hash).await?;

    let settings = app_data.cfg.settings.read().await;

    let tracker_url = settings.tracker.url.clone();

    drop(settings);

    // add personal tracker url or default tracker url
    match user {
        Ok(user) => {
            let personal_announce_url = app_data
                .tracker
                .get_personal_announce_url(user.user_id)
                .await
                .unwrap_or(tracker_url);
            torrent.announce = Some(personal_announce_url.clone());
            if let Some(list) = &mut torrent.announce_list {
                let vec = vec![personal_announce_url];
                list.insert(0, vec);
            }
        }
        Err(_) => {
            torrent.announce = Some(tracker_url);
        }
    }

    let buffer = parse_torrent::encode_torrent(&torrent).map_err(|_| ServiceError::InternalServerError)?;

    Ok(HttpResponse::Ok().content_type("application/x-bittorrent").body(buffer))
}

pub async fn get_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    // optional
    let user = app_data.auth.get_user_compact_from_request(&req).await;

    let settings = app_data.cfg.settings.read().await;

    let torrent_id = get_torrent_id_from_request(&req)?;

    let torrent_listing = app_data.database.get_torrent_listing_from_id(torrent_id).await?;

    let category = app_data.database.get_category_from_id(torrent_listing.category_id).await?;

    let mut torrent_response = TorrentResponse::from_listing(torrent_listing);

    torrent_response.category = category;

    let tracker_url = settings.tracker.url.clone();

    drop(settings);

    torrent_response.files = app_data.database.get_torrent_files_from_id(torrent_id).await?;

    if torrent_response.files.len() == 1 {
        let torrent_info = app_data.database.get_torrent_info_from_id(torrent_id).await?;

        torrent_response
            .files
            .iter_mut()
            .for_each(|v| v.path = vec![torrent_info.name.to_string()]);
    }

    torrent_response.trackers = app_data
        .database
        .get_torrent_announce_urls_from_id(torrent_id)
        .await
        .map(|v| v.into_iter().flatten().collect())?;

    // add tracker url
    match user {
        Ok(user) => {
            // if no user owned tracker key can be found, use default tracker url
            let personal_announce_url = app_data
                .tracker
                .get_personal_announce_url(user.user_id)
                .await
                .unwrap_or(tracker_url);
            // add personal tracker url to front of vec
            torrent_response.trackers.insert(0, personal_announce_url);
        }
        Err(_) => {
            torrent_response.trackers.insert(0, tracker_url);
        }
    }

    // todo: extract a struct or function to build the magnet links

    // add magnet link
    let mut magnet = format!(
        "magnet:?xt=urn:btih:{}&dn={}",
        torrent_response.info_hash,
        urlencoding::encode(&torrent_response.title)
    );

    // add trackers from torrent file to magnet link
    for tracker in &torrent_response.trackers {
        magnet.push_str(&format!("&tr={}", urlencoding::encode(tracker)));
    }

    torrent_response.magnet_link = magnet;

    // get realtime seeders and leechers
    if let Ok(torrent_info) = app_data
        .tracker
        .get_torrent_info(torrent_response.torrent_id, &torrent_response.info_hash)
        .await
    {
        torrent_response.seeders = torrent_info.seeders;
        torrent_response.leechers = torrent_info.leechers;
    }

    Ok(HttpResponse::Ok().json(OkResponse { data: torrent_response }))
}

pub async fn update_torrent(
    req: HttpRequest,
    payload: web::Json<TorrentUpdate>,
    app_data: WebAppData,
) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    let torrent_id = get_torrent_id_from_request(&req)?;

    let torrent_listing = app_data.database.get_torrent_listing_from_id(torrent_id).await?;

    // check if user is owner or administrator
    if torrent_listing.uploader != user.username && !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    // update torrent title
    if let Some(title) = &payload.title {
        app_data.database.update_torrent_title(torrent_id, title).await?;
    }

    // update torrent description
    if let Some(description) = &payload.description {
        app_data.database.update_torrent_description(torrent_id, description).await?;
    }

    let torrent_listing = app_data.database.get_torrent_listing_from_id(torrent_id).await?;

    let torrent_response = TorrentResponse::from_listing(torrent_listing);

    Ok(HttpResponse::Ok().json(OkResponse { data: torrent_response }))
}

pub async fn delete_torrent(req: HttpRequest, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let user = app_data.auth.get_user_compact_from_request(&req).await?;

    // check if user is administrator
    if !user.administrator {
        return Err(ServiceError::Unauthorized);
    }

    let torrent_id = get_torrent_id_from_request(&req)?;

    // needed later for removing torrent from tracker whitelist
    let torrent_listing = app_data.database.get_torrent_listing_from_id(torrent_id).await?;

    app_data.database.delete_torrent(torrent_id).await?;

    // remove info_hash from tracker whitelist
    let _ = app_data
        .tracker
        .remove_info_hash_from_whitelist(torrent_listing.info_hash)
        .await;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: NewTorrentResponse { torrent_id },
    }))
}

// eg: /torrents?categories=music,other,movie&search=bunny&sort=size_DESC
pub async fn get_torrents(params: Query<TorrentSearch>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let sort = params.sort.unwrap_or(Sorting::UploadedDesc);

    let page = params.page.unwrap_or(0);

    // make sure the min page size = 10
    let page_size = match params.page_size.unwrap_or(30) {
        0..=9 => 10,
        v => v,
    };

    let offset = (page * page_size as u32) as u64;

    let categories = params.categories.as_csv::<String>().unwrap_or(None);

    let torrents_response = app_data
        .database
        .get_torrents_search_sorted_paginated(&params.search, &categories, &sort, offset, page_size as u8)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse { data: torrents_response }))
}

fn get_torrent_id_from_request(req: &HttpRequest) -> Result<i64, ServiceError> {
    match req.match_info().get("id") {
        None => Err(ServiceError::BadRequest),
        Some(torrent_id) => match torrent_id.parse() {
            Err(_) => Err(ServiceError::BadRequest),
            Ok(v) => Ok(v),
        },
    }
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

    let mut title = "".to_string();
    let mut description = "".to_string();
    let mut category = "".to_string();

    while let Ok(Some(mut field)) = payload.try_next().await {
        match field.content_disposition().get_name().unwrap() {
            "title" | "description" | "category" => {
                let data = field.next().await;
                if data.is_none() {
                    continue;
                }
                let wrapped_data = &data.unwrap().unwrap();
                let parsed_data = std::str::from_utf8(wrapped_data).unwrap();

                match field.content_disposition().get_name().unwrap() {
                    "title" => title = parsed_data.to_string(),
                    "description" => description = parsed_data.to_string(),
                    "category" => category = parsed_data.to_string(),
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

    let fields = CreateTorrent {
        title,
        description,
        category,
    };

    fields.verify()?;

    let position = torrent_cursor.position() as usize;
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
