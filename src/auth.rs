use actix_web::HttpRequest;
use crate::models::user::{Claims, User};
use jsonwebtoken::{decode, DecodingKey, Validation, Algorithm, encode, Header, EncodingKey, TokenData};
use crate::utils::time::current_time;
use crate::errors::ServiceError;
use std::sync::Arc;
use crate::data::Database;
use jsonwebtoken::errors::Error;
use std::future::Future;
use crate::config::TorrustConfig;
use crate::models::tracker_key::TrackerKey;
use std::error;
use reqwest::Response;

pub struct AuthorizationService {
    cfg: Arc<TorrustConfig>,
    database: Arc<Database>,
}

impl AuthorizationService {
    pub fn new(cfg: Arc<TorrustConfig>, database: Arc<Database>) -> AuthorizationService {
        AuthorizationService {
            cfg,
            database
        }
    }

    pub fn sign_jwt(&self, user: User) -> String {
        // create JWT that expires in two weeks
        let key = self.cfg.auth.secret_key.as_bytes();
        let exp_date = current_time() + 1_209_600; // two weeks from now

        let claims = Claims {
            sub: user.username,
            exp: exp_date,
        };

        let token = encode(
            &Header::default(),
            &claims,
            &EncodingKey::from_secret(key),
        )
            .unwrap();

        token
    }

    pub fn verify_jwt(&self, token: &str) -> Result<Claims, ServiceError> {
        match decode::<Claims>(
            token,
            &DecodingKey::from_secret(self.cfg.auth.secret_key.as_bytes()),
            &Validation::new(Algorithm::HS256),
        ) {
            Ok(token_data) => {
                if token_data.claims.exp < current_time() {
                    return Err(ServiceError::TokenExpired)
                }
                Ok(token_data.claims)
            },
            Err(e) => Err(ServiceError::TokenInvalid)
        }
    }

    pub fn get_claims_from_request(&self, req: &HttpRequest) -> Result<Claims, ServiceError> {
        let _auth = req.headers().get("Authorization");
        match _auth {
            Some(_) => {
                let _split: Vec<&str> = _auth.unwrap().to_str().unwrap().split("Bearer").collect();
                let token = _split[1].trim();

                match self.verify_jwt(token) {
                    Ok(claims) => Ok(claims),
                    Err(e) => Err(e),
                }
            }
            None => Err(ServiceError::TokenNotFound)
        }
    }

    pub async fn get_user_from_request(&self, req: &HttpRequest) -> Result<User, ServiceError> {
        let claims = match self.get_claims_from_request(req) {
            Ok(claims) => Ok(claims),
            Err(e) => Err(e)
        }?;

        match self.database.get_user_with_username(&claims.sub).await {
            Some(user) => Ok(user),
            None => Err(ServiceError::AccountNotFound)
        }
    }

    pub async fn get_personal_announce_url(&self, user: &User) -> Option<String> {
        let mut tracker_key = self.database.get_valid_tracker_key(user.user_id).await;

        if tracker_key.is_none() {
            match self.retrieve_new_tracker_key(user.user_id).await {
                Ok(v) => { tracker_key = Some(v) },
                Err(_) => { return None }
            }
        }

        Some(format!("{}/{}", self.cfg.tracker.url, tracker_key.unwrap().key))
    }

    pub async fn retrieve_new_tracker_key(&self, user_id: i64) -> Result<TrackerKey, ServiceError> {
        let request_url =
            format!("{}/api/key/{}?token={}", self.cfg.tracker.api_url, self.cfg.tracker.token_valid_seconds, self.cfg.tracker.token);

        let client = reqwest::Client::new();
        let response = match client.post(request_url)
            .send()
            .await {
            Ok(v) => Ok(v),
            Err(_) => Err(ServiceError::InternalServerError)
        }?;

        let tracker_key: TrackerKey = match response.json::<TrackerKey>().await {
            Ok(v) => Ok(v),
            Err(_) => Err(ServiceError::InternalServerError)
        }?;

        println!("{:?}", tracker_key);

        self.database.issue_tracker_key(&tracker_key, user_id).await?;

        Ok(tracker_key)
    }
}
