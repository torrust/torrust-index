use std::sync::Arc;

use actix_web::HttpRequest;

use crate::errors::ServiceError;
use crate::models::user::{UserClaims, UserCompact, UserId};
use crate::services::authentication::JsonWebToken;
use crate::web::api::v1::extractors::bearer_token::BearerToken;

// todo: refactor this after finishing migration to Axum.
// - Extract service to handle Json Web Tokens: `new`, `sign_jwt`, `verify_jwt`.
// - Move the rest to `src/web/api/v1/auth.rs`. It's a helper for Axum handlers
//  to get user id from request.

pub struct Authentication {
    json_web_token: Arc<JsonWebToken>,
}

impl Authentication {
    #[must_use]
    pub fn new(json_web_token: Arc<JsonWebToken>) -> Self {
        Self { json_web_token }
    }

    /// Create Json Web Token
    pub async fn sign_jwt(&self, user: UserCompact) -> String {
        self.json_web_token.sign(user).await
    }

    /// Verify Json Web Token
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is not good or expired.
    pub async fn verify_jwt(&self, token: &str) -> Result<UserClaims, ServiceError> {
        self.json_web_token.verify(token).await
    }

    // Begin ActixWeb

    /// Get User id from `ActixWeb` Request
    ///
    /// # Errors
    ///
    /// This function will return an error if it can get claims from the request
    pub async fn get_user_id_from_actix_web_request(&self, req: &HttpRequest) -> Result<UserId, ServiceError> {
        let claims = self.get_claims_from_actix_web_request(req).await?;
        Ok(claims.user.user_id)
    }

    /// Get Claims from `ActixWeb` Request
    ///
    /// # Errors
    ///
    /// - Return an `ServiceError::TokenNotFound` if `HeaderValue` is `None`.
    /// - Pass through the `ServiceError::TokenInvalid` if unable to verify the JWT.
    async fn get_claims_from_actix_web_request(&self, req: &HttpRequest) -> Result<UserClaims, ServiceError> {
        match req.headers().get("Authorization") {
            Some(auth) => {
                let split: Vec<&str> = auth
                    .to_str()
                    .expect("variable `auth` contains data that is not visible ASCII chars.")
                    .split("Bearer")
                    .collect();
                let token = split[1].trim();

                match self.verify_jwt(token).await {
                    Ok(claims) => Ok(claims),
                    Err(e) => Err(e),
                }
            }
            None => Err(ServiceError::TokenNotFound),
        }
    }

    // End ActixWeb

    // Begin Axum

    /// Get User id from bearer token
    ///
    /// # Errors
    ///
    /// This function will return an error if it can get claims from the request
    pub async fn get_user_id_from_bearer_token(&self, maybe_token: &Option<BearerToken>) -> Result<UserId, ServiceError> {
        let claims = self.get_claims_from_bearer_token(maybe_token).await?;
        Ok(claims.user.user_id)
    }

    /// Get Claims from bearer token
    ///
    /// # Errors
    ///
    /// This function will:
    ///
    /// - Return an `ServiceError::TokenNotFound` if `HeaderValue` is `None`.
    /// - Pass through the `ServiceError::TokenInvalid` if unable to verify the JWT.
    async fn get_claims_from_bearer_token(&self, maybe_token: &Option<BearerToken>) -> Result<UserClaims, ServiceError> {
        match maybe_token {
            Some(token) => match self.verify_jwt(&token.value()).await {
                Ok(claims) => Ok(claims),
                Err(e) => Err(e),
            },
            None => Err(ServiceError::TokenNotFound),
        }
    }

    // End Axum
}
