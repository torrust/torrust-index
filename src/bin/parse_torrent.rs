//! Command line tool to parse a torrent file and print the decoded torrent.
//!
//! It's only used for debugging purposes. ADR-T-010 classifies it as a stdout
//! result-data command; after migration it will emit one JSON object on stdout,
//! refuse direct terminal stdout, and report diagnostics on stderr as JSON. The
//! current implementation is a legacy output gap until migration.
use std::env;
use std::fs::File;
use std::io::{self, Read};
use std::process::ExitCode;

use serde_bencode::de::from_bytes;
use serde_bencode::value::Value as BValue;
use torrust_index::utils::parse_torrent;
use torrust_index_cli_common::{CommandExit, ControlPlaneRecord, emit_control_plane_record, install_json_panic_hook};

const COMMAND_NAME: &str = "parse_torrent";

fn main() -> ExitCode {
    install_json_panic_hook(COMMAND_NAME);

    match run() {
        Ok(exit) => exit.exit_code(),
        Err(error) => {
            let record = ControlPlaneRecord::diagnostic(COMMAND_NAME, &error.to_string());
            let _ignored = emit_control_plane_record(&record);
            CommandExit::Failure.exit_code()
        }
    }
}

fn run() -> io::Result<CommandExit> {
    let args: Vec<String> = env::args().collect();
    if args.len() != 2 {
        eprintln!("Usage:   cargo run --bin parse_torrent <PATH_TO_TORRENT_FILE>");
        eprintln!(
            "Example: cargo run --bin parse_torrent ./tests/fixtures/torrents/MC_GRID.zip-3cd18ff2d3eec881207dcc5ca5a2c3a2a3afe462.torrent"
        );
        return Ok(CommandExit::Usage);
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
                Ok(CommandExit::Success)
            }
            Err(e) => Err(io::Error::other(format!("Error: invalid torrent!. {e}"))),
        },
        Err(e) => Err(io::Error::other(format!("Error: invalid bencode data!. {e}"))),
    }
}
