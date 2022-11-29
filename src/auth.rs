use std::sync::Arc;

use actix_web::HttpRequest;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};

use crate::config::Configuration;
use crate::databases::database::Database;
use crate::errors::ServiceError;
use crate::models::user::{UserClaims, UserCompact};
use crate::utils::time::current_time;

pub struct AuthorizationService {
    cfg: Arc<Configuration>,
    database: Arc<Box<dyn Database>>,
}

impl AuthorizationService {
    pub fn new(cfg: Arc<Configuration>, database: Arc<Box<dyn Database>>) -> AuthorizationService {
        AuthorizationService { cfg, database }
    }

    pub async fn sign_jwt(&self, user: UserCompact) -> String {
        let settings = self.cfg.settings.read().await;

        // create JWT that expires in two weeks
        let key = settings.auth.secret_key.as_bytes();
        // TODO: create config option for setting the token validity in seconds
        let exp_date = current_time() + 1_209_600; // two weeks from now

        let claims = UserClaims { user, exp: exp_date };

        let token = encode(&Header::default(), &claims, &EncodingKey::from_secret(key)).unwrap();

        token
    }

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

    pub async fn get_claims_from_request(&self, req: &HttpRequest) -> Result<UserClaims, ServiceError> {
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[1].trim();

                match self.verify_jwt(token).await {
                    Ok(claims) => Ok(claims),
                    Err(e) => Err(e),
                }
            }
            None => Err(ServiceError::TokenNotFound),
        }
    }

    pub async fn get_user_compact_from_request(&self, req: &HttpRequest) -> Result<UserCompact, ServiceError> {
        let claims = self.get_claims_from_request(req).await?;

        self.database
            .get_user_compact_from_id(claims.user.user_id)
            .await
            .map_err(|_| ServiceError::UserNotFound)
    }
}
