use serde::{Deserialize, Serialize};
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

use crate::config::Configuration;
use crate::utils::hex::{bytes_to_hex, hex_to_bytes};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentNode(String, i64);

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    #[serde(default)]
    pub pieces: Option<ByteBuf>,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
    #[serde(default)]
    pub length: Option<i64>,
    #[serde(default)]
    pub files: Option<Vec<TorrentFile>>,
    #[serde(default)]
    pub private: Option<u8>,
    #[serde(default)]
    pub path: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "root hash")]
    pub root_hash: Option<String>,
}

impl TorrentInfo {
    /// torrent file can only hold a pieces key or a root hash key:
    /// http://www.bittorrent.org/beps/bep_0030.html
    pub fn get_pieces_as_string(&self) -> String {
        match &self.pieces {
            None => "".to_string(),
            Some(byte_buf) => bytes_to_hex(byte_buf.as_ref()),
        }
    }

    pub fn get_root_hash_as_i64(&self) -> i64 {
        match &self.root_hash {
            None => 0i64,
            Some(root_hash) => root_hash.parse::<i64>().unwrap(),
        }
    }

    pub fn is_a_single_file_torrent(&self) -> bool {
        self.length.is_some()
    }

    pub fn is_a_multiple_file_torrent(&self) -> bool {
        self.files.is_some()
    }
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub info: TorrentInfo, //
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    pub nodes: Option<Vec<TorrentNode>>,
    #[serde(default)]
    pub encoding: Option<String>,
    #[serde(default)]
    pub httpseeds: Option<Vec<String>>,
    #[serde(default)]
    #[serde(rename = "announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename = "creation date")]
    pub creation_date: Option<i64>,
    #[serde(rename = "comment")]
    pub comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
}

impl Torrent {
    pub fn from_db_info_files_and_announce_urls(
        torrent_info: DbTorrentInfo,
        torrent_files: Vec<TorrentFile>,
        torrent_announce_urls: Vec<Vec<String>>,
    ) -> Self {
        let private = if let Some(private_i64) = torrent_info.private {
            // must fit in a byte
            let private = if (0..256).contains(&private_i64) { private_i64 } else { 0 };
            Some(private as u8)
        } else {
            None
        };

        // the info part of the torrent file
        let mut info = TorrentInfo {
            name: torrent_info.name.to_string(),
            pieces: None,
            piece_length: torrent_info.piece_length,
            md5sum: None,
            length: None,
            files: None,
            private,
            path: None,
            root_hash: None,
        };

        // a torrent file has a root hash or a pieces key, but not both.
        if torrent_info.root_hash > 0 {
            info.root_hash = Some(torrent_info.pieces);
        } else {
            let pieces = hex_to_bytes(&torrent_info.pieces).unwrap();
            info.pieces = Some(ByteBuf::from(pieces));
        }

        // either set the single file or the multiple files information
        if torrent_files.len() == 1 {
            // can safely unwrap because we know there is 1 element
            let torrent_file = torrent_files.first().unwrap();

            info.md5sum = torrent_file.md5sum.clone();

            info.length = Some(torrent_file.length);

            let path = if torrent_file.path.first().as_ref().unwrap().is_empty() {
                None
            } else {
                Some(torrent_file.path.clone())
            };

            info.path = path;
        } else {
            info.files = Some(torrent_files);
        }

        Self {
            info,
            announce: None,
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: Some(torrent_announce_urls),
            creation_date: None,
            comment: None,
            created_by: None,
        }
    }

    pub async fn set_torrust_config(&mut self, cfg: &Configuration) {
        let settings = cfg.settings.read().await;

        self.announce = Some(settings.tracker.url.clone());

        // if torrent is private, remove all other trackers
        if let Some(private) = self.info.private {
            if private == 1 {
                self.announce_list = None;
            }
        }
    }

    pub fn calculate_info_hash_as_bytes(&self) -> [u8; 20] {
        let info_bencoded = ser::to_bytes(&self.info).unwrap();
        let mut hasher = Sha1::new();
        hasher.update(info_bencoded);
        let sum_hex = hasher.finalize();
        let mut sum_bytes: [u8; 20] = Default::default();
        sum_bytes.copy_from_slice(sum_hex.as_slice());
        sum_bytes
    }

    pub fn info_hash(&self) -> String {
        bytes_to_hex(&self.calculate_info_hash_as_bytes())
    }

    pub fn file_size(&self) -> i64 {
        if self.info.length.is_some() {
            self.info.length.unwrap()
        } else {
            match &self.info.files {
                None => 0,
                Some(files) => {
                    let mut file_size = 0;
                    for file in files.iter() {
                        file_size += file.length;
                    }
                    file_size
                }
            }
        }
    }

    pub fn announce_urls(&self) -> Vec<String> {
        if self.announce_list.is_none() {
            return vec![self.announce.clone().unwrap()];
        }

        self.announce_list
            .clone()
            .unwrap()
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
    }

    pub fn is_a_single_file_torrent(&self) -> bool {
        self.info.is_a_single_file_torrent()
    }

    pub fn is_a_multiple_file_torrent(&self) -> bool {
        self.info.is_a_multiple_file_torrent()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentFile {
    pub path: Option<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentInfo {
    pub torrent_id: i64,
    pub info_hash: String,
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    #[serde(default)]
    pub private: Option<i64>,
    pub root_hash: i64,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentAnnounceUrl {
    pub tracker_url: String,
}
