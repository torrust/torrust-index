//! Generic responses for the API.
use axum::response::{IntoResponse, Response};
use serde::{Deserialize, Serialize};

use crate::databases::database;
use crate::errors::{http_status_code_for_service_error, map_database_error_to_service_error, ServiceError};

#[derive(Serialize, Deserialize, Debug)]
pub struct OkResponse<T> {
    pub data: T,
}

impl IntoResponse for database::Error {
    fn into_response(self) -> Response {
        let service_error = map_database_error_to_service_error(&self);

        (http_status_code_for_service_error(&service_error), service_error.to_string()).into_response()
    }
}

impl IntoResponse for ServiceError {
    fn into_response(self) -> Response {
        (http_status_code_for_service_error(&self), self.to_string()).into_response()
    }
}
