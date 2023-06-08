//! API routes for the [`about`](crate::web::api::v1::contexts::about) API context.
//!
//! Refer to the [API endpoint documentation](crate::web::api::v1::contexts::about).
use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use super::handlers::{about_page_handler, license_page_handler};
use crate::common::AppData;

/// It adds the routes to the router for the [`about`](crate::web::api::v1::contexts::about) API context.
pub fn add(prefix: &str, router: Router, app_data: Arc<AppData>) -> Router {
    router
        .route(
            &format!("{prefix}/about"),
            get(about_page_handler).with_state(app_data.clone()),
        )
        .route(
            &format!("{prefix}/about/license"),
            get(license_page_handler).with_state(app_data),
        )
}
