//! API handlers for the the [`category`](crate::web::api::v1::contexts::category) API
//! context.
use std::sync::Arc;

use axum::extract::{self, State};
use axum::response::Json;

use crate::common::AppData;
use crate::config::{ConfigurationPublic, TorrustBackend};
use crate::errors::ServiceError;
use crate::web::api::v1::extractors::bearer_token::Extract;
use crate::web::api::v1::responses::{self, OkResponseData};

/// Get all settings.
///
/// # Errors
///
/// This function will return an error if the user does not have permission to
/// view all the settings.
#[allow(clippy::unused_async)]
pub async fn get_all_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
) -> Result<Json<OkResponseData<TorrustBackend>>, ServiceError> {
    let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await?;

    let all_settings = app_data.settings_service.get_all(&user_id).await?;

    Ok(Json(responses::OkResponseData { data: all_settings }))
}

/// Get public Settings.
#[allow(clippy::unused_async)]
pub async fn get_public_handler(State(app_data): State<Arc<AppData>>) -> Json<responses::OkResponseData<ConfigurationPublic>> {
    let public_settings = app_data.settings_service.get_public().await;

    Json(responses::OkResponseData { data: public_settings })
}

/// Get website name.
#[allow(clippy::unused_async)]
pub async fn get_site_name_handler(State(app_data): State<Arc<AppData>>) -> Json<responses::OkResponseData<String>> {
    let site_name = app_data.settings_service.get_site_name().await;

    Json(responses::OkResponseData { data: site_name })
}

/// Update all the settings.
///
/// # Errors
///
/// This function will return an error if:
///
/// - The user does not have permission to update the settings.
/// - The settings could not be updated because they were loaded from env vars.
///   See <https://github.com/torrust/torrust-index-backend/issues/144.>
#[allow(clippy::unused_async)]
pub async fn update_handler(
    State(app_data): State<Arc<AppData>>,
    Extract(maybe_bearer_token): Extract,
    extract::Json(torrust_backend): extract::Json<TorrustBackend>,
) -> Result<Json<OkResponseData<TorrustBackend>>, ServiceError> {
    let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await?;

    let new_settings = app_data.settings_service.update_all(torrust_backend, &user_id).await?;

    Ok(Json(responses::OkResponseData { data: new_settings }))
}
