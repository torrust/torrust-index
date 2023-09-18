//! This module contains the services related to torrent file management.
use serde_bytes::ByteBuf;
use uuid::Uuid;

use crate::models::torrent_file::{Torrent, TorrentFile, TorrentInfoDictionary};
use crate::services::hasher::sha1;
use crate::utils::hex::into_bytes;

/// It contains the information required to create a new torrent file.
///
/// It's not the full in-memory representation of a torrent file. The full
/// in-memory representation is the `Torrent` struct.
pub struct CreateTorrentRequest {
    // The `info` dictionary fields
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    pub private: Option<u8>,
    pub root_hash: i64,
    pub files: Vec<TorrentFile>,
    // Other fields of the root level metainfo dictionary
    pub announce_urls: Vec<Vec<String>>,
    pub comment: Option<String>,
}

impl CreateTorrentRequest {
    /// It builds a `TorrentInfoDictionary` from the current torrent request.
    ///
    /// # Panics
    ///
    /// This function will panic if the `pieces` field is not a valid hex string.
    #[must_use]
    pub fn build_info_dictionary(&self) -> TorrentInfoDictionary {
        let mut info_dict = TorrentInfoDictionary {
            name: self.name.to_string(),
            pieces: None,
            piece_length: self.piece_length,
            md5sum: None,
            length: None,
            files: None,
            private: self.private,
            path: None,
            root_hash: None,
            source: None,
        };

        // a torrent file has a root hash or a pieces key, but not both.
        if self.root_hash > 0 {
            info_dict.root_hash = Some(self.pieces.clone());
        } else {
            let buffer = into_bytes(&self.pieces).expect("variable `torrent_info.pieces` is not a valid hex string");
            info_dict.pieces = Some(ByteBuf::from(buffer));
        }

        // either set the single file or the multiple files information
        if self.files.len() == 1 {
            let torrent_file = self
                .files
                .first()
                .expect("vector `torrent_files` should have at least one element");

            info_dict.md5sum = torrent_file.md5sum.clone();

            info_dict.length = Some(torrent_file.length);

            let path = if torrent_file
                .path
                .first()
                .as_ref()
                .expect("the vector for the `path` should have at least one element")
                .is_empty()
            {
                None
            } else {
                Some(torrent_file.path.clone())
            };

            info_dict.path = path;
        } else {
            info_dict.files = Some(self.files.clone());
        }

        info_dict
    }
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

    let torrent_info_request = CreateTorrentRequest {
        name: format!("file-{id}.txt"),
        pieces: sha1(&file_contents),
        piece_length: 16384,
        private: None,
        root_hash: 0,
        files: torrent_files,
        announce_urls: torrent_announce_urls,
        comment: None,
    };

    Torrent::from_request(torrent_info_request)
}

#[cfg(test)]
mod tests {
    use serde_bytes::ByteBuf;
    use uuid::Uuid;

    use crate::models::torrent_file::{Torrent, TorrentInfoDictionary};
    use crate::services::torrent_file::generate_random_torrent;

    #[test]
    fn it_should_generate_a_random_meta_info_file() {
        let uuid = Uuid::parse_str("d6170378-2c14-4ccc-870d-2a8e15195e23").unwrap();

        let torrent = generate_random_torrent(uuid);

        let expected_torrent = Torrent {
            info: TorrentInfoDictionary {
                name: "file-d6170378-2c14-4ccc-870d-2a8e15195e23.txt".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    62, 231, 243, 51, 234, 165, 204, 209, 51, 132, 163, 133, 249, 50, 107, 46, 24, 15, 251, 32,
                ])),
                piece_length: 16384,
                md5sum: None,
                length: Some(37),
                files: None,
                private: None,
                path: None,
                root_hash: None,
                source: None,
            },
            announce: None,
            announce_list: Some(vec![]),
            creation_date: None,
            comment: None,
            created_by: None,
            nodes: None,
            encoding: None,
            httpseeds: None,
        };

        assert_eq!(torrent, expected_torrent);
    }
}
