use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::Query;
use serde::{Deserialize};

use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::models::response::{CategoryResponse, OkResponse, TorrentsResponse, TorrentResponse};
use crate::models::torrent::{TorrentListing};

pub fn init_routes(cfg: &mut web::ServiceConfig) {
    cfg.service(
        web::scope("/category")
            .service(web::resource("/")
                .route(web::get().to(get_categories)))
            .service(web::resource("/{category}/torrents")
                .route(web::get().to(get_torrents)))
    );
}

pub async fn get_categories(app_data: WebAppData) -> ServiceResult<impl Responder> {
    // Count torrents with category
    let res = sqlx::query_as::<_, CategoryResponse>(
        r#"SELECT name, COUNT(tt.category_id) as num_torrents
           FROM torrust_categories tc
           LEFT JOIN torrust_torrents tt on tc.category_id = tt.category_id
           GROUP BY tc.name"#
    )
        .fetch_all(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: res
    }))
}

#[derive(Debug, Deserialize)]
pub struct DisplayInfo {
    page_size: Option<i32>,
    page: Option<i32>,
    sort: Option<String>,
}

pub struct TorrentCount {
    pub count: i32,
}

pub async fn get_torrents(req: HttpRequest, info: Query<DisplayInfo>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let category = req.match_info().get("category").unwrap();
    let page = info.page.unwrap_or(0);
    let page_size = info.page_size.unwrap_or(30);
    let offset = page * page_size;


    let mut count: TorrentCount = sqlx::query_as!(
        TorrentCount,
        r#"SELECT COUNT(torrent_id) as count FROM torrust_torrents tt
               INNER JOIN torrust_categories tc ON tt.category_id = tc.category_id AND tc.name = $1
        "#,
        category
    )
        .fetch_one(&app_data.database.pool)
        .await?;

    let res = match &info.sort {
        None => app_data.database.get_torrents_order_by_upload_date_desc(category, offset, page_size).await,
        Some(sort) => {
            match sort.as_str() {
                "uploaded" => app_data.database.get_torrents_order_by_upload_date_desc(category, offset, page_size).await,
                "seeders" => app_data.database.get_torrents_order_by_seeders_desc(category, offset, page_size).await,
                "leechers" => app_data.database.get_torrents_order_by_leechers_desc(category, offset, page_size).await,
                _ => app_data.database.get_torrents_order_by_upload_date_desc(category, offset, page_size).await
            }
        }
    }?;

    // app_data.database.get_torrents_order_by_seeders_desc(category, offset, page_size).await?;

    let torrents_response = TorrentsResponse {
        total: count.count,
        results: res
    };

    Ok(HttpResponse::Ok().json(OkResponse {
        data: torrents_response
    }))
}
