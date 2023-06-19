//! API authentication.
//!
//! The API uses a [bearer token authentication scheme](https://datatracker.ietf.org/doc/html/rfc6750).
//!
//! API clients must have an account on the website to be able to use the API.
//!
//! # Authentication flow
//!
//! - [Registration](#registration)
//! - [Login](#login)
//! - [Using the token](#using-the-token)
//!
//! ## Registration
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"username":"indexadmin","email":"indexadmin@torrust.com","password":"BenoitMandelbrot1924","confirm_password":"BenoitMandelbrot1924"}' \
//!   http://127.0.0.1:3000/v1/user/register
//! ```
//!
//! **NOTICE**: The first user is automatically an administrator. Currently,
//! there is no way to change this. There is one administrator per instance.
//! And you cannot delete the administrator account or make another user an
//! administrator. For testing purposes, you can create a new administrator
//! account by creating a new user and then manually changing the `administrator`
//! field in the `torrust_users` table to `1`.
//!
//! ## Login
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --request POST \
//!   --data '{"login":"indexadmin","password":"BenoitMandelbrot1924"}' \
//!   http://127.0.0.1:3000/v1/user/login
//! ```
//!
//! **Response**
//!
//! ```json
//! {
//!     "data":{
//!       "token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI",
//!       "username":"indexadmin",
//!       "admin":true
//!     }
//!   }
//! ```
//!
//! **NOTICE**: The token is valid for 2 weeks (`1_209_600` seconds). After that,
//! you will have to renew the token.
//!
//! **NOTICE**: The token is associated with the user role. If you change the
//! user's role, you will have to log in again to get a new token with the new
//! role.
//!
//! ## Using the token
//!
//! Some endpoints require authentication. To use the token, you must add the
//! `Authorization` header to your request. For example, if you want to add a
//! new category, you must do the following:
//!
//! ```bash
//! curl \
//!   --header "Content-Type: application/json" \
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJ1c2VyIjp7InVzZXJfaWQiOjEsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImFkbWluaXN0cmF0b3IiOnRydWV9LCJleHAiOjE2ODYyMTU3ODh9.4k8ty27DiWwOk4WVcYEhIrAndhpXMRWnLZ3i_HlJnvI" \
//!   --request POST \
//!   --data '{"name":"new category","icon":null}' \
//!   http://127.0.0.1:3000/v1/category
//! ```
//!
//! **Response**
//!
//! ```json
//! {
//!   "data": "new category"
//! }
//! ```
use std::sync::Arc;

use hyper::http::HeaderValue;

use crate::common::AppData;
use crate::errors::ServiceError;
use crate::models::user::UserId;
use crate::web::api::v1::extractors::bearer_token::BearerToken;

/// Parses the token from the `Authorization` header.
pub fn parse_token(authorization: &HeaderValue) -> String {
    let split: Vec<&str> = authorization
        .to_str()
        .expect("variable `auth` contains data that is not visible ASCII chars.")
        .split("Bearer")
        .collect();
    let token = split[1].trim();
    token.to_string()
}

/// If the user is logged in, returns the user's ID. Otherwise, returns `None`.
///
/// # Errors
///
/// It returns an error if we cannot get the user from the bearer token.
pub async fn get_optional_logged_in_user(
    maybe_bearer_token: Option<BearerToken>,
    app_data: Arc<AppData>,
) -> Result<Option<UserId>, ServiceError> {
    match maybe_bearer_token {
        Some(bearer_token) => match app_data.auth.get_user_id_from_bearer_token(&Some(bearer_token)).await {
            Ok(user_id) => Ok(Some(user_id)),
            Err(error) => Err(error),
        },
        None => Ok(None),
    }
}
