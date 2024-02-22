use std::sync::Arc;

use async_trait::async_trait;
use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::Response;

use crate::common::AppData;
use crate::models::user::UserId;
use crate::web::api::server::v1::extractors::bearer_token;

pub struct ExtractOptionalLoggedInUser(pub Option<UserId>);

#[async_trait]
impl<S> FromRequestParts<S> for ExtractOptionalLoggedInUser
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        /* let maybe_bearer_token = match bearer_token::Extract::from_request_parts(parts, state).await {
            Ok(maybe_bearer_token) => maybe_bearer_token.0,
            Err(_) => return Err(ServiceError::TokenNotFound.into_response()),
        }; */

        let bearer_token = match bearer_token::Extract::from_request_parts(parts, state).await {
            Ok(bearer_token) => bearer_token.0,
            Err(_) => None,
        };

        //Extracts the app state
        let app_data = Arc::from_ref(state);

        match app_data.auth.get_user_id_from_bearer_token(&bearer_token).await {
            Ok(user_id) => Ok(ExtractOptionalLoggedInUser(Some(user_id))),
            Err(_) => Ok(ExtractOptionalLoggedInUser(None)),
        }
    }
}
