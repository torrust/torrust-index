//! API handlers for the the [`about`](crate::web::api::server::v1::contexts::about) API
//! context.
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::common::AppData;
use crate::web::api::server::v1::extractors::optional_user_id::ExtractOptionalLoggedInUser;

#[allow(clippy::unused_async)]
pub async fn about_page_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractOptionalLoggedInUser(maybe_user_id): ExtractOptionalLoggedInUser,
) -> Response {
    match app_data.about_service.get_about_page(maybe_user_id).await {
        Ok(html) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html).into_response(),
        Err(error) => error.into_response(),
    }
}

#[allow(clippy::unused_async)]
pub async fn license_page_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractOptionalLoggedInUser(maybe_user_id): ExtractOptionalLoggedInUser,
) -> Response {
    match app_data.about_service.get_license_page(maybe_user_id).await {
        Ok(html) => (StatusCode::OK, [(header::CONTENT_TYPE, "text/html; charset=utf-8")], html)
            .into_response()
            .into_response(),
        Err(error) => error.into_response(),
    }
}
