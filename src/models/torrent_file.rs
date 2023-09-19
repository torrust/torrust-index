use serde::{Deserialize, Serialize};
use serde_bencode::ser;
use serde_bytes::ByteBuf;
use sha1::{Digest, Sha1};

use super::info_hash::InfoHash;
use crate::config::Configuration;
use crate::utils::hex::{from_bytes, into_bytes};

#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct Torrent {
    pub info: TorrentInfoDictionary, //
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
    #[serde(default)]
    pub comment: Option<String>,
    #[serde(default)]
    #[serde(rename = "created by")]
    pub created_by: Option<String>,
}

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct TorrentNode(String, i64);

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
        torrent_files: &Vec<TorrentFile>,
        torrent_announce_urls: Vec<Vec<String>>,
    ) -> Self {
        let info_dict = TorrentInfoDictionary::with(
            &db_torrent.name,
            db_torrent.piece_length,
            db_torrent.private,
            db_torrent.root_hash,
            &db_torrent.pieces,
            torrent_files,
        );

        Self {
            info: info_dict,
            announce: None,
            nodes: None,
            encoding: None,
            httpseeds: None,
            announce_list: Some(torrent_announce_urls),
            creation_date: None,
            comment: db_torrent.comment.clone(),
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
        sum_bytes.copy_from_slice(sum_hex.as_slice());
        sum_bytes
    }

    #[must_use]
    pub fn info_hash_hex(&self) -> String {
        from_bytes(&self.calculate_info_hash_as_bytes()).to_lowercase()
    }

    #[must_use]
    pub fn canonical_info_hash(&self) -> InfoHash {
        self.calculate_info_hash_as_bytes().into()
    }

    #[must_use]
    pub fn file_size(&self) -> i64 {
        match self.info.length {
            Some(length) => length,
            None => match &self.info.files {
                None => 0,
                Some(files) => {
                    let mut file_size = 0;
                    for file in files {
                        file_size += file.length;
                    }
                    file_size
                }
            },
        }
    }

    /// It returns the announce urls of the torrent file.
    ///
    /// # Panics
    ///
    /// This function will panic if both the `announce_list` and the `announce` are `None`.
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
        root_hash: i64,
        pieces: &str,
        files: &Vec<TorrentFile>,
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

        // a torrent file has a root hash or a pieces key, but not both.
        if root_hash > 0 {
            info_dict.root_hash = Some(pieces.to_owned());
        } else {
            let buffer = into_bytes(pieces).expect("variable `torrent_info.pieces` is not a valid hex string");
            info_dict.pieces = Some(ByteBuf::from(buffer));
        }

        // either set the single file or the multiple files information
        if files.len() == 1 {
            let torrent_file = files
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
            info_dict.files = Some(files.clone());
        }

        info_dict
    }

    /// torrent file can only hold a pieces key or a root hash key:
    /// [BEP 39](http://www.bittorrent.org/beps/bep_0030.html)
    #[must_use]
    pub fn get_pieces_as_string(&self) -> String {
        match &self.pieces {
            None => String::new(),
            Some(byte_buf) => from_bytes(byte_buf.as_ref()),
        }
    }

    /// It returns the root hash as a `i64` value.
    ///
    /// # Panics
    ///
    /// This function will panic if the root hash cannot be converted into a
    /// `i64` value.
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

#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize, sqlx::FromRow)]
pub struct DbTorrent {
    pub torrent_id: i64,
    pub info_hash: String,
    pub name: String,
    pub pieces: String,
    pub piece_length: i64,
    #[serde(default)]
    pub private: Option<u8>,
    pub root_hash: i64,
    pub comment: Option<String>,
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

#[cfg(test)]
mod tests {

    mod info_hash_calculation_for_version_v1 {

        use serde_bytes::ByteBuf;

        use crate::models::torrent_file::{Torrent, TorrentInfoDictionary};

        #[test]
        fn the_parsed_torrent_file_should_calculated_the_torrent_info_hash() {
            /* The sample.txt content (`mandelbrot`):

               ```
               6d616e64656c62726f740a
               ```

               The sample.txt.torrent content:

               ```
               6431303a6372656174656420627931383a71426974746f7272656e742076
               342e352e3431333a6372656174696f6e2064617465693136393131343935
               373265343a696e666f64363a6c656e67746869313165343a6e616d653130
               3a73616d706c652e74787431323a7069656365206c656e67746869313633
               383465363a70696563657332303ad491587f1c42dff0cb0ff5c2b8cefe22
               b3ad310a6565
               ```

               ```json
               {
                 "created by": "qBittorrent v4.5.4",
                 "creation date": 1691149572,
                 "info": {
                   "length": 11,
                   "name": "sample.txt",
                   "piece length": 16384,
                   "pieces": "<hex>D4 91 58 7F 1C 42 DF F0 CB 0F F5 C2 B8 CE FE 22 B3 AD 31 0A</hex>"
                 }
               }
               ```
            */

            let sample_data_in_txt_file = "mandelbrot\n";

            let info = TorrentInfoDictionary {
                name: "sample.txt".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                    212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                ])),
                piece_length: 16384,
                md5sum: None,
                length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                files: None,
                private: None,
                path: None,
                root_hash: None,
                source: None,
            };

