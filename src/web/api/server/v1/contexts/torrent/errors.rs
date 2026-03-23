use axum::response::{IntoResponse, Response};
use derive_more::{Display, Error};
use hyper::StatusCode;

use crate::web::api::server::v1::responses::{ErrorResponseData, json_error_response};

#[derive(Debug, Display, PartialEq, Eq, Error)]
pub enum Request {
    #[display("torrent title bytes are nota valid UTF8 string.")]
    TitleIsNotValidUtf8,

    #[display("torrent description bytes are nota valid UTF8 string.")]
    DescriptionIsNotValidUtf8,

    #[display("torrent category bytes are nota valid UTF8 string.")]
    CategoryIsNotValidUtf8,

    #[display("torrent tags arrays bytes are nota valid UTF8 string array.")]
    TagsArrayIsNotValidUtf8,

    #[display("torrent tags string is not a valid JSON.")]
    TagsArrayIsNotValidJson,

    #[display(
        "upload torrent request header `content-type` should be preferably `application/x-bittorrent` or `application/octet-stream`."
    )]
    InvalidFileType,

    #[display("cannot write uploaded torrent bytes (binary file) into memory.")]
    CannotWriteChunkFromUploadedBinary,

    #[display("cannot read a chunk of bytes from the uploaded torrent file. Review the request body size limit.")]
    CannotReadChunkFromUploadedBinary,

    #[display("provided path param for Info-hash is not valid.")]
    InvalidInfoHashParam,
}

impl IntoResponse for Request {
    fn into_response(self) -> Response {
        json_error_response(
            http_status_code_for_handler_error(&self),
            &ErrorResponseData { error: self.to_string() },
        )
    }
}

#[must_use]
pub const fn http_status_code_for_handler_error(error: &Request) -> StatusCode {
    #[allow(clippy::match_same_arms)]
    match error {
        Request::TitleIsNotValidUtf8 => StatusCode::BAD_REQUEST,
        Request::DescriptionIsNotValidUtf8 => StatusCode::BAD_REQUEST,
        Request::CategoryIsNotValidUtf8 => StatusCode::BAD_REQUEST,
        Request::TagsArrayIsNotValidUtf8 => StatusCode::BAD_REQUEST,
        Request::TagsArrayIsNotValidJson => StatusCode::BAD_REQUEST,
        Request::InvalidFileType => StatusCode::BAD_REQUEST,
        Request::InvalidInfoHashParam => StatusCode::BAD_REQUEST,
        // Internal errors processing the request
        Request::CannotWriteChunkFromUploadedBinary => StatusCode::INTERNAL_SERVER_ERROR,
        Request::CannotReadChunkFromUploadedBinary => StatusCode::INTERNAL_SERVER_ERROR,
    }
}
