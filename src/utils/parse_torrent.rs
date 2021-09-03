use std::{fs, error};
use serde_bencode::{de, Error};

use crate::errors::ServiceError;
use crate::models::Torrent;

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