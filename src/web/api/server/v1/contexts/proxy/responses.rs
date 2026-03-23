use axum::response::{IntoResponse, Response};
use bytes::Bytes;
use hyper::{StatusCode, header};

#[must_use]
pub fn png_image(bytes: Bytes) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, "image/png")], bytes).into_response()
}
