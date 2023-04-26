use serde::{Deserialize, Serialize};

use crate::models::torrent_file::Torrent;
use crate::routes::torrent::Create;

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct TorrentListing {
    pub torrent_id: i64,
    pub uploader: String,
    pub info_hash: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: i64,
    pub date_uploaded: String,
    pub file_size: i64,
    pub seeders: i64,
    pub leechers: i64,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug)]
pub struct TorrentRequest {
    pub fields: Create,
    pub torrent: Torrent,
}
