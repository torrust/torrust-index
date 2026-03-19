use serde_bytes::ByteBuf;
use uuid::Uuid;

use crate::models::torrent_file::{Torrent, TorrentInfoDictionary};
use crate::services::torrent_file::generate_random_torrent;

#[test]
fn it_should_generate_a_random_meta_info_file() {
    let uuid = Uuid::parse_str("d6170378-2c14-4ccc-870d-2a8e15195e23").unwrap();

    let torrent = generate_random_torrent(uuid);

    let expected_torrent = Torrent {
        info: TorrentInfoDictionary {
            name: "file-d6170378-2c14-4ccc-870d-2a8e15195e23.txt".to_string(),
            pieces: Some(ByteBuf::from(vec![
                62, 231, 243, 51, 234, 165, 204, 209, 51, 132, 163, 133, 249, 50, 107, 46, 24, 15, 251, 32,
            ])),
            piece_length: 16384,
            md5sum: None,
            length: Some(37),
            files: None,
            private: None,
            path: None,
            root_hash: None,
            source: None,
        },
        announce: None,
        announce_list: Some(vec![]),
        creation_date: None,
        comment: None,
        created_by: None,
        nodes: None,
        encoding: None,
        httpseeds: None,
    };

    assert_eq!(torrent, expected_torrent);
}
