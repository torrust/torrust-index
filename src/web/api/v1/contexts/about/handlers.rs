//! API handlers for the the [`about`](crate::web::api::v1::contexts::about) API
//! context.
use std::sync::Arc;

use axum::extract::State;
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};

use crate::common::AppData;
use crate::services::about;

#[allow(clippy::unused_async)]
pub async fn about_page_handler(State(_app_data): State<Arc<AppData>>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        about::page(),
    )
        .into_response()
}

#[allow(clippy::unused_async)]
pub async fn license_page_handler(State(_app_data): State<Arc<AppData>>) -> Response {
    (
        StatusCode::OK,
        [(header::CONTENT_TYPE, "text/html; charset=utf-8")],
        about::license_page(),
    )
        .into_response()
}
