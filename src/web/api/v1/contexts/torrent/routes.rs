//! API routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::torrent).
use std::sync::Arc;

use axum::routing::{delete, get, post, put};
use axum::Router;

use super::handlers::{
    create_random_torrent_handler, delete_torrent_handler, download_torrent_handler, get_torrent_info_handler,
    get_torrents_handler, update_torrent_info_handler, upload_torrent_handler,
};
use crate::common::AppData;

/// Routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context for single resources.
pub fn router_for_single_resources(app_data: Arc<AppData>) -> Router {
    let torrent_info_routes = Router::new()
        .route("/", get(get_torrent_info_handler).with_state(app_data.clone()))
        .route("/", put(update_torrent_info_handler).with_state(app_data.clone()))
        .route("/", delete(delete_torrent_handler).with_state(app_data.clone()));

    Router::new()
        .route("/upload", post(upload_torrent_handler).with_state(app_data.clone()))
        .route(
            "/download/:info_hash",
            get(download_torrent_handler).with_state(app_data.clone()),
        )
        .route(
            "/meta-info/random/:uuid",
            get(create_random_torrent_handler).with_state(app_data),
        )
        .nest("/:info_hash", torrent_info_routes)
}

/// Routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context for multiple resources.
pub fn router_for_multiple_resources(app_data: Arc<AppData>) -> Router {
    Router::new().route("/", get(get_torrents_handler).with_state(app_data))
}
