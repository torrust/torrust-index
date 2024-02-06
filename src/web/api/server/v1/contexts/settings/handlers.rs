//! API handlers for the the [`category`](crate::web::api::server::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json, Response};

use crate::common::AppData;
use crate::web::api::server::v1::extractors::bearer_token::Extract;
use crate::web::api::server::v1::responses;

/// Get all settings.
///
/// # Errors
///
/// This function will return an error if the user does not have permission to
/// view all the settings.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(State(app_data): State<Arc<AppData>>, Extract(maybe_bearer_token): Extract) -> Response {
    let user_id = match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
        Ok(user_id) => user_id,
        Err(error) => return error.into_response(),
    };

    let all_settings = match app_data.settings_service.get_all(&user_id).await {
        Ok(all_settings) => all_settings,
        Err(error) => return error.into_response(),
    };

    Json(responses::OkResponseData { data: all_settings }).into_response()
}

/// Get public Settings.
#[allow(clippy::unused_async)]
pub async fn get_public_handler(State(app_data): State<Arc<AppData>>) -> Response {
    let public_settings = app_data.settings_service.get_public().await;

    Json(responses::OkResponseData { data: public_settings }).into_response()
}

/// Get website name.
#[allow(clippy::unused_async)]
pub async fn get_site_name_handler(State(app_data): State<Arc<AppData>>) -> Response {
    let site_name = app_data.settings_service.get_site_name().await;

    Json(responses::OkResponseData { data: site_name }).into_response()
}
