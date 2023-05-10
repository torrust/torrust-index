use serde::Deserialize;

pub type Id = i64;
pub type CategoryId = i64;
pub type UtcDateTime = String; // %Y-%m-%d %H:%M:%S

#[derive(Deserialize, PartialEq, Debug)]
pub struct ErrorResponse {
    pub error: String,
}

#[derive(Deserialize)]
pub struct TorrentListResponse {
    pub data: TorrentList,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TorrentList {
    pub total: u32,
    pub results: Vec<ListItem>,
}

impl TorrentList {
    pub fn _contains(&self, torrent_id: Id) -> bool {
        self.results.iter().any(|item| item.torrent_id == torrent_id)
    }
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct ListItem {
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

#[derive(Deserialize, PartialEq, Debug)]
pub struct TorrentDetailsResponse {
    pub data: TorrentDetails,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct TorrentDetails {
    pub torrent_id: Id,
    pub uploader: String,
    pub info_hash: String,
    pub title: String,
    pub description: String,
    pub category: Category,
    pub upload_date: UtcDateTime,
    pub file_size: u64,
    pub seeders: u64,
    pub leechers: u64,
    pub files: Vec<File>,
    pub trackers: Vec<String>,
    pub magnet_link: String,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct Category {
    pub category_id: CategoryId,
    pub name: String,
    pub num_torrents: u64,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct File {
    pub path: Vec<String>,
    pub length: u64,
    pub md5sum: Option<String>,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct UploadedTorrentResponse {
    pub data: UploadedTorrent,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct UploadedTorrent {
    pub torrent_id: Id,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct DeletedTorrentResponse {
    pub data: DeletedTorrent,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct DeletedTorrent {
    pub torrent_id: Id,
}

#[derive(Deserialize, PartialEq, Debug)]
pub struct UpdatedTorrentResponse {
    pub data: UpdatedTorrent,
}

pub type UpdatedTorrent = TorrentDetails;
