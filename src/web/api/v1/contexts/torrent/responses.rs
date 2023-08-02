use axum::response::{IntoResponse, Response};
use axum::Json;
use hyper::{header, HeaderMap, StatusCode};
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

/// Builds the binary response for a torrent file.
///
/// # Panics
///
/// Panics if the filename is not a valid header value for the `content-disposition`
/// header.
#[must_use]
pub fn torrent_file_response(bytes: Vec<u8>, filename: &str, info_hash: &str) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(
        header::CONTENT_TYPE,
        "application/x-bittorrent"
            .parse()
            .expect("HTTP content type header should be valid"),
    );
    headers.insert(
        header::CONTENT_DISPOSITION,
        format!("attachment; filename={filename}")
            .parse()
            .expect("Torrent filename should be a valid header value for the content disposition header"),
    );
    headers.insert(
        "x-torrust-torrent-infohash",
        info_hash
            .parse()
            .expect("Torrent infohash should be a valid header value for the content disposition header"),
    );

    (StatusCode::OK, headers, bytes).into_response()
}
