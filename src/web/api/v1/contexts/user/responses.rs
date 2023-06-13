use axum::Json;
use serde::{Deserialize, Serialize};

use crate::models::user::UserId;
use crate::web::api::v1::responses::OkResponse;

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
