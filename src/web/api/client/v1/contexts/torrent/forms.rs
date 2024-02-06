use std::fs;
use std::path::Path;

use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct UpdateTorrentFrom {
    pub title: Option<String>,
    pub description: Option<String>,
    pub category: Option<i64>,
    pub tags: Option<Vec<i64>>,
}

use reqwest::multipart::Form;

pub struct UploadTorrentMultipartForm {
    pub title: String,
    pub description: String,
    pub category: String,
    pub torrent_file: BinaryFile,
}

#[derive(Clone)]
pub struct BinaryFile {
    pub name: String,
    pub contents: Vec<u8>,
}

impl BinaryFile {
    /// # Panics
    /// 
    /// Will panic if:
    /// 
    /// - The path is not a file.
    /// - The path can't be converted into string.
    /// - The file can't be read.
    #[must_use]
    pub fn from_file_at_path(path: &Path) -> Self {
        BinaryFile {
            name: path.file_name().unwrap().to_owned().into_string().unwrap(),
            contents: fs::read(path).unwrap(),
        }
    }
}

impl From<UploadTorrentMultipartForm> for Form {
    fn from(form: UploadTorrentMultipartForm) -> Self {
        Form::new()
            .text("title", form.title)
            .text("description", form.description)
            .text("category", form.category)
            .part(
                "torrent",
                reqwest::multipart::Part::bytes(form.torrent_file.contents)
                    .file_name(form.torrent_file.name)
                    .mime_str("application/x-bittorrent")
                    .unwrap(),
            )
    }
}
