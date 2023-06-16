use axum::response::{IntoResponse, Response};
use axum::Json;
use hyper::{header, StatusCode};
use serde::{Deserialize, Serialize};

use crate::models::torrent::TorrentId;
use crate::web::api::v1::responses::OkResponseData;

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct NewTorrentResponseData {
    pub torrent_id: TorrentId,
    pub info_hash: String,
}

/// Response after successfully uploading a new torrent.
pub fn new_torrent_response(torrent_id: TorrentId, info_hash: &str) -> Json<OkResponseData<NewTorrentResponseData>> {
    Json(OkResponseData {
        data: NewTorrentResponseData {
            torrent_id,
            info_hash: info_hash.to_owned(),
        },
    })
}

#[must_use]
pub fn torrent_file_response(bytes: Vec<u8>) -> Response {
    (StatusCode::OK, [(header::CONTENT_TYPE, "application/x-bittorrent")], bytes).into_response()
}
