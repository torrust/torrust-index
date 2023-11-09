//! Utility functions for torrent files.
//!
//! It's a wrapper around the [imdl](https://crates.io/crates/imdl) program.
use std::path::{Path, PathBuf};
use std::process::Command;

use serde::Deserialize;
use which::which;

/// Attributes parsed from a torrent file.
#[derive(Deserialize, Clone, Debug)]
pub struct TorrentFileInfo {
    pub name: String,
    pub comment: Option<String>,
    pub creation_date: Option<i64>,
    pub created_by: Option<String>,
    pub encoding: Option<String>,
    pub source: Option<String>,
    pub info_hash: String,
    pub torrent_size: u64,
    pub content_size: u64,
    pub private: bool,
    pub tracker: Option<String>,
    pub announce_list: Option<Vec<Vec<String>>>,
    pub update_url: Option<String>,
    pub dht_nodes: Option<Vec<String>>,
    pub piece_size: u64,
    pub piece_count: u64,
    pub file_count: u64,
    pub files: Vec<String>,
}

/// Creates a torrent file for the given file.
/// This function requires the `imdl` program to be installed.
/// <https://crates.io/crates/imdl>
pub fn create_torrent(dir: &Path, file_name: &str) -> PathBuf {
    guard_that_torrent_edition_cmd_is_installed();

    let input_file_path = Path::new(dir).join(file_name);
    let output_file_path = Path::new(dir).join(format!("{file_name}.torrent"));

    let _output = Command::new("imdl")
        .args(["torrent", "create", "--show"])
        .args(["--input", &format!("{}", input_file_path.to_string_lossy())])
        .args(["--output", &format!("{}", output_file_path.to_string_lossy())])
        .output()
        .unwrap_or_else(|_| panic!("failed to create torrent file: {:?}", output_file_path.to_string_lossy()));

    //io::stdout().write_all(&output.stdout).unwrap();
    //io::stderr().write_all(&output.stderr).unwrap();

    output_file_path
}

/// Parses torrent file.
/// This function requires the `imdl` program to be installed.
/// <https://crates.io/crates/imdl>
pub fn parse_torrent(torrent_file_path: &Path) -> TorrentFileInfo {
    guard_that_torrent_edition_cmd_is_installed();

    let output = Command::new("imdl")
        .args(["torrent", "show", "--json", &torrent_file_path.to_string_lossy()])
        .output()
        .unwrap_or_else(|_| panic!("failed to open torrent file: {:?}", &torrent_file_path.to_string_lossy()));

    match std::str::from_utf8(&output.stdout) {
        Ok(parsed_torrent_json) => {
            let res: TorrentFileInfo = serde_json::from_str(parsed_torrent_json).unwrap();
            res
        }
        Err(err) => panic!("got non UTF-8 data from 'imdl'. Error: {err}"),
    }
}

/// It panics if the `imdl` console application is not installed.
fn guard_that_torrent_edition_cmd_is_installed() {
    const IMDL_BINARY: &str = "imdl";
    match which(IMDL_BINARY) {
        Ok(_path) => (),
        Err(err) => {
            panic!("Can't create torrent with \"imdl\": {err}. Please install it with: `cargo install imdl`");
        }
    }
}
