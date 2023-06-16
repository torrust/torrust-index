//! API routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::torrent).
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use super::handlers::{download_torrent_handler, get_torrents_handler, upload_torrent_handler};
use crate::common::AppData;

/// Routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context for single resources.
pub fn router_for_single_resources(app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/upload", post(upload_torrent_handler).with_state(app_data.clone()))
        .route("/download/:info_hash", get(download_torrent_handler).with_state(app_data))
}

/// Routes for the [`torrent`](crate::web::api::v1::contexts::torrent) API context for multiple resources.
pub fn router_for_multiple_resources(app_data: Arc<AppData>) -> Router {
    Router::new().route("/", get(get_torrents_handler).with_state(app_data))
}
