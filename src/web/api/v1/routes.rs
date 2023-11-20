//! Route initialization for the v1 API.
use std::env;
use std::sync::Arc;

use axum::extract::DefaultBodyLimit;
use axum::routing::get;
use axum::{Json, Router};
use serde_json::{json, Value};
use tower_http::compression::CompressionLayer;
use tower_http::cors::CorsLayer;

use super::contexts::about::handlers::about_page_handler;
use super::contexts::{about, category, proxy, settings, tag, torrent, user};
use crate::bootstrap::config::ENV_VAR_CORS_PERMISSIVE;
use crate::common::AppData;

pub const API_VERSION_URL_PREFIX: &str = "v1";

/// Add all API routes to the router.
#[allow(clippy::needless_pass_by_value)]
pub fn router(app_data: Arc<AppData>) -> Router {
    // code-review: should we use plural for the resource prefix: `users`, `categories`, `tags`?
    // See: https://stackoverflow.com/questions/6845772/should-i-use-singular-or-plural-name-convention-for-rest-resources

    let v1_api_routes = Router::new()
        .route("/", get(about_page_handler).with_state(app_data.clone()))
        .nest("/user", user::routes::router(app_data.clone()))
        .nest("/about", about::routes::router(app_data.clone()))
        .nest("/category", category::routes::router(app_data.clone()))
        .nest("/tag", tag::routes::router_for_single_resources(app_data.clone()))
        .nest("/tags", tag::routes::router_for_multiple_resources(app_data.clone()))
        .nest("/settings", settings::routes::router(app_data.clone()))
        .nest("/torrent", torrent::routes::router_for_single_resources(app_data.clone()))
        .nest("/torrents", torrent::routes::router_for_multiple_resources(app_data.clone()))
        .nest("/proxy", proxy::routes::router(app_data.clone()));

    let router = Router::new()
        .route("/", get(about_page_handler).with_state(app_data.clone()))
        .route("/health_check", get(health_check_handler).with_state(app_data))
        .nest(&format!("/{API_VERSION_URL_PREFIX}"), v1_api_routes);

    let router = if env::var(ENV_VAR_CORS_PERMISSIVE).is_ok() {
        router.layer(CorsLayer::permissive())
    } else {
        router
    };

    router.layer(DefaultBodyLimit::max(10_485_760)).layer(CompressionLayer::new())
}

/// Endpoint for container health check.
async fn health_check_handler() -> Json<Value> {
    Json(json!({ "status": "Ok" }))
}
