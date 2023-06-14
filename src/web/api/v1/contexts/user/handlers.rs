//! API handlers for the the [`user`](crate::web::api::v1::contexts::user) API
//! context.
use std::sync::Arc;

use axum::extract::{self, Host, Path, State};
use axum::Json;
use serde::Deserialize;

use super::forms::{JsonWebToken, LoginForm, RegistrationForm};
use super::responses::{self, NewUser, TokenResponse};
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::web::api::v1::responses::OkResponse;

// Registration

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

#[derive(Deserialize)]
pub struct TokenParam(String);

/// It handles the verification of the email verification token.
#[allow(clippy::unused_async)]
pub async fn email_verification_handler(State(app_data): State<Arc<AppData>>, Path(token): Path<TokenParam>) -> String {
    match app_data.registration_service.verify_email(&token.0).await {
        Ok(_) => String::from("Email verified, you can close this page."),
        Err(error) => error.to_string(),
    }
}

// Authentication

/// It handles the user login.
///
/// # Errors
///
/// It returns an error if the user could not be registered.
#[allow(clippy::unused_async)]
pub async fn login_handler(
    State(app_data): State<Arc<AppData>>,
    extract::Json(login_form): extract::Json<LoginForm>,
) -> Result<Json<OkResponse<TokenResponse>>, ServiceError> {
    match app_data
        .authentication_service
        .login(&login_form.login, &login_form.password)
        .await
    {
        Ok((token, user_compact)) => Ok(responses::logged_in_user(token, user_compact)),
        Err(error) => Err(error),
    }
}

/// It verifies a supplied JWT.
///
/// # Errors
///
/// It returns an error if:
///
/// - Unable to verify the supplied payload as a valid JWT.
/// - The JWT is not invalid or expired.
#[allow(clippy::unused_async)]
pub async fn verify_token_handler(
    State(app_data): State<Arc<AppData>>,
    extract::Json(token): extract::Json<JsonWebToken>,
) -> Result<Json<OkResponse<String>>, ServiceError> {
    match app_data.json_web_token.verify(&token.token).await {
        Ok(_) => Ok(axum::Json(OkResponse {
            data: "Token is valid.".to_string(),
        })),
        Err(error) => Err(error),
    }
}

/// It returns the base API URL without the port. For example: `http://localhost`.
fn api_base_url(host: &str) -> String {
    // HTTPS is not supported yet.
    // See https://github.com/torrust/torrust-index-backend/issues/131
    format!("http://{host}")
}
