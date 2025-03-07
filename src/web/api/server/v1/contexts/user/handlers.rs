//! API handlers for the the [`user`](crate::web::api::server::v1::contexts::user) API
//! context.
use std::sync::Arc;

use axum::extract::{self, Host, Path, Query, State};
use axum::response::{IntoResponse, Response};
use axum::Json;
use serde::Deserialize;

use super::forms::{ChangePasswordForm, JsonWebToken, LoginForm, RegistrationForm};
use super::responses::{self};
use crate::common::AppData;
use crate::services::user::ListingRequest;
use crate::web::api::server::v1::extractors::optional_user_id::ExtractOptionalLoggedInUser;
use crate::web::api::server::v1::responses::OkResponseData;

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
) -> Response {
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
        Ok(user_id) => responses::added_user(user_id).into_response(),
        Err(error) => error.into_response(),
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
/// It returns an error if:
///
/// - Unable to verify the supplied payload as a valid JWT.
/// - The JWT is not invalid or expired.
#[allow(clippy::unused_async)]
pub async fn login_handler(
    State(app_data): State<Arc<AppData>>,
    extract::Json(login_form): extract::Json<LoginForm>,
) -> Response {
    match app_data
        .authentication_service
        .login(&login_form.login, &login_form.password)
        .await
    {
        Ok((token, user_compact)) => responses::logged_in_user(token, user_compact).into_response(),
        Err(error) => error.into_response(),
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
) -> Response {
    match app_data.json_web_token.verify(&token.token).await {
        Ok(_) => axum::Json(OkResponseData {
            data: "Token is valid.".to_string(),
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

#[derive(Deserialize)]
pub struct UsernameParam(pub String);

/// It renews the JWT.
///
/// # Errors
///
/// It returns an error if:
///
/// - Unable to parse the supplied payload as a valid JWT.
/// - The JWT is not invalid or expired.
#[allow(clippy::unused_async)]
pub async fn renew_token_handler(
    State(app_data): State<Arc<AppData>>,
    extract::Json(token): extract::Json<JsonWebToken>,
) -> Response {
    match app_data.authentication_service.renew_token(&token.token).await {
        Ok((token, user_compact)) => responses::renewed_token(token, user_compact).into_response(),
        Err(error) => error.into_response(),
    }
}

/// It changes the user's password.
///
/// # Errors
///
/// It returns an error if:
///
/// - The user account is not found.
#[allow(clippy::unused_async)]
#[allow(clippy::missing_panics_doc)]
pub async fn change_password_handler(
    State(app_data): State<Arc<AppData>>,
    ExtractOptionalLoggedInUser(maybe_user_id): ExtractOptionalLoggedInUser,
    extract::Json(change_password_form): extract::Json<ChangePasswordForm>,
) -> Response {
    match app_data
        .profile_service
        .change_password(maybe_user_id, &change_password_form)
        .await
    {
        Ok(()) => Json(OkResponseData {
            data: format!("Password changed for user with ID: {}", maybe_user_id.unwrap()),
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

/// It bans a user from the index.
///
/// # Errors
///
/// This function will return if:
///
/// - The JWT provided by the banning authority was not valid.
/// - The user could not be banned: it does not exist, etcetera.
#[allow(clippy::unused_async)]
pub async fn ban_handler(
    State(app_data): State<Arc<AppData>>,
    Path(to_be_banned_username): Path<UsernameParam>,
    ExtractOptionalLoggedInUser(maybe_user_id): ExtractOptionalLoggedInUser,
) -> Response {
    // todo: add reason and `date_expiry` parameters to request

    match app_data.ban_service.ban_user(&to_be_banned_username.0, maybe_user_id).await {
        Ok(()) => Json(OkResponseData {
            data: format!("Banned user: {}", to_be_banned_username.0),
        })
        .into_response(),
        Err(error) => error.into_response(),
    }
}

/// It returns the base API URL without the port. For example: `http://localhost`.
fn api_base_url(host: &str) -> String {
    // HTTPS is not supported yet.
    // See https://github.com/torrust/torrust-index/issues/131
    format!("http://{host}")
}

/// It handles the request to get all the user profiles.
///
///It returns a list of user profiles matching the search criteria.
///
/// # Errors
///
/// It returns an error if:
/// There is a database error
/// There is a problem authorizing the action.
/// The user is not authorized to perform the action
#[allow(clippy::unused_async)]
pub async fn get_user_profiles_handler(
    State(app_data): State<Arc<AppData>>,
    Query(criteria): Query<ListingRequest>,
    ExtractOptionalLoggedInUser(maybe_user_id): ExtractOptionalLoggedInUser,
) -> Response {
    let listing = match app_data
        .listing_service
        .listing_specification_from_user_request(maybe_user_id, &criteria)
        .await
    {
        Ok(listing_value) => listing_value,
        Err(err) => return err.into_response(),
    };

    match app_data.listing_service.generate_user_profile_listing(&listing).await {
        Ok(users) => Json(crate::web::api::server::v1::responses::OkResponseData { data: users }).into_response(),
        Err(error) => error.into_response(),
    }
}
