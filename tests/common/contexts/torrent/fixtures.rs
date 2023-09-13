use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};
use serde_bytes::ByteBuf;
use tempfile::{tempdir, TempDir};
use torrust_index_backend::services::hasher::sha1;
use torrust_index_backend::utils::hex::into_bytes;
use uuid::Uuid;

use super::file::{create_torrent, parse_torrent, TorrentFileInfo};
use super::forms::{BinaryFile, UploadTorrentMultipartForm};
use super::requests::InfoHash;
use super::responses::Id;
use crate::common::contexts::category::fixtures::software_predefined_category_name;

/// Information about a torrent that is going to added to the index.
#[derive(Clone)]
pub struct TorrentIndexInfo {
    pub title: String,
    pub description: String,
    pub category: String,
    pub torrent_file: BinaryFile,
    pub name: String,
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
            name: format!("name-{id}"),
        };

        TestTorrent {
            file_info: torrent_info,
            index_info: torrent_to_index,
        }
    }

    pub fn with_custom_info_dict_field(id: Uuid, file_contents: &str, custom: &str) -> Self {
        let temp_dir = temp_dir();

        let torrents_dir_path = temp_dir.path().to_owned();

        // Create the torrent in memory
        let torrent = TestTorrentWithCustomInfoField::with_contents(id, file_contents, custom);

        // Bencode the torrent
        let torrent_data = TestTorrentWithCustomInfoField::encode(&torrent).unwrap();

        // Torrent temporary file path
        let filename = format!("file-{id}.txt.torrent");
        let torrent_path = torrents_dir_path.join(filename.clone());

        // Write the torrent file to the temporary file
        let mut file = File::create(torrent_path.clone()).unwrap();
        file.write_all(&torrent_data).unwrap();

        // Load torrent binary file
        let torrent_file = BinaryFile::from_file_at_path(&torrent_path);

        // Load torrent file metadata
        let torrent_info = parse_torrent(&torrent_path);

        let torrent_to_index = TorrentIndexInfo {
            title: format!("title-{id}"),
            description: format!("description-{id}"),
            category: software_predefined_category_name(),
            torrent_file,
            name: filename,
        };

        TestTorrent {
            file_info: torrent_info,
            index_info: torrent_to_index,
        }
    }

    pub fn info_hash_as_hex_string(&self) -> InfoHash {
        self.file_info.info_hash.clone()
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

/// A minimal torrent file with a custom field in the info dict.
///
/// ```json
/// {
///     "info": {
///        "length": 602515,
///        "name": "mandelbrot_set_01",
///        "piece length": 32768,
///        "pieces": "<hex>8A 88 32 BE ED 05 5F AA C4 AF 4A 90 4B 9A BF 0D EC 83 42 1C 73 39 05 B8 D6 20 2C 1B D1 8A 53 28 1F B5 D4 23 0A 23 C8 DB AC C4 E6 6B 16 12 08 C7 A4 AD 64 45 70 ED 91 0D F1 38 E7 DF 0C 1A D0 C9 23 27 7C D1 F9 D4 E5 A1 5F F5 E5 A0 E4 9E FB B1 43 F5 4B AD 0E D4 9D CB 49 F7 E6 7B BA 30 5F AF F9 88 56 FB 45 9A B4 95 92 3E 2C 7F DA A6 D3 82 E7 63 A3 BB 4B 28 F3 57 C7 CB 7D 8C 06 E3 46 AB D7 E8 8E 8A 8C 9F C7 E6 C5 C5 64 82 ED 47 BB 2A F1 B7 3F A5 3C 5B 9C AF 43 EC 2A E1 08 68 9A 49 C8 BF 1B 07 AD BE E9 2D 7E BE 9C 18 7F 4C A1 97 0E 54 3A 18 94 0E 60 8D 5C 69 0E 41 46 0D 3C 9A 37 F6 81 62 4F 95 C0 73 92 CA 9A D5 A9 89 AC 8B 85 12 53 0B FB E2 96 26 3E 26 A6 5B 70 53 48 65 F3 6C 27 0F 6B BD 1C EE EB 1A 9D 5F 77 A8 D8 AF D8 14 82 4A E0 B4 62 BC F1 A5 F5 F2 C7 60 F8 38 C8 5B 0B A9 07 DD 86 FA C0 7B F0 26 D7 D1 9A 42 C3 1F 9F B9 59 83 10 62 41 E9 06 3C 6D A1 19 75 01 57 25 9E B7 FE DF 91 04 D4 51 4B 6D 44 02 8D 31 8E 84 26 95 0F 30 31 F0 2C 16 39 BD 53 1D CF D3 5E 3E 41 A9 1E 14 3F 73 24 AC 5E 9E FC 4D C5 70 45 0F 45 8B 9B 52 E6 D0 26 47 8F 43 08 9E 2A 7C C5 92 D5 86 36 FE 48 E9 B8 86 84 92 23 49 5B EE C4 31 B2 1D 10 75 8E 4C 07 84 8F</hex>",
///        "custom": "custom03"
///     }
/// }
/// ```
///
/// Changing the value of the `custom` field will change the info-hash of the torrent.
#[derive(PartialEq, Debug, Clone, Serialize, Deserialize)]
pub struct TestTorrentWithCustomInfoField {
    pub info: InfoDictWithCustomField,
}

/// A minimal torrent info dict with a custom field.
#[derive(PartialEq, Eq, Debug, Clone, Serialize, Deserialize)]
pub struct InfoDictWithCustomField {
    #[serde(default)]
    pub length: i64,
    #[serde(default)]
    pub name: String,
    #[serde(rename = "piece length")]
    pub piece_length: i64,
    #[serde(default)]
    pub pieces: ByteBuf,
    #[serde(default)]
    pub custom: String,
}

impl TestTorrentWithCustomInfoField {
    pub fn with_contents(id: Uuid, file_contents: &str, custom: &str) -> Self {
        let sha1_of_file_contents = sha1(file_contents);
        let pieces = into_bytes(&sha1_of_file_contents).expect("sha1 of test torrent contents cannot be converted to bytes");

        Self {
            info: InfoDictWithCustomField {
                length: i64::try_from(file_contents.len()).expect("file contents size in bytes cannot exceed i64::MAX"),
                name: format!("file-{id}.txt"),
                piece_length: 16384,
                pieces: ByteBuf::from(pieces),
                custom: custom.to_owned(),
            },
        }
    }

    pub fn encode(torrent: &Self) -> Result<Vec<u8>, serde_bencode::Error> {
        match serde_bencode::to_bytes(torrent) {
            Ok(bencode_bytes) => Ok(bencode_bytes),
            Err(e) => {
                eprintln!("{e:?}");
                Err(e)
            }
        }
    }
}
