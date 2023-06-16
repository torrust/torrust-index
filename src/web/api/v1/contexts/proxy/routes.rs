//! API routes for the [`proxy`](crate::web::api::v1::contexts::proxy) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::proxy).
use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use super::handlers::get_proxy_image_handler;
use crate::common::AppData;

/// Routes for the [`about`](crate::web::api::v1::contexts::about) API context.
pub fn router(app_data: Arc<AppData>) -> Router {
    Router::new().route("/image/:url", get(get_proxy_image_handler).with_state(app_data))
}
