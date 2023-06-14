use axum::Json;
use serde::{Deserialize, Serialize};

use crate::models::user::{UserCompact, UserId};
use crate::web::api::v1::responses::OkResponse;

// Registration

#[derive(Serialize, Deserialize, Debug)]
pub struct NewUser {
    pub user_id: UserId,
}

/// Response after successfully creating a new user.
pub fn added_user(user_id: i64) -> Json<OkResponse<NewUser>> {
    Json(OkResponse {
        data: NewUser { user_id },
    })
}

// Authentication

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub token: String,
    pub username: String,
    pub admin: bool,
}

/// Response after successfully logging in a user.
pub fn logged_in_user(token: String, user_compact: UserCompact) -> Json<OkResponse<TokenResponse>> {
    Json(OkResponse {
        data: TokenResponse {
            token,
            username: user_compact.username,
            admin: user_compact.administrator,
        },
    })
}

/// Response after successfully renewing a JWT.
pub fn renewed_token(token: String, user_compact: UserCompact) -> Json<OkResponse<TokenResponse>> {
    Json(OkResponse {
        data: TokenResponse {
            token,
            username: user_compact.username,
            admin: user_compact.administrator,
        },
    })
}
