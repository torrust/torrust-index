use serde::Deserialize;

use crate::models::category::CategoryId;
use crate::models::torrent_tag::TagId;

#[derive(Debug, Deserialize)]
pub struct UpdateTorrentInfoForm {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<CategoryId>,
    pub tags: Option<Vec<TagId>>,
}
