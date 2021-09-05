use serde::{Deserialize, Serialize};

#[allow(dead_code)]
#[derive(Debug, Serialize)]
pub struct TorrentListing {
    pub torrent_id: i64,
    pub uploader_id: Option<i64>,
    pub title: String,
    pub description: Option<String>,
    pub category_id: i64,
}