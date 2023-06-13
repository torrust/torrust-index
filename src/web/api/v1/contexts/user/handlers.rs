//! API handlers for the the [`user`](crate::web::api::v1::contexts::user) API
//! context.
use std::sync::Arc;

use axum::extract::{self, Host, State};
use axum::Json;

use super::forms::RegistrationForm;
use super::responses::{self, NewUser};
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::web::api::v1::responses::OkResponse;

/// It handles the registration of a new user.
///
/// # Errors
///
/// It returns an error if the user could not be registered.
#[allow(clippy::unused_async)]
pub async fn registration_handler(
    State(app_data): State<Arc<AppData>>,
    Host(host_from_header): Host,
    extract::Json(registration_form): extract::Json<RegistrationForm>,
) -> Result<Json<OkResponse<NewUser>>, ServiceError> {
    let api_base_url = app_data
        .cfg
        .get_api_base_url()
        .await
        .unwrap_or(api_base_url(&host_from_header));

    match app_data
        .registration_service
        .register_user(&registration_form, &api_base_url)
        .await
    {
        Ok(user_id) => Ok(responses::added_user(user_id)),
        Err(error) => Err(error),
    }
}

/// It returns the base API URL without the port. For example: `http://localhost`.
fn api_base_url(host: &str) -> String {
    // HTTPS is not supported yet.
    // See https://github.com/torrust/torrust-index-backend/issues/131
    format!("http://{host}")
}
