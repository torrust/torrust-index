use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::errors::AuthError;
use crate::web::api::server::v1::auth::parse_token;

/// A validated bearer token extracted from the `Authorization` header.
///
/// Implements [`FromRequestParts`] and **rejects** the request when
/// the header is missing (`AuthError::TokenNotFound`) or malformed
/// (`AuthError::TokenInvalid`).
///
/// For endpoints where authentication is optional, use
/// `Option<BearerToken>` — Axum will yield `None` when extraction
/// fails instead of rejecting the request.
///
/// This addresses Problem #11 from ADR-T-007: the extractor now
/// catches missing/invalid tokens at the boundary rather than
/// deferring the check downstream.
#[derive(Debug)]
pub struct BearerToken(String);

impl BearerToken {
    #[must_use]
    pub fn value(&self) -> String {
        self.0.clone()
    }
}

impl<S> FromRequestParts<S> for BearerToken
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
        let header_value = parts
            .headers
            .get("Authorization")
            .ok_or_else(|| AuthError::TokenNotFound.into_response())?;

        let token = parse_token(header_value).map_err(IntoResponse::into_response)?;

        Ok(Self(token))
    }
}
