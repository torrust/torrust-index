//! Route initialization for the v1 API.
use std::sync::Arc;

use axum::Router;

use super::contexts::about;
use crate::common::AppData;

/// Add all API routes to the router.
#[allow(clippy::needless_pass_by_value)]
pub fn router(app_data: Arc<AppData>) -> Router {
    let router = Router::new();

    add(router, app_data)
}

/// Add the routes for the v1 API.
fn add(router: Router, app_data: Arc<AppData>) -> Router {
    let v1_prefix = "/v1".to_string();

    about::routes::add(&v1_prefix, router, app_data)
}
