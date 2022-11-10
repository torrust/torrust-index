use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v1_0_0::SqliteDatabaseV1_0_0;
use crate::upgrades::from_v1_0_0_to_v2_0_0::sqlite_v2_0_0::SqliteDatabaseV2_0_0;
use std::sync::Arc;
use torrust_index_backend::models::torrent_file::Torrent;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v1_0_0::{
    TorrentRecordV1, UserRecordV1,
};
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::databases::sqlite_v2_0_0::convert_timestamp_to_datetime;
use torrust_index_backend::upgrades::from_v1_0_0_to_v2_0_0::upgrader::read_torrent_from_file;

pub struct TorrentTester {
    source_database: Arc<SqliteDatabaseV1_0_0>,
    destiny_database: Arc<SqliteDatabaseV2_0_0>,
    test_data: TestData,
}

pub struct TestData {
    pub torrent: TorrentRecordV1,
    pub user: UserRecordV1,
}

impl TorrentTester {
    pub fn new(
        source_database: Arc<SqliteDatabaseV1_0_0>,
        destiny_database: Arc<SqliteDatabaseV2_0_0>,
        user: &UserRecordV1,
    ) -> Self {
        let torrent = TorrentRecordV1 {
            torrent_id: 1,
            uploader: user.username.clone(),
            info_hash: "9e0217d0fa71c87332cd8bf9dbeabcb2c2cf3c4d".to_string(),
            title: "title".to_string(),
            category_id: 1,
            description: "description".to_string(),
            upload_date: 1667546358, // 2022-11-04 07:19:18
            file_size: 9219566,
            seeders: 0,
            leechers: 0,
        };

        Self {
            source_database,
            destiny_database,
            test_data: TestData {
                torrent,
                user: user.clone(),
            },
        }
    }

    pub async fn load_data_into_source_db(&self) {
        self.source_database
            .insert_torrent(&self.test_data.torrent)
            .await
            .unwrap();
    }

    pub async fn assert(&self, upload_path: &str) {
        let filepath = self.torrent_file_path(upload_path, self.test_data.torrent.torrent_id);
        let torrent_file = read_torrent_from_file(&filepath).unwrap();

        self.assert_torrent(&torrent_file).await;
        // TODO
        // `torrust_torrent_files`,
        // `torrust_torrent_info`
        // `torrust_torrent_announce_urls`
    }

    pub fn torrent_file_path(&self, upload_path: &str, torrent_id: i64) -> String {
        format!("{}/{}.torrent", &upload_path, &torrent_id)
    }

    /// Table `torrust_torrents`
    async fn assert_torrent(&self, torrent_file: &Torrent) {
        let imported_torrent = self
            .destiny_database
            .get_torrent(self.test_data.torrent.torrent_id)
            .await
            .unwrap();

        assert_eq!(
            imported_torrent.torrent_id,
            self.test_data.torrent.torrent_id
        );
        assert_eq!(imported_torrent.uploader_id, self.test_data.user.user_id);
        assert_eq!(
            imported_torrent.category_id,
            self.test_data.torrent.category_id
        );
        assert_eq!(imported_torrent.info_hash, self.test_data.torrent.info_hash);
        assert_eq!(imported_torrent.size, self.test_data.torrent.file_size);
        assert_eq!(imported_torrent.name, torrent_file.info.name);
        assert_eq!(
            imported_torrent.pieces,
            torrent_file.info.get_pieces_as_string()
        );
        assert_eq!(
            imported_torrent.piece_length,
            torrent_file.info.piece_length
        );
        if torrent_file.info.private.is_none() {
            assert_eq!(imported_torrent.private, Some(0));
        } else {
            assert_eq!(imported_torrent.private, torrent_file.info.private);
        }
        assert_eq!(
            imported_torrent.root_hash,
            torrent_file.info.get_root_hash_as_i64()
        );
        assert_eq!(
            imported_torrent.date_uploaded,
            convert_timestamp_to_datetime(self.test_data.torrent.upload_date)
        );
    }
}
