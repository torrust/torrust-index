//! API routes for the [`category`](crate::web::api::v1::contexts::category) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::category).
use std::sync::Arc;

use axum::routing::{get, post};
use axum::Router;

use super::handlers::{add_handler, get_all_handler};
use crate::common::AppData;

/// Routes for the [`category`](crate::web::api::v1::contexts::category) API context.
pub fn router(app_data: Arc<AppData>) -> Router {
    Router::new()
        .route("/", get(get_all_handler).with_state(app_data.clone()))
        .route("/", post(add_handler).with_state(app_data))
}
