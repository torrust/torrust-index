use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use tempfile::{tempdir, TempDir};
use uuid::Uuid;

use super::file::{create_torrent, parse_torrent, TorrentFileInfo};
use super::requests::{BinaryFile, UploadTorrentMultipartForm};
use super::responses::{Id, UploadedTorrentResponse};
use crate::e2e::contexts::category::fixtures::software_predefined_category_name;
use crate::e2e::contexts::user::LoggedInUserData;
use crate::e2e::environment::TestEnv;

/// Information about a torrent that is going to added to the index.
#[derive(Clone)]
pub struct TorrentIndexInfo {
    pub title: String,
    pub description: String,
    pub category: String,
    pub torrent_file: BinaryFile,
}

impl From<TorrentIndexInfo> for UploadTorrentMultipartForm {
    fn from(indexed_torrent: TorrentIndexInfo) -> UploadTorrentMultipartForm {
        UploadTorrentMultipartForm {
            title: indexed_torrent.title,
            description: indexed_torrent.description,
            category: indexed_torrent.category,
            torrent_file: indexed_torrent.torrent_file,
        }
    }
}

/// Torrent that has been added to the index.
pub struct TorrentListedInIndex {
    pub torrent_id: Id,
    pub title: String,
    pub description: String,
    pub category: String,
    pub torrent_file: BinaryFile,
}

impl TorrentListedInIndex {
    pub fn from(torrent_to_index: TorrentIndexInfo, torrent_id: Id) -> Self {
        Self {
            torrent_id,
            title: torrent_to_index.title,
            description: torrent_to_index.description,
            category: torrent_to_index.category,
            torrent_file: torrent_to_index.torrent_file,
        }
    }
}

/// Add a new random torrent to the index
pub async fn upload_random_torrent_to_index(uploader: &LoggedInUserData) -> (TestTorrent, TorrentListedInIndex) {
    let random_torrent = random_torrent();
    let indexed_torrent = upload_torrent(uploader, &random_torrent.index_info).await;
    (random_torrent, indexed_torrent)
}

/// Upload a torrent to the index
pub async fn upload_torrent(uploader: &LoggedInUserData, torrent: &TorrentIndexInfo) -> TorrentListedInIndex {
    let client = TestEnv::default().authenticated_client(&uploader.token);

    let form: UploadTorrentMultipartForm = torrent.clone().into();

    let response = client.upload_torrent(form.into()).await;

    let res: UploadedTorrentResponse = serde_json::from_str(&response.body).unwrap();

    TorrentListedInIndex::from(torrent.clone(), res.data.torrent_id)
}

#[derive(Clone)]
pub struct TestTorrent {
    /// Parsed info from torrent file.
    pub file_info: TorrentFileInfo,
    /// Torrent info needed to add the torrent to the index.
    pub index_info: TorrentIndexInfo,
}

impl TestTorrent {
    pub fn random() -> Self {
        let temp_dir = temp_dir();

        let torrents_dir_path = temp_dir.path().to_owned();

        // Random ID to identify all the torrent related entities: files, fields, ...
        // That makes easier to debug the tests outputs.
        let id = Uuid::new_v4();

        // Create a random torrent file
        let torrent_path = random_torrent_file(&torrents_dir_path, &id);

        // Load torrent binary file
        let torrent_file = BinaryFile::from_file_at_path(&torrent_path);

        // Load torrent file metadata
        let torrent_info = parse_torrent(&torrent_path);

        let torrent_to_index = TorrentIndexInfo {
            title: format!("title-{id}"),
            description: format!("description-{id}"),
            category: software_predefined_category_name(),
            torrent_file,
        };

        TestTorrent {
            file_info: torrent_info,
            index_info: torrent_to_index,
        }
    }
}

pub fn random_torrent() -> TestTorrent {
    TestTorrent::random()
}

pub fn random_torrent_file(dir: &Path, id: &Uuid) -> PathBuf {
    // Create random text file
    let file_name = random_txt_file(dir, id);

    // Create torrent file for the text file
    create_torrent(dir, &file_name)
}

pub fn random_txt_file(dir: &Path, id: &Uuid) -> String {
    // Sample file name
    let file_name = format!("file-{id}.txt");

    // Sample file path
    let file_path = dir.join(file_name.clone());

    // Write sample text to the temporary file
    let mut file = File::create(file_path).unwrap();
    file.write_all(id.as_bytes()).unwrap();

    file_name
}

pub fn temp_dir() -> TempDir {
    tempdir().unwrap()
}
