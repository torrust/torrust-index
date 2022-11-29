use std::error;

use serde_bencode::{de, Error};

use crate::models::torrent_file::Torrent;

pub fn decode_torrent(bytes: &[u8]) -> Result<Torrent, Box<dyn error::Error>> {
    match de::from_bytes::<Torrent>(bytes) {
        Ok(torrent) => Ok(torrent),
        Err(e) => {
            println!("{:?}", e);
            Err(e.into())
        }
    }
}

pub fn encode_torrent(torrent: &Torrent) -> Result<Vec<u8>, Error> {
    match serde_bencode::to_bytes(torrent) {
        Ok(bencode_bytes) => Ok(bencode_bytes),
        Err(e) => {
            eprintln!("{:?}", e);
            Err(e)
        }
    }
}
