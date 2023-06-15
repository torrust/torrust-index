//! Generic responses for the API.
use axum::response::{IntoResponse, Response};
use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};
use serde_json::json;

use crate::databases::database;
use crate::errors::{http_status_code_for_service_error, map_database_error_to_service_error, ServiceError};

#[derive(Serialize, Deserialize, Debug)]
pub struct OkResponseData<T> {
    pub data: T,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponseData {
    pub error: String,
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        json_error_response(
            http_status_code_for_service_error(&self),
            &ErrorResponseData { error: self.to_string() },
        )
    }
}

impl IntoResponse for database::Error {
    fn into_response(self) -> Response {
        let service_error = map_database_error_to_service_error(&self);

        json_error_response(
            http_status_code_for_service_error(&service_error),
            &ErrorResponseData {
                error: service_error.to_string(),
            },
        )
    }
}

fn json_error_response(status_code: StatusCode, error_response_data: &ErrorResponseData) -> Response {
    (
        status_code,
        [(header::CONTENT_TYPE, "application/json")],
        json!(error_response_data).to_string(),
    )
        .into_response()
}
