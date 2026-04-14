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
//!   http://127.0.0.1:3001/v1/user/register
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
//!   http://127.0.0.1:3001/v1/user/login
//! ```
//!
//! **Response**
//!
//! ```json
//! {
//!     "data":{
//!       "token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjEsImlzcyI6InRvcnJ1c3QtaW5kZXgiLCJhdWQiOiJzZXNzaW9uIiwiaWF0IjoxNjg2MjE1Nzg4LCJleHAiOjE2ODc0MjUzODgsInJvbGUiOiJhZG1pbiIsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiJ9.-EfY9CrZz2OLfjiVQzkhxSjV7tWTFivP2yMuzZkbEak",
//!       "username":"indexadmin",
//!       "admin":true
//!     }
//!   }
//! ```
//!
//! The JWT payload contains RFC 7519 registered claims:
//!
//! ```json
//! {
//!   "sub": 1,
//!   "iss": "torrust-index",
//!   "aud": "session",
//!   "iat": 1686215788,
//!   "exp": 1687425388,
//!   "role": "admin",
//!   "username": "indexadmin"
//! }
//! ```
//!
//! The `role` and `username` fields are **advisory only** — the
//! authoritative role is always re-checked from the database on each
//! authenticated request (see ADR-T-007 Phase 2).
//!
//! **NOTICE**: The token lifetime is configurable via
//! `auth.session_token_lifetime_secs` (default: 2 weeks / `1_209_600` seconds).
//! After expiry you will have to renew the token.
//!
//! **NOTICE**: The token `role` is advisory. If the user's role changes in
//! the database, the new role takes effect immediately on the next request.
//! However, you may still want to log in again to get a token that reflects
//! the current role.
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
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9.eyJzdWIiOjEsImlzcyI6InRvcnJ1c3QtaW5kZXgiLCJhdWQiOiJzZXNzaW9uIiwiaWF0IjoxNjg2MjE1Nzg4LCJleHAiOjE2ODc0MjUzODgsInJvbGUiOiJhZG1pbiIsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiJ9.-EfY9CrZz2OLfjiVQzkhxSjV7tWTFivP2yMuzZkbEak" \
//!   --request POST \
//!   --data '{"name":"new category","icon":null}' \
//!   http://127.0.0.1:3001/v1/category
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
use crate::errors::AuthError;
use crate::jwt::{JsonWebToken, SessionClaims};
use crate::models::user::{UserCompact, UserId};
use crate::web::api::server::v1::extractors::bearer_token::BearerToken;

pub struct Authentication {
    json_web_token: Arc<JsonWebToken>,
}

impl Authentication {
    #[must_use]
    pub const fn new(json_web_token: Arc<JsonWebToken>) -> Self {
        Self { json_web_token }
    }

    /// Create Json Web Token
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if the token cannot be encoded.
    pub async fn sign_jwt(&self, user: UserCompact) -> Result<String, AuthError> {
        self.json_web_token.sign(user).await
    }

    /// Verify Json Web Token
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is not good or expired.
    pub async fn verify_jwt(&self, token: &str) -> Result<SessionClaims, AuthError> {
        self.json_web_token.verify(token).await
    }

    /// Get logged-in user ID from bearer token
    ///
    /// # Errors
    ///
    /// This function will return an error if it can get claims from the request
    pub async fn get_user_id_from_bearer_token(&self, maybe_token: Option<BearerToken>) -> Result<UserId, AuthError> {
        let claims = self.get_claims_from_bearer_token(maybe_token).await?;
        Ok(claims.sub)
    }

    /// Get Claims from bearer token
    ///
    /// # Errors
    ///
    /// This function will:
    ///
    /// - Return an `AuthError::TokenNotFound` if `HeaderValue` is `None`.
    /// - Pass through the `AuthError::TokenInvalid` if unable to verify the JWT.
    async fn get_claims_from_bearer_token(&self, maybe_token: Option<BearerToken>) -> Result<SessionClaims, AuthError> {
        match maybe_token {
            Some(token) => match self.verify_jwt(&token.value()).await {
                Ok(claims) => Ok(claims),
                Err(e) => Err(e),
            },
            None => Err(AuthError::TokenNotFound),
        }
    }
}

/// Parses the token from the `Authorization` header.
///
/// # Errors
///
/// Returns `AuthError::TokenInvalid` if the header value is not valid
/// ASCII or does not contain a `Bearer <token>` pair.
pub fn parse_token(authorization: &HeaderValue) -> Result<String, AuthError> {
    let header_str = authorization.to_str().map_err(|_| AuthError::TokenInvalid)?;

    let token = header_str.strip_prefix("Bearer").ok_or(AuthError::TokenInvalid)?.trim();

    if token.is_empty() {
        return Err(AuthError::TokenInvalid);
    }

    Ok(token.to_string())
}

/// If the user is logged in, returns the user's ID. Otherwise, returns `None`.
///
/// # Errors
///
/// It returns an error if we cannot get the user from the bearer token.
pub async fn get_optional_logged_in_user(
    maybe_bearer_token: Option<BearerToken>,
    app_data: Arc<AppData>,
) -> Result<Option<UserId>, AuthError> {
    match maybe_bearer_token {
        Some(bearer_token) => match app_data.auth.get_user_id_from_bearer_token(Some(bearer_token)).await {
            Ok(user_id) => Ok(Some(user_id)),
            Err(error) => Err(error),
        },
        None => Ok(None),
    }
}
