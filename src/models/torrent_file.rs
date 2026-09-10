use serde::{Deserialize, Serialize};
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};
use torrust_info_hash::InfoHash;
use tracing::error;
use url::Url;

use crate::utils::hex::{from_bytes, into_bytes};

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub info: TorrentInfoDictionary, //
    #[serde(default)]
    pub announce: Option<String>,
    #[serde(default)]
    pub nodes: Option<Vec<(String, i64)>>,
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
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfoDictionary {
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
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

impl Torrent {
    /// It hydrates a `Torrent` struct from the database data.
    ///
    /// # Panics
    ///
    /// This function will panic if the `torrent_info.pieces` is not a valid
    /// hex string.
    #[must_use]
    pub fn from_database(
        db_torrent: &DbTorrent,
        torrent_files: &[TorrentFile],
        torrent_announce_urls: Vec<Vec<String>>,
        torrent_http_seed_urls: Vec<String>,
        torrent_nodes: Vec<(String, i64)>,
    ) -> Self {
        let pieces_or_root_hash = if db_torrent.is_bep_30 == 0 {
            db_torrent.pieces.as_ref().map_or_else(
                || {
                    error!("Invalid torrent #{}. Null `pieces` in database", db_torrent.torrent_id);
                    String::new()
                },
                std::clone::Clone::clone,
            )
        } else {
            // A BEP-30 torrent
            db_torrent.root_hash.as_ref().map_or_else(
                || {
                    error!("Invalid torrent #{}. Null `root_hash` in database", db_torrent.torrent_id);
                    String::new()
                },
                std::clone::Clone::clone,
            )
        };

        let info_dict = TorrentInfoDictionary::with(
            &db_torrent.name,
            db_torrent.piece_length,
            db_torrent.private,
            db_torrent.is_bep_30,
            &pieces_or_root_hash,
            torrent_files,
        );

        Self {
            info: info_dict,
            announce: None,
            nodes: if torrent_nodes.is_empty() { None } else { Some(torrent_nodes) },
            encoding: db_torrent.encoding.clone(),
            httpseeds: if torrent_http_seed_urls.is_empty() {
                None
            } else {
                Some(torrent_http_seed_urls)
            },
            announce_list: Some(torrent_announce_urls),
            creation_date: db_torrent.creation_date,
            comment: db_torrent.comment.clone(),
            created_by: db_torrent.created_by.clone(),
        }
    }

    /// Includes the tracker URL a the main tracker in the torrent.
    ///
    /// It will be the URL in the `announce` field and also the first URL in the
    /// `announce_list`.
    pub fn include_url_as_main_tracker(&mut self, tracker_url: &Url) {
        self.set_announce_to(tracker_url);
        self.add_url_to_front_of_announce_list(tracker_url);
    }

    /// Sets the announce url to the tracker url.
    pub fn set_announce_to(&mut self, tracker_url: &Url) {
        self.announce = Some(tracker_url.to_owned().to_string());
    }

    /// Adds a new tracker URL to the front of the `announce_list`, removes duplicates,
    /// and cleans up any empty inner lists.
    ///
    /// In practice, it's common for the `announce_list` to include the URL from
    /// the `announce` field as one of its entries, often in the first tier,
    /// to ensure that this primary tracker is always used. However, this is not
    /// a strict requirement of the `BitTorrent` protocol; it's more of a
    /// convention followed by some torrent creators for redundancy and to
    /// ensure better availability of trackers.    
    pub fn add_url_to_front_of_announce_list(&mut self, tracker_url: &Url) {
        if let Some(list) = &mut self.announce_list {
            // Remove the tracker URL from existing lists
            for inner_list in list.iter_mut() {
                inner_list.retain(|url| *url != tracker_url.to_string());
            }

            // Prepend a new vector containing the tracker_url
            let vec = vec![tracker_url.to_owned().to_string()];
            list.insert(0, vec);

            // Remove any empty inner lists
            list.retain(|inner_list| !inner_list.is_empty());
        }
    }

    /// Removes all other trackers if the torrent is private.
    pub fn reset_announce_list_if_private(&mut self) {
        if self.is_private() {
            self.announce_list = None;
        }
    }

    const fn is_private(&self) -> bool {
        matches!(self.info.private, Some(1))
    }

    /// It calculates the info hash of the torrent file.
    ///
    /// # Panics
    ///
    /// This function will panic if the `info` part of the torrent file cannot be serialized.
    #[must_use]
    pub fn calculate_info_hash_as_bytes(&self) -> [u8; 20] {
        let info_bencoded = ser::to_bytes(&self.info).expect("variable `info` was not able to be serialized.");
        let mut hasher = Sha1::new();
        hasher.update(info_bencoded);
        let sum_hex = hasher.finalize();
        let mut sum_bytes: [u8; 20] = Default::default();
        sum_bytes.copy_from_slice(&sum_hex);
        sum_bytes
    }

    #[must_use]
    pub fn canonical_info_hash(&self) -> InfoHash {
        self.calculate_info_hash_as_bytes().into()
    }

    #[must_use]
    pub fn canonical_info_hash_hex(&self) -> String {
        self.canonical_info_hash().to_hex_string()
    }

    #[must_use]
    pub fn file_size(&self) -> i64 {
        self.info.length.unwrap_or_else(|| {
            self.info.files.as_ref().map_or(0, |files| {
                let mut file_size = 0;
                for file in files {
                    file_size += file.length;
                }
                file_size
            })
        })
    }

    /// It returns the announce urls of the torrent file.
    ///
    /// # Panics
    ///
    /// This function will panic if both the `announce_list` and the `announce` are `None`.
    #[must_use]
    pub fn announce_urls(&self) -> Vec<String> {
        self.announce_list.as_ref().map_or_else(
            || vec![self.announce.clone().expect("variable `announce` should not be None")],
            |list| list.clone().into_iter().flatten().collect::<Vec<String>>(),
        )
    }

    #[must_use]
    pub const fn is_a_single_file_torrent(&self) -> bool {
        self.info.is_a_single_file_torrent()
    }

    #[must_use]
    pub const fn is_a_multiple_file_torrent(&self) -> bool {
        self.info.is_a_multiple_file_torrent()
    }
}

impl TorrentInfoDictionary {
    /// Constructor.
    ///
    /// # Panics
    ///
    /// This function will panic if:
    ///
    /// - The `pieces` field is not a valid hex string.
    /// - For single files torrents the `TorrentFile` path is empty.
    #[must_use]
    pub fn with(
        name: &str,
        piece_length: i64,
        private: Option<u8>,
        is_bep_30: i64,
        pieces_or_root_hash: &str,
        files: &[TorrentFile],
    ) -> Self {
        let mut info_dict = Self {
            name: name.to_string(),
            pieces: None,
            piece_length,
            md5sum: None,
            length: None,
            files: None,
            private,
            path: None,
            root_hash: None,
            source: None,
        };

        // BEP 30: <http://www.bittorrent.org/beps/bep_0030.html>.
        // Torrent file can only hold a `pieces` key or a `root hash` key
        if is_bep_30 == 0 {
            let buffer = into_bytes(pieces_or_root_hash).expect("variable `torrent_info.pieces` is not a valid hex string");
            info_dict.pieces = Some(ByteBuf::from(buffer));
        } else {
            info_dict.root_hash = Some(pieces_or_root_hash.to_owned());
        }

        // either set the single file or the multiple files information
        if files.len() == 1 {
            let torrent_file = files
                .first()
                .expect("vector `torrent_files` should have at least one element");

            info_dict.md5sum.clone_from(&torrent_file.md5sum); // DevSkim: ignore DS126858

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
            info_dict.files = Some(files.to_vec());
        }

        info_dict
    }

    /// torrent file can only hold a pieces key or a root hash key:
    /// [BEP 39](http://www.bittorrent.org/beps/bep_0030.html)
    #[must_use]
    pub fn get_pieces_as_string(&self) -> String {
        self.pieces
            .as_ref()
            .map_or_else(String::new, |byte_buf| from_bytes(byte_buf.as_ref()))
    }

    /// torrent file can only hold a pieces key or a root hash key:
    /// [BEP 39](http://www.bittorrent.org/beps/bep_0030.html)
    #[must_use]
    pub fn get_root_hash_as_string(&self) -> String {
        self.root_hash.as_ref().map_or_else(String::new, std::clone::Clone::clone)
    }

    /// It returns true if the torrent is a BEP-30 torrent.
    #[must_use]
    pub const fn is_bep_30(&self) -> bool {
        self.root_hash.is_some()
    }

    #[must_use]
    pub const fn is_a_single_file_torrent(&self) -> bool {
        self.length.is_some()
    }

    #[must_use]
    pub const fn is_a_multiple_file_torrent(&self) -> bool {
        self.files.is_some()
    }
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrent {
    pub torrent_id: i64,
    pub info_hash: String,
    pub name: String,
    pub pieces: Option<String>,
    pub root_hash: Option<String>,
    pub piece_length: i64,
    #[serde(default)]
    pub private: Option<u8>,
    pub is_bep_30: i64,
    pub comment: Option<String>,
    pub creation_date: Option<i64>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
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
pub struct DbTorrentAnnounceUrl {
    pub tracker_url: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentHttpSeedUrl {
    pub seed_url: String,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentNode {
    pub node_ip: String,
    pub node_port: i64,
}
