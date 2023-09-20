use derive_more::{Display, Error};
use serde::{Deserialize, Serialize};

use super::torrent_tag::TagId;

const MIN_TORRENT_TITLE_LENGTH: usize = 3;

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
    pub comment: Option<String>,
}

#[derive(Debug, Display, PartialEq, Eq, Error)]
pub enum MetadataError {
    #[display(fmt = "Missing mandatory torrent title")]
    MissingTorrentTitle,

    #[display(fmt = "Missing mandatory torrent category name")]
    MissingTorrentCategoryName,

    #[display(fmt = "Torrent title is too short.")]
    InvalidTorrentTitleLength,
}

#[derive(Debug, Deserialize)]
pub struct Metadata {
    pub title: String,
    pub description: String,
    pub category: String,
    pub tags: Vec<TagId>,
}

impl Metadata {
    /// Create a new struct.
    ///
    /// # Errors
    ///
    /// This function will return an error if the metadata fields do not have a
    /// valid format.
    pub fn new(title: &str, description: &str, category: &str, tag_ids: &[TagId]) -> Result<Self, MetadataError> {
        Self::validate_format(title, description, category, tag_ids)?;

        Ok(Self {
            title: title.to_owned(),
            description: description.to_owned(),
            category: category.to_owned(),
            tags: tag_ids.to_vec(),
        })
    }

    /// It validates the format of the metadata fields.
    ///
    /// It does not validate domain rules, like:
    ///
    /// - Duplicate titles.
    /// - Non-existing categories.
    /// - ...
    ///
    /// # Errors
    ///
    /// This function will return an error if any of the metadata fields does
    /// not have a valid format.
    fn validate_format(title: &str, _description: &str, category: &str, _tag_ids: &[TagId]) -> Result<(), MetadataError> {
        if title.is_empty() {
            return Err(MetadataError::MissingTorrentTitle);
        }

        if category.is_empty() {
            return Err(MetadataError::MissingTorrentCategoryName);
        }

        if title.len() < MIN_TORRENT_TITLE_LENGTH {
            return Err(MetadataError::InvalidTorrentTitleLength);
        }

        Ok(())
    }
}
