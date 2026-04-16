use std::sync::Arc;

use axum::extract::{FromRef, FromRequestParts};
use axum::http::request::Parts;
use axum::response::{IntoResponse, Response};

use crate::common::AppData;
use crate::models::user::UserId;
use crate::web::api::server::v1::extractors::bearer_token::BearerToken;

pub struct ExtractLoggedInUser(pub UserId);

impl<S> FromRequestParts<S> for ExtractLoggedInUser
where
    Arc<AppData>: FromRef<S>,
    S: Send + Sync,
{
    type Rejection = Response;

    async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
        let token = BearerToken::from_request_parts(parts, state).await?;

        let app_data = Arc::from_ref(state);

        app_data
            .auth
            .get_user_id_from_bearer_token(token)
            .await
            .map(Self)
            .map_err(IntoResponse::into_response)
    }
}
