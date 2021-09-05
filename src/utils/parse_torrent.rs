use std::{fs, error};
use serde_bencode::{de, Error};
use crate::models::torrent_file::Torrent;
use crate::errors::ServiceError;

pub fn read_torrent(path: &str) -> Result<Torrent, Box<dyn error::Error>> {
    let contents = match fs::read_to_string(path) {
        Ok(contents) => contents,
        Err(e) => return Err(e.into()),
    };

    match de::from_str::<Torrent>(&contents) {
        Ok(torrent) => Ok(torrent),
        Err(e) => Err(e.into()),
    }
}

pub fn decode_torrent(bytes: &[u8]) -> Result<Torrent, Box<dyn error::Error>> {
    match de::from_bytes::<Torrent>(&bytes) {
        Ok(torrent) => Ok(torrent),
        Err(e) => Err(e.into()),
    }
}

pub fn encode_torrent(torrent: &Torrent) -> Result<Vec<u8>, Error> {
    match serde_bencode::to_bytes(torrent) {
        Ok(bencode_bytes) => Ok(bencode_bytes),
        Err(e) => {
            println!("{:?}", e);
            Err(e)
        }
    }
}

pub fn encode_torrent_as_string(torrent: &Torrent) -> Result<String, Error> {
    let bencode_bytes = encode_torrent(torrent)?;

    match serde_bencode::to_string(&bencode_bytes) {
        Ok(bencode_string) => Ok(bencode_string),
        Err(e) => {
            println!("{:?}", e);
            Err(e)
        }
    }
}
