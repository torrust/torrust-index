//! API routes for the [`tag`](crate::web::api::v1::contexts::tag) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::tag).
use std::sync::Arc;

use axum::routing::{delete, get, post};
use axum::Router;

use super::handlers::{add_handler, delete_handler, get_all_handler};
use crate::common::AppData;

// code-review: should we use `tags` also for single resources?

/// Routes for the [`tag`](crate::web::api::v1::contexts::tag) API context.
pub fn router_for_single_resources(app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/", post(add_handler).with_state(app_data.clone()))
        .route("/", delete(delete_handler).with_state(app_data))
}

/// Routes for the [`tag`](crate::web::api::v1::contexts::tag) API context.
pub fn router_for_multiple_resources(app_data: Arc<AppData>) -> Router {
    Router::new().route("/", get(get_all_handler).with_state(app_data))
}