            let torrent = Torrent {
                info: info.clone(),
                announce: None,
                announce_list: Some(vec![]),
                creation_date: None,
                comment: None,
                created_by: None,
                nodes: None,
                encoding: None,
                httpseeds: None,
            };

            assert_eq!(torrent.info_hash_hex(), "79fa9e4a2927804fe4feab488a76c8c2d3d1cdca");
        }

        mod infohash_should_be_calculated_for {

            use serde_bytes::ByteBuf;

            use crate::models::torrent_file::{Torrent, TorrentFile, TorrentInfoDictionary};

            #[test]
            fn a_simple_single_file_torrent() {
                let sample_data_in_txt_file = "mandelbrot\n";

                let info = TorrentInfoDictionary {
                    name: "sample.txt".to_string(),
                    pieces: Some(ByteBuf::from(vec![
                        // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                        212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                    ])),
                    piece_length: 16384,
                    md5sum: None,
                    length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                    files: None,
                    private: None,
                    path: None,
                    root_hash: None,
                    source: None,
                };

                let torrent = Torrent {
                    info: info.clone(),
                    announce: None,
                    announce_list: Some(vec![]),
                    creation_date: None,
                    comment: None,
                    created_by: None,
                    nodes: None,
                    encoding: None,
                    httpseeds: None,
                };

                assert_eq!(torrent.info_hash_hex(), "79fa9e4a2927804fe4feab488a76c8c2d3d1cdca");
            }

            #[test]
            fn a_simple_multi_file_torrent() {
                let sample_data_in_txt_file = "mandelbrot\n";

                let info = TorrentInfoDictionary {
                    name: "sample".to_string(),
                    pieces: Some(ByteBuf::from(vec![
                        // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                        212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                    ])),
                    piece_length: 16384,
                    md5sum: None,
                    length: None,
                    files: Some(vec![TorrentFile {
                        path: vec!["sample.txt".to_string()],
                        length: sample_data_in_txt_file.len().try_into().unwrap(),
                        md5sum: None,
                    }]),
                    private: None,
                    path: None,
                    root_hash: None,
                    source: None,
                };

                let torrent = Torrent {
                    info: info.clone(),
                    announce: None,
                    announce_list: Some(vec![]),
                    creation_date: None,
                    comment: None,
                    created_by: None,
                    nodes: None,
                    encoding: None,
                    httpseeds: None,
                };

                assert_eq!(torrent.info_hash_hex(), "aa2aca91ab650c4d249c475ca3fa604f2ccb0d2a");
            }

            #[test]
            fn a_simple_single_file_torrent_with_a_source() {
                let sample_data_in_txt_file = "mandelbrot\n";

                let info = TorrentInfoDictionary {
                    name: "sample.txt".to_string(),
                    pieces: Some(ByteBuf::from(vec![
                        // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                        212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                    ])),
                    piece_length: 16384,
                    md5sum: None,
                    length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                    files: None,
                    private: None,
                    path: None,
                    root_hash: None,
                    source: Some("ABC".to_string()), // The tracker three-letter code
                };

                let torrent = Torrent {
                    info: info.clone(),
                    announce: None,
                    announce_list: Some(vec![]),
                    creation_date: None,
                    comment: None,
                    created_by: None,
                    nodes: None,
                    encoding: None,
                    httpseeds: None,
                };

                assert_eq!(torrent.info_hash_hex(), "ccc1cf4feb59f3fa85c96c9be1ebbafcfe8a9cc8");
            }

            #[test]
            fn a_simple_single_file_private_torrent() {
                let sample_data_in_txt_file = "mandelbrot\n";

                let info = TorrentInfoDictionary {
                    name: "sample.txt".to_string(),
                    pieces: Some(ByteBuf::from(vec![
                        // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                        212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                    ])),
                    piece_length: 16384,
                    md5sum: None,
                    length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                    files: None,
                    private: Some(1),
                    path: None,
                    root_hash: None,
                    source: None,
                };

                let torrent = Torrent {
                    info: info.clone(),
                    announce: None,
                    announce_list: Some(vec![]),
                    creation_date: None,
                    comment: None,
                    created_by: None,
                    nodes: None,
                    encoding: None,
                    httpseeds: None,
                };

                assert_eq!(torrent.info_hash_hex(), "d3a558d0a19aaa23ba6f9f430f40924d10fefa86");
            }
        }
    }
}
