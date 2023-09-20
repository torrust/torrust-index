use std::error;

use derive_more::{Display, Error};
use serde::{self, Deserialize, Serialize};
use serde_bencode::value::Value;
use serde_bencode::{de, Error};
use sha1::{Digest, Sha1};

use crate::models::info_hash::InfoHash;
use crate::models::torrent_file::Torrent;

#[derive(Debug, Display, PartialEq, Eq, Error)]
pub enum DecodeTorrentFileError {
    #[display(fmt = "Torrent data could not be decoded from the bencoded format.")]
    InvalidBencodeData,

    #[display(fmt = "Torrent info dictionary key could not be decoded from the bencoded format.")]
    InvalidInfoDictionary,

    #[display(fmt = "Torrent has an invalid pieces key length. It should be a multiple of 20.")]
    InvalidTorrentPiecesLength,

    #[display(fmt = "Cannot bencode the parsed `info` dictionary again to generate the info-hash.")]
    CannotBencodeInfoDict,
}

/// It decodes and validate an array of bytes containing a torrent file.
///
/// It returns a tuple containing the decoded torrent and the original info hash.
/// The original info-hash migth not match the new one in the `Torrent` because
/// the info dictionary might have been modified. For example, ignoring some
/// non-standard fields.
///
/// # Errors
///
/// This function will return an error if
///
/// - The torrent file is not a valid bencoded file.
/// - The pieces key has a length that is not a multiple of 20.
pub fn decode_and_validate_torrent_file(bytes: &[u8]) -> Result<(Torrent, InfoHash), DecodeTorrentFileError> {
    let original_info_hash = calculate_info_hash(bytes)?;

    let torrent = decode_torrent(bytes).map_err(|_| DecodeTorrentFileError::InvalidBencodeData)?;

    // Make sure that the pieces key has a length that is a multiple of 20
    if let Some(pieces) = torrent.info.pieces.as_ref() {
        if pieces.as_ref().len() % 20 != 0 {
            return Err(DecodeTorrentFileError::InvalidTorrentPiecesLength);
        }
    }

    Ok((torrent, original_info_hash))
}

/// Decode a Torrent from Bencoded Bytes.
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

/// Encode a Torrent into Bencoded Bytes.
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

#[derive(Serialize, Deserialize, Debug, PartialEq)]
struct ParsedInfoDictFromMetainfoFile {
    pub info: Value,
}

/// Calculates the `InfoHash` from a the torrent file binary data.
///
/// # Errors
///
/// This function will return an error if:
///
/// - The torrent file is not a valid bencoded torrent file containing an `info`
///  dictionary key.
/// - The original torrent info-hash cannot be bencoded from the parsed `info`
/// dictionary is not a valid bencoded dictionary.
pub fn calculate_info_hash(bytes: &[u8]) -> Result<InfoHash, DecodeTorrentFileError> {
    // Extract the info dictionary
    let metainfo: ParsedInfoDictFromMetainfoFile =
        serde_bencode::from_bytes(bytes).map_err(|_| DecodeTorrentFileError::InvalidInfoDictionary)?;

    // Bencode the info dictionary
    let info_dict_bytes = serde_bencode::to_bytes(&metainfo.info).map_err(|_| DecodeTorrentFileError::CannotBencodeInfoDict)?;

    // Calculate the SHA-1 hash of the bencoded info dictionary
    let mut hasher = Sha1::new();
    hasher.update(&info_dict_bytes);
    let result = hasher.finalize();

    Ok(InfoHash::from_bytes(&result))
}

#[cfg(test)]
mod tests {
    use std::path::Path;
    use std::str::FromStr;

    use crate::models::info_hash::InfoHash;

    #[test]
    fn it_should_calculate_the_original_info_hash_using_all_fields_in_the_info_key_dictionary() {
        let torrent_path = Path::new(
            // cspell:disable-next-line
            "tests/fixtures/torrents/6c690018c5786dbbb00161f62b0712d69296df97_with_custom_info_dict_key.torrent",
        );

        let original_info_hash = super::calculate_info_hash(&std::fs::read(torrent_path).unwrap()).unwrap();

        assert_eq!(
            original_info_hash,
            InfoHash::from_str("6c690018c5786dbbb00161f62b0712d69296df97").unwrap()
        );
    }

    #[test]
    fn it_should_calculate_the_new_info_hash_ignoring_non_standard_fields_in_the_info_key_dictionary() {
        let torrent_path = Path::new(
            // cspell:disable-next-line
            "tests/fixtures/torrents/6c690018c5786dbbb00161f62b0712d69296df97_with_custom_info_dict_key.torrent",
        );

        let torrent = super::decode_torrent(&std::fs::read(torrent_path).unwrap()).unwrap();

        // The infohash is not the original infohash of the torrent file,
        // but the infohash of the info dictionary without the custom keys.
        assert_eq!(
            torrent.canonical_info_hash_hex(),
            "8aa01a4c816332045ffec83247ccbc654547fedf".to_string()
        );
    }
}
