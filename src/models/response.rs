use serde::{Deserialize, Serialize};
use std::any::Any;

pub enum OkResponses {
    TokenResponse(TokenResponse)
}

#[derive(Serialize, Deserialize, Debug)]
pub struct OkResponse<T> {
    pub data: T
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse<T> {
    pub errors: Vec<T>
}

#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub token: String
}
