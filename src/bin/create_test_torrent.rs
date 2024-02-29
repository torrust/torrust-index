//! Command line tool to create a test torrent file.
//!
//! It's only used for debugging purposes.
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::Path;

use torrust_index::models::torrent_file::{Torrent, TorrentFile, TorrentInfoDictionary};
use torrust_index::services::hasher::sha1; // DevSkim: ignore DS126858
use torrust_index::utils::parse_torrent;
use uuid::Uuid;

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 2 {
        eprintln!("Usage:   cargo run --bin create_test_torrent <destination_folder>");
        eprintln!("Example: cargo run --bin create_test_torrent ./output/test/torrents");
        std::process::exit(1);
    }

    let destination_folder = &args[1];

    let id = Uuid::new_v4();

    // Content of the file from which the torrent will be generated.
    // We use the UUID as the content of the file.
    let file_contents = format!("{id}\n");
    let file_name = format!("file-{id}.txt");

    let torrent = Torrent {
        info: TorrentInfoDictionary::with(
            &file_name,
            16384,
            None,
            0,
            &sha1(&file_contents), // DevSkim: ignore DS126858
            &[TorrentFile {
                path: vec![file_name.clone()], // Adjusted to include the actual file name
                length: i64::try_from(file_contents.len()).expect("file contents size in bytes cannot exceed i64::MAX"),
                md5sum: None, // DevSkim: ignore DS126858
            }],
        ),
        announce: None,
        nodes: Some(vec![("99.236.6.144".to_string(), 6881), ("91.109.195.156".to_string(), 1996)]),
        encoding: None,
        httpseeds: Some(vec!["https://seeder.torrust-demo.com/seed".to_string()]),
        announce_list: Some(vec![vec!["https://tracker.torrust-demo.com/announce".to_string()]]),
        creation_date: None,
        comment: None,
        created_by: None,
    };

    match parse_torrent::encode_torrent(&torrent) {
        Ok(bytes) => {
            // Construct the path where the torrent file will be saved
            let file_path = Path::new(destination_folder).join(format!("{file_name}.torrent"));

            // Attempt to create and write to the file
            let mut file = match File::create(&file_path) {
                Ok(file) => file,
                Err(e) => panic!("Failed to create file {file_path:?}: {e}"),
            };

            if let Err(e) = file.write_all(&bytes) {
                panic!("Failed to write to file {file_path:?}: {e}");
            }

            println!("File successfully written to {file_path:?}");
        }
        Err(e) => panic!("Error encoding torrent: {e}"),
    };
}
