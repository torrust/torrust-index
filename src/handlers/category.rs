use actix_web::{HttpRequest, HttpResponse, Responder, web};
use actix_web::web::Query;
use serde::{Deserialize};

use crate::common::WebAppData;
use crate::errors::ServiceResult;
use crate::models::response::{CategoryResponse, OkResponse};
use crate::models::torrent::{TorrentListing, TorrentResponse};

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
}

pub async fn get_torrents(req: HttpRequest, info: Query<DisplayInfo>, app_data: WebAppData) -> ServiceResult<impl Responder> {
    let category = req.match_info().get("category").unwrap();
    let page = info.page.unwrap_or(0);
    let page_size = info.page_size.unwrap_or(30);
    let offset = page * page_size;

    // r#"SELECT * FROM torrust_torrents WHERE category_id = $1 AND info_hash IS NOT NULL
    //            LIMIT $2, $3"#,

    let res = sqlx::query_as!(
        TorrentResponse,
        r#"SELECT tt.*, 0 as seeders, 0 as leechers FROM torrust_torrents tt
               INNER JOIN torrust_categories tc ON tt.category_id = tc.category_id AND tc.name = $1
               LIMIT $2, $3"#,
        category,
        offset,
        page_size
    )
        .fetch_all(&app_data.database.pool)
        .await?;

    Ok(HttpResponse::Ok().json(OkResponse {
        data: res
    }))
}
