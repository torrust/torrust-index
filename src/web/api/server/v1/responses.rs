//! Generic responses for the API.
use axum::response::{IntoResponse, Response};
use hyper::{StatusCode, header};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::errors::{ApiError, AuthError, CategoryTagError, TorrentError, UserError};

#[derive(Serialize, Deserialize, Debug)]
pub struct OkResponseData<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponseData {
    pub error: String,
}

impl IntoResponse for AuthError {
    fn into_response(self) -> Response {
        json_error_response(self.status_code(), &ErrorResponseData { error: self.to_string() })
    }
}

impl IntoResponse for UserError {
    fn into_response(self) -> Response {
        json_error_response(self.status_code(), &ErrorResponseData { error: self.to_string() })
    }
}

impl IntoResponse for TorrentError {
    fn into_response(self) -> Response {
        json_error_response(self.status_code(), &ErrorResponseData { error: self.to_string() })
    }
}

impl IntoResponse for CategoryTagError {
    fn into_response(self) -> Response {
        json_error_response(self.status_code(), &ErrorResponseData { error: self.to_string() })
    }
}

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        json_error_response(self.status_code(), &ErrorResponseData { error: self.to_string() })
    }
}

#[must_use]
pub fn json_error_response(status_code: StatusCode, error_response_data: &ErrorResponseData) -> Response {
    (
        status_code,
        [(header::CONTENT_TYPE, "application/json")],
        json!(error_response_data).to_string(),
    )
        .into_response()
}
