use serde::{Deserialize, Serialize};

use super::torrent::TorrentId;
use crate::databases::database::Category;
use crate::models::torrent::TorrentListing;
use crate::models::torrent_file::TorrentFile;
use crate::models::torrent_tag::TorrentTag;

pub enum OkResponses {
    TokenResponse(TokenResponse),
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct OkResponse<T> {
    pub data: T,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct ErrorResponse<T> {
    pub errors: Vec<T>,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct TokenResponse {
    pub token: String,
    pub username: String,
    pub admin: bool,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct NewTorrentResponse {
    pub torrent_id: TorrentId,
    pub info_hash: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug)]
pub struct DeletedTorrentResponse {
    pub torrent_id: TorrentId,
    pub info_hash: String,
}

#[allow(clippy::module_name_repetitions)]
#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct TorrentResponse {
    pub torrent_id: i64,
    pub uploader: String,
    pub info_hash: String,
    pub title: String,
    pub description: Option<String>,
    pub category: Option<Category>,
    pub upload_date: String,
    pub file_size: i64,
    pub seeders: i64,
    pub leechers: i64,
    pub files: Vec<TorrentFile>,
    pub trackers: Vec<String>,
    pub magnet_link: String,
    pub tags: Vec<TorrentTag>,
    pub name: String,
}

impl TorrentResponse {
    #[must_use]
    pub fn from_listing(torrent_listing: TorrentListing, category: Option<Category>) -> TorrentResponse {
        TorrentResponse {
            torrent_id: torrent_listing.torrent_id,
            uploader: torrent_listing.uploader,
            info_hash: torrent_listing.info_hash,
            title: torrent_listing.title,
            description: torrent_listing.description,
            category,
            upload_date: torrent_listing.date_uploaded,
            file_size: torrent_listing.file_size,
            seeders: torrent_listing.seeders,
            leechers: torrent_listing.leechers,
            files: vec![],
            trackers: vec![],
            magnet_link: String::new(),
            tags: vec![],
            name: torrent_listing.name,
        }
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct TorrentsResponse {
    pub total: u32,
    pub results: Vec<TorrentListing>,
}
