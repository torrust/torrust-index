//! Route initialization for the v1 API.
use std::sync::Arc;

use axum::Router;

//use tower_http::cors::CorsLayer;
use super::contexts::{about, user};
use crate::common::AppData;

/// Add all API routes to the router.
#[allow(clippy::needless_pass_by_value)]
pub fn router(app_data: Arc<AppData>) -> Router {
    let user_routes = user::routes::router(app_data.clone());
    let about_routes = about::routes::router(app_data);

    let api_routes = Router::new().nest("/user", user_routes).nest("/about", about_routes);

    // For development purposes only.
    // It allows calling the API on a different port. For example
    // API: http://localhost:3000/v1
    // Webapp: http://localhost:8080
    //Router::new().nest("/v1", api_routes).layer(CorsLayer::permissive())

    Router::new().nest("/v1", api_routes)
}
