//! API routes for the [`tag`](crate::web::api::v1::contexts::tag) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::tag).
use std::sync::Arc;

use axum::routing::post;
use axum::Router;

use super::handlers::upload_torrent_handler;
use crate::common::AppData;

/// Routes for the [`tag`](crate::web::api::v1::contexts::tag) API context.
pub fn router_for_single_resources(app_data: Arc<AppData>) -> Router {
    Router::new().route("/upload", post(upload_torrent_handler).with_state(app_data))
}
