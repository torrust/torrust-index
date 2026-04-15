//! API handlers for the the [`settings`](crate::web::api::server::v1::contexts::settings) API
//! context.
use std::sync::Arc;

use axum::extract::State;
use axum::response::{IntoResponse, Json, Response};

use crate::common::AppData;
use crate::web::api::server::v1::extractors::require_permission::{
    GetPublicSettings, GetSettingsSecret, GetSiteName, RequirePermission,
};
use crate::web::api::server::v1::responses;

/// Get all settings (with secrets masked).
///
/// # Errors
///
/// This function will return an error if the user does not have permission to
/// view all the settings.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(
    State(app_data): State<Arc<AppData>>,
    RequirePermission(_actor, _): RequirePermission<GetSettingsSecret>,
) -> Response {
    let all_settings = match app_data.settings_service.get_all_masking_secrets().await {
        Ok(all_settings) => all_settings,
        Err(error) => return error.into_response(),
    };

    Json(responses::OkResponseData { data: all_settings }).into_response()
}

/// Get public Settings.
#[allow(clippy::unused_async)]
pub async fn get_public_handler(
    State(app_data): State<Arc<AppData>>,
    RequirePermission(_actor, _): RequirePermission<GetPublicSettings>,
) -> Response {
    match app_data.settings_service.get_public().await {
        Ok(public_settings) => Json(responses::OkResponseData { data: public_settings }).into_response(),
        Err(error) => error.into_response(),
    }
}

/// Get website name.
#[allow(clippy::unused_async)]
pub async fn get_site_name_handler(
    State(app_data): State<Arc<AppData>>,
    RequirePermission(_actor, _): RequirePermission<GetSiteName>,
) -> Response {
    match app_data.settings_service.get_site_name().await {
        Ok(site_name) => Json(responses::OkResponseData { data: site_name }).into_response(),
        Err(error) => error.into_response(),
    }
}
