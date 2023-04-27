use std::error;

use serde_bencode::{de, Error};

use crate::models::torrent_file::Torrent;

/// Decode a Torrent from Bencoded Bytes
///
/// # Errors
///
/// This function will return an error if unable to parse bytes into torrent.
pub fn decode_torrent(bytes: &[u8]) -> Result<Torrent, Box<dyn error::Error>> {
    match de::from_bytes::<Torrent>(bytes) {
        Ok(torrent) => Ok(torrent),
        Err(e) => {
            println!("{e:?}");
            Err(e.into())
        }
    }
}

/// Encode a Torrent into Bencoded Bytes
///
/// # Errors
///
/// This function will return an error if unable to bencode torrent.
pub fn encode_torrent(torrent: &Torrent) -> Result<Vec<u8>, Error> {
    match serde_bencode::to_bytes(torrent) {
        Ok(bencode_bytes) => Ok(bencode_bytes),
        Err(e) => {
            eprintln!("{e:?}");
            Err(e)
        }
    }
}
