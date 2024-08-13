use serde::{Deserialize, Serialize};
use url::Url;

use super::category::Category;
use super::torrent::TorrentId;
use crate::databases::database::Category as DatabaseCategory;
use crate::models::torrent::TorrentListing;
use crate::models::torrent_file::TorrentFile;
use crate::models::torrent_tag::TorrentTag;
use crate::services::torrent::CanonicalInfoHashGroup;

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
    pub comment: Option<String>,
    pub creation_date: Option<i64>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
    pub canonical_info_hash_group: Vec<String>,
}

impl TorrentResponse {
    #[must_use]
    pub fn from_listing(
        torrent_listing: TorrentListing,
        category: Option<DatabaseCategory>,
        canonical_info_hash_group: &CanonicalInfoHashGroup,
    ) -> TorrentResponse {
        TorrentResponse {
            torrent_id: torrent_listing.torrent_id,
            uploader: torrent_listing.uploader,
            info_hash: torrent_listing.info_hash,
            title: torrent_listing.title,
            description: torrent_listing.description,
            category: category.map(std::convert::Into::into),
            upload_date: torrent_listing.date_uploaded,
            file_size: torrent_listing.file_size,
            seeders: torrent_listing.seeders,
            leechers: torrent_listing.leechers,
            files: vec![],
            trackers: vec![],
            magnet_link: String::new(),
            tags: vec![],
            name: torrent_listing.name,
            comment: torrent_listing.comment,
            creation_date: torrent_listing.creation_date,
            created_by: torrent_listing.created_by,
            encoding: torrent_listing.encoding,
            canonical_info_hash_group: canonical_info_hash_group
                .original_info_hashes
                .iter()
                .map(super::info_hash::InfoHash::to_hex_string)
                .collect(),
        }
    }

    /// It adds the tracker URL in the first position of the tracker list.
    pub fn include_url_as_main_tracker(&mut self, tracker_url: &Url) {
        // Remove any existing instances of tracker_url
        self.trackers.retain(|tracker| *tracker != tracker_url.to_string());

        // Insert tracker_url at the first position
        self.trackers.insert(0, tracker_url.to_owned().to_string());
    }
}

#[allow(clippy::module_name_repetitions)]
#[derive(Serialize, Deserialize, Debug, sqlx::FromRow)]
pub struct TorrentsResponse {
    pub total: u32,
    pub results: Vec<TorrentListing>,
}
