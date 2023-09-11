use serde::{Deserialize, Serialize};

use super::torrent_tag::TagId;
use crate::errors::ServiceError;

#[allow(clippy::module_name_repetitions)]
pub type TorrentId = i64;

#[allow(clippy::module_name_repetitions)]
#[allow(dead_code)]
#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, sqlx::FromRow)]
pub struct TorrentListing {
    pub torrent_id: TorrentId,
    pub uploader: String,
    pub info_hash: String,
    pub title: String,
    pub description: Option<String>,
    pub category_id: Option<i64>,
    pub date_uploaded: String,
    pub file_size: i64,
    pub seeders: i64,
    pub leechers: i64,
    pub name: String,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<TagId>,
}

impl Metadata {
    /// Returns the verify of this [`Metadata`].
    ///
    /// # Errors
    ///
    /// This function will return an error if the any of the mandatory metadata fields are missing.
    pub fn verify(&self) -> Result<(), ServiceError> {
        if self.title.is_empty() || self.category.is_empty() {
            Err(ServiceError::MissingMandatoryMetadataFields)
        } else {
            Ok(())
        }
    }
}
