use std::sync::Arc;

use axum::async_trait;
use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;
use tokio::sync::RwLock;

use super::bearer_token;
use crate::services;
use crate::web::api::v1::auth::{self, Authentication};

pub struct ExtractLoggedInUser(pub Option<UserId>);

#[derive(Deserialize, Debug)]
pub struct UserId(i64);

impl UserId {
    #[must_use]
    pub fn value(&self) -> i64 {
        self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ExtractLoggedInUser
where
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let maybe_bearer_token = match bearer_token::Extract::from_request_parts(parts, state).await {
            Ok(maybe_bearer_token) => maybe_bearer_token.0,
            Err(_) => None,
        };

        let bearer_token = services::authentication::JsonWebToken::new(Arc::new(crate::config::Configuration {
            settings: RwLock::default(),
            config_path: Option::default(),
        }));

        let auth: Authentication = auth::Authentication::new(Arc::new(bearer_token));

        match auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
            Ok(user_id) => Ok(ExtractLoggedInUser(Some(UserId(user_id)))),
            Err(error) => return Err(error.into_response()),
        }
    }
}
