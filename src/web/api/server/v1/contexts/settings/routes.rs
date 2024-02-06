//! API routes for the [`settings`](crate::web::api::server::v1::contexts::settings) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::server::v1::contexts::settings).
use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use super::handlers::{get_all_handler, get_public_handler, get_site_name_handler};
use crate::common::AppData;

/// Routes for the [`category`](crate::web::api::server::v1::contexts::category) API context.
pub fn router(app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/", get(get_all_handler).with_state(app_data.clone()))
        .route("/name", get(get_site_name_handler).with_state(app_data.clone()))
        .route("/public", get(get_public_handler).with_state(app_data))
}
