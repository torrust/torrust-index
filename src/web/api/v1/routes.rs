//! Route initialization for the v1 API.
use std::sync::Arc;

use axum::routing::get;
use axum::Router;

use super::contexts::about::handlers::about_page_handler;
//use tower_http::cors::CorsLayer;
use super::contexts::{about, proxy, settings, tag, torrent};
use super::contexts::{category, user};
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

    Router::new()
        .route("/", get(about_page_handler).with_state(app_data))
        .nest(&format!("/{API_VERSION_URL_PREFIX}"), v1_api_routes)
    // For development purposes only.
    //
    //.layer(CorsLayer::permissive()) // Uncomment this line and the `use` import.
    //
    // It allows calling the API on a different port. For example
    // API: http://localhost:3000/v1
    // Webapp: http://localhost:8080
}
