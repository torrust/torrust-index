use serde::{Deserialize, Serialize};
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

use crate::config::Configuration;
use crate::utils::hex::{from_bytes, into_bytes};

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
    /// [BEP 39](http://www.bittorrent.org/beps/bep_0030.html)
    #[must_use]
    pub fn get_pieces_as_string(&self) -> String {
        match &self.pieces {
            None => String::new(),
            Some(byte_buf) => from_bytes(byte_buf.as_ref()),
        }
    }

    #[must_use]
    pub fn get_root_hash_as_i64(&self) -> i64 {
        match &self.root_hash {
            None => 0i64,
            Some(root_hash) => root_hash
                .parse::<i64>()
                .expect("variable `root_hash` cannot be converted into a `i64`"),
        }
    }

    #[must_use]
    pub fn is_a_single_file_torrent(&self) -> bool {
        self.length.is_some()
    }

    #[must_use]
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
    #[must_use]
    pub fn from_db_info_files_and_announce_urls(
        torrent_info: DbTorrentInfo,
        torrent_files: Vec<TorrentFile>,
        torrent_announce_urls: Vec<Vec<String>>,
    ) -> Self {
        let private = u8::try_from(torrent_info.private.unwrap_or(0)).ok();

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
            let pieces = into_bytes(&torrent_info.pieces).expect("variable `torrent_info.pieces` is not a valid hex string");
            info.pieces = Some(ByteBuf::from(pieces));
        }

        // either set the single file or the multiple files information
        if torrent_files.len() == 1 {
            let torrent_file = torrent_files
                .first()
                .expect("vector `torrent_files` should have at least one element");

            info.md5sum = torrent_file.md5sum.clone();

            info.length = Some(torrent_file.length);

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

    /// Sets the announce url to the tracker url and removes all other trackers
    /// if the torrent is private.
    pub async fn set_announce_urls(&mut self, cfg: &Configuration) {
        let settings = cfg.settings.read().await;

        self.announce = Some(settings.tracker.url.clone());

        // if torrent is private, remove all other trackers
        if let Some(private) = self.info.private {
            if private == 1 {
                self.announce_list = None;
            }
        }
    }

    #[must_use]
    pub fn calculate_info_hash_as_bytes(&self) -> [u8; 20] {
        let info_bencoded = ser::to_bytes(&self.info).expect("variable `info` was not able to be serialized.");
        let mut hasher = Sha1::new();
        hasher.update(info_bencoded);
        let sum_hex = hasher.finalize();
        let mut sum_bytes: [u8; 20] = Default::default();
        sum_bytes.copy_from_slice(sum_hex.as_slice());
        sum_bytes
    }

    #[must_use]
    pub fn info_hash(&self) -> String {
        from_bytes(&self.calculate_info_hash_as_bytes()).to_lowercase()
    }

    #[must_use]
    pub fn file_size(&self) -> i64 {
        match self.info.length {
            Some(length) => length,
            None => match &self.info.files {
                None => 0,
                Some(files) => {
                    let mut file_size = 0;
                    for file in files.iter() {
                        file_size += file.length;
                    }
                    file_size
                }
            },
        }
    }

    #[must_use]
    pub fn announce_urls(&self) -> Vec<String> {
        match &self.announce_list {
            Some(list) => list.clone().into_iter().flatten().collect::<Vec<String>>(),
            None => vec![self.announce.clone().expect("variable `announce` should not be None")],
        }
    }

    #[must_use]
    pub fn is_a_single_file_torrent(&self) -> bool {
        self.info.is_a_single_file_torrent()
    }

    #[must_use]
    pub fn is_a_multiple_file_torrent(&self) -> bool {
        self.info.is_a_multiple_file_torrent()
    }
}

#[allow(clippy::module_name_repetitions)]
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
