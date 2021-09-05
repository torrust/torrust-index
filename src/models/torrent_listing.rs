use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TorrentListing {
    pub torrent_id: i64,
    pub uploader_id: i64,
    pub info_hash: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub category_id: i64,
}
