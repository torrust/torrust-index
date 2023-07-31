//! This module contains the services related to torrent file management.
use uuid::Uuid;

use crate::models::torrent_file::{Torrent, TorrentFile};
use crate::services::hasher::sha1;

pub struct NewTorrentInfoRequest {
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    pub private: Option<i64>,
    pub root_hash: i64,
    pub files: Vec<TorrentFile>,
    pub announce_urls: Vec<Vec<String>>,
}

/// It generates a random single-file torrent for testing purposes.
///
/// The torrent will contain a single text file with the UUID as its content.
///
/// # Panics
///
/// This function will panic if the sample file contents length in bytes is
/// greater than `i64::MAX`.
#[must_use]
pub fn generate_random_torrent(id: Uuid) -> Torrent {
    // Content of the file from which the torrent will be generated.
    // We use the UUID as the content of the file.
    let file_contents = format!("{id}\n");

    let torrent_files: Vec<TorrentFile> = vec![TorrentFile {
        path: vec![String::new()],
        length: i64::try_from(file_contents.len()).expect("file contents size in bytes cannot exceed i64::MAX"),
        md5sum: None,
    }];

    let torrent_announce_urls: Vec<Vec<String>> = vec![];

    let torrent_info_request = NewTorrentInfoRequest {
        name: format!("file-{id}.txt"),
        pieces: sha1(&file_contents),
        piece_length: 16384,
        private: None,
        root_hash: 0,
        files: torrent_files,
        announce_urls: torrent_announce_urls,
    };

    Torrent::from_new_torrent_info_request(torrent_info_request)
}
