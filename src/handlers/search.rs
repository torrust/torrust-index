use actix_multipart::Multipart;
use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::{Query, Form, Json};
use async_std::fs::create_dir_all;
use async_std::prelude::*;
use futures::{AsyncWriteExt, StreamExt, TryStreamExt};
use serde::{Deserialize, Serialize};
use crate::errors::{ServiceError, ServiceResult};
use crate::models::response::{NewTorrentResponse, OkResponse, TorrentResponse, TorrentsResponse};
use crate::models::torrent::{TorrentListing, TorrentRequest};
use crate::utils::parse_torrent;
use crate::common::{WebAppData, Username};
use crate::models::user::{User, Claims};
use std::io::Cursor;
use std::io::{Write};
use crate::models::torrent_file::Torrent;
use std::error::Error;
use crate::utils::time::current_time;
use std::collections::HashMap;
use crate::handlers::category::TorrentCount;

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/search")
            // eg: http://localhost:3000/search/?query=big
            .service(web::resource("/")
                .route(web::get().to(get_torrent_search_results)))
    );
}

// search params
#[derive(Debug, Deserialize)]
pub struct Search {
    pub query: String,
}

async fn get_torrent_search_results(web::Query(info): web::Query<Search>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let query = format!("%{}%", info.query);

    let count: TorrentCount = sqlx::query_as!(
        TorrentCount,
        r#"SELECT COUNT(torrent_id) as count FROM torrust_torrents WHERE title LIKE $1"#,
        query
    )
        .fetch_one(&app_data.database.pool)
        .await?;

    let res = sqlx::query_as!(
        TorrentListing,
        r#"SELECT * FROM torrust_torrents WHERE title LIKE $1"#,
        query
    )
        .fetch_all(&app_data.database.pool)
        .await?;

    let torrents_response = TorrentsResponse {
        total: count.count,
        results: res
    };

    Ok(HttpResponse::Ok().json(OkResponse {
        data: torrents_response
    }))
}
