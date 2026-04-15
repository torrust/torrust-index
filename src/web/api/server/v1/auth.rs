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
//! **NOTICE**: The first registered user is automatically granted the
//! `admin` role. You can grant the admin role to additional users by
//! setting the `role` column in the `torrust_users` table to `'admin'`.
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
//!       "token":"eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6ImExYjJjM2Q0ZTVmNmE3YjgifQ.eyJzdWIiOjEsImlzcyI6InRvcnJ1c3QtaW5kZXgiLCJhdWQiOiJzZXNzaW9uIiwiaWF0IjoxNjg2MjE1Nzg4LCJleHAiOjE2ODc0MjUzODgsInJvbGUiOiJhZG1pbiIsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImdlbiI6MH0.RS256-SIGNATURE",
//!       "username":"indexadmin",
//!       "role":"admin"
//!     }
//!   }
//! ```
//!
//! The JWT is signed with RS256 (RSA + SHA-256). The payload contains
//! RFC 7519 registered claims plus advisory fields:
//!
//! ```json
//! {
//!   "sub": 1,
//!   "iss": "torrust-index",
//!   "aud": "session",
//!   "iat": 1686215788,
//!   "exp": 1687425388,
//!   "role": "admin",
//!   "username": "indexadmin",
//!   "gen": 0
//! }
//! ```
//!
//! The `role` and `username` fields are **advisory only** — the
//! authoritative role is always re-checked from the database on each
//! authenticated request (see ADR-T-007 Phase 2).
//!
//! The `gen` field is the token-generation counter. When a user's
//! password changes, role changes, or the user is banned, the counter
//! is incremented and any token carrying an older `gen` value is
//! rejected. Validation is performed by
//! [`JsonWebToken::validate_session`](crate::jwt::JsonWebToken::validate_session)
//! (see ADR-T-007 Phases 4 & 7).
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
//!   --header "Authorization: Bearer eyJ0eXAiOiJKV1QiLCJhbGciOiJSUzI1NiIsImtpZCI6ImExYjJjM2Q0ZTVmNmE3YjgifQ.eyJzdWIiOjEsImlzcyI6InRvcnJ1c3QtaW5kZXgiLCJhdWQiOiJzZXNzaW9uIiwiaWF0IjoxNjg2MjE1Nzg4LCJleHAiOjE2ODc0MjUzODgsInJvbGUiOiJhZG1pbiIsInVzZXJuYW1lIjoiaW5kZXhhZG1pbiIsImdlbiI6MH0.RS256-SIGNATURE" \
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

use crate::databases::database::Database;
use crate::errors::AuthError;
use crate::jwt::{JsonWebToken, SessionClaims};
use crate::models::user::{UserCompact, UserId};
use crate::web::api::server::v1::extractors::bearer_token::BearerToken;

pub struct Authentication {
    json_web_token: Arc<JsonWebToken>,
    database: Arc<Box<dyn Database>>,
}

impl Authentication {
    #[must_use]
    pub fn new(json_web_token: Arc<JsonWebToken>, database: Arc<Box<dyn Database>>) -> Self {
        Self {
            json_web_token,
            database,
        }
    }

    /// Create Json Web Token
    ///
    /// # Errors
    ///
    /// Returns `AuthError::InternalServerError` if the token cannot be encoded.
    pub async fn sign_jwt(&self, user: UserCompact, token_generation: u64) -> Result<String, AuthError> {
        self.json_web_token.sign(user, token_generation).await
    }

    /// Verify Json Web Token
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is not good or expired.
    pub fn verify_jwt(&self, token: &str) -> Result<SessionClaims, AuthError> {
        self.json_web_token.verify(token)
    }

    /// Get logged-in user ID from bearer token, validating the token
    /// generation counter against the database.
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is invalid, expired,
    /// or if the token's generation has been revoked.
    pub async fn get_user_id_from_bearer_token(&self, token: BearerToken) -> Result<UserId, AuthError> {
        let claims = self.json_web_token.validate_session(&**self.database, token.as_str()).await?;
        Ok(claims.sub)
    }
}

/// Parses the token from the `Authorization` header.
///
/// # Errors
///
/// Returns `AuthError::TokenInvalid` if the header value is not valid
/// ASCII or does not contain a `Bearer <token>` pair. The scheme name
/// is matched case-insensitively per RFC 7235 §2.1.
pub fn parse_token(authorization: &HeaderValue) -> Result<String, AuthError> {
    let header_str = authorization.to_str().map_err(|_| AuthError::TokenInvalid)?;

    let token = header_str
        .get(7..)
        .filter(|_| header_str[..7].eq_ignore_ascii_case("bearer "))
        .ok_or(AuthError::TokenInvalid)?
        .trim();

    if token.is_empty() {
        return Err(AuthError::TokenInvalid);
    }

    Ok(token.to_string())
}
