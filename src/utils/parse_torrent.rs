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

#[cfg(test)]
mod tests {
    use std::path::Path;

    #[test]
    fn it_should_ignore_non_standard_fields_in_info_dictionary() {
        let torrent_path = Path::new(
            // cspell:disable-next-line
            "tests/fixtures/torrents/6c690018c5786dbbb00161f62b0712d69296df97_with_custom_info_dict_key.torrent",
        );

        let torrent = super::decode_torrent(&std::fs::read(torrent_path).unwrap()).unwrap();

        // The infohash is not the original infohash of the torrent file, but the infohash of the
        // info dictionary without the custom keys.
        assert_eq!(torrent.info_hash(), "8aa01a4c816332045ffec83247ccbc654547fedf".to_string());
    }
}
