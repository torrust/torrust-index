use std::sync::Arc;

use actix_web::HttpRequest;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::config::Configuration;
use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::user::{UserClaims, UserCompact};
use crate::utils::clock::current_time;

pub struct AuthorizationService {
    cfg: Arc<Configuration>,
    database: Arc<Box<dyn Database>>,
}

impl AuthorizationService {
    pub fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> AuthorizationService {
        AuthorizationService { cfg, database }
    }

    /// Create Json Web Token
    pub async fn sign_jwt(&self, user: UserCompact) -> String {
        let settings = self.cfg.settings.read().await;

        // create JWT that expires in two weeks
        let key = settings.auth.secret_key.as_bytes();
        // TODO: create config option for setting the token validity in seconds
        let exp_date = current_time() + 1_209_600; // two weeks from now

        let claims = UserClaims { user, exp: exp_date };

        encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).expect("argument `Header` should match `EncodingKey`")
    }

    /// Verify Json Web Token
    ///
    /// # Errors
    ///
    /// This function will return an error if the JWT is not good or expired.
    pub async fn verify_jwt(&self, token: &str) -> Result<UserClaims, ServiceError> {
        let settings = self.cfg.settings.read().await;

        match decode::<UserClaims>(
            token,
            &DecodingKey::from_secret(settings.auth.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if token_data.claims.exp < current_time() {
                    return Err(ServiceError::TokenExpired);
                }
                Ok(token_data.claims)
            }
            Err(_) => Err(ServiceError::TokenInvalid),
        }
    }

    /// Get Claims from Request
    ///
    /// # Errors
    ///
    /// This function will return an `ServiceError::TokenNotFound` if `HeaderValue` is `None`
    /// This function will pass through the `ServiceError::TokenInvalid` if unable to verify the JWT.
    pub async fn get_claims_from_request(&self, req: &HttpRequest) -> Result<UserClaims, ServiceError> {
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

    /// Get User (in compact form) from Request
    ///
    /// # Errors
    ///
    /// This function will return an `ServiceError::UserNotFound` if unable to get user from database.
    pub async fn get_user_compact_from_request(&self, req: &HttpRequest) -> Result<UserCompact, ServiceError> {
        let claims = self.get_claims_from_request(req).await?;

        self.database
            .get_user_compact_from_id(claims.user.user_id)
            .await
            .map_err(|_| ServiceError::UserNotFound)
    }
}
