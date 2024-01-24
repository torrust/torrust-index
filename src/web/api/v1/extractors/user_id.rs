use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};
use serde::Deserialize;

use super::bearer_token;
use crate::common::AppData;
use crate::errors::ServiceError;

pub struct ExtractLoggedInUser(pub UserId);

#[derive(Deserialize, Debug)]
pub struct UserId(i64);

impl UserId {
    #[must_use]
    pub fn value(self) -> i64 {
        self.0
    }
}

#[async_trait]
impl<S> FromRequestParts<S> for ExtractLoggedInUser
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let maybe_bearer_token = match bearer_token::Extract::from_request_parts(parts, state).await {
            Ok(maybe_bearer_token) => maybe_bearer_token.0,
            Err(_) => return Err(ServiceError::Unauthorized.into_response()),
        };

        /*   let app_data = match axum::extract::State::from_request_parts(parts, state).await {
            Ok(app_data) => Ok(app_data.0),
            Err(_) => Err(ServiceError::Unauthorized),
        }; */

        let app_data = Arc::from_ref(state);

        /* match app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await {
            Ok(user_id) => ExtractLoggedInUser(UserId(user_id)),
            Error(err) => return ServiceError::Unauthorized.into_response(),
        } */

        /*    let user_id =  ExtractLoggedInUser(UserId(app_data
        .auth
        .get_user_id_from_bearer_token(&maybe_bearer_token))
        .await
        .map_err(|e| {
            dbg!(e);
            ServiceError::Unauthorized
        })? */
        let user_id = app_data.auth.get_user_id_from_bearer_token(&maybe_bearer_token).await;

        match user_id {
            Ok(user_id) => Ok(ExtractLoggedInUser(UserId(user_id))),
            Err(error) => Err(error.into_response()),
        }

        /* match header {
            Some(header_value) => Ok(Extract(Some(BearerToken(parse_token(header_value))))),
            None => Ok(Extract(None)),
        } */
    }
}
