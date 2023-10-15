//! Command line tool to parse a torrent file and print the decoded torrent.
//!
//! It's only used for debugging purposes.
use std::env;
use std::fs::File;
use std::io::{self, Read};

use serde_bencode::de::from_bytes;
use serde_bencode::value::Value as BValue;
use torrust_index::utils::parse_torrent;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:   cargo run --bin parse_torrent <PATH_TO_TORRENT_FILE>");
        eprintln!("Example: cargo run --bin parse_torrent ./tests/fixtures/torrents/MC_GRID.zip-3cd18ff2d3eec881207dcc5ca5a2c3a2a3afe462.torrent");
        std::process::exit(1);
    }

    println!("Reading the torrent file ...");

    let mut file = File::open(&args[1])?;
    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)?;

    println!("Decoding torrent with standard serde implementation ...");

    match from_bytes::<BValue>(&bytes) {
        Ok(_value) => match parse_torrent::decode_torrent(&bytes) {
            Ok(torrent) => {
                println!("Parsed torrent: \n{torrent:#?}");
                Ok(())
            }
            Err(e) => Err(io::Error::new(io::ErrorKind::Other, format!("Error: invalid torrent!. {e}"))),
        },
        Err(e) => Err(io::Error::new(
            io::ErrorKind::Other,
            format!("Error: invalid bencode data!. {e}"),
        )),
    }
}
