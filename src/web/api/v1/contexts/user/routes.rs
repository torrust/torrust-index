//! API routes for the [`user`](crate::web::api::v1::contexts::user) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::user).
use std::sync::Arc;

use axum::routing::post;
use axum::Router;

use super::handlers::registration_handler;
use crate::common::AppData;

/// Routes for the [`user`](crate::web::api::v1::contexts::user) API context.
pub fn router(app_data: Arc<AppData>) -> Router {
    Router::new().route("/register", post(registration_handler).with_state(app_data))
}
