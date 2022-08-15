use serde::{Deserialize, Serialize};
use crate::config::Configuration;
use serde_bencode::ser;
use sha1::{Digest, Sha1};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentNode(String, i64);

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentFile {
    pub path: Vec<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentInfo {
    pub name: String,
    #[serde(default)]
    pub pieces: Option<String>,
    #[serde(rename="piece length")]
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
    #[serde(rename="root hash")]
    pub root_hash: Option<String>,
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
    #[serde(rename="announce-list")]
    pub announce_list: Option<Vec<Vec<String>>>,
    #[serde(default)]
    #[serde(rename="creation date")]
    pub creation_date: Option<i64>,
    #[serde(rename="comment")]
    pub comment: Option<String>,
    #[serde(default)]
    #[serde(rename="created by")]
    pub created_by: Option<String>,
}

impl Torrent {
    pub fn from_db_info_files_and_announce_urls(torrent_info: DbTorrentInfo, torrent_files: Vec<DbTorrentFile>, torrent_announce_urls: Vec<DbTorrentAnnounceUrl>) -> Self {
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
            root_hash: None
        };

        // a torrent file has a root hash or a pieces key, but not both.
        if torrent_info.root_hash > 0 {
            info.root_hash = Some(torrent_info.pieces);
        } else {
            info.pieces = Some(torrent_info.pieces);
        }

        // either set the single file or the multiple files information
        if torrent_files.len() == 1 {
            // can safely unwrap because we know there is 1 element
            let torrent_file = torrent_files.first().unwrap();

            let path: Option<Vec<String>> = torrent_file.path.as_ref().map(|path| path.split('/').map(|s| s.to_string()).collect());

            info.md5sum = torrent_file.md5sum.clone();

            info.length = Some(torrent_file.length);

            // when storing the path in the database, we join the elements on '/'. So now they have to be separated again.
            info.path = path;
        } else {
            let mut files: Vec<TorrentFile> = vec![];

            for torrent_file in torrent_files.iter() {
                let file = TorrentFile {
                    // path must be set when there are multiple files, so we can safely unwrap
                    path: torrent_file.path.as_ref().unwrap().split('/').map(|s| s.to_string()).collect(),
                    length: torrent_file.length,
                    md5sum: torrent_file.md5sum.clone()
                };

                files.push(file);
            }

            info.files = Some(files);
        }

        // form the tracker announce-list
        let mut tracker_urls: Vec<Vec<String>> = vec![];

        for torrent_announce_url in torrent_announce_urls.iter() {
            tracker_urls.push(vec![torrent_announce_url.tracker_url.to_string()]);
        }

        Self {
            info,
            announce: None,
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: Some(tracker_urls),
            creation_date: None,
            comment: None,
            created_by: None
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
        let mut buffer = [0u8; 40];
        let input = &self.calculate_info_hash_as_bytes();
        let bytes_out = binascii::bin2hex(input, &mut buffer).ok().unwrap();
        String::from(std::str::from_utf8(bytes_out).unwrap())
    }

    pub fn file_size(&self) -> i64 {
        if self.info.length.is_some() {
            return self.info.length.unwrap()
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
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentFile {
    pub path: Option<String>,
    pub length: i64,
    #[serde(default)]
    pub md5sum: Option<String>,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentInfo {
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    #[serde(default)]
    pub private: Option<i64>,
    pub root_hash: i64,
}

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrentAnnounceUrl {
    pub tracker_url: String,
}
