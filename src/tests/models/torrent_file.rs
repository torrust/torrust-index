mod info_hash_calculation_for_version_v1 {

    use serde_bytes::ByteBuf;

    use crate::models::torrent_file::{Torrent, TorrentInfoDictionary};

    #[test]
    fn the_parsed_torrent_file_should_calculated_the_torrent_info_hash() {
        /* The sample.txt content (`mandelbrot`):

           ```
           6d616e64656c62726f740a
           ```

           The sample.txt.torrent content:

           ```
           6431303a6372656174656420627931383a71426974746f7272656e742076
           342e352e3431333a6372656174696f6e2064617465693136393131343935
           373265343a696e666f64363a6c656e67746869313165343a6e616d653130
           3a73616d706c652e74787431323a7069656365206c656e67746869313633
           383465363a70696563657332303ad491587f1c42dff0cb0ff5c2b8cefe22
           b3ad310a6565
           ```

           ```json
           {
             "created by": "qBittorrent v4.5.4",
             "creation date": 1691149572,
             "info": {
               "length": 11,
               "name": "sample.txt",
               "piece length": 16384,
               "pieces": "<hex>D4 91 58 7F 1C 42 DF F0 CB 0F F5 C2 B8 CE FE 22 B3 AD 31 0A</hex>"
             }
           }
           ```
        */

        let sample_data_in_txt_file = "mandelbrot\n";

        let info = TorrentInfoDictionary {
            name: "sample.txt".to_string(),
            pieces: Some(ByteBuf::from(vec![
                // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
            ])),
            piece_length: 16384,
            md5sum: None,
            length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
            files: None,
            private: None,
            path: None,
            root_hash: None,
            source: None,
        };

        let torrent = Torrent {
            info,
            announce: None,
            announce_list: Some(vec![]),
            creation_date: None,
            comment: None,
            created_by: None,
            nodes: None,
            encoding: None,
            httpseeds: None,
        };

        assert_eq!(torrent.canonical_info_hash_hex(), "79fa9e4a2927804fe4feab488a76c8c2d3d1cdca");
    }

    mod infohash_should_be_calculated_for {

        use serde_bytes::ByteBuf;

        use crate::models::torrent_file::{Torrent, TorrentFile, TorrentInfoDictionary};

        #[test]
        fn a_simple_single_file_torrent() {
            let sample_data_in_txt_file = "mandelbrot\n";

            let info = TorrentInfoDictionary {
                name: "sample.txt".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                    212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                ])),
                piece_length: 16384,
                md5sum: None,
                length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                files: None,
                private: None,
                path: None,
                root_hash: None,
                source: None,
            };

            let torrent = Torrent {
                info,
                announce: None,
                announce_list: Some(vec![]),
                creation_date: None,
                comment: None,
                created_by: None,
                nodes: None,
                encoding: None,
                httpseeds: None,
            };

            assert_eq!(torrent.canonical_info_hash_hex(), "79fa9e4a2927804fe4feab488a76c8c2d3d1cdca");
        }

        #[test]
        fn a_simple_multi_file_torrent() {
            let sample_data_in_txt_file = "mandelbrot\n";

            let info = TorrentInfoDictionary {
                name: "sample".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                    212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                ])),
                piece_length: 16384,
                md5sum: None,
                length: None,
                files: Some(vec![TorrentFile {
                    path: vec!["sample.txt".to_string()],
                    length: sample_data_in_txt_file.len().try_into().unwrap(),
                    md5sum: None,
                }]),
                private: None,
                path: None,
                root_hash: None,
                source: None,
            };

            let torrent = Torrent {
                info,
                announce: None,
                announce_list: Some(vec![]),
                creation_date: None,
                comment: None,
                created_by: None,
                nodes: None,
                encoding: None,
                httpseeds: None,
            };

            assert_eq!(torrent.canonical_info_hash_hex(), "aa2aca91ab650c4d249c475ca3fa604f2ccb0d2a");
        }

        #[test]
        fn a_simple_single_file_torrent_with_a_source() {
            let sample_data_in_txt_file = "mandelbrot\n";

            let info = TorrentInfoDictionary {
                name: "sample.txt".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                    212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                ])),
                piece_length: 16384,
                md5sum: None,
                length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                files: None,
                private: None,
                path: None,
                root_hash: None,
                source: Some("ABC".to_string()), // The tracker three-letter code
            };

            let torrent = Torrent {
                info,
                announce: None,
                announce_list: Some(vec![]),
                creation_date: None,
                comment: None,
                created_by: None,
                nodes: None,
                encoding: None,
                httpseeds: None,
            };

            assert_eq!(torrent.canonical_info_hash_hex(), "ccc1cf4feb59f3fa85c96c9be1ebbafcfe8a9cc8");
        }

        #[test]
        fn a_simple_single_file_private_torrent() {
            let sample_data_in_txt_file = "mandelbrot\n";

            let info = TorrentInfoDictionary {
                name: "sample.txt".to_string(),
                pieces: Some(ByteBuf::from(vec![
                    // D4 91  58   7F  1C  42   DF   F0   CB  0F   F5   C2   B8   CE   FE  22   B3   AD  31  0A  // hex
                    212, 145, 88, 127, 28, 66, 223, 240, 203, 15, 245, 194, 184, 206, 254, 34, 179, 173, 49, 10, // dec
                ])),
                piece_length: 16384,
                md5sum: None,
                length: Some(sample_data_in_txt_file.len().try_into().unwrap()),
                files: None,
                private: Some(1),
                path: None,
                root_hash: None,
                source: None,
            };

            let torrent = Torrent {
                info,
                announce: None,
                announce_list: Some(vec![]),
                creation_date: None,
                comment: None,
                created_by: None,
                nodes: None,
                encoding: None,
                httpseeds: None,
            };

            assert_eq!(torrent.canonical_info_hash_hex(), "d3a558d0a19aaa23ba6f9f430f40924d10fefa86");
        }
    }
}
