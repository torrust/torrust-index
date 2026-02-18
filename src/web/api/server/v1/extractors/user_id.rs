use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use super::bearer_token;
use crate::common::AppData;
use crate::errors::ServiceError;
use crate::models::user::UserId;

pub struct ExtractLoggedInUser(pub UserId);

impl<S> FromRequestParts<S> for ExtractLoggedInUser
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let maybe_bearer_token = match bearer_token::Extract::from_request_parts(parts, state).await {
            Ok(maybe_bearer_token) => maybe_bearer_token.0,
            Err(_) => return Err(ServiceError::TokenNotFound.into_response()),
        };

        //Extracts the app state
        let app_data = Arc::from_ref(state);

        #[allow(clippy::option_if_let_else)]
        let result = match app_data.auth.get_user_id_from_bearer_token(maybe_bearer_token).await {
            Ok(user_id) => Ok(Self(user_id)),
            Err(_) => Err(ServiceError::LoggedInUserNotFound.into_response()),
        };

        result
    }
}
